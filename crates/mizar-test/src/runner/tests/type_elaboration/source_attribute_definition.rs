use super::type_elaboration::{
    SOURCE_ATTRIBUTE_DEFINITION_TEXT, SourceAttributeDefinitionRouteMutation,
    source_attribute_definition_output, source_attribute_definition_output_with_mutation,
};

const TASK261_CASE: &str = "pass_type_elaboration_attribute_definition_payload_001";
const TASK261_HISTORICAL_GAP_CASE: &str = "fail_type_elaboration_attribute_definition_gap_001";
const TASK261_MIXED_CASE: &str = "fail_type_elaboration_predicate_functor_definition_gap_001";
const TASK261_SPEC_REF: &str =
    "spec.en.checker.type_elaboration.source_attribute_definition_payload";

#[derive(Debug)]
enum Task261ExpectedSurfaceKind {
    Token(SurfaceTokenKind, &'static str),
    Structural(SurfaceNodeKind),
}

struct Task261ExpectedSurfaceRow {
    kind: Task261ExpectedSurfaceKind,
    start: usize,
    end: usize,
    children: &'static [usize],
}

macro_rules! task261_token_row {
    ($kind:ident, $text:literal, $start:literal, $end:literal) => {
        Task261ExpectedSurfaceRow {
            kind: Task261ExpectedSurfaceKind::Token(SurfaceTokenKind::$kind, $text),
            start: $start,
            end: $end,
            children: &[],
        }
    };
}

macro_rules! task261_structural_row {
    ($kind:ident, $start:literal, $end:literal, [$($child:literal),* $(,)?]) => {
        Task261ExpectedSurfaceRow {
            kind: Task261ExpectedSurfaceKind::Structural(SurfaceNodeKind::$kind),
            start: $start,
            end: $end,
            children: &[$($child),*],
        }
    };
}

const TASK261_EXPECTED_SURFACE_ROWS: &[Task261ExpectedSurfaceRow] = &[
    task261_token_row!(ReservedWord, "definition", 0, 10),
    task261_token_row!(ReservedWord, "let", 13, 16),
    task261_token_row!(Identifier, "x", 17, 18),
    task261_token_row!(ReservedWord, "be", 19, 21),
    task261_token_row!(ReservedWord, "set", 22, 25),
    task261_token_row!(ReservedSymbol, ";", 25, 26),
    task261_token_row!(ReservedWord, "let", 29, 32),
    task261_token_row!(Identifier, "y", 33, 34),
    task261_token_row!(ReservedWord, "be", 35, 37),
    task261_token_row!(ReservedWord, "set", 38, 41),
    task261_token_row!(ReservedSymbol, ";", 41, 42),
    task261_token_row!(ReservedWord, "attr", 45, 49),
    task261_token_row!(Identifier, "Task261AttributeDefinition", 50, 76),
    task261_token_row!(ReservedSymbol, ":", 76, 77),
    task261_token_row!(Identifier, "x", 78, 79),
    task261_token_row!(ReservedWord, "is", 80, 82),
    task261_token_row!(Identifier, "task261_marked", 83, 97),
    task261_token_row!(ReservedWord, "means", 98, 103),
    task261_token_row!(Identifier, "x", 104, 105),
    task261_token_row!(ReservedSymbol, "=", 106, 107),
    task261_token_row!(Identifier, "y", 108, 109),
    task261_token_row!(ReservedSymbol, ";", 109, 110),
    task261_token_row!(ReservedWord, "end", 111, 114),
    task261_token_row!(ReservedSymbol, ";", 114, 115),
    task261_structural_row!(TypeHead, 22, 25, [4]),
    task261_structural_row!(TypeExpression, 22, 25, [24]),
    task261_structural_row!(QualifiedVariableSegment, 17, 25, [2, 3, 25]),
    task261_structural_row!(DefinitionParameter, 13, 26, [1, 26, 5]),
    task261_structural_row!(TypeHead, 38, 41, [9]),
    task261_structural_row!(TypeExpression, 38, 41, [28]),
    task261_structural_row!(QualifiedVariableSegment, 33, 41, [7, 8, 29]),
    task261_structural_row!(DefinitionParameter, 29, 42, [6, 30, 10]),
    task261_structural_row!(AttributePattern, 83, 97, [16]),
    task261_structural_row!(TermReference, 104, 105, [18]),
    task261_structural_row!(TermExpression, 104, 105, [33]),
    task261_structural_row!(TermReference, 108, 109, [20]),
    task261_structural_row!(TermExpression, 108, 109, [35]),
    task261_structural_row!(BuiltinPredicateApplication, 104, 109, [34, 19, 36]),
    task261_structural_row!(FormulaExpression, 104, 109, [37]),
    task261_structural_row!(FormulaDefiniens, 104, 109, [38]),
    task261_structural_row!(
        AttributeDefinition,
        45,
        110,
        [11, 12, 13, 14, 15, 32, 17, 39, 21]
    ),
    task261_structural_row!(DefinitionBlockItem, 0, 115, [0, 27, 31, 40, 22, 23]),
    task261_structural_row!(ItemList, 0, 115, [41]),
    task261_structural_row!(CompilationUnit, 0, 115, [42]),
    task261_structural_row!(
        Root,
        0,
        115,
        [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            43
        ]
    ),
];

const fn task261_range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
    SourceRange {
        source_id,
        start,
        end,
    }
}

#[test]
fn type_elaboration_runner_transports_exact_source_attribute_definition_payload() {
    assert_eq!(SOURCE_ATTRIBUTE_DEFINITION_TEXT.len(), 116);
    assert!(SOURCE_ATTRIBUTE_DEFINITION_TEXT.ends_with('\n'));
    assert!(!SOURCE_ATTRIBUTE_DEFINITION_TEXT.ends_with("\n\n"));
    assert_eq!(
        sha256_text(SOURCE_ATTRIBUTE_DEFINITION_TEXT),
        "ffd4954aad628d7946aaf7afb1b472a6bdfca7bce5ba0cf09f5b284c9dda07bf"
    );

    let (ast, module, shells, symbols, diagnostics) =
        task253_ast_from_source_text_with_diagnostic_count(
            SOURCE_ATTRIBUTE_DEFINITION_TEXT,
            261_000,
        );
    assert_eq!(diagnostics, 0);
    assert_eq!(
        (ast.nodes().len(), ast.root().map(|id| id.index())),
        (45, Some(44))
    );
    assert!(ast.expression_root().is_none());
    assert_eq!(TASK261_EXPECTED_SURFACE_ROWS.len(), 45);
    for (index, (node, expected)) in ast
        .nodes()
        .iter()
        .zip(TASK261_EXPECTED_SURFACE_ROWS)
        .enumerate()
    {
        assert_eq!(
            node.range,
            task261_range(ast.source_id, expected.start, expected.end),
            "surface row {index} range"
        );
        assert!(!node.recovered, "surface row {index} recovery");
        assert_eq!(
            node.children
                .iter()
                .map(|child| child.index())
                .collect::<Vec<_>>(),
            expected.children,
            "surface row {index} ordered children"
        );
        match (&node.kind, &expected.kind) {
            (
                SurfaceNodeKind::Token(actual),
                Task261ExpectedSurfaceKind::Token(expected_kind, expected_text),
            ) => {
                assert_eq!(
                    actual.kind, *expected_kind,
                    "surface row {index} token kind"
                );
                assert_eq!(
                    actual.text.as_ref(),
                    *expected_text,
                    "surface row {index} token text"
                );
            }
            (actual, Task261ExpectedSurfaceKind::Structural(expected_kind)) => {
                assert_eq!(actual, expected_kind, "surface row {index} structural kind");
            }
            (actual, expected) => {
                panic!("surface row {index} kind mismatch: {actual:?} vs {expected:?}")
            }
        }
    }

    let [block, attribute] = shells.declarations() else {
        panic!("Task261 must expose exactly two declaration shells");
    };
    assert_eq!(
        [
            (block.id().index(), block.ordinal(), block.node_id().index()),
            (
                attribute.id().index(),
                attribute.ordinal(),
                attribute.node_id().index(),
            ),
        ],
        [(0, 0, 41), (1, 1, 40)]
    );
    assert_eq!(attribute.parent(), Some(block.id()));
    assert!(shells.exports().is_empty());
    let projections = mizar_resolve::symbols::SignatureProjectionExtractor::new(
        &ast,
        &shells,
        mizar_resolve::env::NamespacePath::new(module.path().as_str()),
    )
    .extract();
    let [projection] = projections.as_slice() else {
        panic!("Task261 must expose one resolver projection");
    };
    assert_eq!(projection.primary_spelling(), "task261_marked");
    assert_eq!(projection.notation_spelling(), Some("task261_marked"));
    assert_eq!(projection.symbol_kind(), SymbolKind::Attribute);
    assert_eq!(
        projection.definition_kind(),
        Some(mizar_resolve::env::DefinitionKind::Attribute)
    );
    assert_eq!(
        projection.overload_policy(),
        mizar_resolve::symbols::SymbolOverloadPolicy::Overloadable
    );
    assert!(projection.arity().is_none());
    assert!(projection.signature().is_some());
    assert_eq!(
        (
            symbols.symbols().len(),
            symbols.definitions().len(),
            symbols.contributions().len(),
        ),
        (1, 1, 1)
    );

    let output = source_attribute_definition_output(
        &ast,
        module,
        &shells,
        &symbols,
        SOURCE_ATTRIBUTE_DEFINITION_TEXT,
    )
    .expect("Task261 exact selector")
    .unwrap_or_else(|error| panic!("Task261 exact route failed: {error}"));
    let context = output.typed_ast.source_context().expect("Task248");
    let source_type = output.typed_ast.source_type().expect("Task249");
    let terms = output.typed_ast.source_term().expect("Task252");
    let atomic = output.typed_ast.source_atomic_formula().expect("Task256");
    let handoff = output
        .typed_ast
        .source_attribute_definition()
        .expect("Task261");
    assert_eq!(
        (
            context.items().len(),
            context.declarations().len(),
            context.binding_env().bindings().len(),
            context.binding_env().contexts().len(),
            context.local_contexts().len(),
            context.context_links().len(),
            context.binding_env().diagnostics().len(),
        ),
        (1, 2, 2, 2, 2, 2, 0)
    );
    assert_eq!(
        (
            source_type.applications().len(),
            source_type.expressions().len(),
            source_type.arguments().len(),
            source_type.definition_returns().len(),
            terms.terms().len(),
            terms.references().len(),
            terms.numeric_type_requests().len(),
        ),
        (2, 2, 0, 0, 2, 2, 0)
    );
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
    assert_eq!(
        (
            handoff.definitions().len(),
            handoff.parameters().len(),
            handoff.subjects().len(),
            handoff.definientia().len(),
        ),
        (1, 2, 1, 1)
    );
    assert_eq!(handoff.source_id(), ast.source_id);
    assert_eq!(handoff.module_id(), symbols.module_id());
    assert_eq!(handoff.source_context_fingerprint(), context.debug_text());
    assert_eq!(handoff.source_type_fingerprint(), source_type.debug_text());
    assert_eq!(handoff.source_term_fingerprint(), terms.debug_text());
    assert_eq!(
        handoff.source_atomic_formula_fingerprint(),
        atomic.debug_text()
    );

    let definition_rows = handoff.definitions().iter().collect::<Vec<_>>();
    let [(definition_id, definition)] = definition_rows.as_slice() else {
        panic!("Task261 definition table changed");
    };
    assert_eq!(definition_id.index(), 0);
    assert_eq!(definition.id().index(), 0);
    assert_eq!(definition.definition().index(), 0);
    assert_eq!(definition.contribution().index(), 0);
    assert_eq!(
        definition.site(),
        &TypedSiteRef::Node(mizar_checker::typed_ast::TypedNodeId::new(40))
    );
    assert_eq!(
        definition.source_range(),
        task261_range(ast.source_id, 45, 110)
    );
    assert_eq!(definition.source_ordinal(), 0);
    assert_eq!(definition.context(), BindingContextId::new(1));
    assert_eq!(
        definition.recovery(),
        mizar_checker::source_attribute_definition::SourceAttributeDefinitionRecovery::Normal
    );
    assert_eq!(
        definition.spelling(),
        "attr Task261AttributeDefinition: x is task261_marked means x = y;"
    );
    assert_eq!(definition.subject().index(), 0);
    assert_eq!(definition.definiens().index(), 0);
    assert_eq!(definition.origin().source_id(), ast.source_id);
    assert_eq!(definition.origin().structural_path(), [4, 0, 7, 0]);
    let resolver_definition = symbols
        .definitions()
        .get(definition.definition())
        .expect("Task261 resolver definition");
    assert_eq!(definition.symbol(), resolver_definition.symbol());
    assert_eq!(definition.origin(), resolver_definition.origin());

    for (index, id, row, binding, written_type, site, start, end, declaration, spelling) in handoff
        .parameters()
        .iter()
        .zip([
            (0, 0, 27, 13, 26, (17, 18), "let x be set;"),
            (1, 1, 31, 29, 42, (33, 34), "let y be set;"),
        ])
        .enumerate()
        .map(|(index, ((id, row), expected))| {
            (
                index, id, row, expected.0, expected.1, expected.2, expected.3, expected.4,
                expected.5, expected.6,
            )
        })
    {
        assert_eq!(id.index(), index);
        assert_eq!(row.id().index(), index);
        assert_eq!(row.owner().index(), 0);
        assert_eq!(row.ordinal(), index);
        assert_eq!(row.binding().index(), binding);
        assert_eq!(row.written_type().index(), written_type);
        assert_eq!(
            row.site(),
            &TypedSiteRef::Node(mizar_checker::typed_ast::TypedNodeId::new(site))
        );
        assert_eq!(row.source_range(), task261_range(ast.source_id, start, end));
        assert_eq!(
            row.declaration_range(),
            task261_range(ast.source_id, declaration.0, declaration.1)
        );
        assert_eq!(row.context(), BindingContextId::new(1));
        assert_eq!(row.spelling(), spelling);
    }
    let subject_rows = handoff.subjects().iter().collect::<Vec<_>>();
    let [(subject_id, subject)] = subject_rows.as_slice() else {
        panic!("Task261 subject table changed");
    };
    assert_eq!(subject_id.index(), 0);
    assert_eq!(subject.id().index(), 0);
    assert_eq!(subject.owner().index(), 0);
    assert_eq!(subject.binding(), BindingId::new(0));
    assert_eq!(
        subject.site(),
        &TypedSiteRef::Node(mizar_checker::typed_ast::TypedNodeId::new(40))
    );
    assert_eq!(subject.source_range(), task261_range(ast.source_id, 78, 79));
    assert_eq!(subject.context(), BindingContextId::new(1));
    assert_eq!(subject.spelling(), "x");
    let definiens_rows = handoff.definientia().iter().collect::<Vec<_>>();
    let [(definiens_id, definiens)] = definiens_rows.as_slice() else {
        panic!("Task261 definiens table changed");
    };
    assert_eq!(definiens_id.index(), 0);
    assert_eq!(definiens.id().index(), 0);
    assert_eq!(definiens.owner().index(), 0);
    assert_eq!(definiens.ordinal(), 0);
    assert_eq!(definiens.formula().index(), 0);
    assert_eq!(
        definiens.site(),
        &TypedSiteRef::Node(mizar_checker::typed_ast::TypedNodeId::new(39))
    );
    assert_eq!(
        definiens.source_range(),
        task261_range(ast.source_id, 104, 109)
    );
    assert_eq!(definiens.context(), BindingContextId::new(1));
    assert_eq!(definiens.spelling(), "x = y");
    assert!(output.typed_ast.initial_obligations().is_empty());
    assert_eq!(
        output.typed_ast.source_attribute_definition(),
        output.resolved.source_attribute_definition()
    );
    assert_eq!(
        output
            .typed_ast
            .debug_text()
            .matches("source-attribute-definition-debug-v1")
            .count(),
        1
    );
    assert_eq!(
        output
            .resolved
            .debug_text()
            .matches("source-attribute-definition-debug-v1")
            .count(),
        1
    );
}

#[derive(Debug, Clone, Copy)]
enum Task261SurfaceMutation {
    StructuralKind,
    StructuralRange,
    StructuralChildren,
    RootIdentity,
    RootChildOrder,
    DefinitionSiblingOrder,
    DefinitionChildRelocation,
    PatternKind,
    LabelToken,
    SubjectToken,
    PatternToken,
    DefiniensToken,
    TokenRecovery,
    ExpressionRoot,
}

fn task261_mutated_surface_ast(ast: &SurfaceAst, mutation: Task261SurfaceMutation) -> SurfaceAst {
    let mut builder = SurfaceAstBuilder::new(ast.source_id);
    let mut rebuilt = Vec::<SurfaceBuilderNodeId>::with_capacity(ast.nodes().len());
    for (index, node) in ast.nodes().iter().enumerate() {
        let kind = match (index, mutation) {
            (25, Task261SurfaceMutation::StructuralKind) => SurfaceNodeKind::AttributeChain,
            (32, Task261SurfaceMutation::PatternKind) => SurfaceNodeKind::PredicatePattern,
            _ => node.kind.clone(),
        };
        let range = match (index, mutation) {
            (25, Task261SurfaceMutation::StructuralRange) => SourceRange {
                source_id: ast.source_id,
                start: 22,
                end: 24,
            },
            _ => node.range,
        };
        let mut children = node
            .children
            .iter()
            .map(|child| rebuilt[child.index()])
            .collect::<Vec<_>>();
        match (index, mutation) {
            (25, Task261SurfaceMutation::StructuralChildren) => children.clear(),
            (44, Task261SurfaceMutation::RootIdentity) => children.clear(),
            (44, Task261SurfaceMutation::RootChildOrder) => children.swap(0, 1),
            (41, Task261SurfaceMutation::DefinitionSiblingOrder) => children.swap(1, 2),
            (40, Task261SurfaceMutation::DefinitionChildRelocation) => {
                children.remove(5);
            }
            (41, Task261SurfaceMutation::DefinitionChildRelocation) => {
                children.insert(3, rebuilt[32]);
            }
            _ => {}
        }
        let rebuilt_id = match kind {
            SurfaceNodeKind::Token(token) => {
                if index == 4 && matches!(mutation, Task261SurfaceMutation::TokenRecovery) {
                    builder.add_recovered_token(token.kind, token.text, range)
                } else {
                    let text = match (index, mutation) {
                        (12, Task261SurfaceMutation::LabelToken) => "Task261CorruptedDefinition",
                        (14, Task261SurfaceMutation::SubjectToken) => "y",
                        (16, Task261SurfaceMutation::PatternToken) => "task261_corrupted",
                        (20, Task261SurfaceMutation::DefiniensToken) => "x",
                        _ => token.text.as_ref(),
                    };
                    builder.add_token(token.kind, text, range)
                }
            }
            structural => builder.add_node(structural, range, children),
        };
        rebuilt.push(rebuilt_id);
    }
    let expression_root = if matches!(mutation, Task261SurfaceMutation::ExpressionRoot) {
        Some(rebuilt[25])
    } else {
        ast.expression_root().map(|id| rebuilt[id.index()])
    };
    let root = if matches!(mutation, Task261SurfaceMutation::RootIdentity) {
        Some(rebuilt[43])
    } else {
        ast.root().map(|id| rebuilt[id.index()])
    };
    builder.finish(root, expression_root)
}

#[test]
fn source_attribute_definition_route_rejects_source_resolver_and_lower_corruption() {
    let (ast, module, shells, symbols) =
        task253_ast_from_source_text(SOURCE_ATTRIBUTE_DEFINITION_TEXT, 261_010);
    for (ordinal, near_source) in [
        SOURCE_ATTRIBUTE_DEFINITION_TEXT
            .trim_end_matches('\n')
            .to_owned(),
        format!("{SOURCE_ATTRIBUTE_DEFINITION_TEXT}\n"),
        SOURCE_ATTRIBUTE_DEFINITION_TEXT.replace("let y be set;", "let z be set;"),
        SOURCE_ATTRIBUTE_DEFINITION_TEXT.replace("Task261AttributeDefinition", "Task261OtherDef"),
        SOURCE_ATTRIBUTE_DEFINITION_TEXT.replace("task261_marked", "task261_other"),
        SOURCE_ATTRIBUTE_DEFINITION_TEXT.replace("x = y", "x = x"),
    ]
    .into_iter()
    .enumerate()
    {
        let (near_ast, near_module, near_shells, near_symbols) =
            task253_ast_from_source_text(&near_source, 261_020 + ordinal);
        assert!(
            source_attribute_definition_output(
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
        Task261SurfaceMutation::StructuralKind,
        Task261SurfaceMutation::StructuralRange,
        Task261SurfaceMutation::StructuralChildren,
        Task261SurfaceMutation::RootIdentity,
        Task261SurfaceMutation::RootChildOrder,
        Task261SurfaceMutation::DefinitionSiblingOrder,
        Task261SurfaceMutation::DefinitionChildRelocation,
        Task261SurfaceMutation::PatternKind,
        Task261SurfaceMutation::LabelToken,
        Task261SurfaceMutation::SubjectToken,
        Task261SurfaceMutation::PatternToken,
        Task261SurfaceMutation::DefiniensToken,
        Task261SurfaceMutation::TokenRecovery,
        Task261SurfaceMutation::ExpressionRoot,
    ] {
        let malformed = task261_mutated_surface_ast(&ast, mutation);
        assert!(
            source_attribute_definition_output(
                &malformed,
                module.clone(),
                &shells,
                &symbols,
                SOURCE_ATTRIBUTE_DEFINITION_TEXT,
            )
            .is_none(),
            "same-source Surface mutation {mutation:?} selected"
        );
    }

    let wrong_module = mizar_resolve::resolved_ast::ModuleId::new(
        mizar_session::PackageId::new("task261"),
        mizar_session::ModulePath::new("task261.wrong"),
    );
    let wrong_module_error = source_attribute_definition_output(
        &ast,
        wrong_module,
        &shells,
        &symbols,
        SOURCE_ATTRIBUTE_DEFINITION_TEXT,
    )
    .expect("exact source remains selected")
    .expect_err("foreign module must fail");
    assert!(wrong_module_error.starts_with("Task261 resolver:"));

    for (mutation, owner) in [
        (
            SourceAttributeDefinitionRouteMutation::RemoveResolverShell,
            "Task261 resolver:",
        ),
        (
            SourceAttributeDefinitionRouteMutation::WrongResolverProjection,
            "Task261 resolver:",
        ),
        (
            SourceAttributeDefinitionRouteMutation::WrongResolverSymbolEntry,
            "Task261 resolver:",
        ),
        (
            SourceAttributeDefinitionRouteMutation::WrongResolverDefinitionEntry,
            "Task261 resolver:",
        ),
        (
            SourceAttributeDefinitionRouteMutation::WrongResolverContribution,
            "Task261 resolver:",
        ),
        (
            SourceAttributeDefinitionRouteMutation::WrongContextModuleSite,
            "Task248 source context:",
        ),
        (
            SourceAttributeDefinitionRouteMutation::WrongContextItemSite,
            "Task248 source context:",
        ),
        (
            SourceAttributeDefinitionRouteMutation::StaleContextItemSite,
            "Task261 attribute definition:",
        ),
        (
            SourceAttributeDefinitionRouteMutation::WrongContextBindingSite(0),
            "Task248 source context:",
        ),
        (
            SourceAttributeDefinitionRouteMutation::WrongContextBindingSite(1),
            "Task248 source context:",
        ),
        (
            SourceAttributeDefinitionRouteMutation::WrongContextBindingOwner(0),
            "Task248 source context:",
        ),
        (
            SourceAttributeDefinitionRouteMutation::WrongContextBindingOwner(1),
            "Task248 source context:",
        ),
        (
            SourceAttributeDefinitionRouteMutation::RemoveTypeExpression,
            "Task249 source type:",
        ),
        (
            SourceAttributeDefinitionRouteMutation::WrongTypeApplicationBinding(0),
            "Task249 source type:",
        ),
        (
            SourceAttributeDefinitionRouteMutation::WrongTypeApplicationBinding(1),
            "Task249 source type:",
        ),
        (
            SourceAttributeDefinitionRouteMutation::WrongTypeApplicationRoot(0),
            "Task249 source type:",
        ),
        (
            SourceAttributeDefinitionRouteMutation::WrongTypeApplicationRoot(1),
            "Task249 source type:",
        ),
        (
            SourceAttributeDefinitionRouteMutation::WrongTypeExpressionSite(0),
            "Task249 source type:",
        ),
        (
            SourceAttributeDefinitionRouteMutation::WrongTypeExpressionSite(1),
            "Task249 source type:",
        ),
        (
            SourceAttributeDefinitionRouteMutation::WrongTermBinding(0),
            "Task252 source term:",
        ),
        (
            SourceAttributeDefinitionRouteMutation::WrongTermBinding(1),
            "Task252 source term:",
        ),
        (
            SourceAttributeDefinitionRouteMutation::WrongTermSite(0),
            "Task252 source term:",
        ),
        (
            SourceAttributeDefinitionRouteMutation::WrongTermSite(1),
            "Task252 source term:",
        ),
        (
            SourceAttributeDefinitionRouteMutation::RemoveAtomicFormula,
            "Task256 atomic formula:",
        ),
        (
            SourceAttributeDefinitionRouteMutation::RemoveAtomicEdge,
            "Task256 atomic formula:",
        ),
        (
            SourceAttributeDefinitionRouteMutation::WrongAtomicFormula,
            "Task256 atomic formula:",
        ),
        (
            SourceAttributeDefinitionRouteMutation::WrongAtomicEdge(0),
            "Task256 atomic formula:",
        ),
        (
            SourceAttributeDefinitionRouteMutation::WrongAtomicEdge(1),
            "Task256 atomic formula:",
        ),
        (
            SourceAttributeDefinitionRouteMutation::WrongAtomicRequest(0),
            "Task256 atomic formula:",
        ),
        (
            SourceAttributeDefinitionRouteMutation::WrongAtomicRequest(1),
            "Task256 atomic formula:",
        ),
        (
            SourceAttributeDefinitionRouteMutation::RemoveAttributeParameter,
            "Task261 attribute definition:",
        ),
        (
            SourceAttributeDefinitionRouteMutation::RemoveAttributeSubject,
            "Task261 attribute definition:",
        ),
        (
            SourceAttributeDefinitionRouteMutation::WrongAttributeParameterOwner,
            "Task261 attribute definition:",
        ),
        (
            SourceAttributeDefinitionRouteMutation::WrongAttributeSubjectBinding,
            "Task261 attribute definition:",
        ),
        (
            SourceAttributeDefinitionRouteMutation::WrongAttributeDefiniensFormula,
            "Task261 attribute definition:",
        ),
    ] {
        let error = source_attribute_definition_output_with_mutation(
            &ast,
            module.clone(),
            &shells,
            &symbols,
            SOURCE_ATTRIBUTE_DEFINITION_TEXT,
            mutation,
        )
        .expect("exact source remains selected")
        .expect_err("mutation must fail at its owner");
        assert!(error.starts_with(owner), "{mutation:?}: {error}");
    }

    let first = source_attribute_definition_output(
        &ast,
        module.clone(),
        &shells,
        &symbols,
        SOURCE_ATTRIBUTE_DEFINITION_TEXT,
    )
    .expect("valid replay selector")
    .expect("valid replay route");
    let second = source_attribute_definition_output(
        &ast,
        module,
        &shells,
        &symbols,
        SOURCE_ATTRIBUTE_DEFINITION_TEXT,
    )
    .expect("second replay selector")
    .expect("second replay route");
    assert_eq!(first.typed_ast.debug_text(), second.typed_ast.debug_text());
    assert_eq!(first.resolved.debug_text(), second.resolved.debug_text());
}

#[test]
fn source_attribute_definition_route_selection_is_source_only_and_trace_is_reciprocal() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("mizar-test crate should live below workspace root")
        .to_path_buf();
    let config = DiscoveryConfig {
        workspace_root: workspace_root.clone(),
        tests_root: workspace_root.join("tests"),
        manifest_path: workspace_root.join("tests/coverage/spec_trace.toml"),
        profile: TestProfile::Fast,
        validation_mode: ValidationMode::Metadata,
    };
    let plan = build_test_plan(&config).expect("Task261 repository plan should build");
    let selected = active_type_elaboration_cases(&plan)
        .filter(|case| {
            std::fs::read_to_string(&case.source_path)
                .is_ok_and(|source| source == SOURCE_ATTRIBUTE_DEFINITION_TEXT)
        })
        .collect::<Vec<_>>();
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].id.0, TASK261_CASE);
    assert!(selected[0].expectation_path.ends_with(Path::new(
        "tests/miz/pass/types/pass_type_elaboration_attribute_definition_payload_001.expect.toml"
    )));

    let (ordinal, case) = active_type_elaboration_cases(&plan)
        .enumerate()
        .find(|(_, case)| case.id.0 == TASK261_CASE)
        .expect("Task261 active sidecar");
    assert_eq!(case.expectation.schema_version, 1);
    assert_eq!(case.expectation.kind, crate::expectation::TestKind::Pass);
    assert_eq!(
        case.expectation.stage,
        crate::staged_model::Stage::TypeElaboration
    );
    assert_eq!(case.expectation.domain, "checker.type_elaboration");
    assert_eq!(
        case.expectation.expected_outcome,
        crate::expectation::ExpectedOutcome::Pass
    );
    assert_eq!(
        case.expectation.expected_phase,
        Some(crate::expectation::PipelinePhase::TypeCheck)
    );
    assert_eq!(
        case.expectation
            .spec_refs
            .iter()
            .map(|reference| reference.0.as_str())
            .collect::<Vec<_>>(),
        [TASK261_SPEC_REF]
    );
    assert_eq!(case.expectation.tags, ["active_type_elaboration"]);
    assert!(case.expectation.diagnostic_codes.is_empty());
    assert!(case.expectation.diagnostic_payloads.is_empty());
    assert!(case.expectation.failure_category.is_none());
    assert!(case.expectation.rejection_reason.is_none());
    assert!(case.expectation.stable_detail_key.is_none());
    let requirement = plan
        .manifest
        .requirements
        .iter()
        .find(|requirement| requirement.id.0 == TASK261_SPEC_REF)
        .expect("Task261 trace row");
    assert_eq!(
        requirement.source,
        Path::new("doc/design/mizar-checker/en/source_attribute_definition.md")
    );
    assert_eq!(requirement.section, "Dedicated Consumer And Trace Intent");
    assert_eq!(
        requirement.stage,
        crate::staged_model::Stage::TypeElaboration
    );
    assert_eq!(
        requirement.status,
        crate::traceability::RequirementStatus::Covered
    );
    assert_eq!(
        requirement.coverage,
        crate::traceability::CoverageShape::Pass
    );
    assert!(requirement.required);
    assert_eq!(
        requirement.tests,
        [Path::new(
            "tests/miz/pass/types/pass_type_elaboration_attribute_definition_payload_001.expect.toml"
        )]
    );
    assert_eq!(
        plan.cases
            .iter()
            .filter(|candidate| candidate
                .expectation
                .spec_refs
                .iter()
                .any(|reference| reference.0 == TASK261_SPEC_REF))
            .map(|candidate| candidate.id.0.as_str())
            .collect::<Vec<_>>(),
        [TASK261_CASE]
    );
    let result = run_type_elaboration_case(
        &workspace_root,
        &workspace_root.join("tests"),
        case,
        ordinal,
    );
    assert_eq!(result.status, TypeElaborationCaseStatus::Passed);
    assert!(result.actual_detail_keys.is_empty());

    for (source, ordinal) in [
        (
            super::type_elaboration::SOURCE_PREDICATE_DEFINITION_TEXT,
            261_090,
        ),
        (
            super::type_elaboration::SOURCE_FUNCTOR_DEFINITION_TEXT,
            261_091,
        ),
    ] {
        let (ast, module, shells, symbols) = task253_ast_from_source_text(source, ordinal);
        assert!(
            source_attribute_definition_output(&ast, module, &shells, &symbols, source).is_none()
        );
    }
    for (case_id, ordinal) in [
        (TASK261_HISTORICAL_GAP_CASE, 261_092),
        (TASK261_MIXED_CASE, 261_093),
    ] {
        let (case_ordinal, old_case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| case.id.0 == case_id)
            .expect("Task261 isolation boundary remains active");
        let source = std::fs::read_to_string(&old_case.source_path).expect("boundary source");
        let (ast, module, shells, symbols) = task253_ast_from_source_text(&source, ordinal);
        assert!(
            source_attribute_definition_output(&ast, module, &shells, &symbols, &source).is_none(),
            "boundary {case_id} entered Task261 selector"
        );
        let old_result = run_type_elaboration_case(
            &workspace_root,
            &workspace_root.join("tests"),
            old_case,
            case_ordinal,
        );
        assert_eq!(old_result.status, TypeElaborationCaseStatus::Passed);
        assert_eq!(
            old_result.actual_detail_keys,
            old_case.expectation.diagnostic_payloads
        );
    }
}

#[test]
fn source_attribute_definition_route_publishes_no_semantic_outputs() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("mizar-test crate should live below workspace root")
        .to_path_buf();
    let plan = build_test_plan(&DiscoveryConfig {
        workspace_root: workspace_root.clone(),
        tests_root: workspace_root.join("tests"),
        manifest_path: workspace_root.join("tests/coverage/spec_trace.toml"),
        profile: TestProfile::Fast,
        validation_mode: ValidationMode::Metadata,
    })
    .expect("Task261 repository count plan");
    let has_active_type_tag = |case: &&crate::harness::TestCase| {
        case.expectation
            .tags
            .iter()
            .any(|tag| tag == "active_type_elaboration")
    };
    assert_eq!(
        [
            active_type_elaboration_cases(&plan).count(),
            plan.cases.iter().filter(has_active_type_tag).count(),
            plan.cases
                .iter()
                .filter(has_active_type_tag)
                .filter(|case| case.expectation.stage == crate::staged_model::Stage::TypeElaboration)
                .count(),
            plan.cases
                .iter()
                .filter(has_active_type_tag)
                .filter(|case| case.expectation.expected_phase == Some(crate::expectation::PipelinePhase::TypeCheck))
                .count(),
            plan.cases
                .iter()
                .filter(has_active_type_tag)
                .filter(|case| matches!(case.expectation.expected_outcome, crate::expectation::ExpectedOutcome::Pass | crate::expectation::ExpectedOutcome::Fail))
                .count(),
            plan.cases
                .iter()
                .filter(has_active_type_tag)
                .filter(|case| case.source_path.extension().is_some_and(|ext| ext == "miz"))
                .count(),
        ],
        [205; 6]
    );
    assert_eq!(
        (plan.cases.len(), plan.manifest.requirements.len()),
        (556, 499)
    );
    assert_eq!(
        plan.cases.iter().fold((0, 0), |(pass, fail), case| {
            match case.expectation.expected_outcome {
                crate::expectation::ExpectedOutcome::Pass => (pass + 1, fail),
                crate::expectation::ExpectedOutcome::Fail => (pass, fail + 1),
                _ => (pass, fail),
            }
        }),
        (313, 243)
    );
    assert_eq!(
        (
            crate::active_parse_only_cases(&plan).count(),
            crate::active_declaration_symbol_cases(&plan).count(),
            active_type_elaboration_cases(&plan).count(),
            crate::active_proof_verification_cases(&plan).count(),
        ),
        (107, 7, 205, 1)
    );
    let type_stage = plan
        .coverage_report
        .stages
        .iter()
        .find(|stage| stage.stage == crate::staged_model::Stage::TypeElaboration)
        .expect("Task261 type coverage stage");
    assert_eq!((type_stage.requirements, type_stage.covered), (307, 295));
    assert_eq!(
        (plan.warning_count(), plan.error_count()),
        (23, 0),
        "{:#?}",
        plan.diagnostics
    );

    let (ast, module, shells, symbols) =
        task253_ast_from_source_text(SOURCE_ATTRIBUTE_DEFINITION_TEXT, 261_100);
    let symbols_before = symbols.clone();
    let first = source_attribute_definition_output(
        &ast,
        module.clone(),
        &shells,
        &symbols,
        SOURCE_ATTRIBUTE_DEFINITION_TEXT,
    )
    .expect("Task261 selector")
    .expect("Task261 route");
    let second = source_attribute_definition_output(
        &ast,
        module,
        &shells,
        &symbols,
        SOURCE_ATTRIBUTE_DEFINITION_TEXT,
    )
    .expect("Task261 replay selector")
    .expect("Task261 replay route");
    assert_eq!(
        symbols, symbols_before,
        "Task261 must not activate resolver data"
    );
    assert!(first.typed_ast.initial_obligations().is_empty());
    assert!(first.typed_ast.facts().is_empty());
    assert!(first.typed_ast.types().is_empty());
    assert!(first.typed_ast.coercions().is_empty());
    assert!(first.typed_ast.diagnostics().is_empty());
    assert!(first.typed_ast.source_attribute().is_none());
    assert!(first.typed_ast.source_evidence().is_none());
    assert!(first.typed_ast.source_application().is_none());
    assert!(first.typed_ast.source_structure().is_none());
    assert!(first.typed_ast.source_set_term().is_none());
    assert!(first.typed_ast.source_predicate_definition().is_none());
    assert!(first.typed_ast.source_functor_definition().is_none());
    assert!(first.typed_ast.source_composite_formula().is_none());
    assert!(first.typed_ast.source_formula_composition().is_none());
    assert!(
        first
            .typed_ast
            .source_condition_formula_composition()
            .is_none()
    );
    assert!(
        first
            .typed_ast
            .source_predicate_chain_composition()
            .is_none()
    );
    assert!(first.typed_ast.source_statement().is_none());
    assert!(first.resolved.checked_formulas().is_empty());
    assert!(first.resolved.expr_metadata().is_empty());
    assert!(first.resolved.collection_candidates().is_empty());
    assert!(first.resolved.expanded_candidates().is_empty());
    assert!(first.resolved.template_expansions().is_empty());
    assert!(first.resolved.viable_candidates().is_empty());
    assert!(first.resolved.viability_decisions().is_empty());
    assert!(first.resolved.specificity_graphs().is_empty());
    assert!(first.resolved.resolved_overloads().is_empty());
    assert!(first.resolved.inserted_coercions().is_empty());
    assert!(first.resolved.statement_semantics().is_empty());
    assert!(first.resolved.checked_proofs().is_empty());
    assert!(first.resolved.checked_proof_nodes().is_empty());
    assert!(first.resolved.checked_terminal_goals().is_empty());
    assert!(first.resolved.cluster_facts().is_empty());
    assert!(first.resolved.diagnostics().is_empty());
    assert!(first.resolved.source_predicate_definition().is_none());
    assert!(first.resolved.source_functor_definition().is_none());
    assert!(first.resolved.source_statement().is_none());
    for index in [12, 14, 16, 32] {
        let node = first
            .typed_ast
            .nodes()
            .node(mizar_checker::typed_ast::TypedNodeId::new(index))
            .expect("Task261 excluded label/subject/pattern subtree is preserved");
        assert_eq!(node.typing, mizar_checker::typed_ast::TypingState::Unknown);
        assert!(node.links.facts.is_empty());
        assert!(node.links.initial_obligations.is_empty());
        assert!(node.links.diagnostics.is_empty());
    }
    assert_eq!(
        first.typed_ast.source_attribute_definition(),
        first.resolved.source_attribute_definition()
    );
    assert_eq!(first.typed_ast.debug_text(), second.typed_ast.debug_text());
    assert_eq!(first.resolved.debug_text(), second.resolved.debug_text());
    let debug = format!(
        "{}{}",
        first.typed_ast.debug_text(),
        first.resolved.debug_text()
    );
    for forbidden in [
        "accepted",
        "activated",
        "axiom",
        "core-ir",
        "control-flow",
        "verification-condition",
    ] {
        assert!(
            !debug.to_ascii_lowercase().contains(forbidden),
            "Task261 leaked deferred semantic marker {forbidden}"
        );
    }
}

#[test]
fn task261_core_item_context_association_is_exact_and_deterministic() {
    let (ast, module, shells, symbols) =
        task253_ast_from_source_text(SOURCE_ATTRIBUTE_DEFINITION_TEXT, 261_200);
    let output = source_attribute_definition_output(
        &ast,
        module,
        &shells,
        &symbols,
        SOURCE_ATTRIBUTE_DEFINITION_TEXT,
    )
    .expect("Task261 selector")
    .expect("Task261 route");
    let source_context = output
        .typed_ast
        .source_context()
        .expect("Task248 source context")
        .clone();
    let checker_owner = output
        .typed_ast
        .source_attribute_definition()
        .expect("Task261 checker owner")
        .clone();
    let source_bindings = task261_source_binding_core_handoff(&source_context, &checker_owner);
    let expected_source_bindings = source_bindings.clone();
    let first = mizar_core::elaborator::SourceAttributeCoreContextProducer::build(
        source_bindings.clone(),
        source_context.clone(),
        checker_owner.clone(),
    )
    .expect("Task261 Core item context");
    let second = mizar_core::elaborator::SourceAttributeCoreContextProducer::build(
        source_bindings,
        source_context.clone(),
        checker_owner.clone(),
    )
    .expect("Task261 deterministic replay");
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
        .get(mizar_checker::source_attribute_definition::SourceAttributeDefinitionId::new(0))
        .expect("Task261 definition");
    let source_item = source_context
        .context_links()
        .get(definition.context())
        .expect("Task248 definition link")
        .item
        .expect("Task248 containing source item");
    let association = first
        .items()
        .get(definition.id())
        .expect("Task261 association");
    assert_eq!(association.definition(), definition.id());
    assert_eq!(association.source_item(), source_item);
    assert_eq!(association.symbol(), definition.symbol());
    let core_item = first
        .context()
        .item_registry()
        .id_for_symbol(definition.symbol())
        .expect("Core attribute item");
    assert_eq!(association.core_item(), core_item);
    assert_eq!(
        first
            .items()
            .iter()
            .map(|(id, row)| (id, row.source_item(), row.symbol().clone(), row.core_item()))
            .collect::<Vec<_>>(),
        vec![(definition.id(), source_item, definition.symbol().clone(), core_item)]
    );
    assert_eq!(first.context().item_registry().items().len(), 1);
    assert!(first.context().dependency_summaries().is_empty());
    assert!(first.context().generated_origins().table().is_empty());
    assert!(first.context().diagnostics().is_empty());

    let item = first
        .context()
        .item_registry()
        .items()
        .get(core_item)
        .expect("Core attribute row");
    assert_eq!(item.symbol, *definition.symbol());
    assert_eq!(item.kind, mizar_core::core_ir::CoreItemKind::Attribute);
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
            "source-attribute-core-item-v1.definition.0",
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
            "source-attribute-core-item-v1.definition.0",
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
fn task261_core_item_context_default_deny_mutations_and_foreign_environment() {
    let (ast, module, shells, symbols) =
        task253_ast_from_source_text(SOURCE_ATTRIBUTE_DEFINITION_TEXT, 261_210);
    let output = source_attribute_definition_output(
        &ast,
        module,
        &shells,
        &symbols,
        SOURCE_ATTRIBUTE_DEFINITION_TEXT,
    )
    .expect("Task261 selector")
    .expect("Task261 route");
    let source_context = output
        .typed_ast
        .source_context()
        .expect("Task248 source context")
        .clone();
    let checker_owner = output
        .typed_ast
        .source_attribute_definition()
        .expect("Task261 checker owner")
        .clone();
    for mutation in [
        Task261CoreContextMutation::MissingItem,
        Task261CoreContextMutation::ExtraItem,
        Task261CoreContextMutation::WrongKind,
        Task261CoreContextMutation::WrongVisibility,
        Task261CoreContextMutation::WrongSource,
        Task261CoreContextMutation::WrongProvenance,
        Task261CoreContextMutation::MissingBoundary,
        Task261CoreContextMutation::WrongBoundary,
        Task261CoreContextMutation::UnexpectedDependency,
        Task261CoreContextMutation::InvalidStatus,
    ] {
        let source_bindings =
            task261_source_binding_core_handoff_with_mutation(&source_context, &checker_owner, mutation);
        let error = mizar_core::elaborator::SourceAttributeCoreContextProducer::build(
            source_bindings,
            source_context.clone(),
            checker_owner.clone(),
        )
        .expect_err("Core mutation must fail closed");
        assert_eq!(
            error,
            mizar_core::elaborator::SourceAttributeCoreContextError::InvalidCoreContext,
            "{mutation:?}"
        );
    }

    let (foreign_ast, foreign_module, foreign_shells, foreign_symbols) =
        task253_ast_from_source_text(SOURCE_ATTRIBUTE_DEFINITION_TEXT, 261_211);
    let foreign_output = source_attribute_definition_output(
        &foreign_ast,
        foreign_module,
        &foreign_shells,
        &foreign_symbols,
        SOURCE_ATTRIBUTE_DEFINITION_TEXT,
    )
    .expect("foreign Task261 selector")
    .expect("foreign Task261 route");
    let foreign_context = foreign_output
        .typed_ast
        .source_context()
        .expect("foreign source context")
        .clone();
    let foreign_owner = foreign_output
        .typed_ast
        .source_attribute_definition()
        .expect("foreign checker owner")
        .clone();
    let base_bindings = task261_source_binding_core_handoff(&source_context, &checker_owner);
    let foreign_bindings = task261_source_binding_core_handoff(&foreign_context, &foreign_owner);
    for (bindings, context, owner) in [
        (
            base_bindings.clone(),
            foreign_context.clone(),
            foreign_owner.clone(),
        ),
        (
            base_bindings.clone(),
            foreign_context.clone(),
            checker_owner.clone(),
        ),
        (
            base_bindings.clone(),
            source_context.clone(),
            foreign_owner.clone(),
        ),
        (foreign_bindings, source_context, checker_owner),
    ] {
        let error = mizar_core::elaborator::SourceAttributeCoreContextProducer::build(
            bindings, context, owner,
        )
        .expect_err("foreign environment must fail closed");
        assert_eq!(
            error,
            mizar_core::elaborator::SourceAttributeCoreContextError::EnvironmentMismatch
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Task261CoreContextMutation {
    Baseline,
    MissingItem,
    ExtraItem,
    WrongKind,
    WrongVisibility,
    WrongSource,
    WrongProvenance,
    MissingBoundary,
    WrongBoundary,
    UnexpectedDependency,
    InvalidStatus,
}

fn task261_source_binding_core_handoff(
    source_context: &mizar_checker::source_context::SourceBindingContextHandoff,
    checker_owner: &mizar_checker::source_attribute_definition::SourceAttributeDefinitionHandoff,
) -> mizar_core::elaborator::SourceBindingCoreContextHandoff {
    task261_source_binding_core_handoff_with_mutation(
        source_context,
        checker_owner,
        Task261CoreContextMutation::Baseline,
    )
}

fn task261_source_binding_core_handoff_with_mutation(
    source_context: &mizar_checker::source_context::SourceBindingContextHandoff,
    checker_owner: &mizar_checker::source_attribute_definition::SourceAttributeDefinitionHandoff,
    mutation: Task261CoreContextMutation,
) -> mizar_core::elaborator::SourceBindingCoreContextHandoff {
    let definition = checker_owner
        .definitions()
        .get(mizar_checker::source_attribute_definition::SourceAttributeDefinitionId::new(0))
        .expect("Task261 definition");
    let source_range = if mutation == Task261CoreContextMutation::WrongSource {
        mizar_session::SourceRange {
            source_id: source_context.source_id(),
            start: definition.source_range().start + 1,
            end: definition.source_range().end,
        }
    } else {
        definition.source_range()
    };
    let provenance_key = if mutation == Task261CoreContextMutation::WrongProvenance {
        "wrong-task261-provenance"
    } else {
        "source-attribute-core-item-v1.definition.0"
    };
    let source = mizar_core::core_ir::CoreSourceRef::direct(source_range).with_provenance(vec![
        mizar_core::core_ir::CoreProvenance::new(
            mizar_core::core_ir::CoreProvenancePhase::Checker,
            provenance_key,
        ),
    ]);
    let kind = if mutation == Task261CoreContextMutation::WrongKind {
        mizar_core::core_ir::CoreItemKind::Predicate
    } else {
        mizar_core::core_ir::CoreItemKind::Attribute
    };
    let visibility = if mutation == Task261CoreContextMutation::WrongVisibility {
        "private"
    } else {
        "public"
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
        mizar_core::elaborator::CheckerOwnedProvenance::checker(provenance_key),
    );
    let seed = if mutation == Task261CoreContextMutation::MissingBoundary {
        seed
    } else if mutation == Task261CoreContextMutation::WrongBoundary {
        seed.with_definition_boundary(mizar_core::elaborator::DefinitionBoundaryKind::Theorem)
    } else {
        seed.with_definition_boundary(
            mizar_core::elaborator::DefinitionBoundaryKind::DefinitionalItem,
        )
    };
    let seed = if mutation == Task261CoreContextMutation::UnexpectedDependency {
        seed.with_dependencies(vec![definition.symbol().clone()])
    } else if mutation == Task261CoreContextMutation::InvalidStatus {
        seed.with_dependencies(vec![mizar_resolve::resolved_ast::SymbolId::new(
            definition.symbol().module().clone(),
            mizar_resolve::resolved_ast::LocalSymbolId::new("task261-missing"),
            mizar_resolve::resolved_ast::FullyQualifiedName::new(format!(
                "{}.task261-missing",
                definition.symbol().fqn().as_str()
            )),
        )])
    } else {
        seed
    };
    if mutation != Task261CoreContextMutation::MissingItem {
        input.item_seeds.push(seed);
    }
    if mutation == Task261CoreContextMutation::ExtraItem {
        let extra_symbol = mizar_resolve::resolved_ast::SymbolId::new(
            definition.symbol().module().clone(),
            mizar_resolve::resolved_ast::LocalSymbolId::new("task261-extra"),
            mizar_resolve::resolved_ast::FullyQualifiedName::new(format!(
                "{}.task261-extra",
                definition.symbol().fqn().as_str()
            )),
        );
        input.item_seeds.push(
            mizar_core::elaborator::CoreItemSeed::new(
                extra_symbol,
                mizar_core::core_ir::CoreItemKind::Attribute,
                "public",
                mizar_core::core_ir::CoreSourceRef::direct(definition.source_range())
                    .with_provenance(vec![mizar_core::core_ir::CoreProvenance::new(
                        mizar_core::core_ir::CoreProvenancePhase::Checker,
                        "source-attribute-core-item-v1.definition.extra",
                    )]),
                mizar_core::elaborator::CheckerOwnedProvenance::checker(
                    "source-attribute-core-item-v1.definition.extra",
                ),
            )
            .with_definition_boundary(
                mizar_core::elaborator::DefinitionBoundaryKind::DefinitionalItem,
            ),
        );
    }
    let context = mizar_core::elaborator::prepare_core_context(input)
        .expect("Task261 Core context seed should prepare");
    mizar_core::elaborator::SourceBindingCoreContextProducer::build(
        context,
        source_context.binding_env().clone(),
    )
    .expect("Task261 33LB handoff should build")
}
