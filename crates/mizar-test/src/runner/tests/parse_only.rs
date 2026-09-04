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
        assert_eq!(
            resolved.summaries[1]
                .exported_symbols
                .iter()
                .map(|symbol| (
                    symbol.symbol_id.as_str(),
                    symbol.source_module.as_str(),
                    symbol.export_rank.get(),
                    symbol.arity.minimum,
                    symbol.arity.maximum,
                ))
                .collect::<Vec<_>>(),
            vec![
                ("parser.type_fixtures#parse-only#empty", "parser.type_fixtures", 0, 0, Some(0)),
                ("parser.type_fixtures#parse-only#T", "parser.type_fixtures", 1, 0, None),
                ("parser.type_fixtures#parse-only#R", "parser.type_fixtures", 2, 0, None),
                ("parser.type_fixtures#parse-only#TypeCaseAttr", "parser.type_fixtures", 3, 0, Some(0)),
                ("parser.type_fixtures#parse-only#TypeCaseMode", "parser.type_fixtures", 4, 0, None),
                ("parser.type_fixtures#parse-only#TypeCaseStruct", "parser.type_fixtures", 5, 0, None),
                ("parser.type_fixtures#parse-only#divides", "parser.type_fixtures", 6, 2, Some(2)),
                ("parser.type_fixtures#parse-only#<=", "parser.type_fixtures", 7, 2, Some(2)),
                ("parser.type_fixtures#parse-only#~", "parser.type_fixtures", 8, 1, Some(1)),
                ("parser.type_fixtures#parse-only#!", "parser.type_fixtures", 9, 1, Some(1)),
                ("parser.type_fixtures#parse-only#|.", "parser.type_fixtures", 10, 1, Some(1)),
                ("parser.type_fixtures#parse-only#.|", "parser.type_fixtures", 11, 1, Some(1)),
                ("parser.type_fixtures#parse-only#++", "parser.type_fixtures", 12, 2, Some(2)),
                ("parser.type_fixtures#parse-only#**", "parser.type_fixtures", 13, 2, Some(2)),
                ("parser.type_fixtures#parse-only#%%", "parser.type_fixtures", 14, 2, Some(2)),
            ]
        );
        assert!(resolved.diagnostics.is_empty());
    }

    const TASK257C4C1_FIXTURE_SOURCE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/testdata/parser/nested_capture_fixtures.miz"
    ));
    const TASK257C4C1_CANONICAL_SOURCE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/miz/pass/types/",
        "pass_types_nested_comprehension_outer_generator_capture_001.miz"
    ));
    const TASK257C4C1_HISTORICAL_SOURCE: &str = "definition\n  func NestedCapture -> set equals\n    { { x where y is Element of NAT }\n      where x is Element of NAT };\nend;\n";

    fn task257c4c1_frontend_output(
        source: &str,
        ordinal: usize,
    ) -> mizar_frontend::orchestration::FrontendOutput<SurfaceAst> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |duration| duration.as_nanos());
        let package_root = std::env::temp_dir().join(format!(
            "mizar-test-task257c4c1-{}-{ordinal}-{unique}",
            std::process::id()
        ));
        let source_path = package_root.join("src").join("nested_capture.miz");
        std::fs::create_dir_all(source_path.parent().expect("C4C1 source parent"))
            .expect("create C4C1 package");
        std::fs::write(&source_path, source).expect("write C4C1 source");
        let package = PackageId::new("mizar-test-task257c4c1");
        let module_path = ModulePath::new(format!("tests.nested_capture_{ordinal}"));
        let normalized_path = mizar_session::normalize_path(&package_root, &source_path)
            .expect("normalize C4C1 source path");
        let frontend = mizar_frontend::orchestration::Frontend::new(
            mizar_frontend::source::FrontendSourceLoader::new(
                mizar_session::DiskSourceLoader::new(&package_root),
            ),
            ParseOnlyImportProvider,
            mizar_frontend::parsing::MizarParserSeam,
        );
        let output = frontend
            .run(
                mizar_frontend::source::SourceUnitRequest {
                    snapshot: super::shared::snapshot_id(25_700 + ordinal),
                    input: mizar_session::SourceInput {
                        package_id: package,
                        module_path,
                        normalized_path,
                        edition: Edition::new("2026"),
                        origin: mizar_session::SourceOriginInput::Disk {
                            path: source_path,
                        },
                    },
                },
                &InMemorySessionIdAllocator::new(),
            )
            .expect("C4C1 frontend should run");
        std::fs::remove_dir_all(&package_root).expect("clean C4C1 package");
        output
    }

    fn task257c4c1_token_ranges(ast: &SurfaceAst, spelling: &str) -> Vec<(usize, usize)> {
        ast.token_views()
            .filter(|view| {
                view.as_token()
                    .is_some_and(|token| token.text.as_ref() == spelling)
            })
            .map(|view| (view.range().start, view.range().end))
            .collect()
    }

    #[test]
    fn task257c4c1_physical_fixture_declarations_are_exact() {
        assert_eq!(TASK257C4C1_FIXTURE_SOURCE.len(), 140);
        assert_eq!(
            sha256_text(TASK257C4C1_FIXTURE_SOURCE),
            "dd721a48620f985d5612cc718a94aef576e87d616c239712b8deb2d65c84a11c"
        );

        let output = task257c4c1_frontend_output(TASK257C4C1_FIXTURE_SOURCE, 0);
        assert!(output.diagnostics.is_empty());
        let ast = output.ast.expect("C4C1 fixture AST");
        assert!(ast.nodes().iter().all(|node| !node.recovered));
        assert_eq!(task257c4c1_token_ranges(&ast, "S"), vec![(17, 18), (64, 65)]);
        assert_eq!(task257c4c1_token_ranges(&ast, "ElementDef"), vec![(41, 51)]);
        assert_eq!(task257c4c1_token_ranges(&ast, "Element"), vec![(53, 60)]);
        assert_eq!(task257c4c1_token_ranges(&ast, "NatDef"), vec![(105, 111)]);
        assert_eq!(task257c4c1_token_ranges(&ast, "NAT"), vec![(113, 116)]);

        let module = ResolverModuleId::new(output.source.package_id, output.source.module_path);
        let shells = mizar_resolve::declarations::DeclarationShellCollector::new(&ast, &module)
            .collect();
        let definitions = shells
            .declarations()
            .iter()
            .filter(|shell| {
                matches!(
                    shell.kind(),
                    mizar_resolve::declarations::DeclarationShellKind::ModeDefinition
                        | mizar_resolve::declarations::DeclarationShellKind::FunctorDefinition
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(definitions.len(), 2);
        assert_eq!(
            definitions
                .iter()
                .map(|shell| shell.kind())
                .collect::<Vec<_>>(),
            vec![
                mizar_resolve::declarations::DeclarationShellKind::ModeDefinition,
                mizar_resolve::declarations::DeclarationShellKind::FunctorDefinition,
            ]
        );
        assert!(definitions.iter().all(|shell| {
            shell.visibility().state()
                == mizar_resolve::declarations::DeclarationShellVisibilityState::Public
                && !shell.recovered()
        }));
    }

    #[test]
    fn task257c4c1_provider_summary_is_exact_and_unrelated_modules_are_isolated() {
        let source_id = source_id(241);
        let target_stub = import_stub(
            source_id,
            "parser.nested_capture_fixtures",
            7,
            37,
        );
        let resolved = ParseOnlyImportProvider
            .resolve_imports(&LexicalEnvironmentRequest {
                source_id,
                import_stubs: std::slice::from_ref(&target_stub),
                edition: Edition::new("2026"),
            })
            .expect("C4C1 provider should resolve the exact logical module");
        assert_eq!(resolved.imports.len(), 1);
        assert_eq!(resolved.imports[0].stub_ordinal, 0);
        assert_eq!(resolved.imports[0].stub_span, range(source_id, 7, 37));
        assert_eq!(
            resolved.imports[0].import.module_id.as_str(),
            "parser.nested_capture_fixtures"
        );
        assert_eq!(resolved.summaries.len(), 1);
        let summary = &resolved.summaries[0];
        assert_eq!(summary.module_id.as_str(), "parser.nested_capture_fixtures");
        assert_eq!(summary.fingerprint.get(), 1);
        assert_eq!(
            summary
                .exported_symbols
                .iter()
                .map(|symbol| (
                    symbol.spelling.as_str(),
                    symbol.symbol_id.as_str(),
                    symbol.source_module.as_str(),
                    symbol.export_rank.get(),
                    symbol.kind,
                    symbol.arity.minimum,
                    symbol.arity.maximum,
                    symbol.operator,
                ))
                .collect::<Vec<_>>(),
            vec![
                (
                    "Element",
                    "parser.nested_capture_fixtures#parse-only#Element",
                    "parser.nested_capture_fixtures",
                    0,
                    UserSymbolKind::Mode,
                    1,
                    Some(1),
                    None,
                ),
                (
                    "NAT",
                    "parser.nested_capture_fixtures#parse-only#NAT",
                    "parser.nested_capture_fixtures",
                    1,
                    UserSymbolKind::Functor,
                    0,
                    Some(0),
                    None,
                ),
            ]
        );
        assert!(resolved.diagnostics.is_empty());

        let unrelated_stub = import_stub(source_id, "parser.unrelated", 40, 56);
        let unrelated = ParseOnlyImportProvider
            .resolve_imports(&LexicalEnvironmentRequest {
                source_id,
                import_stubs: std::slice::from_ref(&unrelated_stub),
                edition: Edition::new("2026"),
            })
            .expect("unrelated C4C1 module should remain a valid empty summary");
        assert_eq!(unrelated.imports.len(), 1);
        assert_eq!(unrelated.summaries.len(), 1);
        assert_eq!(unrelated.summaries[0].module_id.as_str(), "parser.unrelated");
        assert!(unrelated.summaries[0].exported_symbols.is_empty());
        assert!(unrelated.diagnostics.is_empty());

        let (ast, module, _, symbols) =
            task253_ast_from_source_text(TASK257C4C1_CANONICAL_SOURCE, 25_741);
        let augmented = augment_type_elaboration_import_summaries(&ast, &module, symbols.clone());
        assert_eq!(augmented, symbols);
    }

    #[test]
    fn task257c4c1_canonical_imported_source_is_zero_diagnostic_with_exact_frontend_provenance() {
        assert_eq!(TASK257C4C1_CANONICAL_SOURCE.len(), 164);
        assert_eq!(
            sha256_text(TASK257C4C1_CANONICAL_SOURCE),
            "c3b8bd62c16406ccedee2e64a71ef62a5c4b329d2319be33ad3834a9541af431"
        );

        let output = task257c4c1_frontend_output(TASK257C4C1_CANONICAL_SOURCE, 1);
        assert!(output.diagnostics.is_empty());
        assert_eq!(output.preprocessed.import_stubs.len(), 1);
        let stub = &output.preprocessed.import_stubs[0];
        let source_id = output.source.source_id;
        assert_eq!(stub.span, range(source_id, 7, 37));
        assert_eq!(stub.path.spelling.as_ref(), "parser.nested_capture_fixtures");
        assert_eq!(stub.path.span, range(source_id, 7, 37));
        assert_eq!(stub.path.relative, None);
        assert_eq!(stub.alias, None);
        assert_eq!(
            stub.path
                .components
                .iter()
                .map(AsRef::as_ref)
                .collect::<Vec<&str>>(),
            vec!["parser", "nested_capture_fixtures"]
        );
        assert_eq!(
            stub.path.source_segments,
            vec![range(source_id, 7, 37)]
        );

        let ast = output.ast.expect("C4C1 canonical AST");
        assert!(ast.nodes().iter().all(|node| !node.recovered));
        assert_eq!(
            ast.nodes()
                .iter()
                .filter(|node| matches!(node.kind, SurfaceNodeKind::ImportAliasDecl))
                .count(),
            1
        );
        let import_decl_view = ast
            .node_views()
            .find(|view| matches!(view.kind(), SurfaceNodeKind::ImportAliasDecl))
            .expect("C4C1 import declaration");
        let import_decl = ast
            .node(import_decl_view.id())
            .expect("C4C1 import declaration node");
        assert_eq!(import_decl.range, range(source_id, 7, 37));
        let module_path = import_decl
            .children
            .iter()
            .filter_map(|child| ast.node(*child))
            .find(|node| matches!(node.kind, SurfaceNodeKind::ModulePath))
            .expect("C4C1 import module path");
        assert_eq!(module_path.range, range(source_id, 7, 37));
        let import_item = ast
            .nodes()
            .iter()
            .find(|node| {
                matches!(node.kind, SurfaceNodeKind::ImportItem)
                    && node.children.contains(&import_decl_view.id())
            })
            .expect("C4C1 surrounding import item");
        assert_eq!(import_item.range, range(source_id, 0, 38));
        let path_segments = module_path
            .children
            .iter()
            .filter_map(|child| ast.node(*child))
            .filter(|node| matches!(node.kind, SurfaceNodeKind::PathSegment))
            .map(|node| (node.range.start, node.range.end))
            .collect::<Vec<_>>();
        assert_eq!(path_segments, vec![(7, 13), (14, 37)]);
        assert_eq!(task257c4c1_token_ranges(&ast, "NestedCapture"), vec![(58, 71)]);
        assert_eq!(task257c4c1_token_ranges(&ast, "x"), vec![(94, 95), (136, 137)]);
        assert_eq!(task257c4c1_token_ranges(&ast, "y"), vec![(102, 103)]);
        assert_eq!(task257c4c1_token_ranges(&ast, "Element"), vec![(107, 114), (141, 148)]);
        assert_eq!(task257c4c1_token_ranges(&ast, "NAT"), vec![(118, 121), (152, 155)]);
    }

    #[test]
    fn task257c4c1_historical_no_import_source_retains_six_diagnostics_without_leakage() {
        assert_eq!(TASK257C4C1_HISTORICAL_SOURCE.len(), 124);
        assert_eq!(
            sha256_text(TASK257C4C1_HISTORICAL_SOURCE),
            "f1a35d2d7f6cb4a57ece3b1143a68c1a01ab9ac478960862057025cc9838cea7"
        );

        let output = task257c4c1_frontend_output(TASK257C4C1_HISTORICAL_SOURCE, 2);
        assert!(output.preprocessed.import_stubs.is_empty());
        assert_eq!(output.diagnostics.len(), 6);
        assert_eq!(
            output.diagnostics[0].location,
            mizar_frontend::orchestration::DiagnosticLocation::SourceRange(range(
                output.source.source_id,
                67,
                74,
            ))
        );
        let ast = output.ast.expect("historical C4C0 AST");
        assert_eq!(
            ast.nodes()
                .iter()
                .filter(|node| matches!(node.kind, SurfaceNodeKind::ImportAliasDecl))
                .count(),
            0
        );
    }

    #[test]
    fn step5c3_parse_shape_rejects_token_recovery_and_ast_drift() {
        let source = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/miz/fail/attributes/",
            "fail_type_elaboration_attr_param_prefix_unbound_001.miz"
        ));
        let (ast, _, _, _, _) =
            task253_ast_from_source_text_with_diagnostic_count(source, 503_200);
        assert!(super::parse_only::step5c3_parse_shape(&ast));

        for (ordinal, mutated) in [
            source.replacen("k-scaled", "q-scaled", 1),
            source.replacen("let X be set;", "let k be object;\n  let X be set;", 1),
        ]
        .into_iter()
        .enumerate()
        {
            let (ast, _, _, _, _) =
                task253_ast_from_source_text_with_diagnostic_count(&mutated, 503_201 + ordinal);
            assert!(!super::parse_only::step5c3_parse_shape(&ast));
        }

        let mut builder = SurfaceAstBuilder::new(ast.source_id);
        let mut rebuilt = Vec::with_capacity(ast.nodes().len());
        for node in ast.nodes() {
            let children = node
                .children
                .iter()
                .map(|child| rebuilt[child.index()])
                .collect();
            let id = match &node.kind {
                SurfaceNodeKind::Token(token) if node.recovered => {
                    builder.add_recovered_token(token.kind, token.text.as_ref(), node.range)
                }
                SurfaceNodeKind::Token(token) => {
                    builder.add_token(token.kind, token.text.as_ref(), node.range)
                }
                SurfaceNodeKind::ErrorRecovery(kind) => {
                    builder.add_recovery(*kind, node.range, children)
                }
                SurfaceNodeKind::AttributeDefinition => builder.add_node(
                    SurfaceNodeKind::PredicateDefinition,
                    node.range,
                    children,
                ),
                kind => builder.add_node(kind.clone(), node.range, children),
            };
            rebuilt.push(id);
        }
        let mutated = builder.finish(
            ast.root().map(|node| rebuilt[node.index()]),
            ast.expression_root().map(|node| rebuilt[node.index()]),
        );
        assert_eq!(mutated.token_texts(), ast.token_texts());
        assert!(!super::parse_only::step5c3_parse_shape(&mutated));
    }

    #[test]
    fn step5c4_parse_shape_rejects_source_and_ast_semantic_drift() {
        let source = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/miz/fail/modes/",
            "fail_parse_only_mode_property_impl_missing_correctness_001.miz"
        ));
        let (ast, _, _, _, _) =
            task253_ast_from_source_text_with_diagnostic_count(source, 503_300);
        assert!(super::parse_only::step5c4_parse_shape(&ast));
        for mutated in [
            source.replacen("mark2", "mark3", 1),
            source.replacen("means it = B.data", "equals B.data", 1),
        ] {
            let (mutated_ast, _, _, _, _) =
                task253_ast_from_source_text_with_diagnostic_count(&mutated, 503_301);
            assert!(!super::parse_only::step5c4_parse_shape(&mutated_ast));
        }
        let changed_kind = rebuild_surface_ast_replacing_kind(
            &ast,
            SurfaceNodeKind::CorrectnessCondition,
            SurfaceNodeKind::FormulaDefiniens,
        );
        assert_eq!(changed_kind.token_texts(), ast.token_texts());
        assert!(!super::parse_only::step5c4_parse_shape(&changed_kind));
    }

    #[test]
    fn step5c4_parse_admission_and_inventory_reject_metadata_and_path_drift() {
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("workspace root")
            .to_path_buf();
        let config = DiscoveryConfig {
            workspace_root: workspace_root.clone(),
            tests_root: workspace_root.join("tests"),
            manifest_path: workspace_root.join("tests/coverage/spec_trace.toml"),
            profile: TestProfile::Fast,
            validation_mode: ValidationMode::Metadata,
        };
        let plan = build_test_plan(&config).expect("repository plan");
        let id = "fail_parse_only_mode_property_impl_missing_correctness_001";
        let exact = plan
            .cases
            .iter()
            .find(|case| case.id.0 == id)
            .expect("Step 5C.4 parse case")
            .clone();
        assert!(super::parse_only::is_step5c4_parse_only_case(&exact));
        assert!(super::is_active_parse_only(&exact));

        let mut variants = Vec::new();
        let mut missing_tag = exact.clone();
        missing_tag.expectation.tags.clear();
        variants.push(missing_tag);
        let mut duplicate_tag = exact.clone();
        duplicate_tag
            .expectation
            .tags
            .push("active_parse_only".to_owned());
        variants.push(duplicate_tag);
        let mut wrong_stage = exact.clone();
        wrong_stage.expectation.stage = crate::Stage::TypeElaboration;
        variants.push(wrong_stage);
        let mut wrong_phase = exact.clone();
        wrong_phase.expectation.expected_phase = Some(crate::PipelinePhase::TypeCheck);
        variants.push(wrong_phase);
        let mut wrong_outcome = exact.clone();
        wrong_outcome.expectation.expected_outcome = crate::ExpectedOutcome::Pass;
        variants.push(wrong_outcome);
        let mut wrong_key = exact.clone();
        wrong_key.expectation.stable_detail_key = Some("wrong.key".to_owned());
        variants.push(wrong_key);
        for variant in variants {
            assert!(!super::parse_only::is_step5c4_parse_only_case(&variant));
            let mut mutated = plan.clone();
            *mutated
                .cases
                .iter_mut()
                .find(|case| case.id.0 == id)
                .expect("mutable Step 5C.4 parse case") = variant;
            assert!(super::validate_active_parse_only_tags(&workspace_root, &mutated)
                .iter()
                .any(|diagnostic| diagnostic.code.0 == "E-PARSE-ONLY-ACTIVE-GATE"));
        }

        let mut alias_plan = plan.clone();
        let mut alias = exact.clone();
        alias.source_path = workspace_root.join("alias").join(&alias.expectation.source);
        alias_plan.cases.push(alias);
        assert!(super::validate_active_parse_only_tags(&workspace_root, &alias_plan)
            .iter()
            .any(|diagnostic| diagnostic.code.0 == "E-PARSE-ONLY-ACTIVE-GATE"));

        let mut duplicate_plan = plan.clone();
        duplicate_plan.cases.push(exact);
        assert!(super::parse_only::validate_step5c4_parse_only_inventory(
            &workspace_root,
            &duplicate_plan
        )
        .iter()
        .any(|diagnostic| diagnostic.code.0 == "E-PARSE-ONLY-STEP5C4-INVENTORY"));
        let mut missing_plan = plan;
        missing_plan.cases.retain(|case| case.id.0 != id);
        assert!(super::parse_only::validate_step5c4_parse_only_inventory(
            &workspace_root,
            &missing_plan
        )
        .iter()
        .any(|diagnostic| diagnostic.code.0 == "E-PARSE-ONLY-STEP5C4-INVENTORY"));
    }
