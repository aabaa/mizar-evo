    #[test]
    fn source_reserved_variable_equality_bridge_uses_real_binding_and_type_payloads() {
        let source_id = source_id(96);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("reserved_variable_equality"),
        );
        let symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        let ast = reserve_then_identifier_equality_theorem_ast(
            source_id,
            vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
            "ReservedVariableEqualityPayloadBoundary",
            "x",
            "x",
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&ast, module.clone(), &symbols),
            Vec::<String>::new()
        );
        let payload = extract_source_reserved_variable_equality(&ast, module.clone(), &symbols)
            .expect("exact reserved-variable equality source should extract");
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(payload.reserve.bridge.bindings()[0].spelling, "x");
        assert_eq!(payload.left_spelling, "x");
        assert_eq!(payload.right_spelling, "x");
        assert_eq!(payload.left_lookup_ordinal, 1);
        assert_eq!(payload.right_lookup_ordinal, 2);
        assert_eq!(
            payload.reserve.bridge.bindings()[0].type_head,
            TypeHeadInput::BuiltinSet
        );

        let output = source_reserved_variable_equality_output(&ast, module.clone(), &symbols)
            .expect("exact reserved-variable equality source should reach the checker");
        assert_source_reserved_variable_formula_output(&output)
            .expect("reserved-variable equality output invariants should hold");
        assert_eq!(output.handoff.binding_env.bindings().len(), 1);
        assert_eq!(output.handoff.declarations.declarations().len(), 1);
        assert!(output.handoff.declarations.diagnostics().is_empty());
        assert_eq!(output.term_formula.terms().len(), 2);
        for (_, term) in output.term_formula.terms().iter() {
            assert_eq!(term.kind, TermKind::Variable);
            assert_eq!(
                term.reference,
                Some(TermReference::Binding(BindingId::new(0)))
            );
            assert_eq!(term.status, TermStatus::Inferred);
            assert_eq!(
                output
                    .term_formula
                    .type_entries()
                    .get(term.type_entry)
                    .expect("variable term type entry should exist")
                    .status,
                TypeStatus::Known
            );
        }
        assert_eq!(output.term_formula.formulas().len(), 1);
        let (_, formula) = output
            .term_formula
            .formulas()
            .iter()
            .next()
            .expect("equality formula should be checked");
        assert_eq!(formula.site, payload.formula_site);
        assert_eq!(formula.kind, FormulaKind::Equality);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert_eq!(formula.terms, vec![payload.left_site, payload.right_site]);
        assert_eq!(formula.expected_types.len(), 2);
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
                "reserved-variable-left-expected".to_owned(),
                "reserved-variable-left-result".to_owned(),
                "reserved-variable-right-expected".to_owned(),
                "reserved-variable-right-result".to_owned(),
            ])
        );
        assert_eq!(output.term_formula.type_entries().len(), 6);

        let mut invalid_output =
            source_reserved_variable_equality_output(&ast, module.clone(), &symbols)
                .expect("exact source should produce a second checker output");
        invalid_output.left_binding = BindingId::new(1);
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&invalid_output),
            vec![super::TYPE_ELABORATION_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY.to_owned()]
        );
        let pre_output_payload =
            extract_source_reserved_variable_equality(&ast, module.clone(), &symbols)
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
                super::TYPE_ELABORATION_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
            ),
            vec![super::TYPE_ELABORATION_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY.to_owned()]
        );

        let gap_cases = [
            reserve_then_identifier_equality_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
                "OtherPayloadBoundary",
                "x",
                "x",
            ),
            reserve_then_identifier_equality_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
                "ReservedVariableEqualityPayloadBoundary",
                "x",
                "y",
            ),
            reserve_then_identifier_equality_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
                "ReservedVariableEqualityPayloadBoundary",
                "y",
                "x",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
                "ReservedVariableEqualityPayloadBoundary",
                "x",
                "<>",
                "x",
            ),
            reserve_then_identifier_equality_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("object"))],
                "ReservedVariableEqualityPayloadBoundary",
                "x",
                "x",
            ),
            reserve_then_identifier_equality_theorem_ast(
                source_id,
                vec![reserve_item(
                    vec!["x", "y"],
                    ReserveTypeShape::Builtin("set"),
                )],
                "ReservedVariableEqualityPayloadBoundary",
                "x",
                "x",
            ),
            reserve_then_identifier_equality_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::AttributedSet)],
                "ReservedVariableEqualityPayloadBoundary",
                "x",
                "x",
            ),
            reserve_then_identifier_equality_theorem_ast(
                source_id,
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::Builtin("set")),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                ],
                "ReservedVariableEqualityPayloadBoundary",
                "x",
                "x",
            ),
            reserve_then_identifier_binary_theorem_ast_with_options(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
                IdentifierBinaryTheoremSpec {
                    status: Some("registration"),
                    label: "ReservedVariableEqualityPayloadBoundary",
                    left: "x",
                    operator: "=",
                    right: "x",
                    recovered_label: false,
                },
            ),
            reserve_then_identifier_binary_theorem_ast_with_options(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
                IdentifierBinaryTheoremSpec {
                    status: None,
                    label: "ReservedVariableEqualityPayloadBoundary",
                    left: "x",
                    operator: "=",
                    right: "x",
                    recovered_label: true,
                },
            ),
            reserve_then_two_identifier_equality_theorems_ast(source_id),
            theorem_then_reserve_identifier_equality_ast(source_id),
            reserve_then_builtin_equality_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
                "ReservedVariableEqualityPayloadBoundary",
                "1",
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
    fn source_parenthesized_reserved_variable_equality_bridge_is_transparent() {
        let source_id = source_id(223);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("parenthesized_reserved_variable_equality"),
        );
        let symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        let reserve = || vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))];
        let parenthesized = |spelling| ParenthesizedIdentifierOperandShape::Identifier {
            spelling,
            depth: 1,
            recovered: false,
            open: "(",
            close: ")",
        };
        let exact_spec = ParenthesizedIdentifierBinaryTheoremSpec {
            status: None,
            label: "ParenthesizedReservedVariableEqualityPayloadBoundary",
            left: parenthesized("x"),
            operator: "=",
            right: ParenthesizedIdentifierOperandShape::Direct("x"),
            recovered_label: false,
        };
        let exact = reserve_then_parenthesized_identifier_binary_theorem_ast(
            source_id,
            reserve(),
            exact_spec,
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        let payload = super::extract_source_parenthesized_reserved_variable_equality(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact parenthesized reserved-variable equality should extract");
        assert_eq!(payload.formula.reserve.bridge.bindings().len(), 1);
        assert_eq!(payload.formula.reserve.bridge.bindings()[0].spelling, "x");
        assert_eq!(
            payload.formula.reserve.bridge.bindings()[0].type_head,
            TypeHeadInput::BuiltinSet
        );
        assert_eq!(payload.formula.left_spelling, "x");
        assert_eq!(payload.formula.right_spelling, "x");
        assert_eq!(payload.formula.left_lookup_ordinal, 1);
        assert_eq!(payload.formula.right_lookup_ordinal, 2);
        assert_ne!(payload.wrapper_site, payload.formula.left_site);
        assert_ne!(payload.wrapper_site, payload.formula.right_site);
        assert!(payload.wrapper_range.start < payload.formula.left_range.start);
        assert!(payload.wrapper_range.end > payload.formula.left_range.end);
        assert!(payload.wrapper_range.end <= payload.formula.right_range.start);

        let output = super::source_parenthesized_reserved_variable_equality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact parenthesized equality should reach the real equality checker");
        assert_eq!(output.wrapper_site, output.source_wrapper_site);
        assert_eq!(output.wrapper_range, output.source_wrapper_range);
        super::assert_source_parenthesized_reserved_variable_equality_output(&output)
            .expect("parenthesized equality output invariants should hold");
        assert_eq!(output.formula.left_binding, BindingId::new(0));
        assert_eq!(output.formula.right_binding, BindingId::new(0));
        assert_eq!(output.formula.term_formula.terms().len(), 2);
        assert_eq!(output.formula.term_formula.type_entries().len(), 6);
        assert_eq!(output.formula.term_formula.normalized_types().len(), 1);
        assert!(output.formula.term_formula.candidate_sets().is_empty());
        assert!(output.formula.term_formula.facts().is_empty());
        assert!(output.formula.term_formula.diagnostics().is_empty());
        let (_, normalized) = output
            .formula
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("parenthesized equality normalized set identity should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_eq!(
            normalized.source.range,
            output.formula.payload.reserve.bridge.bindings()[0].type_range
        );
        let (_, checked_formula) = output
            .formula
            .term_formula
            .formulas()
            .iter()
            .next()
            .expect("parenthesized equality formula should be checked");
        assert_eq!(checked_formula.kind, FormulaKind::Equality);
        assert_eq!(checked_formula.status, FormulaStatus::Checked);
        assert_eq!(
            checked_formula.terms,
            [
                output.formula.payload.left_site.clone(),
                output.formula.payload.right_site.clone(),
            ]
        );
        assert_eq!(checked_formula.expected_types.len(), 2);
        assert!(checked_formula.facts.is_empty());
        assert!(checked_formula.deferred.is_empty());
        let role_names = output
            .formula
            .term_formula
            .type_entries()
            .iter()
            .filter_map(|(_, entry)| match &entry.owner {
                TypedSiteRef::Role { role, .. } => Some(role.as_str().to_owned()),
                _ => None,
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(
            role_names,
            BTreeSet::from([
                "parenthesized-reserved-variable-left-expected".to_owned(),
                "parenthesized-reserved-variable-left-result".to_owned(),
                "parenthesized-reserved-variable-right-expected".to_owned(),
                "parenthesized-reserved-variable-right-result".to_owned(),
            ])
        );
        assert!(
            output
                .formula
                .term_formula
                .terms()
                .iter()
                .all(|(_, term)| term.site.node() != output.wrapper_site.node())
        );
        assert!(
            output
                .formula
                .term_formula
                .type_entries()
                .iter()
                .all(|(_, entry)| entry.owner.node() != output.wrapper_site.node())
        );

        let invalid_key =
            super::TYPE_ELABORATION_PARENTHESIZED_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY;
        let mut collapsed_wrapper = super::source_parenthesized_reserved_variable_equality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce a wrapper corruption target");
        collapsed_wrapper.wrapper_site = collapsed_wrapper.formula.payload.left_site.clone();
        assert_eq!(
            super::source_parenthesized_reserved_variable_equality_output_detail_keys(
                &collapsed_wrapper
            ),
            vec![invalid_key.to_owned()]
        );
        let mut collapsed_wrapper_range =
            super::source_parenthesized_reserved_variable_equality_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce a wrapper-range corruption target");
        collapsed_wrapper_range.wrapper_range = collapsed_wrapper_range.formula.payload.left_range;
        assert_eq!(
            super::source_parenthesized_reserved_variable_equality_output_detail_keys(
                &collapsed_wrapper_range
            ),
            vec![invalid_key.to_owned()]
        );
        let mut unrelated_wrapper_site =
            super::source_parenthesized_reserved_variable_equality_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce an unrelated-site corruption target");
        unrelated_wrapper_site.wrapper_site = TypedSiteRef::Role {
            node: unrelated_wrapper_site.wrapper_site.node(),
            role: super::TypeRole::new("unrelated-parenthesized-wrapper"),
        };
        assert_eq!(
            super::source_parenthesized_reserved_variable_equality_output_detail_keys(
                &unrelated_wrapper_site
            ),
            vec![invalid_key.to_owned()]
        );
        let mut plausible_wrapper_range =
            super::source_parenthesized_reserved_variable_equality_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce a plausible-range corruption target");
        plausible_wrapper_range.wrapper_range.end =
            plausible_wrapper_range.formula.payload.right_range.start;
        assert_eq!(
            super::source_parenthesized_reserved_variable_equality_output_detail_keys(
                &plausible_wrapper_range
            ),
            vec![invalid_key.to_owned()]
        );
        let mut collapsed_inner = super::source_parenthesized_reserved_variable_equality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce an inner-provenance corruption target");
        collapsed_inner.formula.payload.left_site = collapsed_inner.wrapper_site.clone();
        assert_eq!(
            super::source_parenthesized_reserved_variable_equality_output_detail_keys(
                &collapsed_inner
            ),
            vec![invalid_key.to_owned()]
        );
        let mut collapsed_right = super::source_parenthesized_reserved_variable_equality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce a right-provenance corruption target");
        collapsed_right.formula.payload.right_range = collapsed_right.formula.payload.left_range;
        assert_eq!(
            super::source_parenthesized_reserved_variable_equality_output_detail_keys(
                &collapsed_right
            ),
            vec![invalid_key.to_owned()]
        );
        let mut wrong_binding = super::source_parenthesized_reserved_variable_equality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce a checker corruption target");
        wrong_binding.formula.left_binding = BindingId::new(1);
        assert_eq!(
            super::source_parenthesized_reserved_variable_equality_output_detail_keys(
                &wrong_binding
            ),
            vec![invalid_key.to_owned()]
        );
        let mut immutable_wrapper_payload =
            super::extract_source_parenthesized_reserved_variable_equality(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce an immutable-output target");
        immutable_wrapper_payload.wrapper_range = immutable_wrapper_payload.formula.left_range;
        let immutable_wrapper_output =
            super::build_source_parenthesized_reserved_variable_binary_formula_output(
                immutable_wrapper_payload,
                &symbols,
            )
            .expect("wrapper-only corruption must not fabricate or mutate checker payloads");
        assert_eq!(
            immutable_wrapper_output.formula.term_formula.terms().len(),
            2
        );
        assert_eq!(
            immutable_wrapper_output
                .formula
                .term_formula
                .type_entries()
                .len(),
            6
        );
        assert_eq!(
            super::source_parenthesized_reserved_variable_equality_output_detail_keys(
                &immutable_wrapper_output
            ),
            vec![invalid_key.to_owned()]
        );
        let mismatched_symbols = SymbolEnv::new(
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("other_module")),
            SymbolEnvIndexes::default(),
        );
        let mismatched_payload = super::extract_source_parenthesized_reserved_variable_equality(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce a module-corruption payload");
        assert!(
            super::build_source_parenthesized_reserved_variable_binary_formula_output(
                mismatched_payload,
                &mismatched_symbols,
            )
            .is_err()
        );

        let direct_task_119 = reserve_then_identifier_equality_theorem_ast(
            source_id,
            reserve(),
            "ReservedVariableEqualityPayloadBoundary",
            "x",
            "x",
        );
        assert!(
            super::extract_source_parenthesized_reserved_variable_equality(
                &direct_task_119,
                module.clone(),
                &symbols,
            )
            .is_none()
        );
        assert!(
            extract_source_reserved_variable_equality(&direct_task_119, module.clone(), &symbols,)
                .is_some()
        );
        assert_eq!(
            source_type_elaboration_detail_keys(&direct_task_119, module.clone(), &symbols),
            Vec::<String>::new()
        );

        let near_misses = [
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: ParenthesizedIdentifierOperandShape::Direct("x"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: ParenthesizedIdentifierOperandShape::Direct("x"),
                right: parenthesized("x"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                right: parenthesized("x"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: ParenthesizedIdentifierOperandShape::Identifier {
                    spelling: "x",
                    depth: 2,
                    recovered: false,
                    open: "(",
                    close: ")",
                },
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: parenthesized("y"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                right: ParenthesizedIdentifierOperandShape::Direct("y"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: ParenthesizedIdentifierOperandShape::Identifier {
                    spelling: "x",
                    depth: 1,
                    recovered: true,
                    open: "(",
                    close: ")",
                },
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: ParenthesizedIdentifierOperandShape::Identifier {
                    spelling: "x",
                    depth: 1,
                    recovered: false,
                    open: "[",
                    close: "]",
                },
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: ParenthesizedIdentifierOperandShape::Numeral("1"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: ParenthesizedIdentifierOperandShape::Empty,
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: ParenthesizedIdentifierOperandShape::DoubleIdentifier("x"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                label: "OtherPayloadBoundary",
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                operator: "<>",
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                status: Some("registration"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                recovered_label: true,
                ..exact_spec
            },
        ]
        .into_iter()
        .map(|spec| {
            reserve_then_parenthesized_identifier_binary_theorem_ast(source_id, reserve(), spec)
        })
        .chain([
            reserve_then_parenthesized_identifier_binary_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("object"))],
                exact_spec,
            ),
            reserve_then_parenthesized_identifier_binary_theorem_ast(
                source_id,
                vec![reserve_item(
                    vec!["x", "y"],
                    ReserveTypeShape::Builtin("set"),
                )],
                exact_spec,
            ),
            reserve_then_parenthesized_identifier_binary_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::AttributedSet)],
                exact_spec,
            ),
            reserve_then_parenthesized_identifier_binary_theorem_ast(
                source_id,
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::Builtin("set")),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                ],
                exact_spec,
            ),
        ]);
        for near_miss in near_misses {
            assert!(
                super::extract_source_parenthesized_reserved_variable_equality(
                    &near_miss,
                    module.clone(),
                    &symbols,
                )
                .is_none()
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
    }

    #[test]
    fn source_parenthesized_reserved_variable_inequality_bridge_is_transparent() {
        let source_id = source_id(241);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("parenthesized_reserved_variable_inequality"),
        );
        let symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        let reserve = || vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))];
        let parenthesized = |spelling| ParenthesizedIdentifierOperandShape::Identifier {
            spelling,
            depth: 1,
            recovered: false,
            open: "(",
            close: ")",
        };
        let exact_spec = ParenthesizedIdentifierBinaryTheoremSpec {
            status: None,
            label: "ParenthesizedReservedVariableInequalityPayloadBoundary",
            left: parenthesized("x"),
            operator: "<>",
            right: ParenthesizedIdentifierOperandShape::Direct("x"),
            recovered_label: false,
        };
        let exact = reserve_then_parenthesized_identifier_binary_theorem_ast(
            source_id,
            reserve(),
            exact_spec,
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        let payload = super::extract_source_parenthesized_reserved_variable_inequality(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact parenthesized reserved-variable inequality should extract");
        assert!(
            super::extract_source_parenthesized_reserved_variable_equality(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none()
        );
        assert_eq!(payload.formula.reserve.bridge.bindings().len(), 1);
        assert_eq!(payload.formula.reserve.bridge.bindings()[0].spelling, "x");
        assert_eq!(
            payload.formula.reserve.bridge.bindings()[0].type_head,
            TypeHeadInput::BuiltinSet
        );
        assert_eq!(payload.formula.left_lookup_ordinal, 1);
        assert_eq!(payload.formula.right_lookup_ordinal, 2);
        assert_ne!(payload.wrapper_site, payload.formula.left_site);
        assert_ne!(payload.wrapper_site, payload.formula.right_site);
        assert!(payload.wrapper_range.start < payload.formula.left_range.start);
        assert!(payload.wrapper_range.end > payload.formula.left_range.end);
        assert!(payload.wrapper_range.end <= payload.formula.right_range.start);

        let output = super::source_parenthesized_reserved_variable_inequality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact parenthesized inequality should reach the real inequality checker");
        super::assert_source_parenthesized_reserved_variable_inequality_output(&output)
            .expect("parenthesized inequality output invariants should hold");
        assert_eq!(output.wrapper_site, output.source_wrapper_site);
        assert_eq!(output.wrapper_range, output.source_wrapper_range);
        assert_eq!(output.formula.left_binding, BindingId::new(0));
        assert_eq!(output.formula.right_binding, BindingId::new(0));
        assert_eq!(
            output.formula.left_result_input.head,
            TypeHeadInput::BuiltinSet
        );
        assert_eq!(
            output.formula.right_result_input.head,
            TypeHeadInput::BuiltinSet
        );
        assert_eq!(
            output.formula.left_expected_input.as_ref().unwrap().head,
            TypeHeadInput::BuiltinSet
        );
        assert_eq!(
            output.formula.right_expected_input.as_ref().unwrap().head,
            TypeHeadInput::BuiltinSet
        );
        assert_eq!(output.formula.term_formula.terms().len(), 2);
        assert_eq!(output.formula.term_formula.type_entries().len(), 6);
        assert_eq!(output.formula.term_formula.normalized_types().len(), 1);
        assert!(output.formula.term_formula.candidate_sets().is_empty());
        assert!(output.formula.term_formula.facts().is_empty());
        assert!(output.formula.term_formula.diagnostics().is_empty());
        let (_, normalized) = output
            .formula
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("parenthesized inequality normalized set identity should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_eq!(normalized.source.spelling, "set");
        assert_eq!(
            normalized.source.range,
            output.formula.payload.reserve.bridge.bindings()[0].type_range
        );
        let (_, checked_formula) = output
            .formula
            .term_formula
            .formulas()
            .iter()
            .next()
            .expect("parenthesized inequality formula should be checked");
        assert_eq!(checked_formula.kind, FormulaKind::Inequality);
        assert_eq!(checked_formula.status, FormulaStatus::Checked);
        assert_eq!(
            checked_formula.terms,
            [
                output.formula.payload.left_site.clone(),
                output.formula.payload.right_site.clone(),
            ]
        );
        assert_eq!(checked_formula.expected_types.len(), 2);
        assert!(checked_formula.facts.is_empty());
        assert!(checked_formula.deferred.is_empty());
        let role_names = output
            .formula
            .term_formula
            .type_entries()
            .iter()
            .filter_map(|(_, entry)| match &entry.owner {
                TypedSiteRef::Role { role, .. } => Some(role.as_str().to_owned()),
                _ => None,
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(
            role_names,
            BTreeSet::from([
                "parenthesized-reserved-variable-inequality-left-expected".to_owned(),
                "parenthesized-reserved-variable-inequality-left-result".to_owned(),
                "parenthesized-reserved-variable-inequality-right-expected".to_owned(),
                "parenthesized-reserved-variable-inequality-right-result".to_owned(),
            ])
        );
        assert!(
            output
                .formula
                .term_formula
                .terms()
                .iter()
                .all(|(_, term)| { term.site.node() != output.wrapper_site.node() })
        );
        assert!(
            output
                .formula
                .term_formula
                .type_entries()
                .iter()
                .all(|(_, entry)| { entry.owner.node() != output.wrapper_site.node() })
        );
        assert!(
            output
                .formula
                .term_formula
                .formulas()
                .iter()
                .all(|(_, formula)| {
                    formula.site.node() != output.wrapper_site.node()
                        && formula
                            .terms
                            .iter()
                            .all(|term| term.node() != output.wrapper_site.node())
                })
        );

        let invalid_key =
            super::TYPE_ELABORATION_PARENTHESIZED_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY;
        let output_for_corruption = || {
            super::source_parenthesized_reserved_variable_inequality_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce an inequality corruption target")
        };
        let assert_invalid = |output| {
            assert_eq!(
                super::source_parenthesized_reserved_variable_inequality_output_detail_keys(output,),
                vec![invalid_key.to_owned()]
            );
        };
        let mut collapsed_wrapper = output_for_corruption();
        collapsed_wrapper.wrapper_site = collapsed_wrapper.formula.payload.left_site.clone();
        assert_invalid(&collapsed_wrapper);
        let mut collapsed_wrapper_range = output_for_corruption();
        collapsed_wrapper_range.wrapper_range = collapsed_wrapper_range.formula.payload.left_range;
        assert_invalid(&collapsed_wrapper_range);
        let mut stale_source_wrapper = output_for_corruption();
        stale_source_wrapper.source_wrapper_site =
            stale_source_wrapper.formula.payload.right_site.clone();
        assert_invalid(&stale_source_wrapper);
        let mut collapsed_inner = output_for_corruption();
        collapsed_inner.formula.payload.left_site = collapsed_inner.wrapper_site.clone();
        assert_invalid(&collapsed_inner);
        let mut collapsed_right = output_for_corruption();
        collapsed_right.formula.payload.right_range = collapsed_right.formula.payload.left_range;
        assert_invalid(&collapsed_right);
        let mut wrong_binding = output_for_corruption();
        wrong_binding.formula.left_binding = BindingId::new(1);
        assert_invalid(&wrong_binding);
        let mut wrong_head = output_for_corruption();
        wrong_head.formula.left_result_input.head = TypeHeadInput::BuiltinObject;
        assert_invalid(&wrong_head);
        let mut collapsed_role = output_for_corruption();
        collapsed_role
            .formula
            .left_expected_input
            .as_mut()
            .unwrap()
            .site = collapsed_role.formula.left_result_input.site.clone();
        assert_invalid(&collapsed_role);
        let mut wrong_source = output_for_corruption();
        wrong_source.formula.left_result_input.source_range = payload.formula.left_range;
        assert_invalid(&wrong_source);
        let mut wrong_canonical_source = output_for_corruption();
        let (bridge_source_id, bridge_module, bridge_range, mut bridge_bindings) = {
            let bridge = &wrong_canonical_source.formula.payload.reserve.bridge;
            (
                bridge.source_id(),
                bridge.module_id().clone(),
                bridge.source_range(),
                bridge.bindings().to_vec(),
            )
        };
        assert_ne!(
            bridge_bindings[0].type_range,
            wrong_canonical_source.formula.payload.left_range
        );
        bridge_bindings[0].type_range = wrong_canonical_source.formula.payload.left_range;
        wrong_canonical_source.formula.payload.reserve.bridge =
            super::SourceReserveDeclarationBridge::new(
                bridge_source_id,
                bridge_module,
                bridge_range,
                bridge_bindings,
            )
            .expect("canonical-source corruption bridge should remain structurally valid");
        assert_invalid(&wrong_canonical_source);
        let mut missing_expected = output_for_corruption();
        missing_expected.formula.right_expected_input = None;
        assert_invalid(&missing_expected);
        let equality_output = super::source_parenthesized_reserved_variable_equality_output(
            &reserve_then_parenthesized_identifier_binary_theorem_ast(
                source_id,
                reserve(),
                ParenthesizedIdentifierBinaryTheoremSpec {
                    label: "ParenthesizedReservedVariableEqualityPayloadBoundary",
                    operator: "=",
                    ..exact_spec
                },
            ),
            module.clone(),
            &symbols,
        )
        .expect("Task 223 equality source should retain its own closed owner");
        assert_invalid(&equality_output);

        let mut wrong_ordinal = super::extract_source_parenthesized_reserved_variable_inequality(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce an ordinal corruption target");
        wrong_ordinal.formula.left_lookup_ordinal = wrong_ordinal.formula.right_lookup_ordinal;
        assert_eq!(
            super::source_parenthesized_reserved_variable_binary_formula_payload_detail_keys(
                wrong_ordinal,
                &symbols,
                &super::SOURCE_PARENTHESIZED_RESERVED_VARIABLE_INEQUALITY_CONFIG,
                super::SourceParenthesizedOperandSide::Left,
            ),
            vec![invalid_key.to_owned()]
        );
        let mut immutable_payload =
            super::extract_source_parenthesized_reserved_variable_inequality(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce an immutable-output target");
        immutable_payload.wrapper_range = immutable_payload.formula.left_range;
        let immutable_output =
            super::build_source_parenthesized_reserved_variable_binary_formula_output(
                immutable_payload,
                &symbols,
            )
            .expect("wrapper-only corruption must not mutate checker payloads");
        assert_eq!(immutable_output.formula.term_formula.terms().len(), 2);
        assert_eq!(
            immutable_output.formula.term_formula.type_entries().len(),
            6
        );
        assert_invalid(&immutable_output);
        let mismatched_symbols = SymbolEnv::new(
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("other_module")),
            SymbolEnvIndexes::default(),
        );
        let mismatched_payload = super::extract_source_parenthesized_reserved_variable_inequality(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce a module-corruption target");
        assert!(
            super::build_source_parenthesized_reserved_variable_binary_formula_output(
                mismatched_payload,
                &mismatched_symbols,
            )
            .is_err()
        );

        let direct_task_121 = reserve_then_identifier_binary_theorem_ast(
            source_id,
            reserve(),
            "ReservedVariableInequalityPayloadBoundary",
            "x",
            "<>",
            "x",
        );
        assert!(
            super::extract_source_parenthesized_reserved_variable_inequality(
                &direct_task_121,
                module.clone(),
                &symbols,
            )
            .is_none()
        );
        assert!(
            super::extract_source_reserved_variable_inequality(
                &direct_task_121,
                module.clone(),
                &symbols,
            )
            .is_some()
        );

        let near_miss_specs = [
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: ParenthesizedIdentifierOperandShape::Direct("x"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: ParenthesizedIdentifierOperandShape::Direct("x"),
                right: parenthesized("x"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                right: parenthesized("x"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: ParenthesizedIdentifierOperandShape::Identifier {
                    spelling: "x",
                    depth: 2,
                    recovered: false,
                    open: "(",
                    close: ")",
                },
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: parenthesized("y"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                right: ParenthesizedIdentifierOperandShape::Direct("y"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: ParenthesizedIdentifierOperandShape::Identifier {
                    spelling: "x",
                    depth: 1,
                    recovered: true,
                    open: "(",
                    close: ")",
                },
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: ParenthesizedIdentifierOperandShape::Identifier {
                    spelling: "x",
                    depth: 1,
                    recovered: false,
                    open: "[",
                    close: "]",
                },
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: ParenthesizedIdentifierOperandShape::Numeral("1"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: ParenthesizedIdentifierOperandShape::Empty,
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: ParenthesizedIdentifierOperandShape::DoubleIdentifier("x"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                label: "OtherPayloadBoundary",
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                operator: "=",
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                operator: "in",
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                status: Some("registration"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                recovered_label: true,
                ..exact_spec
            },
        ];
        let near_misses = near_miss_specs
            .into_iter()
            .map(|spec| {
                reserve_then_parenthesized_identifier_binary_theorem_ast(source_id, reserve(), spec)
            })
            .chain([
                reserve_then_parenthesized_identifier_binary_theorem_ast(
                    source_id,
                    vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("object"))],
                    exact_spec,
                ),
                reserve_then_parenthesized_identifier_binary_theorem_ast(
                    source_id,
                    vec![reserve_item(
                        vec!["x", "y"],
                        ReserveTypeShape::Builtin("set"),
                    )],
                    exact_spec,
                ),
                reserve_then_parenthesized_identifier_binary_theorem_ast(
                    source_id,
                    vec![reserve_item(vec!["x"], ReserveTypeShape::AttributedSet)],
                    exact_spec,
                ),
                reserve_then_parenthesized_identifier_binary_theorem_ast(
                    source_id,
                    vec![
                        reserve_item(vec!["x"], ReserveTypeShape::Builtin("set")),
                        reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                    ],
                    exact_spec,
                ),
            ]);
        for near_miss in near_misses {
            assert!(
                super::extract_source_parenthesized_reserved_variable_inequality(
                    &near_miss,
                    module.clone(),
                    &symbols,
                )
                .is_none()
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
    }

    #[test]
    fn source_parenthesized_reserved_variable_membership_bridge_is_transparent() {
        let source_id = source_id(243);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("parenthesized_reserved_variable_membership"),
        );
        let symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        let reserve = || vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))];
        let parenthesized = |spelling| ParenthesizedIdentifierOperandShape::Identifier {
            spelling,
            depth: 1,
            recovered: false,
            open: "(",
            close: ")",
        };
        let exact_spec = ParenthesizedIdentifierBinaryTheoremSpec {
            status: None,
            label: "ParenthesizedReservedVariableMembershipPayloadBoundary",
            left: parenthesized("x"),
            operator: "in",
            right: ParenthesizedIdentifierOperandShape::Direct("x"),
            recovered_label: false,
        };
        let exact = reserve_then_parenthesized_identifier_binary_theorem_ast(
            source_id,
            reserve(),
            exact_spec,
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        let payload = super::extract_source_parenthesized_reserved_variable_membership(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact parenthesized reserved-variable membership should extract");
        for extractor in [
            super::extract_source_parenthesized_reserved_variable_equality,
            super::extract_source_parenthesized_reserved_variable_inequality,
            super::extract_source_parenthesized_reserved_object_variable_equality,
            super::extract_source_parenthesized_reserved_object_variable_inequality,
        ] {
            assert!(extractor(&exact, module.clone(), &symbols).is_none());
        }
        assert_eq!(payload.formula.reserve.bridge.bindings().len(), 1);
        assert_eq!(payload.formula.reserve.bridge.bindings()[0].spelling, "x");
        assert_eq!(
            payload.formula.reserve.bridge.bindings()[0].type_head,
            TypeHeadInput::BuiltinSet
        );
        assert_eq!(payload.formula.left_lookup_ordinal, 1);
        assert_eq!(payload.formula.right_lookup_ordinal, 2);
        assert_ne!(payload.wrapper_site, payload.formula.left_site);
        assert_ne!(payload.wrapper_site, payload.formula.right_site);
        assert!(payload.wrapper_range.start < payload.formula.left_range.start);
        assert!(payload.wrapper_range.end > payload.formula.left_range.end);
        assert!(payload.wrapper_range.end <= payload.formula.right_range.start);

        let output = super::source_parenthesized_reserved_variable_membership_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact parenthesized membership should reach the real membership checker");
        super::assert_source_parenthesized_reserved_variable_membership_output(&output)
            .expect("parenthesized membership output invariants should hold");
        assert_eq!(output.wrapper_site, output.source_wrapper_site);
        assert_eq!(output.wrapper_range, output.source_wrapper_range);
        assert_eq!(output.formula.left_binding, BindingId::new(0));
        assert_eq!(output.formula.right_binding, BindingId::new(0));
        assert_eq!(
            output.formula.left_result_input.head,
            TypeHeadInput::BuiltinSet
        );
        assert_eq!(
            output.formula.right_result_input.head,
            TypeHeadInput::BuiltinSet
        );
        assert!(output.formula.left_expected_input.is_none());
        assert_eq!(
            output.formula.right_expected_input.as_ref().unwrap().head,
            TypeHeadInput::BuiltinSet
        );
        assert_eq!(output.formula.term_formula.terms().len(), 2);
        assert_eq!(output.formula.term_formula.type_entries().len(), 5);
        assert_eq!(output.formula.term_formula.normalized_types().len(), 1);
        assert!(output.formula.term_formula.candidate_sets().is_empty());
        assert!(output.formula.term_formula.facts().is_empty());
        assert!(output.formula.term_formula.diagnostics().is_empty());
        let (_, normalized) = output
            .formula
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("parenthesized membership normalized set identity should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_eq!(normalized.source.spelling, "set");
        assert_eq!(
            normalized.source.range,
            output.formula.payload.reserve.bridge.bindings()[0].type_range
        );
        let (_, checked_formula) = output
            .formula
            .term_formula
            .formulas()
            .iter()
            .next()
            .expect("parenthesized membership formula should be checked");
        assert_eq!(checked_formula.kind, FormulaKind::Membership);
        assert_eq!(checked_formula.status, FormulaStatus::Checked);
        assert_eq!(
            checked_formula.terms,
            [
                output.formula.payload.left_site.clone(),
                output.formula.payload.right_site.clone(),
            ]
        );
        assert_eq!(checked_formula.expected_types.len(), 1);
        assert_eq!(
            checked_formula.expected_types[0].term,
            output.formula.payload.right_site
        );
        assert!(checked_formula.facts.is_empty());
        assert!(checked_formula.deferred.is_empty());
        let role_names = output
            .formula
            .term_formula
            .type_entries()
            .iter()
            .filter_map(|(_, entry)| match &entry.owner {
                TypedSiteRef::Role { role, .. } => Some(role.as_str().to_owned()),
                _ => None,
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(
            role_names,
            BTreeSet::from([
                "parenthesized-reserved-variable-membership-left-result".to_owned(),
                "parenthesized-reserved-variable-membership-right-expected".to_owned(),
                "parenthesized-reserved-variable-membership-right-result".to_owned(),
            ])
        );
        assert!(
            output
                .formula
                .term_formula
                .terms()
                .iter()
                .all(|(_, term)| term.site.node() != output.wrapper_site.node())
        );
        assert!(
            output
                .formula
                .term_formula
                .type_entries()
                .iter()
                .all(|(_, entry)| entry.owner.node() != output.wrapper_site.node())
        );
        assert!(
            output
                .formula
                .term_formula
                .formulas()
                .iter()
                .all(|(_, formula)| {
                    formula.site.node() != output.wrapper_site.node()
                        && formula
                            .terms
                            .iter()
                            .all(|term| term.node() != output.wrapper_site.node())
                })
        );

        let invalid_key =
            super::TYPE_ELABORATION_PARENTHESIZED_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY;
        let output_for_corruption = || {
            super::source_parenthesized_reserved_variable_membership_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce a membership corruption target")
        };
        let assert_invalid = |output| {
            assert_eq!(
                super::source_parenthesized_reserved_variable_membership_output_detail_keys(output,),
                vec![invalid_key.to_owned()]
            );
        };
        let mut collapsed_wrapper = output_for_corruption();
        collapsed_wrapper.wrapper_site = collapsed_wrapper.formula.payload.left_site.clone();
        assert_invalid(&collapsed_wrapper);
        let mut collapsed_wrapper_range = output_for_corruption();
        collapsed_wrapper_range.wrapper_range = collapsed_wrapper_range.formula.payload.left_range;
        assert_invalid(&collapsed_wrapper_range);
        let mut stale_source_wrapper_site = output_for_corruption();
        stale_source_wrapper_site.source_wrapper_site =
            stale_source_wrapper_site.formula.payload.right_site.clone();
        assert_invalid(&stale_source_wrapper_site);
        let mut stale_source_wrapper_range = output_for_corruption();
        stale_source_wrapper_range.source_wrapper_range =
            stale_source_wrapper_range.formula.payload.left_range;
        assert_invalid(&stale_source_wrapper_range);
        let mut collapsed_inner_site = output_for_corruption();
        collapsed_inner_site.formula.payload.left_site = collapsed_inner_site.wrapper_site.clone();
        assert_invalid(&collapsed_inner_site);
        let mut collapsed_inner_range = output_for_corruption();
        collapsed_inner_range.formula.payload.left_range = collapsed_inner_range.wrapper_range;
        assert_invalid(&collapsed_inner_range);
        let mut collapsed_right_site = output_for_corruption();
        collapsed_right_site.formula.payload.right_site = collapsed_right_site.wrapper_site.clone();
        assert_invalid(&collapsed_right_site);
        let mut collapsed_right_range = output_for_corruption();
        collapsed_right_range.formula.payload.right_range =
            collapsed_right_range.formula.payload.left_range;
        assert_invalid(&collapsed_right_range);
        let mut collapsed_formula_site = output_for_corruption();
        collapsed_formula_site.formula.payload.formula_site =
            collapsed_formula_site.wrapper_site.clone();
        assert_invalid(&collapsed_formula_site);
        let mut collapsed_formula_range = output_for_corruption();
        collapsed_formula_range.formula.payload.formula_range =
            collapsed_formula_range.wrapper_range;
        assert_invalid(&collapsed_formula_range);
        let mut wrong_binding = output_for_corruption();
        wrong_binding.formula.right_binding = BindingId::new(1);
        assert_invalid(&wrong_binding);
        let mut wrong_head = output_for_corruption();
        wrong_head.formula.left_result_input.head = TypeHeadInput::BuiltinObject;
        assert_invalid(&wrong_head);
        let mut unexpected_left_expected = output_for_corruption();
        unexpected_left_expected.formula.left_expected_input = unexpected_left_expected
            .formula
            .right_expected_input
            .clone();
        assert_invalid(&unexpected_left_expected);
        let mut wrong_right_expected_head = output_for_corruption();
        wrong_right_expected_head
            .formula
            .right_expected_input
            .as_mut()
            .unwrap()
            .head = TypeHeadInput::BuiltinObject;
        assert_invalid(&wrong_right_expected_head);
        let mut collapsed_role = output_for_corruption();
        collapsed_role
            .formula
            .right_expected_input
            .as_mut()
            .unwrap()
            .site = collapsed_role.formula.right_result_input.site.clone();
        assert_invalid(&collapsed_role);
        let mut wrong_source = output_for_corruption();
        wrong_source
            .formula
            .right_expected_input
            .as_mut()
            .unwrap()
            .source_range = payload.formula.right_range;
        assert_invalid(&wrong_source);
        let mut wrong_canonical_source = output_for_corruption();
        let (bridge_source_id, bridge_module, bridge_range, mut bridge_bindings) = {
            let bridge = &wrong_canonical_source.formula.payload.reserve.bridge;
            (
                bridge.source_id(),
                bridge.module_id().clone(),
                bridge.source_range(),
                bridge.bindings().to_vec(),
            )
        };
        bridge_bindings[0].type_range = wrong_canonical_source.formula.payload.left_range;
        wrong_canonical_source.formula.payload.reserve.bridge =
            super::SourceReserveDeclarationBridge::new(
                bridge_source_id,
                bridge_module,
                bridge_range,
                bridge_bindings,
            )
            .expect("canonical-source corruption bridge should remain structurally valid");
        assert_invalid(&wrong_canonical_source);
        let mut missing_expected = output_for_corruption();
        missing_expected.formula.right_expected_input = None;
        assert_invalid(&missing_expected);

        let equality_output = super::source_parenthesized_reserved_variable_equality_output(
            &reserve_then_parenthesized_identifier_binary_theorem_ast(
                source_id,
                reserve(),
                ParenthesizedIdentifierBinaryTheoremSpec {
                    label: "ParenthesizedReservedVariableEqualityPayloadBoundary",
                    operator: "=",
                    ..exact_spec
                },
            ),
            module.clone(),
            &symbols,
        )
        .expect("Task 223 equality source should retain its closed owner");
        assert_invalid(&equality_output);
        let inequality_output = super::source_parenthesized_reserved_variable_inequality_output(
            &reserve_then_parenthesized_identifier_binary_theorem_ast(
                source_id,
                reserve(),
                ParenthesizedIdentifierBinaryTheoremSpec {
                    label: "ParenthesizedReservedVariableInequalityPayloadBoundary",
                    operator: "<>",
                    ..exact_spec
                },
            ),
            module.clone(),
            &symbols,
        )
        .expect("Task 241 inequality source should retain its closed owner");
        assert_invalid(&inequality_output);
        let object_inequality_output =
            super::source_parenthesized_reserved_object_variable_inequality_output(
                &reserve_then_parenthesized_identifier_binary_theorem_ast(
                    source_id,
                    vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("object"))],
                    ParenthesizedIdentifierBinaryTheoremSpec {
                        label: "ParenthesizedReservedObjectVariableInequalityPayloadBoundary",
                        operator: "<>",
                        ..exact_spec
                    },
                ),
                module.clone(),
                &symbols,
            )
            .expect("Task 242 object inequality source should retain its closed owner");
        assert_invalid(&object_inequality_output);

        let mut wrong_ordinal = super::extract_source_parenthesized_reserved_variable_membership(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce an ordinal corruption target");
        wrong_ordinal.formula.left_lookup_ordinal = wrong_ordinal.formula.right_lookup_ordinal;
        assert_eq!(
            super::source_parenthesized_reserved_variable_binary_formula_payload_detail_keys(
                wrong_ordinal,
                &symbols,
                &super::SOURCE_PARENTHESIZED_RESERVED_VARIABLE_MEMBERSHIP_CONFIG,
                super::SourceParenthesizedOperandSide::Left,
            ),
            vec![invalid_key.to_owned()]
        );
        let mut immutable_payload =
            super::extract_source_parenthesized_reserved_variable_membership(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce an immutable-output target");
        immutable_payload.wrapper_range = immutable_payload.formula.left_range;
        let immutable_output =
            super::build_source_parenthesized_reserved_variable_binary_formula_output(
                immutable_payload,
                &symbols,
            )
            .expect("wrapper-only corruption must not mutate checker payloads");
        assert_eq!(immutable_output.formula.term_formula.terms().len(), 2);
        assert_eq!(
            immutable_output.formula.term_formula.type_entries().len(),
            5
        );
        assert_invalid(&immutable_output);
        let mismatched_symbols = SymbolEnv::new(
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("other_module")),
            SymbolEnvIndexes::default(),
        );
        let mismatched_payload = super::extract_source_parenthesized_reserved_variable_membership(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce a module-corruption target");
        assert!(
            super::build_source_parenthesized_reserved_variable_binary_formula_output(
                mismatched_payload,
                &mismatched_symbols,
            )
            .is_err()
        );

        let direct_task_120 = reserve_then_identifier_binary_theorem_ast(
            source_id,
            reserve(),
            "ReservedVariableMembershipPayloadBoundary",
            "x",
            "in",
            "x",
        );
        assert!(
            super::extract_source_parenthesized_reserved_variable_membership(
                &direct_task_120,
                module.clone(),
                &symbols,
            )
            .is_none()
        );
        assert!(
            super::extract_source_reserved_variable_membership(
                &direct_task_120,
                module.clone(),
                &symbols,
            )
            .is_some()
        );
        let heterogeneous_direct = reserve_then_identifier_binary_theorem_ast(
            source_id,
            vec![
                reserve_item(vec!["x"], ReserveTypeShape::Builtin("object")),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
            ],
            "HeterogeneousReserveMembershipPayloadBoundary",
            "x",
            "in",
            "y",
        );
        assert!(
            super::extract_source_heterogeneous_reserve_membership(
                &heterogeneous_direct,
                module.clone(),
                &symbols,
            )
            .is_some()
        );
        assert!(
            super::extract_source_parenthesized_reserved_variable_membership(
                &heterogeneous_direct,
                module.clone(),
                &symbols,
            )
            .is_none()
        );
        assert_eq!(
            source_type_elaboration_detail_keys(&heterogeneous_direct, module.clone(), &symbols,),
            Vec::<String>::new()
        );

        let near_miss_specs = [
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: ParenthesizedIdentifierOperandShape::Direct("x"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: ParenthesizedIdentifierOperandShape::Direct("x"),
                right: parenthesized("x"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                right: parenthesized("x"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: ParenthesizedIdentifierOperandShape::Identifier {
                    spelling: "x",
                    depth: 2,
                    recovered: false,
                    open: "(",
                    close: ")",
                },
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: parenthesized("y"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                right: ParenthesizedIdentifierOperandShape::Direct("y"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: ParenthesizedIdentifierOperandShape::Identifier {
                    spelling: "x",
                    depth: 1,
                    recovered: true,
                    open: "(",
                    close: ")",
                },
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: ParenthesizedIdentifierOperandShape::Identifier {
                    spelling: "x",
                    depth: 1,
                    recovered: false,
                    open: "[",
                    close: "]",
                },
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: ParenthesizedIdentifierOperandShape::Numeral("1"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: ParenthesizedIdentifierOperandShape::Empty,
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: ParenthesizedIdentifierOperandShape::DoubleIdentifier("x"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                label: "OtherPayloadBoundary",
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                operator: "=",
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                operator: "<>",
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                status: Some("registration"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                recovered_label: true,
                ..exact_spec
            },
        ];
        let near_misses = near_miss_specs
            .into_iter()
            .map(|spec| {
                reserve_then_parenthesized_identifier_binary_theorem_ast(source_id, reserve(), spec)
            })
            .chain([
                reserve_then_parenthesized_identifier_binary_theorem_ast(
                    source_id,
                    vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("object"))],
                    exact_spec,
                ),
                reserve_then_parenthesized_identifier_binary_theorem_ast(
                    source_id,
                    vec![reserve_item(
                        vec!["x", "y"],
                        ReserveTypeShape::Builtin("set"),
                    )],
                    exact_spec,
                ),
                reserve_then_parenthesized_identifier_binary_theorem_ast(
                    source_id,
                    vec![reserve_item(vec!["x"], ReserveTypeShape::AttributedSet)],
                    exact_spec,
                ),
                reserve_then_parenthesized_identifier_binary_theorem_ast(
                    source_id,
                    vec![
                        reserve_item(vec!["x"], ReserveTypeShape::Builtin("set")),
                        reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                    ],
                    exact_spec,
                ),
            ]);
        for near_miss in near_misses {
            assert!(
                super::extract_source_parenthesized_reserved_variable_membership(
                    &near_miss,
                    module.clone(),
                    &symbols,
                )
                .is_none()
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
    }

    #[test]
    fn source_right_parenthesized_reserved_variable_membership_bridge_is_transparent() {
        let source_id = source_id(245);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("right_parenthesized_reserved_variable_membership"),
        );
        let symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        let reserve = || vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))];
        let parenthesized = |spelling| ParenthesizedIdentifierOperandShape::Identifier {
            spelling,
            depth: 1,
            recovered: false,
            open: "(",
            close: ")",
        };
        let exact_spec = ParenthesizedIdentifierBinaryTheoremSpec {
            status: None,
            label: "RightParenthesizedReservedVariableMembershipPayloadBoundary",
            left: ParenthesizedIdentifierOperandShape::Direct("x"),
            operator: "in",
            right: parenthesized("x"),
            recovered_label: false,
        };
        let exact = reserve_then_parenthesized_identifier_binary_theorem_ast(
            source_id,
            reserve(),
            exact_spec,
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        let payload = super::extract_source_right_parenthesized_reserved_variable_membership(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact right-parenthesized membership should extract");
        assert_eq!(
            payload.wrapper_side,
            super::SourceParenthesizedOperandSide::Right
        );
        assert_eq!(payload.formula.reserve.bridge.bindings().len(), 1);
        assert_eq!(payload.formula.reserve.bridge.bindings()[0].spelling, "x");
        assert_eq!(
            payload.formula.reserve.bridge.bindings()[0].type_head,
            TypeHeadInput::BuiltinSet
        );
        assert_eq!(payload.formula.left_lookup_ordinal, 1);
        assert_eq!(payload.formula.right_lookup_ordinal, 2);
        assert_ne!(payload.wrapper_site, payload.formula.formula_site);
        assert_ne!(payload.wrapper_site, payload.formula.left_site);
        assert_ne!(payload.wrapper_site, payload.formula.right_site);
        assert_ne!(payload.formula.formula_site, payload.formula.left_site);
        assert_ne!(payload.formula.formula_site, payload.formula.right_site);
        assert_ne!(payload.formula.left_site, payload.formula.right_site);
        assert!(payload.formula.left_range.end <= payload.wrapper_range.start);
        assert!(payload.wrapper_range.start < payload.formula.right_range.start);
        assert!(payload.wrapper_range.end > payload.formula.right_range.end);
        assert!(payload.formula.formula_range.start <= payload.formula.left_range.start);
        assert!(payload.formula.formula_range.end >= payload.wrapper_range.end);

        for extractor in [
            super::extract_source_parenthesized_reserved_variable_equality,
            super::extract_source_parenthesized_reserved_variable_inequality,
            super::extract_source_parenthesized_reserved_variable_membership,
            super::extract_source_parenthesized_heterogeneous_reserve_membership,
            super::extract_source_parenthesized_reserved_object_variable_equality,
            super::extract_source_parenthesized_reserved_object_variable_inequality,
        ] {
            assert!(extractor(&exact, module.clone(), &symbols).is_none());
        }

        let output = super::source_right_parenthesized_reserved_variable_membership_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact right-parenthesized membership should reach the real checker");
        super::assert_source_right_parenthesized_reserved_variable_membership_output(&output)
            .expect("right-parenthesized membership output invariants should hold");
        assert_eq!(
            output.source_wrapper_side,
            super::SourceParenthesizedOperandSide::Right
        );
        assert_eq!(output.wrapper_side, output.source_wrapper_side);
        assert_eq!(output.wrapper_site, output.source_wrapper_site);
        assert_eq!(output.wrapper_range, output.source_wrapper_range);
        assert_eq!(output.formula.left_binding, BindingId::new(0));
        assert_eq!(output.formula.right_binding, BindingId::new(0));
        assert_eq!(
            output.formula.left_result_input.head,
            TypeHeadInput::BuiltinSet
        );
        assert_eq!(
            output.formula.right_result_input.head,
            TypeHeadInput::BuiltinSet
        );
        assert!(output.formula.left_expected_input.is_none());
        assert_eq!(
            output.formula.right_expected_input.as_ref().unwrap().head,
            TypeHeadInput::BuiltinSet
        );
        assert_eq!(output.formula.term_formula.terms().len(), 2);
        assert_eq!(output.formula.term_formula.type_entries().len(), 5);
        assert_eq!(output.formula.term_formula.normalized_types().len(), 1);
        assert!(output.formula.term_formula.candidate_sets().is_empty());
        assert!(output.formula.term_formula.facts().is_empty());
        assert!(output.formula.term_formula.diagnostics().is_empty());
        let (_, normalized) = output
            .formula
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("right-parenthesized membership normalized set identity should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_eq!(normalized.source.spelling, "set");
        assert_eq!(
            normalized.source.range,
            output.formula.payload.reserve.bridge.bindings()[0].type_range
        );
        let (_, checked_formula) = output
            .formula
            .term_formula
            .formulas()
            .iter()
            .next()
            .expect("right-parenthesized membership formula should be checked");
        assert_eq!(checked_formula.kind, FormulaKind::Membership);
        assert_eq!(checked_formula.status, FormulaStatus::Checked);
        assert_eq!(
            checked_formula.terms,
            [
                output.formula.payload.left_site.clone(),
                output.formula.payload.right_site.clone(),
            ]
        );
        assert_eq!(checked_formula.expected_types.len(), 1);
        assert_eq!(
            checked_formula.expected_types[0].term,
            output.formula.payload.right_site
        );
        assert!(checked_formula.facts.is_empty());
        assert!(checked_formula.deferred.is_empty());
        let roles = output
            .formula
            .term_formula
            .type_entries()
            .iter()
            .filter_map(|(_, entry)| match &entry.owner {
                TypedSiteRef::Role { role, .. } => Some(role.as_str().to_owned()),
                _ => None,
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(
            roles,
            BTreeSet::from([
                "right-parenthesized-reserved-variable-membership-left-result".to_owned(),
                "right-parenthesized-reserved-variable-membership-right-expected".to_owned(),
                "right-parenthesized-reserved-variable-membership-right-result".to_owned(),
            ])
        );
        assert!(
            output
                .formula
                .term_formula
                .terms()
                .iter()
                .all(|(_, term)| term.site.node() != output.wrapper_site.node())
        );
        assert!(
            output
                .formula
                .term_formula
                .type_entries()
                .iter()
                .all(|(_, entry)| entry.owner.node() != output.wrapper_site.node())
        );
        assert!(
            output
                .formula
                .term_formula
                .formulas()
                .iter()
                .all(
                    |(_, formula)| formula.site.node() != output.wrapper_site.node()
                        && formula
                            .terms
                            .iter()
                            .all(|term| term.node() != output.wrapper_site.node())
                )
        );

        let invalid_key = super::TYPE_ELABORATION_RIGHT_PARENTHESIZED_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY;
        let output_for_corruption = || {
            super::source_right_parenthesized_reserved_variable_membership_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce a Task 245 corruption target")
        };
        let assert_invalid = |output| {
            assert_eq!(
                super::source_right_parenthesized_reserved_variable_membership_output_detail_keys(
                    output,
                ),
                vec![invalid_key.to_owned()]
            );
        };
        let mut wrong_side = output_for_corruption();
        wrong_side.wrapper_side = super::SourceParenthesizedOperandSide::Left;
        assert_invalid(&wrong_side);
        let mut stale_source_side = output_for_corruption();
        stale_source_side.source_wrapper_side = super::SourceParenthesizedOperandSide::Left;
        assert_invalid(&stale_source_side);
        let mut collapsed_wrapper_site = output_for_corruption();
        collapsed_wrapper_site.wrapper_site =
            collapsed_wrapper_site.formula.payload.right_site.clone();
        assert_invalid(&collapsed_wrapper_site);
        let mut stale_source_site = output_for_corruption();
        stale_source_site.source_wrapper_site = stale_source_site.formula.payload.left_site.clone();
        assert_invalid(&stale_source_site);
        let mut collapsed_wrapper_range = output_for_corruption();
        collapsed_wrapper_range.wrapper_range = collapsed_wrapper_range.formula.payload.right_range;
        assert_invalid(&collapsed_wrapper_range);
        let mut stale_source_range = output_for_corruption();
        stale_source_range.source_wrapper_range = stale_source_range.formula.payload.right_range;
        assert_invalid(&stale_source_range);
        let mut wrong_right_inner_range = output_for_corruption();
        wrong_right_inner_range.formula.payload.right_range = wrong_right_inner_range.wrapper_range;
        assert_invalid(&wrong_right_inner_range);
        let mut reversed_order = output_for_corruption();
        reversed_order.formula.payload.left_range = reversed_order.formula.payload.right_range;
        assert_invalid(&reversed_order);
        let mut collapsed_formula_site = output_for_corruption();
        collapsed_formula_site.formula.payload.formula_site =
            collapsed_formula_site.wrapper_site.clone();
        assert_invalid(&collapsed_formula_site);
        let mut collapsed_formula_range = output_for_corruption();
        collapsed_formula_range.formula.payload.formula_range =
            collapsed_formula_range.formula.payload.left_range;
        assert_invalid(&collapsed_formula_range);
        let mut wrong_binding = output_for_corruption();
        wrong_binding.formula.right_binding = BindingId::new(1);
        assert_invalid(&wrong_binding);
        let mut missing_expected = output_for_corruption();
        missing_expected.formula.right_expected_input = None;
        assert_invalid(&missing_expected);
        let mut unexpected_left_expected = output_for_corruption();
        unexpected_left_expected.formula.left_expected_input = unexpected_left_expected
            .formula
            .right_expected_input
            .clone();
        assert_invalid(&unexpected_left_expected);
        let mut wrapper_owned_expected = output_for_corruption();
        wrapper_owned_expected
            .formula
            .right_expected_input
            .as_mut()
            .unwrap()
            .site = TypedSiteRef::Role {
            node: wrapper_owned_expected.wrapper_site.node(),
            role: super::TypeRole::new(
                "right-parenthesized-reserved-variable-membership-right-expected",
            ),
        };
        assert_invalid(&wrapper_owned_expected);
        let mut wrong_expected_range = output_for_corruption();
        wrong_expected_range
            .formula
            .right_expected_input
            .as_mut()
            .unwrap()
            .source_range = wrong_expected_range.formula.payload.right_range;
        assert_invalid(&wrong_expected_range);
        let mut swapped_formula_sites = output_for_corruption();
        std::mem::swap(
            &mut swapped_formula_sites.formula.payload.left_site,
            &mut swapped_formula_sites.formula.payload.right_site,
        );
        assert_invalid(&swapped_formula_sites);

        let task_243_source = reserve_then_parenthesized_identifier_binary_theorem_ast(
            source_id,
            reserve(),
            ParenthesizedIdentifierBinaryTheoremSpec {
                label: "ParenthesizedReservedVariableMembershipPayloadBoundary",
                left: parenthesized("x"),
                right: ParenthesizedIdentifierOperandShape::Direct("x"),
                ..exact_spec
            },
        );
        let task_243_output = super::source_parenthesized_reserved_variable_membership_output(
            &task_243_source,
            module.clone(),
            &symbols,
        )
        .expect("Task 243 source should retain its Left owner");
        assert_invalid(&task_243_output);
        assert_eq!(
            super::source_parenthesized_reserved_variable_membership_output_detail_keys(&output),
            vec![super::TYPE_ELABORATION_PARENTHESIZED_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY.to_owned()]
        );

        let mut wrong_payload_side =
            super::extract_source_right_parenthesized_reserved_variable_membership(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce a side-corruption target");
        wrong_payload_side.wrapper_side = super::SourceParenthesizedOperandSide::Left;
        assert_eq!(
            super::source_parenthesized_reserved_variable_binary_formula_payload_detail_keys(
                wrong_payload_side,
                &symbols,
                &super::SOURCE_RIGHT_PARENTHESIZED_RESERVED_VARIABLE_MEMBERSHIP_CONFIG,
                super::SourceParenthesizedOperandSide::Right,
            ),
            vec![invalid_key.to_owned()]
        );
        let mut wrong_config =
            super::extract_source_right_parenthesized_reserved_variable_membership(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce a config-corruption target");
        wrong_config.formula.config =
            &super::SOURCE_PARENTHESIZED_RESERVED_VARIABLE_MEMBERSHIP_CONFIG;
        assert_eq!(
            super::source_parenthesized_reserved_variable_binary_formula_payload_detail_keys(
                wrong_config,
                &symbols,
                &super::SOURCE_RIGHT_PARENTHESIZED_RESERVED_VARIABLE_MEMBERSHIP_CONFIG,
                super::SourceParenthesizedOperandSide::Right,
            ),
            vec![invalid_key.to_owned()]
        );
        for collapse_to in [
            payload.formula.left_site.clone(),
            payload.formula.right_site.clone(),
        ] {
            let mut collapsed_formula_operand =
                super::extract_source_right_parenthesized_reserved_variable_membership(
                    &exact,
                    module.clone(),
                    &symbols,
                )
                .expect("exact source should produce a formula-site corruption target");
            collapsed_formula_operand.formula.formula_site = collapse_to;
            assert_eq!(
                super::source_parenthesized_reserved_variable_binary_formula_payload_detail_keys(
                    collapsed_formula_operand,
                    &symbols,
                    &super::SOURCE_RIGHT_PARENTHESIZED_RESERVED_VARIABLE_MEMBERSHIP_CONFIG,
                    super::SourceParenthesizedOperandSide::Right,
                ),
                vec![invalid_key.to_owned()]
            );
        }
        let mut immutable_payload =
            super::extract_source_right_parenthesized_reserved_variable_membership(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce an immutable-output target");
        immutable_payload.wrapper_range = immutable_payload.formula.right_range;
        let immutable_output =
            super::build_source_parenthesized_reserved_variable_binary_formula_output(
                immutable_payload,
                &symbols,
            )
            .expect("wrapper-only corruption must not mutate checker payloads");
        assert_eq!(immutable_output.formula.term_formula.terms().len(), 2);
        assert_eq!(
            immutable_output.formula.term_formula.type_entries().len(),
            5
        );
        assert!(immutable_output.formula.left_expected_input.is_none());
        assert!(immutable_output.formula.right_expected_input.is_some());
        let (_, immutable_formula) = immutable_output
            .formula
            .term_formula
            .formulas()
            .iter()
            .next()
            .expect("immutable checker payload should retain one formula");
        assert_eq!(immutable_formula.kind, FormulaKind::Membership);
        assert_eq!(immutable_formula.status, FormulaStatus::Checked);
        assert_eq!(immutable_formula.expected_types.len(), 1);
        assert_eq!(
            immutable_formula.expected_types[0].term,
            immutable_output.formula.payload.right_site
        );
        assert_invalid(&immutable_output);
        let mismatched_symbols = SymbolEnv::new(
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("other_module")),
            SymbolEnvIndexes::default(),
        );
        let mismatched_payload =
            super::extract_source_right_parenthesized_reserved_variable_membership(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce a module-corruption target");
        assert!(
            super::build_source_parenthesized_reserved_variable_binary_formula_output(
                mismatched_payload,
                &mismatched_symbols,
            )
            .is_err()
        );

        let direct_task_120 = reserve_then_identifier_binary_theorem_ast(
            source_id,
            reserve(),
            "ReservedVariableMembershipPayloadBoundary",
            "x",
            "in",
            "x",
        );
        assert!(
            super::extract_source_right_parenthesized_reserved_variable_membership(
                &direct_task_120,
                module.clone(),
                &symbols,
            )
            .is_none()
        );
        assert!(
            super::extract_source_reserved_variable_membership(
                &direct_task_120,
                module.clone(),
                &symbols,
            )
            .is_some()
        );

        let near_miss_specs = [
            ParenthesizedIdentifierBinaryTheoremSpec {
                right: ParenthesizedIdentifierOperandShape::Direct("x"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: parenthesized("x"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: parenthesized("x"),
                right: ParenthesizedIdentifierOperandShape::Direct("x"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                right: ParenthesizedIdentifierOperandShape::Identifier {
                    spelling: "x",
                    depth: 2,
                    recovered: false,
                    open: "(",
                    close: ")",
                },
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                right: parenthesized("y"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: ParenthesizedIdentifierOperandShape::Direct("y"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                right: ParenthesizedIdentifierOperandShape::Identifier {
                    spelling: "x",
                    depth: 1,
                    recovered: true,
                    open: "(",
                    close: ")",
                },
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                right: ParenthesizedIdentifierOperandShape::Identifier {
                    spelling: "x",
                    depth: 1,
                    recovered: false,
                    open: "[",
                    close: "]",
                },
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                right: ParenthesizedIdentifierOperandShape::Numeral("1"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                right: ParenthesizedIdentifierOperandShape::Empty,
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                right: ParenthesizedIdentifierOperandShape::DoubleIdentifier("x"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                label: "OtherPayloadBoundary",
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                operator: "=",
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                operator: "<>",
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                status: Some("registration"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                recovered_label: true,
                ..exact_spec
            },
        ];
        let near_misses = near_miss_specs
            .into_iter()
            .map(|spec| {
                reserve_then_parenthesized_identifier_binary_theorem_ast(source_id, reserve(), spec)
            })
            .chain([
                reserve_then_parenthesized_identifier_binary_theorem_ast(
                    source_id,
                    vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("object"))],
                    exact_spec,
                ),
                reserve_then_parenthesized_identifier_binary_theorem_ast(
                    source_id,
                    vec![reserve_item(
                        vec!["x", "y"],
                        ReserveTypeShape::Builtin("set"),
                    )],
                    exact_spec,
                ),
                reserve_then_parenthesized_identifier_binary_theorem_ast(
                    source_id,
                    vec![reserve_item(vec!["x"], ReserveTypeShape::AttributedSet)],
                    exact_spec,
                ),
                reserve_then_parenthesized_identifier_binary_theorem_ast(
                    source_id,
                    vec![
                        reserve_item(vec!["x"], ReserveTypeShape::Builtin("set")),
                        reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                    ],
                    exact_spec,
                ),
            ]);
        for near_miss in near_misses {
            assert!(
                super::extract_source_right_parenthesized_reserved_variable_membership(
                    &near_miss,
                    module.clone(),
                    &symbols,
                )
                .is_none()
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
    }

    #[test]
    fn source_parenthesized_two_edge_local_mode_reserved_variable_equality_bridge_is_transparent() {
        let source_id = source_id(246);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("parenthesized_two_edge_local_mode_reserved_variable_equality"),
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
        let parenthesized = |spelling| ParenthesizedIdentifierOperandShape::Identifier {
            spelling,
            depth: 1,
            recovered: false,
            open: "(",
            close: ")",
        };
        let exact_spec = ParenthesizedIdentifierBinaryTheoremSpec {
            status: None,
            label: "ParenthesizedTwoEdgeLocalModeReservedVariableEqualityPayloadBoundary",
            left: parenthesized("z"),
            operator: "=",
            right: ParenthesizedIdentifierOperandShape::Direct("z"),
            recovered_label: false,
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
        let reserve = || {
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("OuterTwoEdgeModeEquality"),
            )]
        };
        let exact = mode_then_reserve_parenthesized_identifier_binary_theorem_ast(
            source_id,
            exact_modes(),
            reserve(),
            exact_spec,
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        let payload =
            super::extract_source_parenthesized_two_edge_local_mode_reserved_variable_equality(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact Task 246 source should extract");
        assert_eq!(
            payload.wrapper_side,
            super::SourceParenthesizedOperandSide::Left
        );
        assert_eq!(payload.formula.left_lookup_ordinal, 1);
        assert_eq!(payload.formula.right_lookup_ordinal, 2);
        assert_eq!(payload.formula.reserve.mode_expansions.len(), 3);
        assert_eq!(payload.formula.reserve.bridge.bindings().len(), 1);
        assert_eq!(
            payload.formula.reserve.bridge.bindings()[0].type_spelling,
            "OuterTwoEdgeModeEquality"
        );
        let sites = [
            payload.wrapper_site.clone(),
            payload.formula.formula_site.clone(),
            payload.formula.left_site.clone(),
            payload.formula.right_site.clone(),
        ];
        for left in 0..sites.len() {
            for right in left + 1..sites.len() {
                assert_ne!(sites[left], sites[right]);
            }
        }
        let expansion_ranges = payload
            .formula
            .reserve
            .mode_expansions
            .values()
            .map(|expansion| expansion.radix.source_range)
            .collect::<Vec<_>>();
        assert_eq!(expansion_ranges.len(), 3);
        for left in 0..expansion_ranges.len() {
            for right in left + 1..expansion_ranges.len() {
                assert_ne!(expansion_ranges[left], expansion_ranges[right]);
            }
        }

        let output =
            super::source_parenthesized_two_edge_local_mode_reserved_variable_equality_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact Task 246 source should reach the real checker consumer");
        super::assert_source_parenthesized_two_edge_local_mode_reserved_variable_equality_output(
            &output,
        )
        .expect("Task 246 output invariants should hold");
        assert_eq!(output.formula.left_binding, BindingId::new(0));
        assert_eq!(output.formula.right_binding, BindingId::new(0));
        assert_eq!(output.formula.term_formula.terms().len(), 2);
        assert_eq!(output.formula.term_formula.type_entries().len(), 6);
        assert_eq!(output.formula.term_formula.normalized_types().len(), 1);
        for input in [
            &output.formula.left_result_input,
            &output.formula.right_result_input,
            output
                .formula
                .left_expected_input
                .as_ref()
                .expect("left expected input should exist"),
            output
                .formula
                .right_expected_input
                .as_ref()
                .expect("right expected input should exist"),
        ] {
            assert_eq!(input.spelling, "OuterTwoEdgeModeEquality");
            assert!(matches!(input.head, TypeHeadInput::Symbol(_)));
        }
        let formula = output
            .formula
            .term_formula
            .formulas()
            .iter()
            .map(|(_, formula)| formula)
            .next()
            .expect("Task 246 checked equality should exist");
        assert_eq!(formula.kind, FormulaKind::Equality);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert_eq!(formula.expected_types.len(), 2);
        assert_eq!(
            formula.expected_types[0].term,
            output.formula.payload.left_site
        );
        assert_eq!(
            formula.expected_types[1].term,
            output.formula.payload.right_site
        );
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
        let terminal = output
            .formula
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("BaseTwoEdgeModeEquality")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("Task 246 base terminal expansion should exist");
        let (_, normalized) = output
            .formula
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 246 normalized set identity should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);

        let invalid_key = super::TYPE_ELABORATION_PARENTHESIZED_TWO_EDGE_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY;
        let output_for_corruption = || {
            super::source_parenthesized_two_edge_local_mode_reserved_variable_equality_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact Task 246 source should produce a corruption target")
        };
        let assert_invalid =
            |output: &super::SourceParenthesizedReservedVariableBinaryFormulaOutput| {
                assert_eq!(
                super::source_parenthesized_two_edge_local_mode_reserved_variable_equality_output_detail_keys(output),
                vec![invalid_key.to_owned()]
            );
            };
        let mut wrong_side = output_for_corruption();
        wrong_side.wrapper_side = super::SourceParenthesizedOperandSide::Right;
        assert_invalid(&wrong_side);
        let mut wrong_source_side = output_for_corruption();
        wrong_source_side.source_wrapper_side = super::SourceParenthesizedOperandSide::Right;
        assert_invalid(&wrong_source_side);
        let mut wrong_config = output_for_corruption();
        wrong_config.formula.payload.config =
            &super::SOURCE_TWO_EDGE_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_CONFIG;
        assert_invalid(&wrong_config);
        let mut collapsed_wrapper = output_for_corruption();
        collapsed_wrapper.wrapper_site = collapsed_wrapper.formula.payload.left_site.clone();
        assert_invalid(&collapsed_wrapper);
        let mut collapsed_formula = output_for_corruption();
        collapsed_formula.formula.payload.formula_site = collapsed_formula.wrapper_site.clone();
        assert_invalid(&collapsed_formula);
        let mut collapsed_operands = output_for_corruption();
        collapsed_operands.formula.payload.right_site =
            collapsed_operands.formula.payload.left_site.clone();
        assert_invalid(&collapsed_operands);
        let mut wrong_wrapper_range = output_for_corruption();
        wrong_wrapper_range.wrapper_range = wrong_wrapper_range.formula.payload.left_range;
        assert_invalid(&wrong_wrapper_range);
        let mut wrong_binding = output_for_corruption();
        wrong_binding.formula.right_binding = BindingId::new(1);
        assert_invalid(&wrong_binding);
        let mut missing_expected = output_for_corruption();
        missing_expected.formula.left_expected_input = None;
        assert_invalid(&missing_expected);
        let mut wrong_result_role = output_for_corruption();
        wrong_result_role.formula.left_result_input.site = TypedSiteRef::Role {
            node: wrong_result_role.formula.payload.left_site.node(),
            role: super::TypeRole::new("wrong-role"),
        };
        assert_invalid(&wrong_result_role);
        let mut wrong_expected_range = output_for_corruption();
        wrong_expected_range
            .formula
            .right_expected_input
            .as_mut()
            .expect("right expected input should exist")
            .source_range = wrong_expected_range.formula.payload.right_range;
        assert_invalid(&wrong_expected_range);
        for spelling in [
            "BaseTwoEdgeModeEquality",
            "MiddleTwoEdgeModeEquality",
            "OuterTwoEdgeModeEquality",
        ] {
            let mut missing_expansion = output_for_corruption();
            missing_expansion
                .formula
                .payload
                .reserve
                .mode_expansions
                .retain(|symbol, _| source_mode_symbol_spelling(symbol) != Some(spelling));
            assert_invalid(&missing_expansion);
        }

        let mut immutable_payload =
            super::extract_source_parenthesized_two_edge_local_mode_reserved_variable_equality(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact Task 246 source should produce an immutable-output target");
        immutable_payload.wrapper_range = immutable_payload.formula.left_range;
        let immutable_output =
            super::build_source_parenthesized_reserved_variable_binary_formula_output(
                immutable_payload,
                &symbols,
            )
            .expect("wrapper-only corruption must not mutate real checker payloads");
        assert_eq!(immutable_output.formula.term_formula.terms().len(), 2);
        assert_eq!(
            immutable_output.formula.term_formula.type_entries().len(),
            6
        );
        assert_eq!(
            immutable_output
                .formula
                .payload
                .reserve
                .mode_expansions
                .len(),
            3
        );
        assert!(immutable_output.formula.left_expected_input.is_some());
        assert!(immutable_output.formula.right_expected_input.is_some());
        let immutable_terminal = immutable_output
            .formula
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("BaseTwoEdgeModeEquality")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("immutable output should retain the Base RHS terminal");
        let (_, immutable_normalized) = immutable_output
            .formula
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("immutable output should retain one normalized set identity");
        assert_eq!(immutable_normalized.head, TypeHeadRef::BuiltinSet);
        assert_eq!(
            immutable_normalized.source.range,
            immutable_terminal.source_range
        );
        assert_eq!(immutable_output.formula.term_formula.formulas().len(), 1);
        let (_, immutable_formula) = immutable_output
            .formula
            .term_formula
            .formulas()
            .iter()
            .next()
            .expect("immutable output should retain one checked equality");
        assert_eq!(immutable_formula.kind, FormulaKind::Equality);
        assert_eq!(immutable_formula.status, FormulaStatus::Checked);
        assert_eq!(immutable_formula.expected_types.len(), 2);
        assert_eq!(
            immutable_formula.expected_types[0].term,
            immutable_output.formula.payload.left_site
        );
        assert_eq!(
            immutable_formula.expected_types[1].term,
            immutable_output.formula.payload.right_site
        );
        assert!(immutable_formula.facts.is_empty());
        assert!(immutable_formula.deferred.is_empty());
        assert_invalid(&immutable_output);
        let mismatched_symbols = SymbolEnv::new(
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("other_module")),
            SymbolEnvIndexes::default(),
        );
        let mismatched_payload =
            super::extract_source_parenthesized_two_edge_local_mode_reserved_variable_equality(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact Task 246 source should produce a module-corruption target");
        assert!(
            super::build_source_parenthesized_reserved_variable_binary_formula_output(
                mismatched_payload,
                &mismatched_symbols,
            )
            .is_err()
        );

        let task_134 = mode_then_reserve_identifier_binary_theorem_ast(
            source_id,
            exact_modes(),
            reserve(),
            IdentifierBinaryTheoremSpec {
                status: None,
                label: "TwoEdgeLocalModeReservedVariableEqualityPayloadBoundary",
                left: "z",
                operator: "=",
                right: "z",
                recovered_label: false,
            },
        );
        assert!(
            super::extract_source_two_edge_local_mode_reserved_variable_equality(
                &task_134,
                module.clone(),
                &symbols,
            )
            .is_some()
        );
        assert!(
            super::extract_source_parenthesized_two_edge_local_mode_reserved_variable_equality(
                &task_134,
                module.clone(),
                &symbols,
            )
            .is_none()
        );
        assert!(
            super::extract_source_two_edge_local_mode_reserved_variable_equality(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none()
        );
        let task_223 = reserve_then_parenthesized_identifier_binary_theorem_ast(
            source_id,
            vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
            ParenthesizedIdentifierBinaryTheoremSpec {
                status: None,
                label: "ParenthesizedReservedVariableEqualityPayloadBoundary",
                left: parenthesized("x"),
                operator: "=",
                right: ParenthesizedIdentifierOperandShape::Direct("x"),
                recovered_label: false,
            },
        );
        assert!(
            super::extract_source_parenthesized_reserved_variable_equality(
                &task_223,
                module.clone(),
                &symbols,
            )
            .is_some()
        );
        assert!(
            super::extract_source_parenthesized_two_edge_local_mode_reserved_variable_equality(
                &task_223,
                module.clone(),
                &symbols,
            )
            .is_none()
        );
        assert!(
            super::extract_source_parenthesized_reserved_variable_equality(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none()
        );

        for order in [[0, 2, 1], [1, 0, 2], [1, 2, 0], [2, 0, 1], [2, 1, 0]] {
            let modes = exact_modes();
            let permuted = mode_then_reserve_parenthesized_identifier_binary_theorem_ast(
                source_id,
                order.map(|index| modes[index]),
                reserve(),
                exact_spec,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&permuted, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }

        let near_miss_modes = [
            vec![mode_definition(
                "OuterTwoEdgeModeEquality",
                ReserveTypeShape::Builtin("set"),
            )],
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
            ],
            {
                let mut modes = exact_modes();
                modes.push(modes[1]);
                modes
            },
            {
                let mut modes = exact_modes();
                modes.insert(
                    2,
                    mode_definition(
                        "ExtraTwoEdgeModeEquality",
                        ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeModeEquality"),
                    ),
                );
                modes
            },
            vec![
                mode_definition(
                    "BaseTwoEdgeModeEquality",
                    ReserveTypeShape::Builtin("object"),
                ),
                exact_modes()[1],
                exact_modes()[2],
            ],
            vec![
                mode_definition("BaseTwoEdgeModeEquality", ReserveTypeShape::AttributedSet),
                exact_modes()[1],
                exact_modes()[2],
            ],
            vec![
                exact_modes()[0],
                mode_definition(
                    "MiddleTwoEdgeModeEquality",
                    ReserveTypeShape::QualifiedSymbol("OuterTwoEdgeModeEquality"),
                ),
                exact_modes()[2],
            ],
            vec![
                exact_modes()[0],
                exact_modes()[1],
                mode_definition(
                    "OuterTwoEdgeModeEquality",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeModeEquality"),
                ),
            ],
            vec![
                contextual_mode_definition(
                    "BaseTwoEdgeModeEquality",
                    ReserveTypeShape::Builtin("set"),
                ),
                exact_modes()[1],
                exact_modes()[2],
            ],
            vec![
                exact_modes()[0],
                parameterized_mode_definition(
                    "MiddleTwoEdgeModeEquality",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeModeEquality"),
                ),
                exact_modes()[2],
            ],
            vec![
                exact_modes()[0],
                recovered_mode_definition(
                    "MiddleTwoEdgeModeEquality",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeModeEquality"),
                ),
                exact_modes()[2],
            ],
            vec![
                exact_modes()[0],
                exact_modes()[1],
                mode_definition(
                    "OuterTwoEdgeModeEquality",
                    ReserveTypeShape::QualifiedSymbolWithArgs("MiddleTwoEdgeModeEquality"),
                ),
            ],
            {
                let mut modes = exact_modes();
                modes.insert(
                    2,
                    mode_definition(
                        "ExtraTwoEdgeModeEquality",
                        ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeModeEquality"),
                    ),
                );
                modes[3] = mode_definition(
                    "OuterTwoEdgeModeEquality",
                    ReserveTypeShape::QualifiedSymbol("ExtraTwoEdgeModeEquality"),
                );
                modes
            },
        ];
        for modes in near_miss_modes {
            let near_miss = mode_then_reserve_parenthesized_identifier_binary_theorem_ast(
                source_id,
                modes,
                reserve(),
                exact_spec,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }

        let near_miss_specs = [
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: ParenthesizedIdentifierOperandShape::Direct("z"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: ParenthesizedIdentifierOperandShape::Direct("z"),
                right: parenthesized("z"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                right: parenthesized("z"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: ParenthesizedIdentifierOperandShape::Identifier {
                    spelling: "z",
                    depth: 2,
                    recovered: false,
                    open: "(",
                    close: ")",
                },
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: ParenthesizedIdentifierOperandShape::Identifier {
                    spelling: "z",
                    depth: 1,
                    recovered: true,
                    open: "(",
                    close: ")",
                },
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: ParenthesizedIdentifierOperandShape::Identifier {
                    spelling: "z",
                    depth: 1,
                    recovered: false,
                    open: "[",
                    close: "]",
                },
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: ParenthesizedIdentifierOperandShape::Numeral("1"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: ParenthesizedIdentifierOperandShape::Empty,
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: ParenthesizedIdentifierOperandShape::DoubleIdentifier("z"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: parenthesized("x"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                right: ParenthesizedIdentifierOperandShape::Direct("x"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                label: "OtherPayloadBoundary",
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                operator: "<>",
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                status: Some("registration"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                recovered_label: true,
                ..exact_spec
            },
        ];
        for spec in near_miss_specs {
            let near_miss = mode_then_reserve_parenthesized_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                spec,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
        for near_miss in [
            mode_then_reserve_parenthesized_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(
                    vec!["z"],
                    ReserveTypeShape::QualifiedSymbolWithArgs("OuterTwoEdgeModeEquality"),
                )],
                exact_spec,
            ),
            mode_then_reserve_parenthesized_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
                exact_spec,
            ),
            mode_then_reserve_parenthesized_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                vec![
                    reserve_item(
                        vec!["z"],
                        ReserveTypeShape::QualifiedSymbol("OuterTwoEdgeModeEquality"),
                    ),
                    reserve_item(vec!["x"], ReserveTypeShape::Builtin("set")),
                ],
                exact_spec,
            ),
            modes_then_empty_definition_reserve_parenthesized_identifier_binary_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                exact_spec,
            ),
        ] {
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }

        let old_parenthesized = reserve_then_parenthesized_identifier_binary_theorem_ast(
            source_id,
            vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
            ParenthesizedIdentifierBinaryTheoremSpec {
                status: None,
                label: "ParenthesizedReservedVariableEqualityPayloadBoundary",
                left: parenthesized("x"),
                operator: "=",
                right: ParenthesizedIdentifierOperandShape::Direct("x"),
                recovered_label: false,
            },
        );
        assert!(
            super::extract_source_parenthesized_reserved_variable_equality(
                &old_parenthesized,
                module.clone(),
                &symbols,
            )
            .is_some(),
            "empty-mode configs must retain their exact route"
        );
        let old_with_modes = mode_then_reserve_parenthesized_identifier_binary_theorem_ast(
            source_id,
            exact_modes(),
            vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
            ParenthesizedIdentifierBinaryTheoremSpec {
                status: None,
                label: "ParenthesizedReservedVariableEqualityPayloadBoundary",
                left: parenthesized("x"),
                operator: "=",
                right: ParenthesizedIdentifierOperandShape::Direct("x"),
                recovered_label: false,
            },
        );
        assert!(
            super::extract_source_parenthesized_reserved_variable_equality(
                &old_with_modes,
                module,
                &symbols,
            )
            .is_none(),
            "empty-mode configs must still reject mode-definition nodes"
        );
    }

    #[test]
    fn source_parenthesized_heterogeneous_reserve_membership_bridge_is_transparent() {
        let source_id = source_id(244);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("parenthesized_heterogeneous_reserve_membership"),
        );
        let symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        let reserve = || {
            vec![
                reserve_item(vec!["x"], ReserveTypeShape::Builtin("object")),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
            ]
        };
        let parenthesized = |spelling| ParenthesizedIdentifierOperandShape::Identifier {
            spelling,
            depth: 1,
            recovered: false,
            open: "(",
            close: ")",
        };
        let exact_spec = ParenthesizedIdentifierBinaryTheoremSpec {
            status: None,
            label: "ParenthesizedHeterogeneousReserveMembershipPayloadBoundary",
            left: parenthesized("x"),
            operator: "in",
            right: ParenthesizedIdentifierOperandShape::Direct("y"),
            recovered_label: false,
        };
        let exact = reserve_then_parenthesized_identifier_binary_theorem_ast(
            source_id,
            reserve(),
            exact_spec,
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        let payload = super::extract_source_parenthesized_heterogeneous_reserve_membership(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact parenthesized heterogeneous membership should extract");
        for extractor in [
            super::extract_source_parenthesized_reserved_variable_equality,
            super::extract_source_parenthesized_reserved_object_variable_equality,
            super::extract_source_parenthesized_reserved_variable_inequality,
            super::extract_source_parenthesized_reserved_object_variable_inequality,
            super::extract_source_parenthesized_reserved_variable_membership,
        ] {
            assert!(extractor(&exact, module.clone(), &symbols).is_none());
        }
        let bindings = payload.formula.reserve.bridge.bindings();
        assert_eq!(bindings.len(), 2);
        assert_eq!(bindings[0].spelling, "x");
        assert_eq!(bindings[0].type_head, TypeHeadInput::BuiltinObject);
        assert_eq!(bindings[1].spelling, "y");
        assert_eq!(bindings[1].type_head, TypeHeadInput::BuiltinSet);
        assert_ne!(bindings[0].type_range, bindings[1].type_range);
        assert!(bindings[0].type_range.start < bindings[1].type_range.start);
        assert_eq!(payload.formula.left_lookup_ordinal, 2);
        assert_eq!(payload.formula.right_lookup_ordinal, 3);
        assert_ne!(payload.wrapper_site, payload.formula.left_site);
        assert_ne!(payload.wrapper_site, payload.formula.right_site);
        assert!(payload.wrapper_range.start < payload.formula.left_range.start);
        assert!(payload.wrapper_range.end > payload.formula.left_range.end);
        assert!(payload.wrapper_range.end <= payload.formula.right_range.start);

        let output = super::source_parenthesized_heterogeneous_reserve_membership_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should reach the heterogeneous membership consumer");
        super::assert_source_parenthesized_heterogeneous_reserve_membership_output(&output)
            .expect("Task 244 output invariants should hold");
        assert_eq!(output.wrapper_site, output.source_wrapper_site);
        assert_eq!(output.wrapper_range, output.source_wrapper_range);
        assert_eq!(output.formula.left_binding, BindingId::new(0));
        assert_eq!(output.formula.right_binding, BindingId::new(1));
        assert_eq!(
            output.formula.left_result_input.head,
            TypeHeadInput::BuiltinObject
        );
        assert_eq!(
            output.formula.right_result_input.head,
            TypeHeadInput::BuiltinSet
        );
        assert!(output.formula.left_expected_input.is_none());
        assert_eq!(
            output.formula.right_expected_input.as_ref().unwrap().head,
            TypeHeadInput::BuiltinSet
        );
        assert_eq!(output.formula.term_formula.terms().len(), 2);
        assert_eq!(output.formula.term_formula.type_entries().len(), 5);
        assert_eq!(output.formula.term_formula.normalized_types().len(), 2);
        assert!(output.formula.term_formula.candidate_sets().is_empty());
        assert!(output.formula.term_formula.facts().is_empty());
        assert!(output.formula.term_formula.diagnostics().is_empty());
        let normalized_by_head = output
            .formula
            .term_formula
            .normalized_types()
            .iter()
            .map(|(_, normalized)| (normalized.head.clone(), normalized))
            .collect::<BTreeMap<_, _>>();
        let normalized_object = normalized_by_head
            .get(&TypeHeadRef::BuiltinObject)
            .expect("object identity should remain distinct");
        assert_eq!(normalized_object.source.spelling, "object");
        assert_eq!(normalized_object.source.range, bindings[0].type_range);
        let normalized_set = normalized_by_head
            .get(&TypeHeadRef::BuiltinSet)
            .expect("set identity should remain distinct");
        assert_eq!(normalized_set.source.spelling, "set");
        assert_eq!(normalized_set.source.range, bindings[1].type_range);
        let (_, checked_formula) = output
            .formula
            .term_formula
            .formulas()
            .iter()
            .next()
            .expect("one membership formula should be checked");
        assert_eq!(checked_formula.kind, FormulaKind::Membership);
        assert_eq!(checked_formula.status, FormulaStatus::Checked);
        assert_eq!(checked_formula.expected_types.len(), 1);
        assert_eq!(
            checked_formula.expected_types[0].term,
            output.formula.payload.right_site
        );
        assert!(checked_formula.facts.is_empty());
        assert!(checked_formula.deferred.is_empty());
        let role_names = output
            .formula
            .term_formula
            .type_entries()
            .iter()
            .filter_map(|(_, entry)| match &entry.owner {
                TypedSiteRef::Role { role, .. } => Some(role.as_str().to_owned()),
                _ => None,
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(
            role_names,
            BTreeSet::from([
                "parenthesized-heterogeneous-reserve-membership-left-result".to_owned(),
                "parenthesized-heterogeneous-reserve-membership-right-expected".to_owned(),
                "parenthesized-heterogeneous-reserve-membership-right-result".to_owned(),
            ])
        );
        assert!(
            output
                .formula
                .term_formula
                .terms()
                .iter()
                .all(|(_, term)| term.site.node() != output.wrapper_site.node())
        );
        assert!(
            output
                .formula
                .term_formula
                .type_entries()
                .iter()
                .all(|(_, entry)| entry.owner.node() != output.wrapper_site.node())
        );
        assert!(
            output
                .formula
                .term_formula
                .formulas()
                .iter()
                .all(
                    |(_, formula)| formula.site.node() != output.wrapper_site.node()
                        && formula
                            .terms
                            .iter()
                            .all(|term| term.node() != output.wrapper_site.node())
                )
        );

        let invalid_key = super::TYPE_ELABORATION_PARENTHESIZED_HETEROGENEOUS_RESERVE_MEMBERSHIP_INVALID_PAYLOAD_KEY;
        let output_for_corruption = || {
            super::source_parenthesized_heterogeneous_reserve_membership_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce a corruption target")
        };
        let assert_invalid = |output| {
            assert_eq!(
                super::source_parenthesized_heterogeneous_reserve_membership_output_detail_keys(
                    &output,
                ),
                vec![invalid_key.to_owned()]
            );
        };
        let mut collapsed_wrapper = output_for_corruption();
        collapsed_wrapper.wrapper_site = collapsed_wrapper.formula.payload.left_site.clone();
        assert_invalid(collapsed_wrapper);
        let mut stale_wrapper_range = output_for_corruption();
        stale_wrapper_range.source_wrapper_range = stale_wrapper_range.formula.payload.left_range;
        assert_invalid(stale_wrapper_range);
        let mut collapsed_inner = output_for_corruption();
        collapsed_inner.formula.payload.left_site = collapsed_inner.wrapper_site.clone();
        assert_invalid(collapsed_inner);
        let mut collapsed_right = output_for_corruption();
        collapsed_right.formula.payload.right_range = collapsed_right.formula.payload.left_range;
        assert_invalid(collapsed_right);
        let mut collapsed_formula = output_for_corruption();
        collapsed_formula.formula.payload.formula_site = collapsed_formula.wrapper_site.clone();
        assert_invalid(collapsed_formula);
        let mut collapsed_binding = output_for_corruption();
        collapsed_binding.formula.right_binding = BindingId::new(0);
        assert_invalid(collapsed_binding);
        let mut wrong_left_head = output_for_corruption();
        wrong_left_head.formula.left_result_input.head = TypeHeadInput::BuiltinSet;
        assert_invalid(wrong_left_head);
        let mut wrong_right_head = output_for_corruption();
        wrong_right_head.formula.right_result_input.head = TypeHeadInput::BuiltinObject;
        assert_invalid(wrong_right_head);
        let mut unexpected_left_expected = output_for_corruption();
        unexpected_left_expected.formula.left_expected_input = unexpected_left_expected
            .formula
            .right_expected_input
            .clone();
        assert_invalid(unexpected_left_expected);
        let mut wrong_right_expected = output_for_corruption();
        wrong_right_expected
            .formula
            .right_expected_input
            .as_mut()
            .unwrap()
            .head = TypeHeadInput::BuiltinObject;
        assert_invalid(wrong_right_expected);
        let mut missing_right_expected = output_for_corruption();
        missing_right_expected.formula.right_expected_input = None;
        assert_invalid(missing_right_expected);
        let mut collapsed_role = output_for_corruption();
        collapsed_role
            .formula
            .right_expected_input
            .as_mut()
            .unwrap()
            .site = collapsed_role.formula.right_result_input.site.clone();
        assert_invalid(collapsed_role);
        let mut wrong_right_source = output_for_corruption();
        wrong_right_source
            .formula
            .right_expected_input
            .as_mut()
            .unwrap()
            .source_range = wrong_right_source.formula.payload.right_range;
        assert_invalid(wrong_right_source);

        for swap_ranges in [false, true] {
            let mut corrupt = output_for_corruption();
            let (source_id, bridge_module, source_range, mut corrupt_bindings) = {
                let bridge = &corrupt.formula.payload.reserve.bridge;
                (
                    bridge.source_id(),
                    bridge.module_id().clone(),
                    bridge.source_range(),
                    bridge.bindings().to_vec(),
                )
            };
            if swap_ranges {
                let object_range = corrupt_bindings[0].type_range;
                corrupt_bindings[0].type_range = corrupt_bindings[1].type_range;
                corrupt_bindings[1].type_range = object_range;
            } else {
                corrupt_bindings[1].type_range = corrupt_bindings[0].type_range;
            }
            corrupt.formula.payload.reserve.bridge = super::SourceReserveDeclarationBridge::new(
                source_id,
                bridge_module,
                source_range,
                corrupt_bindings,
            )
            .expect("range corruption should remain a structurally valid bridge");
            assert_invalid(corrupt);
        }
        let mut wrong_ordinal =
            super::extract_source_parenthesized_heterogeneous_reserve_membership(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce an ordinal corruption target");
        wrong_ordinal.formula.right_lookup_ordinal = wrong_ordinal.formula.left_lookup_ordinal;
        assert_eq!(
            super::source_parenthesized_reserved_variable_binary_formula_payload_detail_keys(
                wrong_ordinal,
                &symbols,
                &super::SOURCE_PARENTHESIZED_HETEROGENEOUS_RESERVE_MEMBERSHIP_CONFIG,
                super::SourceParenthesizedOperandSide::Left,
            ),
            vec![invalid_key.to_owned()]
        );
        let mut wrong_config = output_for_corruption();
        wrong_config.formula.payload.config =
            &super::SOURCE_PARENTHESIZED_RESERVED_VARIABLE_MEMBERSHIP_CONFIG;
        assert_invalid(wrong_config);
        let mut immutable_payload =
            super::extract_source_parenthesized_heterogeneous_reserve_membership(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce an immutable-output target");
        immutable_payload.wrapper_range = immutable_payload.formula.left_range;
        let immutable_output =
            super::build_source_parenthesized_reserved_variable_binary_formula_output(
                immutable_payload,
                &symbols,
            )
            .expect("wrapper corruption must not mutate checker-owned output");
        assert_eq!(immutable_output.formula.term_formula.terms().len(), 2);
        assert_eq!(
            immutable_output.formula.term_formula.type_entries().len(),
            5
        );
        assert_invalid(immutable_output);
        let mismatched_symbols = SymbolEnv::new(
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("other_module")),
            SymbolEnvIndexes::default(),
        );
        let mismatched_payload =
            super::extract_source_parenthesized_heterogeneous_reserve_membership(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce a module corruption target");
        assert!(
            super::build_source_parenthesized_reserved_variable_binary_formula_output(
                mismatched_payload,
                &mismatched_symbols,
            )
            .is_err()
        );

        let direct_task_125 = reserve_then_identifier_binary_theorem_ast(
            source_id,
            reserve(),
            "HeterogeneousReserveMembershipPayloadBoundary",
            "x",
            "in",
            "y",
        );
        assert!(
            super::extract_source_heterogeneous_reserve_membership(
                &direct_task_125,
                module.clone(),
                &symbols,
            )
            .is_some()
        );
        assert!(
            super::extract_source_parenthesized_heterogeneous_reserve_membership(
                &direct_task_125,
                module.clone(),
                &symbols,
            )
            .is_none()
        );
        assert_eq!(
            source_type_elaboration_detail_keys(&direct_task_125, module.clone(), &symbols),
            Vec::<String>::new()
        );
        let task_243 = reserve_then_parenthesized_identifier_binary_theorem_ast(
            source_id,
            vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
            ParenthesizedIdentifierBinaryTheoremSpec {
                label: "ParenthesizedReservedVariableMembershipPayloadBoundary",
                right: ParenthesizedIdentifierOperandShape::Direct("x"),
                ..exact_spec
            },
        );
        assert!(
            super::extract_source_parenthesized_reserved_variable_membership(
                &task_243,
                module.clone(),
                &symbols,
            )
            .is_some()
        );
        assert!(
            super::extract_source_parenthesized_heterogeneous_reserve_membership(
                &task_243,
                module.clone(),
                &symbols,
            )
            .is_none()
        );

        for corruption in [
            ParenthesizedHeterogeneousTypeRangeCorruption::Collapsed,
            ParenthesizedHeterogeneousTypeRangeCorruption::Reversed,
        ] {
            let corrupt =
                parenthesized_heterogeneous_reserve_membership_ast_with_corrupt_type_ranges(
                    source_id, corruption,
                );
            let extracted_reserve =
                super::extract_builtin_source_reserve_declarations_after_node_guard(
                    &corrupt,
                    module.clone(),
                    &symbols,
                )
                .expect("the corrupt AST should retain the exact two builtin reserve declarations");
            let corrupt_bindings = extracted_reserve.bridge.bindings();
            assert_eq!(corrupt_bindings.len(), 2);
            assert_eq!(corrupt_bindings[0].spelling, "x");
            assert_eq!(corrupt_bindings[0].type_head, TypeHeadInput::BuiltinObject);
            assert_eq!(corrupt_bindings[1].spelling, "y");
            assert_eq!(corrupt_bindings[1].type_head, TypeHeadInput::BuiltinSet);
            match corruption {
                ParenthesizedHeterogeneousTypeRangeCorruption::Collapsed => {
                    assert_eq!(
                        corrupt_bindings[0].type_range,
                        corrupt_bindings[1].type_range
                    );
                }
                ParenthesizedHeterogeneousTypeRangeCorruption::Reversed => {
                    assert!(
                        (
                            corrupt_bindings[0].type_range.start,
                            corrupt_bindings[0].type_range.end
                        ) > (
                            corrupt_bindings[1].type_range.start,
                            corrupt_bindings[1].type_range.end
                        )
                    );
                }
            }
            assert!(
                super::extract_source_parenthesized_heterogeneous_reserve_membership(
                    &corrupt,
                    module.clone(),
                    &symbols,
                )
                .is_none()
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&corrupt, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }

        let near_miss_specs = [
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: ParenthesizedIdentifierOperandShape::Direct("x"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: ParenthesizedIdentifierOperandShape::Direct("x"),
                right: parenthesized("y"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                right: parenthesized("y"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: ParenthesizedIdentifierOperandShape::Identifier {
                    spelling: "x",
                    depth: 2,
                    recovered: false,
                    open: "(",
                    close: ")",
                },
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: parenthesized("y"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                right: ParenthesizedIdentifierOperandShape::Direct("x"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: ParenthesizedIdentifierOperandShape::Identifier {
                    spelling: "x",
                    depth: 1,
                    recovered: true,
                    open: "(",
                    close: ")",
                },
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: ParenthesizedIdentifierOperandShape::Numeral("1"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: ParenthesizedIdentifierOperandShape::Empty,
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: ParenthesizedIdentifierOperandShape::DoubleIdentifier("x"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                label: "OtherPayloadBoundary",
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                operator: "=",
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                status: Some("registration"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                recovered_label: true,
                ..exact_spec
            },
        ];
        let near_misses = near_miss_specs
            .into_iter()
            .map(|spec| {
                reserve_then_parenthesized_identifier_binary_theorem_ast(source_id, reserve(), spec)
            })
            .chain([
                reserve_then_parenthesized_identifier_binary_theorem_ast(
                    source_id,
                    vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("object"))],
                    exact_spec,
                ),
                reserve_then_parenthesized_identifier_binary_theorem_ast(
                    source_id,
                    vec![
                        reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                        reserve_item(vec!["x"], ReserveTypeShape::Builtin("object")),
                    ],
                    exact_spec,
                ),
                reserve_then_parenthesized_identifier_binary_theorem_ast(
                    source_id,
                    vec![
                        reserve_item(vec!["x"], ReserveTypeShape::Builtin("set")),
                        reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                    ],
                    exact_spec,
                ),
                reserve_then_parenthesized_identifier_binary_theorem_ast(
                    source_id,
                    vec![
                        reserve_item(vec!["x"], ReserveTypeShape::Builtin("object")),
                        reserve_item(vec!["y"], ReserveTypeShape::Builtin("object")),
                    ],
                    exact_spec,
                ),
                reserve_then_parenthesized_identifier_binary_theorem_ast(
                    source_id,
                    vec![reserve_item(
                        vec!["x", "y"],
                        ReserveTypeShape::Builtin("object"),
                    )],
                    exact_spec,
                ),
                reserve_then_parenthesized_identifier_binary_theorem_ast(
                    source_id,
                    vec![
                        reserve_item(vec!["x"], ReserveTypeShape::Builtin("object")),
                        reserve_item(vec!["x"], ReserveTypeShape::Builtin("set")),
                    ],
                    exact_spec,
                ),
                reserve_then_parenthesized_identifier_binary_theorem_ast(
                    source_id,
                    vec![
                        reserve_item(vec!["x"], ReserveTypeShape::Builtin("object")),
                        reserve_item(vec!["y"], ReserveTypeShape::AttributedSet),
                    ],
                    exact_spec,
                ),
                reserve_then_parenthesized_identifier_binary_theorem_ast(
                    source_id,
                    vec![
                        reserve_item(vec!["x"], ReserveTypeShape::Builtin("object")),
                        reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                        reserve_item(vec!["z"], ReserveTypeShape::Builtin("set")),
                    ],
                    exact_spec,
                ),
            ]);
        for near_miss in near_misses {
            assert!(
                super::extract_source_parenthesized_heterogeneous_reserve_membership(
                    &near_miss,
                    module.clone(),
                    &symbols,
                )
                .is_none()
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
    }

    #[test]
    fn source_parenthesized_reserved_object_variable_equality_bridge_is_transparent() {
        let source_id = source_id(233);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("parenthesized_reserved_object_variable_equality"),
        );
        let symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        let reserve = || vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("object"))];
        let parenthesized = |spelling| ParenthesizedIdentifierOperandShape::Identifier {
            spelling,
            depth: 1,
            recovered: false,
            open: "(",
            close: ")",
        };
        let exact_spec = ParenthesizedIdentifierBinaryTheoremSpec {
            status: None,
            label: "ParenthesizedReservedObjectVariableEqualityPayloadBoundary",
            left: parenthesized("x"),
            operator: "=",
            right: ParenthesizedIdentifierOperandShape::Direct("x"),
            recovered_label: false,
        };
        let exact = reserve_then_parenthesized_identifier_binary_theorem_ast(
            source_id,
            reserve(),
            exact_spec,
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        let payload = super::extract_source_parenthesized_reserved_object_variable_equality(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact parenthesized builtin-object equality should extract");
        assert!(
            super::extract_source_parenthesized_reserved_variable_equality(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none()
        );
        assert!(
            super::extract_source_reserved_object_variable_equality(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none()
        );
        assert_eq!(payload.formula.reserve.bridge.bindings().len(), 1);
        assert_eq!(payload.formula.reserve.bridge.bindings()[0].spelling, "x");
        assert_eq!(
            payload.formula.reserve.bridge.bindings()[0].type_head,
            TypeHeadInput::BuiltinObject
        );
        assert_eq!(payload.formula.left_lookup_ordinal, 1);
        assert_eq!(payload.formula.right_lookup_ordinal, 2);
        assert_ne!(payload.wrapper_site, payload.formula.left_site);
        assert_ne!(payload.wrapper_site, payload.formula.right_site);
        assert!(payload.wrapper_range.start < payload.formula.left_range.start);
        assert!(payload.wrapper_range.end > payload.formula.left_range.end);
        assert!(payload.wrapper_range.end <= payload.formula.right_range.start);

        let output = super::source_parenthesized_reserved_object_variable_equality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact parenthesized builtin-object equality should reach the checker");
        super::assert_source_parenthesized_reserved_object_variable_equality_output(&output)
            .expect("parenthesized builtin-object equality invariants should hold");
        assert_eq!(output.wrapper_site, output.source_wrapper_site);
        assert_eq!(output.wrapper_range, output.source_wrapper_range);
        assert_eq!(output.formula.left_binding, BindingId::new(0));
        assert_eq!(output.formula.right_binding, BindingId::new(0));
        assert_eq!(
            output.formula.left_result_input.head,
            TypeHeadInput::BuiltinObject
        );
        assert_eq!(
            output.formula.right_result_input.head,
            TypeHeadInput::BuiltinObject
        );
        assert_eq!(
            output.formula.left_expected_input.as_ref().unwrap().head,
            TypeHeadInput::BuiltinObject
        );
        assert_eq!(
            output.formula.right_expected_input.as_ref().unwrap().head,
            TypeHeadInput::BuiltinObject
        );
        assert_eq!(output.formula.term_formula.terms().len(), 2);
        assert_eq!(output.formula.term_formula.type_entries().len(), 6);
        assert_eq!(output.formula.term_formula.normalized_types().len(), 1);
        assert!(output.formula.term_formula.candidate_sets().is_empty());
        assert!(output.formula.term_formula.facts().is_empty());
        assert!(output.formula.term_formula.diagnostics().is_empty());
        let (_, normalized) = output
            .formula
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("parenthesized builtin-object identity should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_eq!(normalized.source.spelling, "object");
        let (_, checked_formula) = output
            .formula
            .term_formula
            .formulas()
            .iter()
            .next()
            .expect("parenthesized builtin-object equality should be checked");
        assert_eq!(checked_formula.kind, FormulaKind::Equality);
        assert_eq!(checked_formula.status, FormulaStatus::Checked);
        assert_eq!(checked_formula.expected_types.len(), 2);
        assert!(checked_formula.facts.is_empty());
        assert!(checked_formula.deferred.is_empty());
        let role_names = output
            .formula
            .term_formula
            .type_entries()
            .iter()
            .filter_map(|(_, entry)| match &entry.owner {
                TypedSiteRef::Role { role, .. } => Some(role.as_str().to_owned()),
                _ => None,
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(
            role_names,
            BTreeSet::from([
                "parenthesized-reserved-object-variable-left-expected".to_owned(),
                "parenthesized-reserved-object-variable-left-result".to_owned(),
                "parenthesized-reserved-object-variable-right-expected".to_owned(),
                "parenthesized-reserved-object-variable-right-result".to_owned(),
            ])
        );
        assert!(
            output
                .formula
                .term_formula
                .terms()
                .iter()
                .all(|(_, term)| term.site.node() != output.wrapper_site.node())
        );

        let invalid_key = super::TYPE_ELABORATION_PARENTHESIZED_RESERVED_OBJECT_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY;
        let output_for_corruption = || {
            super::source_parenthesized_reserved_object_variable_equality_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce an object-wrapper corruption target")
        };
        let mut collapsed_wrapper = output_for_corruption();
        collapsed_wrapper.wrapper_site = collapsed_wrapper.formula.payload.left_site.clone();
        assert_eq!(
            super::source_parenthesized_reserved_object_variable_equality_output_detail_keys(
                &collapsed_wrapper,
            ),
            vec![invalid_key.to_owned()]
        );
        let mut collapsed_wrapper_range = output_for_corruption();
        collapsed_wrapper_range.wrapper_range = collapsed_wrapper_range.formula.payload.left_range;
        assert_eq!(
            super::source_parenthesized_reserved_object_variable_equality_output_detail_keys(
                &collapsed_wrapper_range,
            ),
            vec![invalid_key.to_owned()]
        );
        let mut collapsed_inner = output_for_corruption();
        collapsed_inner.formula.payload.left_site = collapsed_inner.wrapper_site.clone();
        assert_eq!(
            super::source_parenthesized_reserved_object_variable_equality_output_detail_keys(
                &collapsed_inner,
            ),
            vec![invalid_key.to_owned()]
        );
        let mut collapsed_right = output_for_corruption();
        collapsed_right.formula.payload.right_range = collapsed_right.formula.payload.left_range;
        assert_eq!(
            super::source_parenthesized_reserved_object_variable_equality_output_detail_keys(
                &collapsed_right,
            ),
            vec![invalid_key.to_owned()]
        );
        let mut wrong_binding = output_for_corruption();
        wrong_binding.formula.right_binding = BindingId::new(1);
        assert_eq!(
            super::source_parenthesized_reserved_object_variable_equality_output_detail_keys(
                &wrong_binding,
            ),
            vec![invalid_key.to_owned()]
        );
        let mut wrong_head = output_for_corruption();
        wrong_head.formula.left_result_input.head = TypeHeadInput::BuiltinSet;
        assert_eq!(
            super::source_parenthesized_reserved_object_variable_equality_output_detail_keys(
                &wrong_head,
            ),
            vec![invalid_key.to_owned()]
        );
        let mut collapsed_role = output_for_corruption();
        collapsed_role
            .formula
            .left_expected_input
            .as_mut()
            .unwrap()
            .site = collapsed_role.formula.left_result_input.site.clone();
        assert_eq!(
            super::source_parenthesized_reserved_object_variable_equality_output_detail_keys(
                &collapsed_role,
            ),
            vec![invalid_key.to_owned()]
        );
        let mut wrong_source = output_for_corruption();
        wrong_source.formula.left_result_input.source_range = payload.formula.left_range;
        assert_eq!(
            super::source_parenthesized_reserved_object_variable_equality_output_detail_keys(
                &wrong_source,
            ),
            vec![invalid_key.to_owned()]
        );
        let mut wrong_canonical_source = output_for_corruption();
        let (bridge_source_id, bridge_module, bridge_range, mut bridge_bindings) = {
            let bridge = &wrong_canonical_source.formula.payload.reserve.bridge;
            (
                bridge.source_id(),
                bridge.module_id().clone(),
                bridge.source_range(),
                bridge.bindings().to_vec(),
            )
        };
        assert_ne!(
            bridge_bindings[0].type_range,
            wrong_canonical_source.formula.payload.left_range
        );
        bridge_bindings[0].type_range = wrong_canonical_source.formula.payload.left_range;
        wrong_canonical_source.formula.payload.reserve.bridge =
            super::SourceReserveDeclarationBridge::new(
                bridge_source_id,
                bridge_module,
                bridge_range,
                bridge_bindings,
            )
            .expect("canonical-source corruption bridge should remain structurally valid");
        assert_eq!(
            super::source_parenthesized_reserved_object_variable_equality_output_detail_keys(
                &wrong_canonical_source,
            ),
            vec![invalid_key.to_owned()]
        );
        let mut missing_expected = output_for_corruption();
        missing_expected.formula.right_expected_input = None;
        assert_eq!(
            super::source_parenthesized_reserved_object_variable_equality_output_detail_keys(
                &missing_expected,
            ),
            vec![invalid_key.to_owned()]
        );
        let set_output = super::source_parenthesized_reserved_variable_equality_output(
            &reserve_then_parenthesized_identifier_binary_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
                ParenthesizedIdentifierBinaryTheoremSpec {
                    label: "ParenthesizedReservedVariableEqualityPayloadBoundary",
                    ..exact_spec
                },
            ),
            module.clone(),
            &symbols,
        )
        .expect("Task 223 set source should retain its own closed owner");
        assert_eq!(
            super::source_parenthesized_reserved_object_variable_equality_output_detail_keys(
                &set_output,
            ),
            vec![invalid_key.to_owned()]
        );
        let mut wrong_ordinal =
            super::extract_source_parenthesized_reserved_object_variable_equality(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce an ordinal corruption target");
        wrong_ordinal.formula.left_lookup_ordinal = wrong_ordinal.formula.right_lookup_ordinal;
        assert_eq!(
            super::source_parenthesized_reserved_variable_binary_formula_payload_detail_keys(
                wrong_ordinal,
                &symbols,
                &super::SOURCE_PARENTHESIZED_RESERVED_OBJECT_VARIABLE_EQUALITY_CONFIG,
                super::SourceParenthesizedOperandSide::Left,
            ),
            vec![invalid_key.to_owned()]
        );
        let mut immutable_payload =
            super::extract_source_parenthesized_reserved_object_variable_equality(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce an immutable-output target");
        immutable_payload.wrapper_range = immutable_payload.formula.left_range;
        let immutable_output =
            super::build_source_parenthesized_reserved_variable_binary_formula_output(
                immutable_payload,
                &symbols,
            )
            .expect("wrapper-only corruption must not mutate checker payloads");
        assert_eq!(immutable_output.formula.term_formula.terms().len(), 2);
        assert_eq!(
            immutable_output.formula.term_formula.type_entries().len(),
            6
        );
        assert_eq!(
            super::source_parenthesized_reserved_object_variable_equality_output_detail_keys(
                &immutable_output,
            ),
            vec![invalid_key.to_owned()]
        );
        let mismatched_symbols = SymbolEnv::new(
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("other_module")),
            SymbolEnvIndexes::default(),
        );
        let mismatched_payload =
            super::extract_source_parenthesized_reserved_object_variable_equality(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce a module-corruption target");
        assert!(
            super::build_source_parenthesized_reserved_variable_binary_formula_output(
                mismatched_payload,
                &mismatched_symbols,
            )
            .is_err()
        );

        let direct_task_188 = reserve_then_identifier_equality_theorem_ast(
            source_id,
            reserve(),
            "ReservedObjectVariableEqualityPayloadBoundary",
            "x",
            "x",
        );
        assert!(
            super::extract_source_parenthesized_reserved_object_variable_equality(
                &direct_task_188,
                module.clone(),
                &symbols,
            )
            .is_none()
        );
        assert!(
            super::extract_source_reserved_object_variable_equality(
                &direct_task_188,
                module.clone(),
                &symbols,
            )
            .is_some()
        );
        assert_eq!(
            source_type_elaboration_detail_keys(&direct_task_188, module.clone(), &symbols),
            Vec::<String>::new()
        );

        let near_misses = [
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: ParenthesizedIdentifierOperandShape::Direct("x"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: ParenthesizedIdentifierOperandShape::Direct("x"),
                right: parenthesized("x"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                right: parenthesized("x"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: ParenthesizedIdentifierOperandShape::Identifier {
                    spelling: "x",
                    depth: 2,
                    recovered: false,
                    open: "(",
                    close: ")",
                },
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: parenthesized("y"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                right: ParenthesizedIdentifierOperandShape::Direct("y"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: ParenthesizedIdentifierOperandShape::Identifier {
                    spelling: "x",
                    depth: 1,
                    recovered: true,
                    open: "(",
                    close: ")",
                },
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: ParenthesizedIdentifierOperandShape::Identifier {
                    spelling: "x",
                    depth: 1,
                    recovered: false,
                    open: "[",
                    close: "]",
                },
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: ParenthesizedIdentifierOperandShape::Numeral("1"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: ParenthesizedIdentifierOperandShape::Empty,
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: ParenthesizedIdentifierOperandShape::DoubleIdentifier("x"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                label: "OtherPayloadBoundary",
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                operator: "<>",
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                status: Some("registration"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                recovered_label: true,
                ..exact_spec
            },
        ]
        .into_iter()
        .map(|spec| {
            reserve_then_parenthesized_identifier_binary_theorem_ast(source_id, reserve(), spec)
        })
        .chain([
            reserve_then_parenthesized_identifier_binary_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
                exact_spec,
            ),
            reserve_then_parenthesized_identifier_binary_theorem_ast(
                source_id,
                vec![reserve_item(
                    vec!["x", "y"],
                    ReserveTypeShape::Builtin("object"),
                )],
                exact_spec,
            ),
            reserve_then_parenthesized_identifier_binary_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::AttributedObject)],
                exact_spec,
            ),
            reserve_then_parenthesized_identifier_binary_theorem_ast(
                source_id,
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::Builtin("object")),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("object")),
                ],
                exact_spec,
            ),
        ]);
        for near_miss in near_misses {
            assert!(
                super::extract_source_parenthesized_reserved_object_variable_equality(
                    &near_miss,
                    module.clone(),
                    &symbols,
                )
                .is_none()
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
    }

    #[test]
    fn source_parenthesized_reserved_object_variable_inequality_bridge_is_transparent() {
        let source_id = source_id(242);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("parenthesized_reserved_object_variable_inequality"),
        );
        let symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        let reserve = || vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("object"))];
        let parenthesized = |spelling| ParenthesizedIdentifierOperandShape::Identifier {
            spelling,
            depth: 1,
            recovered: false,
            open: "(",
            close: ")",
        };
        let exact_spec = ParenthesizedIdentifierBinaryTheoremSpec {
            status: None,
            label: "ParenthesizedReservedObjectVariableInequalityPayloadBoundary",
            left: parenthesized("x"),
            operator: "<>",
            right: ParenthesizedIdentifierOperandShape::Direct("x"),
            recovered_label: false,
        };
        let exact = reserve_then_parenthesized_identifier_binary_theorem_ast(
            source_id,
            reserve(),
            exact_spec,
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        let payload = super::extract_source_parenthesized_reserved_object_variable_inequality(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact parenthesized builtin-object inequality should extract");
        assert!(
            super::extract_source_parenthesized_reserved_object_variable_equality(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none()
        );
        assert!(
            super::extract_source_parenthesized_reserved_variable_inequality(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none()
        );
        assert_eq!(payload.formula.reserve.bridge.bindings().len(), 1);
        assert_eq!(payload.formula.reserve.bridge.bindings()[0].spelling, "x");
        assert_eq!(
            payload.formula.reserve.bridge.bindings()[0].type_head,
            TypeHeadInput::BuiltinObject
        );
        assert_eq!(payload.formula.left_lookup_ordinal, 1);
        assert_eq!(payload.formula.right_lookup_ordinal, 2);
        assert_ne!(payload.wrapper_site, payload.formula.left_site);
        assert_ne!(payload.wrapper_site, payload.formula.right_site);
        assert!(payload.wrapper_range.start < payload.formula.left_range.start);
        assert!(payload.wrapper_range.end > payload.formula.left_range.end);
        assert!(payload.wrapper_range.end <= payload.formula.right_range.start);

        let output = super::source_parenthesized_reserved_object_variable_inequality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact parenthesized builtin-object inequality should reach the checker");
        super::assert_source_parenthesized_reserved_object_variable_inequality_output(&output)
            .expect("parenthesized builtin-object inequality invariants should hold");
        assert_eq!(output.wrapper_site, output.source_wrapper_site);
        assert_eq!(output.wrapper_range, output.source_wrapper_range);
        assert_eq!(output.formula.left_binding, BindingId::new(0));
        assert_eq!(output.formula.right_binding, BindingId::new(0));
        assert_eq!(
            output.formula.left_result_input.head,
            TypeHeadInput::BuiltinObject
        );
        assert_eq!(
            output.formula.right_result_input.head,
            TypeHeadInput::BuiltinObject
        );
        assert_eq!(
            output.formula.left_expected_input.as_ref().unwrap().head,
            TypeHeadInput::BuiltinObject
        );
        assert_eq!(
            output.formula.right_expected_input.as_ref().unwrap().head,
            TypeHeadInput::BuiltinObject
        );
        assert_eq!(output.formula.term_formula.terms().len(), 2);
        assert_eq!(output.formula.term_formula.type_entries().len(), 6);
        assert_eq!(output.formula.term_formula.normalized_types().len(), 1);
        assert!(output.formula.term_formula.candidate_sets().is_empty());
        assert!(output.formula.term_formula.facts().is_empty());
        assert!(output.formula.term_formula.diagnostics().is_empty());
        let (_, normalized) = output
            .formula
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("parenthesized builtin-object inequality identity should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_ne!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_eq!(normalized.source.spelling, "object");
        assert_eq!(
            normalized.source.range,
            output.formula.payload.reserve.bridge.bindings()[0].type_range
        );
        let formulas = output.formula.term_formula.formulas();
        assert_eq!(formulas.len(), 1);
        let (_, checked_formula) = formulas
            .iter()
            .next()
            .expect("parenthesized builtin-object inequality should be checked");
        assert_eq!(checked_formula.kind, FormulaKind::Inequality);
        assert_eq!(checked_formula.status, FormulaStatus::Checked);
        assert_eq!(
            checked_formula.terms,
            [
                output.formula.payload.left_site.clone(),
                output.formula.payload.right_site.clone(),
            ]
        );
        assert_eq!(checked_formula.expected_types.len(), 2);
        assert!(checked_formula.facts.is_empty());
        assert!(checked_formula.deferred.is_empty());
        let role_names = output
            .formula
            .term_formula
            .type_entries()
            .iter()
            .filter_map(|(_, entry)| match &entry.owner {
                TypedSiteRef::Role { role, .. } => Some(role.as_str().to_owned()),
                _ => None,
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(
            role_names,
            BTreeSet::from([
                "parenthesized-reserved-object-variable-inequality-left-expected".to_owned(),
                "parenthesized-reserved-object-variable-inequality-left-result".to_owned(),
                "parenthesized-reserved-object-variable-inequality-right-expected".to_owned(),
                "parenthesized-reserved-object-variable-inequality-right-result".to_owned(),
            ])
        );
        assert!(
            output
                .formula
                .term_formula
                .terms()
                .iter()
                .all(|(_, term)| { term.site.node() != output.wrapper_site.node() })
        );
        assert!(
            output
                .formula
                .term_formula
                .type_entries()
                .iter()
                .all(|(_, entry)| { entry.owner.node() != output.wrapper_site.node() })
        );
        assert!(
            output
                .formula
                .term_formula
                .formulas()
                .iter()
                .all(|(_, formula)| {
                    formula.site.node() != output.wrapper_site.node()
                        && formula
                            .terms
                            .iter()
                            .all(|term| term.node() != output.wrapper_site.node())
                })
        );

        let invalid_key = super::TYPE_ELABORATION_PARENTHESIZED_RESERVED_OBJECT_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY;
        let output_for_corruption = || {
            super::source_parenthesized_reserved_object_variable_inequality_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce an object-inequality corruption target")
        };
        let assert_invalid = |output| {
            assert_eq!(
                super::source_parenthesized_reserved_object_variable_inequality_output_detail_keys(
                    output,
                ),
                vec![invalid_key.to_owned()]
            );
        };
        let mut collapsed_wrapper = output_for_corruption();
        collapsed_wrapper.wrapper_site = collapsed_wrapper.formula.payload.left_site.clone();
        assert_invalid(&collapsed_wrapper);
        let mut collapsed_wrapper_range = output_for_corruption();
        collapsed_wrapper_range.wrapper_range = collapsed_wrapper_range.formula.payload.left_range;
        assert_invalid(&collapsed_wrapper_range);
        let mut stale_source_wrapper_site = output_for_corruption();
        stale_source_wrapper_site.source_wrapper_site =
            stale_source_wrapper_site.formula.payload.right_site.clone();
        assert_invalid(&stale_source_wrapper_site);
        let mut stale_source_wrapper_range = output_for_corruption();
        stale_source_wrapper_range.source_wrapper_range =
            stale_source_wrapper_range.formula.payload.left_range;
        assert_invalid(&stale_source_wrapper_range);
        let mut collapsed_inner_site = output_for_corruption();
        collapsed_inner_site.formula.payload.left_site = collapsed_inner_site.wrapper_site.clone();
        assert_invalid(&collapsed_inner_site);
        let mut collapsed_inner_range = output_for_corruption();
        collapsed_inner_range.formula.payload.left_range = collapsed_inner_range.wrapper_range;
        assert_invalid(&collapsed_inner_range);
        let mut collapsed_right_site = output_for_corruption();
        collapsed_right_site.formula.payload.right_site = collapsed_right_site.wrapper_site.clone();
        assert_invalid(&collapsed_right_site);
        let mut collapsed_right_range = output_for_corruption();
        collapsed_right_range.formula.payload.right_range =
            collapsed_right_range.formula.payload.left_range;
        assert_invalid(&collapsed_right_range);
        let mut wrong_binding = output_for_corruption();
        wrong_binding.formula.right_binding = BindingId::new(1);
        assert_invalid(&wrong_binding);
        let mut wrong_head = output_for_corruption();
        wrong_head.formula.left_result_input.head = TypeHeadInput::BuiltinSet;
        assert_invalid(&wrong_head);
        let mut collapsed_role = output_for_corruption();
        collapsed_role
            .formula
            .left_expected_input
            .as_mut()
            .unwrap()
            .site = collapsed_role.formula.left_result_input.site.clone();
        assert_invalid(&collapsed_role);
        let mut wrong_source = output_for_corruption();
        wrong_source.formula.left_result_input.source_range = payload.formula.left_range;
        assert_invalid(&wrong_source);
        let mut wrong_canonical_source = output_for_corruption();
        let (bridge_source_id, bridge_module, bridge_range, mut bridge_bindings) = {
            let bridge = &wrong_canonical_source.formula.payload.reserve.bridge;
            (
                bridge.source_id(),
                bridge.module_id().clone(),
                bridge.source_range(),
                bridge.bindings().to_vec(),
            )
        };
        bridge_bindings[0].type_range = wrong_canonical_source.formula.payload.left_range;
        wrong_canonical_source.formula.payload.reserve.bridge =
            super::SourceReserveDeclarationBridge::new(
                bridge_source_id,
                bridge_module,
                bridge_range,
                bridge_bindings,
            )
            .expect("canonical-source corruption bridge should remain structurally valid");
        assert_invalid(&wrong_canonical_source);
        let mut missing_expected = output_for_corruption();
        missing_expected.formula.left_expected_input = None;
        assert_invalid(&missing_expected);

        let task_233_output = super::source_parenthesized_reserved_object_variable_equality_output(
            &reserve_then_parenthesized_identifier_binary_theorem_ast(
                source_id,
                reserve(),
                ParenthesizedIdentifierBinaryTheoremSpec {
                    label: "ParenthesizedReservedObjectVariableEqualityPayloadBoundary",
                    operator: "=",
                    ..exact_spec
                },
            ),
            module.clone(),
            &symbols,
        )
        .expect("Task 233 object equality source should retain its closed owner");
        assert_invalid(&task_233_output);
        let task_241_output = super::source_parenthesized_reserved_variable_inequality_output(
            &reserve_then_parenthesized_identifier_binary_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
                ParenthesizedIdentifierBinaryTheoremSpec {
                    label: "ParenthesizedReservedVariableInequalityPayloadBoundary",
                    ..exact_spec
                },
            ),
            module.clone(),
            &symbols,
        )
        .expect("Task 241 set inequality source should retain its closed owner");
        assert_invalid(&task_241_output);

        let mut wrong_ordinal =
            super::extract_source_parenthesized_reserved_object_variable_inequality(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce an ordinal corruption target");
        wrong_ordinal.formula.left_lookup_ordinal = wrong_ordinal.formula.right_lookup_ordinal;
        assert_eq!(
            super::source_parenthesized_reserved_variable_binary_formula_payload_detail_keys(
                wrong_ordinal,
                &symbols,
                &super::SOURCE_PARENTHESIZED_RESERVED_OBJECT_VARIABLE_INEQUALITY_CONFIG,
                super::SourceParenthesizedOperandSide::Left,
            ),
            vec![invalid_key.to_owned()]
        );
        let mut immutable_payload =
            super::extract_source_parenthesized_reserved_object_variable_inequality(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce an immutable-output target");
        immutable_payload.wrapper_range = immutable_payload.formula.left_range;
        let immutable_output =
            super::build_source_parenthesized_reserved_variable_binary_formula_output(
                immutable_payload,
                &symbols,
            )
            .expect("wrapper-only corruption must not mutate checker payloads");
        assert_eq!(immutable_output.formula.term_formula.terms().len(), 2);
        assert_eq!(
            immutable_output.formula.term_formula.type_entries().len(),
            6
        );
        assert_invalid(&immutable_output);
        let mismatched_symbols = SymbolEnv::new(
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("other_module")),
            SymbolEnvIndexes::default(),
        );
        let mismatched_payload =
            super::extract_source_parenthesized_reserved_object_variable_inequality(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce a module-corruption target");
        assert!(
            super::build_source_parenthesized_reserved_variable_binary_formula_output(
                mismatched_payload,
                &mismatched_symbols,
            )
            .is_err()
        );

        let direct_task_190 = reserve_then_identifier_binary_theorem_ast(
            source_id,
            reserve(),
            "ReservedObjectVariableInequalityPayloadBoundary",
            "x",
            "<>",
            "x",
        );
        assert!(
            super::extract_source_parenthesized_reserved_object_variable_inequality(
                &direct_task_190,
                module.clone(),
                &symbols,
            )
            .is_none()
        );
        assert!(
            super::extract_source_reserved_object_variable_inequality(
                &direct_task_190,
                module.clone(),
                &symbols,
            )
            .is_some()
        );

        let near_miss_specs = [
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: ParenthesizedIdentifierOperandShape::Direct("x"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: ParenthesizedIdentifierOperandShape::Direct("x"),
                right: parenthesized("x"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                right: parenthesized("x"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: ParenthesizedIdentifierOperandShape::Identifier {
                    spelling: "x",
                    depth: 2,
                    recovered: false,
                    open: "(",
                    close: ")",
                },
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: parenthesized("y"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                right: ParenthesizedIdentifierOperandShape::Direct("y"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: ParenthesizedIdentifierOperandShape::Identifier {
                    spelling: "x",
                    depth: 1,
                    recovered: true,
                    open: "(",
                    close: ")",
                },
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: ParenthesizedIdentifierOperandShape::Identifier {
                    spelling: "x",
                    depth: 1,
                    recovered: false,
                    open: "[",
                    close: "]",
                },
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: ParenthesizedIdentifierOperandShape::Numeral("1"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: ParenthesizedIdentifierOperandShape::Empty,
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                left: ParenthesizedIdentifierOperandShape::DoubleIdentifier("x"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                label: "OtherPayloadBoundary",
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                operator: "=",
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                operator: "in",
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                status: Some("registration"),
                ..exact_spec
            },
            ParenthesizedIdentifierBinaryTheoremSpec {
                recovered_label: true,
                ..exact_spec
            },
        ];
        let near_misses = near_miss_specs
            .into_iter()
            .map(|spec| {
                reserve_then_parenthesized_identifier_binary_theorem_ast(source_id, reserve(), spec)
            })
            .chain([
                reserve_then_parenthesized_identifier_binary_theorem_ast(
                    source_id,
                    vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
                    exact_spec,
                ),
                reserve_then_parenthesized_identifier_binary_theorem_ast(
                    source_id,
                    vec![reserve_item(
                        vec!["x", "y"],
                        ReserveTypeShape::Builtin("object"),
                    )],
                    exact_spec,
                ),
                reserve_then_parenthesized_identifier_binary_theorem_ast(
                    source_id,
                    vec![reserve_item(vec!["x"], ReserveTypeShape::AttributedObject)],
                    exact_spec,
                ),
                reserve_then_parenthesized_identifier_binary_theorem_ast(
                    source_id,
                    vec![
                        reserve_item(vec!["x"], ReserveTypeShape::Builtin("object")),
                        reserve_item(vec!["y"], ReserveTypeShape::Builtin("object")),
                    ],
                    exact_spec,
                ),
            ]);
        for near_miss in near_misses {
            assert!(
                super::extract_source_parenthesized_reserved_object_variable_inequality(
                    &near_miss,
                    module.clone(),
                    &symbols,
                )
                .is_none()
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
    }

    #[test]
    fn source_reserved_object_variable_equality_bridge_preserves_builtin_object_identity() {
        let source_id = source_id(188);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("reserved_object_variable_equality"),
        );
        let symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        let exact = reserve_then_identifier_equality_theorem_ast(
            source_id,
            vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("object"))],
            "ReservedObjectVariableEqualityPayloadBoundary",
            "x",
            "x",
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        let payload = super::extract_source_reserved_object_variable_equality(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact builtin-object reserved-variable equality should extract");
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(payload.reserve.bridge.bindings()[0].spelling, "x");
        assert_eq!(payload.reserve.bridge.bindings()[0].type_spelling, "object");
        assert_eq!(
            payload.reserve.bridge.bindings()[0].type_head,
            TypeHeadInput::BuiltinObject
        );
        assert_eq!(payload.left_lookup_ordinal, 1);
        assert_eq!(payload.right_lookup_ordinal, 2);

        let output = super::source_reserved_object_variable_equality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact builtin-object equality should reach the checker");
        assert_source_reserved_variable_formula_output(&output)
            .expect("builtin-object equality invariants should hold");
        assert_eq!(output.left_binding, BindingId::new(0));
        assert_eq!(output.right_binding, BindingId::new(0));
        assert_eq!(output.left_result_input.head, TypeHeadInput::BuiltinObject);
        assert_eq!(output.right_result_input.head, TypeHeadInput::BuiltinObject);
        assert_eq!(
            output.left_expected_input.as_ref().unwrap().head,
            TypeHeadInput::BuiltinObject
        );
        assert_eq!(
            output.right_expected_input.as_ref().unwrap().head,
            TypeHeadInput::BuiltinObject
        );
        let role_sites = [
            &output.left_result_input.site,
            &output.right_result_input.site,
            &output.left_expected_input.as_ref().unwrap().site,
            &output.right_expected_input.as_ref().unwrap().site,
        ]
        .into_iter()
        .collect::<BTreeSet<_>>();
        assert_eq!(role_sites.len(), 4);
        let written_type_range = payload.reserve.bridge.bindings()[0].type_range;
        for input in [
            &output.left_result_input,
            &output.right_result_input,
            output.left_expected_input.as_ref().unwrap(),
            output.right_expected_input.as_ref().unwrap(),
        ] {
            assert_eq!(input.source_range, written_type_range);
            assert_eq!(input.spelling, "object");
        }
        assert_eq!(output.term_formula.type_entries().len(), 6);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("builtin-object normalized type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_eq!(normalized.source.range, written_type_range);
        assert_eq!(normalized.source.spelling, "object");
        let formula = output
            .term_formula
            .formulas()
            .iter()
            .map(|(_, formula)| formula)
            .next()
            .expect("builtin-object equality formula should exist");
        assert_eq!(formula.kind, FormulaKind::Equality);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert_eq!(formula.expected_types.len(), 2);
        assert_eq!(formula.expected_types[0].term, payload.left_site);
        assert_eq!(formula.expected_types[1].term, payload.right_site);
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
        assert!(output.term_formula.facts().is_empty());
        assert!(output.term_formula.diagnostics().is_empty());

        let mut wrong_head = super::source_reserved_object_variable_equality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce a corruption target");
        wrong_head.left_result_input.head = TypeHeadInput::BuiltinSet;
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&wrong_head),
            vec![
                super::TYPE_ELABORATION_RESERVED_OBJECT_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY
                    .to_owned()
            ]
        );
        let mut collapsed_role = super::source_reserved_object_variable_equality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce another corruption target");
        collapsed_role.left_expected_input.as_mut().unwrap().site =
            collapsed_role.left_result_input.site.clone();
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&collapsed_role),
            vec![
                super::TYPE_ELABORATION_RESERVED_OBJECT_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY
                    .to_owned()
            ]
        );
        let mut wrong_binding = super::source_reserved_object_variable_equality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce a binding corruption target");
        wrong_binding.right_binding = BindingId::new(1);
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&wrong_binding),
            vec![
                super::TYPE_ELABORATION_RESERVED_OBJECT_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY
                    .to_owned()
            ]
        );
        let mut wrong_ordinal = super::extract_source_reserved_object_variable_equality(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce an ordinal corruption target");
        wrong_ordinal.left_lookup_ordinal = wrong_ordinal.right_lookup_ordinal;
        assert_eq!(
            source_reserved_variable_formula_result_detail_keys(
                build_source_reserved_variable_formula_output(wrong_ordinal, &symbols),
                super::TYPE_ELABORATION_RESERVED_OBJECT_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
            ),
            vec![
                super::TYPE_ELABORATION_RESERVED_OBJECT_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY
                    .to_owned()
            ]
        );
        let mut wrong_source = super::source_reserved_object_variable_equality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce a source corruption target");
        wrong_source.left_result_input.source_range = payload.left_range;
        assert_ne!(
            wrong_source.left_result_input.source_range,
            written_type_range
        );
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&wrong_source),
            vec![
                super::TYPE_ELABORATION_RESERVED_OBJECT_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY
                    .to_owned()
            ]
        );
        let mut missing_expected = super::source_reserved_object_variable_equality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce an expected-input corruption target");
        missing_expected.right_expected_input = None;
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&missing_expected),
            vec![
                super::TYPE_ELABORATION_RESERVED_OBJECT_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY
                    .to_owned()
            ]
        );

        let near_misses = [
            reserve_then_identifier_equality_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("object"))],
                "OtherPayloadBoundary",
                "x",
                "x",
            ),
            reserve_then_identifier_equality_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("object"))],
                "ReservedObjectVariableEqualityPayloadBoundary",
                "x",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("object"))],
                "ReservedObjectVariableEqualityPayloadBoundary",
                "x",
                "<>",
                "x",
            ),
            reserve_then_identifier_equality_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
                "ReservedObjectVariableEqualityPayloadBoundary",
                "x",
                "x",
            ),
            reserve_then_identifier_equality_theorem_ast(
                source_id,
                vec![reserve_item(
                    vec!["x", "y"],
                    ReserveTypeShape::Builtin("object"),
                )],
                "ReservedObjectVariableEqualityPayloadBoundary",
                "x",
                "x",
            ),
            reserve_then_identifier_equality_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::AttributedObject)],
                "ReservedObjectVariableEqualityPayloadBoundary",
                "x",
                "x",
            ),
            reserve_then_identifier_equality_theorem_ast(
                source_id,
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbolWithArgs("Mode"),
                )],
                "ReservedObjectVariableEqualityPayloadBoundary",
                "x",
                "x",
            ),
            reserve_then_identifier_equality_theorem_ast(
                source_id,
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::Builtin("object")),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("object")),
                ],
                "ReservedObjectVariableEqualityPayloadBoundary",
                "x",
                "x",
            ),
            reserve_then_identifier_binary_theorem_ast_with_options(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("object"))],
                IdentifierBinaryTheoremSpec {
                    status: Some("registration"),
                    label: "ReservedObjectVariableEqualityPayloadBoundary",
                    left: "x",
                    operator: "=",
                    right: "x",
                    recovered_label: false,
                },
            ),
            reserve_then_identifier_binary_theorem_ast_with_options(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("object"))],
                IdentifierBinaryTheoremSpec {
                    status: None,
                    label: "ReservedObjectVariableEqualityPayloadBoundary",
                    left: "x",
                    operator: "=",
                    right: "x",
                    recovered_label: true,
                },
            ),
            reserve_then_two_identifier_equality_theorems_ast(source_id),
            theorem_then_reserve_identifier_equality_ast(source_id),
            reserve_then_builtin_equality_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("object"))],
                "ReservedObjectVariableEqualityPayloadBoundary",
                "1",
                "1",
            ),
        ];
        for near_miss in near_misses {
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }

        let provenance_near_misses = [
            (
                reserve_then_identifier_equality_theorem_ast(
                    source_id,
                    vec![reserve_item(
                        vec!["x"],
                        ReserveTypeShape::QualifiedSymbol("Mode"),
                    )],
                    "ReservedObjectVariableEqualityPayloadBoundary",
                    "x",
                    "x",
                ),
                source_mode_symbol_env(module.clone()),
            ),
            (
                reserve_then_identifier_equality_theorem_ast(
                    source_id,
                    vec![reserve_item(
                        vec!["x"],
                        ReserveTypeShape::QualifiedSymbol("Struct"),
                    )],
                    "ReservedObjectVariableEqualityPayloadBoundary",
                    "x",
                    "x",
                ),
                source_structure_symbol_env(module.clone()),
            ),
            (
                reserve_then_identifier_equality_theorem_ast(
                    source_id,
                    vec![reserve_item(
                        vec!["x"],
                        ReserveTypeShape::QualifiedSymbol("Mode"),
                    )],
                    "ReservedObjectVariableEqualityPayloadBoundary",
                    "x",
                    "x",
                ),
                imported_mode_symbol_env(module.clone()),
            ),
            (
                reserve_then_identifier_equality_theorem_ast(
                    source_id,
                    vec![reserve_item(
                        vec!["x"],
                        ReserveTypeShape::QualifiedSymbol("Struct"),
                    )],
                    "ReservedObjectVariableEqualityPayloadBoundary",
                    "x",
                    "x",
                ),
                imported_structure_symbol_env(module.clone()),
            ),
            (
                reserve_then_identifier_equality_theorem_ast(
                    source_id,
                    vec![reserve_item(
                        vec!["x"],
                        ReserveTypeShape::QualifiedSymbol("Mode"),
                    )],
                    "ReservedObjectVariableEqualityPayloadBoundary",
                    "x",
                    "x",
                ),
                ambiguous_mode_symbol_env(module.clone()),
            ),
            (
                reserve_then_identifier_equality_theorem_ast(
                    source_id,
                    vec![reserve_item(
                        vec!["x"],
                        ReserveTypeShape::QualifiedSymbol("Struct"),
                    )],
                    "ReservedObjectVariableEqualityPayloadBoundary",
                    "x",
                    "x",
                ),
                ambiguous_structure_symbol_env(module.clone()),
            ),
        ];
        for (near_miss, near_miss_symbols) in provenance_near_misses {
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &near_miss_symbols,),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
    }

    #[test]
    fn source_reserved_object_variable_inequality_bridge_preserves_builtin_object_identity() {
        let source_id = source_id(190);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("reserved_object_variable_inequality"),
        );
        let symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        let exact = reserve_then_identifier_binary_theorem_ast(
            source_id,
            vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("object"))],
            "ReservedObjectVariableInequalityPayloadBoundary",
            "x",
            "<>",
            "x",
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        let payload = super::extract_source_reserved_object_variable_inequality(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact builtin-object reserved-variable inequality should extract");
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(payload.reserve.bridge.bindings()[0].spelling, "x");
        assert_eq!(payload.reserve.bridge.bindings()[0].type_spelling, "object");
        assert_eq!(
            payload.reserve.bridge.bindings()[0].type_head,
            TypeHeadInput::BuiltinObject
        );
        assert_eq!(payload.left_lookup_ordinal, 1);
        assert_eq!(payload.right_lookup_ordinal, 2);

        let output = super::source_reserved_object_variable_inequality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact builtin-object inequality should reach the checker");
        assert_source_reserved_variable_formula_output(&output)
            .expect("builtin-object inequality invariants should hold");
        assert_eq!(output.left_binding, BindingId::new(0));
        assert_eq!(output.right_binding, BindingId::new(0));
        let raw_inputs = [
            &output.left_result_input,
            &output.right_result_input,
            output.left_expected_input.as_ref().unwrap(),
            output.right_expected_input.as_ref().unwrap(),
        ];
        let role_sites = raw_inputs
            .iter()
            .map(|input| &input.site)
            .collect::<BTreeSet<_>>();
        assert_eq!(role_sites.len(), 4);
        let written_type_range = payload.reserve.bridge.bindings()[0].type_range;
        for input in raw_inputs {
            assert_eq!(input.head, TypeHeadInput::BuiltinObject);
            assert_eq!(input.source_range, written_type_range);
            assert_eq!(input.spelling, "object");
            assert!(input.args.is_empty());
            assert!(input.attributes.is_empty());
        }
        assert_eq!(output.term_formula.terms().len(), 2);
        assert!(
            output
                .term_formula
                .terms()
                .iter()
                .all(|(_, term)| term.status == TermStatus::Inferred && term.deferred.is_empty())
        );
        assert_eq!(output.term_formula.type_entries().len(), 6);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert!(output.term_formula.candidate_sets().is_empty());
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("builtin-object normalized type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_eq!(normalized.source.range, written_type_range);
        assert_eq!(normalized.source.spelling, "object");
        let formula = output
            .term_formula
            .formulas()
            .iter()
            .map(|(_, formula)| formula)
            .next()
            .expect("builtin-object inequality formula should exist");
        assert_eq!(formula.kind, FormulaKind::Inequality);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert_eq!(formula.expected_types.len(), 2);
        assert_eq!(formula.expected_types[0].term, payload.left_site);
        assert_eq!(formula.expected_types[1].term, payload.right_site);
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
        assert!(output.term_formula.facts().is_empty());
        assert!(output.term_formula.diagnostics().is_empty());

        let invalid_key =
            super::TYPE_ELABORATION_RESERVED_OBJECT_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY;
        let mut wrong_head = super::source_reserved_object_variable_inequality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce a head corruption target");
        wrong_head.left_result_input.head = TypeHeadInput::BuiltinSet;
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&wrong_head),
            vec![invalid_key.to_owned()]
        );
        let mut collapsed_role = super::source_reserved_object_variable_inequality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce a role corruption target");
        collapsed_role.left_expected_input.as_mut().unwrap().site =
            collapsed_role.left_result_input.site.clone();
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&collapsed_role),
            vec![invalid_key.to_owned()]
        );
        let mut wrong_binding = super::source_reserved_object_variable_inequality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce a binding corruption target");
        wrong_binding.right_binding = BindingId::new(1);
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&wrong_binding),
            vec![invalid_key.to_owned()]
        );
        let mut wrong_ordinal = super::extract_source_reserved_object_variable_inequality(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce an ordinal corruption target");
        wrong_ordinal.left_lookup_ordinal = wrong_ordinal.right_lookup_ordinal;
        assert_eq!(
            source_reserved_variable_formula_result_detail_keys(
                build_source_reserved_variable_formula_output(wrong_ordinal, &symbols),
                invalid_key,
            ),
            vec![invalid_key.to_owned()]
        );
        let mut wrong_source = super::source_reserved_object_variable_inequality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce a source corruption target");
        wrong_source
            .right_expected_input
            .as_mut()
            .unwrap()
            .source_range = payload.right_range;
        assert_ne!(
            wrong_source
                .right_expected_input
                .as_ref()
                .unwrap()
                .source_range,
            written_type_range
        );
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&wrong_source),
            vec![invalid_key.to_owned()]
        );
        let mut missing_expected = super::source_reserved_object_variable_inequality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce an expected-input corruption target");
        missing_expected.right_expected_input = None;
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&missing_expected),
            vec![invalid_key.to_owned()]
        );
        let pre_output = super::extract_source_reserved_object_variable_inequality(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce a pre-output payload");
        let mismatched_symbols = SymbolEnv::new(
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("other_module")),
            SymbolEnvIndexes::default(),
        );
        assert_eq!(
            source_reserved_variable_formula_result_detail_keys(
                build_source_reserved_variable_formula_output(pre_output, &mismatched_symbols),
                invalid_key,
            ),
            vec![invalid_key.to_owned()]
        );

        let near_misses = [
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("object"))],
                "OtherPayloadBoundary",
                "x",
                "<>",
                "x",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("object"))],
                "ReservedObjectVariableInequalityPayloadBoundary",
                "x",
                "<>",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("object"))],
                "ReservedObjectVariableInequalityPayloadBoundary",
                "y",
                "<>",
                "x",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("object"))],
                "ReservedObjectVariableInequalityPayloadBoundary",
                "x",
                "=",
                "x",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
                "ReservedObjectVariableInequalityPayloadBoundary",
                "x",
                "<>",
                "x",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![reserve_item(
                    vec!["x", "y"],
                    ReserveTypeShape::Builtin("object"),
                )],
                "ReservedObjectVariableInequalityPayloadBoundary",
                "x",
                "<>",
                "x",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::AttributedObject)],
                "ReservedObjectVariableInequalityPayloadBoundary",
                "x",
                "<>",
                "x",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbolWithArgs("Mode"),
                )],
                "ReservedObjectVariableInequalityPayloadBoundary",
                "x",
                "<>",
                "x",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::Builtin("object")),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("object")),
                ],
                "ReservedObjectVariableInequalityPayloadBoundary",
                "x",
                "<>",
                "x",
            ),
            reserve_then_identifier_binary_theorem_ast_with_options(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("object"))],
                IdentifierBinaryTheoremSpec {
                    status: Some("open"),
                    label: "ReservedObjectVariableInequalityPayloadBoundary",
                    left: "x",
                    operator: "<>",
                    right: "x",
                    recovered_label: false,
                },
            ),
            reserve_then_identifier_binary_theorem_ast_with_options(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("object"))],
                IdentifierBinaryTheoremSpec {
                    status: Some("registration"),
                    label: "ReservedObjectVariableInequalityPayloadBoundary",
                    left: "x",
                    operator: "<>",
                    right: "x",
                    recovered_label: false,
                },
            ),
            reserve_then_identifier_binary_theorem_ast_with_options(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("object"))],
                IdentifierBinaryTheoremSpec {
                    status: None,
                    label: "ReservedObjectVariableInequalityPayloadBoundary",
                    left: "x",
                    operator: "<>",
                    right: "x",
                    recovered_label: true,
                },
            ),
            reserve_then_two_identifier_binary_theorems_ast(
                source_id,
                "ReservedObjectVariableInequalityPayloadBoundary",
                "<>",
            ),
            theorem_then_reserve_identifier_binary_ast(
                source_id,
                "ReservedObjectVariableInequalityPayloadBoundary",
                "<>",
            ),
            reserve_then_builtin_binary_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("object"))],
                "ReservedObjectVariableInequalityPayloadBoundary",
                "1",
                "<>",
                "1",
            ),
        ];
        for near_miss in near_misses {
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }

        for (near_miss_symbols, shape) in [
            (source_mode_symbol_env(module.clone()), "Mode"),
            (source_structure_symbol_env(module.clone()), "Struct"),
            (imported_mode_symbol_env(module.clone()), "Mode"),
            (imported_structure_symbol_env(module.clone()), "Struct"),
            (ambiguous_mode_symbol_env(module.clone()), "Mode"),
            (ambiguous_structure_symbol_env(module.clone()), "Struct"),
        ] {
            let near_miss = reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbol(shape),
                )],
                "ReservedObjectVariableInequalityPayloadBoundary",
                "x",
                "<>",
                "x",
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &near_miss_symbols,),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
    }

    #[test]
    fn source_reserved_object_variable_type_assertion_bridge_preserves_builtin_object_identity() {
        let source_id = source_id(189);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("reserved_object_variable_type_assertion"),
        );
        let symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        let exact_spec = exact_reserved_object_identifier_type_assertion_spec();
        let exact = reserve_then_identifier_type_assertion_theorem_ast(
            source_id,
            vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("object"))],
            exact_spec,
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        let payload = super::extract_source_reserved_object_variable_type_assertion(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact builtin-object reserved-variable type assertion should extract");
        let [source_binding] = payload.reserve.bridge.bindings() else {
            panic!("exact builtin-object reserve should produce one binding");
        };
        assert_eq!(source_binding.spelling, "x");
        assert_eq!(source_binding.type_spelling, "object");
        assert_eq!(source_binding.type_head, TypeHeadInput::BuiltinObject);
        assert_eq!(payload.subject_spelling, "x");
        assert_eq!(payload.subject_lookup_ordinal, 1);
        assert_eq!(payload.asserted_type.spelling, "object");
        assert_eq!(payload.asserted_type.head, TypeHeadInput::BuiltinObject);
        assert_ne!(source_binding.type_range, payload.asserted_type.range);

        let output = super::source_reserved_object_variable_type_assertion_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact builtin-object type assertion should reach the checker");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("builtin-object type-assertion invariants should hold");
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(
            output.subject_result_input.head,
            TypeHeadInput::BuiltinObject
        );
        assert_eq!(
            output.asserted_type_input.head,
            TypeHeadInput::BuiltinObject
        );
        assert_eq!(
            output.subject_result_input.source_range,
            source_binding.type_range
        );
        assert_eq!(output.subject_result_input.spelling, "object");
        assert_eq!(
            output.asserted_type_input.source_range,
            payload.asserted_type.range
        );
        assert_eq!(output.asserted_type_input.spelling, "object");
        assert_ne!(
            output.subject_result_input.site,
            output.asserted_type_input.site
        );
        assert_ne!(
            output.subject_result_input.source_range,
            output.asserted_type_input.source_range
        );
        assert!(output.subject_result_input.args.is_empty());
        assert!(output.subject_result_input.attributes.is_empty());
        assert!(output.asserted_type_input.args.is_empty());
        assert!(output.asserted_type_input.attributes.is_empty());
        assert_eq!(output.term_formula.terms().len(), 1);
        assert_eq!(output.term_formula.formulas().len(), 1);
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert!(output.term_formula.candidate_sets().is_empty());
        assert!(output.term_formula.facts().is_empty());
        assert!(output.term_formula.diagnostics().is_empty());
        let (_, term) = output
            .term_formula
            .terms()
            .iter()
            .next()
            .expect("builtin-object subject term should exist");
        assert_eq!(term.status, TermStatus::Inferred);
        assert!(term.expected_type.is_none());
        assert!(term.deferred.is_empty());
        let (_, formula) = output
            .term_formula
            .formulas()
            .iter()
            .next()
            .expect("builtin-object type assertion should exist");
        assert_eq!(formula.kind, FormulaKind::TypeAssertion);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert!(formula.expected_types.is_empty());
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("builtin-object normalized type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_eq!(normalized.source.range, source_binding.type_range);
        assert_eq!(normalized.source.spelling, "object");

        let invalid_key =
            super::TYPE_ELABORATION_RESERVED_OBJECT_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY;
        let invalid_output_keys = |output| {
            source_reserved_variable_type_assertion_result_detail_keys(Ok(output), invalid_key)
        };

        let mut wrong_binding = super::source_reserved_object_variable_type_assertion_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce a binding corruption target");
        wrong_binding.subject_binding = BindingId::new(1);
        assert_eq!(
            invalid_output_keys(wrong_binding),
            vec![invalid_key.to_owned()]
        );

        let mut wrong_ordinal = super::extract_source_reserved_object_variable_type_assertion(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce an ordinal corruption target");
        wrong_ordinal.subject_lookup_ordinal = 2;
        assert_eq!(
            source_reserved_variable_type_assertion_result_detail_keys(
                build_source_reserved_variable_type_assertion_output(wrong_ordinal, &symbols),
                invalid_key,
            ),
            vec![invalid_key.to_owned()]
        );

        let mut wrong_subject_head = super::source_reserved_object_variable_type_assertion_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce a subject-head corruption target");
        wrong_subject_head.subject_result_input.head = TypeHeadInput::BuiltinSet;
        assert_eq!(
            invalid_output_keys(wrong_subject_head),
            vec![invalid_key.to_owned()]
        );

        let mut wrong_asserted_head = super::source_reserved_object_variable_type_assertion_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce an asserted-head corruption target");
        wrong_asserted_head.asserted_type_input.head = TypeHeadInput::BuiltinSet;
        assert_eq!(
            invalid_output_keys(wrong_asserted_head),
            vec![invalid_key.to_owned()]
        );

        let mut collapsed_site = super::source_reserved_object_variable_type_assertion_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce a site corruption target");
        collapsed_site.asserted_type_input.site = collapsed_site.subject_result_input.site.clone();
        assert_eq!(
            invalid_output_keys(collapsed_site),
            vec![invalid_key.to_owned()]
        );

        let mut wrong_subject_source =
            super::source_reserved_object_variable_type_assertion_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce a subject-source corruption target");
        wrong_subject_source.subject_result_input.source_range = payload.subject_range;
        assert_ne!(
            wrong_subject_source.subject_result_input.source_range,
            source_binding.type_range
        );
        assert_eq!(
            invalid_output_keys(wrong_subject_source),
            vec![invalid_key.to_owned()]
        );

        let mut collapsed_source = super::source_reserved_object_variable_type_assertion_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce an asserted-source corruption target");
        collapsed_source.asserted_type_input.source_range = source_binding.type_range;
        assert_eq!(
            invalid_output_keys(collapsed_source),
            vec![invalid_key.to_owned()]
        );

        let pre_output = super::extract_source_reserved_object_variable_type_assertion(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce a matched-output failure target");
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
                invalid_key,
            ),
            vec![invalid_key.to_owned()]
        );

        let near_misses = [
            reserve_then_identifier_type_assertion_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("object"))],
                IdentifierTypeAssertionTheoremSpec {
                    label: "OtherPayloadBoundary",
                    ..exact_spec
                },
            ),
            reserve_then_identifier_type_assertion_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("object"))],
                IdentifierTypeAssertionTheoremSpec {
                    subject: "y",
                    ..exact_spec
                },
            ),
            reserve_then_builtin_type_assertion_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("object"))],
                exact_spec.label,
                "1",
                ReserveTypeShape::Builtin("object"),
            ),
            reserve_then_identifier_type_assertion_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
                exact_spec,
            ),
            reserve_then_identifier_type_assertion_theorem_ast(
                source_id,
                vec![reserve_item(
                    vec!["x", "y"],
                    ReserveTypeShape::Builtin("object"),
                )],
                exact_spec,
            ),
            reserve_then_identifier_type_assertion_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::AttributedObject)],
                exact_spec,
            ),
            reserve_then_identifier_type_assertion_theorem_ast(
                source_id,
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbolWithArgs("Mode"),
                )],
                exact_spec,
            ),
            reserve_then_identifier_type_assertion_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("object"))],
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::Builtin("set"),
                    ..exact_spec
                },
            ),
            reserve_then_identifier_type_assertion_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("object"))],
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::AttributedObject,
                    ..exact_spec
                },
            ),
            reserve_then_identifier_type_assertion_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("object"))],
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbolWithArgs("Mode"),
                    ..exact_spec
                },
            ),
            reserve_then_identifier_type_assertion_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("object"))],
                IdentifierTypeAssertionTheoremSpec {
                    negated: true,
                    ..exact_spec
                },
            ),
            reserve_then_identifier_type_assertion_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("object"))],
                IdentifierTypeAssertionTheoremSpec {
                    status: Some("open"),
                    ..exact_spec
                },
            ),
            reserve_then_identifier_type_assertion_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("object"))],
                IdentifierTypeAssertionTheoremSpec {
                    status: Some("registration"),
                    ..exact_spec
                },
            ),
            reserve_then_identifier_type_assertion_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("object"))],
                IdentifierTypeAssertionTheoremSpec {
                    recovered_label: true,
                    ..exact_spec
                },
            ),
            reserve_then_identifier_type_assertion_theorem_ast(
                source_id,
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::Builtin("object")),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("object")),
                ],
                exact_spec,
            ),
        ];
        for near_miss in near_misses {
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }

        let provenance_near_misses = [
            (
                ReserveTypeShape::QualifiedSymbol("Mode"),
                ReserveTypeShape::Builtin("object"),
                source_mode_symbol_env(module.clone()),
            ),
            (
                ReserveTypeShape::QualifiedSymbol("Struct"),
                ReserveTypeShape::Builtin("object"),
                source_structure_symbol_env(module.clone()),
            ),
            (
                ReserveTypeShape::QualifiedSymbol("Mode"),
                ReserveTypeShape::Builtin("object"),
                imported_mode_symbol_env(module.clone()),
            ),
            (
                ReserveTypeShape::QualifiedSymbol("Struct"),
                ReserveTypeShape::Builtin("object"),
                imported_structure_symbol_env(module.clone()),
            ),
            (
                ReserveTypeShape::QualifiedSymbol("Mode"),
                ReserveTypeShape::Builtin("object"),
                ambiguous_mode_symbol_env(module.clone()),
            ),
            (
                ReserveTypeShape::QualifiedSymbol("Struct"),
                ReserveTypeShape::Builtin("object"),
                ambiguous_structure_symbol_env(module.clone()),
            ),
            (
                ReserveTypeShape::Builtin("object"),
                ReserveTypeShape::QualifiedSymbol("Mode"),
                source_mode_symbol_env(module.clone()),
            ),
            (
                ReserveTypeShape::Builtin("object"),
                ReserveTypeShape::QualifiedSymbol("Struct"),
                source_structure_symbol_env(module.clone()),
            ),
            (
                ReserveTypeShape::Builtin("object"),
                ReserveTypeShape::QualifiedSymbol("Mode"),
                imported_mode_symbol_env(module.clone()),
            ),
            (
                ReserveTypeShape::Builtin("object"),
                ReserveTypeShape::QualifiedSymbol("Struct"),
                imported_structure_symbol_env(module.clone()),
            ),
            (
                ReserveTypeShape::Builtin("object"),
                ReserveTypeShape::QualifiedSymbol("Mode"),
                ambiguous_mode_symbol_env(module.clone()),
            ),
            (
                ReserveTypeShape::Builtin("object"),
                ReserveTypeShape::QualifiedSymbol("Struct"),
                ambiguous_structure_symbol_env(module.clone()),
            ),
        ];
        for (reserve_type, asserted_type, near_miss_symbols) in provenance_near_misses {
            let near_miss = reserve_then_identifier_type_assertion_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], reserve_type)],
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type,
                    ..exact_spec
                },
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &near_miss_symbols,),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
    }

    #[test]
    fn source_distinct_reserved_variable_equality_bridge_preserves_binding_identity() {
        let source_id = source_id(123);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("distinct_reserved_variable_equality"),
        );
        let symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        let exact = reserve_then_identifier_equality_theorem_ast(
            source_id,
            vec![reserve_item(
                vec!["x", "y"],
                ReserveTypeShape::Builtin("set"),
            )],
            "DistinctReservedVariableEqualityPayloadBoundary",
            "x",
            "y",
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        let payload =
            extract_source_distinct_reserved_variable_equality(&exact, module.clone(), &symbols)
                .expect("exact distinct reserved-variable equality source should extract");
        assert_eq!(payload.reserve.bridge.bindings().len(), 2);
        assert_eq!(payload.reserve.bridge.bindings()[0].spelling, "x");
        assert_eq!(payload.reserve.bridge.bindings()[1].spelling, "y");
        assert_eq!(
            payload.reserve.bridge.bindings()[0].type_range,
            payload.reserve.bridge.bindings()[1].type_range
        );
        assert_eq!(payload.left_lookup_ordinal, 2);
        assert_eq!(payload.right_lookup_ordinal, 3);

        let output =
            source_distinct_reserved_variable_equality_output(&exact, module.clone(), &symbols)
                .expect("exact distinct reserved-variable equality should reach the checker");
        assert_source_reserved_variable_formula_output(&output)
            .expect("distinct reserved-variable equality invariants should hold");
        assert_eq!(output.left_binding, BindingId::new(0));
        assert_eq!(output.right_binding, BindingId::new(1));
        assert_ne!(output.left_binding, output.right_binding);
        assert_eq!(output.handoff.binding_env.bindings().len(), 2);
        assert_eq!(output.handoff.declarations.declarations().len(), 2);
        for (site, binding) in [
            (&output.payload.left_site, output.left_binding),
            (&output.payload.right_site, output.right_binding),
        ] {
            let term = output
                .term_formula
                .terms()
                .iter()
                .map(|(_, term)| term)
                .find(|term| &term.site == site)
                .expect("distinct reserved-variable term should be checked");
            assert_eq!(term.reference, Some(TermReference::Binding(binding)));
            assert_eq!(term.status, TermStatus::Inferred);
        }
        let formula = output
            .term_formula
            .formulas()
            .iter()
            .map(|(_, formula)| formula)
            .next()
            .expect("distinct reserved-variable equality formula should be checked");
        assert_eq!(formula.kind, FormulaKind::Equality);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert_eq!(formula.expected_types.len(), 2);
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
                "distinct-reserved-variable-left-expected".to_owned(),
                "distinct-reserved-variable-left-result".to_owned(),
                "distinct-reserved-variable-right-expected".to_owned(),
                "distinct-reserved-variable-right-result".to_owned(),
            ])
        );

        let mut invalid_output =
            source_distinct_reserved_variable_equality_output(&exact, module.clone(), &symbols)
                .expect("exact source should produce a second checker output");
        invalid_output.right_binding = BindingId::new(0);
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&invalid_output),
            vec![
                TYPE_ELABORATION_DISTINCT_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY.to_owned()
            ]
        );

        let near_misses = [
            reserve_then_identifier_equality_theorem_ast(
                source_id,
                vec![reserve_item(
                    vec!["x", "y"],
                    ReserveTypeShape::Builtin("set"),
                )],
                "OtherPayloadBoundary",
                "x",
                "y",
            ),
            reserve_then_identifier_equality_theorem_ast(
                source_id,
                vec![reserve_item(
                    vec!["x", "y"],
                    ReserveTypeShape::Builtin("set"),
                )],
                "DistinctReservedVariableEqualityPayloadBoundary",
                "y",
                "x",
            ),
            reserve_then_identifier_equality_theorem_ast(
                source_id,
                vec![reserve_item(
                    vec!["x", "y"],
                    ReserveTypeShape::Builtin("set"),
                )],
                "DistinctReservedVariableEqualityPayloadBoundary",
                "x",
                "x",
            ),
            reserve_then_identifier_equality_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
                "DistinctReservedVariableEqualityPayloadBoundary",
                "x",
                "y",
            ),
            reserve_then_identifier_equality_theorem_ast(
                source_id,
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::Builtin("set")),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                ],
                "DistinctReservedVariableEqualityPayloadBoundary",
                "x",
                "y",
            ),
            reserve_then_identifier_equality_theorem_ast(
                source_id,
                vec![reserve_item(
                    vec!["x", "y"],
                    ReserveTypeShape::Builtin("object"),
                )],
                "DistinctReservedVariableEqualityPayloadBoundary",
                "x",
                "y",
            ),
            reserve_then_identifier_equality_theorem_ast(
                source_id,
                vec![reserve_item(
                    vec!["x", "y", "z"],
                    ReserveTypeShape::Builtin("set"),
                )],
                "DistinctReservedVariableEqualityPayloadBoundary",
                "x",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![reserve_item(
                    vec!["x", "y"],
                    ReserveTypeShape::Builtin("set"),
                )],
                "DistinctReservedVariableEqualityPayloadBoundary",
                "x",
                "<>",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast_with_options(
                source_id,
                vec![reserve_item(
                    vec!["x", "y"],
                    ReserveTypeShape::Builtin("set"),
                )],
                IdentifierBinaryTheoremSpec {
                    status: Some("registration"),
                    label: "DistinctReservedVariableEqualityPayloadBoundary",
                    left: "x",
                    operator: "=",
                    right: "y",
                    recovered_label: false,
                },
            ),
            reserve_then_two_identifier_binary_theorems_ast(
                source_id,
                "DistinctReservedVariableEqualityPayloadBoundary",
                "=",
            ),
            reserve_then_builtin_equality_theorem_ast(
                source_id,
                vec![reserve_item(
                    vec!["x", "y"],
                    ReserveTypeShape::Builtin("set"),
                )],
                "DistinctReservedVariableEqualityPayloadBoundary",
                "1",
                "1",
            ),
        ];
        for near_miss in near_misses {
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
    }

    #[test]
    fn source_distinct_reserved_object_variable_equality_bridge_preserves_shared_object_provenance()
    {
        let source_id = source_id(191);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("distinct_reserved_object_variable_equality"),
        );
        let symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        let exact = reserve_then_identifier_equality_theorem_ast(
            source_id,
            vec![reserve_item(
                vec!["x", "y"],
                ReserveTypeShape::Builtin("object"),
            )],
            "DistinctReservedObjectVariableEqualityPayloadBoundary",
            "x",
            "y",
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        let payload = super::extract_source_distinct_reserved_object_variable_equality(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact distinct reserved-object-variable equality should extract");
        assert_eq!(payload.reserve.bridge.bindings().len(), 2);
        assert_eq!(payload.reserve.bridge.bindings()[0].spelling, "x");
        assert_eq!(payload.reserve.bridge.bindings()[1].spelling, "y");
        assert_eq!(payload.left_lookup_ordinal, 2);
        assert_eq!(payload.right_lookup_ordinal, 3);
        let written_type_range = payload.reserve.bridge.bindings()[0].type_range;
        assert_eq!(
            payload.reserve.bridge.bindings()[1].type_range,
            written_type_range
        );
        assert!(payload.reserve.bridge.bindings().iter().all(|binding| {
            binding.type_spelling == "object" && binding.type_head == TypeHeadInput::BuiltinObject
        }));

        let output = super::source_distinct_reserved_object_variable_equality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact distinct reserved-object-variable equality should reach the checker");
        assert_source_reserved_variable_formula_output(&output)
            .expect("distinct reserved-object-variable equality invariants should hold");
        assert_eq!(output.left_binding, BindingId::new(0));
        assert_eq!(output.right_binding, BindingId::new(1));
        assert_ne!(output.left_binding, output.right_binding);
        assert_eq!(output.handoff.binding_env.bindings().len(), 2);
        assert_eq!(output.handoff.declarations.declarations().len(), 2);
        let role_sites = [
            &output.left_result_input.site,
            &output.right_result_input.site,
            &output.left_expected_input.as_ref().unwrap().site,
            &output.right_expected_input.as_ref().unwrap().site,
        ]
        .into_iter()
        .collect::<BTreeSet<_>>();
        assert_eq!(role_sites.len(), 4);
        for input in [
            &output.left_result_input,
            &output.right_result_input,
            output.left_expected_input.as_ref().unwrap(),
            output.right_expected_input.as_ref().unwrap(),
        ] {
            assert_eq!(
                input.site.node(),
                if input.site == output.left_result_input.site
                    || input.site == output.left_expected_input.as_ref().unwrap().site
                {
                    payload.left_site.node()
                } else {
                    payload.right_site.node()
                }
            );
            assert_eq!(input.source_range, written_type_range);
            assert_eq!(input.spelling, "object");
            assert_eq!(input.head, TypeHeadInput::BuiltinObject);
            assert!(input.args.is_empty());
            assert!(input.attributes.is_empty());
        }
        assert_eq!(output.term_formula.terms().len(), 2);
        for (site, binding) in [
            (&payload.left_site, output.left_binding),
            (&payload.right_site, output.right_binding),
        ] {
            let term = output
                .term_formula
                .terms()
                .iter()
                .map(|(_, term)| term)
                .find(|term| &term.site == site)
                .expect("distinct reserved-object-variable term should be checked");
            assert_eq!(term.reference, Some(TermReference::Binding(binding)));
            assert_eq!(term.status, TermStatus::Inferred);
            assert!(term.deferred.is_empty());
        }
        assert_eq!(output.term_formula.type_entries().len(), 6);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert!(output.term_formula.candidate_sets().is_empty());
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("distinct reserved-object normalized identity should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_eq!(normalized.source.range, written_type_range);
        assert_eq!(normalized.source.spelling, "object");
        let formula = output
            .term_formula
            .formulas()
            .iter()
            .map(|(_, formula)| formula)
            .next()
            .expect("distinct reserved-object equality formula should be checked");
        assert_eq!(formula.kind, FormulaKind::Equality);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert_eq!(
            formula.terms,
            [payload.left_site.clone(), payload.right_site.clone()]
        );
        assert_eq!(formula.expected_types.len(), 2);
        assert_eq!(formula.expected_types[0].term, payload.left_site);
        assert_eq!(formula.expected_types[1].term, payload.right_site);
        assert!(formula.candidate_set.is_none());
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
        assert!(output.term_formula.facts().is_empty());
        assert!(output.term_formula.diagnostics().is_empty());

        let invalid_key =
            super::TYPE_ELABORATION_DISTINCT_RESERVED_OBJECT_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY;
        let mut wrong_binding = super::source_distinct_reserved_object_variable_equality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce a binding corruption target");
        wrong_binding.right_binding = BindingId::new(0);
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&wrong_binding),
            vec![invalid_key.to_owned()]
        );
        let mut wrong_ordinal = super::extract_source_distinct_reserved_object_variable_equality(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce an ordinal corruption target");
        wrong_ordinal.left_lookup_ordinal = wrong_ordinal.right_lookup_ordinal;
        assert_eq!(
            source_reserved_variable_formula_result_detail_keys(
                build_source_reserved_variable_formula_output(wrong_ordinal, &symbols),
                invalid_key,
            ),
            vec![invalid_key.to_owned()]
        );
        let mut wrong_head = super::source_distinct_reserved_object_variable_equality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce a head corruption target");
        wrong_head.left_result_input.head = TypeHeadInput::BuiltinSet;
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&wrong_head),
            vec![invalid_key.to_owned()]
        );
        let mut collapsed_role = super::source_distinct_reserved_object_variable_equality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce a role corruption target");
        collapsed_role.left_expected_input.as_mut().unwrap().site =
            collapsed_role.left_result_input.site.clone();
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&collapsed_role),
            vec![invalid_key.to_owned()]
        );
        let mut wrong_source = super::source_distinct_reserved_object_variable_equality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce a source corruption target");
        wrong_source
            .right_expected_input
            .as_mut()
            .unwrap()
            .source_range = payload.right_range;
        assert_ne!(
            wrong_source
                .right_expected_input
                .as_ref()
                .unwrap()
                .source_range,
            written_type_range
        );
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&wrong_source),
            vec![invalid_key.to_owned()]
        );
        let mut missing_expected = super::source_distinct_reserved_object_variable_equality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce an expected-input corruption target");
        missing_expected.right_expected_input = None;
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&missing_expected),
            vec![invalid_key.to_owned()]
        );
        let pre_output = super::extract_source_distinct_reserved_object_variable_equality(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce a pre-output payload");
        let mismatched_symbols = SymbolEnv::new(
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("other_module")),
            SymbolEnvIndexes::default(),
        );
        assert_eq!(
            source_reserved_variable_formula_result_detail_keys(
                build_source_reserved_variable_formula_output(pre_output, &mismatched_symbols),
                invalid_key,
            ),
            vec![invalid_key.to_owned()]
        );

        let exact_spec = IdentifierBinaryTheoremSpec {
            status: None,
            label: "DistinctReservedObjectVariableEqualityPayloadBoundary",
            left: "x",
            operator: "=",
            right: "y",
            recovered_label: false,
        };
        let near_misses = [
            reserve_then_identifier_equality_theorem_ast(
                source_id,
                vec![reserve_item(
                    vec!["x", "y"],
                    ReserveTypeShape::Builtin("object"),
                )],
                "OtherPayloadBoundary",
                "x",
                "y",
            ),
            reserve_then_identifier_equality_theorem_ast(
                source_id,
                vec![reserve_item(
                    vec!["x", "y"],
                    ReserveTypeShape::Builtin("object"),
                )],
                exact_spec.label,
                "y",
                "x",
            ),
            reserve_then_identifier_equality_theorem_ast(
                source_id,
                vec![reserve_item(
                    vec!["x", "y"],
                    ReserveTypeShape::Builtin("object"),
                )],
                exact_spec.label,
                "x",
                "x",
            ),
            reserve_then_identifier_equality_theorem_ast(
                source_id,
                vec![reserve_item(
                    vec!["x", "y"],
                    ReserveTypeShape::Builtin("object"),
                )],
                exact_spec.label,
                "x",
                "z",
            ),
            reserve_then_identifier_equality_theorem_ast(
                source_id,
                vec![reserve_item(
                    vec!["x", "y"],
                    ReserveTypeShape::Builtin("object"),
                )],
                exact_spec.label,
                "z",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![reserve_item(
                    vec!["x", "y"],
                    ReserveTypeShape::Builtin("object"),
                )],
                exact_spec.label,
                "x",
                "<>",
                "y",
            ),
            reserve_then_identifier_equality_theorem_ast(
                source_id,
                vec![reserve_item(
                    vec!["x", "y"],
                    ReserveTypeShape::Builtin("set"),
                )],
                exact_spec.label,
                "x",
                "y",
            ),
            reserve_then_identifier_equality_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("object"))],
                exact_spec.label,
                "x",
                "y",
            ),
            reserve_then_identifier_equality_theorem_ast(
                source_id,
                vec![reserve_item(
                    vec!["x", "y", "z"],
                    ReserveTypeShape::Builtin("object"),
                )],
                exact_spec.label,
                "x",
                "y",
            ),
            reserve_then_identifier_equality_theorem_ast(
                source_id,
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::Builtin("object")),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("object")),
                ],
                exact_spec.label,
                "x",
                "y",
            ),
            reserve_then_identifier_equality_theorem_ast(
                source_id,
                vec![reserve_item(
                    vec!["x", "y"],
                    ReserveTypeShape::AttributedObject,
                )],
                exact_spec.label,
                "x",
                "y",
            ),
            reserve_then_identifier_equality_theorem_ast(
                source_id,
                vec![reserve_item(
                    vec!["x", "y"],
                    ReserveTypeShape::QualifiedSymbolWithArgs("Mode"),
                )],
                exact_spec.label,
                "x",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast_with_options(
                source_id,
                vec![reserve_item(
                    vec!["x", "y"],
                    ReserveTypeShape::Builtin("object"),
                )],
                IdentifierBinaryTheoremSpec {
                    status: Some("open"),
                    ..exact_spec
                },
            ),
            reserve_then_identifier_binary_theorem_ast_with_options(
                source_id,
                vec![reserve_item(
                    vec!["x", "y"],
                    ReserveTypeShape::Builtin("object"),
                )],
                IdentifierBinaryTheoremSpec {
                    status: Some("registration"),
                    ..exact_spec
                },
            ),
            reserve_then_identifier_binary_theorem_ast_with_options(
                source_id,
                vec![reserve_item(
                    vec!["x", "y"],
                    ReserveTypeShape::Builtin("object"),
                )],
                IdentifierBinaryTheoremSpec {
                    recovered_label: true,
                    ..exact_spec
                },
            ),
            reserve_then_two_identifier_binary_theorems_ast(source_id, exact_spec.label, "="),
            reserve_then_builtin_equality_theorem_ast(
                source_id,
                vec![reserve_item(
                    vec!["x", "y"],
                    ReserveTypeShape::Builtin("object"),
                )],
                exact_spec.label,
                "1",
                "1",
            ),
        ];
        for near_miss in near_misses {
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }

        let provenance_near_misses = [
            (
                ReserveTypeShape::QualifiedSymbol("Mode"),
                source_mode_symbol_env(module.clone()),
            ),
            (
                ReserveTypeShape::QualifiedSymbol("Struct"),
                source_structure_symbol_env(module.clone()),
            ),
            (
                ReserveTypeShape::QualifiedSymbol("Mode"),
                imported_mode_symbol_env(module.clone()),
            ),
            (
                ReserveTypeShape::QualifiedSymbol("Struct"),
                imported_structure_symbol_env(module.clone()),
            ),
            (
                ReserveTypeShape::QualifiedSymbol("Mode"),
                ambiguous_mode_symbol_env(module.clone()),
            ),
            (
                ReserveTypeShape::QualifiedSymbol("Struct"),
                ambiguous_structure_symbol_env(module.clone()),
            ),
        ];
        for (reserve_type, near_miss_symbols) in provenance_near_misses {
            let near_miss = reserve_then_identifier_equality_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x", "y"], reserve_type)],
                exact_spec.label,
                "x",
                "y",
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &near_miss_symbols,),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
    }

    #[test]
    fn source_distinct_reserved_object_variable_inequality_bridge_preserves_shared_object_provenance()
     {
        let source_id = source_id(192);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("distinct_reserved_object_variable_inequality"),
        );
        let symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        let exact = reserve_then_identifier_binary_theorem_ast(
            source_id,
            vec![reserve_item(
                vec!["x", "y"],
                ReserveTypeShape::Builtin("object"),
            )],
            "DistinctReservedObjectVariableInequalityPayloadBoundary",
            "x",
            "<>",
            "y",
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        let payload = super::extract_source_distinct_reserved_object_variable_inequality(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact distinct reserved-object-variable inequality should extract");
        assert_eq!(payload.reserve.bridge.bindings().len(), 2);
        assert_eq!(payload.reserve.bridge.bindings()[0].spelling, "x");
        assert_eq!(payload.reserve.bridge.bindings()[1].spelling, "y");
        assert_eq!(payload.left_lookup_ordinal, 2);
        assert_eq!(payload.right_lookup_ordinal, 3);
        let written_type_range = payload.reserve.bridge.bindings()[0].type_range;
        assert_eq!(
            payload.reserve.bridge.bindings()[1].type_range,
            written_type_range
        );
        assert!(payload.reserve.bridge.bindings().iter().all(|binding| {
            binding.type_spelling == "object" && binding.type_head == TypeHeadInput::BuiltinObject
        }));

        let output = super::source_distinct_reserved_object_variable_inequality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact distinct reserved-object-variable inequality should reach the checker");
        assert_source_reserved_variable_formula_output(&output)
            .expect("distinct reserved-object-variable inequality invariants should hold");
        assert_eq!(output.left_binding, BindingId::new(0));
        assert_eq!(output.right_binding, BindingId::new(1));
        assert_ne!(output.left_binding, output.right_binding);
        assert_eq!(output.handoff.binding_env.bindings().len(), 2);
        assert_eq!(output.handoff.declarations.declarations().len(), 2);
        let role_sites = [
            &output.left_result_input.site,
            &output.right_result_input.site,
            &output.left_expected_input.as_ref().unwrap().site,
            &output.right_expected_input.as_ref().unwrap().site,
        ]
        .into_iter()
        .collect::<BTreeSet<_>>();
        assert_eq!(role_sites.len(), 4);
        for input in [
            &output.left_result_input,
            &output.right_result_input,
            output.left_expected_input.as_ref().unwrap(),
            output.right_expected_input.as_ref().unwrap(),
        ] {
            assert_eq!(input.source_range, written_type_range);
            assert_eq!(input.spelling, "object");
            assert_eq!(input.head, TypeHeadInput::BuiltinObject);
            assert!(input.args.is_empty());
            assert!(input.attributes.is_empty());
        }
        assert_eq!(output.term_formula.terms().len(), 2);
        for (site, binding) in [
            (&payload.left_site, output.left_binding),
            (&payload.right_site, output.right_binding),
        ] {
            let term = output
                .term_formula
                .terms()
                .iter()
                .map(|(_, term)| term)
                .find(|term| &term.site == site)
                .expect("distinct reserved-object-variable term should be checked");
            assert_eq!(term.reference, Some(TermReference::Binding(binding)));
            assert_eq!(term.status, TermStatus::Inferred);
            assert!(term.deferred.is_empty());
        }
        assert_eq!(output.term_formula.type_entries().len(), 6);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert!(output.term_formula.candidate_sets().is_empty());
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("distinct reserved-object normalized identity should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_eq!(normalized.source.range, written_type_range);
        assert_eq!(normalized.source.spelling, "object");
        let formula = output
            .term_formula
            .formulas()
            .iter()
            .map(|(_, formula)| formula)
            .next()
            .expect("distinct reserved-object inequality formula should be checked");
        assert_eq!(formula.kind, FormulaKind::Inequality);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert_eq!(
            formula.terms,
            [payload.left_site.clone(), payload.right_site.clone()]
        );
        assert_eq!(formula.expected_types.len(), 2);
        assert_eq!(formula.expected_types[0].term, payload.left_site);
        assert_eq!(formula.expected_types[1].term, payload.right_site);
        assert!(formula.candidate_set.is_none());
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
        assert!(output.term_formula.facts().is_empty());
        assert!(output.term_formula.diagnostics().is_empty());

        let invalid_key =
            super::TYPE_ELABORATION_DISTINCT_RESERVED_OBJECT_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY;
        let mut wrong_binding = super::source_distinct_reserved_object_variable_inequality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce a binding corruption target");
        wrong_binding.right_binding = BindingId::new(0);
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&wrong_binding),
            vec![invalid_key.to_owned()]
        );
        let mut wrong_ordinal = super::extract_source_distinct_reserved_object_variable_inequality(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce an ordinal corruption target");
        wrong_ordinal.left_lookup_ordinal = wrong_ordinal.right_lookup_ordinal;
        assert_eq!(
            source_reserved_variable_formula_result_detail_keys(
                build_source_reserved_variable_formula_output(wrong_ordinal, &symbols),
                invalid_key,
            ),
            vec![invalid_key.to_owned()]
        );
        let mut wrong_head = super::source_distinct_reserved_object_variable_inequality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce a head corruption target");
        wrong_head.left_result_input.head = TypeHeadInput::BuiltinSet;
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&wrong_head),
            vec![invalid_key.to_owned()]
        );
        let mut collapsed_role = super::source_distinct_reserved_object_variable_inequality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce a role corruption target");
        collapsed_role.left_expected_input.as_mut().unwrap().site =
            collapsed_role.left_result_input.site.clone();
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&collapsed_role),
            vec![invalid_key.to_owned()]
        );
        let mut wrong_source = super::source_distinct_reserved_object_variable_inequality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce a source corruption target");
        wrong_source
            .right_expected_input
            .as_mut()
            .unwrap()
            .source_range = payload.right_range;
        assert_ne!(
            wrong_source
                .right_expected_input
                .as_ref()
                .unwrap()
                .source_range,
            written_type_range
        );
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&wrong_source),
            vec![invalid_key.to_owned()]
        );
        let mut missing_expected =
            super::source_distinct_reserved_object_variable_inequality_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce an expected-input corruption target");
        missing_expected.right_expected_input = None;
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&missing_expected),
            vec![invalid_key.to_owned()]
        );
        let pre_output = super::extract_source_distinct_reserved_object_variable_inequality(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce a pre-output payload");
        let mismatched_symbols = SymbolEnv::new(
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("other_module")),
            SymbolEnvIndexes::default(),
        );
        assert_eq!(
            source_reserved_variable_formula_result_detail_keys(
                build_source_reserved_variable_formula_output(pre_output, &mismatched_symbols),
                invalid_key,
            ),
            vec![invalid_key.to_owned()]
        );

        let exact_spec = IdentifierBinaryTheoremSpec {
            status: None,
            label: "DistinctReservedObjectVariableInequalityPayloadBoundary",
            left: "x",
            operator: "<>",
            right: "y",
            recovered_label: false,
        };
        let exact_reserve = || {
            vec![reserve_item(
                vec!["x", "y"],
                ReserveTypeShape::Builtin("object"),
            )]
        };
        let near_misses = [
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                exact_reserve(),
                "OtherPayloadBoundary",
                "x",
                "<>",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                exact_reserve(),
                exact_spec.label,
                "y",
                "<>",
                "x",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                exact_reserve(),
                exact_spec.label,
                "x",
                "<>",
                "x",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                exact_reserve(),
                exact_spec.label,
                "x",
                "<>",
                "z",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                exact_reserve(),
                exact_spec.label,
                "z",
                "<>",
                "y",
            ),
            reserve_then_identifier_equality_theorem_ast(
                source_id,
                exact_reserve(),
                exact_spec.label,
                "x",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![reserve_item(
                    vec!["x", "y"],
                    ReserveTypeShape::Builtin("set"),
                )],
                exact_spec.label,
                "x",
                "<>",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("object"))],
                exact_spec.label,
                "x",
                "<>",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![reserve_item(
                    vec!["x", "y", "z"],
                    ReserveTypeShape::Builtin("object"),
                )],
                exact_spec.label,
                "x",
                "<>",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::Builtin("object")),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("object")),
                ],
                exact_spec.label,
                "x",
                "<>",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![reserve_item(
                    vec!["x", "y"],
                    ReserveTypeShape::AttributedObject,
                )],
                exact_spec.label,
                "x",
                "<>",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![reserve_item(
                    vec!["x", "y"],
                    ReserveTypeShape::QualifiedSymbolWithArgs("Mode"),
                )],
                exact_spec.label,
                "x",
                "<>",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast_with_options(
                source_id,
                exact_reserve(),
                IdentifierBinaryTheoremSpec {
                    status: Some("open"),
                    ..exact_spec
                },
            ),
            reserve_then_identifier_binary_theorem_ast_with_options(
                source_id,
                exact_reserve(),
                IdentifierBinaryTheoremSpec {
                    status: Some("registration"),
                    ..exact_spec
                },
            ),
            reserve_then_identifier_binary_theorem_ast_with_options(
                source_id,
                exact_reserve(),
                IdentifierBinaryTheoremSpec {
                    recovered_label: true,
                    ..exact_spec
                },
            ),
            reserve_then_two_identifier_binary_theorems_with_options_ast(
                source_id,
                exact_reserve(),
                exact_spec,
            ),
            reserve_then_builtin_binary_theorem_ast(
                source_id,
                exact_reserve(),
                exact_spec.label,
                "1",
                "<>",
                "1",
            ),
        ];
        for near_miss in near_misses {
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }

        let provenance_near_misses = [
            (
                ReserveTypeShape::QualifiedSymbol("Mode"),
                source_mode_symbol_env(module.clone()),
            ),
            (
                ReserveTypeShape::QualifiedSymbol("Struct"),
                source_structure_symbol_env(module.clone()),
            ),
            (
                ReserveTypeShape::QualifiedSymbol("Mode"),
                imported_mode_symbol_env(module.clone()),
            ),
            (
                ReserveTypeShape::QualifiedSymbol("Struct"),
                imported_structure_symbol_env(module.clone()),
            ),
            (
                ReserveTypeShape::QualifiedSymbol("Mode"),
                ambiguous_mode_symbol_env(module.clone()),
            ),
            (
                ReserveTypeShape::QualifiedSymbol("Struct"),
                ambiguous_structure_symbol_env(module.clone()),
            ),
        ];
        for (reserve_type, near_miss_symbols) in provenance_near_misses {
            let near_miss = reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x", "y"], reserve_type)],
                exact_spec.label,
                "x",
                "<>",
                "y",
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &near_miss_symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
    }

    #[test]
    fn source_distinct_reserved_variable_inequality_bridge_preserves_shared_provenance() {
        let source_id = source_id(160);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("distinct_reserved_variable_inequality"),
        );
        let symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        let shared_reserve = || {
            vec![reserve_item(
                vec!["x", "y"],
                ReserveTypeShape::Builtin("set"),
            )]
        };
        let exact = reserve_then_identifier_binary_theorem_ast(
            source_id,
            shared_reserve(),
            "DistinctReservedVariableInequalityPayloadBoundary",
            "x",
            "<>",
            "y",
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        let payload =
            extract_source_distinct_reserved_variable_inequality(&exact, module.clone(), &symbols)
                .expect("exact distinct reserved-variable inequality source should extract");
        assert_eq!(payload.reserve.bridge.bindings().len(), 2);
        assert_eq!(payload.reserve.bridge.bindings()[0].spelling, "x");
        assert_eq!(payload.reserve.bridge.bindings()[1].spelling, "y");
        assert_eq!(
            payload.reserve.bridge.bindings()[0].type_range,
            payload.reserve.bridge.bindings()[1].type_range
        );
        assert_eq!(payload.left_lookup_ordinal, 2);
        assert_eq!(payload.right_lookup_ordinal, 3);

        let output =
            source_distinct_reserved_variable_inequality_output(&exact, module.clone(), &symbols)
                .expect("exact distinct reserved-variable inequality should reach the checker");
        assert_source_reserved_variable_formula_output(&output)
            .expect("distinct reserved-variable inequality invariants should hold");
        assert_eq!(output.left_binding, BindingId::new(0));
        assert_eq!(output.right_binding, BindingId::new(1));
        assert_ne!(output.left_binding, output.right_binding);
        assert_eq!(output.handoff.binding_env.bindings().len(), 2);
        assert_eq!(output.handoff.declarations.declarations().len(), 2);
        let shared_type_range = output.payload.reserve.bridge.bindings()[0].type_range;
        assert_eq!(output.left_result_input.source_range, shared_type_range);
        assert_eq!(output.right_result_input.source_range, shared_type_range);
        assert_eq!(
            output
                .left_expected_input
                .as_ref()
                .expect("left expected input should exist")
                .source_range,
            shared_type_range
        );
        assert_eq!(
            output
                .right_expected_input
                .as_ref()
                .expect("right expected input should exist")
                .source_range,
            shared_type_range
        );
        assert_eq!(output.term_formula.type_entries().len(), 6);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert!(output.term_formula.candidate_sets().is_empty());
        assert!(output.term_formula.facts().is_empty());
        assert!(output.term_formula.diagnostics().is_empty());
        for (site, binding) in [
            (&output.payload.left_site, output.left_binding),
            (&output.payload.right_site, output.right_binding),
        ] {
            let term = output
                .term_formula
                .terms()
                .iter()
                .map(|(_, term)| term)
                .find(|term| &term.site == site)
                .expect("distinct reserved-variable inequality term should be checked");
            assert_eq!(term.reference, Some(TermReference::Binding(binding)));
            assert_eq!(term.status, TermStatus::Inferred);
            assert!(term.deferred.is_empty());
        }
        let formula = output
            .term_formula
            .formulas()
            .iter()
            .map(|(_, formula)| formula)
            .next()
            .expect("distinct reserved-variable inequality formula should be checked");
        assert_eq!(formula.kind, FormulaKind::Inequality);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert_eq!(formula.expected_types.len(), 2);
        assert_eq!(formula.expected_types[0].term, output.payload.left_site);
        assert_eq!(formula.expected_types[1].term, output.payload.right_site);
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
                "distinct-reserved-variable-inequality-left-expected".to_owned(),
                "distinct-reserved-variable-inequality-left-result".to_owned(),
                "distinct-reserved-variable-inequality-right-expected".to_owned(),
                "distinct-reserved-variable-inequality-right-result".to_owned(),
            ])
        );

        let invalid_key = || {
            vec![
                TYPE_ELABORATION_DISTINCT_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY
                    .to_owned(),
            ]
        };
        let mut collapsed_binding =
            source_distinct_reserved_variable_inequality_output(&exact, module.clone(), &symbols)
                .expect("exact source should produce a second checker output");
        collapsed_binding.right_binding = BindingId::new(0);
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&collapsed_binding),
            invalid_key()
        );

        let mut corrupted_result =
            source_distinct_reserved_variable_inequality_output(&exact, module.clone(), &symbols)
                .expect("exact source should produce a third checker output");
        corrupted_result.right_result_input.source_range = corrupted_result.payload.right_range;
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&corrupted_result),
            invalid_key()
        );

        let mut missing_left_expected =
            source_distinct_reserved_variable_inequality_output(&exact, module.clone(), &symbols)
                .expect("exact source should produce a fourth checker output");
        missing_left_expected.left_expected_input = None;
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&missing_left_expected),
            invalid_key()
        );

        let mut missing_right_expected =
            source_distinct_reserved_variable_inequality_output(&exact, module.clone(), &symbols)
                .expect("exact source should produce a fifth checker output");
        missing_right_expected.right_expected_input = None;
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&missing_right_expected),
            invalid_key()
        );

        let mut swapped_expected =
            source_distinct_reserved_variable_inequality_output(&exact, module.clone(), &symbols)
                .expect("exact source should produce a sixth checker output");
        std::mem::swap(
            &mut swapped_expected.left_expected_input,
            &mut swapped_expected.right_expected_input,
        );
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&swapped_expected),
            invalid_key()
        );

        let mut corrupted_expected =
            source_distinct_reserved_variable_inequality_output(&exact, module.clone(), &symbols)
                .expect("exact source should produce a seventh checker output");
        corrupted_expected
            .right_expected_input
            .as_mut()
            .expect("right expected input should exist")
            .head = TypeHeadInput::BuiltinObject;
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&corrupted_expected),
            invalid_key()
        );

        let near_misses = [
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                shared_reserve(),
                "OtherPayloadBoundary",
                "x",
                "<>",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                shared_reserve(),
                "DistinctReservedVariableInequalityPayloadBoundary",
                "y",
                "<>",
                "x",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                shared_reserve(),
                "DistinctReservedVariableInequalityPayloadBoundary",
                "x",
                "<>",
                "x",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
                "DistinctReservedVariableInequalityPayloadBoundary",
                "x",
                "<>",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::Builtin("set")),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                ],
                "DistinctReservedVariableInequalityPayloadBoundary",
                "x",
                "<>",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![reserve_item(
                    vec!["x", "y"],
                    ReserveTypeShape::Builtin("object"),
                )],
                "DistinctReservedVariableInequalityPayloadBoundary",
                "x",
                "<>",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![reserve_item(
                    vec!["x", "y"],
                    ReserveTypeShape::AttributedSet,
                )],
                "DistinctReservedVariableInequalityPayloadBoundary",
                "x",
                "<>",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![reserve_item(
                    vec!["x", "y"],
                    ReserveTypeShape::NonBuiltin("Thing"),
                )],
                "DistinctReservedVariableInequalityPayloadBoundary",
                "x",
                "<>",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![reserve_item(
                    vec!["x", "y"],
                    ReserveTypeShape::QualifiedSymbolWithArgs("Mode"),
                )],
                "DistinctReservedVariableInequalityPayloadBoundary",
                "x",
                "<>",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![reserve_item(
                    vec!["x", "y", "z"],
                    ReserveTypeShape::Builtin("set"),
                )],
                "DistinctReservedVariableInequalityPayloadBoundary",
                "x",
                "<>",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                shared_reserve(),
                "DistinctReservedVariableInequalityPayloadBoundary",
                "x",
                "=",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast_with_options(
                source_id,
                shared_reserve(),
                IdentifierBinaryTheoremSpec {
                    status: Some("registration"),
                    label: "DistinctReservedVariableInequalityPayloadBoundary",
                    left: "x",
                    operator: "<>",
                    right: "y",
                    recovered_label: false,
                },
            ),
            reserve_then_identifier_binary_theorem_ast_with_options(
                source_id,
                shared_reserve(),
                IdentifierBinaryTheoremSpec {
                    status: None,
                    label: "DistinctReservedVariableInequalityPayloadBoundary",
                    left: "x",
                    operator: "<>",
                    right: "y",
                    recovered_label: true,
                },
            ),
            reserve_then_two_identifier_binary_theorems_ast(
                source_id,
                "DistinctReservedVariableInequalityPayloadBoundary",
                "<>",
            ),
            reserve_then_builtin_binary_theorem_ast(
                source_id,
                shared_reserve(),
                "DistinctReservedVariableInequalityPayloadBoundary",
                "1",
                "<>",
                "1",
            ),
            modes_then_empty_definition_reserve_identifier_binary_theorem_ast(
                source_id,
                [],
                shared_reserve(),
                IdentifierBinaryTheoremSpec {
                    status: None,
                    label: "DistinctReservedVariableInequalityPayloadBoundary",
                    left: "x",
                    operator: "<>",
                    right: "y",
                    recovered_label: false,
                },
            ),
        ];
        for near_miss in near_misses {
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
    }

    #[test]
    fn source_distinct_reserved_variable_membership_bridge_preserves_shared_provenance() {
        let source_id = source_id(159);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("distinct_reserved_variable_membership"),
        );
        let symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        let exact = reserve_then_identifier_binary_theorem_ast(
            source_id,
            vec![reserve_item(
                vec!["x", "y"],
                ReserveTypeShape::Builtin("set"),
            )],
            "DistinctReservedVariableMembershipPayloadBoundary",
            "x",
            "in",
            "y",
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        let payload =
            extract_source_distinct_reserved_variable_membership(&exact, module.clone(), &symbols)
                .expect("exact distinct reserved-variable membership source should extract");
        assert_eq!(payload.reserve.bridge.bindings().len(), 2);
        assert_eq!(payload.reserve.bridge.bindings()[0].spelling, "x");
        assert_eq!(payload.reserve.bridge.bindings()[1].spelling, "y");
        assert_eq!(
            payload.reserve.bridge.bindings()[0].type_range,
            payload.reserve.bridge.bindings()[1].type_range
        );
        assert_eq!(payload.left_lookup_ordinal, 2);
        assert_eq!(payload.right_lookup_ordinal, 3);

        let output =
            source_distinct_reserved_variable_membership_output(&exact, module.clone(), &symbols)
                .expect("exact distinct reserved-variable membership should reach the checker");
        assert_source_reserved_variable_formula_output(&output)
            .expect("distinct reserved-variable membership invariants should hold");
        assert_eq!(output.left_binding, BindingId::new(0));
        assert_eq!(output.right_binding, BindingId::new(1));
        assert_ne!(output.left_binding, output.right_binding);
        let shared_type_range = output.payload.reserve.bridge.bindings()[0].type_range;
        assert_eq!(output.left_result_input.source_range, shared_type_range);
        assert_eq!(output.right_result_input.source_range, shared_type_range);
        assert!(output.left_expected_input.is_none());
        assert_eq!(
            output
                .right_expected_input
                .as_ref()
                .expect("right membership expected input should exist")
                .source_range,
            shared_type_range
        );
        assert_eq!(output.term_formula.type_entries().len(), 5);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        let formula = output
            .term_formula
            .formulas()
            .iter()
            .map(|(_, formula)| formula)
            .next()
            .expect("distinct reserved-variable membership formula should be checked");
        assert_eq!(formula.kind, FormulaKind::Membership);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert_eq!(formula.expected_types.len(), 1);
        assert_eq!(formula.expected_types[0].term, output.payload.right_site);
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
        assert!(output.term_formula.facts().is_empty());
        assert!(output.term_formula.candidate_sets().is_empty());
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
                "distinct-reserved-variable-membership-left-result".to_owned(),
                "distinct-reserved-variable-membership-right-expected".to_owned(),
                "distinct-reserved-variable-membership-right-result".to_owned(),
            ])
        );

        let mut invalid_binding =
            source_distinct_reserved_variable_membership_output(&exact, module.clone(), &symbols)
                .expect("exact source should produce a second checker output");
        invalid_binding.right_binding = BindingId::new(0);
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&invalid_binding),
            vec![
                TYPE_ELABORATION_DISTINCT_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY
                    .to_owned()
            ]
        );

        let mut invalid_expected =
            source_distinct_reserved_variable_membership_output(&exact, module.clone(), &symbols)
                .expect("exact source should produce a third checker output");
        invalid_expected
            .right_expected_input
            .as_mut()
            .expect("right expected input should exist")
            .head = TypeHeadInput::BuiltinObject;
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&invalid_expected),
            vec![
                TYPE_ELABORATION_DISTINCT_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY
                    .to_owned()
            ]
        );

        let mut missing_expected =
            source_distinct_reserved_variable_membership_output(&exact, module.clone(), &symbols)
                .expect("exact source should produce a fourth checker output");
        missing_expected.right_expected_input = None;
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&missing_expected),
            vec![
                TYPE_ELABORATION_DISTINCT_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY
                    .to_owned()
            ]
        );

        let mut invalid_left_expected =
            source_distinct_reserved_variable_membership_output(&exact, module.clone(), &symbols)
                .expect("exact source should produce a fifth checker output");
        invalid_left_expected.left_expected_input =
            invalid_left_expected.right_expected_input.clone();
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&invalid_left_expected),
            vec![
                TYPE_ELABORATION_DISTINCT_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY
                    .to_owned()
            ]
        );

        let shared_reserve = || {
            vec![reserve_item(
                vec!["x", "y"],
                ReserveTypeShape::Builtin("set"),
            )]
        };
        let near_misses = [
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                shared_reserve(),
                "OtherPayloadBoundary",
                "x",
                "in",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                shared_reserve(),
                "DistinctReservedVariableMembershipPayloadBoundary",
                "y",
                "in",
                "x",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                shared_reserve(),
                "DistinctReservedVariableMembershipPayloadBoundary",
                "x",
                "in",
                "x",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                shared_reserve(),
                "DistinctReservedVariableMembershipPayloadBoundary",
                "y",
                "in",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
                "DistinctReservedVariableMembershipPayloadBoundary",
                "x",
                "in",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::Builtin("set")),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                ],
                "DistinctReservedVariableMembershipPayloadBoundary",
                "x",
                "in",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![reserve_item(
                    vec!["x", "y"],
                    ReserveTypeShape::Builtin("object"),
                )],
                "DistinctReservedVariableMembershipPayloadBoundary",
                "x",
                "in",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![reserve_item(
                    vec!["x", "y"],
                    ReserveTypeShape::AttributedSet,
                )],
                "DistinctReservedVariableMembershipPayloadBoundary",
                "x",
                "in",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![reserve_item(
                    vec!["x", "y"],
                    ReserveTypeShape::NonBuiltin("Thing"),
                )],
                "DistinctReservedVariableMembershipPayloadBoundary",
                "x",
                "in",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![reserve_item(
                    vec!["x", "y"],
                    ReserveTypeShape::QualifiedSymbolWithArgs("Mode"),
                )],
                "DistinctReservedVariableMembershipPayloadBoundary",
                "x",
                "in",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![reserve_item(
                    vec!["x", "y", "z"],
                    ReserveTypeShape::Builtin("set"),
                )],
                "DistinctReservedVariableMembershipPayloadBoundary",
                "x",
                "in",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![reserve_item(
                    vec!["x", "z"],
                    ReserveTypeShape::Builtin("set"),
                )],
                "DistinctReservedVariableMembershipPayloadBoundary",
                "x",
                "in",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                shared_reserve(),
                "DistinctReservedVariableMembershipPayloadBoundary",
                "x",
                "=",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast_with_options(
                source_id,
                shared_reserve(),
                IdentifierBinaryTheoremSpec {
                    status: Some("registration"),
                    label: "DistinctReservedVariableMembershipPayloadBoundary",
                    left: "x",
                    operator: "in",
                    right: "y",
                    recovered_label: false,
                },
            ),
            reserve_then_identifier_binary_theorem_ast_with_options(
                source_id,
                shared_reserve(),
                IdentifierBinaryTheoremSpec {
                    status: None,
                    label: "DistinctReservedVariableMembershipPayloadBoundary",
                    left: "x",
                    operator: "in",
                    right: "y",
                    recovered_label: true,
                },
            ),
            reserve_then_two_identifier_binary_theorems_ast(
                source_id,
                "DistinctReservedVariableMembershipPayloadBoundary",
                "in",
            ),
            reserve_then_builtin_binary_theorem_ast(
                source_id,
                shared_reserve(),
                "DistinctReservedVariableMembershipPayloadBoundary",
                "1",
                "in",
                "1",
            ),
            modes_then_empty_definition_reserve_identifier_binary_theorem_ast(
                source_id,
                [],
                shared_reserve(),
                IdentifierBinaryTheoremSpec {
                    status: None,
                    label: "DistinctReservedVariableMembershipPayloadBoundary",
                    left: "x",
                    operator: "in",
                    right: "y",
                    recovered_label: false,
                },
            ),
        ];
        for near_miss in near_misses {
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
    }

    #[test]
    fn source_multiple_object_reserve_declaration_inequality_bridge_preserves_distinct_object_provenance()
     {
        let source_id = source_id(194);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("multiple_object_reserve_declaration_inequality"),
        );
        let symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        let separate_reserves = || {
            vec![
                reserve_item(vec!["x"], ReserveTypeShape::Builtin("object")),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("object")),
            ]
        };
        let exact = reserve_then_identifier_binary_theorem_ast(
            source_id,
            separate_reserves(),
            "MultipleObjectReserveDeclarationInequalityPayloadBoundary",
            "x",
            "<>",
            "y",
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        assert!(
            super::extract_source_multiple_object_reserve_declaration_equality(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none()
        );
        assert!(
            super::extract_source_multiple_reserve_declaration_inequality(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none()
        );
        let payload = super::extract_source_multiple_object_reserve_declaration_inequality(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact multiple-object-reserve inequality should extract");
        assert_eq!(payload.reserve.bridge.bindings().len(), 2);
        assert_eq!(payload.reserve.bridge.bindings()[0].spelling, "x");
        assert_eq!(payload.reserve.bridge.bindings()[1].spelling, "y");
        assert_eq!(payload.left_lookup_ordinal, 2);
        assert_eq!(payload.right_lookup_ordinal, 3);
        let left_type_range = payload.reserve.bridge.bindings()[0].type_range;
        let right_type_range = payload.reserve.bridge.bindings()[1].type_range;
        assert!(
            (left_type_range.start, left_type_range.end)
                < (right_type_range.start, right_type_range.end)
        );
        assert!(payload.reserve.bridge.bindings().iter().all(|binding| {
            binding.type_spelling == "object" && binding.type_head == TypeHeadInput::BuiltinObject
        }));

        let output = super::source_multiple_object_reserve_declaration_inequality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact multiple-object-reserve inequality should reach the checker");
        assert_source_reserved_variable_formula_output(&output)
            .expect("multiple-object-reserve inequality invariants should hold");
        assert_eq!(output.left_binding, BindingId::new(0));
        assert_eq!(output.right_binding, BindingId::new(1));
        assert_eq!(output.handoff.binding_env.bindings().len(), 2);
        assert_eq!(output.handoff.declarations.declarations().len(), 2);
        let left_expected = output.left_expected_input.as_ref().unwrap();
        let right_expected = output.right_expected_input.as_ref().unwrap();
        let role_sites = [
            &output.left_result_input.site,
            &output.right_result_input.site,
            &left_expected.site,
            &right_expected.site,
        ]
        .into_iter()
        .collect::<BTreeSet<_>>();
        assert_eq!(role_sites.len(), 4);
        for input in [&output.left_result_input, left_expected] {
            assert_eq!(input.source_range, left_type_range);
            assert_eq!(input.spelling, "object");
            assert_eq!(input.head, TypeHeadInput::BuiltinObject);
            assert!(input.args.is_empty());
            assert!(input.attributes.is_empty());
        }
        for input in [&output.right_result_input, right_expected] {
            assert_eq!(input.source_range, right_type_range);
            assert_eq!(input.spelling, "object");
            assert_eq!(input.head, TypeHeadInput::BuiltinObject);
            assert!(input.args.is_empty());
            assert!(input.attributes.is_empty());
        }
        assert_eq!(output.term_formula.terms().len(), 2);
        for (site, binding) in [
            (&payload.left_site, output.left_binding),
            (&payload.right_site, output.right_binding),
        ] {
            let term = output
                .term_formula
                .terms()
                .iter()
                .map(|(_, term)| term)
                .find(|term| &term.site == site)
                .expect("multiple-object-reserve inequality term should be checked");
            assert_eq!(term.reference, Some(TermReference::Binding(binding)));
            assert_eq!(term.status, TermStatus::Inferred);
            assert!(term.deferred.is_empty());
        }
        assert_eq!(output.term_formula.type_entries().len(), 6);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert!(output.term_formula.candidate_sets().is_empty());
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("multiple-object-reserve inequality normalized identity should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_eq!(normalized.source.range, left_type_range);
        let formula = output
            .term_formula
            .formulas()
            .iter()
            .map(|(_, formula)| formula)
            .next()
            .expect("multiple-object-reserve inequality should be checked");
        assert_eq!(formula.kind, FormulaKind::Inequality);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert_eq!(
            formula.terms,
            [payload.left_site.clone(), payload.right_site.clone()]
        );
        assert_eq!(formula.expected_types.len(), 2);
        assert_eq!(formula.expected_types[0].term, payload.left_site);
        assert_eq!(formula.expected_types[1].term, payload.right_site);
        assert!(formula.candidate_set.is_none());
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
        assert!(output.term_formula.facts().is_empty());
        assert!(output.term_formula.diagnostics().is_empty());

        let invalid_key =
            super::TYPE_ELABORATION_MULTIPLE_OBJECT_RESERVE_DECLARATION_INEQUALITY_INVALID_PAYLOAD_KEY;
        let mut wrong_binding =
            super::source_multiple_object_reserve_declaration_inequality_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce a binding corruption target");
        wrong_binding.right_binding = BindingId::new(0);
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&wrong_binding),
            vec![invalid_key.to_owned()]
        );
        let mut wrong_ordinal =
            super::extract_source_multiple_object_reserve_declaration_inequality(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce an ordinal corruption target");
        wrong_ordinal.left_lookup_ordinal = wrong_ordinal.right_lookup_ordinal;
        assert_eq!(
            source_reserved_variable_formula_result_detail_keys(
                build_source_reserved_variable_formula_output(wrong_ordinal, &symbols),
                invalid_key,
            ),
            vec![invalid_key.to_owned()]
        );
        let mut collapsed_range =
            super::source_multiple_object_reserve_declaration_inequality_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce a range corruption target");
        collapsed_range.right_result_input.source_range = left_type_range;
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&collapsed_range),
            vec![invalid_key.to_owned()]
        );
        let mut collapsed_role =
            super::source_multiple_object_reserve_declaration_inequality_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce a role corruption target");
        collapsed_role.left_expected_input.as_mut().unwrap().site =
            collapsed_role.left_result_input.site.clone();
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&collapsed_role),
            vec![invalid_key.to_owned()]
        );
        let mut wrong_head = super::source_multiple_object_reserve_declaration_inequality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce a head corruption target");
        wrong_head.left_result_input.head = TypeHeadInput::BuiltinSet;
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&wrong_head),
            vec![invalid_key.to_owned()]
        );
        let mut wrong_source = super::source_multiple_object_reserve_declaration_inequality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce a raw-source corruption target");
        wrong_source
            .right_expected_input
            .as_mut()
            .unwrap()
            .source_range = payload.right_range;
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&wrong_source),
            vec![invalid_key.to_owned()]
        );
        let mut canonical_source_payload =
            super::extract_source_multiple_object_reserve_declaration_inequality(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce a canonical-source corruption target");
        let bridge_source_id = canonical_source_payload.reserve.bridge.source_id();
        let bridge_module_id = canonical_source_payload.reserve.bridge.module_id().clone();
        let bridge_source_range = canonical_source_payload.reserve.bridge.source_range();
        let mut reordered_type_ranges = canonical_source_payload.reserve.bridge.bindings().to_vec();
        reordered_type_ranges[0].type_range = right_type_range;
        reordered_type_ranges[1].type_range = left_type_range;
        canonical_source_payload.reserve.bridge = super::SourceReserveDeclarationBridge::new(
            bridge_source_id,
            bridge_module_id,
            bridge_source_range,
            reordered_type_ranges,
        )
        .expect("range-swapped reserve bridge should remain constructible");
        let canonical_source_corruption =
            build_source_reserved_variable_formula_output(canonical_source_payload, &symbols)
                .expect("range-swapped payload should build an immutable checker output");
        let corrupted_bindings = canonical_source_corruption
            .payload
            .reserve
            .bridge
            .bindings();
        assert_eq!(
            canonical_source_corruption.left_result_input.source_range,
            corrupted_bindings[0].type_range
        );
        assert_eq!(
            canonical_source_corruption.right_result_input.source_range,
            corrupted_bindings[1].type_range
        );
        let (_, corrupted_normalized) = canonical_source_corruption
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("corrupted output should contain one object identity");
        assert_eq!(
            corrupted_normalized.source.range,
            corrupted_bindings[1].type_range
        );
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&canonical_source_corruption),
            vec![invalid_key.to_owned()]
        );
        let mut missing_expected =
            super::source_multiple_object_reserve_declaration_inequality_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce an expected-input corruption target");
        missing_expected.right_expected_input = None;
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&missing_expected),
            vec![invalid_key.to_owned()]
        );
        let pre_output = super::extract_source_multiple_object_reserve_declaration_inequality(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce a module-corruption payload");
        let mismatched_symbols = SymbolEnv::new(
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("other_module")),
            SymbolEnvIndexes::default(),
        );
        assert_eq!(
            source_reserved_variable_formula_result_detail_keys(
                build_source_reserved_variable_formula_output(pre_output, &mismatched_symbols),
                invalid_key,
            ),
            vec![invalid_key.to_owned()]
        );

        let exact_spec = IdentifierBinaryTheoremSpec {
            status: None,
            label: "MultipleObjectReserveDeclarationInequalityPayloadBoundary",
            left: "x",
            operator: "<>",
            right: "y",
            recovered_label: false,
        };
        let near_misses = [
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                separate_reserves(),
                "OtherPayloadBoundary",
                "x",
                "<>",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                separate_reserves(),
                exact_spec.label,
                "y",
                "<>",
                "x",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                separate_reserves(),
                exact_spec.label,
                "x",
                "<>",
                "x",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                separate_reserves(),
                exact_spec.label,
                "z",
                "<>",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                separate_reserves(),
                exact_spec.label,
                "x",
                "<>",
                "z",
            ),
            reserve_then_identifier_equality_theorem_ast(
                source_id,
                separate_reserves(),
                exact_spec.label,
                "x",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![reserve_item(
                    vec!["x", "y"],
                    ReserveTypeShape::Builtin("object"),
                )],
                exact_spec.label,
                "x",
                "<>",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("object")),
                    reserve_item(vec!["x"], ReserveTypeShape::Builtin("object")),
                ],
                exact_spec.label,
                "x",
                "<>",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::Builtin("set")),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("object")),
                ],
                exact_spec.label,
                "x",
                "<>",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::Builtin("object")),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                ],
                exact_spec.label,
                "x",
                "<>",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::Builtin("object")),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("object")),
                    reserve_item(vec!["z"], ReserveTypeShape::Builtin("object")),
                ],
                exact_spec.label,
                "x",
                "<>",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::AttributedObject),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("object")),
                ],
                exact_spec.label,
                "x",
                "<>",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::QualifiedSymbolWithArgs("Mode")),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("object")),
                ],
                exact_spec.label,
                "x",
                "<>",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast_with_options(
                source_id,
                separate_reserves(),
                IdentifierBinaryTheoremSpec {
                    status: Some("registration"),
                    ..exact_spec
                },
            ),
            reserve_then_identifier_binary_theorem_ast_with_options(
                source_id,
                separate_reserves(),
                IdentifierBinaryTheoremSpec {
                    recovered_label: true,
                    ..exact_spec
                },
            ),
            reserve_then_two_identifier_binary_theorems_with_options_ast(
                source_id,
                separate_reserves(),
                exact_spec,
            ),
            reserve_then_builtin_binary_theorem_ast(
                source_id,
                separate_reserves(),
                exact_spec.label,
                "1",
                "<>",
                "1",
            ),
        ];
        for near_miss in near_misses {
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }

        let provenance_near_misses = [
            (
                ReserveTypeShape::QualifiedSymbol("Mode"),
                source_mode_symbol_env(module.clone()),
            ),
            (
                ReserveTypeShape::QualifiedSymbol("Struct"),
                source_structure_symbol_env(module.clone()),
            ),
            (
                ReserveTypeShape::QualifiedSymbol("Mode"),
                imported_mode_symbol_env(module.clone()),
            ),
            (
                ReserveTypeShape::QualifiedSymbol("Struct"),
                imported_structure_symbol_env(module.clone()),
            ),
            (
                ReserveTypeShape::QualifiedSymbol("Mode"),
                ambiguous_mode_symbol_env(module.clone()),
            ),
            (
                ReserveTypeShape::QualifiedSymbol("Struct"),
                ambiguous_structure_symbol_env(module.clone()),
            ),
        ];
        for (reserve_type, near_miss_symbols) in provenance_near_misses {
            let near_miss = reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![
                    reserve_item(vec!["x"], reserve_type),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("object")),
                ],
                exact_spec.label,
                "x",
                "<>",
                "y",
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &near_miss_symbols,),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
    }

    #[test]
    fn source_multiple_object_reserve_declaration_equality_bridge_preserves_distinct_object_provenance()
     {
        let source_id = source_id(193);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("multiple_object_reserve_declaration_equality"),
        );
        let symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        let separate_reserves = || {
            vec![
                reserve_item(vec!["x"], ReserveTypeShape::Builtin("object")),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("object")),
            ]
        };
        let exact = reserve_then_identifier_equality_theorem_ast(
            source_id,
            separate_reserves(),
            "MultipleObjectReserveDeclarationEqualityPayloadBoundary",
            "x",
            "y",
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        assert!(
            super::extract_source_multiple_reserve_declaration_equality(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none()
        );
        assert!(
            super::extract_source_distinct_reserved_object_variable_equality(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_none()
        );
        let payload = super::extract_source_multiple_object_reserve_declaration_equality(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact multiple-object-reserve equality should extract");
        assert_eq!(payload.reserve.bridge.bindings().len(), 2);
        assert_eq!(payload.reserve.bridge.bindings()[0].spelling, "x");
        assert_eq!(payload.reserve.bridge.bindings()[1].spelling, "y");
        assert_eq!(payload.left_lookup_ordinal, 2);
        assert_eq!(payload.right_lookup_ordinal, 3);
        let left_type_range = payload.reserve.bridge.bindings()[0].type_range;
        let right_type_range = payload.reserve.bridge.bindings()[1].type_range;
        assert_ne!(left_type_range, right_type_range);
        assert!(payload.reserve.bridge.bindings().iter().all(|binding| {
            binding.type_spelling == "object" && binding.type_head == TypeHeadInput::BuiltinObject
        }));

        let output = super::source_multiple_object_reserve_declaration_equality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact multiple-object-reserve equality should reach the checker");
        assert_source_reserved_variable_formula_output(&output)
            .expect("multiple-object-reserve equality invariants should hold");
        assert_eq!(output.left_binding, BindingId::new(0));
        assert_eq!(output.right_binding, BindingId::new(1));
        assert_ne!(output.left_binding, output.right_binding);
        assert_eq!(output.handoff.binding_env.bindings().len(), 2);
        assert_eq!(output.handoff.declarations.declarations().len(), 2);
        let left_expected = output
            .left_expected_input
            .as_ref()
            .expect("left expected object input should exist");
        let right_expected = output
            .right_expected_input
            .as_ref()
            .expect("right expected object input should exist");
        let role_sites = [
            &output.left_result_input.site,
            &output.right_result_input.site,
            &left_expected.site,
            &right_expected.site,
        ]
        .into_iter()
        .collect::<BTreeSet<_>>();
        assert_eq!(role_sites.len(), 4);
        for input in [&output.left_result_input, left_expected] {
            assert_eq!(input.source_range, left_type_range);
            assert_eq!(input.spelling, "object");
            assert_eq!(input.head, TypeHeadInput::BuiltinObject);
            assert!(input.args.is_empty());
            assert!(input.attributes.is_empty());
        }
        for input in [&output.right_result_input, right_expected] {
            assert_eq!(input.source_range, right_type_range);
            assert_eq!(input.spelling, "object");
            assert_eq!(input.head, TypeHeadInput::BuiltinObject);
            assert!(input.args.is_empty());
            assert!(input.attributes.is_empty());
        }
        assert_eq!(output.term_formula.terms().len(), 2);
        for (site, binding) in [
            (&payload.left_site, output.left_binding),
            (&payload.right_site, output.right_binding),
        ] {
            let term = output
                .term_formula
                .terms()
                .iter()
                .map(|(_, term)| term)
                .find(|term| &term.site == site)
                .expect("multiple-object-reserve term should be checked");
            assert_eq!(term.reference, Some(TermReference::Binding(binding)));
            assert_eq!(term.status, TermStatus::Inferred);
            assert!(term.deferred.is_empty());
        }
        assert_eq!(output.term_formula.type_entries().len(), 6);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert!(output.term_formula.candidate_sets().is_empty());
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("multiple-object-reserve normalized identity should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_eq!(normalized.source.range, left_type_range);
        assert_eq!(normalized.source.spelling, "object");
        let formula = output
            .term_formula
            .formulas()
            .iter()
            .map(|(_, formula)| formula)
            .next()
            .expect("multiple-object-reserve equality should be checked");
        assert_eq!(formula.kind, FormulaKind::Equality);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert_eq!(
            formula.terms,
            [payload.left_site.clone(), payload.right_site.clone()]
        );
        assert_eq!(formula.expected_types.len(), 2);
        assert_eq!(formula.expected_types[0].term, payload.left_site);
        assert_eq!(formula.expected_types[1].term, payload.right_site);
        assert!(formula.candidate_set.is_none());
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
        assert!(output.term_formula.facts().is_empty());
        assert!(output.term_formula.diagnostics().is_empty());

        let invalid_key =
            super::TYPE_ELABORATION_MULTIPLE_OBJECT_RESERVE_DECLARATION_EQUALITY_INVALID_PAYLOAD_KEY;
        let mut wrong_binding = super::source_multiple_object_reserve_declaration_equality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce a binding corruption target");
        wrong_binding.right_binding = BindingId::new(0);
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&wrong_binding),
            vec![invalid_key.to_owned()]
        );
        let mut wrong_ordinal = super::extract_source_multiple_object_reserve_declaration_equality(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce an ordinal corruption target");
        wrong_ordinal.left_lookup_ordinal = wrong_ordinal.right_lookup_ordinal;
        assert_eq!(
            source_reserved_variable_formula_result_detail_keys(
                build_source_reserved_variable_formula_output(wrong_ordinal, &symbols),
                invalid_key,
            ),
            vec![invalid_key.to_owned()]
        );
        let mut collapsed_range =
            super::source_multiple_object_reserve_declaration_equality_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce a range corruption target");
        collapsed_range.right_result_input.source_range = left_type_range;
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&collapsed_range),
            vec![invalid_key.to_owned()]
        );
        let mut collapsed_role = super::source_multiple_object_reserve_declaration_equality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce a role corruption target");
        collapsed_role.left_expected_input.as_mut().unwrap().site =
            collapsed_role.left_result_input.site.clone();
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&collapsed_role),
            vec![invalid_key.to_owned()]
        );
        let mut wrong_head = super::source_multiple_object_reserve_declaration_equality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce a builtin-head corruption target");
        wrong_head.left_result_input.head = TypeHeadInput::BuiltinSet;
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&wrong_head),
            vec![invalid_key.to_owned()]
        );
        let mut wrong_source = super::source_multiple_object_reserve_declaration_equality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce a source corruption target");
        wrong_source
            .right_expected_input
            .as_mut()
            .unwrap()
            .source_range = payload.right_range;
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&wrong_source),
            vec![invalid_key.to_owned()]
        );
        let mut canonical_source_payload =
            super::extract_source_multiple_object_reserve_declaration_equality(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce a canonical-source corruption target");
        let bridge_source_id = canonical_source_payload.reserve.bridge.source_id();
        let bridge_module_id = canonical_source_payload.reserve.bridge.module_id().clone();
        let bridge_source_range = canonical_source_payload.reserve.bridge.source_range();
        let mut reordered_type_ranges = canonical_source_payload.reserve.bridge.bindings().to_vec();
        reordered_type_ranges[0].type_range = right_type_range;
        reordered_type_ranges[1].type_range = left_type_range;
        canonical_source_payload.reserve.bridge = super::SourceReserveDeclarationBridge::new(
            bridge_source_id,
            bridge_module_id,
            bridge_source_range,
            reordered_type_ranges,
        )
        .expect("range-swapped reserve bridge should remain structurally constructible");
        let canonical_source_corruption =
            build_source_reserved_variable_formula_output(canonical_source_payload, &symbols)
                .expect("range-swapped payload should reach immutable checker output validation");
        let corrupted_bindings = canonical_source_corruption
            .payload
            .reserve
            .bridge
            .bindings();
        assert_eq!(
            canonical_source_corruption.left_result_input.source_range,
            corrupted_bindings[0].type_range
        );
        assert_eq!(
            canonical_source_corruption.right_result_input.source_range,
            corrupted_bindings[1].type_range
        );
        let (_, corrupted_normalized) = canonical_source_corruption
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("corrupted output should still contain one normalized object identity");
        assert_eq!(
            corrupted_normalized.source.range,
            corrupted_bindings[1].type_range
        );
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&canonical_source_corruption),
            vec![invalid_key.to_owned()]
        );
        let mut missing_expected =
            super::source_multiple_object_reserve_declaration_equality_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce an expected-input corruption target");
        missing_expected.right_expected_input = None;
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&missing_expected),
            vec![invalid_key.to_owned()]
        );
        let pre_output = super::extract_source_multiple_object_reserve_declaration_equality(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce a pre-output payload");
        let mismatched_symbols = SymbolEnv::new(
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("other_module")),
            SymbolEnvIndexes::default(),
        );
        assert_eq!(
            source_reserved_variable_formula_result_detail_keys(
                build_source_reserved_variable_formula_output(pre_output, &mismatched_symbols),
                invalid_key,
            ),
            vec![invalid_key.to_owned()]
        );

        let exact_spec = IdentifierBinaryTheoremSpec {
            status: None,
            label: "MultipleObjectReserveDeclarationEqualityPayloadBoundary",
            left: "x",
            operator: "=",
            right: "y",
            recovered_label: false,
        };
        let near_misses = [
            reserve_then_identifier_equality_theorem_ast(
                source_id,
                separate_reserves(),
                "OtherPayloadBoundary",
                "x",
                "y",
            ),
            reserve_then_identifier_equality_theorem_ast(
                source_id,
                separate_reserves(),
                exact_spec.label,
                "y",
                "x",
            ),
            reserve_then_identifier_equality_theorem_ast(
                source_id,
                separate_reserves(),
                exact_spec.label,
                "x",
                "x",
            ),
            reserve_then_identifier_equality_theorem_ast(
                source_id,
                separate_reserves(),
                exact_spec.label,
                "z",
                "y",
            ),
            reserve_then_identifier_equality_theorem_ast(
                source_id,
                separate_reserves(),
                exact_spec.label,
                "x",
                "z",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                separate_reserves(),
                exact_spec.label,
                "x",
                "<>",
                "y",
            ),
            reserve_then_identifier_equality_theorem_ast(
                source_id,
                vec![reserve_item(
                    vec!["x", "y"],
                    ReserveTypeShape::Builtin("object"),
                )],
                exact_spec.label,
                "x",
                "y",
            ),
            reserve_then_identifier_equality_theorem_ast(
                source_id,
                vec![
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("object")),
                    reserve_item(vec!["x"], ReserveTypeShape::Builtin("object")),
                ],
                exact_spec.label,
                "x",
                "y",
            ),
            reserve_then_identifier_equality_theorem_ast(
                source_id,
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::Builtin("set")),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("object")),
                ],
                exact_spec.label,
                "x",
                "y",
            ),
            reserve_then_identifier_equality_theorem_ast(
                source_id,
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::Builtin("object")),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                ],
                exact_spec.label,
                "x",
                "y",
            ),
            reserve_then_identifier_equality_theorem_ast(
                source_id,
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::Builtin("object")),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("object")),
                    reserve_item(vec!["z"], ReserveTypeShape::Builtin("object")),
                ],
                exact_spec.label,
                "x",
                "y",
            ),
            reserve_then_identifier_equality_theorem_ast(
                source_id,
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::AttributedObject),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("object")),
                ],
                exact_spec.label,
                "x",
                "y",
            ),
            reserve_then_identifier_equality_theorem_ast(
                source_id,
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::QualifiedSymbolWithArgs("Mode")),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("object")),
                ],
                exact_spec.label,
                "x",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast_with_options(
                source_id,
                separate_reserves(),
                IdentifierBinaryTheoremSpec {
                    status: Some("registration"),
                    ..exact_spec
                },
            ),
            reserve_then_identifier_binary_theorem_ast_with_options(
                source_id,
                separate_reserves(),
                IdentifierBinaryTheoremSpec {
                    recovered_label: true,
                    ..exact_spec
                },
            ),
            reserve_then_two_identifier_binary_theorems_with_options_ast(
                source_id,
                separate_reserves(),
                exact_spec,
            ),
            reserve_then_builtin_equality_theorem_ast(
                source_id,
                separate_reserves(),
                exact_spec.label,
                "1",
                "1",
            ),
        ];
        for near_miss in near_misses {
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }

        let provenance_near_misses = [
            (
                ReserveTypeShape::QualifiedSymbol("Mode"),
                source_mode_symbol_env(module.clone()),
            ),
            (
                ReserveTypeShape::QualifiedSymbol("Struct"),
                source_structure_symbol_env(module.clone()),
            ),
            (
                ReserveTypeShape::QualifiedSymbol("Mode"),
                imported_mode_symbol_env(module.clone()),
            ),
            (
                ReserveTypeShape::QualifiedSymbol("Struct"),
                imported_structure_symbol_env(module.clone()),
            ),
            (
                ReserveTypeShape::QualifiedSymbol("Mode"),
                ambiguous_mode_symbol_env(module.clone()),
            ),
            (
                ReserveTypeShape::QualifiedSymbol("Struct"),
                ambiguous_structure_symbol_env(module.clone()),
            ),
        ];
        for (reserve_type, near_miss_symbols) in provenance_near_misses {
            let near_miss = reserve_then_identifier_equality_theorem_ast(
                source_id,
                vec![
                    reserve_item(vec!["x"], reserve_type),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("object")),
                ],
                exact_spec.label,
                "x",
                "y",
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &near_miss_symbols,),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
    }

    #[test]
    fn source_multiple_reserve_declaration_equality_bridge_preserves_source_provenance() {
        let source_id = source_id(124);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("multiple_reserve_declaration_equality"),
        );
        let symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        let exact = reserve_then_identifier_equality_theorem_ast(
            source_id,
            vec![
                reserve_item(vec!["x"], ReserveTypeShape::Builtin("set")),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
            ],
            "MultipleReserveDeclarationEqualityPayloadBoundary",
            "x",
            "y",
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        let payload =
            extract_source_multiple_reserve_declaration_equality(&exact, module.clone(), &symbols)
                .expect("exact multiple-reserve equality source should extract");
        assert_eq!(payload.reserve.bridge.bindings().len(), 2);
        assert_eq!(payload.reserve.bridge.bindings()[0].spelling, "x");
        assert_eq!(payload.reserve.bridge.bindings()[1].spelling, "y");
        assert_ne!(
            payload.reserve.bridge.bindings()[0].type_range,
            payload.reserve.bridge.bindings()[1].type_range
        );
        assert_eq!(payload.left_lookup_ordinal, 2);
        assert_eq!(payload.right_lookup_ordinal, 3);

        let output =
            source_multiple_reserve_declaration_equality_output(&exact, module.clone(), &symbols)
                .expect("exact multiple-reserve equality should reach the checker");
        assert_source_reserved_variable_formula_output(&output)
            .expect("multiple-reserve equality invariants should hold");
        assert_eq!(output.left_binding, BindingId::new(0));
        assert_eq!(output.right_binding, BindingId::new(1));
        let source_bindings = output.payload.reserve.bridge.bindings();
        assert_eq!(
            output.left_result_input.source_range,
            source_bindings[0].type_range
        );
        assert_eq!(
            output
                .left_expected_input
                .as_ref()
                .expect("left expected type input should exist")
                .source_range,
            source_bindings[0].type_range
        );
        assert_eq!(
            output.right_result_input.source_range,
            source_bindings[1].type_range
        );
        assert_eq!(
            output
                .right_expected_input
                .as_ref()
                .expect("right expected type input should exist")
                .source_range,
            source_bindings[1].type_range
        );
        assert_eq!(output.term_formula.normalized_types().len(), 1);
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
                "multiple-reserve-declaration-left-expected".to_owned(),
                "multiple-reserve-declaration-left-result".to_owned(),
                "multiple-reserve-declaration-right-expected".to_owned(),
                "multiple-reserve-declaration-right-result".to_owned(),
            ])
        );

        let mut invalid_output =
            source_multiple_reserve_declaration_equality_output(&exact, module.clone(), &symbols)
                .expect("exact source should produce a second checker output");
        invalid_output.right_result_input.source_range = source_bindings[0].type_range;
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&invalid_output),
            vec![
                TYPE_ELABORATION_MULTIPLE_RESERVE_DECLARATION_EQUALITY_INVALID_PAYLOAD_KEY
                    .to_owned()
            ]
        );

        let near_misses = [
            reserve_then_identifier_equality_theorem_ast(
                source_id,
                vec![reserve_item(
                    vec!["x", "y"],
                    ReserveTypeShape::Builtin("set"),
                )],
                "MultipleReserveDeclarationEqualityPayloadBoundary",
                "x",
                "y",
            ),
            reserve_then_identifier_equality_theorem_ast(
                source_id,
                vec![
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                    reserve_item(vec!["x"], ReserveTypeShape::Builtin("set")),
                ],
                "MultipleReserveDeclarationEqualityPayloadBoundary",
                "x",
                "y",
            ),
            reserve_then_identifier_equality_theorem_ast(
                source_id,
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::Builtin("set")),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                ],
                "MultipleReserveDeclarationEqualityPayloadBoundary",
                "y",
                "x",
            ),
            reserve_then_identifier_equality_theorem_ast(
                source_id,
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::Builtin("set")),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("object")),
                ],
                "MultipleReserveDeclarationEqualityPayloadBoundary",
                "x",
                "y",
            ),
            reserve_then_identifier_equality_theorem_ast(
                source_id,
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::Builtin("set")),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                    reserve_item(vec!["z"], ReserveTypeShape::Builtin("set")),
                ],
                "MultipleReserveDeclarationEqualityPayloadBoundary",
                "x",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::Builtin("set")),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                ],
                "MultipleReserveDeclarationEqualityPayloadBoundary",
                "x",
                "<>",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast_with_options(
                source_id,
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::Builtin("set")),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                ],
                IdentifierBinaryTheoremSpec {
                    status: Some("registration"),
                    label: "MultipleReserveDeclarationEqualityPayloadBoundary",
                    left: "x",
                    operator: "=",
                    right: "y",
                    recovered_label: false,
                },
            ),
            reserve_then_identifier_binary_theorem_ast_with_options(
                source_id,
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::Builtin("set")),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                ],
                IdentifierBinaryTheoremSpec {
                    status: None,
                    label: "MultipleReserveDeclarationEqualityPayloadBoundary",
                    left: "x",
                    operator: "=",
                    right: "y",
                    recovered_label: true,
                },
            ),
            reserve_then_two_identifier_binary_theorems_ast(
                source_id,
                "MultipleReserveDeclarationEqualityPayloadBoundary",
                "=",
            ),
            reserve_then_builtin_equality_theorem_ast(
                source_id,
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::Builtin("set")),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                ],
                "MultipleReserveDeclarationEqualityPayloadBoundary",
                "1",
                "1",
            ),
        ];
        for near_miss in near_misses {
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
    }

    #[test]
    fn source_multiple_reserve_declaration_inequality_bridge_preserves_source_provenance() {
        let source_id = source_id(161);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("multiple_reserve_declaration_inequality"),
        );
        let symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        let separate_reserves = || {
            vec![
                reserve_item(vec!["x"], ReserveTypeShape::Builtin("set")),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
            ]
        };
        let exact = reserve_then_identifier_binary_theorem_ast(
            source_id,
            separate_reserves(),
            "MultipleReserveDeclarationInequalityPayloadBoundary",
            "x",
            "<>",
            "y",
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        let payload = extract_source_multiple_reserve_declaration_inequality(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact multiple-reserve inequality source should extract");
        assert_eq!(payload.reserve.bridge.bindings().len(), 2);
        assert_eq!(payload.reserve.bridge.bindings()[0].spelling, "x");
        assert_eq!(payload.reserve.bridge.bindings()[1].spelling, "y");
        assert_ne!(
            payload.reserve.bridge.bindings()[0].type_range,
            payload.reserve.bridge.bindings()[1].type_range
        );
        assert_eq!(payload.left_lookup_ordinal, 2);
        assert_eq!(payload.right_lookup_ordinal, 3);

        let output =
            source_multiple_reserve_declaration_inequality_output(&exact, module.clone(), &symbols)
                .expect("exact multiple-reserve inequality should reach the checker");
        assert_source_reserved_variable_formula_output(&output)
            .expect("multiple-reserve inequality invariants should hold");
        assert_eq!(output.left_binding, BindingId::new(0));
        assert_eq!(output.right_binding, BindingId::new(1));
        assert_eq!(output.handoff.binding_env.bindings().len(), 2);
        assert_eq!(output.handoff.declarations.declarations().len(), 2);
        let source_bindings = output.payload.reserve.bridge.bindings();
        assert_ne!(source_bindings[0].type_range, source_bindings[1].type_range);
        assert_eq!(
            output.left_result_input.source_range,
            source_bindings[0].type_range
        );
        assert_eq!(
            output.right_result_input.source_range,
            source_bindings[1].type_range
        );
        assert_eq!(
            output
                .left_expected_input
                .as_ref()
                .expect("left expected input should exist")
                .source_range,
            source_bindings[0].type_range
        );
        assert_eq!(
            output
                .right_expected_input
                .as_ref()
                .expect("right expected input should exist")
                .source_range,
            source_bindings[1].type_range
        );
        assert_eq!(output.term_formula.type_entries().len(), 6);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("one canonical set identity should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_eq!(normalized.source.range, source_bindings[0].type_range);
        assert_eq!(normalized.source.spelling, "set");
        for (site, binding) in [
            (&output.payload.left_site, output.left_binding),
            (&output.payload.right_site, output.right_binding),
        ] {
            let term = output
                .term_formula
                .terms()
                .iter()
                .map(|(_, term)| term)
                .find(|term| &term.site == site)
                .expect("multiple-reserve inequality term should be checked");
            assert_eq!(term.reference, Some(TermReference::Binding(binding)));
            assert_eq!(term.status, TermStatus::Inferred);
            assert!(term.deferred.is_empty());
        }
        let formula = output
            .term_formula
            .formulas()
            .iter()
            .map(|(_, formula)| formula)
            .next()
            .expect("multiple-reserve inequality formula should be checked");
        assert_eq!(formula.kind, FormulaKind::Inequality);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert_eq!(formula.expected_types.len(), 2);
        assert_eq!(formula.expected_types[0].term, output.payload.left_site);
        assert_eq!(formula.expected_types[1].term, output.payload.right_site);
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
        assert!(output.term_formula.candidate_sets().is_empty());
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
                "multiple-reserve-declaration-inequality-left-expected".to_owned(),
                "multiple-reserve-declaration-inequality-left-result".to_owned(),
                "multiple-reserve-declaration-inequality-right-expected".to_owned(),
                "multiple-reserve-declaration-inequality-right-result".to_owned(),
            ])
        );

        let invalid_key = || {
            vec![
                TYPE_ELABORATION_MULTIPLE_RESERVE_DECLARATION_INEQUALITY_INVALID_PAYLOAD_KEY
                    .to_owned(),
            ]
        };
        let mut collapsed_binding =
            source_multiple_reserve_declaration_inequality_output(&exact, module.clone(), &symbols)
                .expect("exact source should produce a second output");
        collapsed_binding.right_binding = BindingId::new(0);
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&collapsed_binding),
            invalid_key()
        );

        let mut collapsed_range =
            source_multiple_reserve_declaration_inequality_output(&exact, module.clone(), &symbols)
                .expect("exact source should produce a third output");
        collapsed_range.right_result_input.source_range = source_bindings[0].type_range;
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&collapsed_range),
            invalid_key()
        );

        let mut missing_left_expected =
            source_multiple_reserve_declaration_inequality_output(&exact, module.clone(), &symbols)
                .expect("exact source should produce a fourth output");
        missing_left_expected.left_expected_input = None;
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&missing_left_expected),
            invalid_key()
        );

        let mut missing_right_expected =
            source_multiple_reserve_declaration_inequality_output(&exact, module.clone(), &symbols)
                .expect("exact source should produce a fifth output");
        missing_right_expected.right_expected_input = None;
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&missing_right_expected),
            invalid_key()
        );

        let mut swapped_expected =
            source_multiple_reserve_declaration_inequality_output(&exact, module.clone(), &symbols)
                .expect("exact source should produce a sixth output");
        std::mem::swap(
            &mut swapped_expected.left_expected_input,
            &mut swapped_expected.right_expected_input,
        );
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&swapped_expected),
            invalid_key()
        );

        let mut corrupted_expected =
            source_multiple_reserve_declaration_inequality_output(&exact, module.clone(), &symbols)
                .expect("exact source should produce a seventh output");
        corrupted_expected
            .right_expected_input
            .as_mut()
            .expect("right expected input should exist")
            .head = TypeHeadInput::BuiltinObject;
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&corrupted_expected),
            invalid_key()
        );

        let near_misses = [
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![reserve_item(
                    vec!["x", "y"],
                    ReserveTypeShape::Builtin("set"),
                )],
                "MultipleReserveDeclarationInequalityPayloadBoundary",
                "x",
                "<>",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                    reserve_item(vec!["x"], ReserveTypeShape::Builtin("set")),
                ],
                "MultipleReserveDeclarationInequalityPayloadBoundary",
                "x",
                "<>",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                separate_reserves(),
                "OtherPayloadBoundary",
                "x",
                "<>",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                separate_reserves(),
                "MultipleReserveDeclarationInequalityPayloadBoundary",
                "y",
                "<>",
                "x",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                separate_reserves(),
                "MultipleReserveDeclarationInequalityPayloadBoundary",
                "x",
                "<>",
                "x",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
                "MultipleReserveDeclarationInequalityPayloadBoundary",
                "x",
                "<>",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::Builtin("set")),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("object")),
                ],
                "MultipleReserveDeclarationInequalityPayloadBoundary",
                "x",
                "<>",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::Builtin("set")),
                    reserve_item(vec!["y"], ReserveTypeShape::AttributedSet),
                ],
                "MultipleReserveDeclarationInequalityPayloadBoundary",
                "x",
                "<>",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::Builtin("set")),
                    reserve_item(vec!["y"], ReserveTypeShape::NonBuiltin("Thing")),
                ],
                "MultipleReserveDeclarationInequalityPayloadBoundary",
                "x",
                "<>",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::Builtin("set")),
                    reserve_item(vec!["y"], ReserveTypeShape::QualifiedSymbolWithArgs("Mode")),
                ],
                "MultipleReserveDeclarationInequalityPayloadBoundary",
                "x",
                "<>",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::Builtin("set")),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                    reserve_item(vec!["z"], ReserveTypeShape::Builtin("set")),
                ],
                "MultipleReserveDeclarationInequalityPayloadBoundary",
                "x",
                "<>",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                separate_reserves(),
                "MultipleReserveDeclarationInequalityPayloadBoundary",
                "x",
                "=",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast_with_options(
                source_id,
                separate_reserves(),
                IdentifierBinaryTheoremSpec {
                    status: Some("registration"),
                    label: "MultipleReserveDeclarationInequalityPayloadBoundary",
                    left: "x",
                    operator: "<>",
                    right: "y",
                    recovered_label: false,
                },
            ),
            reserve_then_identifier_binary_theorem_ast_with_options(
                source_id,
                separate_reserves(),
                IdentifierBinaryTheoremSpec {
                    status: None,
                    label: "MultipleReserveDeclarationInequalityPayloadBoundary",
                    left: "x",
                    operator: "<>",
                    right: "y",
                    recovered_label: true,
                },
            ),
            reserve_then_two_identifier_binary_theorems_ast(
                source_id,
                "MultipleReserveDeclarationInequalityPayloadBoundary",
                "<>",
            ),
            reserve_then_builtin_binary_theorem_ast(
                source_id,
                separate_reserves(),
                "MultipleReserveDeclarationInequalityPayloadBoundary",
                "1",
                "<>",
                "1",
            ),
            modes_then_empty_definition_reserve_identifier_binary_theorem_ast(
                source_id,
                [],
                separate_reserves(),
                IdentifierBinaryTheoremSpec {
                    status: None,
                    label: "MultipleReserveDeclarationInequalityPayloadBoundary",
                    left: "x",
                    operator: "<>",
                    right: "y",
                    recovered_label: false,
                },
            ),
        ];
        for near_miss in near_misses {
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
    }

    #[test]
    fn source_multiple_reserve_declaration_membership_bridge_preserves_source_provenance() {
        let source_id = source_id(162);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("multiple_reserve_declaration_membership"),
        );
        let symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        let separate_reserves = || {
            vec![
                reserve_item(vec!["x"], ReserveTypeShape::Builtin("set")),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
            ]
        };
        let exact = reserve_then_identifier_binary_theorem_ast(
            source_id,
            separate_reserves(),
            "MultipleReserveDeclarationMembershipPayloadBoundary",
            "x",
            "in",
            "y",
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        let payload = extract_source_multiple_reserve_declaration_membership(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact multiple-reserve membership source should extract");
        assert_eq!(payload.reserve.bridge.bindings().len(), 2);
        assert_eq!(payload.reserve.bridge.bindings()[0].spelling, "x");
        assert_eq!(payload.reserve.bridge.bindings()[1].spelling, "y");
        assert_ne!(
            payload.reserve.bridge.bindings()[0].type_range,
            payload.reserve.bridge.bindings()[1].type_range
        );
        assert_eq!(payload.left_lookup_ordinal, 2);
        assert_eq!(payload.right_lookup_ordinal, 3);

        let output =
            source_multiple_reserve_declaration_membership_output(&exact, module.clone(), &symbols)
                .expect("exact multiple-reserve membership should reach the checker");
        assert_source_reserved_variable_formula_output(&output)
            .expect("multiple-reserve membership invariants should hold");
        assert_eq!(output.left_binding, BindingId::new(0));
        assert_eq!(output.right_binding, BindingId::new(1));
        assert_eq!(output.handoff.binding_env.bindings().len(), 2);
        assert_eq!(output.handoff.declarations.declarations().len(), 2);
        let source_bindings = output.payload.reserve.bridge.bindings();
        assert_ne!(source_bindings[0].type_range, source_bindings[1].type_range);
        assert_eq!(
            output.left_result_input.source_range,
            source_bindings[0].type_range
        );
        assert_eq!(
            output.right_result_input.source_range,
            source_bindings[1].type_range
        );
        assert!(output.left_expected_input.is_none());
        assert_eq!(
            output
                .right_expected_input
                .as_ref()
                .expect("right expected input should exist")
                .source_range,
            source_bindings[1].type_range
        );
        assert_eq!(output.term_formula.type_entries().len(), 5);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("one canonical set identity should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_eq!(normalized.source.range, source_bindings[0].type_range);
        assert_eq!(normalized.source.spelling, "set");
        for (site, binding) in [
            (&output.payload.left_site, output.left_binding),
            (&output.payload.right_site, output.right_binding),
        ] {
            let term = output
                .term_formula
                .terms()
                .iter()
                .map(|(_, term)| term)
                .find(|term| &term.site == site)
                .expect("multiple-reserve membership term should be checked");
            assert_eq!(term.reference, Some(TermReference::Binding(binding)));
            assert_eq!(term.status, TermStatus::Inferred);
            assert!(term.deferred.is_empty());
        }
        let formula = output
            .term_formula
            .formulas()
            .iter()
            .map(|(_, formula)| formula)
            .next()
            .expect("multiple-reserve membership formula should be checked");
        assert_eq!(formula.kind, FormulaKind::Membership);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert_eq!(formula.expected_types.len(), 1);
        assert_eq!(formula.expected_types[0].term, output.payload.right_site);
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
        assert!(output.term_formula.candidate_sets().is_empty());
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
                "multiple-reserve-declaration-membership-left-result".to_owned(),
                "multiple-reserve-declaration-membership-right-expected".to_owned(),
                "multiple-reserve-declaration-membership-right-result".to_owned(),
            ])
        );

        let invalid_key = || {
            vec![
                TYPE_ELABORATION_MULTIPLE_RESERVE_DECLARATION_MEMBERSHIP_INVALID_PAYLOAD_KEY
                    .to_owned(),
            ]
        };
        let mut collapsed_binding =
            source_multiple_reserve_declaration_membership_output(&exact, module.clone(), &symbols)
                .expect("exact source should produce a second output");
        collapsed_binding.right_binding = BindingId::new(0);
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&collapsed_binding),
            invalid_key()
        );

        let mut collapsed_range =
            source_multiple_reserve_declaration_membership_output(&exact, module.clone(), &symbols)
                .expect("exact source should produce a third output");
        collapsed_range.right_result_input.source_range = source_bindings[0].type_range;
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&collapsed_range),
            invalid_key()
        );

        let mut collapsed_expected_range =
            source_multiple_reserve_declaration_membership_output(&exact, module.clone(), &symbols)
                .expect("exact source should produce a fourth output");
        collapsed_expected_range
            .right_expected_input
            .as_mut()
            .expect("right expected input should exist")
            .source_range = source_bindings[0].type_range;
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&collapsed_expected_range),
            invalid_key()
        );

        let mut invalid_left_expected =
            source_multiple_reserve_declaration_membership_output(&exact, module.clone(), &symbols)
                .expect("exact source should produce a fifth output");
        invalid_left_expected.left_expected_input =
            invalid_left_expected.right_expected_input.clone();
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&invalid_left_expected),
            invalid_key()
        );

        let mut missing_right_expected =
            source_multiple_reserve_declaration_membership_output(&exact, module.clone(), &symbols)
                .expect("exact source should produce a sixth output");
        missing_right_expected.right_expected_input = None;
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&missing_right_expected),
            invalid_key()
        );

        let mut swapped_expected =
            source_multiple_reserve_declaration_membership_output(&exact, module.clone(), &symbols)
                .expect("exact source should produce a seventh output");
        std::mem::swap(
            &mut swapped_expected.left_expected_input,
            &mut swapped_expected.right_expected_input,
        );
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&swapped_expected),
            invalid_key()
        );

        let mut corrupted_expected =
            source_multiple_reserve_declaration_membership_output(&exact, module.clone(), &symbols)
                .expect("exact source should produce an eighth output");
        corrupted_expected
            .right_expected_input
            .as_mut()
            .expect("right expected input should exist")
            .head = TypeHeadInput::BuiltinObject;
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&corrupted_expected),
            invalid_key()
        );

        let near_misses = [
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![reserve_item(
                    vec!["x", "y"],
                    ReserveTypeShape::Builtin("set"),
                )],
                "MultipleReserveDeclarationMembershipPayloadBoundary",
                "x",
                "in",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                    reserve_item(vec!["x"], ReserveTypeShape::Builtin("set")),
                ],
                "MultipleReserveDeclarationMembershipPayloadBoundary",
                "x",
                "in",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                separate_reserves(),
                "OtherPayloadBoundary",
                "x",
                "in",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                separate_reserves(),
                "MultipleReserveDeclarationMembershipPayloadBoundary",
                "y",
                "in",
                "x",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                separate_reserves(),
                "MultipleReserveDeclarationMembershipPayloadBoundary",
                "x",
                "in",
                "x",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
                "MultipleReserveDeclarationMembershipPayloadBoundary",
                "x",
                "in",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::Builtin("set")),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("object")),
                ],
                "MultipleReserveDeclarationMembershipPayloadBoundary",
                "x",
                "in",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::Builtin("set")),
                    reserve_item(vec!["y"], ReserveTypeShape::AttributedSet),
                ],
                "MultipleReserveDeclarationMembershipPayloadBoundary",
                "x",
                "in",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::Builtin("set")),
                    reserve_item(vec!["y"], ReserveTypeShape::NonBuiltin("Thing")),
                ],
                "MultipleReserveDeclarationMembershipPayloadBoundary",
                "x",
                "in",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::Builtin("set")),
                    reserve_item(vec!["y"], ReserveTypeShape::QualifiedSymbolWithArgs("Mode")),
                ],
                "MultipleReserveDeclarationMembershipPayloadBoundary",
                "x",
                "in",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::Builtin("set")),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                    reserve_item(vec!["z"], ReserveTypeShape::Builtin("set")),
                ],
                "MultipleReserveDeclarationMembershipPayloadBoundary",
                "x",
                "in",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                separate_reserves(),
                "MultipleReserveDeclarationMembershipPayloadBoundary",
                "x",
                "=",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast_with_options(
                source_id,
                separate_reserves(),
                IdentifierBinaryTheoremSpec {
                    status: Some("registration"),
                    label: "MultipleReserveDeclarationMembershipPayloadBoundary",
                    left: "x",
                    operator: "in",
                    right: "y",
                    recovered_label: false,
                },
            ),
            reserve_then_identifier_binary_theorem_ast_with_options(
                source_id,
                separate_reserves(),
                IdentifierBinaryTheoremSpec {
                    status: None,
                    label: "MultipleReserveDeclarationMembershipPayloadBoundary",
                    left: "x",
                    operator: "in",
                    right: "y",
                    recovered_label: true,
                },
            ),
            reserve_then_two_identifier_binary_theorems_ast(
                source_id,
                "MultipleReserveDeclarationMembershipPayloadBoundary",
                "in",
            ),
            reserve_then_builtin_binary_theorem_ast(
                source_id,
                separate_reserves(),
                "MultipleReserveDeclarationMembershipPayloadBoundary",
                "1",
                "in",
                "1",
            ),
            modes_then_empty_definition_reserve_identifier_binary_theorem_ast(
                source_id,
                [],
                separate_reserves(),
                IdentifierBinaryTheoremSpec {
                    status: None,
                    label: "MultipleReserveDeclarationMembershipPayloadBoundary",
                    left: "x",
                    operator: "in",
                    right: "y",
                    recovered_label: false,
                },
            ),
        ];
        for near_miss in near_misses {
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
    }

    #[test]
    fn source_heterogeneous_reserve_membership_bridge_preserves_distinct_types() {
        let source_id = source_id(125);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("heterogeneous_reserve_membership"),
        );
        let symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        let exact = reserve_then_identifier_binary_theorem_ast(
            source_id,
            vec![
                reserve_item(vec!["x"], ReserveTypeShape::Builtin("object")),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
            ],
            "HeterogeneousReserveMembershipPayloadBoundary",
            "x",
            "in",
            "y",
        );

        assert_eq!(
            source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
            Vec::<String>::new()
        );
        let payload =
            extract_source_heterogeneous_reserve_membership(&exact, module.clone(), &symbols)
                .expect("exact heterogeneous reserve membership source should extract");
        assert_eq!(payload.reserve.bridge.bindings().len(), 2);
        assert_eq!(payload.reserve.bridge.bindings()[0].spelling, "x");
        assert_eq!(
            payload.reserve.bridge.bindings()[0].type_head,
            TypeHeadInput::BuiltinObject
        );
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
            source_heterogeneous_reserve_membership_output(&exact, module.clone(), &symbols)
                .expect("exact heterogeneous membership should reach the checker");
        assert_source_reserved_variable_formula_output(&output)
            .expect("heterogeneous membership invariants should hold");
        assert_eq!(output.left_binding, BindingId::new(0));
        assert_eq!(output.right_binding, BindingId::new(1));
        assert_eq!(output.left_result_input.head, TypeHeadInput::BuiltinObject);
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
        let formula = output
            .term_formula
            .formulas()
            .iter()
            .map(|(_, formula)| formula)
            .next()
            .expect("heterogeneous membership formula should be checked");
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
                "heterogeneous-reserve-membership-left-result".to_owned(),
                "heterogeneous-reserve-membership-right-expected".to_owned(),
                "heterogeneous-reserve-membership-right-result".to_owned(),
            ])
        );

        let mut invalid_output =
            source_heterogeneous_reserve_membership_output(&exact, module.clone(), &symbols)
                .expect("exact source should produce a second checker output");
        invalid_output
            .right_expected_input
            .as_mut()
            .expect("right expected input should exist")
            .head = TypeHeadInput::BuiltinObject;
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&invalid_output),
            vec![TYPE_ELABORATION_HETEROGENEOUS_RESERVE_MEMBERSHIP_INVALID_PAYLOAD_KEY.to_owned()]
        );

        let near_misses = [
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::Builtin("object")),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                ],
                "OtherPayloadBoundary",
                "x",
                "in",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::Builtin("set")),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                ],
                "HeterogeneousReserveMembershipPayloadBoundary",
                "x",
                "in",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::Builtin("object")),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("object")),
                ],
                "HeterogeneousReserveMembershipPayloadBoundary",
                "x",
                "in",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                    reserve_item(vec!["x"], ReserveTypeShape::Builtin("object")),
                ],
                "HeterogeneousReserveMembershipPayloadBoundary",
                "x",
                "in",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::Builtin("object")),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                ],
                "HeterogeneousReserveMembershipPayloadBoundary",
                "y",
                "in",
                "x",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::Builtin("object")),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                ],
                "HeterogeneousReserveMembershipPayloadBoundary",
                "x",
                "in",
                "x",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::Builtin("object")),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                ],
                "HeterogeneousReserveMembershipPayloadBoundary",
                "x",
                "=",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast(
                source_id,
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::Builtin("object")),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                    reserve_item(vec!["z"], ReserveTypeShape::Builtin("set")),
                ],
                "HeterogeneousReserveMembershipPayloadBoundary",
                "x",
                "in",
                "y",
            ),
            reserve_then_identifier_binary_theorem_ast_with_options(
                source_id,
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::Builtin("object")),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                ],
                IdentifierBinaryTheoremSpec {
                    status: Some("registration"),
                    label: "HeterogeneousReserveMembershipPayloadBoundary",
                    left: "x",
                    operator: "in",
                    right: "y",
                    recovered_label: false,
                },
            ),
            reserve_then_identifier_binary_theorem_ast_with_options(
                source_id,
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::Builtin("object")),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                ],
                IdentifierBinaryTheoremSpec {
                    status: None,
                    label: "HeterogeneousReserveMembershipPayloadBoundary",
                    left: "x",
                    operator: "in",
                    right: "y",
                    recovered_label: true,
                },
            ),
            reserve_then_builtin_binary_theorem_ast(
                source_id,
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::Builtin("object")),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                ],
                "HeterogeneousReserveMembershipPayloadBoundary",
                "1",
                "in",
                "1",
            ),
        ];
        for near_miss in near_misses {
            assert_eq!(
                source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
    }
