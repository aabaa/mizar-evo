    #[test]
    fn source_reserved_variable_membership_bridge_uses_real_binding_and_type_payloads() {
        let source_id = source_id(97);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("reserved_variable_membership"),
        );
        let symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        let ast = reserve_then_identifier_binary_theorem_ast(
            source_id,
            vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
            "ReservedVariableMembershipPayloadBoundary",
            "x",
            "in",
            "x",
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&ast, module.clone(), &symbols),
            Vec::<String>::new()
        );
        let payload = extract_source_reserved_variable_membership(&ast, module.clone(), &symbols)
            .expect("exact reserved-variable membership source should extract");
        assert_eq!(payload.config.formula_kind, FormulaKind::Membership);
        assert_eq!(payload.left_spelling, "x");
        assert_eq!(payload.right_spelling, "x");
        assert_eq!(payload.left_lookup_ordinal, 1);
        assert_eq!(payload.right_lookup_ordinal, 2);

        let output = source_reserved_variable_membership_output(&ast, module.clone(), &symbols)
            .expect("exact reserved-variable membership source should reach the checker");
        assert_source_reserved_variable_formula_output(&output)
            .expect("reserved-variable membership output invariants should hold");
        assert_eq!(output.left_binding, BindingId::new(0));
        assert_eq!(output.right_binding, BindingId::new(0));
        assert_eq!(output.term_formula.terms().len(), 2);
        let (_, formula) = output
            .term_formula
            .formulas()
            .iter()
            .next()
            .expect("membership formula should be checked");
        assert_eq!(formula.kind, FormulaKind::Membership);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert_eq!(formula.expected_types.len(), 1);
        assert_eq!(formula.expected_types[0].term, payload.right_site);
        assert!(formula.facts.is_empty());
        assert!(output.term_formula.facts().is_empty());
        assert_eq!(output.term_formula.type_entries().len(), 5);

        let role_owners = output
            .term_formula
            .type_entries()
            .iter()
            .filter_map(|(_, entry)| match &entry.owner {
                TypedSiteRef::Role { node, role } => Some((node.index(), role.as_str().to_owned())),
                _ => None,
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(
            role_owners,
            BTreeSet::from([
                (
                    payload.left_site.node().index(),
                    "reserved-variable-membership-left-result".to_owned(),
                ),
                (
                    payload.right_site.node().index(),
                    "reserved-variable-membership-right-expected".to_owned(),
                ),
                (
                    payload.right_site.node().index(),
                    "reserved-variable-membership-right-result".to_owned(),
                ),
            ])
        );

        let pre_output_payload =
            extract_source_reserved_variable_membership(&ast, module.clone(), &symbols)
                .expect("exact source should produce a pre-output payload");
        let mismatched_symbols = SymbolEnv::new(
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("other_module")),
            SymbolEnvIndexes::default(),
        );
        assert_eq!(
            source_reserved_variable_formula_result_detail_keys(
                build_source_reserved_variable_formula_output(
                    pre_output_payload,
                    &mismatched_symbols,
                ),
                super::TYPE_ELABORATION_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
            ),
            vec![
                super::TYPE_ELABORATION_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY.to_owned()
            ]
        );

        let gap_cases = [
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
                "OtherPayloadBoundary",
                "x",
                "in",
                "x",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
                "ReservedVariableMembershipPayloadBoundary",
                "x",
                "in",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
                "ReservedVariableMembershipPayloadBoundary",
                "y",
                "in",
                "x",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
                "ReservedVariableMembershipPayloadBoundary",
                "x",
                "=",
                "x",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("object"))],
                "ReservedVariableMembershipPayloadBoundary",
                "x",
                "in",
                "x",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![reserve_item(
                    vec!["x", "y"],
                    ReserveTypeShape::Builtin("set"),
                )],
                "ReservedVariableMembershipPayloadBoundary",
                "x",
                "in",
                "x",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::AttributedSet)],
                "ReservedVariableMembershipPayloadBoundary",
                "x",
                "in",
                "x",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::Builtin("set")),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                ],
                "ReservedVariableMembershipPayloadBoundary",
                "x",
                "in",
                "x",
            ),
            reserve_then_identifier_binary_theorem_ast_with_options(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
                IdentifierBinaryTheoremSpec {
                    status: Some("registration"),
                    label: "ReservedVariableMembershipPayloadBoundary",
                    left: "x",
                    operator: "in",
                    right: "x",
                    recovered_label: false,
                },
            ),
            reserve_then_identifier_binary_theorem_ast_with_options(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
                IdentifierBinaryTheoremSpec {
                    status: None,
                    label: "ReservedVariableMembershipPayloadBoundary",
                    left: "x",
                    operator: "in",
                    right: "x",
                    recovered_label: true,
                },
            ),
            reserve_then_two_identifier_binary_theorems_ast(
                source_id,
                "ReservedVariableMembershipPayloadBoundary",
                "in",
            ),
            theorem_then_reserve_identifier_binary_ast(
                source_id,
                "ReservedVariableMembershipPayloadBoundary",
                "in",
            ),
            reserve_then_builtin_binary_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
                "ReservedVariableMembershipPayloadBoundary",
                "1",
                "in",
                "1",
            ),
        ];
        for gap_case in gap_cases {
            assert_eq!(
                source_type_elaboration_detail_keys(&gap_case, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
    }

    #[test]
    fn source_reserved_variable_inequality_bridge_uses_real_binding_and_type_payloads() {
        let source_id = source_id(98);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("reserved_variable_inequality"),
        );
        let symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        let ast = reserve_then_identifier_binary_theorem_ast(
            source_id,
            vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
            "ReservedVariableInequalityPayloadBoundary",
            "x",
            "<>",
            "x",
        );
        assert_eq!(
            source_type_elaboration_detail_keys(&ast, module.clone(), &symbols),
            Vec::<String>::new()
        );
        let payload = extract_source_reserved_variable_inequality(&ast, module.clone(), &symbols)
            .expect("exact reserved-variable inequality source should extract");
        assert_eq!(payload.config.formula_kind, FormulaKind::Inequality);
        assert_eq!(
            [payload.left_lookup_ordinal, payload.right_lookup_ordinal],
            [1, 2]
        );
        let output = source_reserved_variable_inequality_output(&ast, module.clone(), &symbols)
            .expect("exact reserved-variable inequality should reach the checker");
        assert_source_reserved_variable_formula_output(&output)
            .expect("reserved-variable inequality invariants should hold");
        let (_, formula) = output.term_formula.formulas().iter().next().unwrap();
        assert_eq!(formula.kind, FormulaKind::Inequality);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert_eq!(formula.expected_types.len(), 2);
        assert!(formula.facts.is_empty());
        assert_eq!(output.term_formula.type_entries().len(), 6);

        let pre_output =
            extract_source_reserved_variable_inequality(&ast, module.clone(), &symbols)
                .expect("exact source should produce a pre-output payload");
        let mismatched_symbols = SymbolEnv::new(
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("other_module")),
            SymbolEnvIndexes::default(),
        );
        assert_eq!(
            source_reserved_variable_formula_result_detail_keys(
                build_source_reserved_variable_formula_output(pre_output, &mismatched_symbols),
                super::TYPE_ELABORATION_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
            ),
            vec![
                super::TYPE_ELABORATION_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY.to_owned()
            ]
        );
        for gap_case in reserved_variable_binary_gap_cases(
            source_id,
            "ReservedVariableInequalityPayloadBoundary",
            "<>",
        ) {
            assert_eq!(
                source_type_elaboration_detail_keys(&gap_case, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
    }
