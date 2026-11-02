use super::*;

#[test]
fn attribute_parameter_arity_uses_genuine_structured_prefix_slots() {
    use mizar_resolve::{declarations::DeclarationShellCollector, symbols::SignatureProjectionExtractor};
    use mizar_syntax::{SurfaceNodeKind as K, SurfaceTokenKind as T};
    let text = concat!(
        "definition\n let n,m,x be set;\n",
        " attr PlainDef: x is plainmarked means thesis;\n",
        " attr IdentDef: x is n-identmarked means thesis;\n",
        " attr NumeralDef: x is 2-numeralmarked means thesis;\n",
        " attr SingleDef: x is (n)-singlemarked means thesis;\n",
        " attr PairDef: x is (n,2)-pairmarked means thesis;\n",
        " attr TripleDef: x is (n,2,m)-triplemarked means thesis;\nend;\n",
    );
    let (frontend, source, collected) =
        source_registration_owner_inputs(text, "attribute-arity", "main", 0);
    assert!(frontend.diagnostics.is_empty(), "{:?}", frontend.diagnostics);
    assert!(collected.diagnostics().is_empty());
    let ast = frontend.ast.as_ref().unwrap();
    assert!(ast.node_views().all(|node| !node.is_recovered()));
    let expected: [(&str, &[(T, &str)]); 6] = [
        ("plainmarked", &[]),
        ("identmarked", &[(T::Identifier, "n"), (T::ReservedSymbol, "-")]),
        ("numeralmarked", &[(T::Numeral, "2"), (T::ReservedSymbol, "-")]),
        ("singlemarked", &[(T::ReservedSymbol, "("), (T::Identifier, "n"), (T::ReservedSymbol, ")"), (T::ReservedSymbol, "-")]),
        ("pairmarked", &[(T::ReservedSymbol, "("), (T::Identifier, "n"), (T::ReservedSymbol, ","), (T::Numeral, "2"), (T::ReservedSymbol, ")"), (T::ReservedSymbol, "-")]),
        ("triplemarked", &[(T::ReservedSymbol, "("), (T::Identifier, "n"), (T::ReservedSymbol, ","), (T::Numeral, "2"), (T::ReservedSymbol, ","), (T::Identifier, "m"), (T::ReservedSymbol, ")"), (T::ReservedSymbol, "-")]),
    ];
    let patterns = ast.node_views().filter(|node| node.kind() == &K::AttributePattern).collect::<Vec<_>>();
    assert_eq!(patterns.len(), expected.len());
    assert_eq!(patterns.iter().map(|pattern| pattern.child_views().last().unwrap().as_token().unwrap().kind).collect::<Vec<_>>(),
        [T::Identifier, T::UserSymbol, T::UserSymbol, T::UserSymbol, T::UserSymbol, T::UserSymbol]);
    for (pattern, (name, prefix_tokens)) in patterns.iter().zip(expected) {
        let children = pattern.child_views().collect::<Vec<_>>();
        assert_eq!(children.len(), if prefix_tokens.is_empty() { 1 } else { 2 });
        assert_eq!(children.last().unwrap().as_token().unwrap().text.as_ref(), name);
        if !prefix_tokens.is_empty() {
            assert_eq!(children[0].kind(), &K::ParameterPrefix);
            assert_eq!(children[0].child_views().map(|node| {
                let token = node.as_token().unwrap();
                (token.kind, token.text.as_ref())
            }).collect::<Vec<_>>(), prefix_tokens);
        }
    }
    let shells = DeclarationShellCollector::new(ast, source.module()).collect();
    let projections = SignatureProjectionExtractor::new(ast, &shells, NamespacePath::new("main")).extract();
    assert_eq!(projections.iter().map(|projection| projection.arity()).collect::<Vec<_>>(),
        [Some(0), Some(1), Some(1), Some(1), Some(2), Some(3)]);
    use mizar_resolve::{env::SignatureShell, symbols::{SymbolCollector, SymbolDeclarationProjection, validate_source_symbol_env}};
    let arities = [0, 1, 1, 1, 2, 3];
    let notations = ["plainmarked", "n - identmarked", "2 - numeralmarked", "( n ) - singlemarked", "( n , 2 ) - pairmarked", "( n , 2 , m ) - triplemarked"];
    let mut old_unknown = Vec::new();
    for ((projection, arity), notation) in projections.iter().zip(arities).zip(notations) {
        assert_eq!(projection.notation_spelling(), Some(notation));
        let symbol = collected.env().symbols().iter().find(|entry| entry.primary_spelling() == projection.primary_spelling()).unwrap();
        let definition = collected.env().definitions().by_symbol(symbol.symbol()).unwrap();
        assert_eq!((definition.arity(), definition.signature(), symbol.signature()),
            (Some(arity), projection.signature(), projection.signature()));
        let rows = collected.env().lexical_summaries().visible_candidates(&NamespacePath::new("main"), projection.primary_spelling());
        if arity == 0 { assert!(rows.is_empty()); }
        else {
            assert_eq!(rows.len(), 1);
            assert_eq!((rows[0].symbol(), rows[0].arity()), (symbol.symbol(), Some(arity)));
            assert!(collected.env().lexical_summaries().visible_candidates(&NamespacePath::new("main"), notation).is_empty());
        }
        let Some(SignatureShell::Opaque { schema, payload }) = projection.signature() else { panic!("parser signature"); };
        assert_eq!(schema, "parser-signature-v1");
        let operand = format!(";arity={arity};");
        assert!(payload.contains(&operand));
        // Identity-only old projection; lexical flags do not enter symbol identity.
        old_unknown.push(SymbolDeclarationProjection::new(projection.shell(), projection.namespace().clone(), projection.primary_spelling(), projection.symbol_kind())
            .with_definition_kind(projection.definition_kind().unwrap()).with_overload_policy(projection.overload_policy())
            .with_notation_spelling(notation).with_signature(SignatureShell::Opaque { schema: schema.clone(), payload: payload.replace(&operand, ";arity=_;") }));
    }
    assert!(collected.env().overloads().is_empty());
    let old = SymbolCollector::new(ast.source_id, source.module(), &shells, &old_unknown).collect();
    for symbol in collected.env().symbols().iter() {
        let prior = old.env().symbols().iter().find(|entry| entry.primary_spelling() == symbol.primary_spelling()).unwrap();
        assert_ne!(prior.symbol().fqn(), symbol.symbol().fqn());
        assert_eq!(old.env().definitions().by_symbol(prior.symbol()).unwrap().arity(), None);
    }
    validate_source_symbol_env(&source, collected.env()).unwrap();
    let overload_text = concat!("definition let n,x,y be set; ",
        "attr First: x is (n,2)-sharedrank means thesis; attr Second: y is (n,2)-sharedrank means thesis; end;");
    let (parsed, arena, overloaded) = source_registration_owner_inputs(overload_text, "attribute-arity", "overloaded", 0);
    assert!(parsed.diagnostics.is_empty() && overloaded.diagnostics().is_empty());
    let groups = overloaded.env().overloads().iter().collect::<Vec<_>>();
    assert_eq!(groups.len(), 1);
    assert_eq!((groups[0].key().spelling(), groups[0].key().arity()), ("( n , 2 ) - sharedrank", Some(2)));
    assert_eq!(groups[0].candidates().len(), 2);
    for symbol in groups[0].candidates() {
        assert_eq!(overloaded.env().definitions().by_symbol(symbol).unwrap().arity(), Some(2));
    }
    validate_source_symbol_env(&arena, overloaded.env()).unwrap();

    for (edit, module, skip) in [(text.to_owned(), "main", 0), (format!(":: trivia\n{}", text.replace(" attr ", "    attr ")), "main", 0),
        (text.replace('x', "subject"), "main", 0), (text.replace('2', "987"), "main", 0), (text.to_owned(), "elsewhere", 0), (text.to_owned(), "main", 7)] {
        let (replay, arena, actual) = source_registration_owner_inputs(&edit, "attribute-arity", module, skip);
        assert!(replay.diagnostics.is_empty());
        assert!(actual.diagnostics().is_empty());
        validate_source_symbol_env(&arena, actual.env()).unwrap();
        for (projection, arity) in projections.iter().zip(arities) {
            let symbol = actual.env().symbols().iter().find(|entry| entry.primary_spelling() == projection.primary_spelling()).unwrap();
            assert_eq!(actual.env().definitions().by_symbol(symbol.symbol()).unwrap().arity(), Some(arity));
            let original = collected.env().symbols().iter().find(|entry| entry.primary_spelling() == projection.primary_spelling()).unwrap();
            if module != "main" { assert_ne!(symbol.symbol(), original.symbol()); }
            else if edit == text || edit.starts_with(":: trivia") { assert_eq!(symbol.symbol(), original.symbol()); }
        }
        if edit == text && skip == 0 && module == "main" { assert_eq!(actual, collected); }
        if skip != 0 { assert_ne!(arena.source_id(), source.source_id()); }
    }
}

#[test]
fn attribute_parameter_arity_keeps_genuine_frontend_and_wire_lexical_arity_unary() {
    use mizar_artifact::module_summary::ModuleSummaryIdentity;
    use mizar_lexer::{ExportedSymbolShape, UserSymbolArity, UserSymbolKind};
    use mizar_resolve::{declarations::DeclarationShellCollector, symbols::SignatureProjectionExtractor};
    let frontend = task257c4c1_frontend_output(concat!("definition let n,m,x be set; ",
        "attr Plain: x is arityplain means thesis; attr Pair: x is (n,m)-aritypair means thesis; ",
        "attr Triple: x is (n,2,m)-aritytriple means thesis; end;"), 164);
    assert!(frontend.diagnostics.is_empty());
    let ast = frontend.ast.as_ref().unwrap();
    let module = ResolverModuleId::new(frontend.source.package_id.clone(), frontend.source.module_path.clone());
    let shells = DeclarationShellCollector::new(ast, &module).collect();
    let collected = SignatureProjectionExtractor::new(ast, &shells, NamespacePath::new(module.path().as_str())).collect(&module);
    use mizar_syntax::{SurfaceNodeKind as K, SurfaceTokenKind as T};
    assert_eq!(ast.node_views().filter(|node| node.kind() == &K::AttributePattern)
        .map(|node| node.child_views().last().unwrap().as_token().unwrap().kind).collect::<Vec<_>>(),
        [T::Identifier, T::UserSymbol, T::UserSymbol]);
    let pairs = collected.pair_frontend_lexical_declarations(&frontend).unwrap();
    assert_eq!(pairs.len(), 3);
    let identity = ModuleSummaryIdentity { package_id: module.package().as_str().into(), package_version: None,
        lockfile_identity: None, module_path: module.path().as_str().into(), language_edition: "2026".into() };
    let wire = collected.export_frontend_lexical_contributions(&frontend, &identity).unwrap();
    assert_eq!(wire.len(), 3);
    for (name, arity) in [("arityplain", 0), ("aritypair", 2), ("aritytriple", 3)] {
        let (symbol, local) = pairs.iter().find(|(symbol, _)| symbol.primary_spelling() == name).unwrap();
        assert_eq!((local.kind, local.arity), (UserSymbolKind::Attribute, UserSymbolArity::exact(1)));
        assert_eq!(collected.env().definitions().by_symbol(symbol.symbol()).unwrap().arity(), Some(arity));
        let rows = collected.env().lexical_summaries().visible_candidates(&NamespacePath::new(module.path().as_str()), name);
        if arity == 0 { assert!(rows.is_empty()); }
        else {
            assert_eq!(rows.len(), 1);
            assert_eq!((rows[0].symbol(), rows[0].arity()), (symbol.symbol(), Some(arity)));
        }
        let payload = wire.iter().find(|entry| entry.key == symbol.symbol().local().as_str()).unwrap();
        let shape = ExportedSymbolShape::from_canonical_bytes(payload.payload.as_bytes()).unwrap();
        assert_eq!((shape.spelling.as_str(), shape.kind, shape.arity),
            (name, UserSymbolKind::Attribute, UserSymbolArity::exact(1)));
    }
}

#[test]
fn attribute_arguments_own_complete_unary_terms_before_root_and_qua_type_heads() {
    use mizar_syntax::{SurfaceNodeKind as K, SyntaxKind as S};
    let plan = step5c11_plan();
    let case = plan.cases.iter().find(|case| case.id.0 == "pass_parser_attribute_argument_ownership_001").unwrap();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let output = super::super::formula_statement::step5c8_test_frontend(&text);
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    let ast = output.ast.unwrap();
    assert!(ast.node_views().all(|v| !v.is_recovered()));
    assert!(ast.trivia().skipped_token_ranges().is_empty());
    assert_eq!(ast.node_views().filter(|v| v.kind() == &K::AttributeRef && text[v.range().start..].starts_with("tagged(")).count(), 17);
    for (name, root_body, qua_body, kind) in [
        ("prefix", "~ Atom", "~ Atom", S::PrefixExpression),
        ("infix", "Atom ++ Other", "Atom ++ Other", S::InfixExpression),
        ("postfix", "Atom !", "Atom !", S::PostfixExpression),
        ("selector", "Atom.value", "p.value", S::SelectorAccess),
        ("binder", "x |-> Atom", "x |-> Atom", S::BinderExpression),
        ("multi", "(x,y) |-> Other", "(x,y) |-> Other", S::BinderExpression),
        ("suffix", "~ Atom", "~ Atom", S::PrefixExpression),
        ("nested", "image(x)", "image(x)", S::ApplicationTerm),
        ("stale", "~ Atom ++ Other", "", S::InfixExpression),
    ] {
        for qua in [false, true] {
            if qua && qua_body.is_empty() { continue; }
            let marker = if qua { format!("set qua_{name} = ") } else { format!("reserve {name}_value for ") };
            let host_start = text.find(&marker).unwrap();
            let start = host_start + text[host_start..].find("tagged(").unwrap();
            let semicolon = host_start + text[host_start..].find(';').unwrap();
            let body = if qua { qua_body } else { root_body };
            let close = start + "tagged(".len() + body.len();
            let expression = ast.node_views().find(|v| v.kind() == &K::TypeExpression && v.range().start == start).unwrap();
            assert_eq!(expression.range(), range(ast.source_id, start, semicolon));
            assert_eq!(expression.children().len(), 2);
            let chain = ast.node_view(expression.children()[0]).unwrap();
            let head = ast.node_view(expression.children()[1]).unwrap();
            assert_eq!(chain.kind(), &K::AttributeChain);
            assert_eq!(chain.range(), range(ast.source_id, start, if name == "suffix" { semicolon - 2 } else { close + 1 }));
            assert_eq!(head.kind(), &K::TypeHead);
            assert_eq!(head.range(), range(ast.source_id, semicolon - 1, semicolon));
            assert_eq!(&text[head.range().start..head.range().end], "R");
            let attribute = ast.node_view(chain.children()[0]).unwrap();
            assert_eq!(attribute.kind(), &K::AttributeRef);
            assert_eq!(attribute.range(), range(ast.source_id, start, close + 1));
            let arguments = attribute.children().iter().map(|id| ast.node_view(*id).unwrap()).filter(|v| v.kind() == &K::TermExpression).collect::<Vec<_>>();
            assert_eq!(arguments.len(), 1, "{marker}");
            assert_eq!(arguments[0].range(), range(ast.source_id, start + 7, close));
            let term = ast.node_view(arguments[0].children()[0]).unwrap();
            if name != "stale" { assert_eq!(term.syntax_kind(), kind, "{marker}"); }
            assert_eq!(ast.node_view(*attribute.children().last().unwrap()).unwrap().as_token().unwrap().text.as_ref(), ")");
            assert_eq!(chain.children().len(), if name == "suffix" { 2 } else { 1 });
            if name == "suffix" {
                let suffix = ast.node_view(chain.children()[1]).unwrap();
                assert_eq!(&text[suffix.range().start..suffix.range().end], "non empty");
            }
            if name == "multi" {
                assert_eq!(term.children().len(), 7);
                assert_eq!(term.children()[..5].iter().map(|id| ast.node_view(*id).unwrap().as_token().unwrap().text.as_ref()).collect::<String>(), "(x,y)");
            }
            if qua {
                let qualification = ast.node_views().find(|v| v.kind() == &K::QuaExpression && v.children().contains(&expression.id())).unwrap();
                assert_eq!(qualification.range(), range(ast.source_id, host_start + marker.len(), semicolon));
                assert_eq!(*qualification.children().last().unwrap(), expression.id());
            }
            let mut pending = vec![expression.id()];
            let mut seen = std::collections::HashSet::new();
            let mut tokens = Vec::new();
            let mut operators = (0, 0);
            while let Some(id) = pending.pop() {
                assert!(seen.insert(id), "duplicate attribute/type child in {marker}");
                let node = ast.node_view(id).unwrap();
                if name == "stale" {
                    operators.0 += usize::from(node.syntax_kind() == S::PrefixExpression);
                    operators.1 += usize::from(node.syntax_kind() == kind);
                }
                if node.as_token().is_some() { tokens.push(id); }
                pending.extend(node.children().iter().rev().copied());
            }
            if name == "stale" { assert_eq!(operators, (1, 1)); }
            assert_eq!(tokens, ast.token_views().filter(|v| v.range().start >= start && v.range().end <= semicolon).map(|v| v.id()).collect::<Vec<_>>());
        }
    }
    let after = text.find("set after_attribute_ownership").unwrap();
    assert!(ast.node_views().filter_map(|v| v.as_set_statement()).any(|v| v.range() == range(ast.source_id, after, after + "set after_attribute_ownership = sentinel;".len()) && !v.is_recovered()));
    let frontend = task257c4c1_frontend_output(&text, 127);
    let current = frontend.cache_keys.ast.as_ref().unwrap();
    assert_eq!(current.parser_version.version.as_ref(), "mizar-parser/surface-ast-v30");
    let mut retained = current.clone();
    retained.parser_version = mizar_frontend::parsing::ParserCacheKeyVersion::new("mizar-parser/surface-ast-v18");
    assert_ne!(current.stable_hash(), retained.stable_hash());
    let bytes = ast.canonical_bytes().expect("valid ownership AST storage");
    let restored = mizar_syntax::SurfaceAst::from_canonical_bytes(&bytes, ast.source_id).unwrap();
    assert_eq!(restored.snapshot_text_with_trivia(), ast.snapshot_text_with_trivia());
}

#[test]
fn attribute_argument_recovery_retains_consumed_terms_heads_and_host_boundaries() {
    use mizar_frontend::orchestration::{DiagnosticCode, DiagnosticLocation};
    use mizar_syntax::{SurfaceNodeKind as K, SyntaxKind as S};
    let plan = step5c11_plan();
    let case = plan.cases.iter().find(|case| case.id.0 == "fail_parser_attribute_argument_ownership_001").unwrap();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let output = super::super::formula_statement::step5c8_test_frontend(&text);
    assert_eq!(output.diagnostics.len(), 12, "{:?}", output.diagnostics);
    let ast = output.ast.unwrap();
    let mut diagnostic = output.diagnostics.iter();
    for (name, body, has_head, balanced) in [
        ("prefix_head", "~ Atom", true, false),
        ("binder_head", "x |-> Atom", true, false),
        ("prefix_missing", "~ Atom", false, false),
        ("binder_missing", "x |-> Atom", false, false),
        ("inner_operator_missing", "x |-> Atom ++ Other", false, false),
        ("balanced_missing", "~ Atom", false, true),
        ("legacy_missing", "x", true, false),
        ("legacy_balanced", "x", true, false),
    ] {
        let host_start = text.find(&format!("reserve {name} for ")).unwrap();
        let start = host_start + text[host_start..].find("tagged(").unwrap();
        let semicolon = host_start + text[host_start..].find(';').unwrap();
        let body_end = start + 7 + body.len();
        let boundary = if has_head { body_end + 1 } else { semicolon };
        let expression = ast.node_views().find(|v| v.kind() == &K::TypeExpression && v.range().start == start).unwrap();
        assert_eq!(expression.children().len(), 2);
        let chain = ast.node_view(expression.children()[0]).unwrap();
        let head = ast.node_view(expression.children()[1]).unwrap();
        assert_eq!(chain.kind(), &K::AttributeChain);
        assert_eq!(chain.children().len(), 1);
        let attribute = ast.node_view(chain.children()[0]).unwrap();
        let attribute_end = body_end + usize::from(balanced);
        assert_eq!(attribute.kind(), &K::AttributeRef);
        assert_eq!(attribute.range(), range(ast.source_id, start, attribute_end));
        assert_eq!(chain.range(), attribute.range());
        assert_eq!(expression.range(), range(ast.source_id, start, if has_head { boundary + 1 } else { attribute_end }));
        let argument = attribute.children().iter().map(|id| ast.node_view(*id).unwrap()).filter(|v| v.kind() == &K::TermExpression).collect::<Vec<_>>();
        assert_eq!(argument.len(), 1);
        assert_eq!(argument[0].range(), range(ast.source_id, start + 7, body_end));
        assert!(!argument[0].is_recovered());
        let term = ast.node_view(argument[0].children()[0]).unwrap();
        assert_eq!(term.syntax_kind(), if body.starts_with('~') { S::PrefixExpression } else if body.contains("|->") { S::BinderExpression } else { S::TermReference });
        if name == "inner_operator_missing" {
            let binder_body = ast.node_view(*term.children().last().unwrap()).unwrap();
            assert_eq!(&text[binder_body.range().start..binder_body.range().end], "Atom ++ Other");
            assert_eq!(ast.node_view(binder_body.children()[0]).unwrap().syntax_kind(), S::InfixExpression);
        }
        let unmatched = attribute.children().iter().map(|id| ast.node_view(*id).unwrap()).filter(|v| v.as_recovery() == Some(SyntaxRecoveryKind::UnmatchedOpeningDelimiter)).collect::<Vec<_>>();
        assert_eq!(unmatched.len(), usize::from(!balanced));
        if balanced {
            assert_eq!(ast.node_view(*attribute.children().last().unwrap()).unwrap().as_token().unwrap().text.as_ref(), ")");
        } else {
            assert!(unmatched[0].is_recovered());
            assert_eq!(unmatched[0].range(), range(ast.source_id, boundary, boundary));
            let actual = diagnostic.next().unwrap();
            assert_eq!(actual.code, DiagnosticCode::Syntax("malformed_term_expression".into()));
            assert_eq!(actual.location, DiagnosticLocation::SourceRange(range(ast.source_id, boundary, boundary + 1)));
            assert!(actual.secondary.iter().any(|a| matches!(a, mizar_session::SourceAnchor::Range(r) if *r == range(ast.source_id, start + 6, start + 7))));
        }
        if has_head {
            assert_eq!(head.kind(), &K::TypeHead);
            assert_eq!(head.range(), range(ast.source_id, boundary, boundary + 1));
            assert_eq!(&text[head.range().start..head.range().end], "R");
        } else {
            assert!(head.is_recovered());
            assert_eq!(head.as_recovery(), Some(SyntaxRecoveryKind::MissingTypeExpression));
            assert_eq!(head.range(), range(ast.source_id, semicolon, semicolon));
        }
        if !has_head || name == "legacy_balanced" {
            let actual = diagnostic.next().unwrap();
            let anchor = if has_head { boundary + 1 } else { semicolon };
            assert_eq!(actual.code, DiagnosticCode::Syntax("malformed_type_expression".into()));
            assert_eq!(actual.location, DiagnosticLocation::SourceRange(range(ast.source_id, anchor, anchor + 1)));
        }
        let host = ast.node_views().find(|v| v.kind() == &K::ReserveItem && v.range().start == host_start).unwrap();
        assert_eq!(host.range(), range(ast.source_id, host_start, semicolon + 1));
        let delimiter = ast.node_view(*host.children().last().unwrap()).unwrap();
        assert_eq!(delimiter.range(), range(ast.source_id, semicolon, semicolon + 1));
        assert_eq!(delimiter.as_token().unwrap().text.as_ref(), ";");
        let after = text.find(&format!("set after_{name} = sentinel;")).unwrap();
        assert!(ast.node_views().filter_map(|v| v.as_set_statement()).any(|v| v.range() == range(ast.source_id, after, after + format!("set after_{name} = sentinel;").len()) && !v.is_recovered()));
    }
    assert!(diagnostic.next().is_none());
    for (kind, count) in [(SyntaxRecoveryKind::MissingTypeExpression, 4), (SyntaxRecoveryKind::UnmatchedOpeningDelimiter, 7), (SyntaxRecoveryKind::MissingTerm, 0), (SyntaxRecoveryKind::SkippedToken, 1)] {
        assert_eq!(ast.node_views().filter(|v| v.as_recovery() == Some(kind)).count(), count);
    }
    let skipped = ast.node_views().find(|v| v.as_recovery() == Some(SyntaxRecoveryKind::SkippedToken)).unwrap();
    let tail = text.find("tagged(x R) R;").unwrap() + "tagged(x R".len();
    assert_eq!(skipped.range(), range(ast.source_id, tail, tail + 3));
    let host_start = text.find("reserve legacy_balanced for ").unwrap();
    let host = ast.node_views().find(|v| v.kind() == &K::ReserveItem && v.range().start == host_start).unwrap();
    assert!(host.children().contains(&skipped.id()));
    assert_eq!(skipped.children().iter().map(|id| ast.node_view(*id).unwrap().as_token().unwrap().text.as_ref()).collect::<String>(), ")R");
    assert_eq!(ast.trivia().skipped_token_ranges().len(), 1);
    assert_eq!(ast.trivia().skipped_token_ranges()[0].range, skipped.range());
    let bytes = ast.canonical_bytes().expect("valid recovered ownership AST storage");
    let restored = mizar_syntax::SurfaceAst::from_canonical_bytes(&bytes, ast.source_id).unwrap();
    assert_eq!(restored.snapshot_text_with_trivia(), ast.snapshot_text_with_trivia());
}

#[test]
fn attribute_argument_continuation_preserves_order_payloads_and_actual_type_heads() {
    use mizar_syntax::{SurfaceNodeKind as K, SyntaxKind as S};
    let plan = step5c11_plan();
    let case = plan.cases.iter().find(|case| case.id.0 == "pass_parser_attribute_argument_continuation_001").unwrap();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let output = super::super::formula_statement::step5c8_test_frontend(&text);
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    let ast = output.ast.unwrap();
    assert!(ast.node_views().all(|v| !v.is_recovered()));
    assert!(ast.trivia().skipped_token_ranges().is_empty());
    let cases: &[(&str, &[(&str, S)])] = &[
        ("complex_first", &[("~ Atom", S::PrefixExpression), ("x", S::TermReference)]),
        ("infix_first", &[("Atom ++ Other", S::InfixExpression), ("x", S::TermReference)]),
        ("postfix_middle", &[("x", S::TermReference), ("Atom !", S::PostfixExpression), ("y", S::TermReference)]),
        ("selector_second", &[("~ Atom", S::PrefixExpression), ("p.value", S::SelectorAccess)]),
        ("multi_binder_third", &[("~ Atom", S::PrefixExpression), ("y", S::TermReference), ("(x,y) |-> Atom", S::BinderExpression)]),
        ("complex_middle", &[("x", S::TermReference), ("~ Atom", S::PrefixExpression), ("y", S::TermReference)]),
        ("complex_last", &[("x", S::TermReference), ("~ Atom", S::PrefixExpression)]),
        ("binder_first", &[("x |-> Atom", S::BinderExpression), ("y", S::TermReference)]),
        ("nested_collection", &[("~ Atom", S::PrefixExpression), ("{x,y}", S::SetEnumeration), ("[x,y]", S::ApplicationTerm)]),
        ("constructor_second", &[("~ Atom", S::PrefixExpression), ("Box[x](value: y)", S::StructureConstructor)]),
        ("ordinary_qua", &[("~ Atom", S::PrefixExpression), ("image(x)", S::ApplicationTerm)]),
    ];
    assert_eq!(ast.node_views().filter(|v| v.kind() == &K::AttributeRef && (text[v.range().start..].starts_with("linked(") || text[v.range().start..].starts_with("named("))).count(), cases.len());
    for (name, expected) in cases {
        let qua = *name == "ordinary_qua";
        let marker = if qua { "set ordinary_qua = ".to_owned() } else { format!("reserve {name} for ") };
        let host_start = text.find(&marker).unwrap();
        let semicolon = host_start + text[host_start..].find(';').unwrap();
        let spelling = format!("{}({})", if expected.len() == 3 { "named" } else { "linked" }, expected.iter().map(|(body, _)| *body).collect::<Vec<_>>().join(","));
        let start = host_start + text[host_start..].find(&spelling).unwrap();
        let close = start + spelling.len() - 1;
        let expression = ast.node_views().find(|v| v.kind() == &K::TypeExpression && v.range().start == start).unwrap();
        assert_eq!(expression.range(), range(ast.source_id, start, semicolon));
        assert_eq!(expression.children().len(), 2);
        let chain = ast.node_view(expression.children()[0]).unwrap();
        let head = ast.node_view(expression.children()[1]).unwrap();
        assert_eq!(chain.kind(), &K::AttributeChain);
        assert_eq!(head.kind(), &K::TypeHead);
        assert_eq!(head.range(), range(ast.source_id, semicolon - 1, semicolon));
        assert_eq!(&text[head.range().start..head.range().end], "R");
        let attribute = ast.node_view(chain.children()[0]).unwrap();
        assert_eq!(attribute.kind(), &K::AttributeRef);
        assert_eq!(attribute.range(), range(ast.source_id, start, close + 1));
        let closer = ast.node_view(*attribute.children().last().unwrap()).unwrap();
        assert_eq!(closer.range(), range(ast.source_id, close, close + 1));
        assert_eq!(closer.as_token().unwrap().text.as_ref(), ")");
        let arguments = attribute.children().iter().map(|id| ast.node_view(*id).unwrap()).filter(|v| v.kind() == &K::TermExpression).collect::<Vec<_>>();
        assert_eq!(arguments.iter().map(|v| &text[v.range().start..v.range().end]).collect::<Vec<_>>(), expected.iter().map(|(body, _)| *body).collect::<Vec<_>>(), "{name}");
        for (argument, (body, kind)) in arguments.iter().zip(*expected) {
            let term = ast.node_view(argument.children()[0]).unwrap();
            assert_eq!(term.syntax_kind(), *kind, "{name}: {body}");
            if let Some(operator) = term.as_prefix_expression() { assert_eq!((operator.spelling.as_ref(), operator.precedence), ("~", 85)); }
            if let Some(operator) = term.as_infix_expression() { assert_eq!((operator.spelling.as_ref(), operator.precedence), ("++", 80)); }
            if let Some(operator) = term.as_postfix_expression() { assert_eq!((operator.spelling.as_ref(), operator.precedence), ("!", 95)); }
            if *kind == S::BinderExpression {
                assert_eq!(term.children()[..term.children().len() - 2].iter().map(|id| ast.node_view(*id).unwrap().as_token().unwrap().text.as_ref()).collect::<String>(), if body.starts_with('(') { "(x,y)" } else { "x" });
                let binder_body = ast.node_view(*term.children().last().unwrap()).unwrap();
                assert_eq!(&text[binder_body.range().start..binder_body.range().end], "Atom");
            }
            if *kind == S::SetEnumeration || *body == "[x,y]" {
                assert_eq!(term.children().iter().map(|id| ast.node_view(*id).unwrap()).filter(|v| v.kind() == &K::TermExpression).map(|v| &text[v.range().start..v.range().end]).collect::<Vec<_>>(), ["x", "y"]);
            }
            if *kind == S::SelectorAccess { assert_eq!(ast.node_view(*term.children().last().unwrap()).unwrap().as_token().unwrap().text.as_ref(), "value"); }
            if *body == "image(x)" { assert!(term.children().iter().map(|id| ast.node_view(*id).unwrap()).any(|v| v.kind() == &K::TermExpression && &text[v.range().start..v.range().end] == "x")); }
            if *kind == S::StructureConstructor {
                for (kind, source) in [(K::TypeArguments, "[x]"), (K::FieldArgument, "value: y")] {
                    assert!(term.children().iter().map(|id| ast.node_view(*id).unwrap()).any(|v| v.kind() == &kind && &text[v.range().start..v.range().end] == source));
                }
            }
        }
        let suffix = *name == "nested_collection" || qua;
        assert_eq!(chain.children().len(), if suffix { 2 } else { 1 });
        assert_eq!(chain.range(), range(ast.source_id, start, if suffix { semicolon - 2 } else { close + 1 }));
        if suffix { let node = ast.node_view(chain.children()[1]).unwrap(); assert_eq!(&text[node.range().start..node.range().end], "non empty"); }
        if qua { assert!(ast.node_views().any(|v| v.kind() == &K::QuaExpression && v.children().last() == Some(&expression.id()) && v.range() == range(ast.source_id, host_start + marker.len(), semicolon))); }
        let host = ast.node_views().find(|v| v.kind() == (if qua { &K::SetStatement } else { &K::ReserveItem }) && v.range().start == host_start).unwrap();
        assert_eq!(host.range(), range(ast.source_id, host_start, semicolon + 1));
        let delimiter = ast.node_view(*host.children().last().unwrap()).unwrap();
        assert_eq!(delimiter.range(), range(ast.source_id, semicolon, semicolon + 1));
        assert_eq!(delimiter.as_token().unwrap().text.as_ref(), ";");
        let mut pending = vec![expression.id()];
        let mut seen = std::collections::HashSet::new();
        let mut tokens = Vec::new();
        while let Some(id) = pending.pop() {
            assert!(seen.insert(id), "duplicate attribute/type child in {name}");
            let node = ast.node_view(id).unwrap();
            if node.as_token().is_some() { tokens.push(id); }
            pending.extend(node.children().iter().rev().copied());
        }
        assert_eq!(tokens, ast.token_views().filter(|v| v.range().start >= start && v.range().end <= semicolon).map(|v| v.id()).collect::<Vec<_>>());
    }
    let after = text.find("set after_attribute_continuation").unwrap();
    assert!(ast.node_views().filter_map(|v| v.as_set_statement()).any(|v| v.range() == range(ast.source_id, after, after + "set after_attribute_continuation = sentinel;".len()) && !v.is_recovered()));
    let frontend = task257c4c1_frontend_output(&text, 128);
    let current = frontend.cache_keys.ast.as_ref().unwrap();
    assert_eq!(current.parser_version.version.as_ref(), "mizar-parser/surface-ast-v30");
    let mut retained = current.clone();
    retained.parser_version = mizar_frontend::parsing::ParserCacheKeyVersion::new("mizar-parser/surface-ast-v19");
    assert_ne!(current.stable_hash(), retained.stable_hash());
    let bytes = ast.canonical_bytes().expect("valid continued attribute ownership AST storage");
    let restored = mizar_syntax::SurfaceAst::from_canonical_bytes(&bytes, ast.source_id).unwrap();
    assert_eq!(restored.snapshot_text_with_trivia(), ast.snapshot_text_with_trivia());
}

#[test]
fn attribute_argument_continuation_recovery_preserves_holes_heads_and_successors() {
    use mizar_frontend::orchestration::{DiagnosticCode, DiagnosticLocation};
    use mizar_syntax::SurfaceNodeKind as K;
    let plan = step5c11_plan();
    let case = plan.cases.iter().find(|case| case.id.0 == "fail_parser_attribute_argument_continuation_001").unwrap();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let output = super::super::formula_statement::step5c8_test_frontend(&text);
    assert_eq!(output.diagnostics.len(), 10, "{:?}", output.diagnostics);
    let ast = output.ast.unwrap();
    let mut diagnostics = output.diagnostics.iter();
    let cases: &[(&str, &str, &[&str], Option<&str>)] = &[
        ("trailing_comma", "linked(~ Atom,)", &["~ Atom"], Some(")")),
        ("leading_comma", "linked(,~ Atom)", &["~ Atom"], Some(",")),
        ("repeated_comma", "named(~ Atom,,x)", &["~ Atom", "x"], Some(",x")),
        ("empty_arguments", "linked()", &[], Some(")")),
        ("missing_close", "linked(~ Atom,x", &["~ Atom", "x"], None),
        ("complex_missing_close", "linked(~ Atom,~ Other", &["~ Atom", "~ Other"], None),
        ("balanced_missing_head", "linked(~ Atom,x)", &["~ Atom", "x"], None),
        ("missing_comma", "linked(~ Atom", &["~ Atom"], None),
    ];
    for (name, spelling, expected, hole) in cases {
        let host_start = text.find(&format!("reserve {name} for ")).unwrap();
        let start = host_start + text[host_start..].find(*spelling).unwrap();
        let end = start + spelling.len();
        let semicolon = host_start + text[host_start..].find(';').unwrap();
        let missing_comma = *name == "missing_comma";
        let unclosed = name.ends_with("missing_close") || missing_comma;
        let has_head = *name != "balanced_missing_head" && !missing_comma;
        let boundary = if missing_comma { end + 1 } else if has_head { semicolon - 1 } else { semicolon };
        let expression = ast.node_views().find(|v| v.kind() == &K::TypeExpression && v.range().start == start).unwrap();
        assert_eq!(expression.children().len(), 2);
        let chain = ast.node_view(expression.children()[0]).unwrap();
        let head = ast.node_view(expression.children()[1]).unwrap();
        assert_eq!(chain.kind(), &K::AttributeChain);
        assert_eq!(chain.children().len(), 1);
        let attribute = ast.node_view(chain.children()[0]).unwrap();
        assert_eq!(attribute.kind(), &K::AttributeRef);
        assert_eq!(attribute.range(), range(ast.source_id, start, end));
        assert_eq!(chain.range(), attribute.range());
        assert_eq!(expression.range(), range(ast.source_id, start, if has_head { semicolon } else { end }));
        let children = attribute.children().iter().map(|id| ast.node_view(*id).unwrap()).collect::<Vec<_>>();
        let arguments = children.iter().filter(|v| v.kind() == &K::TermExpression).collect::<Vec<_>>();
        assert_eq!(arguments.iter().map(|v| &text[v.range().start..v.range().end]).collect::<Vec<_>>(), *expected, "{name}");
        assert!(arguments.iter().all(|v| !v.is_recovered()));
        let missing = children.iter().filter(|v| v.as_recovery() == Some(SyntaxRecoveryKind::MissingTerm)).collect::<Vec<_>>();
        assert_eq!(missing.len(), usize::from(hole.is_some()));
        let anchor = hole.map_or(boundary, |marker| start + spelling.find(marker).unwrap());
        if hole.is_some() { assert!(missing[0].is_recovered()); assert_eq!(missing[0].range(), range(ast.source_id, anchor, anchor)); }
        let unmatched = children.iter().filter(|v| v.as_recovery() == Some(SyntaxRecoveryKind::UnmatchedOpeningDelimiter)).collect::<Vec<_>>();
        assert_eq!(unmatched.len(), usize::from(unclosed));
        if unclosed { assert!(unmatched[0].is_recovered()); assert_eq!(unmatched[0].range(), range(ast.source_id, boundary, boundary)); }
        else { let close = children.last().unwrap(); assert_eq!(close.range(), range(ast.source_id, end - 1, end)); assert_eq!(close.as_token().unwrap().text.as_ref(), ")"); }
        let codes: &[&str] = if missing_comma { &["malformed_term_expression", "malformed_type_expression", "malformed_type_expression"] } else if has_head { &["malformed_term_expression"] } else { &["malformed_type_expression"] };
        for code in codes {
            let diagnostic = diagnostics.next().unwrap();
            assert_eq!(diagnostic.code, DiagnosticCode::Syntax((*code).into()), "{name}");
            assert_eq!(diagnostic.location, DiagnosticLocation::SourceRange(range(ast.source_id, anchor, anchor + 1)), "{name}");
        }
        if has_head { assert_eq!(head.kind(), &K::TypeHead); assert_eq!(head.range(), range(ast.source_id, boundary, boundary + 1)); assert_eq!(&text[head.range().start..head.range().end], "R"); }
        else { assert!(head.is_recovered()); assert_eq!(head.as_recovery(), Some(SyntaxRecoveryKind::MissingTypeExpression)); assert_eq!(head.range(), range(ast.source_id, boundary, boundary)); }
        let host = ast.node_views().find(|v| v.kind() == &K::ReserveItem && v.range().start == host_start).unwrap();
        assert_eq!(host.range(), range(ast.source_id, host_start, semicolon + 1));
        let delimiter = ast.node_view(*host.children().last().unwrap()).unwrap();
        assert_eq!(delimiter.range(), range(ast.source_id, semicolon, semicolon + 1));
        assert_eq!(delimiter.as_token().unwrap().text.as_ref(), ";");
        let after_spelling = format!("set after_{name} = sentinel;");
        let after = text.find(&after_spelling).unwrap();
        assert!(ast.node_views().filter_map(|v| v.as_set_statement()).any(|v| v.range() == range(ast.source_id, after, after + after_spelling.len()) && !v.is_recovered()));
        let mut pending = vec![host.id()];
        let mut seen = std::collections::HashSet::new();
        let mut tokens = Vec::new();
        while let Some(id) = pending.pop() {
            assert!(seen.insert(id), "duplicate recovered attribute child in {name}");
            let node = ast.node_view(id).unwrap();
            if node.as_token().is_some() { tokens.push(id); }
            pending.extend(node.children().iter().rev().copied());
        }
        assert_eq!(tokens, ast.token_views().filter(|v| v.range().start >= host_start && v.range().end <= semicolon + 1).map(|v| v.id()).collect::<Vec<_>>());
        if missing_comma {
            let skipped = host.children().iter().map(|id| ast.node_view(*id).unwrap()).find(|v| v.as_recovery() == Some(SyntaxRecoveryKind::SkippedToken)).unwrap();
            assert!(skipped.is_recovered());
            assert_eq!(skipped.range(), range(ast.source_id, boundary, semicolon));
            assert_eq!(skipped.children().iter().map(|id| ast.node_view(*id).unwrap().as_token().unwrap().text.as_ref()).collect::<String>(), "x)R");
            assert_eq!(ast.trivia().skipped_token_ranges().len(), 1);
            assert_eq!(ast.trivia().skipped_token_ranges()[0].range, skipped.range());
        }
    }
    assert!(diagnostics.next().is_none());
    for (kind, count) in [(SyntaxRecoveryKind::MissingTerm, 4), (SyntaxRecoveryKind::UnmatchedOpeningDelimiter, 3), (SyntaxRecoveryKind::MissingTypeExpression, 2), (SyntaxRecoveryKind::SkippedToken, 1)] {
        assert_eq!(ast.node_views().filter(|v| v.as_recovery() == Some(kind)).count(), count);
    }
    let bytes = ast.canonical_bytes().expect("valid recovered continuation AST storage");
    let restored = mizar_syntax::SurfaceAst::from_canonical_bytes(&bytes, ast.source_id).unwrap();
    assert_eq!(restored.snapshot_text_with_trivia(), ast.snapshot_text_with_trivia());
}

#[test]
fn attribute_dispatch_preserves_complete_payloads_heads_and_bare_assertions() {
    use mizar_syntax::{SurfaceNodeKind as K, SyntaxKind as S};
    let plan = build_test_plan(&step5c11_config()).unwrap();
    let case = plan.cases.iter().find(|case| case.id.0 == "pass_parser_attribute_dispatch_001").unwrap();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let output = super::super::formula_statement::step5c8_test_frontend(&text);
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    let ast = output.ast.unwrap();
    assert!(ast.node_views().all(|v| !v.is_recovered()));
    assert!(ast.trivia().skipped_token_ranges().is_empty());
    let cases = &[
        ("reserve prefix_value for Family", K::TypeArguments, &[("~ Atom", S::PrefixExpression)][..]),
        ("set generic_prefix = pick", K::TemplateArguments, &[("~ Atom", S::PrefixExpression)]),
        ("reserve infix_value for Family", K::TypeArguments, &[("Atom ++ Other", S::InfixExpression)]),
        ("set generic_infix = pick", K::TemplateArguments, &[("Atom ++ Other", S::InfixExpression)]),
        ("reserve postfix_value for Family", K::TypeArguments, &[("Atom !", S::PostfixExpression)]),
        ("set generic_postfix = pick", K::TemplateArguments, &[("Atom !", S::PostfixExpression)]),
        ("reserve selector_value for Family", K::TypeArguments, &[("p.value", S::SelectorAccess)]),
        ("set generic_selector = pick", K::TemplateArguments, &[("p.value", S::SelectorAccess)]),
        ("reserve binder_value for Family", K::TypeArguments, &[("x |-> Atom", S::BinderExpression)]),
        ("set generic_binder = pick", K::TemplateArguments, &[("x |-> Atom", S::BinderExpression)]),
        ("reserve multi_value for Family", K::TypeArguments, &[("(x,y) |-> Other", S::BinderExpression)]),
        ("set generic_multi = pick", K::TemplateArguments, &[("(x,y) |-> Other", S::BinderExpression)]),
        ("reserve nested_value for Family", K::TypeArguments, &[("image([x,{y}])", S::ApplicationTerm), ("x", S::TermReference)]),
        ("set generic_nested = pick", K::TemplateArguments, &[("image([x,{y}])", S::ApplicationTerm), ("x", S::TermReference)]),
        ("reserve choice_value for Family", K::TypeArguments, &[("the set", S::ChoiceTerm), ("x", S::TermReference)]),
        ("set generic_qua = pick", K::TemplateArguments, &[("x qua set", S::QuaExpression), ("y", S::TermReference)]),
        ("reserve collection_value for Family", K::TypeArguments, &[("{x where x is set: thesis}", S::SetComprehension), ("y", S::TermReference)]),
        ("set generic_constructor = pick", K::TemplateArguments, &[("Box[x](value: y)", S::StructureConstructor), ("x", S::TermReference)]),
    ];
    for (marker, list_kind, expected) in cases {
        let host_start = text.find(marker).unwrap();
        let start = host_start + marker.len();
        let semicolon = host_start + text[host_start..].find(';').unwrap();
        let close = start + text[start..semicolon].rfind(']').unwrap();
        let list = ast.node_views().find(|v| v.kind() == list_kind && v.range().start == start).unwrap();
        assert_eq!(list.range(), range(ast.source_id, start, close + 1));
        assert_eq!(list.children().len(), 3);
        for (id, spelling, offset) in [(list.children()[0], "[", start), (list.children()[2], "]", close)] {
            let token = ast.node_view(id).unwrap();
            assert_eq!(token.as_token().unwrap().text.as_ref(), spelling);
            assert_eq!(token.range(), range(ast.source_id, offset, offset + 1));
        }
        let mut expression = ast.node_view(list.children()[1]).unwrap();
        if *list_kind == K::TemplateArguments {
            assert_eq!(expression.kind(), &K::TemplateArgument);
            assert_eq!(expression.children().len(), 1);
            expression = ast.node_view(expression.children()[0]).unwrap();
            let reference = ast.node_views().find(|v| v.kind() == &K::TermReference && v.children().contains(&list.id())).unwrap();
            assert_eq!(reference.range().end, list.range().end);
            let application = ast.node_views().find(|v| v.kind() == &K::ApplicationTerm && v.children().first() == Some(&reference.id())).unwrap();
            assert_eq!(&text[application.range().end - 3..application.range().end], "(x)");
            assert_eq!(application.range().end, semicolon);
        }
        assert_eq!(expression.kind(), &K::TypeExpression);
        assert_eq!(expression.range(), range(ast.source_id, start + 1, close));
        assert_eq!(expression.children().len(), 2);
        let chain = ast.node_view(expression.children()[0]).unwrap();
        let head = ast.node_view(expression.children()[1]).unwrap();
        assert_eq!(chain.kind(), &K::AttributeChain);
        assert_eq!(head.kind(), &K::TypeHead);
        assert_eq!(head.range(), range(ast.source_id, close - 1, close));
        assert_eq!(&text[head.range().start..head.range().end], "R");
        let attribute = ast.node_view(chain.children()[0]).unwrap();
        assert_eq!(attribute.kind(), &K::AttributeRef);
        let arguments = attribute.children().iter().map(|id| ast.node_view(*id).unwrap()).filter(|v| v.kind() == &K::TermExpression).collect::<Vec<_>>();
        assert_eq!(arguments.iter().map(|v| &text[v.range().start..v.range().end]).collect::<Vec<_>>(), expected.iter().map(|(body, _)| *body).collect::<Vec<_>>(), "{marker}");
        let attribute_close = ast.node_view(*attribute.children().last().unwrap()).unwrap();
        assert_eq!(attribute_close.as_token().unwrap().text.as_ref(), ")");
        assert_eq!(attribute_close.range().end, attribute.range().end);
        for (argument, (body, kind)) in arguments.iter().zip(*expected) {
            let term = ast.node_view(argument.children()[0]).unwrap();
            assert_eq!(term.syntax_kind(), *kind, "{marker}: {body}");
            if let Some(operator) = term.as_prefix_expression() { assert_eq!((operator.spelling.as_ref(), operator.precedence), ("~", 85)); }
            if let Some(operator) = term.as_infix_expression() { assert_eq!((operator.spelling.as_ref(), operator.precedence), ("++", 80)); }
            if let Some(operator) = term.as_postfix_expression() { assert_eq!((operator.spelling.as_ref(), operator.precedence), ("!", 95)); }
            if *kind == S::BinderExpression {
                assert_eq!(term.children()[..term.children().len() - 2].iter().map(|id| ast.node_view(*id).unwrap().as_token().unwrap().text.as_ref()).collect::<String>(), if body.starts_with('(') { "(x,y)" } else { "x" });
                let binder_body = ast.node_view(*term.children().last().unwrap()).unwrap();
                assert_eq!(&text[binder_body.range().start..binder_body.range().end], if body.starts_with('(') { "Other" } else { "Atom" });
            }
            if *kind == S::SelectorAccess { assert_eq!(ast.node_view(*term.children().last().unwrap()).unwrap().as_token().unwrap().text.as_ref(), "value"); }
            if *body == "image([x,{y}])" {
                for (kind, source) in [(K::ApplicationTerm, "[x,{y}]"), (K::SetEnumeration, "{y}")] {
                    assert!(ast.node_views().any(|v| v.kind() == &kind && v.range().start >= term.range().start && v.range().end <= term.range().end && &text[v.range().start..v.range().end] == source));
                }
            }
            if *kind == S::StructureConstructor {
                for (kind, source) in [(K::TypeArguments, "[x]"), (K::FieldArgument, "value: y")] {
                    assert!(term.children().iter().map(|id| ast.node_view(*id).unwrap()).any(|v| v.kind() == &kind && &text[v.range().start..v.range().end] == source));
                }
            }
        }
        let suffix = marker.contains("multi");
        assert_eq!(chain.children().len(), if suffix { 2 } else { 1 });
        if suffix { let node = ast.node_view(chain.children()[1]).unwrap(); assert_eq!(&text[node.range().start..node.range().end], "non empty"); }
        let host_kind = if *list_kind == K::TypeArguments { K::ReserveItem } else { K::SetStatement };
        let host = ast.node_views().find(|v| v.kind() == &host_kind && v.range().start == host_start).unwrap();
        assert_eq!(host.range(), range(ast.source_id, host_start, semicolon + 1));
        assert_eq!(ast.node_view(*host.children().last().unwrap()).unwrap().as_token().unwrap().text.as_ref(), ";");
        let mut pending = vec![host.id()];
        let mut seen = std::collections::HashSet::new();
        let mut tokens = Vec::new();
        while let Some(id) = pending.pop() {
            assert!(seen.insert(id), "duplicate child in {marker}");
            let node = ast.node_view(id).unwrap();
            if node.as_token().is_some() { tokens.push(id); }
            pending.extend(node.children().iter().rev().copied());
        }
        assert_eq!(tokens, ast.token_views().filter(|v| v.range().start >= host_start && v.range().end <= semicolon + 1).map(|v| v.id()).collect::<Vec<_>>());
    }
    for (name, spelling, typed) in [("BareTagged", "tagged(~ Atom)", false), ("BareNonLinked", "non linked(~ Atom,x)", false), ("TypedTagged", "tagged(~ Atom) R", true)] {
        let host_start = text.find(&format!("theorem {name}:")).unwrap();
        let start = host_start + text[host_start..].find(spelling).unwrap();
        let assertion = ast.node_views().find(|v| v.kind() == &K::IsAssertion && v.range().start == host_start + format!("theorem {name}: ").len()).unwrap();
        let body = ast.node_view(*assertion.children().last().unwrap()).unwrap();
        assert_eq!(body.kind(), if typed { &K::TypeExpression } else { &K::AttributeTestChain });
        assert_eq!(body.range(), range(ast.source_id, start, start + spelling.len()));
        if typed { assert_eq!(&text[ast.node_view(body.children()[1]).unwrap().range().start..body.range().end], "R"); }
        else {
            assert_eq!(body.children().len(), 1);
            let attribute = ast.node_view(body.children()[0]).unwrap();
            assert_eq!(attribute.kind(), &K::AttributeRef);
            assert_eq!(attribute.range(), body.range());
            let arguments = attribute.children().iter().map(|id| ast.node_view(*id).unwrap()).filter(|v| v.kind() == &K::TermExpression).collect::<Vec<_>>();
            assert_eq!(arguments.iter().map(|v| &text[v.range().start..v.range().end]).collect::<Vec<_>>(), if name == "BareTagged" { vec!["~ Atom"] } else { vec!["~ Atom", "x"] });
            assert_eq!(ast.node_view(arguments[0].children()[0]).unwrap().syntax_kind(), S::PrefixExpression);
            if name == "BareNonLinked" { assert_eq!(ast.node_view(attribute.children()[0]).unwrap().as_token().unwrap().text.as_ref(), "non"); }
        }
        if name == "BareNonLinked" { assert!(ast.node_views().any(|v| v.kind() == &K::FormulaConstant(mizar_syntax::SurfaceFormulaConstant::Thesis) && v.range().start == start + spelling.len() + 3)); }
    }
    for (marker, list_kind, source, kind) in [
        ("reserve ordinary_value for Family", K::TypeArguments, "image(~ Atom)", S::ApplicationTerm),
        ("set generic_ordinary = pick", K::TemplateArguments, "image(~ Atom)", S::ApplicationTerm),
        ("reserve infix_control for Family", K::TypeArguments, "image(~ Atom) ++ Other", S::InfixExpression),
        ("set generic_postfix_control = pick", K::TemplateArguments, "image(~ Atom) !", S::PostfixExpression),
    ] {
        let start = text.find(marker).unwrap() + marker.len();
        let list = ast.node_views().find(|v| v.kind() == &list_kind && v.range().start == start).unwrap();
        let mut term = ast.node_view(list.children()[1]).unwrap();
        if list_kind == K::TemplateArguments { term = ast.node_view(term.children()[0]).unwrap(); }
        assert_eq!(term.kind(), &K::TermExpression);
        assert_eq!(&text[term.range().start..term.range().end], source);
        assert_eq!(ast.node_view(term.children()[0]).unwrap().syntax_kind(), kind);
    }
    let after = text.find("set after_attribute_dispatch").unwrap();
    assert!(ast.node_views().filter_map(|v| v.as_set_statement()).any(|v| v.range().start == after && !v.is_recovered()));
    let frontend = task257c4c1_frontend_output(&text, 129);
    let current = frontend.cache_keys.ast.as_ref().unwrap();
    assert_eq!(current.parser_version.version.as_ref(), "mizar-parser/surface-ast-v30");
    let mut retained = current.clone();
    retained.parser_version = mizar_frontend::parsing::ParserCacheKeyVersion::new("mizar-parser/surface-ast-v20");
    assert_eq!(current.token_stream_hash, retained.token_stream_hash);
    assert_eq!(current.parser_inputs_hash, retained.parser_inputs_hash);
    assert_ne!(current.stable_hash(), retained.stable_hash());
    let bytes = ast.canonical_bytes().expect("valid complete attribute dispatch storage");
    let restored = mizar_syntax::SurfaceAst::from_canonical_bytes(&bytes, ast.source_id).unwrap();
    assert_eq!(restored.snapshot_text_with_trivia(), ast.snapshot_text_with_trivia());
}

#[test]
fn attribute_dispatch_recovery_preserves_balanced_malformed_lists_and_successors() {
    use mizar_frontend::orchestration::{DiagnosticCode, DiagnosticLocation};
    use mizar_syntax::SurfaceNodeKind as K;
    let plan = build_test_plan(&step5c11_config()).unwrap();
    let case = plan.cases.iter().find(|case| case.id.0 == "fail_parser_attribute_dispatch_001").unwrap();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let output = super::super::formula_statement::step5c8_test_frontend(&text);
    assert_eq!(output.diagnostics.len(), 3, "{:?}", output.diagnostics);
    let ast = output.ast.unwrap();
    assert!(ast.trivia().skipped_token_ranges().is_empty());
    for ((marker, spelling, host_kind, list_kind, after), diagnostic) in [
        ("reserve trailing_comma for Family", "linked(~ Atom,)", K::ReserveItem, Some(K::TypeArguments), "after_bracket_trailing_comma"),
        ("set empty_arguments = pick", "linked()", K::SetStatement, Some(K::TemplateArguments), "after_template_empty_arguments"),
        ("theorem BareTrailingComma: x is ", "linked(~ Atom,)", K::TheoremItem, None, "after_bare_trailing_comma"),
    ].into_iter().zip(&output.diagnostics) {
        let host_start = text.find(marker).unwrap();
        let start = host_start + text[host_start..].find(spelling).unwrap();
        let close = start + spelling.len() - 1;
        let semicolon = host_start + text[host_start..].find(';').unwrap();
        assert_eq!(diagnostic.code, DiagnosticCode::Syntax("malformed_term_expression".into()));
        assert_eq!(diagnostic.location, DiagnosticLocation::SourceRange(range(ast.source_id, close, close + 1)));
        let attribute = ast.node_views().find(|v| v.kind() == &K::AttributeRef && v.range().start == start).unwrap();
        assert_eq!(attribute.range(), range(ast.source_id, start, close + 1));
        let children = attribute.children().iter().map(|id| ast.node_view(*id).unwrap()).collect::<Vec<_>>();
        let missing = children.iter().filter(|v| v.as_recovery() == Some(SyntaxRecoveryKind::MissingTerm)).collect::<Vec<_>>();
        assert_eq!(missing.len(), 1);
        assert_eq!(missing[0].range(), range(ast.source_id, close, close));
        assert!(missing[0].is_recovered());
        assert_eq!(children.last().unwrap().as_token().unwrap().text.as_ref(), ")");
        assert_eq!(children.last().unwrap().range(), range(ast.source_id, close, close + 1));
        let arguments = children.iter().filter(|v| v.kind() == &K::TermExpression).collect::<Vec<_>>();
        assert_eq!(arguments.iter().map(|v| &text[v.range().start..v.range().end]).collect::<Vec<_>>(), if spelling == "linked()" { vec![] } else { vec!["~ Atom"] });
        assert!(arguments.iter().all(|v| !v.is_recovered()));
        if let Some(kind) = list_kind {
            let list = ast.node_views().find(|v| v.kind() == &kind && v.range().start == host_start + marker.len()).unwrap();
            assert_eq!(list.children().len(), 3);
            assert_eq!(ast.node_view(list.children()[0]).unwrap().as_token().unwrap().text.as_ref(), "[");
            let bracket = ast.node_view(*list.children().last().unwrap()).unwrap();
            assert_eq!(bracket.as_token().unwrap().text.as_ref(), "]");
            let mut expression = ast.node_view(list.children()[1]).unwrap();
            if kind == K::TemplateArguments {
                assert_eq!(expression.kind(), &K::TemplateArgument);
                expression = ast.node_view(expression.children()[0]).unwrap();
                let reference = ast.node_views().find(|v| v.kind() == &K::TermReference && v.children().contains(&list.id())).unwrap();
                assert_eq!(reference.range().end, list.range().end);
                let application = ast.node_views().find(|v| v.kind() == &K::ApplicationTerm && v.children().first() == Some(&reference.id())).unwrap();
                assert_eq!(&text[list.range().end..application.range().end], "(x)");
                assert_eq!(application.range().end, semicolon);
            }
            assert_eq!(expression.kind(), &K::TypeExpression);
            let head = ast.node_view(expression.children()[1]).unwrap();
            assert_eq!(head.kind(), &K::TypeHead);
            assert_eq!(head.range(), range(ast.source_id, bracket.range().start - 1, bracket.range().start));
            assert_eq!(&text[head.range().start..head.range().end], "R");
        } else {
            let chain = ast.node_views().find(|v| v.kind() == &K::AttributeTestChain && v.children() == [attribute.id()]).unwrap();
            assert!(ast.node_views().any(|v| v.kind() == &K::IsAssertion && v.children().last() == Some(&chain.id())));
        }
        let host = ast.node_views().find(|v| v.kind() == &host_kind && v.range().start == host_start).unwrap();
        assert_eq!(host.range(), range(ast.source_id, host_start, semicolon + 1));
        assert_eq!(ast.node_view(*host.children().last().unwrap()).unwrap().as_token().unwrap().text.as_ref(), ";");
        let successor = format!("set {after} = sentinel;");
        let successor_start = text.find(&successor).unwrap();
        assert!(ast.node_views().filter_map(|v| v.as_set_statement()).any(|v| v.range() == range(ast.source_id, successor_start, successor_start + successor.len()) && !v.is_recovered()));
        let mut pending = vec![host.id()];
        let mut seen = std::collections::HashSet::new();
        let mut tokens = Vec::new();
        while let Some(id) = pending.pop() {
            assert!(seen.insert(id), "duplicate recovered child in {marker}");
            let node = ast.node_view(id).unwrap();
            if node.as_token().is_some() { tokens.push(id); }
            pending.extend(node.children().iter().rev().copied());
        }
        assert_eq!(tokens, ast.token_views().filter(|v| v.range().start >= host_start && v.range().end <= semicolon + 1).map(|v| v.id()).collect::<Vec<_>>());
    }
    assert_eq!(ast.node_views().filter(|v| v.as_recovery() == Some(SyntaxRecoveryKind::MissingTerm)).count(), 3);
    assert!(ast.node_views().filter_map(|v| v.as_recovery()).all(|kind| kind == SyntaxRecoveryKind::MissingTerm));
    let bytes = ast.canonical_bytes().expect("valid recovered complete attribute dispatch storage");
    let restored = mizar_syntax::SurfaceAst::from_canonical_bytes(&bytes, ast.source_id).unwrap();
    assert_eq!(restored.snapshot_text_with_trivia(), ast.snapshot_text_with_trivia());
}
