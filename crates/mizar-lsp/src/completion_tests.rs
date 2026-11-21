use super::{Context, byte_offset, completion_context, sort_items, syntax_items};
use crate::{LspPosition, server::full_snapshot_registry_a::RegistryFixture};
use mizar_artifact::module_summary::ModuleSummaryIdentity;
use serde_json::{Value, json};

fn position(text: &str, byte: usize) -> LspPosition {
    let before = &text[..byte];
    let line = before
        .bytes()
        .filter(|byte| *byte == b'\n' || *byte == b'\r')
        .count() as u32;
    let line_start = before.rfind(['\n', '\r']).map_or(0, |index| index + 1);
    LspPosition {
        line,
        character: before[line_start..].encode_utf16().count() as u32,
    }
}

fn at_end(text: &str) -> Option<(Context, String)> {
    completion_context(text, position(text, text.len()))
}

fn genuine_notation_fixture(
    overload: bool,
) -> (
    String,
    Option<mizar_artifact::module_summary::ModuleSummary>,
    mizar_resolve::symbols::SourceNotationView,
) {
    let mut source_text = concat!(
        "definition\n",
        "mode M: MyMode is set;\n",
        "let x, y be set;\n",
        "func Base: x base y -> set equals x;\n",
        "pred Related: x related y means thesis;\n",
        "func Plus: +(x,y) -> set equals x;\n",
        "infix_operator(\"+\", left, 80);\n",
        "infix_operator(\"+\", none, 80);\n",
        "private mode Hidden: HiddenMode is set;\n",
        "end;\n",
    )
    .to_owned();
    if overload {
        source_text
            .push_str("definition\nlet x be set;\nfunc Unary: base x -> set equals x;\nend;\n");
        source_text
            .push_str("definition\nlet a be set;\nfunc Multi: |. a .| -> set equals a;\nend;\n");
    }
    genuine_notation_source(&source_text)
}

fn genuine_notation_source(
    source_text: &str,
) -> (
    String,
    Option<mizar_artifact::module_summary::ModuleSummary>,
    mizar_resolve::symbols::SourceNotationView,
) {
    use mizar_frontend::{
        lexical_env::{FrontendLexicalEnvironmentError, LexicalSummaryProvider, ResolvedImports},
        orchestration::Frontend,
        parsing::MizarParserSeam,
        source::FrontendSourceLoader,
    };
    use mizar_resolve::{
        declarations::DeclarationShellCollector, resolved_ast::ModuleId,
        symbols::SignatureProjectionExtractor,
    };
    use mizar_session::{
        BuildSnapshotId, DiskSourceLoader, Edition, Hash, InMemorySessionIdAllocator, ModulePath,
        PackageId, SourceInput, SourceOriginInput, normalize_path,
    };

    struct NoImports;
    impl LexicalSummaryProvider for NoImports {
        fn resolve_imports(
            &self,
            _request: &mizar_frontend::lexical_env::LexicalEnvironmentRequest<'_>,
        ) -> Result<ResolvedImports, FrontendLexicalEnvironmentError> {
            Ok(ResolvedImports {
                imports: vec![],
                summaries: vec![],
                diagnostics: vec![],
            })
        }
    }

    let source_text = format!("\u{feff}{}", source_text.replace('\n', "\r\n"));
    let root = std::env::temp_dir().join(format!(
        "mizar-lsp-completion-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let path = root.join("src/main.miz");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, &source_text).unwrap();
    let frontend = Frontend::new(
        FrontendSourceLoader::new(DiskSourceLoader::new(&root)),
        NoImports,
        MizarParserSeam,
    );
    let output = frontend
        .run(
            mizar_frontend::source::SourceUnitRequest {
                snapshot: BuildSnapshotId::from_published_schema_str(&format!(
                    "mizar-session-build-snapshot-v1:{}",
                    "31".repeat(Hash::BYTE_LEN)
                ))
                .unwrap(),
                input: SourceInput {
                    package_id: PackageId::new("app"),
                    module_path: ModulePath::new("main"),
                    normalized_path: normalize_path(&root, &path).unwrap(),
                    edition: Edition::new("2026"),
                    origin: SourceOriginInput::Disk { path },
                },
            },
            &InMemorySessionIdAllocator::new(),
        )
        .unwrap();
    std::fs::remove_dir_all(root).unwrap();
    assert!(
        output.diagnostics.is_empty(),
        "genuine fixture frontend diagnostics: {:?}",
        output.diagnostics
    );
    let ast = output.ast.as_ref().expect("genuine parsed source");
    let module = ModuleId::new(
        output.source.package_id.clone(),
        output.source.module_path.clone(),
    );
    let shells = DeclarationShellCollector::new(ast, &module).collect();
    let collection = SignatureProjectionExtractor::new(
        ast,
        &shells,
        mizar_resolve::env::NamespacePath::new(module.path().as_str()),
    )
    .collect(&module);
    assert!(
        collection.diagnostics().is_empty(),
        "genuine fixture resolver diagnostics: {:?}",
        collection.diagnostics()
    );
    let identity = ModuleSummaryIdentity {
        package_id: output.source.package_id.as_str().to_owned(),
        package_version: Some("0.1.0".into()),
        lockfile_identity: None,
        module_path: output.source.module_path.as_str().to_owned(),
        language_edition: output.source.edition.as_str().to_owned(),
    };
    let summary = collection
        .source_notation_module_summary_candidate(&output, &identity)
        .unwrap();
    let notation = collection
        .source_notation_view(&output)
        .expect("genuine authenticated notation view");
    (source_text, summary, notation)
}

fn labels(items: &[Value]) -> Vec<&str> {
    items
        .iter()
        .map(|item| item["label"].as_str().expect("completion label"))
        .collect()
}

#[test]
fn lexical_cues_filter_syntax_candidates_and_restart_after_semicolon() {
    let cases = [
        ("be ", Context::Types, vec!["object", "set"]),
        ("being se", Context::Types, vec!["set"]),
        ("X->se", Context::Types, vec!["set"]),
        ("X = ", Context::Terms, vec!["the"]),
        ("X equals ", Context::Terms, vec!["the"]),
        ("holds ", Context::Formulas, vec!["ex", "for", "not"]),
        ("X and ", Context::Formulas, vec!["ex", "for", "not"]),
        ("by ", Context::Citations, vec![]),
        ("by T, ", Context::Citations, vec![]),
        ("by T; be ", Context::Types, vec!["object", "set"]),
        ("unbounded ", Context::Keywords, vec![]),
    ];
    for (text, expected_context, expected_labels) in cases {
        let context = at_end(text).expect("valid lexical context");
        assert_eq!(context.0, expected_context, "{text:?}");
        assert_eq!(labels(&syntax_items(&context)), expected_labels, "{text:?}");
    }
    assert_eq!(
        at_end("Pkg.Mod"),
        Some((Context::QualifiedSymbols, "Pkg.Mod".into()))
    );
    assert_eq!(
        at_end("import Pkg.Mod"),
        Some((Context::Keywords, "Pkg.Mod".into()))
    );
}

#[test]
fn comments_strings_and_invalid_utf16_boundaries_refuse_completion() {
    for (text, byte) in [
        ("be :: not code", "be :: not code".len()),
        ("be @latex(\"set\")", "be @latex(\"set\")".len() - 2),
        ("@latex(\"T\")", "@latex(\"T\")".len() - 2),
    ] {
        assert_eq!(
            completion_context(text, position(text, byte)),
            None,
            "{text:?}"
        );
    }

    let multiline_comment = "be ::= comment\n\n=::";
    assert_eq!(
        completion_context(
            multiline_comment,
            LspPosition {
                line: 1,
                character: 0,
            },
        ),
        None,
        "an empty line inside a multiline comment is not a completion context",
    );
    for text in ["be :: comment\n", "be ::: comment\n"] {
        assert_eq!(
            at_end(text),
            Some((Context::Types, String::new())),
            "line and documentation comments leave their newline usable: {text:?}",
        );
    }

    let text = "😀be ";
    assert_eq!(
        byte_offset(
            text,
            LspPosition {
                line: 0,
                character: 1
            }
        ),
        None,
        "a cursor between the surrogate halves has no byte offset",
    );
    assert_eq!(
        completion_context(
            text,
            LspPosition {
                line: 0,
                character: 5
            }
        ),
        Some((Context::Types, String::new())),
        "the cursor after the complete type cue is valid",
    );
    assert_eq!(
        completion_context(
            text,
            LspPosition {
                line: 0,
                character: 8
            }
        ),
        None,
        "a valid integer coordinate beyond the line has no lexical context",
    );
}

#[test]
fn byte_offsets_preserve_bom_and_crlf_utf16_coordinates() {
    let text = "\u{feff}x\r\n😀be ";
    assert_eq!(
        byte_offset(
            text,
            LspPosition {
                line: 0,
                character: 0
            }
        ),
        Some(0)
    );
    assert_eq!(
        byte_offset(
            text,
            LspPosition {
                line: 0,
                character: 2
            }
        ),
        Some(4)
    );
    assert_eq!(
        byte_offset(
            text,
            LspPosition {
                line: 1,
                character: 0
            }
        ),
        Some(6)
    );
    assert_eq!(
        byte_offset(
            text,
            LspPosition {
                line: 1,
                character: 2
            }
        ),
        Some(10)
    );
    assert_eq!(
        byte_offset(
            text,
            LspPosition {
                line: 1,
                character: 5
            }
        ),
        Some(13)
    );
    assert_eq!(
        byte_offset(
            text,
            LspPosition {
                line: 1,
                character: 1
            }
        ),
        None
    );
    assert_eq!(
        byte_offset(
            text,
            LspPosition {
                line: 2,
                character: 0
            }
        ),
        None
    );
}

#[test]
fn repeated_and_permuted_candidate_rows_have_identical_order_and_sort_text() {
    let rows = vec![
        json!({"label":"Beta","data":{"rank":1,"identity":"b"}}),
        json!({"label":"alpha","data":{"rank":0,"identity":"a"}}),
        json!({"label":"Alpha","data":{"rank":1,"identity":"z"}}),
    ];
    let first = sort_items(rows.clone(), "A");
    let repeated = sort_items(rows.clone(), "A");
    let permuted = sort_items(rows.into_iter().rev().collect(), "A");
    assert_eq!(first, repeated);
    assert_eq!(first, permuted);
    assert_eq!(labels(&first), ["alpha", "Alpha", "Beta"]);
    assert_eq!(first[0]["sortText"], "00000000");
    assert_eq!(first[1]["sortText"], "00000001");
    assert_eq!(first[2]["sortText"], "00000002");
}

#[test]
fn genuine_frontend_notation_filters_modes_functors_and_predicates_by_context() {
    use super::append_semantic_items;
    let (source, summary, notation) = genuine_notation_fixture(false);
    let summary = summary.expect("genuine source notation summary candidate");
    assert!(
        notation
            .declarations()
            .iter()
            .any(|row| row.declaration().spelling == "MyMode")
    );
    assert!(
        notation
            .declarations()
            .iter()
            .any(|row| row.declaration().spelling == "base")
    );
    assert!(
        notation
            .declarations()
            .iter()
            .any(|row| row.declaration().spelling == "related")
    );
    for (prefix_text, context_kind, required, forbidden) in [
        (
            "be ",
            Context::Types,
            vec!["MyMode", "object", "set"],
            vec!["base", "related"],
        ),
        (
            "x = ",
            Context::Terms,
            vec!["+", "base", "the"],
            vec!["MyMode", "related"],
        ),
        (
            "x holds ",
            Context::Formulas,
            vec!["related", "ex", "for", "not"],
            vec!["MyMode", "base"],
        ),
    ] {
        let context = at_end(prefix_text).unwrap();
        assert_eq!(context.0, context_kind);
        let mut items = syntax_items(&context);
        append_semantic_items(
            &mut items,
            &summary,
            Some(&notation),
            None,
            (source.len(), source.len()),
            &context,
            "Source: fixture (version 1)",
        );
        let items = sort_items(items, &context.1);
        let actual = labels(&items);
        for label in required {
            assert!(
                actual.contains(&label),
                "{actual:?} lacks {label:?} in {prefix_text:?}"
            );
        }
        for label in forbidden {
            assert!(
                !actual.contains(&label),
                "{actual:?} unexpectedly includes {label:?} in {prefix_text:?}"
            );
        }
    }
    let prefix_context = at_end("be My").unwrap();
    let mut filtered = syntax_items(&prefix_context);
    append_semantic_items(
        &mut filtered,
        &summary,
        Some(&notation),
        None,
        (source.len(), source.len()),
        &prefix_context,
        "Source: fixture (version 1)",
    );
    assert_eq!(labels(&filtered), ["MyMode"]);
    let hidden = notation
        .declarations()
        .iter()
        .find(|row| row.declaration().spelling == "HiddenMode")
        .unwrap();
    assert_eq!(
        hidden.owner().visibility(),
        mizar_resolve::env::Visibility::Private
    );

    let mode = notation
        .declarations()
        .iter()
        .find(|row| row.declaration().spelling == "MyMode")
        .unwrap();
    let activation = mode.activation().original_input.unwrap();
    let type_context = at_end("be ").unwrap();
    let mut before = syntax_items(&type_context);
    append_semantic_items(
        &mut before,
        &summary,
        Some(&notation),
        None,
        (
            activation.start.saturating_sub(1),
            activation.start.saturating_sub(1),
        ),
        &type_context,
        "Source: fixture (version 1)",
    );
    assert!(
        !labels(&before).contains(&"MyMode"),
        "local notation is unavailable before its raw CRLF source boundary"
    );
    let mut active = syntax_items(&type_context);
    append_semantic_items(
        &mut active,
        &summary,
        Some(&notation),
        None,
        (activation.end, activation.end),
        &type_context,
        "Source: fixture (version 1)",
    );
    assert!(
        labels(&active).contains(&"MyMode"),
        "local notation activates at its original raw source boundary"
    );

    let (overload_source, _, overload_notation) = genuine_notation_fixture(true);
    let mut overload_items = syntax_items(&at_end("x = ").unwrap());
    let term_context = at_end("x = ").unwrap();
    append_semantic_items(
        &mut overload_items,
        &summary,
        Some(&overload_notation),
        None,
        (overload_source.len(), overload_source.len()),
        &term_context,
        "Source: overload fixture (version 1)",
    );
    let overload_items = sort_items(overload_items, &term_context.1);
    let overloads = overload_items
        .iter()
        .filter(|item| item["label"] == "base")
        .collect::<Vec<_>>();
    assert_eq!(
        overloads.len(),
        2,
        "both genuine same-spelling arities survive"
    );
    assert_ne!(
        overloads[0]["data"]["identity"],
        overloads[1]["data"]["identity"]
    );
    assert_ne!(overloads[0]["detail"], overloads[1]["detail"]);
    let latest = overload_notation.operator_events().last().unwrap();
    let plus = overload_items
        .iter()
        .find(|item| item["label"] == "+")
        .unwrap();
    assert!(
        plus["detail"]
            .as_str()
            .unwrap()
            .contains(&format!("operator {:?}", latest.declaration().operator))
    );
    for arity in [1, 2] {
        assert!(
            overloads
                .iter()
                .any(|item| item["detail"].as_str().unwrap().contains(&format!(
                    "arity {:?}",
                    mizar_lexer::UserSymbolArity::exact(arity)
                )))
        );
    }
    let pieces = overload_notation
        .declarations()
        .iter()
        .filter(|row| matches!(row.declaration().spelling.as_str(), "|." | ".|"))
        .collect::<Vec<_>>();
    assert_eq!(pieces.len(), 2);
    assert_eq!(pieces[0].owner().symbol(), pieces[1].owner().symbol());
    for row in pieces {
        let item = overload_items
            .iter()
            .find(|item| item["label"] == row.declaration().spelling)
            .unwrap();
        assert!(item["detail"].as_str().unwrap().contains(&format!(
            "arity {:?}",
            mizar_lexer::UserSymbolArity::exact(1)
        )));
    }
    let (alias_source, _, alias_notation) = genuine_notation_source(concat!(
        "definition\nlet x, y be set;\nfunc Base: x base y -> set equals x;\npred Related: x related y means thesis;\nend;\n",
        "definition\nlet x, y be set;\nsynonym x alias y for x base y;\nend;\n",
        "definition\nlet x, y be set;\nantonym x opposite y for x related y;\nend;\n",
    ));
    for (spelling, kind, relation, context) in [
        (
            "alias",
            mizar_lexer::UserSymbolKind::Functor,
            mizar_resolve::env::RelationKind::Synonym,
            &term_context,
        ),
        (
            "opposite",
            mizar_lexer::UserSymbolKind::Predicate,
            mizar_resolve::env::RelationKind::Antonym,
            &at_end("holds ").unwrap(),
        ),
    ] {
        let row = alias_notation
            .declarations()
            .iter()
            .find(|row| row.declaration().spelling == spelling)
            .unwrap();
        assert_eq!(row.declaration().kind, kind);
        assert_eq!(row.owner().relations().len(), 1);
        assert_eq!(row.owner().relations()[0].kind(), relation);
        assert_ne!(row.owner().symbol(), row.owner().relations()[0].target());
        let mut items = Vec::new();
        append_semantic_items(
            &mut items,
            &summary,
            Some(&alias_notation),
            None,
            (alias_source.len(), alias_source.len()),
            context,
            "Source: fixture (version 1)",
        );
        let alias = items.iter().find(|item| item["label"] == spelling).unwrap();
        assert!(
            alias["detail"]
                .as_str()
                .unwrap()
                .contains(&format!("relations {:?}", row.owner().relations()))
        );
        assert!(
            alias["detail"]
                .as_str()
                .unwrap()
                .contains(&format!("{kind:?}"))
        );
    }
    let piece_prefix = at_end("= |.").unwrap();
    let mut filtered_piece = Vec::new();
    append_semantic_items(
        &mut filtered_piece,
        &summary,
        Some(&overload_notation),
        None,
        (overload_source.len(), overload_source.len()),
        &piece_prefix,
        "Source: fixture (version 1)",
    );
    assert_eq!(labels(&sort_items(filtered_piece, &piece_prefix.1)), ["|."]);
    let plus_declaration = notation
        .declarations()
        .iter()
        .find(|row| row.declaration().spelling == "+")
        .unwrap();
    for (index, event) in notation.operator_events().iter().enumerate() {
        let activation = event.activation().original_input.unwrap().start;
        for (offset, expected) in [
            (
                activation - 1,
                if index == 0 {
                    plus_declaration.declaration().operator
                } else {
                    notation.operator_events()[index - 1].declaration().operator
                },
            ),
            (activation, event.declaration().operator),
        ] {
            let mut items = Vec::new();
            append_semantic_items(
                &mut items,
                &summary,
                Some(&notation),
                None,
                (offset, offset),
                &term_context,
                "Source: fixture (version 1)",
            );
            let plus = items
                .iter()
                .filter(|item| item["label"] == "+")
                .collect::<Vec<_>>();
            assert_eq!(plus.len(), 1, "operator events do not introduce a symbol");
            assert!(
                plus[0]["detail"]
                    .as_str()
                    .unwrap()
                    .contains(&format!("operator {expected:?}"))
            );
        }
    }
    let activation = hidden.activation().original_input.unwrap().start;
    for (offset, visible) in [(activation - 1, false), (activation, true)] {
        let mut items = Vec::new();
        append_semantic_items(
            &mut items,
            &summary,
            Some(&notation),
            None,
            (offset, offset),
            &type_context,
            "Source: fixture (version 1)",
        );
        assert_eq!(labels(&items).contains(&"HiddenMode"), visible);
    }
}

#[test]
fn selected_snapshot_completion_keeps_stale_source_identity_and_current_syntax_fallback() {
    let mut fixture = RegistryFixture::new();
    let (_, activated) = fixture.build();
    assert!(activated);
    let uri = fixture.sources[0].2.clone();
    let original = fixture.server.document(&uri).unwrap().1.to_owned();
    let retained = fixture
        .server
        .lsp_snapshot(&fixture.driver, &fixture.snapshots)
        .unwrap()
        .0;
    let original_metadata = retained.publication().source_metadata().to_vec();
    let eof = LspPosition {
        line: 0,
        character: original.encode_utf16().count() as u32,
    };

    let be_offset = original.find(" be set").unwrap() + 4;
    let before_version_only = fixture.server.snapshot_completion(
        &fixture.driver,
        &fixture.snapshots,
        &uri,
        position(&original, be_offset),
    );
    fixture.did_change_version_only(&uri, 2);
    let (version_only_snapshot, version_only_freshness) = fixture
        .server
        .lsp_snapshot(&fixture.driver, &fixture.snapshots)
        .unwrap();
    assert!(std::sync::Arc::ptr_eq(&retained, &version_only_snapshot));
    assert_eq!(
        version_only_snapshot.publication().source_metadata(),
        original_metadata
    );
    assert_eq!(
        version_only_freshness,
        super::super::LspSnapshotFreshness::StaleArtifact
    );
    let after_version_only = fixture.server.snapshot_completion(
        &fixture.driver,
        &fixture.snapshots,
        &uri,
        position(&original, be_offset),
    );
    assert_eq!(labels(&before_version_only), labels(&after_version_only));
    assert_eq!(labels(&after_version_only), ["object", "set"]);

    // Keep the citation cursor at the retained source's EOF while adding an
    // in-scope proof label that has never appeared in the selected publication.
    let edited_prefix = original
        .replace("X = X proof", "X=X proof")
        .replace("thus X = X; end;", "L:X=X;thus X=X by ");
    assert_eq!(edited_prefix.len(), original.len());
    let edited = format!("{edited_prefix}Main; end;");
    let (parsed, _) = import_fixture(&edited, "main", &[]);
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    fixture.did_change_text(&uri, 3, edited.clone());
    let stale_items =
        fixture
            .server
            .snapshot_completion(&fixture.driver, &fixture.snapshots, &uri, eof);
    assert_eq!(labels(&stale_items), ["Main"], "unpublished L is excluded");
    let stale_main = stale_items
        .iter()
        .find(|item| item["label"] == "Main")
        .expect("retained theorem label remains available at the original EOF");
    let stale_identity = stale_main["data"]["identity"].clone();
    let stale_detail = stale_main["detail"].as_str().unwrap();
    assert!(stale_detail.contains(&format!("stale_artifact: Source: {uri} (version 1)")));
    assert!(stale_main.get("textEdit").is_none());
    assert!(stale_main.get("additionalTextEdits").is_none());
    assert!(stale_main.get("range").is_none());

    let host_response = fixture
        .server
        .handle_message_with_metadata(
            json!({
                "jsonrpc": "2.0", "id": 52, "method": "textDocument/completion",
                "params": {"textDocument": {"uri": uri}, "position": {
                    "line": eof.line, "character": eof.character
                }}
            }),
            &fixture.driver,
            &fixture.snapshots,
        )
        .unwrap();
    assert_eq!(host_response["id"], 52);
    let host_main = host_response["result"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["label"] == "Main")
        .expect("metadata-host route enriches completion with published labels");
    assert_eq!(host_main["data"]["identity"], stale_identity);
    assert!(
        host_main["detail"]
            .as_str()
            .unwrap()
            .contains("stale_artifact")
    );

    fixture.did_change_version_only(&uri, 4);
    let version_only_items =
        fixture
            .server
            .snapshot_completion(&fixture.driver, &fixture.snapshots, &uri, eof);
    let version_only_main = version_only_items
        .iter()
        .find(|item| item["label"] == "Main")
        .unwrap();
    assert_eq!(version_only_main["data"]["identity"], stale_identity);
    assert_eq!(version_only_main["detail"], stale_main["detail"]);

    fixture.did_close_and_reopen(&uri, 4);
    let reopened_items =
        fixture
            .server
            .snapshot_completion(&fixture.driver, &fixture.snapshots, &uri, eof);
    let reopened_main = reopened_items
        .iter()
        .find(|item| item["label"] == "Main")
        .expect("identical-tuple reopen retains the selected publication candidate");
    assert_eq!(reopened_main["data"]["identity"], stale_identity);
    assert!(
        reopened_main["detail"]
            .as_str()
            .unwrap()
            .contains(&format!("stale_artifact: Source: {uri} (version 1)"))
    );

    // Renaming the genuine theorem replaces the selected publication and its
    // semantic candidate identity while keeping the source shape unchanged.
    let replacement_source = original.replacen("theorem Main:", "theorem Next:", 1);
    fixture.did_change_text(&uri, 5, replacement_source.clone());
    let (_, replacement_activated) = fixture.build();
    assert!(replacement_activated);
    let (replacement, replacement_freshness) = fixture
        .server
        .lsp_snapshot(&fixture.driver, &fixture.snapshots)
        .unwrap();
    assert!(!std::sync::Arc::ptr_eq(&retained, &replacement));
    assert_eq!(
        replacement_freshness,
        super::super::LspSnapshotFreshness::Current
    );
    let replacement_items = fixture.server.snapshot_completion(
        &fixture.driver,
        &fixture.snapshots,
        &uri,
        position(&replacement_source, be_offset),
    );
    assert_eq!(labels(&replacement_items), ["object", "set"]);
    assert!(replacement_items.iter().all(|item| {
        !item["detail"]
            .as_str()
            .unwrap_or_default()
            .contains("stale_artifact")
    }));

    let replacement_stale_source = format!(
        "{}by  ",
        &replacement_source[..replacement_source.len() - 4]
    );
    assert_eq!(replacement_stale_source.len(), replacement_source.len());
    fixture.did_change_text(&uri, 6, replacement_stale_source);
    let replacement_stale_items =
        fixture
            .server
            .snapshot_completion(&fixture.driver, &fixture.snapshots, &uri, eof);
    let replacement_next = replacement_stale_items
        .iter()
        .find(|item| item["label"] == "Next")
        .expect("replacement publication supplies its renamed theorem label");
    assert!(
        replacement_stale_items
            .iter()
            .all(|item| item["label"] != "Main")
    );
    assert_ne!(replacement_next["data"]["identity"], stale_identity);
    assert!(
        replacement_next["detail"]
            .as_str()
            .unwrap()
            .contains(&format!("stale_artifact: Source: {uri} (version 5)"))
    );

    fixture.did_change_text(&uri, 7, format!("{original}\nby "));
    let beyond_original = fixture.server.snapshot_completion(
        &fixture.driver,
        &fixture.snapshots,
        &uri,
        LspPosition {
            line: 1,
            character: 3,
        },
    );
    assert!(
        beyond_original.is_empty(),
        "valid current coordinates outside the retained source cannot supply semantic candidates"
    );

    fixture.did_change_text(&uri, 8, "be ".into());
    let recovered = fixture.server.snapshot_completion(
        &fixture.driver,
        &fixture.snapshots,
        &uri,
        LspPosition {
            line: 0,
            character: 3,
        },
    );
    assert_eq!(labels(&recovered), ["object", "set"]);
    assert!(recovered.iter().all(|item| item["detail"] == "keyword"));

    fixture.server.handle_message(json!({
        "jsonrpc": "2.0", "id": 53, "method": "shutdown", "params": null
    }));
    let terminal_response = fixture
        .server
        .handle_message_with_metadata(
            json!({
                "jsonrpc": "2.0", "id": 54, "method": "textDocument/completion",
                "params": {"textDocument": {"uri": uri}, "position": {
                    "line": eof.line, "character": eof.character
                }}
            }),
            &fixture.driver,
            &fixture.snapshots,
        )
        .unwrap();
    assert_eq!(terminal_response["error"]["code"], -32600);
}

// These fixtures exercise the consumer against genuine resolver publication;
// artifact-only definition-label rows below do not claim native producer coverage.
pub(in crate::server) fn import_fixture(
    text: &str,
    module_path: &str,
    summaries: &[mizar_artifact::module_summary::ModuleSummary],
) -> (
    mizar_frontend::orchestration::FrontendOutput<mizar_syntax::SurfaceAst>,
    Option<mizar_resolve::imports::PublishedImportView>,
) {
    use mizar_artifact::{
        module_summary::MODULE_SUMMARY_SCHEMA_FAMILY, store::artifact_hash_domain,
    };
    use mizar_build::module_index::{ModuleId, PackageIndexSource};
    use mizar_resolve::{
        imports::{ImportPathCandidate, ImportPathResolver},
        module_index::{
            DependencyModuleSummaryRef, ModuleIndexEntry, ModuleIndexInput, ModuleIndexLocation,
            NamespaceIndexEntry, NamespaceRoot, PackageIndexEntry,
            WorkspaceStubModuleIndexProvider,
        },
        module_summary_reuse::{ModuleSummaryReuse, ModuleSummaryReuseRequest},
    };
    use mizar_session::{Edition, ModulePath, PackageId, SourceAnchor};

    let values = summaries
        .iter()
        .map(|summary| {
            (
                ModuleId::new(
                    PackageId::new(&summary.module.package_id),
                    ModulePath::new(&summary.module.module_path),
                ),
                mizar_artifact::module_summary::module_summary_json(summary).unwrap(),
            )
        })
        .collect::<Vec<_>>();
    let domain = artifact_hash_domain(
        MODULE_SUMMARY_SCHEMA_FAMILY,
        mizar_artifact::module_summary::current_schema_version(),
    );
    let references = values
        .iter()
        .map(|(module, value)| DependencyModuleSummaryRef {
            module: module.clone(),
            artifact: format!(
                "{}.{}.summary",
                module.package.as_str(),
                module.path.as_str()
            ),
            content_hash: domain.hash(value, &[]),
        })
        .collect::<Vec<_>>();
    let packages = std::iter::once("app")
        .chain(
            summaries
                .iter()
                .map(|summary| summary.module.package_id.as_str()),
        )
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .map(|package| PackageIndexEntry {
            package_id: PackageId::new(package),
            version: "0.1.0".parse().unwrap(),
            edition: Edition::new("2026"),
            source: PackageIndexSource::Workspace {
                package_root: format!("/workspace/{package}"),
                source_root: format!("/workspace/{package}/src"),
                manifest_path: format!("/workspace/{package}/mizar.toml"),
            },
            dependencies: vec![],
        })
        .collect::<Vec<_>>();
    let namespaces = packages
        .iter()
        .map(|package| NamespaceIndexEntry {
            root: NamespaceRoot::PackageName,
            prefix: vec![package.package_id.as_str().to_owned()],
            package_id: package.package_id.clone(),
        })
        .collect();
    let mut modules = references
        .iter()
        .map(|reference| ModuleIndexEntry {
            module: reference.module.clone(),
            package_id: reference.module.package.clone(),
            module_path: reference.module.path.clone(),
            location: ModuleIndexLocation::DependencySummary {
                artifact: reference.artifact.clone(),
                content_hash: reference.content_hash,
            },
            edition: Edition::new(
                &summaries
                    .iter()
                    .find(|summary| {
                        summary.module.package_id == reference.module.package.as_str()
                            && summary.module.module_path == reference.module.path.as_str()
                    })
                    .unwrap()
                    .module
                    .language_edition,
            ),
        })
        .collect::<Vec<_>>();
    modules.push(ModuleIndexEntry {
        module: ModuleId::new(PackageId::new("app"), ModulePath::new(module_path)),
        package_id: PackageId::new("app"),
        module_path: ModulePath::new(module_path),
        location: ModuleIndexLocation::WorkspaceFile {
            source_root: "/workspace/app/src".into(),
            normalized_path: format!("/workspace/app/src/{module_path}.miz"),
            source_relative_path: format!("{module_path}.miz"),
        },
        edition: Edition::new("2026"),
    });
    let index = WorkspaceStubModuleIndexProvider::new(packages, namespaces, modules, references);
    struct Dependencies<'a> {
        index: &'a WorkspaceStubModuleIndexProvider,
        values: &'a [(ModuleId, mizar_artifact::store::CanonicalJson)],
        current: mizar_resolve::resolved_ast::ModuleId,
    }
    impl mizar_frontend::lexical_env::LexicalSummaryProvider for Dependencies<'_> {
        fn resolve_imports(
            &self,
            request: &mizar_frontend::lexical_env::LexicalEnvironmentRequest<'_>,
        ) -> Result<
            mizar_frontend::lexical_env::ResolvedImports,
            mizar_frontend::lexical_env::FrontendLexicalEnvironmentError,
        > {
            use mizar_frontend::lexical_env::{
                ResolvedImport, ResolvedImportEntry, ResolvedImports,
            };
            let index = ModuleIndexInput::new(self.index);
            let mut imports = Vec::new();
            let mut summaries = std::collections::BTreeMap::new();
            for candidate in ImportPathCandidate::from_frontend_imports(request).unwrap() {
                let resolution = ImportPathResolver::new(index)
                    .resolve(&self.current, std::slice::from_ref(&candidate));
                let Some(binding) = resolution.resolved().first() else {
                    continue;
                };
                let module = ModuleId::new(
                    binding.target().package().clone(),
                    binding.target().path().clone(),
                );
                let Some((_, value)) = self.values.iter().find(|(id, _)| id == &module) else {
                    continue;
                };
                let Some(summary) = ModuleSummaryReuse::new(index).read_lexical_summary(
                    ModuleSummaryReuseRequest::new(&module, SourceAnchor::Range(candidate.range())),
                    value,
                ) else {
                    continue;
                };
                imports.push(ResolvedImportEntry {
                    stub_ordinal: candidate.ordinal(),
                    stub_span: candidate.range(),
                    import: ResolvedImport {
                        module_id: summary.module_id.clone(),
                    },
                });
                summaries.insert(summary.module_id.clone(), summary);
            }
            Ok(ResolvedImports {
                imports,
                summaries: summaries.into_values().collect(),
                diagnostics: vec![],
            })
        }
    }
    let root = std::env::temp_dir().join(format!(
        "mizar-completion-import-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let path = root.join(format!("src/{module_path}.miz"));
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, text).unwrap();
    let output = mizar_frontend::orchestration::Frontend::new(
        mizar_frontend::source::FrontendSourceLoader::new(mizar_session::DiskSourceLoader::new(
            &root,
        )),
        Dependencies {
            index: &index,
            values: &values,
            current: mizar_resolve::resolved_ast::ModuleId::new(
                PackageId::new("app"),
                ModulePath::new(module_path),
            ),
        },
        mizar_frontend::parsing::MizarParserSeam,
    )
    .run(
        mizar_frontend::source::SourceUnitRequest {
            snapshot: mizar_session::BuildSnapshotId::from_published_schema_str(&format!(
                "mizar-session-build-snapshot-v1:{}",
                "39".repeat(32)
            ))
            .unwrap(),
            input: mizar_session::SourceInput {
                package_id: PackageId::new("app"),
                module_path: ModulePath::new(module_path),
                normalized_path: mizar_session::normalize_path(&root, &path).unwrap(),
                edition: Edition::new("2026"),
                origin: mizar_session::SourceOriginInput::Disk { path },
            },
        },
        &mizar_session::InMemorySessionIdAllocator::new(),
    )
    .unwrap();
    std::fs::remove_dir_all(root).unwrap();
    let view = ImportPathResolver::new(ModuleIndexInput::new(&index)).publish_frontend(
        &output,
        &output.source,
        |module, _| {
            values
                .iter()
                .find(|(id, _)| id == module)
                .map(|(_, value)| value.clone())
        },
    );
    (output, view)
}

#[test]
fn resolver_import_aliases_and_facades_preserve_public_spelling_and_identity() {
    use super::append_semantic_items;
    let (_, summary, _) = genuine_notation_fixture(false);
    let mut summary = summary.expect("public notation product");
    let (_, opaque) = import_fixture(
        "import app.main as Grp;\ndefinition\nend;",
        "client",
        std::slice::from_ref(&summary),
    );
    assert!(
        opaque.is_none(),
        "opaque source FQNs cannot manufacture import access paths"
    );
    // Readable access names are explicit artifact fixture inputs. The source
    // producer above remains refused; completion never performs this mapping.
    for row in &mut summary.exported_symbols {
        let contribution = summary
            .lexical_summary
            .contributions
            .iter()
            .find(|shape| shape.key == row.origin_id)
            .unwrap();
        let shape =
            mizar_lexer::ExportedSymbolShape::from_canonical_bytes(contribution.payload.as_bytes())
                .unwrap();
        if row.declaration_kind == "mode" {
            row.fully_qualified_name = format!("app::main::Inner::{}", shape.spelling);
            row.namespace_path.push("Inner".into());
        } else {
            row.fully_qualified_name = format!("app::main::{}", shape.spelling);
        }
    }
    summary.refresh_interface_hash().unwrap();
    let (output, view) = import_fixture(
        "import app.main as Grp, app.main as Grp;\ndefinition\nend;",
        "client",
        std::slice::from_ref(&summary),
    );
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    let view = view.unwrap_or_else(|| {
        panic!(
            "import product unavailable: {:?}",
            summary
                .exported_symbols
                .iter()
                .map(|row| (
                    &row.fully_qualified_name,
                    &row.namespace_path,
                    &row.declaration_kind
                ))
                .collect::<Vec<_>>()
        )
    });
    assert_eq!(view.resolution().resolved().len(), 2);
    let complete = |text: &str, view: &mizar_resolve::imports::PublishedImportView| {
        let context = at_end(text).unwrap();
        let mut items = syntax_items(&context);
        append_semantic_items(
            &mut items,
            &summary,
            None,
            Some(view),
            (0, 0),
            &context,
            "Source: imports (version 1)",
        );
        sort_items(items, &context.1)
    };
    assert_eq!(labels(&complete("be Gr", &view)), ["Grp"]);
    assert_eq!(labels(&complete("be Grp.", &view)), ["Inner"]);
    assert_eq!(labels(&complete("be Grp.Inner.", &view)), ["MyMode"]);
    assert_eq!(
        labels(&complete("Grp.", &view)),
        ["+", "Inner", "base", "related"]
    );
    let terms = complete("= Grp.", &view);
    assert!(labels(&terms).contains(&"base"));
    assert!(!labels(&terms).contains(&"HiddenMode"));
    let base = terms.iter().find(|item| item["label"] == "base").unwrap();
    let owner = summary
        .exported_symbols
        .iter()
        .find(|row| row.fully_qualified_name.ends_with("base"))
        .unwrap();
    assert!(
        base["data"]["identity"]
            .as_str()
            .unwrap()
            .contains(&owner.origin_id)
    );
    assert_eq!(base["data"]["accessPath"], "Grp.base");

    let (local, local_imports) = import_fixture(
        "import app.main as Grp;\ndefinition\nlet x, y be set;\nfunc LocalPlus: +(x,y) -> set equals x;\ninfix_operator(\"+\", right, 70);\nend;",
        "client",
        std::slice::from_ref(&summary),
    );
    assert!(local.diagnostics.is_empty(), "{:?}", local.diagnostics);
    let module = mizar_resolve::resolved_ast::ModuleId::new(
        local.source.package_id.clone(),
        local.source.module_path.clone(),
    );
    let ast = local.ast.as_ref().unwrap();
    let shells =
        mizar_resolve::declarations::DeclarationShellCollector::new(ast, &module).collect();
    let collection = mizar_resolve::symbols::SignatureProjectionExtractor::new(
        ast,
        &shells,
        mizar_resolve::env::NamespacePath::new("client"),
    )
    .collect(&module);
    let local_notation = collection.source_notation_view(&local).unwrap();
    let event = local_notation.operator_events().last().unwrap();
    let activation = event.activation().primary.start;
    assert!(
        event.activation().original_input.is_none(),
        "identity loading uses native coordinates"
    );
    let imported_operator = summary
        .lexical_summary
        .contributions
        .iter()
        .filter_map(|entry| {
            mizar_lexer::ExportedSymbolShape::from_canonical_bytes(entry.payload.as_bytes())
        })
        .find(|shape| shape.spelling == "+")
        .unwrap()
        .operator;
    let context = at_end("= Grp.").unwrap();
    let mut identities = Vec::new();
    for (offset, operator) in [
        (activation - 1, imported_operator),
        (activation, event.declaration().operator),
    ] {
        let mut items = Vec::new();
        append_semantic_items(
            &mut items,
            &summary,
            Some(&local_notation),
            local_imports.as_ref(),
            (offset, offset),
            &context,
            "Source: client (version 1)",
        );
        let plus = items.iter().find(|item| item["label"] == "+").unwrap();
        let detail = plus["detail"].as_str().unwrap();
        assert!(detail.contains(&format!("operator {operator:?}")));
        assert!(detail.contains(&format!(
            "arity {:?}",
            mizar_lexer::UserSymbolArity::exact(2)
        )));
        identities.push(plus["data"]["identity"].clone());
    }
    assert_eq!(identities[0], identities[1]);
    let mut local_items = Vec::new();
    append_semantic_items(
        &mut local_items,
        &summary,
        Some(&local_notation),
        local_imports.as_ref(),
        (activation, activation),
        &at_end("= ").unwrap(),
        "Source: client (version 1)",
    );
    let local_plus = local_items
        .iter()
        .find(|item| item["data"]["accessPath"] == "+")
        .unwrap();
    assert!(
        local_plus["detail"]
            .as_str()
            .unwrap()
            .contains(&format!("operator {:?}", event.declaration().operator))
    );

    let (facade, forwarding) = import_fixture(
        "import app.main as Inner;\nexport Inner;\ndefinition\nend;",
        "facade",
        std::slice::from_ref(&summary),
    );
    assert!(facade.diagnostics.is_empty(), "{:?}", facade.diagnostics);
    let module = mizar_resolve::resolved_ast::ModuleId::new(
        facade.source.package_id.clone(),
        facade.source.module_path.clone(),
    );
    let ast = facade.ast.as_ref().unwrap();
    let shells =
        mizar_resolve::declarations::DeclarationShellCollector::new(ast, &module).collect();
    let collection = mizar_resolve::symbols::SignatureProjectionExtractor::new(
        ast,
        &shells,
        mizar_resolve::env::NamespacePath::new("facade"),
    )
    .collect(&module);
    let mut identity = summary.module.clone();
    identity.module_path = "facade".into();
    let facade_summary = collection
        .source_notation_module_summary_candidate(&facade, &identity)
        .unwrap()
        .unwrap();
    let facade_summary = forwarding.unwrap().compose_summary(facade_summary).unwrap();
    let (_, via_facade) = import_fixture(
        "import app.facade as Grp;\ndefinition\nend;",
        "client",
        &[summary.clone(), facade_summary.clone()],
    );
    assert_eq!(complete("= Grp.", &via_facade.unwrap()), terms);
    let (_, missing) = import_fixture(
        "import app.facade as Grp;\ndefinition\nend;",
        "client",
        std::slice::from_ref(&facade_summary),
    );
    assert!(
        missing.is_none(),
        "missing forwarding dependency cannot be a complete empty closure"
    );
    let (_, conflict) = import_fixture(
        "import app.main as Grp, app.facade as Grp;\ndefinition\nend;",
        "client",
        &[summary.clone(), facade_summary],
    );
    assert!(conflict.is_none_or(|view| view.visible_surfaces().is_none()));
}

#[test]
fn citations_exclude_active_local_notation_and_proof_keywords() {
    use super::append_semantic_items;

    let (source, summary, notation) = genuine_notation_fixture(false);
    let mut summary = summary.expect("genuine source notation summary candidate");
    let owner = summary.exported_symbols[0].clone();
    let mut theorem = owner.clone();
    theorem.origin_id = "artifact-theorem-owner".into();
    theorem.fully_qualified_name = "app::main::TheoremT".into();
    theorem.declaration_kind = "theorem".into();
    let mut definition = owner.clone();
    definition.origin_id = "artifact-definition-owner".into();
    definition.fully_qualified_name = "app::main::DefinitionD".into();
    definition.declaration_kind = "functor".into();
    summary
        .exported_symbols
        .extend([theorem.clone(), definition.clone()]);
    summary.exported_labels = vec![
        mizar_artifact::module_summary::ExportedLabelSummary {
            origin_id: "artifact-theorem-label".into(),
            label: "T".into(),
            owner_fully_qualified_name: theorem.fully_qualified_name.clone(),
            visibility: "public".into(),
            source_range: theorem.source_range,
            target_kind: "theorem".into(),
        },
        mizar_artifact::module_summary::ExportedLabelSummary {
            origin_id: "artifact-definition-label".into(),
            label: "D".into(),
            owner_fully_qualified_name: definition.fully_qualified_name.clone(),
            visibility: "public".into(),
            source_range: definition.source_range,
            target_kind: "definition".into(),
        },
    ];
    summary.refresh_interface_hash().unwrap();

    let complete = |text: &str| {
        let context = at_end(text).unwrap();
        let mut items = syntax_items(&context);
        append_semantic_items(
            &mut items,
            &summary,
            Some(&notation),
            None,
            (source.len(), source.len()),
            &context,
            "Source: genuine local (version 1)",
        );
        sort_items(items, &context.1)
    };

    let terms = complete("x = ");
    assert!(
        labels(&terms).contains(&"base"),
        "active functor missing: {terms:?}"
    );
    let formulas = complete("x holds ");
    assert!(
        labels(&formulas).contains(&"related"),
        "active predicate missing: {formulas:?}"
    );

    for cue in ["by ", "by T, "] {
        assert_eq!(
            labels(&complete(cue)),
            ["D", "T"],
            "citation candidates for {cue:?} must exclude active local notation and proof keywords"
        );
    }
}

#[test]
fn imported_citations_keep_theorem_definition_signatures_and_exclude_unpublished_labels() {
    use super::append_semantic_items;
    let mut fixture = super::super::full_snapshot_registry_a::RegistryFixture::new();
    assert!(fixture.build().1);
    let (snapshot, _) = fixture
        .server
        .lsp_snapshot(&fixture.driver, &fixture.snapshots)
        .unwrap();
    let metadata = snapshot
        .publication()
        .source_metadata()
        .iter()
        .flatten()
        .find(|metadata| metadata.summary().module.module_path == "main")
        .unwrap();
    let mut summary = metadata.summary().clone();
    let (_, opaque) = import_fixture(
        "import alpha.main as Grp;\ndefinition\nend;",
        "client",
        std::slice::from_ref(&summary),
    );
    assert!(
        opaque.is_none(),
        "opaque source-owned theorem paths stay unavailable"
    );
    summary.exported_symbols[0].fully_qualified_name = "alpha::main::Main".into();
    summary.exported_labels[0].owner_fully_qualified_name = "alpha::main::Main".into();
    let owner = summary.exported_symbols.first().unwrap().clone();
    let mut definition = owner.clone();
    definition.origin_id = "definition-artifact-owner".into();
    definition.fully_qualified_name = "alpha::main::Defined".into();
    definition.declaration_kind = "functor".into();
    definition.rendered_signature = "published definition signature".into();
    definition.proof_status = Some(mizar_artifact::module_summary::ProofStatusSummary::NotRequired);
    summary.exported_symbols.push(definition.clone());
    summary
        .exported_labels
        .push(mizar_artifact::module_summary::ExportedLabelSummary {
            origin_id: "definition-label-artifact-fixture".into(),
            label: "D".into(),
            owner_fully_qualified_name: definition.fully_qualified_name.clone(),
            visibility: "public".into(),
            source_range: owner.source_range,
            target_kind: "definition".into(),
        });
    summary.refresh_interface_hash().unwrap();
    let (client, imports) = import_fixture(
        concat!(
            "import alpha.main as Grp;\n",
            "theorem Client: for X being set holds X = X proof ",
            "let X be set; L: X = X; thus X = X by Grp.Main, Grp.Main; end;",
        ),
        "client",
        std::slice::from_ref(&summary),
    );
    assert!(client.diagnostics.is_empty(), "{:?}", client.diagnostics);
    let imports = imports.unwrap_or_else(|| {
        panic!(
            "label import unavailable identity {:?}, symbols {:?}, labels {:?}",
            summary.module,
            summary
                .exported_symbols
                .iter()
                .map(|row| (
                    &row.fully_qualified_name,
                    &row.namespace_path,
                    &row.declaration_kind
                ))
                .collect::<Vec<_>>(),
            summary.exported_labels
        )
    });
    for cue in ["by Grp.", "by Grp.Main, Grp."] {
        let offset = client.source.source_text.find(cue).unwrap() + cue.len();
        let context = completion_context(
            &client.source.source_text,
            position(&client.source.source_text, offset),
        )
        .unwrap();
        let mut items = syntax_items(&context);
        append_semantic_items(
            &mut items,
            &summary,
            None,
            Some(&imports),
            (offset, offset),
            &context,
            "Source: imports (version 1)",
        );
        let items = sort_items(items, &context.1);
        assert_eq!(labels(&items), ["D", "Main"]);
        for item in items {
            let detail = item["detail"].as_str().unwrap();
            let expected_owner = if item["label"] == "D" {
                &definition
            } else {
                &owner
            };
            assert!(detail.contains(&expected_owner.rendered_signature));
            assert!(detail.contains(&format!("{:?}", expected_owner.proof_status)));
            assert!(item.get("textEdit").is_none());
        }
    }
    let context = at_end("by ").unwrap();
    let mut items = Vec::new();
    append_semantic_items(
        &mut items,
        &summary,
        None,
        None,
        (0, 0),
        &context,
        "Source: local (version 1)",
    );
    assert!(
        items.is_empty(),
        "self/future owner labels are unavailable before the complete declaration"
    );
    let end = owner.source_range.end_byte as usize;
    append_semantic_items(
        &mut items,
        &summary,
        None,
        None,
        (end, end),
        &context,
        "Source: local (version 1)",
    );
    assert_eq!(labels(&sort_items(items, "")), ["D", "Main"]);
}
