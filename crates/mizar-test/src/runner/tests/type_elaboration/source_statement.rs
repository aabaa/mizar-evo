use super::{
    SOURCE_STATEMENT_TEXT, SourceStatementExtraction, SourceStatementRouteInputs,
    SourceStatementRouteOutput, extract_source_reserved_variable_theorem_statement,
    source_statement_output_with_source, source_statement_output_with_source_and_mutation,
    source_statement_output_with_resolver_mutation, source_statement_resolver_env_for_test,
};

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
