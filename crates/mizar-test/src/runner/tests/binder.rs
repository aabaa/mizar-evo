use super::*;

#[test]
fn binder_roots_preserve_order_precedence_and_grouped_hosts() {
    use mizar_syntax::{SurfaceNodeKind as K, SyntaxKind};
    let plan = step5c11_plan();
    for (id, expected_count) in [
        ("pass_parser_binder_roots_001", 14),
        ("pass_parser_binder_grouped_hosts_001", 9),
    ] {
        let case = plan.cases.iter().find(|case| case.id.0 == id).unwrap();
        let text = std::fs::read_to_string(&case.source_path).unwrap();
        let output = super::super::formula_statement::step5c8_test_frontend(&text);
        assert!(
            output.diagnostics.is_empty(),
            "{id}: {:?}",
            output.diagnostics
        );
        let ast = output.ast.unwrap();
        let binders = ast
            .node_views()
            .filter(|v| format!("{:?}", v.kind()) == "BinderExpression")
            .collect::<Vec<_>>();
        assert_eq!(binders.len(), expected_count);
        assert!(ast.node_views().all(|v| !v.is_recovered()));
        assert!(ast.trivia().skipped_token_ranges().is_empty());
        let mut arrows = ast
            .token_views()
            .filter(|v| v.as_token().unwrap().text.as_ref() == "|->")
            .map(|v| {
                assert_eq!(v.as_token().unwrap().kind, SurfaceTokenKind::ReservedSymbol);
                v.range()
            })
            .collect::<Vec<_>>();
        arrows.sort_by_key(|r| r.start);
        assert_eq!(
            arrows,
            text.match_indices("|->")
                .map(|(start, _)| range(ast.source_id, start, start + 3))
                .collect::<Vec<_>>()
        );
        for binder in binders {
            assert_eq!(binder.syntax_kind(), SyntaxKind::from_raw(195));
            let children = binder.children();
            let arrow = ast.node_view(children[children.len() - 2]).unwrap();
            let body = ast.node_view(*children.last().unwrap()).unwrap();
            assert_eq!(arrow.as_token().unwrap().text.as_ref(), "|->");
            assert_eq!(body.kind(), &K::TermExpression);
            assert_eq!(
                binder.range(),
                range(
                    ast.source_id,
                    ast.node_view(children[0]).unwrap().range().start,
                    body.range().end
                )
            );
            assert_eq!(
                children[..children.len() - 2]
                    .iter()
                    .map(|child| ast.node_view(*child).unwrap().as_token().unwrap())
                    .map(|token| token.text.as_ref())
                    .collect::<String>(),
                text[binder.range().start..arrow.range().start]
                    .chars()
                    .filter(|c| !c.is_whitespace())
                    .collect::<String>()
            );
        }
        let rhs = |name: &str| {
            let equating = ast
                .node_views()
                .filter_map(|v| v.as_equating())
                .find(|v| text[v.range().start..].starts_with(format!("{name} =").as_str()))
                .unwrap();
            let term = ast.node_view(*equating.children().last().unwrap()).unwrap();
            ast.node_view(term.children()[0]).unwrap()
        };
        if id == "pass_parser_binder_roots_001" {
            for (name, body_kind) in [
                ("right_nested", "BinderExpression"),
                ("body_qua", "QuaExpression"),
                ("body_restriction", "InfixExpression"),
            ] {
                let binder = rhs(name);
                let body = ast.node_view(*binder.children().last().unwrap()).unwrap();
                let payload = ast.node_view(body.children()[0]).unwrap();
                assert!(format!("{:?}", payload.kind()).starts_with(body_kind));
                if name == "right_nested" {
                    assert_eq!(
                        ast.node_view(payload.children()[0])
                            .unwrap()
                            .as_token()
                            .unwrap()
                            .text
                            .as_ref(),
                        "y"
                    );
                }
            }
            for (name, kind, operand) in [
                ("whole_qua", K::QuaExpression, 0),
                ("application", K::SelectorAccess, 0),
            ] {
                let root = rhs(name);
                assert_eq!(root.kind(), &kind);
                assert_eq!(
                    ast.node_view(root.children()[operand]).unwrap().kind(),
                    &K::ParenthesizedTerm
                );
            }
            let target = ast
                .node_view(*rhs("whole_qua").children().last().unwrap())
                .unwrap();
            assert_eq!(
                &text[target.range().start..target.range().end],
                "expr of set"
            );
            for (name, grouped_operand) in [("whole_restriction", 0), ("grouped_operand", 2)] {
                let root = rhs(name);
                let operator = root.as_infix_expression().unwrap();
                assert_eq!(operator.spelling.as_ref(), "|");
                assert_eq!(operator.precedence, 64);
                assert_eq!(
                    operator.associativity,
                    mizar_syntax::SurfaceOperatorAssociativity::NonAssociative
                );
                assert_eq!(
                    ast.node_view(root.children()[grouped_operand])
                        .unwrap()
                        .kind(),
                    &K::ParenthesizedTerm
                );
            }
            assert!(ast.token_views().any(|v| {
                v.as_token().is_some_and(|token| {
                    token.text.as_ref() == "|" && token.kind == SurfaceTokenKind::UserSymbol
                })
            }));
        } else {
            for name in ["call", "call_before", "nested_call"] {
                let call = rhs(name);
                assert_eq!(call.kind(), &K::ApplicationTerm);
                let arguments = call
                    .children()
                    .iter()
                    .filter(|child| ast.node_view(**child).unwrap().kind() == &K::TermExpression)
                    .count();
                assert_eq!(arguments, 2);
            }
            for name in ["GroupedEquality", "GroupedPredicate"] {
                let theorem = ast
                    .node_views()
                    .filter_map(|v| v.as_theorem_item())
                    .find(|v| {
                        text[v.range().start..v.range().end]
                            .starts_with(format!("theorem {name}:").as_str())
                    })
                    .unwrap();
                assert_eq!(
                    ast.node_views()
                        .filter(|v| format!("{:?}", v.kind()) == "BinderExpression"
                            && v.range().start > theorem.range().start
                            && v.range().end < theorem.range().end)
                        .count(),
                    2
                );
            }
        }
    }
}

#[test]
fn binder_roots_recovery_cache_storage_and_source_provenance() {
    use mizar_frontend::orchestration::{DiagnosticCode, DiagnosticLocation, FrontendOutput};
    use mizar_syntax::SurfaceNodeKind as K;
    let plan = step5c11_plan();
    for (ordinal, (id, boundaries)) in [
        ("pass_parser_binder_roots_001", vec![]),
        ("pass_parser_binder_grouped_hosts_001", vec![]),
        (
            "fail_parser_binder_heads_001",
            vec![
                ("() |->", 1),
                ("(x,x) |->", 1),
                ("(x,y,) |->", 1),
                ("(x being object,y) |->", 1),
                ("(x,(y,z)) |->", 1),
                ("(x) |->", 1),
                ("|-> x;\nset after_unparenthesized", 3),
                ("|-> x;\nset after_unparenthesized", 3),
            ],
        ),
        (
            "fail_parser_binder_missing_body_001",
            vec![
                (";\nset after_semicolon_body", 1),
                (");\nset after_grouped_body", 1),
            ],
        ),
        (
            "fail_parser_binder_operator_operands_001",
            vec![
                ("|-> x;\nset after_restriction", 3),
                ("|-> x;\nset after_selector", 3),
            ],
        ),
    ]
    .iter()
    .enumerate()
    {
        let case = plan.cases.iter().find(|case| case.id.0 == *id).unwrap();
        let text = std::fs::read_to_string(&case.source_path).unwrap();
        let checked = super::super::formula_statement::step5c8_test_frontend(&text);
        assert_eq!(
            checked.diagnostics.len(),
            boundaries.len(),
            "{id}: {:?}",
            checked.diagnostics
        );
        let ast = checked.ast.unwrap();
        for (diagnostic, (boundary, width)) in checked.diagnostics.iter().zip(boundaries) {
            let start = text.find(*boundary).unwrap();
            assert_eq!(
                diagnostic.code,
                DiagnosticCode::Syntax("malformed_term_expression".into())
            );
            assert_eq!(
                diagnostic.location,
                DiagnosticLocation::SourceRange(range(ast.source_id, start, start + width))
            );
            assert!(diagnostic.secondary.is_empty());
        }
        for (start, _) in text.match_indices("set after_") {
            let end = start + text[start..].find(';').unwrap() + 1;
            let sentinel = ast
                .node_views()
                .filter_map(|v| v.as_set_statement())
                .find(|v| v.range().start == start)
                .unwrap();
            assert_eq!(sentinel.range(), range(ast.source_id, start, end));
            assert!(!sentinel.is_recovered());
            let delimiter = ast.node_view(*sentinel.children().last().unwrap()).unwrap();
            assert_eq!(delimiter.as_token().unwrap().text.as_ref(), ";");
        }
        if ordinal == 2 {
            let mut recovered_binders = ast
                .node_views()
                .filter(|v| format!("{:?}", v.kind()) == "BinderExpression")
                .collect::<Vec<_>>();
            recovered_binders.sort_by_key(|v| v.range().start);
            assert_eq!(recovered_binders.len(), 6);
            for (binder, head) in recovered_binders.iter().zip([
                "()",
                "(x,x)",
                "(x,y,)",
                "(x being object,y)",
                "(x,(y,z))",
                "(x)",
            ]) {
                let start = text.find(format!("{head} |->").as_str()).unwrap();
                let head_range = range(ast.source_id, start, start + head.len());
                assert!(binder.is_recovered());
                assert_eq!(binder.children().len(), 3);
                let recovery = ast.node_view(binder.children()[0]).unwrap();
                assert_eq!(
                    recovery.as_recovery(),
                    Some(SyntaxRecoveryKind::SkippedToken)
                );
                assert_eq!(recovery.range(), head_range);
                let retained = recovery
                    .children()
                    .iter()
                    .filter_map(|child| ast.node_view(*child).unwrap().as_token())
                    .map(|token| token.text.as_ref())
                    .collect::<String>();
                assert_eq!(
                    retained,
                    head.chars()
                        .filter(|c| !c.is_whitespace())
                        .collect::<String>()
                );
                let arrow = ast.node_view(binder.children()[1]).unwrap();
                assert_eq!(arrow.as_token().unwrap().text.as_ref(), "|->");
                assert_eq!(arrow.range().start, head_range.end + 1);
                assert_eq!(
                    ast.node_view(binder.children()[2]).unwrap().kind(),
                    &K::TermExpression
                );
                assert!(
                    ast.trivia()
                        .skipped_token_ranges()
                        .iter()
                        .any(|skip| skip.range == head_range)
                );
            }
            assert_eq!(ast.trivia().skipped_token_ranges().len(), 7);
        } else if ordinal == 3 {
            let binders = ast
                .node_views()
                .filter(|v| format!("{:?}", v.kind()) == "BinderExpression")
                .collect::<Vec<_>>();
            assert_eq!(binders.len(), 2);
            for binder in binders {
                let arrow = ast.node_view(binder.children()[1]).unwrap();
                let missing = ast.node_view(binder.children()[2]).unwrap();
                assert!(binder.is_recovered());
                assert_eq!(missing.as_recovery(), Some(SyntaxRecoveryKind::MissingTerm));
                assert_eq!(missing.range().start, missing.range().end);
                assert_eq!(binder.range().end, arrow.range().end);
                assert!(missing.range().start > binder.range().end);
                assert!(
                    ast.token_views()
                        .any(|v| v.range().start == missing.range().start
                            && matches!(v.as_token().unwrap().text.as_ref(), ";" | ")"))
                );
                let wrapper = ast
                    .node_views()
                    .find(|v| v.kind() == &K::TermExpression && v.children() == [binder.id()])
                    .unwrap();
                assert_eq!(wrapper.range(), binder.range());
            }
            assert!(ast.trivia().skipped_token_ranges().is_empty());
        }
        assert!(
            ast.node_views()
                .all(|v| v.range().source_id == ast.source_id)
        );
        let output = task257c4c1_frontend_output(&text, 100 + ordinal);
        let replay = task257c4c1_frontend_output(&text, 100 + ordinal);
        assert_eq!(output.cache_keys, replay.cache_keys);
        assert_eq!(output.ast, replay.ast);
        assert_eq!(output.diagnostics, replay.diagnostics);
        let source = &output.source;
        let input = mizar_session::SourceInput {
            package_id: source.package_id.clone(),
            module_path: source.module_path.clone(),
            normalized_path: source.normalized_path.clone(),
            edition: source.edition.clone(),
            origin: mizar_session::SourceOriginInput::Disk {
                path: source.file_path.clone(),
            },
        };
        let bytes = output.canonical_disk_bytes().unwrap();
        let same =
            FrontendOutput::from_canonical_disk_bytes(&bytes, source.source_id, &input).unwrap();
        assert_eq!(same, output);
        assert_eq!(same.canonical_disk_bytes().unwrap(), bytes);
        assert!(
            same.ast
                .as_ref()
                .unwrap()
                .node_views()
                .all(|v| v.range().source_id == source.source_id)
        );
        assert_eq!(
            output
                .cache_keys
                .ast
                .as_ref()
                .unwrap()
                .parser_version
                .version
                .as_ref(),
            "mizar-parser/surface-ast-v30"
        );
    }
    let checked = super::super::formula_statement::step5c8_test_frontend("set f = x |-> y |-> ;");
    assert_eq!(checked.diagnostics.len(), 1);
    let ast = checked.ast.unwrap();
    let binders = ast
        .node_views()
        .filter(|v| v.kind() == &K::BinderExpression)
        .collect::<Vec<_>>();
    assert_eq!(binders.len(), 2);
    assert!(binders.iter().all(|v| v.is_recovered()));
    assert!(
        ast.node_views()
            .filter(|v| v.kind() == &K::TermExpression)
            .all(|v| !v.is_recovered())
    );
    let successor = "set after_boundary = y;";
    for text in [
        "set f = (x,y;\n) |-> x;\nset after_boundary = y;",
        "theorem Boundary: thesis proof\nset f = (x,y\nend\n) |-> x;\nset after_boundary = y;",
        "set f = (x,y\ntheorem Boundary: thesis\n) |-> x;\nset after_boundary = y;",
        "set f = (x,y\nset gap = y\n) |-> x;\nset after_boundary = y;",
    ] {
        let ast = super::super::formula_statement::step5c8_test_frontend(text)
            .ast
            .unwrap();
        assert!(ast.node_views().all(|v| v.kind() != &K::BinderExpression));
        let start = text.find(successor).unwrap();
        let sentinel = ast
            .node_views()
            .filter_map(|v| v.as_set_statement())
            .find(|v| v.range().start == start)
            .unwrap();
        assert_eq!(
            sentinel.range(),
            range(ast.source_id, start, start + successor.len())
        );
        assert!(!sentinel.is_recovered());
    }
}

#[test]
fn binder_host_roots_preserve_list_arguments_algorithm_routes_and_recovery() {
    use mizar_syntax::SurfaceNodeKind as K;
    let plan = step5c11_plan();
    for (id, diagnostics) in [
        ("pass_parser_binder_term_lists_001", 0),
        ("pass_parser_binder_algorithm_roots_001", 0),
        ("fail_parser_binder_list_recovery_001", 3),
        ("fail_parser_binder_algorithm_recovery_001", 3),
    ] {
        let case = plan.cases.iter().find(|case| case.id.0 == id).unwrap();
        let text = std::fs::read_to_string(&case.source_path).unwrap();
        let output = super::super::formula_statement::step5c8_test_frontend(&text);
        assert_eq!(output.diagnostics.len(), diagnostics, "{id}: {:?}", output.diagnostics);
        assert!(output.diagnostics.iter().all(|d| matches!(
            &d.code, mizar_frontend::orchestration::DiagnosticCode::Syntax(code)
                if code.as_ref() == "malformed_term_expression"
        )));
        let ast = output.ast.unwrap();
        let binders = ast.node_views().filter(|v| v.kind() == &K::BinderExpression).collect::<Vec<_>>();
        assert_eq!(binders.len(), text.matches("|->").count(), "{id}");
        for binder in &binders {
            let children = binder.children();
            let body = ast.node_view(*children.last().unwrap()).unwrap();
            let arrow = ast.node_view(children[children.len()-2]).unwrap();
            assert_eq!(arrow.as_token().unwrap().text.as_ref(), "|->");
            assert!(body.kind() == &K::TermExpression || body.as_recovery() == Some(SyntaxRecoveryKind::MissingTerm));
            assert_eq!(binder.range().source_id, ast.source_id);
        }
        if diagnostics == 0 {
            assert!(ast.node_views().all(|v| !v.is_recovered()));
            assert!(ast.trivia().skipped_token_ranges().is_empty());
        } else {
            assert_eq!(binders.iter().filter(|v| v.is_recovered()).count(), 3);
            let head_start = text.find("(x,x)").unwrap();
            let malformed = binders.iter().find(|v| v.range().start == head_start).unwrap();
            let skipped = ast.node_view(malformed.children()[0]).unwrap();
            assert_eq!(skipped.as_recovery(), Some(SyntaxRecoveryKind::SkippedToken));
            assert_eq!(skipped.range(), range(ast.source_id, head_start, head_start + 5));
            assert!(ast.trivia().skipped_token_ranges().iter().any(|v| v.range == skipped.range()));
            if id.contains("algorithm") {
                let start = text.find("after_const :=").unwrap();
                assert!(ast.node_views().any(|v| v.kind() == &K::VariableBinding && v.range().start == start && !v.is_recovered()));
            } else {
                assert_eq!(ast.node_views().filter(|v| v.kind() == &K::TermReference && &text[v.range().start..v.range().end] == "after").count(), 2);
            }
            let mut missing = ast.node_views().filter(|v| v.as_recovery() == Some(SyntaxRecoveryKind::MissingTerm)).collect::<Vec<_>>();
            missing.sort_by_key(|v| v.range().start);
            let expected = if id.contains("list") {
                [text.find(", after").unwrap(), text.find(");\nset repeated").unwrap()]
            } else {
                [text.find(";\n    var after_const").unwrap(), text.find(";\n  end").unwrap()]
            };
            assert_eq!(missing.len(), 2);
            for (index, recovery) in missing.iter().enumerate() {
                let start = expected[index];
                assert_eq!(recovery.range(), range(ast.source_id, start, start));
                assert!(ast.token_views().any(|v| v.range().start == start));
            }
            let successor = if id.contains("list") { "set after_list" } else { "set after_algorithm" };
            let start = text.find(successor).unwrap();
            assert!(ast.node_views().filter_map(|v| v.as_set_statement()).any(|v| v.range().start == start && !v.is_recovered()));
        }
        if id == "pass_parser_binder_term_lists_001" {
            let frontend = task257c4c1_frontend_output(&text, 120);
            let current = frontend.cache_keys.ast.as_ref().unwrap();
            assert_eq!(current.parser_version.version.as_ref(), "mizar-parser/surface-ast-v30");
            let mut retained = current.clone();
            retained.parser_version = mizar_frontend::parsing::ParserCacheKeyVersion::new("mizar-parser/surface-ast-v10");
            assert_ne!(current.stable_hash(), retained.stable_hash());
            for kind in [K::AttributeRef, K::InlinePredicateApplication, K::SelectorAccess, K::SetEnumeration] {
                assert!(ast.node_views().any(|v| v.kind() == &kind && v.children().iter().any(|child| {
                    let term = ast.node_view(*child).unwrap();
                    term.kind() == &K::TermExpression && ast.node_view(term.children()[0]).unwrap().kind() == &K::BinderExpression
                })), "missing list route {kind:?}");
            }
            let equating = ast.node_views().filter_map(|v| v.as_equating()).find(|v| text[v.range().start..].starts_with("split =")).unwrap();
            let term = ast.node_view(*equating.children().last().unwrap()).unwrap();
            let call = ast.node_view(term.children()[0]).unwrap();
            let actuals = call.children().iter().filter_map(|id| ast.node_view(*id)).filter(|v| v.kind() == &K::TermExpression).collect::<Vec<_>>();
            assert_eq!(actuals.len(), 2);
            assert_eq!(ast.node_view(actuals[0].children()[0]).unwrap().kind(), &K::TermReference);
            assert_eq!(ast.node_view(actuals[1].children()[0]).unwrap().kind(), &K::BinderExpression);
        }
    }
}

#[test]
fn binder_container_values_preserve_fields_generators_and_missing_body_boundaries() {
    use mizar_syntax::SurfaceNodeKind as K;
    let plan = step5c11_plan();
    for (id, diagnostics) in [
        ("pass_parser_binder_container_values_001", 0),
        ("fail_parser_binder_container_recovery_001", 6),
    ] {
        let case = plan.cases.iter().find(|case| case.id.0 == id).unwrap();
        let text = std::fs::read_to_string(&case.source_path).unwrap();
        let checked = super::super::formula_statement::step5c8_test_frontend(&text);
        assert_eq!(checked.diagnostics.len(), diagnostics, "{id}: {:?}", checked.diagnostics);
        assert!(checked.diagnostics.iter().all(|d| matches!(
            &d.code, mizar_frontend::orchestration::DiagnosticCode::Syntax(code)
                if code.as_ref() == "malformed_term_expression"
        )));
        let ast = checked.ast.unwrap();
        let binders = ast.node_views().filter(|v| v.kind() == &K::BinderExpression).collect::<Vec<_>>();
        assert_eq!(binders.len(), text.matches("|->").count(), "{id}");
        let start = text.find("set after_containers").unwrap();
        assert!(ast.node_views().filter_map(|v| v.as_set_statement()).any(|v| v.range().start == start && !v.is_recovered()));
        if diagnostics == 0 {
            assert!(ast.node_views().all(|v| !v.is_recovered()));
            for kind in [K::StructureField, K::FieldArgument, K::FieldUpdate, K::SetComprehension] {
                assert!(ast.node_views().any(|v| v.kind() == &kind && v.children().iter().any(|child| {
                    let term = ast.node_view(*child).unwrap();
                    term.kind() == &K::TermExpression && ast.node_view(term.children()[0]).unwrap().kind() == &K::BinderExpression
                })), "missing Binder value in {kind:?}");
            }
            let output = task257c4c1_frontend_output(&text, 121);
            let current = output.cache_keys.ast.as_ref().unwrap();
            assert_eq!(current.parser_version.version.as_ref(), "mizar-parser/surface-ast-v30");
            let mut retained = current.clone();
            retained.parser_version = mizar_frontend::parsing::ParserCacheKeyVersion::new("mizar-parser/surface-ast-v11");
            assert_ne!(current.stable_hash(), retained.stable_hash());
        } else {
            assert!(binders.iter().all(|v| v.is_recovered()));
            let mut missing = ast.node_views().filter(|v| v.as_recovery() == Some(SyntaxRecoveryKind::MissingTerm)).collect::<Vec<_>>();
            missing.sort_by_key(|v| v.range().start);
            let expected = [
                text.find(";\n    field other").unwrap(),
                text.find(", other: sentinel").unwrap(),
                text.find(");\nset missing_update").unwrap(),
                text.find(");\nset missing_mapper").unwrap(),
                text.find("where y is set").unwrap(),
            ];
            assert_eq!(missing.len(), expected.len());
            for (recovery, start) in missing.iter().zip(expected) {
                assert_eq!(recovery.range(), range(ast.source_id, start, start));
                assert!(ast.token_views().any(|v| v.range().start == start));
            }
            let head_start = text.find("(x,x)").unwrap();
            let binder = binders.iter().find(|v| v.range().start == head_start).unwrap();
            let skipped = ast.node_view(binder.children()[0]).unwrap();
            assert_eq!(skipped.as_recovery(), Some(SyntaxRecoveryKind::SkippedToken));
            assert_eq!(skipped.range(), range(ast.source_id, head_start, head_start + 5));
            assert!(ast.trivia().skipped_token_ranges().iter().any(|v| v.range == skipped.range()));
            for (kind, prefix) in [
                (K::StructureField, "field other"),
                (K::FieldArgument, "other: sentinel"),
                (K::ComprehensionVariableSegment, "y is set"),
            ] {
                let start = text.find(prefix).unwrap();
                assert!(ast.node_views().any(|v| v.kind() == &kind && v.range().start == start && !v.is_recovered()), "lost {kind:?}");
            }
        }
    }
}

#[test]
fn binder_mathematical_hosts_preserve_definiens_ownership_and_delimiters() {
    use mizar_syntax::SurfaceNodeKind as K;
    use mizar_frontend::orchestration::{DiagnosticCode, DiagnosticLocation};
    let plan = step5c11_plan();
    for (id, diagnostics) in [
        ("pass_parser_binder_mathematical_hosts_001", 0),
        ("fail_parser_binder_mathematical_recovery_001", 9),
    ] {
        let case = plan.cases.iter().find(|case| case.id.0 == id).unwrap();
        let text = std::fs::read_to_string(&case.source_path).unwrap();
        let output = super::super::formula_statement::step5c8_test_frontend(&text);
        assert_eq!(output.diagnostics.len(), diagnostics, "{id}: {:?}", output.diagnostics);
        let ast = output.ast.unwrap();
        let binders = ast.node_views().filter(|v| v.kind() == &K::BinderExpression).collect::<Vec<_>>();
        assert_eq!(binders.len(), text.matches("|->").count(), "{id}");
        assert!(ast.trivia().skipped_token_ranges().is_empty());
        let after = text.find("set after_math_hosts").unwrap();
        assert!(ast.node_views().filter_map(|v| v.as_set_statement()).any(|v| v.range().start == after && !v.is_recovered()));
        if diagnostics == 0 {
            assert!(ast.node_views().all(|v| !v.is_recovered()));
            for (kind, expected) in [
                (K::InlineFunctorDefinition, 1), (K::Witness, 2),
                (K::ReconsiderItem, 2), (K::ReductionRegistration, 2),
                (K::TermCase, 2), (K::TermDefiniens, 4),
            ] {
                let actual = ast.node_views().filter(|v| v.kind() == &kind).map(|v| {
                    v.children().iter().filter(|child| {
                        let term = ast.node_view(**child).unwrap();
                        term.kind() == &K::TermExpression && ast.node_view(term.children()[0]).unwrap().kind() == &K::BinderExpression
                    }).count()
                }).sum::<usize>();
                assert_eq!(actual, expected, "direct Binder terms in {kind:?}");
            }
            for kind in [K::FunctorDefinition, K::FunctorRedefinition, K::PropertyImplementation] {
                assert!(ast.node_views().any(|v| v.kind() == &kind && v.children().iter().any(|child| {
                    let definiens = ast.node_view(*child).unwrap();
                    definiens.kind() == &K::TermDefiniens && definiens.children().iter().any(|child| {
                        let term = ast.node_view(*child).unwrap();
                        term.kind() == &K::TermExpression && ast.node_view(term.children()[0]).unwrap().kind() == &K::BinderExpression
                    })
                })), "missing equals consumer {kind:?}");
            }
            for case in ast.node_views().filter(|v| v.kind() == &K::TermCase) {
                assert_eq!(case.children().len(), 3);
                let condition = ast.node_view(case.children()[2]).unwrap();
                assert_eq!(condition.kind(), &K::FormulaExpression);
                let keyword = ast.node_view(case.children()[1]).unwrap();
                assert_eq!(keyword.as_token().unwrap().text.as_ref(), "if");
                assert!(ast.node_view(case.children()[0]).unwrap().range().end <= keyword.range().start);
            }
            let frontend = task257c4c1_frontend_output(&text, 122);
            let current = frontend.cache_keys.ast.as_ref().unwrap();
            assert_eq!(current.parser_version.version.as_ref(), "mizar-parser/surface-ast-v30");
            let mut retained = current.clone();
            retained.parser_version = mizar_frontend::parsing::ParserCacheKeyVersion::new("mizar-parser/surface-ast-v13");
            assert_ne!(current.stable_hash(), retained.stable_hash());
        } else {
            let expected = text.match_indices("|->").map(|(start, arrow)| {
                let rest = &text[start + arrow.len()..];
                start + arrow.len() + rest.len() - rest.trim_start().len()
            }).collect::<Vec<_>>();
            let mut missing = ast.node_views().filter(|v| v.as_recovery() == Some(SyntaxRecoveryKind::MissingTerm)).collect::<Vec<_>>();
            missing.sort_by_key(|v| v.range().start);
            assert_eq!(missing.len(), expected.len());
            for ((recovery, start), diagnostic) in missing.iter().zip(expected).zip(&output.diagnostics) {
                assert_eq!(recovery.range(), range(ast.source_id, start, start));
                let delimiter = ast.token_views().find(|v| v.range().start == start).unwrap();
                assert!(matches!(delimiter.as_token().unwrap().text.as_ref(), ";" | "," | "as" | "to" | "if"));
                assert_eq!(diagnostic.location, DiagnosticLocation::SourceRange(delimiter.range()));
                assert!(matches!(&diagnostic.code, DiagnosticCode::Syntax(code) if code.as_ref() == "malformed_term_expression"));
            }
            for binder in binders {
                assert!(binder.is_recovered());
                let body = ast.node_view(*binder.children().last().unwrap()).unwrap();
                assert_eq!(body.as_recovery(), Some(SyntaxRecoveryKind::MissingTerm));
                assert!(binder.range().end <= body.range().start);
            }
            for (kind, count) in [(K::Witness, 2), (K::ReductionRegistration, 2), (K::TermCase, 2)] {
                assert_eq!(ast.node_views().filter(|v| v.kind() == &kind).count(), count);
            }
        }
    }
}

#[test]
fn binder_actuals_preserve_type_template_and_outer_field_boundaries() {
    use mizar_syntax::SurfaceNodeKind as K;
    use mizar_frontend::orchestration::{DiagnosticCode, DiagnosticLocation};
    let plan = step5c11_plan();
    for (id, diagnostics, constructors, calls) in [
        ("pass_parser_binder_actuals_001", 0, 5, 5),
        ("fail_parser_binder_actual_recovery_001", 7, 3, 2),
    ] {
        let case = plan.cases.iter().find(|case| case.id.0 == id).unwrap();
        let text = std::fs::read_to_string(&case.source_path).unwrap();
        let output = super::super::formula_statement::step5c8_test_frontend(&text);
        assert_eq!(output.diagnostics.len(), diagnostics, "{id}: {:?}", output.diagnostics);
        let ast = output.ast.unwrap();
        let binders = ast.node_views().filter(|v| v.kind() == &K::BinderExpression).collect::<Vec<_>>();
        assert_eq!(binders.len(), text.matches("|->").count());
        let values = ast.node_views().filter(|v| v.kind() == &K::StructureConstructor).collect::<Vec<_>>();
        assert_eq!(values.len(), constructors);
        for value in &values {
            let start = value.range().start;
            assert!(text[start..].starts_with("ValueBox of"));
            let opener = start + text[start..].find("(value:").unwrap();
            let end = start + text[start..].find(';').unwrap();
            assert_eq!(value.range().end, end);
            let children = value.children().iter().map(|id| ast.node_view(*id).unwrap()).collect::<Vec<_>>();
            let fields = children.iter().filter(|v| v.kind() == &K::FieldArgument).collect::<Vec<_>>();
            assert_eq!(fields.len(), 2);
            assert_eq!(fields.iter().map(|field| ast.node_view(field.children()[0]).unwrap().as_token().unwrap().text.as_ref().to_owned()).collect::<Vec<_>>(), ["value", "other"]);
            assert!(fields.iter().all(|v| !v.is_recovered()));
            assert!(children.iter().any(|v| v.as_token().is_some_and(|t| t.text.as_ref() == "(") && v.range().start == opener));
            assert!(children.iter().any(|v| v.as_token().is_some_and(|t| t.text.as_ref() == ")") && v.range().end == end));
            let arguments = children.iter().find(|v| v.kind() == &K::TypeArguments).unwrap();
            assert!(arguments.range().end <= opener);
            let actuals = arguments.children().iter().map(|id| ast.node_view(*id).unwrap()).filter(|v| v.kind() == &K::TermExpression).collect::<Vec<_>>();
            assert_eq!(actuals.len(), 1);
            assert_eq!(ast.node_view(actuals[0].children()[0]).unwrap().kind(), &K::BinderExpression);
        }
        let template_lists = ast.node_views().filter(|v| v.kind() == &K::TemplateArguments).collect::<Vec<_>>();
        assert_eq!(template_lists.len(), calls);
        assert_eq!(ast.node_views().filter(|v| v.kind() == &K::TemplateArgument && &text[v.range().start..v.range().end] == "sentinel" && !v.is_recovered()).count(), calls);
        assert_eq!(ast.node_views().filter(|v| v.kind() == &K::TermReference && &text[v.range().start..v.range().end] == "a" && !v.is_recovered()).count(), calls);
        for arguments in &template_lists {
            assert_eq!(arguments.children().iter().filter(|id| ast.node_view(**id).unwrap().kind() == &K::TemplateArgument).count(), 2);
        }
        let after = text.find("set after_binder_actuals").unwrap();
        assert!(ast.node_views().filter_map(|v| v.as_set_statement()).any(|v| v.range().start == after && !v.is_recovered()));
        let ordered_actuals: &[(&str, &[&str])] = if diagnostics == 0 {
            &[
                ("reserve family_of", &["x |-> x", "(x, y) |-> y"]),
                ("reserve family_over", &["x |-> y |-> x", "sentinel"]),
            ]
        } else {
            &[
                ("reserve missing_of_body", &["x |->", "sentinel"]),
                ("reserve missing_over_body", &["x |->"]),
            ]
        };
        for (prefix, expected) in ordered_actuals {
            let start = text.find(prefix).unwrap();
            let end = start + text[start..].find(';').unwrap();
            let args = ast.node_views().find(|v| v.kind() == &K::TypeArguments && v.range().start > start && v.range().end <= end).unwrap();
            let actuals = args.children().iter().map(|id| ast.node_view(*id).unwrap()).filter(|v| v.kind() == &K::TermExpression).collect::<Vec<_>>();
            assert_eq!(actuals.iter().map(|v| text[v.range().start..v.range().end].trim()).collect::<Vec<_>>(), *expected);
            for actual in actuals {
                if &text[actual.range().start..actual.range().end] == "sentinel" {
                    assert!(!actual.is_recovered());
                }
            }
        }
        if diagnostics == 0 {
            assert!(ast.node_views().all(|v| !v.is_recovered()));
            assert!(ast.trivia().skipped_token_ranges().is_empty());
            for arguments in template_lists {
                let first = arguments.children().iter().map(|id| ast.node_view(*id).unwrap()).find(|v| v.kind() == &K::TemplateArgument).unwrap();
                let mut descendants = first.children().to_vec();
                let mut has_binder = false;
                while let Some(child) = descendants.pop() {
                    let view = ast.node_view(child).unwrap();
                    has_binder |= view.kind() == &K::BinderExpression;
                    descendants.extend(view.children());
                }
                assert!(has_binder);
            }
            let constructor_qua = text.find("ValueBox of x |-> x qua").unwrap() + "ValueBox of ".len();
            let binder = binders.iter().find(|v| v.range().start == constructor_qua).unwrap();
            let body = ast.node_view(*binder.children().last().unwrap()).unwrap();
            assert_eq!(ast.node_view(body.children()[0]).unwrap().kind(), &K::QuaExpression);
            let whole_start = text.find("(x |-> x) qua set").unwrap();
            assert!(ast.node_views().any(|v| v.kind() == &K::QuaExpression && v.range().start == whole_start));
            let frontend = task257c4c1_frontend_output(&text, 124);
            let current = frontend.cache_keys.ast.as_ref().unwrap();
            assert_eq!(current.parser_version.version.as_ref(), "mizar-parser/surface-ast-v30");
            let mut retained = current.clone();
            retained.parser_version = mizar_frontend::parsing::ParserCacheKeyVersion::new("mizar-parser/surface-ast-v15");
            assert_ne!(current.stable_hash(), retained.stable_hash());
        } else {
            assert!(output.diagnostics.iter().all(|d| matches!(&d.code, DiagnosticCode::Syntax(code) if code.as_ref() == "malformed_term_expression")));
            let expected = text.match_indices("|->").filter_map(|(start, arrow)| {
                let rest = &text[start + arrow.len()..];
                let next = start + arrow.len() + rest.len() - rest.trim_start().len();
                matches!(text.as_bytes()[next], b',' | b';' | b'(' | b']').then_some(next)
            }).collect::<Vec<_>>();
            assert_eq!(expected.len(), 6);
            let mut missing = ast.node_views().filter(|v| v.as_recovery() == Some(SyntaxRecoveryKind::MissingTerm)).collect::<Vec<_>>();
            missing.sort_by_key(|v| v.range().start);
            assert_eq!(missing.len(), expected.len());
            for ((recovery, start), diagnostic) in missing.iter().zip(&expected).zip(&output.diagnostics) {
                assert_eq!(recovery.range(), range(ast.source_id, *start, *start));
                assert_eq!(diagnostic.location, DiagnosticLocation::SourceRange(range(ast.source_id, *start, *start + 1)));
            }
            let head = text.find("(x,x)").unwrap();
            let skipped = ast.node_views().filter(|v| v.as_recovery() == Some(SyntaxRecoveryKind::SkippedToken)).collect::<Vec<_>>();
            assert_eq!(skipped.len(), 1);
            assert_eq!(skipped[0].range(), range(ast.source_id, head, head + 5));
            assert_eq!(ast.trivia().skipped_token_ranges().len(), 1);
            assert_eq!(ast.trivia().skipped_token_ranges()[0].range, skipped[0].range());
            assert_eq!(output.diagnostics.last().unwrap().location, DiagnosticLocation::SourceRange(range(ast.source_id, head, head + 1)));
            let next = text.find("reserve after_over_body").unwrap();
            assert!(ast.node_views().any(|v| v.kind() == &K::ReserveItem && v.range().start == next && !v.is_recovered()));
        }
    }
}

#[test]
fn binder_formula_roots_preserve_relations_predicate_loci_and_iterative_steps() {
    use mizar_syntax::SurfaceNodeKind as K;
    use mizar_frontend::orchestration::{DiagnosticCode, DiagnosticLocation};
    let plan = step5c11_plan();
    for (id, diagnostics) in [
        ("pass_parser_binder_formula_roots_001", 0),
        ("fail_parser_binder_formula_recovery_001", 7),
    ] {
        let case = plan.cases.iter().find(|case| case.id.0 == id).unwrap();
        let text = std::fs::read_to_string(&case.source_path).unwrap();
        let output = super::super::formula_statement::step5c8_test_frontend(&text);
        assert_eq!(output.diagnostics.len(), diagnostics, "{id}: {:?}", output.diagnostics);
        let ast = output.ast.unwrap();
        let binders = ast.node_views().filter(|v| v.kind() == &K::BinderExpression).collect::<Vec<_>>();
        assert_eq!(binders.len(), text.matches("|->").count());
        assert!(ast.trivia().skipped_token_ranges().is_empty());
        let after = text.rfind("set after_formula").unwrap();
        assert!(ast.node_views().filter_map(|v| v.as_set_statement()).any(|v| v.range().start == after && !v.is_recovered()));
        assert_eq!(ast.node_views().filter(|v| v.kind() == &K::IterativeEqualityStatement).count(), 1);
        let step = ast.node_views().find(|v| v.kind() == &K::IterativeEqualityStep).unwrap();
        assert_eq!(ast.node_view(step.children()[0]).unwrap().as_token().unwrap().text.as_ref(), ".=");
        assert!(step.children().iter().any(|id| ast.node_view(*id).unwrap().kind() == &K::JustificationClause));
        if diagnostics == 0 {
            assert!(ast.node_views().all(|v| !v.is_recovered()));
            for (kind, expected) in [
                (K::BuiltinPredicateApplication, 6), (K::IsAssertion, 1),
                (K::PredicateSegment, 6), (K::IterativeEqualityStatement, 2),
                (K::IterativeEqualityStep, 1),
            ] {
                let count = ast.node_views().filter(|v| v.kind() == &kind).map(|v| {
                    v.children().iter().filter(|id| {
                        let term = ast.node_view(**id).unwrap();
                        term.kind() == &K::TermExpression && ast.node_view(term.children()[0]).unwrap().kind() == &K::BinderExpression
                    }).count()
                }).sum::<usize>();
                assert_eq!(count, expected, "direct Binder term ownership in {kind:?}");
            }
            for relation in ast.node_views().filter(|v| v.kind() == &K::BuiltinPredicateApplication) {
                let predicate = ast.node_view(relation.children()[1]).unwrap();
                assert!(matches!(predicate.as_token().unwrap().text.as_ref(), "=" | "<>" | "in"));
                assert!(ast.node_view(relation.children()[0]).unwrap().range().end <= predicate.range().start);
                assert!(ast.node_view(relation.children()[2]).unwrap().range().start >= predicate.range().end);
            }
            let many = ast.node_views().find(|v| v.kind() == &K::PredicateSegment && text[v.range().start..v.range().end].contains(" many ")).unwrap();
            let actuals = many.children().iter().map(|id| ast.node_view(*id).unwrap()).filter(|v| v.kind() == &K::TermExpression).map(|v| text[v.range().start..v.range().end].trim()).collect::<Vec<_>>();
            assert_eq!(actuals, ["x |-> x", "y |-> y", "z |-> z", "f |-> f"]);
            assert_eq!(binders.iter().filter(|v| {
                let body = ast.node_view(*v.children().last().unwrap()).unwrap();
                body.kind() == &K::TermExpression && ast.node_view(body.children()[0]).unwrap().kind() == &K::BinderExpression
            }).count(), 2);
            assert_eq!(ast.node_views().filter(|v| v.kind() == &K::JustificationClause).count(), 2);
            let frontend = task257c4c1_frontend_output(&text, 125);
            let current = frontend.cache_keys.ast.as_ref().unwrap();
            assert_eq!(current.parser_version.version.as_ref(), "mizar-parser/surface-ast-v30");
            let mut retained = current.clone();
            retained.parser_version = mizar_frontend::parsing::ParserCacheKeyVersion::new("mizar-parser/surface-ast-v16");
            assert_ne!(current.stable_hash(), retained.stable_hash());
        } else {
            let expected = text.match_indices("|->").map(|(start, arrow)| {
                let rest = &text[start + arrow.len()..];
                start + arrow.len() + rest.len() - rest.trim_start().len()
            }).collect::<Vec<_>>();
            assert_eq!(expected.len(), 7);
            let mut missing = ast.node_views().filter(|v| v.as_recovery() == Some(SyntaxRecoveryKind::MissingTerm)).collect::<Vec<_>>();
            missing.sort_by_key(|v| v.range().start);
            assert_eq!(missing.len(), expected.len());
            for ((recovery, start), diagnostic) in missing.iter().zip(expected).zip(&output.diagnostics) {
                assert_eq!(recovery.range(), range(ast.source_id, start, start));
                let delimiter = ast.token_views().find(|v| v.range().start == start).unwrap();
                assert!(matches!(delimiter.as_token().unwrap().text.as_ref(), "=" | ";" | "is" | "," | ".=" | "by"));
                assert_eq!(diagnostic.location, DiagnosticLocation::SourceRange(delimiter.range()));
                assert!(matches!(&diagnostic.code, DiagnosticCode::Syntax(code) if code.as_ref() == "malformed_term_expression"));
            }
            for (head, expected) in [
                (" triples ", ["f", "x |->", "g", "y"]),
                (" many ", ["f", "g", "x |->", "y"]),
            ] {
                let segment = ast.node_views().find(|v| v.kind() == &K::PredicateSegment && text[v.range().start..v.range().end].contains(head)).unwrap();
                let actuals = segment.children().iter().map(|id| ast.node_view(*id).unwrap()).filter(|v| v.kind() == &K::TermExpression).collect::<Vec<_>>();
                assert_eq!(actuals.iter().map(|v| text[v.range().start..v.range().end].trim()).collect::<Vec<_>>(), expected);
                assert!(!actuals.last().unwrap().is_recovered());
            }
            assert_eq!(ast.node_views().filter(|v| v.kind() == &K::JustificationClause).count(), 1);
        }
    }
}
