use crate::{
    OperatorAssociativity, OperatorFixityEntry, ParseRequest, ParserToken, ParserTokenKind, parse,
};
use mizar_session::{
    BuildSnapshotId, Edition, Hash, InMemorySessionIdAllocator, SessionIdAllocator, SourceId,
    SourceRange,
};
use mizar_syntax::{
    SkippedTokenReason, SurfaceBuilderNodeId, SurfaceFormulaConnective, SurfaceFormulaConstant,
    SurfaceNodeKind, SurfaceOperatorAssociativity, SurfaceQuantifierKind, SyntaxDiagnosticCode,
    SyntaxRecoveryKind,
};

fn assert_task33_control_flow_accessors_and_child_order(ast: &mizar_syntax::SurfaceAst) {
    let snapshot = ast.snapshot_text();
    for node_name in [
        "IfStatement",
        "WhileStatement",
        "ForRangeStatement",
        "ForCollectionStatement",
        "MatchStatement",
        "MatchCase",
        "MatchEnding",
        "BreakStatement",
        "ContinueStatement",
    ] {
        assert!(
            snapshot.contains(node_name),
            "parser-produced snapshot should include {node_name}"
        );
    }

    let if_views = surface_views(ast, |kind| matches!(kind, SurfaceNodeKind::IfStatement));
    assert!(
        if_views
            .iter()
            .all(|view| (*view).as_if_statement().is_some())
    );
    let outer_if = if_views
        .iter()
        .copied()
        .find(|view| direct_structural_child_kind_names(*view).contains(&"IfStatement"))
        .expect("else-if should be owned as a nested IfStatement child");
    assert_eq!(
        direct_structural_child_kind_names(outer_if),
        vec!["FormulaExpression", "AlgorithmStatementList", "IfStatement"]
    );

    let while_views = surface_views(ast, |kind| matches!(kind, SurfaceNodeKind::WhileStatement));
    assert!(
        while_views
            .iter()
            .all(|view| (*view).as_while_statement().is_some())
    );
    let while_signatures = while_views
        .into_iter()
        .map(direct_child_signature)
        .collect::<Vec<_>>();
    assert_contains_signature(
        &while_signatures,
        &[
            "while",
            "FormulaExpression",
            "do",
            "AlgorithmStatementList",
            "end",
            ";",
        ],
    );

    let range_views = surface_views(ast, |kind| {
        matches!(kind, SurfaceNodeKind::ForRangeStatement)
    });
    assert!(
        range_views
            .iter()
            .all(|view| (*view).as_for_range_statement().is_some())
    );
    let range_signatures = range_views
        .into_iter()
        .map(direct_child_signature)
        .collect::<Vec<_>>();
    assert_contains_signature(
        &range_signatures,
        &[
            "for",
            "i",
            "=",
            "TermExpression",
            "to",
            "TermExpression",
            "step",
            "TermExpression",
            "do",
            "AlgorithmStatementList",
            "end",
            ";",
        ],
    );
    assert_contains_signature(
        &range_signatures,
        &[
            "for",
            "j",
            "=",
            "TermExpression",
            "downto",
            "TermExpression",
            "do",
            "AlgorithmStatementList",
            "end",
            ";",
        ],
    );

    let collection_views = surface_views(ast, |kind| {
        matches!(kind, SurfaceNodeKind::ForCollectionStatement)
    });
    assert!(
        collection_views
            .iter()
            .all(|view| (*view).as_for_collection_statement().is_some())
    );
    let collection_signatures = collection_views
        .into_iter()
        .map(direct_child_signature)
        .collect::<Vec<_>>();
    assert_contains_signature(
        &collection_signatures,
        &[
            "for",
            "x",
            "in",
            "TermExpression",
            "processed",
            "Seen",
            "do",
            "AlgorithmStatementList",
            "end",
            ";",
        ],
    );
    assert_contains_signature(
        &collection_signatures,
        &[
            "for",
            "plain",
            "in",
            "TermExpression",
            "do",
            "AlgorithmStatementList",
            "end",
            ";",
        ],
    );
    assert!(
        surface_views(ast, |kind| matches!(kind, SurfaceNodeKind::BreakStatement))
            .iter()
            .all(|view| (*view).as_break_statement().is_some())
    );
    assert!(
        surface_views(ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ContinueStatement
        ))
        .iter()
        .all(|view| (*view).as_continue_statement().is_some())
    );

    let match_views = surface_views(ast, |kind| matches!(kind, SurfaceNodeKind::MatchStatement));
    assert!(
        match_views
            .iter()
            .all(|view| (*view).as_match_statement().is_some())
    );
    let multi_case_match = match_views
        .iter()
        .copied()
        .find(|view| {
            view.child_views()
                .filter(|child| child.as_match_case().is_some())
                .count()
                == 2
        })
        .expect("first match should own both case branches before its ending");
    assert_eq!(
        direct_structural_child_kind_names(multi_case_match),
        vec!["TermExpression", "MatchCase", "MatchCase", "MatchEnding"]
    );

    let ending_views = surface_views(ast, |kind| matches!(kind, SurfaceNodeKind::MatchEnding));
    assert_eq!(ending_views.len(), 1);
    assert!(
        ending_views
            .iter()
            .all(|view| (*view).as_match_ending().is_some())
    );
    let otherwise_ending = ending_views
        .iter()
        .copied()
        .find(|view| view_starts_with_token(*view, "otherwise"))
        .expect("otherwise ending should be present");
    assert_eq!(
        direct_structural_child_kind_names(otherwise_ending),
        vec!["AlgorithmStatementList"]
    );

    let omitted_ending_matches = match_views
        .iter()
        .copied()
        .filter(|view| {
            view.child_views()
                .filter(|child| child.as_match_case().is_some())
                .count()
                == 1
        })
        .collect::<Vec<_>>();
    assert_eq!(omitted_ending_matches.len(), 2);
    for match_view in omitted_ending_matches {
        assert_eq!(
            direct_structural_child_kind_names(match_view),
            vec!["TermExpression", "MatchCase"]
        );
        assert_eq!(
            direct_child_signature(match_view),
            vec!["match", "TermExpression", "do", "MatchCase", "end", ";"]
        );
    }
}

fn assert_task34_verification_accessors_and_child_order(ast: &mizar_syntax::SurfaceAst) {
    let snapshot = ast.snapshot_text();
    for node_name in [
        "AlgorithmTerminationClause",
        "AlgorithmRequiresClause",
        "AlgorithmEnsuresClause",
        "AlgorithmDecreasingClause",
        "LoopInvariantClause",
        "LoopDecreasingClause",
        "AssertStatement",
        "TermList",
    ] {
        assert!(
            snapshot.contains(node_name),
            "parser-produced snapshot should include {node_name}"
        );
    }

    assert!(
        surface_views(ast, |kind| matches!(
            kind,
            SurfaceNodeKind::AlgorithmTerminationClause
        ))
        .iter()
        .all(|view| (*view).as_algorithm_termination_clause().is_some())
    );
    assert!(
        surface_views(ast, |kind| matches!(
            kind,
            SurfaceNodeKind::AlgorithmRequiresClause
        ))
        .iter()
        .all(|view| (*view).as_algorithm_requires_clause().is_some())
    );
    assert!(
        surface_views(ast, |kind| matches!(
            kind,
            SurfaceNodeKind::AlgorithmEnsuresClause
        ))
        .iter()
        .all(|view| (*view).as_algorithm_ensures_clause().is_some())
    );
    assert!(
        surface_views(ast, |kind| matches!(
            kind,
            SurfaceNodeKind::AlgorithmDecreasingClause
        ))
        .iter()
        .all(|view| (*view).as_algorithm_decreasing_clause().is_some())
    );
    assert!(
        surface_views(ast, |kind| matches!(
            kind,
            SurfaceNodeKind::LoopInvariantClause
        ))
        .iter()
        .all(|view| (*view).as_loop_invariant_clause().is_some())
    );
    assert!(
        surface_views(ast, |kind| matches!(
            kind,
            SurfaceNodeKind::LoopDecreasingClause
        ))
        .iter()
        .all(|view| (*view).as_loop_decreasing_clause().is_some())
    );
    assert!(
        surface_views(ast, |kind| matches!(kind, SurfaceNodeKind::AssertStatement))
            .iter()
            .all(|view| (*view).as_assert_statement().is_some())
    );
    assert!(
        surface_views(ast, |kind| matches!(kind, SurfaceNodeKind::TermList))
            .iter()
            .all(|view| (*view).as_term_list().is_some())
    );

    let algorithm_signatures = surface_views(ast, |kind| {
        matches!(kind, SurfaceNodeKind::AlgorithmDefinition)
    })
    .into_iter()
    .map(direct_child_signature)
    .collect::<Vec<_>>();
    assert_contains_signature(
        &algorithm_signatures,
        &[
            "AlgorithmTerminationClause",
            "algorithm",
            "verified",
            "AlgorithmParameters",
            "->",
            "TypeExpression",
            "AlgorithmRequiresClause",
            "AlgorithmEnsuresClause",
            "AlgorithmDecreasingClause",
            "AlgorithmBody",
            ";",
        ],
    );
    assert_contains_signature(
        &algorithm_signatures,
        &[
            "AlgorithmTerminationClause",
            "algorithm",
            "bare",
            "AlgorithmParameters",
            "AlgorithmBody",
            ";",
        ],
    );

    let term_list_signatures = surface_views(ast, |kind| matches!(kind, SurfaceNodeKind::TermList))
        .into_iter()
        .map(direct_child_signature)
        .collect::<Vec<_>>();
    assert_contains_signature(
        &term_list_signatures,
        &["TermExpression", ",", "TermExpression"],
    );

    let invariant_signatures = surface_views(ast, |kind| {
        matches!(kind, SurfaceNodeKind::LoopInvariantClause)
    })
    .into_iter()
    .map(direct_child_signature)
    .collect::<Vec<_>>();
    assert_contains_signature(
        &invariant_signatures,
        &["invariant", "FormulaExpression", "JustificationClause", ";"],
    );
    assert_contains_signature(
        &invariant_signatures,
        &["invariant", "FormulaExpression", ";"],
    );

    let decreasing_signatures = surface_views(ast, |kind| {
        matches!(kind, SurfaceNodeKind::LoopDecreasingClause)
    })
    .into_iter()
    .map(direct_child_signature)
    .collect::<Vec<_>>();
    assert_contains_signature(
        &decreasing_signatures,
        &["decreasing", "TermList", "JustificationClause", ";"],
    );

    let while_signatures =
        surface_views(ast, |kind| matches!(kind, SurfaceNodeKind::WhileStatement))
            .into_iter()
            .map(direct_child_signature)
            .collect::<Vec<_>>();
    assert_contains_signature(
        &while_signatures,
        &[
            "while",
            "FormulaExpression",
            "do",
            "LoopInvariantClause",
            "LoopDecreasingClause",
            "AlgorithmStatementList",
            "end",
            ";",
        ],
    );

    let range_signatures = surface_views(ast, |kind| {
        matches!(kind, SurfaceNodeKind::ForRangeStatement)
    })
    .into_iter()
    .map(direct_child_signature)
    .collect::<Vec<_>>();
    assert_contains_signature(
        &range_signatures,
        &[
            "for",
            "i",
            "=",
            "TermExpression",
            "to",
            "TermExpression",
            "do",
            "LoopInvariantClause",
            "AlgorithmStatementList",
            "end",
            ";",
        ],
    );

    let collection_signatures = surface_views(ast, |kind| {
        matches!(kind, SurfaceNodeKind::ForCollectionStatement)
    })
    .into_iter()
    .map(direct_child_signature)
    .collect::<Vec<_>>();
    assert_contains_signature(
        &collection_signatures,
        &[
            "for",
            "item",
            "in",
            "TermExpression",
            "do",
            "LoopInvariantClause",
            "AlgorithmStatementList",
            "end",
            ";",
        ],
    );

    let assert_signatures =
        surface_views(ast, |kind| matches!(kind, SurfaceNodeKind::AssertStatement))
            .into_iter()
            .map(direct_child_signature)
            .collect::<Vec<_>>();
    assert_contains_signature(
        &assert_signatures,
        &["assert", "FormulaExpression", "JustificationClause", ";"],
    );
    assert_contains_signature(&assert_signatures, &["assert", "FormulaExpression", ";"]);
}

fn single_node(
    ast: &mizar_syntax::SurfaceAst,
    predicate: impl Fn(&SurfaceNodeKind) -> bool,
) -> &mizar_syntax::SurfaceNode {
    ast.nodes()
        .iter()
        .find(|node| predicate(&node.kind))
        .expect("expected exactly one matching node")
}

fn formula_root_for_label<'a>(
    ast: &'a mizar_syntax::SurfaceAst,
    label: &str,
) -> &'a mizar_syntax::SurfaceNode {
    let item = ast
        .nodes()
        .iter()
        .find(|node| {
            matches!(
                node.kind,
                SurfaceNodeKind::TheoremItem
                    | SurfaceNodeKind::LemmaItem
                    | SurfaceNodeKind::PlaceholderItem
            ) && node.children.iter().any(|child| {
                ast.node(*child)
                    .and_then(mizar_syntax::SurfaceNode::token_text)
                    == Some(label)
            })
        })
        .unwrap_or_else(|| panic!("expected theorem-like item labelled `{label}`"));
    let formula_expression = item
        .children
        .iter()
        .filter_map(|child| ast.node(*child))
        .find(|child| matches!(child.kind, SurfaceNodeKind::FormulaExpression))
        .unwrap_or_else(|| panic!("expected formula expression for `{label}`"));
    assert_eq!(
        formula_expression.children.len(),
        1,
        "FormulaExpression for `{label}` should wrap exactly one child"
    );
    ast.node(formula_expression.children[0])
        .unwrap_or_else(|| panic!("expected formula child for `{label}`"))
}

fn set_comprehension_term_for_label<'a>(
    ast: &'a mizar_syntax::SurfaceAst,
    label: &str,
) -> &'a mizar_syntax::SurfaceNode {
    let formula = formula_root_for_label(ast, label);
    let term = builtin_predicate_left_term_payload(ast, formula);
    assert!(
        matches!(term.kind, SurfaceNodeKind::SetComprehension),
        "expected left term for `{label}` to be SetComprehension, got {:?}",
        term.kind
    );
    term
}

fn builtin_predicate_left_term_payload<'a>(
    ast: &'a mizar_syntax::SurfaceAst,
    formula: &mizar_syntax::SurfaceNode,
) -> &'a mizar_syntax::SurfaceNode {
    assert!(
        matches!(formula.kind, SurfaceNodeKind::BuiltinPredicateApplication),
        "expected builtin predicate application, got {:?}",
        formula.kind
    );
    let terms = structural_children(ast, formula)
        .into_iter()
        .filter(|child| matches!(child.kind, SurfaceNodeKind::TermExpression))
        .collect::<Vec<_>>();
    assert!(
        terms.len() >= 2,
        "builtin predicate application should have left and right terms"
    );
    let left_children = structural_children(ast, terms[0]);
    assert_eq!(
        left_children.len(),
        1,
        "left TermExpression should wrap one concrete term"
    );
    left_children[0]
}

fn set_comprehension_mapper(
    ast: &mizar_syntax::SurfaceAst,
    comprehension: &mizar_syntax::SurfaceNode,
) -> mizar_syntax::SurfaceNodeId {
    assert!(
        matches!(comprehension.kind, SurfaceNodeKind::SetComprehension),
        "expected SetComprehension, got {:?}",
        comprehension.kind
    );
    comprehension
        .children
        .iter()
        .copied()
        .find(|child| {
            ast.node(*child)
                .is_some_and(|node| matches!(node.kind, SurfaceNodeKind::TermExpression))
        })
        .expect("SetComprehension should own a mapper TermExpression")
}

fn structural_children<'a>(
    ast: &'a mizar_syntax::SurfaceAst,
    node: &mizar_syntax::SurfaceNode,
) -> Vec<&'a mizar_syntax::SurfaceNode> {
    node.children
        .iter()
        .filter_map(|child| ast.node(*child))
        .filter(|child| !matches!(child.kind, SurfaceNodeKind::Token(_)))
        .collect()
}

fn direct_child_labels<'a>(
    ast: &'a mizar_syntax::SurfaceAst,
    node: &'a mizar_syntax::SurfaceNode,
) -> Vec<&'a str> {
    node.children
        .iter()
        .filter_map(|child| ast.node(*child))
        .map(|child| match child.token_text() {
            Some(text) => text,
            None => match child.kind {
                SurfaceNodeKind::Annotation => "Annotation",
                SurfaceNodeKind::DefinitionParameter => "DefinitionParameter",
                SurfaceNodeKind::QualifiedSymbol => "QualifiedSymbol",
                SurfaceNodeKind::TypeArguments => "TypeArguments",
                SurfaceNodeKind::TypeHead => "TypeHead",
                SurfaceNodeKind::FormulaDefiniens => "FormulaDefiniens",
                SurfaceNodeKind::TermDefiniens => "TermDefiniens",
                SurfaceNodeKind::CorrectnessCondition => "CorrectnessCondition",
                SurfaceNodeKind::VisibilityMarker => "VisibilityMarker",
                SurfaceNodeKind::OperatorDeclaration => "OperatorDeclaration",
                SurfaceNodeKind::ErrorRecovery(_) => "ErrorRecovery",
                ref other => panic!("unexpected direct child kind in assertion: {other:?}"),
            },
        })
        .collect()
}

fn surface_views<'a>(
    ast: &'a mizar_syntax::SurfaceAst,
    predicate: impl Fn(&SurfaceNodeKind) -> bool + Copy,
) -> Vec<mizar_syntax::SurfaceNodeView<'a>> {
    let mut views = Vec::new();
    if let Some(root) = ast.root_view() {
        collect_surface_views(root, predicate, &mut views);
    }
    views
}

fn collect_surface_views<'a>(
    view: mizar_syntax::SurfaceNodeView<'a>,
    predicate: impl Fn(&SurfaceNodeKind) -> bool + Copy,
    views: &mut Vec<mizar_syntax::SurfaceNodeView<'a>>,
) {
    if predicate(view.kind()) {
        views.push(view);
    }
    for child in view.child_views() {
        collect_surface_views(child, predicate, views);
    }
}

fn direct_structural_child_kind_names(
    view: mizar_syntax::SurfaceNodeView<'_>,
) -> Vec<&'static str> {
    view.child_views()
        .filter(|child| !matches!(child.kind(), SurfaceNodeKind::Token(_)))
        .filter_map(|child| task33_kind_name(child.kind()))
        .collect()
}

fn direct_child_signature(view: mizar_syntax::SurfaceNodeView<'_>) -> Vec<String> {
    view.child_views()
        .filter_map(|child| {
            view_token_text(child)
                .map(str::to_owned)
                .or_else(|| task33_kind_name(child.kind()).map(str::to_owned))
        })
        .collect()
}

fn assert_contains_signature(signatures: &[Vec<String>], expected: &[&str]) {
    let expected = expected
        .iter()
        .copied()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    assert!(
        signatures.iter().any(|signature| signature == &expected),
        "expected signature {:?}, got {signatures:#?}",
        expected
    );
}

fn task33_kind_name(kind: &SurfaceNodeKind) -> Option<&'static str> {
    match kind {
        SurfaceNodeKind::FormulaExpression => Some("FormulaExpression"),
        SurfaceNodeKind::TermExpression => Some("TermExpression"),
        SurfaceNodeKind::TypeExpression => Some("TypeExpression"),
        SurfaceNodeKind::AlgorithmParameters => Some("AlgorithmParameters"),
        SurfaceNodeKind::AlgorithmBody => Some("AlgorithmBody"),
        SurfaceNodeKind::AlgorithmStatementList => Some("AlgorithmStatementList"),
        SurfaceNodeKind::AlgorithmTerminationClause => Some("AlgorithmTerminationClause"),
        SurfaceNodeKind::AlgorithmRequiresClause => Some("AlgorithmRequiresClause"),
        SurfaceNodeKind::AlgorithmEnsuresClause => Some("AlgorithmEnsuresClause"),
        SurfaceNodeKind::AlgorithmDecreasingClause => Some("AlgorithmDecreasingClause"),
        SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTerm) => Some("MissingTerm"),
        SurfaceNodeKind::IfStatement => Some("IfStatement"),
        SurfaceNodeKind::LoopInvariantClause => Some("LoopInvariantClause"),
        SurfaceNodeKind::LoopDecreasingClause => Some("LoopDecreasingClause"),
        SurfaceNodeKind::AssertStatement => Some("AssertStatement"),
        SurfaceNodeKind::TermList => Some("TermList"),
        SurfaceNodeKind::MatchCase => Some("MatchCase"),
        SurfaceNodeKind::MatchEnding => Some("MatchEnding"),
        SurfaceNodeKind::JustificationClause => Some("JustificationClause"),
        SurfaceNodeKind::PredicatePattern => Some("PredicatePattern"),
        SurfaceNodeKind::FormulaDefiniens => Some("FormulaDefiniens"),
        SurfaceNodeKind::CoherenceCondition => Some("CoherenceCondition"),
        _ => None,
    }
}

fn view_starts_with_token(view: mizar_syntax::SurfaceNodeView<'_>, expected: &str) -> bool {
    view.child_views().find_map(view_token_text) == Some(expected)
}

fn view_token_text(view: mizar_syntax::SurfaceNodeView<'_>) -> Option<&str> {
    match view.kind() {
        SurfaceNodeKind::Token(token) => Some(token.text.as_ref()),
        _ => None,
    }
}

fn direct_token_texts(
    ast: &mizar_syntax::SurfaceAst,
    node: &mizar_syntax::SurfaceNode,
) -> Vec<String> {
    node.children
        .iter()
        .filter_map(|child| ast.node(*child))
        .filter_map(mizar_syntax::SurfaceNode::token_text)
        .map(str::to_owned)
        .collect()
}

fn subtree_token_texts(
    ast: &mizar_syntax::SurfaceAst,
    node: &mizar_syntax::SurfaceNode,
) -> Vec<String> {
    let mut texts = Vec::new();
    collect_subtree_token_texts(ast, node, &mut texts);
    texts
}

fn collect_subtree_token_texts(
    ast: &mizar_syntax::SurfaceAst,
    node: &mizar_syntax::SurfaceNode,
    texts: &mut Vec<String>,
) {
    for child in &node.children {
        let Some(child) = ast.node(*child) else {
            continue;
        };
        if let Some(text) = child.token_text() {
            texts.push(text.to_owned());
        } else {
            collect_subtree_token_texts(ast, child, texts);
        }
    }
}

fn assert_binary_formula(
    node: &mizar_syntax::SurfaceNode,
    connective: SurfaceFormulaConnective,
    repeated: bool,
) {
    let SurfaceNodeKind::BinaryFormula(operator) = &node.kind else {
        panic!("expected BinaryFormula, got {:?}", node.kind);
    };
    assert_eq!(operator.connective, connective);
    assert_eq!(operator.repeated, repeated);
}

fn assert_quantified_formula(node: &mizar_syntax::SurfaceNode, quantifier: SurfaceQuantifierKind) {
    let SurfaceNodeKind::QuantifiedFormula(actual) = &node.kind else {
        panic!("expected QuantifiedFormula, got {:?}", node.kind);
    };
    assert_eq!(*actual, quantifier);
}

fn assert_formula_constant(node: &mizar_syntax::SurfaceNode, constant: SurfaceFormulaConstant) {
    let SurfaceNodeKind::FormulaConstant(actual) = &node.kind else {
        panic!("expected FormulaConstant, got {:?}", node.kind);
    };
    assert_eq!(*actual, constant);
}

fn count_nodes(
    ast: &mizar_syntax::SurfaceAst,
    predicate: impl Fn(&SurfaceNodeKind) -> bool,
) -> usize {
    ast.nodes()
        .iter()
        .filter(|node| predicate(&node.kind))
        .count()
}

fn subtree_contains_kind(
    ast: &mizar_syntax::SurfaceAst,
    id: mizar_syntax::SurfaceNodeId,
    predicate: impl Fn(&SurfaceNodeKind) -> bool + Copy,
) -> bool {
    let Some(node) = ast.node(id) else {
        return false;
    };
    predicate(&node.kind)
        || node
            .children
            .iter()
            .any(|child| subtree_contains_kind(ast, *child, predicate))
}

fn node_subtree_contains_kind(
    ast: &mizar_syntax::SurfaceAst,
    node: &mizar_syntax::SurfaceNode,
    predicate: impl Fn(&SurfaceNodeKind) -> bool + Copy,
) -> bool {
    predicate(&node.kind)
        || node
            .children
            .iter()
            .any(|child| subtree_contains_kind(ast, *child, predicate))
}

fn direct_child_has_kind(
    ast: &mizar_syntax::SurfaceAst,
    node: &mizar_syntax::SurfaceNode,
    predicate: impl Fn(&SurfaceNodeKind) -> bool,
) -> bool {
    node.children
        .iter()
        .filter_map(|child| ast.node(*child))
        .any(|child| predicate(&child.kind))
}

fn rowan_token_texts(ast: &mizar_syntax::SurfaceAst) -> Vec<String> {
    ast.rowan_root()
        .descendants_with_tokens()
        .filter_map(|element| element.into_token())
        .map(|token| token.text().to_owned())
        .collect()
}

fn qualified_symbol_token_texts(
    ast: &mizar_syntax::SurfaceAst,
    qualified: &mizar_syntax::SurfaceNode,
) -> Vec<String> {
    qualified
        .children
        .iter()
        .filter_map(|id| ast.node(*id))
        .filter_map(|node| {
            node.token_text().map(str::to_owned).or_else(|| {
                node.children
                    .first()
                    .and_then(|id| ast.node(*id))
                    .and_then(mizar_syntax::SurfaceNode::token_text)
                    .map(str::to_owned)
            })
        })
        .collect()
}

const B1B1P_RECOVERY_SOURCE: &str = concat!(
    "import parser.type_fixtures;\n",
    "reserve x for set;\n",
    "theorem FormulaStatementParenthesizedApplicationWitnessSmoke: x = x proof\n",
    "  take (1 ++ 2);\n",
    "  thus x = x;\n",
    "end;\n",
);

fn b1b1p_recovery_tokens(source_id: SourceId, mutation: usize) -> Vec<ParserToken> {
    let mut entries = vec![
        ("import".to_owned(), ParserTokenKind::ReservedWord),
        ("parser".to_owned(), ParserTokenKind::Identifier),
        (".".to_owned(), ParserTokenKind::ReservedSymbol),
        ("type_fixtures".to_owned(), ParserTokenKind::Identifier),
        (";".to_owned(), ParserTokenKind::ReservedSymbol),
        ("reserve".to_owned(), ParserTokenKind::ReservedWord),
        ("x".to_owned(), ParserTokenKind::Identifier),
        ("for".to_owned(), ParserTokenKind::ReservedWord),
        ("set".to_owned(), ParserTokenKind::ReservedWord),
        (";".to_owned(), ParserTokenKind::ReservedSymbol),
    ];

    if (48..=54).contains(&mutation) {
        let keyword_offset = mutation - 48;
        let (prefix, suffix_with_mutation) = "theorem".split_at(keyword_offset);
        let suffix = &suffix_with_mutation[1..];
        if !prefix.is_empty() {
            entries.push((prefix.to_owned(), ParserTokenKind::Identifier));
        }
        entries.push(("!".to_owned(), ParserTokenKind::UserSymbol));
        if !suffix.is_empty() {
            entries.push((suffix.to_owned(), ParserTokenKind::Identifier));
        }
    } else {
        entries.push(("theorem".to_owned(), ParserTokenKind::ReservedWord));
    }

    entries.extend([
        (
            "FormulaStatementParenthesizedApplicationWitnessSmoke".to_owned(),
            ParserTokenKind::Identifier,
        ),
        (":".to_owned(), ParserTokenKind::ReservedSymbol),
        ("x".to_owned(), ParserTokenKind::Identifier),
        (
            if mutation == 112 { "!" } else { "=" }.to_owned(),
            if mutation == 112 {
                ParserTokenKind::UserSymbol
            } else {
                ParserTokenKind::ReservedSymbol
            },
        ),
        ("x".to_owned(), ParserTokenKind::Identifier),
        ("proof".to_owned(), ParserTokenKind::ReservedWord),
        ("take".to_owned(), ParserTokenKind::ReservedWord),
        ("(".to_owned(), ParserTokenKind::ReservedSymbol),
        ("1".to_owned(), ParserTokenKind::Numeral),
        ("++".to_owned(), ParserTokenKind::UserSymbol),
        ("2".to_owned(), ParserTokenKind::Numeral),
        (")".to_owned(), ParserTokenKind::ReservedSymbol),
        (";".to_owned(), ParserTokenKind::ReservedSymbol),
        ("thus".to_owned(), ParserTokenKind::ReservedWord),
        ("x".to_owned(), ParserTokenKind::Identifier),
        (
            if mutation == 148 { "!" } else { "=" }.to_owned(),
            if mutation == 148 {
                ParserTokenKind::UserSymbol
            } else {
                ParserTokenKind::ReservedSymbol
            },
        ),
        ("x".to_owned(), ParserTokenKind::Identifier),
        (";".to_owned(), ParserTokenKind::ReservedSymbol),
        ("end".to_owned(), ParserTokenKind::ReservedWord),
        (";".to_owned(), ParserTokenKind::ReservedSymbol),
    ]);

    let mut cursor = 0;
    entries
        .into_iter()
        .map(|(text, kind)| {
            let start = cursor;
            let end = start + text.len();
            cursor = end + 1;
            token(source_id, kind, &text, start, end)
        })
        .collect()
}

fn token(
    source_id: SourceId,
    kind: ParserTokenKind,
    text: &str,
    start: usize,
    end: usize,
) -> ParserToken {
    ParserToken::new(kind, text, range(source_id, start, end))
}

fn token_sequence(source_id: SourceId, entries: &[(&str, ParserTokenKind)]) -> Vec<ParserToken> {
    let mut cursor = 0;
    entries
        .iter()
        .map(|(text, kind)| {
            let start = cursor;
            let end = start + text.len();
            cursor = end + 1;
            token(source_id, *kind, text, start, end)
        })
        .collect()
}

const FRAENKEL_TEMPLATE_SOURCE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/miz/fail/templates/fail_template_fraenkel_over_type_param_001.miz"
));

fn fraenkel_template_tokens(source_id: SourceId) -> Vec<ParserToken> {
    let entries = [
        ("definition", ParserTokenKind::ReservedWord, 593, 603),
        ("let", ParserTokenKind::ReservedWord, 606, 609),
        ("T", ParserTokenKind::Identifier, 610, 611),
        ("be", ParserTokenKind::ReservedWord, 612, 614),
        ("type", ParserTokenKind::ReservedWord, 615, 619),
        (";", ParserTokenKind::ReservedSymbol, 619, 620),
        ("func", ParserTokenKind::ReservedWord, 623, 627),
        ("ParaDef", ParserTokenKind::Identifier, 628, 635),
        (":", ParserTokenKind::ReservedSymbol, 635, 636),
        ("para", ParserTokenKind::Identifier, 637, 641),
        ("[", ParserTokenKind::ReservedSymbol, 641, 642),
        ("T", ParserTokenKind::Identifier, 642, 643),
        ("]", ParserTokenKind::ReservedSymbol, 643, 644),
        ("->", ParserTokenKind::ReservedSymbol, 645, 647),
        ("set", ParserTokenKind::ReservedWord, 648, 651),
        ("equals", ParserTokenKind::ReservedWord, 652, 658),
        ("{", ParserTokenKind::ReservedSymbol, 663, 664),
        ("x", ParserTokenKind::Identifier, 665, 666),
        ("where", ParserTokenKind::ReservedWord, 667, 672),
        ("x", ParserTokenKind::Identifier, 673, 674),
        ("is", ParserTokenKind::ReservedWord, 675, 677),
        ("T", ParserTokenKind::Identifier, 678, 679),
        (":", ParserTokenKind::ReservedSymbol, 680, 681),
        ("not", ParserTokenKind::ReservedWord, 682, 685),
        ("x", ParserTokenKind::Identifier, 686, 687),
        ("in", ParserTokenKind::ReservedWord, 688, 690),
        ("x", ParserTokenKind::Identifier, 691, 692),
        ("}", ParserTokenKind::ReservedSymbol, 693, 694),
        (";", ParserTokenKind::ReservedSymbol, 694, 695),
        ("end", ParserTokenKind::ReservedWord, 696, 699),
        (";", ParserTokenKind::ReservedSymbol, 699, 700),
    ];

    assert_eq!(FRAENKEL_TEMPLATE_SOURCE.len(), 701);
    entries
        .into_iter()
        .map(|(text, kind, start, end)| {
            assert_eq!(&FRAENKEL_TEMPLATE_SOURCE[start..end], text);
            token(source_id, kind, text, start, end)
        })
        .collect()
}

#[derive(Debug)]
struct ExpectedParserDiagnostic {
    code: SyntaxDiagnosticCode,
    message: &'static str,
    primary_text: &'static str,
    primary_ordinal: usize,
    recovery_note: &'static str,
}

impl ExpectedParserDiagnostic {
    fn term(message: &'static str, primary_text: &'static str, primary_ordinal: usize) -> Self {
        Self {
            code: SyntaxDiagnosticCode::MalformedTermExpression,
            message,
            primary_text,
            primary_ordinal,
            recovery_note: "repair the term expression before continuing",
        }
    }

    fn formula(message: &'static str, primary_text: &'static str, primary_ordinal: usize) -> Self {
        Self {
            code: SyntaxDiagnosticCode::MalformedFormulaExpression,
            message,
            primary_text,
            primary_ordinal,
            recovery_note: "repair the formula expression before continuing",
        }
    }
}

fn assert_task33_algorithm_body_diagnostics_exact(
    source_id: SourceId,
    body: &[(&str, ParserTokenKind)],
    expected: &[ExpectedParserDiagnostic],
) {
    let tokens = task33_algorithm_body_tokens(source_id, body);
    assert_parsed_tokens_diagnostics_exact(source_id, tokens, expected);
}

fn assert_parser_diagnostics_exact(
    source_id: SourceId,
    entries: &[(&str, ParserTokenKind)],
    expected: &[ExpectedParserDiagnostic],
) {
    let tokens = token_sequence(source_id, entries);
    assert_parsed_tokens_diagnostics_exact(source_id, tokens, expected);
}

fn assert_parsed_tokens_diagnostics_exact(
    source_id: SourceId,
    tokens: Vec<ParserToken>,
    expected: &[ExpectedParserDiagnostic],
) {
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens.clone(),
        Vec::new(),
    ));

    assert_eq!(
        output.diagnostics.len(),
        expected.len(),
        "{:#?}",
        output.diagnostics
    );
    for (diagnostic, expected) in output.diagnostics.iter().zip(expected) {
        assert_matches_expected_diagnostic(diagnostic, &tokens, expected);
    }
}

fn task33_algorithm_body_tokens(
    source_id: SourceId,
    body: &[(&str, ParserTokenKind)],
) -> Vec<ParserToken> {
    let mut entries = vec![
        ("definition", ParserTokenKind::ReservedWord),
        ("algorithm", ParserTokenKind::ReservedWord),
        ("diagnostic_flow", ParserTokenKind::Identifier),
        ("(", ParserTokenKind::ReservedSymbol),
        (")", ParserTokenKind::ReservedSymbol),
        ("do", ParserTokenKind::ReservedWord),
    ];
    entries.extend_from_slice(body);
    entries.extend_from_slice(&[
        ("end", ParserTokenKind::ReservedWord),
        (";", ParserTokenKind::ReservedSymbol),
        ("end", ParserTokenKind::ReservedWord),
        (";", ParserTokenKind::ReservedSymbol),
    ]);
    token_sequence(source_id, &entries)
}

fn assert_matches_expected_diagnostic(
    diagnostic: &mizar_syntax::SyntaxDiagnostic,
    tokens: &[ParserToken],
    expected: &ExpectedParserDiagnostic,
) {
    assert!(
        diagnostic_matches_expected(diagnostic, tokens, expected),
        "expected {expected:?}, got {diagnostic:#?}"
    );
}

fn diagnostic_matches_expected(
    diagnostic: &mizar_syntax::SyntaxDiagnostic,
    tokens: &[ParserToken],
    expected: &ExpectedParserDiagnostic,
) -> bool {
    diagnostic.code == expected.code
        && diagnostic.message.as_ref() == expected.message
        && diagnostic.primary
            == nth_token_range(tokens, expected.primary_text, expected.primary_ordinal)
        && diagnostic.recovery_note.as_deref() == Some(expected.recovery_note)
}

fn nth_token_range(tokens: &[ParserToken], text: &str, nth: usize) -> SourceRange {
    tokens
        .iter()
        .filter(|token| token.text.as_ref() == text)
        .nth(nth)
        .unwrap_or_else(|| panic!("expected token `{text}` at ordinal {nth}"))
        .span
}

fn operator_fixture_fixities() -> Vec<OperatorFixityEntry> {
    vec![
        OperatorFixityEntry::prefix("~", 70),
        OperatorFixityEntry::postfix("!", 90),
        OperatorFixityEntry::infix("++", 10, OperatorAssociativity::Left),
        OperatorFixityEntry::infix("**", 20, OperatorAssociativity::Right),
        OperatorFixityEntry::infix("%%", 10, OperatorAssociativity::NonAssociative),
    ]
}

const fn range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
    SourceRange {
        source_id,
        start,
        end,
    }
}

fn source_id(byte: u8) -> SourceId {
    InMemorySessionIdAllocator::new()
        .next_source_id(snapshot_id(byte))
        .unwrap()
}

fn run_step5a8_private_path(
    entries: &[(&str, ParserTokenKind)],
    parse: impl FnOnce(&mut crate::grammar::Parser) -> SurfaceBuilderNodeId,
) -> (
    mizar_syntax::SurfaceAst,
    Vec<mizar_syntax::SyntaxDiagnostic>,
) {
    let source_id = source_id(253);
    let mut parser = crate::grammar::Parser::new(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(source_id, entries),
        Vec::new(),
    ));
    parser.add_token_nodes();
    let node = parse(&mut parser);
    let diagnostics = parser.diagnostics.clone();
    let ast = parser.events.finish(None, Some(node));
    (ast, diagnostics)
}

fn assert_step5a8_omitted_path(
    entries: &[(&str, ParserTokenKind)],
    parse: impl FnOnce(&mut crate::grammar::Parser) -> SurfaceBuilderNodeId,
    expected_kind: impl Fn(&SurfaceNodeKind) -> bool,
) {
    let (ast, diagnostics) = run_step5a8_private_path(entries, parse);
    assert!(
        diagnostics.is_empty(),
        "omitted path diagnostics: {diagnostics:#?}"
    );
    assert_eq!(count_nodes(&ast, expected_kind), 1);
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(_)
        )),
        0,
        "omitted justifications must not fabricate recovery nodes"
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::JustificationClause
        )),
        0,
        "omitted justifications must not fabricate JustificationClause nodes"
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingProofStep)
        )),
        0,
        "omitted justifications must not fabricate MissingProofStep recovery"
    );
}

fn assert_step5a8_malformed_path(
    entries: &[(&str, ParserTokenKind)],
    parse: impl FnOnce(&mut crate::grammar::Parser) -> SurfaceBuilderNodeId,
    expected_kind: impl Fn(&SurfaceNodeKind) -> bool,
    expected_recovery: impl Fn(&SurfaceNodeKind) -> bool,
) {
    let (ast, diagnostics) = run_step5a8_private_path(entries, parse);
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MalformedJustification),
        "malformed path should retain MalformedJustification: {diagnostics:#?}"
    );
    assert_eq!(count_nodes(&ast, expected_kind), 1);
    assert!(
        count_nodes(&ast, expected_recovery) > 0,
        "malformed path should retain its recovery node"
    );
}

fn snapshot_id(byte: u8) -> BuildSnapshotId {
    let hex = format!("{byte:02x}").repeat(Hash::BYTE_LEN);
    BuildSnapshotId::from_published_schema_str(&format!("mizar-session-build-snapshot-v1:{hex}"))
        .unwrap()
}

mod malformed;
mod misc;
mod parser_diagnoses;
mod parser_keeps;
mod parser_parses;
mod parser_pins;
mod parser_preserves;
mod parser_recovers;
mod parser_task46;
mod task48;
