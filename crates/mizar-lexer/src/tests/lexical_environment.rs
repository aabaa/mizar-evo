use super::common::*;

#[test]
fn lexical_identity_newtypes_expose_stable_helpers() {
    let module = ModuleId::new("std.hidden");
    let symbol = SymbolId::new("std.hidden#plus");
    let rank = ExportRank::new(17);
    let summary_fingerprint = LexicalSummaryFingerprint::new(23);
    let environment_fingerprint = crate::LexicalEnvironmentFingerprint::new(29);

    assert_eq!(module.as_str(), "std.hidden");
    assert_eq!(symbol.as_str(), "std.hidden#plus");
    assert_eq!(rank.get(), 17);
    assert_eq!(summary_fingerprint.get(), 23);
    assert_eq!(environment_fingerprint.get(), 29);
}

#[test]
fn lexical_environment_always_contains_reserved_tables() {
    let env = build_lexical_environment(&[], &[]).expect("empty imports should build");

    assert_eq!(env.reserved_word("theorem"), Some("theorem"));
    assert_eq!(env.reserved_symbol(":="), Some(":="));
    assert!(env.user_symbol("+").is_none());
}

#[test]
fn lexical_environment_imports_identifier_punctuation_and_dot_symbols() {
    let env = build_lexical_environment(
        &[resolved_import("std.algebra.ops")],
        &[summary(
            "std.algebra.ops",
            11,
            &[
                exported("succ", "std.algebra.ops#succ", "std.algebra.ops", 0),
                exported("*+", "std.algebra.ops#star_plus", "std.algebra.ops", 1),
                exported("|.", "std.algebra.ops#abs_open", "std.algebra.ops", 2),
                exported("grp.mul", "std.algebra.ops#qualified", "std.algebra.ops", 3),
            ],
        )],
    )
    .expect("environment should build");

    assert_eq!(
        env.user_symbol("succ")
            .expect("identifier-shaped symbol")
            .symbol_id,
        symbol_id("std.algebra.ops#succ")
    );
    assert_eq!(
        env.longest_user_symbol_at("*+x", 0)[0].symbol_id,
        symbol_id("std.algebra.ops#star_plus")
    );
    assert_eq!(
        env.longest_user_symbol_at("|.x.|", 0)[0].symbol_id,
        symbol_id("std.algebra.ops#abs_open")
    );
    assert_eq!(
        env.longest_user_symbol_at("let grp.mul be", 4)[0].symbol_id,
        symbol_id("std.algebra.ops#qualified")
    );
}

#[test]
fn lexical_environment_longest_match_prefers_longest_user_symbol() {
    let env = build_lexical_environment(
        &[resolved_import("std.algebra.ops")],
        &[summary(
            "std.algebra.ops",
            12,
            &[
                exported("+", "std.algebra.ops#plus", "std.algebra.ops", 0),
                exported("+*", "std.algebra.ops#plus_star", "std.algebra.ops", 1),
                exported(
                    "+*+",
                    "std.algebra.ops#plus_star_plus",
                    "std.algebra.ops",
                    2,
                ),
            ],
        )],
    )
    .expect("environment should build");

    assert_eq!(
        env.longest_user_symbol_at("+*+x", 0),
        vec![UserSymbolCandidate {
            spelling: "+*+".to_owned(),
            symbol_id: symbol_id("std.algebra.ops#plus_star_plus"),
            source_module: module_id("std.algebra.ops"),
            imported_module: module_id("std.algebra.ops"),
            import_ordinal: 0,
            export_rank: ExportRank(2),
            kind: UserSymbolKind::Functor,
            arity: UserSymbolArity::exact(2),
            operator: None,
        }]
    );
}

#[test]
fn lexical_environment_distinguishes_equal_length_symbols_by_spelling() {
    let env = build_lexical_environment(
        &[resolved_import("std.first"), resolved_import("std.second")],
        &[
            summary(
                "std.first",
                13,
                &[exported("++", "std.first#plusplus", "std.first", 0)],
            ),
            summary(
                "std.second",
                14,
                &[exported("+*", "std.second#plus_star", "std.second", 0)],
            ),
        ],
    )
    .expect("environment should build");

    let candidates = env.longest_user_symbol_at("+*++", 0);
    assert_eq!(
        candidates
            .iter()
            .map(|candidate| candidate.symbol_id.clone())
            .collect::<Vec<_>>(),
        vec![symbol_id("std.second#plus_star")]
    );

    let same_start = env.longest_user_symbol_at("++", 0);
    assert_eq!(
        same_start
            .iter()
            .map(|candidate| candidate.symbol_id.clone())
            .collect::<Vec<_>>(),
        vec![symbol_id("std.first#plusplus")]
    );
}

#[test]
fn lexical_environment_trie_lookup_handles_many_symbols_and_overlaps() {
    let mut symbols = vec![
        exported("+", "bulk#plus", "bulk", 0),
        exported("+*", "bulk#plus_star", "bulk", 1),
        exported("+*+", "bulk#plus_star_plus", "bulk", 2),
        exported("alpha", "bulk#alpha", "bulk", 3),
        exported("alphabet", "bulk#alphabet", "bulk", 4),
    ];
    for index in 0..2_048 {
        let spelling = format!("bulk_symbol_{index:04}");
        let symbol = format!("bulk#{index:04}");
        symbols.push(exported(&spelling, &symbol, "bulk", index + 5));
    }
    let env =
        build_lexical_environment(&[resolved_import("bulk")], &[summary("bulk", 91, &symbols)])
            .expect("large imported lexicon should build");

    assert_eq!(env.user_symbols.spelling_count(), 2_053);
    assert!(env.user_symbols.trie_node_count() > env.user_symbols.spelling_count());
    assert_eq!(
        env.longest_user_symbol_at("+*+x", 0)[0].symbol_id,
        symbol_id("bulk#plus_star_plus")
    );
    assert_eq!(
        env.longest_user_symbol_at("alphabet_soup", 0)[0].symbol_id,
        symbol_id("bulk#alphabet")
    );
    assert_eq!(
        env.longest_user_symbol_at("bulk_symbol_2047", 0)[0].symbol_id,
        symbol_id("bulk#2047")
    );
    assert!(env.longest_user_symbol_at("not_imported", 0).is_empty());
}

#[test]
fn lexical_environment_returns_empty_lookup_for_invalid_offsets() {
    let env = build_lexical_environment(
        &[resolved_import("std.unicode_fixture")],
        &[summary(
            "std.unicode_fixture",
            15,
            &[exported(
                "+",
                "std.unicode_fixture#plus",
                "std.unicode_fixture",
                0,
            )],
        )],
    )
    .expect("environment should build");

    assert!(env.longest_user_symbol_at("+", 4).is_empty());
    assert!(env.longest_user_symbol_at("aé+", 2).is_empty());
}

#[test]
fn lexical_environment_rejects_equal_spelling_across_imports() {
    let error = build_lexical_environment(
        &[resolved_import("std.first"), resolved_import("std.second")],
        &[
            summary(
                "std.first",
                21,
                &[exported("+", "std.first#plus", "std.first", 0)],
            ),
            summary(
                "std.second",
                22,
                &[exported("+", "std.second#plus", "std.second", 0)],
            ),
        ],
    )
    .expect_err("equal imported user-symbol spelling should be a conflict");

    assert!(matches!(
        error,
        LexicalEnvironmentError::UserSymbolImportConflict { .. }
    ));
}

#[test]
fn lexical_environment_import_conflict_reports_imported_modules() {
    let error = build_lexical_environment(
        &[resolved_import("facade.a"), resolved_import("facade.b")],
        &[
            summary(
                "facade.a",
                24,
                &[exported("+", "std.origin#plus", "std.origin", 0)],
            ),
            summary(
                "facade.b",
                25,
                &[exported("+", "std.origin#plus", "std.origin", 0)],
            ),
        ],
    )
    .expect_err("conflict diagnostics should mention imported modules");

    assert_eq!(
        error,
        LexicalEnvironmentError::UserSymbolImportConflict {
            spelling: "+".to_owned(),
            earlier_import: module_id("facade.a"),
            later_import: module_id("facade.b"),
        }
    );
}

#[test]
fn lexical_environment_keeps_same_import_candidates_for_same_spelling() {
    let env = build_lexical_environment(
        &[resolved_import("std.overloaded")],
        &[summary(
            "std.overloaded",
            23,
            &[
                exported("+", "std.overloaded#plus_nat", "std.overloaded", 0),
                exported("+", "std.overloaded#plus_real", "std.overloaded", 1),
            ],
        )],
    )
    .expect("same imported module may export overloaded notation candidates");

    assert_eq!(
        env.longest_user_symbol_at("+ x", 0)
            .iter()
            .map(|candidate| candidate.symbol_id.clone())
            .collect::<Vec<_>>(),
        vec![
            symbol_id("std.overloaded#plus_nat"),
            symbol_id("std.overloaded#plus_real")
        ]
    );
}

#[test]
fn lexical_environment_preserves_symbol_kind_and_arity_metadata() {
    let env = build_lexical_environment(
        &[resolved_import("std.overloaded")],
        &[summary(
            "std.overloaded",
            26,
            &[
                exported_with_metadata(
                    "op",
                    "std.overloaded#op_functor",
                    "std.overloaded",
                    0,
                    UserSymbolKind::Functor,
                    UserSymbolArity::exact(2),
                ),
                exported_with_metadata(
                    "op",
                    "std.overloaded#op_predicate",
                    "std.overloaded",
                    1,
                    UserSymbolKind::Predicate,
                    UserSymbolArity::range(1, 2),
                ),
                exported_with_metadata(
                    "Vector",
                    "std.overloaded#Vector",
                    "std.overloaded",
                    2,
                    UserSymbolKind::Mode,
                    UserSymbolArity::at_least(1),
                ),
            ],
        )],
    )
    .expect("same spelling overloads with metadata should build");

    let candidates = env.longest_user_symbol_at("op x", 0);
    assert_eq!(
        candidates
            .iter()
            .map(|candidate| (candidate.symbol_id.clone(), candidate.kind, candidate.arity))
            .collect::<Vec<_>>(),
        vec![
            (
                symbol_id("std.overloaded#op_functor"),
                UserSymbolKind::Functor,
                UserSymbolArity::exact(2),
            ),
            (
                symbol_id("std.overloaded#op_predicate"),
                UserSymbolKind::Predicate,
                UserSymbolArity::range(1, 2),
            ),
        ]
    );
    assert_eq!(
        env.user_symbol("Vector")
            .map(|candidate| (candidate.kind, candidate.arity)),
        Some((UserSymbolKind::Mode, UserSymbolArity::at_least(1)))
    );
}

#[test]
fn local_declaration_prepass_activates_symbols_only_after_declaring_item() {
    let source = "a + b;\nfunc PlusDef: a + b -> set equals a;\na + b;";
    let raw = scan_raw(source).expect("source should raw scan");
    let locals = collect_local_lexical_declarations(&raw, module_id("current"));
    let env = build_lexical_environment(&[], &[]).expect("environment should build");

    let first_use = nth_index(source, "+", 0);
    let declaration_header = nth_index(source, "+", 1);
    let later_use = nth_index(source, "+", 2);
    assert!(
        env.longest_user_symbol_at_position(source, first_use, first_use, &locals)
            .is_empty(),
        "local declaration must not be visible before its item"
    );
    assert!(
        env.longest_user_symbol_at_position(
            source,
            declaration_header,
            declaration_header,
            &locals
        )
        .is_empty(),
        "declaration header/definiens must not see the introduced spelling"
    );
    assert_eq!(
        env.longest_user_symbol_at_position(source, later_use, later_use, &locals)
            .iter()
            .map(|candidate| (candidate.spelling.as_str(), candidate.kind, candidate.arity))
            .collect::<Vec<_>>(),
        vec![("+", UserSymbolKind::Functor, UserSymbolArity::exact(2))]
    );

    let skeleton = build_scope_skeleton(&raw);
    let stream = disambiguate_with_local_declarations(
        &raw,
        &env,
        &locals,
        &ParserLexContext::general(),
        &skeleton,
    );
    assert_eq!(
        stream
            .tokens
            .iter()
            .filter(|token| token.lexeme == "+")
            .map(|token| token.kind)
            .collect::<Vec<_>>(),
        vec![
            TokenKind::ErrorRecovery,
            TokenKind::ErrorRecovery,
            TokenKind::UserSymbol
        ]
    );
}

#[test]
fn local_declarations_preserve_same_spelling_import_candidates() {
    let env = build_lexical_environment(
        &[resolved_import("std.imported")],
        &[summary(
            "std.imported",
            81,
            &[exported("+", "std.imported#plus", "std.imported", 0)],
        )],
    )
    .expect("imported plus should build");
    let source = "func LocalPlus: a + b -> set equals a;\na + b;";
    let raw = scan_raw(source).expect("source should raw scan");
    let locals = collect_local_lexical_declarations(&raw, module_id("current"));
    let later_use = nth_index(source, "+", 1);

    let candidates = env.longest_user_symbol_at_position(source, later_use, later_use, &locals);
    assert_eq!(
        candidates
            .iter()
            .map(|candidate| candidate.symbol_id.as_str().to_owned())
            .collect::<Vec<_>>(),
        vec![
            "std.imported#plus".to_owned(),
            "current#local:0:+".to_owned()
        ]
    );
}

#[test]
fn local_declaration_prepass_keeps_visibility_out_of_lexing_and_operators_separate() {
    let source = concat!(
        "private func PrivatePlus: a + b -> set equals a;\n",
        "public pred LessDef: a < b means a = b;\n",
        "infix_operator(\"++\", left, 70);\n",
        "a + b; a < b; a ++ b;"
    );
    let raw = scan_raw(source).expect("source should raw scan");
    let locals = collect_local_lexical_declarations(&raw, module_id("current"));
    let env = build_lexical_environment(&[], &[]).expect("environment should build");

    let later_plus = nth_index(source, "+", 3);
    let later_less = nth_index(source, "<", 1);
    let later_operator = nth_index(source, "++", 1);
    assert_eq!(
        env.longest_user_symbol_at_position(source, later_plus, later_plus, &locals)[0].spelling,
        "+"
    );
    assert_eq!(
        env.longest_user_symbol_at_position(source, later_less, later_less, &locals)[0].spelling,
        "<"
    );
    assert_eq!(
        env.longest_user_symbol_at_position(source, later_operator, later_operator, &locals)[0]
            .spelling,
        "+",
        "the existing `+` declaration may match; `++` itself is not introduced"
    );
    assert!(
        locals.user_symbols("++", later_operator).is_empty(),
        "operator declarations attach metadata later; they do not introduce user-symbol entries"
    );
    assert_eq!(locals.operator_declarations.len(), 1);
    assert_eq!(locals.operator_declarations[0].spelling, "++");
}

#[test]
fn local_declaration_prepass_collects_constructor_declarations_without_selectors() {
    let source = concat!(
        "mode ModeLabel: LocalMode is set;\n",
        "attr AttrLabel: x is local_attr means x = x;\n",
        "struct LocalStruct where ",
        "field local_field -> set; ",
        "property local_prop -> set; ",
        "end;\n",
        "LocalMode local_attr LocalStruct local_field local_prop;"
    );
    let raw = scan_raw(source).expect("source should raw scan");
    let locals = collect_local_lexical_declarations(&raw, module_id("current"));
    let env = build_lexical_environment(&[], &[]).expect("environment should build");

    let mode_decl = nth_index(source, "LocalMode", 0);
    let mode_use = nth_index(source, "LocalMode", 1);
    assert!(
        env.longest_user_symbol_at_position(source, mode_decl, mode_decl, &locals)
            .is_empty()
    );
    assert_eq!(
        env.longest_user_symbol_at_position(source, mode_use, mode_use, &locals)
            .iter()
            .map(|candidate| (candidate.spelling.as_str(), candidate.kind, candidate.arity))
            .collect::<Vec<_>>(),
        vec![("LocalMode", UserSymbolKind::Mode, UserSymbolArity::exact(0))]
    );

    let attr_use = nth_index(source, "local_attr", 1);
    assert_eq!(
        env.longest_user_symbol_at_position(source, attr_use, attr_use, &locals)
            .iter()
            .map(|candidate| (candidate.spelling.as_str(), candidate.kind, candidate.arity))
            .collect::<Vec<_>>(),
        vec![(
            "local_attr",
            UserSymbolKind::Attribute,
            UserSymbolArity::exact(1)
        )]
    );

    let struct_decl = nth_index(source, "LocalStruct", 0);
    let struct_use = nth_index(source, "LocalStruct", 1);
    assert!(
        env.longest_user_symbol_at_position(source, struct_decl, struct_decl, &locals)
            .is_empty(),
        "struct declaration name is active only after its closing end;"
    );
    assert_eq!(
        env.longest_user_symbol_at_position(source, struct_use, struct_use, &locals)
            .iter()
            .map(|candidate| (candidate.spelling.as_str(), candidate.kind, candidate.arity))
            .collect::<Vec<_>>(),
        vec![(
            "LocalStruct",
            UserSymbolKind::Structure,
            UserSymbolArity::exact(0)
        )]
    );

    let field_use = nth_index(source, "local_field", 1);
    let property_use = nth_index(source, "local_prop", 1);
    assert!(
        env.longest_user_symbol_at_position(source, field_use, field_use, &locals)
            .is_empty()
    );
    assert!(
        env.longest_user_symbol_at_position(source, property_use, property_use, &locals)
            .is_empty()
    );
}

#[test]
fn local_declaration_prepass_collects_readable_constructor_names_as_whole_spellings() {
    let source = concat!(
        "mode OneSortedDef: 1-sorted is set;\n",
        "attr CStarDef: x is C-star-algebraic means x = x;\n",
        "struct R-module where field carrier -> set; end;\n",
        "1-sorted C-star-algebraic R-module;"
    );
    let raw = scan_raw(source).expect("source should raw scan");
    let locals = collect_local_lexical_declarations(&raw, module_id("current"));
    let env = build_lexical_environment(&[], &[]).expect("environment should build");

    for (spelling, kind) in [
        ("1-sorted", UserSymbolKind::Mode),
        ("C-star-algebraic", UserSymbolKind::Attribute),
        ("R-module", UserSymbolKind::Structure),
    ] {
        let use_position = nth_index(source, spelling, 1);
        assert_eq!(
            env.longest_user_symbol_at_position(source, use_position, use_position, &locals)
                .iter()
                .map(|candidate| (candidate.spelling.as_str(), candidate.kind))
                .collect::<Vec<_>>(),
            vec![(spelling, kind)]
        );
    }
    assert!(
        locals.user_symbols("-", source.len()).is_empty(),
        "hyphens inside constructor names must not be introduced as notation symbols"
    );
}

#[test]
fn local_attribute_prepass_splits_parameter_prefix_from_constructor_name() {
    let source = concat!(
        "definition\n",
        "let n be Nat, row, col be Nat, implicit;\n",
        "let f be Function of REAL, REAL;\n",
        "let fnc be Function[REAL, REAL], X be set;\n",
        "let h be Function of X, Y, g, k be set;\n",
        "let ref_bound be T by A, B;\n",
        "attr NDimDef: V is n-dimensional means V = V;\n",
        "attr RowColSizeDef: A is (row,col)-size means A = A;\n",
        "attr TwoRankedDef: W is 2-ranked means W = W;\n",
        "attr ImplicitShapeDef: U is implicit-shaped means U = U;\n",
        "attr RealLinearDef: X is REAL-linear means X = X;\n",
        "attr XGradedDef: Y is X-graded means Y = Y;\n",
        "attr RealBracketedDef: Z is REAL-bracketed means Z = Z;\n",
        "attr YStableDef: P is Y-stable means P = P;\n",
        "attr GMetricDef: P is g-metric means P = P;\n",
        "attr KValuedDef: P is k-valued means P = P;\n",
        "attr BReferenceDef: P is B-reference means P = P;\n",
        "end;\n",
        "dimensional size ranked shaped graded metric valued REAL-linear REAL-bracketed ",
        "Y-stable B-reference linear bracketed stable reference ",
        "n-dimensional (row,col)-size 2-ranked implicit-shaped X-graded g-metric k-valued;"
    );
    let raw = scan_raw(source).expect("source should raw scan");
    let locals = collect_local_lexical_declarations(&raw, module_id("current"));
    let env = build_lexical_environment(&[], &[]).expect("environment should build");

    for spelling in [
        "dimensional",
        "size",
        "ranked",
        "shaped",
        "graded",
        "metric",
        "valued",
    ] {
        let use_position = nth_index(source, spelling, 1);
        assert_eq!(
            env.longest_user_symbol_at_position(source, use_position, use_position, &locals)
                .iter()
                .map(|candidate| (candidate.spelling.as_str(), candidate.kind, candidate.arity))
                .collect::<Vec<_>>(),
            vec![(
                spelling,
                UserSymbolKind::Attribute,
                UserSymbolArity::exact(1)
            )]
        );
    }

    assert!(
        locals
            .user_symbols("n-dimensional", source.len())
            .is_empty(),
        "the param-prefix spelling must not be registered as the attribute name"
    );
    assert!(
        locals
            .user_symbols("(row,col)-size", source.len())
            .is_empty(),
        "the parenthesized param-prefix spelling must not be registered as a user symbol"
    );
    assert!(
        locals.user_symbols("2-ranked", source.len()).is_empty(),
        "the numeral param-prefix spelling must not be registered as the attribute name"
    );
    assert!(
        locals
            .user_symbols("implicit-shaped", source.len())
            .is_empty(),
        "the implicit let param-prefix spelling must not be registered as the attribute name"
    );
    assert!(
        locals.user_symbols("X-graded", source.len()).is_empty(),
        "the uppercase explicit let segment after a parameterized type should split as a param-prefix"
    );
    assert!(
        locals.user_symbols("g-metric", source.len()).is_empty(),
        "the explicit let segment after an of-type argument list should split as a param-prefix"
    );
    assert!(
        locals.user_symbols("k-valued", source.len()).is_empty(),
        "comma-separated names in the explicit segment after an of-type argument list should split"
    );
    let real_linear_use = nth_index(source, "REAL-linear", 1);
    assert_eq!(
        env.longest_user_symbol_at_position(source, real_linear_use, real_linear_use, &locals)
            .iter()
            .map(|candidate| (candidate.spelling.as_str(), candidate.kind, candidate.arity))
            .collect::<Vec<_>>(),
        vec![(
            "REAL-linear",
            UserSymbolKind::Attribute,
            UserSymbolArity::exact(1)
        )],
        "commas inside a parameterized type expression must not introduce fake let parameters"
    );
    assert!(
        locals.user_symbols("linear", source.len()).is_empty(),
        "a type-expression comma must not cause REAL-linear to split as an attribute param-prefix"
    );
    let real_bracketed_use = nth_index(source, "REAL-bracketed", 1);
    assert_eq!(
        env.longest_user_symbol_at_position(
            source,
            real_bracketed_use,
            real_bracketed_use,
            &locals
        )
        .iter()
        .map(|candidate| (candidate.spelling.as_str(), candidate.kind, candidate.arity))
        .collect::<Vec<_>>(),
        vec![(
            "REAL-bracketed",
            UserSymbolKind::Attribute,
            UserSymbolArity::exact(1)
        )],
        "commas inside bracketed type arguments must not introduce fake let parameters"
    );
    assert!(
        locals.user_symbols("bracketed", source.len()).is_empty(),
        "a bracketed type-argument comma must not cause REAL-bracketed to split"
    );
    let y_stable_use = nth_index(source, "Y-stable", 1);
    assert_eq!(
        env.longest_user_symbol_at_position(source, y_stable_use, y_stable_use, &locals)
            .iter()
            .map(|candidate| (candidate.spelling.as_str(), candidate.kind, candidate.arity))
            .collect::<Vec<_>>(),
        vec![(
            "Y-stable",
            UserSymbolKind::Attribute,
            UserSymbolArity::exact(1)
        )],
        "earlier of-type arguments must not be collected as fake let parameters"
    );
    assert!(
        locals.user_symbols("stable", source.len()).is_empty(),
        "an of-type argument comma before a later let segment must not split Y-stable"
    );
    let b_reference_use = nth_index(source, "B-reference", 1);
    assert_eq!(
        env.longest_user_symbol_at_position(source, b_reference_use, b_reference_use, &locals)
            .iter()
            .map(|candidate| (candidate.spelling.as_str(), candidate.kind, candidate.arity))
            .collect::<Vec<_>>(),
        vec![(
            "B-reference",
            UserSymbolKind::Attribute,
            UserSymbolArity::exact(1)
        )],
        "commas in a trailing by-reference list must not introduce fake let parameters"
    );
    assert!(
        locals.user_symbols("reference", source.len()).is_empty(),
        "a trailing by-reference must not cause B-reference to split"
    );
}

#[test]
fn local_declaration_prepass_rejects_symbolic_constructor_declaration_names() {
    let source = concat!(
        "mode BadMode: + is set;\n",
        "attr BadAttr: x is * means x = x;\n",
        "struct [: where end;\n",
        "+ * [:;"
    );
    let raw = scan_raw(source).expect("source should raw scan");
    let locals = collect_local_lexical_declarations(&raw, module_id("current"));
    let env = build_lexical_environment(&[], &[]).expect("environment should build");

    for spelling in ["+", "*", "[:"] {
        let use_position = nth_index(source, spelling, 1);
        assert!(
            env.longest_user_symbol_at_position(source, use_position, use_position, &locals)
                .is_empty(),
            "{spelling:?} must not become a local constructor lexical entry"
        );
    }
}

#[test]
fn local_declaration_prepass_handles_identifier_shaped_prefix_and_call_symbols() {
    let source = concat!(
        "func CallDef: f(x, y) -> set equals x;\n",
        "pred PrefixDef: R x means x = x;\n",
        "pred RelDef: x R y means x = y;\n",
        "f(a, b); R a; x R y;"
    );
    let raw = scan_raw(source).expect("source should raw scan");
    let locals = collect_local_lexical_declarations(&raw, module_id("current"));
    let env = build_lexical_environment(&[], &[]).expect("environment should build");

    let call_use = nth_index(source, "f", 3);
    assert_eq!(
        env.longest_user_symbol_at_position(source, call_use, call_use, &locals)
            .iter()
            .map(|candidate| (candidate.spelling.as_str(), candidate.kind))
            .collect::<Vec<_>>(),
        vec![("f", UserSymbolKind::Functor)]
    );
    assert!(
        locals.user_symbols("x", source.len()).is_empty(),
        "call parameters must not be registered as the functor spelling"
    );

    let prefix_use = nth_index(source, "R", 1);
    assert_eq!(
        env.longest_user_symbol_at_position(source, prefix_use, prefix_use, &locals)
            .iter()
            .map(|candidate| (candidate.spelling.as_str(), candidate.kind))
            .collect::<Vec<_>>(),
        vec![("R", UserSymbolKind::Predicate)]
    );
    let infix_use = nth_index(source, "R", 3);
    assert_eq!(
        env.longest_user_symbol_at_position(source, infix_use, infix_use, &locals)
            .iter()
            .map(|candidate| (candidate.spelling.as_str(), candidate.kind))
            .collect::<Vec<_>>(),
        vec![
            ("R", UserSymbolKind::Predicate),
            ("R", UserSymbolKind::Predicate)
        ]
    );
}

#[test]
fn local_declaration_prepass_handles_identifier_shaped_postfix_symbols() {
    let source = "pred PostfixDef: x R means x = x;\nx R;";
    let raw = scan_raw(source).expect("source should raw scan");
    let locals = collect_local_lexical_declarations(&raw, module_id("current"));
    let env = build_lexical_environment(&[], &[]).expect("environment should build");

    let postfix_use = nth_index(source, "R", 1);
    assert_eq!(
        env.longest_user_symbol_at_position(source, postfix_use, postfix_use, &locals)
            .iter()
            .map(|candidate| (candidate.spelling.as_str(), candidate.kind))
            .collect::<Vec<_>>(),
        vec![("R", UserSymbolKind::Predicate)]
    );
    assert!(locals.user_symbols("x", source.len()).is_empty());
}

#[test]
fn local_declaration_prepass_collects_hyphenated_functor_notation_as_one_symbol() {
    let source = concat!(
        "func HyphenDef: x foo-bar y -> set equals x;\n",
        "pred HyphenPred: x rel-to y means x = y;\n",
        "x foo-bar y; x rel-to y;"
    );
    let raw = scan_raw(source).expect("source should raw scan");
    let locals = collect_local_lexical_declarations(&raw, module_id("current"));
    let env = build_lexical_environment(&[], &[]).expect("environment should build");

    for (spelling, kind) in [
        ("foo-bar", UserSymbolKind::Functor),
        ("rel-to", UserSymbolKind::Predicate),
    ] {
        let use_position = nth_index(source, spelling, 1);
        assert_eq!(
            env.longest_user_symbol_at_position(source, use_position, use_position, &locals)
                .iter()
                .map(|candidate| (candidate.spelling.as_str(), candidate.kind))
                .collect::<Vec<_>>(),
            vec![(spelling, kind)]
        );
    }
    assert!(
        locals.user_symbols("-", source.len()).is_empty(),
        "hyphenated notation must not be reduced to the hyphen token"
    );
}

#[test]
fn local_declaration_prepass_collects_circumfix_symbol_parts() {
    let source = concat!(
        "func AbsDef: |. x .| -> set equals x;\n",
        "func PairDef: [: X, Y :] -> set equals X;\n",
        "|. y .|; [: A, B :];"
    );
    let raw = scan_raw(source).expect("source should raw scan");
    let locals = collect_local_lexical_declarations(&raw, module_id("current"));
    let env = build_lexical_environment(&[], &[]).expect("environment should build");

    for (spelling, ordinal) in [("|.", 1), (".|", 1), ("[:", 1), (":]", 1)] {
        let position = nth_index(source, spelling, ordinal);
        assert_eq!(
            env.longest_user_symbol_at_position(source, position, position, &locals)[0].spelling,
            spelling,
            "{spelling:?} should be active after the circumfix declaration"
        );
    }
}

#[test]
fn local_declaration_prepass_collects_parameterized_alias_heads() {
    let source = "synonym FinSeq of G for FinSequence of G;\nFinSeq of G;";
    let raw = scan_raw(source).expect("source should raw scan");
    let locals = collect_local_lexical_declarations(&raw, module_id("current"));
    let env = build_lexical_environment(&[], &[]).expect("environment should build");

    let alias_use = nth_index(source, "FinSeq", 2);
    assert_eq!(
        env.longest_user_symbol_at_position(source, alias_use, alias_use, &locals)[0].spelling,
        "FinSeq"
    );
    assert!(
        locals.user_symbols("of", source.len()).is_empty(),
        "parameter separator words must not be registered as alias spellings"
    );
}

#[test]
fn local_alias_prepass_classifies_identifier_alias_from_operator_like_original() {
    let source = "synonym Inv x for x\";\nInv y;";
    let raw = scan_raw(source).expect("source should raw scan");
    let locals = collect_local_lexical_declarations(&raw, module_id("current"));
    let env = build_lexical_environment(&[], &[]).expect("environment should build");

    let alias_use = nth_index(source, "Inv", 1);
    assert_eq!(
        env.longest_user_symbol_at_position(source, alias_use, alias_use, &locals)
            .iter()
            .map(|candidate| (candidate.spelling.as_str(), candidate.kind, candidate.arity))
            .collect::<Vec<_>>(),
        vec![("Inv", UserSymbolKind::Functor, UserSymbolArity::exact(1))]
    );
}

#[test]
fn local_alias_prepass_keeps_constructor_alias_hyphens_inside_the_name() {
    let source = "synonym Fin-Seq for FinSequence;\nFin-Seq;";
    let raw = scan_raw(source).expect("source should raw scan");
    let locals = collect_local_lexical_declarations(&raw, module_id("current"));
    let env = build_lexical_environment(&[], &[]).expect("environment should build");

    let alias_use = nth_index(source, "Fin-Seq", 1);
    assert_eq!(
        env.longest_user_symbol_at_position(source, alias_use, alias_use, &locals)
            .iter()
            .map(|candidate| (candidate.spelling.as_str(), candidate.kind))
            .collect::<Vec<_>>(),
        vec![("Fin-Seq", UserSymbolKind::Mode)]
    );
    assert!(
        locals.user_symbols("-", source.len()).is_empty(),
        "constructor alias hyphen must not be introduced as a notation symbol"
    );
}

#[test]
fn local_alias_prepass_ignores_reserved_word_alias_heads_without_notation_evidence() {
    let source = "synonym theorem for FinSequence;\ntheorem;";
    let raw = scan_raw(source).expect("source should raw scan");
    let locals = collect_local_lexical_declarations(&raw, module_id("current"));

    assert!(
        locals.user_symbols("theorem", source.len()).is_empty(),
        "word-only aliases without operator-like evidence still need constructor-name spelling"
    );
}

#[test]
fn local_declaration_prepass_delays_activation_through_correctness_trail() {
    let source = concat!(
        "func FDef: u -> set means x = x;\n",
        "existence proof now u; end; u; end;\n",
        "u;"
    );
    let raw = scan_raw(source).expect("source should raw scan");
    let locals = collect_local_lexical_declarations(&raw, module_id("current"));
    let env = build_lexical_environment(&[], &[]).expect("environment should build");

    let trail_use = nth_index(source, "u", 2);
    let after_nested_end_use = nth_index(source, "u", 3);
    let later_use = nth_index(source, "u", 4);
    assert!(
        env.longest_user_symbol_at_position(source, trail_use, trail_use, &locals)
            .is_empty(),
        "functor spelling is inactive inside declaration-owned correctness trail"
    );
    assert!(
        env.longest_user_symbol_at_position(
            source,
            after_nested_end_use,
            after_nested_end_use,
            &locals
        )
        .is_empty(),
        "nested proof blocks must not end the declaration-owned correctness trail early"
    );
    assert_eq!(
        env.longest_user_symbol_at_position(source, later_use, later_use, &locals)[0].spelling,
        "u"
    );
    assert_eq!(
        locals.user_symbols("u", later_use)[0].symbol_id.as_str(),
        "current#local:0:u"
    );
}

#[test]
fn local_declaration_prepass_ignores_deffunc_defpred_algorithm_and_redefinitions() {
    let source = concat!(
        "deffunc F(x) = x;\n",
        "defpred P[x] means x = x;\n",
        "redefine func OldPlus: x + y -> set equals x;\n",
        "algorithm AlgoName(acc) -> set do acc := acc + y; end;\n",
        "x + y;"
    );
    let raw = scan_raw(source).expect("source should raw scan");
    let locals = collect_local_lexical_declarations(&raw, module_id("current"));
    let env = build_lexical_environment(&[], &[]).expect("environment should build");
    let later_plus = nth_index(source, "+", 2);

    assert!(locals.user_symbols.is_empty());
    for spelling in ["AlgoName", "acc", "+", "y"] {
        assert!(
            locals.user_symbols(spelling, source.len()).is_empty(),
            "algorithm spelling {spelling:?} must remain identifier/body text, not a user symbol"
        );
    }
    assert!(
        env.longest_user_symbol_at_position(source, later_plus, later_plus, &locals)
            .is_empty()
    );
}

#[test]
fn local_alias_declarations_activate_only_the_alias_pattern() {
    let source = "synonym b > a for a < b;\nb > a; a < b;";
    let raw = scan_raw(source).expect("source should raw scan");
    let locals = collect_local_lexical_declarations(&raw, module_id("current"));
    let env = build_lexical_environment(&[], &[]).expect("environment should build");

    let alias_use = nth_index(source, ">", 1);
    let original_use = nth_index(source, "<", 1);
    assert_eq!(
        env.longest_user_symbol_at_position(source, alias_use, alias_use, &locals)[0].spelling,
        ">"
    );
    assert!(
        env.longest_user_symbol_at_position(source, original_use, original_use, &locals)
            .is_empty(),
        "the original pattern after `for` is not a newly introduced alias"
    );
}

#[test]
fn local_antonym_declarations_activate_only_the_alias_pattern() {
    let source = "antonym a >= b for a < b;\na >= b; a < b;";
    let raw = scan_raw(source).expect("source should raw scan");
    let locals = collect_local_lexical_declarations(&raw, module_id("current"));
    let env = build_lexical_environment(&[], &[]).expect("environment should build");

    let alias_use = nth_index(source, ">=", 1);
    let original_use = nth_index(source, "<", 1);
    assert_eq!(
        env.longest_user_symbol_at_position(source, alias_use, alias_use, &locals)[0].spelling,
        ">="
    );
    assert!(
        env.longest_user_symbol_at_position(source, original_use, original_use, &locals)
            .is_empty()
    );
}

#[test]
fn local_operator_declarations_record_metadata_and_activation_ranges() {
    let source = concat!(
        "prefix_operator(\"~\", 70);\n",
        "postfix_operator(\"!\", 95);\n",
        "infix_operator(\"**\", right, 80);\n",
        "infix_operator(\"%%\", none, 40);\n"
    );
    let raw = scan_raw(source).expect("source should raw scan");
    let locals = collect_local_lexical_declarations(&raw, module_id("current"));

    assert_eq!(locals.operator_declarations.len(), 4);
    assert_eq!(
        locals
            .operator_declarations
            .iter()
            .map(|declaration| (
                declaration.spelling.as_str(),
                declaration.operator,
                declaration.activation_start
            ))
            .collect::<Vec<_>>(),
        vec![
            (
                "~",
                Some(ExportedOperatorMetadata {
                    fixity: ExportedOperatorFixity::Prefix,
                    precedence: 70,
                }),
                nth_index(source, ";", 0) + 1,
            ),
            (
                "!",
                Some(ExportedOperatorMetadata {
                    fixity: ExportedOperatorFixity::Postfix,
                    precedence: 95,
                }),
                nth_index(source, ";", 1) + 1,
            ),
            (
                "**",
                Some(ExportedOperatorMetadata {
                    fixity: ExportedOperatorFixity::Infix(ExportedOperatorAssociativity::Right),
                    precedence: 80,
                }),
                nth_index(source, ";", 2) + 1,
            ),
            (
                "%%",
                Some(ExportedOperatorMetadata {
                    fixity: ExportedOperatorFixity::Infix(
                        ExportedOperatorAssociativity::NonAssociative
                    ),
                    precedence: 40,
                }),
                nth_index(source, ";", 3) + 1,
            ),
        ]
    );
    for spelling in ["~", "!", "**", "%%"] {
        assert!(
            locals.user_symbols(spelling, source.len()).is_empty(),
            "operator metadata for {spelling:?} must not introduce a user symbol"
        );
    }
}

#[test]
fn imported_operator_metadata_is_active_from_start() {
    let env = build_lexical_environment(
        &[resolved_import("ops")],
        &[summary(
            "ops",
            41,
            &[exported_with_operator_metadata(
                "+",
                "ops#plus",
                "ops",
                0,
                ExportedOperatorMetadata {
                    fixity: ExportedOperatorFixity::Infix(ExportedOperatorAssociativity::Left),
                    precedence: 50,
                },
            )],
        )],
    )
    .expect("operator metadata environment should build");
    let locals = crate::LocalLexicalDeclarations::empty();

    let metadata = env.operator_metadata_at("+", 0, &locals);

    assert_eq!(metadata.len(), 1);
    assert_eq!(metadata[0].spelling, "+");
    assert_eq!(metadata[0].activation_start, 0);
    assert_eq!(
        metadata[0].operator,
        ExportedOperatorMetadata {
            fixity: ExportedOperatorFixity::Infix(ExportedOperatorAssociativity::Left),
            precedence: 50,
        }
    );
}

#[test]
fn local_operator_metadata_is_active_only_after_declaration_completion() {
    let source = concat!(
        "func Plus: x + y -> set;\n",
        "a + b;\n",
        "infix_operator(\"+\", left, 80);\n",
        "a + b;\n"
    );
    let raw = scan_raw(source).expect("source should raw scan");
    let locals = collect_local_lexical_declarations(&raw, module_id("current"));
    let env = build_lexical_environment(&[], &[]).expect("environment should build");
    let before_operator_use = nth_index(source, "+", 1);
    let after_operator_use = nth_index(source, "+", 3);
    let activation_start = nth_index(source, ";", 2) + 1;

    assert!(
        env.operator_metadata_at("+", before_operator_use, &locals)
            .is_empty()
    );
    let metadata = env.operator_metadata_at("+", after_operator_use, &locals);

    assert_eq!(metadata.len(), 1);
    assert_eq!(metadata[0].activation_start, activation_start);
    assert_eq!(
        metadata[0].operator,
        ExportedOperatorMetadata {
            fixity: ExportedOperatorFixity::Infix(ExportedOperatorAssociativity::Left),
            precedence: 80,
        }
    );
}

#[test]
fn local_operator_metadata_does_not_forward_reference_later_functors() {
    let source = concat!(
        "infix_operator(\"+\", left, 80);\n",
        "func Plus: x + y -> set;\n",
        "a + b;\n"
    );
    let raw = scan_raw(source).expect("source should raw scan");
    let locals = collect_local_lexical_declarations(&raw, module_id("current"));
    let env = build_lexical_environment(&[], &[]).expect("environment should build");
    let after_functor_use = nth_index(source, "+", 2);

    assert!(
        env.operator_metadata_at("+", after_functor_use, &locals)
            .is_empty(),
        "operator metadata declared before its functor spelling is active is not a forward declaration"
    );
}

#[test]
fn local_operator_metadata_requires_active_functor_with_matching_arity() {
    let source = concat!(
        "func Unary: + x -> set;\n",
        "infix_operator(\"+\", left, 80);\n",
        "a + b;\n"
    );
    let raw = scan_raw(source).expect("source should raw scan");
    let locals = collect_local_lexical_declarations(&raw, module_id("current"));
    let env = build_lexical_environment(&[], &[]).expect("environment should build");
    let after_operator_use = nth_index(source, "+", 2);

    let candidates = env.user_symbols_at("+", after_operator_use, &locals);

    assert_eq!(
        candidates
            .iter()
            .map(|candidate| candidate.arity)
            .collect::<Vec<_>>(),
        vec![UserSymbolArity::exact(1)],
        "the same spelling is active, but only as a unary functor"
    );
    assert!(
        env.operator_metadata_at("+", after_operator_use, &locals)
            .is_empty(),
        "infix operator metadata must require an already-active binary functor"
    );
}

#[test]
fn local_operator_metadata_ignores_visibility_keywords_for_lexing() {
    let source = concat!(
        "private func Plus: x + y -> set;\n",
        "public infix_operator(\"+\", left, 80);\n",
        "a + b;\n"
    );
    let raw = scan_raw(source).expect("source should raw scan");
    let locals = collect_local_lexical_declarations(&raw, module_id("current"));
    let env = build_lexical_environment(&[], &[]).expect("environment should build");
    let after_operator_use = nth_index(source, "+", 2);

    let metadata = env.operator_metadata_at("+", after_operator_use, &locals);

    assert_eq!(metadata.len(), 1);
    assert_eq!(metadata[0].operator.precedence, 80);
}

#[test]
fn local_operator_metadata_preserves_same_spelling_overload_candidates() {
    let env = build_lexical_environment(
        &[resolved_import("imported")],
        &[summary(
            "imported",
            42,
            &[exported("+", "imported#plus", "imported", 0)],
        )],
    )
    .expect("environment should build");
    let source = concat!(
        "func Plus: x + y -> set;\n",
        "infix_operator(\"+\", left, 80);\n",
        "a + b;\n"
    );
    let raw = scan_raw(source).expect("source should raw scan");
    let locals = collect_local_lexical_declarations(&raw, module_id("current"));
    let after_operator_use = nth_index(source, "+", 2);

    let candidates = env.user_symbols_at("+", after_operator_use, &locals);
    let metadata = env.operator_metadata_at("+", after_operator_use, &locals);

    assert_eq!(
        candidates
            .iter()
            .map(|candidate| candidate.source_module.as_str())
            .collect::<Vec<_>>(),
        vec!["imported", "current"]
    );
    assert_eq!(metadata.len(), 1);
    assert_eq!(metadata[0].operator.precedence, 80);
}

#[test]
fn latest_active_operator_metadata_is_returned_before_imported_metadata() {
    let env = build_lexical_environment(
        &[resolved_import("imported")],
        &[summary(
            "imported",
            43,
            &[exported_with_operator_metadata(
                "+",
                "imported#plus",
                "imported",
                0,
                ExportedOperatorMetadata {
                    fixity: ExportedOperatorFixity::Infix(ExportedOperatorAssociativity::Left),
                    precedence: 50,
                },
            )],
        )],
    )
    .expect("environment should build");
    let source = "infix_operator(\"+\", left, 80);\na + b;\n";
    let raw = scan_raw(source).expect("source should raw scan");
    let locals = collect_local_lexical_declarations(&raw, module_id("current"));
    let after_operator_use = nth_index(source, "+", 1);

    let metadata = env.operator_metadata_at("+", after_operator_use, &locals);

    assert_eq!(
        metadata
            .iter()
            .map(|metadata| metadata.operator.precedence)
            .collect::<Vec<_>>(),
        vec![80, 50]
    );
}

#[test]
fn lexical_environment_fingerprint_changes_with_symbol_metadata() {
    let imports = vec![resolved_import("metadata")];
    let functor_env = build_lexical_environment(
        &imports,
        &[summary(
            "metadata",
            27,
            &[exported_with_metadata(
                "op",
                "metadata#op",
                "metadata",
                0,
                UserSymbolKind::Functor,
                UserSymbolArity::exact(2),
            )],
        )],
    )
    .expect("functor metadata environment should build");
    let predicate_env = build_lexical_environment(
        &imports,
        &[summary(
            "metadata",
            27,
            &[exported_with_metadata(
                "op",
                "metadata#op",
                "metadata",
                0,
                UserSymbolKind::Predicate,
                UserSymbolArity::exact(2),
            )],
        )],
    )
    .expect("predicate metadata environment should build");
    let unary_functor_env = build_lexical_environment(
        &imports,
        &[summary(
            "metadata",
            27,
            &[exported_with_metadata(
                "op",
                "metadata#op",
                "metadata",
                0,
                UserSymbolKind::Functor,
                UserSymbolArity::exact(1),
            )],
        )],
    )
    .expect("arity metadata environment should build");
    let prefix_operator_env = build_lexical_environment(
        &imports,
        &[summary(
            "metadata",
            27,
            &[exported_with_operator_metadata(
                "op",
                "metadata#op",
                "metadata",
                0,
                ExportedOperatorMetadata {
                    fixity: ExportedOperatorFixity::Prefix,
                    precedence: 70,
                },
            )],
        )],
    )
    .expect("prefix operator metadata environment should build");
    let postfix_operator_env = build_lexical_environment(
        &imports,
        &[summary(
            "metadata",
            27,
            &[exported_with_operator_metadata(
                "op",
                "metadata#op",
                "metadata",
                0,
                ExportedOperatorMetadata {
                    fixity: ExportedOperatorFixity::Postfix,
                    precedence: 70,
                },
            )],
        )],
    )
    .expect("postfix operator metadata environment should build");
    let right_infix_operator_env = build_lexical_environment(
        &imports,
        &[summary(
            "metadata",
            27,
            &[exported_with_operator_metadata(
                "op",
                "metadata#op",
                "metadata",
                0,
                ExportedOperatorMetadata {
                    fixity: ExportedOperatorFixity::Infix(ExportedOperatorAssociativity::Right),
                    precedence: 70,
                },
            )],
        )],
    )
    .expect("infix operator metadata environment should build");
    let left_infix_operator_env = build_lexical_environment(
        &imports,
        &[summary(
            "metadata",
            27,
            &[exported_with_operator_metadata(
                "op",
                "metadata#op",
                "metadata",
                0,
                ExportedOperatorMetadata {
                    fixity: ExportedOperatorFixity::Infix(ExportedOperatorAssociativity::Left),
                    precedence: 70,
                },
            )],
        )],
    )
    .expect("left-infix operator metadata environment should build");
    let left_infix_higher_precedence_env = build_lexical_environment(
        &imports,
        &[summary(
            "metadata",
            27,
            &[exported_with_operator_metadata(
                "op",
                "metadata#op",
                "metadata",
                0,
                ExportedOperatorMetadata {
                    fixity: ExportedOperatorFixity::Infix(ExportedOperatorAssociativity::Left),
                    precedence: 71,
                },
            )],
        )],
    )
    .expect("operator precedence metadata environment should build");

    assert_ne!(functor_env.fingerprint, predicate_env.fingerprint);
    assert_ne!(functor_env.fingerprint, unary_functor_env.fingerprint);
    assert_ne!(functor_env.fingerprint, prefix_operator_env.fingerprint);
    assert_ne!(
        prefix_operator_env.fingerprint,
        postfix_operator_env.fingerprint
    );
    assert_ne!(
        prefix_operator_env.fingerprint,
        right_infix_operator_env.fingerprint
    );
    assert_ne!(
        left_infix_operator_env.fingerprint,
        right_infix_operator_env.fingerprint
    );
    assert_ne!(
        left_infix_operator_env.fingerprint,
        left_infix_higher_precedence_env.fingerprint
    );
}

#[test]
fn lexical_environment_rejects_invalid_user_symbol_arity() {
    let error = build_lexical_environment(
        &[resolved_import("bad.arity")],
        &[summary(
            "bad.arity",
            28,
            &[exported_with_metadata(
                "op",
                "bad.arity#op",
                "bad.arity",
                0,
                UserSymbolKind::Functor,
                UserSymbolArity::range(3, 2),
            )],
        )],
    )
    .expect_err("invalid arity shape should fail");

    assert!(matches!(
        error,
        LexicalEnvironmentError::InvalidUserSymbolArity { .. }
    ));
}

#[test]
fn lexical_environment_rejects_operator_metadata_on_non_functors() {
    let mut symbol = exported_with_metadata(
        "op",
        "bad.operator_kind#op",
        "bad.operator_kind",
        0,
        UserSymbolKind::Predicate,
        UserSymbolArity::exact(2),
    );
    symbol.operator = Some(ExportedOperatorMetadata {
        fixity: ExportedOperatorFixity::Infix(ExportedOperatorAssociativity::Left),
        precedence: 50,
    });

    let error = build_lexical_environment(
        &[resolved_import("bad.operator_kind")],
        &[summary("bad.operator_kind", 29, &[symbol])],
    )
    .expect_err("operator metadata is only valid for functors");

    assert!(matches!(
        error,
        LexicalEnvironmentError::InvalidOperatorMetadata { .. }
    ));
}

#[test]
fn lexical_environment_rejects_operator_metadata_with_incompatible_arity() {
    let mut symbol = exported_with_metadata(
        "op",
        "bad.operator_arity#op",
        "bad.operator_arity",
        0,
        UserSymbolKind::Functor,
        UserSymbolArity::exact(1),
    );
    symbol.operator = Some(ExportedOperatorMetadata {
        fixity: ExportedOperatorFixity::Infix(ExportedOperatorAssociativity::Left),
        precedence: 50,
    });

    let error = build_lexical_environment(
        &[resolved_import("bad.operator_arity")],
        &[summary("bad.operator_arity", 30, &[symbol])],
    )
    .expect_err("infix operator metadata requires binary functor arity");

    assert!(matches!(
        error,
        LexicalEnvironmentError::InvalidOperatorMetadata { .. }
    ));
}

#[test]
fn lexical_environment_admits_constructor_name_entries_for_type_like_kinds() {
    let env = build_lexical_environment(
        &[resolved_import("constructors")],
        &[summary(
            "constructors",
            31,
            &[
                exported_with_metadata(
                    "1-sorted",
                    "constructors#one_sorted",
                    "constructors",
                    0,
                    UserSymbolKind::Mode,
                    UserSymbolArity::exact(0),
                ),
                exported_with_metadata(
                    "C-star-algebraic",
                    "constructors#c_star",
                    "constructors",
                    1,
                    UserSymbolKind::Attribute,
                    UserSymbolArity::exact(1),
                ),
                exported_with_metadata(
                    "R-module",
                    "constructors#r_module",
                    "constructors",
                    2,
                    UserSymbolKind::Structure,
                    UserSymbolArity::exact(0),
                ),
            ],
        )],
    )
    .expect("readable constructor names should be accepted for type-like kinds");

    assert_eq!(
        env.user_symbol("1-sorted")
            .map(|candidate| (candidate.kind, candidate.arity)),
        Some((UserSymbolKind::Mode, UserSymbolArity::exact(0)))
    );
    assert_eq!(
        env.user_symbol("C-star-algebraic")
            .map(|candidate| (candidate.kind, candidate.arity)),
        Some((UserSymbolKind::Attribute, UserSymbolArity::exact(1)))
    );
    assert_eq!(
        env.user_symbol("R-module")
            .map(|candidate| (candidate.kind, candidate.arity)),
        Some((UserSymbolKind::Structure, UserSymbolArity::exact(0)))
    );
}

#[test]
fn lexical_environment_admits_predicate_free_form_punctuation_entries() {
    let env = build_lexical_environment(
        &[resolved_import("predicates")],
        &[summary(
            "predicates",
            38,
            &[exported_with_metadata(
                "<~>",
                "predicates#relation",
                "predicates",
                0,
                UserSymbolKind::Predicate,
                UserSymbolArity::exact(2),
            )],
        )],
    )
    .expect("predicate notation may use free-form punctuation");

    assert_eq!(
        env.user_symbol("<~>")
            .map(|candidate| (candidate.kind, candidate.arity)),
        Some((UserSymbolKind::Predicate, UserSymbolArity::exact(2)))
    );
}

#[test]
fn lexical_environment_rejects_symbolic_constructor_name_entries() {
    for (spelling, kind) in [
        ("+", UserSymbolKind::Mode),
        ("[:", UserSymbolKind::Attribute),
        ("C star", UserSymbolKind::Structure),
    ] {
        let error = build_lexical_environment(
            &[resolved_import("bad.constructors")],
            &[summary(
                "bad.constructors",
                32,
                &[exported_with_metadata(
                    spelling,
                    "bad.constructors#symbolic",
                    "bad.constructors",
                    0,
                    kind,
                    UserSymbolArity::exact(0),
                )],
            )],
        )
        .expect_err("type-like entries require constructor names");

        assert!(matches!(
            error,
            LexicalEnvironmentError::InvalidConstructorNameSpelling { .. }
        ));
    }
}

#[test]
fn lexical_environment_rejects_selector_and_generic_constructor_summary_entries() {
    for kind in [UserSymbolKind::Selector, UserSymbolKind::Constructor] {
        let error = build_lexical_environment(
            &[resolved_import("bad.legacy_kinds")],
            &[summary(
                "bad.legacy_kinds",
                33,
                &[exported_with_metadata(
                    "legacy_name",
                    "bad.legacy_kinds#legacy",
                    "bad.legacy_kinds",
                    0,
                    kind,
                    UserSymbolArity::exact(0),
                )],
            )],
        )
        .expect_err("selectors and generic constructors are not lexer entries");

        assert!(matches!(
            error,
            LexicalEnvironmentError::UnsupportedLexicalEntryKind { .. }
        ));
    }
}

#[test]
fn lexical_environment_rejects_illegal_reserved_collisions() {
    let word_error = build_lexical_environment(
        &[resolved_import("bad.words")],
        &[summary(
            "bad.words",
            34,
            &[exported("theorem", "bad.words#theorem", "bad.words", 0)],
        )],
    )
    .expect_err("reserved word collision should fail");
    assert!(matches!(
        word_error,
        LexicalEnvironmentError::ReservedWordCollision { .. }
    ));

    let symbol_error = build_lexical_environment(
        &[resolved_import("bad.symbols")],
        &[summary(
            "bad.symbols",
            35,
            &[exported(":=", "bad.symbols#assign", "bad.symbols", 0)],
        )],
    )
    .expect_err("reserved symbol collision should fail");
    assert!(matches!(
        symbol_error,
        LexicalEnvironmentError::ReservedSymbolCollision { .. }
    ));
}

#[test]
fn lexical_environment_rejects_invalid_user_symbol_spelling() {
    let error = build_lexical_environment(
        &[resolved_import("bad.annotations")],
        &[summary(
            "bad.annotations",
            36,
            &[exported(
                "@bad",
                "bad.annotations#bad",
                "bad.annotations",
                0,
            )],
        )],
    )
    .expect_err("annotation marker characters are not valid user symbols");

    assert!(matches!(
        error,
        LexicalEnvironmentError::InvalidUserSymbolSpelling { .. }
    ));
}

#[test]
fn lexical_environment_allows_dot_user_symbol_exception() {
    let env = build_lexical_environment(
        &[resolved_import("std.application")],
        &[summary(
            "std.application",
            33,
            &[exported(".", "std.application#dot", "std.application", 0)],
        )],
    )
    .expect("dot is the reserved-symbol collision exception");

    assert_eq!(
        env.user_symbol(".").expect("dot user symbol").symbol_id,
        symbol_id("std.application#dot")
    );
}

#[test]
fn lexical_environment_restricts_dot_exception_to_functors() {
    let error = build_lexical_environment(
        &[resolved_import("bad.dot_predicate")],
        &[summary(
            "bad.dot_predicate",
            37,
            &[exported_with_metadata(
                ".",
                "bad.dot_predicate#dot",
                "bad.dot_predicate",
                0,
                UserSymbolKind::Predicate,
                UserSymbolArity::exact(2),
            )],
        )],
    )
    .expect_err("only functors may use the reserved dot exception");

    assert!(matches!(
        error,
        LexicalEnvironmentError::ReservedSymbolCollision { .. }
    ));
}

#[test]
fn lexical_environment_rejects_dot_exception_for_type_like_kinds() {
    for kind in [
        UserSymbolKind::Mode,
        UserSymbolKind::Attribute,
        UserSymbolKind::Structure,
    ] {
        let error = build_lexical_environment(
            &[resolved_import("bad.dot_type_like")],
            &[summary(
                "bad.dot_type_like",
                39,
                &[exported_with_metadata(
                    ".",
                    "bad.dot_type_like#dot",
                    "bad.dot_type_like",
                    0,
                    kind,
                    UserSymbolArity::exact(0),
                )],
            )],
        )
        .expect_err("dot exception is functor-only");

        assert!(matches!(
            error,
            LexicalEnvironmentError::ReservedSymbolCollision { .. }
        ));
    }
}

#[test]
fn lexical_environment_fingerprint_is_stable_for_same_ordered_inputs() {
    let imports = vec![resolved_import("std.first"), resolved_import("std.second")];
    let summaries = vec![
        summary(
            "std.second",
            42,
            &[exported("*+", "s#star", "std.second", 0)],
        ),
        summary(
            "std.first",
            41,
            &[exported("succ", "f#succ", "std.first", 0)],
        ),
    ];

    let first =
        build_lexical_environment(&imports, &summaries).expect("first environment should build");
    let second =
        build_lexical_environment(&imports, &summaries).expect("second environment should build");
    let reversed_imports = vec![resolved_import("std.second"), resolved_import("std.first")];
    let reversed = build_lexical_environment(&reversed_imports, &summaries)
        .expect("reversed environment should build");

    assert_eq!(first.fingerprint, second.fingerprint);
    assert_ne!(first.fingerprint, reversed.fingerprint);
}

#[test]
fn lexical_environment_reports_missing_and_inconsistent_summaries() {
    let missing = build_lexical_environment(&[resolved_import("missing")], &[])
        .expect_err("missing summary should fail");
    assert!(matches!(
        missing,
        LexicalEnvironmentError::MissingModuleSummary { .. }
    ));

    let inconsistent = build_lexical_environment(
        &[resolved_import("dup")],
        &[
            summary("dup", 1, &[exported("+", "dup#plus", "dup", 0)]),
            summary("dup", 2, &[exported("+", "dup#plus", "dup", 0)]),
        ],
    )
    .expect_err("inconsistent duplicate summary should fail");
    assert!(matches!(
        inconsistent,
        LexicalEnvironmentError::InconsistentDuplicateSummary { .. }
    ));

    let same_fingerprint_different_exports = build_lexical_environment(
        &[resolved_import("same_hash")],
        &[
            summary(
                "same_hash",
                5,
                &[exported("+", "same_hash#plus", "same_hash", 0)],
            ),
            summary(
                "same_hash",
                5,
                &[exported("*", "same_hash#star", "same_hash", 0)],
            ),
        ],
    )
    .expect_err("duplicate summary content must match exactly");
    assert!(matches!(
        same_fingerprint_different_exports,
        LexicalEnvironmentError::InconsistentDuplicateSummary { .. }
    ));
}

#[test]
fn lexical_environment_treats_summary_order_as_canonical_input() {
    let imports = vec![resolved_import("canonical")];
    let canonical = vec![summary(
        "canonical",
        61,
        &[
            exported("+", "canonical#plus", "canonical", 0),
            exported("*", "canonical#star", "canonical", 1),
        ],
    )];
    let reordered = vec![summary(
        "canonical",
        61,
        &[
            exported("*", "canonical#star", "canonical", 1),
            exported("+", "canonical#plus", "canonical", 0),
        ],
    )];

    let canonical_env =
        build_lexical_environment(&imports, &canonical).expect("canonical summary should build");
    let reordered_env = build_lexical_environment(&imports, &reordered)
        .expect("environment does not recanonicalize summaries");

    assert_ne!(canonical_env.fingerprint, reordered_env.fingerprint);
}
