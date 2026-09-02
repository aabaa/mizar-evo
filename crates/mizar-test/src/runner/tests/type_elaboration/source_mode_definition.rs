use super::type_elaboration::{
    SOURCE_MODE_DEFINITION_TEXT, SourceModeDefinitionRouteMutation, source_mode_definition_output,
    source_mode_definition_output_with_mutation,
};
use mizar_checker::typed_ast::TypedNodeId;

const TASK262_CASE: &str = "pass_type_elaboration_mode_definition_payload_001";
const TASK262_SPEC_REF: &str = "spec.en.checker.type_elaboration.source_mode_definition_payload";
const TASK262_HISTORICAL_GAP_CASE: &str = "fail_type_elaboration_mode_structure_definition_gap_001";
const TASK262_MIXED_CASE: &str = "fail_type_elaboration_predicate_functor_definition_gap_001";

#[derive(Debug)]
enum Task262ExpectedSurfaceKind {
    Token(SurfaceTokenKind, &'static str),
    Structural(SurfaceNodeKind),
}

struct Task262ExpectedSurfaceRow {
    kind: Task262ExpectedSurfaceKind,
    start: usize,
    end: usize,
    children: &'static [usize],
}

macro_rules! task262_token_row {
    ($kind:ident, $text:literal, $start:literal, $end:literal) => {
        Task262ExpectedSurfaceRow {
            kind: Task262ExpectedSurfaceKind::Token(SurfaceTokenKind::$kind, $text),
            start: $start,
            end: $end,
            children: &[],
        }
    };
}

macro_rules! task262_structural_row {
    ($kind:ident, $start:literal, $end:literal, [$($child:literal),* $(,)?]) => {
        Task262ExpectedSurfaceRow {
            kind: Task262ExpectedSurfaceKind::Structural(SurfaceNodeKind::$kind),
            start: $start,
            end: $end,
            children: &[$($child),*],
        }
    };
}

const TASK262_EXPECTED_SURFACE_ROWS: &[Task262ExpectedSurfaceRow] = &[
    task262_token_row!(ReservedWord, "definition", 0, 10),
    task262_token_row!(ReservedWord, "let", 13, 16),
    task262_token_row!(Identifier, "x", 17, 18),
    task262_token_row!(ReservedWord, "be", 19, 21),
    task262_token_row!(ReservedWord, "set", 22, 25),
    task262_token_row!(ReservedSymbol, ";", 25, 26),
    task262_token_row!(ReservedWord, "let", 29, 32),
    task262_token_row!(Identifier, "y", 33, 34),
    task262_token_row!(ReservedWord, "be", 35, 37),
    task262_token_row!(ReservedWord, "set", 38, 41),
    task262_token_row!(ReservedSymbol, ";", 41, 42),
    task262_token_row!(ReservedWord, "mode", 45, 49),
    task262_token_row!(Identifier, "Task262ModeDefinition", 50, 71),
    task262_token_row!(ReservedSymbol, ":", 71, 72),
    task262_token_row!(Identifier, "Task262Mode", 73, 84),
    task262_token_row!(ReservedSymbol, "[", 85, 86),
    task262_token_row!(Identifier, "x", 86, 87),
    task262_token_row!(ReservedSymbol, ",", 87, 88),
    task262_token_row!(Identifier, "y", 89, 90),
    task262_token_row!(ReservedSymbol, "]", 90, 91),
    task262_token_row!(ReservedWord, "is", 92, 94),
    task262_token_row!(ReservedWord, "set", 95, 98),
    task262_token_row!(ReservedSymbol, ";", 98, 99),
    task262_token_row!(ReservedWord, "sethood", 102, 109),
    task262_token_row!(ReservedWord, "by", 110, 112),
    task262_token_row!(ReservedWord, "computation", 113, 124),
    task262_token_row!(ReservedSymbol, "(", 124, 125),
    task262_token_row!(Identifier, "steps", 125, 130),
    task262_token_row!(ReservedSymbol, ":", 130, 131),
    task262_token_row!(Numeral, "1", 132, 133),
    task262_token_row!(ReservedSymbol, ")", 133, 134),
    task262_token_row!(ReservedSymbol, ";", 134, 135),
    task262_token_row!(ReservedWord, "end", 136, 139),
    task262_token_row!(ReservedSymbol, ";", 139, 140),
    task262_structural_row!(TypeHead, 22, 25, [4]),
    task262_structural_row!(TypeExpression, 22, 25, [34]),
    task262_structural_row!(QualifiedVariableSegment, 17, 25, [2, 3, 35]),
    task262_structural_row!(DefinitionParameter, 13, 26, [1, 36, 5]),
    task262_structural_row!(TypeHead, 38, 41, [9]),
    task262_structural_row!(TypeExpression, 38, 41, [38]),
    task262_structural_row!(QualifiedVariableSegment, 33, 41, [7, 8, 39]),
    task262_structural_row!(DefinitionParameter, 29, 42, [6, 40, 10]),
    task262_structural_row!(ModePattern, 73, 91, [14, 15, 16, 17, 18, 19]),
    task262_structural_row!(TypeHead, 95, 98, [21]),
    task262_structural_row!(TypeExpression, 95, 98, [43]),
    task262_structural_row!(ComputationOption, 125, 133, [27, 28, 29]),
    task262_structural_row!(ComputationJustification, 113, 134, [25, 26, 45, 30]),
    task262_structural_row!(JustificationClause, 110, 134, [24, 46]),
    task262_structural_row!(ModeProperty, 102, 135, [23, 47, 31]),
    task262_structural_row!(ModeDefinition, 45, 135, [11, 12, 13, 42, 20, 44, 22, 48]),
    task262_structural_row!(DefinitionBlockItem, 0, 140, [0, 37, 41, 49, 32, 33]),
    task262_structural_row!(ItemList, 0, 140, [50]),
    task262_structural_row!(CompilationUnit, 0, 140, [51]),
    task262_structural_row!(
        Root,
        0,
        140,
        [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 52
        ]
    ),
];

const fn task262_range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
    SourceRange {
        source_id,
        start,
        end,
    }
}

#[test]
fn task262_mode_definition_source_consumer_is_exact() {
    assert_eq!(SOURCE_MODE_DEFINITION_TEXT.len(), 141);
    assert!(SOURCE_MODE_DEFINITION_TEXT.ends_with('\n'));
    assert!(!SOURCE_MODE_DEFINITION_TEXT.ends_with("\n\n"));
    assert_eq!(
        sha256_text(SOURCE_MODE_DEFINITION_TEXT),
        "3271f243670bd781c7167ff0d3bf463263a318abbe261aabdde1842c532a725e"
    );

    let (ast, module, shells, symbols, diagnostics) =
        task253_ast_from_source_text_with_diagnostic_count(SOURCE_MODE_DEFINITION_TEXT, 262_000);
    assert_eq!(diagnostics, 0);
    assert_eq!(
        (ast.nodes().len(), ast.root().map(|id| id.index())),
        (54, Some(53))
    );
    assert!(ast.expression_root().is_none());
    assert_eq!(TASK262_EXPECTED_SURFACE_ROWS.len(), 54);
    for (index, (node, expected)) in ast
        .nodes()
        .iter()
        .zip(TASK262_EXPECTED_SURFACE_ROWS)
        .enumerate()
    {
        assert_eq!(
            node.range,
            task262_range(ast.source_id, expected.start, expected.end),
            "surface row {index} range"
        );
        assert!(!node.recovered, "surface row {index} recovery");
        assert_eq!(
            node.children
                .iter()
                .map(|child| child.index())
                .collect::<Vec<_>>(),
            expected.children,
            "surface row {index} children"
        );
        match (&node.kind, &expected.kind) {
            (
                SurfaceNodeKind::Token(actual),
                Task262ExpectedSurfaceKind::Token(expected_kind, expected_text),
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
            (actual, Task262ExpectedSurfaceKind::Structural(expected_kind)) => {
                assert_eq!(actual, expected_kind, "surface row {index} structural kind");
            }
            (actual, expected) => panic!("surface row {index}: {actual:?} != {expected:?}"),
        }
    }

    let [block, mode_shell] = shells.declarations() else {
        panic!("Task262 declaration shells changed");
    };
    assert_eq!(
        [
            (block.id().index(), block.ordinal(), block.node_id().index()),
            (
                mode_shell.id().index(),
                mode_shell.ordinal(),
                mode_shell.node_id().index()
            ),
        ],
        [(0, 0, 50), (1, 1, 49)]
    );
    assert_eq!(mode_shell.parent(), Some(block.id()));
    assert!(shells.exports().is_empty());
    let projections = mizar_resolve::symbols::SignatureProjectionExtractor::new(
        &ast,
        &shells,
        mizar_resolve::env::NamespacePath::new(module.path().as_str()),
    )
    .extract();
    assert_eq!(projections.len(), 1);
    assert_eq!(projections[0].primary_spelling(), "Task262Mode [ x , y ]");
    assert_eq!(
        (
            symbols.symbols().len(),
            symbols.definitions().len(),
            symbols.contributions().len(),
        ),
        (1, 1, 1)
    );

    let output = source_mode_definition_output(
        &ast,
        module.clone(),
        &shells,
        &symbols,
        SOURCE_MODE_DEFINITION_TEXT,
    )
    .expect("Task262 exact selector")
    .unwrap_or_else(|error| panic!("Task262 exact route failed: {error}"));
    let context = output.typed_ast.source_context().expect("Task248 context");
    let source_type = output.typed_ast.source_type().expect("Task249+249M type");
    let mode = output
        .typed_ast
        .source_mode_definition()
        .expect("Task262 handoff");
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
            source_type.mode_rhs().len(),
        ),
        (2, 3, 0, 0, 1)
    );
    assert_eq!(
        (
            mode.definitions().len(),
            mode.parameters().len(),
            mode.applications().len(),
            mode.expansions().len(),
            mode.inhabitation_requests().len(),
            mode.properties().len(),
        ),
        (1, 2, 1, 1, 1, 1)
    );
    assert_eq!(mode.source_id(), ast.source_id);
    assert_eq!(mode.module_id(), &module);
    assert_eq!(mode.source_context_fingerprint(), context.debug_text());
    assert_eq!(mode.source_type_fingerprint(), source_type.debug_text());
    assert_eq!(mode.base_initial_obligation_count(), 0);

    let definitions = mode.definitions().iter().collect::<Vec<_>>();
    let [(definition_id, definition)] = definitions.as_slice() else {
        panic!("Task262 definition row changed");
    };
    let resolver_definition = symbols
        .definitions()
        .iter()
        .next()
        .expect("resolver definition");
    assert_eq!(definition_id.index(), 0);
    assert_eq!(definition.id().index(), 0);
    assert_eq!(definition.symbol(), resolver_definition.symbol());
    assert_eq!(definition.definition(), resolver_definition.id());
    assert_eq!(
        definition.contribution(),
        resolver_definition.contribution()
    );
    assert_eq!(definition.site(), &TypedSiteRef::Node(TypedNodeId::new(49)));
    assert_eq!(
        definition.source_range(),
        task262_range(ast.source_id, 45, 135)
    );
    assert_eq!(definition.source_ordinal(), 0);
    assert_eq!(definition.context().index(), 1);
    assert_eq!(
        definition.recovery(),
        mizar_checker::source_mode_definition::SourceModeDefinitionRecovery::Normal
    );
    assert_eq!(
        definition.spelling(),
        "mode Task262ModeDefinition: Task262Mode [x, y] is set;\n  sethood by computation(steps: 1);"
    );
    assert_eq!(definition.application().index(), 0);
    assert_eq!(definition.expansion().index(), 0);
    assert_eq!(definition.inhabitation_request().index(), 0);
    assert_eq!(definition.property().map(|id| id.index()), Some(0));
    assert_eq!(definition.origin().source_id(), ast.source_id);
    assert_eq!(definition.origin().module_id(), &module);
    assert_eq!(
        definition.origin().anchor(),
        &SourceAnchor::Range(task262_range(ast.source_id, 45, 135))
    );
    assert_eq!(definition.origin().structural_path(), [4, 0, 10, 0]);
    assert!(definition.origin().import_edge().is_none());
    assert!(!definition.origin().is_recovered());

    let parameters = mode.parameters().iter().collect::<Vec<_>>();
    assert_eq!(parameters.len(), 2);
    for (index, ((id, row), (site, start, end, declaration_start, pattern_start, spelling))) in
        parameters
            .into_iter()
            .zip([
                (37, 13, 26, 17, 86, "let x be set;"),
                (41, 29, 42, 33, 89, "let y be set;"),
            ])
            .enumerate()
    {
        assert_eq!(id.index(), index);
        assert_eq!(row.id().index(), index);
        assert_eq!(row.owner().index(), 0);
        assert_eq!(row.ordinal(), index);
        assert_eq!(row.binding().index(), index);
        assert_eq!(row.written_type().index(), index);
        assert_eq!(row.site(), &TypedSiteRef::Node(TypedNodeId::new(site)));
        assert_eq!(row.source_range(), task262_range(ast.source_id, start, end));
        assert_eq!(
            row.declaration_range(),
            task262_range(ast.source_id, declaration_start, declaration_start + 1)
        );
        assert_eq!(
            row.pattern_range(),
            task262_range(ast.source_id, pattern_start, pattern_start + 1)
        );
        assert_eq!(row.context().index(), 1);
        assert_eq!(
            row.recovery(),
            mizar_checker::source_mode_definition::SourceModeDefinitionRecovery::Normal
        );
        assert_eq!(row.spelling(), spelling);
    }

    let applications = mode.applications().iter().collect::<Vec<_>>();
    let [(application_id, application)] = applications.as_slice() else {
        panic!("Task262 application row changed");
    };
    assert_eq!(application_id.index(), 0);
    assert_eq!(
        (
            application.id().index(),
            application.owner().index(),
            application.ordinal()
        ),
        (0, 0, 0)
    );
    assert_eq!(
        application
            .parameters()
            .iter()
            .map(|id| id.index())
            .collect::<Vec<_>>(),
        [0, 1]
    );
    assert_eq!(
        application.site(),
        &TypedSiteRef::Node(TypedNodeId::new(42))
    );
    assert_eq!(
        application.source_range(),
        task262_range(ast.source_id, 73, 91)
    );
    assert_eq!(application.context().index(), 1);
    assert_eq!(
        application.recovery(),
        mizar_checker::source_mode_definition::SourceModeDefinitionRecovery::Normal
    );
    assert_eq!(application.spelling(), "Task262Mode [ x , y ]");

    let expansions = mode.expansions().iter().collect::<Vec<_>>();
    let [(expansion_id, expansion)] = expansions.as_slice() else {
        panic!("Task262 expansion row changed");
    };
    assert_eq!(expansion_id.index(), 0);
    assert_eq!(
        (
            expansion.id().index(),
            expansion.owner().index(),
            expansion.ordinal(),
            expansion.rhs().index()
        ),
        (0, 0, 0, 0)
    );
    assert_eq!(expansion.site(), &TypedSiteRef::Node(TypedNodeId::new(44)));
    assert_eq!(
        expansion.source_range(),
        task262_range(ast.source_id, 95, 98)
    );
    assert_eq!(expansion.context().index(), 1);
    assert_eq!(
        expansion.recovery(),
        mizar_checker::source_mode_definition::SourceModeDefinitionRecovery::Normal
    );
    assert_eq!(expansion.spelling(), "set");

    let requests = mode.inhabitation_requests().iter().collect::<Vec<_>>();
    let [(request_id, request)] = requests.as_slice() else {
        panic!("Task262 request row changed");
    };
    assert_eq!(request_id.index(), 0);
    assert_eq!(
        (
            request.id().index(),
            request.owner().index(),
            request.ordinal(),
            request.expansion().index()
        ),
        (0, 0, 0, 0)
    );
    assert_eq!(
        request.kind(),
        mizar_checker::source_mode_definition::SourceModeInhabitationRequestKind::Rhs
    );
    assert_eq!(request.site(), &TypedSiteRef::Node(TypedNodeId::new(44)));
    assert_eq!(request.source_range(), task262_range(ast.source_id, 95, 98));
    assert_eq!(request.context().index(), 1);
    assert_eq!(
        request.recovery(),
        mizar_checker::source_mode_definition::SourceModeDefinitionRecovery::Normal
    );
    assert_eq!(request.spelling(), "set");

    let properties = mode.properties().iter().collect::<Vec<_>>();
    let [(property_id, property)] = properties.as_slice() else {
        panic!("Task262 property row changed");
    };
    assert_eq!(property_id.index(), 0);
    assert_eq!(
        (
            property.id().index(),
            property.owner().index(),
            property.ordinal()
        ),
        (0, 0, 0)
    );
    assert_eq!(
        property.kind(),
        mizar_checker::source_mode_definition::SourceModePropertyKind::Sethood
    );
    assert_eq!(property.site(), &TypedSiteRef::Node(TypedNodeId::new(48)));
    assert_eq!(
        property.source_range(),
        task262_range(ast.source_id, 102, 135)
    );
    assert_eq!(
        property.justification(),
        &SourceAnchor::Range(task262_range(ast.source_id, 113, 134))
    );
    assert_eq!(
        property.recovery(),
        mizar_checker::source_mode_definition::SourceModeDefinitionRecovery::Normal
    );
    assert_eq!(property.spelling(), "sethood by computation(steps: 1);");
    assert_eq!(property.obligation().index(), 0);

    let obligations = output
        .typed_ast
        .initial_obligations()
        .iter()
        .collect::<Vec<_>>();
    let [(obligation_id, obligation)] = obligations.as_slice() else {
        panic!("Task262 obligation row changed");
    };
    assert_eq!((obligation_id.index(), obligation.id.index()), (0, 0));
    assert_eq!(
        obligation.kind,
        mizar_checker::typed_ast::InitialObligationKind::Sethood
    );
    assert_eq!(&obligation.owner, &TypedSiteRef::Node(TypedNodeId::new(48)));
    assert_eq!(
        obligation.source_range,
        task262_range(ast.source_id, 102, 135)
    );
    assert!(obligation.assumptions.is_empty());
    assert_eq!(
        obligation.goal.as_str(),
        "source.definition.mode.correctness:definition=0:sethood"
    );
    assert_eq!(
        obligation.provenance.as_str(),
        "source.definition.mode:definition=0:property=0"
    );
    assert_eq!(
        obligation.status,
        mizar_checker::typed_ast::InitialObligationStatus::Pending
    );

    let debug = mode.debug_text();
    assert_eq!(debug.matches("source-mode-definition-debug-v1").count(), 1);
    for row in [
        "definition#0 ",
        "parameter#0 ",
        "parameter#1 ",
        "application#0 ",
        "expansion#0 ",
        "inhabitation-request#0 ",
        "property#0 ",
    ] {
        assert!(
            debug.lines().any(|line| line.starts_with(row)),
            "missing {row}"
        );
    }
    assert!(debug.ends_with('\n'));
    assert_eq!(
        output.typed_ast.source_mode_definition(),
        output.resolved.source_mode_definition()
    );
    assert_eq!(
        output
            .typed_ast
            .debug_text()
            .matches("source-mode-definition-debug-v1")
            .count(),
        1
    );
    assert_eq!(
        output
            .resolved
            .debug_text()
            .matches("source-mode-definition-debug-v1")
            .count(),
        1
    );
}

#[derive(Debug, Clone, Copy)]
enum Task262SurfaceMutation {
    StructuralKind,
    StructuralRange,
    StructuralChildren,
    RootIdentity,
    RootChildOrder,
    DefinitionSiblingOrder,
    ModeChildRelocation,
    PatternKind,
    LabelToken,
    ModeNameToken,
    PatternParameterToken,
    RhsToken,
    PropertyToken,
    JustificationToken,
    TokenRecovery,
    ExpressionRoot,
}

fn task262_mutated_surface_ast(ast: &SurfaceAst, mutation: Task262SurfaceMutation) -> SurfaceAst {
    let mut builder = SurfaceAstBuilder::new(ast.source_id);
    let mut rebuilt = Vec::<SurfaceBuilderNodeId>::with_capacity(ast.nodes().len());
    for (index, node) in ast.nodes().iter().enumerate() {
        let kind = match (index, mutation) {
            (49, Task262SurfaceMutation::StructuralKind) => SurfaceNodeKind::FunctorDefinition,
            (42, Task262SurfaceMutation::PatternKind) => SurfaceNodeKind::FunctorPattern,
            _ => node.kind.clone(),
        };
        let range = match (index, mutation) {
            (49, Task262SurfaceMutation::StructuralRange) => task262_range(ast.source_id, 45, 134),
            _ => node.range,
        };
        let mut children = node
            .children
            .iter()
            .map(|child| rebuilt[child.index()])
            .collect::<Vec<_>>();
        match (index, mutation) {
            (49, Task262SurfaceMutation::StructuralChildren) => children.swap(3, 5),
            (53, Task262SurfaceMutation::RootIdentity) => children.clear(),
            (53, Task262SurfaceMutation::RootChildOrder) => children.swap(0, 1),
            (50, Task262SurfaceMutation::DefinitionSiblingOrder) => children.swap(1, 2),
            (48, Task262SurfaceMutation::ModeChildRelocation) => children.push(rebuilt[44]),
            (49, Task262SurfaceMutation::ModeChildRelocation) => {
                children.remove(5);
            }
            _ => {}
        }
        let rebuilt_id = match kind {
            SurfaceNodeKind::Token(token) => {
                if index == 14 && matches!(mutation, Task262SurfaceMutation::TokenRecovery) {
                    builder.add_recovered_token(token.kind, token.text, range)
                } else {
                    let text = match (index, mutation) {
                        (12, Task262SurfaceMutation::LabelToken) => "Task262CorruptDefinition",
                        (14, Task262SurfaceMutation::ModeNameToken) => "Task262CorruptMode",
                        (16, Task262SurfaceMutation::PatternParameterToken) => "y",
                        (21, Task262SurfaceMutation::RhsToken) => "object",
                        (23, Task262SurfaceMutation::PropertyToken) => "existence",
                        (25, Task262SurfaceMutation::JustificationToken) => "proof",
                        _ => token.text.as_ref(),
                    };
                    builder.add_token(token.kind, text, range)
                }
            }
            structural => builder.add_node(structural, range, children),
        };
        rebuilt.push(rebuilt_id);
    }
    let expression_root = if matches!(mutation, Task262SurfaceMutation::ExpressionRoot) {
        Some(rebuilt[44])
    } else {
        ast.expression_root().map(|id| rebuilt[id.index()])
    };
    let root = if matches!(mutation, Task262SurfaceMutation::RootIdentity) {
        Some(rebuilt[52])
    } else {
        ast.root().map(|id| rebuilt[id.index()])
    };
    builder.finish(root, expression_root)
}

#[test]
fn task262_mode_definition_surface_resolver_lower_and_payload_corruption_fail_closed() {
    let (ast, module, shells, symbols) =
        task253_ast_from_source_text(SOURCE_MODE_DEFINITION_TEXT, 262_010);
    for mutation in [
        Task262SurfaceMutation::StructuralKind,
        Task262SurfaceMutation::StructuralRange,
        Task262SurfaceMutation::StructuralChildren,
        Task262SurfaceMutation::RootIdentity,
        Task262SurfaceMutation::RootChildOrder,
        Task262SurfaceMutation::DefinitionSiblingOrder,
        Task262SurfaceMutation::ModeChildRelocation,
        Task262SurfaceMutation::PatternKind,
        Task262SurfaceMutation::LabelToken,
        Task262SurfaceMutation::ModeNameToken,
        Task262SurfaceMutation::PatternParameterToken,
        Task262SurfaceMutation::RhsToken,
        Task262SurfaceMutation::PropertyToken,
        Task262SurfaceMutation::JustificationToken,
        Task262SurfaceMutation::TokenRecovery,
        Task262SurfaceMutation::ExpressionRoot,
    ] {
        assert!(
            source_mode_definition_output(
                &task262_mutated_surface_ast(&ast, mutation),
                module.clone(),
                &shells,
                &symbols,
                SOURCE_MODE_DEFINITION_TEXT,
            )
            .is_none(),
            "surface mutation {mutation:?} entered Task262"
        );
    }
    for near_source in [
        SOURCE_MODE_DEFINITION_TEXT
            .trim_end_matches('\n')
            .to_owned(),
        format!("{SOURCE_MODE_DEFINITION_TEXT}\n"),
        SOURCE_MODE_DEFINITION_TEXT.replacen("let x", "let z", 1),
        SOURCE_MODE_DEFINITION_TEXT.replacen(
            "Task262ModeDefinition",
            "Task262ChangedDefinition",
            1,
        ),
        SOURCE_MODE_DEFINITION_TEXT.replacen("Task262Mode", "Task262ChangedMode", 1),
        SOURCE_MODE_DEFINITION_TEXT.replacen("is set", "is object", 1),
        SOURCE_MODE_DEFINITION_TEXT.replacen("sethood", "existence", 1),
    ] {
        let (near_ast, near_module, near_shells, near_symbols) =
            task253_ast_from_source_text(&near_source, 262_011);
        assert!(
            source_mode_definition_output(
                &near_ast,
                near_module,
                &near_shells,
                &near_symbols,
                &near_source
            )
            .is_none()
        );
    }

    for (mutation, owner) in [
        (
            SourceModeDefinitionRouteMutation::RemoveResolverShell,
            "Task262 resolver:",
        ),
        (
            SourceModeDefinitionRouteMutation::WrongResolverProjection,
            "Task262 resolver:",
        ),
        (
            SourceModeDefinitionRouteMutation::WrongResolverEntry,
            "Task262 resolver:",
        ),
        (
            SourceModeDefinitionRouteMutation::WrongResolverDefinitionEntry,
            "Task262 resolver:",
        ),
        (
            SourceModeDefinitionRouteMutation::WrongResolverContribution,
            "Task262 resolver:",
        ),
        (
            SourceModeDefinitionRouteMutation::WrongContextModuleSite,
            "Task248 source context:",
        ),
        (
            SourceModeDefinitionRouteMutation::WrongContextItemSite,
            "Task248 source context:",
        ),
        (
            SourceModeDefinitionRouteMutation::WrongContextBindingSite(0),
            "Task248 source context:",
        ),
        (
            SourceModeDefinitionRouteMutation::WrongContextBindingOwner(1),
            "Task248 source context:",
        ),
        (
            SourceModeDefinitionRouteMutation::RemoveTypeExpression,
            "Task249 source type:",
        ),
        (
            SourceModeDefinitionRouteMutation::WrongTypeApplicationBinding(0),
            "Task249 source type:",
        ),
        (
            SourceModeDefinitionRouteMutation::WrongTypeApplicationRoot(1),
            "Task249 source type:",
        ),
        (
            SourceModeDefinitionRouteMutation::WrongTypeExpressionSite(0),
            "Task249 source type:",
        ),
        (
            SourceModeDefinitionRouteMutation::WrongModeRhsOwner,
            "Task249M mode RHS:",
        ),
        (
            SourceModeDefinitionRouteMutation::WrongModeRhsRange,
            "Task249M mode RHS:",
        ),
        (
            SourceModeDefinitionRouteMutation::WrongModeRhsExpression,
            "Task249M mode RHS:",
        ),
        (
            SourceModeDefinitionRouteMutation::RemoveModeDefinition,
            "Task262 mode definition:",
        ),
        (
            SourceModeDefinitionRouteMutation::RemoveModeParameter,
            "Task262 mode definition:",
        ),
        (
            SourceModeDefinitionRouteMutation::RemoveModeApplication,
            "Task262 mode definition:",
        ),
        (
            SourceModeDefinitionRouteMutation::RemoveModeExpansion,
            "Task262 mode definition:",
        ),
        (
            SourceModeDefinitionRouteMutation::RemoveModeRequest,
            "Task262 mode definition:",
        ),
        (
            SourceModeDefinitionRouteMutation::RemoveModeProperty,
            "Task262 mode definition:",
        ),
        (
            SourceModeDefinitionRouteMutation::WrongModeParameterOwner,
            "Task262 mode definition:",
        ),
        (
            SourceModeDefinitionRouteMutation::WrongModeParameterPatternRange,
            "Task262 mode definition:",
        ),
        (
            SourceModeDefinitionRouteMutation::WrongModeApplicationParameters,
            "Task262 mode definition:",
        ),
        (
            SourceModeDefinitionRouteMutation::WrongModeExpansionRhs,
            "Task262 mode definition:",
        ),
        (
            SourceModeDefinitionRouteMutation::WrongModeRequestExpansion,
            "Task262 mode definition:",
        ),
        (
            SourceModeDefinitionRouteMutation::WrongModeDefinitionProperty,
            "Task262 mode definition:",
        ),
        (
            SourceModeDefinitionRouteMutation::WrongModePropertyJustification,
            "Task262 mode definition:",
        ),
    ] {
        let error = source_mode_definition_output_with_mutation(
            &ast,
            module.clone(),
            &shells,
            &symbols,
            SOURCE_MODE_DEFINITION_TEXT,
            mutation,
        )
        .unwrap_or_else(|| panic!("mutation {mutation:?} escaped selector"))
        .expect_err("corruption must fail closed");
        assert!(error.starts_with(owner), "{mutation:?}: {error}");
    }

    let first = source_mode_definition_output(
        &ast,
        module.clone(),
        &shells,
        &symbols,
        SOURCE_MODE_DEFINITION_TEXT,
    )
    .expect("first selector")
    .expect("first route");
    let second =
        source_mode_definition_output(&ast, module, &shells, &symbols, SOURCE_MODE_DEFINITION_TEXT)
            .expect("second selector")
            .expect("second route");
    assert_eq!(first.typed_ast.debug_text(), second.typed_ast.debug_text());
    assert_eq!(first.resolved.debug_text(), second.resolved.debug_text());
}

#[test]
fn task262_mode_definition_selection_and_family_isolation_are_exact() {
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
    let plan = build_test_plan(&config).expect("Task262 repository plan should build");
    let selected = active_type_elaboration_cases(&plan)
        .filter(|case| {
            std::fs::read_to_string(&case.source_path)
                .is_ok_and(|source| source == SOURCE_MODE_DEFINITION_TEXT)
        })
        .collect::<Vec<_>>();
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].id.0, TASK262_CASE);
    assert!(selected[0].expectation_path.ends_with(Path::new(
        "tests/miz/pass/types/pass_type_elaboration_mode_definition_payload_001.expect.toml"
    )));

    let (ordinal, case) = active_type_elaboration_cases(&plan)
        .enumerate()
        .find(|(_, case)| case.id.0 == TASK262_CASE)
        .expect("Task262 active sidecar");
    assert_eq!(case.expectation.schema_version, 1);
    assert_eq!(case.expectation.kind, crate::expectation::TestKind::Pass);
    assert_eq!(
        case.expectation.stage,
        crate::staged_model::Stage::TypeElaboration
    );
    assert_eq!(case.expectation.domain, "checker.type_elaboration");
    assert_eq!(
        case.expectation.source,
        Path::new("pass_type_elaboration_mode_definition_payload_001.miz")
    );
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
        [TASK262_SPEC_REF]
    );
    assert_eq!(case.expectation.tags, ["active_type_elaboration"]);
    assert_eq!(case.expectation.profiles, ["fast"]);
    assert!(case.expectation.diagnostic_codes.is_empty());
    assert!(case.expectation.diagnostic_payloads.is_empty());
    assert!(case.expectation.failure_category.is_none());
    assert!(case.expectation.rejection_reason.is_none());
    assert!(case.expectation.stable_detail_key.is_none());
    assert!(case.expectation.snapshots.is_none());
    assert!(case.expectation.tokens.is_empty());

    let requirement = plan
        .manifest
        .requirements
        .iter()
        .find(|row| row.id.0 == TASK262_SPEC_REF)
        .expect("Task262 trace row");
    assert_eq!(
        requirement.source,
        Path::new("doc/design/mizar-checker/en/source_mode_definition.md")
    );
    assert_eq!(
        requirement.section,
        "Dedicated Runner Consumer And Trace Intent"
    );
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
    assert!(!requirement.built_in);
    assert!(requirement.depends_on.is_empty());
    assert!(requirement.deferred_reason.is_none());
    assert_eq!(
        requirement.tests,
        [Path::new(
            "tests/miz/pass/types/pass_type_elaboration_mode_definition_payload_001.expect.toml"
        )]
    );
    assert_eq!(
        plan.cases
            .iter()
            .filter(|candidate| candidate
                .expectation
                .spec_refs
                .iter()
                .any(|reference| reference.0 == TASK262_SPEC_REF))
            .map(|candidate| candidate.id.0.as_str())
            .collect::<Vec<_>>(),
        [TASK262_CASE]
    );
    let result = run_type_elaboration_case(
        &workspace_root,
        &workspace_root.join("tests"),
        case,
        ordinal,
    );
    assert_eq!(result.status, TypeElaborationCaseStatus::Passed);
    assert!(result.actual_detail_keys.is_empty());

    for (source, source_ordinal) in [
        (
            super::type_elaboration::SOURCE_PREDICATE_DEFINITION_TEXT,
            262_090,
        ),
        (
            super::type_elaboration::SOURCE_FUNCTOR_DEFINITION_TEXT,
            262_091,
        ),
        (
            super::type_elaboration::SOURCE_ATTRIBUTE_DEFINITION_TEXT,
            262_092,
        ),
    ] {
        let (ast, module, shells, symbols) = task253_ast_from_source_text(source, source_ordinal);
        assert!(source_mode_definition_output(&ast, module, &shells, &symbols, source).is_none());
    }
    for (case_id, source_ordinal) in [
        (TASK262_HISTORICAL_GAP_CASE, 262_093),
        (TASK262_MIXED_CASE, 262_094),
    ] {
        let (case_ordinal, boundary) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| case.id.0 == case_id)
            .expect("Task262 boundary remains active");
        let source = std::fs::read_to_string(&boundary.source_path).expect("boundary source");
        let (ast, module, shells, symbols) = task253_ast_from_source_text(&source, source_ordinal);
        assert!(source_mode_definition_output(&ast, module, &shells, &symbols, &source).is_none());
        let old_result = run_type_elaboration_case(
            &workspace_root,
            &workspace_root.join("tests"),
            boundary,
            case_ordinal,
        );
        assert_eq!(old_result.status, TypeElaborationCaseStatus::Passed);
        assert_eq!(
            old_result.actual_detail_keys,
            boundary.expectation.diagnostic_payloads
        );
    }
}

#[test]
fn task262_mode_definition_justification_and_semantic_subtrees_are_not_published() {
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
    .expect("Task262 count plan");
    let has_active_type_tag = |case: &&crate::harness::TestCase| {
        case.expectation
            .tags
            .iter()
            .any(|tag| tag == "active_type_elaboration")
    };
    assert_eq!([
        active_type_elaboration_cases(&plan).count(),
        plan.cases.iter().filter(has_active_type_tag).count(),
        plan.cases.iter().filter(has_active_type_tag).filter(|case| case.expectation.stage == crate::staged_model::Stage::TypeElaboration).count(),
        plan.cases.iter().filter(has_active_type_tag).filter(|case| case.expectation.expected_phase == Some(crate::expectation::PipelinePhase::TypeCheck)).count(),
        plan.cases.iter().filter(has_active_type_tag).filter(|case| matches!(case.expectation.expected_outcome, crate::expectation::ExpectedOutcome::Pass | crate::expectation::ExpectedOutcome::Fail)).count(),
        plan.cases.iter().filter(has_active_type_tag).filter(|case| case.source_path.extension().is_some_and(|ext| ext == "miz")).count(),
    ], [205; 6]);
    assert_eq!(
        (plan.cases.len(), plan.manifest.requirements.len()),
        (551, 499)
    );
    assert_eq!(
        plan.cases.iter().fold((0, 0), |(pass, fail), case| {
            match case.expectation.expected_outcome {
                crate::expectation::ExpectedOutcome::Pass => (pass + 1, fail),
                crate::expectation::ExpectedOutcome::Fail => (pass, fail + 1),
                _ => (pass, fail),
            }
        }),
        (308, 243)
    );
    assert_eq!(
        (
            crate::active_parse_only_cases(&plan).count(),
            crate::active_declaration_symbol_cases(&plan).count(),
            active_type_elaboration_cases(&plan).count(),
            crate::active_proof_verification_cases(&plan).count(),
        ),
        (102, 7, 205, 1)
    );
    let type_stage = plan
        .coverage_report
        .stages
        .iter()
        .find(|stage| stage.stage == crate::staged_model::Stage::TypeElaboration)
        .expect("type stage");
    assert_eq!((type_stage.requirements, type_stage.covered), (307, 295));
    assert_eq!(
        (plan.warning_count(), plan.error_count()),
        (23, 0),
        "{:#?}",
        plan.diagnostics
    );

    let (ast, module, shells, symbols) =
        task253_ast_from_source_text(SOURCE_MODE_DEFINITION_TEXT, 262_100);
    let symbols_before = symbols.clone();
    let first = source_mode_definition_output(
        &ast,
        module.clone(),
        &shells,
        &symbols,
        SOURCE_MODE_DEFINITION_TEXT,
    )
    .expect("selector")
    .expect("route");
    let second =
        source_mode_definition_output(&ast, module, &shells, &symbols, SOURCE_MODE_DEFINITION_TEXT)
            .expect("replay selector")
            .expect("replay route");
    assert_eq!(
        symbols, symbols_before,
        "Task262 must not activate resolver data"
    );
    let obligations = first
        .typed_ast
        .initial_obligations()
        .iter()
        .collect::<Vec<_>>();
    let [(obligation_id, obligation)] = obligations.as_slice() else {
        panic!("Task262 must publish exactly one pending obligation");
    };
    assert_eq!((obligation_id.index(), obligation.id.index()), (0, 0));
    assert_eq!(
        obligation.kind,
        mizar_checker::typed_ast::InitialObligationKind::Sethood
    );
    assert_eq!(&obligation.owner, &TypedSiteRef::Node(TypedNodeId::new(48)));
    assert_eq!(
        obligation.source_range,
        task262_range(ast.source_id, 102, 135)
    );
    assert!(obligation.assumptions.is_empty());
    assert_eq!(
        obligation.goal.as_str(),
        "source.definition.mode.correctness:definition=0:sethood"
    );
    assert_eq!(
        obligation.provenance.as_str(),
        "source.definition.mode:definition=0:property=0"
    );
    assert_eq!(
        obligation.status,
        mizar_checker::typed_ast::InitialObligationStatus::Pending
    );

    for index in 45..=48 {
        let surface = &ast.nodes()[index];
        let typed = first
            .typed_ast
            .nodes()
            .node(TypedNodeId::new(index))
            .expect("preserved typed subtree");
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
        assert_eq!(typed.anchor, SourceAnchor::Range(surface.range));
        assert_eq!(typed.typing, mizar_checker::typed_ast::TypingState::Unknown);
        assert_eq!(
            typed.recovery,
            mizar_checker::typed_ast::NodeRecoveryState::Normal
        );
        assert!(typed.links.facts.is_empty());
        assert!(typed.links.coercions.is_empty());
        assert!(typed.links.initial_obligations.is_empty());
        assert!(typed.links.diagnostics.is_empty());
        let (_, resolved) = first
            .resolved
            .nodes()
            .iter()
            .find(|(_, row)| row.typed_node.index() == index)
            .expect("preserved resolved subtree");
        assert!(matches!(
            &resolved.kind,
            mizar_checker::resolved_typed_ast::ResolvedTypedNodeKind::Degraded {
                reason: mizar_checker::resolved_typed_ast::ResolvedNodeRecoveryReason::TypingState(
                    mizar_checker::typed_ast::TypingState::Unknown
                )
            }
        ));
        assert!(resolved.final_type.is_none());
        assert!(resolved.metadata.is_none());
        assert!(resolved.diagnostics.is_empty());
    }

    assert!(first.typed_ast.types().is_empty());
    assert!(first.typed_ast.facts().is_empty());
    assert!(first.typed_ast.coercions().is_empty());
    assert!(first.typed_ast.diagnostics().is_empty());
    assert!(first.typed_ast.source_term().is_none());
    assert!(first.typed_ast.source_attribute().is_none());
    assert!(first.typed_ast.source_evidence().is_none());
    assert!(first.typed_ast.source_predicate_definition().is_none());
    assert!(first.typed_ast.source_functor_definition().is_none());
    assert!(first.typed_ast.source_attribute_definition().is_none());
    assert!(first.typed_ast.source_application().is_none());
    assert!(first.typed_ast.source_structure().is_none());
    assert!(first.typed_ast.source_set_term().is_none());
    assert!(first.typed_ast.source_atomic_formula().is_none());
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
    assert!(first.resolved.source_attribute_definition().is_none());
    assert_eq!(
        first.typed_ast.source_mode_definition(),
        first.resolved.source_mode_definition()
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
        "discharged",
        "proof",
        "witness",
        "verification-condition",
        "vc:",
        "semantic-fact",
    ] {
        assert!(
            !debug.contains(forbidden),
            "Task262 debug leaked {forbidden}"
        );
    }
}

#[test]
fn task262_core_item_context_association_is_exact_and_deterministic() {
    let (ast, module, shells, symbols) =
        task253_ast_from_source_text(SOURCE_MODE_DEFINITION_TEXT, 262_200);
    let output =
        source_mode_definition_output(&ast, module, &shells, &symbols, SOURCE_MODE_DEFINITION_TEXT)
            .expect("Task262 selector")
            .expect("Task262 route");
    let source_context = output
        .typed_ast
        .source_context()
        .expect("Task248 source context")
        .clone();
    let checker_owner = output
        .typed_ast
        .source_mode_definition()
        .expect("Task262 checker owner")
        .clone();
    let source_bindings = task262_source_binding_core_handoff(&source_context, &checker_owner);
    let expected_source_bindings = source_bindings.clone();
    let first = mizar_core::elaborator::SourceModeCoreContextProducer::build(
        source_bindings.clone(),
        source_context.clone(),
        checker_owner.clone(),
    )
    .expect("Task262 Core item context");
    let second = mizar_core::elaborator::SourceModeCoreContextProducer::build(
        source_bindings,
        source_context.clone(),
        checker_owner.clone(),
    )
    .expect("Task262 deterministic replay");
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
        .get(mizar_checker::source_mode_definition::SourceModeDefinitionId::new(0))
        .expect("Task262 definition");
    let source_item = source_context
        .context_links()
        .get(definition.context())
        .expect("Task248 definition link")
        .item
        .expect("Task248 containing source item");
    let association = first
        .items()
        .get(definition.id())
        .expect("Task262 association");
    assert_eq!(association.definition(), definition.id());
    assert_eq!(association.source_item(), source_item);
    assert_eq!(association.symbol(), definition.symbol());
    let core_item = first
        .context()
        .item_registry()
        .id_for_symbol(definition.symbol())
        .expect("Core mode item");
    assert_eq!(association.core_item(), core_item);
    assert_eq!(
        first
            .items()
            .iter()
            .map(|(id, row)| (id, row.source_item(), row.symbol().clone(), row.core_item()))
            .collect::<Vec<_>>(),
        vec![(
            definition.id(),
            source_item,
            definition.symbol().clone(),
            core_item
        )]
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
        .expect("Core mode row");
    assert_eq!(item.symbol, *definition.symbol());
    assert_eq!(item.kind, mizar_core::core_ir::CoreItemKind::Mode);
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
            "source-mode-core-item-v1.definition.0",
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
            "source-mode-core-item-v1.definition.0",
        )]
    );
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
fn task262_core_item_context_default_deny_mutations_and_foreign_environment() {
    let (ast, module, shells, symbols) =
        task253_ast_from_source_text(SOURCE_MODE_DEFINITION_TEXT, 262_210);
    let output =
        source_mode_definition_output(&ast, module, &shells, &symbols, SOURCE_MODE_DEFINITION_TEXT)
            .expect("Task262 selector")
            .expect("Task262 route");
    let source_context = output
        .typed_ast
        .source_context()
        .expect("Task248 source context")
        .clone();
    let checker_owner = output
        .typed_ast
        .source_mode_definition()
        .expect("Task262 checker owner")
        .clone();
    for mutation in [
        Task262CoreContextMutation::MissingItem,
        Task262CoreContextMutation::ExtraItem,
        Task262CoreContextMutation::WrongKind,
        Task262CoreContextMutation::WrongVisibility,
        Task262CoreContextMutation::WrongSource,
        Task262CoreContextMutation::WrongProvenance,
        Task262CoreContextMutation::MissingBoundary,
        Task262CoreContextMutation::WrongBoundary,
        Task262CoreContextMutation::UnexpectedDependency,
        Task262CoreContextMutation::InvalidStatus,
    ] {
        let source_bindings = task262_source_binding_core_handoff_with_mutation(
            &source_context,
            &checker_owner,
            mutation,
        );
        let error = mizar_core::elaborator::SourceModeCoreContextProducer::build(
            source_bindings,
            source_context.clone(),
            checker_owner.clone(),
        )
        .expect_err("Core mutation must fail closed");
        assert_eq!(
            error,
            mizar_core::elaborator::SourceModeCoreContextError::InvalidCoreContext,
            "{mutation:?}"
        );
    }

    let (foreign_ast, foreign_module, foreign_shells, foreign_symbols) =
        task253_ast_from_source_text(SOURCE_MODE_DEFINITION_TEXT, 262_211);
    let foreign_output = source_mode_definition_output(
        &foreign_ast,
        foreign_module,
        &foreign_shells,
        &foreign_symbols,
        SOURCE_MODE_DEFINITION_TEXT,
    )
    .expect("foreign Task262 selector")
    .expect("foreign Task262 route");
    let foreign_context = foreign_output
        .typed_ast
        .source_context()
        .expect("foreign source context")
        .clone();
    let foreign_owner = foreign_output
        .typed_ast
        .source_mode_definition()
        .expect("foreign checker owner")
        .clone();
    let base_bindings = task262_source_binding_core_handoff(&source_context, &checker_owner);
    let foreign_bindings = task262_source_binding_core_handoff(&foreign_context, &foreign_owner);
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
        let error =
            mizar_core::elaborator::SourceModeCoreContextProducer::build(bindings, context, owner)
                .expect_err("foreign environment must fail closed");
        assert_eq!(
            error,
            mizar_core::elaborator::SourceModeCoreContextError::EnvironmentMismatch
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Task262CoreContextMutation {
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

fn task262_source_binding_core_handoff(
    source_context: &mizar_checker::source_context::SourceBindingContextHandoff,
    checker_owner: &mizar_checker::source_mode_definition::SourceModeDefinitionHandoff,
) -> mizar_core::elaborator::SourceBindingCoreContextHandoff {
    task262_source_binding_core_handoff_with_mutation(
        source_context,
        checker_owner,
        Task262CoreContextMutation::Baseline,
    )
}

fn task262_source_binding_core_handoff_with_mutation(
    source_context: &mizar_checker::source_context::SourceBindingContextHandoff,
    checker_owner: &mizar_checker::source_mode_definition::SourceModeDefinitionHandoff,
    mutation: Task262CoreContextMutation,
) -> mizar_core::elaborator::SourceBindingCoreContextHandoff {
    let definition = checker_owner
        .definitions()
        .get(mizar_checker::source_mode_definition::SourceModeDefinitionId::new(0))
        .expect("Task262 definition");
    let source_range = if mutation == Task262CoreContextMutation::WrongSource {
        mizar_session::SourceRange {
            source_id: source_context.source_id(),
            start: definition.source_range().start + 1,
            end: definition.source_range().end,
        }
    } else {
        definition.source_range()
    };
    let provenance_key = if mutation == Task262CoreContextMutation::WrongProvenance {
        "wrong-task262-provenance"
    } else {
        "source-mode-core-item-v1.definition.0"
    };
    let source = mizar_core::core_ir::CoreSourceRef::direct(source_range).with_provenance(vec![
        mizar_core::core_ir::CoreProvenance::new(
            mizar_core::core_ir::CoreProvenancePhase::Checker,
            provenance_key,
        ),
    ]);
    let kind = if mutation == Task262CoreContextMutation::WrongKind {
        mizar_core::core_ir::CoreItemKind::Predicate
    } else {
        mizar_core::core_ir::CoreItemKind::Mode
    };
    let visibility = if mutation == Task262CoreContextMutation::WrongVisibility {
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
    let seed = if mutation == Task262CoreContextMutation::MissingBoundary {
        seed
    } else if mutation == Task262CoreContextMutation::WrongBoundary {
        seed.with_definition_boundary(mizar_core::elaborator::DefinitionBoundaryKind::Theorem)
    } else {
        seed.with_definition_boundary(
            mizar_core::elaborator::DefinitionBoundaryKind::DefinitionalItem,
        )
    };
    let seed = if mutation == Task262CoreContextMutation::UnexpectedDependency {
        seed.with_dependencies(vec![definition.symbol().clone()])
    } else if mutation == Task262CoreContextMutation::InvalidStatus {
        seed.with_dependencies(vec![mizar_resolve::resolved_ast::SymbolId::new(
            definition.symbol().module().clone(),
            mizar_resolve::resolved_ast::LocalSymbolId::new("task262-missing"),
            mizar_resolve::resolved_ast::FullyQualifiedName::new(format!(
                "{}.task262-missing",
                definition.symbol().fqn().as_str()
            )),
        )])
    } else {
        seed
    };
    if mutation != Task262CoreContextMutation::MissingItem {
        input.item_seeds.push(seed);
    }
    if mutation == Task262CoreContextMutation::ExtraItem {
        let extra_symbol = mizar_resolve::resolved_ast::SymbolId::new(
            definition.symbol().module().clone(),
            mizar_resolve::resolved_ast::LocalSymbolId::new("task262-extra"),
            mizar_resolve::resolved_ast::FullyQualifiedName::new(format!(
                "{}.task262-extra",
                definition.symbol().fqn().as_str()
            )),
        );
        input.item_seeds.push(
            mizar_core::elaborator::CoreItemSeed::new(
                extra_symbol,
                mizar_core::core_ir::CoreItemKind::Mode,
                "public",
                mizar_core::core_ir::CoreSourceRef::direct(definition.source_range())
                    .with_provenance(vec![mizar_core::core_ir::CoreProvenance::new(
                        mizar_core::core_ir::CoreProvenancePhase::Checker,
                        "source-mode-core-item-v1.definition.extra",
                    )]),
                mizar_core::elaborator::CheckerOwnedProvenance::checker(
                    "source-mode-core-item-v1.definition.extra",
                ),
            )
            .with_definition_boundary(
                mizar_core::elaborator::DefinitionBoundaryKind::DefinitionalItem,
            ),
        );
    }
    let context = mizar_core::elaborator::prepare_core_context(input)
        .expect("Task262 Core context seed should prepare");
    mizar_core::elaborator::SourceBindingCoreContextProducer::build(
        context,
        source_context.binding_env().clone(),
    )
    .expect("Task262 33LB handoff should build")
}
