use super::*;

#[test]
fn algorithm_ordinary_occurrences_preserve_source_binding_targets() {
    use mizar_checker::binding_env::BindingKind;
    use mizar_checker::type_checker::{TermReference, TermStatus, TypeHeadRef};
    use mizar_checker::typed_ast::TypeEntryActual;
    use mizar_session::SourceAnchor;
    use mizar_syntax::ast::SurfaceNodeKind as K;

    let config = step5c11_config();
    let case = step5c14_return_case();
    for (label, text, declarations, expected_uses, captures) in [
        (
            "shadow_snapshot",
            "definition let a be object; terminating algorithm sample(a) -> object ensures result = a do snapshot before; var a := a; ghost var g := a; const c := a; a := c; assert not a = c; snapshot after; return a; end; end;",
            vec![15, 113, 131, 145],
            vec![
                (57, "header", Some(15)),
                (78, "term", None),
                (87, "term", Some(15)),
                (118, "term", Some(15)),
                (136, "term", Some(113)),
                (150, "term", Some(113)),
                (153, "lvalue", Some(113)),
                (158, "term", Some(145)),
                (172, "term", Some(113)),
                (176, "term", Some(145)),
                (202, "term", Some(113)),
            ],
            vec![("before", vec![15]), ("after", vec![113, 131, 145])],
        ),
        (
            "parameter_result",
            "definition let result be object; algorithm named(result) -> object do var result := result; return result; end; end;",
            vec![15, 74],
            vec![
                (49, "header", Some(15)),
                (84, "term", Some(15)),
                (99, "term", Some(74)),
            ],
            vec![],
        ),
        (
            "local_result",
            "definition let a be object; algorithm ordinary(a) -> object ensures result = a do var result := a; result := a; assert result = a; return result; end; end;",
            vec![15, 86],
            vec![
                (47, "header", Some(15)),
                (68, "term", None),
                (77, "term", Some(15)),
                (96, "term", Some(15)),
                (99, "lvalue", Some(86)),
                (109, "term", Some(15)),
                (119, "term", Some(86)),
                (128, "term", Some(15)),
                (138, "term", Some(86)),
            ],
            vec![],
        ),
    ] {
        let (source, typed, symbols) = super::super::source_registration_inputs(
            &config.workspace_root,
            &case,
            super::super::formula_statement::step5c8_test_frontend(text),
        )
        .unwrap();
        let checked =
            mizar_checker::type_checker::check_source_algorithm_types(&source, &typed, &symbols)
                .unwrap();
        let bindings = checked.bindings().bindings();
        assert_eq!(
            bindings
                .iter()
                .filter(|(_, binding)| binding.kind != BindingKind::Generated)
                .map(|(_, binding)| binding.declaration_range.start)
                .collect::<Vec<_>>(),
            declarations,
            "{label} source declarations"
        );
        let generated = bindings
            .iter()
            .filter(|(_, binding)| binding.kind == BindingKind::Generated)
            .collect::<Vec<_>>();
        assert_eq!(generated.len(), usize::from(label != "parameter_result"));
        if let Some((id, _)) = generated.first() {
            assert_eq!(bindings.iter().nth(1).unwrap().0, *id);
        }
        let inference = checked.inference();
        assert!(inference.diagnostics().is_empty());
        assert!(inference.facts().is_empty());
        assert!(inference.candidate_sets().is_empty());
        let mut actual_uses = Vec::new();
        for (_, term) in inference.terms().iter() {
            assert_eq!(term.status, TermStatus::Inferred);
            let Some(TermReference::Binding(binding)) = term.reference else {
                panic!("{label} missing binding")
            };
            let node = typed.node(term.site.node()).unwrap();
            let SourceAnchor::Range(range) = node.anchor else {
                panic!("{label} missing source range")
            };
            let role = match source
                .arena()
                .node(node.resolved_node.unwrap())
                .unwrap()
                .kind()
            {
                K::Token(_) => "header",
                K::Lvalue => "lvalue",
                K::TermReference => "term",
                _ => panic!("{label} unexpected ordinary use owner"),
            };
            let target = bindings.get(binding).unwrap();
            let declaration =
                (target.kind != BindingKind::Generated).then_some(target.declaration_range.start);
            actual_uses.push((range.start, role, declaration));
            let TypeEntryActual::Known(ty) = inference
                .type_entries()
                .get(term.type_entry)
                .unwrap()
                .actual
            else {
                panic!("{label} unknown type")
            };
            assert_eq!(
                inference.normalized_types().get(ty).unwrap().head,
                TypeHeadRef::BuiltinObject
            );
        }
        actual_uses.sort_by_key(|(start, _, _)| *start);
        assert_eq!(actual_uses, expected_uses, "{label} source use targets");
        assert_eq!(
            checked
                .snapshots()
                .values()
                .map(|(name, captures)| (
                    name.as_str(),
                    captures
                        .iter()
                        .map(|id| bindings.get(*id).unwrap().declaration_range.start)
                        .collect::<Vec<_>>()
                ))
                .collect::<Vec<_>>(),
            captures,
            "{label} source snapshot targets"
        );
        assert_eq!(checked.typed(), &typed);
        let core = mizar_core::elaborator::lower_source_algorithms(&checked).unwrap();
        assert_eq!(core, step5c14_static_core(&case, text).unwrap());
        eprintln!(
            "ALGORITHM_OCCURRENCE_BASELINE:{label}:{:?}",
            (&checked, &core)
        );
    }
}

#[test]
fn algorithm_formula_rhs_preserves_direct_payloads_terms_and_statement_boundaries() {
    use mizar_syntax::{SurfaceNodeKind as K, SyntaxKind as S, SurfaceFormulaPrefixOperator as P, SurfaceQuantifierKind as Q};
    let config = step5c11_config();
    let plan = build_test_plan(&config).unwrap();
    let case = plan.cases.iter().find(|case| case.id.0 == "pass_parser_algorithm_formula_rhs_001").unwrap();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let first = super::super::run_parse_only_case(&config.workspace_root, &config.workspace_root.join("tests"), case, 137);
    assert_eq!(first.status, super::super::ParseOnlyCaseStatus::Passed, "{first:?}");
    assert_eq!(first, super::super::run_parse_only_case(&config.workspace_root, &config.workspace_root.join("tests"), case, 137));
    let output = super::super::formula_statement::step5c8_test_frontend(&text);
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    let ast = output.ast.unwrap();
    assert!(ast.node_views().all(|v| !v.is_recovered()));
    assert!(ast.trivia().skipped_token_ranges().is_empty());
    for (kind, syntax, count) in [(K::VariableBinding, S::VariableBinding, 17), (K::AssignmentStatement, S::AssignmentStatement, 3), (K::ReturnStatement, S::ReturnStatement, 2)] {
        assert_eq!(ast.node_views().filter(|v| v.kind() == &kind).count(), count);
        assert_eq!(ast.rowan_root().descendants().filter(|v| v.kind() == syntax).count(), count);
        assert_eq!(ast.snapshot_text().matches(&format!("{kind:?}")).count(), count);
    }
    for (marker, owner_kind, rhs, root_kind) in [
        ("eq := x = y,", K::VariableBinding, "x = y", K::BuiltinPredicateApplication),
        ("membership :=", K::VariableBinding, "x is object", K::IsAssertion),
        ("negative := not", K::VariableBinding, "not (x = y)", K::PrefixFormula(P::Not)),
        ("impossible :=", K::VariableBinding, "contradiction", K::FormulaConstant(mizar_syntax::SurfaceFormulaConstant::Contradiction)),
        ("eq := P(x)", K::AssignmentStatement, "P(x) & Q(x)", K::BinaryFormula(mizar_syntax::SurfaceFormulaBinaryOperator { connective: mizar_syntax::SurfaceFormulaConnective::And, repeated: false })),
        ("ghost negative :=", K::AssignmentStatement, "((x = y))", K::ParenthesizedFormula),
        ("quantified :=", K::VariableBinding, "for a,b being object holds a = b", K::QuantifiedFormula(Q::Universal)),
        ("exists :=", K::VariableBinding, "ex a being object st a = x", K::QuantifiedFormula(Q::Existential)),
        ("after_binder :=", K::VariableBinding, "x = y", K::BuiltinPredicateApplication),
        ("eq := (x,y)", K::AssignmentStatement, "(x,y) |-> x = y", K::BuiltinPredicateApplication),
        ("formula_after :=", K::VariableBinding, "x = y", K::BuiltinPredicateApplication),
        ("compared :=", K::VariableBinding, "{x where x is set : x = x} = z", K::BuiltinPredicateApplication),
        ("return x = y by", K::ReturnStatement, "x = y", K::BuiltinPredicateApplication),
        ("return x = x proof", K::ReturnStatement, "x = x", K::BuiltinPredicateApplication),
    ] {
        let start = text.find(marker).unwrap();
        let rhs_start = start + text[start..].find(rhs).unwrap();
        let owner = ast.node_views().find(|v| v.kind() == &owner_kind && v.range().start == start).unwrap();
        let payloads = owner.children().iter().map(|id| ast.node_view(*id).unwrap()).filter(|v| matches!(v.kind(), K::FormulaExpression | K::TermExpression)).collect::<Vec<_>>();
        assert_eq!(payloads.len(), 1, "{marker}");
        let rhs_view = payloads[0];
        assert_eq!(rhs_view.kind(), &K::FormulaExpression, "{marker}");
        assert_eq!(rhs_view.range(), range(ast.source_id, rhs_start, rhs_start + rhs.len()));
        assert_eq!(rhs_view.children().len(), 1);
        let root = ast.node_view(rhs_view.children()[0]).unwrap();
        assert_eq!(root.kind(), &root_kind, "{marker}");
        assert_eq!(root.range(), rhs_view.range());
        if root_kind == K::BuiltinPredicateApplication {
            assert_eq!(root.children().len(), 3);
            let left = ast.node_view(root.children()[0]).unwrap();
            assert_eq!(left.kind(), &K::TermExpression);
            assert_eq!(ast.node_view(root.children()[1]).unwrap().as_token().unwrap().text.as_ref(), "=");
            assert_eq!(ast.node_view(root.children()[2]).unwrap().kind(), &K::TermExpression);
            if rhs.starts_with("(x,y)") || rhs.starts_with('{') {
                let term = ast.node_view(left.children()[0]).unwrap();
                assert_eq!(term.kind(), if rhs.starts_with('(') { &K::BinderExpression } else { &K::SetComprehension });
                assert_eq!(&text[left.range().start..left.range().end], if rhs.starts_with('(') { "(x,y) |-> x" } else { "{x where x is set : x = x}" });
            }
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
            assert_eq!(ast.node_views().filter(|v| v.kind() == &K::ParenthesizedFormula && v.range().start >= rhs_start && v.range().end <= rhs_start + rhs.len()).count(), 2);
        }
        if matches!(root_kind, K::QuantifiedFormula(_)) {
            let segment = ast.node_view(root.children()[1]).unwrap();
            assert_eq!(segment.kind(), &K::QuantifierVariableSegment);
            assert_eq!(&text[segment.range().start..segment.range().end], if rhs.starts_with("for") { "a,b being object" } else { "a being object" });
        }
        let mut pending = vec![owner.id()];
        let mut tokens = Vec::new();
        let mut seen = std::collections::HashSet::new();
        while let Some(id) = pending.pop() {
            assert!(seen.insert(id), "duplicate RHS owner child: {marker}");
            let node = ast.node_view(id).unwrap();
            if node.as_token().is_some() { tokens.push(id); }
            pending.extend(node.children().iter().rev().copied());
        }
        assert_eq!(tokens, ast.token_views().filter(|v| v.range().start >= owner.range().start && v.range().end <= owner.range().end).map(|v| v.id()).collect::<Vec<_>>());
    }
    for (name, rhs, root_kind) in [
        ("term_tail", "z", K::TermReference), ("ambiguous", "P", K::TermReference), ("call", "P(x)", K::ApplicationTerm),
        ("binder", "x |-> x", K::BinderExpression), ("opaque", "{x where x is set : x = x}", K::SetComprehension),
        ("nested", "f({x where x is set : x = x})", K::ApplicationTerm), ("bracket", "[{x where x is set : x = x}, y]", K::ApplicationTerm), ("after_returns", "z", K::TermReference),
    ] {
        let marker = format!("{name} := ");
        let start = text.find(&marker).unwrap();
        let owner = ast.node_views().filter_map(|v| v.as_variable_binding()).find(|v| v.range().start == start).unwrap();
        assert_eq!(owner.children().len(), 3);
        let payload = ast.node_view(owner.children()[2]).unwrap();
        assert_eq!(payload.kind(), &K::TermExpression, "{name}");
        assert_eq!(payload.range(), range(ast.source_id, start + marker.len(), start + marker.len() + rhs.len()));
        assert_eq!(ast.node_view(payload.children()[0]).unwrap().kind(), &root_kind);
        if rhs.contains('{') {
            assert!(ast.node_views().any(|v| v.kind() == &K::FormulaExpression && v.range().start > payload.range().start && v.range().end < payload.range().end));
        }
    }
    for (marker, expected) in [
        ("var eq :=", ["var", "eq := x = y", ",", "term_tail := z", "as", "object", "by TypeOk", ";"].as_slice()),
        ("var binder :=", &["var", "binder := x |-> x", ",", "after_binder := x = y", ";"]),
        ("var opaque :=", &["var", "opaque := {x where x is set : x = x}", ",", "formula_after := x = y", ";"]),
        ("return x = y by", &["return", "x = y", "by ReturnOk", ";"]),
        ("return x = x proof", &["return", "x = x", "proof thus thesis; end", ";"]),
    ] {
        let start = text.find(marker).unwrap();
        let owner = ast.node_views().find(|v| matches!(v.kind(), K::VariableDeclaration | K::ReturnStatement) && v.range().start == start).unwrap();
        assert_eq!(owner.children().iter().map(|id| { let child = ast.node_view(*id).unwrap(); &text[child.range().start..child.range().end] }).collect::<Vec<_>>(), expected);
    }
    let body = ast.node_views().find(|v| v.kind() == &K::AlgorithmStatementList).unwrap();
    assert_eq!(body.children().iter().map(|id| { let child = ast.node_view(*id).unwrap(); &text[child.range().start..child.range().end] }).collect::<Vec<_>>(), text.lines().skip(2).take(18).map(str::trim).collect::<Vec<_>>());
    let proof_start = text.find("proof thus thesis; end").unwrap();
    assert!(ast.node_views().any(|v| v.kind() == &K::ProofBlock && v.range() == range(ast.source_id, proof_start, proof_start + "proof thus thesis; end".len())));
    let final_start = text.find("set after_formula_rhs = sentinel;").unwrap();
    assert!(ast.node_views().filter_map(|v| v.as_set_statement()).any(|v| v.range() == range(ast.source_id, final_start, text.trim_end().len())));
    let frontend = task257c4c1_frontend_output(&text, 137);
    assert!(frontend.diagnostics.is_empty());
    assert_eq!(frontend.ast.as_ref().unwrap().snapshot_text_with_trivia(), ast.snapshot_text_with_trivia());
    let repeated = task257c4c1_frontend_output(&text, 137);
    assert_eq!(frontend.cache_keys, repeated.cache_keys);
    assert_eq!(frontend.ast, repeated.ast);
    let current = frontend.cache_keys.ast.as_ref().unwrap();
    assert_eq!(current.parser_version.version.as_ref(), "mizar-parser/surface-ast-v30");
    let mut retained = current.clone();
    retained.parser_version = mizar_frontend::parsing::ParserCacheKeyVersion::new("mizar-parser/surface-ast-v24");
    assert_eq!(current.token_stream_hash, retained.token_stream_hash);
    assert_eq!(current.parser_inputs_hash, retained.parser_inputs_hash);
    assert_ne!(current.stable_hash(), retained.stable_hash());
    let bytes = ast.canonical_bytes().unwrap();
    let restored = mizar_syntax::SurfaceAst::from_canonical_bytes(&bytes, ast.source_id).unwrap();
    assert_eq!(restored.snapshot_text_with_trivia(), ast.snapshot_text_with_trivia());
}

#[test]
fn algorithm_formula_rhs_recovery_and_thesis_restrictions_preserve_owned_payloads() {
    use mizar_frontend::orchestration::{DiagnosticCode, DiagnosticLocation};
    use mizar_syntax::{SurfaceNodeKind as K, SurfaceFormulaPrefixOperator as P, SurfaceQuantifierKind as Q};
    let config = step5c11_config();
    let plan = build_test_plan(&config).unwrap();
    let case = plan.cases.iter().find(|case| case.id.0 == "fail_parser_algorithm_formula_rhs_001").unwrap();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let first = super::super::run_parse_only_case(&config.workspace_root, &config.workspace_root.join("tests"), case, 138);
    assert_eq!(first.status, super::super::ParseOnlyCaseStatus::Passed, "{first:?}");
    assert_eq!(first, super::super::run_parse_only_case(&config.workspace_root, &config.workspace_root.join("tests"), case, 138));
    let output = super::super::formula_statement::step5c8_test_frontend(&text);
    assert_eq!(output.diagnostics.len(), 7, "{:?}", output.diagnostics);
    let ast = output.ast.unwrap();
    for (diagnostic, (code, offset, word, message)) in output.diagnostics.iter().zip([
        ("malformed_term_expression", 60, ";", "expected term after built-in predicate"),
        ("malformed_formula_expression", 104, ";", "expected formula after `not`"),
        ("malformed_formula_expression", 150, ";", "expected `)` to close formula delimiter"),
        ("malformed_formula_expression", 198, ",", "expected formula after `not`"),
        ("malformed_formula_expression", 254, "thesis", "`thesis` is restricted to explicit computation inputs"),
        ("malformed_formula_expression", 334, "thesis", "`thesis` is restricted to explicit computation inputs"),
        ("malformed_term_expression", 391, "thesis", "unexpected token in return statement"),
    ]) {
        assert_eq!(&text[offset..offset + word.len()], word);
        assert_eq!(diagnostic.code, DiagnosticCode::Syntax(code.into()));
        assert_eq!(diagnostic.message.as_ref(), message);
        assert_eq!(diagnostic.location, DiagnosticLocation::SourceRange(range(ast.source_id, offset, offset + word.len())));
        if offset == 150 {
            let opener = text.find("return (x = y").unwrap() + "return ".len();
            assert_eq!(diagnostic.secondary, [mizar_session::SourceAnchor::Range(range(ast.source_id, opener, opener + 1))]);
            assert_eq!(diagnostic.recovery_note.as_deref(), Some("insert `)` before continuing"));
        } else {
            assert!(diagnostic.secondary.is_empty());
            assert_eq!(diagnostic.recovery_note.as_deref(), Some(if code == "malformed_term_expression" { "repair the term expression before continuing" } else { "repair the formula expression before continuing" }));
        }
    }
    for (kind, count) in [(SyntaxRecoveryKind::MissingTerm, 1), (SyntaxRecoveryKind::MissingFormula, 2), (SyntaxRecoveryKind::UnmatchedOpeningDelimiter, 1), (SyntaxRecoveryKind::SkippedToken, 1)] {
        assert_eq!(ast.node_views().filter(|v| v.as_recovery() == Some(kind)).count(), count);
    }
    assert_eq!(ast.node_views().filter_map(|v| v.as_recovery()).count(), 5);
    for (marker, owner_kind, rhs, root_kind, anchor, recovery_kind) in [
        ("eq := x =", K::VariableBinding, "x =", K::BuiltinPredicateApplication, 60, Some(SyntaxRecoveryKind::MissingTerm)),
        ("x := not ;", K::AssignmentStatement, "not", K::PrefixFormula(P::Not), 104, Some(SyntaxRecoveryKind::MissingFormula)),
        ("return (x = y", K::ReturnStatement, "(x = y", K::ParenthesizedFormula, 150, Some(SyntaxRecoveryKind::UnmatchedOpeningDelimiter)),
        ("gaps := not", K::VariableBinding, "not", K::PrefixFormula(P::Not), 198, Some(SyntaxRecoveryKind::MissingFormula)),
        ("x := not thesis", K::AssignmentStatement, "not thesis", K::PrefixFormula(P::Not), 254, None),
        ("return for a,b", K::ReturnStatement, "for a,b being object holds thesis", K::QuantifiedFormula(Q::Universal), 334, None),
    ] {
        let start = text.find(marker).unwrap();
        let owner = ast.node_views().find(|v| v.kind() == &owner_kind && v.range().start == start).unwrap();
        let payloads = owner.children().iter().map(|id| ast.node_view(*id).unwrap()).filter(|v| matches!(v.kind(), K::TermExpression | K::FormulaExpression)).collect::<Vec<_>>();
        assert_eq!(payloads.len(), 1);
        let payload = payloads[0];
        let rhs_start = start + text[start..].find(rhs).unwrap();
        assert_eq!(payload.kind(), &K::FormulaExpression);
        assert_eq!(payload.range(), range(ast.source_id, rhs_start, rhs_start + rhs.len()));
        let root = ast.node_view(payload.children()[0]).unwrap();
        assert_eq!(root.kind(), &root_kind);
        if let Some(kind) = recovery_kind {
            let recovery = ast.node_view(*root.children().last().unwrap()).unwrap();
            assert_eq!(recovery.as_recovery(), Some(kind));
            assert_eq!(recovery.range(), range(ast.source_id, anchor, anchor));
            assert!(recovery.children().is_empty());
        } else {
            assert!(!owner.is_recovered());
            assert!(ast.node_views().filter(|v| v.range().start >= payload.range().start && v.range().end <= payload.range().end).all(|v| v.as_recovery().is_none()));
            let thesis = ast.node_views().find(|v| v.kind() == &K::FormulaConstant(mizar_syntax::SurfaceFormulaConstant::Thesis) && v.range().start == anchor).unwrap();
            assert_eq!(thesis.range(), range(ast.source_id, anchor, anchor + "thesis".len()));
        }
    }
    let direct_start = text.find("return thesis;").unwrap();
    let direct = ast.node_views().filter_map(|v| v.as_return_statement()).find(|v| v.range().start == direct_start).unwrap();
    assert_eq!(direct.children().len(), 3);
    let skipped = ast.node_view(direct.children()[1]).unwrap();
    assert_eq!(skipped.as_recovery(), Some(SyntaxRecoveryKind::SkippedToken));
    assert_eq!(skipped.range(), range(ast.source_id, 391, 397));
    assert_eq!(skipped.children().len(), 1);
    assert_eq!(ast.node_view(skipped.children()[0]).unwrap().as_token().unwrap().text.as_ref(), "thesis");
    assert_eq!(ast.trivia().skipped_token_ranges().len(), 1);
    assert_eq!(ast.trivia().skipped_token_ranges()[0].range, skipped.range());
    for name in ["retained", "after_equality", "after_prefix", "after_group", "after_comma", "after_nested_thesis", "after_quantified_thesis", "after_direct_thesis"] {
        let spelling = format!("{name} := x");
        let start = text.find(&spelling).unwrap();
        let binding = ast.node_views().filter_map(|v| v.as_variable_binding()).find(|v| v.range().start == start).unwrap();
        assert_eq!(binding.range(), range(ast.source_id, start, start + spelling.len()));
        assert!(!binding.is_recovered());
        assert_eq!(binding.children().len(), 3);
        let payload = ast.node_view(binding.children()[2]).unwrap();
        assert_eq!(payload.kind(), &K::TermExpression);
        assert_eq!(payload.range(), range(ast.source_id, start + spelling.len() - 1, start + spelling.len()));
        assert_eq!(ast.node_view(payload.children()[0]).unwrap().kind(), &K::TermReference);
    }
    let gaps_start = text.find("var gaps :=").unwrap();
    let gaps = ast.node_views().find(|v| v.kind() == &K::VariableDeclaration && v.range().start == gaps_start).unwrap();
    assert_eq!(gaps.children().iter().map(|id| { let v = ast.node_view(*id).unwrap(); &text[v.range().start..v.range().end] }).collect::<Vec<_>>(), ["var", "gaps := not", ",", "retained := x", ";"]);
    for owner in ast.node_views().filter(|v| matches!(v.kind(), K::VariableDeclaration | K::AssignmentStatement | K::ReturnStatement)) {
        let last = ast.node_view(*owner.children().last().unwrap()).unwrap();
        assert_eq!(last.as_token().unwrap().text.as_ref(), ";");
        assert_eq!(last.range().end, owner.range().end);
    }
    let final_start = text.find("set after_formula_rhs_failure = sentinel;").unwrap();
    assert!(ast.node_views().filter_map(|v| v.as_set_statement()).any(|v| v.range() == range(ast.source_id, final_start, text.trim_end().len()) && !v.is_recovered()));
    let bytes = ast.canonical_bytes().unwrap();
    let restored = mizar_syntax::SurfaceAst::from_canonical_bytes(&bytes, ast.source_id).unwrap();
    assert_eq!(restored.snapshot_text_with_trivia(), ast.snapshot_text_with_trivia());
}

#[test]
fn algorithm_formula_rhs_has_clean_syntax_but_no_bounded_semantic_admission() {
    use mizar_resolve::names::{SourceVariableScopeError, SourceVariableScopeResolver};
    use mizar_syntax::SurfaceNodeKind as K;

    let config = step5c11_config();
    let case = step5c14_state_case();
    let base = concat!(
        "definition\n  let x be object;\n",
        "  algorithm rhs(x) -> object\n  do\n",
        "    var local := x;\n    local := x;\n    return x;\n",
        "  end;\nend;\n",
    );
    for (selected, text) in [
        (None, base.to_owned()),
        (Some(0), base.replace("var local := x;", "var local := x = x;")),
        (Some(1), base.replace("\n    local := x;", "\n    local := x = x;")),
        (Some(2), base.replace("return x;", "return x = x;")),
    ] {
        let frontend = super::super::formula_statement::step5c8_test_frontend(&text);
        assert!(frontend.diagnostics.is_empty(), "{selected:?}: {:?}", frontend.diagnostics);
        let ast = frontend.ast.as_ref().expect("clean source must retain AST");
        assert!(ast.node_views().all(|node| !node.is_recovered() && node.as_recovery().is_none()));
        for (index, kind) in [K::VariableBinding, K::AssignmentStatement, K::ReturnStatement]
            .into_iter().enumerate()
        {
            let owner = ast.node_views().find(|node| node.kind() == &kind).unwrap();
            let rhs = owner.child_views().filter(|child| {
                matches!(child.kind(), K::TermExpression | K::FormulaExpression)
            }).collect::<Vec<_>>();
            assert_eq!(rhs.len(), 1, "{selected:?}: {kind:?}");
            assert_eq!(
                rhs[0].kind(),
                if selected == Some(index) { &K::FormulaExpression } else { &K::TermExpression },
                "{selected:?}: {kind:?}",
            );
            assert_eq!(
                &text[rhs[0].range().start..rhs[0].range().end],
                if selected == Some(index) { "x = x" } else { "x" },
            );
        }
        let (source, typed, symbols) = super::super::source_registration_inputs(
            &config.workspace_root, &case, frontend,
        ).unwrap();
        let scope = SourceVariableScopeResolver::resolve_algorithm_occurrences(&source, &symbols);
        let checked = mizar_checker::type_checker::check_source_algorithm_types(
            &source, &typed, &symbols,
        );
        if selected.is_some() {
            assert_eq!(scope.unwrap_err(), SourceVariableScopeError::InvalidShape);
            assert_eq!(checked.unwrap_err(), "algorithms.unsupported_source_types");
        } else {
            scope.unwrap();
            let checked = checked.unwrap();
            let core = mizar_core::elaborator::lower_source_algorithms(&checked).unwrap();
            assert_eq!(core.algorithms().len(), 1);
        }
    }
}

#[test]
fn algorithm_boolean_operands_preserve_references_predicates_and_declaration_tails() {
    use mizar_syntax::{SurfaceFormulaBinaryOperator as B, SurfaceFormulaConnective as C, SurfaceFormulaPrefixOperator as P, SurfaceNodeKind as K, SyntaxKind as S};
    let config = step5c11_config();
    let plan = build_test_plan(&config).unwrap();
    let case = plan.cases.iter().find(|case| case.id.0 == "pass_parser_algorithm_boolean_operands_001").unwrap();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let first = super::super::run_parse_only_case(&config.workspace_root, &config.workspace_root.join("tests"), case, 161);
    assert_eq!(first.status, super::super::ParseOnlyCaseStatus::Passed, "{first:?}");
    assert_eq!(first, super::super::run_parse_only_case(&config.workspace_root, &config.workspace_root.join("tests"), case, 161));
    let output = super::super::formula_statement::step5c8_test_frontend(&text);
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    let ast = output.ast.unwrap();
    assert!(ast.node_views().all(|v| !v.is_recovered() && v.as_recovery().is_none()));
    assert!(ast.trivia().skipped_token_ranges().is_empty());
    for (kind, syntax, count) in [(K::VariableDeclaration, S::VariableDeclaration, 31), (K::VariableBinding, S::VariableBinding, 38), (K::AssignmentStatement, S::AssignmentStatement, 2), (K::ReturnStatement, S::ReturnStatement, 2)] {
        assert_eq!(ast.node_views().filter(|v| v.kind() == &kind).count(), count);
        assert_eq!(ast.rowan_root().descendants().filter(|v| v.kind() == syntax).count(), count);
        assert_eq!(ast.snapshot_text().matches(&format!("{kind:?}")).count(), count);
    }
    let and = K::BinaryFormula(B { connective: C::And, repeated: false });
    for (marker, owner_kind, rhs, root_kind, references) in [
        ("bare := p & q;", K::VariableBinding, "p & q", and.clone(), ["p", "q"].as_slice()),
        ("negative :=", K::VariableBinding, "not p", K::PrefixFormula(P::Not), &["p"]),
        ("repeated := p", K::VariableBinding, "p & p", and.clone(), &["p", "p"]),
        ("grouped :=", K::VariableBinding, "(p) & ((q))", and.clone(), &["p", "q"]),
        ("bare := p & (", K::AssignmentStatement, "p & (q or not r)", and.clone(), &["p", "q", "r"]),
        ("ghost repeated :=", K::AssignmentStatement, "not (p & q)", K::PrefixFormula(P::Not), &["p", "q"]),
        ("mixed :=", K::VariableBinding, "p & x = y", and.clone(), &["p"]),
        ("inline :=", K::VariableBinding, "P(x) & q", and.clone(), &["q"]),
        ("ready :=", K::VariableBinding, "p & local_ready", and.clone(), &["p"]),
        ("typed :=", K::VariableBinding, "p & q", and.clone(), &["p", "q"]),
        ("typed_group :=", K::VariableBinding, "not (q)", K::PrefixFormula(P::Not), &["q"]),
        ("init_tail :=", K::VariableBinding, "p & q", and.clone(), &["p", "q"]),
        ("as_tail :=", K::VariableBinding, "p & q", and.clone(), &["p", "q"]),
        ("empty_tail :=", K::VariableBinding, "p & q", and.clone(), &["p", "q"]),
        ("many_tail :=", K::VariableBinding, "p & q", and.clone(), &["p", "q"]),
        ("last_many :=", K::VariableBinding, "not r", K::PrefixFormula(P::Not), &["r"]),
        ("group_tail :=", K::VariableBinding, "not (q)", K::PrefixFormula(P::Not), &["q"]),
        ("nested_tail :=", K::VariableBinding, "not ((q))", K::PrefixFormula(P::Not), &["q"]),
        ("return p & q by", K::ReturnStatement, "p & q", and.clone(), &["p", "q"]),
        ("return not (q)", K::ReturnStatement, "not (q)", K::PrefixFormula(P::Not), &["q"]),
        ("after_returns :=", K::VariableBinding, "not p", K::PrefixFormula(P::Not), &["p"]),
    ] {
        let start = text.find(marker).unwrap();
        let rhs_start = start + text[start..].find(rhs).unwrap();
        let owner = ast.node_views().find(|v| v.kind() == &owner_kind && v.range().start == start).unwrap();
        let payloads = owner.child_views().filter(|v| matches!(v.kind(), K::FormulaExpression | K::TermExpression)).collect::<Vec<_>>();
        assert_eq!(payloads.len(), 1, "{marker}");
        let payload = payloads[0];
        assert_eq!(payload.kind(), &K::FormulaExpression, "{marker}");
        assert_eq!(payload.range(), range(ast.source_id, rhs_start, rhs_start + rhs.len()));
        assert_eq!(payload.children().len(), 1);
        let root = ast.node_view(payload.children()[0]).unwrap();
        assert_eq!(root.kind(), &root_kind, "{marker}");
        assert_eq!(root.range(), payload.range());
        let mut found = Vec::new();
        let mut pending = vec![payload.id()];
        while let Some(id) = pending.pop() {
            let parent = ast.node_view(id).unwrap();
            for child in parent.child_views() {
                if child.kind() == &K::TermExpression && matches!(parent.kind(), K::FormulaExpression | K::BinaryFormula(_) | K::PrefixFormula(_)) {
                    assert_eq!(child.children().len(), 1);
                    let reference = ast.node_view(child.children()[0]).unwrap();
                    assert_eq!(reference.kind(), &K::TermReference, "{marker}");
                    assert_eq!(reference.range(), child.range());
                    assert_eq!(reference.children().len(), 1);
                    let token = ast.node_view(reference.children()[0]).unwrap();
                    assert_eq!(token.range(), child.range());
                    assert_eq!(token.as_token().unwrap().text.as_ref(), &text[child.range().start..child.range().end]);
                    found.push((child.range().start, token.as_token().unwrap().text.as_ref()));
                }
            }
            pending.extend(parent.children().iter().rev().copied());
        }
        found.sort_by_key(|(start, _)| *start);
        assert_eq!(found.iter().map(|(_, word)| *word).collect::<Vec<_>>(), references, "{marker}");
        if root_kind == and {
            let (left, right) = rhs.split_once(" & ").unwrap();
            assert_eq!(root.child_views().map(|v| &text[v.range().start..v.range().end]).collect::<Vec<_>>(), [left, "&", right]);
        } else {
            assert_eq!(root.child_views().map(|v| &text[v.range().start..v.range().end]).collect::<Vec<_>>(), ["not", &rhs[4..]]);
        }
        if rhs == "p & (q or not r)" {
            let group = ast.node_view(root.children()[2]).unwrap();
            assert_eq!(group.kind(), &K::ParenthesizedFormula);
            let expression = ast.node_view(group.children()[1]).unwrap();
            let nested = ast.node_view(expression.children()[0]).unwrap();
            assert_eq!(nested.kind(), &K::BinaryFormula(B { connective: C::Or, repeated: false }));
            assert_eq!(nested.child_views().map(|v| &text[v.range().start..v.range().end]).collect::<Vec<_>>(), ["q", "or", "not r"]);
        }
    }
    for (name, rhs, root_kind) in [("term", "p", K::TermReference), ("term_group", "(p)", K::ParenthesizedTerm), ("call", "P(x)", K::ApplicationTerm), ("binder", "x |-> x", K::BinderExpression), ("next_init", "x = y", K::BuiltinPredicateApplication), ("next_group", "x = y", K::BuiltinPredicateApplication)] {
        let marker = format!("{name} := ");
        let start = text.find(&marker).unwrap();
        let owner = ast.node_views().filter_map(|v| v.as_variable_binding()).find(|v| v.range().start == start).unwrap();
        assert_eq!(owner.children().len(), 3);
        let payload = ast.node_view(owner.children()[2]).unwrap();
        assert_eq!(payload.kind(), if name.starts_with("next_") { &K::FormulaExpression } else { &K::TermExpression });
        assert_eq!(payload.range(), range(ast.source_id, start + marker.len(), start + marker.len() + rhs.len()));
        assert_eq!(ast.node_view(payload.children()[0]).unwrap().kind(), &root_kind);
    }
    for (name, segment_text, argument_kind) in [
        ("pair", "q,r local_pair", K::TermReference), ("trip", "q,r,s local_trip", K::TermReference),
        ("grouped_pair", "(q),r local_pair", K::ParenthesizedTerm), ("grouped_trip", "((q)),r,s local_trip", K::ParenthesizedTerm),
        ("negated", "q,r do not local_pair", K::TermReference), ("complex", "q,f(r) local_pair", K::ApplicationTerm),
        ("container", "q,[r,s] local_pair", K::ApplicationTerm), ("binder_arg", "q,(r,s) |-> r local_pair", K::BinderExpression),
    ] {
        let start = text.find(&format!("{name} := not ")).unwrap() + name.len() + " := not ".len();
        let segment = ast.node_views().find(|v| v.kind() == &K::PredicateSegment && v.range() == range(ast.source_id, start, start + segment_text.len())).unwrap();
        let head = segment.child_views().find(|v| v.kind() == &K::PredicateHead).unwrap();
        assert_eq!(&text[head.range().start..head.range().end], if name.ends_with("trip") { "local_trip" } else { "local_pair" });
        let terms = segment.child_views().filter(|v| v.kind() == &K::TermExpression).collect::<Vec<_>>();
        assert_eq!(terms.len(), if name.ends_with("trip") { 3 } else { 2 });
        let argument = if name.starts_with("grouped_") { terms[0] } else { *terms.last().unwrap() };
        assert_eq!(ast.node_view(argument.children()[0]).unwrap().kind(), &argument_kind);
        assert_eq!(segment.child_views().filter_map(|v| v.as_token().map(|t| t.text.as_ref())).collect::<Vec<_>>(), if name == "negated" { vec![",", "do", "not"] } else if name.ends_with("trip") { vec![",", ","] } else { vec![","] });
        let token = ast.token_views().find(|v| v.range() == head.range()).unwrap().as_token().unwrap();
        assert_eq!(token.kind, mizar_syntax::SurfaceTokenKind::UserSymbol);
    }
    for name in ["atomic", "nested_atomic", "binder_atom"] {
        let start = text.find(&format!("{name} := not ")).unwrap() + name.len() + " := not ".len();
        let atom = ast.node_views().find(|v| v.kind() == &K::BuiltinPredicateApplication && v.range().start == start).unwrap();
        let left = ast.node_view(atom.children()[0]).unwrap();
        assert_eq!(left.kind(), &K::TermExpression);
        assert_eq!(ast.node_view(left.children()[0]).unwrap().kind(), &K::ParenthesizedTerm);
    }
    for (marker, expected) in [
        ("var typed :=", ["var", "typed := p & q", "as", "expr of boolean", "by TypeOk", ";"].as_slice()),
        ("var typed_group :=", &["var", "typed_group := not (q)", "as", "expr of boolean", "proof thus thesis; end", ";"]),
        ("var init_tail :=", &["var", "init_tail := p & q", ",", "next_init := x = y", ";"]),
        ("var as_tail :=", &["var", "as_tail := p & q", ",", "next_as", "as", "expr of boolean", "proof thus thesis; end", ";"]),
        ("var empty_tail :=", &["var", "empty_tail := p & q", ",", "next_empty", ";"]),
        ("var many_tail :=", &["var", "many_tail := p & q", ",", "next_many", ",", "last_many := not r", ";"]),
        ("var group_tail :=", &["var", "group_tail := not (q)", ",", "next_group := x = y", ";"]),
        ("var nested_tail :=", &["var", "nested_tail := not ((q))", ",", "next_nested", "as", "expr of boolean", ";"]),
        ("return p & q by", &["return", "p & q", "by ReturnOk", ";"]),
        ("return not (q)", &["return", "not (q)", "proof thus thesis; end", ";"]),
    ] {
        let start = text.find(marker).unwrap();
        let owner = ast.node_views().find(|v| matches!(v.kind(), K::VariableDeclaration | K::ReturnStatement) && v.range().start == start).unwrap();
        assert_eq!(owner.child_views().map(|v| &text[v.range().start..v.range().end]).collect::<Vec<_>>(), expected, "{marker}");
        if expected.contains(&"as") {
            let parts = owner.child_views().collect::<Vec<_>>();
            let as_index = parts.iter().position(|v| v.as_token().is_some_and(|t| t.text.as_ref() == "as")).unwrap();
            assert_eq!(parts[as_index + 1].kind(), &K::TypeExpression);
            assert!(parts[..as_index].iter().filter(|v| v.kind() == &K::VariableBinding).all(|v| v.child_views().all(|c| c.kind() != &K::TypeExpression)));
        }
    }
    let list = ast.node_views().find(|v| v.kind() == &K::AlgorithmStatementList).unwrap();
    assert_eq!(list.children().len(), 35);
    assert!(list.child_views().all(|v| matches!(v.kind(), K::VariableDeclaration | K::AssignmentStatement | K::ReturnStatement)));
    assert_eq!(&text[ast.node_view(*list.children().last().unwrap()).unwrap().range().start..list.range().end], "var after_returns := not p;");
    for proof in ast.node_views().filter(|v| v.kind() == &K::ProofBlock) {
        assert_eq!(&text[proof.range().start..proof.range().end], "proof thus thesis; end");
        assert!(!ast.node_views().any(|v| v.kind() == &K::FormulaExpression && v.range().start < proof.range().start && v.range().end > proof.range().start));
    }
    let final_start = text.find("set after_boolean_operands = sentinel;").unwrap();
    assert!(ast.node_views().filter_map(|v| v.as_set_statement()).any(|v| v.range() == range(ast.source_id, final_start, text.trim_end().len())));
    let mut pending = vec![ast.node_views().find(|v| v.kind() == &K::CompilationUnit).unwrap().id()];
    let mut seen = std::collections::HashSet::new();
    let mut tokens = Vec::new();
    while let Some(id) = pending.pop() {
        assert!(seen.insert(id), "duplicate Boolean RHS source child {id:?}");
        let node = ast.node_view(id).unwrap();
        if node.as_token().is_some() { tokens.push(id); }
        pending.extend(node.children().iter().rev().copied());
    }
    assert_eq!(tokens, ast.token_views().map(|v| v.id()).collect::<Vec<_>>());
    let frontend = task257c4c1_frontend_output(&text, 161);
    assert!(frontend.diagnostics.is_empty());
    assert_eq!(frontend.ast.as_ref().unwrap().snapshot_text_with_trivia(), ast.snapshot_text_with_trivia());
    let repeated = task257c4c1_frontend_output(&text, 161);
    assert_eq!(frontend.cache_keys, repeated.cache_keys);
    assert_eq!(frontend.ast, repeated.ast);
    let token_key = &frontend.cache_keys.tokens;
    assert_eq!(token_key.version.as_ref(), "mizar-frontend/token-stream-cache-key/v6");
    let mut retained_token = token_key.clone();
    retained_token.version = "mizar-frontend/token-stream-cache-key/v4".into();
    assert_eq!(token_key.lexical_hash, retained_token.lexical_hash);
    assert_eq!(token_key.active_lexical_environment, retained_token.active_lexical_environment);
    assert_eq!(token_key.parser_context, retained_token.parser_context);
    assert_eq!(token_key.parser_lexing_plan, retained_token.parser_lexing_plan);
    assert_ne!(token_key.stable_hash(), retained_token.stable_hash());
    let current = frontend.cache_keys.ast.as_ref().unwrap();
    assert_eq!(current.parser_version.version.as_ref(), "mizar-parser/surface-ast-v30");
    let mut retained = current.clone();
    retained.parser_version = mizar_frontend::parsing::ParserCacheKeyVersion::new("mizar-parser/surface-ast-v28");
    assert_eq!(current.token_stream_hash, retained.token_stream_hash);
    assert_eq!(current.parser_inputs_hash, retained.parser_inputs_hash);
    assert_ne!(current.stable_hash(), retained.stable_hash());
    let bytes = ast.canonical_bytes().unwrap();
    let restored = mizar_syntax::SurfaceAst::from_canonical_bytes(&bytes, ast.source_id).unwrap();
    assert_eq!(restored.snapshot_text_with_trivia(), ast.snapshot_text_with_trivia());
}

#[test]
fn algorithm_boolean_operand_recovery_preserves_existing_owners_and_successors() {
    use mizar_frontend::orchestration::{DiagnosticCode, DiagnosticLocation};
    use mizar_syntax::{SurfaceFormulaBinaryOperator as B, SurfaceFormulaConnective as C, SurfaceFormulaPrefixOperator as P, SurfaceNodeKind as K};
    let config = step5c11_config();
    let plan = build_test_plan(&config).unwrap();
    let case = plan.cases.iter().find(|case| case.id.0 == "fail_parser_algorithm_boolean_operands_001").unwrap();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let first = super::super::run_parse_only_case(&config.workspace_root, &config.workspace_root.join("tests"), case, 162);
    assert_eq!(first.status, super::super::ParseOnlyCaseStatus::Passed, "{first:?}");
    assert_eq!(first, super::super::run_parse_only_case(&config.workspace_root, &config.workspace_root.join("tests"), case, 162));
    let output = super::super::formula_statement::step5c8_test_frontend(&text);
    assert_eq!(output.diagnostics.len(), 11, "{:?}", output.diagnostics);
    let ast = output.ast.unwrap();
    for (diagnostic, (marker, word, code, message)) in output.diagnostics.iter().zip([
        ("p & ;", ";", "malformed_formula_expression", "expected formula after connective"),
        ("not ;", ";", "malformed_formula_expression", "expected formula after `not`"),
        ("return (p & q;", ";", "malformed_formula_expression", "expected `)` to close formula delimiter"),
        ("missing_binding := p, ;", ";", "malformed_term_expression", "expected declaration binding after `,`"),
        ("const missing_const;", ";", "malformed_term_expression", "expected `:=` in const declaration binding"),
        ("missing_initializer := ;", ";", "malformed_term_expression", "expected initializer term after `:=`"),
        ("return not thesis;", "thesis", "malformed_formula_expression", "`thesis` is restricted to explicit computation inputs"),
        ("p := p & q as", "as", "malformed_term_expression", "unexpected token in assignment"),
        ("p := p & q,", ",", "malformed_term_expression", "unexpected token in assignment"),
        ("return p & q as", "as", "malformed_term_expression", "unexpected token in return statement"),
        ("return p & q,", ",", "malformed_term_expression", "unexpected token in return statement"),
    ]) {
        let start = text.find(marker).unwrap();
        let anchor = start + marker.rfind(word).unwrap();
        assert_eq!(diagnostic.code, DiagnosticCode::Syntax(code.into()));
        assert_eq!(diagnostic.message.as_ref(), message);
        assert_eq!(diagnostic.location, DiagnosticLocation::SourceRange(range(ast.source_id, anchor, anchor + word.len())));
        if marker.starts_with("return (") {
            assert_eq!(diagnostic.secondary, [mizar_session::SourceAnchor::Range(range(ast.source_id, start + 7, start + 8))]);
            assert_eq!(diagnostic.recovery_note.as_deref(), Some("insert `)` before continuing"));
        } else {
            assert!(diagnostic.secondary.is_empty());
            assert_eq!(diagnostic.recovery_note.as_deref(), Some(if code == "malformed_term_expression" { "repair the term expression before continuing" } else { "repair the formula expression before continuing" }));
        }
    }
    for (marker, rhs, root_kind, recovery_kind) in [
        ("missing_conjunction :=", "p &", K::BinaryFormula(B { connective: C::And, repeated: false }), SyntaxRecoveryKind::MissingFormula),
        ("missing_not :=", "not", K::PrefixFormula(P::Not), SyntaxRecoveryKind::MissingFormula),
        ("return (p & q", "(p & q", K::ParenthesizedFormula, SyntaxRecoveryKind::UnmatchedOpeningDelimiter),
    ] {
        let start = text.find(marker).unwrap();
        let owner = ast.node_views().find(|v| matches!(v.kind(), K::VariableBinding | K::ReturnStatement) && v.range().start == start).unwrap();
        let payload = owner.child_views().find(|v| v.kind() == &K::FormulaExpression).unwrap();
        let rhs_start = start + if marker.contains(":=") { marker.len() + 1 } else { "return ".len() };
        assert_eq!(payload.range(), range(ast.source_id, rhs_start, rhs_start + rhs.len()));
        let root = ast.node_view(payload.children()[0]).unwrap();
        assert_eq!(root.kind(), &root_kind);
        let recovery = ast.node_view(*root.children().last().unwrap()).unwrap();
        assert_eq!(recovery.as_recovery(), Some(recovery_kind));
        let anchor = start + text[start..].find(';').unwrap();
        assert_eq!(recovery.range(), range(ast.source_id, anchor, anchor));
        assert!(recovery.children().is_empty());
    }
    for (marker, skip) in [("p := p & q as", "as expr of boolean"), ("p := p & q,", ", forbidden_assignment"), ("return p & q as", "as expr of boolean"), ("return p & q,", ", forbidden_return")] {
        let start = text.find(marker).unwrap();
        let owner = ast.node_views().find(|v| matches!(v.kind(), K::AssignmentStatement | K::ReturnStatement) && v.range().start == start).unwrap();
        let parts = owner.child_views().collect::<Vec<_>>();
        let payload = parts.iter().find(|v| v.kind() == &K::FormulaExpression).unwrap();
        assert_eq!(&text[payload.range().start..payload.range().end], "p & q");
        let skipped = parts.iter().find(|v| v.as_recovery() == Some(SyntaxRecoveryKind::SkippedToken)).unwrap();
        assert_eq!(&text[skipped.range().start..skipped.range().end], skip);
        assert!(skipped.child_views().all(|v| v.as_token().is_some()));
        assert_eq!(parts.last().unwrap().as_token().unwrap().text.as_ref(), ";");
        assert_eq!(parts.last().unwrap().range().end, owner.range().end);
        assert!(ast.trivia().skipped_token_ranges().iter().any(|v| v.range == skipped.range()));
    }
    let start = text.find("var missing_binding").unwrap();
    let declaration = ast.node_views().find(|v| v.kind() == &K::VariableDeclaration && v.range().start == start).unwrap();
    assert_eq!(declaration.child_views().map(|v| &text[v.range().start..v.range().end]).collect::<Vec<_>>(), ["var", "missing_binding := p", ",", "", ";"]);
    let binding = ast.node_view(declaration.children()[3]).unwrap();
    assert_eq!(binding.kind(), &K::VariableBinding);
    assert_eq!(ast.node_view(binding.children()[0]).unwrap().as_recovery(), Some(SyntaxRecoveryKind::MissingTerm));
    for name in ["missing_const", "missing_initializer"] {
        let start = text.find(&format!("const {name}")).unwrap() + "const ".len();
        let binding = ast.node_views().filter_map(|v| v.as_variable_binding()).find(|v| v.range().start == start).unwrap();
        assert_eq!(ast.node_view(*binding.children().last().unwrap()).unwrap().as_recovery(), Some(SyntaxRecoveryKind::MissingTerm));
    }
    let start = text.find("return not thesis").unwrap();
    let returned = ast.node_views().filter_map(|v| v.as_return_statement()).find(|v| v.range().start == start).unwrap();
    assert!(!returned.is_recovered());
    let payload = ast.node_view(returned.children()[1]).unwrap();
    assert_eq!(&text[payload.range().start..payload.range().end], "not thesis");
    assert!(ast.node_views().any(|v| v.kind() == &K::FormulaConstant(mizar_syntax::SurfaceFormulaConstant::Thesis) && v.range().start == payload.range().start + 4));
    for name in ["after_conjunction", "after_not", "after_group", "after_binding", "after_const", "after_initializer", "after_thesis", "after_assignment_as", "after_assignment_comma", "after_return_as", "after_return_comma"] {
        let spelling = format!("{name} := p");
        let start = text.find(&spelling).unwrap();
        let binding = ast.node_views().filter_map(|v| v.as_variable_binding()).find(|v| v.range().start == start).unwrap();
        assert_eq!(binding.range(), range(ast.source_id, start, start + spelling.len()));
        assert!(!binding.is_recovered());
        let payload = ast.node_view(binding.children()[2]).unwrap();
        assert_eq!(payload.kind(), &K::TermExpression);
        assert_eq!(ast.node_view(payload.children()[0]).unwrap().kind(), &K::TermReference);
    }
    for (kind, count) in [(SyntaxRecoveryKind::MissingFormula, 2), (SyntaxRecoveryKind::UnmatchedOpeningDelimiter, 1), (SyntaxRecoveryKind::MissingTerm, 3), (SyntaxRecoveryKind::SkippedToken, 4)] {
        assert_eq!(ast.node_views().filter(|v| v.as_recovery() == Some(kind)).count(), count);
    }
    assert_eq!(ast.node_views().filter_map(|v| v.as_recovery()).count(), 10);
    assert_eq!(ast.trivia().skipped_token_ranges().len(), 4);
    assert_eq!(ast.node_views().find(|v| v.kind() == &K::AlgorithmStatementList).unwrap().children().len(), 22);
    let final_start = text.find("set after_boolean_failure = sentinel;").unwrap();
    assert!(ast.node_views().filter_map(|v| v.as_set_statement()).any(|v| v.range() == range(ast.source_id, final_start, text.trim_end().len()) && !v.is_recovered()));
    let mut pending = vec![ast.node_views().find(|v| v.kind() == &K::CompilationUnit).unwrap().id()];
    let mut seen = std::collections::HashSet::new();
    let mut tokens = Vec::new();
    while let Some(id) = pending.pop() {
        assert!(seen.insert(id), "duplicate recovered Boolean RHS source child {id:?}");
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
fn algorithm_boolean_rhs_keeps_expr_source_but_consumers_refuse_its_profile() {
    use mizar_resolve::names::{SourceVariableScopeError, SourceVariableScopeResolver};
    use mizar_syntax::SurfaceNodeKind as K;
    let config = step5c11_config();
    let case = step5c14_state_case();
    let base = concat!(
        "definition\n  let p,q be expr of boolean;\n",
        "  algorithm rhs(p,q) -> expr of boolean\n  do\n",
        "    var local := p;\n    local := p;\n    return p;\n",
        "    return p;\n  end;\nend;\n",
    );
    let object = "definition let x be object; algorithm rhs(x) -> object do var local := x; local := x; return x; end; end;";
    for (selected, text) in [
        (None, object.to_owned()),
        (None, base.to_owned()), // The unchanged term RHS already has an unsupported expr profile.
        (Some(0), base.replace("var local := p;", "var local := not p;")),
        (Some(1), base.replace("\n    local := p;", "\n    local := p & q;")),
        (Some(2), base.replacen("return p;", "return (p) & q;", 1)),
    ] {
        let expr_source = text.contains("expr of boolean");
        let frontend = super::super::formula_statement::step5c8_test_frontend(&text);
        assert!(frontend.diagnostics.is_empty(), "{selected:?}: {:?}", frontend.diagnostics);
        let ast = frontend.ast.as_ref().unwrap();
        assert!(ast.node_views().all(|v| !v.is_recovered() && v.as_recovery().is_none()));
        assert!(ast.trivia().skipped_token_ranges().is_empty());
        let body = ast.node_views().find(|v| v.kind() == &K::AlgorithmBody).unwrap();
        assert_eq!(body.children().len(), 3);
        assert_eq!(body.child_views().next().unwrap().as_token().unwrap().text.as_ref(), "do");
        assert_eq!(body.child_views().last().unwrap().as_token().unwrap().text.as_ref(), "end");
        assert_eq!(ast.node_view(body.children()[1]).unwrap().kind(), &K::AlgorithmStatementList);
        let statements = ast.node_view(body.children()[1]).unwrap().child_views().collect::<Vec<_>>();
        let kinds = [&K::VariableDeclaration, &K::AssignmentStatement, &K::ReturnStatement, &K::ReturnStatement];
        assert_eq!(statements.iter().map(|v| v.kind()).collect::<Vec<_>>(), kinds[..if expr_source { 4 } else { 3 }]);
        for statement in &statements {
            assert_eq!(statement.child_views().last().unwrap().as_token().unwrap().text.as_ref(), ";");
        }
        assert_eq!(statements.last().unwrap().range().start, text.rfind(if expr_source { "return p;" } else { "return x;" }).unwrap());
        if expr_source {
            let parameter = ast.node_views().find(|v| v.kind() == &K::DefinitionParameter).unwrap();
            assert_eq!(&text[parameter.range().start..parameter.range().end], "let p,q be expr of boolean;");
            let params = ast.node_views().find(|v| v.kind() == &K::AlgorithmParameters).unwrap();
            assert_eq!(&text[params.range().start..params.range().end], "(p,q)");
            assert_eq!(ast.node_views().filter(|v| v.kind() == &K::TypeExpression && &text[v.range().start..v.range().end] == "expr of boolean").count(), 2);
        }
        for (index, kind) in [K::VariableBinding, K::AssignmentStatement, K::ReturnStatement].into_iter().enumerate() {
            let owner = ast.node_views().find(|v| v.kind() == &kind).unwrap();
            let rhs = owner.child_views().filter(|v| matches!(v.kind(), K::TermExpression | K::FormulaExpression)).collect::<Vec<_>>();
            assert_eq!(rhs.len(), 1);
            let rhs = rhs[0];
            let expected = if selected == Some(index) { ["not p", "p & q", "(p) & q"][index] } else if expr_source { "p" } else { "x" };
            assert_eq!(&text[rhs.range().start..rhs.range().end], expected);
            assert_eq!(rhs.kind(), if selected == Some(index) { &K::FormulaExpression } else { &K::TermExpression });
            assert_eq!(rhs.children().len(), 1);
            if selected != Some(index) { continue; }
            let root = ast.node_view(rhs.children()[0]).unwrap();
            let operands = ast.node_views().filter(|v| v.kind() == &K::TermExpression && v.range().start >= rhs.range().start && v.range().end <= rhs.range().end).collect::<Vec<_>>();
            let names = expected.char_indices().filter(|(_, ch)| matches!(*ch, 'p' | 'q')).collect::<Vec<_>>();
            assert_eq!(operands.len(), names.len());
            for (operand, (offset, name)) in operands.iter().zip(names) {
                assert_eq!((operand.range().start, operand.range().end), (rhs.range().start + offset, rhs.range().start + offset + 1));
                assert_eq!(operand.children().len(), 1);
                let reference = ast.node_view(operand.children()[0]).unwrap();
                assert_eq!(reference.kind(), &K::TermReference);
                assert_eq!(reference.range(), operand.range());
                assert_eq!(reference.children().len(), 1);
                let token = ast.node_view(reference.children()[0]).unwrap();
                assert_eq!(token.range(), reference.range());
                assert_eq!(token.as_token().unwrap().text.as_ref(), name.to_string());
            }
            assert_eq!(root.children().last(), Some(&operands.last().unwrap().id()));
            if index == 0 {
                assert_eq!(root.kind(), &K::PrefixFormula(mizar_syntax::SurfaceFormulaPrefixOperator::Not));
                assert_eq!(root.children().len(), 2);
            } else {
                assert_eq!(root.kind(), &K::BinaryFormula(mizar_syntax::SurfaceFormulaBinaryOperator { connective: mizar_syntax::SurfaceFormulaConnective::And, repeated: false }));
                assert_eq!(root.children().len(), 3);
                if index == 1 { assert_eq!(root.children()[0], operands[0].id()); }
                else {
                    let group = ast.node_view(root.children()[0]).unwrap();
                    assert_eq!(group.kind(), &K::ParenthesizedFormula);
                    assert_eq!(group.child_views().map(|v| &text[v.range().start..v.range().end]).collect::<Vec<_>>(), ["(", "p", ")"]);
                    let inner = ast.node_view(group.children()[1]).unwrap();
                    assert_eq!(inner.kind(), &K::FormulaExpression);
                    assert_eq!(inner.children(), &[operands[0].id()]);
                }
            }
        }
        assert!(!ast.node_views().any(|v| matches!(v.kind(), K::PredicateApplication | K::InlinePredicateApplication | K::MatchCase)));
        let mut pending = vec![ast.node_views().find(|v| v.kind() == &K::CompilationUnit).unwrap().id()];
        let mut seen = std::collections::HashSet::new();
        let mut tokens = Vec::new();
        while let Some(id) = pending.pop() {
            assert!(seen.insert(id), "duplicate RHS source child {id:?}");
            let node = ast.node_view(id).unwrap();
            if node.as_token().is_some() { tokens.push(id); }
            pending.extend(node.children().iter().rev().copied());
        }
        assert_eq!(tokens, ast.token_views().map(|v| v.id()).collect::<Vec<_>>());
        let (source, typed, symbols) = super::super::source_registration_inputs(&config.workspace_root, &case, frontend).unwrap();
        let scope = SourceVariableScopeResolver::resolve_algorithm_occurrences(&source, &symbols);
        let checked = mizar_checker::type_checker::check_source_algorithm_types(&source, &typed, &symbols);
        if expr_source {
            // Both consumers require object formals/results and refuse before inspecting these RHSs.
            assert_eq!(scope.unwrap_err(), SourceVariableScopeError::InvalidShape);
            assert_eq!(checked.unwrap_err(), "algorithms.unsupported_source_types");
        } else {
            // This supported object/Core control validates the harness only.
            scope.unwrap();
            assert_eq!(mizar_core::elaborator::lower_source_algorithms(&checked.unwrap()).unwrap().algorithms().len(), 1);
        }
    }
}

#[test]
fn algorithm_boolean_expr_targets_preserve_identifier_and_symbol_roles() {
    use mizar_syntax::{SurfaceNodeKind as K, SurfaceTokenKind as T};
    for (role, text) in [
        ("term", "definition let boolean be object; algorithm f(boolean) -> object do var value := g[boolean](boolean); return boolean; end; end;"),
        ("symbol", "definition mode RegisteredBoolean: boolean is object; let p be expr of boolean; algorithm f(p) -> expr of boolean do return p; end; end;"),
        ("qualified", "definition mode RegisteredType: T is object; let p be expr of boolean.T; algorithm f(p) -> object do return p; end; end;"),
        ("raw", "definition let p be expr of boolean; algorithm f(p) -> expr of boolean do return p; end; end;"),
    ] {
        let output = super::super::formula_statement::step5c8_test_frontend(text);
        assert!(output.diagnostics.is_empty(), "{role}: {:?}", output.diagnostics);
        let ast = output.ast.unwrap();
        assert!(ast.node_views().all(|v| !v.is_recovered()));
        if role == "term" {
            let arguments = ast.node_views().find(|v| v.kind() == &K::TemplateArguments).unwrap();
            let wrapper = arguments.child_views().find(|v| v.kind() == &K::TemplateArgument).unwrap();
            assert_eq!(wrapper.children().len(), 1);
            let argument = ast.node_view(wrapper.children()[0]).unwrap();
            assert_eq!(argument.kind(), &K::TermExpression);
            assert_eq!(&text[argument.range().start..argument.range().end], "boolean");
            assert!(ast.token_views().filter(|v| v.as_token().unwrap().text.as_ref() == "boolean").all(|v| v.as_token().unwrap().kind == T::Identifier));
            continue;
        }
        let outer = ast.node_views().find(|v| v.kind() == &K::TypeHead && text[v.range().start..v.range().end].starts_with("expr of ")).unwrap();
        let parts = outer.child_views().collect::<Vec<_>>();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0].as_token().unwrap().text.as_ref(), "expr");
        assert_eq!(parts[1].as_token().unwrap().text.as_ref(), "of");
        assert_eq!(parts[2].kind(), &K::TypeExpression);
        let head = ast.node_view(parts[2].children()[0]).unwrap();
        assert_eq!(head.kind(), &K::TypeHead);
        assert_eq!(head.range(), parts[2].range());
        assert_eq!(head.children().len(), 1);
        let leaf = ast.node_view(head.children()[0]).unwrap();
        if role == "raw" {
            assert_eq!(leaf.as_token().unwrap().kind, T::Identifier);
            assert_eq!(leaf.as_token().unwrap().text.as_ref(), "boolean");
            assert_eq!(leaf.range(), head.range());
        } else {
            assert_eq!(leaf.kind(), &K::QualifiedSymbol);
            assert_eq!(&text[leaf.range().start..leaf.range().end], if role == "symbol" { "boolean" } else { "boolean.T" });
            assert!(ast.token_views().any(|v| v.as_token().unwrap().kind == T::UserSymbol && v.range().start >= head.range().start && v.range().end <= head.range().end));
        }
    }
}

#[test]
fn algorithm_boolean_predicate_commas_do_not_introduce_lexical_bindings() {
    use mizar_frontend::lexing::BindingShapeKind as B;
    let text = concat!(
        "definition let x,y,z be object; pred Pair: x,y local_pair means x = y; ",
        "pred Trip: x,y,z local_trip means x = y; algorithm f(x,y,z) -> object do ",
        "var a,b as object; var c := x, d, e := y; const k := x, l := y; ",
        "var pair_result := not x,y local_pair; var trip_result := not x,y,z local_trip; ",
        "var grouped := not (x),y local_pair; return x; end; end;",
    );
    let output = task257c4c1_frontend_output(text, 163);
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    let ast = output.ast.as_ref().unwrap();
    assert!(ast.node_views().all(|v| !v.is_recovered()));
    let bindings = output.tokens.scope_view.frames.iter().flat_map(|frame| &frame.bindings)
        .filter(|binding| matches!(binding.kind, B::Var | B::Const)).collect::<Vec<_>>();
    assert_eq!(bindings.iter().map(|binding| binding.spelling.as_ref()).collect::<Vec<_>>(), ["a", "b", "c", "d", "e", "k", "l", "pair_result", "trip_result", "grouped"]);
    for binding in bindings {
        assert_eq!(&text[binding.introduced_at.start..binding.introduced_at.end], binding.spelling.as_ref());
        assert_eq!(binding.introduced_at.source_id, ast.source_id);
    }
}
