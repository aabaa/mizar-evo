use super::type_elaboration::{
    SOURCE_PREDICATE_DEFINITION_TEXT, SourcePredicateDefinitionRouteMutation,
    source_predicate_definition_output, source_predicate_definition_output_with_mutation,
    step5c5_predicate_semantics_detail_keys,
};

const TASK259_CASE: &str = "pass_type_elaboration_predicate_definition_payload_001";
const TASK260_MIXED_CASE: &str = "fail_type_elaboration_predicate_functor_definition_gap_001";

#[test]
fn step5c5_predicate_semantics_require_authenticated_binary_properties() {
    let pass = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/miz/pass/predicates/pass_type_elaboration_pred_properties_declaration_001.miz"
    ));
    let fail = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/miz/fail/predicates/fail_type_elaboration_pred_property_arity_mismatch_001.miz"
    ));
    let (pass_ast, pass_module, pass_shells, pass_symbols) =
        task253_ast_from_source_text(pass, 505_000);
    assert_eq!(
        step5c5_predicate_semantics_detail_keys(
            &pass_ast,
            &pass_module,
            &pass_shells,
            &pass_symbols
        ),
        Some(Vec::new())
    );

    let (fail_ast, fail_module, fail_shells, fail_symbols) =
        task253_ast_from_source_text(fail, 505_001);
    let mismatch = Some(vec!["predicates.property.arity_mismatch".to_owned()]);
    assert_eq!(
        step5c5_predicate_semantics_detail_keys(
            &fail_ast,
            &fail_module,
            &fail_shells,
            &fail_symbols
        ),
        mismatch
    );
    assert_ne!(
        step5c5_predicate_semantics_detail_keys(
            &pass_ast,
            &pass_module,
            &fail_shells,
            &fail_symbols
        ),
        Some(Vec::new())
    );
    let changed_kind = rebuild_surface_ast_replacing_kind(
        &pass_ast,
        SurfaceNodeKind::PredicatePattern,
        SurfaceNodeKind::TermList,
    );
    assert_ne!(
        step5c5_predicate_semantics_detail_keys(
            &changed_kind,
            &pass_module,
            &pass_shells,
            &pass_symbols
        ),
        Some(Vec::new())
    );
    let recovered = rebuild_surface_ast_recovering_first_token_in_kind(
        &fail_ast,
        SurfaceNodeKind::PredicateDefinition,
    );
    assert_ne!(
        step5c5_predicate_semantics_detail_keys(
            &recovered,
            &fail_module,
            &fail_shells,
            &fail_symbols
        ),
        mismatch
    );

    let widened = fail
        .replacen("let X be set", "let X, Y be set", 1)
        .replacen("pred SoloDef: solo X", "pred SoloDef: X solo Y", 1);
    let (ast, module, shells, symbols) = task253_ast_from_source_text(&widened, 505_002);
    assert_ne!(
        step5c5_predicate_semantics_detail_keys(&ast, &module, &shells, &symbols),
        mismatch
    );
}

#[test]
fn task259_real_source_rejects_missing_or_wrong_resolver_predicate_arity() {
    use mizar_resolve::env::{DefinitionIndex, DefinitionShell, SymbolEnvIndexes};
    let (ast, module, shells, symbols) = task253_ast_from_source_text(SOURCE_PREDICATE_DEFINITION_TEXT, 259_060);
    for arity in [None, Some(0), Some(1), Some(3)] {
        let mut definitions = DefinitionIndex::default();
        for original in symbols.definitions().iter() {
            let mut replacement = DefinitionShell::new(original.symbol().clone(), original.kind(), original.origin().clone(), original.contribution())
                .with_visibility(original.visibility());
            if let Some(notation) = original.notation_shape() { replacement = replacement.with_notation_shape(notation); }
            if let Some(signature) = original.signature() { replacement = replacement.with_signature(signature.clone()); }
            if original.kind() == mizar_resolve::env::DefinitionKind::Predicate {
                if let Some(arity) = arity { replacement = replacement.with_arity(arity); }
            } else if let Some(arity) = original.arity() { replacement = replacement.with_arity(arity); }
            assert_eq!(definitions.insert(replacement), original.id());
        }
        let changed = mizar_resolve::env::SymbolEnv::new(module.clone(), SymbolEnvIndexes {
            imports: symbols.imports().clone(), exports: symbols.exports().clone(), symbols: symbols.symbols().clone(),
            labels: symbols.labels().clone(), definitions, overloads: symbols.overloads().clone(),
            registrations: symbols.registrations().clone(), lexical_summaries: symbols.lexical_summaries().clone(),
            namespace_graph: symbols.namespace_graph().clone(), declaration_dependencies: symbols.declaration_dependencies().clone(),
            contributions: symbols.contributions().clone(), module_summaries: symbols.module_summaries().clone(),
        });
        let error = source_predicate_definition_output(&ast, module.clone(), &shells, &changed, SOURCE_PREDICATE_DEFINITION_TEXT)
            .expect("exact source remains selected").expect_err("wrong predicate arity must fail closed");
        assert_eq!(error, "Task259 resolver: predicate projection is not exact", "{arity:?}");
    }
}

#[test]
fn task259_real_source_surface_resolver_and_lower_bundle_is_exact() {
    assert_eq!(
        SOURCE_PREDICATE_DEFINITION_TEXT,
        TASK248_TWO_PARAMETER_SOURCE
    );
    assert_eq!(SOURCE_PREDICATE_DEFINITION_TEXT.len(), 165);
    assert!(SOURCE_PREDICATE_DEFINITION_TEXT.ends_with('\n'));
    assert!(!SOURCE_PREDICATE_DEFINITION_TEXT.ends_with("\n\n"));
    assert_eq!(
        sha256_text(SOURCE_PREDICATE_DEFINITION_TEXT),
        "91bdb5f51c0ea5f07bdd831700cb9803f2aa57e005921c7e4e1798ecbbf2bd9f"
    );

    let (ast, module, shells, symbols, diagnostics) =
        task253_ast_from_source_text_with_diagnostic_count(
            SOURCE_PREDICATE_DEFINITION_TEXT,
            259_000,
        );
    assert_eq!(diagnostics, 0);
    assert_eq!(
        (ast.nodes().len(), ast.root().map(|id| id.index())),
        (71, Some(70))
    );
    assert_eq!(
        ast.root().and_then(|id| ast.node(id)).map(|node| (
            node.range.start,
            node.range.end,
            node.recovered
        )),
        Some((0, 164, false))
    );
    for (index, node) in ast.nodes().iter().enumerate() {
        assert_eq!(node.range.source_id, ast.source_id, "surface row {index}");
        assert!(node.range.start <= node.range.end, "surface row {index}");
        assert!(node.range.end <= 164, "surface row {index}");
        assert!(!node.recovered, "surface row {index}");
        assert!(
            node.children.iter().all(|child| child.index() < index),
            "surface row {index}"
        );
    }
    assert_eq!(
        sha256_text(&task259_surface_profile(&ast)),
        "42efaaadfccaa5c857f504764a5af25b6cf98393d72a6d979c8e4a0f18bddf95"
    );

    let [block, predicate, property] = shells.declarations() else {
        panic!("Task259 must expose exactly three declaration shells");
    };
    assert_eq!(
        [
            (block.id().index(), block.ordinal(), block.node_id().index()),
            (
                predicate.id().index(),
                predicate.ordinal(),
                predicate.node_id().index(),
            ),
            (
                property.id().index(),
                property.ordinal(),
                property.node_id().index(),
            ),
        ],
        [(0, 0, 67), (1, 1, 62), (2, 2, 66)]
    );
    assert_eq!(predicate.parent(), Some(block.id()));
    assert_eq!(property.parent(), Some(block.id()));
    assert!(shells.exports().is_empty());
    let projections = mizar_resolve::symbols::SignatureProjectionExtractor::new(
        &ast,
        &shells,
        mizar_resolve::env::NamespacePath::new(module.path().as_str()),
    )
    .extract();
    assert_eq!(projections.len(), 2);
    let projection = projections.iter().find(|projection| projection.symbol_kind() == mizar_resolve::env::SymbolKind::Predicate).unwrap();
    assert_eq!(projection.arity(), Some(2));
    let definition = symbols.definitions().iter().find(|definition| definition.kind() == mizar_resolve::env::DefinitionKind::Predicate).unwrap();
    assert_eq!(definition.arity(), Some(2));
    assert_eq!(
        (
            symbols.symbols().len(),
            symbols.definitions().len(),
            symbols.contributions().len(),
        ),
        (2, 2, 1)
    );

    let output = source_predicate_definition_output(
        &ast,
        module,
        &shells,
        &symbols,
        SOURCE_PREDICATE_DEFINITION_TEXT,
    )
    .expect("Task259 exact selector")
    .unwrap_or_else(|error| panic!("Task259 exact route failed: {error}"));
    let context = output.typed_ast.source_context().expect("Task248");
    let source_type = output.typed_ast.source_type().expect("Task249");
    let terms = output.typed_ast.source_term().expect("Task252");
    let formulas = output.typed_ast.source_atomic_formula().expect("Task256");
    let predicate = output
        .typed_ast
        .source_predicate_definition()
        .expect("Task259");
    assert_eq!(
        (
            context.binding_env().bindings().len(),
            context.binding_env().contexts().len(),
            context.binding_env().diagnostics().len(),
            source_type.applications().len(),
            source_type.expressions().len(),
            source_type.arguments().len(),
            terms.terms().len(),
            terms.references().len(),
            terms.numeric_type_requests().len(),
        ),
        (2, 2, 0, 2, 2, 0, 4, 4, 0)
    );
    assert_eq!(
        (
            formulas.formulas().len(),
            formulas.wrappers().len(),
            formulas.predicate_segments().len(),
            formulas.predicate_heads().len(),
            formulas.candidates().len(),
            formulas.type_sites().len(),
            formulas.attributes().len(),
            formulas.edges().len(),
            formulas.requests().len(),
        ),
        (2, 0, 0, 0, 0, 0, 0, 4, 4)
    );
    assert_eq!(
        (
            predicate.definitions().len(),
            predicate.parameters().len(),
            predicate.guards().len(),
            predicate.properties().len(),
            predicate.correctness().len(),
        ),
        (1, 2, 1, 1, 1)
    );
    assert_eq!(
        output.typed_ast.source_predicate_definition(),
        output.resolved.source_predicate_definition()
    );
}

fn task259_surface_profile(ast: &mizar_syntax::SurfaceAst) -> String {
    let mut rows = ast
        .nodes()
        .iter()
        .enumerate()
        .map(|(index, node)| {
            format!(
                "{index}:{:?}:{}..{}:recovered={}:children={:?}",
                node.kind,
                node.range.start,
                node.range.end,
                node.recovered,
                node.children
                    .iter()
                    .map(|child| child.index())
                    .collect::<Vec<_>>()
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    rows.push('\n');
    rows
}

#[derive(Debug, Clone, Copy)]
enum Task259SurfaceMutation {
    StructuralKind,
    StructuralRange,
    StructuralChildren,
    TokenRecovery,
    ExpressionRoot,
}

fn task259_mutated_surface_ast(ast: &SurfaceAst, mutation: Task259SurfaceMutation) -> SurfaceAst {
    let mut builder = SurfaceAstBuilder::new(ast.source_id);
    let mut rebuilt = Vec::<SurfaceBuilderNodeId>::with_capacity(ast.nodes().len());
    for (index, node) in ast.nodes().iter().enumerate() {
        let kind = if index == 39 && matches!(mutation, Task259SurfaceMutation::StructuralKind) {
            SurfaceNodeKind::AttributeChain
        } else {
            node.kind.clone()
        };
        let range = if index == 39 && matches!(mutation, Task259SurfaceMutation::StructuralRange) {
            SourceRange {
                source_id: ast.source_id,
                start: 22,
                end: 24,
            }
        } else {
            node.range
        };
        let children =
            if index == 39 && matches!(mutation, Task259SurfaceMutation::StructuralChildren) {
                Vec::new()
            } else {
                node.children
                    .iter()
                    .map(|child| rebuilt[child.index()])
                    .collect()
            };
        let rebuilt_id = match kind {
            SurfaceNodeKind::Token(token) => {
                if index == 4 && matches!(mutation, Task259SurfaceMutation::TokenRecovery) {
                    builder.add_recovered_token(token.kind, token.text, range)
                } else {
                    builder.add_token(token.kind, token.text, range)
                }
            }
            structural => builder.add_node(structural, range, children),
        };
        rebuilt.push(rebuilt_id);
    }
    let expression_root = if matches!(mutation, Task259SurfaceMutation::ExpressionRoot) {
        Some(rebuilt[39])
    } else {
        ast.expression_root()
            .map(|expression_root| rebuilt[expression_root.index()])
    };
    builder.finish(
        ast.root().map(|root| rebuilt[root.index()]),
        expression_root,
    )
}

#[test]
fn task259_source_ast_resolver_and_lower_mutations_fail_at_the_owner() {
    let (ast, module, shells, symbols) =
        task253_ast_from_source_text(SOURCE_PREDICATE_DEFINITION_TEXT, 259_010);
    for (ordinal, near_source) in [
        SOURCE_PREDICATE_DEFINITION_TEXT.replace("let y be set;", "let z be set;"),
        SOURCE_PREDICATE_DEFINITION_TEXT.replace("assume x = x;", "assume y = y;"),
        SOURCE_PREDICATE_DEFINITION_TEXT.replace("task259_rel", "task260_rel"),
        SOURCE_PREDICATE_DEFINITION_TEXT.replace(
            "symmetry by computation(steps: 1);",
            "symmetry by computation(steps: 2);",
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let (near_ast, near_module, near_shells, near_symbols) =
            task253_ast_from_source_text(&near_source, 259_020 + ordinal);
        assert!(
            source_predicate_definition_output(
                &near_ast,
                near_module,
                &near_shells,
                &near_symbols,
                &near_source,
            )
            .is_none(),
            "surface near miss {ordinal} selected"
        );
    }

    for mutation in [
        Task259SurfaceMutation::StructuralKind,
        Task259SurfaceMutation::StructuralRange,
        Task259SurfaceMutation::StructuralChildren,
        Task259SurfaceMutation::TokenRecovery,
        Task259SurfaceMutation::ExpressionRoot,
    ] {
        let malformed_ast = task259_mutated_surface_ast(&ast, mutation);
        assert!(
            source_predicate_definition_output(
                &malformed_ast,
                module.clone(),
                &shells,
                &symbols,
                SOURCE_PREDICATE_DEFINITION_TEXT,
            )
            .is_none(),
            "same-source malformed Surface AST {mutation:?} selected"
        );
    }

    let wrong_module = mizar_resolve::resolved_ast::ModuleId::new(
        mizar_session::PackageId::new("task259"),
        mizar_session::ModulePath::new("task259.wrong"),
    );
    let resolver_error = source_predicate_definition_output(
        &ast,
        wrong_module,
        &shells,
        &symbols,
        SOURCE_PREDICATE_DEFINITION_TEXT,
    )
    .expect("exact source remains selected")
    .expect_err("foreign module must fail");
    assert!(resolver_error.starts_with("Task259 resolver:"));

    for (mutation, owner) in [
        (
            SourcePredicateDefinitionRouteMutation::DuplicateContextParameterSite,
            "Task248 source context:",
        ),
        (
            SourcePredicateDefinitionRouteMutation::WrongContextDefinitionSite,
            "Task248 source context:",
        ),
        (
            SourcePredicateDefinitionRouteMutation::RemoveResolverShell,
            "Task259 resolver:",
        ),
        (
            SourcePredicateDefinitionRouteMutation::WrongResolverEntry,
            "Task259 resolver:",
        ),
        (
            SourcePredicateDefinitionRouteMutation::WrongResolverPropertyEntry,
            "Task259 resolver:",
        ),
        (
            SourcePredicateDefinitionRouteMutation::RemoveTypeExpression,
            "Task249 source type:",
        ),
        (
            SourcePredicateDefinitionRouteMutation::WrongTermBinding,
            "Task252 source term:",
        ),
        (
            SourcePredicateDefinitionRouteMutation::RemoveAtomicFormula,
            "Task256 atomic formula:",
        ),
        (
            SourcePredicateDefinitionRouteMutation::RemovePredicateGuard,
            "Task259 predicate definition:",
        ),
    ] {
        let error = source_predicate_definition_output_with_mutation(
            &ast,
            module.clone(),
            &shells,
            &symbols,
            SOURCE_PREDICATE_DEFINITION_TEXT,
            mutation,
        )
        .expect("exact source remains selected")
        .expect_err("lower mutation must fail");
        assert!(error.starts_with(owner), "{mutation:?}: {error}");
    }

    let first = source_predicate_definition_output(
        &ast,
        module.clone(),
        &shells,
        &symbols,
        SOURCE_PREDICATE_DEFINITION_TEXT,
    )
    .expect("first selector")
    .expect("first route");
    let second = source_predicate_definition_output(
        &ast,
        module,
        &shells,
        &symbols,
        SOURCE_PREDICATE_DEFINITION_TEXT,
    )
    .expect("replay selector")
    .expect("replay route");
    assert_eq!(first.typed_ast.debug_text(), second.typed_ast.debug_text());
    assert_eq!(first.resolved.debug_text(), second.resolved.debug_text());
}

#[test]
fn task259_expectation_selection_and_mixed_definition_route_stay_isolated() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("mizar-test crate should live below the workspace root")
        .to_path_buf();
    let config = DiscoveryConfig {
        workspace_root: workspace_root.clone(),
        tests_root: workspace_root.join("tests"),
        manifest_path: workspace_root.join("tests/coverage/spec_trace.toml"),
        profile: TestProfile::Fast,
        validation_mode: ValidationMode::Metadata,
    };
    let plan = build_test_plan(&config).expect("Task259 repository plan should build");
    let selected = active_type_elaboration_cases(&plan)
        .filter(|case| {
            std::fs::read_to_string(&case.source_path)
                .is_ok_and(|source| source == SOURCE_PREDICATE_DEFINITION_TEXT)
        })
        .collect::<Vec<_>>();
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].id.0, TASK259_CASE);
    assert!(selected[0].expectation_path.ends_with(Path::new(
        "tests/miz/pass/types/pass_type_elaboration_predicate_definition_payload_001.expect.toml"
    )));

    let (task259_ordinal, task259_case) = active_type_elaboration_cases(&plan)
        .enumerate()
        .find(|(_, case)| case.id.0 == TASK259_CASE)
        .expect("Task259 active sidecar");
    let task259_result = run_type_elaboration_case(
        &workspace_root,
        &workspace_root.join("tests"),
        task259_case,
        task259_ordinal,
    );
    assert_eq!(task259_result.status, TypeElaborationCaseStatus::Passed);
    assert!(task259_result.actual_detail_keys.is_empty());

    let (mixed_ordinal, mixed_case) = active_type_elaboration_cases(&plan)
        .enumerate()
        .find(|(_, case)| case.id.0 == TASK260_MIXED_CASE)
        .expect("Task260 mixed boundary remains active");
    let mixed_frontend =
        run_frontend(&workspace_root, mixed_case, mixed_ordinal).expect("mixed frontend");
    let mixed_source = mixed_frontend.source_text;
    let mixed_ast = mixed_frontend.ast.expect("mixed AST");
    let mixed_resolver = resolver_symbol_collection(&workspace_root, mixed_case, &mixed_ast);
    assert!(mixed_resolver.detail_keys.is_empty());
    let mixed_symbols = augment_type_elaboration_import_summaries(
        &mixed_ast,
        &mixed_resolver.module,
        mixed_resolver.env,
    );
    assert!(
        source_predicate_definition_output(
            &mixed_ast,
            mixed_resolver.module,
            &mixed_resolver.shells,
            &mixed_symbols,
            &mixed_source,
        )
        .is_none()
    );
    let mixed_result = run_type_elaboration_case(
        &workspace_root,
        &workspace_root.join("tests"),
        mixed_case,
        mixed_ordinal,
    );
    assert_eq!(mixed_result.status, TypeElaborationCaseStatus::Passed);
}

#[test]
fn task259_route_publishes_no_property_proof_fact_or_acceptance() {
    let (ast, module, shells, symbols) =
        task253_ast_from_source_text(SOURCE_PREDICATE_DEFINITION_TEXT, 259_100);
    let first = source_predicate_definition_output(
        &ast,
        module.clone(),
        &shells,
        &symbols,
        SOURCE_PREDICATE_DEFINITION_TEXT,
    )
    .expect("Task259 selector")
    .expect("Task259 route");
    let second = source_predicate_definition_output(
        &ast,
        module,
        &shells,
        &symbols,
        SOURCE_PREDICATE_DEFINITION_TEXT,
    )
    .expect("Task259 replay selector")
    .expect("Task259 replay route");
    let obligations = first
        .typed_ast
        .initial_obligations()
        .iter()
        .collect::<Vec<_>>();
    let [(id, obligation)] = obligations.as_slice() else {
        panic!("Task259 must publish exactly one pending obligation");
    };
    assert_eq!(id.index(), 0);
    assert_eq!(
        obligation.kind,
        mizar_checker::typed_ast::InitialObligationKind::PredicatePropertyCorrectness
    );
    assert_eq!(
        obligation.status,
        mizar_checker::typed_ast::InitialObligationStatus::Pending
    );
    assert!(obligation.assumptions.is_empty());
    assert_eq!(
        obligation.goal.as_str(),
        "source.definition.predicate.correctness:property=0"
    );
    assert_eq!(
        obligation.provenance.as_str(),
        "source.definition.predicate:definition=0:property=0"
    );
    assert!(first.typed_ast.facts().is_empty());
    assert!(first.typed_ast.types().is_empty());
    assert!(first.typed_ast.coercions().is_empty());
    assert!(first.resolved.checked_formulas().is_empty());
    assert!(first.resolved.statement_semantics().is_empty());
    assert!(first.resolved.checked_proofs().is_empty());
    assert!(first.resolved.checked_proof_nodes().is_empty());
    assert!(first.resolved.checked_terminal_goals().is_empty());
    assert!(first.resolved.cluster_facts().is_empty());
    assert!(first.resolved.diagnostics().is_empty());
    assert_eq!(
        first.typed_ast.source_predicate_definition(),
        first.resolved.source_predicate_definition()
    );
    assert_eq!(first.typed_ast.debug_text(), second.typed_ast.debug_text());
    assert_eq!(first.resolved.debug_text(), second.resolved.debug_text());
}

#[test]
fn task259_core_item_context_association_is_exact_and_deterministic() {
    let (ast, module, shells, symbols) =
        task253_ast_from_source_text(SOURCE_PREDICATE_DEFINITION_TEXT, 259_200);
    let output = source_predicate_definition_output(
        &ast,
        module,
        &shells,
        &symbols,
        SOURCE_PREDICATE_DEFINITION_TEXT,
    )
    .expect("Task259 selector")
    .expect("Task259 route");
    let source_context = output
        .typed_ast
        .source_context()
        .expect("Task248 source context")
        .clone();
    let checker_owner = output
        .typed_ast
        .source_predicate_definition()
        .expect("Task259 checker owner")
        .clone();
    let source_bindings = task259_source_binding_core_handoff(&source_context, &checker_owner);
    let expected_source_bindings = source_bindings.clone();

    let first = mizar_core::elaborator::SourcePredicateCoreContextProducer::build(
        source_bindings.clone(),
        source_context.clone(),
        checker_owner.clone(),
    )
    .expect("Task259 Core item context");
    let second = mizar_core::elaborator::SourcePredicateCoreContextProducer::build(
        source_bindings,
        source_context.clone(),
        checker_owner.clone(),
    )
    .expect("Task259 deterministic replay");
    assert_eq!(first, second);
    assert_eq!(first.source_id(), source_context.source_id());
    assert_eq!(first.module_id(), source_context.module_id());
    assert_eq!(first.source_bindings(), &expected_source_bindings);
    assert_eq!(
        first.source_bindings().binding_env(),
        source_context.binding_env()
    );
    assert_eq!(first.source_context(), &source_context);
    assert_eq!(first.checker_owner(), &checker_owner);
    assert_eq!(first.items().len(), 1);
    assert!(!first.items().is_empty());

    let definition = checker_owner
        .definitions()
        .get(mizar_checker::source_predicate_definition::SourcePredicateDefinitionId::new(0))
        .expect("Task259 definition");
    let source_item = source_context
        .context_links()
        .get(definition.context())
        .expect("Task248 definition link")
        .item
        .expect("Task248 containing source item");
    let association = first
        .items()
        .get(definition.id())
        .expect("Task259 association");
    assert_eq!(association.definition(), definition.id());
    assert_eq!(association.source_item(), source_item);
    assert_eq!(association.symbol(), definition.symbol());
    let core_item = first
        .context()
        .item_registry()
        .id_for_symbol(definition.symbol())
        .expect("Core predicate item");
    assert_eq!(association.core_item(), core_item);
    assert_eq!(
        first
            .items()
            .iter()
            .map(|(id, row)| (id, row.source_item(), row.symbol().clone(), row.core_item()))
            .collect::<Vec<_>>(),
        vec![(definition.id(), source_item, definition.symbol().clone(), core_item)]
    );

    let item = first
        .context()
        .item_registry()
        .items()
        .get(core_item)
        .expect("Core item row");
    assert_eq!(item.symbol, *definition.symbol());
    assert_eq!(item.kind, mizar_core::core_ir::CoreItemKind::Predicate);
    assert_eq!(item.visibility.as_str(), "public");
    assert_eq!(item.status, mizar_core::core_ir::CoreItemStatus::Valid);
    assert!(item.dependencies.is_empty());
    assert!(item.diagnostics.is_empty());
    assert_eq!(
        item.source.anchor,
        mizar_core::core_ir::CoreSourceAnchor::SourceRange(definition.source_range())
    );
    assert_eq!(
        item.source.provenance,
        vec![mizar_core::core_ir::CoreProvenance::new(
            mizar_core::core_ir::CoreProvenancePhase::Checker,
            "source-predicate-core-item-v1.definition.0",
        )]
    );
    let boundary = first
        .context()
        .definition_boundaries()
        .get_by_item(core_item)
        .expect("pending definition boundary");
    assert_eq!(
        boundary.kind,
        mizar_core::elaborator::DefinitionBoundaryKind::DefinitionalItem
    );
    assert_eq!(
        boundary.status,
        mizar_core::elaborator::DefinitionBoundaryStatus::PendingBody
    );
    assert_eq!(boundary.item, core_item);
    assert_eq!(boundary.symbol, *definition.symbol());
    assert_eq!(boundary.source, item.source);
    assert_eq!(
        boundary.provenance.as_slice(),
        &[mizar_core::core_ir::CoreProvenance::new(
            mizar_core::core_ir::CoreProvenancePhase::Checker,
            "source-predicate-core-item-v1.definition.0",
        )]
    );
    let source_map = first.context().source_map();
    assert_eq!(source_map.item_sources.len(), 1);
    assert_eq!(source_map.item_sources.get(&core_item), Some(&item.source));
    assert!(source_map.term_sources.is_empty());
    assert!(source_map.formula_sources.is_empty());
    assert!(source_map.definition_sources.is_empty());
    assert!(source_map.proof_sources.is_empty());
    assert!(source_map.algorithm_sources.is_empty());
    assert!(source_map.generated_sources.is_empty());
    assert!(source_map.obligation_sources.is_empty());
    assert_eq!(
        first.context().worklist().entries(),
        &[mizar_core::elaborator::ElaborationWorkItem {
            kind: mizar_core::elaborator::ElaborationWorkItemKind::Item(core_item),
            status: mizar_core::elaborator::ElaborationWorkStatus::Pending,
            source: item.source.clone(),
            diagnostics: Vec::new(),
            checker_diagnostics: Vec::new(),
        }]
    );
}

#[test]
fn task259_core_item_context_default_deny_mutations() {
    let (ast, module, shells, symbols) =
        task253_ast_from_source_text(SOURCE_PREDICATE_DEFINITION_TEXT, 259_210);
    let output = source_predicate_definition_output(
        &ast,
        module,
        &shells,
        &symbols,
        SOURCE_PREDICATE_DEFINITION_TEXT,
    )
    .expect("Task259 selector")
    .expect("Task259 route");
    let source_context = output
        .typed_ast
        .source_context()
        .expect("Task248 source context")
        .clone();
    let checker_owner = output
        .typed_ast
        .source_predicate_definition()
        .expect("Task259 checker owner")
        .clone();
    for mutation in [
        Task259CoreContextMutation::MissingItem,
        Task259CoreContextMutation::ExtraItem,
        Task259CoreContextMutation::WrongKind,
        Task259CoreContextMutation::WrongVisibility,
        Task259CoreContextMutation::WrongSource,
        Task259CoreContextMutation::WrongProvenance,
        Task259CoreContextMutation::MissingBoundary,
        Task259CoreContextMutation::WrongBoundary,
        Task259CoreContextMutation::MissingDependency,
    ] {
        let source_bindings =
            task259_source_binding_core_handoff_with_mutation(&source_context, &checker_owner, mutation);
        let error = mizar_core::elaborator::SourcePredicateCoreContextProducer::build(
            source_bindings,
            source_context.clone(),
            checker_owner.clone(),
        )
        .expect_err("Core mutation must fail closed");
        assert_eq!(
            error,
            mizar_core::elaborator::SourcePredicateCoreContextError::InvalidCoreContext,
            "{mutation:?}"
        );
    }

    let (foreign_ast, foreign_module, foreign_shells, foreign_symbols) =
        task253_ast_from_source_text(SOURCE_PREDICATE_DEFINITION_TEXT, 259_211);
    let foreign_output = source_predicate_definition_output(
        &foreign_ast,
        foreign_module,
        &foreign_shells,
        &foreign_symbols,
        SOURCE_PREDICATE_DEFINITION_TEXT,
    )
    .expect("foreign Task259 selector")
    .expect("foreign Task259 route");
    let source_bindings = task259_source_binding_core_handoff(&source_context, &checker_owner);
    let error = mizar_core::elaborator::SourcePredicateCoreContextProducer::build(
        source_bindings,
        foreign_output
            .typed_ast
            .source_context()
            .expect("foreign source context")
            .clone(),
        foreign_output
            .typed_ast
            .source_predicate_definition()
            .expect("foreign checker owner")
            .clone(),
    )
    .expect_err("foreign source and owner must fail closed");
    assert_eq!(
        error,
        mizar_core::elaborator::SourcePredicateCoreContextError::EnvironmentMismatch
    );
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Task259CoreContextMutation {
    Baseline,
    MissingItem,
    ExtraItem,
    WrongKind,
    WrongVisibility,
    WrongSource,
    WrongProvenance,
    MissingBoundary,
    WrongBoundary,
    MissingDependency,
}

fn task259_source_binding_core_handoff(
    source_context: &mizar_checker::source_context::SourceBindingContextHandoff,
    checker_owner: &mizar_checker::source_predicate_definition::SourcePredicateDefinitionHandoff,
) -> mizar_core::elaborator::SourceBindingCoreContextHandoff {
    task259_source_binding_core_handoff_with_mutation(
        source_context,
        checker_owner,
        Task259CoreContextMutation::Baseline,
    )
}

fn task259_source_binding_core_handoff_with_mutation(
    source_context: &mizar_checker::source_context::SourceBindingContextHandoff,
    checker_owner: &mizar_checker::source_predicate_definition::SourcePredicateDefinitionHandoff,
    mutation: Task259CoreContextMutation,
) -> mizar_core::elaborator::SourceBindingCoreContextHandoff {
    let definition = checker_owner
        .definitions()
        .get(mizar_checker::source_predicate_definition::SourcePredicateDefinitionId::new(0))
        .expect("Task259 definition");
    let source_range = match mutation {
        Task259CoreContextMutation::WrongSource => mizar_core::core_ir::CoreSourceRef::direct(
            mizar_session::SourceRange {
                source_id: source_context.source_id(),
                start: definition.source_range().start + 1,
                end: definition.source_range().end,
            },
        ),
        _ => mizar_core::core_ir::CoreSourceRef::direct(definition.source_range()),
    };
    let provenance_key = match mutation {
        Task259CoreContextMutation::WrongProvenance => "wrong-task259-provenance",
        _ => "source-predicate-core-item-v1.definition.0",
    };
    let source = source_range.with_provenance(vec![mizar_core::core_ir::CoreProvenance::new(
        mizar_core::core_ir::CoreProvenancePhase::Checker,
        provenance_key,
    )]);
    let provenance =
        mizar_core::elaborator::CheckerOwnedProvenance::checker(provenance_key);
    let kind = match mutation {
        Task259CoreContextMutation::WrongKind => mizar_core::core_ir::CoreItemKind::Functor,
        _ => mizar_core::core_ir::CoreItemKind::Predicate,
    };
    let visibility = match mutation {
        Task259CoreContextMutation::WrongVisibility => "private",
        _ => "public",
    };
    let mut input = mizar_core::elaborator::CoreContextInput::new(
        mizar_core::elaborator::ResolvedTypedAstSummary::new(
            source_context.source_id(),
            source_context.module_id().clone(),
        ),
    );
    let seed = mizar_core::elaborator::CoreItemSeed::new(
        definition.symbol().clone(),
        kind,
        visibility,
        source,
        provenance,
    );
    let seed = match mutation {
        Task259CoreContextMutation::MissingBoundary => seed,
        Task259CoreContextMutation::WrongBoundary => seed
            .with_definition_boundary(mizar_core::elaborator::DefinitionBoundaryKind::Theorem),
        _ => seed.with_definition_boundary(
            mizar_core::elaborator::DefinitionBoundaryKind::DefinitionalItem,
        ),
    };
    let seed = match mutation {
        Task259CoreContextMutation::MissingDependency => {
            seed.with_dependencies(vec![definition.symbol().clone()])
        }
        _ => seed,
    };
    if mutation != Task259CoreContextMutation::MissingItem {
        input.item_seeds.push(seed);
    }
    if mutation == Task259CoreContextMutation::ExtraItem {
        input.item_seeds.push(
            mizar_core::elaborator::CoreItemSeed::new(
                mizar_resolve::resolved_ast::SymbolId::new(
                    definition.symbol().module().clone(),
                    mizar_resolve::resolved_ast::LocalSymbolId::new("task259-extra"),
                    mizar_resolve::resolved_ast::FullyQualifiedName::new(format!(
                        "{}.task259-extra",
                        definition.symbol().fqn().as_str()
                    )),
                ),
                mizar_core::core_ir::CoreItemKind::Predicate,
                "public",
                mizar_core::core_ir::CoreSourceRef::direct(definition.source_range())
                    .with_provenance(vec![mizar_core::core_ir::CoreProvenance::new(
                        mizar_core::core_ir::CoreProvenancePhase::Checker,
                        "source-predicate-core-item-v1.definition.extra",
                    )]),
                mizar_core::elaborator::CheckerOwnedProvenance::checker(
                    "source-predicate-core-item-v1.definition.extra",
                ),
            )
            .with_definition_boundary(
                mizar_core::elaborator::DefinitionBoundaryKind::DefinitionalItem,
            ),
        );
    }
    let context = mizar_core::elaborator::prepare_core_context(input)
        .expect("Task259 Core context seed should prepare");
    mizar_core::elaborator::SourceBindingCoreContextProducer::build(
        context,
        source_context.binding_env().clone(),
    )
    .expect("Task259 33LB handoff should build")
}

#[test]
fn metamorphic_predicate_layout_lexical_stream_matches_genuine_frontend() {
    let original = task257c4c1_frontend_output(SOURCE_PREDICATE_DEFINITION_TEXT, 259_200);
    assert!(original.diagnostics.is_empty());
    let ast = original.ast.as_ref().unwrap();
    let mut sources = vec![("original", SOURCE_PREDICATE_DEFINITION_TEXT.to_owned())];
    sources.extend(
        metamorphic_edits::variants(ast, SOURCE_PREDICATE_DEFINITION_TEXT)
            .into_iter()
            .filter(|(kind, _)| matches!(*kind, "leading_line" | "intertoken_layout"))
            .map(|(kind, edits)| {
                (
                    kind,
                    metamorphic_edits::apply(SOURCE_PREDICATE_DEFINITION_TEXT, &edits),
                )
            }),
    );
    assert_eq!(sources.len(), 3);
    for (ordinal, (layout, source)) in sources.into_iter().enumerate() {
        let frontend = task257c4c1_frontend_output(&source, 259_201 + ordinal);
        assert!(frontend.diagnostics.is_empty(), "{layout}");
        assert!(frontend.tokens.diagnostics.is_empty(), "{layout}");
        let ast = frontend.ast.as_ref().unwrap();
        assert!(ast.canonical_bytes().is_some(), "{layout}");
        assert!(
            ast.node_views().all(|node| !node.is_recovered()),
            "{layout}"
        );
        assert!(frontend.preprocessed.import_stubs.is_empty());
        let preprocessed = mizar_lexer::preprocess_source_for_lexing(&source);
        assert!(preprocessed.diagnostics.is_empty(), "{layout}");
        assert_eq!(
            preprocessed.lexical_text,
            frontend.preprocessed.lexical_text.as_str(),
            "{layout}"
        );
        assert_eq!(
            preprocessed.preprocess_map, frontend.preprocessed.source_map.preprocess_map,
            "{layout}"
        );
        let raw = mizar_lexer::scan_raw(&preprocessed.lexical_text).unwrap();
        let environment = mizar_lexer::build_lexical_environment(&[], &[]).unwrap();
        let scopes = mizar_lexer::build_scope_skeleton(&raw);
        let actual = mizar_lexer::disambiguate(
            &raw,
            &environment,
            &mizar_lexer::ParserLexContext::general(),
            &scopes,
        );
        assert!(actual.diagnostics().is_empty(), "{layout}");
        assert_eq!(actual.tokens().len(), 38, "{layout}");
        assert_eq!(
            actual.tokens().len(),
            frontend.tokens.tokens().len(),
            "{layout}"
        );
        assert_eq!(actual.tokens()[33].kind, mizar_lexer::TokenKind::Numeral);
        assert_eq!(actual.tokens()[33].lexeme, "1");
        for (token, native) in actual.tokens().iter().zip(frontend.tokens.tokens()) {
            let mapped = preprocessed
                .preprocess_map
                .source_ranges_for_lexical(token.span)
                .unwrap();
            assert_eq!(mapped.len(), 1, "{layout}: token split by preprocessing");
            let span = mizar_session::SourceRange {
                source_id: ast.source_id,
                start: mapped[0].start,
                end: mapped[0].end,
            };
            assert_eq!(
                (token.kind, token.lexeme.as_str(), span),
                (native.kind, native.text.as_ref(), native.span),
                "{layout}"
            );
            assert_eq!(
                source.get(span.start..span.end),
                Some(token.lexeme.as_str()),
                "{layout}"
            );
        }
    }
}

#[test]
fn metamorphic_predicate_layout_transports_actual_source_handoffs() {
    let config = step5c11_config();
    let plan = build_test_plan(&config).unwrap();
    let (ordinal, case) = active_type_elaboration_cases(&plan)
        .enumerate()
        .find(|(_, case)| case.id.0 == TASK259_CASE)
        .unwrap();
    let original = std::fs::read_to_string(&case.source_path).unwrap();
    assert_eq!(original, SOURCE_PREDICATE_DEFINITION_TEXT);
    let frontend = run_frontend(&config.workspace_root, case, ordinal).unwrap();
    assert!(frontend.diagnostics.is_empty());
    let original_ast = frontend.ast.as_ref().unwrap();
    let mut sources = vec![("original", original.clone())];
    sources.extend(
        metamorphic_edits::variants(original_ast, &original)
            .into_iter()
            .filter(|(kind, _)| matches!(*kind, "leading_line" | "intertoken_layout"))
            .map(|(kind, edits)| (kind, metamorphic_edits::apply(&original, &edits))),
    );
    assert_eq!(sources.len(), 3);
    sources.extend([
        ("missing_lf", original.trim_end_matches('\n').to_owned()),
        ("extra_lf", format!("{original}\n")),
        ("trailing_comment", format!("{original}:: trailing\n")),
        ("parameter_comment", original.replacen("x be", "x :: parameter\n be", 1)),
        ("guard_comment", original.replacen("assume x = x;", "assume x :: guard\n = x;", 1)),
        ("body_comment", original.replacen("means x = y;", "means x :: body\n = y;", 1)),
        ("justification_only", original.replacen("symmetry by computation(steps: 1)", "symmetry  by computation(steps:1)", 1)),
        ("compact", "definition let x be set;let y be set;assume x=x;pred Task259PredicateDefinition:x task259_rel y means x=y;symmetry by computation(steps:1);end;".to_owned()),
    ]);
    assert_eq!(sources.len(), 11);
    let mut failures = Vec::new();
    let mut baseline_fingerprints: Option<[String; 5]> = None;
    for (layout, source) in sources {
        metamorphic_guard::with_source(case, &source, || {
            let frontend = run_frontend(&config.workspace_root, case, ordinal).unwrap();
            assert_eq!(frontend.source_text.as_ref(), source);
            assert!(frontend.diagnostics.is_empty(), "{layout}");
            let ast = frontend.ast.as_ref().unwrap();
            assert!(ast.canonical_bytes().is_some(), "{layout}");
            assert!(
                ast.node_views().all(|node| !node.is_recovered()),
                "{layout}"
            );
            let resolver = resolver_symbol_collection(&config.workspace_root, case, ast);
            assert!(resolver.detail_keys.is_empty(), "{layout}");
            let output = source_predicate_definition_output(
                ast,
                resolver.module.clone(),
                &resolver.shells,
                &resolver.env,
                &source,
            );
            let output = match output {
                Some(Ok(output)) => output,
                result => {
                    let error = format!("{layout}: {result:?}");
                    assert_ne!(layout, "original", "{error}");
                    failures.push(error);
                    return;
                }
            };
            let handoff = output.typed_ast.source_predicate_definition().unwrap();
            assert_eq!(Some(handoff), output.resolved.source_predicate_definition());
            let context = output.typed_ast.source_context().unwrap();
            let types = output.typed_ast.source_type().unwrap();
            let terms = output.typed_ast.source_term().unwrap();
            let atomic = output.typed_ast.source_atomic_formula().unwrap();
            let expected = |index: usize| ast.nodes()[index].range;
            assert_eq!(
                (
                    ast.nodes().len(),
                    ast.token_nodes().len(),
                    output.typed_ast.nodes().len()
                ),
                (71, 38, 17)
            );
            for token in ast.token_views() {
                let range = token.range();
                assert_eq!(
                    source.get(range.start..range.end),
                    Some(token.as_token().unwrap().text.as_ref())
                );
            }
            for ((id, node), member) in output.typed_ast.nodes().iter().zip([
                38, 39, 2, 42, 43, 7, 46, 48, 50, 55, 57, 59, 53, 62, 66, 67, 70,
            ]) {
                assert_eq!(
                    node.anchor,
                    mizar_session::SourceAnchor::Range(expected(member)),
                    "{layout} anchor{}",
                    id.index()
                );
            }
            assert_eq!(
                context.items().iter().next().unwrap().1.source_range,
                expected(67)
            );
            for (index, ((_, declaration), (_, parameter))) in context
                .declarations()
                .iter()
                .zip(handoff.parameters().iter())
                .enumerate()
            {
                let (identifier, type_token, statement) = [(2, 4, 41), (7, 9, 45)][index];
                assert_eq!(declaration.declaration_range, expected(identifier));
                assert_eq!(declaration.written_type_range, expected(type_token));
                assert_eq!(parameter.declaration_range(), expected(identifier));
                assert_eq!(parameter.source_range(), expected(statement));
                assert_eq!(parameter.owner().index(), 0);
                assert_eq!(parameter.site().node().index(), [2, 5][index]);
                assert_eq!(
                    (
                        parameter.ordinal(),
                        parameter.binding().index(),
                        parameter.written_type().index()
                    ),
                    (index, index, index)
                );
                let binding = context
                    .binding_env()
                    .bindings()
                    .get(parameter.binding())
                    .unwrap();
                assert_eq!(binding.spelling, ["x", "y"][index]);
                assert_eq!(
                    binding.identity,
                    mizar_checker::binding_env::BinderIdentity::ResolverLocal {
                        scope: mizar_resolve::names::LocalTermScope::new(vec![0]),
                        ordinal: index,
                        declaration_range: expected(identifier),
                    }
                );
                assert_eq!(
                    (binding.owner_context.index(), binding.visible_after_ordinal),
                    (1, index)
                );
                assert_eq!(
                    binding.type_site,
                    mizar_checker::binding_env::BindingTypeSite::Source(expected(type_token))
                );
                assert!(binding.captured.identities().is_empty());
                let application = types.applications().iter().nth(index).unwrap().1;
                assert_eq!(application.binding(), parameter.binding());
                let expression = types.expressions().get(application.root()).unwrap();
                assert_eq!(
                    (expression.source_range(), expression.head_range()),
                    (expected(type_token), expected(type_token))
                );
            }
            assert_eq!(
                (
                    terms.terms().len(),
                    terms.references().len(),
                    terms.numeric_type_requests().len()
                ),
                (4, 4, 0)
            );
            for (index, (_, reference)) in terms.references().iter().enumerate() {
                let term = terms.terms().get(reference.term()).unwrap();
                assert_eq!(term.source_range(), expected([46, 48, 55, 57][index]));
                assert_eq!(term.spelling(), ["x", "x", "x", "y"][index]);
                assert_eq!(reference.binding().index(), [0, 0, 0, 1][index]);
                assert_eq!(reference.use_ordinal(), 2);
                assert_eq!(term.context().index(), 1);
                assert_eq!(reference.lexical_scope().unwrap().path(), [0]);
                assert_eq!(
                    context
                        .binding_env()
                        .lookup(&mizar_checker::binding_env::BindingLookupSite::new(
                            term.spelling(),
                            term.context(),
                            reference.lexical_scope().cloned(),
                            reference.use_ordinal(),
                        ))
                        .unwrap(),
                    mizar_checker::binding_env::BindingLookupResult::Local(BindingId::new(
                        [0, 0, 0, 1][index]
                    ))
                );
            }
            assert_eq!(
                (
                    atomic.formulas().len(),
                    atomic.edges().len(),
                    atomic.requests().len()
                ),
                (2, 4, 4)
            );
            for (index, (_, formula)) in atomic.formulas().iter().enumerate() {
                assert_eq!(formula.source_range(), expected([50, 59][index]));
            }
            for (index, ((_, edge), (_, request))) in atomic
                .edges()
                .iter()
                .zip(atomic.requests().iter())
                .enumerate()
            {
                assert_eq!(
                    (edge.formula().index(), edge.ordinal()),
                    (index / 2, index % 2)
                );
                assert_eq!(request.edge().unwrap().index(), index);
                assert_eq!(
                    edge.target(),
                    mizar_checker::source_atomic_formula::SourceAtomicTermTarget::Primary(
                        mizar_checker::source_term::SourcePrimaryTermId::new(index)
                    )
                );
            }
            assert_eq!(
                (
                    handoff.definitions().len(),
                    handoff.parameters().len(),
                    handoff.guards().len(),
                    handoff.properties().len(),
                    handoff.correctness().len()
                ),
                (1, 2, 1, 1, 1)
            );
            let definition = handoff.definitions().iter().next().unwrap().1;
            assert_eq!(definition.source_range(), expected(62));
            assert_eq!(definition.definiens().index(), 1);
            assert_eq!(
                (
                    definition.site().node().index(),
                    definition.context().index(),
                    definition.source_ordinal()
                ),
                (13, 1, 0)
            );
            let actual_definition = resolver
                .env
                .definitions()
                .get(definition.definition())
                .unwrap();
            assert_eq!(definition.symbol(), actual_definition.symbol());
            assert_eq!(definition.origin(), actual_definition.origin());
            assert_eq!(
                definition.origin().anchor(),
                &mizar_session::SourceAnchor::Range(expected(62))
            );
            assert_eq!(
                resolver
                    .env
                    .contributions()
                    .get(definition.contribution())
                    .unwrap()
                    .anchor(),
                &mizar_session::SourceAnchor::Range(expected(67))
            );
            let guard = handoff.guards().iter().next().unwrap().1;
            let property = handoff.properties().iter().next().unwrap().1;
            let correctness = handoff.correctness().iter().next().unwrap().1;
            assert_eq!(
                (
                    guard.owner().index(),
                    guard.ordinal(),
                    guard.formula().index()
                ),
                (0, 0, 0)
            );
            assert_eq!(guard.source_range(), expected(53));
            assert_eq!(guard.site().node().index(), 12);
            assert_eq!(property.source_range(), expected(66));
            assert_eq!(
                (
                    property.site().node().index(),
                    property.owner().index(),
                    property.ordinal()
                ),
                (14, 0, 0)
            );
            assert_eq!(
                property.justification(),
                &mizar_session::SourceAnchor::Range(expected(65))
            );
            assert_eq!(
                (
                    correctness.owner().index(),
                    correctness.property().index(),
                    correctness.obligation().index()
                ),
                (0, 0, 0)
            );
            assert_eq!(
                correctness.source_anchor(),
                &mizar_session::SourceAnchor::Range(expected(66))
            );
            assert_eq!(
                [
                    handoff.source_context_fingerprint(),
                    handoff.source_type_fingerprint(),
                    handoff.source_term_fingerprint(),
                    handoff.source_atomic_formula_fingerprint()
                ],
                [
                    context.debug_text(),
                    types.debug_text(),
                    terms.debug_text(),
                    atomic.debug_text()
                ]
            );
            let fingerprints = [
                context.debug_text(),
                types.debug_text(),
                terms.debug_text(),
                atomic.debug_text(),
                handoff.debug_text(),
            ];
            if layout == "original" {
                baseline_fingerprints = Some(fingerprints);
            } else if layout == "justification_only" {
                let baseline = baseline_fingerprints.as_ref().unwrap();
                assert_eq!(&fingerprints[..4], &baseline[..4]);
                assert_ne!(fingerprints[4], baseline[4]);
                assert_eq!(source.len(), original.len());
                for member in [
                    38, 39, 2, 42, 43, 7, 46, 48, 50, 55, 57, 59, 53, 62, 66, 67, 70,
                ] {
                    assert_eq!(expected(member), original_ast.nodes()[member].range);
                }
                assert_eq!(
                    (expected(65).start, expected(65).end),
                    (
                        original_ast.nodes()[65].range.start + 1,
                        original_ast.nodes()[65].range.end
                    )
                );
            } else if [2, 7, 46, 48, 55, 57, 67]
                .iter()
                .any(|index| expected(*index) != original_ast.nodes()[*index].range)
            {
                assert_ne!(
                    Some(&fingerprints),
                    baseline_fingerprints.as_ref(),
                    "{layout}"
                );
            } else {
                assert_eq!(
                    Some(&fingerprints),
                    baseline_fingerprints.as_ref(),
                    "{layout}"
                );
            }
            if layout == "compact" {
                assert_eq!(
                    [
                        expected(41).start - expected(67).start,
                        expected(2).start - expected(41).start,
                        expected(4).start - expected(2).end,
                        expected(41).end - expected(4).end,
                        expected(50).start - expected(53).start,
                        expected(53).end - expected(50).end,
                        expected(59).start - expected(62).start,
                        expected(62).end - expected(59).end,
                        expected(65).start - expected(66).start,
                        expected(65).end - expected(65).start,
                        expected(66).end - expected(65).end,
                        expected(67).end - expected(66).end,
                        expected(48).start - expected(46).end,
                    ],
                    [11, 4, 4, 1, 7, 1, 54, 1, 9, 23, 1, 4, 1]
                );
                assert_eq!(
                    [
                        expected(7).start - expected(45).start,
                        expected(9).start - expected(7).end,
                        expected(45).end - expected(9).end,
                        expected(57).start - expected(55).end
                    ],
                    [4, 4, 1, 1]
                );
                assert_eq!(
                    (
                        expected(41).end,
                        expected(45).end,
                        expected(53).end,
                        expected(62).end
                    ),
                    (
                        expected(45).start,
                        expected(53).start,
                        expected(62).start,
                        expected(66).start
                    )
                );
            }
            assert_eq!(output.typed_ast.initial_obligations().len(), 1);
            let (id, obligation) = output
                .typed_ast
                .initial_obligations()
                .iter()
                .next()
                .unwrap();
            assert_eq!(id.index(), 0);
            assert_eq!(
                obligation.kind,
                mizar_checker::typed_ast::InitialObligationKind::PredicatePropertyCorrectness
            );
            assert_eq!(
                obligation.status,
                mizar_checker::typed_ast::InitialObligationStatus::Pending
            );
            assert_eq!(obligation.owner, *property.site());
            assert_eq!(obligation.source_range, expected(66));
            assert!(obligation.assumptions.is_empty());
            assert_eq!(
                obligation.goal.as_str(),
                "source.definition.predicate.correctness:property=0"
            );
            assert_eq!(
                obligation.provenance.as_str(),
                "source.definition.predicate:definition=0:property=0"
            );
            let replay = source_predicate_definition_output(
                ast,
                resolver.module.clone(),
                &resolver.shells,
                &resolver.env,
                &source,
            )
            .unwrap()
            .unwrap();
            assert_eq!(output.typed_ast.debug_text(), replay.typed_ast.debug_text());
            assert_eq!(output.resolved.debug_text(), replay.resolved.debug_text());
            assert_eq!(
                output.typed_ast.clone().source_predicate_definition(),
                Some(handoff)
            );
            assert_eq!(
                output.resolved.clone().source_predicate_definition(),
                Some(handoff)
            );
            let result = run_type_elaboration_case(
                &config.workspace_root,
                &config.workspace_root.join("tests"),
                case,
                ordinal,
            );
            assert_eq!(result.status, TypeElaborationCaseStatus::Passed, "{layout}");
            assert!(result.actual_detail_keys.is_empty(), "{layout}");
        });
    }
    assert!(
        failures.is_empty(),
        "actual Predicate layout transport failed: {failures:?}"
    );
}

#[test]
fn metamorphic_predicate_layout_distinguishes_source_authentication_and_unsupported_shape() {
    let config = step5c11_config();
    let root = &config.workspace_root;
    let plan = build_test_plan(&config).unwrap();
    let (ordinal, case) = active_type_elaboration_cases(&plan)
        .enumerate()
        .find(|(_, case)| case.id.0 == TASK259_CASE)
        .unwrap();
    let source = std::fs::read_to_string(&case.source_path).unwrap();
    let frontend = run_frontend(root, case, ordinal).unwrap();
    assert!(frontend.diagnostics.is_empty());
    let ast = frontend.ast.as_ref().unwrap();
    let resolver = resolver_symbol_collection(root, case, ast);
    assert!(resolver.detail_keys.is_empty());
    let module = resolver.module;
    source_predicate_definition_output(
        ast,
        module.clone(),
        &resolver.shells,
        &resolver.env,
        &source,
    )
    .expect("normal supported original")
    .expect("genuine resolver and all lower producers");
    for trailing in [
        source.trim_end_matches('\n').to_owned(),
        format!("{source}\n"),
        format!("{source}:: trailing\n"),
    ] {
        source_predicate_definition_output(
            ast,
            module.clone(),
            &resolver.shells,
            &resolver.env,
            &trailing,
        )
        .expect("unchanged token anchors")
        .expect("only trailing layout differs");
    }
    let mut stale_sources = vec![String::new(), source.replacen("steps: 1", "steps: 2", 1)];
    stale_sources.extend(
        metamorphic_edits::variants(ast, &source)
            .into_iter()
            .filter(|(kind, _)| matches!(*kind, "leading_line" | "intertoken_layout"))
            .map(|(_, edits)| metamorphic_edits::apply(&source, &edits)),
    );
    for raw in stale_sources {
        let error = source_predicate_definition_output_with_mutation(
            ast,
            module.clone(),
            &resolver.shells,
            &resolver.env,
            &raw,
            SourcePredicateDefinitionRouteMutation::RemoveTypeExpression,
        )
        .expect("supported AST independent of stale raw")
        .expect_err("raw authentication precedes lower Type failure");
        assert_eq!(error, "Task259 source association is invalid");
    }
    let resolver_error = source_predicate_definition_output_with_mutation(
        ast,
        module.clone(),
        &resolver.shells,
        &resolver.env,
        &source.replacen("steps: 1", "steps: 2", 1),
        SourcePredicateDefinitionRouteMutation::RemoveResolverShell,
    )
    .expect("supported AST with bad raw")
    .expect_err("resolver owner precedes source authentication");
    assert!(resolver_error.starts_with("Task259 resolver:"));
    let mut indexes = super::import_fixtures::clone_symbol_env_indexes(&resolver.env);
    indexes.namespace_graph.insert_node(
        mizar_resolve::env::NamespaceNodeKind::Segment,
        Some(module.clone()),
        "unowned",
        resolver.env.contributions().iter().next().unwrap().id(),
    );
    let changed_env = mizar_resolve::env::SymbolEnv::new(module.clone(), indexes);
    assert_eq!(changed_env.symbols(), resolver.env.symbols());
    assert_eq!(changed_env.definitions(), resolver.env.definitions());
    assert_eq!(changed_env.contributions(), resolver.env.contributions());
    assert_ne!(changed_env, resolver.env);
    assert_eq!(
        source_predicate_definition_output_with_mutation(
            ast,
            module.clone(),
            &resolver.shells,
            &changed_env,
            &source,
            SourcePredicateDefinitionRouteMutation::RemoveTypeExpression,
        )
        .expect("selected original AST")
        .expect_err("complete replay rejects unowned namespace state before lower Type"),
        "Task259 source association is invalid"
    );

    for corruption in 0..2 {
        let mut changed = source.clone();
        if corruption == 1 {
            changed.remove(16);
        }
        let mut builder = SurfaceAstBuilder::new(ast.source_id);
        let mut rebuilt = Vec::new();
        for (index, node) in ast.nodes().iter().enumerate() {
            let mut range = node.range;
            if corruption == 1 {
                range.start -= usize::from(range.start > 16);
                range.end -= usize::from(range.end > 16);
            }
            let children = node
                .children
                .iter()
                .map(|child| rebuilt[child.index()])
                .collect();
            rebuilt.push(match &node.kind {
                SurfaceNodeKind::Token(token) => builder.add_token(
                    if corruption == 0 && index == 33 {
                        SurfaceTokenKind::Identifier
                    } else {
                        token.kind
                    },
                    token.text.clone(),
                    range,
                ),
                kind => builder.add_node(kind.clone(), range, children),
            });
        }
        let malformed = builder.finish(Some(rebuilt[70]), None);
        assert_eq!(malformed.nodes().len(), 71);
        assert!(malformed.canonical_bytes().is_some());
        assert!(malformed.node_views().all(|node| !node.is_recovered()));
        let fresh = resolver_symbol_collection(root, case, &malformed);
        assert!(fresh.detail_keys.is_empty());
        let output = source_predicate_definition_output_with_mutation(
            &malformed,
            module.clone(),
            &fresh.shells,
            &fresh.env,
            &changed,
            SourcePredicateDefinitionRouteMutation::RemoveTypeExpression,
        );
        if corruption == 0 {
            assert!(output.is_none(), "unsupported Numeral kind {corruption}");
        } else {
            assert_eq!(changed.get(13..17), Some("letx"));
            for token in malformed.token_views() {
                let range = token.range();
                assert_eq!(
                    changed.get(range.start..range.end),
                    Some(token.as_token().unwrap().text.as_ref())
                );
            }
            assert_eq!(
                output
                    .expect("closed graph and coherent token slices remain selected")
                    .expect_err("genuine lexer refuses merged keyword/binder before lower build"),
                "Task259 source association is invalid"
            );
        }
    }

    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let foreign_root = std::env::temp_dir().join(format!(
        "mizar-test-predicate-foreign-{}-{nanos}",
        std::process::id()
    ));
    let foreign_path = foreign_root
        .join("src")
        .join(case.source_path.file_name().unwrap());
    std::fs::create_dir_all(foreign_path.parent().unwrap()).unwrap();
    std::fs::write(&foreign_path, &source).unwrap();
    let ids = mizar_session::InMemorySessionIdAllocator::new();
    let snapshot = super::shared::snapshot_id(ordinal + 1);
    mizar_session::SessionIdAllocator::next_source_id(&ids, snapshot).unwrap();
    let foreign = mizar_frontend::orchestration::Frontend::new(
        mizar_frontend::source::FrontendSourceLoader::new(mizar_session::DiskSourceLoader::new(
            &foreign_root,
        )),
        ParseOnlyImportProvider,
        mizar_frontend::parsing::MizarParserSeam,
    )
    .run(
        mizar_frontend::source::SourceUnitRequest {
            snapshot,
            input: mizar_session::SourceInput {
                package_id: module.package().clone(),
                module_path: module.path().clone(),
                normalized_path: mizar_session::normalize_path(&foreign_root, &foreign_path)
                    .unwrap(),
                edition: mizar_session::Edition::new("2026"),
                origin: mizar_session::SourceOriginInput::Disk { path: foreign_path },
            },
        },
        &ids,
    )
    .unwrap();
    std::fs::remove_dir_all(&foreign_root).unwrap();
    assert_eq!(foreign.source.source_text.as_ref(), source);
    assert_eq!(&foreign.source.package_id, module.package());
    assert_eq!(&foreign.source.module_path, module.path());
    assert!(foreign.diagnostics.is_empty());
    let foreign_ast = foreign.ast.as_ref().unwrap();
    assert!(foreign_ast.canonical_bytes().is_some());
    assert!(foreign_ast.node_views().all(|node| !node.is_recovered()));
    assert_ne!(foreign_ast.source_id, ast.source_id);
    let fresh = resolver_symbol_collection(root, case, foreign_ast);
    assert!(fresh.detail_keys.is_empty());
    assert_eq!(fresh.module, module);
    let positive = source_predicate_definition_output(
        foreign_ast,
        module.clone(),
        &fresh.shells,
        &fresh.env,
        &source,
    )
    .expect("genuine fresh source remains supported")
    .expect("matching real current resolver environment");
    assert_eq!(positive.typed_ast.source_id(), foreign_ast.source_id);
    let (id, obligation) = positive
        .typed_ast
        .initial_obligations()
        .iter()
        .next()
        .unwrap();
    assert_eq!(id.index(), 0);
    assert_eq!(positive.typed_ast.initial_obligations().len(), 1);
    assert_eq!(obligation.source_range, foreign_ast.nodes()[66].range);
    assert_eq!(
        obligation.kind,
        mizar_checker::typed_ast::InitialObligationKind::PredicatePropertyCorrectness
    );
    assert_eq!(
        obligation.status,
        mizar_checker::typed_ast::InitialObligationStatus::Pending
    );
    assert!(obligation.assumptions.is_empty());
    assert_eq!(
        positive.typed_ast.source_predicate_definition(),
        positive.resolved.source_predicate_definition()
    );
    for shells in [&resolver.shells, &fresh.shells] {
        let error = source_predicate_definition_output_with_mutation(
            foreign_ast,
            module.clone(),
            shells,
            &resolver.env,
            &source,
            SourcePredicateDefinitionRouteMutation::RemoveTypeExpression,
        )
        .expect("supported foreign AST")
        .expect_err("stale source association before lower Type");
        assert!(error.starts_with("Task259 resolver:"), "{error}");
    }
}
