#[test]
fn task258b3_real_frontend_freezes_single_witness_transport() {
    assert_eq!(SOURCE_STATEMENT_B3_TEXT.len(), 104);
    assert!(SOURCE_STATEMENT_B3_TEXT.ends_with('\n'));
    let (ast, module, _, _) = task253_ast_from_source_text(SOURCE_STATEMENT_B3_TEXT, 258_600);
    let extracted: SourceStatementB3Extraction =
        extract_single_witness_source_statement(&ast, SOURCE_STATEMENT_B3_TEXT)
            .expect("Task258B3 exact parser shape");
    assert_eq!(
        (ast.nodes().len(), ast.root().expect("root").index()),
        (49, 48)
    );
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
    let (source, resolver) = mizar_resolve::symbols::source_theorem_module_original_inputs(&ast, &module).unwrap().unwrap();
    let (scope, owner_labels, label_resolution) = mizar_resolve::symbols::source_proof_inputs(&source, &resolver).unwrap();
    assert!(resolver.labels().is_empty());
    assert_eq!((owner_labels.projections().len(), label_resolution.index().len()), (1, 1));
    let (typed, resolved) = task258_native_output(&ast, &module);
    let statement = typed
        .source_statement()
        .expect("Task258B3 base statement");
    let witnesses = typed
        .source_statement_witnesses()
        .expect("Task258B3 witness handoff");
    assert!(typed.source_statement_references().is_none());
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
        .get(mizar_checker::source_statement::SourceTheoremOwnerId::new(
            0,
        ))
        .expect("Task258B3 owner");
    assert_eq!(owner.contribution().index(), 0);
    assert_eq!(
        (owner.source_range().start, owner.source_range().end),
        (19, 103)
    );
    assert_eq!(statement.checked_owner().origin().structural_path(), [2, 1]);
    let resolver_owner = resolver
        .symbols()
        .get(owner.symbol())
        .expect("owner symbol");
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
    let labels = label_resolution.index().by_contribution(owner.contribution());
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
        resolver_owner.symbol().fqn().as_str()
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
        [4]
    );
    assert_eq!(
        proof_context.visible_bindings,
        [mizar_checker::binding_env::BindingId::new(0)]
    );
    let binding = statement.binding_env().bindings().get(mizar_checker::binding_env::BindingId::new(0)).expect("module reservation");
    assert_eq!(binding.owner_context.index(), 0);
    assert_eq!(binding.kind, mizar_checker::binding_env::BindingKind::ReservedVariable);
    assert_eq!(binding.type_site, mizar_checker::binding_env::BindingTypeSite::Missing);
    assert_eq!(binding.status, mizar_checker::binding_env::BindingStatus::Active);
    assert!(matches!(binding.identity, mizar_checker::binding_env::BinderIdentity::ReservedVariable { .. }));
    assert_eq!(proof_context.parent, Some(mizar_checker::binding_env::BindingContextId::new(0)));
    assert!(proof_context.bindings.is_empty(), "unnamed take introduces no binding");
    assert_eq!(binding.declaration_range, scope.bindings()[0].range());
    assert_eq!(binding.visible_after_ordinal, 0);
    assert!(scope.bindings()[0].captures().is_empty());
    assert!(statement.binding_env().diagnostics().is_empty());
    let primary = typed.source_term().expect("Task252 handoff");
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
        assert_eq!(reference.use_ordinal(), index);
        let actual = &scope.references()[index];
        assert_eq!(actual.range(), term.source_range());
        assert_eq!(actual.spelling(), term.spelling());
        assert_eq!(source.resolved_node_for(actual.node()).unwrap().index(), term.site().node().index());
        if index < 2 {
            assert!(reference.lexical_scope().expect("module reference scope").path().is_empty());
        } else {
            assert_eq!(
                reference.lexical_scope().expect("reference scope").path(),
                [4]
            );
        }
    }
    let atomic = typed
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
    assert!(atomic.edges().iter().all(|(_, edge)| !matches!(
        edge.target(),
        mizar_checker::source_atomic_formula::SourceAtomicTermTarget::Primary(term)
            if term.index() == 2
    )));
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
            .get(mizar_checker::source_statement::SourceStatementId::new(
                index,
            ))
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
            fact.uses().iter().map(|id| id.index()).collect::<Vec<_>>(),
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
        assert_eq!(candidate.formula(), row.formula());
    }
    let witness = witnesses
        .witnesses()
        .get(mizar_checker::source_statement::SourceStatementWitnessId::new(0))
        .expect("witness row");
    assert!(witnesses.names().is_empty());
    assert_eq!(witness.binding_context().index(), 1);
    assert_eq!(witness.owner().index(), 0);
    assert_eq!(witness.take_site(), &extracted.take_site);
    assert_eq!(witness.take_range(), extracted.take_range);
    assert_eq!(witness.site(), &extracted.witness_site);
    assert_eq!(witness.source_range(), extracted.witness_range);
    assert_eq!(witness.spelling(), "x");
    assert_eq!(witness.kind(), mizar_checker::source_statement::SourceStatementWitnessKind::Unnamed);
    assert_eq!(witness.recovery(), mizar_checker::source_statement::SourceStatementRecovery::Normal);
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
    assert_eq!(primary.references().iter().map(|(_, reference)| (reference.lexical_scope().unwrap(), reference.use_ordinal(), reference.binding().index())).collect::<Vec<_>>(), scope.references().iter().map(|reference| (reference.scope(), reference.ordinal(), reference.binding().index())).collect::<Vec<_>>());
    assert_eq!(
        typed.source_statement(),
        resolved.source_statement()
    );
    assert_eq!(
        typed.source_statement_witnesses(),
        resolved.source_statement_witnesses()
    );
    assert_eq!(typed.nodes().len(), ast.nodes().len());
    assert_eq!(
        typed.nodes().root().map(|root| root.index()),
        ast.root().map(|root| root.index())
    );
    for (id, node) in typed.nodes().iter() {
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
        let (resolved_id, original) = source.arena().iter().nth(id.index()).expect("original native node");
        assert_eq!(node.resolved_node, Some(resolved_id));
        let expected_kind = match original.kind() {
            mizar_syntax::ast::SurfaceNodeKind::TermReference => "source.term.variable-reference".to_owned(),
            mizar_syntax::ast::SurfaceNodeKind::BuiltinPredicateApplication => "source.formula.atomic.equality".to_owned(),
            kind => format!("{kind:?}"),
        };
        assert_eq!(node.kind.as_str(), expected_kind, "node {} kind", id.index());
    }
}

#[test]
fn task258b3_dependency_witness_rows_and_replay_fail_closed() {
    let (ast, module, _, _) = task253_ast_from_source_text(SOURCE_STATEMENT_B3_TEXT, 258_610);
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
    // The 347 historical corruption cases are preserved at the private B3 producers.
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
        None
    );
    let baseline = task258_native_output(&exact_ast, &exact_module);
    let capture = |source: &str, ordinal: usize| {
        let unique = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let package_root = std::env::temp_dir().join(format!("mizar-test-b3-native-selector-{}-{ordinal}-{unique}", std::process::id()));
        let source_path = package_root.join("src").join("selector.miz");
        std::fs::create_dir_all(source_path.parent().unwrap()).unwrap();
        std::fs::write(&source_path, source).unwrap();
        let package = PackageId::new("mizar-test-b3-native-selector");
        let module_path = ModulePath::new(format!("tests.b3_native_selector_{ordinal}"));
        let normalized_path = mizar_session::normalize_path(&package_root, &source_path).unwrap();
        let frontend = mizar_frontend::orchestration::Frontend::new(
            mizar_frontend::source::FrontendSourceLoader::new(mizar_session::DiskSourceLoader::new(&package_root)),
            ParseOnlyImportProvider, mizar_frontend::parsing::MizarParserSeam,
        );
        let output = frontend.run(mizar_frontend::source::SourceUnitRequest {
            snapshot: super::shared::snapshot_id(258_660 + ordinal),
            input: mizar_session::SourceInput {
                package_id: package.clone(), module_path: module_path.clone(), normalized_path,
                edition: Edition::new("2026"), origin: mizar_session::SourceOriginInput::Disk { path: source_path },
            },
        }, &mizar_session::InMemorySessionIdAllocator::new()).unwrap();
        std::fs::remove_dir_all(&package_root).unwrap();
        (output, ResolverModuleId::new(package, module_path))
    };
    for (label, loaded_source) in [
        (
            "missing final LF",
            SOURCE_STATEMENT_B3_TEXT.trim_end_matches('\n').to_owned(),
        ),
        ("extra final LF", format!("{SOURCE_STATEMENT_B3_TEXT}\n")),
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
        assert!(extract_single_witness_source_statement(&exact_ast, &loaded_source).is_none(), "{label} exact extractor guard");
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
        let (frontend, module) = capture(&loaded_source, 0);
        assert!(frontend.diagnostics.is_empty(), "{label} coherent native capture");
        let ast = frontend.ast.expect("coherent source requires AST");
        let (typed, resolved) = task258_native_output(&ast, &module);
        assert!(typed.source_statement_witnesses().is_some(), "{label}");
        assert_eq!(typed.source_statement_witnesses(), resolved.source_statement_witnesses(), "{label}");
        assert_eq!(task258_native_output(&exact_ast, &exact_module), baseline, "{label} pristine native retry");
    }
    let near_misses = [
        SOURCE_STATEMENT_B3_TEXT.replace("take x;", "take y = x;"),
        SOURCE_STATEMENT_B3_TEXT.replace("take x;", "take x, x;"),
        SOURCE_STATEMENT_B3_TEXT.replace("take x;", "take y;"),
        SOURCE_STATEMENT_B3_TEXT.replace("take x;", "take (x);"),
        SOURCE_STATEMENT_B3_TEXT.replace("take x;", "take {x};"),
        SOURCE_STATEMENT_B3_TEXT.replace("  take x;\n", ""),
        SOURCE_STATEMENT_B3_TEXT.replace("  take x;\n", "  take x;\n  take x;\n"),
        SOURCE_STATEMENT_B3_TEXT.replace("  take x;\n  thus x = x;", "  thus x = x;\n  take x;"),
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
        assert!(extract_single_witness_source_statement(&exact_ast, source).is_none(), "near miss {ordinal} exact AST");
        let (frontend, module) = capture(source, ordinal + 1);
        if ordinal == 3 || ordinal == 4 {
            assert!(frontend.diagnostics.is_empty());
            let ast = frontend.ast.as_ref().expect("native witness source AST");
            assert!(ast.nodes().iter().all(|node| !node.recovered));
            let (original, symbols) = mizar_resolve::symbols::source_theorem_module_original_inputs(ast, &module)
                .expect("native witness original collection").expect("native witness original inputs");
            mizar_resolve::symbols::source_proof_inputs(&original, &symbols)
                .expect("native witness original occurrences");
        }
        if let Some(ast) = frontend.ast {
            assert!(extract_single_witness_source_statement(&ast, source).is_none(), "near miss {ordinal} parsed AST");
            assert!(extract_single_witness_source_statement(&ast, SOURCE_STATEMENT_B3_TEXT).is_none(), "near miss {ordinal} exact bytes with wrong AST");
            if ordinal == 0 || ordinal == 11 {
                assert!(frontend.diagnostics.is_empty(), "coherent theorem label drift");
                let (typed, resolved) = task258_native_output(&ast, &module);
                assert!(typed.source_statement_witnesses().is_some());
                assert_eq!(typed.source_statement_witnesses(), resolved.source_statement_witnesses());
            } else if frontend.diagnostics.is_empty() && ast.nodes().iter().all(|node| !node.recovered) {
                match mizar_resolve::symbols::source_theorem_module_original_inputs(&ast, &module) {
                    Err(error) => assert_eq!(error, mizar_resolve::resolved_ast::SurfaceResolvedArena::lower(&ast, &module).unwrap_err().to_string(), "near miss {ordinal} source lowering refusal"),
                    Ok(None) => {
                        let shells = mizar_resolve::declarations::DeclarationShellCollector::new(&ast, &module).collect();
                        let collected = mizar_resolve::symbols::SignatureProjectionExtractor::new(&ast, &shells, mizar_resolve::env::NamespacePath::new(module.path().as_str())).collect(&module);
                        assert!(!collected.diagnostics().is_empty(), "near miss {ordinal} declaration collection refusal");
                    }
                    Ok(Some((original, symbols))) => match mizar_resolve::symbols::source_proof_inputs(&original, &symbols) {
                        Err(error) => assert_eq!(mizar_resolve::symbols::source_proof_inputs(&original, &symbols), Err(error), "near miss {ordinal} resolver refusal replay"),
                        Ok((scope, labels, resolution)) => {
                            mizar_resolve::symbols::validate_source_proof_occurrences(&original, &symbols, &scope).unwrap();
                            let before = (original.clone(), symbols.clone(), scope.clone(), labels.clone(), resolution.clone());
                            if ordinal == 3 {
                                use mizar_checker::type_checker::{FormulaStatus, NormalizedTypeStatus, TermKind, TermStatus, TypeHeadRef};
                                use mizar_checker::typed_ast::{TypeEntryActual, TypeStatus};
                                assert_eq!((ast.nodes().len(), scope.references().len()), (53, 5));
                                assert!(source_statement_output_with_source(&ast, module.clone(), &symbols, source).is_none());
                                let (_, inference, base) = mizar_checker::type_checker::TermFormulaChecker::check_source_theorem_equalities(&original, &symbols, &scope).expect("single group preserves the reserved child's type");
                                let primary = base.source_term().unwrap();
                                assert_eq!((primary.terms().len(), primary.references().len(), primary.numeric_type_requests().len()), (6, 5, 0));
                                let wrapper = primary.terms().get(mizar_checker::source_term::SourcePrimaryTermId::new(2)).unwrap();
                                let child = primary.terms().get(mizar_checker::source_term::SourcePrimaryTermId::new(3)).unwrap();
                                assert_eq!(wrapper.kind(), mizar_checker::source_term::SourcePrimaryTermKind::Parenthesized);
                                assert_eq!(child.parent(), Some(mizar_checker::source_term::SourcePrimaryTermId::new(2)));
                                assert!(wrapper.parent().is_none());
                                assert!(!primary.references().iter().any(|(_, row)| row.term().index() == 2));
                                for (((_, reference), occurrence), target) in primary.references().iter().zip(scope.references()).zip([0, 1, 3, 4, 5]) {
                                    assert_eq!(reference.term().index(), target);
                                    assert_eq!(reference.use_ordinal(), occurrence.ordinal());
                                    assert_eq!(reference.lexical_scope(), Some(occurrence.scope()));
                                    let term = primary.terms().get(reference.term()).unwrap();
                                    assert_eq!((term.site().node().index(), term.source_range(), term.spelling()), (occurrence.node().index(), occurrence.range(), occurrence.spelling()));
                                }
                                assert_eq!((inference.terms().len(), inference.formulas().len(), inference.normalized_types().len()), (6, 2, 1));
                                let wrapper_checked = inference.terms().iter().find(|(_, row)| row.site == *wrapper.site()).unwrap().1;
                                let child_checked = inference.terms().iter().find(|(_, row)| row.site == *child.site()).unwrap().1;
                                assert_eq!(wrapper_checked.kind, TermKind::Parenthesized);
                                assert!(wrapper_checked.reference.is_none());
                                assert_ne!(wrapper_checked.type_entry, child_checked.type_entry);
                                let wrapper_type = inference.type_entries().get(wrapper_checked.type_entry).unwrap();
                                let child_type = inference.type_entries().get(child_checked.type_entry).unwrap();
                                assert_eq!((&wrapper_type.owner, &child_type.owner), (wrapper.site(), child.site()));
                                assert_eq!(wrapper_type.actual, child_type.actual);
                                let TypeEntryActual::Known(normalized_id) = wrapper_type.actual else { panic!("known preserved reserved type") };
                                let normalized = inference.normalized_types().get(normalized_id).unwrap();
                                let written_type = ast.nodes().iter().find(|node| matches!(node.kind, mizar_syntax::SurfaceNodeKind::TypeExpression)).unwrap();
                                assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
                                assert_eq!(normalized.status, NormalizedTypeStatus::Known);
                                assert_eq!(normalized.source.range, written_type.range);
                                assert_eq!(normalized.source.spelling, "set");
                                for (_, term) in primary.terms().iter() {
                                    let checked = inference.terms().iter().find(|(_, row)| row.site == *term.site()).unwrap().1;
                                    assert_eq!(checked.status, TermStatus::Inferred);
                                    assert!(checked.deferred.is_empty() && checked.candidate_set.is_none());
                                    let entry = inference.type_entries().get(checked.type_entry).unwrap();
                                    assert_eq!(entry.owner, *term.site());
                                    assert_eq!(entry.status, TypeStatus::Known);
                                    assert_eq!(entry.actual, TypeEntryActual::Known(normalized_id));
                                }
                                assert!(inference.formulas().iter().all(|(_, row)| row.status == FormulaStatus::Checked));
                                assert!(inference.diagnostics().is_empty() && inference.candidate_sets().is_empty() && inference.facts().is_empty());
                                let typed = base.with_source_statement_from_original(&original, &symbols, &scope).expect("single group native pair");
                                let resolved = assemble_empty_resolved_typed_ast(&typed, Vec::new()).expect("single group native final");
                                assert_eq!(task258_native_output(&ast, &module), (typed.clone(), resolved.clone()));
                                assert_eq!(typed.source_statement_witnesses(), resolved.source_statement_witnesses());
                                assert!(resolved.expr_metadata().is_empty() && resolved.statement_semantics().is_empty() && resolved.checked_proofs().is_empty());
                            } else if ordinal == 4 {
                                use mizar_checker::source_set_term::{SourceSetEdgeRole, SourceSetRequestKind, SourceSetTarget, SourceSetTermKind};
                                use mizar_checker::type_checker::{FormulaStatus, NormalizedTypeStatus, TermFormulaChecker, TermKind, TermStatus, TypeHeadRef};
                                use mizar_checker::typed_ast::{TypeEntryActual, TypeStatus};
                                let lower = TermFormulaChecker::check_source_theorem_equalities(&original, &symbols, &scope).expect("singleton native lower");
                                let (bindings, inference, base) = &lower;
                                let primary = base.source_term().unwrap();
                                let set = base.source_set_term().unwrap();
                                assert_eq!((bindings.bindings().len(), primary.terms().len(), primary.references().len(), primary.numeric_type_requests().len()), (1, 5, 5, 0));
                                assert_eq!(scope.references().len(), 5);
                                for ((_, reference), occurrence) in primary.references().iter().zip(scope.references()) {
                                    let term = primary.terms().get(reference.term()).unwrap();
                                    assert_eq!((term.site().node().index(), term.source_range(), term.spelling()), (occurrence.node().index(), occurrence.range(), occurrence.spelling()));
                                    assert_eq!((reference.use_ordinal(), reference.lexical_scope()), (occurrence.ordinal(), Some(occurrence.scope())));
                                }
                                assert_eq!(set.primary_term_fingerprint(), primary.debug_text());
                                assert_eq!((set.terms().len(), set.edges().len(), set.requests().len()), (1, 1, 1));
                                assert!(set.wrappers().is_empty() && set.generators().is_empty() && set.type_sites().is_empty() && set.conditions().is_empty());
                                let (id, enumeration) = set.terms().iter().next().unwrap();
                                assert_eq!((enumeration.kind(), enumeration.spelling()), (SourceSetTermKind::Enumeration, "{ x }"));
                                let (element_id, element) = primary.terms().iter().find(|(_, term)| term.site().node().index() == scope.references()[2].node().index()).unwrap();
                                let (_, edge) = set.edges().iter().next().unwrap();
                                assert_eq!((edge.term(), edge.ordinal(), edge.role(), edge.target()), (id, 0, SourceSetEdgeRole::EnumerationElement, SourceSetTarget::Primary(element_id)));
                                assert_eq!(enumeration.context(), element.context());
                                let (_, request) = set.requests().iter().next().unwrap();
                                assert_eq!((request.term(), request.ordinal(), request.kind(), request.generator(), request.type_site()), (id, 0, SourceSetRequestKind::ResultType, None, None));
                                assert_eq!((inference.terms().len(), inference.formulas().len(), inference.type_entries().len(), inference.normalized_types().len()), (6, 2, 12, 1));
                                let (normalized_id, normalized) = inference.normalized_types().iter().next().unwrap();
                                assert_eq!((&normalized.head, normalized.status), (&TypeHeadRef::BuiltinSet, NormalizedTypeStatus::Known));
                                for (_, entry) in inference.type_entries().iter() {
                                    assert_eq!((entry.status, entry.actual), (TypeStatus::Known, TypeEntryActual::Known(normalized_id)));
                                }
                                for (_, term) in inference.terms().iter() {
                                    assert_eq!(term.status, TermStatus::Inferred);
                                    assert!(term.deferred.is_empty() && term.candidate_set.is_none());
                                    assert_eq!(inference.type_entries().get(term.type_entry).unwrap().owner, term.site);
                                }
                                let checked = inference.terms().iter().find(|(_, term)| term.site == *enumeration.site()).unwrap().1;
                                assert_eq!(checked.kind, TermKind::SetEnumeration);
                                assert!(checked.reference.is_none());
                                for (_, formula) in inference.formulas().iter() {
                                    assert_eq!(formula.status, FormulaStatus::Checked);
                                    assert!(formula.deferred.is_empty() && formula.facts.is_empty() && formula.candidate_set.is_none());
                                }
                                assert!(inference.diagnostics().is_empty() && inference.facts().is_empty() && inference.candidate_sets().is_empty());
                                let typed = base.clone().with_source_statement_from_original(&original, &symbols, &scope).expect("singleton own native pair");
                                let resolved = assemble_empty_resolved_typed_ast(&typed, Vec::new()).expect("singleton own native final");
                                assert_eq!(task258_native_output(&ast, &module), (typed.clone(), resolved.clone()));
                                assert_eq!(typed.source_set_term(), base.source_set_term());
                                assert_eq!(typed.source_statement(), resolved.source_statement());
                                assert_eq!(typed.source_statement_witnesses(), resolved.source_statement_witnesses());
                                assert!(resolved.expr_metadata().is_empty() && resolved.statement_semantics().is_empty() && resolved.checked_proofs().is_empty());
                                assert_eq!(assemble_empty_resolved_typed_ast(&typed.clone(), Vec::new()).unwrap(), resolved);
                                assert_eq!(TermFormulaChecker::check_source_theorem_equalities(&original, &symbols, &scope).as_ref(), Ok(&lower));
                            } else {
                            assert!(mizar_checker::type_checker::TermFormulaChecker::check_source_theorem_equalities(&original, &symbols, &scope).is_err(), "near miss {ordinal} bounded native refusal");
                            }
                            assert_eq!((original, symbols, scope, labels, resolution), before, "near miss {ordinal} dependency purity");
                        }
                    },
                }
            } else {
                assert!(!frontend.diagnostics.is_empty() || ast.nodes().iter().any(|node| node.recovered), "near miss {ordinal} Frontend recovery refusal");
            }
        } else {
            assert!(!frontend.diagnostics.is_empty(), "near miss {ordinal} absent AST requires Frontend diagnostics");
        }
        assert_eq!(task258_native_output(&exact_ast, &exact_module), baseline, "near miss {ordinal} pristine native retry");
    }
    for (ordinal, source) in [
        SOURCE_STATEMENT_TEXT,
        SOURCE_STATEMENT_B1_TEXT,
        SOURCE_STATEMENT_B2_TEXT,
    ]
    .into_iter()
    .enumerate()
    {
        let (ast, module, _, symbols) = task253_ast_from_source_text(source, 258_630 + ordinal);
        let (output_typed, _) = if source == TASK258A_SOURCE || source == SOURCE_STATEMENT_B1_TEXT || source == SOURCE_STATEMENT_B2_TEXT {
            task258_native_output(&ast, &module)
        } else {
            let output = source_statement_output_with_source(&ast, module, &symbols, source)
            .expect("existing selector")
            .expect("existing route");
            (output.typed_ast, output.resolved)
        };
        assert!(output_typed.source_statement_witnesses().is_none());
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
        let selected_route =
            source_statement_output_with_source(&ast, resolver.module.clone(), &symbols, &source)
                .is_some_and(|result| {
                    result
                        .is_ok_and(|output| output.typed_ast.source_statement_witnesses().is_some())
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
    let (b3_ast, b3_module, _, _) = task253_ast_from_source_text(SOURCE_STATEMENT_B3_TEXT, 258_640);
    let (b3, _) = task258_native_output(&b3_ast, &b3_module);
    let b3_statement = b3
        .source_statement()
        .expect("B3 statement")
        .clone();
    let b3_witnesses = b3
        .source_statement_witnesses()
        .expect("B3 witnesses")
        .clone();
    let b3_debug = b3.debug_text();

    for (ordinal, source) in [SOURCE_STATEMENT_TEXT, SOURCE_STATEMENT_B2_TEXT]
        .into_iter()
        .enumerate()
    {
        let (ast, module, _, _) = task253_ast_from_source_text(source, 258_641 + ordinal);
        let (existing_typed, _) = task258_native_output(&ast, &module);
        let existing_debug = existing_typed.debug_text();
        assert_eq!(
            existing_typed
                .clone()
                .with_source_statement_witnesses(b3_statement.clone(), b3_witnesses.clone()),
            Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement)
        );
        assert_eq!(existing_typed.debug_text(), existing_debug);
        assert_eq!(
            b3.clone().with_source_statement(
                existing_typed
                    .source_statement()
                    .expect("existing statement")
                    .clone()
            ),
            Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement)
        );
        assert_eq!(b3.debug_text(), b3_debug);
    }
    let (b1_ast, b1_module, _, _) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B1_TEXT, 258_645);
    let (b1, _) = task258_native_output(&b1_ast, &b1_module);
    let b1_debug = b1.debug_text();
    assert_eq!(
        b1
            .clone()
            .with_source_statement_witnesses(b3_statement.clone(), b3_witnesses.clone()),
        Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(b1.debug_text(), b1_debug);
    assert_eq!(
        b3.clone().with_source_statement_references(
            b1
                .source_statement()
                .expect("B1 statement")
                .clone(),
            b1
                .source_statement_references()
                .expect("B1 references")
                .clone(),
        ),
        Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(b3.debug_text(), b3_debug);
}

#[test]
fn task258b3_typed_and_final_debug_clone_preserve_empty_semantics() {
    let (ast, module, _, _) = task253_ast_from_source_text(SOURCE_STATEMENT_B3_TEXT, 258_650);
    let (typed, resolved) = task258_native_output(&ast, &module);
    let typed_debug = typed.debug_text();
    let base_at = typed_debug
        .find("source-statement-debug-v1")
        .expect("base debug");
    let witness_at = typed_debug
        .find("source-statement-witness-debug-v1")
        .expect("witness debug");
    let nodes_at = typed_debug.find("nodes:\n").expect("nodes debug");
    assert!(base_at < witness_at && witness_at < nodes_at);
    assert_eq!(typed.clone().debug_text(), typed_debug);
    assert_eq!(
        resolved.clone().debug_text(),
        resolved.debug_text()
    );
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
    assert!(typed_debug.contains(
        "witness#0 owner=0 binding_context=1 term=primary#2 take_range=77..84 take_site=35 range=82..83 site=34 source_ordinal=1 ordinal=0 kind=unnamed recovery=normal spelling=\"x\""
    ));
}
