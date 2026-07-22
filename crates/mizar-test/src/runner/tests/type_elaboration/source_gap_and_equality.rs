    #[test]
    fn source_reserve_bridge_reports_gap_or_evidence_detail_for_unsupported_shapes() {
        let source_id = source_id(95);
        let module = ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge"));
        let symbols = source_symbol_env(module.clone());
        let non_builtin = reserve_ast(
            source_id,
            vec![reserve_item(vec!["x"], ReserveTypeShape::NonBuiltin("T"))],
        );
        assert_eq!(
            source_type_elaboration_detail_keys(&non_builtin, module.clone(), &symbols),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );
        let formula_statement_theorem =
            formula_statement_theorem_ast(source_id, exact_formula_statement_spec());
        let formula_statement_detail_keys =
            vec!["type_elaboration.checker.checker.formula.external.formula_payload".to_owned()];
        assert_eq!(
            source_type_elaboration_detail_keys(
                &formula_statement_theorem,
                module.clone(),
                &symbols
            ),
            formula_statement_detail_keys
        );
        let formula_statement_output =
            source_formula_statement_output(&formula_statement_theorem, module.clone(), &symbols)
                .expect("exact formula statement bridge should produce checker output");
        let formula_statement_payload =
            extract_source_formula_statement(&formula_statement_theorem)
                .expect("exact formula statement bridge should extract source payload");
        let expected_formula_statement_range = range(source_id, 33, 39);
        let expected_formula_statement_sites = surface_sites_for_kind_ranges(
            &formula_statement_theorem,
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
            &[expected_formula_statement_range],
        );
        assert_eq!(
            formula_statement_payload.formula_site,
            expected_formula_statement_sites[0]
        );
        assert_eq!(
            formula_statement_payload.formula_range,
            expected_formula_statement_range
        );
        assert_eq!(formula_statement_output.terms().len(), 0);
        assert_eq!(formula_statement_output.formulas().len(), 1);
        let (_, checked_formula_statement) = formula_statement_output
            .formulas()
            .iter()
            .next()
            .expect("formula statement payload should be checked");
        assert_eq!(
            checked_formula_statement.site,
            formula_statement_payload.formula_site
        );
        assert_eq!(checked_formula_statement.kind, FormulaKind::Thesis);
        assert_eq!(checked_formula_statement.status, FormulaStatus::Partial);
        assert_eq!(checked_formula_statement.context, BindingContextId::new(0));
        assert!(checked_formula_statement.terms.is_empty());
        assert!(checked_formula_statement.facts.is_empty());
        assert_eq!(
            checked_formula_statement.deferred,
            vec![FormulaDeferredReason::MissingFormulaPayload]
        );
        let diagnostic_ranges = formula_statement_output
            .diagnostics()
            .canonical_iter()
            .filter_map(|(_, diagnostic)| {
                (diagnostic.message_key == "checker.formula.external.formula_payload")
                    .then_some(diagnostic.source_range)
            })
            .collect::<Vec<_>>();
        assert_eq!(
            diagnostic_ranges,
            vec![expected_formula_statement_range],
            "missing formula payload diagnostic should be anchored to thesis"
        );
        let contradiction_theorem =
            formula_statement_theorem_ast(source_id, exact_contradiction_formula_spec());
        assert_eq!(
            source_type_elaboration_detail_keys(&contradiction_theorem, module.clone(), &symbols),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );
        assert!(
            source_contradiction_formula_output(
                &contradiction_theorem,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "synthetic contradiction without a real resolver theorem owner must fail closed"
        );
        let contradiction_payload = extract_source_contradiction_formula(&contradiction_theorem)
            .expect("exact standalone contradiction bridge should extract source payload");
        let expected_contradiction_range = range(source_id, 53, 66);
        let expected_contradiction_sites = surface_sites_for_kind_ranges(
            &contradiction_theorem,
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Contradiction),
            &[expected_contradiction_range],
        );
        assert_eq!(
            contradiction_payload.formula_site,
            expected_contradiction_sites[0]
        );
        assert_eq!(
            contradiction_payload.formula_range,
            expected_contradiction_range
        );
        let expected_builtin_binary_configs = [
            (
                "TermFormulaPayloadBoundary",
                "1",
                "=",
                "1",
                FormulaKind::Equality,
            ),
            (
                "BuiltinInequalityPayloadBoundary",
                "1",
                "<>",
                "2",
                FormulaKind::Inequality,
            ),
            (
                "BuiltinMembershipPayloadBoundary",
                "1",
                "in",
                "1",
                FormulaKind::Membership,
            ),
        ];
        assert_eq!(
            SOURCE_BUILTIN_BINARY_TERM_FORMULA_CONFIGS.len(),
            expected_builtin_binary_configs.len()
        );
        for (config, (label, left, operator, right, formula_kind)) in
            SOURCE_BUILTIN_BINARY_TERM_FORMULA_CONFIGS
                .iter()
                .zip(expected_builtin_binary_configs)
        {
            assert_eq!(
                (config.label, config.left, config.operator, config.right),
                (label, left, operator, right)
            );
            let theorem =
                builtin_binary_theorem_ast(source_id, label, left, operator, right);
            let payload = extract_source_builtin_binary_term_formula(&theorem)
                .expect("exact builtin binary theorem should extract source payload");
            let formula_start = 11 + label.len();
            let left_range = range(source_id, formula_start, formula_start + left.len());
            let right_start = formula_start + left.len() + operator.len() + 2;
            let right_range = range(source_id, right_start, right_start + right.len());
            let formula_range = range(source_id, formula_start, right_start + right.len());
            let expected_operand_sites = surface_sites_for_kind_ranges(
                &theorem,
                SurfaceNodeKind::NumeralTerm,
                &[left_range, right_range],
            );
            let expected_formula_sites = surface_sites_for_kind_ranges(
                &theorem,
                SurfaceNodeKind::BuiltinPredicateApplication,
                &[formula_range],
            );
            assert_eq!(payload.formula_site, expected_formula_sites[0]);
            assert_eq!(payload.formula_range, formula_range);
            assert_eq!(payload.formula_kind, formula_kind);
            assert_eq!(payload.left_site, expected_operand_sites[0]);
            assert_eq!(payload.left_range, left_range);
            assert_eq!(payload.right_site, expected_operand_sites[1]);
            assert_eq!(payload.right_range, right_range);
        }
        let equality_theorem =
            builtin_equality_theorem_ast(source_id, "TermFormulaPayloadBoundary", "1", "1");
        assert_eq!(
            source_type_elaboration_detail_keys(&equality_theorem, module.clone(), &symbols),
            vec![
                "type_elaboration.checker.checker.formula.term.partial".to_owned(),
                "type_elaboration.checker.checker.term.external.numeric_type_payload".to_owned(),
            ]
        );
        let inequality_theorem = builtin_binary_theorem_ast(
            source_id,
            "BuiltinInequalityPayloadBoundary",
            "1",
            "<>",
            "2",
        );
        assert_eq!(
            source_type_elaboration_detail_keys(&inequality_theorem, module.clone(), &symbols),
            vec![
                "type_elaboration.checker.checker.formula.term.partial".to_owned(),
                "type_elaboration.checker.checker.term.external.numeric_type_payload".to_owned(),
            ]
        );
        let membership_theorem = builtin_binary_theorem_ast(
            source_id,
            "BuiltinMembershipPayloadBoundary",
            "1",
            "in",
            "1",
        );
        assert_eq!(
            source_type_elaboration_detail_keys(&membership_theorem, module.clone(), &symbols),
            vec![
                "type_elaboration.checker.checker.formula.term.partial".to_owned(),
                "type_elaboration.checker.checker.term.external.numeric_type_payload".to_owned(),
            ]
        );
        let type_assertion_label = "BuiltinTypeAssertionPayloadBoundary";
        let type_assertion_subject = "1";
        let type_assertion_operator = "is";
        let type_assertion_asserted_type = "set";
        let type_assertion_theorem = builtin_type_assertion_theorem_ast(
            source_id,
            type_assertion_label,
            type_assertion_subject,
            ReserveTypeShape::Builtin(type_assertion_asserted_type),
        );
        assert_eq!(
            source_type_elaboration_detail_keys(&type_assertion_theorem, module.clone(), &symbols),
            vec![
                "type_elaboration.checker.checker.formula.term.partial".to_owned(),
                "type_elaboration.checker.checker.term.external.numeric_type_payload".to_owned(),
            ]
        );
        let type_assertion_output = source_builtin_type_assertion_formula_output(
            &type_assertion_theorem,
            module.clone(),
            &symbols,
        )
        .expect("exact builtin type assertion bridge should produce checker output");
        let type_assertion_payload = extract_source_builtin_type_assertion_formula(
            &type_assertion_theorem,
            &module,
            &symbols,
        )
        .expect("exact builtin type assertion bridge should extract source payload");
        let type_assertion_formula_start = 11 + type_assertion_label.len();
        let expected_type_assertion_subject_range = range(
            source_id,
            type_assertion_formula_start,
            type_assertion_formula_start + type_assertion_subject.len(),
        );
        let type_assertion_asserted_type_start = type_assertion_formula_start
            + type_assertion_subject.len()
            + type_assertion_operator.len()
            + 2;
        let expected_type_assertion_asserted_type_range = range(
            source_id,
            type_assertion_asserted_type_start,
            type_assertion_asserted_type_start + type_assertion_asserted_type.len(),
        );
        let expected_type_assertion_formula_range = range(
            source_id,
            type_assertion_formula_start,
            type_assertion_asserted_type_start + type_assertion_asserted_type.len(),
        );
        let expected_type_assertion_formula_sites = surface_sites_for_kind_ranges(
            &type_assertion_theorem,
            SurfaceNodeKind::IsAssertion,
            &[expected_type_assertion_formula_range],
        );
        let expected_type_assertion_subject_sites = surface_sites_for_kind_ranges(
            &type_assertion_theorem,
            SurfaceNodeKind::NumeralTerm,
            &[expected_type_assertion_subject_range],
        );
        let expected_type_assertion_asserted_type_sites = surface_sites_for_kind_ranges(
            &type_assertion_theorem,
            SurfaceNodeKind::TypeExpression,
            &[expected_type_assertion_asserted_type_range],
        );
        assert_eq!(
            type_assertion_payload.formula_site,
            expected_type_assertion_formula_sites[0]
        );
        assert_eq!(
            type_assertion_payload.formula_range,
            expected_type_assertion_formula_range
        );
        assert_eq!(
            type_assertion_payload.subject_site,
            expected_type_assertion_subject_sites[0]
        );
        assert_eq!(
            type_assertion_payload.subject_range,
            expected_type_assertion_subject_range
        );
        assert_eq!(
            type_assertion_payload.asserted_type_site,
            expected_type_assertion_asserted_type_sites[0]
        );
        assert_eq!(
            type_assertion_payload.asserted_type.range,
            expected_type_assertion_asserted_type_range
        );
        assert_eq!(
            type_assertion_payload.asserted_type.spelling,
            type_assertion_asserted_type
        );
        assert_eq!(
            type_assertion_payload.asserted_type.head,
            TypeHeadInput::BuiltinSet
        );
        assert!(type_assertion_payload.asserted_type.attributes.is_empty());
        assert_eq!(type_assertion_output.type_entries().len(), 2);
        let asserted_type_entries = type_assertion_output
            .type_entries()
            .iter()
            .filter(|(_, entry)| {
                entry.owner == expected_type_assertion_asserted_type_sites[0]
            })
            .collect::<Vec<_>>();
        let [(_, asserted_type_entry)] = asserted_type_entries.as_slice() else {
            panic!("asserted type should have exactly one checker type entry");
        };
        assert_eq!(
            asserted_type_entry.owner,
            expected_type_assertion_asserted_type_sites[0]
        );
        assert_eq!(type_assertion_output.terms().len(), 1);
        let (_, checked_subject) = type_assertion_output
            .terms()
            .iter()
            .next()
            .expect("subject term should be checked");
        assert_eq!(checked_subject.kind, TermKind::Numeral);
        assert_eq!(checked_subject.status, TermStatus::Partial);
        assert_eq!(
            checked_subject.site,
            expected_type_assertion_subject_sites[0]
        );
        assert_eq!(
            type_assertion_output
                .type_entries()
                .get(checked_subject.type_entry)
                .expect("subject term type entry should exist")
                .owner,
            expected_type_assertion_subject_sites[0]
        );
        assert_eq!(type_assertion_output.formulas().len(), 1);
        let (_, checked_formula) = type_assertion_output
            .formulas()
            .iter()
            .next()
            .expect("type assertion formula should be checked");
        assert_eq!(
            checked_formula.site,
            expected_type_assertion_formula_sites[0]
        );
        assert_eq!(checked_formula.kind, FormulaKind::TypeAssertion);
        assert_eq!(checked_formula.status, FormulaStatus::Partial);
        assert_eq!(checked_formula.terms, vec![checked_subject.site.clone()]);
        let asserted_type = checked_formula
            .asserted_type
            .and_then(|id| type_assertion_output.normalized_types().get(id))
            .expect("type assertion bridge must pass the asserted type to the checker");
        assert_eq!(asserted_type.head, TypeHeadRef::BuiltinSet);
        assert!(asserted_type.attributes.positive().is_empty());
        assert!(asserted_type.attributes.negative().is_empty());
        assert_eq!(asserted_type.source.spelling, "set");
        assert_eq!(
            asserted_type.source.range,
            expected_type_assertion_asserted_type_range
        );
        let imported_predicate_functor_symbols =
            imported_predicate_functor_symbol_env(symbols.module_id().clone());
        let imported_predicate_functor_import = "parser.type_fixtures";
        let imported_predicate_functor_spec = exact_imported_predicate_functor_theorem_spec();
        let imported_predicate_functor_theorem = imported_predicate_functor_theorem_ast(
            source_id,
            &[imported_predicate_functor_import],
            imported_predicate_functor_spec,
        );
        let imported_predicate_functor_detail_keys = vec![
            "type_elaboration.checker.checker.formula.external.predicate_signature_payload"
                .to_owned(),
            "type_elaboration.checker.checker.formula.term.partial".to_owned(),
            "type_elaboration.checker.checker.term.external.numeric_type_payload".to_owned(),
            "type_elaboration.checker.checker.term.external.signature_payload".to_owned(),
        ];
        assert_eq!(
            source_type_elaboration_detail_keys(
                &imported_predicate_functor_theorem,
                imported_predicate_functor_symbols.module_id().clone(),
                &imported_predicate_functor_symbols
            ),
            imported_predicate_functor_detail_keys
        );
        let set_enumeration_theorem = set_enumeration_equality_theorem_ast(
            source_id,
            "SetEnumerationPayloadBoundary",
            ["1", "2"],
            "=",
            ["1", "2"],
        );
        let set_enumeration_detail_keys = vec![
            "type_elaboration.checker.checker.formula.term.partial".to_owned(),
            "type_elaboration.checker.checker.term.external.numeric_type_payload".to_owned(),
            "type_elaboration.checker.checker.term.external.result_type_payload".to_owned(),
        ];
        assert_eq!(
            source_type_elaboration_detail_keys(&set_enumeration_theorem, module.clone(), &symbols),
            set_enumeration_detail_keys
        );
        let set_enumeration_output = source_set_enumeration_formula_output(
            &set_enumeration_theorem,
            module.clone(),
            &symbols,
        )
        .expect("exact set-enumeration bridge should produce checker output");
        let set_enumeration_payload =
            extract_source_set_enumeration_formula(&set_enumeration_theorem)
                .expect("exact set-enumeration bridge should extract source payload");
        let expected_item_ranges = vec![
            range(source_id, 42, 43),
            range(source_id, 46, 47),
            range(source_id, 54, 55),
            range(source_id, 58, 59),
        ];
        let expected_set_ranges = vec![range(source_id, 40, 49), range(source_id, 52, 61)];
        let expected_formula_range = range(source_id, 40, 61);
        let expected_item_sites = surface_sites_for_kind_ranges(
            &set_enumeration_theorem,
            SurfaceNodeKind::NumeralTerm,
            &expected_item_ranges,
        );
        let expected_set_sites = surface_sites_for_kind_ranges(
            &set_enumeration_theorem,
            SurfaceNodeKind::SetEnumeration,
            &expected_set_ranges,
        );
        let exact_set_nodes = surface_nodes_with_kind(
            &set_enumeration_theorem,
            SurfaceNodeKind::SetEnumeration,
        )
        .into_iter()
        .filter(|(_, node)| expected_set_ranges.contains(&node.range))
        .map(|(_, node)| node)
        .collect::<Vec<_>>();
        assert_eq!(exact_set_nodes.len(), 2);
        for set_node in exact_set_nodes {
            assert_eq!(
                surface_direct_token_texts(&set_enumeration_theorem, set_node),
                vec!["{", ",", "}"]
            );
        }
        let expected_formula_sites = surface_sites_for_kind_ranges(
            &set_enumeration_theorem,
            SurfaceNodeKind::BuiltinPredicateApplication,
            &[expected_formula_range],
        );
        assert_eq!(
            set_enumeration_payload
                .left_items
                .iter()
                .map(|(site, range)| (site.clone(), *range))
                .collect::<Vec<_>>(),
            expected_item_sites[..2]
                .iter()
                .cloned()
                .zip(expected_item_ranges[..2].iter().copied())
                .collect::<Vec<_>>()
        );
        assert_eq!(
            set_enumeration_payload
                .right_items
                .iter()
                .map(|(site, range)| (site.clone(), *range))
                .collect::<Vec<_>>(),
            expected_item_sites[2..]
                .iter()
                .cloned()
                .zip(expected_item_ranges[2..].iter().copied())
                .collect::<Vec<_>>()
        );
        assert_eq!(
            set_enumeration_payload
                .left_items
                .iter()
                .chain(set_enumeration_payload.right_items.iter())
                .map(|(site, _)| site.clone())
                .collect::<Vec<_>>(),
            expected_item_sites
        );
        assert_eq!(
            set_enumeration_payload
                .left_items
                .iter()
                .chain(set_enumeration_payload.right_items.iter())
                .map(|(_, range)| *range)
                .collect::<Vec<_>>(),
            expected_item_ranges
        );
        assert_eq!(
            vec![
                set_enumeration_payload.left_site.clone(),
                set_enumeration_payload.right_site.clone(),
            ],
            expected_set_sites
        );
        assert_eq!(
            vec![
                set_enumeration_payload.left_range,
                set_enumeration_payload.right_range,
            ],
            expected_set_ranges
        );
        assert_eq!(
            set_enumeration_payload.formula_site,
            expected_formula_sites[0]
        );
        assert_eq!(
            set_enumeration_payload.formula_range,
            expected_formula_range
        );
        assert_eq!(set_enumeration_output.terms().len(), 6);
        assert_eq!(
            set_enumeration_output
                .terms()
                .iter()
                .map(|(_, term)| term.site.clone())
                .collect::<Vec<_>>(),
            vec![
                expected_set_sites[0].clone(),
                expected_item_sites[2].clone(),
                expected_item_sites[3].clone(),
                expected_set_sites[1].clone(),
                expected_item_sites[0].clone(),
                expected_item_sites[1].clone(),
            ]
        );
        assert_eq!(
            set_enumeration_output
                .terms()
                .iter()
                .map(|(_, term)| term.kind)
                .collect::<Vec<_>>(),
            vec![
                TermKind::SetEnumeration,
                TermKind::Numeral,
                TermKind::Numeral,
                TermKind::SetEnumeration,
                TermKind::Numeral,
                TermKind::Numeral,
            ]
        );
        for (site, _) in set_enumeration_payload
            .left_items
            .iter()
            .chain(set_enumeration_payload.right_items.iter())
        {
            let checked_numeral = set_enumeration_output
                .terms()
                .iter()
                .map(|(_, term)| term)
                .find(|term| term.site == *site)
                .expect("set-enumeration item numeral should be checked");
            assert_eq!(checked_numeral.kind, TermKind::Numeral);
            assert_eq!(checked_numeral.status, TermStatus::Partial);
            assert_eq!(
                set_enumeration_output
                    .type_entries()
                    .get(checked_numeral.type_entry)
                    .expect("numeral term type entry should exist")
                    .status,
                TypeStatus::Unknown
            );
            assert!(checked_numeral.candidate_set.is_none());
        }
        for site in [
            &set_enumeration_payload.left_site,
            &set_enumeration_payload.right_site,
        ] {
            let checked_set = set_enumeration_output
                .terms()
                .iter()
                .map(|(_, term)| term)
                .find(|term| term.site == *site)
                .expect("set-enumeration term should be checked");
            assert_eq!(checked_set.kind, TermKind::SetEnumeration);
            assert_eq!(checked_set.status, TermStatus::Partial);
            assert_eq!(
                set_enumeration_output
                    .type_entries()
                    .get(checked_set.type_entry)
                    .expect("set-enumeration term type entry should exist")
                    .status,
                TypeStatus::Unknown
            );
            assert!(checked_set.candidate_set.is_none());
        }
        assert_eq!(set_enumeration_output.formulas().len(), 1);
        let (_, checked_set_formula) = set_enumeration_output
            .formulas()
            .iter()
            .next()
            .expect("set-enumeration equality formula should be checked");
        assert_eq!(
            checked_set_formula.site,
            expected_formula_sites[0]
        );
        assert_eq!(checked_set_formula.kind, FormulaKind::Equality);
        assert_eq!(checked_set_formula.status, FormulaStatus::Partial);
        assert_eq!(
            checked_set_formula.terms,
            vec![
                expected_set_sites[0].clone(),
                expected_set_sites[1].clone(),
            ]
        );
        assert!(checked_set_formula.candidate_set.is_none());
        assert!(checked_set_formula.facts.is_empty());
        let formula_connective_quantifier_theorem =
            formula_connective_quantifier_theorem_ast(source_id, exact_formula_shell_spec());
        let formula_shell_detail_keys = vec![
            "type_elaboration.checker.checker.formula.external.formula_payload".to_owned(),
            "type_elaboration.checker.checker.formula.external.quantifier_payload".to_owned(),
        ];
        assert_eq!(
            source_type_elaboration_detail_keys(
                &formula_connective_quantifier_theorem,
                module.clone(),
                &symbols
            ),
            formula_shell_detail_keys
        );
        let formula_shell_output = source_formula_connective_quantifier_output(
            &formula_connective_quantifier_theorem,
            module.clone(),
            &symbols,
        )
        .expect("exact formula connective/quantifier bridge should produce checker output");
        let formula_shell_payload = extract_source_formula_connective_quantifier(
            &formula_connective_quantifier_theorem,
            &module,
            &symbols,
        )
        .expect("exact formula connective/quantifier bridge should extract source payload");
        let expected_implication_range = range(source_id, 53, 114);
        let expected_premise_constant_range = range(source_id, 53, 66);
        let expected_quantified_range = range(source_id, 75, 114);
        let expected_binder_segment_range = range(source_id, 79, 90);
        let expected_binder_type_range = range(source_id, 87, 90);
        let expected_negation_range = range(source_id, 97, 114);
        let expected_body_constant_range = range(source_id, 101, 114);
        let expected_contradiction_sites = surface_sites_for_kind_ranges(
            &formula_connective_quantifier_theorem,
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Contradiction),
            &[
                expected_premise_constant_range,
                expected_body_constant_range,
            ],
        );
        let expected_implication_sites = surface_sites_for_kind_ranges(
            &formula_connective_quantifier_theorem,
            SurfaceNodeKind::BinaryFormula(SurfaceFormulaBinaryOperator {
                connective: SurfaceFormulaConnective::Implies,
                repeated: false,
            }),
            &[expected_implication_range],
        );
        let expected_quantified_sites = surface_sites_for_kind_ranges(
            &formula_connective_quantifier_theorem,
            SurfaceNodeKind::QuantifiedFormula(SurfaceQuantifierKind::Universal),
            &[expected_quantified_range],
        );
        let expected_negation_sites = surface_sites_for_kind_ranges(
            &formula_connective_quantifier_theorem,
            SurfaceNodeKind::PrefixFormula(SurfaceFormulaPrefixOperator::Not),
            &[expected_negation_range],
        );
        let expected_binder_segment_sites = surface_sites_for_kind_ranges(
            &formula_connective_quantifier_theorem,
            SurfaceNodeKind::QuantifierVariableSegment,
            &[expected_binder_segment_range],
        );
        let expected_binder_type_sites = surface_sites_for_kind_ranges(
            &formula_connective_quantifier_theorem,
            SurfaceNodeKind::TypeExpression,
            &[expected_binder_type_range],
        );
        let expected_binder_head_sites = surface_sites_for_kind_ranges(
            &formula_connective_quantifier_theorem,
            SurfaceNodeKind::TypeHead,
            &[expected_binder_type_range],
        );
        let binder_segments = surface_nodes_with_kind(
            &formula_connective_quantifier_theorem,
            SurfaceNodeKind::QuantifierVariableSegment,
        );
        let [(_, binder_segment)] = binder_segments.as_slice() else {
            panic!("exact formula shell should have one binder segment");
        };
        assert_eq!(
            surface_direct_token_texts(&formula_connective_quantifier_theorem, binder_segment),
            vec!["x", "being"]
        );
        let binder_type_heads = surface_nodes_with_kind(
            &formula_connective_quantifier_theorem,
            SurfaceNodeKind::TypeHead,
        );
        let [(_, binder_type_head)] = binder_type_heads.as_slice() else {
            panic!("exact formula shell should have one binder type head");
        };
        assert_eq!(
            surface_direct_token_texts(&formula_connective_quantifier_theorem, binder_type_head),
            vec!["set"]
        );
        assert_eq!(expected_binder_segment_sites.len(), 1);
        assert_eq!(expected_binder_type_sites.len(), 1);
        assert_eq!(expected_binder_head_sites.len(), 1);
        assert_eq!(
            formula_shell_payload.premise_constant_site,
            expected_contradiction_sites[0]
        );
        assert_eq!(
            formula_shell_payload.premise_constant_range,
            expected_premise_constant_range
        );
        assert_eq!(
            formula_shell_payload.implication_site,
            expected_implication_sites[0]
        );
        assert_eq!(
            formula_shell_payload.implication_range,
            expected_implication_range
        );
        assert_eq!(
            formula_shell_payload.quantified_site,
            expected_quantified_sites[0]
        );
        assert_eq!(
            formula_shell_payload.quantified_range,
            expected_quantified_range
        );
        assert_eq!(
            formula_shell_payload.negation_site,
            expected_negation_sites[0]
        );
        assert_eq!(
            formula_shell_payload.negation_range,
            expected_negation_range
        );
        assert_eq!(
            formula_shell_payload.body_constant_site,
            expected_contradiction_sites[1]
        );
        assert_eq!(
            formula_shell_payload.body_constant_range,
            expected_body_constant_range
        );
        assert_eq!(formula_shell_output.terms().len(), 0);
        assert_eq!(formula_shell_output.formulas().len(), 5);
        let ordered_formula_shells = formula_shell_output
            .formulas()
            .iter()
            .map(|(_, formula)| formula)
            .collect::<Vec<_>>();
        assert_eq!(
            ordered_formula_shells
                .iter()
                .map(|formula| formula.site.clone())
                .collect::<Vec<_>>(),
            vec![
                expected_contradiction_sites[1].clone(),
                expected_negation_sites[0].clone(),
                expected_quantified_sites[0].clone(),
                expected_implication_sites[0].clone(),
                expected_contradiction_sites[0].clone(),
            ]
        );
        assert_eq!(
            ordered_formula_shells
                .iter()
                .map(|formula| formula.kind)
                .collect::<Vec<_>>(),
            vec![
                FormulaKind::Contradiction,
                FormulaKind::Negation,
                FormulaKind::Quantified,
                FormulaKind::Implication,
                FormulaKind::Contradiction,
            ]
        );
        for (formula, deferred) in ordered_formula_shells.iter().zip([
            FormulaDeferredReason::MissingFormulaPayload,
            FormulaDeferredReason::MissingFormulaPayload,
            FormulaDeferredReason::MissingQuantifierPayload,
            FormulaDeferredReason::MissingFormulaPayload,
            FormulaDeferredReason::MissingFormulaPayload,
        ]) {
            assert_eq!(formula.context, BindingContextId::new(0));
            assert_eq!(formula.status, FormulaStatus::Partial);
            assert!(formula.terms.is_empty());
            assert!(formula.facts.is_empty());
            assert_eq!(formula.deferred, vec![deferred]);
        }
        let mut formula_shell_diagnostics = formula_shell_output
            .diagnostics()
            .canonical_iter()
            .map(|(_, diagnostic)| {
                (diagnostic.message_key.clone(), diagnostic.source_range)
            })
            .collect::<Vec<_>>();
        formula_shell_diagnostics.sort_by(|left, right| {
            left.0
                .cmp(&right.0)
                .then_with(|| left.1.start.cmp(&right.1.start))
                .then_with(|| left.1.end.cmp(&right.1.end))
        });
        let mut expected_formula_shell_diagnostics = vec![
            (
                "checker.formula.external.formula_payload".to_owned(),
                expected_premise_constant_range,
            ),
            (
                "checker.formula.external.formula_payload".to_owned(),
                expected_implication_range,
            ),
            (
                "checker.formula.external.formula_payload".to_owned(),
                expected_negation_range,
            ),
            (
                "checker.formula.external.formula_payload".to_owned(),
                expected_body_constant_range,
            ),
            (
                "checker.formula.external.quantifier_payload".to_owned(),
                expected_quantified_range,
            ),
        ];
        expected_formula_shell_diagnostics.sort_by(|left, right| {
            left.0
                .cmp(&right.0)
                .then_with(|| left.1.start.cmp(&right.1.start))
                .then_with(|| left.1.end.cmp(&right.1.end))
        });
        assert_eq!(
            formula_shell_diagnostics,
            expected_formula_shell_diagnostics
        );
        for (site, range) in [
            (
                &formula_shell_payload.premise_constant_site,
                expected_premise_constant_range,
            ),
            (
                &formula_shell_payload.body_constant_site,
                expected_body_constant_range,
            ),
        ] {
            let checked_constant = formula_shell_output
                .formulas()
                .iter()
                .map(|(_, formula)| formula)
                .find(|formula| formula.site == *site)
                .expect("contradiction constant formula should be checked");
            assert_eq!(checked_constant.kind, FormulaKind::Contradiction);
            assert_eq!(checked_constant.context, BindingContextId::new(0));
            assert_eq!(checked_constant.status, FormulaStatus::Partial);
            assert!(checked_constant.terms.is_empty());
            assert!(checked_constant.facts.is_empty());
            assert_eq!(
                checked_constant.deferred,
                vec![FormulaDeferredReason::MissingFormulaPayload]
            );
            let diagnostic_ranges = formula_shell_output
                .diagnostics()
                .canonical_iter()
                .filter_map(|(_, diagnostic)| {
                    (diagnostic.message_key == "checker.formula.external.formula_payload"
                        && diagnostic.source_range == range)
                        .then_some(diagnostic.source_range)
                })
                .collect::<Vec<_>>();
            assert_eq!(
                diagnostic_ranges,
                vec![range],
                "missing formula payload diagnostic should be anchored to contradiction constant"
            );
        }
        let checked_implication = formula_shell_output
            .formulas()
            .iter()
            .map(|(_, formula)| formula)
            .find(|formula| formula.site == formula_shell_payload.implication_site)
            .expect("implication shell formula should be checked");
        assert_eq!(checked_implication.kind, FormulaKind::Implication);
        assert_eq!(checked_implication.context, BindingContextId::new(0));
        assert_eq!(checked_implication.status, FormulaStatus::Partial);
        assert!(checked_implication.terms.is_empty());
        assert!(checked_implication.facts.is_empty());
        assert_eq!(
            checked_implication.deferred,
            vec![FormulaDeferredReason::MissingFormulaPayload]
        );
        let checked_quantified = formula_shell_output
            .formulas()
            .iter()
            .map(|(_, formula)| formula)
            .find(|formula| formula.site == formula_shell_payload.quantified_site)
            .expect("quantified shell formula should be checked");
        assert_eq!(checked_quantified.kind, FormulaKind::Quantified);
        assert_eq!(checked_quantified.context, BindingContextId::new(0));
        assert_eq!(checked_quantified.status, FormulaStatus::Partial);
        assert!(checked_quantified.terms.is_empty());
        assert!(checked_quantified.facts.is_empty());
        assert_eq!(
            checked_quantified.deferred,
            vec![FormulaDeferredReason::MissingQuantifierPayload]
        );
        let checked_negation = formula_shell_output
            .formulas()
            .iter()
            .map(|(_, formula)| formula)
            .find(|formula| formula.site == formula_shell_payload.negation_site)
            .expect("negation shell formula should be checked");
        assert_eq!(checked_negation.kind, FormulaKind::Negation);
        assert_eq!(checked_negation.context, BindingContextId::new(0));
        assert_eq!(checked_negation.status, FormulaStatus::Partial);
        assert!(checked_negation.terms.is_empty());
        assert!(checked_negation.facts.is_empty());
        assert_eq!(
            checked_negation.deferred,
            vec![FormulaDeferredReason::MissingFormulaPayload]
        );
        let imported_predicate_functor_output = source_imported_predicate_functor_formula_output(
            &imported_predicate_functor_theorem,
            imported_predicate_functor_symbols.module_id().clone(),
            &imported_predicate_functor_symbols,
        )
        .expect("exact imported predicate/functor bridge should produce checker output");
        let imported_predicate_functor_payload = extract_source_imported_predicate_functor_formula(
            &imported_predicate_functor_theorem,
            imported_predicate_functor_symbols.module_id(),
            &imported_predicate_functor_symbols,
        )
        .expect("exact imported predicate/functor bridge should extract source payload");
        let imported_predicate_functor_formula_start = [
            "import",
            "parser",
            ".",
            "type_fixtures",
            ";",
            "theorem",
            imported_predicate_functor_spec.label,
            ":",
        ]
        .iter()
        .map(|token| token.len() + 1)
        .sum::<usize>();
        let imported_predicate_left_range = range(
            source_id,
            imported_predicate_functor_formula_start,
            imported_predicate_functor_formula_start
                + imported_predicate_functor_spec.left.len(),
        );
        let imported_predicate_start = imported_predicate_left_range.end + 1;
        let imported_parenthesized_start = imported_predicate_start
            + imported_predicate_functor_spec.predicate.len()
            + 1;
        let imported_functor_left_start = imported_parenthesized_start + 2;
        let imported_functor_left_range = range(
            source_id,
            imported_functor_left_start,
            imported_functor_left_start + imported_predicate_functor_spec.functor_left.len(),
        );
        let imported_functor_operator_start = imported_functor_left_range.end + 1;
        let imported_functor_right_start = imported_functor_operator_start
            + imported_predicate_functor_spec.functor.len()
            + 1;
        let imported_functor_right_range = range(
            source_id,
            imported_functor_right_start,
            imported_functor_right_start + imported_predicate_functor_spec.functor_right.len(),
        );
        let imported_functor_range = range(
            source_id,
            imported_functor_left_start,
            imported_functor_right_range.end,
        );
        let imported_predicate_formula_range = range(
            source_id,
            imported_predicate_functor_formula_start,
            imported_functor_right_range.end + 2,
        );
        let expected_imported_numeral_sites = surface_sites_for_kind_ranges(
            &imported_predicate_functor_theorem,
            SurfaceNodeKind::NumeralTerm,
            &[
                imported_predicate_left_range,
                imported_functor_left_range,
                imported_functor_right_range,
            ],
        );
        let expected_imported_functor_sites = surface_sites_for_kind_ranges(
            &imported_predicate_functor_theorem,
            SurfaceNodeKind::InfixExpression(mizar_syntax::SurfaceInfixOperator {
                spelling: imported_predicate_functor_spec.functor.into(),
                precedence: 10,
                associativity: mizar_syntax::SurfaceOperatorAssociativity::Left,
            }),
            &[imported_functor_range],
        );
        let expected_imported_formula_sites = surface_sites_for_kind_ranges(
            &imported_predicate_functor_theorem,
            SurfaceNodeKind::PredicateApplication,
            &[imported_predicate_formula_range],
        );
        assert_eq!(
            imported_predicate_functor_payload.formula_site,
            expected_imported_formula_sites[0]
        );
        assert_eq!(
            imported_predicate_functor_payload.formula_range,
            imported_predicate_formula_range
        );
        assert_eq!(
            imported_predicate_functor_payload.left_site,
            expected_imported_numeral_sites[0]
        );
        assert_eq!(
            imported_predicate_functor_payload.left_range,
            imported_predicate_left_range
        );
        assert_eq!(
            imported_predicate_functor_payload.functor_site,
            expected_imported_functor_sites[0]
        );
        assert_eq!(
            imported_predicate_functor_payload.functor_range,
            imported_functor_range
        );
        assert_eq!(
            imported_predicate_functor_payload.functor_left_site,
            expected_imported_numeral_sites[1]
        );
        assert_eq!(
            imported_predicate_functor_payload.functor_left_range,
            imported_functor_left_range
        );
        assert_eq!(
            imported_predicate_functor_payload.functor_right_site,
            expected_imported_numeral_sites[2]
        );
        assert_eq!(
            imported_predicate_functor_payload.functor_right_range,
            imported_functor_right_range
        );
        for (symbol, expected_kind, expected_spelling) in [
            (
                &imported_predicate_functor_payload.predicate_symbol,
                SymbolKind::Predicate,
                imported_predicate_functor_spec.predicate,
            ),
            (
                &imported_predicate_functor_payload.functor_symbol,
                SymbolKind::Functor,
                imported_predicate_functor_spec.functor,
            ),
        ] {
            assert_eq!(symbol.module().path().as_str(), imported_predicate_functor_import);
            let entry = imported_predicate_functor_symbols
                .symbols()
                .get(symbol)
                .expect("imported predicate/functor symbol should exist");
            assert_eq!(entry.kind(), expected_kind);
            assert_eq!(entry.primary_spelling(), expected_spelling);
            let contribution = imported_predicate_functor_symbols
                .contributions()
                .get(entry.contribution())
                .expect("imported predicate/functor contribution should exist");
            assert_eq!(contribution.module(), symbol.module());
            assert!(matches!(
                contribution.kind(),
                ContributionKind::ImportedSource { .. }
            ));
        }
        assert_eq!(imported_predicate_functor_output.terms().len(), 4);
        assert_eq!(
            imported_predicate_functor_output
                .terms()
                .iter()
                .map(|(_, term)| term.site.clone())
                .collect::<Vec<_>>(),
            vec![
                expected_imported_numeral_sites[0].clone(),
                expected_imported_numeral_sites[1].clone(),
                expected_imported_numeral_sites[2].clone(),
                expected_imported_functor_sites[0].clone(),
            ]
        );
        let checked_left = imported_predicate_functor_output
            .terms()
            .iter()
            .map(|(_, term)| term)
            .find(|term| term.site == expected_imported_numeral_sites[0])
            .expect("left numeral term should be checked");
        assert_eq!(checked_left.kind, TermKind::Numeral);
        assert_eq!(checked_left.status, TermStatus::Partial);
        let checked_functor_left = imported_predicate_functor_output
            .terms()
            .iter()
            .map(|(_, term)| term)
            .find(|term| term.site == expected_imported_numeral_sites[1])
            .expect("functor left numeral term should be checked");
        assert_eq!(checked_functor_left.kind, TermKind::Numeral);
        assert_eq!(checked_functor_left.status, TermStatus::Partial);
        let checked_functor_right = imported_predicate_functor_output
            .terms()
            .iter()
            .map(|(_, term)| term)
            .find(|term| term.site == expected_imported_numeral_sites[2])
            .expect("functor right numeral term should be checked");
        assert_eq!(checked_functor_right.kind, TermKind::Numeral);
        assert_eq!(checked_functor_right.status, TermStatus::Partial);
        let checked_functor = imported_predicate_functor_output
            .terms()
            .iter()
            .map(|(_, term)| term)
            .find(|term| term.site == expected_imported_functor_sites[0])
            .expect("infix functor application term should be checked");
        assert_eq!(checked_functor.kind, TermKind::FunctorApplication);
        assert_eq!(checked_functor.status, TermStatus::Partial);
        assert_eq!(
            checked_functor.reference,
            Some(TermReference::Symbol(
                imported_predicate_functor_payload.functor_symbol.clone()
            ))
        );
        assert!(checked_functor.candidate_set.is_none());
        assert_eq!(imported_predicate_functor_output.formulas().len(), 1);
        let (_, checked_predicate_formula) = imported_predicate_functor_output
            .formulas()
            .iter()
            .next()
            .expect("predicate application formula should be checked");
        assert_eq!(
            checked_predicate_formula.site,
            expected_imported_formula_sites[0]
        );
        assert_eq!(
            checked_predicate_formula.kind,
            FormulaKind::PredicateApplication
        );
        assert_eq!(checked_predicate_formula.status, FormulaStatus::Partial);
        assert_eq!(
            checked_predicate_formula.terms,
            vec![
                expected_imported_numeral_sites[0].clone(),
                expected_imported_functor_sites[0].clone(),
            ]
        );
        assert!(checked_predicate_formula.candidate_set.is_none());
        assert!(checked_predicate_formula.facts.is_empty());

        let imported_predicate_functor_gap_cases = [
            imported_predicate_functor_theorem_ast(
                source_id,
                &["parser.type_fixtures"],
                ImportedPredicateFunctorTheoremSpec {
                    label: "OtherPayloadBoundary",
                    ..exact_imported_predicate_functor_theorem_spec()
                },
            ),
            imported_predicate_functor_theorem_ast(
                source_id,
                &["parser.type_fixtures"],
                ImportedPredicateFunctorTheoremSpec {
                    predicate: "<=",
                    ..exact_imported_predicate_functor_theorem_spec()
                },
            ),
            imported_predicate_functor_theorem_ast(
                source_id,
                &["parser.type_fixtures"],
                ImportedPredicateFunctorTheoremSpec {
                    functor: "**",
                    ..exact_imported_predicate_functor_theorem_spec()
                },
            ),
            imported_predicate_functor_theorem_ast(
                source_id,
                &["parser.type_fixtures"],
                ImportedPredicateFunctorTheoremSpec {
                    left: "2",
                    ..exact_imported_predicate_functor_theorem_spec()
                },
            ),
            imported_predicate_functor_theorem_ast(
                source_id,
                &["parser.type_fixtures"],
                ImportedPredicateFunctorTheoremSpec {
                    functor_left: "2",
                    ..exact_imported_predicate_functor_theorem_spec()
                },
            ),
            imported_predicate_functor_theorem_ast(
                source_id,
                &["parser.type_fixtures"],
                ImportedPredicateFunctorTheoremSpec {
                    functor_right: "1",
                    ..exact_imported_predicate_functor_theorem_spec()
                },
            ),
            imported_predicate_functor_theorem_ast(
                source_id,
                &[],
                exact_imported_predicate_functor_theorem_spec(),
            ),
            imported_predicate_functor_theorem_ast(
                source_id,
                &["other.module"],
                exact_imported_predicate_functor_theorem_spec(),
            ),
            imported_predicate_functor_theorem_ast(
                source_id,
                &["parser.type_fixtures", "parser.type_fixtures"],
                exact_imported_predicate_functor_theorem_spec(),
            ),
            imported_predicate_functor_theorem_ast(
                source_id,
                &["parser.type_fixtures", "other.module"],
                exact_imported_predicate_functor_theorem_spec(),
            ),
            imported_predicate_functor_theorem_ast(
                source_id,
                &["parser.type_fixtures"],
                ImportedPredicateFunctorTheoremSpec {
                    status: Some("open"),
                    ..exact_imported_predicate_functor_theorem_spec()
                },
            ),
            reserve_then_imported_predicate_functor_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
            ),
        ];
        for gap_case in imported_predicate_functor_gap_cases {
            assert!(
                extract_source_imported_predicate_functor_formula(
                    &gap_case,
                    imported_predicate_functor_symbols.module_id(),
                    &imported_predicate_functor_symbols,
                )
                .is_none()
            );
            assert_eq!(
                source_type_elaboration_detail_keys(
                    &gap_case,
                    imported_predicate_functor_symbols.module_id().clone(),
                    &imported_predicate_functor_symbols
                ),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
        let imported_predicate_functor_corruption_cases = [
            imported_predicate_functor_theorem_ast_with_corruption(
                source_id,
                &[imported_predicate_functor_import],
                imported_predicate_functor_spec,
                ImportedPredicateFunctorTheoremCorruption {
                    recovered_label: true,
                    ..ImportedPredicateFunctorTheoremCorruption::default()
                },
            ),
            imported_predicate_functor_theorem_ast_with_corruption(
                source_id,
                &[imported_predicate_functor_import],
                imported_predicate_functor_spec,
                ImportedPredicateFunctorTheoremCorruption {
                    recovered_functor: true,
                    ..ImportedPredicateFunctorTheoremCorruption::default()
                },
            ),
            imported_predicate_functor_theorem_ast_with_corruption(
                source_id,
                &[imported_predicate_functor_import],
                imported_predicate_functor_spec,
                ImportedPredicateFunctorTheoremCorruption {
                    duplicate_theorem: true,
                    ..ImportedPredicateFunctorTheoremCorruption::default()
                },
            ),
            imported_predicate_functor_theorem_ast_with_corruption(
                source_id,
                &[imported_predicate_functor_import],
                imported_predicate_functor_spec,
                ImportedPredicateFunctorTheoremCorruption {
                    duplicate_formula_expression: true,
                    ..ImportedPredicateFunctorTheoremCorruption::default()
                },
            ),
            imported_predicate_functor_theorem_ast_with_corruption(
                source_id,
                &[imported_predicate_functor_import],
                imported_predicate_functor_spec,
                ImportedPredicateFunctorTheoremCorruption {
                    extra_formula_child: true,
                    ..ImportedPredicateFunctorTheoremCorruption::default()
                },
            ),
            imported_predicate_functor_theorem_ast_with_corruption(
                source_id,
                &[imported_predicate_functor_import],
                imported_predicate_functor_spec,
                ImportedPredicateFunctorTheoremCorruption {
                    extra_predicate_segment: true,
                    ..ImportedPredicateFunctorTheoremCorruption::default()
                },
            ),
            imported_predicate_functor_theorem_ast_with_corruption(
                source_id,
                &[imported_predicate_functor_import],
                imported_predicate_functor_spec,
                ImportedPredicateFunctorTheoremCorruption {
                    extra_segment_child: true,
                    ..ImportedPredicateFunctorTheoremCorruption::default()
                },
            ),
            imported_predicate_functor_theorem_ast_with_corruption(
                source_id,
                &[imported_predicate_functor_import],
                imported_predicate_functor_spec,
                ImportedPredicateFunctorTheoremCorruption {
                    extra_predicate_head_child: true,
                    ..ImportedPredicateFunctorTheoremCorruption::default()
                },
            ),
            imported_predicate_functor_theorem_ast_with_corruption(
                source_id,
                &[imported_predicate_functor_import],
                imported_predicate_functor_spec,
                ImportedPredicateFunctorTheoremCorruption {
                    extra_parenthesized_child: true,
                    ..ImportedPredicateFunctorTheoremCorruption::default()
                },
            ),
            imported_predicate_functor_theorem_ast_with_corruption(
                source_id,
                &[imported_predicate_functor_import],
                imported_predicate_functor_spec,
                ImportedPredicateFunctorTheoremCorruption {
                    extra_inner_expression_child: true,
                    ..ImportedPredicateFunctorTheoremCorruption::default()
                },
            ),
            imported_predicate_functor_theorem_ast_with_corruption(
                source_id,
                &[imported_predicate_functor_import],
                imported_predicate_functor_spec,
                ImportedPredicateFunctorTheoremCorruption {
                    extra_infix_operand: true,
                    ..ImportedPredicateFunctorTheoremCorruption::default()
                },
            ),
        ];
        for gap_case in imported_predicate_functor_corruption_cases {
            assert!(
                extract_source_imported_predicate_functor_formula(
                    &gap_case,
                    imported_predicate_functor_symbols.module_id(),
                    &imported_predicate_functor_symbols,
                )
                .is_none()
            );
            assert_eq!(
                source_type_elaboration_detail_keys(
                    &gap_case,
                    imported_predicate_functor_symbols.module_id().clone(),
                    &imported_predicate_functor_symbols,
                ),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
        for gap_symbols in [
            source_local_predicate_and_imported_functor_env(symbols.module_id().clone()),
            source_local_functor_and_imported_predicate_env(symbols.module_id().clone()),
            imported_predicate_wrong_functor_kind_env(symbols.module_id().clone()),
            imported_functor_wrong_predicate_kind_env(symbols.module_id().clone()),
            ambiguous_imported_predicate_functor_env(symbols.module_id().clone(), "divides"),
            ambiguous_imported_predicate_functor_env(symbols.module_id().clone(), "++"),
            imported_predicate_functor_local_contribution_env(symbols.module_id().clone()),
        ] {
            assert!(
                extract_source_imported_predicate_functor_formula(
                    &imported_predicate_functor_theorem,
                    gap_symbols.module_id(),
                    &gap_symbols,
                )
                .is_none()
            );
            assert_eq!(
                source_type_elaboration_detail_keys(
                    &imported_predicate_functor_theorem,
                    gap_symbols.module_id().clone(),
                    &gap_symbols
                ),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
        let imported_attribute_assertion_symbols =
            imported_empty_fixture_attribute_symbol_env(symbols.module_id().clone());
        let imported_attribute_assertion_import = "parser.type_fixtures";
        let imported_attribute_assertion_label = "ImportedAttributeAssertionPayloadBoundary";
        let imported_attribute_assertion_subject = "1";
        let imported_attribute_assertion_attribute = "empty";
        let imported_attribute_assertion_theorem = imported_attribute_assertion_theorem_ast(
            source_id,
            &[imported_attribute_assertion_import],
            imported_attribute_assertion_label,
            imported_attribute_assertion_subject,
            imported_attribute_assertion_attribute,
        );
        let imported_attribute_assertion_detail_keys = vec![
            "type_elaboration.checker.checker.formula.external.formula_payload".to_owned(),
            "type_elaboration.checker.checker.formula.term.partial".to_owned(),
            "type_elaboration.checker.checker.term.external.numeric_type_payload".to_owned(),
        ];
        assert_eq!(
            source_type_elaboration_detail_keys(
                &imported_attribute_assertion_theorem,
                imported_attribute_assertion_symbols.module_id().clone(),
                &imported_attribute_assertion_symbols
            ),
            imported_attribute_assertion_detail_keys
        );
        let imported_attribute_assertion_output =
            source_imported_attribute_assertion_formula_output(
                &imported_attribute_assertion_theorem,
                imported_attribute_assertion_symbols.module_id().clone(),
                &imported_attribute_assertion_symbols,
            )
            .expect("exact imported attribute assertion bridge should produce checker output");
        let imported_attribute_assertion_payload =
            extract_source_imported_attribute_assertion_formula(
                &imported_attribute_assertion_theorem,
                imported_attribute_assertion_symbols.module_id(),
                &imported_attribute_assertion_symbols,
            )
            .expect("exact imported attribute assertion bridge should extract source payload");
        let imported_attribute_assertion_formula_start = [
            "import",
            "parser",
            ".",
            "type_fixtures",
            ";",
            "theorem",
            imported_attribute_assertion_label,
            ":",
        ]
        .iter()
        .map(|token| token.len() + 1)
        .sum::<usize>();
        let imported_attribute_assertion_subject_range = range(
            source_id,
            imported_attribute_assertion_formula_start,
            imported_attribute_assertion_formula_start + imported_attribute_assertion_subject.len(),
        );
        let imported_attribute_assertion_attribute_start =
            imported_attribute_assertion_subject_range.end + 1 + "is".len() + 1;
        let imported_attribute_assertion_attribute_range = range(
            source_id,
            imported_attribute_assertion_attribute_start,
            imported_attribute_assertion_attribute_start
                + imported_attribute_assertion_attribute.len(),
        );
        let imported_attribute_assertion_formula_range = range(
            source_id,
            imported_attribute_assertion_formula_start,
            imported_attribute_assertion_attribute_range.end,
        );
        let expected_imported_attribute_subject_sites = surface_sites_for_kind_ranges(
            &imported_attribute_assertion_theorem,
            SurfaceNodeKind::NumeralTerm,
            &[imported_attribute_assertion_subject_range],
        );
        let expected_imported_attribute_formula_sites = surface_sites_for_kind_ranges(
            &imported_attribute_assertion_theorem,
            SurfaceNodeKind::IsAssertion,
            &[imported_attribute_assertion_formula_range],
        );
        let imported_attribute_refs = surface_nodes_with_kind(
            &imported_attribute_assertion_theorem,
            SurfaceNodeKind::AttributeRef,
        );
        let [(_, imported_attribute_ref)] = imported_attribute_refs.as_slice() else {
            panic!("exact imported attribute assertion should have one attribute ref");
        };
        assert!(surface_direct_token_texts(
            &imported_attribute_assertion_theorem,
            imported_attribute_ref
        )
        .is_empty());
        assert_eq!(
            imported_attribute_assertion_payload.formula_site,
            expected_imported_attribute_formula_sites[0]
        );
        assert_eq!(
            imported_attribute_assertion_payload.formula_range,
            imported_attribute_assertion_formula_range
        );
        assert_eq!(
            imported_attribute_assertion_payload.subject_site,
            expected_imported_attribute_subject_sites[0]
        );
        assert_eq!(
            imported_attribute_assertion_payload.subject_range,
            imported_attribute_assertion_subject_range
        );
        assert_eq!(
            imported_attribute_assertion_payload
                .attribute_symbol
                .module()
                .path()
                .as_str(),
            imported_attribute_assertion_import
        );
        let imported_attribute_entry = imported_attribute_assertion_symbols
            .symbols()
            .get(&imported_attribute_assertion_payload.attribute_symbol)
            .expect("imported attribute symbol should exist");
        assert_eq!(imported_attribute_entry.kind(), SymbolKind::Attribute);
        assert_eq!(
            imported_attribute_entry.primary_spelling(),
            imported_attribute_assertion_attribute
        );
        let imported_attribute_contribution = imported_attribute_assertion_symbols
            .contributions()
            .get(imported_attribute_entry.contribution())
            .expect("imported attribute contribution should exist");
        assert_eq!(
            imported_attribute_contribution.module(),
            imported_attribute_assertion_payload.attribute_symbol.module()
        );
        assert!(matches!(
            imported_attribute_contribution.kind(),
            ContributionKind::ImportedSource { .. }
        ));
        assert_eq!(imported_attribute_assertion_output.terms().len(), 1);
        assert_eq!(
            imported_attribute_assertion_output
                .terms()
                .iter()
                .map(|(_, term)| term.site.clone())
                .collect::<Vec<_>>(),
            vec![expected_imported_attribute_subject_sites[0].clone()]
        );
        let (_, checked_attribute_subject) = imported_attribute_assertion_output
            .terms()
            .iter()
            .next()
            .expect("attribute assertion subject term should be checked");
        assert_eq!(checked_attribute_subject.kind, TermKind::Numeral);
        assert_eq!(checked_attribute_subject.status, TermStatus::Partial);
        assert_eq!(
            checked_attribute_subject.site,
            expected_imported_attribute_subject_sites[0]
        );
        assert_eq!(checked_attribute_subject.context, BindingContextId::new(0));
        assert!(checked_attribute_subject.candidate_set.is_none());
        assert_eq!(imported_attribute_assertion_output.formulas().len(), 1);
        assert_eq!(
            imported_attribute_assertion_output
                .formulas()
                .iter()
                .map(|(_, formula)| formula.site.clone())
                .collect::<Vec<_>>(),
            vec![expected_imported_attribute_formula_sites[0].clone()]
        );
        let (_, checked_attribute_formula) = imported_attribute_assertion_output
            .formulas()
            .iter()
            .next()
            .expect("attribute assertion formula should be checked");
        assert_eq!(
            checked_attribute_formula.site,
            expected_imported_attribute_formula_sites[0]
        );
        assert_eq!(
            checked_attribute_formula.kind,
            FormulaKind::AttributeAssertion
        );
        assert_eq!(checked_attribute_formula.status, FormulaStatus::Partial);
        assert_eq!(checked_attribute_formula.context, BindingContextId::new(0));
        assert_eq!(
            checked_attribute_formula.terms,
            vec![expected_imported_attribute_subject_sites[0].clone()]
        );
        assert!(checked_attribute_formula.facts.is_empty());
        assert_eq!(
            checked_attribute_formula.deferred,
            vec![FormulaDeferredReason::MissingFormulaPayload]
        );
        let imported_attribute_assertion_gap_cases = [
            imported_attribute_assertion_theorem_ast(
                source_id,
                &["parser.type_fixtures"],
                "OtherPayloadBoundary",
                "1",
                "empty",
            ),
            imported_attribute_assertion_theorem_ast(
                source_id,
                &["parser.type_fixtures"],
                "ImportedAttributeAssertionPayloadBoundary",
                "2",
                "empty",
            ),
            imported_attribute_assertion_theorem_ast(
                source_id,
                &["parser.type_fixtures"],
                "ImportedAttributeAssertionPayloadBoundary",
                "1",
                "TypeCaseAttr",
            ),
            imported_attribute_assertion_theorem_ast(
                source_id,
                &[],
                "ImportedAttributeAssertionPayloadBoundary",
                "1",
                "empty",
            ),
            imported_attribute_assertion_theorem_ast(
                source_id,
                &["other.module"],
                "ImportedAttributeAssertionPayloadBoundary",
                "1",
                "empty",
            ),
            imported_attribute_assertion_theorem_ast(
                source_id,
                &["parser.type_fixtures", "parser.type_fixtures"],
                "ImportedAttributeAssertionPayloadBoundary",
                "1",
                "empty",
            ),
            reserve_then_imported_attribute_assertion_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
            ),
        ];
        for gap_case in imported_attribute_assertion_gap_cases {
            assert!(
                extract_source_imported_attribute_assertion_formula(
                    &gap_case,
                    imported_attribute_assertion_symbols.module_id(),
                    &imported_attribute_assertion_symbols,
                )
                .is_none()
            );
            assert_eq!(
                source_type_elaboration_detail_keys(
                    &gap_case,
                    imported_attribute_assertion_symbols.module_id().clone(),
                    &imported_attribute_assertion_symbols
                ),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
        let imported_attribute_assertion_corruption_cases = [
            ImportedAttributeAssertionCorruption {
                recovered_label: true,
                ..ImportedAttributeAssertionCorruption::default()
            },
            ImportedAttributeAssertionCorruption {
                recovered_attribute_symbol: true,
                ..ImportedAttributeAssertionCorruption::default()
            },
            ImportedAttributeAssertionCorruption {
                duplicate_theorem: true,
                ..ImportedAttributeAssertionCorruption::default()
            },
            ImportedAttributeAssertionCorruption {
                duplicate_formula_expression: true,
                ..ImportedAttributeAssertionCorruption::default()
            },
            ImportedAttributeAssertionCorruption {
                extra_formula_child: true,
                ..ImportedAttributeAssertionCorruption::default()
            },
            ImportedAttributeAssertionCorruption {
                extra_assertion_child: true,
                ..ImportedAttributeAssertionCorruption::default()
            },
            ImportedAttributeAssertionCorruption {
                extra_attribute_chain_child: true,
                ..ImportedAttributeAssertionCorruption::default()
            },
            ImportedAttributeAssertionCorruption {
                extra_attribute_ref_child: true,
                ..ImportedAttributeAssertionCorruption::default()
            },
            ImportedAttributeAssertionCorruption {
                extra_qualified_symbol_child: true,
                ..ImportedAttributeAssertionCorruption::default()
            },
            ImportedAttributeAssertionCorruption {
                extra_numeral_child: true,
                ..ImportedAttributeAssertionCorruption::default()
            },
            ImportedAttributeAssertionCorruption {
                extra_non: true,
                ..ImportedAttributeAssertionCorruption::default()
            },
        ]
        .map(|corruption| {
            imported_attribute_assertion_theorem_ast_with_corruption(
                source_id,
                &[imported_attribute_assertion_import],
                imported_attribute_assertion_label,
                imported_attribute_assertion_subject,
                imported_attribute_assertion_attribute,
                false,
                corruption,
            )
        });
        for gap_case in imported_attribute_assertion_corruption_cases {
            assert!(
                extract_source_imported_attribute_assertion_formula(
                    &gap_case,
                    imported_attribute_assertion_symbols.module_id(),
                    &imported_attribute_assertion_symbols,
                )
                .is_none()
            );
            assert_eq!(
                source_type_elaboration_detail_keys(
                    &gap_case,
                    imported_attribute_assertion_symbols.module_id().clone(),
                    &imported_attribute_assertion_symbols,
                ),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
        let imported_attribute_contribution_control = imported_attribute_contribution_env(
            imported_attribute_assertion_symbols.module_id().clone(),
            true,
        );
        assert!(
            extract_source_imported_attribute_assertion_formula(
                &imported_attribute_assertion_theorem,
                imported_attribute_contribution_control.module_id(),
                &imported_attribute_contribution_control,
            )
            .is_some()
        );
        for gap_symbols in [
            source_local_symbol_env(
                imported_attribute_assertion_symbols.module_id().clone(),
                "empty",
                SymbolKind::Attribute,
            ),
            local_and_imported_attribute_symbol_env(
                imported_attribute_assertion_symbols.module_id().clone(),
                "empty",
            ),
            imported_empty_fixture_wrong_kind_env(
                imported_attribute_assertion_symbols.module_id().clone(),
            ),
            ambiguous_imported_attribute_assertion_env(
                imported_attribute_assertion_symbols.module_id().clone(),
            ),
            imported_attribute_local_contribution_env(
                imported_attribute_assertion_symbols.module_id().clone(),
            ),
        ] {
            assert!(
                extract_source_imported_attribute_assertion_formula(
                    &imported_attribute_assertion_theorem,
                    gap_symbols.module_id(),
                    &gap_symbols,
                )
                .is_none()
            );
            assert_eq!(
                source_type_elaboration_detail_keys(
                    &imported_attribute_assertion_theorem,
                    gap_symbols.module_id().clone(),
                    &gap_symbols
                ),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
        let imported_non_empty_attribute_assertion_symbols =
            imported_empty_fixture_attribute_symbol_env(symbols.module_id().clone());
        let imported_non_empty_attribute_assertion_import = "parser.type_fixtures";
        let imported_non_empty_attribute_assertion_label =
            "ImportedNonEmptyAttributeAssertionPayloadBoundary";
        let imported_non_empty_attribute_assertion_subject = "1";
        let imported_non_empty_attribute_assertion_attribute = "empty";
        let imported_non_empty_attribute_assertion_theorem =
            imported_non_empty_attribute_assertion_theorem_ast(
                source_id,
                &[imported_non_empty_attribute_assertion_import],
                imported_non_empty_attribute_assertion_label,
                imported_non_empty_attribute_assertion_subject,
                imported_non_empty_attribute_assertion_attribute,
            );
        let imported_non_empty_attribute_assertion_detail_keys = vec![
            "type_elaboration.checker.checker.formula.external.formula_payload".to_owned(),
            "type_elaboration.checker.checker.formula.term.partial".to_owned(),
            "type_elaboration.checker.checker.term.external.numeric_type_payload".to_owned(),
        ];
        assert_eq!(
            source_type_elaboration_detail_keys(
                &imported_non_empty_attribute_assertion_theorem,
                imported_non_empty_attribute_assertion_symbols
                    .module_id()
                    .clone(),
                &imported_non_empty_attribute_assertion_symbols
            ),
            imported_non_empty_attribute_assertion_detail_keys
        );
        let imported_non_empty_attribute_assertion_output =
            source_imported_non_empty_attribute_assertion_formula_output(
                &imported_non_empty_attribute_assertion_theorem,
                imported_non_empty_attribute_assertion_symbols
                    .module_id()
                    .clone(),
                &imported_non_empty_attribute_assertion_symbols,
            )
            .expect(
                "exact imported non-empty attribute assertion bridge should produce checker output",
            );
        let imported_non_empty_attribute_assertion_payload =
            extract_source_imported_non_empty_attribute_assertion_formula(
                &imported_non_empty_attribute_assertion_theorem,
                imported_non_empty_attribute_assertion_symbols.module_id(),
                &imported_non_empty_attribute_assertion_symbols,
            )
            .expect("exact imported non-empty attribute assertion bridge should extract payload");
        let imported_non_empty_attribute_assertion_formula_start = [
            "import",
            "parser",
            ".",
            "type_fixtures",
            ";",
            "theorem",
            imported_non_empty_attribute_assertion_label,
            ":",
        ]
        .iter()
        .map(|token| token.len() + 1)
        .sum::<usize>();
        let imported_non_empty_attribute_assertion_subject_range = range(
            source_id,
            imported_non_empty_attribute_assertion_formula_start,
            imported_non_empty_attribute_assertion_formula_start
                + imported_non_empty_attribute_assertion_subject.len(),
        );
        let imported_non_empty_attribute_assertion_non_start =
            imported_non_empty_attribute_assertion_subject_range.end + 1 + "is".len() + 1;
        let imported_non_empty_attribute_assertion_attribute_start =
            imported_non_empty_attribute_assertion_non_start + "non".len() + 1;
        let imported_non_empty_attribute_assertion_attribute_range = range(
            source_id,
            imported_non_empty_attribute_assertion_attribute_start,
            imported_non_empty_attribute_assertion_attribute_start
                + imported_non_empty_attribute_assertion_attribute.len(),
        );
        let imported_non_empty_attribute_assertion_formula_range = range(
            source_id,
            imported_non_empty_attribute_assertion_formula_start,
            imported_non_empty_attribute_assertion_attribute_range.end,
        );
        let expected_imported_non_empty_attribute_subject_sites = surface_sites_for_kind_ranges(
            &imported_non_empty_attribute_assertion_theorem,
            SurfaceNodeKind::NumeralTerm,
            &[imported_non_empty_attribute_assertion_subject_range],
        );
        let expected_imported_non_empty_attribute_formula_sites = surface_sites_for_kind_ranges(
            &imported_non_empty_attribute_assertion_theorem,
            SurfaceNodeKind::IsAssertion,
            &[imported_non_empty_attribute_assertion_formula_range],
        );
        let imported_non_empty_attribute_refs = surface_nodes_with_kind(
            &imported_non_empty_attribute_assertion_theorem,
            SurfaceNodeKind::AttributeRef,
        );
        let [(_, imported_non_empty_attribute_ref)] =
            imported_non_empty_attribute_refs.as_slice()
        else {
            panic!("exact imported non-empty assertion should have one attribute ref");
        };
        assert_eq!(
            surface_direct_token_texts(
                &imported_non_empty_attribute_assertion_theorem,
                imported_non_empty_attribute_ref,
            ),
            vec!["non"]
        );
        assert_eq!(
            imported_non_empty_attribute_assertion_payload.formula_site,
            expected_imported_non_empty_attribute_formula_sites[0]
        );
        assert_eq!(
            imported_non_empty_attribute_assertion_payload.formula_range,
            imported_non_empty_attribute_assertion_formula_range
        );
        assert_eq!(
            imported_non_empty_attribute_assertion_payload.subject_site,
            expected_imported_non_empty_attribute_subject_sites[0]
        );
        assert_eq!(
            imported_non_empty_attribute_assertion_payload.subject_range,
            imported_non_empty_attribute_assertion_subject_range
        );
        assert_eq!(
            imported_non_empty_attribute_assertion_payload
                .attribute_symbol
                .module()
                .path()
                .as_str(),
            imported_non_empty_attribute_assertion_import
        );
        let imported_non_empty_attribute_entry = imported_non_empty_attribute_assertion_symbols
            .symbols()
            .get(&imported_non_empty_attribute_assertion_payload.attribute_symbol)
            .expect("imported non-empty attribute symbol should exist");
        assert_eq!(
            imported_non_empty_attribute_entry.kind(),
            SymbolKind::Attribute
        );
        assert_eq!(
            imported_non_empty_attribute_entry.primary_spelling(),
            imported_non_empty_attribute_assertion_attribute
        );
        let imported_non_empty_attribute_contribution =
            imported_non_empty_attribute_assertion_symbols
                .contributions()
                .get(imported_non_empty_attribute_entry.contribution())
                .expect("imported non-empty attribute contribution should exist");
        assert_eq!(
            imported_non_empty_attribute_contribution.module(),
            imported_non_empty_attribute_assertion_payload
                .attribute_symbol
                .module()
        );
        assert!(matches!(
            imported_non_empty_attribute_contribution.kind(),
            ContributionKind::ImportedSource { .. }
        ));
        assert_eq!(
            imported_non_empty_attribute_assertion_output.terms().len(),
            1
        );
        assert_eq!(
            imported_non_empty_attribute_assertion_output
                .terms()
                .iter()
                .map(|(_, term)| term.site.clone())
                .collect::<Vec<_>>(),
            vec![expected_imported_non_empty_attribute_subject_sites[0].clone()]
        );
        let (_, checked_non_empty_subject) = imported_non_empty_attribute_assertion_output
            .terms()
            .iter()
            .next()
            .expect("non-empty attribute assertion subject term should be checked");
        assert_eq!(checked_non_empty_subject.kind, TermKind::Numeral);
        assert_eq!(checked_non_empty_subject.status, TermStatus::Partial);
        assert_eq!(
            checked_non_empty_subject.site,
            expected_imported_non_empty_attribute_subject_sites[0]
        );
        assert_eq!(checked_non_empty_subject.context, BindingContextId::new(0));
        assert!(checked_non_empty_subject.candidate_set.is_none());
        assert_eq!(
            imported_non_empty_attribute_assertion_output
                .formulas()
                .len(),
            1
        );
        assert_eq!(
            imported_non_empty_attribute_assertion_output
                .formulas()
                .iter()
                .map(|(_, formula)| formula.site.clone())
                .collect::<Vec<_>>(),
            vec![expected_imported_non_empty_attribute_formula_sites[0].clone()]
        );
        let (_, checked_non_empty_formula) = imported_non_empty_attribute_assertion_output
            .formulas()
            .iter()
            .next()
            .expect("non-empty attribute assertion formula should be checked");
        assert_eq!(
            checked_non_empty_formula.site,
            expected_imported_non_empty_attribute_formula_sites[0]
        );
        assert_eq!(
            checked_non_empty_formula.kind,
            FormulaKind::AttributeAssertion
        );
        assert_eq!(checked_non_empty_formula.status, FormulaStatus::Partial);
        assert_eq!(checked_non_empty_formula.context, BindingContextId::new(0));
        assert_eq!(
            checked_non_empty_formula.terms,
            vec![expected_imported_non_empty_attribute_subject_sites[0].clone()]
        );
        assert!(checked_non_empty_formula.facts.is_empty());
        assert_eq!(
            checked_non_empty_formula.deferred,
            vec![FormulaDeferredReason::MissingFormulaPayload]
        );
        let imported_non_empty_attribute_assertion_gap_cases = [
            imported_non_empty_attribute_assertion_theorem_ast(
                source_id,
                &["parser.type_fixtures"],
                "OtherPayloadBoundary",
                "1",
                "empty",
            ),
            imported_non_empty_attribute_assertion_theorem_ast(
                source_id,
                &["parser.type_fixtures"],
                "ImportedNonEmptyAttributeAssertionPayloadBoundary",
                "2",
                "empty",
            ),
            imported_non_empty_attribute_assertion_theorem_ast(
                source_id,
                &["parser.type_fixtures"],
                "ImportedNonEmptyAttributeAssertionPayloadBoundary",
                "1",
                "TypeCaseAttr",
            ),
            imported_attribute_assertion_theorem_ast(
                source_id,
                &["parser.type_fixtures"],
                "ImportedNonEmptyAttributeAssertionPayloadBoundary",
                "1",
                "empty",
            ),
            imported_non_empty_attribute_assertion_theorem_ast(
                source_id,
                &[],
                "ImportedNonEmptyAttributeAssertionPayloadBoundary",
                "1",
                "empty",
            ),
            imported_non_empty_attribute_assertion_theorem_ast(
                source_id,
                &["other.module"],
                "ImportedNonEmptyAttributeAssertionPayloadBoundary",
                "1",
                "empty",
            ),
            imported_non_empty_attribute_assertion_theorem_ast(
                source_id,
                &["parser.type_fixtures", "parser.type_fixtures"],
                "ImportedNonEmptyAttributeAssertionPayloadBoundary",
                "1",
                "empty",
            ),
            reserve_then_imported_non_empty_attribute_assertion_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
            ),
        ];
        for gap_case in imported_non_empty_attribute_assertion_gap_cases {
            assert!(
                extract_source_imported_non_empty_attribute_assertion_formula(
                    &gap_case,
                    imported_non_empty_attribute_assertion_symbols.module_id(),
                    &imported_non_empty_attribute_assertion_symbols,
                )
                .is_none()
            );
            assert_eq!(
                source_type_elaboration_detail_keys(
                    &gap_case,
                    imported_non_empty_attribute_assertion_symbols
                        .module_id()
                        .clone(),
                    &imported_non_empty_attribute_assertion_symbols
                ),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
        let imported_non_empty_attribute_assertion_corruption_cases = [
            ImportedAttributeAssertionCorruption {
                recovered_label: true,
                ..ImportedAttributeAssertionCorruption::default()
            },
            ImportedAttributeAssertionCorruption {
                recovered_attribute_symbol: true,
                ..ImportedAttributeAssertionCorruption::default()
            },
            ImportedAttributeAssertionCorruption {
                duplicate_theorem: true,
                ..ImportedAttributeAssertionCorruption::default()
            },
            ImportedAttributeAssertionCorruption {
                duplicate_formula_expression: true,
                ..ImportedAttributeAssertionCorruption::default()
            },
            ImportedAttributeAssertionCorruption {
                extra_formula_child: true,
                ..ImportedAttributeAssertionCorruption::default()
            },
            ImportedAttributeAssertionCorruption {
                extra_assertion_child: true,
                ..ImportedAttributeAssertionCorruption::default()
            },
            ImportedAttributeAssertionCorruption {
                extra_attribute_chain_child: true,
                ..ImportedAttributeAssertionCorruption::default()
            },
            ImportedAttributeAssertionCorruption {
                extra_attribute_ref_child: true,
                ..ImportedAttributeAssertionCorruption::default()
            },
            ImportedAttributeAssertionCorruption {
                extra_qualified_symbol_child: true,
                ..ImportedAttributeAssertionCorruption::default()
            },
            ImportedAttributeAssertionCorruption {
                extra_numeral_child: true,
                ..ImportedAttributeAssertionCorruption::default()
            },
            ImportedAttributeAssertionCorruption {
                extra_non: true,
                ..ImportedAttributeAssertionCorruption::default()
            },
        ]
        .map(|corruption| {
            imported_attribute_assertion_theorem_ast_with_corruption(
                source_id,
                &[imported_non_empty_attribute_assertion_import],
                imported_non_empty_attribute_assertion_label,
                imported_non_empty_attribute_assertion_subject,
                imported_non_empty_attribute_assertion_attribute,
                true,
                corruption,
            )
        });
        for gap_case in imported_non_empty_attribute_assertion_corruption_cases {
            assert!(
                extract_source_imported_non_empty_attribute_assertion_formula(
                    &gap_case,
                    imported_non_empty_attribute_assertion_symbols.module_id(),
                    &imported_non_empty_attribute_assertion_symbols,
                )
                .is_none()
            );
            assert_eq!(
                source_type_elaboration_detail_keys(
                    &gap_case,
                    imported_non_empty_attribute_assertion_symbols
                        .module_id()
                        .clone(),
                    &imported_non_empty_attribute_assertion_symbols,
                ),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
        let imported_non_empty_attribute_contribution_control =
            imported_attribute_contribution_env(
                imported_non_empty_attribute_assertion_symbols
                    .module_id()
                    .clone(),
                true,
            );
        assert!(
            extract_source_imported_non_empty_attribute_assertion_formula(
                &imported_non_empty_attribute_assertion_theorem,
                imported_non_empty_attribute_contribution_control.module_id(),
                &imported_non_empty_attribute_contribution_control,
            )
            .is_some()
        );
        for gap_symbols in [
            source_local_symbol_env(
                imported_non_empty_attribute_assertion_symbols
                    .module_id()
                    .clone(),
                "empty",
                SymbolKind::Attribute,
            ),
            local_and_imported_attribute_symbol_env(
                imported_non_empty_attribute_assertion_symbols
                    .module_id()
                    .clone(),
                "empty",
            ),
            imported_empty_fixture_wrong_kind_env(
                imported_non_empty_attribute_assertion_symbols
                    .module_id()
                    .clone(),
            ),
            ambiguous_imported_attribute_assertion_env(
                imported_non_empty_attribute_assertion_symbols
                    .module_id()
                    .clone(),
            ),
            imported_attribute_local_contribution_env(
                imported_non_empty_attribute_assertion_symbols
                    .module_id()
                    .clone(),
            ),
        ] {
            assert!(
                extract_source_imported_non_empty_attribute_assertion_formula(
                    &imported_non_empty_attribute_assertion_theorem,
                    gap_symbols.module_id(),
                    &gap_symbols,
                )
                .is_none()
            );
            assert_eq!(
                source_type_elaboration_detail_keys(
                    &imported_non_empty_attribute_assertion_theorem,
                    gap_symbols.module_id().clone(),
                    &gap_symbols
                ),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
        let set_enumeration_gap_cases = [
            set_enumeration_equality_theorem_ast(
                source_id,
                "OtherPayloadBoundary",
                ["1", "2"],
                "=",
                ["1", "2"],
            ),
            set_enumeration_equality_theorem_ast(
                source_id,
                "SetEnumerationPayloadBoundary",
                ["1", "2"],
                "<>",
                ["1", "2"],
            ),
            set_enumeration_equality_theorem_ast(
                source_id,
                "SetEnumerationPayloadBoundary",
                ["2", "2"],
                "=",
                ["1", "2"],
            ),
            set_enumeration_equality_theorem_ast(
                source_id,
                "SetEnumerationPayloadBoundary",
                ["1", "1"],
                "=",
                ["1", "2"],
            ),
            set_enumeration_equality_theorem_ast(
                source_id,
                "SetEnumerationPayloadBoundary",
                ["1", "2"],
                "=",
                ["2", "2"],
            ),
            set_enumeration_equality_theorem_ast(
                source_id,
                "SetEnumerationPayloadBoundary",
                ["1", "2"],
                "=",
                ["1", "1"],
            ),
            set_enumeration_equality_theorem_ast_with_status(
                source_id,
                "open",
                "SetEnumerationPayloadBoundary",
                ["1", "2"],
                "=",
                ["1", "2"],
            ),
            reserve_then_set_enumeration_equality_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
            ),
            import_then_set_enumeration_equality_theorem_ast(source_id, "parser.type_fixtures"),
            double_set_enumeration_equality_theorem_ast(source_id),
            recovered_set_enumeration_equality_theorem_ast(source_id),
        ];
        for gap_case in set_enumeration_gap_cases {
            assert!(extract_source_set_enumeration_formula(&gap_case).is_none());
            assert_eq!(
                source_type_elaboration_detail_keys(&gap_case, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
        for corruption in [
            SetEnumerationTheoremCorruption::DuplicateFormulaExpression,
            SetEnumerationTheoremCorruption::FormulaExpressionKind,
            SetEnumerationTheoremCorruption::ExtraFormulaChild,
            SetEnumerationTheoremCorruption::FormulaKind,
            SetEnumerationTheoremCorruption::ExtraFormulaOperand,
            SetEnumerationTheoremCorruption::TermWrapperKind,
            SetEnumerationTheoremCorruption::ExtraTermWrapperChild,
            SetEnumerationTheoremCorruption::SetKind,
            SetEnumerationTheoremCorruption::SetPunctuation,
            SetEnumerationTheoremCorruption::ExtraSetItem,
            SetEnumerationTheoremCorruption::ExtraNumeralChild,
        ] {
            let gap_case =
                corrupted_set_enumeration_equality_theorem_ast(source_id, corruption);
            assert!(extract_source_set_enumeration_formula(&gap_case).is_none());
            assert_eq!(
                source_type_elaboration_detail_keys(&gap_case, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
        let formula_shell_gap_cases = [
            formula_connective_quantifier_theorem_ast(
                source_id,
                FormulaConnectiveQuantifierTheoremSpec {
                    label: "OtherPayloadBoundary",
                    ..exact_formula_shell_spec()
                },
            ),
            formula_connective_quantifier_theorem_ast(
                source_id,
                FormulaConnectiveQuantifierTheoremSpec {
                    connective: SurfaceFormulaConnective::Or,
                    ..exact_formula_shell_spec()
                },
            ),
            formula_connective_quantifier_theorem_ast(
                source_id,
                FormulaConnectiveQuantifierTheoremSpec {
                    quantifier: SurfaceQuantifierKind::Existential,
                    ..exact_formula_shell_spec()
                },
            ),
            formula_connective_quantifier_theorem_ast(
                source_id,
                FormulaConnectiveQuantifierTheoremSpec {
                    binder_type: ReserveTypeShape::Builtin("object"),
                    ..exact_formula_shell_spec()
                },
            ),
            formula_connective_quantifier_theorem_ast(
                source_id,
                FormulaConnectiveQuantifierTheoremSpec {
                    binder_type: ReserveTypeShape::AttributedSet,
                    ..exact_formula_shell_spec()
                },
            ),
            formula_connective_quantifier_theorem_ast(
                source_id,
                FormulaConnectiveQuantifierTheoremSpec {
                    negated: false,
                    ..exact_formula_shell_spec()
                },
            ),
            formula_connective_quantifier_theorem_ast(
                source_id,
                FormulaConnectiveQuantifierTheoremSpec {
                    status: Some("open"),
                    ..exact_formula_shell_spec()
                },
            ),
            formula_connective_quantifier_theorem_ast(
                source_id,
                FormulaConnectiveQuantifierTheoremSpec {
                    recovered_label: true,
                    ..exact_formula_shell_spec()
                },
            ),
            reserve_then_formula_connective_quantifier_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
            ),
            double_formula_connective_quantifier_theorem_ast(source_id),
            builtin_binary_theorem_ast(
                source_id,
                "FormulaConnectiveQuantifierPayloadBoundary",
                "1",
                "in",
                "1",
            ),
            builtin_type_assertion_theorem_ast(
                source_id,
                "FormulaConnectiveQuantifierPayloadBoundary",
                "1",
                ReserveTypeShape::Builtin("set"),
            ),
            attribute_assertion_theorem_ast(
                source_id,
                "FormulaConnectiveQuantifierPayloadBoundary",
                "1",
                "empty",
            ),
            set_enumeration_equality_theorem_ast(
                source_id,
                "FormulaConnectiveQuantifierPayloadBoundary",
                ["1", "2"],
                "=",
                ["1", "2"],
            ),
            imported_predicate_functor_theorem_ast(
                source_id,
                &["parser.type_fixtures"],
                ImportedPredicateFunctorTheoremSpec {
                    label: "FormulaConnectiveQuantifierPayloadBoundary",
                    ..exact_imported_predicate_functor_theorem_spec()
                },
            ),
            proof_block_formula_shell_label_ast(source_id),
        ];
        for gap_case in formula_shell_gap_cases {
            assert!(
                extract_source_formula_connective_quantifier(&gap_case, &module, &symbols)
                    .is_none()
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&gap_case, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
        for corruption in [
            FormulaConnectiveQuantifierCorruption::DuplicateFormulaExpression,
            FormulaConnectiveQuantifierCorruption::FormulaExpressionKind,
            FormulaConnectiveQuantifierCorruption::ExtraFormulaChild,
            FormulaConnectiveQuantifierCorruption::RepeatedImplication,
            FormulaConnectiveQuantifierCorruption::ImplicationToken,
            FormulaConnectiveQuantifierCorruption::ExtraImplicationOperand,
            FormulaConnectiveQuantifierCorruption::PremiseConstantKind,
            FormulaConnectiveQuantifierCorruption::PremiseConstantToken,
            FormulaConnectiveQuantifierCorruption::UniversalToken,
            FormulaConnectiveQuantifierCorruption::ExtraQuantifiedChild,
            FormulaConnectiveQuantifierCorruption::SegmentKind,
            FormulaConnectiveQuantifierCorruption::SegmentToken,
            FormulaConnectiveQuantifierCorruption::ExtraSegmentChild,
            FormulaConnectiveQuantifierCorruption::NegationToken,
            FormulaConnectiveQuantifierCorruption::ExtraNegationChild,
            FormulaConnectiveQuantifierCorruption::BodyConstantKind,
            FormulaConnectiveQuantifierCorruption::BodyConstantToken,
            FormulaConnectiveQuantifierCorruption::RecoveredInnerToken,
        ] {
            let gap_case =
                corrupted_formula_connective_quantifier_theorem_ast(source_id, corruption);
            assert!(
                extract_source_formula_connective_quantifier(&gap_case, &module, &symbols)
                    .is_none()
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&gap_case, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
        let non_connective_same_label = builtin_equality_theorem_ast(
            source_id,
            "FormulaConnectiveQuantifierPayloadBoundary",
            "1",
            "1",
        );
        assert!(
            extract_source_formula_connective_quantifier(
                &non_connective_same_label,
                &module,
                &symbols
            )
            .is_none()
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &non_connective_same_label,
                module.clone(),
                &symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );
        let non_set_enumeration_equality =
            builtin_equality_theorem_ast(source_id, "SetEnumerationPayloadBoundary", "1", "1");
        assert_eq!(
            source_type_elaboration_detail_keys(
                &non_set_enumeration_equality,
                module.clone(),
                &symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );
        let other_label_theorem =
            builtin_equality_theorem_ast(source_id, "OtherPayloadBoundary", "1", "1");
        assert_eq!(
            source_type_elaboration_detail_keys(&other_label_theorem, module.clone(), &symbols),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );
        let formula_statement_gap_cases = vec![
            formula_statement_theorem_ast(
                source_id,
                FormulaStatementTheoremSpec {
                    label: "OtherPayloadBoundary",
                    ..exact_formula_statement_spec()
                },
            ),
            formula_statement_theorem_ast(
                source_id,
                FormulaStatementTheoremSpec {
                    constant: SurfaceFormulaConstant::Contradiction,
                    ..exact_formula_statement_spec()
                },
            ),
            formula_statement_theorem_ast(
                source_id,
                FormulaStatementTheoremSpec {
                    status: Some("open"),
                    ..exact_formula_statement_spec()
                },
            ),
            formula_statement_theorem_ast(
                source_id,
                FormulaStatementTheoremSpec {
                    recovered_label: true,
                    ..exact_formula_statement_spec()
                },
            ),
            reserve_then_formula_statement_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
            ),
            double_formula_statement_theorem_ast(source_id),
            proof_block_formula_theorem_ast(source_id, "FormulaPayloadBoundary"),
        ];
        for gap_case in formula_statement_gap_cases {
            assert_eq!(
                source_type_elaboration_detail_keys(&gap_case, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
        let contradiction_formula_gap_cases = vec![
            formula_statement_theorem_ast(
                source_id,
                FormulaStatementTheoremSpec {
                    label: "OtherPayloadBoundary",
                    ..exact_contradiction_formula_spec()
                },
            ),
            formula_statement_theorem_ast(
                source_id,
                FormulaStatementTheoremSpec {
                    constant: SurfaceFormulaConstant::Thesis,
                    ..exact_contradiction_formula_spec()
                },
            ),
            formula_statement_theorem_ast(
                source_id,
                FormulaStatementTheoremSpec {
                    status: Some("open"),
                    ..exact_contradiction_formula_spec()
                },
            ),
            formula_statement_theorem_ast(
                source_id,
                FormulaStatementTheoremSpec {
                    recovered_label: true,
                    ..exact_contradiction_formula_spec()
                },
            ),
            reserve_then_exact_formula_constant_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
                exact_contradiction_formula_spec(),
            ),
            double_exact_formula_constant_theorem_ast(
                source_id,
                exact_contradiction_formula_spec(),
            ),
        ];
        for gap_case in contradiction_formula_gap_cases {
            assert_eq!(
                source_type_elaboration_detail_keys(&gap_case, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
        let other_literal_theorem =
            builtin_equality_theorem_ast(source_id, "TermFormulaPayloadBoundary", "1", "2");
        assert_eq!(
            source_type_elaboration_detail_keys(&other_literal_theorem, module.clone(), &symbols),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );
        let other_inequality_label_theorem =
            builtin_binary_theorem_ast(source_id, "OtherPayloadBoundary", "1", "<>", "2");
        assert_eq!(
            source_type_elaboration_detail_keys(
                &other_inequality_label_theorem,
                module.clone(),
                &symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );
        let other_inequality_literal_theorem = builtin_binary_theorem_ast(
            source_id,
            "BuiltinInequalityPayloadBoundary",
            "1",
            "<>",
            "1",
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &other_inequality_literal_theorem,
                module.clone(),
                &symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );
        let other_operator_theorem = builtin_binary_theorem_ast(
            source_id,
            "BuiltinInequalityPayloadBoundary",
            "1",
            "=",
            "2",
        );
        assert_eq!(
            source_type_elaboration_detail_keys(&other_operator_theorem, module.clone(), &symbols),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );
        let other_membership_label_theorem =
            builtin_binary_theorem_ast(source_id, "OtherPayloadBoundary", "1", "in", "1");
        assert_eq!(
            source_type_elaboration_detail_keys(
                &other_membership_label_theorem,
                module.clone(),
                &symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );
        let other_membership_literal_theorem = builtin_binary_theorem_ast(
            source_id,
            "BuiltinMembershipPayloadBoundary",
            "1",
            "in",
            "2",
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &other_membership_literal_theorem,
                module.clone(),
                &symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );
        let other_membership_operator_theorem = builtin_binary_theorem_ast(
            source_id,
            "BuiltinMembershipPayloadBoundary",
            "1",
            "=",
            "1",
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &other_membership_operator_theorem,
                module.clone(),
                &symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );
        let mixed_reserve_and_membership_theorem = reserve_then_builtin_binary_theorem_ast(
            source_id,
            vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
            "BuiltinMembershipPayloadBoundary",
            "1",
            "in",
            "1",
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &mixed_reserve_and_membership_theorem,
                module.clone(),
                &symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );
        for config in SOURCE_BUILTIN_BINARY_TERM_FORMULA_CONFIGS {
            let wrong_left_builtin_binary_theorem = builtin_binary_theorem_ast(
                source_id,
                config.label,
                "2",
                config.operator,
                config.right,
            );
            assert!(
                extract_source_builtin_binary_term_formula(&wrong_left_builtin_binary_theorem)
                    .is_none(),
                "wrong-left builtin binary theorem should not extract for {}",
                config.label
            );
            assert_eq!(
                source_type_elaboration_detail_keys(
                    &wrong_left_builtin_binary_theorem,
                    module.clone(),
                    &symbols
                ),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
            let status_prefixed_builtin_binary_theorem = builtin_binary_theorem_ast_with_status(
                source_id,
                "open",
                config.label,
                config.left,
                config.operator,
                config.right,
            );
            assert_eq!(
                source_type_elaboration_detail_keys(
                    &status_prefixed_builtin_binary_theorem,
                    module.clone(),
                    &symbols
                ),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()],
                "status-prefixed builtin binary theorem should not satisfy exact token guard for {}",
                config.label
            );
        }
        let builtin_binary_corruption_cases = [
            builtin_binary_theorem_ast_with_corruption(
                source_id,
                "TermFormulaPayloadBoundary",
                "1",
                "=",
                "1",
                BuiltinBinaryTheoremCorruption {
                    recovered_label: true,
                    ..BuiltinBinaryTheoremCorruption::default()
                },
            ),
            builtin_binary_theorem_ast_with_corruption(
                source_id,
                "TermFormulaPayloadBoundary",
                "1",
                "=",
                "1",
                BuiltinBinaryTheoremCorruption {
                    recovered_operator: true,
                    ..BuiltinBinaryTheoremCorruption::default()
                },
            ),
            double_builtin_binary_theorem_ast(
                source_id,
                "TermFormulaPayloadBoundary",
                "1",
                "=",
                "1",
            ),
            builtin_binary_theorem_ast_with_corruption(
                source_id,
                "TermFormulaPayloadBoundary",
                "1",
                "=",
                "1",
                BuiltinBinaryTheoremCorruption {
                    duplicate_formula_expression: true,
                    ..BuiltinBinaryTheoremCorruption::default()
                },
            ),
            builtin_binary_theorem_ast_with_corruption(
                source_id,
                "TermFormulaPayloadBoundary",
                "1",
                "=",
                "1",
                BuiltinBinaryTheoremCorruption {
                    extra_term_expression: true,
                    ..BuiltinBinaryTheoremCorruption::default()
                },
            ),
        ];
        for gap_case in builtin_binary_corruption_cases {
            assert!(extract_source_builtin_binary_term_formula(&gap_case).is_none());
            assert_eq!(
                source_type_elaboration_detail_keys(&gap_case, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
        let other_type_assertion_label_theorem = builtin_type_assertion_theorem_ast(
            source_id,
            "OtherPayloadBoundary",
            "1",
            ReserveTypeShape::Builtin("set"),
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &other_type_assertion_label_theorem,
                module.clone(),
                &symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );
        let status_prefixed_type_assertion_theorem = builtin_type_assertion_theorem_ast_with_status(
            source_id,
            "open",
            "BuiltinTypeAssertionPayloadBoundary",
            "1",
            ReserveTypeShape::Builtin("set"),
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &status_prefixed_type_assertion_theorem,
                module.clone(),
                &symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );
        let other_type_assertion_literal_theorem = builtin_type_assertion_theorem_ast(
            source_id,
            "BuiltinTypeAssertionPayloadBoundary",
            "2",
            ReserveTypeShape::Builtin("set"),
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &other_type_assertion_literal_theorem,
                module.clone(),
                &symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );
        let other_type_assertion_type_theorem = builtin_type_assertion_theorem_ast(
            source_id,
            "BuiltinTypeAssertionPayloadBoundary",
            "1",
            ReserveTypeShape::Builtin("object"),
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &other_type_assertion_type_theorem,
                module.clone(),
                &symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );
        let attributed_type_assertion_theorem = builtin_type_assertion_theorem_ast(
            source_id,
            "BuiltinTypeAssertionPayloadBoundary",
            "1",
            ReserveTypeShape::AttributedSet,
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &attributed_type_assertion_theorem,
                module.clone(),
                &symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );
        let builtin_type_assertion_corruption_cases = [
            builtin_type_assertion_theorem_ast_with_corruption(
                source_id,
                "BuiltinTypeAssertionPayloadBoundary",
                "1",
                ReserveTypeShape::Builtin("set"),
                BuiltinTypeAssertionTheoremCorruption {
                    recovered_label: true,
                    ..BuiltinTypeAssertionTheoremCorruption::default()
                },
            ),
            builtin_type_assertion_theorem_ast_with_corruption(
                source_id,
                "BuiltinTypeAssertionPayloadBoundary",
                "1",
                ReserveTypeShape::Builtin("set"),
                BuiltinTypeAssertionTheoremCorruption {
                    recovered_is: true,
                    ..BuiltinTypeAssertionTheoremCorruption::default()
                },
            ),
            builtin_type_assertion_theorem_ast_with_corruption(
                source_id,
                "BuiltinTypeAssertionPayloadBoundary",
                "1",
                ReserveTypeShape::Builtin("set"),
                BuiltinTypeAssertionTheoremCorruption {
                    duplicate_theorem: true,
                    ..BuiltinTypeAssertionTheoremCorruption::default()
                },
            ),
            builtin_type_assertion_theorem_ast_with_corruption(
                source_id,
                "BuiltinTypeAssertionPayloadBoundary",
                "1",
                ReserveTypeShape::Builtin("set"),
                BuiltinTypeAssertionTheoremCorruption {
                    duplicate_formula_expression: true,
                    ..BuiltinTypeAssertionTheoremCorruption::default()
                },
            ),
            builtin_type_assertion_theorem_ast_with_corruption(
                source_id,
                "BuiltinTypeAssertionPayloadBoundary",
                "1",
                ReserveTypeShape::Builtin("set"),
                BuiltinTypeAssertionTheoremCorruption {
                    extra_formula_child: true,
                    ..BuiltinTypeAssertionTheoremCorruption::default()
                },
            ),
            builtin_type_assertion_theorem_ast_with_corruption(
                source_id,
                "BuiltinTypeAssertionPayloadBoundary",
                "1",
                ReserveTypeShape::Builtin("set"),
                BuiltinTypeAssertionTheoremCorruption {
                    negated: true,
                    ..BuiltinTypeAssertionTheoremCorruption::default()
                },
            ),
            builtin_type_assertion_theorem_ast_with_corruption(
                source_id,
                "BuiltinTypeAssertionPayloadBoundary",
                "1",
                ReserveTypeShape::Builtin("set"),
                BuiltinTypeAssertionTheoremCorruption {
                    extra_assertion_operand: true,
                    ..BuiltinTypeAssertionTheoremCorruption::default()
                },
            ),
        ];
        for gap_case in builtin_type_assertion_corruption_cases {
            assert!(
                extract_source_builtin_type_assertion_formula(&gap_case, &module, &symbols)
                    .is_none()
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&gap_case, module.clone(), &symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
        let mixed_reserve_and_type_assertion_theorem =
            reserve_then_builtin_type_assertion_theorem_ast(
                source_id,
                vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
                "BuiltinTypeAssertionPayloadBoundary",
                "1",
                ReserveTypeShape::Builtin("set"),
            );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &mixed_reserve_and_type_assertion_theorem,
                module.clone(),
                &symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );
        let mixed_reserve_and_inequality_theorem = reserve_then_builtin_binary_theorem_ast(
            source_id,
            vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
            "BuiltinInequalityPayloadBoundary",
            "1",
            "<>",
            "2",
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &mixed_reserve_and_inequality_theorem,
                module.clone(),
                &symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );
        let mixed_reserve_and_theorem = reserve_then_builtin_equality_theorem_ast(
            source_id,
            vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
            "TermFormulaPayloadBoundary",
            "1",
            "1",
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &mixed_reserve_and_theorem,
                module.clone(),
                &symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );

        let mixed = reserve_ast(
            source_id,
            vec![
                reserve_item(vec!["x"], ReserveTypeShape::Builtin("set")),
                reserve_item(vec!["y"], ReserveTypeShape::AttributedSet),
            ],
        );
        assert_eq!(
            source_type_elaboration_detail_keys(&mixed, module, &symbols),
            vec!["type_elaboration.checker.checker.declaration.deferred.evidence_query".to_owned()]
        );

        let attributed = reserve_ast(
            source_id,
            vec![reserve_item(vec!["x"], ReserveTypeShape::AttributedSet)],
        );
        assert_eq!(
            source_type_elaboration_detail_keys(&attributed, symbols.module_id().clone(), &symbols),
            vec!["type_elaboration.checker.checker.declaration.deferred.evidence_query".to_owned()]
        );

        let imported_symbols = imported_attribute_symbol_env(symbols.module_id().clone());
        assert_eq!(
            source_type_elaboration_detail_keys(
                &attributed,
                imported_symbols.module_id().clone(),
                &imported_symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );

        let imported_fixture_attribute_symbols =
            imported_fixture_attribute_symbol_env(symbols.module_id().clone());
        let imported_fixture_attribute = imported_reserve_ast(
            source_id,
            &["parser.type_fixtures"],
            vec![reserve_item(
                vec!["a"],
                ReserveTypeShape::AttributedSetWithNamedAttribute("TypeCaseAttr"),
            )],
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &imported_fixture_attribute,
                imported_fixture_attribute_symbols.module_id().clone(),
                &imported_fixture_attribute_symbols
            ),
            vec!["type_elaboration.checker.checker.declaration.deferred.evidence_query".to_owned()]
        );
        let imported_fixture_negative_type_case_attribute = imported_reserve_ast(
            source_id,
            &["parser.type_fixtures"],
            vec![reserve_item(
                vec!["a"],
                ReserveTypeShape::AttributedSetWithNegativeNamedAttribute("TypeCaseAttr"),
            )],
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &imported_fixture_negative_type_case_attribute,
                imported_fixture_attribute_symbols.module_id().clone(),
                &imported_fixture_attribute_symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );
        let imported_fixture_attribute_object = imported_reserve_ast(
            source_id,
            &["parser.type_fixtures"],
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::AttributedObjectWithNamedAttribute("TypeCaseAttr"),
            )],
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &imported_fixture_attribute_object,
                imported_fixture_attribute_symbols.module_id().clone(),
                &imported_fixture_attribute_symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );
        let imported_fixture_attribute_local_structure_symbols =
            local_structure_and_imported_fixture_attribute_symbol_env(
                symbols.module_id().clone(),
                "Struct",
                "TypeCaseAttr",
            );
        let imported_fixture_attribute_local_structure = reserve_ast(
            source_id,
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::AttributedQualifiedSymbolWithNamedAttribute(
                    "TypeCaseAttr",
                    "Struct",
                ),
            )],
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &imported_fixture_attribute_local_structure,
                imported_fixture_attribute_local_structure_symbols
                    .module_id()
                    .clone(),
                &imported_fixture_attribute_local_structure_symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );

        let imported_fixture_empty_attribute_symbols = imported_parser_fixture_symbol_env(
            symbols.module_id().clone(),
            "empty",
            SymbolKind::Attribute,
        );
        let imported_fixture_negative_empty_attribute = imported_reserve_ast(
            source_id,
            &["parser.type_fixtures"],
            vec![reserve_item(vec!["x"], ReserveTypeShape::AttributedSet)],
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &imported_fixture_negative_empty_attribute,
                imported_fixture_empty_attribute_symbols.module_id().clone(),
                &imported_fixture_empty_attribute_symbols
            ),
            vec!["type_elaboration.checker.checker.declaration.deferred.evidence_query".to_owned()]
        );
        let imported_fixture_positive_empty_attribute = imported_reserve_ast(
            source_id,
            &["parser.type_fixtures"],
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::AttributedSetWithNamedAttribute("empty"),
            )],
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &imported_fixture_positive_empty_attribute,
                imported_fixture_empty_attribute_symbols.module_id().clone(),
                &imported_fixture_empty_attribute_symbols
            ),
            vec!["type_elaboration.checker.checker.declaration.deferred.evidence_query".to_owned()]
        );
        let imported_fixture_missing_import_empty_attribute = imported_reserve_ast(
            source_id,
            &[],
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::AttributedSetWithNamedAttribute("empty"),
            )],
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &imported_fixture_missing_import_empty_attribute,
                imported_fixture_empty_attribute_symbols.module_id().clone(),
                &imported_fixture_empty_attribute_symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );
        let imported_fixture_wrong_import_empty_attribute = imported_reserve_ast(
            source_id,
            &["other.module"],
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::AttributedSetWithNamedAttribute("empty"),
            )],
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &imported_fixture_wrong_import_empty_attribute,
                imported_fixture_empty_attribute_symbols.module_id().clone(),
                &imported_fixture_empty_attribute_symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );
        let imported_fixture_duplicate_import_empty_attribute = imported_reserve_ast(
            source_id,
            &["parser.type_fixtures", "parser.type_fixtures"],
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::AttributedSetWithNamedAttribute("empty"),
            )],
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &imported_fixture_duplicate_import_empty_attribute,
                imported_fixture_empty_attribute_symbols.module_id().clone(),
                &imported_fixture_empty_attribute_symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );
        let imported_fixture_extra_definition_empty_attribute =
            imported_reserve_ast_with_extra_definition(
                source_id,
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::AttributedSetWithNamedAttribute("empty"),
                )],
            );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &imported_fixture_extra_definition_empty_attribute,
                imported_fixture_empty_attribute_symbols.module_id().clone(),
                &imported_fixture_empty_attribute_symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );
        let imported_fixture_extra_recovery_empty_attribute =
            imported_reserve_ast_with_extra_recovery(
                source_id,
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::AttributedSetWithNamedAttribute("empty"),
                )],
            );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &imported_fixture_extra_recovery_empty_attribute,
                imported_fixture_empty_attribute_symbols.module_id().clone(),
                &imported_fixture_empty_attribute_symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );
        let imported_fixture_mixed_polarity_empty_attribute = imported_reserve_ast(
            source_id,
            &["parser.type_fixtures"],
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::AttributedSetWithMixedPolarityNamedAttribute("empty"),
            )],
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &imported_fixture_mixed_polarity_empty_attribute,
                imported_fixture_empty_attribute_symbols.module_id().clone(),
                &imported_fixture_empty_attribute_symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );
        let imported_fixture_positive_empty_object_attribute = imported_reserve_ast(
            source_id,
            &["parser.type_fixtures"],
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::AttributedObjectWithNamedAttribute("empty"),
            )],
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &imported_fixture_positive_empty_object_attribute,
                imported_fixture_empty_attribute_symbols.module_id().clone(),
                &imported_fixture_empty_attribute_symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );
        let imported_fixture_empty_object_attribute = imported_reserve_ast(
            source_id,
            &["parser.type_fixtures"],
            vec![reserve_item(vec!["x"], ReserveTypeShape::AttributedObject)],
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &imported_fixture_empty_object_attribute,
                imported_fixture_empty_attribute_symbols.module_id().clone(),
                &imported_fixture_empty_attribute_symbols
            ),
            vec!["type_elaboration.checker.checker.declaration.deferred.evidence_query".to_owned()]
        );
        let imported_fixture_duplicate_empty_object_attribute = imported_reserve_ast(
            source_id,
            &["parser.type_fixtures"],
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::AttributedObjectWithDuplicateNamedAttribute("empty"),
            )],
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &imported_fixture_duplicate_empty_object_attribute,
                imported_fixture_empty_attribute_symbols.module_id().clone(),
                &imported_fixture_empty_attribute_symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );
        let imported_fixture_duplicate_empty_set_attribute = imported_reserve_ast(
            source_id,
            &["parser.type_fixtures"],
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::AttributedSetWithDuplicateNamedAttribute("empty"),
            )],
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &imported_fixture_duplicate_empty_set_attribute,
                imported_fixture_empty_attribute_symbols.module_id().clone(),
                &imported_fixture_empty_attribute_symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );
        let imported_fixture_argumented_empty_set_attribute = imported_reserve_ast(
            source_id,
            &["parser.type_fixtures"],
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::AttributedSetWithAttributeArgs,
            )],
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &imported_fixture_argumented_empty_set_attribute,
                imported_fixture_empty_attribute_symbols.module_id().clone(),
                &imported_fixture_empty_attribute_symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );
        let imported_fixture_multiple_name_empty_set_attribute = imported_reserve_ast(
            source_id,
            &["parser.type_fixtures"],
            vec![reserve_item(
                vec!["x", "y"],
                ReserveTypeShape::AttributedSetWithNamedAttribute("empty"),
            )],
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &imported_fixture_multiple_name_empty_set_attribute,
                imported_fixture_empty_attribute_symbols.module_id().clone(),
                &imported_fixture_empty_attribute_symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );
        let imported_fixture_exact_mixed_empty_set = imported_reserve_ast(
            source_id,
            &["parser.type_fixtures"],
            vec![
                reserve_item(vec!["x"], ReserveTypeShape::Builtin("set")),
                reserve_item(vec!["y"], ReserveTypeShape::AttributedSet),
            ],
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &imported_fixture_exact_mixed_empty_set,
                imported_fixture_empty_attribute_symbols.module_id().clone(),
                &imported_fixture_empty_attribute_symbols
            ),
            vec!["type_elaboration.checker.checker.declaration.deferred.evidence_query".to_owned()]
        );
        let imported_fixture_reordered_mixed_empty_set = imported_reserve_ast(
            source_id,
            &["parser.type_fixtures"],
            vec![
                reserve_item(vec!["y"], ReserveTypeShape::AttributedSet),
                reserve_item(vec!["x"], ReserveTypeShape::Builtin("set")),
            ],
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &imported_fixture_reordered_mixed_empty_set,
                imported_fixture_empty_attribute_symbols.module_id().clone(),
                &imported_fixture_empty_attribute_symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );
        let imported_fixture_extra_mixed_empty_set = imported_reserve_ast(
            source_id,
            &["parser.type_fixtures"],
            vec![
                reserve_item(vec!["x"], ReserveTypeShape::Builtin("set")),
                reserve_item(vec!["y"], ReserveTypeShape::AttributedSet),
                reserve_item(vec!["z"], ReserveTypeShape::Builtin("set")),
            ],
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &imported_fixture_extra_mixed_empty_set,
                imported_fixture_empty_attribute_symbols.module_id().clone(),
                &imported_fixture_empty_attribute_symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );
        let imported_fixture_empty_local_structure_symbols =
            local_structure_and_imported_fixture_attribute_symbol_env(
                symbols.module_id().clone(),
                "Struct",
                "empty",
            );
        let imported_fixture_empty_local_structure = reserve_ast(
            source_id,
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::AttributedQualifiedSymbol("Struct"),
            )],
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &imported_fixture_empty_local_structure,
                imported_fixture_empty_local_structure_symbols
                    .module_id()
                    .clone(),
                &imported_fixture_empty_local_structure_symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );

        let shadowed_imported_attribute_symbols =
            local_and_imported_attribute_symbol_env(symbols.module_id().clone(), "TypeCaseAttr");
        let resolved_shadowed_attribute = resolve_visible_attribute(
            &shadowed_imported_attribute_symbols,
            shadowed_imported_attribute_symbols.module_id(),
            "TypeCaseAttr",
        )
        .expect("a local attribute should shadow an imported attribute of the same spelling");
        assert_eq!(
            resolved_shadowed_attribute.module(),
            shadowed_imported_attribute_symbols.module_id()
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &imported_fixture_attribute,
                shadowed_imported_attribute_symbols.module_id().clone(),
                &shadowed_imported_attribute_symbols
            ),
            vec!["type_elaboration.checker.checker.declaration.deferred.evidence_query".to_owned()]
        );

        let local_mode = reserve_ast(
            source_id,
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("Mode"),
            )],
        );
        let mode_symbols = source_mode_symbol_env(symbols.module_id().clone());
        assert_eq!(
            source_type_elaboration_detail_keys(
                &local_mode,
                mode_symbols.module_id().clone(),
                &mode_symbols
            ),
            vec![
                "type_elaboration.checker.checker.type.external.mode_expansion_payload".to_owned(),
                "type_elaboration.checker.checker.type.recovery".to_owned(),
            ]
        );

        let local_structure = reserve_ast(
            source_id,
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("Struct"),
            )],
        );
        let structure_symbols = source_structure_symbol_env(symbols.module_id().clone());
        assert_eq!(
            source_type_elaboration_detail_keys(
                &local_structure,
                structure_symbols.module_id().clone(),
                &structure_symbols
            ),
            vec!["type_elaboration.checker.checker.declaration.deferred.evidence_query".to_owned()]
        );

        let imported_mode_symbols = imported_mode_symbol_env(symbols.module_id().clone());
        assert_eq!(
            source_type_elaboration_detail_keys(
                &local_mode,
                imported_mode_symbols.module_id().clone(),
                &imported_mode_symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );

        let shadowed_imported_mode_symbols =
            local_and_imported_mode_symbol_env(symbols.module_id().clone(), "TypeCaseMode");
        let shadowed_imported_mode = reserve_ast(
            source_id,
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("TypeCaseMode"),
            )],
        );
        let resolved_shadowed_head = resolve_visible_type_head(
            &shadowed_imported_mode_symbols,
            shadowed_imported_mode_symbols.module_id(),
            "TypeCaseMode",
        )
        .expect("a local mode should shadow an imported mode of the same spelling");
        assert_eq!(
            resolved_shadowed_head.module(),
            shadowed_imported_mode_symbols.module_id()
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &shadowed_imported_mode,
                shadowed_imported_mode_symbols.module_id().clone(),
                &shadowed_imported_mode_symbols
            ),
            vec![
                "type_elaboration.checker.checker.type.external.mode_expansion_payload".to_owned(),
                "type_elaboration.checker.checker.type.recovery".to_owned(),
            ]
        );

        let imported_structure_symbols = imported_structure_symbol_env(symbols.module_id().clone());
        assert_eq!(
            source_type_elaboration_detail_keys(
                &local_structure,
                imported_structure_symbols.module_id().clone(),
                &imported_structure_symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );

        let imported_fixture_structure_symbols =
            imported_fixture_structure_symbol_env(symbols.module_id().clone());
        let imported_fixture_structure = reserve_ast(
            source_id,
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("R"),
            )],
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &imported_fixture_structure,
                imported_fixture_structure_symbols.module_id().clone(),
                &imported_fixture_structure_symbols
            ),
            vec!["type_elaboration.checker.checker.declaration.deferred.evidence_query".to_owned()]
        );

        let shadowed_imported_structure_symbols =
            local_and_imported_structure_symbol_env(symbols.module_id().clone(), "R");
        let resolved_shadowed_structure_head = resolve_visible_type_head(
            &shadowed_imported_structure_symbols,
            shadowed_imported_structure_symbols.module_id(),
            "R",
        )
        .expect("a local structure should shadow an imported structure of the same spelling");
        assert_eq!(
            resolved_shadowed_structure_head.module(),
            shadowed_imported_structure_symbols.module_id()
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &imported_fixture_structure,
                shadowed_imported_structure_symbols.module_id().clone(),
                &shadowed_imported_structure_symbols
            ),
            vec!["type_elaboration.checker.checker.declaration.deferred.evidence_query".to_owned()]
        );

        let imported_fixture_type_case_struct_symbols = imported_parser_fixture_symbol_env(
            symbols.module_id().clone(),
            "TypeCaseStruct",
            SymbolKind::Structure,
        );
        let imported_fixture_type_case_struct = reserve_ast(
            source_id,
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("TypeCaseStruct"),
            )],
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &imported_fixture_type_case_struct,
                imported_fixture_type_case_struct_symbols
                    .module_id()
                    .clone(),
                &imported_fixture_type_case_struct_symbols
            ),
            vec!["type_elaboration.checker.checker.declaration.deferred.evidence_query".to_owned()]
        );

        let ambiguous_mode_symbols = ambiguous_mode_symbol_env(symbols.module_id().clone());
        assert_eq!(
            source_type_elaboration_detail_keys(
                &local_mode,
                ambiguous_mode_symbols.module_id().clone(),
                &ambiguous_mode_symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );

        let ambiguous_structure_symbols =
            ambiguous_structure_symbol_env(symbols.module_id().clone());
        assert_eq!(
            source_type_elaboration_detail_keys(
                &local_structure,
                ambiguous_structure_symbols.module_id().clone(),
                &ambiguous_structure_symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );

        let mode_with_args = reserve_ast(
            source_id,
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbolWithArgs("Mode"),
            )],
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &mode_with_args,
                mode_symbols.module_id().clone(),
                &mode_symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );

        let structure_with_args = reserve_ast(
            source_id,
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbolWithArgs("Struct"),
            )],
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &structure_with_args,
                structure_symbols.module_id().clone(),
                &structure_symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );

        let mode_attribute_symbols = source_mode_attribute_symbol_env(symbols.module_id().clone());
        let attributed_mode = reserve_ast(
            source_id,
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::AttributedQualifiedSymbol("Mode"),
            )],
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &attributed_mode,
                mode_attribute_symbols.module_id().clone(),
                &mode_attribute_symbols
            ),
            vec![
                "type_elaboration.checker.checker.type.external.mode_expansion_payload".to_owned(),
                "type_elaboration.checker.checker.type.recovery".to_owned(),
            ]
        );

        assert_eq!(
            source_type_elaboration_detail_keys(
                &attributed_mode,
                mode_symbols.module_id().clone(),
                &mode_symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );

        let imported_attribute_mode_symbols =
            imported_attribute_mode_symbol_env(symbols.module_id().clone());
        assert_eq!(
            source_type_elaboration_detail_keys(
                &attributed_mode,
                imported_attribute_mode_symbols.module_id().clone(),
                &imported_attribute_mode_symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );

        let structure_attribute_symbols =
            source_structure_attribute_symbol_env(symbols.module_id().clone());
        let attributed_structure = reserve_ast(
            source_id,
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::AttributedQualifiedSymbol("Struct"),
            )],
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &attributed_structure,
                structure_attribute_symbols.module_id().clone(),
                &structure_attribute_symbols
            ),
            vec!["type_elaboration.checker.checker.declaration.deferred.evidence_query".to_owned()]
        );

        assert_eq!(
            source_type_elaboration_detail_keys(
                &attributed_structure,
                structure_symbols.module_id().clone(),
                &structure_symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );

        let imported_attribute_structure_symbols =
            imported_attribute_structure_symbol_env(symbols.module_id().clone());
        assert_eq!(
            source_type_elaboration_detail_keys(
                &attributed_structure,
                imported_attribute_structure_symbols.module_id().clone(),
                &imported_attribute_structure_symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );

        let ambiguous_attribute_structure_symbols =
            ambiguous_attribute_structure_symbol_env(symbols.module_id().clone());
        assert_eq!(
            source_type_elaboration_detail_keys(
                &attributed_structure,
                ambiguous_attribute_structure_symbols.module_id().clone(),
                &ambiguous_attribute_structure_symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );

        let attributed_structure_with_attribute_args = reserve_ast(
            source_id,
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::AttributedQualifiedSymbolWithAttributeArgs("Struct"),
            )],
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &attributed_structure_with_attribute_args,
                structure_attribute_symbols.module_id().clone(),
                &structure_attribute_symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );

        let attributed_mode_with_attribute_args = reserve_ast(
            source_id,
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::AttributedQualifiedSymbolWithAttributeArgs("Mode"),
            )],
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &attributed_mode_with_attribute_args,
                mode_attribute_symbols.module_id().clone(),
                &mode_attribute_symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );

        let qualified_attribute_mode = reserve_ast(
            source_id,
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedAttributeQualifiedSymbol("Mode"),
            )],
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &qualified_attribute_mode,
                mode_attribute_symbols.module_id().clone(),
                &mode_attribute_symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );

        let qualified_attribute_structure = reserve_ast(
            source_id,
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedAttributeQualifiedSymbol("Struct"),
            )],
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &qualified_attribute_structure,
                structure_attribute_symbols.module_id().clone(),
                &structure_attribute_symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );
    }

    #[test]
    fn source_four_edge_local_mode_reserved_variable_equality_consumes_five_expansions() {
        let source_id = source_id(166);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("four_edge_local_mode_reserved_variable_equality"),
        );
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseFourEdgeModeEquality", SymbolKind::Mode),
                ("InnerFourEdgeModeEquality", SymbolKind::Mode),
                ("MiddleFourEdgeModeEquality", SymbolKind::Mode),
                ("OuterFourEdgeModeEquality", SymbolKind::Mode),
                ("TooDeepFourEdgeModeEquality", SymbolKind::Mode),
                ("ExtraFourEdgeModeEquality", SymbolKind::Mode),
            ],
        );
        let theorem = IdentifierBinaryTheoremSpec {
            status: None,
            label: "FourEdgeLocalModeReservedVariableEqualityPayloadBoundary",
            left: "z",
            operator: "=",
            right: "z",
            recovered_label: false,
        };
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    "BaseFourEdgeModeEquality",
                    "BaseFourEdgeModeEqualityDef",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition_with_label(
                    "InnerFourEdgeModeEquality",
                    "InnerFourEdgeModeEqualityDef",
                    ReserveTypeShape::QualifiedSymbol("BaseFourEdgeModeEquality"),
                ),
                mode_definition_with_label(
                    "MiddleFourEdgeModeEquality",
                    "MiddleFourEdgeModeEqualityDef",
                    ReserveTypeShape::QualifiedSymbol("InnerFourEdgeModeEquality"),
                ),
                mode_definition_with_label(
                    "OuterFourEdgeModeEquality",
                    "OuterFourEdgeModeEqualityDef",
                    ReserveTypeShape::QualifiedSymbol("MiddleFourEdgeModeEquality"),
                ),
                mode_definition_with_label(
                    "TooDeepFourEdgeModeEquality",
                    "TooDeepFourEdgeModeEqualityDef",
                    ReserveTypeShape::QualifiedSymbol("OuterFourEdgeModeEquality"),
                ),
            ]
        };
        let reserve = || {
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("TooDeepFourEdgeModeEquality"),
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
        let payload = extract_source_four_edge_local_mode_reserved_variable_equality(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact four-edge local-mode reserved-variable equality should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 5);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(payload.reserve.bridge.bindings()[0].spelling, "z");
        assert_eq!(
            payload.reserve.bridge.bindings()[0].type_spelling,
            "TooDeepFourEdgeModeEquality"
        );
        assert_eq!(payload.left_lookup_ordinal, 1);
        assert_eq!(payload.right_lookup_ordinal, 2);

        let output = source_four_edge_local_mode_reserved_variable_equality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact four-edge local-mode equality should reach TermFormulaChecker");
        assert_source_reserved_variable_formula_output(&output)
            .expect("four-edge local-mode equality invariants should hold");
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
            assert_eq!(input.spelling, "TooDeepFourEdgeModeEquality");
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
                source_mode_symbol_spelling(symbol) == Some("BaseFourEdgeModeEquality")
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
                TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY
                    .to_owned(),
            ]
        };
        let mut missing_left_expected =
            source_four_edge_local_mode_reserved_variable_equality_output(
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
            source_four_edge_local_mode_reserved_variable_equality_output(
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
            "BaseFourEdgeModeEquality",
            "InnerFourEdgeModeEquality",
            "MiddleFourEdgeModeEquality",
            "OuterFourEdgeModeEquality",
            "TooDeepFourEdgeModeEquality",
        ] {
            let mut invalid = source_four_edge_local_mode_reserved_variable_equality_output(
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
                    TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY
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
            "BaseFourEdgeModeEquality",
            "InnerFourEdgeModeEquality",
            "MiddleFourEdgeModeEquality",
            "OuterFourEdgeModeEquality",
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
                ReserveTypeShape::QualifiedSymbol("ExtraFourEdgeModeEquality");
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
            ReserveTypeShape::QualifiedSymbol("BaseFourEdgeModeEquality");
        assert_extraction_gap(one_edge_outermost_radix);

        let mut two_edge_outermost_radix = exact_modes();
        two_edge_outermost_radix[4].rhs_shape =
            ReserveTypeShape::QualifiedSymbol("InnerFourEdgeModeEquality");
        assert_extraction_gap(two_edge_outermost_radix);

        let mut three_edge_outermost_radix = exact_modes();
        three_edge_outermost_radix[4].rhs_shape =
            ReserveTypeShape::QualifiedSymbol("MiddleFourEdgeModeEquality");
        assert_extraction_gap(three_edge_outermost_radix);

        let mut cyclic = exact_modes();
        cyclic[0].rhs_shape = ReserveTypeShape::QualifiedSymbol("TooDeepFourEdgeModeEquality");
        assert_extraction_gap(cyclic);

        assert_extraction_gap(vec![
            exact_modes()[0],
            mode_definition_with_label(
                "ExtraFourEdgeModeEquality",
                "ExtraFourEdgeModeEqualityDef",
                ReserveTypeShape::QualifiedSymbol("BaseFourEdgeModeEquality"),
            ),
            mode_definition_with_label(
                "InnerFourEdgeModeEquality",
                "InnerFourEdgeModeEqualityDef",
                ReserveTypeShape::QualifiedSymbol("ExtraFourEdgeModeEquality"),
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
                    ReserveTypeShape::QualifiedSymbolWithArgs("TooDeepFourEdgeModeEquality"),
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
    fn active_four_edge_local_mode_reserved_variable_equality_fixture_consumes_five_expansions() {
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
                    == "pass_type_elaboration_four_edge_local_mode_reserved_variable_equality_001"
            })
            .expect("Task 166 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 166 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 166 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_four_edge_local_mode_reserved_variable_equality_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 166 real AST should reach the four-edge local-mode equality seam");
        assert_source_reserved_variable_formula_output(&output)
            .expect("Task 166 real AST should preserve every checked payload invariant");
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
                source_mode_symbol_spelling(symbol) == Some("BaseFourEdgeModeEquality")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("Task 166 base terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 166 normalized set type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
    }
