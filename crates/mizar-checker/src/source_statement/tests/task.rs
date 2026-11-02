use super::*;

#[test]
fn task_258b5b_imported_public_theorem_citation_profile_and_debug_are_exact() {
    let fixture = B5BFixture::new(2580);
    let references = fixture
        .references(fixture.reference_input())
        .expect("Task258B5B references");
    assert!(fixture.statement.is_task_258b5b_profile());
    assert!(exact_task258b5b_shared_arena(
        fixture.source,
        &fixture.arena
    ));
    for formula_index in 0..2 {
        let formula_site = fixture
            .atomic
            .formulas()
            .get(SourceAtomicFormulaId::new(formula_index))
            .expect("Task258B5B formula")
            .site()
            .node();
        let left_site = fixture
            .primary
            .terms()
            .get(SourcePrimaryTermId::new(formula_index * 2))
            .expect("Task258B5B left operand")
            .site()
            .node();
        let right_site = fixture
            .primary
            .terms()
            .get(SourcePrimaryTermId::new(formula_index * 2 + 1))
            .expect("Task258B5B right operand")
            .site()
            .node();
        assert_eq!(
            containing_child_position(&fixture.arena, formula_site, left_site),
            Some(0)
        );
        assert_eq!(
            containing_child_position(&fixture.arena, formula_site, right_site),
            Some(0)
        );
        assert!(
            containing_child_path(&fixture.arena, formula_site, left_site)
                < containing_child_path(&fixture.arena, formula_site, right_site)
        );
    }
    assert_eq!(fixture.arena.len(), 57);
    assert_eq!(fixture.arena.root(), Some(TypedNodeId::new(56)));
    assert_eq!(
        fixture
            .arena
            .iter()
            .filter(|(_, row)| row.kind.as_str() != "source.surface.unowned")
            .count(),
        8
    );
    assert_eq!(fixture.bindings.contexts().len(), 2);
    assert_eq!(fixture.primary.terms().len(), 4);
    assert_eq!(fixture.atomic.formulas().len(), 2);
    assert_eq!(fixture.statement.statements().len(), 2);
    assert!(references.labels().is_empty());
    assert_eq!(references.citations().len(), 1);
    let citation = references
        .citations()
        .get(SourceStatementCitationId::new(0))
        .expect("imported citation");
    assert_eq!(citation.statement(), SourceStatementId::new(1));
    assert_eq!(citation.context(), SourceStatementContextId::new(1));
    assert_eq!(citation.target(), SourceStatementCitationTarget::Imported);
    assert_eq!(citation.label_ref(), fixture.resolution.ids()[0]);
    assert_eq!(citation.proof_scope().path(), [0]);
    assert_eq!(citation.source_range(), range(fixture.source, 136, 139));
    assert_eq!(citation.ordinal(), 0);
    assert_eq!(citation.kind(), SourceStatementCitationKind::SimpleImported);
    assert_eq!(citation.recovery(), SourceStatementRecovery::Normal);
    let expected = b5b_expected_reference_debug(&references);
    assert_eq!(references.debug_text(), expected);
    assert!(!references.debug_text().contains("\nlabel#0 "));
}

#[test]
fn task_258b5b_imported_provenance_and_row_corruption_fail_atomically() {
    let fixture = B5BFixture::new(2581);
    let baseline = fixture
        .references(fixture.reference_input())
        .expect("Task258B5B baseline");

    let mut local_target = fixture.reference_input();
    local_target.citations[0].target =
        SourceStatementCitationTarget::Local(SourceStatementLabelId::new(0));
    assert_eq!(
        fixture.references(local_target),
        Err(SourceStatementReferenceError::InvalidCitation {
            citation: SourceStatementCitationId::new(0)
        })
    );
    let mut wrong_kind = fixture.reference_input();
    wrong_kind.citations[0].kind = SourceStatementCitationKind::SimpleLocal;
    assert_eq!(
        fixture.references(wrong_kind),
        Err(SourceStatementReferenceError::InvalidCitation {
            citation: SourceStatementCitationId::new(0)
        })
    );
    let reexported = fixture
        .projection
        .clone()
        .with_export_status(ExportStatus::ReExported);
    let namespace = NamespacePath::new(fixture.module.path().as_str());
    let reexported_resolution = LabelResolver::new(std::slice::from_ref(&reexported)).resolve(
        &fixture.module,
        &namespace,
        std::slice::from_ref(&fixture.reference),
    );
    let reexported_ast = b5b_resolved_ast(
        fixture.source,
        &fixture.module,
        Some(reexported_resolution.ids()[0]),
        reexported_resolution.table().clone(),
    );
    let mut input = fixture.reference_input();
    input.citations[0].label_ref = reexported_resolution.ids()[0];
    assert_eq!(
        SourceStatementReferenceProducer::build(
            input,
            &fixture.statement,
            &reexported_ast,
            &reexported,
            &fixture.reference,
            &reexported_resolution,
            &fixture.arena,
        ),
        Err(SourceStatementReferenceError::InvalidLabel {
            label: SourceStatementLabelId::new(0)
        })
    );
    let imported_module = ModuleId::new(
        fixture.module.package().clone(),
        ModulePath::new("parser.type_fixtures"),
    );
    let wrong_origin = LabelProjection::imported(mizar_resolve::labels::LabelProjectionData {
        origin_path: fixture.projection.origin_path().clone(),
        module: imported_module.clone(),
        namespace,
        primary_spelling: "Ref".to_owned(),
        kind: LabelKind::Theorem,
        declaration_range: range(fixture.source, 7, 27),
        origin: SemanticOrigin::new(
            fixture.source,
            imported_module,
            SourceAnchor::Range(range(fixture.source, 7, 27)),
            vec![1, 1],
        ),
        contribution: fixture.projection.contribution(),
    })
    .with_export_status(ExportStatus::Exported);
    let wrong_resolution = LabelResolver::new(std::slice::from_ref(&wrong_origin)).resolve(
        &fixture.module,
        &NamespacePath::new(fixture.module.path().as_str()),
        std::slice::from_ref(&fixture.reference),
    );
    let wrong_ast = b5b_resolved_ast(
        fixture.source,
        &fixture.module,
        Some(wrong_resolution.ids()[0]),
        wrong_resolution.table().clone(),
    );
    let mut wrong_input = fixture.reference_input();
    wrong_input.citations[0].label_ref = wrong_resolution.ids()[0];
    assert!(
        SourceStatementReferenceProducer::build(
            wrong_input,
            &fixture.statement,
            &wrong_ast,
            &wrong_origin,
            &fixture.reference,
            &wrong_resolution,
            &fixture.arena,
        )
        .is_err()
    );
    let missing_key_ast =
        b5b_resolved_ast_with_node_mutation(&fixture, B5BResolverNodeMutation::MissingReferenceKey)
            .expect("missing node key remains structurally valid resolver output");
    assert_eq!(
        SourceStatementReferenceProducer::build(
            fixture.reference_input(),
            &fixture.statement,
            &missing_key_ast,
            &fixture.projection,
            &fixture.reference,
            &fixture.resolution,
            &fixture.arena,
        ),
        Err(SourceStatementReferenceError::DependencyMismatch)
    );
    assert!(matches!(
        b5b_resolved_ast_with_node_mutation(
            &fixture,
            B5BResolverNodeMutation::RelocatedReferenceKey,
        ),
        Err(mizar_resolve::resolved_ast::ResolvedAstError::NodeReferenceSiteMismatch {
            node,
            key: NodeReferenceKey::Label(label),
        }) if node.index() == 47 && label.index() == 0
    ));
    assert_eq!(
        baseline,
        fixture
            .references(fixture.reference_input())
            .expect("failed mutations leave replay unchanged")
    );
}

#[test]
fn task_258b5b_b5a_local_cross_profile_installation_is_atomic() {
    let imported = B5BFixture::new(2582);
    let b5a = B5AFixture::new(2582);
    let b1 = B1Fixture::new(2582);
    let imported_references = imported
        .references(imported.reference_input())
        .expect("Task258B5B references");
    let b5a_references = b5a
        .references(b5a.reference_input())
        .expect("Task258B5A references");
    let b1_references = b1
        .references(b1.reference_input())
        .expect("Task258B1 references");
    let imported_base = imported.empty_typed();
    let b5a_base = b5a.empty_typed();
    let b1_base = b1.empty_typed();
    for (label, base, statement, references) in [
        (
            "B5A pair on B5B base",
            &imported_base,
            b5a.statement.clone(),
            b5a_references.clone(),
        ),
        (
            "B5B pair on B5A base",
            &b5a_base,
            imported.statement.clone(),
            imported_references.clone(),
        ),
        (
            "B1 pair on B5B base",
            &imported_base,
            b1.statement.clone(),
            b1_references.clone(),
        ),
        (
            "B5B pair on B1 base",
            &b1_base,
            imported.statement.clone(),
            imported_references.clone(),
        ),
        (
            "B5B statement with B1 references on B5B base",
            &imported_base,
            imported.statement.clone(),
            b1_references.clone(),
        ),
        (
            "B1 statement with B5B references on B5B base",
            &imported_base,
            b1.statement.clone(),
            imported_references.clone(),
        ),
        (
            "B1 statement with B5B references on B1 base",
            &b1_base,
            b1.statement.clone(),
            imported_references.clone(),
        ),
        (
            "B5B statement with B1 references on B1 base",
            &b1_base,
            imported.statement.clone(),
            b1_references.clone(),
        ),
    ] {
        assert_eq!(
            base.clone()
                .with_source_statement_references(statement, references),
            Err(TypedAstError::InvalidSourceStatement),
            "{label}"
        );
    }
    assert_eq!(imported_base, imported.empty_typed());
    assert_eq!(b5a_base, b5a.empty_typed());
    assert_eq!(b1_base, b1.empty_typed());
}

#[test]
fn task_258b5b_final_clone_revalidates_imported_visibility_and_empty_semantics() {
    let fixture = B5BFixture::new(2583);
    let references = fixture
        .references(fixture.reference_input())
        .expect("Task258B5B references");
    let typed = fixture
        .empty_typed()
        .with_source_statement_references(fixture.statement.clone(), references.clone())
        .expect("Task258B5B typed AST");
    let resolved = assemble_empty_resolved(&typed).expect("Task258B5B final assembly");
    assert!(resolved.statement_semantics().is_empty());
    assert!(resolved.checked_formulas().is_empty());
    assert!(resolved.checked_proofs().is_empty());
    assert!(resolved.checked_proof_nodes().is_empty());
    assert!(resolved.checked_terminal_goals().is_empty());
    assert_eq!(resolved.clone(), resolved);
    let expected_reference_debug = b5b_expected_reference_debug(&references);
    assert_eq!(references.debug_text(), expected_reference_debug);
    assert_eq!(
        typed
            .source_statement_references()
            .expect("Task258B5B installed references")
            .debug_text(),
        expected_reference_debug
    );
    assert!(typed.debug_text().contains(&expected_reference_debug));
    assert!(resolved.debug_text().contains(&expected_reference_debug));
    assert!(expected_reference_debug.contains("label_node=absent"));
    assert!(expected_reference_debug.contains("source=imported"));
    assert!(!expected_reference_debug.contains("\nlabel#0 "));

    let mut relocated_import = references.clone();
    relocated_import.resolver_ast = b5b_resolved_ast_with_relocated_import(&fixture);
    assert_b5b_final_assembly_rejects(&fixture, relocated_import);
    let mut relocated_projection = references.clone();
    relocated_projection.label_projection = vec![b5b_projection_with_relocated_origin(&fixture)];
    assert_b5b_final_assembly_rejects(&fixture, relocated_projection);
    let mut reexported = references.clone();
    reexported.label_projection[0] = reexported.label_projection[0]
        .clone()
        .with_export_status(ExportStatus::ReExported);
    assert_b5b_final_assembly_rejects(&fixture, reexported);
    let mut relocated_reference = references.clone();
    relocated_reference.reference_candidate = b5b_reference_with_relocated_origin(&fixture);
    assert_b5b_final_assembly_rejects(&fixture, relocated_reference);
    let mut missing_key = references.clone();
    missing_key.resolver_ast =
        b5b_resolved_ast_with_node_mutation(&fixture, B5BResolverNodeMutation::MissingReferenceKey)
            .expect("missing key remains a structurally valid resolver AST");
    assert_b5b_final_assembly_rejects(&fixture, missing_key);
    let mut local_target = references;
    local_target.citations.rows[0].target =
        SourceStatementCitationTarget::Local(SourceStatementLabelId::new(0));
    assert_b5b_final_assembly_rejects(&fixture, local_target);
    assert_eq!(
        assemble_empty_resolved(&typed)
            .expect("Task258B5B replay")
            .debug_text(),
        resolved.debug_text()
    );
}

#[test]
fn task_258b5a_ancestor_label_descendant_citation_profile_and_debug_are_exact() {
    let fixture = B5AFixture::new(250);
    let references = fixture
        .references(fixture.reference_input())
        .expect("Task258B5A references");
    assert!(fixture.statement.is_task_258b5a_profile());
    assert!(exact_task258b5a_shared_arena(
        fixture.source,
        &fixture.arena
    ));
    assert_eq!(fixture.arena.len(), 93);
    assert_eq!(fixture.arena.root(), Some(TypedNodeId::new(92)));
    assert_eq!(fixture.bindings.contexts().len(), 4);
    assert_eq!(fixture.primary.terms().len(), 10);
    assert_eq!(fixture.atomic.formulas().len(), 5);
    assert_eq!(fixture.statement.statements().len(), 5);
    assert_eq!(references.labels().len(), 1);
    assert_eq!(references.citations().len(), 1);
    assert_eq!(
        references
            .labels()
            .get(SourceStatementLabelId::new(0))
            .expect("label")
            .proof_scope()
            .path(),
        [0]
    );
    assert_eq!(
        references
            .citations()
            .get(SourceStatementCitationId::new(0))
            .expect("citation")
            .proof_scope()
            .path(),
        [0, 1]
    );
    let debug = references.debug_text();
    for line in [
        "resolver-ast root=92 nodes=93 name_refs=0 label_refs=1 imports=0 exports=0 label_node=12 reference_node=82 reference_state=resolved reference_key=label#0\n",
        "resolver-projection origin=pkg::statement.fixture::proof::A namespace=statement.fixture range=95..96 visible_after=1 scope=[0] kind=proof-step visibility=private export=local-only spelling=\"A\"\n",
        "resolver-reference node=82 range=170..171 source_ordinal=4 scope=[0, 1] expectation=proof-or-theorem spelling=\"A\"\n",
        "resolver-result index=1 references=1 ids=[0] diagnostics=0\n",
        "label#0 statement=1 context=1 candidate=1 origin=pkg::statement.fixture::proof::A scope=[0] range=95..96 source_ordinal=0 visible_after=1 kind=proof-step recovery=normal spelling=\"A\"\n",
        "citation#0 statement=4 context=4 label=0 label_ref=0 scope=[0, 1] range=170..171 ordinal=0 kind=simple-local recovery=normal\n",
    ] {
        assert!(debug.contains(line), "missing exact debug line: {line:?}");
    }
    assert_eq!(references.clone(), references);
}

#[test]
fn task_258b5a_scope_provenance_and_row_corruption_fail_atomically() {
    let fixture = B5AFixture::new(251);
    let valid = fixture
        .references(fixture.reference_input())
        .expect("Task258B5A baseline");

    let mut invalid_statement = fixture.statement.clone();
    invalid_statement.contexts.rows[4].binding_context = BindingContextId::new(2);
    assert_eq!(
        invalid_statement.validate_installation(
            fixture.source,
            &fixture.module,
            &fixture.primary,
            &fixture.atomic,
            &fixture.arena,
        ),
        Err(SourceStatementError::InvalidContext {
            context: SourceStatementContextId::new(4)
        })
    );
    let mut invalid_facts = fixture.statement.clone();
    invalid_facts.input_facts.rows[4].uses.swap(0, 1);
    assert_eq!(
        invalid_facts.validate_installation(
            fixture.source,
            &fixture.module,
            &fixture.primary,
            &fixture.atomic,
            &fixture.arena,
        ),
        Err(SourceStatementError::InvalidInputFact {
            fact: SourceStatementInputFactId::new(4)
        })
    );
    let invalid_arena = mutate_arena(&fixture.arena, |id, row| {
        if id == TypedNodeId::new(82) {
            row.anchor = SourceAnchor::Range(range(fixture.source, 169, 171));
        }
    });
    assert_eq!(
        fixture.statement.validate_installation(
            fixture.source,
            &fixture.module,
            &fixture.primary,
            &fixture.atomic,
            &invalid_arena,
        ),
        Err(SourceStatementError::DependencyMismatch)
    );

    let mut invalid_label = fixture.reference_input();
    invalid_label.labels[0].proof_scope = LabelScopePath::new(vec![0, 0]);
    assert_eq!(
        fixture.references(invalid_label),
        Err(SourceStatementReferenceError::InvalidLabel {
            label: SourceStatementLabelId::new(0)
        })
    );
    let mut invalid_citation = fixture.reference_input();
    invalid_citation.citations[0].proof_scope = LabelScopePath::new(vec![0]);
    assert_eq!(
        fixture.references(invalid_citation),
        Err(SourceStatementReferenceError::InvalidCitation {
            citation: SourceStatementCitationId::new(0)
        })
    );
    let wrong_kind_resolver = ResolvedAst::try_new(
        fixture.source,
        fixture.module.clone(),
        b5a_resolved_arena_with_kind_mutation(
            fixture.source,
            &fixture.module,
            Some(fixture.resolution.ids()[0]),
            Some(0),
        ),
        mizar_resolve::resolved_ast::NameRefTable::new(),
        fixture.resolution.table().clone(),
        mizar_resolve::resolved_ast::ResolvedImports::new(),
    )
    .expect("coherent resolver kind mutation");
    assert_eq!(
        SourceStatementReferenceProducer::build(
            fixture.reference_input(),
            &fixture.statement,
            &wrong_kind_resolver,
            &fixture.projection,
            &fixture.reference,
            &fixture.resolution,
            &fixture.arena,
        ),
        Err(SourceStatementReferenceError::DependencyMismatch)
    );
    let mut stale = valid.clone();
    stale.statement_fingerprint.push_str(":stale");
    stale.labels.rows[0].visible_after_ordinal = 0;
    assert_eq!(
        stale.validate_installation(
            fixture.source,
            &fixture.module,
            &fixture.statement,
            &fixture.arena,
        ),
        Err(SourceStatementReferenceError::DependencyMismatch)
    );

    let pristine = fixture.clone();
    let typed = fixture
        .empty_typed()
        .with_source_statement_references(fixture.statement.clone(), valid.clone())
        .expect("Task258B5A pristine typed");
    let typed_debug = typed.debug_text();
    let resolved_debug = assemble_empty_resolved(&typed)
        .expect("Task258B5A pristine final")
        .debug_text();
    let replay = || {
        assert_eq!(
            (
                &fixture.bindings,
                &fixture.primary,
                &fixture.atomic,
                &fixture.arena,
                &fixture.statement,
                &fixture.resolver_ast,
                &fixture.projection,
                &fixture.reference,
                &fixture.resolution,
            ),
            (
                &pristine.bindings,
                &pristine.primary,
                &pristine.atomic,
                &pristine.arena,
                &pristine.statement,
                &pristine.resolver_ast,
                &pristine.projection,
                &pristine.reference,
                &pristine.resolution,
            ),
            "all pristine producer inputs remain unchanged"
        );
        let rebuilt = B5AFixture::new(251);
        assert_eq!(rebuilt.statement, pristine.statement);
        assert_eq!(rebuilt.resolver_ast, pristine.resolver_ast);
        let references = rebuilt
            .references(rebuilt.reference_input())
            .expect("reference replay");
        assert_eq!(references, valid);
        let typed = rebuilt
            .empty_typed()
            .with_source_statement_references(rebuilt.statement.clone(), references)
            .expect("typed replay");
        assert_eq!(typed.debug_text(), typed_debug);
        assert_eq!(
            assemble_empty_resolved(&typed)
                .expect("final replay")
                .debug_text(),
            resolved_debug
        );
    };
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
    let (_, _, symbols) = b5a_symbol_env(fixture.source, &fixture.module);
    let pristine_input = statement_input();
    assert_eq!(
        SourceStatementProducer::build(
            pristine_input.clone(),
            &symbols,
            &fixture.bindings,
            &fixture.primary,
            &fixture.atomic,
            &fixture.arena
        ),
        Ok(fixture.statement.clone())
    );
    let mut missing_contexts = pristine_input.clone();
    missing_contexts.contexts.clear();
    assert_eq!(
        SourceStatementProducer::build(
            missing_contexts,
            &symbols,
            &fixture.bindings,
            &fixture.primary,
            &fixture.atomic,
            &fixture.arena
        ),
        Err(SourceStatementError::InvalidAggregate)
    );
    replay();

    // Rebuild coherent dependencies so recovery faults reach the binding-profile owner.
    for mutation in 0..5 {
        let mut contexts = BindingContextTable::new();
        for (id, row) in fixture.bindings.contexts().iter() {
            let inserted = contexts.insert(BindingContextDraft {
                owner: row.owner.clone(),
                parent: row.parent,
                layer: row.layer,
                lexical_scope: row.lexical_scope.clone(),
                bindings: row.bindings.clone(),
                visible_bindings: row.visible_bindings.clone(),
                recovery: if id.index() == mutation {
                    BindingContextRecovery::Degraded
                } else {
                    row.recovery
                },
            });
            assert_eq!(inserted, id);
        }
        let mut bindings = BindingTable::new();
        for (id, row) in fixture.bindings.bindings().iter() {
            let inserted = bindings.insert(BindingDraft {
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
                recovery: if mutation == 4 {
                    BindingRecoveryState::Degraded
                } else {
                    row.recovery
                },
            });
            assert_eq!(inserted, id);
        }
        let mutated = BindingEnv::try_new(BindingEnvParts {
            source_id: fixture.source,
            module_id: fixture.module.clone(),
            contexts,
            bindings,
            diagnostics: fixture.bindings.diagnostics().clone(),
        })
        .expect("coherent recovery mutation");
        let mutated_before = mutated.clone();
        assert_eq!(
            SourceStatementProducer::build(
                pristine_input.clone(),
                &symbols,
                &mutated,
                &fixture.primary,
                &fixture.atomic,
                &fixture.arena
            ),
            Err(SourceStatementError::DependencyMismatch),
            "recovery {mutation}"
        );
        assert_eq!(mutated, mutated_before);
        assert_eq!(statement_input(), pristine_input);
        replay();
    }

    for row in 0..5 {
        for family in 0..4 {
            let mut statement = fixture.statement.clone();
            let expected = match family {
                0 => {
                    statement.statements.rows[row].source_ordinal = 5;
                    SourceStatementError::InvalidStatement {
                        statement: SourceStatementId::new(row),
                    }
                }
                1 => {
                    let context = &mut statement.contexts.rows[row];
                    context.binding_context =
                        BindingContextId::new((context.binding_context.index() + 1) % 4);
                    SourceStatementError::InvalidContext {
                        context: SourceStatementContextId::new(row),
                    }
                }
                2 => {
                    statement.input_facts.rows[row].uses.swap(0, 1);
                    SourceStatementError::InvalidInputFact {
                        fact: SourceStatementInputFactId::new(row),
                    }
                }
                3 => {
                    statement.candidate_facts.rows[row].ordinal = 1;
                    SourceStatementError::InvalidCandidateFact {
                        fact: SourceStatementCandidateFactId::new(row),
                    }
                }
                _ => unreachable!(),
            };
            assert_eq!(
                statement.validate_installation(
                    fixture.source,
                    &fixture.module,
                    &fixture.primary,
                    &fixture.atomic,
                    &fixture.arena
                ),
                Err(expected),
                "family {family} row {row}"
            );
            replay();
        }
    }
    let mut earlier_ordinal = fixture.statement.clone();
    earlier_ordinal.statements.rows[4].source_ordinal = 3;
    assert_eq!(
        earlier_ordinal.validate_installation(
            fixture.source,
            &fixture.module,
            &fixture.primary,
            &fixture.atomic,
            &fixture.arena
        ),
        Err(SourceStatementError::InvalidStatement {
            statement: SourceStatementId::new(4)
        })
    );
    replay();
    let wrong_typed_kind = mutate_arena(&fixture.arena, |id, row| {
        if id == TypedNodeId::new(82) {
            row.kind = "source.statement.citation".into();
        }
    });
    assert_eq!(
        fixture.statement.validate_installation(
            fixture.source,
            &fixture.module,
            &fixture.primary,
            &fixture.atomic,
            &wrong_typed_kind
        ),
        Err(SourceStatementError::DependencyMismatch)
    );
    replay();

    for mutation in 0..20 {
        let mut input = fixture.reference_input();
        let mut projection = fixture.projection.clone();
        let mut reference = fixture.reference.clone();
        let mut resolution = fixture.resolution.clone();
        match mutation {
            0 => input.labels[0].statement = SourceStatementId::new(0),
            1 => input.labels[0].context = SourceStatementContextId::new(0),
            2 => input.labels[0].candidate = SourceStatementCandidateFactId::new(0),
            3 => input.labels[0].origin_path = LabelOriginPath::new("wrong::proof::A"),
            4 => input.labels[0].proof_scope = LabelScopePath::new(vec![0, 1]),
            5 => input.labels[0].source_range.start -= 1,
            6 => input.labels[0].source_ordinal = 1,
            7 => input.labels[0].visible_after_ordinal = 2,
            8 => input.labels[0].spelling.push('B'),
            9 => input.labels[0].recovery = SourceStatementRecovery::Degraded,
            10 => input.citations[0].statement = SourceStatementId::new(3),
            11 => input.citations[0].context = SourceStatementContextId::new(3),
            12 => input.citations[0].proof_scope = LabelScopePath::new(vec![0]),
            13 => input.citations[0].source_range.start -= 1,
            14 => input.citations[0].ordinal = 1,
            15 => input.citations[0].recovery = SourceStatementRecovery::Degraded,
            16 => projection = projection.with_visibility(Visibility::Public),
            17 => projection = projection.with_export_status(ExportStatus::Exported),
            18 => reference = reference.with_expectation(LabelExpectation::Theorem),
            19 => {
                resolution = LabelResolver::new(&[]).resolve(
                    &fixture.module,
                    fixture.projection.namespace(),
                    std::slice::from_ref(&fixture.reference),
                )
            }
            _ => unreachable!(),
        }
        let expected = match mutation {
            0..=9 => SourceStatementReferenceError::InvalidLabel {
                label: SourceStatementLabelId::new(0),
            },
            10..=15 => SourceStatementReferenceError::InvalidCitation {
                citation: SourceStatementCitationId::new(0),
            },
            _ => SourceStatementReferenceError::DependencyMismatch,
        };
        assert_eq!(
            SourceStatementReferenceProducer::build(
                input,
                &fixture.statement,
                &fixture.resolver_ast,
                &projection,
                &reference,
                &resolution,
                &fixture.arena
            ),
            Err(expected),
            "reference {mutation}"
        );
        replay();
    }

    for mutation in 0..8 {
        let projection = &fixture.projection;
        let mut origin_path = projection.origin_path().clone();
        let mut namespace = projection.namespace().clone();
        let mut primary_spelling = projection.primary_spelling().to_owned();
        let mut declaration_range = projection.declaration_range();
        let mut origin = projection.origin().clone();
        let mut visible_after_ordinal = 1;
        let mut proof_scope = LabelScopePath::new(vec![0]);
        match mutation {
            0 => origin_path = LabelOriginPath::new("wrong::proof::A"),
            1 => namespace = NamespacePath::new("statement.wrong.projection"),
            2 => primary_spelling.push('B'),
            3 => declaration_range.start -= 1,
            4 => {
                origin = SemanticOrigin::new(
                    origin.source_id(),
                    origin.module_id().clone(),
                    SourceAnchor::Range(range(origin.source_id(), 94, 96)),
                    origin.structural_path().to_vec(),
                )
            }
            5 => origin = origin.recovered(),
            6 => visible_after_ordinal = 2,
            7 => proof_scope = LabelScopePath::new(vec![0, 1]),
            _ => unreachable!(),
        }
        let mutated = LabelProjection::proof_step(
            mizar_resolve::labels::LabelProjectionData {
                origin_path,
                module: projection.module().clone(),
                namespace,
                primary_spelling,
                kind: projection.kind(),
                declaration_range,
                origin,
                contribution: projection.contribution(),
            },
            visible_after_ordinal,
            proof_scope,
        );
        let expected = if matches!(mutation, 3 | 6 | 7) {
            SourceStatementReferenceError::InvalidLabel {
                label: SourceStatementLabelId::new(0),
            }
        } else {
            SourceStatementReferenceError::DependencyMismatch
        };
        assert_eq!(
            SourceStatementReferenceProducer::build(
                fixture.reference_input(),
                &fixture.statement,
                &fixture.resolver_ast,
                &mutated,
                &fixture.reference,
                &fixture.resolution,
                &fixture.arena
            ),
            Err(expected),
            "projection {mutation}"
        );
        replay();
    }

    for mutation in 0..10 {
        let reference = &fixture.reference;
        let mut site_range = reference.site().range();
        let mut site_spelling = reference.site().spelling().to_owned();
        let mut origin = reference.origin().clone();
        let mut ordinal = reference.ordinal();
        let mut scope = Some(LabelScopePath::new(vec![0, 1]));
        match mutation {
            0 => site_range.start -= 1,
            1 => site_spelling.push('B'),
            2 => {
                origin = SemanticOrigin::new(
                    origin.source_id(),
                    origin.module_id().clone(),
                    origin.anchor().clone(),
                    vec![999],
                )
            }
            3 => {
                origin = SemanticOrigin::new(
                    origin.source_id(),
                    origin.module_id().clone(),
                    SourceAnchor::Range(range(origin.source_id(), 169, 171)),
                    origin.structural_path().to_vec(),
                )
            }
            4 => origin = origin.recovered(),
            5 => ordinal = 3,
            6 => scope = Some(LabelScopePath::new(vec![0])),
            7 => scope = None,
            8 => {
                origin = SemanticOrigin::new(
                    source_id(252),
                    origin.module_id().clone(),
                    origin.anchor().clone(),
                    origin.structural_path().to_vec(),
                )
            }
            9 => {
                origin = SemanticOrigin::new(
                    origin.source_id(),
                    ModuleId::new(
                        fixture.module.package().clone(),
                        ModulePath::new("statement.b5a.reference-wrong"),
                    ),
                    origin.anchor().clone(),
                    origin.structural_path().to_vec(),
                )
            }
            _ => unreachable!(),
        }
        let mutated = LabelReferenceCandidate::unqualified_citation(
            mizar_resolve::resolved_ast::ReferenceSite::new(
                reference.site().node(),
                site_range,
                site_spelling,
            ),
            origin,
            ordinal,
            scope,
        );
        let expected = if matches!(mutation, 5 | 6) {
            SourceStatementReferenceError::InvalidCitation {
                citation: SourceStatementCitationId::new(0),
            }
        } else {
            SourceStatementReferenceError::DependencyMismatch
        };
        assert_eq!(
            SourceStatementReferenceProducer::build(
                fixture.reference_input(),
                &fixture.statement,
                &fixture.resolver_ast,
                &fixture.projection,
                &mutated,
                &fixture.resolution,
                &fixture.arena
            ),
            Err(expected),
            "candidate {mutation}"
        );
        replay();
    }

    for mutation in 0..6 {
        let mut builder = ResolvedArenaBuilder::new();
        let mut ids = Vec::new();
        for (id, node) in fixture.resolver_ast.nodes().iter() {
            let mut kind = node.kind().clone();
            let mut origin = node.origin().clone();
            let mut recovery = node.recovery();
            let mut resolution = node.resolution();
            let mut reference_key = node.reference_key();
            if id.index() == 82 {
                match mutation {
                    0 => {
                        kind = fixture
                            .resolver_ast
                            .nodes()
                            .node(ids[81])
                            .expect("preceding actual resolver node")
                            .kind()
                            .clone()
                    }
                    1 => {
                        origin = SemanticOrigin::new(
                            origin.source_id(),
                            origin.module_id().clone(),
                            origin.anchor().clone(),
                            vec![999],
                        )
                    }
                    2 => {
                        origin = SemanticOrigin::new(
                            origin.source_id(),
                            origin.module_id().clone(),
                            SourceAnchor::Range(range(origin.source_id(), 169, 171)),
                            origin.structural_path().to_vec(),
                        )
                    }
                    3 => {
                        origin = origin.recovered();
                        recovery = RecoveryState::Recovered;
                    }
                    4 => resolution = NodeResolutionState::Deferred,
                    5 => {
                        resolution = NodeResolutionState::NotApplicable;
                        reference_key = None;
                    }
                    _ => unreachable!(),
                }
            }
            let mut rebuilt = ResolvedNode::new(kind, node.children().to_vec(), origin)
                .with_recovery(recovery)
                .with_resolution(resolution);
            if let Some(key) = reference_key {
                rebuilt = rebuilt.with_reference_key(key);
            }
            let inserted = builder
                .push(rebuilt)
                .expect("coherent resolver node mutation");
            assert_eq!(inserted.index(), id.index());
            ids.push(inserted);
        }
        let mutated = ResolvedAst::try_new(
            fixture.source,
            fixture.module.clone(),
            builder
                .finish(ids[fixture.resolver_ast.nodes().root().index()])
                .expect("coherent resolver arena mutation"),
            fixture.resolver_ast.name_refs().clone(),
            fixture.resolver_ast.label_refs().clone(),
            fixture.resolver_ast.imports().clone(),
        )
        .expect("coherent resolver AST mutation");
        assert_eq!(
            SourceStatementReferenceProducer::build(
                fixture.reference_input(),
                &fixture.statement,
                &mutated,
                &fixture.projection,
                &fixture.reference,
                &fixture.resolution,
                &fixture.arena
            ),
            Err(SourceStatementReferenceError::DependencyMismatch),
            "resolver AST {mutation}"
        );
        replay();
    }
}

#[test]
fn task_258b5a_b1_same_scope_cross_profile_installation_is_atomic() {
    let b1 = B1Fixture::new(252);
    let b5a = B5AFixture::new(252);
    let b1_references = b1
        .references(b1.reference_input())
        .expect("Task258B1 references");
    let b5a_references = b5a
        .references(b5a.reference_input())
        .expect("Task258B5A references");
    let b1_base = b1.empty_typed();
    let b5a_base = b5a.empty_typed();
    let b5a_pristine = b5a_base
        .clone()
        .with_source_statement_references(b5a.statement.clone(), b5a_references.clone())
        .expect("pristine B5A pair");
    let b5a_final = assemble_empty_resolved(&b5a_pristine).expect("pristine B5A final");
    let b4a = Task258B4AFixture::new(50_006);
    let foreign_statement = b4a.statement();
    let run = || {
        b5a_base
            .clone()
            .with_source_statement(foreign_statement.clone())
    };
    assert_eq!(run(), Err(TypedAstError::InvalidSourceStatement));
    assert_eq!(run(), run(), "B4A statement on B5A lower replay");
    assert_eq!(foreign_statement, b4a.statement());
    assert_eq!(
        b5a_base
            .clone()
            .with_source_statement_references(b5a.statement.clone(), b1_references.clone(),),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        b1_base
            .clone()
            .with_source_statement_references(b1.statement.clone(), b5a_references.clone(),),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(b1_base, b1.empty_typed());
    assert_eq!(b5a_base, b5a.empty_typed());
    assert!(
        b1.empty_typed()
            .with_source_statement_references(b1.statement.clone(), b1_references)
            .is_ok()
    );
    let replay = b5a
        .empty_typed()
        .with_source_statement_references(b5a.statement.clone(), b5a_references)
        .expect("pristine B5A pair replay");
    assert_eq!(replay, b5a_pristine);
    assert_eq!(assemble_empty_resolved(&replay), Ok(b5a_final));
}

#[test]
fn task_258b5a_final_clone_revalidates_descendant_visibility_and_empty_semantics() {
    let fixture = B5AFixture::new(253);
    let references = fixture
        .references(fixture.reference_input())
        .expect("Task258B5A references");
    let typed = fixture
        .empty_typed()
        .with_source_statement_references(fixture.statement.clone(), references.clone())
        .expect("Task258B5A typed AST");
    let resolved = assemble_empty_resolved(&typed).expect("Task258B5A final assembly");
    assert_eq!(resolved.source_statement(), Some(&fixture.statement));
    assert_eq!(resolved.source_statement_references(), Some(&references));
    assert!(resolved.statement_semantics().is_empty());
    assert!(resolved.checked_formulas().is_empty());
    assert!(resolved.checked_proofs().is_empty());
    assert!(resolved.checked_proof_nodes().is_empty());
    assert!(resolved.checked_terminal_goals().is_empty());
    assert_eq!(resolved.clone(), resolved);
    let debug = resolved.debug_text();
    assert!(
        debug
            .find("source-statement-debug-v1\n")
            .expect("statement debug")
            < debug
                .find("source-statement-reference-debug-v1\n")
                .expect("reference debug")
    );

    let mut wrong_descendant = references.clone();
    wrong_descendant.citations.rows[0].proof_scope = LabelScopePath::new(vec![0, 0]);
    let mut corrupted = typed.clone();
    corrupted.inject_source_statement_bundle_for_test(fixture.statement.clone(), wrong_descendant);
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
    assert_eq!(
        assemble_empty_resolved(&typed)
            .expect("Task258B5A replay")
            .debug_text(),
        debug
    );
}

#[test]
fn task_258b4a_composite_root_profile_and_debug_are_exact() {
    let fixture = Task258B4AFixture::new(50_000);
    let statement = fixture.statement();
    assert!(statement.is_task_258b4a_profile());
    assert_eq!(
        statement.composite_formula_fingerprint(),
        Some(fixture.composite.debug_text().as_str())
    );
    assert_eq!(
        statement.formula_composition_fingerprint(),
        Some(fixture.composition.debug_text().as_str())
    );
    assert_eq!(statement.owners().len(), 1);
    assert_eq!(statement.statements().len(), 1);
    assert_eq!(statement.contexts().len(), 1);
    assert!(statement.input_facts().is_empty());
    assert_eq!(statement.candidate_facts().len(), 1);
    let owner = statement
        .owners()
        .get(SourceTheoremOwnerId::new(0))
        .expect("owner");
    assert_eq!(owner.symbol(), &fixture.symbol);
    assert_eq!(owner.contribution(), fixture.contribution);
    assert_eq!(owner.contribution().index(), 0);
    assert_eq!(owner.site(), &node(22));
    assert_eq!(owner.source_range(), range(fixture.source, 0, 78));
    assert_eq!(owner.spelling(), "FormulaQuantifierBoundUsePayloadBoundary");
    assert_eq!(owner.role(), SourceTheoremRole::Theorem);
    assert_eq!(owner.status(), SourceTheoremStatus::Unmodified);
    assert_eq!(owner.recovery(), SourceStatementRecovery::Normal);
    assert_eq!(statement.checked_owner().symbol(), &fixture.symbol);
    assert_eq!(
        statement.checked_owner().source_range(),
        range(fixture.source, 0, 78)
    );
    assert_eq!(statement.checked_owner().origin().structural_path(), [2, 0]);
    assert!(statement.checked_owner().origin().import_edge().is_none());
    assert!(!statement.checked_owner().origin().is_recovered());
    assert_eq!(statement.checked_owner().visibility(), Visibility::Public);
    assert_eq!(
        statement.checked_owner().export_status(),
        ExportStatus::Exported
    );
    let statement_row = statement
        .statements()
        .get(SourceStatementId::new(0))
        .expect("statement");
    assert_eq!(statement_row.owner(), SourceTheoremOwnerId::new(0));
    assert_eq!(statement_row.context(), SourceStatementContextId::new(0));
    assert_eq!(
        statement_row.formula(),
        SourceStatementFormulaTarget::Composite(SourceCompositeFormulaId::new(0))
    );
    assert_eq!(statement_row.site(), &node(22));
    assert_eq!(statement_row.source_range(), range(fixture.source, 0, 78));
    assert_eq!(statement_row.source_ordinal(), 0);
    assert_eq!(
        statement_row.spelling(),
        "theorem FormulaQuantifierBoundUsePayloadBoundary : for x being set holds x = x ;"
    );
    assert_eq!(
        statement_row.kind(),
        SourceStatementKind::TheoremProposition
    );
    assert_eq!(statement_row.recovery(), SourceStatementRecovery::Normal);
    let context = statement
        .contexts()
        .get(SourceStatementContextId::new(0))
        .expect("context");
    assert_eq!(context.statement(), SourceStatementId::new(0));
    assert_eq!(context.binding_context(), BindingContextId::new(0));
    assert_eq!(context.source_range(), range(fixture.source, 0, 78));
    assert!(context.visible_bindings().is_empty());
    let candidate = statement
        .candidate_facts()
        .get(SourceStatementCandidateFactId::new(0))
        .expect("candidate");
    assert_eq!(candidate.statement(), SourceStatementId::new(0));
    assert_eq!(candidate.context(), SourceStatementContextId::new(0));
    assert_eq!(candidate.ordinal(), 0);
    assert_eq!(
        candidate.kind(),
        SourceStatementCandidateFactKind::UnverifiedProposition
    );
    assert_eq!(
        candidate.formula(),
        SourceStatementFormulaTarget::Composite(SourceCompositeFormulaId::new(0))
    );
    let debug = statement.debug_text();
    let atomic = debug
        .find("atomic-formula-fingerprint:")
        .expect("atomic fingerprint");
    let composite = debug
        .find("composite-formula-fingerprint:")
        .expect("composite fingerprint");
    let composition = debug
        .find("formula-composition-fingerprint:")
        .expect("composition fingerprint");
    let owner = debug.find("owner#0 ").expect("owner");
    assert!(atomic < composite && composite < composition && composition < owner);
    assert!(debug.contains("formula=composite:0"));
}

#[test]
fn task_258b4a_composite_root_corruption_matrix_fails_atomically() {
    let fixture = Task258B4AFixture::new(50_001);
    let other = Task258B4AFixture::new(50_101);
    let foreign = Fixture::new(50_201);
    let baseline = fixture.statement();
    let pristine_fixture = fixture.clone();
    let pristine_typed = fixture.typed();
    let pristine_final = assemble_empty_resolved(&pristine_typed).expect("pristine final");
    let assert_pristine = || {
        assert_eq!(fixture, pristine_fixture);
        assert_eq!(fixture.statement(), baseline);
        let replay = fixture.typed();
        assert_eq!(replay, pristine_typed);
        assert_eq!(assemble_empty_resolved(&replay), Ok(pristine_final.clone()));
    };
    for mutation in 0..38 {
        let mut input = fixture.input();
        let expected = match mutation {
            0 => {
                input.source_id = other.source;
                SourceStatementError::DependencyMismatch
            }
            1 => {
                input.module_id = ModuleId::new(
                    PackageId::new("pkg"),
                    ModulePath::new("statement.task258b4a.other"),
                );
                SourceStatementError::DependencyMismatch
            }
            2 => {
                input.owners.clear();
                SourceStatementError::InvalidAggregate
            }
            3 => {
                input.owners.push(input.owners[0].clone());
                SourceStatementError::InvalidAggregate
            }
            4 => {
                input.owners[0].symbol = foreign.symbol.clone();
                SourceStatementError::InvalidOwner {
                    owner: SourceTheoremOwnerId::new(0),
                }
            }
            5 => {
                input.owners[0].site = node(21);
                SourceStatementError::InvalidOwner {
                    owner: SourceTheoremOwnerId::new(0),
                }
            }
            6 => {
                input.owners[0].source_range.end -= 1;
                SourceStatementError::InvalidOwner {
                    owner: SourceTheoremOwnerId::new(0),
                }
            }
            7 => {
                input.owners[0].spelling.push('!');
                SourceStatementError::InvalidOwner {
                    owner: SourceTheoremOwnerId::new(0),
                }
            }
            8 => {
                input.owners[0].recovery = SourceStatementRecovery::Degraded;
                SourceStatementError::InvalidOwner {
                    owner: SourceTheoremOwnerId::new(0),
                }
            }
            9 => {
                input.statements.clear();
                SourceStatementError::InvalidAggregate
            }
            10 => {
                input.statements.push(input.statements[0].clone());
                SourceStatementError::InvalidAggregate
            }
            11 => {
                input.statements[0].owner = SourceTheoremOwnerId::new(1);
                SourceStatementError::InvalidStatement {
                    statement: SourceStatementId::new(0),
                }
            }
            12 => {
                input.statements[0].context = SourceStatementContextId::new(1);
                SourceStatementError::InvalidStatement {
                    statement: SourceStatementId::new(0),
                }
            }
            13 => {
                input.statements[0].formula =
                    SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(0));
                SourceStatementError::InvalidStatement {
                    statement: SourceStatementId::new(0),
                }
            }
            14 => {
                input.statements[0].site = node(21);
                SourceStatementError::InvalidStatement {
                    statement: SourceStatementId::new(0),
                }
            }
            15 => {
                input.statements[0].source_range.end -= 1;
                SourceStatementError::InvalidStatement {
                    statement: SourceStatementId::new(0),
                }
            }
            16 => {
                input.statements[0].source_ordinal = 1;
                SourceStatementError::InvalidStatement {
                    statement: SourceStatementId::new(0),
                }
            }
            17 => {
                input.statements[0].spelling.push('!');
                SourceStatementError::InvalidStatement {
                    statement: SourceStatementId::new(0),
                }
            }
            18 => {
                input.statements[0].kind = SourceStatementKind::Conclusion;
                SourceStatementError::InvalidStatement {
                    statement: SourceStatementId::new(0),
                }
            }
            19 => {
                input.statements[0].recovery = SourceStatementRecovery::Degraded;
                SourceStatementError::InvalidStatement {
                    statement: SourceStatementId::new(0),
                }
            }
            20 => {
                input.contexts.clear();
                SourceStatementError::InvalidAggregate
            }
            21 => {
                input.contexts.push(input.contexts[0].clone());
                SourceStatementError::InvalidAggregate
            }
            22 => {
                input.contexts[0].statement = SourceStatementId::new(1);
                SourceStatementError::InvalidContext {
                    context: SourceStatementContextId::new(0),
                }
            }
            23 => {
                input.contexts[0].binding_context = BindingContextId::new(1);
                SourceStatementError::InvalidContext {
                    context: SourceStatementContextId::new(0),
                }
            }
            24 => {
                input.contexts[0].source_range.end -= 1;
                SourceStatementError::InvalidContext {
                    context: SourceStatementContextId::new(0),
                }
            }
            25 => {
                input.contexts[0].visible_bindings.push(BindingId::new(0));
                SourceStatementError::InvalidContext {
                    context: SourceStatementContextId::new(0),
                }
            }
            26 => {
                input.input_facts.push(SourceStatementInputFactInput {
                    statement: SourceStatementId::new(0),
                    context: SourceStatementContextId::new(0),
                    ordinal: 0,
                    kind: SourceStatementInputFactKind::ReservedTypeGuard,
                    binding: BindingId::new(0),
                    uses: vec![
                        SourcePrimaryTermReferenceId::new(0),
                        SourcePrimaryTermReferenceId::new(1),
                    ],
                });
                SourceStatementError::InvalidAggregate
            }
            27 => {
                input.candidate_facts.clear();
                SourceStatementError::InvalidAggregate
            }
            28 => {
                input.candidate_facts.push(input.candidate_facts[0].clone());
                SourceStatementError::InvalidAggregate
            }
            29 => {
                input.candidate_facts[0].statement = SourceStatementId::new(1);
                SourceStatementError::InvalidCandidateFact {
                    fact: SourceStatementCandidateFactId::new(0),
                }
            }
            30 => {
                input.candidate_facts[0].context = SourceStatementContextId::new(1);
                SourceStatementError::InvalidCandidateFact {
                    fact: SourceStatementCandidateFactId::new(0),
                }
            }
            31 => {
                input.candidate_facts[0].ordinal = 1;
                SourceStatementError::InvalidCandidateFact {
                    fact: SourceStatementCandidateFactId::new(0),
                }
            }
            32 => {
                input.candidate_facts[0].formula =
                    SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(0));
                SourceStatementError::InvalidCandidateFact {
                    fact: SourceStatementCandidateFactId::new(0),
                }
            }
            33 => {
                input.owners[0].source_range.start += 1;
                input.statements[0].source_range.start += 1;
                input.contexts[0].source_range.start += 1;
                SourceStatementError::InvalidOwner {
                    owner: SourceTheoremOwnerId::new(0),
                }
            }
            34 => {
                input.statements[0].formula =
                    SourceStatementFormulaTarget::Composite(SourceCompositeFormulaId::new(1));
                input.candidate_facts[0].formula =
                    SourceStatementFormulaTarget::Composite(SourceCompositeFormulaId::new(1));
                SourceStatementError::InvalidStatement {
                    statement: SourceStatementId::new(0),
                }
            }
            35 => {
                input.statements[0].context = SourceStatementContextId::new(1);
                input.contexts[0].statement = SourceStatementId::new(1);
                input.candidate_facts[0].context = SourceStatementContextId::new(1);
                SourceStatementError::InvalidStatement {
                    statement: SourceStatementId::new(0),
                }
            }
            36 => {
                input.owners.clear();
                input.candidate_facts.clear();
                SourceStatementError::InvalidAggregate
            }
            37 => {
                input.statements.clear();
                input.contexts.clear();
                SourceStatementError::InvalidAggregate
            }
            _ => unreachable!(),
        };
        assert_ne!(
            input,
            fixture.input(),
            "input mutation {mutation} actual delta"
        );
        let input_before = input.clone();
        let run = || fixture.build_statement(input.clone());
        assert_eq!(run(), Err(expected.clone()), "input mutation {mutation}");
        assert_eq!(run(), run(), "input mutation {mutation} replay");
        assert_eq!(input, input_before, "input mutation {mutation} purity");
        assert_pristine();
    }

    for mutation in [
        "Imported",
        "Missing",
        "Duplicate",
        "WrongPath",
        "WrongKind",
        "Private",
        "Recovered",
    ] {
        let label = fixture
            .symbols
            .labels()
            .iter()
            .next()
            .expect("B4A theorem label");
        let origin = match mutation {
            "Imported" => {
                let mut nodes = ResolvedArenaBuilder::new();
                let owner = nodes
                    .push(ResolvedNode::new(
                        syntax::SurfaceNodeKind::ImportAliasDecl,
                        Vec::new(),
                        label.origin().clone(),
                    ))
                    .expect("imported-label owner");
                let mut imports = ResolvedImports::new();
                let import = imports.push_import(ResolvedImport::new(
                    owner,
                    range(fixture.source, 0, 1),
                    "import statement.fixture;",
                    None,
                    ImportResolution::Resolved(fixture.module.clone()),
                    label.origin().clone(),
                ));
                label.origin().clone().with_import_edge(import)
            }
            "Recovered" => label.origin().clone().recovered(),
            _ => label.origin().clone(),
        };
        let mut labels = LabelIndex::new();
        if mutation != "Missing" {
            labels.insert(
                LabelEntry::new(
                    if mutation == "WrongPath" {
                        LabelOriginPath::new("task258b2::wrong::theorem::origin")
                    } else {
                        label.origin_path().clone()
                    },
                    if mutation == "WrongKind" {
                        LabelKind::ProofStep
                    } else {
                        label.kind()
                    },
                    label.namespace().clone(),
                    label.primary_spelling(),
                    origin,
                    label.contribution(),
                )
                .with_visibility(if mutation == "Private" {
                    Visibility::Private
                } else {
                    label.visibility()
                })
                .with_export_status(label.export_status()),
            );
        }
        if mutation == "Duplicate" {
            labels.insert(
                LabelEntry::new(
                    LabelOriginPath::new("task258b2::duplicate::theorem::origin"),
                    label.kind(),
                    label.namespace().clone(),
                    label.primary_spelling(),
                    label.origin().clone(),
                    label.contribution(),
                )
                .with_visibility(label.visibility())
                .with_export_status(label.export_status()),
            );
        }
        let mut corrupted = fixture.clone();
        corrupted.symbols = SymbolEnv::new(
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
                contributions: fixture.symbols.contributions().clone(),
                module_summaries: fixture.symbols.module_summaries().clone(),
            },
        );
        assert_ne!(
            corrupted.symbols, fixture.symbols,
            "resolver {mutation} actual delta"
        );
        assert_eq!(corrupted.input(), fixture.input());
        let before = corrupted.clone();
        let run = || corrupted.build_statement(corrupted.input());
        assert_eq!(
            run(),
            Err(SourceStatementError::InvalidOwner {
                owner: SourceTheoremOwnerId::new(0)
            }),
            "resolver {mutation}"
        );
        assert_eq!(run(), run(), "resolver {mutation} replay");
        assert_eq!(corrupted, before, "resolver {mutation} purity");
        assert_pristine();
    }

    for mutation in 0..8 {
        let mut corrupted = baseline.clone();
        match mutation {
            0 => corrupted.composite_formula_fingerprint = None,
            1 => corrupted.formula_composition_fingerprint = None,
            2 => corrupted.composite_formula_fingerprint = Some("stale".to_owned()),
            3 => corrupted.formula_composition_fingerprint = Some("stale".to_owned()),
            4 => {
                corrupted.composite_formula_fingerprint = Some(fixture.composition.debug_text());
                corrupted.formula_composition_fingerprint = Some(fixture.composite.debug_text());
            }
            5 => {
                corrupted.composite_formula_fingerprint = Some(fixture.composition.debug_text());
                corrupted.formula_composition_fingerprint = Some(fixture.composition.debug_text());
            }
            6 => {
                corrupted.binding_fingerprint.push_str("stale");
                corrupted.composite_formula_fingerprint = None;
            }
            7 => {
                corrupted.primary_term_fingerprint.push_str("stale");
                corrupted.atomic_formula_fingerprint.push_str("stale");
                corrupted.formula_composition_fingerprint = None;
            }
            _ => unreachable!(),
        }
        assert_ne!(
            corrupted, baseline,
            "fingerprint mutation {mutation} actual delta"
        );
        let corrupted_before = corrupted.clone();
        let run = || {
            corrupted.validate_installation_with_formula_composition(
                fixture.source,
                &fixture.module,
                &fixture.primary,
                &fixture.atomic,
                &fixture.composite,
                &fixture.composition,
                &fixture.arena,
            )
        };
        assert_eq!(
            run(),
            Err(SourceStatementError::DependencyMismatch),
            "fingerprint mutation {mutation}"
        );
        assert_eq!(run(), run(), "mutation {mutation} replay");
        assert_eq!(
            corrupted, corrupted_before,
            "fingerprint mutation {mutation} purity"
        );
        assert_pristine();
    }

    // Only nodes owned by the lower tuple or theorem are normative here.
    for node_index in [4, 12, 13, 14, 15, 17, 19, 20, 22] {
        for mutate_kind in [false, true] {
            let mut nodes = fixture
                .arena
                .iter()
                .map(|(_, node)| node.clone())
                .collect::<Vec<_>>();
            if mutate_kind {
                nodes[node_index].kind = "source.statement.corrupted".into();
            } else {
                nodes[node_index].anchor = SourceAnchor::Range(range(fixture.source, 0, 0));
            }
            let arena = TypedArena::try_new(None, nodes).expect("owned-node mutation");
            assert_ne!(
                arena, fixture.arena,
                "node {node_index} kind={mutate_kind} actual delta"
            );
            let arena_before = arena.clone();
            let expected = if node_index == 22 && !mutate_kind {
                SourceStatementError::InvalidOwner {
                    owner: SourceTheoremOwnerId::new(0),
                }
            } else {
                SourceStatementError::DependencyMismatch
            };
            let produce = || {
                SourceStatementProducer::build_with_formula_composition(
                    fixture.input(),
                    &fixture.symbols,
                    &fixture.bindings,
                    &fixture.primary,
                    &fixture.atomic,
                    &fixture.composite,
                    &fixture.composition,
                    &arena,
                )
            };
            let install = || {
                baseline.validate_installation_with_formula_composition(
                    fixture.source,
                    &fixture.module,
                    &fixture.primary,
                    &fixture.atomic,
                    &fixture.composite,
                    &fixture.composition,
                    &arena,
                )
            };
            assert_eq!(
                produce(),
                Err(expected.clone()),
                "node {node_index} kind={mutate_kind} producer"
            );
            assert_eq!(
                produce(),
                produce(),
                "node {node_index} kind={mutate_kind} producer replay"
            );
            assert_eq!(
                install(),
                Err(expected),
                "node {node_index} kind={mutate_kind} installer"
            );
            assert_eq!(
                install(),
                install(),
                "node {node_index} kind={mutate_kind} installer replay"
            );
            assert_eq!(
                arena, arena_before,
                "node {node_index} kind={mutate_kind} purity"
            );
            assert_pristine();
        }
    }

    let rooted_arena = TypedArena::try_new(
        Some(TypedNodeId::new(25)),
        fixture.arena.iter().map(|(_, node)| node.clone()).collect(),
    )
    .expect("coherent rooted lower arena");
    fixture
        .primary
        .validate_installation(fixture.source, &fixture.module, &rooted_arena)
        .expect("rooted Task252 remains valid");
    fixture
        .atomic
        .validate_installation(
            fixture.source,
            &fixture.module,
            &fixture.primary,
            None,
            None,
            None,
            &rooted_arena,
        )
        .expect("rooted Task256 remains valid");
    fixture
        .composite
        .validate_installation(fixture.source, &fixture.module, &rooted_arena)
        .expect("rooted Task257 remains valid");
    fixture
        .composition
        .validate_installation(
            fixture.source,
            &fixture.module,
            &fixture.primary,
            &fixture.atomic,
            &fixture.composite,
            &rooted_arena,
        )
        .expect("rooted Task257B1 remains valid");
    let rooted_before = rooted_arena.clone();
    let rooted_run = || {
        SourceStatementProducer::build_with_formula_composition(
            fixture.input(),
            &fixture.symbols,
            &fixture.bindings,
            &fixture.primary,
            &fixture.atomic,
            &fixture.composite,
            &fixture.composition,
            &rooted_arena,
        )
    };
    assert_eq!(rooted_run(), Err(SourceStatementError::DependencyMismatch));
    assert_eq!(rooted_run(), rooted_run(), "rooted lower replay");
    let rooted_install = || {
        baseline.validate_installation_with_formula_composition(
            fixture.source,
            &fixture.module,
            &fixture.primary,
            &fixture.atomic,
            &fixture.composite,
            &fixture.composition,
            &rooted_arena,
        )
    };
    assert_ne!(rooted_arena, fixture.arena);
    assert_eq!(
        rooted_install(),
        Err(SourceStatementError::DependencyMismatch)
    );
    assert_eq!(
        rooted_install(),
        rooted_install(),
        "rooted retained installer replay"
    );
    assert_eq!(rooted_arena, rooted_before);
    assert_pristine();

    let (relocated_arena, relocated_primary, relocated_atomic, relocated_composition) =
        fixture.coherent_relocated_primary_lower();
    assert_ne!(
        (
            &relocated_arena,
            &relocated_primary,
            &relocated_atomic,
            &relocated_composition
        ),
        (
            &fixture.arena,
            &fixture.primary,
            &fixture.atomic,
            &fixture.composition
        ),
        "relocated lower actual delta"
    );
    let relocated_before = (
        relocated_arena.clone(),
        relocated_primary.clone(),
        relocated_atomic.clone(),
        relocated_composition.clone(),
    );
    let relocated_run = || {
        SourceStatementProducer::build_with_formula_composition(
            fixture.input(),
            &fixture.symbols,
            &fixture.bindings,
            &relocated_primary,
            &relocated_atomic,
            &fixture.composite,
            &relocated_composition,
            &relocated_arena,
        )
    };
    assert_eq!(
        relocated_run(),
        Err(SourceStatementError::DependencyMismatch)
    );
    assert_eq!(relocated_run(), relocated_run(), "relocated lower replay");
    let relocated_install = || {
        baseline.validate_installation_with_formula_composition(
            fixture.source,
            &fixture.module,
            &relocated_primary,
            &relocated_atomic,
            &fixture.composite,
            &relocated_composition,
            &relocated_arena,
        )
    };
    assert_eq!(
        relocated_install(),
        Err(SourceStatementError::DependencyMismatch)
    );
    assert_eq!(
        relocated_install(),
        relocated_install(),
        "relocated retained installer replay"
    );
    assert_eq!(
        (
            relocated_arena,
            relocated_primary,
            relocated_atomic,
            relocated_composition
        ),
        relocated_before
    );
    assert_pristine();
}

#[test]
fn task_258b4a_cross_family_installation_and_route_isolation_are_atomic() {
    let fixture = Task258B4AFixture::new(50_002);
    let base = fixture.typed_base();
    let paired = base
        .clone()
        .with_source_formula_composition_statement(
            fixture.composite.clone(),
            fixture.composition.clone(),
            fixture.statement(),
        )
        .expect("paired install");
    assert!(paired.source_statement().is_some());
    assert!(paired.source_composite_formula().is_some());
    assert!(paired.source_formula_composition().is_some());
    assert_eq!(
        paired.clone().with_source_formula_composition(
            fixture.composite.clone(),
            fixture.composition.clone(),
        ),
        Err(TypedAstError::InvalidSourceFormulaComposition)
    );
    assert_eq!(
        base.clone()
            .with_source_formula_composition(
                fixture.composite.clone(),
                fixture.composition.clone(),
            )
            .expect("lower-only install")
            .with_source_statement(fixture.statement()),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        base.clone().with_source_statement(fixture.statement()),
        Err(TypedAstError::InvalidSourceStatement)
    );
    let atomic = Fixture::new(50_102);
    let atomic_statement = atomic
        .build(atomic.input())
        .expect("atomic Task258 statement");
    let atomic_debug = atomic_statement.debug_text();
    assert_eq!(atomic_statement.composite_formula_fingerprint(), None);
    assert_eq!(atomic_statement.formula_composition_fingerprint(), None);
    assert!(!atomic_debug.contains("composite-formula-fingerprint:"));
    assert!(!atomic_debug.contains("formula-composition-fingerprint:"));
    let atomic_typed = TypedAst::try_new(TypedAstParts {
        source_id: atomic.source,
        module_id: atomic.module.clone(),
        resolved_root: None,
        source_context: None,
        source_type: None,
        source_attribute: None,
        nodes: atomic.arena.clone(),
        contexts: LocalTypeContextTable::new(),
        types: TypeTable::new(),
        facts: TypeFactTable::new(),
        coercions: CoercionTable::new(),
        initial_obligations: InitialObligationTable::new(),
        diagnostics: TypeDiagnosticTable::new(),
    })
    .expect("atomic Task258 typed base")
    .with_source_term(atomic.primary.clone())
    .expect("atomic Task252")
    .with_source_atomic_formula(atomic.atomic.clone())
    .expect("atomic Task256")
    .with_source_statement(atomic_statement.clone())
    .expect("atomic Task258 family");
    let atomic_before = atomic_typed.clone();
    assert_eq!(
        atomic_typed
            .clone()
            .with_source_formula_composition_statement(
                fixture.composite.clone(),
                fixture.composition.clone(),
                fixture.statement(),
            ),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(atomic_typed, atomic_before);
    let paired_before = paired.clone();
    assert_eq!(
        paired.clone().with_source_statement(atomic_statement),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(paired, paired_before);
    let replay = fixture.typed();
    assert_eq!(paired, replay);
    assert_eq!(base, fixture.typed_base());
    let paired_final = assemble_empty_resolved(&paired).expect("paired final");
    assert_eq!(assemble_empty_resolved(&replay), Ok(paired_final));
    assert_eq!(
        atomic
            .build(atomic.input())
            .expect("atomic clean replay")
            .debug_text(),
        atomic_debug
    );
}

#[test]
fn task_258b4a_final_clone_revalidates_composite_root_and_empty_semantics() {
    let fixture = Task258B4AFixture::new(50_003);
    let typed = fixture.typed();
    let resolved = assemble_empty_resolved(&typed).expect("Task258B4A final");
    assert_eq!(typed.source_statement(), resolved.source_statement());
    assert_eq!(
        typed.source_composite_formula(),
        resolved.source_composite_formula()
    );
    assert_eq!(
        typed.source_formula_composition(),
        resolved.source_formula_composition()
    );
    assert_eq!(resolved.clone(), resolved);
    assert_eq!(resolved.clone().debug_text(), resolved.debug_text());
    assert!(resolved.expr_metadata().is_empty());
    assert!(resolved.checked_formulas().is_empty());
    assert!(resolved.statement_semantics().is_empty());
    assert!(resolved.checked_proofs().is_empty());
    assert!(resolved.checked_proof_nodes().is_empty());
    assert!(resolved.checked_terminal_goals().is_empty());

    let other = Task258B4AFixture::new(50_103);
    for mutation in 0..19 {
        let mut stale_statement = fixture.statement();
        match mutation {
            0 => stale_statement.composite_formula_fingerprint = None,
            1 => stale_statement.formula_composition_fingerprint = None,
            2 => stale_statement.composite_formula_fingerprint = Some("stale".to_owned()),
            3 => stale_statement.formula_composition_fingerprint = Some("stale".to_owned()),
            4 => stale_statement.checked_owner = other.statement().checked_owner,
            5 => {
                stale_statement.binding_env = other.bindings.clone();
                stale_statement.binding_fingerprint = other.bindings.debug_text();
            }
            6 => stale_statement.owners.rows.clear(),
            7 => stale_statement.statements.rows.clear(),
            8 => stale_statement.contexts.rows.clear(),
            9 => stale_statement.candidate_facts.rows.clear(),
            10 => stale_statement.owners.rows[0].source_range.end -= 1,
            11 => {
                stale_statement.statements.rows[0].formula =
                    SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(0))
            }
            12 => stale_statement.contexts.rows[0]
                .visible_bindings
                .push(BindingId::new(0)),
            13 => stale_statement.candidate_facts.rows[0].ordinal = 1,
            14 => stale_statement.source_id = other.source,
            15 => {
                stale_statement.module_id = ModuleId::new(
                    PackageId::new("pkg"),
                    ModulePath::new("statement.task258b4a.other"),
                )
            }
            16 => stale_statement
                .input_facts
                .rows
                .push(SourceStatementInputFact {
                    statement: SourceStatementId::new(0),
                    context: SourceStatementContextId::new(0),
                    ordinal: 0,
                    kind: SourceStatementInputFactKind::ReservedTypeGuard,
                    binding: BindingId::new(0),
                    uses: vec![
                        SourcePrimaryTermReferenceId::new(0),
                        SourcePrimaryTermReferenceId::new(1),
                    ],
                }),
            17 => {
                let row = stale_statement.candidate_facts.rows[0].clone();
                stale_statement.candidate_facts.rows.push(row);
            }
            18 => {
                stale_statement.composite_formula_fingerprint =
                    Some(fixture.composition.debug_text());
                stale_statement.formula_composition_fingerprint =
                    Some(fixture.composite.debug_text());
            }
            _ => unreachable!(),
        }
        assert_ne!(
            stale_statement,
            fixture.statement(),
            "final mutation {mutation} actual delta"
        );
        let mut stale_typed = typed.clone();
        stale_typed.inject_source_statement_for_test(stale_statement);
        assert_ne!(stale_typed, typed);
        let stale_before = stale_typed.clone();
        let run = || assemble_empty_resolved(&stale_typed);
        assert_eq!(
            run(),
            Err(ResolvedTypedAstError::InvalidSourceStatement),
            "final statement mutation {mutation}"
        );
        assert_eq!(run(), run(), "final statement mutation {mutation} replay");
        assert_eq!(stale_typed, stale_before);
        assert_eq!(fixture.typed(), typed);
        assert_eq!(assemble_empty_resolved(&typed), Ok(resolved.clone()));
    }
    for partial in 0..3 {
        let mut stale_typed = typed.clone();
        match partial {
            0 => stale_typed.remove_source_composite_formula_for_test(),
            1 => stale_typed.remove_source_formula_composition_for_test(),
            2 => stale_typed.remove_source_atomic_formula_for_test(),
            _ => unreachable!(),
        }
        assert_ne!(stale_typed, typed, "partial tuple {partial} actual delta");
        let stale_before = stale_typed.clone();
        let run = || assemble_empty_resolved(&stale_typed);
        assert_eq!(
            run(),
            Err(ResolvedTypedAstError::InvalidSourceStatement),
            "partial lower tuple {partial}"
        );
        assert_eq!(run(), run(), "partial lower tuple {partial} replay");
        assert_eq!(stale_typed, stale_before);
        assert_eq!(fixture.typed(), typed);
        assert_eq!(assemble_empty_resolved(&typed), Ok(resolved.clone()));
    }
    assert_eq!(assemble_empty_resolved(&typed), Ok(resolved));
}

#[test]
fn task_258b4b_connective_grouping_root_profile_and_debug_are_exact() {
    let fixture = Task258B4BFixture::new();
    let statement = fixture.statement();
    assert!(statement.is_task_258b4b_profile());
    assert!(!statement.is_task_258b4a_profile());
    assert_eq!(statement.owners().len(), 1);
    assert_eq!(statement.statements().len(), 1);
    assert_eq!(statement.contexts().len(), 1);
    assert!(statement.input_facts().is_empty());
    assert_eq!(statement.candidate_facts().len(), 1);
    assert_eq!(
        statement.composite_formula_fingerprint(),
        Some(fixture.composite.debug_text().as_str())
    );
    assert_eq!(
        statement.formula_composition_fingerprint(),
        Some(fixture.composition.debug_text().as_str())
    );
    let owner = statement
        .owners()
        .get(SourceTheoremOwnerId::new(0))
        .expect("Task258B4B owner");
    assert_eq!(owner.symbol(), &fixture.symbol);
    assert_eq!(owner.contribution(), fixture.contribution);
    assert_eq!(owner.site(), &node(120));
    assert_eq!(owner.source_range(), range(fixture.source, 0, 165));
    assert_eq!(owner.spelling(), "FormulaConnectiveGroupingPayloadBoundary");
    let row = statement
        .statements()
        .get(SourceStatementId::new(0))
        .expect("Task258B4B statement");
    assert_eq!(
        row.formula(),
        SourceStatementFormulaTarget::Composite(SourceCompositeFormulaId::new(0))
    );
    assert_eq!(row.site(), &node(120));
    assert_eq!(row.source_range(), range(fixture.source, 0, 165));
    assert_eq!(
        row.spelling(),
        "theorem FormulaConnectiveGroupingPayloadBoundary : for x being set holds ( ( 0 = 0 & ... & 0 = 3 ) or ( 0 = 0 or ... or 0 = 3 ) ) iff ( ( 0 = 0 & 0 = 0 ) or ( 0 = 0 or 0 = 0 ) ) ;"
    );
    assert_eq!(
        statement
            .candidate_facts()
            .get(SourceStatementCandidateFactId::new(0))
            .expect("Task258B4B candidate")
            .formula(),
        row.formula()
    );
    assert_eq!(
        fixture
            .composite
            .roots()
            .get(crate::source_composite_formula::SourceFormulaRootId::new(0))
            .expect("Task258B4B lower root")
            .ownership(),
        SourceFormulaRootOwnership::UnassignedStatement
    );
    let owned = fixture
        .arena
        .iter()
        .filter_map(|(id, node)| (!node.kind.as_str().ends_with(".unowned")).then_some(id.index()))
        .collect::<Vec<_>>();
    assert_eq!(owned.len(), 43);
    assert!(owned.contains(&120));
    assert!(statement.debug_text().contains("formula=composite:0"));
}

#[test]
fn task_258b4b_connective_grouping_root_corruption_matrix_fails_atomically() {
    let fixture = Task258B4BFixture::new();
    let baseline = fixture.statement();
    for mutation in 0..24 {
        let mut input = fixture.input();
        match mutation {
            0 => input.owners.clear(),
            1 => input.owners.push(input.owners[0].clone()),
            2 => input.owners[0].site = node(119),
            3 => input.owners[0].source_range.end -= 1,
            4 => input.owners[0].spelling.push('!'),
            5 => {
                input.owners[0].symbol = SymbolId::new(
                    fixture.module.clone(),
                    LocalSymbolId::new("Other"),
                    FullyQualifiedName::new("pkg::statement.task258b4b.fixture::Other"),
                )
            }
            6 => input.owners[0].source_range = range(source_id(50_999), 0, 165),
            7 => input.owners[0].recovery = SourceStatementRecovery::Degraded,
            8 => input.statements.clear(),
            9 => input.statements.push(input.statements[0].clone()),
            10 => {
                input.statements[0].formula =
                    SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(0))
            }
            11 => input.statements[0].site = node(119),
            12 => input.statements[0].source_range.end -= 1,
            13 => input.statements[0].source_ordinal = 1,
            14 => input.statements[0].spelling.push('!'),
            15 => input.statements[0].kind = SourceStatementKind::Conclusion,
            16 => input.statements[0].recovery = SourceStatementRecovery::Degraded,
            17 => input.contexts.clear(),
            18 => input.contexts[0].binding_context = BindingContextId::new(1),
            19 => input.contexts[0].visible_bindings.push(BindingId::new(0)),
            20 => input.input_facts.push(SourceStatementInputFactInput {
                statement: SourceStatementId::new(0),
                context: SourceStatementContextId::new(0),
                ordinal: 0,
                kind: SourceStatementInputFactKind::ReservedTypeGuard,
                binding: BindingId::new(0),
                uses: Vec::new(),
            }),
            21 => input.candidate_facts.clear(),
            22 => input.candidate_facts[0].ordinal = 1,
            23 => {
                input.candidate_facts[0].formula =
                    SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(0))
            }
            _ => unreachable!(),
        }
        let run = || fixture.build_statement(input.clone());
        assert!(run().is_err(), "upper mutation {mutation}");
        assert_eq!(run(), run(), "upper mutation {mutation} replay");
    }

    for mutation in 0..5 {
        let mut corrupted = baseline.clone();
        match mutation {
            0 => corrupted.composite_formula_fingerprint = None,
            1 => corrupted.formula_composition_fingerprint = None,
            2 => corrupted.composite_formula_fingerprint = Some("stale".to_owned()),
            3 => corrupted.formula_composition_fingerprint = Some("stale".to_owned()),
            4 => corrupted.owners.rows[0].site = node(119),
            _ => unreachable!(),
        }
        let run = || {
            corrupted.validate_installation_with_formula_composition(
                fixture.source,
                &fixture.module,
                &fixture.primary,
                &fixture.atomic,
                &fixture.composite,
                &fixture.composition,
                &fixture.arena,
            )
        };
        let expected = if mutation == 4 {
            SourceStatementError::InvalidOwner {
                owner: SourceTheoremOwnerId::new(0),
            }
        } else {
            SourceStatementError::DependencyMismatch
        };
        assert_eq!(run(), Err(expected), "stored mutation {mutation}");
        assert_eq!(run(), run(), "stored mutation {mutation} replay");
    }

    let mut nodes = fixture
        .arena
        .iter()
        .map(|(_, node)| node.clone())
        .collect::<Vec<_>>();
    nodes[120].kind = "source.formula.composition.unowned".into();
    let arena = TypedArena::try_new(None, nodes).expect("Task258B4B corrupt arena");
    let run = || {
        SourceStatementProducer::build_with_formula_composition(
            fixture.input(),
            &fixture.symbols,
            &fixture.bindings,
            &fixture.primary,
            &fixture.atomic,
            &fixture.composite,
            &fixture.composition,
            &arena,
        )
    };
    assert!(run().is_err());
    assert_eq!(run(), run(), "arena replay");

    assert_eq!(fixture.statement().debug_text(), baseline.debug_text());
}

#[test]
fn task_258b4b_cross_family_and_b4a_installation_orders_are_atomic() {
    let fixture = Task258B4BFixture::new();
    let b4a = Task258B4AFixture::new(50_004);
    let a_before = b4a.clone();
    let hybrid = || {
        SourceStatementProducer::build_with_formula_composition(
            b4a.input(),
            &b4a.symbols,
            &b4a.bindings,
            &b4a.primary,
            &b4a.atomic,
            &fixture.composite,
            &fixture.composition,
            &b4a.arena,
        )
    };
    assert_eq!(hybrid(), Err(SourceStatementError::DependencyMismatch));
    assert_eq!(
        hybrid(),
        hybrid(),
        "foreign lower pair on B4A producer replay"
    );
    assert_eq!(b4a, a_before);
    let base = fixture.typed_base();
    let paired = fixture.typed();
    let a_paired = b4a.typed();
    let a_final = assemble_empty_resolved(&a_paired).expect("pristine B4A final");
    assert_eq!(
        a_paired.clone().with_source_formula_composition_statement(
            fixture.composite.clone(),
            fixture.composition.clone(),
            fixture.statement(),
        ),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(a_paired, b4a.typed());
    assert_eq!(assemble_empty_resolved(&a_paired), Ok(a_final));
    let pristine_final = assemble_empty_resolved(&paired).expect("pristine paired final");
    let lower = base
        .clone()
        .with_source_formula_composition(fixture.composite.clone(), fixture.composition.clone())
        .expect("independent lower-only tuple");
    assert_eq!(
        lower.clone().with_source_statement(b4a.statement()),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        paired.clone().with_source_formula_composition_statement(
            b4a.composite.clone(),
            b4a.composition.clone(),
            b4a.statement(),
        ),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert!(
        paired
            .source_statement()
            .is_some_and(SourceStatementHandoff::is_task_258b4b_profile)
    );
    assert_eq!(
        base.clone()
            .with_source_formula_composition(
                fixture.composite.clone(),
                fixture.composition.clone(),
            )
            .expect("Task258B4B lower-only")
            .with_source_statement(fixture.statement()),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        base.clone().with_source_statement(fixture.statement()),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        base.clone().with_source_formula_composition_statement(
            fixture.composite.clone(),
            fixture.composition.clone(),
            b4a.statement(),
        ),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        b4a.typed_base().with_source_formula_composition_statement(
            b4a.composite.clone(),
            b4a.composition.clone(),
            fixture.statement(),
        ),
        Err(TypedAstError::InvalidSourceStatement)
    );
    let before = paired.clone();
    assert_eq!(
        paired.clone().with_source_statement(b4a.statement()),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(paired, before);
    let replay = fixture.typed();
    assert_eq!(replay, before);
    assert_eq!(assemble_empty_resolved(&replay), Ok(pristine_final));
    assert_eq!(base, fixture.typed_base());
    assert_eq!(
        lower,
        base.with_source_formula_composition(
            fixture.composite.clone(),
            fixture.composition.clone(),
        )
        .expect("lower-only pristine replay")
    );
    assert!(b4a.statement().is_task_258b4a_profile());
    assert!(!b4a.statement().is_task_258b4b_profile());
}

#[test]
fn task_258b4b_final_clone_revalidates_connective_grouping_root_and_empty_semantics() {
    let fixture = Task258B4BFixture::new();
    let typed = fixture.typed();
    let resolved = assemble_empty_resolved(&typed).expect("Task258B4B final");
    assert_eq!(typed.source_statement(), resolved.source_statement());
    assert_eq!(
        typed.source_composite_formula(),
        resolved.source_composite_formula()
    );
    assert_eq!(
        typed.source_formula_composition(),
        resolved.source_formula_composition()
    );
    assert_eq!(resolved.clone(), resolved);
    assert_eq!(resolved.clone().debug_text(), resolved.debug_text());
    assert!(resolved.expr_metadata().is_empty());
    assert!(resolved.checked_formulas().is_empty());
    assert!(resolved.statement_semantics().is_empty());
    assert!(resolved.checked_proofs().is_empty());
    assert!(resolved.checked_proof_nodes().is_empty());
    assert!(resolved.checked_terminal_goals().is_empty());

    for mutation in 0..7 {
        let mut stale = fixture.statement();
        match mutation {
            0 => stale.composite_formula_fingerprint = None,
            1 => stale.formula_composition_fingerprint = None,
            2 => stale.owners.rows.clear(),
            3 => {
                stale.statements.rows[0].formula =
                    SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(0))
            }
            4 => stale.contexts.rows[0]
                .visible_bindings
                .push(BindingId::new(0)),
            5 => stale.candidate_facts.rows[0].ordinal = 1,
            6 => stale.source_id = source_id(50_104),
            _ => unreachable!(),
        }
        let mut stale_typed = typed.clone();
        stale_typed.inject_source_statement_for_test(stale);
        let run = || assemble_empty_resolved(&stale_typed);
        assert_eq!(
            run(),
            Err(ResolvedTypedAstError::InvalidSourceStatement),
            "final mutation {mutation}"
        );
        assert_eq!(run(), run(), "final mutation {mutation} replay");
    }
    for partial in 0..3 {
        let mut stale = typed.clone();
        match partial {
            0 => stale.remove_source_composite_formula_for_test(),
            1 => stale.remove_source_formula_composition_for_test(),
            2 => stale.remove_source_atomic_formula_for_test(),
            _ => unreachable!(),
        }
        assert_eq!(
            assemble_empty_resolved(&stale),
            Err(ResolvedTypedAstError::InvalidSourceStatement)
        );
    }
    assert_eq!(
        assemble_empty_resolved(&typed)
            .expect("Task258B4B clean replay")
            .debug_text(),
        resolved.debug_text()
    );
}

#[test]
fn task_258b4c_nested_quantifier_root_profile_and_debug_are_exact() {
    let fixture = Task258B4CFixture::new();
    let statement = fixture.statement();
    assert!(statement.is_task_258b4c_profile());
    assert!(!statement.is_task_258b4a_profile());
    assert!(!statement.is_task_258b4b_profile());
    assert_eq!(statement.owners().len(), 1);
    assert_eq!(statement.statements().len(), 1);
    assert_eq!(statement.contexts().len(), 1);
    assert!(statement.input_facts().is_empty());
    assert_eq!(statement.candidate_facts().len(), 1);
    assert_eq!(
        statement.composite_formula_fingerprint(),
        Some(fixture.composite.debug_text().as_str())
    );
    assert_eq!(
        statement.formula_composition_fingerprint(),
        Some(fixture.composition.debug_text().as_str())
    );
    let owner = statement
        .owners()
        .get(SourceTheoremOwnerId::new(0))
        .expect("Task258B4C owner");
    assert_eq!(owner.symbol(), &fixture.symbol);
    assert_eq!(owner.contribution(), fixture.contribution);
    assert_eq!(owner.site(), &node(62));
    assert_eq!(owner.source_range(), range(fixture.source, 19, 137));
    assert_eq!(owner.spelling(), "FormulaNestedQuantifierPayloadBoundary");
    let row = statement
        .statements()
        .get(SourceStatementId::new(0))
        .expect("Task258B4C statement");
    assert_eq!(
        row.formula(),
        SourceStatementFormulaTarget::Composite(SourceCompositeFormulaId::new(0))
    );
    assert_eq!(row.site(), &node(62));
    assert_eq!(row.source_range(), range(fixture.source, 19, 137));
    assert_eq!(
        row.spelling(),
        "theorem FormulaNestedQuantifierPayloadBoundary : for x being set st x = x ex y being set st for r st r = y holds x = r ;"
    );
    let context = statement
        .contexts()
        .get(SourceStatementContextId::new(0))
        .expect("Task258B4C context");
    assert_eq!(context.visible_bindings(), &[BindingId::new(0)]);
    assert_eq!(
        statement
            .candidate_facts()
            .get(SourceStatementCandidateFactId::new(0))
            .expect("Task258B4C candidate")
            .formula(),
        row.formula()
    );
    assert_eq!(
        fixture
            .composite
            .roots()
            .get(crate::source_composite_formula::SourceFormulaRootId::new(0))
            .expect("Task258B4C lower root")
            .ownership(),
        SourceFormulaRootOwnership::UnassignedStatement
    );
    assert_eq!(statement.binding_env(), &fixture.bindings);
    assert_eq!(
        (
            fixture.bindings.contexts().len(),
            fixture.bindings.bindings().len(),
            fixture.bindings.diagnostics().len()
        ),
        (4, 4, 0)
    );
    assert_eq!(
        statement.binding_fingerprint(),
        fixture.bindings.debug_text()
    );
    assert_eq!(
        statement.primary_term_fingerprint(),
        fixture.primary.debug_text()
    );
    assert_eq!(
        statement.atomic_formula_fingerprint(),
        fixture.atomic.debug_text()
    );
    assert_eq!(
        (owner.role(), owner.status(), owner.recovery()),
        (
            SourceTheoremRole::Theorem,
            SourceTheoremStatus::Unmodified,
            SourceStatementRecovery::Normal
        )
    );
    assert_eq!(statement.checked_owner().origin().structural_path(), [2, 1]);
    assert_eq!(
        (
            row.owner(),
            row.context(),
            row.source_ordinal(),
            row.kind(),
            row.recovery()
        ),
        (
            SourceTheoremOwnerId::new(0),
            SourceStatementContextId::new(0),
            0,
            SourceStatementKind::TheoremProposition,
            SourceStatementRecovery::Normal
        )
    );
    assert_eq!(
        (
            context.statement(),
            context.binding_context(),
            context.source_range()
        ),
        (
            SourceStatementId::new(0),
            BindingContextId::new(0),
            range(fixture.source, 19, 137)
        )
    );
    let candidate = statement
        .candidate_facts()
        .get(SourceStatementCandidateFactId::new(0))
        .expect("Task258B4C candidate");
    assert_eq!(
        (
            candidate.statement(),
            candidate.context(),
            candidate.ordinal(),
            candidate.kind()
        ),
        (
            SourceStatementId::new(0),
            SourceStatementContextId::new(0),
            0,
            SourceStatementCandidateFactKind::UnverifiedProposition
        )
    );
    assert_eq!(
        (
            fixture.primary.terms().len(),
            fixture.primary.references().len(),
            fixture.primary.numeric_type_requests().len()
        ),
        (6, 6, 0)
    );
    assert_eq!(
        fixture
            .primary
            .references()
            .iter()
            .map(|(_, reference)| (reference.binding().index(), reference.use_ordinal()))
            .collect::<Vec<_>>(),
        [(1, 2), (1, 2), (3, 4), (2, 4), (1, 4), (3, 4)]
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
            fixture.atomic.requests().len()
        ),
        (3, 0, 0, 0, 0, 0, 0, 6, 6)
    );
    assert_eq!(
        (
            fixture.composite.formulas().len(),
            fixture.composite.wrappers().len(),
            fixture.composite.roots().len(),
            fixture.composite.binders().len(),
            fixture.composite.type_sites().len(),
            fixture.composite.edges().len(),
            fixture.composite.requests().len(),
            fixture.composition.atomic_edges().len(),
            fixture.composition.bound_uses().len()
        ),
        (3, 0, 1, 3, 3, 2, 6, 3, 6)
    );
    assert_eq!(fixture.arena.len(), 66);
    assert!(fixture.arena.root().is_none());
    assert!(
        fixture
            .arena
            .iter()
            .all(|(_, node)| node.recovery == NodeRecoveryState::Normal)
    );
    let owned = fixture
        .arena
        .iter()
        .filter_map(|(id, node)| (!node.kind.as_str().ends_with(".unowned")).then_some(id.index()))
        .collect::<Vec<_>>();
    assert_eq!(owned.len(), 25);
    assert_eq!(
        owned,
        [
            9, 17, 22, 32, 33, 36, 37, 38, 39, 41, 43, 44, 45, 46, 47, 48, 50, 52, 53, 55, 57, 58,
            59, 60, 62,
        ]
    );
    assert!(statement.debug_text().contains("formula=composite:0"));
}

#[test]
fn task_258b4c_nested_quantifier_root_corruption_matrix_fails_atomically() {
    let fixture = Task258B4CFixture::new();
    let pristine = fixture.clone();
    let baseline = fixture.statement();
    let typed = fixture.typed();
    let resolved = assemble_empty_resolved(&typed).expect("Task258B4C pristine final");
    let mut contributions = SourceContributionIndex::new();
    contributions.insert(
        fixture.module.clone(),
        ContributionKind::LocalSource {
            source_id: fixture.source,
        },
        SourceAnchor::Range(range(fixture.source, 0, 1)),
    );
    let foreign_contribution = contributions.insert(
        fixture.module.clone(),
        ContributionKind::LocalSource {
            source_id: fixture.source,
        },
        SourceAnchor::Range(range(fixture.source, 1, 2)),
    );
    assert_eq!(foreign_contribution.index(), 1);
    for mutation in 0..38 {
        let mut input = fixture.input();
        match mutation {
            0 => input.owners.clear(),
            1 => input.owners.push(input.owners[0].clone()),
            2 => input.owners[0].site = node(61),
            3 => input.owners[0].source_range.end -= 1,
            4 => input.owners[0].spelling.push('!'),
            5 => {
                input.owners[0].symbol = SymbolId::new(
                    fixture.module.clone(),
                    LocalSymbolId::new("Other"),
                    FullyQualifiedName::new("pkg::statement.task258b4c.fixture::Other"),
                )
            }
            6 => input.owners[0].source_range = range(source_id(50_999), 19, 137),
            7 => input.owners[0].recovery = SourceStatementRecovery::Degraded,
            8 => input.statements.clear(),
            9 => input.statements.push(input.statements[0].clone()),
            10 => {
                input.statements[0].formula =
                    SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(0))
            }
            11 => input.statements[0].site = node(61),
            12 => input.statements[0].source_range.start += 1,
            13 => input.statements[0].source_ordinal = 1,
            14 => input.statements[0].spelling.push('!'),
            15 => input.statements[0].kind = SourceStatementKind::Conclusion,
            16 => input.statements[0].recovery = SourceStatementRecovery::Degraded,
            17 => input.contexts.clear(),
            18 => input.contexts[0].binding_context = BindingContextId::new(1),
            19 => input.contexts[0].visible_bindings.clear(),
            20 => input.input_facts.push(SourceStatementInputFactInput {
                statement: SourceStatementId::new(0),
                context: SourceStatementContextId::new(0),
                ordinal: 0,
                kind: SourceStatementInputFactKind::ReservedTypeGuard,
                binding: BindingId::new(0),
                uses: Vec::new(),
            }),
            21 => input.candidate_facts.clear(),
            22 => input.candidate_facts[0].ordinal = 1,
            23 => {
                input.candidate_facts[0].formula =
                    SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(0))
            }
            24 => input.source_id = source_id(50_999),
            25 => {
                input.module_id =
                    ModuleId::new(PackageId::new("pkg"), ModulePath::new("statement.foreign"))
            }
            26 => input.owners[0].contribution = foreign_contribution,
            27 => input.owners[0].source_range.start += 1,
            28 => input.statements[0].owner = SourceTheoremOwnerId::new(1),
            29 => input.statements[0].context = SourceStatementContextId::new(1),
            30 => input.statements[0].source_range.end -= 1,
            31 => input.contexts.push(input.contexts[0].clone()),
            32 => input.contexts[0].statement = SourceStatementId::new(1),
            33 => input.contexts[0].source_range.start += 1,
            34 => input.contexts[0].source_range.end -= 1,
            35 => input.candidate_facts.push(input.candidate_facts[0].clone()),
            36 => input.candidate_facts[0].statement = SourceStatementId::new(1),
            37 => input.candidate_facts[0].context = SourceStatementContextId::new(1),
            _ => unreachable!(),
        }
        assert_ne!(
            input,
            fixture.input(),
            "upper mutation {mutation} actual delta"
        );
        let before = input.clone();
        let run = || fixture.build_statement(input.clone());
        let expected = match mutation {
            0 | 1 | 8 | 9 | 17 | 20 | 21 | 31 | 35 => SourceStatementError::InvalidAggregate,
            2..=7 | 26 | 27 => SourceStatementError::InvalidOwner {
                owner: SourceTheoremOwnerId::new(0),
            },
            10..=16 | 28..=30 => SourceStatementError::InvalidStatement {
                statement: SourceStatementId::new(0),
            },
            18 | 19 | 32..=34 => SourceStatementError::InvalidContext {
                context: SourceStatementContextId::new(0),
            },
            22 | 23 | 36 | 37 => SourceStatementError::InvalidCandidateFact {
                fact: SourceStatementCandidateFactId::new(0),
            },
            24 | 25 => SourceStatementError::DependencyMismatch,
            _ => unreachable!(),
        };
        assert_eq!(run(), Err(expected), "upper mutation {mutation}");
        assert_eq!(run(), run(), "upper mutation {mutation} replay");
        assert_eq!(input, before, "upper mutation {mutation} input purity");
        assert_eq!(
            fixture, pristine,
            "upper mutation {mutation} dependency purity"
        );
        assert_eq!(
            fixture.typed(),
            typed,
            "upper mutation {mutation} typed replay"
        );
        assert_eq!(
            assemble_empty_resolved(&fixture.typed()),
            Ok(resolved.clone())
        );
    }

    for mutation in 0..8 {
        let mut corrupted = baseline.clone();
        match mutation {
            0 => corrupted.binding_fingerprint = "stale".to_owned(),
            1 => corrupted.primary_term_fingerprint = "stale".to_owned(),
            2 => corrupted.atomic_formula_fingerprint = "stale".to_owned(),
            3 => corrupted.composite_formula_fingerprint = None,
            4 => corrupted.formula_composition_fingerprint = None,
            5 => corrupted.composite_formula_fingerprint = Some("stale".to_owned()),
            6 => corrupted.formula_composition_fingerprint = Some("stale".to_owned()),
            7 => corrupted.owners.rows[0].site = node(61),
            _ => unreachable!(),
        }
        assert_ne!(
            corrupted, baseline,
            "stored mutation {mutation} actual delta"
        );
        let before = corrupted.clone();
        let run = || {
            corrupted.validate_installation_with_formula_composition(
                fixture.source,
                &fixture.module,
                &fixture.primary,
                &fixture.atomic,
                &fixture.composite,
                &fixture.composition,
                &fixture.arena,
            )
        };
        let expected = if mutation == 7 {
            SourceStatementError::InvalidOwner {
                owner: SourceTheoremOwnerId::new(0),
            }
        } else {
            SourceStatementError::DependencyMismatch
        };
        assert_eq!(run(), Err(expected), "stored mutation {mutation}");
        assert_eq!(run(), run(), "stored mutation {mutation} replay");
        assert_eq!(corrupted, before);
        assert_eq!(fixture, pristine);
        assert_eq!(fixture.typed(), typed);
        assert_eq!(
            assemble_empty_resolved(&fixture.typed()),
            Ok(resolved.clone())
        );
    }

    const OWNED_NODES: [usize; 25] = [
        9, 17, 22, 32, 33, 36, 37, 38, 39, 41, 43, 44, 45, 46, 47, 48, 50, 52, 53, 55, 57, 58, 59,
        60, 62,
    ];
    for node_index in OWNED_NODES {
        for mutation in 0..2 {
            let mut nodes = fixture
                .arena
                .iter()
                .map(|(_, node)| node.clone())
                .collect::<Vec<_>>();
            match mutation {
                0 => nodes[node_index].kind = "source.formula.composition.unowned".into(),
                1 => nodes[node_index].anchor = SourceAnchor::Range(range(fixture.source, 0, 0)),
                _ => unreachable!(),
            }
            let arena = TypedArena::try_new(None, nodes).expect("Task258B4C owned-node arena");
            assert_ne!(
                arena, fixture.arena,
                "owned node {node_index} mutation {mutation} actual delta"
            );
            let before = arena.clone();
            let run = || {
                SourceStatementProducer::build_with_formula_composition(
                    fixture.input(),
                    &fixture.symbols,
                    &fixture.bindings,
                    &fixture.primary,
                    &fixture.atomic,
                    &fixture.composite,
                    &fixture.composition,
                    &arena,
                )
            };
            assert_eq!(
                run(),
                Err(SourceStatementError::DependencyMismatch),
                "owned node {node_index} mutation {mutation}"
            );
            assert_eq!(
                run(),
                run(),
                "owned node {node_index} mutation {mutation} replay"
            );
            assert_eq!(arena, before);
            assert_eq!(fixture, pristine);
            assert_eq!(fixture.typed(), typed);
            assert_eq!(
                assemble_empty_resolved(&fixture.typed()),
                Ok(resolved.clone())
            );
        }
    }
    let rooted = TypedArena::try_new(
        Some(TypedNodeId::new(65)),
        fixture.arena.iter().map(|(_, node)| node.clone()).collect(),
    )
    .expect("Task258B4C rooted arena");
    assert_ne!(rooted, fixture.arena);
    let before = rooted.clone();
    let run = || {
        SourceStatementProducer::build_with_formula_composition(
            fixture.input(),
            &fixture.symbols,
            &fixture.bindings,
            &fixture.primary,
            &fixture.atomic,
            &fixture.composite,
            &fixture.composition,
            &rooted,
        )
    };
    assert_eq!(run(), Err(SourceStatementError::DependencyMismatch));
    assert_eq!(run(), run(), "rooted arena replay");
    assert_eq!(rooted, before);
    assert_eq!(fixture, pristine);
    assert_eq!(fixture.typed(), typed);
    assert_eq!(
        assemble_empty_resolved(&fixture.typed()),
        Ok(resolved.clone())
    );

    const UNOWNED_NODES: [usize; 41] = [
        0, 1, 2, 3, 4, 5, 6, 7, 8, 10, 11, 12, 13, 14, 15, 16, 18, 19, 20, 21, 23, 24, 25, 26, 27,
        28, 29, 30, 31, 34, 35, 40, 42, 49, 51, 54, 56, 61, 63, 64, 65,
    ];
    for node_index in UNOWNED_NODES {
        for mutation in 0..3 {
            let mut nodes = fixture
                .arena
                .iter()
                .map(|(_, node)| node.clone())
                .collect::<Vec<_>>();
            match mutation {
                0 => nodes[node_index].kind = "source.formula.unexpected".into(),
                1 => nodes[node_index].anchor = SourceAnchor::Range(range(fixture.source, 0, 0)),
                2 => nodes[node_index].recovery = NodeRecoveryState::Degraded,
                _ => unreachable!(),
            }
            let arena = TypedArena::try_new(None, nodes).expect("Task258B4C unowned-node arena");
            assert_ne!(
                arena, fixture.arena,
                "unowned node {node_index} mutation {mutation} actual delta"
            );
            let before = arena.clone();
            let run = || {
                SourceStatementProducer::build_with_formula_composition(
                    fixture.input(),
                    &fixture.symbols,
                    &fixture.bindings,
                    &fixture.primary,
                    &fixture.atomic,
                    &fixture.composite,
                    &fixture.composition,
                    &arena,
                )
            };
            assert_eq!(
                run(),
                Err(SourceStatementError::DependencyMismatch),
                "unowned node {node_index} mutation {mutation}"
            );
            assert_eq!(
                run(),
                run(),
                "unowned node {node_index} mutation {mutation} replay"
            );
            assert_eq!(arena, before);
            assert_eq!(fixture, pristine);
            assert_eq!(fixture.typed(), typed);
            assert_eq!(
                assemble_empty_resolved(&fixture.typed()),
                Ok(resolved.clone())
            );
        }
    }
    assert_eq!(fixture.statement(), baseline);
}

#[test]
fn task_258b4c_cross_family_b4a_b4b_installation_orders_are_atomic() {
    let fixture = Task258B4CFixture::new();
    let b4a = Task258B4AFixture::new(50_005);
    let b4b = Task258B4BFixture::new();
    let a_before = b4a.clone();
    let b_before = b4b.clone();
    let c_before = fixture.clone();
    let hybrid = || {
        SourceStatementProducer::build_with_formula_composition(
            b4a.input(),
            &b4a.symbols,
            &b4a.bindings,
            &b4a.primary,
            &b4a.atomic,
            &fixture.composite,
            &fixture.composition,
            &b4a.arena,
        )
    };
    assert_eq!(hybrid(), Err(SourceStatementError::DependencyMismatch));
    assert_eq!(
        hybrid(),
        hybrid(),
        "foreign lower pair on B4A producer replay"
    );
    assert_eq!(b4a, a_before);
    for (composite, composition) in [
        (&b4a.composite, &b4a.composition),
        (&b4b.composite, &b4b.composition),
    ] {
        assert_ne!(composite, &fixture.composite);
        assert_ne!(composition, &fixture.composition);
        let run = || {
            SourceStatementProducer::build_with_formula_composition(
                fixture.input(),
                &fixture.symbols,
                &fixture.bindings,
                &fixture.primary,
                &fixture.atomic,
                composite,
                composition,
                &fixture.arena,
            )
        };
        assert_eq!(run(), Err(SourceStatementError::DependencyMismatch));
        assert_eq!(run(), run(), "foreign lower pair on C producer replay");
    }
    let run = || {
        SourceStatementProducer::build_with_formula_composition(
            b4b.input(),
            &b4b.symbols,
            &b4b.bindings,
            &b4b.primary,
            &b4b.atomic,
            &fixture.composite,
            &fixture.composition,
            &b4b.arena,
        )
    };
    assert_ne!(fixture.composite, b4b.composite);
    assert_ne!(fixture.composition, b4b.composition);
    assert_eq!(run(), Err(SourceStatementError::DependencyMismatch));
    assert_eq!(run(), run(), "C lower pair on B producer replay");
    let base = fixture.typed_base();
    let paired = fixture.typed();
    let pristine_final = assemble_empty_resolved(&paired).expect("pristine paired final");
    let lower = base
        .clone()
        .with_source_formula_composition(fixture.composite.clone(), fixture.composition.clone())
        .expect("independent lower-only tuple");
    assert_eq!(
        lower.clone().with_source_statement(b4a.statement()),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        paired.clone().with_source_formula_composition_statement(
            b4a.composite.clone(),
            b4a.composition.clone(),
            b4a.statement(),
        ),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert!(
        paired
            .source_statement()
            .is_some_and(SourceStatementHandoff::is_task_258b4c_profile)
    );
    assert_eq!(
        base.clone()
            .with_source_formula_composition(
                fixture.composite.clone(),
                fixture.composition.clone(),
            )
            .expect("Task258B4C lower-only")
            .with_source_statement(fixture.statement()),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        base.clone().with_source_statement(fixture.statement()),
        Err(TypedAstError::InvalidSourceStatement)
    );
    for foreign in [b4a.statement(), b4b.statement()] {
        assert_eq!(
            base.clone().with_source_formula_composition_statement(
                fixture.composite.clone(),
                fixture.composition.clone(),
                foreign,
            ),
            Err(TypedAstError::InvalidSourceStatement)
        );
    }
    assert_eq!(
        b4a.typed_base().with_source_formula_composition_statement(
            b4a.composite.clone(),
            b4a.composition.clone(),
            fixture.statement(),
        ),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        b4b.typed_base().with_source_formula_composition_statement(
            b4b.composite.clone(),
            b4b.composition.clone(),
            fixture.statement(),
        ),
        Err(TypedAstError::InvalidSourceStatement)
    );
    let a_lower = b4a
        .typed_base()
        .with_source_formula_composition(b4a.composite.clone(), b4a.composition.clone())
        .expect("A lower-only donor");
    let b_lower = b4b
        .typed_base()
        .with_source_formula_composition(b4b.composite.clone(), b4b.composition.clone())
        .expect("B lower-only donor");
    let atomic = Fixture::new(50_105);
    let atomic_statement = atomic
        .build(atomic.input())
        .expect("atomic statement donor");
    let atomic_typed = TypedAst::try_new(TypedAstParts {
        source_id: atomic.source,
        module_id: atomic.module.clone(),
        resolved_root: None,
        source_context: None,
        source_type: None,
        source_attribute: None,
        nodes: atomic.arena.clone(),
        contexts: LocalTypeContextTable::new(),
        types: TypeTable::new(),
        facts: TypeFactTable::new(),
        coercions: CoercionTable::new(),
        initial_obligations: InitialObligationTable::new(),
        diagnostics: TypeDiagnosticTable::new(),
    })
    .expect("atomic typed donor")
    .with_source_term(atomic.primary.clone())
    .expect("atomic primary")
    .with_source_atomic_formula(atomic.atomic.clone())
    .expect("atomic formula")
    .with_source_statement(atomic_statement.clone())
    .expect("atomic statement");
    let atomic_before = atomic_typed.clone();
    let a_lower_before = a_lower.clone();
    let b_lower_before = b_lower.clone();
    let run = || {
        [
            lower.clone().with_source_statement(b4b.statement()),
            a_lower.clone().with_source_statement(fixture.statement()),
            b_lower.clone().with_source_statement(fixture.statement()),
            paired.clone().with_source_formula_composition_statement(
                b4b.composite.clone(),
                b4b.composition.clone(),
                b4b.statement(),
            ),
            lower
                .clone()
                .with_source_statement(atomic_statement.clone()),
            atomic_typed
                .clone()
                .with_source_formula_composition_statement(
                    fixture.composite.clone(),
                    fixture.composition.clone(),
                    fixture.statement(),
                ),
            paired
                .clone()
                .with_source_statement(atomic_statement.clone()),
        ]
    };
    for result in run() {
        assert_eq!(result, Err(TypedAstError::InvalidSourceStatement));
    }
    assert_eq!(run(), run(), "cross-family setter replay");
    assert_eq!(atomic_typed, atomic_before);
    assert_eq!(a_lower, a_lower_before);
    assert_eq!(b_lower, b_lower_before);
    assert_eq!(atomic.build(atomic.input()), Ok(atomic_statement));

    use crate::source_context::{
        SourceBindingContextInput, SourceBindingContextOwner, SourceBindingContextProducer,
        SourceBindingSiteInput, SourceBindingSiteRole, SourceItemInput, SourceItemRecovery,
        SourceItemRole, SourceItemVisibility,
    };
    let mut surface = syntax::SurfaceAstBuilder::new(fixture.source);
    let parameter = surface.add_node(
        syntax::SurfaceNodeKind::DefinitionParameter,
        range(fixture.source, 22, 36),
        Vec::new(),
    );
    let property = surface.add_node(
        syntax::SurfaceNodeKind::PropertyImplementation,
        range(fixture.source, 20, 80),
        vec![parameter],
    );
    let root = surface.add_node(
        syntax::SurfaceNodeKind::Root,
        range(fixture.source, 0, 80),
        vec![property],
    );
    let surface = surface.finish(Some(root), None);
    let shells =
        mizar_resolve::declarations::DeclarationShellCollector::new(&surface, &fixture.module)
            .collect();
    let shell = shells.declarations()[0].id();
    let scope = LocalTermScope::new(vec![0]);
    let declaration_range = range(fixture.source, 24, 25);
    let context_input = SourceBindingContextInput {
        source_id: fixture.source,
        module_id: fixture.module.clone(),
        module_site: node(0),
        items: vec![SourceItemInput {
            shell,
            shell_ordinal: 0,
            role: SourceItemRole::PropertyImplementation,
            module_id: fixture.module.clone(),
            source_range: range(fixture.source, 20, 80),
            parent: None,
            visibility: SourceItemVisibility::Unspecified,
            site: node(1),
            local_scope: Some(scope.clone()),
            recovery: SourceItemRecovery::Normal,
        }],
        bindings: vec![SourceBindingSiteInput {
            shell,
            context_owner: SourceBindingContextOwner::Shell(shell),
            source_ordinal: 0,
            spelling: "M".to_owned(),
            declaration_range,
            written_type_range: range(fixture.source, 29, 35),
            site: node(2),
            role: SourceBindingSiteRole::DefinitionParameter {
                local: LocalTermBinding::new("M", scope, declaration_range, 0),
            },
            recovery: BindingRecoveryState::Normal,
        }],
    };
    let context_before = context_input.clone();
    let projection = SourceBindingContextProducer::build(context_input.clone())
        .expect("occupied source context build")
        .into_complete()
        .expect("complete source context");
    let handoff = projection.handoff().clone();
    let nodes = TypedArena::try_new(
        Some(TypedNodeId::new(0)),
        vec![
            TypedNode::new(
                "source.module",
                SourceAnchor::Range(range(fixture.source, 0, 80)),
            )
            .with_children(vec![TypedNodeId::new(1)])
            .with_links(TypedNodeLinks {
                context: Some(LocalTypeContextId::new(0)),
                ..TypedNodeLinks::default()
            }),
            TypedNode::new(
                "source.property-implementation",
                SourceAnchor::Range(range(fixture.source, 20, 80)),
            )
            .with_children(vec![TypedNodeId::new(2)])
            .with_links(TypedNodeLinks {
                context: Some(LocalTypeContextId::new(1)),
                ..TypedNodeLinks::default()
            }),
            TypedNode::new(
                "source.definition-parameter",
                SourceAnchor::Range(declaration_range),
            )
            .with_links(TypedNodeLinks {
                context: Some(LocalTypeContextId::new(1)),
                ..TypedNodeLinks::default()
            }),
        ],
    )
    .expect("occupied source context arena");
    let occupied = TypedAst::try_new(TypedAstParts {
        source_id: fixture.source,
        module_id: fixture.module.clone(),
        resolved_root: None,
        source_context: Some(handoff.clone()),
        source_type: None,
        source_attribute: None,
        nodes,
        contexts: handoff.local_contexts().clone(),
        types: TypeTable::new(),
        facts: TypeFactTable::new(),
        coercions: CoercionTable::new(),
        initial_obligations: InitialObligationTable::new(),
        diagnostics: TypeDiagnosticTable::new(),
    })
    .expect("genuinely valid occupied source context AST");
    assert_eq!(occupied.source_context(), Some(&handoff));
    assert_eq!(handoff.items().len(), 1);
    assert_eq!(handoff.declarations().len(), 1);
    let occupied_before = occupied.clone();
    let run = || {
        occupied.clone().with_source_formula_composition_statement(
            fixture.composite.clone(),
            fixture.composition.clone(),
            fixture.statement(),
        )
    };
    assert_eq!(run(), Err(TypedAstError::InvalidSourceStatement));
    assert_eq!(run(), run(), "occupied source context setter replay");
    assert_eq!(occupied, occupied_before);
    assert_eq!(context_input, context_before);
    assert_eq!(
        SourceBindingContextProducer::build(context_input)
            .expect("context replay")
            .into_complete()
            .expect("complete context replay")
            .handoff(),
        &handoff
    );
    assert_eq!(b4a, a_before);
    assert_eq!(b4b, b_before);
    assert_eq!(fixture, c_before);
    let before = paired.clone();
    assert_eq!(
        paired.clone().with_source_statement(b4a.statement()),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        paired.clone().with_source_statement(b4b.statement()),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(paired, before);
    let replay = fixture.typed();
    assert_eq!(replay, before);
    assert_eq!(assemble_empty_resolved(&replay), Ok(pristine_final));
    assert_eq!(base, fixture.typed_base());
    assert_eq!(
        lower,
        base.with_source_formula_composition(
            fixture.composite.clone(),
            fixture.composition.clone(),
        )
        .expect("lower-only pristine replay")
    );
    assert!(b4a.statement().is_task_258b4a_profile());
    assert!(b4b.statement().is_task_258b4b_profile());
    assert!(!fixture.statement().is_task_258b4a_profile());
    assert!(!fixture.statement().is_task_258b4b_profile());
}

#[test]
fn task_258b4c_final_clone_revalidates_nested_quantifier_root_and_empty_semantics() {
    let fixture = Task258B4CFixture::new();
    let typed = fixture.typed();
    let resolved = assemble_empty_resolved(&typed).expect("Task258B4C final");
    assert_eq!(typed.source_statement(), resolved.source_statement());
    assert_eq!(
        typed.source_composite_formula(),
        resolved.source_composite_formula()
    );
    assert_eq!(
        typed.source_formula_composition(),
        resolved.source_formula_composition()
    );
    assert_eq!(resolved.clone(), resolved);
    assert_eq!(resolved.clone().debug_text(), resolved.debug_text());
    assert!(resolved.expr_metadata().is_empty());
    assert!(resolved.checked_formulas().is_empty());
    assert!(resolved.statement_semantics().is_empty());
    assert!(resolved.checked_proofs().is_empty());
    assert!(resolved.checked_proof_nodes().is_empty());
    assert!(resolved.checked_terminal_goals().is_empty());

    assert_eq!(typed.clone(), typed);
    assert_eq!(typed.clone().debug_text(), typed.debug_text());
    let debug = typed
        .source_statement()
        .expect("Task258B4C statement")
        .debug_text();
    let lines = debug.lines().collect::<Vec<_>>();
    let atomic = lines
        .iter()
        .position(|line| line.starts_with("atomic-formula-fingerprint:"))
        .expect("atomic fingerprint");
    let composite = lines
        .iter()
        .position(|line| line.starts_with("composite-formula-fingerprint:"))
        .expect("composite fingerprint");
    let composition = lines
        .iter()
        .position(|line| line.starts_with("formula-composition-fingerprint:"))
        .expect("composition fingerprint");
    let owner = lines
        .iter()
        .position(|line| line.starts_with("owner#0 "))
        .expect("owner");
    assert!(atomic < composite && composite < composition && composition < owner);
    assert!(debug.contains("formula=composite:0"));
    assert!(typed.source_statement_references().is_none());
    assert!(typed.source_statement_witnesses().is_none());
    assert!(typed.source_context().is_none());
    assert!(typed.source_type().is_none());
    assert!(typed.source_attribute().is_none());
    assert!(typed.source_evidence().is_none());
    assert!(typed.source_application().is_none());
    assert!(typed.source_structure().is_none());
    assert!(typed.source_set_term().is_none());
    assert!(typed.source_condition_formula_composition().is_none());
    assert!(typed.source_predicate_chain_composition().is_none());
    assert!(typed.contexts().is_empty());
    assert!(typed.types().is_empty());
    assert!(typed.facts().is_empty());
    assert!(typed.coercions().is_empty());
    assert!(typed.initial_obligations().is_empty());
    assert!(typed.diagnostics().is_empty());
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

    for mutation in 0..10 {
        let mut stale = fixture.statement();
        match mutation {
            0 => stale.binding_fingerprint = "stale".to_owned(),
            1 => stale.primary_term_fingerprint = "stale".to_owned(),
            2 => stale.atomic_formula_fingerprint = "stale".to_owned(),
            3 => stale.composite_formula_fingerprint = None,
            4 => stale.formula_composition_fingerprint = None,
            5 => stale.owners.rows.clear(),
            6 => {
                stale.statements.rows[0].formula =
                    SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(0))
            }
            7 => stale.contexts.rows[0].visible_bindings.clear(),
            8 => stale.candidate_facts.rows[0].ordinal = 1,
            9 => stale.source_id = source_id(50_105),
            _ => unreachable!(),
        }
        assert_ne!(
            &stale,
            typed.source_statement().expect("pristine statement")
        );
        let mut stale_typed = typed.clone();
        stale_typed.inject_source_statement_for_test(stale);
        let before = stale_typed.clone();
        let run = || assemble_empty_resolved(&stale_typed);
        assert_eq!(
            run(),
            Err(ResolvedTypedAstError::InvalidSourceStatement),
            "final mutation {mutation}"
        );
        assert_eq!(run(), run(), "final mutation {mutation} replay");
        assert_eq!(stale_typed, before);
        assert_eq!(fixture.typed(), typed);
        assert_eq!(
            assemble_empty_resolved(&fixture.typed()),
            Ok(resolved.clone())
        );
    }
    for partial in 0..3 {
        let mut stale = typed.clone();
        match partial {
            0 => stale.remove_source_composite_formula_for_test(),
            1 => stale.remove_source_formula_composition_for_test(),
            2 => stale.remove_source_atomic_formula_for_test(),
            _ => unreachable!(),
        }
        assert_ne!(stale, typed, "missing tuple {partial} actual delta");
        let before = stale.clone();
        let run = || assemble_empty_resolved(&stale);
        assert_eq!(run(), Err(ResolvedTypedAstError::InvalidSourceStatement));
        assert_eq!(run(), run(), "missing tuple {partial} replay");
        assert_eq!(stale, before);
        assert_eq!(fixture.typed(), typed);
        assert_eq!(
            assemble_empty_resolved(&fixture.typed()),
            Ok(resolved.clone())
        );
    }
    assert_eq!(assemble_empty_resolved(&typed), Ok(resolved));
}
