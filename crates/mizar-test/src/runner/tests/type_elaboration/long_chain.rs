    #[test]
    fn source_local_mode_long_chain_reserved_variable_equality_consumes_seven_expansions() {
        let source_id = source_id(172);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("local_mode_long_chain_reserved_variable_equality"),
        );
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseMode", SymbolKind::Mode),
                ("ChainMode1", SymbolKind::Mode),
                ("ChainMode2", SymbolKind::Mode),
                ("ChainMode3", SymbolKind::Mode),
                ("ChainMode4", SymbolKind::Mode),
                ("ChainMode5", SymbolKind::Mode),
                ("ChainMode6", SymbolKind::Mode),
                ("ExtraLongMode", SymbolKind::Mode),
            ],
        );
        let theorem = IdentifierBinaryTheoremSpec {
            status: None,
            label: "LongLocalModeReservedVariableEqualityPayloadBoundary",
            left: "z",
            operator: "=",
            right: "z",
            recovered_label: false,
        };
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    "BaseMode",
                    "BaseModeDef",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition_with_label(
                    "ChainMode1",
                    "ChainMode1Def",
                    ReserveTypeShape::QualifiedSymbol("BaseMode"),
                ),
                mode_definition_with_label(
                    "ChainMode2",
                    "ChainMode2Def",
                    ReserveTypeShape::QualifiedSymbol("ChainMode1"),
                ),
                mode_definition_with_label(
                    "ChainMode3",
                    "ChainMode3Def",
                    ReserveTypeShape::QualifiedSymbol("ChainMode2"),
                ),
                mode_definition_with_label(
                    "ChainMode4",
                    "ChainMode4Def",
                    ReserveTypeShape::QualifiedSymbol("ChainMode3"),
                ),
                mode_definition_with_label(
                    "ChainMode5",
                    "ChainMode5Def",
                    ReserveTypeShape::QualifiedSymbol("ChainMode4"),
                ),
                mode_definition_with_label(
                    "ChainMode6",
                    "ChainMode6Def",
                    ReserveTypeShape::QualifiedSymbol("ChainMode5"),
                ),
            ]
        };
        let reserve = || {
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("ChainMode6"),
            )]
        };
        let exact = mode_then_reserve_identifier_binary_theorem_ast(
            source_id,
            exact_modes(),
            reserve(),
            theorem,
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        let payload = extract_source_local_mode_long_chain_reserved_variable_equality(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact local-mode long-chain equality should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 7);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(payload.reserve.bridge.bindings()[0].spelling, "z");
        assert_eq!(
            payload.reserve.bridge.bindings()[0].type_spelling,
            "ChainMode6"
        );
        assert_eq!(payload.left_lookup_ordinal, 1);
        assert_eq!(payload.right_lookup_ordinal, 2);

        let output = source_local_mode_long_chain_reserved_variable_equality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact long-chain equality should reach TermFormulaChecker");
        assert_source_reserved_variable_formula_output(&output)
            .expect("long-chain equality invariants should hold");
        assert_eq!(output.left_binding, BindingId::new(0));
        assert_eq!(output.right_binding, BindingId::new(0));
        for input in [
            &output.left_result_input,
            &output.right_result_input,
            output
                .left_expected_input
                .as_ref()
                .expect("left expected input should exist"),
            output
                .right_expected_input
                .as_ref()
                .expect("right expected input should exist"),
        ] {
            assert_eq!(input.spelling, "ChainMode6");
            assert!(matches!(input.head, TypeHeadInput::Symbol(_)));
            assert_eq!(
                input.source_range,
                output.payload.reserve.bridge.bindings()[0].type_range
            );
        }
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("BaseMode"))
            .map(|(_, expansion)| &expansion.radix)
            .expect("BaseMode terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("one normalized terminal type should exist");
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
        assert_ne!(
            output.payload.reserve.bridge.bindings()[0].type_range,
            terminal.source_range
        );
        assert_eq!(output.term_formula.terms().len(), 2);
        let (_, formula) = output
            .term_formula
            .formulas()
            .iter()
            .next()
            .expect("long-chain equality formula should be checked");
        assert_eq!(formula.kind, FormulaKind::Equality);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert_eq!(formula.expected_types.len(), 2);
        assert_eq!(
            formula.expected_types[0].source_range,
            output.payload.left_range
        );
        assert_eq!(
            formula.expected_types[1].source_range,
            output.payload.right_range
        );
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
        assert!(output.term_formula.facts().is_empty());
        assert!(output.term_formula.diagnostics().is_empty());

        let invalid_key = || {
            vec![
                TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY
                    .to_owned(),
            ]
        };
        let mut missing_left_expected =
            source_local_mode_long_chain_reserved_variable_equality_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce a left-expected corruption target");
        missing_left_expected.left_expected_input = None;
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&missing_left_expected),
            invalid_key()
        );
        let mut missing_right_expected =
            source_local_mode_long_chain_reserved_variable_equality_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce a right-expected corruption target");
        missing_right_expected.right_expected_input = None;
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&missing_right_expected),
            invalid_key()
        );
        for removed in [
            "BaseMode",
            "ChainMode1",
            "ChainMode2",
            "ChainMode3",
            "ChainMode4",
            "ChainMode5",
            "ChainMode6",
        ] {
            let mut invalid = source_local_mode_long_chain_reserved_variable_equality_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce a seven-expansion corruption target");
            invalid
                .payload
                .reserve
                .mode_expansions
                .retain(|symbol, _| source_mode_symbol_spelling(symbol) != Some(removed));
            assert_eq!(
                source_reserved_variable_formula_output_detail_keys(&invalid),
                invalid_key()
            );
        }

        let assert_extraction_gap = |modes| {
            let near_miss = mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                modes,
                reserve(),
                theorem,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        };
        for index in 0..7 {
            let mut missing = exact_modes();
            missing.remove(index);
            assert_extraction_gap(missing);

            let mut wrong_label = exact_modes();
            wrong_label[index].label = Some("OtherDef");
            assert_extraction_gap(wrong_label);

            let mut recovered = exact_modes();
            recovered[index].recovered = true;
            assert_extraction_gap(recovered);

            let mut contextual = exact_modes();
            contextual[index].local_context = true;
            assert_extraction_gap(contextual);

            let mut parameterized = exact_modes();
            parameterized[index].parameterized_pattern = true;
            assert_extraction_gap(parameterized);
        }
        for (index, radix) in [
            "BaseMode",
            "ChainMode1",
            "ChainMode2",
            "ChainMode3",
            "ChainMode4",
            "ChainMode5",
        ]
        .into_iter()
        .enumerate()
        {
            let index = index + 1;
            let mut argument_bearing = exact_modes();
            argument_bearing[index].rhs_shape = ReserveTypeShape::QualifiedSymbolWithArgs(radix);
            assert_extraction_gap(argument_bearing);

            let mut wrong_radix = exact_modes();
            wrong_radix[index].rhs_shape = ReserveTypeShape::QualifiedSymbol("ExtraLongMode");
            assert_extraction_gap(wrong_radix);
        }
        let mut duplicate = exact_modes();
        duplicate.push(duplicate[0]);
        assert_extraction_gap(duplicate);

        let mut attributed_terminal = exact_modes();
        attributed_terminal[0].rhs_shape = ReserveTypeShape::AttributedSet;
        assert_extraction_gap(attributed_terminal);

        let mut object_terminal = exact_modes();
        object_terminal[0].rhs_shape = ReserveTypeShape::Builtin("object");
        assert_extraction_gap(object_terminal);

        let mut forward_order = exact_modes();
        forward_order.reverse();
        assert_extraction_gap(forward_order);

        let mut direct_outermost = exact_modes();
        direct_outermost[6].rhs_shape = ReserveTypeShape::Builtin("set");
        assert_extraction_gap(direct_outermost);
        for radix in [
            "BaseMode",
            "ChainMode1",
            "ChainMode2",
            "ChainMode3",
            "ChainMode4",
        ] {
            let mut shorter = exact_modes();
            shorter[6].rhs_shape = ReserveTypeShape::QualifiedSymbol(radix);
            assert_extraction_gap(shorter);
        }

        let mut cyclic = exact_modes();
        cyclic[0].rhs_shape = ReserveTypeShape::QualifiedSymbol("ChainMode6");
        assert_extraction_gap(cyclic);

        let mut longer = exact_modes();
        longer[1].rhs_shape = ReserveTypeShape::QualifiedSymbol("ExtraLongMode");
        longer.insert(
            1,
            mode_definition_with_label(
                "ExtraLongMode",
                "ExtraLongModeDef",
                ReserveTypeShape::QualifiedSymbol("BaseMode"),
            ),
        );
        assert_extraction_gap(longer);

        for near_miss in [
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(
                    vec!["z"],
                    ReserveTypeShape::QualifiedSymbolWithArgs("ChainMode6"),
                )],
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierBinaryTheoremSpec {
                    label: "OtherPayloadBoundary",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierBinaryTheoremSpec {
                    left: "y",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierBinaryTheoremSpec {
                    right: "y",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierBinaryTheoremSpec {
                    operator: "<>",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierBinaryTheoremSpec {
                    status: Some("open"),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierBinaryTheoremSpec {
                    recovered_label: true,
                    ..theorem
                },
            ),
            modes_then_empty_definition_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                theorem,
            ),
        ] {
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
        let unresolved_symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module, &unresolved_symbols),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );
    }
    #[test]
    fn active_local_mode_long_chain_reserved_variable_equality_fixture_consumes_seven_expansions() {
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("mizar-test crate should live below the workspace root")
            .to_path_buf();
        let config = DiscoveryConfig {
            workspace_root: workspace_root.clone(),
            tests_root: workspace_root.join("tests"),
            manifest_path: workspace_root.join("tests/coverage/spec_trace.toml"),
            profile: TestProfile::Fast,
            validation_mode: ValidationMode::Metadata,
        };
        let plan = build_test_plan(&config).expect("repository test plan should build");
        let (ordinal, case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| {
                case.id.0
                    == "pass_type_elaboration_local_mode_long_chain_reserved_variable_equality_001"
            })
            .expect("Task 172 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 172 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 172 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_local_mode_long_chain_reserved_variable_equality_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 172 real AST should reach the long-chain equality seam");
        assert_source_reserved_variable_formula_output(&output)
            .expect("Task 172 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
        assert_eq!(output.left_binding, BindingId::new(0));
        assert_eq!(output.right_binding, BindingId::new(0));
        assert!(matches!(
            output.left_result_input.head,
            TypeHeadInput::Symbol(_)
        ));
        assert!(matches!(
            output.right_result_input.head,
            TypeHeadInput::Symbol(_)
        ));
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("BaseMode"))
            .map(|(_, expansion)| &expansion.radix)
            .expect("Task 172 BaseMode terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 172 normalized set type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
    }
    #[test]
    fn source_local_mode_long_chain_reserved_variable_inequality_consumes_seven_expansions() {
        let source_id = source_id(173);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("local_mode_long_chain_reserved_variable_inequality"),
        );
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseMode", SymbolKind::Mode),
                ("ChainMode1", SymbolKind::Mode),
                ("ChainMode2", SymbolKind::Mode),
                ("ChainMode3", SymbolKind::Mode),
                ("ChainMode4", SymbolKind::Mode),
                ("ChainMode5", SymbolKind::Mode),
                ("ChainMode6", SymbolKind::Mode),
            ],
        );
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    "BaseMode",
                    "BaseModeDef",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition_with_label(
                    "ChainMode1",
                    "ChainMode1Def",
                    ReserveTypeShape::QualifiedSymbol("BaseMode"),
                ),
                mode_definition_with_label(
                    "ChainMode2",
                    "ChainMode2Def",
                    ReserveTypeShape::QualifiedSymbol("ChainMode1"),
                ),
                mode_definition_with_label(
                    "ChainMode3",
                    "ChainMode3Def",
                    ReserveTypeShape::QualifiedSymbol("ChainMode2"),
                ),
                mode_definition_with_label(
                    "ChainMode4",
                    "ChainMode4Def",
                    ReserveTypeShape::QualifiedSymbol("ChainMode3"),
                ),
                mode_definition_with_label(
                    "ChainMode5",
                    "ChainMode5Def",
                    ReserveTypeShape::QualifiedSymbol("ChainMode4"),
                ),
                mode_definition_with_label(
                    "ChainMode6",
                    "ChainMode6Def",
                    ReserveTypeShape::QualifiedSymbol("ChainMode5"),
                ),
            ]
        };
        let reserve = || {
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("ChainMode6"),
            )]
        };
        let theorem = IdentifierBinaryTheoremSpec {
            status: None,
            label: "LongLocalModeReservedVariableInequalityPayloadBoundary",
            left: "z",
            operator: "<>",
            right: "z",
            recovered_label: false,
        };
        let exact = mode_then_reserve_identifier_binary_theorem_ast(
            source_id,
            exact_modes(),
            reserve(),
            theorem,
        );
        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        let payload = extract_source_local_mode_long_chain_reserved_variable_inequality(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact long-chain inequality should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 7);
        assert_eq!(payload.left_lookup_ordinal, 1);
        assert_eq!(payload.right_lookup_ordinal, 2);
        let output = source_local_mode_long_chain_reserved_variable_inequality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact long-chain inequality should reach TermFormulaChecker");
        assert_source_reserved_variable_formula_output(&output).unwrap();
        assert_eq!(output.left_binding, BindingId::new(0));
        assert_eq!(output.right_binding, BindingId::new(0));
        for input in [
            &output.left_result_input,
            &output.right_result_input,
            output.left_expected_input.as_ref().unwrap(),
            output.right_expected_input.as_ref().unwrap(),
        ] {
            assert_eq!(input.spelling, "ChainMode6");
            assert!(matches!(input.head, TypeHeadInput::Symbol(_)));
            assert_eq!(
                input.source_range,
                output.payload.reserve.bridge.bindings()[0].type_range
            );
        }
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("BaseMode"))
            .map(|(_, expansion)| &expansion.radix)
            .unwrap();
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .unwrap();
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_eq!(normalized.source.range, terminal.source_range);
        let (_, formula) = output.term_formula.formulas().iter().next().unwrap();
        assert_eq!(formula.kind, FormulaKind::Inequality);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert_eq!(formula.expected_types.len(), 2);
        assert_eq!(
            formula.expected_types[0].source_range,
            output.payload.left_range
        );
        assert_eq!(
            formula.expected_types[1].source_range,
            output.payload.right_range
        );
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());

        let invalid_key = || {
            vec![TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY.to_owned()]
        };
        for removed in [
            "BaseMode",
            "ChainMode1",
            "ChainMode2",
            "ChainMode3",
            "ChainMode4",
            "ChainMode5",
            "ChainMode6",
        ] {
            let mut invalid = source_local_mode_long_chain_reserved_variable_inequality_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .unwrap();
            invalid
                .payload
                .reserve
                .mode_expansions
                .retain(|symbol, _| source_mode_symbol_spelling(symbol) != Some(removed));
            assert_eq!(
                source_reserved_variable_formula_output_detail_keys(&invalid),
                invalid_key()
            );
        }
        for clear_left in [true, false] {
            let mut invalid = source_local_mode_long_chain_reserved_variable_inequality_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .unwrap();
            if clear_left {
                invalid.left_expected_input = None;
            } else {
                invalid.right_expected_input = None;
            }
            assert_eq!(
                source_reserved_variable_formula_output_detail_keys(&invalid),
                invalid_key()
            );
        }
        for near_miss in [
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierBinaryTheoremSpec {
                    operator: "=",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierBinaryTheoremSpec {
                    status: Some("open"),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(
                    vec!["z"],
                    ReserveTypeShape::QualifiedSymbolWithArgs("ChainMode6"),
                )],
                theorem,
            ),
        ] {
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
    }
    #[test]
    fn active_local_mode_long_chain_reserved_variable_inequality_fixture_consumes_seven_expansions()
    {
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("mizar-test crate should live below the workspace root")
            .to_path_buf();
        let config = DiscoveryConfig {
            workspace_root: workspace_root.clone(),
            tests_root: workspace_root.join("tests"),
            manifest_path: workspace_root.join("tests/coverage/spec_trace.toml"),
            profile: TestProfile::Fast,
            validation_mode: ValidationMode::Metadata,
        };
        let plan = build_test_plan(&config).expect("repository test plan should build");
        let (ordinal, case) = active_type_elaboration_cases(&plan).enumerate()
            .find(|(_, case)| case.id.0 == "pass_type_elaboration_local_mode_long_chain_reserved_variable_inequality_001")
            .expect("Task 173 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 173 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 173 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_local_mode_long_chain_reserved_variable_inequality_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 173 real AST should reach long-chain inequality");
        assert_source_reserved_variable_formula_output(&output).unwrap();
        assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("BaseMode"))
            .map(|(_, expansion)| &expansion.radix)
            .unwrap();
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .unwrap();
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_eq!(normalized.source.range, terminal.source_range);
    }
    #[test]
    fn source_local_mode_long_chain_reserved_variable_membership_consumes_seven_expansions() {
        let source_id = source_id(174);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("local_mode_long_chain_reserved_variable_membership"),
        );
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseMode", SymbolKind::Mode),
                ("ChainMode1", SymbolKind::Mode),
                ("ChainMode2", SymbolKind::Mode),
                ("ChainMode3", SymbolKind::Mode),
                ("ChainMode4", SymbolKind::Mode),
                ("ChainMode5", SymbolKind::Mode),
                ("ChainMode6", SymbolKind::Mode),
            ],
        );
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    "BaseMode",
                    "BaseModeDef",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition_with_label(
                    "ChainMode1",
                    "ChainMode1Def",
                    ReserveTypeShape::QualifiedSymbol("BaseMode"),
                ),
                mode_definition_with_label(
                    "ChainMode2",
                    "ChainMode2Def",
                    ReserveTypeShape::QualifiedSymbol("ChainMode1"),
                ),
                mode_definition_with_label(
                    "ChainMode3",
                    "ChainMode3Def",
                    ReserveTypeShape::QualifiedSymbol("ChainMode2"),
                ),
                mode_definition_with_label(
                    "ChainMode4",
                    "ChainMode4Def",
                    ReserveTypeShape::QualifiedSymbol("ChainMode3"),
                ),
                mode_definition_with_label(
                    "ChainMode5",
                    "ChainMode5Def",
                    ReserveTypeShape::QualifiedSymbol("ChainMode4"),
                ),
                mode_definition_with_label(
                    "ChainMode6",
                    "ChainMode6Def",
                    ReserveTypeShape::QualifiedSymbol("ChainMode5"),
                ),
            ]
        };
        let reserves = || {
            vec![
                reserve_item(vec!["x"], ReserveTypeShape::QualifiedSymbol("ChainMode6")),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
            ]
        };
        let theorem = IdentifierBinaryTheoremSpec {
            status: None,
            label: "LongLocalModeReservedVariableMembershipPayloadBoundary",
            left: "x",
            operator: "in",
            right: "y",
            recovered_label: false,
        };
        let exact = mode_then_reserve_identifier_binary_theorem_ast(
            source_id,
            exact_modes(),
            reserves(),
            theorem,
        );
        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );

        let payload = extract_source_local_mode_long_chain_reserved_variable_membership(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact long-chain membership should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 7);
        assert_eq!(payload.reserve.bridge.bindings().len(), 2);
        assert_eq!(payload.reserve.bridge.bindings()[0].spelling, "x");
        assert_eq!(
            payload.reserve.bridge.bindings()[0].type_spelling,
            "ChainMode6"
        );
        assert_eq!(payload.reserve.bridge.bindings()[1].spelling, "y");
        assert_eq!(payload.reserve.bridge.bindings()[1].type_spelling, "set");
        assert_ne!(
            payload.reserve.bridge.bindings()[0].type_range,
            payload.reserve.bridge.bindings()[1].type_range
        );
        assert_eq!(payload.left_lookup_ordinal, 2);
        assert_eq!(payload.right_lookup_ordinal, 3);

        let output = source_local_mode_long_chain_reserved_variable_membership_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact long-chain membership should reach TermFormulaChecker");
        assert_source_reserved_variable_formula_output(&output)
            .expect("long-chain membership invariants should hold");
        assert_eq!(output.left_binding, BindingId::new(0));
        assert_eq!(output.right_binding, BindingId::new(1));
        assert_eq!(output.left_result_input.spelling, "ChainMode6");
        assert!(matches!(
            output.left_result_input.head,
            TypeHeadInput::Symbol(_)
        ));
        assert_eq!(
            output.left_result_input.source_range,
            output.payload.reserve.bridge.bindings()[0].type_range
        );
        assert!(output.left_expected_input.is_none());
        assert_eq!(output.right_result_input.spelling, "set");
        assert_eq!(output.right_result_input.head, TypeHeadInput::BuiltinSet);
        let right_expected = output
            .right_expected_input
            .as_ref()
            .expect("right expected-set input should exist");
        assert_eq!(right_expected.spelling, "set");
        assert_eq!(right_expected.head, TypeHeadInput::BuiltinSet);
        assert_eq!(
            output.right_result_input.source_range,
            output.payload.reserve.bridge.bindings()[1].type_range
        );
        assert_eq!(
            right_expected.source_range,
            output.payload.reserve.bridge.bindings()[1].type_range
        );
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("BaseMode"))
            .map(|(_, expansion)| &expansion.radix)
            .expect("BaseMode terminal expansion should exist");
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("one normalized terminal type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
        assert_eq!(output.term_formula.terms().len(), 2);
        for (_, term) in output.term_formula.terms().iter() {
            assert_eq!(term.kind, TermKind::Variable);
            assert_eq!(term.status, TermStatus::Inferred);
        }
        let (_, formula) = output
            .term_formula
            .formulas()
            .iter()
            .next()
            .expect("long-chain membership formula should be checked");
        assert_eq!(formula.kind, FormulaKind::Membership);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert_eq!(formula.expected_types.len(), 1);
        assert_eq!(formula.expected_types[0].term, output.payload.right_site);
        assert_eq!(
            formula.expected_types[0].source_range,
            output.payload.right_range
        );
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
        assert!(output.term_formula.facts().is_empty());
        assert!(output.term_formula.diagnostics().is_empty());
        let type_roles = output
            .term_formula
            .type_entries()
            .iter()
            .filter_map(|(_, entry)| match &entry.owner {
                TypedSiteRef::Role { role, .. } => Some(role.as_str().to_owned()),
                _ => None,
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(
            type_roles,
            BTreeSet::from([
                "long-local-mode-reserved-variable-membership-left-result".to_owned(),
                "long-local-mode-reserved-variable-membership-right-expected".to_owned(),
                "long-local-mode-reserved-variable-membership-right-result".to_owned(),
            ])
        );

        let invalid_key = || {
            vec![TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY.to_owned()]
        };
        for removed in [
            "BaseMode",
            "ChainMode1",
            "ChainMode2",
            "ChainMode3",
            "ChainMode4",
            "ChainMode5",
            "ChainMode6",
        ] {
            let mut invalid = source_local_mode_long_chain_reserved_variable_membership_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce an expansion corruption target");
            invalid
                .payload
                .reserve
                .mode_expansions
                .retain(|symbol, _| source_mode_symbol_spelling(symbol) != Some(removed));
            assert_eq!(
                source_reserved_variable_formula_output_detail_keys(&invalid),
                invalid_key()
            );
        }
        let mut unexpected_left = source_local_mode_long_chain_reserved_variable_membership_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce a left-expected corruption target");
        unexpected_left.left_expected_input = unexpected_left.right_expected_input.clone();
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&unexpected_left),
            invalid_key()
        );
        let mut missing_right = source_local_mode_long_chain_reserved_variable_membership_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce a missing-right corruption target");
        missing_right.right_expected_input = None;
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&missing_right),
            invalid_key()
        );
        let mut wrong_right = source_local_mode_long_chain_reserved_variable_membership_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce a right-type corruption target");
        wrong_right
            .right_expected_input
            .as_mut()
            .expect("right expected input should exist")
            .head = TypeHeadInput::BuiltinObject;
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&wrong_right),
            invalid_key()
        );

        for near_miss in [
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                reserves(),
                IdentifierBinaryTheoremSpec {
                    operator: "=",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                reserves(),
                IdentifierBinaryTheoremSpec {
                    status: Some("open"),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                vec![
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                    reserve_item(vec!["x"], ReserveTypeShape::QualifiedSymbol("ChainMode6")),
                ],
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                vec![
                    reserve_item(
                        vec!["x"],
                        ReserveTypeShape::QualifiedSymbolWithArgs("ChainMode6"),
                    ),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                ],
                theorem,
            ),
        ] {
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
    }
    #[test]
    fn active_local_mode_long_chain_reserved_variable_membership_fixture_consumes_seven_expansions()
    {
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("mizar-test crate should live below the workspace root")
            .to_path_buf();
        let config = DiscoveryConfig {
            workspace_root: workspace_root.clone(),
            tests_root: workspace_root.join("tests"),
            manifest_path: workspace_root.join("tests/coverage/spec_trace.toml"),
            profile: TestProfile::Fast,
            validation_mode: ValidationMode::Metadata,
        };
        let plan = build_test_plan(&config).expect("repository test plan should build");
        let (ordinal, case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| {
                case.id.0
                    == "pass_type_elaboration_local_mode_long_chain_reserved_variable_membership_001"
            })
            .expect("Task 174 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 174 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 174 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_local_mode_long_chain_reserved_variable_membership_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 174 real AST should reach long-chain membership");
        assert_source_reserved_variable_formula_output(&output)
            .expect("Task 174 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
        assert_eq!(output.left_binding, BindingId::new(0));
        assert_eq!(output.right_binding, BindingId::new(1));
        assert_eq!(output.payload.left_lookup_ordinal, 2);
        assert_eq!(output.payload.right_lookup_ordinal, 3);
        assert!(output.left_expected_input.is_none());
        let (_, formula) = output
            .term_formula
            .formulas()
            .iter()
            .next()
            .expect("Task 174 membership formula should be checked");
        assert_eq!(formula.kind, FormulaKind::Membership);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert_eq!(formula.expected_types.len(), 1);
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("BaseMode"))
            .map(|(_, expansion)| &expansion.radix)
            .expect("Task 174 BaseMode terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 174 normalized set type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
    }
    #[test]
    fn source_local_mode_long_chain_reserved_variable_type_assertion_consumes_seven_expansions() {
        let source_id = source_id(175);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("local_mode_long_chain_reserved_variable_type_assertion"),
        );
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseMode", SymbolKind::Mode),
                ("ChainMode1", SymbolKind::Mode),
                ("ChainMode2", SymbolKind::Mode),
                ("ChainMode3", SymbolKind::Mode),
                ("ChainMode4", SymbolKind::Mode),
                ("ChainMode5", SymbolKind::Mode),
                ("ChainMode6", SymbolKind::Mode),
            ],
        );
        let theorem = IdentifierTypeAssertionTheoremSpec {
            status: None,
            label: "LongLocalModeReservedVariableTypeAssertionPayloadBoundary",
            subject: "x",
            asserted_type: ReserveTypeShape::Builtin("set"),
            recovered_label: false,
            negated: false,
        };
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    "BaseMode",
                    "BaseModeDef",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition_with_label(
                    "ChainMode1",
                    "ChainMode1Def",
                    ReserveTypeShape::QualifiedSymbol("BaseMode"),
                ),
                mode_definition_with_label(
                    "ChainMode2",
                    "ChainMode2Def",
                    ReserveTypeShape::QualifiedSymbol("ChainMode1"),
                ),
                mode_definition_with_label(
                    "ChainMode3",
                    "ChainMode3Def",
                    ReserveTypeShape::QualifiedSymbol("ChainMode2"),
                ),
                mode_definition_with_label(
                    "ChainMode4",
                    "ChainMode4Def",
                    ReserveTypeShape::QualifiedSymbol("ChainMode3"),
                ),
                mode_definition_with_label(
                    "ChainMode5",
                    "ChainMode5Def",
                    ReserveTypeShape::QualifiedSymbol("ChainMode4"),
                ),
                mode_definition_with_label(
                    "ChainMode6",
                    "ChainMode6Def",
                    ReserveTypeShape::QualifiedSymbol("ChainMode5"),
                ),
            ]
        };
        let reserve = || {
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("ChainMode6"),
            )]
        };
        let exact = mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            exact_modes(),
            reserve(),
            theorem,
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        let payload = extract_source_local_mode_long_chain_reserved_variable_type_assertion(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact local-mode long-chain type assertion should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 7);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(payload.reserve.bridge.bindings()[0].spelling, "x");
        assert_eq!(
            payload.reserve.bridge.bindings()[0].type_spelling,
            "ChainMode6"
        );
        assert_eq!(payload.asserted_type.spelling, "set");
        assert_eq!(payload.asserted_type.head, TypeHeadInput::BuiltinSet);
        assert_ne!(
            payload.reserve.bridge.bindings()[0].type_range,
            payload.asserted_type.range
        );
        assert_eq!(payload.subject_lookup_ordinal, 1);

        let output = source_local_mode_long_chain_reserved_variable_type_assertion_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact long-chain type assertion should reach TermFormulaChecker");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("long-chain type assertion invariants should hold");
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.subject_result_input.spelling, "ChainMode6");
        assert!(matches!(
            output.subject_result_input.head,
            TypeHeadInput::Symbol(_)
        ));
        assert_eq!(output.asserted_type_input.spelling, "set");
        assert_eq!(output.asserted_type_input.head, TypeHeadInput::BuiltinSet);
        assert_ne!(
            output.subject_result_input.source_range,
            output.asserted_type_input.source_range
        );
        assert_eq!(output.term_formula.terms().len(), 1);
        let (_, term) = output
            .term_formula
            .terms()
            .iter()
            .next()
            .expect("long-chain type-assertion subject should be inferred");
        assert_eq!(term.status, TermStatus::Inferred);
        assert!(term.deferred.is_empty());
        let (_, formula) = output
            .term_formula
            .formulas()
            .iter()
            .next()
            .expect("long-chain type assertion should be checked");
        assert_eq!(formula.kind, FormulaKind::TypeAssertion);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert!(formula.expected_types.is_empty());
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("BaseMode"))
            .map(|(_, expansion)| &expansion.radix)
            .expect("BaseMode terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("one normalized terminal set should exist");
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);

        let invalid_key = || {
            vec![
                TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY
                    .to_owned(),
            ]
        };
        for removed in [
            "BaseMode",
            "ChainMode1",
            "ChainMode2",
            "ChainMode3",
            "ChainMode4",
            "ChainMode5",
            "ChainMode6",
        ] {
            let mut invalid = source_local_mode_long_chain_reserved_variable_type_assertion_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce a seven-expansion corruption target");
            invalid
                .payload
                .reserve
                .mode_expansions
                .retain(|symbol, _| source_mode_symbol_spelling(symbol) != Some(removed));
            let invalid_result =
                assert_source_reserved_variable_type_assertion_output(&invalid).map(|()| invalid);
            assert_eq!(
                source_reserved_variable_type_assertion_result_detail_keys(
                    invalid_result,
                    TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY,
                ),
                invalid_key()
            );
        }
        let mut wrong_asserted =
            source_local_mode_long_chain_reserved_variable_type_assertion_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce an asserted-type corruption target");
        wrong_asserted.asserted_type_input.head = TypeHeadInput::BuiltinObject;
        let wrong_asserted_result =
            assert_source_reserved_variable_type_assertion_output(&wrong_asserted)
                .map(|()| wrong_asserted);
        assert_eq!(
            source_reserved_variable_type_assertion_result_detail_keys(
                wrong_asserted_result,
                TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY,
            ),
            invalid_key()
        );
        let mut wrong_subject =
            source_local_mode_long_chain_reserved_variable_type_assertion_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce a subject-provenance corruption target");
        wrong_subject.subject_result_input.spelling = "set".to_owned();
        let wrong_subject_result =
            assert_source_reserved_variable_type_assertion_output(&wrong_subject)
                .map(|()| wrong_subject);
        assert_eq!(
            source_reserved_variable_type_assertion_result_detail_keys(
                wrong_subject_result,
                TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY,
            ),
            invalid_key()
        );

        for near_miss in [
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbolWithArgs("ChainMode6"),
                )],
                theorem,
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    label: "OtherPayloadBoundary",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    subject: "y",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::Builtin("object"),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol("ChainMode6"),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    negated: true,
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    status: Some("open"),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    recovered_label: true,
                    ..theorem
                },
            ),
            modes_then_empty_definition_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                theorem,
            ),
        ] {
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
        let unresolved_symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module, &unresolved_symbols),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );
    }
    #[test]
    fn source_local_mode_long_chain_radix_asserted_head_consumes_seven_expansions() {
        let source_id = source_id(209);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("local_mode_long_chain_radix_asserted_head"),
        );
        let mode_names = [
            "BaseMode",
            "ChainMode1",
            "ChainMode2",
            "ChainMode3",
            "ChainMode4",
            "ChainMode5",
            "ChainMode6",
        ];
        let mode_labels = [
            "BaseModeDef",
            "ChainMode1Def",
            "ChainMode2Def",
            "ChainMode3Def",
            "ChainMode4Def",
            "ChainMode5Def",
            "ChainMode6Def",
        ];
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                (mode_names[0], SymbolKind::Mode),
                (mode_names[1], SymbolKind::Mode),
                (mode_names[2], SymbolKind::Mode),
                (mode_names[3], SymbolKind::Mode),
                (mode_names[4], SymbolKind::Mode),
                (mode_names[5], SymbolKind::Mode),
                (mode_names[6], SymbolKind::Mode),
                ("ChainMode7", SymbolKind::Mode),
                ("OtherLongMode", SymbolKind::Mode),
            ],
        );
        let theorem = IdentifierTypeAssertionTheoremSpec {
            status: None,
            label: "LongLocalModeRadixAssertedHeadPayloadBoundary",
            subject: "x",
            asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[5]),
            negated: false,
            recovered_label: false,
        };
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    mode_names[0],
                    mode_labels[0],
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition_with_label(
                    mode_names[1],
                    mode_labels[1],
                    ReserveTypeShape::QualifiedSymbol(mode_names[0]),
                ),
                mode_definition_with_label(
                    mode_names[2],
                    mode_labels[2],
                    ReserveTypeShape::QualifiedSymbol(mode_names[1]),
                ),
                mode_definition_with_label(
                    mode_names[3],
                    mode_labels[3],
                    ReserveTypeShape::QualifiedSymbol(mode_names[2]),
                ),
                mode_definition_with_label(
                    mode_names[4],
                    mode_labels[4],
                    ReserveTypeShape::QualifiedSymbol(mode_names[3]),
                ),
                mode_definition_with_label(
                    mode_names[5],
                    mode_labels[5],
                    ReserveTypeShape::QualifiedSymbol(mode_names[4]),
                ),
                mode_definition_with_label(
                    mode_names[6],
                    mode_labels[6],
                    ReserveTypeShape::QualifiedSymbol(mode_names[5]),
                ),
            ]
        };
        let reserve = || {
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol(mode_names[6]),
            )]
        };
        let exact = mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            exact_modes(),
            reserve(),
            theorem,
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        assert!(
            extract_source_local_mode_long_chain_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 175 must reject the Task 209 immediate-radix source"
        );
        assert!(
            super::extract_source_local_mode_long_chain_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 199 must reject the Task 209 immediate-radix source"
        );
        let preexisting_owner_results = [
            extract_source_reserved_variable_type_assertion(&exact, module.clone(), &symbols),
            super::extract_source_reserved_object_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_local_object_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_chained_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_chained_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_chained_local_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_chained_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_chained_local_mode_radix_asserted_head(&exact, module.clone(), &symbols),
            extract_source_chained_local_object_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_two_edge_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_object_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_three_edge_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_object_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_four_edge_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_four_edge_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            super::extract_source_four_edge_local_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            super::extract_source_four_edge_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_four_edge_local_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_four_edge_local_object_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_mode_long_chain_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_object_mode_long_chain_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            super::extract_source_local_mode_long_chain_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            super::extract_source_local_object_mode_long_chain_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
        ];
        assert_eq!(preexisting_owner_results.len(), 34);
        assert!(
            preexisting_owner_results
                .into_iter()
                .all(|result| result.is_none()),
            "all 34 pre-existing type-assertion owner routes must reject Task 209"
        );
        let payload = super::extract_source_local_mode_long_chain_radix_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact seven-expansion immediate-radix source should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 7);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(payload.subject_lookup_ordinal, 1);
        let source_binding = &payload.reserve.bridge.bindings()[0];
        assert_eq!(source_binding.type_spelling, mode_names[6]);
        assert_eq!(payload.asserted_type.spelling, mode_names[5]);
        assert_ne!(payload.asserted_type.head, source_binding.type_head);
        assert!(matches!(source_binding.type_head, TypeHeadInput::Symbol(_)));
        assert_ne!(payload.asserted_type_site, payload.subject_site);
        assert_ne!(payload.asserted_type.range, source_binding.type_range);
        let binding_expansion = payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[6]))
            .map(|(_, expansion)| expansion)
            .expect("ChainMode6 binding expansion should exist");
        assert_eq!(binding_expansion.radix.spelling, mode_names[5]);
        assert_eq!(binding_expansion.radix.head, payload.asserted_type.head);

        let output = super::source_local_mode_long_chain_radix_asserted_head_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact seven-expansion asserted-head source should reach checker");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 209 checker payload invariants should hold");
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert_eq!(output.subject_result_input.spelling, mode_names[6]);
        assert_eq!(output.asserted_type_input.spelling, mode_names[5]);
        assert_ne!(
            output.subject_result_input.head,
            output.asserted_type_input.head
        );
        assert!(matches!(
            output.asserted_type_input.head,
            TypeHeadInput::Symbol(_)
        ));
        assert_ne!(
            output.subject_result_input.site,
            output.asserted_type_input.site
        );
        assert_ne!(
            output.subject_result_input.source_range,
            output.asserted_type_input.source_range
        );
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[0]))
            .map(|(_, expansion)| &expansion.radix)
            .expect("BaseMode terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("one normalized builtin-set type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
        let (_, term) = output.term_formula.terms().iter().next().unwrap();
        assert_eq!(term.status, TermStatus::Inferred);
        assert!(term.deferred.is_empty());
        let (_, formula) = output.term_formula.formulas().iter().next().unwrap();
        assert_eq!(formula.kind, FormulaKind::TypeAssertion);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert!(formula.expected_types.is_empty());
        assert!(formula.candidate_set.is_none());
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
        assert!(output.term_formula.candidate_sets().is_empty());
        assert!(output.term_formula.facts().is_empty());
        assert!(output.term_formula.diagnostics().is_empty());

        let assert_invalid_output = |invalid| {
            let invalid_result =
                assert_source_reserved_variable_type_assertion_output(&invalid).map(|()| invalid);
            assert_eq!(
                source_reserved_variable_type_assertion_result_detail_keys(
                    invalid_result,
                    super::TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_RADIX_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
                ),
                vec![
                    super::TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_RADIX_ASSERTED_HEAD_INVALID_PAYLOAD_KEY
                        .to_owned()
                ]
            );
        };
        for removed in mode_names {
            let mut invalid = super::source_local_mode_long_chain_radix_asserted_head_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .unwrap();
            invalid
                .payload
                .reserve
                .mode_expansions
                .retain(|symbol, _| source_mode_symbol_spelling(symbol) != Some(removed));
            assert_invalid_output(invalid);
        }
        let exact_output = || {
            super::source_local_mode_long_chain_radix_asserted_head_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .unwrap()
        };
        let mut corruptions = Vec::new();
        let mut invalid = exact_output();
        invalid.subject_binding = BindingId::new(1);
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.payload.subject_lookup_ordinal = 2;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.spelling = mode_names[5].to_owned();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.spelling = "OtherLongMode".to_owned();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.head = invalid.subject_result_input.head.clone();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.head = TypeHeadInput::BuiltinObject;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.head = TypeHeadInput::BuiltinObject;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.source_range = invalid.asserted_type_input.source_range;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.site = invalid.asserted_type_input.site.clone();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.source_range = invalid.subject_result_input.source_range;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.site = invalid.subject_result_input.site.clone();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        let binding_head = invalid.subject_result_input.head.clone();
        let (_, binding_expansion) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[6]))
            .unwrap();
        binding_expansion.radix.head = binding_head;
        corruptions.push(invalid);
        for mode_name in &mode_names[1..] {
            let mut invalid = exact_output();
            let (_, expansion) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(*mode_name))
                .unwrap();
            expansion.radix.spelling = "object".to_owned();
            expansion.radix.head = TypeHeadInput::BuiltinObject;
            corruptions.push(invalid);
        }
        let mut invalid = exact_output();
        let (_, base_expansion) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[0]))
            .unwrap();
        base_expansion.radix.source_range = range(source_id, 0, 1);
        corruptions.push(invalid);
        let mut invalid = exact_output();
        let (_, base_expansion) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[0]))
            .unwrap();
        base_expansion.radix.spelling = "object".to_owned();
        base_expansion.radix.head = TypeHeadInput::BuiltinObject;
        corruptions.push(invalid);
        for invalid in corruptions {
            assert_invalid_output(invalid);
        }
        assert_source_reserved_variable_type_assertion_output(&exact_output())
            .expect("corrupting cloned Task 209 outputs must not mutate the exact output");

        let assert_extraction_gap = |ast, context: &str| {
            assert_eq!(
                source_type_elaboration_detail_keys(&ast, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()],
                "{context}"
            );
        };
        fn next_permutation(order: &mut [usize; 7]) -> bool {
            let Some(pivot) = (0..order.len() - 1)
                .rev()
                .find(|&index| order[index] < order[index + 1])
            else {
                return false;
            };
            let successor = (pivot + 1..order.len())
                .rev()
                .find(|&index| order[pivot] < order[index])
                .expect("a lexicographic successor must exist after the pivot");
            order.swap(pivot, successor);
            order[pivot + 1..].reverse();
            true
        }
        let ordered = exact_modes();
        let public_dispatch_orders = [
            [1, 0, 2, 3, 4, 5, 6],
            [0, 2, 1, 3, 4, 5, 6],
            [0, 1, 3, 2, 4, 5, 6],
            [0, 1, 2, 4, 3, 5, 6],
            [0, 1, 2, 3, 5, 4, 6],
            [0, 1, 2, 3, 4, 6, 5],
            [1, 2, 3, 4, 5, 6, 0],
            [6, 5, 4, 3, 2, 1, 0],
        ];
        let mut order = [0, 1, 2, 3, 4, 5, 6];
        let mut permutation_count = 0;
        let mut public_dispatch_representative_count = 0;
        while next_permutation(&mut order) {
            permutation_count += 1;
            let permuted = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                vec![
                    ordered[order[0]],
                    ordered[order[1]],
                    ordered[order[2]],
                    ordered[order[3]],
                    ordered[order[4]],
                    ordered[order[5]],
                    ordered[order[6]],
                ],
                reserve(),
                theorem,
            );
            assert!(
                extract_source_local_mode_long_chain_radix_asserted_head(
                    &permuted,
                    module.clone(),
                    &symbols,
                )
                .is_none(),
                "definition permutation {order:?} must not reach Task 209"
            );
            if public_dispatch_orders.contains(&order) {
                public_dispatch_representative_count += 1;
                assert_extraction_gap(
                    permuted,
                    &format!("public definition permutation {order:?}"),
                );
            }
        }
        assert_eq!(permutation_count, 5_039);
        assert_eq!(public_dispatch_representative_count, 8);
        let mut mode_near_misses = vec![
            Vec::<ModeDefinitionSpec>::new(),
            {
                let mut modes = exact_modes();
                modes.push(modes[0]);
                modes
            },
            {
                let mut modes = exact_modes();
                modes.push(mode_definition_with_label(
                    "UnusedLongMode",
                    "UnusedLongModeDef",
                    ReserveTypeShape::QualifiedSymbol(mode_names[6]),
                ));
                modes
            },
        ];
        for index in 0..mode_names.len() {
            let mut missing = exact_modes();
            missing.remove(index);
            mode_near_misses.push(missing);
            let mut duplicate = exact_modes();
            duplicate.push(duplicate[index]);
            mode_near_misses.push(duplicate);
            let mut malformed = exact_modes();
            malformed[index].rhs_shape = if index == 0 {
                ReserveTypeShape::Builtin("object")
            } else {
                ReserveTypeShape::QualifiedSymbol(mode_names[(index + 1) % mode_names.len()])
            };
            mode_near_misses.push(malformed);
            let mut wrong_label = exact_modes();
            wrong_label[index].label = Some("WrongLongModeDef");
            mode_near_misses.push(wrong_label);
            let mut wrong_spelling = exact_modes();
            wrong_spelling[index].pattern = "OtherLongMode";
            mode_near_misses.push(wrong_spelling);
            let mut recovered = exact_modes();
            recovered[index].recovered = true;
            mode_near_misses.push(recovered);
            let mut contextual = exact_modes();
            contextual[index].local_context = true;
            mode_near_misses.push(contextual);
            let mut parameterized = exact_modes();
            parameterized[index].parameterized_pattern = true;
            mode_near_misses.push(parameterized);
            let mut args = exact_modes();
            args[index].rhs_shape = if index == 0 {
                ReserveTypeShape::QualifiedSymbolWithArgs("set")
            } else {
                ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[index - 1])
            };
            mode_near_misses.push(args);
            let mut attrs = exact_modes();
            attrs[index].rhs_shape = if index == 0 {
                ReserveTypeShape::AttributedSet
            } else {
                ReserveTypeShape::AttributedQualifiedSymbol(mode_names[index - 1])
            };
            mode_near_misses.push(attrs);
        }
        for (index, modes) in mode_near_misses.into_iter().enumerate() {
            assert_extraction_gap(
                mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    modes,
                    reserve(),
                    theorem,
                ),
                &format!("mode near miss {index}"),
            );
        }

        let mut source_near_misses = Vec::new();
        let mut deeper_modes = exact_modes();
        deeper_modes.push(mode_definition_with_label(
            "ChainMode7",
            "ChainMode7Def",
            ReserveTypeShape::QualifiedSymbol(mode_names[6]),
        ));
        source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            deeper_modes,
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("ChainMode7"),
            )],
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[6]),
                ..theorem
            },
        ));
        for bad_reserve in [
            ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[6]),
            ReserveTypeShape::AttributedQualifiedSymbol(mode_names[6]),
            ReserveTypeShape::Builtin("set"),
            ReserveTypeShape::Builtin("object"),
        ] {
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(vec!["x"], bad_reserve)],
                theorem,
            ));
        }
        source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            exact_modes(),
            vec![reserve_item(
                vec!["y"],
                ReserveTypeShape::QualifiedSymbol(mode_names[6]),
            )],
            theorem,
        ));
        source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            exact_modes(),
            vec![
                reserve_item(vec!["x"], ReserveTypeShape::QualifiedSymbol(mode_names[6])),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
            ],
            theorem,
        ));
        for bad_reserve in [
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol(mode_names[5]),
            )],
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("OtherLongMode"),
            )],
            vec![reserve_item(
                vec!["x", "y"],
                ReserveTypeShape::QualifiedSymbol(mode_names[6]),
            )],
        ] {
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                bad_reserve,
                theorem,
            ));
        }
        let theorem_near_misses = [
            IdentifierTypeAssertionTheoremSpec {
                label: "OtherPayloadBoundary",
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                subject: "y",
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::Builtin("set"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::Builtin("object"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[0]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[1]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[2]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[3]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[4]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[6]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol("OtherLongMode"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[5]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::AttributedQualifiedSymbol(mode_names[5]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                negated: true,
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                status: Some("open"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                recovered_label: true,
                ..theorem
            },
        ];
        for near_miss_theorem in theorem_near_misses {
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                near_miss_theorem,
            ));
        }
        source_near_misses.push(
            modes_then_empty_definition_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                theorem,
            ),
        );
        for (index, near_miss) in source_near_misses.into_iter().enumerate() {
            assert_extraction_gap(near_miss, &format!("source near miss {index}"));
        }

        for imported_index in 0..mode_names.len() {
            let local_modes = mode_names
                .iter()
                .enumerate()
                .filter(|(index, _)| *index != imported_index)
                .map(|(_, spelling)| (*spelling, SymbolKind::Mode))
                .collect::<Vec<_>>();
            for imported in [
                vec![(mode_names[imported_index], SymbolKind::Mode)],
                vec![
                    (mode_names[imported_index], SymbolKind::Mode),
                    (mode_names[imported_index], SymbolKind::Mode),
                ],
            ] {
                let imported_symbols = source_local_and_imported_symbols_env(
                    module.clone(),
                    &local_modes,
                    imported.as_slice(),
                );
                assert_eq!(
                    source_type_elaboration_detail_keys(&exact, module.clone(), &imported_symbols),
                    vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
                );
            }
        }
        let unrelated_import = source_local_and_imported_symbols_env(
            module.clone(),
            &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
            &[("UnrelatedLongMode", SymbolKind::Mode)],
        );
        assert!(
            super::extract_source_local_mode_long_chain_radix_asserted_head(
                &exact,
                module.clone(),
                &unrelated_import,
            )
            .is_some()
        );
        for (asserted_spelling, asserted_head_symbols) in [
            (
                "ImportedLongMode",
                source_local_and_imported_symbols_env(
                    module.clone(),
                    &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
                    &[("ImportedLongMode", SymbolKind::Mode)],
                ),
            ),
            (
                "AmbiguousLongMode",
                source_local_and_imported_symbols_env(
                    module.clone(),
                    &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
                    &[
                        ("AmbiguousLongMode", SymbolKind::Mode),
                        ("AmbiguousLongMode", SymbolKind::Mode),
                    ],
                ),
            ),
        ] {
            let near_miss = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(asserted_spelling),
                    ..theorem
                },
            );
            assert_eq!(
                source_type_elaboration_detail_keys(
                    &near_miss,
                    module.clone(),
                    &asserted_head_symbols,
                ),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
    }

    #[test]
    fn source_local_mode_long_chain_three_hop_asserted_head_consumes_seven_expansions() {
        let source_id = source_id(226);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("local_mode_long_chain_three_hop_asserted_head"),
        );
        let mode_names = [
            "BaseMode",
            "ChainMode1",
            "ChainMode2",
            "ChainMode3",
            "ChainMode4",
            "ChainMode5",
            "ChainMode6",
        ];
        let mode_labels = [
            "BaseModeDef",
            "ChainMode1Def",
            "ChainMode2Def",
            "ChainMode3Def",
            "ChainMode4Def",
            "ChainMode5Def",
            "ChainMode6Def",
        ];
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                (mode_names[0], SymbolKind::Mode),
                (mode_names[1], SymbolKind::Mode),
                (mode_names[2], SymbolKind::Mode),
                (mode_names[3], SymbolKind::Mode),
                (mode_names[4], SymbolKind::Mode),
                (mode_names[5], SymbolKind::Mode),
                (mode_names[6], SymbolKind::Mode),
                ("ChainMode7", SymbolKind::Mode),
                ("OtherLongMode", SymbolKind::Mode),
            ],
        );
        let theorem = IdentifierTypeAssertionTheoremSpec {
            status: None,
            label: "LongLocalModeThreeHopAssertedHeadPayloadBoundary",
            subject: "x",
            asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[3]),
            negated: false,
            recovered_label: false,
        };
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    mode_names[0],
                    mode_labels[0],
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition_with_label(
                    mode_names[1],
                    mode_labels[1],
                    ReserveTypeShape::QualifiedSymbol(mode_names[0]),
                ),
                mode_definition_with_label(
                    mode_names[2],
                    mode_labels[2],
                    ReserveTypeShape::QualifiedSymbol(mode_names[1]),
                ),
                mode_definition_with_label(
                    mode_names[3],
                    mode_labels[3],
                    ReserveTypeShape::QualifiedSymbol(mode_names[2]),
                ),
                mode_definition_with_label(
                    mode_names[4],
                    mode_labels[4],
                    ReserveTypeShape::QualifiedSymbol(mode_names[3]),
                ),
                mode_definition_with_label(
                    mode_names[5],
                    mode_labels[5],
                    ReserveTypeShape::QualifiedSymbol(mode_names[4]),
                ),
                mode_definition_with_label(
                    mode_names[6],
                    mode_labels[6],
                    ReserveTypeShape::QualifiedSymbol(mode_names[5]),
                ),
            ]
        };
        let reserve = || {
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol(mode_names[6]),
            )]
        };
        let exact = mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            exact_modes(),
            reserve(),
            theorem,
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        assert!(
            extract_source_local_mode_long_chain_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 175 must reject the Task 226 three-hop source"
        );
        assert!(
            super::extract_source_local_mode_long_chain_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 199 must reject the Task 226 three-hop source"
        );
        assert!(
            super::extract_source_local_mode_long_chain_two_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 224 must reject the Task 226 three-hop source"
        );
        assert!(
            super::extract_source_local_object_mode_long_chain_two_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 225 must reject the Task 226 set-terminal three-hop source"
        );
        let preexisting_owner_results = [
            extract_source_reserved_variable_type_assertion(&exact, module.clone(), &symbols),
            super::extract_source_reserved_object_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_local_object_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_chained_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_chained_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_chained_local_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_chained_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_chained_local_mode_radix_asserted_head(&exact, module.clone(), &symbols),
            extract_source_chained_local_object_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_two_edge_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_object_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_three_edge_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_object_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_four_edge_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_four_edge_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            super::extract_source_four_edge_local_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            super::extract_source_four_edge_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_four_edge_local_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_four_edge_local_object_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_mode_long_chain_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_object_mode_long_chain_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            super::extract_source_local_mode_long_chain_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            super::extract_source_local_object_mode_long_chain_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
        ];
        assert_eq!(preexisting_owner_results.len(), 34);
        assert!(
            preexisting_owner_results
                .into_iter()
                .all(|result| result.is_none()),
            "all 34 legacy type-assertion owner routes must reject Task 226"
        );
        let payload = super::extract_source_local_mode_long_chain_three_hop_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact seven-expansion three-hop source should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 7);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(payload.subject_lookup_ordinal, 1);
        let source_binding = &payload.reserve.bridge.bindings()[0];
        assert_eq!(source_binding.type_spelling, mode_names[6]);
        assert_eq!(payload.asserted_type.spelling, mode_names[3]);
        assert_ne!(payload.asserted_type.head, source_binding.type_head);
        assert!(matches!(source_binding.type_head, TypeHeadInput::Symbol(_)));
        assert_ne!(payload.asserted_type_site, payload.subject_site);
        assert_ne!(payload.asserted_type.range, source_binding.type_range);
        let binding_expansion = payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[6]))
            .map(|(_, expansion)| expansion)
            .expect("ChainMode6 binding expansion should exist");
        assert_eq!(binding_expansion.radix.spelling, mode_names[5]);
        assert_ne!(binding_expansion.radix.head, payload.asserted_type.head);
        let TypeHeadInput::Symbol(first_intermediate_symbol) = &binding_expansion.radix.head else {
            panic!("ChainMode5 first relation intermediate should resolve to a symbol");
        };
        let first_intermediate_expansion = payload
            .reserve
            .mode_expansions
            .get(first_intermediate_symbol)
            .expect("ChainMode5 first intermediate expansion should exist");
        assert_eq!(first_intermediate_expansion.radix.spelling, mode_names[4]);
        assert_ne!(
            first_intermediate_expansion.radix.head,
            payload.asserted_type.head
        );
        let TypeHeadInput::Symbol(second_intermediate_symbol) =
            &first_intermediate_expansion.radix.head
        else {
            panic!("ChainMode4 second relation intermediate should resolve to a symbol");
        };
        let second_intermediate_expansion = payload
            .reserve
            .mode_expansions
            .get(second_intermediate_symbol)
            .expect("ChainMode4 second intermediate expansion should exist");
        assert_eq!(second_intermediate_expansion.radix.spelling, mode_names[3]);
        assert_eq!(
            second_intermediate_expansion.radix.head,
            payload.asserted_type.head
        );

        let output = super::source_local_mode_long_chain_three_hop_asserted_head_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact seven-expansion asserted-head source should reach checker");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 226 checker payload invariants should hold");
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert_eq!(output.subject_result_input.spelling, mode_names[6]);
        assert_eq!(output.asserted_type_input.spelling, mode_names[3]);
        assert_ne!(
            output.subject_result_input.head,
            output.asserted_type_input.head
        );
        assert!(matches!(
            output.asserted_type_input.head,
            TypeHeadInput::Symbol(_)
        ));
        assert_ne!(
            output.subject_result_input.site,
            output.asserted_type_input.site
        );
        assert_ne!(
            output.subject_result_input.source_range,
            output.asserted_type_input.source_range
        );
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[0]))
            .map(|(_, expansion)| &expansion.radix)
            .expect("BaseMode terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("one normalized builtin-set type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
        let (_, term) = output.term_formula.terms().iter().next().unwrap();
        assert_eq!(term.status, TermStatus::Inferred);
        assert!(term.deferred.is_empty());
        let (_, formula) = output.term_formula.formulas().iter().next().unwrap();
        assert_eq!(formula.kind, FormulaKind::TypeAssertion);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert!(formula.expected_types.is_empty());
        assert!(formula.candidate_set.is_none());
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
        assert!(output.term_formula.candidate_sets().is_empty());
        assert!(output.term_formula.facts().is_empty());
        assert!(output.term_formula.diagnostics().is_empty());

        let assert_invalid_output = |invalid| {
            let invalid_result =
                assert_source_reserved_variable_type_assertion_output(&invalid).map(|()| invalid);
            assert_eq!(
                source_reserved_variable_type_assertion_result_detail_keys(
                    invalid_result,
                    super::TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_THREE_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
                ),
                vec![
                    super::TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_THREE_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY
                        .to_owned()
                ]
            );
        };
        for removed in mode_names {
            let mut invalid = super::source_local_mode_long_chain_three_hop_asserted_head_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .unwrap();
            invalid
                .payload
                .reserve
                .mode_expansions
                .retain(|symbol, _| source_mode_symbol_spelling(symbol) != Some(removed));
            assert_invalid_output(invalid);
        }
        let exact_output = || {
            super::source_local_mode_long_chain_three_hop_asserted_head_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .unwrap()
        };
        let mut corruptions = Vec::new();
        let mut invalid = exact_output();
        invalid.subject_binding = BindingId::new(1);
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.payload.subject_lookup_ordinal = 2;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.spelling = mode_names[5].to_owned();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.spelling = "OtherLongMode".to_owned();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.head = invalid.subject_result_input.head.clone();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.head = TypeHeadInput::BuiltinObject;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.head = TypeHeadInput::BuiltinObject;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.source_range = invalid.asserted_type_input.source_range;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.site = invalid.asserted_type_input.site.clone();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.source_range = invalid.subject_result_input.source_range;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.site = invalid.subject_result_input.site.clone();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        let binding_head = invalid.subject_result_input.head.clone();
        let (_, binding_expansion) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[6]))
            .unwrap();
        binding_expansion.radix.head = binding_head;
        corruptions.push(invalid);
        for mode_name in &mode_names[1..] {
            let mut invalid = exact_output();
            let (_, expansion) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(*mode_name))
                .unwrap();
            expansion.radix.spelling = "object".to_owned();
            expansion.radix.head = TypeHeadInput::BuiltinObject;
            corruptions.push(invalid);
        }
        let mut invalid = exact_output();
        let (_, base_expansion) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[0]))
            .unwrap();
        base_expansion.radix.source_range = range(source_id, 0, 1);
        corruptions.push(invalid);
        let mut invalid = exact_output();
        let (_, base_expansion) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[0]))
            .unwrap();
        base_expansion.radix.spelling = "object".to_owned();
        base_expansion.radix.head = TypeHeadInput::BuiltinObject;
        corruptions.push(invalid);
        for invalid in corruptions {
            assert_invalid_output(invalid);
        }
        assert_source_reserved_variable_type_assertion_output(&exact_output())
            .expect("corrupting cloned Task 226 outputs must not mutate the exact output");

        let assert_extraction_gap = |ast, context: &str| {
            assert_eq!(
                source_type_elaboration_detail_keys(&ast, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()],
                "{context}"
            );
        };
        fn next_permutation(order: &mut [usize; 7]) -> bool {
            let Some(pivot) = (0..order.len() - 1)
                .rev()
                .find(|&index| order[index] < order[index + 1])
            else {
                return false;
            };
            let successor = (pivot + 1..order.len())
                .rev()
                .find(|&index| order[pivot] < order[index])
                .expect("a lexicographic successor must exist after the pivot");
            order.swap(pivot, successor);
            order[pivot + 1..].reverse();
            true
        }
        let ordered = exact_modes();
        let public_dispatch_orders = [
            [1, 0, 2, 3, 4, 5, 6],
            [0, 2, 1, 3, 4, 5, 6],
            [0, 1, 3, 2, 4, 5, 6],
            [0, 1, 2, 4, 3, 5, 6],
            [0, 1, 2, 3, 5, 4, 6],
            [0, 1, 2, 3, 4, 6, 5],
            [1, 2, 3, 4, 5, 6, 0],
            [6, 5, 4, 3, 2, 1, 0],
        ];
        let mut order = [0, 1, 2, 3, 4, 5, 6];
        let mut permutation_count = 0;
        let mut public_dispatch_representative_count = 0;
        while next_permutation(&mut order) {
            permutation_count += 1;
            let permuted = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                vec![
                    ordered[order[0]],
                    ordered[order[1]],
                    ordered[order[2]],
                    ordered[order[3]],
                    ordered[order[4]],
                    ordered[order[5]],
                    ordered[order[6]],
                ],
                reserve(),
                theorem,
            );
            assert!(
                super::extract_source_local_mode_long_chain_three_hop_asserted_head(
                    &permuted,
                    module.clone(),
                    &symbols,
                )
                .is_none(),
                "definition permutation {order:?} must not reach Task 226"
            );
            if public_dispatch_orders.contains(&order) {
                public_dispatch_representative_count += 1;
                assert_extraction_gap(
                    permuted,
                    &format!("public definition permutation {order:?}"),
                );
            }
        }
        assert_eq!(permutation_count, 5_039);
        assert_eq!(public_dispatch_representative_count, 8);
        let mut mode_near_misses = vec![
            Vec::<ModeDefinitionSpec>::new(),
            {
                let mut modes = exact_modes();
                modes.push(modes[0]);
                modes
            },
            {
                let mut modes = exact_modes();
                modes.push(mode_definition_with_label(
                    "UnusedLongMode",
                    "UnusedLongModeDef",
                    ReserveTypeShape::QualifiedSymbol(mode_names[6]),
                ));
                modes
            },
        ];
        for index in 0..mode_names.len() {
            let mut missing = exact_modes();
            missing.remove(index);
            mode_near_misses.push(missing);
            let mut duplicate = exact_modes();
            duplicate.push(duplicate[index]);
            mode_near_misses.push(duplicate);
            let mut malformed = exact_modes();
            malformed[index].rhs_shape = if index == 0 {
                ReserveTypeShape::Builtin("object")
            } else {
                ReserveTypeShape::QualifiedSymbol(mode_names[(index + 1) % mode_names.len()])
            };
            mode_near_misses.push(malformed);
            let mut wrong_label = exact_modes();
            wrong_label[index].label = Some("WrongLongModeDef");
            mode_near_misses.push(wrong_label);
            let mut wrong_spelling = exact_modes();
            wrong_spelling[index].pattern = "OtherLongMode";
            mode_near_misses.push(wrong_spelling);
            let mut recovered = exact_modes();
            recovered[index].recovered = true;
            mode_near_misses.push(recovered);
            let mut contextual = exact_modes();
            contextual[index].local_context = true;
            mode_near_misses.push(contextual);
            let mut parameterized = exact_modes();
            parameterized[index].parameterized_pattern = true;
            mode_near_misses.push(parameterized);
            let mut args = exact_modes();
            args[index].rhs_shape = if index == 0 {
                ReserveTypeShape::QualifiedSymbolWithArgs("set")
            } else {
                ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[index - 1])
            };
            mode_near_misses.push(args);
            let mut attrs = exact_modes();
            attrs[index].rhs_shape = if index == 0 {
                ReserveTypeShape::AttributedSet
            } else {
                ReserveTypeShape::AttributedQualifiedSymbol(mode_names[index - 1])
            };
            mode_near_misses.push(attrs);
        }
        for (index, modes) in mode_near_misses.into_iter().enumerate() {
            assert_extraction_gap(
                mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    modes,
                    reserve(),
                    theorem,
                ),
                &format!("mode near miss {index}"),
            );
        }

        let mut source_near_misses = Vec::new();
        let mut deeper_modes = exact_modes();
        deeper_modes.push(mode_definition_with_label(
            "ChainMode7",
            "ChainMode7Def",
            ReserveTypeShape::QualifiedSymbol(mode_names[6]),
        ));
        source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            deeper_modes,
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("ChainMode7"),
            )],
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[6]),
                ..theorem
            },
        ));
        for bad_reserve in [
            ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[6]),
            ReserveTypeShape::AttributedQualifiedSymbol(mode_names[6]),
            ReserveTypeShape::Builtin("set"),
            ReserveTypeShape::Builtin("object"),
        ] {
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(vec!["x"], bad_reserve)],
                theorem,
            ));
        }
        source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            exact_modes(),
            vec![reserve_item(
                vec!["y"],
                ReserveTypeShape::QualifiedSymbol(mode_names[6]),
            )],
            theorem,
        ));
        source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            exact_modes(),
            vec![
                reserve_item(vec!["x"], ReserveTypeShape::QualifiedSymbol(mode_names[6])),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
            ],
            theorem,
        ));
        for bad_reserve in [
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol(mode_names[5]),
            )],
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("OtherLongMode"),
            )],
            vec![reserve_item(
                vec!["x", "y"],
                ReserveTypeShape::QualifiedSymbol(mode_names[6]),
            )],
        ] {
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                bad_reserve,
                theorem,
            ));
        }
        let theorem_near_misses = [
            IdentifierTypeAssertionTheoremSpec {
                label: "OtherPayloadBoundary",
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                subject: "y",
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::Builtin("set"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::Builtin("object"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[0]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[1]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[2]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[4]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[5]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[6]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol("OtherLongMode"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[3]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::AttributedQualifiedSymbol(mode_names[3]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                negated: true,
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                status: Some("open"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                recovered_label: true,
                ..theorem
            },
        ];
        for near_miss_theorem in theorem_near_misses {
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                near_miss_theorem,
            ));
        }
        source_near_misses.push(
            modes_then_empty_definition_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                theorem,
            ),
        );
        for (index, near_miss) in source_near_misses.into_iter().enumerate() {
            assert_extraction_gap(near_miss, &format!("source near miss {index}"));
        }

        for imported_index in 0..mode_names.len() {
            let local_modes = mode_names
                .iter()
                .enumerate()
                .filter(|(index, _)| *index != imported_index)
                .map(|(_, spelling)| (*spelling, SymbolKind::Mode))
                .collect::<Vec<_>>();
            for imported in [
                vec![(mode_names[imported_index], SymbolKind::Mode)],
                vec![
                    (mode_names[imported_index], SymbolKind::Mode),
                    (mode_names[imported_index], SymbolKind::Mode),
                ],
            ] {
                let imported_symbols = source_local_and_imported_symbols_env(
                    module.clone(),
                    &local_modes,
                    imported.as_slice(),
                );
                assert_eq!(
                    source_type_elaboration_detail_keys(&exact, module.clone(), &imported_symbols),
                    vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
                );
            }
        }
        let unrelated_import = source_local_and_imported_symbols_env(
            module.clone(),
            &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
            &[("UnrelatedLongMode", SymbolKind::Mode)],
        );
        assert!(
            super::extract_source_local_mode_long_chain_three_hop_asserted_head(
                &exact,
                module.clone(),
                &unrelated_import,
            )
            .is_some()
        );
        for (asserted_spelling, asserted_head_symbols) in [
            (
                "ImportedLongMode",
                source_local_and_imported_symbols_env(
                    module.clone(),
                    &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
                    &[("ImportedLongMode", SymbolKind::Mode)],
                ),
            ),
            (
                "AmbiguousLongMode",
                source_local_and_imported_symbols_env(
                    module.clone(),
                    &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
                    &[
                        ("AmbiguousLongMode", SymbolKind::Mode),
                        ("AmbiguousLongMode", SymbolKind::Mode),
                    ],
                ),
            ),
        ] {
            let near_miss = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(asserted_spelling),
                    ..theorem
                },
            );
            assert_eq!(
                source_type_elaboration_detail_keys(
                    &near_miss,
                    module.clone(),
                    &asserted_head_symbols,
                ),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
    }

    #[test]
    fn source_local_mode_long_chain_four_hop_asserted_head_consumes_seven_expansions() {
        let source_id = source_id(228);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("local_mode_long_chain_four_hop_asserted_head"),
        );
        let mode_names = [
            "BaseMode",
            "ChainMode1",
            "ChainMode2",
            "ChainMode3",
            "ChainMode4",
            "ChainMode5",
            "ChainMode6",
        ];
        let mode_labels = [
            "BaseModeDef",
            "ChainMode1Def",
            "ChainMode2Def",
            "ChainMode3Def",
            "ChainMode4Def",
            "ChainMode5Def",
            "ChainMode6Def",
        ];
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                (mode_names[0], SymbolKind::Mode),
                (mode_names[1], SymbolKind::Mode),
                (mode_names[2], SymbolKind::Mode),
                (mode_names[3], SymbolKind::Mode),
                (mode_names[4], SymbolKind::Mode),
                (mode_names[5], SymbolKind::Mode),
                (mode_names[6], SymbolKind::Mode),
                ("ChainMode7", SymbolKind::Mode),
                ("OtherLongMode", SymbolKind::Mode),
            ],
        );
        let theorem = IdentifierTypeAssertionTheoremSpec {
            status: None,
            label: "LongLocalModeFourHopAssertedHeadPayloadBoundary",
            subject: "x",
            asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[2]),
            negated: false,
            recovered_label: false,
        };
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    mode_names[0],
                    mode_labels[0],
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition_with_label(
                    mode_names[1],
                    mode_labels[1],
                    ReserveTypeShape::QualifiedSymbol(mode_names[0]),
                ),
                mode_definition_with_label(
                    mode_names[2],
                    mode_labels[2],
                    ReserveTypeShape::QualifiedSymbol(mode_names[1]),
                ),
                mode_definition_with_label(
                    mode_names[3],
                    mode_labels[3],
                    ReserveTypeShape::QualifiedSymbol(mode_names[2]),
                ),
                mode_definition_with_label(
                    mode_names[4],
                    mode_labels[4],
                    ReserveTypeShape::QualifiedSymbol(mode_names[3]),
                ),
                mode_definition_with_label(
                    mode_names[5],
                    mode_labels[5],
                    ReserveTypeShape::QualifiedSymbol(mode_names[4]),
                ),
                mode_definition_with_label(
                    mode_names[6],
                    mode_labels[6],
                    ReserveTypeShape::QualifiedSymbol(mode_names[5]),
                ),
            ]
        };
        let reserve = || {
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol(mode_names[6]),
            )]
        };
        let exact = mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            exact_modes(),
            reserve(),
            theorem,
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        assert!(
            extract_source_local_mode_long_chain_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 175 must reject the Task 228 four-hop source"
        );
        assert!(
            super::extract_source_local_mode_long_chain_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 199 must reject the Task 228 four-hop source"
        );
        assert!(
            super::extract_source_local_mode_long_chain_two_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 224 must reject the Task 228 four-hop source"
        );
        assert!(
            super::extract_source_local_object_mode_long_chain_two_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 225 must reject the Task 228 set-terminal four-hop source"
        );
        assert!(
            super::extract_source_local_mode_long_chain_three_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 226 must reject the Task 228 four-hop source"
        );
        assert!(
            super::extract_source_local_object_mode_long_chain_three_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 227 must reject the Task 228 set-terminal four-hop source"
        );
        let preexisting_owner_results = [
            extract_source_reserved_variable_type_assertion(&exact, module.clone(), &symbols),
            super::extract_source_reserved_object_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_local_object_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_chained_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_chained_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_chained_local_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_chained_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_chained_local_mode_radix_asserted_head(&exact, module.clone(), &symbols),
            extract_source_chained_local_object_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_two_edge_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_object_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_three_edge_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_object_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_four_edge_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_four_edge_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            super::extract_source_four_edge_local_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            super::extract_source_four_edge_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_four_edge_local_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_four_edge_local_object_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_mode_long_chain_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_object_mode_long_chain_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            super::extract_source_local_mode_long_chain_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            super::extract_source_local_object_mode_long_chain_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
        ];
        assert_eq!(preexisting_owner_results.len(), 34);
        assert!(
            preexisting_owner_results
                .into_iter()
                .all(|result| result.is_none()),
            "all 34 legacy type-assertion owner routes must reject Task 228"
        );
        let payload = super::extract_source_local_mode_long_chain_four_hop_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact seven-expansion four-hop source should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 7);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(payload.subject_lookup_ordinal, 1);
        let source_binding = &payload.reserve.bridge.bindings()[0];
        assert_eq!(source_binding.type_spelling, mode_names[6]);
        assert_eq!(payload.asserted_type.spelling, mode_names[2]);
        assert_ne!(payload.asserted_type.head, source_binding.type_head);
        assert!(matches!(source_binding.type_head, TypeHeadInput::Symbol(_)));
        assert_ne!(payload.asserted_type_site, payload.subject_site);
        assert_ne!(payload.asserted_type.range, source_binding.type_range);
        let binding_expansion = payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[6]))
            .map(|(_, expansion)| expansion)
            .expect("ChainMode6 binding expansion should exist");
        assert_eq!(binding_expansion.radix.spelling, mode_names[5]);
        assert_ne!(binding_expansion.radix.head, payload.asserted_type.head);
        let TypeHeadInput::Symbol(first_intermediate_symbol) = &binding_expansion.radix.head else {
            panic!("ChainMode5 first relation intermediate should resolve to a symbol");
        };
        let first_intermediate_expansion = payload
            .reserve
            .mode_expansions
            .get(first_intermediate_symbol)
            .expect("ChainMode5 first intermediate expansion should exist");
        assert_eq!(first_intermediate_expansion.radix.spelling, mode_names[4]);
        assert_ne!(
            first_intermediate_expansion.radix.head,
            payload.asserted_type.head
        );
        let TypeHeadInput::Symbol(second_intermediate_symbol) =
            &first_intermediate_expansion.radix.head
        else {
            panic!("ChainMode4 second relation intermediate should resolve to a symbol");
        };
        let second_intermediate_expansion = payload
            .reserve
            .mode_expansions
            .get(second_intermediate_symbol)
            .expect("ChainMode4 second intermediate expansion should exist");
        assert_eq!(second_intermediate_expansion.radix.spelling, mode_names[3]);
        assert_ne!(
            second_intermediate_expansion.radix.head,
            payload.asserted_type.head
        );
        let TypeHeadInput::Symbol(third_intermediate_symbol) =
            &second_intermediate_expansion.radix.head
        else {
            panic!("ChainMode3 third relation intermediate should resolve to a symbol");
        };
        let third_intermediate_expansion = payload
            .reserve
            .mode_expansions
            .get(third_intermediate_symbol)
            .expect("ChainMode3 third intermediate expansion should exist");
        assert_eq!(third_intermediate_expansion.radix.spelling, mode_names[2]);
        assert_eq!(
            third_intermediate_expansion.radix.head,
            payload.asserted_type.head
        );

        let output = super::source_local_mode_long_chain_four_hop_asserted_head_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact seven-expansion asserted-head source should reach checker");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 228 checker payload invariants should hold");
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert_eq!(output.subject_result_input.spelling, mode_names[6]);
        assert_eq!(output.asserted_type_input.spelling, mode_names[2]);
        assert_ne!(
            output.subject_result_input.head,
            output.asserted_type_input.head
        );
        assert!(matches!(
            output.asserted_type_input.head,
            TypeHeadInput::Symbol(_)
        ));
        assert_ne!(
            output.subject_result_input.site,
            output.asserted_type_input.site
        );
        assert_ne!(
            output.subject_result_input.source_range,
            output.asserted_type_input.source_range
        );
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[0]))
            .map(|(_, expansion)| &expansion.radix)
            .expect("BaseMode terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("one normalized builtin-set type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
        let (_, term) = output.term_formula.terms().iter().next().unwrap();
        assert_eq!(term.status, TermStatus::Inferred);
        assert!(term.deferred.is_empty());
        let (_, formula) = output.term_formula.formulas().iter().next().unwrap();
        assert_eq!(formula.kind, FormulaKind::TypeAssertion);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert!(formula.expected_types.is_empty());
        assert!(formula.candidate_set.is_none());
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
        assert!(output.term_formula.candidate_sets().is_empty());
        assert!(output.term_formula.facts().is_empty());
        assert!(output.term_formula.diagnostics().is_empty());

        let assert_invalid_output = |invalid| {
            let invalid_result =
                assert_source_reserved_variable_type_assertion_output(&invalid).map(|()| invalid);
            assert_eq!(
                source_reserved_variable_type_assertion_result_detail_keys(
                    invalid_result,
                    super::TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_FOUR_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
                ),
                vec![
                    super::TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_FOUR_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY
                        .to_owned()
                ]
            );
        };
        for removed in mode_names {
            let mut invalid = super::source_local_mode_long_chain_four_hop_asserted_head_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .unwrap();
            invalid
                .payload
                .reserve
                .mode_expansions
                .retain(|symbol, _| source_mode_symbol_spelling(symbol) != Some(removed));
            assert_invalid_output(invalid);
        }
        let exact_output = || {
            super::source_local_mode_long_chain_four_hop_asserted_head_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .unwrap()
        };
        let mut corruptions = Vec::new();
        let mut invalid = exact_output();
        invalid.subject_binding = BindingId::new(1);
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.payload.subject_lookup_ordinal = 2;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.spelling = mode_names[5].to_owned();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.spelling = "OtherLongMode".to_owned();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.head = invalid.subject_result_input.head.clone();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.head = TypeHeadInput::BuiltinObject;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.head = TypeHeadInput::BuiltinObject;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.source_range = invalid.asserted_type_input.source_range;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.site = invalid.asserted_type_input.site.clone();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.source_range = invalid.subject_result_input.source_range;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.site = invalid.subject_result_input.site.clone();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        let binding_head = invalid.subject_result_input.head.clone();
        let (_, binding_expansion) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[6]))
            .unwrap();
        binding_expansion.radix.head = binding_head;
        corruptions.push(invalid);
        for mode_name in &mode_names[1..] {
            let mut invalid = exact_output();
            let (_, expansion) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(*mode_name))
                .unwrap();
            expansion.radix.spelling = "object".to_owned();
            expansion.radix.head = TypeHeadInput::BuiltinObject;
            corruptions.push(invalid);
        }
        let mut invalid = exact_output();
        let (_, base_expansion) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[0]))
            .unwrap();
        base_expansion.radix.source_range = range(source_id, 0, 1);
        corruptions.push(invalid);
        let mut invalid = exact_output();
        let (_, base_expansion) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[0]))
            .unwrap();
        base_expansion.radix.spelling = "object".to_owned();
        base_expansion.radix.head = TypeHeadInput::BuiltinObject;
        corruptions.push(invalid);
        for invalid in corruptions {
            assert_invalid_output(invalid);
        }
        assert_source_reserved_variable_type_assertion_output(&exact_output())
            .expect("corrupting cloned Task 228 outputs must not mutate the exact output");

        let assert_extraction_gap = |ast, context: &str| {
            assert_eq!(
                source_type_elaboration_detail_keys(&ast, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()],
                "{context}"
            );
        };
        fn next_permutation(order: &mut [usize; 7]) -> bool {
            let Some(pivot) = (0..order.len() - 1)
                .rev()
                .find(|&index| order[index] < order[index + 1])
            else {
                return false;
            };
            let successor = (pivot + 1..order.len())
                .rev()
                .find(|&index| order[pivot] < order[index])
                .expect("a lexicographic successor must exist after the pivot");
            order.swap(pivot, successor);
            order[pivot + 1..].reverse();
            true
        }
        let ordered = exact_modes();
        let public_dispatch_orders = [
            [1, 0, 2, 3, 4, 5, 6],
            [0, 2, 1, 3, 4, 5, 6],
            [0, 1, 3, 2, 4, 5, 6],
            [0, 1, 2, 4, 3, 5, 6],
            [0, 1, 2, 3, 5, 4, 6],
            [0, 1, 2, 3, 4, 6, 5],
            [1, 2, 3, 4, 5, 6, 0],
            [6, 5, 4, 3, 2, 1, 0],
        ];
        let mut order = [0, 1, 2, 3, 4, 5, 6];
        let mut permutation_count = 0;
        let mut public_dispatch_representative_count = 0;
        while next_permutation(&mut order) {
            permutation_count += 1;
            let permuted = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                vec![
                    ordered[order[0]],
                    ordered[order[1]],
                    ordered[order[2]],
                    ordered[order[3]],
                    ordered[order[4]],
                    ordered[order[5]],
                    ordered[order[6]],
                ],
                reserve(),
                theorem,
            );
            assert!(
                super::extract_source_local_mode_long_chain_four_hop_asserted_head(
                    &permuted,
                    module.clone(),
                    &symbols,
                )
                .is_none(),
                "definition permutation {order:?} must not reach Task 228"
            );
            if public_dispatch_orders.contains(&order) {
                public_dispatch_representative_count += 1;
                assert_extraction_gap(
                    permuted,
                    &format!("public definition permutation {order:?}"),
                );
            }
        }
        assert_eq!(permutation_count, 5_039);
        assert_eq!(public_dispatch_representative_count, 8);
        let mut mode_near_misses = vec![
            Vec::<ModeDefinitionSpec>::new(),
            {
                let mut modes = exact_modes();
                modes.push(modes[0]);
                modes
            },
            {
                let mut modes = exact_modes();
                modes.push(mode_definition_with_label(
                    "UnusedLongMode",
                    "UnusedLongModeDef",
                    ReserveTypeShape::QualifiedSymbol(mode_names[6]),
                ));
                modes
            },
        ];
        for index in 0..mode_names.len() {
            let mut missing = exact_modes();
            missing.remove(index);
            mode_near_misses.push(missing);
            let mut duplicate = exact_modes();
            duplicate.push(duplicate[index]);
            mode_near_misses.push(duplicate);
            let mut malformed = exact_modes();
            malformed[index].rhs_shape = if index == 0 {
                ReserveTypeShape::Builtin("object")
            } else {
                ReserveTypeShape::QualifiedSymbol(mode_names[(index + 1) % mode_names.len()])
            };
            mode_near_misses.push(malformed);
            let mut wrong_label = exact_modes();
            wrong_label[index].label = Some("WrongLongModeDef");
            mode_near_misses.push(wrong_label);
            let mut wrong_spelling = exact_modes();
            wrong_spelling[index].pattern = "OtherLongMode";
            mode_near_misses.push(wrong_spelling);
            let mut recovered = exact_modes();
            recovered[index].recovered = true;
            mode_near_misses.push(recovered);
            let mut contextual = exact_modes();
            contextual[index].local_context = true;
            mode_near_misses.push(contextual);
            let mut parameterized = exact_modes();
            parameterized[index].parameterized_pattern = true;
            mode_near_misses.push(parameterized);
            let mut args = exact_modes();
            args[index].rhs_shape = if index == 0 {
                ReserveTypeShape::QualifiedSymbolWithArgs("set")
            } else {
                ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[index - 1])
            };
            mode_near_misses.push(args);
            let mut attrs = exact_modes();
            attrs[index].rhs_shape = if index == 0 {
                ReserveTypeShape::AttributedSet
            } else {
                ReserveTypeShape::AttributedQualifiedSymbol(mode_names[index - 1])
            };
            mode_near_misses.push(attrs);
        }
        for (index, modes) in mode_near_misses.into_iter().enumerate() {
            assert_extraction_gap(
                mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    modes,
                    reserve(),
                    theorem,
                ),
                &format!("mode near miss {index}"),
            );
        }

        let mut source_near_misses = Vec::new();
        let mut deeper_modes = exact_modes();
        deeper_modes.push(mode_definition_with_label(
            "ChainMode7",
            "ChainMode7Def",
            ReserveTypeShape::QualifiedSymbol(mode_names[6]),
        ));
        source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            deeper_modes,
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("ChainMode7"),
            )],
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[6]),
                ..theorem
            },
        ));
        for bad_reserve in [
            ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[6]),
            ReserveTypeShape::AttributedQualifiedSymbol(mode_names[6]),
            ReserveTypeShape::Builtin("set"),
            ReserveTypeShape::Builtin("object"),
        ] {
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(vec!["x"], bad_reserve)],
                theorem,
            ));
        }
        source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            exact_modes(),
            vec![reserve_item(
                vec!["y"],
                ReserveTypeShape::QualifiedSymbol(mode_names[6]),
            )],
            theorem,
        ));
        source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            exact_modes(),
            vec![
                reserve_item(vec!["x"], ReserveTypeShape::QualifiedSymbol(mode_names[6])),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
            ],
            theorem,
        ));
        for bad_reserve in [
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol(mode_names[5]),
            )],
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("OtherLongMode"),
            )],
            vec![reserve_item(
                vec!["x", "y"],
                ReserveTypeShape::QualifiedSymbol(mode_names[6]),
            )],
        ] {
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                bad_reserve,
                theorem,
            ));
        }
        let theorem_near_misses = [
            IdentifierTypeAssertionTheoremSpec {
                label: "OtherPayloadBoundary",
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                subject: "y",
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::Builtin("set"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::Builtin("object"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[0]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[1]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[3]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[4]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[5]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[6]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol("OtherLongMode"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[2]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::AttributedQualifiedSymbol(mode_names[2]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                negated: true,
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                status: Some("open"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                recovered_label: true,
                ..theorem
            },
        ];
        for near_miss_theorem in theorem_near_misses {
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                near_miss_theorem,
            ));
        }
        source_near_misses.push(
            modes_then_empty_definition_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                theorem,
            ),
        );
        for (index, near_miss) in source_near_misses.into_iter().enumerate() {
            assert_extraction_gap(near_miss, &format!("source near miss {index}"));
        }

        for imported_index in 0..mode_names.len() {
            let local_modes = mode_names
                .iter()
                .enumerate()
                .filter(|(index, _)| *index != imported_index)
                .map(|(_, spelling)| (*spelling, SymbolKind::Mode))
                .collect::<Vec<_>>();
            for imported in [
                vec![(mode_names[imported_index], SymbolKind::Mode)],
                vec![
                    (mode_names[imported_index], SymbolKind::Mode),
                    (mode_names[imported_index], SymbolKind::Mode),
                ],
            ] {
                let imported_symbols = source_local_and_imported_symbols_env(
                    module.clone(),
                    &local_modes,
                    imported.as_slice(),
                );
                assert_eq!(
                    source_type_elaboration_detail_keys(&exact, module.clone(), &imported_symbols),
                    vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
                );
            }
        }
        let unrelated_import = source_local_and_imported_symbols_env(
            module.clone(),
            &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
            &[("UnrelatedLongMode", SymbolKind::Mode)],
        );
        assert!(
            super::extract_source_local_mode_long_chain_four_hop_asserted_head(
                &exact,
                module.clone(),
                &unrelated_import,
            )
            .is_some()
        );
        for (asserted_spelling, asserted_head_symbols) in [
            (
                "ImportedLongMode",
                source_local_and_imported_symbols_env(
                    module.clone(),
                    &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
                    &[("ImportedLongMode", SymbolKind::Mode)],
                ),
            ),
            (
                "AmbiguousLongMode",
                source_local_and_imported_symbols_env(
                    module.clone(),
                    &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
                    &[
                        ("AmbiguousLongMode", SymbolKind::Mode),
                        ("AmbiguousLongMode", SymbolKind::Mode),
                    ],
                ),
            ),
        ] {
            let near_miss = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(asserted_spelling),
                    ..theorem
                },
            );
            assert_eq!(
                source_type_elaboration_detail_keys(
                    &near_miss,
                    module.clone(),
                    &asserted_head_symbols,
                ),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
    }
    #[test]
    fn source_local_mode_long_chain_five_hop_asserted_head_consumes_seven_expansions() {
        let source_id = source_id(230);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("local_mode_long_chain_five_hop_asserted_head"),
        );
        let mode_names = [
            "BaseMode",
            "ChainMode1",
            "ChainMode2",
            "ChainMode3",
            "ChainMode4",
            "ChainMode5",
            "ChainMode6",
        ];
        let mode_labels = [
            "BaseModeDef",
            "ChainMode1Def",
            "ChainMode2Def",
            "ChainMode3Def",
            "ChainMode4Def",
            "ChainMode5Def",
            "ChainMode6Def",
        ];
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                (mode_names[0], SymbolKind::Mode),
                (mode_names[1], SymbolKind::Mode),
                (mode_names[2], SymbolKind::Mode),
                (mode_names[3], SymbolKind::Mode),
                (mode_names[4], SymbolKind::Mode),
                (mode_names[5], SymbolKind::Mode),
                (mode_names[6], SymbolKind::Mode),
                ("ChainMode7", SymbolKind::Mode),
                ("OtherLongMode", SymbolKind::Mode),
            ],
        );
        let theorem = IdentifierTypeAssertionTheoremSpec {
            status: None,
            label: "LongLocalModeFiveHopAssertedHeadPayloadBoundary",
            subject: "x",
            asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[1]),
            negated: false,
            recovered_label: false,
        };
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    mode_names[0],
                    mode_labels[0],
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition_with_label(
                    mode_names[1],
                    mode_labels[1],
                    ReserveTypeShape::QualifiedSymbol(mode_names[0]),
                ),
                mode_definition_with_label(
                    mode_names[2],
                    mode_labels[2],
                    ReserveTypeShape::QualifiedSymbol(mode_names[1]),
                ),
                mode_definition_with_label(
                    mode_names[3],
                    mode_labels[3],
                    ReserveTypeShape::QualifiedSymbol(mode_names[2]),
                ),
                mode_definition_with_label(
                    mode_names[4],
                    mode_labels[4],
                    ReserveTypeShape::QualifiedSymbol(mode_names[3]),
                ),
                mode_definition_with_label(
                    mode_names[5],
                    mode_labels[5],
                    ReserveTypeShape::QualifiedSymbol(mode_names[4]),
                ),
                mode_definition_with_label(
                    mode_names[6],
                    mode_labels[6],
                    ReserveTypeShape::QualifiedSymbol(mode_names[5]),
                ),
            ]
        };
        let reserve = || {
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol(mode_names[6]),
            )]
        };
        let exact = mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            exact_modes(),
            reserve(),
            theorem,
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        assert!(
            extract_source_local_mode_long_chain_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 175 must reject the Task 230 five-hop source"
        );
        assert!(
            super::extract_source_local_mode_long_chain_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 199 must reject the Task 230 five-hop source"
        );
        assert!(
            super::extract_source_local_mode_long_chain_two_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 224 must reject the Task 230 five-hop source"
        );
        assert!(
            super::extract_source_local_object_mode_long_chain_two_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 225 must reject the Task 230 set-terminal five-hop source"
        );
        assert!(
            super::extract_source_local_mode_long_chain_three_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 226 must reject the Task 230 five-hop source"
        );
        assert!(
            super::extract_source_local_object_mode_long_chain_three_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 227 must reject the Task 230 set-terminal five-hop source"
        );
        assert!(
            super::extract_source_local_mode_long_chain_four_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 228 must reject the Task 230 five-hop source"
        );
        assert!(
            super::extract_source_local_object_mode_long_chain_four_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 229 must reject the Task 230 set-terminal five-hop source"
        );
        let preexisting_owner_results = [
            extract_source_reserved_variable_type_assertion(&exact, module.clone(), &symbols),
            super::extract_source_reserved_object_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_local_object_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_chained_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_chained_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_chained_local_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_chained_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_chained_local_mode_radix_asserted_head(&exact, module.clone(), &symbols),
            extract_source_chained_local_object_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_two_edge_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_object_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_three_edge_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_object_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_four_edge_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_four_edge_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            super::extract_source_four_edge_local_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            super::extract_source_four_edge_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_four_edge_local_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_four_edge_local_object_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_mode_long_chain_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_object_mode_long_chain_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            super::extract_source_local_mode_long_chain_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            super::extract_source_local_object_mode_long_chain_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
        ];
        assert_eq!(preexisting_owner_results.len(), 34);
        assert!(
            preexisting_owner_results
                .into_iter()
                .all(|result| result.is_none()),
            "all 34 legacy type-assertion owner routes must reject Task 230"
        );
        let payload = super::extract_source_local_mode_long_chain_five_hop_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact seven-expansion five-hop source should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 7);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(payload.subject_lookup_ordinal, 1);
        let source_binding = &payload.reserve.bridge.bindings()[0];
        assert_eq!(source_binding.type_spelling, mode_names[6]);
        assert_eq!(payload.asserted_type.spelling, mode_names[1]);
        assert_ne!(payload.asserted_type.head, source_binding.type_head);
        assert!(matches!(source_binding.type_head, TypeHeadInput::Symbol(_)));
        assert_ne!(payload.asserted_type_site, payload.subject_site);
        assert_ne!(payload.asserted_type.range, source_binding.type_range);
        let binding_expansion = payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[6]))
            .map(|(_, expansion)| expansion)
            .expect("ChainMode6 binding expansion should exist");
        assert_eq!(binding_expansion.radix.spelling, mode_names[5]);
        assert_ne!(binding_expansion.radix.head, payload.asserted_type.head);
        let TypeHeadInput::Symbol(first_intermediate_symbol) = &binding_expansion.radix.head else {
            panic!("ChainMode5 first relation intermediate should resolve to a symbol");
        };
        let first_intermediate_expansion = payload
            .reserve
            .mode_expansions
            .get(first_intermediate_symbol)
            .expect("ChainMode5 first intermediate expansion should exist");
        assert_eq!(first_intermediate_expansion.radix.spelling, mode_names[4]);
        assert_ne!(
            first_intermediate_expansion.radix.head,
            payload.asserted_type.head
        );
        let TypeHeadInput::Symbol(second_intermediate_symbol) =
            &first_intermediate_expansion.radix.head
        else {
            panic!("ChainMode4 second relation intermediate should resolve to a symbol");
        };
        let second_intermediate_expansion = payload
            .reserve
            .mode_expansions
            .get(second_intermediate_symbol)
            .expect("ChainMode4 second intermediate expansion should exist");
        assert_eq!(second_intermediate_expansion.radix.spelling, mode_names[3]);
        assert_ne!(
            second_intermediate_expansion.radix.head,
            payload.asserted_type.head
        );
        let TypeHeadInput::Symbol(third_intermediate_symbol) =
            &second_intermediate_expansion.radix.head
        else {
            panic!("ChainMode3 third relation intermediate should resolve to a symbol");
        };
        let third_intermediate_expansion = payload
            .reserve
            .mode_expansions
            .get(third_intermediate_symbol)
            .expect("ChainMode3 third intermediate expansion should exist");
        assert_eq!(third_intermediate_expansion.radix.spelling, mode_names[2]);
        assert_ne!(
            third_intermediate_expansion.radix.head,
            payload.asserted_type.head
        );
        let TypeHeadInput::Symbol(fourth_intermediate_symbol) =
            &third_intermediate_expansion.radix.head
        else {
            panic!("ChainMode2 fourth relation intermediate should resolve to a symbol");
        };
        let fourth_intermediate_expansion = payload
            .reserve
            .mode_expansions
            .get(fourth_intermediate_symbol)
            .expect("ChainMode2 fourth intermediate expansion should exist");
        assert_eq!(fourth_intermediate_expansion.radix.spelling, mode_names[1]);
        assert_eq!(
            fourth_intermediate_expansion.radix.head,
            payload.asserted_type.head
        );

        let output = super::source_local_mode_long_chain_five_hop_asserted_head_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact seven-expansion asserted-head source should reach checker");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 230 checker payload invariants should hold");
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert_eq!(output.subject_result_input.spelling, mode_names[6]);
        assert_eq!(output.asserted_type_input.spelling, mode_names[1]);
        assert_ne!(
            output.subject_result_input.head,
            output.asserted_type_input.head
        );
        assert!(matches!(
            output.asserted_type_input.head,
            TypeHeadInput::Symbol(_)
        ));
        assert_ne!(
            output.subject_result_input.site,
            output.asserted_type_input.site
        );
        assert_ne!(
            output.subject_result_input.source_range,
            output.asserted_type_input.source_range
        );
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[0]))
            .map(|(_, expansion)| &expansion.radix)
            .expect("BaseMode terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("one normalized builtin-set type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
        let (_, term) = output.term_formula.terms().iter().next().unwrap();
        assert_eq!(term.status, TermStatus::Inferred);
        assert!(term.deferred.is_empty());
        let (_, formula) = output.term_formula.formulas().iter().next().unwrap();
        assert_eq!(formula.kind, FormulaKind::TypeAssertion);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert!(formula.expected_types.is_empty());
        assert!(formula.candidate_set.is_none());
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
        assert!(output.term_formula.candidate_sets().is_empty());
        assert!(output.term_formula.facts().is_empty());
        assert!(output.term_formula.diagnostics().is_empty());

        let assert_invalid_output = |invalid| {
            let invalid_result =
                assert_source_reserved_variable_type_assertion_output(&invalid).map(|()| invalid);
            assert_eq!(
                source_reserved_variable_type_assertion_result_detail_keys(
                    invalid_result,
                    super::TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_FIVE_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
                ),
                vec![
                    super::TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_FIVE_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY
                        .to_owned()
                ]
            );
        };
        for removed in mode_names {
            let mut invalid = super::source_local_mode_long_chain_five_hop_asserted_head_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .unwrap();
            invalid
                .payload
                .reserve
                .mode_expansions
                .retain(|symbol, _| source_mode_symbol_spelling(symbol) != Some(removed));
            assert_invalid_output(invalid);
        }
        let exact_output = || {
            super::source_local_mode_long_chain_five_hop_asserted_head_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .unwrap()
        };
        let mut corruptions = Vec::new();
        let mut invalid = exact_output();
        invalid.subject_binding = BindingId::new(1);
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.payload.subject_lookup_ordinal = 2;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.spelling = mode_names[5].to_owned();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.spelling = "OtherLongMode".to_owned();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.head = invalid.subject_result_input.head.clone();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.head = TypeHeadInput::BuiltinObject;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.head = TypeHeadInput::BuiltinObject;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.source_range = invalid.asserted_type_input.source_range;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.site = invalid.asserted_type_input.site.clone();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.source_range = invalid.subject_result_input.source_range;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.site = invalid.subject_result_input.site.clone();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        let binding_head = invalid.subject_result_input.head.clone();
        let (_, binding_expansion) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[6]))
            .unwrap();
        binding_expansion.radix.head = binding_head;
        corruptions.push(invalid);
        for mode_name in &mode_names[1..] {
            let mut invalid = exact_output();
            let (_, expansion) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(*mode_name))
                .unwrap();
            expansion.radix.spelling = "object".to_owned();
            expansion.radix.head = TypeHeadInput::BuiltinObject;
            corruptions.push(invalid);
        }
        let mut invalid = exact_output();
        let (_, base_expansion) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[0]))
            .unwrap();
        base_expansion.radix.source_range = range(source_id, 0, 1);
        corruptions.push(invalid);
        let mut invalid = exact_output();
        let (_, base_expansion) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[0]))
            .unwrap();
        base_expansion.radix.spelling = "object".to_owned();
        base_expansion.radix.head = TypeHeadInput::BuiltinObject;
        corruptions.push(invalid);
        for invalid in corruptions {
            assert_invalid_output(invalid);
        }
        assert_source_reserved_variable_type_assertion_output(&exact_output())
            .expect("corrupting cloned Task 230 outputs must not mutate the exact output");

        let assert_extraction_gap = |ast, context: &str| {
            assert_eq!(
                source_type_elaboration_detail_keys(&ast, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()],
                "{context}"
            );
        };
        fn next_permutation(order: &mut [usize; 7]) -> bool {
            let Some(pivot) = (0..order.len() - 1)
                .rev()
                .find(|&index| order[index] < order[index + 1])
            else {
                return false;
            };
            let successor = (pivot + 1..order.len())
                .rev()
                .find(|&index| order[pivot] < order[index])
                .expect("a lexicographic successor must exist after the pivot");
            order.swap(pivot, successor);
            order[pivot + 1..].reverse();
            true
        }
        let ordered = exact_modes();
        let public_dispatch_orders = [
            [1, 0, 2, 3, 4, 5, 6],
            [0, 2, 1, 3, 4, 5, 6],
            [0, 1, 3, 2, 4, 5, 6],
            [0, 1, 2, 4, 3, 5, 6],
            [0, 1, 2, 3, 5, 4, 6],
            [0, 1, 2, 3, 4, 6, 5],
            [1, 2, 3, 4, 5, 6, 0],
            [6, 5, 4, 3, 2, 1, 0],
        ];
        let mut order = [0, 1, 2, 3, 4, 5, 6];
        let mut permutation_count = 0;
        let mut public_dispatch_representative_count = 0;
        while next_permutation(&mut order) {
            permutation_count += 1;
            let permuted = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                vec![
                    ordered[order[0]],
                    ordered[order[1]],
                    ordered[order[2]],
                    ordered[order[3]],
                    ordered[order[4]],
                    ordered[order[5]],
                    ordered[order[6]],
                ],
                reserve(),
                theorem,
            );
            assert!(
                super::extract_source_local_mode_long_chain_five_hop_asserted_head(
                    &permuted,
                    module.clone(),
                    &symbols,
                )
                .is_none(),
                "definition permutation {order:?} must not reach Task 230"
            );
            if public_dispatch_orders.contains(&order) {
                public_dispatch_representative_count += 1;
                assert_extraction_gap(
                    permuted,
                    &format!("public definition permutation {order:?}"),
                );
            }
        }
        assert_eq!(permutation_count, 5_039);
        assert_eq!(public_dispatch_representative_count, 8);
        let mut mode_near_misses = vec![
            Vec::<ModeDefinitionSpec>::new(),
            {
                let mut modes = exact_modes();
                modes.push(modes[0]);
                modes
            },
            {
                let mut modes = exact_modes();
                modes.push(mode_definition_with_label(
                    "UnusedLongMode",
                    "UnusedLongModeDef",
                    ReserveTypeShape::QualifiedSymbol(mode_names[6]),
                ));
                modes
            },
        ];
        for index in 0..mode_names.len() {
            let mut missing = exact_modes();
            missing.remove(index);
            mode_near_misses.push(missing);
            let mut duplicate = exact_modes();
            duplicate.push(duplicate[index]);
            mode_near_misses.push(duplicate);
            let mut malformed = exact_modes();
            malformed[index].rhs_shape = if index == 0 {
                ReserveTypeShape::Builtin("object")
            } else {
                ReserveTypeShape::QualifiedSymbol(mode_names[(index + 1) % mode_names.len()])
            };
            mode_near_misses.push(malformed);
            let mut wrong_label = exact_modes();
            wrong_label[index].label = Some("WrongLongModeDef");
            mode_near_misses.push(wrong_label);
            let mut wrong_spelling = exact_modes();
            wrong_spelling[index].pattern = "OtherLongMode";
            mode_near_misses.push(wrong_spelling);
            let mut recovered = exact_modes();
            recovered[index].recovered = true;
            mode_near_misses.push(recovered);
            let mut contextual = exact_modes();
            contextual[index].local_context = true;
            mode_near_misses.push(contextual);
            let mut parameterized = exact_modes();
            parameterized[index].parameterized_pattern = true;
            mode_near_misses.push(parameterized);
            let mut args = exact_modes();
            args[index].rhs_shape = if index == 0 {
                ReserveTypeShape::QualifiedSymbolWithArgs("set")
            } else {
                ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[index - 1])
            };
            mode_near_misses.push(args);
            let mut attrs = exact_modes();
            attrs[index].rhs_shape = if index == 0 {
                ReserveTypeShape::AttributedSet
            } else {
                ReserveTypeShape::AttributedQualifiedSymbol(mode_names[index - 1])
            };
            mode_near_misses.push(attrs);
        }
        for (index, modes) in mode_near_misses.into_iter().enumerate() {
            assert_extraction_gap(
                mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    modes,
                    reserve(),
                    theorem,
                ),
                &format!("mode near miss {index}"),
            );
        }

        let mut source_near_misses = Vec::new();
        let mut deeper_modes = exact_modes();
        deeper_modes.push(mode_definition_with_label(
            "ChainMode7",
            "ChainMode7Def",
            ReserveTypeShape::QualifiedSymbol(mode_names[6]),
        ));
        source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            deeper_modes,
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("ChainMode7"),
            )],
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[6]),
                ..theorem
            },
        ));
        for bad_reserve in [
            ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[6]),
            ReserveTypeShape::AttributedQualifiedSymbol(mode_names[6]),
            ReserveTypeShape::Builtin("set"),
            ReserveTypeShape::Builtin("object"),
        ] {
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(vec!["x"], bad_reserve)],
                theorem,
            ));
        }
        source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            exact_modes(),
            vec![reserve_item(
                vec!["y"],
                ReserveTypeShape::QualifiedSymbol(mode_names[6]),
            )],
            theorem,
        ));
        source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            exact_modes(),
            vec![
                reserve_item(vec!["x"], ReserveTypeShape::QualifiedSymbol(mode_names[6])),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
            ],
            theorem,
        ));
        for bad_reserve in [
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol(mode_names[5]),
            )],
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("OtherLongMode"),
            )],
            vec![reserve_item(
                vec!["x", "y"],
                ReserveTypeShape::QualifiedSymbol(mode_names[6]),
            )],
        ] {
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                bad_reserve,
                theorem,
            ));
        }
        let theorem_near_misses = [
            IdentifierTypeAssertionTheoremSpec {
                label: "OtherPayloadBoundary",
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                subject: "y",
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::Builtin("set"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::Builtin("object"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[0]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[2]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[3]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[4]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[5]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[6]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol("OtherLongMode"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[1]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::AttributedQualifiedSymbol(mode_names[1]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                negated: true,
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                status: Some("open"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                recovered_label: true,
                ..theorem
            },
        ];
        for near_miss_theorem in theorem_near_misses {
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                near_miss_theorem,
            ));
        }
        source_near_misses.push(
            modes_then_empty_definition_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                theorem,
            ),
        );
        for (index, near_miss) in source_near_misses.into_iter().enumerate() {
            assert_extraction_gap(near_miss, &format!("source near miss {index}"));
        }

        for imported_index in 0..mode_names.len() {
            let local_modes = mode_names
                .iter()
                .enumerate()
                .filter(|(index, _)| *index != imported_index)
                .map(|(_, spelling)| (*spelling, SymbolKind::Mode))
                .collect::<Vec<_>>();
            for imported in [
                vec![(mode_names[imported_index], SymbolKind::Mode)],
                vec![
                    (mode_names[imported_index], SymbolKind::Mode),
                    (mode_names[imported_index], SymbolKind::Mode),
                ],
            ] {
                let imported_symbols = source_local_and_imported_symbols_env(
                    module.clone(),
                    &local_modes,
                    imported.as_slice(),
                );
                assert_eq!(
                    source_type_elaboration_detail_keys(&exact, module.clone(), &imported_symbols),
                    vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
                );
            }
        }
        let unrelated_import = source_local_and_imported_symbols_env(
            module.clone(),
            &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
            &[("UnrelatedLongMode", SymbolKind::Mode)],
        );
        assert!(
            super::extract_source_local_mode_long_chain_five_hop_asserted_head(
                &exact,
                module.clone(),
                &unrelated_import,
            )
            .is_some()
        );
        for (asserted_spelling, asserted_head_symbols) in [
            (
                "ImportedLongMode",
                source_local_and_imported_symbols_env(
                    module.clone(),
                    &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
                    &[("ImportedLongMode", SymbolKind::Mode)],
                ),
            ),
            (
                "AmbiguousLongMode",
                source_local_and_imported_symbols_env(
                    module.clone(),
                    &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
                    &[
                        ("AmbiguousLongMode", SymbolKind::Mode),
                        ("AmbiguousLongMode", SymbolKind::Mode),
                    ],
                ),
            ),
        ] {
            let near_miss = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(asserted_spelling),
                    ..theorem
                },
            );
            assert_eq!(
                source_type_elaboration_detail_keys(
                    &near_miss,
                    module.clone(),
                    &asserted_head_symbols,
                ),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
    }
    #[test]
    fn source_local_mode_long_chain_six_hop_asserted_head_consumes_seven_expansions() {
        let source_id = source_id(234);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("local_mode_long_chain_six_hop_asserted_head"),
        );
        let mode_names = [
            "BaseMode",
            "ChainMode1",
            "ChainMode2",
            "ChainMode3",
            "ChainMode4",
            "ChainMode5",
            "ChainMode6",
        ];
        let mode_labels = [
            "BaseModeDef",
            "ChainMode1Def",
            "ChainMode2Def",
            "ChainMode3Def",
            "ChainMode4Def",
            "ChainMode5Def",
            "ChainMode6Def",
        ];
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                (mode_names[0], SymbolKind::Mode),
                (mode_names[1], SymbolKind::Mode),
                (mode_names[2], SymbolKind::Mode),
                (mode_names[3], SymbolKind::Mode),
                (mode_names[4], SymbolKind::Mode),
                (mode_names[5], SymbolKind::Mode),
                (mode_names[6], SymbolKind::Mode),
                ("ChainMode7", SymbolKind::Mode),
                ("OtherLongMode", SymbolKind::Mode),
            ],
        );
        let theorem = IdentifierTypeAssertionTheoremSpec {
            status: None,
            label: "LongLocalModeSixHopAssertedHeadPayloadBoundary",
            subject: "x",
            asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[0]),
            negated: false,
            recovered_label: false,
        };
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    mode_names[0],
                    mode_labels[0],
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition_with_label(
                    mode_names[1],
                    mode_labels[1],
                    ReserveTypeShape::QualifiedSymbol(mode_names[0]),
                ),
                mode_definition_with_label(
                    mode_names[2],
                    mode_labels[2],
                    ReserveTypeShape::QualifiedSymbol(mode_names[1]),
                ),
                mode_definition_with_label(
                    mode_names[3],
                    mode_labels[3],
                    ReserveTypeShape::QualifiedSymbol(mode_names[2]),
                ),
                mode_definition_with_label(
                    mode_names[4],
                    mode_labels[4],
                    ReserveTypeShape::QualifiedSymbol(mode_names[3]),
                ),
                mode_definition_with_label(
                    mode_names[5],
                    mode_labels[5],
                    ReserveTypeShape::QualifiedSymbol(mode_names[4]),
                ),
                mode_definition_with_label(
                    mode_names[6],
                    mode_labels[6],
                    ReserveTypeShape::QualifiedSymbol(mode_names[5]),
                ),
            ]
        };
        let reserve = || {
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol(mode_names[6]),
            )]
        };
        let exact = mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            exact_modes(),
            reserve(),
            theorem,
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        assert!(
            extract_source_local_mode_long_chain_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 175 must reject the Task 234 six-hop source"
        );
        assert!(
            super::extract_source_local_mode_long_chain_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 199 must reject the Task 234 six-hop source"
        );
        assert!(
            super::extract_source_local_mode_long_chain_two_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 224 must reject the Task 234 six-hop source"
        );
        assert!(
            super::extract_source_local_object_mode_long_chain_two_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 225 must reject the Task 234 set-terminal six-hop source"
        );
        assert!(
            super::extract_source_local_mode_long_chain_three_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 226 must reject the Task 234 six-hop source"
        );
        assert!(
            super::extract_source_local_object_mode_long_chain_three_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 227 must reject the Task 234 set-terminal six-hop source"
        );
        assert!(
            super::extract_source_local_mode_long_chain_four_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 228 must reject the Task 234 six-hop source"
        );
        assert!(
            super::extract_source_local_object_mode_long_chain_four_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 229 must reject the Task 234 set-terminal six-hop source"
        );
        assert!(
            super::extract_source_local_mode_long_chain_five_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 230 must reject the Task 234 six-hop source"
        );
        assert!(
            super::extract_source_local_object_mode_long_chain_five_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 231 must reject the Task 234 set-terminal six-hop source"
        );
        let preexisting_owner_results = [
            extract_source_reserved_variable_type_assertion(&exact, module.clone(), &symbols),
            super::extract_source_reserved_object_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_local_object_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_chained_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_chained_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_chained_local_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_chained_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_chained_local_mode_radix_asserted_head(&exact, module.clone(), &symbols),
            extract_source_chained_local_object_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_two_edge_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_object_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_three_edge_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_object_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_four_edge_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_four_edge_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            super::extract_source_four_edge_local_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            super::extract_source_four_edge_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_four_edge_local_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_four_edge_local_object_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_mode_long_chain_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_object_mode_long_chain_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            super::extract_source_local_mode_long_chain_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            super::extract_source_local_object_mode_long_chain_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
        ];
        assert_eq!(preexisting_owner_results.len(), 34);
        assert!(
            preexisting_owner_results
                .into_iter()
                .all(|result| result.is_none()),
            "all 34 legacy type-assertion owner routes must reject Task 234"
        );
        let payload = super::extract_source_local_mode_long_chain_six_hop_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact seven-expansion six-hop source should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 7);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(payload.subject_lookup_ordinal, 1);
        let source_binding = &payload.reserve.bridge.bindings()[0];
        assert_eq!(source_binding.type_spelling, mode_names[6]);
        assert_eq!(payload.asserted_type.spelling, mode_names[0]);
        assert_ne!(payload.asserted_type.head, source_binding.type_head);
        assert!(matches!(source_binding.type_head, TypeHeadInput::Symbol(_)));
        assert_ne!(payload.asserted_type_site, payload.subject_site);
        assert_ne!(payload.asserted_type.range, source_binding.type_range);
        let binding_expansion = payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[6]))
            .map(|(_, expansion)| expansion)
            .expect("ChainMode6 binding expansion should exist");
        assert_eq!(binding_expansion.radix.spelling, mode_names[5]);
        assert_ne!(binding_expansion.radix.head, payload.asserted_type.head);
        let TypeHeadInput::Symbol(first_intermediate_symbol) = &binding_expansion.radix.head else {
            panic!("ChainMode5 first relation intermediate should resolve to a symbol");
        };
        let first_intermediate_expansion = payload
            .reserve
            .mode_expansions
            .get(first_intermediate_symbol)
            .expect("ChainMode5 first intermediate expansion should exist");
        assert_eq!(first_intermediate_expansion.radix.spelling, mode_names[4]);
        assert_ne!(
            first_intermediate_expansion.radix.head,
            payload.asserted_type.head
        );
        let TypeHeadInput::Symbol(second_intermediate_symbol) =
            &first_intermediate_expansion.radix.head
        else {
            panic!("ChainMode4 second relation intermediate should resolve to a symbol");
        };
        let second_intermediate_expansion = payload
            .reserve
            .mode_expansions
            .get(second_intermediate_symbol)
            .expect("ChainMode4 second intermediate expansion should exist");
        assert_eq!(second_intermediate_expansion.radix.spelling, mode_names[3]);
        assert_ne!(
            second_intermediate_expansion.radix.head,
            payload.asserted_type.head
        );
        let TypeHeadInput::Symbol(third_intermediate_symbol) =
            &second_intermediate_expansion.radix.head
        else {
            panic!("ChainMode3 third relation intermediate should resolve to a symbol");
        };
        let third_intermediate_expansion = payload
            .reserve
            .mode_expansions
            .get(third_intermediate_symbol)
            .expect("ChainMode3 third intermediate expansion should exist");
        assert_eq!(third_intermediate_expansion.radix.spelling, mode_names[2]);
        assert_ne!(
            third_intermediate_expansion.radix.head,
            payload.asserted_type.head
        );
        let TypeHeadInput::Symbol(fourth_intermediate_symbol) =
            &third_intermediate_expansion.radix.head
        else {
            panic!("ChainMode2 fourth relation intermediate should resolve to a symbol");
        };
        let fourth_intermediate_expansion = payload
            .reserve
            .mode_expansions
            .get(fourth_intermediate_symbol)
            .expect("ChainMode2 fourth intermediate expansion should exist");
        assert_eq!(fourth_intermediate_expansion.radix.spelling, mode_names[1]);
        assert_ne!(
            fourth_intermediate_expansion.radix.head,
            payload.asserted_type.head
        );
        let TypeHeadInput::Symbol(fifth_intermediate_symbol) =
            &fourth_intermediate_expansion.radix.head
        else {
            panic!("ChainMode1 fifth relation intermediate should resolve to a symbol");
        };
        let fifth_intermediate_expansion = payload
            .reserve
            .mode_expansions
            .get(fifth_intermediate_symbol)
            .expect("ChainMode1 fifth intermediate expansion should exist");
        assert_eq!(fifth_intermediate_expansion.radix.spelling, mode_names[0]);
        assert_eq!(
            fifth_intermediate_expansion.radix.head,
            payload.asserted_type.head
        );

        let output = super::source_local_mode_long_chain_six_hop_asserted_head_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact seven-expansion asserted-head source should reach checker");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 234 checker payload invariants should hold");
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert_eq!(output.subject_result_input.spelling, mode_names[6]);
        assert_eq!(output.asserted_type_input.spelling, mode_names[0]);
        assert_ne!(
            output.subject_result_input.head,
            output.asserted_type_input.head
        );
        assert!(matches!(
            output.asserted_type_input.head,
            TypeHeadInput::Symbol(_)
        ));
        assert_ne!(
            output.subject_result_input.site,
            output.asserted_type_input.site
        );
        assert_ne!(
            output.subject_result_input.source_range,
            output.asserted_type_input.source_range
        );
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[0]))
            .map(|(_, expansion)| &expansion.radix)
            .expect("BaseMode terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("one normalized builtin-set type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
        let (_, term) = output.term_formula.terms().iter().next().unwrap();
        assert_eq!(term.status, TermStatus::Inferred);
        assert!(term.deferred.is_empty());
        let (_, formula) = output.term_formula.formulas().iter().next().unwrap();
        assert_eq!(formula.kind, FormulaKind::TypeAssertion);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert!(formula.expected_types.is_empty());
        assert!(formula.candidate_set.is_none());
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
        assert!(output.term_formula.candidate_sets().is_empty());
        assert!(output.term_formula.facts().is_empty());
        assert!(output.term_formula.diagnostics().is_empty());

        let assert_invalid_output = |invalid| {
            let invalid_result =
                assert_source_reserved_variable_type_assertion_output(&invalid).map(|()| invalid);
            assert_eq!(
                source_reserved_variable_type_assertion_result_detail_keys(
                    invalid_result,
                    super::TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_SIX_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
                ),
                vec![
                    super::TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_SIX_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY
                        .to_owned()
                ]
            );
        };
        for removed in mode_names {
            let mut invalid = super::source_local_mode_long_chain_six_hop_asserted_head_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .unwrap();
            invalid
                .payload
                .reserve
                .mode_expansions
                .retain(|symbol, _| source_mode_symbol_spelling(symbol) != Some(removed));
            assert_invalid_output(invalid);
        }
        let exact_output = || {
            super::source_local_mode_long_chain_six_hop_asserted_head_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .unwrap()
        };
        let mut corruptions = Vec::new();
        let mut invalid = exact_output();
        invalid.subject_binding = BindingId::new(1);
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.payload.subject_lookup_ordinal = 2;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.spelling = mode_names[5].to_owned();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.spelling = "OtherLongMode".to_owned();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.head = invalid.subject_result_input.head.clone();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.head = TypeHeadInput::BuiltinObject;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.head = TypeHeadInput::BuiltinObject;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.source_range = invalid.asserted_type_input.source_range;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.site = invalid.asserted_type_input.site.clone();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.source_range = invalid.subject_result_input.source_range;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.site = invalid.subject_result_input.site.clone();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        let binding_head = invalid.subject_result_input.head.clone();
        let (_, binding_expansion) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[6]))
            .unwrap();
        binding_expansion.radix.head = binding_head;
        corruptions.push(invalid);
        for mode_name in &mode_names[1..] {
            let mut invalid = exact_output();
            let (_, expansion) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(*mode_name))
                .unwrap();
            expansion.radix.spelling = "object".to_owned();
            expansion.radix.head = TypeHeadInput::BuiltinObject;
            corruptions.push(invalid);
        }
        let mut invalid = exact_output();
        let (_, base_expansion) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[0]))
            .unwrap();
        base_expansion.radix.source_range = range(source_id, 0, 1);
        corruptions.push(invalid);
        let mut invalid = exact_output();
        let (_, base_expansion) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[0]))
            .unwrap();
        base_expansion.radix.spelling = "object".to_owned();
        base_expansion.radix.head = TypeHeadInput::BuiltinObject;
        corruptions.push(invalid);
        for invalid in corruptions {
            assert_invalid_output(invalid);
        }
        assert_source_reserved_variable_type_assertion_output(&exact_output())
            .expect("corrupting cloned Task 234 outputs must not mutate the exact output");

        let assert_extraction_gap = |ast, context: &str| {
            assert_eq!(
                source_type_elaboration_detail_keys(&ast, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()],
                "{context}"
            );
        };
        fn next_permutation(order: &mut [usize; 7]) -> bool {
            let Some(pivot) = (0..order.len() - 1)
                .rev()
                .find(|&index| order[index] < order[index + 1])
            else {
                return false;
            };
            let successor = (pivot + 1..order.len())
                .rev()
                .find(|&index| order[pivot] < order[index])
                .expect("a lexicographic successor must exist after the pivot");
            order.swap(pivot, successor);
            order[pivot + 1..].reverse();
            true
        }
        let ordered = exact_modes();
        let public_dispatch_orders = [
            [1, 0, 2, 3, 4, 5, 6],
            [0, 2, 1, 3, 4, 5, 6],
            [0, 1, 3, 2, 4, 5, 6],
            [0, 1, 2, 4, 3, 5, 6],
            [0, 1, 2, 3, 5, 4, 6],
            [0, 1, 2, 3, 4, 6, 5],
            [1, 2, 3, 4, 5, 6, 0],
            [6, 5, 4, 3, 2, 1, 0],
        ];
        let mut order = [0, 1, 2, 3, 4, 5, 6];
        let mut permutation_count = 0;
        let mut public_dispatch_representative_count = 0;
        while next_permutation(&mut order) {
            permutation_count += 1;
            let permuted = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                vec![
                    ordered[order[0]],
                    ordered[order[1]],
                    ordered[order[2]],
                    ordered[order[3]],
                    ordered[order[4]],
                    ordered[order[5]],
                    ordered[order[6]],
                ],
                reserve(),
                theorem,
            );
            assert!(
                super::extract_source_local_mode_long_chain_six_hop_asserted_head(
                    &permuted,
                    module.clone(),
                    &symbols,
                )
                .is_none(),
                "definition permutation {order:?} must not reach Task 234"
            );
            if public_dispatch_orders.contains(&order) {
                public_dispatch_representative_count += 1;
                assert_extraction_gap(
                    permuted,
                    &format!("public definition permutation {order:?}"),
                );
            }
        }
        assert_eq!(permutation_count, 5_039);
        assert_eq!(public_dispatch_representative_count, 8);
        let mut mode_near_misses = vec![
            Vec::<ModeDefinitionSpec>::new(),
            {
                let mut modes = exact_modes();
                modes.push(modes[0]);
                modes
            },
            {
                let mut modes = exact_modes();
                modes.push(mode_definition_with_label(
                    "UnusedLongMode",
                    "UnusedLongModeDef",
                    ReserveTypeShape::QualifiedSymbol(mode_names[6]),
                ));
                modes
            },
        ];
        for index in 0..mode_names.len() {
            let mut missing = exact_modes();
            missing.remove(index);
            mode_near_misses.push(missing);
            let mut duplicate = exact_modes();
            duplicate.push(duplicate[index]);
            mode_near_misses.push(duplicate);
            let mut malformed = exact_modes();
            malformed[index].rhs_shape = if index == 0 {
                ReserveTypeShape::Builtin("object")
            } else {
                ReserveTypeShape::QualifiedSymbol(mode_names[(index + 1) % mode_names.len()])
            };
            mode_near_misses.push(malformed);
            let mut wrong_label = exact_modes();
            wrong_label[index].label = Some("WrongLongModeDef");
            mode_near_misses.push(wrong_label);
            let mut wrong_spelling = exact_modes();
            wrong_spelling[index].pattern = "OtherLongMode";
            mode_near_misses.push(wrong_spelling);
            let mut recovered = exact_modes();
            recovered[index].recovered = true;
            mode_near_misses.push(recovered);
            let mut contextual = exact_modes();
            contextual[index].local_context = true;
            mode_near_misses.push(contextual);
            let mut parameterized = exact_modes();
            parameterized[index].parameterized_pattern = true;
            mode_near_misses.push(parameterized);
            let mut args = exact_modes();
            args[index].rhs_shape = if index == 0 {
                ReserveTypeShape::QualifiedSymbolWithArgs("set")
            } else {
                ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[index - 1])
            };
            mode_near_misses.push(args);
            let mut attrs = exact_modes();
            attrs[index].rhs_shape = if index == 0 {
                ReserveTypeShape::AttributedSet
            } else {
                ReserveTypeShape::AttributedQualifiedSymbol(mode_names[index - 1])
            };
            mode_near_misses.push(attrs);
        }
        for (index, modes) in mode_near_misses.into_iter().enumerate() {
            assert_extraction_gap(
                mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    modes,
                    reserve(),
                    theorem,
                ),
                &format!("mode near miss {index}"),
            );
        }

        let mut source_near_misses = Vec::new();
        let mut deeper_modes = exact_modes();
        deeper_modes.push(mode_definition_with_label(
            "ChainMode7",
            "ChainMode7Def",
            ReserveTypeShape::QualifiedSymbol(mode_names[6]),
        ));
        source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            deeper_modes,
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("ChainMode7"),
            )],
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[6]),
                ..theorem
            },
        ));
        for bad_reserve in [
            ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[6]),
            ReserveTypeShape::AttributedQualifiedSymbol(mode_names[6]),
            ReserveTypeShape::Builtin("set"),
            ReserveTypeShape::Builtin("object"),
        ] {
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(vec!["x"], bad_reserve)],
                theorem,
            ));
        }
        source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            exact_modes(),
            vec![reserve_item(
                vec!["y"],
                ReserveTypeShape::QualifiedSymbol(mode_names[6]),
            )],
            theorem,
        ));
        source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            exact_modes(),
            vec![
                reserve_item(vec!["x"], ReserveTypeShape::QualifiedSymbol(mode_names[6])),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
            ],
            theorem,
        ));
        for bad_reserve in [
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol(mode_names[5]),
            )],
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("OtherLongMode"),
            )],
            vec![reserve_item(
                vec!["x", "y"],
                ReserveTypeShape::QualifiedSymbol(mode_names[6]),
            )],
        ] {
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                bad_reserve,
                theorem,
            ));
        }
        let theorem_near_misses = [
            IdentifierTypeAssertionTheoremSpec {
                label: "OtherPayloadBoundary",
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                subject: "y",
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::Builtin("set"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::Builtin("object"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[1]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[2]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[3]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[4]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[5]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[6]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol("OtherLongMode"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[0]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::AttributedQualifiedSymbol(mode_names[0]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                negated: true,
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                status: Some("open"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                recovered_label: true,
                ..theorem
            },
        ];
        for near_miss_theorem in theorem_near_misses {
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                near_miss_theorem,
            ));
        }
        source_near_misses.push(
            modes_then_empty_definition_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                theorem,
            ),
        );
        for (index, near_miss) in source_near_misses.into_iter().enumerate() {
            assert_extraction_gap(near_miss, &format!("source near miss {index}"));
        }

        for imported_index in 0..mode_names.len() {
            let local_modes = mode_names
                .iter()
                .enumerate()
                .filter(|(index, _)| *index != imported_index)
                .map(|(_, spelling)| (*spelling, SymbolKind::Mode))
                .collect::<Vec<_>>();
            for imported in [
                vec![(mode_names[imported_index], SymbolKind::Mode)],
                vec![
                    (mode_names[imported_index], SymbolKind::Mode),
                    (mode_names[imported_index], SymbolKind::Mode),
                ],
            ] {
                let imported_symbols = source_local_and_imported_symbols_env(
                    module.clone(),
                    &local_modes,
                    imported.as_slice(),
                );
                assert_eq!(
                    source_type_elaboration_detail_keys(&exact, module.clone(), &imported_symbols),
                    vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
                );
            }
        }
        let unrelated_import = source_local_and_imported_symbols_env(
            module.clone(),
            &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
            &[("UnrelatedLongMode", SymbolKind::Mode)],
        );
        assert!(
            super::extract_source_local_mode_long_chain_six_hop_asserted_head(
                &exact,
                module.clone(),
                &unrelated_import,
            )
            .is_some()
        );
        for (asserted_spelling, asserted_head_symbols) in [
            (
                "ImportedLongMode",
                source_local_and_imported_symbols_env(
                    module.clone(),
                    &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
                    &[("ImportedLongMode", SymbolKind::Mode)],
                ),
            ),
            (
                "AmbiguousLongMode",
                source_local_and_imported_symbols_env(
                    module.clone(),
                    &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
                    &[
                        ("AmbiguousLongMode", SymbolKind::Mode),
                        ("AmbiguousLongMode", SymbolKind::Mode),
                    ],
                ),
            ),
        ] {
            let near_miss = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(asserted_spelling),
                    ..theorem
                },
            );
            assert_eq!(
                source_type_elaboration_detail_keys(
                    &near_miss,
                    module.clone(),
                    &asserted_head_symbols,
                ),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
    }

    #[test]
    fn source_local_object_mode_long_chain_six_hop_asserted_head_consumes_seven_expansions() {
        let source_id = source_id(236);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("local_object_mode_long_chain_six_hop_asserted_head"),
        );
        let mode_names = [
            "BaseObjectMode",
            "ChainObjectMode1",
            "ChainObjectMode2",
            "ChainObjectMode3",
            "ChainObjectMode4",
            "ChainObjectMode5",
            "ChainObjectMode6",
        ];
        let mode_labels = [
            "BaseObjectModeDef",
            "ChainObjectMode1Def",
            "ChainObjectMode2Def",
            "ChainObjectMode3Def",
            "ChainObjectMode4Def",
            "ChainObjectMode5Def",
            "ChainObjectMode6Def",
        ];
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                (mode_names[0], SymbolKind::Mode),
                (mode_names[1], SymbolKind::Mode),
                (mode_names[2], SymbolKind::Mode),
                (mode_names[3], SymbolKind::Mode),
                (mode_names[4], SymbolKind::Mode),
                (mode_names[5], SymbolKind::Mode),
                (mode_names[6], SymbolKind::Mode),
                ("ChainObjectMode7", SymbolKind::Mode),
                ("OtherLongMode", SymbolKind::Mode),
            ],
        );
        let theorem = IdentifierTypeAssertionTheoremSpec {
            status: None,
            label: "LongLocalObjectModeSixHopAssertedHeadPayloadBoundary",
            subject: "x",
            asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[0]),
            negated: false,
            recovered_label: false,
        };
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    mode_names[0],
                    mode_labels[0],
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition_with_label(
                    mode_names[1],
                    mode_labels[1],
                    ReserveTypeShape::QualifiedSymbol(mode_names[0]),
                ),
                mode_definition_with_label(
                    mode_names[2],
                    mode_labels[2],
                    ReserveTypeShape::QualifiedSymbol(mode_names[1]),
                ),
                mode_definition_with_label(
                    mode_names[3],
                    mode_labels[3],
                    ReserveTypeShape::QualifiedSymbol(mode_names[2]),
                ),
                mode_definition_with_label(
                    mode_names[4],
                    mode_labels[4],
                    ReserveTypeShape::QualifiedSymbol(mode_names[3]),
                ),
                mode_definition_with_label(
                    mode_names[5],
                    mode_labels[5],
                    ReserveTypeShape::QualifiedSymbol(mode_names[4]),
                ),
                mode_definition_with_label(
                    mode_names[6],
                    mode_labels[6],
                    ReserveTypeShape::QualifiedSymbol(mode_names[5]),
                ),
            ]
        };
        let reserve = || {
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol(mode_names[6]),
            )]
        };
        let exact = mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            exact_modes(),
            reserve(),
            theorem,
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        assert!(
            extract_source_local_mode_long_chain_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 175 must reject the Task 236 six-hop source"
        );
        assert!(
            super::extract_source_local_mode_long_chain_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 199 must reject the Task 236 six-hop source"
        );
        assert!(
            super::extract_source_local_mode_long_chain_two_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 224 must reject the Task 236 six-hop source"
        );
        assert!(
            super::extract_source_local_object_mode_long_chain_two_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 225 must reject the Task 236 object-terminal six-hop source"
        );
        assert!(
            super::extract_source_local_mode_long_chain_three_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 226 must reject the Task 236 six-hop source"
        );
        assert!(
            super::extract_source_local_object_mode_long_chain_three_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 227 must reject the Task 236 object-terminal six-hop source"
        );
        assert!(
            super::extract_source_local_mode_long_chain_four_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 228 must reject the Task 236 six-hop source"
        );
        assert!(
            super::extract_source_local_object_mode_long_chain_four_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 229 must reject the Task 236 object-terminal six-hop source"
        );
        assert!(
            super::extract_source_local_mode_long_chain_five_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 230 must reject the Task 236 six-hop source"
        );
        assert!(
            super::extract_source_local_object_mode_long_chain_five_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 231 must reject the Task 236 object-terminal six-hop source"
        );
        assert!(
            super::extract_source_local_mode_long_chain_six_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 234 must reject the Task 236 object-terminal six-hop source"
        );
        let preexisting_owner_results = [
            extract_source_reserved_variable_type_assertion(&exact, module.clone(), &symbols),
            super::extract_source_reserved_object_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_local_object_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_chained_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_chained_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_chained_local_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_chained_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_chained_local_mode_radix_asserted_head(&exact, module.clone(), &symbols),
            extract_source_chained_local_object_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_two_edge_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_object_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_three_edge_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_object_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_four_edge_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_four_edge_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            super::extract_source_four_edge_local_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            super::extract_source_four_edge_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_four_edge_local_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_four_edge_local_object_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_mode_long_chain_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_object_mode_long_chain_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            super::extract_source_local_mode_long_chain_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            super::extract_source_local_object_mode_long_chain_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
        ];
        assert_eq!(preexisting_owner_results.len(), 34);
        assert!(
            preexisting_owner_results
                .into_iter()
                .all(|result| result.is_none()),
            "all 34 legacy type-assertion owner routes must reject Task 236"
        );
        let payload = super::extract_source_local_object_mode_long_chain_six_hop_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact seven-expansion six-hop source should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 7);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(payload.subject_lookup_ordinal, 1);
        let source_binding = &payload.reserve.bridge.bindings()[0];
        assert_eq!(source_binding.type_spelling, mode_names[6]);
        assert_eq!(payload.asserted_type.spelling, mode_names[0]);
        assert_ne!(payload.asserted_type.head, source_binding.type_head);
        assert!(matches!(source_binding.type_head, TypeHeadInput::Symbol(_)));
        assert_ne!(payload.asserted_type_site, payload.subject_site);
        assert_ne!(payload.asserted_type.range, source_binding.type_range);
        let binding_expansion = payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[6]))
            .map(|(_, expansion)| expansion)
            .expect("ChainObjectMode6 binding expansion should exist");
        assert_eq!(binding_expansion.radix.spelling, mode_names[5]);
        assert_ne!(binding_expansion.radix.head, payload.asserted_type.head);
        let TypeHeadInput::Symbol(first_intermediate_symbol) = &binding_expansion.radix.head else {
            panic!("ChainObjectMode5 first relation intermediate should resolve to a symbol");
        };
        let first_intermediate_expansion = payload
            .reserve
            .mode_expansions
            .get(first_intermediate_symbol)
            .expect("ChainObjectMode5 first intermediate expansion should exist");
        assert_eq!(first_intermediate_expansion.radix.spelling, mode_names[4]);
        assert_ne!(
            first_intermediate_expansion.radix.head,
            payload.asserted_type.head
        );
        let TypeHeadInput::Symbol(second_intermediate_symbol) =
            &first_intermediate_expansion.radix.head
        else {
            panic!("ChainObjectMode4 second relation intermediate should resolve to a symbol");
        };
        let second_intermediate_expansion = payload
            .reserve
            .mode_expansions
            .get(second_intermediate_symbol)
            .expect("ChainObjectMode4 second intermediate expansion should exist");
        assert_eq!(second_intermediate_expansion.radix.spelling, mode_names[3]);
        assert_ne!(
            second_intermediate_expansion.radix.head,
            payload.asserted_type.head
        );
        let TypeHeadInput::Symbol(third_intermediate_symbol) =
            &second_intermediate_expansion.radix.head
        else {
            panic!("ChainObjectMode3 third relation intermediate should resolve to a symbol");
        };
        let third_intermediate_expansion = payload
            .reserve
            .mode_expansions
            .get(third_intermediate_symbol)
            .expect("ChainObjectMode3 third intermediate expansion should exist");
        assert_eq!(third_intermediate_expansion.radix.spelling, mode_names[2]);
        assert_ne!(
            third_intermediate_expansion.radix.head,
            payload.asserted_type.head
        );
        let TypeHeadInput::Symbol(fourth_intermediate_symbol) =
            &third_intermediate_expansion.radix.head
        else {
            panic!("ChainObjectMode2 fourth relation intermediate should resolve to a symbol");
        };
        let fourth_intermediate_expansion = payload
            .reserve
            .mode_expansions
            .get(fourth_intermediate_symbol)
            .expect("ChainObjectMode2 fourth intermediate expansion should exist");
        assert_eq!(fourth_intermediate_expansion.radix.spelling, mode_names[1]);
        assert_ne!(
            fourth_intermediate_expansion.radix.head,
            payload.asserted_type.head
        );
        let TypeHeadInput::Symbol(fifth_intermediate_symbol) =
            &fourth_intermediate_expansion.radix.head
        else {
            panic!("ChainObjectMode1 fifth relation intermediate should resolve to a symbol");
        };
        let fifth_intermediate_expansion = payload
            .reserve
            .mode_expansions
            .get(fifth_intermediate_symbol)
            .expect("ChainObjectMode1 fifth intermediate expansion should exist");
        assert_eq!(fifth_intermediate_expansion.radix.spelling, mode_names[0]);
        assert_eq!(
            fifth_intermediate_expansion.radix.head,
            payload.asserted_type.head
        );

        let output = super::source_local_object_mode_long_chain_six_hop_asserted_head_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact seven-expansion asserted-head source should reach checker");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 236 checker payload invariants should hold");
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert_eq!(output.subject_result_input.spelling, mode_names[6]);
        assert_eq!(output.asserted_type_input.spelling, mode_names[0]);
        assert_ne!(
            output.subject_result_input.head,
            output.asserted_type_input.head
        );
        assert!(matches!(
            output.asserted_type_input.head,
            TypeHeadInput::Symbol(_)
        ));
        assert_ne!(
            output.subject_result_input.site,
            output.asserted_type_input.site
        );
        assert_ne!(
            output.subject_result_input.source_range,
            output.asserted_type_input.source_range
        );
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[0]))
            .map(|(_, expansion)| &expansion.radix)
            .expect("BaseObjectMode terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("one normalized builtin-object type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
        let (_, term) = output.term_formula.terms().iter().next().unwrap();
        assert_eq!(term.status, TermStatus::Inferred);
        assert!(term.deferred.is_empty());
        let (_, formula) = output.term_formula.formulas().iter().next().unwrap();
        assert_eq!(formula.kind, FormulaKind::TypeAssertion);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert!(formula.expected_types.is_empty());
        assert!(formula.candidate_set.is_none());
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
        assert!(output.term_formula.candidate_sets().is_empty());
        assert!(output.term_formula.facts().is_empty());
        assert!(output.term_formula.diagnostics().is_empty());

        let assert_invalid_output = |invalid| {
            let invalid_result =
                assert_source_reserved_variable_type_assertion_output(&invalid).map(|()| invalid);
            assert_eq!(
                source_reserved_variable_type_assertion_result_detail_keys(
                    invalid_result,
                    super::TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_SIX_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
                ),
                vec![
                    super::TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_SIX_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY
                        .to_owned()
                ]
            );
        };
        for removed in mode_names {
            let mut invalid =
                super::source_local_object_mode_long_chain_six_hop_asserted_head_output(
                    &exact,
                    module.clone(),
                    &symbols,
                )
                .unwrap();
            invalid
                .payload
                .reserve
                .mode_expansions
                .retain(|symbol, _| source_mode_symbol_spelling(symbol) != Some(removed));
            assert_invalid_output(invalid);
        }
        let exact_output = || {
            super::source_local_object_mode_long_chain_six_hop_asserted_head_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .unwrap()
        };
        let mut corruptions = Vec::new();
        let mut invalid = exact_output();
        invalid.subject_binding = BindingId::new(1);
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.payload.subject_lookup_ordinal = 2;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.spelling = mode_names[5].to_owned();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.spelling = "OtherLongMode".to_owned();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.head = invalid.subject_result_input.head.clone();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.head = TypeHeadInput::BuiltinSet;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.head = TypeHeadInput::BuiltinSet;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.source_range = invalid.asserted_type_input.source_range;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.site = invalid.asserted_type_input.site.clone();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.source_range = invalid.subject_result_input.source_range;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.site = invalid.subject_result_input.site.clone();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        let binding_head = invalid.subject_result_input.head.clone();
        let (_, binding_expansion) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[6]))
            .unwrap();
        binding_expansion.radix.head = binding_head;
        corruptions.push(invalid);
        for mode_name in &mode_names[1..] {
            let mut invalid = exact_output();
            let (_, expansion) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(*mode_name))
                .unwrap();
            expansion.radix.spelling = "set".to_owned();
            expansion.radix.head = TypeHeadInput::BuiltinSet;
            corruptions.push(invalid);
        }
        let mut invalid = exact_output();
        let (_, base_expansion) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[0]))
            .unwrap();
        base_expansion.radix.source_range = range(source_id, 0, 1);
        corruptions.push(invalid);
        let mut invalid = exact_output();
        let (_, base_expansion) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[0]))
            .unwrap();
        base_expansion.radix.spelling = "set".to_owned();
        base_expansion.radix.head = TypeHeadInput::BuiltinSet;
        corruptions.push(invalid);
        for invalid in corruptions {
            assert_invalid_output(invalid);
        }
        assert_source_reserved_variable_type_assertion_output(&exact_output())
            .expect("corrupting cloned Task 236 outputs must not mutate the exact output");

        let assert_extraction_gap = |ast, context: &str| {
            assert_eq!(
                source_type_elaboration_detail_keys(&ast, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()],
                "{context}"
            );
        };
        fn next_permutation(order: &mut [usize; 7]) -> bool {
            let Some(pivot) = (0..order.len() - 1)
                .rev()
                .find(|&index| order[index] < order[index + 1])
            else {
                return false;
            };
            let successor = (pivot + 1..order.len())
                .rev()
                .find(|&index| order[pivot] < order[index])
                .expect("a lexicographic successor must exist after the pivot");
            order.swap(pivot, successor);
            order[pivot + 1..].reverse();
            true
        }
        let ordered = exact_modes();
        let public_dispatch_orders = [
            [1, 0, 2, 3, 4, 5, 6],
            [0, 2, 1, 3, 4, 5, 6],
            [0, 1, 3, 2, 4, 5, 6],
            [0, 1, 2, 4, 3, 5, 6],
            [0, 1, 2, 3, 5, 4, 6],
            [0, 1, 2, 3, 4, 6, 5],
            [1, 2, 3, 4, 5, 6, 0],
            [6, 5, 4, 3, 2, 1, 0],
        ];
        let mut order = [0, 1, 2, 3, 4, 5, 6];
        let mut permutation_count = 0;
        let mut public_dispatch_representative_count = 0;
        while next_permutation(&mut order) {
            permutation_count += 1;
            let permuted = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                vec![
                    ordered[order[0]],
                    ordered[order[1]],
                    ordered[order[2]],
                    ordered[order[3]],
                    ordered[order[4]],
                    ordered[order[5]],
                    ordered[order[6]],
                ],
                reserve(),
                theorem,
            );
            assert!(
                super::extract_source_local_object_mode_long_chain_six_hop_asserted_head(
                    &permuted,
                    module.clone(),
                    &symbols,
                )
                .is_none(),
                "definition permutation {order:?} must not reach Task 236"
            );
            if public_dispatch_orders.contains(&order) {
                public_dispatch_representative_count += 1;
                assert_extraction_gap(
                    permuted,
                    &format!("public definition permutation {order:?}"),
                );
            }
        }
        assert_eq!(permutation_count, 5_039);
        assert_eq!(public_dispatch_representative_count, 8);
        let mut mode_near_misses = vec![
            Vec::<ModeDefinitionSpec>::new(),
            {
                let mut modes = exact_modes();
                modes.push(modes[0]);
                modes
            },
            {
                let mut modes = exact_modes();
                modes.push(mode_definition_with_label(
                    "UnusedLongMode",
                    "UnusedLongModeDef",
                    ReserveTypeShape::QualifiedSymbol(mode_names[6]),
                ));
                modes
            },
        ];
        for index in 0..mode_names.len() {
            let mut missing = exact_modes();
            missing.remove(index);
            mode_near_misses.push(missing);
            let mut duplicate = exact_modes();
            duplicate.push(duplicate[index]);
            mode_near_misses.push(duplicate);
            let mut malformed = exact_modes();
            malformed[index].rhs_shape = if index == 0 {
                ReserveTypeShape::Builtin("set")
            } else {
                ReserveTypeShape::QualifiedSymbol(mode_names[(index + 1) % mode_names.len()])
            };
            mode_near_misses.push(malformed);
            let mut wrong_label = exact_modes();
            wrong_label[index].label = Some("WrongLongModeDef");
            mode_near_misses.push(wrong_label);
            let mut wrong_spelling = exact_modes();
            wrong_spelling[index].pattern = "OtherLongMode";
            mode_near_misses.push(wrong_spelling);
            let mut recovered = exact_modes();
            recovered[index].recovered = true;
            mode_near_misses.push(recovered);
            let mut contextual = exact_modes();
            contextual[index].local_context = true;
            mode_near_misses.push(contextual);
            let mut parameterized = exact_modes();
            parameterized[index].parameterized_pattern = true;
            mode_near_misses.push(parameterized);
            let mut args = exact_modes();
            args[index].rhs_shape = if index == 0 {
                ReserveTypeShape::QualifiedSymbolWithArgs("object")
            } else {
                ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[index - 1])
            };
            mode_near_misses.push(args);
            let mut attrs = exact_modes();
            attrs[index].rhs_shape = if index == 0 {
                ReserveTypeShape::AttributedObject
            } else {
                ReserveTypeShape::AttributedQualifiedSymbol(mode_names[index - 1])
            };
            mode_near_misses.push(attrs);
        }
        for (index, modes) in mode_near_misses.into_iter().enumerate() {
            assert_extraction_gap(
                mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    modes,
                    reserve(),
                    theorem,
                ),
                &format!("mode near miss {index}"),
            );
        }

        let mut source_near_misses = Vec::new();
        let mut deeper_modes = exact_modes();
        deeper_modes.push(mode_definition_with_label(
            "ChainObjectMode7",
            "ChainObjectMode7Def",
            ReserveTypeShape::QualifiedSymbol(mode_names[6]),
        ));
        source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            deeper_modes,
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("ChainObjectMode7"),
            )],
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[6]),
                ..theorem
            },
        ));
        for bad_reserve in [
            ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[6]),
            ReserveTypeShape::AttributedQualifiedSymbol(mode_names[6]),
            ReserveTypeShape::Builtin("object"),
            ReserveTypeShape::Builtin("set"),
        ] {
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(vec!["x"], bad_reserve)],
                theorem,
            ));
        }
        source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            exact_modes(),
            vec![reserve_item(
                vec!["y"],
                ReserveTypeShape::QualifiedSymbol(mode_names[6]),
            )],
            theorem,
        ));
        source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            exact_modes(),
            vec![
                reserve_item(vec!["x"], ReserveTypeShape::QualifiedSymbol(mode_names[6])),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("object")),
            ],
            theorem,
        ));
        for bad_reserve in [
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol(mode_names[5]),
            )],
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("OtherLongMode"),
            )],
            vec![reserve_item(
                vec!["x", "y"],
                ReserveTypeShape::QualifiedSymbol(mode_names[6]),
            )],
        ] {
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                bad_reserve,
                theorem,
            ));
        }
        let theorem_near_misses = [
            IdentifierTypeAssertionTheoremSpec {
                label: "OtherPayloadBoundary",
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                subject: "y",
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::Builtin("object"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::Builtin("set"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[1]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[2]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[3]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[4]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[5]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[6]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol("OtherLongMode"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[0]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::AttributedQualifiedSymbol(mode_names[0]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                negated: true,
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                status: Some("open"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                recovered_label: true,
                ..theorem
            },
        ];
        for near_miss_theorem in theorem_near_misses {
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                near_miss_theorem,
            ));
        }
        source_near_misses.push(
            modes_then_empty_definition_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                theorem,
            ),
        );
        for (index, near_miss) in source_near_misses.into_iter().enumerate() {
            assert_extraction_gap(near_miss, &format!("source near miss {index}"));
        }

        for imported_index in 0..mode_names.len() {
            let local_modes = mode_names
                .iter()
                .enumerate()
                .filter(|(index, _)| *index != imported_index)
                .map(|(_, spelling)| (*spelling, SymbolKind::Mode))
                .collect::<Vec<_>>();
            for imported in [
                vec![(mode_names[imported_index], SymbolKind::Mode)],
                vec![
                    (mode_names[imported_index], SymbolKind::Mode),
                    (mode_names[imported_index], SymbolKind::Mode),
                ],
            ] {
                let imported_symbols = source_local_and_imported_symbols_env(
                    module.clone(),
                    &local_modes,
                    imported.as_slice(),
                );
                assert_eq!(
                    source_type_elaboration_detail_keys(&exact, module.clone(), &imported_symbols),
                    vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
                );
            }
        }
        let unrelated_import = source_local_and_imported_symbols_env(
            module.clone(),
            &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
            &[("UnrelatedLongMode", SymbolKind::Mode)],
        );
        assert!(
            super::extract_source_local_object_mode_long_chain_six_hop_asserted_head(
                &exact,
                module.clone(),
                &unrelated_import,
            )
            .is_some()
        );
        for (asserted_spelling, asserted_head_symbols) in [
            (
                "ImportedLongMode",
                source_local_and_imported_symbols_env(
                    module.clone(),
                    &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
                    &[("ImportedLongMode", SymbolKind::Mode)],
                ),
            ),
            (
                "AmbiguousLongMode",
                source_local_and_imported_symbols_env(
                    module.clone(),
                    &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
                    &[
                        ("AmbiguousLongMode", SymbolKind::Mode),
                        ("AmbiguousLongMode", SymbolKind::Mode),
                    ],
                ),
            ),
        ] {
            let near_miss = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(asserted_spelling),
                    ..theorem
                },
            );
            assert_eq!(
                source_type_elaboration_detail_keys(
                    &near_miss,
                    module.clone(),
                    &asserted_head_symbols,
                ),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
    }

    #[test]
    fn source_local_object_mode_long_chain_five_hop_asserted_head_consumes_seven_expansions() {
        let source_id = source_id(231);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("local_object_mode_long_chain_five_hop_asserted_head"),
        );
        let mode_names = [
            "BaseObjectMode",
            "ChainObjectMode1",
            "ChainObjectMode2",
            "ChainObjectMode3",
            "ChainObjectMode4",
            "ChainObjectMode5",
            "ChainObjectMode6",
        ];
        let mode_labels = [
            "BaseObjectModeDef",
            "ChainObjectMode1Def",
            "ChainObjectMode2Def",
            "ChainObjectMode3Def",
            "ChainObjectMode4Def",
            "ChainObjectMode5Def",
            "ChainObjectMode6Def",
        ];
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                (mode_names[0], SymbolKind::Mode),
                (mode_names[1], SymbolKind::Mode),
                (mode_names[2], SymbolKind::Mode),
                (mode_names[3], SymbolKind::Mode),
                (mode_names[4], SymbolKind::Mode),
                (mode_names[5], SymbolKind::Mode),
                (mode_names[6], SymbolKind::Mode),
                ("ChainObjectMode7", SymbolKind::Mode),
                ("OtherLongMode", SymbolKind::Mode),
            ],
        );
        let theorem = IdentifierTypeAssertionTheoremSpec {
            status: None,
            label: "LongLocalObjectModeFiveHopAssertedHeadPayloadBoundary",
            subject: "x",
            asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[1]),
            negated: false,
            recovered_label: false,
        };
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    mode_names[0],
                    mode_labels[0],
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition_with_label(
                    mode_names[1],
                    mode_labels[1],
                    ReserveTypeShape::QualifiedSymbol(mode_names[0]),
                ),
                mode_definition_with_label(
                    mode_names[2],
                    mode_labels[2],
                    ReserveTypeShape::QualifiedSymbol(mode_names[1]),
                ),
                mode_definition_with_label(
                    mode_names[3],
                    mode_labels[3],
                    ReserveTypeShape::QualifiedSymbol(mode_names[2]),
                ),
                mode_definition_with_label(
                    mode_names[4],
                    mode_labels[4],
                    ReserveTypeShape::QualifiedSymbol(mode_names[3]),
                ),
                mode_definition_with_label(
                    mode_names[5],
                    mode_labels[5],
                    ReserveTypeShape::QualifiedSymbol(mode_names[4]),
                ),
                mode_definition_with_label(
                    mode_names[6],
                    mode_labels[6],
                    ReserveTypeShape::QualifiedSymbol(mode_names[5]),
                ),
            ]
        };
        let reserve = || {
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol(mode_names[6]),
            )]
        };
        let exact = mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            exact_modes(),
            reserve(),
            theorem,
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        assert!(
            extract_source_local_object_mode_long_chain_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 179 must reject the Task 231 five-hop source"
        );
        assert!(
            super::extract_source_local_object_mode_long_chain_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 200 must reject the Task 231 five-hop source"
        );
        assert!(
            super::extract_source_local_object_mode_long_chain_two_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 225 must reject the Task 231 five-hop source"
        );
        assert!(
            super::extract_source_local_mode_long_chain_two_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 224 must reject the Task 231 object-terminal five-hop source"
        );
        assert!(
            super::extract_source_local_object_mode_long_chain_three_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 227 must reject the Task 231 five-hop source"
        );
        assert!(
            super::extract_source_local_mode_long_chain_three_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 226 must reject the Task 231 object-terminal five-hop source"
        );
        assert!(
            super::extract_source_local_object_mode_long_chain_four_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 229 must reject the Task 231 five-hop source"
        );
        assert!(
            super::extract_source_local_mode_long_chain_four_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 228 must reject the Task 231 object-terminal five-hop source"
        );
        let preexisting_owner_results = [
            extract_source_reserved_variable_type_assertion(&exact, module.clone(), &symbols),
            super::extract_source_reserved_object_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_object_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_local_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_chained_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_chained_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_chained_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_chained_local_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_chained_local_object_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_chained_local_mode_radix_asserted_head(&exact, module.clone(), &symbols),
            extract_source_two_edge_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_two_edge_local_object_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_three_edge_local_object_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_four_edge_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_four_edge_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            super::extract_source_four_edge_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            super::extract_source_four_edge_local_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_four_edge_local_object_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_four_edge_local_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_object_mode_long_chain_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_mode_long_chain_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            super::extract_source_local_object_mode_long_chain_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            super::extract_source_local_mode_long_chain_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
        ];
        assert_eq!(preexisting_owner_results.len(), 34);
        assert!(
            preexisting_owner_results
                .into_iter()
                .all(|result| result.is_none()),
            "all 34 legacy type-assertion owner routes must reject Task 231"
        );
        let payload = super::extract_source_local_object_mode_long_chain_five_hop_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact seven-expansion five-hop source should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 7);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(payload.subject_lookup_ordinal, 1);
        let source_binding = &payload.reserve.bridge.bindings()[0];
        assert_eq!(source_binding.type_spelling, mode_names[6]);
        assert_eq!(payload.asserted_type.spelling, mode_names[1]);
        assert_ne!(payload.asserted_type.head, source_binding.type_head);
        assert!(matches!(source_binding.type_head, TypeHeadInput::Symbol(_)));
        assert_ne!(payload.asserted_type_site, payload.subject_site);
        assert_ne!(payload.asserted_type.range, source_binding.type_range);
        let binding_expansion = payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[6]))
            .map(|(_, expansion)| expansion)
            .expect("ChainObjectMode6 binding expansion should exist");
        assert_eq!(binding_expansion.radix.spelling, mode_names[5]);
        assert_ne!(binding_expansion.radix.head, payload.asserted_type.head);
        let TypeHeadInput::Symbol(first_intermediate_symbol) = &binding_expansion.radix.head else {
            panic!("ChainObjectMode5 first relation intermediate should resolve to a symbol");
        };
        let first_intermediate_expansion = payload
            .reserve
            .mode_expansions
            .get(first_intermediate_symbol)
            .expect("ChainObjectMode5 first intermediate expansion should exist");
        assert_eq!(first_intermediate_expansion.radix.spelling, mode_names[4]);
        assert_ne!(
            first_intermediate_expansion.radix.head,
            payload.asserted_type.head
        );
        let TypeHeadInput::Symbol(second_intermediate_symbol) =
            &first_intermediate_expansion.radix.head
        else {
            panic!("ChainObjectMode4 second relation intermediate should resolve to a symbol");
        };
        let second_intermediate_expansion = payload
            .reserve
            .mode_expansions
            .get(second_intermediate_symbol)
            .expect("ChainObjectMode4 second intermediate expansion should exist");
        assert_eq!(second_intermediate_expansion.radix.spelling, mode_names[3]);
        assert_ne!(
            second_intermediate_expansion.radix.head,
            payload.asserted_type.head
        );
        let TypeHeadInput::Symbol(third_intermediate_symbol) =
            &second_intermediate_expansion.radix.head
        else {
            panic!("ChainObjectMode3 third relation intermediate should resolve to a symbol");
        };
        let third_intermediate_expansion = payload
            .reserve
            .mode_expansions
            .get(third_intermediate_symbol)
            .expect("ChainObjectMode3 third intermediate expansion should exist");
        assert_eq!(third_intermediate_expansion.radix.spelling, mode_names[2]);
        assert_ne!(
            third_intermediate_expansion.radix.head,
            payload.asserted_type.head
        );
        let TypeHeadInput::Symbol(fourth_intermediate_symbol) =
            &third_intermediate_expansion.radix.head
        else {
            panic!("ChainObjectMode2 fourth relation intermediate should resolve to a symbol");
        };
        let fourth_intermediate_expansion = payload
            .reserve
            .mode_expansions
            .get(fourth_intermediate_symbol)
            .expect("ChainObjectMode2 fourth intermediate expansion should exist");
        assert_eq!(fourth_intermediate_expansion.radix.spelling, mode_names[1]);
        assert_eq!(
            fourth_intermediate_expansion.radix.head,
            payload.asserted_type.head
        );

        let output = super::source_local_object_mode_long_chain_five_hop_asserted_head_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact seven-expansion asserted-head source should reach checker");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 231 checker payload invariants should hold");
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert_eq!(output.subject_result_input.spelling, mode_names[6]);
        assert_eq!(output.asserted_type_input.spelling, mode_names[1]);
        assert_ne!(
            output.subject_result_input.head,
            output.asserted_type_input.head
        );
        assert!(matches!(
            output.asserted_type_input.head,
            TypeHeadInput::Symbol(_)
        ));
        assert_ne!(
            output.subject_result_input.site,
            output.asserted_type_input.site
        );
        assert_ne!(
            output.subject_result_input.source_range,
            output.asserted_type_input.source_range
        );
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[0]))
            .map(|(_, expansion)| &expansion.radix)
            .expect("BaseObjectMode terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("one normalized builtin-object type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
        let (_, term) = output.term_formula.terms().iter().next().unwrap();
        assert_eq!(term.status, TermStatus::Inferred);
        assert!(term.deferred.is_empty());
        let (_, formula) = output.term_formula.formulas().iter().next().unwrap();
        assert_eq!(formula.kind, FormulaKind::TypeAssertion);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert!(formula.expected_types.is_empty());
        assert!(formula.candidate_set.is_none());
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
        assert!(output.term_formula.candidate_sets().is_empty());
        assert!(output.term_formula.facts().is_empty());
        assert!(output.term_formula.diagnostics().is_empty());

        let assert_invalid_output = |invalid| {
            let invalid_result =
                assert_source_reserved_variable_type_assertion_output(&invalid).map(|()| invalid);
            assert_eq!(
                source_reserved_variable_type_assertion_result_detail_keys(
                    invalid_result,
                    super::TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_FIVE_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
                ),
                vec![
                    super::TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_FIVE_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY
                        .to_owned()
                ]
            );
        };
        for removed in mode_names {
            let mut invalid =
                super::source_local_object_mode_long_chain_five_hop_asserted_head_output(
                    &exact,
                    module.clone(),
                    &symbols,
                )
                .unwrap();
            invalid
                .payload
                .reserve
                .mode_expansions
                .retain(|symbol, _| source_mode_symbol_spelling(symbol) != Some(removed));
            assert_invalid_output(invalid);
        }
        let exact_output = || {
            super::source_local_object_mode_long_chain_five_hop_asserted_head_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .unwrap()
        };
        let mut corruptions = Vec::new();
        let mut invalid = exact_output();
        invalid.subject_binding = BindingId::new(1);
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.payload.subject_lookup_ordinal = 2;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.spelling = mode_names[5].to_owned();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.spelling = "OtherLongMode".to_owned();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.head = invalid.subject_result_input.head.clone();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.head = TypeHeadInput::BuiltinSet;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.head = TypeHeadInput::BuiltinSet;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.source_range = invalid.asserted_type_input.source_range;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.site = invalid.asserted_type_input.site.clone();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.source_range = invalid.subject_result_input.source_range;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.site = invalid.subject_result_input.site.clone();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        let binding_head = invalid.subject_result_input.head.clone();
        let (_, binding_expansion) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[6]))
            .unwrap();
        binding_expansion.radix.head = binding_head;
        corruptions.push(invalid);
        for mode_name in &mode_names[1..] {
            let mut invalid = exact_output();
            let (_, expansion) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(*mode_name))
                .unwrap();
            expansion.radix.spelling = "set".to_owned();
            expansion.radix.head = TypeHeadInput::BuiltinSet;
            corruptions.push(invalid);
        }
        let mut invalid = exact_output();
        let (_, base_expansion) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[0]))
            .unwrap();
        base_expansion.radix.source_range = range(source_id, 0, 1);
        corruptions.push(invalid);
        let mut invalid = exact_output();
        let (_, base_expansion) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[0]))
            .unwrap();
        base_expansion.radix.spelling = "set".to_owned();
        base_expansion.radix.head = TypeHeadInput::BuiltinSet;
        corruptions.push(invalid);
        for invalid in corruptions {
            assert_invalid_output(invalid);
        }
        assert_source_reserved_variable_type_assertion_output(&exact_output())
            .expect("corrupting cloned Task 231 outputs must not mutate the exact output");

        let assert_extraction_gap = |ast, context: &str| {
            assert_eq!(
                source_type_elaboration_detail_keys(&ast, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()],
                "{context}"
            );
        };
        fn next_permutation(order: &mut [usize; 7]) -> bool {
            let Some(pivot) = (0..order.len() - 1)
                .rev()
                .find(|&index| order[index] < order[index + 1])
            else {
                return false;
            };
            let successor = (pivot + 1..order.len())
                .rev()
                .find(|&index| order[pivot] < order[index])
                .expect("a lexicographic successor must exist after the pivot");
            order.swap(pivot, successor);
            order[pivot + 1..].reverse();
            true
        }
        let ordered = exact_modes();
        let public_dispatch_orders = [
            [1, 0, 2, 3, 4, 5, 6],
            [0, 2, 1, 3, 4, 5, 6],
            [0, 1, 3, 2, 4, 5, 6],
            [0, 1, 2, 4, 3, 5, 6],
            [0, 1, 2, 3, 5, 4, 6],
            [0, 1, 2, 3, 4, 6, 5],
            [1, 2, 3, 4, 5, 6, 0],
            [6, 5, 4, 3, 2, 1, 0],
        ];
        let mut order = [0, 1, 2, 3, 4, 5, 6];
        let mut permutation_count = 0;
        let mut public_dispatch_representative_count = 0;
        while next_permutation(&mut order) {
            permutation_count += 1;
            let permuted = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                vec![
                    ordered[order[0]],
                    ordered[order[1]],
                    ordered[order[2]],
                    ordered[order[3]],
                    ordered[order[4]],
                    ordered[order[5]],
                    ordered[order[6]],
                ],
                reserve(),
                theorem,
            );
            assert!(
                super::extract_source_local_object_mode_long_chain_five_hop_asserted_head(
                    &permuted,
                    module.clone(),
                    &symbols,
                )
                .is_none(),
                "definition permutation {order:?} must not reach Task 231"
            );
            if public_dispatch_orders.contains(&order) {
                public_dispatch_representative_count += 1;
                assert_extraction_gap(
                    permuted,
                    &format!("public definition permutation {order:?}"),
                );
            }
        }
        assert_eq!(permutation_count, 5_039);
        assert_eq!(public_dispatch_representative_count, 8);
        let mut mode_near_misses = vec![
            Vec::<ModeDefinitionSpec>::new(),
            {
                let mut modes = exact_modes();
                modes.push(modes[0]);
                modes
            },
            {
                let mut modes = exact_modes();
                modes.push(mode_definition_with_label(
                    "UnusedLongMode",
                    "UnusedLongModeDef",
                    ReserveTypeShape::QualifiedSymbol(mode_names[6]),
                ));
                modes
            },
        ];
        for index in 0..mode_names.len() {
            let mut missing = exact_modes();
            missing.remove(index);
            mode_near_misses.push(missing);
            let mut duplicate = exact_modes();
            duplicate.push(duplicate[index]);
            mode_near_misses.push(duplicate);
            let mut malformed = exact_modes();
            malformed[index].rhs_shape = if index == 0 {
                ReserveTypeShape::Builtin("set")
            } else {
                ReserveTypeShape::QualifiedSymbol(mode_names[(index + 1) % mode_names.len()])
            };
            mode_near_misses.push(malformed);
            let mut wrong_label = exact_modes();
            wrong_label[index].label = Some("WrongLongModeDef");
            mode_near_misses.push(wrong_label);
            let mut wrong_spelling = exact_modes();
            wrong_spelling[index].pattern = "OtherLongMode";
            mode_near_misses.push(wrong_spelling);
            let mut recovered = exact_modes();
            recovered[index].recovered = true;
            mode_near_misses.push(recovered);
            let mut contextual = exact_modes();
            contextual[index].local_context = true;
            mode_near_misses.push(contextual);
            let mut parameterized = exact_modes();
            parameterized[index].parameterized_pattern = true;
            mode_near_misses.push(parameterized);
            let mut args = exact_modes();
            args[index].rhs_shape = if index == 0 {
                ReserveTypeShape::QualifiedSymbolWithArgs("object")
            } else {
                ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[index - 1])
            };
            mode_near_misses.push(args);
            let mut attrs = exact_modes();
            attrs[index].rhs_shape = if index == 0 {
                ReserveTypeShape::AttributedObject
            } else {
                ReserveTypeShape::AttributedQualifiedSymbol(mode_names[index - 1])
            };
            mode_near_misses.push(attrs);
        }
        for (index, modes) in mode_near_misses.into_iter().enumerate() {
            assert_extraction_gap(
                mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    modes,
                    reserve(),
                    theorem,
                ),
                &format!("mode near miss {index}"),
            );
        }

        let mut source_near_misses = Vec::new();
        let mut deeper_modes = exact_modes();
        deeper_modes.push(mode_definition_with_label(
            "ChainObjectMode7",
            "ChainObjectMode7Def",
            ReserveTypeShape::QualifiedSymbol(mode_names[6]),
        ));
        source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            deeper_modes,
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("ChainObjectMode7"),
            )],
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[6]),
                ..theorem
            },
        ));
        for bad_reserve in [
            ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[6]),
            ReserveTypeShape::AttributedQualifiedSymbol(mode_names[6]),
            ReserveTypeShape::Builtin("object"),
            ReserveTypeShape::Builtin("set"),
        ] {
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(vec!["x"], bad_reserve)],
                theorem,
            ));
        }
        source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            exact_modes(),
            vec![reserve_item(
                vec!["y"],
                ReserveTypeShape::QualifiedSymbol(mode_names[6]),
            )],
            theorem,
        ));
        source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            exact_modes(),
            vec![
                reserve_item(vec!["x"], ReserveTypeShape::QualifiedSymbol(mode_names[6])),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("object")),
            ],
            theorem,
        ));
        for bad_reserve in [
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol(mode_names[5]),
            )],
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("OtherLongMode"),
            )],
            vec![reserve_item(
                vec!["x", "y"],
                ReserveTypeShape::QualifiedSymbol(mode_names[6]),
            )],
        ] {
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                bad_reserve,
                theorem,
            ));
        }
        let theorem_near_misses = [
            IdentifierTypeAssertionTheoremSpec {
                label: "OtherPayloadBoundary",
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                subject: "y",
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::Builtin("object"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::Builtin("set"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[0]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[2]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[3]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[4]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[5]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[6]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol("OtherLongMode"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[1]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::AttributedQualifiedSymbol(mode_names[1]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                negated: true,
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                status: Some("open"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                recovered_label: true,
                ..theorem
            },
        ];
        for near_miss_theorem in theorem_near_misses {
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                near_miss_theorem,
            ));
        }
        source_near_misses.push(
            modes_then_empty_definition_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                theorem,
            ),
        );
        for (index, near_miss) in source_near_misses.into_iter().enumerate() {
            assert_extraction_gap(near_miss, &format!("source near miss {index}"));
        }

        for imported_index in 0..mode_names.len() {
            let local_object_modes = mode_names
                .iter()
                .enumerate()
                .filter(|(index, _)| *index != imported_index)
                .map(|(_, spelling)| (*spelling, SymbolKind::Mode))
                .collect::<Vec<_>>();
            for imported in [
                vec![(mode_names[imported_index], SymbolKind::Mode)],
                vec![
                    (mode_names[imported_index], SymbolKind::Mode),
                    (mode_names[imported_index], SymbolKind::Mode),
                ],
            ] {
                let imported_symbols = source_local_and_imported_symbols_env(
                    module.clone(),
                    &local_object_modes,
                    imported.as_slice(),
                );
                assert_eq!(
                    source_type_elaboration_detail_keys(&exact, module.clone(), &imported_symbols),
                    vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
                );
            }
        }
        let unrelated_import = source_local_and_imported_symbols_env(
            module.clone(),
            &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
            &[("UnrelatedLongMode", SymbolKind::Mode)],
        );
        assert!(
            super::extract_source_local_object_mode_long_chain_five_hop_asserted_head(
                &exact,
                module.clone(),
                &unrelated_import,
            )
            .is_some()
        );
        for (asserted_spelling, asserted_head_symbols) in [
            (
                "ImportedLongMode",
                source_local_and_imported_symbols_env(
                    module.clone(),
                    &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
                    &[("ImportedLongMode", SymbolKind::Mode)],
                ),
            ),
            (
                "AmbiguousLongMode",
                source_local_and_imported_symbols_env(
                    module.clone(),
                    &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
                    &[
                        ("AmbiguousLongMode", SymbolKind::Mode),
                        ("AmbiguousLongMode", SymbolKind::Mode),
                    ],
                ),
            ),
        ] {
            let near_miss = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(asserted_spelling),
                    ..theorem
                },
            );
            assert_eq!(
                source_type_elaboration_detail_keys(
                    &near_miss,
                    module.clone(),
                    &asserted_head_symbols,
                ),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
    }

    #[test]
    fn source_local_object_mode_long_chain_four_hop_asserted_head_consumes_seven_expansions() {
        let source_id = source_id(229);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("local_object_mode_long_chain_four_hop_asserted_head"),
        );
        let mode_names = [
            "BaseObjectMode",
            "ChainObjectMode1",
            "ChainObjectMode2",
            "ChainObjectMode3",
            "ChainObjectMode4",
            "ChainObjectMode5",
            "ChainObjectMode6",
        ];
        let mode_labels = [
            "BaseObjectModeDef",
            "ChainObjectMode1Def",
            "ChainObjectMode2Def",
            "ChainObjectMode3Def",
            "ChainObjectMode4Def",
            "ChainObjectMode5Def",
            "ChainObjectMode6Def",
        ];
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                (mode_names[0], SymbolKind::Mode),
                (mode_names[1], SymbolKind::Mode),
                (mode_names[2], SymbolKind::Mode),
                (mode_names[3], SymbolKind::Mode),
                (mode_names[4], SymbolKind::Mode),
                (mode_names[5], SymbolKind::Mode),
                (mode_names[6], SymbolKind::Mode),
                ("ChainObjectMode7", SymbolKind::Mode),
                ("OtherLongObjectMode", SymbolKind::Mode),
            ],
        );
        let theorem = IdentifierTypeAssertionTheoremSpec {
            status: None,
            label: "LongLocalObjectModeFourHopAssertedHeadPayloadBoundary",
            subject: "x",
            asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[2]),
            negated: false,
            recovered_label: false,
        };
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    mode_names[0],
                    mode_labels[0],
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition_with_label(
                    mode_names[1],
                    mode_labels[1],
                    ReserveTypeShape::QualifiedSymbol(mode_names[0]),
                ),
                mode_definition_with_label(
                    mode_names[2],
                    mode_labels[2],
                    ReserveTypeShape::QualifiedSymbol(mode_names[1]),
                ),
                mode_definition_with_label(
                    mode_names[3],
                    mode_labels[3],
                    ReserveTypeShape::QualifiedSymbol(mode_names[2]),
                ),
                mode_definition_with_label(
                    mode_names[4],
                    mode_labels[4],
                    ReserveTypeShape::QualifiedSymbol(mode_names[3]),
                ),
                mode_definition_with_label(
                    mode_names[5],
                    mode_labels[5],
                    ReserveTypeShape::QualifiedSymbol(mode_names[4]),
                ),
                mode_definition_with_label(
                    mode_names[6],
                    mode_labels[6],
                    ReserveTypeShape::QualifiedSymbol(mode_names[5]),
                ),
            ]
        };
        let reserve = || {
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol(mode_names[6]),
            )]
        };
        let exact = mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            exact_modes(),
            reserve(),
            theorem,
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        assert!(
            extract_source_local_mode_long_chain_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 175 must reject the Task 229 four-hop source"
        );
        assert!(
            super::extract_source_local_mode_long_chain_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 199 must reject the Task 229 four-hop source"
        );
        assert!(
            super::extract_source_local_mode_long_chain_two_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 224 must reject the Task 229 four-hop source"
        );
        assert!(
            super::extract_source_local_object_mode_long_chain_two_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 225 must reject the Task 229 object-terminal four-hop source"
        );
        assert!(
            super::extract_source_local_mode_long_chain_three_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 226 must reject the Task 229 four-hop source"
        );
        assert!(
            super::extract_source_local_object_mode_long_chain_three_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 227 must reject the Task 229 object-terminal four-hop source"
        );
        assert!(
            super::extract_source_local_mode_long_chain_four_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 228 must reject the Task 229 object-terminal four-hop source"
        );
        let preexisting_owner_results = [
            extract_source_reserved_variable_type_assertion(&exact, module.clone(), &symbols),
            super::extract_source_reserved_object_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_local_object_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_chained_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_chained_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_chained_local_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_chained_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_chained_local_mode_radix_asserted_head(&exact, module.clone(), &symbols),
            extract_source_chained_local_object_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_two_edge_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_object_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_three_edge_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_object_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_four_edge_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_four_edge_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            super::extract_source_four_edge_local_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            super::extract_source_four_edge_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_four_edge_local_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_four_edge_local_object_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_mode_long_chain_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_object_mode_long_chain_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            super::extract_source_local_mode_long_chain_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            super::extract_source_local_object_mode_long_chain_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
        ];
        assert_eq!(preexisting_owner_results.len(), 34);
        assert!(
            preexisting_owner_results
                .into_iter()
                .all(|result| result.is_none()),
            "all 34 legacy type-assertion owner routes must reject Task 229"
        );
        let payload = super::extract_source_local_object_mode_long_chain_four_hop_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact seven-expansion four-hop source should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 7);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(payload.subject_lookup_ordinal, 1);
        let source_binding = &payload.reserve.bridge.bindings()[0];
        assert_eq!(source_binding.type_spelling, mode_names[6]);
        assert_eq!(payload.asserted_type.spelling, mode_names[2]);
        assert_ne!(payload.asserted_type.head, source_binding.type_head);
        assert!(matches!(source_binding.type_head, TypeHeadInput::Symbol(_)));
        assert_ne!(payload.asserted_type_site, payload.subject_site);
        assert_ne!(payload.asserted_type.range, source_binding.type_range);
        let binding_expansion = payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[6]))
            .map(|(_, expansion)| expansion)
            .expect("ChainObjectMode6 binding expansion should exist");
        assert_eq!(binding_expansion.radix.spelling, mode_names[5]);
        assert_ne!(binding_expansion.radix.head, payload.asserted_type.head);
        let TypeHeadInput::Symbol(first_intermediate_symbol) = &binding_expansion.radix.head else {
            panic!("ChainObjectMode5 first relation intermediate should resolve to a symbol");
        };
        let first_intermediate_expansion = payload
            .reserve
            .mode_expansions
            .get(first_intermediate_symbol)
            .expect("ChainObjectMode5 first intermediate expansion should exist");
        assert_eq!(first_intermediate_expansion.radix.spelling, mode_names[4]);
        assert_ne!(
            first_intermediate_expansion.radix.head,
            payload.asserted_type.head
        );
        let TypeHeadInput::Symbol(second_intermediate_symbol) =
            &first_intermediate_expansion.radix.head
        else {
            panic!("ChainObjectMode4 second relation intermediate should resolve to a symbol");
        };
        let second_intermediate_expansion = payload
            .reserve
            .mode_expansions
            .get(second_intermediate_symbol)
            .expect("ChainObjectMode4 second intermediate expansion should exist");
        assert_eq!(second_intermediate_expansion.radix.spelling, mode_names[3]);
        assert_ne!(
            second_intermediate_expansion.radix.head,
            payload.asserted_type.head
        );
        let TypeHeadInput::Symbol(third_intermediate_symbol) =
            &second_intermediate_expansion.radix.head
        else {
            panic!("ChainObjectMode3 third relation intermediate should resolve to a symbol");
        };
        let third_intermediate_expansion = payload
            .reserve
            .mode_expansions
            .get(third_intermediate_symbol)
            .expect("ChainObjectMode3 third intermediate expansion should exist");
        assert_eq!(third_intermediate_expansion.radix.spelling, mode_names[2]);
        assert_eq!(
            third_intermediate_expansion.radix.head,
            payload.asserted_type.head
        );

        let output = super::source_local_object_mode_long_chain_four_hop_asserted_head_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact seven-expansion asserted-head source should reach checker");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 229 checker payload invariants should hold");
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert_eq!(output.subject_result_input.spelling, mode_names[6]);
        assert_eq!(output.asserted_type_input.spelling, mode_names[2]);
        assert_ne!(
            output.subject_result_input.head,
            output.asserted_type_input.head
        );
        assert!(matches!(
            output.asserted_type_input.head,
            TypeHeadInput::Symbol(_)
        ));
        assert_ne!(
            output.subject_result_input.site,
            output.asserted_type_input.site
        );
        assert_ne!(
            output.subject_result_input.source_range,
            output.asserted_type_input.source_range
        );
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[0]))
            .map(|(_, expansion)| &expansion.radix)
            .expect("BaseObjectMode terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("one normalized builtin-object type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
        let (_, term) = output.term_formula.terms().iter().next().unwrap();
        assert_eq!(term.status, TermStatus::Inferred);
        assert!(term.deferred.is_empty());
        let (_, formula) = output.term_formula.formulas().iter().next().unwrap();
        assert_eq!(formula.kind, FormulaKind::TypeAssertion);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert!(formula.expected_types.is_empty());
        assert!(formula.candidate_set.is_none());
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
        assert!(output.term_formula.candidate_sets().is_empty());
        assert!(output.term_formula.facts().is_empty());
        assert!(output.term_formula.diagnostics().is_empty());

        let assert_invalid_output = |invalid| {
            let invalid_result =
                assert_source_reserved_variable_type_assertion_output(&invalid).map(|()| invalid);
            assert_eq!(
                source_reserved_variable_type_assertion_result_detail_keys(
                    invalid_result,
                    super::TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_FOUR_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
                ),
                vec![
                    super::TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_FOUR_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY
                        .to_owned()
                ]
            );
        };
        for removed in mode_names {
            let mut invalid =
                super::source_local_object_mode_long_chain_four_hop_asserted_head_output(
                    &exact,
                    module.clone(),
                    &symbols,
                )
                .unwrap();
            invalid
                .payload
                .reserve
                .mode_expansions
                .retain(|symbol, _| source_mode_symbol_spelling(symbol) != Some(removed));
            assert_invalid_output(invalid);
        }
        let exact_output = || {
            super::source_local_object_mode_long_chain_four_hop_asserted_head_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .unwrap()
        };
        let mut corruptions = Vec::new();
        let mut invalid = exact_output();
        invalid.subject_binding = BindingId::new(1);
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.payload.subject_lookup_ordinal = 2;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.spelling = mode_names[5].to_owned();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.spelling = "OtherLongObjectMode".to_owned();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.head = invalid.subject_result_input.head.clone();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.head = TypeHeadInput::BuiltinSet;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.head = TypeHeadInput::BuiltinSet;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.source_range = invalid.asserted_type_input.source_range;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.site = invalid.asserted_type_input.site.clone();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.source_range = invalid.subject_result_input.source_range;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.site = invalid.subject_result_input.site.clone();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        let binding_head = invalid.subject_result_input.head.clone();
        let (_, binding_expansion) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[6]))
            .unwrap();
        binding_expansion.radix.head = binding_head;
        corruptions.push(invalid);
        for mode_name in &mode_names[1..] {
            let mut invalid = exact_output();
            let (_, expansion) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(*mode_name))
                .unwrap();
            expansion.radix.spelling = "set".to_owned();
            expansion.radix.head = TypeHeadInput::BuiltinSet;
            corruptions.push(invalid);
        }
        let mut invalid = exact_output();
        let (_, base_expansion) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[0]))
            .unwrap();
        base_expansion.radix.source_range = range(source_id, 0, 1);
        corruptions.push(invalid);
        let mut invalid = exact_output();
        let (_, base_expansion) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[0]))
            .unwrap();
        base_expansion.radix.spelling = "set".to_owned();
        base_expansion.radix.head = TypeHeadInput::BuiltinSet;
        corruptions.push(invalid);
        for invalid in corruptions {
            assert_invalid_output(invalid);
        }
        assert_source_reserved_variable_type_assertion_output(&exact_output())
            .expect("corrupting cloned Task 229 outputs must not mutate the exact output");

        let assert_extraction_gap = |ast, context: &str| {
            assert_eq!(
                source_type_elaboration_detail_keys(&ast, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()],
                "{context}"
            );
        };
        fn next_permutation(order: &mut [usize; 7]) -> bool {
            let Some(pivot) = (0..order.len() - 1)
                .rev()
                .find(|&index| order[index] < order[index + 1])
            else {
                return false;
            };
            let successor = (pivot + 1..order.len())
                .rev()
                .find(|&index| order[pivot] < order[index])
                .expect("a lexicographic successor must exist after the pivot");
            order.swap(pivot, successor);
            order[pivot + 1..].reverse();
            true
        }
        let ordered = exact_modes();
        let public_dispatch_orders = [
            [1, 0, 2, 3, 4, 5, 6],
            [0, 2, 1, 3, 4, 5, 6],
            [0, 1, 3, 2, 4, 5, 6],
            [0, 1, 2, 4, 3, 5, 6],
            [0, 1, 2, 3, 5, 4, 6],
            [0, 1, 2, 3, 4, 6, 5],
            [1, 2, 3, 4, 5, 6, 0],
            [6, 5, 4, 3, 2, 1, 0],
        ];
        let mut order = [0, 1, 2, 3, 4, 5, 6];
        let mut permutation_count = 0;
        let mut public_dispatch_representative_count = 0;
        while next_permutation(&mut order) {
            permutation_count += 1;
            let permuted = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                vec![
                    ordered[order[0]],
                    ordered[order[1]],
                    ordered[order[2]],
                    ordered[order[3]],
                    ordered[order[4]],
                    ordered[order[5]],
                    ordered[order[6]],
                ],
                reserve(),
                theorem,
            );
            assert!(
                super::extract_source_local_object_mode_long_chain_four_hop_asserted_head(
                    &permuted,
                    module.clone(),
                    &symbols,
                )
                .is_none(),
                "definition permutation {order:?} must not reach Task 229"
            );
            if public_dispatch_orders.contains(&order) {
                public_dispatch_representative_count += 1;
                assert_extraction_gap(
                    permuted,
                    &format!("public definition permutation {order:?}"),
                );
            }
        }
        assert_eq!(permutation_count, 5_039);
        assert_eq!(public_dispatch_representative_count, 8);
        let mut mode_near_misses = vec![
            Vec::<ModeDefinitionSpec>::new(),
            {
                let mut modes = exact_modes();
                modes.push(modes[0]);
                modes
            },
            {
                let mut modes = exact_modes();
                modes.push(mode_definition_with_label(
                    "UnusedLongObjectMode",
                    "UnusedLongObjectModeDef",
                    ReserveTypeShape::QualifiedSymbol(mode_names[6]),
                ));
                modes
            },
        ];
        for index in 0..mode_names.len() {
            let mut missing = exact_modes();
            missing.remove(index);
            mode_near_misses.push(missing);
            let mut duplicate = exact_modes();
            duplicate.push(duplicate[index]);
            mode_near_misses.push(duplicate);
            let mut malformed = exact_modes();
            malformed[index].rhs_shape = if index == 0 {
                ReserveTypeShape::Builtin("set")
            } else {
                ReserveTypeShape::QualifiedSymbol(mode_names[(index + 1) % mode_names.len()])
            };
            mode_near_misses.push(malformed);
            let mut wrong_label = exact_modes();
            wrong_label[index].label = Some("WrongLongObjectModeDef");
            mode_near_misses.push(wrong_label);
            let mut wrong_spelling = exact_modes();
            wrong_spelling[index].pattern = "OtherLongObjectMode";
            mode_near_misses.push(wrong_spelling);
            let mut recovered = exact_modes();
            recovered[index].recovered = true;
            mode_near_misses.push(recovered);
            let mut contextual = exact_modes();
            contextual[index].local_context = true;
            mode_near_misses.push(contextual);
            let mut parameterized = exact_modes();
            parameterized[index].parameterized_pattern = true;
            mode_near_misses.push(parameterized);
            let mut args = exact_modes();
            args[index].rhs_shape = if index == 0 {
                ReserveTypeShape::QualifiedSymbolWithArgs("object")
            } else {
                ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[index - 1])
            };
            mode_near_misses.push(args);
            let mut attrs = exact_modes();
            attrs[index].rhs_shape = if index == 0 {
                ReserveTypeShape::AttributedObject
            } else {
                ReserveTypeShape::AttributedQualifiedSymbol(mode_names[index - 1])
            };
            mode_near_misses.push(attrs);
        }
        for (index, modes) in mode_near_misses.into_iter().enumerate() {
            assert_extraction_gap(
                mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    modes,
                    reserve(),
                    theorem,
                ),
                &format!("mode near miss {index}"),
            );
        }

        let mut source_near_misses = Vec::new();
        let mut deeper_modes = exact_modes();
        deeper_modes.push(mode_definition_with_label(
            "ChainObjectMode7",
            "ChainObjectMode7Def",
            ReserveTypeShape::QualifiedSymbol(mode_names[6]),
        ));
        source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            deeper_modes,
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("ChainObjectMode7"),
            )],
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[6]),
                ..theorem
            },
        ));
        for bad_reserve in [
            ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[6]),
            ReserveTypeShape::AttributedQualifiedSymbol(mode_names[6]),
            ReserveTypeShape::Builtin("set"),
            ReserveTypeShape::Builtin("object"),
        ] {
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(vec!["x"], bad_reserve)],
                theorem,
            ));
        }
        source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            exact_modes(),
            vec![reserve_item(
                vec!["y"],
                ReserveTypeShape::QualifiedSymbol(mode_names[6]),
            )],
            theorem,
        ));
        source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            exact_modes(),
            vec![
                reserve_item(vec!["x"], ReserveTypeShape::QualifiedSymbol(mode_names[6])),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("object")),
            ],
            theorem,
        ));
        for bad_reserve in [
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol(mode_names[5]),
            )],
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("OtherLongObjectMode"),
            )],
            vec![reserve_item(
                vec!["x", "y"],
                ReserveTypeShape::QualifiedSymbol(mode_names[6]),
            )],
        ] {
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                bad_reserve,
                theorem,
            ));
        }
        let theorem_near_misses = [
            IdentifierTypeAssertionTheoremSpec {
                label: "OtherPayloadBoundary",
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                subject: "y",
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::Builtin("set"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::Builtin("object"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[0]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[1]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[3]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[4]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[5]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[6]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol("OtherLongObjectMode"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[2]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::AttributedQualifiedSymbol(mode_names[2]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                negated: true,
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                status: Some("open"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                recovered_label: true,
                ..theorem
            },
        ];
        for near_miss_theorem in theorem_near_misses {
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                near_miss_theorem,
            ));
        }
        source_near_misses.push(
            modes_then_empty_definition_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                theorem,
            ),
        );
        for (index, near_miss) in source_near_misses.into_iter().enumerate() {
            assert_extraction_gap(near_miss, &format!("source near miss {index}"));
        }

        for imported_index in 0..mode_names.len() {
            let local_modes = mode_names
                .iter()
                .enumerate()
                .filter(|(index, _)| *index != imported_index)
                .map(|(_, spelling)| (*spelling, SymbolKind::Mode))
                .collect::<Vec<_>>();
            for imported in [
                vec![(mode_names[imported_index], SymbolKind::Mode)],
                vec![
                    (mode_names[imported_index], SymbolKind::Mode),
                    (mode_names[imported_index], SymbolKind::Mode),
                ],
            ] {
                let imported_symbols = source_local_and_imported_symbols_env(
                    module.clone(),
                    &local_modes,
                    imported.as_slice(),
                );
                assert_eq!(
                    source_type_elaboration_detail_keys(&exact, module.clone(), &imported_symbols),
                    vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
                );
            }
        }
        let unrelated_import = source_local_and_imported_symbols_env(
            module.clone(),
            &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
            &[("UnrelatedLongObjectMode", SymbolKind::Mode)],
        );
        assert!(
            super::extract_source_local_object_mode_long_chain_four_hop_asserted_head(
                &exact,
                module.clone(),
                &unrelated_import,
            )
            .is_some()
        );
        for (asserted_spelling, asserted_head_symbols) in [
            (
                "ImportedLongObjectMode",
                source_local_and_imported_symbols_env(
                    module.clone(),
                    &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
                    &[("ImportedLongObjectMode", SymbolKind::Mode)],
                ),
            ),
            (
                "AmbiguousLongObjectMode",
                source_local_and_imported_symbols_env(
                    module.clone(),
                    &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
                    &[
                        ("AmbiguousLongObjectMode", SymbolKind::Mode),
                        ("AmbiguousLongObjectMode", SymbolKind::Mode),
                    ],
                ),
            ),
        ] {
            let near_miss = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(asserted_spelling),
                    ..theorem
                },
            );
            assert_eq!(
                source_type_elaboration_detail_keys(
                    &near_miss,
                    module.clone(),
                    &asserted_head_symbols,
                ),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
    }
    #[test]
    fn source_local_object_mode_long_chain_three_hop_asserted_head_consumes_seven_expansions() {
        let source_id = source_id(227);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("local_object_mode_long_chain_three_hop_asserted_head"),
        );
        let mode_names = [
            "BaseObjectMode",
            "ChainObjectMode1",
            "ChainObjectMode2",
            "ChainObjectMode3",
            "ChainObjectMode4",
            "ChainObjectMode5",
            "ChainObjectMode6",
        ];
        let mode_labels = [
            "BaseObjectModeDef",
            "ChainObjectMode1Def",
            "ChainObjectMode2Def",
            "ChainObjectMode3Def",
            "ChainObjectMode4Def",
            "ChainObjectMode5Def",
            "ChainObjectMode6Def",
        ];
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                (mode_names[0], SymbolKind::Mode),
                (mode_names[1], SymbolKind::Mode),
                (mode_names[2], SymbolKind::Mode),
                (mode_names[3], SymbolKind::Mode),
                (mode_names[4], SymbolKind::Mode),
                (mode_names[5], SymbolKind::Mode),
                (mode_names[6], SymbolKind::Mode),
                ("ChainObjectMode7", SymbolKind::Mode),
                ("OtherLongObjectMode", SymbolKind::Mode),
            ],
        );
        let theorem = IdentifierTypeAssertionTheoremSpec {
            status: None,
            label: "LongLocalObjectModeThreeHopAssertedHeadPayloadBoundary",
            subject: "x",
            asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[3]),
            negated: false,
            recovered_label: false,
        };
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    mode_names[0],
                    mode_labels[0],
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition_with_label(
                    mode_names[1],
                    mode_labels[1],
                    ReserveTypeShape::QualifiedSymbol(mode_names[0]),
                ),
                mode_definition_with_label(
                    mode_names[2],
                    mode_labels[2],
                    ReserveTypeShape::QualifiedSymbol(mode_names[1]),
                ),
                mode_definition_with_label(
                    mode_names[3],
                    mode_labels[3],
                    ReserveTypeShape::QualifiedSymbol(mode_names[2]),
                ),
                mode_definition_with_label(
                    mode_names[4],
                    mode_labels[4],
                    ReserveTypeShape::QualifiedSymbol(mode_names[3]),
                ),
                mode_definition_with_label(
                    mode_names[5],
                    mode_labels[5],
                    ReserveTypeShape::QualifiedSymbol(mode_names[4]),
                ),
                mode_definition_with_label(
                    mode_names[6],
                    mode_labels[6],
                    ReserveTypeShape::QualifiedSymbol(mode_names[5]),
                ),
            ]
        };
        let reserve = || {
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol(mode_names[6]),
            )]
        };
        let exact = mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            exact_modes(),
            reserve(),
            theorem,
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        assert!(
            extract_source_local_mode_long_chain_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 175 must reject the Task 227 three-hop source"
        );
        assert!(
            super::extract_source_local_mode_long_chain_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 199 must reject the Task 227 three-hop source"
        );
        assert!(
            super::extract_source_local_mode_long_chain_two_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 224 must reject the Task 227 three-hop source"
        );
        assert!(
            super::extract_source_local_object_mode_long_chain_two_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 225 must reject the Task 227 object-terminal three-hop source"
        );
        assert!(
            super::extract_source_local_mode_long_chain_three_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 226 must reject the Task 227 object-terminal three-hop source"
        );
        let preexisting_owner_results = [
            extract_source_reserved_variable_type_assertion(&exact, module.clone(), &symbols),
            super::extract_source_reserved_object_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_local_object_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_chained_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_chained_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_chained_local_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_chained_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_chained_local_mode_radix_asserted_head(&exact, module.clone(), &symbols),
            extract_source_chained_local_object_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_two_edge_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_object_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_three_edge_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_object_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_four_edge_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_four_edge_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            super::extract_source_four_edge_local_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            super::extract_source_four_edge_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_four_edge_local_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_four_edge_local_object_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_mode_long_chain_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_object_mode_long_chain_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            super::extract_source_local_mode_long_chain_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            super::extract_source_local_object_mode_long_chain_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
        ];
        assert_eq!(preexisting_owner_results.len(), 34);
        assert!(
            preexisting_owner_results
                .into_iter()
                .all(|result| result.is_none()),
            "all 34 legacy type-assertion owner routes must reject Task 227"
        );
        let payload = super::extract_source_local_object_mode_long_chain_three_hop_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact seven-expansion three-hop source should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 7);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(payload.subject_lookup_ordinal, 1);
        let source_binding = &payload.reserve.bridge.bindings()[0];
        assert_eq!(source_binding.type_spelling, mode_names[6]);
        assert_eq!(payload.asserted_type.spelling, mode_names[3]);
        assert_ne!(payload.asserted_type.head, source_binding.type_head);
        assert!(matches!(source_binding.type_head, TypeHeadInput::Symbol(_)));
        assert_ne!(payload.asserted_type_site, payload.subject_site);
        assert_ne!(payload.asserted_type.range, source_binding.type_range);
        let binding_expansion = payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[6]))
            .map(|(_, expansion)| expansion)
            .expect("ChainObjectMode6 binding expansion should exist");
        assert_eq!(binding_expansion.radix.spelling, mode_names[5]);
        assert_ne!(binding_expansion.radix.head, payload.asserted_type.head);
        let TypeHeadInput::Symbol(first_intermediate_symbol) = &binding_expansion.radix.head else {
            panic!("ChainObjectMode5 first relation intermediate should resolve to a symbol");
        };
        let first_intermediate_expansion = payload
            .reserve
            .mode_expansions
            .get(first_intermediate_symbol)
            .expect("ChainObjectMode5 first intermediate expansion should exist");
        assert_eq!(first_intermediate_expansion.radix.spelling, mode_names[4]);
        assert_ne!(
            first_intermediate_expansion.radix.head,
            payload.asserted_type.head
        );
        let TypeHeadInput::Symbol(second_intermediate_symbol) =
            &first_intermediate_expansion.radix.head
        else {
            panic!("ChainObjectMode4 second relation intermediate should resolve to a symbol");
        };
        let second_intermediate_expansion = payload
            .reserve
            .mode_expansions
            .get(second_intermediate_symbol)
            .expect("ChainObjectMode4 second intermediate expansion should exist");
        assert_eq!(second_intermediate_expansion.radix.spelling, mode_names[3]);
        assert_eq!(
            second_intermediate_expansion.radix.head,
            payload.asserted_type.head
        );

        let output = super::source_local_object_mode_long_chain_three_hop_asserted_head_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact seven-expansion asserted-head source should reach checker");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 227 checker payload invariants should hold");
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert_eq!(output.subject_result_input.spelling, mode_names[6]);
        assert_eq!(output.asserted_type_input.spelling, mode_names[3]);
        assert_ne!(
            output.subject_result_input.head,
            output.asserted_type_input.head
        );
        assert!(matches!(
            output.asserted_type_input.head,
            TypeHeadInput::Symbol(_)
        ));
        assert_ne!(
            output.subject_result_input.site,
            output.asserted_type_input.site
        );
        assert_ne!(
            output.subject_result_input.source_range,
            output.asserted_type_input.source_range
        );
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[0]))
            .map(|(_, expansion)| &expansion.radix)
            .expect("BaseObjectMode terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("one normalized builtin-object type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
        let (_, term) = output.term_formula.terms().iter().next().unwrap();
        assert_eq!(term.status, TermStatus::Inferred);
        assert!(term.deferred.is_empty());
        let (_, formula) = output.term_formula.formulas().iter().next().unwrap();
        assert_eq!(formula.kind, FormulaKind::TypeAssertion);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert!(formula.expected_types.is_empty());
        assert!(formula.candidate_set.is_none());
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
        assert!(output.term_formula.candidate_sets().is_empty());
        assert!(output.term_formula.facts().is_empty());
        assert!(output.term_formula.diagnostics().is_empty());

        let assert_invalid_output = |invalid| {
            let invalid_result =
                assert_source_reserved_variable_type_assertion_output(&invalid).map(|()| invalid);
            assert_eq!(
                source_reserved_variable_type_assertion_result_detail_keys(
                    invalid_result,
                    super::TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_THREE_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
                ),
                vec![
                    super::TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_THREE_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY
                        .to_owned()
                ]
            );
        };
        for removed in mode_names {
            let mut invalid =
                super::source_local_object_mode_long_chain_three_hop_asserted_head_output(
                    &exact,
                    module.clone(),
                    &symbols,
                )
                .unwrap();
            invalid
                .payload
                .reserve
                .mode_expansions
                .retain(|symbol, _| source_mode_symbol_spelling(symbol) != Some(removed));
            assert_invalid_output(invalid);
        }
        let exact_output = || {
            super::source_local_object_mode_long_chain_three_hop_asserted_head_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .unwrap()
        };
        let mut corruptions = Vec::new();
        let mut invalid = exact_output();
        invalid.subject_binding = BindingId::new(1);
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.payload.subject_lookup_ordinal = 2;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.spelling = mode_names[5].to_owned();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.spelling = "OtherLongObjectMode".to_owned();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.head = invalid.subject_result_input.head.clone();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.head = TypeHeadInput::BuiltinSet;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.head = TypeHeadInput::BuiltinSet;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.source_range = invalid.asserted_type_input.source_range;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.site = invalid.asserted_type_input.site.clone();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.source_range = invalid.subject_result_input.source_range;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.site = invalid.subject_result_input.site.clone();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        let binding_head = invalid.subject_result_input.head.clone();
        let (_, binding_expansion) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[6]))
            .unwrap();
        binding_expansion.radix.head = binding_head;
        corruptions.push(invalid);
        for mode_name in &mode_names[1..] {
            let mut invalid = exact_output();
            let (_, expansion) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(*mode_name))
                .unwrap();
            expansion.radix.spelling = "set".to_owned();
            expansion.radix.head = TypeHeadInput::BuiltinSet;
            corruptions.push(invalid);
        }
        let mut invalid = exact_output();
        let (_, base_expansion) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[0]))
            .unwrap();
        base_expansion.radix.source_range = range(source_id, 0, 1);
        corruptions.push(invalid);
        let mut invalid = exact_output();
        let (_, base_expansion) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[0]))
            .unwrap();
        base_expansion.radix.spelling = "set".to_owned();
        base_expansion.radix.head = TypeHeadInput::BuiltinSet;
        corruptions.push(invalid);
        for invalid in corruptions {
            assert_invalid_output(invalid);
        }
        assert_source_reserved_variable_type_assertion_output(&exact_output())
            .expect("corrupting cloned Task 227 outputs must not mutate the exact output");

        let assert_extraction_gap = |ast, context: &str| {
            assert_eq!(
                source_type_elaboration_detail_keys(&ast, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()],
                "{context}"
            );
        };
        fn next_permutation(order: &mut [usize; 7]) -> bool {
            let Some(pivot) = (0..order.len() - 1)
                .rev()
                .find(|&index| order[index] < order[index + 1])
            else {
                return false;
            };
            let successor = (pivot + 1..order.len())
                .rev()
                .find(|&index| order[pivot] < order[index])
                .expect("a lexicographic successor must exist after the pivot");
            order.swap(pivot, successor);
            order[pivot + 1..].reverse();
            true
        }
        let ordered = exact_modes();
        let public_dispatch_orders = [
            [1, 0, 2, 3, 4, 5, 6],
            [0, 2, 1, 3, 4, 5, 6],
            [0, 1, 3, 2, 4, 5, 6],
            [0, 1, 2, 4, 3, 5, 6],
            [0, 1, 2, 3, 5, 4, 6],
            [0, 1, 2, 3, 4, 6, 5],
            [1, 2, 3, 4, 5, 6, 0],
            [6, 5, 4, 3, 2, 1, 0],
        ];
        let mut order = [0, 1, 2, 3, 4, 5, 6];
        let mut permutation_count = 0;
        let mut public_dispatch_representative_count = 0;
        while next_permutation(&mut order) {
            permutation_count += 1;
            let permuted = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                vec![
                    ordered[order[0]],
                    ordered[order[1]],
                    ordered[order[2]],
                    ordered[order[3]],
                    ordered[order[4]],
                    ordered[order[5]],
                    ordered[order[6]],
                ],
                reserve(),
                theorem,
            );
            assert!(
                super::extract_source_local_object_mode_long_chain_three_hop_asserted_head(
                    &permuted,
                    module.clone(),
                    &symbols,
                )
                .is_none(),
                "definition permutation {order:?} must not reach Task 227"
            );
            if public_dispatch_orders.contains(&order) {
                public_dispatch_representative_count += 1;
                assert_extraction_gap(
                    permuted,
                    &format!("public definition permutation {order:?}"),
                );
            }
        }
        assert_eq!(permutation_count, 5_039);
        assert_eq!(public_dispatch_representative_count, 8);
        let mut mode_near_misses = vec![
            Vec::<ModeDefinitionSpec>::new(),
            {
                let mut modes = exact_modes();
                modes.push(modes[0]);
                modes
            },
            {
                let mut modes = exact_modes();
                modes.push(mode_definition_with_label(
                    "UnusedLongObjectMode",
                    "UnusedLongObjectModeDef",
                    ReserveTypeShape::QualifiedSymbol(mode_names[6]),
                ));
                modes
            },
        ];
        for index in 0..mode_names.len() {
            let mut missing = exact_modes();
            missing.remove(index);
            mode_near_misses.push(missing);
            let mut duplicate = exact_modes();
            duplicate.push(duplicate[index]);
            mode_near_misses.push(duplicate);
            let mut malformed = exact_modes();
            malformed[index].rhs_shape = if index == 0 {
                ReserveTypeShape::Builtin("set")
            } else {
                ReserveTypeShape::QualifiedSymbol(mode_names[(index + 1) % mode_names.len()])
            };
            mode_near_misses.push(malformed);
            let mut wrong_label = exact_modes();
            wrong_label[index].label = Some("WrongLongModeDef");
            mode_near_misses.push(wrong_label);
            let mut wrong_spelling = exact_modes();
            wrong_spelling[index].pattern = "OtherLongObjectMode";
            mode_near_misses.push(wrong_spelling);
            let mut recovered = exact_modes();
            recovered[index].recovered = true;
            mode_near_misses.push(recovered);
            let mut contextual = exact_modes();
            contextual[index].local_context = true;
            mode_near_misses.push(contextual);
            let mut parameterized = exact_modes();
            parameterized[index].parameterized_pattern = true;
            mode_near_misses.push(parameterized);
            let mut args = exact_modes();
            args[index].rhs_shape = if index == 0 {
                ReserveTypeShape::QualifiedSymbolWithArgs("object")
            } else {
                ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[index - 1])
            };
            mode_near_misses.push(args);
            let mut attrs = exact_modes();
            attrs[index].rhs_shape = if index == 0 {
                ReserveTypeShape::AttributedObject
            } else {
                ReserveTypeShape::AttributedQualifiedSymbol(mode_names[index - 1])
            };
            mode_near_misses.push(attrs);
        }
        for (index, modes) in mode_near_misses.into_iter().enumerate() {
            assert_extraction_gap(
                mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    modes,
                    reserve(),
                    theorem,
                ),
                &format!("mode near miss {index}"),
            );
        }

        let mut source_near_misses = Vec::new();
        let mut deeper_modes = exact_modes();
        deeper_modes.push(mode_definition_with_label(
            "ChainObjectMode7",
            "ChainObjectMode7Def",
            ReserveTypeShape::QualifiedSymbol(mode_names[6]),
        ));
        source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            deeper_modes,
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("ChainObjectMode7"),
            )],
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[6]),
                ..theorem
            },
        ));
        for bad_reserve in [
            ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[6]),
            ReserveTypeShape::AttributedQualifiedSymbol(mode_names[6]),
            ReserveTypeShape::Builtin("object"),
            ReserveTypeShape::Builtin("set"),
        ] {
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(vec!["x"], bad_reserve)],
                theorem,
            ));
        }
        source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            exact_modes(),
            vec![reserve_item(
                vec!["y"],
                ReserveTypeShape::QualifiedSymbol(mode_names[6]),
            )],
            theorem,
        ));
        source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            exact_modes(),
            vec![
                reserve_item(vec!["x"], ReserveTypeShape::QualifiedSymbol(mode_names[6])),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("object")),
            ],
            theorem,
        ));
        for bad_reserve in [
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol(mode_names[5]),
            )],
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("OtherLongObjectMode"),
            )],
            vec![reserve_item(
                vec!["x", "y"],
                ReserveTypeShape::QualifiedSymbol(mode_names[6]),
            )],
        ] {
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                bad_reserve,
                theorem,
            ));
        }
        let theorem_near_misses = [
            IdentifierTypeAssertionTheoremSpec {
                label: "OtherPayloadBoundary",
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                subject: "y",
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::Builtin("object"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::Builtin("set"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[0]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[1]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[2]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[4]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[5]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[6]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol("OtherLongObjectMode"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[3]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::AttributedQualifiedSymbol(mode_names[3]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                negated: true,
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                status: Some("open"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                recovered_label: true,
                ..theorem
            },
        ];
        for near_miss_theorem in theorem_near_misses {
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                near_miss_theorem,
            ));
        }
        source_near_misses.push(
            modes_then_empty_definition_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                theorem,
            ),
        );
        for (index, near_miss) in source_near_misses.into_iter().enumerate() {
            assert_extraction_gap(near_miss, &format!("source near miss {index}"));
        }

        for imported_index in 0..mode_names.len() {
            let local_modes = mode_names
                .iter()
                .enumerate()
                .filter(|(index, _)| *index != imported_index)
                .map(|(_, spelling)| (*spelling, SymbolKind::Mode))
                .collect::<Vec<_>>();
            for imported in [
                vec![(mode_names[imported_index], SymbolKind::Mode)],
                vec![
                    (mode_names[imported_index], SymbolKind::Mode),
                    (mode_names[imported_index], SymbolKind::Mode),
                ],
            ] {
                let imported_symbols = source_local_and_imported_symbols_env(
                    module.clone(),
                    &local_modes,
                    imported.as_slice(),
                );
                assert_eq!(
                    source_type_elaboration_detail_keys(&exact, module.clone(), &imported_symbols),
                    vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
                );
            }
        }
        let unrelated_import = source_local_and_imported_symbols_env(
            module.clone(),
            &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
            &[("UnrelatedLongObjectMode", SymbolKind::Mode)],
        );
        assert!(
            super::extract_source_local_object_mode_long_chain_three_hop_asserted_head(
                &exact,
                module.clone(),
                &unrelated_import,
            )
            .is_some()
        );
        for (asserted_spelling, asserted_head_symbols) in [
            (
                "ImportedLongObjectMode",
                source_local_and_imported_symbols_env(
                    module.clone(),
                    &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
                    &[("ImportedLongObjectMode", SymbolKind::Mode)],
                ),
            ),
            (
                "AmbiguousLongObjectMode",
                source_local_and_imported_symbols_env(
                    module.clone(),
                    &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
                    &[
                        ("AmbiguousLongObjectMode", SymbolKind::Mode),
                        ("AmbiguousLongObjectMode", SymbolKind::Mode),
                    ],
                ),
            ),
        ] {
            let near_miss = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(asserted_spelling),
                    ..theorem
                },
            );
            assert_eq!(
                source_type_elaboration_detail_keys(
                    &near_miss,
                    module.clone(),
                    &asserted_head_symbols,
                ),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
    }

    #[test]
    fn source_local_mode_long_chain_two_hop_asserted_head_consumes_seven_expansions() {
        let source_id = source_id(224);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("local_mode_long_chain_two_hop_asserted_head"),
        );
        let mode_names = [
            "BaseMode",
            "ChainMode1",
            "ChainMode2",
            "ChainMode3",
            "ChainMode4",
            "ChainMode5",
            "ChainMode6",
        ];
        let mode_labels = [
            "BaseModeDef",
            "ChainMode1Def",
            "ChainMode2Def",
            "ChainMode3Def",
            "ChainMode4Def",
            "ChainMode5Def",
            "ChainMode6Def",
        ];
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                (mode_names[0], SymbolKind::Mode),
                (mode_names[1], SymbolKind::Mode),
                (mode_names[2], SymbolKind::Mode),
                (mode_names[3], SymbolKind::Mode),
                (mode_names[4], SymbolKind::Mode),
                (mode_names[5], SymbolKind::Mode),
                (mode_names[6], SymbolKind::Mode),
                ("ChainMode7", SymbolKind::Mode),
                ("OtherLongMode", SymbolKind::Mode),
            ],
        );
        let theorem = IdentifierTypeAssertionTheoremSpec {
            status: None,
            label: "LongLocalModeTwoHopAssertedHeadPayloadBoundary",
            subject: "x",
            asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[4]),
            negated: false,
            recovered_label: false,
        };
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    mode_names[0],
                    mode_labels[0],
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition_with_label(
                    mode_names[1],
                    mode_labels[1],
                    ReserveTypeShape::QualifiedSymbol(mode_names[0]),
                ),
                mode_definition_with_label(
                    mode_names[2],
                    mode_labels[2],
                    ReserveTypeShape::QualifiedSymbol(mode_names[1]),
                ),
                mode_definition_with_label(
                    mode_names[3],
                    mode_labels[3],
                    ReserveTypeShape::QualifiedSymbol(mode_names[2]),
                ),
                mode_definition_with_label(
                    mode_names[4],
                    mode_labels[4],
                    ReserveTypeShape::QualifiedSymbol(mode_names[3]),
                ),
                mode_definition_with_label(
                    mode_names[5],
                    mode_labels[5],
                    ReserveTypeShape::QualifiedSymbol(mode_names[4]),
                ),
                mode_definition_with_label(
                    mode_names[6],
                    mode_labels[6],
                    ReserveTypeShape::QualifiedSymbol(mode_names[5]),
                ),
            ]
        };
        let reserve = || {
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol(mode_names[6]),
            )]
        };
        let exact = mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            exact_modes(),
            reserve(),
            theorem,
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        assert!(
            extract_source_local_mode_long_chain_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 175 must reject the Task 224 two-hop source"
        );
        assert!(
            super::extract_source_local_mode_long_chain_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 199 must reject the Task 224 two-hop source"
        );
        let preexisting_owner_results = [
            extract_source_reserved_variable_type_assertion(&exact, module.clone(), &symbols),
            super::extract_source_reserved_object_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_local_object_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_chained_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_chained_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_chained_local_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_chained_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_chained_local_mode_radix_asserted_head(&exact, module.clone(), &symbols),
            extract_source_chained_local_object_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_two_edge_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_object_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_three_edge_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_object_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_four_edge_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_four_edge_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            super::extract_source_four_edge_local_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            super::extract_source_four_edge_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_four_edge_local_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_four_edge_local_object_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_mode_long_chain_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_object_mode_long_chain_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            super::extract_source_local_mode_long_chain_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            super::extract_source_local_object_mode_long_chain_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
        ];
        assert_eq!(preexisting_owner_results.len(), 34);
        assert!(
            preexisting_owner_results
                .into_iter()
                .all(|result| result.is_none()),
            "all 34 pre-existing type-assertion owner routes must reject Task 224"
        );
        let payload = super::extract_source_local_mode_long_chain_two_hop_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact seven-expansion two-hop source should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 7);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(payload.subject_lookup_ordinal, 1);
        let source_binding = &payload.reserve.bridge.bindings()[0];
        assert_eq!(source_binding.type_spelling, mode_names[6]);
        assert_eq!(payload.asserted_type.spelling, mode_names[4]);
        assert_ne!(payload.asserted_type.head, source_binding.type_head);
        assert!(matches!(source_binding.type_head, TypeHeadInput::Symbol(_)));
        assert_ne!(payload.asserted_type_site, payload.subject_site);
        assert_ne!(payload.asserted_type.range, source_binding.type_range);
        let binding_expansion = payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[6]))
            .map(|(_, expansion)| expansion)
            .expect("ChainMode6 binding expansion should exist");
        assert_eq!(binding_expansion.radix.spelling, mode_names[5]);
        assert_ne!(binding_expansion.radix.head, payload.asserted_type.head);
        let TypeHeadInput::Symbol(intermediate_symbol) = &binding_expansion.radix.head else {
            panic!("ChainMode5 relation intermediate should resolve to a symbol");
        };
        let intermediate_expansion = payload
            .reserve
            .mode_expansions
            .get(intermediate_symbol)
            .expect("ChainMode5 intermediate expansion should exist");
        assert_eq!(intermediate_expansion.radix.spelling, mode_names[4]);
        assert_eq!(
            intermediate_expansion.radix.head,
            payload.asserted_type.head
        );

        let output = super::source_local_mode_long_chain_two_hop_asserted_head_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact seven-expansion asserted-head source should reach checker");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 224 checker payload invariants should hold");
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert_eq!(output.subject_result_input.spelling, mode_names[6]);
        assert_eq!(output.asserted_type_input.spelling, mode_names[4]);
        assert_ne!(
            output.subject_result_input.head,
            output.asserted_type_input.head
        );
        assert!(matches!(
            output.asserted_type_input.head,
            TypeHeadInput::Symbol(_)
        ));
        assert_ne!(
            output.subject_result_input.site,
            output.asserted_type_input.site
        );
        assert_ne!(
            output.subject_result_input.source_range,
            output.asserted_type_input.source_range
        );
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[0]))
            .map(|(_, expansion)| &expansion.radix)
            .expect("BaseMode terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("one normalized builtin-set type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
        let (_, term) = output.term_formula.terms().iter().next().unwrap();
        assert_eq!(term.status, TermStatus::Inferred);
        assert!(term.deferred.is_empty());
        let (_, formula) = output.term_formula.formulas().iter().next().unwrap();
        assert_eq!(formula.kind, FormulaKind::TypeAssertion);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert!(formula.expected_types.is_empty());
        assert!(formula.candidate_set.is_none());
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
        assert!(output.term_formula.candidate_sets().is_empty());
        assert!(output.term_formula.facts().is_empty());
        assert!(output.term_formula.diagnostics().is_empty());

        let assert_invalid_output = |invalid| {
            let invalid_result =
                assert_source_reserved_variable_type_assertion_output(&invalid).map(|()| invalid);
            assert_eq!(
                source_reserved_variable_type_assertion_result_detail_keys(
                    invalid_result,
                    super::TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_TWO_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
                ),
                vec![
                    super::TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_TWO_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY
                        .to_owned()
                ]
            );
        };
        for removed in mode_names {
            let mut invalid = super::source_local_mode_long_chain_two_hop_asserted_head_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .unwrap();
            invalid
                .payload
                .reserve
                .mode_expansions
                .retain(|symbol, _| source_mode_symbol_spelling(symbol) != Some(removed));
            assert_invalid_output(invalid);
        }
        let exact_output = || {
            super::source_local_mode_long_chain_two_hop_asserted_head_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .unwrap()
        };
        let mut corruptions = Vec::new();
        let mut invalid = exact_output();
        invalid.subject_binding = BindingId::new(1);
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.payload.subject_lookup_ordinal = 2;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.spelling = mode_names[5].to_owned();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.spelling = "OtherLongMode".to_owned();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.head = invalid.subject_result_input.head.clone();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.head = TypeHeadInput::BuiltinObject;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.head = TypeHeadInput::BuiltinObject;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.source_range = invalid.asserted_type_input.source_range;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.site = invalid.asserted_type_input.site.clone();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.source_range = invalid.subject_result_input.source_range;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.site = invalid.subject_result_input.site.clone();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        let binding_head = invalid.subject_result_input.head.clone();
        let (_, binding_expansion) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[6]))
            .unwrap();
        binding_expansion.radix.head = binding_head;
        corruptions.push(invalid);
        for mode_name in &mode_names[1..] {
            let mut invalid = exact_output();
            let (_, expansion) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(*mode_name))
                .unwrap();
            expansion.radix.spelling = "object".to_owned();
            expansion.radix.head = TypeHeadInput::BuiltinObject;
            corruptions.push(invalid);
        }
        let mut invalid = exact_output();
        let (_, base_expansion) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[0]))
            .unwrap();
        base_expansion.radix.source_range = range(source_id, 0, 1);
        corruptions.push(invalid);
        let mut invalid = exact_output();
        let (_, base_expansion) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[0]))
            .unwrap();
        base_expansion.radix.spelling = "object".to_owned();
        base_expansion.radix.head = TypeHeadInput::BuiltinObject;
        corruptions.push(invalid);
        for invalid in corruptions {
            assert_invalid_output(invalid);
        }
        assert_source_reserved_variable_type_assertion_output(&exact_output())
            .expect("corrupting cloned Task 224 outputs must not mutate the exact output");

        let assert_extraction_gap = |ast, context: &str| {
            assert_eq!(
                source_type_elaboration_detail_keys(&ast, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()],
                "{context}"
            );
        };
        fn next_permutation(order: &mut [usize; 7]) -> bool {
            let Some(pivot) = (0..order.len() - 1)
                .rev()
                .find(|&index| order[index] < order[index + 1])
            else {
                return false;
            };
            let successor = (pivot + 1..order.len())
                .rev()
                .find(|&index| order[pivot] < order[index])
                .expect("a lexicographic successor must exist after the pivot");
            order.swap(pivot, successor);
            order[pivot + 1..].reverse();
            true
        }
        let ordered = exact_modes();
        let public_dispatch_orders = [
            [1, 0, 2, 3, 4, 5, 6],
            [0, 2, 1, 3, 4, 5, 6],
            [0, 1, 3, 2, 4, 5, 6],
            [0, 1, 2, 4, 3, 5, 6],
            [0, 1, 2, 3, 5, 4, 6],
            [0, 1, 2, 3, 4, 6, 5],
            [1, 2, 3, 4, 5, 6, 0],
            [6, 5, 4, 3, 2, 1, 0],
        ];
        let mut order = [0, 1, 2, 3, 4, 5, 6];
        let mut permutation_count = 0;
        let mut public_dispatch_representative_count = 0;
        while next_permutation(&mut order) {
            permutation_count += 1;
            let permuted = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                vec![
                    ordered[order[0]],
                    ordered[order[1]],
                    ordered[order[2]],
                    ordered[order[3]],
                    ordered[order[4]],
                    ordered[order[5]],
                    ordered[order[6]],
                ],
                reserve(),
                theorem,
            );
            assert!(
                super::extract_source_local_mode_long_chain_two_hop_asserted_head(
                    &permuted,
                    module.clone(),
                    &symbols,
                )
                .is_none(),
                "definition permutation {order:?} must not reach Task 224"
            );
            if public_dispatch_orders.contains(&order) {
                public_dispatch_representative_count += 1;
                assert_extraction_gap(
                    permuted,
                    &format!("public definition permutation {order:?}"),
                );
            }
        }
        assert_eq!(permutation_count, 5_039);
        assert_eq!(public_dispatch_representative_count, 8);
        let mut mode_near_misses = vec![
            Vec::<ModeDefinitionSpec>::new(),
            {
                let mut modes = exact_modes();
                modes.push(modes[0]);
                modes
            },
            {
                let mut modes = exact_modes();
                modes.push(mode_definition_with_label(
                    "UnusedLongMode",
                    "UnusedLongModeDef",
                    ReserveTypeShape::QualifiedSymbol(mode_names[6]),
                ));
                modes
            },
        ];
        for index in 0..mode_names.len() {
            let mut missing = exact_modes();
            missing.remove(index);
            mode_near_misses.push(missing);
            let mut duplicate = exact_modes();
            duplicate.push(duplicate[index]);
            mode_near_misses.push(duplicate);
            let mut malformed = exact_modes();
            malformed[index].rhs_shape = if index == 0 {
                ReserveTypeShape::Builtin("object")
            } else {
                ReserveTypeShape::QualifiedSymbol(mode_names[(index + 1) % mode_names.len()])
            };
            mode_near_misses.push(malformed);
            let mut wrong_label = exact_modes();
            wrong_label[index].label = Some("WrongLongModeDef");
            mode_near_misses.push(wrong_label);
            let mut wrong_spelling = exact_modes();
            wrong_spelling[index].pattern = "OtherLongMode";
            mode_near_misses.push(wrong_spelling);
            let mut recovered = exact_modes();
            recovered[index].recovered = true;
            mode_near_misses.push(recovered);
            let mut contextual = exact_modes();
            contextual[index].local_context = true;
            mode_near_misses.push(contextual);
            let mut parameterized = exact_modes();
            parameterized[index].parameterized_pattern = true;
            mode_near_misses.push(parameterized);
            let mut args = exact_modes();
            args[index].rhs_shape = if index == 0 {
                ReserveTypeShape::QualifiedSymbolWithArgs("set")
            } else {
                ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[index - 1])
            };
            mode_near_misses.push(args);
            let mut attrs = exact_modes();
            attrs[index].rhs_shape = if index == 0 {
                ReserveTypeShape::AttributedSet
            } else {
                ReserveTypeShape::AttributedQualifiedSymbol(mode_names[index - 1])
            };
            mode_near_misses.push(attrs);
        }
        for (index, modes) in mode_near_misses.into_iter().enumerate() {
            assert_extraction_gap(
                mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    modes,
                    reserve(),
                    theorem,
                ),
                &format!("mode near miss {index}"),
            );
        }

        let mut source_near_misses = Vec::new();
        let mut deeper_modes = exact_modes();
        deeper_modes.push(mode_definition_with_label(
            "ChainMode7",
            "ChainMode7Def",
            ReserveTypeShape::QualifiedSymbol(mode_names[6]),
        ));
        source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            deeper_modes,
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("ChainMode7"),
            )],
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[6]),
                ..theorem
            },
        ));
        for bad_reserve in [
            ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[6]),
            ReserveTypeShape::AttributedQualifiedSymbol(mode_names[6]),
            ReserveTypeShape::Builtin("set"),
            ReserveTypeShape::Builtin("object"),
        ] {
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(vec!["x"], bad_reserve)],
                theorem,
            ));
        }
        source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            exact_modes(),
            vec![reserve_item(
                vec!["y"],
                ReserveTypeShape::QualifiedSymbol(mode_names[6]),
            )],
            theorem,
        ));
        source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            exact_modes(),
            vec![
                reserve_item(vec!["x"], ReserveTypeShape::QualifiedSymbol(mode_names[6])),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
            ],
            theorem,
        ));
        for bad_reserve in [
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol(mode_names[5]),
            )],
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("OtherLongMode"),
            )],
            vec![reserve_item(
                vec!["x", "y"],
                ReserveTypeShape::QualifiedSymbol(mode_names[6]),
            )],
        ] {
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                bad_reserve,
                theorem,
            ));
        }
        let theorem_near_misses = [
            IdentifierTypeAssertionTheoremSpec {
                label: "OtherPayloadBoundary",
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                subject: "y",
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::Builtin("set"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::Builtin("object"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[0]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[1]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[2]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[3]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[5]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[6]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol("OtherLongMode"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[4]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::AttributedQualifiedSymbol(mode_names[4]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                negated: true,
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                status: Some("open"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                recovered_label: true,
                ..theorem
            },
        ];
        for near_miss_theorem in theorem_near_misses {
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                near_miss_theorem,
            ));
        }
        source_near_misses.push(
            modes_then_empty_definition_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                theorem,
            ),
        );
        for (index, near_miss) in source_near_misses.into_iter().enumerate() {
            assert_extraction_gap(near_miss, &format!("source near miss {index}"));
        }

        for imported_index in 0..mode_names.len() {
            let local_modes = mode_names
                .iter()
                .enumerate()
                .filter(|(index, _)| *index != imported_index)
                .map(|(_, spelling)| (*spelling, SymbolKind::Mode))
                .collect::<Vec<_>>();
            for imported in [
                vec![(mode_names[imported_index], SymbolKind::Mode)],
                vec![
                    (mode_names[imported_index], SymbolKind::Mode),
                    (mode_names[imported_index], SymbolKind::Mode),
                ],
            ] {
                let imported_symbols = source_local_and_imported_symbols_env(
                    module.clone(),
                    &local_modes,
                    imported.as_slice(),
                );
                assert_eq!(
                    source_type_elaboration_detail_keys(&exact, module.clone(), &imported_symbols),
                    vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
                );
            }
        }
        let unrelated_import = source_local_and_imported_symbols_env(
            module.clone(),
            &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
            &[("UnrelatedLongMode", SymbolKind::Mode)],
        );
        assert!(
            super::extract_source_local_mode_long_chain_two_hop_asserted_head(
                &exact,
                module.clone(),
                &unrelated_import,
            )
            .is_some()
        );
        for (asserted_spelling, asserted_head_symbols) in [
            (
                "ImportedLongMode",
                source_local_and_imported_symbols_env(
                    module.clone(),
                    &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
                    &[("ImportedLongMode", SymbolKind::Mode)],
                ),
            ),
            (
                "AmbiguousLongMode",
                source_local_and_imported_symbols_env(
                    module.clone(),
                    &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
                    &[
                        ("AmbiguousLongMode", SymbolKind::Mode),
                        ("AmbiguousLongMode", SymbolKind::Mode),
                    ],
                ),
            ),
        ] {
            let near_miss = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(asserted_spelling),
                    ..theorem
                },
            );
            assert_eq!(
                source_type_elaboration_detail_keys(
                    &near_miss,
                    module.clone(),
                    &asserted_head_symbols,
                ),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
    }

    #[test]
    fn source_local_object_mode_long_chain_two_hop_asserted_head_consumes_seven_expansions() {
        let source_id = source_id(225);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("local_object_mode_long_chain_two_hop_asserted_head"),
        );
        let mode_names = [
            "BaseObjectMode",
            "ChainObjectMode1",
            "ChainObjectMode2",
            "ChainObjectMode3",
            "ChainObjectMode4",
            "ChainObjectMode5",
            "ChainObjectMode6",
        ];
        let mode_labels = [
            "BaseObjectModeDef",
            "ChainObjectMode1Def",
            "ChainObjectMode2Def",
            "ChainObjectMode3Def",
            "ChainObjectMode4Def",
            "ChainObjectMode5Def",
            "ChainObjectMode6Def",
        ];
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                (mode_names[0], SymbolKind::Mode),
                (mode_names[1], SymbolKind::Mode),
                (mode_names[2], SymbolKind::Mode),
                (mode_names[3], SymbolKind::Mode),
                (mode_names[4], SymbolKind::Mode),
                (mode_names[5], SymbolKind::Mode),
                (mode_names[6], SymbolKind::Mode),
                ("ChainObjectMode7", SymbolKind::Mode),
                ("OtherLongObjectMode", SymbolKind::Mode),
            ],
        );
        let theorem = IdentifierTypeAssertionTheoremSpec {
            status: None,
            label: "LongLocalObjectModeTwoHopAssertedHeadPayloadBoundary",
            subject: "x",
            asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[4]),
            negated: false,
            recovered_label: false,
        };
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    mode_names[0],
                    mode_labels[0],
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition_with_label(
                    mode_names[1],
                    mode_labels[1],
                    ReserveTypeShape::QualifiedSymbol(mode_names[0]),
                ),
                mode_definition_with_label(
                    mode_names[2],
                    mode_labels[2],
                    ReserveTypeShape::QualifiedSymbol(mode_names[1]),
                ),
                mode_definition_with_label(
                    mode_names[3],
                    mode_labels[3],
                    ReserveTypeShape::QualifiedSymbol(mode_names[2]),
                ),
                mode_definition_with_label(
                    mode_names[4],
                    mode_labels[4],
                    ReserveTypeShape::QualifiedSymbol(mode_names[3]),
                ),
                mode_definition_with_label(
                    mode_names[5],
                    mode_labels[5],
                    ReserveTypeShape::QualifiedSymbol(mode_names[4]),
                ),
                mode_definition_with_label(
                    mode_names[6],
                    mode_labels[6],
                    ReserveTypeShape::QualifiedSymbol(mode_names[5]),
                ),
            ]
        };
        let reserve = || {
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol(mode_names[6]),
            )]
        };
        let exact = mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            exact_modes(),
            reserve(),
            theorem,
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        assert!(
            extract_source_local_object_mode_long_chain_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 179 must reject the Task 225 two-hop source"
        );
        assert!(
            super::extract_source_local_object_mode_long_chain_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 200 must reject the Task 225 two-hop source"
        );
        assert!(
            super::extract_source_local_object_mode_long_chain_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 210 must reject the Task 225 two-hop source"
        );
        assert!(
            super::extract_source_local_mode_long_chain_two_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 224 must reject the Task 225 object-terminal two-hop source"
        );
        let preexisting_owner_results = [
            extract_source_reserved_variable_type_assertion(&exact, module.clone(), &symbols),
            super::extract_source_reserved_object_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_local_object_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_chained_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_chained_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_chained_local_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_chained_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_chained_local_mode_radix_asserted_head(&exact, module.clone(), &symbols),
            extract_source_chained_local_object_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_two_edge_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_object_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_three_edge_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_object_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_four_edge_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_four_edge_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            super::extract_source_four_edge_local_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            super::extract_source_four_edge_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_four_edge_local_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_four_edge_local_object_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_mode_long_chain_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_object_mode_long_chain_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            super::extract_source_local_mode_long_chain_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            super::extract_source_local_object_mode_long_chain_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
        ];
        assert_eq!(preexisting_owner_results.len(), 34);
        assert!(
            preexisting_owner_results
                .into_iter()
                .all(|result| result.is_none()),
            "all 34 legacy type-assertion owner routes must reject Task 225"
        );
        let payload = super::extract_source_local_object_mode_long_chain_two_hop_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact seven-expansion two-hop source should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 7);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(payload.subject_lookup_ordinal, 1);
        let source_binding = &payload.reserve.bridge.bindings()[0];
        assert_eq!(source_binding.type_spelling, mode_names[6]);
        assert_eq!(payload.asserted_type.spelling, mode_names[4]);
        assert_ne!(payload.asserted_type.head, source_binding.type_head);
        assert!(matches!(source_binding.type_head, TypeHeadInput::Symbol(_)));
        assert_ne!(payload.asserted_type_site, payload.subject_site);
        assert_ne!(payload.asserted_type.range, source_binding.type_range);
        let binding_expansion = payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[6]))
            .map(|(_, expansion)| expansion)
            .expect("ChainObjectMode6 binding expansion should exist");
        assert_eq!(binding_expansion.radix.spelling, mode_names[5]);
        assert_ne!(binding_expansion.radix.head, payload.asserted_type.head);
        let TypeHeadInput::Symbol(intermediate_symbol) = &binding_expansion.radix.head else {
            panic!("ChainObjectMode5 relation intermediate should resolve to a symbol");
        };
        let intermediate_expansion = payload
            .reserve
            .mode_expansions
            .get(intermediate_symbol)
            .expect("ChainObjectMode5 intermediate expansion should exist");
        assert_eq!(intermediate_expansion.radix.spelling, mode_names[4]);
        assert_eq!(
            intermediate_expansion.radix.head,
            payload.asserted_type.head
        );

        let output = super::source_local_object_mode_long_chain_two_hop_asserted_head_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact seven-expansion asserted-head source should reach checker");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 225 checker payload invariants should hold");
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert_eq!(output.subject_result_input.spelling, mode_names[6]);
        assert_eq!(output.asserted_type_input.spelling, mode_names[4]);
        assert_ne!(
            output.subject_result_input.head,
            output.asserted_type_input.head
        );
        assert!(matches!(
            output.asserted_type_input.head,
            TypeHeadInput::Symbol(_)
        ));
        assert_ne!(
            output.subject_result_input.site,
            output.asserted_type_input.site
        );
        assert_ne!(
            output.subject_result_input.source_range,
            output.asserted_type_input.source_range
        );
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[0]))
            .map(|(_, expansion)| &expansion.radix)
            .expect("BaseObjectMode terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("one normalized builtin-object type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
        let (_, term) = output.term_formula.terms().iter().next().unwrap();
        assert_eq!(term.status, TermStatus::Inferred);
        assert!(term.deferred.is_empty());
        let (_, formula) = output.term_formula.formulas().iter().next().unwrap();
        assert_eq!(formula.kind, FormulaKind::TypeAssertion);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert!(formula.expected_types.is_empty());
        assert!(formula.candidate_set.is_none());
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
        assert!(output.term_formula.candidate_sets().is_empty());
        assert!(output.term_formula.facts().is_empty());
        assert!(output.term_formula.diagnostics().is_empty());

        let assert_invalid_output = |invalid| {
            let invalid_result =
                assert_source_reserved_variable_type_assertion_output(&invalid).map(|()| invalid);
            assert_eq!(
                source_reserved_variable_type_assertion_result_detail_keys(
                    invalid_result,
                    super::TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_TWO_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
                ),
                vec![
                    super::TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_TWO_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY
                        .to_owned()
                ]
            );
        };
        for removed in mode_names {
            let mut invalid =
                super::source_local_object_mode_long_chain_two_hop_asserted_head_output(
                    &exact,
                    module.clone(),
                    &symbols,
                )
                .unwrap();
            invalid
                .payload
                .reserve
                .mode_expansions
                .retain(|symbol, _| source_mode_symbol_spelling(symbol) != Some(removed));
            assert_invalid_output(invalid);
        }
        let exact_output = || {
            super::source_local_object_mode_long_chain_two_hop_asserted_head_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .unwrap()
        };
        let mut corruptions = Vec::new();
        let mut invalid = exact_output();
        invalid.subject_binding = BindingId::new(1);
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.payload.subject_lookup_ordinal = 2;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.spelling = mode_names[5].to_owned();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.spelling = "OtherLongObjectMode".to_owned();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.head = invalid.subject_result_input.head.clone();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.head = TypeHeadInput::BuiltinSet;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.head = TypeHeadInput::BuiltinSet;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.source_range = invalid.asserted_type_input.source_range;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.site = invalid.asserted_type_input.site.clone();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.source_range = invalid.subject_result_input.source_range;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.site = invalid.subject_result_input.site.clone();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        let binding_head = invalid.subject_result_input.head.clone();
        let (_, binding_expansion) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[6]))
            .unwrap();
        binding_expansion.radix.head = binding_head;
        corruptions.push(invalid);
        for mode_name in &mode_names[1..] {
            let mut invalid = exact_output();
            let (_, expansion) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(*mode_name))
                .unwrap();
            expansion.radix.spelling = "object".to_owned();
            expansion.radix.head = TypeHeadInput::BuiltinSet;
            corruptions.push(invalid);
        }
        let mut invalid = exact_output();
        let (_, base_expansion) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[0]))
            .unwrap();
        base_expansion.radix.source_range = range(source_id, 0, 1);
        corruptions.push(invalid);
        let mut invalid = exact_output();
        let (_, base_expansion) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[0]))
            .unwrap();
        base_expansion.radix.spelling = "object".to_owned();
        base_expansion.radix.head = TypeHeadInput::BuiltinSet;
        corruptions.push(invalid);
        for invalid in corruptions {
            assert_invalid_output(invalid);
        }
        assert_source_reserved_variable_type_assertion_output(&exact_output())
            .expect("corrupting cloned Task 225 outputs must not mutate the exact output");

        let assert_extraction_gap = |ast, context: &str| {
            assert_eq!(
                source_type_elaboration_detail_keys(&ast, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()],
                "{context}"
            );
        };
        fn next_permutation(order: &mut [usize; 7]) -> bool {
            let Some(pivot) = (0..order.len() - 1)
                .rev()
                .find(|&index| order[index] < order[index + 1])
            else {
                return false;
            };
            let successor = (pivot + 1..order.len())
                .rev()
                .find(|&index| order[pivot] < order[index])
                .expect("a lexicographic successor must exist after the pivot");
            order.swap(pivot, successor);
            order[pivot + 1..].reverse();
            true
        }
        let ordered = exact_modes();
        let public_dispatch_orders = [
            [1, 0, 2, 3, 4, 5, 6],
            [0, 2, 1, 3, 4, 5, 6],
            [0, 1, 3, 2, 4, 5, 6],
            [0, 1, 2, 4, 3, 5, 6],
            [0, 1, 2, 3, 5, 4, 6],
            [0, 1, 2, 3, 4, 6, 5],
            [1, 2, 3, 4, 5, 6, 0],
            [6, 5, 4, 3, 2, 1, 0],
        ];
        let mut order = [0, 1, 2, 3, 4, 5, 6];
        let mut permutation_count = 0;
        let mut public_dispatch_representative_count = 0;
        while next_permutation(&mut order) {
            permutation_count += 1;
            let permuted = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                vec![
                    ordered[order[0]],
                    ordered[order[1]],
                    ordered[order[2]],
                    ordered[order[3]],
                    ordered[order[4]],
                    ordered[order[5]],
                    ordered[order[6]],
                ],
                reserve(),
                theorem,
            );
            assert!(
                super::extract_source_local_object_mode_long_chain_two_hop_asserted_head(
                    &permuted,
                    module.clone(),
                    &symbols,
                )
                .is_none(),
                "definition permutation {order:?} must not reach Task 225"
            );
            if public_dispatch_orders.contains(&order) {
                public_dispatch_representative_count += 1;
                assert_extraction_gap(
                    permuted,
                    &format!("public definition permutation {order:?}"),
                );
            }
        }
        assert_eq!(permutation_count, 5_039);
        assert_eq!(public_dispatch_representative_count, 8);
        let mut mode_near_misses = vec![
            Vec::<ModeDefinitionSpec>::new(),
            {
                let mut modes = exact_modes();
                modes.push(modes[0]);
                modes
            },
            {
                let mut modes = exact_modes();
                modes.push(mode_definition_with_label(
                    "UnusedLongObjectMode",
                    "UnusedLongObjectModeDef",
                    ReserveTypeShape::QualifiedSymbol(mode_names[6]),
                ));
                modes
            },
        ];
        for index in 0..mode_names.len() {
            let mut missing = exact_modes();
            missing.remove(index);
            mode_near_misses.push(missing);
            let mut duplicate = exact_modes();
            duplicate.push(duplicate[index]);
            mode_near_misses.push(duplicate);
            let mut malformed = exact_modes();
            malformed[index].rhs_shape = if index == 0 {
                ReserveTypeShape::Builtin("set")
            } else {
                ReserveTypeShape::QualifiedSymbol(mode_names[(index + 1) % mode_names.len()])
            };
            mode_near_misses.push(malformed);
            let mut wrong_label = exact_modes();
            wrong_label[index].label = Some("WrongLongObjectModeDef");
            mode_near_misses.push(wrong_label);
            let mut wrong_spelling = exact_modes();
            wrong_spelling[index].pattern = "OtherLongObjectMode";
            mode_near_misses.push(wrong_spelling);
            let mut recovered = exact_modes();
            recovered[index].recovered = true;
            mode_near_misses.push(recovered);
            let mut contextual = exact_modes();
            contextual[index].local_context = true;
            mode_near_misses.push(contextual);
            let mut parameterized = exact_modes();
            parameterized[index].parameterized_pattern = true;
            mode_near_misses.push(parameterized);
            let mut args = exact_modes();
            args[index].rhs_shape = if index == 0 {
                ReserveTypeShape::QualifiedSymbolWithArgs("object")
            } else {
                ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[index - 1])
            };
            mode_near_misses.push(args);
            let mut attrs = exact_modes();
            attrs[index].rhs_shape = if index == 0 {
                ReserveTypeShape::AttributedObject
            } else {
                ReserveTypeShape::AttributedQualifiedSymbol(mode_names[index - 1])
            };
            mode_near_misses.push(attrs);
        }
        for (index, modes) in mode_near_misses.into_iter().enumerate() {
            assert_extraction_gap(
                mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    modes,
                    reserve(),
                    theorem,
                ),
                &format!("mode near miss {index}"),
            );
        }

        let mut source_near_misses = Vec::new();
        let mut deeper_modes = exact_modes();
        deeper_modes.push(mode_definition_with_label(
            "ChainObjectMode7",
            "ChainObjectMode7Def",
            ReserveTypeShape::QualifiedSymbol(mode_names[6]),
        ));
        source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            deeper_modes,
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("ChainObjectMode7"),
            )],
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[6]),
                ..theorem
            },
        ));
        for bad_reserve in [
            ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[6]),
            ReserveTypeShape::AttributedQualifiedSymbol(mode_names[6]),
            ReserveTypeShape::Builtin("object"),
            ReserveTypeShape::Builtin("set"),
        ] {
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(vec!["x"], bad_reserve)],
                theorem,
            ));
        }
        source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            exact_modes(),
            vec![reserve_item(
                vec!["y"],
                ReserveTypeShape::QualifiedSymbol(mode_names[6]),
            )],
            theorem,
        ));
        source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            exact_modes(),
            vec![
                reserve_item(vec!["x"], ReserveTypeShape::QualifiedSymbol(mode_names[6])),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("object")),
            ],
            theorem,
        ));
        for bad_reserve in [
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol(mode_names[5]),
            )],
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("OtherLongObjectMode"),
            )],
            vec![reserve_item(
                vec!["x", "y"],
                ReserveTypeShape::QualifiedSymbol(mode_names[6]),
            )],
        ] {
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                bad_reserve,
                theorem,
            ));
        }
        let theorem_near_misses = [
            IdentifierTypeAssertionTheoremSpec {
                label: "OtherPayloadBoundary",
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                subject: "y",
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::Builtin("object"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::Builtin("set"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[0]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[1]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[2]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[3]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[5]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[6]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol("OtherLongObjectMode"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[4]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::AttributedQualifiedSymbol(mode_names[4]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                negated: true,
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                status: Some("open"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                recovered_label: true,
                ..theorem
            },
        ];
        for near_miss_theorem in theorem_near_misses {
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                near_miss_theorem,
            ));
        }
        source_near_misses.push(
            modes_then_empty_definition_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                theorem,
            ),
        );
        for (index, near_miss) in source_near_misses.into_iter().enumerate() {
            assert_extraction_gap(near_miss, &format!("source near miss {index}"));
        }

        for imported_index in 0..mode_names.len() {
            let local_modes = mode_names
                .iter()
                .enumerate()
                .filter(|(index, _)| *index != imported_index)
                .map(|(_, spelling)| (*spelling, SymbolKind::Mode))
                .collect::<Vec<_>>();
            for imported in [
                vec![(mode_names[imported_index], SymbolKind::Mode)],
                vec![
                    (mode_names[imported_index], SymbolKind::Mode),
                    (mode_names[imported_index], SymbolKind::Mode),
                ],
            ] {
                let imported_symbols = source_local_and_imported_symbols_env(
                    module.clone(),
                    &local_modes,
                    imported.as_slice(),
                );
                assert_eq!(
                    source_type_elaboration_detail_keys(&exact, module.clone(), &imported_symbols),
                    vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
                );
            }
        }
        let unrelated_import = source_local_and_imported_symbols_env(
            module.clone(),
            &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
            &[("UnrelatedLongObjectMode", SymbolKind::Mode)],
        );
        assert!(
            super::extract_source_local_object_mode_long_chain_two_hop_asserted_head(
                &exact,
                module.clone(),
                &unrelated_import,
            )
            .is_some()
        );
        for (asserted_spelling, asserted_head_symbols) in [
            (
                "ImportedLongObjectMode",
                source_local_and_imported_symbols_env(
                    module.clone(),
                    &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
                    &[("ImportedLongObjectMode", SymbolKind::Mode)],
                ),
            ),
            (
                "AmbiguousLongObjectMode",
                source_local_and_imported_symbols_env(
                    module.clone(),
                    &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
                    &[
                        ("AmbiguousLongObjectMode", SymbolKind::Mode),
                        ("AmbiguousLongObjectMode", SymbolKind::Mode),
                    ],
                ),
            ),
        ] {
            let near_miss = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(asserted_spelling),
                    ..theorem
                },
            );
            assert_eq!(
                source_type_elaboration_detail_keys(
                    &near_miss,
                    module.clone(),
                    &asserted_head_symbols,
                ),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
    }

    #[test]
    fn source_local_object_mode_long_chain_radix_asserted_head_consumes_seven_expansions() {
        let source_id = source_id(210);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("local_object_mode_long_chain_radix_asserted_head"),
        );
        let mode_names = [
            "BaseObjectMode",
            "ChainObjectMode1",
            "ChainObjectMode2",
            "ChainObjectMode3",
            "ChainObjectMode4",
            "ChainObjectMode5",
            "ChainObjectMode6",
        ];
        let mode_labels = [
            "BaseObjectModeDef",
            "ChainObjectMode1Def",
            "ChainObjectMode2Def",
            "ChainObjectMode3Def",
            "ChainObjectMode4Def",
            "ChainObjectMode5Def",
            "ChainObjectMode6Def",
        ];
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                (mode_names[0], SymbolKind::Mode),
                (mode_names[1], SymbolKind::Mode),
                (mode_names[2], SymbolKind::Mode),
                (mode_names[3], SymbolKind::Mode),
                (mode_names[4], SymbolKind::Mode),
                (mode_names[5], SymbolKind::Mode),
                (mode_names[6], SymbolKind::Mode),
                ("ChainObjectMode7", SymbolKind::Mode),
                ("OtherLongObjectMode", SymbolKind::Mode),
            ],
        );
        let theorem = IdentifierTypeAssertionTheoremSpec {
            status: None,
            label: "LongLocalObjectModeRadixAssertedHeadPayloadBoundary",
            subject: "x",
            asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[5]),
            negated: false,
            recovered_label: false,
        };
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    mode_names[0],
                    mode_labels[0],
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition_with_label(
                    mode_names[1],
                    mode_labels[1],
                    ReserveTypeShape::QualifiedSymbol(mode_names[0]),
                ),
                mode_definition_with_label(
                    mode_names[2],
                    mode_labels[2],
                    ReserveTypeShape::QualifiedSymbol(mode_names[1]),
                ),
                mode_definition_with_label(
                    mode_names[3],
                    mode_labels[3],
                    ReserveTypeShape::QualifiedSymbol(mode_names[2]),
                ),
                mode_definition_with_label(
                    mode_names[4],
                    mode_labels[4],
                    ReserveTypeShape::QualifiedSymbol(mode_names[3]),
                ),
                mode_definition_with_label(
                    mode_names[5],
                    mode_labels[5],
                    ReserveTypeShape::QualifiedSymbol(mode_names[4]),
                ),
                mode_definition_with_label(
                    mode_names[6],
                    mode_labels[6],
                    ReserveTypeShape::QualifiedSymbol(mode_names[5]),
                ),
            ]
        };
        let reserve = || {
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol(mode_names[6]),
            )]
        };
        let exact = mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            exact_modes(),
            reserve(),
            theorem,
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        assert!(
            extract_source_local_object_mode_long_chain_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 179 must reject the Task 210 object immediate-radix source"
        );
        assert!(
            super::extract_source_local_object_mode_long_chain_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 200 must reject the Task 210 object immediate-radix source"
        );
        let preexisting_owner_results = [
            extract_source_reserved_variable_type_assertion(&exact, module.clone(), &symbols),
            super::extract_source_reserved_object_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_local_object_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_chained_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_chained_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_chained_local_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_chained_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_chained_local_mode_radix_asserted_head(&exact, module.clone(), &symbols),
            extract_source_chained_local_object_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_two_edge_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_object_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_three_edge_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_object_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_four_edge_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_four_edge_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            super::extract_source_four_edge_local_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            super::extract_source_four_edge_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_four_edge_local_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_four_edge_local_object_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_mode_long_chain_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_object_mode_long_chain_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            super::extract_source_local_mode_long_chain_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            super::extract_source_local_object_mode_long_chain_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_local_mode_long_chain_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
        ];
        assert_eq!(preexisting_owner_results.len(), 35);
        assert!(
            preexisting_owner_results
                .into_iter()
                .all(|result| result.is_none()),
            "all 35 pre-existing type-assertion owner routes must reject Task 210"
        );
        let payload = super::extract_source_local_object_mode_long_chain_radix_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact seven-expansion immediate-radix source should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 7);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(payload.subject_lookup_ordinal, 1);
        let source_binding = &payload.reserve.bridge.bindings()[0];
        assert_eq!(source_binding.type_spelling, mode_names[6]);
        assert_eq!(payload.asserted_type.spelling, mode_names[5]);
        assert_ne!(payload.asserted_type.head, source_binding.type_head);
        assert!(matches!(source_binding.type_head, TypeHeadInput::Symbol(_)));
        assert_ne!(payload.asserted_type_site, payload.subject_site);
        assert_ne!(payload.asserted_type.range, source_binding.type_range);
        let binding_expansion = payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[6]))
            .map(|(_, expansion)| expansion)
            .expect("ChainObjectMode6 binding expansion should exist");
        assert_eq!(binding_expansion.radix.spelling, mode_names[5]);
        assert_eq!(binding_expansion.radix.head, payload.asserted_type.head);

        let output = super::source_local_object_mode_long_chain_radix_asserted_head_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact seven-expansion asserted-head source should reach checker");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 210 checker payload invariants should hold");
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert_eq!(output.subject_result_input.spelling, mode_names[6]);
        assert_eq!(output.asserted_type_input.spelling, mode_names[5]);
        assert_ne!(
            output.subject_result_input.head,
            output.asserted_type_input.head
        );
        assert!(matches!(
            output.asserted_type_input.head,
            TypeHeadInput::Symbol(_)
        ));
        assert_ne!(
            output.subject_result_input.site,
            output.asserted_type_input.site
        );
        assert_ne!(
            output.subject_result_input.source_range,
            output.asserted_type_input.source_range
        );
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[0]))
            .map(|(_, expansion)| &expansion.radix)
            .expect("BaseObjectMode terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("one normalized builtin-object type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
        let (_, term) = output.term_formula.terms().iter().next().unwrap();
        assert_eq!(term.status, TermStatus::Inferred);
        assert!(term.deferred.is_empty());
        let (_, formula) = output.term_formula.formulas().iter().next().unwrap();
        assert_eq!(formula.kind, FormulaKind::TypeAssertion);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert!(formula.expected_types.is_empty());
        assert!(formula.candidate_set.is_none());
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
        assert!(output.term_formula.candidate_sets().is_empty());
        assert!(output.term_formula.facts().is_empty());
        assert!(output.term_formula.diagnostics().is_empty());

        let assert_invalid_output = |invalid| {
            let invalid_result =
                assert_source_reserved_variable_type_assertion_output(&invalid).map(|()| invalid);
            assert_eq!(
                source_reserved_variable_type_assertion_result_detail_keys(
                    invalid_result,
                    super::TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_RADIX_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
                ),
                vec![
                    super::TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_RADIX_ASSERTED_HEAD_INVALID_PAYLOAD_KEY
                        .to_owned()
                ]
            );
        };
        for removed in mode_names {
            let mut invalid =
                super::source_local_object_mode_long_chain_radix_asserted_head_output(
                    &exact,
                    module.clone(),
                    &symbols,
                )
                .unwrap();
            invalid
                .payload
                .reserve
                .mode_expansions
                .retain(|symbol, _| source_mode_symbol_spelling(symbol) != Some(removed));
            assert_invalid_output(invalid);
        }
        let exact_output = || {
            super::source_local_object_mode_long_chain_radix_asserted_head_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .unwrap()
        };
        let mut corruptions = Vec::new();
        let mut invalid = exact_output();
        invalid.subject_binding = BindingId::new(1);
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.payload.subject_lookup_ordinal = 2;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.spelling = mode_names[5].to_owned();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.spelling = "OtherLongObjectMode".to_owned();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.head = invalid.subject_result_input.head.clone();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.head = TypeHeadInput::BuiltinSet;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.head = TypeHeadInput::BuiltinSet;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.source_range = invalid.asserted_type_input.source_range;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.site = invalid.asserted_type_input.site.clone();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.source_range = invalid.subject_result_input.source_range;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.site = invalid.subject_result_input.site.clone();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        let binding_head = invalid.subject_result_input.head.clone();
        let (_, binding_expansion) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[6]))
            .unwrap();
        binding_expansion.radix.head = binding_head;
        corruptions.push(invalid);
        for mode_name in &mode_names[1..] {
            let mut invalid = exact_output();
            let (_, expansion) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(*mode_name))
                .unwrap();
            expansion.radix.spelling = "set".to_owned();
            expansion.radix.head = TypeHeadInput::BuiltinSet;
            corruptions.push(invalid);
        }
        let mut invalid = exact_output();
        let (_, base_expansion) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[0]))
            .unwrap();
        base_expansion.radix.source_range = range(source_id, 0, 1);
        corruptions.push(invalid);
        let mut invalid = exact_output();
        let (_, base_expansion) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[0]))
            .unwrap();
        base_expansion.radix.spelling = "set".to_owned();
        base_expansion.radix.head = TypeHeadInput::BuiltinSet;
        corruptions.push(invalid);
        for invalid in corruptions {
            assert_invalid_output(invalid);
        }
        assert_source_reserved_variable_type_assertion_output(&exact_output())
            .expect("corrupting cloned Task 210 outputs must not mutate the exact output");

        let assert_extraction_gap = |ast, context: &str| {
            assert_eq!(
                source_type_elaboration_detail_keys(&ast, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()],
                "{context}"
            );
        };
        fn next_permutation(order: &mut [usize; 7]) -> bool {
            let Some(pivot) = (0..order.len() - 1)
                .rev()
                .find(|&index| order[index] < order[index + 1])
            else {
                return false;
            };
            let successor = (pivot + 1..order.len())
                .rev()
                .find(|&index| order[pivot] < order[index])
                .expect("a lexicographic successor must exist after the pivot");
            order.swap(pivot, successor);
            order[pivot + 1..].reverse();
            true
        }
        let ordered = exact_modes();
        let public_dispatch_orders = [
            [1, 0, 2, 3, 4, 5, 6],
            [0, 2, 1, 3, 4, 5, 6],
            [0, 1, 3, 2, 4, 5, 6],
            [0, 1, 2, 4, 3, 5, 6],
            [0, 1, 2, 3, 5, 4, 6],
            [0, 1, 2, 3, 4, 6, 5],
            [1, 2, 3, 4, 5, 6, 0],
            [6, 5, 4, 3, 2, 1, 0],
        ];
        let mut order = [0, 1, 2, 3, 4, 5, 6];
        let mut permutation_count = 0;
        let mut public_dispatch_representative_count = 0;
        while next_permutation(&mut order) {
            permutation_count += 1;
            let permuted = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                vec![
                    ordered[order[0]],
                    ordered[order[1]],
                    ordered[order[2]],
                    ordered[order[3]],
                    ordered[order[4]],
                    ordered[order[5]],
                    ordered[order[6]],
                ],
                reserve(),
                theorem,
            );
            assert!(
                super::extract_source_local_object_mode_long_chain_radix_asserted_head(
                    &permuted,
                    module.clone(),
                    &symbols,
                )
                .is_none(),
                "definition permutation {order:?} must not reach Task 210"
            );
            if public_dispatch_orders.contains(&order) {
                public_dispatch_representative_count += 1;
                assert_extraction_gap(
                    permuted,
                    &format!("public definition permutation {order:?}"),
                );
            }
        }
        assert_eq!(permutation_count, 5_039);
        assert_eq!(public_dispatch_representative_count, 8);
        let mut mode_near_misses = vec![
            Vec::<ModeDefinitionSpec>::new(),
            {
                let mut modes = exact_modes();
                modes.push(modes[0]);
                modes
            },
            {
                let mut modes = exact_modes();
                modes.push(mode_definition_with_label(
                    "UnusedLongObjectMode",
                    "UnusedLongObjectModeDef",
                    ReserveTypeShape::QualifiedSymbol(mode_names[6]),
                ));
                modes
            },
        ];
        for index in 0..mode_names.len() {
            let mut missing = exact_modes();
            missing.remove(index);
            mode_near_misses.push(missing);
            let mut duplicate = exact_modes();
            duplicate.push(duplicate[index]);
            mode_near_misses.push(duplicate);
            let mut malformed = exact_modes();
            malformed[index].rhs_shape = if index == 0 {
                ReserveTypeShape::Builtin("set")
            } else {
                ReserveTypeShape::QualifiedSymbol(mode_names[(index + 1) % mode_names.len()])
            };
            mode_near_misses.push(malformed);
            let mut wrong_label = exact_modes();
            wrong_label[index].label = Some("WrongLongObjectModeDef");
            mode_near_misses.push(wrong_label);
            let mut wrong_spelling = exact_modes();
            wrong_spelling[index].pattern = "OtherLongObjectMode";
            mode_near_misses.push(wrong_spelling);
            let mut recovered = exact_modes();
            recovered[index].recovered = true;
            mode_near_misses.push(recovered);
            let mut contextual = exact_modes();
            contextual[index].local_context = true;
            mode_near_misses.push(contextual);
            let mut parameterized = exact_modes();
            parameterized[index].parameterized_pattern = true;
            mode_near_misses.push(parameterized);
            let mut args = exact_modes();
            args[index].rhs_shape = if index == 0 {
                ReserveTypeShape::QualifiedSymbolWithArgs("object")
            } else {
                ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[index - 1])
            };
            mode_near_misses.push(args);
            let mut attrs = exact_modes();
            attrs[index].rhs_shape = if index == 0 {
                ReserveTypeShape::AttributedObject
            } else {
                ReserveTypeShape::AttributedQualifiedSymbol(mode_names[index - 1])
            };
            mode_near_misses.push(attrs);
        }
        for (index, modes) in mode_near_misses.into_iter().enumerate() {
            assert_extraction_gap(
                mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    modes,
                    reserve(),
                    theorem,
                ),
                &format!("mode near miss {index}"),
            );
        }

        let mut source_near_misses = Vec::new();
        let mut deeper_modes = exact_modes();
        deeper_modes.push(mode_definition_with_label(
            "ChainObjectMode7",
            "ChainObjectMode7Def",
            ReserveTypeShape::QualifiedSymbol(mode_names[6]),
        ));
        source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            deeper_modes,
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("ChainObjectMode7"),
            )],
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[6]),
                ..theorem
            },
        ));
        for bad_reserve in [
            ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[6]),
            ReserveTypeShape::AttributedQualifiedSymbol(mode_names[6]),
            ReserveTypeShape::Builtin("object"),
            ReserveTypeShape::Builtin("set"),
        ] {
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(vec!["x"], bad_reserve)],
                theorem,
            ));
        }
        source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            exact_modes(),
            vec![reserve_item(
                vec!["y"],
                ReserveTypeShape::QualifiedSymbol(mode_names[6]),
            )],
            theorem,
        ));
        source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            exact_modes(),
            vec![
                reserve_item(vec!["x"], ReserveTypeShape::QualifiedSymbol(mode_names[6])),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("object")),
            ],
            theorem,
        ));
        for bad_reserve in [
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol(mode_names[5]),
            )],
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("OtherLongObjectMode"),
            )],
            vec![reserve_item(
                vec!["x", "y"],
                ReserveTypeShape::QualifiedSymbol(mode_names[6]),
            )],
        ] {
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                bad_reserve,
                theorem,
            ));
        }
        let theorem_near_misses = [
            IdentifierTypeAssertionTheoremSpec {
                label: "OtherPayloadBoundary",
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                subject: "y",
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::Builtin("object"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::Builtin("set"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[0]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[1]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[2]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[3]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[4]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[6]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol("OtherLongObjectMode"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[5]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::AttributedQualifiedSymbol(mode_names[5]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                negated: true,
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                status: Some("open"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                recovered_label: true,
                ..theorem
            },
        ];
        for near_miss_theorem in theorem_near_misses {
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                near_miss_theorem,
            ));
        }
        source_near_misses.push(
            modes_then_empty_definition_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                theorem,
            ),
        );
        for (index, near_miss) in source_near_misses.into_iter().enumerate() {
            assert_extraction_gap(near_miss, &format!("source near miss {index}"));
        }

        for imported_index in 0..mode_names.len() {
            let local_modes = mode_names
                .iter()
                .enumerate()
                .filter(|(index, _)| *index != imported_index)
                .map(|(_, spelling)| (*spelling, SymbolKind::Mode))
                .collect::<Vec<_>>();
            for imported in [
                vec![(mode_names[imported_index], SymbolKind::Mode)],
                vec![
                    (mode_names[imported_index], SymbolKind::Mode),
                    (mode_names[imported_index], SymbolKind::Mode),
                ],
            ] {
                let imported_symbols = source_local_and_imported_symbols_env(
                    module.clone(),
                    &local_modes,
                    imported.as_slice(),
                );
                assert_eq!(
                    source_type_elaboration_detail_keys(&exact, module.clone(), &imported_symbols),
                    vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
                );
            }
        }
        let unrelated_import = source_local_and_imported_symbols_env(
            module.clone(),
            &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
            &[("UnrelatedLongObjectMode", SymbolKind::Mode)],
        );
        assert!(
            super::extract_source_local_object_mode_long_chain_radix_asserted_head(
                &exact,
                module.clone(),
                &unrelated_import,
            )
            .is_some()
        );
        for (asserted_spelling, asserted_head_symbols) in [
            (
                "ImportedLongObjectMode",
                source_local_and_imported_symbols_env(
                    module.clone(),
                    &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
                    &[("ImportedLongObjectMode", SymbolKind::Mode)],
                ),
            ),
            (
                "AmbiguousLongObjectMode",
                source_local_and_imported_symbols_env(
                    module.clone(),
                    &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
                    &[
                        ("AmbiguousLongObjectMode", SymbolKind::Mode),
                        ("AmbiguousLongObjectMode", SymbolKind::Mode),
                    ],
                ),
            ),
        ] {
            let near_miss = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(asserted_spelling),
                    ..theorem
                },
            );
            assert_eq!(
                source_type_elaboration_detail_keys(
                    &near_miss,
                    module.clone(),
                    &asserted_head_symbols,
                ),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
    }

    #[test]
    fn source_local_mode_long_chain_asserted_head_consumes_seven_expansions() {
        let source_id = source_id(199);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("local_mode_long_chain_asserted_head"),
        );
        let mode_names = [
            "BaseMode",
            "ChainMode1",
            "ChainMode2",
            "ChainMode3",
            "ChainMode4",
            "ChainMode5",
            "ChainMode6",
        ];
        let mode_labels = [
            "BaseModeDef",
            "ChainMode1Def",
            "ChainMode2Def",
            "ChainMode3Def",
            "ChainMode4Def",
            "ChainMode5Def",
            "ChainMode6Def",
        ];
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                (mode_names[0], SymbolKind::Mode),
                (mode_names[1], SymbolKind::Mode),
                (mode_names[2], SymbolKind::Mode),
                (mode_names[3], SymbolKind::Mode),
                (mode_names[4], SymbolKind::Mode),
                (mode_names[5], SymbolKind::Mode),
                (mode_names[6], SymbolKind::Mode),
                ("ChainMode7", SymbolKind::Mode),
                ("OtherLongMode", SymbolKind::Mode),
            ],
        );
        let theorem = IdentifierTypeAssertionTheoremSpec {
            status: None,
            label: "LongLocalModeAssertedHeadPayloadBoundary",
            subject: "x",
            asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[6]),
            negated: false,
            recovered_label: false,
        };
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    mode_names[0],
                    mode_labels[0],
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition_with_label(
                    mode_names[1],
                    mode_labels[1],
                    ReserveTypeShape::QualifiedSymbol(mode_names[0]),
                ),
                mode_definition_with_label(
                    mode_names[2],
                    mode_labels[2],
                    ReserveTypeShape::QualifiedSymbol(mode_names[1]),
                ),
                mode_definition_with_label(
                    mode_names[3],
                    mode_labels[3],
                    ReserveTypeShape::QualifiedSymbol(mode_names[2]),
                ),
                mode_definition_with_label(
                    mode_names[4],
                    mode_labels[4],
                    ReserveTypeShape::QualifiedSymbol(mode_names[3]),
                ),
                mode_definition_with_label(
                    mode_names[5],
                    mode_labels[5],
                    ReserveTypeShape::QualifiedSymbol(mode_names[4]),
                ),
                mode_definition_with_label(
                    mode_names[6],
                    mode_labels[6],
                    ReserveTypeShape::QualifiedSymbol(mode_names[5]),
                ),
            ]
        };
        let reserve = || {
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol(mode_names[6]),
            )]
        };
        let exact = mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            exact_modes(),
            reserve(),
            theorem,
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        assert!(
            extract_source_local_mode_long_chain_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none()
        );
        assert!(
            super::extract_source_four_edge_local_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none()
        );
        assert!(
            super::extract_source_four_edge_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none()
        );
        let payload = super::extract_source_local_mode_long_chain_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact seven-expansion same-head source should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 7);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(payload.subject_lookup_ordinal, 1);
        let source_binding = &payload.reserve.bridge.bindings()[0];
        assert_eq!(source_binding.type_spelling, mode_names[6]);
        assert_eq!(payload.asserted_type.spelling, mode_names[6]);
        assert_eq!(payload.asserted_type.head, source_binding.type_head);
        assert!(matches!(source_binding.type_head, TypeHeadInput::Symbol(_)));
        assert_ne!(payload.asserted_type_site, payload.subject_site);
        assert_ne!(payload.asserted_type.range, source_binding.type_range);

        let output = super::source_local_mode_long_chain_asserted_head_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact seven-expansion asserted-head source should reach checker");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 199 checker payload invariants should hold");
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert_eq!(output.subject_result_input.spelling, mode_names[6]);
        assert_eq!(output.asserted_type_input.spelling, mode_names[6]);
        assert_eq!(
            output.subject_result_input.head,
            output.asserted_type_input.head
        );
        assert!(matches!(
            output.asserted_type_input.head,
            TypeHeadInput::Symbol(_)
        ));
        assert_ne!(
            output.subject_result_input.site,
            output.asserted_type_input.site
        );
        assert_ne!(
            output.subject_result_input.source_range,
            output.asserted_type_input.source_range
        );
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[0]))
            .map(|(_, expansion)| &expansion.radix)
            .expect("BaseMode terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("one normalized builtin-set type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
        let (_, term) = output.term_formula.terms().iter().next().unwrap();
        assert_eq!(term.status, TermStatus::Inferred);
        assert!(term.deferred.is_empty());
        let (_, formula) = output.term_formula.formulas().iter().next().unwrap();
        assert_eq!(formula.kind, FormulaKind::TypeAssertion);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert!(formula.expected_types.is_empty());
        assert!(formula.candidate_set.is_none());
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
        assert!(output.term_formula.candidate_sets().is_empty());
        assert!(output.term_formula.facts().is_empty());
        assert!(output.term_formula.diagnostics().is_empty());

        let assert_invalid_output = |invalid| {
            let invalid_result =
                assert_source_reserved_variable_type_assertion_output(&invalid).map(|()| invalid);
            assert_eq!(
                source_reserved_variable_type_assertion_result_detail_keys(
                    invalid_result,
                    super::TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
                ),
                vec![
                    super::TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_ASSERTED_HEAD_INVALID_PAYLOAD_KEY
                        .to_owned()
                ]
            );
        };
        for removed in mode_names {
            let mut invalid = super::source_local_mode_long_chain_asserted_head_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .unwrap();
            invalid
                .payload
                .reserve
                .mode_expansions
                .retain(|symbol, _| source_mode_symbol_spelling(symbol) != Some(removed));
            assert_invalid_output(invalid);
        }
        let exact_output = || {
            super::source_local_mode_long_chain_asserted_head_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .unwrap()
        };
        let mut corruptions = Vec::new();
        let mut invalid = exact_output();
        invalid.subject_binding = BindingId::new(1);
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.payload.subject_lookup_ordinal = 2;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.spelling = "OtherLongMode".to_owned();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.head = TypeHeadInput::BuiltinObject;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.head = TypeHeadInput::BuiltinObject;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.source_range = invalid.subject_result_input.source_range;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.site = invalid.subject_result_input.site.clone();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        let (_, base_expansion) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[0]))
            .unwrap();
        base_expansion.radix.source_range = range(source_id, 0, 1);
        corruptions.push(invalid);
        for invalid in corruptions {
            assert_invalid_output(invalid);
        }

        let assert_extraction_gap = |ast, context: &str| {
            assert_eq!(
                source_type_elaboration_detail_keys(&ast, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()],
                "{context}"
            );
        };
        let mut mode_near_misses = vec![
            Vec::<ModeDefinitionSpec>::new(),
            vec![
                exact_modes()[6],
                exact_modes()[5],
                exact_modes()[4],
                exact_modes()[3],
                exact_modes()[2],
                exact_modes()[1],
                exact_modes()[0],
            ],
            {
                let mut modes = exact_modes();
                modes.push(modes[0]);
                modes
            },
            {
                let mut modes = exact_modes();
                modes.push(mode_definition_with_label(
                    "UnusedLongMode",
                    "UnusedLongModeDef",
                    ReserveTypeShape::QualifiedSymbol(mode_names[6]),
                ));
                modes
            },
        ];
        for index in 0..mode_names.len() {
            let mut missing = exact_modes();
            missing.remove(index);
            mode_near_misses.push(missing);
            let mut malformed = exact_modes();
            malformed[index].rhs_shape = if index == 0 {
                ReserveTypeShape::Builtin("object")
            } else {
                ReserveTypeShape::QualifiedSymbol(mode_names[(index + 1) % mode_names.len()])
            };
            mode_near_misses.push(malformed);
            let mut wrong_label = exact_modes();
            wrong_label[index].label = Some("WrongLongModeDef");
            mode_near_misses.push(wrong_label);
            let mut recovered = exact_modes();
            recovered[index].recovered = true;
            mode_near_misses.push(recovered);
            let mut contextual = exact_modes();
            contextual[index].local_context = true;
            mode_near_misses.push(contextual);
            let mut parameterized = exact_modes();
            parameterized[index].parameterized_pattern = true;
            mode_near_misses.push(parameterized);
            let mut args = exact_modes();
            args[index].rhs_shape = if index == 0 {
                ReserveTypeShape::QualifiedSymbolWithArgs("set")
            } else {
                ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[index - 1])
            };
            mode_near_misses.push(args);
            let mut attrs = exact_modes();
            attrs[index].rhs_shape = if index == 0 {
                ReserveTypeShape::AttributedSet
            } else {
                ReserveTypeShape::AttributedQualifiedSymbol(mode_names[index - 1])
            };
            mode_near_misses.push(attrs);
        }
        for (index, modes) in mode_near_misses.into_iter().enumerate() {
            assert_extraction_gap(
                mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    modes,
                    reserve(),
                    theorem,
                ),
                &format!("mode near miss {index}"),
            );
        }

        let mut source_near_misses = Vec::new();
        let mut deeper_modes = exact_modes();
        deeper_modes.push(mode_definition_with_label(
            "ChainMode7",
            "ChainMode7Def",
            ReserveTypeShape::QualifiedSymbol(mode_names[6]),
        ));
        source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            deeper_modes,
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("ChainMode7"),
            )],
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol("ChainMode7"),
                ..theorem
            },
        ));
        for bad_reserve in [
            ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[6]),
            ReserveTypeShape::AttributedQualifiedSymbol(mode_names[6]),
            ReserveTypeShape::Builtin("set"),
        ] {
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(vec!["x"], bad_reserve)],
                theorem,
            ));
        }
        source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            exact_modes(),
            vec![
                reserve_item(vec!["x"], ReserveTypeShape::QualifiedSymbol(mode_names[6])),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
            ],
            theorem,
        ));
        let theorem_near_misses = [
            IdentifierTypeAssertionTheoremSpec {
                label: "OtherPayloadBoundary",
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                subject: "y",
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::Builtin("set"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[0]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[1]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[2]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[3]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[4]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[5]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol("OtherLongMode"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[6]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::AttributedQualifiedSymbol(mode_names[6]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                negated: true,
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                status: Some("open"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                recovered_label: true,
                ..theorem
            },
        ];
        for near_miss_theorem in theorem_near_misses {
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                near_miss_theorem,
            ));
        }
        source_near_misses.push(
            modes_then_empty_definition_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                theorem,
            ),
        );
        for (index, near_miss) in source_near_misses.into_iter().enumerate() {
            assert_extraction_gap(near_miss, &format!("source near miss {index}"));
        }

        for imported_index in 0..mode_names.len() {
            let local_modes = mode_names
                .iter()
                .enumerate()
                .filter(|(index, _)| *index != imported_index)
                .map(|(_, spelling)| (*spelling, SymbolKind::Mode))
                .collect::<Vec<_>>();
            let imported_symbols = source_local_and_imported_symbols_env(
                module.clone(),
                &local_modes,
                &[(mode_names[imported_index], SymbolKind::Mode)],
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&exact, module.clone(), &imported_symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
        let unrelated_import = source_local_and_imported_symbols_env(
            module.clone(),
            &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
            &[("UnrelatedLongMode", SymbolKind::Mode)],
        );
        assert!(
            super::extract_source_local_mode_long_chain_asserted_head(
                &exact,
                module.clone(),
                &unrelated_import,
            )
            .is_some()
        );
        for (asserted_spelling, asserted_head_symbols) in [
            (
                "ImportedLongMode",
                source_local_and_imported_symbols_env(
                    module.clone(),
                    &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
                    &[("ImportedLongMode", SymbolKind::Mode)],
                ),
            ),
            (
                "AmbiguousLongMode",
                source_local_and_imported_symbols_env(
                    module.clone(),
                    &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
                    &[
                        ("AmbiguousLongMode", SymbolKind::Mode),
                        ("AmbiguousLongMode", SymbolKind::Mode),
                    ],
                ),
            ),
        ] {
            let near_miss = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(asserted_spelling),
                    ..theorem
                },
            );
            assert_eq!(
                source_type_elaboration_detail_keys(
                    &near_miss,
                    module.clone(),
                    &asserted_head_symbols,
                ),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
    }

    #[test]
    fn source_local_object_mode_long_chain_asserted_head_consumes_seven_expansions() {
        let source_id = source_id(200);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("local_object_mode_long_chain_asserted_head"),
        );
        let mode_names = [
            "BaseObjectMode",
            "ChainObjectMode1",
            "ChainObjectMode2",
            "ChainObjectMode3",
            "ChainObjectMode4",
            "ChainObjectMode5",
            "ChainObjectMode6",
        ];
        let mode_labels = [
            "BaseObjectModeDef",
            "ChainObjectMode1Def",
            "ChainObjectMode2Def",
            "ChainObjectMode3Def",
            "ChainObjectMode4Def",
            "ChainObjectMode5Def",
            "ChainObjectMode6Def",
        ];
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                (mode_names[0], SymbolKind::Mode),
                (mode_names[1], SymbolKind::Mode),
                (mode_names[2], SymbolKind::Mode),
                (mode_names[3], SymbolKind::Mode),
                (mode_names[4], SymbolKind::Mode),
                (mode_names[5], SymbolKind::Mode),
                (mode_names[6], SymbolKind::Mode),
                ("ChainObjectMode7", SymbolKind::Mode),
                ("OtherLongObjectMode", SymbolKind::Mode),
            ],
        );
        let theorem = IdentifierTypeAssertionTheoremSpec {
            status: None,
            label: "LongLocalObjectModeAssertedHeadPayloadBoundary",
            subject: "x",
            asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[6]),
            negated: false,
            recovered_label: false,
        };
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    mode_names[0],
                    mode_labels[0],
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition_with_label(
                    mode_names[1],
                    mode_labels[1],
                    ReserveTypeShape::QualifiedSymbol(mode_names[0]),
                ),
                mode_definition_with_label(
                    mode_names[2],
                    mode_labels[2],
                    ReserveTypeShape::QualifiedSymbol(mode_names[1]),
                ),
                mode_definition_with_label(
                    mode_names[3],
                    mode_labels[3],
                    ReserveTypeShape::QualifiedSymbol(mode_names[2]),
                ),
                mode_definition_with_label(
                    mode_names[4],
                    mode_labels[4],
                    ReserveTypeShape::QualifiedSymbol(mode_names[3]),
                ),
                mode_definition_with_label(
                    mode_names[5],
                    mode_labels[5],
                    ReserveTypeShape::QualifiedSymbol(mode_names[4]),
                ),
                mode_definition_with_label(
                    mode_names[6],
                    mode_labels[6],
                    ReserveTypeShape::QualifiedSymbol(mode_names[5]),
                ),
            ]
        };
        let reserve = || {
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol(mode_names[6]),
            )]
        };
        let exact = mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            exact_modes(),
            reserve(),
            theorem,
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        assert!(
            extract_source_local_object_mode_long_chain_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none()
        );
        assert!(
            super::extract_source_local_mode_long_chain_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none()
        );
        assert!(
            super::extract_source_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none()
        );
        assert!(
            super::extract_source_chained_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none()
        );
        assert!(
            super::extract_source_two_edge_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none()
        );
        assert!(
            super::extract_source_three_edge_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none()
        );
        assert!(
            super::extract_source_four_edge_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none()
        );
        let payload = super::extract_source_local_object_mode_long_chain_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact seven-expansion object same-head source should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 7);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(payload.subject_lookup_ordinal, 1);
        let source_binding = &payload.reserve.bridge.bindings()[0];
        assert_eq!(source_binding.type_spelling, mode_names[6]);
        assert_eq!(payload.asserted_type.spelling, mode_names[6]);
        assert_eq!(payload.asserted_type.head, source_binding.type_head);
        assert!(matches!(source_binding.type_head, TypeHeadInput::Symbol(_)));
        assert_ne!(payload.asserted_type_site, payload.subject_site);
        assert_ne!(payload.asserted_type.range, source_binding.type_range);

        let output = super::source_local_object_mode_long_chain_asserted_head_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact seven-expansion object asserted-head source should reach checker");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 200 checker payload invariants should hold");
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert_eq!(output.subject_result_input.spelling, mode_names[6]);
        assert_eq!(output.asserted_type_input.spelling, mode_names[6]);
        assert_eq!(
            output.subject_result_input.head,
            output.asserted_type_input.head
        );
        assert!(matches!(
            output.asserted_type_input.head,
            TypeHeadInput::Symbol(_)
        ));
        assert_ne!(
            output.subject_result_input.site,
            output.asserted_type_input.site
        );
        assert_ne!(
            output.subject_result_input.source_range,
            output.asserted_type_input.source_range
        );
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[0]))
            .map(|(_, expansion)| &expansion.radix)
            .expect("BaseObjectMode terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("one normalized builtin-object type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
        let (_, term) = output.term_formula.terms().iter().next().unwrap();
        assert_eq!(term.status, TermStatus::Inferred);
        assert!(term.deferred.is_empty());
        let (_, formula) = output.term_formula.formulas().iter().next().unwrap();
        assert_eq!(formula.kind, FormulaKind::TypeAssertion);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert!(formula.expected_types.is_empty());
        assert!(formula.candidate_set.is_none());
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
        assert!(output.term_formula.candidate_sets().is_empty());
        assert!(output.term_formula.facts().is_empty());
        assert!(output.term_formula.diagnostics().is_empty());

        let assert_invalid_output = |invalid| {
            let invalid_result =
                assert_source_reserved_variable_type_assertion_output(&invalid).map(|()| invalid);
            assert_eq!(
                source_reserved_variable_type_assertion_result_detail_keys(
                    invalid_result,
                    super::TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
                ),
                vec![
                    super::TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_ASSERTED_HEAD_INVALID_PAYLOAD_KEY
                        .to_owned()
                ]
            );
        };
        for removed in mode_names {
            let mut invalid = super::source_local_object_mode_long_chain_asserted_head_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .unwrap();
            invalid
                .payload
                .reserve
                .mode_expansions
                .retain(|symbol, _| source_mode_symbol_spelling(symbol) != Some(removed));
            assert_invalid_output(invalid);
        }
        let exact_output = || {
            super::source_local_object_mode_long_chain_asserted_head_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .unwrap()
        };
        let mut corruptions = Vec::new();
        let mut invalid = exact_output();
        invalid.subject_binding = BindingId::new(1);
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.payload.subject_lookup_ordinal = 2;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.spelling = "OtherLongObjectMode".to_owned();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.head = TypeHeadInput::BuiltinSet;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.head = TypeHeadInput::BuiltinSet;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.source_range = invalid.subject_result_input.source_range;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.site = invalid.subject_result_input.site.clone();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        let (_, base_expansion) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[0]))
            .unwrap();
        base_expansion.radix.source_range = range(source_id, 0, 1);
        corruptions.push(invalid);
        for invalid in corruptions {
            assert_invalid_output(invalid);
        }
        assert_source_reserved_variable_type_assertion_output(&exact_output())
            .expect("corrupting cloned outputs must not mutate the exact output");

        let assert_extraction_gap = |ast, context: &str| {
            assert_eq!(
                source_type_elaboration_detail_keys(&ast, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()],
                "{context}"
            );
        };
        let mut mode_near_misses = vec![
            Vec::<ModeDefinitionSpec>::new(),
            vec![
                exact_modes()[6],
                exact_modes()[5],
                exact_modes()[4],
                exact_modes()[3],
                exact_modes()[2],
                exact_modes()[1],
                exact_modes()[0],
            ],
            {
                let mut modes = exact_modes();
                modes.push(modes[0]);
                modes
            },
            {
                let mut modes = exact_modes();
                modes.push(mode_definition_with_label(
                    "UnusedLongObjectMode",
                    "UnusedLongObjectModeDef",
                    ReserveTypeShape::QualifiedSymbol(mode_names[6]),
                ));
                modes
            },
        ];
        for index in 0..mode_names.len() {
            let mut missing = exact_modes();
            missing.remove(index);
            mode_near_misses.push(missing);
            let mut malformed = exact_modes();
            malformed[index].rhs_shape = if index == 0 {
                ReserveTypeShape::Builtin("set")
            } else {
                ReserveTypeShape::QualifiedSymbol(mode_names[(index + 1) % mode_names.len()])
            };
            mode_near_misses.push(malformed);
            let mut wrong_label = exact_modes();
            wrong_label[index].label = Some("WrongLongObjectModeDef");
            mode_near_misses.push(wrong_label);
            let mut recovered = exact_modes();
            recovered[index].recovered = true;
            mode_near_misses.push(recovered);
            let mut contextual = exact_modes();
            contextual[index].local_context = true;
            mode_near_misses.push(contextual);
            let mut parameterized = exact_modes();
            parameterized[index].parameterized_pattern = true;
            mode_near_misses.push(parameterized);
            let mut args = exact_modes();
            args[index].rhs_shape = if index == 0 {
                ReserveTypeShape::QualifiedSymbolWithArgs("object")
            } else {
                ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[index - 1])
            };
            mode_near_misses.push(args);
            let mut attrs = exact_modes();
            attrs[index].rhs_shape = if index == 0 {
                ReserveTypeShape::AttributedObject
            } else {
                ReserveTypeShape::AttributedQualifiedSymbol(mode_names[index - 1])
            };
            mode_near_misses.push(attrs);
        }
        for (index, modes) in mode_near_misses.into_iter().enumerate() {
            assert_extraction_gap(
                mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    modes,
                    reserve(),
                    theorem,
                ),
                &format!("object mode near miss {index}"),
            );
        }

        let mut source_near_misses = Vec::new();
        let mut deeper_modes = exact_modes();
        deeper_modes.push(mode_definition_with_label(
            "ChainObjectMode7",
            "ChainObjectMode7Def",
            ReserveTypeShape::QualifiedSymbol(mode_names[6]),
        ));
        source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            deeper_modes,
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("ChainObjectMode7"),
            )],
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol("ChainObjectMode7"),
                ..theorem
            },
        ));
        for bad_reserve in [
            ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[6]),
            ReserveTypeShape::AttributedQualifiedSymbol(mode_names[6]),
            ReserveTypeShape::Builtin("object"),
            ReserveTypeShape::Builtin("set"),
        ] {
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(vec!["x"], bad_reserve)],
                theorem,
            ));
        }
        source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            exact_modes(),
            vec![
                reserve_item(vec!["x"], ReserveTypeShape::QualifiedSymbol(mode_names[6])),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("object")),
            ],
            theorem,
        ));
        let theorem_near_misses = [
            IdentifierTypeAssertionTheoremSpec {
                label: "OtherPayloadBoundary",
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                subject: "y",
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::Builtin("object"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::Builtin("set"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[0]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[1]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[2]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[3]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[4]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[5]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol("OtherLongObjectMode"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[6]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::AttributedQualifiedSymbol(mode_names[6]),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                negated: true,
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                status: Some("open"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                recovered_label: true,
                ..theorem
            },
        ];
        for near_miss_theorem in theorem_near_misses {
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                near_miss_theorem,
            ));
        }
        source_near_misses.push(
            modes_then_empty_definition_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                theorem,
            ),
        );
        for (index, near_miss) in source_near_misses.into_iter().enumerate() {
            assert_extraction_gap(near_miss, &format!("object source near miss {index}"));
        }

        for imported_index in 0..mode_names.len() {
            let local_modes = mode_names
                .iter()
                .enumerate()
                .filter(|(index, _)| *index != imported_index)
                .map(|(_, spelling)| (*spelling, SymbolKind::Mode))
                .collect::<Vec<_>>();
            let imported_symbols = source_local_and_imported_symbols_env(
                module.clone(),
                &local_modes,
                &[(mode_names[imported_index], SymbolKind::Mode)],
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&exact, module.clone(), &imported_symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
        let unrelated_import = source_local_and_imported_symbols_env(
            module.clone(),
            &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
            &[("UnrelatedLongObjectMode", SymbolKind::Mode)],
        );
        assert!(
            super::extract_source_local_object_mode_long_chain_asserted_head(
                &exact,
                module.clone(),
                &unrelated_import,
            )
            .is_some()
        );
        for (asserted_spelling, asserted_head_symbols) in [
            (
                "ImportedLongObjectMode",
                source_local_and_imported_symbols_env(
                    module.clone(),
                    &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
                    &[("ImportedLongObjectMode", SymbolKind::Mode)],
                ),
            ),
            (
                "AmbiguousLongObjectMode",
                source_local_and_imported_symbols_env(
                    module.clone(),
                    &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
                    &[
                        ("AmbiguousLongObjectMode", SymbolKind::Mode),
                        ("AmbiguousLongObjectMode", SymbolKind::Mode),
                    ],
                ),
            ),
        ] {
            let near_miss = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(asserted_spelling),
                    ..theorem
                },
            );
            assert_eq!(
                source_type_elaboration_detail_keys(
                    &near_miss,
                    module.clone(),
                    &asserted_head_symbols,
                ),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
    }

    #[test]
    fn active_local_mode_long_chain_reserved_variable_type_assertion_fixture_consumes_seven_expansions()
     {
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("mizar-test crate should live below the workspace root")
            .to_path_buf();
        let config = DiscoveryConfig {
            workspace_root: workspace_root.clone(),
            tests_root: workspace_root.join("tests"),
            manifest_path: workspace_root.join("tests/coverage/spec_trace.toml"),
            profile: TestProfile::Fast,
            validation_mode: ValidationMode::Metadata,
        };
        let plan = build_test_plan(&config).expect("repository test plan should build");
        let (ordinal, case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| {
                case.id.0
                    == "pass_type_elaboration_local_mode_long_chain_reserved_variable_type_assertion_001"
            })
            .expect("Task 175 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 175 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 175 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_local_mode_long_chain_reserved_variable_type_assertion_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 175 real AST should reach long-chain type assertion");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 175 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.subject_result_input.spelling, "ChainMode6");
        assert!(matches!(
            output.subject_result_input.head,
            TypeHeadInput::Symbol(_)
        ));
        assert_eq!(output.asserted_type_input.spelling, "set");
        assert_eq!(output.asserted_type_input.head, TypeHeadInput::BuiltinSet);
        assert_ne!(
            output.subject_result_input.source_range,
            output.asserted_type_input.source_range
        );
        let (_, formula) = output
            .term_formula
            .formulas()
            .iter()
            .next()
            .expect("Task 175 type assertion should be checked");
        assert_eq!(formula.kind, FormulaKind::TypeAssertion);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("BaseMode"))
            .map(|(_, expansion)| &expansion.radix)
            .expect("Task 175 BaseMode terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 175 normalized set type should exist");
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
    }

    #[test]
    fn active_local_mode_long_chain_three_hop_asserted_head_fixture_consumes_seven_expansions() {
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("mizar-test crate should live below the workspace root")
            .to_path_buf();
        let config = DiscoveryConfig {
            workspace_root: workspace_root.clone(),
            tests_root: workspace_root.join("tests"),
            manifest_path: workspace_root.join("tests/coverage/spec_trace.toml"),
            profile: TestProfile::Fast,
            validation_mode: ValidationMode::Metadata,
        };
        let plan = build_test_plan(&config).expect("Task 226 repository plan should build");
        let (ordinal, case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| {
                case.id.0
                    == "pass_type_elaboration_local_mode_long_chain_three_hop_asserted_head_001"
            })
            .expect("Task 226 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 226 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 226 fixture should produce a real AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = super::source_local_mode_long_chain_three_hop_asserted_head_output(
            &ast,
            resolver.module.clone(),
            &symbols,
        )
        .expect("Task 226 real AST should reach the seven-expansion three-hop seam");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 226 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.subject_result_input.spelling, "ChainMode6");
        assert_eq!(output.asserted_type_input.spelling, "ChainMode3");
        assert_ne!(
            output.subject_result_input.head,
            output.asserted_type_input.head
        );
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert!(output.term_formula.candidate_sets().is_empty());
        assert!(output.term_formula.facts().is_empty());
        assert!(output.term_formula.diagnostics().is_empty());
        let chain6 = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("ChainMode6"))
            .map(|(_, expansion)| expansion)
            .expect("Task 226 ChainMode6 expansion should exist");
        assert_eq!(chain6.radix.spelling, "ChainMode5");
        let TypeHeadInput::Symbol(chain5_symbol) = &chain6.radix.head else {
            panic!("Task 226 ChainMode5 first intermediate should be a symbol");
        };
        let chain5 = output
            .payload
            .reserve
            .mode_expansions
            .get(chain5_symbol)
            .expect("Task 226 ChainMode5 expansion should exist");
        assert_eq!(chain5.radix.spelling, "ChainMode4");
        assert_ne!(chain5.radix.head, output.asserted_type_input.head);
        let TypeHeadInput::Symbol(chain4_symbol) = &chain5.radix.head else {
            panic!("Task 226 ChainMode4 second intermediate should be a symbol");
        };
        let chain4 = output
            .payload
            .reserve
            .mode_expansions
            .get(chain4_symbol)
            .expect("Task 226 ChainMode4 expansion should exist");
        assert_eq!(chain4.radix.spelling, "ChainMode3");
        assert_eq!(chain4.radix.head, output.asserted_type_input.head);

        type OwnerExtractor = fn(
            &SurfaceAst,
            ResolverModuleId,
            &SymbolEnv,
        ) -> Option<super::SourceReservedVariableTypeAssertion>;
        let prior_extractors: [(&str, OwnerExtractor); 50] = [
            (
                "extract_source_reserved_variable_type_assertion",
                super::extract_source_reserved_variable_type_assertion,
            ),
            (
                "extract_source_reserved_object_variable_type_assertion",
                super::extract_source_reserved_object_variable_type_assertion,
            ),
            (
                "extract_source_local_mode_reserved_variable_type_assertion",
                super::extract_source_local_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_local_object_mode_reserved_variable_type_assertion",
                super::extract_source_local_object_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_local_mode_asserted_head",
                super::extract_source_local_mode_asserted_head,
            ),
            (
                "extract_source_local_object_mode_asserted_head",
                super::extract_source_local_object_mode_asserted_head,
            ),
            (
                "extract_source_chained_local_mode_reserved_variable_type_assertion",
                super::extract_source_chained_local_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_chained_local_object_mode_reserved_variable_type_assertion",
                super::extract_source_chained_local_object_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_chained_local_mode_asserted_head",
                super::extract_source_chained_local_mode_asserted_head,
            ),
            (
                "extract_source_chained_local_object_mode_asserted_head",
                super::extract_source_chained_local_object_mode_asserted_head,
            ),
            (
                "extract_source_chained_local_mode_radix_asserted_head",
                super::extract_source_chained_local_mode_radix_asserted_head,
            ),
            (
                "extract_source_chained_local_object_mode_radix_asserted_head",
                super::extract_source_chained_local_object_mode_radix_asserted_head,
            ),
            (
                "extract_source_two_edge_local_mode_reserved_variable_type_assertion",
                super::extract_source_two_edge_local_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_two_edge_local_object_mode_reserved_variable_type_assertion",
                super::extract_source_two_edge_local_object_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_two_edge_local_mode_asserted_head",
                super::extract_source_two_edge_local_mode_asserted_head,
            ),
            (
                "extract_source_two_edge_local_object_mode_asserted_head",
                super::extract_source_two_edge_local_object_mode_asserted_head,
            ),
            (
                "extract_source_two_edge_local_mode_radix_asserted_head",
                super::extract_source_two_edge_local_mode_radix_asserted_head,
            ),
            (
                "extract_source_two_edge_local_object_mode_radix_asserted_head",
                super::extract_source_two_edge_local_object_mode_radix_asserted_head,
            ),
            (
                "extract_source_three_edge_local_mode_reserved_variable_type_assertion",
                super::extract_source_three_edge_local_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_three_edge_local_object_mode_reserved_variable_type_assertion",
                super::extract_source_three_edge_local_object_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_three_edge_local_mode_asserted_head",
                super::extract_source_three_edge_local_mode_asserted_head,
            ),
            (
                "extract_source_three_edge_local_object_mode_asserted_head",
                super::extract_source_three_edge_local_object_mode_asserted_head,
            ),
            (
                "extract_source_three_edge_local_mode_radix_asserted_head",
                super::extract_source_three_edge_local_mode_radix_asserted_head,
            ),
            (
                "extract_source_three_edge_local_object_mode_radix_asserted_head",
                super::extract_source_three_edge_local_object_mode_radix_asserted_head,
            ),
            (
                "extract_source_four_edge_local_mode_reserved_variable_type_assertion",
                super::extract_source_four_edge_local_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_four_edge_local_object_mode_reserved_variable_type_assertion",
                super::extract_source_four_edge_local_object_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_four_edge_local_mode_asserted_head",
                super::extract_source_four_edge_local_mode_asserted_head,
            ),
            (
                "extract_source_four_edge_local_object_mode_asserted_head",
                super::extract_source_four_edge_local_object_mode_asserted_head,
            ),
            (
                "extract_source_four_edge_local_mode_radix_asserted_head",
                super::extract_source_four_edge_local_mode_radix_asserted_head,
            ),
            (
                "extract_source_four_edge_local_object_mode_radix_asserted_head",
                super::extract_source_four_edge_local_object_mode_radix_asserted_head,
            ),
            (
                "extract_source_local_mode_long_chain_reserved_variable_type_assertion",
                super::extract_source_local_mode_long_chain_reserved_variable_type_assertion,
            ),
            (
                "extract_source_local_object_mode_long_chain_reserved_variable_type_assertion",
                super::extract_source_local_object_mode_long_chain_reserved_variable_type_assertion,
            ),
            (
                "extract_source_local_mode_long_chain_asserted_head",
                super::extract_source_local_mode_long_chain_asserted_head,
            ),
            (
                "extract_source_local_object_mode_long_chain_asserted_head",
                super::extract_source_local_object_mode_long_chain_asserted_head,
            ),
            (
                "extract_source_local_mode_long_chain_radix_asserted_head",
                super::extract_source_local_mode_long_chain_radix_asserted_head,
            ),
            (
                "extract_source_local_object_mode_long_chain_radix_asserted_head",
                super::extract_source_local_object_mode_long_chain_radix_asserted_head,
            ),
            (
                "extract_source_local_mode_long_chain_two_hop_asserted_head",
                super::extract_source_local_mode_long_chain_two_hop_asserted_head,
            ),
            (
                "extract_source_local_object_mode_long_chain_two_hop_asserted_head",
                super::extract_source_local_object_mode_long_chain_two_hop_asserted_head,
            ),
            (
                "extract_source_two_edge_local_mode_two_hop_asserted_head",
                super::extract_source_two_edge_local_mode_two_hop_asserted_head,
            ),
            (
                "extract_source_two_edge_local_object_mode_two_hop_asserted_head",
                super::extract_source_two_edge_local_object_mode_two_hop_asserted_head,
            ),
            (
                "extract_source_three_edge_local_mode_two_hop_asserted_head",
                super::extract_source_three_edge_local_mode_two_hop_asserted_head,
            ),
            (
                "extract_source_three_edge_local_object_mode_two_hop_asserted_head",
                super::extract_source_three_edge_local_object_mode_two_hop_asserted_head,
            ),
            (
                "extract_source_four_edge_local_mode_two_hop_asserted_head",
                super::extract_source_four_edge_local_mode_two_hop_asserted_head,
            ),
            (
                "extract_source_four_edge_local_object_mode_two_hop_asserted_head",
                super::extract_source_four_edge_local_object_mode_two_hop_asserted_head,
            ),
            (
                "extract_source_three_edge_local_mode_three_hop_asserted_head",
                super::extract_source_three_edge_local_mode_three_hop_asserted_head,
            ),
            (
                "extract_source_three_edge_local_object_mode_three_hop_asserted_head",
                super::extract_source_three_edge_local_object_mode_three_hop_asserted_head,
            ),
            (
                "extract_source_four_edge_local_mode_three_hop_asserted_head",
                super::extract_source_four_edge_local_mode_three_hop_asserted_head,
            ),
            (
                "extract_source_four_edge_local_object_mode_three_hop_asserted_head",
                super::extract_source_four_edge_local_object_mode_three_hop_asserted_head,
            ),
            (
                "extract_source_four_edge_local_mode_four_hop_asserted_head",
                super::extract_source_four_edge_local_mode_four_hop_asserted_head,
            ),
            (
                "extract_source_four_edge_local_object_mode_four_hop_asserted_head",
                super::extract_source_four_edge_local_object_mode_four_hop_asserted_head,
            ),
        ];
        for (owner, extractor) in prior_extractors {
            assert!(
                extractor(&ast, resolver.module.clone(), &symbols).is_none(),
                "Task 226 source must not be owned by {owner}"
            );
        }

        let prior_owner_ids = [
            "pass_type_elaboration_reserved_variable_type_assertion_001",
            "pass_type_elaboration_reserved_object_variable_type_assertion_001",
            "pass_type_elaboration_local_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_local_object_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_local_mode_asserted_head_001",
            "pass_type_elaboration_local_object_mode_asserted_head_001",
            "pass_type_elaboration_chained_local_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_chained_local_object_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_chained_local_mode_asserted_head_001",
            "pass_type_elaboration_chained_local_object_mode_asserted_head_001",
            "pass_type_elaboration_chained_local_mode_radix_asserted_head_001",
            "pass_type_elaboration_chained_local_object_mode_radix_asserted_head_001",
            "pass_type_elaboration_two_edge_local_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_two_edge_local_object_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_two_edge_local_mode_asserted_head_001",
            "pass_type_elaboration_two_edge_local_object_mode_asserted_head_001",
            "pass_type_elaboration_two_edge_local_mode_radix_asserted_head_001",
            "pass_type_elaboration_two_edge_local_object_mode_radix_asserted_head_001",
            "pass_type_elaboration_three_edge_local_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_three_edge_local_object_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_three_edge_local_mode_asserted_head_001",
            "pass_type_elaboration_three_edge_local_object_mode_asserted_head_001",
            "pass_type_elaboration_three_edge_local_mode_radix_asserted_head_001",
            "pass_type_elaboration_three_edge_local_object_mode_radix_asserted_head_001",
            "pass_type_elaboration_four_edge_local_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_four_edge_local_object_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_four_edge_local_mode_asserted_head_001",
            "pass_type_elaboration_four_edge_local_object_mode_asserted_head_001",
            "pass_type_elaboration_four_edge_local_mode_radix_asserted_head_001",
            "pass_type_elaboration_four_edge_local_object_mode_radix_asserted_head_001",
            "pass_type_elaboration_local_mode_long_chain_reserved_variable_type_assertion_001",
            "pass_type_elaboration_local_object_mode_long_chain_reserved_variable_type_assertion_001",
            "pass_type_elaboration_local_mode_long_chain_asserted_head_001",
            "pass_type_elaboration_local_object_mode_long_chain_asserted_head_001",
            "pass_type_elaboration_local_mode_long_chain_radix_asserted_head_001",
            "pass_type_elaboration_local_object_mode_long_chain_radix_asserted_head_001",
            "pass_type_elaboration_local_mode_long_chain_two_hop_asserted_head_001",
            "pass_type_elaboration_local_object_mode_long_chain_two_hop_asserted_head_001",
            "pass_type_elaboration_two_edge_local_mode_two_hop_asserted_head_001",
            "pass_type_elaboration_two_edge_local_object_mode_two_hop_asserted_head_001",
            "pass_type_elaboration_three_edge_local_mode_two_hop_asserted_head_001",
            "pass_type_elaboration_three_edge_local_object_mode_two_hop_asserted_head_001",
            "pass_type_elaboration_four_edge_local_mode_two_hop_asserted_head_001",
            "pass_type_elaboration_four_edge_local_object_mode_two_hop_asserted_head_001",
            "pass_type_elaboration_three_edge_local_mode_three_hop_asserted_head_001",
            "pass_type_elaboration_three_edge_local_object_mode_three_hop_asserted_head_001",
            "pass_type_elaboration_four_edge_local_mode_three_hop_asserted_head_001",
            "pass_type_elaboration_four_edge_local_object_mode_three_hop_asserted_head_001",
            "pass_type_elaboration_four_edge_local_mode_four_hop_asserted_head_001",
            "pass_type_elaboration_four_edge_local_object_mode_four_hop_asserted_head_001",
        ];
        assert_eq!(prior_owner_ids.len(), 50);
        for owner_id in prior_owner_ids {
            let (owner_ordinal, owner_case) = active_type_elaboration_cases(&plan)
                .enumerate()
                .find(|(_, case)| case.id.0 == owner_id)
                .unwrap_or_else(|| panic!("prior owner fixture {owner_id} must be active"));
            let owner_frontend = run_frontend(&workspace_root, owner_case, owner_ordinal)
                .unwrap_or_else(|error| panic!("owner fixture {owner_id} must parse: {error}"));
            assert!(owner_frontend.diagnostics.is_empty());
            let owner_ast = owner_frontend
                .ast
                .unwrap_or_else(|| panic!("owner fixture {owner_id} must produce an AST"));
            let owner_resolver =
                resolver_symbol_collection(&workspace_root, owner_case, &owner_ast);
            assert!(owner_resolver.detail_keys.is_empty());
            let owner_symbols = augment_type_elaboration_import_summaries(
                &owner_ast,
                &owner_resolver.module,
                owner_resolver.env,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                ),
                Vec::<String>::new(),
                "prior owner fixture {owner_id} must still reach its active route"
            );
            assert!(
                super::extract_source_local_mode_long_chain_three_hop_asserted_head(
                    &owner_ast,
                    owner_resolver.module,
                    &owner_symbols,
                )
                .is_none(),
                "Task 226 must reject prior owner fixture {owner_id}"
            );
        }
    }

    #[test]
    fn active_local_mode_long_chain_four_hop_asserted_head_fixture_consumes_seven_expansions() {
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("mizar-test crate should live below the workspace root")
            .to_path_buf();
        let config = DiscoveryConfig {
            workspace_root: workspace_root.clone(),
            tests_root: workspace_root.join("tests"),
            manifest_path: workspace_root.join("tests/coverage/spec_trace.toml"),
            profile: TestProfile::Fast,
            validation_mode: ValidationMode::Metadata,
        };
        let plan = build_test_plan(&config).expect("Task 228 repository plan should build");
        let (ordinal, case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| {
                case.id.0
                    == "pass_type_elaboration_local_mode_long_chain_four_hop_asserted_head_001"
            })
            .expect("Task 228 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 228 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 228 fixture should produce a real AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = super::source_local_mode_long_chain_four_hop_asserted_head_output(
            &ast,
            resolver.module.clone(),
            &symbols,
        )
        .expect("Task 228 real AST should reach the seven-expansion four-hop seam");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 228 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.subject_result_input.spelling, "ChainMode6");
        assert_eq!(output.asserted_type_input.spelling, "ChainMode2");
        assert_ne!(
            output.subject_result_input.head,
            output.asserted_type_input.head
        );
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert!(output.term_formula.candidate_sets().is_empty());
        assert!(output.term_formula.facts().is_empty());
        assert!(output.term_formula.diagnostics().is_empty());
        let chain6 = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("ChainMode6"))
            .map(|(_, expansion)| expansion)
            .expect("Task 228 ChainMode6 expansion should exist");
        assert_eq!(chain6.radix.spelling, "ChainMode5");
        let TypeHeadInput::Symbol(chain5_symbol) = &chain6.radix.head else {
            panic!("Task 228 ChainMode5 first intermediate should be a symbol");
        };
        let chain5 = output
            .payload
            .reserve
            .mode_expansions
            .get(chain5_symbol)
            .expect("Task 228 ChainMode5 expansion should exist");
        assert_eq!(chain5.radix.spelling, "ChainMode4");
        assert_ne!(chain5.radix.head, output.asserted_type_input.head);
        let TypeHeadInput::Symbol(chain4_symbol) = &chain5.radix.head else {
            panic!("Task 228 ChainMode4 second intermediate should be a symbol");
        };
        let chain4 = output
            .payload
            .reserve
            .mode_expansions
            .get(chain4_symbol)
            .expect("Task 228 ChainMode4 expansion should exist");
        assert_eq!(chain4.radix.spelling, "ChainMode3");
        assert_ne!(chain4.radix.head, output.asserted_type_input.head);
        let TypeHeadInput::Symbol(chain3_symbol) = &chain4.radix.head else {
            panic!("Task 228 ChainMode3 third intermediate should be a symbol");
        };
        let chain3 = output
            .payload
            .reserve
            .mode_expansions
            .get(chain3_symbol)
            .expect("Task 228 ChainMode3 expansion should exist");
        assert_eq!(chain3.radix.spelling, "ChainMode2");
        assert_eq!(chain3.radix.head, output.asserted_type_input.head);

        type OwnerExtractor = fn(
            &SurfaceAst,
            ResolverModuleId,
            &SymbolEnv,
        ) -> Option<super::SourceReservedVariableTypeAssertion>;
        let prior_extractors: [(&str, OwnerExtractor); 52] = [
            (
                "extract_source_reserved_variable_type_assertion",
                super::extract_source_reserved_variable_type_assertion,
            ),
            (
                "extract_source_reserved_object_variable_type_assertion",
                super::extract_source_reserved_object_variable_type_assertion,
            ),
            (
                "extract_source_local_mode_reserved_variable_type_assertion",
                super::extract_source_local_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_local_object_mode_reserved_variable_type_assertion",
                super::extract_source_local_object_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_local_mode_asserted_head",
                super::extract_source_local_mode_asserted_head,
            ),
            (
                "extract_source_local_object_mode_asserted_head",
                super::extract_source_local_object_mode_asserted_head,
            ),
            (
                "extract_source_chained_local_mode_reserved_variable_type_assertion",
                super::extract_source_chained_local_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_chained_local_object_mode_reserved_variable_type_assertion",
                super::extract_source_chained_local_object_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_chained_local_mode_asserted_head",
                super::extract_source_chained_local_mode_asserted_head,
            ),
            (
                "extract_source_chained_local_object_mode_asserted_head",
                super::extract_source_chained_local_object_mode_asserted_head,
            ),
            (
                "extract_source_chained_local_mode_radix_asserted_head",
                super::extract_source_chained_local_mode_radix_asserted_head,
            ),
            (
                "extract_source_chained_local_object_mode_radix_asserted_head",
                super::extract_source_chained_local_object_mode_radix_asserted_head,
            ),
            (
                "extract_source_two_edge_local_mode_reserved_variable_type_assertion",
                super::extract_source_two_edge_local_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_two_edge_local_object_mode_reserved_variable_type_assertion",
                super::extract_source_two_edge_local_object_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_two_edge_local_mode_asserted_head",
                super::extract_source_two_edge_local_mode_asserted_head,
            ),
            (
                "extract_source_two_edge_local_object_mode_asserted_head",
                super::extract_source_two_edge_local_object_mode_asserted_head,
            ),
            (
                "extract_source_two_edge_local_mode_radix_asserted_head",
                super::extract_source_two_edge_local_mode_radix_asserted_head,
            ),
            (
                "extract_source_two_edge_local_object_mode_radix_asserted_head",
                super::extract_source_two_edge_local_object_mode_radix_asserted_head,
            ),
            (
                "extract_source_three_edge_local_mode_reserved_variable_type_assertion",
                super::extract_source_three_edge_local_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_three_edge_local_object_mode_reserved_variable_type_assertion",
                super::extract_source_three_edge_local_object_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_three_edge_local_mode_asserted_head",
                super::extract_source_three_edge_local_mode_asserted_head,
            ),
            (
                "extract_source_three_edge_local_object_mode_asserted_head",
                super::extract_source_three_edge_local_object_mode_asserted_head,
            ),
            (
                "extract_source_three_edge_local_mode_radix_asserted_head",
                super::extract_source_three_edge_local_mode_radix_asserted_head,
            ),
            (
                "extract_source_three_edge_local_object_mode_radix_asserted_head",
                super::extract_source_three_edge_local_object_mode_radix_asserted_head,
            ),
            (
                "extract_source_four_edge_local_mode_reserved_variable_type_assertion",
                super::extract_source_four_edge_local_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_four_edge_local_object_mode_reserved_variable_type_assertion",
                super::extract_source_four_edge_local_object_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_four_edge_local_mode_asserted_head",
                super::extract_source_four_edge_local_mode_asserted_head,
            ),
            (
                "extract_source_four_edge_local_object_mode_asserted_head",
                super::extract_source_four_edge_local_object_mode_asserted_head,
            ),
            (
                "extract_source_four_edge_local_mode_radix_asserted_head",
                super::extract_source_four_edge_local_mode_radix_asserted_head,
            ),
            (
                "extract_source_four_edge_local_object_mode_radix_asserted_head",
                super::extract_source_four_edge_local_object_mode_radix_asserted_head,
            ),
            (
                "extract_source_local_mode_long_chain_reserved_variable_type_assertion",
                super::extract_source_local_mode_long_chain_reserved_variable_type_assertion,
            ),
            (
                "extract_source_local_object_mode_long_chain_reserved_variable_type_assertion",
                super::extract_source_local_object_mode_long_chain_reserved_variable_type_assertion,
            ),
            (
                "extract_source_local_mode_long_chain_asserted_head",
                super::extract_source_local_mode_long_chain_asserted_head,
            ),
            (
                "extract_source_local_object_mode_long_chain_asserted_head",
                super::extract_source_local_object_mode_long_chain_asserted_head,
            ),
            (
                "extract_source_local_mode_long_chain_radix_asserted_head",
                super::extract_source_local_mode_long_chain_radix_asserted_head,
            ),
            (
                "extract_source_local_object_mode_long_chain_radix_asserted_head",
                super::extract_source_local_object_mode_long_chain_radix_asserted_head,
            ),
            (
                "extract_source_local_mode_long_chain_two_hop_asserted_head",
                super::extract_source_local_mode_long_chain_two_hop_asserted_head,
            ),
            (
                "extract_source_local_object_mode_long_chain_two_hop_asserted_head",
                super::extract_source_local_object_mode_long_chain_two_hop_asserted_head,
            ),
            (
                "extract_source_two_edge_local_mode_two_hop_asserted_head",
                super::extract_source_two_edge_local_mode_two_hop_asserted_head,
            ),
            (
                "extract_source_two_edge_local_object_mode_two_hop_asserted_head",
                super::extract_source_two_edge_local_object_mode_two_hop_asserted_head,
            ),
            (
                "extract_source_three_edge_local_mode_two_hop_asserted_head",
                super::extract_source_three_edge_local_mode_two_hop_asserted_head,
            ),
            (
                "extract_source_three_edge_local_object_mode_two_hop_asserted_head",
                super::extract_source_three_edge_local_object_mode_two_hop_asserted_head,
            ),
            (
                "extract_source_four_edge_local_mode_two_hop_asserted_head",
                super::extract_source_four_edge_local_mode_two_hop_asserted_head,
            ),
            (
                "extract_source_four_edge_local_object_mode_two_hop_asserted_head",
                super::extract_source_four_edge_local_object_mode_two_hop_asserted_head,
            ),
            (
                "extract_source_three_edge_local_mode_three_hop_asserted_head",
                super::extract_source_three_edge_local_mode_three_hop_asserted_head,
            ),
            (
                "extract_source_three_edge_local_object_mode_three_hop_asserted_head",
                super::extract_source_three_edge_local_object_mode_three_hop_asserted_head,
            ),
            (
                "extract_source_four_edge_local_mode_three_hop_asserted_head",
                super::extract_source_four_edge_local_mode_three_hop_asserted_head,
            ),
            (
                "extract_source_four_edge_local_object_mode_three_hop_asserted_head",
                super::extract_source_four_edge_local_object_mode_three_hop_asserted_head,
            ),
            (
                "extract_source_four_edge_local_mode_four_hop_asserted_head",
                super::extract_source_four_edge_local_mode_four_hop_asserted_head,
            ),
            (
                "extract_source_four_edge_local_object_mode_four_hop_asserted_head",
                super::extract_source_four_edge_local_object_mode_four_hop_asserted_head,
            ),
            (
                "extract_source_local_mode_long_chain_three_hop_asserted_head",
                super::extract_source_local_mode_long_chain_three_hop_asserted_head,
            ),
            (
                "extract_source_local_object_mode_long_chain_three_hop_asserted_head",
                super::extract_source_local_object_mode_long_chain_three_hop_asserted_head,
            ),
        ];
        for (owner, extractor) in prior_extractors {
            assert!(
                extractor(&ast, resolver.module.clone(), &symbols).is_none(),
                "Task 228 source must not be owned by {owner}"
            );
        }

        let prior_owner_ids = [
            "pass_type_elaboration_reserved_variable_type_assertion_001",
            "pass_type_elaboration_reserved_object_variable_type_assertion_001",
            "pass_type_elaboration_local_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_local_object_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_local_mode_asserted_head_001",
            "pass_type_elaboration_local_object_mode_asserted_head_001",
            "pass_type_elaboration_chained_local_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_chained_local_object_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_chained_local_mode_asserted_head_001",
            "pass_type_elaboration_chained_local_object_mode_asserted_head_001",
            "pass_type_elaboration_chained_local_mode_radix_asserted_head_001",
            "pass_type_elaboration_chained_local_object_mode_radix_asserted_head_001",
            "pass_type_elaboration_two_edge_local_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_two_edge_local_object_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_two_edge_local_mode_asserted_head_001",
            "pass_type_elaboration_two_edge_local_object_mode_asserted_head_001",
            "pass_type_elaboration_two_edge_local_mode_radix_asserted_head_001",
            "pass_type_elaboration_two_edge_local_object_mode_radix_asserted_head_001",
            "pass_type_elaboration_three_edge_local_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_three_edge_local_object_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_three_edge_local_mode_asserted_head_001",
            "pass_type_elaboration_three_edge_local_object_mode_asserted_head_001",
            "pass_type_elaboration_three_edge_local_mode_radix_asserted_head_001",
            "pass_type_elaboration_three_edge_local_object_mode_radix_asserted_head_001",
            "pass_type_elaboration_four_edge_local_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_four_edge_local_object_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_four_edge_local_mode_asserted_head_001",
            "pass_type_elaboration_four_edge_local_object_mode_asserted_head_001",
            "pass_type_elaboration_four_edge_local_mode_radix_asserted_head_001",
            "pass_type_elaboration_four_edge_local_object_mode_radix_asserted_head_001",
            "pass_type_elaboration_local_mode_long_chain_reserved_variable_type_assertion_001",
            "pass_type_elaboration_local_object_mode_long_chain_reserved_variable_type_assertion_001",
            "pass_type_elaboration_local_mode_long_chain_asserted_head_001",
            "pass_type_elaboration_local_object_mode_long_chain_asserted_head_001",
            "pass_type_elaboration_local_mode_long_chain_radix_asserted_head_001",
            "pass_type_elaboration_local_object_mode_long_chain_radix_asserted_head_001",
            "pass_type_elaboration_local_mode_long_chain_two_hop_asserted_head_001",
            "pass_type_elaboration_local_object_mode_long_chain_two_hop_asserted_head_001",
            "pass_type_elaboration_two_edge_local_mode_two_hop_asserted_head_001",
            "pass_type_elaboration_two_edge_local_object_mode_two_hop_asserted_head_001",
            "pass_type_elaboration_three_edge_local_mode_two_hop_asserted_head_001",
            "pass_type_elaboration_three_edge_local_object_mode_two_hop_asserted_head_001",
            "pass_type_elaboration_four_edge_local_mode_two_hop_asserted_head_001",
            "pass_type_elaboration_four_edge_local_object_mode_two_hop_asserted_head_001",
            "pass_type_elaboration_three_edge_local_mode_three_hop_asserted_head_001",
            "pass_type_elaboration_three_edge_local_object_mode_three_hop_asserted_head_001",
            "pass_type_elaboration_four_edge_local_mode_three_hop_asserted_head_001",
            "pass_type_elaboration_four_edge_local_object_mode_three_hop_asserted_head_001",
            "pass_type_elaboration_four_edge_local_mode_four_hop_asserted_head_001",
            "pass_type_elaboration_four_edge_local_object_mode_four_hop_asserted_head_001",
            "pass_type_elaboration_local_mode_long_chain_three_hop_asserted_head_001",
            "pass_type_elaboration_local_object_mode_long_chain_three_hop_asserted_head_001",
        ];
        assert_eq!(prior_owner_ids.len(), 52);
        for owner_id in prior_owner_ids {
            let (owner_ordinal, owner_case) = active_type_elaboration_cases(&plan)
                .enumerate()
                .find(|(_, case)| case.id.0 == owner_id)
                .unwrap_or_else(|| panic!("prior owner fixture {owner_id} must be active"));
            let owner_frontend = run_frontend(&workspace_root, owner_case, owner_ordinal)
                .unwrap_or_else(|error| panic!("owner fixture {owner_id} must parse: {error}"));
            assert!(owner_frontend.diagnostics.is_empty());
            let owner_ast = owner_frontend
                .ast
                .unwrap_or_else(|| panic!("owner fixture {owner_id} must produce an AST"));
            let owner_resolver =
                resolver_symbol_collection(&workspace_root, owner_case, &owner_ast);
            assert!(owner_resolver.detail_keys.is_empty());
            let owner_symbols = augment_type_elaboration_import_summaries(
                &owner_ast,
                &owner_resolver.module,
                owner_resolver.env,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                ),
                Vec::<String>::new(),
                "prior owner fixture {owner_id} must still reach its active route"
            );
            assert!(
                super::extract_source_local_mode_long_chain_four_hop_asserted_head(
                    &owner_ast,
                    owner_resolver.module,
                    &owner_symbols,
                )
                .is_none(),
                "Task 228 must reject prior owner fixture {owner_id}"
            );
        }
    }

    #[test]
    fn active_local_mode_long_chain_five_hop_asserted_head_fixture_consumes_seven_expansions() {
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("mizar-test crate should live below the workspace root")
            .to_path_buf();
        let config = DiscoveryConfig {
            workspace_root: workspace_root.clone(),
            tests_root: workspace_root.join("tests"),
            manifest_path: workspace_root.join("tests/coverage/spec_trace.toml"),
            profile: TestProfile::Fast,
            validation_mode: ValidationMode::Metadata,
        };
        let plan = build_test_plan(&config).expect("Task 230 repository plan should build");
        let (ordinal, case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| {
                case.id.0
                    == "pass_type_elaboration_local_mode_long_chain_five_hop_asserted_head_001"
            })
            .expect("Task 230 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 230 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 230 fixture should produce a real AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = super::source_local_mode_long_chain_five_hop_asserted_head_output(
            &ast,
            resolver.module.clone(),
            &symbols,
        )
        .expect("Task 230 real AST should reach the seven-expansion five-hop seam");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 230 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.subject_result_input.spelling, "ChainMode6");
        assert_eq!(output.asserted_type_input.spelling, "ChainMode1");
        assert_ne!(
            output.subject_result_input.head,
            output.asserted_type_input.head
        );
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert!(output.term_formula.candidate_sets().is_empty());
        assert!(output.term_formula.facts().is_empty());
        assert!(output.term_formula.diagnostics().is_empty());
        let chain6 = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("ChainMode6"))
            .map(|(_, expansion)| expansion)
            .expect("Task 230 ChainMode6 expansion should exist");
        assert_eq!(chain6.radix.spelling, "ChainMode5");
        let TypeHeadInput::Symbol(chain5_symbol) = &chain6.radix.head else {
            panic!("Task 230 ChainMode5 first intermediate should be a symbol");
        };
        let chain5 = output
            .payload
            .reserve
            .mode_expansions
            .get(chain5_symbol)
            .expect("Task 230 ChainMode5 expansion should exist");
        assert_eq!(chain5.radix.spelling, "ChainMode4");
        assert_ne!(chain5.radix.head, output.asserted_type_input.head);
        let TypeHeadInput::Symbol(chain4_symbol) = &chain5.radix.head else {
            panic!("Task 230 ChainMode4 second intermediate should be a symbol");
        };
        let chain4 = output
            .payload
            .reserve
            .mode_expansions
            .get(chain4_symbol)
            .expect("Task 230 ChainMode4 expansion should exist");
        assert_eq!(chain4.radix.spelling, "ChainMode3");
        assert_ne!(chain4.radix.head, output.asserted_type_input.head);
        let TypeHeadInput::Symbol(chain3_symbol) = &chain4.radix.head else {
            panic!("Task 230 ChainMode3 third intermediate should be a symbol");
        };
        let chain3 = output
            .payload
            .reserve
            .mode_expansions
            .get(chain3_symbol)
            .expect("Task 230 ChainMode3 expansion should exist");
        assert_eq!(chain3.radix.spelling, "ChainMode2");
        assert_ne!(chain3.radix.head, output.asserted_type_input.head);
        let TypeHeadInput::Symbol(chain2_symbol) = &chain3.radix.head else {
            panic!("Task 230 ChainMode2 fourth intermediate should be a symbol");
        };
        let chain2 = output
            .payload
            .reserve
            .mode_expansions
            .get(chain2_symbol)
            .expect("Task 230 ChainMode2 expansion should exist");
        assert_eq!(chain2.radix.spelling, "ChainMode1");
        assert_eq!(chain2.radix.head, output.asserted_type_input.head);

        type OwnerExtractor = fn(
            &SurfaceAst,
            ResolverModuleId,
            &SymbolEnv,
        ) -> Option<super::SourceReservedVariableTypeAssertion>;
        let prior_extractors: [(&str, OwnerExtractor); 54] = [
            (
                "extract_source_reserved_variable_type_assertion",
                super::extract_source_reserved_variable_type_assertion,
            ),
            (
                "extract_source_reserved_object_variable_type_assertion",
                super::extract_source_reserved_object_variable_type_assertion,
            ),
            (
                "extract_source_local_mode_reserved_variable_type_assertion",
                super::extract_source_local_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_local_object_mode_reserved_variable_type_assertion",
                super::extract_source_local_object_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_local_mode_asserted_head",
                super::extract_source_local_mode_asserted_head,
            ),
            (
                "extract_source_local_object_mode_asserted_head",
                super::extract_source_local_object_mode_asserted_head,
            ),
            (
                "extract_source_chained_local_mode_reserved_variable_type_assertion",
                super::extract_source_chained_local_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_chained_local_object_mode_reserved_variable_type_assertion",
                super::extract_source_chained_local_object_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_chained_local_mode_asserted_head",
                super::extract_source_chained_local_mode_asserted_head,
            ),
            (
                "extract_source_chained_local_object_mode_asserted_head",
                super::extract_source_chained_local_object_mode_asserted_head,
            ),
            (
                "extract_source_chained_local_mode_radix_asserted_head",
                super::extract_source_chained_local_mode_radix_asserted_head,
            ),
            (
                "extract_source_chained_local_object_mode_radix_asserted_head",
                super::extract_source_chained_local_object_mode_radix_asserted_head,
            ),
            (
                "extract_source_two_edge_local_mode_reserved_variable_type_assertion",
                super::extract_source_two_edge_local_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_two_edge_local_object_mode_reserved_variable_type_assertion",
                super::extract_source_two_edge_local_object_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_two_edge_local_mode_asserted_head",
                super::extract_source_two_edge_local_mode_asserted_head,
            ),
            (
                "extract_source_two_edge_local_object_mode_asserted_head",
                super::extract_source_two_edge_local_object_mode_asserted_head,
            ),
            (
                "extract_source_two_edge_local_mode_radix_asserted_head",
                super::extract_source_two_edge_local_mode_radix_asserted_head,
            ),
            (
                "extract_source_two_edge_local_object_mode_radix_asserted_head",
                super::extract_source_two_edge_local_object_mode_radix_asserted_head,
            ),
            (
                "extract_source_three_edge_local_mode_reserved_variable_type_assertion",
                super::extract_source_three_edge_local_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_three_edge_local_object_mode_reserved_variable_type_assertion",
                super::extract_source_three_edge_local_object_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_three_edge_local_mode_asserted_head",
                super::extract_source_three_edge_local_mode_asserted_head,
            ),
            (
                "extract_source_three_edge_local_object_mode_asserted_head",
                super::extract_source_three_edge_local_object_mode_asserted_head,
            ),
            (
                "extract_source_three_edge_local_mode_radix_asserted_head",
                super::extract_source_three_edge_local_mode_radix_asserted_head,
            ),
            (
                "extract_source_three_edge_local_object_mode_radix_asserted_head",
                super::extract_source_three_edge_local_object_mode_radix_asserted_head,
            ),
            (
                "extract_source_four_edge_local_mode_reserved_variable_type_assertion",
                super::extract_source_four_edge_local_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_four_edge_local_object_mode_reserved_variable_type_assertion",
                super::extract_source_four_edge_local_object_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_four_edge_local_mode_asserted_head",
                super::extract_source_four_edge_local_mode_asserted_head,
            ),
            (
                "extract_source_four_edge_local_object_mode_asserted_head",
                super::extract_source_four_edge_local_object_mode_asserted_head,
            ),
            (
                "extract_source_four_edge_local_mode_radix_asserted_head",
                super::extract_source_four_edge_local_mode_radix_asserted_head,
            ),
            (
                "extract_source_four_edge_local_object_mode_radix_asserted_head",
                super::extract_source_four_edge_local_object_mode_radix_asserted_head,
            ),
            (
                "extract_source_local_mode_long_chain_reserved_variable_type_assertion",
                super::extract_source_local_mode_long_chain_reserved_variable_type_assertion,
            ),
            (
                "extract_source_local_object_mode_long_chain_reserved_variable_type_assertion",
                super::extract_source_local_object_mode_long_chain_reserved_variable_type_assertion,
            ),
            (
                "extract_source_local_mode_long_chain_asserted_head",
                super::extract_source_local_mode_long_chain_asserted_head,
            ),
            (
                "extract_source_local_object_mode_long_chain_asserted_head",
                super::extract_source_local_object_mode_long_chain_asserted_head,
            ),
            (
                "extract_source_local_mode_long_chain_radix_asserted_head",
                super::extract_source_local_mode_long_chain_radix_asserted_head,
            ),
            (
                "extract_source_local_object_mode_long_chain_radix_asserted_head",
                super::extract_source_local_object_mode_long_chain_radix_asserted_head,
            ),
            (
                "extract_source_local_mode_long_chain_two_hop_asserted_head",
                super::extract_source_local_mode_long_chain_two_hop_asserted_head,
            ),
            (
                "extract_source_local_object_mode_long_chain_two_hop_asserted_head",
                super::extract_source_local_object_mode_long_chain_two_hop_asserted_head,
            ),
            (
                "extract_source_two_edge_local_mode_two_hop_asserted_head",
                super::extract_source_two_edge_local_mode_two_hop_asserted_head,
            ),
            (
                "extract_source_two_edge_local_object_mode_two_hop_asserted_head",
                super::extract_source_two_edge_local_object_mode_two_hop_asserted_head,
            ),
            (
                "extract_source_three_edge_local_mode_two_hop_asserted_head",
                super::extract_source_three_edge_local_mode_two_hop_asserted_head,
            ),
            (
                "extract_source_three_edge_local_object_mode_two_hop_asserted_head",
                super::extract_source_three_edge_local_object_mode_two_hop_asserted_head,
            ),
            (
                "extract_source_four_edge_local_mode_two_hop_asserted_head",
                super::extract_source_four_edge_local_mode_two_hop_asserted_head,
            ),
            (
                "extract_source_four_edge_local_object_mode_two_hop_asserted_head",
                super::extract_source_four_edge_local_object_mode_two_hop_asserted_head,
            ),
            (
                "extract_source_three_edge_local_mode_three_hop_asserted_head",
                super::extract_source_three_edge_local_mode_three_hop_asserted_head,
            ),
            (
                "extract_source_three_edge_local_object_mode_three_hop_asserted_head",
                super::extract_source_three_edge_local_object_mode_three_hop_asserted_head,
            ),
            (
                "extract_source_four_edge_local_mode_three_hop_asserted_head",
                super::extract_source_four_edge_local_mode_three_hop_asserted_head,
            ),
            (
                "extract_source_four_edge_local_object_mode_three_hop_asserted_head",
                super::extract_source_four_edge_local_object_mode_three_hop_asserted_head,
            ),
            (
                "extract_source_four_edge_local_mode_four_hop_asserted_head",
                super::extract_source_four_edge_local_mode_four_hop_asserted_head,
            ),
            (
                "extract_source_four_edge_local_object_mode_four_hop_asserted_head",
                super::extract_source_four_edge_local_object_mode_four_hop_asserted_head,
            ),
            (
                "extract_source_local_mode_long_chain_three_hop_asserted_head",
                super::extract_source_local_mode_long_chain_three_hop_asserted_head,
            ),
            (
                "extract_source_local_object_mode_long_chain_three_hop_asserted_head",
                super::extract_source_local_object_mode_long_chain_three_hop_asserted_head,
            ),
            (
                "extract_source_local_mode_long_chain_four_hop_asserted_head",
                super::extract_source_local_mode_long_chain_four_hop_asserted_head,
            ),
            (
                "extract_source_local_object_mode_long_chain_four_hop_asserted_head",
                super::extract_source_local_object_mode_long_chain_four_hop_asserted_head,
            ),
        ];
        for (owner, extractor) in prior_extractors {
            assert!(
                extractor(&ast, resolver.module.clone(), &symbols).is_none(),
                "Task 230 source must not be owned by {owner}"
            );
        }

        let prior_owner_ids = [
            "pass_type_elaboration_reserved_variable_type_assertion_001",
            "pass_type_elaboration_reserved_object_variable_type_assertion_001",
            "pass_type_elaboration_local_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_local_object_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_local_mode_asserted_head_001",
            "pass_type_elaboration_local_object_mode_asserted_head_001",
            "pass_type_elaboration_chained_local_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_chained_local_object_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_chained_local_mode_asserted_head_001",
            "pass_type_elaboration_chained_local_object_mode_asserted_head_001",
            "pass_type_elaboration_chained_local_mode_radix_asserted_head_001",
            "pass_type_elaboration_chained_local_object_mode_radix_asserted_head_001",
            "pass_type_elaboration_two_edge_local_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_two_edge_local_object_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_two_edge_local_mode_asserted_head_001",
            "pass_type_elaboration_two_edge_local_object_mode_asserted_head_001",
            "pass_type_elaboration_two_edge_local_mode_radix_asserted_head_001",
            "pass_type_elaboration_two_edge_local_object_mode_radix_asserted_head_001",
            "pass_type_elaboration_three_edge_local_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_three_edge_local_object_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_three_edge_local_mode_asserted_head_001",
            "pass_type_elaboration_three_edge_local_object_mode_asserted_head_001",
            "pass_type_elaboration_three_edge_local_mode_radix_asserted_head_001",
            "pass_type_elaboration_three_edge_local_object_mode_radix_asserted_head_001",
            "pass_type_elaboration_four_edge_local_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_four_edge_local_object_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_four_edge_local_mode_asserted_head_001",
            "pass_type_elaboration_four_edge_local_object_mode_asserted_head_001",
            "pass_type_elaboration_four_edge_local_mode_radix_asserted_head_001",
            "pass_type_elaboration_four_edge_local_object_mode_radix_asserted_head_001",
            "pass_type_elaboration_local_mode_long_chain_reserved_variable_type_assertion_001",
            "pass_type_elaboration_local_object_mode_long_chain_reserved_variable_type_assertion_001",
            "pass_type_elaboration_local_mode_long_chain_asserted_head_001",
            "pass_type_elaboration_local_object_mode_long_chain_asserted_head_001",
            "pass_type_elaboration_local_mode_long_chain_radix_asserted_head_001",
            "pass_type_elaboration_local_object_mode_long_chain_radix_asserted_head_001",
            "pass_type_elaboration_local_mode_long_chain_two_hop_asserted_head_001",
            "pass_type_elaboration_local_object_mode_long_chain_two_hop_asserted_head_001",
            "pass_type_elaboration_two_edge_local_mode_two_hop_asserted_head_001",
            "pass_type_elaboration_two_edge_local_object_mode_two_hop_asserted_head_001",
            "pass_type_elaboration_three_edge_local_mode_two_hop_asserted_head_001",
            "pass_type_elaboration_three_edge_local_object_mode_two_hop_asserted_head_001",
            "pass_type_elaboration_four_edge_local_mode_two_hop_asserted_head_001",
            "pass_type_elaboration_four_edge_local_object_mode_two_hop_asserted_head_001",
            "pass_type_elaboration_three_edge_local_mode_three_hop_asserted_head_001",
            "pass_type_elaboration_three_edge_local_object_mode_three_hop_asserted_head_001",
            "pass_type_elaboration_four_edge_local_mode_three_hop_asserted_head_001",
            "pass_type_elaboration_four_edge_local_object_mode_three_hop_asserted_head_001",
            "pass_type_elaboration_four_edge_local_mode_four_hop_asserted_head_001",
            "pass_type_elaboration_four_edge_local_object_mode_four_hop_asserted_head_001",
            "pass_type_elaboration_local_mode_long_chain_three_hop_asserted_head_001",
            "pass_type_elaboration_local_object_mode_long_chain_three_hop_asserted_head_001",
            "pass_type_elaboration_local_mode_long_chain_four_hop_asserted_head_001",
            "pass_type_elaboration_local_object_mode_long_chain_four_hop_asserted_head_001",
        ];
        assert_eq!(prior_owner_ids.len(), 54);
        for owner_id in prior_owner_ids {
            let (owner_ordinal, owner_case) = active_type_elaboration_cases(&plan)
                .enumerate()
                .find(|(_, case)| case.id.0 == owner_id)
                .unwrap_or_else(|| panic!("prior owner fixture {owner_id} must be active"));
            let owner_frontend = run_frontend(&workspace_root, owner_case, owner_ordinal)
                .unwrap_or_else(|error| panic!("owner fixture {owner_id} must parse: {error}"));
            assert!(owner_frontend.diagnostics.is_empty());
            let owner_ast = owner_frontend
                .ast
                .unwrap_or_else(|| panic!("owner fixture {owner_id} must produce an AST"));
            let owner_resolver =
                resolver_symbol_collection(&workspace_root, owner_case, &owner_ast);
            assert!(owner_resolver.detail_keys.is_empty());
            let owner_symbols = augment_type_elaboration_import_summaries(
                &owner_ast,
                &owner_resolver.module,
                owner_resolver.env,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                ),
                Vec::<String>::new(),
                "prior owner fixture {owner_id} must still reach its active route"
            );
            assert!(
                super::extract_source_local_mode_long_chain_five_hop_asserted_head(
                    &owner_ast,
                    owner_resolver.module,
                    &owner_symbols,
                )
                .is_none(),
                "Task 230 must reject prior owner fixture {owner_id}"
            );
        }
    }

    #[test]
    fn active_local_object_mode_long_chain_five_hop_asserted_head_fixture_consumes_seven_expansions()
     {
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("mizar-test crate should live below the workspace root")
            .to_path_buf();
        let config = DiscoveryConfig {
            workspace_root: workspace_root.clone(),
            tests_root: workspace_root.join("tests"),
            manifest_path: workspace_root.join("tests/coverage/spec_trace.toml"),
            profile: TestProfile::Fast,
            validation_mode: ValidationMode::Metadata,
        };
        let plan = build_test_plan(&config).expect("Task 231 repository plan should build");
        let (ordinal, case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| {
                case.id.0
                    == "pass_type_elaboration_local_object_mode_long_chain_five_hop_asserted_head_001"
            })
            .expect("Task 231 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 231 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 231 fixture should produce a real AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = super::source_local_object_mode_long_chain_five_hop_asserted_head_output(
            &ast,
            resolver.module.clone(),
            &symbols,
        )
        .expect("Task 231 real AST should reach the seven-expansion five-hop seam");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 231 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.subject_result_input.spelling, "ChainObjectMode6");
        assert_eq!(output.asserted_type_input.spelling, "ChainObjectMode1");
        assert_ne!(
            output.subject_result_input.head,
            output.asserted_type_input.head
        );
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert!(output.term_formula.candidate_sets().is_empty());
        assert!(output.term_formula.facts().is_empty());
        assert!(output.term_formula.diagnostics().is_empty());
        let chain6 = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("ChainObjectMode6"))
            .map(|(_, expansion)| expansion)
            .expect("Task 231 ChainObjectMode6 expansion should exist");
        assert_eq!(chain6.radix.spelling, "ChainObjectMode5");
        let TypeHeadInput::Symbol(chain5_symbol) = &chain6.radix.head else {
            panic!("Task 231 ChainObjectMode5 first intermediate should be a symbol");
        };
        let chain5 = output
            .payload
            .reserve
            .mode_expansions
            .get(chain5_symbol)
            .expect("Task 231 ChainObjectMode5 expansion should exist");
        assert_eq!(chain5.radix.spelling, "ChainObjectMode4");
        assert_ne!(chain5.radix.head, output.asserted_type_input.head);
        let TypeHeadInput::Symbol(chain4_symbol) = &chain5.radix.head else {
            panic!("Task 231 ChainObjectMode4 second intermediate should be a symbol");
        };
        let chain4 = output
            .payload
            .reserve
            .mode_expansions
            .get(chain4_symbol)
            .expect("Task 231 ChainObjectMode4 expansion should exist");
        assert_eq!(chain4.radix.spelling, "ChainObjectMode3");
        assert_ne!(chain4.radix.head, output.asserted_type_input.head);
        let TypeHeadInput::Symbol(chain3_symbol) = &chain4.radix.head else {
            panic!("Task 231 ChainObjectMode3 third intermediate should be a symbol");
        };
        let chain3 = output
            .payload
            .reserve
            .mode_expansions
            .get(chain3_symbol)
            .expect("Task 231 ChainObjectMode3 expansion should exist");
        assert_eq!(chain3.radix.spelling, "ChainObjectMode2");
        assert_ne!(chain3.radix.head, output.asserted_type_input.head);
        let TypeHeadInput::Symbol(chain2_symbol) = &chain3.radix.head else {
            panic!("Task 231 ChainObjectMode2 fourth intermediate should be a symbol");
        };
        let chain2 = output
            .payload
            .reserve
            .mode_expansions
            .get(chain2_symbol)
            .expect("Task 231 ChainObjectMode2 expansion should exist");
        assert_eq!(chain2.radix.spelling, "ChainObjectMode1");
        assert_eq!(chain2.radix.head, output.asserted_type_input.head);

        type OwnerExtractor = fn(
            &SurfaceAst,
            ResolverModuleId,
            &SymbolEnv,
        ) -> Option<super::SourceReservedVariableTypeAssertion>;
        let prior_extractors: [(&str, OwnerExtractor); 55] = [
            (
                "extract_source_reserved_variable_type_assertion",
                super::extract_source_reserved_variable_type_assertion,
            ),
            (
                "extract_source_reserved_object_variable_type_assertion",
                super::extract_source_reserved_object_variable_type_assertion,
            ),
            (
                "extract_source_local_object_mode_reserved_variable_type_assertion",
                super::extract_source_local_object_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_local_mode_reserved_variable_type_assertion",
                super::extract_source_local_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_local_object_mode_asserted_head",
                super::extract_source_local_object_mode_asserted_head,
            ),
            (
                "extract_source_local_mode_asserted_head",
                super::extract_source_local_mode_asserted_head,
            ),
            (
                "extract_source_chained_local_object_mode_reserved_variable_type_assertion",
                super::extract_source_chained_local_object_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_chained_local_mode_reserved_variable_type_assertion",
                super::extract_source_chained_local_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_chained_local_object_mode_asserted_head",
                super::extract_source_chained_local_object_mode_asserted_head,
            ),
            (
                "extract_source_chained_local_mode_asserted_head",
                super::extract_source_chained_local_mode_asserted_head,
            ),
            (
                "extract_source_chained_local_object_mode_radix_asserted_head",
                super::extract_source_chained_local_object_mode_radix_asserted_head,
            ),
            (
                "extract_source_chained_local_mode_radix_asserted_head",
                super::extract_source_chained_local_mode_radix_asserted_head,
            ),
            (
                "extract_source_two_edge_local_object_mode_reserved_variable_type_assertion",
                super::extract_source_two_edge_local_object_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_two_edge_local_mode_reserved_variable_type_assertion",
                super::extract_source_two_edge_local_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_two_edge_local_object_mode_asserted_head",
                super::extract_source_two_edge_local_object_mode_asserted_head,
            ),
            (
                "extract_source_two_edge_local_mode_asserted_head",
                super::extract_source_two_edge_local_mode_asserted_head,
            ),
            (
                "extract_source_two_edge_local_object_mode_radix_asserted_head",
                super::extract_source_two_edge_local_object_mode_radix_asserted_head,
            ),
            (
                "extract_source_two_edge_local_mode_radix_asserted_head",
                super::extract_source_two_edge_local_mode_radix_asserted_head,
            ),
            (
                "extract_source_three_edge_local_object_mode_reserved_variable_type_assertion",
                super::extract_source_three_edge_local_object_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_three_edge_local_mode_reserved_variable_type_assertion",
                super::extract_source_three_edge_local_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_three_edge_local_object_mode_asserted_head",
                super::extract_source_three_edge_local_object_mode_asserted_head,
            ),
            (
                "extract_source_three_edge_local_mode_asserted_head",
                super::extract_source_three_edge_local_mode_asserted_head,
            ),
            (
                "extract_source_three_edge_local_object_mode_radix_asserted_head",
                super::extract_source_three_edge_local_object_mode_radix_asserted_head,
            ),
            (
                "extract_source_three_edge_local_mode_radix_asserted_head",
                super::extract_source_three_edge_local_mode_radix_asserted_head,
            ),
            (
                "extract_source_four_edge_local_object_mode_reserved_variable_type_assertion",
                super::extract_source_four_edge_local_object_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_four_edge_local_mode_reserved_variable_type_assertion",
                super::extract_source_four_edge_local_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_four_edge_local_object_mode_asserted_head",
                super::extract_source_four_edge_local_object_mode_asserted_head,
            ),
            (
                "extract_source_four_edge_local_mode_asserted_head",
                super::extract_source_four_edge_local_mode_asserted_head,
            ),
            (
                "extract_source_four_edge_local_object_mode_radix_asserted_head",
                super::extract_source_four_edge_local_object_mode_radix_asserted_head,
            ),
            (
                "extract_source_four_edge_local_mode_radix_asserted_head",
                super::extract_source_four_edge_local_mode_radix_asserted_head,
            ),
            (
                "extract_source_local_object_mode_long_chain_reserved_variable_type_assertion",
                super::extract_source_local_object_mode_long_chain_reserved_variable_type_assertion,
            ),
            (
                "extract_source_local_mode_long_chain_reserved_variable_type_assertion",
                super::extract_source_local_mode_long_chain_reserved_variable_type_assertion,
            ),
            (
                "extract_source_local_object_mode_long_chain_asserted_head",
                super::extract_source_local_object_mode_long_chain_asserted_head,
            ),
            (
                "extract_source_local_mode_long_chain_asserted_head",
                super::extract_source_local_mode_long_chain_asserted_head,
            ),
            (
                "extract_source_local_object_mode_long_chain_radix_asserted_head",
                super::extract_source_local_object_mode_long_chain_radix_asserted_head,
            ),
            (
                "extract_source_local_mode_long_chain_radix_asserted_head",
                super::extract_source_local_mode_long_chain_radix_asserted_head,
            ),
            (
                "extract_source_local_object_mode_long_chain_two_hop_asserted_head",
                super::extract_source_local_object_mode_long_chain_two_hop_asserted_head,
            ),
            (
                "extract_source_local_mode_long_chain_two_hop_asserted_head",
                super::extract_source_local_mode_long_chain_two_hop_asserted_head,
            ),
            (
                "extract_source_two_edge_local_object_mode_two_hop_asserted_head",
                super::extract_source_two_edge_local_object_mode_two_hop_asserted_head,
            ),
            (
                "extract_source_two_edge_local_mode_two_hop_asserted_head",
                super::extract_source_two_edge_local_mode_two_hop_asserted_head,
            ),
            (
                "extract_source_three_edge_local_object_mode_two_hop_asserted_head",
                super::extract_source_three_edge_local_object_mode_two_hop_asserted_head,
            ),
            (
                "extract_source_three_edge_local_mode_two_hop_asserted_head",
                super::extract_source_three_edge_local_mode_two_hop_asserted_head,
            ),
            (
                "extract_source_four_edge_local_object_mode_two_hop_asserted_head",
                super::extract_source_four_edge_local_object_mode_two_hop_asserted_head,
            ),
            (
                "extract_source_four_edge_local_mode_two_hop_asserted_head",
                super::extract_source_four_edge_local_mode_two_hop_asserted_head,
            ),
            (
                "extract_source_three_edge_local_object_mode_three_hop_asserted_head",
                super::extract_source_three_edge_local_object_mode_three_hop_asserted_head,
            ),
            (
                "extract_source_three_edge_local_mode_three_hop_asserted_head",
                super::extract_source_three_edge_local_mode_three_hop_asserted_head,
            ),
            (
                "extract_source_four_edge_local_object_mode_three_hop_asserted_head",
                super::extract_source_four_edge_local_object_mode_three_hop_asserted_head,
            ),
            (
                "extract_source_four_edge_local_mode_three_hop_asserted_head",
                super::extract_source_four_edge_local_mode_three_hop_asserted_head,
            ),
            (
                "extract_source_four_edge_local_object_mode_four_hop_asserted_head",
                super::extract_source_four_edge_local_object_mode_four_hop_asserted_head,
            ),
            (
                "extract_source_four_edge_local_mode_four_hop_asserted_head",
                super::extract_source_four_edge_local_mode_four_hop_asserted_head,
            ),
            (
                "extract_source_local_object_mode_long_chain_three_hop_asserted_head",
                super::extract_source_local_object_mode_long_chain_three_hop_asserted_head,
            ),
            (
                "extract_source_local_mode_long_chain_three_hop_asserted_head",
                super::extract_source_local_mode_long_chain_three_hop_asserted_head,
            ),
            (
                "extract_source_local_object_mode_long_chain_four_hop_asserted_head",
                super::extract_source_local_object_mode_long_chain_four_hop_asserted_head,
            ),
            (
                "extract_source_local_mode_long_chain_four_hop_asserted_head",
                super::extract_source_local_mode_long_chain_four_hop_asserted_head,
            ),
            (
                "extract_source_local_mode_long_chain_five_hop_asserted_head",
                super::extract_source_local_mode_long_chain_five_hop_asserted_head,
            ),
        ];
        for (owner, extractor) in prior_extractors {
            assert!(
                extractor(&ast, resolver.module.clone(), &symbols).is_none(),
                "Task 231 source must not be owned by {owner}"
            );
        }

        let prior_owner_ids = [
            "pass_type_elaboration_reserved_variable_type_assertion_001",
            "pass_type_elaboration_reserved_object_variable_type_assertion_001",
            "pass_type_elaboration_local_object_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_local_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_local_object_mode_asserted_head_001",
            "pass_type_elaboration_local_mode_asserted_head_001",
            "pass_type_elaboration_chained_local_object_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_chained_local_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_chained_local_object_mode_asserted_head_001",
            "pass_type_elaboration_chained_local_mode_asserted_head_001",
            "pass_type_elaboration_chained_local_object_mode_radix_asserted_head_001",
            "pass_type_elaboration_chained_local_mode_radix_asserted_head_001",
            "pass_type_elaboration_two_edge_local_object_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_two_edge_local_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_two_edge_local_object_mode_asserted_head_001",
            "pass_type_elaboration_two_edge_local_mode_asserted_head_001",
            "pass_type_elaboration_two_edge_local_object_mode_radix_asserted_head_001",
            "pass_type_elaboration_two_edge_local_mode_radix_asserted_head_001",
            "pass_type_elaboration_three_edge_local_object_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_three_edge_local_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_three_edge_local_object_mode_asserted_head_001",
            "pass_type_elaboration_three_edge_local_mode_asserted_head_001",
            "pass_type_elaboration_three_edge_local_object_mode_radix_asserted_head_001",
            "pass_type_elaboration_three_edge_local_mode_radix_asserted_head_001",
            "pass_type_elaboration_four_edge_local_object_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_four_edge_local_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_four_edge_local_object_mode_asserted_head_001",
            "pass_type_elaboration_four_edge_local_mode_asserted_head_001",
            "pass_type_elaboration_four_edge_local_object_mode_radix_asserted_head_001",
            "pass_type_elaboration_four_edge_local_mode_radix_asserted_head_001",
            "pass_type_elaboration_local_object_mode_long_chain_reserved_variable_type_assertion_001",
            "pass_type_elaboration_local_mode_long_chain_reserved_variable_type_assertion_001",
            "pass_type_elaboration_local_object_mode_long_chain_asserted_head_001",
            "pass_type_elaboration_local_mode_long_chain_asserted_head_001",
            "pass_type_elaboration_local_object_mode_long_chain_radix_asserted_head_001",
            "pass_type_elaboration_local_mode_long_chain_radix_asserted_head_001",
            "pass_type_elaboration_local_object_mode_long_chain_two_hop_asserted_head_001",
            "pass_type_elaboration_local_mode_long_chain_two_hop_asserted_head_001",
            "pass_type_elaboration_two_edge_local_object_mode_two_hop_asserted_head_001",
            "pass_type_elaboration_two_edge_local_mode_two_hop_asserted_head_001",
            "pass_type_elaboration_three_edge_local_object_mode_two_hop_asserted_head_001",
            "pass_type_elaboration_three_edge_local_mode_two_hop_asserted_head_001",
            "pass_type_elaboration_four_edge_local_object_mode_two_hop_asserted_head_001",
            "pass_type_elaboration_four_edge_local_mode_two_hop_asserted_head_001",
            "pass_type_elaboration_three_edge_local_object_mode_three_hop_asserted_head_001",
            "pass_type_elaboration_three_edge_local_mode_three_hop_asserted_head_001",
            "pass_type_elaboration_four_edge_local_object_mode_three_hop_asserted_head_001",
            "pass_type_elaboration_four_edge_local_mode_three_hop_asserted_head_001",
            "pass_type_elaboration_four_edge_local_object_mode_four_hop_asserted_head_001",
            "pass_type_elaboration_four_edge_local_mode_four_hop_asserted_head_001",
            "pass_type_elaboration_local_object_mode_long_chain_three_hop_asserted_head_001",
            "pass_type_elaboration_local_mode_long_chain_three_hop_asserted_head_001",
            "pass_type_elaboration_local_object_mode_long_chain_four_hop_asserted_head_001",
            "pass_type_elaboration_local_mode_long_chain_four_hop_asserted_head_001",
            "pass_type_elaboration_local_mode_long_chain_five_hop_asserted_head_001",
        ];
        assert_eq!(prior_owner_ids.len(), 55);
        for owner_id in prior_owner_ids {
            let (owner_ordinal, owner_case) = active_type_elaboration_cases(&plan)
                .enumerate()
                .find(|(_, case)| case.id.0 == owner_id)
                .unwrap_or_else(|| panic!("prior owner fixture {owner_id} must be active"));
            let owner_frontend = run_frontend(&workspace_root, owner_case, owner_ordinal)
                .unwrap_or_else(|error| panic!("owner fixture {owner_id} must parse: {error}"));
            assert!(owner_frontend.diagnostics.is_empty());
            let owner_ast = owner_frontend
                .ast
                .unwrap_or_else(|| panic!("owner fixture {owner_id} must produce an AST"));
            let owner_resolver =
                resolver_symbol_collection(&workspace_root, owner_case, &owner_ast);
            assert!(owner_resolver.detail_keys.is_empty());
            let owner_symbols = augment_type_elaboration_import_summaries(
                &owner_ast,
                &owner_resolver.module,
                owner_resolver.env,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                ),
                Vec::<String>::new(),
                "prior owner fixture {owner_id} must still reach its active route"
            );
            assert!(
                super::extract_source_local_object_mode_long_chain_five_hop_asserted_head(
                    &owner_ast,
                    owner_resolver.module,
                    &owner_symbols,
                )
                .is_none(),
                "Task 231 must reject prior owner fixture {owner_id}"
            );
        }
    }

    #[test]
    fn active_local_mode_long_chain_six_hop_asserted_head_fixture_consumes_seven_expansions() {
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("mizar-test crate should live below the workspace root")
            .to_path_buf();
        let config = DiscoveryConfig {
            workspace_root: workspace_root.clone(),
            tests_root: workspace_root.join("tests"),
            manifest_path: workspace_root.join("tests/coverage/spec_trace.toml"),
            profile: TestProfile::Fast,
            validation_mode: ValidationMode::Metadata,
        };
        let plan = build_test_plan(&config).expect("Task 234 repository plan should build");
        let (ordinal, case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| {
                case.id.0 == "pass_type_elaboration_local_mode_long_chain_six_hop_asserted_head_001"
            })
            .expect("Task 234 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 234 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 234 fixture should produce a real AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = super::source_local_mode_long_chain_six_hop_asserted_head_output(
            &ast,
            resolver.module.clone(),
            &symbols,
        )
        .expect("Task 234 real AST should reach the seven-expansion six-hop seam");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 234 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.subject_result_input.spelling, "ChainMode6");
        assert_eq!(output.asserted_type_input.spelling, "BaseMode");
        assert_ne!(
            output.subject_result_input.head,
            output.asserted_type_input.head
        );
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert!(output.term_formula.candidate_sets().is_empty());
        assert!(output.term_formula.facts().is_empty());
        assert!(output.term_formula.diagnostics().is_empty());
        let chain6 = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("ChainMode6"))
            .map(|(_, expansion)| expansion)
            .expect("Task 234 ChainMode6 expansion should exist");
        assert_eq!(chain6.radix.spelling, "ChainMode5");
        let TypeHeadInput::Symbol(chain5_symbol) = &chain6.radix.head else {
            panic!("Task 234 ChainMode5 first intermediate should be a symbol");
        };
        let chain5 = output
            .payload
            .reserve
            .mode_expansions
            .get(chain5_symbol)
            .expect("Task 234 ChainMode5 expansion should exist");
        assert_eq!(chain5.radix.spelling, "ChainMode4");
        assert_ne!(chain5.radix.head, output.asserted_type_input.head);
        let TypeHeadInput::Symbol(chain4_symbol) = &chain5.radix.head else {
            panic!("Task 234 ChainMode4 second intermediate should be a symbol");
        };
        let chain4 = output
            .payload
            .reserve
            .mode_expansions
            .get(chain4_symbol)
            .expect("Task 234 ChainMode4 expansion should exist");
        assert_eq!(chain4.radix.spelling, "ChainMode3");
        assert_ne!(chain4.radix.head, output.asserted_type_input.head);
        let TypeHeadInput::Symbol(chain3_symbol) = &chain4.radix.head else {
            panic!("Task 234 ChainMode3 third intermediate should be a symbol");
        };
        let chain3 = output
            .payload
            .reserve
            .mode_expansions
            .get(chain3_symbol)
            .expect("Task 234 ChainMode3 expansion should exist");
        assert_eq!(chain3.radix.spelling, "ChainMode2");
        assert_ne!(chain3.radix.head, output.asserted_type_input.head);
        let TypeHeadInput::Symbol(chain2_symbol) = &chain3.radix.head else {
            panic!("Task 234 ChainMode2 fourth intermediate should be a symbol");
        };
        let chain2 = output
            .payload
            .reserve
            .mode_expansions
            .get(chain2_symbol)
            .expect("Task 234 ChainMode2 expansion should exist");
        assert_eq!(chain2.radix.spelling, "ChainMode1");
        assert_ne!(chain2.radix.head, output.asserted_type_input.head);
        let TypeHeadInput::Symbol(chain1_symbol) = &chain2.radix.head else {
            panic!("Task 234 ChainMode1 fifth intermediate should be a symbol");
        };
        let chain1 = output
            .payload
            .reserve
            .mode_expansions
            .get(chain1_symbol)
            .expect("Task 234 ChainMode1 expansion should exist");
        assert_eq!(chain1.radix.spelling, "BaseMode");
        assert_eq!(chain1.radix.head, output.asserted_type_input.head);

        type OwnerExtractor = fn(
            &SurfaceAst,
            ResolverModuleId,
            &SymbolEnv,
        ) -> Option<super::SourceReservedVariableTypeAssertion>;
        let prior_extractors: [(&str, OwnerExtractor); 56] = [
            (
                "extract_source_reserved_variable_type_assertion",
                super::extract_source_reserved_variable_type_assertion,
            ),
            (
                "extract_source_reserved_object_variable_type_assertion",
                super::extract_source_reserved_object_variable_type_assertion,
            ),
            (
                "extract_source_local_object_mode_reserved_variable_type_assertion",
                super::extract_source_local_object_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_local_mode_reserved_variable_type_assertion",
                super::extract_source_local_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_local_object_mode_asserted_head",
                super::extract_source_local_object_mode_asserted_head,
            ),
            (
                "extract_source_local_mode_asserted_head",
                super::extract_source_local_mode_asserted_head,
            ),
            (
                "extract_source_chained_local_object_mode_reserved_variable_type_assertion",
                super::extract_source_chained_local_object_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_chained_local_mode_reserved_variable_type_assertion",
                super::extract_source_chained_local_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_chained_local_object_mode_asserted_head",
                super::extract_source_chained_local_object_mode_asserted_head,
            ),
            (
                "extract_source_chained_local_mode_asserted_head",
                super::extract_source_chained_local_mode_asserted_head,
            ),
            (
                "extract_source_chained_local_object_mode_radix_asserted_head",
                super::extract_source_chained_local_object_mode_radix_asserted_head,
            ),
            (
                "extract_source_chained_local_mode_radix_asserted_head",
                super::extract_source_chained_local_mode_radix_asserted_head,
            ),
            (
                "extract_source_two_edge_local_object_mode_reserved_variable_type_assertion",
                super::extract_source_two_edge_local_object_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_two_edge_local_mode_reserved_variable_type_assertion",
                super::extract_source_two_edge_local_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_two_edge_local_object_mode_asserted_head",
                super::extract_source_two_edge_local_object_mode_asserted_head,
            ),
            (
                "extract_source_two_edge_local_mode_asserted_head",
                super::extract_source_two_edge_local_mode_asserted_head,
            ),
            (
                "extract_source_two_edge_local_object_mode_radix_asserted_head",
                super::extract_source_two_edge_local_object_mode_radix_asserted_head,
            ),
            (
                "extract_source_two_edge_local_mode_radix_asserted_head",
                super::extract_source_two_edge_local_mode_radix_asserted_head,
            ),
            (
                "extract_source_three_edge_local_object_mode_reserved_variable_type_assertion",
                super::extract_source_three_edge_local_object_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_three_edge_local_mode_reserved_variable_type_assertion",
                super::extract_source_three_edge_local_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_three_edge_local_object_mode_asserted_head",
                super::extract_source_three_edge_local_object_mode_asserted_head,
            ),
            (
                "extract_source_three_edge_local_mode_asserted_head",
                super::extract_source_three_edge_local_mode_asserted_head,
            ),
            (
                "extract_source_three_edge_local_object_mode_radix_asserted_head",
                super::extract_source_three_edge_local_object_mode_radix_asserted_head,
            ),
            (
                "extract_source_three_edge_local_mode_radix_asserted_head",
                super::extract_source_three_edge_local_mode_radix_asserted_head,
            ),
            (
                "extract_source_four_edge_local_object_mode_reserved_variable_type_assertion",
                super::extract_source_four_edge_local_object_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_four_edge_local_mode_reserved_variable_type_assertion",
                super::extract_source_four_edge_local_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_four_edge_local_object_mode_asserted_head",
                super::extract_source_four_edge_local_object_mode_asserted_head,
            ),
            (
                "extract_source_four_edge_local_mode_asserted_head",
                super::extract_source_four_edge_local_mode_asserted_head,
            ),
            (
                "extract_source_four_edge_local_object_mode_radix_asserted_head",
                super::extract_source_four_edge_local_object_mode_radix_asserted_head,
            ),
            (
                "extract_source_four_edge_local_mode_radix_asserted_head",
                super::extract_source_four_edge_local_mode_radix_asserted_head,
            ),
            (
                "extract_source_local_object_mode_long_chain_reserved_variable_type_assertion",
                super::extract_source_local_object_mode_long_chain_reserved_variable_type_assertion,
            ),
            (
                "extract_source_local_mode_long_chain_reserved_variable_type_assertion",
                super::extract_source_local_mode_long_chain_reserved_variable_type_assertion,
            ),
            (
                "extract_source_local_object_mode_long_chain_asserted_head",
                super::extract_source_local_object_mode_long_chain_asserted_head,
            ),
            (
                "extract_source_local_mode_long_chain_asserted_head",
                super::extract_source_local_mode_long_chain_asserted_head,
            ),
            (
                "extract_source_local_object_mode_long_chain_radix_asserted_head",
                super::extract_source_local_object_mode_long_chain_radix_asserted_head,
            ),
            (
                "extract_source_local_mode_long_chain_radix_asserted_head",
                super::extract_source_local_mode_long_chain_radix_asserted_head,
            ),
            (
                "extract_source_local_object_mode_long_chain_two_hop_asserted_head",
                super::extract_source_local_object_mode_long_chain_two_hop_asserted_head,
            ),
            (
                "extract_source_local_mode_long_chain_two_hop_asserted_head",
                super::extract_source_local_mode_long_chain_two_hop_asserted_head,
            ),
            (
                "extract_source_two_edge_local_object_mode_two_hop_asserted_head",
                super::extract_source_two_edge_local_object_mode_two_hop_asserted_head,
            ),
            (
                "extract_source_two_edge_local_mode_two_hop_asserted_head",
                super::extract_source_two_edge_local_mode_two_hop_asserted_head,
            ),
            (
                "extract_source_three_edge_local_object_mode_two_hop_asserted_head",
                super::extract_source_three_edge_local_object_mode_two_hop_asserted_head,
            ),
            (
                "extract_source_three_edge_local_mode_two_hop_asserted_head",
                super::extract_source_three_edge_local_mode_two_hop_asserted_head,
            ),
            (
                "extract_source_four_edge_local_object_mode_two_hop_asserted_head",
                super::extract_source_four_edge_local_object_mode_two_hop_asserted_head,
            ),
            (
                "extract_source_four_edge_local_mode_two_hop_asserted_head",
                super::extract_source_four_edge_local_mode_two_hop_asserted_head,
            ),
            (
                "extract_source_three_edge_local_object_mode_three_hop_asserted_head",
                super::extract_source_three_edge_local_object_mode_three_hop_asserted_head,
            ),
            (
                "extract_source_three_edge_local_mode_three_hop_asserted_head",
                super::extract_source_three_edge_local_mode_three_hop_asserted_head,
            ),
            (
                "extract_source_four_edge_local_object_mode_three_hop_asserted_head",
                super::extract_source_four_edge_local_object_mode_three_hop_asserted_head,
            ),
            (
                "extract_source_four_edge_local_mode_three_hop_asserted_head",
                super::extract_source_four_edge_local_mode_three_hop_asserted_head,
            ),
            (
                "extract_source_four_edge_local_object_mode_four_hop_asserted_head",
                super::extract_source_four_edge_local_object_mode_four_hop_asserted_head,
            ),
            (
                "extract_source_four_edge_local_mode_four_hop_asserted_head",
                super::extract_source_four_edge_local_mode_four_hop_asserted_head,
            ),
            (
                "extract_source_local_object_mode_long_chain_three_hop_asserted_head",
                super::extract_source_local_object_mode_long_chain_three_hop_asserted_head,
            ),
            (
                "extract_source_local_mode_long_chain_three_hop_asserted_head",
                super::extract_source_local_mode_long_chain_three_hop_asserted_head,
            ),
            (
                "extract_source_local_object_mode_long_chain_four_hop_asserted_head",
                super::extract_source_local_object_mode_long_chain_four_hop_asserted_head,
            ),
            (
                "extract_source_local_mode_long_chain_four_hop_asserted_head",
                super::extract_source_local_mode_long_chain_four_hop_asserted_head,
            ),
            (
                "extract_source_local_mode_long_chain_five_hop_asserted_head",
                super::extract_source_local_mode_long_chain_five_hop_asserted_head,
            ),
            (
                "extract_source_local_object_mode_long_chain_five_hop_asserted_head",
                super::extract_source_local_object_mode_long_chain_five_hop_asserted_head,
            ),
        ];
        for (owner, extractor) in prior_extractors {
            assert!(
                extractor(&ast, resolver.module.clone(), &symbols).is_none(),
                "Task 234 source must not be owned by {owner}"
            );
        }

        let prior_owner_ids = [
            "pass_type_elaboration_reserved_variable_type_assertion_001",
            "pass_type_elaboration_reserved_object_variable_type_assertion_001",
            "pass_type_elaboration_local_object_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_local_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_local_object_mode_asserted_head_001",
            "pass_type_elaboration_local_mode_asserted_head_001",
            "pass_type_elaboration_chained_local_object_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_chained_local_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_chained_local_object_mode_asserted_head_001",
            "pass_type_elaboration_chained_local_mode_asserted_head_001",
            "pass_type_elaboration_chained_local_object_mode_radix_asserted_head_001",
            "pass_type_elaboration_chained_local_mode_radix_asserted_head_001",
            "pass_type_elaboration_two_edge_local_object_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_two_edge_local_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_two_edge_local_object_mode_asserted_head_001",
            "pass_type_elaboration_two_edge_local_mode_asserted_head_001",
            "pass_type_elaboration_two_edge_local_object_mode_radix_asserted_head_001",
            "pass_type_elaboration_two_edge_local_mode_radix_asserted_head_001",
            "pass_type_elaboration_three_edge_local_object_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_three_edge_local_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_three_edge_local_object_mode_asserted_head_001",
            "pass_type_elaboration_three_edge_local_mode_asserted_head_001",
            "pass_type_elaboration_three_edge_local_object_mode_radix_asserted_head_001",
            "pass_type_elaboration_three_edge_local_mode_radix_asserted_head_001",
            "pass_type_elaboration_four_edge_local_object_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_four_edge_local_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_four_edge_local_object_mode_asserted_head_001",
            "pass_type_elaboration_four_edge_local_mode_asserted_head_001",
            "pass_type_elaboration_four_edge_local_object_mode_radix_asserted_head_001",
            "pass_type_elaboration_four_edge_local_mode_radix_asserted_head_001",
            "pass_type_elaboration_local_object_mode_long_chain_reserved_variable_type_assertion_001",
            "pass_type_elaboration_local_mode_long_chain_reserved_variable_type_assertion_001",
            "pass_type_elaboration_local_object_mode_long_chain_asserted_head_001",
            "pass_type_elaboration_local_mode_long_chain_asserted_head_001",
            "pass_type_elaboration_local_object_mode_long_chain_radix_asserted_head_001",
            "pass_type_elaboration_local_mode_long_chain_radix_asserted_head_001",
            "pass_type_elaboration_local_object_mode_long_chain_two_hop_asserted_head_001",
            "pass_type_elaboration_local_mode_long_chain_two_hop_asserted_head_001",
            "pass_type_elaboration_two_edge_local_object_mode_two_hop_asserted_head_001",
            "pass_type_elaboration_two_edge_local_mode_two_hop_asserted_head_001",
            "pass_type_elaboration_three_edge_local_object_mode_two_hop_asserted_head_001",
            "pass_type_elaboration_three_edge_local_mode_two_hop_asserted_head_001",
            "pass_type_elaboration_four_edge_local_object_mode_two_hop_asserted_head_001",
            "pass_type_elaboration_four_edge_local_mode_two_hop_asserted_head_001",
            "pass_type_elaboration_three_edge_local_object_mode_three_hop_asserted_head_001",
            "pass_type_elaboration_three_edge_local_mode_three_hop_asserted_head_001",
            "pass_type_elaboration_four_edge_local_object_mode_three_hop_asserted_head_001",
            "pass_type_elaboration_four_edge_local_mode_three_hop_asserted_head_001",
            "pass_type_elaboration_four_edge_local_object_mode_four_hop_asserted_head_001",
            "pass_type_elaboration_four_edge_local_mode_four_hop_asserted_head_001",
            "pass_type_elaboration_local_object_mode_long_chain_three_hop_asserted_head_001",
            "pass_type_elaboration_local_mode_long_chain_three_hop_asserted_head_001",
            "pass_type_elaboration_local_object_mode_long_chain_four_hop_asserted_head_001",
            "pass_type_elaboration_local_mode_long_chain_four_hop_asserted_head_001",
            "pass_type_elaboration_local_mode_long_chain_five_hop_asserted_head_001",
            "pass_type_elaboration_local_object_mode_long_chain_five_hop_asserted_head_001",
        ];
        assert_eq!(prior_owner_ids.len(), 56);
        for owner_id in prior_owner_ids {
            let (owner_ordinal, owner_case) = active_type_elaboration_cases(&plan)
                .enumerate()
                .find(|(_, case)| case.id.0 == owner_id)
                .unwrap_or_else(|| panic!("prior owner fixture {owner_id} must be active"));
            let owner_frontend = run_frontend(&workspace_root, owner_case, owner_ordinal)
                .unwrap_or_else(|error| panic!("owner fixture {owner_id} must parse: {error}"));
            assert!(owner_frontend.diagnostics.is_empty());
            let owner_ast = owner_frontend
                .ast
                .unwrap_or_else(|| panic!("owner fixture {owner_id} must produce an AST"));
            let owner_resolver =
                resolver_symbol_collection(&workspace_root, owner_case, &owner_ast);
            assert!(owner_resolver.detail_keys.is_empty());
            let owner_symbols = augment_type_elaboration_import_summaries(
                &owner_ast,
                &owner_resolver.module,
                owner_resolver.env,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                ),
                Vec::<String>::new(),
                "prior owner fixture {owner_id} must still reach its active route"
            );
            assert!(
                super::extract_source_local_mode_long_chain_six_hop_asserted_head(
                    &owner_ast,
                    owner_resolver.module,
                    &owner_symbols,
                )
                .is_none(),
                "Task 234 must reject prior owner fixture {owner_id}"
            );
        }
    }

    #[test]
    fn active_local_object_mode_long_chain_six_hop_asserted_head_fixture_consumes_seven_expansions()
    {
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("mizar-test crate should live below the workspace root")
            .to_path_buf();
        let config = DiscoveryConfig {
            workspace_root: workspace_root.clone(),
            tests_root: workspace_root.join("tests"),
            manifest_path: workspace_root.join("tests/coverage/spec_trace.toml"),
            profile: TestProfile::Fast,
            validation_mode: ValidationMode::Metadata,
        };
        let plan = build_test_plan(&config).expect("Task 236 repository plan should build");
        let (ordinal, case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| {
                case.id.0 == "pass_type_elaboration_local_object_mode_long_chain_six_hop_asserted_head_001"
            })
            .expect("Task 236 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 236 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 236 fixture should produce a real AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = super::source_local_object_mode_long_chain_six_hop_asserted_head_output(
            &ast,
            resolver.module.clone(),
            &symbols,
        )
        .expect("Task 236 real AST should reach the seven-expansion six-hop seam");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 236 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.subject_result_input.spelling, "ChainObjectMode6");
        assert_eq!(output.asserted_type_input.spelling, "BaseObjectMode");
        assert_ne!(
            output.subject_result_input.head,
            output.asserted_type_input.head
        );
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert!(output.term_formula.candidate_sets().is_empty());
        assert!(output.term_formula.facts().is_empty());
        assert!(output.term_formula.diagnostics().is_empty());
        let chain6 = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("ChainObjectMode6"))
            .map(|(_, expansion)| expansion)
            .expect("Task 236 ChainObjectMode6 expansion should exist");
        assert_eq!(chain6.radix.spelling, "ChainObjectMode5");
        let TypeHeadInput::Symbol(chain5_symbol) = &chain6.radix.head else {
            panic!("Task 236 ChainObjectMode5 first intermediate should be a symbol");
        };
        let chain5 = output
            .payload
            .reserve
            .mode_expansions
            .get(chain5_symbol)
            .expect("Task 236 ChainObjectMode5 expansion should exist");
        assert_eq!(chain5.radix.spelling, "ChainObjectMode4");
        assert_ne!(chain5.radix.head, output.asserted_type_input.head);
        let TypeHeadInput::Symbol(chain4_symbol) = &chain5.radix.head else {
            panic!("Task 236 ChainObjectMode4 second intermediate should be a symbol");
        };
        let chain4 = output
            .payload
            .reserve
            .mode_expansions
            .get(chain4_symbol)
            .expect("Task 236 ChainObjectMode4 expansion should exist");
        assert_eq!(chain4.radix.spelling, "ChainObjectMode3");
        assert_ne!(chain4.radix.head, output.asserted_type_input.head);
        let TypeHeadInput::Symbol(chain3_symbol) = &chain4.radix.head else {
            panic!("Task 236 ChainObjectMode3 third intermediate should be a symbol");
        };
        let chain3 = output
            .payload
            .reserve
            .mode_expansions
            .get(chain3_symbol)
            .expect("Task 236 ChainObjectMode3 expansion should exist");
        assert_eq!(chain3.radix.spelling, "ChainObjectMode2");
        assert_ne!(chain3.radix.head, output.asserted_type_input.head);
        let TypeHeadInput::Symbol(chain2_symbol) = &chain3.radix.head else {
            panic!("Task 236 ChainObjectMode2 fourth intermediate should be a symbol");
        };
        let chain2 = output
            .payload
            .reserve
            .mode_expansions
            .get(chain2_symbol)
            .expect("Task 236 ChainObjectMode2 expansion should exist");
        assert_eq!(chain2.radix.spelling, "ChainObjectMode1");
        assert_ne!(chain2.radix.head, output.asserted_type_input.head);
        let TypeHeadInput::Symbol(chain1_symbol) = &chain2.radix.head else {
            panic!("Task 236 ChainObjectMode1 fifth intermediate should be a symbol");
        };
        let chain1 = output
            .payload
            .reserve
            .mode_expansions
            .get(chain1_symbol)
            .expect("Task 236 ChainObjectMode1 expansion should exist");
        assert_eq!(chain1.radix.spelling, "BaseObjectMode");
        assert_eq!(chain1.radix.head, output.asserted_type_input.head);

        type OwnerExtractor = fn(
            &SurfaceAst,
            ResolverModuleId,
            &SymbolEnv,
        ) -> Option<super::SourceReservedVariableTypeAssertion>;
        let prior_extractors: [(&str, OwnerExtractor); 57] = [
            (
                "extract_source_reserved_variable_type_assertion",
                super::extract_source_reserved_variable_type_assertion,
            ),
            (
                "extract_source_reserved_object_variable_type_assertion",
                super::extract_source_reserved_object_variable_type_assertion,
            ),
            (
                "extract_source_local_object_mode_reserved_variable_type_assertion",
                super::extract_source_local_object_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_local_mode_reserved_variable_type_assertion",
                super::extract_source_local_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_local_object_mode_asserted_head",
                super::extract_source_local_object_mode_asserted_head,
            ),
            (
                "extract_source_local_mode_asserted_head",
                super::extract_source_local_mode_asserted_head,
            ),
            (
                "extract_source_chained_local_object_mode_reserved_variable_type_assertion",
                super::extract_source_chained_local_object_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_chained_local_mode_reserved_variable_type_assertion",
                super::extract_source_chained_local_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_chained_local_object_mode_asserted_head",
                super::extract_source_chained_local_object_mode_asserted_head,
            ),
            (
                "extract_source_chained_local_mode_asserted_head",
                super::extract_source_chained_local_mode_asserted_head,
            ),
            (
                "extract_source_chained_local_object_mode_radix_asserted_head",
                super::extract_source_chained_local_object_mode_radix_asserted_head,
            ),
            (
                "extract_source_chained_local_mode_radix_asserted_head",
                super::extract_source_chained_local_mode_radix_asserted_head,
            ),
            (
                "extract_source_two_edge_local_object_mode_reserved_variable_type_assertion",
                super::extract_source_two_edge_local_object_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_two_edge_local_mode_reserved_variable_type_assertion",
                super::extract_source_two_edge_local_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_two_edge_local_object_mode_asserted_head",
                super::extract_source_two_edge_local_object_mode_asserted_head,
            ),
            (
                "extract_source_two_edge_local_mode_asserted_head",
                super::extract_source_two_edge_local_mode_asserted_head,
            ),
            (
                "extract_source_two_edge_local_object_mode_radix_asserted_head",
                super::extract_source_two_edge_local_object_mode_radix_asserted_head,
            ),
            (
                "extract_source_two_edge_local_mode_radix_asserted_head",
                super::extract_source_two_edge_local_mode_radix_asserted_head,
            ),
            (
                "extract_source_three_edge_local_object_mode_reserved_variable_type_assertion",
                super::extract_source_three_edge_local_object_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_three_edge_local_mode_reserved_variable_type_assertion",
                super::extract_source_three_edge_local_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_three_edge_local_object_mode_asserted_head",
                super::extract_source_three_edge_local_object_mode_asserted_head,
            ),
            (
                "extract_source_three_edge_local_mode_asserted_head",
                super::extract_source_three_edge_local_mode_asserted_head,
            ),
            (
                "extract_source_three_edge_local_object_mode_radix_asserted_head",
                super::extract_source_three_edge_local_object_mode_radix_asserted_head,
            ),
            (
                "extract_source_three_edge_local_mode_radix_asserted_head",
                super::extract_source_three_edge_local_mode_radix_asserted_head,
            ),
            (
                "extract_source_four_edge_local_object_mode_reserved_variable_type_assertion",
                super::extract_source_four_edge_local_object_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_four_edge_local_mode_reserved_variable_type_assertion",
                super::extract_source_four_edge_local_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_four_edge_local_object_mode_asserted_head",
                super::extract_source_four_edge_local_object_mode_asserted_head,
            ),
            (
                "extract_source_four_edge_local_mode_asserted_head",
                super::extract_source_four_edge_local_mode_asserted_head,
            ),
            (
                "extract_source_four_edge_local_object_mode_radix_asserted_head",
                super::extract_source_four_edge_local_object_mode_radix_asserted_head,
            ),
            (
                "extract_source_four_edge_local_mode_radix_asserted_head",
                super::extract_source_four_edge_local_mode_radix_asserted_head,
            ),
            (
                "extract_source_local_object_mode_long_chain_reserved_variable_type_assertion",
                super::extract_source_local_object_mode_long_chain_reserved_variable_type_assertion,
            ),
            (
                "extract_source_local_mode_long_chain_reserved_variable_type_assertion",
                super::extract_source_local_mode_long_chain_reserved_variable_type_assertion,
            ),
            (
                "extract_source_local_object_mode_long_chain_asserted_head",
                super::extract_source_local_object_mode_long_chain_asserted_head,
            ),
            (
                "extract_source_local_mode_long_chain_asserted_head",
                super::extract_source_local_mode_long_chain_asserted_head,
            ),
            (
                "extract_source_local_object_mode_long_chain_radix_asserted_head",
                super::extract_source_local_object_mode_long_chain_radix_asserted_head,
            ),
            (
                "extract_source_local_mode_long_chain_radix_asserted_head",
                super::extract_source_local_mode_long_chain_radix_asserted_head,
            ),
            (
                "extract_source_local_object_mode_long_chain_two_hop_asserted_head",
                super::extract_source_local_object_mode_long_chain_two_hop_asserted_head,
            ),
            (
                "extract_source_local_mode_long_chain_two_hop_asserted_head",
                super::extract_source_local_mode_long_chain_two_hop_asserted_head,
            ),
            (
                "extract_source_two_edge_local_object_mode_two_hop_asserted_head",
                super::extract_source_two_edge_local_object_mode_two_hop_asserted_head,
            ),
            (
                "extract_source_two_edge_local_mode_two_hop_asserted_head",
                super::extract_source_two_edge_local_mode_two_hop_asserted_head,
            ),
            (
                "extract_source_three_edge_local_object_mode_two_hop_asserted_head",
                super::extract_source_three_edge_local_object_mode_two_hop_asserted_head,
            ),
            (
                "extract_source_three_edge_local_mode_two_hop_asserted_head",
                super::extract_source_three_edge_local_mode_two_hop_asserted_head,
            ),
            (
                "extract_source_four_edge_local_object_mode_two_hop_asserted_head",
                super::extract_source_four_edge_local_object_mode_two_hop_asserted_head,
            ),
            (
                "extract_source_four_edge_local_mode_two_hop_asserted_head",
                super::extract_source_four_edge_local_mode_two_hop_asserted_head,
            ),
            (
                "extract_source_three_edge_local_object_mode_three_hop_asserted_head",
                super::extract_source_three_edge_local_object_mode_three_hop_asserted_head,
            ),
            (
                "extract_source_three_edge_local_mode_three_hop_asserted_head",
                super::extract_source_three_edge_local_mode_three_hop_asserted_head,
            ),
            (
                "extract_source_four_edge_local_object_mode_three_hop_asserted_head",
                super::extract_source_four_edge_local_object_mode_three_hop_asserted_head,
            ),
            (
                "extract_source_four_edge_local_mode_three_hop_asserted_head",
                super::extract_source_four_edge_local_mode_three_hop_asserted_head,
            ),
            (
                "extract_source_four_edge_local_object_mode_four_hop_asserted_head",
                super::extract_source_four_edge_local_object_mode_four_hop_asserted_head,
            ),
            (
                "extract_source_four_edge_local_mode_four_hop_asserted_head",
                super::extract_source_four_edge_local_mode_four_hop_asserted_head,
            ),
            (
                "extract_source_local_object_mode_long_chain_three_hop_asserted_head",
                super::extract_source_local_object_mode_long_chain_three_hop_asserted_head,
            ),
            (
                "extract_source_local_mode_long_chain_three_hop_asserted_head",
                super::extract_source_local_mode_long_chain_three_hop_asserted_head,
            ),
            (
                "extract_source_local_object_mode_long_chain_four_hop_asserted_head",
                super::extract_source_local_object_mode_long_chain_four_hop_asserted_head,
            ),
            (
                "extract_source_local_mode_long_chain_four_hop_asserted_head",
                super::extract_source_local_mode_long_chain_four_hop_asserted_head,
            ),
            (
                "extract_source_local_mode_long_chain_five_hop_asserted_head",
                super::extract_source_local_mode_long_chain_five_hop_asserted_head,
            ),
            (
                "extract_source_local_object_mode_long_chain_five_hop_asserted_head",
                super::extract_source_local_object_mode_long_chain_five_hop_asserted_head,
            ),
            (
                "extract_source_local_mode_long_chain_six_hop_asserted_head",
                super::extract_source_local_mode_long_chain_six_hop_asserted_head,
            ),
        ];
        for (owner, extractor) in prior_extractors {
            assert!(
                extractor(&ast, resolver.module.clone(), &symbols).is_none(),
                "Task 236 source must not be owned by {owner}"
            );
        }

        let prior_owner_ids = [
            "pass_type_elaboration_reserved_variable_type_assertion_001",
            "pass_type_elaboration_reserved_object_variable_type_assertion_001",
            "pass_type_elaboration_local_object_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_local_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_local_object_mode_asserted_head_001",
            "pass_type_elaboration_local_mode_asserted_head_001",
            "pass_type_elaboration_chained_local_object_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_chained_local_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_chained_local_object_mode_asserted_head_001",
            "pass_type_elaboration_chained_local_mode_asserted_head_001",
            "pass_type_elaboration_chained_local_object_mode_radix_asserted_head_001",
            "pass_type_elaboration_chained_local_mode_radix_asserted_head_001",
            "pass_type_elaboration_two_edge_local_object_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_two_edge_local_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_two_edge_local_object_mode_asserted_head_001",
            "pass_type_elaboration_two_edge_local_mode_asserted_head_001",
            "pass_type_elaboration_two_edge_local_object_mode_radix_asserted_head_001",
            "pass_type_elaboration_two_edge_local_mode_radix_asserted_head_001",
            "pass_type_elaboration_three_edge_local_object_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_three_edge_local_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_three_edge_local_object_mode_asserted_head_001",
            "pass_type_elaboration_three_edge_local_mode_asserted_head_001",
            "pass_type_elaboration_three_edge_local_object_mode_radix_asserted_head_001",
            "pass_type_elaboration_three_edge_local_mode_radix_asserted_head_001",
            "pass_type_elaboration_four_edge_local_object_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_four_edge_local_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_four_edge_local_object_mode_asserted_head_001",
            "pass_type_elaboration_four_edge_local_mode_asserted_head_001",
            "pass_type_elaboration_four_edge_local_object_mode_radix_asserted_head_001",
            "pass_type_elaboration_four_edge_local_mode_radix_asserted_head_001",
            "pass_type_elaboration_local_object_mode_long_chain_reserved_variable_type_assertion_001",
            "pass_type_elaboration_local_mode_long_chain_reserved_variable_type_assertion_001",
            "pass_type_elaboration_local_object_mode_long_chain_asserted_head_001",
            "pass_type_elaboration_local_mode_long_chain_asserted_head_001",
            "pass_type_elaboration_local_object_mode_long_chain_radix_asserted_head_001",
            "pass_type_elaboration_local_mode_long_chain_radix_asserted_head_001",
            "pass_type_elaboration_local_object_mode_long_chain_two_hop_asserted_head_001",
            "pass_type_elaboration_local_mode_long_chain_two_hop_asserted_head_001",
            "pass_type_elaboration_two_edge_local_object_mode_two_hop_asserted_head_001",
            "pass_type_elaboration_two_edge_local_mode_two_hop_asserted_head_001",
            "pass_type_elaboration_three_edge_local_object_mode_two_hop_asserted_head_001",
            "pass_type_elaboration_three_edge_local_mode_two_hop_asserted_head_001",
            "pass_type_elaboration_four_edge_local_object_mode_two_hop_asserted_head_001",
            "pass_type_elaboration_four_edge_local_mode_two_hop_asserted_head_001",
            "pass_type_elaboration_three_edge_local_object_mode_three_hop_asserted_head_001",
            "pass_type_elaboration_three_edge_local_mode_three_hop_asserted_head_001",
            "pass_type_elaboration_four_edge_local_object_mode_three_hop_asserted_head_001",
            "pass_type_elaboration_four_edge_local_mode_three_hop_asserted_head_001",
            "pass_type_elaboration_four_edge_local_object_mode_four_hop_asserted_head_001",
            "pass_type_elaboration_four_edge_local_mode_four_hop_asserted_head_001",
            "pass_type_elaboration_local_object_mode_long_chain_three_hop_asserted_head_001",
            "pass_type_elaboration_local_mode_long_chain_three_hop_asserted_head_001",
            "pass_type_elaboration_local_object_mode_long_chain_four_hop_asserted_head_001",
            "pass_type_elaboration_local_mode_long_chain_four_hop_asserted_head_001",
            "pass_type_elaboration_local_mode_long_chain_five_hop_asserted_head_001",
            "pass_type_elaboration_local_object_mode_long_chain_five_hop_asserted_head_001",
            "pass_type_elaboration_local_mode_long_chain_six_hop_asserted_head_001",
        ];
        assert_eq!(prior_owner_ids.len(), 57);
        for owner_id in prior_owner_ids {
            let (owner_ordinal, owner_case) = active_type_elaboration_cases(&plan)
                .enumerate()
                .find(|(_, case)| case.id.0 == owner_id)
                .unwrap_or_else(|| panic!("prior owner fixture {owner_id} must be active"));
            let owner_frontend = run_frontend(&workspace_root, owner_case, owner_ordinal)
                .unwrap_or_else(|error| panic!("owner fixture {owner_id} must parse: {error}"));
            assert!(owner_frontend.diagnostics.is_empty());
            let owner_ast = owner_frontend
                .ast
                .unwrap_or_else(|| panic!("owner fixture {owner_id} must produce an AST"));
            let owner_resolver =
                resolver_symbol_collection(&workspace_root, owner_case, &owner_ast);
            assert!(owner_resolver.detail_keys.is_empty());
            let owner_symbols = augment_type_elaboration_import_summaries(
                &owner_ast,
                &owner_resolver.module,
                owner_resolver.env,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                ),
                Vec::<String>::new(),
                "prior owner fixture {owner_id} must still reach its active route"
            );
            assert!(
                super::extract_source_local_object_mode_long_chain_six_hop_asserted_head(
                    &owner_ast,
                    owner_resolver.module,
                    &owner_symbols,
                )
                .is_none(),
                "Task 236 must reject prior owner fixture {owner_id}"
            );
        }
    }

    #[test]
    fn active_local_object_mode_long_chain_four_hop_asserted_head_fixture_consumes_seven_expansions()
     {
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("mizar-test crate should live below the workspace root")
            .to_path_buf();
        let config = DiscoveryConfig {
            workspace_root: workspace_root.clone(),
            tests_root: workspace_root.join("tests"),
            manifest_path: workspace_root.join("tests/coverage/spec_trace.toml"),
            profile: TestProfile::Fast,
            validation_mode: ValidationMode::Metadata,
        };
        let plan = build_test_plan(&config).expect("Task 229 repository plan should build");
        let (ordinal, case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| {
                case.id.0
                    == "pass_type_elaboration_local_object_mode_long_chain_four_hop_asserted_head_001"
            })
            .expect("Task 229 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 229 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 229 fixture should produce a real AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = super::source_local_object_mode_long_chain_four_hop_asserted_head_output(
            &ast,
            resolver.module.clone(),
            &symbols,
        )
        .expect("Task 229 real AST should reach the seven-expansion four-hop seam");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 229 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.subject_result_input.spelling, "ChainObjectMode6");
        assert_eq!(output.asserted_type_input.spelling, "ChainObjectMode2");
        assert_ne!(
            output.subject_result_input.head,
            output.asserted_type_input.head
        );
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert!(output.term_formula.candidate_sets().is_empty());
        assert!(output.term_formula.facts().is_empty());
        assert!(output.term_formula.diagnostics().is_empty());
        let chain6 = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("ChainObjectMode6"))
            .map(|(_, expansion)| expansion)
            .expect("Task 229 ChainObjectMode6 expansion should exist");
        assert_eq!(chain6.radix.spelling, "ChainObjectMode5");
        let TypeHeadInput::Symbol(chain5_symbol) = &chain6.radix.head else {
            panic!("Task 229 ChainObjectMode5 first intermediate should be a symbol");
        };
        let chain5 = output
            .payload
            .reserve
            .mode_expansions
            .get(chain5_symbol)
            .expect("Task 229 ChainObjectMode5 expansion should exist");
        assert_eq!(chain5.radix.spelling, "ChainObjectMode4");
        assert_ne!(chain5.radix.head, output.asserted_type_input.head);
        let TypeHeadInput::Symbol(chain4_symbol) = &chain5.radix.head else {
            panic!("Task 229 ChainObjectMode4 second intermediate should be a symbol");
        };
        let chain4 = output
            .payload
            .reserve
            .mode_expansions
            .get(chain4_symbol)
            .expect("Task 229 ChainObjectMode4 expansion should exist");
        assert_eq!(chain4.radix.spelling, "ChainObjectMode3");
        assert_ne!(chain4.radix.head, output.asserted_type_input.head);
        let TypeHeadInput::Symbol(chain3_symbol) = &chain4.radix.head else {
            panic!("Task 229 ChainObjectMode3 third intermediate should be a symbol");
        };
        let chain3 = output
            .payload
            .reserve
            .mode_expansions
            .get(chain3_symbol)
            .expect("Task 229 ChainObjectMode3 expansion should exist");
        assert_eq!(chain3.radix.spelling, "ChainObjectMode2");
        assert_eq!(chain3.radix.head, output.asserted_type_input.head);

        type OwnerExtractor = fn(
            &SurfaceAst,
            ResolverModuleId,
            &SymbolEnv,
        ) -> Option<super::SourceReservedVariableTypeAssertion>;
        let prior_extractors: [(&str, OwnerExtractor); 53] = [
            (
                "extract_source_reserved_variable_type_assertion",
                super::extract_source_reserved_variable_type_assertion,
            ),
            (
                "extract_source_reserved_object_variable_type_assertion",
                super::extract_source_reserved_object_variable_type_assertion,
            ),
            (
                "extract_source_local_mode_reserved_variable_type_assertion",
                super::extract_source_local_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_local_object_mode_reserved_variable_type_assertion",
                super::extract_source_local_object_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_local_mode_asserted_head",
                super::extract_source_local_mode_asserted_head,
            ),
            (
                "extract_source_local_object_mode_asserted_head",
                super::extract_source_local_object_mode_asserted_head,
            ),
            (
                "extract_source_chained_local_mode_reserved_variable_type_assertion",
                super::extract_source_chained_local_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_chained_local_object_mode_reserved_variable_type_assertion",
                super::extract_source_chained_local_object_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_chained_local_mode_asserted_head",
                super::extract_source_chained_local_mode_asserted_head,
            ),
            (
                "extract_source_chained_local_object_mode_asserted_head",
                super::extract_source_chained_local_object_mode_asserted_head,
            ),
            (
                "extract_source_chained_local_mode_radix_asserted_head",
                super::extract_source_chained_local_mode_radix_asserted_head,
            ),
            (
                "extract_source_chained_local_object_mode_radix_asserted_head",
                super::extract_source_chained_local_object_mode_radix_asserted_head,
            ),
            (
                "extract_source_two_edge_local_mode_reserved_variable_type_assertion",
                super::extract_source_two_edge_local_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_two_edge_local_object_mode_reserved_variable_type_assertion",
                super::extract_source_two_edge_local_object_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_two_edge_local_mode_asserted_head",
                super::extract_source_two_edge_local_mode_asserted_head,
            ),
            (
                "extract_source_two_edge_local_object_mode_asserted_head",
                super::extract_source_two_edge_local_object_mode_asserted_head,
            ),
            (
                "extract_source_two_edge_local_mode_radix_asserted_head",
                super::extract_source_two_edge_local_mode_radix_asserted_head,
            ),
            (
                "extract_source_two_edge_local_object_mode_radix_asserted_head",
                super::extract_source_two_edge_local_object_mode_radix_asserted_head,
            ),
            (
                "extract_source_three_edge_local_mode_reserved_variable_type_assertion",
                super::extract_source_three_edge_local_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_three_edge_local_object_mode_reserved_variable_type_assertion",
                super::extract_source_three_edge_local_object_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_three_edge_local_mode_asserted_head",
                super::extract_source_three_edge_local_mode_asserted_head,
            ),
            (
                "extract_source_three_edge_local_object_mode_asserted_head",
                super::extract_source_three_edge_local_object_mode_asserted_head,
            ),
            (
                "extract_source_three_edge_local_mode_radix_asserted_head",
                super::extract_source_three_edge_local_mode_radix_asserted_head,
            ),
            (
                "extract_source_three_edge_local_object_mode_radix_asserted_head",
                super::extract_source_three_edge_local_object_mode_radix_asserted_head,
            ),
            (
                "extract_source_four_edge_local_mode_reserved_variable_type_assertion",
                super::extract_source_four_edge_local_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_four_edge_local_object_mode_reserved_variable_type_assertion",
                super::extract_source_four_edge_local_object_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_four_edge_local_mode_asserted_head",
                super::extract_source_four_edge_local_mode_asserted_head,
            ),
            (
                "extract_source_four_edge_local_object_mode_asserted_head",
                super::extract_source_four_edge_local_object_mode_asserted_head,
            ),
            (
                "extract_source_four_edge_local_mode_radix_asserted_head",
                super::extract_source_four_edge_local_mode_radix_asserted_head,
            ),
            (
                "extract_source_four_edge_local_object_mode_radix_asserted_head",
                super::extract_source_four_edge_local_object_mode_radix_asserted_head,
            ),
            (
                "extract_source_local_mode_long_chain_reserved_variable_type_assertion",
                super::extract_source_local_mode_long_chain_reserved_variable_type_assertion,
            ),
            (
                "extract_source_local_object_mode_long_chain_reserved_variable_type_assertion",
                super::extract_source_local_object_mode_long_chain_reserved_variable_type_assertion,
            ),
            (
                "extract_source_local_mode_long_chain_asserted_head",
                super::extract_source_local_mode_long_chain_asserted_head,
            ),
            (
                "extract_source_local_object_mode_long_chain_asserted_head",
                super::extract_source_local_object_mode_long_chain_asserted_head,
            ),
            (
                "extract_source_local_mode_long_chain_radix_asserted_head",
                super::extract_source_local_mode_long_chain_radix_asserted_head,
            ),
            (
                "extract_source_local_object_mode_long_chain_radix_asserted_head",
                super::extract_source_local_object_mode_long_chain_radix_asserted_head,
            ),
            (
                "extract_source_local_mode_long_chain_two_hop_asserted_head",
                super::extract_source_local_mode_long_chain_two_hop_asserted_head,
            ),
            (
                "extract_source_local_object_mode_long_chain_two_hop_asserted_head",
                super::extract_source_local_object_mode_long_chain_two_hop_asserted_head,
            ),
            (
                "extract_source_two_edge_local_mode_two_hop_asserted_head",
                super::extract_source_two_edge_local_mode_two_hop_asserted_head,
            ),
            (
                "extract_source_two_edge_local_object_mode_two_hop_asserted_head",
                super::extract_source_two_edge_local_object_mode_two_hop_asserted_head,
            ),
            (
                "extract_source_three_edge_local_mode_two_hop_asserted_head",
                super::extract_source_three_edge_local_mode_two_hop_asserted_head,
            ),
            (
                "extract_source_three_edge_local_object_mode_two_hop_asserted_head",
                super::extract_source_three_edge_local_object_mode_two_hop_asserted_head,
            ),
            (
                "extract_source_four_edge_local_mode_two_hop_asserted_head",
                super::extract_source_four_edge_local_mode_two_hop_asserted_head,
            ),
            (
                "extract_source_four_edge_local_object_mode_two_hop_asserted_head",
                super::extract_source_four_edge_local_object_mode_two_hop_asserted_head,
            ),
            (
                "extract_source_three_edge_local_mode_three_hop_asserted_head",
                super::extract_source_three_edge_local_mode_three_hop_asserted_head,
            ),
            (
                "extract_source_three_edge_local_object_mode_three_hop_asserted_head",
                super::extract_source_three_edge_local_object_mode_three_hop_asserted_head,
            ),
            (
                "extract_source_four_edge_local_mode_three_hop_asserted_head",
                super::extract_source_four_edge_local_mode_three_hop_asserted_head,
            ),
            (
                "extract_source_four_edge_local_object_mode_three_hop_asserted_head",
                super::extract_source_four_edge_local_object_mode_three_hop_asserted_head,
            ),
            (
                "extract_source_four_edge_local_mode_four_hop_asserted_head",
                super::extract_source_four_edge_local_mode_four_hop_asserted_head,
            ),
            (
                "extract_source_four_edge_local_object_mode_four_hop_asserted_head",
                super::extract_source_four_edge_local_object_mode_four_hop_asserted_head,
            ),
            (
                "extract_source_local_mode_long_chain_three_hop_asserted_head",
                super::extract_source_local_mode_long_chain_three_hop_asserted_head,
            ),
            (
                "extract_source_local_object_mode_long_chain_three_hop_asserted_head",
                super::extract_source_local_object_mode_long_chain_three_hop_asserted_head,
            ),
            (
                "extract_source_local_mode_long_chain_four_hop_asserted_head",
                super::extract_source_local_mode_long_chain_four_hop_asserted_head,
            ),
        ];
        for (owner, extractor) in prior_extractors {
            assert!(
                extractor(&ast, resolver.module.clone(), &symbols).is_none(),
                "Task 229 source must not be owned by {owner}"
            );
        }

        let prior_owner_ids = [
            "pass_type_elaboration_reserved_variable_type_assertion_001",
            "pass_type_elaboration_reserved_object_variable_type_assertion_001",
            "pass_type_elaboration_local_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_local_object_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_local_mode_asserted_head_001",
            "pass_type_elaboration_local_object_mode_asserted_head_001",
            "pass_type_elaboration_chained_local_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_chained_local_object_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_chained_local_mode_asserted_head_001",
            "pass_type_elaboration_chained_local_object_mode_asserted_head_001",
            "pass_type_elaboration_chained_local_mode_radix_asserted_head_001",
            "pass_type_elaboration_chained_local_object_mode_radix_asserted_head_001",
            "pass_type_elaboration_two_edge_local_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_two_edge_local_object_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_two_edge_local_mode_asserted_head_001",
            "pass_type_elaboration_two_edge_local_object_mode_asserted_head_001",
            "pass_type_elaboration_two_edge_local_mode_radix_asserted_head_001",
            "pass_type_elaboration_two_edge_local_object_mode_radix_asserted_head_001",
            "pass_type_elaboration_three_edge_local_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_three_edge_local_object_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_three_edge_local_mode_asserted_head_001",
            "pass_type_elaboration_three_edge_local_object_mode_asserted_head_001",
            "pass_type_elaboration_three_edge_local_mode_radix_asserted_head_001",
            "pass_type_elaboration_three_edge_local_object_mode_radix_asserted_head_001",
            "pass_type_elaboration_four_edge_local_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_four_edge_local_object_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_four_edge_local_mode_asserted_head_001",
            "pass_type_elaboration_four_edge_local_object_mode_asserted_head_001",
            "pass_type_elaboration_four_edge_local_mode_radix_asserted_head_001",
            "pass_type_elaboration_four_edge_local_object_mode_radix_asserted_head_001",
            "pass_type_elaboration_local_mode_long_chain_reserved_variable_type_assertion_001",
            "pass_type_elaboration_local_object_mode_long_chain_reserved_variable_type_assertion_001",
            "pass_type_elaboration_local_mode_long_chain_asserted_head_001",
            "pass_type_elaboration_local_object_mode_long_chain_asserted_head_001",
            "pass_type_elaboration_local_mode_long_chain_radix_asserted_head_001",
            "pass_type_elaboration_local_object_mode_long_chain_radix_asserted_head_001",
            "pass_type_elaboration_local_mode_long_chain_two_hop_asserted_head_001",
            "pass_type_elaboration_local_object_mode_long_chain_two_hop_asserted_head_001",
            "pass_type_elaboration_two_edge_local_mode_two_hop_asserted_head_001",
            "pass_type_elaboration_two_edge_local_object_mode_two_hop_asserted_head_001",
            "pass_type_elaboration_three_edge_local_mode_two_hop_asserted_head_001",
            "pass_type_elaboration_three_edge_local_object_mode_two_hop_asserted_head_001",
            "pass_type_elaboration_four_edge_local_mode_two_hop_asserted_head_001",
            "pass_type_elaboration_four_edge_local_object_mode_two_hop_asserted_head_001",
            "pass_type_elaboration_three_edge_local_mode_three_hop_asserted_head_001",
            "pass_type_elaboration_three_edge_local_object_mode_three_hop_asserted_head_001",
            "pass_type_elaboration_four_edge_local_mode_three_hop_asserted_head_001",
            "pass_type_elaboration_four_edge_local_object_mode_three_hop_asserted_head_001",
            "pass_type_elaboration_four_edge_local_mode_four_hop_asserted_head_001",
            "pass_type_elaboration_four_edge_local_object_mode_four_hop_asserted_head_001",
            "pass_type_elaboration_local_mode_long_chain_three_hop_asserted_head_001",
            "pass_type_elaboration_local_object_mode_long_chain_three_hop_asserted_head_001",
            "pass_type_elaboration_local_mode_long_chain_four_hop_asserted_head_001",
        ];
        assert_eq!(prior_owner_ids.len(), 53);
        for owner_id in prior_owner_ids {
            let (owner_ordinal, owner_case) = active_type_elaboration_cases(&plan)
                .enumerate()
                .find(|(_, case)| case.id.0 == owner_id)
                .unwrap_or_else(|| panic!("prior owner fixture {owner_id} must be active"));
            let owner_frontend = run_frontend(&workspace_root, owner_case, owner_ordinal)
                .unwrap_or_else(|error| panic!("owner fixture {owner_id} must parse: {error}"));
            assert!(owner_frontend.diagnostics.is_empty());
            let owner_ast = owner_frontend
                .ast
                .unwrap_or_else(|| panic!("owner fixture {owner_id} must produce an AST"));
            let owner_resolver =
                resolver_symbol_collection(&workspace_root, owner_case, &owner_ast);
            assert!(owner_resolver.detail_keys.is_empty());
            let owner_symbols = augment_type_elaboration_import_summaries(
                &owner_ast,
                &owner_resolver.module,
                owner_resolver.env,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                ),
                Vec::<String>::new(),
                "prior owner fixture {owner_id} must still reach its active route"
            );
            assert!(
                super::extract_source_local_object_mode_long_chain_four_hop_asserted_head(
                    &owner_ast,
                    owner_resolver.module,
                    &owner_symbols,
                )
                .is_none(),
                "Task 229 must reject prior owner fixture {owner_id}"
            );
        }
    }

    #[test]
    fn active_local_object_mode_long_chain_three_hop_asserted_head_fixture_consumes_seven_expansions()
     {
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("mizar-test crate should live below the workspace root")
            .to_path_buf();
        let config = DiscoveryConfig {
            workspace_root: workspace_root.clone(),
            tests_root: workspace_root.join("tests"),
            manifest_path: workspace_root.join("tests/coverage/spec_trace.toml"),
            profile: TestProfile::Fast,
            validation_mode: ValidationMode::Metadata,
        };
        let plan = build_test_plan(&config).expect("Task 227 repository plan should build");
        let (ordinal, case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| {
                case.id.0 == "pass_type_elaboration_local_object_mode_long_chain_three_hop_asserted_head_001"
            })
            .expect("Task 227 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 227 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 227 fixture should produce a real AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = super::source_local_object_mode_long_chain_three_hop_asserted_head_output(
            &ast,
            resolver.module.clone(),
            &symbols,
        )
        .expect("Task 227 real AST should reach the seven-expansion object three-hop seam");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 227 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.subject_result_input.spelling, "ChainObjectMode6");
        assert_eq!(output.asserted_type_input.spelling, "ChainObjectMode3");
        assert_ne!(
            output.subject_result_input.head,
            output.asserted_type_input.head
        );
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert!(output.term_formula.candidate_sets().is_empty());
        assert!(output.term_formula.facts().is_empty());
        assert!(output.term_formula.diagnostics().is_empty());
        let chain6 = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("ChainObjectMode6"))
            .map(|(_, expansion)| expansion)
            .expect("Task 227 ChainObjectMode6 expansion should exist");
        assert_eq!(chain6.radix.spelling, "ChainObjectMode5");
        let TypeHeadInput::Symbol(chain5_symbol) = &chain6.radix.head else {
            panic!("Task 227 ChainObjectMode5 first intermediate should be a symbol");
        };
        let chain5 = output
            .payload
            .reserve
            .mode_expansions
            .get(chain5_symbol)
            .expect("Task 227 ChainObjectMode5 expansion should exist");
        assert_eq!(chain5.radix.spelling, "ChainObjectMode4");
        assert_ne!(chain5.radix.head, output.asserted_type_input.head);
        let TypeHeadInput::Symbol(chain4_symbol) = &chain5.radix.head else {
            panic!("Task 227 ChainObjectMode4 second intermediate should be a symbol");
        };
        let chain4 = output
            .payload
            .reserve
            .mode_expansions
            .get(chain4_symbol)
            .expect("Task 227 ChainObjectMode4 expansion should exist");
        assert_eq!(chain4.radix.spelling, "ChainObjectMode3");
        assert_eq!(chain4.radix.head, output.asserted_type_input.head);

        type OwnerExtractor = fn(
            &SurfaceAst,
            ResolverModuleId,
            &SymbolEnv,
        ) -> Option<super::SourceReservedVariableTypeAssertion>;
        let prior_extractors: [(&str, OwnerExtractor); 51] = [
            (
                "extract_source_reserved_variable_type_assertion",
                super::extract_source_reserved_variable_type_assertion,
            ),
            (
                "extract_source_reserved_object_variable_type_assertion",
                super::extract_source_reserved_object_variable_type_assertion,
            ),
            (
                "extract_source_local_mode_reserved_variable_type_assertion",
                super::extract_source_local_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_local_object_mode_reserved_variable_type_assertion",
                super::extract_source_local_object_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_local_mode_asserted_head",
                super::extract_source_local_mode_asserted_head,
            ),
            (
                "extract_source_local_object_mode_asserted_head",
                super::extract_source_local_object_mode_asserted_head,
            ),
            (
                "extract_source_chained_local_mode_reserved_variable_type_assertion",
                super::extract_source_chained_local_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_chained_local_object_mode_reserved_variable_type_assertion",
                super::extract_source_chained_local_object_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_chained_local_mode_asserted_head",
                super::extract_source_chained_local_mode_asserted_head,
            ),
            (
                "extract_source_chained_local_object_mode_asserted_head",
                super::extract_source_chained_local_object_mode_asserted_head,
            ),
            (
                "extract_source_chained_local_mode_radix_asserted_head",
                super::extract_source_chained_local_mode_radix_asserted_head,
            ),
            (
                "extract_source_chained_local_object_mode_radix_asserted_head",
                super::extract_source_chained_local_object_mode_radix_asserted_head,
            ),
            (
                "extract_source_two_edge_local_mode_reserved_variable_type_assertion",
                super::extract_source_two_edge_local_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_two_edge_local_object_mode_reserved_variable_type_assertion",
                super::extract_source_two_edge_local_object_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_two_edge_local_mode_asserted_head",
                super::extract_source_two_edge_local_mode_asserted_head,
            ),
            (
                "extract_source_two_edge_local_object_mode_asserted_head",
                super::extract_source_two_edge_local_object_mode_asserted_head,
            ),
            (
                "extract_source_two_edge_local_mode_radix_asserted_head",
                super::extract_source_two_edge_local_mode_radix_asserted_head,
            ),
            (
                "extract_source_two_edge_local_object_mode_radix_asserted_head",
                super::extract_source_two_edge_local_object_mode_radix_asserted_head,
            ),
            (
                "extract_source_three_edge_local_mode_reserved_variable_type_assertion",
                super::extract_source_three_edge_local_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_three_edge_local_object_mode_reserved_variable_type_assertion",
                super::extract_source_three_edge_local_object_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_three_edge_local_mode_asserted_head",
                super::extract_source_three_edge_local_mode_asserted_head,
            ),
            (
                "extract_source_three_edge_local_object_mode_asserted_head",
                super::extract_source_three_edge_local_object_mode_asserted_head,
            ),
            (
                "extract_source_three_edge_local_mode_radix_asserted_head",
                super::extract_source_three_edge_local_mode_radix_asserted_head,
            ),
            (
                "extract_source_three_edge_local_object_mode_radix_asserted_head",
                super::extract_source_three_edge_local_object_mode_radix_asserted_head,
            ),
            (
                "extract_source_four_edge_local_mode_reserved_variable_type_assertion",
                super::extract_source_four_edge_local_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_four_edge_local_object_mode_reserved_variable_type_assertion",
                super::extract_source_four_edge_local_object_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_four_edge_local_mode_asserted_head",
                super::extract_source_four_edge_local_mode_asserted_head,
            ),
            (
                "extract_source_four_edge_local_object_mode_asserted_head",
                super::extract_source_four_edge_local_object_mode_asserted_head,
            ),
            (
                "extract_source_four_edge_local_mode_radix_asserted_head",
                super::extract_source_four_edge_local_mode_radix_asserted_head,
            ),
            (
                "extract_source_four_edge_local_object_mode_radix_asserted_head",
                super::extract_source_four_edge_local_object_mode_radix_asserted_head,
            ),
            (
                "extract_source_local_mode_long_chain_reserved_variable_type_assertion",
                super::extract_source_local_mode_long_chain_reserved_variable_type_assertion,
            ),
            (
                "extract_source_local_object_mode_long_chain_reserved_variable_type_assertion",
                super::extract_source_local_object_mode_long_chain_reserved_variable_type_assertion,
            ),
            (
                "extract_source_local_mode_long_chain_asserted_head",
                super::extract_source_local_mode_long_chain_asserted_head,
            ),
            (
                "extract_source_local_object_mode_long_chain_asserted_head",
                super::extract_source_local_object_mode_long_chain_asserted_head,
            ),
            (
                "extract_source_local_mode_long_chain_radix_asserted_head",
                super::extract_source_local_mode_long_chain_radix_asserted_head,
            ),
            (
                "extract_source_local_object_mode_long_chain_radix_asserted_head",
                super::extract_source_local_object_mode_long_chain_radix_asserted_head,
            ),
            (
                "extract_source_local_mode_long_chain_two_hop_asserted_head",
                super::extract_source_local_mode_long_chain_two_hop_asserted_head,
            ),
            (
                "extract_source_local_object_mode_long_chain_two_hop_asserted_head",
                super::extract_source_local_object_mode_long_chain_two_hop_asserted_head,
            ),
            (
                "extract_source_two_edge_local_mode_two_hop_asserted_head",
                super::extract_source_two_edge_local_mode_two_hop_asserted_head,
            ),
            (
                "extract_source_two_edge_local_object_mode_two_hop_asserted_head",
                super::extract_source_two_edge_local_object_mode_two_hop_asserted_head,
            ),
            (
                "extract_source_three_edge_local_mode_two_hop_asserted_head",
                super::extract_source_three_edge_local_mode_two_hop_asserted_head,
            ),
            (
                "extract_source_three_edge_local_object_mode_two_hop_asserted_head",
                super::extract_source_three_edge_local_object_mode_two_hop_asserted_head,
            ),
            (
                "extract_source_four_edge_local_mode_two_hop_asserted_head",
                super::extract_source_four_edge_local_mode_two_hop_asserted_head,
            ),
            (
                "extract_source_four_edge_local_object_mode_two_hop_asserted_head",
                super::extract_source_four_edge_local_object_mode_two_hop_asserted_head,
            ),
            (
                "extract_source_three_edge_local_mode_three_hop_asserted_head",
                super::extract_source_three_edge_local_mode_three_hop_asserted_head,
            ),
            (
                "extract_source_three_edge_local_object_mode_three_hop_asserted_head",
                super::extract_source_three_edge_local_object_mode_three_hop_asserted_head,
            ),
            (
                "extract_source_four_edge_local_mode_three_hop_asserted_head",
                super::extract_source_four_edge_local_mode_three_hop_asserted_head,
            ),
            (
                "extract_source_four_edge_local_object_mode_three_hop_asserted_head",
                super::extract_source_four_edge_local_object_mode_three_hop_asserted_head,
            ),
            (
                "extract_source_four_edge_local_mode_four_hop_asserted_head",
                super::extract_source_four_edge_local_mode_four_hop_asserted_head,
            ),
            (
                "extract_source_four_edge_local_object_mode_four_hop_asserted_head",
                super::extract_source_four_edge_local_object_mode_four_hop_asserted_head,
            ),
            (
                "extract_source_local_mode_long_chain_three_hop_asserted_head",
                super::extract_source_local_mode_long_chain_three_hop_asserted_head,
            ),
        ];
        for (owner, extractor) in prior_extractors {
            assert!(
                extractor(&ast, resolver.module.clone(), &symbols).is_none(),
                "Task 227 source must not be owned by {owner}"
            );
        }

        let prior_owner_ids = [
            "pass_type_elaboration_reserved_variable_type_assertion_001",
            "pass_type_elaboration_reserved_object_variable_type_assertion_001",
            "pass_type_elaboration_local_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_local_object_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_local_mode_asserted_head_001",
            "pass_type_elaboration_local_object_mode_asserted_head_001",
            "pass_type_elaboration_chained_local_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_chained_local_object_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_chained_local_mode_asserted_head_001",
            "pass_type_elaboration_chained_local_object_mode_asserted_head_001",
            "pass_type_elaboration_chained_local_mode_radix_asserted_head_001",
            "pass_type_elaboration_chained_local_object_mode_radix_asserted_head_001",
            "pass_type_elaboration_two_edge_local_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_two_edge_local_object_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_two_edge_local_mode_asserted_head_001",
            "pass_type_elaboration_two_edge_local_object_mode_asserted_head_001",
            "pass_type_elaboration_two_edge_local_mode_radix_asserted_head_001",
            "pass_type_elaboration_two_edge_local_object_mode_radix_asserted_head_001",
            "pass_type_elaboration_three_edge_local_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_three_edge_local_object_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_three_edge_local_mode_asserted_head_001",
            "pass_type_elaboration_three_edge_local_object_mode_asserted_head_001",
            "pass_type_elaboration_three_edge_local_mode_radix_asserted_head_001",
            "pass_type_elaboration_three_edge_local_object_mode_radix_asserted_head_001",
            "pass_type_elaboration_four_edge_local_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_four_edge_local_object_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_four_edge_local_mode_asserted_head_001",
            "pass_type_elaboration_four_edge_local_object_mode_asserted_head_001",
            "pass_type_elaboration_four_edge_local_mode_radix_asserted_head_001",
            "pass_type_elaboration_four_edge_local_object_mode_radix_asserted_head_001",
            "pass_type_elaboration_local_mode_long_chain_reserved_variable_type_assertion_001",
            "pass_type_elaboration_local_object_mode_long_chain_reserved_variable_type_assertion_001",
            "pass_type_elaboration_local_mode_long_chain_asserted_head_001",
            "pass_type_elaboration_local_object_mode_long_chain_asserted_head_001",
            "pass_type_elaboration_local_mode_long_chain_radix_asserted_head_001",
            "pass_type_elaboration_local_object_mode_long_chain_radix_asserted_head_001",
            "pass_type_elaboration_local_mode_long_chain_two_hop_asserted_head_001",
            "pass_type_elaboration_local_object_mode_long_chain_two_hop_asserted_head_001",
            "pass_type_elaboration_two_edge_local_mode_two_hop_asserted_head_001",
            "pass_type_elaboration_two_edge_local_object_mode_two_hop_asserted_head_001",
            "pass_type_elaboration_three_edge_local_mode_two_hop_asserted_head_001",
            "pass_type_elaboration_three_edge_local_object_mode_two_hop_asserted_head_001",
            "pass_type_elaboration_four_edge_local_mode_two_hop_asserted_head_001",
            "pass_type_elaboration_four_edge_local_object_mode_two_hop_asserted_head_001",
            "pass_type_elaboration_three_edge_local_mode_three_hop_asserted_head_001",
            "pass_type_elaboration_three_edge_local_object_mode_three_hop_asserted_head_001",
            "pass_type_elaboration_four_edge_local_mode_three_hop_asserted_head_001",
            "pass_type_elaboration_four_edge_local_object_mode_three_hop_asserted_head_001",
            "pass_type_elaboration_four_edge_local_mode_four_hop_asserted_head_001",
            "pass_type_elaboration_four_edge_local_object_mode_four_hop_asserted_head_001",
            "pass_type_elaboration_local_mode_long_chain_three_hop_asserted_head_001",
        ];
        assert_eq!(prior_owner_ids.len(), 51);
        for owner_id in prior_owner_ids {
            let (owner_ordinal, owner_case) = active_type_elaboration_cases(&plan)
                .enumerate()
                .find(|(_, case)| case.id.0 == owner_id)
                .unwrap_or_else(|| panic!("prior owner fixture {owner_id} must be active"));
            let owner_frontend = run_frontend(&workspace_root, owner_case, owner_ordinal)
                .unwrap_or_else(|error| panic!("owner fixture {owner_id} must parse: {error}"));
            assert!(owner_frontend.diagnostics.is_empty());
            let owner_ast = owner_frontend
                .ast
                .unwrap_or_else(|| panic!("owner fixture {owner_id} must produce an AST"));
            let owner_resolver =
                resolver_symbol_collection(&workspace_root, owner_case, &owner_ast);
            assert!(owner_resolver.detail_keys.is_empty());
            let owner_symbols = augment_type_elaboration_import_summaries(
                &owner_ast,
                &owner_resolver.module,
                owner_resolver.env,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                ),
                Vec::<String>::new(),
                "prior owner fixture {owner_id} must still reach its active route"
            );
            assert!(
                super::extract_source_local_object_mode_long_chain_three_hop_asserted_head(
                    &owner_ast,
                    owner_resolver.module,
                    &owner_symbols,
                )
                .is_none(),
                "Task 227 must reject prior owner fixture {owner_id}"
            );
        }
    }

    #[test]
    fn active_local_mode_long_chain_two_hop_asserted_head_fixture_consumes_seven_expansions() {
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("mizar-test crate should live below the workspace root")
            .to_path_buf();
        let config = DiscoveryConfig {
            workspace_root: workspace_root.clone(),
            tests_root: workspace_root.join("tests"),
            manifest_path: workspace_root.join("tests/coverage/spec_trace.toml"),
            profile: TestProfile::Fast,
            validation_mode: ValidationMode::Metadata,
        };
        let plan = build_test_plan(&config).expect("Task 224 repository plan should build");
        let (ordinal, case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| {
                case.id.0 == "pass_type_elaboration_local_mode_long_chain_two_hop_asserted_head_001"
            })
            .expect("Task 224 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 224 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 224 fixture should produce a real AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = super::source_local_mode_long_chain_two_hop_asserted_head_output(
            &ast,
            resolver.module.clone(),
            &symbols,
        )
        .expect("Task 224 real AST should reach the seven-expansion two-hop seam");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 224 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.subject_result_input.spelling, "ChainMode6");
        assert_eq!(output.asserted_type_input.spelling, "ChainMode4");
        assert_ne!(
            output.subject_result_input.head,
            output.asserted_type_input.head
        );
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert!(output.term_formula.candidate_sets().is_empty());
        assert!(output.term_formula.facts().is_empty());
        assert!(output.term_formula.diagnostics().is_empty());
        let chain6 = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("ChainMode6"))
            .map(|(_, expansion)| expansion)
            .expect("Task 224 ChainMode6 expansion should exist");
        assert_eq!(chain6.radix.spelling, "ChainMode5");
        let TypeHeadInput::Symbol(chain5_symbol) = &chain6.radix.head else {
            panic!("Task 224 ChainMode5 intermediate should be a symbol");
        };
        let chain5 = output
            .payload
            .reserve
            .mode_expansions
            .get(chain5_symbol)
            .expect("Task 224 ChainMode5 expansion should exist");
        assert_eq!(chain5.radix.spelling, "ChainMode4");
        assert_eq!(chain5.radix.head, output.asserted_type_input.head);

        type OwnerExtractor = fn(
            &SurfaceAst,
            ResolverModuleId,
            &SymbolEnv,
        ) -> Option<super::SourceReservedVariableTypeAssertion>;
        let prior_extractors: [(&str, OwnerExtractor); 48] = [
            (
                "extract_source_reserved_variable_type_assertion",
                super::extract_source_reserved_variable_type_assertion,
            ),
            (
                "extract_source_reserved_object_variable_type_assertion",
                super::extract_source_reserved_object_variable_type_assertion,
            ),
            (
                "extract_source_local_mode_reserved_variable_type_assertion",
                super::extract_source_local_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_local_object_mode_reserved_variable_type_assertion",
                super::extract_source_local_object_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_local_mode_asserted_head",
                super::extract_source_local_mode_asserted_head,
            ),
            (
                "extract_source_local_object_mode_asserted_head",
                super::extract_source_local_object_mode_asserted_head,
            ),
            (
                "extract_source_chained_local_mode_reserved_variable_type_assertion",
                super::extract_source_chained_local_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_chained_local_object_mode_reserved_variable_type_assertion",
                super::extract_source_chained_local_object_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_chained_local_mode_asserted_head",
                super::extract_source_chained_local_mode_asserted_head,
            ),
            (
                "extract_source_chained_local_object_mode_asserted_head",
                super::extract_source_chained_local_object_mode_asserted_head,
            ),
            (
                "extract_source_chained_local_mode_radix_asserted_head",
                super::extract_source_chained_local_mode_radix_asserted_head,
            ),
            (
                "extract_source_chained_local_object_mode_radix_asserted_head",
                super::extract_source_chained_local_object_mode_radix_asserted_head,
            ),
            (
                "extract_source_two_edge_local_mode_reserved_variable_type_assertion",
                super::extract_source_two_edge_local_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_two_edge_local_object_mode_reserved_variable_type_assertion",
                super::extract_source_two_edge_local_object_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_two_edge_local_mode_asserted_head",
                super::extract_source_two_edge_local_mode_asserted_head,
            ),
            (
                "extract_source_two_edge_local_object_mode_asserted_head",
                super::extract_source_two_edge_local_object_mode_asserted_head,
            ),
            (
                "extract_source_two_edge_local_mode_radix_asserted_head",
                super::extract_source_two_edge_local_mode_radix_asserted_head,
            ),
            (
                "extract_source_two_edge_local_object_mode_radix_asserted_head",
                super::extract_source_two_edge_local_object_mode_radix_asserted_head,
            ),
            (
                "extract_source_three_edge_local_mode_reserved_variable_type_assertion",
                super::extract_source_three_edge_local_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_three_edge_local_object_mode_reserved_variable_type_assertion",
                super::extract_source_three_edge_local_object_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_three_edge_local_mode_asserted_head",
                super::extract_source_three_edge_local_mode_asserted_head,
            ),
            (
                "extract_source_three_edge_local_object_mode_asserted_head",
                super::extract_source_three_edge_local_object_mode_asserted_head,
            ),
            (
                "extract_source_three_edge_local_mode_radix_asserted_head",
                super::extract_source_three_edge_local_mode_radix_asserted_head,
            ),
            (
                "extract_source_three_edge_local_object_mode_radix_asserted_head",
                super::extract_source_three_edge_local_object_mode_radix_asserted_head,
            ),
            (
                "extract_source_four_edge_local_mode_reserved_variable_type_assertion",
                super::extract_source_four_edge_local_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_four_edge_local_object_mode_reserved_variable_type_assertion",
                super::extract_source_four_edge_local_object_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_four_edge_local_mode_asserted_head",
                super::extract_source_four_edge_local_mode_asserted_head,
            ),
            (
                "extract_source_four_edge_local_object_mode_asserted_head",
                super::extract_source_four_edge_local_object_mode_asserted_head,
            ),
            (
                "extract_source_four_edge_local_mode_radix_asserted_head",
                super::extract_source_four_edge_local_mode_radix_asserted_head,
            ),
            (
                "extract_source_four_edge_local_object_mode_radix_asserted_head",
                super::extract_source_four_edge_local_object_mode_radix_asserted_head,
            ),
            (
                "extract_source_local_mode_long_chain_reserved_variable_type_assertion",
                super::extract_source_local_mode_long_chain_reserved_variable_type_assertion,
            ),
            (
                "extract_source_local_object_mode_long_chain_reserved_variable_type_assertion",
                super::extract_source_local_object_mode_long_chain_reserved_variable_type_assertion,
            ),
            (
                "extract_source_local_mode_long_chain_asserted_head",
                super::extract_source_local_mode_long_chain_asserted_head,
            ),
            (
                "extract_source_local_object_mode_long_chain_asserted_head",
                super::extract_source_local_object_mode_long_chain_asserted_head,
            ),
            (
                "extract_source_local_mode_long_chain_radix_asserted_head",
                super::extract_source_local_mode_long_chain_radix_asserted_head,
            ),
            (
                "extract_source_local_object_mode_long_chain_radix_asserted_head",
                super::extract_source_local_object_mode_long_chain_radix_asserted_head,
            ),
            (
                "extract_source_two_edge_local_mode_two_hop_asserted_head",
                super::extract_source_two_edge_local_mode_two_hop_asserted_head,
            ),
            (
                "extract_source_two_edge_local_object_mode_two_hop_asserted_head",
                super::extract_source_two_edge_local_object_mode_two_hop_asserted_head,
            ),
            (
                "extract_source_three_edge_local_mode_two_hop_asserted_head",
                super::extract_source_three_edge_local_mode_two_hop_asserted_head,
            ),
            (
                "extract_source_three_edge_local_object_mode_two_hop_asserted_head",
                super::extract_source_three_edge_local_object_mode_two_hop_asserted_head,
            ),
            (
                "extract_source_four_edge_local_mode_two_hop_asserted_head",
                super::extract_source_four_edge_local_mode_two_hop_asserted_head,
            ),
            (
                "extract_source_four_edge_local_object_mode_two_hop_asserted_head",
                super::extract_source_four_edge_local_object_mode_two_hop_asserted_head,
            ),
            (
                "extract_source_three_edge_local_mode_three_hop_asserted_head",
                super::extract_source_three_edge_local_mode_three_hop_asserted_head,
            ),
            (
                "extract_source_three_edge_local_object_mode_three_hop_asserted_head",
                super::extract_source_three_edge_local_object_mode_three_hop_asserted_head,
            ),
            (
                "extract_source_four_edge_local_mode_three_hop_asserted_head",
                super::extract_source_four_edge_local_mode_three_hop_asserted_head,
            ),
            (
                "extract_source_four_edge_local_object_mode_three_hop_asserted_head",
                super::extract_source_four_edge_local_object_mode_three_hop_asserted_head,
            ),
            (
                "extract_source_four_edge_local_mode_four_hop_asserted_head",
                super::extract_source_four_edge_local_mode_four_hop_asserted_head,
            ),
            (
                "extract_source_four_edge_local_object_mode_four_hop_asserted_head",
                super::extract_source_four_edge_local_object_mode_four_hop_asserted_head,
            ),
        ];
        for (owner, extractor) in prior_extractors {
            assert!(
                extractor(&ast, resolver.module.clone(), &symbols).is_none(),
                "Task 224 source must not be owned by {owner}"
            );
        }

        let prior_owner_ids = [
            "pass_type_elaboration_reserved_variable_type_assertion_001",
            "pass_type_elaboration_reserved_object_variable_type_assertion_001",
            "pass_type_elaboration_local_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_local_object_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_local_mode_asserted_head_001",
            "pass_type_elaboration_local_object_mode_asserted_head_001",
            "pass_type_elaboration_chained_local_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_chained_local_object_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_chained_local_mode_asserted_head_001",
            "pass_type_elaboration_chained_local_object_mode_asserted_head_001",
            "pass_type_elaboration_chained_local_mode_radix_asserted_head_001",
            "pass_type_elaboration_chained_local_object_mode_radix_asserted_head_001",
            "pass_type_elaboration_two_edge_local_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_two_edge_local_object_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_two_edge_local_mode_asserted_head_001",
            "pass_type_elaboration_two_edge_local_object_mode_asserted_head_001",
            "pass_type_elaboration_two_edge_local_mode_radix_asserted_head_001",
            "pass_type_elaboration_two_edge_local_object_mode_radix_asserted_head_001",
            "pass_type_elaboration_three_edge_local_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_three_edge_local_object_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_three_edge_local_mode_asserted_head_001",
            "pass_type_elaboration_three_edge_local_object_mode_asserted_head_001",
            "pass_type_elaboration_three_edge_local_mode_radix_asserted_head_001",
            "pass_type_elaboration_three_edge_local_object_mode_radix_asserted_head_001",
            "pass_type_elaboration_four_edge_local_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_four_edge_local_object_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_four_edge_local_mode_asserted_head_001",
            "pass_type_elaboration_four_edge_local_object_mode_asserted_head_001",
            "pass_type_elaboration_four_edge_local_mode_radix_asserted_head_001",
            "pass_type_elaboration_four_edge_local_object_mode_radix_asserted_head_001",
            "pass_type_elaboration_local_mode_long_chain_reserved_variable_type_assertion_001",
            "pass_type_elaboration_local_object_mode_long_chain_reserved_variable_type_assertion_001",
            "pass_type_elaboration_local_mode_long_chain_asserted_head_001",
            "pass_type_elaboration_local_object_mode_long_chain_asserted_head_001",
            "pass_type_elaboration_local_mode_long_chain_radix_asserted_head_001",
            "pass_type_elaboration_local_object_mode_long_chain_radix_asserted_head_001",
            "pass_type_elaboration_two_edge_local_mode_two_hop_asserted_head_001",
            "pass_type_elaboration_two_edge_local_object_mode_two_hop_asserted_head_001",
            "pass_type_elaboration_three_edge_local_mode_two_hop_asserted_head_001",
            "pass_type_elaboration_three_edge_local_object_mode_two_hop_asserted_head_001",
            "pass_type_elaboration_four_edge_local_mode_two_hop_asserted_head_001",
            "pass_type_elaboration_four_edge_local_object_mode_two_hop_asserted_head_001",
            "pass_type_elaboration_three_edge_local_mode_three_hop_asserted_head_001",
            "pass_type_elaboration_three_edge_local_object_mode_three_hop_asserted_head_001",
            "pass_type_elaboration_four_edge_local_mode_three_hop_asserted_head_001",
            "pass_type_elaboration_four_edge_local_object_mode_three_hop_asserted_head_001",
            "pass_type_elaboration_four_edge_local_mode_four_hop_asserted_head_001",
            "pass_type_elaboration_four_edge_local_object_mode_four_hop_asserted_head_001",
        ];
        assert_eq!(prior_owner_ids.len(), 48);
        for owner_id in prior_owner_ids {
            let (owner_ordinal, owner_case) = active_type_elaboration_cases(&plan)
                .enumerate()
                .find(|(_, case)| case.id.0 == owner_id)
                .unwrap_or_else(|| panic!("prior owner fixture {owner_id} must be active"));
            let owner_frontend = run_frontend(&workspace_root, owner_case, owner_ordinal)
                .unwrap_or_else(|error| panic!("owner fixture {owner_id} must parse: {error}"));
            assert!(owner_frontend.diagnostics.is_empty());
            let owner_ast = owner_frontend
                .ast
                .unwrap_or_else(|| panic!("owner fixture {owner_id} must produce an AST"));
            let owner_resolver =
                resolver_symbol_collection(&workspace_root, owner_case, &owner_ast);
            assert!(owner_resolver.detail_keys.is_empty());
            let owner_symbols = augment_type_elaboration_import_summaries(
                &owner_ast,
                &owner_resolver.module,
                owner_resolver.env,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                ),
                Vec::<String>::new(),
                "prior owner fixture {owner_id} must still reach its active route"
            );
            assert!(
                super::extract_source_local_mode_long_chain_two_hop_asserted_head(
                    &owner_ast,
                    owner_resolver.module,
                    &owner_symbols,
                )
                .is_none(),
                "Task 224 must reject prior owner fixture {owner_id}"
            );
        }
    }

    #[test]
    fn active_local_object_mode_long_chain_two_hop_asserted_head_fixture_consumes_seven_expansions()
    {
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("mizar-test crate should live below the workspace root")
            .to_path_buf();
        let config = DiscoveryConfig {
            workspace_root: workspace_root.clone(),
            tests_root: workspace_root.join("tests"),
            manifest_path: workspace_root.join("tests/coverage/spec_trace.toml"),
            profile: TestProfile::Fast,
            validation_mode: ValidationMode::Metadata,
        };
        let plan = build_test_plan(&config).expect("Task 225 repository plan should build");
        let (ordinal, case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| {
                case.id.0 == "pass_type_elaboration_local_object_mode_long_chain_two_hop_asserted_head_001"
            })
            .expect("Task 225 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 225 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 225 fixture should produce a real AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = super::source_local_object_mode_long_chain_two_hop_asserted_head_output(
            &ast,
            resolver.module.clone(),
            &symbols,
        )
        .expect("Task 225 real AST should reach the seven-expansion two-hop seam");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 225 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.subject_result_input.spelling, "ChainObjectMode6");
        assert_eq!(output.asserted_type_input.spelling, "ChainObjectMode4");
        assert_ne!(
            output.subject_result_input.head,
            output.asserted_type_input.head
        );
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 225 normalized object type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert!(output.term_formula.candidate_sets().is_empty());
        assert!(output.term_formula.facts().is_empty());
        assert!(output.term_formula.diagnostics().is_empty());
        let chain6 = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("ChainObjectMode6"))
            .map(|(_, expansion)| expansion)
            .expect("Task 225 ChainObjectMode6 expansion should exist");
        assert_eq!(chain6.radix.spelling, "ChainObjectMode5");
        let TypeHeadInput::Symbol(chain5_symbol) = &chain6.radix.head else {
            panic!("Task 225 ChainObjectMode5 intermediate should be a symbol");
        };
        let chain5 = output
            .payload
            .reserve
            .mode_expansions
            .get(chain5_symbol)
            .expect("Task 225 ChainObjectMode5 expansion should exist");
        assert_eq!(chain5.radix.spelling, "ChainObjectMode4");
        assert_eq!(chain5.radix.head, output.asserted_type_input.head);

        type OwnerExtractor = fn(
            &SurfaceAst,
            ResolverModuleId,
            &SymbolEnv,
        ) -> Option<super::SourceReservedVariableTypeAssertion>;
        let prior_extractors: [(&str, OwnerExtractor); 49] = [
            (
                "extract_source_reserved_variable_type_assertion",
                super::extract_source_reserved_variable_type_assertion,
            ),
            (
                "extract_source_reserved_object_variable_type_assertion",
                super::extract_source_reserved_object_variable_type_assertion,
            ),
            (
                "extract_source_local_mode_reserved_variable_type_assertion",
                super::extract_source_local_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_local_object_mode_reserved_variable_type_assertion",
                super::extract_source_local_object_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_local_mode_asserted_head",
                super::extract_source_local_mode_asserted_head,
            ),
            (
                "extract_source_local_object_mode_asserted_head",
                super::extract_source_local_object_mode_asserted_head,
            ),
            (
                "extract_source_chained_local_mode_reserved_variable_type_assertion",
                super::extract_source_chained_local_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_chained_local_object_mode_reserved_variable_type_assertion",
                super::extract_source_chained_local_object_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_chained_local_mode_asserted_head",
                super::extract_source_chained_local_mode_asserted_head,
            ),
            (
                "extract_source_chained_local_object_mode_asserted_head",
                super::extract_source_chained_local_object_mode_asserted_head,
            ),
            (
                "extract_source_chained_local_mode_radix_asserted_head",
                super::extract_source_chained_local_mode_radix_asserted_head,
            ),
            (
                "extract_source_chained_local_object_mode_radix_asserted_head",
                super::extract_source_chained_local_object_mode_radix_asserted_head,
            ),
            (
                "extract_source_two_edge_local_mode_reserved_variable_type_assertion",
                super::extract_source_two_edge_local_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_two_edge_local_object_mode_reserved_variable_type_assertion",
                super::extract_source_two_edge_local_object_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_two_edge_local_mode_asserted_head",
                super::extract_source_two_edge_local_mode_asserted_head,
            ),
            (
                "extract_source_two_edge_local_object_mode_asserted_head",
                super::extract_source_two_edge_local_object_mode_asserted_head,
            ),
            (
                "extract_source_two_edge_local_mode_radix_asserted_head",
                super::extract_source_two_edge_local_mode_radix_asserted_head,
            ),
            (
                "extract_source_two_edge_local_object_mode_radix_asserted_head",
                super::extract_source_two_edge_local_object_mode_radix_asserted_head,
            ),
            (
                "extract_source_three_edge_local_mode_reserved_variable_type_assertion",
                super::extract_source_three_edge_local_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_three_edge_local_object_mode_reserved_variable_type_assertion",
                super::extract_source_three_edge_local_object_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_three_edge_local_mode_asserted_head",
                super::extract_source_three_edge_local_mode_asserted_head,
            ),
            (
                "extract_source_three_edge_local_object_mode_asserted_head",
                super::extract_source_three_edge_local_object_mode_asserted_head,
            ),
            (
                "extract_source_three_edge_local_mode_radix_asserted_head",
                super::extract_source_three_edge_local_mode_radix_asserted_head,
            ),
            (
                "extract_source_three_edge_local_object_mode_radix_asserted_head",
                super::extract_source_three_edge_local_object_mode_radix_asserted_head,
            ),
            (
                "extract_source_four_edge_local_mode_reserved_variable_type_assertion",
                super::extract_source_four_edge_local_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_four_edge_local_object_mode_reserved_variable_type_assertion",
                super::extract_source_four_edge_local_object_mode_reserved_variable_type_assertion,
            ),
            (
                "extract_source_four_edge_local_mode_asserted_head",
                super::extract_source_four_edge_local_mode_asserted_head,
            ),
            (
                "extract_source_four_edge_local_object_mode_asserted_head",
                super::extract_source_four_edge_local_object_mode_asserted_head,
            ),
            (
                "extract_source_four_edge_local_mode_radix_asserted_head",
                super::extract_source_four_edge_local_mode_radix_asserted_head,
            ),
            (
                "extract_source_four_edge_local_object_mode_radix_asserted_head",
                super::extract_source_four_edge_local_object_mode_radix_asserted_head,
            ),
            (
                "extract_source_local_mode_long_chain_reserved_variable_type_assertion",
                super::extract_source_local_mode_long_chain_reserved_variable_type_assertion,
            ),
            (
                "extract_source_local_object_mode_long_chain_reserved_variable_type_assertion",
                super::extract_source_local_object_mode_long_chain_reserved_variable_type_assertion,
            ),
            (
                "extract_source_local_mode_long_chain_asserted_head",
                super::extract_source_local_mode_long_chain_asserted_head,
            ),
            (
                "extract_source_local_object_mode_long_chain_asserted_head",
                super::extract_source_local_object_mode_long_chain_asserted_head,
            ),
            (
                "extract_source_local_mode_long_chain_radix_asserted_head",
                super::extract_source_local_mode_long_chain_radix_asserted_head,
            ),
            (
                "extract_source_local_object_mode_long_chain_radix_asserted_head",
                super::extract_source_local_object_mode_long_chain_radix_asserted_head,
            ),
            (
                "extract_source_two_edge_local_mode_two_hop_asserted_head",
                super::extract_source_two_edge_local_mode_two_hop_asserted_head,
            ),
            (
                "extract_source_two_edge_local_object_mode_two_hop_asserted_head",
                super::extract_source_two_edge_local_object_mode_two_hop_asserted_head,
            ),
            (
                "extract_source_three_edge_local_mode_two_hop_asserted_head",
                super::extract_source_three_edge_local_mode_two_hop_asserted_head,
            ),
            (
                "extract_source_three_edge_local_object_mode_two_hop_asserted_head",
                super::extract_source_three_edge_local_object_mode_two_hop_asserted_head,
            ),
            (
                "extract_source_four_edge_local_mode_two_hop_asserted_head",
                super::extract_source_four_edge_local_mode_two_hop_asserted_head,
            ),
            (
                "extract_source_four_edge_local_object_mode_two_hop_asserted_head",
                super::extract_source_four_edge_local_object_mode_two_hop_asserted_head,
            ),
            (
                "extract_source_three_edge_local_mode_three_hop_asserted_head",
                super::extract_source_three_edge_local_mode_three_hop_asserted_head,
            ),
            (
                "extract_source_three_edge_local_object_mode_three_hop_asserted_head",
                super::extract_source_three_edge_local_object_mode_three_hop_asserted_head,
            ),
            (
                "extract_source_four_edge_local_mode_three_hop_asserted_head",
                super::extract_source_four_edge_local_mode_three_hop_asserted_head,
            ),
            (
                "extract_source_four_edge_local_object_mode_three_hop_asserted_head",
                super::extract_source_four_edge_local_object_mode_three_hop_asserted_head,
            ),
            (
                "extract_source_four_edge_local_mode_four_hop_asserted_head",
                super::extract_source_four_edge_local_mode_four_hop_asserted_head,
            ),
            (
                "extract_source_four_edge_local_object_mode_four_hop_asserted_head",
                super::extract_source_four_edge_local_object_mode_four_hop_asserted_head,
            ),
            (
                "extract_source_local_mode_long_chain_two_hop_asserted_head",
                super::extract_source_local_mode_long_chain_two_hop_asserted_head,
            ),
        ];
        for (owner, extractor) in prior_extractors {
            assert!(
                extractor(&ast, resolver.module.clone(), &symbols).is_none(),
                "Task 225 source must not be owned by {owner}"
            );
        }

        let prior_owner_ids = [
            "pass_type_elaboration_reserved_variable_type_assertion_001",
            "pass_type_elaboration_reserved_object_variable_type_assertion_001",
            "pass_type_elaboration_local_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_local_object_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_local_mode_asserted_head_001",
            "pass_type_elaboration_local_object_mode_asserted_head_001",
            "pass_type_elaboration_chained_local_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_chained_local_object_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_chained_local_mode_asserted_head_001",
            "pass_type_elaboration_chained_local_object_mode_asserted_head_001",
            "pass_type_elaboration_chained_local_mode_radix_asserted_head_001",
            "pass_type_elaboration_chained_local_object_mode_radix_asserted_head_001",
            "pass_type_elaboration_two_edge_local_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_two_edge_local_object_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_two_edge_local_mode_asserted_head_001",
            "pass_type_elaboration_two_edge_local_object_mode_asserted_head_001",
            "pass_type_elaboration_two_edge_local_mode_radix_asserted_head_001",
            "pass_type_elaboration_two_edge_local_object_mode_radix_asserted_head_001",
            "pass_type_elaboration_three_edge_local_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_three_edge_local_object_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_three_edge_local_mode_asserted_head_001",
            "pass_type_elaboration_three_edge_local_object_mode_asserted_head_001",
            "pass_type_elaboration_three_edge_local_mode_radix_asserted_head_001",
            "pass_type_elaboration_three_edge_local_object_mode_radix_asserted_head_001",
            "pass_type_elaboration_four_edge_local_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_four_edge_local_object_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_four_edge_local_mode_asserted_head_001",
            "pass_type_elaboration_four_edge_local_object_mode_asserted_head_001",
            "pass_type_elaboration_four_edge_local_mode_radix_asserted_head_001",
            "pass_type_elaboration_four_edge_local_object_mode_radix_asserted_head_001",
            "pass_type_elaboration_local_mode_long_chain_reserved_variable_type_assertion_001",
            "pass_type_elaboration_local_object_mode_long_chain_reserved_variable_type_assertion_001",
            "pass_type_elaboration_local_mode_long_chain_asserted_head_001",
            "pass_type_elaboration_local_object_mode_long_chain_asserted_head_001",
            "pass_type_elaboration_local_mode_long_chain_radix_asserted_head_001",
            "pass_type_elaboration_local_object_mode_long_chain_radix_asserted_head_001",
            "pass_type_elaboration_two_edge_local_mode_two_hop_asserted_head_001",
            "pass_type_elaboration_two_edge_local_object_mode_two_hop_asserted_head_001",
            "pass_type_elaboration_three_edge_local_mode_two_hop_asserted_head_001",
            "pass_type_elaboration_three_edge_local_object_mode_two_hop_asserted_head_001",
            "pass_type_elaboration_four_edge_local_mode_two_hop_asserted_head_001",
            "pass_type_elaboration_four_edge_local_object_mode_two_hop_asserted_head_001",
            "pass_type_elaboration_three_edge_local_mode_three_hop_asserted_head_001",
            "pass_type_elaboration_three_edge_local_object_mode_three_hop_asserted_head_001",
            "pass_type_elaboration_four_edge_local_mode_three_hop_asserted_head_001",
            "pass_type_elaboration_four_edge_local_object_mode_three_hop_asserted_head_001",
            "pass_type_elaboration_four_edge_local_mode_four_hop_asserted_head_001",
            "pass_type_elaboration_four_edge_local_object_mode_four_hop_asserted_head_001",
            "pass_type_elaboration_local_mode_long_chain_two_hop_asserted_head_001",
        ];
        assert_eq!(prior_owner_ids.len(), 49);
        for owner_id in prior_owner_ids {
            let (owner_ordinal, owner_case) = active_type_elaboration_cases(&plan)
                .enumerate()
                .find(|(_, case)| case.id.0 == owner_id)
                .unwrap_or_else(|| panic!("prior owner fixture {owner_id} must be active"));
            let owner_frontend = run_frontend(&workspace_root, owner_case, owner_ordinal)
                .unwrap_or_else(|error| panic!("owner fixture {owner_id} must parse: {error}"));
            assert!(owner_frontend.diagnostics.is_empty());
            let owner_ast = owner_frontend
                .ast
                .unwrap_or_else(|| panic!("owner fixture {owner_id} must produce an AST"));
            let owner_resolver =
                resolver_symbol_collection(&workspace_root, owner_case, &owner_ast);
            assert!(owner_resolver.detail_keys.is_empty());
            let owner_symbols = augment_type_elaboration_import_summaries(
                &owner_ast,
                &owner_resolver.module,
                owner_resolver.env,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                ),
                Vec::<String>::new(),
                "prior owner fixture {owner_id} must still reach its active route"
            );
            assert!(
                super::extract_source_local_object_mode_long_chain_two_hop_asserted_head(
                    &owner_ast,
                    owner_resolver.module,
                    &owner_symbols,
                )
                .is_none(),
                "Task 225 must reject prior owner fixture {owner_id}"
            );
        }
    }

    #[test]
    fn active_local_mode_long_chain_radix_asserted_head_fixture_consumes_seven_expansions() {
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("mizar-test crate should live below the workspace root")
            .to_path_buf();
        let config = DiscoveryConfig {
            workspace_root: workspace_root.clone(),
            tests_root: workspace_root.join("tests"),
            manifest_path: workspace_root.join("tests/coverage/spec_trace.toml"),
            profile: TestProfile::Fast,
            validation_mode: ValidationMode::Metadata,
        };
        let plan = build_test_plan(&config).expect("Task 209 repository plan should build");
        let (ordinal, case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| {
                case.id.0 == "pass_type_elaboration_local_mode_long_chain_radix_asserted_head_001"
            })
            .expect("Task 209 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 209 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 209 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = super::source_local_mode_long_chain_radix_asserted_head_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 209 real AST should reach the seven-expansion asserted-head seam");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 209 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert_eq!(output.subject_result_input.spelling, "ChainMode6");
        assert_eq!(output.asserted_type_input.spelling, "ChainMode5");
        assert_ne!(
            output.subject_result_input.head,
            output.asserted_type_input.head
        );
        assert!(matches!(
            output.asserted_type_input.head,
            TypeHeadInput::Symbol(_)
        ));
        assert_ne!(
            output.subject_result_input.site,
            output.asserted_type_input.site
        );
        assert_ne!(
            output.subject_result_input.source_range,
            output.asserted_type_input.source_range
        );
        let binding_expansion = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("ChainMode6"))
            .map(|(_, expansion)| expansion)
            .expect("Task 209 ChainMode6 binding expansion should exist");
        assert_eq!(binding_expansion.radix.spelling, "ChainMode5");
        assert_eq!(
            binding_expansion.radix.head,
            output.asserted_type_input.head
        );
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("BaseMode"))
            .map(|(_, expansion)| &expansion.radix)
            .expect("Task 209 BaseMode terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 209 normalized set type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
        let (_, term) = output.term_formula.terms().iter().next().unwrap();
        assert_eq!(term.status, TermStatus::Inferred);
        assert!(term.deferred.is_empty());
        let (_, formula) = output.term_formula.formulas().iter().next().unwrap();
        assert_eq!(formula.kind, FormulaKind::TypeAssertion);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert!(formula.expected_types.is_empty());
        assert!(formula.candidate_set.is_none());
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
        assert!(output.term_formula.candidate_sets().is_empty());
        assert!(output.term_formula.facts().is_empty());
        assert!(output.term_formula.diagnostics().is_empty());

        let preexisting_owner_ids = [
            "pass_type_elaboration_reserved_variable_type_assertion_001",
            "pass_type_elaboration_reserved_object_variable_type_assertion_001",
            "pass_type_elaboration_local_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_local_object_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_local_mode_asserted_head_001",
            "pass_type_elaboration_local_object_mode_asserted_head_001",
            "pass_type_elaboration_chained_local_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_chained_local_object_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_chained_local_mode_asserted_head_001",
            "pass_type_elaboration_chained_local_object_mode_asserted_head_001",
            "pass_type_elaboration_chained_local_mode_radix_asserted_head_001",
            "pass_type_elaboration_chained_local_object_mode_radix_asserted_head_001",
            "pass_type_elaboration_two_edge_local_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_two_edge_local_object_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_two_edge_local_mode_asserted_head_001",
            "pass_type_elaboration_two_edge_local_object_mode_asserted_head_001",
            "pass_type_elaboration_two_edge_local_mode_radix_asserted_head_001",
            "pass_type_elaboration_two_edge_local_object_mode_radix_asserted_head_001",
            "pass_type_elaboration_three_edge_local_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_three_edge_local_object_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_three_edge_local_mode_asserted_head_001",
            "pass_type_elaboration_three_edge_local_object_mode_asserted_head_001",
            "pass_type_elaboration_three_edge_local_mode_radix_asserted_head_001",
            "pass_type_elaboration_three_edge_local_object_mode_radix_asserted_head_001",
            "pass_type_elaboration_four_edge_local_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_four_edge_local_object_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_four_edge_local_mode_asserted_head_001",
            "pass_type_elaboration_four_edge_local_object_mode_asserted_head_001",
            "pass_type_elaboration_four_edge_local_mode_radix_asserted_head_001",
            "pass_type_elaboration_four_edge_local_object_mode_radix_asserted_head_001",
            "pass_type_elaboration_local_mode_long_chain_reserved_variable_type_assertion_001",
            "pass_type_elaboration_local_object_mode_long_chain_reserved_variable_type_assertion_001",
            "pass_type_elaboration_local_mode_long_chain_asserted_head_001",
            "pass_type_elaboration_local_object_mode_long_chain_asserted_head_001",
        ];
        assert_eq!(preexisting_owner_ids.len(), 34);
        for owner_id in preexisting_owner_ids {
            let (owner_ordinal, owner_case) = active_type_elaboration_cases(&plan)
                .enumerate()
                .find(|(_, case)| case.id.0 == owner_id)
                .unwrap_or_else(|| panic!("pre-existing owner fixture {owner_id} must be active"));
            let owner_frontend = run_frontend(&workspace_root, owner_case, owner_ordinal)
                .unwrap_or_else(|error| panic!("owner fixture {owner_id} must parse: {error}"));
            assert!(
                owner_frontend.diagnostics.is_empty(),
                "owner fixture {owner_id} must remain frontend-clean"
            );
            let owner_ast = owner_frontend
                .ast
                .unwrap_or_else(|| panic!("owner fixture {owner_id} must produce an AST"));
            let owner_resolver =
                resolver_symbol_collection(&workspace_root, owner_case, &owner_ast);
            assert!(
                owner_resolver.detail_keys.is_empty(),
                "owner fixture {owner_id} must remain resolver-clean"
            );
            let owner_symbols = augment_type_elaboration_import_summaries(
                &owner_ast,
                &owner_resolver.module,
                owner_resolver.env,
            );
            let owner_self_extracts = match owner_id {
                "pass_type_elaboration_reserved_variable_type_assertion_001" => {
                    extract_source_reserved_variable_type_assertion(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_reserved_object_variable_type_assertion_001" => {
                    super::extract_source_reserved_object_variable_type_assertion(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_local_mode_reserved_variable_type_assertion_001" => {
                    extract_source_local_mode_reserved_variable_type_assertion(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_local_object_mode_reserved_variable_type_assertion_001" => {
                    extract_source_local_object_mode_reserved_variable_type_assertion(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_local_mode_asserted_head_001" => {
                    extract_source_local_mode_asserted_head(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_local_object_mode_asserted_head_001" => {
                    extract_source_local_object_mode_asserted_head(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_chained_local_mode_reserved_variable_type_assertion_001" => {
                    extract_source_chained_local_mode_reserved_variable_type_assertion(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_chained_local_object_mode_reserved_variable_type_assertion_001" => {
                    extract_source_chained_local_object_mode_reserved_variable_type_assertion(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_chained_local_mode_asserted_head_001" => {
                    extract_source_chained_local_mode_asserted_head(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_chained_local_object_mode_asserted_head_001" => {
                    extract_source_chained_local_object_mode_asserted_head(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_chained_local_mode_radix_asserted_head_001" => {
                    extract_source_chained_local_mode_radix_asserted_head(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_chained_local_object_mode_radix_asserted_head_001" => {
                    extract_source_chained_local_object_mode_radix_asserted_head(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_two_edge_local_mode_reserved_variable_type_assertion_001" => {
                    extract_source_two_edge_local_mode_reserved_variable_type_assertion(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_two_edge_local_object_mode_reserved_variable_type_assertion_001" => {
                    extract_source_two_edge_local_object_mode_reserved_variable_type_assertion(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_two_edge_local_mode_asserted_head_001" => {
                    extract_source_two_edge_local_mode_asserted_head(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_two_edge_local_object_mode_asserted_head_001" => {
                    extract_source_two_edge_local_object_mode_asserted_head(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_two_edge_local_mode_radix_asserted_head_001" => {
                    extract_source_two_edge_local_mode_radix_asserted_head(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_two_edge_local_object_mode_radix_asserted_head_001" => {
                    extract_source_two_edge_local_object_mode_radix_asserted_head(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_three_edge_local_mode_reserved_variable_type_assertion_001" => {
                    extract_source_three_edge_local_mode_reserved_variable_type_assertion(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_three_edge_local_object_mode_reserved_variable_type_assertion_001" => {
                    extract_source_three_edge_local_object_mode_reserved_variable_type_assertion(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_three_edge_local_mode_asserted_head_001" => {
                    extract_source_three_edge_local_mode_asserted_head(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_three_edge_local_object_mode_asserted_head_001" => {
                    extract_source_three_edge_local_object_mode_asserted_head(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_three_edge_local_mode_radix_asserted_head_001" => {
                    extract_source_three_edge_local_mode_radix_asserted_head(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_three_edge_local_object_mode_radix_asserted_head_001" => {
                    extract_source_three_edge_local_object_mode_radix_asserted_head(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_four_edge_local_mode_reserved_variable_type_assertion_001" => {
                    extract_source_four_edge_local_mode_reserved_variable_type_assertion(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_four_edge_local_object_mode_reserved_variable_type_assertion_001" => {
                    extract_source_four_edge_local_object_mode_reserved_variable_type_assertion(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_four_edge_local_mode_asserted_head_001" => {
                    super::extract_source_four_edge_local_mode_asserted_head(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_four_edge_local_object_mode_asserted_head_001" => {
                    super::extract_source_four_edge_local_object_mode_asserted_head(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_four_edge_local_mode_radix_asserted_head_001" => {
                    extract_source_four_edge_local_mode_radix_asserted_head(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_four_edge_local_object_mode_radix_asserted_head_001" => {
                    extract_source_four_edge_local_object_mode_radix_asserted_head(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_local_mode_long_chain_reserved_variable_type_assertion_001" => {
                    extract_source_local_mode_long_chain_reserved_variable_type_assertion(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_local_object_mode_long_chain_reserved_variable_type_assertion_001" => {
                    extract_source_local_object_mode_long_chain_reserved_variable_type_assertion(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_local_mode_long_chain_asserted_head_001" => {
                    super::extract_source_local_mode_long_chain_asserted_head(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_local_object_mode_long_chain_asserted_head_001" => {
                    super::extract_source_local_object_mode_long_chain_asserted_head(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                _ => unreachable!("undeclared pre-existing owner fixture {owner_id}"),
            };
            assert!(
                owner_self_extracts,
                "pre-existing owner fixture {owner_id} must non-vacuously reach its own extractor"
            );
            assert!(
                extract_source_local_mode_long_chain_radix_asserted_head(
                    &owner_ast,
                    owner_resolver.module,
                    &owner_symbols,
                )
                .is_none(),
                "Task 209 must reject pre-existing owner fixture {owner_id}"
            );
        }
    }

    #[test]
    fn active_local_object_mode_long_chain_radix_asserted_head_fixture_consumes_seven_expansions() {
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("mizar-test crate should live below the workspace root")
            .to_path_buf();
        let config = DiscoveryConfig {
            workspace_root: workspace_root.clone(),
            tests_root: workspace_root.join("tests"),
            manifest_path: workspace_root.join("tests/coverage/spec_trace.toml"),
            profile: TestProfile::Fast,
            validation_mode: ValidationMode::Metadata,
        };
        let plan = build_test_plan(&config).expect("Task 210 repository plan should build");
        let (ordinal, case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| {
                case.id.0
                    == "pass_type_elaboration_local_object_mode_long_chain_radix_asserted_head_001"
            })
            .expect("Task 210 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 210 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 210 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = super::source_local_object_mode_long_chain_radix_asserted_head_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 210 real AST should reach the seven-expansion asserted-head seam");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 210 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert_eq!(output.subject_result_input.spelling, "ChainObjectMode6");
        assert_eq!(output.asserted_type_input.spelling, "ChainObjectMode5");
        assert_ne!(
            output.subject_result_input.head,
            output.asserted_type_input.head
        );
        assert!(matches!(
            output.asserted_type_input.head,
            TypeHeadInput::Symbol(_)
        ));
        assert_ne!(
            output.subject_result_input.site,
            output.asserted_type_input.site
        );
        assert_ne!(
            output.subject_result_input.source_range,
            output.asserted_type_input.source_range
        );
        let binding_expansion = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("ChainObjectMode6"))
            .map(|(_, expansion)| expansion)
            .expect("Task 210 ChainObjectMode6 binding expansion should exist");
        assert_eq!(binding_expansion.radix.spelling, "ChainObjectMode5");
        assert_eq!(
            binding_expansion.radix.head,
            output.asserted_type_input.head
        );
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("BaseObjectMode"))
            .map(|(_, expansion)| &expansion.radix)
            .expect("Task 210 BaseObjectMode terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 210 normalized object type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
        let (_, term) = output.term_formula.terms().iter().next().unwrap();
        assert_eq!(term.status, TermStatus::Inferred);
        assert!(term.deferred.is_empty());
        let (_, formula) = output.term_formula.formulas().iter().next().unwrap();
        assert_eq!(formula.kind, FormulaKind::TypeAssertion);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert!(formula.expected_types.is_empty());
        assert!(formula.candidate_set.is_none());
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
        assert!(output.term_formula.candidate_sets().is_empty());
        assert!(output.term_formula.facts().is_empty());
        assert!(output.term_formula.diagnostics().is_empty());

        let preexisting_owner_ids = [
            "pass_type_elaboration_reserved_variable_type_assertion_001",
            "pass_type_elaboration_reserved_object_variable_type_assertion_001",
            "pass_type_elaboration_local_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_local_object_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_local_mode_asserted_head_001",
            "pass_type_elaboration_local_object_mode_asserted_head_001",
            "pass_type_elaboration_chained_local_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_chained_local_object_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_chained_local_mode_asserted_head_001",
            "pass_type_elaboration_chained_local_object_mode_asserted_head_001",
            "pass_type_elaboration_chained_local_mode_radix_asserted_head_001",
            "pass_type_elaboration_chained_local_object_mode_radix_asserted_head_001",
            "pass_type_elaboration_two_edge_local_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_two_edge_local_object_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_two_edge_local_mode_asserted_head_001",
            "pass_type_elaboration_two_edge_local_object_mode_asserted_head_001",
            "pass_type_elaboration_two_edge_local_mode_radix_asserted_head_001",
            "pass_type_elaboration_two_edge_local_object_mode_radix_asserted_head_001",
            "pass_type_elaboration_three_edge_local_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_three_edge_local_object_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_three_edge_local_mode_asserted_head_001",
            "pass_type_elaboration_three_edge_local_object_mode_asserted_head_001",
            "pass_type_elaboration_three_edge_local_mode_radix_asserted_head_001",
            "pass_type_elaboration_three_edge_local_object_mode_radix_asserted_head_001",
            "pass_type_elaboration_four_edge_local_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_four_edge_local_object_mode_reserved_variable_type_assertion_001",
            "pass_type_elaboration_four_edge_local_mode_asserted_head_001",
            "pass_type_elaboration_four_edge_local_object_mode_asserted_head_001",
            "pass_type_elaboration_four_edge_local_mode_radix_asserted_head_001",
            "pass_type_elaboration_four_edge_local_object_mode_radix_asserted_head_001",
            "pass_type_elaboration_local_mode_long_chain_reserved_variable_type_assertion_001",
            "pass_type_elaboration_local_object_mode_long_chain_reserved_variable_type_assertion_001",
            "pass_type_elaboration_local_mode_long_chain_asserted_head_001",
            "pass_type_elaboration_local_object_mode_long_chain_asserted_head_001",
            "pass_type_elaboration_local_mode_long_chain_radix_asserted_head_001",
        ];
        assert_eq!(preexisting_owner_ids.len(), 35);
        for owner_id in preexisting_owner_ids {
            let (owner_ordinal, owner_case) = active_type_elaboration_cases(&plan)
                .enumerate()
                .find(|(_, case)| case.id.0 == owner_id)
                .unwrap_or_else(|| panic!("pre-existing owner fixture {owner_id} must be active"));
            let owner_frontend = run_frontend(&workspace_root, owner_case, owner_ordinal)
                .unwrap_or_else(|error| panic!("owner fixture {owner_id} must parse: {error}"));
            assert!(
                owner_frontend.diagnostics.is_empty(),
                "owner fixture {owner_id} must remain frontend-clean"
            );
            let owner_ast = owner_frontend
                .ast
                .unwrap_or_else(|| panic!("owner fixture {owner_id} must produce an AST"));
            let owner_resolver =
                resolver_symbol_collection(&workspace_root, owner_case, &owner_ast);
            assert!(
                owner_resolver.detail_keys.is_empty(),
                "owner fixture {owner_id} must remain resolver-clean"
            );
            let owner_symbols = augment_type_elaboration_import_summaries(
                &owner_ast,
                &owner_resolver.module,
                owner_resolver.env,
            );
            let owner_self_extracts = match owner_id {
                "pass_type_elaboration_reserved_variable_type_assertion_001" => {
                    extract_source_reserved_variable_type_assertion(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_reserved_object_variable_type_assertion_001" => {
                    super::extract_source_reserved_object_variable_type_assertion(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_local_mode_reserved_variable_type_assertion_001" => {
                    extract_source_local_mode_reserved_variable_type_assertion(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_local_object_mode_reserved_variable_type_assertion_001" => {
                    extract_source_local_object_mode_reserved_variable_type_assertion(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_local_mode_asserted_head_001" => {
                    extract_source_local_mode_asserted_head(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_local_object_mode_asserted_head_001" => {
                    extract_source_local_object_mode_asserted_head(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_chained_local_mode_reserved_variable_type_assertion_001" => {
                    extract_source_chained_local_mode_reserved_variable_type_assertion(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_chained_local_object_mode_reserved_variable_type_assertion_001" => {
                    extract_source_chained_local_object_mode_reserved_variable_type_assertion(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_chained_local_mode_asserted_head_001" => {
                    extract_source_chained_local_mode_asserted_head(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_chained_local_object_mode_asserted_head_001" => {
                    extract_source_chained_local_object_mode_asserted_head(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_chained_local_mode_radix_asserted_head_001" => {
                    extract_source_chained_local_mode_radix_asserted_head(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_chained_local_object_mode_radix_asserted_head_001" => {
                    extract_source_chained_local_object_mode_radix_asserted_head(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_two_edge_local_mode_reserved_variable_type_assertion_001" => {
                    extract_source_two_edge_local_mode_reserved_variable_type_assertion(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_two_edge_local_object_mode_reserved_variable_type_assertion_001" => {
                    extract_source_two_edge_local_object_mode_reserved_variable_type_assertion(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_two_edge_local_mode_asserted_head_001" => {
                    extract_source_two_edge_local_mode_asserted_head(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_two_edge_local_object_mode_asserted_head_001" => {
                    extract_source_two_edge_local_object_mode_asserted_head(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_two_edge_local_mode_radix_asserted_head_001" => {
                    extract_source_two_edge_local_mode_radix_asserted_head(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_two_edge_local_object_mode_radix_asserted_head_001" => {
                    extract_source_two_edge_local_object_mode_radix_asserted_head(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_three_edge_local_mode_reserved_variable_type_assertion_001" => {
                    extract_source_three_edge_local_mode_reserved_variable_type_assertion(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_three_edge_local_object_mode_reserved_variable_type_assertion_001" => {
                    extract_source_three_edge_local_object_mode_reserved_variable_type_assertion(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_three_edge_local_mode_asserted_head_001" => {
                    extract_source_three_edge_local_mode_asserted_head(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_three_edge_local_object_mode_asserted_head_001" => {
                    extract_source_three_edge_local_object_mode_asserted_head(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_three_edge_local_mode_radix_asserted_head_001" => {
                    extract_source_three_edge_local_mode_radix_asserted_head(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_three_edge_local_object_mode_radix_asserted_head_001" => {
                    extract_source_three_edge_local_object_mode_radix_asserted_head(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_four_edge_local_mode_reserved_variable_type_assertion_001" => {
                    extract_source_four_edge_local_mode_reserved_variable_type_assertion(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_four_edge_local_object_mode_reserved_variable_type_assertion_001" => {
                    extract_source_four_edge_local_object_mode_reserved_variable_type_assertion(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_four_edge_local_mode_asserted_head_001" => {
                    super::extract_source_four_edge_local_mode_asserted_head(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_four_edge_local_object_mode_asserted_head_001" => {
                    super::extract_source_four_edge_local_object_mode_asserted_head(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_four_edge_local_mode_radix_asserted_head_001" => {
                    extract_source_four_edge_local_mode_radix_asserted_head(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_four_edge_local_object_mode_radix_asserted_head_001" => {
                    extract_source_four_edge_local_object_mode_radix_asserted_head(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_local_mode_long_chain_reserved_variable_type_assertion_001" => {
                    extract_source_local_mode_long_chain_reserved_variable_type_assertion(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_local_object_mode_long_chain_reserved_variable_type_assertion_001" => {
                    extract_source_local_object_mode_long_chain_reserved_variable_type_assertion(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_local_mode_long_chain_asserted_head_001" => {
                    super::extract_source_local_mode_long_chain_asserted_head(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_local_object_mode_long_chain_asserted_head_001" => {
                    super::extract_source_local_object_mode_long_chain_asserted_head(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                "pass_type_elaboration_local_mode_long_chain_radix_asserted_head_001" => {
                    extract_source_local_mode_long_chain_radix_asserted_head(
                        &owner_ast,
                        owner_resolver.module.clone(),
                        &owner_symbols,
                    )
                    .is_some()
                }
                _ => unreachable!("undeclared pre-existing owner fixture {owner_id}"),
            };
            assert!(
                owner_self_extracts,
                "pre-existing owner fixture {owner_id} must non-vacuously reach its own extractor"
            );
            assert!(
                super::extract_source_local_object_mode_long_chain_radix_asserted_head(
                    &owner_ast,
                    owner_resolver.module,
                    &owner_symbols,
                )
                .is_none(),
                "Task 210 must reject pre-existing owner fixture {owner_id}"
            );
        }
    }

    #[test]
    fn active_local_mode_long_chain_asserted_head_fixture_consumes_seven_expansions() {
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("mizar-test crate should live below the workspace root")
            .to_path_buf();
        let config = DiscoveryConfig {
            workspace_root: workspace_root.clone(),
            tests_root: workspace_root.join("tests"),
            manifest_path: workspace_root.join("tests/coverage/spec_trace.toml"),
            profile: TestProfile::Fast,
            validation_mode: ValidationMode::Metadata,
        };
        let plan = build_test_plan(&config).expect("Task 199 repository plan should build");
        let (ordinal, case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| {
                case.id.0 == "pass_type_elaboration_local_mode_long_chain_asserted_head_001"
            })
            .expect("Task 199 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 199 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 199 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = super::source_local_mode_long_chain_asserted_head_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 199 real AST should reach the seven-expansion asserted-head seam");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 199 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert_eq!(output.subject_result_input.spelling, "ChainMode6");
        assert_eq!(output.asserted_type_input.spelling, "ChainMode6");
        assert_eq!(
            output.subject_result_input.head,
            output.asserted_type_input.head
        );
        assert!(matches!(
            output.asserted_type_input.head,
            TypeHeadInput::Symbol(_)
        ));
        assert_ne!(
            output.subject_result_input.site,
            output.asserted_type_input.site
        );
        assert_ne!(
            output.subject_result_input.source_range,
            output.asserted_type_input.source_range
        );
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("BaseMode"))
            .map(|(_, expansion)| &expansion.radix)
            .expect("Task 199 BaseMode terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 199 normalized set type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
        let (_, term) = output.term_formula.terms().iter().next().unwrap();
        assert_eq!(term.status, TermStatus::Inferred);
        assert!(term.deferred.is_empty());
        let (_, formula) = output.term_formula.formulas().iter().next().unwrap();
        assert_eq!(formula.kind, FormulaKind::TypeAssertion);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert!(formula.expected_types.is_empty());
        assert!(formula.candidate_set.is_none());
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
        assert!(output.term_formula.candidate_sets().is_empty());
        assert!(output.term_formula.facts().is_empty());
        assert!(output.term_formula.diagnostics().is_empty());
    }

    #[test]
    fn active_local_object_mode_long_chain_asserted_head_fixture_consumes_seven_expansions() {
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("mizar-test crate should live below the workspace root")
            .to_path_buf();
        let config = DiscoveryConfig {
            workspace_root: workspace_root.clone(),
            tests_root: workspace_root.join("tests"),
            manifest_path: workspace_root.join("tests/coverage/spec_trace.toml"),
            profile: TestProfile::Fast,
            validation_mode: ValidationMode::Metadata,
        };
        let plan = build_test_plan(&config).expect("Task 200 repository plan should build");
        let (ordinal, case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| {
                case.id.0 == "pass_type_elaboration_local_object_mode_long_chain_asserted_head_001"
            })
            .expect("Task 200 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 200 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 200 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = super::source_local_object_mode_long_chain_asserted_head_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 200 real AST should reach the seven-expansion object asserted-head seam");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 200 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert_eq!(output.subject_result_input.spelling, "ChainObjectMode6");
        assert_eq!(output.asserted_type_input.spelling, "ChainObjectMode6");
        assert_eq!(
            output.subject_result_input.head,
            output.asserted_type_input.head
        );
        assert!(matches!(
            output.asserted_type_input.head,
            TypeHeadInput::Symbol(_)
        ));
        assert_ne!(
            output.subject_result_input.site,
            output.asserted_type_input.site
        );
        assert_ne!(
            output.subject_result_input.source_range,
            output.asserted_type_input.source_range
        );
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("BaseObjectMode"))
            .map(|(_, expansion)| &expansion.radix)
            .expect("Task 200 BaseObjectMode terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 200 normalized object type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
        let (_, term) = output.term_formula.terms().iter().next().unwrap();
        assert_eq!(term.status, TermStatus::Inferred);
        assert!(term.deferred.is_empty());
        let (_, formula) = output.term_formula.formulas().iter().next().unwrap();
        assert_eq!(formula.kind, FormulaKind::TypeAssertion);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert!(formula.expected_types.is_empty());
        assert!(formula.candidate_set.is_none());
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
        assert!(output.term_formula.candidate_sets().is_empty());
        assert!(output.term_formula.facts().is_empty());
        assert!(output.term_formula.diagnostics().is_empty());
    }

    #[test]
    fn source_local_object_mode_long_chain_reserved_variable_type_assertion_consumes_seven_expansions()
     {
        let source_id = source_id(179);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("local_object_mode_long_chain_reserved_variable_type_assertion"),
        );
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseObjectMode", SymbolKind::Mode),
                ("ChainObjectMode1", SymbolKind::Mode),
                ("ChainObjectMode2", SymbolKind::Mode),
                ("ChainObjectMode3", SymbolKind::Mode),
                ("ChainObjectMode4", SymbolKind::Mode),
                ("ChainObjectMode5", SymbolKind::Mode),
                ("ChainObjectMode6", SymbolKind::Mode),
            ],
        );
        let theorem = IdentifierTypeAssertionTheoremSpec {
            status: None,
            label: "LongLocalObjectModeReservedVariableTypeAssertionPayloadBoundary",
            subject: "x",
            asserted_type: ReserveTypeShape::Builtin("object"),
            recovered_label: false,
            negated: false,
        };
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    "BaseObjectMode",
                    "BaseObjectModeDef",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition_with_label(
                    "ChainObjectMode1",
                    "ChainObjectMode1Def",
                    ReserveTypeShape::QualifiedSymbol("BaseObjectMode"),
                ),
                mode_definition_with_label(
                    "ChainObjectMode2",
                    "ChainObjectMode2Def",
                    ReserveTypeShape::QualifiedSymbol("ChainObjectMode1"),
                ),
                mode_definition_with_label(
                    "ChainObjectMode3",
                    "ChainObjectMode3Def",
                    ReserveTypeShape::QualifiedSymbol("ChainObjectMode2"),
                ),
                mode_definition_with_label(
                    "ChainObjectMode4",
                    "ChainObjectMode4Def",
                    ReserveTypeShape::QualifiedSymbol("ChainObjectMode3"),
                ),
                mode_definition_with_label(
                    "ChainObjectMode5",
                    "ChainObjectMode5Def",
                    ReserveTypeShape::QualifiedSymbol("ChainObjectMode4"),
                ),
                mode_definition_with_label(
                    "ChainObjectMode6",
                    "ChainObjectMode6Def",
                    ReserveTypeShape::QualifiedSymbol("ChainObjectMode5"),
                ),
            ]
        };
        let reserve = || {
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("ChainObjectMode6"),
            )]
        };
        let exact = mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            exact_modes(),
            reserve(),
            theorem,
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        let payload = extract_source_local_object_mode_long_chain_reserved_variable_type_assertion(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact object-terminal long-chain type assertion should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 7);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(payload.reserve.bridge.bindings()[0].spelling, "x");
        assert_eq!(
            payload.reserve.bridge.bindings()[0].type_spelling,
            "ChainObjectMode6"
        );
        assert_eq!(payload.asserted_type.spelling, "object");
        assert_eq!(payload.asserted_type.head, TypeHeadInput::BuiltinObject);
        assert_ne!(
            payload.reserve.bridge.bindings()[0].type_range,
            payload.asserted_type.range
        );
        assert_eq!(payload.subject_lookup_ordinal, 1);

        let output = source_local_object_mode_long_chain_reserved_variable_type_assertion_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact object-terminal long-chain type assertion should reach checker");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 179 checked payload invariants should hold");
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.subject_result_input.spelling, "ChainObjectMode6");
        assert!(matches!(
            output.subject_result_input.head,
            TypeHeadInput::Symbol(_)
        ));
        assert_eq!(output.asserted_type_input.spelling, "object");
        assert_eq!(
            output.asserted_type_input.head,
            TypeHeadInput::BuiltinObject
        );
        assert_ne!(
            output.subject_result_input.source_range,
            output.asserted_type_input.source_range
        );
        assert_eq!(output.term_formula.terms().len(), 1);
        let (_, term) = output
            .term_formula
            .terms()
            .iter()
            .next()
            .expect("Task 179 subject should be inferred");
        assert_eq!(term.status, TermStatus::Inferred);
        assert!(term.deferred.is_empty());
        let (_, formula) = output
            .term_formula
            .formulas()
            .iter()
            .next()
            .expect("Task 179 type assertion should be checked");
        assert_eq!(formula.kind, FormulaKind::TypeAssertion);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert!(formula.expected_types.is_empty());
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("BaseObjectMode"))
            .map(|(_, expansion)| &expansion.radix)
            .expect("Task 179 BaseObjectMode terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 179 normalized object type should exist");
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);

        let invalid_key = || {
            vec![
                TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY
                    .to_owned(),
            ]
        };
        for removed in [
            "BaseObjectMode",
            "ChainObjectMode1",
            "ChainObjectMode2",
            "ChainObjectMode3",
            "ChainObjectMode4",
            "ChainObjectMode5",
            "ChainObjectMode6",
        ] {
            let mut invalid =
                source_local_object_mode_long_chain_reserved_variable_type_assertion_output(
                    &exact,
                    module.clone(),
                    &symbols,
                )
                .expect("exact source should produce an expansion corruption target");
            invalid
                .payload
                .reserve
                .mode_expansions
                .retain(|symbol, _| source_mode_symbol_spelling(symbol) != Some(removed));
            let invalid_result =
                assert_source_reserved_variable_type_assertion_output(&invalid).map(|()| invalid);
            assert_eq!(
                source_reserved_variable_type_assertion_result_detail_keys(
                    invalid_result,
                    TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY,
                ),
                invalid_key()
            );
        }
        let mut wrong_asserted =
            source_local_object_mode_long_chain_reserved_variable_type_assertion_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce an asserted-input corruption target");
        wrong_asserted.asserted_type_input.head = TypeHeadInput::BuiltinSet;
        let wrong_asserted_result =
            assert_source_reserved_variable_type_assertion_output(&wrong_asserted)
                .map(|()| wrong_asserted);
        assert_eq!(
            source_reserved_variable_type_assertion_result_detail_keys(
                wrong_asserted_result,
                TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY,
            ),
            invalid_key()
        );
        let mut wrong_subject =
            source_local_object_mode_long_chain_reserved_variable_type_assertion_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce a raw-subject corruption target");
        wrong_subject.subject_result_input.spelling = "object".to_owned();
        let wrong_subject_result =
            assert_source_reserved_variable_type_assertion_output(&wrong_subject)
                .map(|()| wrong_subject);
        assert_eq!(
            source_reserved_variable_type_assertion_result_detail_keys(
                wrong_subject_result,
                TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY,
            ),
            invalid_key()
        );

        let mut wrong_terminal = exact_modes();
        wrong_terminal[0] = mode_definition_with_label(
            "BaseObjectMode",
            "BaseObjectModeDef",
            ReserveTypeShape::Builtin("set"),
        );
        let mut wrong_link = exact_modes();
        wrong_link[3] = mode_definition_with_label(
            "ChainObjectMode3",
            "ChainObjectMode3Def",
            ReserveTypeShape::QualifiedSymbol("BaseObjectMode"),
        );
        let mut argument_bearing_link = exact_modes();
        argument_bearing_link[6] = mode_definition_with_label(
            "ChainObjectMode6",
            "ChainObjectMode6Def",
            ReserveTypeShape::QualifiedSymbolWithArgs("ChainObjectMode5"),
        );
        for near_miss in [
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                wrong_terminal,
                reserve(),
                theorem,
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                wrong_link,
                reserve(),
                theorem,
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                argument_bearing_link,
                reserve(),
                theorem,
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbolWithArgs("ChainObjectMode6"),
                )],
                theorem,
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    label: "OtherPayloadBoundary",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    subject: "y",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::Builtin("set"),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol("ChainObjectMode6"),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    negated: true,
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    status: Some("open"),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    recovered_label: true,
                    ..theorem
                },
            ),
            modes_then_empty_definition_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                theorem,
            ),
        ] {
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
        let unresolved_symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module, &unresolved_symbols),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );
    }
    #[test]
    fn active_local_object_mode_long_chain_reserved_variable_type_assertion_fixture_consumes_seven_expansions()
     {
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("mizar-test crate should live below the workspace root")
            .to_path_buf();
        let config = DiscoveryConfig {
            workspace_root: workspace_root.clone(),
            tests_root: workspace_root.join("tests"),
            manifest_path: workspace_root.join("tests/coverage/spec_trace.toml"),
            profile: TestProfile::Fast,
            validation_mode: ValidationMode::Metadata,
        };
        let plan = build_test_plan(&config).expect("repository test plan should build");
        let (ordinal, case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| {
                case.id.0
                    == "pass_type_elaboration_local_object_mode_long_chain_reserved_variable_type_assertion_001"
            })
            .expect("Task 179 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 179 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 179 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_local_object_mode_long_chain_reserved_variable_type_assertion_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 179 real AST should reach long-chain object type assertion");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 179 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.subject_result_input.spelling, "ChainObjectMode6");
        assert!(matches!(
            output.subject_result_input.head,
            TypeHeadInput::Symbol(_)
        ));
        assert_eq!(output.asserted_type_input.spelling, "object");
        assert_eq!(
            output.asserted_type_input.head,
            TypeHeadInput::BuiltinObject
        );
        assert_ne!(
            output.subject_result_input.source_range,
            output.asserted_type_input.source_range
        );
        let (_, formula) = output
            .term_formula
            .formulas()
            .iter()
            .next()
            .expect("Task 179 type assertion should be checked");
        assert_eq!(formula.kind, FormulaKind::TypeAssertion);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert!(formula.expected_types.is_empty());
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("BaseObjectMode"))
            .map(|(_, expansion)| &expansion.radix)
            .expect("Task 179 BaseObjectMode terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 179 normalized object type should exist");
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
    }
    #[test]
    fn source_local_object_mode_long_chain_reserved_variable_equality_consumes_seven_expansions() {
        let source_id = source_id(176);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("local_object_mode_long_chain_reserved_variable_equality"),
        );
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseObjectMode", SymbolKind::Mode),
                ("ChainObjectMode1", SymbolKind::Mode),
                ("ChainObjectMode2", SymbolKind::Mode),
                ("ChainObjectMode3", SymbolKind::Mode),
                ("ChainObjectMode4", SymbolKind::Mode),
                ("ChainObjectMode5", SymbolKind::Mode),
                ("ChainObjectMode6", SymbolKind::Mode),
                ("ExtraObjectMode", SymbolKind::Mode),
            ],
        );
        let theorem = IdentifierBinaryTheoremSpec {
            status: None,
            label: "LongLocalObjectModeReservedVariableEqualityPayloadBoundary",
            left: "z",
            operator: "=",
            right: "z",
            recovered_label: false,
        };
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    "BaseObjectMode",
                    "BaseObjectModeDef",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition_with_label(
                    "ChainObjectMode1",
                    "ChainObjectMode1Def",
                    ReserveTypeShape::QualifiedSymbol("BaseObjectMode"),
                ),
                mode_definition_with_label(
                    "ChainObjectMode2",
                    "ChainObjectMode2Def",
                    ReserveTypeShape::QualifiedSymbol("ChainObjectMode1"),
                ),
                mode_definition_with_label(
                    "ChainObjectMode3",
                    "ChainObjectMode3Def",
                    ReserveTypeShape::QualifiedSymbol("ChainObjectMode2"),
                ),
                mode_definition_with_label(
                    "ChainObjectMode4",
                    "ChainObjectMode4Def",
                    ReserveTypeShape::QualifiedSymbol("ChainObjectMode3"),
                ),
                mode_definition_with_label(
                    "ChainObjectMode5",
                    "ChainObjectMode5Def",
                    ReserveTypeShape::QualifiedSymbol("ChainObjectMode4"),
                ),
                mode_definition_with_label(
                    "ChainObjectMode6",
                    "ChainObjectMode6Def",
                    ReserveTypeShape::QualifiedSymbol("ChainObjectMode5"),
                ),
            ]
        };
        let reserve = || {
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("ChainObjectMode6"),
            )]
        };
        let exact = mode_then_reserve_identifier_binary_theorem_ast(
            source_id,
            exact_modes(),
            reserve(),
            theorem,
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        let payload = extract_source_local_object_mode_long_chain_reserved_variable_equality(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact local-object-mode long-chain equality should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 7);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(payload.reserve.bridge.bindings()[0].spelling, "z");
        assert_eq!(
            payload.reserve.bridge.bindings()[0].type_spelling,
            "ChainObjectMode6"
        );
        assert_eq!(payload.left_lookup_ordinal, 1);
        assert_eq!(payload.right_lookup_ordinal, 2);

        let output = source_local_object_mode_long_chain_reserved_variable_equality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact long-chain object equality should reach TermFormulaChecker");
        assert_source_reserved_variable_formula_output(&output)
            .expect("long-chain object equality invariants should hold");
        assert_eq!(output.left_binding, BindingId::new(0));
        assert_eq!(output.right_binding, BindingId::new(0));
        for input in [
            &output.left_result_input,
            &output.right_result_input,
            output
                .left_expected_input
                .as_ref()
                .expect("left expected input should exist"),
            output
                .right_expected_input
                .as_ref()
                .expect("right expected input should exist"),
        ] {
            assert_eq!(input.spelling, "ChainObjectMode6");
            assert!(matches!(input.head, TypeHeadInput::Symbol(_)));
            assert_eq!(
                input.source_range,
                output.payload.reserve.bridge.bindings()[0].type_range
            );
        }
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("BaseObjectMode"))
            .map(|(_, expansion)| &expansion.radix)
            .expect("BaseObjectMode terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("one normalized terminal object should exist");
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
        assert_ne!(
            output.payload.reserve.bridge.bindings()[0].type_range,
            terminal.source_range
        );
        assert_eq!(output.term_formula.terms().len(), 2);
        let (_, formula) = output
            .term_formula
            .formulas()
            .iter()
            .next()
            .expect("long-chain object equality should be checked");
        assert_eq!(formula.kind, FormulaKind::Equality);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert_eq!(formula.expected_types.len(), 2);
        assert_eq!(
            formula.expected_types[0].source_range,
            output.payload.left_range
        );
        assert_eq!(
            formula.expected_types[1].source_range,
            output.payload.right_range
        );
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
        assert!(output.term_formula.facts().is_empty());
        assert!(output.term_formula.diagnostics().is_empty());

        let invalid_key = || {
            vec![
                TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY
                    .to_owned(),
            ]
        };
        for removed in [
            "BaseObjectMode",
            "ChainObjectMode1",
            "ChainObjectMode2",
            "ChainObjectMode3",
            "ChainObjectMode4",
            "ChainObjectMode5",
            "ChainObjectMode6",
        ] {
            let mut invalid =
                source_local_object_mode_long_chain_reserved_variable_equality_output(
                    &exact,
                    module.clone(),
                    &symbols,
                )
                .expect("exact source should produce a seven-expansion corruption target");
            invalid
                .payload
                .reserve
                .mode_expansions
                .retain(|symbol, _| source_mode_symbol_spelling(symbol) != Some(removed));
            assert_eq!(
                source_reserved_variable_formula_output_detail_keys(&invalid),
                invalid_key()
            );
        }
        for clear_left in [true, false] {
            let mut invalid =
                source_local_object_mode_long_chain_reserved_variable_equality_output(
                    &exact,
                    module.clone(),
                    &symbols,
                )
                .expect("exact source should produce an expected-role corruption target");
            if clear_left {
                invalid.left_expected_input = None;
            } else {
                invalid.right_expected_input = None;
            }
            assert_eq!(
                source_reserved_variable_formula_output_detail_keys(&invalid),
                invalid_key()
            );
        }
        let mut coerced = source_local_object_mode_long_chain_reserved_variable_equality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce an object/set corruption target");
        coerced
            .left_expected_input
            .as_mut()
            .expect("left expected input should exist")
            .head = TypeHeadInput::BuiltinSet;
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&coerced),
            invalid_key()
        );

        let assert_extraction_gap = |modes| {
            let near_miss = mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                modes,
                reserve(),
                theorem,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        };
        let mut set_terminal = exact_modes();
        set_terminal[0].rhs_shape = ReserveTypeShape::Builtin("set");
        assert_extraction_gap(set_terminal);
        let mut wrong_link = exact_modes();
        wrong_link[6].rhs_shape = ReserveTypeShape::QualifiedSymbol("ChainObjectMode4");
        assert_extraction_gap(wrong_link);
        let mut argument_bearing = exact_modes();
        argument_bearing[6].rhs_shape =
            ReserveTypeShape::QualifiedSymbolWithArgs("ChainObjectMode5");
        assert_extraction_gap(argument_bearing);

        for near_miss in [
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(
                    vec!["z"],
                    ReserveTypeShape::QualifiedSymbolWithArgs("ChainObjectMode6"),
                )],
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierBinaryTheoremSpec {
                    label: "OtherPayloadBoundary",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierBinaryTheoremSpec {
                    left: "y",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierBinaryTheoremSpec {
                    right: "y",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierBinaryTheoremSpec {
                    operator: "<>",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierBinaryTheoremSpec {
                    status: Some("open"),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierBinaryTheoremSpec {
                    recovered_label: true,
                    ..theorem
                },
            ),
            modes_then_empty_definition_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                theorem,
            ),
        ] {
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
        let unresolved_symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module, &unresolved_symbols),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );
    }
    #[test]
    fn active_local_object_mode_long_chain_reserved_variable_equality_fixture_consumes_seven_expansions()
     {
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("mizar-test crate should live below the workspace root")
            .to_path_buf();
        let config = DiscoveryConfig {
            workspace_root: workspace_root.clone(),
            tests_root: workspace_root.join("tests"),
            manifest_path: workspace_root.join("tests/coverage/spec_trace.toml"),
            profile: TestProfile::Fast,
            validation_mode: ValidationMode::Metadata,
        };
        let plan = build_test_plan(&config).expect("repository test plan should build");
        let (ordinal, case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| {
                case.id.0
                    == "pass_type_elaboration_local_object_mode_long_chain_reserved_variable_equality_001"
            })
            .expect("Task 176 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 176 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 176 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_local_object_mode_long_chain_reserved_variable_equality_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 176 real AST should reach long-chain object equality");
        assert_source_reserved_variable_formula_output(&output)
            .expect("Task 176 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
        assert_eq!(output.left_binding, BindingId::new(0));
        assert_eq!(output.right_binding, BindingId::new(0));
        assert_eq!(output.payload.left_lookup_ordinal, 1);
        assert_eq!(output.payload.right_lookup_ordinal, 2);
        for input in [
            &output.left_result_input,
            &output.right_result_input,
            output.left_expected_input.as_ref().unwrap(),
            output.right_expected_input.as_ref().unwrap(),
        ] {
            assert_eq!(input.spelling, "ChainObjectMode6");
            assert!(matches!(input.head, TypeHeadInput::Symbol(_)));
        }
        let (_, formula) = output
            .term_formula
            .formulas()
            .iter()
            .next()
            .expect("Task 176 equality should be checked");
        assert_eq!(formula.kind, FormulaKind::Equality);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert_eq!(formula.expected_types.len(), 2);
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("BaseObjectMode"))
            .map(|(_, expansion)| &expansion.radix)
            .expect("Task 176 BaseObjectMode terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 176 normalized object type should exist");
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
    }
    #[test]
    fn source_local_object_mode_long_chain_reserved_variable_inequality_consumes_seven_expansions()
    {
        let source_id = source_id(177);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("local_object_mode_long_chain_reserved_variable_inequality"),
        );
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseObjectMode", SymbolKind::Mode),
                ("ChainObjectMode1", SymbolKind::Mode),
                ("ChainObjectMode2", SymbolKind::Mode),
                ("ChainObjectMode3", SymbolKind::Mode),
                ("ChainObjectMode4", SymbolKind::Mode),
                ("ChainObjectMode5", SymbolKind::Mode),
                ("ChainObjectMode6", SymbolKind::Mode),
                ("ExtraObjectMode", SymbolKind::Mode),
            ],
        );
        let theorem = IdentifierBinaryTheoremSpec {
            status: None,
            label: "LongLocalObjectModeReservedVariableInequalityPayloadBoundary",
            left: "z",
            operator: "<>",
            right: "z",
            recovered_label: false,
        };
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    "BaseObjectMode",
                    "BaseObjectModeDef",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition_with_label(
                    "ChainObjectMode1",
                    "ChainObjectMode1Def",
                    ReserveTypeShape::QualifiedSymbol("BaseObjectMode"),
                ),
                mode_definition_with_label(
                    "ChainObjectMode2",
                    "ChainObjectMode2Def",
                    ReserveTypeShape::QualifiedSymbol("ChainObjectMode1"),
                ),
                mode_definition_with_label(
                    "ChainObjectMode3",
                    "ChainObjectMode3Def",
                    ReserveTypeShape::QualifiedSymbol("ChainObjectMode2"),
                ),
                mode_definition_with_label(
                    "ChainObjectMode4",
                    "ChainObjectMode4Def",
                    ReserveTypeShape::QualifiedSymbol("ChainObjectMode3"),
                ),
                mode_definition_with_label(
                    "ChainObjectMode5",
                    "ChainObjectMode5Def",
                    ReserveTypeShape::QualifiedSymbol("ChainObjectMode4"),
                ),
                mode_definition_with_label(
                    "ChainObjectMode6",
                    "ChainObjectMode6Def",
                    ReserveTypeShape::QualifiedSymbol("ChainObjectMode5"),
                ),
            ]
        };
        let reserve = || {
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("ChainObjectMode6"),
            )]
        };
        let exact = mode_then_reserve_identifier_binary_theorem_ast(
            source_id,
            exact_modes(),
            reserve(),
            theorem,
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        let payload = extract_source_local_object_mode_long_chain_reserved_variable_inequality(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact local-object-mode long-chain inequality should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 7);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(payload.reserve.bridge.bindings()[0].spelling, "z");
        assert_eq!(
            payload.reserve.bridge.bindings()[0].type_spelling,
            "ChainObjectMode6"
        );
        assert_eq!(payload.left_lookup_ordinal, 1);
        assert_eq!(payload.right_lookup_ordinal, 2);

        let output = source_local_object_mode_long_chain_reserved_variable_inequality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact long-chain object inequality should reach TermFormulaChecker");
        assert_source_reserved_variable_formula_output(&output)
            .expect("long-chain object inequality invariants should hold");
        assert_eq!(output.left_binding, BindingId::new(0));
        assert_eq!(output.right_binding, BindingId::new(0));
        for input in [
            &output.left_result_input,
            &output.right_result_input,
            output
                .left_expected_input
                .as_ref()
                .expect("left expected input should exist"),
            output
                .right_expected_input
                .as_ref()
                .expect("right expected input should exist"),
        ] {
            assert_eq!(input.spelling, "ChainObjectMode6");
            assert!(matches!(input.head, TypeHeadInput::Symbol(_)));
            assert_eq!(
                input.source_range,
                output.payload.reserve.bridge.bindings()[0].type_range
            );
        }
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("BaseObjectMode"))
            .map(|(_, expansion)| &expansion.radix)
            .expect("BaseObjectMode terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("one normalized terminal object should exist");
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
        assert_ne!(
            output.payload.reserve.bridge.bindings()[0].type_range,
            terminal.source_range
        );
        assert_eq!(output.term_formula.terms().len(), 2);
        let (_, formula) = output
            .term_formula
            .formulas()
            .iter()
            .next()
            .expect("long-chain object inequality should be checked");
        assert_eq!(formula.kind, FormulaKind::Inequality);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert_eq!(formula.expected_types.len(), 2);
        assert_eq!(
            formula.expected_types[0].source_range,
            output.payload.left_range
        );
        assert_eq!(
            formula.expected_types[1].source_range,
            output.payload.right_range
        );
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
        assert!(output.term_formula.facts().is_empty());
        assert!(output.term_formula.diagnostics().is_empty());

        let invalid_key = || {
            vec![
                TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY
                    .to_owned(),
            ]
        };
        for removed in [
            "BaseObjectMode",
            "ChainObjectMode1",
            "ChainObjectMode2",
            "ChainObjectMode3",
            "ChainObjectMode4",
            "ChainObjectMode5",
            "ChainObjectMode6",
        ] {
            let mut invalid =
                source_local_object_mode_long_chain_reserved_variable_inequality_output(
                    &exact,
                    module.clone(),
                    &symbols,
                )
                .expect("exact source should produce a seven-expansion corruption target");
            invalid
                .payload
                .reserve
                .mode_expansions
                .retain(|symbol, _| source_mode_symbol_spelling(symbol) != Some(removed));
            assert_eq!(
                source_reserved_variable_formula_output_detail_keys(&invalid),
                invalid_key()
            );
        }
        for clear_left in [true, false] {
            let mut invalid =
                source_local_object_mode_long_chain_reserved_variable_inequality_output(
                    &exact,
                    module.clone(),
                    &symbols,
                )
                .expect("exact source should produce an expected-role corruption target");
            if clear_left {
                invalid.left_expected_input = None;
            } else {
                invalid.right_expected_input = None;
            }
            assert_eq!(
                source_reserved_variable_formula_output_detail_keys(&invalid),
                invalid_key()
            );
        }
        let mut coerced = source_local_object_mode_long_chain_reserved_variable_inequality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce an object/set corruption target");
        coerced
            .left_expected_input
            .as_mut()
            .expect("left expected input should exist")
            .head = TypeHeadInput::BuiltinSet;
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&coerced),
            invalid_key()
        );

        let assert_extraction_gap = |modes| {
            let near_miss = mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                modes,
                reserve(),
                theorem,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        };
        let mut set_terminal = exact_modes();
        set_terminal[0].rhs_shape = ReserveTypeShape::Builtin("set");
        assert_extraction_gap(set_terminal);
        let mut wrong_link = exact_modes();
        wrong_link[6].rhs_shape = ReserveTypeShape::QualifiedSymbol("ChainObjectMode4");
        assert_extraction_gap(wrong_link);
        let mut argument_bearing = exact_modes();
        argument_bearing[6].rhs_shape =
            ReserveTypeShape::QualifiedSymbolWithArgs("ChainObjectMode5");
        assert_extraction_gap(argument_bearing);

        for near_miss in [
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(
                    vec!["z"],
                    ReserveTypeShape::QualifiedSymbolWithArgs("ChainObjectMode6"),
                )],
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierBinaryTheoremSpec {
                    label: "OtherPayloadBoundary",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierBinaryTheoremSpec {
                    left: "y",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierBinaryTheoremSpec {
                    right: "y",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierBinaryTheoremSpec {
                    operator: "=",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierBinaryTheoremSpec {
                    status: Some("open"),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierBinaryTheoremSpec {
                    recovered_label: true,
                    ..theorem
                },
            ),
            modes_then_empty_definition_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                theorem,
            ),
        ] {
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
        let unresolved_symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module, &unresolved_symbols),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );
    }
    #[test]
    fn active_local_object_mode_long_chain_reserved_variable_inequality_fixture_consumes_seven_expansions()
     {
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("mizar-test crate should live below the workspace root")
            .to_path_buf();
        let config = DiscoveryConfig {
            workspace_root: workspace_root.clone(),
            tests_root: workspace_root.join("tests"),
            manifest_path: workspace_root.join("tests/coverage/spec_trace.toml"),
            profile: TestProfile::Fast,
            validation_mode: ValidationMode::Metadata,
        };
        let plan = build_test_plan(&config).expect("repository test plan should build");
        let (ordinal, case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| {
                case.id.0
                    == "pass_type_elaboration_local_object_mode_long_chain_reserved_variable_inequality_001"
            })
            .expect("Task 177 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 177 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 177 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_local_object_mode_long_chain_reserved_variable_inequality_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 177 real AST should reach long-chain object inequality");
        assert_source_reserved_variable_formula_output(&output)
            .expect("Task 177 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
        assert_eq!(output.left_binding, BindingId::new(0));
        assert_eq!(output.right_binding, BindingId::new(0));
        assert_eq!(output.payload.left_lookup_ordinal, 1);
        assert_eq!(output.payload.right_lookup_ordinal, 2);
        for input in [
            &output.left_result_input,
            &output.right_result_input,
            output.left_expected_input.as_ref().unwrap(),
            output.right_expected_input.as_ref().unwrap(),
        ] {
            assert_eq!(input.spelling, "ChainObjectMode6");
            assert!(matches!(input.head, TypeHeadInput::Symbol(_)));
        }
        let (_, formula) = output
            .term_formula
            .formulas()
            .iter()
            .next()
            .expect("Task 177 inequality should be checked");
        assert_eq!(formula.kind, FormulaKind::Inequality);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert_eq!(formula.expected_types.len(), 2);
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("BaseObjectMode"))
            .map(|(_, expansion)| &expansion.radix)
            .expect("Task 177 BaseObjectMode terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 177 normalized object type should exist");
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
    }
    #[test]
    fn source_local_object_mode_long_chain_reserved_variable_membership_consumes_seven_expansions()
    {
        let source_id = source_id(178);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("local_object_mode_long_chain_reserved_variable_membership"),
        );
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseObjectMode", SymbolKind::Mode),
                ("ChainObjectMode1", SymbolKind::Mode),
                ("ChainObjectMode2", SymbolKind::Mode),
                ("ChainObjectMode3", SymbolKind::Mode),
                ("ChainObjectMode4", SymbolKind::Mode),
                ("ChainObjectMode5", SymbolKind::Mode),
                ("ChainObjectMode6", SymbolKind::Mode),
                ("ExtraObjectMode", SymbolKind::Mode),
            ],
        );
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    "BaseObjectMode",
                    "BaseObjectModeDef",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition_with_label(
                    "ChainObjectMode1",
                    "ChainObjectMode1Def",
                    ReserveTypeShape::QualifiedSymbol("BaseObjectMode"),
                ),
                mode_definition_with_label(
                    "ChainObjectMode2",
                    "ChainObjectMode2Def",
                    ReserveTypeShape::QualifiedSymbol("ChainObjectMode1"),
                ),
                mode_definition_with_label(
                    "ChainObjectMode3",
                    "ChainObjectMode3Def",
                    ReserveTypeShape::QualifiedSymbol("ChainObjectMode2"),
                ),
                mode_definition_with_label(
                    "ChainObjectMode4",
                    "ChainObjectMode4Def",
                    ReserveTypeShape::QualifiedSymbol("ChainObjectMode3"),
                ),
                mode_definition_with_label(
                    "ChainObjectMode5",
                    "ChainObjectMode5Def",
                    ReserveTypeShape::QualifiedSymbol("ChainObjectMode4"),
                ),
                mode_definition_with_label(
                    "ChainObjectMode6",
                    "ChainObjectMode6Def",
                    ReserveTypeShape::QualifiedSymbol("ChainObjectMode5"),
                ),
            ]
        };
        let reserves = || {
            vec![
                reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbol("ChainObjectMode6"),
                ),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
            ]
        };
        let theorem = IdentifierBinaryTheoremSpec {
            status: None,
            label: "LongLocalObjectModeReservedVariableMembershipPayloadBoundary",
            left: "x",
            operator: "in",
            right: "y",
            recovered_label: false,
        };
        let exact = mode_then_reserve_identifier_binary_theorem_ast(
            source_id,
            exact_modes(),
            reserves(),
            theorem,
        );
        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );

        let payload = extract_source_local_object_mode_long_chain_reserved_variable_membership(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact long-chain object membership should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 7);
        assert_eq!(payload.reserve.bridge.bindings().len(), 2);
        assert_eq!(payload.reserve.bridge.bindings()[0].spelling, "x");
        assert_eq!(
            payload.reserve.bridge.bindings()[0].type_spelling,
            "ChainObjectMode6"
        );
        assert_eq!(payload.reserve.bridge.bindings()[1].spelling, "y");
        assert_eq!(payload.reserve.bridge.bindings()[1].type_spelling, "set");
        assert_ne!(
            payload.reserve.bridge.bindings()[0].type_range,
            payload.reserve.bridge.bindings()[1].type_range
        );
        assert_eq!(payload.left_lookup_ordinal, 2);
        assert_eq!(payload.right_lookup_ordinal, 3);

        let output = source_local_object_mode_long_chain_reserved_variable_membership_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact long-chain object membership should reach TermFormulaChecker");
        assert_source_reserved_variable_formula_output(&output)
            .expect("long-chain object membership invariants should hold");
        assert_eq!(output.left_binding, BindingId::new(0));
        assert_eq!(output.right_binding, BindingId::new(1));
        assert_eq!(output.left_result_input.spelling, "ChainObjectMode6");
        assert!(matches!(
            output.left_result_input.head,
            TypeHeadInput::Symbol(_)
        ));
        assert_eq!(
            output.left_result_input.source_range,
            output.payload.reserve.bridge.bindings()[0].type_range
        );
        assert!(output.left_expected_input.is_none());
        assert_eq!(output.right_result_input.spelling, "set");
        assert_eq!(output.right_result_input.head, TypeHeadInput::BuiltinSet);
        let right_expected = output
            .right_expected_input
            .as_ref()
            .expect("right expected-set input should exist");
        assert_eq!(right_expected.spelling, "set");
        assert_eq!(right_expected.head, TypeHeadInput::BuiltinSet);
        assert_eq!(
            output.right_result_input.source_range,
            output.payload.reserve.bridge.bindings()[1].type_range
        );
        assert_eq!(
            right_expected.source_range,
            output.payload.reserve.bridge.bindings()[1].type_range
        );
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("BaseObjectMode"))
            .map(|(_, expansion)| &expansion.radix)
            .expect("BaseObjectMode terminal expansion should exist");
        assert_eq!(output.term_formula.normalized_types().len(), 2);
        let (_, normalized_object) = output
            .term_formula
            .normalized_types()
            .iter()
            .find(|(_, normalized)| normalized.head == TypeHeadRef::BuiltinObject)
            .expect("normalized terminal object type should exist");
        assert_eq!(normalized_object.source.range, terminal.source_range);
        assert_eq!(normalized_object.source.spelling, terminal.spelling);
        let (_, normalized_set) = output
            .term_formula
            .normalized_types()
            .iter()
            .find(|(_, normalized)| normalized.head == TypeHeadRef::BuiltinSet)
            .expect("normalized explicit-set type should exist");
        assert_eq!(
            normalized_set.source.range,
            output.right_result_input.source_range
        );
        assert_eq!(normalized_set.source.spelling, "set");
        assert_eq!(output.term_formula.terms().len(), 2);
        for (_, term) in output.term_formula.terms().iter() {
            assert_eq!(term.kind, TermKind::Variable);
            assert_eq!(term.status, TermStatus::Inferred);
        }
        let (_, formula) = output
            .term_formula
            .formulas()
            .iter()
            .next()
            .expect("long-chain object membership should be checked");
        assert_eq!(formula.kind, FormulaKind::Membership);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert_eq!(formula.expected_types.len(), 1);
        assert_eq!(formula.expected_types[0].term, output.payload.right_site);
        assert_eq!(
            formula.expected_types[0].source_range,
            output.payload.right_range
        );
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
        assert!(output.term_formula.facts().is_empty());
        assert!(output.term_formula.diagnostics().is_empty());
        let type_roles = output
            .term_formula
            .type_entries()
            .iter()
            .filter_map(|(_, entry)| match &entry.owner {
                TypedSiteRef::Role { role, .. } => Some(role.as_str().to_owned()),
                _ => None,
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(
            type_roles,
            BTreeSet::from([
                "long-local-object-mode-reserved-variable-membership-left-result".to_owned(),
                "long-local-object-mode-reserved-variable-membership-right-expected".to_owned(),
                "long-local-object-mode-reserved-variable-membership-right-result".to_owned(),
            ])
        );

        let invalid_key = || {
            vec![
                TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY
                    .to_owned(),
            ]
        };
        for removed in [
            "BaseObjectMode",
            "ChainObjectMode1",
            "ChainObjectMode2",
            "ChainObjectMode3",
            "ChainObjectMode4",
            "ChainObjectMode5",
            "ChainObjectMode6",
        ] {
            let mut invalid =
                source_local_object_mode_long_chain_reserved_variable_membership_output(
                    &exact,
                    module.clone(),
                    &symbols,
                )
                .expect("exact source should produce an expansion corruption target");
            invalid
                .payload
                .reserve
                .mode_expansions
                .retain(|symbol, _| source_mode_symbol_spelling(symbol) != Some(removed));
            assert_eq!(
                source_reserved_variable_formula_output_detail_keys(&invalid),
                invalid_key()
            );
        }
        let mut unexpected_left =
            source_local_object_mode_long_chain_reserved_variable_membership_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce a left-expected corruption target");
        unexpected_left.left_expected_input = unexpected_left.right_expected_input.clone();
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&unexpected_left),
            invalid_key()
        );
        let mut missing_right =
            source_local_object_mode_long_chain_reserved_variable_membership_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce a missing-right corruption target");
        missing_right.right_expected_input = None;
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&missing_right),
            invalid_key()
        );
        let mut wrong_right =
            source_local_object_mode_long_chain_reserved_variable_membership_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce a right-type corruption target");
        wrong_right
            .right_expected_input
            .as_mut()
            .expect("right expected input should exist")
            .head = TypeHeadInput::BuiltinObject;
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&wrong_right),
            invalid_key()
        );

        let assert_extraction_gap = |modes| {
            let near_miss = mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                modes,
                reserves(),
                theorem,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        };
        let mut set_terminal = exact_modes();
        set_terminal[0].rhs_shape = ReserveTypeShape::Builtin("set");
        assert_extraction_gap(set_terminal);
        let mut wrong_link = exact_modes();
        wrong_link[6].rhs_shape = ReserveTypeShape::QualifiedSymbol("ChainObjectMode4");
        assert_extraction_gap(wrong_link);
        let mut argument_bearing = exact_modes();
        argument_bearing[6].rhs_shape =
            ReserveTypeShape::QualifiedSymbolWithArgs("ChainObjectMode5");
        assert_extraction_gap(argument_bearing);

        for near_miss in [
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                vec![
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                    reserve_item(
                        vec!["x"],
                        ReserveTypeShape::QualifiedSymbol("ChainObjectMode6"),
                    ),
                ],
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                vec![
                    reserve_item(
                        vec!["x"],
                        ReserveTypeShape::QualifiedSymbolWithArgs("ChainObjectMode6"),
                    ),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                ],
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                reserves(),
                IdentifierBinaryTheoremSpec {
                    label: "OtherPayloadBoundary",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                reserves(),
                IdentifierBinaryTheoremSpec {
                    left: "y",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                reserves(),
                IdentifierBinaryTheoremSpec {
                    right: "x",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                reserves(),
                IdentifierBinaryTheoremSpec {
                    operator: "=",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                reserves(),
                IdentifierBinaryTheoremSpec {
                    status: Some("open"),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                reserves(),
                IdentifierBinaryTheoremSpec {
                    recovered_label: true,
                    ..theorem
                },
            ),
            modes_then_empty_definition_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                reserves(),
                theorem,
            ),
        ] {
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
        let unresolved_symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module, &unresolved_symbols),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );
    }
    #[test]
    fn active_local_object_mode_long_chain_reserved_variable_membership_fixture_consumes_seven_expansions()
     {
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("mizar-test crate should live below the workspace root")
            .to_path_buf();
        let config = DiscoveryConfig {
            workspace_root: workspace_root.clone(),
            tests_root: workspace_root.join("tests"),
            manifest_path: workspace_root.join("tests/coverage/spec_trace.toml"),
            profile: TestProfile::Fast,
            validation_mode: ValidationMode::Metadata,
        };
        let plan = build_test_plan(&config).expect("repository test plan should build");
        let (ordinal, case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| {
                case.id.0
                    == "pass_type_elaboration_local_object_mode_long_chain_reserved_variable_membership_001"
            })
            .expect("Task 178 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 178 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 178 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_local_object_mode_long_chain_reserved_variable_membership_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 178 real AST should reach long-chain object membership");
        assert_source_reserved_variable_formula_output(&output)
            .expect("Task 178 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
        assert_eq!(output.left_binding, BindingId::new(0));
        assert_eq!(output.right_binding, BindingId::new(1));
        assert_eq!(output.payload.left_lookup_ordinal, 2);
        assert_eq!(output.payload.right_lookup_ordinal, 3);
        assert_eq!(output.left_result_input.spelling, "ChainObjectMode6");
        assert!(output.left_expected_input.is_none());
        assert_eq!(output.right_result_input.head, TypeHeadInput::BuiltinSet);
        assert_eq!(
            output.right_expected_input.as_ref().unwrap().head,
            TypeHeadInput::BuiltinSet
        );
        let (_, formula) = output
            .term_formula
            .formulas()
            .iter()
            .next()
            .expect("Task 178 membership formula should be checked");
        assert_eq!(formula.kind, FormulaKind::Membership);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert_eq!(formula.expected_types.len(), 1);
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
        assert_eq!(output.term_formula.normalized_types().len(), 2);
        assert!(
            output
                .term_formula
                .normalized_types()
                .iter()
                .any(|(_, ty)| ty.head == TypeHeadRef::BuiltinObject)
        );
        assert!(
            output
                .term_formula
                .normalized_types()
                .iter()
                .any(|(_, ty)| ty.head == TypeHeadRef::BuiltinSet)
        );
    }
