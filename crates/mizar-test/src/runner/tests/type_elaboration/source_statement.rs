use super::{
    SOURCE_STATEMENT_B1_TEXT, SOURCE_STATEMENT_B2_TEXT, SOURCE_STATEMENT_B3_TEXT,
    SOURCE_STATEMENT_B3M1_TEXT, SOURCE_STATEMENT_B3N_TEXT, SOURCE_STATEMENT_TEXT,
    SourceStatementB1Extraction, SourceStatementB2Extraction, SourceStatementB3Extraction,
    SourceStatementB3M1Extraction, SourceStatementB3M1RouteInputs,
    SourceStatementB3NExtraction, SourceStatementB3NRouteInputs, SourceStatementB3RouteInputs,
    SourceStatementExtraction, SourceStatementRouteInputs, SourceStatementRouteOutput,
    extract_multiple_witness_source_statement, extract_named_witness_source_statement,
    extract_nested_source_statement, extract_single_assumption_source_statement,
    extract_single_witness_source_statement, extract_source_reserved_variable_theorem_statement,
    source_statement_b1_output_with_mutation, source_statement_b2_output_with_mutation,
    source_statement_b2_output_with_resolver_mutation, source_statement_b3_output_with_mutation,
    source_statement_b3_output_with_resolver_mutation,
    source_statement_b3_resolver_env_for_test, source_statement_b3n_output_with_mutation,
    source_statement_b3n_output_with_resolver_mutation,
    source_statement_b3n_resolver_env_for_test, source_statement_b3m1_output_with_mutation,
    source_statement_b3m1_output_with_resolver_mutation,
    source_statement_b3m1_resolver_env_for_test, source_statement_output_with_source,
    source_statement_output_with_source_and_mutation,
    source_statement_output_with_resolver_mutation, source_statement_resolver_env_for_test,
    source_statement_transport_detail_keys,
};

fn sha256_text(text: &str) -> String {
    use std::{
        io::Write as _,
        process::{Command, Stdio},
    };

    let mut child = Command::new("sha256sum")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("sha256sum");
    child
        .stdin
        .as_mut()
        .expect("sha256 stdin")
        .write_all(text.as_bytes())
        .expect("hash input");
    let output = child.wait_with_output().expect("hash output");
    assert!(output.status.success());
    String::from_utf8(output.stdout)
        .expect("hash text")
        .split_whitespace()
        .next()
        .expect("hash")
        .to_owned()
}

#[test]
fn task258b3m1_real_frontend_freezes_mixed_multiple_witness_contract() {
    assert_eq!(SOURCE_STATEMENT_B3M1_TEXT.len(), 113);
    assert_eq!(
        sha256_text(SOURCE_STATEMENT_B3M1_TEXT),
        "412a6a7f8fddebd67418f3482855ea89a1e7da922b42ebb93463971d8e49c186"
    );
    assert!(SOURCE_STATEMENT_B3M1_TEXT.ends_with('\n'));
    let (ast, module, _, symbols) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B3M1_TEXT, 258_800);
    let extracted: SourceStatementB3M1Extraction =
        extract_multiple_witness_source_statement(&ast, SOURCE_STATEMENT_B3M1_TEXT)
            .expect("Task258B3M1 exact parser shape");
    assert_eq!((ast.nodes().len(), ast.root().expect("root").index()), (56, 55));
    assert_eq!(
        extracted
            .statement_sites
            .iter()
            .map(|site| site.node().index())
            .collect::<Vec<_>>(),
        [52, 50]
    );
    assert_eq!(
        extracted
            .formula_sites
            .iter()
            .map(|site| site.node().index())
            .collect::<Vec<_>>(),
        [34, 47]
    );
    assert_eq!(
        extracted
            .term_sites
            .iter()
            .map(|site| site.node().index())
            .collect::<Vec<_>>(),
        [30, 32, 36, 39, 43, 45]
    );
    assert_eq!(
        extracted
            .term_ranges
            .iter()
            .map(|range| (range.start, range.end))
            .collect::<Vec<_>>(),
        [
            (65, 66),
            (69, 70),
            (88, 89),
            (91, 92),
            (101, 102),
            (105, 106)
        ]
    );
    assert_eq!(
        (
            extracted.take_site.node().index(),
            extracted
                .witness_sites
                .iter()
                .map(|site| site.node().index())
                .collect::<Vec<_>>(),
            extracted.name_site.node().index(),
            (extracted.proof_range.start, extracted.proof_range.end),
        ),
        (42, vec![38, 41], 13, (71, 111))
    );
    let resolver =
        source_statement_b3m1_resolver_env_for_test(&module, &symbols, &extracted)
            .expect("Task258B3M1 resolver provenance");
    assert_eq!((resolver.labels().len(), resolver.contributions().len()), (1, 1));
    assert!(resolver.imports().is_empty());
    let labels = resolver.labels().visible_candidates(
        &mizar_resolve::env::NamespacePath::new(module.path().as_str()),
        "FormulaStatementMultipleWitnessSmoke",
    );
    let [label] = labels.as_slice() else {
        panic!("one Task258B3M1 theorem label")
    };
    assert_eq!(label.kind(), mizar_resolve::resolved_ast::LabelKind::Theorem);
    assert_eq!(label.visibility(), mizar_resolve::env::Visibility::Public);
    assert_eq!(
        label.export_status(),
        mizar_resolve::env::ExportStatus::Exported
    );
    assert_eq!(label.contribution().index(), 0);
    assert_eq!(label.origin().structural_path(), [2, 1]);
    assert!(
        resolver
            .symbols()
            .visible_candidates(
                &mizar_resolve::env::NamespacePath::new(module.path().as_str()),
                "y",
            )
            .is_empty()
    );

    let output = source_statement_output_with_source(
        &ast,
        module,
        &symbols,
        SOURCE_STATEMENT_B3M1_TEXT,
    )
    .expect("Task258B3M1 selector")
    .expect("Task258B3M1 route");
    let statement = output.typed_ast.source_statement().expect("statement");
    let primary = output.typed_ast.source_term().expect("primary");
    let atomic = output
        .typed_ast
        .source_atomic_formula()
        .expect("atomic");
    let handoff = output
        .typed_ast
        .source_statement_witnesses()
        .expect("witness handoff");
    assert_eq!(
        (
            statement.binding_env().contexts().len(),
            statement.binding_env().bindings().len(),
            statement.binding_env().diagnostics().len(),
            primary.terms().len(),
            primary.references().len(),
            atomic.formulas().len(),
            atomic.edges().len(),
            atomic.requests().len(),
        ),
        (2, 1, 0, 6, 6, 2, 4, 4)
    );
    let proof = statement
        .binding_env()
        .contexts()
        .get(mizar_checker::binding_env::BindingContextId::new(1))
        .expect("proof context");
    assert_eq!(
        proof.owner,
        mizar_checker::binding_env::BindingContextOwner::SourceStatement {
            source_range: mizar_session::SourceRange {
                source_id: ast.source_id,
                start: 71,
                end: 111,
            },
        }
    );
    for (index, ((_, term), (_, reference))) in primary
        .terms()
        .iter()
        .zip(primary.references().iter())
        .enumerate()
    {
        assert_eq!(term.site().node().index(), [30, 32, 36, 39, 43, 45][index]);
        assert_eq!(term.source_ordinal(), index);
        assert_eq!(term.context().index(), usize::from(index >= 2));
        assert_eq!(term.spelling(), "x");
        assert_eq!(reference.term().index(), index);
        assert_eq!(reference.binding().index(), 0);
        assert_eq!(reference.use_ordinal(), 1);
    }
    assert_eq!(
        atomic
            .edges()
            .iter()
            .map(|(_, edge)| match edge.target() {
                mizar_checker::source_atomic_formula::SourceAtomicTermTarget::Primary(term) => {
                    term.index()
                }
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
            handoff.witnesses().len(),
            handoff.names().len(),
        ),
        (1, 2, 2, 2, 2, 2, 1)
    );
    let owner = statement
        .owners()
        .get(mizar_checker::source_statement::SourceTheoremOwnerId::new(0))
        .expect("owner");
    assert_eq!(
        (
            owner.site().node().index(),
            owner.source_range().start,
            owner.source_range().end,
            owner.spelling(),
            owner.contribution().index(),
        ),
        (52, 19, 112, "FormulaStatementMultipleWitnessSmoke", 0)
    );
    for index in 0..2 {
        let row = statement
            .statements()
            .get(mizar_checker::source_statement::SourceStatementId::new(index))
            .expect("statement row");
        assert_eq!(
            (
                row.site().node().index(),
                row.source_range().start,
                row.source_range().end,
                row.source_ordinal(),
            ),
            (
                [52, 50][index],
                [19, 96][index],
                [112, 107][index],
                [0, 2][index],
            )
        );
        let fact = statement
            .input_facts()
            .get(mizar_checker::source_statement::SourceStatementInputFactId::new(index))
            .expect("input fact");
        assert_eq!(
            fact.uses()
                .iter()
                .map(|id| id.index())
                .collect::<Vec<_>>(),
            if index == 0 { vec![0, 1] } else { vec![4, 5] }
        );
    }
    for index in 0..2 {
        let witness = handoff
            .witnesses()
            .get(mizar_checker::source_statement::SourceStatementWitnessId::new(index))
            .expect("witness");
        assert_eq!(
            (
                witness.owner().index(),
                witness.binding_context().index(),
                witness.term(),
                witness.take_site().node().index(),
                witness.take_range().start,
                witness.take_range().end,
                witness.site().node().index(),
                witness.source_range().start,
                witness.source_range().end,
            ),
            (
                0,
                1,
                mizar_checker::source_statement::SourceStatementWitnessTermTarget::Primary(
                    mizar_checker::source_term::SourcePrimaryTermId::new(2 + index)
                ),
                42,
                79,
                93,
                [38, 41][index],
                [84, 91][index],
                [89, 92][index],
            )
        );
        assert_eq!(
            (
                witness.source_ordinal(),
                witness.ordinal(),
                witness.spelling(),
                witness.kind(),
                witness.name().map(|id| id.index()),
            ),
            (
                1,
                index,
                ["y = x", "x"][index],
                [
                    mizar_checker::source_statement::SourceStatementWitnessKind::Named,
                    mizar_checker::source_statement::SourceStatementWitnessKind::Unnamed,
                ][index],
                [Some(0), None][index],
            )
        );
    }
    let name = handoff
        .names()
        .get(mizar_checker::source_statement::SourceStatementWitnessNameId::new(0))
        .expect("name");
    assert_eq!(name.witness().index(), 0);
    assert_eq!(
        (
            name.site().node().index(),
            name.source_range().start,
            name.source_range().end,
            name.spelling(),
        ),
        (13, 84, 85, "y")
    );
    assert_eq!(handoff.statement_fingerprint(), statement.debug_text());
    assert_eq!(handoff.primary_term_fingerprint(), primary.debug_text());
    assert_eq!(output.reference_use_ordinals, [1; 6]);
    for index in 0..56 {
        let surface = ast.nodes().get(index).expect("surface node");
        let typed = output
            .typed_ast
            .nodes()
            .node(mizar_checker::typed_ast::TypedNodeId::new(index))
            .expect("typed node");
        assert_eq!(typed.anchor, mizar_session::SourceAnchor::Range(surface.range));
        assert_eq!(
            typed
                .children
                .iter()
                .map(|child| child.index())
                .collect::<Vec<_>>(),
            surface
                .children
                .iter()
                .map(|child| child.index())
                .collect::<Vec<_>>()
        );
        assert_eq!(
            typed.recovery,
            mizar_checker::typed_ast::NodeRecoveryState::Normal
        );
    }
    assert_eq!(
        output.typed_ast.source_statement(),
        output.resolved.source_statement()
    );
    assert_eq!(
        output.typed_ast.source_statement_witnesses(),
        output.resolved.source_statement_witnesses()
    );
    assert!(output.typed_ast.source_statement_references().is_none());
}

#[test]
fn task258b3m1_validation_precedence_mutation_and_replay_fail_closed() {
    let (ast, module, _, symbols) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B3M1_TEXT, 258_801);
    let baseline = source_statement_output_with_source(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B3M1_TEXT,
    )
    .expect("baseline selector")
    .expect("baseline");
    let baseline_typed = baseline.typed_ast.debug_text();
    let baseline_resolved = baseline.resolved.debug_text();
    let (b3n_ast, b3n_module, _, b3n_symbols) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B3N_TEXT, 258_802);
    let snapshot = mizar_session::BuildSnapshotId::from_published_schema_str(&format!(
        "mizar-session-build-snapshot-v1:{}",
        "e8".repeat(32)
    ))
    .expect("foreign snapshot");
    let allocator = mizar_session::InMemorySessionIdAllocator::new();
    let mut foreign_source =
        mizar_session::SessionIdAllocator::next_source_id(&allocator, snapshot)
            .expect("foreign source");
    while foreign_source == ast.source_id {
        foreign_source =
            mizar_session::SessionIdAllocator::next_source_id(&allocator, snapshot)
                .expect("distinct foreign source");
    }
    let mut foreign_contributions = mizar_resolve::env::SourceContributionIndex::new();
    let contribution_anchor = mizar_session::SourceAnchor::Range(mizar_session::SourceRange {
        source_id: ast.source_id,
        start: 0,
        end: 18,
    });
    let _ = foreign_contributions.insert(
        module.clone(),
        mizar_resolve::env::ContributionKind::LocalSource {
            source_id: ast.source_id,
        },
        contribution_anchor.clone(),
    );
    let foreign_contribution = foreign_contributions.insert(
        module.clone(),
        mizar_resolve::env::ContributionKind::LocalSource {
            source_id: ast.source_id,
        },
        contribution_anchor,
    );
    let b3n = source_statement_output_with_source(
        &b3n_ast,
        b3n_module,
        &b3n_symbols,
        SOURCE_STATEMENT_B3N_TEXT,
    )
    .expect("B3N selector")
    .expect("B3N output");
    let wrong_binding = b3n
        .typed_ast
        .source_statement()
        .expect("B3N statement")
        .binding_env()
        .clone();
    let wrong_primary = b3n.typed_ast.source_term().expect("B3N primary").clone();
    let wrong_atomic = b3n
        .typed_ast
        .source_atomic_formula()
        .expect("B3N atomic")
        .clone();
    let assert_replay = || {
        let replay = source_statement_output_with_source(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M1_TEXT,
        )
        .expect("replay selector")
        .expect("replay");
        assert_eq!(replay.typed_ast.debug_text(), baseline_typed);
        assert_eq!(replay.resolved.debug_text(), baseline_resolved);
    };
    let stale_witnesses = b3n
        .typed_ast
        .source_statement_witnesses()
        .expect("B3N witness fingerprints")
        .clone();
    let empty = mizar_checker::typed_ast::TypedAst::try_new(
        mizar_checker::typed_ast::TypedAstParts {
            source_id: ast.source_id,
            module_id: module.clone(),
            resolved_root: None,
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes: baseline.typed_ast.nodes().clone(),
            contexts: mizar_checker::typed_ast::LocalTypeContextTable::new(),
            types: mizar_checker::typed_ast::TypeTable::new(),
            facts: mizar_checker::typed_ast::TypeFactTable::new(),
            coercions: mizar_checker::typed_ast::CoercionTable::new(),
            initial_obligations: mizar_checker::typed_ast::InitialObligationTable::new(),
            diagnostics: mizar_checker::typed_ast::TypeDiagnosticTable::new(),
        },
    )
    .expect("empty B3M1 typed AST")
    .with_source_term(
        baseline
            .typed_ast
            .source_term()
            .expect("B3M1 primary")
            .clone(),
    )
    .expect("B3M1 primary install")
    .with_source_atomic_formula(
        baseline
            .typed_ast
            .source_atomic_formula()
            .expect("B3M1 atomic")
            .clone(),
    )
    .expect("B3M1 atomic install");
    assert_eq!(
        empty.with_source_statement_witnesses(
            baseline
                .typed_ast
                .source_statement()
                .expect("B3M1 statement")
                .clone(),
            stale_witnesses,
        ),
        Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement),
        "copied cross-profile witness fingerprints must fail"
    );
    assert_replay();

    for mutation in 0..5 {
        let expected = ["dependency", "aggregate", "witness 0", "witness 1", "name 0"][mutation];
        let error = source_statement_b3m1_output_with_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M1_TEXT,
            |input: &mut SourceStatementB3M1RouteInputs| match mutation {
                0 => {
                    input.binding_env = wrong_binding.clone();
                    input.witness.names.clear();
                    input.witness.witnesses[0].ordinal = 1;
                }
                1 => {
                    input.witness.names.clear();
                    input.witness.witnesses[0].ordinal = 1;
                }
                2 => {
                    input.witness.witnesses[0].ordinal = 1;
                    input.witness.witnesses[1].ordinal = 0;
                    input.witness.names[0].spelling.push('z');
                }
                3 => {
                    input.witness.witnesses[1].ordinal = 0;
                    input.witness.names[0].spelling.push('z');
                }
                4 => input.witness.names[0].spelling.push('z'),
                _ => unreachable!(),
            },
        )
        .expect("precedence selector")
        .expect_err("mixed fault must fail");
        assert!(
            error.to_ascii_lowercase().contains(expected),
            "precedence {mutation}: {error}"
        );
        let replay = source_statement_output_with_source(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M1_TEXT,
        )
        .expect("precedence replay selector")
        .expect("precedence replay");
        assert_eq!(replay.typed_ast.debug_text(), baseline_typed);
        assert_eq!(replay.resolved.debug_text(), baseline_resolved);
    }

    for (label, expected, mutation) in [
        ("missing witness", "aggregate", 0usize),
        ("extra witness", "aggregate", 1),
        ("missing name", "aggregate", 2),
        ("extra name", "aggregate", 3),
        ("wrong primary dependency", "dependency", 4),
        ("wrong atomic dependency", "dependency", 5),
        ("statement aggregate", "aggregate", 6),
    ] {
        let error = source_statement_b3m1_output_with_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M1_TEXT,
            |input| match mutation {
                0 => {
                    input.witness.witnesses.pop();
                }
                1 => input
                    .witness
                    .witnesses
                    .push(input.witness.witnesses[1].clone()),
                2 => input.witness.names.clear(),
                3 => input.witness.names.push(input.witness.names[0].clone()),
                4 => input.primary = wrong_primary.clone(),
                5 => input.atomic = wrong_atomic.clone(),
                6 => {
                    input.statement.candidate_facts.pop();
                }
                _ => unreachable!(),
            },
        )
        .unwrap_or_else(|| panic!("{label} selector"))
        .expect_err(label);
        assert!(
            error.to_ascii_lowercase().contains(expected),
            "{label}: {error}"
        );
        assert_replay();
    }

    for mutation in 0..10 {
        let error = source_statement_b3m1_output_with_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M1_TEXT,
            |input| match mutation {
                0 => input.statement.owners.clear(),
                1 => input
                    .statement
                    .owners
                    .push(input.statement.owners[0].clone()),
                2 => {
                    input.statement.statements.pop();
                }
                3 => input
                    .statement
                    .statements
                    .push(input.statement.statements[1].clone()),
                4 => {
                    input.statement.contexts.pop();
                }
                5 => input
                    .statement
                    .contexts
                    .push(input.statement.contexts[1].clone()),
                6 => {
                    input.statement.input_facts.pop();
                }
                7 => input
                    .statement
                    .input_facts
                    .push(input.statement.input_facts[1].clone()),
                8 => {
                    input.statement.candidate_facts.pop();
                }
                9 => input
                    .statement
                    .candidate_facts
                    .push(input.statement.candidate_facts[1].clone()),
                _ => unreachable!(),
            },
        )
        .expect("base aggregate selector")
        .expect_err("base aggregate mutation");
        assert!(
            error.to_ascii_lowercase().contains("aggregate"),
            "base aggregate {mutation}: {error}"
        );
        assert_replay();
    }
    for mutation in 0..9 {
        let error = source_statement_b3m1_output_with_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M1_TEXT,
            |input| match mutation {
                0 => input.statement.source_id = foreign_source,
                1 => {
                    input.statement.module_id =
                        ResolverModuleId::new(PackageId::new("pkg"), ModulePath::new("wrong"))
                }
                2 => input.statement.owners[0].site = input.statement.statements[1].site.clone(),
                3 => input.statement.owners[0].source_range.start += 1,
                4 => input.statement.owners[0].source_range.end -= 1,
                5 => input.statement.owners[0].spelling.push('z'),
                6 => {
                    input.statement.owners[0].recovery =
                        mizar_checker::source_statement::SourceStatementRecovery::Degraded
                }
                7 => {
                    input.statement.owners[0].symbol = b3n
                        .typed_ast
                        .source_statement()
                        .expect("B3N statement")
                        .owners()
                        .get(mizar_checker::source_statement::SourceTheoremOwnerId::new(0))
                        .expect("B3N owner")
                        .symbol()
                        .clone()
                }
                8 => {
                    input.statement.owners[0].contribution = foreign_contribution
                }
                _ => unreachable!(),
            },
        )
        .expect("owner selector")
        .expect_err("owner mutation");
        assert!(!error.is_empty(), "owner field {mutation}");
        assert_replay();
    }
    for index in 0..2 {
        let other = 1 - index;
        for mutation in 0..10 {
            let error = source_statement_b3m1_output_with_mutation(
                &ast,
                module.clone(),
                &symbols,
                SOURCE_STATEMENT_B3M1_TEXT,
                |input| {
                    let other_site = input.statement.statements[other].site.clone();
                    let row = &mut input.statement.statements[index];
                    match mutation {
                        0 => {
                            row.owner =
                                mizar_checker::source_statement::SourceTheoremOwnerId::new(1)
                        }
                        1 => {
                            row.context =
                                mizar_checker::source_statement::SourceStatementContextId::new(other)
                        }
                        2 => {
                            row.formula =
                                mizar_checker::source_statement::SourceStatementFormulaTarget::
                                    Atomic(
                                        mizar_checker::source_atomic_formula::
                                            SourceAtomicFormulaId::new(other),
                                    )
                        }
                        3 => row.site = other_site,
                        4 => row.source_range.start += 1,
                        5 => row.source_range.end -= 1,
                        6 => row.source_ordinal = 1,
                        7 => row.spelling.push('z'),
                        8 => {
                            row.recovery =
                                mizar_checker::source_statement::SourceStatementRecovery::Degraded
                        }
                        9 => {
                            row.kind = if index == 0 {
                                mizar_checker::source_statement::SourceStatementKind::Conclusion
                            } else {
                                mizar_checker::source_statement::SourceStatementKind::
                                    TheoremProposition
                            }
                        }
                        _ => unreachable!(),
                    }
                },
            )
            .expect("statement selector")
            .expect_err("statement mutation");
            assert!(!error.is_empty(), "statement {index} field {mutation}");
            assert_replay();
        }
        for mutation in 0..5 {
            let error = source_statement_b3m1_output_with_mutation(
                &ast,
                module.clone(),
                &symbols,
                SOURCE_STATEMENT_B3M1_TEXT,
                |input| {
                    let row = &mut input.statement.contexts[index];
                    match mutation {
                        0 => {
                            row.statement =
                                mizar_checker::source_statement::SourceStatementId::new(other)
                        }
                        1 => {
                            row.binding_context =
                                mizar_checker::binding_env::BindingContextId::new(other)
                        }
                        2 => row.source_range.start += 1,
                        3 => row.source_range.end -= 1,
                        4 => row.visible_bindings.clear(),
                        _ => unreachable!(),
                    }
                },
            )
            .expect("context selector")
            .expect_err("context mutation");
            assert!(!error.is_empty(), "context {index} field {mutation}");
            assert_replay();
        }
        for mutation in 0..8 {
            let error = source_statement_b3m1_output_with_mutation(
                &ast,
                module.clone(),
                &symbols,
                SOURCE_STATEMENT_B3M1_TEXT,
                |input| {
                    let row = &mut input.statement.input_facts[index];
                    match mutation {
                        0 => {
                            row.statement =
                                mizar_checker::source_statement::SourceStatementId::new(other)
                        }
                        1 => {
                            row.context =
                                mizar_checker::source_statement::SourceStatementContextId::new(other)
                        }
                        2 => row.ordinal = 1,
                        3 => row.binding = mizar_checker::binding_env::BindingId::new(1),
                        4 => row.uses.swap(0, 1),
                        5 => row.uses.clear(),
                        6 => row.uses.push(row.uses[0]),
                        7 => {
                            row.uses[0] =
                                mizar_checker::source_term::SourcePrimaryTermReferenceId::new(99)
                        }
                        _ => unreachable!(),
                    }
                },
            )
            .expect("input fact selector")
            .expect_err("input fact mutation");
            assert!(!error.is_empty(), "input fact {index} field {mutation}");
            assert_replay();
        }
        for mutation in 0..4 {
            let error = source_statement_b3m1_output_with_mutation(
                &ast,
                module.clone(),
                &symbols,
                SOURCE_STATEMENT_B3M1_TEXT,
                |input| {
                    let row = &mut input.statement.candidate_facts[index];
                    match mutation {
                        0 => {
                            row.statement =
                                mizar_checker::source_statement::SourceStatementId::new(other)
                        }
                        1 => {
                            row.context =
                                mizar_checker::source_statement::SourceStatementContextId::new(other)
                        }
                        2 => row.ordinal = 1,
                        3 => {
                            row.formula =
                                mizar_checker::source_statement::SourceStatementFormulaTarget::
                                    Atomic(
                                        mizar_checker::source_atomic_formula::
                                            SourceAtomicFormulaId::new(other),
                                    )
                        }
                        _ => unreachable!(),
                    }
                },
            )
            .expect("candidate selector")
            .expect_err("candidate mutation");
            assert!(!error.is_empty(), "candidate {index} field {mutation}");
            assert_replay();
        }
    }

    for witness_index in 0..2 {
        for mutation in 0..16 {
            let error = source_statement_b3m1_output_with_mutation(
                &ast,
                module.clone(),
                &symbols,
                SOURCE_STATEMENT_B3M1_TEXT,
                |input| {
                    let row = &mut input.witness.witnesses[witness_index];
                    match mutation {
                        0 => {
                            row.owner =
                                mizar_checker::source_statement::SourceTheoremOwnerId::new(1)
                        }
                        1 => {
                            row.binding_context =
                                mizar_checker::binding_env::BindingContextId::new(0)
                        }
                        2 => {
                            row.term =
                                mizar_checker::source_statement::SourceStatementWitnessTermTarget::
                                    Primary(mizar_checker::source_term::SourcePrimaryTermId::new(0))
                        }
                        3 => row.take_site = row.site.clone(),
                        4 => row.take_range.start += 1,
                        5 => row.take_range.end -= 1,
                        6 => row.site = row.take_site.clone(),
                        7 => row.source_range.start += 1,
                        8 => row.source_range.end -= 1,
                        9 => row.source_ordinal = 2,
                        10 => row.ordinal = 1 - witness_index,
                        11 => row.spelling.push('z'),
                        12 => {
                            row.recovery =
                                mizar_checker::source_statement::SourceStatementRecovery::Degraded
                        }
                        13 => {
                            row.kind =
                                if witness_index == 0 {
                                    mizar_checker::source_statement::SourceStatementWitnessKind::
                                        Unnamed
                                } else {
                                    mizar_checker::source_statement::SourceStatementWitnessKind::
                                        Named
                                }
                        }
                        14 => {
                            row.name = if witness_index == 0 {
                                None
                            } else {
                                Some(
                                    mizar_checker::source_statement::SourceStatementWitnessNameId::
                                        new(0),
                                )
                            }
                        }
                        15 => input.witness.module_id =
                            ResolverModuleId::new(PackageId::new("pkg"), ModulePath::new("wrong")),
                        _ => unreachable!(),
                    }
                },
            )
            .expect("witness row selector")
            .expect_err("witness row mutation");
            let expected = if mutation == 15 {
                "dependency"
            } else {
                if witness_index == 0 {
                    "witness 0"
                } else {
                    "witness 1"
                }
            };
            assert!(
                error.to_ascii_lowercase().contains(expected),
                "witness {witness_index} field {mutation}: {error}"
            );
            assert_replay();
        }
    }
    for mutation in 0..6 {
        let error = source_statement_b3m1_output_with_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M1_TEXT,
            |input| match mutation {
                0 => {
                    input.witness.names[0].witness =
                        mizar_checker::source_statement::SourceStatementWitnessId::new(1)
                }
                1 => input.witness.names[0].site = input.witness.witnesses[0].site.clone(),
                2 => input.witness.names[0].source_range.start += 1,
                3 => input.witness.names[0].source_range.end -= 1,
                4 => input.witness.names[0].spelling.push('z'),
                5 => {
                    input.witness.names[0].recovery =
                        mizar_checker::source_statement::SourceStatementRecovery::Degraded
                }
                _ => unreachable!(),
            },
        )
        .expect("name row selector")
        .expect_err("name row mutation");
        assert!(
            error.to_ascii_lowercase().contains("name 0"),
            "name field {mutation}: {error}"
        );
        assert_replay();
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
        let error = source_statement_b3m1_output_with_resolver_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M1_TEXT,
            |symbols| task258b2_mutate_resolver(symbols, mutation),
        )
        .expect("resolver selector")
        .expect_err("resolver mutation");
        assert!(
            error.to_ascii_lowercase().contains("resolver")
                || error.to_ascii_lowercase().contains("owner")
                || error.to_ascii_lowercase().contains("label"),
            "{mutation:?}: {error}"
        );
        assert_replay();
    }
    let witness_symbol_error = source_statement_b3m1_output_with_resolver_mutation(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B3M1_TEXT,
        task258b3m1_env_with_witness_symbol,
    )
    .expect("witness-symbol resolver selector")
    .expect_err("resolver-owned witness name must fail");
    assert!(
        witness_symbol_error
            .to_ascii_lowercase()
            .contains("provenance"),
        "{witness_symbol_error}"
    );
    assert_replay();

    for index in 0..56 {
        for mutation in 0..5 {
            let error = source_statement_b3m1_output_with_mutation(
                &ast,
                module.clone(),
                &symbols,
                SOURCE_STATEMENT_B3M1_TEXT,
                |input| {
                    input.arena = mizar_checker::typed_ast::TypedArena::try_new(
                        input.arena.root(),
                        input
                            .arena
                            .iter()
                            .map(|(id, row)| {
                                let mut row = row.clone();
                                if id.index() == index {
                                    match mutation {
                                        0 => {
                                            let mizar_session::SourceAnchor::Range(mut range) =
                                                row.anchor.clone()
                                            else {
                                                unreachable!("range anchor")
                                            };
                                            range.end += 1;
                                            row.anchor = mizar_session::SourceAnchor::Range(range);
                                        }
                                        1 => {
                                            row.recovery = mizar_checker::typed_ast::
                                                NodeRecoveryState::Recovered
                                        }
                                        2 => {
                                            row.recovery = mizar_checker::typed_ast::
                                                NodeRecoveryState::Degraded
                                        }
                                        3 => row.kind = "source.task258b3m1.mutated".into(),
                                        4 if row.children.len() > 1 => row.children.swap(0, 1),
                                        4 if row.children.len() == 1 => row.children.clear(),
                                        4 => row.children.push(
                                            mizar_checker::typed_ast::TypedNodeId::new(
                                                usize::from(index == 0),
                                            ),
                                        ),
                                        _ => unreachable!(),
                                    }
                                }
                                row
                            })
                            .collect(),
                    )
                    .expect("structurally valid arena mutation");
                },
            )
            .expect("arena selector")
            .expect_err("arena mutation");
            assert!(!error.is_empty(), "node {index} mutation {mutation}");
        }
        let replay = source_statement_output_with_source(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M1_TEXT,
        )
        .expect("arena replay selector")
        .expect("arena replay");
        assert_eq!(replay.typed_ast.debug_text(), baseline_typed, "node {index}");
        assert_eq!(replay.resolved.debug_text(), baseline_resolved, "node {index}");
    }
}

#[test]
fn task258b3m1_selector_and_byte_subtree_near_misses_are_exact() {
    let (ast, module, _, symbols) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B3M1_TEXT, 258_803);
    let replay = source_statement_b3m1_output_with_resolver_mutation(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B3M1_TEXT,
        |symbols| symbols,
    )
    .expect("resolver selector")
    .expect("resolver replay");
    assert_eq!((replay.typed_ast.nodes().len(), replay.reference_use_ordinals.len()), (56, 6));
    for (ordinal, source) in [
        SOURCE_STATEMENT_B3M1_TEXT.trim_end_matches('\n').to_owned(),
        format!("{SOURCE_STATEMENT_B3M1_TEXT}\n"),
        SOURCE_STATEMENT_B3M1_TEXT.replacen("  take y = x, x;", "   take y = x, x;", 1),
        SOURCE_STATEMENT_B3M1_TEXT.replacen(
            "FormulaStatementMultipleWitnessSmoke",
            "FormulaStatementMultipleWitnessNearMiss",
            1,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        assert!(
            source_statement_output_with_source(&ast, module.clone(), &symbols, &source).is_none(),
            "byte near miss {ordinal}"
        );
    }
    let reordered = SOURCE_STATEMENT_B3M1_TEXT.replacen(
        "  take y = x, x;\n  thus x = x;\n",
        "  thus x = x;\n  take y = x, x;\n",
        1,
    );
    for (ordinal, source) in [
        SOURCE_STATEMENT_B3_TEXT.to_owned(),
        SOURCE_STATEMENT_B3N_TEXT.to_owned(),
        SOURCE_STATEMENT_B3M1_TEXT.replacen("reserve x for set;", "reserve z for set;", 1),
        SOURCE_STATEMENT_B3M1_TEXT.replacen("take y = x, x;", "take x, y = x;", 1),
        SOURCE_STATEMENT_B3M1_TEXT.replacen("take y = x, x;", "take x, x;", 1),
        SOURCE_STATEMENT_B3M1_TEXT.replacen("take y = x, x;", "take y = x;", 1),
        SOURCE_STATEMENT_B3M1_TEXT.replacen("take y = x, x;", "take y = x, x, x;", 1),
        SOURCE_STATEMENT_B3M1_TEXT.replacen("take y = x, x;", "take y = x, y = x;", 1),
        SOURCE_STATEMENT_B3M1_TEXT.replacen("take y = x, x;", "take z = x, x;", 1),
        SOURCE_STATEMENT_B3M1_TEXT.replacen("take y = x, x;", "take y <> x, x;", 1),
        SOURCE_STATEMENT_B3M1_TEXT.replacen("take y = x, x;", "take y = {x}, x;", 1),
        SOURCE_STATEMENT_B3M1_TEXT.replacen("  take y = x, x;\n", "", 1),
        SOURCE_STATEMENT_B3M1_TEXT.replacen(
            "  take y = x, x;\n",
            "  take y = x;\n  take x;\n",
            1,
        ),
        SOURCE_STATEMENT_B3M1_TEXT.replacen(
            "  thus x = x;\n",
            "  thus x = x;\n  thus x = x;\n",
            1,
        ),
        reordered,
        SOURCE_STATEMENT_B3M1_TEXT.replacen("take y = x, x;", "take y = x, x", 1),
        SOURCE_STATEMENT_B3M1_TEXT.replacen(
            ": x = x proof",
            ": ex z being set st z = z proof",
            1,
        ),
        SOURCE_STATEMENT_B3M1_TEXT.replacen(
            ": x = x proof",
            ": (x = x) & (x = x) proof",
            1,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let (near_ast, near_module, _, near_symbols) =
            task253_ast_from_source_text(&source, 258_810 + ordinal);
        assert!(
            extract_multiple_witness_source_statement(&near_ast, SOURCE_STATEMENT_B3M1_TEXT)
                .is_none(),
            "subtree extraction near miss {ordinal}"
        );
        assert!(
            source_statement_output_with_source(
                &near_ast,
                near_module,
                &near_symbols,
                SOURCE_STATEMENT_B3M1_TEXT,
            )
            .is_none(),
            "subtree route near miss {ordinal}"
        );
    }
}

#[test]
fn task258b3m1_family_and_active_route_isolation_is_atomic_in_both_orders() {
    let (ast, module, _, symbols) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B3M1_TEXT, 258_804);
    let output = source_statement_output_with_source(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B3M1_TEXT,
    )
    .expect("B3M1 selector")
    .expect("B3M1 route");
    let statement = output
        .typed_ast
        .source_statement()
        .expect("B3M1 statement")
        .clone();
    let witnesses = output
        .typed_ast
        .source_statement_witnesses()
        .expect("B3M1 witnesses")
        .clone();
    let baseline_debug = output.typed_ast.debug_text();

    for (ordinal, source) in [
        SOURCE_STATEMENT_TEXT,
        SOURCE_STATEMENT_B1_TEXT,
        SOURCE_STATEMENT_B2_TEXT,
        SOURCE_STATEMENT_B3_TEXT,
        SOURCE_STATEMENT_B3N_TEXT,
    ]
    .into_iter()
    .enumerate()
    {
        let (foreign_ast, foreign_module, _, foreign_symbols) =
            task253_ast_from_source_text(source, 258_830 + ordinal);
        let foreign = source_statement_output_with_source(
            &foreign_ast,
            foreign_module,
            &foreign_symbols,
            source,
        )
        .expect("foreign selector")
        .expect("foreign route");
        let foreign_debug = foreign.typed_ast.debug_text();
        assert_eq!(
            foreign
                .typed_ast
                .clone()
                .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
            Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement),
            "foreign-first family {ordinal}"
        );
        assert_eq!(foreign.typed_ast.debug_text(), foreign_debug);
        if let Some(references) = foreign.typed_ast.source_statement_references() {
            assert_eq!(
                output.typed_ast.clone().with_source_statement_references(
                    foreign
                        .typed_ast
                        .source_statement()
                        .expect("foreign statement")
                        .clone(),
                    references.clone(),
                ),
                Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement),
                "B3M1-first reference family {ordinal}"
            );
        } else if let Some(foreign_witnesses) =
            foreign.typed_ast.source_statement_witnesses()
        {
            assert_eq!(
                output.typed_ast.clone().with_source_statement_witnesses(
                    foreign
                        .typed_ast
                        .source_statement()
                        .expect("foreign statement")
                        .clone(),
                    foreign_witnesses.clone(),
                ),
                Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement),
                "B3M1-first witness family {ordinal}"
            );
        } else {
            assert_eq!(
                output.typed_ast.clone().with_source_statement(
                    foreign
                        .typed_ast
                        .source_statement()
                        .expect("foreign statement")
                        .clone(),
                ),
                Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement),
                "B3M1-first base family {ordinal}"
            );
        }
        let foreign_binding = foreign
            .typed_ast
            .source_statement()
            .expect("foreign statement")
            .binding_env()
            .clone();
        let foreign_primary = foreign
            .typed_ast
            .source_term()
            .expect("foreign primary")
            .clone();
        let foreign_atomic = foreign
            .typed_ast
            .source_atomic_formula()
            .expect("foreign atomic")
            .clone();
        let error = source_statement_b3m1_output_with_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3M1_TEXT,
            move |input| {
                input.binding_env = foreign_binding;
                input.primary = foreign_primary;
                input.atomic = foreign_atomic;
            },
        )
        .expect("cross-family selector")
        .expect_err("cross-family lower tuple");
        assert!(
            error.to_ascii_lowercase().contains("dependency"),
            "foreign family {ordinal}: {error}"
        );
        assert_eq!(output.typed_ast.debug_text(), baseline_debug);
    }

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
    let plan = build_test_plan(&config).expect("Task258B3M1 isolation plan");
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
        let extracted =
            extract_multiple_witness_source_statement(&active_ast, &source).is_some();
        let selected_route = source_statement_output_with_source(
            &active_ast,
            resolver.module.clone(),
            &active_symbols,
            &source,
        )
        .is_some_and(|result| {
            result.is_ok_and(|output| {
                output
                    .typed_ast
                    .source_statement_witnesses()
                    .is_some_and(|handoff| {
                        handoff.witnesses().len() == 2 && handoff.names().len() == 1
                    })
            })
        });
        if extracted || selected_route {
            selected.push(case.id.0.clone());
        }
    }
    assert!(
        selected.is_empty(),
        "Task258B3M1 selected active cases: {selected:?}"
    );
}

#[test]
fn task258b3m1_typed_final_clone_debug_rollback_and_empty_semantics_are_stable() {
    let (ast, module, _, symbols) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B3M1_TEXT, 258_805);
    let output = source_statement_output_with_source(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B3M1_TEXT,
    )
    .expect("selector")
    .expect("route");
    assert_eq!(output.typed_ast.clone().debug_text(), output.typed_ast.debug_text());
    assert_eq!(output.resolved.clone().debug_text(), output.resolved.debug_text());
    assert_eq!(
        output.typed_ast.source_statement(),
        output.resolved.source_statement()
    );
    assert_eq!(
        output.typed_ast.source_statement_witnesses(),
        output.resolved.source_statement_witnesses()
    );
    assert!(output.typed_ast.source_statement_references().is_none());
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
    assert!(output.resolved.checked_formulas().is_empty());
    assert!(output.resolved.statement_semantics().is_empty());
    assert!(output.resolved.checked_proofs().is_empty());
    assert!(output.resolved.checked_proof_nodes().is_empty());
    assert!(output.resolved.checked_terminal_goals().is_empty());
    let baseline_typed = output.typed_ast.debug_text();
    let baseline_resolved = output.resolved.debug_text();
    let error = source_statement_b3m1_output_with_mutation(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B3M1_TEXT,
        |input| {
            input.witness.witnesses[1].spelling.push('z');
            input.witness.names[0].spelling.push('z');
        },
    )
    .expect("rollback selector")
    .expect_err("invalid second witness must roll back");
    assert!(
        error.to_ascii_lowercase().contains("witness 1"),
        "{error}"
    );
    let replay = source_statement_output_with_source(
        &ast,
        module,
        &symbols,
        SOURCE_STATEMENT_B3M1_TEXT,
    )
    .expect("replay selector")
    .expect("replay");
    assert_eq!(replay.typed_ast.debug_text(), baseline_typed);
    assert_eq!(replay.resolved.debug_text(), baseline_resolved);
}

#[test]
fn task258b3n_real_frontend_freezes_named_witness_table_contract() {
    assert_eq!(SOURCE_STATEMENT_B3N_TEXT.len(), 107);
    assert_eq!(
        sha256_text(SOURCE_STATEMENT_B3N_TEXT),
        "a57022c4b75991dd4308943477e03819f5bfe2c0d23ea1030730256252d7d329"
    );
    assert!(SOURCE_STATEMENT_B3N_TEXT.ends_with('\n'));
    let (ast, module, _, symbols) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B3N_TEXT, 258_700);
    let extracted: SourceStatementB3NExtraction =
        extract_named_witness_source_statement(&ast, SOURCE_STATEMENT_B3N_TEXT)
            .expect("Task258B3N exact parser shape");
    assert_eq!((ast.nodes().len(), ast.root().expect("root").index()), (51, 50));
    assert_eq!(
        extracted
            .statement_sites
            .iter()
            .map(|site| site.node().index())
            .collect::<Vec<_>>(),
        [47, 45]
    );
    assert_eq!(
        extracted
            .formula_sites
            .iter()
            .map(|site| site.node().index())
            .collect::<Vec<_>>(),
        [32, 42]
    );
    assert_eq!(
        extracted
            .statement_ranges
            .iter()
            .map(|range| (range.start, range.end))
            .collect::<Vec<_>>(),
        [(19, 106), (90, 101)]
    );
    assert_eq!(
        extracted
            .formula_ranges
            .iter()
            .map(|range| (range.start, range.end))
            .collect::<Vec<_>>(),
        [(62, 67), (95, 100)]
    );
    assert_eq!(
        extracted
            .term_sites
            .iter()
            .map(|site| site.node().index())
            .collect::<Vec<_>>(),
        [28, 30, 34, 38, 40]
    );
    assert_eq!(
        extracted
            .term_ranges
            .iter()
            .map(|range| (range.start, range.end))
            .collect::<Vec<_>>(),
        [(62, 63), (66, 67), (85, 86), (95, 96), (99, 100)]
    );
    assert_eq!(
        (
            extracted.take_site.node().index(),
            extracted.witness_site.node().index(),
            extracted.name_site.node().index(),
        ),
        (37, 36, 13)
    );
    let resolver =
        source_statement_b3n_resolver_env_for_test(&module, &symbols, &extracted)
            .expect("Task258B3N resolver provenance");
    assert_eq!(resolver.labels().len(), 1);
    assert!(resolver.imports().is_empty());
    assert_eq!(resolver.contributions().len(), 1);
    let label = resolver
        .labels()
        .visible_candidates(
            &mizar_resolve::env::NamespacePath::new(module.path().as_str()),
            "FormulaStatementNamedWitnessSmoke",
        );
    let [label] = label.as_slice() else {
        panic!("one resolver theorem label")
    };
    assert_eq!(label.kind(), mizar_resolve::resolved_ast::LabelKind::Theorem);
    assert_eq!(label.visibility(), mizar_resolve::env::Visibility::Public);
    assert_eq!(
        label.export_status(),
        mizar_resolve::env::ExportStatus::Exported
    );
    assert_eq!(label.contribution().index(), 0);
    assert_eq!(label.origin().structural_path(), [2, 1]);

    let output = source_statement_output_with_source(
        &ast,
        module,
        &symbols,
        SOURCE_STATEMENT_B3N_TEXT,
    )
    .expect("Task258B3N selector")
    .expect("Task258B3N route");
    let statement = output.typed_ast.source_statement().expect("statement");
    let handoff = output
        .typed_ast
        .source_statement_witnesses()
        .expect("witness handoff");
    assert_eq!(
        (
            statement.binding_env().contexts().len(),
            statement.binding_env().bindings().len(),
            statement.binding_env().diagnostics().len(),
        ),
        (2, 1, 0)
    );
    let primary = output.typed_ast.source_term().expect("primary");
    assert_eq!(
        (
            primary.terms().len(),
            primary.references().len(),
            primary.numeric_type_requests().len(),
        ),
        (5, 5, 0)
    );
    let atomic = output
        .typed_ast
        .source_atomic_formula()
        .expect("atomic");
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
    let proof_context = statement
        .binding_env()
        .contexts()
        .get(mizar_checker::binding_env::BindingContextId::new(1))
        .expect("proof context");
    assert_eq!(
        proof_context.owner,
        mizar_checker::binding_env::BindingContextOwner::SourceStatement {
            source_range: mizar_session::SourceRange {
                source_id: ast.source_id,
                start: 68,
                end: 105,
            }
        }
    );
    assert_eq!(
        (
            proof_context.parent.map(|id| id.index()),
            proof_context.layer,
            proof_context
                .lexical_scope
                .as_ref()
                .map(|scope| scope.path().to_vec()),
            proof_context
                .bindings
                .iter()
                .map(|id| id.index())
                .collect::<Vec<_>>(),
            proof_context
                .visible_bindings
                .iter()
                .map(|id| id.index())
                .collect::<Vec<_>>(),
            proof_context.recovery,
        ),
        (
            Some(0),
            mizar_checker::binding_env::BindingContextLayer::Proof,
            Some(vec![0]),
            Vec::new(),
            vec![0],
            mizar_checker::binding_env::BindingContextRecovery::Normal,
        )
    );
    for (index, ((_, term), (_, reference))) in primary
        .terms()
        .iter()
        .zip(primary.references().iter())
        .enumerate()
    {
        assert_eq!(term.site().node().index(), [28, 30, 34, 38, 40][index]);
        assert_eq!(
            (term.source_range().start, term.source_range().end),
            [(62, 63), (66, 67), (85, 86), (95, 96), (99, 100)][index]
        );
        assert_eq!(term.source_ordinal(), index);
        assert_eq!(term.context().index(), [0, 0, 1, 1, 1][index]);
        assert_eq!(term.spelling(), "x");
        assert_eq!(
            term.kind(),
            mizar_checker::source_term::SourcePrimaryTermKind::VariableReference
        );
        assert_eq!(
            term.role(),
            mizar_checker::source_term::SourcePrimaryTermRole::Value
        );
        assert_eq!(
            term.recovery(),
            mizar_checker::source_term::SourcePrimaryTermRecovery::Normal
        );
        assert!(term.parent().is_none());
        assert_eq!(reference.term().index(), index);
        assert_eq!(reference.binding().index(), 0);
        assert_eq!(
            reference.role(),
            mizar_checker::source_term::SourcePrimaryTermReferenceRole::Variable
        );
        assert_eq!(reference.use_ordinal(), 1);
        assert_eq!(
            reference
                .lexical_scope()
                .map(|scope| scope.path().to_vec()),
            if index < 2 { None } else { Some(vec![0]) }
        );
    }
    for index in 0..2 {
        let formula = atomic
            .formulas()
            .get(mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(index))
            .expect("atomic formula");
        assert_eq!(formula.site().node().index(), [32, 42][index]);
        assert_eq!(
            (formula.source_range().start, formula.source_range().end),
            [(62, 67), (95, 100)][index]
        );
        assert_eq!(formula.source_ordinal(), index);
        assert_eq!(formula.context().index(), index);
        assert_eq!(formula.spelling(), "x = x");
        assert_eq!(
            formula.kind(),
            mizar_checker::source_atomic_formula::SourceAtomicFormulaKind::Equality
        );
        assert_eq!(
            formula.recovery(),
            mizar_checker::source_atomic_formula::SourceAtomicFormulaRecovery::Normal
        );
    }
    assert_eq!(
        atomic
            .edges()
            .iter()
            .map(|(_, edge)| match edge.target() {
                mizar_checker::source_atomic_formula::SourceAtomicTermTarget::Primary(term) => {
                    term.index()
                }
                _ => usize::MAX,
            })
            .collect::<Vec<_>>(),
        [0, 1, 3, 4]
    );
    for index in 0..4 {
        let edge = atomic
            .edges()
            .get(mizar_checker::source_atomic_formula::SourceAtomicEdgeId::new(index))
            .expect("atomic edge");
        let formula = usize::from(index >= 2);
        let ordinal = index % 2;
        assert_eq!((edge.formula().index(), edge.ordinal()), (formula, ordinal));
        assert_eq!(
            edge.role(),
            if ordinal == 0 {
                mizar_checker::source_atomic_formula::SourceAtomicEdgeRole::BuiltinLeftOperand
            } else {
                mizar_checker::source_atomic_formula::SourceAtomicEdgeRole::BuiltinRightOperand
            }
        );
        let request = atomic
            .requests()
            .get(mizar_checker::source_atomic_formula::SourceAtomicRequestId::new(index))
            .expect("atomic request");
        assert_eq!((request.formula().index(), request.ordinal()), (formula, ordinal));
        assert_eq!(
            request.kind(),
            mizar_checker::source_atomic_formula::SourceAtomicRequestKind::OperandExpectedType
        );
        assert_eq!(request.edge().map(|id| id.index()), Some(index));
        assert!(request.candidate().is_none());
        assert!(request.type_site().is_none());
        assert!(request.attribute().is_none());
    }
    let owner = statement
        .owners()
        .get(mizar_checker::source_statement::SourceTheoremOwnerId::new(0))
        .expect("owner");
    assert_eq!(
        (
            owner.site().node().index(),
            owner.source_range().start,
            owner.source_range().end,
            owner.spelling(),
            owner.contribution().index(),
            owner.role(),
            owner.status(),
            owner.recovery(),
        ),
        (
            47,
            19,
            106,
            "FormulaStatementNamedWitnessSmoke",
            0,
            mizar_checker::source_statement::SourceTheoremRole::Theorem,
            mizar_checker::source_statement::SourceTheoremStatus::Unmodified,
            mizar_checker::source_statement::SourceStatementRecovery::Normal,
        )
    );
    assert_eq!(statement.checked_owner().origin().structural_path(), [2, 1]);
    assert_eq!(statement.checked_owner().symbol(), owner.symbol());
    for index in 0..2 {
        let row = statement
            .statements()
            .get(mizar_checker::source_statement::SourceStatementId::new(index))
            .expect("statement row");
        assert_eq!(
            (
                row.owner().index(),
                row.context().index(),
                row.site().node().index(),
                row.source_range().start,
                row.source_range().end,
                row.source_ordinal(),
                row.spelling(),
                row.kind(),
                row.recovery(),
            ),
            (
                0,
                index,
                [47, 45][index],
                [19, 90][index],
                [106, 101][index],
                [0, 2][index],
                [
                    "theorem FormulaStatementNamedWitnessSmoke : x = x proof take y = x ; thus x = x ; end ;",
                    "thus x = x ;",
                ][index],
                [
                    mizar_checker::source_statement::SourceStatementKind::TheoremProposition,
                    mizar_checker::source_statement::SourceStatementKind::Conclusion,
                ][index],
                mizar_checker::source_statement::SourceStatementRecovery::Normal,
            )
        );
        assert_eq!(
            row.formula(),
            mizar_checker::source_statement::SourceStatementFormulaTarget::Atomic(
                mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(index)
            )
        );
        let context = statement
            .contexts()
            .get(mizar_checker::source_statement::SourceStatementContextId::new(index))
            .expect("statement context");
        assert_eq!(
            (
                context.statement().index(),
                context.binding_context().index(),
                context.source_range(),
                context
                    .visible_bindings()
                    .iter()
                    .map(|id| id.index())
                    .collect::<Vec<_>>(),
            ),
            (index, index, row.source_range(), vec![0])
        );
        let fact = statement
            .input_facts()
            .get(mizar_checker::source_statement::SourceStatementInputFactId::new(index))
            .expect("input fact");
        assert_eq!(
            (
                fact.statement().index(),
                fact.context().index(),
                fact.ordinal(),
                fact.kind(),
                fact.binding().index(),
                fact.uses()
                    .iter()
                    .map(|id| id.index())
                    .collect::<Vec<_>>(),
            ),
            (
                index,
                index,
                0,
                mizar_checker::source_statement::SourceStatementInputFactKind::ReservedTypeGuard,
                0,
                if index == 0 { vec![0, 1] } else { vec![3, 4] },
            )
        );
        let candidate = statement
            .candidate_facts()
            .get(mizar_checker::source_statement::SourceStatementCandidateFactId::new(index))
            .expect("candidate fact");
        assert_eq!(
            (
                candidate.statement().index(),
                candidate.context().index(),
                candidate.ordinal(),
                candidate.kind(),
                candidate.formula(),
            ),
            (
                index,
                index,
                0,
                mizar_checker::source_statement::SourceStatementCandidateFactKind::UnverifiedProposition,
                row.formula(),
            )
        );
    }
    assert_eq!(
        (
            statement.owners().len(),
            statement.statements().len(),
            statement.contexts().len(),
            statement.input_facts().len(),
            statement.candidate_facts().len(),
            handoff.witnesses().len(),
            handoff.names().len(),
        ),
        (1, 2, 2, 2, 2, 1, 1)
    );
    let witness = handoff
        .witnesses()
        .get(mizar_checker::source_statement::SourceStatementWitnessId::new(0))
        .expect("witness");
    let name = handoff
        .names()
        .get(mizar_checker::source_statement::SourceStatementWitnessNameId::new(0))
        .expect("name");
    assert_eq!(witness.name(), Some(mizar_checker::source_statement::SourceStatementWitnessNameId::new(0)));
    assert_eq!(witness.kind(), mizar_checker::source_statement::SourceStatementWitnessKind::Named);
    assert_eq!(witness.spelling(), "y = x");
    assert_eq!(handoff.statement_fingerprint(), statement.debug_text());
    assert_eq!(handoff.primary_term_fingerprint(), primary.debug_text());
    assert_eq!(name.witness(), mizar_checker::source_statement::SourceStatementWitnessId::new(0));
    assert_eq!((name.site().node().index(), name.source_range().start, name.source_range().end), (13, 81, 82));
    assert_eq!(name.spelling(), "y");
    assert_eq!(
        [
            statement
                .statements()
                .get(mizar_checker::source_statement::SourceStatementId::new(0))
                .expect("theorem")
                .source_ordinal(),
            witness.source_ordinal(),
            statement
                .statements()
                .get(mizar_checker::source_statement::SourceStatementId::new(1))
                .expect("conclusion")
                .source_ordinal(),
        ],
        [0, 1, 2]
    );
    for index in 0..51 {
        let surface = ast.nodes().get(index).expect("surface node");
        let typed = output
            .typed_ast
            .nodes()
            .node(mizar_checker::typed_ast::TypedNodeId::new(index))
            .expect("typed node");
        assert_eq!(
            typed.anchor,
            mizar_session::SourceAnchor::Range(surface.range),
            "node {index}"
        );
        assert_eq!(
            typed
                .children
                .iter()
                .map(|child| child.index())
                .collect::<Vec<_>>(),
            surface
                .children
                .iter()
                .map(|child| child.index())
                .collect::<Vec<_>>(),
            "node {index} children"
        );
    }
    assert!(handoff.debug_text().contains(
        "witness#0 owner=0 binding_context=1 term=primary#2 take_range=76..87 take_site=37 range=81..86 site=36 source_ordinal=1 ordinal=0 kind=named recovery=normal spelling=\"y = x\" name=0\nwitness-name#0 witness=0 range=81..82 site=13 recovery=normal spelling=\"y\""
    ));
    assert_eq!(output.reference_use_ordinals, [1; 5]);
    assert_eq!(
        output.typed_ast.source_statement(),
        output.resolved.source_statement()
    );
    assert_eq!(
        output.typed_ast.source_statement_witnesses(),
        output.resolved.source_statement_witnesses()
    );
    assert!(output.resolved.source_statement_references().is_none());
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
    assert!(output.resolved.checked_formulas().is_empty());
    assert!(output.resolved.statement_semantics().is_empty());
    assert!(output.resolved.checked_proofs().is_empty());
    assert!(output.resolved.checked_proof_nodes().is_empty());
    assert!(output.resolved.checked_terminal_goals().is_empty());
}

#[test]
fn task258b3n_validation_precedence_rows_and_replay_fail_closed() {
    let (ast, module, _, symbols) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B3N_TEXT, 258_701);
    let snapshot = mizar_session::BuildSnapshotId::from_published_schema_str(&format!(
        "mizar-session-build-snapshot-v1:{}",
        "e7".repeat(32)
    ))
    .expect("foreign snapshot");
    let allocator = mizar_session::InMemorySessionIdAllocator::new();
    let mut foreign_source =
        mizar_session::SessionIdAllocator::next_source_id(&allocator, snapshot)
            .expect("foreign source");
    while foreign_source == ast.source_id {
        foreign_source =
            mizar_session::SessionIdAllocator::next_source_id(&allocator, snapshot)
                .expect("distinct foreign source");
    }
    let foreign_module =
        ResolverModuleId::new(PackageId::new("pkg"), ModulePath::new("statement.foreign"));
    let baseline = source_statement_output_with_source(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B3N_TEXT,
    )
    .expect("baseline selector")
    .expect("baseline");
    let baseline_debug = baseline.typed_ast.debug_text();
    let expected = [
        "aggregate",
        "witness 0",
        "name 0",
        "name 0",
        "dependency",
        "dependency",
    ];
    for (mutation, expected_fragment) in expected.into_iter().enumerate() {
        let error = source_statement_b3n_output_with_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3N_TEXT,
            |input: &mut SourceStatementB3NRouteInputs| match mutation {
                0 => input.witness.names.clear(),
                1 => {
                    input.witness.witnesses[0].kind =
                        mizar_checker::source_statement::SourceStatementWitnessKind::Unnamed
                }
                2 => {
                    input.witness.names[0].witness =
                        mizar_checker::source_statement::SourceStatementWitnessId::new(1)
                }
                3 => input.witness.names[0].spelling = "z".to_owned(),
                4 => {
                    input.witness.source_id = foreign_source;
                    input.witness.names.clear();
                }
                5 => input.witness.module_id = foreign_module.clone(),
                _ => unreachable!(),
            },
        )
        .expect("mutation selector")
        .expect_err("mutation must fail");
        assert!(
            error.to_ascii_lowercase().contains(expected_fragment),
            "mutation {mutation}: {error}"
        );
        let replay = source_statement_output_with_source(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3N_TEXT,
        )
        .expect("replay selector")
        .expect("replay");
        assert_eq!(replay.typed_ast.debug_text(), baseline_debug);
    }
    for mutation in 0..24 {
        let error = source_statement_b3n_output_with_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3N_TEXT,
            |input: &mut SourceStatementB3NRouteInputs| match mutation {
                0 => input.witness.witnesses.clear(),
                1 => input
                    .witness
                    .witnesses
                    .push(input.witness.witnesses[0].clone()),
                2 => input.witness.names.push(input.witness.names[0].clone()),
                3 => {
                    input.witness.witnesses[0].owner =
                        mizar_checker::source_statement::SourceTheoremOwnerId::new(1)
                }
                4 => {
                    input.witness.witnesses[0].binding_context =
                        mizar_checker::binding_env::BindingContextId::new(0)
                }
                5 => {
                    input.witness.witnesses[0].term =
                        mizar_checker::source_statement::SourceStatementWitnessTermTarget::Primary(
                            mizar_checker::source_term::SourcePrimaryTermId::new(1),
                        )
                }
                6 => input.witness.witnesses[0].take_site = input.witness.witnesses[0].site.clone(),
                7 => input.witness.witnesses[0].take_range.start += 1,
                8 => input.witness.witnesses[0].take_range.end -= 1,
                9 => {
                    input.witness.witnesses[0].site =
                        input.witness.witnesses[0].take_site.clone()
                }
                10 => input.witness.witnesses[0].source_range.start += 1,
                11 => input.witness.witnesses[0].source_range.end -= 1,
                12 => input.witness.witnesses[0].source_ordinal = 2,
                13 => input.witness.witnesses[0].ordinal = 1,
                14 => input.witness.witnesses[0].spelling.push('x'),
                15 => {
                    input.witness.witnesses[0].recovery =
                        mizar_checker::source_statement::SourceStatementRecovery::Degraded
                }
                16 => input.witness.witnesses[0].name = None,
                17 => {
                    input.witness.witnesses[0].name =
                        Some(mizar_checker::source_statement::SourceStatementWitnessNameId::new(1))
                }
                18 => input.witness.names[0].site = input.witness.witnesses[0].site.clone(),
                19 => input.witness.names[0].source_range.start += 1,
                20 => input.witness.names[0].source_range.end -= 1,
                21 => input.witness.names[0].spelling.push('x'),
                22 => {
                    input.witness.names[0].recovery =
                        mizar_checker::source_statement::SourceStatementRecovery::Degraded
                }
                23 => {
                    input.witness.witnesses[0].kind =
                        mizar_checker::source_statement::SourceStatementWitnessKind::Unnamed;
                    input.witness.names[0].spelling.push('x');
                }
                _ => unreachable!(),
            },
        )
        .expect("row selector")
        .expect_err("row mutation");
        let expected = if mutation <= 2 {
            "aggregate"
        } else if mutation <= 17 || mutation == 23 {
            "witness"
        } else {
            "name"
        };
        assert!(
            error.to_ascii_lowercase().contains(expected),
            "row {mutation}: {error}"
        );
    }

    for mutation in 0..26 {
        let error = source_statement_b3n_output_with_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3N_TEXT,
            |input: &mut SourceStatementB3NRouteInputs| match mutation {
                0 => input.statement.owners.clear(),
                1 => input.statement.owners.push(input.statement.owners[0].clone()),
                2 => {
                    input.statement.statements.pop();
                }
                3 => input
                    .statement
                    .statements
                    .push(input.statement.statements[1].clone()),
                4 => {
                    input.statement.contexts.pop();
                }
                5 => input
                    .statement
                    .contexts
                    .push(input.statement.contexts[1].clone()),
                6 => {
                    input.statement.input_facts.pop();
                }
                7 => input
                    .statement
                    .input_facts
                    .push(input.statement.input_facts[1].clone()),
                8 => {
                    input.statement.candidate_facts.pop();
                }
                9 => input
                    .statement
                    .candidate_facts
                    .push(input.statement.candidate_facts[1].clone()),
                10 => input.statement.owners[0].site = input.statement.statements[1].site.clone(),
                11 => input.statement.owners[0].source_range.start += 1,
                12 => input.statement.owners[0].source_range.end -= 1,
                13 => input.statement.owners[0].spelling.push('x'),
                14 => {
                    input.statement.owners[0].recovery =
                        mizar_checker::source_statement::SourceStatementRecovery::Degraded
                }
                15 => {
                    input.statement.statements[0].owner =
                        mizar_checker::source_statement::SourceTheoremOwnerId::new(1)
                }
                16 => {
                    input.statement.statements[1].context =
                        mizar_checker::source_statement::SourceStatementContextId::new(0)
                }
                17 => input.statement.statements[1].source_range.start += 1,
                18 => input.statement.statements[1].source_ordinal = 1,
                19 => input.statement.statements[1].spelling.push('x'),
                20 => {
                    input.statement.contexts[1].binding_context =
                        mizar_checker::binding_env::BindingContextId::new(0)
                }
                21 => input.statement.contexts[1].visible_bindings.clear(),
                22 => input.statement.input_facts[1].uses.swap(0, 1),
                23 => input.statement.input_facts[1].ordinal = 1,
                24 => input.statement.candidate_facts[1].ordinal = 1,
                25 => {
                    input.statement.candidate_facts[1].formula =
                        mizar_checker::source_statement::SourceStatementFormulaTarget::Atomic(
                            mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(0),
                        )
                }
                _ => unreachable!(),
            },
        )
        .expect("base selector")
        .expect_err("base mutation");
        assert!(!error.is_empty(), "base {mutation}");
    }
    for mutation in 0..6 {
        let error = source_statement_b3n_output_with_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3N_TEXT,
            |input| match mutation {
                0 => input.statement.source_id = foreign_source,
                1 => input.statement.module_id = foreign_module.clone(),
                2 => {
                    input.statement.owners[0].site =
                        input.statement.statements[1].site.clone()
                }
                3 => input.statement.owners[0].source_range.start += 1,
                4 => input.statement.owners[0].source_range.end -= 1,
                5 => input.statement.owners[0].spelling.push('x'),
                _ => unreachable!(),
            },
        )
        .expect("owner selector")
        .expect_err("owner mutation");
        assert!(!error.is_empty(), "owner {mutation}");
    }
    for index in 0..2 {
        let other = 1 - index;
        for mutation in 0..10 {
            let error = source_statement_b3n_output_with_mutation(
                &ast,
                module.clone(),
                &symbols,
                SOURCE_STATEMENT_B3N_TEXT,
                |input| {
                    let other_site = input.statement.statements[other].site.clone();
                    let row = &mut input.statement.statements[index];
                    match mutation {
                        0 => {
                            row.owner =
                                mizar_checker::source_statement::SourceTheoremOwnerId::new(1)
                        }
                        1 => {
                            row.context =
                                mizar_checker::source_statement::SourceStatementContextId::new(
                                    other,
                                )
                        }
                        2 => {
                            row.formula = mizar_checker::source_statement::
                                SourceStatementFormulaTarget::Atomic(
                                    mizar_checker::source_atomic_formula::
                                        SourceAtomicFormulaId::new(other),
                                )
                        }
                        3 => row.site = other_site,
                        4 => row.source_range.start += 1,
                        5 => row.source_range.end -= 1,
                        6 => row.source_ordinal = 1,
                        7 => row.spelling.push('x'),
                        8 => {
                            row.recovery = mizar_checker::source_statement::
                                SourceStatementRecovery::Degraded
                        }
                        9 => {
                            row.kind = if index == 0 {
                                mizar_checker::source_statement::SourceStatementKind::Conclusion
                            } else {
                                mizar_checker::source_statement::SourceStatementKind::
                                    TheoremProposition
                            }
                        }
                        _ => unreachable!(),
                    }
                },
            )
            .expect("statement row selector")
            .expect_err("statement row mutation");
            assert!(!error.is_empty(), "statement {index} field {mutation}");
        }
        for mutation in 0..5 {
            let error = source_statement_b3n_output_with_mutation(
                &ast,
                module.clone(),
                &symbols,
                SOURCE_STATEMENT_B3N_TEXT,
                |input| {
                    let row = &mut input.statement.contexts[index];
                    match mutation {
                        0 => {
                            row.statement =
                                mizar_checker::source_statement::SourceStatementId::new(other)
                        }
                        1 => {
                            row.binding_context =
                                mizar_checker::binding_env::BindingContextId::new(other)
                        }
                        2 => row.source_range.start += 1,
                        3 => row.source_range.end -= 1,
                        4 => row.visible_bindings.clear(),
                        _ => unreachable!(),
                    }
                },
            )
            .expect("context row selector")
            .expect_err("context row mutation");
            assert!(!error.is_empty(), "context {index} field {mutation}");
        }
        for mutation in 0..5 {
            let error = source_statement_b3n_output_with_mutation(
                &ast,
                module.clone(),
                &symbols,
                SOURCE_STATEMENT_B3N_TEXT,
                |input| {
                    let row = &mut input.statement.input_facts[index];
                    match mutation {
                        0 => {
                            row.statement =
                                mizar_checker::source_statement::SourceStatementId::new(other)
                        }
                        1 => {
                            row.context =
                                mizar_checker::source_statement::SourceStatementContextId::new(
                                    other,
                                )
                        }
                        2 => row.ordinal = 1,
                        3 => {
                            row.binding = mizar_checker::binding_env::BindingId::new(1)
                        }
                        4 => row.uses.swap(0, 1),
                        _ => unreachable!(),
                    }
                },
            )
            .expect("input-fact row selector")
            .expect_err("input-fact row mutation");
            assert!(!error.is_empty(), "input fact {index} field {mutation}");
        }
        for mutation in 0..4 {
            let error = source_statement_b3n_output_with_mutation(
                &ast,
                module.clone(),
                &symbols,
                SOURCE_STATEMENT_B3N_TEXT,
                |input| {
                    let row = &mut input.statement.candidate_facts[index];
                    match mutation {
                        0 => {
                            row.statement =
                                mizar_checker::source_statement::SourceStatementId::new(other)
                        }
                        1 => {
                            row.context =
                                mizar_checker::source_statement::SourceStatementContextId::new(
                                    other,
                                )
                        }
                        2 => row.ordinal = 1,
                        3 => {
                            row.formula = mizar_checker::source_statement::
                                SourceStatementFormulaTarget::Atomic(
                                    mizar_checker::source_atomic_formula::
                                        SourceAtomicFormulaId::new(other),
                                )
                        }
                        _ => unreachable!(),
                    }
                },
            )
            .expect("candidate row selector")
            .expect_err("candidate row mutation");
            assert!(!error.is_empty(), "candidate {index} field {mutation}");
        }
    }

    let (b3_ast, b3_module, _, b3_symbols) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B3_TEXT, 258_702);
    let b3 = source_statement_output_with_source(
        &b3_ast,
        b3_module,
        &b3_symbols,
        SOURCE_STATEMENT_B3_TEXT,
    )
    .expect("B3 selector")
    .expect("B3");
    let wrong_binding = b3
        .typed_ast
        .source_statement()
        .expect("B3 statement")
        .binding_env()
        .clone();
    let wrong_primary = b3.typed_ast.source_term().expect("B3 primary").clone();
    let wrong_atomic = b3
        .typed_ast
        .source_atomic_formula()
        .expect("B3 atomic")
        .clone();
    let wrong_owner_symbol = b3
        .typed_ast
        .source_statement()
        .expect("B3 statement")
        .owners()
        .get(mizar_checker::source_statement::SourceTheoremOwnerId::new(0))
        .expect("B3 owner")
        .symbol()
        .clone();
    let owner_error = source_statement_b3n_output_with_mutation(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B3N_TEXT,
        |input| input.statement.owners[0].symbol = wrong_owner_symbol,
    )
    .expect("owner symbol selector")
    .expect_err("foreign owner symbol");
    assert!(!owner_error.is_empty(), "{owner_error}");
    for mutation in 0..3 {
        let error = source_statement_b3n_output_with_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3N_TEXT,
            |input| match mutation {
                0 => input.binding_env = wrong_binding.clone(),
                1 => input.primary = wrong_primary.clone(),
                2 => input.atomic = wrong_atomic.clone(),
                _ => unreachable!(),
            },
        )
        .expect("lower selector")
        .expect_err("lower mutation");
        assert!(
            error.to_ascii_lowercase().contains("dependency"),
            "lower {mutation}: {error}"
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
        let error = source_statement_b3n_output_with_resolver_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3N_TEXT,
            |symbols| task258b2_mutate_resolver(symbols, mutation),
        )
        .expect("resolver selector")
        .expect_err("resolver mutation");
        assert!(
            error.to_ascii_lowercase().contains("resolver")
                || error.to_ascii_lowercase().contains("owner")
                || error.to_ascii_lowercase().contains("label"),
            "{mutation:?}: {error}"
        );
    }
    let replay = source_statement_output_with_source(
        &ast,
        module,
        &symbols,
        SOURCE_STATEMENT_B3N_TEXT,
    )
    .expect("final replay selector")
    .expect("final replay");
    assert_eq!(replay.typed_ast.debug_text(), baseline_debug);
    assert_task258b3n_all_51_nodes_are_authenticated_before_publication();
}

fn assert_task258b3n_all_51_nodes_are_authenticated_before_publication() {
    let (ast, module, _, symbols) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B3N_TEXT, 258_703);
    let baseline = source_statement_output_with_source(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B3N_TEXT,
    )
    .expect("baseline selector")
    .expect("baseline");
    let baseline_debug = baseline.typed_ast.debug_text();
    for index in 0..51 {
        for mutation in 0..5 {
            let error = source_statement_b3n_output_with_mutation(
                &ast,
                module.clone(),
                &symbols,
                SOURCE_STATEMENT_B3N_TEXT,
                |input: &mut SourceStatementB3NRouteInputs| {
                    input.arena = mizar_checker::typed_ast::TypedArena::try_new(
                        input.arena.root(),
                        input
                            .arena
                            .iter()
                            .map(|(id, row)| {
                                let mut row = row.clone();
                                if id.index() == index {
                                    match mutation {
                                        0 => {
                                            let mizar_session::SourceAnchor::Range(mut range) =
                                                row.anchor.clone()
                                            else {
                                                unreachable!("Task258B3N range anchor")
                                            };
                                            range.end += 1;
                                            row.anchor =
                                                mizar_session::SourceAnchor::Range(range);
                                        }
                                        1 => {
                                            row.recovery = mizar_checker::typed_ast::
                                                NodeRecoveryState::Recovered
                                        }
                                        2 => {
                                            row.recovery = mizar_checker::typed_ast::
                                                NodeRecoveryState::Degraded
                                        }
                                        3 => row.kind = "source.task258b3n.mutated".into(),
                                        4 if row.children.len() > 1 => row.children.swap(0, 1),
                                        4 if row.children.len() == 1 => row.children.clear(),
                                        4 => row.children.push(
                                            mizar_checker::typed_ast::TypedNodeId::new(
                                                usize::from(index == 0),
                                            ),
                                        ),
                                        _ => unreachable!(),
                                    }
                                }
                                row
                            })
                            .collect(),
                    )
                    .expect("structurally valid mutation");
                },
            )
            .expect("selector")
            .expect_err("node mutation must fail");
            assert!(!error.is_empty(), "node {index} mutation {mutation}");
        }
        let replay = source_statement_output_with_source(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3N_TEXT,
        )
        .expect("replay selector")
        .expect("replay");
        assert_eq!(replay.typed_ast.debug_text(), baseline_debug, "node {index}");
    }
}

#[test]
fn task258b3n_selector_and_byte_subtree_near_misses_are_exact() {
    let (ast, module, _, symbols) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B3N_TEXT, 258_704);
    let resolver_replay = source_statement_b3n_output_with_resolver_mutation(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B3N_TEXT,
        |symbols| symbols,
    )
    .expect("resolver selector")
    .expect("resolver replay");
    assert_eq!(resolver_replay.typed_ast.nodes().len(), 51);
    for (ordinal, source) in [
        SOURCE_STATEMENT_B3N_TEXT.trim_end_matches('\n').to_owned(),
        format!("{SOURCE_STATEMENT_B3N_TEXT}\n"),
        SOURCE_STATEMENT_B3N_TEXT.replacen("  take y = x;", "   take y = x;", 1),
        SOURCE_STATEMENT_B3N_TEXT.replacen(
            "FormulaStatementNamedWitnessSmoke",
            "FormulaStatementNamedWitnessNearMiss",
            1,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        assert!(
            source_statement_output_with_source(
                &ast,
                module.clone(),
                &symbols,
                &source,
            )
            .is_none(),
            "byte near miss {ordinal}"
        );
    }
    let reordered = SOURCE_STATEMENT_B3N_TEXT.replacen(
        "  take y = x;\n  thus x = x;\n",
        "  thus x = x;\n  take y = x;\n",
        1,
    );
    for (ordinal, source) in [
        SOURCE_STATEMENT_B3_TEXT.to_owned(),
        SOURCE_STATEMENT_B3N_TEXT.replacen("take y = x;", "take z = x;", 1),
        SOURCE_STATEMENT_B3N_TEXT.replacen("take y = x;", "take = x;", 1),
        SOURCE_STATEMENT_B3N_TEXT.replacen("take y = x;", "take y x;", 1),
        SOURCE_STATEMENT_B3N_TEXT.replacen("take y = x;", "take y = x, x;", 1),
        SOURCE_STATEMENT_B3N_TEXT.replacen("take y = x;", "take y = {x};", 1),
        SOURCE_STATEMENT_B3N_TEXT.replacen("  take y = x;\n", "", 1),
        SOURCE_STATEMENT_B3N_TEXT.replacen(
            "  take y = x;\n",
            "  take y = x;\n  take z = x;\n",
            1,
        ),
        SOURCE_STATEMENT_B3N_TEXT.replacen(
            "  thus x = x;\n",
            "  thus x = x;\n  thus x = x;\n",
            1,
        ),
        reordered,
        SOURCE_STATEMENT_B3N_TEXT.replacen("take y = x;", "take y = x", 1),
        SOURCE_STATEMENT_B3N_TEXT.replacen(
            ": x = x proof",
            ": ex z being set st z = z proof",
            1,
        ),
        SOURCE_STATEMENT_B3N_TEXT.replacen(
            ": x = x proof",
            ": (x = x) & (x = x) proof",
            1,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let (near_ast, near_module, _, near_symbols) =
            task253_ast_from_source_text(&source, 258_710 + ordinal);
        assert!(
            extract_named_witness_source_statement(&near_ast, SOURCE_STATEMENT_B3N_TEXT).is_none(),
            "subtree extraction near miss {ordinal}"
        );
        assert!(
            source_statement_output_with_source(
                &near_ast,
                near_module,
                &near_symbols,
                SOURCE_STATEMENT_B3N_TEXT,
            )
                .is_none(),
            "subtree route near miss {ordinal}"
        );
    }
}

#[test]
fn task258b3n_family_and_active_route_isolation_is_atomic_in_both_orders() {
    let (ast, module, _, symbols) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B3N_TEXT, 258_704);
    let output = source_statement_output_with_source(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B3N_TEXT,
    )
    .expect("B3N selector")
    .expect("B3N route");
    let b3n_statement = output
        .typed_ast
        .source_statement()
        .expect("B3N statement")
        .clone();
    let b3n_witnesses = output
        .typed_ast
        .source_statement_witnesses()
        .expect("B3N witnesses")
        .clone();
    let baseline_debug = output.typed_ast.debug_text();

    let (b3_ast, b3_module, _, b3_symbols) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B3_TEXT, 258_720);
    let b3 = source_statement_output_with_source(
        &b3_ast,
        b3_module,
        &b3_symbols,
        SOURCE_STATEMENT_B3_TEXT,
    )
    .expect("B3 selector")
    .expect("B3");
    let b3_witness = b3
        .typed_ast
        .source_statement_witnesses()
        .expect("B3 witness");
    assert!(b3_witness.names().is_empty());
    assert_eq!(
        b3_witness.debug_text(),
        format!(
            "source-statement-witness-debug-v1\nmodule: {}::{}\nstatement-fingerprint: {:?}\nprimary-term-fingerprint: {:?}\nwitness#0 owner=0 binding_context=1 term=primary#2 take_range=77..84 take_site=35 range=82..83 site=34 source_ordinal=1 ordinal=0 kind=unnamed recovery=normal spelling=\"x\"\n",
            b3_witness.module_id().package().as_str(),
            b3_witness.module_id().path().as_str(),
            b3.typed_ast
                .source_statement()
                .expect("B3 statement")
                .debug_text(),
            b3.typed_ast
                .source_term()
                .expect("B3 primary")
                .debug_text(),
        )
    );
    for (ordinal, source) in [
        SOURCE_STATEMENT_TEXT,
        SOURCE_STATEMENT_B1_TEXT,
        SOURCE_STATEMENT_B2_TEXT,
        SOURCE_STATEMENT_B3_TEXT,
    ]
    .into_iter()
    .enumerate()
    {
        let (foreign_ast, foreign_module, _, foreign_symbols) =
            task253_ast_from_source_text(source, 258_722 + ordinal);
        let foreign = source_statement_output_with_source(
            &foreign_ast,
            foreign_module,
            &foreign_symbols,
            source,
        )
        .expect("foreign selector")
        .expect("foreign route");
        let foreign_debug = foreign.typed_ast.debug_text();
        assert_eq!(
            foreign
                .typed_ast
                .clone()
                .with_source_statement_witnesses(
                    b3n_statement.clone(),
                    b3n_witnesses.clone(),
                ),
            Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement),
            "foreign family {ordinal}"
        );
        assert_eq!(foreign.typed_ast.debug_text(), foreign_debug);
        if let Some(references) = foreign.typed_ast.source_statement_references() {
            assert_eq!(
                output.typed_ast.clone().with_source_statement_references(
                    foreign
                        .typed_ast
                        .source_statement()
                        .expect("foreign statement")
                        .clone(),
                    references.clone(),
                ),
                Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement),
                "B3N receiver family {ordinal}"
            );
        } else if let Some(foreign_witnesses) =
            foreign.typed_ast.source_statement_witnesses()
        {
            assert_eq!(
                output.typed_ast.clone().with_source_statement_witnesses(
                    foreign
                        .typed_ast
                        .source_statement()
                        .expect("foreign statement")
                        .clone(),
                    foreign_witnesses.clone(),
                ),
                Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement),
                "B3N receiver family {ordinal}"
            );
        } else {
            assert_eq!(
                output.typed_ast.clone().with_source_statement(
                    foreign
                        .typed_ast
                        .source_statement()
                        .expect("foreign statement")
                        .clone(),
                ),
                Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement),
                "B3N receiver family {ordinal}"
            );
        }
        let foreign_binding = foreign
            .typed_ast
            .source_statement()
            .expect("foreign statement")
            .binding_env()
            .clone();
        let foreign_primary = foreign
            .typed_ast
            .source_term()
            .expect("foreign primary")
            .clone();
        let foreign_atomic = foreign
            .typed_ast
            .source_atomic_formula()
            .expect("foreign atomic")
            .clone();
        let error = source_statement_b3n_output_with_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3N_TEXT,
            move |input| {
                input.binding_env = foreign_binding;
                input.primary = foreign_primary;
                input.atomic = foreign_atomic;
            },
        )
        .expect("cross-family selector")
        .expect_err("cross-family lower tuple must fail");
        assert!(
            error.to_ascii_lowercase().contains("dependency"),
            "foreign family {ordinal}: {error}"
        );
        assert_eq!(output.typed_ast.debug_text(), baseline_debug);
    }

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
    let plan = build_test_plan(&config).expect("Task258B3N isolation plan");
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
        let extracted = extract_named_witness_source_statement(&active_ast, &source).is_some();
        let selected_route = source_statement_output_with_source(
            &active_ast,
            resolver.module.clone(),
            &active_symbols,
            &source,
        )
        .is_some_and(|result| {
            result.is_ok_and(|output| {
                output
                    .typed_ast
                    .source_statement_witnesses()
                    .is_some_and(|witnesses| !witnesses.names().is_empty())
            })
        });
        if extracted || selected_route {
            selected.push(case.id.0.clone());
        }
    }
    assert!(
        selected.is_empty(),
        "Task258B3N selected active cases: {selected:?}"
    );
}

#[test]
fn task258b3n_typed_final_clone_debug_rollback_and_empty_semantics_are_stable() {
    let (ast, module, _, symbols) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B3N_TEXT, 258_721);
    let output = source_statement_output_with_source(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B3N_TEXT,
    )
    .expect("selector")
    .expect("route");
    assert_eq!(output.typed_ast.clone().debug_text(), output.typed_ast.debug_text());
    assert_eq!(output.resolved.clone().debug_text(), output.resolved.debug_text());
    assert_eq!(
        output.typed_ast.source_statement(),
        output.resolved.source_statement()
    );
    assert_eq!(
        output.typed_ast.source_statement_witnesses(),
        output.resolved.source_statement_witnesses()
    );
    assert!(output.typed_ast.source_statement_references().is_none());
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
    let baseline_debug = output.typed_ast.debug_text();
    let error = source_statement_b3n_output_with_mutation(
        &ast,
        module,
        &symbols,
        SOURCE_STATEMENT_B3N_TEXT,
        |input| input.witness.names[0].spelling.push('z'),
    )
    .expect("rollback selector")
    .expect_err("invalid name must roll back");
    assert!(error.to_ascii_lowercase().contains("name 0"), "{error}");
    assert_eq!(output.typed_ast.debug_text(), baseline_debug);
}

const TASK258A_SOURCE: &str = concat!(
    "reserve x for set;\n",
    "theorem FormulaStatementReservedVariableEqualitySmoke: x = x;\n",
);
const TASK258A_LABEL: &str = "FormulaStatementReservedVariableEqualitySmoke";
const TASK258A_STATEMENT_SPELLING: &str =
    "theorem FormulaStatementReservedVariableEqualitySmoke : x = x ;";
const TASK258A_LOWER_SOURCE: &str = concat!(
    "import parser.type_fixtures;\n",
    "theorem FormulaPredicateChainPayloadBoundary: ",
    "1 divides 2 does not divides 3;\n",
);

struct Task258B3RejectContext<'a> {
    ast: &'a mizar_syntax::ast::SurfaceAst,
    module: &'a mizar_resolve::resolved_ast::ModuleId,
    symbols: &'a mizar_resolve::env::SymbolEnv,
    baseline_debug: &'a str,
    baseline_resolved: &'a str,
}

impl Task258B3RejectContext<'_> {
    #[inline(never)]
    fn assert_rejects(
        &self,
        label: &str,
        fragment: &str,
        mutate: &dyn Fn(&mut SourceStatementB3RouteInputs),
    ) {
        let error = source_statement_b3_output_with_mutation(
            self.ast,
            self.module.clone(),
            self.symbols,
            SOURCE_STATEMENT_B3_TEXT,
            |input| mutate(input),
        )
        .unwrap_or_else(|| panic!("{label} selector"))
        .expect_err(label);
        assert!(
            error.to_ascii_lowercase().contains(fragment),
            "{label}: {error}"
        );
        let replay = source_statement_output_with_source(
            self.ast,
            self.module.clone(),
            self.symbols,
            SOURCE_STATEMENT_B3_TEXT,
        )
        .expect("replay selector")
        .expect("replay");
        assert_eq!(
            replay.typed_ast.debug_text(),
            self.baseline_debug,
            "{label}"
        );
        assert_eq!(
            replay.resolved.debug_text(),
            self.baseline_resolved,
            "{label}"
        );
    }
}

#[test]
fn task258b3_real_frontend_freezes_single_witness_transport() {
    assert_eq!(SOURCE_STATEMENT_B3_TEXT.len(), 104);
    assert!(SOURCE_STATEMENT_B3_TEXT.ends_with('\n'));
    let (ast, module, _, symbols) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B3_TEXT, 258_600);
    let extracted: SourceStatementB3Extraction =
        extract_single_witness_source_statement(&ast, SOURCE_STATEMENT_B3_TEXT)
            .expect("Task258B3 exact parser shape");
    assert_eq!((ast.nodes().len(), ast.root().expect("root").index()), (49, 48));
    assert_eq!(
        extracted
            .statement_sites
            .iter()
            .map(|site| site.node().index())
            .collect::<Vec<_>>(),
        [45, 43]
    );
    assert_eq!(
        extracted
            .statement_ranges
            .iter()
            .map(|range| (range.start, range.end))
            .collect::<Vec<_>>(),
        [(19, 103), (87, 98)]
    );
    assert_eq!(
        extracted
            .formula_sites
            .iter()
            .map(|site| site.node().index())
            .collect::<Vec<_>>(),
        [30, 40]
    );
    assert_eq!(
        extracted
            .formula_ranges
            .iter()
            .map(|range| (range.start, range.end))
            .collect::<Vec<_>>(),
        [(63, 68), (92, 97)]
    );
    assert_eq!(
        extracted
            .term_sites
            .iter()
            .map(|site| site.node().index())
            .collect::<Vec<_>>(),
        [26, 28, 32, 36, 38]
    );
    assert_eq!(
        extracted
            .term_ranges
            .iter()
            .map(|range| (range.start, range.end))
            .collect::<Vec<_>>(),
        [(63, 64), (67, 68), (82, 83), (92, 93), (96, 97)]
    );
    assert_eq!(
        (
            extracted.take_site.node().index(),
            extracted.take_range.start,
            extracted.take_range.end,
            extracted.witness_site.node().index(),
            extracted.witness_range.start,
            extracted.witness_range.end,
            extracted.proof_range.start,
            extracted.proof_range.end,
        ),
        (35, 77, 84, 34, 82, 83, 69, 102)
    );
    assert_eq!(
        (
            extracted.theorem_site.node().index(),
            extracted.theorem_range.start,
            extracted.theorem_range.end,
            extracted.label_range.start,
            extracted.label_range.end,
        ),
        (45, 19, 103, 27, 61)
    );
    let resolver =
        source_statement_b3_resolver_env_for_test(&module, &symbols, &extracted)
            .expect("Task258B3 resolver projection");

    let output = source_statement_output_with_source(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B3_TEXT,
    )
            .expect("Task258B3 selector")
            .unwrap_or_else(|error| panic!("Task258B3 route failed: {error}"));
    let statement = output
        .typed_ast
        .source_statement()
        .expect("Task258B3 base statement");
    let witnesses = output
        .typed_ast
        .source_statement_witnesses()
        .expect("Task258B3 witness handoff");
    assert!(output.typed_ast.source_statement_references().is_none());
    assert_eq!(
        (
            statement.owners().len(),
            statement.statements().len(),
            statement.contexts().len(),
            statement.input_facts().len(),
            statement.candidate_facts().len(),
            witnesses.witnesses().len(),
        ),
        (1, 2, 2, 2, 2, 1)
    );
    let owner = statement
        .owners()
        .get(mizar_checker::source_statement::SourceTheoremOwnerId::new(0))
        .expect("Task258B3 owner");
    assert_eq!(owner.contribution().index(), 0);
    assert_eq!(
        (owner.source_range().start, owner.source_range().end),
        (19, 103)
    );
    assert_eq!(statement.checked_owner().origin().structural_path(), [2, 1]);
    let resolver_owner = resolver.symbols().get(owner.symbol()).expect("owner symbol");
    assert_eq!(resolver_owner.kind(), mizar_resolve::env::SymbolKind::Theorem);
    assert_eq!(
        resolver_owner.visibility(),
        mizar_resolve::env::Visibility::Public
    );
    assert_eq!(
        resolver_owner.export_status(),
        mizar_resolve::env::ExportStatus::Exported
    );
    assert_eq!(resolver_owner.contribution(), owner.contribution());
    let definition = resolver
        .definitions()
        .by_symbol(owner.symbol())
        .expect("theorem definition");
    assert_eq!(
        definition.kind(),
        mizar_resolve::env::DefinitionKind::Theorem
    );
    assert_eq!(definition.contribution(), owner.contribution());
    let contribution = resolver
        .contributions()
        .get(owner.contribution())
        .expect("local contribution");
    assert!(matches!(
        contribution.kind(),
        mizar_resolve::env::ContributionKind::LocalSource { .. }
    ));
    assert!(contribution.effects().symbols().contains(owner.symbol()));
    assert!(
        contribution
            .effects()
            .definitions()
            .contains(&definition.id())
    );
    assert!(contribution.effects().imports().is_empty());
    let labels = resolver.labels().by_contribution(owner.contribution());
    assert_eq!(labels.len(), 1);
    assert_eq!(
        labels[0].primary_spelling(),
        "FormulaStatementSingleWitnessSmoke"
    );
    assert_eq!(
        labels[0].kind(),
        mizar_resolve::resolved_ast::LabelKind::Theorem
    );
    assert_eq!(
        labels[0].visibility(),
        mizar_resolve::env::Visibility::Public
    );
    assert_eq!(
        labels[0].export_status(),
        mizar_resolve::env::ExportStatus::Exported
    );
    assert_eq!(
        labels[0].origin_path().as_str(),
        format!(
            "{}::{}::theorem::FormulaStatementSingleWitnessSmoke",
            module.package().as_str(),
            module.path().as_str(),
        )
    );
    assert!(resolver.imports().is_empty());
    let proof_context = statement
        .binding_env()
        .contexts()
        .get(mizar_checker::binding_env::BindingContextId::new(1))
        .expect("proof context");
    assert_eq!(
        proof_context
            .lexical_scope
            .as_ref()
            .expect("proof scope")
            .path(),
        [0]
    );
    assert_eq!(
        proof_context.visible_bindings,
        [mizar_checker::binding_env::BindingId::new(0)]
    );
    let primary = output
        .typed_ast
        .source_term()
        .expect("Task252 handoff");
    assert_eq!(
        (
            primary.terms().len(),
            primary.references().len(),
            primary.numeric_type_requests().len(),
        ),
        (5, 5, 0)
    );
    for (index, ((_, term), (_, reference))) in primary
        .terms()
        .iter()
        .zip(primary.references().iter())
        .enumerate()
    {
        assert_eq!(term.site().node().index(), [26, 28, 32, 36, 38][index]);
        assert_eq!(
            (term.source_range().start, term.source_range().end),
            [(63, 64), (67, 68), (82, 83), (92, 93), (96, 97)][index]
        );
        assert_eq!(term.source_ordinal(), index);
        assert_eq!(term.context().index(), [0, 0, 1, 1, 1][index]);
        assert_eq!(term.spelling(), "x");
        assert_eq!(
            term.kind(),
            mizar_checker::source_term::SourcePrimaryTermKind::VariableReference
        );
        assert_eq!(
            term.role(),
            mizar_checker::source_term::SourcePrimaryTermRole::Value
        );
        assert_eq!(
            term.recovery(),
            mizar_checker::source_term::SourcePrimaryTermRecovery::Normal
        );
        assert!(term.parent().is_none());
        assert_eq!(reference.term().index(), index);
        assert_eq!(reference.binding().index(), 0);
        assert_eq!(
            reference.role(),
            mizar_checker::source_term::SourcePrimaryTermReferenceRole::Variable
        );
        assert_eq!(reference.use_ordinal(), 1);
        if index < 2 {
            assert!(reference.lexical_scope().is_none());
        } else {
            assert_eq!(
                reference.lexical_scope().expect("reference scope").path(),
                [0]
            );
        }
    }
    let atomic = output
        .typed_ast
        .source_atomic_formula()
        .expect("Task256 handoff");
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
    for index in 0..2 {
        let formula = atomic
            .formulas()
            .get(mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(index))
            .expect("atomic formula");
        assert_eq!(formula.site().node().index(), [30, 40][index]);
        assert_eq!(
            (formula.source_range().start, formula.source_range().end),
            [(63, 68), (92, 97)][index]
        );
        assert_eq!(formula.source_ordinal(), index);
        assert_eq!(formula.context().index(), index);
        assert_eq!(formula.spelling(), "x = x");
        assert_eq!(
            formula.kind(),
            mizar_checker::source_atomic_formula::SourceAtomicFormulaKind::Equality
        );
        assert_eq!(
            formula.recovery(),
            mizar_checker::source_atomic_formula::SourceAtomicFormulaRecovery::Normal
        );
    }
    assert_eq!(
        atomic
            .edges()
            .iter()
            .map(|(_, edge)| match edge.target() {
                mizar_checker::source_atomic_formula::SourceAtomicTermTarget::Primary(term) => {
                    term.index()
                }
                _ => usize::MAX,
            })
            .collect::<Vec<_>>(),
        [0, 1, 3, 4]
    );
    assert!(
        atomic.edges().iter().all(|(_, edge)| !matches!(
            edge.target(),
            mizar_checker::source_atomic_formula::SourceAtomicTermTarget::Primary(term)
                if term.index() == 2
        ))
    );
    for index in 0..4 {
        let edge = atomic
            .edges()
            .get(mizar_checker::source_atomic_formula::SourceAtomicEdgeId::new(index))
            .expect("atomic edge");
        let formula = usize::from(index >= 2);
        let ordinal = index % 2;
        assert_eq!(edge.formula().index(), formula);
        assert_eq!(edge.ordinal(), ordinal);
        assert_eq!(
            edge.role(),
            if ordinal == 0 {
                mizar_checker::source_atomic_formula::SourceAtomicEdgeRole::BuiltinLeftOperand
            } else {
                mizar_checker::source_atomic_formula::SourceAtomicEdgeRole::BuiltinRightOperand
            }
        );
        let request = atomic
            .requests()
            .get(mizar_checker::source_atomic_formula::SourceAtomicRequestId::new(index))
            .expect("atomic request");
        assert_eq!(request.formula().index(), formula);
        assert_eq!(request.ordinal(), ordinal);
        assert_eq!(
            request.kind(),
            mizar_checker::source_atomic_formula::SourceAtomicRequestKind::OperandExpectedType
        );
        assert_eq!(request.edge().map(|id| id.index()), Some(index));
        assert!(request.candidate().is_none());
        assert!(request.type_site().is_none());
        assert!(request.attribute().is_none());
    }
    assert_eq!(owner.site().node().index(), 45);
    assert_eq!(owner.spelling(), "FormulaStatementSingleWitnessSmoke");
    assert_eq!(
        owner.role(),
        mizar_checker::source_statement::SourceTheoremRole::Theorem
    );
    assert_eq!(
        owner.status(),
        mizar_checker::source_statement::SourceTheoremStatus::Unmodified
    );
    assert_eq!(
        owner.recovery(),
        mizar_checker::source_statement::SourceStatementRecovery::Normal
    );
    for index in 0..2 {
        let row = statement
            .statements()
            .get(mizar_checker::source_statement::SourceStatementId::new(index))
            .expect("statement row");
        assert_eq!(row.owner().index(), 0);
        assert_eq!(row.context().index(), index);
        assert_eq!(
            row.formula(),
            mizar_checker::source_statement::SourceStatementFormulaTarget::Atomic(
                mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(index),
            )
        );
        assert_eq!(row.site().node().index(), [45, 43][index]);
        assert_eq!(
            (row.source_range().start, row.source_range().end),
            [(19, 103), (87, 98)][index]
        );
        assert_eq!(row.source_ordinal(), [0, 2][index]);
        assert_eq!(
            row.spelling(),
            [
                "theorem FormulaStatementSingleWitnessSmoke : x = x proof take x ; thus x = x ; end ;",
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
            .expect("statement context");
        assert_eq!(context.statement().index(), index);
        assert_eq!(context.binding_context().index(), index);
        assert_eq!(context.source_range(), row.source_range());
        assert_eq!(
            context
                .visible_bindings()
                .iter()
                .map(|id| id.index())
                .collect::<Vec<_>>(),
            [0]
        );
        let fact = statement
            .input_facts()
            .get(mizar_checker::source_statement::SourceStatementInputFactId::new(index))
            .expect("input fact");
        assert_eq!(fact.statement().index(), index);
        assert_eq!(fact.context().index(), index);
        assert_eq!(fact.ordinal(), 0);
        assert_eq!(
            fact.kind(),
            mizar_checker::source_statement::SourceStatementInputFactKind::ReservedTypeGuard
        );
        assert_eq!(fact.binding().index(), 0);
        assert_eq!(
            fact.uses()
                .iter()
                .map(|id| id.index())
                .collect::<Vec<_>>(),
            if index == 0 { vec![0, 1] } else { vec![3, 4] }
        );
        let candidate = statement
            .candidate_facts()
            .get(mizar_checker::source_statement::SourceStatementCandidateFactId::new(index))
            .expect("candidate");
        assert_eq!(candidate.statement().index(), index);
        assert_eq!(candidate.context().index(), index);
        assert_eq!(candidate.ordinal(), 0);
        assert_eq!(
            candidate.kind(),
            mizar_checker::source_statement::SourceStatementCandidateFactKind::UnverifiedProposition
        );
        assert_eq!(
            candidate.formula(),
            row.formula()
        );
    }
    let witness = witnesses
        .witnesses()
        .get(mizar_checker::source_statement::SourceStatementWitnessId::new(0))
        .expect("witness row");
    assert_eq!(witness.binding_context().index(), 1);
    assert_eq!(
        witness.term(),
        mizar_checker::source_statement::SourceStatementWitnessTermTarget::Primary(
            mizar_checker::source_term::SourcePrimaryTermId::new(2)
        )
    );
    assert_eq!((witness.source_ordinal(), witness.ordinal()), (1, 0));
    assert_eq!(witnesses.statement_fingerprint(), statement.debug_text());
    assert_eq!(witnesses.primary_term_fingerprint(), primary.debug_text());
    assert_eq!(
        statement
            .statements()
            .iter()
            .map(|(_, row)| row.source_ordinal())
            .collect::<Vec<_>>(),
        [0, 2]
    );
    assert_eq!(output.reference_use_ordinals, [1; 5]);
    assert_eq!(
        output.typed_ast.source_statement(),
        output.resolved.source_statement()
    );
    assert_eq!(
        output.typed_ast.source_statement_witnesses(),
        output.resolved.source_statement_witnesses()
    );
    assert_eq!(output.typed_ast.nodes().len(), ast.nodes().len());
    assert_eq!(
        output.typed_ast.nodes().root().map(|root| root.index()),
        ast.root().map(|root| root.index())
    );
    for (id, node) in output.typed_ast.nodes().iter() {
        let surface = &ast.nodes()[id.index()];
        assert_eq!(
            node.anchor,
            mizar_session::SourceAnchor::Range(surface.range),
            "node {} range",
            id.index()
        );
        assert_eq!(
            node.children
                .iter()
                .map(|child| child.index())
                .collect::<Vec<_>>(),
            surface
                .children
                .iter()
                .map(|child| child.index())
                .collect::<Vec<_>>(),
            "node {} children",
            id.index()
        );
        assert_eq!(
            node.recovery,
            mizar_checker::typed_ast::NodeRecoveryState::Normal,
            "node {} recovery",
            id.index()
        );
        assert_eq!(
            node.kind.as_str(),
            match id.index() {
                26 | 28 | 32 | 36 | 38 => "source.term.variable-reference",
                30 | 40 => "source.formula.atomic.equality",
                34 => "source.statement-witness.item",
                35 => "source.statement-witness.take",
                43 => "source.statement.conclusion",
                45 => "source.statement.theorem",
                _ => "source.surface.unowned",
            },
            "node {} kind",
            id.index()
        );
    }
}

#[test]
fn task258b3_dependency_witness_rows_and_replay_fail_closed() {
    let (ast, module, _, symbols) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B3_TEXT, 258_610);
    let baseline = source_statement_output_with_source(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B3_TEXT,
    )
    .expect("selector")
    .expect("baseline");
    let baseline_debug = baseline.typed_ast.debug_text();
    let baseline_resolved = baseline.resolved.debug_text();
    let reject_context = Task258B3RejectContext {
        ast: &ast,
        module: &module,
        symbols: &symbols,
        baseline_debug: &baseline_debug,
        baseline_resolved: &baseline_resolved,
    };
    let snapshot = mizar_session::BuildSnapshotId::from_published_schema_str(&format!(
        "mizar-session-build-snapshot-v1:{}",
        "d9".repeat(32)
    ))
    .expect("foreign snapshot");
    let allocator = mizar_session::InMemorySessionIdAllocator::new();
    let _ = mizar_session::SessionIdAllocator::next_source_id(&allocator, snapshot)
        .expect("first source");
    let foreign_source = mizar_session::SessionIdAllocator::next_source_id(&allocator, snapshot)
        .expect("foreign source");
    let mut foreign_contributions = mizar_resolve::env::SourceContributionIndex::new();
    foreign_contributions.insert(
        module.clone(),
        mizar_resolve::env::ContributionKind::LocalSource {
            source_id: ast.source_id,
        },
        mizar_session::SourceAnchor::Range(mizar_session::SourceRange {
            source_id: ast.source_id,
            start: 0,
            end: 1,
        }),
    );
    let foreign_contribution = foreign_contributions.insert(
        module.clone(),
        mizar_resolve::env::ContributionKind::LocalSource {
            source_id: ast.source_id,
        },
        mizar_session::SourceAnchor::Range(mizar_session::SourceRange {
            source_id: ast.source_id,
            start: 1,
            end: 2,
        }),
    );
    assert_eq!(foreign_contribution.index(), 1);
    macro_rules! assert_rejects {
        ($label:expr, $fragment:expr, $mutate:expr) => {{
            reject_context.assert_rejects($label, $fragment, &$mutate);
        }};
    }

    assert_rejects!("missing witness aggregate", "aggregate", |input| {
        input.witness.witnesses.clear()
    });
    assert_rejects!("duplicate witness aggregate", "aggregate", |input| {
        input
            .witness
            .witnesses
            .push(input.witness.witnesses[0].clone())
    });
    assert_rejects!("wrong witness source", "dependency", |input| {
        input.witness.source_id = foreign_source
    });
    assert_rejects!("wrong witness module", "dependency", |input| {
        input.witness.module_id = mizar_resolve::resolved_ast::ModuleId::new(
            mizar_session::PackageId::new("pkg"),
            mizar_session::ModulePath::new("statement.foreign"),
        )
    });
    for term in [0, 1, 3, 4] {
        assert_rejects!(
            &format!("witness term {term}"),
            "witness",
            |input: &mut SourceStatementB3RouteInputs| {
                input.witness.witnesses[0].term =
                    mizar_checker::source_statement::SourceStatementWitnessTermTarget::Primary(
                        mizar_checker::source_term::SourcePrimaryTermId::new(term),
                    )
            }
        );
    }
    for (label, mutate) in [
        (
            "wrong witness owner",
            (|input: &mut SourceStatementB3RouteInputs| {
                input.witness.witnesses[0].owner =
                    mizar_checker::source_statement::SourceTheoremOwnerId::new(1)
            }) as fn(&mut SourceStatementB3RouteInputs),
        ),
        ("wrong witness context", |input| {
            input.witness.witnesses[0].binding_context =
                mizar_checker::binding_env::BindingContextId::new(0)
        }),
        ("swapped take site", |input| {
            input.witness.witnesses[0].take_site =
                mizar_checker::typed_ast::TypedSiteRef::Node(
                    mizar_checker::typed_ast::TypedNodeId::new(34),
                )
        }),
        ("take range start", |input| {
            input.witness.witnesses[0].take_range.start += 1
        }),
        ("take range end", |input| {
            input.witness.witnesses[0].take_range.end -= 1
        }),
        ("swapped witness site", |input| {
            input.witness.witnesses[0].site =
                mizar_checker::typed_ast::TypedSiteRef::Node(
                    mizar_checker::typed_ast::TypedNodeId::new(35),
                )
        }),
        ("witness range start", |input| {
            input.witness.witnesses[0].source_range.start -= 1
        }),
        ("witness range end", |input| {
            input.witness.witnesses[0].source_range.end += 1
        }),
        ("witness source ordinal", |input| {
            input.witness.witnesses[0].source_ordinal = 2
        }),
        ("within-take ordinal", |input| {
            input.witness.witnesses[0].ordinal = 1
        }),
        ("witness spelling", |input| {
            input.witness.witnesses[0].spelling = "y".to_owned()
        }),
        ("witness recovery", |input| {
            input.witness.witnesses[0].recovery =
                mizar_checker::source_statement::SourceStatementRecovery::Degraded
        }),
    ] {
        assert_rejects!(label, "witness", mutate);
    }
    assert_rejects!("missing base owner", "aggregate", |input| {
        input.statement.owners.clear()
    });
    assert_rejects!("extra base owner", "aggregate", |input| {
        input
            .statement
            .owners
            .push(input.statement.owners[0].clone())
    });
    assert_rejects!("wrong base source", "", |input| {
        input.statement.source_id = foreign_source
    });
    assert_rejects!("wrong base module", "", |input| {
        input.statement.module_id = mizar_resolve::resolved_ast::ModuleId::new(
            mizar_session::PackageId::new("pkg"),
            mizar_session::ModulePath::new("statement.foreign"),
        )
    });
    assert_rejects!("wrong base owner contribution", "", |input| {
        input.statement.owners[0].contribution = foreign_contribution
    });
    for (label, mutate) in [
        (
            "missing base statement",
            (|input: &mut SourceStatementB3RouteInputs| {
                input.statement.statements.pop();
            }) as fn(&mut SourceStatementB3RouteInputs),
        ),
        ("extra base statement", |input| {
            input
                .statement
                .statements
                .push(input.statement.statements[1].clone())
        }),
        ("missing base context", |input| {
            input.statement.contexts.pop();
        }),
        ("extra base context", |input| {
            input
                .statement
                .contexts
                .push(input.statement.contexts[1].clone())
        }),
        ("missing base input fact", |input| {
            input.statement.input_facts.pop();
        }),
        ("extra base input fact", |input| {
            input
                .statement
                .input_facts
                .push(input.statement.input_facts[1].clone())
        }),
        ("missing base candidate", |input| {
            input.statement.candidate_facts.pop();
        }),
        ("extra base candidate", |input| {
            input
                .statement
                .candidate_facts
                .push(input.statement.candidate_facts[1].clone())
        }),
    ] {
        assert_rejects!(label, "", mutate);
    }
    assert_rejects!("theorem base ordinal", "statement", |input| {
        input.statement.statements[0].source_ordinal = 1
    });
    assert_rejects!("conclusion base ordinal", "statement", |input| {
        input.statement.statements[1].source_ordinal = 1
    });
    assert_rejects!("base statement reorder", "statement", |input| {
        input.statement.statements.swap(0, 1)
    });
    assert_rejects!("base context", "context", |input| {
        input.statement.contexts[1].binding_context =
            mizar_checker::binding_env::BindingContextId::new(0)
    });
    assert_rejects!("base input fact", "input fact", |input| {
        input.statement.input_facts[1].uses.swap(0, 1)
    });
    assert_rejects!("base candidate", "candidate", |input| {
        input.statement.candidate_facts[1].formula =
            mizar_checker::source_statement::SourceStatementFormulaTarget::Atomic(
                mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(0),
            )
    });
    for (label, mutate) in [
        (
            "base owner symbol",
            (|input: &mut SourceStatementB3RouteInputs| {
                input.statement.owners[0].symbol =
                    mizar_resolve::resolved_ast::SymbolId::new(
                        input.statement.module_id.clone(),
                        mizar_resolve::resolved_ast::LocalSymbolId::new(
                            "ForeignTask258B3Owner",
                        ),
                        mizar_resolve::resolved_ast::FullyQualifiedName::new(
                            "pkg::statement.fixture::theorem::ForeignTask258B3Owner",
                        ),
                    )
            }) as fn(&mut SourceStatementB3RouteInputs),
        ),
        ("base owner site", |input| {
            input.statement.owners[0].site =
                mizar_checker::typed_ast::TypedSiteRef::Node(
                    mizar_checker::typed_ast::TypedNodeId::new(43),
                )
        }),
        ("base owner range start", |input| {
            input.statement.owners[0].source_range.start += 1
        }),
        ("base owner range end", |input| {
            input.statement.owners[0].source_range.end -= 1
        }),
        ("base owner spelling", |input| {
            input.statement.owners[0].spelling.push('x')
        }),
        ("base owner recovery", |input| {
            input.statement.owners[0].recovery =
                mizar_checker::source_statement::SourceStatementRecovery::Degraded
        }),
    ] {
        assert_rejects!(label, "", mutate);
    }
    for index in 0..2 {
        let other = 1 - index;
        assert_rejects!(&format!("base statement {index} owner"), "", |input| {
            input.statement.statements[index].owner =
                mizar_checker::source_statement::SourceTheoremOwnerId::new(1)
        });
        assert_rejects!(
            &format!("base statement {index} context"),
            "",
            |input| {
                input.statement.statements[index].context =
                    mizar_checker::source_statement::SourceStatementContextId::new(other)
            }
        );
        assert_rejects!(
            &format!("base statement {index} formula"),
            "",
            |input| {
                input.statement.statements[index].formula =
                    mizar_checker::source_statement::SourceStatementFormulaTarget::Atomic(
                        mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(other),
                    )
            }
        );
        assert_rejects!(&format!("base statement {index} site"), "", |input| {
            input.statement.statements[index].site =
                mizar_checker::typed_ast::TypedSiteRef::Node(
                    mizar_checker::typed_ast::TypedNodeId::new([43, 45][index]),
                )
        });
        assert_rejects!(
            &format!("base statement {index} range start"),
            "",
            |input| input.statement.statements[index].source_range.start += 1
        );
        assert_rejects!(
            &format!("base statement {index} range end"),
            "",
            |input| input.statement.statements[index].source_range.end -= 1
        );
        assert_rejects!(
            &format!("base statement {index} spelling"),
            "",
            |input| input.statement.statements[index].spelling.push('x')
        );
        assert_rejects!(&format!("base statement {index} kind"), "", |input| {
            input.statement.statements[index].kind = [
                mizar_checker::source_statement::SourceStatementKind::Conclusion,
                mizar_checker::source_statement::SourceStatementKind::TheoremProposition,
            ][index]
        });
        assert_rejects!(
            &format!("base statement {index} recovery"),
            "",
            |input| {
                input.statement.statements[index].recovery =
                    mizar_checker::source_statement::SourceStatementRecovery::Degraded
            }
        );
        assert_rejects!(
            &format!("base context {index} statement"),
            "",
            |input| {
                input.statement.contexts[index].statement =
                    mizar_checker::source_statement::SourceStatementId::new(other)
            }
        );
        assert_rejects!(
            &format!("base context {index} binding context"),
            "",
            |input| {
                input.statement.contexts[index].binding_context =
                    mizar_checker::binding_env::BindingContextId::new(other)
            }
        );
        assert_rejects!(
            &format!("base context {index} range start"),
            "",
            |input| input.statement.contexts[index].source_range.start += 1
        );
        assert_rejects!(
            &format!("base context {index} range end"),
            "",
            |input| input.statement.contexts[index].source_range.end -= 1
        );
        assert_rejects!(
            &format!("base context {index} visibility"),
            "",
            |input| input.statement.contexts[index].visible_bindings.clear()
        );
        assert_rejects!(
            &format!("base input fact {index} statement"),
            "",
            |input| {
                input.statement.input_facts[index].statement =
                    mizar_checker::source_statement::SourceStatementId::new(other)
            }
        );
        assert_rejects!(
            &format!("base input fact {index} context"),
            "",
            |input| {
                input.statement.input_facts[index].context =
                    mizar_checker::source_statement::SourceStatementContextId::new(other)
            }
        );
        assert_rejects!(
            &format!("base input fact {index} ordinal"),
            "",
            |input| input.statement.input_facts[index].ordinal = 1
        );
        assert_rejects!(
            &format!("base input fact {index} binding"),
            "",
            |input| {
                input.statement.input_facts[index].binding =
                    mizar_checker::binding_env::BindingId::new(1)
            }
        );
        assert_rejects!(&format!("base input fact {index} uses"), "", |input| {
            input.statement.input_facts[index].uses.swap(0, 1)
        });
        assert_rejects!(
            &format!("base candidate {index} statement"),
            "",
            |input| {
                input.statement.candidate_facts[index].statement =
                    mizar_checker::source_statement::SourceStatementId::new(other)
            }
        );
        assert_rejects!(
            &format!("base candidate {index} context"),
            "",
            |input| {
                input.statement.candidate_facts[index].context =
                    mizar_checker::source_statement::SourceStatementContextId::new(other)
            }
        );
        assert_rejects!(
            &format!("base candidate {index} ordinal"),
            "",
            |input| input.statement.candidate_facts[index].ordinal = 1
        );
        assert_rejects!(
            &format!("base candidate {index} formula"),
            "",
            |input| {
                input.statement.candidate_facts[index].formula =
                    mizar_checker::source_statement::SourceStatementFormulaTarget::Atomic(
                        mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(other),
                    )
            }
        );
    }

    let (b1_ast, b1_module, _, b1_symbols) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B1_TEXT, 258_611);
    let b1 = source_statement_output_with_source(
        &b1_ast,
        b1_module,
        &b1_symbols,
        SOURCE_STATEMENT_B1_TEXT,
    )
    .expect("B1 selector")
    .expect("B1 output");
    let wrong_binding = b1
        .typed_ast
        .source_statement()
        .expect("B1 statement")
        .binding_env()
        .clone();
    let wrong_primary = b1.typed_ast.source_term().expect("B1 primary").clone();
    let wrong_atomic = b1
        .typed_ast
        .source_atomic_formula()
        .expect("B1 atomic")
        .clone();
    assert_rejects!("foreign proof scope binding", "dependency", move |input| {
        input.binding_env = wrong_binding.clone()
    });
    assert_rejects!("cross-profile primary", "dependency", move |input| {
        input.primary = wrong_primary.clone()
    });
    assert_rejects!("cross-profile atomic", "dependency", move |input| {
        input.atomic = wrong_atomic.clone()
    });

    for index in 0..49 {
        for mutation in 0..5 {
            assert_rejects!(
                &format!("arena node {index} mutation {mutation}"),
                "",
                |input: &mut SourceStatementB3RouteInputs| {
                    input.arena = mizar_checker::typed_ast::TypedArena::try_new(
                        input.arena.root(),
                        input
                            .arena
                            .iter()
                            .map(|(id, row)| {
                                let mut row = row.clone();
                                if id.index() == index {
                                    match mutation {
                                        0 => {
                                            let mizar_session::SourceAnchor::Range(mut range) =
                                                row.anchor.clone()
                                            else {
                                                unreachable!("Task258B3 range anchor")
                                            };
                                            range.end += 1;
                                            row.anchor = mizar_session::SourceAnchor::Range(range);
                                        }
                                        1 => {
                                            row.recovery = mizar_checker::typed_ast::
                                                NodeRecoveryState::Recovered
                                        }
                                        2 => {
                                            row.recovery = mizar_checker::typed_ast::
                                                NodeRecoveryState::Degraded
                                        }
                                        3 => {
                                            row.kind = "source.task258b3.mutated".into()
                                        }
                                        4 if row.children.len() > 1 => row.children.swap(0, 1),
                                        4 if row.children.len() == 1 => row.children.clear(),
                                        4 => row.children.push(
                                            mizar_checker::typed_ast::TypedNodeId::new(
                                                usize::from(index == 0),
                                            ),
                                        ),
                                        _ => unreachable!(),
                                    }
                                }
                                row
                            })
                            .collect(),
                    )
                    .expect("mutated arena remains structurally valid")
                }
            );
        }
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
        let resolver_error = source_statement_b3_output_with_resolver_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3_TEXT,
            |symbols| task258b2_mutate_resolver(symbols, mutation),
        )
        .expect("resolver selector")
        .expect_err("resolver provenance mutation");
        assert!(
            resolver_error.to_ascii_lowercase().contains("resolver")
                || resolver_error.to_ascii_lowercase().contains("owner")
                || resolver_error.to_ascii_lowercase().contains("label"),
            "{mutation:?}: {resolver_error}"
        );
        let replay = source_statement_output_with_source(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B3_TEXT,
        )
        .expect("resolver replay selector")
        .expect("resolver replay");
        assert_eq!(replay.typed_ast.debug_text(), baseline_debug);
        assert_eq!(replay.resolved.debug_text(), baseline_resolved);
    }
}

#[test]
fn task258b3_selector_rejects_named_multiple_and_cross_slice_sources() {
    let (exact_ast, exact_module, _, exact_symbols) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B3_TEXT, 258_619);
    assert_eq!(
        source_statement_transport_detail_keys(
            &exact_ast,
            exact_module.clone(),
            &exact_symbols,
            SOURCE_STATEMENT_B3_TEXT,
        ),
        Some(Vec::new())
    );
    for (label, loaded_source) in [
        (
            "missing final LF",
            SOURCE_STATEMENT_B3_TEXT.trim_end_matches('\n').to_owned(),
        ),
        (
            "extra final LF",
            format!("{SOURCE_STATEMENT_B3_TEXT}\n"),
        ),
        (
            "whitespace drift",
            SOURCE_STATEMENT_B3_TEXT.replacen("  take x;", "   take x;", 1),
        ),
        (
            "comment drift",
            SOURCE_STATEMENT_B3_TEXT.replacen("  take x;", "  :: witness\n  take x;", 1),
        ),
        (
            "hash-relevant label drift",
            SOURCE_STATEMENT_B3_TEXT.replacen(
                "FormulaStatementSingleWitnessSmoke",
                "FormulaStatementSingleWitnessNearMiss",
                1,
            ),
        ),
    ] {
        assert!(
            source_statement_output_with_source(
                &exact_ast,
                exact_module.clone(),
                &exact_symbols,
                &loaded_source,
            )
            .is_none(),
            "{label}"
        );
        assert!(
            source_statement_transport_detail_keys(
                &exact_ast,
                exact_module.clone(),
                &exact_symbols,
                &loaded_source,
            )
            .is_none(),
            "{label} production detail dispatch"
        );
    }
    let near_misses = [
        SOURCE_STATEMENT_B3_TEXT.replace("take x;", "take y = x;"),
        SOURCE_STATEMENT_B3_TEXT.replace("take x;", "take x, x;"),
        SOURCE_STATEMENT_B3_TEXT.replace("take x;", "take y;"),
        SOURCE_STATEMENT_B3_TEXT.replace("take x;", "take (x);"),
        SOURCE_STATEMENT_B3_TEXT.replace("take x;", "take {x};"),
        SOURCE_STATEMENT_B3_TEXT.replace("  take x;\n", ""),
        SOURCE_STATEMENT_B3_TEXT.replace("  take x;\n", "  take x;\n  take x;\n"),
        SOURCE_STATEMENT_B3_TEXT.replace(
            "  take x;\n  thus x = x;",
            "  thus x = x;\n  take x;",
        ),
        SOURCE_STATEMENT_B3_TEXT.replace("thus x = x;", "thus x = x;\n  thus x = x;"),
        SOURCE_STATEMENT_B3_TEXT.replace("take x;", "take x"),
        SOURCE_STATEMENT_B3_TEXT.replace("thus x = x;", "thus x = x by A;"),
        SOURCE_STATEMENT_B3_TEXT.replace(
            "FormulaStatementSingleWitnessSmoke",
            "FormulaStatementSingleWitnessNearMiss",
        ),
        SOURCE_STATEMENT_B3_TEXT.replace(": x = x proof", ": ex y being set st y = y proof"),
        SOURCE_STATEMENT_B3_TEXT.replace(": x = x proof", ": (x = x) & (x = x) proof"),
    ];
    for (ordinal, source) in near_misses.iter().enumerate() {
        let (ast, module, _, symbols) =
            task253_ast_from_source_text(source, 258_620 + ordinal);
        assert!(
            source_statement_output_with_source(&ast, module, &symbols, source).is_none(),
            "near miss {ordinal}"
        );
    }
    for (ordinal, source) in [
        SOURCE_STATEMENT_TEXT,
        SOURCE_STATEMENT_B1_TEXT,
        SOURCE_STATEMENT_B2_TEXT,
    ]
    .into_iter()
    .enumerate()
    {
        let (ast, module, _, symbols) =
            task253_ast_from_source_text(source, 258_630 + ordinal);
        let output = source_statement_output_with_source(&ast, module, &symbols, source)
            .expect("existing selector")
            .expect("existing route");
        assert!(output.typed_ast.source_statement_witnesses().is_none());
    }

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
    let plan = build_test_plan(&config).expect("Task258B3 isolation plan");
    let mut selected = Vec::new();
    for (ordinal, case) in active_type_elaboration_cases(&plan).enumerate() {
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .unwrap_or_else(|error| panic!("{} frontend failed: {error}", case.id.0));
        let source = frontend.source_text;
        let Some(ast) = frontend.ast else {
            continue;
        };
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        if !resolver.detail_keys.is_empty() {
            continue;
        }
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let extracted = extract_single_witness_source_statement(&ast, &source).is_some();
        let selected_route = source_statement_output_with_source(
            &ast,
            resolver.module.clone(),
            &symbols,
            &source,
        )
        .is_some_and(|result| {
            result.is_ok_and(|output| {
                output.typed_ast.source_statement_witnesses().is_some()
            })
        });
        if extracted || selected_route {
            selected.push(case.id.0.clone());
        }
    }
    assert!(
        selected.is_empty(),
        "Task258B3 selected active cases: {selected:?}"
    );
}

#[test]
fn task258b3_paired_ownership_rejects_all_existing_statement_profiles_in_both_orders() {
    let (b3_ast, b3_module, _, b3_symbols) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B3_TEXT, 258_640);
    let b3 = source_statement_output_with_source(
        &b3_ast,
        b3_module,
        &b3_symbols,
        SOURCE_STATEMENT_B3_TEXT,
    )
    .expect("B3 selector")
    .expect("B3 route");
    let b3_statement = b3.typed_ast.source_statement().expect("B3 statement").clone();
    let b3_witnesses = b3
        .typed_ast
        .source_statement_witnesses()
        .expect("B3 witnesses")
        .clone();
    let b3_debug = b3.typed_ast.debug_text();

    for (ordinal, source) in [SOURCE_STATEMENT_TEXT, SOURCE_STATEMENT_B2_TEXT]
        .into_iter()
        .enumerate()
    {
        let (ast, module, _, symbols) =
            task253_ast_from_source_text(source, 258_641 + ordinal);
        let existing = source_statement_output_with_source(&ast, module, &symbols, source)
            .expect("existing selector")
            .expect("existing route");
        let existing_debug = existing.typed_ast.debug_text();
        assert_eq!(
            existing
                .typed_ast
                .clone()
                .with_source_statement_witnesses(b3_statement.clone(), b3_witnesses.clone()),
            Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement)
        );
        assert_eq!(existing.typed_ast.debug_text(), existing_debug);
        assert_eq!(
            b3.typed_ast.clone().with_source_statement(
                existing
                    .typed_ast
                    .source_statement()
                    .expect("existing statement")
                    .clone()
            ),
            Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement)
        );
        assert_eq!(b3.typed_ast.debug_text(), b3_debug);
    }
    let (b1_ast, b1_module, _, b1_symbols) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B1_TEXT, 258_645);
    let b1 = source_statement_output_with_source(
        &b1_ast,
        b1_module,
        &b1_symbols,
        SOURCE_STATEMENT_B1_TEXT,
    )
    .expect("B1 selector")
    .expect("B1 route");
    let b1_debug = b1.typed_ast.debug_text();
    assert_eq!(
        b1.typed_ast
            .clone()
            .with_source_statement_witnesses(b3_statement.clone(), b3_witnesses.clone()),
        Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(b1.typed_ast.debug_text(), b1_debug);
    assert_eq!(
        b3.typed_ast.clone().with_source_statement_references(
            b1.typed_ast.source_statement().expect("B1 statement").clone(),
            b1.typed_ast
                .source_statement_references()
                .expect("B1 references")
                .clone(),
        ),
        Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(b3.typed_ast.debug_text(), b3_debug);
}

#[test]
fn task258b3_typed_and_final_debug_clone_preserve_empty_semantics() {
    let (ast, module, _, symbols) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B3_TEXT, 258_650);
    let output =
        source_statement_output_with_source(&ast, module, &symbols, SOURCE_STATEMENT_B3_TEXT)
            .expect("selector")
            .expect("route");
    let typed_debug = output.typed_ast.debug_text();
    let base_at = typed_debug.find("source-statement-debug-v1").expect("base debug");
    let witness_at = typed_debug
        .find("source-statement-witness-debug-v1")
        .expect("witness debug");
    let nodes_at = typed_debug.find("nodes:\n").expect("nodes debug");
    assert!(base_at < witness_at && witness_at < nodes_at);
    assert_eq!(output.typed_ast.clone().debug_text(), typed_debug);
    assert_eq!(output.resolved.clone().debug_text(), output.resolved.debug_text());
    assert!(output.resolved.source_statement_references().is_none());
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
    assert!(output.resolved.checked_formulas().is_empty());
    assert!(output.resolved.statement_semantics().is_empty());
    assert!(output.resolved.checked_proofs().is_empty());
    assert!(output.resolved.checked_proof_nodes().is_empty());
    assert!(output.resolved.checked_terminal_goals().is_empty());
    assert!(typed_debug.contains(
        "witness#0 owner=0 binding_context=1 term=primary#2 take_range=77..84 take_site=35 range=82..83 site=34 source_ordinal=1 ordinal=0 kind=unnamed recovery=normal spelling=\"x\""
    ));
}

#[test]
fn task258b2_real_frontend_freezes_single_assumption_transport() {
    assert_eq!(SOURCE_STATEMENT_B2_TEXT.len(), 113);
    assert!(SOURCE_STATEMENT_B2_TEXT.ends_with('\n'));
    let (ast, module, _, symbols) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B2_TEXT, 258_500);
    let extracted: SourceStatementB2Extraction =
        extract_single_assumption_source_statement(&ast, SOURCE_STATEMENT_B2_TEXT)
            .expect("Task258B2 exact parser shape");
    assert_eq!(ast.nodes().len(), 55);
    assert_eq!(ast.root().expect("root").index(), 54);
    assert_eq!(
        extracted
            .statement_sites
            .iter()
            .map(|site| site.node().index())
            .collect::<Vec<_>>(),
        [51, 41, 49]
    );
    assert_eq!(
        extracted
            .statement_ranges
            .iter()
            .map(|range| (range.start, range.end))
            .collect::<Vec<_>>(),
        [(19, 112), (80, 93), (96, 107)]
    );
    assert_eq!(
        extracted
            .formula_sites
            .iter()
            .map(|site| site.node().index())
            .collect::<Vec<_>>(),
        [32, 38, 46]
    );
    assert_eq!(
        extracted
            .formula_ranges
            .iter()
            .map(|range| (range.start, range.end))
            .collect::<Vec<_>>(),
        [(66, 71), (87, 92), (101, 106)]
    );
    assert_eq!(
        extracted
            .term_sites
            .iter()
            .map(|site| site.node().index())
            .collect::<Vec<_>>(),
        [28, 30, 34, 36, 42, 44]
    );
    assert_eq!(
        extracted
            .term_ranges
            .iter()
            .map(|range| (range.start, range.end))
            .collect::<Vec<_>>(),
        [(66, 67), (70, 71), (87, 88), (91, 92), (101, 102), (105, 106)]
    );
    assert_eq!(
        (
            extracted.theorem_site.node().index(),
            extracted.theorem_range.start,
            extracted.theorem_range.end,
            extracted.label_range.start,
            extracted.label_range.end,
            extracted.proof_range.start,
            extracted.proof_range.end,
        ),
        (51, 19, 112, 27, 64, 72, 111)
    );

    let output =
        source_statement_output_with_source(&ast, module, &symbols, SOURCE_STATEMENT_B2_TEXT)
            .expect("Task258B2 selector")
            .unwrap_or_else(|error| panic!("Task258B2 route failed: {error}"));
    let statement = output
        .typed_ast
        .source_statement()
        .expect("Task258B2 statement");
    assert_eq!(statement.statements().len(), 3);
    assert_eq!(
        statement
            .statements()
            .get(mizar_checker::source_statement::SourceStatementId::new(1))
            .expect("assumption row")
            .kind(),
        mizar_checker::source_statement::SourceStatementKind::Assumption
    );
    assert_eq!(
        (
            statement.binding_env().contexts().len(),
            statement.binding_env().bindings().len(),
            statement.binding_env().diagnostics().len(),
            statement.owners().len(),
            statement.statements().len(),
            statement.contexts().len(),
            statement.input_facts().len(),
            statement.candidate_facts().len(),
        ),
        (2, 1, 0, 1, 3, 3, 3, 3)
    );
    assert_eq!(
        statement
            .statements()
            .iter()
            .map(|(_, row)| (
                row.site().node().index(),
                row.context().index(),
                row.kind(),
                row.spelling(),
            ))
            .collect::<Vec<_>>(),
        [
            (
                51,
                0,
                mizar_checker::source_statement::SourceStatementKind::TheoremProposition,
                "theorem FormulaStatementSingleAssumptionSmoke : x = x proof assume x = x ; thus x = x ; end ;",
            ),
            (
                41,
                1,
                mizar_checker::source_statement::SourceStatementKind::Assumption,
                "assume x = x ;",
            ),
            (
                49,
                2,
                mizar_checker::source_statement::SourceStatementKind::Conclusion,
                "thus x = x ;",
            ),
        ]
    );
    assert_eq!(
        output
            .typed_ast
            .source_term()
            .expect("Task252 handoff")
            .references()
            .iter()
            .map(|(_, row)| row.use_ordinal())
            .collect::<Vec<_>>(),
        [1; 6]
    );
    assert_eq!(
        output
            .typed_ast
            .source_atomic_formula()
            .expect("Task256 handoff")
            .formulas()
            .iter()
            .map(|(_, row)| row.context().index())
            .collect::<Vec<_>>(),
        [0, 1, 1]
    );
    assert_eq!(output.reference_use_ordinals, [1; 6]);
    assert!(output.typed_ast.source_statement_references().is_none());
    assert_eq!(
        output.typed_ast.source_statement(),
        output.resolved.source_statement()
    );
    assert_eq!(output.typed_ast.nodes().len(), ast.nodes().len());
    assert_eq!(
        output.typed_ast.nodes().root().map(|root| root.index()),
        ast.root().map(|root| root.index())
    );
    for (id, node) in output.typed_ast.nodes().iter() {
        let surface = &ast.nodes()[id.index()];
        assert_eq!(
            node.anchor,
            mizar_session::SourceAnchor::Range(surface.range),
            "node {} range",
            id.index()
        );
        assert_eq!(
            node.children
                .iter()
                .map(|child| child.index())
                .collect::<Vec<_>>(),
            surface
                .children
                .iter()
                .map(|child| child.index())
                .collect::<Vec<_>>(),
            "node {} children",
            id.index()
        );
        assert_eq!(
            node.recovery,
            mizar_checker::typed_ast::NodeRecoveryState::Normal,
            "node {} recovery",
            id.index()
        );
    }
}

#[test]
fn task258b2_lower_resolver_and_row_mutations_fail_closed_then_replay() {
    let (ast, module, _, symbols) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B2_TEXT, 258_510);
    let baseline =
        source_statement_output_with_source(&ast, module.clone(), &symbols, SOURCE_STATEMENT_B2_TEXT)
            .expect("Task258B2 selector")
            .expect("Task258B2 baseline");
    let baseline_typed = baseline.typed_ast.debug_text();
    let baseline_resolved = baseline.resolved.debug_text();
    let (b1_ast, b1_module, _, b1_symbols) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B1_TEXT, 258_511);
    let b1 = source_statement_output_with_source(
        &b1_ast,
        b1_module,
        &b1_symbols,
        SOURCE_STATEMENT_B1_TEXT,
    )
    .expect("Task258B1 selector")
    .expect("Task258B1 output");
    let wrong_binding = b1
        .typed_ast
        .source_statement()
        .expect("Task258B1 statement")
        .binding_env()
        .clone();
    let wrong_primary = b1
        .typed_ast
        .source_term()
        .expect("Task258B1 Task252")
        .clone();
    let wrong_atomic = b1
        .typed_ast
        .source_atomic_formula()
        .expect("Task258B1 Task256")
        .clone();
    for (label, expected, mutate) in [
        ("statement aggregate", "aggregate", 0usize),
        ("assumption kind", "statement", 1usize),
        ("assumption context", "context", 2usize),
        ("assumption input", "input fact", 3usize),
        ("assumption candidate", "candidate", 4usize),
        ("cross-formula edge", "statement", 5usize),
        ("theorem owner range", "owner", 6usize),
        ("recovered assumption", "statement", 7usize),
        ("cross-profile binding", "dependency", 8usize),
        ("cross-profile primary", "dependency", 9usize),
        ("cross-profile atomic", "dependency", 10usize),
    ] {
        let error = source_statement_b2_output_with_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B2_TEXT,
            |inputs| match mutate {
                0 => inputs.statement.candidate_facts.pop().map(drop).unwrap(),
                1 => {
                    inputs.statement.statements[1].kind =
                        mizar_checker::source_statement::SourceStatementKind::Conclusion;
                }
                2 => {
                    inputs.statement.contexts[1].binding_context =
                        mizar_checker::binding_env::BindingContextId::new(0);
                }
                3 => inputs.statement.input_facts[1].uses.swap(0, 1),
                4 => {
                    inputs.statement.candidate_facts[1].formula =
                        mizar_checker::source_statement::SourceStatementFormulaTarget::Atomic(
                            mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(2),
                        );
                }
                5 => {
                    inputs.statement.statements[1].formula =
                        mizar_checker::source_statement::SourceStatementFormulaTarget::Atomic(
                            mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(2),
                        );
                }
                6 => {
                    inputs.statement.owners[0].source_range.end -= 1;
                }
                7 => {
                    inputs.arena = mizar_checker::typed_ast::TypedArena::try_new(
                        inputs.arena.root(),
                        inputs
                            .arena
                            .iter()
                            .map(|(id, row)| {
                                let mut row = row.clone();
                                if id.index() == 41 {
                                    row.recovery =
                                        mizar_checker::typed_ast::NodeRecoveryState::Recovered;
                                }
                                row
                            })
                            .collect(),
                    )
                    .expect("recovered arena remains structurally valid");
                }
                8 => inputs.binding_env = wrong_binding.clone(),
                9 => inputs.primary = wrong_primary.clone(),
                10 => inputs.atomic = wrong_atomic.clone(),
                _ => unreachable!(),
            },
        )
        .unwrap_or_else(|| panic!("{label} selector"))
        .expect_err(label);
        assert!(
            error.to_ascii_lowercase().contains(expected),
            "{label}: {error}"
        );
        let replay = source_statement_output_with_source(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B2_TEXT,
        )
        .expect("replay selector")
        .expect("replay output");
        assert_eq!(replay.typed_ast.debug_text(), baseline_typed, "{label}");
        assert_eq!(replay.resolved.debug_text(), baseline_resolved, "{label}");
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
        let resolver_error = source_statement_b2_output_with_resolver_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B2_TEXT,
            |symbols| task258b2_mutate_resolver(symbols, mutation),
        )
        .expect("Task258B2 resolver selector")
        .expect_err("resolver provenance mutation must fail");
        assert!(
            resolver_error.to_ascii_lowercase().contains("provenance")
                || resolver_error.to_ascii_lowercase().contains("owner")
                || resolver_error.to_ascii_lowercase().contains("label"),
            "{mutation:?}: {resolver_error}"
        );
        let replay = source_statement_output_with_source(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B2_TEXT,
        )
        .expect("resolver replay selector")
        .expect("resolver replay output");
        assert_eq!(
            replay.typed_ast.debug_text(),
            baseline_typed,
            "{mutation:?}"
        );
        assert_eq!(
            replay.resolved.debug_text(),
            baseline_resolved,
            "{mutation:?}"
        );
    }
}

#[test]
fn task258b2_selector_rejects_exact_assumption_near_misses() {
    let (exact_ast, exact_module, _, exact_symbols) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B2_TEXT, 258_520);
    for (label, loaded) in [
        (
            "missing final LF",
            SOURCE_STATEMENT_B2_TEXT.trim_end_matches('\n').to_owned(),
        ),
        ("extra final LF", format!("{SOURCE_STATEMENT_B2_TEXT}\n")),
        (
            "whitespace byte drift",
            SOURCE_STATEMENT_B2_TEXT.replacen("  assume", " assume", 1),
        ),
        (
            "comment byte drift",
            SOURCE_STATEMENT_B2_TEXT.replacen("reserve x for set;", "reserve x for set; :: drift", 1),
        ),
        (
            "theorem name drift",
            SOURCE_STATEMENT_B2_TEXT.replacen(
                "FormulaStatementSingleAssumptionSmoke",
                "FormulaStatementSingleAssumptionOther",
                1,
            ),
        ),
    ] {
        assert!(
            source_statement_output_with_source(
                &exact_ast,
                exact_module.clone(),
                &exact_symbols,
                &loaded,
            )
            .is_none(),
            "{label}"
        );
    }
    for (ordinal, (label, source)) in [
        (
            "labeled assumption",
            SOURCE_STATEMENT_B2_TEXT.replacen("  assume", "  A: assume", 1),
        ),
        (
            "collective assumption",
            SOURCE_STATEMENT_B2_TEXT.replacen("assume x = x", "assume that x = x", 1),
        ),
        (
            "given statement",
            SOURCE_STATEMENT_B2_TEXT.replacen("assume x = x", "given y being set", 1),
        ),
        (
            "consider statement",
            SOURCE_STATEMENT_B2_TEXT.replacen(
                "assume x = x",
                "consider y being set such that x = x",
                1,
            ),
        ),
        (
            "witness statement",
            SOURCE_STATEMENT_B2_TEXT.replacen("assume x = x;", "take x;", 1),
        ),
        (
            "then assumption",
            SOURCE_STATEMENT_B2_TEXT.replacen("assume x = x", "then assume x = x", 1),
        ),
        (
            "hence conclusion",
            SOURCE_STATEMENT_B2_TEXT.replacen("thus x = x", "hence x = x", 1),
        ),
        (
            "nested proof",
            SOURCE_STATEMENT_B2_TEXT.replacen(
                "  assume x = x;",
                "  A: x = x proof\n    thus x = x;\n  end;",
                1,
            ),
        ),
        (
            "reordered statements",
            SOURCE_STATEMENT_B2_TEXT.replacen(
                "  assume x = x;\n  thus x = x;",
                "  thus x = x;\n  assume x = x;",
                1,
            ),
        ),
        (
            "composite assumption",
            SOURCE_STATEMENT_B2_TEXT.replacen("assume x = x;", "assume x = x & x = x;", 1),
        ),
        (
            "extra statement",
            SOURCE_STATEMENT_B2_TEXT.replacen(
                "  thus x = x;",
                "  assume x = x;\n  thus x = x;",
                1,
            ),
        ),
        (
            "recovered assumption",
            SOURCE_STATEMENT_B2_TEXT.replacen("assume x = x;", "assume x = x", 1),
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let (ast, module, _, symbols) =
            task253_ast_from_source_text(&source, 258_521 + ordinal);
        assert!(
            source_statement_output_with_source(&ast, module.clone(), &symbols, &source).is_none(),
            "{label}"
        );
        assert!(
            source_statement_output_with_source(
                &ast,
                module,
                &symbols,
                SOURCE_STATEMENT_B2_TEXT,
            )
            .is_none(),
            "{label} exact guard with wrong AST"
        );
    }
}

#[test]
fn task258b2_keeps_task258a_task258b1_and_active_routes_isolated() {
    for (ordinal, source) in [TASK258A_SOURCE, SOURCE_STATEMENT_B1_TEXT]
        .into_iter()
        .enumerate()
    {
        let (ast, module, _, symbols) = task253_ast_from_source_text(source, 258_540 + ordinal);
        assert!(
            extract_single_assumption_source_statement(&ast, source).is_none(),
            "cross-profile selector"
        );
        let output = source_statement_output_with_source(&ast, module, &symbols, source)
            .expect("prior profile selector")
            .expect("prior profile output");
        assert!(
            output
                .typed_ast
                .source_statement()
                .is_some_and(|statement| {
                    statement.statements().len() != 3
                        || statement
                            .statements()
                            .get(mizar_checker::source_statement::SourceStatementId::new(1))
                            .is_none_or(|row| {
                                row.kind()
                                    != mizar_checker::source_statement::SourceStatementKind::Assumption
                            })
                })
        );
    }

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
    let plan = build_test_plan(&config).expect("Task258B2 isolation plan");
    let mut selected = Vec::new();
    for (ordinal, case) in active_type_elaboration_cases(&plan).enumerate() {
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .unwrap_or_else(|error| panic!("{} frontend failed: {error}", case.id.0));
        let source = frontend.source_text;
        let Some(ast) = frontend.ast else {
            continue;
        };
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        if !resolver.detail_keys.is_empty() {
            continue;
        }
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        if extract_single_assumption_source_statement(&ast, &source).is_some()
            || source_statement_output_with_source(&ast, resolver.module, &symbols, &source)
                .is_some_and(|result| {
                    result.is_ok_and(|output| {
                        output
                            .typed_ast
                            .source_statement()
                            .is_some_and(|statement| {
                                statement.statements().len() == 3
                                    && statement
                                        .statements()
                                        .get(
                                            mizar_checker::source_statement::SourceStatementId::new(
                                                1,
                                            ),
                                        )
                                        .is_some_and(|row| {
                                            row.kind()
                                                == mizar_checker::source_statement::SourceStatementKind::Assumption
                                        })
                            })
                    })
                })
        {
            selected.push(case.id.0.clone());
        }
    }
    assert!(
        selected.is_empty(),
        "Task258B2 selected active cases: {selected:?}"
    );
}

#[test]
fn task258b2_typed_final_ownership_clone_and_empty_semantics_are_atomic() {
    let (ast, module, _, symbols) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B2_TEXT, 258_550);
    let first =
        source_statement_output_with_source(&ast, module.clone(), &symbols, SOURCE_STATEMENT_B2_TEXT)
            .expect("Task258B2 selector")
            .expect("Task258B2 output");
    let second =
        source_statement_output_with_source(&ast, module, &symbols, SOURCE_STATEMENT_B2_TEXT)
            .expect("Task258B2 replay selector")
            .expect("Task258B2 replay output");
    assert_eq!(first.typed_ast, first.typed_ast.clone());
    assert_eq!(first.resolved, first.resolved.clone());
    assert_eq!(first.typed_ast.debug_text(), second.typed_ast.debug_text());
    assert_eq!(first.resolved.debug_text(), second.resolved.debug_text());
    let debug = first.typed_ast.debug_text();
    let primary = debug.find("source-primary-term-debug-v1").expect("primary");
    let atomic = debug.find("source-atomic-formula-debug-v1").expect("atomic");
    let statement = debug.find("source-statement-debug-v1").expect("statement");
    let nodes = debug.find("nodes:").expect("nodes");
    assert!(primary < atomic && atomic < statement && statement < nodes);
    assert!(first.typed_ast.source_statement_references().is_none());
    assert!(first.resolved.source_statement_references().is_none());
    assert!(first.typed_ast.types().is_empty());
    assert!(first.typed_ast.facts().is_empty());
    assert!(first.typed_ast.coercions().is_empty());
    assert!(first.typed_ast.initial_obligations().is_empty());
    assert!(first.typed_ast.diagnostics().is_empty());
    assert!(first.resolved.expr_metadata().is_empty());
    assert!(first.resolved.cluster_facts().is_empty());
    assert!(first.resolved.diagnostics().is_empty());
    assert!(first.resolved.checked_formulas().is_empty());
    assert!(first.resolved.statement_semantics().is_empty());
    assert!(first.resolved.checked_proofs().is_empty());
    assert!(first.resolved.checked_proof_nodes().is_empty());
    assert!(first.resolved.checked_terminal_goals().is_empty());

    let task248 = task248_real_output();
    let before = task248.typed_ast.debug_text();
    let statement = first
        .typed_ast
        .source_statement()
        .expect("Task258B2 statement")
        .clone();
    assert_eq!(
        task248.typed_ast.clone().with_source_statement(statement),
        Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(task248.typed_ast.debug_text(), before);
}

#[test]
fn task258b1_real_frontend_freezes_nested_statement_and_resolver_bundle() {
    assert_eq!(SOURCE_STATEMENT_B1_TEXT.len(), 139);
    assert!(SOURCE_STATEMENT_B1_TEXT.ends_with('\n'));
    let (ast, module, _, symbols) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B1_TEXT, 258_400);
    let extracted: SourceStatementB1Extraction =
        extract_nested_source_statement(&ast, SOURCE_STATEMENT_B1_TEXT)
            .expect("Task258B1 exact parser shape");
    assert_eq!(ast.nodes().len(), 77);
    assert_eq!(ast.root().expect("root").index(), 76);
    assert_eq!(extracted.theorem_site.node().index(), 73);
    assert_eq!(
        extracted
            .statement_sites
            .iter()
            .map(|site| site.node().index())
            .collect::<Vec<_>>(),
        [73, 60, 58, 71]
    );
    assert_eq!(
        extracted
            .statement_ranges
            .iter()
            .map(|range| (range.start, range.end))
            .collect::<Vec<_>>(),
        [(19, 138), (77, 114), (96, 107), (117, 133)]
    );
    assert_eq!(
        extracted
            .formula_ranges
            .iter()
            .map(|range| (range.start, range.end))
            .collect::<Vec<_>>(),
        [(63, 68), (80, 85), (101, 106), (122, 127)]
    );
    assert_eq!((extracted.theorem_range.start, extracted.theorem_range.end), (19, 138));
    assert_eq!((extracted.label_range.start, extracted.label_range.end), (27, 61));
    assert_eq!(
        extracted
            .formula_sites
            .iter()
            .map(|site| site.node().index())
            .collect::<Vec<_>>(),
        [42, 48, 55, 65]
    );
    assert_eq!(
        extracted
            .term_sites
            .iter()
            .map(|site| site.node().index())
            .collect::<Vec<_>>(),
        [38, 40, 44, 46, 51, 53, 61, 63]
    );
    assert_eq!(
        extracted
            .term_ranges
            .iter()
            .map(|range| (range.start, range.end))
            .collect::<Vec<_>>(),
        [
            (63, 64),
            (67, 68),
            (80, 81),
            (84, 85),
            (101, 102),
            (105, 106),
            (122, 123),
            (126, 127),
        ]
    );
    assert_eq!(
        extracted
            .proof_ranges
            .map(|range| (range.start, range.end)),
        [(69, 137), (86, 113)]
    );
    let output =
        source_statement_output_with_source(&ast, module, &symbols, SOURCE_STATEMENT_B1_TEXT)
            .expect("Task258B1 selector")
            .unwrap_or_else(|error| panic!("Task258B1 route failed: {error}"));
    let statement = output
        .typed_ast
        .source_statement()
        .expect("Task258B1 statement");
    let references = output
        .typed_ast
        .source_statement_references()
        .expect("Task258B1 references");
    assert_eq!(
        (
            statement.owners().len(),
            statement.statements().len(),
            statement.contexts().len(),
            statement.input_facts().len(),
            statement.candidate_facts().len(),
            references.labels().len(),
            references.citations().len(),
        ),
        (1, 4, 4, 4, 4, 1, 1)
    );
    let owner = statement
        .owners()
        .get(mizar_checker::source_statement::SourceTheoremOwnerId::new(0))
        .expect("theorem owner");
    assert_eq!(owner.site().node().index(), 73);
    assert_eq!((owner.source_range().start, owner.source_range().end), (19, 138));
    assert_eq!(owner.spelling(), "FormulaStatementNestedContextSmoke");
    assert_eq!(
        (
            owner.role(),
            owner.status(),
            owner.recovery(),
        ),
        (
            mizar_checker::source_statement::SourceTheoremRole::Theorem,
            mizar_checker::source_statement::SourceTheoremStatus::Unmodified,
            mizar_checker::source_statement::SourceStatementRecovery::Normal,
        )
    );
    assert_eq!(
        statement
            .statements()
            .iter()
            .map(|(id, row)| (
                id.index(),
                row.owner().index(),
                row.context().index(),
                row.formula(),
                row.site().node().index(),
                row.source_range().start,
                row.source_range().end,
                row.source_ordinal(),
                row.spelling(),
                row.kind(),
                row.recovery(),
            ))
            .collect::<Vec<_>>(),
        [
            (
                0,
                0,
                0,
                mizar_checker::source_statement::SourceStatementFormulaTarget::Atomic(
                    mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(0),
                ),
                73,
                19,
                138,
                0,
                "theorem FormulaStatementNestedContextSmoke : x = x proof A : x = x proof thus x = x ; end ; thus x = x by A ; end ;",
                mizar_checker::source_statement::SourceStatementKind::TheoremProposition,
                mizar_checker::source_statement::SourceStatementRecovery::Normal,
            ),
            (
                1,
                0,
                1,
                mizar_checker::source_statement::SourceStatementFormulaTarget::Atomic(
                    mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(1),
                ),
                60,
                77,
                114,
                1,
                "A : x = x proof thus x = x ; end ;",
                mizar_checker::source_statement::SourceStatementKind::ProofStepProposition,
                mizar_checker::source_statement::SourceStatementRecovery::Normal,
            ),
            (
                2,
                0,
                2,
                mizar_checker::source_statement::SourceStatementFormulaTarget::Atomic(
                    mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(2),
                ),
                58,
                96,
                107,
                2,
                "thus x = x ;",
                mizar_checker::source_statement::SourceStatementKind::Conclusion,
                mizar_checker::source_statement::SourceStatementRecovery::Normal,
            ),
            (
                3,
                0,
                3,
                mizar_checker::source_statement::SourceStatementFormulaTarget::Atomic(
                    mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(3),
                ),
                71,
                117,
                133,
                3,
                "thus x = x by A ;",
                mizar_checker::source_statement::SourceStatementKind::Conclusion,
                mizar_checker::source_statement::SourceStatementRecovery::Normal,
            ),
        ]
    );
    assert_eq!(
        statement
            .contexts()
            .iter()
            .map(|(id, row)| (
                id.index(),
                row.statement().index(),
                row.binding_context().index(),
                row.source_range().start,
                row.source_range().end,
                row.visible_bindings()
                    .iter()
                    .map(|binding| binding.index())
                    .collect::<Vec<_>>(),
            ))
            .collect::<Vec<_>>(),
        [
            (0, 0, 0, 19, 138, vec![0]),
            (1, 1, 1, 77, 114, vec![0]),
            (2, 2, 2, 96, 107, vec![0]),
            (3, 3, 1, 117, 133, vec![0]),
        ]
    );
    assert_eq!(
        statement
            .input_facts()
            .iter()
            .map(|(id, row)| (
                id.index(),
                row.statement().index(),
                row.context().index(),
                row.ordinal(),
                row.kind(),
                row.binding().index(),
                row.uses()
                    .iter()
                    .map(|reference| reference.index())
                    .collect::<Vec<_>>(),
            ))
            .collect::<Vec<_>>(),
        [
            (
                0,
                0,
                0,
                0,
                mizar_checker::source_statement::SourceStatementInputFactKind::ReservedTypeGuard,
                0,
                vec![0, 1],
            ),
            (
                1,
                1,
                1,
                0,
                mizar_checker::source_statement::SourceStatementInputFactKind::ReservedTypeGuard,
                0,
                vec![2, 3],
            ),
            (
                2,
                2,
                2,
                0,
                mizar_checker::source_statement::SourceStatementInputFactKind::ReservedTypeGuard,
                0,
                vec![4, 5],
            ),
            (
                3,
                3,
                3,
                0,
                mizar_checker::source_statement::SourceStatementInputFactKind::ReservedTypeGuard,
                0,
                vec![6, 7],
            ),
        ]
    );
    assert_eq!(
        statement
            .candidate_facts()
            .iter()
            .map(|(id, row)| (
                id.index(),
                row.statement().index(),
                row.context().index(),
                row.ordinal(),
                row.kind(),
                row.formula(),
            ))
            .collect::<Vec<_>>(),
        (0..4)
            .map(|index| (
                index,
                index,
                index,
                0,
                mizar_checker::source_statement::SourceStatementCandidateFactKind::UnverifiedProposition,
                mizar_checker::source_statement::SourceStatementFormulaTarget::Atomic(
                    mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(index),
                ),
            ))
            .collect::<Vec<_>>()
    );
    let binding_contexts = statement.binding_env().contexts();
    assert_eq!(binding_contexts.len(), 3);
    let module_context = binding_contexts
        .get(mizar_checker::binding_env::BindingContextId::new(0))
        .expect("module context");
    assert_eq!(module_context.owner, mizar_checker::binding_env::BindingContextOwner::Module);
    assert_eq!(module_context.parent, None);
    assert_eq!(
        module_context.layer,
        mizar_checker::binding_env::BindingContextLayer::Module
    );
    assert_eq!(module_context.bindings, [mizar_checker::binding_env::BindingId::new(0)]);
    assert_eq!(
        module_context.visible_bindings,
        [mizar_checker::binding_env::BindingId::new(0)]
    );
    assert_eq!(
        module_context.recovery,
        mizar_checker::binding_env::BindingContextRecovery::Normal
    );
    for (id, parent, scope, range) in [
        (1, 0, &[0][..], (69, 137)),
        (2, 1, &[0, 0][..], (86, 113)),
    ] {
        let context = binding_contexts
            .get(mizar_checker::binding_env::BindingContextId::new(id))
            .expect("proof context");
        assert_eq!(
            context.parent.map(|parent| parent.index()),
            Some(parent)
        );
        assert_eq!(
            context.layer,
            mizar_checker::binding_env::BindingContextLayer::Proof
        );
        assert_eq!(
            context
                .lexical_scope
                .as_ref()
                .expect("proof scope")
                .path(),
            scope
        );
        assert_eq!(
            context.owner,
            mizar_checker::binding_env::BindingContextOwner::SourceStatement {
                source_range: mizar_session::SourceRange {
                    source_id: ast.source_id,
                    start: range.0,
                    end: range.1,
                },
            }
        );
        assert_eq!(context.visible_bindings, [mizar_checker::binding_env::BindingId::new(0)]);
        assert!(context.bindings.is_empty());
        assert_eq!(
            context.recovery,
            mizar_checker::binding_env::BindingContextRecovery::Normal
        );
    }
    assert_eq!(statement.binding_env().bindings().len(), 1);
    assert!(statement.binding_env().diagnostics().is_empty());
    let binding = statement
        .binding_env()
        .bindings()
        .get(mizar_checker::binding_env::BindingId::new(0))
        .expect("reserved binding");
    assert_eq!(
        (
            binding.id.index(),
            binding.spelling.as_str(),
            binding.kind,
            binding.owner_context.index(),
            binding.declaration_range.start,
            binding.declaration_range.end,
            binding.visible_after_ordinal,
            binding.status,
            binding.recovery,
        ),
        (
            0,
            "x",
            mizar_checker::binding_env::BindingKind::ReservedVariable,
            0,
            8,
            9,
            0,
            mizar_checker::binding_env::BindingStatus::Reserved,
            mizar_checker::binding_env::BindingRecoveryState::Normal,
        )
    );
    assert!(matches!(
        binding.identity,
        mizar_checker::binding_env::BinderIdentity::ReservedVariable { .. }
    ));
    assert!(matches!(
        binding.type_site,
        mizar_checker::binding_env::BindingTypeSite::Source(range)
            if (range.start, range.end) == (14, 17)
    ));
    assert!(binding.captured.identities().is_empty());
    assert!(binding.diagnostics.is_empty());
    assert_eq!(
        statement.binding_fingerprint(),
        statement.binding_env().debug_text()
    );
    let primary = output.typed_ast.source_term().expect("Task252 terms");
    assert_eq!(
        primary
            .terms()
            .iter()
            .map(|(id, term)| (
                id.index(),
                term.site().node().index(),
                term.context().index(),
                term.source_range().start,
                term.source_range().end,
                term.source_ordinal(),
                term.recovery(),
                term.spelling(),
                term.kind(),
                term.role(),
                term.parent().map(|parent| parent.index()),
            ))
            .collect::<Vec<_>>(),
        [
            (0, 38, 0, 63, 64, 0, mizar_checker::source_term::SourcePrimaryTermRecovery::Normal, "x", mizar_checker::source_term::SourcePrimaryTermKind::VariableReference, mizar_checker::source_term::SourcePrimaryTermRole::Value, None),
            (1, 40, 0, 67, 68, 1, mizar_checker::source_term::SourcePrimaryTermRecovery::Normal, "x", mizar_checker::source_term::SourcePrimaryTermKind::VariableReference, mizar_checker::source_term::SourcePrimaryTermRole::Value, None),
            (2, 44, 1, 80, 81, 2, mizar_checker::source_term::SourcePrimaryTermRecovery::Normal, "x", mizar_checker::source_term::SourcePrimaryTermKind::VariableReference, mizar_checker::source_term::SourcePrimaryTermRole::Value, None),
            (3, 46, 1, 84, 85, 3, mizar_checker::source_term::SourcePrimaryTermRecovery::Normal, "x", mizar_checker::source_term::SourcePrimaryTermKind::VariableReference, mizar_checker::source_term::SourcePrimaryTermRole::Value, None),
            (4, 51, 2, 101, 102, 4, mizar_checker::source_term::SourcePrimaryTermRecovery::Normal, "x", mizar_checker::source_term::SourcePrimaryTermKind::VariableReference, mizar_checker::source_term::SourcePrimaryTermRole::Value, None),
            (5, 53, 2, 105, 106, 5, mizar_checker::source_term::SourcePrimaryTermRecovery::Normal, "x", mizar_checker::source_term::SourcePrimaryTermKind::VariableReference, mizar_checker::source_term::SourcePrimaryTermRole::Value, None),
            (6, 61, 1, 122, 123, 6, mizar_checker::source_term::SourcePrimaryTermRecovery::Normal, "x", mizar_checker::source_term::SourcePrimaryTermKind::VariableReference, mizar_checker::source_term::SourcePrimaryTermRole::Value, None),
            (7, 63, 1, 126, 127, 7, mizar_checker::source_term::SourcePrimaryTermRecovery::Normal, "x", mizar_checker::source_term::SourcePrimaryTermKind::VariableReference, mizar_checker::source_term::SourcePrimaryTermRole::Value, None),
        ]
    );
    assert_eq!(
        primary
            .references()
            .iter()
            .map(|(id, reference)| (
                id.index(),
                reference.term().index(),
                reference.binding().index(),
                reference
                    .lexical_scope()
                    .map(|scope| scope.path().to_vec()),
                reference.use_ordinal(),
                reference.role(),
            ))
            .collect::<Vec<_>>(),
        [
            (0, 0, 0, None, 1, mizar_checker::source_term::SourcePrimaryTermReferenceRole::Variable),
            (1, 1, 0, None, 1, mizar_checker::source_term::SourcePrimaryTermReferenceRole::Variable),
            (2, 2, 0, Some(vec![0]), 1, mizar_checker::source_term::SourcePrimaryTermReferenceRole::Variable),
            (3, 3, 0, Some(vec![0]), 1, mizar_checker::source_term::SourcePrimaryTermReferenceRole::Variable),
            (4, 4, 0, Some(vec![0, 0]), 1, mizar_checker::source_term::SourcePrimaryTermReferenceRole::Variable),
            (5, 5, 0, Some(vec![0, 0]), 1, mizar_checker::source_term::SourcePrimaryTermReferenceRole::Variable),
            (6, 6, 0, Some(vec![0]), 1, mizar_checker::source_term::SourcePrimaryTermReferenceRole::Variable),
            (7, 7, 0, Some(vec![0]), 1, mizar_checker::source_term::SourcePrimaryTermReferenceRole::Variable),
        ]
    );
    let atomic = output
        .typed_ast
        .source_atomic_formula()
        .expect("Task256 atomics");
    assert_eq!((atomic.formulas().len(), atomic.edges().len(), atomic.requests().len()), (4, 8, 8));
    assert_eq!(
        atomic
            .formulas()
            .iter()
            .map(|(id, formula)| (
                id.index(),
                formula.site().node().index(),
                formula.context().index(),
                formula.source_range().start,
                formula.source_range().end,
                formula.source_ordinal(),
                formula.recovery(),
                formula.spelling(),
                formula.kind(),
            ))
            .collect::<Vec<_>>(),
        [
            (0, 42, 0, 63, 68, 0, mizar_checker::source_atomic_formula::SourceAtomicFormulaRecovery::Normal, "x = x", mizar_checker::source_atomic_formula::SourceAtomicFormulaKind::Equality),
            (1, 48, 1, 80, 85, 1, mizar_checker::source_atomic_formula::SourceAtomicFormulaRecovery::Normal, "x = x", mizar_checker::source_atomic_formula::SourceAtomicFormulaKind::Equality),
            (2, 55, 2, 101, 106, 2, mizar_checker::source_atomic_formula::SourceAtomicFormulaRecovery::Normal, "x = x", mizar_checker::source_atomic_formula::SourceAtomicFormulaKind::Equality),
            (3, 65, 1, 122, 127, 3, mizar_checker::source_atomic_formula::SourceAtomicFormulaRecovery::Normal, "x = x", mizar_checker::source_atomic_formula::SourceAtomicFormulaKind::Equality),
        ]
    );
    for (id, edge) in atomic.edges().iter() {
        assert_eq!(edge.formula().index(), id.index() / 2);
        assert_eq!(edge.ordinal(), id.index() % 2);
        assert_eq!(
            edge.role(),
            if id.index() % 2 == 0 {
                mizar_checker::source_atomic_formula::SourceAtomicEdgeRole::BuiltinLeftOperand
            } else {
                mizar_checker::source_atomic_formula::SourceAtomicEdgeRole::BuiltinRightOperand
            }
        );
        assert!(matches!(
            edge.target(),
            mizar_checker::source_atomic_formula::SourceAtomicTermTarget::Primary(term)
                if term.index() == id.index()
        ));
    }
    for (id, request) in atomic.requests().iter() {
        assert_eq!(request.formula().index(), id.index() / 2);
        assert_eq!(request.ordinal(), id.index() % 2);
        assert_eq!(
            request.kind(),
            mizar_checker::source_atomic_formula::SourceAtomicRequestKind::OperandExpectedType
        );
        assert_eq!(request.edge().map(|edge| edge.index()), Some(id.index()));
        assert_eq!(request.candidate(), None);
        assert_eq!(request.type_site(), None);
        assert_eq!(request.attribute(), None);
    }
    assert_eq!(output.reference_use_ordinals, [1; 8]);
    assert_eq!(output.typed_ast.nodes().len(), 77);
    assert_eq!(
        output.typed_ast.nodes().root().map(|root| root.index()),
        Some(76)
    );
    for (id, node) in output.typed_ast.nodes().iter() {
        assert_eq!(
            node.anchor,
            mizar_session::SourceAnchor::Range(ast.nodes()[id.index()].range)
        );
        assert_eq!(
            node.children
                .iter()
                .map(|child| child.index())
                .collect::<Vec<_>>(),
            ast.nodes()[id.index()]
                .children
                .iter()
                .map(|child| child.index())
                .collect::<Vec<_>>()
        );
        assert_eq!(
            node.recovery,
            mizar_checker::typed_ast::NodeRecoveryState::Normal
        );
    }
    assert_eq!(references.resolver_ast().nodes().len(), 77);
    assert_eq!(references.resolver_ast().nodes().root().index(), 76);
    for (id, node) in references.resolver_ast().nodes().iter() {
        assert_eq!(node.kind(), &ast.nodes()[id.index()].kind);
        assert_eq!(
            node.children()
                .iter()
                .map(|child| child.index())
                .collect::<Vec<_>>(),
            ast.nodes()[id.index()]
                .children
                .iter()
                .map(|child| child.index())
                .collect::<Vec<_>>()
        );
        assert_eq!(node.origin().source_id(), ast.source_id);
        assert_eq!(node.origin().module_id(), output.typed_ast.module_id());
        assert_eq!(node.origin().import_edge(), None);
        assert_eq!(node.origin().structural_path(), [id.index() as u32]);
        assert_eq!(
            node.origin().anchor(),
            &mizar_session::SourceAnchor::Range(ast.nodes()[id.index()].range)
        );
        assert!(!node.origin().is_recovered());
        assert_eq!(
            node.recovery(),
            mizar_resolve::resolved_ast::RecoveryState::Normal
        );
        if id.index() == 68 {
            assert_eq!(
                node.resolution(),
                mizar_resolve::resolved_ast::NodeResolutionState::Resolved
            );
            assert!(matches!(
                node.reference_key(),
                Some(mizar_resolve::resolved_ast::NodeReferenceKey::Label(id))
                    if id.index() == 0
            ));
        } else {
            assert_eq!(
                node.resolution(),
                mizar_resolve::resolved_ast::NodeResolutionState::NotApplicable
            );
            assert_eq!(node.reference_key(), None);
        }
    }
    let projection = references.label_projection();
    assert_eq!(projection.origin().structural_path(), [12]);
    assert_eq!((projection.declaration_range().start, projection.declaration_range().end), (77, 78));
    assert_eq!(projection.primary_spelling(), "A");
    assert_eq!(projection.module(), output.typed_ast.module_id());
    assert_eq!(projection.namespace().as_str(), output.typed_ast.module_id().path().as_str());
    assert_eq!(
        projection.kind(),
        mizar_resolve::resolved_ast::LabelKind::ProofStep
    );
    assert_eq!(
        projection.visibility(),
        mizar_resolve::env::Visibility::Private
    );
    assert_eq!(
        projection.export_status(),
        mizar_resolve::env::ExportStatus::LocalOnly
    );
    assert_eq!(projection.contribution(), owner.contribution());
    assert_eq!(projection.origin().source_id(), ast.source_id);
    assert_eq!(projection.origin().module_id(), output.typed_ast.module_id());
    assert_eq!(projection.origin().import_edge(), None);
    assert_eq!(
        projection.origin().anchor(),
        &mizar_session::SourceAnchor::Range(mizar_session::SourceRange {
            source_id: ast.source_id,
            start: 77,
            end: 78,
        })
    );
    assert!(!projection.origin().is_recovered());
    assert!(matches!(
        projection.source(),
        mizar_resolve::labels::LabelProjectionSource::CurrentModule {
            visible_after_ordinal: 1,
            proof_scope: Some(scope),
        } if scope.path() == [0]
    ));
    assert_eq!(references.reference_candidate().site().node().index(), 68);
    assert_eq!(
        (
            references.reference_candidate().site().range().start,
            references.reference_candidate().site().range().end,
            references.reference_candidate().site().spelling(),
            references.reference_candidate().ordinal(),
            references.reference_candidate().expectation(),
        ),
        (
            131,
            132,
            "A",
            3,
            mizar_resolve::resolved_ast::LabelExpectation::ProofOrTheorem,
        )
    );
    assert_eq!(references.reference_candidate().origin().source_id(), ast.source_id);
    assert_eq!(
        references.reference_candidate().origin().module_id(),
        output.typed_ast.module_id()
    );
    assert_eq!(references.reference_candidate().origin().import_edge(), None);
    assert_eq!(
        references.reference_candidate().origin().structural_path(),
        [68]
    );
    assert_eq!(
        references.reference_candidate().origin().anchor(),
        &mizar_session::SourceAnchor::Range(mizar_session::SourceRange {
            source_id: ast.source_id,
            start: 131,
            end: 132,
        })
    );
    assert!(!references.reference_candidate().origin().is_recovered());
    assert!(matches!(
        references.reference_candidate().scope(),
        mizar_resolve::labels::LabelReferenceScope::Unqualified {
            proof_scope: Some(scope),
        } if scope.path() == [0]
    ));
    assert_eq!(references.label_resolution().ids().len(), 1);
    assert!(references.label_resolution().diagnostics().is_empty());
    assert_eq!(
        references.resolver_ast().label_refs(),
        references.label_resolution().table()
    );
    let label_ref = references
        .label_resolution()
        .table()
        .get(references.label_resolution().ids()[0])
        .expect("resolved local label");
    assert_eq!(
        (
            label_ref.site().node().index(),
            label_ref.site().range().start,
            label_ref.site().range().end,
            label_ref.site().spelling(),
            label_ref.origin().source_id(),
            label_ref.origin().structural_path(),
            label_ref.recovery(),
        ),
        (
            68,
            131,
            132,
            "A",
            ast.source_id,
            &[68][..],
            mizar_resolve::resolved_ast::RecoveryState::Normal,
        )
    );
    assert_eq!(label_ref.origin().module_id(), output.typed_ast.module_id());
    assert_eq!(label_ref.origin().import_edge(), None);
    assert!(!label_ref.origin().is_recovered());
    assert!(matches!(
        label_ref.resolution(),
        mizar_resolve::resolved_ast::LabelResolution::Resolved(row)
            if row.origin() == projection.origin_path()
                && row.kind() == mizar_resolve::resolved_ast::LabelKind::ProofStep
                && (row.range().start, row.range().end) == (131, 132)
    ));
    let label = references
        .labels()
        .get(mizar_checker::source_statement::SourceStatementLabelId::new(0))
        .expect("statement label");
    assert_eq!(
        (
            label.statement().index(),
            label.context().index(),
            label.candidate().index(),
            label.origin_path(),
            label.proof_scope().path(),
            label.source_range().start,
            label.source_range().end,
            label.source_ordinal(),
            label.visible_after_ordinal(),
            label.spelling(),
            label.kind(),
            label.recovery(),
        ),
        (
            1,
            1,
            1,
            projection.origin_path(),
            &[0][..],
            77,
            78,
            0,
            1,
            "A",
            mizar_checker::source_statement::SourceStatementLabelKind::ProofStep,
            mizar_checker::source_statement::SourceStatementRecovery::Normal,
        )
    );
    let citation = references
        .citations()
        .get(mizar_checker::source_statement::SourceStatementCitationId::new(0))
        .expect("statement citation");
    assert_eq!(
        (
            citation.statement().index(),
            citation.context().index(),
            citation.label().index(),
            citation.label_ref().index(),
            citation.proof_scope().path(),
            citation.source_range().start,
            citation.source_range().end,
            citation.ordinal(),
            citation.kind(),
            citation.recovery(),
        ),
        (
            3,
            3,
            0,
            0,
            &[0][..],
            131,
            132,
            0,
            mizar_checker::source_statement::SourceStatementCitationKind::SimpleLocal,
            mizar_checker::source_statement::SourceStatementRecovery::Normal,
        )
    );
    assert_eq!(
        output.typed_ast.source_statement_references(),
        output.resolved.source_statement_references()
    );
    assert!(references
        .debug_text()
        .contains("resolver-ast root=76 nodes=77 name_refs=0 label_refs=1"));
}

#[test]
fn task258b1_lower_resolver_and_row_mutations_fail_closed_then_replay() {
    let (ast, module, _, symbols) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B1_TEXT, 258_410);
    let baseline =
        source_statement_output_with_source(&ast, module.clone(), &symbols, SOURCE_STATEMENT_B1_TEXT)
            .expect("Task258B1 selector")
            .expect("Task258B1 baseline");
    let baseline_typed = baseline.typed_ast.debug_text();
    for (label, expected, mutate) in [
        ("statement aggregate", "aggregate", 0usize),
        ("statement row", "statement", 1usize),
        ("stale projection", "dependency", 2usize),
        ("citation row", "citation", 3usize),
        ("owner row", "owner", 4usize),
        ("context row", "context", 5usize),
        ("input row", "input fact", 6usize),
        ("candidate row", "candidate", 7usize),
        ("label row", "label", 8usize),
        ("stale reference expectation", "dependency", 9usize),
        ("typed recovery", "statement", 10usize),
    ] {
        let error = source_statement_b1_output_with_mutation(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B1_TEXT,
            |inputs| match mutate {
                0 => inputs.statement.contexts.clear(),
                1 => inputs.statement.statements[2].source_ordinal = 9,
                2 => {
                    inputs.projection = inputs
                        .projection
                        .clone()
                        .with_visibility(mizar_resolve::env::Visibility::Public);
                }
                3 => inputs.reference_input.citations[0].ordinal = 1,
                4 => inputs.statement.owners[0].spelling.push_str("Drift"),
                5 => {
                    inputs.statement.contexts[2].binding_context =
                        mizar_checker::binding_env::BindingContextId::new(1);
                }
                6 => inputs.statement.input_facts[2].uses.swap(0, 1),
                7 => inputs.statement.candidate_facts[2].ordinal = 1,
                8 => inputs.reference_input.labels[0].source_ordinal = 1,
                9 => {
                    inputs.reference = inputs
                        .reference
                        .clone()
                        .with_expectation(
                            mizar_resolve::resolved_ast::LabelExpectation::Theorem,
                        );
                }
                10 => {
                    inputs.arena = mizar_checker::typed_ast::TypedArena::try_new(
                        inputs.arena.root(),
                        inputs
                            .arena
                            .iter()
                            .map(|(id, row)| {
                                let mut row = row.clone();
                                if id.index() == 50 {
                                    row.recovery =
                                        mizar_checker::typed_ast::NodeRecoveryState::Degraded;
                                }
                                row
                            })
                            .collect(),
                    )
                    .expect("recovered arena remains structurally valid");
                }
                _ => unreachable!(),
            },
        )
        .unwrap_or_else(|| panic!("{label} selector"))
        .expect_err(label);
        assert!(
            error.to_ascii_lowercase().contains(expected),
            "{label}: {error}"
        );
        let replay = source_statement_output_with_source(
            &ast,
            module.clone(),
            &symbols,
            SOURCE_STATEMENT_B1_TEXT,
        )
        .expect("replay selector")
        .expect("replay output");
        assert_eq!(replay.typed_ast.debug_text(), baseline_typed, "{label}");
    }
}

#[test]
fn task258b1_selector_rejects_exact_nested_statement_near_misses() {
    let (exact_ast, exact_module, _, exact_symbols) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B1_TEXT, 258_420);
    for (label, loaded) in [
        (
            "missing final LF",
            SOURCE_STATEMENT_B1_TEXT.trim_end_matches('\n').to_owned(),
        ),
        (
            "extra final LF",
            format!("{SOURCE_STATEMENT_B1_TEXT}\n"),
        ),
        (
            "whitespace byte drift",
            SOURCE_STATEMENT_B1_TEXT.replacen("  A:", " A:", 1),
        ),
        (
            "comment byte drift",
            SOURCE_STATEMENT_B1_TEXT.replacen("reserve x for set;", "reserve x for set; :: drift", 1),
        ),
        (
            "theorem name drift",
            SOURCE_STATEMENT_B1_TEXT.replacen(
                "FormulaStatementNestedContextSmoke",
                "FormulaStatementNestedContextOther",
                1,
            ),
        ),
        (
            "theorem role change",
            SOURCE_STATEMENT_B1_TEXT.replacen("theorem ", "scheme ", 1),
        ),
        (
            "theorem status change",
            SOURCE_STATEMENT_B1_TEXT.replacen("theorem ", "canceled theorem ", 1),
        ),
        (
            "reserve drift",
            SOURCE_STATEMENT_B1_TEXT.replacen("reserve x for set", "reserve y for set", 1),
        ),
        (
            "omitted reserve item",
            SOURCE_STATEMENT_B1_TEXT.replacen("reserve x for set;\n", "", 1),
        ),
        (
            "reordered top-level items",
            format!(
                "{}{}",
                &SOURCE_STATEMENT_B1_TEXT[19..],
                &SOURCE_STATEMENT_B1_TEXT[..19]
            ),
        ),
        (
            "omitted nested statement",
            SOURCE_STATEMENT_B1_TEXT.replacen("    thus x = x;\n", "", 1),
        ),
        (
            "reordered proof statements",
            SOURCE_STATEMENT_B1_TEXT.replacen(
                "  A: x = x proof\n    thus x = x;\n  end;\n  thus x = x by A;",
                "  thus x = x by A;\n  A: x = x proof\n    thus x = x;\n  end;",
                1,
            ),
        ),
        (
            "omitted outer proof block",
            SOURCE_STATEMENT_B1_TEXT.replacen(
                " proof\n  A: x = x proof\n    thus x = x;\n  end;\n  thus x = x by A;\nend;",
                ";",
                1,
            ),
        ),
        (
            "reordered proof blocks",
            SOURCE_STATEMENT_B1_TEXT.replacen(
                "  A: x = x proof\n    thus x = x;\n  end;\n  thus x = x by A;",
                "  thus x = x by A;\n  A: x = x proof\n    thus x = x;\n  end;",
                1,
            ),
        ),
        (
            "then statement",
            SOURCE_STATEMENT_B1_TEXT.replacen("  thus x = x by A;", "  then thus x = x by A;", 1),
        ),
        (
            "given statement",
            SOURCE_STATEMENT_B1_TEXT.replacen(
                "  A: x = x proof",
                "  given y being set such that x = x;\n  A: x = x proof",
                1,
            ),
        ),
        (
            "consider statement",
            SOURCE_STATEMENT_B1_TEXT.replacen(
                "  A: x = x proof",
                "  consider y being set such that x = x;\n  A: x = x proof",
                1,
            ),
        ),
        (
            "now block",
            SOURCE_STATEMENT_B1_TEXT.replacen(
                "  A: x = x proof",
                "  now\n    thus x = x;\n  end;\n  A: x = x proof",
                1,
            ),
        ),
        (
            "hereby block",
            SOURCE_STATEMENT_B1_TEXT.replacen(
                "  A: x = x proof",
                "  hereby\n    thus x = x;\n  end;\n  A: x = x proof",
                1,
            ),
        ),
        (
            "case block",
            SOURCE_STATEMENT_B1_TEXT.replacen(
                "  A: x = x proof",
                "  per cases;\n  suppose x = x;\n    thus x = x;\n  end;\n  A: x = x proof",
                1,
            ),
        ),
        (
            "suppose block",
            SOURCE_STATEMENT_B1_TEXT.replacen(
                "  A: x = x proof",
                "  suppose x = x;\n    thus x = x;\n  end;\n  A: x = x proof",
                1,
            ),
        ),
        (
            "iterative equality",
            SOURCE_STATEMENT_B1_TEXT.replacen("thus x = x;", "thus x = x = x;", 1),
        ),
        (
            "imported label citation",
            SOURCE_STATEMENT_B1_TEXT.replacen("by A", "by Other.A", 1),
        ),
        (
            "local label shadowing",
            SOURCE_STATEMENT_B1_TEXT.replacen(
                "    thus x = x;",
                "    A: x = x;\n    thus x = x;",
                1,
            ),
        ),
        (
            "missing nested proof",
            SOURCE_STATEMENT_B1_TEXT.replacen("  A: x = x proof", "  A: x = x", 1),
        ),
        (
            "recovered missing semicolon",
            SOURCE_STATEMENT_B1_TEXT.replacen("    thus x = x;", "    thus x = x", 1),
        ),
    ] {
        assert!(
            source_statement_output_with_source(
                &exact_ast,
                exact_module.clone(),
                &exact_symbols,
                &loaded,
            )
            .is_none(),
            "{label}"
        );
    }
    for (ordinal, (label, source)) in [
        (
            "forward citation",
            SOURCE_STATEMENT_B1_TEXT.replacen(
                "  A: x = x proof",
                "  x = x by A;\n  A: x = x proof",
                1,
            ),
        ),
        (
            "theorem citation",
            SOURCE_STATEMENT_B1_TEXT.replacen("by A", "by FormulaStatementNestedContextSmoke", 1),
        ),
        (
            "hence",
            SOURCE_STATEMENT_B1_TEXT.replacen("thus x = x by A", "hence x = x by A", 1),
        ),
        (
            "parenthesized equality",
            SOURCE_STATEMENT_B1_TEXT.replacen("thus x = x;", "thus (x = x);", 1),
        ),
        (
            "second label",
            SOURCE_STATEMENT_B1_TEXT.replacen("thus x = x;", "B: thus x = x;", 1),
        ),
        (
            "extra theorem",
            format!("{SOURCE_STATEMENT_B1_TEXT}theorem Extra: x = x;\n"),
        ),
        (
            "assumption",
            SOURCE_STATEMENT_B1_TEXT.replacen(
                "  A: x = x proof",
                "  A: x = x proof\n    assume x = x;",
                1,
            ),
        ),
        (
            "witness",
            SOURCE_STATEMENT_B1_TEXT.replacen(
                "    thus x = x;",
                "    take x;\n    thus x = x;",
                1,
            ),
        ),
        (
            "non-equality",
            SOURCE_STATEMENT_B1_TEXT.replacen("thus x = x;", "thus x <> x;", 1),
        ),
        (
            "composite formula",
            SOURCE_STATEMENT_B1_TEXT.replacen("thus x = x;", "thus x = x & x = x;", 1),
        ),
        (
            "qualified citation",
            SOURCE_STATEMENT_B1_TEXT.replacen("by A", "by Other.A", 1),
        ),
        (
            "citation moved inward",
            SOURCE_STATEMENT_B1_TEXT.replacen(
                "    thus x = x;",
                "    thus x = x by A;",
                1,
            ),
        ),
        (
            "explicit justification",
            SOURCE_STATEMENT_B1_TEXT.replacen("thus x = x;", "thus x = x by A;", 1),
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let (ast, module, _, symbols) =
            task253_ast_from_source_text(&source, 258_421 + ordinal);
        assert!(
            source_statement_output_with_source(&ast, module.clone(), &symbols, &source).is_none(),
            "{label}"
        );
        assert!(
            source_statement_output_with_source(
                &ast,
                module,
                &symbols,
                SOURCE_STATEMENT_B1_TEXT,
            )
            .is_none(),
            "{label} exact guard with wrong AST"
        );
    }
}

#[test]
fn task258b1_keeps_task258a_and_active_corpus_routes_isolated() {
    let (a_ast, a_module, _, a_symbols) =
        task253_ast_from_source_text(TASK258A_SOURCE, 258_430);
    let task_a =
        source_statement_output_with_source(&a_ast, a_module, &a_symbols, TASK258A_SOURCE)
            .expect("Task258A selector")
            .expect("Task258A output");
    assert!(task_a.typed_ast.source_statement_references().is_none());
    assert_eq!(task_a.reference_use_ordinals, [1, 2]);

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
    let plan = build_test_plan(&config).expect("Task258B1 isolation plan");
    let mut selected = Vec::new();
    for (ordinal, case) in active_type_elaboration_cases(&plan).enumerate() {
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .unwrap_or_else(|error| panic!("{} frontend failed: {error}", case.id.0));
        let source = frontend.source_text;
        let Some(ast) = frontend.ast else {
            continue;
        };
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        if !resolver.detail_keys.is_empty() {
            continue;
        }
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        if source_statement_output_with_source(&ast, resolver.module, &symbols, &source)
            .is_some_and(|result| {
                result.is_ok_and(|output| {
                    output
                        .typed_ast
                        .source_statement_references()
                        .is_some()
                })
            })
        {
            selected.push(case.id.0.clone());
        }
    }
    assert!(selected.is_empty(), "Task258B1 selected active cases: {selected:?}");
}

#[test]
fn task258b1_typed_final_ownership_clone_debug_and_empty_semantics_are_atomic() {
    let (ast, module, _, symbols) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B1_TEXT, 258_440);
    let first =
        source_statement_output_with_source(&ast, module.clone(), &symbols, SOURCE_STATEMENT_B1_TEXT)
            .expect("Task258B1 selector")
            .expect("Task258B1 output");
    let second =
        source_statement_output_with_source(&ast, module, &symbols, SOURCE_STATEMENT_B1_TEXT)
            .expect("Task258B1 replay selector")
            .expect("Task258B1 replay output");
    assert_eq!(first.typed_ast, first.typed_ast.clone());
    assert_eq!(first.resolved, first.resolved.clone());
    assert_eq!(first.typed_ast.debug_text(), second.typed_ast.debug_text());
    assert_eq!(first.resolved.debug_text(), second.resolved.debug_text());
    let debug = first.typed_ast.debug_text();
    let base = debug.find("source-statement-debug-v1").expect("base debug");
    let references = debug
        .find("source-statement-reference-debug-v1")
        .expect("reference debug");
    let nodes = debug.find("nodes:").expect("node debug");
    assert!(base < references && references < nodes);
    assert!(first.typed_ast.facts().is_empty());
    assert!(first.typed_ast.diagnostics().is_empty());
    assert!(first.resolved.checked_formulas().is_empty());
    assert!(first.resolved.statement_semantics().is_empty());
    assert!(first.resolved.checked_proofs().is_empty());
    assert!(first.resolved.checked_proof_nodes().is_empty());
    assert!(first.resolved.checked_terminal_goals().is_empty());
    let task248 = task248_real_output();
    let before = task248.typed_ast.debug_text();
    let statement = first
        .typed_ast
        .source_statement()
        .expect("statement")
        .clone();
    let statement_references = first
        .typed_ast
        .source_statement_references()
        .expect("statement references")
        .clone();
    assert_eq!(
        task248
            .typed_ast
            .clone()
            .with_source_statement(statement.clone())
            .expect_err("Task258B1 base must not use legacy installer"),
        mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement
    );
    assert_eq!(
        task248
            .typed_ast
            .clone()
            .with_source_statement_references(statement, statement_references)
            .expect_err("Task248 and Task258B1 owners must not coexist"),
        mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement
    );
    assert_eq!(task248.typed_ast.debug_text(), before);
}

#[test]
fn task258a_real_frontend_publishes_exact_statement_provenance_and_empty_semantics() {
    assert_eq!(TASK258A_SOURCE.len(), 81);
    assert!(TASK258A_SOURCE.ends_with('\n'));
    assert_eq!(TASK258A_SOURCE, SOURCE_STATEMENT_TEXT);
    let (ast, module, _, symbols) = task253_ast_from_source_text(TASK258A_SOURCE, 258_000);
    let extracted: SourceStatementExtraction = extract_source_reserved_variable_theorem_statement(
        &ast,
        module.clone(),
        &symbols,
        TASK258A_SOURCE,
    )
    .expect("Task258A exact source must select");
    let symbols = source_statement_resolver_env_for_test(&module, &symbols, &extracted)
        .expect("Task258A exact label projection must resolve");
    assert_eq!(
        (
            extracted.theorem_range.start,
            extracted.theorem_range.end,
            extracted.label_range.start,
            extracted.label_range.end,
            extracted.payload.formula_range.start,
            extracted.payload.formula_range.end,
            extracted.payload.left_range.start,
            extracted.payload.left_range.end,
            extracted.payload.right_range.start,
            extracted.payload.right_range.end,
        ),
        (19, 80, 27, 72, 74, 79, 74, 75, 78, 79)
    );
    assert_eq!(extracted.label_spelling, TASK258A_LABEL);
    assert_eq!(extracted.statement_spelling, TASK258A_STATEMENT_SPELLING);
    assert_eq!(
        (
            extracted.payload.left_spelling.as_str(),
            extracted.payload.right_spelling.as_str(),
            extracted.payload.left_lookup_ordinal,
            extracted.payload.right_lookup_ordinal,
        ),
        ("x", "x", 1, 2)
    );
    let namespace = NamespacePath::new(module.path().as_str());
    let theorem_candidates = symbols
        .symbols()
        .visible_candidates(&namespace, TASK258A_LABEL)
        .into_iter()
        .filter(|entry| entry.kind() == mizar_resolve::env::SymbolKind::Theorem)
        .collect::<Vec<_>>();
    let [theorem_candidate] = theorem_candidates.as_slice() else {
        panic!(
            "Task258A requires one real resolver theorem candidate, got {}",
            theorem_candidates.len()
        );
    };
    let prechecked_owner =
        mizar_checker::type_checker::CheckedStatementOwner::validate_exact_local_theorem(
            &symbols,
            theorem_candidate.symbol().clone(),
            ast.source_id,
            &module,
        )
        .expect("Task258A real resolver theorem must validate");
    assert_eq!(prechecked_owner.source_range(), extracted.theorem_range);

    let first: SourceStatementRouteOutput =
        source_statement_output_with_source(&ast, module.clone(), &symbols, TASK258A_SOURCE)
            .expect("Task258A exact selector")
            .unwrap_or_else(|error| panic!("Task258A exact route failed: {error}"));
    let second = source_statement_output_with_source(&ast, module, &symbols, TASK258A_SOURCE)
        .expect("Task258A repeated selector")
        .unwrap_or_else(|error| panic!("Task258A repeated route failed: {error}"));
    assert_eq!(
        (first.left_lookup_ordinal, first.right_lookup_ordinal),
        (1, 2)
    );

    let primary = first.typed_ast.source_term().expect("Task252 handoff");
    assert_eq!(
        (
            primary.terms().len(),
            primary.references().len(),
            primary.numeric_type_requests().len(),
        ),
        (2, 2, 0)
    );
    assert_eq!(
        primary
            .terms()
            .iter()
            .map(|(id, term)| (
                id.index(),
                term.source_range().start,
                term.source_range().end,
                term.source_ordinal(),
                term.context().index(),
                term.kind(),
                term.role(),
                term.recovery(),
                term.parent(),
                term.spelling().to_owned(),
            ))
            .collect::<Vec<_>>(),
        [
            (
                0,
                74,
                75,
                0,
                0,
                mizar_checker::source_term::SourcePrimaryTermKind::VariableReference,
                mizar_checker::source_term::SourcePrimaryTermRole::Value,
                mizar_checker::source_term::SourcePrimaryTermRecovery::Normal,
                None,
                "x".to_owned(),
            ),
            (
                1,
                78,
                79,
                1,
                0,
                mizar_checker::source_term::SourcePrimaryTermKind::VariableReference,
                mizar_checker::source_term::SourcePrimaryTermRole::Value,
                mizar_checker::source_term::SourcePrimaryTermRecovery::Normal,
                None,
                "x".to_owned(),
            ),
        ]
    );
    assert_eq!(
        primary
            .references()
            .iter()
            .map(|(id, reference)| (
                id.index(),
                reference.term().index(),
                reference.binding().index(),
                reference.use_ordinal(),
            ))
            .collect::<Vec<_>>(),
        [(0, 0, 0, 1), (1, 1, 0, 1)]
    );

    let atomic = first
        .typed_ast
        .source_atomic_formula()
        .expect("Task256 handoff");
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
        (1, 0, 0, 0, 0, 0, 0, 2, 2)
    );
    let formula = atomic
        .formulas()
        .get(mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(0))
        .expect("Task258A equality");
    assert_eq!(
        (
            formula.source_range().start,
            formula.source_range().end,
            formula.source_ordinal(),
            formula.context().index(),
            formula.kind(),
            formula.recovery(),
            formula.spelling(),
        ),
        (
            74,
            79,
            0,
            0,
            mizar_checker::source_atomic_formula::SourceAtomicFormulaKind::Equality,
            mizar_checker::source_atomic_formula::SourceAtomicFormulaRecovery::Normal,
            "x = x",
        )
    );
    assert_eq!(
        atomic
            .edges()
            .iter()
            .map(|(id, edge)| (
                id.index(),
                edge.formula().index(),
                edge.ordinal(),
                edge.role(),
                edge.target(),
            ))
            .collect::<Vec<_>>(),
        [
            (
                0,
                0,
                0,
                mizar_checker::source_atomic_formula::SourceAtomicEdgeRole::BuiltinLeftOperand,
                mizar_checker::source_atomic_formula::SourceAtomicTermTarget::Primary(
                    mizar_checker::source_term::SourcePrimaryTermId::new(0),
                ),
            ),
            (
                1,
                0,
                1,
                mizar_checker::source_atomic_formula::SourceAtomicEdgeRole::BuiltinRightOperand,
                mizar_checker::source_atomic_formula::SourceAtomicTermTarget::Primary(
                    mizar_checker::source_term::SourcePrimaryTermId::new(1),
                ),
            ),
        ]
    );
    assert_eq!(
        atomic
            .requests()
            .iter()
            .map(|(id, request)| (
                id.index(),
                request.formula().index(),
                request.ordinal(),
                request.kind(),
                request.edge().map(|id| id.index()),
                request.candidate(),
                request.type_site(),
                request.attribute(),
            ))
            .collect::<Vec<_>>(),
        [
            (
                0,
                0,
                0,
                mizar_checker::source_atomic_formula::SourceAtomicRequestKind::OperandExpectedType,
                Some(0),
                None,
                None,
                None,
            ),
            (
                1,
                0,
                1,
                mizar_checker::source_atomic_formula::SourceAtomicRequestKind::OperandExpectedType,
                Some(1),
                None,
                None,
                None,
            ),
        ]
    );

    let statement = first
        .typed_ast
        .source_statement()
        .expect("Task258A statement handoff");
    assert_eq!(
        (
            statement.owners().len(),
            statement.statements().len(),
            statement.contexts().len(),
            statement.input_facts().len(),
            statement.candidate_facts().len(),
        ),
        (1, 1, 1, 1, 1)
    );
    let owner = statement
        .owners()
        .get(mizar_checker::source_statement::SourceTheoremOwnerId::new(
            0,
        ))
        .expect("Task258A owner row");
    assert_eq!(owner.source_range(), extracted.theorem_range);
    assert_eq!(owner.site(), &extracted.theorem_site);
    assert_eq!(owner.spelling(), TASK258A_LABEL);
    assert_eq!(
        owner.role(),
        mizar_checker::source_statement::SourceTheoremRole::Theorem
    );
    assert_eq!(
        owner.status(),
        mizar_checker::source_statement::SourceTheoremStatus::Unmodified
    );
    assert_eq!(
        owner.recovery(),
        mizar_checker::source_statement::SourceStatementRecovery::Normal
    );
    assert_eq!(statement.checked_owner().symbol(), owner.symbol());
    assert_eq!(
        statement.checked_owner().source_range(),
        owner.source_range()
    );
    let statement_row = statement
        .statements()
        .get(mizar_checker::source_statement::SourceStatementId::new(0))
        .expect("Task258A statement row");
    assert_eq!(statement_row.owner().index(), 0);
    assert_eq!(statement_row.context().index(), 0);
    assert_eq!(statement_row.site(), &extracted.theorem_site);
    assert_eq!(statement_row.source_range(), extracted.theorem_range);
    assert_eq!(statement_row.source_ordinal(), 0);
    assert_eq!(statement_row.spelling(), TASK258A_STATEMENT_SPELLING);
    assert_eq!(
        statement_row.kind(),
        mizar_checker::source_statement::SourceStatementKind::TheoremProposition
    );
    assert_eq!(
        statement_row.recovery(),
        mizar_checker::source_statement::SourceStatementRecovery::Normal
    );
    assert_eq!(
        statement_row.formula(),
        mizar_checker::source_statement::SourceStatementFormulaTarget::Atomic(
            mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(0),
        )
    );
    let context = statement
        .contexts()
        .get(mizar_checker::source_statement::SourceStatementContextId::new(0))
        .expect("Task258A context row");
    assert_eq!(context.statement().index(), 0);
    assert_eq!(context.binding_context().index(), 0);
    assert_eq!(context.source_range(), extracted.theorem_range);
    assert_eq!(
        context
            .visible_bindings()
            .iter()
            .map(|binding| binding.index())
            .collect::<Vec<_>>(),
        [0]
    );
    let input_fact = statement
        .input_facts()
        .get(mizar_checker::source_statement::SourceStatementInputFactId::new(0))
        .expect("Task258A input fact");
    assert_eq!(input_fact.statement().index(), 0);
    assert_eq!(input_fact.context().index(), 0);
    assert_eq!(input_fact.ordinal(), 0);
    assert_eq!(input_fact.binding().index(), 0);
    assert_eq!(
        input_fact.kind(),
        mizar_checker::source_statement::SourceStatementInputFactKind::ReservedTypeGuard
    );
    assert_eq!(
        input_fact
            .uses()
            .iter()
            .map(|reference| reference.index())
            .collect::<Vec<_>>(),
        [0, 1]
    );
    let candidate = statement
        .candidate_facts()
        .get(mizar_checker::source_statement::SourceStatementCandidateFactId::new(0))
        .expect("Task258A candidate fact");
    assert_eq!(candidate.statement().index(), 0);
    assert_eq!(candidate.context().index(), 0);
    assert_eq!(candidate.ordinal(), 0);
    assert_eq!(
        candidate.kind(),
        mizar_checker::source_statement::SourceStatementCandidateFactKind::UnverifiedProposition
    );
    assert_eq!(candidate.formula(), statement_row.formula());
    let resolver_owner = symbols
        .symbols()
        .get(owner.symbol())
        .expect("Task258A resolver symbol");
    assert_eq!(
        resolver_owner.kind(),
        mizar_resolve::env::SymbolKind::Theorem
    );
    assert_eq!(
        resolver_owner.visibility(),
        mizar_resolve::env::Visibility::Public
    );
    assert_eq!(
        resolver_owner.export_status(),
        mizar_resolve::env::ExportStatus::Exported
    );
    assert_eq!(resolver_owner.contribution(), owner.contribution());
    let definition = symbols
        .definitions()
        .by_symbol(owner.symbol())
        .expect("Task258A theorem definition");
    assert_eq!(
        definition.kind(),
        mizar_resolve::env::DefinitionKind::Theorem
    );
    assert_eq!(definition.contribution(), owner.contribution());
    let contribution = symbols
        .contributions()
        .get(owner.contribution())
        .expect("Task258A local contribution");
    assert!(matches!(
        contribution.kind(),
        mizar_resolve::env::ContributionKind::LocalSource { .. }
    ));
    assert!(contribution.effects().symbols().contains(owner.symbol()));
    assert!(
        contribution
            .effects()
            .definitions()
            .contains(&definition.id())
    );
    assert!(contribution.effects().imports().is_empty());
    let labels = symbols.labels().by_contribution(owner.contribution());
    assert_eq!(labels.len(), 1);
    assert_eq!(labels[0].primary_spelling(), TASK258A_LABEL);
    assert_eq!(
        labels[0].kind(),
        mizar_resolve::resolved_ast::LabelKind::Theorem
    );
    assert_eq!(
        labels[0].visibility(),
        mizar_resolve::env::Visibility::Public
    );
    assert_eq!(
        labels[0].export_status(),
        mizar_resolve::env::ExportStatus::Exported
    );

    assert_eq!(statement.binding_env().bindings().len(), 1);
    assert_eq!(statement.binding_env().contexts().len(), 1);
    assert!(statement.binding_env().diagnostics().is_empty());
    let binding = statement
        .binding_env()
        .bindings()
        .get(mizar_checker::binding_env::BindingId::new(0))
        .expect("Task258A reserved binding");
    assert_eq!(binding.spelling, "x");
    assert_eq!(
        binding.kind,
        mizar_checker::binding_env::BindingKind::ReservedVariable
    );
    assert_eq!(
        binding.status,
        mizar_checker::binding_env::BindingStatus::Reserved
    );
    assert_eq!(binding.owner_context.index(), 0);
    assert_eq!(
        (
            binding.declaration_range.start,
            binding.declaration_range.end,
            binding.visible_after_ordinal,
        ),
        (8, 9, 0)
    );
    assert!(matches!(
        binding.type_site,
        mizar_checker::binding_env::BindingTypeSite::Source(range)
            if (range.start, range.end) == (14, 17)
    ));
    assert_eq!(
        statement.binding_fingerprint(),
        statement.binding_env().debug_text()
    );
    assert_eq!(statement.primary_term_fingerprint(), primary.debug_text());
    assert_eq!(statement.atomic_formula_fingerprint(), atomic.debug_text());
    assert_eq!(
        first.typed_ast.source_statement(),
        first.resolved.source_statement()
    );
    assert_eq!(first.typed_ast.source_term(), first.resolved.source_term());
    assert_eq!(
        first.typed_ast.source_atomic_formula(),
        first.resolved.source_atomic_formula()
    );
    assert!(first.typed_ast.source_context().is_none());
    assert!(first.typed_ast.types().is_empty());
    assert!(first.typed_ast.facts().is_empty());
    assert!(first.typed_ast.diagnostics().is_empty());
    assert!(first.resolved.expr_metadata().is_empty());
    assert!(first.resolved.cluster_facts().is_empty());
    assert!(first.resolved.diagnostics().is_empty());
    assert!(first.resolved.checked_formulas().is_empty());
    assert!(first.resolved.statement_semantics().is_empty());
    assert!(first.resolved.checked_proofs().is_empty());
    assert!(first.resolved.checked_proof_nodes().is_empty());
    assert!(first.resolved.checked_terminal_goals().is_empty());
    assert_eq!(first.typed_ast.debug_text(), second.typed_ast.debug_text());
    assert_eq!(first.resolved.debug_text(), second.resolved.debug_text());
    let typed_debug = first.typed_ast.debug_text();
    let primary_at = typed_debug
        .find("source-primary-term-debug-v1")
        .expect("typed Task252 debug");
    let atomic_at = typed_debug
        .find("source-atomic-formula-debug-v1")
        .expect("typed Task256 debug");
    let statement_at = typed_debug
        .find("source-statement-debug-v1")
        .expect("typed Task258A debug");
    let nodes_at = typed_debug.find("nodes:").expect("typed node debug");
    assert!(primary_at < atomic_at && atomic_at < statement_at && statement_at < nodes_at);
    let typed_clone = first.typed_ast.clone();
    assert_eq!(typed_clone, first.typed_ast);
    assert_eq!(typed_clone.source_statement(), Some(statement));
    let resolved_clone = first.resolved.clone();
    assert_eq!(resolved_clone, first.resolved);
    assert_eq!(resolved_clone.source_statement(), Some(statement));
}

#[test]
fn task258a_dependency_and_row_corruption_fail_atomically_then_replay() {
    let (ast, module, _, symbols) = task253_ast_from_source_text(TASK258A_SOURCE, 258_100);
    let baseline =
        source_statement_output_with_source(&ast, module.clone(), &symbols, TASK258A_SOURCE)
            .expect("Task258A baseline selector")
            .expect("Task258A baseline output");
    let baseline_typed = baseline.typed_ast.debug_text();
    let baseline_resolved = baseline.resolved.debug_text();
    let imported_label_error = source_statement_output_with_resolver_mutation(
        &ast,
        module.clone(),
        &symbols,
        TASK258A_SOURCE,
        statement_env_with_imported_label,
    )
    .expect("Task258A imported-label selector")
    .expect_err("imported theorem label provenance must fail");
    assert!(
        imported_label_error
            .to_ascii_lowercase()
            .contains("owner"),
        "unexpected imported-label error: {imported_label_error}"
    );

    let (lower_ast, lower_module, lower_symbols) =
        task252_real_ast("pass_type_elaboration_formula_predicate_chain_segment_payload_001");
    let lower = source_atomic_formula_output_with_source(
        &lower_ast,
        lower_module,
        &lower_symbols,
        TASK258A_LOWER_SOURCE,
    )
    .expect("Task257C1 lower selector")
    .expect("Task257C1 lower profile");
    let wrong_primary = lower
        .typed_ast
        .source_term()
        .expect("Task257C1 primary handoff")
        .clone();
    let wrong_atomic = lower
        .typed_ast
        .source_atomic_formula()
        .expect("Task257C1 atomic handoff")
        .clone();
    let replay = Task258AReplayFixture {
        ast: &ast,
        module,
        symbols: &symbols,
        baseline_typed: &baseline_typed,
        baseline_resolved: &baseline_resolved,
    };

    assert_task258a_mutation_rejects_then_replays(
        &replay,
        "wrong primary profile",
        "dependency",
        move |input| input.primary = wrong_primary,
    );
    assert_task258a_mutation_rejects_then_replays(
        &replay,
        "wrong atomic profile",
        "dependency",
        move |input| input.atomic = wrong_atomic,
    );
    assert_task258a_mutation_rejects_then_replays(
        &replay,
        "missing owner aggregate",
        "aggregate",
        |input| input.statement.owners.clear(),
    );
    assert_task258a_mutation_rejects_then_replays(
        &replay,
        "invalid statement ordinal",
        "statement",
        |input| input.statement.statements[0].source_ordinal = 1,
    );
    assert_task258a_mutation_rejects_then_replays(
        &replay,
        "swapped input reference ids",
        "input fact",
        |input| input.statement.input_facts[0].uses.swap(0, 1),
    );
}

#[derive(Debug, Clone, Copy)]
enum Task258B2ResolverMutation {
    Imported,
    Missing,
    Duplicate,
    WrongPath,
    WrongKind,
    Private,
    LocalOnly,
    Recovered,
}

fn task258b2_mutate_resolver(
    symbols: SymbolEnv,
    mutation: Task258B2ResolverMutation,
) -> SymbolEnv {
    if matches!(mutation, Task258B2ResolverMutation::Imported) {
        return statement_env_with_imported_label(symbols);
    }
    let label = symbols
        .labels()
        .iter()
        .next()
        .expect("Task258B2 exact theorem label")
        .clone();
    let mut labels = mizar_resolve::env::LabelIndex::new();
    if !matches!(mutation, Task258B2ResolverMutation::Missing) {
        let origin_path = if matches!(mutation, Task258B2ResolverMutation::WrongPath) {
            mizar_resolve::resolved_ast::LabelOriginPath::new(
                "task258b2::wrong::theorem::origin",
            )
        } else {
            label.origin_path().clone()
        };
        let kind = if matches!(mutation, Task258B2ResolverMutation::WrongKind) {
            mizar_resolve::resolved_ast::LabelKind::ProofStep
        } else {
            label.kind()
        };
        let origin = if matches!(mutation, Task258B2ResolverMutation::Recovered) {
            label.origin().clone().recovered()
        } else {
            label.origin().clone()
        };
        let visibility = if matches!(mutation, Task258B2ResolverMutation::Private) {
            mizar_resolve::env::Visibility::Private
        } else {
            label.visibility()
        };
        let export_status = if matches!(mutation, Task258B2ResolverMutation::LocalOnly) {
            mizar_resolve::env::ExportStatus::LocalOnly
        } else {
            label.export_status()
        };
        labels.insert(
            mizar_resolve::env::LabelEntry::new(
                origin_path,
                kind,
                label.namespace().clone(),
                label.primary_spelling(),
                origin,
                label.contribution(),
            )
            .with_visibility(visibility)
            .with_export_status(export_status),
        );
        if matches!(mutation, Task258B2ResolverMutation::Duplicate) {
            labels.insert(
                mizar_resolve::env::LabelEntry::new(
                    mizar_resolve::resolved_ast::LabelOriginPath::new(
                        "task258b2::duplicate::theorem::origin",
                    ),
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
    }
    SymbolEnv::new(
        symbols.module_id().clone(),
        mizar_resolve::env::SymbolEnvIndexes {
            imports: symbols.imports().clone(),
            exports: symbols.exports().clone(),
            symbols: symbols.symbols().clone(),
            labels,
            definitions: symbols.definitions().clone(),
            overloads: symbols.overloads().clone(),
            registrations: symbols.registrations().clone(),
            lexical_summaries: symbols.lexical_summaries().clone(),
            namespace_graph: symbols.namespace_graph().clone(),
            declaration_dependencies: symbols.declaration_dependencies().clone(),
            contributions: symbols.contributions().clone(),
            module_summaries: symbols.module_summaries().clone(),
        },
    )
}

fn task258b3m1_env_with_witness_symbol(symbols: SymbolEnv) -> SymbolEnv {
    let module = symbols.module_id().clone();
    let namespace = mizar_resolve::env::NamespacePath::new(module.path().as_str());
    let contribution = symbols
        .contributions()
        .iter()
        .next()
        .expect("Task258B3M1 local contribution")
        .id();
    let label = symbols
        .labels()
        .iter()
        .next()
        .expect("Task258B3M1 theorem label");
    let source_range = mizar_session::SourceRange {
        source_id: label.origin().source_id(),
        start: 84,
        end: 85,
    };
    let symbol = mizar_resolve::resolved_ast::SymbolId::new(
        module.clone(),
        mizar_resolve::resolved_ast::LocalSymbolId::new("Witness/y/0"),
        mizar_resolve::resolved_ast::FullyQualifiedName::new(format!(
            "{}::y/0",
            module.path().as_str()
        )),
    );
    let mut symbol_index = symbols.symbols().clone();
    symbol_index.insert(
        mizar_resolve::env::SymbolEntry::new(
            symbol.clone(),
            mizar_resolve::env::SymbolKind::Builtin,
            namespace,
            "y",
            mizar_resolve::resolved_ast::SemanticOrigin::new(
                source_range.source_id,
                module.clone(),
                mizar_session::SourceAnchor::Range(source_range),
                vec![2, 1, 0],
            ),
            contribution,
        )
        .with_visibility(mizar_resolve::env::Visibility::Public)
        .with_export_status(mizar_resolve::env::ExportStatus::Exported),
    );
    let mut contributions = symbols.contributions().clone();
    contributions.add_symbol(contribution, symbol);
    SymbolEnv::new(
        module,
        mizar_resolve::env::SymbolEnvIndexes {
            imports: symbols.imports().clone(),
            exports: symbols.exports().clone(),
            symbols: symbol_index,
            labels: symbols.labels().clone(),
            definitions: symbols.definitions().clone(),
            overloads: symbols.overloads().clone(),
            registrations: symbols.registrations().clone(),
            lexical_summaries: symbols.lexical_summaries().clone(),
            namespace_graph: symbols.namespace_graph().clone(),
            declaration_dependencies: symbols.declaration_dependencies().clone(),
            contributions,
            module_summaries: symbols.module_summaries().clone(),
        },
    )
}

fn statement_env_with_imported_label(symbols: SymbolEnv) -> SymbolEnv {
    let label = symbols
        .labels()
        .iter()
        .next()
        .expect("Task258A exact label")
        .clone();
    let mut nodes = mizar_resolve::resolved_ast::ResolvedArenaBuilder::new();
    let owner = nodes
        .push(mizar_resolve::resolved_ast::ResolvedNode::new(
            mizar_syntax::SurfaceNodeKind::ImportAliasDecl,
            Vec::new(),
            label.origin().clone(),
        ))
        .expect("Task258A imported-label owner");
    let source_range = mizar_session::SourceRange {
        source_id: label.origin().source_id(),
        start: 0,
        end: 1,
    };
    let mut imports = mizar_resolve::resolved_ast::ResolvedImports::new();
    let import = imports.push_import(mizar_resolve::resolved_ast::ResolvedImport::new(
        owner,
        source_range,
        "import statement.fixture;",
        None,
        mizar_resolve::resolved_ast::ImportResolution::Resolved(
            symbols.module_id().clone(),
        ),
        label.origin().clone(),
    ));
    let mut labels = mizar_resolve::env::LabelIndex::new();
    labels.insert(
        mizar_resolve::env::LabelEntry::new(
            label.origin_path().clone(),
            label.kind(),
            label.namespace().clone(),
            label.primary_spelling(),
            label.origin().clone().with_import_edge(import),
            label.contribution(),
        )
        .with_visibility(label.visibility())
        .with_export_status(label.export_status()),
    );
    SymbolEnv::new(
        symbols.module_id().clone(),
        mizar_resolve::env::SymbolEnvIndexes {
            imports: symbols.imports().clone(),
            exports: symbols.exports().clone(),
            symbols: symbols.symbols().clone(),
            labels,
            definitions: symbols.definitions().clone(),
            overloads: symbols.overloads().clone(),
            registrations: symbols.registrations().clone(),
            lexical_summaries: symbols.lexical_summaries().clone(),
            namespace_graph: symbols.namespace_graph().clone(),
            declaration_dependencies: symbols.declaration_dependencies().clone(),
            contributions: symbols.contributions().clone(),
            module_summaries: symbols.module_summaries().clone(),
        },
    )
}

#[test]
fn task258a_selector_rejects_loaded_named_recovered_subtree_and_active_near_misses() {
    let (exact_ast, exact_module, _, exact_symbols) =
        task253_ast_from_source_text(TASK258A_SOURCE, 258_200);
    for (label, loaded_source) in [
        (
            "missing final LF",
            TASK258A_SOURCE.trim_end_matches('\n').to_owned(),
        ),
        (
            "loaded source has an extra final LF",
            format!("{TASK258A_SOURCE}\n"),
        ),
        (
            "loaded source has byte-different whitespace",
            TASK258A_SOURCE.replacen(": x = x;", ":  x = x;", 1),
        ),
    ] {
        assert!(
            source_statement_output_with_source(
                &exact_ast,
                exact_module.clone(),
                &exact_symbols,
                &loaded_source,
            )
            .is_none(),
            "{label}"
        );
    }

    for (ordinal, (label, source)) in [
        (
            "named theorem near miss",
            TASK258A_SOURCE.replacen(
                TASK258A_LABEL,
                "FormulaStatementReservedVariableEqualityNearMiss",
                1,
            ),
        ),
        (
            "recovered equality",
            TASK258A_SOURCE.replacen("x = x;", "x = ;", 1),
        ),
        (
            "parenthesized formula subtree",
            TASK258A_SOURCE.replacen("x = x;", "(x = x);", 1),
        ),
        (
            "composite formula subtree",
            TASK258A_SOURCE.replacen("x = x;", "(x = x) & (x = x);", 1),
        ),
        (
            "multiple theorem items",
            format!("{TASK258A_SOURCE}theorem ExtraStatement: x = x;\n"),
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let (ast, module, _, symbols) = task253_ast_from_source_text(&source, 258_210 + ordinal);
        assert!(
            source_statement_output_with_source(&ast, module.clone(), &symbols, &source).is_none(),
            "{label} must fail its real loaded-source selector"
        );
        assert!(
            source_statement_output_with_source(&ast, module, &symbols, TASK258A_SOURCE).is_none(),
            "{label} must also fail when the exact loaded-source guard is supplied"
        );
    }

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
    let plan = build_test_plan(&config).expect("Task258A isolation plan");
    let mut selected = Vec::new();
    for (ordinal, case) in active_type_elaboration_cases(&plan).enumerate() {
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .unwrap_or_else(|error| panic!("{} frontend failed: {error}", case.id.0));
        let source = frontend.source_text;
        let Some(ast) = frontend.ast else {
            continue;
        };
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        if !resolver.detail_keys.is_empty() {
            continue;
        }
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        if source_statement_output_with_source(&ast, resolver.module, &symbols, &source).is_some() {
            selected.push(case.id.0.clone());
        }
    }
    assert!(
        selected.is_empty(),
        "dormant Task258A route must not select an active type-elaboration case: {selected:?}"
    );
}

#[test]
fn task258a_typed_resolved_ownership_and_task248_first_exclusion_are_atomic() {
    let (ast, module, _, symbols) = task253_ast_from_source_text(TASK258A_SOURCE, 258_300);
    let output = source_statement_output_with_source(&ast, module, &symbols, TASK258A_SOURCE)
        .expect("Task258A ownership selector")
        .expect("Task258A ownership output");
    let statement = output
        .typed_ast
        .source_statement()
        .expect("Task258A typed owner")
        .clone();
    assert_eq!(
        output.typed_ast.source_statement(),
        output.resolved.source_statement()
    );
    assert_eq!(
        output.typed_ast.source_term(),
        output.resolved.source_term()
    );
    assert_eq!(
        output.typed_ast.source_atomic_formula(),
        output.resolved.source_atomic_formula()
    );

    let task248 = task248_real_output();
    assert!(task248.typed_ast.source_context().is_some());
    assert!(task248.typed_ast.source_statement().is_none());
    let before = task248.typed_ast.debug_text();
    assert_eq!(
        task248
            .typed_ast
            .clone()
            .with_source_statement(statement)
            .expect_err("Task248-first/Task258A-second must fail"),
        mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement
    );
    assert_eq!(task248.typed_ast.debug_text(), before);
    assert!(task248.typed_ast.source_statement().is_none());
    assert_eq!(
        task248.typed_ast.source_context(),
        task248.resolved.source_context()
    );
    assert_eq!(task248.typed_ast.debug_text(), before);

    let replay = source_statement_output_with_source(
        &ast,
        output.typed_ast.module_id().clone(),
        &symbols,
        TASK258A_SOURCE,
    )
    .expect("Task258A replay selector")
    .expect("Task258A replay output");
    assert_eq!(replay.typed_ast.debug_text(), output.typed_ast.debug_text());
    assert_eq!(replay.resolved.debug_text(), output.resolved.debug_text());
}

struct Task258AReplayFixture<'a> {
    ast: &'a SurfaceAst,
    module: ResolverModuleId,
    symbols: &'a SymbolEnv,
    baseline_typed: &'a str,
    baseline_resolved: &'a str,
}

fn assert_task258a_mutation_rejects_then_replays(
    fixture: &Task258AReplayFixture<'_>,
    label: &str,
    expected_error_fragment: &str,
    mutate: impl FnOnce(&mut SourceStatementRouteInputs),
) {
    let error = source_statement_output_with_source_and_mutation(
        fixture.ast,
        fixture.module.clone(),
        fixture.symbols,
        TASK258A_SOURCE,
        mutate,
    )
    .unwrap_or_else(|| panic!("{label} must preserve the exact selector"))
    .expect_err(label);
    assert!(
        error.to_ascii_lowercase().contains(expected_error_fragment),
        "{label}: expected {expected_error_fragment:?}, got {error:?}"
    );
    let replay = source_statement_output_with_source(
        fixture.ast,
        fixture.module.clone(),
        fixture.symbols,
        TASK258A_SOURCE,
    )
    .unwrap_or_else(|| panic!("{label} replay must preserve the selector"))
    .unwrap_or_else(|error| panic!("{label} replay failed: {error}"));
    assert_eq!(
        replay.typed_ast.debug_text(),
        fixture.baseline_typed,
        "{label}"
    );
    assert_eq!(
        replay.resolved.debug_text(),
        fixture.baseline_resolved,
        "{label}"
    );
}
