use super::*;

#[test]
fn task258b3m1_exact_multi_witness_api_debug_and_compatibility_are_stable() {
    let fixture = B3M1Fixture::new(51);
    let statement = fixture.statement();
    assert!(statement.is_task_258b3m1_profile());
    assert!(!statement.is_task_258b3_profile());
    assert!(!statement.is_task_258b3n_profile());
    assert_eq!(
        (
            fixture.bindings.contexts().len(),
            fixture.bindings.bindings().len(),
            fixture.bindings.diagnostics().len(),
            fixture.primary.terms().len(),
            fixture.primary.references().len(),
            fixture.primary.numeric_type_requests().len(),
            fixture.atomic.formulas().len(),
            fixture.atomic.edges().len(),
            fixture.atomic.requests().len(),
        ),
        (2, 1, 0, 6, 6, 0, 2, 4, 4)
    );
    let proof = fixture
        .bindings
        .contexts()
        .get(BindingContextId::new(1))
        .expect("proof context");
    assert_eq!(
        (
            &proof.owner,
            proof.parent,
            proof.layer,
            proof
                .lexical_scope
                .as_ref()
                .map(|scope| scope.path().to_vec()),
            proof.bindings.as_slice(),
            proof.visible_bindings.as_slice(),
            proof.recovery,
        ),
        (
            &BindingContextOwner::SourceStatement {
                source_range: range(fixture.source, 71, 111),
            },
            Some(BindingContextId::new(0)),
            BindingContextLayer::Proof,
            Some(vec![0]),
            &[][..],
            &[BindingId::new(0)][..],
            BindingContextRecovery::Normal,
        )
    );
    for (index, ((_, term), (_, reference))) in fixture
        .primary
        .terms()
        .iter()
        .zip(fixture.primary.references().iter())
        .enumerate()
    {
        assert_eq!(
            (
                term.site().node().index(),
                term.source_range(),
                term.source_ordinal(),
                term.context().index(),
                term.spelling(),
                term.kind(),
                term.role(),
                term.recovery(),
                term.parent(),
            ),
            (
                [30, 32, 36, 39, 43, 45][index],
                range(
                    fixture.source,
                    [65, 69, 88, 91, 101, 105][index],
                    [66, 70, 89, 92, 102, 106][index],
                ),
                index,
                [0, 0, 1, 1, 1, 1][index],
                "x",
                SourcePrimaryTermKind::VariableReference,
                SourcePrimaryTermRole::Value,
                SourcePrimaryTermRecovery::Normal,
                None,
            )
        );
        assert_eq!(
            (
                reference.term().index(),
                reference.binding().index(),
                reference.role(),
                reference.use_ordinal(),
                reference.lexical_scope().map(|scope| scope.path().to_vec()),
            ),
            (
                index,
                0,
                SourcePrimaryTermReferenceRole::Variable,
                1,
                if index < 2 { None } else { Some(vec![0]) },
            )
        );
    }
    assert_eq!(
        fixture
            .atomic
            .edges()
            .iter()
            .map(|(_, edge)| match edge.target() {
                SourceAtomicTermTarget::Primary(term) => term.index(),
                _ => usize::MAX,
            })
            .collect::<Vec<_>>(),
        [0, 1, 4, 5]
    );
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
    let owner = statement
        .owners()
        .get(SourceTheoremOwnerId::new(0))
        .expect("owner");
    assert_eq!(
        (
            owner.symbol(),
            owner.contribution(),
            owner.site().node().index(),
            owner.source_range(),
            owner.spelling(),
            owner.role(),
            owner.status(),
            owner.recovery(),
        ),
        (
            &fixture.symbol,
            fixture.contribution,
            52,
            range(fixture.source, 19, 112),
            "FormulaStatementMultipleWitnessSmoke",
            SourceTheoremRole::Theorem,
            SourceTheoremStatus::Unmodified,
            SourceStatementRecovery::Normal,
        )
    );
    assert_eq!(statement.checked_owner().origin().structural_path(), [2, 1]);
    let labels = fixture
        .symbols
        .labels()
        .by_contribution(fixture.contribution);
    assert_eq!(labels.len(), 1);
    assert_eq!(
        (
            labels[0].primary_spelling(),
            labels[0].origin().structural_path(),
            labels[0].contribution(),
        ),
        (
            "FormulaStatementMultipleWitnessSmoke",
            &[2, 1][..],
            fixture.contribution,
        )
    );
    for index in 0..2 {
        let row = statement
            .statements()
            .get(SourceStatementId::new(index))
            .expect("statement");
        assert_eq!(
            (
                row.site().node().index(),
                row.source_range(),
                row.source_ordinal(),
                row.spelling(),
                row.kind(),
            ),
            (
                [52, 50][index],
                range(fixture.source, [19, 96][index], [112, 107][index]),
                [0, 2][index],
                [
                    "theorem FormulaStatementMultipleWitnessSmoke : x = x proof take y = x , x ; thus x = x ; end ;",
                    "thus x = x ;",
                ][index],
                [
                    SourceStatementKind::TheoremProposition,
                    SourceStatementKind::Conclusion,
                ][index],
            )
        );
        assert_eq!(
            statement
                .input_facts()
                .get(SourceStatementInputFactId::new(index))
                .expect("fact")
                .uses()
                .iter()
                .map(|id| id.index())
                .collect::<Vec<_>>(),
            if index == 0 { vec![0, 1] } else { vec![4, 5] },
        );
    }
    let handoff = fixture
        .witnesses(&statement, fixture.witness_input())
        .expect("witness handoff");
    assert_eq!(handoff.source_id(), fixture.source);
    assert_eq!(handoff.module_id(), &fixture.module);
    assert_eq!(handoff.statement_fingerprint(), statement.debug_text());
    assert_eq!(
        handoff.primary_term_fingerprint(),
        fixture.primary.debug_text()
    );
    assert_eq!((handoff.witnesses().len(), handoff.names().len()), (2, 1));
    for index in 0..2 {
        let row = handoff
            .witnesses()
            .get(SourceStatementWitnessId::new(index))
            .expect("witness");
        assert_eq!(
            (
                row.owner().index(),
                row.binding_context().index(),
                row.term(),
                row.take_site().node().index(),
                row.take_range(),
                row.site().node().index(),
                row.source_range(),
                row.source_ordinal(),
            ),
            (
                0,
                1,
                SourceStatementWitnessTermTarget::Primary(SourcePrimaryTermId::new(2 + index)),
                42,
                range(fixture.source, 79, 93),
                [38, 41][index],
                range(fixture.source, [84, 91][index], [89, 92][index]),
                1,
            )
        );
        assert_eq!(
            (
                row.ordinal(),
                row.spelling(),
                row.kind(),
                row.recovery(),
                row.name(),
            ),
            (
                index,
                ["y = x", "x"][index],
                [
                    SourceStatementWitnessKind::Named,
                    SourceStatementWitnessKind::Unnamed,
                ][index],
                SourceStatementRecovery::Normal,
                [Some(SourceStatementWitnessNameId::new(0)), None][index],
            )
        );
    }
    let name = handoff
        .names()
        .get(SourceStatementWitnessNameId::new(0))
        .expect("name");
    assert_eq!(
        (
            name.witness().index(),
            name.site().node().index(),
            name.source_range(),
            name.spelling(),
            name.recovery(),
        ),
        (
            0,
            13,
            range(fixture.source, 84, 85),
            "y",
            SourceStatementRecovery::Normal,
        )
    );
    assert_eq!(
        handoff.debug_text(),
        format!(
            "source-statement-witness-debug-v1\nmodule: pkg::statement.fixture\nstatement-fingerprint: {:?}\nprimary-term-fingerprint: {:?}\nwitness#0 owner=0 binding_context=1 term=primary#2 take_range=79..93 take_site=42 range=84..89 site=38 source_ordinal=1 ordinal=0 kind=named recovery=normal spelling=\"y = x\" name=0\nwitness#1 owner=0 binding_context=1 term=primary#3 take_range=79..93 take_site=42 range=91..92 site=41 source_ordinal=1 ordinal=1 kind=unnamed recovery=normal spelling=\"x\"\nwitness-name#0 witness=0 range=84..85 site=13 recovery=normal spelling=\"y\"\n",
            statement.debug_text(),
            fixture.primary.debug_text(),
        )
    );
    for (index, (start, end)) in TASK258B3M1_NODE_RANGES.iter().copied().enumerate() {
        let node = fixture
            .arena
            .node(TypedNodeId::new(index))
            .expect("arena node");
        assert_eq!(
            (
                &node.anchor,
                node.kind.as_str(),
                node.recovery,
                node.children
                    .iter()
                    .map(|child| child.index())
                    .collect::<Vec<_>>(),
            ),
            (
                &SourceAnchor::Range(range(fixture.source, start, end)),
                task258b3m1_node_kind(index),
                NodeRecoveryState::Normal,
                task258b3m1_node_children(index).to_vec(),
            ),
            "node {index}"
        );
    }
    let b3 = B3Fixture::new(52);
    let b3_statement = b3.statement();
    let b3_debug = b3
        .witnesses(&b3_statement, b3.witness_input())
        .expect("B3")
        .debug_text();
    assert_eq!(
        b3.witnesses(&b3_statement, b3.witness_input())
            .expect("B3 replay")
            .debug_text(),
        b3_debug
    );
    let b3n = B3NFixture::new(53);
    let b3n_statement = b3n.statement();
    let b3n_debug = b3n
        .witnesses(&b3n_statement, b3n.witness_input())
        .expect("B3N")
        .debug_text();
    assert_eq!(
        b3n.witnesses(&b3n_statement, b3n.witness_input())
            .expect("B3N replay")
            .debug_text(),
        b3n_debug
    );
}

#[test]
fn task258b3m1_dependencies_rows_precedence_and_all_nodes_fail_closed() {
    let fixture = B3M1Fixture::new(54);
    let pristine_input = fixture.statement_input();
    let pristine_symbols = fixture.symbols.clone();
    let pristine_bindings = fixture.bindings.clone();
    let pristine_primary = fixture.primary.clone();
    let pristine_atomic = fixture.atomic.clone();
    let pristine_arena = fixture.arena.clone();
    let statement = SourceStatementProducer::build(
        pristine_input.clone(),
        &pristine_symbols,
        &pristine_bindings,
        &pristine_primary,
        &pristine_atomic,
        &pristine_arena,
    )
    .expect("pristine B3M1 statement");
    let baseline = fixture
        .witnesses(&statement, fixture.witness_input())
        .expect("baseline");
    let baseline_debug = baseline.debug_text();
    let pristine_base = fixture
        .empty_typed()
        .with_source_statement_witnesses(statement.clone(), baseline.clone())
        .expect("pristine paired install");
    let pristine_typed = pristine_base
        .clone()
        .with_source_proof_local_declaration(
            fixture
                .proof_local(&statement, &baseline, fixture.proof_local_input())
                .expect("pristine historical upper"),
        )
        .expect("pristine upper install");
    let pristine_final = assemble_empty_resolved(&pristine_typed).expect("pristine final");
    let pristine_typed_debug = pristine_typed.debug_text();
    let pristine_final_debug = pristine_final.debug_text();

    let assert_pristine = || {
        assert_eq!(fixture.symbols, pristine_symbols, "pristine symbols");
        assert_eq!(fixture.bindings, pristine_bindings, "pristine bindings");
        assert_eq!(fixture.primary, pristine_primary, "pristine primary");
        assert_eq!(fixture.atomic, pristine_atomic, "pristine atomic");
        assert_eq!(fixture.arena, pristine_arena, "pristine arena");
        let replay_statement = SourceStatementProducer::build(
            pristine_input.clone(),
            &pristine_symbols,
            &pristine_bindings,
            &pristine_primary,
            &pristine_atomic,
            &pristine_arena,
        )
        .expect("pristine statement replay");
        assert_eq!(replay_statement, statement, "statement replay");
        let replay_witness = fixture
            .witnesses(&statement, fixture.witness_input())
            .expect("pristine witness replay");
        assert_eq!(replay_witness, baseline, "witness replay");
        assert_eq!(replay_witness.debug_text(), baseline_debug);
        let replay_base = fixture
            .empty_typed()
            .with_source_statement_witnesses(statement.clone(), baseline.clone())
            .expect("pristine pair replay");
        let replay_typed = replay_base
            .with_source_proof_local_declaration(
                fixture
                    .proof_local(&statement, &baseline, fixture.proof_local_input())
                    .expect("pristine upper replay"),
            )
            .expect("pristine typed replay");
        assert_eq!(replay_typed.debug_text(), pristine_typed_debug);
        assert_eq!(
            assemble_empty_resolved(&replay_typed)
                .expect("pristine final replay")
                .debug_text(),
            pristine_final_debug
        );
    };
    let assert_base_refusal = |case: &str,
                               input: SourceStatementHandoffInput,
                               symbols: &SymbolEnv,
                               expected: SourceStatementError| {
        let input_before = input.clone();
        let symbols_before = symbols.clone();
        let result = SourceStatementProducer::build(
            input.clone(),
            symbols,
            &pristine_bindings,
            &pristine_primary,
            &pristine_atomic,
            &pristine_arena,
        );
        assert_eq!(result, Err(expected), "{case}");
        assert_eq!(input, input_before, "{case} input purity");
        assert_eq!(symbols, &symbols_before, "{case} symbol purity");
        assert_pristine();
    };

    for mutation in 0..10 {
        let mut input = pristine_input.clone();
        let append = mutation % 2 == 1;
        match mutation / 2 {
            0 => {
                if append {
                    input.owners.push(input.owners[0].clone());
                } else {
                    input.owners.pop();
                }
            }
            1 => {
                if append {
                    input.statements.push(input.statements[1].clone());
                } else {
                    input.statements.pop();
                }
            }
            2 => {
                if append {
                    input.contexts.push(input.contexts[1].clone());
                } else {
                    input.contexts.pop();
                }
            }
            3 => {
                if append {
                    input.input_facts.push(input.input_facts[1].clone());
                } else {
                    input.input_facts.pop();
                }
            }
            4 => {
                if append {
                    input.candidate_facts.push(input.candidate_facts[1].clone());
                } else {
                    input.candidate_facts.pop();
                }
            }
            _ => unreachable!(),
        }
        assert_base_refusal(
            &format!("aggregate {mutation}"),
            input,
            &pristine_symbols,
            SourceStatementError::InvalidAggregate,
        );
    }

    let mut wrong_source = pristine_input.clone();
    wrong_source.source_id = B3M1Fixture::new(55).source;
    assert_base_refusal(
        "transaction source",
        wrong_source,
        &pristine_symbols,
        SourceStatementError::DependencyMismatch,
    );
    let mut wrong_module = pristine_input.clone();
    wrong_module.module_id = ModuleId::new(
        PackageId::new("foreign"),
        ModulePath::new("statement.fixture"),
    );
    assert_base_refusal(
        "transaction module",
        wrong_module,
        &pristine_symbols,
        SourceStatementError::DependencyMismatch,
    );

    let foreign_owner = B3NFixture::new(55);
    let mut wrong_owner_symbol = pristine_input.clone();
    wrong_owner_symbol.owners[0].symbol = foreign_owner.symbol.clone();
    assert_base_refusal(
        "owner symbol",
        wrong_owner_symbol,
        &pristine_symbols,
        SourceStatementError::InvalidOwner {
            owner: SourceTheoremOwnerId::new(0),
        },
    );
    let mut wrong_owner_contribution = pristine_input.clone();
    let mut contributions = pristine_symbols.contributions().clone();
    let foreign_contribution = contributions.insert(
        fixture.module.clone(),
        ContributionKind::LocalSource {
            source_id: fixture.source,
        },
        SourceAnchor::Range(range(fixture.source, 0, 18)),
    );
    let symbols_with_foreign_contribution = SymbolEnv::new(
        pristine_symbols.module_id().clone(),
        SymbolEnvIndexes {
            imports: pristine_symbols.imports().clone(),
            exports: pristine_symbols.exports().clone(),
            symbols: pristine_symbols.symbols().clone(),
            labels: pristine_symbols.labels().clone(),
            definitions: pristine_symbols.definitions().clone(),
            overloads: pristine_symbols.overloads().clone(),
            registrations: pristine_symbols.registrations().clone(),
            lexical_summaries: pristine_symbols.lexical_summaries().clone(),
            namespace_graph: pristine_symbols.namespace_graph().clone(),
            declaration_dependencies: pristine_symbols.declaration_dependencies().clone(),
            contributions,
            module_summaries: pristine_symbols.module_summaries().clone(),
        },
    );
    wrong_owner_contribution.owners[0].contribution = foreign_contribution;
    assert_base_refusal(
        "owner contribution",
        wrong_owner_contribution,
        &symbols_with_foreign_contribution,
        SourceStatementError::InvalidOwner {
            owner: SourceTheoremOwnerId::new(0),
        },
    );
    for mutation in 0..5 {
        let mut input = pristine_input.clone();
        let owner = &mut input.owners[0];
        match mutation {
            0 => owner.site = node(53),
            1 => owner.source_range.start += 1,
            2 => owner.source_range.end -= 1,
            3 => owner.spelling.push('x'),
            4 => owner.recovery = SourceStatementRecovery::Degraded,
            _ => unreachable!(),
        }
        assert_base_refusal(
            &format!("owner row {mutation}"),
            input,
            &pristine_symbols,
            SourceStatementError::InvalidOwner {
                owner: SourceTheoremOwnerId::new(0),
            },
        );
    }

    for index in 0..2 {
        for mutation in 0..10 {
            let mut input = pristine_input.clone();
            let row = &mut input.statements[index];
            match mutation {
                0 => row.owner = SourceTheoremOwnerId::new(1),
                1 => row.context = SourceStatementContextId::new(1 - index),
                2 => {
                    row.formula =
                        SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(1 - index))
                }
                3 => row.site = node(if index == 0 { 50 } else { 52 }),
                4 => row.source_range.start += 1,
                5 => row.source_range.end -= 1,
                6 => row.source_ordinal = 1,
                7 => row.spelling.push('x'),
                8 => row.recovery = SourceStatementRecovery::Degraded,
                9 => {
                    row.kind = if index == 0 {
                        SourceStatementKind::Conclusion
                    } else {
                        SourceStatementKind::TheoremProposition
                    }
                }
                _ => unreachable!(),
            }
            assert_base_refusal(
                &format!("statement {index} field {mutation}"),
                input,
                &pristine_symbols,
                SourceStatementError::InvalidStatement {
                    statement: SourceStatementId::new(index),
                },
            );
        }
    }
    let mut conclusion_ordinal_three = pristine_input.clone();
    conclusion_ordinal_three.statements[1].source_ordinal = 3;
    assert_base_refusal(
        "exact lower conclusion ordinal 3",
        conclusion_ordinal_three,
        &pristine_symbols,
        SourceStatementError::InvalidStatement {
            statement: SourceStatementId::new(1),
        },
    );

    for index in 0..2 {
        for mutation in 0..5 {
            let mut input = pristine_input.clone();
            let row = &mut input.contexts[index];
            match mutation {
                0 => row.statement = SourceStatementId::new(1 - index),
                1 => row.binding_context = BindingContextId::new(1 - index),
                2 => row.source_range.start += 1,
                3 => row.source_range.end -= 1,
                4 => row.visible_bindings.clear(),
                _ => unreachable!(),
            }
            assert_base_refusal(
                &format!("context {index} field {mutation}"),
                input,
                &pristine_symbols,
                SourceStatementError::InvalidContext {
                    context: SourceStatementContextId::new(index),
                },
            );
        }
    }

    for index in 0..2 {
        for mutation in 0..8 {
            let mut input = pristine_input.clone();
            let row = &mut input.input_facts[index];
            match mutation {
                0 => row.statement = SourceStatementId::new(1 - index),
                1 => row.context = SourceStatementContextId::new(1 - index),
                2 => row.ordinal = 1,
                3 => row.binding = BindingId::new(1),
                4 => row.uses.swap(0, 1),
                5 => row.uses.clear(),
                6 => {
                    let duplicate = row.uses[0];
                    row.uses.push(duplicate);
                }
                7 => row.uses.push(SourcePrimaryTermReferenceId::new(99)),
                _ => unreachable!(),
            }
            assert_base_refusal(
                &format!("input fact {index} field {mutation}"),
                input,
                &pristine_symbols,
                SourceStatementError::InvalidInputFact {
                    fact: SourceStatementInputFactId::new(index),
                },
            );
        }
    }

    for index in 0..2 {
        for mutation in 0..4 {
            let mut input = pristine_input.clone();
            let row = &mut input.candidate_facts[index];
            match mutation {
                0 => row.statement = SourceStatementId::new(1 - index),
                1 => row.context = SourceStatementContextId::new(1 - index),
                2 => row.ordinal = 1,
                3 => {
                    row.formula =
                        SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(1 - index))
                }
                _ => unreachable!(),
            }
            assert_base_refusal(
                &format!("candidate fact {index} field {mutation}"),
                input,
                &pristine_symbols,
                SourceStatementError::InvalidCandidateFact {
                    fact: SourceStatementCandidateFactId::new(index),
                },
            );
        }
    }

    let empty_primary = SourcePrimaryTermProducer::build(
        SourcePrimaryTermHandoffInput {
            source_id: fixture.source,
            module_id: fixture.module.clone(),
            terms: Vec::new(),
            references: Vec::new(),
            numeric_type_requests: Vec::new(),
        },
        &pristine_bindings,
        &pristine_arena,
    )
    .expect("valid empty primary lower handoff");
    let empty_primary_before = empty_primary.clone();
    assert_eq!(
        SourceStatementProducer::build(
            pristine_input.clone(),
            &pristine_symbols,
            &pristine_bindings,
            &empty_primary,
            &pristine_atomic,
            &pristine_arena,
        ),
        Err(SourceStatementError::DependencyMismatch),
        "historical empty-primary dependency refusal"
    );
    assert_eq!(empty_primary, empty_primary_before);
    assert_pristine();

    let actual_labels = pristine_symbols
        .labels()
        .by_contribution(fixture.contribution);
    assert_eq!(actual_labels.len(), 1);
    let actual_label = actual_labels[0].clone();
    let make_symbols_with_labels = |labels: LabelIndex| {
        SymbolEnv::new(
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
        )
    };
    let mut import_owner_builder = ResolvedArenaBuilder::new();
    let import_owner = import_owner_builder
        .push(ResolvedNode::new(
            syntax::SurfaceNodeKind::ImportAliasDecl,
            Vec::new(),
            actual_label.origin().clone(),
        ))
        .expect("actual import owner node");
    let mut temporary_imports = ResolvedImports::new();
    let imported_edge = temporary_imports.push_import(ResolvedImport::new(
        import_owner,
        range(fixture.source, 0, 1),
        "import statement.fixture;",
        None,
        ImportResolution::Resolved(fixture.module.clone()),
        actual_label.origin().clone(),
    ));
    assert_eq!(temporary_imports.imports().count(), 1);
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
                LabelOriginPath::new("statement.fixture.wrong.theorem")
            } else {
                actual_label.origin_path().clone()
            };
            let kind = if case == 4 {
                LabelKind::ProofStep
            } else {
                actual_label.kind()
            };
            let origin = match case {
                0 => actual_label
                    .origin()
                    .clone()
                    .with_import_edge(imported_edge),
                7 => actual_label.origin().clone().recovered(),
                _ => actual_label.origin().clone(),
            };
            let visibility = if case == 5 {
                Visibility::Private
            } else {
                actual_label.visibility()
            };
            let export_status = if case == 6 {
                ExportStatus::LocalOnly
            } else {
                actual_label.export_status()
            };
            labels.insert(
                LabelEntry::new(
                    origin_path,
                    kind,
                    actual_label.namespace().clone(),
                    actual_label.primary_spelling(),
                    origin,
                    actual_label.contribution(),
                )
                .with_visibility(visibility)
                .with_export_status(export_status),
            );
            if case == 2 {
                labels.insert(
                    LabelEntry::new(
                        LabelOriginPath::new("statement.fixture.duplicate.theorem"),
                        actual_label.kind(),
                        actual_label.namespace().clone(),
                        actual_label.primary_spelling(),
                        actual_label.origin().clone(),
                        actual_label.contribution(),
                    )
                    .with_visibility(actual_label.visibility())
                    .with_export_status(actual_label.export_status()),
                );
            }
        }
        let attempted_symbols = make_symbols_with_labels(labels);
        assert_ne!(attempted_symbols, pristine_symbols, "{name} env differs");
        assert_eq!(
            attempted_symbols.imports(),
            pristine_symbols.imports(),
            "{name} imports stay unchanged"
        );
        assert_eq!(
            attempted_symbols.contributions(),
            pristine_symbols.contributions(),
            "{name} contribution effects stay unchanged"
        );
        assert!(
            CheckedStatementOwner::validate_exact_local_theorem(
                &attempted_symbols,
                fixture.symbol.clone(),
                fixture.source,
                &fixture.module,
            )
            .is_ok(),
            "{name} owner-only check remains valid"
        );
        assert_base_refusal(
            &format!("label join {name}"),
            pristine_input.clone(),
            &attempted_symbols,
            SourceStatementError::InvalidOwner {
                owner: SourceTheoremOwnerId::new(0),
            },
        );
    }

    for mutation in 0..6 {
        let mut input = fixture.witness_input();
        match mutation {
            0 => {
                input.witnesses.pop();
            }
            1 => input.witnesses.push(input.witnesses[1].clone()),
            2 => input.names.clear(),
            3 => input.names.push(input.names[0].clone()),
            4 => input.witnesses.clear(),
            5 => {
                input.witnesses.swap(0, 1);
            }
            _ => unreachable!(),
        };
        assert_eq!(
            fixture.witnesses(&statement, input),
            if mutation == 5 {
                Err(SourceStatementWitnessError::InvalidWitness {
                    witness: SourceStatementWitnessId::new(0),
                })
            } else {
                Err(SourceStatementWitnessError::InvalidAggregate)
            },
            "aggregate/dense order {mutation}"
        );
    }
    for index in 0..2 {
        for mutation in 0..16 {
            let mut input = fixture.witness_input();
            let row = &mut input.witnesses[index];
            match mutation {
                0 => row.owner = SourceTheoremOwnerId::new(1),
                1 => row.binding_context = BindingContextId::new(0),
                2 => {
                    row.term = SourceStatementWitnessTermTarget::Primary(SourcePrimaryTermId::new(
                        if index == 0 { 3 } else { 2 },
                    ))
                }
                3 => row.take_site = node(41),
                4 => row.take_range.start += 1,
                5 => row.take_range.end -= 1,
                6 => row.site = node(if index == 0 { 41 } else { 38 }),
                7 => row.source_range.start += 1,
                8 => row.source_range.end -= 1,
                9 => row.source_ordinal = 2,
                10 => row.ordinal = 1 - index,
                11 => row.spelling.push('x'),
                12 => {
                    row.kind = if index == 0 {
                        SourceStatementWitnessKind::Unnamed
                    } else {
                        SourceStatementWitnessKind::Named
                    }
                }
                13 => row.recovery = SourceStatementRecovery::Degraded,
                14 => {
                    row.name = if index == 0 {
                        None
                    } else {
                        Some(SourceStatementWitnessNameId::new(0))
                    }
                }
                15 => row.name = Some(SourceStatementWitnessNameId::new(1)),
                _ => unreachable!(),
            }
            assert_eq!(
                fixture.witnesses(&statement, input),
                Err(SourceStatementWitnessError::InvalidWitness {
                    witness: SourceStatementWitnessId::new(index),
                }),
                "witness {index} field {mutation}"
            );
        }
    }
    for mutation in 0..6 {
        let mut input = fixture.witness_input();
        let row = &mut input.names[0];
        match mutation {
            0 => row.witness = SourceStatementWitnessId::new(1),
            1 => row.site = node(14),
            2 => row.source_range.start += 1,
            3 => row.source_range.end -= 1,
            4 => row.spelling.push('x'),
            5 => row.recovery = SourceStatementRecovery::Degraded,
            _ => unreachable!(),
        }
        assert_eq!(
            fixture.witnesses(&statement, input),
            Err(SourceStatementWitnessError::InvalidName {
                name: SourceStatementWitnessNameId::new(0),
            }),
            "name field {mutation}"
        );
    }
    let mut wrong_lower_name = fixture.witness_input();
    wrong_lower_name.names[0].spelling = "z".to_owned();
    let wrong_lower_name_before = wrong_lower_name.clone();
    assert_eq!(
        fixture.witnesses(&statement, wrong_lower_name.clone()),
        Err(SourceStatementWitnessError::InvalidName {
            name: SourceStatementWitnessNameId::new(0),
        }),
        "exact historical lower name spelling"
    );
    assert_eq!(wrong_lower_name, wrong_lower_name_before);
    assert_pristine();
    let mut aggregate_first = fixture.witness_input();
    aggregate_first.names.clear();
    aggregate_first.witnesses[0].owner = SourceTheoremOwnerId::new(1);
    assert_eq!(
        fixture.witnesses(&statement, aggregate_first),
        Err(SourceStatementWitnessError::InvalidAggregate)
    );
    let mut witness_zero_first = fixture.witness_input();
    witness_zero_first.witnesses[0].owner = SourceTheoremOwnerId::new(1);
    witness_zero_first.witnesses[1].owner = SourceTheoremOwnerId::new(1);
    witness_zero_first.names[0].spelling.push('x');
    assert_eq!(
        fixture.witnesses(&statement, witness_zero_first),
        Err(SourceStatementWitnessError::InvalidWitness {
            witness: SourceStatementWitnessId::new(0),
        })
    );
    let mut witness_one_first = fixture.witness_input();
    witness_one_first.witnesses[1].owner = SourceTheoremOwnerId::new(1);
    witness_one_first.names[0].spelling.push('x');
    assert_eq!(
        fixture.witnesses(&statement, witness_one_first),
        Err(SourceStatementWitnessError::InvalidWitness {
            witness: SourceStatementWitnessId::new(1),
        })
    );
    let mut name_last = fixture.witness_input();
    name_last.names[0].spelling.push('x');
    assert_eq!(
        fixture.witnesses(&statement, name_last),
        Err(SourceStatementWitnessError::InvalidName {
            name: SourceStatementWitnessNameId::new(0),
        })
    );
    let mut dependency_first = fixture.witness_input();
    dependency_first.source_id = B3M1Fixture::new(55).source;
    dependency_first.names.clear();
    assert_eq!(
        fixture.witnesses(&statement, dependency_first),
        Err(SourceStatementWitnessError::DependencyMismatch)
    );
    let mut stale_statement = baseline.clone();
    stale_statement.statement_fingerprint.push('x');
    assert_eq!(
        stale_statement.validate_installation(
            fixture.source,
            &fixture.module,
            &statement,
            &fixture.primary,
            &fixture.arena,
        ),
        Err(SourceStatementWitnessError::DependencyMismatch)
    );
    let mut stale_primary = baseline.clone();
    stale_primary.primary_term_fingerprint.push('x');
    assert_eq!(
        stale_primary.validate_installation(
            fixture.source,
            &fixture.module,
            &statement,
            &fixture.primary,
            &fixture.arena,
        ),
        Err(SourceStatementWitnessError::DependencyMismatch)
    );
    let b3n = B3NFixture::new(56);
    assert_eq!(
        baseline.validate_installation(
            fixture.source,
            &fixture.module,
            &b3n.statement(),
            &fixture.primary,
            &fixture.arena,
        ),
        Err(SourceStatementWitnessError::DependencyMismatch)
    );
    assert_eq!(
        baseline.validate_installation(
            fixture.source,
            &fixture.module,
            &statement,
            &b3n.primary,
            &fixture.arena,
        ),
        Err(SourceStatementWitnessError::DependencyMismatch)
    );
    for index in 0..56 {
        for mutation in 0..5 {
            let arena = mutate_arena(&fixture.arena, |id, row| {
                if id.index() == index {
                    match mutation {
                        0 => {
                            let SourceAnchor::Range(mut source_range) = row.anchor.clone() else {
                                unreachable!("Task258B3M1 range")
                            };
                            source_range.end += 1;
                            row.anchor = SourceAnchor::Range(source_range);
                        }
                        1 => row.recovery = NodeRecoveryState::Recovered,
                        2 => row.recovery = NodeRecoveryState::Degraded,
                        3 => row.kind = "source.task258b3m1.mutated".into(),
                        4 if row.children.len() > 1 => row.children.swap(0, 1),
                        4 if row.children.len() == 1 => row.children.clear(),
                        4 => row.children.push(TypedNodeId::new(usize::from(index == 0))),
                        _ => unreachable!(),
                    }
                }
            });
            assert_eq!(
                baseline.validate_installation(
                    fixture.source,
                    &fixture.module,
                    &statement,
                    &fixture.primary,
                    &arena,
                ),
                Err(SourceStatementWitnessError::DependencyMismatch),
                "node {index} mutation {mutation}"
            );
            let witness_input = fixture.witness_input();
            let witness_input_before = witness_input.clone();
            let arena_before = arena.clone();
            assert_eq!(
                SourceStatementWitnessProducer::build(
                    witness_input.clone(),
                    &statement,
                    &fixture.primary,
                    &arena,
                ),
                Err(SourceStatementWitnessError::DependencyMismatch),
                "direct witness producer node {index} mutation {mutation}"
            );
            assert_eq!(witness_input, witness_input_before);
            assert_eq!(arena, arena_before);
            assert_pristine();
        }
    }
    assert_eq!(
        fixture
            .witnesses(&statement, fixture.witness_input())
            .expect("replay")
            .debug_text(),
        baseline_debug
    );
}

#[test]
fn task258b3m1_paired_ownership_hybrids_and_all_family_orders_are_atomic() {
    let fixture = B3M1Fixture::new(57);
    let statement = fixture.statement();
    let witnesses = fixture
        .witnesses(&statement, fixture.witness_input())
        .expect("witnesses");
    let base = fixture.empty_typed();
    let base_debug = base.debug_text();
    assert_eq!(
        base.clone().with_source_statement(statement.clone()),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(base.debug_text(), base_debug);
    let typed = base
        .clone()
        .with_source_statement_witnesses(statement.clone(), witnesses.clone())
        .expect("paired install");
    let typed_debug = typed.debug_text();
    assert_eq!(typed.source_statement(), Some(&statement));
    assert_eq!(typed.source_statement_witnesses(), Some(&witnesses));
    assert!(typed.source_statement_references().is_none());
    assert_eq!(
        typed
            .clone()
            .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(typed.debug_text(), typed_debug);

    let task258a = Fixture::new(57);
    let task258a_statement = task258a
        .build(task258a.input())
        .expect("Task258A statement");
    let task258a_typed = TypedAst::try_new(TypedAstParts {
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
    .expect("Task258A empty")
    .with_source_term(task258a.primary.clone())
    .expect("Task258A primary")
    .with_source_atomic_formula(task258a.atomic.clone())
    .expect("Task258A atomic")
    .with_source_statement(task258a_statement.clone())
    .expect("Task258A install");
    let task258a_debug = task258a_typed.debug_text();
    assert_eq!(
        task258a_typed
            .clone()
            .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(task258a_typed.debug_text(), task258a_debug);
    assert_eq!(
        typed.clone().with_source_statement(task258a_statement),
        Err(TypedAstError::InvalidSourceStatement)
    );

    let b1 = B1Fixture::new(57);
    let b1_references = b1.references(b1.reference_input()).expect("B1 references");
    let b1_typed = b1
        .empty_typed()
        .with_source_statement_references(b1.statement.clone(), b1_references.clone())
        .expect("B1 install");
    let b1_debug = b1_typed.debug_text();
    assert_eq!(
        b1_typed
            .clone()
            .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(b1_typed.debug_text(), b1_debug);
    assert_eq!(
        typed
            .clone()
            .with_source_statement_references(b1.statement.clone(), b1_references.clone(),),
        Err(TypedAstError::InvalidSourceStatement)
    );

    let b2 = B2Fixture::new(57);
    let b2_statement = b2.build(b2.input()).expect("B2 statement");
    let b2_typed = b2
        .empty_typed()
        .with_source_statement(b2_statement.clone())
        .expect("B2 install");
    let b2_debug = b2_typed.debug_text();
    assert_eq!(
        b2_typed
            .clone()
            .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(b2_typed.debug_text(), b2_debug);
    assert_eq!(
        typed.clone().with_source_statement(b2_statement),
        Err(TypedAstError::InvalidSourceStatement)
    );

    let b3 = B3Fixture::new(58);
    let b3_statement = b3.statement();
    let b3_witnesses = b3
        .witnesses(&b3_statement, b3.witness_input())
        .expect("B3 witnesses");
    let b3_typed = b3
        .empty_typed()
        .with_source_statement_witnesses(b3_statement.clone(), b3_witnesses.clone())
        .expect("B3 install");
    let b3_debug = b3_typed.debug_text();
    assert_eq!(
        base.clone()
            .with_source_statement_witnesses(b3_statement.clone(), b3_witnesses.clone(),),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        b3_typed
            .clone()
            .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(b3_typed.debug_text(), b3_debug);
    assert_eq!(
        typed
            .clone()
            .with_source_statement_witnesses(b3_statement, b3_witnesses),
        Err(TypedAstError::InvalidSourceStatement)
    );

    let b3n = B3NFixture::new(59);
    let b3n_statement = b3n.statement();
    let b3n_witnesses = b3n
        .witnesses(&b3n_statement, b3n.witness_input())
        .expect("B3N witnesses");
    let b3n_typed = b3n
        .empty_typed()
        .with_source_statement_witnesses(b3n_statement.clone(), b3n_witnesses.clone())
        .expect("B3N install");
    let b3n_debug = b3n_typed.debug_text();
    assert_eq!(
        base.clone()
            .with_source_statement_witnesses(b3n_statement.clone(), b3n_witnesses.clone(),),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        b3n_typed
            .clone()
            .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(b3n_typed.debug_text(), b3n_debug);
    assert_eq!(
        typed
            .clone()
            .with_source_statement_witnesses(b3n_statement, b3n_witnesses),
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
            .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(task248.debug_text(), task248_debug);
    assert_eq!(
        typed.clone().with_source_context_for_test(
            task248
                .source_context()
                .expect("Task248 source context")
                .clone(),
        ),
        Err(TypedAstError::InvalidSourceContext)
    );

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
            occupied
                .clone()
                .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
            Err(TypedAstError::InvalidSourceStatement),
            "{family}"
        );
        assert_eq!(occupied.debug_text(), debug, "{family} rollback");
    }
    assert_eq!(
        typed.clone().with_source_composite_formula(
            task257a
                .source_composite_formula()
                .expect("Task257A composite")
                .clone(),
        ),
        Err(TypedAstError::InvalidSourceCompositeFormula)
    );
    assert_eq!(
        typed.clone().with_source_formula_composition(
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
        typed.clone().with_source_condition_formula_composition(
            task257c2
                .source_condition_formula_composition()
                .expect("Task257C2 composition")
                .clone(),
        ),
        Err(TypedAstError::InvalidSourceConditionFormulaComposition)
    );
    assert_eq!(
        typed.clone().with_source_predicate_chain_composition(
            task257c3
                .source_predicate_chain_composition()
                .expect("Task257C3 composition")
                .clone(),
        ),
        Err(TypedAstError::InvalidSourcePredicateChainComposition)
    );
    assert_eq!(typed.debug_text(), typed_debug);
    assert_eq!(
        base.with_source_statement_witnesses(statement, witnesses)
            .expect("replay")
            .debug_text(),
        typed_debug
    );
}

#[test]
fn task258b3m1_final_clone_revalidation_and_semantic_deferrals_are_stable() {
    let fixture = B3M1Fixture::new(60);
    let statement = fixture.statement();
    let witnesses = fixture
        .witnesses(&statement, fixture.witness_input())
        .expect("witnesses");
    let typed = fixture
        .empty_typed()
        .with_source_statement_witnesses(statement.clone(), witnesses.clone())
        .expect("typed");
    let typed_debug = typed.debug_text();
    let resolved = assemble_empty_resolved(&typed).expect("resolved");
    let resolved_debug = resolved.debug_text();
    assert_eq!(typed.clone().debug_text(), typed_debug);
    assert_eq!(resolved.clone().debug_text(), resolved_debug);
    assert_eq!(resolved.source_statement(), Some(&statement));
    assert_eq!(resolved.source_statement_witnesses(), Some(&witnesses));
    assert!(resolved.source_statement_references().is_none());
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

    let mut orphan = fixture.empty_typed();
    orphan.inject_source_statement_witnesses_for_test(witnesses.clone());
    assert_eq!(
        assemble_empty_resolved(&orphan),
        Err(ResolvedTypedAstError::InvalidSourceStatement)
    );
    let mut standalone = fixture.empty_typed();
    standalone.inject_source_statement_for_test(statement.clone());
    assert_eq!(
        assemble_empty_resolved(&standalone),
        Err(ResolvedTypedAstError::InvalidSourceStatement)
    );
    let mut stale_statement = witnesses.clone();
    stale_statement.statement_fingerprint.push('x');
    let mut stale_statement_pair = fixture.empty_typed();
    stale_statement_pair
        .inject_source_statement_witness_bundle_for_test(statement.clone(), stale_statement);
    assert_eq!(
        assemble_empty_resolved(&stale_statement_pair),
        Err(ResolvedTypedAstError::InvalidSourceStatement)
    );
    let mut stale_primary = witnesses.clone();
    stale_primary.primary_term_fingerprint.push('x');
    let mut stale_primary_pair = fixture.empty_typed();
    stale_primary_pair
        .inject_source_statement_witness_bundle_for_test(statement.clone(), stale_primary);
    assert_eq!(
        assemble_empty_resolved(&stale_primary_pair),
        Err(ResolvedTypedAstError::InvalidSourceStatement)
    );
    let b1 = B1Fixture::new(60);
    let mut reference_hybrid = fixture.empty_typed();
    reference_hybrid.inject_source_statement_bundle_for_test(
        statement.clone(),
        b1.references(b1.reference_input()).expect("B1 references"),
    );
    reference_hybrid.inject_source_statement_witnesses_for_test(witnesses.clone());
    assert_eq!(
        assemble_empty_resolved(&reference_hybrid),
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
        let occupied_debug = occupied.debug_text();
        assert_eq!(
            assemble_empty_resolved(&occupied),
            Err(ResolvedTypedAstError::InvalidSourceStatement),
            "{table:?}"
        );
        assert_eq!(occupied.debug_text(), occupied_debug, "{table:?} rollback");
    }
    let term_formula = TermFormulaChecker::default().infer(
        &fixture.symbols,
        &fixture.bindings,
        Vec::<crate::type_checker::TermInput>::new(),
        Vec::<crate::type_checker::FormulaInput>::new(),
    );
    assert_eq!(
        assemble_resolved_with_statement_semantics(
            &typed,
            statement.checked_owner(),
            &fixture.bindings,
            &term_formula,
        ),
        Err(ResolvedTypedAstError::InvalidSourceStatement)
    );
    let mut cluster_facts = ClusterFactTable::new();
    cluster_facts.insert(ClusterFactDraft {
        fingerprint: ClusterFactFingerprint::new("task258b3m1-coexistence"),
        source_type: ClusterTypeFingerprint::new("set"),
        attribute: ClusterAttributeFingerprint::new("inhabited"),
        generated_type: ClusterTypeFingerprint::new("set"),
        provenance: ClusterFactProvenance::Input,
        source_range: range(fixture.source, 84, 85),
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
                typed_node: TypedNodeId::new(13),
                kind: ResolvedNodeKindHintKind::SourcePreserved {
                    role: SourceNodeRole::new("source.statement-witness-name.semantic-coexistence",),
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
                expr: ExprId::new("task258b3m1-semantic-coexistence"),
                typed_site: node(36),
                local_context: None,
                cluster_facts: Vec::new(),
            }],
        ),
        Err(ResolvedTypedAstError::InvalidSourceStatement)
    );
    let empty_collection = OverloadCollectionOutput::collect(
        Vec::<OverloadSiteInput>::new(),
        Vec::<OverloadCandidateInput>::new(),
    );
    let empty_expansion = TemplateExpansionOutput::expand(&empty_collection);
    let empty_viability =
        CandidateViabilityOutput::filter(&empty_expansion, Vec::<CandidateViabilityInput>::new());
    let empty_specificity =
        SpecificityGraphOutput::build(&empty_viability, Vec::<SpecificityComparisonInput>::new());
    let empty_selection = OverloadSelectionOutput::resolve(
        &empty_specificity,
        Vec::<OverloadSiteResolutionInput>::new(),
    );
    let occupied_collection = OverloadCollectionOutput::collect(
        vec![OverloadSiteInput {
            key: OverloadSiteKey::new("task258b3m1-overload"),
            owner: TypedSiteRef::Role {
                node: TypedNodeId::new(36),
                role: TypeRole::new("task258b3m1-overload-owner"),
            },
            source_range: range(fixture.source, 88, 89),
            kind: OverloadSiteKind::FunctorApplication,
            name: OverloadNameKey::new("task258b3m1-overload"),
            arguments: Vec::new(),
            expected: None,
            source_qua: Vec::<SourceQuaView>::new(),
            recovery: OverloadSiteRecovery::Normal,
        }],
        vec![OverloadCandidateInput {
            site: OverloadSiteKey::new("task258b3m1-overload"),
            symbol: fixture.symbol.clone(),
            ordinary_root: fixture.symbol.clone(),
            declaration_kind: CandidateDeclarationKind::Functor,
            parameters: Vec::new(),
            result: None,
            origin: CandidateOrigin::Ordinary,
            template: None,
            coherence: None,
            provenance: CandidateProvenance {
                stable_key: CandidateProvenanceKey::new("task258b3m1-overload"),
                source_range: Some(range(fixture.source, 88, 89)),
                scope: CandidateScope::Local,
                declaration_order: 0,
            },
        }],
    );
    let occupied_expansion = TemplateExpansionOutput::expand(&occupied_collection);
    let occupied_viability = CandidateViabilityOutput::filter(
        &occupied_expansion,
        vec![CandidateViabilityInput {
            candidate: OverloadCandidateId::new(0),
            arguments: Vec::new(),
        }],
    );
    let occupied_specificity = SpecificityGraphOutput::build(
        &occupied_viability,
        Vec::<SpecificityComparisonInput>::new(),
    );
    let occupied_selection = OverloadSelectionOutput::resolve(
        &occupied_specificity,
        vec![OverloadSiteResolutionInput {
            site: OverloadSiteId::new(0),
            refinements: vec![OverloadCandidateId::new(0)],
            refinement_join: RefinementJoinPayload {
                status: RefinementJoinStatus::Compatible,
                exposed_result: None,
            },
            inserted_views: Vec::new(),
        }],
    );
    assert_eq!(occupied_collection.candidates().len(), 1);
    assert_eq!(occupied_expansion.candidates().len(), 1);
    assert_eq!(occupied_viability.candidates().len(), 1);
    assert_eq!(occupied_specificity.candidates().len(), 1);
    assert_eq!(occupied_selection.results().len(), 1);
    let cluster_facts = ClusterFactTable::new();
    for (label, collection, expansion, viability, specificity, selection) in [
        (
            "collection",
            &occupied_collection,
            &empty_expansion,
            &empty_viability,
            &empty_specificity,
            &empty_selection,
        ),
        (
            "expansion",
            &empty_collection,
            &occupied_expansion,
            &empty_viability,
            &empty_specificity,
            &empty_selection,
        ),
        (
            "viability",
            &empty_collection,
            &empty_expansion,
            &occupied_viability,
            &empty_specificity,
            &empty_selection,
        ),
        (
            "specificity",
            &empty_collection,
            &empty_expansion,
            &empty_viability,
            &occupied_specificity,
            &empty_selection,
        ),
        (
            "selection",
            &empty_collection,
            &empty_expansion,
            &empty_viability,
            &empty_specificity,
            &occupied_selection,
        ),
    ] {
        assert_eq!(
            ResolvedTypedAst::assemble(ResolvedTypedAstInputs {
                typed_ast: &typed,
                cluster_facts: &cluster_facts,
                overload_collection: collection,
                template_expansion: expansion,
                viability,
                specificity,
                overload_selection: selection,
                expressions: Vec::new(),
                node_hints: Vec::new(),
                statement_semantics: None,
                statement_proofs: None,
            }),
            Err(ResolvedTypedAstError::InvalidSourceStatement),
            "{label}"
        );
    }
    assert_eq!(typed.debug_text(), typed_debug);
    assert_eq!(
        assemble_empty_resolved(&typed)
            .expect("final replay")
            .debug_text(),
        resolved_debug
    );
}
