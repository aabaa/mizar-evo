use mizar_checker::type_checker::FormulaKind;
use mizar_checker::typed_ast::TypedSiteRef;
use mizar_session::SourceRange;
use mizar_syntax::{
    SurfaceAst, SurfaceFormulaConstant, SurfaceNode, SurfaceNodeId, SurfaceNodeKind,
};

use super::source_ast::{
    direct_token_texts, structural_child_ids, subtree_has_recovery, surface_nodes_with_kind,
    surface_site,
};

#[derive(Debug, Clone, Copy)]
pub(in crate::runner) struct SourceBuiltinBinaryTermFormulaConfig {
    pub(in crate::runner) label: &'static str,
    pub(in crate::runner) operator: &'static str,
    pub(in crate::runner) left: &'static str,
    pub(in crate::runner) right: &'static str,
    formula_kind: FormulaKind,
}

pub(in crate::runner) const SOURCE_BUILTIN_BINARY_TERM_FORMULA_CONFIGS:
    &[SourceBuiltinBinaryTermFormulaConfig] = &[
    SourceBuiltinBinaryTermFormulaConfig {
        label: "TermFormulaPayloadBoundary",
        operator: "=",
        left: "1",
        right: "1",
        formula_kind: FormulaKind::Equality,
    },
    SourceBuiltinBinaryTermFormulaConfig {
        label: "BuiltinInequalityPayloadBoundary",
        operator: "<>",
        left: "1",
        right: "2",
        formula_kind: FormulaKind::Inequality,
    },
    SourceBuiltinBinaryTermFormulaConfig {
        label: "BuiltinMembershipPayloadBoundary",
        operator: "in",
        left: "1",
        right: "1",
        formula_kind: FormulaKind::Membership,
    },
];

#[derive(Debug, Clone)]
pub(in crate::runner) struct SourceBuiltinBinaryTermFormula {
    pub(in crate::runner) formula_site: TypedSiteRef,
    pub(in crate::runner) formula_range: SourceRange,
    pub(in crate::runner) formula_kind: FormulaKind,
    pub(in crate::runner) left_site: TypedSiteRef,
    pub(in crate::runner) left_range: SourceRange,
    pub(in crate::runner) right_site: TypedSiteRef,
    pub(in crate::runner) right_range: SourceRange,
}

#[derive(Debug, Clone)]
pub(in crate::runner) struct SourceFormulaStatement {
    pub(in crate::runner) formula_site: TypedSiteRef,
    pub(in crate::runner) formula_range: SourceRange,
}

pub(in crate::runner) fn extract_source_formula_statement(
    ast: &SurfaceAst,
) -> Option<SourceFormulaStatement> {
    extract_exact_source_formula_constant(
        ast,
        "FormulaPayloadBoundary",
        SurfaceFormulaConstant::Thesis,
    )
}

pub(in crate::runner) fn extract_source_contradiction_formula(
    ast: &SurfaceAst,
) -> Option<SourceFormulaStatement> {
    extract_exact_source_formula_constant(
        ast,
        "SourceDerivedContradictionConstantBoundary",
        SurfaceFormulaConstant::Contradiction,
    )
}

pub(in crate::runner) fn extract_source_builtin_binary_term_formula(
    ast: &SurfaceAst,
) -> Option<SourceBuiltinBinaryTermFormula> {
    if ast
        .nodes()
        .iter()
        .any(|node| !is_supported_builtin_binary_theorem_bridge_node(node))
    {
        return None;
    }
    let theorem_items = surface_nodes_with_kind(ast, SurfaceNodeKind::TheoremItem);
    let [(_, theorem)] = theorem_items.as_slice() else {
        return None;
    };
    if subtree_has_recovery(ast, theorem) {
        return None;
    }
    let theorem_tokens = direct_token_texts(ast, theorem);
    let config = SOURCE_BUILTIN_BINARY_TERM_FORMULA_CONFIGS
        .iter()
        .copied()
        .find(|config| theorem_tokens.as_slice() == ["theorem", config.label, ":", ";"])?;

    let theorem_structural_children = structural_child_ids(ast, theorem);
    let formula_expressions = theorem_structural_children
        .iter()
        .copied()
        .filter(|id| {
            ast.node(*id)
                .is_some_and(|node| matches!(node.kind, SurfaceNodeKind::FormulaExpression))
        })
        .collect::<Vec<_>>();
    if formula_expressions.len() != 1
        || theorem_structural_children
            .iter()
            .any(|child| !formula_expressions.contains(child))
    {
        return None;
    }
    let formula_expression = ast.node(formula_expressions[0])?;
    let formula_children = structural_child_ids(ast, formula_expression);
    let [formula_id] = formula_children.as_slice() else {
        return None;
    };
    let formula = ast.node(*formula_id)?;
    let operator_tokens = direct_token_texts(ast, formula);
    if !matches!(formula.kind, SurfaceNodeKind::BuiltinPredicateApplication)
        || subtree_has_recovery(ast, formula)
        || operator_tokens.len() != 1
        || operator_tokens[0] != config.operator
    {
        return None;
    }

    let predicate_structural_children = structural_child_ids(ast, formula);
    let term_expressions = predicate_structural_children
        .iter()
        .copied()
        .filter(|id| {
            ast.node(*id)
                .is_some_and(|node| matches!(node.kind, SurfaceNodeKind::TermExpression))
        })
        .collect::<Vec<_>>();
    if term_expressions.len() != 2
        || predicate_structural_children
            .iter()
            .any(|child| !term_expressions.contains(child))
    {
        return None;
    }

    let left = exact_numeral_term_operand(ast, term_expressions[0], config.left)?;
    let right = exact_numeral_term_operand(ast, term_expressions[1], config.right)?;
    Some(SourceBuiltinBinaryTermFormula {
        formula_site: surface_site(*formula_id),
        formula_range: formula.range,
        formula_kind: config.formula_kind,
        left_site: surface_site(left.0),
        left_range: left.1,
        right_site: surface_site(right.0),
        right_range: right.1,
    })
}

fn extract_exact_source_formula_constant(
    ast: &SurfaceAst,
    expected_label: &str,
    expected_constant: SurfaceFormulaConstant,
) -> Option<SourceFormulaStatement> {
    if ast
        .nodes()
        .iter()
        .any(|node| !is_supported_formula_statement_theorem_bridge_node(node))
    {
        return None;
    }
    let theorem_items = surface_nodes_with_kind(ast, SurfaceNodeKind::TheoremItem);
    let [(_, theorem)] = theorem_items.as_slice() else {
        return None;
    };
    if subtree_has_recovery(ast, theorem) {
        return None;
    }
    let theorem_tokens = direct_token_texts(ast, theorem);
    if theorem_tokens.len() != 4
        || theorem_tokens[0] != "theorem"
        || theorem_tokens[1] != expected_label
        || theorem_tokens[2] != ":"
        || theorem_tokens[3] != ";"
    {
        return None;
    }

    let theorem_structural_children = structural_child_ids(ast, theorem);
    let [formula_expression_id] = theorem_structural_children.as_slice() else {
        return None;
    };
    let formula_expression = ast.node(*formula_expression_id)?;
    if !matches!(formula_expression.kind, SurfaceNodeKind::FormulaExpression) {
        return None;
    }
    let formula_children = structural_child_ids(ast, formula_expression);
    let [formula_id] = formula_children.as_slice() else {
        return None;
    };
    let formula = ast.node(*formula_id)?;
    let expected_spelling = match expected_constant {
        SurfaceFormulaConstant::Thesis => "thesis",
        SurfaceFormulaConstant::Contradiction => "contradiction",
    };
    let constant_matches = match expected_constant {
        SurfaceFormulaConstant::Thesis => matches!(
            formula.kind,
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis)
        ),
        SurfaceFormulaConstant::Contradiction => matches!(
            formula.kind,
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Contradiction)
        ),
    };
    if !constant_matches
        || direct_token_texts(ast, formula).as_slice() != [expected_spelling]
        || !structural_child_ids(ast, formula).is_empty()
    {
        return None;
    }

    Some(SourceFormulaStatement {
        formula_site: surface_site(*formula_id),
        formula_range: formula.range,
    })
}

pub(in crate::runner) fn exact_numeral_term_node_or_expression(
    ast: &SurfaceAst,
    id: SurfaceNodeId,
    expected_spelling: &str,
) -> Option<(SurfaceNodeId, SourceRange)> {
    let node = ast.node(id)?;
    match node.kind {
        SurfaceNodeKind::TermExpression => exact_numeral_term_operand(ast, id, expected_spelling),
        SurfaceNodeKind::NumeralTerm => exact_numeral_term_node(ast, id, expected_spelling),
        _ => None,
    }
}

pub(in crate::runner) fn exact_numeral_term_operand(
    ast: &SurfaceAst,
    term_expression_id: SurfaceNodeId,
    expected_spelling: &str,
) -> Option<(SurfaceNodeId, SourceRange)> {
    let term_expression = ast.node(term_expression_id)?;
    if !matches!(term_expression.kind, SurfaceNodeKind::TermExpression)
        || subtree_has_recovery(ast, term_expression)
    {
        return None;
    }
    let term_children = structural_child_ids(ast, term_expression);
    let [term_id] = term_children.as_slice() else {
        return None;
    };
    exact_numeral_term_node(ast, *term_id, expected_spelling)
}

fn exact_numeral_term_node(
    ast: &SurfaceAst,
    term_id: SurfaceNodeId,
    expected_spelling: &str,
) -> Option<(SurfaceNodeId, SourceRange)> {
    let term = ast.node(term_id)?;
    if matches!(term.kind, SurfaceNodeKind::NumeralTerm)
        && direct_token_texts(ast, term).as_slice() == [expected_spelling]
        && structural_child_ids(ast, term).is_empty()
    {
        Some((term_id, term.range))
    } else {
        None
    }
}

fn is_supported_formula_statement_theorem_bridge_node(node: &SurfaceNode) -> bool {
    matches!(
        node.kind,
        SurfaceNodeKind::Root
            | SurfaceNodeKind::CompilationUnit
            | SurfaceNodeKind::ItemList
            | SurfaceNodeKind::TheoremItem
            | SurfaceNodeKind::FormulaExpression
            | SurfaceNodeKind::FormulaConstant(_)
            | SurfaceNodeKind::Token(_)
    )
}

fn is_supported_builtin_binary_theorem_bridge_node(node: &SurfaceNode) -> bool {
    matches!(
        node.kind,
        SurfaceNodeKind::Root
            | SurfaceNodeKind::CompilationUnit
            | SurfaceNodeKind::ItemList
            | SurfaceNodeKind::TheoremItem
            | SurfaceNodeKind::FormulaExpression
            | SurfaceNodeKind::BuiltinPredicateApplication
            | SurfaceNodeKind::TermExpression
            | SurfaceNodeKind::NumeralTerm
            | SurfaceNodeKind::Token(_)
    )
}
