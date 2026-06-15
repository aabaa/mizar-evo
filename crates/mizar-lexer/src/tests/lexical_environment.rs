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
fn lexical_environment_rejects_illegal_reserved_collisions() {
    let word_error = build_lexical_environment(
        &[resolved_import("bad.words")],
        &[summary(
            "bad.words",
            31,
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
            32,
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
            34,
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
