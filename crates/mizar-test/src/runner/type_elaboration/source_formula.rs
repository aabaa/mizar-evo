use mizar_checker::typed_ast::TypedSiteRef;
use mizar_session::SourceRange;
use mizar_syntax::{SurfaceAst, SurfaceFormulaConstant, SurfaceNode, SurfaceNodeKind};

use super::source_ast::{
    direct_token_texts, structural_child_ids, subtree_has_recovery, surface_nodes_with_kind,
    surface_site,
};

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
