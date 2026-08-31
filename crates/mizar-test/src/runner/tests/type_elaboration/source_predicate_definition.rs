use super::type_elaboration::{
    SOURCE_PREDICATE_DEFINITION_TEXT, SourcePredicateDefinitionRouteMutation,
    source_predicate_definition_output, source_predicate_definition_output_with_mutation,
};

const TASK259_CASE: &str = "pass_type_elaboration_predicate_definition_payload_001";
const TASK260_MIXED_CASE: &str = "fail_type_elaboration_predicate_functor_definition_gap_001";

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
