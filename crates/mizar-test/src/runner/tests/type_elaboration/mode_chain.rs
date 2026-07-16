    #[test]
    fn source_local_mode_reserved_variable_membership_consumes_real_expansion() {
        let source_id = source_id(139);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("local_mode_reserved_variable_membership"),
        );
        let symbols =
            source_local_symbols_env(module.clone(), &[("LocalModeMembership", SymbolKind::Mode)]);
        let theorem = IdentifierBinaryTheoremSpec {
            status: None,
            label: "LocalModeReservedVariableMembershipPayloadBoundary",
            left: "x",
            operator: "in",
            right: "y",
            recovered_label: false,
        };
        let mode = || {
            mode_definition_with_label(
                "LocalModeMembership",
                "LocalModeMembershipDef",
                ReserveTypeShape::Builtin("set"),
            )
        };
        let reserve = || {
            vec![
                reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbol("LocalModeMembership"),
                ),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
            ]
        };
        let exact = mode_then_reserve_identifier_binary_theorem_ast(
            source_id,
            [mode()],
            reserve(),
            theorem,
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        let payload = extract_source_local_mode_reserved_variable_membership(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact local-mode reserved-variable membership should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 1);
        assert_eq!(payload.reserve.bridge.bindings().len(), 2);
        assert_eq!(payload.reserve.bridge.bindings()[0].spelling, "x");
        assert_eq!(
            payload.reserve.bridge.bindings()[0].type_spelling,
            "LocalModeMembership"
        );
        assert!(matches!(
            payload.reserve.bridge.bindings()[0].type_head,
            TypeHeadInput::Symbol(_)
        ));
        assert_eq!(payload.reserve.bridge.bindings()[1].spelling, "y");
        assert_eq!(
            payload.reserve.bridge.bindings()[1].type_head,
            TypeHeadInput::BuiltinSet
        );
        assert_ne!(
            payload.reserve.bridge.bindings()[0].type_range,
            payload.reserve.bridge.bindings()[1].type_range
        );
        assert_eq!(payload.left_lookup_ordinal, 2);
        assert_eq!(payload.right_lookup_ordinal, 3);

        let output =
            source_local_mode_reserved_variable_membership_output(&exact, module.clone(), &symbols)
                .expect("exact local-mode membership should reach TermFormulaChecker");
        assert_source_reserved_variable_formula_output(&output)
            .expect("local-mode membership invariants should hold");
        assert_eq!(output.left_binding, BindingId::new(0));
        assert_eq!(output.right_binding, BindingId::new(1));
        assert!(matches!(
            output.left_result_input.head,
            TypeHeadInput::Symbol(_)
        ));
        assert_eq!(output.right_result_input.head, TypeHeadInput::BuiltinSet);
        assert!(output.left_expected_input.is_none());
        assert_eq!(
            output
                .right_expected_input
                .as_ref()
                .expect("right membership expected input should exist")
                .head,
            TypeHeadInput::BuiltinSet
        );
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
            .expect("one normalized set identity should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_eq!(normalized.source.range, terminal.radix.source_range);
        let formula = output
            .term_formula
            .formulas()
            .iter()
            .map(|(_, formula)| formula)
            .next()
            .expect("membership formula should be checked");
        assert_eq!(formula.kind, FormulaKind::Membership);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert_eq!(formula.expected_types.len(), 1);
        assert_eq!(formula.expected_types[0].term, output.payload.right_site);
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
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
                "local-mode-reserved-variable-membership-left-result".to_owned(),
                "local-mode-reserved-variable-membership-right-expected".to_owned(),
                "local-mode-reserved-variable-membership-right-result".to_owned(),
            ])
        );

        let mut invalid_expansion =
            source_local_mode_reserved_variable_membership_output(&exact, module.clone(), &symbols)
                .expect("exact source should produce a corruption target");
        invalid_expansion.payload.reserve.mode_expansions.clear();
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&invalid_expansion),
            vec![
                TYPE_ELABORATION_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY
                    .to_owned()
            ]
        );

        let mut invalid_expected =
            source_local_mode_reserved_variable_membership_output(&exact, module.clone(), &symbols)
                .expect("exact source should produce a second corruption target");
        invalid_expected
            .right_expected_input
            .as_mut()
            .expect("right expected input should exist")
            .head = TypeHeadInput::BuiltinObject;
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&invalid_expected),
            vec![
                TYPE_ELABORATION_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY
                    .to_owned()
            ]
        );

        for definition in [
            mode_definition_with_label(
                "LocalModeMembership",
                "LocalModeMembershipDef",
                ReserveTypeShape::Builtin("object"),
            ),
            mode_definition_with_label(
                "LocalModeMembership",
                "LocalModeMembershipDef",
                ReserveTypeShape::AttributedSet,
            ),
            contextual_mode_definition("LocalModeMembership", ReserveTypeShape::Builtin("set")),
            parameterized_mode_definition("LocalModeMembership", ReserveTypeShape::Builtin("set")),
            recovered_mode_definition("LocalModeMembership", ReserveTypeShape::Builtin("set")),
        ] {
            let near_miss = mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [definition],
                reserve(),
                theorem,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }

        let near_misses = [
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                Vec::<ModeDefinitionSpec>::new(),
                reserve(),
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode_definition(
                    "OtherMode",
                    ReserveTypeShape::Builtin("set"),
                )],
                reserve(),
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode_definition_with_label(
                    "LocalModeMembership",
                    "OtherDef",
                    ReserveTypeShape::Builtin("set"),
                )],
                reserve(),
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode(), mode()],
                reserve(),
                theorem,
            ),
            reserve_then_mode_identifier_binary_theorem_ast(source_id, reserve(), mode(), theorem),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [
                    mode_definition("BaseMode", ReserveTypeShape::Builtin("set")),
                    mode_definition_with_label(
                        "LocalModeMembership",
                        "LocalModeMembershipDef",
                        ReserveTypeShape::QualifiedSymbol("BaseMode"),
                    ),
                ],
                reserve(),
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode()],
                vec![
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                    reserve_item(
                        vec!["x"],
                        ReserveTypeShape::QualifiedSymbol("LocalModeMembership"),
                    ),
                ],
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode()],
                vec![reserve_item(
                    vec!["x", "y"],
                    ReserveTypeShape::QualifiedSymbol("LocalModeMembership"),
                )],
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode()],
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::Builtin("set")),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                ],
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode()],
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbol("LocalModeMembership"),
                )],
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode()],
                {
                    let mut items = reserve();
                    items.push(reserve_item(vec!["z"], ReserveTypeShape::Builtin("set")));
                    items
                },
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode()],
                vec![
                    reserve_item(
                        vec!["x"],
                        ReserveTypeShape::QualifiedSymbolWithArgs("LocalModeMembership"),
                    ),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                ],
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode()],
                reserve(),
                IdentifierBinaryTheoremSpec {
                    left: "y",
                    right: "x",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode()],
                reserve(),
                IdentifierBinaryTheoremSpec {
                    right: "x",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode()],
                reserve(),
                IdentifierBinaryTheoremSpec {
                    label: "OtherPayloadBoundary",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode()],
                reserve(),
                IdentifierBinaryTheoremSpec {
                    operator: "=",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode()],
                reserve(),
                IdentifierBinaryTheoremSpec {
                    status: Some("registration"),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode()],
                reserve(),
                IdentifierBinaryTheoremSpec {
                    recovered_label: true,
                    ..theorem
                },
            ),
            modes_then_empty_definition_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode()],
                reserve(),
                theorem,
            ),
        ];
        for near_miss in near_misses {
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
    fn source_local_object_mode_reserved_variable_membership_consumes_real_expansion() {
        let source_id = source_id(140);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("local_object_mode_reserved_variable_membership"),
        );
        let symbols = source_local_symbols_env(
            module.clone(),
            &[("LocalObjectModeMembership", SymbolKind::Mode)],
        );
        let theorem = IdentifierBinaryTheoremSpec {
            status: None,
            label: "LocalObjectModeReservedVariableMembershipPayloadBoundary",
            left: "x",
            operator: "in",
            right: "y",
            recovered_label: false,
        };
        let mode = || {
            mode_definition_with_label(
                "LocalObjectModeMembership",
                "LocalObjectModeMembershipDef",
                ReserveTypeShape::Builtin("object"),
            )
        };
        let reserve = || {
            vec![
                reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbol("LocalObjectModeMembership"),
                ),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
            ]
        };
        let exact = mode_then_reserve_identifier_binary_theorem_ast(
            source_id,
            [mode()],
            reserve(),
            theorem,
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        let payload = extract_source_local_object_mode_reserved_variable_membership(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact local object-mode reserved-variable membership should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 1);
        assert_eq!(payload.reserve.bridge.bindings().len(), 2);
        assert_eq!(payload.reserve.bridge.bindings()[0].spelling, "x");
        assert_eq!(
            payload.reserve.bridge.bindings()[0].type_spelling,
            "LocalObjectModeMembership"
        );
        assert!(matches!(
            payload.reserve.bridge.bindings()[0].type_head,
            TypeHeadInput::Symbol(_)
        ));
        assert_eq!(payload.reserve.bridge.bindings()[1].spelling, "y");
        assert_eq!(
            payload.reserve.bridge.bindings()[1].type_head,
            TypeHeadInput::BuiltinSet
        );
        assert_ne!(
            payload.reserve.bridge.bindings()[0].type_range,
            payload.reserve.bridge.bindings()[1].type_range
        );
        assert_eq!(payload.left_lookup_ordinal, 2);
        assert_eq!(payload.right_lookup_ordinal, 3);

        let output = source_local_object_mode_reserved_variable_membership_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact local object-mode membership should reach TermFormulaChecker");
        assert_source_reserved_variable_formula_output(&output)
            .expect("local object-mode membership invariants should hold");
        assert_eq!(output.left_binding, BindingId::new(0));
        assert_eq!(output.right_binding, BindingId::new(1));
        assert!(matches!(
            output.left_result_input.head,
            TypeHeadInput::Symbol(_)
        ));
        assert_eq!(output.right_result_input.head, TypeHeadInput::BuiltinSet);
        assert!(output.left_expected_input.is_none());
        assert_eq!(
            output
                .right_expected_input
                .as_ref()
                .expect("right membership expected input should exist")
                .head,
            TypeHeadInput::BuiltinSet
        );
        assert_eq!(output.term_formula.normalized_types().len(), 2);
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .values()
            .next()
            .expect("real direct expansion should exist");
        let object_normalized = output
            .term_formula
            .normalized_types()
            .iter()
            .map(|(_, normalized)| normalized)
            .find(|normalized| normalized.head == TypeHeadRef::BuiltinObject)
            .expect("one normalized object identity should exist");
        assert_eq!(object_normalized.source.range, terminal.radix.source_range);
        let set_normalized = output
            .term_formula
            .normalized_types()
            .iter()
            .map(|(_, normalized)| normalized)
            .find(|normalized| normalized.head == TypeHeadRef::BuiltinSet)
            .expect("one normalized set identity should exist");
        assert_eq!(
            set_normalized.source.range,
            output.payload.reserve.bridge.bindings()[1].type_range
        );
        let formula = output
            .term_formula
            .formulas()
            .iter()
            .map(|(_, formula)| formula)
            .next()
            .expect("membership formula should be checked");
        assert_eq!(formula.kind, FormulaKind::Membership);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert_eq!(formula.expected_types.len(), 1);
        assert_eq!(formula.expected_types[0].term, output.payload.right_site);
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
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
                "local-object-mode-reserved-variable-membership-left-result".to_owned(),
                "local-object-mode-reserved-variable-membership-right-expected".to_owned(),
                "local-object-mode-reserved-variable-membership-right-result".to_owned(),
            ])
        );

        let mut invalid_expansion = source_local_object_mode_reserved_variable_membership_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce a corruption target");
        invalid_expansion.payload.reserve.mode_expansions.clear();
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&invalid_expansion),
            vec![
                TYPE_ELABORATION_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY
                    .to_owned()
            ]
        );

        let mut invalid_expected = source_local_object_mode_reserved_variable_membership_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce a second corruption target");
        invalid_expected
            .right_expected_input
            .as_mut()
            .expect("right expected input should exist")
            .head = TypeHeadInput::BuiltinObject;
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&invalid_expected),
            vec![
                TYPE_ELABORATION_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY
                    .to_owned()
            ]
        );

        for definition in [
            mode_definition_with_label(
                "LocalObjectModeMembership",
                "LocalObjectModeMembershipDef",
                ReserveTypeShape::Builtin("set"),
            ),
            mode_definition_with_label(
                "LocalObjectModeMembership",
                "LocalObjectModeMembershipDef",
                ReserveTypeShape::AttributedObject,
            ),
            contextual_mode_definition(
                "LocalObjectModeMembership",
                ReserveTypeShape::Builtin("object"),
            ),
            parameterized_mode_definition(
                "LocalObjectModeMembership",
                ReserveTypeShape::Builtin("object"),
            ),
            recovered_mode_definition(
                "LocalObjectModeMembership",
                ReserveTypeShape::Builtin("object"),
            ),
        ] {
            let near_miss = mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [definition],
                reserve(),
                theorem,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }

        let near_misses = [
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                Vec::<ModeDefinitionSpec>::new(),
                reserve(),
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode_definition(
                    "OtherMode",
                    ReserveTypeShape::Builtin("object"),
                )],
                reserve(),
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode_definition_with_label(
                    "LocalObjectModeMembership",
                    "OtherDef",
                    ReserveTypeShape::Builtin("object"),
                )],
                reserve(),
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode(), mode()],
                reserve(),
                theorem,
            ),
            reserve_then_mode_identifier_binary_theorem_ast(source_id, reserve(), mode(), theorem),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [
                    mode_definition("BaseMode", ReserveTypeShape::Builtin("object")),
                    mode_definition_with_label(
                        "LocalObjectModeMembership",
                        "LocalObjectModeMembershipDef",
                        ReserveTypeShape::QualifiedSymbol("BaseMode"),
                    ),
                ],
                reserve(),
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode()],
                vec![
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                    reserve_item(
                        vec!["x"],
                        ReserveTypeShape::QualifiedSymbol("LocalObjectModeMembership"),
                    ),
                ],
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode()],
                vec![reserve_item(
                    vec!["x", "y"],
                    ReserveTypeShape::QualifiedSymbol("LocalObjectModeMembership"),
                )],
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode()],
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::Builtin("object")),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                ],
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode()],
                vec![
                    reserve_item(
                        vec!["x"],
                        ReserveTypeShape::QualifiedSymbol("LocalObjectModeMembership"),
                    ),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("object")),
                ],
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode()],
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbol("LocalObjectModeMembership"),
                )],
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode()],
                {
                    let mut items = reserve();
                    items.push(reserve_item(vec!["z"], ReserveTypeShape::Builtin("set")));
                    items
                },
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode()],
                vec![
                    reserve_item(
                        vec!["x"],
                        ReserveTypeShape::QualifiedSymbolWithArgs("LocalObjectModeMembership"),
                    ),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                ],
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode()],
                reserve(),
                IdentifierBinaryTheoremSpec {
                    left: "y",
                    right: "x",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode()],
                reserve(),
                IdentifierBinaryTheoremSpec {
                    right: "x",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode()],
                reserve(),
                IdentifierBinaryTheoremSpec {
                    label: "OtherPayloadBoundary",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode()],
                reserve(),
                IdentifierBinaryTheoremSpec {
                    operator: "=",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode()],
                reserve(),
                IdentifierBinaryTheoremSpec {
                    status: Some("registration"),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode()],
                reserve(),
                IdentifierBinaryTheoremSpec {
                    recovered_label: true,
                    ..theorem
                },
            ),
            modes_then_empty_definition_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode()],
                reserve(),
                theorem,
            ),
        ];
        for near_miss in near_misses {
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
    fn source_chained_local_mode_reserved_variable_membership_consumes_both_expansions() {
        let source_id = source_id(141);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("chained_local_mode_reserved_variable_membership"),
        );
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseModeMembership", SymbolKind::Mode),
                ("ChainModeMembership", SymbolKind::Mode),
                ("InnerModeMembership", SymbolKind::Mode),
            ],
        );
        let theorem = IdentifierBinaryTheoremSpec {
            status: None,
            label: "ChainedLocalModeReservedVariableMembershipPayloadBoundary",
            left: "x",
            operator: "in",
            right: "y",
            recovered_label: false,
        };
        let exact_modes = || {
            vec![
                mode_definition("BaseModeMembership", ReserveTypeShape::Builtin("set")),
                mode_definition(
                    "ChainModeMembership",
                    ReserveTypeShape::QualifiedSymbol("BaseModeMembership"),
                ),
            ]
        };
        let exact_reserves = || {
            vec![
                reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbol("ChainModeMembership"),
                ),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
            ]
        };
        let exact = mode_then_reserve_identifier_binary_theorem_ast(
            source_id,
            exact_modes(),
            exact_reserves(),
            theorem,
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        let payload = extract_source_chained_local_mode_reserved_variable_membership(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact chained local-mode membership should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 2);
        assert_eq!(payload.reserve.bridge.bindings().len(), 2);
        assert_eq!(
            payload.reserve.bridge.bindings()[0].type_spelling,
            "ChainModeMembership"
        );
        assert_eq!(payload.reserve.bridge.bindings()[1].type_spelling, "set");
        assert_ne!(
            payload.reserve.bridge.bindings()[0].type_range,
            payload.reserve.bridge.bindings()[1].type_range
        );
        assert_eq!(payload.left_lookup_ordinal, 2);
        assert_eq!(payload.right_lookup_ordinal, 3);

        let output = source_chained_local_mode_reserved_variable_membership_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact chained local-mode membership should reach TermFormulaChecker");
        assert_source_reserved_variable_formula_output(&output)
            .expect("chained local-mode membership invariants should hold");
        assert_eq!(output.left_binding, BindingId::new(0));
        assert_eq!(output.right_binding, BindingId::new(1));
        assert_eq!(output.left_result_input.spelling, "ChainModeMembership");
        assert!(matches!(
            output.left_result_input.head,
            TypeHeadInput::Symbol(_)
        ));
        assert_eq!(output.right_result_input.spelling, "set");
        assert!(matches!(
            output.right_result_input.head,
            TypeHeadInput::BuiltinSet
        ));
        assert!(output.left_expected_input.is_none());
        let right_expected = output
            .right_expected_input
            .as_ref()
            .expect("right expected input should exist");
        assert_eq!(right_expected.spelling, "set");
        assert!(matches!(right_expected.head, TypeHeadInput::BuiltinSet));
        assert_eq!(
            output.right_result_input.source_range,
            right_expected.source_range
        );
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("BaseModeMembership"))
            .map(|(_, expansion)| &expansion.radix)
            .expect("base mode terminal expansion should exist");
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
        let formula = output
            .term_formula
            .formulas()
            .iter()
            .map(|(_, formula)| formula)
            .next()
            .expect("membership formula should be checked");
        assert_eq!(formula.kind, FormulaKind::Membership);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert_eq!(formula.expected_types.len(), 1);
        assert_eq!(formula.expected_types[0].term, output.payload.right_site);
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
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
                "chained-local-mode-reserved-variable-membership-left-result".to_owned(),
                "chained-local-mode-reserved-variable-membership-right-expected".to_owned(),
                "chained-local-mode-reserved-variable-membership-right-result".to_owned(),
            ])
        );

        for removed in ["BaseModeMembership", "ChainModeMembership"] {
            let mut invalid = source_chained_local_mode_reserved_variable_membership_output(
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
            assert_eq!(
                source_reserved_variable_formula_output_detail_keys(&invalid),
                vec![
                    TYPE_ELABORATION_CHAINED_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY
                        .to_owned()
                ]
            );
        }

        let mut invalid_expected = source_chained_local_mode_reserved_variable_membership_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce a right-expected corruption target");
        invalid_expected
            .right_expected_input
            .as_mut()
            .expect("right expected input should exist")
            .head = TypeHeadInput::BuiltinObject;
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&invalid_expected),
            vec![
                TYPE_ELABORATION_CHAINED_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY
                    .to_owned()
            ]
        );

        let near_miss_modes = [
            vec![mode_definition(
                "ChainModeMembership",
                ReserveTypeShape::QualifiedSymbol("BaseModeMembership"),
            )],
            vec![mode_definition(
                "BaseModeMembership",
                ReserveTypeShape::Builtin("set"),
            )],
            vec![
                mode_definition("BaseModeMembership", ReserveTypeShape::Builtin("set")),
                mode_definition("BaseModeMembership", ReserveTypeShape::Builtin("set")),
                mode_definition(
                    "ChainModeMembership",
                    ReserveTypeShape::QualifiedSymbol("BaseModeMembership"),
                ),
            ],
            vec![
                recovered_mode_definition("BaseModeMembership", ReserveTypeShape::Builtin("set")),
                mode_definition(
                    "ChainModeMembership",
                    ReserveTypeShape::QualifiedSymbol("BaseModeMembership"),
                ),
            ],
            vec![
                contextual_mode_definition("BaseModeMembership", ReserveTypeShape::Builtin("set")),
                mode_definition(
                    "ChainModeMembership",
                    ReserveTypeShape::QualifiedSymbol("BaseModeMembership"),
                ),
            ],
            vec![
                mode_definition_with_label(
                    "BaseModeMembership",
                    "OtherDef",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition(
                    "ChainModeMembership",
                    ReserveTypeShape::QualifiedSymbol("BaseModeMembership"),
                ),
            ],
            vec![
                mode_definition("BaseModeMembership", ReserveTypeShape::Builtin("set")),
                mode_definition_with_label(
                    "ChainModeMembership",
                    "OtherDef",
                    ReserveTypeShape::QualifiedSymbol("BaseModeMembership"),
                ),
            ],
            vec![
                mode_definition("BaseModeMembership", ReserveTypeShape::Builtin("set")),
                parameterized_mode_definition(
                    "ChainModeMembership",
                    ReserveTypeShape::QualifiedSymbol("BaseModeMembership"),
                ),
            ],
            vec![
                mode_definition("BaseModeMembership", ReserveTypeShape::AttributedSet),
                mode_definition(
                    "ChainModeMembership",
                    ReserveTypeShape::QualifiedSymbol("BaseModeMembership"),
                ),
            ],
            vec![
                mode_definition("BaseModeMembership", ReserveTypeShape::Builtin("object")),
                mode_definition(
                    "ChainModeMembership",
                    ReserveTypeShape::QualifiedSymbol("BaseModeMembership"),
                ),
            ],
            vec![
                mode_definition(
                    "ChainModeMembership",
                    ReserveTypeShape::QualifiedSymbol("BaseModeMembership"),
                ),
                mode_definition("BaseModeMembership", ReserveTypeShape::Builtin("set")),
            ],
            vec![
                mode_definition("BaseModeMembership", ReserveTypeShape::Builtin("set")),
                mode_definition(
                    "ChainModeMembership",
                    ReserveTypeShape::QualifiedSymbolWithArgs("BaseModeMembership"),
                ),
            ],
            vec![
                mode_definition("BaseModeMembership", ReserveTypeShape::Builtin("set")),
                mode_definition("ChainModeMembership", ReserveTypeShape::Builtin("set")),
            ],
            vec![
                mode_definition(
                    "BaseModeMembership",
                    ReserveTypeShape::QualifiedSymbol("ChainModeMembership"),
                ),
                mode_definition(
                    "ChainModeMembership",
                    ReserveTypeShape::QualifiedSymbol("BaseModeMembership"),
                ),
            ],
            vec![
                mode_definition("BaseModeMembership", ReserveTypeShape::Builtin("set")),
                mode_definition(
                    "InnerModeMembership",
                    ReserveTypeShape::QualifiedSymbol("BaseModeMembership"),
                ),
                mode_definition(
                    "ChainModeMembership",
                    ReserveTypeShape::QualifiedSymbol("InnerModeMembership"),
                ),
            ],
        ];
        for modes in near_miss_modes {
            let near_miss = mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                modes,
                exact_reserves(),
                theorem,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }

        let near_miss_reserves = [
            vec![
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbol("ChainModeMembership"),
                ),
            ],
            vec![reserve_item(
                vec!["x", "y"],
                ReserveTypeShape::QualifiedSymbol("ChainModeMembership"),
            )],
            vec![
                reserve_item(vec!["x"], ReserveTypeShape::Builtin("set")),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
            ],
            vec![
                reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbol("ChainModeMembership"),
                ),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("object")),
            ],
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("ChainModeMembership"),
            )],
            vec![
                reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbol("ChainModeMembership"),
                ),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                reserve_item(vec!["z"], ReserveTypeShape::Builtin("set")),
            ],
            vec![
                reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbolWithArgs("ChainModeMembership"),
                ),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
            ],
        ];
        for reserves in near_miss_reserves {
            let near_miss = mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                reserves,
                theorem,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }

        for near_miss_theorem in [
            IdentifierBinaryTheoremSpec {
                left: "y",
                right: "x",
                ..theorem
            },
            IdentifierBinaryTheoremSpec {
                right: "x",
                ..theorem
            },
            IdentifierBinaryTheoremSpec {
                label: "OtherPayloadBoundary",
                ..theorem
            },
            IdentifierBinaryTheoremSpec {
                operator: "=",
                ..theorem
            },
            IdentifierBinaryTheoremSpec {
                status: Some("open"),
                ..theorem
            },
            IdentifierBinaryTheoremSpec {
                recovered_label: true,
                ..theorem
            },
        ] {
            let near_miss = mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                exact_reserves(),
                near_miss_theorem,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }

        let extra_item = modes_then_empty_definition_reserve_identifier_binary_theorem_ast(
            source_id,
            exact_modes(),
            exact_reserves(),
            theorem,
        );
        assert_eq!(
            source_type_elaboration_detail_keys(&extra_item, module.clone(), &symbols),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );
        let unresolved_symbols =
            source_local_symbols_env(module.clone(), &[("BaseModeMembership", SymbolKind::Mode)]);
        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module, &unresolved_symbols),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );
    }

    #[test]
    fn source_chained_local_object_mode_reserved_variable_membership_consumes_both_expansions() {
        let source_id = source_id(142);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("chained_local_object_mode_reserved_variable_membership"),
        );
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseObjectModeMembership", SymbolKind::Mode),
                ("ChainObjectModeMembership", SymbolKind::Mode),
                ("InnerObjectModeMembership", SymbolKind::Mode),
            ],
        );
        let theorem = IdentifierBinaryTheoremSpec {
            status: None,
            label: "ChainedLocalObjectModeReservedVariableMembershipPayloadBoundary",
            left: "x",
            operator: "in",
            right: "y",
            recovered_label: false,
        };
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    "BaseObjectModeMembership",
                    "BaseObjectModeMembershipDef",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition_with_label(
                    "ChainObjectModeMembership",
                    "ChainObjectModeMembershipDef",
                    ReserveTypeShape::QualifiedSymbol("BaseObjectModeMembership"),
                ),
            ]
        };
        let exact_reserves = || {
            vec![
                reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbol("ChainObjectModeMembership"),
                ),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
            ]
        };
        let exact = mode_then_reserve_identifier_binary_theorem_ast(
            source_id,
            exact_modes(),
            exact_reserves(),
            theorem,
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        let payload = extract_source_chained_local_object_mode_reserved_variable_membership(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact chained local object-mode membership should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 2);
        assert_eq!(payload.reserve.bridge.bindings().len(), 2);
        assert_eq!(
            payload.reserve.bridge.bindings()[0].type_spelling,
            "ChainObjectModeMembership"
        );
        assert_eq!(payload.reserve.bridge.bindings()[1].type_spelling, "set");
        assert_ne!(
            payload.reserve.bridge.bindings()[0].type_range,
            payload.reserve.bridge.bindings()[1].type_range
        );
        assert_eq!(payload.left_lookup_ordinal, 2);
        assert_eq!(payload.right_lookup_ordinal, 3);

        let output = source_chained_local_object_mode_reserved_variable_membership_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact chained local object-mode membership should reach TermFormulaChecker");
        assert_source_reserved_variable_formula_output(&output)
            .expect("chained local object-mode membership invariants should hold");
        assert_eq!(output.left_binding, BindingId::new(0));
        assert_eq!(output.right_binding, BindingId::new(1));
        assert_eq!(
            output.left_result_input.spelling,
            "ChainObjectModeMembership"
        );
        assert!(matches!(
            output.left_result_input.head,
            TypeHeadInput::Symbol(_)
        ));
        assert_eq!(output.right_result_input.spelling, "set");
        assert_eq!(output.right_result_input.head, TypeHeadInput::BuiltinSet);
        assert!(output.left_expected_input.is_none());
        let right_expected = output
            .right_expected_input
            .as_ref()
            .expect("right expected input should exist");
        assert_eq!(right_expected.head, TypeHeadInput::BuiltinSet);
        assert_eq!(
            output.right_result_input.source_range,
            right_expected.source_range
        );
        assert_eq!(output.term_formula.normalized_types().len(), 2);
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("BaseObjectModeMembership")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("base object-mode terminal expansion should exist");
        let object_normalized = output
            .term_formula
            .normalized_types()
            .iter()
            .map(|(_, normalized)| normalized)
            .find(|normalized| normalized.head == TypeHeadRef::BuiltinObject)
            .expect("one normalized object identity should exist");
        assert_eq!(object_normalized.source.range, terminal.source_range);
        assert_eq!(object_normalized.source.spelling, terminal.spelling);
        let set_normalized = output
            .term_formula
            .normalized_types()
            .iter()
            .map(|(_, normalized)| normalized)
            .find(|normalized| normalized.head == TypeHeadRef::BuiltinSet)
            .expect("one normalized set identity should exist");
        assert_eq!(
            set_normalized.source.range,
            output.payload.reserve.bridge.bindings()[1].type_range
        );
        let formula = output
            .term_formula
            .formulas()
            .iter()
            .map(|(_, formula)| formula)
            .next()
            .expect("membership formula should be checked");
        assert_eq!(formula.kind, FormulaKind::Membership);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert_eq!(formula.expected_types.len(), 1);
        assert_eq!(formula.expected_types[0].term, output.payload.right_site);
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
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
                "chained-local-object-mode-reserved-variable-membership-left-result".to_owned(),
                "chained-local-object-mode-reserved-variable-membership-right-expected".to_owned(),
                "chained-local-object-mode-reserved-variable-membership-right-result".to_owned(),
            ])
        );

        for removed in ["BaseObjectModeMembership", "ChainObjectModeMembership"] {
            let mut invalid = source_chained_local_object_mode_reserved_variable_membership_output(
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
            assert_eq!(
                source_reserved_variable_formula_output_detail_keys(&invalid),
                vec![
                    TYPE_ELABORATION_CHAINED_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY
                        .to_owned()
                ]
            );
        }

        let mut invalid_expected =
            source_chained_local_object_mode_reserved_variable_membership_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce a right-expected corruption target");
        invalid_expected
            .right_expected_input
            .as_mut()
            .expect("right expected input should exist")
            .head = TypeHeadInput::BuiltinObject;
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&invalid_expected),
            vec![
                TYPE_ELABORATION_CHAINED_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY
                    .to_owned()
            ]
        );

        let near_miss_modes = [
            vec![mode_definition(
                "ChainObjectModeMembership",
                ReserveTypeShape::QualifiedSymbol("BaseObjectModeMembership"),
            )],
            vec![mode_definition(
                "BaseObjectModeMembership",
                ReserveTypeShape::Builtin("object"),
            )],
            vec![
                mode_definition(
                    "BaseObjectModeMembership",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition(
                    "BaseObjectModeMembership",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition(
                    "ChainObjectModeMembership",
                    ReserveTypeShape::QualifiedSymbol("BaseObjectModeMembership"),
                ),
            ],
            vec![
                recovered_mode_definition(
                    "BaseObjectModeMembership",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition(
                    "ChainObjectModeMembership",
                    ReserveTypeShape::QualifiedSymbol("BaseObjectModeMembership"),
                ),
            ],
            vec![
                contextual_mode_definition(
                    "BaseObjectModeMembership",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition(
                    "ChainObjectModeMembership",
                    ReserveTypeShape::QualifiedSymbol("BaseObjectModeMembership"),
                ),
            ],
            vec![
                mode_definition_with_label(
                    "BaseObjectModeMembership",
                    "OtherDef",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition(
                    "ChainObjectModeMembership",
                    ReserveTypeShape::QualifiedSymbol("BaseObjectModeMembership"),
                ),
            ],
            vec![
                mode_definition(
                    "BaseObjectModeMembership",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition_with_label(
                    "ChainObjectModeMembership",
                    "OtherDef",
                    ReserveTypeShape::QualifiedSymbol("BaseObjectModeMembership"),
                ),
            ],
            vec![
                mode_definition(
                    "BaseObjectModeMembership",
                    ReserveTypeShape::Builtin("object"),
                ),
                parameterized_mode_definition(
                    "ChainObjectModeMembership",
                    ReserveTypeShape::QualifiedSymbol("BaseObjectModeMembership"),
                ),
            ],
            vec![
                mode_definition(
                    "BaseObjectModeMembership",
                    ReserveTypeShape::AttributedObject,
                ),
                mode_definition(
                    "ChainObjectModeMembership",
                    ReserveTypeShape::QualifiedSymbol("BaseObjectModeMembership"),
                ),
            ],
            vec![
                mode_definition("BaseObjectModeMembership", ReserveTypeShape::Builtin("set")),
                mode_definition(
                    "ChainObjectModeMembership",
                    ReserveTypeShape::QualifiedSymbol("BaseObjectModeMembership"),
                ),
            ],
            vec![
                mode_definition(
                    "ChainObjectModeMembership",
                    ReserveTypeShape::QualifiedSymbol("BaseObjectModeMembership"),
                ),
                mode_definition(
                    "BaseObjectModeMembership",
                    ReserveTypeShape::Builtin("object"),
                ),
            ],
            vec![
                mode_definition(
                    "BaseObjectModeMembership",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition(
                    "ChainObjectModeMembership",
                    ReserveTypeShape::QualifiedSymbolWithArgs("BaseObjectModeMembership"),
                ),
            ],
            vec![
                mode_definition(
                    "BaseObjectModeMembership",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition(
                    "ChainObjectModeMembership",
                    ReserveTypeShape::Builtin("object"),
                ),
            ],
            vec![
                mode_definition(
                    "BaseObjectModeMembership",
                    ReserveTypeShape::QualifiedSymbol("ChainObjectModeMembership"),
                ),
                mode_definition(
                    "ChainObjectModeMembership",
                    ReserveTypeShape::QualifiedSymbol("BaseObjectModeMembership"),
                ),
            ],
            vec![
                mode_definition(
                    "BaseObjectModeMembership",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition(
                    "InnerObjectModeMembership",
                    ReserveTypeShape::QualifiedSymbol("BaseObjectModeMembership"),
                ),
                mode_definition(
                    "ChainObjectModeMembership",
                    ReserveTypeShape::QualifiedSymbol("InnerObjectModeMembership"),
                ),
            ],
        ];
        for modes in near_miss_modes {
            let near_miss = mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                modes,
                exact_reserves(),
                theorem,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }

        let near_miss_reserves = [
            vec![
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbol("ChainObjectModeMembership"),
                ),
            ],
            vec![reserve_item(
                vec!["x", "y"],
                ReserveTypeShape::QualifiedSymbol("ChainObjectModeMembership"),
            )],
            vec![
                reserve_item(vec!["x"], ReserveTypeShape::Builtin("object")),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
            ],
            vec![
                reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbol("ChainObjectModeMembership"),
                ),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("object")),
            ],
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("ChainObjectModeMembership"),
            )],
            vec![
                reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbol("ChainObjectModeMembership"),
                ),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                reserve_item(vec!["z"], ReserveTypeShape::Builtin("set")),
            ],
            vec![
                reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbolWithArgs("ChainObjectModeMembership"),
                ),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
            ],
        ];
        for reserves in near_miss_reserves {
            let near_miss = mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                reserves,
                theorem,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }

        for near_miss_theorem in [
            IdentifierBinaryTheoremSpec {
                left: "y",
                right: "x",
                ..theorem
            },
            IdentifierBinaryTheoremSpec {
                right: "x",
                ..theorem
            },
            IdentifierBinaryTheoremSpec {
                label: "OtherPayloadBoundary",
                ..theorem
            },
            IdentifierBinaryTheoremSpec {
                operator: "=",
                ..theorem
            },
            IdentifierBinaryTheoremSpec {
                status: Some("open"),
                ..theorem
            },
            IdentifierBinaryTheoremSpec {
                recovered_label: true,
                ..theorem
            },
        ] {
            let near_miss = mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                exact_reserves(),
                near_miss_theorem,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }

        let extra_item = modes_then_empty_definition_reserve_identifier_binary_theorem_ast(
            source_id,
            exact_modes(),
            exact_reserves(),
            theorem,
        );
        assert_eq!(
            source_type_elaboration_detail_keys(&extra_item, module.clone(), &symbols),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );
        let unresolved_symbols = source_local_symbols_env(
            module.clone(),
            &[("BaseObjectModeMembership", SymbolKind::Mode)],
        );
        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module, &unresolved_symbols),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );
    }

    #[test]
    fn source_local_mode_reserved_variable_equality_consumes_real_expansion() {
        let source_id = source_id(126);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("local_mode_reserved_variable_equality"),
        );
        let symbols =
            source_local_symbols_env(module.clone(), &[("LocalModeFormula", SymbolKind::Mode)]);
        let theorem = IdentifierBinaryTheoremSpec {
            status: None,
            label: "LocalModeReservedVariableEqualityPayloadBoundary",
            left: "x",
            operator: "=",
            right: "x",
            recovered_label: false,
        };
        let exact = mode_then_reserve_identifier_binary_theorem_ast(
            source_id,
            [mode_definition(
                "LocalModeFormula",
                ReserveTypeShape::Builtin("set"),
            )],
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("LocalModeFormula"),
            )],
            theorem,
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        let payload =
            extract_source_local_mode_reserved_variable_equality(&exact, module.clone(), &symbols)
                .expect("exact local-mode reserved-variable equality should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 1);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(
            payload.reserve.bridge.bindings()[0].type_spelling,
            "LocalModeFormula"
        );
        assert!(matches!(
            payload.reserve.bridge.bindings()[0].type_head,
            TypeHeadInput::Symbol(_)
        ));
        assert_eq!(payload.left_lookup_ordinal, 1);
        assert_eq!(payload.right_lookup_ordinal, 2);

        let output =
            source_local_mode_reserved_variable_equality_output(&exact, module.clone(), &symbols)
                .expect("exact local-mode equality should reach TermFormulaChecker");
        assert_source_reserved_variable_formula_output(&output)
            .expect("local-mode equality invariants should hold");
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
            assert_eq!(input.spelling, "LocalModeFormula");
            assert!(matches!(input.head, TypeHeadInput::Symbol(_)));
        }
        assert_eq!(output.term_formula.normalized_types().len(), 1);

        let mut invalid_output =
            source_local_mode_reserved_variable_equality_output(&exact, module.clone(), &symbols)
                .expect("exact source should produce a second checker output");
        invalid_output.payload.reserve.mode_expansions.clear();
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&invalid_output),
            vec![
                TYPE_ELABORATION_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY
                    .to_owned()
            ]
        );

        let gap_modes = [
            mode_definition("LocalModeFormula", ReserveTypeShape::Builtin("object")),
            mode_definition("LocalModeFormula", ReserveTypeShape::AttributedSet),
            contextual_mode_definition("LocalModeFormula", ReserveTypeShape::Builtin("set")),
            parameterized_mode_definition("LocalModeFormula", ReserveTypeShape::Builtin("set")),
            recovered_mode_definition("LocalModeFormula", ReserveTypeShape::Builtin("set")),
        ];
        for mode in gap_modes {
            let near_miss = mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode],
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbol("LocalModeFormula"),
                )],
                theorem,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
        for near_miss in [
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode_definition(
                    "OtherMode",
                    ReserveTypeShape::Builtin("set"),
                )],
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbol("LocalModeFormula"),
                )],
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [
                    mode_definition("LocalModeFormula", ReserveTypeShape::Builtin("set")),
                    mode_definition("LocalModeFormula", ReserveTypeShape::Builtin("set")),
                ],
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbol("LocalModeFormula"),
                )],
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode_definition_with_label(
                    "LocalModeFormula",
                    "OtherDef",
                    ReserveTypeShape::Builtin("set"),
                )],
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbol("LocalModeFormula"),
                )],
                theorem,
            ),
            modes_then_empty_definition_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode_definition(
                    "LocalModeFormula",
                    ReserveTypeShape::Builtin("set"),
                )],
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbol("LocalModeFormula"),
                )],
                theorem,
            ),
            reserve_then_mode_identifier_binary_theorem_ast(
                source_id,
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbol("LocalModeFormula"),
                )],
                mode_definition("LocalModeFormula", ReserveTypeShape::Builtin("set")),
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode_definition(
                    "LocalModeFormula",
                    ReserveTypeShape::Builtin("set"),
                )],
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbolWithArgs("LocalModeFormula"),
                )],
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [
                    mode_definition("BaseMode", ReserveTypeShape::Builtin("set")),
                    mode_definition(
                        "LocalModeFormula",
                        ReserveTypeShape::QualifiedSymbol("BaseMode"),
                    ),
                ],
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbol("LocalModeFormula"),
                )],
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode_definition(
                    "LocalModeFormula",
                    ReserveTypeShape::Builtin("set"),
                )],
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbol("LocalModeFormula"),
                )],
                IdentifierBinaryTheoremSpec {
                    label: "OtherPayloadBoundary",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode_definition(
                    "LocalModeFormula",
                    ReserveTypeShape::Builtin("set"),
                )],
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbol("LocalModeFormula"),
                )],
                IdentifierBinaryTheoremSpec {
                    operator: "<>",
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
    fn source_local_mode_reserved_variable_inequality_consumes_real_expansion() {
        let source_id = source_id(130);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("local_mode_reserved_variable_inequality"),
        );
        let symbols =
            source_local_symbols_env(module.clone(), &[("LocalModeInequality", SymbolKind::Mode)]);
        let theorem = IdentifierBinaryTheoremSpec {
            status: None,
            label: "LocalModeReservedVariableInequalityPayloadBoundary",
            left: "x",
            operator: "<>",
            right: "x",
            recovered_label: false,
        };
        let reserve = || {
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("LocalModeInequality"),
            )]
        };
        let exact = mode_then_reserve_identifier_binary_theorem_ast(
            source_id,
            [mode_definition(
                "LocalModeInequality",
                ReserveTypeShape::Builtin("set"),
            )],
            reserve(),
            theorem,
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        let payload = extract_source_local_mode_reserved_variable_inequality(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact local-mode reserved-variable inequality should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 1);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(
            payload.reserve.bridge.bindings()[0].type_spelling,
            "LocalModeInequality"
        );
        assert_eq!(payload.left_lookup_ordinal, 1);
        assert_eq!(payload.right_lookup_ordinal, 2);

        let output =
            source_local_mode_reserved_variable_inequality_output(&exact, module.clone(), &symbols)
                .expect("exact local-mode inequality should reach TermFormulaChecker");
        assert_source_reserved_variable_formula_output(&output)
            .expect("local-mode inequality invariants should hold");
        assert_eq!(output.left_binding, BindingId::new(0));
        assert_eq!(output.right_binding, BindingId::new(0));
        for input in [
            &output.left_result_input,
            &output.right_result_input,
            output.left_expected_input.as_ref().expect("left expected"),
            output
                .right_expected_input
                .as_ref()
                .expect("right expected"),
        ] {
            assert_eq!(input.spelling, "LocalModeInequality");
            assert!(matches!(input.head, TypeHeadInput::Symbol(_)));
        }
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
            .expect("one normalized set type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_eq!(normalized.source.range, terminal.radix.source_range);
        let formula = output
            .term_formula
            .formulas()
            .iter()
            .map(|(_, formula)| formula)
            .next()
            .expect("inequality formula should be checked");
        assert_eq!(formula.kind, FormulaKind::Inequality);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());

        let mut corrupted_output =
            source_local_mode_reserved_variable_inequality_output(&exact, module.clone(), &symbols)
                .expect("exact source should produce a corrupted checker output");
        corrupted_output
            .payload
            .reserve
            .mode_expansions
            .values_mut()
            .next()
            .expect("direct expansion should exist")
            .radix
            .head = TypeHeadInput::BuiltinObject;
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&corrupted_output),
            vec![
                TYPE_ELABORATION_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY
                    .to_owned()
            ]
        );

        let mut invalid_output =
            source_local_mode_reserved_variable_inequality_output(&exact, module.clone(), &symbols)
                .expect("exact source should produce a second checker output");
        invalid_output.payload.reserve.mode_expansions.clear();
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&invalid_output),
            vec![
                TYPE_ELABORATION_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY
                    .to_owned()
            ]
        );

        for mode in [
            mode_definition("LocalModeInequality", ReserveTypeShape::Builtin("object")),
            mode_definition("LocalModeInequality", ReserveTypeShape::AttributedSet),
            contextual_mode_definition("LocalModeInequality", ReserveTypeShape::Builtin("set")),
            parameterized_mode_definition("LocalModeInequality", ReserveTypeShape::Builtin("set")),
            recovered_mode_definition("LocalModeInequality", ReserveTypeShape::Builtin("set")),
        ] {
            let near_miss = mode_then_reserve_identifier_binary_theorem_ast(
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
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                Vec::<ModeDefinitionSpec>::new(),
                reserve(),
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode_definition(
                    "OtherMode",
                    ReserveTypeShape::Builtin("set"),
                )],
                reserve(),
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode_definition_with_label(
                    "LocalModeInequality",
                    "OtherDef",
                    ReserveTypeShape::Builtin("set"),
                )],
                reserve(),
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [
                    mode_definition("LocalModeInequality", ReserveTypeShape::Builtin("set")),
                    mode_definition("LocalModeInequality", ReserveTypeShape::Builtin("set")),
                ],
                reserve(),
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [
                    mode_definition("OtherMode", ReserveTypeShape::Builtin("set")),
                    mode_definition("LocalModeInequality", ReserveTypeShape::Builtin("set")),
                ],
                reserve(),
                theorem,
            ),
            reserve_then_mode_identifier_binary_theorem_ast(
                source_id,
                reserve(),
                mode_definition("LocalModeInequality", ReserveTypeShape::Builtin("set")),
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode_definition(
                    "LocalModeInequality",
                    ReserveTypeShape::Builtin("set"),
                )],
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode_definition(
                    "LocalModeInequality",
                    ReserveTypeShape::Builtin("set"),
                )],
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbolWithArgs("LocalModeInequality"),
                )],
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [
                    mode_definition("BaseMode", ReserveTypeShape::Builtin("set")),
                    mode_definition(
                        "LocalModeInequality",
                        ReserveTypeShape::QualifiedSymbol("BaseMode"),
                    ),
                ],
                reserve(),
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode_definition(
                    "LocalModeInequality",
                    ReserveTypeShape::Builtin("set"),
                )],
                reserve(),
                IdentifierBinaryTheoremSpec {
                    label: "OtherPayloadBoundary",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode_definition(
                    "LocalModeInequality",
                    ReserveTypeShape::Builtin("set"),
                )],
                reserve(),
                IdentifierBinaryTheoremSpec {
                    operator: "=",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode_definition(
                    "LocalModeInequality",
                    ReserveTypeShape::Builtin("set"),
                )],
                reserve(),
                IdentifierBinaryTheoremSpec {
                    left: "y",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode_definition(
                    "LocalModeInequality",
                    ReserveTypeShape::Builtin("set"),
                )],
                reserve(),
                IdentifierBinaryTheoremSpec {
                    recovered_label: true,
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode_definition(
                    "LocalModeInequality",
                    ReserveTypeShape::Builtin("set"),
                )],
                reserve(),
                IdentifierBinaryTheoremSpec {
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
    fn source_local_object_mode_reserved_variable_inequality_consumes_real_expansion() {
        let source_id = source_id(131);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("local_object_mode_reserved_variable_inequality"),
        );
        let symbols = source_local_symbols_env(
            module.clone(),
            &[("LocalObjectModeInequality", SymbolKind::Mode)],
        );
        let theorem = IdentifierBinaryTheoremSpec {
            status: None,
            label: "LocalObjectModeReservedVariableInequalityPayloadBoundary",
            left: "x",
            operator: "<>",
            right: "x",
            recovered_label: false,
        };
        let reserve = || {
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("LocalObjectModeInequality"),
            )]
        };
        let exact = mode_then_reserve_identifier_binary_theorem_ast(
            source_id,
            [mode_definition(
                "LocalObjectModeInequality",
                ReserveTypeShape::Builtin("object"),
            )],
            reserve(),
            theorem,
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        let payload = extract_source_local_object_mode_reserved_variable_inequality(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact local object-mode reserved-variable inequality should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 1);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(
            payload.reserve.bridge.bindings()[0].type_spelling,
            "LocalObjectModeInequality"
        );
        assert_eq!(payload.left_lookup_ordinal, 1);
        assert_eq!(payload.right_lookup_ordinal, 2);

        let output = source_local_object_mode_reserved_variable_inequality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact local object-mode inequality should reach TermFormulaChecker");
        assert_source_reserved_variable_formula_output(&output)
            .expect("local object-mode inequality invariants should hold");
        assert_eq!(output.left_binding, BindingId::new(0));
        assert_eq!(output.right_binding, BindingId::new(0));
        let reserve_range = output.payload.reserve.bridge.bindings()[0].type_range;
        for input in [
            &output.left_result_input,
            &output.right_result_input,
            output.left_expected_input.as_ref().expect("left expected"),
            output
                .right_expected_input
                .as_ref()
                .expect("right expected"),
        ] {
            assert_eq!(input.spelling, "LocalObjectModeInequality");
            assert_eq!(input.source_range, reserve_range);
            assert!(matches!(input.head, TypeHeadInput::Symbol(_)));
        }
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
            .expect("one normalized object type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_eq!(normalized.source.range, terminal.radix.source_range);
        assert_eq!(normalized.source.spelling, terminal.radix.spelling);
        let formula = output
            .term_formula
            .formulas()
            .iter()
            .map(|(_, formula)| formula)
            .next()
            .expect("inequality formula should be checked");
        assert_eq!(formula.kind, FormulaKind::Inequality);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());

        let mut corrupted_output = source_local_object_mode_reserved_variable_inequality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce a corrupted checker output");
        corrupted_output
            .payload
            .reserve
            .mode_expansions
            .values_mut()
            .next()
            .expect("direct object expansion should exist")
            .radix
            .head = TypeHeadInput::BuiltinSet;
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&corrupted_output),
            vec![
                TYPE_ELABORATION_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY
                    .to_owned()
            ]
        );

        let mut invalid_output = source_local_object_mode_reserved_variable_inequality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce a second checker output");
        invalid_output.payload.reserve.mode_expansions.clear();
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&invalid_output),
            vec![
                TYPE_ELABORATION_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY
                    .to_owned()
            ]
        );

        for mode in [
            mode_definition(
                "LocalObjectModeInequality",
                ReserveTypeShape::Builtin("set"),
            ),
            mode_definition(
                "LocalObjectModeInequality",
                ReserveTypeShape::AttributedObject,
            ),
            contextual_mode_definition(
                "LocalObjectModeInequality",
                ReserveTypeShape::Builtin("object"),
            ),
            parameterized_mode_definition(
                "LocalObjectModeInequality",
                ReserveTypeShape::Builtin("object"),
            ),
            recovered_mode_definition(
                "LocalObjectModeInequality",
                ReserveTypeShape::Builtin("object"),
            ),
        ] {
            let near_miss = mode_then_reserve_identifier_binary_theorem_ast(
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
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                Vec::<ModeDefinitionSpec>::new(),
                reserve(),
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode_definition(
                    "OtherMode",
                    ReserveTypeShape::Builtin("object"),
                )],
                reserve(),
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode_definition_with_label(
                    "LocalObjectModeInequality",
                    "OtherDef",
                    ReserveTypeShape::Builtin("object"),
                )],
                reserve(),
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [
                    mode_definition(
                        "LocalObjectModeInequality",
                        ReserveTypeShape::Builtin("object"),
                    ),
                    mode_definition(
                        "LocalObjectModeInequality",
                        ReserveTypeShape::Builtin("object"),
                    ),
                ],
                reserve(),
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [
                    mode_definition("OtherMode", ReserveTypeShape::Builtin("object")),
                    mode_definition(
                        "LocalObjectModeInequality",
                        ReserveTypeShape::Builtin("object"),
                    ),
                ],
                reserve(),
                theorem,
            ),
            reserve_then_mode_identifier_binary_theorem_ast(
                source_id,
                reserve(),
                mode_definition(
                    "LocalObjectModeInequality",
                    ReserveTypeShape::Builtin("object"),
                ),
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode_definition(
                    "LocalObjectModeInequality",
                    ReserveTypeShape::Builtin("object"),
                )],
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("object"))],
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode_definition(
                    "LocalObjectModeInequality",
                    ReserveTypeShape::Builtin("object"),
                )],
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbolWithArgs("LocalObjectModeInequality"),
                )],
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [
                    mode_definition("BaseMode", ReserveTypeShape::Builtin("object")),
                    mode_definition(
                        "LocalObjectModeInequality",
                        ReserveTypeShape::QualifiedSymbol("BaseMode"),
                    ),
                ],
                reserve(),
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode_definition(
                    "LocalObjectModeInequality",
                    ReserveTypeShape::Builtin("object"),
                )],
                reserve(),
                IdentifierBinaryTheoremSpec {
                    label: "OtherPayloadBoundary",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode_definition(
                    "LocalObjectModeInequality",
                    ReserveTypeShape::Builtin("object"),
                )],
                reserve(),
                IdentifierBinaryTheoremSpec {
                    operator: "=",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode_definition(
                    "LocalObjectModeInequality",
                    ReserveTypeShape::Builtin("object"),
                )],
                reserve(),
                IdentifierBinaryTheoremSpec {
                    left: "y",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode_definition(
                    "LocalObjectModeInequality",
                    ReserveTypeShape::Builtin("object"),
                )],
                reserve(),
                IdentifierBinaryTheoremSpec {
                    recovered_label: true,
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode_definition(
                    "LocalObjectModeInequality",
                    ReserveTypeShape::Builtin("object"),
                )],
                reserve(),
                IdentifierBinaryTheoremSpec {
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
    fn source_chained_local_mode_reserved_variable_equality_consumes_both_expansions() {
        let source_id = source_id(127);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("chained_local_mode_reserved_variable_equality"),
        );
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseModeFormula", SymbolKind::Mode),
                ("ChainModeFormula", SymbolKind::Mode),
                ("InnerModeFormula", SymbolKind::Mode),
            ],
        );
        let theorem = IdentifierBinaryTheoremSpec {
            status: None,
            label: "ChainedLocalModeReservedVariableEqualityPayloadBoundary",
            left: "x",
            operator: "=",
            right: "x",
            recovered_label: false,
        };
        let reserve = || {
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("ChainModeFormula"),
            )]
        };
        let exact = mode_then_reserve_identifier_binary_theorem_ast(
            source_id,
            [
                mode_definition("BaseModeFormula", ReserveTypeShape::Builtin("set")),
                mode_definition(
                    "ChainModeFormula",
                    ReserveTypeShape::QualifiedSymbol("BaseModeFormula"),
                ),
            ],
            reserve(),
            theorem,
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        let payload = extract_source_chained_local_mode_reserved_variable_equality(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact chained local-mode equality should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 2);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(
            payload.reserve.bridge.bindings()[0].type_spelling,
            "ChainModeFormula"
        );
        assert_eq!(payload.left_lookup_ordinal, 1);
        assert_eq!(payload.right_lookup_ordinal, 2);

        let output = source_chained_local_mode_reserved_variable_equality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact chained local-mode equality should reach TermFormulaChecker");
        assert_source_reserved_variable_formula_output(&output)
            .expect("chained local-mode equality invariants should hold");
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
            assert_eq!(input.spelling, "ChainModeFormula");
            assert!(matches!(input.head, TypeHeadInput::Symbol(_)));
        }
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("BaseModeFormula"))
            .map(|(_, expansion)| &expansion.radix)
            .expect("base mode terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("one normalized terminal type should exist");
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);

        let mut invalid_output = source_chained_local_mode_reserved_variable_equality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce a second checker output");
        invalid_output
            .payload
            .reserve
            .mode_expansions
            .retain(|symbol, _| source_mode_symbol_spelling(symbol) != Some("BaseModeFormula"));
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&invalid_output),
            vec![
                TYPE_ELABORATION_CHAINED_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY
                    .to_owned()
            ]
        );

        let near_miss_modes = [
            vec![mode_definition(
                "ChainModeFormula",
                ReserveTypeShape::QualifiedSymbol("BaseModeFormula"),
            )],
            vec![
                mode_definition("BaseModeFormula", ReserveTypeShape::Builtin("set")),
                mode_definition("BaseModeFormula", ReserveTypeShape::Builtin("set")),
                mode_definition(
                    "ChainModeFormula",
                    ReserveTypeShape::QualifiedSymbol("BaseModeFormula"),
                ),
            ],
            vec![
                mode_definition("BaseModeFormula", ReserveTypeShape::Builtin("object")),
                mode_definition(
                    "ChainModeFormula",
                    ReserveTypeShape::QualifiedSymbol("BaseModeFormula"),
                ),
            ],
            vec![
                mode_definition("BaseModeFormula", ReserveTypeShape::AttributedSet),
                mode_definition(
                    "ChainModeFormula",
                    ReserveTypeShape::QualifiedSymbol("BaseModeFormula"),
                ),
            ],
            vec![
                contextual_mode_definition("BaseModeFormula", ReserveTypeShape::Builtin("set")),
                mode_definition(
                    "ChainModeFormula",
                    ReserveTypeShape::QualifiedSymbol("BaseModeFormula"),
                ),
            ],
            vec![
                mode_definition("BaseModeFormula", ReserveTypeShape::Builtin("set")),
                parameterized_mode_definition(
                    "ChainModeFormula",
                    ReserveTypeShape::QualifiedSymbol("BaseModeFormula"),
                ),
            ],
            vec![
                recovered_mode_definition("BaseModeFormula", ReserveTypeShape::Builtin("set")),
                mode_definition(
                    "ChainModeFormula",
                    ReserveTypeShape::QualifiedSymbol("BaseModeFormula"),
                ),
            ],
            vec![
                mode_definition_with_label(
                    "BaseModeFormula",
                    "OtherDef",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition(
                    "ChainModeFormula",
                    ReserveTypeShape::QualifiedSymbol("BaseModeFormula"),
                ),
            ],
            vec![
                mode_definition("BaseModeFormula", ReserveTypeShape::Builtin("set")),
                mode_definition_with_label(
                    "ChainModeFormula",
                    "OtherDef",
                    ReserveTypeShape::QualifiedSymbol("BaseModeFormula"),
                ),
            ],
            vec![
                mode_definition(
                    "ChainModeFormula",
                    ReserveTypeShape::QualifiedSymbol("BaseModeFormula"),
                ),
                mode_definition("BaseModeFormula", ReserveTypeShape::Builtin("set")),
            ],
            vec![
                mode_definition("BaseModeFormula", ReserveTypeShape::Builtin("set")),
                mode_definition(
                    "ChainModeFormula",
                    ReserveTypeShape::QualifiedSymbolWithArgs("BaseModeFormula"),
                ),
            ],
            vec![
                mode_definition("BaseModeFormula", ReserveTypeShape::Builtin("set")),
                mode_definition("ChainModeFormula", ReserveTypeShape::Builtin("set")),
            ],
            vec![
                mode_definition(
                    "BaseModeFormula",
                    ReserveTypeShape::QualifiedSymbol("ChainModeFormula"),
                ),
                mode_definition(
                    "ChainModeFormula",
                    ReserveTypeShape::QualifiedSymbol("BaseModeFormula"),
                ),
            ],
            vec![
                mode_definition("BaseModeFormula", ReserveTypeShape::Builtin("set")),
                mode_definition(
                    "InnerModeFormula",
                    ReserveTypeShape::QualifiedSymbol("BaseModeFormula"),
                ),
                mode_definition(
                    "ChainModeFormula",
                    ReserveTypeShape::QualifiedSymbol("InnerModeFormula"),
                ),
            ],
        ];
        for modes in near_miss_modes {
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
        }
        for near_miss in [
            modes_then_empty_definition_reserve_identifier_binary_theorem_ast(
                source_id,
                [
                    mode_definition("BaseModeFormula", ReserveTypeShape::Builtin("set")),
                    mode_definition(
                        "ChainModeFormula",
                        ReserveTypeShape::QualifiedSymbol("BaseModeFormula"),
                    ),
                ],
                reserve(),
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [
                    mode_definition("BaseModeFormula", ReserveTypeShape::Builtin("set")),
                    mode_definition(
                        "ChainModeFormula",
                        ReserveTypeShape::QualifiedSymbol("BaseModeFormula"),
                    ),
                ],
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbolWithArgs("ChainModeFormula"),
                )],
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [
                    mode_definition("BaseModeFormula", ReserveTypeShape::Builtin("set")),
                    mode_definition(
                        "ChainModeFormula",
                        ReserveTypeShape::QualifiedSymbol("BaseModeFormula"),
                    ),
                ],
                reserve(),
                IdentifierBinaryTheoremSpec {
                    label: "OtherPayloadBoundary",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [
                    mode_definition("BaseModeFormula", ReserveTypeShape::Builtin("set")),
                    mode_definition(
                        "ChainModeFormula",
                        ReserveTypeShape::QualifiedSymbol("BaseModeFormula"),
                    ),
                ],
                reserve(),
                IdentifierBinaryTheoremSpec {
                    operator: "<>",
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
    fn source_two_edge_local_mode_reserved_variable_membership_consumes_three_expansions() {
        let source_id = source_id(143);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("two_edge_local_mode_reserved_variable_membership"),
        );
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseTwoEdgeModeMembership", SymbolKind::Mode),
                ("MiddleTwoEdgeModeMembership", SymbolKind::Mode),
                ("OuterTwoEdgeModeMembership", SymbolKind::Mode),
                ("ExtraTwoEdgeModeMembership", SymbolKind::Mode),
            ],
        );
        let theorem = IdentifierBinaryTheoremSpec {
            status: None,
            label: "TwoEdgeLocalModeReservedVariableMembershipPayloadBoundary",
            left: "x",
            operator: "in",
            right: "y",
            recovered_label: false,
        };
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    "BaseTwoEdgeModeMembership",
                    "BaseTwoEdgeModeMembershipDef",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition_with_label(
                    "MiddleTwoEdgeModeMembership",
                    "MiddleTwoEdgeModeMembershipDef",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeModeMembership"),
                ),
                mode_definition_with_label(
                    "OuterTwoEdgeModeMembership",
                    "OuterTwoEdgeModeMembershipDef",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeModeMembership"),
                ),
            ]
        };
        let exact_reserves = || {
            vec![
                reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbol("OuterTwoEdgeModeMembership"),
                ),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
            ]
        };
        let exact = mode_then_reserve_identifier_binary_theorem_ast(
            source_id,
            exact_modes(),
            exact_reserves(),
            theorem,
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        let payload = extract_source_two_edge_local_mode_reserved_variable_membership(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact two-edge local-mode membership should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 3);
        assert_eq!(payload.reserve.bridge.bindings().len(), 2);
        assert_eq!(
            payload.reserve.bridge.bindings()[0].type_spelling,
            "OuterTwoEdgeModeMembership"
        );
        assert_eq!(payload.reserve.bridge.bindings()[1].type_spelling, "set");
        assert_ne!(
            payload.reserve.bridge.bindings()[0].type_range,
            payload.reserve.bridge.bindings()[1].type_range
        );
        assert_eq!(payload.left_lookup_ordinal, 2);
        assert_eq!(payload.right_lookup_ordinal, 3);

        let output = source_two_edge_local_mode_reserved_variable_membership_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact two-edge local-mode membership should reach TermFormulaChecker");
        assert_source_reserved_variable_formula_output(&output)
            .expect("two-edge local-mode membership invariants should hold");
        assert_eq!(output.left_binding, BindingId::new(0));
        assert_eq!(output.right_binding, BindingId::new(1));
        assert_eq!(
            output.left_result_input.spelling,
            "OuterTwoEdgeModeMembership"
        );
        assert!(matches!(
            output.left_result_input.head,
            TypeHeadInput::Symbol(_)
        ));
        assert_eq!(output.right_result_input.head, TypeHeadInput::BuiltinSet);
        assert!(output.left_expected_input.is_none());
        let right_expected = output
            .right_expected_input
            .as_ref()
            .expect("right expected input should exist");
        assert_eq!(right_expected.head, TypeHeadInput::BuiltinSet);
        assert_eq!(
            output.right_result_input.source_range,
            right_expected.source_range
        );
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("BaseTwoEdgeModeMembership")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("base mode terminal expansion should exist");
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
        let formula = output
            .term_formula
            .formulas()
            .iter()
            .map(|(_, formula)| formula)
            .next()
            .expect("membership formula should be checked");
        assert_eq!(formula.kind, FormulaKind::Membership);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert_eq!(formula.expected_types.len(), 1);
        assert_eq!(formula.expected_types[0].term, output.payload.right_site);
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
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
                "two-edge-local-mode-reserved-variable-membership-left-result".to_owned(),
                "two-edge-local-mode-reserved-variable-membership-right-expected".to_owned(),
                "two-edge-local-mode-reserved-variable-membership-right-result".to_owned(),
            ])
        );

        for removed in [
            "BaseTwoEdgeModeMembership",
            "MiddleTwoEdgeModeMembership",
            "OuterTwoEdgeModeMembership",
        ] {
            let mut invalid = source_two_edge_local_mode_reserved_variable_membership_output(
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
            assert_eq!(
                source_reserved_variable_formula_output_detail_keys(&invalid),
                vec![
                    TYPE_ELABORATION_TWO_EDGE_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY
                        .to_owned()
                ]
            );
        }

        let mut invalid_expected = source_two_edge_local_mode_reserved_variable_membership_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce a right-expected corruption target");
        invalid_expected
            .right_expected_input
            .as_mut()
            .expect("right expected input should exist")
            .head = TypeHeadInput::BuiltinObject;
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&invalid_expected),
            vec![
                TYPE_ELABORATION_TWO_EDGE_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY
                    .to_owned()
            ]
        );

        let near_miss_modes = [
            vec![exact_modes()[1], exact_modes()[2]],
            vec![exact_modes()[0], exact_modes()[2]],
            vec![exact_modes()[0], exact_modes()[1]],
            {
                let mut modes = exact_modes();
                modes.insert(1, modes[0]);
                modes
            },
            vec![
                recovered_mode_definition(
                    "BaseTwoEdgeModeMembership",
                    ReserveTypeShape::Builtin("set"),
                ),
                exact_modes()[1],
                exact_modes()[2],
            ],
            vec![
                contextual_mode_definition(
                    "BaseTwoEdgeModeMembership",
                    ReserveTypeShape::Builtin("set"),
                ),
                exact_modes()[1],
                exact_modes()[2],
            ],
            vec![
                mode_definition_with_label(
                    "BaseTwoEdgeModeMembership",
                    "OtherDef",
                    ReserveTypeShape::Builtin("set"),
                ),
                exact_modes()[1],
                exact_modes()[2],
            ],
            vec![
                exact_modes()[0],
                mode_definition_with_label(
                    "MiddleTwoEdgeModeMembership",
                    "OtherDef",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeModeMembership"),
                ),
                exact_modes()[2],
            ],
            vec![
                exact_modes()[0],
                exact_modes()[1],
                mode_definition_with_label(
                    "OuterTwoEdgeModeMembership",
                    "OtherDef",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeModeMembership"),
                ),
            ],
            vec![
                exact_modes()[0],
                mode_definition(
                    "MiddleTwoEdgeModeMembership",
                    ReserveTypeShape::QualifiedSymbolWithArgs("BaseTwoEdgeModeMembership"),
                ),
                exact_modes()[2],
            ],
            vec![
                exact_modes()[0],
                exact_modes()[1],
                mode_definition(
                    "OuterTwoEdgeModeMembership",
                    ReserveTypeShape::QualifiedSymbolWithArgs("MiddleTwoEdgeModeMembership"),
                ),
            ],
            vec![
                mode_definition("BaseTwoEdgeModeMembership", ReserveTypeShape::AttributedSet),
                exact_modes()[1],
                exact_modes()[2],
            ],
            vec![
                mode_definition(
                    "BaseTwoEdgeModeMembership",
                    ReserveTypeShape::Builtin("object"),
                ),
                exact_modes()[1],
                exact_modes()[2],
            ],
            vec![exact_modes()[2], exact_modes()[1], exact_modes()[0]],
            vec![
                exact_modes()[0],
                mode_definition(
                    "MiddleTwoEdgeModeMembership",
                    ReserveTypeShape::Builtin("set"),
                ),
                exact_modes()[2],
            ],
            vec![
                exact_modes()[0],
                exact_modes()[1],
                mode_definition(
                    "OuterTwoEdgeModeMembership",
                    ReserveTypeShape::Builtin("set"),
                ),
            ],
            vec![
                exact_modes()[0],
                mode_definition(
                    "MiddleTwoEdgeModeMembership",
                    ReserveTypeShape::QualifiedSymbol("OuterTwoEdgeModeMembership"),
                ),
                mode_definition(
                    "OuterTwoEdgeModeMembership",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeModeMembership"),
                ),
            ],
            vec![
                exact_modes()[0],
                mode_definition(
                    "OuterTwoEdgeModeMembership",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeModeMembership"),
                ),
            ],
            vec![
                exact_modes()[0],
                exact_modes()[1],
                mode_definition(
                    "ExtraTwoEdgeModeMembership",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeModeMembership"),
                ),
                mode_definition(
                    "OuterTwoEdgeModeMembership",
                    ReserveTypeShape::QualifiedSymbol("ExtraTwoEdgeModeMembership"),
                ),
            ],
            vec![
                exact_modes()[0],
                exact_modes()[1],
                parameterized_mode_definition(
                    "OuterTwoEdgeModeMembership",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeModeMembership"),
                ),
            ],
        ];
        for modes in near_miss_modes {
            let near_miss = mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                modes,
                exact_reserves(),
                theorem,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }

        let near_miss_reserves = [
            vec![
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbol("OuterTwoEdgeModeMembership"),
                ),
            ],
            vec![reserve_item(
                vec!["x", "y"],
                ReserveTypeShape::QualifiedSymbol("OuterTwoEdgeModeMembership"),
            )],
            vec![
                reserve_item(vec!["x"], ReserveTypeShape::Builtin("set")),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
            ],
            vec![
                reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbol("OuterTwoEdgeModeMembership"),
                ),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("object")),
            ],
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("OuterTwoEdgeModeMembership"),
            )],
            {
                let mut reserves = exact_reserves();
                reserves.push(reserve_item(vec!["z"], ReserveTypeShape::Builtin("set")));
                reserves
            },
            vec![
                reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbolWithArgs("OuterTwoEdgeModeMembership"),
                ),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
            ],
        ];
        for reserves in near_miss_reserves {
            let near_miss = mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                reserves,
                theorem,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }

        for near_miss in [
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                exact_reserves(),
                IdentifierBinaryTheoremSpec {
                    left: "y",
                    right: "x",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                exact_reserves(),
                IdentifierBinaryTheoremSpec {
                    right: "x",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                exact_reserves(),
                IdentifierBinaryTheoremSpec {
                    label: "OtherPayloadBoundary",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                exact_reserves(),
                IdentifierBinaryTheoremSpec {
                    operator: "=",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                exact_reserves(),
                IdentifierBinaryTheoremSpec {
                    status: Some("registration"),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                exact_reserves(),
                IdentifierBinaryTheoremSpec {
                    recovered_label: true,
                    ..theorem
                },
            ),
            modes_then_empty_definition_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                exact_reserves(),
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
    fn source_two_edge_local_object_mode_reserved_variable_membership_consumes_three_expansions() {
        let source_id = source_id(144);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("two_edge_local_object_mode_reserved_variable_membership"),
        );
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseTwoEdgeObjectModeMembership", SymbolKind::Mode),
                ("MiddleTwoEdgeObjectModeMembership", SymbolKind::Mode),
                ("OuterTwoEdgeObjectModeMembership", SymbolKind::Mode),
                ("ExtraTwoEdgeObjectModeMembership", SymbolKind::Mode),
            ],
        );
        let theorem = IdentifierBinaryTheoremSpec {
            status: None,
            label: "TwoEdgeLocalObjectModeReservedVariableMembershipPayloadBoundary",
            left: "x",
            operator: "in",
            right: "y",
            recovered_label: false,
        };
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    "BaseTwoEdgeObjectModeMembership",
                    "BaseTwoEdgeObjectModeMembershipDef",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition_with_label(
                    "MiddleTwoEdgeObjectModeMembership",
                    "MiddleTwoEdgeObjectModeMembershipDef",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeObjectModeMembership"),
                ),
                mode_definition_with_label(
                    "OuterTwoEdgeObjectModeMembership",
                    "OuterTwoEdgeObjectModeMembershipDef",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeObjectModeMembership"),
                ),
            ]
        };
        let exact_reserves = || {
            vec![
                reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbol("OuterTwoEdgeObjectModeMembership"),
                ),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
            ]
        };
        let exact = mode_then_reserve_identifier_binary_theorem_ast(
            source_id,
            exact_modes(),
            exact_reserves(),
            theorem,
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        let payload = extract_source_two_edge_local_object_mode_reserved_variable_membership(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact two-edge local-object-mode membership should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 3);
        assert_eq!(payload.reserve.bridge.bindings().len(), 2);
        assert_eq!(
            payload.reserve.bridge.bindings()[0].type_spelling,
            "OuterTwoEdgeObjectModeMembership"
        );
        assert_eq!(payload.reserve.bridge.bindings()[1].type_spelling, "set");
        assert_ne!(
            payload.reserve.bridge.bindings()[0].type_range,
            payload.reserve.bridge.bindings()[1].type_range
        );
        assert_eq!(payload.left_lookup_ordinal, 2);
        assert_eq!(payload.right_lookup_ordinal, 3);

        let output = source_two_edge_local_object_mode_reserved_variable_membership_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact two-edge local-object-mode membership should reach TermFormulaChecker");
        assert_source_reserved_variable_formula_output(&output)
            .expect("two-edge local-object-mode membership invariants should hold");
        assert_eq!(output.left_binding, BindingId::new(0));
        assert_eq!(output.right_binding, BindingId::new(1));
        assert_eq!(
            output.left_result_input.spelling,
            "OuterTwoEdgeObjectModeMembership"
        );
        assert!(matches!(
            output.left_result_input.head,
            TypeHeadInput::Symbol(_)
        ));
        assert_eq!(output.right_result_input.spelling, "set");
        assert_eq!(output.right_result_input.head, TypeHeadInput::BuiltinSet);
        assert!(output.left_expected_input.is_none());
        let right_expected = output
            .right_expected_input
            .as_ref()
            .expect("right expected input should exist");
        assert_eq!(right_expected.head, TypeHeadInput::BuiltinSet);
        assert_eq!(
            output.right_result_input.source_range,
            right_expected.source_range
        );
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("BaseTwoEdgeObjectModeMembership")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("base mode terminal expansion should exist");
        assert_eq!(output.term_formula.normalized_types().len(), 2);
        let object_normalized = output
            .term_formula
            .normalized_types()
            .iter()
            .map(|(_, normalized)| normalized)
            .find(|normalized| normalized.head == TypeHeadRef::BuiltinObject)
            .expect("one normalized object identity should exist");
        assert_eq!(object_normalized.source.range, terminal.source_range);
        assert_eq!(object_normalized.source.spelling, terminal.spelling);
        let set_normalized = output
            .term_formula
            .normalized_types()
            .iter()
            .map(|(_, normalized)| normalized)
            .find(|normalized| normalized.head == TypeHeadRef::BuiltinSet)
            .expect("one normalized set identity should exist");
        assert_eq!(
            set_normalized.source.range,
            output.payload.reserve.bridge.bindings()[1].type_range
        );
        let formula = output
            .term_formula
            .formulas()
            .iter()
            .map(|(_, formula)| formula)
            .next()
            .expect("membership formula should be checked");
        assert_eq!(formula.kind, FormulaKind::Membership);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert_eq!(formula.expected_types.len(), 1);
        assert_eq!(formula.expected_types[0].term, output.payload.right_site);
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
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
                "two-edge-local-object-mode-reserved-variable-membership-left-result".to_owned(),
                "two-edge-local-object-mode-reserved-variable-membership-right-expected".to_owned(),
                "two-edge-local-object-mode-reserved-variable-membership-right-result".to_owned(),
            ])
        );

        for removed in [
            "BaseTwoEdgeObjectModeMembership",
            "MiddleTwoEdgeObjectModeMembership",
            "OuterTwoEdgeObjectModeMembership",
        ] {
            let mut invalid =
                source_two_edge_local_object_mode_reserved_variable_membership_output(
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
            assert_eq!(
                source_reserved_variable_formula_output_detail_keys(&invalid),
                vec![
                    TYPE_ELABORATION_TWO_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY
                        .to_owned()
                ]
            );
        }

        let mut invalid_expected =
            source_two_edge_local_object_mode_reserved_variable_membership_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce a right-expected corruption target");
        invalid_expected
            .right_expected_input
            .as_mut()
            .expect("right expected input should exist")
            .head = TypeHeadInput::BuiltinObject;
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&invalid_expected),
            vec![
                TYPE_ELABORATION_TWO_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY
                    .to_owned()
            ]
        );

        let near_miss_modes = [
            vec![exact_modes()[1], exact_modes()[2]],
            vec![exact_modes()[0], exact_modes()[2]],
            vec![exact_modes()[0], exact_modes()[1]],
            {
                let mut modes = exact_modes();
                modes.insert(1, modes[0]);
                modes
            },
            vec![
                recovered_mode_definition(
                    "BaseTwoEdgeObjectModeMembership",
                    ReserveTypeShape::Builtin("object"),
                ),
                exact_modes()[1],
                exact_modes()[2],
            ],
            vec![
                contextual_mode_definition(
                    "BaseTwoEdgeObjectModeMembership",
                    ReserveTypeShape::Builtin("object"),
                ),
                exact_modes()[1],
                exact_modes()[2],
            ],
            vec![
                mode_definition_with_label(
                    "BaseTwoEdgeObjectModeMembership",
                    "OtherDef",
                    ReserveTypeShape::Builtin("object"),
                ),
                exact_modes()[1],
                exact_modes()[2],
            ],
            vec![
                exact_modes()[0],
                mode_definition_with_label(
                    "MiddleTwoEdgeObjectModeMembership",
                    "OtherDef",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeObjectModeMembership"),
                ),
                exact_modes()[2],
            ],
            vec![
                exact_modes()[0],
                exact_modes()[1],
                mode_definition_with_label(
                    "OuterTwoEdgeObjectModeMembership",
                    "OtherDef",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeObjectModeMembership"),
                ),
            ],
            vec![
                exact_modes()[0],
                mode_definition(
                    "MiddleTwoEdgeObjectModeMembership",
                    ReserveTypeShape::QualifiedSymbolWithArgs("BaseTwoEdgeObjectModeMembership"),
                ),
                exact_modes()[2],
            ],
            vec![
                exact_modes()[0],
                exact_modes()[1],
                mode_definition(
                    "OuterTwoEdgeObjectModeMembership",
                    ReserveTypeShape::QualifiedSymbolWithArgs("MiddleTwoEdgeObjectModeMembership"),
                ),
            ],
            vec![
                mode_definition(
                    "BaseTwoEdgeObjectModeMembership",
                    ReserveTypeShape::AttributedSet,
                ),
                exact_modes()[1],
                exact_modes()[2],
            ],
            vec![
                mode_definition(
                    "BaseTwoEdgeObjectModeMembership",
                    ReserveTypeShape::Builtin("set"),
                ),
                exact_modes()[1],
                exact_modes()[2],
            ],
            vec![exact_modes()[2], exact_modes()[1], exact_modes()[0]],
            vec![
                exact_modes()[0],
                mode_definition(
                    "MiddleTwoEdgeObjectModeMembership",
                    ReserveTypeShape::Builtin("object"),
                ),
                exact_modes()[2],
            ],
            vec![
                exact_modes()[0],
                exact_modes()[1],
                mode_definition(
                    "OuterTwoEdgeObjectModeMembership",
                    ReserveTypeShape::Builtin("object"),
                ),
            ],
            vec![
                exact_modes()[0],
                mode_definition(
                    "MiddleTwoEdgeObjectModeMembership",
                    ReserveTypeShape::QualifiedSymbol("OuterTwoEdgeObjectModeMembership"),
                ),
                mode_definition(
                    "OuterTwoEdgeObjectModeMembership",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeObjectModeMembership"),
                ),
            ],
            vec![
                exact_modes()[0],
                mode_definition(
                    "OuterTwoEdgeObjectModeMembership",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeObjectModeMembership"),
                ),
            ],
            vec![
                exact_modes()[0],
                exact_modes()[1],
                mode_definition(
                    "ExtraTwoEdgeObjectModeMembership",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeObjectModeMembership"),
                ),
                mode_definition(
                    "OuterTwoEdgeObjectModeMembership",
                    ReserveTypeShape::QualifiedSymbol("ExtraTwoEdgeObjectModeMembership"),
                ),
            ],
            vec![
                exact_modes()[0],
                exact_modes()[1],
                parameterized_mode_definition(
                    "OuterTwoEdgeObjectModeMembership",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeObjectModeMembership"),
                ),
            ],
        ];
        for modes in near_miss_modes {
            let near_miss = mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                modes,
                exact_reserves(),
                theorem,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }

        let near_miss_reserves = [
            vec![
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbol("OuterTwoEdgeObjectModeMembership"),
                ),
            ],
            vec![reserve_item(
                vec!["x", "y"],
                ReserveTypeShape::QualifiedSymbol("OuterTwoEdgeObjectModeMembership"),
            )],
            vec![
                reserve_item(vec!["x"], ReserveTypeShape::Builtin("object")),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
            ],
            vec![
                reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbol("OuterTwoEdgeObjectModeMembership"),
                ),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("object")),
            ],
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("OuterTwoEdgeObjectModeMembership"),
            )],
            {
                let mut reserves = exact_reserves();
                reserves.push(reserve_item(vec!["z"], ReserveTypeShape::Builtin("set")));
                reserves
            },
            vec![
                reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbolWithArgs("OuterTwoEdgeObjectModeMembership"),
                ),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
            ],
        ];
        for reserves in near_miss_reserves {
            let near_miss = mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                reserves,
                theorem,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }

        for near_miss in [
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                exact_reserves(),
                IdentifierBinaryTheoremSpec {
                    left: "y",
                    right: "x",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                exact_reserves(),
                IdentifierBinaryTheoremSpec {
                    right: "x",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                exact_reserves(),
                IdentifierBinaryTheoremSpec {
                    label: "OtherPayloadBoundary",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                exact_reserves(),
                IdentifierBinaryTheoremSpec {
                    operator: "=",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                exact_reserves(),
                IdentifierBinaryTheoremSpec {
                    status: Some("registration"),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                exact_reserves(),
                IdentifierBinaryTheoremSpec {
                    recovered_label: true,
                    ..theorem
                },
            ),
            modes_then_empty_definition_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                exact_reserves(),
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
    fn source_three_edge_local_mode_reserved_variable_equality_consumes_four_expansions() {
        let source_id = source_id(154);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("three_edge_local_mode_reserved_variable_equality"),
        );
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseThreeEdgeModeEquality", SymbolKind::Mode),
                ("InnerThreeEdgeModeEquality", SymbolKind::Mode),
                ("MiddleThreeEdgeModeEquality", SymbolKind::Mode),
                ("OuterThreeEdgeModeEquality", SymbolKind::Mode),
                ("ExtraThreeEdgeModeEquality", SymbolKind::Mode),
            ],
        );
        let theorem = IdentifierBinaryTheoremSpec {
            status: None,
            label: "ThreeEdgeLocalModeReservedVariableEqualityPayloadBoundary",
            left: "z",
            operator: "=",
            right: "z",
            recovered_label: false,
        };
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    "BaseThreeEdgeModeEquality",
                    "BaseThreeEdgeModeEqualityDef",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition_with_label(
                    "InnerThreeEdgeModeEquality",
                    "InnerThreeEdgeModeEqualityDef",
                    ReserveTypeShape::QualifiedSymbol("BaseThreeEdgeModeEquality"),
                ),
                mode_definition_with_label(
                    "MiddleThreeEdgeModeEquality",
                    "MiddleThreeEdgeModeEqualityDef",
                    ReserveTypeShape::QualifiedSymbol("InnerThreeEdgeModeEquality"),
                ),
                mode_definition_with_label(
                    "OuterThreeEdgeModeEquality",
                    "OuterThreeEdgeModeEqualityDef",
                    ReserveTypeShape::QualifiedSymbol("MiddleThreeEdgeModeEquality"),
                ),
            ]
        };
        let reserve = || {
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("OuterThreeEdgeModeEquality"),
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
        let payload = extract_source_three_edge_local_mode_reserved_variable_equality(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact three-edge local-mode reserved-variable equality should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 4);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(payload.reserve.bridge.bindings()[0].spelling, "z");
        assert_eq!(
            payload.reserve.bridge.bindings()[0].type_spelling,
            "OuterThreeEdgeModeEquality"
        );
        assert_eq!(payload.left_lookup_ordinal, 1);
        assert_eq!(payload.right_lookup_ordinal, 2);

        let output = source_three_edge_local_mode_reserved_variable_equality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact three-edge local-mode equality should reach TermFormulaChecker");
        assert_source_reserved_variable_formula_output(&output)
            .expect("three-edge local-mode equality invariants should hold");
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
            assert_eq!(input.spelling, "OuterThreeEdgeModeEquality");
            assert!(matches!(input.head, TypeHeadInput::Symbol(_)));
        }
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("BaseThreeEdgeModeEquality")
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
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
        assert!(output.term_formula.facts().is_empty());
        assert!(output.term_formula.diagnostics().is_empty());

        for removed in [
            "BaseThreeEdgeModeEquality",
            "InnerThreeEdgeModeEquality",
            "MiddleThreeEdgeModeEquality",
            "OuterThreeEdgeModeEquality",
        ] {
            let mut invalid = source_three_edge_local_mode_reserved_variable_equality_output(
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
            assert_eq!(
                source_reserved_variable_formula_output_detail_keys(&invalid),
                vec![
                    TYPE_ELABORATION_THREE_EDGE_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY
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

        for (index, radix) in [
            "BaseThreeEdgeModeEquality",
            "InnerThreeEdgeModeEquality",
            "MiddleThreeEdgeModeEquality",
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
                ReserveTypeShape::QualifiedSymbol("ExtraThreeEdgeModeEquality");
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
        direct_outermost_radix[3].rhs_shape = ReserveTypeShape::Builtin("set");
        assert_extraction_gap(direct_outermost_radix);

        let mut one_edge_outermost_radix = exact_modes();
        one_edge_outermost_radix[3].rhs_shape =
            ReserveTypeShape::QualifiedSymbol("BaseThreeEdgeModeEquality");
        assert_extraction_gap(one_edge_outermost_radix);

        let mut two_edge_outermost_radix = exact_modes();
        two_edge_outermost_radix[3].rhs_shape =
            ReserveTypeShape::QualifiedSymbol("InnerThreeEdgeModeEquality");
        assert_extraction_gap(two_edge_outermost_radix);

        let mut cyclic = exact_modes();
        cyclic[0].rhs_shape = ReserveTypeShape::QualifiedSymbol("OuterThreeEdgeModeEquality");
        assert_extraction_gap(cyclic);

        assert_extraction_gap(vec![
            exact_modes()[0],
            mode_definition_with_label(
                "ExtraThreeEdgeModeEquality",
                "ExtraThreeEdgeModeEqualityDef",
                ReserveTypeShape::QualifiedSymbol("BaseThreeEdgeModeEquality"),
            ),
            mode_definition_with_label(
                "InnerThreeEdgeModeEquality",
                "InnerThreeEdgeModeEqualityDef",
                ReserveTypeShape::QualifiedSymbol("ExtraThreeEdgeModeEquality"),
            ),
            exact_modes()[2],
            exact_modes()[3],
        ]);

        for near_miss in [
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(
                    vec!["z"],
                    ReserveTypeShape::QualifiedSymbolWithArgs("OuterThreeEdgeModeEquality"),
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
    fn source_three_edge_local_mode_reserved_variable_inequality_consumes_four_expansions() {
        let source_id = source_id(156);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("three_edge_local_mode_reserved_variable_inequality"),
        );
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseThreeEdgeModeInequality", SymbolKind::Mode),
                ("InnerThreeEdgeModeInequality", SymbolKind::Mode),
                ("MiddleThreeEdgeModeInequality", SymbolKind::Mode),
                ("OuterThreeEdgeModeInequality", SymbolKind::Mode),
                ("ExtraThreeEdgeModeInequality", SymbolKind::Mode),
            ],
        );
        let theorem = IdentifierBinaryTheoremSpec {
            status: None,
            label: "ThreeEdgeLocalModeReservedVariableInequalityPayloadBoundary",
            left: "z",
            operator: "<>",
            right: "z",
            recovered_label: false,
        };
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    "BaseThreeEdgeModeInequality",
                    "BaseThreeEdgeModeInequalityDef",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition_with_label(
                    "InnerThreeEdgeModeInequality",
                    "InnerThreeEdgeModeInequalityDef",
                    ReserveTypeShape::QualifiedSymbol("BaseThreeEdgeModeInequality"),
                ),
                mode_definition_with_label(
                    "MiddleThreeEdgeModeInequality",
                    "MiddleThreeEdgeModeInequalityDef",
                    ReserveTypeShape::QualifiedSymbol("InnerThreeEdgeModeInequality"),
                ),
                mode_definition_with_label(
                    "OuterThreeEdgeModeInequality",
                    "OuterThreeEdgeModeInequalityDef",
                    ReserveTypeShape::QualifiedSymbol("MiddleThreeEdgeModeInequality"),
                ),
            ]
        };
        let reserve = || {
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("OuterThreeEdgeModeInequality"),
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
        let payload = extract_source_three_edge_local_mode_reserved_variable_inequality(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact three-edge local-mode reserved-variable inequality should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 4);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(payload.reserve.bridge.bindings()[0].spelling, "z");
        assert_eq!(
            payload.reserve.bridge.bindings()[0].type_spelling,
            "OuterThreeEdgeModeInequality"
        );
        assert_eq!(payload.left_lookup_ordinal, 1);
        assert_eq!(payload.right_lookup_ordinal, 2);

        let output = source_three_edge_local_mode_reserved_variable_inequality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact three-edge local-mode inequality should reach TermFormulaChecker");
        assert_source_reserved_variable_formula_output(&output)
            .expect("three-edge local-mode inequality invariants should hold");
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
            assert_eq!(input.spelling, "OuterThreeEdgeModeInequality");
            assert!(matches!(input.head, TypeHeadInput::Symbol(_)));
        }
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("BaseThreeEdgeModeInequality")
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
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
        assert!(output.term_formula.facts().is_empty());
        assert!(output.term_formula.diagnostics().is_empty());

        for removed in [
            "BaseThreeEdgeModeInequality",
            "InnerThreeEdgeModeInequality",
            "MiddleThreeEdgeModeInequality",
            "OuterThreeEdgeModeInequality",
        ] {
            let mut invalid = source_three_edge_local_mode_reserved_variable_inequality_output(
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
            assert_eq!(
                source_reserved_variable_formula_output_detail_keys(&invalid),
                vec![
                    TYPE_ELABORATION_THREE_EDGE_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY
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

        for (index, radix) in [
            "BaseThreeEdgeModeInequality",
            "InnerThreeEdgeModeInequality",
            "MiddleThreeEdgeModeInequality",
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
                ReserveTypeShape::QualifiedSymbol("ExtraThreeEdgeModeInequality");
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
        direct_outermost_radix[3].rhs_shape = ReserveTypeShape::Builtin("set");
        assert_extraction_gap(direct_outermost_radix);

        let mut one_edge_outermost_radix = exact_modes();
        one_edge_outermost_radix[3].rhs_shape =
            ReserveTypeShape::QualifiedSymbol("BaseThreeEdgeModeInequality");
        assert_extraction_gap(one_edge_outermost_radix);

        let mut two_edge_outermost_radix = exact_modes();
        two_edge_outermost_radix[3].rhs_shape =
            ReserveTypeShape::QualifiedSymbol("InnerThreeEdgeModeInequality");
        assert_extraction_gap(two_edge_outermost_radix);

        let mut cyclic = exact_modes();
        cyclic[0].rhs_shape = ReserveTypeShape::QualifiedSymbol("OuterThreeEdgeModeInequality");
        assert_extraction_gap(cyclic);

        assert_extraction_gap(vec![
            exact_modes()[0],
            mode_definition_with_label(
                "ExtraThreeEdgeModeInequality",
                "ExtraThreeEdgeModeInequalityDef",
                ReserveTypeShape::QualifiedSymbol("BaseThreeEdgeModeInequality"),
            ),
            mode_definition_with_label(
                "InnerThreeEdgeModeInequality",
                "InnerThreeEdgeModeInequalityDef",
                ReserveTypeShape::QualifiedSymbol("ExtraThreeEdgeModeInequality"),
            ),
            exact_modes()[2],
            exact_modes()[3],
        ]);

        for near_miss in [
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(
                    vec!["z"],
                    ReserveTypeShape::QualifiedSymbol("MiddleThreeEdgeModeInequality"),
                )],
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(
                    vec!["z", "y"],
                    ReserveTypeShape::QualifiedSymbol("OuterThreeEdgeModeInequality"),
                )],
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(vec!["z"], ReserveTypeShape::Builtin("set"))],
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                vec![
                    reserve_item(
                        vec!["z"],
                        ReserveTypeShape::QualifiedSymbol("OuterThreeEdgeModeInequality"),
                    ),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                ],
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(
                    vec!["z"],
                    ReserveTypeShape::QualifiedSymbolWithArgs("OuterThreeEdgeModeInequality"),
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
    fn source_three_edge_local_object_mode_reserved_variable_inequality_consumes_four_expansions() {
        let source_id = source_id(157);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("three_edge_local_object_mode_reserved_variable_inequality"),
        );
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseThreeEdgeObjectModeInequality", SymbolKind::Mode),
                ("InnerThreeEdgeObjectModeInequality", SymbolKind::Mode),
                ("MiddleThreeEdgeObjectModeInequality", SymbolKind::Mode),
                ("OuterThreeEdgeObjectModeInequality", SymbolKind::Mode),
                ("ExtraThreeEdgeObjectModeInequality", SymbolKind::Mode),
            ],
        );
        let theorem = IdentifierBinaryTheoremSpec {
            status: None,
            label: "ThreeEdgeLocalObjectModeReservedVariableInequalityPayloadBoundary",
            left: "z",
            operator: "<>",
            right: "z",
            recovered_label: false,
        };
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    "BaseThreeEdgeObjectModeInequality",
                    "BaseThreeEdgeObjectModeInequalityDef",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition_with_label(
                    "InnerThreeEdgeObjectModeInequality",
                    "InnerThreeEdgeObjectModeInequalityDef",
                    ReserveTypeShape::QualifiedSymbol("BaseThreeEdgeObjectModeInequality"),
                ),
                mode_definition_with_label(
                    "MiddleThreeEdgeObjectModeInequality",
                    "MiddleThreeEdgeObjectModeInequalityDef",
                    ReserveTypeShape::QualifiedSymbol("InnerThreeEdgeObjectModeInequality"),
                ),
                mode_definition_with_label(
                    "OuterThreeEdgeObjectModeInequality",
                    "OuterThreeEdgeObjectModeInequalityDef",
                    ReserveTypeShape::QualifiedSymbol("MiddleThreeEdgeObjectModeInequality"),
                ),
            ]
        };
        let reserve = || {
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("OuterThreeEdgeObjectModeInequality"),
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
        let payload = extract_source_three_edge_local_object_mode_reserved_variable_inequality(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact three-edge local-object-mode reserved-variable inequality should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 4);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(payload.reserve.bridge.bindings()[0].spelling, "z");
        assert_eq!(
            payload.reserve.bridge.bindings()[0].type_spelling,
            "OuterThreeEdgeObjectModeInequality"
        );
        assert_eq!(payload.left_lookup_ordinal, 1);
        assert_eq!(payload.right_lookup_ordinal, 2);

        let output = source_three_edge_local_object_mode_reserved_variable_inequality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact three-edge local-object-mode inequality should reach TermFormulaChecker");
        assert_source_reserved_variable_formula_output(&output)
            .expect("three-edge local-object-mode inequality invariants should hold");
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
            assert_eq!(input.spelling, "OuterThreeEdgeObjectModeInequality");
            assert!(matches!(input.head, TypeHeadInput::Symbol(_)));
        }
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("BaseThreeEdgeObjectModeInequality")
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
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
        assert!(output.term_formula.facts().is_empty());
        assert!(output.term_formula.diagnostics().is_empty());

        for removed in [
            "BaseThreeEdgeObjectModeInequality",
            "InnerThreeEdgeObjectModeInequality",
            "MiddleThreeEdgeObjectModeInequality",
            "OuterThreeEdgeObjectModeInequality",
        ] {
            let mut invalid =
                source_three_edge_local_object_mode_reserved_variable_inequality_output(
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
            assert_eq!(
                source_reserved_variable_formula_output_detail_keys(&invalid),
                vec![
                    TYPE_ELABORATION_THREE_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY
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

        for (index, radix) in [
            "BaseThreeEdgeObjectModeInequality",
            "InnerThreeEdgeObjectModeInequality",
            "MiddleThreeEdgeObjectModeInequality",
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
                ReserveTypeShape::QualifiedSymbol("ExtraThreeEdgeObjectModeInequality");
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
        direct_outermost_radix[3].rhs_shape = ReserveTypeShape::Builtin("object");
        assert_extraction_gap(direct_outermost_radix);

        let mut one_edge_outermost_radix = exact_modes();
        one_edge_outermost_radix[3].rhs_shape =
            ReserveTypeShape::QualifiedSymbol("BaseThreeEdgeObjectModeInequality");
        assert_extraction_gap(one_edge_outermost_radix);

        let mut two_edge_outermost_radix = exact_modes();
        two_edge_outermost_radix[3].rhs_shape =
            ReserveTypeShape::QualifiedSymbol("InnerThreeEdgeObjectModeInequality");
        assert_extraction_gap(two_edge_outermost_radix);

        let mut cyclic = exact_modes();
        cyclic[0].rhs_shape =
            ReserveTypeShape::QualifiedSymbol("OuterThreeEdgeObjectModeInequality");
        assert_extraction_gap(cyclic);

        assert_extraction_gap(vec![
            exact_modes()[0],
            mode_definition_with_label(
                "ExtraThreeEdgeObjectModeInequality",
                "ExtraThreeEdgeObjectModeInequalityDef",
                ReserveTypeShape::QualifiedSymbol("BaseThreeEdgeObjectModeInequality"),
            ),
            mode_definition_with_label(
                "InnerThreeEdgeObjectModeInequality",
                "InnerThreeEdgeObjectModeInequalityDef",
                ReserveTypeShape::QualifiedSymbol("ExtraThreeEdgeObjectModeInequality"),
            ),
            exact_modes()[2],
            exact_modes()[3],
        ]);

        for near_miss in [
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(
                    vec!["z"],
                    ReserveTypeShape::QualifiedSymbol("MiddleThreeEdgeObjectModeInequality"),
                )],
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(
                    vec!["z", "y"],
                    ReserveTypeShape::QualifiedSymbol("OuterThreeEdgeObjectModeInequality"),
                )],
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(vec!["z"], ReserveTypeShape::Builtin("object"))],
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                vec![
                    reserve_item(
                        vec!["z"],
                        ReserveTypeShape::QualifiedSymbol("OuterThreeEdgeObjectModeInequality"),
                    ),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("object")),
                ],
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(
                    vec!["z"],
                    ReserveTypeShape::QualifiedSymbolWithArgs("OuterThreeEdgeObjectModeInequality"),
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
    fn source_three_edge_local_object_mode_reserved_variable_equality_consumes_four_expansions() {
        let source_id = source_id(155);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("three_edge_local_object_mode_reserved_variable_equality"),
        );
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseThreeEdgeObjectModeEquality", SymbolKind::Mode),
                ("InnerThreeEdgeObjectModeEquality", SymbolKind::Mode),
                ("MiddleThreeEdgeObjectModeEquality", SymbolKind::Mode),
                ("OuterThreeEdgeObjectModeEquality", SymbolKind::Mode),
                ("ExtraThreeEdgeObjectModeEquality", SymbolKind::Mode),
            ],
        );
        let theorem = IdentifierBinaryTheoremSpec {
            status: None,
            label: "ThreeEdgeLocalObjectModeReservedVariableEqualityPayloadBoundary",
            left: "z",
            operator: "=",
            right: "z",
            recovered_label: false,
        };
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    "BaseThreeEdgeObjectModeEquality",
                    "BaseThreeEdgeObjectModeEqualityDef",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition_with_label(
                    "InnerThreeEdgeObjectModeEquality",
                    "InnerThreeEdgeObjectModeEqualityDef",
                    ReserveTypeShape::QualifiedSymbol("BaseThreeEdgeObjectModeEquality"),
                ),
                mode_definition_with_label(
                    "MiddleThreeEdgeObjectModeEquality",
                    "MiddleThreeEdgeObjectModeEqualityDef",
                    ReserveTypeShape::QualifiedSymbol("InnerThreeEdgeObjectModeEquality"),
                ),
                mode_definition_with_label(
                    "OuterThreeEdgeObjectModeEquality",
                    "OuterThreeEdgeObjectModeEqualityDef",
                    ReserveTypeShape::QualifiedSymbol("MiddleThreeEdgeObjectModeEquality"),
                ),
            ]
        };
        let reserve = || {
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("OuterThreeEdgeObjectModeEquality"),
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
        let payload = extract_source_three_edge_local_object_mode_reserved_variable_equality(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact three-edge local-object-mode reserved-variable equality should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 4);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(payload.reserve.bridge.bindings()[0].spelling, "z");
        assert_eq!(
            payload.reserve.bridge.bindings()[0].type_spelling,
            "OuterThreeEdgeObjectModeEquality"
        );
        assert_eq!(payload.left_lookup_ordinal, 1);
        assert_eq!(payload.right_lookup_ordinal, 2);

        let output = source_three_edge_local_object_mode_reserved_variable_equality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact three-edge local-object-mode equality should reach TermFormulaChecker");
        assert_source_reserved_variable_formula_output(&output)
            .expect("three-edge local-object-mode equality invariants should hold");
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
            assert_eq!(input.spelling, "OuterThreeEdgeObjectModeEquality");
            assert!(matches!(input.head, TypeHeadInput::Symbol(_)));
        }
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("BaseThreeEdgeObjectModeEquality")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("base object mode terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("one normalized terminal type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
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
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
        assert!(output.term_formula.facts().is_empty());
        assert!(output.term_formula.diagnostics().is_empty());

        for removed in [
            "BaseThreeEdgeObjectModeEquality",
            "InnerThreeEdgeObjectModeEquality",
            "MiddleThreeEdgeObjectModeEquality",
            "OuterThreeEdgeObjectModeEquality",
        ] {
            let mut invalid =
                source_three_edge_local_object_mode_reserved_variable_equality_output(
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
            assert_eq!(
                source_reserved_variable_formula_output_detail_keys(&invalid),
                vec![
                    TYPE_ELABORATION_THREE_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY
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

        for (index, radix) in [
            "BaseThreeEdgeObjectModeEquality",
            "InnerThreeEdgeObjectModeEquality",
            "MiddleThreeEdgeObjectModeEquality",
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
                ReserveTypeShape::QualifiedSymbol("ExtraThreeEdgeObjectModeEquality");
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
        direct_outermost_radix[3].rhs_shape = ReserveTypeShape::Builtin("object");
        assert_extraction_gap(direct_outermost_radix);

        let mut one_edge_outermost_radix = exact_modes();
        one_edge_outermost_radix[3].rhs_shape =
            ReserveTypeShape::QualifiedSymbol("BaseThreeEdgeObjectModeEquality");
        assert_extraction_gap(one_edge_outermost_radix);

        let mut two_edge_outermost_radix = exact_modes();
        two_edge_outermost_radix[3].rhs_shape =
            ReserveTypeShape::QualifiedSymbol("InnerThreeEdgeObjectModeEquality");
        assert_extraction_gap(two_edge_outermost_radix);

        let mut cyclic = exact_modes();
        cyclic[0].rhs_shape = ReserveTypeShape::QualifiedSymbol("OuterThreeEdgeObjectModeEquality");
        assert_extraction_gap(cyclic);

        assert_extraction_gap(vec![
            exact_modes()[0],
            mode_definition_with_label(
                "ExtraThreeEdgeObjectModeEquality",
                "ExtraThreeEdgeObjectModeEqualityDef",
                ReserveTypeShape::QualifiedSymbol("BaseThreeEdgeObjectModeEquality"),
            ),
            mode_definition_with_label(
                "InnerThreeEdgeObjectModeEquality",
                "InnerThreeEdgeObjectModeEqualityDef",
                ReserveTypeShape::QualifiedSymbol("ExtraThreeEdgeObjectModeEquality"),
            ),
            exact_modes()[2],
            exact_modes()[3],
        ]);

        for near_miss in [
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(
                    vec!["z"],
                    ReserveTypeShape::QualifiedSymbolWithArgs("OuterThreeEdgeObjectModeEquality"),
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
    fn source_three_edge_local_mode_reserved_variable_membership_consumes_four_expansions() {
        let source_id = source_id(158);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("three_edge_local_mode_reserved_variable_membership"),
        );
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseThreeEdgeModeMembership", SymbolKind::Mode),
                ("InnerThreeEdgeModeMembership", SymbolKind::Mode),
                ("MiddleThreeEdgeModeMembership", SymbolKind::Mode),
                ("OuterThreeEdgeModeMembership", SymbolKind::Mode),
                ("ExtraThreeEdgeModeMembership", SymbolKind::Mode),
                ("TooDeepThreeEdgeModeMembership", SymbolKind::Mode),
            ],
        );
        let theorem = IdentifierBinaryTheoremSpec {
            status: None,
            label: "ThreeEdgeLocalModeReservedVariableMembershipPayloadBoundary",
            left: "x",
            operator: "in",
            right: "y",
            recovered_label: false,
        };
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    "BaseThreeEdgeModeMembership",
                    "BaseThreeEdgeModeMembershipDef",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition_with_label(
                    "InnerThreeEdgeModeMembership",
                    "InnerThreeEdgeModeMembershipDef",
                    ReserveTypeShape::QualifiedSymbol("BaseThreeEdgeModeMembership"),
                ),
                mode_definition_with_label(
                    "MiddleThreeEdgeModeMembership",
                    "MiddleThreeEdgeModeMembershipDef",
                    ReserveTypeShape::QualifiedSymbol("InnerThreeEdgeModeMembership"),
                ),
                mode_definition_with_label(
                    "OuterThreeEdgeModeMembership",
                    "OuterThreeEdgeModeMembershipDef",
                    ReserveTypeShape::QualifiedSymbol("MiddleThreeEdgeModeMembership"),
                ),
            ]
        };
        let exact_reserves = || {
            vec![
                reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbol("OuterThreeEdgeModeMembership"),
                ),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
            ]
        };
        let exact = mode_then_reserve_identifier_binary_theorem_ast(
            source_id,
            exact_modes(),
            exact_reserves(),
            theorem,
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        let payload = extract_source_three_edge_local_mode_reserved_variable_membership(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact three-edge local-mode membership should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 4);
        assert_eq!(payload.reserve.bridge.bindings().len(), 2);
        assert_eq!(
            payload.reserve.bridge.bindings()[0].type_spelling,
            "OuterThreeEdgeModeMembership"
        );
        assert_eq!(payload.reserve.bridge.bindings()[1].type_spelling, "set");
        assert_ne!(
            payload.reserve.bridge.bindings()[0].type_range,
            payload.reserve.bridge.bindings()[1].type_range
        );
        assert_eq!(payload.left_lookup_ordinal, 2);
        assert_eq!(payload.right_lookup_ordinal, 3);

        let output = source_three_edge_local_mode_reserved_variable_membership_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact three-edge local-mode membership should reach TermFormulaChecker");
        assert_source_reserved_variable_formula_output(&output)
            .expect("three-edge local-mode membership invariants should hold");
        assert_eq!(output.left_binding, BindingId::new(0));
        assert_eq!(output.right_binding, BindingId::new(1));
        assert_eq!(
            output.left_result_input.spelling,
            "OuterThreeEdgeModeMembership"
        );
        assert!(matches!(
            output.left_result_input.head,
            TypeHeadInput::Symbol(_)
        ));
        assert_eq!(output.right_result_input.head, TypeHeadInput::BuiltinSet);
        assert!(output.left_expected_input.is_none());
        let right_expected = output
            .right_expected_input
            .as_ref()
            .expect("right expected input should exist");
        assert_eq!(right_expected.head, TypeHeadInput::BuiltinSet);
        assert_eq!(
            output.right_result_input.source_range,
            right_expected.source_range
        );
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("BaseThreeEdgeModeMembership")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("base mode terminal expansion should exist");
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
        let formula = output
            .term_formula
            .formulas()
            .iter()
            .map(|(_, formula)| formula)
            .next()
            .expect("membership formula should be checked");
        assert_eq!(formula.kind, FormulaKind::Membership);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert_eq!(formula.expected_types.len(), 1);
        assert_eq!(formula.expected_types[0].term, output.payload.right_site);
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
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
                "three-edge-local-mode-reserved-variable-membership-left-result".to_owned(),
                "three-edge-local-mode-reserved-variable-membership-right-expected".to_owned(),
                "three-edge-local-mode-reserved-variable-membership-right-result".to_owned(),
            ])
        );

        for removed in [
            "BaseThreeEdgeModeMembership",
            "InnerThreeEdgeModeMembership",
            "MiddleThreeEdgeModeMembership",
            "OuterThreeEdgeModeMembership",
        ] {
            let mut invalid = source_three_edge_local_mode_reserved_variable_membership_output(
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
            assert_eq!(
                source_reserved_variable_formula_output_detail_keys(&invalid),
                vec![
                    TYPE_ELABORATION_THREE_EDGE_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY
                        .to_owned()
                ]
            );
        }

        let mut invalid_expected =
            source_three_edge_local_mode_reserved_variable_membership_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce a right-expected corruption target");
        invalid_expected
            .right_expected_input
            .as_mut()
            .expect("right expected input should exist")
            .head = TypeHeadInput::BuiltinObject;
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&invalid_expected),
            vec![
                TYPE_ELABORATION_THREE_EDGE_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY
                    .to_owned()
            ]
        );

        for index in 0..4 {
            let mut modes = exact_modes();
            modes.remove(index);
            let near_miss = mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                modes,
                exact_reserves(),
                theorem,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );

            let mut modes = exact_modes();
            modes.insert(index, modes[index]);
            let near_miss = mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                modes,
                exact_reserves(),
                theorem,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );

            for mutate in [
                |mode: &mut ModeDefinitionSpec| mode.label = Some("OtherDef"),
                |mode: &mut ModeDefinitionSpec| mode.recovered = true,
                |mode: &mut ModeDefinitionSpec| mode.local_context = true,
                |mode: &mut ModeDefinitionSpec| mode.parameterized_pattern = true,
            ] {
                let mut modes = exact_modes();
                mutate(&mut modes[index]);
                let near_miss = mode_then_reserve_identifier_binary_theorem_ast(
                    source_id,
                    modes,
                    exact_reserves(),
                    theorem,
                );
                assert_eq!(
                    source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                    vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
                );
            }
        }

        for (index, rhs_shape) in [
            (0, ReserveTypeShape::AttributedSet),
            (0, ReserveTypeShape::Builtin("object")),
            (
                0,
                ReserveTypeShape::QualifiedSymbol("InnerThreeEdgeModeMembership"),
            ),
            (1, ReserveTypeShape::Builtin("set")),
            (
                1,
                ReserveTypeShape::QualifiedSymbolWithArgs("BaseThreeEdgeModeMembership"),
            ),
            (
                1,
                ReserveTypeShape::QualifiedSymbol("OuterThreeEdgeModeMembership"),
            ),
            (2, ReserveTypeShape::Builtin("set")),
            (
                2,
                ReserveTypeShape::QualifiedSymbolWithArgs("InnerThreeEdgeModeMembership"),
            ),
            (
                2,
                ReserveTypeShape::QualifiedSymbol("OuterThreeEdgeModeMembership"),
            ),
            (3, ReserveTypeShape::Builtin("set")),
            (
                3,
                ReserveTypeShape::QualifiedSymbolWithArgs("MiddleThreeEdgeModeMembership"),
            ),
            (
                3,
                ReserveTypeShape::QualifiedSymbol("InnerThreeEdgeModeMembership"),
            ),
        ] {
            let mut modes = exact_modes();
            modes[index].rhs_shape = rhs_shape;
            let near_miss = mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                modes,
                exact_reserves(),
                theorem,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }

        for modes in [
            vec![mode_definition_with_label(
                "OuterThreeEdgeModeMembership",
                "OuterThreeEdgeModeMembershipDef",
                ReserveTypeShape::Builtin("set"),
            )],
            vec![
                exact_modes()[0],
                mode_definition_with_label(
                    "OuterThreeEdgeModeMembership",
                    "OuterThreeEdgeModeMembershipDef",
                    ReserveTypeShape::QualifiedSymbol("BaseThreeEdgeModeMembership"),
                ),
            ],
            vec![exact_modes()[0], exact_modes()[2], exact_modes()[3]],
            vec![
                exact_modes()[3],
                exact_modes()[2],
                exact_modes()[1],
                exact_modes()[0],
            ],
            {
                let mut modes = exact_modes();
                modes.insert(
                    3,
                    mode_definition_with_label(
                        "ExtraThreeEdgeModeMembership",
                        "ExtraThreeEdgeModeMembershipDef",
                        ReserveTypeShape::QualifiedSymbol("MiddleThreeEdgeModeMembership"),
                    ),
                );
                modes[4] = mode_definition_with_label(
                    "OuterThreeEdgeModeMembership",
                    "OuterThreeEdgeModeMembershipDef",
                    ReserveTypeShape::QualifiedSymbol("ExtraThreeEdgeModeMembership"),
                );
                modes
            },
            {
                let mut modes = exact_modes();
                modes.insert(
                    3,
                    mode_definition_with_label(
                        "ExtraThreeEdgeModeMembership",
                        "ExtraThreeEdgeModeMembershipDef",
                        ReserveTypeShape::QualifiedSymbol("MiddleThreeEdgeModeMembership"),
                    ),
                );
                modes.insert(
                    4,
                    mode_definition_with_label(
                        "TooDeepThreeEdgeModeMembership",
                        "TooDeepThreeEdgeModeMembershipDef",
                        ReserveTypeShape::QualifiedSymbol("ExtraThreeEdgeModeMembership"),
                    ),
                );
                modes[5] = mode_definition_with_label(
                    "OuterThreeEdgeModeMembership",
                    "OuterThreeEdgeModeMembershipDef",
                    ReserveTypeShape::QualifiedSymbol("TooDeepThreeEdgeModeMembership"),
                );
                modes
            },
        ] {
            let near_miss = mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                modes,
                exact_reserves(),
                theorem,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }

        let near_miss_reserves = [
            vec![
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbol("OuterThreeEdgeModeMembership"),
                ),
            ],
            vec![reserve_item(
                vec!["x", "y"],
                ReserveTypeShape::QualifiedSymbol("OuterThreeEdgeModeMembership"),
            )],
            vec![
                reserve_item(vec!["x"], ReserveTypeShape::Builtin("set")),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
            ],
            vec![
                reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbol("OuterThreeEdgeModeMembership"),
                ),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("object")),
            ],
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("OuterThreeEdgeModeMembership"),
            )],
            {
                let mut reserves = exact_reserves();
                reserves.push(reserve_item(vec!["z"], ReserveTypeShape::Builtin("set")));
                reserves
            },
            vec![
                reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbolWithArgs("OuterThreeEdgeModeMembership"),
                ),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
            ],
        ];
        for reserves in near_miss_reserves {
            let near_miss = mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                reserves,
                theorem,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }

        for near_miss in [
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                exact_reserves(),
                IdentifierBinaryTheoremSpec {
                    left: "y",
                    right: "x",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                exact_reserves(),
                IdentifierBinaryTheoremSpec {
                    right: "x",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                exact_reserves(),
                IdentifierBinaryTheoremSpec {
                    label: "OtherPayloadBoundary",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                exact_reserves(),
                IdentifierBinaryTheoremSpec {
                    operator: "=",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                exact_reserves(),
                IdentifierBinaryTheoremSpec {
                    status: Some("registration"),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                exact_reserves(),
                IdentifierBinaryTheoremSpec {
                    recovered_label: true,
                    ..theorem
                },
            ),
            modes_then_empty_definition_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                exact_reserves(),
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
    fn source_four_edge_local_mode_reserved_variable_membership_consumes_five_expansions() {
        let source_id = source_id(164);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("four_edge_local_mode_reserved_variable_membership"),
        );
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseFourEdgeModeMembership", SymbolKind::Mode),
                ("InnerFourEdgeModeMembership", SymbolKind::Mode),
                ("MiddleFourEdgeModeMembership", SymbolKind::Mode),
                ("OuterFourEdgeModeMembership", SymbolKind::Mode),
                ("ExtraFourEdgeModeMembership", SymbolKind::Mode),
                ("TooDeepFourEdgeModeMembership", SymbolKind::Mode),
                ("EvenDeeperFourEdgeModeMembership", SymbolKind::Mode),
            ],
        );
        let theorem = IdentifierBinaryTheoremSpec {
            status: None,
            label: "FourEdgeLocalModeReservedVariableMembershipPayloadBoundary",
            left: "x",
            operator: "in",
            right: "y",
            recovered_label: false,
        };
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    "BaseFourEdgeModeMembership",
                    "BaseFourEdgeModeMembershipDef",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition_with_label(
                    "InnerFourEdgeModeMembership",
                    "InnerFourEdgeModeMembershipDef",
                    ReserveTypeShape::QualifiedSymbol("BaseFourEdgeModeMembership"),
                ),
                mode_definition_with_label(
                    "MiddleFourEdgeModeMembership",
                    "MiddleFourEdgeModeMembershipDef",
                    ReserveTypeShape::QualifiedSymbol("InnerFourEdgeModeMembership"),
                ),
                mode_definition_with_label(
                    "OuterFourEdgeModeMembership",
                    "OuterFourEdgeModeMembershipDef",
                    ReserveTypeShape::QualifiedSymbol("MiddleFourEdgeModeMembership"),
                ),
                mode_definition_with_label(
                    "TooDeepFourEdgeModeMembership",
                    "TooDeepFourEdgeModeMembershipDef",
                    ReserveTypeShape::QualifiedSymbol("OuterFourEdgeModeMembership"),
                ),
            ]
        };
        let exact_reserves = || {
            vec![
                reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbol("TooDeepFourEdgeModeMembership"),
                ),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
            ]
        };
        let exact = mode_then_reserve_identifier_binary_theorem_ast(
            source_id,
            exact_modes(),
            exact_reserves(),
            theorem,
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        let payload = extract_source_four_edge_local_mode_reserved_variable_membership(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact four-edge local-mode membership should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 5);
        assert_eq!(payload.reserve.bridge.bindings().len(), 2);
        assert_eq!(
            payload.reserve.bridge.bindings()[0].type_spelling,
            "TooDeepFourEdgeModeMembership"
        );
        assert_eq!(payload.reserve.bridge.bindings()[1].type_spelling, "set");
        assert_ne!(
            payload.reserve.bridge.bindings()[0].type_range,
            payload.reserve.bridge.bindings()[1].type_range
        );
        assert_eq!(payload.left_lookup_ordinal, 2);
        assert_eq!(payload.right_lookup_ordinal, 3);

        let output = source_four_edge_local_mode_reserved_variable_membership_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact four-edge local-mode membership should reach TermFormulaChecker");
        assert_source_reserved_variable_formula_output(&output)
            .expect("four-edge local-mode membership invariants should hold");
        assert_eq!(output.left_binding, BindingId::new(0));
        assert_eq!(output.right_binding, BindingId::new(1));
        assert_eq!(
            output.left_result_input.spelling,
            "TooDeepFourEdgeModeMembership"
        );
        assert!(matches!(
            output.left_result_input.head,
            TypeHeadInput::Symbol(_)
        ));
        assert_eq!(output.right_result_input.head, TypeHeadInput::BuiltinSet);
        assert!(output.left_expected_input.is_none());
        let right_expected = output
            .right_expected_input
            .as_ref()
            .expect("right expected input should exist");
        assert_eq!(right_expected.head, TypeHeadInput::BuiltinSet);
        assert_eq!(
            output.right_result_input.source_range,
            right_expected.source_range
        );
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("BaseFourEdgeModeMembership")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("base mode terminal expansion should exist");
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
        let formula = output
            .term_formula
            .formulas()
            .iter()
            .map(|(_, formula)| formula)
            .next()
            .expect("membership formula should be checked");
        assert_eq!(formula.kind, FormulaKind::Membership);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert_eq!(formula.expected_types.len(), 1);
        assert_eq!(formula.expected_types[0].term, output.payload.right_site);
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
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
                "four-edge-local-mode-reserved-variable-membership-left-result".to_owned(),
                "four-edge-local-mode-reserved-variable-membership-right-expected".to_owned(),
                "four-edge-local-mode-reserved-variable-membership-right-result".to_owned(),
            ])
        );

        for removed in [
            "BaseFourEdgeModeMembership",
            "InnerFourEdgeModeMembership",
            "MiddleFourEdgeModeMembership",
            "OuterFourEdgeModeMembership",
            "TooDeepFourEdgeModeMembership",
        ] {
            let mut invalid = source_four_edge_local_mode_reserved_variable_membership_output(
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
            assert_eq!(
                source_reserved_variable_formula_output_detail_keys(&invalid),
                vec![
                    TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY
                        .to_owned()
                ]
            );
        }

        let mut invalid_expected = source_four_edge_local_mode_reserved_variable_membership_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce a right-expected corruption target");
        invalid_expected
            .right_expected_input
            .as_mut()
            .expect("right expected input should exist")
            .head = TypeHeadInput::BuiltinObject;
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&invalid_expected),
            vec![
                TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY
                    .to_owned()
            ]
        );

        for index in 0..5 {
            let mut modes = exact_modes();
            modes.remove(index);
            let near_miss = mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                modes,
                exact_reserves(),
                theorem,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );

            let mut modes = exact_modes();
            modes.insert(index, modes[index]);
            let near_miss = mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                modes,
                exact_reserves(),
                theorem,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );

            for mutate in [
                |mode: &mut ModeDefinitionSpec| mode.label = Some("OtherDef"),
                |mode: &mut ModeDefinitionSpec| mode.recovered = true,
                |mode: &mut ModeDefinitionSpec| mode.local_context = true,
                |mode: &mut ModeDefinitionSpec| mode.parameterized_pattern = true,
            ] {
                let mut modes = exact_modes();
                mutate(&mut modes[index]);
                let near_miss = mode_then_reserve_identifier_binary_theorem_ast(
                    source_id,
                    modes,
                    exact_reserves(),
                    theorem,
                );
                assert_eq!(
                    source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                    vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
                );
            }
        }

        for (index, rhs_shape) in [
            (0, ReserveTypeShape::AttributedSet),
            (0, ReserveTypeShape::Builtin("object")),
            (
                0,
                ReserveTypeShape::QualifiedSymbol("InnerFourEdgeModeMembership"),
            ),
            (1, ReserveTypeShape::Builtin("set")),
            (
                1,
                ReserveTypeShape::QualifiedSymbolWithArgs("BaseFourEdgeModeMembership"),
            ),
            (
                1,
                ReserveTypeShape::QualifiedSymbol("TooDeepFourEdgeModeMembership"),
            ),
            (2, ReserveTypeShape::Builtin("set")),
            (
                2,
                ReserveTypeShape::QualifiedSymbolWithArgs("InnerFourEdgeModeMembership"),
            ),
            (
                2,
                ReserveTypeShape::QualifiedSymbol("TooDeepFourEdgeModeMembership"),
            ),
            (3, ReserveTypeShape::Builtin("set")),
            (
                3,
                ReserveTypeShape::QualifiedSymbolWithArgs("MiddleFourEdgeModeMembership"),
            ),
            (
                3,
                ReserveTypeShape::QualifiedSymbol("InnerFourEdgeModeMembership"),
            ),
            (4, ReserveTypeShape::Builtin("set")),
            (
                4,
                ReserveTypeShape::QualifiedSymbolWithArgs("OuterFourEdgeModeMembership"),
            ),
            (
                4,
                ReserveTypeShape::QualifiedSymbol("MiddleFourEdgeModeMembership"),
            ),
        ] {
            let mut modes = exact_modes();
            modes[index].rhs_shape = rhs_shape;
            let near_miss = mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                modes,
                exact_reserves(),
                theorem,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }

        for modes in [
            vec![mode_definition_with_label(
                "TooDeepFourEdgeModeMembership",
                "TooDeepFourEdgeModeMembershipDef",
                ReserveTypeShape::Builtin("set"),
            )],
            vec![
                exact_modes()[0],
                mode_definition_with_label(
                    "TooDeepFourEdgeModeMembership",
                    "TooDeepFourEdgeModeMembershipDef",
                    ReserveTypeShape::QualifiedSymbol("BaseFourEdgeModeMembership"),
                ),
            ],
            vec![
                exact_modes()[0],
                exact_modes()[2],
                exact_modes()[3],
                exact_modes()[4],
            ],
            vec![
                exact_modes()[4],
                exact_modes()[3],
                exact_modes()[2],
                exact_modes()[1],
                exact_modes()[0],
            ],
            {
                let mut modes = exact_modes();
                modes.insert(
                    4,
                    mode_definition_with_label(
                        "ExtraFourEdgeModeMembership",
                        "ExtraFourEdgeModeMembershipDef",
                        ReserveTypeShape::QualifiedSymbol("OuterFourEdgeModeMembership"),
                    ),
                );
                modes[5] = mode_definition_with_label(
                    "TooDeepFourEdgeModeMembership",
                    "TooDeepFourEdgeModeMembershipDef",
                    ReserveTypeShape::QualifiedSymbol("ExtraFourEdgeModeMembership"),
                );
                modes
            },
            {
                let mut modes = exact_modes();
                modes.push(mode_definition_with_label(
                    "EvenDeeperFourEdgeModeMembership",
                    "EvenDeeperFourEdgeModeMembershipDef",
                    ReserveTypeShape::QualifiedSymbol("TooDeepFourEdgeModeMembership"),
                ));
                modes
            },
        ] {
            let near_miss = mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                modes,
                exact_reserves(),
                theorem,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }

        let near_miss_reserves = [
            vec![
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbol("TooDeepFourEdgeModeMembership"),
                ),
            ],
            vec![reserve_item(
                vec!["x", "y"],
                ReserveTypeShape::QualifiedSymbol("TooDeepFourEdgeModeMembership"),
            )],
            vec![
                reserve_item(vec!["x"], ReserveTypeShape::Builtin("set")),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
            ],
            vec![
                reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbol("TooDeepFourEdgeModeMembership"),
                ),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("object")),
            ],
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("TooDeepFourEdgeModeMembership"),
            )],
            {
                let mut reserves = exact_reserves();
                reserves.push(reserve_item(vec!["z"], ReserveTypeShape::Builtin("set")));
                reserves
            },
            vec![
                reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbolWithArgs("TooDeepFourEdgeModeMembership"),
                ),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
            ],
        ];
        for reserves in near_miss_reserves {
            let near_miss = mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                reserves,
                theorem,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }

        for near_miss in [
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                exact_reserves(),
                IdentifierBinaryTheoremSpec {
                    left: "y",
                    right: "x",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                exact_reserves(),
                IdentifierBinaryTheoremSpec {
                    right: "x",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                exact_reserves(),
                IdentifierBinaryTheoremSpec {
                    label: "OtherPayloadBoundary",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                exact_reserves(),
                IdentifierBinaryTheoremSpec {
                    operator: "=",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                exact_reserves(),
                IdentifierBinaryTheoremSpec {
                    status: Some("registration"),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                exact_reserves(),
                IdentifierBinaryTheoremSpec {
                    recovered_label: true,
                    ..theorem
                },
            ),
            modes_then_empty_definition_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                exact_reserves(),
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
    fn source_four_edge_local_object_mode_reserved_variable_membership_consumes_five_expansions() {
        let source_id = source_id(165);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("four_edge_local_object_mode_reserved_variable_membership"),
        );
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseFourEdgeObjectModeMembership", SymbolKind::Mode),
                ("InnerFourEdgeObjectModeMembership", SymbolKind::Mode),
                ("MiddleFourEdgeObjectModeMembership", SymbolKind::Mode),
                ("OuterFourEdgeObjectModeMembership", SymbolKind::Mode),
                ("ExtraFourEdgeObjectModeMembership", SymbolKind::Mode),
                ("TooDeepFourEdgeObjectModeMembership", SymbolKind::Mode),
                ("EvenDeeperFourEdgeObjectModeMembership", SymbolKind::Mode),
            ],
        );
        let theorem = IdentifierBinaryTheoremSpec {
            status: None,
            label: "FourEdgeLocalObjectModeReservedVariableMembershipPayloadBoundary",
            left: "x",
            operator: "in",
            right: "y",
            recovered_label: false,
        };
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    "BaseFourEdgeObjectModeMembership",
                    "BaseFourEdgeObjectModeMembershipDef",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition_with_label(
                    "InnerFourEdgeObjectModeMembership",
                    "InnerFourEdgeObjectModeMembershipDef",
                    ReserveTypeShape::QualifiedSymbol("BaseFourEdgeObjectModeMembership"),
                ),
                mode_definition_with_label(
                    "MiddleFourEdgeObjectModeMembership",
                    "MiddleFourEdgeObjectModeMembershipDef",
                    ReserveTypeShape::QualifiedSymbol("InnerFourEdgeObjectModeMembership"),
                ),
                mode_definition_with_label(
                    "OuterFourEdgeObjectModeMembership",
                    "OuterFourEdgeObjectModeMembershipDef",
                    ReserveTypeShape::QualifiedSymbol("MiddleFourEdgeObjectModeMembership"),
                ),
                mode_definition_with_label(
                    "TooDeepFourEdgeObjectModeMembership",
                    "TooDeepFourEdgeObjectModeMembershipDef",
                    ReserveTypeShape::QualifiedSymbol("OuterFourEdgeObjectModeMembership"),
                ),
            ]
        };
        let exact_reserves = || {
            vec![
                reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbol("TooDeepFourEdgeObjectModeMembership"),
                ),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
            ]
        };
        let exact = mode_then_reserve_identifier_binary_theorem_ast(
            source_id,
            exact_modes(),
            exact_reserves(),
            theorem,
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        let payload = extract_source_four_edge_local_object_mode_reserved_variable_membership(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact four-edge local-object-mode membership should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 5);
        assert_eq!(payload.reserve.bridge.bindings().len(), 2);
        assert_eq!(
            payload.reserve.bridge.bindings()[0].type_spelling,
            "TooDeepFourEdgeObjectModeMembership"
        );
        assert_eq!(payload.reserve.bridge.bindings()[1].type_spelling, "set");
        assert_ne!(
            payload.reserve.bridge.bindings()[0].type_range,
            payload.reserve.bridge.bindings()[1].type_range
        );
        assert_eq!(payload.left_lookup_ordinal, 2);
        assert_eq!(payload.right_lookup_ordinal, 3);

        let output = source_four_edge_local_object_mode_reserved_variable_membership_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact four-edge local-object-mode membership should reach TermFormulaChecker");
        assert_source_reserved_variable_formula_output(&output)
            .expect("four-edge local-object-mode membership invariants should hold");
        assert_eq!(output.left_binding, BindingId::new(0));
        assert_eq!(output.right_binding, BindingId::new(1));
        assert_eq!(
            output.left_result_input.spelling,
            "TooDeepFourEdgeObjectModeMembership"
        );
        assert!(matches!(
            output.left_result_input.head,
            TypeHeadInput::Symbol(_)
        ));
        assert_eq!(output.right_result_input.head, TypeHeadInput::BuiltinSet);
        assert!(output.left_expected_input.is_none());
        let right_expected = output
            .right_expected_input
            .as_ref()
            .expect("right expected input should exist");
        assert_eq!(right_expected.head, TypeHeadInput::BuiltinSet);
        assert_eq!(
            output.right_result_input.source_range,
            right_expected.source_range
        );
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("BaseFourEdgeObjectModeMembership")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("base mode terminal expansion should exist");
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
        let formula = output
            .term_formula
            .formulas()
            .iter()
            .map(|(_, formula)| formula)
            .next()
            .expect("membership formula should be checked");
        assert_eq!(formula.kind, FormulaKind::Membership);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert_eq!(formula.expected_types.len(), 1);
        assert_eq!(formula.expected_types[0].term, output.payload.right_site);
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
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
                "four-edge-local-object-mode-reserved-variable-membership-left-result".to_owned(),
                "four-edge-local-object-mode-reserved-variable-membership-right-expected"
                    .to_owned(),
                "four-edge-local-object-mode-reserved-variable-membership-right-result".to_owned(),
            ])
        );

        for removed in [
            "BaseFourEdgeObjectModeMembership",
            "InnerFourEdgeObjectModeMembership",
            "MiddleFourEdgeObjectModeMembership",
            "OuterFourEdgeObjectModeMembership",
            "TooDeepFourEdgeObjectModeMembership",
        ] {
            let mut invalid =
                source_four_edge_local_object_mode_reserved_variable_membership_output(
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
            assert_eq!(
                source_reserved_variable_formula_output_detail_keys(&invalid),
                vec![
                    TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY
                        .to_owned()
                ]
            );
        }

        let mut invalid_expected =
            source_four_edge_local_object_mode_reserved_variable_membership_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce a right-expected corruption target");
        invalid_expected
            .right_expected_input
            .as_mut()
            .expect("right expected input should exist")
            .head = TypeHeadInput::BuiltinObject;
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&invalid_expected),
            vec![
                TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY
                    .to_owned()
            ]
        );

        for index in 0..5 {
            let mut modes = exact_modes();
            modes.remove(index);
            let near_miss = mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                modes,
                exact_reserves(),
                theorem,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );

            let mut modes = exact_modes();
            modes.insert(index, modes[index]);
            let near_miss = mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                modes,
                exact_reserves(),
                theorem,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );

            for mutate in [
                |mode: &mut ModeDefinitionSpec| mode.label = Some("OtherDef"),
                |mode: &mut ModeDefinitionSpec| mode.recovered = true,
                |mode: &mut ModeDefinitionSpec| mode.local_context = true,
                |mode: &mut ModeDefinitionSpec| mode.parameterized_pattern = true,
            ] {
                let mut modes = exact_modes();
                mutate(&mut modes[index]);
                let near_miss = mode_then_reserve_identifier_binary_theorem_ast(
                    source_id,
                    modes,
                    exact_reserves(),
                    theorem,
                );
                assert_eq!(
                    source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                    vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
                );
            }
        }

        for (index, rhs_shape) in [
            (0, ReserveTypeShape::AttributedSet),
            (0, ReserveTypeShape::Builtin("set")),
            (
                0,
                ReserveTypeShape::QualifiedSymbol("InnerFourEdgeObjectModeMembership"),
            ),
            (1, ReserveTypeShape::Builtin("set")),
            (
                1,
                ReserveTypeShape::QualifiedSymbolWithArgs("BaseFourEdgeObjectModeMembership"),
            ),
            (
                1,
                ReserveTypeShape::QualifiedSymbol("TooDeepFourEdgeObjectModeMembership"),
            ),
            (2, ReserveTypeShape::Builtin("set")),
            (
                2,
                ReserveTypeShape::QualifiedSymbolWithArgs("InnerFourEdgeObjectModeMembership"),
            ),
            (
                2,
                ReserveTypeShape::QualifiedSymbol("TooDeepFourEdgeObjectModeMembership"),
            ),
            (3, ReserveTypeShape::Builtin("set")),
            (
                3,
                ReserveTypeShape::QualifiedSymbolWithArgs("MiddleFourEdgeObjectModeMembership"),
            ),
            (
                3,
                ReserveTypeShape::QualifiedSymbol("InnerFourEdgeObjectModeMembership"),
            ),
            (4, ReserveTypeShape::Builtin("set")),
            (
                4,
                ReserveTypeShape::QualifiedSymbolWithArgs("OuterFourEdgeObjectModeMembership"),
            ),
            (
                4,
                ReserveTypeShape::QualifiedSymbol("MiddleFourEdgeObjectModeMembership"),
            ),
        ] {
            let mut modes = exact_modes();
            modes[index].rhs_shape = rhs_shape;
            let near_miss = mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                modes,
                exact_reserves(),
                theorem,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }

        for modes in [
            vec![mode_definition_with_label(
                "TooDeepFourEdgeObjectModeMembership",
                "TooDeepFourEdgeObjectModeMembershipDef",
                ReserveTypeShape::Builtin("object"),
            )],
            vec![
                exact_modes()[0],
                mode_definition_with_label(
                    "TooDeepFourEdgeObjectModeMembership",
                    "TooDeepFourEdgeObjectModeMembershipDef",
                    ReserveTypeShape::QualifiedSymbol("BaseFourEdgeObjectModeMembership"),
                ),
            ],
            vec![
                exact_modes()[0],
                exact_modes()[2],
                exact_modes()[3],
                exact_modes()[4],
            ],
            vec![
                exact_modes()[4],
                exact_modes()[3],
                exact_modes()[2],
                exact_modes()[1],
                exact_modes()[0],
            ],
            {
                let mut modes = exact_modes();
                modes.insert(
                    4,
                    mode_definition_with_label(
                        "ExtraFourEdgeObjectModeMembership",
                        "ExtraFourEdgeObjectModeMembershipDef",
                        ReserveTypeShape::QualifiedSymbol("OuterFourEdgeObjectModeMembership"),
                    ),
                );
                modes[5] = mode_definition_with_label(
                    "TooDeepFourEdgeObjectModeMembership",
                    "TooDeepFourEdgeObjectModeMembershipDef",
                    ReserveTypeShape::QualifiedSymbol("ExtraFourEdgeObjectModeMembership"),
                );
                modes
            },
            {
                let mut modes = exact_modes();
                modes.push(mode_definition_with_label(
                    "EvenDeeperFourEdgeObjectModeMembership",
                    "EvenDeeperFourEdgeObjectModeMembershipDef",
                    ReserveTypeShape::QualifiedSymbol("TooDeepFourEdgeObjectModeMembership"),
                ));
                modes
            },
        ] {
            let near_miss = mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                modes,
                exact_reserves(),
                theorem,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }

        let near_miss_reserves = [
            vec![
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbol("TooDeepFourEdgeObjectModeMembership"),
                ),
            ],
            vec![reserve_item(
                vec!["x", "y"],
                ReserveTypeShape::QualifiedSymbol("TooDeepFourEdgeObjectModeMembership"),
            )],
            vec![
                reserve_item(vec!["x"], ReserveTypeShape::Builtin("set")),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
            ],
            vec![
                reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbol("TooDeepFourEdgeObjectModeMembership"),
                ),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("object")),
            ],
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("TooDeepFourEdgeObjectModeMembership"),
            )],
            {
                let mut reserves = exact_reserves();
                reserves.push(reserve_item(vec!["z"], ReserveTypeShape::Builtin("set")));
                reserves
            },
            vec![
                reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbolWithArgs(
                        "TooDeepFourEdgeObjectModeMembership",
                    ),
                ),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
            ],
        ];
        for reserves in near_miss_reserves {
            let near_miss = mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                reserves,
                theorem,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }

        for near_miss in [
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                exact_reserves(),
                IdentifierBinaryTheoremSpec {
                    left: "y",
                    right: "x",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                exact_reserves(),
                IdentifierBinaryTheoremSpec {
                    right: "x",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                exact_reserves(),
                IdentifierBinaryTheoremSpec {
                    label: "OtherPayloadBoundary",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                exact_reserves(),
                IdentifierBinaryTheoremSpec {
                    operator: "=",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                exact_reserves(),
                IdentifierBinaryTheoremSpec {
                    status: Some("registration"),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                exact_reserves(),
                IdentifierBinaryTheoremSpec {
                    recovered_label: true,
                    ..theorem
                },
            ),
            modes_then_empty_definition_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                exact_reserves(),
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
    fn source_three_edge_local_object_mode_reserved_variable_membership_consumes_four_expansions() {
        let source_id = source_id(163);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("three_edge_local_object_mode_reserved_variable_membership"),
        );
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseThreeEdgeObjectModeMembership", SymbolKind::Mode),
                ("InnerThreeEdgeObjectModeMembership", SymbolKind::Mode),
                ("MiddleThreeEdgeObjectModeMembership", SymbolKind::Mode),
                ("OuterThreeEdgeObjectModeMembership", SymbolKind::Mode),
                ("ExtraThreeEdgeObjectModeMembership", SymbolKind::Mode),
                ("TooDeepThreeEdgeObjectModeMembership", SymbolKind::Mode),
            ],
        );
        let theorem = IdentifierBinaryTheoremSpec {
            status: None,
            label: "ThreeEdgeLocalObjectModeReservedVariableMembershipPayloadBoundary",
            left: "x",
            operator: "in",
            right: "y",
            recovered_label: false,
        };
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    "BaseThreeEdgeObjectModeMembership",
                    "BaseThreeEdgeObjectModeMembershipDef",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition_with_label(
                    "InnerThreeEdgeObjectModeMembership",
                    "InnerThreeEdgeObjectModeMembershipDef",
                    ReserveTypeShape::QualifiedSymbol("BaseThreeEdgeObjectModeMembership"),
                ),
                mode_definition_with_label(
                    "MiddleThreeEdgeObjectModeMembership",
                    "MiddleThreeEdgeObjectModeMembershipDef",
                    ReserveTypeShape::QualifiedSymbol("InnerThreeEdgeObjectModeMembership"),
                ),
                mode_definition_with_label(
                    "OuterThreeEdgeObjectModeMembership",
                    "OuterThreeEdgeObjectModeMembershipDef",
                    ReserveTypeShape::QualifiedSymbol("MiddleThreeEdgeObjectModeMembership"),
                ),
            ]
        };
        let exact_reserves = || {
            vec![
                reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbol("OuterThreeEdgeObjectModeMembership"),
                ),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
            ]
        };
        let exact = mode_then_reserve_identifier_binary_theorem_ast(
            source_id,
            exact_modes(),
            exact_reserves(),
            theorem,
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        let payload = extract_source_three_edge_local_object_mode_reserved_variable_membership(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact three-edge local-object-mode membership should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 4);
        assert_eq!(payload.reserve.bridge.bindings().len(), 2);
        assert_eq!(
            payload.reserve.bridge.bindings()[0].type_spelling,
            "OuterThreeEdgeObjectModeMembership"
        );
        assert_eq!(payload.reserve.bridge.bindings()[1].type_spelling, "set");
        assert_ne!(
            payload.reserve.bridge.bindings()[0].type_range,
            payload.reserve.bridge.bindings()[1].type_range
        );
        assert_eq!(payload.left_lookup_ordinal, 2);
        assert_eq!(payload.right_lookup_ordinal, 3);

        let output = source_three_edge_local_object_mode_reserved_variable_membership_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact three-edge local-object-mode membership should reach TermFormulaChecker");
        assert_source_reserved_variable_formula_output(&output)
            .expect("three-edge local-object-mode membership invariants should hold");
        assert_eq!(output.left_binding, BindingId::new(0));
        assert_eq!(output.right_binding, BindingId::new(1));
        assert_eq!(
            output.left_result_input.spelling,
            "OuterThreeEdgeObjectModeMembership"
        );
        assert!(matches!(
            output.left_result_input.head,
            TypeHeadInput::Symbol(_)
        ));
        assert_eq!(output.right_result_input.head, TypeHeadInput::BuiltinSet);
        assert!(output.left_expected_input.is_none());
        let right_expected = output
            .right_expected_input
            .as_ref()
            .expect("right expected input should exist");
        assert_eq!(right_expected.head, TypeHeadInput::BuiltinSet);
        assert_eq!(
            output.right_result_input.source_range,
            right_expected.source_range
        );
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("BaseThreeEdgeObjectModeMembership")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("base mode terminal expansion should exist");
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
        let formula = output
            .term_formula
            .formulas()
            .iter()
            .map(|(_, formula)| formula)
            .next()
            .expect("membership formula should be checked");
        assert_eq!(formula.kind, FormulaKind::Membership);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert_eq!(formula.expected_types.len(), 1);
        assert_eq!(formula.expected_types[0].term, output.payload.right_site);
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
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
                "three-edge-local-object-mode-reserved-variable-membership-left-result".to_owned(),
                "three-edge-local-object-mode-reserved-variable-membership-right-expected"
                    .to_owned(),
                "three-edge-local-object-mode-reserved-variable-membership-right-result".to_owned(),
            ])
        );

        for removed in [
            "BaseThreeEdgeObjectModeMembership",
            "InnerThreeEdgeObjectModeMembership",
            "MiddleThreeEdgeObjectModeMembership",
            "OuterThreeEdgeObjectModeMembership",
        ] {
            let mut invalid =
                source_three_edge_local_object_mode_reserved_variable_membership_output(
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
            assert_eq!(
                source_reserved_variable_formula_output_detail_keys(&invalid),
                vec![
                    TYPE_ELABORATION_THREE_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY
                        .to_owned()
                ]
            );
        }

        let mut invalid_expected =
            source_three_edge_local_object_mode_reserved_variable_membership_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce a right-expected corruption target");
        invalid_expected
            .right_expected_input
            .as_mut()
            .expect("right expected input should exist")
            .head = TypeHeadInput::BuiltinObject;
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&invalid_expected),
            vec![
                TYPE_ELABORATION_THREE_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY
                    .to_owned()
            ]
        );

        for index in 0..4 {
            let mut modes = exact_modes();
            modes.remove(index);
            let near_miss = mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                modes,
                exact_reserves(),
                theorem,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );

            let mut modes = exact_modes();
            modes.insert(index, modes[index]);
            let near_miss = mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                modes,
                exact_reserves(),
                theorem,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );

            for mutate in [
                |mode: &mut ModeDefinitionSpec| mode.label = Some("OtherDef"),
                |mode: &mut ModeDefinitionSpec| mode.recovered = true,
                |mode: &mut ModeDefinitionSpec| mode.local_context = true,
                |mode: &mut ModeDefinitionSpec| mode.parameterized_pattern = true,
            ] {
                let mut modes = exact_modes();
                mutate(&mut modes[index]);
                let near_miss = mode_then_reserve_identifier_binary_theorem_ast(
                    source_id,
                    modes,
                    exact_reserves(),
                    theorem,
                );
                assert_eq!(
                    source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                    vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
                );
            }
        }

        for (index, rhs_shape) in [
            (0, ReserveTypeShape::AttributedSet),
            (0, ReserveTypeShape::Builtin("set")),
            (
                0,
                ReserveTypeShape::QualifiedSymbol("InnerThreeEdgeObjectModeMembership"),
            ),
            (1, ReserveTypeShape::Builtin("set")),
            (
                1,
                ReserveTypeShape::QualifiedSymbolWithArgs("BaseThreeEdgeObjectModeMembership"),
            ),
            (
                1,
                ReserveTypeShape::QualifiedSymbol("OuterThreeEdgeObjectModeMembership"),
            ),
            (2, ReserveTypeShape::Builtin("set")),
            (
                2,
                ReserveTypeShape::QualifiedSymbolWithArgs("InnerThreeEdgeObjectModeMembership"),
            ),
            (
                2,
                ReserveTypeShape::QualifiedSymbol("OuterThreeEdgeObjectModeMembership"),
            ),
            (3, ReserveTypeShape::Builtin("set")),
            (
                3,
                ReserveTypeShape::QualifiedSymbolWithArgs("MiddleThreeEdgeObjectModeMembership"),
            ),
            (
                3,
                ReserveTypeShape::QualifiedSymbol("InnerThreeEdgeObjectModeMembership"),
            ),
        ] {
            let mut modes = exact_modes();
            modes[index].rhs_shape = rhs_shape;
            let near_miss = mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                modes,
                exact_reserves(),
                theorem,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }

        for modes in [
            vec![mode_definition_with_label(
                "OuterThreeEdgeObjectModeMembership",
                "OuterThreeEdgeObjectModeMembershipDef",
                ReserveTypeShape::Builtin("set"),
            )],
            vec![
                exact_modes()[0],
                mode_definition_with_label(
                    "OuterThreeEdgeObjectModeMembership",
                    "OuterThreeEdgeObjectModeMembershipDef",
                    ReserveTypeShape::QualifiedSymbol("BaseThreeEdgeObjectModeMembership"),
                ),
            ],
            vec![exact_modes()[0], exact_modes()[2], exact_modes()[3]],
            vec![
                exact_modes()[3],
                exact_modes()[2],
                exact_modes()[1],
                exact_modes()[0],
            ],
            {
                let mut modes = exact_modes();
                modes.insert(
                    3,
                    mode_definition_with_label(
                        "ExtraThreeEdgeObjectModeMembership",
                        "ExtraThreeEdgeObjectModeMembershipDef",
                        ReserveTypeShape::QualifiedSymbol("MiddleThreeEdgeObjectModeMembership"),
                    ),
                );
                modes[4] = mode_definition_with_label(
                    "OuterThreeEdgeObjectModeMembership",
                    "OuterThreeEdgeObjectModeMembershipDef",
                    ReserveTypeShape::QualifiedSymbol("ExtraThreeEdgeObjectModeMembership"),
                );
                modes
            },
            {
                let mut modes = exact_modes();
                modes.insert(
                    3,
                    mode_definition_with_label(
                        "ExtraThreeEdgeObjectModeMembership",
                        "ExtraThreeEdgeObjectModeMembershipDef",
                        ReserveTypeShape::QualifiedSymbol("MiddleThreeEdgeObjectModeMembership"),
                    ),
                );
                modes.insert(
                    4,
                    mode_definition_with_label(
                        "TooDeepThreeEdgeObjectModeMembership",
                        "TooDeepThreeEdgeObjectModeMembershipDef",
                        ReserveTypeShape::QualifiedSymbol("ExtraThreeEdgeObjectModeMembership"),
                    ),
                );
                modes[5] = mode_definition_with_label(
                    "OuterThreeEdgeObjectModeMembership",
                    "OuterThreeEdgeObjectModeMembershipDef",
                    ReserveTypeShape::QualifiedSymbol("TooDeepThreeEdgeObjectModeMembership"),
                );
                modes
            },
        ] {
            let near_miss = mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                modes,
                exact_reserves(),
                theorem,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }

        let near_miss_reserves = [
            vec![
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbol("OuterThreeEdgeObjectModeMembership"),
                ),
            ],
            vec![reserve_item(
                vec!["x", "y"],
                ReserveTypeShape::QualifiedSymbol("OuterThreeEdgeObjectModeMembership"),
            )],
            vec![
                reserve_item(vec!["x"], ReserveTypeShape::Builtin("set")),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
            ],
            vec![
                reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbol("OuterThreeEdgeObjectModeMembership"),
                ),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("object")),
            ],
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("OuterThreeEdgeObjectModeMembership"),
            )],
            {
                let mut reserves = exact_reserves();
                reserves.push(reserve_item(vec!["z"], ReserveTypeShape::Builtin("set")));
                reserves
            },
            vec![
                reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbolWithArgs("OuterThreeEdgeObjectModeMembership"),
                ),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
            ],
        ];
        for reserves in near_miss_reserves {
            let near_miss = mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                reserves,
                theorem,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }

        for near_miss in [
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                exact_reserves(),
                IdentifierBinaryTheoremSpec {
                    left: "y",
                    right: "x",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                exact_reserves(),
                IdentifierBinaryTheoremSpec {
                    right: "x",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                exact_reserves(),
                IdentifierBinaryTheoremSpec {
                    label: "OtherPayloadBoundary",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                exact_reserves(),
                IdentifierBinaryTheoremSpec {
                    operator: "=",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                exact_reserves(),
                IdentifierBinaryTheoremSpec {
                    status: Some("registration"),
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                exact_reserves(),
                IdentifierBinaryTheoremSpec {
                    recovered_label: true,
                    ..theorem
                },
            ),
            modes_then_empty_definition_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                exact_reserves(),
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
    fn source_two_edge_local_mode_reserved_variable_equality_consumes_three_expansions() {
        let source_id = source_id(134);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("two_edge_local_mode_reserved_variable_equality"),
        );
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseTwoEdgeModeEquality", SymbolKind::Mode),
                ("MiddleTwoEdgeModeEquality", SymbolKind::Mode),
                ("OuterTwoEdgeModeEquality", SymbolKind::Mode),
                ("ExtraTwoEdgeModeEquality", SymbolKind::Mode),
            ],
        );
        let theorem = IdentifierBinaryTheoremSpec {
            status: None,
            label: "TwoEdgeLocalModeReservedVariableEqualityPayloadBoundary",
            left: "z",
            operator: "=",
            right: "z",
            recovered_label: false,
        };
        let reserve = || {
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("OuterTwoEdgeModeEquality"),
            )]
        };
        let exact_modes = || {
            vec![
                mode_definition("BaseTwoEdgeModeEquality", ReserveTypeShape::Builtin("set")),
                mode_definition(
                    "MiddleTwoEdgeModeEquality",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeModeEquality"),
                ),
                mode_definition(
                    "OuterTwoEdgeModeEquality",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeModeEquality"),
                ),
            ]
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
        let payload = extract_source_two_edge_local_mode_reserved_variable_equality(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact two-edge local-mode equality should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 3);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(
            payload.reserve.bridge.bindings()[0].type_spelling,
            "OuterTwoEdgeModeEquality"
        );
        assert_eq!(payload.left_lookup_ordinal, 1);
        assert_eq!(payload.right_lookup_ordinal, 2);

        let output = source_two_edge_local_mode_reserved_variable_equality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact two-edge local-mode equality should reach TermFormulaChecker");
        assert_source_reserved_variable_formula_output(&output)
            .expect("two-edge local-mode equality invariants should hold");
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
            assert_eq!(input.spelling, "OuterTwoEdgeModeEquality");
            assert!(matches!(input.head, TypeHeadInput::Symbol(_)));
        }
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("BaseTwoEdgeModeEquality")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("base mode terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("one normalized terminal type should exist");
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);

        let mut invalid_output = source_two_edge_local_mode_reserved_variable_equality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce a second checker output");
        invalid_output
            .payload
            .reserve
            .mode_expansions
            .retain(|symbol, _| {
                source_mode_symbol_spelling(symbol) != Some("BaseTwoEdgeModeEquality")
            });
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&invalid_output),
            vec![
                TYPE_ELABORATION_TWO_EDGE_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY
                    .to_owned()
            ]
        );

        let near_miss_modes = [
            vec![mode_definition(
                "OuterTwoEdgeModeEquality",
                ReserveTypeShape::Builtin("set"),
            )],
            vec![
                mode_definition("BaseTwoEdgeModeEquality", ReserveTypeShape::Builtin("set")),
                mode_definition(
                    "OuterTwoEdgeModeEquality",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeModeEquality"),
                ),
            ],
            vec![
                mode_definition("BaseTwoEdgeModeEquality", ReserveTypeShape::Builtin("set")),
                mode_definition(
                    "OuterTwoEdgeModeEquality",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeModeEquality"),
                ),
            ],
            vec![
                mode_definition("BaseTwoEdgeModeEquality", ReserveTypeShape::Builtin("set")),
                mode_definition(
                    "MiddleTwoEdgeModeEquality",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeModeEquality"),
                ),
                mode_definition(
                    "MiddleTwoEdgeModeEquality",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeModeEquality"),
                ),
                mode_definition(
                    "OuterTwoEdgeModeEquality",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeModeEquality"),
                ),
            ],
            vec![
                mode_definition(
                    "BaseTwoEdgeModeEquality",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition(
                    "MiddleTwoEdgeModeEquality",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeModeEquality"),
                ),
                mode_definition(
                    "OuterTwoEdgeModeEquality",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeModeEquality"),
                ),
            ],
            vec![
                mode_definition("BaseTwoEdgeModeEquality", ReserveTypeShape::AttributedSet),
                mode_definition(
                    "MiddleTwoEdgeModeEquality",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeModeEquality"),
                ),
                mode_definition(
                    "OuterTwoEdgeModeEquality",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeModeEquality"),
                ),
            ],
            vec![
                contextual_mode_definition(
                    "BaseTwoEdgeModeEquality",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition(
                    "MiddleTwoEdgeModeEquality",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeModeEquality"),
                ),
                mode_definition(
                    "OuterTwoEdgeModeEquality",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeModeEquality"),
                ),
            ],
            vec![
                mode_definition("BaseTwoEdgeModeEquality", ReserveTypeShape::Builtin("set")),
                parameterized_mode_definition(
                    "MiddleTwoEdgeModeEquality",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeModeEquality"),
                ),
                mode_definition(
                    "OuterTwoEdgeModeEquality",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeModeEquality"),
                ),
            ],
            vec![
                mode_definition("BaseTwoEdgeModeEquality", ReserveTypeShape::Builtin("set")),
                recovered_mode_definition(
                    "MiddleTwoEdgeModeEquality",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeModeEquality"),
                ),
                mode_definition(
                    "OuterTwoEdgeModeEquality",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeModeEquality"),
                ),
            ],
            vec![
                mode_definition_with_label(
                    "BaseTwoEdgeModeEquality",
                    "OtherDef",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition(
                    "MiddleTwoEdgeModeEquality",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeModeEquality"),
                ),
                mode_definition(
                    "OuterTwoEdgeModeEquality",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeModeEquality"),
                ),
            ],
            vec![
                mode_definition("BaseTwoEdgeModeEquality", ReserveTypeShape::Builtin("set")),
                mode_definition_with_label(
                    "MiddleTwoEdgeModeEquality",
                    "OtherDef",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeModeEquality"),
                ),
                mode_definition(
                    "OuterTwoEdgeModeEquality",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeModeEquality"),
                ),
            ],
            vec![
                mode_definition("BaseTwoEdgeModeEquality", ReserveTypeShape::Builtin("set")),
                mode_definition(
                    "MiddleTwoEdgeModeEquality",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeModeEquality"),
                ),
                mode_definition_with_label(
                    "OuterTwoEdgeModeEquality",
                    "OtherDef",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeModeEquality"),
                ),
            ],
            vec![
                mode_definition("BaseTwoEdgeModeEquality", ReserveTypeShape::Builtin("set")),
                mode_definition(
                    "OuterTwoEdgeModeEquality",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeModeEquality"),
                ),
                mode_definition(
                    "MiddleTwoEdgeModeEquality",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeModeEquality"),
                ),
            ],
            vec![
                mode_definition("BaseTwoEdgeModeEquality", ReserveTypeShape::Builtin("set")),
                mode_definition(
                    "MiddleTwoEdgeModeEquality",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeModeEquality"),
                ),
                mode_definition(
                    "OuterTwoEdgeModeEquality",
                    ReserveTypeShape::QualifiedSymbolWithArgs("MiddleTwoEdgeModeEquality"),
                ),
            ],
            vec![
                mode_definition("BaseTwoEdgeModeEquality", ReserveTypeShape::Builtin("set")),
                mode_definition(
                    "MiddleTwoEdgeModeEquality",
                    ReserveTypeShape::QualifiedSymbol("OuterTwoEdgeModeEquality"),
                ),
                mode_definition(
                    "OuterTwoEdgeModeEquality",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeModeEquality"),
                ),
            ],
            vec![
                mode_definition("BaseTwoEdgeModeEquality", ReserveTypeShape::Builtin("set")),
                mode_definition(
                    "ExtraTwoEdgeModeEquality",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeModeEquality"),
                ),
                mode_definition(
                    "MiddleTwoEdgeModeEquality",
                    ReserveTypeShape::QualifiedSymbol("ExtraTwoEdgeModeEquality"),
                ),
                mode_definition(
                    "OuterTwoEdgeModeEquality",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeModeEquality"),
                ),
            ],
        ];
        for modes in near_miss_modes {
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
        }
        for near_miss in [
            modes_then_empty_definition_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(
                    vec!["z"],
                    ReserveTypeShape::QualifiedSymbolWithArgs("OuterTwoEdgeModeEquality"),
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
                    operator: "<>",
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
    fn source_two_edge_local_mode_reserved_variable_inequality_consumes_three_expansions() {
        let source_id = source_id(136);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("two_edge_local_mode_reserved_variable_inequality"),
        );
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseTwoEdgeModeInequality", SymbolKind::Mode),
                ("MiddleTwoEdgeModeInequality", SymbolKind::Mode),
                ("OuterTwoEdgeModeInequality", SymbolKind::Mode),
                ("ExtraTwoEdgeModeInequality", SymbolKind::Mode),
            ],
        );
        let theorem = IdentifierBinaryTheoremSpec {
            status: None,
            label: "TwoEdgeLocalModeReservedVariableInequalityPayloadBoundary",
            left: "z",
            operator: "<>",
            right: "z",
            recovered_label: false,
        };
        let reserve = || {
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("OuterTwoEdgeModeInequality"),
            )]
        };
        let exact_modes = || {
            vec![
                mode_definition(
                    "BaseTwoEdgeModeInequality",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition(
                    "MiddleTwoEdgeModeInequality",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeModeInequality"),
                ),
                mode_definition(
                    "OuterTwoEdgeModeInequality",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeModeInequality"),
                ),
            ]
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
        let payload = extract_source_two_edge_local_mode_reserved_variable_inequality(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact two-edge local-mode inequality should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 3);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(
            payload.reserve.bridge.bindings()[0].type_spelling,
            "OuterTwoEdgeModeInequality"
        );
        assert_eq!(payload.left_lookup_ordinal, 1);
        assert_eq!(payload.right_lookup_ordinal, 2);

        let output = source_two_edge_local_mode_reserved_variable_inequality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact two-edge local-mode inequality should reach TermFormulaChecker");
        assert_source_reserved_variable_formula_output(&output)
            .expect("two-edge local-mode inequality invariants should hold");
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
            assert_eq!(input.spelling, "OuterTwoEdgeModeInequality");
            assert!(matches!(input.head, TypeHeadInput::Symbol(_)));
        }
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("BaseTwoEdgeModeInequality")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("base mode terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("one normalized terminal type should exist");
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);

        let mut invalid_output = source_two_edge_local_mode_reserved_variable_inequality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce a second checker output");
        invalid_output
            .payload
            .reserve
            .mode_expansions
            .retain(|symbol, _| {
                source_mode_symbol_spelling(symbol) != Some("BaseTwoEdgeModeInequality")
            });
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&invalid_output),
            vec![
                TYPE_ELABORATION_TWO_EDGE_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY
                    .to_owned()
            ]
        );

        let near_miss_modes = [
            vec![mode_definition(
                "OuterTwoEdgeModeInequality",
                ReserveTypeShape::Builtin("set"),
            )],
            vec![
                mode_definition(
                    "BaseTwoEdgeModeInequality",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition(
                    "OuterTwoEdgeModeInequality",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeModeInequality"),
                ),
            ],
            vec![
                mode_definition(
                    "BaseTwoEdgeModeInequality",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition(
                    "OuterTwoEdgeModeInequality",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeModeInequality"),
                ),
            ],
            vec![
                mode_definition(
                    "BaseTwoEdgeModeInequality",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition(
                    "MiddleTwoEdgeModeInequality",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeModeInequality"),
                ),
                mode_definition(
                    "MiddleTwoEdgeModeInequality",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeModeInequality"),
                ),
                mode_definition(
                    "OuterTwoEdgeModeInequality",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeModeInequality"),
                ),
            ],
            vec![
                mode_definition(
                    "BaseTwoEdgeModeInequality",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition(
                    "MiddleTwoEdgeModeInequality",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeModeInequality"),
                ),
                mode_definition(
                    "OuterTwoEdgeModeInequality",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeModeInequality"),
                ),
            ],
            vec![
                mode_definition("BaseTwoEdgeModeInequality", ReserveTypeShape::AttributedSet),
                mode_definition(
                    "MiddleTwoEdgeModeInequality",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeModeInequality"),
                ),
                mode_definition(
                    "OuterTwoEdgeModeInequality",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeModeInequality"),
                ),
            ],
            vec![
                contextual_mode_definition(
                    "BaseTwoEdgeModeInequality",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition(
                    "MiddleTwoEdgeModeInequality",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeModeInequality"),
                ),
                mode_definition(
                    "OuterTwoEdgeModeInequality",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeModeInequality"),
                ),
            ],
            vec![
                mode_definition(
                    "BaseTwoEdgeModeInequality",
                    ReserveTypeShape::Builtin("set"),
                ),
                parameterized_mode_definition(
                    "MiddleTwoEdgeModeInequality",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeModeInequality"),
                ),
                mode_definition(
                    "OuterTwoEdgeModeInequality",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeModeInequality"),
                ),
            ],
            vec![
                mode_definition(
                    "BaseTwoEdgeModeInequality",
                    ReserveTypeShape::Builtin("set"),
                ),
                recovered_mode_definition(
                    "MiddleTwoEdgeModeInequality",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeModeInequality"),
                ),
                mode_definition(
                    "OuterTwoEdgeModeInequality",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeModeInequality"),
                ),
            ],
            vec![
                mode_definition_with_label(
                    "BaseTwoEdgeModeInequality",
                    "OtherDef",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition(
                    "MiddleTwoEdgeModeInequality",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeModeInequality"),
                ),
                mode_definition(
                    "OuterTwoEdgeModeInequality",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeModeInequality"),
                ),
            ],
            vec![
                mode_definition(
                    "BaseTwoEdgeModeInequality",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition_with_label(
                    "MiddleTwoEdgeModeInequality",
                    "OtherDef",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeModeInequality"),
                ),
                mode_definition(
                    "OuterTwoEdgeModeInequality",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeModeInequality"),
                ),
            ],
            vec![
                mode_definition(
                    "BaseTwoEdgeModeInequality",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition(
                    "MiddleTwoEdgeModeInequality",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeModeInequality"),
                ),
                mode_definition_with_label(
                    "OuterTwoEdgeModeInequality",
                    "OtherDef",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeModeInequality"),
                ),
            ],
            vec![
                mode_definition(
                    "BaseTwoEdgeModeInequality",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition(
                    "OuterTwoEdgeModeInequality",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeModeInequality"),
                ),
                mode_definition(
                    "MiddleTwoEdgeModeInequality",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeModeInequality"),
                ),
            ],
            vec![
                mode_definition(
                    "BaseTwoEdgeModeInequality",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition(
                    "MiddleTwoEdgeModeInequality",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeModeInequality"),
                ),
                mode_definition(
                    "OuterTwoEdgeModeInequality",
                    ReserveTypeShape::QualifiedSymbolWithArgs("MiddleTwoEdgeModeInequality"),
                ),
            ],
            vec![
                mode_definition(
                    "BaseTwoEdgeModeInequality",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition(
                    "MiddleTwoEdgeModeInequality",
                    ReserveTypeShape::QualifiedSymbol("OuterTwoEdgeModeInequality"),
                ),
                mode_definition(
                    "OuterTwoEdgeModeInequality",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeModeInequality"),
                ),
            ],
            vec![
                mode_definition(
                    "BaseTwoEdgeModeInequality",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition(
                    "ExtraTwoEdgeModeInequality",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeModeInequality"),
                ),
                mode_definition(
                    "MiddleTwoEdgeModeInequality",
                    ReserveTypeShape::QualifiedSymbol("ExtraTwoEdgeModeInequality"),
                ),
                mode_definition(
                    "OuterTwoEdgeModeInequality",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeModeInequality"),
                ),
            ],
        ];
        for modes in near_miss_modes {
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
        }
        for near_miss in [
            modes_then_empty_definition_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(
                    vec!["z"],
                    ReserveTypeShape::QualifiedSymbolWithArgs("OuterTwoEdgeModeInequality"),
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
                    operator: "=",
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
    fn source_two_edge_local_object_mode_reserved_variable_inequality_consumes_three_expansions() {
        let source_id = source_id(137);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("two_edge_local_object_mode_reserved_variable_inequality"),
        );
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseTwoEdgeObjectModeInequality", SymbolKind::Mode),
                ("MiddleTwoEdgeObjectModeInequality", SymbolKind::Mode),
                ("OuterTwoEdgeObjectModeInequality", SymbolKind::Mode),
                ("ExtraTwoEdgeObjectModeInequality", SymbolKind::Mode),
            ],
        );
        let theorem = IdentifierBinaryTheoremSpec {
            status: None,
            label: "TwoEdgeLocalObjectModeReservedVariableInequalityPayloadBoundary",
            left: "z",
            operator: "<>",
            right: "z",
            recovered_label: false,
        };
        let reserve = || {
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("OuterTwoEdgeObjectModeInequality"),
            )]
        };
        let exact_modes = || {
            vec![
                mode_definition(
                    "BaseTwoEdgeObjectModeInequality",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition(
                    "MiddleTwoEdgeObjectModeInequality",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeObjectModeInequality"),
                ),
                mode_definition(
                    "OuterTwoEdgeObjectModeInequality",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeObjectModeInequality"),
                ),
            ]
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
        let payload = extract_source_two_edge_local_object_mode_reserved_variable_inequality(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact two-edge local-object-mode inequality should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 3);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(
            payload.reserve.bridge.bindings()[0].type_spelling,
            "OuterTwoEdgeObjectModeInequality"
        );
        assert_eq!(payload.left_lookup_ordinal, 1);
        assert_eq!(payload.right_lookup_ordinal, 2);

        let output = source_two_edge_local_object_mode_reserved_variable_inequality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact two-edge local-object-mode inequality should reach TermFormulaChecker");
        assert_source_reserved_variable_formula_output(&output)
            .expect("two-edge local-object-mode inequality invariants should hold");
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
            assert_eq!(input.spelling, "OuterTwoEdgeObjectModeInequality");
            assert!(matches!(input.head, TypeHeadInput::Symbol(_)));
        }
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("BaseTwoEdgeObjectModeInequality")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("base mode terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("one normalized terminal type should exist");
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);

        let mut invalid_output =
            source_two_edge_local_object_mode_reserved_variable_inequality_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce a second checker output");
        invalid_output
            .payload
            .reserve
            .mode_expansions
            .retain(|symbol, _| {
                source_mode_symbol_spelling(symbol) != Some("BaseTwoEdgeObjectModeInequality")
            });
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&invalid_output),
            vec![
                TYPE_ELABORATION_TWO_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY
                    .to_owned()
            ]
        );

        let near_miss_modes = [
            vec![mode_definition(
                "OuterTwoEdgeObjectModeInequality",
                ReserveTypeShape::Builtin("object"),
            )],
            vec![
                mode_definition(
                    "BaseTwoEdgeObjectModeInequality",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition(
                    "OuterTwoEdgeObjectModeInequality",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeObjectModeInequality"),
                ),
            ],
            vec![
                mode_definition(
                    "BaseTwoEdgeObjectModeInequality",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition(
                    "OuterTwoEdgeObjectModeInequality",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeObjectModeInequality"),
                ),
            ],
            vec![
                mode_definition(
                    "BaseTwoEdgeObjectModeInequality",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition(
                    "MiddleTwoEdgeObjectModeInequality",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeObjectModeInequality"),
                ),
                mode_definition(
                    "MiddleTwoEdgeObjectModeInequality",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeObjectModeInequality"),
                ),
                mode_definition(
                    "OuterTwoEdgeObjectModeInequality",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeObjectModeInequality"),
                ),
            ],
            vec![
                mode_definition(
                    "BaseTwoEdgeObjectModeInequality",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition(
                    "MiddleTwoEdgeObjectModeInequality",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeObjectModeInequality"),
                ),
                mode_definition(
                    "OuterTwoEdgeObjectModeInequality",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeObjectModeInequality"),
                ),
            ],
            vec![
                mode_definition(
                    "BaseTwoEdgeObjectModeInequality",
                    ReserveTypeShape::AttributedObject,
                ),
                mode_definition(
                    "MiddleTwoEdgeObjectModeInequality",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeObjectModeInequality"),
                ),
                mode_definition(
                    "OuterTwoEdgeObjectModeInequality",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeObjectModeInequality"),
                ),
            ],
            vec![
                contextual_mode_definition(
                    "BaseTwoEdgeObjectModeInequality",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition(
                    "MiddleTwoEdgeObjectModeInequality",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeObjectModeInequality"),
                ),
                mode_definition(
                    "OuterTwoEdgeObjectModeInequality",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeObjectModeInequality"),
                ),
            ],
            vec![
                mode_definition(
                    "BaseTwoEdgeObjectModeInequality",
                    ReserveTypeShape::Builtin("object"),
                ),
                parameterized_mode_definition(
                    "MiddleTwoEdgeObjectModeInequality",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeObjectModeInequality"),
                ),
                mode_definition(
                    "OuterTwoEdgeObjectModeInequality",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeObjectModeInequality"),
                ),
            ],
            vec![
                mode_definition(
                    "BaseTwoEdgeObjectModeInequality",
                    ReserveTypeShape::Builtin("object"),
                ),
                recovered_mode_definition(
                    "MiddleTwoEdgeObjectModeInequality",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeObjectModeInequality"),
                ),
                mode_definition(
                    "OuterTwoEdgeObjectModeInequality",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeObjectModeInequality"),
                ),
            ],
            vec![
                mode_definition_with_label(
                    "BaseTwoEdgeObjectModeInequality",
                    "OtherDef",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition(
                    "MiddleTwoEdgeObjectModeInequality",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeObjectModeInequality"),
                ),
                mode_definition(
                    "OuterTwoEdgeObjectModeInequality",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeObjectModeInequality"),
                ),
            ],
            vec![
                mode_definition(
                    "BaseTwoEdgeObjectModeInequality",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition_with_label(
                    "MiddleTwoEdgeObjectModeInequality",
                    "OtherDef",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeObjectModeInequality"),
                ),
                mode_definition(
                    "OuterTwoEdgeObjectModeInequality",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeObjectModeInequality"),
                ),
            ],
            vec![
                mode_definition(
                    "BaseTwoEdgeObjectModeInequality",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition(
                    "MiddleTwoEdgeObjectModeInequality",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeObjectModeInequality"),
                ),
                mode_definition_with_label(
                    "OuterTwoEdgeObjectModeInequality",
                    "OtherDef",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeObjectModeInequality"),
                ),
            ],
            vec![
                mode_definition(
                    "BaseTwoEdgeObjectModeInequality",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition(
                    "OuterTwoEdgeObjectModeInequality",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeObjectModeInequality"),
                ),
                mode_definition(
                    "MiddleTwoEdgeObjectModeInequality",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeObjectModeInequality"),
                ),
            ],
            vec![
                mode_definition(
                    "BaseTwoEdgeObjectModeInequality",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition(
                    "MiddleTwoEdgeObjectModeInequality",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeObjectModeInequality"),
                ),
                mode_definition(
                    "OuterTwoEdgeObjectModeInequality",
                    ReserveTypeShape::QualifiedSymbolWithArgs("MiddleTwoEdgeObjectModeInequality"),
                ),
            ],
            vec![
                mode_definition(
                    "BaseTwoEdgeObjectModeInequality",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition(
                    "MiddleTwoEdgeObjectModeInequality",
                    ReserveTypeShape::QualifiedSymbol("OuterTwoEdgeObjectModeInequality"),
                ),
                mode_definition(
                    "OuterTwoEdgeObjectModeInequality",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeObjectModeInequality"),
                ),
            ],
            vec![
                mode_definition(
                    "BaseTwoEdgeObjectModeInequality",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition(
                    "ExtraTwoEdgeObjectModeInequality",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeObjectModeInequality"),
                ),
                mode_definition(
                    "MiddleTwoEdgeObjectModeInequality",
                    ReserveTypeShape::QualifiedSymbol("ExtraTwoEdgeObjectModeInequality"),
                ),
                mode_definition(
                    "OuterTwoEdgeObjectModeInequality",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeObjectModeInequality"),
                ),
            ],
        ];
        for modes in near_miss_modes {
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
        }
        for near_miss in [
            modes_then_empty_definition_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(
                    vec!["z"],
                    ReserveTypeShape::QualifiedSymbolWithArgs("OuterTwoEdgeObjectModeInequality"),
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
                    operator: "=",
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
    fn source_two_edge_local_object_mode_reserved_variable_equality_consumes_three_expansions() {
        let source_id = source_id(135);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("two_edge_local_object_mode_reserved_variable_equality"),
        );
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseTwoEdgeObjectModeEquality", SymbolKind::Mode),
                ("MiddleTwoEdgeObjectModeEquality", SymbolKind::Mode),
                ("OuterTwoEdgeObjectModeEquality", SymbolKind::Mode),
                ("ExtraTwoEdgeObjectModeEquality", SymbolKind::Mode),
            ],
        );
        let theorem = IdentifierBinaryTheoremSpec {
            status: None,
            label: "TwoEdgeLocalObjectModeReservedVariableEqualityPayloadBoundary",
            left: "z",
            operator: "=",
            right: "z",
            recovered_label: false,
        };
        let reserve = || {
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("OuterTwoEdgeObjectModeEquality"),
            )]
        };
        let exact_modes = || {
            vec![
                mode_definition(
                    "BaseTwoEdgeObjectModeEquality",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition(
                    "MiddleTwoEdgeObjectModeEquality",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeObjectModeEquality"),
                ),
                mode_definition(
                    "OuterTwoEdgeObjectModeEquality",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeObjectModeEquality"),
                ),
            ]
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
        let payload = extract_source_two_edge_local_object_mode_reserved_variable_equality(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact two-edge local-object-mode equality should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 3);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(
            payload.reserve.bridge.bindings()[0].type_spelling,
            "OuterTwoEdgeObjectModeEquality"
        );
        assert_eq!(payload.left_lookup_ordinal, 1);
        assert_eq!(payload.right_lookup_ordinal, 2);

        let output = source_two_edge_local_object_mode_reserved_variable_equality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact two-edge local-object-mode equality should reach TermFormulaChecker");
        assert_source_reserved_variable_formula_output(&output)
            .expect("two-edge local-object-mode equality invariants should hold");
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
            assert_eq!(input.spelling, "OuterTwoEdgeObjectModeEquality");
            assert!(matches!(input.head, TypeHeadInput::Symbol(_)));
        }
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("BaseTwoEdgeObjectModeEquality")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("base mode terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("one normalized terminal type should exist");
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);

        let mut invalid_output =
            source_two_edge_local_object_mode_reserved_variable_equality_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce a second checker output");
        invalid_output
            .payload
            .reserve
            .mode_expansions
            .retain(|symbol, _| {
                source_mode_symbol_spelling(symbol) != Some("BaseTwoEdgeObjectModeEquality")
            });
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&invalid_output),
            vec![
                TYPE_ELABORATION_TWO_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY
                    .to_owned()
            ]
        );

        let near_miss_modes = [
            vec![mode_definition(
                "OuterTwoEdgeObjectModeEquality",
                ReserveTypeShape::Builtin("object"),
            )],
            vec![
                mode_definition(
                    "BaseTwoEdgeObjectModeEquality",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition(
                    "OuterTwoEdgeObjectModeEquality",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeObjectModeEquality"),
                ),
            ],
            vec![
                mode_definition(
                    "BaseTwoEdgeObjectModeEquality",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition(
                    "OuterTwoEdgeObjectModeEquality",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeObjectModeEquality"),
                ),
            ],
            vec![
                mode_definition(
                    "BaseTwoEdgeObjectModeEquality",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition(
                    "MiddleTwoEdgeObjectModeEquality",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeObjectModeEquality"),
                ),
                mode_definition(
                    "MiddleTwoEdgeObjectModeEquality",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeObjectModeEquality"),
                ),
                mode_definition(
                    "OuterTwoEdgeObjectModeEquality",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeObjectModeEquality"),
                ),
            ],
            vec![
                mode_definition(
                    "BaseTwoEdgeObjectModeEquality",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition(
                    "MiddleTwoEdgeObjectModeEquality",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeObjectModeEquality"),
                ),
                mode_definition(
                    "OuterTwoEdgeObjectModeEquality",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeObjectModeEquality"),
                ),
            ],
            vec![
                mode_definition(
                    "BaseTwoEdgeObjectModeEquality",
                    ReserveTypeShape::AttributedObject,
                ),
                mode_definition(
                    "MiddleTwoEdgeObjectModeEquality",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeObjectModeEquality"),
                ),
                mode_definition(
                    "OuterTwoEdgeObjectModeEquality",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeObjectModeEquality"),
                ),
            ],
            vec![
                contextual_mode_definition(
                    "BaseTwoEdgeObjectModeEquality",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition(
                    "MiddleTwoEdgeObjectModeEquality",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeObjectModeEquality"),
                ),
                mode_definition(
                    "OuterTwoEdgeObjectModeEquality",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeObjectModeEquality"),
                ),
            ],
            vec![
                mode_definition(
                    "BaseTwoEdgeObjectModeEquality",
                    ReserveTypeShape::Builtin("object"),
                ),
                parameterized_mode_definition(
                    "MiddleTwoEdgeObjectModeEquality",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeObjectModeEquality"),
                ),
                mode_definition(
                    "OuterTwoEdgeObjectModeEquality",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeObjectModeEquality"),
                ),
            ],
            vec![
                mode_definition(
                    "BaseTwoEdgeObjectModeEquality",
                    ReserveTypeShape::Builtin("object"),
                ),
                recovered_mode_definition(
                    "MiddleTwoEdgeObjectModeEquality",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeObjectModeEquality"),
                ),
                mode_definition(
                    "OuterTwoEdgeObjectModeEquality",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeObjectModeEquality"),
                ),
            ],
            vec![
                mode_definition_with_label(
                    "BaseTwoEdgeObjectModeEquality",
                    "OtherDef",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition(
                    "MiddleTwoEdgeObjectModeEquality",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeObjectModeEquality"),
                ),
                mode_definition(
                    "OuterTwoEdgeObjectModeEquality",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeObjectModeEquality"),
                ),
            ],
            vec![
                mode_definition(
                    "BaseTwoEdgeObjectModeEquality",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition_with_label(
                    "MiddleTwoEdgeObjectModeEquality",
                    "OtherDef",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeObjectModeEquality"),
                ),
                mode_definition(
                    "OuterTwoEdgeObjectModeEquality",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeObjectModeEquality"),
                ),
            ],
            vec![
                mode_definition(
                    "BaseTwoEdgeObjectModeEquality",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition(
                    "MiddleTwoEdgeObjectModeEquality",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeObjectModeEquality"),
                ),
                mode_definition_with_label(
                    "OuterTwoEdgeObjectModeEquality",
                    "OtherDef",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeObjectModeEquality"),
                ),
            ],
            vec![
                mode_definition(
                    "BaseTwoEdgeObjectModeEquality",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition(
                    "OuterTwoEdgeObjectModeEquality",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeObjectModeEquality"),
                ),
                mode_definition(
                    "MiddleTwoEdgeObjectModeEquality",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeObjectModeEquality"),
                ),
            ],
            vec![
                mode_definition(
                    "BaseTwoEdgeObjectModeEquality",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition(
                    "MiddleTwoEdgeObjectModeEquality",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeObjectModeEquality"),
                ),
                mode_definition(
                    "OuterTwoEdgeObjectModeEquality",
                    ReserveTypeShape::QualifiedSymbolWithArgs("MiddleTwoEdgeObjectModeEquality"),
                ),
            ],
            vec![
                mode_definition(
                    "BaseTwoEdgeObjectModeEquality",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition(
                    "MiddleTwoEdgeObjectModeEquality",
                    ReserveTypeShape::QualifiedSymbol("OuterTwoEdgeObjectModeEquality"),
                ),
                mode_definition(
                    "OuterTwoEdgeObjectModeEquality",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeObjectModeEquality"),
                ),
            ],
            vec![
                mode_definition(
                    "BaseTwoEdgeObjectModeEquality",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition(
                    "ExtraTwoEdgeObjectModeEquality",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeObjectModeEquality"),
                ),
                mode_definition(
                    "MiddleTwoEdgeObjectModeEquality",
                    ReserveTypeShape::QualifiedSymbol("ExtraTwoEdgeObjectModeEquality"),
                ),
                mode_definition(
                    "OuterTwoEdgeObjectModeEquality",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeObjectModeEquality"),
                ),
            ],
        ];
        for modes in near_miss_modes {
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
        }
        for near_miss in [
            modes_then_empty_definition_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(
                    vec!["z"],
                    ReserveTypeShape::QualifiedSymbolWithArgs("OuterTwoEdgeObjectModeEquality"),
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
                    operator: "<>",
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
    fn source_chained_local_mode_reserved_variable_inequality_consumes_both_expansions() {
        let source_id = source_id(132);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("chained_local_mode_reserved_variable_inequality"),
        );
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseModeInequality", SymbolKind::Mode),
                ("ChainModeInequality", SymbolKind::Mode),
                ("InnerModeInequality", SymbolKind::Mode),
            ],
        );
        let theorem = IdentifierBinaryTheoremSpec {
            status: None,
            label: "ChainedLocalModeReservedVariableInequalityPayloadBoundary",
            left: "x",
            operator: "<>",
            right: "x",
            recovered_label: false,
        };
        let reserve = || {
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("ChainModeInequality"),
            )]
        };
        let exact_modes = || {
            vec![
                mode_definition("BaseModeInequality", ReserveTypeShape::Builtin("set")),
                mode_definition(
                    "ChainModeInequality",
                    ReserveTypeShape::QualifiedSymbol("BaseModeInequality"),
                ),
            ]
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
        let payload = extract_source_chained_local_mode_reserved_variable_inequality(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact chained local-mode inequality should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 2);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(
            payload.reserve.bridge.bindings()[0].type_spelling,
            "ChainModeInequality"
        );
        assert_eq!(payload.left_lookup_ordinal, 1);
        assert_eq!(payload.right_lookup_ordinal, 2);

        let output = source_chained_local_mode_reserved_variable_inequality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact chained local-mode inequality should reach TermFormulaChecker");
        assert_source_reserved_variable_formula_output(&output)
            .expect("chained local-mode inequality invariants should hold");
        assert_eq!(output.left_binding, BindingId::new(0));
        assert_eq!(output.right_binding, BindingId::new(0));
        for input in [
            &output.left_result_input,
            &output.right_result_input,
            output.left_expected_input.as_ref().expect("left expected"),
            output
                .right_expected_input
                .as_ref()
                .expect("right expected"),
        ] {
            assert_eq!(input.spelling, "ChainModeInequality");
            assert!(matches!(input.head, TypeHeadInput::Symbol(_)));
        }
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("BaseModeInequality"))
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
        let formula = output
            .term_formula
            .formulas()
            .iter()
            .map(|(_, formula)| formula)
            .next()
            .expect("inequality formula should be checked");
        assert_eq!(formula.kind, FormulaKind::Inequality);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());

        let mut invalid_output = source_chained_local_mode_reserved_variable_inequality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce a corruptible checker output");
        invalid_output
            .payload
            .reserve
            .mode_expansions
            .retain(|symbol, _| source_mode_symbol_spelling(symbol) != Some("BaseModeInequality"));
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&invalid_output),
            vec![
                TYPE_ELABORATION_CHAINED_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY
                    .to_owned()
            ]
        );

        let near_miss_modes = [
            vec![mode_definition(
                "ChainModeInequality",
                ReserveTypeShape::QualifiedSymbol("BaseModeInequality"),
            )],
            vec![
                mode_definition("BaseModeInequality", ReserveTypeShape::Builtin("set")),
                mode_definition("BaseModeInequality", ReserveTypeShape::Builtin("set")),
                mode_definition(
                    "ChainModeInequality",
                    ReserveTypeShape::QualifiedSymbol("BaseModeInequality"),
                ),
            ],
            vec![
                mode_definition("BaseModeInequality", ReserveTypeShape::Builtin("object")),
                mode_definition(
                    "ChainModeInequality",
                    ReserveTypeShape::QualifiedSymbol("BaseModeInequality"),
                ),
            ],
            vec![
                mode_definition("BaseModeInequality", ReserveTypeShape::AttributedSet),
                mode_definition(
                    "ChainModeInequality",
                    ReserveTypeShape::QualifiedSymbol("BaseModeInequality"),
                ),
            ],
            vec![
                contextual_mode_definition("BaseModeInequality", ReserveTypeShape::Builtin("set")),
                mode_definition(
                    "ChainModeInequality",
                    ReserveTypeShape::QualifiedSymbol("BaseModeInequality"),
                ),
            ],
            vec![
                mode_definition("BaseModeInequality", ReserveTypeShape::Builtin("set")),
                parameterized_mode_definition(
                    "ChainModeInequality",
                    ReserveTypeShape::QualifiedSymbol("BaseModeInequality"),
                ),
            ],
            vec![
                recovered_mode_definition("BaseModeInequality", ReserveTypeShape::Builtin("set")),
                mode_definition(
                    "ChainModeInequality",
                    ReserveTypeShape::QualifiedSymbol("BaseModeInequality"),
                ),
            ],
            vec![
                mode_definition_with_label(
                    "BaseModeInequality",
                    "OtherDef",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition(
                    "ChainModeInequality",
                    ReserveTypeShape::QualifiedSymbol("BaseModeInequality"),
                ),
            ],
            vec![
                mode_definition("BaseModeInequality", ReserveTypeShape::Builtin("set")),
                mode_definition_with_label(
                    "ChainModeInequality",
                    "OtherDef",
                    ReserveTypeShape::QualifiedSymbol("BaseModeInequality"),
                ),
            ],
            vec![
                mode_definition(
                    "ChainModeInequality",
                    ReserveTypeShape::QualifiedSymbol("BaseModeInequality"),
                ),
                mode_definition("BaseModeInequality", ReserveTypeShape::Builtin("set")),
            ],
            vec![
                mode_definition("BaseModeInequality", ReserveTypeShape::Builtin("set")),
                mode_definition(
                    "ChainModeInequality",
                    ReserveTypeShape::QualifiedSymbolWithArgs("BaseModeInequality"),
                ),
            ],
            vec![
                mode_definition("BaseModeInequality", ReserveTypeShape::Builtin("set")),
                mode_definition("ChainModeInequality", ReserveTypeShape::Builtin("set")),
            ],
            vec![
                mode_definition(
                    "BaseModeInequality",
                    ReserveTypeShape::QualifiedSymbol("ChainModeInequality"),
                ),
                mode_definition(
                    "ChainModeInequality",
                    ReserveTypeShape::QualifiedSymbol("BaseModeInequality"),
                ),
            ],
            vec![
                mode_definition("BaseModeInequality", ReserveTypeShape::Builtin("set")),
                mode_definition(
                    "InnerModeInequality",
                    ReserveTypeShape::QualifiedSymbol("BaseModeInequality"),
                ),
                mode_definition(
                    "ChainModeInequality",
                    ReserveTypeShape::QualifiedSymbol("InnerModeInequality"),
                ),
            ],
        ];
        for modes in near_miss_modes {
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
        }
        for near_miss in [
            modes_then_empty_definition_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbolWithArgs("ChainModeInequality"),
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
                    operator: "=",
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
    fn source_chained_local_object_mode_reserved_variable_inequality_consumes_both_expansions() {
        let source_id = source_id(133);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("chained_local_object_mode_reserved_variable_inequality"),
        );
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseObjectModeInequality", SymbolKind::Mode),
                ("ChainObjectModeInequality", SymbolKind::Mode),
                ("InnerObjectModeInequality", SymbolKind::Mode),
            ],
        );
        let theorem = IdentifierBinaryTheoremSpec {
            status: None,
            label: "ChainedLocalObjectModeReservedVariableInequalityPayloadBoundary",
            left: "z",
            operator: "<>",
            right: "z",
            recovered_label: false,
        };
        let reserve = || {
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("ChainObjectModeInequality"),
            )]
        };
        let exact_modes = || {
            vec![
                mode_definition(
                    "BaseObjectModeInequality",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition(
                    "ChainObjectModeInequality",
                    ReserveTypeShape::QualifiedSymbol("BaseObjectModeInequality"),
                ),
            ]
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
        let payload = extract_source_chained_local_object_mode_reserved_variable_inequality(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact chained local-mode inequality should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 2);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(
            payload.reserve.bridge.bindings()[0].type_spelling,
            "ChainObjectModeInequality"
        );
        assert_eq!(payload.left_lookup_ordinal, 1);
        assert_eq!(payload.right_lookup_ordinal, 2);

        let output = source_chained_local_object_mode_reserved_variable_inequality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact chained local-mode inequality should reach TermFormulaChecker");
        assert_source_reserved_variable_formula_output(&output)
            .expect("chained local-mode inequality invariants should hold");
        assert_eq!(output.left_binding, BindingId::new(0));
        assert_eq!(output.right_binding, BindingId::new(0));
        for input in [
            &output.left_result_input,
            &output.right_result_input,
            output.left_expected_input.as_ref().expect("left expected"),
            output
                .right_expected_input
                .as_ref()
                .expect("right expected"),
        ] {
            assert_eq!(input.spelling, "ChainObjectModeInequality");
            assert!(matches!(input.head, TypeHeadInput::Symbol(_)));
        }
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("BaseObjectModeInequality")
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
        let formula = output
            .term_formula
            .formulas()
            .iter()
            .map(|(_, formula)| formula)
            .next()
            .expect("inequality formula should be checked");
        assert_eq!(formula.kind, FormulaKind::Inequality);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());

        let mut invalid_output =
            source_chained_local_object_mode_reserved_variable_inequality_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce a corruptible checker output");
        invalid_output
            .payload
            .reserve
            .mode_expansions
            .retain(|symbol, _| {
                source_mode_symbol_spelling(symbol) != Some("BaseObjectModeInequality")
            });
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&invalid_output),
            vec![
                TYPE_ELABORATION_CHAINED_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY
                    .to_owned()
            ]
        );

        let near_miss_modes = [
            vec![mode_definition(
                "ChainObjectModeInequality",
                ReserveTypeShape::QualifiedSymbol("BaseObjectModeInequality"),
            )],
            vec![
                mode_definition(
                    "BaseObjectModeInequality",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition(
                    "BaseObjectModeInequality",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition(
                    "ChainObjectModeInequality",
                    ReserveTypeShape::QualifiedSymbol("BaseObjectModeInequality"),
                ),
            ],
            vec![
                mode_definition("BaseObjectModeInequality", ReserveTypeShape::Builtin("set")),
                mode_definition(
                    "ChainObjectModeInequality",
                    ReserveTypeShape::QualifiedSymbol("BaseObjectModeInequality"),
                ),
            ],
            vec![
                mode_definition(
                    "BaseObjectModeInequality",
                    ReserveTypeShape::AttributedObject,
                ),
                mode_definition(
                    "ChainObjectModeInequality",
                    ReserveTypeShape::QualifiedSymbol("BaseObjectModeInequality"),
                ),
            ],
            vec![
                contextual_mode_definition(
                    "BaseObjectModeInequality",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition(
                    "ChainObjectModeInequality",
                    ReserveTypeShape::QualifiedSymbol("BaseObjectModeInequality"),
                ),
            ],
            vec![
                mode_definition(
                    "BaseObjectModeInequality",
                    ReserveTypeShape::Builtin("object"),
                ),
                parameterized_mode_definition(
                    "ChainObjectModeInequality",
                    ReserveTypeShape::QualifiedSymbol("BaseObjectModeInequality"),
                ),
            ],
            vec![
                recovered_mode_definition(
                    "BaseObjectModeInequality",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition(
                    "ChainObjectModeInequality",
                    ReserveTypeShape::QualifiedSymbol("BaseObjectModeInequality"),
                ),
            ],
            vec![
                mode_definition_with_label(
                    "BaseObjectModeInequality",
                    "OtherDef",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition(
                    "ChainObjectModeInequality",
                    ReserveTypeShape::QualifiedSymbol("BaseObjectModeInequality"),
                ),
            ],
            vec![
                mode_definition(
                    "BaseObjectModeInequality",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition_with_label(
                    "ChainObjectModeInequality",
                    "OtherDef",
                    ReserveTypeShape::QualifiedSymbol("BaseObjectModeInequality"),
                ),
            ],
            vec![
                mode_definition(
                    "ChainObjectModeInequality",
                    ReserveTypeShape::QualifiedSymbol("BaseObjectModeInequality"),
                ),
                mode_definition(
                    "BaseObjectModeInequality",
                    ReserveTypeShape::Builtin("object"),
                ),
            ],
            vec![
                mode_definition(
                    "BaseObjectModeInequality",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition(
                    "ChainObjectModeInequality",
                    ReserveTypeShape::QualifiedSymbolWithArgs("BaseObjectModeInequality"),
                ),
            ],
            vec![
                mode_definition(
                    "BaseObjectModeInequality",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition(
                    "ChainObjectModeInequality",
                    ReserveTypeShape::Builtin("object"),
                ),
            ],
            vec![
                mode_definition(
                    "BaseObjectModeInequality",
                    ReserveTypeShape::QualifiedSymbol("ChainObjectModeInequality"),
                ),
                mode_definition(
                    "ChainObjectModeInequality",
                    ReserveTypeShape::QualifiedSymbol("BaseObjectModeInequality"),
                ),
            ],
            vec![
                mode_definition(
                    "BaseObjectModeInequality",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition(
                    "InnerObjectModeInequality",
                    ReserveTypeShape::QualifiedSymbol("BaseObjectModeInequality"),
                ),
                mode_definition(
                    "ChainObjectModeInequality",
                    ReserveTypeShape::QualifiedSymbol("InnerObjectModeInequality"),
                ),
            ],
        ];
        for modes in near_miss_modes {
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
        }
        for near_miss in [
            modes_then_empty_definition_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(
                    vec!["z"],
                    ReserveTypeShape::QualifiedSymbolWithArgs("ChainObjectModeInequality"),
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
                    operator: "=",
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
    fn source_local_object_mode_reserved_variable_equality_consumes_real_expansion() {
        let source_id = source_id(128);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("local_object_mode_reserved_variable_equality"),
        );
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("LocalObjectMode", SymbolKind::Mode),
                ("BaseObjectMode", SymbolKind::Mode),
            ],
        );
        let theorem = IdentifierBinaryTheoremSpec {
            status: None,
            label: "LocalObjectModeReservedVariableEqualityPayloadBoundary",
            left: "x",
            operator: "=",
            right: "x",
            recovered_label: false,
        };
        let reserve = || {
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("LocalObjectMode"),
            )]
        };
        let exact = mode_then_reserve_identifier_binary_theorem_ast(
            source_id,
            [mode_definition(
                "LocalObjectMode",
                ReserveTypeShape::Builtin("object"),
            )],
            reserve(),
            theorem,
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        let payload = extract_source_local_object_mode_reserved_variable_equality(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact local object-mode equality should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 1);
        assert_eq!(
            payload.reserve.bridge.bindings()[0].type_spelling,
            "LocalObjectMode"
        );
        assert_eq!(payload.left_lookup_ordinal, 1);
        assert_eq!(payload.right_lookup_ordinal, 2);

        let output = source_local_object_mode_reserved_variable_equality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact local object-mode equality should reach TermFormulaChecker");
        assert_source_reserved_variable_formula_output(&output)
            .expect("local object-mode equality invariants should hold");
        assert_eq!(output.left_binding, BindingId::new(0));
        assert_eq!(output.right_binding, BindingId::new(0));
        for input in [
            &output.left_result_input,
            &output.right_result_input,
            output
                .left_expected_input
                .as_ref()
                .expect("left expected input"),
            output
                .right_expected_input
                .as_ref()
                .expect("right expected input"),
        ] {
            assert_eq!(input.spelling, "LocalObjectMode");
            assert!(matches!(input.head, TypeHeadInput::Symbol(_)));
        }
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .values()
            .next()
            .map(|expansion| &expansion.radix)
            .expect("object terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("one normalized object type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);

        let mut invalid_output = source_local_object_mode_reserved_variable_equality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce a second checker output");
        invalid_output.payload.reserve.mode_expansions.clear();
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&invalid_output),
            vec![
                TYPE_ELABORATION_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY
                    .to_owned()
            ]
        );

        let gap_modes = [
            mode_definition("LocalObjectMode", ReserveTypeShape::Builtin("set")),
            mode_definition("LocalObjectMode", ReserveTypeShape::AttributedObject),
            contextual_mode_definition("LocalObjectMode", ReserveTypeShape::Builtin("object")),
            parameterized_mode_definition("LocalObjectMode", ReserveTypeShape::Builtin("object")),
            recovered_mode_definition("LocalObjectMode", ReserveTypeShape::Builtin("object")),
        ];
        for mode in gap_modes {
            let near_miss = mode_then_reserve_identifier_binary_theorem_ast(
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
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode_definition(
                    "OtherMode",
                    ReserveTypeShape::Builtin("object"),
                )],
                reserve(),
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode_definition_with_label(
                    "LocalObjectMode",
                    "OtherDef",
                    ReserveTypeShape::Builtin("object"),
                )],
                reserve(),
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [
                    mode_definition("LocalObjectMode", ReserveTypeShape::Builtin("object")),
                    mode_definition("LocalObjectMode", ReserveTypeShape::Builtin("object")),
                ],
                reserve(),
                theorem,
            ),
            modes_then_empty_definition_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode_definition(
                    "LocalObjectMode",
                    ReserveTypeShape::Builtin("object"),
                )],
                reserve(),
                theorem,
            ),
            reserve_then_mode_identifier_binary_theorem_ast(
                source_id,
                reserve(),
                mode_definition("LocalObjectMode", ReserveTypeShape::Builtin("object")),
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode_definition(
                    "LocalObjectMode",
                    ReserveTypeShape::Builtin("object"),
                )],
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbolWithArgs("LocalObjectMode"),
                )],
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [
                    mode_definition("BaseObjectMode", ReserveTypeShape::Builtin("object")),
                    mode_definition(
                        "LocalObjectMode",
                        ReserveTypeShape::QualifiedSymbol("BaseObjectMode"),
                    ),
                ],
                reserve(),
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode_definition(
                    "LocalObjectMode",
                    ReserveTypeShape::Builtin("object"),
                )],
                reserve(),
                IdentifierBinaryTheoremSpec {
                    label: "OtherPayloadBoundary",
                    ..theorem
                },
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                [mode_definition(
                    "LocalObjectMode",
                    ReserveTypeShape::Builtin("object"),
                )],
                reserve(),
                IdentifierBinaryTheoremSpec {
                    operator: "<>",
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
    fn source_chained_local_object_mode_reserved_variable_equality_consumes_both_expansions() {
        let source_id = source_id(129);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("chained_local_object_mode_reserved_variable_equality"),
        );
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseObjectMode", SymbolKind::Mode),
                ("ChainObjectMode", SymbolKind::Mode),
                ("InnerObjectMode", SymbolKind::Mode),
            ],
        );
        let theorem = IdentifierBinaryTheoremSpec {
            status: None,
            label: "ChainedLocalObjectModeReservedVariableEqualityPayloadBoundary",
            left: "z",
            operator: "=",
            right: "z",
            recovered_label: false,
        };
        let reserve = || {
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("ChainObjectMode"),
            )]
        };
        let exact_modes = || {
            vec![
                mode_definition("BaseObjectMode", ReserveTypeShape::Builtin("object")),
                mode_definition(
                    "ChainObjectMode",
                    ReserveTypeShape::QualifiedSymbol("BaseObjectMode"),
                ),
            ]
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
        let output = super::source_chained_local_object_mode_reserved_variable_equality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact object-terminal chain should reach TermFormulaChecker");
        assert_source_reserved_variable_formula_output(&output)
            .expect("object-terminal chain invariants should hold");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 2);
        assert_eq!(output.left_binding, BindingId::new(0));
        assert_eq!(output.right_binding, BindingId::new(0));
        for input in [
            &output.left_result_input,
            &output.right_result_input,
            output.left_expected_input.as_ref().expect("left expected"),
            output
                .right_expected_input
                .as_ref()
                .expect("right expected"),
        ] {
            assert_eq!(input.spelling, "ChainObjectMode");
            assert!(matches!(input.head, TypeHeadInput::Symbol(_)));
        }
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("one normalized object type");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .values()
            .find(|expansion| matches!(expansion.radix.head, TypeHeadInput::BuiltinObject))
            .expect("terminal object expansion");
        assert_eq!(normalized.source.range, terminal.radix.source_range);

        let mut partial_chain_output =
            super::source_chained_local_object_mode_reserved_variable_equality_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("third exact output");
        partial_chain_output
            .payload
            .reserve
            .mode_expansions
            .retain(|symbol, _| source_mode_symbol_spelling(symbol) != Some("BaseObjectMode"));
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&partial_chain_output),
            vec![super::TYPE_ELABORATION_CHAINED_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY.to_owned()]
        );

        let mut invalid_output =
            super::source_chained_local_object_mode_reserved_variable_equality_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("second exact output");
        invalid_output.payload.reserve.mode_expansions.clear();
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&invalid_output),
            vec![super::TYPE_ELABORATION_CHAINED_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY.to_owned()]
        );

        let near_miss_modes = [
            vec![mode_definition(
                "BaseObjectMode",
                ReserveTypeShape::Builtin("object"),
            )],
            vec![
                mode_definition("BaseObjectMode", ReserveTypeShape::Builtin("set")),
                mode_definition(
                    "ChainObjectMode",
                    ReserveTypeShape::QualifiedSymbol("BaseObjectMode"),
                ),
            ],
            vec![
                contextual_mode_definition("BaseObjectMode", ReserveTypeShape::Builtin("object")),
                mode_definition(
                    "ChainObjectMode",
                    ReserveTypeShape::QualifiedSymbol("BaseObjectMode"),
                ),
            ],
            vec![
                parameterized_mode_definition(
                    "BaseObjectMode",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition(
                    "ChainObjectMode",
                    ReserveTypeShape::QualifiedSymbol("BaseObjectMode"),
                ),
            ],
            vec![
                recovered_mode_definition("BaseObjectMode", ReserveTypeShape::Builtin("object")),
                mode_definition(
                    "ChainObjectMode",
                    ReserveTypeShape::QualifiedSymbol("BaseObjectMode"),
                ),
            ],
            vec![
                mode_definition("BaseObjectMode", ReserveTypeShape::AttributedObject),
                mode_definition(
                    "ChainObjectMode",
                    ReserveTypeShape::QualifiedSymbol("BaseObjectMode"),
                ),
            ],
            vec![
                mode_definition("BaseObjectMode", ReserveTypeShape::Builtin("object")),
                mode_definition("BaseObjectMode", ReserveTypeShape::Builtin("object")),
                mode_definition(
                    "ChainObjectMode",
                    ReserveTypeShape::QualifiedSymbol("BaseObjectMode"),
                ),
            ],
            vec![
                mode_definition(
                    "BaseObjectMode",
                    ReserveTypeShape::QualifiedSymbol("ChainObjectMode"),
                ),
                mode_definition(
                    "ChainObjectMode",
                    ReserveTypeShape::QualifiedSymbol("BaseObjectMode"),
                ),
            ],
            vec![
                mode_definition("BaseObjectMode", ReserveTypeShape::Builtin("object")),
                mode_definition(
                    "InnerObjectMode",
                    ReserveTypeShape::QualifiedSymbol("BaseObjectMode"),
                ),
                mode_definition(
                    "ChainObjectMode",
                    ReserveTypeShape::QualifiedSymbol("InnerObjectMode"),
                ),
            ],
        ];
        for modes in near_miss_modes {
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
        }
        for near_miss in [
            reserve_then_mode_identifier_binary_theorem_ast(
                source_id,
                reserve(),
                mode_definition("ChainObjectMode", ReserveTypeShape::Builtin("object")),
                theorem,
            ),
            mode_then_reserve_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(
                    vec!["z"],
                    ReserveTypeShape::QualifiedSymbolWithArgs("ChainObjectMode"),
                )],
                theorem,
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
        ] {
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
    }
