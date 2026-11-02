use super::*;

#[test]
fn task258b2_exact_assumption_profile_accessors_and_debug_are_stable() {
    let fixture = B2Fixture::new(30);
    let handoff = fixture.build(fixture.input()).expect("Task258B2 handoff");
    assert!(handoff.is_task_258b2_profile());
    assert!(!handoff.is_task_258a_profile());
    assert!(!handoff.is_task_258b1_profile());
    assert_eq!(handoff.source_id(), fixture.source);
    assert_eq!(handoff.module_id(), &fixture.module);
    assert_eq!(handoff.binding_env(), &fixture.bindings);
    assert_eq!(handoff.binding_env().contexts().len(), 2);
    assert_eq!(handoff.binding_env().bindings().len(), 1);
    assert!(handoff.binding_env().diagnostics().is_empty());
    let proof = handoff
        .binding_env()
        .contexts()
        .get(BindingContextId::new(1))
        .expect("proof context");
    assert_eq!(
        proof.owner,
        BindingContextOwner::SourceStatement {
            source_range: range(fixture.source, 72, 111),
        }
    );
    assert_eq!(proof.parent, Some(BindingContextId::new(0)));
    assert_eq!(proof.layer, BindingContextLayer::Proof);
    assert_eq!(proof.lexical_scope.as_ref().expect("scope").path(), [0]);
    assert!(proof.bindings.is_empty());
    assert_eq!(proof.visible_bindings, [BindingId::new(0)]);

    assert_eq!(fixture.primary.terms().len(), 6);
    assert_eq!(fixture.primary.references().len(), 6);
    assert!(fixture.primary.numeric_type_requests().is_empty());
    assert_eq!(
        fixture
            .primary
            .terms()
            .iter()
            .map(|(_, row)| row.context().index())
            .collect::<Vec<_>>(),
        [0, 0, 1, 1, 1, 1]
    );
    assert_eq!(
        fixture
            .primary
            .references()
            .iter()
            .map(|(_, row)| row.use_ordinal())
            .collect::<Vec<_>>(),
        [1; 6]
    );
    assert_eq!(fixture.atomic.formulas().len(), 3);
    assert_eq!(fixture.atomic.edges().len(), 6);
    assert_eq!(fixture.atomic.requests().len(), 6);
    assert_eq!(handoff.binding_fingerprint(), fixture.bindings.debug_text());
    assert_eq!(
        handoff.primary_term_fingerprint(),
        fixture.primary.debug_text()
    );
    assert_eq!(
        handoff.atomic_formula_fingerprint(),
        fixture.atomic.debug_text()
    );
    assert_eq!(
        fixture
            .atomic
            .formulas()
            .iter()
            .map(|(_, row)| row.context().index())
            .collect::<Vec<_>>(),
        [0, 1, 1]
    );

    assert_eq!(handoff.owners().len(), 1);
    assert_eq!(handoff.statements().len(), 3);
    assert_eq!(handoff.contexts().len(), 3);
    assert_eq!(handoff.input_facts().len(), 3);
    assert_eq!(handoff.candidate_facts().len(), 3);
    let owner = handoff
        .owners()
        .get(SourceTheoremOwnerId::new(0))
        .expect("owner");
    assert_eq!(owner.contribution().index(), 0);
    assert_eq!(owner.source_range(), range(fixture.source, 19, 112));
    assert_eq!(handoff.checked_owner().origin().structural_path(), [2, 1]);
    let expected = [
        (
            SourceStatementKind::TheoremProposition,
            51,
            (19, 112),
            0,
            "theorem FormulaStatementSingleAssumptionSmoke : x = x proof assume x = x ; thus x = x ; end ;",
        ),
        (
            SourceStatementKind::Assumption,
            41,
            (80, 93),
            1,
            "assume x = x ;",
        ),
        (
            SourceStatementKind::Conclusion,
            49,
            (96, 107),
            1,
            "thus x = x ;",
        ),
    ];
    for (index, (kind, site, (start, end), binding_context, spelling)) in
        expected.into_iter().enumerate()
    {
        let statement = handoff
            .statements()
            .get(SourceStatementId::new(index))
            .expect("statement");
        assert_eq!(statement.owner(), SourceTheoremOwnerId::new(0));
        assert_eq!(statement.context(), SourceStatementContextId::new(index));
        assert_eq!(
            statement.formula(),
            SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(index))
        );
        assert_eq!(statement.site().node().index(), site);
        assert_eq!(statement.source_range(), range(fixture.source, start, end));
        assert_eq!(statement.source_ordinal(), index);
        assert_eq!(statement.spelling(), spelling);
        assert_eq!(statement.kind(), kind);
        assert_eq!(statement.recovery(), SourceStatementRecovery::Normal);
        let context = handoff
            .contexts()
            .get(SourceStatementContextId::new(index))
            .expect("context");
        assert_eq!(context.statement(), SourceStatementId::new(index));
        assert_eq!(
            context.binding_context(),
            BindingContextId::new(binding_context)
        );
        assert_eq!(context.source_range(), statement.source_range());
        assert_eq!(context.visible_bindings(), [BindingId::new(0)]);
        let input = handoff
            .input_facts()
            .get(SourceStatementInputFactId::new(index))
            .expect("input fact");
        assert_eq!(input.statement(), SourceStatementId::new(index));
        assert_eq!(input.context(), SourceStatementContextId::new(index));
        assert_eq!(input.ordinal(), 0);
        assert_eq!(
            input.kind(),
            SourceStatementInputFactKind::ReservedTypeGuard
        );
        assert_eq!(input.binding(), BindingId::new(0));
        assert_eq!(
            input.uses(),
            [
                SourcePrimaryTermReferenceId::new(index * 2),
                SourcePrimaryTermReferenceId::new(index * 2 + 1),
            ]
        );
        let candidate = handoff
            .candidate_facts()
            .get(SourceStatementCandidateFactId::new(index))
            .expect("candidate fact");
        assert_eq!(candidate.statement(), SourceStatementId::new(index));
        assert_eq!(candidate.context(), SourceStatementContextId::new(index));
        assert_eq!(candidate.ordinal(), 0);
        assert_eq!(
            candidate.kind(),
            SourceStatementCandidateFactKind::UnverifiedProposition
        );
        assert_eq!(candidate.formula(), statement.formula());
    }
    let debug = handoff.debug_text();
    assert!(debug.contains(
        "statement#1 ordinal=1 owner=0 context=1 formula=atomic:1 kind=assumption range=80..93 site=41 recovery=normal spelling=\"assume x = x ;\""
    ));
    assert_eq!(handoff.clone().debug_text(), debug);
    assert_eq!(
        fixture
            .build(fixture.input())
            .expect("Task258B2 replay")
            .debug_text(),
        debug
    );
}

#[test]
fn task258b2_dependency_rows_containment_and_replay_fail_closed() {
    let fixture = B2Fixture::new(31);
    let pristine_input = fixture.input();
    let pristine_symbols = fixture.symbols.clone();
    let pristine_bindings = fixture.bindings.clone();
    let pristine_primary = fixture.primary.clone();
    let pristine_atomic = fixture.atomic.clone();
    let pristine_arena = fixture.arena.clone();
    let valid = SourceStatementProducer::build(
        pristine_input.clone(),
        &pristine_symbols,
        &pristine_bindings,
        &pristine_primary,
        &pristine_atomic,
        &pristine_arena,
    )
    .expect("Task258B2 baseline");
    let baseline = valid.debug_text();

    let same_source_b1 = B1Fixture::new(31);
    assert_eq!(same_source_b1.source, fixture.source);
    assert_eq!(same_source_b1.module, fixture.module);
    assert_eq!(same_source_b1.bindings.source_id(), fixture.source);
    assert_eq!(same_source_b1.bindings.module_id(), &fixture.module);
    assert_eq!(same_source_b1.primary.source_id(), fixture.source);
    assert_eq!(same_source_b1.atomic.source_id(), fixture.source);
    for case in 0..11 {
        let mut input = pristine_input.clone();
        let mut bindings = pristine_bindings.clone();
        let mut primary = pristine_primary.clone();
        let mut atomic = pristine_atomic.clone();
        let mut arena = pristine_arena.clone();
        let expected = match case {
            0 => {
                input.candidate_facts.pop();
                SourceStatementError::InvalidAggregate
            }
            1 => {
                input.statements[1].kind = SourceStatementKind::Conclusion;
                SourceStatementError::InvalidStatement {
                    statement: SourceStatementId::new(1),
                }
            }
            2 => {
                input.contexts[1].binding_context = BindingContextId::new(0);
                SourceStatementError::InvalidContext {
                    context: SourceStatementContextId::new(1),
                }
            }
            3 => {
                input.input_facts[1].uses.swap(0, 1);
                SourceStatementError::InvalidInputFact {
                    fact: SourceStatementInputFactId::new(1),
                }
            }
            4 => {
                input.candidate_facts[1].formula =
                    SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(2));
                SourceStatementError::InvalidCandidateFact {
                    fact: SourceStatementCandidateFactId::new(1),
                }
            }
            5 => {
                input.statements[1].formula =
                    SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(2));
                SourceStatementError::InvalidStatement {
                    statement: SourceStatementId::new(1),
                }
            }
            6 => {
                input.owners[0].source_range.end -= 1;
                SourceStatementError::InvalidOwner {
                    owner: SourceTheoremOwnerId::new(0),
                }
            }
            7 => {
                arena = mutate_arena(&pristine_arena, |id, row| {
                    if id == TypedNodeId::new(41) {
                        row.recovery = NodeRecoveryState::Recovered;
                    }
                });
                SourceStatementError::InvalidStatement {
                    statement: SourceStatementId::new(1),
                }
            }
            8 => {
                bindings = same_source_b1.bindings.clone();
                SourceStatementError::DependencyMismatch
            }
            9 => {
                primary = same_source_b1.primary.clone();
                SourceStatementError::DependencyMismatch
            }
            10 => {
                atomic = same_source_b1.atomic.clone();
                SourceStatementError::DependencyMismatch
            }
            _ => unreachable!(),
        };
        let attempted_bindings = bindings.clone();
        let attempted_primary = primary.clone();
        let attempted_atomic = atomic.clone();
        let attempted_arena = arena.clone();
        let actual = SourceStatementProducer::build(
            input,
            &pristine_symbols,
            &bindings,
            &primary,
            &atomic,
            &arena,
        )
        .expect_err("Task258B2 mutation must fail at its owning producer");
        assert_eq!(actual, expected, "Task258B2 mutation case {case}");

        assert_eq!(fixture.input(), pristine_input);
        assert_eq!(fixture.symbols, pristine_symbols);
        assert_eq!(fixture.bindings, pristine_bindings);
        assert_eq!(fixture.primary, pristine_primary);
        assert_eq!(fixture.atomic, pristine_atomic);
        assert_eq!(fixture.arena, pristine_arena);
        assert_eq!(bindings, attempted_bindings);
        assert_eq!(primary, attempted_primary);
        assert_eq!(atomic, attempted_atomic);
        assert_eq!(arena, attempted_arena);
        let replay = SourceStatementProducer::build(
            pristine_input.clone(),
            &pristine_symbols,
            &pristine_bindings,
            &pristine_primary,
            &pristine_atomic,
            &pristine_arena,
        )
        .expect("Task258B2 canonical replay after mutation");
        assert_eq!(
            replay.debug_text(),
            baseline,
            "Task258B2 replay after mutation case {case}"
        );
    }

    let pristine_label = pristine_symbols
        .labels()
        .iter()
        .next()
        .expect("Task258B2 exact theorem label")
        .clone();
    for (case, name) in [
        "Imported",
        "Missing",
        "Duplicate",
        "WrongPath",
        "WrongKind",
        "Private",
        "LocalOnly",
        "Recovered",
    ]
    .into_iter()
    .enumerate()
    {
        let mut labels = LabelIndex::new();
        if case != 1 {
            let origin_path = if case == 3 {
                LabelOriginPath::new("task258b2::wrong::theorem::origin")
            } else {
                pristine_label.origin_path().clone()
            };
            let kind = if case == 4 {
                mizar_resolve::resolved_ast::LabelKind::ProofStep
            } else {
                pristine_label.kind()
            };
            let origin = match case {
                0 => {
                    let (import_owner, _) =
                        same_source_b1.resolver_ast.nodes().iter().next().unwrap();
                    assert_eq!(import_owner.index(), 0);
                    let source_range = range(pristine_label.origin().source_id(), 0, 1);
                    let mut import_metadata = ResolvedImports::new();
                    let import = import_metadata.push_import(ResolvedImport::new(
                        import_owner,
                        source_range,
                        "import statement.fixture;",
                        None,
                        ImportResolution::Resolved(pristine_symbols.module_id().clone()),
                        pristine_label.origin().clone(),
                    ));
                    pristine_label.origin().clone().with_import_edge(import)
                }
                7 => pristine_label.origin().clone().recovered(),
                _ => pristine_label.origin().clone(),
            };
            let visibility = if case == 5 {
                mizar_resolve::env::Visibility::Private
            } else {
                pristine_label.visibility()
            };
            let export_status = if case == 6 {
                mizar_resolve::env::ExportStatus::LocalOnly
            } else {
                pristine_label.export_status()
            };
            labels.insert(
                LabelEntry::new(
                    origin_path,
                    kind,
                    pristine_label.namespace().clone(),
                    pristine_label.primary_spelling(),
                    origin,
                    pristine_label.contribution(),
                )
                .with_visibility(visibility)
                .with_export_status(export_status),
            );
            if case == 2 {
                labels.insert(
                    LabelEntry::new(
                        LabelOriginPath::new("task258b2::duplicate::theorem::origin"),
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
        let attempted_symbols = SymbolEnv::new(
            pristine_symbols.module_id().clone(),
            SymbolEnvIndexes {
                imports: pristine_symbols.imports().clone(),
                exports: pristine_symbols.exports().clone(),
                symbols: pristine_symbols.symbols().clone(),
                labels,
                definitions: pristine_symbols.definitions().clone(),
                overloads: pristine_symbols.overloads().clone(),
                registrations: pristine_symbols.registrations().clone(),
                lexical_summaries: pristine_symbols.lexical_summaries().clone(),
                namespace_graph: pristine_symbols.namespace_graph().clone(),
                declaration_dependencies: pristine_symbols.declaration_dependencies().clone(),
                contributions: pristine_symbols.contributions().clone(),
                module_summaries: pristine_symbols.module_summaries().clone(),
            },
        );
        assert_eq!(
            attempted_symbols.imports(),
            pristine_symbols.imports(),
            "{name} imports"
        );
        let attempted_symbols_before = attempted_symbols.clone();
        let actual = SourceStatementProducer::build(
            pristine_input.clone(),
            &attempted_symbols,
            &pristine_bindings,
            &pristine_primary,
            &pristine_atomic,
            &pristine_arena,
        )
        .expect_err("Task258B2 Resolver-label mutation must fail at its owning producer");
        assert_eq!(
            actual,
            SourceStatementError::InvalidOwner {
                owner: SourceTheoremOwnerId::new(0),
            },
            "Task258B2 Resolver-label mutation {name}"
        );

        assert_eq!(attempted_symbols, attempted_symbols_before, "{name} purity");
        assert_eq!(fixture.input(), pristine_input, "{name} source input");
        assert_eq!(fixture.symbols, pristine_symbols, "{name} symbols");
        assert_eq!(fixture.bindings, pristine_bindings, "{name} bindings");
        assert_eq!(fixture.primary, pristine_primary, "{name} primary terms");
        assert_eq!(fixture.atomic, pristine_atomic, "{name} atomic formulas");
        assert_eq!(fixture.arena, pristine_arena, "{name} typed arena");
        let replay = SourceStatementProducer::build(
            pristine_input.clone(),
            &pristine_symbols,
            &pristine_bindings,
            &pristine_primary,
            &pristine_atomic,
            &pristine_arena,
        )
        .expect("Task258B2 canonical replay after Resolver-label mutation");
        assert_eq!(
            replay.debug_text(),
            baseline,
            "Task258B2 replay after Resolver-label mutation {name}"
        );
    }

    let escaped_assumption = mutate_arena(&fixture.arena, |id, row| {
        if id == TypedNodeId::new(41) {
            row.children.retain(|child| *child != TypedNodeId::new(40));
        }
    });
    assert_eq!(
        valid.validate_installation(
            fixture.source,
            &fixture.module,
            &fixture.primary,
            &fixture.atomic,
            &escaped_assumption,
        ),
        Err(SourceStatementError::InvalidStatement {
            statement: SourceStatementId::new(1)
        })
    );
    let cross_subtree = mutate_arena(&fixture.arena, |id, row| {
        if id == TypedNodeId::new(41) {
            row.children.push(TypedNodeId::new(46));
        }
    });
    assert_eq!(
        valid.validate_installation(
            fixture.source,
            &fixture.module,
            &fixture.primary,
            &fixture.atomic,
            &cross_subtree,
        ),
        Err(SourceStatementError::InvalidStatement {
            statement: SourceStatementId::new(0)
        })
    );
    let recovered_assumption = mutate_arena(&fixture.arena, |id, row| {
        if id == TypedNodeId::new(41) {
            row.recovery = NodeRecoveryState::Recovered;
        }
    });
    assert_eq!(
        valid.validate_installation(
            fixture.source,
            &fixture.module,
            &fixture.primary,
            &fixture.atomic,
            &recovered_assumption,
        ),
        Err(SourceStatementError::InvalidStatement {
            statement: SourceStatementId::new(1)
        })
    );

    let mut stale_origin = valid.clone();
    stale_origin.checked_owner = CheckedStatementOwner::from_validated_parts_for_test(
        fixture.symbol.clone(),
        range(fixture.source, 19, 112),
        SemanticOrigin::new(
            fixture.source,
            fixture.module.clone(),
            SourceAnchor::Range(range(fixture.source, 19, 112)),
            vec![1],
        ),
    );
    assert_eq!(
        stale_origin.validate_installation(
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
    let mut cross_profile = valid.clone();
    cross_profile.binding_env = b1_binding_env(fixture.source, &fixture.module);
    cross_profile.binding_fingerprint = cross_profile.binding_env.debug_text();
    assert_eq!(
        cross_profile.validate_installation(
            fixture.source,
            &fixture.module,
            &fixture.primary,
            &fixture.atomic,
            &fixture.arena,
        ),
        Err(SourceStatementError::DependencyMismatch)
    );
    let foreign = B2Fixture::new(32);
    assert_eq!(
        valid.validate_installation(
            fixture.source,
            &fixture.module,
            &foreign.primary,
            &fixture.atomic,
            &fixture.arena,
        ),
        Err(SourceStatementError::DependencyMismatch)
    );
    assert_eq!(
        valid.validate_installation(
            fixture.source,
            &fixture.module,
            &fixture.primary,
            &foreign.atomic,
            &fixture.arena,
        ),
        Err(SourceStatementError::DependencyMismatch)
    );
    let mut stale_atomic_fingerprint = valid.clone();
    stale_atomic_fingerprint
        .atomic_formula_fingerprint
        .push('x');
    assert_eq!(
        stale_atomic_fingerprint.validate_installation(
            fixture.source,
            &fixture.module,
            &fixture.primary,
            &fixture.atomic,
            &fixture.arena,
        ),
        Err(SourceStatementError::DependencyMismatch)
    );
    assert_eq!(
        fixture
            .build(fixture.input())
            .expect("Task258B2 replay")
            .debug_text(),
        baseline
    );
}

#[test]
fn task258b2_base_only_typed_ownership_and_cross_profiles_are_atomic() {
    let fixture = B2Fixture::new(33);
    let handoff = fixture.build(fixture.input()).expect("Task258B2 handoff");
    let base = fixture.empty_typed();
    let base_debug = base.debug_text();
    let installed = base
        .clone()
        .with_source_statement(handoff.clone())
        .expect("Task258B2 base-only install");
    assert_eq!(installed.source_statement(), Some(&handoff));
    assert!(installed.source_statement_references().is_none());
    let installed_debug = installed.debug_text();
    assert_eq!(
        installed.clone().with_source_statement(handoff.clone()),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(installed.debug_text(), installed_debug);

    let b1 = B1Fixture::new(33);
    let references = b1
        .references(b1.reference_input())
        .expect("Task258B1 references");
    assert_eq!(
        base.clone()
            .with_source_statement_references(handoff.clone(), references),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(base.debug_text(), base_debug);
    assert_eq!(
        base.clone().with_source_statement(b1.statement.clone()),
        Err(TypedAstError::InvalidSourceStatement)
    );
    let task258a = Fixture::new(33);
    assert_eq!(
        base.clone().with_source_statement(
            task258a
                .build(task258a.input())
                .expect("Task258A statement")
        ),
        Err(TypedAstError::InvalidSourceStatement)
    );

    let task248 = crate::source_context::tests::task_248_occupied_typed_ast(
        fixture.source,
        fixture.module.clone(),
    );
    let task248_debug = task248.debug_text();
    assert_eq!(
        task248.clone().with_source_statement(handoff.clone()),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(task248.debug_text(), task248_debug);

    let task257a = crate::source_composite_formula::tests::task_257a_installed_typed_ast();
    let task257b = crate::source_formula_composition::tests::task_257b_installed_typed_ast();
    let task257c2 = crate::source_formula_composition::tests::task_257c2_installed_typed_ast();
    let task257c3 = crate::source_formula_composition::tests::task_257c3_installed_typed_ast();
    for (family, occupied) in [
        ("Task257A", &task257a),
        ("Task257B", &task257b),
        ("Task257C2", &task257c2),
        ("Task257C3", &task257c3),
    ] {
        let debug = occupied.debug_text();
        assert_eq!(
            occupied.clone().with_source_statement(handoff.clone()),
            Err(TypedAstError::InvalidSourceStatement),
            "{family} first"
        );
        assert_eq!(occupied.debug_text(), debug, "{family} rollback");
    }
    assert_eq!(
        installed.clone().with_source_composite_formula(
            task257a
                .source_composite_formula()
                .expect("Task257A composite")
                .clone(),
        ),
        Err(TypedAstError::InvalidSourceCompositeFormula)
    );
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

    let mut semantic_facts = TypeFactTable::new();
    semantic_facts.insert(TypeFactDraft {
        subject: node(28),
        predicate: TypePredicateRef::new("set"),
        polarity: Polarity::Positive,
        provenance: FactProvenance::Inferred(TypeRuleId::new("task258b2-coexistence")),
        status: FactStatus::Known,
    });
    let semantic = TypedAst::try_new(TypedAstParts {
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
    .expect("Task258B2 semantic base")
    .with_source_term(fixture.primary.clone())
    .expect("Task252")
    .with_source_atomic_formula(fixture.atomic.clone())
    .expect("Task256");
    let semantic_debug = semantic.debug_text();
    assert_eq!(
        semantic.clone().with_source_statement(handoff.clone()),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(semantic.debug_text(), semantic_debug);
    assert_eq!(
        base.with_source_statement(handoff)
            .expect("Task258B2 replay")
            .debug_text(),
        installed_debug
    );
}

#[test]
fn task258b2_final_clone_revalidation_and_empty_semantics_are_stable() {
    let fixture = B2Fixture::new(34);
    let handoff = fixture.build(fixture.input()).expect("Task258B2 handoff");
    let typed = fixture
        .empty_typed()
        .with_source_statement(handoff.clone())
        .expect("Task258B2 typed AST");
    let resolved = assemble_empty_resolved(&typed).expect("Task258B2 final assembly");
    assert_eq!(resolved.source_statement(), Some(&handoff));
    assert!(resolved.source_statement_references().is_none());
    assert_eq!(resolved.source_term(), typed.source_term());
    assert_eq!(
        resolved.source_atomic_formula(),
        typed.source_atomic_formula()
    );
    assert_eq!(resolved.clone().debug_text(), resolved.debug_text());
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

    let b1 = B1Fixture::new(34);
    let references = b1
        .references(b1.reference_input())
        .expect("Task258B1 references");
    let mut orphan_reference = typed.clone();
    orphan_reference.inject_source_statement_references_for_test(references);
    assert_eq!(
        assemble_empty_resolved(&orphan_reference),
        Err(ResolvedTypedAstError::InvalidSourceStatement)
    );

    let mut cluster_facts = ClusterFactTable::new();
    cluster_facts.insert(ClusterFactDraft {
        fingerprint: ClusterFactFingerprint::new("task258b2-coexistence"),
        source_type: ClusterTypeFingerprint::new("set"),
        attribute: ClusterAttributeFingerprint::new("inhabited"),
        generated_type: ClusterTypeFingerprint::new("set"),
        provenance: ClusterFactProvenance::Input,
        source_range: range(fixture.source, 87, 88),
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
                owner: handoff.checked_owner(),
                rows: Vec::new(),
            }),
        ),
        Err(ResolvedTypedAstError::InvalidSourceStatement)
    );
    let replay = assemble_empty_resolved(&typed).expect("Task258B2 final replay");
    assert_eq!(replay.debug_text(), resolved.debug_text());
}
