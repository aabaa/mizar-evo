use super::*;

#[test]
fn match_explicit_formula_patterns_preserve_direct_payloads_and_case_boundaries() {
    use mizar_syntax::{SurfaceFormulaPrefixOperator as P, SurfaceNodeKind as K, SurfaceQuantifierKind as Q, SyntaxKind as S};
    let config = step5c11_config();
    let plan = build_test_plan(&config).unwrap();
    let case = plan.cases.iter().find(|case| case.id.0 == "pass_parser_match_explicit_formulas_001").unwrap();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let first = super::super::run_parse_only_case(&config.workspace_root, &config.workspace_root.join("tests"), case, 141);
    assert_eq!(first.status, super::super::ParseOnlyCaseStatus::Passed, "{first:?}");
    assert_eq!(first, super::super::run_parse_only_case(&config.workspace_root, &config.workspace_root.join("tests"), case, 141));
    let output = super::super::formula_statement::step5c8_test_frontend(&text);
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    let ast = output.ast.unwrap();
    assert!(ast.node_views().all(|v| !v.is_recovered()));
    assert!(ast.trivia().skipped_token_ranges().is_empty());
    for (kind, syntax, count) in [(K::MatchStatement, S::MatchStatement, 1), (K::MatchCase, S::MatchCase, 12), (K::MatchEnding, S::MatchEnding, 1), (K::ReturnStatement, S::ReturnStatement, 14)] {
        assert_eq!(ast.node_views().filter(|v| v.kind() == &kind).count(), count);
        assert_eq!(ast.rowan_root().descendants().filter(|v| v.kind() == syntax).count(), count);
        assert_eq!(ast.snapshot_text().matches(&format!("{kind:?}")).count(), count);
    }
    let cases = ast.node_views().filter_map(|v| v.as_match_case()).collect::<Vec<_>>();
    for (owner, (pattern, payload_kind, root_kind)) in cases.iter().zip([
        ("x = y", K::FormulaExpression, K::BuiltinPredicateApplication),
        ("x is object", K::FormulaExpression, K::IsAssertion),
        ("not (x = y)", K::FormulaExpression, K::PrefixFormula(P::Not)),
        ("((x = y))", K::FormulaExpression, K::ParenthesizedFormula),
        ("contradiction", K::FormulaExpression, K::FormulaConstant(mizar_syntax::SurfaceFormulaConstant::Contradiction)),
        ("for a being object holds a = x", K::FormulaExpression, K::QuantifiedFormula(Q::Universal)),
        ("P(x) & Q(x)", K::FormulaExpression, K::BinaryFormula(mizar_syntax::SurfaceFormulaBinaryOperator { connective: mizar_syntax::SurfaceFormulaConnective::And, repeated: false })),
        ("x", K::TermExpression, K::TermReference),
        ("P(x)", K::TermExpression, K::ApplicationTerm),
        ("x |-> x", K::TermExpression, K::BinderExpression),
        ("{x where x is set : x = x}", K::TermExpression, K::SetComprehension),
        ("f({x where x is set : x = x})", K::TermExpression, K::ApplicationTerm),
    ]) {
        let marker = format!("case {pattern} do");
        let start = text.find(&marker).unwrap();
        let body = if pattern == "x" { "assert x = y; return x;" } else { "return x;" };
        let spelling = format!("case {pattern} do {body} end;");
        assert_eq!(owner.range(), range(ast.source_id, start, start + spelling.len()));
        let children = owner.children().iter().map(|id| ast.node_view(*id).unwrap()).collect::<Vec<_>>();
        assert_eq!(children.len(), 6);
        assert_eq!(children.iter().map(|v| &text[v.range().start..v.range().end]).collect::<Vec<_>>(), ["case", pattern, "do", body, "end", ";"]);
        let payload = children[1];
        assert_eq!(payload.kind(), &payload_kind, "{pattern}");
        assert_eq!(payload.range(), range(ast.source_id, start + 5, start + 5 + pattern.len()));
        assert_eq!(payload.children().len(), 1);
        let root = ast.node_view(payload.children()[0]).unwrap();
        assert_eq!(root.kind(), &root_kind);
        assert_eq!(root.range(), payload.range());
        assert_eq!(children[2].as_token().unwrap().text.as_ref(), "do");
        assert_eq!(children[2].range(), range(ast.source_id, start + marker.len() - 2, start + marker.len()));
        assert_eq!(children[3].kind(), &K::AlgorithmStatementList);
        if root_kind == K::BuiltinPredicateApplication {
            assert_eq!(root.children().len(), 3);
            assert_eq!(ast.node_view(root.children()[0]).unwrap().kind(), &K::TermExpression);
            assert_eq!(ast.node_view(root.children()[1]).unwrap().as_token().unwrap().text.as_ref(), "=");
            assert_eq!(ast.node_view(root.children()[2]).unwrap().kind(), &K::TermExpression);
        }
        if matches!(root_kind, K::BinaryFormula(_)) {
            assert_eq!(root.children().len(), 3);
            assert_eq!(ast.node_view(root.children()[0]).unwrap().kind(), &K::InlinePredicateApplication);
            assert_eq!(ast.node_view(root.children()[1]).unwrap().as_token().unwrap().text.as_ref(), "&");
            assert_eq!(ast.node_view(root.children()[2]).unwrap().kind(), &K::InlinePredicateApplication);
        }
        if root_kind == K::PrefixFormula(P::Not) {
            assert_eq!(root.children().len(), 2);
            assert_eq!(ast.node_view(root.children()[1]).unwrap().kind(), &K::ParenthesizedFormula);
        }
        if root_kind == K::ParenthesizedFormula {
            assert_eq!(ast.node_views().filter(|v| v.kind() == &K::ParenthesizedFormula && v.range().start >= payload.range().start && v.range().end <= payload.range().end).count(), 2);
        }
        if root_kind == K::QuantifiedFormula(Q::Universal) {
            let segment = ast.node_view(root.children()[1]).unwrap();
            assert_eq!(segment.kind(), &K::QuantifierVariableSegment);
            assert_eq!(&text[segment.range().start..segment.range().end], "a being object");
        }
        if pattern.contains('{') {
            assert!(ast.node_views().any(|v| v.kind() == &K::FormulaExpression && v.range().start > payload.range().start && v.range().end < payload.range().end));
        }
    }
    let matching = ast.node_views().filter_map(|v| v.as_match_statement()).next().unwrap();
    let children = matching.children().iter().map(|id| ast.node_view(*id).unwrap()).collect::<Vec<_>>();
    assert_eq!(children.len(), 18);
    assert_eq!(children[..3].iter().map(|v| &text[v.range().start..v.range().end]).collect::<Vec<_>>(), ["match", "x", "do"]);
    assert_eq!(children[3..15].iter().map(|v| v.id()).collect::<Vec<_>>(), cases.iter().map(|v| v.id()).collect::<Vec<_>>());
    assert!(children[15].as_match_ending().is_some());
    assert_eq!(&text[children[15].range().start..children[15].range().end], "otherwise return y; end;");
    assert_eq!(children[15].children().iter().map(|id| { let v = ast.node_view(*id).unwrap(); &text[v.range().start..v.range().end] }).collect::<Vec<_>>(), ["otherwise", "return y;", "end", ";"]);
    let body = ast.node_views().find(|v| v.kind() == &K::AlgorithmStatementList && v.children().first() == Some(&matching.id())).unwrap();
    assert_eq!(body.children().len(), 2);
    let following = ast.node_view(body.children()[1]).unwrap();
    assert!(following.as_return_statement().is_some());
    assert_eq!(&text[following.range().start..following.range().end], "return x;");
    let final_start = text.find("set after_match_formulas = sentinel;").unwrap();
    assert!(ast.node_views().filter_map(|v| v.as_set_statement()).any(|v| v.range() == range(ast.source_id, final_start, text.trim_end().len())));
    let mut pending = vec![ast.node_views().find(|v| v.kind() == &K::CompilationUnit).unwrap().id()];
    let mut seen = std::collections::HashSet::new();
    let mut tokens = Vec::new();
    while let Some(id) = pending.pop() {
        assert!(seen.insert(id), "duplicate match source child");
        let node = ast.node_view(id).unwrap();
        if node.as_token().is_some() { tokens.push(id); }
        pending.extend(node.children().iter().rev().copied());
    }
    assert_eq!(tokens, ast.token_views().map(|v| v.id()).collect::<Vec<_>>());
    let frontend = task257c4c1_frontend_output(&text, 141);
    assert!(frontend.diagnostics.is_empty());
    assert_eq!(frontend.ast.as_ref().unwrap().snapshot_text_with_trivia(), ast.snapshot_text_with_trivia());
    let repeated = task257c4c1_frontend_output(&text, 141);
    assert_eq!(frontend.cache_keys, repeated.cache_keys);
    assert_eq!(frontend.ast, repeated.ast);
    let current = frontend.cache_keys.ast.as_ref().unwrap();
    assert_eq!(current.parser_version.version.as_ref(), "mizar-parser/surface-ast-v30");
    let mut retained = current.clone();
    retained.parser_version = mizar_frontend::parsing::ParserCacheKeyVersion::new("mizar-parser/surface-ast-v26");
    assert_eq!(current.token_stream_hash, retained.token_stream_hash);
    assert_eq!(current.parser_inputs_hash, retained.parser_inputs_hash);
    assert_ne!(current.stable_hash(), retained.stable_hash());
    let bytes = ast.canonical_bytes().unwrap();
    let restored = mizar_syntax::SurfaceAst::from_canonical_bytes(&bytes, ast.source_id).unwrap();
    assert_eq!(restored.snapshot_text_with_trivia(), ast.snapshot_text_with_trivia());
}

#[test]
fn match_explicit_formula_recovery_preserves_do_cases_otherwise_and_successors() {
    use mizar_frontend::orchestration::{DiagnosticClass, DiagnosticCode, DiagnosticLocation};
    use mizar_syntax::{SurfaceFormulaPrefixOperator as P, SurfaceNodeKind as K};
    let config = step5c11_config();
    let plan = build_test_plan(&config).unwrap();
    let case = plan.cases.iter().find(|case| case.id.0 == "fail_parser_match_explicit_formulas_001").unwrap();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let first = super::super::run_parse_only_case(&config.workspace_root, &config.workspace_root.join("tests"), case, 142);
    assert_eq!(first.status, super::super::ParseOnlyCaseStatus::Passed, "{first:?}");
    assert_eq!(first, super::super::run_parse_only_case(&config.workspace_root, &config.workspace_root.join("tests"), case, 142));
    let output = super::super::formula_statement::step5c8_test_frontend(&text);
    assert_eq!(output.diagnostics.len(), 4, "{:?}", output.diagnostics);
    let ast = output.ast.unwrap();
    for (diagnostic, (pattern, code, message, recovery_kind, root_kind)) in output.diagnostics.iter().zip([
        ("", "malformed_term_expression", "expected match case pattern", SyntaxRecoveryKind::MissingTerm, None),
        ("x =", "malformed_term_expression", "expected term after built-in predicate", SyntaxRecoveryKind::MissingTerm, Some(K::BuiltinPredicateApplication)),
        ("not", "malformed_formula_expression", "expected formula after `not`", SyntaxRecoveryKind::MissingFormula, Some(K::PrefixFormula(P::Not))),
        ("(x = y", "malformed_formula_expression", "expected `)` to close formula delimiter", SyntaxRecoveryKind::UnmatchedOpeningDelimiter, Some(K::ParenthesizedFormula)),
    ]) {
        let marker = if pattern.is_empty() { "case do".to_owned() } else { format!("case {pattern} do") };
        let start = text.find(&marker).unwrap();
        let anchor = start + marker.len() - 2;
        assert_eq!(diagnostic.class, DiagnosticClass::Syntax);
        assert_eq!(diagnostic.code, DiagnosticCode::Syntax(code.into()));
        assert_eq!(diagnostic.message.as_ref(), message);
        assert_eq!(diagnostic.location, DiagnosticLocation::SourceRange(range(ast.source_id, anchor, anchor + 2)));
        if pattern.starts_with('(') {
            assert_eq!(diagnostic.secondary, [mizar_session::SourceAnchor::Range(range(ast.source_id, start + 5, start + 6))]);
            assert_eq!(diagnostic.recovery_note.as_deref(), Some("insert `)` before continuing"));
        } else {
            assert!(diagnostic.secondary.is_empty());
            assert_eq!(diagnostic.recovery_note.as_deref(), Some(if code == "malformed_term_expression" { "repair the term expression before continuing" } else { "repair the formula expression before continuing" }));
        }
        let owner = ast.node_views().filter_map(|v| v.as_match_case()).find(|v| v.range().start == start).unwrap();
        assert_eq!(owner.range(), range(ast.source_id, start, start + marker.len() + " return x; end;".len()));
        let children = owner.children().iter().map(|id| ast.node_view(*id).unwrap()).collect::<Vec<_>>();
        assert_eq!(children.len(), 6);
        assert_eq!(children.iter().map(|v| &text[v.range().start..v.range().end]).collect::<Vec<_>>(), ["case", pattern, "do", "return x;", "end", ";"]);
        assert_eq!(children[2].range(), range(ast.source_id, anchor, anchor + 2));
        assert_eq!(children[2].as_token().unwrap().text.as_ref(), "do");
        let recovery = if let Some(root_kind) = root_kind {
            let payload = children[1];
            assert_eq!(payload.kind(), &K::FormulaExpression);
            assert_eq!(payload.range(), range(ast.source_id, start + 5, start + 5 + pattern.len()));
            assert_eq!(payload.children().len(), 1);
            let root = ast.node_view(payload.children()[0]).unwrap();
            assert_eq!(root.kind(), &root_kind);
            ast.node_view(*root.children().last().unwrap()).unwrap()
        } else {
            children[1]
        };
        assert_eq!(recovery.as_recovery(), Some(recovery_kind));
        assert_eq!(recovery.range(), range(ast.source_id, anchor, anchor));
        assert!(recovery.children().is_empty());
        assert_eq!(children[3].kind(), &K::AlgorithmStatementList);
        let returned = ast.node_view(children[3].children()[0]).unwrap();
        assert!(returned.as_return_statement().is_some());
        assert!(!returned.is_recovered());
    }
    for (kind, count) in [(SyntaxRecoveryKind::MissingTerm, 2), (SyntaxRecoveryKind::MissingFormula, 1), (SyntaxRecoveryKind::UnmatchedOpeningDelimiter, 1)] {
        assert_eq!(ast.node_views().filter(|v| v.as_recovery() == Some(kind)).count(), count);
    }
    assert_eq!(ast.node_views().filter_map(|v| v.as_recovery()).count(), 4);
    assert!(ast.trivia().skipped_token_ranges().is_empty());
    let matches = ast.node_views().filter_map(|v| v.as_match_statement()).collect::<Vec<_>>();
    assert_eq!(matches.len(), 4);
    assert_eq!(ast.node_views().filter_map(|v| v.as_match_case()).count(), 8);
    assert_eq!(ast.rowan_root().descendants().filter(|v| v.kind() == mizar_syntax::SyntaxKind::MatchCase).count(), 8);
    assert_eq!(ast.snapshot_text().matches("MatchCase").count(), 8);
    for matching in &matches {
        let children = matching.children().iter().map(|id| ast.node_view(*id).unwrap()).collect::<Vec<_>>();
        assert_eq!(children.len(), 8);
        assert!(children[3].as_match_case().is_some());
        let successor = children[4];
        assert!(successor.as_match_case().is_some());
        assert!(!successor.is_recovered());
        assert_eq!(&text[successor.range().start..successor.range().end], "case y = x do return y; end;");
        let payload = ast.node_view(successor.children()[1]).unwrap();
        assert_eq!(payload.kind(), &K::FormulaExpression);
        assert_eq!(ast.node_view(payload.children()[0]).unwrap().kind(), &K::BuiltinPredicateApplication);
        assert!(children[5].as_match_ending().is_some());
        assert!(!children[5].is_recovered());
        assert_eq!(&text[children[5].range().start..children[5].range().end], "otherwise return y; end;");
        assert_eq!(children[6..].iter().map(|v| &text[v.range().start..v.range().end]).collect::<Vec<_>>(), ["end", ";"]);
    }
    let body = ast.node_views().find(|v| v.kind() == &K::AlgorithmStatementList && v.children().first() == Some(&matches[0].id())).unwrap();
    assert_eq!(body.children().len(), 8);
    for (pair, matching) in body.children().chunks_exact(2).zip(matches) {
        assert_eq!(pair[0], matching.id());
        let successor = ast.node_view(pair[1]).unwrap();
        assert!(successor.as_return_statement().is_some());
        assert!(!successor.is_recovered());
        assert_eq!(&text[successor.range().start..successor.range().end], "return x;");
    }
    let final_start = text.find("set after_match_formula_failure = sentinel;").unwrap();
    assert!(ast.node_views().filter_map(|v| v.as_set_statement()).any(|v| v.range() == range(ast.source_id, final_start, text.trim_end().len()) && !v.is_recovered()));
    let mut pending = vec![ast.node_views().find(|v| v.kind() == &K::CompilationUnit).unwrap().id()];
    let mut seen = std::collections::HashSet::new();
    let mut tokens = Vec::new();
    while let Some(id) = pending.pop() {
        assert!(seen.insert(id), "duplicate recovered match source child {id:?}");
        let node = ast.node_view(id).unwrap();
        if node.as_token().is_some() { tokens.push(id); }
        pending.extend(node.children().iter().rev().copied());
    }
    assert_eq!(tokens, ast.token_views().map(|v| v.id()).collect::<Vec<_>>());
    let bytes = ast.canonical_bytes().unwrap();
    let restored = mizar_syntax::SurfaceAst::from_canonical_bytes(&bytes, ast.source_id).unwrap();
    assert_eq!(restored.snapshot_text_with_trivia(), ast.snapshot_text_with_trivia());
}

#[test]
fn match_formula_patterns_have_clean_source_but_no_bounded_semantic_admission() {
    use mizar_resolve::names::{SourceVariableScopeError, SourceVariableScopeResolver};
    use mizar_syntax::SurfaceNodeKind as K;

    let config = step5c11_config();
    let case = step5c14_state_case();
    let base = concat!(
        "definition\n  let x be object;\n",
        "  algorithm patterns(x) -> object\n  do\n",
        "    return x;\n  end;\nend;\n",
    );
    for (pattern, root_kind) in [
        ("", K::TermReference),
        ("x", K::TermReference),
        ("x = x", K::BuiltinPredicateApplication),
        ("not (x = x)", K::PrefixFormula(mizar_syntax::SurfaceFormulaPrefixOperator::Not)),
        ("x = x & x = x", K::BinaryFormula(mizar_syntax::SurfaceFormulaBinaryOperator {
            connective: mizar_syntax::SurfaceFormulaConnective::And,
            repeated: false,
        })),
    ] {
        let text = if pattern.is_empty() {
            base.to_owned()
        } else {
            base.replace("    return x;\n", &format!(
                "    match x do\n      case {pattern} do return x; end;\n      otherwise return x; end;\n    end;\n    return x;\n",
            ))
        };
        let frontend = super::super::formula_statement::step5c8_test_frontend(&text);
        assert!(frontend.diagnostics.is_empty(), "{pattern:?}: {:?}", frontend.diagnostics);
        let ast = frontend.ast.as_ref().expect("clean source must retain AST");
        assert!(ast.node_views().all(|node| !node.is_recovered() && node.as_recovery().is_none()));
        assert!(ast.trivia().skipped_token_ranges().is_empty());
        let body = ast.node_views().find(|node| node.kind() == &K::AlgorithmBody).unwrap();
        let body_parts = body.child_views().collect::<Vec<_>>();
        assert_eq!(body_parts.len(), 3);
        assert_eq!(body_parts[0].as_token().unwrap().text.as_ref(), "do");
        assert_eq!(body_parts[2].as_token().unwrap().text.as_ref(), "end");
        assert_eq!(body_parts[1].kind(), &K::AlgorithmStatementList);
        let statements = body_parts[1].child_views().collect::<Vec<_>>();
        assert_eq!(statements.len(), if pattern.is_empty() { 1 } else { 2 });
        let successor = *statements.last().unwrap();
        assert_eq!(successor.kind(), &K::ReturnStatement);
        assert_eq!(successor.range().start, text.rfind("return x;").unwrap());
        assert_eq!(&text[successor.range().start..successor.range().end], "return x;");
        assert_eq!(successor.child_views().map(|node| &text[node.range().start..node.range().end]).collect::<Vec<_>>(), ["return", "x", ";"]);
        if pattern.is_empty() {
            assert!(!ast.node_views().any(|node| matches!(node.kind(), K::MatchStatement | K::MatchCase | K::MatchEnding)));
        } else {
            let matched = statements[0];
            assert_eq!(matched.kind(), &K::MatchStatement);
            assert_eq!(matched.range().start, text.find("match x do").unwrap());
            let parts = matched.child_views().collect::<Vec<_>>();
            assert_eq!(parts.len(), 7);
            for (index, spelling) in [(0, "match"), (1, "x"), (2, "do"), (5, "end"), (6, ";")] {
                assert_eq!(&text[parts[index].range().start..parts[index].range().end], spelling);
            }
            assert_eq!(parts[1].kind(), &K::TermExpression);
            assert_eq!(parts[3].kind(), &K::MatchCase);
            assert_eq!(parts[4].kind(), &K::MatchEnding);
            assert_eq!(parts[6].range().end, matched.range().end);
            assert!(matched.range().end < successor.range().start);
            let branch = parts[3].child_views().collect::<Vec<_>>();
            assert_eq!(branch.len(), 6);
            assert_eq!(branch.iter().map(|node| &text[node.range().start..node.range().end]).collect::<Vec<_>>(), ["case", pattern, "do", "return x;", "end", ";"]);
            assert_eq!(branch[1].kind(), if pattern == "x" { &K::TermExpression } else { &K::FormulaExpression });
            assert_eq!(branch[1].range().start, text.find(&format!("case {pattern} do")).unwrap() + "case ".len());
            assert_eq!(branch[1].range().end, branch[1].range().start + pattern.len());
            assert_eq!(branch[1].children().len(), 1);
            assert_eq!(ast.node_view(branch[1].children()[0]).unwrap().kind(), &root_kind);
            assert_eq!(branch[5].range().end, parts[3].range().end);
            let otherwise = parts[4].child_views().collect::<Vec<_>>();
            assert_eq!(otherwise.len(), 4);
            assert_eq!(otherwise.iter().map(|node| &text[node.range().start..node.range().end]).collect::<Vec<_>>(), ["otherwise", "return x;", "end", ";"]);
            assert_eq!(otherwise[3].range().end, parts[4].range().end);
            for list in [branch[3], otherwise[1]] {
                assert_eq!(list.kind(), &K::AlgorithmStatementList);
                let returns = list.child_views().collect::<Vec<_>>();
                assert_eq!(returns.len(), 1);
                assert_eq!(returns[0].kind(), &K::ReturnStatement);
                assert_eq!(returns[0].range(), list.range());
                assert_eq!(returns[0].child_views().map(|node| &text[node.range().start..node.range().end]).collect::<Vec<_>>(), ["return", "x", ";"]);
            }
        }
        let (source, typed, symbols) = super::super::source_registration_inputs(
            &config.workspace_root, &case, frontend,
        ).unwrap();
        let scope = SourceVariableScopeResolver::resolve_algorithm_occurrences(&source, &symbols);
        let checked = mizar_checker::type_checker::check_source_algorithm_types(&source, &typed, &symbols);
        if pattern.is_empty() {
            scope.unwrap();
            let checked = checked.unwrap();
            let core = mizar_core::elaborator::lower_source_algorithms(&checked).unwrap();
            assert_eq!(core.algorithms().len(), 1);
        } else {
            assert_eq!(scope.unwrap_err(), SourceVariableScopeError::InvalidShape);
            assert_eq!(checked.unwrap_err(), "algorithms.unsupported_source_types");
        }
    }
}

#[test]
fn match_explicit_formula_quantifiers_preserve_binder_tails_and_branch_do() {
    use mizar_syntax::SurfaceNodeKind as K;
    for pattern in [
        "for a,b being object holds a = b",
        "for a be object holds a = x",
        "for a st a = x holds a = x",
        "for a holds a = x",
        "for a for b holds a = b",
        "for a ex b st a = b",
        "not (for a being object ex b being object st a = b)",
    ] {
        let text = format!("definition algorithm patterns(x) -> object do match x do case {pattern} do return x; end; end; return x; end; end;");
        let output = super::super::formula_statement::step5c8_test_frontend(&text);
        assert!(output.diagnostics.is_empty(), "{pattern}: {:?}", output.diagnostics);
        let ast = output.ast.unwrap();
        assert!(ast.node_views().all(|node| !node.is_recovered()));
        let case = ast.node_views().find(|node| node.kind() == &K::MatchCase).unwrap();
        let children = case.child_views().collect::<Vec<_>>();
        assert_eq!(children.len(), 6);
        assert_eq!(children[1].kind(), &K::FormulaExpression);
        assert_eq!(&text[children[1].range().start..children[1].range().end], pattern);
        assert_eq!(children[2].as_token().unwrap().text.as_ref(), "do");
        assert_eq!(children[2].range().start, text.find(&format!("{pattern} do")).unwrap() + pattern.len() + 1);
        let matching = ast.node_views().find(|node| node.kind() == &K::MatchStatement).unwrap();
        let body = ast.node_views().find(|node| node.kind() == &K::AlgorithmStatementList && node.children().first() == Some(&matching.id())).unwrap();
        assert_eq!(body.children().len(), 2);
        let successor = ast.node_view(body.children()[1]).unwrap();
        assert_eq!(successor.kind(), &K::ReturnStatement);
        assert_eq!(successor.range().start, text.rfind("return x;").unwrap());
    }
}

#[test]
fn match_boolean_operands_preserve_reference_children_and_existing_pattern_roots() {
    use mizar_syntax::{SurfaceFormulaBinaryOperator as B, SurfaceFormulaConnective as C, SurfaceFormulaPrefixOperator as P, SurfaceNodeKind as K, SyntaxKind as S};
    let config = step5c11_config();
    let plan = build_test_plan(&config).unwrap();
    let case = plan.cases.iter().find(|case| case.id.0 == "pass_parser_match_boolean_operands_001").unwrap();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let first = super::super::run_parse_only_case(&config.workspace_root, &config.workspace_root.join("tests"), case, 151);
    assert_eq!(first.status, super::super::ParseOnlyCaseStatus::Passed, "{first:?}");
    assert_eq!(first, super::super::run_parse_only_case(&config.workspace_root, &config.workspace_root.join("tests"), case, 151));
    let output = super::super::formula_statement::step5c8_test_frontend(&text);
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    let ast = output.ast.unwrap();
    assert!(ast.node_views().all(|v| !v.is_recovered() && v.as_recovery().is_none()));
    assert!(ast.trivia().skipped_token_ranges().is_empty());
    for (kind, syntax, count) in [(K::MatchStatement, S::MatchStatement, 1), (K::MatchCase, S::MatchCase, 20), (K::MatchEnding, S::MatchEnding, 1), (K::ReturnStatement, S::ReturnStatement, 22)] {
        assert_eq!(ast.node_views().filter(|v| v.kind() == &kind).count(), count);
        assert_eq!(ast.rowan_root().descendants().filter(|v| v.kind() == syntax).count(), count);
        assert_eq!(ast.snapshot_text().matches(&format!("{kind:?}")).count(), count);
    }
    let and = K::BinaryFormula(B { connective: C::And, repeated: false });
    let cases = ast.node_views().filter_map(|v| v.as_match_case()).collect::<Vec<_>>();
    for (owner, (pattern, payload_kind, root_kind, bare_count)) in cases.iter().zip([
        ("p & q", K::FormulaExpression, and.clone(), 2),
        ("not p", K::FormulaExpression, K::PrefixFormula(P::Not), 1),
        ("(p) & q", K::FormulaExpression, and.clone(), 2),
        ("p & (q or not r)", K::FormulaExpression, and.clone(), 3),
        ("p & p", K::FormulaExpression, and.clone(), 2),
        ("p & x = y", K::FormulaExpression, and.clone(), 1),
        ("P(x) & q", K::FormulaExpression, and.clone(), 1),
        ("(x) = y", K::FormulaExpression, K::BuiltinPredicateApplication, 0),
        ("((x)) = y", K::FormulaExpression, K::BuiltinPredicateApplication, 0),
        ("(x |-> x) = y", K::FormulaExpression, K::BuiltinPredicateApplication, 0),
        ("x |-> x = y", K::FormulaExpression, K::BuiltinPredicateApplication, 0),
        ("p", K::TermExpression, K::TermReference, 1),
        ("(p)", K::TermExpression, K::ParenthesizedTerm, 1),
        ("P(x)", K::TermExpression, K::ApplicationTerm, 0),
        ("x |-> x", K::TermExpression, K::BinderExpression, 0),
        ("(x,y) |-> x", K::TermExpression, K::BinderExpression, 0),
        ("p & (for z being object holds z = x)", K::FormulaExpression, and, 1),
        ("not local_ready", K::FormulaExpression, K::PrefixFormula(P::Not), 0),
        ("not (x,y local_pair)", K::FormulaExpression, K::PrefixFormula(P::Not), 0),
        ("not (x,y do not local_pair)", K::FormulaExpression, K::PrefixFormula(P::Not), 0),
    ]) {
        let marker = format!("case {pattern} do");
        let start = text.find(&marker).unwrap();
        let spelling = format!("{marker} return x; end;");
        assert_eq!(owner.range(), range(ast.source_id, start, start + spelling.len()));
        let children = owner.child_views().collect::<Vec<_>>();
        assert_eq!(children.len(), 6);
        assert_eq!(children.iter().map(|v| &text[v.range().start..v.range().end]).collect::<Vec<_>>(), ["case", pattern, "do", "return x;", "end", ";"]);
        let payload = children[1];
        assert_eq!(payload.kind(), &payload_kind, "{pattern}");
        assert_eq!(payload.range(), range(ast.source_id, start + 5, start + 5 + pattern.len()));
        assert_eq!(payload.children().len(), 1);
        let root = ast.node_view(payload.children()[0]).unwrap();
        assert_eq!(root.kind(), &root_kind, "{pattern}");
        assert_eq!(root.range(), payload.range());
        let bare = ast.node_views().filter(|v| v.kind() == &K::TermExpression && v.range().start >= payload.range().start && v.range().end <= payload.range().end && matches!(&text[v.range().start..v.range().end], "p" | "q" | "r")).collect::<Vec<_>>();
        assert_eq!(bare.len(), bare_count, "{pattern}");
        for operand in bare {
            assert_eq!(operand.children().len(), 1);
            let reference = ast.node_view(operand.children()[0]).unwrap();
            assert_eq!(reference.kind(), &K::TermReference);
            assert_eq!(reference.range(), operand.range());
            assert_eq!(reference.children().len(), 1);
            let token = ast.node_view(reference.children()[0]).unwrap();
            assert_eq!(token.range(), operand.range());
            assert_eq!(token.as_token().unwrap().text.as_ref(), &text[operand.range().start..operand.range().end]);
            let parent = ast.node_views().find(|v| v.children().contains(&operand.id())).unwrap();
            match pattern {
                "p" => assert_eq!(parent.kind(), &K::MatchCase),
                "(p)" => assert_eq!(parent.kind(), &K::ParenthesizedTerm),
                _ => assert!(matches!(parent.kind(), K::BinaryFormula(_) | K::PrefixFormula(_) | K::FormulaExpression), "{pattern}: {:?}", parent.kind()),
            }
        }
        if matches!(pattern, "p & q" | "p & p" | "p & x = y" | "P(x) & q" | "p & (for z being object holds z = x)") {
            let (left, right) = pattern.split_once(" & ").unwrap();
            assert_eq!(root.child_views().map(|v| &text[v.range().start..v.range().end]).collect::<Vec<_>>(), [left, "&", right]);
        }
        if pattern == "not p" {
            assert_eq!(root.child_views().map(|v| &text[v.range().start..v.range().end]).collect::<Vec<_>>(), ["not", "p"]);
        }
        if pattern == "(p) & q" {
            assert_eq!(root.child_views().map(|v| &text[v.range().start..v.range().end]).collect::<Vec<_>>(), ["(p)", "&", "q"]);
            let group = ast.node_view(root.children()[0]).unwrap();
            assert_eq!(group.kind(), &K::ParenthesizedFormula);
            assert_eq!(group.child_views().map(|v| &text[v.range().start..v.range().end]).collect::<Vec<_>>(), ["(", "p", ")"]);
            assert_eq!(ast.node_view(group.children()[1]).unwrap().kind(), &K::FormulaExpression);
        }
        if pattern == "p & (q or not r)" {
            assert_eq!(root.child_views().map(|v| &text[v.range().start..v.range().end]).collect::<Vec<_>>(), ["p", "&", "(q or not r)"]);
            let group = ast.node_view(root.children()[2]).unwrap();
            assert_eq!(group.kind(), &K::ParenthesizedFormula);
            let formula = ast.node_view(group.children()[1]).unwrap();
            assert_eq!(formula.kind(), &K::FormulaExpression);
            let nested = ast.node_view(formula.children()[0]).unwrap();
            assert_eq!(nested.kind(), &K::BinaryFormula(B { connective: C::Or, repeated: false }));
            assert_eq!(nested.child_views().map(|v| &text[v.range().start..v.range().end]).collect::<Vec<_>>(), ["q", "or", "not r"]);
            let prefix = ast.node_view(nested.children()[2]).unwrap();
            assert_eq!(prefix.kind(), &K::PrefixFormula(P::Not));
            assert_eq!(prefix.child_views().map(|v| &text[v.range().start..v.range().end]).collect::<Vec<_>>(), ["not", "r"]);
        }
        let heads = ast.node_views().filter(|v| v.kind() == &K::PredicateHead && v.range().start >= payload.range().start && v.range().end <= payload.range().end).collect::<Vec<_>>();
        assert_eq!(heads.len(), usize::from(pattern.contains("local_")), "{pattern}");
        if let Some(head) = heads.first() {
            assert_eq!(&text[head.range().start..head.range().end], if pattern == "not local_ready" { "local_ready" } else { "local_pair" });
            let segment = ast.node_views().find(|v| v.kind() == &K::PredicateSegment && v.children().contains(&head.id())).unwrap();
            assert_eq!(segment.child_views().map(|v| &text[v.range().start..v.range().end]).collect::<Vec<_>>(), match pattern {
                "not local_ready" => vec!["local_ready"],
                "not (x,y local_pair)" => vec!["x", ",", "y", "local_pair"],
                _ => vec!["x", ",", "y", "do", "not", "local_pair"],
            });
        }
        if pattern.ends_with("= y") && pattern.starts_with('(') {
            let left = ast.node_view(root.children()[0]).unwrap();
            assert_eq!(left.kind(), &K::TermExpression);
            assert_eq!(ast.node_view(left.children()[0]).unwrap().kind(), &K::ParenthesizedTerm);
        }
        if pattern == "x |-> x = y" {
            let left = ast.node_view(root.children()[0]).unwrap();
            assert_eq!(ast.node_view(left.children()[0]).unwrap().kind(), &K::BinderExpression);
        }
        if pattern == "p & x = y" || pattern == "P(x) & q" {
            assert_eq!(ast.node_view(root.children()[if pattern.starts_with('P') { 0 } else { 2 }]).unwrap().kind(), if pattern.starts_with('P') { &K::InlinePredicateApplication } else { &K::BuiltinPredicateApplication });
        }
        if pattern.contains("for z") {
            let quantified = ast.node_views().find(|v| matches!(v.kind(), K::QuantifiedFormula(_)) && v.range().start > payload.range().start && v.range().end < payload.range().end).unwrap();
            assert_eq!(&text[quantified.range().start..quantified.range().end], "for z being object holds z = x");
            let body = quantified.child_views().find(|v| v.kind() == &K::BuiltinPredicateApplication).unwrap();
            assert_eq!(&text[body.range().start..body.range().end], "z = x");
        }
    }
    let matching = ast.node_views().filter_map(|v| v.as_match_statement()).next().unwrap();
    let children = matching.child_views().collect::<Vec<_>>();
    assert_eq!(children.len(), 26);
    assert_eq!(children[..3].iter().map(|v| &text[v.range().start..v.range().end]).collect::<Vec<_>>(), ["match", "x", "do"]);
    assert_eq!(children[3..23].iter().map(|v| v.id()).collect::<Vec<_>>(), cases.iter().map(|v| v.id()).collect::<Vec<_>>());
    assert!(children[23].as_match_ending().is_some());
    assert_eq!(children[23].child_views().map(|v| &text[v.range().start..v.range().end]).collect::<Vec<_>>(), ["otherwise", "return y;", "end", ";"]);
    assert_eq!(children[24..].iter().map(|v| &text[v.range().start..v.range().end]).collect::<Vec<_>>(), ["end", ";"]);
    let body = ast.node_views().find(|v| v.kind() == &K::AlgorithmStatementList && v.children().first() == Some(&matching.id())).unwrap();
    assert_eq!(body.children().len(), 2);
    let following = ast.node_view(body.children()[1]).unwrap();
    assert!(following.as_return_statement().is_some());
    assert_eq!(&text[following.range().start..following.range().end], "return x;");
    let final_start = text.find("set after_match_boolean_operands = sentinel;").unwrap();
    assert!(ast.node_views().filter_map(|v| v.as_set_statement()).any(|v| v.range() == range(ast.source_id, final_start, text.trim_end().len())));
    let mut pending = vec![ast.node_views().find(|v| v.kind() == &K::CompilationUnit).unwrap().id()];
    let mut seen = std::collections::HashSet::new();
    let mut tokens = Vec::new();
    while let Some(id) = pending.pop() {
        assert!(seen.insert(id), "duplicate Boolean pattern source child {id:?}");
        let node = ast.node_view(id).unwrap();
        if node.as_token().is_some() { tokens.push(id); }
        pending.extend(node.children().iter().rev().copied());
    }
    assert_eq!(tokens, ast.token_views().map(|v| v.id()).collect::<Vec<_>>());
    let frontend = task257c4c1_frontend_output(&text, 151);
    assert!(frontend.diagnostics.is_empty());
    assert_eq!(frontend.ast.as_ref().unwrap().snapshot_text_with_trivia(), ast.snapshot_text_with_trivia());
    let repeated = task257c4c1_frontend_output(&text, 151);
    assert_eq!(frontend.cache_keys, repeated.cache_keys);
    assert_eq!(frontend.ast, repeated.ast);
    let token_key = &frontend.cache_keys.tokens;
    assert_eq!(token_key.version.as_ref(), "mizar-frontend/token-stream-cache-key/v6");
    let mut old_token_key = token_key.clone();
    old_token_key.version = "mizar-frontend/token-stream-cache-key/v3".into();
    assert_eq!(token_key.lexical_hash, old_token_key.lexical_hash);
    assert_eq!(token_key.active_lexical_environment, old_token_key.active_lexical_environment);
    assert_eq!(token_key.parser_context, old_token_key.parser_context);
    assert_eq!(token_key.parser_lexing_plan, old_token_key.parser_lexing_plan);
    assert_ne!(token_key.stable_hash(), old_token_key.stable_hash());
    let current = frontend.cache_keys.ast.as_ref().unwrap();
    assert_eq!(current.parser_version.version.as_ref(), "mizar-parser/surface-ast-v30");
    let mut retained = current.clone();
    retained.parser_version = mizar_frontend::parsing::ParserCacheKeyVersion::new("mizar-parser/surface-ast-v27");
    assert_eq!(current.token_stream_hash, retained.token_stream_hash);
    assert_eq!(current.parser_inputs_hash, retained.parser_inputs_hash);
    assert_ne!(current.stable_hash(), retained.stable_hash());
    let bytes = ast.canonical_bytes().unwrap();
    let restored = mizar_syntax::SurfaceAst::from_canonical_bytes(&bytes, ast.source_id).unwrap();
    assert_eq!(restored.snapshot_text_with_trivia(), ast.snapshot_text_with_trivia());
}

#[test]
fn match_boolean_operand_recovery_preserves_do_and_all_successors() {
    use mizar_frontend::orchestration::{DiagnosticClass, DiagnosticCode, DiagnosticLocation};
    use mizar_syntax::{SurfaceFormulaBinaryOperator as B, SurfaceFormulaConnective as C, SurfaceFormulaPrefixOperator as P, SurfaceNodeKind as K};
    let config = step5c11_config();
    let plan = build_test_plan(&config).unwrap();
    let case = plan.cases.iter().find(|case| case.id.0 == "fail_parser_match_boolean_operands_001").unwrap();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let first = super::super::run_parse_only_case(&config.workspace_root, &config.workspace_root.join("tests"), case, 152);
    assert_eq!(first.status, super::super::ParseOnlyCaseStatus::Passed, "{first:?}");
    assert_eq!(first, super::super::run_parse_only_case(&config.workspace_root, &config.workspace_root.join("tests"), case, 152));
    let output = super::super::formula_statement::step5c8_test_frontend(&text);
    assert_eq!(output.diagnostics.len(), 3, "{:?}", output.diagnostics);
    let ast = output.ast.unwrap();
    for (diagnostic, (pattern, message, recovery_kind, root_kind)) in output.diagnostics.iter().zip([
        ("p &", "expected formula after connective", SyntaxRecoveryKind::MissingFormula, K::BinaryFormula(B { connective: C::And, repeated: false })),
        ("not", "expected formula after `not`", SyntaxRecoveryKind::MissingFormula, K::PrefixFormula(P::Not)),
        ("(p & q", "expected `)` to close formula delimiter", SyntaxRecoveryKind::UnmatchedOpeningDelimiter, K::ParenthesizedFormula),
    ]) {
        let marker = format!("case {pattern} do");
        let start = text.find(&marker).unwrap();
        let anchor = start + marker.len() - 2;
        assert_eq!(diagnostic.class, DiagnosticClass::Syntax);
        assert_eq!(diagnostic.code, DiagnosticCode::Syntax("malformed_formula_expression".into()));
        assert_eq!(diagnostic.message.as_ref(), message);
        assert_eq!(diagnostic.location, DiagnosticLocation::SourceRange(range(ast.source_id, anchor, anchor + 2)));
        if pattern.starts_with('(') {
            assert_eq!(diagnostic.secondary, [mizar_session::SourceAnchor::Range(range(ast.source_id, start + 5, start + 6))]);
            assert_eq!(diagnostic.recovery_note.as_deref(), Some("insert `)` before continuing"));
        } else {
            assert!(diagnostic.secondary.is_empty());
            assert_eq!(diagnostic.recovery_note.as_deref(), Some("repair the formula expression before continuing"));
        }
        let owner = ast.node_views().filter_map(|v| v.as_match_case()).find(|v| v.range().start == start).unwrap();
        assert_eq!(owner.range(), range(ast.source_id, start, start + marker.len() + " return x; end;".len()));
        let children = owner.child_views().collect::<Vec<_>>();
        assert_eq!(children.len(), 6);
        assert_eq!(children.iter().map(|v| &text[v.range().start..v.range().end]).collect::<Vec<_>>(), ["case", pattern, "do", "return x;", "end", ";"]);
        assert_eq!(children[2].range(), range(ast.source_id, anchor, anchor + 2));
        assert_eq!(children[2].as_token().unwrap().text.as_ref(), "do");
        let payload = children[1];
        assert_eq!(payload.kind(), &K::FormulaExpression);
        assert_eq!(payload.range(), range(ast.source_id, start + 5, start + 5 + pattern.len()));
        assert_eq!(payload.children().len(), 1);
        let root = ast.node_view(payload.children()[0]).unwrap();
        assert_eq!(root.kind(), &root_kind);
        let recovery = ast.node_view(*root.children().last().unwrap()).unwrap();
        assert_eq!(recovery.as_recovery(), Some(recovery_kind));
        assert_eq!(recovery.range(), range(ast.source_id, anchor, anchor));
        assert!(recovery.children().is_empty());
        assert_eq!(children[3].kind(), &K::AlgorithmStatementList);
        let returned = ast.node_view(children[3].children()[0]).unwrap();
        assert!(returned.as_return_statement().is_some());
        assert!(!returned.is_recovered());
    }
    assert_eq!(ast.node_views().filter(|v| v.as_recovery() == Some(SyntaxRecoveryKind::MissingFormula)).count(), 2);
    assert_eq!(ast.node_views().filter(|v| v.as_recovery() == Some(SyntaxRecoveryKind::UnmatchedOpeningDelimiter)).count(), 1);
    assert_eq!(ast.node_views().filter_map(|v| v.as_recovery()).count(), 3);
    assert!(ast.trivia().skipped_token_ranges().is_empty());
    let matches = ast.node_views().filter_map(|v| v.as_match_statement()).collect::<Vec<_>>();
    assert_eq!(matches.len(), 3);
    assert_eq!(ast.node_views().filter_map(|v| v.as_match_case()).count(), 6);
    assert_eq!(ast.rowan_root().descendants().filter(|v| v.kind() == mizar_syntax::SyntaxKind::MatchCase).count(), 6);
    assert_eq!(ast.snapshot_text().matches("MatchCase").count(), 6);
    for matching in &matches {
        let children = matching.child_views().collect::<Vec<_>>();
        assert_eq!(children.len(), 8);
        assert!(children[3].as_match_case().is_some());
        let successor = children[4];
        assert!(successor.as_match_case().is_some());
        assert!(!successor.is_recovered());
        assert_eq!(&text[successor.range().start..successor.range().end], "case y = x do return y; end;");
        let payload = ast.node_view(successor.children()[1]).unwrap();
        assert_eq!(payload.kind(), &K::FormulaExpression);
        assert_eq!(ast.node_view(payload.children()[0]).unwrap().kind(), &K::BuiltinPredicateApplication);
        assert!(children[5].as_match_ending().is_some());
        assert!(!children[5].is_recovered());
        assert_eq!(children[5].child_views().map(|v| &text[v.range().start..v.range().end]).collect::<Vec<_>>(), ["otherwise", "return y;", "end", ";"]);
        assert_eq!(children[6..].iter().map(|v| &text[v.range().start..v.range().end]).collect::<Vec<_>>(), ["end", ";"]);
    }
    let body = ast.node_views().find(|v| v.kind() == &K::AlgorithmStatementList && v.children().first() == Some(&matches[0].id())).unwrap();
    assert_eq!(body.children().len(), 6);
    for (pair, matching) in body.children().chunks_exact(2).zip(matches) {
        assert_eq!(pair[0], matching.id());
        let successor = ast.node_view(pair[1]).unwrap();
        assert!(successor.as_return_statement().is_some());
        assert!(!successor.is_recovered());
        assert_eq!(&text[successor.range().start..successor.range().end], "return x;");
    }
    let final_start = text.find("set after_match_boolean_failure = sentinel;").unwrap();
    assert!(ast.node_views().filter_map(|v| v.as_set_statement()).any(|v| v.range() == range(ast.source_id, final_start, text.trim_end().len()) && !v.is_recovered()));
    let mut pending = vec![ast.node_views().find(|v| v.kind() == &K::CompilationUnit).unwrap().id()];
    let mut seen = std::collections::HashSet::new();
    let mut tokens = Vec::new();
    while let Some(id) = pending.pop() {
        assert!(seen.insert(id), "duplicate recovered Boolean pattern source child {id:?}");
        let node = ast.node_view(id).unwrap();
        if node.as_token().is_some() { tokens.push(id); }
        pending.extend(node.children().iter().rev().copied());
    }
    assert_eq!(tokens, ast.token_views().map(|v| v.id()).collect::<Vec<_>>());
    let bytes = ast.canonical_bytes().unwrap();
    let restored = mizar_syntax::SurfaceAst::from_canonical_bytes(&bytes, ast.source_id).unwrap();
    assert_eq!(restored.snapshot_text_with_trivia(), ast.snapshot_text_with_trivia());
}

#[test]
fn match_boolean_operands_have_clean_source_but_no_bounded_semantic_admission() {
    use mizar_resolve::names::{SourceVariableScopeError, SourceVariableScopeResolver};
    use mizar_syntax::SurfaceNodeKind as K;
    let config = step5c11_config();
    let case = step5c14_state_case();
    let base = "definition\n  let x be object;\n  algorithm patterns(x) -> object\n  do\n    return x;\n  end;\nend;\n";
    for pattern in ["", "x", "p & q", "not p", "(p) & q", "p & p"] {
        let text = if pattern.is_empty() { base.to_owned() } else {
            base.replace("    return x;\n", &format!("    match x do\n      case {pattern} do return x; end;\n      otherwise return x; end;\n    end;\n    return x;\n"))
        };
        let frontend = super::super::formula_statement::step5c8_test_frontend(&text);
        assert!(frontend.diagnostics.is_empty(), "{pattern:?}: {:?}", frontend.diagnostics);
        let ast = frontend.ast.as_ref().unwrap();
        assert!(ast.node_views().all(|node| !node.is_recovered() && node.as_recovery().is_none()));
        assert!(ast.trivia().skipped_token_ranges().is_empty());
        let body = ast.node_views().find(|node| node.kind() == &K::AlgorithmBody).unwrap();
        let body = body.child_views().collect::<Vec<_>>();
        assert_eq!(body.len(), 3); assert_eq!(body[1].kind(), &K::AlgorithmStatementList);
        assert_eq!(body[0].as_token().unwrap().text.as_ref(), "do");
        assert_eq!(body[2].as_token().unwrap().text.as_ref(), "end");
        let statements = body[1].child_views().collect::<Vec<_>>();
        assert_eq!(statements.len(), if pattern.is_empty() { 1 } else { 2 });
        let successor = *statements.last().unwrap();
        assert_eq!(successor.kind(), &K::ReturnStatement);
        assert_eq!(successor.range().start, text.rfind("return x;").unwrap());
        assert_eq!(&text[successor.range().start..successor.range().end], "return x;");
        if pattern.is_empty() {
            assert!(!ast.node_views().any(|node| matches!(node.kind(), K::MatchStatement | K::MatchCase | K::MatchEnding)));
        } else {
            let matched = statements[0];
            assert_eq!(matched.kind(), &K::MatchStatement);
            let parts = matched.child_views().collect::<Vec<_>>();
            assert_eq!(parts.len(), 7);
            assert_eq!(parts.iter().map(|node| &text[node.range().start..node.range().end]).collect::<Vec<_>>(), ["match", "x", "do", &format!("case {pattern} do return x; end;"), "otherwise return x; end;", "end", ";"]);
            assert_eq!(parts[3].kind(), &K::MatchCase); assert_eq!(parts[4].kind(), &K::MatchEnding);
            assert_eq!(parts[6].range().end, matched.range().end);
            assert!(matched.range().end < successor.range().start);
            let branch = parts[3].child_views().collect::<Vec<_>>();
            assert_eq!(branch.len(), 6);
            assert_eq!(branch.iter().map(|node| &text[node.range().start..node.range().end]).collect::<Vec<_>>(), ["case", pattern, "do", "return x;", "end", ";"]);
            let payload = branch[1];
            let start = text.find(&format!("case {pattern} do")).unwrap() + "case ".len();
            assert_eq!((payload.range().start, payload.range().end), (start, start + pattern.len()));
            assert_eq!(payload.kind(), if pattern == "x" { &K::TermExpression } else { &K::FormulaExpression });
            assert_eq!(payload.children().len(), 1);
            let root = ast.node_view(payload.children()[0]).unwrap();
            if pattern == "x" { assert_eq!(root.kind(), &K::TermReference); } else {
                assert!(!ast.node_views().any(|node| matches!(node.kind(), K::PredicateApplication | K::InlinePredicateApplication)));
                if pattern == "not p" {
                    assert_eq!(root.kind(), &K::PrefixFormula(mizar_syntax::SurfaceFormulaPrefixOperator::Not));
                    assert_eq!(root.children().len(), 2);
                } else {
                    assert_eq!(root.kind(), &K::BinaryFormula(mizar_syntax::SurfaceFormulaBinaryOperator { connective: mizar_syntax::SurfaceFormulaConnective::And, repeated: false }));
                    assert_eq!(root.children().len(), 3);
                }
                let operands = ast.node_views().filter(|node| node.kind() == &K::TermExpression && node.range().start >= start && node.range().end <= start + pattern.len()).collect::<Vec<_>>();
                let expected = pattern.char_indices().filter(|(_, ch)| matches!(*ch, 'p' | 'q')).collect::<Vec<_>>();
                assert_eq!(operands.len(), expected.len());
                for (operand, (offset, ch)) in operands.iter().zip(expected) {
                    assert_eq!((operand.range().start, operand.range().end), (start + offset, start + offset + 1));
                    assert_eq!(operand.children().len(), 1);
                    let reference = ast.node_view(operand.children()[0]).unwrap();
                    assert_eq!(reference.kind(), &K::TermReference);
                    assert_eq!(reference.range(), operand.range());
                    assert_eq!(reference.children().len(), 1);
                    let token = ast.node_view(reference.children()[0]).unwrap();
                    assert_eq!(token.range(), reference.range());
                    assert_eq!(token.as_token().unwrap().text.as_ref(), ch.to_string());
                }
                assert_eq!(root.children().last(), Some(&operands.last().unwrap().id()));
                if pattern == "p & q" || pattern == "p & p" { assert_eq!(root.children()[0], operands[0].id()); }
                if pattern == "(p) & q" {
                    let grouped = ast.node_view(root.children()[0]).unwrap();
                    assert_eq!(grouped.kind(), &K::ParenthesizedFormula);
                    assert_eq!(grouped.child_views().map(|node| &text[node.range().start..node.range().end]).collect::<Vec<_>>(), ["(", "p", ")"]);
                    let inner = ast.node_view(grouped.children()[1]).unwrap();
                    assert_eq!(inner.kind(), &K::FormulaExpression);
                    assert_eq!(inner.children(), &[operands[0].id()]);
                }
            }
            let otherwise = parts[4].child_views().collect::<Vec<_>>();
            assert_eq!(otherwise.iter().map(|node| &text[node.range().start..node.range().end]).collect::<Vec<_>>(), ["otherwise", "return x;", "end", ";"]);
            for list in [branch[3], otherwise[1]] {
                assert_eq!(list.kind(), &K::AlgorithmStatementList);
                let returns = list.child_views().collect::<Vec<_>>();
                assert_eq!(returns.len(), 1); assert_eq!(returns[0].kind(), &K::ReturnStatement);
                assert_eq!(returns[0].range(), list.range());
            }
        }
        let (source, typed, symbols) = super::super::source_registration_inputs(&config.workspace_root, &case, frontend).unwrap();
        let scope = SourceVariableScopeResolver::resolve_algorithm_occurrences(&source, &symbols);
        let checked = mizar_checker::type_checker::check_source_algorithm_types(&source, &typed, &symbols);
        if pattern.is_empty() {
            scope.unwrap(); let checked = checked.unwrap();
            assert_eq!(mizar_core::elaborator::lower_source_algorithms(&checked).unwrap().algorithms().len(), 1);
        } else {
            assert_eq!(scope.unwrap_err(), SourceVariableScopeError::InvalidShape);
            assert_eq!(checked.unwrap_err(), "algorithms.unsupported_source_types");
        }
    }
}

#[test]
fn match_boolean_operands_preserve_predicate_negation_scope_frames() {
    use mizar_syntax::SurfaceNodeKind as K;
    for predicate in ["x,y local_pair", "x,y do not local_pair"] {
        let text = format!("definition let x,y be object; pred PairDef: x,y local_pair means x = y; algorithm patterns(x,y) -> object do match x do case not ({predicate}) do return x; end; end; return x; end; end;");
        let output = super::super::formula_statement::step5c8_test_frontend(&text);
        assert!(output.diagnostics.is_empty(), "{predicate}: {:?}", output.diagnostics);
        let ast = output.ast.unwrap();
        assert!(ast.node_views().all(|v| !v.is_recovered()));
        let case = ast.node_views().find(|v| v.kind() == &K::MatchCase).unwrap();
        assert_eq!(case.child_views().nth(2).unwrap().as_token().unwrap().text.as_ref(), "do");
        let segment = ast.node_views().find(|v| v.kind() == &K::PredicateSegment).unwrap();
        assert_eq!(&text[segment.range().start..segment.range().end], predicate);
        assert_eq!(segment.child_views().filter_map(|v| v.as_token()).map(|t| t.text.as_ref()).collect::<Vec<_>>(), if predicate.contains("do not") { vec![",", "do", "not"] } else { vec![","] });
    }
}
