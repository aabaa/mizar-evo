#[test]
fn task258b3m2b1_real_frontend_freezes_parenthesized_witness_contract() {
    assert_eq!(SOURCE_STATEMENT_B3M2B1_TEXT.len(), 113);
    assert_eq!(
        sha256_text(SOURCE_STATEMENT_B3M2B1_TEXT),
        "f09815b49d1b4598218f656a366ef73ec0dffd1f581a1018f07aa2ebcf410bf2"
    );
    assert!(SOURCE_STATEMENT_B3M2B1_TEXT.ends_with('\n'));
    let (ast, module, _, symbols, diagnostic_count) =
        task253_ast_from_source_text_with_diagnostic_count(SOURCE_STATEMENT_B3M2B1_TEXT, 258_910);
    assert_eq!(diagnostic_count, 0);
    assert_eq!(
        (ast.nodes().len(), ast.root().expect("root").index()),
        (53, 52)
    );
    assert!(ast.nodes().iter().all(|node| !node.recovered));
    let extracted: SourceStatementB3M2B1Extraction =
        extract_parenthesized_witness_source_statement(&ast, SOURCE_STATEMENT_B3M2B1_TEXT)
            .expect("Task258B3M2B1 exact parser shape");
    assert_eq!(
        (
            extracted.theorem_site.node().index(),
            extracted.theorem_range.start,
            extracted.theorem_range.end,
            extracted.label_range.start,
            extracted.label_range.end,
            extracted.take_site.node().index(),
            extracted.take_range.start,
            extracted.take_range.end,
        ),
        (49, 19, 112, 27, 68, 39, 84, 93)
    );
    assert_eq!(
        (
            extracted.witness_site.node().index(),
            extracted.witness_range.start,
            extracted.witness_range.end,
            extracted.proof_range.start,
            extracted.proof_range.end,
        ),
        (38, 89, 92, 76, 111)
    );
    assert_eq!(
        extracted
            .statement_sites
            .iter()
            .map(|site| site.node().index())
            .collect::<Vec<_>>(),
        [49, 47]
    );
    assert_eq!(
        extracted
            .formula_sites
            .iter()
            .map(|site| site.node().index())
            .collect::<Vec<_>>(),
        [32, 44]
    );
    assert_eq!(
        extracted
            .term_sites
            .iter()
            .map(|site| site.node().index())
            .collect::<Vec<_>>(),
        [28, 30, 36, 40, 42]
    );
    assert_eq!(
        extracted
            .term_ranges
            .iter()
            .map(|range| (range.start, range.end))
            .collect::<Vec<_>>(),
        [
            (70, 71),
            (74, 75),
            (89, 92),
            (90, 91),
            (101, 102),
            (105, 106),
        ]
    );

    assert_eq!(extracted.statement_ranges, [ast.nodes()[49].range, ast.nodes()[47].range]);
    assert_eq!(extracted.formula_ranges, [ast.nodes()[32].range, ast.nodes()[44].range]);
    let (source, resolver) = mizar_resolve::symbols::source_theorem_module_original_inputs(&ast, &module).unwrap().unwrap();
    let (scope, labels, resolution) = mizar_resolve::symbols::source_proof_inputs(&source, &resolver).unwrap();
    assert!(resolver.labels().is_empty());
    assert_eq!((labels.projections().len(), resolution.index().len()), (1, 1));
    assert!(resolver.imports().is_empty());
    let (typed, resolved) = task258_native_output(&ast, &module);
    let statement = typed.source_statement().expect("statement");
    let primary = typed.source_term().expect("primary");
    let atomic = typed.source_atomic_formula().expect("atomic");
    let witnesses = typed
        .source_statement_witnesses()
        .expect("witness");
    assert_eq!(
        (
            statement.binding_env().contexts().len(),
            statement.binding_env().bindings().len(),
            statement.binding_env().diagnostics().len(),
            primary.terms().len(),
            primary.references().len(),
            primary.numeric_type_requests().len(),
            atomic.formulas().len(),
            atomic.edges().len(),
            atomic.requests().len(),
        ),
        (2, 1, 0, 6, 5, 0, 2, 4, 4)
    );
    assert_eq!(
        (
            atomic.wrappers().len(),
            atomic.predicate_segments().len(),
            atomic.predicate_heads().len(),
            atomic.candidates().len(),
            atomic.type_sites().len(),
            atomic.attributes().len(),
        ),
        (0, 0, 0, 0, 0, 0)
    );
    assert_eq!(
        (
            statement.owners().len(),
            statement.statements().len(),
            statement.contexts().len(),
            statement.input_facts().len(),
            statement.candidate_facts().len(),
            witnesses.witnesses().len(),
            witnesses.names().len(),
        ),
        (1, 2, 2, 2, 2, 1, 0)
    );
    for (index, statement_row) in statement.statements().iter() {
        assert_eq!(
            (
                statement_row.site().node().index(),
                statement_row.source_range().start,
                statement_row.source_range().end,
                statement_row.source_ordinal(),
                statement_row.context().index(),
            ),
            [(49, 19, 112, 0, 0), (47, 96, 107, 2, 1)][index.index()],
            "statement row {}",
            index.index()
        );
    }
    let mut source_partition = statement
        .statements()
        .iter()
        .map(|(_, row)| row.source_ordinal())
        .chain(
            witnesses
                .witnesses()
                .iter()
                .map(|(_, row)| row.source_ordinal()),
        )
        .collect::<Vec<_>>();
    source_partition.sort_unstable();
    assert_eq!(source_partition, [0, 1, 2]);
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
                start: 76,
                end: 111,
            },
        }
    );
    assert_eq!(proof.parent.map(|id| id.index()), Some(0));
    assert_eq!(
        proof
            .lexical_scope
            .as_ref()
            .map(|scope| scope.path().to_vec()),
        Some(vec![4])
    );
    assert!(proof.bindings.is_empty());
    assert_eq!(
        proof
            .visible_bindings
            .iter()
            .map(|id| id.index())
            .collect::<Vec<_>>(),
        [0]
    );

    for index in 0..6 {
        let term = primary
            .terms()
            .get(mizar_checker::source_term::SourcePrimaryTermId::new(index))
            .expect("term");
        assert_eq!(term.site().node().index(), [28, 30, 36, 34, 40, 42][index]);
        assert_eq!(
            (term.source_range().start, term.source_range().end),
            [
                (70, 71),
                (74, 75),
                (89, 92),
                (90, 91),
                (101, 102),
                (105, 106),
            ][index]
        );
        assert_eq!(term.source_ordinal(), index);
        assert_eq!(term.context().index(), [0, 0, 1, 1, 1, 1][index]);
        assert_eq!(term.spelling(), if index == 2 { "( x )" } else { "x" });
        assert_eq!(
            term.kind(),
            if index == 2 {
                mizar_checker::source_term::SourcePrimaryTermKind::Parenthesized
            } else {
                mizar_checker::source_term::SourcePrimaryTermKind::VariableReference
            }
        );
        assert_eq!(
            term.parent().map(|id| id.index()),
            if index == 3 { Some(2) } else { None }
        );
    }
    for (reference_index, term_index) in [0, 1, 3, 4, 5].into_iter().enumerate() {
        let reference = primary
            .references()
            .get(mizar_checker::source_term::SourcePrimaryTermReferenceId::new(reference_index))
            .expect("reference");
        assert_eq!(reference.term().index(), term_index);
        assert_eq!(reference.binding().index(), 0);
        assert_eq!(reference.use_ordinal(), reference_index);
        assert_eq!(
            reference.lexical_scope().map(|scope| scope.path().to_vec()),
            Some(scope.references()[reference_index].scope().path().to_vec())
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
        [0, 1, 4, 5]
    );
    assert_eq!(
        statement
            .input_facts()
            .iter()
            .map(|(_, fact)| fact.uses().iter().map(|id| id.index()).collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        [vec![0, 1], vec![3, 4]]
    );
    let witness = witnesses
        .witnesses()
        .get(mizar_checker::source_statement::SourceStatementWitnessId::new(0))
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
                mizar_checker::source_term::SourcePrimaryTermId::new(2),
            ),
            39,
            84,
            93,
            38,
            89,
            92,
        )
    );
    assert_eq!(
        (
            witness.source_ordinal(),
            witness.ordinal(),
            witness.spelling(),
            witness.kind(),
            witness.name(),
        ),
        (
            1,
            0,
            "( x )",
            mizar_checker::source_statement::SourceStatementWitnessKind::Unnamed,
            None,
        )
    );
    for index in 0..53 {
        let surface = ast.nodes().get(index).expect("surface node");
        let node = typed
            .nodes()
            .node(mizar_checker::typed_ast::TypedNodeId::new(index))
            .expect("typed node");
        let (original_id, original) = source.arena().iter().nth(index).unwrap();
        assert_eq!(node.resolved_node, Some(original_id));
        let expected_kind = match original.kind() {
            mizar_syntax::SurfaceNodeKind::TermReference => "source.term.variable-reference".to_owned(),
            mizar_syntax::SurfaceNodeKind::ParenthesizedTerm => "source.term.parenthesized".to_owned(),
            mizar_syntax::SurfaceNodeKind::BuiltinPredicateApplication => "source.formula.atomic.equality".to_owned(),
            kind => format!("{kind:?}"),
        };
        assert_eq!(node.kind.as_str(), expected_kind);
        assert_eq!(node.recovery, mizar_checker::typed_ast::NodeRecoveryState::Normal);
        assert_eq!(
            node.anchor,
            mizar_session::SourceAnchor::Range(surface.range)
        );
        assert_eq!(
            node
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
    }
    assert_eq!(primary.references().iter().map(|(_, reference)| reference.use_ordinal()).collect::<Vec<_>>(), [0, 1, 2, 3, 4]);
    assert!(source_statement_output_with_source(&ast, module.clone(), &symbols, SOURCE_STATEMENT_B3M2B1_TEXT).is_none());
    assert_eq!(source_statement_transport_detail_keys(&ast, module, &symbols, SOURCE_STATEMENT_B3M2B1_TEXT), None);
    assert_eq!(
        typed.source_statement(),
        resolved.source_statement()
    );
    assert_eq!(
        typed.source_statement_witnesses(),
        resolved.source_statement_witnesses()
    );
    assert_eq!(
        typed.source_term(),
        resolved.source_term()
    );
    assert_eq!(
        typed.source_atomic_formula(),
        resolved.source_atomic_formula()
    );
}

#[test]
fn task258b3m2b1_validation_precedence_mutation_and_replay_fail_closed() {
    let (ast, module, _, _) = task253_ast_from_source_text(SOURCE_STATEMENT_B3M2B1_TEXT, 258_911);
    let (source, symbols) = mizar_resolve::symbols::source_theorem_module_original_inputs(&ast, &module).unwrap().unwrap();
    let (scope, labels, resolution) = mizar_resolve::symbols::source_proof_inputs(&source, &symbols).unwrap();
    assert!(symbols.labels().is_empty(), "native declaration environment has no legacy label rows");
    assert_eq!((labels.projections().len(), resolution.index().len()), (1, 1));
    let (_, _, base) = mizar_checker::type_checker::TermFormulaChecker::check_source_theorem_equalities(&source, &symbols, &scope).expect("pristine native lower baseline");
    let baseline = task258_native_output(&ast, &module);
    let installed = base.clone().with_source_statement_from_original(&source, &symbols, &scope).expect("pristine original installation");
    assert_eq!(installed, baseline.0);
    assert_eq!(assemble_empty_resolved_typed_ast(&installed, Vec::new()).unwrap(), baseline.1);
    let owner = installed.source_statement().unwrap().checked_owner().symbol();
    let mut missing_owner = mizar_resolve::env::SymbolIndex::new();
    for entry in symbols.symbols().iter() {
        if entry.symbol() != owner { missing_owner.insert(entry.clone()); }
    }
    let before = (ast.snapshot_text(), source.clone(), symbols.clone(), scope.clone(), labels.clone(), resolution.clone(), base.clone());
    // The historical lower, witness, arena, and label faults remain at the private parenthesized producers.
    // These controls alter the two actual native environment representations.
    for control in ["missing owner symbol", "unexpected label enrichment"] {
        let mutated = SymbolEnv::new(module.clone(), mizar_resolve::env::SymbolEnvIndexes {
            imports: symbols.imports().clone(),
            exports: symbols.exports().clone(),
            symbols: if control == "missing owner symbol" { missing_owner.clone() } else { symbols.symbols().clone() },
            labels: if control == "unexpected label enrichment" { resolution.index().clone() } else { symbols.labels().clone() },
            definitions: symbols.definitions().clone(),
            overloads: symbols.overloads().clone(),
            registrations: symbols.registrations().clone(),
            lexical_summaries: symbols.lexical_summaries().clone(),
            namespace_graph: symbols.namespace_graph().clone(),
            declaration_dependencies: symbols.declaration_dependencies().clone(),
            contributions: symbols.contributions().clone(),
            module_summaries: symbols.module_summaries().clone(),
        });
        assert_ne!(mutated, symbols, "{control} changes an actual native field");
        let mutated_before = mutated.clone();
        assert_eq!(mizar_checker::type_checker::TermFormulaChecker::check_source_theorem_equalities(&source, &mutated, &scope).unwrap_err(), "source symbol environment does not match its declaration collection", "{control} original authentication");
        assert_eq!(base.clone().with_source_statement_from_original(&source, &mutated, &scope), Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement), "{control} original installation");
        assert_eq!(mutated, mutated_before, "{control} passed dependency purity");
        assert_eq!((ast.snapshot_text(), source.clone(), symbols.clone(), scope.clone(), labels.clone(), resolution.clone(), base.clone()), before, "{control} pristine dependency purity");
        assert_eq!(task258_native_output(&ast, &module), baseline, "{control} pristine native retry");
    }
}

#[test]
fn task258b3m2b1_selector_and_byte_subtree_near_misses_are_exact() {
    let (ast, module, _, symbols) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B3M2B1_TEXT, 258_913);
    let (typed, resolved) = task258_native_output(&ast, &module);
    assert_eq!((typed.nodes().len(), typed.source_term().unwrap().terms().len(), typed.source_term().unwrap().references().len()), (53, 6, 5));
    assert_eq!(typed.source_statement_witnesses(), resolved.source_statement_witnesses());

    for index in 0..SOURCE_STATEMENT_B3M2B1_TEXT.len() {
        let mut bytes = SOURCE_STATEMENT_B3M2B1_TEXT.as_bytes().to_vec();
        bytes[index] = if bytes[index] == b'!' { b'?' } else { b'!' };
        let source = String::from_utf8(bytes).expect("ascii source");
        assert!(extract_parenthesized_witness_source_statement(&ast, &source).is_none(), "byte {index} extractor");
        assert!(
            source_statement_output_with_source(&ast, module.clone(), &symbols, &source).is_none(),
            "byte {index}"
        );
    }

    let near_misses = [
        (
            "unparenthesized",
            SOURCE_STATEMENT_B3M2B1_TEXT.replacen("take (x);", "take x;", 1),
        ),
        (
            "numeral",
            SOURCE_STATEMENT_B3M2B1_TEXT.replacen("take (x);", "take 101;", 1),
        ),
        (
            "parenthesized numeral",
            SOURCE_STATEMENT_B3M2B1_TEXT.replacen("take (x);", "take (101);", 1),
        ),
        (
            "nested",
            SOURCE_STATEMENT_B3M2B1_TEXT.replacen("take (x);", "take ((x));", 1),
        ),
        (
            "named",
            SOURCE_STATEMENT_B3M2B1_TEXT.replacen("take (x);", "take y = (x);", 1),
        ),
        (
            "multiple",
            SOURCE_STATEMENT_B3M2B1_TEXT.replacen("take (x);", "take (x), x;", 1),
        ),
        (
            "application",
            SOURCE_STATEMENT_B3M2B1_TEXT.replacen("take (x);", "take f(x);", 1),
        ),
        (
            "structure",
            SOURCE_STATEMENT_B3M2B1_TEXT.replacen("take (x);", "take (# x #);", 1),
        ),
        (
            "selector",
            SOURCE_STATEMENT_B3M2B1_TEXT.replacen("take (x);", "take the carrier of x;", 1),
        ),
        (
            "update",
            SOURCE_STATEMENT_B3M2B1_TEXT.replacen("take (x);", "take x(# field -> x #);", 1),
        ),
        (
            "set",
            SOURCE_STATEMENT_B3M2B1_TEXT.replacen("take (x);", "take {x};", 1),
        ),
        (
            "choice",
            SOURCE_STATEMENT_B3M2B1_TEXT.replacen("take (x);", "take the set;", 1),
        ),
        (
            "it",
            SOURCE_STATEMENT_B3M2B1_TEXT.replacen("take (x);", "take it;", 1),
        ),
        (
            "recovery",
            SOURCE_STATEMENT_B3M2B1_TEXT.replacen("take (x);", "take (x)", 1),
        ),
        (
            "changed theorem",
            SOURCE_STATEMENT_B3M2B1_TEXT.replacen(
                ": x = x proof\n  take (x);\n  thus x = x;\nend;",
                ": x = x;",
                1,
            ),
        ),
        (
            "existential",
            SOURCE_STATEMENT_B3M2B1_TEXT.replacen(
                ": x = x proof",
                ": ex z being set st z = z proof",
                1,
            ),
        ),
        (
            "composite",
            SOURCE_STATEMENT_B3M2B1_TEXT.replacen(": x = x proof", ": (x = x) & (x = x) proof", 1),
        ),
    ];
    for (ordinal, (label, source)) in near_misses.into_iter().enumerate() {
        assert!(
            source_statement_output_with_source(&ast, module.clone(), &symbols, &source).is_none(),
            "{label}: byte identity"
        );
        let (near_ast, near_module, _, near_symbols, diagnostics) =
            task253_ast_from_source_text_with_diagnostic_count(&source, 258_930 + ordinal);
        assert!(
            extract_parenthesized_witness_source_statement(&near_ast, &source).is_none(),
            "{label}: own source"
        );
        assert!(
            extract_parenthesized_witness_source_statement(
                &near_ast,
                SOURCE_STATEMENT_B3M2B1_TEXT,
            )
            .is_none(),
            "{label}: exact bytes/wrong subtree"
        );
        assert!(
            source_statement_output_with_source(
                &near_ast,
                near_module.clone(),
                &near_symbols,
                SOURCE_STATEMENT_B3M2B1_TEXT,
            )
            .is_none(),
            "{label}: exact route/wrong subtree"
        );
        if ordinal == 3 {
            assert_eq!(diagnostics, 0);
            let (original, original_symbols) = mizar_resolve::symbols::source_theorem_module_original_inputs(&near_ast, &near_module).unwrap().unwrap();
            let (scope, labels, resolution) = mizar_resolve::symbols::source_proof_inputs(&original, &original_symbols).unwrap();
            mizar_resolve::symbols::validate_source_proof_occurrences(&original, &original_symbols, &scope).unwrap();
            let before = (near_ast.snapshot_text(), original.clone(), original_symbols.clone(), scope.clone(), labels.clone(), resolution.clone());
            assert_eq!((near_ast.nodes().len(), scope.references().len()), (57, 5));
            assert!(near_ast.nodes().iter().all(|node| !node.recovered));
            let (typed, resolved) = task258_native_output(&near_ast, &near_module);
            let primary = typed.source_term().unwrap();
            assert_eq!((primary.terms().len(), primary.references().len(), primary.numeric_type_requests().len()), (7, 5, 0));
            for (index, parent) in [(2, None), (3, Some(2)), (4, Some(3))] {
                let term = primary.terms().get(mizar_checker::source_term::SourcePrimaryTermId::new(index)).unwrap();
                assert_eq!(term.parent().map(|id| id.index()), parent);
                assert_eq!(term.kind(), if index < 4 { mizar_checker::source_term::SourcePrimaryTermKind::Parenthesized } else { mizar_checker::source_term::SourcePrimaryTermKind::VariableReference });
            }
            for (((_, reference), occurrence), target) in primary.references().iter().zip(scope.references()).zip([0, 1, 4, 5, 6]) {
                assert_eq!(reference.term().index(), target);
                assert_eq!(reference.use_ordinal(), occurrence.ordinal());
                assert_eq!(reference.lexical_scope(), Some(occurrence.scope()));
                let term = primary.terms().get(reference.term()).unwrap();
                assert_eq!((term.site().node().index(), term.source_range(), term.spelling()), (occurrence.node().index(), occurrence.range(), occurrence.spelling()));
            }
            assert_eq!(typed.source_statement_witnesses(), resolved.source_statement_witnesses());
            assert_eq!(task258_native_output(&near_ast, &near_module), (typed, resolved));
            assert_eq!((near_ast.snapshot_text(), original, original_symbols, scope, labels, resolution), before);
        }
        if let Some(Ok(output)) =
            source_statement_output_with_source(&near_ast, near_module, &near_symbols, &source)
        {
            let primary = output.typed_ast.source_term().expect("selected primary");
            assert!(
                primary.terms().iter().all(|(_, term)| term.kind()
                    != mizar_checker::source_term::SourcePrimaryTermKind::Parenthesized
                    || term.site().node().index() != 36),
                "{label}: no partial wrapper ownership"
            );
            assert!(
                primary.references().iter().all(|(_, reference)| {
                    primary.terms().get(reference.term()).is_some_and(|term| {
                        term.parent().is_none()
                            || primary
                                .terms()
                                .get(term.parent().expect("parent"))
                                .is_some()
                    })
                }),
                "{label}: no detached child reference"
            );
        }
    }
}

#[test]
fn task258b3m2b1_family_and_active_route_isolation_is_atomic_in_both_orders() {
    let (ast, module, _, _) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B3M2B1_TEXT, 258_914);
    let (typed, _) = task258_native_output(&ast, &module);
    let statement = typed
        .source_statement()
        .expect("statement")
        .clone();
    let witnesses = typed
        .source_statement_witnesses()
        .expect("witnesses")
        .clone();
    let baseline_debug = typed.debug_text();

    for (ordinal, source) in [
        SOURCE_STATEMENT_TEXT,
        SOURCE_STATEMENT_B1_TEXT,
        SOURCE_STATEMENT_B2_TEXT,
        SOURCE_STATEMENT_B3_TEXT,
        SOURCE_STATEMENT_B3N_TEXT,
        SOURCE_STATEMENT_B3M1_TEXT,
        SOURCE_STATEMENT_B3M2A_TEXT,
    ]
    .into_iter()
    .enumerate()
    {
        let (foreign_ast, foreign_module, _, foreign_symbols) =
            task253_ast_from_source_text(source, 258_970 + ordinal);
        let (foreign_typed, _) = if source == TASK258A_SOURCE || source == SOURCE_STATEMENT_B1_TEXT || source == SOURCE_STATEMENT_B2_TEXT || source == SOURCE_STATEMENT_B3_TEXT || source == SOURCE_STATEMENT_B3N_TEXT || source == SOURCE_STATEMENT_B3M1_TEXT {
            task258_native_output(&foreign_ast, &foreign_module)
        } else {
            let output = source_statement_output_with_source(
            &foreign_ast,
            foreign_module,
            &foreign_symbols,
            source,
        )
        .expect("foreign selector")
        .expect("foreign route");
            (output.typed_ast, output.resolved)
        };
        let foreign_debug = foreign_typed.debug_text();
        assert_eq!(
            foreign_typed
                .clone()
                .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
            Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement),
            "foreign-first family {ordinal}"
        );
        assert_eq!(foreign_typed.debug_text(), foreign_debug);
        if let Some(references) = foreign_typed.source_statement_references() {
            assert_eq!(
                typed.clone().with_source_statement_references(
                    foreign_typed
                        .source_statement()
                        .expect("foreign statement")
                        .clone(),
                    references.clone(),
                ),
                Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement),
                "B3M2B1-first reference family {ordinal}"
            );
        } else if let Some(foreign_witnesses) = foreign_typed.source_statement_witnesses() {
            assert_eq!(
                typed.clone().with_source_statement_witnesses(
                    foreign_typed
                        .source_statement()
                        .expect("foreign statement")
                        .clone(),
                    foreign_witnesses.clone(),
                ),
                Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement),
                "B3M2B1-first witness family {ordinal}"
            );
        } else {
            assert_eq!(
                typed.clone().with_source_statement(
                    foreign_typed
                        .source_statement()
                        .expect("foreign statement")
                        .clone(),
                ),
                Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement),
                "B3M2B1-first base family {ordinal}"
            );
        }
        assert_eq!(typed.debug_text(), baseline_debug);
    }

    let (task253_ast, task253_module, task253_shells, task253_symbols) =
        task253_real_ast("fail_type_elaboration_local_functor_application_gap_001");
    let task253 = source_application_output(
        &task253_ast,
        task253_module,
        &task253_shells,
        &task253_symbols,
    )
    .expect("Task253 selector")
    .expect("Task253 route");
    let (task254_ast, task254_module, task254_shells, task254_symbols) = task254_real_ast();
    let task254 = source_structure_output(
        &task254_ast,
        task254_module,
        &task254_shells,
        &task254_symbols,
    )
    .expect("Task254 selector")
    .expect("Task254 route");
    let (task255_ast, task255_module, task255_shells, task255_symbols) = task255_real_ast();
    let task255 = source_set_term_output(
        &task255_ast,
        task255_module,
        &task255_shells,
        &task255_symbols,
    )
    .expect("Task255 selector")
    .expect("Task255 route");
    for (family, occupied) in [
        ("Task253", &task253.typed_ast),
        ("Task254", &task254.typed_ast),
        ("Task255", &task255.typed_ast),
    ] {
        assert_eq!(
            occupied
                .clone()
                .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
            Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement),
            "{family} then B3M2B1"
        );
    }
    assert_eq!(
        typed.clone().with_source_application(
            task253
                .typed_ast
                .source_application()
                .expect("Task253 handoff")
                .clone(),
        ),
        Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceApplication),
        "B3M2B1 then Task253"
    );
    assert_eq!(
        typed.clone().with_source_structure(
            task254
                .typed_ast
                .source_structure()
                .expect("Task254 handoff")
                .clone(),
        ),
        Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStructure),
        "B3M2B1 then Task254"
    );
    assert_eq!(
        typed.clone().with_source_set_term(
            task255
                .typed_ast
                .source_set_term()
                .expect("Task255 handoff")
                .clone(),
        ),
        Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceSetTerm),
        "B3M2B1 then Task255"
    );

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
    let plan = build_test_plan(&config).expect("Task258B3M2B1 isolation plan");
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
            extract_parenthesized_witness_source_statement(&active_ast, &source).is_some();
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
                        handoff.witnesses().len() == 1
                            && handoff.names().is_empty()
                            && output.typed_ast.source_term().is_some_and(|primary| {
                                primary.terms().len() == 6
                                    && primary.references().len() == 5
                                    && primary.terms().get(
                                        mizar_checker::source_term::SourcePrimaryTermId::new(2),
                                    ).is_some_and(|term| {
                                        term.kind()
                                            == mizar_checker::source_term::SourcePrimaryTermKind::Parenthesized
                                    })
                            })
                    })
            })
        });
        if extracted || selected_route {
            selected.push(case.id.0.clone());
        }
    }
    assert!(
        selected.is_empty(),
        "Task258B3M2B1 selected active cases: {selected:?}"
    );
}

#[test]
fn task258b3m2b1_typed_final_clone_debug_rollback_and_empty_semantics_are_stable() {
    let (ast, module, _, symbols) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B3M2B1_TEXT, 258_915);
    let (typed, resolved) = task258_native_output(&ast, &module);
    assert_eq!(
        typed.clone().debug_text(),
        typed.debug_text()
    );
    assert_eq!(
        resolved.clone().debug_text(),
        resolved.debug_text()
    );
    assert_eq!(
        typed.source_statement(),
        resolved.source_statement()
    );
    assert_eq!(
        typed.source_statement_witnesses(),
        resolved.source_statement_witnesses()
    );
    assert_eq!(
        typed.source_term(),
        resolved.source_term()
    );
    assert_eq!(
        typed.source_atomic_formula(),
        resolved.source_atomic_formula()
    );
    assert!(typed.source_statement_references().is_none());
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
    assert_eq!(typed.source_term().unwrap().references().iter().map(|(_, reference)| reference.use_ordinal()).collect::<Vec<_>>(), [0, 1, 2, 3, 4]);
    assert_eq!(source_statement_transport_detail_keys(&ast, module.clone(), &symbols, SOURCE_STATEMENT_B3M2B1_TEXT), None);
    let (source, original_symbols) = mizar_resolve::symbols::source_theorem_module_original_inputs(&ast, &module).unwrap().unwrap();
    let (scope, _, _) = mizar_resolve::symbols::source_proof_inputs(&source, &original_symbols).unwrap();
    let before = (source.clone(), original_symbols.clone(), scope.clone(), typed.clone(), resolved.clone());
    assert_eq!(typed.clone().with_source_statement_from_original(&source, &original_symbols, &scope), Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement));
    assert_eq!((source, original_symbols, scope, typed.clone(), resolved.clone()), before);
    assert_eq!(task258_native_output(&ast, &module), (typed, resolved));
}
