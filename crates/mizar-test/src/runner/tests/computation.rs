use super::*;

#[test]
fn computation_replay_keeps_contextual_boolean_tokens_and_numeric_controls() {
    use mizar_frontend::lexing::TokenKind;
    use mizar_syntax::{SurfaceNodeKind as K, SyntaxKind as S};
    let config = step5c11_config();
    let plan = build_test_plan(&config).unwrap();
    let case = plan.cases.iter().find(|case| case.id.0 == "pass_parser_computation_replay_001").unwrap();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let first = super::super::run_parse_only_case(&config.workspace_root, &config.workspace_root.join("tests"), case, 132);
    let repeated = super::super::run_parse_only_case(&config.workspace_root, &config.workspace_root.join("tests"), case, 132);
    assert_eq!(first.status, super::super::ParseOnlyCaseStatus::Passed, "{first:?}");
    assert_eq!(first, repeated);
    let declarations = "definition\n let x be set;\n func ReplayDef: replay -> set equals x;\n coherence;\n func TrueDef: true -> set equals x;\n coherence;\n func FalseDef: false -> set equals x;\n coherence;\nend;\n";
    let active = format!("{declarations}{}\n", text.lines().take(4).collect::<Vec<_>>().join("\n"));
    for (source, prefix, expected_kind, count) in [
        (text.as_str(), 0, SurfaceTokenKind::Identifier, 13),
        (active.as_str(), declarations.len(), SurfaceTokenKind::UserSymbol, 10),
    ] {
        let output = super::super::formula_statement::step5c8_test_frontend(source);
        assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
        let ast = output.ast.unwrap();
        assert!(ast.node_views().all(|v| !v.is_recovered()));
        assert!(ast.trivia().skipped_token_ranges().is_empty());
        let expected = [
            ["replay: true"].as_slice(), &["replay: false"],
            &["steps: 8", "replay: true", "timeout: 20", "nest: 3"],
            &["replay: false", "steps: 8", "timeout: 20", "nest: 3"],
            &[], &["steps: 8", "timeout: 20", "nest: 3"],
        ];
        let justifications = ast.node_views().filter_map(|v| v.as_computation_justification()).collect::<Vec<_>>();
        assert_eq!(justifications.len(), if prefix == 0 { 6 } else { 4 });
        assert_eq!(ast.node_views().filter_map(|v| v.as_computation_option()).count(), count);
        assert_eq!(ast.rowan_root().descendants().filter(|n| n.kind() == S::ComputationOption).count(), count);
        assert_eq!(ast.snapshot_text().matches("ComputationOption").count(), count);
        for ((justification, expected), (line_start, line)) in justifications.iter().zip(expected).zip(source.match_indices("thesis by computation").map(|(start, _)| (start, source[start..].lines().next().unwrap()))) {
            let start = line_start + "thesis by ".len();
            let end = line_start + line.len() - 1;
            assert_eq!(justification.range(), range(ast.source_id, start, end));
            let children = justification.children().iter().map(|id| ast.node_view(*id).unwrap()).collect::<Vec<_>>();
            let mut expected_children = vec!["computation"];
            if !expected.is_empty() {
                expected_children.push("(");
                for (index, option) in expected.iter().enumerate() {
                    if index > 0 { expected_children.push(","); }
                    expected_children.push(option);
                }
                expected_children.push(")");
            }
            assert_eq!(children.iter().map(|v| &source[v.range().start..v.range().end]).collect::<Vec<_>>(), expected_children);
            let options = children.iter().filter_map(|v| v.as_computation_option()).collect::<Vec<_>>();
            assert_eq!(options.len(), expected.len());
            for (option, spelling) in options.iter().zip(expected) {
                let offset = line_start + line.find(spelling).unwrap();
                assert_eq!(option.range(), range(ast.source_id, offset, offset + spelling.len()));
                let tokens = option.children().iter().map(|id| ast.node_view(*id).unwrap()).collect::<Vec<_>>();
                let (name, value) = spelling.split_once(": ").unwrap();
                assert_eq!(tokens.len(), 3);
                for (token, word, offset, kind) in [
                    (&tokens[0], name, offset, if name == "nest" { SurfaceTokenKind::ReservedWord } else if name == "replay" { expected_kind } else { SurfaceTokenKind::Identifier }),
                    (&tokens[1], ":", offset + name.len(), SurfaceTokenKind::ReservedSymbol),
                    (&tokens[2], value, offset + name.len() + 2, if name == "replay" { expected_kind } else { SurfaceTokenKind::Numeral }),
                ] {
                    assert_eq!(token.as_token().unwrap().text.as_ref(), word);
                    assert_eq!(token.as_token().unwrap().kind, kind);
                    assert_eq!(token.range(), range(ast.source_id, offset, offset + word.len()));
                }
            }
            let host = ast.node_views().find(|v| v.kind() == &K::CompactStatement && v.range().start == line_start).unwrap();
            assert_eq!(ast.node_view(*host.children().last().unwrap()).unwrap().as_token().unwrap().text.as_ref(), ";");
            assert_eq!(host.range().end, end + 1);
        }
        if prefix == 0 {
            assert_eq!(ast.node_views().filter_map(|v| v.as_set_statement()).count(), 2);
            for (target, rhs) in [("replay", "true"), ("false", "replay")] {
                let spelling = format!("set {target} = {rhs};");
                let start = source.find(&spelling).unwrap();
                let set = ast.node_views().filter_map(|v| v.as_set_statement()).find(|v| v.range().start == start).unwrap();
                assert_eq!(set.range(), range(ast.source_id, start, start + spelling.len()));
                let equating = ast.node_view(set.children()[1]).unwrap().as_equating().unwrap();
                assert_eq!(equating.children().len(), 3);
                let target_token = ast.node_view(equating.children()[0]).unwrap();
                assert_eq!(target_token.as_token().unwrap().text.as_ref(), target);
                assert_eq!(target_token.as_token().unwrap().kind, SurfaceTokenKind::Identifier);
                let term = ast.node_view(equating.children()[2]).unwrap();
                let reference = ast.node_view(term.children()[0]).unwrap();
                assert_eq!(reference.kind(), &K::TermReference);
                let token = ast.node_view(reference.children()[0]).unwrap();
                assert_eq!(token.as_token().unwrap().text.as_ref(), rhs);
                assert_eq!(token.as_token().unwrap().kind, SurfaceTokenKind::Identifier);
            }
        }
        let frontend = task257c4c1_frontend_output(source, 132 + usize::from(prefix > 0));
        assert!(frontend.diagnostics.is_empty(), "{:?}", frontend.diagnostics);
        for token in frontend.tokens.tokens.iter().filter(|t| matches!(t.text.as_ref(), "replay" | "true" | "false")) {
            assert_eq!(token.kind, if prefix > 0 && token.span.start >= prefix { TokenKind::UserSymbol } else { TokenKind::Identifier });
        }
        let current = frontend.cache_keys.ast.as_ref().unwrap();
        assert_eq!(current.parser_version.version.as_ref(), "mizar-parser/surface-ast-v30");
        let mut retained = current.clone();
        retained.parser_version = mizar_frontend::parsing::ParserCacheKeyVersion::new("mizar-parser/surface-ast-v22");
        assert_eq!(current.token_stream_hash, retained.token_stream_hash);
        assert_eq!(current.parser_inputs_hash, retained.parser_inputs_hash);
        assert_ne!(current.stable_hash(), retained.stable_hash());
        let repeated = task257c4c1_frontend_output(source, 132 + usize::from(prefix > 0));
        assert_eq!(frontend.cache_keys, repeated.cache_keys);
        assert_eq!(frontend.ast, repeated.ast);
        assert_eq!(frontend.ast.as_ref().unwrap().snapshot_text_with_trivia(), ast.snapshot_text_with_trivia());
        let bytes = ast.canonical_bytes().unwrap();
        let restored = mizar_syntax::SurfaceAst::from_canonical_bytes(&bytes, ast.source_id).unwrap();
        assert_eq!(restored.snapshot_text_with_trivia(), ast.snapshot_text_with_trivia());
    }
}

#[test]
fn computation_replay_recovery_keeps_legacy_anchors_option_ownership_and_successors() {
    use mizar_frontend::orchestration::{DiagnosticCode, DiagnosticLocation};
    use mizar_syntax::SurfaceNodeKind as K;
    let config = step5c11_config();
    let plan = build_test_plan(&config).unwrap();
    let case = plan.cases.iter().find(|case| case.id.0 == "fail_parser_computation_replay_001").unwrap();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let first = super::super::run_parse_only_case(&config.workspace_root, &config.workspace_root.join("tests"), case, 134);
    let repeated = super::super::run_parse_only_case(&config.workspace_root, &config.workspace_root.join("tests"), case, 134);
    assert_eq!(first.status, super::super::ParseOnlyCaseStatus::Passed, "{first:?}");
    assert_eq!(first, repeated);
    let output = super::super::formula_statement::step5c8_test_frontend(&text);
    assert_eq!(output.diagnostics.len(), 9, "{:?}", output.diagnostics);
    let ast = output.ast.unwrap();
    assert_eq!(ast.node_views().filter_map(|v| v.as_computation_justification()).count(), 6);
    assert_eq!(ast.node_views().filter_map(|v| v.as_computation_option()).count(), 7);
    assert_eq!(ast.node_views().filter_map(|v| v.as_set_statement()).count(), 6);
    assert_eq!(ast.node_views().filter(|v| v.as_recovery() == Some(SyntaxRecoveryKind::MissingProofStep)).count(), 5);
    assert_eq!(ast.node_views().filter(|v| v.as_recovery() == Some(SyntaxRecoveryKind::SkippedToken)).count(), 4);
    assert_eq!(ast.node_views().filter_map(|v| v.as_recovery()).count(), 9);
    assert_eq!(ast.trivia().skipped_token_ranges().len(), 4);
    let mut diagnostics = output.diagnostics.iter();
    for (body, options, anchor, missing, skipped, successor) in [
        ("replay: 1", ["replay:"].as_slice(), "1", true, "1", "after_replay_numeral"),
        ("replay: maybe", &["replay:"], "maybe", true, "maybe", "after_replay_identifier"),
        ("replay:", &["replay:"], ")", true, "", "after_replay_close"),
        ("replay:, steps: 2", &["replay:", "steps: 2"], ",", true, "", "after_replay_comma"),
        ("steps: true", &["steps:"], "true", true, "true", "after_steps_boolean"),
        ("replay: true or false", &["replay: true"], "or", false, "or false", "after_replay_expression"),
    ] {
        let spelling = format!("thesis by computation({body});");
        let start = text.find(&spelling).unwrap();
        let body_start = start + "thesis by computation(".len();
        let close = start + spelling.len() - 2;
        let anchor = body_start + text[body_start..].find(anchor).unwrap();
        for _ in 0..usize::from(missing) + usize::from(!skipped.is_empty()) {
            let diagnostic = diagnostics.next().unwrap();
            assert_eq!(diagnostic.code, DiagnosticCode::Syntax("malformed_justification".into()));
            let width = if skipped.is_empty() { 1 } else if body == "replay: maybe" { 5 } else if body == "steps: true" { 4 } else if missing { 1 } else { 2 };
            assert_eq!(diagnostic.location, DiagnosticLocation::SourceRange(range(ast.source_id, anchor, anchor + width)));
        }
        let justification = ast.node_views().filter_map(|v| v.as_computation_justification()).find(|v| v.range().start == start + "thesis by ".len()).unwrap();
        assert_eq!(justification.range().end, close + 1);
        let children = justification.children().iter().map(|id| ast.node_view(*id).unwrap()).collect::<Vec<_>>();
        let mut expected_children = vec!["computation", "("];
        for (index, option) in options.iter().enumerate() {
            if index > 0 { expected_children.push(","); }
            expected_children.push(option);
        }
        if !skipped.is_empty() { expected_children.push(skipped); }
        expected_children.push(")");
        assert_eq!(children.iter().map(|v| &text[v.range().start..v.range().end]).collect::<Vec<_>>(), expected_children);
        let actual_options = children.iter().filter_map(|v| v.as_computation_option()).collect::<Vec<_>>();
        assert_eq!(actual_options.len(), options.len());
        for (index, (option, expected)) in actual_options.iter().zip(options).enumerate() {
            let offset = body_start + body.find(expected).unwrap();
            assert_eq!(option.range(), range(ast.source_id, offset, offset + expected.len()));
            let parts = option.children().iter().map(|id| ast.node_view(*id).unwrap()).collect::<Vec<_>>();
            let tokens = parts.iter().filter_map(|v| v.as_token()).map(|t| t.text.as_ref()).collect::<Vec<_>>();
            let (name, value) = expected.split_once(':').unwrap();
            assert_eq!(tokens, if value.is_empty() { vec![name, ":"] } else { vec![name, ":", value.trim()] });
            assert_eq!(parts.len(), tokens.len() + usize::from(missing && index == 0));
            let raw_tokens = parts.iter().filter(|v| v.as_token().is_some()).collect::<Vec<_>>();
            for (token, word, token_start) in [
                (raw_tokens[0], name, offset), (raw_tokens[1], ":", offset + name.len()),
            ].into_iter().chain(raw_tokens.get(2).map(|token| (*token, value.trim(), offset + name.len() + 2))) {
                assert_eq!(token.range(), range(ast.source_id, token_start, token_start + word.len()));
            }
            if missing && index == 0 {
                let recovery = parts.last().unwrap();
                assert_eq!(recovery.as_recovery(), Some(SyntaxRecoveryKind::MissingProofStep));
                assert_eq!(recovery.range(), range(ast.source_id, anchor, anchor));
                assert!(recovery.is_recovered());
            }
        }
        let skipped_nodes = children.iter().filter(|v| v.as_recovery() == Some(SyntaxRecoveryKind::SkippedToken)).collect::<Vec<_>>();
        assert_eq!(skipped_nodes.len(), usize::from(!skipped.is_empty()));
        if !skipped.is_empty() {
            assert_eq!(skipped_nodes[0].range(), range(ast.source_id, anchor, anchor + skipped.len()));
            assert!(skipped_nodes[0].is_recovered());
            assert_eq!(skipped_nodes[0].children().iter().map(|id| ast.node_view(*id).unwrap().as_token().unwrap().text.as_ref()).collect::<Vec<_>>(), skipped.split_whitespace().collect::<Vec<_>>());
            assert!(ast.trivia().skipped_token_ranges().iter().any(|v| v.range == skipped_nodes[0].range()));
        }
        let closing = children.last().unwrap();
        assert_eq!(closing.as_token().unwrap().text.as_ref(), ")");
        assert_eq!(closing.range(), range(ast.source_id, close, close + 1));
        let next = format!("set {successor} = sentinel;");
        let next_start = text.find(&next).unwrap();
        let set = ast.node_views().filter_map(|v| v.as_set_statement()).find(|v| v.range().start == next_start).unwrap();
        assert_eq!(set.range(), range(ast.source_id, next_start, next_start + next.len()));
        assert!(!set.is_recovered());
        let equating = ast.node_view(set.children()[1]).unwrap().as_equating().unwrap();
        assert_eq!(ast.node_view(equating.children()[0]).unwrap().as_token().unwrap().text.as_ref(), successor);
        let term = ast.node_view(equating.children()[2]).unwrap();
        let reference = ast.node_view(term.children()[0]).unwrap();
        assert_eq!(reference.kind(), &K::TermReference);
        assert_eq!(ast.node_view(reference.children()[0]).unwrap().as_token().unwrap().text.as_ref(), "sentinel");
    }
    assert!(diagnostics.next().is_none());
    let bytes = ast.canonical_bytes().unwrap();
    let restored = mizar_syntax::SurfaceAst::from_canonical_bytes(&bytes, ast.source_id).unwrap();
    assert_eq!(restored.snapshot_text_with_trivia(), ast.snapshot_text_with_trivia());
}

#[test]
fn computation_term_calls_preserve_direct_calls_callees_arguments_and_option_order() {
    use mizar_syntax::{SurfaceNodeKind as K, SyntaxKind as S};
    let config = step5c11_config();
    let plan = build_test_plan(&config).unwrap();
    let case = plan.cases.iter().find(|case| case.id.0 == "pass_parser_computation_term_calls_001").unwrap();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let first = super::super::run_parse_only_case(&config.workspace_root, &config.workspace_root.join("tests"), case, 139);
    assert_eq!(first.status, super::super::ParseOnlyCaseStatus::Passed, "{first:?}");
    assert_eq!(first, super::super::run_parse_only_case(&config.workspace_root, &config.workspace_root.join("tests"), case, 139));
    let output = super::super::formula_statement::step5c8_test_frontend(&text);
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    let ast = output.ast.unwrap();
    assert!(ast.node_views().all(|v| !v.is_recovered()));
    assert!(ast.trivia().skipped_token_ranges().is_empty());
    assert_eq!(ast.node_views().filter_map(|v| v.as_computation_justification()).count(), 10);
    assert_eq!(ast.rowan_root().descendants().filter(|v| v.kind() == S::ComputationJustification).count(), 10);
    assert_eq!(ast.snapshot_text().matches("ComputationJustification").count(), 10);
    let mut calls = 0;
    let mut options = 0;
    for ((line_start, _), expected) in text.match_indices("thesis by computation").zip([
        ["calc()"].as_slice(), &["calc(1,x,f(y),x |-> x,(x,y) |-> y)"], &["pkg.calc(x)"],
        &["pkg.tools.calc[set](x)"], &["calc[set](x)"],
        &["calc()", "other(x)", "steps: 8", "timeout: 20", "nest: 3", "replay: true"],
        &["steps: 8", "replay: false", "calc(x)", "other()"],
        &["steps()", "replay()", "steps: 8", "replay: true"], &[], &["steps: 8", "timeout: 20", "nest: 3"],
    ]) {
        let start = line_start + "thesis by ".len();
        let semicolon = line_start + text[line_start..].find(';').unwrap();
        let justification = ast.node_views().filter_map(|v| v.as_computation_justification()).find(|v| v.range().start == start).unwrap();
        assert_eq!(justification.range(), range(ast.source_id, start, semicolon));
        let children = justification.children().iter().map(|id| ast.node_view(*id).unwrap()).collect::<Vec<_>>();
        let mut expected_children = vec!["computation"];
        if !expected.is_empty() {
            expected_children.push("(");
            for (index, spelling) in expected.iter().enumerate() {
                if index > 0 { expected_children.push(","); }
                expected_children.push(spelling);
            }
            expected_children.push(")");
        }
        assert_eq!(children.iter().map(|v| &text[v.range().start..v.range().end]).collect::<Vec<_>>(), expected_children);
        let arguments = children.iter().filter(|v| v.as_token().is_none()).collect::<Vec<_>>();
        assert_eq!(arguments.len(), expected.len());
        for (argument, spelling) in arguments.iter().zip(expected) {
            let offset = start + text[start..semicolon].find(spelling).unwrap();
            assert_eq!(argument.range(), range(ast.source_id, offset, offset + spelling.len()));
            if spelling.contains(": ") {
                options += 1;
                assert!(argument.as_computation_option().is_some());
                let (name, value) = spelling.split_once(": ").unwrap();
                assert_eq!(argument.children().iter().map(|id| ast.node_view(*id).unwrap().as_token().unwrap().text.as_ref()).collect::<Vec<_>>(), [name, ":", value]);
                continue;
            }
            calls += 1;
            assert!(argument.as_application_term().is_some());
            let parts = argument.children().iter().map(|id| ast.node_view(*id).unwrap()).collect::<Vec<_>>();
            let callee_text = spelling.split_once('(').unwrap().0;
            let terms: &[&str] = if spelling.starts_with("calc(1,") { ["1", "x", "f(y)", "x |-> x", "(x,y) |-> y"].as_slice() } else if spelling.ends_with("()") { &[] } else { &["x"] };
            let mut expected_parts = vec![callee_text, "("];
            for (index, term) in terms.iter().enumerate() {
                if index > 0 { expected_parts.push(","); }
                expected_parts.push(term);
            }
            expected_parts.push(")");
            assert_eq!(parts.iter().map(|v| &text[v.range().start..v.range().end]).collect::<Vec<_>>(), expected_parts);
            let mut cursor = offset;
            for (part, raw) in parts.iter().zip(expected_parts) {
                let part_start = cursor + text[cursor..argument.range().end].find(raw).unwrap();
                assert_eq!(part.range(), range(ast.source_id, part_start, part_start + raw.len()));
                cursor = part_start + raw.len();
            }
            let callee = parts[0];
            assert_eq!(callee.kind(), &K::TermReference);
            let callee_parts = callee.children().iter().map(|id| ast.node_view(*id).unwrap()).collect::<Vec<_>>();
            let head = callee_text.trim_end_matches("[set]");
            let name = head.rsplit('.').next().unwrap();
            let name_index = if let Some((namespace, _)) = head.rsplit_once('.') {
                assert_eq!(callee_parts[0].kind(), &K::NamespacePath);
                assert_eq!(callee_parts[0].range(), range(ast.source_id, offset, offset + namespace.len()));
                let path = callee_parts[0].children().iter().map(|id| ast.node_view(*id).unwrap()).collect::<Vec<_>>();
                assert_eq!(path.len(), namespace.split('.').count() * 2 - 1);
                for (index, segment) in path.iter().enumerate() {
                    if index % 2 == 0 {
                        assert_eq!(segment.kind(), &K::PathSegment);
                        assert_eq!(segment.children().len(), 1);
                        let token = ast.node_view(segment.children()[0]).unwrap();
                        assert_eq!(token.as_token().unwrap().text.as_ref(), namespace.split('.').nth(index / 2).unwrap());
                        assert_eq!(token.as_token().unwrap().kind, SurfaceTokenKind::Identifier);
                        assert_eq!(token.range(), segment.range());
                    } else { assert_eq!(segment.as_token().unwrap().text.as_ref(), "."); }
                }
                assert_eq!(callee_parts[1].as_token().unwrap().text.as_ref(), ".");
                assert_eq!(callee_parts[1].range(), range(ast.source_id, offset + namespace.len(), offset + namespace.len() + 1));
                2
            } else { 0 };
            assert_eq!(callee_parts[name_index].as_token().unwrap().text.as_ref(), name);
            assert_eq!(callee_parts[name_index].as_token().unwrap().kind, SurfaceTokenKind::Identifier);
            assert_eq!(callee_parts[name_index].range(), range(ast.source_id, offset + head.len() - name.len(), offset + head.len()));
            assert_eq!(callee_parts.len(), name_index + 1 + usize::from(callee_text.ends_with("[set]")));
            if callee_text.ends_with("[set]") {
                let template = callee_parts.last().unwrap();
                assert_eq!(template.kind(), &K::TemplateArguments);
                assert_eq!(template.range(), range(ast.source_id, offset + head.len(), offset + callee_text.len()));
                let template_parts = template.children().iter().map(|id| ast.node_view(*id).unwrap()).collect::<Vec<_>>();
                assert_eq!(template_parts.len(), 3);
                assert_eq!(template_parts[0].as_token().unwrap().text.as_ref(), "[");
                assert_eq!(template_parts[1].kind(), &K::TemplateArgument);
                assert_eq!(template_parts[1].children().len(), 1);
                assert_eq!(ast.node_view(template_parts[1].children()[0]).unwrap().kind(), &K::TypeExpression);
                assert_eq!(template_parts[2].as_token().unwrap().text.as_ref(), "]");
            }
            let actual_terms = parts.iter().filter(|v| v.kind() == &K::TermExpression).collect::<Vec<_>>();
            assert_eq!(actual_terms.len(), terms.len());
            for (term, spelling) in actual_terms.iter().zip(terms) {
                assert_eq!(ast.node_view(term.children()[0]).unwrap().kind(), &match *spelling { "1" => K::NumeralTerm, "f(y)" => K::ApplicationTerm, "x |-> x" | "(x,y) |-> y" => K::BinderExpression, _ => K::TermReference });
            }
        }
        let host = ast.node_views().find(|v| v.kind() == &K::CompactStatement && v.range().start == line_start).unwrap();
        assert_eq!(ast.node_view(*host.children().last().unwrap()).unwrap().range(), range(ast.source_id, semicolon, semicolon + 1));
        let mut pending = vec![host.id()];
        let mut seen = std::collections::HashSet::new();
        let mut tokens = Vec::new();
        while let Some(id) = pending.pop() {
            assert!(seen.insert(id), "duplicate computation call child");
            let node = ast.node_view(id).unwrap();
            if node.as_token().is_some() { tokens.push(id); }
            pending.extend(node.children().iter().rev().copied());
        }
        assert_eq!(tokens, ast.token_views().filter(|v| v.range().start >= line_start && v.range().end <= semicolon + 1).map(|v| v.id()).collect::<Vec<_>>());
    }
    assert_eq!((calls, options), (11, 11));
    let after = text.find("set after_computation_calls = sentinel;").unwrap();
    assert!(ast.node_views().filter_map(|v| v.as_set_statement()).any(|v| v.range() == range(ast.source_id, after, text.trim_end().len()) && !v.is_recovered()));
    let frontend = task257c4c1_frontend_output(&text, 139);
    assert!(frontend.diagnostics.is_empty());
    for token in frontend.tokens.tokens.iter().filter(|v| matches!(v.text.as_ref(), "calc" | "other" | "pkg" | "tools" | "f")) { assert_eq!(token.kind, mizar_frontend::lexing::TokenKind::Identifier); }
    assert_eq!(frontend.ast.as_ref().unwrap().snapshot_text_with_trivia(), ast.snapshot_text_with_trivia());
    let repeated = task257c4c1_frontend_output(&text, 139);
    assert_eq!(frontend.cache_keys, repeated.cache_keys);
    assert_eq!(frontend.ast, repeated.ast);
    let current = frontend.cache_keys.ast.as_ref().unwrap();
    assert_eq!(current.parser_version.version.as_ref(), "mizar-parser/surface-ast-v30");
    let mut retained = current.clone();
    retained.parser_version = mizar_frontend::parsing::ParserCacheKeyVersion::new("mizar-parser/surface-ast-v25");
    assert_eq!(current.token_stream_hash, retained.token_stream_hash);
    assert_eq!(current.parser_inputs_hash, retained.parser_inputs_hash);
    assert_ne!(current.stable_hash(), retained.stable_hash());
    let bytes = ast.canonical_bytes().unwrap();
    let restored = mizar_syntax::SurfaceAst::from_canonical_bytes(&bytes, ast.source_id).unwrap();
    assert_eq!(restored.snapshot_text_with_trivia(), ast.snapshot_text_with_trivia());
}

#[test]
fn computation_term_call_recovery_retains_calls_parent_tails_and_both_missing_closes() {
    use mizar_frontend::orchestration::{DiagnosticCode, DiagnosticLocation};
    use mizar_syntax::SurfaceNodeKind as K;
    let config = step5c11_config();
    let plan = build_test_plan(&config).unwrap();
    let case = plan.cases.iter().find(|case| case.id.0 == "fail_parser_computation_term_calls_001").unwrap();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let first = super::super::run_parse_only_case(&config.workspace_root, &config.workspace_root.join("tests"), case, 140);
    assert_eq!(first.status, super::super::ParseOnlyCaseStatus::Passed, "{first:?}");
    assert_eq!(first, super::super::run_parse_only_case(&config.workspace_root, &config.workspace_root.join("tests"), case, 140));
    let output = super::super::formula_statement::step5c8_test_frontend(&text);
    assert_eq!(output.diagnostics.len(), 7, "{:?}", output.diagnostics);
    let ast = output.ast.unwrap();
    for (diagnostic, (code, offset, word, message)) in output.diagnostics.iter().zip([
        ("malformed_justification", 22, "calc", "expected computation option"),
        ("malformed_justification", 91, "=", "unexpected token in computation justification"),
        ("malformed_justification", 159, "steps", "unexpected token in computation justification"),
        ("malformed_term_expression", 237, ",", "expected term before `,`"),
        ("malformed_term_expression", 313, ")", "expected term after `,`"),
        ("malformed_justification", 388, ";", "expected `)` to close justification delimiter"),
        ("malformed_term_expression", 388, ";", "expected `)` to close term delimiter"),
    ]) {
        assert_eq!(&text[offset..offset + word.len()], word);
        assert_eq!(diagnostic.code, DiagnosticCode::Syntax(code.into()));
        assert_eq!(diagnostic.message.as_ref(), message);
        assert_eq!(diagnostic.location, DiagnosticLocation::SourceRange(range(ast.source_id, offset, offset + word.len())));
        if offset == 388 {
            let outer = text.find("computation(calc(x;").unwrap() + "computation".len();
            let opener = outer + if code == "malformed_justification" { 0 } else { "(calc".len() };
            assert_eq!(diagnostic.secondary, [mizar_session::SourceAnchor::Range(range(ast.source_id, opener, opener + 1))]);
        } else { assert!(diagnostic.secondary.is_empty()); }
    }
    for (kind, count) in [(SyntaxRecoveryKind::MissingProofStep, 1), (SyntaxRecoveryKind::SkippedToken, 3), (SyntaxRecoveryKind::MissingTerm, 2), (SyntaxRecoveryKind::UnmatchedOpeningDelimiter, 2)] {
        assert_eq!(ast.node_views().filter(|v| v.as_recovery() == Some(kind)).count(), count);
    }
    assert_eq!(ast.node_views().filter_map(|v| v.as_recovery()).count(), 8);
    assert_eq!(ast.trivia().skipped_token_ranges().len(), 3);
    assert_eq!(ast.node_views().filter_map(|v| v.as_computation_option()).count(), 0);
    for (index, ((line_start, _), (body, call_parts, skipped, successor))) in text.match_indices("thesis by computation").zip([
        ("calc", [].as_slice(), "calc", "after_bare_call"),
        ("calc(x) = y", ["calc", "(", "x", ")"].as_slice(), "= y", "after_call_tail"),
        ("calc(x) steps: 2", &["calc", "(", "x", ")"], "steps: 2", "after_missing_separator"),
        ("calc(,x)", &["calc", "(", "", ",", "x", ")"], "", "after_missing_inner_term"),
        ("calc(x,)", &["calc", "(", "x", ",", "", ")"], "", "after_trailing_inner_comma"),
        ("calc(x", &["calc", "(", "x", ""], "", "after_missing_closes"),
    ]).enumerate() {
        let start = line_start + "thesis by ".len();
        let body_start = start + "computation(".len();
        let semicolon = line_start + text[line_start..].find(';').unwrap();
        assert_eq!(&text[body_start..body_start + body.len()], body);
        let justification = ast.node_views().filter_map(|v| v.as_computation_justification()).find(|v| v.range().start == start).unwrap();
        let children = justification.children().iter().map(|id| ast.node_view(*id).unwrap()).collect::<Vec<_>>();
        assert_eq!(justification.range(), range(ast.source_id, start, semicolon));
        let calls = children.iter().filter(|v| v.kind() == &K::ApplicationTerm).collect::<Vec<_>>();
        assert_eq!(calls.len(), usize::from(index != 0));
        let mut expected_parent = vec!["computation".to_owned(), "(".to_owned(), call_parts.join("")];
        if !skipped.is_empty() { expected_parent.push(skipped.to_owned()); }
        expected_parent.push(if index == 5 { "" } else { ")" }.to_owned());
        assert_eq!(children.iter().map(|v| text[v.range().start..v.range().end].to_owned()).collect::<Vec<_>>(), expected_parent);
        if index == 0 {
            let missing = children.iter().find(|v| v.as_recovery() == Some(SyntaxRecoveryKind::MissingProofStep)).unwrap();
            assert_eq!(missing.range(), range(ast.source_id, 22, 22));
        } else {
            let call = calls[0];
            assert_eq!(call.range(), range(ast.source_id, body_start, body_start + call_parts.join("").len()));
            let parts = call.children().iter().map(|id| ast.node_view(*id).unwrap()).collect::<Vec<_>>();
            assert_eq!(parts.iter().map(|v| &text[v.range().start..v.range().end]).collect::<Vec<_>>(), call_parts);
            assert_eq!(parts[0].kind(), &K::TermReference);
            assert_eq!(parts[0].children().len(), 1);
            assert_eq!(ast.node_view(parts[0].children()[0]).unwrap().as_token().unwrap().text.as_ref(), "calc");
            if index >= 3 {
                let anchor = [237, 313, 388][index - 3];
                let recovery = parts.iter().find(|v| v.as_recovery().is_some()).unwrap();
                assert_eq!(recovery.as_recovery(), Some(if index == 5 { SyntaxRecoveryKind::UnmatchedOpeningDelimiter } else { SyntaxRecoveryKind::MissingTerm }));
                assert_eq!(recovery.range(), range(ast.source_id, anchor, anchor));
            }
        }
        let tails = children.iter().filter(|v| v.as_recovery() == Some(SyntaxRecoveryKind::SkippedToken)).collect::<Vec<_>>();
        assert_eq!(tails.len(), usize::from(!skipped.is_empty()));
        if !skipped.is_empty() {
            let tail_start = body_start + body.find(skipped).unwrap();
            assert_eq!(tails[0].range(), range(ast.source_id, tail_start, tail_start + skipped.len()));
            assert!(ast.trivia().skipped_token_ranges().iter().any(|v| v.range == tails[0].range()));
        }
        let close = children.last().unwrap();
        if index == 5 {
            assert_eq!(close.as_recovery(), Some(SyntaxRecoveryKind::UnmatchedOpeningDelimiter));
            assert_eq!(close.range(), range(ast.source_id, 388, 388));
            assert!(ast.token_views().filter(|v| v.range().start >= body_start && v.range().end <= semicolon).all(|v| v.as_token().unwrap().text.as_ref() != ")"));
        } else { assert_eq!(close.range(), range(ast.source_id, semicolon - 1, semicolon)); assert_eq!(close.as_token().unwrap().text.as_ref(), ")"); }
        let host = ast.node_views().find(|v| v.kind() == &K::CompactStatement && v.range().start == line_start).unwrap();
        assert_eq!(ast.node_view(*host.children().last().unwrap()).unwrap().range(), range(ast.source_id, semicolon, semicolon + 1));
        let next = format!("set {successor} = sentinel;");
        let next_start = text.find(&next).unwrap();
        let set = ast.node_views().filter_map(|v| v.as_set_statement()).find(|v| v.range().start == next_start).unwrap();
        assert_eq!(set.range(), range(ast.source_id, next_start, next_start + next.len()));
        assert!(!set.is_recovered());
        let equating = ast.node_view(set.children()[1]).unwrap().as_equating().unwrap();
        assert_eq!(ast.node_view(equating.children()[0]).unwrap().as_token().unwrap().text.as_ref(), successor);
        let term = ast.node_view(equating.children()[2]).unwrap();
        assert_eq!(term.kind(), &K::TermExpression);
        assert_eq!(&text[term.range().start..term.range().end], "sentinel");
    }
    let bytes = ast.canonical_bytes().unwrap();
    let restored = mizar_syntax::SurfaceAst::from_canonical_bytes(&bytes, ast.source_id).unwrap();
    assert_eq!(restored.snapshot_text_with_trivia(), ast.snapshot_text_with_trivia());
}

#[test]
fn computation_contextual_inputs_preserve_real_source_shapes_order_and_identity() {
    use mizar_syntax::{SurfaceFormulaBinaryOperator as B, SurfaceFormulaConnective as C, SurfaceFormulaConstant as F, SurfaceFormulaPrefixOperator as P, SurfaceNodeKind as K};
    let config = step5c11_config();
    let plan = build_test_plan(&config).unwrap();
    let case = plan.cases.iter().find(|v| v.id.0 == "pass_parser_computation_contextual_inputs_001").unwrap();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let output = super::super::formula_statement::step5c8_test_frontend(&text);
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    let ast = output.ast.unwrap();
    assert!(ast.node_views().all(|v| !v.is_recovered() && v.as_recovery().is_none()));
    assert!(ast.trivia().skipped_token_ranges().is_empty());
    let justifications = ast.node_views().filter_map(|v| v.as_computation_justification()).collect::<Vec<_>>();
    let expected_order: &[&[&str]] = &[
        &["inspect(A1 & A2, x = x)"], &["inspect((A1 & A2), x = x)"],
        &["inspect(x = y, x is object)"], &["inspect(not (x = y), (x = x) & (y = y))"],
        &["inspect((for a being object holds a = a), (ex b being object st b = x))"],
        &["inspect(thesis, x = x)"], &["inspect((thesis), x = x)"], &["inspect(not (thesis), x = x)"],
        &["inspect(A1, nested(x))"], &["inspect(not (x rel y), x = x)"],
        &["inspect(not (x, y multi z), x = x)"], &["inspect(not (x, y do not multi z), x = x)"],
        &["copy_term((x, y) |-> x)"], &["copy_term(x qua object)"], &["copy_term([x, y])"],
        &["pkg.tools.inspect[set](A1 & A2, x = x)"],
        &["inspect(A1 & A2, x = x)", "other(x)", "steps: 8", "timeout: 20", "nest: 3", "replay: true"],
        &["steps: 8", "replay: false", "inspect(x = x, y = y)", "other()"],
    ];
    assert_eq!(justifications.len(), expected_order.len());
    let mut calls = Vec::new();
    for (((start, _), owner), expected) in text.match_indices("computation(").zip(&justifications).zip(expected_order) {
        let end = start + text[start..].find(';').unwrap();
        assert_eq!(owner.range(), range(ast.source_id, start, end));
        let children = owner.child_views().collect::<Vec<_>>();
        let mut spellings = vec!["computation", "("];
        for (index, piece) in expected.iter().enumerate() { if index > 0 { spellings.push(","); } spellings.push(piece); }
        spellings.push(")");
        assert_eq!(children.iter().map(|v| &text[v.range().start..v.range().end]).collect::<Vec<_>>(), spellings);
        let mut cursor = start;
        for (child, raw) in children.iter().zip(spellings) {
            let offset = cursor + text[cursor..end].find(raw).unwrap();
            assert_eq!(child.range(), range(ast.source_id, offset, offset + raw.len()));
            cursor = offset + raw.len();
            if child.kind() == &K::ApplicationTerm { calls.push(*child); }
            if raw.contains(": ") {
                assert!(child.as_computation_option().is_some());
                let (name, value) = raw.split_once(": ").unwrap();
                assert_eq!(child.child_views().map(|v| v.as_token().unwrap().text.as_ref()).collect::<Vec<_>>(), [name, ":", value]);
            }
        }
    }
    let and = K::BinaryFormula(B { connective: C::And, repeated: false });
    let expected_inputs: &[&[(&str, bool, K)]] = &[
        &[("A1 & A2", true, and.clone()), ("x = x", true, K::BuiltinPredicateApplication)],
        &[("(A1 & A2)", true, K::ParenthesizedFormula), ("x = x", true, K::BuiltinPredicateApplication)],
        &[("x = y", true, K::BuiltinPredicateApplication), ("x is object", true, K::IsAssertion)],
        &[("not (x = y)", true, K::PrefixFormula(P::Not)), ("(x = x) & (y = y)", true, and.clone())],
        &[("(for a being object holds a = a)", true, K::ParenthesizedFormula), ("(ex b being object st b = x)", true, K::ParenthesizedFormula)],
        &[("thesis", true, K::FormulaConstant(F::Thesis)), ("x = x", true, K::BuiltinPredicateApplication)],
        &[("(thesis)", true, K::ParenthesizedFormula), ("x = x", true, K::BuiltinPredicateApplication)],
        &[("not (thesis)", true, K::PrefixFormula(P::Not)), ("x = x", true, K::BuiltinPredicateApplication)],
        &[("A1", false, K::TermReference), ("nested(x)", false, K::ApplicationTerm)],
        &[("not (x rel y)", true, K::PrefixFormula(P::Not)), ("x = x", true, K::BuiltinPredicateApplication)],
        &[("not (x, y multi z)", true, K::PrefixFormula(P::Not)), ("x = x", true, K::BuiltinPredicateApplication)],
        &[("not (x, y do not multi z)", true, K::PrefixFormula(P::Not)), ("x = x", true, K::BuiltinPredicateApplication)],
        &[("(x, y) |-> x", false, K::BinderExpression)], &[("x qua object", false, K::QuaExpression)],
        &[("[x, y]", false, K::ApplicationTerm)],
        &[("A1 & A2", true, and.clone()), ("x = x", true, K::BuiltinPredicateApplication)],
        &[("A1 & A2", true, and.clone()), ("x = x", true, K::BuiltinPredicateApplication)], &[("x", false, K::TermReference)],
        &[("x = x", true, K::BuiltinPredicateApplication), ("y = y", true, K::BuiltinPredicateApplication)], &[],
    ];
    assert_eq!(calls.len(), expected_inputs.len());
    for ((call, expected), callee_text) in calls.iter().zip(expected_inputs).zip([
        "inspect", "inspect", "inspect", "inspect", "inspect", "inspect", "inspect", "inspect", "inspect", "inspect", "inspect", "inspect",
        "copy_term", "copy_term", "copy_term", "pkg.tools.inspect[set]", "inspect", "other", "inspect", "other",
    ]) {
        let parts = call.child_views().collect::<Vec<_>>();
        let callee = parts[0];
        assert_eq!(callee.kind(), &K::TermReference);
        let mut spellings = vec![callee_text, "("];
        for (index, (raw, _, _)) in expected.iter().enumerate() { if index > 0 { spellings.push(","); } spellings.push(raw); }
        spellings.push(")");
        assert_eq!(parts.iter().map(|v| &text[v.range().start..v.range().end]).collect::<Vec<_>>(), spellings);
        let mut cursor = call.range().start;
        for (part, raw) in parts.iter().zip(spellings) {
            let offset = cursor + text[cursor..call.range().end].find(raw).unwrap();
            assert_eq!(part.range(), range(ast.source_id, offset, offset + raw.len()));
            cursor = offset + raw.len();
        }
        let inputs = parts.iter().filter(|v| matches!(v.kind(), K::TermExpression | K::FormulaExpression)).collect::<Vec<_>>();
        assert_eq!(inputs.len(), expected.len());
        for (input, (raw, formula, root)) in inputs.iter().zip(*expected) {
            assert_eq!(input.kind(), if *formula { &K::FormulaExpression } else { &K::TermExpression }, "{raw}");
            assert_eq!(input.children().len(), 1);
            let root_node = ast.node_view(input.children()[0]).unwrap();
            assert_eq!(root_node.kind(), root, "{raw}");
            assert_eq!(root_node.range(), input.range());
            if raw.starts_with("(for ") || raw.starts_with("(ex ") {
                let inner = ast.node_view(root_node.children()[1]).unwrap();
                assert_eq!(inner.kind(), &K::FormulaExpression);
                assert_eq!(ast.node_view(inner.children()[0]).unwrap().kind(), &K::QuantifiedFormula(if raw.starts_with("(for ") { mizar_syntax::SurfaceQuantifierKind::Universal } else { mizar_syntax::SurfaceQuantifierKind::Existential }));
            }
            if *raw == "nested(x)" {
                let nested = root_node.child_views().collect::<Vec<_>>();
                assert_eq!(nested.iter().map(|v| &text[v.range().start..v.range().end]).collect::<Vec<_>>(), ["nested", "(", "x", ")"]);
                assert_eq!(nested[2].kind(), &K::TermExpression);
                assert_eq!(ast.node_view(nested[2].children()[0]).unwrap().kind(), &K::TermReference);
            }
        }
        if callee_text.starts_with("pkg.") {
            let head = callee.child_views().collect::<Vec<_>>();
            assert_eq!(head.len(), 4);
            assert_eq!(head[0].kind(), &K::NamespacePath);
            assert_eq!(&text[head[0].range().start..head[0].range().end], "pkg.tools");
            assert_eq!(head[1].as_token().unwrap().text.as_ref(), ".");
            assert_eq!(head[2].as_token().unwrap().text.as_ref(), "inspect");
            assert_eq!(head[3].kind(), &K::TemplateArguments);
            assert_eq!(&text[head[3].range().start..head[3].range().end], "[set]");
        }
    }
    let conjunctions = ast.node_views().filter(|v| v.kind() == &and && &text[v.range().start..v.range().end] == "A1 & A2").collect::<Vec<_>>();
    assert_eq!(conjunctions.len(), 4);
    for root in conjunctions {
        let operands = root.child_views().filter(|v| v.kind() == &K::TermExpression).collect::<Vec<_>>();
        assert_eq!(operands.len(), 2);
        for (operand, name) in operands.iter().zip(["A1", "A2"]) {
            assert_eq!(&text[operand.range().start..operand.range().end], name);
            assert_eq!(operand.children().len(), 1);
            assert_eq!(ast.node_view(operand.children()[0]).unwrap().kind(), &K::TermReference);
        }
    }
    for (raw, pieces) in [("x rel y", vec!["x", "rel", "y"]), ("x, y multi z", vec!["x", ",", "y", "multi", "z"]), ("x, y do not multi z", vec!["x", ",", "y", "do", "not", "multi", "z"])] {
        let start = text.find(&format!("not ({raw})")).unwrap() + 5;
        let segment = ast.node_views().find(|v| v.kind() == &K::PredicateSegment && v.range() == range(ast.source_id, start, start + raw.len())).unwrap();
        assert_eq!(segment.child_views().map(|v| &text[v.range().start..v.range().end]).collect::<Vec<_>>(), pieces);
        assert_eq!(segment.child_views().filter(|v| v.kind() == &K::TermExpression).count(), if raw.contains("multi") { 3 } else { 2 });
    }
    let mut pending = vec![ast.node_views().find(|v| v.kind() == &K::CompilationUnit).unwrap().id()];
    let mut seen = std::collections::HashSet::new();
    let mut tokens = Vec::new();
    while let Some(id) = pending.pop() {
        assert!(seen.insert(id), "duplicate contextual input source child");
        let node = ast.node_view(id).unwrap();
        if node.as_token().is_some() { tokens.push(id); }
        pending.extend(node.children().iter().rev().copied());
    }
    assert_eq!(tokens, ast.token_views().map(|v| v.id()).collect::<Vec<_>>());
    let after = text.find("set after_contextual_inputs = sentinel;").unwrap();
    assert!(ast.node_views().filter_map(|v| v.as_set_statement()).any(|v| v.range() == range(ast.source_id, after, text.trim_end().len()) && !v.is_recovered()));
    let first = super::super::run_parse_only_case(&config.workspace_root, &config.workspace_root.join("tests"), case, 164);
    assert_eq!(first.status, super::super::ParseOnlyCaseStatus::Passed, "{first:?}");
    assert_eq!(first, super::super::run_parse_only_case(&config.workspace_root, &config.workspace_root.join("tests"), case, 164));
    let frontend = task257c4c1_frontend_output(&text, 164);
    let repeated = task257c4c1_frontend_output(&text, 164);
    assert!(frontend.diagnostics.is_empty());
    assert_eq!(frontend.ast.as_ref().unwrap().snapshot_text_with_trivia(), ast.snapshot_text_with_trivia());
    assert_eq!(frontend.cache_keys, repeated.cache_keys);
    assert_eq!(frontend.ast, repeated.ast);
    assert_eq!(frontend.cache_keys.tokens.version.as_ref(), "mizar-frontend/token-stream-cache-key/v6");
    let current = frontend.cache_keys.ast.as_ref().unwrap();
    assert_eq!(current.parser_version.version.as_ref(), "mizar-parser/surface-ast-v30");
    let mut retained = current.clone();
    retained.parser_version = mizar_frontend::parsing::ParserCacheKeyVersion::new("mizar-parser/surface-ast-v29");
    assert_eq!(current.token_stream_hash, retained.token_stream_hash);
    assert_eq!(current.parser_inputs_hash, retained.parser_inputs_hash);
    assert_eq!(current.edition, retained.edition);
    assert_ne!(current.stable_hash(), retained.stable_hash());
}

#[test]
fn computation_contextual_thesis_placement_uses_each_actual_statement_owner() {
    use mizar_frontend::orchestration::{DiagnosticCode, DiagnosticLocation};
    use mizar_syntax::{SurfaceFormulaBinaryOperator as B, SurfaceFormulaConnective as C, SurfaceFormulaConstant as F, SurfaceFormulaPrefixOperator as P, SurfaceNodeKind as K};
    let config = step5c11_config();
    let plan = build_test_plan(&config).unwrap();
    let case = plan.cases.iter().find(|v| v.id.0 == "fail_parser_computation_thesis_placement_001").unwrap();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let output = super::super::formula_statement::step5c8_test_frontend(&text);
    assert_eq!(output.diagnostics.len(), 7, "{:?}", output.diagnostics);
    let ast = output.ast.unwrap();
    assert!(ast.node_views().all(|v| !v.is_recovered() && v.as_recovery().is_none()));
    assert!(ast.trivia().skipped_token_ranges().is_empty());
    let calls = ast.node_views().filter_map(|v| v.as_computation_justification()).map(|v| {
        let call = v.child_views().find(|v| v.kind() == &K::ApplicationTerm).unwrap();
        assert_eq!(v.child_views().last().unwrap().as_token().unwrap().text.as_ref(), ")");
        call
    }).collect::<Vec<_>>();
    assert_eq!(calls.len(), 8);
    for ((call, (input, root_kind, right)), index) in calls.iter().zip([
        ("thesis & thesis", K::BinaryFormula(B { connective: C::And, repeated: false }), "0 = 0"),
        ("(thesis)", K::ParenthesizedFormula, "0 = 0"), ("not thesis", K::PrefixFormula(P::Not), "0 = 0"),
        ("thesis", K::FormulaConstant(F::Thesis), "0 = 0"), ("thesis", K::FormulaConstant(F::Thesis), "0 = 0"),
        ("thesis", K::FormulaConstant(F::Thesis), "0 = 0"), ("thesis", K::FormulaConstant(F::Thesis), "0 = 0"),
        ("thesis", K::FormulaConstant(F::Thesis), "f"),
    ]).zip(0..) {
        let parts = call.child_views().collect::<Vec<_>>();
        assert_eq!(parts.iter().map(|v| &text[v.range().start..v.range().end]).collect::<Vec<_>>(), ["inspect", "(", input, ",", right, ")"]);
        assert_eq!(parts[0].kind(), &K::TermReference);
        assert_eq!(parts[2].kind(), &K::FormulaExpression);
        assert_eq!(parts[2].children().len(), 1);
        assert_eq!(ast.node_view(parts[2].children()[0]).unwrap().kind(), &root_kind);
        assert_eq!(parts[4].kind(), if right == "f" { &K::TermExpression } else { &K::FormulaExpression });
        let mut cursor = call.range().start;
        for (part, raw) in parts.iter().zip(["inspect", "(", input, ",", right, ")"]) {
            let start = cursor + text[cursor..call.range().end].find(raw).unwrap();
            assert_eq!(part.range(), range(ast.source_id, start, start + raw.len()));
            cursor = start + raw.len();
        }
        if index == 5 { continue; } // The inner direct thus establishes its own permission.
        let diagnostic = &output.diagnostics[index - usize::from(index > 5)];
        let anchor = parts[2].range().start + input.find("thesis").unwrap();
        assert_eq!(diagnostic.code, DiagnosticCode::Syntax("malformed_formula_expression".into()));
        assert_eq!(diagnostic.location, DiagnosticLocation::SourceRange(range(ast.source_id, anchor, anchor + 6)));
        assert!(diagnostic.secondary.is_empty());
    }
    for raw in ["theorem AfterOneLine: thesis;", "B: x = x;", "C: x = x;", "D: x = x;", "E: x = x;", "mode E:OtherMode is set;", "var after_return := f;", "set after_contextual_placement = sentinel;"] {
        let start = text.find(raw).unwrap();
        assert!(ast.node_views().any(|v| v.range() == range(ast.source_id, start, start + raw.len()) && !v.is_recovered()), "{raw}");
    }
    let first = super::super::run_parse_only_case(&config.workspace_root, &config.workspace_root.join("tests"), case, 165);
    assert_eq!(first.status, super::super::ParseOnlyCaseStatus::Passed, "{first:?}");
}

#[test]
fn computation_contextual_recovery_preserves_argument_and_enclosing_delimiters() {
    use mizar_frontend::orchestration::{DiagnosticCode, DiagnosticLocation};
    use mizar_syntax::{SurfaceFormulaBinaryOperator as B, SurfaceFormulaConnective as C, SurfaceFormulaPrefixOperator as P, SurfaceNodeKind as K};
    let config = step5c11_config();
    let plan = build_test_plan(&config).unwrap();
    let case = plan.cases.iter().find(|v| v.id.0 == "fail_parser_computation_contextual_recovery_001").unwrap();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let output = super::super::formula_statement::step5c8_test_frontend(&text);
    assert_eq!(output.diagnostics.len(), 3, "{:?}", output.diagnostics);
    let ast = output.ast.unwrap();
    for (diagnostic, (raw, code, root_kind, recovery_kind)) in output.diagnostics.iter().zip([
        ("x =", "malformed_term_expression", K::BuiltinPredicateApplication, SyntaxRecoveryKind::MissingTerm),
        ("not", "malformed_formula_expression", K::PrefixFormula(P::Not), SyntaxRecoveryKind::MissingFormula),
        ("A1 &", "malformed_formula_expression", K::BinaryFormula(B { connective: C::And, repeated: false }), SyntaxRecoveryKind::MissingFormula),
    ]) {
        let start = text.find(&format!("inspect({raw}")).unwrap();
        let comma = start + text[start..].find(',').unwrap();
        assert_eq!(diagnostic.code, DiagnosticCode::Syntax(code.into()));
        assert_eq!(diagnostic.location, DiagnosticLocation::SourceRange(range(ast.source_id, comma, comma + 1)));
        let call = ast.node_views().find(|v| v.kind() == &K::ApplicationTerm && v.range().start == start).unwrap();
        let parts = call.child_views().collect::<Vec<_>>();
        assert_eq!(parts.iter().map(|v| &text[v.range().start..v.range().end]).collect::<Vec<_>>(), ["inspect", "(", raw, ",", "x = x", ")"]);
        assert_eq!(parts[0].kind(), &K::TermReference);
        assert_eq!(parts[2].kind(), &K::FormulaExpression);
        assert_eq!(parts[2].range(), range(ast.source_id, start + 8, start + 8 + raw.len()));
        let root = ast.node_view(parts[2].children()[0]).unwrap();
        assert_eq!(root.kind(), &root_kind);
        let missing = ast.node_view(*root.children().last().unwrap()).unwrap();
        assert_eq!(missing.as_recovery(), Some(recovery_kind));
        assert_eq!(missing.range(), range(ast.source_id, comma, comma));
        assert!(missing.children().is_empty());
        assert_eq!(parts[3].range(), range(ast.source_id, comma, comma + 1));
        assert_eq!(parts[4].kind(), &K::FormulaExpression);
        assert_eq!(ast.node_view(parts[4].children()[0]).unwrap().kind(), &K::BuiltinPredicateApplication);
        let semicolon = start + text[start..].find(';').unwrap();
        assert_eq!(parts[5].range(), range(ast.source_id, semicolon - 2, semicolon - 1));
        let owner = ast.node_views().filter_map(|v| v.as_computation_justification()).find(|v| v.range().start == start - 12).unwrap();
        assert_eq!(owner.child_views().last().unwrap().range(), range(ast.source_id, semicolon - 1, semicolon));
    }
    assert_eq!(ast.node_views().filter_map(|v| v.as_recovery()).count(), 3);
    assert!(ast.trivia().skipped_token_ranges().is_empty());
    assert_eq!(ast.node_views().filter_map(|v| v.as_computation_option()).count(), 3);
    let after = text.find("set after_contextual_recovery = sentinel;").unwrap();
    assert!(ast.node_views().filter_map(|v| v.as_set_statement()).any(|v| v.range() == range(ast.source_id, after, text.trim_end().len()) && !v.is_recovered()));
    let first = super::super::run_parse_only_case(&config.workspace_root, &config.workspace_root.join("tests"), case, 166);
    assert_eq!(first.status, super::super::ParseOnlyCaseStatus::Passed, "{first:?}");
    for (tail, codes) in [("inspect(x = x;", vec!["malformed_justification", "malformed_term_expression"]), ("inspect(x = x) steps: 2);", vec!["malformed_justification"])] {
        let source = format!("thesis by computation({tail} set after_delimiter = sentinel;");
        let output = super::super::formula_statement::step5c8_test_frontend(&source);
        assert_eq!(output.diagnostics.iter().map(|v| v.code.clone()).collect::<Vec<_>>(), codes.into_iter().map(|v| DiagnosticCode::Syntax(v.into())).collect::<Vec<_>>());
        let ast = output.ast.unwrap();
        let owner = ast.node_views().filter_map(|v| v.as_computation_justification()).next().unwrap();
        let call = owner.child_views().find(|v| v.kind() == &K::ApplicationTerm).unwrap();
        let input = call.child_views().find(|v| v.kind() == &K::FormulaExpression).unwrap();
        assert_eq!(&source[input.range().start..input.range().end], "x = x");
        let boundary = source.find(if tail.contains("steps") { "steps" } else { ";" }).unwrap();
        for diagnostic in &output.diagnostics {
            let width = if tail.contains("steps") { 5 } else { 1 };
            assert_eq!(diagnostic.location, DiagnosticLocation::SourceRange(range(ast.source_id, boundary, boundary + width)));
        }
        if tail.contains("steps") {
            assert_eq!(call.child_views().last().unwrap().as_token().unwrap().text.as_ref(), ")");
            assert_eq!(owner.child_views().last().unwrap().as_token().unwrap().text.as_ref(), ")");
            let skipped = owner.child_views().find(|v| v.as_recovery() == Some(SyntaxRecoveryKind::SkippedToken)).unwrap();
            assert_eq!(&source[skipped.range().start..skipped.range().end], "steps: 2");
        } else {
            for close in [call.child_views().last().unwrap(), owner.child_views().last().unwrap()] {
                assert_eq!(close.as_recovery(), Some(SyntaxRecoveryKind::UnmatchedOpeningDelimiter));
                assert_eq!(close.range(), range(ast.source_id, boundary, boundary));
            }
        }
        let after = source.find("set after_delimiter = sentinel;").unwrap();
        assert!(ast.node_views().filter_map(|v| v.as_set_statement()).any(|v| v.range() == range(ast.source_id, after, source.len()) && !v.is_recovered()));
    }
    for raw in ["x = x", "thesis"] {
        let source = format!("thesis by computation(copy_term(nested({raw}))); set after_ordinary = sentinel;");
        let output = super::super::formula_statement::step5c8_test_frontend(&source);
        assert_eq!(output.diagnostics.len(), 1, "{raw}: {:?}", output.diagnostics);
        assert_eq!(output.diagnostics[0].code, DiagnosticCode::Syntax("malformed_term_expression".into()));
        let ast = output.ast.unwrap();
        let nested = ast.node_views().find(|v| v.kind() == &K::ApplicationTerm && v.range().start == source.find("nested(").unwrap()).unwrap();
        assert!(!ast.node_views().any(|v| v.kind() == &K::FormulaExpression && v.range().start >= nested.range().start && v.range().end <= nested.range().end));
        assert_eq!(nested.child_views().last().unwrap().as_token().unwrap().text.as_ref(), ")");
        let after = source.find("set after_ordinary = sentinel;").unwrap();
        assert!(ast.node_views().filter_map(|v| v.as_set_statement()).any(|v| v.range() == range(ast.source_id, after, source.len()) && !v.is_recovered()));
    }
}

#[test]
fn computation_contextual_calls_keep_actual_template_endpoint_and_recovery() {
    use mizar_syntax::{SurfaceNodeKind as K, SyntaxRecoveryKind as R};
    for call in ["calc[not](x)", "calc[](x)"] {
        let text = format!("thesis by computation({call}); set after_template_call = sentinel;");
        let output = super::super::formula_statement::step5c8_test_frontend(&text);
        assert!(!output.diagnostics.is_empty());
        let ast = output.ast.as_ref().unwrap();
        let start = text.find(call).unwrap();
        let owner = ast.node_views().find(|v| v.as_computation_justification().is_some()).unwrap();
        let payload = owner.child_views().find(|v| matches!(v.kind(), K::TermReference | K::ApplicationTerm)).unwrap();
        if call.contains("not") {
            let not = text.find("not").unwrap();
            assert_eq!(payload.kind(), &K::TermReference);
            assert_eq!(payload.range(), range(ast.source_id, start, not));
            assert!(!ast.node_views().any(|v| v.as_application_term().is_some()));
            assert!(ast.node_views().any(|v| v.as_recovery() == Some(R::SkippedToken)
                && v.range() == range(ast.source_id, not, not + 3)));
        } else {
            assert_eq!(payload.kind(), &K::ApplicationTerm);
            let argument = payload.child_views().find(|v| v.kind() == &K::TermExpression).unwrap();
            assert_eq!(&text[argument.range().start..argument.range().end], "x");
            let missing = text.find(']').unwrap();
            assert!(ast.node_view(ast.root().unwrap()).unwrap().child_views().any(|v|
                v.as_recovery() == Some(R::MissingTypeExpression)
                    && v.range() == range(ast.source_id, missing, missing)));
            assert!(ast.node_views().filter(|v| v.kind() == &K::TemplateArgument).any(|v|
                v.child_views().any(|v| v.as_recovery() == Some(R::MissingTypeExpression)
                    && v.range() == range(ast.source_id, missing, missing))));
        }
        let after = text.find("set after_template_call").unwrap();
        assert!(ast.node_views().any(|v| v.as_set_statement().is_some()
            && v.range() == range(ast.source_id, after, text.len())));
    }
}
