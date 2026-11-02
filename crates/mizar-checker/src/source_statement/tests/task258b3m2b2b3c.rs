use super::*;

#[test]
fn task258b3m2b2b3c_exact_choice_witness_profile_and_legacy_compatibility_are_stable() {
    const SOURCE: &str = concat!(
        "reserve x for set;\n",
        "theorem FormulaStatementChoiceWitnessSmoke: x = x proof\n",
        "  take the set;\n",
        "  thus x = x;\n",
        "end;\n",
    );
    let fixture = B3M2B2B3CFixture::new(236);
    assert_eq!(SOURCE.len(), 110);
    assert_eq!(
        SOURCE,
        "reserve x for set;\ntheorem FormulaStatementChoiceWitnessSmoke: x = x proof\n  take the set;\n  thus x = x;\nend;\n"
    );
    assert_eq!(fixture.arena.len(), 52);
    assert_eq!(fixture.arena.root(), Some(TypedNodeId::new(51)));
    for (index, (start, end)) in TASK258B3M2B2B3C_NODE_RANGES.iter().copied().enumerate() {
        let row = fixture
            .arena
            .node(TypedNodeId::new(index))
            .expect("exact arena row");
        assert_eq!(
            row.anchor,
            SourceAnchor::Range(range(fixture.source, start, end))
        );
        assert_eq!(row.kind.as_str(), task258b3m2b2b3c_node_kind(index));
        assert_eq!(row.recovery, NodeRecoveryState::Normal);
        assert_eq!(
            row.children
                .iter()
                .map(|child| child.index())
                .collect::<Vec<_>>(),
            task258b3m2b2b3c_node_children(index)
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
        (1, 0, 0, 1, 0, 0, 2)
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
    assert!(statement.is_task_258b3m2b2b3c_profile());
    assert!(!statement.is_task_258b3m2b2b3a_profile());
    let witnesses = fixture.witnesses(&statement).expect("choice witness");
    assert_eq!(witnesses.application_fingerprint(), None);
    assert_eq!(witnesses.structure_fingerprint(), None);
    assert_eq!(
        witnesses.set_term_fingerprint(),
        Some(fixture.set_terms.debug_text().as_str())
    );
    assert_eq!(
        witnesses.debug_text(),
        format!(
            "source-statement-witness-debug-v1\nmodule: pkg::statement.fixture\nstatement-fingerprint: {:?}\nprimary-term-fingerprint: {:?}\nset-term-fingerprint: Some({:?})\nwitness#0 owner=0 binding_context=1 term=set-term#0 take_range=77..90 take_site=38 range=82..89 site=37 source_ordinal=1 ordinal=0 kind=unnamed recovery=normal spelling=\"the set\"\n",
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
        [27, 29, 39, 41]
    );
    assert_eq!(
        fixture
            .set_terms
            .terms()
            .iter()
            .map(|(_, row)| row.site().node().index())
            .collect::<Vec<_>>(),
        [35]
    );
    assert_eq!(
        fixture
            .atomic
            .formulas()
            .iter()
            .map(|(_, row)| row.site().node().index())
            .collect::<Vec<_>>(),
        [31, 43]
    );
    assert_eq!(
        statement
            .statements()
            .iter()
            .map(|(_, row)| row.site().node().index())
            .collect::<Vec<_>>(),
        [48, 46]
    );
    assert_eq!(
        witnesses
            .witnesses()
            .iter()
            .flat_map(|(_, row)| [row.site().node().index(), row.take_site().node().index()])
            .collect::<Vec<_>>(),
        [37, 38]
    );

    let legacy = B3M2B2B3AFixture::new(236);
    let legacy_statement = legacy.statement();
    legacy
        .witnesses(&legacy_statement)
        .expect("nonempty enumeration remains admitted");
    let empty = B3M2B2B3BFixture::new(236);
    let empty_statement = empty.statement();
    empty
        .witnesses(&empty_statement)
        .expect("empty enumeration remains admitted");
}

#[test]
fn task258b3m2b2b3c_dependencies_choice_precedence_and_all_nodes_fail_closed() {
    use crate::binding_env::BindingEnvError as BindingError;
    use crate::source_atomic_formula::SourceAtomicFormulaError as AtomicError;
    use crate::source_term::SourcePrimaryTermError as PrimaryError;
    let fixture = B3M2B2B3CFixture::new(237);
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
    let pristine_dependencies = Box::new((
        fixture.symbols.clone(),
        fixture.arena.clone(),
        fixture.bindings.clone(),
        fixture.primary.clone(),
        fixture.set_terms.clone(),
        fixture.atomic.clone(),
    ));
    let pristine_inputs = (
        fixture.statement_input(),
        fixture.witness_input(),
        B3M2B2B3CFixture::primary_input(fixture.source, &fixture.module),
        B3M2B2B3CFixture::set_input(fixture.source, &fixture.module),
        B3M2B2B3CFixture::atomic_input(fixture.source, &fixture.module),
    );
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
                &pristine_dependencies.0,
                &pristine_dependencies.1,
                &pristine_dependencies.2,
                &pristine_dependencies.3,
                &pristine_dependencies.4,
                &pristine_dependencies.5,
            ),
            "pristine dependency purity"
        );
        assert_eq!(
            (
                fixture.statement_input(),
                fixture.witness_input(),
                B3M2B2B3CFixture::primary_input(fixture.source, &fixture.module),
                B3M2B2B3CFixture::set_input(fixture.source, &fixture.module),
                B3M2B2B3CFixture::atomic_input(fixture.source, &fixture.module),
            ),
            pristine_inputs,
            "pristine input purity"
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
                        row.kind = "source.task258b3m2b2b3c.mutated".into();
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
    for root in [None, Some(TypedNodeId::new(50))] {
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

    for field in 0..32 {
        let expected = match field {
            0 => Some(BindingError::InvalidContextSourceRange {
                context: BindingContextId::new(1),
            }),
            2 => Some(BindingError::MissingModuleContext),
            4..=6 => Some(BindingError::InvalidModuleContext {
                context: BindingContextId::new(0),
            }),
            8 => Some(BindingError::InvalidVisibleBindingOwner {
                context: BindingContextId::new(0),
                binding: BindingId::new(0),
                owner: BindingContextId::new(0),
            }),
            12 => Some(BindingError::MultipleRootContexts {
                context: BindingContextId::new(1),
            }),
            15 => Some(BindingError::InvalidContextBinding {
                context: BindingContextId::new(1),
                binding: BindingId::new(0),
            }),
            18 | 23 => Some(BindingError::InvalidContextBinding {
                context: BindingContextId::new(0),
                binding: BindingId::new(0),
            }),
            19 | 20 | 24 => Some(BindingError::InconsistentReservedVariableIdentity {
                binding: BindingId::new(usize::from(field == 19)),
            }),
            21 | 22 => Some(BindingError::InvalidBindingKindIdentity {
                binding: BindingId::new(0),
            }),
            29 => Some(BindingError::InvalidBindingDiagnostic {
                binding: BindingId::new(0),
                diagnostic: BindingDiagnosticId::new(0),
            }),
            1 | 3 | 7 | 9..=11 | 13 | 14 | 16 | 17 | 25..=28 | 30 | 31 => None,
            _ => unreachable!(),
        };
        let mut source_id = fixture.bindings.source_id();
        let mut module_id = fixture.bindings.module_id().clone();
        let mut contexts = fixture
            .bindings
            .contexts()
            .iter()
            .map(|(_, row)| BindingContextDraft {
                owner: row.owner.clone(),
                parent: row.parent,
                layer: row.layer,
                lexical_scope: row.lexical_scope.clone(),
                bindings: row.bindings.clone(),
                visible_bindings: row.visible_bindings.clone(),
                recovery: row.recovery,
            })
            .collect::<Vec<_>>();
        let mut bindings = fixture
            .bindings
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
        let mut diagnostics = fixture
            .bindings
            .diagnostics()
            .iter()
            .map(|(_, row)| BindingDiagnosticDraft {
                source_range: row.source_range,
                class: row.class,
                severity: row.severity,
                message_key: row.message_key.clone(),
                recovery: row.recovery,
            })
            .collect::<Vec<_>>();

        match field {
            0 => source_id = super::source_id(999_248),
            1 => {
                module_id =
                    ModuleId::new(PackageId::new("pkg"), ModulePath::new("statement.foreign"))
            }
            2 => contexts.clear(),
            3 => contexts.push(BindingContextDraft {
                owner: BindingContextOwner::Generated("choice extra".to_owned()),
                parent: Some(BindingContextId::new(1)),
                layer: BindingContextLayer::Block,
                lexical_scope: Some(LocalTermScope::new(vec![0, 1])),
                bindings: Vec::new(),
                visible_bindings: vec![BindingId::new(0)],
                recovery: BindingContextRecovery::Normal,
            }),
            11 => {
                contexts[1].owner = BindingContextOwner::SourceStatement {
                    source_range: range(fixture.source, 69, 107),
                }
            }
            4..18 => {
                let offset = field - 4;
                let context = offset / 7;
                match offset % 7 {
                    0 => {
                        contexts[context].owner =
                            BindingContextOwner::Generated(format!("mutated-context-{context}"))
                    }
                    1 => {
                        contexts[context].parent = if context == 0 {
                            Some(BindingContextId::new(1))
                        } else {
                            None
                        }
                    }
                    2 => contexts[context].layer = BindingContextLayer::Expression,
                    3 => {
                        contexts[context].lexical_scope = if context == 0 {
                            Some(LocalTermScope::new(vec![9]))
                        } else {
                            None
                        }
                    }
                    4 => {
                        if context == 0 {
                            contexts[context].bindings.clear();
                        } else {
                            contexts[context].bindings.push(BindingId::new(0));
                        }
                    }
                    5 => contexts[context].visible_bindings.clear(),
                    6 => contexts[context].recovery = BindingContextRecovery::Degraded,
                    _ => unreachable!(),
                }
            }
            18 => bindings.clear(),
            19 => {
                let mut extra = bindings[0].clone();
                extra.spelling.push('!');
                bindings.push(extra);
            }
            20 => bindings[0].spelling.push('!'),
            21 => bindings[0].kind = BindingKind::Generated,
            22 => {
                bindings[0].identity = BinderIdentity::Generated {
                    context: BindingContextId::new(0),
                    counter: 99,
                }
            }
            23 => bindings[0].owner_context = BindingContextId::new(1),
            24 => bindings[0].declaration_range.start += 1,
            25 => bindings[0].visible_after_ordinal += 1,
            26 => bindings[0].type_site = BindingTypeSite::Deferred("mutated".to_owned()),
            27 => bindings[0].status = BindingStatus::Degraded,
            28 => {
                bindings[0].captured = CapturedFreeVariables::new(vec![BinderIdentity::Generated {
                    context: BindingContextId::new(0),
                    counter: 99,
                }])
            }
            29 => bindings[0].diagnostics = vec![BindingDiagnosticId::new(0)],
            30 => bindings[0].recovery = BindingRecoveryState::Degraded,
            31 => diagnostics.push(BindingDiagnosticDraft {
                source_range: None,
                class: BindingDiagnosticClass::UnsupportedSourceShape,
                severity: BindingDiagnosticSeverity::Note,
                message_key: "checker.binding.choice.mutated".to_owned(),
                recovery: BindingDiagnosticRecovery::Degraded,
            }),
            _ => unreachable!(),
        }

        let mut context_table = BindingContextTable::new();
        for context in contexts {
            context_table.insert(context);
        }
        let mut binding_table = BindingTable::new();
        for binding in bindings {
            binding_table.insert(binding);
        }
        let mut diagnostic_table = BindingDiagnosticTable::new();
        for diagnostic in diagnostics {
            diagnostic_table.insert(diagnostic);
        }
        let parts = BindingEnvParts {
            source_id,
            module_id,
            contexts: context_table,
            bindings: binding_table,
            diagnostics: diagnostic_table,
        };
        let pristine_parts = BindingEnvParts {
            source_id: fixture.source,
            module_id: fixture.module.clone(),
            contexts: fixture.bindings.contexts().clone(),
            bindings: fixture.bindings.bindings().clone(),
            diagnostics: fixture.bindings.diagnostics().clone(),
        };
        assert_ne!(parts, pristine_parts, "Task48 field {field} input delta");
        let before = parts.clone();
        let actual = BindingEnv::try_new(parts.clone());
        assert_eq!(parts, before, "Task48 field {field} input purity");
        match (expected, actual) {
            (Some(expected), Err(actual)) => {
                assert_eq!(actual, expected, "Task48 field {field}")
            }
            (Some(expected), Ok(_)) => {
                panic!("Task48 field {field} admitted; expected {expected:?}")
            }
            (None, Ok(bindings)) => {
                assert_ne!(
                    bindings, fixture.bindings,
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
        let pristine_input = B3M2B2B3CFixture::primary_input(fixture.source, &fixture.module);
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
            0 | 1 => Some(PrimaryError::InvalidTransaction),
            2 => Some(PrimaryError::InvalidReference {
                reference: SourcePrimaryTermReferenceId::new(0),
            }),
            3 => Some(PrimaryError::InvalidTerm {
                term: SourcePrimaryTermId::new(4),
            }),
            4..40 => {
                let offset = field - 4;
                (offset % 9 != 3).then_some(PrimaryError::InvalidTerm {
                    term: SourcePrimaryTermId::new(offset / 9),
                })
            }
            40 => Some(PrimaryError::InvalidTerm {
                term: SourcePrimaryTermId::new(0),
            }),
            41 => Some(PrimaryError::InvalidReference {
                reference: SourcePrimaryTermReferenceId::new(4),
            }),
            42..54 => {
                let offset = field - 42;
                let reference = offset / 3;
                Some(PrimaryError::InvalidReference {
                    reference: SourcePrimaryTermReferenceId::new(if offset % 3 == 0 {
                        (reference + 1).min(3)
                    } else {
                        reference
                    }),
                })
            }
            54 => Some(PrimaryError::InvalidNumericTypeRequest {
                request: crate::source_term::SourceNumericTypeRequestId::new(0),
            }),
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
                let primary_before = primary.clone();
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
                assert_eq!(primary, primary_before, "Task252 admitted primary purity");
            }
            (None, Err(actual)) => {
                panic!("Task252 field {field} refused as {actual:?}; expected admitted primary")
            }
        }
        assert_eq!(input, attempted_input, "Task252 field {field} input purity");
        replay_pristine();
    }

    for term in 0..4 {
        let mut primary = fixture.primary.clone();
        primary.set_reference_use_ordinal_for_test(SourcePrimaryTermReferenceId::new(term), 999);
        assert_ne!(
            primary, fixture.primary,
            "Task252 reference {term} clock delta"
        );
        let before = primary.clone();
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
        assert_eq!(primary, before, "Task252 clock purity");
        replay_pristine();
    }
    type Task255Mutation = (&'static str, fn(&mut SourceSetTermHandoffInput));
    let task255_mutations: [Task255Mutation; 39] = [
        ("envelope source", |input| {
            input.source_id = source_id(999_255)
        }),
        ("envelope module", |input| {
            input.module_id =
                ModuleId::new(PackageId::new("pkg"), ModulePath::new("statement.foreign"))
        }),
        ("term missing", |input| input.terms.clear()),
        ("term duplicate", |input| {
            input.terms.push(input.terms[0].clone())
        }),
        ("term site", |input| input.terms[0].site = node(34)),
        ("term range", |input| input.terms[0].source_range.end -= 1),
        ("term source ordinal", |input| {
            input.terms[0].source_ordinal = 1
        }),
        ("term context", |input| {
            input.terms[0].context = BindingContextId::new(0)
        }),
        ("term recovery", |input| {
            input.terms[0].recovery = SourceSetTermRecovery::Degraded
        }),
        ("term spelling", |input| input.terms[0].spelling.push('!')),
        ("term kind", |input| {
            input.terms[0].kind = SourceSetTermKind::Qua
        }),
        ("wrapper present", |input| {
            input.wrappers.push(SourceSetWrapperInput {
                term: SourceSetTermId::new(0),
                ordinal: 0,
                site: node(36),
                source_range: input.terms[0].source_range,
                context: BindingContextId::new(1),
                recovery: SourceSetTermRecovery::Normal,
                spelling: "the set".to_owned(),
            })
        }),
        ("generator present", |input| {
            input.generators.push(SourceSetGeneratorInput {
                term: SourceSetTermId::new(0),
                ordinal: 0,
                site: node(34),
                source_range: input.type_sites[0].source_range,
                spelling: "set".to_owned(),
                context: BindingContextId::new(1),
                recovery: SourceSetTermRecovery::Normal,
                type_site: SourceSetTypeSiteId::new(0),
            })
        }),
        ("type site missing", |input| input.type_sites.clear()),
        ("type site duplicate", |input| {
            input.type_sites.push(input.type_sites[0].clone())
        }),
        ("type owner", |input| {
            input.type_sites[0].owner = SourceSetTypeOwner::Term {
                term: SourceSetTermId::new(0),
                role: SourceSetTypeRole::QuaTarget,
            }
        }),
        ("type site", |input| input.type_sites[0].site = node(33)),
        ("type range", |input| {
            input.type_sites[0].source_range.end -= 1
        }),
        ("type spelling", |input| {
            input.type_sites[0].spelling.push('!')
        }),
        ("type head site", |input| {
            input.type_sites[0].head_site = node(34)
        }),
        ("type head range", |input| {
            input.type_sites[0].head_range.end -= 1
        }),
        ("type head spelling", |input| {
            input.type_sites[0].head_spelling.push('!')
        }),
        ("type context", |input| {
            input.type_sites[0].context = BindingContextId::new(0)
        }),
        ("type recovery", |input| {
            input.type_sites[0].recovery = SourceSetTermRecovery::Degraded
        }),
        ("type head", |input| {
            input.type_sites[0].head = SourceSetTypeHead::BuiltinObject
        }),
        ("condition present", |input| {
            input.conditions.push(SourceSetConditionInput {
                term: SourceSetTermId::new(0),
                ordinal: 0,
                colon_site: node(15),
                colon_range: input.terms[0].source_range,
                colon_spelling: ":".to_owned(),
                condition_site: node(31),
                source_range: input.terms[0].source_range,
                spelling: "x = x".to_owned(),
                recovery: SourceSetTermRecovery::Normal,
            })
        }),
        ("edge present", |input| {
            input.edges.push(SourceSetEdgeInput {
                term: SourceSetTermId::new(0),
                ordinal: 0,
                role: SourceSetEdgeRole::EnumerationElement,
                target: SourceSetTarget::Primary(SourcePrimaryTermId::new(0)),
            })
        }),
        ("request missing", |input| {
            input.requests.pop();
        }),
        ("request duplicate", |input| {
            input.requests.push(input.requests[1].clone())
        }),
        ("request0 term", |input| {
            input.requests[0].term = SourceSetTermId::new(1)
        }),
        ("request0 ordinal", |input| input.requests[0].ordinal = 1),
        ("request0 kind", |input| {
            input.requests[0].kind = SourceSetRequestKind::ResultType
        }),
        ("request0 generator", |input| {
            input.requests[0].generator = Some(crate::source_set_term::SourceSetGeneratorId::new(0))
        }),
        ("request0 type", |input| input.requests[0].type_site = None),
        ("request1 term", |input| {
            input.requests[1].term = SourceSetTermId::new(1)
        }),
        ("request1 ordinal", |input| input.requests[1].ordinal = 0),
        ("request1 kind", |input| {
            input.requests[1].kind = SourceSetRequestKind::ChoiceNonempty
        }),
        ("request1 generator", |input| {
            input.requests[1].generator = Some(crate::source_set_term::SourceSetGeneratorId::new(0))
        }),
        ("request1 type", |input| {
            input.requests[1].type_site = Some(SourceSetTypeSiteId::new(0))
        }),
    ];
    assert_eq!(task255_mutations.len(), 39);
    for (field, (label, mutate)) in task255_mutations.into_iter().enumerate() {
        let pristine_input = B3M2B2B3CFixture::set_input(fixture.source, &fixture.module);
        let mut input = pristine_input.clone();
        mutate(&mut input);
        assert_ne!(input, pristine_input, "Task255 {label} input delta");
        use crate::source_set_term::SourceSetTermError as SetError;
        let expected = match field {
            0..=2 => SetError::EnvironmentMismatch,
            3 => SetError::InvalidTerm {
                term: SourceSetTermId::new(1),
            },
            4..=6 | 8..=10 | 12 | 13 | 25 | 26 => SetError::InvalidTerm {
                term: SourceSetTermId::new(0),
            },
            7 | 15..=24 => SetError::InvalidTypeSite {
                type_site: SourceSetTypeSiteId::new(0),
            },
            11 => SetError::InvalidWrapper {
                wrapper: crate::source_set_term::SourceSetWrapperId::new(0),
            },
            14 => SetError::InvalidTypeSite {
                type_site: SourceSetTypeSiteId::new(1),
            },
            27 => SetError::InvalidRequest {
                request: crate::source_set_term::SourceSetRequestId::new(0),
            },
            28 => SetError::ReorderedRequest {
                request: crate::source_set_term::SourceSetRequestId::new(2),
            },
            29..39 => {
                let request = (field - 29) / 5;
                if (field - 29) % 5 <= 1 {
                    SetError::ReorderedRequest {
                        request: crate::source_set_term::SourceSetRequestId::new(request),
                    }
                } else {
                    SetError::InvalidRequest {
                        request: crate::source_set_term::SourceSetRequestId::new(request),
                    }
                }
            }
            _ => unreachable!(),
        };
        let before = input.clone();
        assert_eq!(
            SourceSetTermProducer::build(
                input.clone(),
                &fixture.bindings,
                &fixture.primary,
                None,
                None,
                &fixture.arena,
            ),
            Err(expected),
            "Task255 {label}"
        );
        assert_eq!(input, before, "Task255 {label} input purity");
        replay_pristine();
    }
    let pristine_atomic_input = B3M2B2B3CFixture::atomic_input(fixture.source, &fixture.module);
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
        match field {
            0 => input.formulas.clear(),
            1 => input.formulas.push(input.formulas[0].clone()),
            2..16 => {
                let offset = field - 2;
                let formula = offset / 7;
                let member = offset % 7;
                let other = 1 - formula;
                match member {
                    0 => input.formulas[formula].site = input.formulas[other].site.clone(),
                    1 => input.formulas[formula].source_range.start += 1,
                    2 => input.formulas[formula].source_ordinal += 2,
                    3 => {
                        input.formulas[formula].context =
                            BindingContextId::new(1 - input.formulas[formula].context.index())
                    }
                    4 => input.formulas[formula].recovery = SourceAtomicFormulaRecovery::Degraded,
                    5 => input.formulas[formula].spelling.push('!'),
                    6 => input.formulas[formula].kind = SourceAtomicFormulaKind::Inequality,
                    _ => unreachable!(),
                }
            }
            16..32 => {
                let offset = field - 16;
                let edge = offset / 4;
                match offset % 4 {
                    0 => {
                        input.edges[edge].formula =
                            SourceAtomicFormulaId::new(1 - input.edges[edge].formula.index())
                    }
                    1 => input.edges[edge].ordinal += 2,
                    2 => input.edges[edge].role = SourceAtomicEdgeRole::PredicateLeftArgument,
                    3 => {
                        input.edges[edge].target =
                            SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(99))
                    }
                    _ => unreachable!(),
                }
            }
            32..60 => {
                let offset = field - 32;
                let request = offset / 7;
                match offset % 7 {
                    0 => {
                        input.requests[request].formula =
                            SourceAtomicFormulaId::new(1 - input.requests[request].formula.index())
                    }
                    1 => input.requests[request].ordinal += 2,
                    2 => {
                        input.requests[request].kind =
                            SourceAtomicRequestKind::PredicateCandidateSignature
                    }
                    3 => input.requests[request].edge = None,
                    4 => {
                        input.requests[request].candidate =
                            Some(crate::source_atomic_formula::SourcePredicateCandidateId::new(0))
                    }
                    5 => {
                        input.requests[request].type_site =
                            Some(crate::source_atomic_formula::SourceAssertionTypeSiteId::new(0))
                    }
                    6 => {
                        input.requests[request].attribute =
                            Some(crate::source_atomic_formula::SourceAssertionAttributeId::new(0))
                    }
                    _ => unreachable!(),
                }
            }
            60 => {
                input.module_id =
                    ModuleId::new(PackageId::new("pkg"), ModulePath::new("statement.foreign"))
            }
            61 => input.source_id = source_id(999_238),
            62 => input.wrappers.push(SourceAtomicWrapperInput {
                formula: SourceAtomicFormulaId::new(0),
                ordinal: 0,
                site: input.formulas[0].site.clone(),
                source_range: input.formulas[0].source_range,
                context: input.formulas[0].context,
                recovery: SourceAtomicFormulaRecovery::Normal,
                spelling: "( x = x )".to_owned(),
            }),
            63 => input.predicate_segments.push(SourcePredicateSegmentInput {
                formula: SourceAtomicFormulaId::new(0),
                ordinal: 0,
                site: input.formulas[0].site.clone(),
                source_range: input.formulas[0].source_range,
                context: input.formulas[0].context,
                recovery: SourceAtomicFormulaRecovery::Normal,
                spelling: "x = x".to_owned(),
                head: SourcePredicateHeadId::new(0),
                polarity: SourcePredicateSegmentPolarityInput::Positive,
                left_edge: SourceAtomicEdgeId::new(0),
                right_edge: SourceAtomicEdgeId::new(1),
            }),
            64 => input.predicate_heads.push(SourcePredicateHeadInput {
                formula: SourceAtomicFormulaId::new(0),
                site: input.formulas[0].site.clone(),
                source_range: input.formulas[0].source_range,
                context: input.formulas[0].context,
                recovery: SourceAtomicFormulaRecovery::Normal,
                spelling: "=".to_owned(),
                left_arity: 1,
                right_arity: 1,
            }),
            65 => input.candidates.push(SourcePredicateCandidateInput {
                head: SourcePredicateHeadId::new(0),
                ordinal: 0,
                symbol: fixture.symbol.clone(),
                contribution: fixture.contribution,
            }),
            66 => input.type_sites.push(SourceAssertionTypeSiteInput {
                formula: SourceAtomicFormulaId::new(0),
                site: input.formulas[0].site.clone(),
                source_range: input.formulas[0].source_range,
                spelling: "set".to_owned(),
                head_site: input.formulas[0].site.clone(),
                head_range: input.formulas[0].source_range,
                head_spelling: "set".to_owned(),
                context: input.formulas[0].context,
                recovery: SourceAtomicFormulaRecovery::Normal,
                head: SourceAssertionTypeHead::BuiltinSet,
            }),
            67 => input.attributes.push(SourceAssertionAttributeInput {
                formula: SourceAtomicFormulaId::new(0),
                ordinal: 0,
                site: input.formulas[0].site.clone(),
                source_range: input.formulas[0].source_range,
                spelling: "attr".to_owned(),
                target_site: input.formulas[0].site.clone(),
                target_range: input.formulas[0].source_range,
                target_spelling: "attr".to_owned(),
                context: input.formulas[0].context,
                recovery: SourceAtomicFormulaRecovery::Normal,
                symbol: fixture.symbol.clone(),
                contribution: fixture.contribution,
                polarity: SourceAssertionAttributePolarityInput::Positive,
            }),
            68 => input.edges.clear(),
            69 => {
                let extra = input.edges[0].clone();
                input.edges.push(extra);
            }
            70 => input.requests.clear(),
            71 => {
                let extra = input.requests[0].clone();
                input.requests.push(extra);
            }
            _ => unreachable!(),
        }
        assert_ne!(
            input, pristine_atomic_input,
            "Task256 field {field} input delta"
        );
        let expected = match field {
            0 | 60 | 61 => AtomicError::EnvironmentMismatch,
            1 => AtomicError::InvalidFormula {
                formula: SourceAtomicFormulaId::new(2),
            },
            2..16 => {
                let offset = field - 2;
                let formula = offset / 7;
                if offset % 7 == 3 {
                    AtomicError::PrimaryDependencyMismatch
                } else {
                    AtomicError::InvalidFormula {
                        formula: SourceAtomicFormulaId::new(formula),
                    }
                }
            }
            16..32 => {
                let offset = field - 16;
                let edge = offset / 4;
                match offset % 4 {
                    0 | 1 => AtomicError::ReorderedEdge {
                        edge: SourceAtomicEdgeId::new(if offset % 4 == 0 {
                            edge.max(1)
                        } else {
                            edge
                        }),
                    },
                    2 => AtomicError::InvalidFormula {
                        formula: SourceAtomicFormulaId::new(edge / 2),
                    },
                    3 => AtomicError::InvalidEdge {
                        edge: SourceAtomicEdgeId::new(edge),
                    },
                    _ => unreachable!(),
                }
            }
            32..60 => {
                let offset = field - 32;
                let request = offset / 7;
                match offset % 7 {
                    0 | 1 => AtomicError::ReorderedRequest {
                        request: SourceAtomicRequestId::new(if offset % 7 == 0 {
                            request.max(1)
                        } else {
                            request
                        }),
                    },
                    2..=6 => AtomicError::InvalidRequest {
                        request: SourceAtomicRequestId::new(request),
                    },
                    _ => unreachable!(),
                }
            }
            62 => AtomicError::InvalidWrapper {
                wrapper: crate::source_atomic_formula::SourceAtomicWrapperId::new(0),
            },
            63 | 67 | 68 => AtomicError::InvalidFormula {
                formula: SourceAtomicFormulaId::new(0),
            },
            64 => AtomicError::ReorderedPredicateHead {
                head: SourcePredicateHeadId::new(0),
            },
            65 => AtomicError::ReorderedCandidate {
                candidate: SourcePredicateCandidateId::new(0),
            },
            66 => AtomicError::InvalidTypeSite {
                type_site: crate::source_atomic_formula::SourceAssertionTypeSiteId::new(0),
            },
            69 => AtomicError::ReorderedEdge {
                edge: SourceAtomicEdgeId::new(4),
            },
            70 => AtomicError::InvalidRequest {
                request: SourceAtomicRequestId::new(0),
            },
            71 => AtomicError::ReorderedRequest {
                request: SourceAtomicRequestId::new(4),
            },
            _ => unreachable!(),
        };
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
        assert_eq!(actual, Err(expected), "Task256 field {field}");
        assert_eq!(input, attempted_input, "Task256 field {field} input purity");
        replay_pristine();
    }

    for field in 0..62 {
        let pristine_input = fixture.statement_input();
        let mut input = pristine_input.clone();
        match field {
            0 => input.owners.clear(),
            1 => input.owners.push(input.owners[0].clone()),
            2 => input.owners[0].site = input.statements[1].site.clone(),
            3 => input.owners[0].source_range.end -= 1,
            4 => input.owners[0].spelling.push('!'),
            5 => input.owners[0].recovery = SourceStatementRecovery::Degraded,
            6 => input.statements.clear(),
            7 => input.statements.push(input.statements[0].clone()),
            8..24 => {
                let offset = field - 8;
                let statement = offset / 8;
                let other = 1 - statement;
                match offset % 8 {
                    0 => input.statements[statement].owner = SourceTheoremOwnerId::new(1),
                    1 => input.statements[statement].context = SourceStatementContextId::new(other),
                    2 => {
                        input.statements[statement].formula =
                            SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(other))
                    }
                    3 => input.statements[statement].site = input.statements[other].site.clone(),
                    4 => input.statements[statement].source_range.start += 1,
                    5 => {
                        if statement == 1 {
                            input.statements[statement].source_ordinal = 1;
                        } else {
                            input.statements[statement].source_ordinal += 3;
                        }
                    }
                    6 => input.statements[statement].spelling.push('!'),
                    7 => input.statements[statement].recovery = SourceStatementRecovery::Degraded,
                    _ => unreachable!(),
                }
            }
            24 => input.contexts.clear(),
            25 => input.contexts.push(input.contexts[0].clone()),
            26..34 => {
                let offset = field - 26;
                let context = offset / 4;
                let other = 1 - context;
                match offset % 4 {
                    0 => input.contexts[context].statement = SourceStatementId::new(other),
                    1 => input.contexts[context].binding_context = BindingContextId::new(other),
                    2 => {
                        if context == 1 {
                            input.contexts[context].source_range.end -= 1;
                        } else {
                            input.contexts[context].source_range.start += 1;
                        }
                    }
                    3 => input.contexts[context].visible_bindings.clear(),
                    _ => unreachable!(),
                }
            }
            34 => input.input_facts.clear(),
            35 => input.input_facts.push(input.input_facts[0].clone()),
            36..46 => {
                let offset = field - 36;
                let fact = offset / 5;
                let other = 1 - fact;
                match offset % 5 {
                    0 => input.input_facts[fact].statement = SourceStatementId::new(other),
                    1 => input.input_facts[fact].context = SourceStatementContextId::new(other),
                    2 => input.input_facts[fact].ordinal += 1,
                    3 => input.input_facts[fact].binding = BindingId::new(99),
                    4 => input.input_facts[fact].uses.reverse(),
                    _ => unreachable!(),
                }
            }
            46 => input.candidate_facts.clear(),
            47 => input.candidate_facts.push(input.candidate_facts[0].clone()),
            48..56 => {
                let offset = field - 48;
                let candidate = offset / 4;
                let other = 1 - candidate;
                match offset % 4 {
                    0 => input.candidate_facts[candidate].statement = SourceStatementId::new(other),
                    1 => {
                        input.candidate_facts[candidate].context =
                            SourceStatementContextId::new(other)
                    }
                    2 => input.candidate_facts[candidate].ordinal += 1,
                    3 => {
                        input.candidate_facts[candidate].formula =
                            SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(other))
                    }
                    _ => unreachable!(),
                }
            }
            56 => {
                input.module_id =
                    ModuleId::new(PackageId::new("pkg"), ModulePath::new("statement.foreign"))
            }
            57 => input.source_id = source_id(999_238),
            58 => {
                input.owners[0].symbol = SymbolId::new(
                    input.module_id.clone(),
                    mizar_resolve::resolved_ast::LocalSymbolId::new(
                        "Task258B3M2B2B3A/mutated-owner",
                    ),
                    mizar_resolve::resolved_ast::FullyQualifiedName::new(format!(
                        "{}::Task258B3M2B2B3A/mutated-owner",
                        input.module_id.path().as_str()
                    )),
                )
            }
            59 => {
                let mut contributions = mizar_resolve::env::SourceContributionIndex::new();
                let kind = mizar_resolve::env::ContributionKind::LocalSource {
                    source_id: input.source_id,
                };
                let anchor = SourceAnchor::Range(input.owners[0].source_range);
                contributions.insert(input.module_id.clone(), kind.clone(), anchor.clone());
                input.owners[0].contribution =
                    contributions.insert(input.module_id.clone(), kind, anchor);
            }
            60 => input.statements[0].kind = SourceStatementKind::Conclusion,
            61 => input.statements[1].kind = SourceStatementKind::TheoremProposition,
            _ => unreachable!(),
        }
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
        match field {
            0 => input.witnesses.clear(),
            1 => input.witnesses.push(input.witnesses[0].clone()),
            2 => input.witnesses[0].owner = SourceTheoremOwnerId::new(1),
            3 => input.witnesses[0].binding_context = BindingContextId::new(0),
            4 => {
                input.witnesses[0].term =
                    SourceStatementWitnessTermTarget::Primary(SourcePrimaryTermId::new(0))
            }
            5 => input.witnesses[0].take_site = node(35),
            6 => input.witnesses[0].take_range.end -= 1,
            7 => input.witnesses[0].site = node(33),
            8 => input.witnesses[0].source_range.end -= 1,
            9 => input.witnesses[0].source_ordinal = 0,
            10 => input.witnesses[0].ordinal = 1,
            11 => input.witnesses[0].spelling.push('!'),
            12 => input.witnesses[0].recovery = SourceStatementRecovery::Degraded,
            13 => input.witnesses[0].name = Some(SourceStatementWitnessNameId::new(0)),
            14 => input.names.push(SourceStatementWitnessNameInput {
                witness: crate::source_statement::SourceStatementWitnessId::new(0),
                site: input.witnesses[0].site.clone(),
                source_range: input.witnesses[0].source_range,
                spelling: "y".to_owned(),
                recovery: SourceStatementRecovery::Normal,
            }),
            15 => {
                input.module_id =
                    ModuleId::new(PackageId::new("pkg"), ModulePath::new("statement.foreign"))
            }
            16 => input.source_id = source_id(999_238),
            17 => input.witnesses[0].kind = SourceStatementWitnessKind::Named,
            18 => {
                input.witnesses[0].term =
                    SourceStatementWitnessTermTarget::SetTerm(SourceSetTermId::new(1))
            }
            19 => {
                input.witnesses[0].term = SourceStatementWitnessTermTarget::Application(
                    SourceFunctorApplicationId::new(0),
                )
            }
            20 => {
                input.witnesses[0].term =
                    SourceStatementWitnessTermTarget::Structure(SourceStructureTermId::new(0))
            }
            _ => unreachable!(),
        }
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

    let mut dependency_before_aggregate = fixture.witness_input();
    dependency_before_aggregate.source_id = source_id(999_238);
    dependency_before_aggregate.witnesses.clear();
    assert_ne!(dependency_before_aggregate, fixture.witness_input());
    let dependency_before = dependency_before_aggregate.clone();
    assert_eq!(
        SourceStatementWitnessProducer::build_with_set_term(
            dependency_before_aggregate.clone(),
            &statement,
            &fixture.primary,
            &fixture.set_terms,
            &fixture.arena,
        ),
        Err(SourceStatementWitnessError::DependencyMismatch)
    );
    assert_eq!(
        dependency_before_aggregate, dependency_before,
        "mixed fault input purity"
    );
    replay_pristine();
}

#[test]
fn task258b3m2b2b3c_combined_ownership_hybrids_and_all_family_orders_are_atomic() {
    let fixture = B3M2B2B3CFixture::new(238);
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

    let pristine_dependencies = Box::new((
        fixture.symbols.clone(),
        fixture.bindings.clone(),
        fixture.primary.clone(),
        fixture.set_terms.clone(),
        fixture.atomic.clone(),
        fixture.arena.clone(),
    ));
    let pristine_witness_input = fixture.witness_input();
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
                &pristine_dependencies.0,
                &pristine_dependencies.1,
                &pristine_dependencies.2,
                &pristine_dependencies.3,
                &pristine_dependencies.4,
                &pristine_dependencies.5
            ),
            "owner dependency purity"
        );
        assert_eq!(
            fixture.statement_input(),
            pristine_input,
            "owner statement input purity"
        );
        assert_eq!(
            fixture.witness_input(),
            pristine_witness_input,
            "owner witness input purity"
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
    let pristine_label = fixture.symbols.labels().iter().next().expect("B3C label");
    let pristine_contribution = fixture
        .symbols
        .contributions()
        .get(fixture.contribution)
        .expect("B3C contribution");
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
                    14 => *structural_path.last_mut().expect("B3C owner path") = 9,
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
            let admitted_witnesses = fixture
                .witnesses(&admitted)
                .expect("extra-effects own witness");
            assert_eq!(admitted_witnesses, witnesses, "{name} own witness rows");
            let admitted_typed = fixture
                .empty_typed()
                .with_source_set_term_statement_witnesses(
                    fixture.set_terms.clone(),
                    admitted,
                    admitted_witnesses,
                )
                .expect("extra-effects own typed install");
            assert_eq!(admitted_typed, typed, "{name} own typed replay");
            assert_eq!(
                assemble_empty_resolved(&admitted_typed).expect("extra-effects own final"),
                *pristine_resolved,
                "{name} own final replay",
            );
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
        "B3C base then B3A"
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
        "B3A base then B3C"
    );
    assert_eq!(
        typed.clone().with_source_set_term_statement_witnesses(
            legacy.set_terms,
            legacy_statement,
            legacy_witnesses,
        ),
        Err(TypedAstError::InvalidSourceStatement),
        "installed B3C then B3A"
    );
    assert_eq!(
        legacy_typed.with_source_set_term_statement_witnesses(
            fixture.set_terms.clone(),
            statement.clone(),
            witnesses.clone(),
        ),
        Err(TypedAstError::InvalidSourceStatement),
        "installed B3A then B3C"
    );
    let empty = B3M2B2B3BFixture::new(238);
    let empty_statement = empty.statement();
    let empty_witnesses = empty
        .witnesses(&empty_statement)
        .expect("empty-enumeration witnesses");
    let empty_typed = empty
        .empty_typed()
        .with_source_set_term_statement_witnesses(
            empty.set_terms.clone(),
            empty_statement.clone(),
            empty_witnesses.clone(),
        )
        .expect("empty enumeration installed");
    assert_eq!(
        base.clone().with_source_set_term_statement_witnesses(
            empty.set_terms.clone(),
            empty_statement.clone(),
            empty_witnesses.clone(),
        ),
        Err(TypedAstError::InvalidSourceStatement),
        "B3C base then B3B"
    );
    assert_eq!(
        empty
            .empty_typed()
            .with_source_set_term_statement_witnesses(
                fixture.set_terms.clone(),
                statement.clone(),
                witnesses.clone(),
            ),
        Err(TypedAstError::InvalidSourceStatement),
        "B3B base then B3C"
    );
    assert_eq!(
        typed.clone().with_source_set_term_statement_witnesses(
            empty.set_terms,
            empty_statement,
            empty_witnesses,
        ),
        Err(TypedAstError::InvalidSourceStatement),
        "installed B3C then B3B"
    );
    assert_eq!(
        empty_typed.with_source_set_term_statement_witnesses(
            fixture.set_terms.clone(),
            statement.clone(),
            witnesses.clone(),
        ),
        Err(TypedAstError::InvalidSourceStatement),
        "installed B3B then B3C"
    );

    macro_rules! reject_plain_witness_family_for_b3c {
        ($label:literal, $family:expr) => {{
            run_in_isolated_test_stack_frame(|| {
                let family = $family;
                let family_statement = family.statement();
                let family_witnesses = family
                    .witnesses(&family_statement, family.witness_input())
                    .expect(concat!($label, " witnesses"));
                let family_base = family.empty_typed();
                let family_installed = family_base
                    .clone()
                    .with_source_statement_witnesses(
                        family_statement.clone(),
                        family_witnesses.clone(),
                    )
                    .expect(concat!($label, " installed"));
                assert_eq!(
                    base.clone().with_source_statement_witnesses(
                        family_statement.clone(),
                        family_witnesses.clone(),
                    ),
                    Err(TypedAstError::InvalidSourceStatement),
                    concat!("B3C base then ", $label)
                );
                assert_eq!(
                    typed.clone().with_source_statement_witnesses(
                        family_statement.clone(),
                        family_witnesses.clone(),
                    ),
                    Err(TypedAstError::InvalidSourceStatement),
                    concat!("installed B3C then ", $label)
                );
                assert_eq!(
                    family_base
                        .clone()
                        .with_source_set_term_statement_witnesses(
                            fixture.set_terms.clone(),
                            statement.clone(),
                            witnesses.clone(),
                        ),
                    Err(TypedAstError::InvalidSourceStatement),
                    concat!($label, " base then B3C")
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
                    concat!("installed ", $label, " then B3C")
                );
                assert_eq!(
                    family_base.with_source_set_term(fixture.set_terms.clone()),
                    Err(TypedAstError::InvalidSourceSetTerm),
                    concat!($label, " base rejects B3C set injection")
                );
                assert_eq!(
                    family_installed.with_source_set_term(fixture.set_terms.clone()),
                    Err(TypedAstError::InvalidSourceSetTerm),
                    concat!("installed ", $label, " rejects B3C set injection")
                );
            });
        }};
    }
    reject_plain_witness_family_for_b3c!("B3", B3Fixture::new(238));
    reject_plain_witness_family_for_b3c!("B3N", B3NFixture::new(238));
    reject_plain_witness_family_for_b3c!("B3M1", B3M1Fixture::new(238));
    reject_plain_witness_family_for_b3c!("B3M2A", B3M2AFixture::new(238));
    reject_plain_witness_family_for_b3c!("B3M2B1", B3M2B1Fixture::new(238));
    reject_plain_witness_family_for_b3c!("B3M2B2A", B3M2B2AFixture::new(238));

    macro_rules! reject_application_witness_family_for_b3c {
        ($label:literal, $family:expr) => {{
            run_in_isolated_test_stack_frame(|| {
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
                    concat!("B3C base then ", $label)
                );
                assert_eq!(
                    typed.clone().with_source_application_statement_witnesses(
                        family.application.clone(),
                        family_statement.clone(),
                        family_witnesses.clone(),
                    ),
                    Err(TypedAstError::InvalidSourceStatement),
                    concat!("installed B3C then ", $label)
                );
                assert_eq!(
                    family_base
                        .clone()
                        .with_source_set_term_statement_witnesses(
                            fixture.set_terms.clone(),
                            statement.clone(),
                            witnesses.clone(),
                        ),
                    Err(TypedAstError::InvalidSourceStatement),
                    concat!($label, " base then B3C")
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
                    concat!("installed ", $label, " then B3C")
                );
                assert_eq!(
                    family_base.with_source_set_term(fixture.set_terms.clone()),
                    Err(TypedAstError::InvalidSourceSetTerm),
                    concat!($label, " base rejects B3C set injection")
                );
                assert_eq!(
                    family_installed.with_source_set_term(fixture.set_terms.clone()),
                    Err(TypedAstError::InvalidSourceSetTerm),
                    concat!("installed ", $label, " rejects B3C set injection")
                );
            });
        }};
    }
    reject_application_witness_family_for_b3c!("B1A", B3M2B2B1AFixture::new(238));
    reject_application_witness_family_for_b3c!("B1B1", B3M2B2B1B1Fixture::new(238));

    macro_rules! reject_structure_witness_family_for_b3c {
        ($label:literal, $family:expr) => {{
            run_in_isolated_test_stack_frame(|| {
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
                    concat!("B3C base then ", $label)
                );
                assert_eq!(
                    typed.clone().with_source_structure_statement_witnesses(
                        family.structure.clone(),
                        family_statement.clone(),
                        family_witnesses.clone(),
                    ),
                    Err(TypedAstError::InvalidSourceStatement),
                    concat!("installed B3C then ", $label)
                );
                assert_eq!(
                    family_base
                        .clone()
                        .with_source_set_term_statement_witnesses(
                            fixture.set_terms.clone(),
                            statement.clone(),
                            witnesses.clone(),
                        ),
                    Err(TypedAstError::InvalidSourceStatement),
                    concat!($label, " base then B3C")
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
                    concat!("installed ", $label, " then B3C")
                );
                assert_eq!(
                    family_base.with_source_set_term(fixture.set_terms.clone()),
                    Err(TypedAstError::InvalidSourceSetTerm),
                    concat!($label, " base rejects B3C set injection")
                );
                assert_eq!(
                    family_installed.with_source_set_term(fixture.set_terms.clone()),
                    Err(TypedAstError::InvalidSourceSetTerm),
                    concat!("installed ", $label, " rejects B3C set injection")
                );
            });
        }};
    }
    reject_structure_witness_family_for_b3c!("B2A", B3M2B2B2AFixture::new(238));
    reject_structure_witness_family_for_b3c!("B2B", B3M2B2B2BFixture::new(238));
    reject_structure_witness_family_for_b3c!("B2C", B3M2B2B2CFixture::new(238));

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
fn task258b3m2b2b3c_final_clone_revalidation_and_semantic_deferrals_are_stable() {
    let fixture = B3M2B2B3CFixture::new(239);
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
    let foreign = B3M2B2B3CFixture::new(240);
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
        fingerprint: ClusterFactFingerprint::new("task258b3m2b2b3c-coexistence"),
        source_type: ClusterTypeFingerprint::new("set"),
        attribute: ClusterAttributeFingerprint::new("inhabited"),
        generated_type: ClusterTypeFingerprint::new("set"),
        provenance: ClusterFactProvenance::Input,
        source_range: range(fixture.source, 82, 89),
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
                expr: ExprId::new("task258b3m2b2b3c-semantic-coexistence"),
                typed_site: node(35),
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
