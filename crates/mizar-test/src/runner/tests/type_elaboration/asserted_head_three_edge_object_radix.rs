    #[test]
    fn source_three_edge_local_object_mode_radix_asserted_head_consumes_immediate_expansion() {
        let source_id = source_id(206);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("three_edge_local_object_mode_radix_asserted_head"),
        );
        let base_mode = "BaseThreeEdgeObjectModeRadixAssertedHead";
        let inner_mode = "InnerThreeEdgeObjectModeRadixAssertedHead";
        let middle_mode = "MiddleThreeEdgeObjectModeRadixAssertedHead";
        let outer_mode = "OuterThreeEdgeObjectModeRadixAssertedHead";
        let all_modes = [base_mode, inner_mode, middle_mode, outer_mode];
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                (base_mode, SymbolKind::Mode),
                (inner_mode, SymbolKind::Mode),
                (middle_mode, SymbolKind::Mode),
                (outer_mode, SymbolKind::Mode),
                (
                    "DeeperThreeEdgeObjectModeRadixAssertedHead",
                    SymbolKind::Mode,
                ),
                (
                    "OtherThreeEdgeObjectModeRadixAssertedHead",
                    SymbolKind::Mode,
                ),
            ],
        );
        let theorem = exact_three_edge_local_object_mode_radix_asserted_head_spec();
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    base_mode,
                    "BaseThreeEdgeObjectModeRadixAssertedHeadDef",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition_with_label(
                    inner_mode,
                    "InnerThreeEdgeObjectModeRadixAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(base_mode),
                ),
                mode_definition_with_label(
                    middle_mode,
                    "MiddleThreeEdgeObjectModeRadixAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(inner_mode),
                ),
                mode_definition_with_label(
                    outer_mode,
                    "OuterThreeEdgeObjectModeRadixAssertedHeadDef",
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
        for rejected in [
            extract_source_reserved_variable_type_assertion(&exact, module.clone(), &symbols),
            extract_source_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_chained_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_mode_asserted_head(&exact, module.clone(), &symbols),
            extract_source_chained_local_mode_radix_asserted_head(&exact, module.clone(), &symbols),
            extract_source_two_edge_local_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
        ] {
            assert!(rejected.is_none());
        }
        for rejected in [
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
            extract_source_chained_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_chained_local_object_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_two_edge_local_object_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
        ] {
            assert!(rejected.is_none());
        }

        assert!(
            extract_source_three_edge_local_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none()
        );

        let payload = extract_source_three_edge_local_object_mode_radix_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact three-edge immediate-radix asserted head should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 4);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(payload.subject_lookup_ordinal, 1);
        let source_binding = &payload.reserve.bridge.bindings()[0];
        assert_eq!(source_binding.type_spelling, outer_mode);
        assert_eq!(payload.asserted_type.spelling, middle_mode);
        assert!(matches!(source_binding.type_head, TypeHeadInput::Symbol(_)));
        assert!(matches!(
            payload.asserted_type.head,
            TypeHeadInput::Symbol(_)
        ));
        assert_ne!(payload.asserted_type.head, source_binding.type_head);
        assert_ne!(payload.asserted_type_site, payload.subject_site);
        assert_ne!(payload.asserted_type.range, source_binding.type_range);
        let outer_expansion = payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(outer_mode))
            .map(|(_, expansion)| expansion)
            .expect("outer binding expansion should exist");
        assert_eq!(outer_expansion.radix.spelling, middle_mode);
        assert_eq!(outer_expansion.radix.head, payload.asserted_type.head);

        let output = source_three_edge_local_object_mode_radix_asserted_head_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact Task 206 source should reach the checker");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 206 checker payload invariants should hold");
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.payload.reserve.mode_expansions.len(), 4);
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert_eq!(output.subject_result_input.spelling, outer_mode);
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
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(base_mode))
            .map(|(_, expansion)| &expansion.radix)
            .expect("base object terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("one normalized builtin-object identity should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_ne!(normalized.head, TypeHeadRef::BuiltinSet);
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

        let exact_output = || {
            source_three_edge_local_object_mode_radix_asserted_head_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .unwrap()
        };
        let assert_invalid_output = |invalid| {
            let invalid_result =
                assert_source_reserved_variable_type_assertion_output(&invalid).map(|()| invalid);
            assert_eq!(
                source_reserved_variable_type_assertion_result_detail_keys(
                    invalid_result,
                    TYPE_ELABORATION_THREE_EDGE_LOCAL_OBJECT_MODE_RADIX_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
                ),
                vec![
                    TYPE_ELABORATION_THREE_EDGE_LOCAL_OBJECT_MODE_RADIX_ASSERTED_HEAD_INVALID_PAYLOAD_KEY
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
        invalid.subject_result_input.head = TypeHeadInput::BuiltinSet;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.site = invalid.asserted_type_input.site.clone();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.source_range = invalid.asserted_type_input.source_range;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.spelling = inner_mode.to_owned();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.head = invalid.subject_result_input.head.clone();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.head = TypeHeadInput::BuiltinSet;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.site = invalid.subject_result_input.site.clone();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.source_range = invalid.subject_result_input.source_range;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        let binding_head = invalid.subject_result_input.head.clone();
        let (_, outer_expansion) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(outer_mode))
            .unwrap();
        outer_expansion.radix.spelling = outer_mode.to_owned();
        outer_expansion.radix.head = binding_head;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        let (_, middle_expansion) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(middle_mode))
            .unwrap();
        middle_expansion.radix.spelling = "set".to_owned();
        middle_expansion.radix.head = TypeHeadInput::BuiltinSet;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        let (_, base_expansion) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(base_mode))
            .unwrap();
        base_expansion.radix.source_range = range(source_id, 0, 1);
        corruptions.push(invalid);
        let mut invalid = exact_output();
        let (_, base_expansion) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(base_mode))
            .unwrap();
        base_expansion.radix.spelling = "set".to_owned();
        base_expansion.radix.head = TypeHeadInput::BuiltinSet;
        corruptions.push(invalid);
        for invalid in corruptions {
            assert_invalid_output(invalid);
        }
        assert_source_reserved_variable_type_assertion_output(&exact_output())
            .expect("corrupting cloned Task 206 outputs must not mutate the exact output");

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
            let mut missing = exact_modes();
            missing.remove(index);
            assert_extraction_gap(
                mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    missing,
                    reserve(),
                    theorem,
                ),
                &format!("missing definition {index}"),
            );
            let mut duplicate = exact_modes();
            duplicate.push(duplicate[index]);
            assert_extraction_gap(
                mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    duplicate,
                    reserve(),
                    theorem,
                ),
                &format!("duplicate definition {index}"),
            );
            let mut wrong_label = exact_modes();
            wrong_label[index].label = Some("WrongThreeEdgeObjectModeRadixAssertedHeadDef");
            assert_extraction_gap(
                mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    wrong_label,
                    reserve(),
                    theorem,
                ),
                &format!("wrong definition label {index}"),
            );
            let mut wrong_spelling = exact_modes();
            wrong_spelling[index].pattern = "OtherThreeEdgeObjectModeRadixAssertedHead";
            assert_extraction_gap(
                mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    wrong_spelling,
                    reserve(),
                    theorem,
                ),
                &format!("wrong definition spelling {index}"),
            );
            let mut wrong_radix = exact_modes();
            wrong_radix[index].rhs_shape = if index == 0 {
                ReserveTypeShape::Builtin("set")
            } else {
                ReserveTypeShape::QualifiedSymbol("OtherThreeEdgeObjectModeRadixAssertedHead")
            };
            assert_extraction_gap(
                mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    wrong_radix,
                    reserve(),
                    theorem,
                ),
                &format!("wrong definition radix {index}"),
            );
        }

        let mut deeper_modes = exact_modes();
        deeper_modes.push(mode_definition_with_label(
            "DeeperThreeEdgeObjectModeRadixAssertedHead",
            "DeeperThreeEdgeObjectModeRadixAssertedHeadDef",
            ReserveTypeShape::QualifiedSymbol(outer_mode),
        ));
        assert_extraction_gap(
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                deeper_modes,
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbol("DeeperThreeEdgeObjectModeRadixAssertedHead"),
                )],
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(outer_mode),
                    ..theorem
                },
            ),
            "connected deeper immediate-radix chain",
        );
        for asserted_type in [
            ReserveTypeShape::QualifiedSymbol(outer_mode),
            ReserveTypeShape::QualifiedSymbol(inner_mode),
            ReserveTypeShape::QualifiedSymbol(base_mode),
            ReserveTypeShape::QualifiedSymbol("OtherThreeEdgeObjectModeRadixAssertedHead"),
            ReserveTypeShape::Builtin("object"),
            ReserveTypeShape::Builtin("set"),
            ReserveTypeShape::QualifiedSymbolWithArgs(middle_mode),
            ReserveTypeShape::AttributedQualifiedSymbol(middle_mode),
        ] {
            assert_extraction_gap(
                mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    exact_modes(),
                    reserve(),
                    IdentifierTypeAssertionTheoremSpec {
                        asserted_type,
                        ..theorem
                    },
                ),
                "asserted-head near miss",
            );
        }

        let unrelated_import = source_local_and_imported_symbols_env(
            module.clone(),
            &[
                (base_mode, SymbolKind::Mode),
                (inner_mode, SymbolKind::Mode),
                (middle_mode, SymbolKind::Mode),
                (outer_mode, SymbolKind::Mode),
            ],
            &[(
                "UnrelatedThreeEdgeObjectModeRadixAssertedHead",
                SymbolKind::Mode,
            )],
        );
        assert!(
            extract_source_three_edge_local_object_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &unrelated_import,
            )
            .is_some()
        );
        for missing_index in 0..4 {
            let local = all_modes
                .iter()
                .enumerate()
                .filter_map(|(index, spelling)| {
                    (index != missing_index).then_some((*spelling, SymbolKind::Mode))
                })
                .collect::<Vec<_>>();
            for imported in [
                vec![(all_modes[missing_index], SymbolKind::Mode)],
                vec![
                    (all_modes[missing_index], SymbolKind::Mode),
                    (all_modes[missing_index], SymbolKind::Mode),
                ],
            ] {
                let bad_symbols = source_local_and_imported_symbols_env(
                    module.clone(),
                    local.as_slice(),
                    imported.as_slice(),
                );
                assert_eq!(
                    source_type_elaboration_detail_keys(&exact, module.clone(), &bad_symbols),
                    vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
                );
            }
        }
        assert_task_206_route_isolation(source_id, module);
    }

    fn assert_task_206_route_isolation(source_id: SourceId, module: ResolverModuleId) {
        let assert_isolated =
            |ast: &SurfaceAst, symbols: &SymbolEnv, owner_extracts: bool, task: &str| {
                assert!(owner_extracts, "{task} owner route should extract");
                assert!(
                    extract_source_three_edge_local_object_mode_radix_asserted_head(
                        ast,
                        module.clone(),
                        symbols,
                    )
                    .is_none(),
                    "Task 206 must reject {task}"
                );
            };

        let task_122_symbols = source_local_symbols_env(module.clone(), &[]);
        let task_122 = reserve_then_identifier_type_assertion_theorem_ast(
            source_id,
            vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
            exact_identifier_type_assertion_spec(),
        );
        assert_isolated(
            &task_122,
            &task_122_symbols,
            extract_source_reserved_variable_type_assertion(
                &task_122,
                module.clone(),
                &task_122_symbols,
            )
            .is_some(),
            "Task 122",
        );

        let task_138_symbols = source_local_symbols_env(
            module.clone(),
            &[("LocalModeTypeAssertion", SymbolKind::Mode)],
        );
        let task_138 = mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            [mode_definition_with_label(
                "LocalModeTypeAssertion",
                "LocalModeTypeAssertionDef",
                ReserveTypeShape::Builtin("set"),
            )],
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("LocalModeTypeAssertion"),
            )],
            exact_local_mode_identifier_type_assertion_spec(),
        );
        assert_isolated(
            &task_138,
            &task_138_symbols,
            extract_source_local_mode_reserved_variable_type_assertion(
                &task_138,
                module.clone(),
                &task_138_symbols,
            )
            .is_some(),
            "Task 138",
        );

        let task_146_symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseModeTypeAssertion", SymbolKind::Mode),
                ("ChainModeTypeAssertion", SymbolKind::Mode),
            ],
        );
        let task_146 = mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            [
                mode_definition_with_label(
                    "BaseModeTypeAssertion",
                    "BaseModeTypeAssertionDef",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition_with_label(
                    "ChainModeTypeAssertion",
                    "ChainModeTypeAssertionDef",
                    ReserveTypeShape::QualifiedSymbol("BaseModeTypeAssertion"),
                ),
            ],
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("ChainModeTypeAssertion"),
            )],
            exact_chained_local_mode_identifier_type_assertion_spec(),
        );
        assert_isolated(
            &task_146,
            &task_146_symbols,
            extract_source_chained_local_mode_reserved_variable_type_assertion(
                &task_146,
                module.clone(),
                &task_146_symbols,
            )
            .is_some(),
            "Task 146",
        );

        let task_148_symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseTwoEdgeModeTypeAssertion", SymbolKind::Mode),
                ("MiddleTwoEdgeModeTypeAssertion", SymbolKind::Mode),
                ("OuterTwoEdgeModeTypeAssertion", SymbolKind::Mode),
            ],
        );
        let task_148 = mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            [
                mode_definition_with_label(
                    "BaseTwoEdgeModeTypeAssertion",
                    "BaseTwoEdgeModeTypeAssertionDef",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition_with_label(
                    "MiddleTwoEdgeModeTypeAssertion",
                    "MiddleTwoEdgeModeTypeAssertionDef",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeModeTypeAssertion"),
                ),
                mode_definition_with_label(
                    "OuterTwoEdgeModeTypeAssertion",
                    "OuterTwoEdgeModeTypeAssertionDef",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeModeTypeAssertion"),
                ),
            ],
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("OuterTwoEdgeModeTypeAssertion"),
            )],
            exact_two_edge_local_mode_identifier_type_assertion_spec(),
        );
        assert_isolated(
            &task_148,
            &task_148_symbols,
            extract_source_two_edge_local_mode_reserved_variable_type_assertion(
                &task_148,
                module.clone(),
                &task_148_symbols,
            )
            .is_some(),
            "Task 148",
        );

        let task_150_symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseThreeEdgeModeTypeAssertion", SymbolKind::Mode),
                ("InnerThreeEdgeModeTypeAssertion", SymbolKind::Mode),
                ("MiddleThreeEdgeModeTypeAssertion", SymbolKind::Mode),
                ("OuterThreeEdgeModeTypeAssertion", SymbolKind::Mode),
            ],
        );
        let task_150 = mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            [
                mode_definition_with_label(
                    "BaseThreeEdgeModeTypeAssertion",
                    "BaseThreeEdgeModeTypeAssertionDef",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition_with_label(
                    "InnerThreeEdgeModeTypeAssertion",
                    "InnerThreeEdgeModeTypeAssertionDef",
                    ReserveTypeShape::QualifiedSymbol("BaseThreeEdgeModeTypeAssertion"),
                ),
                mode_definition_with_label(
                    "MiddleThreeEdgeModeTypeAssertion",
                    "MiddleThreeEdgeModeTypeAssertionDef",
                    ReserveTypeShape::QualifiedSymbol("InnerThreeEdgeModeTypeAssertion"),
                ),
                mode_definition_with_label(
                    "OuterThreeEdgeModeTypeAssertion",
                    "OuterThreeEdgeModeTypeAssertionDef",
                    ReserveTypeShape::QualifiedSymbol("MiddleThreeEdgeModeTypeAssertion"),
                ),
            ],
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("OuterThreeEdgeModeTypeAssertion"),
            )],
            exact_three_edge_local_mode_identifier_type_assertion_spec(),
        );
        assert_isolated(
            &task_150,
            &task_150_symbols,
            extract_source_three_edge_local_mode_reserved_variable_type_assertion(
                &task_150,
                module.clone(),
                &task_150_symbols,
            )
            .is_some(),
            "Task 150",
        );

        let task_195_symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseThreeEdgeModeAssertedHead", SymbolKind::Mode),
                ("InnerThreeEdgeModeAssertedHead", SymbolKind::Mode),
                ("MiddleThreeEdgeModeAssertedHead", SymbolKind::Mode),
                ("OuterThreeEdgeModeAssertedHead", SymbolKind::Mode),
            ],
        );
        let task_195 = mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            [
                mode_definition_with_label(
                    "BaseThreeEdgeModeAssertedHead",
                    "BaseThreeEdgeModeAssertedHeadDef",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition_with_label(
                    "InnerThreeEdgeModeAssertedHead",
                    "InnerThreeEdgeModeAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol("BaseThreeEdgeModeAssertedHead"),
                ),
                mode_definition_with_label(
                    "MiddleThreeEdgeModeAssertedHead",
                    "MiddleThreeEdgeModeAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol("InnerThreeEdgeModeAssertedHead"),
                ),
                mode_definition_with_label(
                    "OuterThreeEdgeModeAssertedHead",
                    "OuterThreeEdgeModeAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol("MiddleThreeEdgeModeAssertedHead"),
                ),
            ],
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("OuterThreeEdgeModeAssertedHead"),
            )],
            IdentifierTypeAssertionTheoremSpec {
                status: None,
                label: "ThreeEdgeLocalModeAssertedHeadPayloadBoundary",
                subject: "x",
                asserted_type: ReserveTypeShape::QualifiedSymbol("OuterThreeEdgeModeAssertedHead"),
                recovered_label: false,
                negated: false,
            },
        );
        assert_isolated(
            &task_195,
            &task_195_symbols,
            extract_source_three_edge_local_mode_asserted_head(
                &task_195,
                module.clone(),
                &task_195_symbols,
            )
            .is_some(),
            "Task 195",
        );

        let task_201_symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseModeRadixAssertedHead", SymbolKind::Mode),
                ("OuterModeRadixAssertedHead", SymbolKind::Mode),
            ],
        );
        let task_201 = mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            [
                mode_definition_with_label(
                    "BaseModeRadixAssertedHead",
                    "BaseModeRadixAssertedHeadDef",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition_with_label(
                    "OuterModeRadixAssertedHead",
                    "OuterModeRadixAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol("BaseModeRadixAssertedHead"),
                ),
            ],
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("OuterModeRadixAssertedHead"),
            )],
            exact_chained_local_mode_radix_asserted_head_spec(),
        );
        assert_isolated(
            &task_201,
            &task_201_symbols,
            extract_source_chained_local_mode_radix_asserted_head(
                &task_201,
                module.clone(),
                &task_201_symbols,
            )
            .is_some(),
            "Task 201",
        );

        let task_203_symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseTwoEdgeModeRadixAssertedHead", SymbolKind::Mode),
                ("MiddleTwoEdgeModeRadixAssertedHead", SymbolKind::Mode),
                ("OuterTwoEdgeModeRadixAssertedHead", SymbolKind::Mode),
            ],
        );
        let task_203 = mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            [
                mode_definition_with_label(
                    "BaseTwoEdgeModeRadixAssertedHead",
                    "BaseTwoEdgeModeRadixAssertedHeadDef",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition_with_label(
                    "MiddleTwoEdgeModeRadixAssertedHead",
                    "MiddleTwoEdgeModeRadixAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeModeRadixAssertedHead"),
                ),
                mode_definition_with_label(
                    "OuterTwoEdgeModeRadixAssertedHead",
                    "OuterTwoEdgeModeRadixAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeModeRadixAssertedHead"),
                ),
            ],
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("OuterTwoEdgeModeRadixAssertedHead"),
            )],
            exact_two_edge_local_mode_radix_asserted_head_spec(),
        );
        assert_isolated(
            &task_203,
            &task_203_symbols,
            extract_source_two_edge_local_mode_radix_asserted_head(
                &task_203,
                module.clone(),
                &task_203_symbols,
            )
            .is_some(),
            "Task 203",
        );

        let task_205_symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseThreeEdgeModeRadixAssertedHead", SymbolKind::Mode),
                ("InnerThreeEdgeModeRadixAssertedHead", SymbolKind::Mode),
                ("MiddleThreeEdgeModeRadixAssertedHead", SymbolKind::Mode),
                ("OuterThreeEdgeModeRadixAssertedHead", SymbolKind::Mode),
            ],
        );
        let task_205 = mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            [
                mode_definition_with_label(
                    "BaseThreeEdgeModeRadixAssertedHead",
                    "BaseThreeEdgeModeRadixAssertedHeadDef",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition_with_label(
                    "InnerThreeEdgeModeRadixAssertedHead",
                    "InnerThreeEdgeModeRadixAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol("BaseThreeEdgeModeRadixAssertedHead"),
                ),
                mode_definition_with_label(
                    "MiddleThreeEdgeModeRadixAssertedHead",
                    "MiddleThreeEdgeModeRadixAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol("InnerThreeEdgeModeRadixAssertedHead"),
                ),
                mode_definition_with_label(
                    "OuterThreeEdgeModeRadixAssertedHead",
                    "OuterThreeEdgeModeRadixAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol("MiddleThreeEdgeModeRadixAssertedHead"),
                ),
            ],
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("OuterThreeEdgeModeRadixAssertedHead"),
            )],
            exact_three_edge_local_mode_radix_asserted_head_spec(),
        );
        assert_isolated(
            &task_205,
            &task_205_symbols,
            extract_source_three_edge_local_mode_radix_asserted_head(
                &task_205,
                module.clone(),
                &task_205_symbols,
            )
            .is_some(),
            "Task 205",
        );

        assert_task_206_object_route_isolation(source_id, module.clone(), &assert_isolated);
    }

    fn assert_task_206_object_route_isolation<F>(
        source_id: SourceId,
        module: ResolverModuleId,
        assert_isolated: &F,
    ) where
        F: Fn(&SurfaceAst, &SymbolEnv, bool, &str),
    {
        let task_189_symbols = source_local_symbols_env(module.clone(), &[]);
        let task_189 = reserve_then_identifier_type_assertion_theorem_ast(
            source_id,
            vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("object"))],
            exact_reserved_object_identifier_type_assertion_spec(),
        );
        assert_isolated(
            &task_189,
            &task_189_symbols,
            super::extract_source_reserved_object_variable_type_assertion(
                &task_189,
                module.clone(),
                &task_189_symbols,
            )
            .is_some(),
            "Task 189",
        );

        let task_145_symbols = source_local_symbols_env(
            module.clone(),
            &[("LocalObjectModeTypeAssertion", SymbolKind::Mode)],
        );
        let task_145 = mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            [mode_definition_with_label(
                "LocalObjectModeTypeAssertion",
                "LocalObjectModeTypeAssertionDef",
                ReserveTypeShape::Builtin("object"),
            )],
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("LocalObjectModeTypeAssertion"),
            )],
            exact_local_object_mode_identifier_type_assertion_spec(),
        );
        assert_isolated(
            &task_145,
            &task_145_symbols,
            extract_source_local_object_mode_reserved_variable_type_assertion(
                &task_145,
                module.clone(),
                &task_145_symbols,
            )
            .is_some(),
            "Task 145",
        );

        let task_147_symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseObjectModeTypeAssertion", SymbolKind::Mode),
                ("ChainObjectModeTypeAssertion", SymbolKind::Mode),
            ],
        );
        let task_147 = mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            [
                mode_definition_with_label(
                    "BaseObjectModeTypeAssertion",
                    "BaseObjectModeTypeAssertionDef",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition_with_label(
                    "ChainObjectModeTypeAssertion",
                    "ChainObjectModeTypeAssertionDef",
                    ReserveTypeShape::QualifiedSymbol("BaseObjectModeTypeAssertion"),
                ),
            ],
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("ChainObjectModeTypeAssertion"),
            )],
            exact_chained_local_object_mode_identifier_type_assertion_spec(),
        );
        assert_isolated(
            &task_147,
            &task_147_symbols,
            extract_source_chained_local_object_mode_reserved_variable_type_assertion(
                &task_147,
                module.clone(),
                &task_147_symbols,
            )
            .is_some(),
            "Task 147",
        );

        let task_149_symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseTwoEdgeObjectModeTypeAssertion", SymbolKind::Mode),
                ("MiddleTwoEdgeObjectModeTypeAssertion", SymbolKind::Mode),
                ("OuterTwoEdgeObjectModeTypeAssertion", SymbolKind::Mode),
            ],
        );
        let task_149 = mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            [
                mode_definition_with_label(
                    "BaseTwoEdgeObjectModeTypeAssertion",
                    "BaseTwoEdgeObjectModeTypeAssertionDef",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition_with_label(
                    "MiddleTwoEdgeObjectModeTypeAssertion",
                    "MiddleTwoEdgeObjectModeTypeAssertionDef",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeObjectModeTypeAssertion"),
                ),
                mode_definition_with_label(
                    "OuterTwoEdgeObjectModeTypeAssertion",
                    "OuterTwoEdgeObjectModeTypeAssertionDef",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeObjectModeTypeAssertion"),
                ),
            ],
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("OuterTwoEdgeObjectModeTypeAssertion"),
            )],
            exact_two_edge_local_object_mode_identifier_type_assertion_spec(),
        );
        assert_isolated(
            &task_149,
            &task_149_symbols,
            extract_source_two_edge_local_object_mode_reserved_variable_type_assertion(
                &task_149,
                module.clone(),
                &task_149_symbols,
            )
            .is_some(),
            "Task 149",
        );

        let task_151_symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseThreeEdgeObjectModeTypeAssertion", SymbolKind::Mode),
                ("InnerThreeEdgeObjectModeTypeAssertion", SymbolKind::Mode),
                ("MiddleThreeEdgeObjectModeTypeAssertion", SymbolKind::Mode),
                ("OuterThreeEdgeObjectModeTypeAssertion", SymbolKind::Mode),
            ],
        );
        let task_151 = mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            [
                mode_definition_with_label(
                    "BaseThreeEdgeObjectModeTypeAssertion",
                    "BaseThreeEdgeObjectModeTypeAssertionDef",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition_with_label(
                    "InnerThreeEdgeObjectModeTypeAssertion",
                    "InnerThreeEdgeObjectModeTypeAssertionDef",
                    ReserveTypeShape::QualifiedSymbol("BaseThreeEdgeObjectModeTypeAssertion"),
                ),
                mode_definition_with_label(
                    "MiddleThreeEdgeObjectModeTypeAssertion",
                    "MiddleThreeEdgeObjectModeTypeAssertionDef",
                    ReserveTypeShape::QualifiedSymbol("InnerThreeEdgeObjectModeTypeAssertion"),
                ),
                mode_definition_with_label(
                    "OuterThreeEdgeObjectModeTypeAssertion",
                    "OuterThreeEdgeObjectModeTypeAssertionDef",
                    ReserveTypeShape::QualifiedSymbol("MiddleThreeEdgeObjectModeTypeAssertion"),
                ),
            ],
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("OuterThreeEdgeObjectModeTypeAssertion"),
            )],
            exact_three_edge_local_object_mode_identifier_type_assertion_spec(),
        );
        assert_isolated(
            &task_151,
            &task_151_symbols,
            extract_source_three_edge_local_object_mode_reserved_variable_type_assertion(
                &task_151,
                module.clone(),
                &task_151_symbols,
            )
            .is_some(),
            "Task 151",
        );

        let task_196_symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseThreeEdgeObjectModeAssertedHead", SymbolKind::Mode),
                ("InnerThreeEdgeObjectModeAssertedHead", SymbolKind::Mode),
                ("MiddleThreeEdgeObjectModeAssertedHead", SymbolKind::Mode),
                ("OuterThreeEdgeObjectModeAssertedHead", SymbolKind::Mode),
            ],
        );
        let task_196 = mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            [
                mode_definition_with_label(
                    "BaseThreeEdgeObjectModeAssertedHead",
                    "BaseThreeEdgeObjectModeAssertedHeadDef",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition_with_label(
                    "InnerThreeEdgeObjectModeAssertedHead",
                    "InnerThreeEdgeObjectModeAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol("BaseThreeEdgeObjectModeAssertedHead"),
                ),
                mode_definition_with_label(
                    "MiddleThreeEdgeObjectModeAssertedHead",
                    "MiddleThreeEdgeObjectModeAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol("InnerThreeEdgeObjectModeAssertedHead"),
                ),
                mode_definition_with_label(
                    "OuterThreeEdgeObjectModeAssertedHead",
                    "OuterThreeEdgeObjectModeAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol("MiddleThreeEdgeObjectModeAssertedHead"),
                ),
            ],
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("OuterThreeEdgeObjectModeAssertedHead"),
            )],
            IdentifierTypeAssertionTheoremSpec {
                status: None,
                label: "ThreeEdgeLocalObjectModeAssertedHeadPayloadBoundary",
                subject: "x",
                asserted_type: ReserveTypeShape::QualifiedSymbol(
                    "OuterThreeEdgeObjectModeAssertedHead",
                ),
                recovered_label: false,
                negated: false,
            },
        );
        assert_isolated(
            &task_196,
            &task_196_symbols,
            extract_source_three_edge_local_object_mode_asserted_head(
                &task_196,
                module.clone(),
                &task_196_symbols,
            )
            .is_some(),
            "Task 196",
        );

        let task_202_symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseObjectModeRadixAssertedHead", SymbolKind::Mode),
                ("OuterObjectModeRadixAssertedHead", SymbolKind::Mode),
            ],
        );
        let task_202 = mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            [
                mode_definition_with_label(
                    "BaseObjectModeRadixAssertedHead",
                    "BaseObjectModeRadixAssertedHeadDef",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition_with_label(
                    "OuterObjectModeRadixAssertedHead",
                    "OuterObjectModeRadixAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol("BaseObjectModeRadixAssertedHead"),
                ),
            ],
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("OuterObjectModeRadixAssertedHead"),
            )],
            exact_chained_local_object_mode_radix_asserted_head_spec(),
        );
        assert_isolated(
            &task_202,
            &task_202_symbols,
            extract_source_chained_local_object_mode_radix_asserted_head(
                &task_202,
                module.clone(),
                &task_202_symbols,
            )
            .is_some(),
            "Task 202",
        );

        let task_204_symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseTwoEdgeObjectModeRadixAssertedHead", SymbolKind::Mode),
                ("MiddleTwoEdgeObjectModeRadixAssertedHead", SymbolKind::Mode),
                ("OuterTwoEdgeObjectModeRadixAssertedHead", SymbolKind::Mode),
            ],
        );
        let task_204 = mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            [
                mode_definition_with_label(
                    "BaseTwoEdgeObjectModeRadixAssertedHead",
                    "BaseTwoEdgeObjectModeRadixAssertedHeadDef",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition_with_label(
                    "MiddleTwoEdgeObjectModeRadixAssertedHead",
                    "MiddleTwoEdgeObjectModeRadixAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeObjectModeRadixAssertedHead"),
                ),
                mode_definition_with_label(
                    "OuterTwoEdgeObjectModeRadixAssertedHead",
                    "OuterTwoEdgeObjectModeRadixAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeObjectModeRadixAssertedHead"),
                ),
            ],
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("OuterTwoEdgeObjectModeRadixAssertedHead"),
            )],
            exact_two_edge_local_object_mode_radix_asserted_head_spec(),
        );
        assert_isolated(
            &task_204,
            &task_204_symbols,
            extract_source_two_edge_local_object_mode_radix_asserted_head(
                &task_204,
                module,
                &task_204_symbols,
            )
            .is_some(),
            "Task 204",
        );
    }
