use super::*;

#[test]
fn terminating_algorithm_labels_preserve_headers_wrappers_and_template_adjacency() {
    use mizar_frontend::lexing::TokenKind;
    use mizar_syntax::{SurfaceNodeKind as K, SyntaxKind as S};
    let config = step5c11_config();
    let plan = build_test_plan(&config).unwrap();
    let case = plan.cases.iter().find(|case| case.id.0 == "pass_parser_terminating_labels_001").unwrap();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let first = super::super::run_parse_only_case(&config.workspace_root, &config.workspace_root.join("tests"), case, 135);
    let repeated = super::super::run_parse_only_case(&config.workspace_root, &config.workspace_root.join("tests"), case, 135);
    assert_eq!(first.status, super::super::ParseOnlyCaseStatus::Passed, "{first:?}");
    assert_eq!(first, repeated);
    let output = super::super::formula_statement::step5c8_test_frontend(&text);
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    let ast = output.ast.unwrap();
    assert!(ast.node_views().all(|v| !v.is_recovered()));
    assert!(ast.trivia().skipped_token_ranges().is_empty());
    assert_eq!(ast.node_views().filter_map(|v| v.as_algorithm_definition()).count(), 6);
    assert_eq!(ast.node_views().filter(|v| v.kind() == &K::AlgorithmTerminationClause).count(), 5);
    assert_eq!(ast.node_views().filter(|v| v.kind() == &K::VisibleItem).count(), 2);
    assert_eq!(ast.rowan_root().descendants().filter(|n| n.kind() == S::AlgorithmDefinition).count(), 6);
    assert_eq!(ast.snapshot_text().matches("AlgorithmDefinition").count(), 6);
    for (label, name, visibility, expected) in [
        ("PublicDef", "public_alg", "public", ["terminating", "algorithm", "PublicDef", ":", "public_alg", "(x)", "->", "object", "do return x; end", ";"].as_slice()),
        ("PrivateDef", "hidden", "private", &["terminating", "algorithm", "PrivateDef", ":", "hidden", "()", "do end", ";"]),
        ("DefaultDef", "default_alg", "", &["terminating", "algorithm", "DefaultDef", ":", "default_alg", "()", "do end", ";"]),
        ("", "unlabelled", "", &["terminating", "algorithm", "unlabelled", "()", "do end", ";"]),
        ("", "ordinary", "", &["algorithm", "ordinary", "()", "do end", ";"]),
        ("TemplateDef", "templated", "", &["terminating", "algorithm", "TemplateDef", ":", "templated", "[T]", "()", "do end", ";"]),
    ] {
        let marker = if label.is_empty() { format!("algorithm {name}") } else { format!("algorithm {label}:") };
        let keyword = text.find(&marker).unwrap();
        let start = keyword - if name == "ordinary" { 0 } else { "terminating ".len() };
        let end = start + text[start..].find("end;").unwrap() + "end;".len();
        let algorithm = ast.node_views().filter_map(|v| v.as_algorithm_definition()).find(|v| v.range().start == start).unwrap();
        assert_eq!(algorithm.range(), range(ast.source_id, start, end));
        let children = algorithm.children().iter().map(|id| ast.node_view(*id).unwrap()).collect::<Vec<_>>();
        assert_eq!(children.iter().map(|v| &text[v.range().start..v.range().end]).collect::<Vec<_>>(), expected);
        let mut cursor = start;
        for (child, spelling) in children.iter().zip(expected) {
            let offset = cursor + text[cursor..end].find(spelling).unwrap();
            assert_eq!(child.range(), range(ast.source_id, offset, offset + spelling.len()));
            cursor = offset + spelling.len();
        }
        let keyword_index = usize::from(name != "ordinary");
        if keyword_index == 1 {
            assert_eq!(children[0].kind(), &K::AlgorithmTerminationClause);
            assert_eq!(children[0].children().len(), 1);
            let token = ast.node_view(children[0].children()[0]).unwrap();
            assert_eq!(token.as_token().unwrap().text.as_ref(), "terminating");
            assert_eq!(token.as_token().unwrap().kind, SurfaceTokenKind::ReservedWord);
            assert_eq!(token.range(), children[0].range());
        }
        assert_eq!(children[keyword_index].as_token().unwrap().kind, SurfaceTokenKind::ReservedWord);
        let name_index = keyword_index + if label.is_empty() { 1 } else { 3 };
        for index in [name_index].into_iter().chain((!label.is_empty()).then_some(keyword_index + 1)) {
            assert_eq!(children[index].as_token().unwrap().kind, SurfaceTokenKind::Identifier);
        }
        if !label.is_empty() {
            assert_eq!(children[keyword_index + 2].as_token().unwrap().kind, SurfaceTokenKind::ReservedSymbol);
            assert_eq!(children[keyword_index + 2].range().start, children[keyword_index + 1].range().end);
            assert!(children[keyword_index + 2].range().end < children[name_index].range().start);
        }
        if name == "templated" {
            let loci = &children[name_index + 1];
            assert_eq!(loci.kind(), &K::TemplateLoci);
            assert_eq!(loci.range().start, children[name_index].range().end);
            let parts = loci.children().iter().map(|id| ast.node_view(*id).unwrap()).collect::<Vec<_>>();
            assert_eq!(parts.len(), 3);
            assert_eq!(parts[0].as_token().unwrap().text.as_ref(), "[");
            assert_eq!(parts[1].kind(), &K::TemplateLocus);
            assert_eq!(parts[1].children().len(), 1);
            let token = ast.node_view(parts[1].children()[0]).unwrap();
            assert_eq!(token.as_token().unwrap().text.as_ref(), "T");
            assert_eq!(token.as_token().unwrap().kind, SurfaceTokenKind::Identifier);
            assert_eq!(parts[2].as_token().unwrap().text.as_ref(), "]");
            for (part, offset) in parts.iter().zip(0..3) {
                assert_eq!(part.range(), range(ast.source_id, loci.range().start + offset, loci.range().start + offset + 1));
            }
        }
        let parameters = children.iter().find(|v| v.kind() == &K::AlgorithmParameters).unwrap();
        assert_eq!(parameters.children().iter().map(|id| ast.node_view(*id).unwrap().as_token().unwrap().text.as_ref()).collect::<Vec<_>>(), if name == "public_alg" { vec!["(", "x", ")"] } else { vec!["(", ")"] });
        let body = children.iter().find(|v| v.kind() == &K::AlgorithmBody).unwrap();
        assert_eq!(body.children().len(), 3);
        assert_eq!(ast.node_view(body.children()[0]).unwrap().as_token().unwrap().text.as_ref(), "do");
        assert_eq!(ast.node_view(body.children()[2]).unwrap().as_token().unwrap().text.as_ref(), "end");
        let statements = ast.node_view(body.children()[1]).unwrap();
        assert_eq!(statements.kind(), &K::AlgorithmStatementList);
        assert_eq!(statements.children().len(), usize::from(name == "public_alg"));
        if name == "public_alg" {
            assert_eq!(children[name_index + 3].kind(), &K::TypeExpression);
            let statement = ast.node_view(statements.children()[0]).unwrap();
            assert_eq!(statement.kind(), &K::ReturnStatement);
            assert_eq!(&text[statement.range().start..statement.range().end], "return x;");
            assert_eq!(statement.children().len(), 3);
            let term = ast.node_view(statement.children()[1]).unwrap();
            assert_eq!(term.kind(), &K::TermExpression);
            let reference = ast.node_view(term.children()[0]).unwrap();
            assert_eq!(reference.kind(), &K::TermReference);
            let token = ast.node_view(reference.children()[0]).unwrap();
            assert_eq!(token.as_token().unwrap().text.as_ref(), "x");
            assert_eq!(token.as_token().unwrap().kind, SurfaceTokenKind::Identifier);
        }
        let wrappers = ast.node_views().filter(|v| v.kind() == &K::VisibleItem && v.children().contains(&algorithm.id())).collect::<Vec<_>>();
        assert_eq!(wrappers.len(), usize::from(!visibility.is_empty()));
        if !visibility.is_empty() {
            let wrapper = wrappers[0];
            assert_eq!(wrapper.range(), range(ast.source_id, start - visibility.len() - 1, end));
            assert_eq!(wrapper.children().len(), 2);
            assert_eq!(wrapper.children()[1], algorithm.id());
            let marker = ast.node_view(wrapper.children()[0]).unwrap();
            assert_eq!(marker.kind(), &K::VisibilityMarker);
            assert_eq!(marker.range(), range(ast.source_id, wrapper.range().start, start - 1));
            assert_eq!(marker.children().len(), 1);
            assert_eq!(ast.node_view(marker.children()[0]).unwrap().as_token().unwrap().text.as_ref(), visibility);
        }
    }
    let frontend = task257c4c1_frontend_output(&text, 135);
    assert!(frontend.diagnostics.is_empty(), "{:?}", frontend.diagnostics);
    assert!(frontend.tokens.tokens.iter().all(|t| t.kind != TokenKind::UserSymbol));
    for word in ["PublicDef", "public_alg", "PrivateDef", "hidden", "DefaultDef", "default_alg", "unlabelled", "ordinary", "TemplateDef", "templated"] {
        let tokens = frontend.tokens.tokens.iter().filter(|t| t.text.as_ref() == word).collect::<Vec<_>>();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::Identifier);
        let start = text.find(word).unwrap();
        assert_eq!(tokens[0].span, range(frontend.tokens.source_id, start, start + word.len()));
    }
    assert_eq!(frontend.ast.as_ref().unwrap().snapshot_text_with_trivia(), ast.snapshot_text_with_trivia());
    let repeated = task257c4c1_frontend_output(&text, 135);
    assert_eq!(frontend.cache_keys, repeated.cache_keys);
    assert_eq!(frontend.ast, repeated.ast);
    let current = frontend.cache_keys.ast.as_ref().unwrap();
    assert_eq!(current.parser_version.version.as_ref(), "mizar-parser/surface-ast-v30");
    let mut retained = current.clone();
    retained.parser_version = mizar_frontend::parsing::ParserCacheKeyVersion::new("mizar-parser/surface-ast-v23");
    assert_eq!(current.token_stream_hash, retained.token_stream_hash);
    assert_eq!(current.parser_inputs_hash, retained.parser_inputs_hash);
    assert_ne!(current.stable_hash(), retained.stable_hash());
    let bytes = ast.canonical_bytes().unwrap();
    let restored = mizar_syntax::SurfaceAst::from_canonical_bytes(&bytes, ast.source_id).unwrap();
    assert_eq!(restored.snapshot_text_with_trivia(), ast.snapshot_text_with_trivia());
}

#[test]
fn terminating_algorithm_label_recovery_keeps_partial_headers_and_successor_algorithms() {
    use mizar_frontend::orchestration::{DiagnosticCode, DiagnosticLocation};
    use mizar_syntax::SurfaceNodeKind as K;
    let config = step5c11_config();
    let plan = build_test_plan(&config).unwrap();
    let case = plan.cases.iter().find(|case| case.id.0 == "fail_parser_terminating_labels_001").unwrap();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let first = super::super::run_parse_only_case(&config.workspace_root, &config.workspace_root.join("tests"), case, 136);
    let repeated = super::super::run_parse_only_case(&config.workspace_root, &config.workspace_root.join("tests"), case, 136);
    assert_eq!(first.status, super::super::ParseOnlyCaseStatus::Passed, "{first:?}");
    assert_eq!(first, repeated);
    let output = super::super::formula_statement::step5c8_test_frontend(&text);
    assert_eq!(output.diagnostics.len(), 11, "{:?}", output.diagnostics);
    let ast = output.ast.unwrap();
    assert_eq!(ast.node_views().filter_map(|v| v.as_algorithm_definition()).count(), 8);
    assert_eq!(ast.node_views().filter(|v| v.as_recovery() == Some(SyntaxRecoveryKind::MissingTerm)).count(), 2);
    assert_eq!(ast.node_views().filter(|v| v.as_recovery() == Some(SyntaxRecoveryKind::UnmatchedOpeningDelimiter)).count(), 3);
    assert_eq!(ast.node_views().filter(|v| v.as_recovery() == Some(SyntaxRecoveryKind::SkippedToken)).count(), 3);
    assert_eq!(ast.node_views().filter_map(|v| v.as_recovery()).count(), 8);
    assert_eq!(ast.trivia().skipped_token_ranges().len(), 3);
    let mut diagnostics = output.diagnostics.iter();
    for (header, anchor_word, expected, codes, skipped, successor) in [
        ("algorithm BadNonterm: partial()", ":", ["algorithm", "BadNonterm", "", ": partial() do end", ";"].as_slice(), ["malformed_formula_expression", "malformed_formula_expression", "malformed_term_expression"].as_slice(), ": partial() do", "after_nonterminating_label"),
        ("terminating algorithm MissingName: ()", "(", &["terminating", "algorithm", "MissingName", ":", "", "()", "do end", ";"], &["malformed_term_expression"], "", "after_missing_name"),
        ("terminating algorithm : missing_label()", ":", &["terminating", "algorithm", "", "", ": missing_label() do end", ";"], &["malformed_formula_expression", "malformed_formula_expression", "malformed_term_expression", "malformed_term_expression"], ": missing_label() do", "after_missing_label"),
        ("terminating algorithm MissingColon adjacent()", "adjacent", &["terminating", "algorithm", "MissingColon", "", "adjacent() do end", ";"], &["malformed_formula_expression", "malformed_formula_expression", "malformed_term_expression"], "adjacent() do", "after_missing_colon"),
    ] {
        let start = text.find(header).unwrap();
        let anchor = start + header.find(anchor_word).unwrap();
        let end = start + text[start..].find("end;").unwrap() + "end;".len();
        for code in codes {
            let diagnostic = diagnostics.next().unwrap();
            assert_eq!(diagnostic.code, DiagnosticCode::Syntax((*code).into()));
            assert_eq!(diagnostic.location, DiagnosticLocation::SourceRange(range(ast.source_id, anchor, anchor + anchor_word.len())));
        }
        let algorithm = ast.node_views().filter_map(|v| v.as_algorithm_definition()).find(|v| v.range().start == start).unwrap();
        assert_eq!(algorithm.range(), range(ast.source_id, start, end));
        let children = algorithm.children().iter().map(|id| ast.node_view(*id).unwrap()).collect::<Vec<_>>();
        assert_eq!(children.iter().map(|v| &text[v.range().start..v.range().end]).collect::<Vec<_>>(), expected);
        let mut cursor = start;
        for (child, spelling) in children.iter().zip(expected) {
            let offset = if spelling.is_empty() { anchor } else { cursor + text[cursor..end].find(spelling).unwrap() };
            assert_eq!(child.range(), range(ast.source_id, offset, offset + spelling.len()));
            cursor = offset + spelling.len();
        }
        let missing = children.iter().filter(|v| v.as_recovery() == Some(SyntaxRecoveryKind::MissingTerm)).collect::<Vec<_>>();
        assert_eq!(missing.len(), usize::from(codes.len() == 1 || codes.len() == 4));
        for node in missing { assert_eq!(node.range(), range(ast.source_id, anchor, anchor)); }
        let parameters = children.iter().find(|v| v.kind() == &K::AlgorithmParameters).unwrap();
        let body = children.iter().find(|v| v.kind() == &K::AlgorithmBody).unwrap();
        let statements = body.children().iter().map(|id| ast.node_view(*id).unwrap()).find(|v| v.kind() == &K::AlgorithmStatementList).unwrap();
        if skipped.is_empty() {
            assert_eq!(parameters.range(), range(ast.source_id, anchor, anchor + 2));
            assert_eq!(parameters.children().iter().map(|id| ast.node_view(*id).unwrap().as_token().unwrap().text.as_ref()).collect::<Vec<_>>(), ["(", ")"]);
            assert_eq!(&text[body.range().start..body.range().end], "do end");
            assert_eq!(body.children().len(), 3);
            assert_eq!(ast.node_view(body.children()[0]).unwrap().as_token().unwrap().text.as_ref(), "do");
            assert!(statements.children().is_empty());
        } else {
            assert_eq!(parameters.range(), range(ast.source_id, anchor, anchor));
            assert_eq!(parameters.children().len(), 1);
            let recovery = ast.node_view(parameters.children()[0]).unwrap();
            assert_eq!(recovery.as_recovery(), Some(SyntaxRecoveryKind::UnmatchedOpeningDelimiter));
            assert_eq!(recovery.range(), range(ast.source_id, anchor, anchor));
            assert_eq!(body.range(), range(ast.source_id, anchor, end - 1));
            assert_eq!(body.children().len(), 2);
            assert_eq!(statements.children().len(), 1);
            let recovery = ast.node_view(statements.children()[0]).unwrap();
            assert_eq!(recovery.as_recovery(), Some(SyntaxRecoveryKind::SkippedToken));
            assert_eq!(recovery.range(), range(ast.source_id, anchor, anchor + skipped.len()));
            assert!(ast.trivia().skipped_token_ranges().iter().any(|v| v.range == recovery.range()));
            assert_eq!(recovery.children().iter().map(|id| ast.node_view(*id).unwrap().as_token().unwrap().text.as_ref()).collect::<String>(), skipped.replace(' ', ""));
        }
        let closing = ast.node_view(*body.children().last().unwrap()).unwrap();
        assert_eq!(closing.as_token().unwrap().text.as_ref(), "end");
        assert_eq!(closing.range(), range(ast.source_id, end - 4, end - 1));
        let next = format!("algorithm {successor}() do end;");
        let next_start = text.find(&next).unwrap();
        let successor = ast.node_views().filter_map(|v| v.as_algorithm_definition()).find(|v| v.range().start == next_start).unwrap();
        assert_eq!(successor.range(), range(ast.source_id, next_start, next_start + next.len()));
        assert!(!successor.is_recovered());
        let parts = successor.children().iter().map(|id| ast.node_view(*id).unwrap()).collect::<Vec<_>>();
        assert_eq!(parts.iter().map(|v| &text[v.range().start..v.range().end]).collect::<Vec<_>>(), ["algorithm", &next["algorithm ".len()..next.find('(').unwrap()], "()", "do end", ";"]);
        assert_eq!(parts[1].as_token().unwrap().kind, SurfaceTokenKind::Identifier);
        assert_eq!(parts[2].kind(), &K::AlgorithmParameters);
        assert_eq!(parts[3].kind(), &K::AlgorithmBody);
        assert_eq!(parts[4].as_token().unwrap().text.as_ref(), ";");
    }
    assert!(diagnostics.next().is_none());
    let bytes = ast.canonical_bytes().unwrap();
    let restored = mizar_syntax::SurfaceAst::from_canonical_bytes(&bytes, ast.source_id).unwrap();
    assert_eq!(restored.snapshot_text_with_trivia(), ast.snapshot_text_with_trivia());
}

#[test]
fn terminating_definition_labels_preserve_algorithm_identity_without_consumer_acceptance() {
    use mizar_artifact::module_summary::ModuleSummaryIdentity;
    use mizar_checker::type_checker::check_source_algorithm_types;
    use mizar_resolve::{
        env::{SignatureShell, SymbolKind, Visibility},
        names::{SourceVariableScopeError, SourceVariableScopeResolver},
        symbols::{source_algorithm_syntax_payloads_from_arena, source_public_algorithm_export_candidates},
    };
    use mizar_session::SourceAnchor;
    use mizar_syntax::{SurfaceNodeKind as K, SurfaceTokenKind as T};
    let original = "definition\n  let x be object;\n  terminating algorithm PublicDef: public_alg(x) -> object do return x; end;\nend;\n";
    let identity = ModuleSummaryIdentity {
        package_id: "label-test".into(),
        package_version: None,
        lockfile_identity: None,
        module_path: "owners".into(),
        language_edition: "2026".into(),
    };
    let config = step5c11_config();
    let case = step5c14_state_case();
    let mut baseline = None;
    for label in [None, Some("PublicDef"), Some("OtherDef")] {
        let text = match label {
            None => original.replace("PublicDef: ", ""),
            Some(label) => original.replace("PublicDef", label),
        };
        let (frontend, source, collected) =
            source_registration_owner_inputs(&text, "label-test", "owners", 0);
        assert!(frontend.diagnostics.is_empty(), "{:?}", frontend.diagnostics);
        assert!(collected.diagnostics().is_empty());
        let ast = frontend.ast.as_ref().unwrap();
        assert!(ast.node_views().all(|node| !node.is_recovered()));
        let env = collected.env();
        let owners = env.symbols().iter().filter(|entry| entry.kind() == SymbolKind::Algorithm).collect::<Vec<_>>();
        let [owner] = owners.as_slice() else { panic!("exactly one Algorithm owner for {text}: {:?}", env.symbols()) };
        assert_eq!((owner.kind(), owner.primary_spelling()), (SymbolKind::Algorithm, "public_alg"));
        assert_eq!(owner.visibility(), Visibility::Public);
        assert!(env.labels().is_empty());
        assert_eq!(owner.symbol().module(), source.module());
        assert_eq!(owner.origin().module_id(), source.module());
        assert_eq!(owner.origin().source_id(), ast.source_id);
        assert_eq!(source.source_id(), ast.source_id);
        let algorithm = ast.node_views().find(|node| node.kind() == &K::AlgorithmDefinition).unwrap();
        let start = text.find("terminating algorithm").unwrap();
        let end = text.find("end;").unwrap() + 4;
        assert_eq!((algorithm.range().start, algorithm.range().end), (start, end));
        assert_eq!(owner.origin().anchor(), &SourceAnchor::Range(algorithm.range()));
        let payloads = source_algorithm_syntax_payloads_from_arena(&source, env).unwrap();
        assert_eq!(payloads.len(), 1);
        let payload = payloads.get(owner.symbol()).unwrap();
        let signature = owner.signature().unwrap();
        if let Some(label) = label {
            let children = algorithm.child_views().collect::<Vec<_>>();
            assert_eq!(children[0].kind(), &K::AlgorithmTerminationClause);
            let label_start = text.find(label).unwrap();
            for (node, kind, spelling, offset) in [
                (children[2], T::Identifier, label, label_start),
                (children[3], T::ReservedSymbol, ":", label_start + label.len()),
                (children[4], T::Identifier, "public_alg", text.find("public_alg").unwrap()),
            ] {
                let token = node.as_token().unwrap();
                assert_eq!((token.kind, token.text.as_ref()), (kind, spelling));
                assert_eq!((node.range().start, node.range().end), (offset, offset + spelling.len()));
                assert_eq!(node.range().source_id, ast.source_id);
            }
            assert!(payload.windows(label.len()).any(|bytes| bytes == label.as_bytes()));
            assert!(matches!(signature, SignatureShell::Opaque { payload, .. } if payload.contains(label)));
            if let Some((symbol, previous_signature, previous_payload)) = &baseline {
                assert_eq!(owner.symbol(), symbol);
                assert_ne!(signature, previous_signature);
                assert_ne!(payload, previous_payload);
            } else {
                baseline = Some((owner.symbol().clone(), signature.clone(), payload.clone()));
            }
        }
        let variables = SourceVariableScopeResolver::resolve_algorithm_occurrences(&source, env);
        let exports = source_public_algorithm_export_candidates(&source, env, &identity).unwrap();
        if label.is_some() {
            assert_eq!(variables.unwrap_err(), SourceVariableScopeError::InvalidShape);
            assert_eq!(exports, None);
        } else {
            variables.unwrap();
            let rows = exports.unwrap();
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].declaration_kind, "algorithm");
            assert!(rows[0].rendered_signature.contains("public_alg"));
        }
        // This helper supplies a structural typed scaffold; only the actual check grants type credit.
        let (source, typed, symbols) = super::super::source_registration_inputs(
            &config.workspace_root, &case, frontend,
        ).unwrap();
        let check = check_source_algorithm_types(&source, &typed, &symbols);
        if label.is_some() {
            assert_eq!(check.unwrap_err(), "algorithms.unsupported_source_types");
        } else {
            check.unwrap();
        }
    }
}
