    #[test]
    fn source_chained_local_object_mode_radix_asserted_head_consumes_immediate_expansion() {
        let source_id = source_id(202);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("chained_local_object_mode_radix_asserted_head"),
        );
        let base_mode = "BaseObjectModeRadixAssertedHead";
        let outer_mode = "OuterObjectModeRadixAssertedHead";
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                (base_mode, SymbolKind::Mode),
                (outer_mode, SymbolKind::Mode),
                ("DeeperObjectModeRadixAssertedHead", SymbolKind::Mode),
                ("OtherObjectModeRadixAssertedHead", SymbolKind::Mode),
            ],
        );
        let theorem = exact_chained_local_object_mode_radix_asserted_head_spec();
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    base_mode,
                    "BaseObjectModeRadixAssertedHeadDef",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition_with_label(
                    outer_mode,
                    "OuterObjectModeRadixAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(base_mode),
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
            extract_source_chained_local_object_mode_asserted_head(
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
            extract_source_chained_local_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none()
        );
        let payload = extract_source_chained_local_object_mode_radix_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact object one-edge immediate-radix asserted head should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 2);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(payload.subject_lookup_ordinal, 1);
        let source_binding = &payload.reserve.bridge.bindings()[0];
        assert_eq!(source_binding.type_spelling, outer_mode);
        assert_eq!(payload.asserted_type.spelling, base_mode);
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
            .expect("object outer binding expansion should exist");
        assert_eq!(outer_expansion.radix.spelling, base_mode);
        assert_eq!(outer_expansion.radix.head, payload.asserted_type.head);

        let output = source_chained_local_object_mode_radix_asserted_head_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact object immediate-radix asserted head should reach the checker");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 202 checker payload invariants should hold");
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.payload.reserve.mode_expansions.len(), 2);
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert_eq!(output.subject_result_input.spelling, outer_mode);
        assert_eq!(output.asserted_type_input.spelling, base_mode);
        assert_ne!(
            output.subject_result_input.head,
            output.asserted_type_input.head
        );
        assert!(matches!(
            output.subject_result_input.head,
            TypeHeadInput::Symbol(_)
        ));
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
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(base_mode))
            .map(|(_, expansion)| &expansion.radix)
            .expect("object base terminal expansion should exist");
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
                    TYPE_ELABORATION_CHAINED_LOCAL_OBJECT_MODE_RADIX_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
                ),
                vec![
                    TYPE_ELABORATION_CHAINED_LOCAL_OBJECT_MODE_RADIX_ASSERTED_HEAD_INVALID_PAYLOAD_KEY
                        .to_owned()
                ]
            );
        };
        for removed in [base_mode, outer_mode] {
            let mut invalid = source_chained_local_object_mode_radix_asserted_head_output(
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
            source_chained_local_object_mode_radix_asserted_head_output(
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
        invalid.subject_result_input.spelling = base_mode.to_owned();
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
        invalid.asserted_type_input.spelling = "OtherObjectModeRadixAssertedHead".to_owned();
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
            .expect("corrupting cloned object outputs must not mutate the exact output");

        let assert_extraction_gap = |ast, context: &str| {
            assert_eq!(
                source_type_elaboration_detail_keys(&ast, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()],
                "{context}"
            );
        };
        let mut mode_near_misses = vec![
            Vec::<ModeDefinitionSpec>::new(),
            vec![exact_modes()[0]],
            vec![exact_modes()[1]],
            vec![exact_modes()[1], exact_modes()[0]],
            {
                let mut modes = exact_modes();
                modes.push(modes[0]);
                modes
            },
            {
                let mut modes = exact_modes();
                modes.push(mode_definition_with_label(
                    "UnusedObjectModeRadixAssertedHead",
                    "UnusedObjectModeRadixAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(base_mode),
                ));
                modes
            },
            vec![
                mode_definition_with_label(
                    base_mode,
                    "BaseObjectModeRadixAssertedHeadDef",
                    ReserveTypeShape::Builtin("set"),
                ),
                exact_modes()[1],
            ],
            vec![
                exact_modes()[0],
                mode_definition_with_label(
                    outer_mode,
                    "OuterObjectModeRadixAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol("OtherObjectModeRadixAssertedHead"),
                ),
            ],
        ];
        for index in 0..2 {
            let mut wrong_label = exact_modes();
            wrong_label[index].label = Some("WrongObjectModeRadixAssertedHeadDef");
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
                ReserveTypeShape::QualifiedSymbolWithArgs(base_mode)
            };
            mode_near_misses.push(args);
            let mut attrs = exact_modes();
            attrs[index].rhs_shape = if index == 0 {
                ReserveTypeShape::AttributedObject
            } else {
                ReserveTypeShape::AttributedQualifiedSymbol(base_mode)
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
                &format!("object definition near miss {index}"),
            );
        }

        let mut source_near_misses = Vec::new();
        let mut deeper_modes = exact_modes();
        deeper_modes.push(mode_definition_with_label(
            "DeeperObjectModeRadixAssertedHead",
            "DeeperObjectModeRadixAssertedHeadDef",
            ReserveTypeShape::QualifiedSymbol(outer_mode),
        ));
        source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            deeper_modes,
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("DeeperObjectModeRadixAssertedHead"),
            )],
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(outer_mode),
                ..theorem
            },
        ));
        for bad_reserve in [
            ReserveTypeShape::QualifiedSymbol(base_mode),
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
                asserted_type: ReserveTypeShape::Builtin("object"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::Builtin("set"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(
                    "DeeperObjectModeRadixAssertedHead",
                ),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(
                    "OtherObjectModeRadixAssertedHead",
                ),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbolWithArgs(base_mode),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::AttributedQualifiedSymbol(base_mode),
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
            assert_extraction_gap(near_miss, &format!("object source near miss {index}"));
        }

        let unrelated_import = source_local_and_imported_symbols_env(
            module.clone(),
            &[
                (base_mode, SymbolKind::Mode),
                (outer_mode, SymbolKind::Mode),
            ],
            &[("UnrelatedObjectModeRadixAssertedHead", SymbolKind::Mode)],
        );
        assert!(
            extract_source_chained_local_object_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &unrelated_import,
            )
            .is_some()
        );
        for asserted_head_symbols in [
            source_local_and_imported_symbols_env(
                module.clone(),
                &[(outer_mode, SymbolKind::Mode)],
                &[(base_mode, SymbolKind::Mode)],
            ),
            source_local_and_imported_symbols_env(
                module.clone(),
                &[(outer_mode, SymbolKind::Mode)],
                &[(base_mode, SymbolKind::Mode), (base_mode, SymbolKind::Mode)],
            ),
        ] {
            assert_eq!(
                source_type_elaboration_detail_keys(&exact, module.clone(), &asserted_head_symbols,),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }

        let task189 = reserve_then_identifier_type_assertion_theorem_ast(
            source_id,
            vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("object"))],
            exact_reserved_object_identifier_type_assertion_spec(),
        );
        assert!(
            super::extract_source_reserved_object_variable_type_assertion(
                &task189,
                module.clone(),
                &symbols,
            )
            .is_some()
        );
        assert!(
            extract_source_chained_local_object_mode_radix_asserted_head(
                &task189,
                module.clone(),
                &symbols,
            )
            .is_none()
        );

        let task147_symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseObjectModeTypeAssertion", SymbolKind::Mode),
                ("ChainObjectModeTypeAssertion", SymbolKind::Mode),
            ],
        );
        let task147 = mode_then_reserve_identifier_type_assertion_theorem_ast(
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
                &task147,
                module.clone(),
                &task147_symbols,
            )
            .is_some()
        );
        assert!(
            extract_source_chained_local_object_mode_radix_asserted_head(
                &task147,
                module.clone(),
                &task147_symbols,
            )
            .is_none()
        );

        let task185_symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseObjectModeAssertedHead", SymbolKind::Mode),
                ("ChainObjectModeAssertedHead", SymbolKind::Mode),
            ],
        );
        let task185 = mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            [
                mode_definition_with_label(
                    "BaseObjectModeAssertedHead",
                    "BaseObjectModeAssertedHeadDef",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition_with_label(
                    "ChainObjectModeAssertedHead",
                    "ChainObjectModeAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol("BaseObjectModeAssertedHead"),
                ),
            ],
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("ChainObjectModeAssertedHead"),
            )],
            exact_chained_local_object_mode_asserted_head_spec(),
        );
        assert!(
            extract_source_chained_local_object_mode_asserted_head(
                &task185,
                module.clone(),
                &task185_symbols,
            )
            .is_some()
        );
        assert!(
            extract_source_chained_local_object_mode_radix_asserted_head(
                &task185,
                module.clone(),
                &task185_symbols,
            )
            .is_none()
        );

        let task201_symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseModeRadixAssertedHead", SymbolKind::Mode),
                ("OuterModeRadixAssertedHead", SymbolKind::Mode),
            ],
        );
        let task201 = mode_then_reserve_identifier_type_assertion_theorem_ast(
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
        assert!(
            extract_source_chained_local_mode_radix_asserted_head(
                &task201,
                module.clone(),
                &task201_symbols,
            )
            .is_some()
        );
        assert!(
            extract_source_chained_local_object_mode_radix_asserted_head(
                &task201,
                module.clone(),
                &task201_symbols,
            )
            .is_none()
        );
    }

    #[test]
    fn source_chained_local_object_mode_asserted_head_type_assertion_consumes_both_expansions() {
        let source_id = source_id(185);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("chained_local_object_mode_asserted_head_type_assertion"),
        );
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseObjectModeAssertedHead", SymbolKind::Mode),
                ("ChainObjectModeAssertedHead", SymbolKind::Mode),
                ("ExtraObjectModeAssertedHead", SymbolKind::Mode),
                ("OtherObjectModeAssertedHead", SymbolKind::Mode),
            ],
        );
        let theorem = exact_chained_local_object_mode_asserted_head_spec();
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    "BaseObjectModeAssertedHead",
                    "BaseObjectModeAssertedHeadDef",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition_with_label(
                    "ChainObjectModeAssertedHead",
                    "ChainObjectModeAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol("BaseObjectModeAssertedHead"),
                ),
            ]
        };
        let reserve = || {
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("ChainObjectModeAssertedHead"),
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
        let payload = extract_source_chained_local_object_mode_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact one-edge formula-side local-object-mode assertion should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 2);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        let source_binding = &payload.reserve.bridge.bindings()[0];
        assert_eq!(source_binding.type_spelling, "ChainObjectModeAssertedHead");
        assert_eq!(
            payload.asserted_type.spelling,
            "ChainObjectModeAssertedHead"
        );
        assert!(matches!(source_binding.type_head, TypeHeadInput::Symbol(_)));
        assert_eq!(payload.asserted_type.head, source_binding.type_head);
        assert_ne!(source_binding.type_range, payload.asserted_type.range);
        assert_eq!(payload.subject_lookup_ordinal, 1);

        let output =
            source_chained_local_object_mode_asserted_head_output(&exact, module.clone(), &symbols)
                .expect("exact one-edge formula-side assertion should reach the checker");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("one-edge formula-side assertion invariants should hold");
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(
            output.subject_result_input.spelling,
            "ChainObjectModeAssertedHead"
        );
        assert_eq!(
            output.asserted_type_input.spelling,
            "ChainObjectModeAssertedHead"
        );
        assert_eq!(
            output.subject_result_input.head,
            output.asserted_type_input.head
        );
        assert!(matches!(
            output.asserted_type_input.head,
            TypeHeadInput::Symbol(_)
        ));
        assert_ne!(
            output.subject_result_input.source_range,
            output.asserted_type_input.source_range
        );
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("BaseObjectModeAssertedHead")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("real base-mode terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("one normalized builtin-object type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
        let (_, term) = output
            .term_formula
            .terms()
            .iter()
            .next()
            .expect("reserved-variable subject should be inferred");
        assert_eq!(term.status, TermStatus::Inferred);
        assert!(term.deferred.is_empty());
        let (_, formula) = output
            .term_formula
            .formulas()
            .iter()
            .next()
            .expect("one-edge local-object-mode type assertion should be checked");
        assert_eq!(formula.kind, FormulaKind::TypeAssertion);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert!(formula.expected_types.is_empty());
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());

        let invalid_key = || {
            vec![
                TYPE_ELABORATION_CHAINED_LOCAL_OBJECT_MODE_ASSERTED_HEAD_INVALID_PAYLOAD_KEY
                    .to_owned(),
            ]
        };
        for removed in ["BaseObjectModeAssertedHead", "ChainObjectModeAssertedHead"] {
            let mut invalid = source_chained_local_object_mode_asserted_head_output(
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
                    TYPE_ELABORATION_CHAINED_LOCAL_OBJECT_MODE_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
                ),
                invalid_key()
            );
        }
        let mut wrong_asserted_spelling =
            source_chained_local_object_mode_asserted_head_output(&exact, module.clone(), &symbols)
                .expect("exact source should produce an asserted-spelling corruption target");
        wrong_asserted_spelling.asserted_type_input.spelling =
            "OtherObjectModeAssertedHead".to_owned();
        let mut wrong_asserted_head =
            source_chained_local_object_mode_asserted_head_output(&exact, module.clone(), &symbols)
                .expect("exact source should produce an asserted-head corruption target");
        wrong_asserted_head.asserted_type_input.head = TypeHeadInput::BuiltinSet;
        let mut collapsed_ranges =
            source_chained_local_object_mode_asserted_head_output(&exact, module.clone(), &symbols)
                .expect("exact source should produce a range corruption target");
        collapsed_ranges.asserted_type_input.source_range =
            collapsed_ranges.subject_result_input.source_range;
        let mut wrong_subject_head =
            source_chained_local_object_mode_asserted_head_output(&exact, module.clone(), &symbols)
                .expect("exact source should produce a subject-head corruption target");
        wrong_subject_head.subject_result_input.head = TypeHeadInput::BuiltinSet;
        for invalid in [
            wrong_asserted_spelling,
            wrong_asserted_head,
            collapsed_ranges,
            wrong_subject_head,
        ] {
            let invalid_result =
                assert_source_reserved_variable_type_assertion_output(&invalid).map(|()| invalid);
            assert_eq!(
                source_reserved_variable_type_assertion_result_detail_keys(
                    invalid_result,
                    TYPE_ELABORATION_CHAINED_LOCAL_OBJECT_MODE_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
                ),
                invalid_key()
            );
        }

        let near_miss_modes = [
            Vec::<ModeDefinitionSpec>::new(),
            vec![exact_modes()[0]],
            vec![exact_modes()[1]],
            {
                let mut modes = exact_modes();
                modes.push(modes[0]);
                modes
            },
            vec![
                mode_definition_with_label(
                    "BaseObjectModeAssertedHead",
                    "OtherDef",
                    ReserveTypeShape::Builtin("object"),
                ),
                exact_modes()[1],
            ],
            vec![
                exact_modes()[0],
                mode_definition_with_label(
                    "ChainObjectModeAssertedHead",
                    "OtherDef",
                    ReserveTypeShape::QualifiedSymbol("BaseObjectModeAssertedHead"),
                ),
            ],
            vec![
                recovered_mode_definition(
                    "BaseObjectModeAssertedHead",
                    ReserveTypeShape::Builtin("object"),
                ),
                exact_modes()[1],
            ],
            vec![
                exact_modes()[0],
                recovered_mode_definition(
                    "ChainObjectModeAssertedHead",
                    ReserveTypeShape::QualifiedSymbol("BaseObjectModeAssertedHead"),
                ),
            ],
            vec![
                contextual_mode_definition(
                    "BaseObjectModeAssertedHead",
                    ReserveTypeShape::Builtin("object"),
                ),
                exact_modes()[1],
            ],
            vec![
                exact_modes()[0],
                parameterized_mode_definition(
                    "ChainObjectModeAssertedHead",
                    ReserveTypeShape::QualifiedSymbol("BaseObjectModeAssertedHead"),
                ),
            ],
            vec![
                exact_modes()[0],
                mode_definition_with_label(
                    "ChainObjectModeAssertedHead",
                    "ChainObjectModeAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbolWithArgs("BaseObjectModeAssertedHead"),
                ),
            ],
            vec![
                exact_modes()[0],
                mode_definition_with_label(
                    "ChainObjectModeAssertedHead",
                    "ChainObjectModeAssertedHeadDef",
                    ReserveTypeShape::AttributedQualifiedSymbol("BaseObjectModeAssertedHead"),
                ),
            ],
            vec![
                mode_definition_with_label(
                    "BaseObjectModeAssertedHead",
                    "BaseObjectModeAssertedHeadDef",
                    ReserveTypeShape::AttributedObject,
                ),
                exact_modes()[1],
            ],
            vec![
                mode_definition_with_label(
                    "BaseObjectModeAssertedHead",
                    "BaseObjectModeAssertedHeadDef",
                    ReserveTypeShape::Builtin("set"),
                ),
                exact_modes()[1],
            ],
            vec![exact_modes()[1], exact_modes()[0]],
            vec![mode_definition_with_label(
                "ChainObjectModeAssertedHead",
                "ChainObjectModeAssertedHeadDef",
                ReserveTypeShape::Builtin("object"),
            )],
            vec![
                mode_definition_with_label(
                    "BaseObjectModeAssertedHead",
                    "BaseObjectModeAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol("ChainObjectModeAssertedHead"),
                ),
                exact_modes()[1],
            ],
            vec![
                exact_modes()[0],
                mode_definition_with_label(
                    "ExtraObjectModeAssertedHead",
                    "ExtraObjectModeAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol("BaseObjectModeAssertedHead"),
                ),
                mode_definition_with_label(
                    "ChainObjectModeAssertedHead",
                    "ChainObjectModeAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol("ExtraObjectModeAssertedHead"),
                ),
            ],
        ];
        for modes in near_miss_modes {
            let near_miss = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                modes,
                reserve(),
                theorem,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }

        for near_miss in [
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbolWithArgs("ChainObjectModeAssertedHead"),
                )],
                theorem,
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::AttributedQualifiedSymbol("ChainObjectModeAssertedHead"),
                )],
                theorem,
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                vec![
                    reserve_item(
                        vec!["x"],
                        ReserveTypeShape::QualifiedSymbol("ChainObjectModeAssertedHead"),
                    ),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("object")),
                ],
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
                    asserted_type: ReserveTypeShape::Builtin("set"),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol("BaseObjectModeAssertedHead"),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol("OtherObjectModeAssertedHead"),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbolWithArgs(
                        "ChainObjectModeAssertedHead",
                    ),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::AttributedQualifiedSymbol(
                        "ChainObjectModeAssertedHead",
                    ),
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
                    status: Some("registration"),
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

        let imported_outer_symbols = source_local_and_imported_symbols_env(
            module.clone(),
            &[("BaseObjectModeAssertedHead", SymbolKind::Mode)],
            &[("ChainObjectModeAssertedHead", SymbolKind::Mode)],
        );
        let imported_base_symbols = source_local_and_imported_symbols_env(
            module.clone(),
            &[("ChainObjectModeAssertedHead", SymbolKind::Mode)],
            &[("BaseObjectModeAssertedHead", SymbolKind::Mode)],
        );
        for provenance_near_miss_symbols in [&imported_outer_symbols, &imported_base_symbols] {
            let provenance_near_miss = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                theorem,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(
                    &provenance_near_miss,
                    module.clone(),
                    provenance_near_miss_symbols,
                ),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }

        for (asserted_spelling, asserted_head_symbols) in [
            (
                "ImportedObjectModeAssertedHead",
                source_local_and_imported_symbols_env(
                    module.clone(),
                    &[
                        ("BaseObjectModeAssertedHead", SymbolKind::Mode),
                        ("ChainObjectModeAssertedHead", SymbolKind::Mode),
                    ],
                    &[("ImportedObjectModeAssertedHead", SymbolKind::Mode)],
                ),
            ),
            (
                "AmbiguousObjectModeAssertedHead",
                source_local_and_imported_symbols_env(
                    module.clone(),
                    &[
                        ("BaseObjectModeAssertedHead", SymbolKind::Mode),
                        ("ChainObjectModeAssertedHead", SymbolKind::Mode),
                    ],
                    &[
                        ("AmbiguousObjectModeAssertedHead", SymbolKind::Mode),
                        ("AmbiguousObjectModeAssertedHead", SymbolKind::Mode),
                    ],
                ),
            ),
        ] {
            let asserted_head_near_miss = mode_then_reserve_identifier_type_assertion_theorem_ast(
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
                    &asserted_head_near_miss,
                    module.clone(),
                    &asserted_head_symbols,
                ),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
    }

    #[test]
    fn source_chained_local_mode_reserved_variable_type_assertion_consumes_both_expansions() {
        let source_id = source_id(146);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("chained_local_mode_reserved_variable_type_assertion"),
        );
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseModeTypeAssertion", SymbolKind::Mode),
                ("ChainModeTypeAssertion", SymbolKind::Mode),
                ("ExtraModeTypeAssertion", SymbolKind::Mode),
            ],
        );
        let theorem = exact_chained_local_mode_identifier_type_assertion_spec();
        let exact_modes = || {
            vec![
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
            ]
        };
        let reserve = || {
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("ChainModeTypeAssertion"),
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
        let payload = extract_source_chained_local_mode_reserved_variable_type_assertion(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact chained local-mode reserved-variable type assertion should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 2);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(
            payload.reserve.bridge.bindings()[0].type_spelling,
            "ChainModeTypeAssertion"
        );
        assert_eq!(payload.asserted_type.spelling, "set");
        assert_eq!(payload.asserted_type.head, TypeHeadInput::BuiltinSet);
        assert_ne!(
            payload.reserve.bridge.bindings()[0].type_range,
            payload.asserted_type.range
        );
        assert_eq!(payload.subject_lookup_ordinal, 1);

        let output = source_chained_local_mode_reserved_variable_type_assertion_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact chained local-mode type assertion should reach TermFormulaChecker");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("chained local-mode type assertion invariants should hold");
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(
            output.subject_result_input.spelling,
            "ChainModeTypeAssertion"
        );
        assert!(matches!(
            output.subject_result_input.head,
            TypeHeadInput::Symbol(_)
        ));
        assert_eq!(output.asserted_type_input.spelling, "set");
        assert_eq!(output.asserted_type_input.head, TypeHeadInput::BuiltinSet);
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("BaseModeTypeAssertion")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("base mode terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("one normalized builtin-set type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
        let (_, formula) = output.term_formula.formulas().iter().next().unwrap();
        assert_eq!(formula.kind, FormulaKind::TypeAssertion);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());

        for removed in ["BaseModeTypeAssertion", "ChainModeTypeAssertion"] {
            let mut invalid = source_chained_local_mode_reserved_variable_type_assertion_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce a chain corruption target");
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
                    TYPE_ELABORATION_CHAINED_LOCAL_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY,
                ),
                vec![
                    TYPE_ELABORATION_CHAINED_LOCAL_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY
                        .to_owned()
                ]
            );
        }

        let near_miss_modes = [
            vec![exact_modes()[1]],
            vec![exact_modes()[0]],
            {
                let mut modes = exact_modes();
                modes.push(modes[0]);
                modes
            },
            vec![
                mode_definition_with_label(
                    "BaseModeTypeAssertion",
                    "OtherDef",
                    ReserveTypeShape::Builtin("set"),
                ),
                exact_modes()[1],
            ],
            vec![
                exact_modes()[0],
                mode_definition_with_label(
                    "ChainModeTypeAssertion",
                    "OtherDef",
                    ReserveTypeShape::QualifiedSymbol("BaseModeTypeAssertion"),
                ),
            ],
            vec![
                recovered_mode_definition(
                    "BaseModeTypeAssertion",
                    ReserveTypeShape::Builtin("set"),
                ),
                exact_modes()[1],
            ],
            vec![
                exact_modes()[0],
                recovered_mode_definition(
                    "ChainModeTypeAssertion",
                    ReserveTypeShape::QualifiedSymbol("BaseModeTypeAssertion"),
                ),
            ],
            vec![
                contextual_mode_definition(
                    "BaseModeTypeAssertion",
                    ReserveTypeShape::Builtin("set"),
                ),
                exact_modes()[1],
            ],
            vec![
                exact_modes()[0],
                parameterized_mode_definition(
                    "ChainModeTypeAssertion",
                    ReserveTypeShape::QualifiedSymbol("BaseModeTypeAssertion"),
                ),
            ],
            vec![
                exact_modes()[0],
                mode_definition(
                    "ChainModeTypeAssertion",
                    ReserveTypeShape::QualifiedSymbolWithArgs("BaseModeTypeAssertion"),
                ),
            ],
            vec![
                mode_definition("BaseModeTypeAssertion", ReserveTypeShape::AttributedSet),
                exact_modes()[1],
            ],
            vec![
                mode_definition("BaseModeTypeAssertion", ReserveTypeShape::Builtin("object")),
                exact_modes()[1],
            ],
            vec![exact_modes()[1], exact_modes()[0]],
            vec![mode_definition_with_label(
                "ChainModeTypeAssertion",
                "ChainModeTypeAssertionDef",
                ReserveTypeShape::Builtin("set"),
            )],
            vec![
                mode_definition_with_label(
                    "BaseModeTypeAssertion",
                    "BaseModeTypeAssertionDef",
                    ReserveTypeShape::QualifiedSymbol("ChainModeTypeAssertion"),
                ),
                exact_modes()[1],
            ],
            vec![
                exact_modes()[0],
                mode_definition_with_label(
                    "ExtraModeTypeAssertion",
                    "ExtraModeTypeAssertionDef",
                    ReserveTypeShape::QualifiedSymbol("BaseModeTypeAssertion"),
                ),
                mode_definition_with_label(
                    "ChainModeTypeAssertion",
                    "ChainModeTypeAssertionDef",
                    ReserveTypeShape::QualifiedSymbol("ExtraModeTypeAssertion"),
                ),
            ],
        ];
        for modes in near_miss_modes {
            let near_miss = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                modes,
                reserve(),
                theorem,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }

        for near_miss in [
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbolWithArgs("ChainModeTypeAssertion"),
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
                    asserted_type: ReserveTypeShape::QualifiedSymbol("ChainModeTypeAssertion"),
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
    }

    #[test]
    fn source_chained_local_object_mode_reserved_variable_type_assertion_consumes_both_expansions()
    {
        let source_id = source_id(147);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("chained_local_object_mode_reserved_variable_type_assertion"),
        );
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseObjectModeTypeAssertion", SymbolKind::Mode),
                ("ChainObjectModeTypeAssertion", SymbolKind::Mode),
                ("ExtraObjectModeTypeAssertion", SymbolKind::Mode),
            ],
        );
        let theorem = exact_chained_local_object_mode_identifier_type_assertion_spec();
        let exact_modes = || {
            vec![
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
            ]
        };
        let reserve = || {
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("ChainObjectModeTypeAssertion"),
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
        let payload = extract_source_chained_local_object_mode_reserved_variable_type_assertion(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact chained local-object-mode reserved-variable type assertion should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 2);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(
            payload.reserve.bridge.bindings()[0].type_spelling,
            "ChainObjectModeTypeAssertion"
        );
        assert_eq!(payload.asserted_type.spelling, "object");
        assert_eq!(payload.asserted_type.head, TypeHeadInput::BuiltinObject);
        assert_ne!(
            payload.reserve.bridge.bindings()[0].type_range,
            payload.asserted_type.range
        );
        assert_eq!(payload.subject_lookup_ordinal, 1);

        let output = source_chained_local_object_mode_reserved_variable_type_assertion_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact chained local-object-mode type assertion should reach TermFormulaChecker");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("chained local-object-mode type assertion invariants should hold");
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(
            output.subject_result_input.spelling,
            "ChainObjectModeTypeAssertion"
        );
        assert!(matches!(
            output.subject_result_input.head,
            TypeHeadInput::Symbol(_)
        ));
        assert_eq!(output.asserted_type_input.spelling, "object");
        assert_eq!(
            output.asserted_type_input.head,
            TypeHeadInput::BuiltinObject
        );
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("BaseObjectModeTypeAssertion")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("base object mode terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("one normalized builtin-object type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
        let (_, formula) = output.term_formula.formulas().iter().next().unwrap();
        assert_eq!(formula.kind, FormulaKind::TypeAssertion);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());

        for removed in [
            "BaseObjectModeTypeAssertion",
            "ChainObjectModeTypeAssertion",
        ] {
            let mut invalid =
                source_chained_local_object_mode_reserved_variable_type_assertion_output(
                    &exact,
                    module.clone(),
                    &symbols,
                )
                .expect("exact source should produce an object-chain corruption target");
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
                    TYPE_ELABORATION_CHAINED_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY,
                ),
                vec![
                    TYPE_ELABORATION_CHAINED_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY
                        .to_owned()
                ]
            );
        }

        let near_miss_modes = [
            vec![exact_modes()[1]],
            vec![exact_modes()[0]],
            {
                let mut modes = exact_modes();
                modes.push(modes[0]);
                modes
            },
            vec![
                mode_definition_with_label(
                    "BaseObjectModeTypeAssertion",
                    "OtherDef",
                    ReserveTypeShape::Builtin("object"),
                ),
                exact_modes()[1],
            ],
            vec![
                exact_modes()[0],
                mode_definition_with_label(
                    "ChainObjectModeTypeAssertion",
                    "OtherDef",
                    ReserveTypeShape::QualifiedSymbol("BaseObjectModeTypeAssertion"),
                ),
            ],
            vec![
                recovered_mode_definition(
                    "BaseObjectModeTypeAssertion",
                    ReserveTypeShape::Builtin("object"),
                ),
                exact_modes()[1],
            ],
            vec![
                exact_modes()[0],
                recovered_mode_definition(
                    "ChainObjectModeTypeAssertion",
                    ReserveTypeShape::QualifiedSymbol("BaseObjectModeTypeAssertion"),
                ),
            ],
            vec![
                contextual_mode_definition(
                    "BaseObjectModeTypeAssertion",
                    ReserveTypeShape::Builtin("object"),
                ),
                exact_modes()[1],
            ],
            vec![
                exact_modes()[0],
                parameterized_mode_definition(
                    "ChainObjectModeTypeAssertion",
                    ReserveTypeShape::QualifiedSymbol("BaseObjectModeTypeAssertion"),
                ),
            ],
            vec![
                exact_modes()[0],
                mode_definition(
                    "ChainObjectModeTypeAssertion",
                    ReserveTypeShape::QualifiedSymbolWithArgs("BaseObjectModeTypeAssertion"),
                ),
            ],
            vec![
                mode_definition(
                    "BaseObjectModeTypeAssertion",
                    ReserveTypeShape::AttributedObject,
                ),
                exact_modes()[1],
            ],
            vec![
                mode_definition(
                    "BaseObjectModeTypeAssertion",
                    ReserveTypeShape::Builtin("set"),
                ),
                exact_modes()[1],
            ],
            vec![exact_modes()[1], exact_modes()[0]],
            vec![mode_definition_with_label(
                "ChainObjectModeTypeAssertion",
                "ChainObjectModeTypeAssertionDef",
                ReserveTypeShape::Builtin("object"),
            )],
            vec![
                mode_definition_with_label(
                    "BaseObjectModeTypeAssertion",
                    "BaseObjectModeTypeAssertionDef",
                    ReserveTypeShape::QualifiedSymbol("ChainObjectModeTypeAssertion"),
                ),
                exact_modes()[1],
            ],
            vec![
                exact_modes()[0],
                mode_definition_with_label(
                    "ExtraObjectModeTypeAssertion",
                    "ExtraObjectModeTypeAssertionDef",
                    ReserveTypeShape::QualifiedSymbol("BaseObjectModeTypeAssertion"),
                ),
                mode_definition_with_label(
                    "ChainObjectModeTypeAssertion",
                    "ChainObjectModeTypeAssertionDef",
                    ReserveTypeShape::QualifiedSymbol("ExtraObjectModeTypeAssertion"),
                ),
            ],
        ];
        for modes in near_miss_modes {
            let near_miss = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                modes,
                reserve(),
                theorem,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }

        for near_miss in [
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbolWithArgs("ChainObjectModeTypeAssertion"),
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
                    asserted_type: ReserveTypeShape::QualifiedSymbol(
                        "ChainObjectModeTypeAssertion",
                    ),
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
    }

    #[test]
    fn source_two_edge_local_mode_reserved_variable_type_assertion_consumes_three_expansions() {
        let source_id = source_id(148);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("two_edge_local_mode_reserved_variable_type_assertion"),
        );
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseTwoEdgeModeTypeAssertion", SymbolKind::Mode),
                ("MiddleTwoEdgeModeTypeAssertion", SymbolKind::Mode),
                ("OuterTwoEdgeModeTypeAssertion", SymbolKind::Mode),
                ("ExtraTwoEdgeModeTypeAssertion", SymbolKind::Mode),
            ],
        );
        let theorem = exact_two_edge_local_mode_identifier_type_assertion_spec();
        let exact_modes = || {
            vec![
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
            ]
        };
        let reserve = || {
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("OuterTwoEdgeModeTypeAssertion"),
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
        let payload = extract_source_two_edge_local_mode_reserved_variable_type_assertion(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact two-edge local-mode reserved-variable type assertion should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 3);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(
            payload.reserve.bridge.bindings()[0].type_spelling,
            "OuterTwoEdgeModeTypeAssertion"
        );
        assert_eq!(payload.asserted_type.spelling, "set");
        assert_eq!(payload.asserted_type.head, TypeHeadInput::BuiltinSet);
        assert_ne!(
            payload.reserve.bridge.bindings()[0].type_range,
            payload.asserted_type.range
        );
        assert_eq!(payload.subject_lookup_ordinal, 1);

        let output = source_two_edge_local_mode_reserved_variable_type_assertion_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact two-edge local-mode type assertion should reach TermFormulaChecker");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("two-edge local-mode type assertion invariants should hold");
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(
            output.subject_result_input.spelling,
            "OuterTwoEdgeModeTypeAssertion"
        );
        assert!(matches!(
            output.subject_result_input.head,
            TypeHeadInput::Symbol(_)
        ));
        assert_eq!(output.asserted_type_input.spelling, "set");
        assert_eq!(output.asserted_type_input.head, TypeHeadInput::BuiltinSet);
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("BaseTwoEdgeModeTypeAssertion")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("base mode terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("one normalized builtin-set type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
        let (_, formula) = output.term_formula.formulas().iter().next().unwrap();
        assert_eq!(formula.kind, FormulaKind::TypeAssertion);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());

        for removed in [
            "BaseTwoEdgeModeTypeAssertion",
            "MiddleTwoEdgeModeTypeAssertion",
            "OuterTwoEdgeModeTypeAssertion",
        ] {
            let mut invalid = source_two_edge_local_mode_reserved_variable_type_assertion_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce a three-expansion corruption target");
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
                    TYPE_ELABORATION_TWO_EDGE_LOCAL_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY,
                ),
                vec![
                    TYPE_ELABORATION_TWO_EDGE_LOCAL_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY
                        .to_owned()
                ]
            );
        }

        let near_miss_modes = [
            vec![exact_modes()[1], exact_modes()[2]],
            vec![exact_modes()[0], exact_modes()[2]],
            vec![exact_modes()[0], exact_modes()[1]],
            {
                let mut modes = exact_modes();
                modes.push(modes[0]);
                modes
            },
            vec![
                mode_definition_with_label(
                    "BaseTwoEdgeModeTypeAssertion",
                    "OtherDef",
                    ReserveTypeShape::Builtin("set"),
                ),
                exact_modes()[1],
                exact_modes()[2],
            ],
            vec![
                exact_modes()[0],
                mode_definition_with_label(
                    "MiddleTwoEdgeModeTypeAssertion",
                    "OtherDef",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeModeTypeAssertion"),
                ),
                exact_modes()[2],
            ],
            vec![
                exact_modes()[0],
                exact_modes()[1],
                mode_definition_with_label(
                    "OuterTwoEdgeModeTypeAssertion",
                    "OtherDef",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeModeTypeAssertion"),
                ),
            ],
            vec![
                recovered_mode_definition(
                    "BaseTwoEdgeModeTypeAssertion",
                    ReserveTypeShape::Builtin("set"),
                ),
                exact_modes()[1],
                exact_modes()[2],
            ],
            vec![
                exact_modes()[0],
                recovered_mode_definition(
                    "MiddleTwoEdgeModeTypeAssertion",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeModeTypeAssertion"),
                ),
                exact_modes()[2],
            ],
            vec![
                exact_modes()[0],
                exact_modes()[1],
                recovered_mode_definition(
                    "OuterTwoEdgeModeTypeAssertion",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeModeTypeAssertion"),
                ),
            ],
            vec![
                contextual_mode_definition(
                    "BaseTwoEdgeModeTypeAssertion",
                    ReserveTypeShape::Builtin("set"),
                ),
                exact_modes()[1],
                exact_modes()[2],
            ],
            vec![
                exact_modes()[0],
                parameterized_mode_definition(
                    "MiddleTwoEdgeModeTypeAssertion",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeModeTypeAssertion"),
                ),
                exact_modes()[2],
            ],
            vec![
                exact_modes()[0],
                exact_modes()[1],
                parameterized_mode_definition(
                    "OuterTwoEdgeModeTypeAssertion",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeModeTypeAssertion"),
                ),
            ],
            vec![
                exact_modes()[0],
                mode_definition(
                    "MiddleTwoEdgeModeTypeAssertion",
                    ReserveTypeShape::QualifiedSymbolWithArgs("BaseTwoEdgeModeTypeAssertion"),
                ),
                exact_modes()[2],
            ],
            vec![
                exact_modes()[0],
                exact_modes()[1],
                mode_definition(
                    "OuterTwoEdgeModeTypeAssertion",
                    ReserveTypeShape::QualifiedSymbolWithArgs("MiddleTwoEdgeModeTypeAssertion"),
                ),
            ],
            vec![
                mode_definition(
                    "BaseTwoEdgeModeTypeAssertion",
                    ReserveTypeShape::AttributedSet,
                ),
                exact_modes()[1],
                exact_modes()[2],
            ],
            vec![
                mode_definition(
                    "BaseTwoEdgeModeTypeAssertion",
                    ReserveTypeShape::Builtin("object"),
                ),
                exact_modes()[1],
                exact_modes()[2],
            ],
            vec![exact_modes()[2], exact_modes()[1], exact_modes()[0]],
            vec![
                exact_modes()[0],
                mode_definition_with_label(
                    "OuterTwoEdgeModeTypeAssertion",
                    "OuterTwoEdgeModeTypeAssertionDef",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeModeTypeAssertion"),
                ),
            ],
            vec![
                exact_modes()[0],
                exact_modes()[1],
                mode_definition_with_label(
                    "OuterTwoEdgeModeTypeAssertion",
                    "OuterTwoEdgeModeTypeAssertionDef",
                    ReserveTypeShape::Builtin("set"),
                ),
            ],
            vec![
                mode_definition_with_label(
                    "BaseTwoEdgeModeTypeAssertion",
                    "BaseTwoEdgeModeTypeAssertionDef",
                    ReserveTypeShape::QualifiedSymbol("OuterTwoEdgeModeTypeAssertion"),
                ),
                exact_modes()[1],
                exact_modes()[2],
            ],
            vec![
                exact_modes()[0],
                mode_definition_with_label(
                    "ExtraTwoEdgeModeTypeAssertion",
                    "ExtraTwoEdgeModeTypeAssertionDef",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeModeTypeAssertion"),
                ),
                mode_definition_with_label(
                    "MiddleTwoEdgeModeTypeAssertion",
                    "MiddleTwoEdgeModeTypeAssertionDef",
                    ReserveTypeShape::QualifiedSymbol("ExtraTwoEdgeModeTypeAssertion"),
                ),
                exact_modes()[2],
            ],
        ];
        for modes in near_miss_modes {
            let near_miss = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                modes,
                reserve(),
                theorem,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }

        for near_miss in [
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbolWithArgs("OuterTwoEdgeModeTypeAssertion"),
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
                    asserted_type: ReserveTypeShape::QualifiedSymbol(
                        "OuterTwoEdgeModeTypeAssertion",
                    ),
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
    }

    #[test]
    fn source_two_edge_local_object_mode_reserved_variable_type_assertion_consumes_three_expansions()
     {
        let source_id = source_id(149);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("two_edge_local_object_mode_reserved_variable_type_assertion"),
        );
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseTwoEdgeObjectModeTypeAssertion", SymbolKind::Mode),
                ("MiddleTwoEdgeObjectModeTypeAssertion", SymbolKind::Mode),
                ("OuterTwoEdgeObjectModeTypeAssertion", SymbolKind::Mode),
                ("ExtraTwoEdgeObjectModeTypeAssertion", SymbolKind::Mode),
            ],
        );
        let theorem = exact_two_edge_local_object_mode_identifier_type_assertion_spec();
        let exact_modes = || {
            vec![
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
            ]
        };
        let reserve = || {
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("OuterTwoEdgeObjectModeTypeAssertion"),
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
        let payload = extract_source_two_edge_local_object_mode_reserved_variable_type_assertion(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact two-edge local-object-mode reserved-variable type assertion should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 3);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(
            payload.reserve.bridge.bindings()[0].type_spelling,
            "OuterTwoEdgeObjectModeTypeAssertion"
        );
        assert_eq!(payload.asserted_type.spelling, "object");
        assert_eq!(payload.asserted_type.head, TypeHeadInput::BuiltinObject);
        assert_ne!(
            payload.reserve.bridge.bindings()[0].type_range,
            payload.asserted_type.range
        );
        assert_eq!(payload.subject_lookup_ordinal, 1);

        let output = source_two_edge_local_object_mode_reserved_variable_type_assertion_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact two-edge local-object-mode type assertion should reach TermFormulaChecker");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("two-edge local-object-mode type assertion invariants should hold");
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(
            output.subject_result_input.spelling,
            "OuterTwoEdgeObjectModeTypeAssertion"
        );
        assert!(matches!(
            output.subject_result_input.head,
            TypeHeadInput::Symbol(_)
        ));
        assert_eq!(output.asserted_type_input.spelling, "object");
        assert_eq!(
            output.asserted_type_input.head,
            TypeHeadInput::BuiltinObject
        );
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("BaseTwoEdgeObjectModeTypeAssertion")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("base mode terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("one normalized builtin-object type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
        let (_, formula) = output.term_formula.formulas().iter().next().unwrap();
        assert_eq!(formula.kind, FormulaKind::TypeAssertion);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());

        for removed in [
            "BaseTwoEdgeObjectModeTypeAssertion",
            "MiddleTwoEdgeObjectModeTypeAssertion",
            "OuterTwoEdgeObjectModeTypeAssertion",
        ] {
            let mut invalid =
                source_two_edge_local_object_mode_reserved_variable_type_assertion_output(
                    &exact,
                    module.clone(),
                    &symbols,
                )
                .expect("exact source should produce a three-expansion corruption target");
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
                    TYPE_ELABORATION_TWO_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY,
                ),
                vec![
                    TYPE_ELABORATION_TWO_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY
                        .to_owned()
                ]
            );
        }

        let near_miss_modes = [
            vec![exact_modes()[1], exact_modes()[2]],
            vec![exact_modes()[0], exact_modes()[2]],
            vec![exact_modes()[0], exact_modes()[1]],
            {
                let mut modes = exact_modes();
                modes.push(modes[0]);
                modes
            },
            vec![
                mode_definition_with_label(
                    "BaseTwoEdgeObjectModeTypeAssertion",
                    "OtherDef",
                    ReserveTypeShape::Builtin("object"),
                ),
                exact_modes()[1],
                exact_modes()[2],
            ],
            vec![
                exact_modes()[0],
                mode_definition_with_label(
                    "MiddleTwoEdgeObjectModeTypeAssertion",
                    "OtherDef",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeObjectModeTypeAssertion"),
                ),
                exact_modes()[2],
            ],
            vec![
                exact_modes()[0],
                exact_modes()[1],
                mode_definition_with_label(
                    "OuterTwoEdgeObjectModeTypeAssertion",
                    "OtherDef",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeObjectModeTypeAssertion"),
                ),
            ],
            vec![
                recovered_mode_definition(
                    "BaseTwoEdgeObjectModeTypeAssertion",
                    ReserveTypeShape::Builtin("object"),
                ),
                exact_modes()[1],
                exact_modes()[2],
            ],
            vec![
                exact_modes()[0],
                recovered_mode_definition(
                    "MiddleTwoEdgeObjectModeTypeAssertion",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeObjectModeTypeAssertion"),
                ),
                exact_modes()[2],
            ],
            vec![
                exact_modes()[0],
                exact_modes()[1],
                recovered_mode_definition(
                    "OuterTwoEdgeObjectModeTypeAssertion",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeObjectModeTypeAssertion"),
                ),
            ],
            vec![
                contextual_mode_definition(
                    "BaseTwoEdgeObjectModeTypeAssertion",
                    ReserveTypeShape::Builtin("object"),
                ),
                exact_modes()[1],
                exact_modes()[2],
            ],
            vec![
                exact_modes()[0],
                parameterized_mode_definition(
                    "MiddleTwoEdgeObjectModeTypeAssertion",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeObjectModeTypeAssertion"),
                ),
                exact_modes()[2],
            ],
            vec![
                exact_modes()[0],
                exact_modes()[1],
                parameterized_mode_definition(
                    "OuterTwoEdgeObjectModeTypeAssertion",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeObjectModeTypeAssertion"),
                ),
            ],
            vec![
                exact_modes()[0],
                mode_definition(
                    "MiddleTwoEdgeObjectModeTypeAssertion",
                    ReserveTypeShape::QualifiedSymbolWithArgs("BaseTwoEdgeObjectModeTypeAssertion"),
                ),
                exact_modes()[2],
            ],
            vec![
                exact_modes()[0],
                exact_modes()[1],
                mode_definition(
                    "OuterTwoEdgeObjectModeTypeAssertion",
                    ReserveTypeShape::QualifiedSymbolWithArgs(
                        "MiddleTwoEdgeObjectModeTypeAssertion",
                    ),
                ),
            ],
            vec![
                mode_definition(
                    "BaseTwoEdgeObjectModeTypeAssertion",
                    ReserveTypeShape::AttributedObject,
                ),
                exact_modes()[1],
                exact_modes()[2],
            ],
            vec![
                mode_definition(
                    "BaseTwoEdgeObjectModeTypeAssertion",
                    ReserveTypeShape::Builtin("set"),
                ),
                exact_modes()[1],
                exact_modes()[2],
            ],
            vec![exact_modes()[2], exact_modes()[1], exact_modes()[0]],
            vec![
                exact_modes()[0],
                mode_definition_with_label(
                    "MiddleTwoEdgeObjectModeTypeAssertion",
                    "MiddleTwoEdgeObjectModeTypeAssertionDef",
                    ReserveTypeShape::QualifiedSymbol("ExtraTwoEdgeObjectModeTypeAssertion"),
                ),
                exact_modes()[2],
            ],
            vec![
                exact_modes()[0],
                mode_definition_with_label(
                    "OuterTwoEdgeObjectModeTypeAssertion",
                    "OuterTwoEdgeObjectModeTypeAssertionDef",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeObjectModeTypeAssertion"),
                ),
            ],
            vec![
                exact_modes()[0],
                exact_modes()[1],
                mode_definition_with_label(
                    "OuterTwoEdgeObjectModeTypeAssertion",
                    "OuterTwoEdgeObjectModeTypeAssertionDef",
                    ReserveTypeShape::Builtin("object"),
                ),
            ],
            vec![
                mode_definition_with_label(
                    "BaseTwoEdgeObjectModeTypeAssertion",
                    "BaseTwoEdgeObjectModeTypeAssertionDef",
                    ReserveTypeShape::QualifiedSymbol("OuterTwoEdgeObjectModeTypeAssertion"),
                ),
                exact_modes()[1],
                exact_modes()[2],
            ],
            vec![
                exact_modes()[0],
                mode_definition_with_label(
                    "ExtraTwoEdgeObjectModeTypeAssertion",
                    "ExtraTwoEdgeObjectModeTypeAssertionDef",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeObjectModeTypeAssertion"),
                ),
                mode_definition_with_label(
                    "MiddleTwoEdgeObjectModeTypeAssertion",
                    "MiddleTwoEdgeObjectModeTypeAssertionDef",
                    ReserveTypeShape::QualifiedSymbol("ExtraTwoEdgeObjectModeTypeAssertion"),
                ),
                exact_modes()[2],
            ],
        ];
        for modes in near_miss_modes {
            let near_miss = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                modes,
                reserve(),
                theorem,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }

        for near_miss in [
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbolWithArgs(
                        "OuterTwoEdgeObjectModeTypeAssertion",
                    ),
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
                    asserted_type: ReserveTypeShape::QualifiedSymbol(
                        "OuterTwoEdgeObjectModeTypeAssertion",
                    ),
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
    }

    #[test]
    fn source_two_edge_local_mode_asserted_head_type_assertion_consumes_three_expansions() {
        let source_id = source_id(186);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("two_edge_local_mode_asserted_head_type_assertion"),
        );
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseTwoEdgeModeAssertedHead", SymbolKind::Mode),
                ("MiddleTwoEdgeModeAssertedHead", SymbolKind::Mode),
                ("OuterTwoEdgeModeAssertedHead", SymbolKind::Mode),
                ("ExtraTwoEdgeModeAssertedHead", SymbolKind::Mode),
                ("OtherTwoEdgeModeAssertedHead", SymbolKind::Mode),
            ],
        );
        let theorem = exact_two_edge_local_mode_asserted_head_spec();
        let exact_modes = || {
            vec![
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
            ]
        };
        let reserve = || {
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("OuterTwoEdgeModeAssertedHead"),
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
        let payload = extract_source_two_edge_local_mode_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact two-edge local-mode same-outer asserted-head type assertion should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 3);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        let source_binding = &payload.reserve.bridge.bindings()[0];
        assert_eq!(source_binding.type_spelling, "OuterTwoEdgeModeAssertedHead");
        assert_eq!(
            payload.asserted_type.spelling,
            "OuterTwoEdgeModeAssertedHead"
        );
        assert!(matches!(source_binding.type_head, TypeHeadInput::Symbol(_)));
        assert_eq!(payload.asserted_type.head, source_binding.type_head);
        assert_ne!(source_binding.type_range, payload.asserted_type.range);
        assert_eq!(payload.subject_lookup_ordinal, 1);

        let output =
            source_two_edge_local_mode_asserted_head_output(&exact, module.clone(), &symbols)
                .expect("exact two-edge local-mode type assertion should reach TermFormulaChecker");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("two-edge local-mode type assertion invariants should hold");
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(
            output.subject_result_input.spelling,
            "OuterTwoEdgeModeAssertedHead"
        );
        assert_eq!(
            output.asserted_type_input.spelling,
            "OuterTwoEdgeModeAssertedHead"
        );
        assert_eq!(
            output.subject_result_input.head,
            output.asserted_type_input.head
        );
        assert!(matches!(
            output.asserted_type_input.head,
            TypeHeadInput::Symbol(_)
        ));
        assert_ne!(
            output.subject_result_input.source_range,
            output.asserted_type_input.source_range
        );
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("BaseTwoEdgeModeAssertedHead")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("base mode terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("one normalized builtin-set type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
        let (_, term) = output
            .term_formula
            .terms()
            .iter()
            .next()
            .expect("reserved-variable subject should be inferred");
        assert_eq!(term.status, TermStatus::Inferred);
        assert!(term.deferred.is_empty());
        let (_, formula) = output.term_formula.formulas().iter().next().unwrap();
        assert_eq!(formula.kind, FormulaKind::TypeAssertion);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert!(formula.expected_types.is_empty());
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());

        for removed in [
            "BaseTwoEdgeModeAssertedHead",
            "MiddleTwoEdgeModeAssertedHead",
            "OuterTwoEdgeModeAssertedHead",
        ] {
            let mut invalid =
                source_two_edge_local_mode_asserted_head_output(&exact, module.clone(), &symbols)
                    .expect("exact source should produce a three-expansion corruption target");
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
                    TYPE_ELABORATION_TWO_EDGE_LOCAL_MODE_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
                ),
                vec![
                    TYPE_ELABORATION_TWO_EDGE_LOCAL_MODE_ASSERTED_HEAD_INVALID_PAYLOAD_KEY
                        .to_owned()
                ]
            );
        }

        let mut wrong_asserted_spelling =
            source_two_edge_local_mode_asserted_head_output(&exact, module.clone(), &symbols)
                .expect("exact source should produce an asserted-spelling corruption target");
        wrong_asserted_spelling.asserted_type_input.spelling =
            "OtherTwoEdgeModeAssertedHead".to_owned();
        let mut wrong_asserted_head =
            source_two_edge_local_mode_asserted_head_output(&exact, module.clone(), &symbols)
                .expect("exact source should produce an asserted-head corruption target");
        wrong_asserted_head.asserted_type_input.head = TypeHeadInput::BuiltinSet;
        let mut collapsed_ranges =
            source_two_edge_local_mode_asserted_head_output(&exact, module.clone(), &symbols)
                .expect("exact source should produce a range corruption target");
        collapsed_ranges.asserted_type_input.source_range =
            collapsed_ranges.subject_result_input.source_range;
        let mut wrong_subject_head =
            source_two_edge_local_mode_asserted_head_output(&exact, module.clone(), &symbols)
                .expect("exact source should produce a subject-head corruption target");
        wrong_subject_head.subject_result_input.head = TypeHeadInput::BuiltinObject;
        for invalid in [
            wrong_asserted_spelling,
            wrong_asserted_head,
            collapsed_ranges,
            wrong_subject_head,
        ] {
            let invalid_result =
                assert_source_reserved_variable_type_assertion_output(&invalid).map(|()| invalid);
            assert_eq!(
                source_reserved_variable_type_assertion_result_detail_keys(
                    invalid_result,
                    TYPE_ELABORATION_TWO_EDGE_LOCAL_MODE_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
                ),
                vec![
                    TYPE_ELABORATION_TWO_EDGE_LOCAL_MODE_ASSERTED_HEAD_INVALID_PAYLOAD_KEY
                        .to_owned()
                ]
            );
        }

        let near_miss_modes = [
            Vec::<ModeDefinitionSpec>::new(),
            vec![exact_modes()[1], exact_modes()[2]],
            vec![exact_modes()[0], exact_modes()[2]],
            vec![exact_modes()[0], exact_modes()[1]],
            {
                let mut modes = exact_modes();
                modes.push(modes[0]);
                modes
            },
            vec![
                mode_definition_with_label(
                    "BaseTwoEdgeModeAssertedHead",
                    "OtherDef",
                    ReserveTypeShape::Builtin("set"),
                ),
                exact_modes()[1],
                exact_modes()[2],
            ],
            vec![
                exact_modes()[0],
                mode_definition_with_label(
                    "MiddleTwoEdgeModeAssertedHead",
                    "OtherDef",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeModeAssertedHead"),
                ),
                exact_modes()[2],
            ],
            vec![
                exact_modes()[0],
                exact_modes()[1],
                mode_definition_with_label(
                    "OuterTwoEdgeModeAssertedHead",
                    "OtherDef",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeModeAssertedHead"),
                ),
            ],
            vec![
                recovered_mode_definition(
                    "BaseTwoEdgeModeAssertedHead",
                    ReserveTypeShape::Builtin("set"),
                ),
                exact_modes()[1],
                exact_modes()[2],
            ],
            vec![
                exact_modes()[0],
                recovered_mode_definition(
                    "MiddleTwoEdgeModeAssertedHead",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeModeAssertedHead"),
                ),
                exact_modes()[2],
            ],
            vec![
                exact_modes()[0],
                exact_modes()[1],
                recovered_mode_definition(
                    "OuterTwoEdgeModeAssertedHead",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeModeAssertedHead"),
                ),
            ],
            vec![
                contextual_mode_definition(
                    "BaseTwoEdgeModeAssertedHead",
                    ReserveTypeShape::Builtin("set"),
                ),
                exact_modes()[1],
                exact_modes()[2],
            ],
            vec![
                exact_modes()[0],
                parameterized_mode_definition(
                    "MiddleTwoEdgeModeAssertedHead",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeModeAssertedHead"),
                ),
                exact_modes()[2],
            ],
            vec![
                exact_modes()[0],
                exact_modes()[1],
                parameterized_mode_definition(
                    "OuterTwoEdgeModeAssertedHead",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeModeAssertedHead"),
                ),
            ],
            vec![
                exact_modes()[0],
                mode_definition(
                    "MiddleTwoEdgeModeAssertedHead",
                    ReserveTypeShape::QualifiedSymbolWithArgs("BaseTwoEdgeModeAssertedHead"),
                ),
                exact_modes()[2],
            ],
            vec![
                exact_modes()[0],
                exact_modes()[1],
                mode_definition(
                    "OuterTwoEdgeModeAssertedHead",
                    ReserveTypeShape::QualifiedSymbolWithArgs("MiddleTwoEdgeModeAssertedHead"),
                ),
            ],
            vec![
                mode_definition(
                    "BaseTwoEdgeModeAssertedHead",
                    ReserveTypeShape::AttributedSet,
                ),
                exact_modes()[1],
                exact_modes()[2],
            ],
            vec![
                mode_definition(
                    "BaseTwoEdgeModeAssertedHead",
                    ReserveTypeShape::Builtin("object"),
                ),
                exact_modes()[1],
                exact_modes()[2],
            ],
            vec![exact_modes()[2], exact_modes()[1], exact_modes()[0]],
            vec![
                exact_modes()[0],
                mode_definition_with_label(
                    "OuterTwoEdgeModeAssertedHead",
                    "OuterTwoEdgeModeAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeModeAssertedHead"),
                ),
            ],
            vec![
                exact_modes()[0],
                exact_modes()[1],
                mode_definition_with_label(
                    "OuterTwoEdgeModeAssertedHead",
                    "OuterTwoEdgeModeAssertedHeadDef",
                    ReserveTypeShape::Builtin("set"),
                ),
            ],
            vec![
                mode_definition_with_label(
                    "BaseTwoEdgeModeAssertedHead",
                    "BaseTwoEdgeModeAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol("OuterTwoEdgeModeAssertedHead"),
                ),
                exact_modes()[1],
                exact_modes()[2],
            ],
            vec![
                exact_modes()[0],
                mode_definition_with_label(
                    "ExtraTwoEdgeModeAssertedHead",
                    "ExtraTwoEdgeModeAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeModeAssertedHead"),
                ),
                mode_definition_with_label(
                    "MiddleTwoEdgeModeAssertedHead",
                    "MiddleTwoEdgeModeAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol("ExtraTwoEdgeModeAssertedHead"),
                ),
                exact_modes()[2],
            ],
        ];
        for modes in near_miss_modes {
            let near_miss = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                modes,
                reserve(),
                theorem,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }

        for near_miss in [
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbolWithArgs("OuterTwoEdgeModeAssertedHead"),
                )],
                theorem,
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::AttributedQualifiedSymbol("OuterTwoEdgeModeAssertedHead"),
                )],
                theorem,
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                vec![
                    reserve_item(
                        vec!["x"],
                        ReserveTypeShape::QualifiedSymbol("OuterTwoEdgeModeAssertedHead"),
                    ),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                ],
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
                    asserted_type: ReserveTypeShape::Builtin("object"),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeModeAssertedHead"),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(
                        "MiddleTwoEdgeModeAssertedHead",
                    ),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(
                        "OtherTwoEdgeModeAssertedHead",
                    ),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbolWithArgs(
                        "OuterTwoEdgeModeAssertedHead",
                    ),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::AttributedQualifiedSymbol(
                        "OuterTwoEdgeModeAssertedHead",
                    ),
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

        let imported_outer_symbols = source_local_and_imported_symbols_env(
            module.clone(),
            &[
                ("BaseTwoEdgeModeAssertedHead", SymbolKind::Mode),
                ("MiddleTwoEdgeModeAssertedHead", SymbolKind::Mode),
            ],
            &[("OuterTwoEdgeModeAssertedHead", SymbolKind::Mode)],
        );
        let imported_base_symbols = source_local_and_imported_symbols_env(
            module.clone(),
            &[
                ("MiddleTwoEdgeModeAssertedHead", SymbolKind::Mode),
                ("OuterTwoEdgeModeAssertedHead", SymbolKind::Mode),
            ],
            &[("BaseTwoEdgeModeAssertedHead", SymbolKind::Mode)],
        );
        let imported_middle_symbols = source_local_and_imported_symbols_env(
            module.clone(),
            &[
                ("BaseTwoEdgeModeAssertedHead", SymbolKind::Mode),
                ("OuterTwoEdgeModeAssertedHead", SymbolKind::Mode),
            ],
            &[("MiddleTwoEdgeModeAssertedHead", SymbolKind::Mode)],
        );
        for provenance_near_miss_symbols in [
            &imported_outer_symbols,
            &imported_base_symbols,
            &imported_middle_symbols,
        ] {
            let provenance_near_miss = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                theorem,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(
                    &provenance_near_miss,
                    module.clone(),
                    provenance_near_miss_symbols,
                ),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }

        for (asserted_spelling, asserted_head_symbols) in [
            (
                "ImportedTwoEdgeModeAssertedHead",
                source_local_and_imported_symbols_env(
                    module.clone(),
                    &[
                        ("BaseTwoEdgeModeAssertedHead", SymbolKind::Mode),
                        ("MiddleTwoEdgeModeAssertedHead", SymbolKind::Mode),
                        ("OuterTwoEdgeModeAssertedHead", SymbolKind::Mode),
                    ],
                    &[("ImportedTwoEdgeModeAssertedHead", SymbolKind::Mode)],
                ),
            ),
            (
                "AmbiguousTwoEdgeModeAssertedHead",
                source_local_and_imported_symbols_env(
                    module.clone(),
                    &[
                        ("BaseTwoEdgeModeAssertedHead", SymbolKind::Mode),
                        ("MiddleTwoEdgeModeAssertedHead", SymbolKind::Mode),
                        ("OuterTwoEdgeModeAssertedHead", SymbolKind::Mode),
                    ],
                    &[
                        ("AmbiguousTwoEdgeModeAssertedHead", SymbolKind::Mode),
                        ("AmbiguousTwoEdgeModeAssertedHead", SymbolKind::Mode),
                    ],
                ),
            ),
        ] {
            let asserted_head_near_miss = mode_then_reserve_identifier_type_assertion_theorem_ast(
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
                    &asserted_head_near_miss,
                    module.clone(),
                    &asserted_head_symbols,
                ),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
    }

    #[test]
    fn source_three_edge_local_mode_asserted_head_type_assertion_consumes_four_expansions() {
        let source_id = source_id(195);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("three_edge_local_mode_asserted_head_type_assertion"),
        );
        let mode_names = [
            "BaseThreeEdgeModeAssertedHead",
            "InnerThreeEdgeModeAssertedHead",
            "MiddleThreeEdgeModeAssertedHead",
            "OuterThreeEdgeModeAssertedHead",
        ];
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                (mode_names[0], SymbolKind::Mode),
                (mode_names[1], SymbolKind::Mode),
                (mode_names[2], SymbolKind::Mode),
                (mode_names[3], SymbolKind::Mode),
                ("ExtraThreeEdgeModeAssertedHead", SymbolKind::Mode),
                ("OtherThreeEdgeModeAssertedHead", SymbolKind::Mode),
            ],
        );
        let theorem = IdentifierTypeAssertionTheoremSpec {
            status: None,
            label: "ThreeEdgeLocalModeAssertedHeadPayloadBoundary",
            subject: "x",
            asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[3]),
            negated: false,
            recovered_label: false,
        };
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    mode_names[0],
                    "BaseThreeEdgeModeAssertedHeadDef",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition_with_label(
                    mode_names[1],
                    "InnerThreeEdgeModeAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(mode_names[0]),
                ),
                mode_definition_with_label(
                    mode_names[2],
                    "MiddleThreeEdgeModeAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(mode_names[1]),
                ),
                mode_definition_with_label(
                    mode_names[3],
                    "OuterThreeEdgeModeAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(mode_names[2]),
                ),
            ]
        };
        let reserve = || {
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol(mode_names[3]),
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
            super::extract_source_two_edge_local_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none()
        );
        assert!(
            super::extract_source_three_edge_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none()
        );
        let payload = super::extract_source_three_edge_local_mode_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact three-edge same-outer asserted-head source should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 4);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(payload.subject_lookup_ordinal, 1);
        let source_binding = &payload.reserve.bridge.bindings()[0];
        assert_eq!(source_binding.type_spelling, mode_names[3]);
        assert_eq!(payload.asserted_type.spelling, mode_names[3]);
        assert_eq!(payload.asserted_type.head, source_binding.type_head);
        assert!(matches!(source_binding.type_head, TypeHeadInput::Symbol(_)));
        assert_ne!(payload.asserted_type_site, payload.subject_site);
        assert_ne!(payload.asserted_type.range, source_binding.type_range);

        let output = super::source_three_edge_local_mode_asserted_head_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact three-edge asserted-head source should reach TermFormulaChecker");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 195 checker payload invariants should hold");
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.payload.reserve.mode_expansions.len(), 4);
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert_eq!(output.subject_result_input.spelling, mode_names[3]);
        assert_eq!(output.asserted_type_input.spelling, mode_names[3]);
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
            .expect("base mode terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("one normalized builtin-set type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
        let (_, term) = output
            .term_formula
            .terms()
            .iter()
            .next()
            .expect("reserved-variable subject should be inferred");
        assert_eq!(term.status, TermStatus::Inferred);
        assert!(term.deferred.is_empty());
        let (_, formula) = output
            .term_formula
            .formulas()
            .iter()
            .next()
            .expect("checked type assertion should exist");
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
                    super::TYPE_ELABORATION_THREE_EDGE_LOCAL_MODE_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
                ),
                vec![
                    super::TYPE_ELABORATION_THREE_EDGE_LOCAL_MODE_ASSERTED_HEAD_INVALID_PAYLOAD_KEY
                        .to_owned()
                ]
            );
        };
        for removed in mode_names {
            let mut invalid = super::source_three_edge_local_mode_asserted_head_output(
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
            assert_invalid_output(invalid);
        }
        let mut wrong_binding = super::source_three_edge_local_mode_asserted_head_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .unwrap();
        wrong_binding.subject_binding = BindingId::new(1);
        let mut wrong_ordinal = super::source_three_edge_local_mode_asserted_head_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .unwrap();
        wrong_ordinal.payload.subject_lookup_ordinal = 2;
        let mut wrong_asserted_spelling = super::source_three_edge_local_mode_asserted_head_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .unwrap();
        wrong_asserted_spelling.asserted_type_input.spelling =
            "OtherThreeEdgeModeAssertedHead".to_owned();
        let mut wrong_asserted_head = super::source_three_edge_local_mode_asserted_head_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .unwrap();
        wrong_asserted_head.asserted_type_input.head = TypeHeadInput::BuiltinObject;
        let mut wrong_subject_head = super::source_three_edge_local_mode_asserted_head_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .unwrap();
        wrong_subject_head.subject_result_input.head = TypeHeadInput::BuiltinObject;
        let mut collapsed_range = super::source_three_edge_local_mode_asserted_head_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .unwrap();
        collapsed_range.asserted_type_input.source_range =
            collapsed_range.subject_result_input.source_range;
        let mut collapsed_site = super::source_three_edge_local_mode_asserted_head_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .unwrap();
        collapsed_site.asserted_type_input.site = collapsed_site.subject_result_input.site.clone();
        let mut wrong_canonical_source = super::source_three_edge_local_mode_asserted_head_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .unwrap();
        let (_, base_expansion) = wrong_canonical_source
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[0]))
            .unwrap();
        base_expansion.radix.source_range = range(source_id, 0, 1);
        for invalid in [
            wrong_binding,
            wrong_ordinal,
            wrong_asserted_spelling,
            wrong_asserted_head,
            wrong_subject_head,
            collapsed_range,
            collapsed_site,
            wrong_canonical_source,
        ] {
            assert_invalid_output(invalid);
        }

        let mut mode_near_misses = vec![
            Vec::<ModeDefinitionSpec>::new(),
            vec![
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
                modes[0].rhs_shape = ReserveTypeShape::Builtin("object");
                modes
            },
            {
                let mut modes = exact_modes();
                modes[1].rhs_shape = ReserveTypeShape::QualifiedSymbol(mode_names[3]);
                modes
            },
            {
                let mut modes = exact_modes();
                modes[2].rhs_shape = ReserveTypeShape::QualifiedSymbol(mode_names[0]);
                modes
            },
            {
                let mut modes = exact_modes();
                modes[3].rhs_shape = ReserveTypeShape::QualifiedSymbol(mode_names[1]);
                modes
            },
            {
                let mut modes = exact_modes();
                modes[0].rhs_shape = ReserveTypeShape::AttributedSet;
                modes
            },
            {
                let mut modes = exact_modes();
                modes[1].rhs_shape = ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[0]);
                modes
            },
        ];
        for index in 0..4 {
            let mut missing = exact_modes();
            missing.remove(index);
            mode_near_misses.push(missing);
            let mut wrong_label = exact_modes();
            wrong_label[index].label = Some("WrongThreeEdgeModeAssertedHeadDef");
            mode_near_misses.push(wrong_label);
            let mut recovered = exact_modes();
            recovered[index].recovered = true;
            mode_near_misses.push(recovered);
        }
        let mut contextual = exact_modes();
        contextual[0].local_context = true;
        mode_near_misses.push(contextual);
        let mut parameterized = exact_modes();
        parameterized[2].parameterized_pattern = true;
        mode_near_misses.push(parameterized);
        for modes in mode_near_misses {
            let near_miss = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                modes,
                reserve(),
                theorem,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }

        let source_near_misses = [
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[3]),
                )],
                theorem,
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::AttributedQualifiedSymbol(mode_names[3]),
                )],
                theorem,
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::QualifiedSymbol(mode_names[3])),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                ],
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
                    asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[0]),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[1]),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[2]),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(
                        "OtherThreeEdgeModeAssertedHead",
                    ),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[3]),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::AttributedQualifiedSymbol(mode_names[3]),
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
        ];
        for near_miss in source_near_misses {
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
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
                source_type_elaboration_detail_keys(&exact, module.clone(), &imported_symbols,),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }

        for (asserted_spelling, asserted_head_symbols) in [
            (
                "ImportedThreeEdgeModeAssertedHead",
                source_local_and_imported_symbols_env(
                    module.clone(),
                    &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
                    &[("ImportedThreeEdgeModeAssertedHead", SymbolKind::Mode)],
                ),
            ),
            (
                "AmbiguousThreeEdgeModeAssertedHead",
                source_local_and_imported_symbols_env(
                    module.clone(),
                    &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
                    &[
                        ("AmbiguousThreeEdgeModeAssertedHead", SymbolKind::Mode),
                        ("AmbiguousThreeEdgeModeAssertedHead", SymbolKind::Mode),
                    ],
                ),
            ),
        ] {
            let asserted_head_near_miss = mode_then_reserve_identifier_type_assertion_theorem_ast(
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
                    &asserted_head_near_miss,
                    module.clone(),
                    &asserted_head_symbols,
                ),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
    }

    #[test]
    fn source_four_edge_local_mode_asserted_head_type_assertion_consumes_five_expansions() {
        let source_id = source_id(197);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("four_edge_local_mode_asserted_head_type_assertion"),
        );
        let mode_names = [
            "BaseFourEdgeModeAssertedHead",
            "InnerFourEdgeModeAssertedHead",
            "MiddleFourEdgeModeAssertedHead",
            "OuterFourEdgeModeAssertedHead",
            "TooDeepFourEdgeModeAssertedHead",
        ];
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                (mode_names[0], SymbolKind::Mode),
                (mode_names[1], SymbolKind::Mode),
                (mode_names[2], SymbolKind::Mode),
                (mode_names[3], SymbolKind::Mode),
                (mode_names[4], SymbolKind::Mode),
                ("ExtraFourEdgeModeAssertedHead", SymbolKind::Mode),
                ("OtherFourEdgeModeAssertedHead", SymbolKind::Mode),
            ],
        );
        let theorem = IdentifierTypeAssertionTheoremSpec {
            status: None,
            label: "FourEdgeLocalModeAssertedHeadPayloadBoundary",
            subject: "x",
            asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[4]),
            negated: false,
            recovered_label: false,
        };
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    mode_names[0],
                    "BaseFourEdgeModeAssertedHeadDef",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition_with_label(
                    mode_names[1],
                    "InnerFourEdgeModeAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(mode_names[0]),
                ),
                mode_definition_with_label(
                    mode_names[2],
                    "MiddleFourEdgeModeAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(mode_names[1]),
                ),
                mode_definition_with_label(
                    mode_names[3],
                    "OuterFourEdgeModeAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(mode_names[2]),
                ),
                mode_definition_with_label(
                    mode_names[4],
                    "TooDeepFourEdgeModeAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(mode_names[3]),
                ),
            ]
        };
        let reserve = || {
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol(mode_names[4]),
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
            super::extract_source_three_edge_local_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none()
        );
        assert!(
            super::extract_source_four_edge_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none()
        );
        let payload = super::extract_source_four_edge_local_mode_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact four-edge same-outer asserted-head source should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 5);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(payload.subject_lookup_ordinal, 1);
        let source_binding = &payload.reserve.bridge.bindings()[0];
        assert_eq!(source_binding.type_spelling, mode_names[4]);
        assert_eq!(payload.asserted_type.spelling, mode_names[4]);
        assert_eq!(payload.asserted_type.head, source_binding.type_head);
        assert!(matches!(source_binding.type_head, TypeHeadInput::Symbol(_)));
        assert_ne!(payload.asserted_type_site, payload.subject_site);
        assert_ne!(payload.asserted_type.range, source_binding.type_range);

        let output = super::source_four_edge_local_mode_asserted_head_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact four-edge asserted-head source should reach TermFormulaChecker");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 197 checker payload invariants should hold");
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.payload.reserve.mode_expansions.len(), 5);
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert_eq!(output.subject_result_input.spelling, mode_names[4]);
        assert_eq!(output.asserted_type_input.spelling, mode_names[4]);
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
            .expect("base mode terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("one normalized builtin-set type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
        let (_, term) = output
            .term_formula
            .terms()
            .iter()
            .next()
            .expect("reserved-variable subject should be inferred");
        assert_eq!(term.status, TermStatus::Inferred);
        assert!(term.deferred.is_empty());
        let (_, formula) = output
            .term_formula
            .formulas()
            .iter()
            .next()
            .expect("checked type assertion should exist");
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
                    super::TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
                ),
                vec![
                    super::TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_ASSERTED_HEAD_INVALID_PAYLOAD_KEY
                        .to_owned()
                ]
            );
        };
        for removed in mode_names {
            let mut invalid = super::source_four_edge_local_mode_asserted_head_output(
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
            assert_invalid_output(invalid);
        }
        let mut wrong_binding = super::source_four_edge_local_mode_asserted_head_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .unwrap();
        wrong_binding.subject_binding = BindingId::new(1);
        let mut wrong_ordinal = super::source_four_edge_local_mode_asserted_head_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .unwrap();
        wrong_ordinal.payload.subject_lookup_ordinal = 2;
        let mut wrong_asserted_spelling = super::source_four_edge_local_mode_asserted_head_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .unwrap();
        wrong_asserted_spelling.asserted_type_input.spelling =
            "OtherFourEdgeModeAssertedHead".to_owned();
        let mut wrong_asserted_head = super::source_four_edge_local_mode_asserted_head_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .unwrap();
        wrong_asserted_head.asserted_type_input.head = TypeHeadInput::BuiltinObject;
        let mut wrong_subject_head = super::source_four_edge_local_mode_asserted_head_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .unwrap();
        wrong_subject_head.subject_result_input.head = TypeHeadInput::BuiltinObject;
        let mut collapsed_range = super::source_four_edge_local_mode_asserted_head_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .unwrap();
        collapsed_range.asserted_type_input.source_range =
            collapsed_range.subject_result_input.source_range;
        let mut collapsed_site = super::source_four_edge_local_mode_asserted_head_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .unwrap();
        collapsed_site.asserted_type_input.site = collapsed_site.subject_result_input.site.clone();
        let mut wrong_canonical_source = super::source_four_edge_local_mode_asserted_head_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .unwrap();
        let (_, base_expansion) = wrong_canonical_source
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[0]))
            .unwrap();
        base_expansion.radix.source_range = range(source_id, 0, 1);
        for invalid in [
            wrong_binding,
            wrong_ordinal,
            wrong_asserted_spelling,
            wrong_asserted_head,
            wrong_subject_head,
            collapsed_range,
            collapsed_site,
            wrong_canonical_source,
        ] {
            assert_invalid_output(invalid);
        }

        let mut mode_near_misses = vec![
            Vec::<ModeDefinitionSpec>::new(),
            vec![
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
                    "ExtraFourEdgeModeAssertedHead",
                    "ExtraFourEdgeModeAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(mode_names[4]),
                ));
                modes
            },
            {
                let mut modes = exact_modes();
                modes[0].rhs_shape = ReserveTypeShape::Builtin("object");
                modes
            },
            {
                let mut modes = exact_modes();
                modes[1].rhs_shape = ReserveTypeShape::QualifiedSymbol(mode_names[4]);
                modes
            },
            {
                let mut modes = exact_modes();
                modes[2].rhs_shape = ReserveTypeShape::QualifiedSymbol(mode_names[0]);
                modes
            },
            {
                let mut modes = exact_modes();
                modes[3].rhs_shape = ReserveTypeShape::QualifiedSymbol(mode_names[1]);
                modes
            },
            {
                let mut modes = exact_modes();
                modes[4].rhs_shape = ReserveTypeShape::QualifiedSymbol(mode_names[2]);
                modes
            },
            {
                let mut modes = exact_modes();
                modes[0].rhs_shape = ReserveTypeShape::AttributedSet;
                modes
            },
            {
                let mut modes = exact_modes();
                modes[1].rhs_shape = ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[0]);
                modes
            },
        ];
        for index in 0..5 {
            let mut missing = exact_modes();
            missing.remove(index);
            mode_near_misses.push(missing);
            let mut wrong_label = exact_modes();
            wrong_label[index].label = Some("WrongFourEdgeModeAssertedHeadDef");
            mode_near_misses.push(wrong_label);
            let mut recovered = exact_modes();
            recovered[index].recovered = true;
            mode_near_misses.push(recovered);
        }
        let mut contextual = exact_modes();
        contextual[0].local_context = true;
        mode_near_misses.push(contextual);
        let mut parameterized = exact_modes();
        parameterized[2].parameterized_pattern = true;
        mode_near_misses.push(parameterized);
        for modes in mode_near_misses {
            let near_miss = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                modes,
                reserve(),
                theorem,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }

        let source_near_misses = [
            {
                let mut deeper_modes = exact_modes();
                deeper_modes.push(mode_definition_with_label(
                    "ExtraFourEdgeModeAssertedHead",
                    "ExtraFourEdgeModeAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(mode_names[4]),
                ));
                mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    deeper_modes,
                    vec![reserve_item(
                        vec!["x"],
                        ReserveTypeShape::QualifiedSymbol("ExtraFourEdgeModeAssertedHead"),
                    )],
                    IdentifierTypeAssertionTheoremSpec {
                        asserted_type: ReserveTypeShape::QualifiedSymbol(
                            "ExtraFourEdgeModeAssertedHead",
                        ),
                        ..theorem
                    },
                )
            },
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[4]),
                )],
                theorem,
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::AttributedQualifiedSymbol(mode_names[4]),
                )],
                theorem,
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::QualifiedSymbol(mode_names[4])),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                ],
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
                    asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[0]),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[1]),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[2]),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[3]),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(
                        "OtherFourEdgeModeAssertedHead",
                    ),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[4]),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::AttributedQualifiedSymbol(mode_names[4]),
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
        ];
        for near_miss in source_near_misses {
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
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
                source_type_elaboration_detail_keys(&exact, module.clone(), &imported_symbols,),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }

        for (asserted_spelling, asserted_head_symbols) in [
            (
                "ImportedFourEdgeModeAssertedHead",
                source_local_and_imported_symbols_env(
                    module.clone(),
                    &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
                    &[("ImportedFourEdgeModeAssertedHead", SymbolKind::Mode)],
                ),
            ),
            (
                "AmbiguousFourEdgeModeAssertedHead",
                source_local_and_imported_symbols_env(
                    module.clone(),
                    &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
                    &[
                        ("AmbiguousFourEdgeModeAssertedHead", SymbolKind::Mode),
                        ("AmbiguousFourEdgeModeAssertedHead", SymbolKind::Mode),
                    ],
                ),
            ),
        ] {
            let asserted_head_near_miss = mode_then_reserve_identifier_type_assertion_theorem_ast(
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
                    &asserted_head_near_miss,
                    module.clone(),
                    &asserted_head_symbols,
                ),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
    }

    #[test]
    fn source_four_edge_local_object_mode_asserted_head_type_assertion_consumes_five_expansions() {
        let source_id = source_id(198);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("four_edge_local_object_mode_asserted_head_type_assertion"),
        );
        let mode_names = [
            "BaseFourEdgeObjectModeAssertedHead",
            "InnerFourEdgeObjectModeAssertedHead",
            "MiddleFourEdgeObjectModeAssertedHead",
            "OuterFourEdgeObjectModeAssertedHead",
            "TooDeepFourEdgeObjectModeAssertedHead",
        ];
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                (mode_names[0], SymbolKind::Mode),
                (mode_names[1], SymbolKind::Mode),
                (mode_names[2], SymbolKind::Mode),
                (mode_names[3], SymbolKind::Mode),
                (mode_names[4], SymbolKind::Mode),
                ("ExtraFourEdgeObjectModeAssertedHead", SymbolKind::Mode),
                ("OtherFourEdgeObjectModeAssertedHead", SymbolKind::Mode),
            ],
        );
        let theorem = IdentifierTypeAssertionTheoremSpec {
            status: None,
            label: "FourEdgeLocalObjectModeAssertedHeadPayloadBoundary",
            subject: "x",
            asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[4]),
            negated: false,
            recovered_label: false,
        };
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    mode_names[0],
                    "BaseFourEdgeObjectModeAssertedHeadDef",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition_with_label(
                    mode_names[1],
                    "InnerFourEdgeObjectModeAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(mode_names[0]),
                ),
                mode_definition_with_label(
                    mode_names[2],
                    "MiddleFourEdgeObjectModeAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(mode_names[1]),
                ),
                mode_definition_with_label(
                    mode_names[3],
                    "OuterFourEdgeObjectModeAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(mode_names[2]),
                ),
                mode_definition_with_label(
                    mode_names[4],
                    "TooDeepFourEdgeObjectModeAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(mode_names[3]),
                ),
            ]
        };
        let reserve = || {
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol(mode_names[4]),
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
            super::extract_source_three_edge_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none()
        );
        assert!(
            super::extract_source_four_edge_local_object_mode_reserved_variable_type_assertion(
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
        let payload = super::extract_source_four_edge_local_object_mode_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact four-edge object same-head source should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 5);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(payload.subject_lookup_ordinal, 1);
        let source_binding = &payload.reserve.bridge.bindings()[0];
        assert_eq!(source_binding.type_spelling, mode_names[4]);
        assert_eq!(payload.asserted_type.spelling, mode_names[4]);
        assert_eq!(payload.asserted_type.head, source_binding.type_head);
        assert!(matches!(source_binding.type_head, TypeHeadInput::Symbol(_)));
        assert_ne!(payload.asserted_type_site, payload.subject_site);
        assert_ne!(payload.asserted_type.range, source_binding.type_range);

        let output = super::source_four_edge_local_object_mode_asserted_head_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact four-edge object asserted-head source should reach checker");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 198 checker payload invariants should hold");
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.payload.reserve.mode_expansions.len(), 5);
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert_eq!(output.subject_result_input.spelling, mode_names[4]);
        assert_eq!(output.asserted_type_input.spelling, mode_names[4]);
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
            .expect("base object terminal expansion should exist");
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
                    super::TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
                ),
                vec![super::TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_ASSERTED_HEAD_INVALID_PAYLOAD_KEY.to_owned()]
            );
        };
        for removed in mode_names {
            let mut invalid = super::source_four_edge_local_object_mode_asserted_head_output(
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
            super::source_four_edge_local_object_mode_asserted_head_output(
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
        invalid.asserted_type_input.spelling = "OtherFourEdgeObjectModeAssertedHead".to_owned();
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
                    "ExtraFourEdgeObjectModeAssertedHead",
                    "ExtraFourEdgeObjectModeAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(mode_names[4]),
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
            wrong_label[index].label = Some("WrongFourEdgeObjectModeAssertedHeadDef");
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
                &format!("mode near miss {index}"),
            );
        }

        let mut source_near_misses = Vec::new();
        let mut deeper_modes = exact_modes();
        deeper_modes.push(mode_definition_with_label(
            "ExtraFourEdgeObjectModeAssertedHead",
            "ExtraFourEdgeObjectModeAssertedHeadDef",
            ReserveTypeShape::QualifiedSymbol(mode_names[4]),
        ));
        source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            deeper_modes,
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("ExtraFourEdgeObjectModeAssertedHead"),
            )],
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(
                    "ExtraFourEdgeObjectModeAssertedHead",
                ),
                ..theorem
            },
        ));
        for bad_reserve in [
            ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[4]),
            ReserveTypeShape::AttributedQualifiedSymbol(mode_names[4]),
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
                reserve_item(vec!["x"], ReserveTypeShape::QualifiedSymbol(mode_names[4])),
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
                asserted_type: ReserveTypeShape::QualifiedSymbol(
                    "OtherFourEdgeObjectModeAssertedHead",
                ),
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
            &[("UnrelatedFourEdgeObjectModeAssertedHead", SymbolKind::Mode)],
        );
        assert!(
            super::extract_source_four_edge_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &unrelated_import,
            )
            .is_some()
        );
        for (asserted_spelling, asserted_head_symbols) in [
            (
                "ImportedFourEdgeObjectModeAssertedHead",
                source_local_and_imported_symbols_env(
                    module.clone(),
                    &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
                    &[("ImportedFourEdgeObjectModeAssertedHead", SymbolKind::Mode)],
                ),
            ),
            (
                "AmbiguousFourEdgeObjectModeAssertedHead",
                source_local_and_imported_symbols_env(
                    module.clone(),
                    &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
                    &[
                        ("AmbiguousFourEdgeObjectModeAssertedHead", SymbolKind::Mode),
                        ("AmbiguousFourEdgeObjectModeAssertedHead", SymbolKind::Mode),
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
    fn source_three_edge_local_object_mode_asserted_head_type_assertion_consumes_four_expansions() {
        let source_id = source_id(196);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("three_edge_local_object_mode_asserted_head_type_assertion"),
        );
        let mode_names = [
            "BaseThreeEdgeObjectModeAssertedHead",
            "InnerThreeEdgeObjectModeAssertedHead",
            "MiddleThreeEdgeObjectModeAssertedHead",
            "OuterThreeEdgeObjectModeAssertedHead",
        ];
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                (mode_names[0], SymbolKind::Mode),
                (mode_names[1], SymbolKind::Mode),
                (mode_names[2], SymbolKind::Mode),
                (mode_names[3], SymbolKind::Mode),
                ("ExtraThreeEdgeObjectModeAssertedHead", SymbolKind::Mode),
                ("OtherThreeEdgeObjectModeAssertedHead", SymbolKind::Mode),
            ],
        );
        let theorem = IdentifierTypeAssertionTheoremSpec {
            status: None,
            label: "ThreeEdgeLocalObjectModeAssertedHeadPayloadBoundary",
            subject: "x",
            asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[3]),
            negated: false,
            recovered_label: false,
        };
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    mode_names[0],
                    "BaseThreeEdgeObjectModeAssertedHeadDef",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition_with_label(
                    mode_names[1],
                    "InnerThreeEdgeObjectModeAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(mode_names[0]),
                ),
                mode_definition_with_label(
                    mode_names[2],
                    "MiddleThreeEdgeObjectModeAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(mode_names[1]),
                ),
                mode_definition_with_label(
                    mode_names[3],
                    "OuterThreeEdgeObjectModeAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(mode_names[2]),
                ),
            ]
        };
        let reserve = || {
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol(mode_names[3]),
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
            super::extract_source_two_edge_local_object_mode_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none()
        );
        assert!(
            super::extract_source_three_edge_local_object_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none()
        );
        let payload = super::extract_source_three_edge_local_object_mode_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact three-edge same-outer asserted-head source should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 4);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(payload.subject_lookup_ordinal, 1);
        let source_binding = &payload.reserve.bridge.bindings()[0];
        assert_eq!(source_binding.type_spelling, mode_names[3]);
        assert_eq!(payload.asserted_type.spelling, mode_names[3]);
        assert_eq!(payload.asserted_type.head, source_binding.type_head);
        assert!(matches!(source_binding.type_head, TypeHeadInput::Symbol(_)));
        assert_ne!(payload.asserted_type_site, payload.subject_site);
        assert_ne!(payload.asserted_type.range, source_binding.type_range);

        let output = super::source_three_edge_local_object_mode_asserted_head_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact three-edge asserted-head source should reach TermFormulaChecker");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 196 checker payload invariants should hold");
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.payload.reserve.mode_expansions.len(), 4);
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert_eq!(output.subject_result_input.spelling, mode_names[3]);
        assert_eq!(output.asserted_type_input.spelling, mode_names[3]);
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
            .expect("base mode terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("one normalized builtin-object type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
        let (_, term) = output
            .term_formula
            .terms()
            .iter()
            .next()
            .expect("reserved-variable subject should be inferred");
        assert_eq!(term.status, TermStatus::Inferred);
        assert!(term.deferred.is_empty());
        let (_, formula) = output
            .term_formula
            .formulas()
            .iter()
            .next()
            .expect("checked type assertion should exist");
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
                    super::TYPE_ELABORATION_THREE_EDGE_LOCAL_OBJECT_MODE_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
                ),
                vec![
                    super::TYPE_ELABORATION_THREE_EDGE_LOCAL_OBJECT_MODE_ASSERTED_HEAD_INVALID_PAYLOAD_KEY
                        .to_owned()
                ]
            );
        };
        for removed in mode_names {
            let mut invalid = super::source_three_edge_local_object_mode_asserted_head_output(
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
            assert_invalid_output(invalid);
        }
        let mut wrong_binding = super::source_three_edge_local_object_mode_asserted_head_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .unwrap();
        wrong_binding.subject_binding = BindingId::new(1);
        let mut wrong_ordinal = super::source_three_edge_local_object_mode_asserted_head_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .unwrap();
        wrong_ordinal.payload.subject_lookup_ordinal = 2;
        let mut wrong_asserted_spelling =
            super::source_three_edge_local_object_mode_asserted_head_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .unwrap();
        wrong_asserted_spelling.asserted_type_input.spelling =
            "OtherThreeEdgeObjectModeAssertedHead".to_owned();
        let mut wrong_asserted_head =
            super::source_three_edge_local_object_mode_asserted_head_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .unwrap();
        wrong_asserted_head.asserted_type_input.head = TypeHeadInput::BuiltinSet;
        let mut wrong_subject_head =
            super::source_three_edge_local_object_mode_asserted_head_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .unwrap();
        wrong_subject_head.subject_result_input.head = TypeHeadInput::BuiltinSet;
        let mut collapsed_range = super::source_three_edge_local_object_mode_asserted_head_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .unwrap();
        collapsed_range.asserted_type_input.source_range =
            collapsed_range.subject_result_input.source_range;
        let mut collapsed_site = super::source_three_edge_local_object_mode_asserted_head_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .unwrap();
        collapsed_site.asserted_type_input.site = collapsed_site.subject_result_input.site.clone();
        let mut wrong_canonical_source =
            super::source_three_edge_local_object_mode_asserted_head_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .unwrap();
        let (_, base_expansion) = wrong_canonical_source
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[0]))
            .unwrap();
        base_expansion.radix.source_range = range(source_id, 0, 1);
        for invalid in [
            wrong_binding,
            wrong_ordinal,
            wrong_asserted_spelling,
            wrong_asserted_head,
            wrong_subject_head,
            collapsed_range,
            collapsed_site,
            wrong_canonical_source,
        ] {
            assert_invalid_output(invalid);
        }

        let mut mode_near_misses = vec![
            Vec::<ModeDefinitionSpec>::new(),
            vec![
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
                modes[0].rhs_shape = ReserveTypeShape::Builtin("set");
                modes
            },
            {
                let mut modes = exact_modes();
                modes[1].rhs_shape = ReserveTypeShape::QualifiedSymbol(mode_names[3]);
                modes
            },
            {
                let mut modes = exact_modes();
                modes[2].rhs_shape = ReserveTypeShape::QualifiedSymbol(mode_names[0]);
                modes
            },
            {
                let mut modes = exact_modes();
                modes[3].rhs_shape = ReserveTypeShape::QualifiedSymbol(mode_names[1]);
                modes
            },
            {
                let mut modes = exact_modes();
                modes[0].rhs_shape = ReserveTypeShape::AttributedObject;
                modes
            },
            {
                let mut modes = exact_modes();
                modes[1].rhs_shape = ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[0]);
                modes
            },
        ];
        for index in 0..4 {
            let mut missing = exact_modes();
            missing.remove(index);
            mode_near_misses.push(missing);
            let mut wrong_label = exact_modes();
            wrong_label[index].label = Some("WrongThreeEdgeObjectModeAssertedHeadDef");
            mode_near_misses.push(wrong_label);
            let mut recovered = exact_modes();
            recovered[index].recovered = true;
            mode_near_misses.push(recovered);
        }
        let mut contextual = exact_modes();
        contextual[0].local_context = true;
        mode_near_misses.push(contextual);
        let mut parameterized = exact_modes();
        parameterized[2].parameterized_pattern = true;
        mode_near_misses.push(parameterized);
        for modes in mode_near_misses {
            let near_miss = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                modes,
                reserve(),
                theorem,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }

        let source_near_misses = [
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[3]),
                )],
                theorem,
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::AttributedQualifiedSymbol(mode_names[3]),
                )],
                theorem,
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::QualifiedSymbol(mode_names[3])),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("object")),
                ],
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
                    asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[0]),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[1]),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[2]),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(
                        "OtherThreeEdgeObjectModeAssertedHead",
                    ),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[3]),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::AttributedQualifiedSymbol(mode_names[3]),
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
        ];
        for near_miss in source_near_misses {
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
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
                source_type_elaboration_detail_keys(&exact, module.clone(), &imported_symbols,),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }

        for (asserted_spelling, asserted_head_symbols) in [
            (
                "ImportedThreeEdgeObjectModeAssertedHead",
                source_local_and_imported_symbols_env(
                    module.clone(),
                    &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
                    &[("ImportedThreeEdgeObjectModeAssertedHead", SymbolKind::Mode)],
                ),
            ),
            (
                "AmbiguousThreeEdgeObjectModeAssertedHead",
                source_local_and_imported_symbols_env(
                    module.clone(),
                    &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
                    &[
                        ("AmbiguousThreeEdgeObjectModeAssertedHead", SymbolKind::Mode),
                        ("AmbiguousThreeEdgeObjectModeAssertedHead", SymbolKind::Mode),
                    ],
                ),
            ),
        ] {
            let asserted_head_near_miss = mode_then_reserve_identifier_type_assertion_theorem_ast(
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
                    &asserted_head_near_miss,
                    module.clone(),
                    &asserted_head_symbols,
                ),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
    }

    #[test]
    fn source_two_edge_local_object_mode_asserted_head_type_assertion_consumes_three_expansions() {
        let source_id = source_id(187);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("two_edge_local_object_mode_asserted_head_type_assertion"),
        );
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseTwoEdgeObjectModeAssertedHead", SymbolKind::Mode),
                ("MiddleTwoEdgeObjectModeAssertedHead", SymbolKind::Mode),
                ("OuterTwoEdgeObjectModeAssertedHead", SymbolKind::Mode),
                ("ExtraTwoEdgeObjectModeAssertedHead", SymbolKind::Mode),
                ("OtherTwoEdgeObjectModeAssertedHead", SymbolKind::Mode),
            ],
        );
        let theorem = exact_two_edge_local_object_mode_asserted_head_spec();
        let exact_modes = || {
            vec![
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
            ]
        };
        let reserve = || {
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("OuterTwoEdgeObjectModeAssertedHead"),
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
        let payload = extract_source_two_edge_local_object_mode_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact Task 187 source should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 3);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(payload.subject_lookup_ordinal, 1);
        let source_binding = &payload.reserve.bridge.bindings()[0];
        assert_eq!(
            source_binding.type_spelling,
            "OuterTwoEdgeObjectModeAssertedHead"
        );
        assert_eq!(payload.asserted_type.spelling, source_binding.type_spelling);
        assert_eq!(payload.asserted_type.head, source_binding.type_head);
        assert!(matches!(source_binding.type_head, TypeHeadInput::Symbol(_)));
        assert_ne!(source_binding.type_range, payload.asserted_type.range);

        let output = source_two_edge_local_object_mode_asserted_head_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact Task 187 source should reach TermFormulaChecker");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 187 checker payload invariants should hold");
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert_eq!(
            output.subject_result_input.spelling,
            "OuterTwoEdgeObjectModeAssertedHead"
        );
        assert_eq!(
            output.asserted_type_input.spelling,
            output.subject_result_input.spelling
        );
        assert_eq!(
            output.asserted_type_input.head,
            output.subject_result_input.head
        );
        assert_ne!(
            output.asserted_type_input.site,
            output.subject_result_input.site
        );
        assert_ne!(
            output.asserted_type_input.source_range,
            output.subject_result_input.source_range
        );
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("BaseTwoEdgeObjectModeAssertedHead")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("Task 187 terminal object expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .unwrap();
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
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());

        let assert_invalid_output = |invalid| {
            let invalid_result =
                assert_source_reserved_variable_type_assertion_output(&invalid).map(|()| invalid);
            assert_eq!(
                source_reserved_variable_type_assertion_result_detail_keys(
                    invalid_result,
                    TYPE_ELABORATION_TWO_EDGE_LOCAL_OBJECT_MODE_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
                ),
                vec![
                    TYPE_ELABORATION_TWO_EDGE_LOCAL_OBJECT_MODE_ASSERTED_HEAD_INVALID_PAYLOAD_KEY
                        .to_owned()
                ]
            );
        };
        for removed in [
            "BaseTwoEdgeObjectModeAssertedHead",
            "MiddleTwoEdgeObjectModeAssertedHead",
            "OuterTwoEdgeObjectModeAssertedHead",
        ] {
            let mut invalid = source_two_edge_local_object_mode_asserted_head_output(
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
        let mut wrong_asserted_spelling = source_two_edge_local_object_mode_asserted_head_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .unwrap();
        wrong_asserted_spelling.asserted_type_input.spelling =
            "OtherTwoEdgeObjectModeAssertedHead".to_owned();
        let mut builtin_set_asserted = source_two_edge_local_object_mode_asserted_head_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .unwrap();
        builtin_set_asserted.asserted_type_input.head = TypeHeadInput::BuiltinSet;
        let mut builtin_set_subject = source_two_edge_local_object_mode_asserted_head_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .unwrap();
        builtin_set_subject.subject_result_input.head = TypeHeadInput::BuiltinSet;
        let mut collapsed_ranges = source_two_edge_local_object_mode_asserted_head_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .unwrap();
        collapsed_ranges.asserted_type_input.source_range =
            collapsed_ranges.subject_result_input.source_range;
        let mut collapsed_sites = source_two_edge_local_object_mode_asserted_head_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .unwrap();
        collapsed_sites.asserted_type_input.site =
            collapsed_sites.subject_result_input.site.clone();
        for invalid in [
            wrong_asserted_spelling,
            builtin_set_asserted,
            builtin_set_subject,
            collapsed_ranges,
            collapsed_sites,
        ] {
            assert_invalid_output(invalid);
        }

        let mut mode_near_misses = vec![
            Vec::<ModeDefinitionSpec>::new(),
            vec![exact_modes()[1], exact_modes()[2]],
            vec![exact_modes()[0], exact_modes()[2]],
            vec![exact_modes()[0], exact_modes()[1]],
            vec![exact_modes()[2], exact_modes()[1], exact_modes()[0]],
            {
                let mut modes = exact_modes();
                modes.push(modes[0]);
                modes
            },
            {
                let mut modes = exact_modes();
                modes[0].rhs_shape = ReserveTypeShape::Builtin("set");
                modes
            },
            {
                let mut modes = exact_modes();
                modes[1].rhs_shape =
                    ReserveTypeShape::QualifiedSymbol("OuterTwoEdgeObjectModeAssertedHead");
                modes
            },
            {
                let mut modes = exact_modes();
                modes[2].rhs_shape =
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeObjectModeAssertedHead");
                modes
            },
            {
                let mut modes = exact_modes();
                modes[0].rhs_shape =
                    ReserveTypeShape::QualifiedSymbol("OuterTwoEdgeObjectModeAssertedHead");
                modes
            },
            {
                let mut modes = exact_modes();
                modes.insert(
                    1,
                    mode_definition_with_label(
                        "ExtraTwoEdgeObjectModeAssertedHead",
                        "ExtraTwoEdgeObjectModeAssertedHeadDef",
                        ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeObjectModeAssertedHead"),
                    ),
                );
                modes[2].rhs_shape =
                    ReserveTypeShape::QualifiedSymbol("ExtraTwoEdgeObjectModeAssertedHead");
                modes
            },
            {
                let mut modes = exact_modes();
                modes[0].recovered = true;
                modes
            },
            {
                let mut modes = exact_modes();
                modes[0].local_context = true;
                modes
            },
            {
                let mut modes = exact_modes();
                modes[1].parameterized_pattern = true;
                modes
            },
            {
                let mut modes = exact_modes();
                modes[1].rhs_shape =
                    ReserveTypeShape::QualifiedSymbolWithArgs("BaseTwoEdgeObjectModeAssertedHead");
                modes
            },
            {
                let mut modes = exact_modes();
                modes[0].rhs_shape = ReserveTypeShape::AttributedObject;
                modes
            },
        ];
        for index in 0..3 {
            let mut modes = exact_modes();
            modes[index].label = Some("WrongTwoEdgeObjectModeAssertedHeadDef");
            mode_near_misses.push(modes);
        }
        for modes in mode_near_misses {
            let near_miss = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                modes,
                reserve(),
                theorem,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }

        let source_near_misses = [
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbolWithArgs("OuterTwoEdgeObjectModeAssertedHead"),
                )],
                theorem,
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::AttributedQualifiedSymbol(
                        "OuterTwoEdgeObjectModeAssertedHead",
                    ),
                )],
                theorem,
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                vec![
                    reserve_item(
                        vec!["x"],
                        ReserveTypeShape::QualifiedSymbol("OuterTwoEdgeObjectModeAssertedHead"),
                    ),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                ],
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
                    asserted_type: ReserveTypeShape::Builtin("set"),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(
                        "BaseTwoEdgeObjectModeAssertedHead",
                    ),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(
                        "MiddleTwoEdgeObjectModeAssertedHead",
                    ),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(
                        "OtherTwoEdgeObjectModeAssertedHead",
                    ),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbolWithArgs(
                        "OuterTwoEdgeObjectModeAssertedHead",
                    ),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::AttributedQualifiedSymbol(
                        "OuterTwoEdgeObjectModeAssertedHead",
                    ),
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
        ];
        for near_miss in source_near_misses {
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }

        let imported_base = source_local_and_imported_symbols_env(
            module.clone(),
            &[
                ("MiddleTwoEdgeObjectModeAssertedHead", SymbolKind::Mode),
                ("OuterTwoEdgeObjectModeAssertedHead", SymbolKind::Mode),
            ],
            &[("BaseTwoEdgeObjectModeAssertedHead", SymbolKind::Mode)],
        );
        let imported_middle = source_local_and_imported_symbols_env(
            module.clone(),
            &[
                ("BaseTwoEdgeObjectModeAssertedHead", SymbolKind::Mode),
                ("OuterTwoEdgeObjectModeAssertedHead", SymbolKind::Mode),
            ],
            &[("MiddleTwoEdgeObjectModeAssertedHead", SymbolKind::Mode)],
        );
        let imported_outer = source_local_and_imported_symbols_env(
            module.clone(),
            &[
                ("BaseTwoEdgeObjectModeAssertedHead", SymbolKind::Mode),
                ("MiddleTwoEdgeObjectModeAssertedHead", SymbolKind::Mode),
            ],
            &[("OuterTwoEdgeObjectModeAssertedHead", SymbolKind::Mode)],
        );
        let ambiguous_outer = source_local_and_imported_symbols_env(
            module.clone(),
            &[
                ("BaseTwoEdgeObjectModeAssertedHead", SymbolKind::Mode),
                ("MiddleTwoEdgeObjectModeAssertedHead", SymbolKind::Mode),
            ],
            &[
                ("OuterTwoEdgeObjectModeAssertedHead", SymbolKind::Mode),
                ("OuterTwoEdgeObjectModeAssertedHead", SymbolKind::Mode),
            ],
        );
        for provenance_symbols in [
            &imported_base,
            &imported_middle,
            &imported_outer,
            &ambiguous_outer,
        ] {
            assert_eq!(
                source_type_elaboration_detail_keys(&exact, module.clone(), provenance_symbols,),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }

        for (asserted_spelling, asserted_symbols) in [
            (
                "ImportedTwoEdgeObjectModeAssertedHead",
                source_local_and_imported_symbols_env(
                    module.clone(),
                    &[
                        ("BaseTwoEdgeObjectModeAssertedHead", SymbolKind::Mode),
                        ("MiddleTwoEdgeObjectModeAssertedHead", SymbolKind::Mode),
                        ("OuterTwoEdgeObjectModeAssertedHead", SymbolKind::Mode),
                    ],
                    &[("ImportedTwoEdgeObjectModeAssertedHead", SymbolKind::Mode)],
                ),
            ),
            (
                "AmbiguousTwoEdgeObjectModeAssertedHead",
                source_local_and_imported_symbols_env(
                    module.clone(),
                    &[
                        ("BaseTwoEdgeObjectModeAssertedHead", SymbolKind::Mode),
                        ("MiddleTwoEdgeObjectModeAssertedHead", SymbolKind::Mode),
                        ("OuterTwoEdgeObjectModeAssertedHead", SymbolKind::Mode),
                    ],
                    &[
                        ("AmbiguousTwoEdgeObjectModeAssertedHead", SymbolKind::Mode),
                        ("AmbiguousTwoEdgeObjectModeAssertedHead", SymbolKind::Mode),
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
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &asserted_symbols,),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
    }

    #[test]
    fn source_three_edge_local_mode_reserved_variable_type_assertion_consumes_four_expansions() {
        let source_id = source_id(150);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("three_edge_local_mode_reserved_variable_type_assertion"),
        );
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseThreeEdgeModeTypeAssertion", SymbolKind::Mode),
                ("InnerThreeEdgeModeTypeAssertion", SymbolKind::Mode),
                ("MiddleThreeEdgeModeTypeAssertion", SymbolKind::Mode),
                ("OuterThreeEdgeModeTypeAssertion", SymbolKind::Mode),
                ("ExtraThreeEdgeModeTypeAssertion", SymbolKind::Mode),
            ],
        );
        let theorem = exact_three_edge_local_mode_identifier_type_assertion_spec();
        let exact_modes = || {
            vec![
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
            ]
        };
        let reserve = || {
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("OuterThreeEdgeModeTypeAssertion"),
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
        let payload = extract_source_three_edge_local_mode_reserved_variable_type_assertion(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact three-edge local-mode reserved-variable type assertion should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 4);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(
            payload.reserve.bridge.bindings()[0].type_spelling,
            "OuterThreeEdgeModeTypeAssertion"
        );
        assert_eq!(payload.asserted_type.spelling, "set");
        assert_eq!(payload.asserted_type.head, TypeHeadInput::BuiltinSet);
        assert_ne!(
            payload.reserve.bridge.bindings()[0].type_range,
            payload.asserted_type.range
        );
        assert_eq!(payload.subject_lookup_ordinal, 1);

        let output = source_three_edge_local_mode_reserved_variable_type_assertion_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact three-edge local-mode type assertion should reach TermFormulaChecker");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("three-edge local-mode type assertion invariants should hold");
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(
            output.subject_result_input.spelling,
            "OuterThreeEdgeModeTypeAssertion"
        );
        assert!(matches!(
            output.subject_result_input.head,
            TypeHeadInput::Symbol(_)
        ));
        assert_eq!(output.asserted_type_input.spelling, "set");
        assert_eq!(output.asserted_type_input.head, TypeHeadInput::BuiltinSet);
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("BaseThreeEdgeModeTypeAssertion")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("base mode terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("one normalized builtin-set type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
        let (_, formula) = output.term_formula.formulas().iter().next().unwrap();
        assert_eq!(formula.kind, FormulaKind::TypeAssertion);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());

        for removed in [
            "BaseThreeEdgeModeTypeAssertion",
            "InnerThreeEdgeModeTypeAssertion",
            "MiddleThreeEdgeModeTypeAssertion",
            "OuterThreeEdgeModeTypeAssertion",
        ] {
            let mut invalid = source_three_edge_local_mode_reserved_variable_type_assertion_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce a four-expansion corruption target");
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
                    TYPE_ELABORATION_THREE_EDGE_LOCAL_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY,
                ),
                vec![
                    TYPE_ELABORATION_THREE_EDGE_LOCAL_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY
                        .to_owned()
                ]
            );
        }

        let assert_extraction_gap = |modes| {
            let near_miss = mode_then_reserve_identifier_type_assertion_theorem_ast(
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

        for index in 0..4 {
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

        for index in 1..4 {
            let mut argument_bearing = exact_modes();
            let radix = match index {
                1 => "BaseThreeEdgeModeTypeAssertion",
                2 => "InnerThreeEdgeModeTypeAssertion",
                3 => "MiddleThreeEdgeModeTypeAssertion",
                _ => unreachable!(),
            };
            argument_bearing[index].rhs_shape = ReserveTypeShape::QualifiedSymbolWithArgs(radix);
            assert_extraction_gap(argument_bearing);
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

        let mut reverse_order = exact_modes();
        reverse_order.reverse();
        assert_extraction_gap(reverse_order);

        let mut wrong_inner_radix = exact_modes();
        wrong_inner_radix[1].rhs_shape =
            ReserveTypeShape::QualifiedSymbol("ExtraThreeEdgeModeTypeAssertion");
        assert_extraction_gap(wrong_inner_radix);

        let mut wrong_middle_radix = exact_modes();
        wrong_middle_radix[2].rhs_shape =
            ReserveTypeShape::QualifiedSymbol("BaseThreeEdgeModeTypeAssertion");
        assert_extraction_gap(wrong_middle_radix);

        let mut wrong_outer_radix = exact_modes();
        wrong_outer_radix[3].rhs_shape =
            ReserveTypeShape::QualifiedSymbol("InnerThreeEdgeModeTypeAssertion");
        assert_extraction_gap(wrong_outer_radix);

        let mut one_edge_outer_radix = exact_modes();
        one_edge_outer_radix[3].rhs_shape =
            ReserveTypeShape::QualifiedSymbol("BaseThreeEdgeModeTypeAssertion");
        assert_extraction_gap(one_edge_outer_radix);

        let mut direct_outer_radix = exact_modes();
        direct_outer_radix[3].rhs_shape = ReserveTypeShape::Builtin("set");
        assert_extraction_gap(direct_outer_radix);

        let mut cyclic = exact_modes();
        cyclic[0].rhs_shape = ReserveTypeShape::QualifiedSymbol("OuterThreeEdgeModeTypeAssertion");
        assert_extraction_gap(cyclic);

        assert_extraction_gap(vec![
            exact_modes()[0],
            mode_definition_with_label(
                "ExtraThreeEdgeModeTypeAssertion",
                "ExtraThreeEdgeModeTypeAssertionDef",
                ReserveTypeShape::QualifiedSymbol("BaseThreeEdgeModeTypeAssertion"),
            ),
            mode_definition_with_label(
                "InnerThreeEdgeModeTypeAssertion",
                "InnerThreeEdgeModeTypeAssertionDef",
                ReserveTypeShape::QualifiedSymbol("ExtraThreeEdgeModeTypeAssertion"),
            ),
            exact_modes()[2],
            exact_modes()[3],
        ]);

        for near_miss in [
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbolWithArgs("OuterThreeEdgeModeTypeAssertion"),
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
                    asserted_type: ReserveTypeShape::QualifiedSymbol(
                        "OuterThreeEdgeModeTypeAssertion",
                    ),
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
    }

    #[test]
    fn source_four_edge_local_mode_reserved_variable_type_assertion_consumes_five_expansions() {
        let source_id = source_id(152);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("four_edge_local_mode_reserved_variable_type_assertion"),
        );
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseFourEdgeModeTypeAssertion", SymbolKind::Mode),
                ("InnerFourEdgeModeTypeAssertion", SymbolKind::Mode),
                ("MiddleFourEdgeModeTypeAssertion", SymbolKind::Mode),
                ("OuterFourEdgeModeTypeAssertion", SymbolKind::Mode),
                ("TooDeepFourEdgeModeTypeAssertion", SymbolKind::Mode),
                ("ExtraFourEdgeModeTypeAssertion", SymbolKind::Mode),
            ],
        );
        let theorem = exact_four_edge_local_mode_identifier_type_assertion_spec();
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    "BaseFourEdgeModeTypeAssertion",
                    "BaseFourEdgeModeTypeAssertionDef",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition_with_label(
                    "InnerFourEdgeModeTypeAssertion",
                    "InnerFourEdgeModeTypeAssertionDef",
                    ReserveTypeShape::QualifiedSymbol("BaseFourEdgeModeTypeAssertion"),
                ),
                mode_definition_with_label(
                    "MiddleFourEdgeModeTypeAssertion",
                    "MiddleFourEdgeModeTypeAssertionDef",
                    ReserveTypeShape::QualifiedSymbol("InnerFourEdgeModeTypeAssertion"),
                ),
                mode_definition_with_label(
                    "OuterFourEdgeModeTypeAssertion",
                    "OuterFourEdgeModeTypeAssertionDef",
                    ReserveTypeShape::QualifiedSymbol("MiddleFourEdgeModeTypeAssertion"),
                ),
                mode_definition_with_label(
                    "TooDeepFourEdgeModeTypeAssertion",
                    "TooDeepFourEdgeModeTypeAssertionDef",
                    ReserveTypeShape::QualifiedSymbol("OuterFourEdgeModeTypeAssertion"),
                ),
            ]
        };
        let reserve = || {
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("TooDeepFourEdgeModeTypeAssertion"),
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
        let payload = extract_source_four_edge_local_mode_reserved_variable_type_assertion(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact four-edge local-mode reserved-variable type assertion should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 5);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(
            payload.reserve.bridge.bindings()[0].type_spelling,
            "TooDeepFourEdgeModeTypeAssertion"
        );
        assert_eq!(payload.asserted_type.spelling, "set");
        assert_eq!(payload.asserted_type.head, TypeHeadInput::BuiltinSet);
        assert_ne!(
            payload.reserve.bridge.bindings()[0].type_range,
            payload.asserted_type.range
        );
        assert_eq!(payload.subject_lookup_ordinal, 1);

        let output = source_four_edge_local_mode_reserved_variable_type_assertion_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact four-edge local-mode type assertion should reach TermFormulaChecker");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("four-edge local-mode type assertion invariants should hold");
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(
            output.subject_result_input.spelling,
            "TooDeepFourEdgeModeTypeAssertion"
        );
        assert!(matches!(
            output.subject_result_input.head,
            TypeHeadInput::Symbol(_)
        ));
        assert_eq!(output.asserted_type_input.spelling, "set");
        assert_eq!(output.asserted_type_input.head, TypeHeadInput::BuiltinSet);
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("BaseFourEdgeModeTypeAssertion")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("base mode terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("one normalized builtin-set type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
        let (_, formula) = output.term_formula.formulas().iter().next().unwrap();
        assert_eq!(formula.kind, FormulaKind::TypeAssertion);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());

        for removed in [
            "BaseFourEdgeModeTypeAssertion",
            "InnerFourEdgeModeTypeAssertion",
            "MiddleFourEdgeModeTypeAssertion",
            "OuterFourEdgeModeTypeAssertion",
            "TooDeepFourEdgeModeTypeAssertion",
        ] {
            let mut invalid = source_four_edge_local_mode_reserved_variable_type_assertion_output(
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
            let invalid_result =
                assert_source_reserved_variable_type_assertion_output(&invalid).map(|()| invalid);
            assert_eq!(
                source_reserved_variable_type_assertion_result_detail_keys(
                    invalid_result,
                    TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY,
                ),
                vec![
                    TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY
                        .to_owned()
                ]
            );
        }

        let assert_extraction_gap = |modes| {
            let near_miss = mode_then_reserve_identifier_type_assertion_theorem_ast(
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
            "BaseFourEdgeModeTypeAssertion",
            "InnerFourEdgeModeTypeAssertion",
            "MiddleFourEdgeModeTypeAssertion",
            "OuterFourEdgeModeTypeAssertion",
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
                ReserveTypeShape::QualifiedSymbol("ExtraFourEdgeModeTypeAssertion");
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

        let mut reverse_order = exact_modes();
        reverse_order.reverse();
        assert_extraction_gap(reverse_order);

        let mut direct_outermost_radix = exact_modes();
        direct_outermost_radix[4].rhs_shape = ReserveTypeShape::Builtin("set");
        assert_extraction_gap(direct_outermost_radix);

        let mut one_edge_outermost_radix = exact_modes();
        one_edge_outermost_radix[4].rhs_shape =
            ReserveTypeShape::QualifiedSymbol("BaseFourEdgeModeTypeAssertion");
        assert_extraction_gap(one_edge_outermost_radix);

        let mut two_edge_outermost_radix = exact_modes();
        two_edge_outermost_radix[4].rhs_shape =
            ReserveTypeShape::QualifiedSymbol("InnerFourEdgeModeTypeAssertion");
        assert_extraction_gap(two_edge_outermost_radix);

        let mut three_edge_outermost_radix = exact_modes();
        three_edge_outermost_radix[4].rhs_shape =
            ReserveTypeShape::QualifiedSymbol("MiddleFourEdgeModeTypeAssertion");
        assert_extraction_gap(three_edge_outermost_radix);

        let mut cyclic = exact_modes();
        cyclic[0].rhs_shape = ReserveTypeShape::QualifiedSymbol("TooDeepFourEdgeModeTypeAssertion");
        assert_extraction_gap(cyclic);

        assert_extraction_gap(vec![
            exact_modes()[0],
            mode_definition_with_label(
                "ExtraFourEdgeModeTypeAssertion",
                "ExtraFourEdgeModeTypeAssertionDef",
                ReserveTypeShape::QualifiedSymbol("BaseFourEdgeModeTypeAssertion"),
            ),
            mode_definition_with_label(
                "InnerFourEdgeModeTypeAssertion",
                "InnerFourEdgeModeTypeAssertionDef",
                ReserveTypeShape::QualifiedSymbol("ExtraFourEdgeModeTypeAssertion"),
            ),
            exact_modes()[2],
            exact_modes()[3],
            exact_modes()[4],
        ]);

        for near_miss in [
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbolWithArgs("TooDeepFourEdgeModeTypeAssertion"),
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
                    asserted_type: ReserveTypeShape::QualifiedSymbol(
                        "TooDeepFourEdgeModeTypeAssertion",
                    ),
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
    }

    #[test]
    fn source_four_edge_local_object_mode_reserved_variable_type_assertion_consumes_five_expansions()
     {
        let source_id = source_id(153);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("four_edge_local_object_mode_reserved_variable_type_assertion"),
        );
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseFourEdgeObjectModeTypeAssertion", SymbolKind::Mode),
                ("InnerFourEdgeObjectModeTypeAssertion", SymbolKind::Mode),
                ("MiddleFourEdgeObjectModeTypeAssertion", SymbolKind::Mode),
                ("OuterFourEdgeObjectModeTypeAssertion", SymbolKind::Mode),
                ("TooDeepFourEdgeObjectModeTypeAssertion", SymbolKind::Mode),
                ("ExtraFourEdgeObjectModeTypeAssertion", SymbolKind::Mode),
            ],
        );
        let theorem = exact_four_edge_local_object_mode_identifier_type_assertion_spec();
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    "BaseFourEdgeObjectModeTypeAssertion",
                    "BaseFourEdgeObjectModeTypeAssertionDef",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition_with_label(
                    "InnerFourEdgeObjectModeTypeAssertion",
                    "InnerFourEdgeObjectModeTypeAssertionDef",
                    ReserveTypeShape::QualifiedSymbol("BaseFourEdgeObjectModeTypeAssertion"),
                ),
                mode_definition_with_label(
                    "MiddleFourEdgeObjectModeTypeAssertion",
                    "MiddleFourEdgeObjectModeTypeAssertionDef",
                    ReserveTypeShape::QualifiedSymbol("InnerFourEdgeObjectModeTypeAssertion"),
                ),
                mode_definition_with_label(
                    "OuterFourEdgeObjectModeTypeAssertion",
                    "OuterFourEdgeObjectModeTypeAssertionDef",
                    ReserveTypeShape::QualifiedSymbol("MiddleFourEdgeObjectModeTypeAssertion"),
                ),
                mode_definition_with_label(
                    "TooDeepFourEdgeObjectModeTypeAssertion",
                    "TooDeepFourEdgeObjectModeTypeAssertionDef",
                    ReserveTypeShape::QualifiedSymbol("OuterFourEdgeObjectModeTypeAssertion"),
                ),
            ]
        };
        let reserve = || {
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("TooDeepFourEdgeObjectModeTypeAssertion"),
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
        let payload = extract_source_four_edge_local_object_mode_reserved_variable_type_assertion(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect(
            "exact four-edge local-object-mode reserved-variable type assertion should extract",
        );
        assert_eq!(payload.reserve.mode_expansions.len(), 5);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(
            payload.reserve.bridge.bindings()[0].type_spelling,
            "TooDeepFourEdgeObjectModeTypeAssertion"
        );
        assert_eq!(payload.asserted_type.spelling, "object");
        assert_eq!(payload.asserted_type.head, TypeHeadInput::BuiltinObject);
        assert_ne!(
            payload.reserve.bridge.bindings()[0].type_range,
            payload.asserted_type.range
        );
        assert_eq!(payload.subject_lookup_ordinal, 1);

        let output = source_four_edge_local_object_mode_reserved_variable_type_assertion_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact four-edge local-object-mode type assertion should reach TermFormulaChecker");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("four-edge local-object-mode type assertion invariants should hold");
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(
            output.subject_result_input.spelling,
            "TooDeepFourEdgeObjectModeTypeAssertion"
        );
        assert!(matches!(
            output.subject_result_input.head,
            TypeHeadInput::Symbol(_)
        ));
        assert_eq!(output.asserted_type_input.spelling, "object");
        assert_eq!(
            output.asserted_type_input.head,
            TypeHeadInput::BuiltinObject
        );
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("BaseFourEdgeObjectModeTypeAssertion")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("base object mode terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("one normalized builtin-object type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
        let (_, formula) = output.term_formula.formulas().iter().next().unwrap();
        assert_eq!(formula.kind, FormulaKind::TypeAssertion);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());

        for removed in [
            "BaseFourEdgeObjectModeTypeAssertion",
            "InnerFourEdgeObjectModeTypeAssertion",
            "MiddleFourEdgeObjectModeTypeAssertion",
            "OuterFourEdgeObjectModeTypeAssertion",
            "TooDeepFourEdgeObjectModeTypeAssertion",
        ] {
            let mut invalid =
                source_four_edge_local_object_mode_reserved_variable_type_assertion_output(
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
            let invalid_result =
                assert_source_reserved_variable_type_assertion_output(&invalid).map(|()| invalid);
            assert_eq!(
                source_reserved_variable_type_assertion_result_detail_keys(
                    invalid_result,
                    TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY,
                ),
                vec![
                    TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY
                        .to_owned()
                ]
            );
        }

        let assert_extraction_gap = |modes| {
            let near_miss = mode_then_reserve_identifier_type_assertion_theorem_ast(
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
            "BaseFourEdgeObjectModeTypeAssertion",
            "InnerFourEdgeObjectModeTypeAssertion",
            "MiddleFourEdgeObjectModeTypeAssertion",
            "OuterFourEdgeObjectModeTypeAssertion",
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
                ReserveTypeShape::QualifiedSymbol("ExtraFourEdgeObjectModeTypeAssertion");
            assert_extraction_gap(wrong_radix);
        }

        let mut duplicate = exact_modes();
        duplicate.push(duplicate[0]);
        assert_extraction_gap(duplicate);

        let mut attributed_terminal = exact_modes();
        attributed_terminal[0].rhs_shape = ReserveTypeShape::AttributedObject;
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
            ReserveTypeShape::QualifiedSymbol("BaseFourEdgeObjectModeTypeAssertion");
        assert_extraction_gap(one_edge_outermost_radix);

        let mut two_edge_outermost_radix = exact_modes();
        two_edge_outermost_radix[4].rhs_shape =
            ReserveTypeShape::QualifiedSymbol("InnerFourEdgeObjectModeTypeAssertion");
        assert_extraction_gap(two_edge_outermost_radix);

        let mut three_edge_outermost_radix = exact_modes();
        three_edge_outermost_radix[4].rhs_shape =
            ReserveTypeShape::QualifiedSymbol("MiddleFourEdgeObjectModeTypeAssertion");
        assert_extraction_gap(three_edge_outermost_radix);

        let mut cyclic = exact_modes();
        cyclic[0].rhs_shape =
            ReserveTypeShape::QualifiedSymbol("TooDeepFourEdgeObjectModeTypeAssertion");
        assert_extraction_gap(cyclic);

        assert_extraction_gap(vec![
            exact_modes()[0],
            mode_definition_with_label(
                "ExtraFourEdgeObjectModeTypeAssertion",
                "ExtraFourEdgeObjectModeTypeAssertionDef",
                ReserveTypeShape::QualifiedSymbol("BaseFourEdgeObjectModeTypeAssertion"),
            ),
            mode_definition_with_label(
                "InnerFourEdgeObjectModeTypeAssertion",
                "InnerFourEdgeObjectModeTypeAssertionDef",
                ReserveTypeShape::QualifiedSymbol("ExtraFourEdgeObjectModeTypeAssertion"),
            ),
            exact_modes()[2],
            exact_modes()[3],
            exact_modes()[4],
        ]);

        for near_miss in [
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbolWithArgs(
                        "TooDeepFourEdgeObjectModeTypeAssertion",
                    ),
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
                    asserted_type: ReserveTypeShape::QualifiedSymbol(
                        "TooDeepFourEdgeObjectModeTypeAssertion",
                    ),
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
    }

    #[test]
    fn source_three_edge_local_object_mode_reserved_variable_type_assertion_consumes_four_expansions()
     {
        let source_id = source_id(151);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("three_edge_local_object_mode_reserved_variable_type_assertion"),
        );
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseThreeEdgeObjectModeTypeAssertion", SymbolKind::Mode),
                ("InnerThreeEdgeObjectModeTypeAssertion", SymbolKind::Mode),
                ("MiddleThreeEdgeObjectModeTypeAssertion", SymbolKind::Mode),
                ("OuterThreeEdgeObjectModeTypeAssertion", SymbolKind::Mode),
                ("ExtraThreeEdgeObjectModeTypeAssertion", SymbolKind::Mode),
            ],
        );
        let theorem = exact_three_edge_local_object_mode_identifier_type_assertion_spec();
        let exact_modes = || {
            vec![
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
            ]
        };
        let reserve = || {
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("OuterThreeEdgeObjectModeTypeAssertion"),
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
        let payload = extract_source_three_edge_local_object_mode_reserved_variable_type_assertion(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect(
            "exact three-edge local-object-mode reserved-variable type assertion should extract",
        );
        assert_eq!(payload.reserve.mode_expansions.len(), 4);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(
            payload.reserve.bridge.bindings()[0].type_spelling,
            "OuterThreeEdgeObjectModeTypeAssertion"
        );
        assert_eq!(payload.asserted_type.spelling, "object");
        assert_eq!(payload.asserted_type.head, TypeHeadInput::BuiltinObject);
        assert_ne!(
            payload.reserve.bridge.bindings()[0].type_range,
            payload.asserted_type.range
        );
        assert_eq!(payload.subject_lookup_ordinal, 1);

        let output = source_three_edge_local_object_mode_reserved_variable_type_assertion_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect(
            "exact three-edge local-object-mode type assertion should reach TermFormulaChecker",
        );
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("three-edge local-object-mode type assertion invariants should hold");
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(
            output.subject_result_input.spelling,
            "OuterThreeEdgeObjectModeTypeAssertion"
        );
        assert!(matches!(
            output.subject_result_input.head,
            TypeHeadInput::Symbol(_)
        ));
        assert_eq!(output.asserted_type_input.spelling, "object");
        assert_eq!(
            output.asserted_type_input.head,
            TypeHeadInput::BuiltinObject
        );
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("BaseThreeEdgeObjectModeTypeAssertion")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("base mode terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("one normalized builtin-object type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
        let (_, formula) = output.term_formula.formulas().iter().next().unwrap();
        assert_eq!(formula.kind, FormulaKind::TypeAssertion);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());

        for removed in [
            "BaseThreeEdgeObjectModeTypeAssertion",
            "InnerThreeEdgeObjectModeTypeAssertion",
            "MiddleThreeEdgeObjectModeTypeAssertion",
            "OuterThreeEdgeObjectModeTypeAssertion",
        ] {
            let mut invalid =
                source_three_edge_local_object_mode_reserved_variable_type_assertion_output(
                    &exact,
                    module.clone(),
                    &symbols,
                )
                .expect("exact source should produce a four-expansion corruption target");
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
                    TYPE_ELABORATION_THREE_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY,
                ),
                vec![
                    TYPE_ELABORATION_THREE_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY
                        .to_owned()
                ]
            );
        }

        let assert_extraction_gap = |modes| {
            let near_miss = mode_then_reserve_identifier_type_assertion_theorem_ast(
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

        for index in 0..4 {
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

        for index in 1..4 {
            let mut argument_bearing = exact_modes();
            let radix = match index {
                1 => "BaseThreeEdgeObjectModeTypeAssertion",
                2 => "InnerThreeEdgeObjectModeTypeAssertion",
                3 => "MiddleThreeEdgeObjectModeTypeAssertion",
                _ => unreachable!(),
            };
            argument_bearing[index].rhs_shape = ReserveTypeShape::QualifiedSymbolWithArgs(radix);
            assert_extraction_gap(argument_bearing);
        }

        let mut duplicate = exact_modes();
        duplicate.push(duplicate[0]);
        assert_extraction_gap(duplicate);

        let mut attributed_terminal = exact_modes();
        attributed_terminal[0].rhs_shape = ReserveTypeShape::AttributedObject;
        assert_extraction_gap(attributed_terminal);

        let mut set_terminal = exact_modes();
        set_terminal[0].rhs_shape = ReserveTypeShape::Builtin("set");
        assert_extraction_gap(set_terminal);

        let mut reverse_order = exact_modes();
        reverse_order.reverse();
        assert_extraction_gap(reverse_order);

        let mut wrong_inner_radix = exact_modes();
        wrong_inner_radix[1].rhs_shape =
            ReserveTypeShape::QualifiedSymbol("ExtraThreeEdgeObjectModeTypeAssertion");
        assert_extraction_gap(wrong_inner_radix);

        let mut wrong_middle_radix = exact_modes();
        wrong_middle_radix[2].rhs_shape =
            ReserveTypeShape::QualifiedSymbol("BaseThreeEdgeObjectModeTypeAssertion");
        assert_extraction_gap(wrong_middle_radix);

        let mut wrong_outer_radix = exact_modes();
        wrong_outer_radix[3].rhs_shape =
            ReserveTypeShape::QualifiedSymbol("InnerThreeEdgeObjectModeTypeAssertion");
        assert_extraction_gap(wrong_outer_radix);

        let mut one_edge_outer_radix = exact_modes();
        one_edge_outer_radix[3].rhs_shape =
            ReserveTypeShape::QualifiedSymbol("BaseThreeEdgeObjectModeTypeAssertion");
        assert_extraction_gap(one_edge_outer_radix);

        let mut direct_outer_radix = exact_modes();
        direct_outer_radix[3].rhs_shape = ReserveTypeShape::Builtin("object");
        assert_extraction_gap(direct_outer_radix);

        let mut cyclic = exact_modes();
        cyclic[0].rhs_shape =
            ReserveTypeShape::QualifiedSymbol("OuterThreeEdgeObjectModeTypeAssertion");
        assert_extraction_gap(cyclic);

        assert_extraction_gap(vec![
            exact_modes()[0],
            mode_definition_with_label(
                "ExtraThreeEdgeObjectModeTypeAssertion",
                "ExtraThreeEdgeObjectModeTypeAssertionDef",
                ReserveTypeShape::QualifiedSymbol("BaseThreeEdgeObjectModeTypeAssertion"),
            ),
            mode_definition_with_label(
                "InnerThreeEdgeObjectModeTypeAssertion",
                "InnerThreeEdgeObjectModeTypeAssertionDef",
                ReserveTypeShape::QualifiedSymbol("ExtraThreeEdgeObjectModeTypeAssertion"),
            ),
            exact_modes()[2],
            exact_modes()[3],
        ]);

        for near_miss in [
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbolWithArgs(
                        "OuterThreeEdgeObjectModeTypeAssertion",
                    ),
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
                    asserted_type: ReserveTypeShape::QualifiedSymbol(
                        "OuterThreeEdgeObjectModeTypeAssertion",
                    ),
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
    }
