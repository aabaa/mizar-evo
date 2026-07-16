    include!("tests/support.rs");

    include!("tests/parse_only.rs");

    include!("tests/type_elaboration/source_extraction.rs");

    include!("tests/type_elaboration/reserved_binary.rs");

    include!("tests/type_elaboration/mode_chain.rs");

    include!("tests/type_elaboration/reserved_direct.rs");

    include!("tests/type_elaboration/asserted_head_base.rs");

    include!("tests/type_elaboration/asserted_head_four_edge_radix.rs");

    include!("tests/type_elaboration/asserted_head_three_edge_object_radix.rs");

    include!("tests/type_elaboration/asserted_head_two_edge_object_radix.rs");

    include!("tests/type_elaboration/asserted_head_type_assertion.rs");

    include!("tests/type_elaboration/binary_route_fixtures.rs");

    include!("tests/type_elaboration/reserve_object_fixtures.rs");

    include!("tests/type_elaboration/formula_constant_fixture.rs");

    include!("tests/type_elaboration/reserve_fixtures.rs");

    include!("tests/type_elaboration/mode_chain_fixtures.rs");

    include!("tests/type_elaboration/asserted_head_fixtures.rs");

    include!("tests/type_elaboration/source_gap_and_equality.rs");
    include!("tests/type_elaboration/long_chain.rs");
    #[test]
    fn source_four_edge_local_mode_reserved_variable_inequality_consumes_five_expansions() {
        let source_id = source_id(168);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("four_edge_local_mode_reserved_variable_inequality"),
        );
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseFourEdgeModeInequality", SymbolKind::Mode),
                ("InnerFourEdgeModeInequality", SymbolKind::Mode),
                ("MiddleFourEdgeModeInequality", SymbolKind::Mode),
                ("OuterFourEdgeModeInequality", SymbolKind::Mode),
                ("TooDeepFourEdgeModeInequality", SymbolKind::Mode),
                ("ExtraFourEdgeModeInequality", SymbolKind::Mode),
            ],
        );
        let theorem = IdentifierBinaryTheoremSpec {
            status: None,
            label: "FourEdgeLocalModeReservedVariableInequalityPayloadBoundary",
            left: "z",
            operator: "<>",
            right: "z",
            recovered_label: false,
        };
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    "BaseFourEdgeModeInequality",
                    "BaseFourEdgeModeInequalityDef",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition_with_label(
                    "InnerFourEdgeModeInequality",
                    "InnerFourEdgeModeInequalityDef",
                    ReserveTypeShape::QualifiedSymbol("BaseFourEdgeModeInequality"),
                ),
                mode_definition_with_label(
                    "MiddleFourEdgeModeInequality",
                    "MiddleFourEdgeModeInequalityDef",
                    ReserveTypeShape::QualifiedSymbol("InnerFourEdgeModeInequality"),
                ),
                mode_definition_with_label(
                    "OuterFourEdgeModeInequality",
                    "OuterFourEdgeModeInequalityDef",
                    ReserveTypeShape::QualifiedSymbol("MiddleFourEdgeModeInequality"),
                ),
                mode_definition_with_label(
                    "TooDeepFourEdgeModeInequality",
                    "TooDeepFourEdgeModeInequalityDef",
                    ReserveTypeShape::QualifiedSymbol("OuterFourEdgeModeInequality"),
                ),
            ]
        };
        let reserve = || {
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("TooDeepFourEdgeModeInequality"),
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
        let payload = extract_source_four_edge_local_mode_reserved_variable_inequality(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact four-edge local-mode reserved-variable inequality should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 5);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(payload.reserve.bridge.bindings()[0].spelling, "z");
        assert_eq!(
            payload.reserve.bridge.bindings()[0].type_spelling,
            "TooDeepFourEdgeModeInequality"
        );
        assert_eq!(payload.left_lookup_ordinal, 1);
        assert_eq!(payload.right_lookup_ordinal, 2);

        let output = source_four_edge_local_mode_reserved_variable_inequality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact four-edge local-mode inequality should reach TermFormulaChecker");
        assert_source_reserved_variable_formula_output(&output)
            .expect("four-edge local-mode inequality invariants should hold");
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
            assert_eq!(input.spelling, "TooDeepFourEdgeModeInequality");
            assert!(matches!(input.head, TypeHeadInput::Symbol(_)));
            assert_eq!(
                input.source_range,
                payload.reserve.bridge.bindings()[0].type_range
            );
        }
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("BaseFourEdgeModeInequality")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("base mode terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("one normalized terminal type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
        assert_ne!(
            payload.reserve.bridge.bindings()[0].type_range,
            terminal.source_range
        );
        assert_eq!(output.term_formula.terms().len(), 2);
        for (_, term) in output.term_formula.terms().iter() {
            assert_eq!(term.kind, TermKind::Variable);
            assert_eq!(
                term.reference,
                Some(TermReference::Binding(BindingId::new(0)))
            );
            assert_eq!(term.status, TermStatus::Inferred);
        }
        let (_, formula) = output
            .term_formula
            .formulas()
            .iter()
            .next()
            .expect("inequality formula should be checked");
        assert_eq!(formula.kind, FormulaKind::Inequality);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert_eq!(formula.expected_types.len(), 2);
        assert_eq!(formula.expected_types[0].source_range, payload.left_range);
        assert_eq!(formula.expected_types[1].source_range, payload.right_range);
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
        assert!(output.term_formula.facts().is_empty());
        assert!(output.term_formula.diagnostics().is_empty());

        let invalid_key = || {
            vec![
                TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY
                    .to_owned(),
            ]
        };
        let mut missing_left_expected =
            source_four_edge_local_mode_reserved_variable_inequality_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce a missing-left-expected corruption target");
        missing_left_expected.left_expected_input = None;
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&missing_left_expected),
            invalid_key()
        );
        let mut missing_right_expected =
            source_four_edge_local_mode_reserved_variable_inequality_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce a missing-right-expected corruption target");
        missing_right_expected.right_expected_input = None;
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&missing_right_expected),
            invalid_key()
        );

        for removed in [
            "BaseFourEdgeModeInequality",
            "InnerFourEdgeModeInequality",
            "MiddleFourEdgeModeInequality",
            "OuterFourEdgeModeInequality",
            "TooDeepFourEdgeModeInequality",
        ] {
            let mut invalid = source_four_edge_local_mode_reserved_variable_inequality_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce a five-expansion corruption target");
            invalid
                .payload
                .reserve
                .mode_expansions
                .retain(|symbol, _| source_mode_symbol_spelling(symbol) != Some(removed));
            assert_eq!(
                source_reserved_variable_formula_output_detail_keys(&invalid),
                vec![
                    TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY
                        .to_owned()
                ]
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

        for index in 0..5 {
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
            "BaseFourEdgeModeInequality",
            "InnerFourEdgeModeInequality",
            "MiddleFourEdgeModeInequality",
            "OuterFourEdgeModeInequality",
        ]
        .into_iter()
        .enumerate()
        {
            let index = index + 1;
            let mut argument_bearing = exact_modes();
            argument_bearing[index].rhs_shape = ReserveTypeShape::QualifiedSymbolWithArgs(radix);
            assert_extraction_gap(argument_bearing);

            let mut wrong_radix = exact_modes();
            wrong_radix[index].rhs_shape =
                ReserveTypeShape::QualifiedSymbol("ExtraFourEdgeModeInequality");
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

        let mut direct_outermost_radix = exact_modes();
        direct_outermost_radix[4].rhs_shape = ReserveTypeShape::Builtin("set");
        assert_extraction_gap(direct_outermost_radix);

        let mut one_edge_outermost_radix = exact_modes();
        one_edge_outermost_radix[4].rhs_shape =
            ReserveTypeShape::QualifiedSymbol("BaseFourEdgeModeInequality");
        assert_extraction_gap(one_edge_outermost_radix);

        let mut two_edge_outermost_radix = exact_modes();
        two_edge_outermost_radix[4].rhs_shape =
            ReserveTypeShape::QualifiedSymbol("InnerFourEdgeModeInequality");
        assert_extraction_gap(two_edge_outermost_radix);

        let mut three_edge_outermost_radix = exact_modes();
        three_edge_outermost_radix[4].rhs_shape =
            ReserveTypeShape::QualifiedSymbol("MiddleFourEdgeModeInequality");
        assert_extraction_gap(three_edge_outermost_radix);

        let mut cyclic = exact_modes();
        cyclic[0].rhs_shape = ReserveTypeShape::QualifiedSymbol("TooDeepFourEdgeModeInequality");
        assert_extraction_gap(cyclic);

        assert_extraction_gap(vec![
            exact_modes()[0],
            mode_definition_with_label(
                "ExtraFourEdgeModeInequality",
                "ExtraFourEdgeModeInequalityDef",
                ReserveTypeShape::QualifiedSymbol("BaseFourEdgeModeInequality"),
            ),
            mode_definition_with_label(
                "InnerFourEdgeModeInequality",
                "InnerFourEdgeModeInequalityDef",
                ReserveTypeShape::QualifiedSymbol("ExtraFourEdgeModeInequality"),
            ),
            exact_modes()[2],
            exact_modes()[3],
            exact_modes()[4],
        ]);

        for near_miss in [
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(
                    vec!["z"],
                    ReserveTypeShape::QualifiedSymbolWithArgs("TooDeepFourEdgeModeInequality"),
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
    fn active_four_edge_local_mode_reserved_variable_inequality_fixture_consumes_five_expansions() {
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
                    == "pass_type_elaboration_four_edge_local_mode_reserved_variable_inequality_001"
            })
            .expect("Task 168 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 168 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 168 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_four_edge_local_mode_reserved_variable_inequality_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 168 real AST should reach the four-edge local-mode inequality seam");
        assert_source_reserved_variable_formula_output(&output)
            .expect("Task 168 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 5);
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
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("BaseFourEdgeModeInequality")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("Task 168 base terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 168 normalized set type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
    }
    #[test]
    fn source_four_edge_local_object_mode_reserved_variable_equality_consumes_five_expansions() {
        let source_id = source_id(167);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("four_edge_local_object_mode_reserved_variable_equality"),
        );
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseFourEdgeObjectModeEquality", SymbolKind::Mode),
                ("InnerFourEdgeObjectModeEquality", SymbolKind::Mode),
                ("MiddleFourEdgeObjectModeEquality", SymbolKind::Mode),
                ("OuterFourEdgeObjectModeEquality", SymbolKind::Mode),
                ("TooDeepFourEdgeObjectModeEquality", SymbolKind::Mode),
                ("ExtraFourEdgeObjectModeEquality", SymbolKind::Mode),
            ],
        );
        let theorem = IdentifierBinaryTheoremSpec {
            status: None,
            label: "FourEdgeLocalObjectModeReservedVariableEqualityPayloadBoundary",
            left: "z",
            operator: "=",
            right: "z",
            recovered_label: false,
        };
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    "BaseFourEdgeObjectModeEquality",
                    "BaseFourEdgeObjectModeEqualityDef",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition_with_label(
                    "InnerFourEdgeObjectModeEquality",
                    "InnerFourEdgeObjectModeEqualityDef",
                    ReserveTypeShape::QualifiedSymbol("BaseFourEdgeObjectModeEquality"),
                ),
                mode_definition_with_label(
                    "MiddleFourEdgeObjectModeEquality",
                    "MiddleFourEdgeObjectModeEqualityDef",
                    ReserveTypeShape::QualifiedSymbol("InnerFourEdgeObjectModeEquality"),
                ),
                mode_definition_with_label(
                    "OuterFourEdgeObjectModeEquality",
                    "OuterFourEdgeObjectModeEqualityDef",
                    ReserveTypeShape::QualifiedSymbol("MiddleFourEdgeObjectModeEquality"),
                ),
                mode_definition_with_label(
                    "TooDeepFourEdgeObjectModeEquality",
                    "TooDeepFourEdgeObjectModeEqualityDef",
                    ReserveTypeShape::QualifiedSymbol("OuterFourEdgeObjectModeEquality"),
                ),
            ]
        };
        let reserve = || {
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("TooDeepFourEdgeObjectModeEquality"),
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
        let payload = extract_source_four_edge_local_object_mode_reserved_variable_equality(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact four-edge local-object-mode reserved-variable equality should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 5);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(payload.reserve.bridge.bindings()[0].spelling, "z");
        assert_eq!(
            payload.reserve.bridge.bindings()[0].type_spelling,
            "TooDeepFourEdgeObjectModeEquality"
        );
        assert_eq!(payload.left_lookup_ordinal, 1);
        assert_eq!(payload.right_lookup_ordinal, 2);

        let output = source_four_edge_local_object_mode_reserved_variable_equality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact four-edge local-object-mode equality should reach TermFormulaChecker");
        assert_source_reserved_variable_formula_output(&output)
            .expect("four-edge local-object-mode equality invariants should hold");
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
            assert_eq!(input.spelling, "TooDeepFourEdgeObjectModeEquality");
            assert!(matches!(input.head, TypeHeadInput::Symbol(_)));
            assert_eq!(
                input.source_range,
                payload.reserve.bridge.bindings()[0].type_range
            );
        }
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("BaseFourEdgeObjectModeEquality")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("base mode terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("one normalized terminal type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
        assert_ne!(
            payload.reserve.bridge.bindings()[0].type_range,
            terminal.source_range
        );
        assert_eq!(output.term_formula.terms().len(), 2);
        for (_, term) in output.term_formula.terms().iter() {
            assert_eq!(term.kind, TermKind::Variable);
            assert_eq!(
                term.reference,
                Some(TermReference::Binding(BindingId::new(0)))
            );
            assert_eq!(term.status, TermStatus::Inferred);
        }
        let (_, formula) = output
            .term_formula
            .formulas()
            .iter()
            .next()
            .expect("equality formula should be checked");
        assert_eq!(formula.kind, FormulaKind::Equality);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert_eq!(formula.expected_types.len(), 2);
        assert_eq!(formula.expected_types[0].source_range, payload.left_range);
        assert_eq!(formula.expected_types[1].source_range, payload.right_range);
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
        assert!(output.term_formula.facts().is_empty());
        assert!(output.term_formula.diagnostics().is_empty());

        let invalid_key = || {
            vec![
                TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY
                    .to_owned(),
            ]
        };
        let mut missing_left_expected =
            source_four_edge_local_object_mode_reserved_variable_equality_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce a missing-left-expected corruption target");
        missing_left_expected.left_expected_input = None;
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&missing_left_expected),
            invalid_key()
        );
        let mut missing_right_expected =
            source_four_edge_local_object_mode_reserved_variable_equality_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce a missing-right-expected corruption target");
        missing_right_expected.right_expected_input = None;
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&missing_right_expected),
            invalid_key()
        );

        for removed in [
            "BaseFourEdgeObjectModeEquality",
            "InnerFourEdgeObjectModeEquality",
            "MiddleFourEdgeObjectModeEquality",
            "OuterFourEdgeObjectModeEquality",
            "TooDeepFourEdgeObjectModeEquality",
        ] {
            let mut invalid = source_four_edge_local_object_mode_reserved_variable_equality_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce a five-expansion corruption target");
            invalid
                .payload
                .reserve
                .mode_expansions
                .retain(|symbol, _| source_mode_symbol_spelling(symbol) != Some(removed));
            assert_eq!(
                source_reserved_variable_formula_output_detail_keys(&invalid),
                vec![
                    TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY
                        .to_owned()
                ]
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

        for index in 0..5 {
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
            "BaseFourEdgeObjectModeEquality",
            "InnerFourEdgeObjectModeEquality",
            "MiddleFourEdgeObjectModeEquality",
            "OuterFourEdgeObjectModeEquality",
        ]
        .into_iter()
        .enumerate()
        {
            let index = index + 1;
            let mut argument_bearing = exact_modes();
            argument_bearing[index].rhs_shape = ReserveTypeShape::QualifiedSymbolWithArgs(radix);
            assert_extraction_gap(argument_bearing);

            let mut wrong_radix = exact_modes();
            wrong_radix[index].rhs_shape =
                ReserveTypeShape::QualifiedSymbol("ExtraFourEdgeObjectModeEquality");
            assert_extraction_gap(wrong_radix);
        }

        let mut duplicate = exact_modes();
        duplicate.push(duplicate[0]);
        assert_extraction_gap(duplicate);

        let mut attributed_terminal = exact_modes();
        attributed_terminal[0].rhs_shape = ReserveTypeShape::AttributedSet;
        assert_extraction_gap(attributed_terminal);

        let mut set_terminal = exact_modes();
        set_terminal[0].rhs_shape = ReserveTypeShape::Builtin("set");
        assert_extraction_gap(set_terminal);

        let mut forward_order = exact_modes();
        forward_order.reverse();
        assert_extraction_gap(forward_order);

        let mut direct_outermost_radix = exact_modes();
        direct_outermost_radix[4].rhs_shape = ReserveTypeShape::Builtin("object");
        assert_extraction_gap(direct_outermost_radix);

        let mut one_edge_outermost_radix = exact_modes();
        one_edge_outermost_radix[4].rhs_shape =
            ReserveTypeShape::QualifiedSymbol("BaseFourEdgeObjectModeEquality");
        assert_extraction_gap(one_edge_outermost_radix);

        let mut two_edge_outermost_radix = exact_modes();
        two_edge_outermost_radix[4].rhs_shape =
            ReserveTypeShape::QualifiedSymbol("InnerFourEdgeObjectModeEquality");
        assert_extraction_gap(two_edge_outermost_radix);

        let mut three_edge_outermost_radix = exact_modes();
        three_edge_outermost_radix[4].rhs_shape =
            ReserveTypeShape::QualifiedSymbol("MiddleFourEdgeObjectModeEquality");
        assert_extraction_gap(three_edge_outermost_radix);

        let mut cyclic = exact_modes();
        cyclic[0].rhs_shape =
            ReserveTypeShape::QualifiedSymbol("TooDeepFourEdgeObjectModeEquality");
        assert_extraction_gap(cyclic);

        assert_extraction_gap(vec![
            exact_modes()[0],
            mode_definition_with_label(
                "ExtraFourEdgeObjectModeEquality",
                "ExtraFourEdgeObjectModeEqualityDef",
                ReserveTypeShape::QualifiedSymbol("BaseFourEdgeObjectModeEquality"),
            ),
            mode_definition_with_label(
                "InnerFourEdgeObjectModeEquality",
                "InnerFourEdgeObjectModeEqualityDef",
                ReserveTypeShape::QualifiedSymbol("ExtraFourEdgeObjectModeEquality"),
            ),
            exact_modes()[2],
            exact_modes()[3],
            exact_modes()[4],
        ]);

        for near_miss in [
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(
                    vec!["z"],
                    ReserveTypeShape::QualifiedSymbolWithArgs("TooDeepFourEdgeObjectModeEquality"),
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
    fn active_four_edge_local_object_mode_reserved_variable_equality_fixture_consumes_five_expansions()
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
                    == "pass_type_elaboration_four_edge_local_object_mode_reserved_variable_equality_001"
            })
            .expect("Task 167 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 167 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 167 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_four_edge_local_object_mode_reserved_variable_equality_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 167 real AST should reach the four-edge local-object-mode equality seam");
        assert_source_reserved_variable_formula_output(&output)
            .expect("Task 167 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 5);
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
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("BaseFourEdgeObjectModeEquality")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("Task 167 base terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 167 normalized object type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
    }
    #[test]
    fn source_four_edge_local_object_mode_reserved_variable_inequality_consumes_five_expansions() {
        let source_id = source_id(169);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("four_edge_local_object_mode_reserved_variable_inequality"),
        );
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseFourEdgeObjectModeInequality", SymbolKind::Mode),
                ("InnerFourEdgeObjectModeInequality", SymbolKind::Mode),
                ("MiddleFourEdgeObjectModeInequality", SymbolKind::Mode),
                ("OuterFourEdgeObjectModeInequality", SymbolKind::Mode),
                ("TooDeepFourEdgeObjectModeInequality", SymbolKind::Mode),
                ("ExtraFourEdgeObjectModeInequality", SymbolKind::Mode),
            ],
        );
        let theorem = IdentifierBinaryTheoremSpec {
            status: None,
            label: "FourEdgeLocalObjectModeReservedVariableInequalityPayloadBoundary",
            left: "z",
            operator: "<>",
            right: "z",
            recovered_label: false,
        };
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    "BaseFourEdgeObjectModeInequality",
                    "BaseFourEdgeObjectModeInequalityDef",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition_with_label(
                    "InnerFourEdgeObjectModeInequality",
                    "InnerFourEdgeObjectModeInequalityDef",
                    ReserveTypeShape::QualifiedSymbol("BaseFourEdgeObjectModeInequality"),
                ),
                mode_definition_with_label(
                    "MiddleFourEdgeObjectModeInequality",
                    "MiddleFourEdgeObjectModeInequalityDef",
                    ReserveTypeShape::QualifiedSymbol("InnerFourEdgeObjectModeInequality"),
                ),
                mode_definition_with_label(
                    "OuterFourEdgeObjectModeInequality",
                    "OuterFourEdgeObjectModeInequalityDef",
                    ReserveTypeShape::QualifiedSymbol("MiddleFourEdgeObjectModeInequality"),
                ),
                mode_definition_with_label(
                    "TooDeepFourEdgeObjectModeInequality",
                    "TooDeepFourEdgeObjectModeInequalityDef",
                    ReserveTypeShape::QualifiedSymbol("OuterFourEdgeObjectModeInequality"),
                ),
            ]
        };
        let reserve = || {
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("TooDeepFourEdgeObjectModeInequality"),
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
        let payload = extract_source_four_edge_local_object_mode_reserved_variable_inequality(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact four-edge local-object-mode reserved-variable inequality should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 5);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(payload.reserve.bridge.bindings()[0].spelling, "z");
        assert_eq!(
            payload.reserve.bridge.bindings()[0].type_spelling,
            "TooDeepFourEdgeObjectModeInequality"
        );
        assert_eq!(payload.left_lookup_ordinal, 1);
        assert_eq!(payload.right_lookup_ordinal, 2);

        let output = source_four_edge_local_object_mode_reserved_variable_inequality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact four-edge local-object-mode inequality should reach TermFormulaChecker");
        assert_source_reserved_variable_formula_output(&output)
            .expect("four-edge local-object-mode inequality invariants should hold");
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
            assert_eq!(input.spelling, "TooDeepFourEdgeObjectModeInequality");
            assert!(matches!(input.head, TypeHeadInput::Symbol(_)));
            assert_eq!(
                input.source_range,
                payload.reserve.bridge.bindings()[0].type_range
            );
        }
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("BaseFourEdgeObjectModeInequality")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("base mode terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("one normalized terminal type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
        assert_ne!(
            payload.reserve.bridge.bindings()[0].type_range,
            terminal.source_range
        );
        assert_eq!(output.term_formula.terms().len(), 2);
        for (_, term) in output.term_formula.terms().iter() {
            assert_eq!(term.kind, TermKind::Variable);
            assert_eq!(
                term.reference,
                Some(TermReference::Binding(BindingId::new(0)))
            );
            assert_eq!(term.status, TermStatus::Inferred);
        }
        let (_, formula) = output
            .term_formula
            .formulas()
            .iter()
            .next()
            .expect("inequality formula should be checked");
        assert_eq!(formula.kind, FormulaKind::Inequality);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert_eq!(formula.expected_types.len(), 2);
        assert_eq!(formula.expected_types[0].source_range, payload.left_range);
        assert_eq!(formula.expected_types[1].source_range, payload.right_range);
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
        assert!(output.term_formula.facts().is_empty());
        assert!(output.term_formula.diagnostics().is_empty());

        let invalid_key = || {
            vec![
                TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY
                    .to_owned(),
            ]
        };
        let mut missing_left_expected =
            source_four_edge_local_object_mode_reserved_variable_inequality_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce a missing-left-expected corruption target");
        missing_left_expected.left_expected_input = None;
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&missing_left_expected),
            invalid_key()
        );
        let mut missing_right_expected =
            source_four_edge_local_object_mode_reserved_variable_inequality_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce a missing-right-expected corruption target");
        missing_right_expected.right_expected_input = None;
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&missing_right_expected),
            invalid_key()
        );

        for removed in [
            "BaseFourEdgeObjectModeInequality",
            "InnerFourEdgeObjectModeInequality",
            "MiddleFourEdgeObjectModeInequality",
            "OuterFourEdgeObjectModeInequality",
            "TooDeepFourEdgeObjectModeInequality",
        ] {
            let mut invalid =
                source_four_edge_local_object_mode_reserved_variable_inequality_output(
                    &exact,
                    module.clone(),
                    &symbols,
                )
                .expect("exact source should produce a five-expansion corruption target");
            invalid
                .payload
                .reserve
                .mode_expansions
                .retain(|symbol, _| source_mode_symbol_spelling(symbol) != Some(removed));
            assert_eq!(
                source_reserved_variable_formula_output_detail_keys(&invalid),
                vec![
                    TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY
                        .to_owned()
                ]
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

        for index in 0..5 {
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
            "BaseFourEdgeObjectModeInequality",
            "InnerFourEdgeObjectModeInequality",
            "MiddleFourEdgeObjectModeInequality",
            "OuterFourEdgeObjectModeInequality",
        ]
        .into_iter()
        .enumerate()
        {
            let index = index + 1;
            let mut argument_bearing = exact_modes();
            argument_bearing[index].rhs_shape = ReserveTypeShape::QualifiedSymbolWithArgs(radix);
            assert_extraction_gap(argument_bearing);

            let mut wrong_radix = exact_modes();
            wrong_radix[index].rhs_shape =
                ReserveTypeShape::QualifiedSymbol("ExtraFourEdgeObjectModeInequality");
            assert_extraction_gap(wrong_radix);
        }

        let mut duplicate = exact_modes();
        duplicate.push(duplicate[0]);
        assert_extraction_gap(duplicate);

        let mut attributed_terminal = exact_modes();
        attributed_terminal[0].rhs_shape = ReserveTypeShape::AttributedSet;
        assert_extraction_gap(attributed_terminal);

        let mut set_terminal = exact_modes();
        set_terminal[0].rhs_shape = ReserveTypeShape::Builtin("set");
        assert_extraction_gap(set_terminal);

        let mut forward_order = exact_modes();
        forward_order.reverse();
        assert_extraction_gap(forward_order);

        let mut direct_outermost_radix = exact_modes();
        direct_outermost_radix[4].rhs_shape = ReserveTypeShape::Builtin("object");
        assert_extraction_gap(direct_outermost_radix);

        let mut one_edge_outermost_radix = exact_modes();
        one_edge_outermost_radix[4].rhs_shape =
            ReserveTypeShape::QualifiedSymbol("BaseFourEdgeObjectModeInequality");
        assert_extraction_gap(one_edge_outermost_radix);

        let mut two_edge_outermost_radix = exact_modes();
        two_edge_outermost_radix[4].rhs_shape =
            ReserveTypeShape::QualifiedSymbol("InnerFourEdgeObjectModeInequality");
        assert_extraction_gap(two_edge_outermost_radix);

        let mut three_edge_outermost_radix = exact_modes();
        three_edge_outermost_radix[4].rhs_shape =
            ReserveTypeShape::QualifiedSymbol("MiddleFourEdgeObjectModeInequality");
        assert_extraction_gap(three_edge_outermost_radix);

        let mut cyclic = exact_modes();
        cyclic[0].rhs_shape =
            ReserveTypeShape::QualifiedSymbol("TooDeepFourEdgeObjectModeInequality");
        assert_extraction_gap(cyclic);

        assert_extraction_gap(vec![
            exact_modes()[0],
            mode_definition_with_label(
                "ExtraFourEdgeObjectModeInequality",
                "ExtraFourEdgeObjectModeInequalityDef",
                ReserveTypeShape::QualifiedSymbol("BaseFourEdgeObjectModeInequality"),
            ),
            mode_definition_with_label(
                "InnerFourEdgeObjectModeInequality",
                "InnerFourEdgeObjectModeInequalityDef",
                ReserveTypeShape::QualifiedSymbol("ExtraFourEdgeObjectModeInequality"),
            ),
            exact_modes()[2],
            exact_modes()[3],
            exact_modes()[4],
        ]);

        for near_miss in [
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(
                    vec!["z"],
                    ReserveTypeShape::QualifiedSymbolWithArgs(
                        "TooDeepFourEdgeObjectModeInequality",
                    ),
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
    fn active_four_edge_local_object_mode_reserved_variable_inequality_fixture_consumes_five_expansions()
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
                    == "pass_type_elaboration_four_edge_local_object_mode_reserved_variable_inequality_001"
            })
            .expect("Task 169 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 169 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 169 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_four_edge_local_object_mode_reserved_variable_inequality_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 169 real AST should reach the four-edge local-object-mode inequality seam");
        assert_source_reserved_variable_formula_output(&output)
            .expect("Task 169 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 5);
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
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("BaseFourEdgeObjectModeInequality")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("Task 169 base terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 169 normalized object type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
    }
    #[test]
    fn source_three_edge_local_object_mode_two_hop_asserted_head_consumes_four_expansions() {
        let source_id = source_id(214);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("three_edge_local_object_mode_two_hop_asserted_head"),
        );
        let base_mode = "BaseThreeEdgeObjectModeTwoHopAssertedHead";
        let inner_mode = "InnerThreeEdgeObjectModeTwoHopAssertedHead";
        let middle_mode = "MiddleThreeEdgeObjectModeTwoHopAssertedHead";
        let outer_mode = "OuterThreeEdgeObjectModeTwoHopAssertedHead";
        let deeper_mode = "DeeperThreeEdgeObjectModeTwoHopAssertedHead";
        let all_modes = [base_mode, inner_mode, middle_mode, outer_mode];
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                (base_mode, SymbolKind::Mode),
                (inner_mode, SymbolKind::Mode),
                (middle_mode, SymbolKind::Mode),
                (outer_mode, SymbolKind::Mode),
                (deeper_mode, SymbolKind::Mode),
                (
                    "OtherThreeEdgeObjectModeTwoHopAssertedHead",
                    SymbolKind::Mode,
                ),
            ],
        );
        let theorem = exact_three_edge_local_object_mode_two_hop_asserted_head_spec();
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    base_mode,
                    "BaseThreeEdgeObjectModeTwoHopAssertedHeadDef",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition_with_label(
                    inner_mode,
                    "InnerThreeEdgeObjectModeTwoHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(base_mode),
                ),
                mode_definition_with_label(
                    middle_mode,
                    "MiddleThreeEdgeObjectModeTwoHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(inner_mode),
                ),
                mode_definition_with_label(
                    outer_mode,
                    "OuterThreeEdgeObjectModeTwoHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(middle_mode),
                ),
            ]
        };
        let reserve = || {
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol(outer_mode),
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
        let payload = extract_source_three_edge_local_object_mode_two_hop_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("the exact object-terminal three-edge two-hop source should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 4);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(payload.subject_lookup_ordinal, 1);
        let source_binding = &payload.reserve.bridge.bindings()[0];
        assert_eq!(source_binding.type_spelling, outer_mode);
        assert_eq!(payload.asserted_type.spelling, inner_mode);
        let expansion = |spelling| {
            payload
                .reserve
                .mode_expansions
                .iter()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(spelling))
                .map(|(_, expansion)| expansion)
                .unwrap()
        };
        let outer_expansion = expansion(outer_mode);
        let middle_expansion = expansion(middle_mode);
        let inner_expansion = expansion(inner_mode);
        let base_expansion = expansion(base_mode);
        assert_eq!(outer_expansion.radix.spelling, middle_mode);
        assert_eq!(middle_expansion.radix.spelling, inner_mode);
        assert_eq!(middle_expansion.radix.head, payload.asserted_type.head);
        assert_eq!(inner_expansion.radix.spelling, base_mode);
        assert_eq!(base_expansion.radix.spelling, "object");
        assert_eq!(base_expansion.radix.head, TypeHeadInput::BuiltinObject);
        let TypeHeadInput::Symbol(outer_symbol) = &source_binding.type_head else {
            panic!("outer binding must resolve to a symbol")
        };
        let TypeHeadInput::Symbol(middle_symbol) = &outer_expansion.radix.head else {
            panic!("outer expansion must resolve to the middle symbol")
        };
        let TypeHeadInput::Symbol(inner_symbol) = &payload.asserted_type.head else {
            panic!("asserted head must resolve to the inner symbol")
        };
        assert_ne!(outer_symbol, middle_symbol);
        assert_ne!(outer_symbol, inner_symbol);
        assert_ne!(middle_symbol, inner_symbol);

        let exact_output = || {
            source_three_edge_local_object_mode_two_hop_asserted_head_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .unwrap()
        };
        let output = exact_output();
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 214 exact checker output must satisfy every invariant");
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.payload.reserve.mode_expansions.len(), 4);
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert_eq!(output.subject_result_input.spelling, outer_mode);
        assert_eq!(output.asserted_type_input.spelling, inner_mode);
        assert_ne!(
            output.subject_result_input.head,
            output.asserted_type_input.head
        );
        assert_ne!(
            output.subject_result_input.site,
            output.asserted_type_input.site
        );
        assert_ne!(
            output.subject_result_input.source_range,
            output.asserted_type_input.source_range
        );
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .unwrap();
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_ne!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_eq!(normalized.source.range, base_expansion.radix.source_range);
        assert_eq!(normalized.source.spelling, "object");
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
                    TYPE_ELABORATION_THREE_EDGE_LOCAL_OBJECT_MODE_TWO_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
                ),
                vec![
                    TYPE_ELABORATION_THREE_EDGE_LOCAL_OBJECT_MODE_TWO_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY
                        .to_owned()
                ]
            );
        };
        for removed in all_modes {
            let mut invalid = exact_output();
            invalid
                .payload
                .reserve
                .mode_expansions
                .retain(|symbol, _| source_mode_symbol_spelling(symbol) != Some(removed));
            assert_invalid_output(invalid);
        }
        let mut corruptions = Vec::new();
        let mut invalid = exact_output();
        invalid.subject_binding = BindingId::new(1);
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.payload.subject_lookup_ordinal = 2;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.spelling = middle_mode.to_owned();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.head = invalid.asserted_type_input.head.clone();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.site = invalid.asserted_type_input.site.clone();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.source_range = invalid.asserted_type_input.source_range;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.spelling = middle_mode.to_owned();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.head = invalid.subject_result_input.head.clone();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.site = invalid.subject_result_input.site.clone();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.source_range = invalid.subject_result_input.source_range;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        let (_, outer) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(outer_mode))
            .unwrap();
        outer.radix.spelling = inner_mode.to_owned();
        outer.radix.head = invalid.asserted_type_input.head.clone();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        let base_symbol = invalid
            .payload
            .reserve
            .mode_expansions
            .keys()
            .find(|symbol| source_mode_symbol_spelling(symbol) == Some(base_mode))
            .unwrap()
            .clone();
        let (_, middle) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(middle_mode))
            .unwrap();
        middle.radix.spelling = base_mode.to_owned();
        middle.radix.head = TypeHeadInput::Symbol(base_symbol);
        corruptions.push(invalid);
        let mut invalid = exact_output();
        let (_, inner) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(inner_mode))
            .unwrap();
        inner.radix.spelling = "object".to_owned();
        inner.radix.head = TypeHeadInput::BuiltinObject;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        let (_, base) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(base_mode))
            .unwrap();
        base.radix.spelling = "set".to_owned();
        base.radix.head = TypeHeadInput::BuiltinSet;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        let (_, base) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(base_mode))
            .unwrap();
        base.radix.source_range = range(source_id, 0, 1);
        corruptions.push(invalid);
        for invalid in corruptions {
            assert_invalid_output(invalid);
        }
        assert_source_reserved_variable_type_assertion_output(&exact_output())
            .expect("mutating cloned Task 214 outputs must not mutate the exact output");

        let assert_extraction_gap = |ast, context: &str| {
            assert_eq!(
                source_type_elaboration_detail_keys(&ast, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()],
                "{context}"
            );
        };
        let ordered = exact_modes();
        let mut permutation_count = 0;
        for a in 0..4 {
            for b in 0..4 {
                for c in 0..4 {
                    for d in 0..4 {
                        if a == b || a == c || a == d || b == c || b == d || c == d {
                            continue;
                        }
                        if [a, b, c, d] == [0, 1, 2, 3] {
                            continue;
                        }
                        permutation_count += 1;
                        assert_extraction_gap(
                            mode_then_reserve_identifier_type_assertion_theorem_ast(
                                source_id,
                                vec![ordered[a], ordered[b], ordered[c], ordered[d]],
                                reserve(),
                                theorem,
                            ),
                            &format!("definition permutation {a}{b}{c}{d}"),
                        );
                    }
                }
            }
        }
        assert_eq!(permutation_count, 23);
        for index in 0..4 {
            let expected_radix = match index {
                0 => "object",
                1 => base_mode,
                2 => inner_mode,
                _ => middle_mode,
            };
            let mut near_misses = Vec::new();
            let mut missing = exact_modes();
            missing.remove(index);
            near_misses.push(missing);
            let mut duplicate = exact_modes();
            duplicate.push(duplicate[index]);
            near_misses.push(duplicate);
            let mut wrong_label = exact_modes();
            wrong_label[index].label = Some("WrongThreeEdgeObjectModeTwoHopAssertedHeadDef");
            near_misses.push(wrong_label);
            let mut wrong_spelling = exact_modes();
            wrong_spelling[index].pattern = "OtherThreeEdgeObjectModeTwoHopAssertedHead";
            near_misses.push(wrong_spelling);
            let mut wrong_radix = exact_modes();
            wrong_radix[index].rhs_shape = if index == 0 {
                ReserveTypeShape::Builtin("set")
            } else {
                ReserveTypeShape::QualifiedSymbol("OtherThreeEdgeObjectModeTwoHopAssertedHead")
            };
            near_misses.push(wrong_radix);
            let mut recovered = exact_modes();
            recovered[index].recovered = true;
            near_misses.push(recovered);
            let mut contextual = exact_modes();
            contextual[index].local_context = true;
            near_misses.push(contextual);
            let mut parameterized = exact_modes();
            parameterized[index].parameterized_pattern = true;
            near_misses.push(parameterized);
            let mut args = exact_modes();
            args[index].rhs_shape = ReserveTypeShape::QualifiedSymbolWithArgs(expected_radix);
            near_misses.push(args);
            let mut attributes = exact_modes();
            attributes[index].rhs_shape = if index == 0 {
                ReserveTypeShape::AttributedObject
            } else {
                ReserveTypeShape::AttributedQualifiedSymbol(expected_radix)
            };
            near_misses.push(attributes);
            for (variant, modes) in near_misses.into_iter().enumerate() {
                assert_extraction_gap(
                    mode_then_reserve_identifier_type_assertion_theorem_ast(
                        source_id,
                        modes,
                        reserve(),
                        theorem,
                    ),
                    &format!("definition {index} near miss {variant}"),
                );
            }
        }

        let mut broken_outer_link = exact_modes();
        broken_outer_link[3].rhs_shape = ReserveTypeShape::QualifiedSymbol(inner_mode);
        let mut broken_middle_link = exact_modes();
        broken_middle_link[2].rhs_shape = ReserveTypeShape::QualifiedSymbol(base_mode);
        let mut broken_tail_link = exact_modes();
        broken_tail_link[1].rhs_shape = ReserveTypeShape::Builtin("object");
        for (context, modes) in [
            ("outer-to-middle relation link", broken_outer_link),
            ("middle-to-inner relation link", broken_middle_link),
            ("inner-to-base terminal tail", broken_tail_link),
        ] {
            assert_extraction_gap(
                mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    modes,
                    reserve(),
                    theorem,
                ),
                context,
            );
        }

        let mut source_near_misses = Vec::new();
        for bad_reserve in [
            ReserveTypeShape::QualifiedSymbol(base_mode),
            ReserveTypeShape::QualifiedSymbol(inner_mode),
            ReserveTypeShape::QualifiedSymbol(middle_mode),
            ReserveTypeShape::QualifiedSymbolWithArgs(outer_mode),
            ReserveTypeShape::AttributedQualifiedSymbol(outer_mode),
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
                reserve_item(vec!["x"], ReserveTypeShape::QualifiedSymbol(outer_mode)),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("object")),
            ],
            theorem,
        ));
        for near_miss_theorem in [
            IdentifierTypeAssertionTheoremSpec {
                label: "OtherPayloadBoundary",
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                subject: "y",
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(outer_mode),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(middle_mode),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(base_mode),
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
                asserted_type: ReserveTypeShape::QualifiedSymbol(
                    "OtherThreeEdgeObjectModeTwoHopAssertedHead",
                ),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbolWithArgs(inner_mode),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::AttributedQualifiedSymbol(inner_mode),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                negated: true,
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                status: Some("registration"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                recovered_label: true,
                ..theorem
            },
        ] {
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                near_miss_theorem,
            ));
        }
        let mut connected_deeper_modes = exact_modes();
        connected_deeper_modes.push(mode_definition_with_label(
            deeper_mode,
            "DeeperThreeEdgeObjectModeTwoHopAssertedHeadDef",
            ReserveTypeShape::QualifiedSymbol(outer_mode),
        ));
        source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            connected_deeper_modes.clone(),
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol(deeper_mode),
            )],
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(outer_mode),
                ..theorem
            },
        ));
        source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            connected_deeper_modes,
            reserve(),
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(deeper_mode),
                ..theorem
            },
        ));
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

        let unrelated_import = source_local_and_imported_symbols_env(
            module.clone(),
            &all_modes.map(|spelling| (spelling, SymbolKind::Mode)),
            &[(
                "UnrelatedThreeEdgeObjectModeTwoHopAssertedHead",
                SymbolKind::Mode,
            )],
        );
        assert!(
            extract_source_three_edge_local_object_mode_two_hop_asserted_head(
                &exact,
                module.clone(),
                &unrelated_import,
            )
            .is_some()
        );
        for imported_index in 0..4 {
            let locals = all_modes
                .iter()
                .enumerate()
                .filter(|(index, _)| *index != imported_index)
                .map(|(_, spelling)| (*spelling, SymbolKind::Mode))
                .collect::<Vec<_>>();
            for imports in [
                vec![(all_modes[imported_index], SymbolKind::Mode)],
                vec![
                    (all_modes[imported_index], SymbolKind::Mode),
                    (all_modes[imported_index], SymbolKind::Mode),
                ],
            ] {
                let provenance_near_miss =
                    source_local_and_imported_symbols_env(module.clone(), &locals, &imports);
                assert_eq!(
                    source_type_elaboration_detail_keys(
                        &exact,
                        module.clone(),
                        &provenance_near_miss,
                    ),
                    vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
                );
            }
        }
    }

    #[test]
    fn task214_synthetic_is_rejected_by_all_39_prior_type_assertion_extractors() {
        let source_id = source_id(214);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("task214_prior_extractor_isolation"),
        );
        let names = [
            "BaseThreeEdgeObjectModeTwoHopAssertedHead",
            "InnerThreeEdgeObjectModeTwoHopAssertedHead",
            "MiddleThreeEdgeObjectModeTwoHopAssertedHead",
            "OuterThreeEdgeObjectModeTwoHopAssertedHead",
        ];
        let symbols = source_local_symbols_env(
            module.clone(),
            &names.map(|spelling| (spelling, SymbolKind::Mode)),
        );
        let exact = mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            vec![
                mode_definition_with_label(
                    names[0],
                    "BaseThreeEdgeObjectModeTwoHopAssertedHeadDef",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition_with_label(
                    names[1],
                    "InnerThreeEdgeObjectModeTwoHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(names[0]),
                ),
                mode_definition_with_label(
                    names[2],
                    "MiddleThreeEdgeObjectModeTwoHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(names[1]),
                ),
                mode_definition_with_label(
                    names[3],
                    "OuterThreeEdgeObjectModeTwoHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(names[2]),
                ),
            ],
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol(names[3]),
            )],
            exact_three_edge_local_object_mode_two_hop_asserted_head_spec(),
        );
        assert!(
            extract_source_three_edge_local_object_mode_two_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_some()
        );
        let prior_results = [
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
            super::extract_source_local_object_mode_long_chain_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_mode_two_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_object_mode_two_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_mode_two_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
        ];
        assert_eq!(prior_results.len(), 39);
        assert!(prior_results.into_iter().all(|result| result.is_none()));
    }

    #[test]
    fn active_four_edge_local_mode_two_hop_asserted_head_fixture_consumes_five_expansions() {
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
        let plan = build_test_plan(&config).expect("Task 215 repository plan should build");
        let (ordinal, case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| {
                case.id.0 == "pass_type_elaboration_four_edge_local_mode_two_hop_asserted_head_001"
            })
            .expect("Task 215 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 215 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 215 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_four_edge_local_mode_two_hop_asserted_head_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 215 real AST should reach the set-terminal two-link asserted-head seam");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 215 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 5);
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert_eq!(
            output.subject_result_input.spelling,
            "TooDeepFourEdgeModeTwoHopAssertedHead"
        );
        assert_eq!(
            output.asserted_type_input.spelling,
            "MiddleFourEdgeModeTwoHopAssertedHead"
        );
        assert_ne!(
            output.subject_result_input.head,
            output.asserted_type_input.head
        );
        assert_ne!(
            output.subject_result_input.site,
            output.asserted_type_input.site
        );
        assert_ne!(
            output.subject_result_input.source_range,
            output.asserted_type_input.source_range
        );
        let expansion = |spelling| {
            output
                .payload
                .reserve
                .mode_expansions
                .iter()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(spelling))
                .map(|(_, expansion)| expansion)
                .unwrap()
        };
        let too_deep = expansion("TooDeepFourEdgeModeTwoHopAssertedHead");
        let outer = expansion("OuterFourEdgeModeTwoHopAssertedHead");
        let middle = expansion("MiddleFourEdgeModeTwoHopAssertedHead");
        let inner = expansion("InnerFourEdgeModeTwoHopAssertedHead");
        let base = expansion("BaseFourEdgeModeTwoHopAssertedHead");
        assert_eq!(
            too_deep.radix.spelling,
            "OuterFourEdgeModeTwoHopAssertedHead"
        );
        assert_eq!(outer.radix.spelling, "MiddleFourEdgeModeTwoHopAssertedHead");
        assert_eq!(outer.radix.head, output.asserted_type_input.head);
        assert_eq!(middle.radix.spelling, "InnerFourEdgeModeTwoHopAssertedHead");
        assert_eq!(inner.radix.spelling, "BaseFourEdgeModeTwoHopAssertedHead");
        assert_eq!(base.radix.spelling, "set");
        assert_eq!(base.radix.head, TypeHeadInput::BuiltinSet);
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .unwrap();
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_ne!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_eq!(normalized.source.range, base.radix.source_range);
        assert_eq!(normalized.source.spelling, base.radix.spelling);
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
    fn four_edge_two_hop_asserted_head_route_rejects_all_40_prior_owner_fixtures() {
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
        let plan = build_test_plan(&config).expect("Task 215 repository plan should build");
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
        ];
        assert_eq!(prior_owner_ids.len(), 40);
        for owner_id in prior_owner_ids {
            let (ordinal, case) = active_type_elaboration_cases(&plan)
                .enumerate()
                .find(|(_, case)| case.id.0 == owner_id)
                .unwrap_or_else(|| panic!("prior owner fixture {owner_id} must be active"));
            let frontend = run_frontend(&workspace_root, case, ordinal)
                .unwrap_or_else(|error| panic!("owner fixture {owner_id} must parse: {error}"));
            assert!(frontend.diagnostics.is_empty());
            let ast = frontend
                .ast
                .unwrap_or_else(|| panic!("owner fixture {owner_id} must produce an AST"));
            let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
            assert!(resolver.detail_keys.is_empty());
            let symbols =
                augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
            assert_eq!(
                source_type_elaboration_detail_keys(&ast, resolver.module.clone(), &symbols),
                Vec::<String>::new(),
                "prior owner fixture {owner_id} must still reach its active route"
            );
            assert!(
                extract_source_four_edge_local_mode_two_hop_asserted_head(
                    &ast,
                    resolver.module,
                    &symbols,
                )
                .is_none(),
                "Task 215 must reject prior owner fixture {owner_id}"
            );
        }
    }

    #[test]
    fn source_four_edge_local_mode_two_hop_asserted_head_consumes_five_expansions() {
        let source_id = source_id(215);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("four_edge_local_mode_two_hop_asserted_head"),
        );
        let base_mode = "BaseFourEdgeModeTwoHopAssertedHead";
        let inner_mode = "InnerFourEdgeModeTwoHopAssertedHead";
        let middle_mode = "MiddleFourEdgeModeTwoHopAssertedHead";
        let outer_mode = "OuterFourEdgeModeTwoHopAssertedHead";
        let too_deep_mode = "TooDeepFourEdgeModeTwoHopAssertedHead";
        let deeper_mode = "DeeperFourEdgeModeTwoHopAssertedHead";
        let other_mode = "OtherFourEdgeModeTwoHopAssertedHead";
        let all_modes = [
            base_mode,
            inner_mode,
            middle_mode,
            outer_mode,
            too_deep_mode,
        ];
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                (base_mode, SymbolKind::Mode),
                (inner_mode, SymbolKind::Mode),
                (middle_mode, SymbolKind::Mode),
                (outer_mode, SymbolKind::Mode),
                (too_deep_mode, SymbolKind::Mode),
                (deeper_mode, SymbolKind::Mode),
                (other_mode, SymbolKind::Mode),
            ],
        );
        let theorem = exact_four_edge_local_mode_two_hop_asserted_head_spec();
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    base_mode,
                    "BaseFourEdgeModeTwoHopAssertedHeadDef",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition_with_label(
                    inner_mode,
                    "InnerFourEdgeModeTwoHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(base_mode),
                ),
                mode_definition_with_label(
                    middle_mode,
                    "MiddleFourEdgeModeTwoHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(inner_mode),
                ),
                mode_definition_with_label(
                    outer_mode,
                    "OuterFourEdgeModeTwoHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(middle_mode),
                ),
                mode_definition_with_label(
                    too_deep_mode,
                    "TooDeepFourEdgeModeTwoHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(outer_mode),
                ),
            ]
        };
        let reserve = || {
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol(too_deep_mode),
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
        let payload = extract_source_four_edge_local_mode_two_hop_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("the exact set-terminal four-edge two-hop source should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 5);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(payload.subject_lookup_ordinal, 1);
        let source_binding = &payload.reserve.bridge.bindings()[0];
        assert_eq!(source_binding.type_spelling, too_deep_mode);
        assert_eq!(payload.asserted_type.spelling, middle_mode);
        let expansion = |spelling| {
            payload
                .reserve
                .mode_expansions
                .iter()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(spelling))
                .map(|(_, expansion)| expansion)
                .unwrap()
        };
        let too_deep_expansion = expansion(too_deep_mode);
        let outer_expansion = expansion(outer_mode);
        let middle_expansion = expansion(middle_mode);
        let inner_expansion = expansion(inner_mode);
        let base_expansion = expansion(base_mode);
        assert_eq!(too_deep_expansion.radix.spelling, outer_mode);
        assert_eq!(outer_expansion.radix.spelling, middle_mode);
        assert_eq!(outer_expansion.radix.head, payload.asserted_type.head);
        assert_eq!(middle_expansion.radix.spelling, inner_mode);
        assert_eq!(inner_expansion.radix.spelling, base_mode);
        assert_eq!(base_expansion.radix.spelling, "set");
        assert_eq!(base_expansion.radix.head, TypeHeadInput::BuiltinSet);
        let TypeHeadInput::Symbol(too_deep_symbol) = &source_binding.type_head else {
            panic!("TooDeep binding must resolve to a symbol")
        };
        let TypeHeadInput::Symbol(outer_symbol) = &too_deep_expansion.radix.head else {
            panic!("TooDeep expansion must resolve to the Outer symbol")
        };
        let TypeHeadInput::Symbol(middle_symbol) = &payload.asserted_type.head else {
            panic!("asserted head must resolve to the Middle symbol")
        };
        assert_ne!(too_deep_symbol, outer_symbol);
        assert_ne!(too_deep_symbol, middle_symbol);
        assert_ne!(outer_symbol, middle_symbol);

        let exact_output = || {
            source_four_edge_local_mode_two_hop_asserted_head_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .unwrap()
        };
        let output = exact_output();
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 215 exact checker output must satisfy every invariant");
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.payload.reserve.mode_expansions.len(), 5);
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert_eq!(output.subject_result_input.spelling, too_deep_mode);
        assert_eq!(output.asserted_type_input.spelling, middle_mode);
        assert_ne!(
            output.subject_result_input.head,
            output.asserted_type_input.head
        );
        assert_ne!(
            output.subject_result_input.site,
            output.asserted_type_input.site
        );
        assert_ne!(
            output.subject_result_input.source_range,
            output.asserted_type_input.source_range
        );
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .unwrap();
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_ne!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_eq!(normalized.source.range, base_expansion.radix.source_range);
        assert_eq!(normalized.source.spelling, "set");
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
                    TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_TWO_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
                ),
                vec![
                    TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_TWO_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY
                        .to_owned()
                ]
            );
        };
        for removed in all_modes {
            let mut invalid = exact_output();
            invalid
                .payload
                .reserve
                .mode_expansions
                .retain(|symbol, _| source_mode_symbol_spelling(symbol) != Some(removed));
            assert_invalid_output(invalid);
        }
        let mut corruptions = Vec::new();
        let mut invalid = exact_output();
        invalid.subject_binding = BindingId::new(1);
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.payload.subject_lookup_ordinal = 2;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.spelling = outer_mode.to_owned();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.head = invalid.asserted_type_input.head.clone();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.site = invalid.asserted_type_input.site.clone();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.source_range = invalid.asserted_type_input.source_range;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.spelling = outer_mode.to_owned();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.head = invalid.subject_result_input.head.clone();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.site = invalid.subject_result_input.site.clone();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.source_range = invalid.subject_result_input.source_range;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        let (_, too_deep) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(too_deep_mode))
            .unwrap();
        too_deep.radix.spelling = middle_mode.to_owned();
        too_deep.radix.head = invalid.asserted_type_input.head.clone();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        let inner_symbol = invalid
            .payload
            .reserve
            .mode_expansions
            .keys()
            .find(|symbol| source_mode_symbol_spelling(symbol) == Some(inner_mode))
            .unwrap()
            .clone();
        let (_, outer) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(outer_mode))
            .unwrap();
        outer.radix.spelling = inner_mode.to_owned();
        outer.radix.head = TypeHeadInput::Symbol(inner_symbol);
        corruptions.push(invalid);
        let mut invalid = exact_output();
        let base_symbol = invalid
            .payload
            .reserve
            .mode_expansions
            .keys()
            .find(|symbol| source_mode_symbol_spelling(symbol) == Some(base_mode))
            .unwrap()
            .clone();
        let (_, middle) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(middle_mode))
            .unwrap();
        middle.radix.spelling = base_mode.to_owned();
        middle.radix.head = TypeHeadInput::Symbol(base_symbol);
        corruptions.push(invalid);
        let mut invalid = exact_output();
        let (_, inner) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(inner_mode))
            .unwrap();
        inner.radix.spelling = "set".to_owned();
        inner.radix.head = TypeHeadInput::BuiltinSet;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        let (_, base) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(base_mode))
            .unwrap();
        base.radix.spelling = "object".to_owned();
        base.radix.head = TypeHeadInput::BuiltinObject;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        let (_, base) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(base_mode))
            .unwrap();
        base.radix.source_range = range(source_id, 0, 1);
        corruptions.push(invalid);
        for invalid in corruptions {
            assert_invalid_output(invalid);
        }
        assert_source_reserved_variable_type_assertion_output(&exact_output())
            .expect("mutating cloned Task 215 outputs must not mutate the exact output");

        let assert_extraction_gap = |ast, context: &str| {
            assert_eq!(
                source_type_elaboration_detail_keys(&ast, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()],
                "{context}"
            );
        };
        let ordered = exact_modes();
        let mut permutation_count = 0;
        for a in 0..5 {
            for b in 0..5 {
                for c in 0..5 {
                    for d in 0..5 {
                        for e in 0..5 {
                            let order = [a, b, c, d, e];
                            if order.iter().enumerate().any(|(index, value)| {
                                order[..index].iter().any(|prior| prior == value)
                            }) {
                                continue;
                            }
                            if order == [0, 1, 2, 3, 4] {
                                continue;
                            }
                            permutation_count += 1;
                            assert_extraction_gap(
                                mode_then_reserve_identifier_type_assertion_theorem_ast(
                                    source_id,
                                    vec![
                                        ordered[a], ordered[b], ordered[c], ordered[d], ordered[e],
                                    ],
                                    reserve(),
                                    theorem,
                                ),
                                &format!("definition permutation {a}{b}{c}{d}{e}"),
                            );
                        }
                    }
                }
            }
        }
        assert_eq!(permutation_count, 119);
        for index in 0..5 {
            let expected_radix = match index {
                0 => "set",
                1 => base_mode,
                2 => inner_mode,
                3 => middle_mode,
                _ => outer_mode,
            };
            let mut near_misses = Vec::new();
            let mut missing = exact_modes();
            missing.remove(index);
            near_misses.push(missing);
            let mut duplicate = exact_modes();
            duplicate.push(duplicate[index]);
            near_misses.push(duplicate);
            let mut wrong_label = exact_modes();
            wrong_label[index].label = Some("WrongFourEdgeModeTwoHopAssertedHeadDef");
            near_misses.push(wrong_label);
            let mut wrong_spelling = exact_modes();
            wrong_spelling[index].pattern = other_mode;
            near_misses.push(wrong_spelling);
            let mut wrong_radix = exact_modes();
            wrong_radix[index].rhs_shape = if index == 0 {
                ReserveTypeShape::Builtin("object")
            } else {
                ReserveTypeShape::QualifiedSymbol(other_mode)
            };
            near_misses.push(wrong_radix);
            let mut recovered = exact_modes();
            recovered[index].recovered = true;
            near_misses.push(recovered);
            let mut contextual = exact_modes();
            contextual[index].local_context = true;
            near_misses.push(contextual);
            let mut parameterized = exact_modes();
            parameterized[index].parameterized_pattern = true;
            near_misses.push(parameterized);
            let mut args = exact_modes();
            args[index].rhs_shape = ReserveTypeShape::QualifiedSymbolWithArgs(expected_radix);
            near_misses.push(args);
            let mut attributes = exact_modes();
            attributes[index].rhs_shape = if index == 0 {
                ReserveTypeShape::AttributedSet
            } else {
                ReserveTypeShape::AttributedQualifiedSymbol(expected_radix)
            };
            near_misses.push(attributes);
            for (variant, modes) in near_misses.into_iter().enumerate() {
                assert_extraction_gap(
                    mode_then_reserve_identifier_type_assertion_theorem_ast(
                        source_id,
                        modes,
                        reserve(),
                        theorem,
                    ),
                    &format!("definition {index} near miss {variant}"),
                );
            }
        }

        let mut broken_too_deep_link = exact_modes();
        broken_too_deep_link[4].rhs_shape = ReserveTypeShape::QualifiedSymbol(middle_mode);
        let mut broken_outer_link = exact_modes();
        broken_outer_link[3].rhs_shape = ReserveTypeShape::QualifiedSymbol(inner_mode);
        let mut broken_middle_tail = exact_modes();
        broken_middle_tail[2].rhs_shape = ReserveTypeShape::QualifiedSymbol(base_mode);
        let mut broken_inner_tail = exact_modes();
        broken_inner_tail[1].rhs_shape = ReserveTypeShape::Builtin("set");
        let mut broken_terminal = exact_modes();
        broken_terminal[0].rhs_shape = ReserveTypeShape::Builtin("object");
        for (context, modes) in [
            ("TooDeep-to-Outer relation link", broken_too_deep_link),
            ("Outer-to-Middle relation link", broken_outer_link),
            ("Middle-to-Inner terminal tail", broken_middle_tail),
            ("Inner-to-Base terminal tail", broken_inner_tail),
            ("Base-to-set terminal", broken_terminal),
        ] {
            assert_extraction_gap(
                mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    modes,
                    reserve(),
                    theorem,
                ),
                context,
            );
        }

        let mut source_near_misses = Vec::new();
        for bad_reserve in [
            ReserveTypeShape::QualifiedSymbol(base_mode),
            ReserveTypeShape::QualifiedSymbol(inner_mode),
            ReserveTypeShape::QualifiedSymbol(middle_mode),
            ReserveTypeShape::QualifiedSymbol(outer_mode),
            ReserveTypeShape::QualifiedSymbolWithArgs(too_deep_mode),
            ReserveTypeShape::AttributedQualifiedSymbol(too_deep_mode),
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
                reserve_item(vec!["x"], ReserveTypeShape::QualifiedSymbol(too_deep_mode)),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
            ],
            theorem,
        ));
        for near_miss_theorem in [
            IdentifierTypeAssertionTheoremSpec {
                label: "OtherPayloadBoundary",
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                subject: "y",
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(too_deep_mode),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(outer_mode),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(inner_mode),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(base_mode),
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
                asserted_type: ReserveTypeShape::QualifiedSymbol(other_mode),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbolWithArgs(middle_mode),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::AttributedQualifiedSymbol(middle_mode),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                negated: true,
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                status: Some("registration"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                recovered_label: true,
                ..theorem
            },
        ] {
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                near_miss_theorem,
            ));
        }
        let mut connected_deeper_modes = exact_modes();
        connected_deeper_modes.push(mode_definition_with_label(
            deeper_mode,
            "DeeperFourEdgeModeTwoHopAssertedHeadDef",
            ReserveTypeShape::QualifiedSymbol(too_deep_mode),
        ));
        source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            connected_deeper_modes.clone(),
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol(deeper_mode),
            )],
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(outer_mode),
                ..theorem
            },
        ));
        source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            connected_deeper_modes,
            reserve(),
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(deeper_mode),
                ..theorem
            },
        ));
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

        let unrelated_import = source_local_and_imported_symbols_env(
            module.clone(),
            &all_modes.map(|spelling| (spelling, SymbolKind::Mode)),
            &[("UnrelatedFourEdgeModeTwoHopAssertedHead", SymbolKind::Mode)],
        );
        assert!(
            extract_source_four_edge_local_mode_two_hop_asserted_head(
                &exact,
                module.clone(),
                &unrelated_import,
            )
            .is_some()
        );
        for imported_index in 0..5 {
            let locals = all_modes
                .iter()
                .enumerate()
                .filter(|(index, _)| *index != imported_index)
                .map(|(_, spelling)| (*spelling, SymbolKind::Mode))
                .collect::<Vec<_>>();
            for imports in [
                vec![(all_modes[imported_index], SymbolKind::Mode)],
                vec![
                    (all_modes[imported_index], SymbolKind::Mode),
                    (all_modes[imported_index], SymbolKind::Mode),
                ],
            ] {
                let provenance_near_miss =
                    source_local_and_imported_symbols_env(module.clone(), &locals, &imports);
                assert_eq!(
                    source_type_elaboration_detail_keys(
                        &exact,
                        module.clone(),
                        &provenance_near_miss,
                    ),
                    vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
                );
            }
        }
        for imports in [
            all_modes
                .iter()
                .map(|spelling| (*spelling, SymbolKind::Mode))
                .collect::<Vec<_>>(),
            all_modes
                .iter()
                .flat_map(|spelling| [(*spelling, SymbolKind::Mode), (*spelling, SymbolKind::Mode)])
                .collect::<Vec<_>>(),
        ] {
            let provenance_near_miss =
                source_local_and_imported_symbols_env(module.clone(), &[], &imports);
            assert_eq!(
                source_type_elaboration_detail_keys(&exact, module.clone(), &provenance_near_miss,),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
    }

    #[test]
    fn task215_synthetic_is_rejected_by_all_40_prior_type_assertion_extractors() {
        let source_id = source_id(215);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("task215_prior_extractor_isolation"),
        );
        let names = [
            "BaseFourEdgeModeTwoHopAssertedHead",
            "InnerFourEdgeModeTwoHopAssertedHead",
            "MiddleFourEdgeModeTwoHopAssertedHead",
            "OuterFourEdgeModeTwoHopAssertedHead",
            "TooDeepFourEdgeModeTwoHopAssertedHead",
        ];
        let symbols = source_local_symbols_env(
            module.clone(),
            &names.map(|spelling| (spelling, SymbolKind::Mode)),
        );
        let exact = mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            vec![
                mode_definition_with_label(
                    names[0],
                    "BaseFourEdgeModeTwoHopAssertedHeadDef",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition_with_label(
                    names[1],
                    "InnerFourEdgeModeTwoHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(names[0]),
                ),
                mode_definition_with_label(
                    names[2],
                    "MiddleFourEdgeModeTwoHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(names[1]),
                ),
                mode_definition_with_label(
                    names[3],
                    "OuterFourEdgeModeTwoHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(names[2]),
                ),
                mode_definition_with_label(
                    names[4],
                    "TooDeepFourEdgeModeTwoHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(names[3]),
                ),
            ],
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol(names[4]),
            )],
            exact_four_edge_local_mode_two_hop_asserted_head_spec(),
        );
        assert!(
            extract_source_four_edge_local_mode_two_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_some()
        );
        let prior_results = [
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
            super::extract_source_local_object_mode_long_chain_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_mode_two_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_object_mode_two_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_mode_two_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_object_mode_two_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
        ];
        assert_eq!(prior_results.len(), 40);
        assert!(prior_results.into_iter().all(|result| result.is_none()));
    }

    mod task_216_four_edge_object_two_hop_asserted_head {
        use super::*;

        fn exact_four_edge_local_object_mode_two_hop_asserted_head_spec()
        -> IdentifierTypeAssertionTheoremSpec<'static> {
            IdentifierTypeAssertionTheoremSpec {
                status: None,
                label: "FourEdgeLocalObjectModeTwoHopAssertedHeadPayloadBoundary",
                subject: "x",
                asserted_type: ReserveTypeShape::QualifiedSymbol(
                    "MiddleFourEdgeObjectModeTwoHopAssertedHead",
                ),
                recovered_label: false,
                negated: false,
            }
        }

        #[test]
        fn active_four_edge_local_object_mode_two_hop_asserted_head_fixture_consumes_five_expansions()
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
            let plan = build_test_plan(&config).expect("Task 216 repository plan should build");
            let (ordinal, case) = active_type_elaboration_cases(&plan)
                .enumerate()
                .find(|(_, case)| {
                    case.id.0 == "pass_type_elaboration_four_edge_local_object_mode_two_hop_asserted_head_001"
                })
                .expect("Task 216 active fixture should be discoverable");
            let frontend = run_frontend(&workspace_root, case, ordinal)
                .expect("Task 216 fixture should run through the real frontend");
            assert!(frontend.diagnostics.is_empty());
            let ast = frontend
                .ast
                .expect("Task 216 fixture should produce an AST");
            let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
            assert!(resolver.detail_keys.is_empty());
            let symbols =
                augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
            let output = source_four_edge_local_object_mode_two_hop_asserted_head_output(
                &ast,
                resolver.module,
                &symbols,
            )
            .expect(
                "Task 216 real AST should reach the object-terminal two-link asserted-head seam",
            );
            assert_source_reserved_variable_type_assertion_output(&output)
                .expect("Task 216 real AST should preserve every checked payload invariant");
            assert_eq!(output.payload.reserve.mode_expansions.len(), 5);
            assert_eq!(output.subject_binding, BindingId::new(0));
            assert_eq!(output.payload.subject_lookup_ordinal, 1);
            assert_eq!(output.term_formula.type_entries().len(), 3);
            assert_eq!(output.term_formula.normalized_types().len(), 1);
            assert_eq!(
                output.subject_result_input.spelling,
                "TooDeepFourEdgeObjectModeTwoHopAssertedHead"
            );
            assert_eq!(
                output.asserted_type_input.spelling,
                "MiddleFourEdgeObjectModeTwoHopAssertedHead"
            );
            assert_ne!(
                output.subject_result_input.head,
                output.asserted_type_input.head
            );
            assert_ne!(
                output.subject_result_input.site,
                output.asserted_type_input.site
            );
            assert_ne!(
                output.subject_result_input.source_range,
                output.asserted_type_input.source_range
            );
            let expansion = |spelling| {
                output
                    .payload
                    .reserve
                    .mode_expansions
                    .iter()
                    .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(spelling))
                    .map(|(_, expansion)| expansion)
                    .unwrap()
            };
            let too_deep = expansion("TooDeepFourEdgeObjectModeTwoHopAssertedHead");
            let outer = expansion("OuterFourEdgeObjectModeTwoHopAssertedHead");
            let middle = expansion("MiddleFourEdgeObjectModeTwoHopAssertedHead");
            let inner = expansion("InnerFourEdgeObjectModeTwoHopAssertedHead");
            let base = expansion("BaseFourEdgeObjectModeTwoHopAssertedHead");
            assert_eq!(
                too_deep.radix.spelling,
                "OuterFourEdgeObjectModeTwoHopAssertedHead"
            );
            assert_eq!(
                outer.radix.spelling,
                "MiddleFourEdgeObjectModeTwoHopAssertedHead"
            );
            assert_eq!(outer.radix.head, output.asserted_type_input.head);
            assert_eq!(
                middle.radix.spelling,
                "InnerFourEdgeObjectModeTwoHopAssertedHead"
            );
            assert_eq!(
                inner.radix.spelling,
                "BaseFourEdgeObjectModeTwoHopAssertedHead"
            );
            assert_eq!(base.radix.spelling, "object");
            assert_eq!(base.radix.head, TypeHeadInput::BuiltinObject);
            let (_, normalized) = output
                .term_formula
                .normalized_types()
                .iter()
                .next()
                .unwrap();
            assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
            assert_ne!(normalized.head, TypeHeadRef::BuiltinSet);
            assert_eq!(normalized.source.range, base.radix.source_range);
            assert_eq!(normalized.source.spelling, base.radix.spelling);
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
        fn four_edge_object_two_hop_asserted_head_route_rejects_all_41_prior_owner_fixtures() {
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
            let plan = build_test_plan(&config).expect("Task 216 repository plan should build");
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
            ];
            assert_eq!(prior_owner_ids.len(), 41);
            for owner_id in prior_owner_ids {
                let (ordinal, case) = active_type_elaboration_cases(&plan)
                    .enumerate()
                    .find(|(_, case)| case.id.0 == owner_id)
                    .unwrap_or_else(|| panic!("prior owner fixture {owner_id} must be active"));
                let frontend = run_frontend(&workspace_root, case, ordinal)
                    .unwrap_or_else(|error| panic!("owner fixture {owner_id} must parse: {error}"));
                assert!(frontend.diagnostics.is_empty());
                let ast = frontend
                    .ast
                    .unwrap_or_else(|| panic!("owner fixture {owner_id} must produce an AST"));
                let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
                assert!(resolver.detail_keys.is_empty());
                let symbols =
                    augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
                assert_eq!(
                    source_type_elaboration_detail_keys(&ast, resolver.module.clone(), &symbols),
                    Vec::<String>::new(),
                    "prior owner fixture {owner_id} must still reach its active route"
                );
                assert!(
                    extract_source_four_edge_local_object_mode_two_hop_asserted_head(
                        &ast,
                        resolver.module,
                        &symbols,
                    )
                    .is_none(),
                    "Task 216 must reject prior owner fixture {owner_id}"
                );
            }
        }

        #[test]
        fn source_four_edge_local_object_mode_two_hop_asserted_head_consumes_five_expansions() {
            let source_id = source_id(216);
            let module = ResolverModuleId::new(
                PackageId::new("test"),
                ModulePath::new("four_edge_local_object_mode_two_hop_asserted_head"),
            );
            let base_mode = "BaseFourEdgeObjectModeTwoHopAssertedHead";
            let inner_mode = "InnerFourEdgeObjectModeTwoHopAssertedHead";
            let middle_mode = "MiddleFourEdgeObjectModeTwoHopAssertedHead";
            let outer_mode = "OuterFourEdgeObjectModeTwoHopAssertedHead";
            let too_deep_mode = "TooDeepFourEdgeObjectModeTwoHopAssertedHead";
            let deeper_mode = "DeeperFourEdgeObjectModeTwoHopAssertedHead";
            let other_mode = "OtherFourEdgeObjectModeTwoHopAssertedHead";
            let all_modes = [
                base_mode,
                inner_mode,
                middle_mode,
                outer_mode,
                too_deep_mode,
            ];
            let symbols = source_local_symbols_env(
                module.clone(),
                &[
                    (base_mode, SymbolKind::Mode),
                    (inner_mode, SymbolKind::Mode),
                    (middle_mode, SymbolKind::Mode),
                    (outer_mode, SymbolKind::Mode),
                    (too_deep_mode, SymbolKind::Mode),
                    (deeper_mode, SymbolKind::Mode),
                    (other_mode, SymbolKind::Mode),
                ],
            );
            let theorem = exact_four_edge_local_object_mode_two_hop_asserted_head_spec();
            let exact_modes = || {
                vec![
                    mode_definition_with_label(
                        base_mode,
                        "BaseFourEdgeObjectModeTwoHopAssertedHeadDef",
                        ReserveTypeShape::Builtin("object"),
                    ),
                    mode_definition_with_label(
                        inner_mode,
                        "InnerFourEdgeObjectModeTwoHopAssertedHeadDef",
                        ReserveTypeShape::QualifiedSymbol(base_mode),
                    ),
                    mode_definition_with_label(
                        middle_mode,
                        "MiddleFourEdgeObjectModeTwoHopAssertedHeadDef",
                        ReserveTypeShape::QualifiedSymbol(inner_mode),
                    ),
                    mode_definition_with_label(
                        outer_mode,
                        "OuterFourEdgeObjectModeTwoHopAssertedHeadDef",
                        ReserveTypeShape::QualifiedSymbol(middle_mode),
                    ),
                    mode_definition_with_label(
                        too_deep_mode,
                        "TooDeepFourEdgeObjectModeTwoHopAssertedHeadDef",
                        ReserveTypeShape::QualifiedSymbol(outer_mode),
                    ),
                ]
            };
            let reserve = || {
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbol(too_deep_mode),
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
            let payload = extract_source_four_edge_local_object_mode_two_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("the exact object-terminal four-edge two-hop source should extract");
            assert_eq!(payload.reserve.mode_expansions.len(), 5);
            assert_eq!(payload.reserve.bridge.bindings().len(), 1);
            assert_eq!(payload.subject_lookup_ordinal, 1);
            let source_binding = &payload.reserve.bridge.bindings()[0];
            assert_eq!(source_binding.type_spelling, too_deep_mode);
            assert_eq!(payload.asserted_type.spelling, middle_mode);
            let expansion = |spelling| {
                payload
                    .reserve
                    .mode_expansions
                    .iter()
                    .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(spelling))
                    .map(|(_, expansion)| expansion)
                    .unwrap()
            };
            let too_deep_expansion = expansion(too_deep_mode);
            let outer_expansion = expansion(outer_mode);
            let middle_expansion = expansion(middle_mode);
            let inner_expansion = expansion(inner_mode);
            let base_expansion = expansion(base_mode);
            assert_eq!(too_deep_expansion.radix.spelling, outer_mode);
            assert_eq!(outer_expansion.radix.spelling, middle_mode);
            assert_eq!(outer_expansion.radix.head, payload.asserted_type.head);
            assert_eq!(middle_expansion.radix.spelling, inner_mode);
            assert_eq!(inner_expansion.radix.spelling, base_mode);
            assert_eq!(base_expansion.radix.spelling, "object");
            assert_eq!(base_expansion.radix.head, TypeHeadInput::BuiltinObject);
            let TypeHeadInput::Symbol(too_deep_symbol) = &source_binding.type_head else {
                panic!("TooDeep binding must resolve to a symbol")
            };
            let TypeHeadInput::Symbol(outer_symbol) = &too_deep_expansion.radix.head else {
                panic!("TooDeep expansion must resolve to the Outer symbol")
            };
            let TypeHeadInput::Symbol(middle_symbol) = &payload.asserted_type.head else {
                panic!("asserted head must resolve to the Middle symbol")
            };
            assert_ne!(too_deep_symbol, outer_symbol);
            assert_ne!(too_deep_symbol, middle_symbol);
            assert_ne!(outer_symbol, middle_symbol);

            let exact_output = || {
                source_four_edge_local_object_mode_two_hop_asserted_head_output(
                    &exact,
                    module.clone(),
                    &symbols,
                )
                .unwrap()
            };
            let output = exact_output();
            assert_source_reserved_variable_type_assertion_output(&output)
                .expect("Task 216 exact checker output must satisfy every invariant");
            assert_eq!(output.subject_binding, BindingId::new(0));
            assert_eq!(output.payload.subject_lookup_ordinal, 1);
            assert_eq!(output.payload.reserve.mode_expansions.len(), 5);
            assert_eq!(output.term_formula.type_entries().len(), 3);
            assert_eq!(output.term_formula.normalized_types().len(), 1);
            assert_eq!(output.subject_result_input.spelling, too_deep_mode);
            assert_eq!(output.asserted_type_input.spelling, middle_mode);
            assert_ne!(
                output.subject_result_input.head,
                output.asserted_type_input.head
            );
            assert_ne!(
                output.subject_result_input.site,
                output.asserted_type_input.site
            );
            assert_ne!(
                output.subject_result_input.source_range,
                output.asserted_type_input.source_range
            );
            let (_, normalized) = output
                .term_formula
                .normalized_types()
                .iter()
                .next()
                .unwrap();
            assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
            assert_ne!(normalized.head, TypeHeadRef::BuiltinSet);
            assert_eq!(normalized.source.range, base_expansion.radix.source_range);
            assert_eq!(normalized.source.spelling, "object");
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
                    assert_source_reserved_variable_type_assertion_output(&invalid)
                        .map(|()| invalid);
                assert_eq!(
                    source_reserved_variable_type_assertion_result_detail_keys(
                        invalid_result,
                        TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_TWO_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
                    ),
                    vec![
                        TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_TWO_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY
                            .to_owned()
                    ]
                );
            };
            for removed in all_modes {
                let mut invalid = exact_output();
                invalid
                    .payload
                    .reserve
                    .mode_expansions
                    .retain(|symbol, _| source_mode_symbol_spelling(symbol) != Some(removed));
                assert_invalid_output(invalid);
            }
            let mut corruptions = Vec::new();
            let mut invalid = exact_output();
            invalid.subject_binding = BindingId::new(1);
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.payload.subject_lookup_ordinal = 2;
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.subject_result_input.spelling = outer_mode.to_owned();
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.subject_result_input.head = invalid.asserted_type_input.head.clone();
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.subject_result_input.site = invalid.asserted_type_input.site.clone();
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.subject_result_input.source_range = invalid.asserted_type_input.source_range;
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.asserted_type_input.spelling = outer_mode.to_owned();
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.asserted_type_input.head = invalid.subject_result_input.head.clone();
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.asserted_type_input.site = invalid.subject_result_input.site.clone();
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.asserted_type_input.source_range = invalid.subject_result_input.source_range;
            corruptions.push(invalid);
            let mut invalid = exact_output();
            let (_, too_deep) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(too_deep_mode))
                .unwrap();
            too_deep.radix.spelling = middle_mode.to_owned();
            too_deep.radix.head = invalid.asserted_type_input.head.clone();
            corruptions.push(invalid);
            let mut invalid = exact_output();
            let inner_symbol = invalid
                .payload
                .reserve
                .mode_expansions
                .keys()
                .find(|symbol| source_mode_symbol_spelling(symbol) == Some(inner_mode))
                .unwrap()
                .clone();
            let (_, outer) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(outer_mode))
                .unwrap();
            outer.radix.spelling = inner_mode.to_owned();
            outer.radix.head = TypeHeadInput::Symbol(inner_symbol);
            corruptions.push(invalid);
            let mut invalid = exact_output();
            let base_symbol = invalid
                .payload
                .reserve
                .mode_expansions
                .keys()
                .find(|symbol| source_mode_symbol_spelling(symbol) == Some(base_mode))
                .unwrap()
                .clone();
            let (_, middle) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(middle_mode))
                .unwrap();
            middle.radix.spelling = base_mode.to_owned();
            middle.radix.head = TypeHeadInput::Symbol(base_symbol);
            corruptions.push(invalid);
            let mut invalid = exact_output();
            let (_, inner) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(inner_mode))
                .unwrap();
            inner.radix.spelling = "object".to_owned();
            inner.radix.head = TypeHeadInput::BuiltinObject;
            corruptions.push(invalid);
            let mut invalid = exact_output();
            let (_, base) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(base_mode))
                .unwrap();
            base.radix.spelling = "set".to_owned();
            base.radix.head = TypeHeadInput::BuiltinSet;
            corruptions.push(invalid);
            let mut invalid = exact_output();
            let (_, base) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(base_mode))
                .unwrap();
            base.radix.source_range = range(source_id, 0, 1);
            corruptions.push(invalid);
            for invalid in corruptions {
                assert_invalid_output(invalid);
            }
            assert_source_reserved_variable_type_assertion_output(&exact_output())
                .expect("mutating cloned Task 216 outputs must not mutate the exact output");

            let assert_extraction_gap = |ast, context: &str| {
                assert_eq!(
                    source_type_elaboration_detail_keys(&ast, module.clone(), &symbols),
                    vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()],
                    "{context}"
                );
            };
            let ordered = exact_modes();
            let mut permutation_count = 0;
            for a in 0..5 {
                for b in 0..5 {
                    for c in 0..5 {
                        for d in 0..5 {
                            for e in 0..5 {
                                let order = [a, b, c, d, e];
                                if order.iter().enumerate().any(|(index, value)| {
                                    order[..index].iter().any(|prior| prior == value)
                                }) {
                                    continue;
                                }
                                if order == [0, 1, 2, 3, 4] {
                                    continue;
                                }
                                permutation_count += 1;
                                assert_extraction_gap(
                                    mode_then_reserve_identifier_type_assertion_theorem_ast(
                                        source_id,
                                        vec![
                                            ordered[a], ordered[b], ordered[c], ordered[d],
                                            ordered[e],
                                        ],
                                        reserve(),
                                        theorem,
                                    ),
                                    &format!("definition permutation {a}{b}{c}{d}{e}"),
                                );
                            }
                        }
                    }
                }
            }
            assert_eq!(permutation_count, 119);
            for index in 0..5 {
                let expected_radix = match index {
                    0 => "object",
                    1 => base_mode,
                    2 => inner_mode,
                    3 => middle_mode,
                    _ => outer_mode,
                };
                let mut near_misses = Vec::new();
                let mut missing = exact_modes();
                missing.remove(index);
                near_misses.push(missing);
                let mut duplicate = exact_modes();
                duplicate.push(duplicate[index]);
                near_misses.push(duplicate);
                let mut wrong_label = exact_modes();
                wrong_label[index].label = Some("WrongFourEdgeObjectModeTwoHopAssertedHeadDef");
                near_misses.push(wrong_label);
                let mut wrong_spelling = exact_modes();
                wrong_spelling[index].pattern = other_mode;
                near_misses.push(wrong_spelling);
                let mut wrong_radix = exact_modes();
                wrong_radix[index].rhs_shape = if index == 0 {
                    ReserveTypeShape::Builtin("set")
                } else {
                    ReserveTypeShape::QualifiedSymbol(other_mode)
                };
                near_misses.push(wrong_radix);
                let mut recovered = exact_modes();
                recovered[index].recovered = true;
                near_misses.push(recovered);
                let mut contextual = exact_modes();
                contextual[index].local_context = true;
                near_misses.push(contextual);
                let mut parameterized = exact_modes();
                parameterized[index].parameterized_pattern = true;
                near_misses.push(parameterized);
                let mut args = exact_modes();
                args[index].rhs_shape = ReserveTypeShape::QualifiedSymbolWithArgs(expected_radix);
                near_misses.push(args);
                let mut attributes = exact_modes();
                attributes[index].rhs_shape = if index == 0 {
                    ReserveTypeShape::AttributedObject
                } else {
                    ReserveTypeShape::AttributedQualifiedSymbol(expected_radix)
                };
                near_misses.push(attributes);
                for (variant, modes) in near_misses.into_iter().enumerate() {
                    assert_extraction_gap(
                        mode_then_reserve_identifier_type_assertion_theorem_ast(
                            source_id,
                            modes,
                            reserve(),
                            theorem,
                        ),
                        &format!("definition {index} near miss {variant}"),
                    );
                }
            }

            let mut broken_too_deep_link = exact_modes();
            broken_too_deep_link[4].rhs_shape = ReserveTypeShape::QualifiedSymbol(middle_mode);
            let mut broken_outer_link = exact_modes();
            broken_outer_link[3].rhs_shape = ReserveTypeShape::QualifiedSymbol(inner_mode);
            let mut broken_middle_tail = exact_modes();
            broken_middle_tail[2].rhs_shape = ReserveTypeShape::QualifiedSymbol(base_mode);
            let mut broken_inner_tail = exact_modes();
            broken_inner_tail[1].rhs_shape = ReserveTypeShape::Builtin("object");
            let mut broken_terminal = exact_modes();
            broken_terminal[0].rhs_shape = ReserveTypeShape::Builtin("set");
            for (context, modes) in [
                ("TooDeep-to-Outer relation link", broken_too_deep_link),
                ("Outer-to-Middle relation link", broken_outer_link),
                ("Middle-to-Inner terminal tail", broken_middle_tail),
                ("Inner-to-Base terminal tail", broken_inner_tail),
                ("Base-to-object terminal", broken_terminal),
            ] {
                assert_extraction_gap(
                    mode_then_reserve_identifier_type_assertion_theorem_ast(
                        source_id,
                        modes,
                        reserve(),
                        theorem,
                    ),
                    context,
                );
            }

            let mut source_near_misses = Vec::new();
            for bad_reserve in [
                ReserveTypeShape::QualifiedSymbol(base_mode),
                ReserveTypeShape::QualifiedSymbol(inner_mode),
                ReserveTypeShape::QualifiedSymbol(middle_mode),
                ReserveTypeShape::QualifiedSymbol(outer_mode),
                ReserveTypeShape::QualifiedSymbolWithArgs(too_deep_mode),
                ReserveTypeShape::AttributedQualifiedSymbol(too_deep_mode),
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
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::QualifiedSymbol(too_deep_mode)),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("object")),
                ],
                theorem,
            ));
            for near_miss_theorem in [
                IdentifierTypeAssertionTheoremSpec {
                    label: "OtherPayloadBoundary",
                    ..theorem
                },
                IdentifierTypeAssertionTheoremSpec {
                    subject: "y",
                    ..theorem
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(too_deep_mode),
                    ..theorem
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(outer_mode),
                    ..theorem
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(inner_mode),
                    ..theorem
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(base_mode),
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
                    asserted_type: ReserveTypeShape::QualifiedSymbol(other_mode),
                    ..theorem
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbolWithArgs(middle_mode),
                    ..theorem
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::AttributedQualifiedSymbol(middle_mode),
                    ..theorem
                },
                IdentifierTypeAssertionTheoremSpec {
                    negated: true,
                    ..theorem
                },
                IdentifierTypeAssertionTheoremSpec {
                    status: Some("registration"),
                    ..theorem
                },
                IdentifierTypeAssertionTheoremSpec {
                    recovered_label: true,
                    ..theorem
                },
            ] {
                source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    exact_modes(),
                    reserve(),
                    near_miss_theorem,
                ));
            }
            let mut connected_deeper_modes = exact_modes();
            connected_deeper_modes.push(mode_definition_with_label(
                deeper_mode,
                "DeeperFourEdgeObjectModeTwoHopAssertedHeadDef",
                ReserveTypeShape::QualifiedSymbol(too_deep_mode),
            ));
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                connected_deeper_modes.clone(),
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbol(deeper_mode),
                )],
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(outer_mode),
                    ..theorem
                },
            ));
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                connected_deeper_modes,
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(deeper_mode),
                    ..theorem
                },
            ));
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

            let unrelated_import = source_local_and_imported_symbols_env(
                module.clone(),
                &all_modes.map(|spelling| (spelling, SymbolKind::Mode)),
                &[(
                    "UnrelatedFourEdgeObjectModeTwoHopAssertedHead",
                    SymbolKind::Mode,
                )],
            );
            assert!(
                extract_source_four_edge_local_object_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &unrelated_import,
                )
                .is_some()
            );
            for imported_index in 0..5 {
                let locals = all_modes
                    .iter()
                    .enumerate()
                    .filter(|(index, _)| *index != imported_index)
                    .map(|(_, spelling)| (*spelling, SymbolKind::Mode))
                    .collect::<Vec<_>>();
                for imports in [
                    vec![(all_modes[imported_index], SymbolKind::Mode)],
                    vec![
                        (all_modes[imported_index], SymbolKind::Mode),
                        (all_modes[imported_index], SymbolKind::Mode),
                    ],
                ] {
                    let provenance_near_miss =
                        source_local_and_imported_symbols_env(module.clone(), &locals, &imports);
                    assert_eq!(
                        source_type_elaboration_detail_keys(
                            &exact,
                            module.clone(),
                            &provenance_near_miss,
                        ),
                        vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
                    );
                }
            }
            for imports in [
                all_modes
                    .iter()
                    .map(|spelling| (*spelling, SymbolKind::Mode))
                    .collect::<Vec<_>>(),
                all_modes
                    .iter()
                    .flat_map(|spelling| {
                        [(*spelling, SymbolKind::Mode), (*spelling, SymbolKind::Mode)]
                    })
                    .collect::<Vec<_>>(),
            ] {
                let provenance_near_miss =
                    source_local_and_imported_symbols_env(module.clone(), &[], &imports);
                assert_eq!(
                    source_type_elaboration_detail_keys(
                        &exact,
                        module.clone(),
                        &provenance_near_miss,
                    ),
                    vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
                );
            }
        }

        #[test]
        fn task216_synthetic_is_rejected_by_all_41_prior_type_assertion_extractors() {
            let source_id = source_id(216);
            let module = ResolverModuleId::new(
                PackageId::new("test"),
                ModulePath::new("task216_prior_extractor_isolation"),
            );
            let names = [
                "BaseFourEdgeObjectModeTwoHopAssertedHead",
                "InnerFourEdgeObjectModeTwoHopAssertedHead",
                "MiddleFourEdgeObjectModeTwoHopAssertedHead",
                "OuterFourEdgeObjectModeTwoHopAssertedHead",
                "TooDeepFourEdgeObjectModeTwoHopAssertedHead",
            ];
            let symbols = source_local_symbols_env(
                module.clone(),
                &names.map(|spelling| (spelling, SymbolKind::Mode)),
            );
            let exact = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                vec![
                    mode_definition_with_label(
                        names[0],
                        "BaseFourEdgeObjectModeTwoHopAssertedHeadDef",
                        ReserveTypeShape::Builtin("object"),
                    ),
                    mode_definition_with_label(
                        names[1],
                        "InnerFourEdgeObjectModeTwoHopAssertedHeadDef",
                        ReserveTypeShape::QualifiedSymbol(names[0]),
                    ),
                    mode_definition_with_label(
                        names[2],
                        "MiddleFourEdgeObjectModeTwoHopAssertedHeadDef",
                        ReserveTypeShape::QualifiedSymbol(names[1]),
                    ),
                    mode_definition_with_label(
                        names[3],
                        "OuterFourEdgeObjectModeTwoHopAssertedHeadDef",
                        ReserveTypeShape::QualifiedSymbol(names[2]),
                    ),
                    mode_definition_with_label(
                        names[4],
                        "TooDeepFourEdgeObjectModeTwoHopAssertedHeadDef",
                        ReserveTypeShape::QualifiedSymbol(names[3]),
                    ),
                ],
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbol(names[4]),
                )],
                exact_four_edge_local_object_mode_two_hop_asserted_head_spec(),
            );
            assert!(
                extract_source_four_edge_local_object_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                )
                .is_some()
            );
            let prior_results = [
                extract_source_reserved_variable_type_assertion(&exact, module.clone(), &symbols),
                super::super::extract_source_reserved_object_variable_type_assertion(
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
                extract_source_chained_local_mode_radix_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
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
                extract_source_three_edge_local_mode_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
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
                super::super::extract_source_four_edge_local_mode_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                super::super::extract_source_four_edge_local_object_mode_asserted_head(
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
                super::super::extract_source_local_mode_long_chain_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                super::super::extract_source_local_object_mode_long_chain_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_local_mode_long_chain_radix_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                super::super::extract_source_local_object_mode_long_chain_radix_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_two_edge_local_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_two_edge_local_object_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_three_edge_local_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_three_edge_local_object_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_four_edge_local_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
            ];
            assert_eq!(prior_results.len(), 41);
            assert!(prior_results.into_iter().all(|result| result.is_none()));
        }
    }

    mod task_217_three_edge_three_hop_asserted_head {
        use super::*;

        const BASE: &str = "BaseThreeEdgeModeThreeHopAssertedHead";
        const INNER: &str = "InnerThreeEdgeModeThreeHopAssertedHead";
        const MIDDLE: &str = "MiddleThreeEdgeModeThreeHopAssertedHead";
        const OUTER: &str = "OuterThreeEdgeModeThreeHopAssertedHead";
        const OTHER: &str = "OtherThreeEdgeModeThreeHopAssertedHead";

        fn theorem() -> IdentifierTypeAssertionTheoremSpec<'static> {
            IdentifierTypeAssertionTheoremSpec {
                status: None,
                label: "ThreeEdgeLocalModeThreeHopAssertedHeadPayloadBoundary",
                subject: "x",
                asserted_type: ReserveTypeShape::QualifiedSymbol(BASE),
                recovered_label: false,
                negated: false,
            }
        }

        fn exact_modes() -> Vec<ModeDefinitionSpec> {
            vec![
                mode_definition_with_label(
                    BASE,
                    "BaseThreeEdgeModeThreeHopAssertedHeadDef",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition_with_label(
                    INNER,
                    "InnerThreeEdgeModeThreeHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(BASE),
                ),
                mode_definition_with_label(
                    MIDDLE,
                    "MiddleThreeEdgeModeThreeHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(INNER),
                ),
                mode_definition_with_label(
                    OUTER,
                    "OuterThreeEdgeModeThreeHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(MIDDLE),
                ),
            ]
        }

        fn reserve() -> Vec<ReserveItemSpec> {
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol(OUTER),
            )]
        }

        #[test]
        fn active_fixture_consumes_four_real_expansions() {
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
            let plan = build_test_plan(&config).expect("Task 217 repository plan should build");
            let (ordinal, case) = active_type_elaboration_cases(&plan)
                .enumerate()
                .find(|(_, case)| {
                    case.id.0
                        == "pass_type_elaboration_three_edge_local_mode_three_hop_asserted_head_001"
                })
                .expect("Task 217 active fixture should be discoverable");
            let frontend = run_frontend(&workspace_root, case, ordinal)
                .expect("Task 217 fixture should run through the real frontend");
            assert!(frontend.diagnostics.is_empty());
            let ast = frontend
                .ast
                .expect("Task 217 fixture should produce an AST");
            let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
            assert!(resolver.detail_keys.is_empty());
            let symbols =
                augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
            assert_eq!(
                source_type_elaboration_detail_keys(&ast, resolver.module.clone(), &symbols),
                Vec::<String>::new()
            );
            let output = source_three_edge_local_mode_three_hop_asserted_head_output(
                &ast,
                resolver.module,
                &symbols,
            )
            .expect("Task 217 real AST should reach the exact three-link asserted-head seam");
            assert_source_reserved_variable_type_assertion_output(&output)
                .expect("Task 217 real AST should preserve every checked payload invariant");
            assert_eq!(output.payload.reserve.mode_expansions.len(), 4);
            assert_eq!(output.subject_binding, BindingId::new(0));
            assert_eq!(output.payload.subject_lookup_ordinal, 1);
            assert_eq!(output.term_formula.type_entries().len(), 3);
            assert_eq!(output.term_formula.normalized_types().len(), 1);
            assert_eq!(output.subject_result_input.spelling, OUTER);
            assert_eq!(output.asserted_type_input.spelling, BASE);
            assert_ne!(
                output.subject_result_input.head,
                output.asserted_type_input.head
            );
            assert_ne!(
                output.subject_result_input.site,
                output.asserted_type_input.site
            );
            assert_ne!(
                output.subject_result_input.source_range,
                output.asserted_type_input.source_range
            );
            let expansion = |spelling| {
                output
                    .payload
                    .reserve
                    .mode_expansions
                    .iter()
                    .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(spelling))
                    .map(|(_, expansion)| expansion)
                    .unwrap()
            };
            let outer = expansion(OUTER);
            let middle = expansion(MIDDLE);
            let inner = expansion(INNER);
            let base = expansion(BASE);
            assert_eq!(outer.radix.spelling, MIDDLE);
            assert_eq!(middle.radix.spelling, INNER);
            assert_eq!(inner.radix.spelling, BASE);
            assert_eq!(inner.radix.head, output.asserted_type_input.head);
            assert_eq!(base.radix.spelling, "set");
            assert_eq!(base.radix.head, TypeHeadInput::BuiltinSet);
            let (_, normalized) = output
                .term_formula
                .normalized_types()
                .iter()
                .next()
                .unwrap();
            assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
            assert_eq!(normalized.source.range, base.radix.source_range);
            assert_eq!(normalized.source.spelling, "set");
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
        fn synthetic_exactness_matrix_rejects_every_near_miss() {
            let source_id = source_id(217);
            let module = ResolverModuleId::new(
                PackageId::new("test"),
                ModulePath::new("three_edge_local_mode_three_hop_asserted_head"),
            );
            let all_modes = [BASE, INNER, MIDDLE, OUTER];
            let symbols = source_local_symbols_env(
                module.clone(),
                &[
                    (BASE, SymbolKind::Mode),
                    (INNER, SymbolKind::Mode),
                    (MIDDLE, SymbolKind::Mode),
                    (OUTER, SymbolKind::Mode),
                    (OTHER, SymbolKind::Mode),
                ],
            );
            let exact = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                theorem(),
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
                Vec::<String>::new()
            );
            let payload = extract_source_three_edge_local_mode_three_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("the exact set-terminal three-edge three-hop source should extract");
            assert_eq!(payload.reserve.mode_expansions.len(), 4);
            assert_eq!(payload.reserve.bridge.bindings().len(), 1);
            assert_eq!(payload.subject_lookup_ordinal, 1);
            assert_eq!(payload.reserve.bridge.bindings()[0].type_spelling, OUTER);
            assert_eq!(payload.asserted_type.spelling, BASE);

            let exact_output = || {
                source_three_edge_local_mode_three_hop_asserted_head_output(
                    &exact,
                    module.clone(),
                    &symbols,
                )
                .unwrap()
            };
            let output = exact_output();
            assert_source_reserved_variable_type_assertion_output(&output)
                .expect("Task 217 exact checker output must satisfy every invariant");

            let assert_invalid_output = |invalid| {
                let invalid_result =
                    assert_source_reserved_variable_type_assertion_output(&invalid)
                        .map(|()| invalid);
                assert_eq!(
                    source_reserved_variable_type_assertion_result_detail_keys(
                        invalid_result,
                        TYPE_ELABORATION_THREE_EDGE_LOCAL_MODE_THREE_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
                    ),
                    vec![
                        TYPE_ELABORATION_THREE_EDGE_LOCAL_MODE_THREE_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY
                            .to_owned()
                    ]
                );
            };
            for removed in all_modes {
                let mut invalid = exact_output();
                invalid
                    .payload
                    .reserve
                    .mode_expansions
                    .retain(|symbol, _| source_mode_symbol_spelling(symbol) != Some(removed));
                assert_invalid_output(invalid);
            }
            let mut corruptions = Vec::new();
            let mut invalid = exact_output();
            invalid.subject_binding = BindingId::new(1);
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.payload.subject_lookup_ordinal = 2;
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.subject_result_input.spelling = MIDDLE.to_owned();
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.subject_result_input.head = invalid.asserted_type_input.head.clone();
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.subject_result_input.site = invalid.asserted_type_input.site.clone();
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.subject_result_input.source_range = invalid.asserted_type_input.source_range;
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.asserted_type_input.spelling = INNER.to_owned();
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.asserted_type_input.head = invalid.subject_result_input.head.clone();
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.asserted_type_input.site = invalid.subject_result_input.site.clone();
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.asserted_type_input.source_range = invalid.subject_result_input.source_range;
            corruptions.push(invalid);

            let mut invalid = exact_output();
            let inner_symbol = invalid
                .payload
                .reserve
                .mode_expansions
                .keys()
                .find(|symbol| source_mode_symbol_spelling(symbol) == Some(INNER))
                .unwrap()
                .clone();
            let (_, outer) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(OUTER))
                .unwrap();
            outer.radix.spelling = INNER.to_owned();
            outer.radix.head = TypeHeadInput::Symbol(inner_symbol);
            corruptions.push(invalid);
            let mut invalid = exact_output();
            let base_symbol = invalid
                .payload
                .reserve
                .mode_expansions
                .keys()
                .find(|symbol| source_mode_symbol_spelling(symbol) == Some(BASE))
                .unwrap()
                .clone();
            let (_, middle) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(MIDDLE))
                .unwrap();
            middle.radix.spelling = BASE.to_owned();
            middle.radix.head = TypeHeadInput::Symbol(base_symbol);
            corruptions.push(invalid);
            let mut invalid = exact_output();
            let (_, inner) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(INNER))
                .unwrap();
            inner.radix.spelling = "set".to_owned();
            inner.radix.head = TypeHeadInput::BuiltinSet;
            corruptions.push(invalid);
            let mut invalid = exact_output();
            let (_, base) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(BASE))
                .unwrap();
            base.radix.spelling = "object".to_owned();
            base.radix.head = TypeHeadInput::BuiltinObject;
            corruptions.push(invalid);
            let mut invalid = exact_output();
            let (_, base) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(BASE))
                .unwrap();
            base.radix.source_range = range(source_id, 0, 1);
            corruptions.push(invalid);
            for invalid in corruptions {
                assert_invalid_output(invalid);
            }
            assert_source_reserved_variable_type_assertion_output(&exact_output())
                .expect("mutating cloned Task 217 outputs must not mutate the exact output");

            let assert_extraction_gap = |ast, context: &str| {
                assert_eq!(
                    source_type_elaboration_detail_keys(&ast, module.clone(), &symbols),
                    vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()],
                    "{context}"
                );
            };
            let ordered = exact_modes();
            let mut permutation_count = 0;
            for a in 0..4 {
                for b in 0..4 {
                    for c in 0..4 {
                        for d in 0..4 {
                            if a == b || a == c || a == d || b == c || b == d || c == d {
                                continue;
                            }
                            if [a, b, c, d] == [0, 1, 2, 3] {
                                continue;
                            }
                            permutation_count += 1;
                            assert_extraction_gap(
                                mode_then_reserve_identifier_type_assertion_theorem_ast(
                                    source_id,
                                    vec![ordered[a], ordered[b], ordered[c], ordered[d]],
                                    reserve(),
                                    theorem(),
                                ),
                                &format!("definition permutation {a}{b}{c}{d}"),
                            );
                        }
                    }
                }
            }
            assert_eq!(permutation_count, 23);

            for index in 0..4 {
                let expected_radix = match index {
                    0 => "set",
                    1 => BASE,
                    2 => INNER,
                    _ => MIDDLE,
                };
                let mut near_misses = Vec::new();
                let mut missing = exact_modes();
                missing.remove(index);
                near_misses.push(missing);
                let mut duplicate = exact_modes();
                duplicate.push(duplicate[index]);
                near_misses.push(duplicate);
                let mut wrong_label = exact_modes();
                wrong_label[index].label = Some("WrongThreeEdgeModeThreeHopAssertedHeadDef");
                near_misses.push(wrong_label);
                let mut wrong_pattern = exact_modes();
                wrong_pattern[index].pattern = OTHER;
                near_misses.push(wrong_pattern);
                let mut wrong_radix = exact_modes();
                wrong_radix[index].rhs_shape = if index == 0 {
                    ReserveTypeShape::Builtin("object")
                } else {
                    ReserveTypeShape::QualifiedSymbol(OTHER)
                };
                near_misses.push(wrong_radix);
                let mut recovered = exact_modes();
                recovered[index].recovered = true;
                near_misses.push(recovered);
                let mut contextual = exact_modes();
                contextual[index].local_context = true;
                near_misses.push(contextual);
                let mut parameterized = exact_modes();
                parameterized[index].parameterized_pattern = true;
                near_misses.push(parameterized);
                let mut args = exact_modes();
                args[index].rhs_shape = ReserveTypeShape::QualifiedSymbolWithArgs(expected_radix);
                near_misses.push(args);
                let mut attributed = exact_modes();
                attributed[index].rhs_shape = if index == 0 {
                    ReserveTypeShape::AttributedSet
                } else {
                    ReserveTypeShape::AttributedQualifiedSymbol(expected_radix)
                };
                near_misses.push(attributed);
                for (variant, modes) in near_misses.into_iter().enumerate() {
                    assert_extraction_gap(
                        mode_then_reserve_identifier_type_assertion_theorem_ast(
                            source_id,
                            modes,
                            reserve(),
                            theorem(),
                        ),
                        &format!("definition {index} near miss {variant}"),
                    );
                }
            }

            for (context, index, wrong_radix) in [
                ("Outer-to-Middle relation link", 3, INNER),
                ("Middle-to-Inner relation link", 2, BASE),
                ("Inner-to-Base relation link", 1, MIDDLE),
            ] {
                let mut modes = exact_modes();
                modes[index].rhs_shape = ReserveTypeShape::QualifiedSymbol(wrong_radix);
                assert_extraction_gap(
                    mode_then_reserve_identifier_type_assertion_theorem_ast(
                        source_id,
                        modes,
                        reserve(),
                        theorem(),
                    ),
                    context,
                );
            }

            let mut source_near_misses = Vec::new();
            for bad_reserve in [
                ReserveTypeShape::QualifiedSymbol(BASE),
                ReserveTypeShape::QualifiedSymbol(INNER),
                ReserveTypeShape::QualifiedSymbol(MIDDLE),
                ReserveTypeShape::QualifiedSymbolWithArgs(OUTER),
                ReserveTypeShape::AttributedQualifiedSymbol(OUTER),
                ReserveTypeShape::Builtin("set"),
                ReserveTypeShape::Builtin("object"),
            ] {
                source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    exact_modes(),
                    vec![reserve_item(vec!["x"], bad_reserve)],
                    theorem(),
                ));
            }
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::QualifiedSymbol(OUTER)),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                ],
                theorem(),
            ));
            for near_miss_theorem in [
                IdentifierTypeAssertionTheoremSpec {
                    label: "OtherPayloadBoundary",
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    subject: "y",
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(INNER),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(MIDDLE),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(OUTER),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::Builtin("set"),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::Builtin("object"),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(OTHER),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbolWithArgs(BASE),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::AttributedQualifiedSymbol(BASE),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    negated: true,
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    status: Some("registration"),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    recovered_label: true,
                    ..theorem()
                },
            ] {
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
                    theorem(),
                ),
            );
            for (index, near_miss) in source_near_misses.into_iter().enumerate() {
                assert_extraction_gap(near_miss, &format!("source near miss {index}"));
            }

            let unrelated_import = source_local_and_imported_symbols_env(
                module.clone(),
                &all_modes.map(|spelling| (spelling, SymbolKind::Mode)),
                &[("UnrelatedThreeHopMode", SymbolKind::Mode)],
            );
            assert!(
                extract_source_three_edge_local_mode_three_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &unrelated_import,
                )
                .is_some()
            );
            for imported_index in 0..4 {
                let locals = all_modes
                    .iter()
                    .enumerate()
                    .filter(|(index, _)| *index != imported_index)
                    .map(|(_, spelling)| (*spelling, SymbolKind::Mode))
                    .collect::<Vec<_>>();
                for imports in [
                    vec![(all_modes[imported_index], SymbolKind::Mode)],
                    vec![
                        (all_modes[imported_index], SymbolKind::Mode),
                        (all_modes[imported_index], SymbolKind::Mode),
                    ],
                ] {
                    let provenance_near_miss =
                        source_local_and_imported_symbols_env(module.clone(), &locals, &imports);
                    assert_eq!(
                        source_type_elaboration_detail_keys(
                            &exact,
                            module.clone(),
                            &provenance_near_miss,
                        ),
                        vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
                    );
                }
            }
            for imports in [
                all_modes
                    .iter()
                    .map(|spelling| (*spelling, SymbolKind::Mode))
                    .collect::<Vec<_>>(),
                all_modes
                    .iter()
                    .flat_map(|spelling| {
                        [(*spelling, SymbolKind::Mode), (*spelling, SymbolKind::Mode)]
                    })
                    .collect::<Vec<_>>(),
            ] {
                let provenance_near_miss =
                    source_local_and_imported_symbols_env(module.clone(), &[], &imports);
                assert_eq!(
                    source_type_elaboration_detail_keys(
                        &exact,
                        module.clone(),
                        &provenance_near_miss,
                    ),
                    vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
                );
            }
        }

        #[test]
        fn route_rejects_all_42_prior_owner_fixtures_including_tasks_211_through_216() {
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
            let plan = build_test_plan(&config).expect("Task 217 repository plan should build");
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
            ];
            assert_eq!(prior_owner_ids.len(), 42);
            assert_eq!(
                &prior_owner_ids[36..],
                &[
                    "pass_type_elaboration_two_edge_local_mode_two_hop_asserted_head_001",
                    "pass_type_elaboration_two_edge_local_object_mode_two_hop_asserted_head_001",
                    "pass_type_elaboration_three_edge_local_mode_two_hop_asserted_head_001",
                    "pass_type_elaboration_three_edge_local_object_mode_two_hop_asserted_head_001",
                    "pass_type_elaboration_four_edge_local_mode_two_hop_asserted_head_001",
                    "pass_type_elaboration_four_edge_local_object_mode_two_hop_asserted_head_001",
                ]
            );
            for owner_id in prior_owner_ids {
                let (ordinal, case) = active_type_elaboration_cases(&plan)
                    .enumerate()
                    .find(|(_, case)| case.id.0 == owner_id)
                    .unwrap_or_else(|| panic!("prior owner fixture {owner_id} must be active"));
                let frontend = run_frontend(&workspace_root, case, ordinal)
                    .unwrap_or_else(|error| panic!("owner fixture {owner_id} must parse: {error}"));
                assert!(frontend.diagnostics.is_empty());
                let ast = frontend
                    .ast
                    .unwrap_or_else(|| panic!("owner fixture {owner_id} must produce an AST"));
                let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
                assert!(resolver.detail_keys.is_empty());
                let symbols =
                    augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
                assert_eq!(
                    source_type_elaboration_detail_keys(&ast, resolver.module.clone(), &symbols),
                    Vec::<String>::new(),
                    "prior owner fixture {owner_id} must still reach its active route"
                );
                assert!(
                    extract_source_three_edge_local_mode_three_hop_asserted_head(
                        &ast,
                        resolver.module,
                        &symbols,
                    )
                    .is_none(),
                    "Task 217 must reject prior owner fixture {owner_id}"
                );
            }
        }

        #[test]
        fn task217_synthetic_is_rejected_by_all_42_prior_type_assertion_extractors() {
            let source_id = source_id(217);
            let module = ResolverModuleId::new(
                PackageId::new("test"),
                ModulePath::new("task217_prior_extractor_isolation"),
            );
            let symbols = source_local_symbols_env(
                module.clone(),
                &[
                    (BASE, SymbolKind::Mode),
                    (INNER, SymbolKind::Mode),
                    (MIDDLE, SymbolKind::Mode),
                    (OUTER, SymbolKind::Mode),
                ],
            );
            let exact = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                theorem(),
            );
            assert!(
                extract_source_three_edge_local_mode_three_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                )
                .is_some()
            );
            let prior_results = [
                extract_source_reserved_variable_type_assertion(&exact, module.clone(), &symbols),
                super::super::extract_source_reserved_object_variable_type_assertion(
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
                extract_source_chained_local_mode_radix_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
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
                extract_source_three_edge_local_mode_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
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
                super::super::extract_source_four_edge_local_mode_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                super::super::extract_source_four_edge_local_object_mode_asserted_head(
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
                super::super::extract_source_local_mode_long_chain_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                super::super::extract_source_local_object_mode_long_chain_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_local_mode_long_chain_radix_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                super::super::extract_source_local_object_mode_long_chain_radix_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_two_edge_local_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_two_edge_local_object_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_three_edge_local_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_three_edge_local_object_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_four_edge_local_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_four_edge_local_object_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
            ];
            assert_eq!(prior_results.len(), 42);
            assert!(prior_results.into_iter().all(|result| result.is_none()));
        }
    }
    mod task_218_three_edge_object_three_hop_asserted_head {
        use super::*;

        const BASE: &str = "BaseThreeEdgeObjectModeThreeHopAssertedHead";
        const INNER: &str = "InnerThreeEdgeObjectModeThreeHopAssertedHead";
        const MIDDLE: &str = "MiddleThreeEdgeObjectModeThreeHopAssertedHead";
        const OUTER: &str = "OuterThreeEdgeObjectModeThreeHopAssertedHead";
        const OTHER: &str = "OtherThreeEdgeObjectModeThreeHopAssertedHead";
        const DEEPER: &str = "DeeperThreeEdgeObjectModeThreeHopAssertedHead";

        fn theorem() -> IdentifierTypeAssertionTheoremSpec<'static> {
            IdentifierTypeAssertionTheoremSpec {
                status: None,
                label: "ThreeEdgeLocalObjectModeThreeHopAssertedHeadPayloadBoundary",
                subject: "x",
                asserted_type: ReserveTypeShape::QualifiedSymbol(BASE),
                recovered_label: false,
                negated: false,
            }
        }

        fn exact_modes() -> Vec<ModeDefinitionSpec> {
            vec![
                mode_definition_with_label(
                    BASE,
                    "BaseThreeEdgeObjectModeThreeHopAssertedHeadDef",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition_with_label(
                    INNER,
                    "InnerThreeEdgeObjectModeThreeHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(BASE),
                ),
                mode_definition_with_label(
                    MIDDLE,
                    "MiddleThreeEdgeObjectModeThreeHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(INNER),
                ),
                mode_definition_with_label(
                    OUTER,
                    "OuterThreeEdgeObjectModeThreeHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(MIDDLE),
                ),
            ]
        }

        fn reserve() -> Vec<ReserveItemSpec> {
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol(OUTER),
            )]
        }

        #[test]
        fn active_fixture_consumes_four_real_expansions() {
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
            let plan = build_test_plan(&config).expect("Task 218 repository plan should build");
            let (ordinal, case) = active_type_elaboration_cases(&plan)
                .enumerate()
                .find(|(_, case)| {
                    case.id.0
                        == "pass_type_elaboration_three_edge_local_object_mode_three_hop_asserted_head_001"
                })
                .expect("Task 218 active fixture should be discoverable");
            let frontend = run_frontend(&workspace_root, case, ordinal)
                .expect("Task 218 fixture should run through the real frontend");
            assert!(frontend.diagnostics.is_empty());
            let ast = frontend
                .ast
                .expect("Task 218 fixture should produce an AST");
            let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
            assert!(resolver.detail_keys.is_empty());
            let symbols =
                augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
            assert_eq!(
                source_type_elaboration_detail_keys(&ast, resolver.module.clone(), &symbols),
                Vec::<String>::new()
            );
            let output = source_three_edge_local_object_mode_three_hop_asserted_head_output(
                &ast,
                resolver.module,
                &symbols,
            )
            .expect("Task 218 real AST should reach the exact three-link asserted-head seam");
            assert_source_reserved_variable_type_assertion_output(&output)
                .expect("Task 218 real AST should preserve every checked payload invariant");
            assert_eq!(output.payload.reserve.mode_expansions.len(), 4);
            assert_eq!(output.subject_binding, BindingId::new(0));
            assert_eq!(output.payload.subject_lookup_ordinal, 1);
            assert_eq!(output.term_formula.type_entries().len(), 3);
            assert_eq!(output.term_formula.normalized_types().len(), 1);
            assert_eq!(output.subject_result_input.spelling, OUTER);
            assert_eq!(output.asserted_type_input.spelling, BASE);
            assert_ne!(
                output.subject_result_input.head,
                output.asserted_type_input.head
            );
            assert_ne!(
                output.subject_result_input.site,
                output.asserted_type_input.site
            );
            assert_ne!(
                output.subject_result_input.source_range,
                output.asserted_type_input.source_range
            );
            let expansion = |spelling| {
                output
                    .payload
                    .reserve
                    .mode_expansions
                    .iter()
                    .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(spelling))
                    .map(|(_, expansion)| expansion)
                    .unwrap()
            };
            let outer = expansion(OUTER);
            let middle = expansion(MIDDLE);
            let inner = expansion(INNER);
            let base = expansion(BASE);
            assert_eq!(outer.radix.spelling, MIDDLE);
            assert_eq!(middle.radix.spelling, INNER);
            assert_eq!(inner.radix.spelling, BASE);
            assert_eq!(inner.radix.head, output.asserted_type_input.head);
            assert_eq!(base.radix.spelling, "object");
            assert_eq!(base.radix.head, TypeHeadInput::BuiltinObject);
            let (_, normalized) = output
                .term_formula
                .normalized_types()
                .iter()
                .next()
                .unwrap();
            assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
            assert_eq!(normalized.source.range, base.radix.source_range);
            assert_eq!(normalized.source.spelling, "object");
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
        fn synthetic_exactness_matrix_rejects_every_near_miss() {
            let source_id = source_id(218);
            let module = ResolverModuleId::new(
                PackageId::new("test"),
                ModulePath::new("three_edge_local_object_mode_three_hop_asserted_head"),
            );
            let all_modes = [BASE, INNER, MIDDLE, OUTER];
            let symbols = source_local_symbols_env(
                module.clone(),
                &[
                    (BASE, SymbolKind::Mode),
                    (INNER, SymbolKind::Mode),
                    (MIDDLE, SymbolKind::Mode),
                    (OUTER, SymbolKind::Mode),
                    (OTHER, SymbolKind::Mode),
                    (DEEPER, SymbolKind::Mode),
                ],
            );
            let exact = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                theorem(),
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
                Vec::<String>::new()
            );
            let payload = extract_source_three_edge_local_object_mode_three_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("the exact object-terminal three-edge three-hop source should extract");
            assert_eq!(payload.reserve.mode_expansions.len(), 4);
            assert_eq!(payload.reserve.bridge.bindings().len(), 1);
            assert_eq!(payload.subject_lookup_ordinal, 1);
            assert_eq!(payload.reserve.bridge.bindings()[0].type_spelling, OUTER);
            assert_eq!(payload.asserted_type.spelling, BASE);

            let exact_output = || {
                source_three_edge_local_object_mode_three_hop_asserted_head_output(
                    &exact,
                    module.clone(),
                    &symbols,
                )
                .unwrap()
            };
            let output = exact_output();
            assert_source_reserved_variable_type_assertion_output(&output)
                .expect("Task 218 exact checker output must satisfy every invariant");

            let assert_invalid_output = |invalid| {
                let invalid_result =
                    assert_source_reserved_variable_type_assertion_output(&invalid)
                        .map(|()| invalid);
                assert_eq!(
                    source_reserved_variable_type_assertion_result_detail_keys(
                        invalid_result,
                        TYPE_ELABORATION_THREE_EDGE_LOCAL_OBJECT_MODE_THREE_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
                    ),
                    vec![
                        TYPE_ELABORATION_THREE_EDGE_LOCAL_OBJECT_MODE_THREE_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY
                            .to_owned()
                    ]
                );
            };
            for removed in all_modes {
                let mut invalid = exact_output();
                invalid
                    .payload
                    .reserve
                    .mode_expansions
                    .retain(|symbol, _| source_mode_symbol_spelling(symbol) != Some(removed));
                assert_invalid_output(invalid);
            }
            let mut corruptions = Vec::new();
            let mut invalid = exact_output();
            invalid.subject_binding = BindingId::new(1);
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.payload.subject_lookup_ordinal = 2;
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.subject_result_input.spelling = MIDDLE.to_owned();
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.subject_result_input.head = invalid.asserted_type_input.head.clone();
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.subject_result_input.site = invalid.asserted_type_input.site.clone();
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.subject_result_input.source_range = invalid.asserted_type_input.source_range;
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.asserted_type_input.spelling = INNER.to_owned();
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.asserted_type_input.head = invalid.subject_result_input.head.clone();
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.asserted_type_input.site = invalid.subject_result_input.site.clone();
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.asserted_type_input.source_range = invalid.subject_result_input.source_range;
            corruptions.push(invalid);

            let mut invalid = exact_output();
            let inner_symbol = invalid
                .payload
                .reserve
                .mode_expansions
                .keys()
                .find(|symbol| source_mode_symbol_spelling(symbol) == Some(INNER))
                .unwrap()
                .clone();
            let (_, outer) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(OUTER))
                .unwrap();
            outer.radix.spelling = INNER.to_owned();
            outer.radix.head = TypeHeadInput::Symbol(inner_symbol);
            corruptions.push(invalid);
            let mut invalid = exact_output();
            let base_symbol = invalid
                .payload
                .reserve
                .mode_expansions
                .keys()
                .find(|symbol| source_mode_symbol_spelling(symbol) == Some(BASE))
                .unwrap()
                .clone();
            let (_, middle) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(MIDDLE))
                .unwrap();
            middle.radix.spelling = BASE.to_owned();
            middle.radix.head = TypeHeadInput::Symbol(base_symbol);
            corruptions.push(invalid);
            let mut invalid = exact_output();
            let (_, inner) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(INNER))
                .unwrap();
            inner.radix.spelling = "set".to_owned();
            inner.radix.head = TypeHeadInput::BuiltinSet;
            corruptions.push(invalid);
            let mut invalid = exact_output();
            let (_, base) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(BASE))
                .unwrap();
            base.radix.spelling = "set".to_owned();
            base.radix.head = TypeHeadInput::BuiltinSet;
            corruptions.push(invalid);
            let mut invalid = exact_output();
            let (_, base) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(BASE))
                .unwrap();
            base.radix.source_range = range(source_id, 0, 1);
            corruptions.push(invalid);
            for invalid in corruptions {
                assert_invalid_output(invalid);
            }
            assert_source_reserved_variable_type_assertion_output(&exact_output())
                .expect("mutating cloned Task 218 outputs must not mutate the exact output");

            let assert_extraction_gap = |ast, context: &str| {
                assert_eq!(
                    source_type_elaboration_detail_keys(&ast, module.clone(), &symbols),
                    vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()],
                    "{context}"
                );
            };
            let ordered = exact_modes();
            let mut permutation_count = 0;
            for a in 0..4 {
                for b in 0..4 {
                    for c in 0..4 {
                        for d in 0..4 {
                            if a == b || a == c || a == d || b == c || b == d || c == d {
                                continue;
                            }
                            if [a, b, c, d] == [0, 1, 2, 3] {
                                continue;
                            }
                            permutation_count += 1;
                            assert_extraction_gap(
                                mode_then_reserve_identifier_type_assertion_theorem_ast(
                                    source_id,
                                    vec![ordered[a], ordered[b], ordered[c], ordered[d]],
                                    reserve(),
                                    theorem(),
                                ),
                                &format!("definition permutation {a}{b}{c}{d}"),
                            );
                        }
                    }
                }
            }
            assert_eq!(permutation_count, 23);

            for index in 0..4 {
                let expected_radix = match index {
                    0 => "object",
                    1 => BASE,
                    2 => INNER,
                    _ => MIDDLE,
                };
                let mut near_misses = Vec::new();
                let mut missing = exact_modes();
                missing.remove(index);
                near_misses.push(missing);
                let mut duplicate = exact_modes();
                duplicate.push(duplicate[index]);
                near_misses.push(duplicate);
                let mut wrong_label = exact_modes();
                wrong_label[index].label = Some("WrongThreeEdgeObjectModeThreeHopAssertedHeadDef");
                near_misses.push(wrong_label);
                let mut wrong_pattern = exact_modes();
                wrong_pattern[index].pattern = OTHER;
                near_misses.push(wrong_pattern);
                let mut wrong_radix = exact_modes();
                wrong_radix[index].rhs_shape = if index == 0 {
                    ReserveTypeShape::Builtin("set")
                } else {
                    ReserveTypeShape::QualifiedSymbol(OTHER)
                };
                near_misses.push(wrong_radix);
                let mut recovered = exact_modes();
                recovered[index].recovered = true;
                near_misses.push(recovered);
                let mut contextual = exact_modes();
                contextual[index].local_context = true;
                near_misses.push(contextual);
                let mut parameterized = exact_modes();
                parameterized[index].parameterized_pattern = true;
                near_misses.push(parameterized);
                let mut args = exact_modes();
                args[index].rhs_shape = ReserveTypeShape::QualifiedSymbolWithArgs(expected_radix);
                near_misses.push(args);
                let mut attributed = exact_modes();
                attributed[index].rhs_shape = if index == 0 {
                    ReserveTypeShape::AttributedObject
                } else {
                    ReserveTypeShape::AttributedQualifiedSymbol(expected_radix)
                };
                near_misses.push(attributed);
                for (variant, modes) in near_misses.into_iter().enumerate() {
                    assert_extraction_gap(
                        mode_then_reserve_identifier_type_assertion_theorem_ast(
                            source_id,
                            modes,
                            reserve(),
                            theorem(),
                        ),
                        &format!("definition {index} near miss {variant}"),
                    );
                }
            }

            for (context, index, wrong_radix) in [
                ("Outer-to-Middle relation link", 3, INNER),
                ("Middle-to-Inner relation link", 2, BASE),
                ("Inner-to-Base relation link", 1, MIDDLE),
            ] {
                let mut modes = exact_modes();
                modes[index].rhs_shape = ReserveTypeShape::QualifiedSymbol(wrong_radix);
                assert_extraction_gap(
                    mode_then_reserve_identifier_type_assertion_theorem_ast(
                        source_id,
                        modes,
                        reserve(),
                        theorem(),
                    ),
                    context,
                );
            }

            let mut source_near_misses = Vec::new();
            for bad_reserve in [
                ReserveTypeShape::QualifiedSymbol(BASE),
                ReserveTypeShape::QualifiedSymbol(INNER),
                ReserveTypeShape::QualifiedSymbol(MIDDLE),
                ReserveTypeShape::QualifiedSymbolWithArgs(OUTER),
                ReserveTypeShape::AttributedQualifiedSymbol(OUTER),
                ReserveTypeShape::Builtin("set"),
                ReserveTypeShape::Builtin("object"),
            ] {
                source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    exact_modes(),
                    vec![reserve_item(vec!["x"], bad_reserve)],
                    theorem(),
                ));
            }
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::QualifiedSymbol(OUTER)),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                ],
                theorem(),
            ));
            for near_miss_theorem in [
                IdentifierTypeAssertionTheoremSpec {
                    label: "OtherPayloadBoundary",
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    subject: "y",
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(INNER),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(MIDDLE),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(OUTER),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::Builtin("set"),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::Builtin("object"),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(OTHER),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbolWithArgs(BASE),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::AttributedQualifiedSymbol(BASE),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    negated: true,
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    status: Some("registration"),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    recovered_label: true,
                    ..theorem()
                },
            ] {
                source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    exact_modes(),
                    reserve(),
                    near_miss_theorem,
                ));
            }
            let mut connected_deeper_modes = exact_modes();
            connected_deeper_modes.push(mode_definition_with_label(
                DEEPER,
                "DeeperThreeEdgeObjectModeThreeHopAssertedHeadDef",
                ReserveTypeShape::QualifiedSymbol(OUTER),
            ));
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                connected_deeper_modes,
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(DEEPER),
                    ..theorem()
                },
            ));
            source_near_misses.push(
                modes_then_empty_definition_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    exact_modes(),
                    reserve(),
                    theorem(),
                ),
            );
            for (index, near_miss) in source_near_misses.into_iter().enumerate() {
                assert_extraction_gap(near_miss, &format!("source near miss {index}"));
            }

            let unrelated_import = source_local_and_imported_symbols_env(
                module.clone(),
                &all_modes.map(|spelling| (spelling, SymbolKind::Mode)),
                &[("UnrelatedThreeHopMode", SymbolKind::Mode)],
            );
            assert!(
                extract_source_three_edge_local_object_mode_three_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &unrelated_import,
                )
                .is_some()
            );
            for imported_index in 0..4 {
                let locals = all_modes
                    .iter()
                    .enumerate()
                    .filter(|(index, _)| *index != imported_index)
                    .map(|(_, spelling)| (*spelling, SymbolKind::Mode))
                    .collect::<Vec<_>>();
                for imports in [
                    vec![(all_modes[imported_index], SymbolKind::Mode)],
                    vec![
                        (all_modes[imported_index], SymbolKind::Mode),
                        (all_modes[imported_index], SymbolKind::Mode),
                    ],
                ] {
                    let provenance_near_miss =
                        source_local_and_imported_symbols_env(module.clone(), &locals, &imports);
                    assert_eq!(
                        source_type_elaboration_detail_keys(
                            &exact,
                            module.clone(),
                            &provenance_near_miss,
                        ),
                        vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
                    );
                }
            }
            for imports in [
                all_modes
                    .iter()
                    .map(|spelling| (*spelling, SymbolKind::Mode))
                    .collect::<Vec<_>>(),
                all_modes
                    .iter()
                    .flat_map(|spelling| {
                        [(*spelling, SymbolKind::Mode), (*spelling, SymbolKind::Mode)]
                    })
                    .collect::<Vec<_>>(),
            ] {
                let provenance_near_miss =
                    source_local_and_imported_symbols_env(module.clone(), &[], &imports);
                assert_eq!(
                    source_type_elaboration_detail_keys(
                        &exact,
                        module.clone(),
                        &provenance_near_miss,
                    ),
                    vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
                );
            }
        }

        #[test]
        fn route_rejects_all_43_prior_owner_fixtures_including_tasks_211_through_217() {
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
            let plan = build_test_plan(&config).expect("Task 218 repository plan should build");
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
            ];
            assert_eq!(prior_owner_ids.len(), 43);
            assert_eq!(
                &prior_owner_ids[36..],
                &[
                    "pass_type_elaboration_two_edge_local_mode_two_hop_asserted_head_001",
                    "pass_type_elaboration_two_edge_local_object_mode_two_hop_asserted_head_001",
                    "pass_type_elaboration_three_edge_local_mode_two_hop_asserted_head_001",
                    "pass_type_elaboration_three_edge_local_object_mode_two_hop_asserted_head_001",
                    "pass_type_elaboration_four_edge_local_mode_two_hop_asserted_head_001",
                    "pass_type_elaboration_four_edge_local_object_mode_two_hop_asserted_head_001",
                    "pass_type_elaboration_three_edge_local_mode_three_hop_asserted_head_001",
                ]
            );
            for owner_id in prior_owner_ids {
                let (ordinal, case) = active_type_elaboration_cases(&plan)
                    .enumerate()
                    .find(|(_, case)| case.id.0 == owner_id)
                    .unwrap_or_else(|| panic!("prior owner fixture {owner_id} must be active"));
                let frontend = run_frontend(&workspace_root, case, ordinal)
                    .unwrap_or_else(|error| panic!("owner fixture {owner_id} must parse: {error}"));
                assert!(frontend.diagnostics.is_empty());
                let ast = frontend
                    .ast
                    .unwrap_or_else(|| panic!("owner fixture {owner_id} must produce an AST"));
                let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
                assert!(resolver.detail_keys.is_empty());
                let symbols =
                    augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
                assert_eq!(
                    source_type_elaboration_detail_keys(&ast, resolver.module.clone(), &symbols),
                    Vec::<String>::new(),
                    "prior owner fixture {owner_id} must still reach its active route"
                );
                assert!(
                    extract_source_three_edge_local_object_mode_three_hop_asserted_head(
                        &ast,
                        resolver.module,
                        &symbols,
                    )
                    .is_none(),
                    "Task 218 must reject prior owner fixture {owner_id}"
                );
            }
        }

        #[test]
        fn task218_synthetic_is_rejected_by_all_43_prior_type_assertion_extractors() {
            let source_id = source_id(218);
            let module = ResolverModuleId::new(
                PackageId::new("test"),
                ModulePath::new("task218_prior_extractor_isolation"),
            );
            let symbols = source_local_symbols_env(
                module.clone(),
                &[
                    (BASE, SymbolKind::Mode),
                    (INNER, SymbolKind::Mode),
                    (MIDDLE, SymbolKind::Mode),
                    (OUTER, SymbolKind::Mode),
                ],
            );
            let exact = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                theorem(),
            );
            assert!(
                extract_source_three_edge_local_object_mode_three_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                )
                .is_some()
            );
            let prior_results = [
                extract_source_reserved_variable_type_assertion(&exact, module.clone(), &symbols),
                super::super::extract_source_reserved_object_variable_type_assertion(
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
                extract_source_chained_local_mode_radix_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
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
                extract_source_three_edge_local_mode_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
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
                super::super::extract_source_four_edge_local_mode_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                super::super::extract_source_four_edge_local_object_mode_asserted_head(
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
                super::super::extract_source_local_mode_long_chain_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                super::super::extract_source_local_object_mode_long_chain_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_local_mode_long_chain_radix_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                super::super::extract_source_local_object_mode_long_chain_radix_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_two_edge_local_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_two_edge_local_object_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_three_edge_local_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_three_edge_local_object_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_four_edge_local_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_four_edge_local_object_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_three_edge_local_mode_three_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
            ];
            assert_eq!(prior_results.len(), 43);
            assert!(prior_results.into_iter().all(|result| result.is_none()));
        }
    }
    mod task_219_four_edge_three_hop_asserted_head {
        use super::*;

        const BASE: &str = "BaseFourEdgeModeThreeHopAssertedHead";
        const INNER: &str = "InnerFourEdgeModeThreeHopAssertedHead";
        const MIDDLE: &str = "MiddleFourEdgeModeThreeHopAssertedHead";
        const OUTER: &str = "OuterFourEdgeModeThreeHopAssertedHead";
        const TOO_DEEP: &str = "TooDeepFourEdgeModeThreeHopAssertedHead";
        const OTHER: &str = "OtherFourEdgeModeThreeHopAssertedHead";
        const DEEPER: &str = "DeeperFourEdgeModeThreeHopAssertedHead";

        fn theorem() -> IdentifierTypeAssertionTheoremSpec<'static> {
            IdentifierTypeAssertionTheoremSpec {
                status: None,
                label: "FourEdgeLocalModeThreeHopAssertedHeadPayloadBoundary",
                subject: "x",
                asserted_type: ReserveTypeShape::QualifiedSymbol(INNER),
                recovered_label: false,
                negated: false,
            }
        }

        fn exact_modes() -> Vec<ModeDefinitionSpec> {
            vec![
                mode_definition_with_label(
                    BASE,
                    "BaseFourEdgeModeThreeHopAssertedHeadDef",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition_with_label(
                    INNER,
                    "InnerFourEdgeModeThreeHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(BASE),
                ),
                mode_definition_with_label(
                    MIDDLE,
                    "MiddleFourEdgeModeThreeHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(INNER),
                ),
                mode_definition_with_label(
                    OUTER,
                    "OuterFourEdgeModeThreeHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(MIDDLE),
                ),
                mode_definition_with_label(
                    TOO_DEEP,
                    "TooDeepFourEdgeModeThreeHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(OUTER),
                ),
            ]
        }

        fn reserve() -> Vec<ReserveItemSpec> {
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol(TOO_DEEP),
            )]
        }

        #[test]
        fn active_fixture_consumes_five_real_expansions() {
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
            let plan = build_test_plan(&config).expect("Task 219 repository plan should build");
            let (ordinal, case) = active_type_elaboration_cases(&plan)
                .enumerate()
                .find(|(_, case)| {
                    case.id.0
                        == "pass_type_elaboration_four_edge_local_mode_three_hop_asserted_head_001"
                })
                .expect("Task 219 active fixture should be discoverable");
            let frontend = run_frontend(&workspace_root, case, ordinal)
                .expect("Task 219 fixture should run through the real frontend");
            assert!(frontend.diagnostics.is_empty());
            let ast = frontend
                .ast
                .expect("Task 219 fixture should produce an AST");
            let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
            assert!(resolver.detail_keys.is_empty());
            let symbols =
                augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
            assert_eq!(
                source_type_elaboration_detail_keys(&ast, resolver.module.clone(), &symbols),
                Vec::<String>::new()
            );
            let output = source_four_edge_local_mode_three_hop_asserted_head_output(
                &ast,
                resolver.module,
                &symbols,
            )
            .expect("Task 219 real AST should reach the exact three-link asserted-head seam");
            assert_source_reserved_variable_type_assertion_output(&output)
                .expect("Task 219 real AST should preserve every checked payload invariant");
            assert_eq!(output.payload.reserve.mode_expansions.len(), 5);
            assert_eq!(output.subject_binding, BindingId::new(0));
            assert_eq!(output.payload.subject_lookup_ordinal, 1);
            assert_eq!(output.term_formula.type_entries().len(), 3);
            assert_eq!(output.term_formula.normalized_types().len(), 1);
            assert_eq!(output.subject_result_input.spelling, TOO_DEEP);
            assert_eq!(output.asserted_type_input.spelling, INNER);
            assert_ne!(
                output.subject_result_input.head,
                output.asserted_type_input.head
            );
            assert_ne!(
                output.subject_result_input.site,
                output.asserted_type_input.site
            );
            assert_ne!(
                output.subject_result_input.source_range,
                output.asserted_type_input.source_range
            );
            let expansion = |spelling| {
                output
                    .payload
                    .reserve
                    .mode_expansions
                    .iter()
                    .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(spelling))
                    .map(|(_, expansion)| expansion)
                    .unwrap()
            };
            let too_deep = expansion(TOO_DEEP);
            let outer = expansion(OUTER);
            let middle = expansion(MIDDLE);
            let inner = expansion(INNER);
            let base = expansion(BASE);
            assert_eq!(too_deep.radix.spelling, OUTER);
            assert_eq!(outer.radix.spelling, MIDDLE);
            assert_eq!(middle.radix.spelling, INNER);
            assert_eq!(inner.radix.spelling, BASE);
            assert_eq!(middle.radix.head, output.asserted_type_input.head);
            assert_eq!(base.radix.spelling, "set");
            assert_eq!(base.radix.head, TypeHeadInput::BuiltinSet);
            let (_, normalized) = output
                .term_formula
                .normalized_types()
                .iter()
                .next()
                .unwrap();
            assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
            assert_eq!(normalized.source.range, base.radix.source_range);
            assert_eq!(normalized.source.spelling, "set");
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
        fn synthetic_exactness_matrix_rejects_every_near_miss() {
            let source_id = source_id(219);
            let module = ResolverModuleId::new(
                PackageId::new("test"),
                ModulePath::new("four_edge_local_mode_three_hop_asserted_head"),
            );
            let all_modes = [BASE, INNER, MIDDLE, OUTER, TOO_DEEP];
            let symbols = source_local_symbols_env(
                module.clone(),
                &[
                    (BASE, SymbolKind::Mode),
                    (INNER, SymbolKind::Mode),
                    (MIDDLE, SymbolKind::Mode),
                    (OUTER, SymbolKind::Mode),
                    (TOO_DEEP, SymbolKind::Mode),
                    (OTHER, SymbolKind::Mode),
                    (DEEPER, SymbolKind::Mode),
                ],
            );
            let exact = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                theorem(),
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
                Vec::<String>::new()
            );
            let payload = extract_source_four_edge_local_mode_three_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("the exact set-terminal four-edge three-hop source should extract");
            assert_eq!(payload.reserve.mode_expansions.len(), 5);
            assert_eq!(payload.reserve.bridge.bindings().len(), 1);
            assert_eq!(payload.subject_lookup_ordinal, 1);
            assert_eq!(payload.reserve.bridge.bindings()[0].type_spelling, TOO_DEEP);
            assert_eq!(payload.asserted_type.spelling, INNER);

            let exact_output = || {
                source_four_edge_local_mode_three_hop_asserted_head_output(
                    &exact,
                    module.clone(),
                    &symbols,
                )
                .unwrap()
            };
            let output = exact_output();
            assert_source_reserved_variable_type_assertion_output(&output)
                .expect("Task 219 exact checker output must satisfy every invariant");

            let assert_invalid_output = |invalid| {
                let invalid_result =
                    assert_source_reserved_variable_type_assertion_output(&invalid)
                        .map(|()| invalid);
                assert_eq!(
                    source_reserved_variable_type_assertion_result_detail_keys(
                        invalid_result,
                        TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_THREE_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
                    ),
                    vec![
                        TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_THREE_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY
                            .to_owned()
                    ]
                );
            };
            for removed in all_modes {
                let mut invalid = exact_output();
                invalid
                    .payload
                    .reserve
                    .mode_expansions
                    .retain(|symbol, _| source_mode_symbol_spelling(symbol) != Some(removed));
                assert_invalid_output(invalid);
            }
            let mut corruptions = Vec::new();
            let mut invalid = exact_output();
            invalid.subject_binding = BindingId::new(1);
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.payload.subject_lookup_ordinal = 2;
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.subject_result_input.spelling = OUTER.to_owned();
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.subject_result_input.head = invalid.asserted_type_input.head.clone();
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.subject_result_input.site = invalid.asserted_type_input.site.clone();
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.subject_result_input.source_range = invalid.asserted_type_input.source_range;
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.asserted_type_input.spelling = BASE.to_owned();
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.asserted_type_input.head = invalid.subject_result_input.head.clone();
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.asserted_type_input.site = invalid.subject_result_input.site.clone();
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.asserted_type_input.source_range = invalid.subject_result_input.source_range;
            corruptions.push(invalid);

            let mut invalid = exact_output();
            let middle_symbol = invalid
                .payload
                .reserve
                .mode_expansions
                .keys()
                .find(|symbol| source_mode_symbol_spelling(symbol) == Some(MIDDLE))
                .unwrap()
                .clone();
            let (_, too_deep) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(TOO_DEEP))
                .unwrap();
            too_deep.radix.spelling = MIDDLE.to_owned();
            too_deep.radix.head = TypeHeadInput::Symbol(middle_symbol);
            corruptions.push(invalid);
            let mut invalid = exact_output();
            let inner_symbol = invalid
                .payload
                .reserve
                .mode_expansions
                .keys()
                .find(|symbol| source_mode_symbol_spelling(symbol) == Some(INNER))
                .unwrap()
                .clone();
            let (_, outer) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(OUTER))
                .unwrap();
            outer.radix.spelling = INNER.to_owned();
            outer.radix.head = TypeHeadInput::Symbol(inner_symbol);
            corruptions.push(invalid);
            let mut invalid = exact_output();
            let base_symbol = invalid
                .payload
                .reserve
                .mode_expansions
                .keys()
                .find(|symbol| source_mode_symbol_spelling(symbol) == Some(BASE))
                .unwrap()
                .clone();
            let (_, middle) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(MIDDLE))
                .unwrap();
            middle.radix.spelling = BASE.to_owned();
            middle.radix.head = TypeHeadInput::Symbol(base_symbol);
            corruptions.push(invalid);
            let mut invalid = exact_output();
            let (_, inner) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(INNER))
                .unwrap();
            inner.radix.spelling = "set".to_owned();
            inner.radix.head = TypeHeadInput::BuiltinSet;
            corruptions.push(invalid);
            let mut invalid = exact_output();
            let (_, base) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(BASE))
                .unwrap();
            base.radix.spelling = "object".to_owned();
            base.radix.head = TypeHeadInput::BuiltinObject;
            corruptions.push(invalid);
            let mut invalid = exact_output();
            let (_, base) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(BASE))
                .unwrap();
            base.radix.source_range = range(source_id, 0, 1);
            corruptions.push(invalid);
            for invalid in corruptions {
                assert_invalid_output(invalid);
            }
            assert_source_reserved_variable_type_assertion_output(&exact_output())
                .expect("mutating cloned Task 219 outputs must not mutate the exact output");

            let assert_extraction_gap = |ast, context: &str| {
                assert_eq!(
                    source_type_elaboration_detail_keys(&ast, module.clone(), &symbols),
                    vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()],
                    "{context}"
                );
            };
            let ordered = exact_modes();
            let mut permutation_count = 0;
            for a in 0..5 {
                for b in 0..5 {
                    for c in 0..5 {
                        for d in 0..5 {
                            for e in 0..5 {
                                let order = [a, b, c, d, e];
                                if order
                                    .iter()
                                    .enumerate()
                                    .any(|(index, value)| order[index + 1..].contains(value))
                                {
                                    continue;
                                }
                                if order == [0, 1, 2, 3, 4] {
                                    continue;
                                }
                                permutation_count += 1;
                                assert_extraction_gap(
                                    mode_then_reserve_identifier_type_assertion_theorem_ast(
                                        source_id,
                                        vec![
                                            ordered[a], ordered[b], ordered[c], ordered[d],
                                            ordered[e],
                                        ],
                                        reserve(),
                                        theorem(),
                                    ),
                                    &format!("definition permutation {a}{b}{c}{d}{e}"),
                                );
                            }
                        }
                    }
                }
            }
            assert_eq!(permutation_count, 119);

            for index in 0..5 {
                let expected_radix = match index {
                    0 => "set",
                    1 => BASE,
                    2 => INNER,
                    3 => MIDDLE,
                    _ => OUTER,
                };
                let mut near_misses = Vec::new();
                let mut missing = exact_modes();
                missing.remove(index);
                near_misses.push(missing);
                let mut duplicate = exact_modes();
                duplicate.push(duplicate[index]);
                near_misses.push(duplicate);
                let mut wrong_label = exact_modes();
                wrong_label[index].label = Some("WrongFourEdgeModeThreeHopAssertedHeadDef");
                near_misses.push(wrong_label);
                let mut wrong_pattern = exact_modes();
                wrong_pattern[index].pattern = OTHER;
                near_misses.push(wrong_pattern);
                let mut wrong_radix = exact_modes();
                wrong_radix[index].rhs_shape = if index == 0 {
                    ReserveTypeShape::Builtin("object")
                } else {
                    ReserveTypeShape::QualifiedSymbol(OTHER)
                };
                near_misses.push(wrong_radix);
                let mut recovered = exact_modes();
                recovered[index].recovered = true;
                near_misses.push(recovered);
                let mut contextual = exact_modes();
                contextual[index].local_context = true;
                near_misses.push(contextual);
                let mut parameterized = exact_modes();
                parameterized[index].parameterized_pattern = true;
                near_misses.push(parameterized);
                let mut args = exact_modes();
                args[index].rhs_shape = ReserveTypeShape::QualifiedSymbolWithArgs(expected_radix);
                near_misses.push(args);
                let mut attributed = exact_modes();
                attributed[index].rhs_shape = if index == 0 {
                    ReserveTypeShape::AttributedSet
                } else {
                    ReserveTypeShape::AttributedQualifiedSymbol(expected_radix)
                };
                near_misses.push(attributed);
                for (variant, modes) in near_misses.into_iter().enumerate() {
                    assert_extraction_gap(
                        mode_then_reserve_identifier_type_assertion_theorem_ast(
                            source_id,
                            modes,
                            reserve(),
                            theorem(),
                        ),
                        &format!("definition {index} near miss {variant}"),
                    );
                }
            }

            for (context, index, wrong_radix) in [
                ("TooDeep-to-Outer relation link", 4, MIDDLE),
                ("Outer-to-Middle relation link", 3, INNER),
                ("Middle-to-Inner relation link", 2, BASE),
                ("Inner-to-Base terminal-normalization link", 1, MIDDLE),
            ] {
                let mut modes = exact_modes();
                modes[index].rhs_shape = ReserveTypeShape::QualifiedSymbol(wrong_radix);
                assert_extraction_gap(
                    mode_then_reserve_identifier_type_assertion_theorem_ast(
                        source_id,
                        modes,
                        reserve(),
                        theorem(),
                    ),
                    context,
                );
            }

            let mut source_near_misses = Vec::new();
            for bad_reserve in [
                ReserveTypeShape::QualifiedSymbol(BASE),
                ReserveTypeShape::QualifiedSymbol(INNER),
                ReserveTypeShape::QualifiedSymbol(MIDDLE),
                ReserveTypeShape::QualifiedSymbol(OUTER),
                ReserveTypeShape::QualifiedSymbolWithArgs(TOO_DEEP),
                ReserveTypeShape::AttributedQualifiedSymbol(TOO_DEEP),
                ReserveTypeShape::Builtin("set"),
                ReserveTypeShape::Builtin("object"),
            ] {
                source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    exact_modes(),
                    vec![reserve_item(vec!["x"], bad_reserve)],
                    theorem(),
                ));
            }
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::QualifiedSymbol(TOO_DEEP)),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                ],
                theorem(),
            ));
            for near_miss_theorem in [
                IdentifierTypeAssertionTheoremSpec {
                    label: "OtherPayloadBoundary",
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    subject: "y",
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(TOO_DEEP),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(OUTER),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(MIDDLE),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(BASE),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::Builtin("set"),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::Builtin("object"),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(OTHER),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(DEEPER),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbolWithArgs(INNER),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::AttributedQualifiedSymbol(INNER),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    negated: true,
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    status: Some("registration"),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    recovered_label: true,
                    ..theorem()
                },
            ] {
                source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    exact_modes(),
                    reserve(),
                    near_miss_theorem,
                ));
            }
            let mut connected_deeper_modes = exact_modes();
            connected_deeper_modes.push(mode_definition_with_label(
                DEEPER,
                "DeeperFourEdgeModeThreeHopAssertedHeadDef",
                ReserveTypeShape::QualifiedSymbol(TOO_DEEP),
            ));
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                connected_deeper_modes,
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(DEEPER),
                    ..theorem()
                },
            ));
            source_near_misses.push(
                modes_then_empty_definition_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    exact_modes(),
                    reserve(),
                    theorem(),
                ),
            );
            for (index, near_miss) in source_near_misses.into_iter().enumerate() {
                assert_extraction_gap(near_miss, &format!("source near miss {index}"));
            }

            let unrelated_import = source_local_and_imported_symbols_env(
                module.clone(),
                &all_modes.map(|spelling| (spelling, SymbolKind::Mode)),
                &[("UnrelatedThreeHopMode", SymbolKind::Mode)],
            );
            assert!(
                extract_source_four_edge_local_mode_three_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &unrelated_import,
                )
                .is_some()
            );
            for imported_index in 0..5 {
                let locals = all_modes
                    .iter()
                    .enumerate()
                    .filter(|(index, _)| *index != imported_index)
                    .map(|(_, spelling)| (*spelling, SymbolKind::Mode))
                    .collect::<Vec<_>>();
                for imports in [
                    vec![(all_modes[imported_index], SymbolKind::Mode)],
                    vec![
                        (all_modes[imported_index], SymbolKind::Mode),
                        (all_modes[imported_index], SymbolKind::Mode),
                    ],
                ] {
                    let provenance_near_miss =
                        source_local_and_imported_symbols_env(module.clone(), &locals, &imports);
                    assert_eq!(
                        source_type_elaboration_detail_keys(
                            &exact,
                            module.clone(),
                            &provenance_near_miss,
                        ),
                        vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
                    );
                }
            }
            for imports in [
                all_modes
                    .iter()
                    .map(|spelling| (*spelling, SymbolKind::Mode))
                    .collect::<Vec<_>>(),
                all_modes
                    .iter()
                    .flat_map(|spelling| {
                        [(*spelling, SymbolKind::Mode), (*spelling, SymbolKind::Mode)]
                    })
                    .collect::<Vec<_>>(),
            ] {
                let provenance_near_miss =
                    source_local_and_imported_symbols_env(module.clone(), &[], &imports);
                assert_eq!(
                    source_type_elaboration_detail_keys(
                        &exact,
                        module.clone(),
                        &provenance_near_miss,
                    ),
                    vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
                );
            }
        }

        #[test]
        fn route_rejects_all_44_prior_owner_fixtures_including_tasks_211_through_218() {
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
            let plan = build_test_plan(&config).expect("Task 219 repository plan should build");
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
            ];
            assert_eq!(prior_owner_ids.len(), 44);
            assert_eq!(
                &prior_owner_ids[36..],
                &[
                    "pass_type_elaboration_two_edge_local_mode_two_hop_asserted_head_001",
                    "pass_type_elaboration_two_edge_local_object_mode_two_hop_asserted_head_001",
                    "pass_type_elaboration_three_edge_local_mode_two_hop_asserted_head_001",
                    "pass_type_elaboration_three_edge_local_object_mode_two_hop_asserted_head_001",
                    "pass_type_elaboration_four_edge_local_mode_two_hop_asserted_head_001",
                    "pass_type_elaboration_four_edge_local_object_mode_two_hop_asserted_head_001",
                    "pass_type_elaboration_three_edge_local_mode_three_hop_asserted_head_001",
                    "pass_type_elaboration_three_edge_local_object_mode_three_hop_asserted_head_001",
                ]
            );
            for owner_id in prior_owner_ids {
                let (ordinal, case) = active_type_elaboration_cases(&plan)
                    .enumerate()
                    .find(|(_, case)| case.id.0 == owner_id)
                    .unwrap_or_else(|| panic!("prior owner fixture {owner_id} must be active"));
                let frontend = run_frontend(&workspace_root, case, ordinal)
                    .unwrap_or_else(|error| panic!("owner fixture {owner_id} must parse: {error}"));
                assert!(frontend.diagnostics.is_empty());
                let ast = frontend
                    .ast
                    .unwrap_or_else(|| panic!("owner fixture {owner_id} must produce an AST"));
                let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
                assert!(resolver.detail_keys.is_empty());
                let symbols =
                    augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
                assert_eq!(
                    source_type_elaboration_detail_keys(&ast, resolver.module.clone(), &symbols),
                    Vec::<String>::new(),
                    "prior owner fixture {owner_id} must still reach its active route"
                );
                assert!(
                    extract_source_four_edge_local_mode_three_hop_asserted_head(
                        &ast,
                        resolver.module,
                        &symbols,
                    )
                    .is_none(),
                    "Task 219 must reject prior owner fixture {owner_id}"
                );
            }
        }

        #[test]
        fn task219_synthetic_is_rejected_by_all_44_prior_type_assertion_extractors() {
            let source_id = source_id(219);
            let module = ResolverModuleId::new(
                PackageId::new("test"),
                ModulePath::new("task219_prior_extractor_isolation"),
            );
            let symbols = source_local_symbols_env(
                module.clone(),
                &[
                    (BASE, SymbolKind::Mode),
                    (INNER, SymbolKind::Mode),
                    (MIDDLE, SymbolKind::Mode),
                    (OUTER, SymbolKind::Mode),
                    (TOO_DEEP, SymbolKind::Mode),
                ],
            );
            let exact = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                theorem(),
            );
            assert!(
                extract_source_four_edge_local_mode_three_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                )
                .is_some()
            );
            let prior_results = [
                extract_source_reserved_variable_type_assertion(&exact, module.clone(), &symbols),
                super::super::extract_source_reserved_object_variable_type_assertion(
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
                extract_source_chained_local_mode_radix_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
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
                extract_source_three_edge_local_mode_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
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
                super::super::extract_source_four_edge_local_mode_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                super::super::extract_source_four_edge_local_object_mode_asserted_head(
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
                super::super::extract_source_local_mode_long_chain_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                super::super::extract_source_local_object_mode_long_chain_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_local_mode_long_chain_radix_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                super::super::extract_source_local_object_mode_long_chain_radix_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_two_edge_local_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_two_edge_local_object_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_three_edge_local_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_three_edge_local_object_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_four_edge_local_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_four_edge_local_object_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_three_edge_local_mode_three_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_three_edge_local_object_mode_three_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
            ];
            assert_eq!(prior_results.len(), 44);
            assert!(prior_results.into_iter().all(|result| result.is_none()));
        }
    }
    mod task_220_four_edge_object_three_hop_asserted_head {
        use super::*;

        const BASE: &str = "BaseFourEdgeObjectModeThreeHopAssertedHead";
        const INNER: &str = "InnerFourEdgeObjectModeThreeHopAssertedHead";
        const MIDDLE: &str = "MiddleFourEdgeObjectModeThreeHopAssertedHead";
        const OUTER: &str = "OuterFourEdgeObjectModeThreeHopAssertedHead";
        const TOO_DEEP: &str = "TooDeepFourEdgeObjectModeThreeHopAssertedHead";
        const OTHER: &str = "OtherFourEdgeObjectModeThreeHopAssertedHead";
        const DEEPER: &str = "DeeperFourEdgeObjectModeThreeHopAssertedHead";

        fn theorem() -> IdentifierTypeAssertionTheoremSpec<'static> {
            IdentifierTypeAssertionTheoremSpec {
                status: None,
                label: "FourEdgeLocalObjectModeThreeHopAssertedHeadPayloadBoundary",
                subject: "x",
                asserted_type: ReserveTypeShape::QualifiedSymbol(INNER),
                recovered_label: false,
                negated: false,
            }
        }

        fn exact_modes() -> Vec<ModeDefinitionSpec> {
            vec![
                mode_definition_with_label(
                    BASE,
                    "BaseFourEdgeObjectModeThreeHopAssertedHeadDef",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition_with_label(
                    INNER,
                    "InnerFourEdgeObjectModeThreeHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(BASE),
                ),
                mode_definition_with_label(
                    MIDDLE,
                    "MiddleFourEdgeObjectModeThreeHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(INNER),
                ),
                mode_definition_with_label(
                    OUTER,
                    "OuterFourEdgeObjectModeThreeHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(MIDDLE),
                ),
                mode_definition_with_label(
                    TOO_DEEP,
                    "TooDeepFourEdgeObjectModeThreeHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(OUTER),
                ),
            ]
        }

        fn reserve() -> Vec<ReserveItemSpec> {
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol(TOO_DEEP),
            )]
        }

        #[test]
        fn active_fixture_consumes_five_real_object_expansions() {
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
            let plan = build_test_plan(&config).expect("Task 220 repository plan should build");
            let (ordinal, case) = active_type_elaboration_cases(&plan)
                .enumerate()
                .find(|(_, case)| {
                    case.id.0
                        == "pass_type_elaboration_four_edge_local_object_mode_three_hop_asserted_head_001"
                })
                .expect("Task 220 active fixture should be discoverable");
            let frontend = run_frontend(&workspace_root, case, ordinal)
                .expect("Task 220 fixture should run through the real frontend");
            assert!(frontend.diagnostics.is_empty());
            let ast = frontend
                .ast
                .expect("Task 220 fixture should produce an AST");
            let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
            assert!(resolver.detail_keys.is_empty());
            let symbols =
                augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
            assert_eq!(
                source_type_elaboration_detail_keys(&ast, resolver.module.clone(), &symbols),
                Vec::<String>::new()
            );
            let output = source_four_edge_local_object_mode_three_hop_asserted_head_output(
                &ast,
                resolver.module,
                &symbols,
            )
            .expect(
                "Task 220 real AST should reach the exact object three-link asserted-head seam",
            );
            assert_source_reserved_variable_type_assertion_output(&output)
                .expect("Task 220 real AST should preserve every checked payload invariant");
            assert_eq!(output.payload.reserve.mode_expansions.len(), 5);
            assert_eq!(output.subject_binding, BindingId::new(0));
            assert_eq!(output.payload.subject_lookup_ordinal, 1);
            assert_eq!(output.term_formula.type_entries().len(), 3);
            assert_eq!(output.term_formula.normalized_types().len(), 1);
            assert_eq!(output.subject_result_input.spelling, TOO_DEEP);
            assert_eq!(output.asserted_type_input.spelling, INNER);
            assert_ne!(
                output.subject_result_input.head,
                output.asserted_type_input.head
            );
            assert_ne!(
                output.subject_result_input.site,
                output.asserted_type_input.site
            );
            assert_ne!(
                output.subject_result_input.source_range,
                output.asserted_type_input.source_range
            );
            let expansion = |spelling| {
                output
                    .payload
                    .reserve
                    .mode_expansions
                    .iter()
                    .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(spelling))
                    .map(|(_, expansion)| expansion)
                    .unwrap()
            };
            let too_deep = expansion(TOO_DEEP);
            let outer = expansion(OUTER);
            let middle = expansion(MIDDLE);
            let inner = expansion(INNER);
            let base = expansion(BASE);
            assert_eq!(too_deep.radix.spelling, OUTER);
            assert_eq!(outer.radix.spelling, MIDDLE);
            assert_eq!(middle.radix.spelling, INNER);
            assert_eq!(inner.radix.spelling, BASE);
            assert_eq!(middle.radix.head, output.asserted_type_input.head);
            assert_eq!(base.radix.spelling, "object");
            assert_eq!(base.radix.head, TypeHeadInput::BuiltinObject);
            let (_, normalized) = output
                .term_formula
                .normalized_types()
                .iter()
                .next()
                .unwrap();
            assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
            assert_eq!(normalized.source.range, base.radix.source_range);
            assert_eq!(normalized.source.spelling, "object");
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
        fn synthetic_exactness_matrix_rejects_every_near_miss() {
            let source_id = source_id(220);
            let module = ResolverModuleId::new(
                PackageId::new("test"),
                ModulePath::new("four_edge_local_object_mode_three_hop_asserted_head"),
            );
            let all_modes = [BASE, INNER, MIDDLE, OUTER, TOO_DEEP];
            let symbols = source_local_symbols_env(
                module.clone(),
                &[
                    (BASE, SymbolKind::Mode),
                    (INNER, SymbolKind::Mode),
                    (MIDDLE, SymbolKind::Mode),
                    (OUTER, SymbolKind::Mode),
                    (TOO_DEEP, SymbolKind::Mode),
                    (OTHER, SymbolKind::Mode),
                    (DEEPER, SymbolKind::Mode),
                ],
            );
            let exact = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                theorem(),
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
                Vec::<String>::new()
            );
            let payload = extract_source_four_edge_local_object_mode_three_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("the exact object-terminal four-edge three-hop source should extract");
            assert_eq!(payload.reserve.mode_expansions.len(), 5);
            assert_eq!(payload.reserve.bridge.bindings().len(), 1);
            assert_eq!(payload.subject_lookup_ordinal, 1);
            assert_eq!(payload.reserve.bridge.bindings()[0].type_spelling, TOO_DEEP);
            assert_eq!(payload.asserted_type.spelling, INNER);

            let exact_output = || {
                source_four_edge_local_object_mode_three_hop_asserted_head_output(
                    &exact,
                    module.clone(),
                    &symbols,
                )
                .unwrap()
            };
            let output = exact_output();
            assert_source_reserved_variable_type_assertion_output(&output)
                .expect("Task 220 exact checker output must satisfy every invariant");

            let assert_invalid_output = |invalid| {
                let invalid_result =
                    assert_source_reserved_variable_type_assertion_output(&invalid)
                        .map(|()| invalid);
                assert_eq!(
                    source_reserved_variable_type_assertion_result_detail_keys(
                        invalid_result,
                        TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_THREE_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
                    ),
                    vec![
                        TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_THREE_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY
                            .to_owned()
                    ]
                );
            };
            for removed in all_modes {
                let mut invalid = exact_output();
                invalid
                    .payload
                    .reserve
                    .mode_expansions
                    .retain(|symbol, _| source_mode_symbol_spelling(symbol) != Some(removed));
                assert_invalid_output(invalid);
            }
            let mut corruptions = Vec::new();
            let mut invalid = exact_output();
            invalid.subject_binding = BindingId::new(1);
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.payload.subject_lookup_ordinal = 2;
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.subject_result_input.spelling = OUTER.to_owned();
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.subject_result_input.head = invalid.asserted_type_input.head.clone();
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.subject_result_input.site = invalid.asserted_type_input.site.clone();
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.subject_result_input.source_range = invalid.asserted_type_input.source_range;
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.asserted_type_input.spelling = BASE.to_owned();
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.asserted_type_input.head = invalid.subject_result_input.head.clone();
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.asserted_type_input.site = invalid.subject_result_input.site.clone();
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.asserted_type_input.source_range = invalid.subject_result_input.source_range;
            corruptions.push(invalid);

            let mut invalid = exact_output();
            let middle_symbol = invalid
                .payload
                .reserve
                .mode_expansions
                .keys()
                .find(|symbol| source_mode_symbol_spelling(symbol) == Some(MIDDLE))
                .unwrap()
                .clone();
            let (_, too_deep) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(TOO_DEEP))
                .unwrap();
            too_deep.radix.spelling = MIDDLE.to_owned();
            too_deep.radix.head = TypeHeadInput::Symbol(middle_symbol);
            corruptions.push(invalid);
            let mut invalid = exact_output();
            let inner_symbol = invalid
                .payload
                .reserve
                .mode_expansions
                .keys()
                .find(|symbol| source_mode_symbol_spelling(symbol) == Some(INNER))
                .unwrap()
                .clone();
            let (_, outer) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(OUTER))
                .unwrap();
            outer.radix.spelling = INNER.to_owned();
            outer.radix.head = TypeHeadInput::Symbol(inner_symbol);
            corruptions.push(invalid);
            let mut invalid = exact_output();
            let base_symbol = invalid
                .payload
                .reserve
                .mode_expansions
                .keys()
                .find(|symbol| source_mode_symbol_spelling(symbol) == Some(BASE))
                .unwrap()
                .clone();
            let (_, middle) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(MIDDLE))
                .unwrap();
            middle.radix.spelling = BASE.to_owned();
            middle.radix.head = TypeHeadInput::Symbol(base_symbol);
            corruptions.push(invalid);
            let mut invalid = exact_output();
            let (_, inner) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(INNER))
                .unwrap();
            inner.radix.spelling = "set".to_owned();
            inner.radix.head = TypeHeadInput::BuiltinSet;
            corruptions.push(invalid);
            let mut invalid = exact_output();
            let (_, base) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(BASE))
                .unwrap();
            base.radix.spelling = "set".to_owned();
            base.radix.head = TypeHeadInput::BuiltinSet;
            corruptions.push(invalid);
            let mut invalid = exact_output();
            let (_, base) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(BASE))
                .unwrap();
            base.radix.source_range = range(source_id, 0, 1);
            corruptions.push(invalid);
            for invalid in corruptions {
                assert_invalid_output(invalid);
            }
            assert_source_reserved_variable_type_assertion_output(&exact_output())
                .expect("mutating cloned Task 220 outputs must not mutate the exact output");

            let assert_extraction_gap = |ast, context: &str| {
                assert_eq!(
                    source_type_elaboration_detail_keys(&ast, module.clone(), &symbols),
                    vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()],
                    "{context}"
                );
            };
            let ordered = exact_modes();
            let mut permutation_count = 0;
            for a in 0..5 {
                for b in 0..5 {
                    for c in 0..5 {
                        for d in 0..5 {
                            for e in 0..5 {
                                let order = [a, b, c, d, e];
                                if order
                                    .iter()
                                    .enumerate()
                                    .any(|(index, value)| order[index + 1..].contains(value))
                                {
                                    continue;
                                }
                                if order == [0, 1, 2, 3, 4] {
                                    continue;
                                }
                                permutation_count += 1;
                                assert_extraction_gap(
                                    mode_then_reserve_identifier_type_assertion_theorem_ast(
                                        source_id,
                                        vec![
                                            ordered[a], ordered[b], ordered[c], ordered[d],
                                            ordered[e],
                                        ],
                                        reserve(),
                                        theorem(),
                                    ),
                                    &format!("definition permutation {a}{b}{c}{d}{e}"),
                                );
                            }
                        }
                    }
                }
            }
            assert_eq!(permutation_count, 119);

            for index in 0..5 {
                let expected_radix = match index {
                    0 => "object",
                    1 => BASE,
                    2 => INNER,
                    3 => MIDDLE,
                    _ => OUTER,
                };
                let mut near_misses = Vec::new();
                let mut missing = exact_modes();
                missing.remove(index);
                near_misses.push(missing);
                let mut duplicate = exact_modes();
                duplicate.push(duplicate[index]);
                near_misses.push(duplicate);
                let mut wrong_label = exact_modes();
                wrong_label[index].label = Some("WrongFourEdgeObjectModeThreeHopAssertedHeadDef");
                near_misses.push(wrong_label);
                let mut wrong_pattern = exact_modes();
                wrong_pattern[index].pattern = OTHER;
                near_misses.push(wrong_pattern);
                let mut wrong_radix = exact_modes();
                wrong_radix[index].rhs_shape = if index == 0 {
                    ReserveTypeShape::Builtin("set")
                } else {
                    ReserveTypeShape::QualifiedSymbol(OTHER)
                };
                near_misses.push(wrong_radix);
                let mut recovered = exact_modes();
                recovered[index].recovered = true;
                near_misses.push(recovered);
                let mut contextual = exact_modes();
                contextual[index].local_context = true;
                near_misses.push(contextual);
                let mut parameterized = exact_modes();
                parameterized[index].parameterized_pattern = true;
                near_misses.push(parameterized);
                let mut args = exact_modes();
                args[index].rhs_shape = ReserveTypeShape::QualifiedSymbolWithArgs(expected_radix);
                near_misses.push(args);
                let mut attributed = exact_modes();
                attributed[index].rhs_shape = if index == 0 {
                    ReserveTypeShape::AttributedObject
                } else {
                    ReserveTypeShape::AttributedQualifiedSymbol(expected_radix)
                };
                near_misses.push(attributed);
                for (variant, modes) in near_misses.into_iter().enumerate() {
                    assert_extraction_gap(
                        mode_then_reserve_identifier_type_assertion_theorem_ast(
                            source_id,
                            modes,
                            reserve(),
                            theorem(),
                        ),
                        &format!("definition {index} near miss {variant}"),
                    );
                }
            }

            for (context, index, wrong_radix) in [
                ("TooDeep-to-Outer relation link", 4, MIDDLE),
                ("Outer-to-Middle relation link", 3, INNER),
                ("Middle-to-Inner relation link", 2, BASE),
                ("Inner-to-Base terminal-normalization link", 1, MIDDLE),
            ] {
                let mut modes = exact_modes();
                modes[index].rhs_shape = ReserveTypeShape::QualifiedSymbol(wrong_radix);
                assert_extraction_gap(
                    mode_then_reserve_identifier_type_assertion_theorem_ast(
                        source_id,
                        modes,
                        reserve(),
                        theorem(),
                    ),
                    context,
                );
            }

            let mut source_near_misses = Vec::new();
            for bad_reserve in [
                ReserveTypeShape::QualifiedSymbol(BASE),
                ReserveTypeShape::QualifiedSymbol(INNER),
                ReserveTypeShape::QualifiedSymbol(MIDDLE),
                ReserveTypeShape::QualifiedSymbol(OUTER),
                ReserveTypeShape::QualifiedSymbolWithArgs(TOO_DEEP),
                ReserveTypeShape::AttributedQualifiedSymbol(TOO_DEEP),
                ReserveTypeShape::Builtin("set"),
                ReserveTypeShape::Builtin("object"),
            ] {
                source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    exact_modes(),
                    vec![reserve_item(vec!["x"], bad_reserve)],
                    theorem(),
                ));
            }
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::QualifiedSymbol(TOO_DEEP)),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("object")),
                ],
                theorem(),
            ));
            for near_miss_theorem in [
                IdentifierTypeAssertionTheoremSpec {
                    label: "OtherPayloadBoundary",
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    subject: "y",
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(TOO_DEEP),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(OUTER),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(MIDDLE),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(BASE),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::Builtin("set"),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::Builtin("object"),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(OTHER),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(DEEPER),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbolWithArgs(INNER),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::AttributedQualifiedSymbol(INNER),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    negated: true,
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    status: Some("registration"),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    recovered_label: true,
                    ..theorem()
                },
            ] {
                source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    exact_modes(),
                    reserve(),
                    near_miss_theorem,
                ));
            }
            let mut connected_deeper_modes = exact_modes();
            connected_deeper_modes.push(mode_definition_with_label(
                DEEPER,
                "DeeperFourEdgeObjectModeThreeHopAssertedHeadDef",
                ReserveTypeShape::QualifiedSymbol(TOO_DEEP),
            ));
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                connected_deeper_modes,
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(DEEPER),
                    ..theorem()
                },
            ));
            source_near_misses.push(
                modes_then_empty_definition_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    exact_modes(),
                    reserve(),
                    theorem(),
                ),
            );
            for (index, near_miss) in source_near_misses.into_iter().enumerate() {
                assert_extraction_gap(near_miss, &format!("source near miss {index}"));
            }

            let unrelated_import = source_local_and_imported_symbols_env(
                module.clone(),
                &all_modes.map(|spelling| (spelling, SymbolKind::Mode)),
                &[("UnrelatedThreeHopMode", SymbolKind::Mode)],
            );
            assert!(
                extract_source_four_edge_local_object_mode_three_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &unrelated_import,
                )
                .is_some()
            );
            for imported_index in 0..5 {
                let locals = all_modes
                    .iter()
                    .enumerate()
                    .filter(|(index, _)| *index != imported_index)
                    .map(|(_, spelling)| (*spelling, SymbolKind::Mode))
                    .collect::<Vec<_>>();
                for imports in [
                    vec![(all_modes[imported_index], SymbolKind::Mode)],
                    vec![
                        (all_modes[imported_index], SymbolKind::Mode),
                        (all_modes[imported_index], SymbolKind::Mode),
                    ],
                ] {
                    let provenance_near_miss =
                        source_local_and_imported_symbols_env(module.clone(), &locals, &imports);
                    assert_eq!(
                        source_type_elaboration_detail_keys(
                            &exact,
                            module.clone(),
                            &provenance_near_miss,
                        ),
                        vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
                    );
                }
            }
            for imports in [
                all_modes
                    .iter()
                    .map(|spelling| (*spelling, SymbolKind::Mode))
                    .collect::<Vec<_>>(),
                all_modes
                    .iter()
                    .flat_map(|spelling| {
                        [(*spelling, SymbolKind::Mode), (*spelling, SymbolKind::Mode)]
                    })
                    .collect::<Vec<_>>(),
            ] {
                let provenance_near_miss =
                    source_local_and_imported_symbols_env(module.clone(), &[], &imports);
                assert_eq!(
                    source_type_elaboration_detail_keys(
                        &exact,
                        module.clone(),
                        &provenance_near_miss,
                    ),
                    vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
                );
            }
        }

        #[test]
        fn route_rejects_all_45_prior_owner_fixtures_including_task_208_and_tasks_211_through_219()
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
            let plan = build_test_plan(&config).expect("Task 220 repository plan should build");
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
            ];
            assert_eq!(prior_owner_ids.len(), 45);
            assert_eq!(
                prior_owner_ids[29],
                "pass_type_elaboration_four_edge_local_object_mode_radix_asserted_head_001"
            );
            assert_eq!(
                &prior_owner_ids[36..],
                &[
                    "pass_type_elaboration_two_edge_local_mode_two_hop_asserted_head_001",
                    "pass_type_elaboration_two_edge_local_object_mode_two_hop_asserted_head_001",
                    "pass_type_elaboration_three_edge_local_mode_two_hop_asserted_head_001",
                    "pass_type_elaboration_three_edge_local_object_mode_two_hop_asserted_head_001",
                    "pass_type_elaboration_four_edge_local_mode_two_hop_asserted_head_001",
                    "pass_type_elaboration_four_edge_local_object_mode_two_hop_asserted_head_001",
                    "pass_type_elaboration_three_edge_local_mode_three_hop_asserted_head_001",
                    "pass_type_elaboration_three_edge_local_object_mode_three_hop_asserted_head_001",
                    "pass_type_elaboration_four_edge_local_mode_three_hop_asserted_head_001",
                ]
            );
            for owner_id in prior_owner_ids {
                let (ordinal, case) = active_type_elaboration_cases(&plan)
                    .enumerate()
                    .find(|(_, case)| case.id.0 == owner_id)
                    .unwrap_or_else(|| panic!("prior owner fixture {owner_id} must be active"));
                let frontend = run_frontend(&workspace_root, case, ordinal)
                    .unwrap_or_else(|error| panic!("owner fixture {owner_id} must parse: {error}"));
                assert!(frontend.diagnostics.is_empty());
                let ast = frontend
                    .ast
                    .unwrap_or_else(|| panic!("owner fixture {owner_id} must produce an AST"));
                let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
                assert!(resolver.detail_keys.is_empty());
                let symbols =
                    augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
                assert_eq!(
                    source_type_elaboration_detail_keys(&ast, resolver.module.clone(), &symbols),
                    Vec::<String>::new(),
                    "prior owner fixture {owner_id} must still reach its active route"
                );
                assert!(
                    extract_source_four_edge_local_object_mode_three_hop_asserted_head(
                        &ast,
                        resolver.module,
                        &symbols,
                    )
                    .is_none(),
                    "Task 220 must reject prior owner fixture {owner_id}"
                );
            }
        }

        #[test]
        fn task220_synthetic_is_rejected_by_all_45_prior_type_assertion_extractors() {
            let source_id = source_id(220);
            let module = ResolverModuleId::new(
                PackageId::new("test"),
                ModulePath::new("task220_prior_extractor_isolation"),
            );
            let symbols = source_local_symbols_env(
                module.clone(),
                &[
                    (BASE, SymbolKind::Mode),
                    (INNER, SymbolKind::Mode),
                    (MIDDLE, SymbolKind::Mode),
                    (OUTER, SymbolKind::Mode),
                    (TOO_DEEP, SymbolKind::Mode),
                ],
            );
            let exact = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                theorem(),
            );
            assert!(
                extract_source_four_edge_local_object_mode_three_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                )
                .is_some()
            );
            let prior_results = [
                extract_source_reserved_variable_type_assertion(&exact, module.clone(), &symbols),
                super::super::extract_source_reserved_object_variable_type_assertion(
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
                extract_source_chained_local_mode_radix_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
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
                extract_source_three_edge_local_mode_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
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
                super::super::extract_source_four_edge_local_mode_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                super::super::extract_source_four_edge_local_object_mode_asserted_head(
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
                super::super::extract_source_local_mode_long_chain_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                super::super::extract_source_local_object_mode_long_chain_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_local_mode_long_chain_radix_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                super::super::extract_source_local_object_mode_long_chain_radix_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_two_edge_local_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_two_edge_local_object_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_three_edge_local_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_three_edge_local_object_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_four_edge_local_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_four_edge_local_object_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_three_edge_local_mode_three_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_three_edge_local_object_mode_three_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_four_edge_local_mode_three_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
            ];
            assert_eq!(prior_results.len(), 45);
            assert!(prior_results.into_iter().all(|result| result.is_none()));
        }
    }
    mod task_221_four_edge_four_hop_asserted_head {
        use super::*;

        const BASE: &str = "BaseFourEdgeModeFourHopAssertedHead";
        const INNER: &str = "InnerFourEdgeModeFourHopAssertedHead";
        const MIDDLE: &str = "MiddleFourEdgeModeFourHopAssertedHead";
        const OUTER: &str = "OuterFourEdgeModeFourHopAssertedHead";
        const TOO_DEEP: &str = "TooDeepFourEdgeModeFourHopAssertedHead";
        const OTHER: &str = "OtherFourEdgeModeFourHopAssertedHead";
        const DEEPER: &str = "DeeperFourEdgeModeFourHopAssertedHead";

        fn theorem() -> IdentifierTypeAssertionTheoremSpec<'static> {
            IdentifierTypeAssertionTheoremSpec {
                status: None,
                label: "FourEdgeLocalModeFourHopAssertedHeadPayloadBoundary",
                subject: "x",
                asserted_type: ReserveTypeShape::QualifiedSymbol(BASE),
                recovered_label: false,
                negated: false,
            }
        }

        fn exact_modes() -> Vec<ModeDefinitionSpec> {
            vec![
                mode_definition_with_label(
                    BASE,
                    "BaseFourEdgeModeFourHopAssertedHeadDef",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition_with_label(
                    INNER,
                    "InnerFourEdgeModeFourHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(BASE),
                ),
                mode_definition_with_label(
                    MIDDLE,
                    "MiddleFourEdgeModeFourHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(INNER),
                ),
                mode_definition_with_label(
                    OUTER,
                    "OuterFourEdgeModeFourHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(MIDDLE),
                ),
                mode_definition_with_label(
                    TOO_DEEP,
                    "TooDeepFourEdgeModeFourHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(OUTER),
                ),
            ]
        }

        fn reserve() -> Vec<ReserveItemSpec> {
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol(TOO_DEEP),
            )]
        }

        #[test]
        fn active_fixture_consumes_five_real_expansions() {
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
            let plan = build_test_plan(&config).expect("Task 221 repository plan should build");
            let (ordinal, case) = active_type_elaboration_cases(&plan)
                .enumerate()
                .find(|(_, case)| {
                    case.id.0
                        == "pass_type_elaboration_four_edge_local_mode_four_hop_asserted_head_001"
                })
                .expect("Task 221 active fixture should be discoverable");
            let frontend = run_frontend(&workspace_root, case, ordinal)
                .expect("Task 221 fixture should run through the real frontend");
            assert!(frontend.diagnostics.is_empty());
            let ast = frontend
                .ast
                .expect("Task 221 fixture should produce an AST");
            let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
            assert!(resolver.detail_keys.is_empty());
            let symbols =
                augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
            assert_eq!(
                source_type_elaboration_detail_keys(&ast, resolver.module.clone(), &symbols),
                Vec::<String>::new()
            );
            let output = source_four_edge_local_mode_four_hop_asserted_head_output(
                &ast,
                resolver.module,
                &symbols,
            )
            .expect("Task 221 real AST should reach the exact four-link asserted-head seam");
            assert_source_reserved_variable_type_assertion_output(&output)
                .expect("Task 221 real AST should preserve every checked payload invariant");
            assert_eq!(output.payload.reserve.mode_expansions.len(), 5);
            assert_eq!(output.subject_binding, BindingId::new(0));
            assert_eq!(output.payload.subject_lookup_ordinal, 1);
            assert_eq!(output.term_formula.type_entries().len(), 3);
            assert_eq!(output.term_formula.normalized_types().len(), 1);
            assert_eq!(output.subject_result_input.spelling, TOO_DEEP);
            assert_eq!(output.asserted_type_input.spelling, BASE);
            assert_ne!(
                output.subject_result_input.head,
                output.asserted_type_input.head
            );
            assert_ne!(
                output.subject_result_input.site,
                output.asserted_type_input.site
            );
            assert_ne!(
                output.subject_result_input.source_range,
                output.asserted_type_input.source_range
            );
            let expansion = |spelling| {
                output
                    .payload
                    .reserve
                    .mode_expansions
                    .iter()
                    .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(spelling))
                    .map(|(_, expansion)| expansion)
                    .unwrap()
            };
            let too_deep = expansion(TOO_DEEP);
            let outer = expansion(OUTER);
            let middle = expansion(MIDDLE);
            let inner = expansion(INNER);
            let base = expansion(BASE);
            assert_eq!(too_deep.radix.spelling, OUTER);
            assert_eq!(outer.radix.spelling, MIDDLE);
            assert_eq!(middle.radix.spelling, INNER);
            assert_eq!(inner.radix.spelling, BASE);
            assert_eq!(inner.radix.head, output.asserted_type_input.head);
            assert_eq!(base.radix.spelling, "set");
            assert_eq!(base.radix.head, TypeHeadInput::BuiltinSet);
            let (_, normalized) = output
                .term_formula
                .normalized_types()
                .iter()
                .next()
                .unwrap();
            assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
            assert_eq!(normalized.source.range, base.radix.source_range);
            assert_eq!(normalized.source.spelling, "set");
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
        fn synthetic_exactness_matrix_rejects_every_near_miss() {
            let source_id = source_id(221);
            let module = ResolverModuleId::new(
                PackageId::new("test"),
                ModulePath::new("four_edge_local_mode_four_hop_asserted_head"),
            );
            let all_modes = [BASE, INNER, MIDDLE, OUTER, TOO_DEEP];
            let symbols = source_local_symbols_env(
                module.clone(),
                &[
                    (BASE, SymbolKind::Mode),
                    (INNER, SymbolKind::Mode),
                    (MIDDLE, SymbolKind::Mode),
                    (OUTER, SymbolKind::Mode),
                    (TOO_DEEP, SymbolKind::Mode),
                    (OTHER, SymbolKind::Mode),
                    (DEEPER, SymbolKind::Mode),
                ],
            );
            let exact = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                theorem(),
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
                Vec::<String>::new()
            );
            let payload = extract_source_four_edge_local_mode_four_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("the exact set-terminal four-edge four-hop source should extract");
            assert_eq!(payload.reserve.mode_expansions.len(), 5);
            assert_eq!(payload.reserve.bridge.bindings().len(), 1);
            assert_eq!(payload.subject_lookup_ordinal, 1);
            assert_eq!(payload.reserve.bridge.bindings()[0].type_spelling, TOO_DEEP);
            assert_eq!(payload.asserted_type.spelling, BASE);

            let exact_output = || {
                source_four_edge_local_mode_four_hop_asserted_head_output(
                    &exact,
                    module.clone(),
                    &symbols,
                )
                .unwrap()
            };
            let output = exact_output();
            assert_source_reserved_variable_type_assertion_output(&output)
                .expect("Task 221 exact checker output must satisfy every invariant");

            let assert_invalid_output = |invalid| {
                let invalid_result =
                    assert_source_reserved_variable_type_assertion_output(&invalid)
                        .map(|()| invalid);
                assert_eq!(
                    source_reserved_variable_type_assertion_result_detail_keys(
                        invalid_result,
                        TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_FOUR_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
                    ),
                    vec![
                        TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_FOUR_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY
                            .to_owned()
                    ]
                );
            };
            for removed in all_modes {
                let mut invalid = exact_output();
                invalid
                    .payload
                    .reserve
                    .mode_expansions
                    .retain(|symbol, _| source_mode_symbol_spelling(symbol) != Some(removed));
                assert_invalid_output(invalid);
            }
            let mut corruptions = Vec::new();
            let mut invalid = exact_output();
            invalid.subject_binding = BindingId::new(1);
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.payload.subject_lookup_ordinal = 2;
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.subject_result_input.spelling = OUTER.to_owned();
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.subject_result_input.head = invalid.asserted_type_input.head.clone();
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.subject_result_input.site = invalid.asserted_type_input.site.clone();
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.subject_result_input.source_range = invalid.asserted_type_input.source_range;
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.asserted_type_input.spelling = INNER.to_owned();
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.asserted_type_input.head = invalid.subject_result_input.head.clone();
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.asserted_type_input.site = invalid.subject_result_input.site.clone();
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.asserted_type_input.source_range = invalid.subject_result_input.source_range;
            corruptions.push(invalid);

            let mut invalid = exact_output();
            let middle_symbol = invalid
                .payload
                .reserve
                .mode_expansions
                .keys()
                .find(|symbol| source_mode_symbol_spelling(symbol) == Some(MIDDLE))
                .unwrap()
                .clone();
            let (_, too_deep) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(TOO_DEEP))
                .unwrap();
            too_deep.radix.spelling = MIDDLE.to_owned();
            too_deep.radix.head = TypeHeadInput::Symbol(middle_symbol);
            corruptions.push(invalid);
            let mut invalid = exact_output();
            let inner_symbol = invalid
                .payload
                .reserve
                .mode_expansions
                .keys()
                .find(|symbol| source_mode_symbol_spelling(symbol) == Some(INNER))
                .unwrap()
                .clone();
            let (_, outer) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(OUTER))
                .unwrap();
            outer.radix.spelling = INNER.to_owned();
            outer.radix.head = TypeHeadInput::Symbol(inner_symbol);
            corruptions.push(invalid);
            let mut invalid = exact_output();
            let base_symbol = invalid
                .payload
                .reserve
                .mode_expansions
                .keys()
                .find(|symbol| source_mode_symbol_spelling(symbol) == Some(BASE))
                .unwrap()
                .clone();
            let (_, middle) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(MIDDLE))
                .unwrap();
            middle.radix.spelling = BASE.to_owned();
            middle.radix.head = TypeHeadInput::Symbol(base_symbol);
            corruptions.push(invalid);
            let mut invalid = exact_output();
            let (_, inner) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(INNER))
                .unwrap();
            inner.radix.spelling = "set".to_owned();
            inner.radix.head = TypeHeadInput::BuiltinSet;
            corruptions.push(invalid);
            let mut invalid = exact_output();
            let (_, base) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(BASE))
                .unwrap();
            base.radix.spelling = "object".to_owned();
            base.radix.head = TypeHeadInput::BuiltinObject;
            corruptions.push(invalid);
            let mut invalid = exact_output();
            let (_, base) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(BASE))
                .unwrap();
            base.radix.source_range = range(source_id, 0, 1);
            corruptions.push(invalid);
            for invalid in corruptions {
                assert_invalid_output(invalid);
            }
            assert_source_reserved_variable_type_assertion_output(&exact_output())
                .expect("mutating cloned Task 221 outputs must not mutate the exact output");

            let assert_extraction_gap = |ast, context: &str| {
                assert_eq!(
                    source_type_elaboration_detail_keys(&ast, module.clone(), &symbols),
                    vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()],
                    "{context}"
                );
            };
            let ordered = exact_modes();
            let mut permutation_count = 0;
            for a in 0..5 {
                for b in 0..5 {
                    for c in 0..5 {
                        for d in 0..5 {
                            for e in 0..5 {
                                let order = [a, b, c, d, e];
                                if order
                                    .iter()
                                    .enumerate()
                                    .any(|(index, value)| order[index + 1..].contains(value))
                                {
                                    continue;
                                }
                                if order == [0, 1, 2, 3, 4] {
                                    continue;
                                }
                                permutation_count += 1;
                                assert_extraction_gap(
                                    mode_then_reserve_identifier_type_assertion_theorem_ast(
                                        source_id,
                                        vec![
                                            ordered[a], ordered[b], ordered[c], ordered[d],
                                            ordered[e],
                                        ],
                                        reserve(),
                                        theorem(),
                                    ),
                                    &format!("definition permutation {a}{b}{c}{d}{e}"),
                                );
                            }
                        }
                    }
                }
            }
            assert_eq!(permutation_count, 119);

            for index in 0..5 {
                let expected_radix = match index {
                    0 => "set",
                    1 => BASE,
                    2 => INNER,
                    3 => MIDDLE,
                    _ => OUTER,
                };
                let mut near_misses = Vec::new();
                let mut missing = exact_modes();
                missing.remove(index);
                near_misses.push(missing);
                let mut duplicate = exact_modes();
                duplicate.push(duplicate[index]);
                near_misses.push(duplicate);
                let mut wrong_label = exact_modes();
                wrong_label[index].label = Some("WrongFourEdgeModeFourHopAssertedHeadDef");
                near_misses.push(wrong_label);
                let mut wrong_pattern = exact_modes();
                wrong_pattern[index].pattern = OTHER;
                near_misses.push(wrong_pattern);
                let mut wrong_radix = exact_modes();
                wrong_radix[index].rhs_shape = if index == 0 {
                    ReserveTypeShape::Builtin("object")
                } else {
                    ReserveTypeShape::QualifiedSymbol(OTHER)
                };
                near_misses.push(wrong_radix);
                let mut recovered = exact_modes();
                recovered[index].recovered = true;
                near_misses.push(recovered);
                let mut contextual = exact_modes();
                contextual[index].local_context = true;
                near_misses.push(contextual);
                let mut parameterized = exact_modes();
                parameterized[index].parameterized_pattern = true;
                near_misses.push(parameterized);
                let mut args = exact_modes();
                args[index].rhs_shape = ReserveTypeShape::QualifiedSymbolWithArgs(expected_radix);
                near_misses.push(args);
                let mut attributed = exact_modes();
                attributed[index].rhs_shape = if index == 0 {
                    ReserveTypeShape::AttributedSet
                } else {
                    ReserveTypeShape::AttributedQualifiedSymbol(expected_radix)
                };
                near_misses.push(attributed);
                for (variant, modes) in near_misses.into_iter().enumerate() {
                    assert_extraction_gap(
                        mode_then_reserve_identifier_type_assertion_theorem_ast(
                            source_id,
                            modes,
                            reserve(),
                            theorem(),
                        ),
                        &format!("definition {index} near miss {variant}"),
                    );
                }
            }

            for (context, index, wrong_radix) in [
                ("TooDeep-to-Outer relation link", 4, MIDDLE),
                ("Outer-to-Middle relation link", 3, INNER),
                ("Middle-to-Inner relation link", 2, BASE),
                ("Inner-to-Base relation link", 1, MIDDLE),
            ] {
                let mut modes = exact_modes();
                modes[index].rhs_shape = ReserveTypeShape::QualifiedSymbol(wrong_radix);
                assert_extraction_gap(
                    mode_then_reserve_identifier_type_assertion_theorem_ast(
                        source_id,
                        modes,
                        reserve(),
                        theorem(),
                    ),
                    context,
                );
            }

            let mut source_near_misses = Vec::new();
            for bad_reserve in [
                ReserveTypeShape::QualifiedSymbol(BASE),
                ReserveTypeShape::QualifiedSymbol(INNER),
                ReserveTypeShape::QualifiedSymbol(MIDDLE),
                ReserveTypeShape::QualifiedSymbol(OUTER),
                ReserveTypeShape::QualifiedSymbolWithArgs(TOO_DEEP),
                ReserveTypeShape::AttributedQualifiedSymbol(TOO_DEEP),
                ReserveTypeShape::Builtin("set"),
                ReserveTypeShape::Builtin("object"),
            ] {
                source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    exact_modes(),
                    vec![reserve_item(vec!["x"], bad_reserve)],
                    theorem(),
                ));
            }
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::QualifiedSymbol(TOO_DEEP)),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                ],
                theorem(),
            ));
            for near_miss_theorem in [
                IdentifierTypeAssertionTheoremSpec {
                    label: "OtherPayloadBoundary",
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    subject: "y",
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(TOO_DEEP),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(OUTER),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(MIDDLE),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(INNER),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::Builtin("set"),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::Builtin("object"),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(OTHER),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(DEEPER),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbolWithArgs(BASE),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::AttributedQualifiedSymbol(BASE),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    negated: true,
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    status: Some("registration"),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    recovered_label: true,
                    ..theorem()
                },
            ] {
                source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    exact_modes(),
                    reserve(),
                    near_miss_theorem,
                ));
            }
            let mut connected_deeper_modes = exact_modes();
            connected_deeper_modes.push(mode_definition_with_label(
                DEEPER,
                "DeeperFourEdgeModeFourHopAssertedHeadDef",
                ReserveTypeShape::QualifiedSymbol(TOO_DEEP),
            ));
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                connected_deeper_modes,
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbol(DEEPER),
                )],
                theorem(),
            ));
            source_near_misses.push(
                modes_then_empty_definition_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    exact_modes(),
                    reserve(),
                    theorem(),
                ),
            );
            for (index, near_miss) in source_near_misses.into_iter().enumerate() {
                assert_extraction_gap(near_miss, &format!("source near miss {index}"));
            }

            let unrelated_import = source_local_and_imported_symbols_env(
                module.clone(),
                &all_modes.map(|spelling| (spelling, SymbolKind::Mode)),
                &[("UnrelatedFourHopMode", SymbolKind::Mode)],
            );
            assert!(
                extract_source_four_edge_local_mode_four_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &unrelated_import,
                )
                .is_some()
            );
            for imported_index in 0..5 {
                let locals = all_modes
                    .iter()
                    .enumerate()
                    .filter(|(index, _)| *index != imported_index)
                    .map(|(_, spelling)| (*spelling, SymbolKind::Mode))
                    .collect::<Vec<_>>();
                for imports in [
                    vec![(all_modes[imported_index], SymbolKind::Mode)],
                    vec![
                        (all_modes[imported_index], SymbolKind::Mode),
                        (all_modes[imported_index], SymbolKind::Mode),
                    ],
                ] {
                    let provenance_near_miss =
                        source_local_and_imported_symbols_env(module.clone(), &locals, &imports);
                    assert_eq!(
                        source_type_elaboration_detail_keys(
                            &exact,
                            module.clone(),
                            &provenance_near_miss,
                        ),
                        vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
                    );
                }
            }
            for imports in [
                all_modes
                    .iter()
                    .map(|spelling| (*spelling, SymbolKind::Mode))
                    .collect::<Vec<_>>(),
                all_modes
                    .iter()
                    .flat_map(|spelling| {
                        [(*spelling, SymbolKind::Mode), (*spelling, SymbolKind::Mode)]
                    })
                    .collect::<Vec<_>>(),
            ] {
                let provenance_near_miss =
                    source_local_and_imported_symbols_env(module.clone(), &[], &imports);
                assert_eq!(
                    source_type_elaboration_detail_keys(
                        &exact,
                        module.clone(),
                        &provenance_near_miss,
                    ),
                    vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
                );
            }
        }

        #[test]
        fn route_rejects_all_46_prior_owner_fixtures_including_task_207_and_tasks_211_through_220()
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
            let plan = build_test_plan(&config).expect("Task 221 repository plan should build");
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
            ];
            assert_eq!(prior_owner_ids.len(), 46);
            assert_eq!(
                &prior_owner_ids[36..],
                &[
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
                ]
            );
            for owner_id in prior_owner_ids {
                let (ordinal, case) = active_type_elaboration_cases(&plan)
                    .enumerate()
                    .find(|(_, case)| case.id.0 == owner_id)
                    .unwrap_or_else(|| panic!("prior owner fixture {owner_id} must be active"));
                let frontend = run_frontend(&workspace_root, case, ordinal)
                    .unwrap_or_else(|error| panic!("owner fixture {owner_id} must parse: {error}"));
                assert!(frontend.diagnostics.is_empty());
                let ast = frontend
                    .ast
                    .unwrap_or_else(|| panic!("owner fixture {owner_id} must produce an AST"));
                let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
                assert!(resolver.detail_keys.is_empty());
                let symbols =
                    augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
                assert_eq!(
                    source_type_elaboration_detail_keys(&ast, resolver.module.clone(), &symbols),
                    Vec::<String>::new(),
                    "prior owner fixture {owner_id} must still reach its active route"
                );
                assert!(
                    extract_source_four_edge_local_mode_four_hop_asserted_head(
                        &ast,
                        resolver.module,
                        &symbols,
                    )
                    .is_none(),
                    "Task 221 must reject prior owner fixture {owner_id}"
                );
            }
        }

        #[test]
        fn task221_synthetic_is_rejected_by_all_46_prior_type_assertion_extractors() {
            let source_id = source_id(221);
            let module = ResolverModuleId::new(
                PackageId::new("test"),
                ModulePath::new("task221_prior_extractor_isolation"),
            );
            let symbols = source_local_symbols_env(
                module.clone(),
                &[
                    (BASE, SymbolKind::Mode),
                    (INNER, SymbolKind::Mode),
                    (MIDDLE, SymbolKind::Mode),
                    (OUTER, SymbolKind::Mode),
                    (TOO_DEEP, SymbolKind::Mode),
                ],
            );
            let exact = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                theorem(),
            );
            assert!(
                extract_source_four_edge_local_mode_four_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                )
                .is_some()
            );
            let prior_results = [
                extract_source_reserved_variable_type_assertion(&exact, module.clone(), &symbols),
                super::super::extract_source_reserved_object_variable_type_assertion(
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
                extract_source_chained_local_mode_radix_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
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
                extract_source_three_edge_local_mode_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
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
                super::super::extract_source_four_edge_local_mode_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                super::super::extract_source_four_edge_local_object_mode_asserted_head(
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
                super::super::extract_source_local_mode_long_chain_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                super::super::extract_source_local_object_mode_long_chain_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_local_mode_long_chain_radix_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                super::super::extract_source_local_object_mode_long_chain_radix_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_two_edge_local_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_two_edge_local_object_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_three_edge_local_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_three_edge_local_object_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_four_edge_local_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_four_edge_local_object_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_three_edge_local_mode_three_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_three_edge_local_object_mode_three_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_four_edge_local_mode_three_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_four_edge_local_object_mode_three_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
            ];
            assert_eq!(prior_results.len(), 46);
            assert!(prior_results.into_iter().all(|result| result.is_none()));
        }
    }
    mod task_222_four_edge_object_four_hop_asserted_head {
        use super::*;

        const BASE: &str = "BaseFourEdgeObjectModeFourHopAssertedHead";
        const INNER: &str = "InnerFourEdgeObjectModeFourHopAssertedHead";
        const MIDDLE: &str = "MiddleFourEdgeObjectModeFourHopAssertedHead";
        const OUTER: &str = "OuterFourEdgeObjectModeFourHopAssertedHead";
        const TOO_DEEP: &str = "TooDeepFourEdgeObjectModeFourHopAssertedHead";
        const OTHER: &str = "OtherFourEdgeObjectModeFourHopAssertedHead";
        const DEEPER: &str = "DeeperFourEdgeObjectModeFourHopAssertedHead";

        fn theorem() -> IdentifierTypeAssertionTheoremSpec<'static> {
            IdentifierTypeAssertionTheoremSpec {
                status: None,
                label: "FourEdgeLocalObjectModeFourHopAssertedHeadPayloadBoundary",
                subject: "x",
                asserted_type: ReserveTypeShape::QualifiedSymbol(BASE),
                recovered_label: false,
                negated: false,
            }
        }

        fn exact_modes() -> Vec<ModeDefinitionSpec> {
            vec![
                mode_definition_with_label(
                    BASE,
                    "BaseFourEdgeObjectModeFourHopAssertedHeadDef",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition_with_label(
                    INNER,
                    "InnerFourEdgeObjectModeFourHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(BASE),
                ),
                mode_definition_with_label(
                    MIDDLE,
                    "MiddleFourEdgeObjectModeFourHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(INNER),
                ),
                mode_definition_with_label(
                    OUTER,
                    "OuterFourEdgeObjectModeFourHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(MIDDLE),
                ),
                mode_definition_with_label(
                    TOO_DEEP,
                    "TooDeepFourEdgeObjectModeFourHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(OUTER),
                ),
            ]
        }

        fn reserve() -> Vec<ReserveItemSpec> {
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol(TOO_DEEP),
            )]
        }

        #[test]
        fn active_fixture_consumes_five_real_object_expansions() {
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
            let plan = build_test_plan(&config).expect("Task 222 repository plan should build");
            let (ordinal, case) = active_type_elaboration_cases(&plan)
                .enumerate()
                .find(|(_, case)| {
                    case.id.0
                        == "pass_type_elaboration_four_edge_local_object_mode_four_hop_asserted_head_001"
                })
                .expect("Task 222 active fixture should be discoverable");
            let frontend = run_frontend(&workspace_root, case, ordinal)
                .expect("Task 222 fixture should run through the real frontend");
            assert!(frontend.diagnostics.is_empty());
            let ast = frontend
                .ast
                .expect("Task 222 fixture should produce an AST");
            let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
            assert!(resolver.detail_keys.is_empty());
            let symbols =
                augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
            assert_eq!(
                source_type_elaboration_detail_keys(&ast, resolver.module.clone(), &symbols),
                Vec::<String>::new()
            );
            let output = source_four_edge_local_object_mode_four_hop_asserted_head_output(
                &ast,
                resolver.module,
                &symbols,
            )
            .expect("Task 222 real AST should reach the exact four-link asserted-head seam");
            assert_source_reserved_variable_type_assertion_output(&output)
                .expect("Task 222 real AST should preserve every checked payload invariant");
            assert_eq!(output.payload.reserve.mode_expansions.len(), 5);
            assert_eq!(output.subject_binding, BindingId::new(0));
            assert_eq!(output.payload.subject_lookup_ordinal, 1);
            assert_eq!(output.term_formula.type_entries().len(), 3);
            assert_eq!(output.term_formula.normalized_types().len(), 1);
            assert_eq!(output.subject_result_input.spelling, TOO_DEEP);
            assert_eq!(output.asserted_type_input.spelling, BASE);
            assert_ne!(
                output.subject_result_input.head,
                output.asserted_type_input.head
            );
            assert_ne!(
                output.subject_result_input.site,
                output.asserted_type_input.site
            );
            assert_ne!(
                output.subject_result_input.source_range,
                output.asserted_type_input.source_range
            );
            let expansion = |spelling| {
                output
                    .payload
                    .reserve
                    .mode_expansions
                    .iter()
                    .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(spelling))
                    .map(|(_, expansion)| expansion)
                    .unwrap()
            };
            let too_deep = expansion(TOO_DEEP);
            let outer = expansion(OUTER);
            let middle = expansion(MIDDLE);
            let inner = expansion(INNER);
            let base = expansion(BASE);
            assert_eq!(too_deep.radix.spelling, OUTER);
            assert_eq!(outer.radix.spelling, MIDDLE);
            assert_eq!(middle.radix.spelling, INNER);
            assert_eq!(inner.radix.spelling, BASE);
            assert_eq!(inner.radix.head, output.asserted_type_input.head);
            assert_eq!(base.radix.spelling, "object");
            assert_eq!(base.radix.head, TypeHeadInput::BuiltinObject);
            let (_, normalized) = output
                .term_formula
                .normalized_types()
                .iter()
                .next()
                .unwrap();
            assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
            assert_eq!(normalized.source.range, base.radix.source_range);
            assert_eq!(normalized.source.spelling, "object");
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
        fn synthetic_exactness_matrix_rejects_every_near_miss() {
            let source_id = source_id(222);
            let module = ResolverModuleId::new(
                PackageId::new("test"),
                ModulePath::new("four_edge_local_object_mode_four_hop_asserted_head"),
            );
            let all_modes = [BASE, INNER, MIDDLE, OUTER, TOO_DEEP];
            let symbols = source_local_symbols_env(
                module.clone(),
                &[
                    (BASE, SymbolKind::Mode),
                    (INNER, SymbolKind::Mode),
                    (MIDDLE, SymbolKind::Mode),
                    (OUTER, SymbolKind::Mode),
                    (TOO_DEEP, SymbolKind::Mode),
                    (OTHER, SymbolKind::Mode),
                    (DEEPER, SymbolKind::Mode),
                ],
            );
            let exact = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                theorem(),
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
                Vec::<String>::new()
            );
            let payload = extract_source_four_edge_local_object_mode_four_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("the exact object-terminal four-edge four-hop source should extract");
            assert_eq!(payload.reserve.mode_expansions.len(), 5);
            assert_eq!(payload.reserve.bridge.bindings().len(), 1);
            assert_eq!(payload.subject_lookup_ordinal, 1);
            assert_eq!(payload.reserve.bridge.bindings()[0].type_spelling, TOO_DEEP);
            assert_eq!(payload.asserted_type.spelling, BASE);

            let exact_output = || {
                source_four_edge_local_object_mode_four_hop_asserted_head_output(
                    &exact,
                    module.clone(),
                    &symbols,
                )
                .unwrap()
            };
            let output = exact_output();
            assert_source_reserved_variable_type_assertion_output(&output)
                .expect("Task 222 exact checker output must satisfy every invariant");

            let assert_invalid_output = |invalid| {
                let invalid_result =
                    assert_source_reserved_variable_type_assertion_output(&invalid)
                        .map(|()| invalid);
                assert_eq!(
                    source_reserved_variable_type_assertion_result_detail_keys(
                        invalid_result,
                        TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_FOUR_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
                    ),
                    vec![
                        TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_FOUR_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY
                            .to_owned()
                    ]
                );
            };
            for removed in all_modes {
                let mut invalid = exact_output();
                invalid
                    .payload
                    .reserve
                    .mode_expansions
                    .retain(|symbol, _| source_mode_symbol_spelling(symbol) != Some(removed));
                assert_invalid_output(invalid);
            }
            let mut corruptions = Vec::new();
            let mut invalid = exact_output();
            invalid.subject_binding = BindingId::new(1);
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.payload.subject_lookup_ordinal = 2;
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.subject_result_input.spelling = OUTER.to_owned();
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.subject_result_input.head = invalid.asserted_type_input.head.clone();
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.subject_result_input.site = invalid.asserted_type_input.site.clone();
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.subject_result_input.source_range = invalid.asserted_type_input.source_range;
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.asserted_type_input.spelling = INNER.to_owned();
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.asserted_type_input.head = invalid.subject_result_input.head.clone();
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.asserted_type_input.site = invalid.subject_result_input.site.clone();
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.asserted_type_input.source_range = invalid.subject_result_input.source_range;
            corruptions.push(invalid);

            let mut invalid = exact_output();
            let middle_symbol = invalid
                .payload
                .reserve
                .mode_expansions
                .keys()
                .find(|symbol| source_mode_symbol_spelling(symbol) == Some(MIDDLE))
                .unwrap()
                .clone();
            let (_, too_deep) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(TOO_DEEP))
                .unwrap();
            too_deep.radix.spelling = MIDDLE.to_owned();
            too_deep.radix.head = TypeHeadInput::Symbol(middle_symbol);
            corruptions.push(invalid);
            let mut invalid = exact_output();
            let inner_symbol = invalid
                .payload
                .reserve
                .mode_expansions
                .keys()
                .find(|symbol| source_mode_symbol_spelling(symbol) == Some(INNER))
                .unwrap()
                .clone();
            let (_, outer) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(OUTER))
                .unwrap();
            outer.radix.spelling = INNER.to_owned();
            outer.radix.head = TypeHeadInput::Symbol(inner_symbol);
            corruptions.push(invalid);
            let mut invalid = exact_output();
            let base_symbol = invalid
                .payload
                .reserve
                .mode_expansions
                .keys()
                .find(|symbol| source_mode_symbol_spelling(symbol) == Some(BASE))
                .unwrap()
                .clone();
            let (_, middle) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(MIDDLE))
                .unwrap();
            middle.radix.spelling = BASE.to_owned();
            middle.radix.head = TypeHeadInput::Symbol(base_symbol);
            corruptions.push(invalid);
            let mut invalid = exact_output();
            let (_, inner) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(INNER))
                .unwrap();
            inner.radix.spelling = "set".to_owned();
            inner.radix.head = TypeHeadInput::BuiltinSet;
            corruptions.push(invalid);
            let mut invalid = exact_output();
            let (_, base) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(BASE))
                .unwrap();
            base.radix.spelling = "set".to_owned();
            base.radix.head = TypeHeadInput::BuiltinSet;
            corruptions.push(invalid);
            let mut invalid = exact_output();
            let (_, base) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(BASE))
                .unwrap();
            base.radix.source_range = range(source_id, 0, 1);
            corruptions.push(invalid);
            for invalid in corruptions {
                assert_invalid_output(invalid);
            }
            assert_source_reserved_variable_type_assertion_output(&exact_output())
                .expect("mutating cloned Task 222 outputs must not mutate the exact output");

            let assert_extraction_gap = |ast, context: &str| {
                assert_eq!(
                    source_type_elaboration_detail_keys(&ast, module.clone(), &symbols),
                    vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()],
                    "{context}"
                );
            };
            let ordered = exact_modes();
            let mut permutation_count = 0;
            for a in 0..5 {
                for b in 0..5 {
                    for c in 0..5 {
                        for d in 0..5 {
                            for e in 0..5 {
                                let order = [a, b, c, d, e];
                                if order
                                    .iter()
                                    .enumerate()
                                    .any(|(index, value)| order[index + 1..].contains(value))
                                {
                                    continue;
                                }
                                if order == [0, 1, 2, 3, 4] {
                                    continue;
                                }
                                permutation_count += 1;
                                assert_extraction_gap(
                                    mode_then_reserve_identifier_type_assertion_theorem_ast(
                                        source_id,
                                        vec![
                                            ordered[a], ordered[b], ordered[c], ordered[d],
                                            ordered[e],
                                        ],
                                        reserve(),
                                        theorem(),
                                    ),
                                    &format!("definition permutation {a}{b}{c}{d}{e}"),
                                );
                            }
                        }
                    }
                }
            }
            assert_eq!(permutation_count, 119);

            for index in 0..5 {
                let expected_radix = match index {
                    0 => "object",
                    1 => BASE,
                    2 => INNER,
                    3 => MIDDLE,
                    _ => OUTER,
                };
                let mut near_misses = Vec::new();
                let mut missing = exact_modes();
                missing.remove(index);
                near_misses.push(missing);
                let mut duplicate = exact_modes();
                duplicate.push(duplicate[index]);
                near_misses.push(duplicate);
                let mut wrong_label = exact_modes();
                wrong_label[index].label = Some("WrongFourEdgeObjectModeFourHopAssertedHeadDef");
                near_misses.push(wrong_label);
                let mut wrong_pattern = exact_modes();
                wrong_pattern[index].pattern = OTHER;
                near_misses.push(wrong_pattern);
                let mut wrong_radix = exact_modes();
                wrong_radix[index].rhs_shape = if index == 0 {
                    ReserveTypeShape::Builtin("set")
                } else {
                    ReserveTypeShape::QualifiedSymbol(OTHER)
                };
                near_misses.push(wrong_radix);
                let mut recovered = exact_modes();
                recovered[index].recovered = true;
                near_misses.push(recovered);
                let mut contextual = exact_modes();
                contextual[index].local_context = true;
                near_misses.push(contextual);
                let mut parameterized = exact_modes();
                parameterized[index].parameterized_pattern = true;
                near_misses.push(parameterized);
                let mut args = exact_modes();
                args[index].rhs_shape = ReserveTypeShape::QualifiedSymbolWithArgs(expected_radix);
                near_misses.push(args);
                let mut attributed = exact_modes();
                attributed[index].rhs_shape = if index == 0 {
                    ReserveTypeShape::AttributedObject
                } else {
                    ReserveTypeShape::AttributedQualifiedSymbol(expected_radix)
                };
                near_misses.push(attributed);
                for (variant, modes) in near_misses.into_iter().enumerate() {
                    assert_extraction_gap(
                        mode_then_reserve_identifier_type_assertion_theorem_ast(
                            source_id,
                            modes,
                            reserve(),
                            theorem(),
                        ),
                        &format!("definition {index} near miss {variant}"),
                    );
                }
            }

            for (context, index, wrong_radix) in [
                ("TooDeep-to-Outer relation link", 4, MIDDLE),
                ("Outer-to-Middle relation link", 3, INNER),
                ("Middle-to-Inner relation link", 2, BASE),
                ("Inner-to-Base relation link", 1, MIDDLE),
            ] {
                let mut modes = exact_modes();
                modes[index].rhs_shape = ReserveTypeShape::QualifiedSymbol(wrong_radix);
                assert_extraction_gap(
                    mode_then_reserve_identifier_type_assertion_theorem_ast(
                        source_id,
                        modes,
                        reserve(),
                        theorem(),
                    ),
                    context,
                );
            }

            let mut source_near_misses = Vec::new();
            for bad_reserve in [
                ReserveTypeShape::QualifiedSymbol(BASE),
                ReserveTypeShape::QualifiedSymbol(INNER),
                ReserveTypeShape::QualifiedSymbol(MIDDLE),
                ReserveTypeShape::QualifiedSymbol(OUTER),
                ReserveTypeShape::QualifiedSymbolWithArgs(TOO_DEEP),
                ReserveTypeShape::AttributedQualifiedSymbol(TOO_DEEP),
                ReserveTypeShape::Builtin("set"),
                ReserveTypeShape::Builtin("object"),
            ] {
                source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    exact_modes(),
                    vec![reserve_item(vec!["x"], bad_reserve)],
                    theorem(),
                ));
            }
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::QualifiedSymbol(TOO_DEEP)),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("object")),
                ],
                theorem(),
            ));
            for near_miss_theorem in [
                IdentifierTypeAssertionTheoremSpec {
                    label: "OtherPayloadBoundary",
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    subject: "y",
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(TOO_DEEP),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(OUTER),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(MIDDLE),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(INNER),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::Builtin("set"),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::Builtin("object"),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(OTHER),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(DEEPER),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbolWithArgs(BASE),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::AttributedQualifiedSymbol(BASE),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    negated: true,
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    status: Some("registration"),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    recovered_label: true,
                    ..theorem()
                },
            ] {
                source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    exact_modes(),
                    reserve(),
                    near_miss_theorem,
                ));
            }
            let mut connected_deeper_modes = exact_modes();
            connected_deeper_modes.push(mode_definition_with_label(
                DEEPER,
                "DeeperFourEdgeObjectModeFourHopAssertedHeadDef",
                ReserveTypeShape::QualifiedSymbol(TOO_DEEP),
            ));
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                connected_deeper_modes,
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbol(DEEPER),
                )],
                theorem(),
            ));
            source_near_misses.push(
                modes_then_empty_definition_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    exact_modes(),
                    reserve(),
                    theorem(),
                ),
            );
            for (index, near_miss) in source_near_misses.into_iter().enumerate() {
                assert_extraction_gap(near_miss, &format!("source near miss {index}"));
            }

            let unrelated_import = source_local_and_imported_symbols_env(
                module.clone(),
                &all_modes.map(|spelling| (spelling, SymbolKind::Mode)),
                &[("UnrelatedFourHopMode", SymbolKind::Mode)],
            );
            assert!(
                extract_source_four_edge_local_object_mode_four_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &unrelated_import,
                )
                .is_some()
            );
            for imported_index in 0..5 {
                let locals = all_modes
                    .iter()
                    .enumerate()
                    .filter(|(index, _)| *index != imported_index)
                    .map(|(_, spelling)| (*spelling, SymbolKind::Mode))
                    .collect::<Vec<_>>();
                for imports in [
                    vec![(all_modes[imported_index], SymbolKind::Mode)],
                    vec![
                        (all_modes[imported_index], SymbolKind::Mode),
                        (all_modes[imported_index], SymbolKind::Mode),
                    ],
                ] {
                    let provenance_near_miss =
                        source_local_and_imported_symbols_env(module.clone(), &locals, &imports);
                    assert_eq!(
                        source_type_elaboration_detail_keys(
                            &exact,
                            module.clone(),
                            &provenance_near_miss,
                        ),
                        vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
                    );
                }
            }
            for imports in [
                all_modes
                    .iter()
                    .map(|spelling| (*spelling, SymbolKind::Mode))
                    .collect::<Vec<_>>(),
                all_modes
                    .iter()
                    .flat_map(|spelling| {
                        [(*spelling, SymbolKind::Mode), (*spelling, SymbolKind::Mode)]
                    })
                    .collect::<Vec<_>>(),
            ] {
                let provenance_near_miss =
                    source_local_and_imported_symbols_env(module.clone(), &[], &imports);
                assert_eq!(
                    source_type_elaboration_detail_keys(
                        &exact,
                        module.clone(),
                        &provenance_near_miss,
                    ),
                    vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
                );
            }
        }

        #[test]
        fn route_rejects_all_47_prior_owner_fixtures_including_task_208_and_tasks_211_through_221()
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
            let plan = build_test_plan(&config).expect("Task 222 repository plan should build");
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
            ];
            assert_eq!(prior_owner_ids.len(), 47);
            assert_eq!(
                &prior_owner_ids[36..],
                &[
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
                ]
            );
            for owner_id in prior_owner_ids {
                let (ordinal, case) = active_type_elaboration_cases(&plan)
                    .enumerate()
                    .find(|(_, case)| case.id.0 == owner_id)
                    .unwrap_or_else(|| panic!("prior owner fixture {owner_id} must be active"));
                let frontend = run_frontend(&workspace_root, case, ordinal)
                    .unwrap_or_else(|error| panic!("owner fixture {owner_id} must parse: {error}"));
                assert!(frontend.diagnostics.is_empty());
                let ast = frontend
                    .ast
                    .unwrap_or_else(|| panic!("owner fixture {owner_id} must produce an AST"));
                let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
                assert!(resolver.detail_keys.is_empty());
                let symbols =
                    augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
                assert_eq!(
                    source_type_elaboration_detail_keys(&ast, resolver.module.clone(), &symbols),
                    Vec::<String>::new(),
                    "prior owner fixture {owner_id} must still reach its active route"
                );
                assert!(
                    extract_source_four_edge_local_object_mode_four_hop_asserted_head(
                        &ast,
                        resolver.module,
                        &symbols,
                    )
                    .is_none(),
                    "Task 222 must reject prior owner fixture {owner_id}"
                );
            }
        }

        #[test]
        fn task222_synthetic_is_rejected_by_all_47_prior_type_assertion_extractors() {
            let source_id = source_id(222);
            let module = ResolverModuleId::new(
                PackageId::new("test"),
                ModulePath::new("task222_prior_extractor_isolation"),
            );
            let symbols = source_local_symbols_env(
                module.clone(),
                &[
                    (BASE, SymbolKind::Mode),
                    (INNER, SymbolKind::Mode),
                    (MIDDLE, SymbolKind::Mode),
                    (OUTER, SymbolKind::Mode),
                    (TOO_DEEP, SymbolKind::Mode),
                ],
            );
            let exact = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                theorem(),
            );
            assert!(
                extract_source_four_edge_local_object_mode_four_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                )
                .is_some()
            );
            let prior_results = [
                extract_source_reserved_variable_type_assertion(&exact, module.clone(), &symbols),
                super::super::extract_source_reserved_object_variable_type_assertion(
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
                extract_source_chained_local_mode_radix_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
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
                extract_source_three_edge_local_mode_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
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
                super::super::extract_source_four_edge_local_mode_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                super::super::extract_source_four_edge_local_object_mode_asserted_head(
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
                super::super::extract_source_local_mode_long_chain_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                super::super::extract_source_local_object_mode_long_chain_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_local_mode_long_chain_radix_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                super::super::extract_source_local_object_mode_long_chain_radix_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_two_edge_local_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_two_edge_local_object_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_three_edge_local_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_three_edge_local_object_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_four_edge_local_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_four_edge_local_object_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_three_edge_local_mode_three_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_three_edge_local_object_mode_three_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_four_edge_local_mode_three_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_four_edge_local_object_mode_three_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_four_edge_local_mode_four_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
            ];
            assert_eq!(prior_results.len(), 47);
            assert!(prior_results.into_iter().all(|result| result.is_none()));
        }
    }
