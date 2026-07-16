    #[test]
    fn parse_only_provider_resolves_every_stub_and_deduplicates_fixture_summaries() {
        let source_id = source_id(90);
        let stubs = vec![
            import_stub(source_id, "alpha", 0, 5),
            import_stub(source_id, "alpha", 7, 12),
            import_stub(source_id, "parser.type_fixtures", 14, 34),
        ];
        let request = LexicalEnvironmentRequest {
            source_id,
            import_stubs: &stubs,
            edition: Edition::new("2026"),
        };

        let resolved = ParseOnlyImportProvider
            .resolve_imports(&request)
            .expect("parse-only provider should not fail");

        assert_eq!(resolved.imports.len(), 3);
        assert_eq!(
            resolved
                .imports
                .iter()
                .map(|entry| (
                    entry.stub_ordinal,
                    entry.stub_span,
                    entry.import.module_id.as_str()
                ))
                .collect::<Vec<_>>(),
            vec![
                (0, range(source_id, 0, 5), "alpha"),
                (1, range(source_id, 7, 12), "alpha"),
                (2, range(source_id, 14, 34), "parser.type_fixtures"),
            ]
        );
        assert_eq!(resolved.summaries.len(), 2);
        assert_eq!(
            resolved
                .summaries
                .iter()
                .map(|summary| (
                    summary.module_id.as_str(),
                    summary.exported_symbols.len(),
                    summary.fingerprint.get()
                ))
                .collect::<Vec<_>>(),
            vec![("alpha", 0, 1), ("parser.type_fixtures", 15, 3)]
        );
        assert_eq!(
            resolved.summaries[1]
                .exported_symbols
                .iter()
                .map(|symbol| (symbol.spelling.as_str(), symbol.kind, symbol.operator))
                .collect::<Vec<_>>(),
            vec![
                ("empty", UserSymbolKind::Attribute, None),
                ("T", UserSymbolKind::Mode, None),
                ("R", UserSymbolKind::Structure, None),
                ("TypeCaseAttr", UserSymbolKind::Attribute, None),
                ("TypeCaseMode", UserSymbolKind::Mode, None),
                ("TypeCaseStruct", UserSymbolKind::Structure, None),
                ("divides", UserSymbolKind::Predicate, None),
                ("<=", UserSymbolKind::Predicate, None),
                (
                    "~",
                    UserSymbolKind::Functor,
                    Some(ExportedOperatorMetadata {
                        fixity: ExportedOperatorFixity::Prefix,
                        precedence: 70,
                    }),
                ),
                (
                    "!",
                    UserSymbolKind::Functor,
                    Some(ExportedOperatorMetadata {
                        fixity: ExportedOperatorFixity::Postfix,
                        precedence: 90,
                    }),
                ),
                ("|.", UserSymbolKind::Functor, None),
                (".|", UserSymbolKind::Functor, None),
                (
                    "++",
                    UserSymbolKind::Functor,
                    Some(ExportedOperatorMetadata {
                        fixity: ExportedOperatorFixity::Infix(ExportedOperatorAssociativity::Left,),
                        precedence: 10,
                    }),
                ),
                (
                    "**",
                    UserSymbolKind::Functor,
                    Some(ExportedOperatorMetadata {
                        fixity: ExportedOperatorFixity::Infix(ExportedOperatorAssociativity::Right,),
                        precedence: 20,
                    }),
                ),
                (
                    "%%",
                    UserSymbolKind::Functor,
                    Some(ExportedOperatorMetadata {
                        fixity: ExportedOperatorFixity::Infix(
                            ExportedOperatorAssociativity::NonAssociative,
                        ),
                        precedence: 10,
                    }),
                ),
            ]
        );
        assert!(resolved.diagnostics.is_empty());
    }
