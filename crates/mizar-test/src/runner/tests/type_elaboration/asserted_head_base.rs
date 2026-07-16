    #[test]
    fn source_reserved_variable_type_assertion_bridge_checks_reflexive_admissibility() {
        let source_id = source_id(99);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("reserved_variable_type_assertion"),
        );
        let symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        let ast = reserve_then_identifier_type_assertion_theorem_ast(
            source_id,
            vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
            exact_identifier_type_assertion_spec(),
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&ast, module.clone(), &symbols),
            Vec::<String>::new()
        );
        let payload =
            extract_source_reserved_variable_type_assertion(&ast, module.clone(), &symbols)
                .expect("exact reserved-variable type assertion should extract");
        let formula_nodes = surface_nodes_with_kind(&ast, SurfaceNodeKind::IsAssertion);
        let [(formula_id, formula)] = formula_nodes.as_slice() else {
            panic!("exact type assertion should contain one assertion formula");
        };
        let formula_children = structural_child_ids(&ast, formula);
        let [subject_expression_id, asserted_type_id] = formula_children.as_slice() else {
            panic!("exact type assertion should contain one subject and asserted type");
        };
        let subject_expression = ast
            .node(*subject_expression_id)
            .expect("exact type assertion subject expression should exist");
        let subject_children = structural_child_ids(&ast, subject_expression);
        let [subject_id] = subject_children.as_slice() else {
            panic!("exact type assertion should contain one subject reference");
        };
        let subject = ast
            .node(*subject_id)
            .expect("exact type assertion subject reference should exist");
        let asserted_type = ast
            .node(*asserted_type_id)
            .expect("exact asserted type should exist");
        assert_eq!(payload.formula_site, surface_site(*formula_id));
        assert_eq!(payload.formula_range, formula.range);
        assert_eq!(payload.subject_site, surface_site(*subject_id));
        assert_eq!(payload.subject_range, subject.range);
        assert_eq!(payload.asserted_type_site, surface_site(*asserted_type_id));
        assert_eq!(payload.asserted_type.range, asserted_type.range);
        assert_ne!(payload.formula_site, payload.subject_site);
        assert_ne!(payload.formula_site, payload.asserted_type_site);
        assert_ne!(payload.subject_site, payload.asserted_type_site);
        assert!(payload.subject_range.end <= payload.asserted_type.range.start);
        assert_eq!(
            payload.config.label,
            "ReservedVariableTypeAssertionPayloadBoundary"
        );
        assert_eq!(
            payload.config.invalid_payload_key,
            super::TYPE_ELABORATION_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY
        );
        assert_eq!(payload.config.binding_spelling, "x");
        assert_eq!(
            payload.config.binding_type,
            super::SourceReservedVariableBuiltinType::Set
        );
        assert_eq!(payload.config.binding_source_mode_spelling, None);
        assert!(payload.config.mode_definitions.is_empty());
        assert_eq!(
            payload.config.asserted_type,
            super::SourceReservedVariableBuiltinType::Set
        );
        assert_eq!(
            payload.config.asserted_head_relation,
            super::SourceReservedVariableAssertedHeadRelation::Builtin
        );
        assert_eq!(
            payload.config.subject_result_role,
            "reserved-variable-type-assertion-subject-result"
        );
        let [source_binding] = payload.reserve.bridge.bindings() else {
            panic!("exact type assertion should contain one reserve binding");
        };
        assert_eq!(source_binding.spelling, "x");
        assert_eq!(source_binding.type_spelling, "set");
        assert_eq!(source_binding.type_head, TypeHeadInput::BuiltinSet);
        assert!(payload.reserve.mode_expansions.is_empty());
        assert_eq!(payload.subject_spelling, "x");
        assert_eq!(payload.subject_lookup_ordinal, 1);
        assert_eq!(payload.asserted_type.spelling, "set");
        assert_eq!(payload.asserted_type.head, TypeHeadInput::BuiltinSet);
        assert_ne!(
            payload.reserve.bridge.bindings()[0].type_range,
            payload.asserted_type.range
        );

        let output = source_reserved_variable_type_assertion_output(&ast, module.clone(), &symbols)
            .expect("exact reserved-variable type assertion should reach the checker");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("reserved-variable type assertion invariants should hold");
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_ne!(
            output.subject_result_input.source_range,
            output.asserted_type_input.source_range
        );
        assert!(output.subject_result_input.args.is_empty());
        assert!(output.asserted_type_input.args.is_empty());
        let (_, formula) = output.term_formula.formulas().iter().next().unwrap();
        assert_eq!(formula.kind, FormulaKind::TypeAssertion);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());

        let pre_output =
            extract_source_reserved_variable_type_assertion(&ast, module.clone(), &symbols)
                .expect("exact source should produce a pre-output payload");
        let mismatched_symbols = SymbolEnv::new(
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("other_module")),
            SymbolEnvIndexes::default(),
        );
        assert_eq!(
            source_reserved_variable_type_assertion_result_detail_keys(
                build_source_reserved_variable_type_assertion_output(
                    pre_output,
                    &mismatched_symbols,
                ),
                super::TYPE_ELABORATION_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY,
            ),
            vec![
                super::TYPE_ELABORATION_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY
                    .to_owned()
            ]
        );

        let exact = exact_identifier_type_assertion_spec();
        let gap_cases = [
            reserve_then_identifier_type_assertion_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
                IdentifierTypeAssertionTheoremSpec {
                    label: "OtherPayloadBoundary",
                    ..exact
                },
            ),
            reserve_then_identifier_type_assertion_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
                IdentifierTypeAssertionTheoremSpec {
                    subject: "y",
                    ..exact
                },
            ),
            reserve_then_builtin_type_assertion_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
                exact.label,
                "1",
                ReserveTypeShape::Builtin("set"),
            ),
            reserve_then_identifier_type_assertion_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("object"))],
                exact,
            ),
            reserve_then_identifier_type_assertion_theorem_ast(
                source_id,
                vec![reserve_item(
                    vec!["x", "y"],
                    ReserveTypeShape::Builtin("set"),
                )],
                exact,
            ),
            reserve_then_identifier_type_assertion_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::AttributedSet)],
                exact,
            ),
            reserve_then_identifier_type_assertion_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::Builtin("object"),
                    ..exact
                },
            ),
            reserve_then_identifier_type_assertion_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::AttributedSet,
                    ..exact
                },
            ),
            reserve_then_identifier_type_assertion_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
                IdentifierTypeAssertionTheoremSpec {
                    negated: true,
                    ..exact
                },
            ),
            reserve_then_identifier_type_assertion_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
                IdentifierTypeAssertionTheoremSpec {
                    status: Some("open"),
                    ..exact
                },
            ),
            reserve_then_identifier_type_assertion_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
                IdentifierTypeAssertionTheoremSpec {
                    status: Some("registration"),
                    ..exact
                },
            ),
            reserve_then_identifier_type_assertion_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
                IdentifierTypeAssertionTheoremSpec {
                    recovered_label: true,
                    ..exact
                },
            ),
            reserve_then_identifier_type_assertion_theorem_ast(
                source_id,
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::Builtin("set")),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                ],
                exact,
            ),
            reserve_then_two_identifier_type_assertion_theorems_ast(source_id),
            identifier_type_assertion_theorem_then_reserve_ast(source_id),
        ];
        for gap_case in gap_cases {
            assert!(
                extract_source_reserved_variable_type_assertion(
                    &gap_case,
                    module.clone(),
                    &symbols
                )
                .is_none()
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&gap_case, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
        for corruption in [
            BuiltinTypeAssertionTheoremCorruption {
                recovered_is: true,
                ..BuiltinTypeAssertionTheoremCorruption::default()
            },
            BuiltinTypeAssertionTheoremCorruption {
                duplicate_formula_expression: true,
                ..BuiltinTypeAssertionTheoremCorruption::default()
            },
            BuiltinTypeAssertionTheoremCorruption {
                extra_formula_child: true,
                ..BuiltinTypeAssertionTheoremCorruption::default()
            },
            BuiltinTypeAssertionTheoremCorruption {
                extra_assertion_operand: true,
                ..BuiltinTypeAssertionTheoremCorruption::default()
            },
        ] {
            let gap_case = reserve_then_identifier_type_assertion_theorem_ast_with_corruption(
                source_id,
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::Builtin("set"),
                )],
                exact,
                corruption,
            );
            assert!(
                extract_source_reserved_variable_type_assertion(
                    &gap_case,
                    module.clone(),
                    &symbols
                )
                .is_none()
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&gap_case, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
    }

    #[test]
    fn source_local_mode_reserved_variable_type_assertion_consumes_real_expansion() {
        let source_id = source_id(138);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("local_mode_reserved_variable_type_assertion"),
        );
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("LocalModeTypeAssertion", SymbolKind::Mode),
                ("BaseModeTypeAssertion", SymbolKind::Mode),
            ],
        );
        let theorem = exact_local_mode_identifier_type_assertion_spec();
        let reserve = || {
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("LocalModeTypeAssertion"),
            )]
        };
        let exact = mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            [mode_definition(
                "LocalModeTypeAssertion",
                ReserveTypeShape::Builtin("set"),
            )],
            reserve(),
            theorem,
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        let payload = extract_source_local_mode_reserved_variable_type_assertion(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact local-mode reserved-variable type assertion should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 1);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(
            payload.reserve.bridge.bindings()[0].type_spelling,
            "LocalModeTypeAssertion"
        );
        assert_eq!(payload.asserted_type.spelling, "set");
        assert_eq!(payload.asserted_type.head, TypeHeadInput::BuiltinSet);
        assert_ne!(
            payload.reserve.bridge.bindings()[0].type_range,
            payload.asserted_type.range
        );

        let output = source_local_mode_reserved_variable_type_assertion_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact local-mode type assertion should reach TermFormulaChecker");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("local-mode type assertion invariants should hold");
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(
            output.subject_result_input.spelling,
            "LocalModeTypeAssertion"
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
            .values()
            .next()
            .expect("real direct expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("one normalized builtin-set type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_eq!(normalized.source.range, terminal.radix.source_range);
        let (_, formula) = output.term_formula.formulas().iter().next().unwrap();
        assert_eq!(formula.kind, FormulaKind::TypeAssertion);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());

        let mut corrupted_output = source_local_mode_reserved_variable_type_assertion_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce a corrupted checker output");
        corrupted_output.payload.reserve.mode_expansions.clear();
        let corrupted_result =
            assert_source_reserved_variable_type_assertion_output(&corrupted_output)
                .map(|()| corrupted_output);
        assert_eq!(
            source_reserved_variable_type_assertion_result_detail_keys(
                corrupted_result,
                TYPE_ELABORATION_LOCAL_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY,
            ),
            vec![
                TYPE_ELABORATION_LOCAL_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY
                    .to_owned()
            ]
        );

        for mode in [
            mode_definition(
                "LocalModeTypeAssertion",
                ReserveTypeShape::Builtin("object"),
            ),
            mode_definition("LocalModeTypeAssertion", ReserveTypeShape::AttributedSet),
            contextual_mode_definition("LocalModeTypeAssertion", ReserveTypeShape::Builtin("set")),
            parameterized_mode_definition(
                "LocalModeTypeAssertion",
                ReserveTypeShape::Builtin("set"),
            ),
            recovered_mode_definition("LocalModeTypeAssertion", ReserveTypeShape::Builtin("set")),
        ] {
            let near_miss = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                [mode],
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
                Vec::<ModeDefinitionSpec>::new(),
                reserve(),
                theorem,
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                [
                    mode_definition("LocalModeTypeAssertion", ReserveTypeShape::Builtin("set")),
                    mode_definition("LocalModeTypeAssertion", ReserveTypeShape::Builtin("set")),
                ],
                reserve(),
                theorem,
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                [mode_definition_with_label(
                    "LocalModeTypeAssertion",
                    "OtherDef",
                    ReserveTypeShape::Builtin("set"),
                )],
                reserve(),
                theorem,
            ),
            reserve_then_mode_identifier_type_assertion_theorem_ast(
                source_id,
                reserve(),
                mode_definition("LocalModeTypeAssertion", ReserveTypeShape::Builtin("set")),
                theorem,
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                [mode_definition(
                    "LocalModeTypeAssertion",
                    ReserveTypeShape::Builtin("set"),
                )],
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbolWithArgs("LocalModeTypeAssertion"),
                )],
                theorem,
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                [
                    mode_definition("BaseModeTypeAssertion", ReserveTypeShape::Builtin("set")),
                    mode_definition(
                        "LocalModeTypeAssertion",
                        ReserveTypeShape::QualifiedSymbol("BaseModeTypeAssertion"),
                    ),
                ],
                reserve(),
                theorem,
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                [mode_definition(
                    "LocalModeTypeAssertion",
                    ReserveTypeShape::Builtin("set"),
                )],
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    label: "OtherPayloadBoundary",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                [mode_definition(
                    "LocalModeTypeAssertion",
                    ReserveTypeShape::Builtin("set"),
                )],
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    subject: "y",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                [mode_definition(
                    "LocalModeTypeAssertion",
                    ReserveTypeShape::Builtin("set"),
                )],
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::Builtin("object"),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                [mode_definition(
                    "LocalModeTypeAssertion",
                    ReserveTypeShape::Builtin("set"),
                )],
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol("LocalModeTypeAssertion"),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                [mode_definition(
                    "LocalModeTypeAssertion",
                    ReserveTypeShape::Builtin("set"),
                )],
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    negated: true,
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                [mode_definition(
                    "LocalModeTypeAssertion",
                    ReserveTypeShape::Builtin("set"),
                )],
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    status: Some("registration"),
                    ..theorem
                },
            ),
        ] {
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
    }

    #[test]
    fn source_local_mode_asserted_head_type_assertion_consumes_real_expansion() {
        let source_id = source_id(182);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("local_mode_asserted_head_type_assertion"),
        );
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("LocalModeAssertedHead", SymbolKind::Mode),
                ("OtherMode", SymbolKind::Mode),
            ],
        );
        let theorem = exact_local_mode_asserted_head_spec();
        let exact_mode =
            || mode_definition("LocalModeAssertedHead", ReserveTypeShape::Builtin("set"));
        let reserve = || {
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("LocalModeAssertedHead"),
            )]
        };
        let exact = mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            [exact_mode()],
            reserve(),
            theorem,
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        let payload = extract_source_local_mode_asserted_head(&exact, module.clone(), &symbols)
            .expect("exact formula-side local-mode assertion should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 1);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        let source_binding = &payload.reserve.bridge.bindings()[0];
        assert_eq!(source_binding.type_spelling, "LocalModeAssertedHead");
        assert_eq!(payload.asserted_type.spelling, "LocalModeAssertedHead");
        assert!(matches!(source_binding.type_head, TypeHeadInput::Symbol(_)));
        assert_eq!(payload.asserted_type.head, source_binding.type_head);
        assert_ne!(source_binding.type_range, payload.asserted_type.range);
        assert_eq!(payload.subject_lookup_ordinal, 1);

        let output = source_local_mode_asserted_head_output(&exact, module.clone(), &symbols)
            .expect("exact formula-side local-mode assertion should reach the checker");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("formula-side local-mode assertion invariants should hold");
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(
            output.subject_result_input.spelling,
            "LocalModeAssertedHead"
        );
        assert_eq!(output.asserted_type_input.spelling, "LocalModeAssertedHead");
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
            .values()
            .next()
            .expect("real direct expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("one normalized builtin-set type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_eq!(normalized.source.range, terminal.radix.source_range);
        assert_eq!(normalized.source.spelling, terminal.radix.spelling);
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
            .expect("local-mode type assertion should be checked");
        assert_eq!(formula.kind, FormulaKind::TypeAssertion);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert!(formula.expected_types.is_empty());
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());

        let invalid_key =
            || vec![TYPE_ELABORATION_LOCAL_MODE_ASSERTED_HEAD_INVALID_PAYLOAD_KEY.to_owned()];
        let mut missing_expansion =
            source_local_mode_asserted_head_output(&exact, module.clone(), &symbols)
                .expect("exact source should produce an expansion corruption target");
        missing_expansion.payload.reserve.mode_expansions.clear();
        let mut wrong_asserted_spelling =
            source_local_mode_asserted_head_output(&exact, module.clone(), &symbols)
                .expect("exact source should produce an asserted-spelling corruption target");
        wrong_asserted_spelling.asserted_type_input.spelling = "OtherMode".to_owned();
        let mut wrong_asserted_head =
            source_local_mode_asserted_head_output(&exact, module.clone(), &symbols)
                .expect("exact source should produce an asserted-head corruption target");
        wrong_asserted_head.asserted_type_input.head = TypeHeadInput::BuiltinSet;
        let mut collapsed_ranges =
            source_local_mode_asserted_head_output(&exact, module.clone(), &symbols)
                .expect("exact source should produce a range corruption target");
        collapsed_ranges.asserted_type_input.source_range =
            collapsed_ranges.subject_result_input.source_range;
        let mut wrong_subject_head =
            source_local_mode_asserted_head_output(&exact, module.clone(), &symbols)
                .expect("exact source should produce a subject-head corruption target");
        wrong_subject_head.subject_result_input.head = TypeHeadInput::BuiltinSet;
        for invalid in [
            missing_expansion,
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
                    TYPE_ELABORATION_LOCAL_MODE_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
                ),
                invalid_key()
            );
        }

        for mode in [
            mode_definition("LocalModeAssertedHead", ReserveTypeShape::Builtin("object")),
            mode_definition("LocalModeAssertedHead", ReserveTypeShape::AttributedSet),
            contextual_mode_definition("LocalModeAssertedHead", ReserveTypeShape::Builtin("set")),
            parameterized_mode_definition(
                "LocalModeAssertedHead",
                ReserveTypeShape::Builtin("set"),
            ),
            recovered_mode_definition("LocalModeAssertedHead", ReserveTypeShape::Builtin("set")),
        ] {
            let near_miss = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                [mode],
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
                Vec::<ModeDefinitionSpec>::new(),
                reserve(),
                theorem,
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                [exact_mode(), exact_mode()],
                reserve(),
                theorem,
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                [mode_definition_with_label(
                    "LocalModeAssertedHead",
                    "OtherDef",
                    ReserveTypeShape::Builtin("set"),
                )],
                reserve(),
                theorem,
            ),
            reserve_then_mode_identifier_type_assertion_theorem_ast(
                source_id,
                reserve(),
                exact_mode(),
                theorem,
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                [exact_mode()],
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbolWithArgs("LocalModeAssertedHead"),
                )],
                theorem,
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                [exact_mode()],
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::AttributedQualifiedSymbol("LocalModeAssertedHead"),
                )],
                theorem,
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                [
                    mode_definition("OtherMode", ReserveTypeShape::Builtin("set")),
                    mode_definition(
                        "LocalModeAssertedHead",
                        ReserveTypeShape::QualifiedSymbol("OtherMode"),
                    ),
                ],
                reserve(),
                theorem,
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                [exact_mode()],
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    label: "OtherPayloadBoundary",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                [exact_mode()],
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    subject: "y",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                [exact_mode()],
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::Builtin("set"),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                [exact_mode()],
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::Builtin("object"),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                [exact_mode()],
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol("OtherMode"),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                [exact_mode()],
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbolWithArgs(
                        "LocalModeAssertedHead",
                    ),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                [exact_mode()],
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::AttributedQualifiedSymbol(
                        "LocalModeAssertedHead",
                    ),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                [exact_mode()],
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    negated: true,
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                [exact_mode()],
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    status: Some("registration"),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                [exact_mode()],
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    recovered_label: true,
                    ..theorem
                },
            ),
        ] {
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
    }

    #[test]
    fn source_local_object_mode_asserted_head_type_assertion_consumes_real_expansion() {
        let source_id = source_id(183);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("local_object_mode_asserted_head_type_assertion"),
        );
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("LocalObjectModeAssertedHead", SymbolKind::Mode),
                ("OtherObjectMode", SymbolKind::Mode),
            ],
        );
        let theorem = exact_local_object_mode_asserted_head_spec();
        let exact_mode = || {
            mode_definition(
                "LocalObjectModeAssertedHead",
                ReserveTypeShape::Builtin("object"),
            )
        };
        let reserve = || {
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("LocalObjectModeAssertedHead"),
            )]
        };
        let exact = mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            [exact_mode()],
            reserve(),
            theorem,
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        let payload =
            extract_source_local_object_mode_asserted_head(&exact, module.clone(), &symbols)
                .expect("exact object-terminal formula-side local-mode assertion should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 1);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        let source_binding = &payload.reserve.bridge.bindings()[0];
        assert_eq!(source_binding.type_spelling, "LocalObjectModeAssertedHead");
        assert_eq!(
            payload.asserted_type.spelling,
            "LocalObjectModeAssertedHead"
        );
        assert!(matches!(source_binding.type_head, TypeHeadInput::Symbol(_)));
        assert_eq!(payload.asserted_type.head, source_binding.type_head);
        assert_ne!(source_binding.type_range, payload.asserted_type.range);
        assert_eq!(payload.subject_lookup_ordinal, 1);

        let output =
            source_local_object_mode_asserted_head_output(&exact, module.clone(), &symbols)
                .expect("exact object-terminal formula-side assertion should reach the checker");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("object-terminal formula-side assertion invariants should hold");
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(
            output.subject_result_input.spelling,
            "LocalObjectModeAssertedHead"
        );
        assert_eq!(
            output.asserted_type_input.spelling,
            "LocalObjectModeAssertedHead"
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
            .values()
            .next()
            .expect("real direct object expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("one normalized builtin-object type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_eq!(normalized.source.range, terminal.radix.source_range);
        assert_eq!(normalized.source.spelling, terminal.radix.spelling);
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
            .expect("object-terminal local-mode type assertion should be checked");
        assert_eq!(formula.kind, FormulaKind::TypeAssertion);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert!(formula.expected_types.is_empty());
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());

        let invalid_key = || {
            vec![TYPE_ELABORATION_LOCAL_OBJECT_MODE_ASSERTED_HEAD_INVALID_PAYLOAD_KEY.to_owned()]
        };
        let mut missing_expansion =
            source_local_object_mode_asserted_head_output(&exact, module.clone(), &symbols)
                .expect("exact source should produce an expansion corruption target");
        missing_expansion.payload.reserve.mode_expansions.clear();
        let mut wrong_asserted_spelling =
            source_local_object_mode_asserted_head_output(&exact, module.clone(), &symbols)
                .expect("exact source should produce an asserted-spelling corruption target");
        wrong_asserted_spelling.asserted_type_input.spelling = "OtherObjectMode".to_owned();
        let mut wrong_asserted_head =
            source_local_object_mode_asserted_head_output(&exact, module.clone(), &symbols)
                .expect("exact source should produce an asserted-head corruption target");
        wrong_asserted_head.asserted_type_input.head = TypeHeadInput::BuiltinSet;
        let mut collapsed_ranges =
            source_local_object_mode_asserted_head_output(&exact, module.clone(), &symbols)
                .expect("exact source should produce a range corruption target");
        collapsed_ranges.asserted_type_input.source_range =
            collapsed_ranges.subject_result_input.source_range;
        let mut wrong_subject_head =
            source_local_object_mode_asserted_head_output(&exact, module.clone(), &symbols)
                .expect("exact source should produce a subject-head corruption target");
        wrong_subject_head.subject_result_input.head = TypeHeadInput::BuiltinSet;
        for invalid in [
            missing_expansion,
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
                    TYPE_ELABORATION_LOCAL_OBJECT_MODE_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
                ),
                invalid_key()
            );
        }

        for mode in [
            mode_definition(
                "LocalObjectModeAssertedHead",
                ReserveTypeShape::Builtin("set"),
            ),
            mode_definition(
                "LocalObjectModeAssertedHead",
                ReserveTypeShape::AttributedObject,
            ),
            contextual_mode_definition(
                "LocalObjectModeAssertedHead",
                ReserveTypeShape::Builtin("object"),
            ),
            parameterized_mode_definition(
                "LocalObjectModeAssertedHead",
                ReserveTypeShape::Builtin("object"),
            ),
            recovered_mode_definition(
                "LocalObjectModeAssertedHead",
                ReserveTypeShape::Builtin("object"),
            ),
        ] {
            let near_miss = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                [mode],
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
                Vec::<ModeDefinitionSpec>::new(),
                reserve(),
                theorem,
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                [exact_mode(), exact_mode()],
                reserve(),
                theorem,
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                [mode_definition_with_label(
                    "LocalObjectModeAssertedHead",
                    "OtherDef",
                    ReserveTypeShape::Builtin("object"),
                )],
                reserve(),
                theorem,
            ),
            reserve_then_mode_identifier_type_assertion_theorem_ast(
                source_id,
                reserve(),
                exact_mode(),
                theorem,
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                [exact_mode()],
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbolWithArgs("LocalObjectModeAssertedHead"),
                )],
                theorem,
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                [exact_mode()],
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::AttributedQualifiedSymbol("LocalObjectModeAssertedHead"),
                )],
                theorem,
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                [
                    mode_definition("OtherObjectMode", ReserveTypeShape::Builtin("object")),
                    mode_definition(
                        "LocalObjectModeAssertedHead",
                        ReserveTypeShape::QualifiedSymbol("OtherObjectMode"),
                    ),
                ],
                reserve(),
                theorem,
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                [exact_mode()],
                vec![
                    reserve_item(
                        vec!["x"],
                        ReserveTypeShape::QualifiedSymbol("LocalObjectModeAssertedHead"),
                    ),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("object")),
                ],
                theorem,
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                [exact_mode()],
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    label: "OtherPayloadBoundary",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                [exact_mode()],
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    subject: "y",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                [exact_mode()],
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::Builtin("object"),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                [exact_mode()],
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::Builtin("set"),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                [exact_mode()],
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol("OtherObjectMode"),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                [exact_mode()],
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbolWithArgs(
                        "LocalObjectModeAssertedHead",
                    ),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                [exact_mode()],
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::AttributedQualifiedSymbol(
                        "LocalObjectModeAssertedHead",
                    ),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                [exact_mode()],
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    negated: true,
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                [exact_mode()],
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    status: Some("registration"),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                [exact_mode()],
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    recovered_label: true,
                    ..theorem
                },
            ),
        ] {
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
    }

    #[test]
    fn source_local_object_mode_reserved_variable_type_assertion_consumes_real_expansion() {
        let source_id = source_id(145);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("local_object_mode_reserved_variable_type_assertion"),
        );
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("LocalObjectModeTypeAssertion", SymbolKind::Mode),
                ("BaseObjectModeTypeAssertion", SymbolKind::Mode),
            ],
        );
        let theorem = exact_local_object_mode_identifier_type_assertion_spec();
        let reserve = || {
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("LocalObjectModeTypeAssertion"),
            )]
        };
        let exact = mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            [mode_definition(
                "LocalObjectModeTypeAssertion",
                ReserveTypeShape::Builtin("object"),
            )],
            reserve(),
            theorem,
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        let payload = extract_source_local_object_mode_reserved_variable_type_assertion(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact local-object-mode reserved-variable type assertion should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 1);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(
            payload.reserve.bridge.bindings()[0].type_spelling,
            "LocalObjectModeTypeAssertion"
        );
        assert_eq!(payload.asserted_type.spelling, "object");
        assert_eq!(payload.asserted_type.head, TypeHeadInput::BuiltinObject);
        assert_ne!(
            payload.reserve.bridge.bindings()[0].type_range,
            payload.asserted_type.range
        );
        assert_eq!(payload.subject_lookup_ordinal, 1);

        let output = source_local_object_mode_reserved_variable_type_assertion_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact local-object-mode type assertion should reach TermFormulaChecker");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("local-object-mode type assertion invariants should hold");
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(
            output.subject_result_input.spelling,
            "LocalObjectModeTypeAssertion"
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
            .values()
            .next()
            .expect("real direct expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("one normalized builtin-object type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_eq!(normalized.source.range, terminal.radix.source_range);
        assert_eq!(normalized.source.spelling, terminal.radix.spelling);
        let (_, formula) = output.term_formula.formulas().iter().next().unwrap();
        assert_eq!(formula.kind, FormulaKind::TypeAssertion);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());

        let mut corrupted_output =
            source_local_object_mode_reserved_variable_type_assertion_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce a corrupted checker output");
        corrupted_output.payload.reserve.mode_expansions.clear();
        let corrupted_result =
            assert_source_reserved_variable_type_assertion_output(&corrupted_output)
                .map(|()| corrupted_output);
        assert_eq!(
            source_reserved_variable_type_assertion_result_detail_keys(
                corrupted_result,
                TYPE_ELABORATION_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY,
            ),
            vec![
                TYPE_ELABORATION_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY
                    .to_owned()
            ]
        );

        for mode in [
            mode_definition(
                "LocalObjectModeTypeAssertion",
                ReserveTypeShape::Builtin("set"),
            ),
            mode_definition(
                "LocalObjectModeTypeAssertion",
                ReserveTypeShape::AttributedSet,
            ),
            contextual_mode_definition(
                "LocalObjectModeTypeAssertion",
                ReserveTypeShape::Builtin("object"),
            ),
            parameterized_mode_definition(
                "LocalObjectModeTypeAssertion",
                ReserveTypeShape::Builtin("object"),
            ),
            recovered_mode_definition(
                "LocalObjectModeTypeAssertion",
                ReserveTypeShape::Builtin("object"),
            ),
        ] {
            let near_miss = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                [mode],
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
                Vec::<ModeDefinitionSpec>::new(),
                reserve(),
                theorem,
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                [
                    mode_definition(
                        "LocalObjectModeTypeAssertion",
                        ReserveTypeShape::Builtin("object"),
                    ),
                    mode_definition(
                        "LocalObjectModeTypeAssertion",
                        ReserveTypeShape::Builtin("object"),
                    ),
                ],
                reserve(),
                theorem,
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                [mode_definition_with_label(
                    "LocalObjectModeTypeAssertion",
                    "OtherDef",
                    ReserveTypeShape::Builtin("object"),
                )],
                reserve(),
                theorem,
            ),
            reserve_then_mode_identifier_type_assertion_theorem_ast(
                source_id,
                reserve(),
                mode_definition(
                    "LocalObjectModeTypeAssertion",
                    ReserveTypeShape::Builtin("object"),
                ),
                theorem,
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                [mode_definition(
                    "LocalObjectModeTypeAssertion",
                    ReserveTypeShape::Builtin("object"),
                )],
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbolWithArgs("LocalObjectModeTypeAssertion"),
                )],
                theorem,
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                [
                    mode_definition(
                        "BaseObjectModeTypeAssertion",
                        ReserveTypeShape::Builtin("object"),
                    ),
                    mode_definition(
                        "LocalObjectModeTypeAssertion",
                        ReserveTypeShape::QualifiedSymbol("BaseObjectModeTypeAssertion"),
                    ),
                ],
                reserve(),
                theorem,
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                [mode_definition(
                    "LocalObjectModeTypeAssertion",
                    ReserveTypeShape::Builtin("object"),
                )],
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    label: "OtherPayloadBoundary",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                [mode_definition(
                    "LocalObjectModeTypeAssertion",
                    ReserveTypeShape::Builtin("object"),
                )],
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    subject: "y",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                [mode_definition(
                    "LocalObjectModeTypeAssertion",
                    ReserveTypeShape::Builtin("object"),
                )],
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::Builtin("set"),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                [mode_definition(
                    "LocalObjectModeTypeAssertion",
                    ReserveTypeShape::Builtin("object"),
                )],
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(
                        "LocalObjectModeTypeAssertion",
                    ),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                [mode_definition(
                    "LocalObjectModeTypeAssertion",
                    ReserveTypeShape::Builtin("object"),
                )],
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    negated: true,
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                [mode_definition(
                    "LocalObjectModeTypeAssertion",
                    ReserveTypeShape::Builtin("object"),
                )],
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    status: Some("registration"),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                [mode_definition(
                    "LocalObjectModeTypeAssertion",
                    ReserveTypeShape::Builtin("object"),
                )],
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    recovered_label: true,
                    ..theorem
                },
            ),
            modes_then_empty_definition_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                [mode_definition(
                    "LocalObjectModeTypeAssertion",
                    ReserveTypeShape::Builtin("object"),
                )],
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
    fn source_chained_local_mode_asserted_head_type_assertion_consumes_both_expansions() {
        let source_id = source_id(184);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("chained_local_mode_asserted_head_type_assertion"),
        );
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseModeAssertedHead", SymbolKind::Mode),
                ("ChainModeAssertedHead", SymbolKind::Mode),
                ("ExtraModeAssertedHead", SymbolKind::Mode),
                ("OtherModeAssertedHead", SymbolKind::Mode),
            ],
        );
        let theorem = exact_chained_local_mode_asserted_head_spec();
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    "BaseModeAssertedHead",
                    "BaseModeAssertedHeadDef",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition_with_label(
                    "ChainModeAssertedHead",
                    "ChainModeAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol("BaseModeAssertedHead"),
                ),
            ]
        };
        let reserve = || {
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("ChainModeAssertedHead"),
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
        let payload =
            extract_source_chained_local_mode_asserted_head(&exact, module.clone(), &symbols)
                .expect("exact one-edge formula-side local-mode assertion should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 2);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        let source_binding = &payload.reserve.bridge.bindings()[0];
        assert_eq!(source_binding.type_spelling, "ChainModeAssertedHead");
        assert_eq!(payload.asserted_type.spelling, "ChainModeAssertedHead");
        assert!(matches!(source_binding.type_head, TypeHeadInput::Symbol(_)));
        assert_eq!(payload.asserted_type.head, source_binding.type_head);
        assert_ne!(source_binding.type_range, payload.asserted_type.range);
        assert_eq!(payload.subject_lookup_ordinal, 1);

        let output =
            source_chained_local_mode_asserted_head_output(&exact, module.clone(), &symbols)
                .expect("exact one-edge formula-side assertion should reach the checker");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("one-edge formula-side assertion invariants should hold");
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(
            output.subject_result_input.spelling,
            "ChainModeAssertedHead"
        );
        assert_eq!(output.asserted_type_input.spelling, "ChainModeAssertedHead");
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
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("BaseModeAssertedHead"))
            .map(|(_, expansion)| &expansion.radix)
            .expect("real base-mode terminal expansion should exist");
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
            .expect("one-edge local-mode type assertion should be checked");
        assert_eq!(formula.kind, FormulaKind::TypeAssertion);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert!(formula.expected_types.is_empty());
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());

        let invalid_key = || {
            vec![TYPE_ELABORATION_CHAINED_LOCAL_MODE_ASSERTED_HEAD_INVALID_PAYLOAD_KEY.to_owned()]
        };
        for removed in ["BaseModeAssertedHead", "ChainModeAssertedHead"] {
            let mut invalid =
                source_chained_local_mode_asserted_head_output(&exact, module.clone(), &symbols)
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
                    TYPE_ELABORATION_CHAINED_LOCAL_MODE_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
                ),
                invalid_key()
            );
        }
        let mut wrong_asserted_spelling =
            source_chained_local_mode_asserted_head_output(&exact, module.clone(), &symbols)
                .expect("exact source should produce an asserted-spelling corruption target");
        wrong_asserted_spelling.asserted_type_input.spelling = "OtherModeAssertedHead".to_owned();
        let mut wrong_asserted_head =
            source_chained_local_mode_asserted_head_output(&exact, module.clone(), &symbols)
                .expect("exact source should produce an asserted-head corruption target");
        wrong_asserted_head.asserted_type_input.head = TypeHeadInput::BuiltinSet;
        let mut collapsed_ranges =
            source_chained_local_mode_asserted_head_output(&exact, module.clone(), &symbols)
                .expect("exact source should produce a range corruption target");
        collapsed_ranges.asserted_type_input.source_range =
            collapsed_ranges.subject_result_input.source_range;
        let mut wrong_subject_head =
            source_chained_local_mode_asserted_head_output(&exact, module.clone(), &symbols)
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
                    TYPE_ELABORATION_CHAINED_LOCAL_MODE_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
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
                    "BaseModeAssertedHead",
                    "OtherDef",
                    ReserveTypeShape::Builtin("set"),
                ),
                exact_modes()[1],
            ],
            vec![
                exact_modes()[0],
                mode_definition_with_label(
                    "ChainModeAssertedHead",
                    "OtherDef",
                    ReserveTypeShape::QualifiedSymbol("BaseModeAssertedHead"),
                ),
            ],
            vec![
                recovered_mode_definition("BaseModeAssertedHead", ReserveTypeShape::Builtin("set")),
                exact_modes()[1],
            ],
            vec![
                exact_modes()[0],
                recovered_mode_definition(
                    "ChainModeAssertedHead",
                    ReserveTypeShape::QualifiedSymbol("BaseModeAssertedHead"),
                ),
            ],
            vec![
                contextual_mode_definition(
                    "BaseModeAssertedHead",
                    ReserveTypeShape::Builtin("set"),
                ),
                exact_modes()[1],
            ],
            vec![
                exact_modes()[0],
                parameterized_mode_definition(
                    "ChainModeAssertedHead",
                    ReserveTypeShape::QualifiedSymbol("BaseModeAssertedHead"),
                ),
            ],
            vec![
                exact_modes()[0],
                mode_definition_with_label(
                    "ChainModeAssertedHead",
                    "ChainModeAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbolWithArgs("BaseModeAssertedHead"),
                ),
            ],
            vec![
                exact_modes()[0],
                mode_definition_with_label(
                    "ChainModeAssertedHead",
                    "ChainModeAssertedHeadDef",
                    ReserveTypeShape::AttributedQualifiedSymbol("BaseModeAssertedHead"),
                ),
            ],
            vec![
                mode_definition_with_label(
                    "BaseModeAssertedHead",
                    "BaseModeAssertedHeadDef",
                    ReserveTypeShape::AttributedSet,
                ),
                exact_modes()[1],
            ],
            vec![
                mode_definition_with_label(
                    "BaseModeAssertedHead",
                    "BaseModeAssertedHeadDef",
                    ReserveTypeShape::Builtin("object"),
                ),
                exact_modes()[1],
            ],
            vec![exact_modes()[1], exact_modes()[0]],
            vec![mode_definition_with_label(
                "ChainModeAssertedHead",
                "ChainModeAssertedHeadDef",
                ReserveTypeShape::Builtin("set"),
            )],
            vec![
                mode_definition_with_label(
                    "BaseModeAssertedHead",
                    "BaseModeAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol("ChainModeAssertedHead"),
                ),
                exact_modes()[1],
            ],
            vec![
                exact_modes()[0],
                mode_definition_with_label(
                    "ExtraModeAssertedHead",
                    "ExtraModeAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol("BaseModeAssertedHead"),
                ),
                mode_definition_with_label(
                    "ChainModeAssertedHead",
                    "ChainModeAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol("ExtraModeAssertedHead"),
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
                    ReserveTypeShape::QualifiedSymbolWithArgs("ChainModeAssertedHead"),
                )],
                theorem,
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::AttributedQualifiedSymbol("ChainModeAssertedHead"),
                )],
                theorem,
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                vec![
                    reserve_item(
                        vec!["x"],
                        ReserveTypeShape::QualifiedSymbol("ChainModeAssertedHead"),
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
                    asserted_type: ReserveTypeShape::QualifiedSymbol("BaseModeAssertedHead"),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol("OtherModeAssertedHead"),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbolWithArgs(
                        "ChainModeAssertedHead",
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
                        "ChainModeAssertedHead",
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
    }

    #[test]
    fn source_chained_local_mode_radix_asserted_head_consumes_immediate_expansion() {
        let source_id = source_id(201);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("chained_local_mode_radix_asserted_head"),
        );
        let base_mode = "BaseModeRadixAssertedHead";
        let outer_mode = "OuterModeRadixAssertedHead";
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                (base_mode, SymbolKind::Mode),
                (outer_mode, SymbolKind::Mode),
                ("DeeperModeRadixAssertedHead", SymbolKind::Mode),
                ("OtherModeRadixAssertedHead", SymbolKind::Mode),
            ],
        );
        let theorem = exact_chained_local_mode_radix_asserted_head_spec();
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    base_mode,
                    "BaseModeRadixAssertedHeadDef",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition_with_label(
                    outer_mode,
                    "OuterModeRadixAssertedHeadDef",
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
            extract_source_reserved_variable_type_assertion(&exact, module.clone(), &symbols,)
                .is_none()
        );
        assert!(
            extract_source_chained_local_mode_asserted_head(&exact, module.clone(), &symbols,)
                .is_none()
        );
        assert!(
            extract_source_chained_local_mode_reserved_variable_type_assertion(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none()
        );
        let task146_symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseModeTypeAssertion", SymbolKind::Mode),
                ("ChainModeTypeAssertion", SymbolKind::Mode),
            ],
        );
        let task146_source = mode_then_reserve_identifier_type_assertion_theorem_ast(
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
        assert!(
            extract_source_chained_local_mode_reserved_variable_type_assertion(
                &task146_source,
                module.clone(),
                &task146_symbols,
            )
            .is_some()
        );
        assert!(
            extract_source_chained_local_mode_radix_asserted_head(
                &task146_source,
                module.clone(),
                &task146_symbols,
            )
            .is_none()
        );
        let payload =
            extract_source_chained_local_mode_radix_asserted_head(&exact, module.clone(), &symbols)
                .expect("exact one-edge immediate-radix asserted head should extract");
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
            .expect("outer binding expansion should exist");
        assert_eq!(outer_expansion.radix.spelling, base_mode);
        assert_eq!(outer_expansion.radix.head, payload.asserted_type.head);

        let output =
            source_chained_local_mode_radix_asserted_head_output(&exact, module.clone(), &symbols)
                .expect("exact immediate-radix asserted head should reach the checker");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 201 checker payload invariants should hold");
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
            .expect("base terminal expansion should exist");
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
                    TYPE_ELABORATION_CHAINED_LOCAL_MODE_RADIX_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
                ),
                vec![
                    TYPE_ELABORATION_CHAINED_LOCAL_MODE_RADIX_ASSERTED_HEAD_INVALID_PAYLOAD_KEY
                        .to_owned()
                ]
            );
        };
        for removed in [base_mode, outer_mode] {
            let mut invalid = source_chained_local_mode_radix_asserted_head_output(
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
            source_chained_local_mode_radix_asserted_head_output(&exact, module.clone(), &symbols)
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
        invalid.asserted_type_input.spelling = "OtherModeRadixAssertedHead".to_owned();
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
                    "UnusedModeRadixAssertedHead",
                    "UnusedModeRadixAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(base_mode),
                ));
                modes
            },
            vec![
                mode_definition_with_label(
                    base_mode,
                    "BaseModeRadixAssertedHeadDef",
                    ReserveTypeShape::Builtin("object"),
                ),
                exact_modes()[1],
            ],
            vec![
                exact_modes()[0],
                mode_definition_with_label(
                    outer_mode,
                    "OuterModeRadixAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol("OtherModeRadixAssertedHead"),
                ),
            ],
        ];
        for index in 0..2 {
            let mut wrong_label = exact_modes();
            wrong_label[index].label = Some("WrongModeRadixAssertedHeadDef");
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
                ReserveTypeShape::QualifiedSymbolWithArgs(base_mode)
            };
            mode_near_misses.push(args);
            let mut attrs = exact_modes();
            attrs[index].rhs_shape = if index == 0 {
                ReserveTypeShape::AttributedSet
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
                &format!("definition near miss {index}"),
            );
        }

        let mut source_near_misses = Vec::new();
        let mut deeper_modes = exact_modes();
        deeper_modes.push(mode_definition_with_label(
            "DeeperModeRadixAssertedHead",
            "DeeperModeRadixAssertedHeadDef",
            ReserveTypeShape::QualifiedSymbol(outer_mode),
        ));
        source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            deeper_modes,
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("DeeperModeRadixAssertedHead"),
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
                asserted_type: ReserveTypeShape::QualifiedSymbol(outer_mode),
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
                asserted_type: ReserveTypeShape::QualifiedSymbol("OtherModeRadixAssertedHead"),
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
            assert_extraction_gap(near_miss, &format!("source near miss {index}"));
        }

        let unrelated_import = source_local_and_imported_symbols_env(
            module.clone(),
            &[
                (base_mode, SymbolKind::Mode),
                (outer_mode, SymbolKind::Mode),
            ],
            &[("UnrelatedModeRadixAssertedHead", SymbolKind::Mode)],
        );
        assert!(
            extract_source_chained_local_mode_radix_asserted_head(
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
    }

    #[test]
    fn source_two_edge_local_mode_radix_asserted_head_consumes_immediate_expansion() {
        let source_id = source_id(203);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("two_edge_local_mode_radix_asserted_head"),
        );
        let base_mode = "BaseTwoEdgeModeRadixAssertedHead";
        let middle_mode = "MiddleTwoEdgeModeRadixAssertedHead";
        let outer_mode = "OuterTwoEdgeModeRadixAssertedHead";
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                (base_mode, SymbolKind::Mode),
                (middle_mode, SymbolKind::Mode),
                (outer_mode, SymbolKind::Mode),
                ("DeeperTwoEdgeModeRadixAssertedHead", SymbolKind::Mode),
                ("OtherTwoEdgeModeRadixAssertedHead", SymbolKind::Mode),
            ],
        );
        let theorem = exact_two_edge_local_mode_radix_asserted_head_spec();
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    base_mode,
                    "BaseTwoEdgeModeRadixAssertedHeadDef",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition_with_label(
                    middle_mode,
                    "MiddleTwoEdgeModeRadixAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(base_mode),
                ),
                mode_definition_with_label(
                    outer_mode,
                    "OuterTwoEdgeModeRadixAssertedHeadDef",
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
            extract_source_reserved_variable_type_assertion(&exact, module.clone(), &symbols)
                .is_none()
        );
        assert!(
            extract_source_two_edge_local_mode_asserted_head(&exact, module.clone(), &symbols,)
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

        let payload = extract_source_two_edge_local_mode_radix_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact two-edge immediate-radix asserted head should extract");
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

        let output =
            source_two_edge_local_mode_radix_asserted_head_output(&exact, module.clone(), &symbols)
                .expect("exact two-edge immediate-radix source should reach the checker");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 203 checker payload invariants should hold");
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
            .expect("base terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("one normalized builtin-set type should exist");
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

        let assert_invalid_output = |invalid| {
            let invalid_result =
                assert_source_reserved_variable_type_assertion_output(&invalid).map(|()| invalid);
            assert_eq!(
                source_reserved_variable_type_assertion_result_detail_keys(
                    invalid_result,
                    TYPE_ELABORATION_TWO_EDGE_LOCAL_MODE_RADIX_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
                ),
                vec![
                    TYPE_ELABORATION_TWO_EDGE_LOCAL_MODE_RADIX_ASSERTED_HEAD_INVALID_PAYLOAD_KEY
                        .to_owned()
                ]
            );
        };
        for removed in [base_mode, middle_mode, outer_mode] {
            let mut invalid = source_two_edge_local_mode_radix_asserted_head_output(
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
            source_two_edge_local_mode_radix_asserted_head_output(&exact, module.clone(), &symbols)
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
        invalid.subject_result_input.head = TypeHeadInput::BuiltinObject;
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
                    "UnusedTwoEdgeModeRadixAssertedHead",
                    "UnusedTwoEdgeModeRadixAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(base_mode),
                ));
                modes
            },
            vec![
                mode_definition_with_label(
                    base_mode,
                    "BaseTwoEdgeModeRadixAssertedHeadDef",
                    ReserveTypeShape::Builtin("object"),
                ),
                exact_modes()[1],
                exact_modes()[2],
            ],
            vec![
                exact_modes()[0],
                mode_definition_with_label(
                    middle_mode,
                    "MiddleTwoEdgeModeRadixAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol("OtherTwoEdgeModeRadixAssertedHead"),
                ),
                exact_modes()[2],
            ],
            vec![
                exact_modes()[0],
                exact_modes()[1],
                mode_definition_with_label(
                    outer_mode,
                    "OuterTwoEdgeModeRadixAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(base_mode),
                ),
            ],
        ];
        for index in 0..3 {
            let mut wrong_pattern = exact_modes();
            wrong_pattern[index].pattern = "OtherTwoEdgeModeRadixAssertedHead";
            mode_near_misses.push(wrong_pattern);
            let mut wrong_label = exact_modes();
            wrong_label[index].label = Some("WrongTwoEdgeModeRadixAssertedHeadDef");
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
                0 => "set",
                1 => base_mode,
                _ => middle_mode,
            };
            let mut args = exact_modes();
            args[index].rhs_shape = ReserveTypeShape::QualifiedSymbolWithArgs(expected_radix);
            mode_near_misses.push(args);
            let mut attributes = exact_modes();
            attributes[index].rhs_shape = if index == 0 {
                ReserveTypeShape::AttributedSet
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
            "DeeperTwoEdgeModeRadixAssertedHead",
            "DeeperTwoEdgeModeRadixAssertedHeadDef",
            ReserveTypeShape::QualifiedSymbol(outer_mode),
        ));
        source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            deeper_modes,
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("DeeperTwoEdgeModeRadixAssertedHead"),
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
                asserted_type: ReserveTypeShape::QualifiedSymbol(outer_mode),
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
                asserted_type: ReserveTypeShape::QualifiedSymbol(
                    "OtherTwoEdgeModeRadixAssertedHead",
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
            &[("UnrelatedTwoEdgeModeRadixAssertedHead", SymbolKind::Mode)],
        );
        assert!(
            extract_source_two_edge_local_mode_radix_asserted_head(
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

        let task122 = reserve_then_identifier_type_assertion_theorem_ast(
            source_id,
            vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
            exact_identifier_type_assertion_spec(),
        );
        assert!(
            extract_source_reserved_variable_type_assertion(&task122, module.clone(), &symbols,)
                .is_some()
        );
        assert!(
            extract_source_two_edge_local_mode_radix_asserted_head(
                &task122,
                module.clone(),
                &symbols,
            )
            .is_none()
        );

        let task148_symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseTwoEdgeModeTypeAssertion", SymbolKind::Mode),
                ("MiddleTwoEdgeModeTypeAssertion", SymbolKind::Mode),
                ("OuterTwoEdgeModeTypeAssertion", SymbolKind::Mode),
            ],
        );
        let task148 = mode_then_reserve_identifier_type_assertion_theorem_ast(
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
                &task148,
                module.clone(),
                &task148_symbols,
            )
            .is_some()
        );
        assert!(
            extract_source_two_edge_local_mode_radix_asserted_head(
                &task148,
                module.clone(),
                &task148_symbols,
            )
            .is_none()
        );

        let task186_symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseTwoEdgeModeAssertedHead", SymbolKind::Mode),
                ("MiddleTwoEdgeModeAssertedHead", SymbolKind::Mode),
                ("OuterTwoEdgeModeAssertedHead", SymbolKind::Mode),
            ],
        );
        let task186 = mode_then_reserve_identifier_type_assertion_theorem_ast(
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
                &task186,
                module.clone(),
                &task186_symbols,
            )
            .is_some()
        );
        assert!(
            extract_source_two_edge_local_mode_radix_asserted_head(
                &task186,
                module.clone(),
                &task186_symbols,
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
            extract_source_two_edge_local_mode_radix_asserted_head(
                &task201,
                module.clone(),
                &task201_symbols,
            )
            .is_none()
        );

        let task149_symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseTwoEdgeObjectModeTypeAssertion", SymbolKind::Mode),
                ("MiddleTwoEdgeObjectModeTypeAssertion", SymbolKind::Mode),
                ("OuterTwoEdgeObjectModeTypeAssertion", SymbolKind::Mode),
            ],
        );
        let task149 = mode_then_reserve_identifier_type_assertion_theorem_ast(
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
                &task149,
                module.clone(),
                &task149_symbols,
            )
            .is_some()
        );
        assert!(
            extract_source_two_edge_local_mode_radix_asserted_head(
                &task149,
                module.clone(),
                &task149_symbols,
            )
            .is_none()
        );

        let task187_symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseTwoEdgeObjectModeAssertedHead", SymbolKind::Mode),
                ("MiddleTwoEdgeObjectModeAssertedHead", SymbolKind::Mode),
                ("OuterTwoEdgeObjectModeAssertedHead", SymbolKind::Mode),
            ],
        );
        let task187 = mode_then_reserve_identifier_type_assertion_theorem_ast(
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
                &task187,
                module.clone(),
                &task187_symbols,
            )
            .is_some()
        );
        assert!(
            extract_source_two_edge_local_mode_radix_asserted_head(
                &task187,
                module.clone(),
                &task187_symbols,
            )
            .is_none()
        );

        let task202_symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseObjectModeRadixAssertedHead", SymbolKind::Mode),
                ("OuterObjectModeRadixAssertedHead", SymbolKind::Mode),
            ],
        );
        let task202 = mode_then_reserve_identifier_type_assertion_theorem_ast(
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
                &task202,
                module.clone(),
                &task202_symbols,
            )
            .is_some()
        );
        assert!(
            extract_source_two_edge_local_mode_radix_asserted_head(
                &task202,
                module,
                &task202_symbols,
            )
            .is_none()
        );
    }

    #[test]
    fn source_two_edge_local_mode_two_hop_asserted_head_consumes_both_expansions() {
        let source_id = source_id(211);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("two_edge_local_mode_two_hop_asserted_head"),
        );
        let base_mode = "BaseTwoHopModeAssertedHead";
        let middle_mode = "MiddleTwoHopModeAssertedHead";
        let outer_mode = "OuterTwoHopModeAssertedHead";
        let deeper_mode = "DeeperTwoHopModeAssertedHead";
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                (base_mode, SymbolKind::Mode),
                (middle_mode, SymbolKind::Mode),
                (outer_mode, SymbolKind::Mode),
                (deeper_mode, SymbolKind::Mode),
                ("OtherTwoHopModeAssertedHead", SymbolKind::Mode),
            ],
        );
        let theorem = exact_two_edge_local_mode_two_hop_asserted_head_spec();
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    base_mode,
                    "BaseTwoHopModeAssertedHeadDef",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition_with_label(
                    middle_mode,
                    "MiddleTwoHopModeAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(base_mode),
                ),
                mode_definition_with_label(
                    outer_mode,
                    "OuterTwoHopModeAssertedHeadDef",
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
            extract_source_two_edge_local_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "the immediate-radix owner must not accept a two-hop asserted head"
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
            super::extract_source_local_object_mode_long_chain_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
        ];
        assert_eq!(preexisting_owner_results.len(), 36);
        assert!(
            preexisting_owner_results
                .into_iter()
                .all(|result| result.is_none()),
            "all 36 pre-existing type-assertion owner routes must reject Task 211"
        );
        let payload = extract_source_two_edge_local_mode_two_hop_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("the exact two-hop asserted-head source should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 3);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(payload.subject_lookup_ordinal, 1);
        let source_binding = &payload.reserve.bridge.bindings()[0];
        assert_eq!(source_binding.type_spelling, outer_mode);
        assert_eq!(payload.asserted_type.spelling, base_mode);
        let outer_expansion = payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(outer_mode))
            .map(|(_, expansion)| expansion)
            .unwrap();
        let middle_expansion = payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(middle_mode))
            .map(|(_, expansion)| expansion)
            .unwrap();
        let base_expansion = payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(base_mode))
            .map(|(_, expansion)| expansion)
            .unwrap();
        assert_eq!(outer_expansion.radix.spelling, middle_mode);
        assert_eq!(middle_expansion.radix.spelling, base_mode);
        assert_eq!(middle_expansion.radix.head, payload.asserted_type.head);
        assert_eq!(base_expansion.radix.spelling, "set");
        assert_eq!(base_expansion.radix.head, TypeHeadInput::BuiltinSet);
        let TypeHeadInput::Symbol(outer_symbol) = &source_binding.type_head else {
            panic!("outer binding must resolve to a symbol")
        };
        let TypeHeadInput::Symbol(middle_symbol) = &outer_expansion.radix.head else {
            panic!("outer expansion must resolve to the middle symbol")
        };
        let TypeHeadInput::Symbol(base_symbol) = &payload.asserted_type.head else {
            panic!("asserted head must resolve to the base symbol")
        };
        assert_ne!(outer_symbol, middle_symbol);
        assert_ne!(outer_symbol, base_symbol);
        assert_ne!(middle_symbol, base_symbol);

        let exact_output = || {
            source_two_edge_local_mode_two_hop_asserted_head_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .unwrap()
        };
        let output = exact_output();
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 211 exact checker output must satisfy every invariant");
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.payload.reserve.mode_expansions.len(), 3);
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert_eq!(output.subject_result_input.spelling, outer_mode);
        assert_eq!(output.asserted_type_input.spelling, base_mode);
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
                    TYPE_ELABORATION_TWO_EDGE_LOCAL_MODE_TWO_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
                ),
                vec![
                    TYPE_ELABORATION_TWO_EDGE_LOCAL_MODE_TWO_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY
                        .to_owned()
                ]
            );
        };
        for removed in [base_mode, middle_mode, outer_mode] {
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
        let asserted_head = invalid.asserted_type_input.head.clone();
        let (_, outer) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(outer_mode))
            .unwrap();
        outer.radix.spelling = base_mode.to_owned();
        outer.radix.head = asserted_head;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        let (_, middle) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(middle_mode))
            .unwrap();
        middle.radix.spelling = "set".to_owned();
        middle.radix.head = TypeHeadInput::BuiltinSet;
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
            .expect("mutating cloned Task 211 outputs must not mutate the exact output");

        let assert_extraction_gap = |ast, context: &str| {
            assert_eq!(
                source_type_elaboration_detail_keys(&ast, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()],
                "{context}"
            );
        };
        let ordered = exact_modes();
        for order in [[0, 2, 1], [1, 0, 2], [1, 2, 0], [2, 0, 1], [2, 1, 0]] {
            assert_extraction_gap(
                mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    vec![ordered[order[0]], ordered[order[1]], ordered[order[2]]],
                    reserve(),
                    theorem,
                ),
                &format!("definition order {order:?}"),
            );
        }
        let mut mode_near_misses = Vec::new();
        for index in 0..3 {
            let mut missing = exact_modes();
            missing.remove(index);
            mode_near_misses.push(missing);
            let mut duplicate = exact_modes();
            duplicate.push(duplicate[index]);
            mode_near_misses.push(duplicate);
            let mut wrong_label = exact_modes();
            wrong_label[index].label = Some("WrongTwoHopModeAssertedHeadDef");
            mode_near_misses.push(wrong_label);
            let mut wrong_pattern = exact_modes();
            wrong_pattern[index].pattern = "OtherTwoHopModeAssertedHead";
            mode_near_misses.push(wrong_pattern);
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
                0 => "set",
                1 => base_mode,
                _ => middle_mode,
            };
            let mut args = exact_modes();
            args[index].rhs_shape = ReserveTypeShape::QualifiedSymbolWithArgs(expected_radix);
            mode_near_misses.push(args);
            let mut attributes = exact_modes();
            attributes[index].rhs_shape = if index == 0 {
                ReserveTypeShape::AttributedSet
            } else {
                ReserveTypeShape::AttributedQualifiedSymbol(expected_radix)
            };
            mode_near_misses.push(attributes);
        }
        let mut wrong_terminal = exact_modes();
        wrong_terminal[0].rhs_shape = ReserveTypeShape::Builtin("object");
        mode_near_misses.push(wrong_terminal);
        let mut skipped_outer_link = exact_modes();
        skipped_outer_link[2].rhs_shape = ReserveTypeShape::QualifiedSymbol(base_mode);
        mode_near_misses.push(skipped_outer_link);
        let mut skipped_middle_link = exact_modes();
        skipped_middle_link[1].rhs_shape = ReserveTypeShape::Builtin("set");
        mode_near_misses.push(skipped_middle_link);
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
                asserted_type: ReserveTypeShape::QualifiedSymbol(middle_mode),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(outer_mode),
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
                asserted_type: ReserveTypeShape::QualifiedSymbol("OtherTwoHopModeAssertedHead"),
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
        let mut connected_deeper_modes = exact_modes();
        connected_deeper_modes.push(mode_definition_with_label(
            deeper_mode,
            "DeeperTwoHopModeAssertedHeadDef",
            ReserveTypeShape::QualifiedSymbol(outer_mode),
        ));
        source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            connected_deeper_modes,
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol(deeper_mode),
            )],
            theorem,
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
            &[
                (base_mode, SymbolKind::Mode),
                (middle_mode, SymbolKind::Mode),
                (outer_mode, SymbolKind::Mode),
            ],
            &[("UnrelatedTwoHopModeAssertedHead", SymbolKind::Mode)],
        );
        assert!(
            extract_source_two_edge_local_mode_two_hop_asserted_head(
                &exact,
                module.clone(),
                &unrelated_import,
            )
            .is_some()
        );
        for imported_index in 0..3 {
            let names = [base_mode, middle_mode, outer_mode];
            let locals = names
                .iter()
                .enumerate()
                .filter(|(index, _)| *index != imported_index)
                .map(|(_, spelling)| (*spelling, SymbolKind::Mode))
                .collect::<Vec<_>>();
            for imports in [
                vec![(names[imported_index], SymbolKind::Mode)],
                vec![
                    (names[imported_index], SymbolKind::Mode),
                    (names[imported_index], SymbolKind::Mode),
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
    fn source_two_edge_local_object_mode_two_hop_asserted_head_consumes_both_expansions() {
        let source_id = source_id(212);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("two_edge_local_object_mode_two_hop_asserted_head"),
        );
        let base_mode = "BaseTwoHopObjectModeAssertedHead";
        let middle_mode = "MiddleTwoHopObjectModeAssertedHead";
        let outer_mode = "OuterTwoHopObjectModeAssertedHead";
        let deeper_mode = "DeeperTwoHopObjectModeAssertedHead";
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                (base_mode, SymbolKind::Mode),
                (middle_mode, SymbolKind::Mode),
                (outer_mode, SymbolKind::Mode),
                (deeper_mode, SymbolKind::Mode),
                ("OtherTwoHopObjectModeAssertedHead", SymbolKind::Mode),
            ],
        );
        let theorem = exact_two_edge_local_object_mode_two_hop_asserted_head_spec();
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    base_mode,
                    "BaseTwoHopObjectModeAssertedHeadDef",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition_with_label(
                    middle_mode,
                    "MiddleTwoHopObjectModeAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(base_mode),
                ),
                mode_definition_with_label(
                    outer_mode,
                    "OuterTwoHopObjectModeAssertedHeadDef",
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
            extract_source_two_edge_local_object_mode_radix_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "the immediate-radix owner must not accept a two-hop asserted head"
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
        ];
        assert_eq!(preexisting_owner_results.len(), 37);
        assert!(
            preexisting_owner_results
                .into_iter()
                .all(|result| result.is_none()),
            "all 37 pre-existing type-assertion owner routes must reject Task 212"
        );
        let payload = extract_source_two_edge_local_object_mode_two_hop_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("the exact two-hop asserted-head source should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 3);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(payload.subject_lookup_ordinal, 1);
        let source_binding = &payload.reserve.bridge.bindings()[0];
        assert_eq!(source_binding.type_spelling, outer_mode);
        assert_eq!(payload.asserted_type.spelling, base_mode);
        let outer_expansion = payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(outer_mode))
            .map(|(_, expansion)| expansion)
            .unwrap();
        let middle_expansion = payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(middle_mode))
            .map(|(_, expansion)| expansion)
            .unwrap();
        let base_expansion = payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(base_mode))
            .map(|(_, expansion)| expansion)
            .unwrap();
        assert_eq!(outer_expansion.radix.spelling, middle_mode);
        assert_eq!(middle_expansion.radix.spelling, base_mode);
        assert_eq!(middle_expansion.radix.head, payload.asserted_type.head);
        assert_eq!(base_expansion.radix.spelling, "object");
        assert_eq!(base_expansion.radix.head, TypeHeadInput::BuiltinObject);
        let TypeHeadInput::Symbol(outer_symbol) = &source_binding.type_head else {
            panic!("outer binding must resolve to a symbol")
        };
        let TypeHeadInput::Symbol(middle_symbol) = &outer_expansion.radix.head else {
            panic!("outer expansion must resolve to the middle symbol")
        };
        let TypeHeadInput::Symbol(base_symbol) = &payload.asserted_type.head else {
            panic!("asserted head must resolve to the base symbol")
        };
        assert_ne!(outer_symbol, middle_symbol);
        assert_ne!(outer_symbol, base_symbol);
        assert_ne!(middle_symbol, base_symbol);

        let exact_output = || {
            source_two_edge_local_object_mode_two_hop_asserted_head_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .unwrap()
        };
        let output = exact_output();
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 212 exact checker output must satisfy every invariant");
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.payload.reserve.mode_expansions.len(), 3);
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert_eq!(output.subject_result_input.spelling, outer_mode);
        assert_eq!(output.asserted_type_input.spelling, base_mode);
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
                    TYPE_ELABORATION_TWO_EDGE_LOCAL_OBJECT_MODE_TWO_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
                ),
                vec![
                    TYPE_ELABORATION_TWO_EDGE_LOCAL_OBJECT_MODE_TWO_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY
                        .to_owned()
                ]
            );
        };
        for removed in [base_mode, middle_mode, outer_mode] {
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
        let asserted_head = invalid.asserted_type_input.head.clone();
        let (_, outer) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(outer_mode))
            .unwrap();
        outer.radix.spelling = base_mode.to_owned();
        outer.radix.head = asserted_head;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        let (_, middle) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(middle_mode))
            .unwrap();
        middle.radix.spelling = "object".to_owned();
        middle.radix.head = TypeHeadInput::BuiltinObject;
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
            .expect("mutating cloned Task 212 outputs must not mutate the exact output");

        let assert_extraction_gap = |ast, context: &str| {
            assert_eq!(
                source_type_elaboration_detail_keys(&ast, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()],
                "{context}"
            );
        };
        let ordered = exact_modes();
        for order in [[0, 2, 1], [1, 0, 2], [1, 2, 0], [2, 0, 1], [2, 1, 0]] {
            assert_extraction_gap(
                mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    vec![ordered[order[0]], ordered[order[1]], ordered[order[2]]],
                    reserve(),
                    theorem,
                ),
                &format!("definition order {order:?}"),
            );
        }
        let mut mode_near_misses = Vec::new();
        for index in 0..3 {
            let mut missing = exact_modes();
            missing.remove(index);
            mode_near_misses.push(missing);
            let mut duplicate = exact_modes();
            duplicate.push(duplicate[index]);
            mode_near_misses.push(duplicate);
            let mut wrong_label = exact_modes();
            wrong_label[index].label = Some("WrongTwoHopObjectModeAssertedHeadDef");
            mode_near_misses.push(wrong_label);
            let mut wrong_pattern = exact_modes();
            wrong_pattern[index].pattern = "OtherTwoHopObjectModeAssertedHead";
            mode_near_misses.push(wrong_pattern);
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
        let mut wrong_terminal = exact_modes();
        wrong_terminal[0].rhs_shape = ReserveTypeShape::Builtin("set");
        mode_near_misses.push(wrong_terminal);
        let mut skipped_outer_link = exact_modes();
        skipped_outer_link[2].rhs_shape = ReserveTypeShape::QualifiedSymbol(base_mode);
        mode_near_misses.push(skipped_outer_link);
        let mut skipped_middle_link = exact_modes();
        skipped_middle_link[1].rhs_shape = ReserveTypeShape::Builtin("object");
        mode_near_misses.push(skipped_middle_link);
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
        for bad_reserve in [
            ReserveTypeShape::QualifiedSymbol(base_mode),
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
                asserted_type: ReserveTypeShape::QualifiedSymbol(middle_mode),
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
                    "OtherTwoHopObjectModeAssertedHead",
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
        let mut connected_deeper_modes = exact_modes();
        connected_deeper_modes.push(mode_definition_with_label(
            deeper_mode,
            "DeeperTwoHopObjectModeAssertedHeadDef",
            ReserveTypeShape::QualifiedSymbol(outer_mode),
        ));
        source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            connected_deeper_modes,
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol(deeper_mode),
            )],
            theorem,
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
            &[
                (base_mode, SymbolKind::Mode),
                (middle_mode, SymbolKind::Mode),
                (outer_mode, SymbolKind::Mode),
            ],
            &[("UnrelatedTwoHopObjectModeAssertedHead", SymbolKind::Mode)],
        );
        assert!(
            extract_source_two_edge_local_object_mode_two_hop_asserted_head(
                &exact,
                module.clone(),
                &unrelated_import,
            )
            .is_some()
        );
        for imported_index in 0..3 {
            let names = [base_mode, middle_mode, outer_mode];
            let locals = names
                .iter()
                .enumerate()
                .filter(|(index, _)| *index != imported_index)
                .map(|(_, spelling)| (*spelling, SymbolKind::Mode))
                .collect::<Vec<_>>();
            for imports in [
                vec![(names[imported_index], SymbolKind::Mode)],
                vec![
                    (names[imported_index], SymbolKind::Mode),
                    (names[imported_index], SymbolKind::Mode),
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
    fn source_three_edge_local_mode_two_hop_asserted_head_consumes_four_expansions() {
        let source_id = source_id(213);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("three_edge_local_mode_two_hop_asserted_head"),
        );
        let base_mode = "BaseThreeEdgeModeTwoHopAssertedHead";
        let inner_mode = "InnerThreeEdgeModeTwoHopAssertedHead";
        let middle_mode = "MiddleThreeEdgeModeTwoHopAssertedHead";
        let outer_mode = "OuterThreeEdgeModeTwoHopAssertedHead";
        let deeper_mode = "DeeperThreeEdgeModeTwoHopAssertedHead";
        let all_modes = [base_mode, inner_mode, middle_mode, outer_mode];
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                (base_mode, SymbolKind::Mode),
                (inner_mode, SymbolKind::Mode),
                (middle_mode, SymbolKind::Mode),
                (outer_mode, SymbolKind::Mode),
                (deeper_mode, SymbolKind::Mode),
                ("OtherThreeEdgeModeTwoHopAssertedHead", SymbolKind::Mode),
            ],
        );
        let theorem = exact_three_edge_local_mode_two_hop_asserted_head_spec();
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    base_mode,
                    "BaseThreeEdgeModeTwoHopAssertedHeadDef",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition_with_label(
                    inner_mode,
                    "InnerThreeEdgeModeTwoHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(base_mode),
                ),
                mode_definition_with_label(
                    middle_mode,
                    "MiddleThreeEdgeModeTwoHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(inner_mode),
                ),
                mode_definition_with_label(
                    outer_mode,
                    "OuterThreeEdgeModeTwoHopAssertedHeadDef",
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
        let payload = extract_source_three_edge_local_mode_two_hop_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("the exact three-edge two-hop asserted-head source should extract");
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
        assert_eq!(base_expansion.radix.spelling, "set");
        assert_eq!(base_expansion.radix.head, TypeHeadInput::BuiltinSet);

        let exact_output = || {
            source_three_edge_local_mode_two_hop_asserted_head_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .unwrap()
        };
        let output = exact_output();
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 213 exact checker output must satisfy every invariant");
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
                    TYPE_ELABORATION_THREE_EDGE_LOCAL_MODE_TWO_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
                ),
                vec![
                    TYPE_ELABORATION_THREE_EDGE_LOCAL_MODE_TWO_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY
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
            .expect("mutating cloned Task 213 outputs must not mutate the exact output");

        // Pairwise-distinct relation symbols must remain explicit. Terminal
        // traversal begins only after the asserted Inner symbol is fixed.
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

        let prior_owner_results = [
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
        ];
        assert_eq!(prior_owner_results.len(), 38);
        assert!(
            prior_owner_results
                .into_iter()
                .all(|result| result.is_none()),
            "all 38 prior type-assertion routes must reject Task 213"
        );

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
                0 => "set",
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
            wrong_label[index].label = Some("WrongThreeEdgeModeTwoHopAssertedHeadDef");
            near_misses.push(wrong_label);
            let mut wrong_spelling = exact_modes();
            wrong_spelling[index].pattern = "OtherThreeEdgeModeTwoHopAssertedHead";
            near_misses.push(wrong_spelling);
            let mut wrong_radix = exact_modes();
            wrong_radix[index].rhs_shape = if index == 0 {
                ReserveTypeShape::Builtin("object")
            } else {
                ReserveTypeShape::QualifiedSymbol("OtherThreeEdgeModeTwoHopAssertedHead")
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

        let mut broken_outer_link = exact_modes();
        broken_outer_link[3].rhs_shape = ReserveTypeShape::QualifiedSymbol(inner_mode);
        let mut broken_middle_link = exact_modes();
        broken_middle_link[2].rhs_shape = ReserveTypeShape::QualifiedSymbol(base_mode);
        let mut broken_tail_link = exact_modes();
        broken_tail_link[1].rhs_shape = ReserveTypeShape::Builtin("set");
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
                asserted_type: ReserveTypeShape::Builtin("set"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::Builtin("object"),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(
                    "OtherThreeEdgeModeTwoHopAssertedHead",
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
            "DeeperThreeEdgeModeTwoHopAssertedHeadDef",
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
            &[("UnrelatedThreeEdgeModeTwoHopAssertedHead", SymbolKind::Mode)],
        );
        assert!(
            extract_source_three_edge_local_mode_two_hop_asserted_head(
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
    fn source_three_edge_local_mode_radix_asserted_head_consumes_immediate_expansion() {
        let source_id = source_id(205);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("three_edge_local_mode_radix_asserted_head"),
        );
        let base_mode = "BaseThreeEdgeModeRadixAssertedHead";
        let inner_mode = "InnerThreeEdgeModeRadixAssertedHead";
        let middle_mode = "MiddleThreeEdgeModeRadixAssertedHead";
        let outer_mode = "OuterThreeEdgeModeRadixAssertedHead";
        let all_modes = [base_mode, inner_mode, middle_mode, outer_mode];
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                (base_mode, SymbolKind::Mode),
                (inner_mode, SymbolKind::Mode),
                (middle_mode, SymbolKind::Mode),
                (outer_mode, SymbolKind::Mode),
                ("DeeperThreeEdgeModeRadixAssertedHead", SymbolKind::Mode),
                ("OtherThreeEdgeModeRadixAssertedHead", SymbolKind::Mode),
            ],
        );
        let theorem = exact_three_edge_local_mode_radix_asserted_head_spec();
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    base_mode,
                    "BaseThreeEdgeModeRadixAssertedHeadDef",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition_with_label(
                    inner_mode,
                    "InnerThreeEdgeModeRadixAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(base_mode),
                ),
                mode_definition_with_label(
                    middle_mode,
                    "MiddleThreeEdgeModeRadixAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(inner_mode),
                ),
                mode_definition_with_label(
                    outer_mode,
                    "OuterThreeEdgeModeRadixAssertedHeadDef",
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

        let payload = extract_source_three_edge_local_mode_radix_asserted_head(
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

        let output = source_three_edge_local_mode_radix_asserted_head_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact Task 205 source should reach the checker");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 205 checker payload invariants should hold");
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
            source_three_edge_local_mode_radix_asserted_head_output(
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
                    TYPE_ELABORATION_THREE_EDGE_LOCAL_MODE_RADIX_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
                ),
                vec![
                    TYPE_ELABORATION_THREE_EDGE_LOCAL_MODE_RADIX_ASSERTED_HEAD_INVALID_PAYLOAD_KEY
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
            .expect("corrupting cloned Task 205 outputs must not mutate the exact output");

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
            wrong_label[index].label = Some("WrongThreeEdgeModeRadixAssertedHeadDef");
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
            wrong_spelling[index].pattern = "OtherThreeEdgeModeRadixAssertedHead";
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
                ReserveTypeShape::QualifiedSymbol("OtherThreeEdgeModeRadixAssertedHead")
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
            "DeeperThreeEdgeModeRadixAssertedHead",
            "DeeperThreeEdgeModeRadixAssertedHeadDef",
            ReserveTypeShape::QualifiedSymbol(outer_mode),
        ));
        assert_extraction_gap(
            mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                deeper_modes,
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbol("DeeperThreeEdgeModeRadixAssertedHead"),
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
            ReserveTypeShape::Builtin("set"),
            ReserveTypeShape::Builtin("object"),
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
            &[("UnrelatedThreeEdgeModeRadixAssertedHead", SymbolKind::Mode)],
        );
        assert!(
            extract_source_three_edge_local_mode_radix_asserted_head(
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
        assert_task_205_route_isolation(source_id, module);
    }

    fn assert_task_205_route_isolation(source_id: SourceId, module: ResolverModuleId) {
        let assert_isolated =
            |ast: &SurfaceAst, symbols: &SymbolEnv, owner_extracts: bool, task: &str| {
                assert!(owner_extracts, "{task} owner route should extract");
                assert!(
                    extract_source_three_edge_local_mode_radix_asserted_head(
                        ast,
                        module.clone(),
                        symbols,
                    )
                    .is_none(),
                    "Task 205 must reject {task}"
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

        assert_task_205_object_route_isolation(source_id, module.clone(), &assert_isolated);
    }

    fn assert_task_205_object_route_isolation<F>(
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
