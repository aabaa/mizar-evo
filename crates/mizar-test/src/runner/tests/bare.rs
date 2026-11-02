use super::*;

#[test]
fn bare_algorithm_fail_source_projection_and_annotation_hosts() {
    use mizar_syntax::{SurfaceTokenKind, SyntaxKind};
    let text = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/miz/pass/parser/pass_parser_algorithm_fail_bare_001.miz"
    ));
    let annotated = text.replace("fail;", "@trace(\"stop\", 1)\n    fail;");
    let kind = SyntaxKind::FailStatement;
    assert_eq!(kind as u16, 194);
    assert_eq!(SyntaxKind::from_raw(194), kind);
    assert!(kind.is_node_kind() && !kind.is_token_kind());
    for text in [text, annotated.as_str()] {
        let output = super::super::formula_statement::step5c8_test_frontend(text);
        assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
        let ast = output.ast.unwrap();
        let views = ast
            .node_views()
            .filter_map(|v| v.as_fail_statement())
            .collect::<Vec<_>>();
        assert_eq!(views.len(), 4);
        for (view, (start, _)) in views.iter().zip(text.match_indices("fail;")) {
            assert_eq!(view.range(), range(ast.source_id, start, start + 5));
            assert_eq!(view.syntax_kind(), kind);
            assert!(!view.is_recovered());
            let children = view.children();
            assert_eq!(children.len(), 2);
            for (id, word) in children.iter().zip(["fail", ";"]) {
                let token = ast.node_view(*id).unwrap().as_token().unwrap();
                assert_eq!(token.text.as_ref(), word);
                if word == "fail" {
                    assert_eq!(token.kind, SurfaceTokenKind::ReservedWord);
                }
            }
        }
        let rowan = ast.rowan_root();
        assert_eq!(rowan.descendants().filter(|n| n.kind() == kind).count(), 4);
        assert_eq!(ast.snapshot_text().matches("FailStatement").count(), 4);
        assert!(
            ast.node_views()
                .all(|v| v.as_fail_statement().is_some() == (v.syntax_kind() == kind))
        );
        if text == annotated {
            let hosts = ast
                .node_views()
                .filter_map(|v| v.as_annotated_algorithm_statement())
                .collect::<Vec<_>>();
            assert_eq!(hosts.len(), 4);
            for host in hosts {
                let mut children = host.children().iter().filter_map(|id| ast.node_view(*id));
                assert!(children.any(|v| v.as_fail_statement().is_some()));
            }
        }
    }
}

#[test]
fn bare_algorithm_fail_frontend_storage_cache_and_source_rebinding() {
    use mizar_frontend::orchestration::{DiagnosticCode, DiagnosticLocation, FrontendOutput};
    use mizar_syntax::SyntaxRecoveryKind;
    let plan = step5c11_plan();
    let cases = [
        "pass_parser_algorithm_fail_bare_001",
        "fail_parser_algorithm_fail_semicolon_001",
        "fail_parser_algorithm_fail_placement_001",
    ];
    for (ordinal, id) in cases.iter().enumerate() {
        let case = plan.cases.iter().find(|case| case.id.0 == *id).unwrap();
        let text = std::fs::read_to_string(&case.source_path).unwrap();
        if id.starts_with("fail_") {
            let checked = super::super::formula_statement::step5c8_test_frontend(&text);
            let syntax = checked
                .diagnostics
                .iter()
                .filter(|d| matches!(d.code, DiagnosticCode::Syntax(_)))
                .collect::<Vec<_>>();
            let semicolon = id.contains("semicolon");
            let expected = if semicolon {
                vec!["missing_semicolon", "missing_semicolon"]
            } else {
                vec!["unexpected_top_level_token"]
            };
            let actual = syntax
                .iter()
                .map(|d| match &d.code {
                    DiagnosticCode::Syntax(c) => c.to_string(),
                    _ => unreachable!(),
                })
                .collect::<Vec<_>>();
            assert_eq!(actual, expected);
            let ast = checked.ast.unwrap();
            let boundaries = if semicolon {
                vec![
                    (text.find("else").unwrap(), 4),
                    (text.find("end;\n    return").unwrap(), 3),
                ]
            } else {
                vec![(0, 4)]
            };
            let note = if semicolon {
                "insert `;` before continuing"
            } else {
                "skip tokens until the next top-level item boundary"
            };
            for (d, (start, width)) in syntax.iter().zip(boundaries) {
                let expected_range = range(ast.source_id, start, start + width);
                assert_eq!(d.location, DiagnosticLocation::SourceRange(expected_range));
                assert!(d.secondary.is_empty());
                assert_eq!(d.recovery_note.as_deref(), Some(note));
            }
            let returns = ast
                .node_views()
                .filter_map(|v| v.as_return_statement())
                .collect::<Vec<_>>();
            assert_eq!(returns.len(), if semicolon { 2 } else { 1 });
            for (view, (start, _)) in returns.iter().zip(text.match_indices("return;")) {
                assert_eq!(view.range(), range(ast.source_id, start, start + 7));
            }
            assert_eq!(
                ast.node_views()
                    .filter_map(|v| v.as_algorithm_definition())
                    .count(),
                1
            );
            let failures = ast
                .node_views()
                .filter_map(|v| v.as_fail_statement())
                .count();
            assert_eq!(failures, if semicolon { 2 } else { 0 });
            if !semicolon {
                let skipped = ast.trivia().skipped_token_ranges();
                assert_eq!(skipped.len(), 1);
                assert_eq!(skipped[0].range, range(ast.source_id, 0, 5));
                let recovery = ast
                    .node_views()
                    .find(|v| v.as_recovery() == Some(SyntaxRecoveryKind::SkippedToken))
                    .unwrap();
                assert_eq!(recovery.range(), skipped[0].range);
            }
        }
        let output = task257c4c1_frontend_output(&text, 80 + ordinal);
        let replay = task257c4c1_frontend_output(&text, 80 + ordinal);
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
        let ids = InMemorySessionIdAllocator::new();
        let snapshot = super::super::shared::snapshot_id(25_780 + ordinal);
        let mut fresh_id = ids.next_source_id(snapshot).unwrap();
        if fresh_id == source.source_id {
            fresh_id = ids.next_source_id(snapshot).unwrap();
        }
        assert_ne!(fresh_id, source.source_id);
        let bytes = output.canonical_disk_bytes().unwrap();
        let same =
            FrontendOutput::from_canonical_disk_bytes(&bytes, source.source_id, &input).unwrap();
        assert_eq!(same, output);
        let rebound = FrontendOutput::from_canonical_disk_bytes(&bytes, fresh_id, &input).unwrap();
        assert!(rebound.source.matches_canonical_storage(source, &input));
        assert_eq!(rebound.canonical_disk_bytes().unwrap(), bytes);
        assert_eq!(rebound.cache_keys, output.cache_keys);
        assert_eq!(rebound.tokens.source_id, fresh_id);
        assert!(
            rebound
                .tokens
                .tokens
                .iter()
                .all(|t| t.span.source_id == fresh_id)
        );
        let ast = output.ast.as_ref().unwrap();
        let restored = rebound.ast.as_ref().unwrap();
        assert_eq!(restored.canonical_bytes(), ast.canonical_bytes());
        assert_eq!(restored.green_node(), ast.green_node());
        for (before, after) in ast.nodes().iter().zip(restored.nodes()) {
            assert_eq!(
                after.range,
                range(fresh_id, before.range.start, before.range.end)
            );
        }
        for (before, after) in output.diagnostics.iter().zip(&rebound.diagnostics) {
            let DiagnosticLocation::SourceRange(original) = before.location else {
                panic!("source range")
            };
            let expected = range(fresh_id, original.start, original.end);
            assert_eq!(after.location, DiagnosticLocation::SourceRange(expected));
        }
        let key = output.cache_keys.ast.as_ref().unwrap();
        assert_eq!(
            key.parser_version.version.as_ref(),
            "mizar-parser/surface-ast-v30"
        );
    }
}

#[test]
fn bare_attribute_chains_preserve_explicit_endings_terms_boundaries_and_heads() {
    use mizar_syntax::{SurfaceNodeKind as K, SyntaxKind as S};
    let plan = build_test_plan(&step5c11_config()).unwrap();
    let case = plan.cases.iter().find(|case| case.id.0 == "pass_parser_bare_attribute_chain_001").unwrap();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let output = super::super::formula_statement::step5c8_test_frontend(&text);
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    let ast = output.ast.unwrap();
    assert!(ast.node_views().all(|v| !v.is_recovered()));
    assert!(ast.trivia().skipped_token_ranges().is_empty());
    for (name, expected) in [
        ("LowerSimple", [("tagged(x)", "x"), ("linked(y,x)", "y\nx")].as_slice()),
        ("LowerComplex", &[("tagged(Atom ++ Other)", "Atom ++ Other"), ("linked(~Other,x)", "~Other\nx")]),
        ("UpperSimple", &[("TaggedUpper(x)", "x")]),
        ("UpperMixed", &[("TaggedUpper(~Atom)", "~Atom"), ("empty", ""), ("non LinkedUpper(Atom !,x)", "Atom !\nx")]),
        ("PrefixLast", &[("TaggedUpper(~Atom)", "~Atom"), ("(x,y)-linked", "")]),
        ("NonLast", &[("tagged(~Atom)", "~Atom"), ("non empty", "")]),
        ("NumeralPrefixLast", &[("tagged(~Atom)", "~Atom"), ("0-TaggedUpper", "")]),
        ("BinderLast", &[("tagged(x |-> Atom)", "x |-> Atom"), ("linked((x,y) |-> Other,x)", "(x,y) |-> Other\nx")]),
        ("NestedLast", &[("tagged(image([x,{y}]))", "image([x,{y}])"), ("LinkedUpper(Box[x](value:p.value),the set)", "Box[x](value:p.value)\nthe set")]),
        ("QuaCollectionLast", &[("tagged(x qua set)", "x qua set"), ("linked({x where x is set:thesis},y)", "{x where x is set:thesis}\ny")]),
        ("AndBoundary", &[("Box.TaggedUpper(~Atom)", "~Atom"), ("linked(~Other,x)", "~Other\nx")]),
        ("GroupOrBoundary", &[("TaggedUpper(~Atom)", "~Atom"), ("linked(~Other,x)", "~Other\nx")]),
        ("ProofBoundary", &[("TaggedUpper(~Atom)", "~Atom"), ("linked(~Other,x)", "~Other\nx")]),
    ] {
        let host_start = text.find(&format!("theorem {name}:")).unwrap();
        let start = host_start + text[host_start..].find(expected[0].0).unwrap();
        let spelling = expected.iter().map(|(body, _)| *body).collect::<Vec<_>>().join(" ");
        let chain = ast.node_views().find(|v| v.kind() == &K::AttributeTestChain && v.range().start == start).unwrap();
        assert_eq!(chain.range(), range(ast.source_id, start, start + spelling.len()), "{name}");
        assert_eq!(&text[chain.range().start..chain.range().end], spelling);
        assert_eq!(chain.children().len(), expected.len());
        let assertion = ast.node_views().find(|v| v.kind() == &K::IsAssertion && v.children().last() == Some(&chain.id())).unwrap();
        let subject = ast.node_view(assertion.children()[0]).unwrap();
        assert_eq!(subject.kind(), &K::TermExpression);
        assert_eq!(&text[subject.range().start..subject.range().end], "x");
        assert_eq!(ast.node_view(assertion.children()[1]).unwrap().as_token().unwrap().text.as_ref(), "is");
        assert_eq!(assertion.children().len(), if name == "AndBoundary" { 4 } else { 3 });
        assert_eq!(assertion.range().end, chain.range().end);
        if name == "AndBoundary" { assert_eq!(ast.node_view(assertion.children()[2]).unwrap().as_token().unwrap().text.as_ref(), "not"); }
        let mut cursor = start;
        for (id, (body, expected_arguments)) in chain.children().iter().zip(expected) {
            let attribute = ast.node_view(*id).unwrap();
            assert_eq!(attribute.kind(), &K::AttributeRef);
            assert_eq!(attribute.range(), range(ast.source_id, cursor, cursor + body.len()));
            let children = attribute.children().iter().map(|id| ast.node_view(*id).unwrap()).collect::<Vec<_>>();
            let arguments = children.iter().filter(|v| v.kind() == &K::TermExpression).collect::<Vec<_>>();
            assert_eq!(arguments.iter().map(|v| &text[v.range().start..v.range().end]).collect::<Vec<_>>(), expected_arguments.lines().collect::<Vec<_>>(), "{name}: {body}");
            let symbol = children.iter().find(|v| v.kind() == &K::QualifiedSymbol).unwrap();
            if body.starts_with("non ") { assert_eq!(children[0].as_token().unwrap().text.as_ref(), "non"); }
            if *body == "(x,y)-linked" || *body == "0-TaggedUpper" {
                let prefix = &children[0];
                assert_eq!(prefix.kind(), &K::ParameterPrefix);
                let expected_prefix = if body.starts_with('(') { "(x,y)-" } else { "0-" };
                assert_eq!(prefix.range(), range(ast.source_id, cursor, cursor + expected_prefix.len()));
                assert_eq!(&text[symbol.range().start..symbol.range().end], if body.starts_with('(') { "linked" } else { "TaggedUpper" });
            }
            if body.starts_with("Box.") {
                assert_eq!(symbol.children().len(), 3);
                assert_eq!(&text[symbol.range().start..symbol.range().end], "Box.TaggedUpper");
                assert_eq!(ast.node_view(symbol.children()[1]).unwrap().as_token().unwrap().text.as_ref(), ".");
            }
            if !arguments.is_empty() {
                let open = children.iter().find(|v| v.as_token().is_some_and(|t| t.text.as_ref() == "(")).unwrap();
                assert_eq!(open.range(), range(ast.source_id, symbol.range().end, symbol.range().end + 1));
                let close = children.last().unwrap();
                assert_eq!(close.as_token().unwrap().text.as_ref(), ")");
                assert_eq!(close.range(), range(ast.source_id, attribute.range().end - 1, attribute.range().end));
                let commas = children.iter().filter(|v| v.as_token().is_some_and(|t| t.text.as_ref() == ",")).collect::<Vec<_>>();
                assert_eq!(commas.len(), arguments.len() - 1);
                for (comma, pair) in commas.iter().zip(arguments.windows(2)) {
                    assert_eq!(comma.range(), range(ast.source_id, pair[0].range().end, pair[0].range().end + 1));
                    assert_eq!(comma.range().end, pair[1].range().start);
                }
                for argument in arguments {
                    let term = ast.node_view(argument.children()[0]).unwrap();
                    let body = &text[argument.range().start..argument.range().end];
                    let kind = match body {
                        "~Atom" | "~Other" => S::PrefixExpression, "Atom !" => S::PostfixExpression, "Atom ++ Other" => S::InfixExpression,
                        "x |-> Atom" | "(x,y) |-> Other" => S::BinderExpression,
                        "image([x,{y}])" => S::ApplicationTerm, "Box[x](value:p.value)" => S::StructureConstructor,
                        "the set" => S::ChoiceTerm, "x qua set" => S::QuaExpression,
                        "{x where x is set:thesis}" => S::SetComprehension, _ => S::TermReference,
                    };
                    assert_eq!(term.syntax_kind(), kind, "{name}: {body}");
                    if let Some(operator) = term.as_prefix_expression() { assert_eq!((operator.spelling.as_ref(), operator.precedence), ("~", 85)); }
                    if let Some(operator) = term.as_postfix_expression() { assert_eq!((operator.spelling.as_ref(), operator.precedence), ("!", 95)); }
                    if let Some(operator) = term.as_infix_expression() { assert_eq!((operator.spelling.as_ref(), operator.precedence), ("++", 80)); }
                    if kind == S::BinderExpression {
                        let binder_body = ast.node_view(*term.children().last().unwrap()).unwrap();
                        assert_eq!(&text[binder_body.range().start..binder_body.range().end], if body.starts_with('(') { "Other" } else { "Atom" });
                    }
                }
            }
            cursor += body.len() + 1;
        }
        for (kind, source) in [(K::ApplicationTerm, "[x,{y}]"), (K::SetEnumeration, "{y}"), (K::TypeArguments, "[x]"), (K::FieldArgument, "value:p.value"), (K::SelectorAccess, "p.value")] {
            if name == "NestedLast" { assert!(ast.node_views().any(|v| v.kind() == &kind && v.range().start >= start && v.range().end <= chain.range().end && &text[v.range().start..v.range().end] == source)); }
        }
        if name == "AndBoundary" || name == "GroupOrBoundary" {
            let connective = if name == "AndBoundary" { "&" } else { "or" };
            let boundary = chain.range().end + if name == "AndBoundary" { 1 } else { 2 };
            assert!(ast.token_views().any(|v| v.range() == range(ast.source_id, boundary, boundary + connective.len()) && v.as_token().unwrap().text.as_ref() == connective));
            assert!(ast.node_views().any(|v| v.kind() == &K::FormulaConstant(mizar_syntax::SurfaceFormulaConstant::Thesis) && v.range().start == boundary + connective.len() + 1));
        }
        if name == "GroupOrBoundary" {
            let group = ast.node_views().find(|v| v.kind() == &K::ParenthesizedFormula && v.range().start == assertion.range().start - 1).unwrap();
            assert_eq!(group.range().end, chain.range().end + 1);
            assert_eq!(ast.node_view(*group.children().last().unwrap()).unwrap().as_token().unwrap().text.as_ref(), ")");
        }
        if name == "ProofBoundary" {
            let proof = ast.node_views().find(|v| v.kind() == &K::ProofBlock && v.range().start == chain.range().end + 1).unwrap();
            assert_eq!(&text[proof.range().start..proof.range().end], "proof thus thesis; end");
        }
    }
    for (name, adjectives, head) in [
        ("TypedRightmost", "TaggedUpper(~Atom) empty linked(~Other,x)", "R"),
        ("TypedBracket", "TaggedUpper(~Atom)", "Family[x]"),
        ("TypedOf", "tagged(~Atom)", "Family of x"),
        ("TypedOver", "TaggedUpper(~Atom)", "Family over x"),
        ("AmbiguousUpper", "tagged(~Atom)", "TaggedUpper"),
        ("AmbiguousEmpty", "TaggedUpper(~Atom)", "empty"),
    ] {
        let host_start = text.find(&format!("theorem {name}:")).unwrap();
        let start = host_start + text[host_start..].find(adjectives).unwrap();
        let expression = ast.node_views().find(|v| v.kind() == &K::TypeExpression && v.range().start == start).unwrap();
        assert_eq!(expression.children().len(), 2);
        let chain = ast.node_view(expression.children()[0]).unwrap();
        let actual_head = ast.node_view(expression.children()[1]).unwrap();
        assert_eq!(chain.kind(), &K::AttributeChain);
        assert_eq!(chain.range(), range(ast.source_id, start, start + adjectives.len()));
        assert_eq!(chain.children().len(), if name == "TypedRightmost" { 3 } else { 1 });
        assert_eq!(actual_head.kind(), &K::TypeHead);
        assert_eq!(&text[actual_head.range().start..actual_head.range().end], head);
        assert_eq!(actual_head.range(), range(ast.source_id, start + adjectives.len() + 1, start + adjectives.len() + 1 + head.len()));
        assert!(ast.node_views().any(|v| v.kind() == &K::IsAssertion && v.children().last() == Some(&expression.id())));
    }
    for host in ast.node_views().filter(|v| v.kind() == &K::TheoremItem) {
        let mut pending = vec![host.id()];
        let mut seen = std::collections::HashSet::new();
        let mut tokens = Vec::new();
        while let Some(id) = pending.pop() {
            assert!(seen.insert(id), "duplicate child in bare chain theorem");
            let node = ast.node_view(id).unwrap();
            if node.as_token().is_some() { tokens.push(id); }
            pending.extend(node.children().iter().rev().copied());
        }
        assert_eq!(tokens, ast.token_views().filter(|v| v.range().start >= host.range().start && v.range().end <= host.range().end).map(|v| v.id()).collect::<Vec<_>>());
        assert_eq!(ast.node_view(*host.children().last().unwrap()).unwrap().as_token().unwrap().text.as_ref(), ";");
    }
    let after = text.find("set after_bare_attribute_chain = sentinel;").unwrap();
    assert!(ast.node_views().filter_map(|v| v.as_set_statement()).any(|v| v.range() == range(ast.source_id, after, text.trim_end().len()) && !v.is_recovered()));
    let frontend = task257c4c1_frontend_output(&text, 131);
    let current = frontend.cache_keys.ast.as_ref().unwrap();
    assert_eq!(current.parser_version.version.as_ref(), "mizar-parser/surface-ast-v30");
    let mut retained = current.clone();
    retained.parser_version = mizar_frontend::parsing::ParserCacheKeyVersion::new("mizar-parser/surface-ast-v21");
    assert_eq!(current.token_stream_hash, retained.token_stream_hash);
    assert_eq!(current.parser_inputs_hash, retained.parser_inputs_hash);
    assert_ne!(current.stable_hash(), retained.stable_hash());
    let bytes = ast.canonical_bytes().expect("valid explicit bare chain storage");
    let restored = mizar_syntax::SurfaceAst::from_canonical_bytes(&bytes, ast.source_id).unwrap();
    assert_eq!(restored.snapshot_text_with_trivia(), ast.snapshot_text_with_trivia());
}

#[test]
fn bare_attribute_chain_recovery_preserves_complete_lists_and_partial_missing_comma() {
    use mizar_frontend::orchestration::{DiagnosticCode, DiagnosticLocation};
    use mizar_syntax::SurfaceNodeKind as K;
    let plan = build_test_plan(&step5c11_config()).unwrap();
    let case = plan.cases.iter().find(|case| case.id.0 == "fail_parser_bare_attribute_chain_001").unwrap();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let output = super::super::formula_statement::step5c8_test_frontend(&text);
    assert_eq!(output.diagnostics.len(), 6, "{:?}", output.diagnostics);
    let ast = output.ast.unwrap();
    let empty_close = text.find("TaggedUpper()").unwrap() + "TaggedUpper(".len();
    let trailing_close = text.find("TaggedUpper(~Atom,)").unwrap() + "TaggedUpper(~Atom,".len();
    let partial_start = text.find("TaggedUpper(x R)").unwrap();
    let radix = partial_start + "TaggedUpper(x ".len();
    let stray_close = radix + 1;
    for (diagnostic, (code, offset)) in output.diagnostics.iter().zip([
        ("malformed_term_expression", empty_close), ("malformed_term_expression", trailing_close),
        ("malformed_term_expression", radix), ("malformed_term_expression", stray_close),
        ("missing_semicolon", stray_close), ("unexpected_top_level_token", stray_close),
    ]) {
        assert_eq!(diagnostic.code, DiagnosticCode::Syntax(code.into()));
        assert_eq!(diagnostic.location, DiagnosticLocation::SourceRange(range(ast.source_id, offset, offset + 1)));
    }
    assert_eq!(output.diagnostics[2].secondary, vec![mizar_session::SourceAnchor::Range(range(ast.source_id, partial_start + "TaggedUpper".len(), partial_start + "TaggedUpper(".len()))]);
    for (name, first, close, arguments) in [
        ("EmptyArguments", "TaggedUpper()", empty_close, ""),
        ("TrailingComma", "TaggedUpper(~Atom,)", trailing_close, "~Atom"),
    ] {
        let start = text.find(first).unwrap();
        let chain = ast.node_views().find(|v| v.kind() == &K::AttributeTestChain && v.range().start == start).unwrap();
        assert_eq!(chain.children().len(), 2);
        assert_eq!(&text[chain.range().start..chain.range().end], format!("{first} linked(~Other,x)"));
        let attribute = ast.node_view(chain.children()[0]).unwrap();
        assert_eq!(attribute.kind(), &K::AttributeRef);
        assert_eq!(attribute.range(), range(ast.source_id, start, close + 1));
        let children = attribute.children().iter().map(|id| ast.node_view(*id).unwrap()).collect::<Vec<_>>();
        let missing = children.iter().filter(|v| v.as_recovery() == Some(SyntaxRecoveryKind::MissingTerm)).collect::<Vec<_>>();
        assert_eq!(missing.len(), 1);
        assert_eq!(missing[0].range(), range(ast.source_id, close, close));
        assert!(missing[0].is_recovered());
        assert_eq!(children.iter().filter(|v| v.kind() == &K::TermExpression).map(|v| &text[v.range().start..v.range().end]).collect::<Vec<_>>(), arguments.lines().collect::<Vec<_>>());
        assert_eq!(children.last().unwrap().as_token().unwrap().text.as_ref(), ")");
        assert_eq!(children.last().unwrap().range(), range(ast.source_id, close, close + 1));
        let second = ast.node_view(chain.children()[1]).unwrap();
        assert_eq!(second.kind(), &K::AttributeRef);
        assert_eq!(&text[second.range().start..second.range().end], "linked(~Other,x)");
        assert_eq!(second.children().iter().map(|id| ast.node_view(*id).unwrap()).filter(|v| v.kind() == &K::TermExpression).map(|v| &text[v.range().start..v.range().end]).collect::<Vec<_>>(), ["~Other", "x"]);
        assert_eq!(ast.node_view(*second.children().last().unwrap()).unwrap().as_token().unwrap().text.as_ref(), ")");
        assert!(ast.node_views().any(|v| v.kind() == &K::IsAssertion && v.children().last() == Some(&chain.id())));
        let host_start = text.find(&format!("theorem {name}:")).unwrap();
        let host = ast.node_views().find(|v| v.kind() == &K::TheoremItem && v.range().start == host_start).unwrap();
        assert_eq!(host.range().end, chain.range().end + 1);
        let mut pending = vec![host.id()];
        let mut seen = std::collections::HashSet::new();
        let mut tokens = Vec::new();
        while let Some(id) = pending.pop() {
            assert!(seen.insert(id), "duplicate recovered chain child");
            let node = ast.node_view(id).unwrap();
            if node.as_token().is_some() { tokens.push(id); }
            pending.extend(node.children().iter().rev().copied());
        }
        assert_eq!(tokens, ast.token_views().filter(|v| v.range().start >= host.range().start && v.range().end <= host.range().end).map(|v| v.id()).collect::<Vec<_>>());
    }
    let chain = ast.node_views().find(|v| v.kind() == &K::AttributeTestChain && v.range().start == partial_start).unwrap();
    assert_eq!(chain.range(), range(ast.source_id, partial_start, stray_close));
    assert_eq!(chain.children().len(), 2);
    let first = ast.node_view(chain.children()[0]).unwrap();
    assert_eq!(first.range(), range(ast.source_id, partial_start, radix - 1));
    assert_eq!(&text[first.range().start..first.range().end], "TaggedUpper(x");
    let term = first.children().iter().map(|id| ast.node_view(*id).unwrap()).find(|v| v.kind() == &K::TermExpression).unwrap();
    assert_eq!(&text[term.range().start..term.range().end], "x");
    let unmatched = ast.node_view(*first.children().last().unwrap()).unwrap();
    assert_eq!(unmatched.as_recovery(), Some(SyntaxRecoveryKind::UnmatchedOpeningDelimiter));
    assert_eq!(unmatched.range(), range(ast.source_id, radix, radix));
    let second = ast.node_view(chain.children()[1]).unwrap();
    assert_eq!(second.kind(), &K::AttributeRef);
    assert_eq!(second.range(), range(ast.source_id, radix, stray_close));
    assert_eq!(second.children().len(), 1);
    assert_eq!(ast.node_view(second.children()[0]).unwrap().kind(), &K::QualifiedSymbol);
    assert_eq!(&text[second.range().start..second.range().end], "R");
    let assertion = ast.node_views().find(|v| v.kind() == &K::IsAssertion && v.children().last() == Some(&chain.id())).unwrap();
    assert_eq!(assertion.range().end, stray_close);
    let suffix = stray_close + ") ".len();
    assert_eq!(&text[suffix..suffix + "linked".len()], "linked");
    assert!(ast.node_views().filter(|v| v.kind() == &K::AttributeRef).all(|v| v.range().start != suffix));
    let skipped = ast.node_views().find(|v| v.as_recovery() == Some(SyntaxRecoveryKind::SkippedToken) && v.range().start == stray_close).unwrap();
    assert_eq!(skipped.range(), range(ast.source_id, stray_close, suffix + "linked".len()));
    assert_eq!(ast.trivia().skipped_token_ranges().len(), 1);
    assert_eq!(ast.trivia().skipped_token_ranges()[0].range, skipped.range());
    assert_eq!(ast.node_views().filter(|v| v.as_recovery() == Some(SyntaxRecoveryKind::MissingTerm)).count(), 2);
    assert!(ast.node_views().all(|v| v.as_recovery() != Some(SyntaxRecoveryKind::MissingTypeExpression)));
    for name in ["empty_arguments", "trailing_comma", "missing_comma"] {
        let successor = format!("set after_{name} = sentinel;");
        let start = text.find(&successor).unwrap();
        assert!(ast.node_views().filter_map(|v| v.as_set_statement()).any(|v| v.range() == range(ast.source_id, start, start + successor.len()) && !v.is_recovered()));
    }
    let bytes = ast.canonical_bytes().expect("valid bounded bare chain recovery storage");
    let restored = mizar_syntax::SurfaceAst::from_canonical_bytes(&bytes, ast.source_id).unwrap();
    assert_eq!(restored.snapshot_text_with_trivia(), ast.snapshot_text_with_trivia());
}
