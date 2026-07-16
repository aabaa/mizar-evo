    #[test]
    fn source_four_edge_local_object_mode_radix_asserted_head_consumes_immediate_expansion() {
        let source_id = source_id(208);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("four_edge_local_object_mode_radix_asserted_head"),
        );
        let base_mode = "BaseFourEdgeObjectModeRadixAssertedHead";
        let inner_mode = "InnerFourEdgeObjectModeRadixAssertedHead";
        let middle_mode = "MiddleFourEdgeObjectModeRadixAssertedHead";
        let outer_mode = "OuterFourEdgeObjectModeRadixAssertedHead";
        let too_deep_mode = "TooDeepFourEdgeObjectModeRadixAssertedHead";
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
                ("SixthFourEdgeObjectModeRadixAssertedHead", SymbolKind::Mode),
                ("OtherFourEdgeObjectModeRadixAssertedHead", SymbolKind::Mode),
            ],
        );
        let theorem = exact_four_edge_local_object_mode_radix_asserted_head_spec();
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    base_mode,
                    "BaseFourEdgeObjectModeRadixAssertedHeadDef",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition_with_label(
                    inner_mode,
                    "InnerFourEdgeObjectModeRadixAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(base_mode),
                ),
                mode_definition_with_label(
                    middle_mode,
                    "MiddleFourEdgeObjectModeRadixAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(inner_mode),
                ),
                mode_definition_with_label(
                    outer_mode,
                    "OuterFourEdgeObjectModeRadixAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(middle_mode),
                ),
                mode_definition_with_label(
                    too_deep_mode,
                    "TooDeepFourEdgeObjectModeRadixAssertedHeadDef",
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
        assert!(
            extract_source_four_edge_local_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "Task 208 must reject the Task 207 owner route"
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

        for rejected in [
            extract_source_four_edge_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            super::extract_source_four_edge_local_mode_asserted_head(
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
            super::extract_source_four_edge_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_object_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
        ] {
            assert!(rejected.is_none());
        }

        let payload = extract_source_four_edge_local_object_mode_radix_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact four-edge immediate-radix asserted head should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 5);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(payload.subject_lookup_ordinal, 1);
        let source_binding = &payload.reserve.bridge.bindings()[0];
        assert_eq!(source_binding.type_spelling, too_deep_mode);
        assert_eq!(payload.asserted_type.spelling, outer_mode);
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
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(too_deep_mode))
            .map(|(_, expansion)| expansion)
            .expect("outer binding expansion should exist");
        assert_eq!(outer_expansion.radix.spelling, outer_mode);
        assert_eq!(outer_expansion.radix.head, payload.asserted_type.head);

        let output = source_four_edge_local_object_mode_radix_asserted_head_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact Task 208 source should reach the checker");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 208 checker payload invariants should hold");
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.payload.reserve.mode_expansions.len(), 5);
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert_eq!(output.subject_result_input.spelling, too_deep_mode);
        assert_eq!(output.asserted_type_input.spelling, outer_mode);
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
            source_four_edge_local_object_mode_radix_asserted_head_output(
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
                    TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_RADIX_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
                ),
                vec![
                    TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_RADIX_ASSERTED_HEAD_INVALID_PAYLOAD_KEY
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
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(too_deep_mode))
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
            .expect("corrupting cloned Task 208 outputs must not mutate the exact output");

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
                                .any(|(index, value)| order[..index].contains(value))
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
            wrong_label[index].label = Some("WrongFourEdgeObjectModeRadixAssertedHeadDef");
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
            wrong_spelling[index].pattern = "OtherFourEdgeObjectModeRadixAssertedHead";
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
                ReserveTypeShape::QualifiedSymbol("OtherFourEdgeObjectModeRadixAssertedHead")
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
            let mut recovered = exact_modes();
            recovered[index].recovered = true;
            assert_extraction_gap(
                mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    recovered,
                    reserve(),
                    theorem,
                ),
                &format!("recovered four-edge definition {index}"),
            );
            let mut contextual = exact_modes();
            contextual[index].local_context = true;
            assert_extraction_gap(
                mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    contextual,
                    reserve(),
                    theorem,
                ),
                &format!("contextual four-edge definition {index}"),
            );
            let mut parameterized = exact_modes();
            parameterized[index].parameterized_pattern = true;
            assert_extraction_gap(
                mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    parameterized,
                    reserve(),
                    theorem,
                ),
                &format!("parameterized four-edge definition {index}"),
            );
            let expected_radix = match index {
                0 => "object",
                1 => base_mode,
                2 => inner_mode,
                3 => middle_mode,
                _ => outer_mode,
            };
            let mut arguments = exact_modes();
            arguments[index].rhs_shape = ReserveTypeShape::QualifiedSymbolWithArgs(expected_radix);
            assert_extraction_gap(
                mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    arguments,
                    reserve(),
                    theorem,
                ),
                &format!("argument-bearing four-edge definition {index}"),
            );
            let mut attributes = exact_modes();
            attributes[index].rhs_shape = if index == 0 {
                ReserveTypeShape::AttributedObject
            } else {
                ReserveTypeShape::AttributedQualifiedSymbol(expected_radix)
            };
            assert_extraction_gap(
                mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    attributes,
                    reserve(),
                    theorem,
                ),
                &format!("attributed four-edge definition {index}"),
            );
        }

        let mut deeper_modes = exact_modes();
        deeper_modes.push(mode_definition_with_label(
            "SixthFourEdgeObjectModeRadixAssertedHead",
            "SixthFourEdgeObjectModeRadixAssertedHeadDef",
            ReserveTypeShape::QualifiedSymbol(too_deep_mode),
        ));
        assert_extraction_gap(
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                deeper_modes,
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbol("SixthFourEdgeObjectModeRadixAssertedHead"),
                )],
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(too_deep_mode),
                    ..theorem
                },
            ),
            "connected deeper immediate-radix chain",
        );
        let mut four_edge_source_near_misses = Vec::new();
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
            four_edge_source_near_misses.push(
                mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    exact_modes(),
                    vec![reserve_item(vec!["x"], bad_reserve)],
                    theorem,
                ),
            );
        }
        four_edge_source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
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
            four_edge_source_near_misses.push(
                mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    exact_modes(),
                    reserve(),
                    near_miss_theorem,
                ),
            );
        }
        four_edge_source_near_misses.push(
            modes_then_empty_definition_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                theorem,
            ),
        );
        for (index, near_miss) in four_edge_source_near_misses.into_iter().enumerate() {
            assert_extraction_gap(near_miss, &format!("four-edge source near miss {index}"));
        }
        for asserted_type in [
            ReserveTypeShape::QualifiedSymbol(too_deep_mode),
            ReserveTypeShape::QualifiedSymbol(middle_mode),
            ReserveTypeShape::QualifiedSymbol(inner_mode),
            ReserveTypeShape::QualifiedSymbol(base_mode),
            ReserveTypeShape::Builtin("object"),
            ReserveTypeShape::Builtin("set"),
            ReserveTypeShape::QualifiedSymbol("OtherFourEdgeObjectModeRadixAssertedHead"),
            ReserveTypeShape::QualifiedSymbolWithArgs(outer_mode),
            ReserveTypeShape::AttributedQualifiedSymbol(outer_mode),
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
                (too_deep_mode, SymbolKind::Mode),
            ],
            &[(
                "UnrelatedFourEdgeObjectModeRadixAssertedHead",
                SymbolKind::Mode,
            )],
        );
        assert!(
            extract_source_four_edge_local_object_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &unrelated_import,
            )
            .is_some()
        );
        for missing_index in 0..5 {
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
        assert_task_208_route_isolation(source_id, module);
    }

    #[test]
    fn source_four_edge_local_mode_radix_asserted_head_consumes_immediate_expansion() {
        let source_id = source_id(207);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("four_edge_local_mode_radix_asserted_head"),
        );
        let base_mode = "BaseFourEdgeModeRadixAssertedHead";
        let inner_mode = "InnerFourEdgeModeRadixAssertedHead";
        let middle_mode = "MiddleFourEdgeModeRadixAssertedHead";
        let outer_mode = "OuterFourEdgeModeRadixAssertedHead";
        let too_deep_mode = "TooDeepFourEdgeModeRadixAssertedHead";
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
                ("SixthFourEdgeModeRadixAssertedHead", SymbolKind::Mode),
                ("OtherFourEdgeModeRadixAssertedHead", SymbolKind::Mode),
            ],
        );
        let theorem = exact_four_edge_local_mode_radix_asserted_head_spec();
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    base_mode,
                    "BaseFourEdgeModeRadixAssertedHeadDef",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition_with_label(
                    inner_mode,
                    "InnerFourEdgeModeRadixAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(base_mode),
                ),
                mode_definition_with_label(
                    middle_mode,
                    "MiddleFourEdgeModeRadixAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(inner_mode),
                ),
                mode_definition_with_label(
                    outer_mode,
                    "OuterFourEdgeModeRadixAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(middle_mode),
                ),
                mode_definition_with_label(
                    too_deep_mode,
                    "TooDeepFourEdgeModeRadixAssertedHeadDef",
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

        for rejected in [
            extract_source_four_edge_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            ),
            super::extract_source_four_edge_local_mode_asserted_head(
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
            super::extract_source_four_edge_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_object_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
        ] {
            assert!(rejected.is_none());
        }

        let payload = extract_source_four_edge_local_mode_radix_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact four-edge immediate-radix asserted head should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 5);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(payload.subject_lookup_ordinal, 1);
        let source_binding = &payload.reserve.bridge.bindings()[0];
        assert_eq!(source_binding.type_spelling, too_deep_mode);
        assert_eq!(payload.asserted_type.spelling, outer_mode);
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
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(too_deep_mode))
            .map(|(_, expansion)| expansion)
            .expect("outer binding expansion should exist");
        assert_eq!(outer_expansion.radix.spelling, outer_mode);
        assert_eq!(outer_expansion.radix.head, payload.asserted_type.head);

        let output = source_four_edge_local_mode_radix_asserted_head_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact Task 207 source should reach the checker");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 207 checker payload invariants should hold");
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.payload.reserve.mode_expansions.len(), 5);
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert_eq!(output.subject_result_input.spelling, too_deep_mode);
        assert_eq!(output.asserted_type_input.spelling, outer_mode);
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
            .expect("base set terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("one normalized builtin-set identity should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_ne!(normalized.head, TypeHeadRef::BuiltinObject);
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
            source_four_edge_local_mode_radix_asserted_head_output(&exact, module.clone(), &symbols)
                .unwrap()
        };
        let assert_invalid_output = |invalid| {
            let invalid_result =
                assert_source_reserved_variable_type_assertion_output(&invalid).map(|()| invalid);
            assert_eq!(
                source_reserved_variable_type_assertion_result_detail_keys(
                    invalid_result,
                    TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_RADIX_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
                ),
                vec![
                    TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_RADIX_ASSERTED_HEAD_INVALID_PAYLOAD_KEY
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
        invalid.subject_result_input.head = TypeHeadInput::BuiltinObject;
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
        invalid.asserted_type_input.head = TypeHeadInput::BuiltinObject;
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
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(too_deep_mode))
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
        middle_expansion.radix.spelling = "object".to_owned();
        middle_expansion.radix.head = TypeHeadInput::BuiltinObject;
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
        base_expansion.radix.spelling = "object".to_owned();
        base_expansion.radix.head = TypeHeadInput::BuiltinObject;
        corruptions.push(invalid);
        for invalid in corruptions {
            assert_invalid_output(invalid);
        }
        assert_source_reserved_variable_type_assertion_output(&exact_output())
            .expect("corrupting cloned Task 207 outputs must not mutate the exact output");

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
                                .any(|(index, value)| order[..index].contains(value))
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
            wrong_label[index].label = Some("WrongFourEdgeModeRadixAssertedHeadDef");
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
            wrong_spelling[index].pattern = "OtherFourEdgeModeRadixAssertedHead";
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
                ReserveTypeShape::Builtin("object")
            } else {
                ReserveTypeShape::QualifiedSymbol("OtherFourEdgeModeRadixAssertedHead")
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
            let mut recovered = exact_modes();
            recovered[index].recovered = true;
            assert_extraction_gap(
                mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    recovered,
                    reserve(),
                    theorem,
                ),
                &format!("recovered four-edge definition {index}"),
            );
            let mut contextual = exact_modes();
            contextual[index].local_context = true;
            assert_extraction_gap(
                mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    contextual,
                    reserve(),
                    theorem,
                ),
                &format!("contextual four-edge definition {index}"),
            );
            let mut parameterized = exact_modes();
            parameterized[index].parameterized_pattern = true;
            assert_extraction_gap(
                mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    parameterized,
                    reserve(),
                    theorem,
                ),
                &format!("parameterized four-edge definition {index}"),
            );
            let expected_radix = match index {
                0 => "set",
                1 => base_mode,
                2 => inner_mode,
                3 => middle_mode,
                _ => outer_mode,
            };
            let mut arguments = exact_modes();
            arguments[index].rhs_shape = ReserveTypeShape::QualifiedSymbolWithArgs(expected_radix);
            assert_extraction_gap(
                mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    arguments,
                    reserve(),
                    theorem,
                ),
                &format!("argument-bearing four-edge definition {index}"),
            );
            let mut attributes = exact_modes();
            attributes[index].rhs_shape = if index == 0 {
                ReserveTypeShape::AttributedSet
            } else {
                ReserveTypeShape::AttributedQualifiedSymbol(expected_radix)
            };
            assert_extraction_gap(
                mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    attributes,
                    reserve(),
                    theorem,
                ),
                &format!("attributed four-edge definition {index}"),
            );
        }

        let mut deeper_modes = exact_modes();
        deeper_modes.push(mode_definition_with_label(
            "SixthFourEdgeModeRadixAssertedHead",
            "SixthFourEdgeModeRadixAssertedHeadDef",
            ReserveTypeShape::QualifiedSymbol(too_deep_mode),
        ));
        assert_extraction_gap(
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                deeper_modes,
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbol("SixthFourEdgeModeRadixAssertedHead"),
                )],
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(too_deep_mode),
                    ..theorem
                },
            ),
            "connected deeper immediate-radix chain",
        );
        let mut four_edge_source_near_misses = Vec::new();
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
            four_edge_source_near_misses.push(
                mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    exact_modes(),
                    vec![reserve_item(vec!["x"], bad_reserve)],
                    theorem,
                ),
            );
        }
        four_edge_source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
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
            four_edge_source_near_misses.push(
                mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    exact_modes(),
                    reserve(),
                    near_miss_theorem,
                ),
            );
        }
        four_edge_source_near_misses.push(
            modes_then_empty_definition_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                theorem,
            ),
        );
        for (index, near_miss) in four_edge_source_near_misses.into_iter().enumerate() {
            assert_extraction_gap(near_miss, &format!("four-edge source near miss {index}"));
        }
        for asserted_type in [
            ReserveTypeShape::QualifiedSymbol(too_deep_mode),
            ReserveTypeShape::QualifiedSymbol(middle_mode),
            ReserveTypeShape::QualifiedSymbol(inner_mode),
            ReserveTypeShape::QualifiedSymbol(base_mode),
            ReserveTypeShape::Builtin("set"),
            ReserveTypeShape::Builtin("object"),
            ReserveTypeShape::QualifiedSymbol("OtherFourEdgeModeRadixAssertedHead"),
            ReserveTypeShape::QualifiedSymbolWithArgs(outer_mode),
            ReserveTypeShape::AttributedQualifiedSymbol(outer_mode),
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
                (too_deep_mode, SymbolKind::Mode),
            ],
            &[("UnrelatedFourEdgeModeRadixAssertedHead", SymbolKind::Mode)],
        );
        assert!(
            extract_source_four_edge_local_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &unrelated_import,
            )
            .is_some()
        );
        for missing_index in 0..5 {
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
        assert_task_207_route_isolation(source_id, module);
    }

    fn assert_task_208_route_isolation(source_id: SourceId, module: ResolverModuleId) {
        let assert_isolated =
            |ast: &SurfaceAst, symbols: &SymbolEnv, owner_extracts: bool, task: &str| {
                assert!(owner_extracts, "{task} owner route should extract");
                assert!(
                    extract_source_four_edge_local_object_mode_radix_asserted_head(
                        ast,
                        module.clone(),
                        symbols,
                    )
                    .is_none(),
                    "Task 208 must reject {task}"
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

        let build_five_mode_source =
            |names: [&'static str; 5],
             labels: [&'static str; 5],
             terminal: &'static str,
             theorem: IdentifierTypeAssertionTheoremSpec<'static>| {
                let symbols = source_local_symbols_env(
                    module.clone(),
                    &names.map(|spelling| (spelling, SymbolKind::Mode)),
                );
                let ast = mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    [
                        mode_definition_with_label(
                            names[0],
                            labels[0],
                            ReserveTypeShape::Builtin(terminal),
                        ),
                        mode_definition_with_label(
                            names[1],
                            labels[1],
                            ReserveTypeShape::QualifiedSymbol(names[0]),
                        ),
                        mode_definition_with_label(
                            names[2],
                            labels[2],
                            ReserveTypeShape::QualifiedSymbol(names[1]),
                        ),
                        mode_definition_with_label(
                            names[3],
                            labels[3],
                            ReserveTypeShape::QualifiedSymbol(names[2]),
                        ),
                        mode_definition_with_label(
                            names[4],
                            labels[4],
                            ReserveTypeShape::QualifiedSymbol(names[3]),
                        ),
                    ],
                    vec![reserve_item(
                        vec!["x"],
                        ReserveTypeShape::QualifiedSymbol(names[4]),
                    )],
                    theorem,
                );
                (ast, symbols)
            };

        let (task_152, task_152_symbols) = build_five_mode_source(
            [
                "BaseFourEdgeModeTypeAssertion",
                "InnerFourEdgeModeTypeAssertion",
                "MiddleFourEdgeModeTypeAssertion",
                "OuterFourEdgeModeTypeAssertion",
                "TooDeepFourEdgeModeTypeAssertion",
            ],
            [
                "BaseFourEdgeModeTypeAssertionDef",
                "InnerFourEdgeModeTypeAssertionDef",
                "MiddleFourEdgeModeTypeAssertionDef",
                "OuterFourEdgeModeTypeAssertionDef",
                "TooDeepFourEdgeModeTypeAssertionDef",
            ],
            "set",
            exact_four_edge_local_mode_identifier_type_assertion_spec(),
        );
        assert_isolated(
            &task_152,
            &task_152_symbols,
            extract_source_four_edge_local_mode_reserved_variable_type_assertion(
                &task_152,
                module.clone(),
                &task_152_symbols,
            )
            .is_some(),
            "Task 152",
        );

        let (task_197, task_197_symbols) = build_five_mode_source(
            [
                "BaseFourEdgeModeAssertedHead",
                "InnerFourEdgeModeAssertedHead",
                "MiddleFourEdgeModeAssertedHead",
                "OuterFourEdgeModeAssertedHead",
                "TooDeepFourEdgeModeAssertedHead",
            ],
            [
                "BaseFourEdgeModeAssertedHeadDef",
                "InnerFourEdgeModeAssertedHeadDef",
                "MiddleFourEdgeModeAssertedHeadDef",
                "OuterFourEdgeModeAssertedHeadDef",
                "TooDeepFourEdgeModeAssertedHeadDef",
            ],
            "set",
            IdentifierTypeAssertionTheoremSpec {
                status: None,
                label: "FourEdgeLocalModeAssertedHeadPayloadBoundary",
                subject: "x",
                asserted_type: ReserveTypeShape::QualifiedSymbol("TooDeepFourEdgeModeAssertedHead"),
                recovered_label: false,
                negated: false,
            },
        );
        assert_isolated(
            &task_197,
            &task_197_symbols,
            super::extract_source_four_edge_local_mode_asserted_head(
                &task_197,
                module.clone(),
                &task_197_symbols,
            )
            .is_some(),
            "Task 197",
        );

        let task_205_names = [
            "BaseThreeEdgeModeRadixAssertedHead",
            "InnerThreeEdgeModeRadixAssertedHead",
            "MiddleThreeEdgeModeRadixAssertedHead",
            "OuterThreeEdgeModeRadixAssertedHead",
        ];
        let task_205_symbols = source_local_symbols_env(
            module.clone(),
            &task_205_names.map(|spelling| (spelling, SymbolKind::Mode)),
        );
        let task_205 = mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            [
                mode_definition_with_label(
                    task_205_names[0],
                    "BaseThreeEdgeModeRadixAssertedHeadDef",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition_with_label(
                    task_205_names[1],
                    "InnerThreeEdgeModeRadixAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(task_205_names[0]),
                ),
                mode_definition_with_label(
                    task_205_names[2],
                    "MiddleThreeEdgeModeRadixAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(task_205_names[1]),
                ),
                mode_definition_with_label(
                    task_205_names[3],
                    "OuterThreeEdgeModeRadixAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(task_205_names[2]),
                ),
            ],
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol(task_205_names[3]),
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

        let task_207_names = [
            "BaseFourEdgeModeRadixAssertedHead",
            "InnerFourEdgeModeRadixAssertedHead",
            "MiddleFourEdgeModeRadixAssertedHead",
            "OuterFourEdgeModeRadixAssertedHead",
            "TooDeepFourEdgeModeRadixAssertedHead",
        ];
        let task_207_symbols = source_local_symbols_env(
            module.clone(),
            &task_207_names.map(|spelling| (spelling, SymbolKind::Mode)),
        );
        let task_207 = mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            [
                mode_definition_with_label(
                    task_207_names[0],
                    "BaseFourEdgeModeRadixAssertedHeadDef",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition_with_label(
                    task_207_names[1],
                    "InnerFourEdgeModeRadixAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(task_207_names[0]),
                ),
                mode_definition_with_label(
                    task_207_names[2],
                    "MiddleFourEdgeModeRadixAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(task_207_names[1]),
                ),
                mode_definition_with_label(
                    task_207_names[3],
                    "OuterFourEdgeModeRadixAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(task_207_names[2]),
                ),
                mode_definition_with_label(
                    task_207_names[4],
                    "TooDeepFourEdgeModeRadixAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(task_207_names[3]),
                ),
            ],
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol(task_207_names[4]),
            )],
            exact_four_edge_local_mode_radix_asserted_head_spec(),
        );
        assert_isolated(
            &task_207,
            &task_207_symbols,
            extract_source_four_edge_local_mode_radix_asserted_head(
                &task_207,
                module.clone(),
                &task_207_symbols,
            )
            .is_some(),
            "Task 207",
        );

        assert_task_208_object_route_isolation(source_id, module.clone(), &assert_isolated);
    }

    fn assert_task_208_object_route_isolation<F>(
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

        let build_five_object_mode_source =
            |names: [&'static str; 5],
             labels: [&'static str; 5],
             theorem: IdentifierTypeAssertionTheoremSpec<'static>| {
                let symbols = source_local_symbols_env(
                    module.clone(),
                    &names.map(|spelling| (spelling, SymbolKind::Mode)),
                );
                let ast = mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    [
                        mode_definition_with_label(
                            names[0],
                            labels[0],
                            ReserveTypeShape::Builtin("object"),
                        ),
                        mode_definition_with_label(
                            names[1],
                            labels[1],
                            ReserveTypeShape::QualifiedSymbol(names[0]),
                        ),
                        mode_definition_with_label(
                            names[2],
                            labels[2],
                            ReserveTypeShape::QualifiedSymbol(names[1]),
                        ),
                        mode_definition_with_label(
                            names[3],
                            labels[3],
                            ReserveTypeShape::QualifiedSymbol(names[2]),
                        ),
                        mode_definition_with_label(
                            names[4],
                            labels[4],
                            ReserveTypeShape::QualifiedSymbol(names[3]),
                        ),
                    ],
                    vec![reserve_item(
                        vec!["x"],
                        ReserveTypeShape::QualifiedSymbol(names[4]),
                    )],
                    theorem,
                );
                (ast, symbols)
            };

        let (task_153, task_153_symbols) = build_five_object_mode_source(
            [
                "BaseFourEdgeObjectModeTypeAssertion",
                "InnerFourEdgeObjectModeTypeAssertion",
                "MiddleFourEdgeObjectModeTypeAssertion",
                "OuterFourEdgeObjectModeTypeAssertion",
                "TooDeepFourEdgeObjectModeTypeAssertion",
            ],
            [
                "BaseFourEdgeObjectModeTypeAssertionDef",
                "InnerFourEdgeObjectModeTypeAssertionDef",
                "MiddleFourEdgeObjectModeTypeAssertionDef",
                "OuterFourEdgeObjectModeTypeAssertionDef",
                "TooDeepFourEdgeObjectModeTypeAssertionDef",
            ],
            exact_four_edge_local_object_mode_identifier_type_assertion_spec(),
        );
        assert_isolated(
            &task_153,
            &task_153_symbols,
            extract_source_four_edge_local_object_mode_reserved_variable_type_assertion(
                &task_153,
                module.clone(),
                &task_153_symbols,
            )
            .is_some(),
            "Task 153",
        );

        let (task_198, task_198_symbols) = build_five_object_mode_source(
            [
                "BaseFourEdgeObjectModeAssertedHead",
                "InnerFourEdgeObjectModeAssertedHead",
                "MiddleFourEdgeObjectModeAssertedHead",
                "OuterFourEdgeObjectModeAssertedHead",
                "TooDeepFourEdgeObjectModeAssertedHead",
            ],
            [
                "BaseFourEdgeObjectModeAssertedHeadDef",
                "InnerFourEdgeObjectModeAssertedHeadDef",
                "MiddleFourEdgeObjectModeAssertedHeadDef",
                "OuterFourEdgeObjectModeAssertedHeadDef",
                "TooDeepFourEdgeObjectModeAssertedHeadDef",
            ],
            IdentifierTypeAssertionTheoremSpec {
                status: None,
                label: "FourEdgeLocalObjectModeAssertedHeadPayloadBoundary",
                subject: "x",
                asserted_type: ReserveTypeShape::QualifiedSymbol(
                    "TooDeepFourEdgeObjectModeAssertedHead",
                ),
                recovered_label: false,
                negated: false,
            },
        );
        assert_isolated(
            &task_198,
            &task_198_symbols,
            super::extract_source_four_edge_local_object_mode_asserted_head(
                &task_198,
                module.clone(),
                &task_198_symbols,
            )
            .is_some(),
            "Task 198",
        );

        let task_206_names = [
            "BaseThreeEdgeObjectModeRadixAssertedHead",
            "InnerThreeEdgeObjectModeRadixAssertedHead",
            "MiddleThreeEdgeObjectModeRadixAssertedHead",
            "OuterThreeEdgeObjectModeRadixAssertedHead",
        ];
        let task_206_symbols = source_local_symbols_env(
            module.clone(),
            &task_206_names.map(|spelling| (spelling, SymbolKind::Mode)),
        );
        let task_206 = mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            [
                mode_definition_with_label(
                    task_206_names[0],
                    "BaseThreeEdgeObjectModeRadixAssertedHeadDef",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition_with_label(
                    task_206_names[1],
                    "InnerThreeEdgeObjectModeRadixAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(task_206_names[0]),
                ),
                mode_definition_with_label(
                    task_206_names[2],
                    "MiddleThreeEdgeObjectModeRadixAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(task_206_names[1]),
                ),
                mode_definition_with_label(
                    task_206_names[3],
                    "OuterThreeEdgeObjectModeRadixAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(task_206_names[2]),
                ),
            ],
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol(task_206_names[3]),
            )],
            exact_three_edge_local_object_mode_radix_asserted_head_spec(),
        );
        assert_isolated(
            &task_206,
            &task_206_symbols,
            extract_source_three_edge_local_object_mode_radix_asserted_head(
                &task_206,
                module.clone(),
                &task_206_symbols,
            )
            .is_some(),
            "Task 206",
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

    fn assert_task_207_route_isolation(source_id: SourceId, module: ResolverModuleId) {
        let assert_isolated =
            |ast: &SurfaceAst, symbols: &SymbolEnv, owner_extracts: bool, task: &str| {
                assert!(owner_extracts, "{task} owner route should extract");
                assert!(
                    extract_source_four_edge_local_mode_radix_asserted_head(
                        ast,
                        module.clone(),
                        symbols,
                    )
                    .is_none(),
                    "Task 207 must reject {task}"
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

        let build_five_mode_source =
            |names: [&'static str; 5],
             labels: [&'static str; 5],
             terminal: &'static str,
             theorem: IdentifierTypeAssertionTheoremSpec<'static>| {
                let symbols = source_local_symbols_env(
                    module.clone(),
                    &names.map(|spelling| (spelling, SymbolKind::Mode)),
                );
                let ast = mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    [
                        mode_definition_with_label(
                            names[0],
                            labels[0],
                            ReserveTypeShape::Builtin(terminal),
                        ),
                        mode_definition_with_label(
                            names[1],
                            labels[1],
                            ReserveTypeShape::QualifiedSymbol(names[0]),
                        ),
                        mode_definition_with_label(
                            names[2],
                            labels[2],
                            ReserveTypeShape::QualifiedSymbol(names[1]),
                        ),
                        mode_definition_with_label(
                            names[3],
                            labels[3],
                            ReserveTypeShape::QualifiedSymbol(names[2]),
                        ),
                        mode_definition_with_label(
                            names[4],
                            labels[4],
                            ReserveTypeShape::QualifiedSymbol(names[3]),
                        ),
                    ],
                    vec![reserve_item(
                        vec!["x"],
                        ReserveTypeShape::QualifiedSymbol(names[4]),
                    )],
                    theorem,
                );
                (ast, symbols)
            };

        let (task_152, task_152_symbols) = build_five_mode_source(
            [
                "BaseFourEdgeModeTypeAssertion",
                "InnerFourEdgeModeTypeAssertion",
                "MiddleFourEdgeModeTypeAssertion",
                "OuterFourEdgeModeTypeAssertion",
                "TooDeepFourEdgeModeTypeAssertion",
            ],
            [
                "BaseFourEdgeModeTypeAssertionDef",
                "InnerFourEdgeModeTypeAssertionDef",
                "MiddleFourEdgeModeTypeAssertionDef",
                "OuterFourEdgeModeTypeAssertionDef",
                "TooDeepFourEdgeModeTypeAssertionDef",
            ],
            "set",
            exact_four_edge_local_mode_identifier_type_assertion_spec(),
        );
        assert_isolated(
            &task_152,
            &task_152_symbols,
            extract_source_four_edge_local_mode_reserved_variable_type_assertion(
                &task_152,
                module.clone(),
                &task_152_symbols,
            )
            .is_some(),
            "Task 152",
        );

        let (task_197, task_197_symbols) = build_five_mode_source(
            [
                "BaseFourEdgeModeAssertedHead",
                "InnerFourEdgeModeAssertedHead",
                "MiddleFourEdgeModeAssertedHead",
                "OuterFourEdgeModeAssertedHead",
                "TooDeepFourEdgeModeAssertedHead",
            ],
            [
                "BaseFourEdgeModeAssertedHeadDef",
                "InnerFourEdgeModeAssertedHeadDef",
                "MiddleFourEdgeModeAssertedHeadDef",
                "OuterFourEdgeModeAssertedHeadDef",
                "TooDeepFourEdgeModeAssertedHeadDef",
            ],
            "set",
            IdentifierTypeAssertionTheoremSpec {
                status: None,
                label: "FourEdgeLocalModeAssertedHeadPayloadBoundary",
                subject: "x",
                asserted_type: ReserveTypeShape::QualifiedSymbol("TooDeepFourEdgeModeAssertedHead"),
                recovered_label: false,
                negated: false,
            },
        );
        assert_isolated(
            &task_197,
            &task_197_symbols,
            super::extract_source_four_edge_local_mode_asserted_head(
                &task_197,
                module.clone(),
                &task_197_symbols,
            )
            .is_some(),
            "Task 197",
        );

        let task_205_names = [
            "BaseThreeEdgeModeRadixAssertedHead",
            "InnerThreeEdgeModeRadixAssertedHead",
            "MiddleThreeEdgeModeRadixAssertedHead",
            "OuterThreeEdgeModeRadixAssertedHead",
        ];
        let task_205_symbols = source_local_symbols_env(
            module.clone(),
            &task_205_names.map(|spelling| (spelling, SymbolKind::Mode)),
        );
        let task_205 = mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            [
                mode_definition_with_label(
                    task_205_names[0],
                    "BaseThreeEdgeModeRadixAssertedHeadDef",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition_with_label(
                    task_205_names[1],
                    "InnerThreeEdgeModeRadixAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(task_205_names[0]),
                ),
                mode_definition_with_label(
                    task_205_names[2],
                    "MiddleThreeEdgeModeRadixAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(task_205_names[1]),
                ),
                mode_definition_with_label(
                    task_205_names[3],
                    "OuterThreeEdgeModeRadixAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(task_205_names[2]),
                ),
            ],
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol(task_205_names[3]),
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

        assert_task_207_object_route_isolation(source_id, module.clone(), &assert_isolated);
    }

    fn assert_task_207_object_route_isolation<F>(
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

        let build_five_object_mode_source =
            |names: [&'static str; 5],
             labels: [&'static str; 5],
             theorem: IdentifierTypeAssertionTheoremSpec<'static>| {
                let symbols = source_local_symbols_env(
                    module.clone(),
                    &names.map(|spelling| (spelling, SymbolKind::Mode)),
                );
                let ast = mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    [
                        mode_definition_with_label(
                            names[0],
                            labels[0],
                            ReserveTypeShape::Builtin("object"),
                        ),
                        mode_definition_with_label(
                            names[1],
                            labels[1],
                            ReserveTypeShape::QualifiedSymbol(names[0]),
                        ),
                        mode_definition_with_label(
                            names[2],
                            labels[2],
                            ReserveTypeShape::QualifiedSymbol(names[1]),
                        ),
                        mode_definition_with_label(
                            names[3],
                            labels[3],
                            ReserveTypeShape::QualifiedSymbol(names[2]),
                        ),
                        mode_definition_with_label(
                            names[4],
                            labels[4],
                            ReserveTypeShape::QualifiedSymbol(names[3]),
                        ),
                    ],
                    vec![reserve_item(
                        vec!["x"],
                        ReserveTypeShape::QualifiedSymbol(names[4]),
                    )],
                    theorem,
                );
                (ast, symbols)
            };

        let (task_153, task_153_symbols) = build_five_object_mode_source(
            [
                "BaseFourEdgeObjectModeTypeAssertion",
                "InnerFourEdgeObjectModeTypeAssertion",
                "MiddleFourEdgeObjectModeTypeAssertion",
                "OuterFourEdgeObjectModeTypeAssertion",
                "TooDeepFourEdgeObjectModeTypeAssertion",
            ],
            [
                "BaseFourEdgeObjectModeTypeAssertionDef",
                "InnerFourEdgeObjectModeTypeAssertionDef",
                "MiddleFourEdgeObjectModeTypeAssertionDef",
                "OuterFourEdgeObjectModeTypeAssertionDef",
                "TooDeepFourEdgeObjectModeTypeAssertionDef",
            ],
            exact_four_edge_local_object_mode_identifier_type_assertion_spec(),
        );
        assert_isolated(
            &task_153,
            &task_153_symbols,
            extract_source_four_edge_local_object_mode_reserved_variable_type_assertion(
                &task_153,
                module.clone(),
                &task_153_symbols,
            )
            .is_some(),
            "Task 153",
        );

        let (task_198, task_198_symbols) = build_five_object_mode_source(
            [
                "BaseFourEdgeObjectModeAssertedHead",
                "InnerFourEdgeObjectModeAssertedHead",
                "MiddleFourEdgeObjectModeAssertedHead",
                "OuterFourEdgeObjectModeAssertedHead",
                "TooDeepFourEdgeObjectModeAssertedHead",
            ],
            [
                "BaseFourEdgeObjectModeAssertedHeadDef",
                "InnerFourEdgeObjectModeAssertedHeadDef",
                "MiddleFourEdgeObjectModeAssertedHeadDef",
                "OuterFourEdgeObjectModeAssertedHeadDef",
                "TooDeepFourEdgeObjectModeAssertedHeadDef",
            ],
            IdentifierTypeAssertionTheoremSpec {
                status: None,
                label: "FourEdgeLocalObjectModeAssertedHeadPayloadBoundary",
                subject: "x",
                asserted_type: ReserveTypeShape::QualifiedSymbol(
                    "TooDeepFourEdgeObjectModeAssertedHead",
                ),
                recovered_label: false,
                negated: false,
            },
        );
        assert_isolated(
            &task_198,
            &task_198_symbols,
            super::extract_source_four_edge_local_object_mode_asserted_head(
                &task_198,
                module.clone(),
                &task_198_symbols,
            )
            .is_some(),
            "Task 198",
        );

        let task_206_names = [
            "BaseThreeEdgeObjectModeRadixAssertedHead",
            "InnerThreeEdgeObjectModeRadixAssertedHead",
            "MiddleThreeEdgeObjectModeRadixAssertedHead",
            "OuterThreeEdgeObjectModeRadixAssertedHead",
        ];
        let task_206_symbols = source_local_symbols_env(
            module.clone(),
            &task_206_names.map(|spelling| (spelling, SymbolKind::Mode)),
        );
        let task_206 = mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            [
                mode_definition_with_label(
                    task_206_names[0],
                    "BaseThreeEdgeObjectModeRadixAssertedHeadDef",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition_with_label(
                    task_206_names[1],
                    "InnerThreeEdgeObjectModeRadixAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(task_206_names[0]),
                ),
                mode_definition_with_label(
                    task_206_names[2],
                    "MiddleThreeEdgeObjectModeRadixAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(task_206_names[1]),
                ),
                mode_definition_with_label(
                    task_206_names[3],
                    "OuterThreeEdgeObjectModeRadixAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(task_206_names[2]),
                ),
            ],
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol(task_206_names[3]),
            )],
            exact_three_edge_local_object_mode_radix_asserted_head_spec(),
        );
        assert_isolated(
            &task_206,
            &task_206_symbols,
            extract_source_three_edge_local_object_mode_radix_asserted_head(
                &task_206,
                module.clone(),
                &task_206_symbols,
            )
            .is_some(),
            "Task 206",
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
