    #[test]
    fn source_two_edge_local_object_mode_radix_asserted_head_consumes_immediate_expansion() {
        let source_id = source_id(204);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("two_edge_local_object_mode_radix_asserted_head"),
        );
        let base_mode = "BaseTwoEdgeObjectModeRadixAssertedHead";
        let middle_mode = "MiddleTwoEdgeObjectModeRadixAssertedHead";
        let outer_mode = "OuterTwoEdgeObjectModeRadixAssertedHead";
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                (base_mode, SymbolKind::Mode),
                (middle_mode, SymbolKind::Mode),
                (outer_mode, SymbolKind::Mode),
                ("DeeperTwoEdgeObjectModeRadixAssertedHead", SymbolKind::Mode),
                ("OtherTwoEdgeObjectModeRadixAssertedHead", SymbolKind::Mode),
            ],
        );
        let theorem = exact_two_edge_local_object_mode_radix_asserted_head_spec();
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    base_mode,
                    "BaseTwoEdgeObjectModeRadixAssertedHeadDef",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition_with_label(
                    middle_mode,
                    "MiddleTwoEdgeObjectModeRadixAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(base_mode),
                ),
                mode_definition_with_label(
                    outer_mode,
                    "OuterTwoEdgeObjectModeRadixAssertedHeadDef",
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
        assert!(
            super::extract_source_reserved_object_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none()
        );
        assert!(
            extract_source_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none()
        );
        assert!(
            extract_source_chained_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none()
        );
        assert!(
            extract_source_two_edge_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none()
        );
        assert!(
            extract_source_two_edge_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none()
        );
        assert!(
            extract_source_chained_local_object_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none()
        );
        assert!(
            extract_source_two_edge_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none()
        );
        assert!(
            extract_source_two_edge_local_mode_asserted_head(&exact, module.clone(), &symbols,)
                .is_none()
        );
        assert!(
            extract_source_two_edge_local_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none()
        );

        let payload = extract_source_two_edge_local_object_mode_radix_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact two-edge object immediate-radix asserted head should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 3);
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

        let output = source_two_edge_local_object_mode_radix_asserted_head_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact two-edge object immediate-radix source should reach the checker");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 204 checker payload invariants should hold");
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.payload.reserve.mode_expansions.len(), 3);
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
            .expect("one normalized builtin-object type should exist");
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

        let assert_invalid_output = |invalid| {
            let invalid_result =
                assert_source_reserved_variable_type_assertion_output(&invalid).map(|()| invalid);
            assert_eq!(
                source_reserved_variable_type_assertion_result_detail_keys(
                    invalid_result,
                    TYPE_ELABORATION_TWO_EDGE_LOCAL_OBJECT_MODE_RADIX_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
                ),
                vec![
                    TYPE_ELABORATION_TWO_EDGE_LOCAL_OBJECT_MODE_RADIX_ASSERTED_HEAD_INVALID_PAYLOAD_KEY
                        .to_owned()
                ]
            );
        };
        for removed in [base_mode, middle_mode, outer_mode] {
            let mut invalid = source_two_edge_local_object_mode_radix_asserted_head_output(
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
            source_two_edge_local_object_mode_radix_asserted_head_output(
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
        invalid.asserted_type_input.spelling = base_mode.to_owned();
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
            .expect("corrupting cloned Task 204 outputs must not mutate the exact output");

        let assert_extraction_gap = |ast, context: &str| {
            assert_eq!(
                source_type_elaboration_detail_keys(&ast, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()],
                "{context}"
            );
        };
        let mut mode_near_misses = vec![
            Vec::<ModeDefinitionSpec>::new(),
            vec![exact_modes()[0], exact_modes()[1]],
            vec![exact_modes()[0], exact_modes()[2]],
            vec![exact_modes()[1], exact_modes()[2]],
            vec![exact_modes()[0], exact_modes()[2], exact_modes()[1]],
            vec![exact_modes()[1], exact_modes()[0], exact_modes()[2]],
            vec![exact_modes()[1], exact_modes()[2], exact_modes()[0]],
            vec![exact_modes()[2], exact_modes()[0], exact_modes()[1]],
            vec![exact_modes()[2], exact_modes()[1], exact_modes()[0]],
            {
                let mut modes = exact_modes();
                modes.push(modes[0]);
                modes
            },
            {
                let mut modes = exact_modes();
                modes.push(modes[1]);
                modes
            },
            {
                let mut modes = exact_modes();
                modes.push(modes[2]);
                modes
            },
            {
                let mut modes = exact_modes();
                modes.push(mode_definition_with_label(
                    "UnusedTwoEdgeObjectModeRadixAssertedHead",
                    "UnusedTwoEdgeObjectModeRadixAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(base_mode),
                ));
                modes
            },
            vec![
                mode_definition_with_label(
                    base_mode,
                    "BaseTwoEdgeObjectModeRadixAssertedHeadDef",
                    ReserveTypeShape::Builtin("set"),
                ),
                exact_modes()[1],
                exact_modes()[2],
            ],
            vec![
                exact_modes()[0],
                mode_definition_with_label(
                    middle_mode,
                    "MiddleTwoEdgeObjectModeRadixAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol("OtherTwoEdgeObjectModeRadixAssertedHead"),
                ),
                exact_modes()[2],
            ],
            vec![
                exact_modes()[0],
                exact_modes()[1],
                mode_definition_with_label(
                    outer_mode,
                    "OuterTwoEdgeObjectModeRadixAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(base_mode),
                ),
            ],
        ];
        for index in 0..3 {
            let mut wrong_pattern = exact_modes();
            wrong_pattern[index].pattern = "OtherTwoEdgeObjectModeRadixAssertedHead";
            mode_near_misses.push(wrong_pattern);
            let mut wrong_label = exact_modes();
            wrong_label[index].label = Some("WrongTwoEdgeObjectModeRadixAssertedHeadDef");
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
            let expected_radix = match index {
                0 => "object",
                1 => base_mode,
                _ => middle_mode,
            };
            let mut args = exact_modes();
            args[index].rhs_shape = ReserveTypeShape::QualifiedSymbolWithArgs(expected_radix);
            mode_near_misses.push(args);
            let mut attributes = exact_modes();
            attributes[index].rhs_shape = if index == 0 {
                ReserveTypeShape::AttributedObject
            } else {
                ReserveTypeShape::AttributedQualifiedSymbol(expected_radix)
            };
            mode_near_misses.push(attributes);
        }
        for (index, modes) in mode_near_misses.into_iter().enumerate() {
            assert_extraction_gap(
                mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    modes,
                    reserve(),
                    theorem,
                ),
                &format!("definition near miss {index}"),
            );
        }

        let mut source_near_misses = Vec::new();
        let mut deeper_modes = exact_modes();
        deeper_modes.push(mode_definition_with_label(
            "DeeperTwoEdgeObjectModeRadixAssertedHead",
            "DeeperTwoEdgeObjectModeRadixAssertedHeadDef",
            ReserveTypeShape::QualifiedSymbol(outer_mode),
        ));
        source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            deeper_modes,
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("DeeperTwoEdgeObjectModeRadixAssertedHead"),
            )],
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(outer_mode),
                ..theorem
            },
        ));
        for bad_reserve in [
            ReserveTypeShape::QualifiedSymbol(base_mode),
            ReserveTypeShape::QualifiedSymbol(middle_mode),
            ReserveTypeShape::QualifiedSymbolWithArgs(outer_mode),
            ReserveTypeShape::AttributedQualifiedSymbol(outer_mode),
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
                    "OtherTwoEdgeObjectModeRadixAssertedHead",
                ),
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
            &[
                (base_mode, SymbolKind::Mode),
                (middle_mode, SymbolKind::Mode),
                (outer_mode, SymbolKind::Mode),
            ],
            &[(
                "UnrelatedTwoEdgeObjectModeRadixAssertedHead",
                SymbolKind::Mode,
            )],
        );
        assert!(
            extract_source_two_edge_local_object_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &unrelated_import,
            )
            .is_some()
        );
        for asserted_head_symbols in [
            source_local_and_imported_symbols_env(
                module.clone(),
                &[
                    (middle_mode, SymbolKind::Mode),
                    (outer_mode, SymbolKind::Mode),
                ],
                &[(base_mode, SymbolKind::Mode)],
            ),
            source_local_and_imported_symbols_env(
                module.clone(),
                &[
                    (middle_mode, SymbolKind::Mode),
                    (outer_mode, SymbolKind::Mode),
                ],
                &[(base_mode, SymbolKind::Mode), (base_mode, SymbolKind::Mode)],
            ),
            source_local_and_imported_symbols_env(
                module.clone(),
                &[
                    (base_mode, SymbolKind::Mode),
                    (outer_mode, SymbolKind::Mode),
                ],
                &[(middle_mode, SymbolKind::Mode)],
            ),
            source_local_and_imported_symbols_env(
                module.clone(),
                &[
                    (base_mode, SymbolKind::Mode),
                    (outer_mode, SymbolKind::Mode),
                ],
                &[
                    (middle_mode, SymbolKind::Mode),
                    (middle_mode, SymbolKind::Mode),
                ],
            ),
            source_local_and_imported_symbols_env(
                module.clone(),
                &[
                    (base_mode, SymbolKind::Mode),
                    (middle_mode, SymbolKind::Mode),
                ],
                &[(outer_mode, SymbolKind::Mode)],
            ),
            source_local_and_imported_symbols_env(
                module.clone(),
                &[
                    (base_mode, SymbolKind::Mode),
                    (middle_mode, SymbolKind::Mode),
                ],
                &[
                    (outer_mode, SymbolKind::Mode),
                    (outer_mode, SymbolKind::Mode),
                ],
            ),
        ] {
            assert_eq!(
                source_type_elaboration_detail_keys(&exact, module.clone(), &asserted_head_symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }

        assert_task_204_route_isolation(source_id, module, &symbols);
    }

    fn assert_task_204_route_isolation(
        source_id: SourceId,
        module: ResolverModuleId,
        task_204_symbols: &SymbolEnv,
    ) {
        let task_189 = reserve_then_identifier_type_assertion_theorem_ast(
            source_id,
            vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("object"))],
            exact_reserved_object_identifier_type_assertion_spec(),
        );
        assert!(
            super::extract_source_reserved_object_variable_type_assertion(
                &task_189,
                module.clone(),
                task_204_symbols,
            )
            .is_some()
        );
        assert!(
            extract_source_two_edge_local_object_mode_radix_asserted_head(
                &task_189,
                module.clone(),
                task_204_symbols,
            )
            .is_none()
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
        assert!(
            extract_source_local_object_mode_reserved_variable_type_assertion(
                &task_145,
                module.clone(),
                &task_145_symbols,
            )
            .is_some()
        );
        assert!(
            extract_source_two_edge_local_object_mode_radix_asserted_head(
                &task_145,
                module.clone(),
                &task_145_symbols,
            )
            .is_none()
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
        assert!(
            extract_source_chained_local_object_mode_reserved_variable_type_assertion(
                &task_147,
                module.clone(),
                &task_147_symbols,
            )
            .is_some()
        );
        assert!(
            extract_source_two_edge_local_object_mode_radix_asserted_head(
                &task_147,
                module.clone(),
                &task_147_symbols,
            )
            .is_none()
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
        assert!(
            extract_source_two_edge_local_object_mode_reserved_variable_type_assertion(
                &task_149,
                module.clone(),
                &task_149_symbols,
            )
            .is_some()
        );
        assert!(
            extract_source_two_edge_local_object_mode_radix_asserted_head(
                &task_149,
                module.clone(),
                &task_149_symbols,
            )
            .is_none()
        );

        let task_187_symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseTwoEdgeObjectModeAssertedHead", SymbolKind::Mode),
                ("MiddleTwoEdgeObjectModeAssertedHead", SymbolKind::Mode),
                ("OuterTwoEdgeObjectModeAssertedHead", SymbolKind::Mode),
            ],
        );
        let task_187 = mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            [
                mode_definition_with_label(
                    "BaseTwoEdgeObjectModeAssertedHead",
                    "BaseTwoEdgeObjectModeAssertedHeadDef",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition_with_label(
                    "MiddleTwoEdgeObjectModeAssertedHead",
                    "MiddleTwoEdgeObjectModeAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeObjectModeAssertedHead"),
                ),
                mode_definition_with_label(
                    "OuterTwoEdgeObjectModeAssertedHead",
                    "OuterTwoEdgeObjectModeAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeObjectModeAssertedHead"),
                ),
            ],
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("OuterTwoEdgeObjectModeAssertedHead"),
            )],
            exact_two_edge_local_object_mode_asserted_head_spec(),
        );
        assert!(
            extract_source_two_edge_local_object_mode_asserted_head(
                &task_187,
                module.clone(),
                &task_187_symbols,
            )
            .is_some()
        );
        assert!(
            extract_source_two_edge_local_object_mode_radix_asserted_head(
                &task_187,
                module.clone(),
                &task_187_symbols,
            )
            .is_none()
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
        assert!(
            extract_source_chained_local_object_mode_radix_asserted_head(
                &task_202,
                module.clone(),
                &task_202_symbols,
            )
            .is_some()
        );
        assert!(
            extract_source_two_edge_local_object_mode_radix_asserted_head(
                &task_202,
                module.clone(),
                &task_202_symbols,
            )
            .is_none()
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
        assert!(
            extract_source_two_edge_local_mode_reserved_variable_type_assertion(
                &task_148,
                module.clone(),
                &task_148_symbols,
            )
            .is_some()
        );
        assert!(
            extract_source_two_edge_local_object_mode_radix_asserted_head(
                &task_148,
                module.clone(),
                &task_148_symbols,
            )
            .is_none()
        );

        let task_186_symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseTwoEdgeModeAssertedHead", SymbolKind::Mode),
                ("MiddleTwoEdgeModeAssertedHead", SymbolKind::Mode),
                ("OuterTwoEdgeModeAssertedHead", SymbolKind::Mode),
            ],
        );
        let task_186 = mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            [
                mode_definition_with_label(
                    "BaseTwoEdgeModeAssertedHead",
                    "BaseTwoEdgeModeAssertedHeadDef",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition_with_label(
                    "MiddleTwoEdgeModeAssertedHead",
                    "MiddleTwoEdgeModeAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeModeAssertedHead"),
                ),
                mode_definition_with_label(
                    "OuterTwoEdgeModeAssertedHead",
                    "OuterTwoEdgeModeAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeModeAssertedHead"),
                ),
            ],
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("OuterTwoEdgeModeAssertedHead"),
            )],
            exact_two_edge_local_mode_asserted_head_spec(),
        );
        assert!(
            extract_source_two_edge_local_mode_asserted_head(
                &task_186,
                module.clone(),
                &task_186_symbols,
            )
            .is_some()
        );
        assert!(
            extract_source_two_edge_local_object_mode_radix_asserted_head(
                &task_186,
                module.clone(),
                &task_186_symbols,
            )
            .is_none()
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
        assert!(
            extract_source_two_edge_local_mode_radix_asserted_head(
                &task_203,
                module.clone(),
                &task_203_symbols,
            )
            .is_some()
        );
        assert!(
            extract_source_two_edge_local_object_mode_radix_asserted_head(
                &task_203,
                module,
                &task_203_symbols,
            )
            .is_none()
        );
    }
