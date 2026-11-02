#[test]
fn source_local_object_mode_long_chain_six_hop_asserted_head_consumes_seven_expansions() {
    let source_id = source_id(236);
    let module = ResolverModuleId::new(
        PackageId::new("test"),
        ModulePath::new("local_object_mode_long_chain_six_hop_asserted_head"),
    );
    let mode_names = [
        "BaseObjectMode",
        "ChainObjectMode1",
        "ChainObjectMode2",
        "ChainObjectMode3",
        "ChainObjectMode4",
        "ChainObjectMode5",
        "ChainObjectMode6",
    ];
    let mode_labels = [
        "BaseObjectModeDef",
        "ChainObjectMode1Def",
        "ChainObjectMode2Def",
        "ChainObjectMode3Def",
        "ChainObjectMode4Def",
        "ChainObjectMode5Def",
        "ChainObjectMode6Def",
    ];
    let symbols = source_local_symbols_env(
        module.clone(),
        &[
            (mode_names[0], SymbolKind::Mode),
            (mode_names[1], SymbolKind::Mode),
            (mode_names[2], SymbolKind::Mode),
            (mode_names[3], SymbolKind::Mode),
            (mode_names[4], SymbolKind::Mode),
            (mode_names[5], SymbolKind::Mode),
            (mode_names[6], SymbolKind::Mode),
            ("ChainObjectMode7", SymbolKind::Mode),
            ("OtherLongMode", SymbolKind::Mode),
        ],
    );
    let theorem = IdentifierTypeAssertionTheoremSpec {
        status: None,
        label: "LongLocalObjectModeSixHopAssertedHeadPayloadBoundary",
        subject: "x",
        asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[0]),
        negated: false,
        recovered_label: false,
    };
    let exact_modes = || {
        vec![
            mode_definition_with_label(
                mode_names[0],
                mode_labels[0],
                ReserveTypeShape::Builtin("object"),
            ),
            mode_definition_with_label(
                mode_names[1],
                mode_labels[1],
                ReserveTypeShape::QualifiedSymbol(mode_names[0]),
            ),
            mode_definition_with_label(
                mode_names[2],
                mode_labels[2],
                ReserveTypeShape::QualifiedSymbol(mode_names[1]),
            ),
            mode_definition_with_label(
                mode_names[3],
                mode_labels[3],
                ReserveTypeShape::QualifiedSymbol(mode_names[2]),
            ),
            mode_definition_with_label(
                mode_names[4],
                mode_labels[4],
                ReserveTypeShape::QualifiedSymbol(mode_names[3]),
            ),
            mode_definition_with_label(
                mode_names[5],
                mode_labels[5],
                ReserveTypeShape::QualifiedSymbol(mode_names[4]),
            ),
            mode_definition_with_label(
                mode_names[6],
                mode_labels[6],
                ReserveTypeShape::QualifiedSymbol(mode_names[5]),
            ),
        ]
    };
    let reserve = || {
        vec![reserve_item(
            vec!["x"],
            ReserveTypeShape::QualifiedSymbol(mode_names[6]),
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
        extract_source_local_mode_long_chain_reserved_variable_type_assertion(
            &exact,
            module.clone(),
            &symbols,
        )
        .is_none(),
        "Task 175 must reject the Task 236 six-hop source"
    );
    assert!(
        super::extract_source_local_mode_long_chain_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .is_none(),
        "Task 199 must reject the Task 236 six-hop source"
    );
    assert!(
        super::extract_source_local_mode_long_chain_two_hop_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .is_none(),
        "Task 224 must reject the Task 236 six-hop source"
    );
    assert!(
        super::extract_source_local_object_mode_long_chain_two_hop_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .is_none(),
        "Task 225 must reject the Task 236 object-terminal six-hop source"
    );
    assert!(
        super::extract_source_local_mode_long_chain_three_hop_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .is_none(),
        "Task 226 must reject the Task 236 six-hop source"
    );
    assert!(
        super::extract_source_local_object_mode_long_chain_three_hop_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .is_none(),
        "Task 227 must reject the Task 236 object-terminal six-hop source"
    );
    assert!(
        super::extract_source_local_mode_long_chain_four_hop_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .is_none(),
        "Task 228 must reject the Task 236 six-hop source"
    );
    assert!(
        super::extract_source_local_object_mode_long_chain_four_hop_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .is_none(),
        "Task 229 must reject the Task 236 object-terminal six-hop source"
    );
    assert!(
        super::extract_source_local_mode_long_chain_five_hop_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .is_none(),
        "Task 230 must reject the Task 236 six-hop source"
    );
    assert!(
        super::extract_source_local_object_mode_long_chain_five_hop_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .is_none(),
        "Task 231 must reject the Task 236 object-terminal six-hop source"
    );
    assert!(
        super::extract_source_local_mode_long_chain_six_hop_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .is_none(),
        "Task 234 must reject the Task 236 object-terminal six-hop source"
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
    ];
    assert_eq!(preexisting_owner_results.len(), 34);
    assert!(
        preexisting_owner_results
            .into_iter()
            .all(|result| result.is_none()),
        "all 34 legacy type-assertion owner routes must reject Task 236"
    );
    let payload = super::extract_source_local_object_mode_long_chain_six_hop_asserted_head(
        &exact,
        module.clone(),
        &symbols,
    )
    .expect("exact seven-expansion six-hop source should extract");
    assert_eq!(payload.reserve.mode_expansions.len(), 7);
    assert_eq!(payload.reserve.bridge.bindings().len(), 1);
    assert_eq!(payload.subject_lookup_ordinal, 1);
    let source_binding = &payload.reserve.bridge.bindings()[0];
    assert_eq!(source_binding.type_spelling, mode_names[6]);
    assert_eq!(payload.asserted_type.spelling, mode_names[0]);
    assert_ne!(payload.asserted_type.head, source_binding.type_head);
    assert!(matches!(source_binding.type_head, TypeHeadInput::Symbol(_)));
    assert_ne!(payload.asserted_type_site, payload.subject_site);
    assert_ne!(payload.asserted_type.range, source_binding.type_range);
    let binding_expansion = payload
        .reserve
        .mode_expansions
        .iter()
        .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[6]))
        .map(|(_, expansion)| expansion)
        .expect("ChainObjectMode6 binding expansion should exist");
    assert_eq!(binding_expansion.radix.spelling, mode_names[5]);
    assert_ne!(binding_expansion.radix.head, payload.asserted_type.head);
    let TypeHeadInput::Symbol(first_intermediate_symbol) = &binding_expansion.radix.head else {
        panic!("ChainObjectMode5 first relation intermediate should resolve to a symbol");
    };
    let first_intermediate_expansion = payload
        .reserve
        .mode_expansions
        .get(first_intermediate_symbol)
        .expect("ChainObjectMode5 first intermediate expansion should exist");
    assert_eq!(first_intermediate_expansion.radix.spelling, mode_names[4]);
    assert_ne!(
        first_intermediate_expansion.radix.head,
        payload.asserted_type.head
    );
    let TypeHeadInput::Symbol(second_intermediate_symbol) =
        &first_intermediate_expansion.radix.head
    else {
        panic!("ChainObjectMode4 second relation intermediate should resolve to a symbol");
    };
    let second_intermediate_expansion = payload
        .reserve
        .mode_expansions
        .get(second_intermediate_symbol)
        .expect("ChainObjectMode4 second intermediate expansion should exist");
    assert_eq!(second_intermediate_expansion.radix.spelling, mode_names[3]);
    assert_ne!(
        second_intermediate_expansion.radix.head,
        payload.asserted_type.head
    );
    let TypeHeadInput::Symbol(third_intermediate_symbol) =
        &second_intermediate_expansion.radix.head
    else {
        panic!("ChainObjectMode3 third relation intermediate should resolve to a symbol");
    };
    let third_intermediate_expansion = payload
        .reserve
        .mode_expansions
        .get(third_intermediate_symbol)
        .expect("ChainObjectMode3 third intermediate expansion should exist");
    assert_eq!(third_intermediate_expansion.radix.spelling, mode_names[2]);
    assert_ne!(
        third_intermediate_expansion.radix.head,
        payload.asserted_type.head
    );
    let TypeHeadInput::Symbol(fourth_intermediate_symbol) =
        &third_intermediate_expansion.radix.head
    else {
        panic!("ChainObjectMode2 fourth relation intermediate should resolve to a symbol");
    };
    let fourth_intermediate_expansion = payload
        .reserve
        .mode_expansions
        .get(fourth_intermediate_symbol)
        .expect("ChainObjectMode2 fourth intermediate expansion should exist");
    assert_eq!(fourth_intermediate_expansion.radix.spelling, mode_names[1]);
    assert_ne!(
        fourth_intermediate_expansion.radix.head,
        payload.asserted_type.head
    );
    let TypeHeadInput::Symbol(fifth_intermediate_symbol) =
        &fourth_intermediate_expansion.radix.head
    else {
        panic!("ChainObjectMode1 fifth relation intermediate should resolve to a symbol");
    };
    let fifth_intermediate_expansion = payload
        .reserve
        .mode_expansions
        .get(fifth_intermediate_symbol)
        .expect("ChainObjectMode1 fifth intermediate expansion should exist");
    assert_eq!(fifth_intermediate_expansion.radix.spelling, mode_names[0]);
    assert_eq!(
        fifth_intermediate_expansion.radix.head,
        payload.asserted_type.head
    );

    let output = super::source_local_object_mode_long_chain_six_hop_asserted_head_output(
        &exact,
        module.clone(),
        &symbols,
    )
    .expect("exact seven-expansion asserted-head source should reach checker");
    assert_source_reserved_variable_type_assertion_output(&output)
        .expect("Task 236 checker payload invariants should hold");
    assert_eq!(output.subject_binding, BindingId::new(0));
    assert_eq!(output.payload.subject_lookup_ordinal, 1);
    assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
    assert_eq!(output.term_formula.type_entries().len(), 3);
    assert_eq!(output.term_formula.normalized_types().len(), 1);
    assert_eq!(output.subject_result_input.spelling, mode_names[6]);
    assert_eq!(output.asserted_type_input.spelling, mode_names[0]);
    assert_ne!(
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
        .expect("BaseObjectMode terminal expansion should exist");
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
                super::TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_SIX_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
            ),
            vec![
                super::TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_SIX_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY
                    .to_owned()
            ]
        );
    };
    for removed in mode_names {
        let mut invalid =
            super::source_local_object_mode_long_chain_six_hop_asserted_head_output(
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
        super::source_local_object_mode_long_chain_six_hop_asserted_head_output(
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
    invalid.subject_result_input.spelling = mode_names[5].to_owned();
    corruptions.push(invalid);
    let mut invalid = exact_output();
    invalid.asserted_type_input.spelling = "OtherLongMode".to_owned();
    corruptions.push(invalid);
    let mut invalid = exact_output();
    invalid.asserted_type_input.head = invalid.subject_result_input.head.clone();
    corruptions.push(invalid);
    let mut invalid = exact_output();
    invalid.asserted_type_input.head = TypeHeadInput::BuiltinSet;
    corruptions.push(invalid);
    let mut invalid = exact_output();
    invalid.subject_result_input.head = TypeHeadInput::BuiltinSet;
    corruptions.push(invalid);
    let mut invalid = exact_output();
    invalid.subject_result_input.source_range = invalid.asserted_type_input.source_range;
    corruptions.push(invalid);
    let mut invalid = exact_output();
    invalid.subject_result_input.site = invalid.asserted_type_input.site.clone();
    corruptions.push(invalid);
    let mut invalid = exact_output();
    invalid.asserted_type_input.source_range = invalid.subject_result_input.source_range;
    corruptions.push(invalid);
    let mut invalid = exact_output();
    invalid.asserted_type_input.site = invalid.subject_result_input.site.clone();
    corruptions.push(invalid);
    let mut invalid = exact_output();
    let binding_head = invalid.subject_result_input.head.clone();
    let (_, binding_expansion) = invalid
        .payload
        .reserve
        .mode_expansions
        .iter_mut()
        .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[6]))
        .unwrap();
    binding_expansion.radix.head = binding_head;
    corruptions.push(invalid);
    for mode_name in &mode_names[1..] {
        let mut invalid = exact_output();
        let (_, expansion) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(*mode_name))
            .unwrap();
        expansion.radix.spelling = "set".to_owned();
        expansion.radix.head = TypeHeadInput::BuiltinSet;
        corruptions.push(invalid);
    }
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
    let mut invalid = exact_output();
    let (_, base_expansion) = invalid
        .payload
        .reserve
        .mode_expansions
        .iter_mut()
        .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[0]))
        .unwrap();
    base_expansion.radix.spelling = "set".to_owned();
    base_expansion.radix.head = TypeHeadInput::BuiltinSet;
    corruptions.push(invalid);
    for invalid in corruptions {
        assert_invalid_output(invalid);
    }
    assert_source_reserved_variable_type_assertion_output(&exact_output())
        .expect("corrupting cloned Task 236 outputs must not mutate the exact output");

    let assert_extraction_gap = |ast, context: &str| {
        assert_eq!(
            source_type_elaboration_detail_keys(&ast, module.clone(), &symbols),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()],
            "{context}"
        );
    };
    fn next_permutation(order: &mut [usize; 7]) -> bool {
        let Some(pivot) = (0..order.len() - 1)
            .rev()
            .find(|&index| order[index] < order[index + 1])
        else {
            return false;
        };
        let successor = (pivot + 1..order.len())
            .rev()
            .find(|&index| order[pivot] < order[index])
            .expect("a lexicographic successor must exist after the pivot");
        order.swap(pivot, successor);
        order[pivot + 1..].reverse();
        true
    }
    let ordered = exact_modes();
    let public_dispatch_orders = [
        [1, 0, 2, 3, 4, 5, 6],
        [0, 2, 1, 3, 4, 5, 6],
        [0, 1, 3, 2, 4, 5, 6],
        [0, 1, 2, 4, 3, 5, 6],
        [0, 1, 2, 3, 5, 4, 6],
        [0, 1, 2, 3, 4, 6, 5],
        [1, 2, 3, 4, 5, 6, 0],
        [6, 5, 4, 3, 2, 1, 0],
    ];
    let mut order = [0, 1, 2, 3, 4, 5, 6];
    let mut permutation_count = 0;
    let mut public_dispatch_representative_count = 0;
    while next_permutation(&mut order) {
        permutation_count += 1;
        let permuted = mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            vec![
                ordered[order[0]],
                ordered[order[1]],
                ordered[order[2]],
                ordered[order[3]],
                ordered[order[4]],
                ordered[order[5]],
                ordered[order[6]],
            ],
            reserve(),
            theorem,
        );
        assert!(
            super::extract_source_local_object_mode_long_chain_six_hop_asserted_head(
                &permuted,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "definition permutation {order:?} must not reach Task 236"
        );
        if public_dispatch_orders.contains(&order) {
            public_dispatch_representative_count += 1;
            assert_extraction_gap(
                permuted,
                &format!("public definition permutation {order:?}"),
            );
        }
    }
    assert_eq!(permutation_count, 5_039);
    assert_eq!(public_dispatch_representative_count, 8);
    let mut mode_near_misses = vec![
        Vec::<ModeDefinitionSpec>::new(),
        {
            let mut modes = exact_modes();
            modes.push(modes[0]);
            modes
        },
        {
            let mut modes = exact_modes();
            modes.push(mode_definition_with_label(
                "UnusedLongMode",
                "UnusedLongModeDef",
                ReserveTypeShape::QualifiedSymbol(mode_names[6]),
            ));
            modes
        },
    ];
    for index in 0..mode_names.len() {
        let mut missing = exact_modes();
        missing.remove(index);
        mode_near_misses.push(missing);
        let mut duplicate = exact_modes();
        duplicate.push(duplicate[index]);
        mode_near_misses.push(duplicate);
        let mut malformed = exact_modes();
        malformed[index].rhs_shape = if index == 0 {
            ReserveTypeShape::Builtin("set")
        } else {
            ReserveTypeShape::QualifiedSymbol(mode_names[(index + 1) % mode_names.len()])
        };
        mode_near_misses.push(malformed);
        let mut wrong_label = exact_modes();
        wrong_label[index].label = Some("WrongLongModeDef");
        mode_near_misses.push(wrong_label);
        let mut wrong_spelling = exact_modes();
        wrong_spelling[index].pattern = "OtherLongMode";
        mode_near_misses.push(wrong_spelling);
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
        "ChainObjectMode7",
        "ChainObjectMode7Def",
        ReserveTypeShape::QualifiedSymbol(mode_names[6]),
    ));
    source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
        source_id,
        deeper_modes,
        vec![reserve_item(
            vec!["x"],
            ReserveTypeShape::QualifiedSymbol("ChainObjectMode7"),
        )],
        IdentifierTypeAssertionTheoremSpec {
            asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[6]),
            ..theorem
        },
    ));
    for bad_reserve in [
        ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[6]),
        ReserveTypeShape::AttributedQualifiedSymbol(mode_names[6]),
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
        vec![reserve_item(
            vec!["y"],
            ReserveTypeShape::QualifiedSymbol(mode_names[6]),
        )],
        theorem,
    ));
    source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
        source_id,
        exact_modes(),
        vec![
            reserve_item(vec!["x"], ReserveTypeShape::QualifiedSymbol(mode_names[6])),
            reserve_item(vec!["y"], ReserveTypeShape::Builtin("object")),
        ],
        theorem,
    ));
    for bad_reserve in [
        vec![reserve_item(
            vec!["x"],
            ReserveTypeShape::QualifiedSymbol(mode_names[5]),
        )],
        vec![reserve_item(
            vec!["x"],
            ReserveTypeShape::QualifiedSymbol("OtherLongMode"),
        )],
        vec![reserve_item(
            vec!["x", "y"],
            ReserveTypeShape::QualifiedSymbol(mode_names[6]),
        )],
    ] {
        source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            exact_modes(),
            bad_reserve,
            theorem,
        ));
    }
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
            asserted_type: ReserveTypeShape::Builtin("set"),
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
            asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[4]),
            ..theorem
        },
        IdentifierTypeAssertionTheoremSpec {
            asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[5]),
            ..theorem
        },
        IdentifierTypeAssertionTheoremSpec {
            asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[6]),
            ..theorem
        },
        IdentifierTypeAssertionTheoremSpec {
            asserted_type: ReserveTypeShape::QualifiedSymbol("OtherLongMode"),
            ..theorem
        },
        IdentifierTypeAssertionTheoremSpec {
            asserted_type: ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[0]),
            ..theorem
        },
        IdentifierTypeAssertionTheoremSpec {
            asserted_type: ReserveTypeShape::AttributedQualifiedSymbol(mode_names[0]),
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
        for imported in [
            vec![(mode_names[imported_index], SymbolKind::Mode)],
            vec![
                (mode_names[imported_index], SymbolKind::Mode),
                (mode_names[imported_index], SymbolKind::Mode),
            ],
        ] {
            let imported_symbols = source_local_and_imported_symbols_env(
                module.clone(),
                &local_modes,
                imported.as_slice(),
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&exact, module.clone(), &imported_symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
    }
    let unrelated_import = source_local_and_imported_symbols_env(
        module.clone(),
        &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
        &[("UnrelatedLongMode", SymbolKind::Mode)],
    );
    assert!(
        super::extract_source_local_object_mode_long_chain_six_hop_asserted_head(
            &exact,
            module.clone(),
            &unrelated_import,
        )
        .is_some()
    );
    for (asserted_spelling, asserted_head_symbols) in [
        (
            "ImportedLongMode",
            source_local_and_imported_symbols_env(
                module.clone(),
                &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
                &[("ImportedLongMode", SymbolKind::Mode)],
            ),
        ),
        (
            "AmbiguousLongMode",
            source_local_and_imported_symbols_env(
                module.clone(),
                &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
                &[
                    ("AmbiguousLongMode", SymbolKind::Mode),
                    ("AmbiguousLongMode", SymbolKind::Mode),
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
fn source_local_object_mode_long_chain_five_hop_asserted_head_consumes_seven_expansions() {
    let source_id = source_id(231);
    let module = ResolverModuleId::new(
        PackageId::new("test"),
        ModulePath::new("local_object_mode_long_chain_five_hop_asserted_head"),
    );
    let mode_names = [
        "BaseObjectMode",
        "ChainObjectMode1",
        "ChainObjectMode2",
        "ChainObjectMode3",
        "ChainObjectMode4",
        "ChainObjectMode5",
        "ChainObjectMode6",
    ];
    let mode_labels = [
        "BaseObjectModeDef",
        "ChainObjectMode1Def",
        "ChainObjectMode2Def",
        "ChainObjectMode3Def",
        "ChainObjectMode4Def",
        "ChainObjectMode5Def",
        "ChainObjectMode6Def",
    ];
    let symbols = source_local_symbols_env(
        module.clone(),
        &[
            (mode_names[0], SymbolKind::Mode),
            (mode_names[1], SymbolKind::Mode),
            (mode_names[2], SymbolKind::Mode),
            (mode_names[3], SymbolKind::Mode),
            (mode_names[4], SymbolKind::Mode),
            (mode_names[5], SymbolKind::Mode),
            (mode_names[6], SymbolKind::Mode),
            ("ChainObjectMode7", SymbolKind::Mode),
            ("OtherLongMode", SymbolKind::Mode),
        ],
    );
    let theorem = IdentifierTypeAssertionTheoremSpec {
        status: None,
        label: "LongLocalObjectModeFiveHopAssertedHeadPayloadBoundary",
        subject: "x",
        asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[1]),
        negated: false,
        recovered_label: false,
    };
    let exact_modes = || {
        vec![
            mode_definition_with_label(
                mode_names[0],
                mode_labels[0],
                ReserveTypeShape::Builtin("object"),
            ),
            mode_definition_with_label(
                mode_names[1],
                mode_labels[1],
                ReserveTypeShape::QualifiedSymbol(mode_names[0]),
            ),
            mode_definition_with_label(
                mode_names[2],
                mode_labels[2],
                ReserveTypeShape::QualifiedSymbol(mode_names[1]),
            ),
            mode_definition_with_label(
                mode_names[3],
                mode_labels[3],
                ReserveTypeShape::QualifiedSymbol(mode_names[2]),
            ),
            mode_definition_with_label(
                mode_names[4],
                mode_labels[4],
                ReserveTypeShape::QualifiedSymbol(mode_names[3]),
            ),
            mode_definition_with_label(
                mode_names[5],
                mode_labels[5],
                ReserveTypeShape::QualifiedSymbol(mode_names[4]),
            ),
            mode_definition_with_label(
                mode_names[6],
                mode_labels[6],
                ReserveTypeShape::QualifiedSymbol(mode_names[5]),
            ),
        ]
    };
    let reserve = || {
        vec![reserve_item(
            vec!["x"],
            ReserveTypeShape::QualifiedSymbol(mode_names[6]),
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
        extract_source_local_object_mode_long_chain_reserved_variable_type_assertion(
            &exact,
            module.clone(),
            &symbols,
        )
        .is_none(),
        "Task 179 must reject the Task 231 five-hop source"
    );
    assert!(
        super::extract_source_local_object_mode_long_chain_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .is_none(),
        "Task 200 must reject the Task 231 five-hop source"
    );
    assert!(
        super::extract_source_local_object_mode_long_chain_two_hop_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .is_none(),
        "Task 225 must reject the Task 231 five-hop source"
    );
    assert!(
        super::extract_source_local_mode_long_chain_two_hop_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .is_none(),
        "Task 224 must reject the Task 231 object-terminal five-hop source"
    );
    assert!(
        super::extract_source_local_object_mode_long_chain_three_hop_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .is_none(),
        "Task 227 must reject the Task 231 five-hop source"
    );
    assert!(
        super::extract_source_local_mode_long_chain_three_hop_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .is_none(),
        "Task 226 must reject the Task 231 object-terminal five-hop source"
    );
    assert!(
        super::extract_source_local_object_mode_long_chain_four_hop_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .is_none(),
        "Task 229 must reject the Task 231 five-hop source"
    );
    assert!(
        super::extract_source_local_mode_long_chain_four_hop_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .is_none(),
        "Task 228 must reject the Task 231 object-terminal five-hop source"
    );
    let preexisting_owner_results = [
        extract_source_reserved_variable_type_assertion(&exact, module.clone(), &symbols),
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
        extract_source_local_mode_reserved_variable_type_assertion(
            &exact,
            module.clone(),
            &symbols,
        ),
        extract_source_local_object_mode_asserted_head(&exact, module.clone(), &symbols),
        extract_source_local_mode_asserted_head(&exact, module.clone(), &symbols),
        extract_source_chained_local_object_mode_reserved_variable_type_assertion(
            &exact,
            module.clone(),
            &symbols,
        ),
        extract_source_chained_local_mode_reserved_variable_type_assertion(
            &exact,
            module.clone(),
            &symbols,
        ),
        extract_source_chained_local_object_mode_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        ),
        extract_source_chained_local_mode_asserted_head(&exact, module.clone(), &symbols),
        extract_source_chained_local_object_mode_radix_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        ),
        extract_source_chained_local_mode_radix_asserted_head(&exact, module.clone(), &symbols),
        extract_source_two_edge_local_object_mode_reserved_variable_type_assertion(
            &exact,
            module.clone(),
            &symbols,
        ),
        extract_source_two_edge_local_mode_reserved_variable_type_assertion(
            &exact,
            module.clone(),
            &symbols,
        ),
        extract_source_two_edge_local_object_mode_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        ),
        extract_source_two_edge_local_mode_asserted_head(&exact, module.clone(), &symbols),
        extract_source_two_edge_local_object_mode_radix_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        ),
        extract_source_two_edge_local_mode_radix_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        ),
        extract_source_three_edge_local_object_mode_reserved_variable_type_assertion(
            &exact,
            module.clone(),
            &symbols,
        ),
        extract_source_three_edge_local_mode_reserved_variable_type_assertion(
            &exact,
            module.clone(),
            &symbols,
        ),
        extract_source_three_edge_local_object_mode_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        ),
        extract_source_three_edge_local_mode_asserted_head(&exact, module.clone(), &symbols),
        extract_source_three_edge_local_object_mode_radix_asserted_head(
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
        extract_source_four_edge_local_mode_reserved_variable_type_assertion(
            &exact,
            module.clone(),
            &symbols,
        ),
        super::extract_source_four_edge_local_object_mode_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        ),
        super::extract_source_four_edge_local_mode_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        ),
        extract_source_four_edge_local_object_mode_radix_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        ),
        extract_source_four_edge_local_mode_radix_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        ),
        extract_source_local_object_mode_long_chain_reserved_variable_type_assertion(
            &exact,
            module.clone(),
            &symbols,
        ),
        extract_source_local_mode_long_chain_reserved_variable_type_assertion(
            &exact,
            module.clone(),
            &symbols,
        ),
        super::extract_source_local_object_mode_long_chain_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        ),
        super::extract_source_local_mode_long_chain_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        ),
    ];
    assert_eq!(preexisting_owner_results.len(), 34);
    assert!(
        preexisting_owner_results
            .into_iter()
            .all(|result| result.is_none()),
        "all 34 legacy type-assertion owner routes must reject Task 231"
    );
    let payload = super::extract_source_local_object_mode_long_chain_five_hop_asserted_head(
        &exact,
        module.clone(),
        &symbols,
    )
    .expect("exact seven-expansion five-hop source should extract");
    assert_eq!(payload.reserve.mode_expansions.len(), 7);
    assert_eq!(payload.reserve.bridge.bindings().len(), 1);
    assert_eq!(payload.subject_lookup_ordinal, 1);
    let source_binding = &payload.reserve.bridge.bindings()[0];
    assert_eq!(source_binding.type_spelling, mode_names[6]);
    assert_eq!(payload.asserted_type.spelling, mode_names[1]);
    assert_ne!(payload.asserted_type.head, source_binding.type_head);
    assert!(matches!(source_binding.type_head, TypeHeadInput::Symbol(_)));
    assert_ne!(payload.asserted_type_site, payload.subject_site);
    assert_ne!(payload.asserted_type.range, source_binding.type_range);
    let binding_expansion = payload
        .reserve
        .mode_expansions
        .iter()
        .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[6]))
        .map(|(_, expansion)| expansion)
        .expect("ChainObjectMode6 binding expansion should exist");
    assert_eq!(binding_expansion.radix.spelling, mode_names[5]);
    assert_ne!(binding_expansion.radix.head, payload.asserted_type.head);
    let TypeHeadInput::Symbol(first_intermediate_symbol) = &binding_expansion.radix.head else {
        panic!("ChainObjectMode5 first relation intermediate should resolve to a symbol");
    };
    let first_intermediate_expansion = payload
        .reserve
        .mode_expansions
        .get(first_intermediate_symbol)
        .expect("ChainObjectMode5 first intermediate expansion should exist");
    assert_eq!(first_intermediate_expansion.radix.spelling, mode_names[4]);
    assert_ne!(
        first_intermediate_expansion.radix.head,
        payload.asserted_type.head
    );
    let TypeHeadInput::Symbol(second_intermediate_symbol) =
        &first_intermediate_expansion.radix.head
    else {
        panic!("ChainObjectMode4 second relation intermediate should resolve to a symbol");
    };
    let second_intermediate_expansion = payload
        .reserve
        .mode_expansions
        .get(second_intermediate_symbol)
        .expect("ChainObjectMode4 second intermediate expansion should exist");
    assert_eq!(second_intermediate_expansion.radix.spelling, mode_names[3]);
    assert_ne!(
        second_intermediate_expansion.radix.head,
        payload.asserted_type.head
    );
    let TypeHeadInput::Symbol(third_intermediate_symbol) =
        &second_intermediate_expansion.radix.head
    else {
        panic!("ChainObjectMode3 third relation intermediate should resolve to a symbol");
    };
    let third_intermediate_expansion = payload
        .reserve
        .mode_expansions
        .get(third_intermediate_symbol)
        .expect("ChainObjectMode3 third intermediate expansion should exist");
    assert_eq!(third_intermediate_expansion.radix.spelling, mode_names[2]);
    assert_ne!(
        third_intermediate_expansion.radix.head,
        payload.asserted_type.head
    );
    let TypeHeadInput::Symbol(fourth_intermediate_symbol) =
        &third_intermediate_expansion.radix.head
    else {
        panic!("ChainObjectMode2 fourth relation intermediate should resolve to a symbol");
    };
    let fourth_intermediate_expansion = payload
        .reserve
        .mode_expansions
        .get(fourth_intermediate_symbol)
        .expect("ChainObjectMode2 fourth intermediate expansion should exist");
    assert_eq!(fourth_intermediate_expansion.radix.spelling, mode_names[1]);
    assert_eq!(
        fourth_intermediate_expansion.radix.head,
        payload.asserted_type.head
    );

    let output = super::source_local_object_mode_long_chain_five_hop_asserted_head_output(
        &exact,
        module.clone(),
        &symbols,
    )
    .expect("exact seven-expansion asserted-head source should reach checker");
    assert_source_reserved_variable_type_assertion_output(&output)
        .expect("Task 231 checker payload invariants should hold");
    assert_eq!(output.subject_binding, BindingId::new(0));
    assert_eq!(output.payload.subject_lookup_ordinal, 1);
    assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
    assert_eq!(output.term_formula.type_entries().len(), 3);
    assert_eq!(output.term_formula.normalized_types().len(), 1);
    assert_eq!(output.subject_result_input.spelling, mode_names[6]);
    assert_eq!(output.asserted_type_input.spelling, mode_names[1]);
    assert_ne!(
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
        .expect("BaseObjectMode terminal expansion should exist");
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
                super::TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_FIVE_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
            ),
            vec![
                super::TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_FIVE_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY
                    .to_owned()
            ]
        );
    };
    for removed in mode_names {
        let mut invalid =
            super::source_local_object_mode_long_chain_five_hop_asserted_head_output(
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
        super::source_local_object_mode_long_chain_five_hop_asserted_head_output(
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
    invalid.subject_result_input.spelling = mode_names[5].to_owned();
    corruptions.push(invalid);
    let mut invalid = exact_output();
    invalid.asserted_type_input.spelling = "OtherLongMode".to_owned();
    corruptions.push(invalid);
    let mut invalid = exact_output();
    invalid.asserted_type_input.head = invalid.subject_result_input.head.clone();
    corruptions.push(invalid);
    let mut invalid = exact_output();
    invalid.asserted_type_input.head = TypeHeadInput::BuiltinSet;
    corruptions.push(invalid);
    let mut invalid = exact_output();
    invalid.subject_result_input.head = TypeHeadInput::BuiltinSet;
    corruptions.push(invalid);
    let mut invalid = exact_output();
    invalid.subject_result_input.source_range = invalid.asserted_type_input.source_range;
    corruptions.push(invalid);
    let mut invalid = exact_output();
    invalid.subject_result_input.site = invalid.asserted_type_input.site.clone();
    corruptions.push(invalid);
    let mut invalid = exact_output();
    invalid.asserted_type_input.source_range = invalid.subject_result_input.source_range;
    corruptions.push(invalid);
    let mut invalid = exact_output();
    invalid.asserted_type_input.site = invalid.subject_result_input.site.clone();
    corruptions.push(invalid);
    let mut invalid = exact_output();
    let binding_head = invalid.subject_result_input.head.clone();
    let (_, binding_expansion) = invalid
        .payload
        .reserve
        .mode_expansions
        .iter_mut()
        .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[6]))
        .unwrap();
    binding_expansion.radix.head = binding_head;
    corruptions.push(invalid);
    for mode_name in &mode_names[1..] {
        let mut invalid = exact_output();
        let (_, expansion) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(*mode_name))
            .unwrap();
        expansion.radix.spelling = "set".to_owned();
        expansion.radix.head = TypeHeadInput::BuiltinSet;
        corruptions.push(invalid);
    }
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
    let mut invalid = exact_output();
    let (_, base_expansion) = invalid
        .payload
        .reserve
        .mode_expansions
        .iter_mut()
        .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[0]))
        .unwrap();
    base_expansion.radix.spelling = "set".to_owned();
    base_expansion.radix.head = TypeHeadInput::BuiltinSet;
    corruptions.push(invalid);
    for invalid in corruptions {
        assert_invalid_output(invalid);
    }
    assert_source_reserved_variable_type_assertion_output(&exact_output())
        .expect("corrupting cloned Task 231 outputs must not mutate the exact output");

    let assert_extraction_gap = |ast, context: &str| {
        assert_eq!(
            source_type_elaboration_detail_keys(&ast, module.clone(), &symbols),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()],
            "{context}"
        );
    };
    fn next_permutation(order: &mut [usize; 7]) -> bool {
        let Some(pivot) = (0..order.len() - 1)
            .rev()
            .find(|&index| order[index] < order[index + 1])
        else {
            return false;
        };
        let successor = (pivot + 1..order.len())
            .rev()
            .find(|&index| order[pivot] < order[index])
            .expect("a lexicographic successor must exist after the pivot");
        order.swap(pivot, successor);
        order[pivot + 1..].reverse();
        true
    }
    let ordered = exact_modes();
    let public_dispatch_orders = [
        [1, 0, 2, 3, 4, 5, 6],
        [0, 2, 1, 3, 4, 5, 6],
        [0, 1, 3, 2, 4, 5, 6],
        [0, 1, 2, 4, 3, 5, 6],
        [0, 1, 2, 3, 5, 4, 6],
        [0, 1, 2, 3, 4, 6, 5],
        [1, 2, 3, 4, 5, 6, 0],
        [6, 5, 4, 3, 2, 1, 0],
    ];
    let mut order = [0, 1, 2, 3, 4, 5, 6];
    let mut permutation_count = 0;
    let mut public_dispatch_representative_count = 0;
    while next_permutation(&mut order) {
        permutation_count += 1;
        let permuted = mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            vec![
                ordered[order[0]],
                ordered[order[1]],
                ordered[order[2]],
                ordered[order[3]],
                ordered[order[4]],
                ordered[order[5]],
                ordered[order[6]],
            ],
            reserve(),
            theorem,
        );
        assert!(
            super::extract_source_local_object_mode_long_chain_five_hop_asserted_head(
                &permuted,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "definition permutation {order:?} must not reach Task 231"
        );
        if public_dispatch_orders.contains(&order) {
            public_dispatch_representative_count += 1;
            assert_extraction_gap(
                permuted,
                &format!("public definition permutation {order:?}"),
            );
        }
    }
    assert_eq!(permutation_count, 5_039);
    assert_eq!(public_dispatch_representative_count, 8);
    let mut mode_near_misses = vec![
        Vec::<ModeDefinitionSpec>::new(),
        {
            let mut modes = exact_modes();
            modes.push(modes[0]);
            modes
        },
        {
            let mut modes = exact_modes();
            modes.push(mode_definition_with_label(
                "UnusedLongMode",
                "UnusedLongModeDef",
                ReserveTypeShape::QualifiedSymbol(mode_names[6]),
            ));
            modes
        },
    ];
    for index in 0..mode_names.len() {
        let mut missing = exact_modes();
        missing.remove(index);
        mode_near_misses.push(missing);
        let mut duplicate = exact_modes();
        duplicate.push(duplicate[index]);
        mode_near_misses.push(duplicate);
        let mut malformed = exact_modes();
        malformed[index].rhs_shape = if index == 0 {
            ReserveTypeShape::Builtin("set")
        } else {
            ReserveTypeShape::QualifiedSymbol(mode_names[(index + 1) % mode_names.len()])
        };
        mode_near_misses.push(malformed);
        let mut wrong_label = exact_modes();
        wrong_label[index].label = Some("WrongLongModeDef");
        mode_near_misses.push(wrong_label);
        let mut wrong_spelling = exact_modes();
        wrong_spelling[index].pattern = "OtherLongMode";
        mode_near_misses.push(wrong_spelling);
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
        "ChainObjectMode7",
        "ChainObjectMode7Def",
        ReserveTypeShape::QualifiedSymbol(mode_names[6]),
    ));
    source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
        source_id,
        deeper_modes,
        vec![reserve_item(
            vec!["x"],
            ReserveTypeShape::QualifiedSymbol("ChainObjectMode7"),
        )],
        IdentifierTypeAssertionTheoremSpec {
            asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[6]),
            ..theorem
        },
    ));
    for bad_reserve in [
        ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[6]),
        ReserveTypeShape::AttributedQualifiedSymbol(mode_names[6]),
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
        vec![reserve_item(
            vec!["y"],
            ReserveTypeShape::QualifiedSymbol(mode_names[6]),
        )],
        theorem,
    ));
    source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
        source_id,
        exact_modes(),
        vec![
            reserve_item(vec!["x"], ReserveTypeShape::QualifiedSymbol(mode_names[6])),
            reserve_item(vec!["y"], ReserveTypeShape::Builtin("object")),
        ],
        theorem,
    ));
    for bad_reserve in [
        vec![reserve_item(
            vec!["x"],
            ReserveTypeShape::QualifiedSymbol(mode_names[5]),
        )],
        vec![reserve_item(
            vec!["x"],
            ReserveTypeShape::QualifiedSymbol("OtherLongMode"),
        )],
        vec![reserve_item(
            vec!["x", "y"],
            ReserveTypeShape::QualifiedSymbol(mode_names[6]),
        )],
    ] {
        source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            exact_modes(),
            bad_reserve,
            theorem,
        ));
    }
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
            asserted_type: ReserveTypeShape::Builtin("set"),
            ..theorem
        },
        IdentifierTypeAssertionTheoremSpec {
            asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[0]),
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
            asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[4]),
            ..theorem
        },
        IdentifierTypeAssertionTheoremSpec {
            asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[5]),
            ..theorem
        },
        IdentifierTypeAssertionTheoremSpec {
            asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[6]),
            ..theorem
        },
        IdentifierTypeAssertionTheoremSpec {
            asserted_type: ReserveTypeShape::QualifiedSymbol("OtherLongMode"),
            ..theorem
        },
        IdentifierTypeAssertionTheoremSpec {
            asserted_type: ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[1]),
            ..theorem
        },
        IdentifierTypeAssertionTheoremSpec {
            asserted_type: ReserveTypeShape::AttributedQualifiedSymbol(mode_names[1]),
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
        let local_object_modes = mode_names
            .iter()
            .enumerate()
            .filter(|(index, _)| *index != imported_index)
            .map(|(_, spelling)| (*spelling, SymbolKind::Mode))
            .collect::<Vec<_>>();
        for imported in [
            vec![(mode_names[imported_index], SymbolKind::Mode)],
            vec![
                (mode_names[imported_index], SymbolKind::Mode),
                (mode_names[imported_index], SymbolKind::Mode),
            ],
        ] {
            let imported_symbols = source_local_and_imported_symbols_env(
                module.clone(),
                &local_object_modes,
                imported.as_slice(),
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&exact, module.clone(), &imported_symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
    }
    let unrelated_import = source_local_and_imported_symbols_env(
        module.clone(),
        &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
        &[("UnrelatedLongMode", SymbolKind::Mode)],
    );
    assert!(
        super::extract_source_local_object_mode_long_chain_five_hop_asserted_head(
            &exact,
            module.clone(),
            &unrelated_import,
        )
        .is_some()
    );
    for (asserted_spelling, asserted_head_symbols) in [
        (
            "ImportedLongMode",
            source_local_and_imported_symbols_env(
                module.clone(),
                &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
                &[("ImportedLongMode", SymbolKind::Mode)],
            ),
        ),
        (
            "AmbiguousLongMode",
            source_local_and_imported_symbols_env(
                module.clone(),
                &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
                &[
                    ("AmbiguousLongMode", SymbolKind::Mode),
                    ("AmbiguousLongMode", SymbolKind::Mode),
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
fn source_local_object_mode_long_chain_four_hop_asserted_head_consumes_seven_expansions() {
    let source_id = source_id(229);
    let module = ResolverModuleId::new(
        PackageId::new("test"),
        ModulePath::new("local_object_mode_long_chain_four_hop_asserted_head"),
    );
    let mode_names = [
        "BaseObjectMode",
        "ChainObjectMode1",
        "ChainObjectMode2",
        "ChainObjectMode3",
        "ChainObjectMode4",
        "ChainObjectMode5",
        "ChainObjectMode6",
    ];
    let mode_labels = [
        "BaseObjectModeDef",
        "ChainObjectMode1Def",
        "ChainObjectMode2Def",
        "ChainObjectMode3Def",
        "ChainObjectMode4Def",
        "ChainObjectMode5Def",
        "ChainObjectMode6Def",
    ];
    let symbols = source_local_symbols_env(
        module.clone(),
        &[
            (mode_names[0], SymbolKind::Mode),
            (mode_names[1], SymbolKind::Mode),
            (mode_names[2], SymbolKind::Mode),
            (mode_names[3], SymbolKind::Mode),
            (mode_names[4], SymbolKind::Mode),
            (mode_names[5], SymbolKind::Mode),
            (mode_names[6], SymbolKind::Mode),
            ("ChainObjectMode7", SymbolKind::Mode),
            ("OtherLongObjectMode", SymbolKind::Mode),
        ],
    );
    let theorem = IdentifierTypeAssertionTheoremSpec {
        status: None,
        label: "LongLocalObjectModeFourHopAssertedHeadPayloadBoundary",
        subject: "x",
        asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[2]),
        negated: false,
        recovered_label: false,
    };
    let exact_modes = || {
        vec![
            mode_definition_with_label(
                mode_names[0],
                mode_labels[0],
                ReserveTypeShape::Builtin("object"),
            ),
            mode_definition_with_label(
                mode_names[1],
                mode_labels[1],
                ReserveTypeShape::QualifiedSymbol(mode_names[0]),
            ),
            mode_definition_with_label(
                mode_names[2],
                mode_labels[2],
                ReserveTypeShape::QualifiedSymbol(mode_names[1]),
            ),
            mode_definition_with_label(
                mode_names[3],
                mode_labels[3],
                ReserveTypeShape::QualifiedSymbol(mode_names[2]),
            ),
            mode_definition_with_label(
                mode_names[4],
                mode_labels[4],
                ReserveTypeShape::QualifiedSymbol(mode_names[3]),
            ),
            mode_definition_with_label(
                mode_names[5],
                mode_labels[5],
                ReserveTypeShape::QualifiedSymbol(mode_names[4]),
            ),
            mode_definition_with_label(
                mode_names[6],
                mode_labels[6],
                ReserveTypeShape::QualifiedSymbol(mode_names[5]),
            ),
        ]
    };
    let reserve = || {
        vec![reserve_item(
            vec!["x"],
            ReserveTypeShape::QualifiedSymbol(mode_names[6]),
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
        extract_source_local_mode_long_chain_reserved_variable_type_assertion(
            &exact,
            module.clone(),
            &symbols,
        )
        .is_none(),
        "Task 175 must reject the Task 229 four-hop source"
    );
    assert!(
        super::extract_source_local_mode_long_chain_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .is_none(),
        "Task 199 must reject the Task 229 four-hop source"
    );
    assert!(
        super::extract_source_local_mode_long_chain_two_hop_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .is_none(),
        "Task 224 must reject the Task 229 four-hop source"
    );
    assert!(
        super::extract_source_local_object_mode_long_chain_two_hop_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .is_none(),
        "Task 225 must reject the Task 229 object-terminal four-hop source"
    );
    assert!(
        super::extract_source_local_mode_long_chain_three_hop_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .is_none(),
        "Task 226 must reject the Task 229 four-hop source"
    );
    assert!(
        super::extract_source_local_object_mode_long_chain_three_hop_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .is_none(),
        "Task 227 must reject the Task 229 object-terminal four-hop source"
    );
    assert!(
        super::extract_source_local_mode_long_chain_four_hop_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .is_none(),
        "Task 228 must reject the Task 229 object-terminal four-hop source"
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
    ];
    assert_eq!(preexisting_owner_results.len(), 34);
    assert!(
        preexisting_owner_results
            .into_iter()
            .all(|result| result.is_none()),
        "all 34 legacy type-assertion owner routes must reject Task 229"
    );
    let payload = super::extract_source_local_object_mode_long_chain_four_hop_asserted_head(
        &exact,
        module.clone(),
        &symbols,
    )
    .expect("exact seven-expansion four-hop source should extract");
    assert_eq!(payload.reserve.mode_expansions.len(), 7);
    assert_eq!(payload.reserve.bridge.bindings().len(), 1);
    assert_eq!(payload.subject_lookup_ordinal, 1);
    let source_binding = &payload.reserve.bridge.bindings()[0];
    assert_eq!(source_binding.type_spelling, mode_names[6]);
    assert_eq!(payload.asserted_type.spelling, mode_names[2]);
    assert_ne!(payload.asserted_type.head, source_binding.type_head);
    assert!(matches!(source_binding.type_head, TypeHeadInput::Symbol(_)));
    assert_ne!(payload.asserted_type_site, payload.subject_site);
    assert_ne!(payload.asserted_type.range, source_binding.type_range);
    let binding_expansion = payload
        .reserve
        .mode_expansions
        .iter()
        .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[6]))
        .map(|(_, expansion)| expansion)
        .expect("ChainObjectMode6 binding expansion should exist");
    assert_eq!(binding_expansion.radix.spelling, mode_names[5]);
    assert_ne!(binding_expansion.radix.head, payload.asserted_type.head);
    let TypeHeadInput::Symbol(first_intermediate_symbol) = &binding_expansion.radix.head else {
        panic!("ChainObjectMode5 first relation intermediate should resolve to a symbol");
    };
    let first_intermediate_expansion = payload
        .reserve
        .mode_expansions
        .get(first_intermediate_symbol)
        .expect("ChainObjectMode5 first intermediate expansion should exist");
    assert_eq!(first_intermediate_expansion.radix.spelling, mode_names[4]);
    assert_ne!(
        first_intermediate_expansion.radix.head,
        payload.asserted_type.head
    );
    let TypeHeadInput::Symbol(second_intermediate_symbol) =
        &first_intermediate_expansion.radix.head
    else {
        panic!("ChainObjectMode4 second relation intermediate should resolve to a symbol");
    };
    let second_intermediate_expansion = payload
        .reserve
        .mode_expansions
        .get(second_intermediate_symbol)
        .expect("ChainObjectMode4 second intermediate expansion should exist");
    assert_eq!(second_intermediate_expansion.radix.spelling, mode_names[3]);
    assert_ne!(
        second_intermediate_expansion.radix.head,
        payload.asserted_type.head
    );
    let TypeHeadInput::Symbol(third_intermediate_symbol) =
        &second_intermediate_expansion.radix.head
    else {
        panic!("ChainObjectMode3 third relation intermediate should resolve to a symbol");
    };
    let third_intermediate_expansion = payload
        .reserve
        .mode_expansions
        .get(third_intermediate_symbol)
        .expect("ChainObjectMode3 third intermediate expansion should exist");
    assert_eq!(third_intermediate_expansion.radix.spelling, mode_names[2]);
    assert_eq!(
        third_intermediate_expansion.radix.head,
        payload.asserted_type.head
    );

    let output = super::source_local_object_mode_long_chain_four_hop_asserted_head_output(
        &exact,
        module.clone(),
        &symbols,
    )
    .expect("exact seven-expansion asserted-head source should reach checker");
    assert_source_reserved_variable_type_assertion_output(&output)
        .expect("Task 229 checker payload invariants should hold");
    assert_eq!(output.subject_binding, BindingId::new(0));
    assert_eq!(output.payload.subject_lookup_ordinal, 1);
    assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
    assert_eq!(output.term_formula.type_entries().len(), 3);
    assert_eq!(output.term_formula.normalized_types().len(), 1);
    assert_eq!(output.subject_result_input.spelling, mode_names[6]);
    assert_eq!(output.asserted_type_input.spelling, mode_names[2]);
    assert_ne!(
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
        .expect("BaseObjectMode terminal expansion should exist");
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
                super::TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_FOUR_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
            ),
            vec![
                super::TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_FOUR_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY
                    .to_owned()
            ]
        );
    };
    for removed in mode_names {
        let mut invalid =
            super::source_local_object_mode_long_chain_four_hop_asserted_head_output(
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
        super::source_local_object_mode_long_chain_four_hop_asserted_head_output(
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
    invalid.subject_result_input.spelling = mode_names[5].to_owned();
    corruptions.push(invalid);
    let mut invalid = exact_output();
    invalid.asserted_type_input.spelling = "OtherLongObjectMode".to_owned();
    corruptions.push(invalid);
    let mut invalid = exact_output();
    invalid.asserted_type_input.head = invalid.subject_result_input.head.clone();
    corruptions.push(invalid);
    let mut invalid = exact_output();
    invalid.asserted_type_input.head = TypeHeadInput::BuiltinSet;
    corruptions.push(invalid);
    let mut invalid = exact_output();
    invalid.subject_result_input.head = TypeHeadInput::BuiltinSet;
    corruptions.push(invalid);
    let mut invalid = exact_output();
    invalid.subject_result_input.source_range = invalid.asserted_type_input.source_range;
    corruptions.push(invalid);
    let mut invalid = exact_output();
    invalid.subject_result_input.site = invalid.asserted_type_input.site.clone();
    corruptions.push(invalid);
    let mut invalid = exact_output();
    invalid.asserted_type_input.source_range = invalid.subject_result_input.source_range;
    corruptions.push(invalid);
    let mut invalid = exact_output();
    invalid.asserted_type_input.site = invalid.subject_result_input.site.clone();
    corruptions.push(invalid);
    let mut invalid = exact_output();
    let binding_head = invalid.subject_result_input.head.clone();
    let (_, binding_expansion) = invalid
        .payload
        .reserve
        .mode_expansions
        .iter_mut()
        .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[6]))
        .unwrap();
    binding_expansion.radix.head = binding_head;
    corruptions.push(invalid);
    for mode_name in &mode_names[1..] {
        let mut invalid = exact_output();
        let (_, expansion) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(*mode_name))
            .unwrap();
        expansion.radix.spelling = "set".to_owned();
        expansion.radix.head = TypeHeadInput::BuiltinSet;
        corruptions.push(invalid);
    }
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
    let mut invalid = exact_output();
    let (_, base_expansion) = invalid
        .payload
        .reserve
        .mode_expansions
        .iter_mut()
        .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[0]))
        .unwrap();
    base_expansion.radix.spelling = "set".to_owned();
    base_expansion.radix.head = TypeHeadInput::BuiltinSet;
    corruptions.push(invalid);
    for invalid in corruptions {
        assert_invalid_output(invalid);
    }
    assert_source_reserved_variable_type_assertion_output(&exact_output())
        .expect("corrupting cloned Task 229 outputs must not mutate the exact output");

    let assert_extraction_gap = |ast, context: &str| {
        assert_eq!(
            source_type_elaboration_detail_keys(&ast, module.clone(), &symbols),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()],
            "{context}"
        );
    };
    fn next_permutation(order: &mut [usize; 7]) -> bool {
        let Some(pivot) = (0..order.len() - 1)
            .rev()
            .find(|&index| order[index] < order[index + 1])
        else {
            return false;
        };
        let successor = (pivot + 1..order.len())
            .rev()
            .find(|&index| order[pivot] < order[index])
            .expect("a lexicographic successor must exist after the pivot");
        order.swap(pivot, successor);
        order[pivot + 1..].reverse();
        true
    }
    let ordered = exact_modes();
    let public_dispatch_orders = [
        [1, 0, 2, 3, 4, 5, 6],
        [0, 2, 1, 3, 4, 5, 6],
        [0, 1, 3, 2, 4, 5, 6],
        [0, 1, 2, 4, 3, 5, 6],
        [0, 1, 2, 3, 5, 4, 6],
        [0, 1, 2, 3, 4, 6, 5],
        [1, 2, 3, 4, 5, 6, 0],
        [6, 5, 4, 3, 2, 1, 0],
    ];
    let mut order = [0, 1, 2, 3, 4, 5, 6];
    let mut permutation_count = 0;
    let mut public_dispatch_representative_count = 0;
    while next_permutation(&mut order) {
        permutation_count += 1;
        let permuted = mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            vec![
                ordered[order[0]],
                ordered[order[1]],
                ordered[order[2]],
                ordered[order[3]],
                ordered[order[4]],
                ordered[order[5]],
                ordered[order[6]],
            ],
            reserve(),
            theorem,
        );
        assert!(
            super::extract_source_local_object_mode_long_chain_four_hop_asserted_head(
                &permuted,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "definition permutation {order:?} must not reach Task 229"
        );
        if public_dispatch_orders.contains(&order) {
            public_dispatch_representative_count += 1;
            assert_extraction_gap(
                permuted,
                &format!("public definition permutation {order:?}"),
            );
        }
    }
    assert_eq!(permutation_count, 5_039);
    assert_eq!(public_dispatch_representative_count, 8);
    let mut mode_near_misses = vec![
        Vec::<ModeDefinitionSpec>::new(),
        {
            let mut modes = exact_modes();
            modes.push(modes[0]);
            modes
        },
        {
            let mut modes = exact_modes();
            modes.push(mode_definition_with_label(
                "UnusedLongObjectMode",
                "UnusedLongObjectModeDef",
                ReserveTypeShape::QualifiedSymbol(mode_names[6]),
            ));
            modes
        },
    ];
    for index in 0..mode_names.len() {
        let mut missing = exact_modes();
        missing.remove(index);
        mode_near_misses.push(missing);
        let mut duplicate = exact_modes();
        duplicate.push(duplicate[index]);
        mode_near_misses.push(duplicate);
        let mut malformed = exact_modes();
        malformed[index].rhs_shape = if index == 0 {
            ReserveTypeShape::Builtin("set")
        } else {
            ReserveTypeShape::QualifiedSymbol(mode_names[(index + 1) % mode_names.len()])
        };
        mode_near_misses.push(malformed);
        let mut wrong_label = exact_modes();
        wrong_label[index].label = Some("WrongLongObjectModeDef");
        mode_near_misses.push(wrong_label);
        let mut wrong_spelling = exact_modes();
        wrong_spelling[index].pattern = "OtherLongObjectMode";
        mode_near_misses.push(wrong_spelling);
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
        "ChainObjectMode7",
        "ChainObjectMode7Def",
        ReserveTypeShape::QualifiedSymbol(mode_names[6]),
    ));
    source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
        source_id,
        deeper_modes,
        vec![reserve_item(
            vec!["x"],
            ReserveTypeShape::QualifiedSymbol("ChainObjectMode7"),
        )],
        IdentifierTypeAssertionTheoremSpec {
            asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[6]),
            ..theorem
        },
    ));
    for bad_reserve in [
        ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[6]),
        ReserveTypeShape::AttributedQualifiedSymbol(mode_names[6]),
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
        vec![reserve_item(
            vec!["y"],
            ReserveTypeShape::QualifiedSymbol(mode_names[6]),
        )],
        theorem,
    ));
    source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
        source_id,
        exact_modes(),
        vec![
            reserve_item(vec!["x"], ReserveTypeShape::QualifiedSymbol(mode_names[6])),
            reserve_item(vec!["y"], ReserveTypeShape::Builtin("object")),
        ],
        theorem,
    ));
    for bad_reserve in [
        vec![reserve_item(
            vec!["x"],
            ReserveTypeShape::QualifiedSymbol(mode_names[5]),
        )],
        vec![reserve_item(
            vec!["x"],
            ReserveTypeShape::QualifiedSymbol("OtherLongObjectMode"),
        )],
        vec![reserve_item(
            vec!["x", "y"],
            ReserveTypeShape::QualifiedSymbol(mode_names[6]),
        )],
    ] {
        source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            exact_modes(),
            bad_reserve,
            theorem,
        ));
    }
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
            asserted_type: ReserveTypeShape::Builtin("set"),
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
            asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[3]),
            ..theorem
        },
        IdentifierTypeAssertionTheoremSpec {
            asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[4]),
            ..theorem
        },
        IdentifierTypeAssertionTheoremSpec {
            asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[5]),
            ..theorem
        },
        IdentifierTypeAssertionTheoremSpec {
            asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[6]),
            ..theorem
        },
        IdentifierTypeAssertionTheoremSpec {
            asserted_type: ReserveTypeShape::QualifiedSymbol("OtherLongObjectMode"),
            ..theorem
        },
        IdentifierTypeAssertionTheoremSpec {
            asserted_type: ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[2]),
            ..theorem
        },
        IdentifierTypeAssertionTheoremSpec {
            asserted_type: ReserveTypeShape::AttributedQualifiedSymbol(mode_names[2]),
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
        for imported in [
            vec![(mode_names[imported_index], SymbolKind::Mode)],
            vec![
                (mode_names[imported_index], SymbolKind::Mode),
                (mode_names[imported_index], SymbolKind::Mode),
            ],
        ] {
            let imported_symbols = source_local_and_imported_symbols_env(
                module.clone(),
                &local_modes,
                imported.as_slice(),
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&exact, module.clone(), &imported_symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
    }
    let unrelated_import = source_local_and_imported_symbols_env(
        module.clone(),
        &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
        &[("UnrelatedLongObjectMode", SymbolKind::Mode)],
    );
    assert!(
        super::extract_source_local_object_mode_long_chain_four_hop_asserted_head(
            &exact,
            module.clone(),
            &unrelated_import,
        )
        .is_some()
    );
    for (asserted_spelling, asserted_head_symbols) in [
        (
            "ImportedLongObjectMode",
            source_local_and_imported_symbols_env(
                module.clone(),
                &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
                &[("ImportedLongObjectMode", SymbolKind::Mode)],
            ),
        ),
        (
            "AmbiguousLongObjectMode",
            source_local_and_imported_symbols_env(
                module.clone(),
                &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
                &[
                    ("AmbiguousLongObjectMode", SymbolKind::Mode),
                    ("AmbiguousLongObjectMode", SymbolKind::Mode),
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
fn source_local_object_mode_long_chain_three_hop_asserted_head_consumes_seven_expansions() {
    let source_id = source_id(227);
    let module = ResolverModuleId::new(
        PackageId::new("test"),
        ModulePath::new("local_object_mode_long_chain_three_hop_asserted_head"),
    );
    let mode_names = [
        "BaseObjectMode",
        "ChainObjectMode1",
        "ChainObjectMode2",
        "ChainObjectMode3",
        "ChainObjectMode4",
        "ChainObjectMode5",
        "ChainObjectMode6",
    ];
    let mode_labels = [
        "BaseObjectModeDef",
        "ChainObjectMode1Def",
        "ChainObjectMode2Def",
        "ChainObjectMode3Def",
        "ChainObjectMode4Def",
        "ChainObjectMode5Def",
        "ChainObjectMode6Def",
    ];
    let symbols = source_local_symbols_env(
        module.clone(),
        &[
            (mode_names[0], SymbolKind::Mode),
            (mode_names[1], SymbolKind::Mode),
            (mode_names[2], SymbolKind::Mode),
            (mode_names[3], SymbolKind::Mode),
            (mode_names[4], SymbolKind::Mode),
            (mode_names[5], SymbolKind::Mode),
            (mode_names[6], SymbolKind::Mode),
            ("ChainObjectMode7", SymbolKind::Mode),
            ("OtherLongObjectMode", SymbolKind::Mode),
        ],
    );
    let theorem = IdentifierTypeAssertionTheoremSpec {
        status: None,
        label: "LongLocalObjectModeThreeHopAssertedHeadPayloadBoundary",
        subject: "x",
        asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[3]),
        negated: false,
        recovered_label: false,
    };
    let exact_modes = || {
        vec![
            mode_definition_with_label(
                mode_names[0],
                mode_labels[0],
                ReserveTypeShape::Builtin("object"),
            ),
            mode_definition_with_label(
                mode_names[1],
                mode_labels[1],
                ReserveTypeShape::QualifiedSymbol(mode_names[0]),
            ),
            mode_definition_with_label(
                mode_names[2],
                mode_labels[2],
                ReserveTypeShape::QualifiedSymbol(mode_names[1]),
            ),
            mode_definition_with_label(
                mode_names[3],
                mode_labels[3],
                ReserveTypeShape::QualifiedSymbol(mode_names[2]),
            ),
            mode_definition_with_label(
                mode_names[4],
                mode_labels[4],
                ReserveTypeShape::QualifiedSymbol(mode_names[3]),
            ),
            mode_definition_with_label(
                mode_names[5],
                mode_labels[5],
                ReserveTypeShape::QualifiedSymbol(mode_names[4]),
            ),
            mode_definition_with_label(
                mode_names[6],
                mode_labels[6],
                ReserveTypeShape::QualifiedSymbol(mode_names[5]),
            ),
        ]
    };
    let reserve = || {
        vec![reserve_item(
            vec!["x"],
            ReserveTypeShape::QualifiedSymbol(mode_names[6]),
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
        extract_source_local_mode_long_chain_reserved_variable_type_assertion(
            &exact,
            module.clone(),
            &symbols,
        )
        .is_none(),
        "Task 175 must reject the Task 227 three-hop source"
    );
    assert!(
        super::extract_source_local_mode_long_chain_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .is_none(),
        "Task 199 must reject the Task 227 three-hop source"
    );
    assert!(
        super::extract_source_local_mode_long_chain_two_hop_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .is_none(),
        "Task 224 must reject the Task 227 three-hop source"
    );
    assert!(
        super::extract_source_local_object_mode_long_chain_two_hop_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .is_none(),
        "Task 225 must reject the Task 227 object-terminal three-hop source"
    );
    assert!(
        super::extract_source_local_mode_long_chain_three_hop_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .is_none(),
        "Task 226 must reject the Task 227 object-terminal three-hop source"
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
    ];
    assert_eq!(preexisting_owner_results.len(), 34);
    assert!(
        preexisting_owner_results
            .into_iter()
            .all(|result| result.is_none()),
        "all 34 legacy type-assertion owner routes must reject Task 227"
    );
    let payload = super::extract_source_local_object_mode_long_chain_three_hop_asserted_head(
        &exact,
        module.clone(),
        &symbols,
    )
    .expect("exact seven-expansion three-hop source should extract");
    assert_eq!(payload.reserve.mode_expansions.len(), 7);
    assert_eq!(payload.reserve.bridge.bindings().len(), 1);
    assert_eq!(payload.subject_lookup_ordinal, 1);
    let source_binding = &payload.reserve.bridge.bindings()[0];
    assert_eq!(source_binding.type_spelling, mode_names[6]);
    assert_eq!(payload.asserted_type.spelling, mode_names[3]);
    assert_ne!(payload.asserted_type.head, source_binding.type_head);
    assert!(matches!(source_binding.type_head, TypeHeadInput::Symbol(_)));
    assert_ne!(payload.asserted_type_site, payload.subject_site);
    assert_ne!(payload.asserted_type.range, source_binding.type_range);
    let binding_expansion = payload
        .reserve
        .mode_expansions
        .iter()
        .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[6]))
        .map(|(_, expansion)| expansion)
        .expect("ChainObjectMode6 binding expansion should exist");
    assert_eq!(binding_expansion.radix.spelling, mode_names[5]);
    assert_ne!(binding_expansion.radix.head, payload.asserted_type.head);
    let TypeHeadInput::Symbol(first_intermediate_symbol) = &binding_expansion.radix.head else {
        panic!("ChainObjectMode5 first relation intermediate should resolve to a symbol");
    };
    let first_intermediate_expansion = payload
        .reserve
        .mode_expansions
        .get(first_intermediate_symbol)
        .expect("ChainObjectMode5 first intermediate expansion should exist");
    assert_eq!(first_intermediate_expansion.radix.spelling, mode_names[4]);
    assert_ne!(
        first_intermediate_expansion.radix.head,
        payload.asserted_type.head
    );
    let TypeHeadInput::Symbol(second_intermediate_symbol) =
        &first_intermediate_expansion.radix.head
    else {
        panic!("ChainObjectMode4 second relation intermediate should resolve to a symbol");
    };
    let second_intermediate_expansion = payload
        .reserve
        .mode_expansions
        .get(second_intermediate_symbol)
        .expect("ChainObjectMode4 second intermediate expansion should exist");
    assert_eq!(second_intermediate_expansion.radix.spelling, mode_names[3]);
    assert_eq!(
        second_intermediate_expansion.radix.head,
        payload.asserted_type.head
    );

    let output = super::source_local_object_mode_long_chain_three_hop_asserted_head_output(
        &exact,
        module.clone(),
        &symbols,
    )
    .expect("exact seven-expansion asserted-head source should reach checker");
    assert_source_reserved_variable_type_assertion_output(&output)
        .expect("Task 227 checker payload invariants should hold");
    assert_eq!(output.subject_binding, BindingId::new(0));
    assert_eq!(output.payload.subject_lookup_ordinal, 1);
    assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
    assert_eq!(output.term_formula.type_entries().len(), 3);
    assert_eq!(output.term_formula.normalized_types().len(), 1);
    assert_eq!(output.subject_result_input.spelling, mode_names[6]);
    assert_eq!(output.asserted_type_input.spelling, mode_names[3]);
    assert_ne!(
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
        .expect("BaseObjectMode terminal expansion should exist");
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
                super::TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_THREE_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
            ),
            vec![
                super::TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_THREE_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY
                    .to_owned()
            ]
        );
    };
    for removed in mode_names {
        let mut invalid =
            super::source_local_object_mode_long_chain_three_hop_asserted_head_output(
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
        super::source_local_object_mode_long_chain_three_hop_asserted_head_output(
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
    invalid.subject_result_input.spelling = mode_names[5].to_owned();
    corruptions.push(invalid);
    let mut invalid = exact_output();
    invalid.asserted_type_input.spelling = "OtherLongObjectMode".to_owned();
    corruptions.push(invalid);
    let mut invalid = exact_output();
    invalid.asserted_type_input.head = invalid.subject_result_input.head.clone();
    corruptions.push(invalid);
    let mut invalid = exact_output();
    invalid.asserted_type_input.head = TypeHeadInput::BuiltinSet;
    corruptions.push(invalid);
    let mut invalid = exact_output();
    invalid.subject_result_input.head = TypeHeadInput::BuiltinSet;
    corruptions.push(invalid);
    let mut invalid = exact_output();
    invalid.subject_result_input.source_range = invalid.asserted_type_input.source_range;
    corruptions.push(invalid);
    let mut invalid = exact_output();
    invalid.subject_result_input.site = invalid.asserted_type_input.site.clone();
    corruptions.push(invalid);
    let mut invalid = exact_output();
    invalid.asserted_type_input.source_range = invalid.subject_result_input.source_range;
    corruptions.push(invalid);
    let mut invalid = exact_output();
    invalid.asserted_type_input.site = invalid.subject_result_input.site.clone();
    corruptions.push(invalid);
    let mut invalid = exact_output();
    let binding_head = invalid.subject_result_input.head.clone();
    let (_, binding_expansion) = invalid
        .payload
        .reserve
        .mode_expansions
        .iter_mut()
        .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[6]))
        .unwrap();
    binding_expansion.radix.head = binding_head;
    corruptions.push(invalid);
    for mode_name in &mode_names[1..] {
        let mut invalid = exact_output();
        let (_, expansion) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(*mode_name))
            .unwrap();
        expansion.radix.spelling = "set".to_owned();
        expansion.radix.head = TypeHeadInput::BuiltinSet;
        corruptions.push(invalid);
    }
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
    let mut invalid = exact_output();
    let (_, base_expansion) = invalid
        .payload
        .reserve
        .mode_expansions
        .iter_mut()
        .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[0]))
        .unwrap();
    base_expansion.radix.spelling = "set".to_owned();
    base_expansion.radix.head = TypeHeadInput::BuiltinSet;
    corruptions.push(invalid);
    for invalid in corruptions {
        assert_invalid_output(invalid);
    }
    assert_source_reserved_variable_type_assertion_output(&exact_output())
        .expect("corrupting cloned Task 227 outputs must not mutate the exact output");

    let assert_extraction_gap = |ast, context: &str| {
        assert_eq!(
            source_type_elaboration_detail_keys(&ast, module.clone(), &symbols),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()],
            "{context}"
        );
    };
    fn next_permutation(order: &mut [usize; 7]) -> bool {
        let Some(pivot) = (0..order.len() - 1)
            .rev()
            .find(|&index| order[index] < order[index + 1])
        else {
            return false;
        };
        let successor = (pivot + 1..order.len())
            .rev()
            .find(|&index| order[pivot] < order[index])
            .expect("a lexicographic successor must exist after the pivot");
        order.swap(pivot, successor);
        order[pivot + 1..].reverse();
        true
    }
    let ordered = exact_modes();
    let public_dispatch_orders = [
        [1, 0, 2, 3, 4, 5, 6],
        [0, 2, 1, 3, 4, 5, 6],
        [0, 1, 3, 2, 4, 5, 6],
        [0, 1, 2, 4, 3, 5, 6],
        [0, 1, 2, 3, 5, 4, 6],
        [0, 1, 2, 3, 4, 6, 5],
        [1, 2, 3, 4, 5, 6, 0],
        [6, 5, 4, 3, 2, 1, 0],
    ];
    let mut order = [0, 1, 2, 3, 4, 5, 6];
    let mut permutation_count = 0;
    let mut public_dispatch_representative_count = 0;
    while next_permutation(&mut order) {
        permutation_count += 1;
        let permuted = mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            vec![
                ordered[order[0]],
                ordered[order[1]],
                ordered[order[2]],
                ordered[order[3]],
                ordered[order[4]],
                ordered[order[5]],
                ordered[order[6]],
            ],
            reserve(),
            theorem,
        );
        assert!(
            super::extract_source_local_object_mode_long_chain_three_hop_asserted_head(
                &permuted,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "definition permutation {order:?} must not reach Task 227"
        );
        if public_dispatch_orders.contains(&order) {
            public_dispatch_representative_count += 1;
            assert_extraction_gap(
                permuted,
                &format!("public definition permutation {order:?}"),
            );
        }
    }
    assert_eq!(permutation_count, 5_039);
    assert_eq!(public_dispatch_representative_count, 8);
    let mut mode_near_misses = vec![
        Vec::<ModeDefinitionSpec>::new(),
        {
            let mut modes = exact_modes();
            modes.push(modes[0]);
            modes
        },
        {
            let mut modes = exact_modes();
            modes.push(mode_definition_with_label(
                "UnusedLongObjectMode",
                "UnusedLongObjectModeDef",
                ReserveTypeShape::QualifiedSymbol(mode_names[6]),
            ));
            modes
        },
    ];
    for index in 0..mode_names.len() {
        let mut missing = exact_modes();
        missing.remove(index);
        mode_near_misses.push(missing);
        let mut duplicate = exact_modes();
        duplicate.push(duplicate[index]);
        mode_near_misses.push(duplicate);
        let mut malformed = exact_modes();
        malformed[index].rhs_shape = if index == 0 {
            ReserveTypeShape::Builtin("set")
        } else {
            ReserveTypeShape::QualifiedSymbol(mode_names[(index + 1) % mode_names.len()])
        };
        mode_near_misses.push(malformed);
        let mut wrong_label = exact_modes();
        wrong_label[index].label = Some("WrongLongModeDef");
        mode_near_misses.push(wrong_label);
        let mut wrong_spelling = exact_modes();
        wrong_spelling[index].pattern = "OtherLongObjectMode";
        mode_near_misses.push(wrong_spelling);
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
        "ChainObjectMode7",
        "ChainObjectMode7Def",
        ReserveTypeShape::QualifiedSymbol(mode_names[6]),
    ));
    source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
        source_id,
        deeper_modes,
        vec![reserve_item(
            vec!["x"],
            ReserveTypeShape::QualifiedSymbol("ChainObjectMode7"),
        )],
        IdentifierTypeAssertionTheoremSpec {
            asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[6]),
            ..theorem
        },
    ));
    for bad_reserve in [
        ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[6]),
        ReserveTypeShape::AttributedQualifiedSymbol(mode_names[6]),
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
        vec![reserve_item(
            vec!["y"],
            ReserveTypeShape::QualifiedSymbol(mode_names[6]),
        )],
        theorem,
    ));
    source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
        source_id,
        exact_modes(),
        vec![
            reserve_item(vec!["x"], ReserveTypeShape::QualifiedSymbol(mode_names[6])),
            reserve_item(vec!["y"], ReserveTypeShape::Builtin("object")),
        ],
        theorem,
    ));
    for bad_reserve in [
        vec![reserve_item(
            vec!["x"],
            ReserveTypeShape::QualifiedSymbol(mode_names[5]),
        )],
        vec![reserve_item(
            vec!["x"],
            ReserveTypeShape::QualifiedSymbol("OtherLongObjectMode"),
        )],
        vec![reserve_item(
            vec!["x", "y"],
            ReserveTypeShape::QualifiedSymbol(mode_names[6]),
        )],
    ] {
        source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            exact_modes(),
            bad_reserve,
            theorem,
        ));
    }
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
            asserted_type: ReserveTypeShape::Builtin("set"),
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
            asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[4]),
            ..theorem
        },
        IdentifierTypeAssertionTheoremSpec {
            asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[5]),
            ..theorem
        },
        IdentifierTypeAssertionTheoremSpec {
            asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[6]),
            ..theorem
        },
        IdentifierTypeAssertionTheoremSpec {
            asserted_type: ReserveTypeShape::QualifiedSymbol("OtherLongObjectMode"),
            ..theorem
        },
        IdentifierTypeAssertionTheoremSpec {
            asserted_type: ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[3]),
            ..theorem
        },
        IdentifierTypeAssertionTheoremSpec {
            asserted_type: ReserveTypeShape::AttributedQualifiedSymbol(mode_names[3]),
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
        for imported in [
            vec![(mode_names[imported_index], SymbolKind::Mode)],
            vec![
                (mode_names[imported_index], SymbolKind::Mode),
                (mode_names[imported_index], SymbolKind::Mode),
            ],
        ] {
            let imported_symbols = source_local_and_imported_symbols_env(
                module.clone(),
                &local_modes,
                imported.as_slice(),
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&exact, module.clone(), &imported_symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
    }
    let unrelated_import = source_local_and_imported_symbols_env(
        module.clone(),
        &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
        &[("UnrelatedLongObjectMode", SymbolKind::Mode)],
    );
    assert!(
        super::extract_source_local_object_mode_long_chain_three_hop_asserted_head(
            &exact,
            module.clone(),
            &unrelated_import,
        )
        .is_some()
    );
    for (asserted_spelling, asserted_head_symbols) in [
        (
            "ImportedLongObjectMode",
            source_local_and_imported_symbols_env(
                module.clone(),
                &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
                &[("ImportedLongObjectMode", SymbolKind::Mode)],
            ),
        ),
        (
            "AmbiguousLongObjectMode",
            source_local_and_imported_symbols_env(
                module.clone(),
                &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
                &[
                    ("AmbiguousLongObjectMode", SymbolKind::Mode),
                    ("AmbiguousLongObjectMode", SymbolKind::Mode),
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
fn source_local_object_mode_long_chain_two_hop_asserted_head_consumes_seven_expansions() {
    let source_id = source_id(225);
    let module = ResolverModuleId::new(
        PackageId::new("test"),
        ModulePath::new("local_object_mode_long_chain_two_hop_asserted_head"),
    );
    let mode_names = [
        "BaseObjectMode",
        "ChainObjectMode1",
        "ChainObjectMode2",
        "ChainObjectMode3",
        "ChainObjectMode4",
        "ChainObjectMode5",
        "ChainObjectMode6",
    ];
    let mode_labels = [
        "BaseObjectModeDef",
        "ChainObjectMode1Def",
        "ChainObjectMode2Def",
        "ChainObjectMode3Def",
        "ChainObjectMode4Def",
        "ChainObjectMode5Def",
        "ChainObjectMode6Def",
    ];
    let symbols = source_local_symbols_env(
        module.clone(),
        &[
            (mode_names[0], SymbolKind::Mode),
            (mode_names[1], SymbolKind::Mode),
            (mode_names[2], SymbolKind::Mode),
            (mode_names[3], SymbolKind::Mode),
            (mode_names[4], SymbolKind::Mode),
            (mode_names[5], SymbolKind::Mode),
            (mode_names[6], SymbolKind::Mode),
            ("ChainObjectMode7", SymbolKind::Mode),
            ("OtherLongObjectMode", SymbolKind::Mode),
        ],
    );
    let theorem = IdentifierTypeAssertionTheoremSpec {
        status: None,
        label: "LongLocalObjectModeTwoHopAssertedHeadPayloadBoundary",
        subject: "x",
        asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[4]),
        negated: false,
        recovered_label: false,
    };
    let exact_modes = || {
        vec![
            mode_definition_with_label(
                mode_names[0],
                mode_labels[0],
                ReserveTypeShape::Builtin("object"),
            ),
            mode_definition_with_label(
                mode_names[1],
                mode_labels[1],
                ReserveTypeShape::QualifiedSymbol(mode_names[0]),
            ),
            mode_definition_with_label(
                mode_names[2],
                mode_labels[2],
                ReserveTypeShape::QualifiedSymbol(mode_names[1]),
            ),
            mode_definition_with_label(
                mode_names[3],
                mode_labels[3],
                ReserveTypeShape::QualifiedSymbol(mode_names[2]),
            ),
            mode_definition_with_label(
                mode_names[4],
                mode_labels[4],
                ReserveTypeShape::QualifiedSymbol(mode_names[3]),
            ),
            mode_definition_with_label(
                mode_names[5],
                mode_labels[5],
                ReserveTypeShape::QualifiedSymbol(mode_names[4]),
            ),
            mode_definition_with_label(
                mode_names[6],
                mode_labels[6],
                ReserveTypeShape::QualifiedSymbol(mode_names[5]),
            ),
        ]
    };
    let reserve = || {
        vec![reserve_item(
            vec!["x"],
            ReserveTypeShape::QualifiedSymbol(mode_names[6]),
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
        extract_source_local_object_mode_long_chain_reserved_variable_type_assertion(
            &exact,
            module.clone(),
            &symbols,
        )
        .is_none(),
        "Task 179 must reject the Task 225 two-hop source"
    );
    assert!(
        super::extract_source_local_object_mode_long_chain_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .is_none(),
        "Task 200 must reject the Task 225 two-hop source"
    );
    assert!(
        super::extract_source_local_object_mode_long_chain_radix_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .is_none(),
        "Task 210 must reject the Task 225 two-hop source"
    );
    assert!(
        super::extract_source_local_mode_long_chain_two_hop_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .is_none(),
        "Task 224 must reject the Task 225 object-terminal two-hop source"
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
    ];
    assert_eq!(preexisting_owner_results.len(), 34);
    assert!(
        preexisting_owner_results
            .into_iter()
            .all(|result| result.is_none()),
        "all 34 legacy type-assertion owner routes must reject Task 225"
    );
    let payload = super::extract_source_local_object_mode_long_chain_two_hop_asserted_head(
        &exact,
        module.clone(),
        &symbols,
    )
    .expect("exact seven-expansion two-hop source should extract");
    assert_eq!(payload.reserve.mode_expansions.len(), 7);
    assert_eq!(payload.reserve.bridge.bindings().len(), 1);
    assert_eq!(payload.subject_lookup_ordinal, 1);
    let source_binding = &payload.reserve.bridge.bindings()[0];
    assert_eq!(source_binding.type_spelling, mode_names[6]);
    assert_eq!(payload.asserted_type.spelling, mode_names[4]);
    assert_ne!(payload.asserted_type.head, source_binding.type_head);
    assert!(matches!(source_binding.type_head, TypeHeadInput::Symbol(_)));
    assert_ne!(payload.asserted_type_site, payload.subject_site);
    assert_ne!(payload.asserted_type.range, source_binding.type_range);
    let binding_expansion = payload
        .reserve
        .mode_expansions
        .iter()
        .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[6]))
        .map(|(_, expansion)| expansion)
        .expect("ChainObjectMode6 binding expansion should exist");
    assert_eq!(binding_expansion.radix.spelling, mode_names[5]);
    assert_ne!(binding_expansion.radix.head, payload.asserted_type.head);
    let TypeHeadInput::Symbol(intermediate_symbol) = &binding_expansion.radix.head else {
        panic!("ChainObjectMode5 relation intermediate should resolve to a symbol");
    };
    let intermediate_expansion = payload
        .reserve
        .mode_expansions
        .get(intermediate_symbol)
        .expect("ChainObjectMode5 intermediate expansion should exist");
    assert_eq!(intermediate_expansion.radix.spelling, mode_names[4]);
    assert_eq!(
        intermediate_expansion.radix.head,
        payload.asserted_type.head
    );

    let output = super::source_local_object_mode_long_chain_two_hop_asserted_head_output(
        &exact,
        module.clone(),
        &symbols,
    )
    .expect("exact seven-expansion asserted-head source should reach checker");
    assert_source_reserved_variable_type_assertion_output(&output)
        .expect("Task 225 checker payload invariants should hold");
    assert_eq!(output.subject_binding, BindingId::new(0));
    assert_eq!(output.payload.subject_lookup_ordinal, 1);
    assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
    assert_eq!(output.term_formula.type_entries().len(), 3);
    assert_eq!(output.term_formula.normalized_types().len(), 1);
    assert_eq!(output.subject_result_input.spelling, mode_names[6]);
    assert_eq!(output.asserted_type_input.spelling, mode_names[4]);
    assert_ne!(
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
        .expect("BaseObjectMode terminal expansion should exist");
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
                super::TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_TWO_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
            ),
            vec![
                super::TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_TWO_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY
                    .to_owned()
            ]
        );
    };
    for removed in mode_names {
        let mut invalid =
            super::source_local_object_mode_long_chain_two_hop_asserted_head_output(
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
        super::source_local_object_mode_long_chain_two_hop_asserted_head_output(
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
    invalid.subject_result_input.spelling = mode_names[5].to_owned();
    corruptions.push(invalid);
    let mut invalid = exact_output();
    invalid.asserted_type_input.spelling = "OtherLongObjectMode".to_owned();
    corruptions.push(invalid);
    let mut invalid = exact_output();
    invalid.asserted_type_input.head = invalid.subject_result_input.head.clone();
    corruptions.push(invalid);
    let mut invalid = exact_output();
    invalid.asserted_type_input.head = TypeHeadInput::BuiltinSet;
    corruptions.push(invalid);
    let mut invalid = exact_output();
    invalid.subject_result_input.head = TypeHeadInput::BuiltinSet;
    corruptions.push(invalid);
    let mut invalid = exact_output();
    invalid.subject_result_input.source_range = invalid.asserted_type_input.source_range;
    corruptions.push(invalid);
    let mut invalid = exact_output();
    invalid.subject_result_input.site = invalid.asserted_type_input.site.clone();
    corruptions.push(invalid);
    let mut invalid = exact_output();
    invalid.asserted_type_input.source_range = invalid.subject_result_input.source_range;
    corruptions.push(invalid);
    let mut invalid = exact_output();
    invalid.asserted_type_input.site = invalid.subject_result_input.site.clone();
    corruptions.push(invalid);
    let mut invalid = exact_output();
    let binding_head = invalid.subject_result_input.head.clone();
    let (_, binding_expansion) = invalid
        .payload
        .reserve
        .mode_expansions
        .iter_mut()
        .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[6]))
        .unwrap();
    binding_expansion.radix.head = binding_head;
    corruptions.push(invalid);
    for mode_name in &mode_names[1..] {
        let mut invalid = exact_output();
        let (_, expansion) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(*mode_name))
            .unwrap();
        expansion.radix.spelling = "object".to_owned();
        expansion.radix.head = TypeHeadInput::BuiltinSet;
        corruptions.push(invalid);
    }
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
    let mut invalid = exact_output();
    let (_, base_expansion) = invalid
        .payload
        .reserve
        .mode_expansions
        .iter_mut()
        .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[0]))
        .unwrap();
    base_expansion.radix.spelling = "object".to_owned();
    base_expansion.radix.head = TypeHeadInput::BuiltinSet;
    corruptions.push(invalid);
    for invalid in corruptions {
        assert_invalid_output(invalid);
    }
    assert_source_reserved_variable_type_assertion_output(&exact_output())
        .expect("corrupting cloned Task 225 outputs must not mutate the exact output");

    let assert_extraction_gap = |ast, context: &str| {
        assert_eq!(
            source_type_elaboration_detail_keys(&ast, module.clone(), &symbols),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()],
            "{context}"
        );
    };
    fn next_permutation(order: &mut [usize; 7]) -> bool {
        let Some(pivot) = (0..order.len() - 1)
            .rev()
            .find(|&index| order[index] < order[index + 1])
        else {
            return false;
        };
        let successor = (pivot + 1..order.len())
            .rev()
            .find(|&index| order[pivot] < order[index])
            .expect("a lexicographic successor must exist after the pivot");
        order.swap(pivot, successor);
        order[pivot + 1..].reverse();
        true
    }
    let ordered = exact_modes();
    let public_dispatch_orders = [
        [1, 0, 2, 3, 4, 5, 6],
        [0, 2, 1, 3, 4, 5, 6],
        [0, 1, 3, 2, 4, 5, 6],
        [0, 1, 2, 4, 3, 5, 6],
        [0, 1, 2, 3, 5, 4, 6],
        [0, 1, 2, 3, 4, 6, 5],
        [1, 2, 3, 4, 5, 6, 0],
        [6, 5, 4, 3, 2, 1, 0],
    ];
    let mut order = [0, 1, 2, 3, 4, 5, 6];
    let mut permutation_count = 0;
    let mut public_dispatch_representative_count = 0;
    while next_permutation(&mut order) {
        permutation_count += 1;
        let permuted = mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            vec![
                ordered[order[0]],
                ordered[order[1]],
                ordered[order[2]],
                ordered[order[3]],
                ordered[order[4]],
                ordered[order[5]],
                ordered[order[6]],
            ],
            reserve(),
            theorem,
        );
        assert!(
            super::extract_source_local_object_mode_long_chain_two_hop_asserted_head(
                &permuted,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "definition permutation {order:?} must not reach Task 225"
        );
        if public_dispatch_orders.contains(&order) {
            public_dispatch_representative_count += 1;
            assert_extraction_gap(
                permuted,
                &format!("public definition permutation {order:?}"),
            );
        }
    }
    assert_eq!(permutation_count, 5_039);
    assert_eq!(public_dispatch_representative_count, 8);
    let mut mode_near_misses = vec![
        Vec::<ModeDefinitionSpec>::new(),
        {
            let mut modes = exact_modes();
            modes.push(modes[0]);
            modes
        },
        {
            let mut modes = exact_modes();
            modes.push(mode_definition_with_label(
                "UnusedLongObjectMode",
                "UnusedLongObjectModeDef",
                ReserveTypeShape::QualifiedSymbol(mode_names[6]),
            ));
            modes
        },
    ];
    for index in 0..mode_names.len() {
        let mut missing = exact_modes();
        missing.remove(index);
        mode_near_misses.push(missing);
        let mut duplicate = exact_modes();
        duplicate.push(duplicate[index]);
        mode_near_misses.push(duplicate);
        let mut malformed = exact_modes();
        malformed[index].rhs_shape = if index == 0 {
            ReserveTypeShape::Builtin("set")
        } else {
            ReserveTypeShape::QualifiedSymbol(mode_names[(index + 1) % mode_names.len()])
        };
        mode_near_misses.push(malformed);
        let mut wrong_label = exact_modes();
        wrong_label[index].label = Some("WrongLongObjectModeDef");
        mode_near_misses.push(wrong_label);
        let mut wrong_spelling = exact_modes();
        wrong_spelling[index].pattern = "OtherLongObjectMode";
        mode_near_misses.push(wrong_spelling);
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
        "ChainObjectMode7",
        "ChainObjectMode7Def",
        ReserveTypeShape::QualifiedSymbol(mode_names[6]),
    ));
    source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
        source_id,
        deeper_modes,
        vec![reserve_item(
            vec!["x"],
            ReserveTypeShape::QualifiedSymbol("ChainObjectMode7"),
        )],
        IdentifierTypeAssertionTheoremSpec {
            asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[6]),
            ..theorem
        },
    ));
    for bad_reserve in [
        ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[6]),
        ReserveTypeShape::AttributedQualifiedSymbol(mode_names[6]),
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
        vec![reserve_item(
            vec!["y"],
            ReserveTypeShape::QualifiedSymbol(mode_names[6]),
        )],
        theorem,
    ));
    source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
        source_id,
        exact_modes(),
        vec![
            reserve_item(vec!["x"], ReserveTypeShape::QualifiedSymbol(mode_names[6])),
            reserve_item(vec!["y"], ReserveTypeShape::Builtin("object")),
        ],
        theorem,
    ));
    for bad_reserve in [
        vec![reserve_item(
            vec!["x"],
            ReserveTypeShape::QualifiedSymbol(mode_names[5]),
        )],
        vec![reserve_item(
            vec!["x"],
            ReserveTypeShape::QualifiedSymbol("OtherLongObjectMode"),
        )],
        vec![reserve_item(
            vec!["x", "y"],
            ReserveTypeShape::QualifiedSymbol(mode_names[6]),
        )],
    ] {
        source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            exact_modes(),
            bad_reserve,
            theorem,
        ));
    }
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
            asserted_type: ReserveTypeShape::Builtin("set"),
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
            asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[5]),
            ..theorem
        },
        IdentifierTypeAssertionTheoremSpec {
            asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[6]),
            ..theorem
        },
        IdentifierTypeAssertionTheoremSpec {
            asserted_type: ReserveTypeShape::QualifiedSymbol("OtherLongObjectMode"),
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
        for imported in [
            vec![(mode_names[imported_index], SymbolKind::Mode)],
            vec![
                (mode_names[imported_index], SymbolKind::Mode),
                (mode_names[imported_index], SymbolKind::Mode),
            ],
        ] {
            let imported_symbols = source_local_and_imported_symbols_env(
                module.clone(),
                &local_modes,
                imported.as_slice(),
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&exact, module.clone(), &imported_symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
    }
    let unrelated_import = source_local_and_imported_symbols_env(
        module.clone(),
        &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
        &[("UnrelatedLongObjectMode", SymbolKind::Mode)],
    );
    assert!(
        super::extract_source_local_object_mode_long_chain_two_hop_asserted_head(
            &exact,
            module.clone(),
            &unrelated_import,
        )
        .is_some()
    );
    for (asserted_spelling, asserted_head_symbols) in [
        (
            "ImportedLongObjectMode",
            source_local_and_imported_symbols_env(
                module.clone(),
                &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
                &[("ImportedLongObjectMode", SymbolKind::Mode)],
            ),
        ),
        (
            "AmbiguousLongObjectMode",
            source_local_and_imported_symbols_env(
                module.clone(),
                &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
                &[
                    ("AmbiguousLongObjectMode", SymbolKind::Mode),
                    ("AmbiguousLongObjectMode", SymbolKind::Mode),
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
fn source_local_object_mode_long_chain_radix_asserted_head_consumes_seven_expansions() {
    let source_id = source_id(210);
    let module = ResolverModuleId::new(
        PackageId::new("test"),
        ModulePath::new("local_object_mode_long_chain_radix_asserted_head"),
    );
    let mode_names = [
        "BaseObjectMode",
        "ChainObjectMode1",
        "ChainObjectMode2",
        "ChainObjectMode3",
        "ChainObjectMode4",
        "ChainObjectMode5",
        "ChainObjectMode6",
    ];
    let mode_labels = [
        "BaseObjectModeDef",
        "ChainObjectMode1Def",
        "ChainObjectMode2Def",
        "ChainObjectMode3Def",
        "ChainObjectMode4Def",
        "ChainObjectMode5Def",
        "ChainObjectMode6Def",
    ];
    let symbols = source_local_symbols_env(
        module.clone(),
        &[
            (mode_names[0], SymbolKind::Mode),
            (mode_names[1], SymbolKind::Mode),
            (mode_names[2], SymbolKind::Mode),
            (mode_names[3], SymbolKind::Mode),
            (mode_names[4], SymbolKind::Mode),
            (mode_names[5], SymbolKind::Mode),
            (mode_names[6], SymbolKind::Mode),
            ("ChainObjectMode7", SymbolKind::Mode),
            ("OtherLongObjectMode", SymbolKind::Mode),
        ],
    );
    let theorem = IdentifierTypeAssertionTheoremSpec {
        status: None,
        label: "LongLocalObjectModeRadixAssertedHeadPayloadBoundary",
        subject: "x",
        asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[5]),
        negated: false,
        recovered_label: false,
    };
    let exact_modes = || {
        vec![
            mode_definition_with_label(
                mode_names[0],
                mode_labels[0],
                ReserveTypeShape::Builtin("object"),
            ),
            mode_definition_with_label(
                mode_names[1],
                mode_labels[1],
                ReserveTypeShape::QualifiedSymbol(mode_names[0]),
            ),
            mode_definition_with_label(
                mode_names[2],
                mode_labels[2],
                ReserveTypeShape::QualifiedSymbol(mode_names[1]),
            ),
            mode_definition_with_label(
                mode_names[3],
                mode_labels[3],
                ReserveTypeShape::QualifiedSymbol(mode_names[2]),
            ),
            mode_definition_with_label(
                mode_names[4],
                mode_labels[4],
                ReserveTypeShape::QualifiedSymbol(mode_names[3]),
            ),
            mode_definition_with_label(
                mode_names[5],
                mode_labels[5],
                ReserveTypeShape::QualifiedSymbol(mode_names[4]),
            ),
            mode_definition_with_label(
                mode_names[6],
                mode_labels[6],
                ReserveTypeShape::QualifiedSymbol(mode_names[5]),
            ),
        ]
    };
    let reserve = || {
        vec![reserve_item(
            vec!["x"],
            ReserveTypeShape::QualifiedSymbol(mode_names[6]),
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
        extract_source_local_object_mode_long_chain_reserved_variable_type_assertion(
            &exact,
            module.clone(),
            &symbols,
        )
        .is_none(),
        "Task 179 must reject the Task 210 object immediate-radix source"
    );
    assert!(
        super::extract_source_local_object_mode_long_chain_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .is_none(),
        "Task 200 must reject the Task 210 object immediate-radix source"
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
    ];
    assert_eq!(preexisting_owner_results.len(), 35);
    assert!(
        preexisting_owner_results
            .into_iter()
            .all(|result| result.is_none()),
        "all 35 pre-existing type-assertion owner routes must reject Task 210"
    );
    let payload = super::extract_source_local_object_mode_long_chain_radix_asserted_head(
        &exact,
        module.clone(),
        &symbols,
    )
    .expect("exact seven-expansion immediate-radix source should extract");
    assert_eq!(payload.reserve.mode_expansions.len(), 7);
    assert_eq!(payload.reserve.bridge.bindings().len(), 1);
    assert_eq!(payload.subject_lookup_ordinal, 1);
    let source_binding = &payload.reserve.bridge.bindings()[0];
    assert_eq!(source_binding.type_spelling, mode_names[6]);
    assert_eq!(payload.asserted_type.spelling, mode_names[5]);
    assert_ne!(payload.asserted_type.head, source_binding.type_head);
    assert!(matches!(source_binding.type_head, TypeHeadInput::Symbol(_)));
    assert_ne!(payload.asserted_type_site, payload.subject_site);
    assert_ne!(payload.asserted_type.range, source_binding.type_range);
    let binding_expansion = payload
        .reserve
        .mode_expansions
        .iter()
        .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[6]))
        .map(|(_, expansion)| expansion)
        .expect("ChainObjectMode6 binding expansion should exist");
    assert_eq!(binding_expansion.radix.spelling, mode_names[5]);
    assert_eq!(binding_expansion.radix.head, payload.asserted_type.head);

    let output = super::source_local_object_mode_long_chain_radix_asserted_head_output(
        &exact,
        module.clone(),
        &symbols,
    )
    .expect("exact seven-expansion asserted-head source should reach checker");
    assert_source_reserved_variable_type_assertion_output(&output)
        .expect("Task 210 checker payload invariants should hold");
    assert_eq!(output.subject_binding, BindingId::new(0));
    assert_eq!(output.payload.subject_lookup_ordinal, 1);
    assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
    assert_eq!(output.term_formula.type_entries().len(), 3);
    assert_eq!(output.term_formula.normalized_types().len(), 1);
    assert_eq!(output.subject_result_input.spelling, mode_names[6]);
    assert_eq!(output.asserted_type_input.spelling, mode_names[5]);
    assert_ne!(
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
        .expect("BaseObjectMode terminal expansion should exist");
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
                super::TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_RADIX_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
            ),
            vec![
                super::TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_RADIX_ASSERTED_HEAD_INVALID_PAYLOAD_KEY
                    .to_owned()
            ]
        );
    };
    for removed in mode_names {
        let mut invalid =
            super::source_local_object_mode_long_chain_radix_asserted_head_output(
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
        super::source_local_object_mode_long_chain_radix_asserted_head_output(
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
    invalid.subject_result_input.spelling = mode_names[5].to_owned();
    corruptions.push(invalid);
    let mut invalid = exact_output();
    invalid.asserted_type_input.spelling = "OtherLongObjectMode".to_owned();
    corruptions.push(invalid);
    let mut invalid = exact_output();
    invalid.asserted_type_input.head = invalid.subject_result_input.head.clone();
    corruptions.push(invalid);
    let mut invalid = exact_output();
    invalid.asserted_type_input.head = TypeHeadInput::BuiltinSet;
    corruptions.push(invalid);
    let mut invalid = exact_output();
    invalid.subject_result_input.head = TypeHeadInput::BuiltinSet;
    corruptions.push(invalid);
    let mut invalid = exact_output();
    invalid.subject_result_input.source_range = invalid.asserted_type_input.source_range;
    corruptions.push(invalid);
    let mut invalid = exact_output();
    invalid.subject_result_input.site = invalid.asserted_type_input.site.clone();
    corruptions.push(invalid);
    let mut invalid = exact_output();
    invalid.asserted_type_input.source_range = invalid.subject_result_input.source_range;
    corruptions.push(invalid);
    let mut invalid = exact_output();
    invalid.asserted_type_input.site = invalid.subject_result_input.site.clone();
    corruptions.push(invalid);
    let mut invalid = exact_output();
    let binding_head = invalid.subject_result_input.head.clone();
    let (_, binding_expansion) = invalid
        .payload
        .reserve
        .mode_expansions
        .iter_mut()
        .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[6]))
        .unwrap();
    binding_expansion.radix.head = binding_head;
    corruptions.push(invalid);
    for mode_name in &mode_names[1..] {
        let mut invalid = exact_output();
        let (_, expansion) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(*mode_name))
            .unwrap();
        expansion.radix.spelling = "set".to_owned();
        expansion.radix.head = TypeHeadInput::BuiltinSet;
        corruptions.push(invalid);
    }
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
    let mut invalid = exact_output();
    let (_, base_expansion) = invalid
        .payload
        .reserve
        .mode_expansions
        .iter_mut()
        .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(mode_names[0]))
        .unwrap();
    base_expansion.radix.spelling = "set".to_owned();
    base_expansion.radix.head = TypeHeadInput::BuiltinSet;
    corruptions.push(invalid);
    for invalid in corruptions {
        assert_invalid_output(invalid);
    }
    assert_source_reserved_variable_type_assertion_output(&exact_output())
        .expect("corrupting cloned Task 210 outputs must not mutate the exact output");

    let assert_extraction_gap = |ast, context: &str| {
        assert_eq!(
            source_type_elaboration_detail_keys(&ast, module.clone(), &symbols),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()],
            "{context}"
        );
    };
    fn next_permutation(order: &mut [usize; 7]) -> bool {
        let Some(pivot) = (0..order.len() - 1)
            .rev()
            .find(|&index| order[index] < order[index + 1])
        else {
            return false;
        };
        let successor = (pivot + 1..order.len())
            .rev()
            .find(|&index| order[pivot] < order[index])
            .expect("a lexicographic successor must exist after the pivot");
        order.swap(pivot, successor);
        order[pivot + 1..].reverse();
        true
    }
    let ordered = exact_modes();
    let public_dispatch_orders = [
        [1, 0, 2, 3, 4, 5, 6],
        [0, 2, 1, 3, 4, 5, 6],
        [0, 1, 3, 2, 4, 5, 6],
        [0, 1, 2, 4, 3, 5, 6],
        [0, 1, 2, 3, 5, 4, 6],
        [0, 1, 2, 3, 4, 6, 5],
        [1, 2, 3, 4, 5, 6, 0],
        [6, 5, 4, 3, 2, 1, 0],
    ];
    let mut order = [0, 1, 2, 3, 4, 5, 6];
    let mut permutation_count = 0;
    let mut public_dispatch_representative_count = 0;
    while next_permutation(&mut order) {
        permutation_count += 1;
        let permuted = mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            vec![
                ordered[order[0]],
                ordered[order[1]],
                ordered[order[2]],
                ordered[order[3]],
                ordered[order[4]],
                ordered[order[5]],
                ordered[order[6]],
            ],
            reserve(),
            theorem,
        );
        assert!(
            super::extract_source_local_object_mode_long_chain_radix_asserted_head(
                &permuted,
                module.clone(),
                &symbols,
            )
            .is_none(),
            "definition permutation {order:?} must not reach Task 210"
        );
        if public_dispatch_orders.contains(&order) {
            public_dispatch_representative_count += 1;
            assert_extraction_gap(
                permuted,
                &format!("public definition permutation {order:?}"),
            );
        }
    }
    assert_eq!(permutation_count, 5_039);
    assert_eq!(public_dispatch_representative_count, 8);
    let mut mode_near_misses = vec![
        Vec::<ModeDefinitionSpec>::new(),
        {
            let mut modes = exact_modes();
            modes.push(modes[0]);
            modes
        },
        {
            let mut modes = exact_modes();
            modes.push(mode_definition_with_label(
                "UnusedLongObjectMode",
                "UnusedLongObjectModeDef",
                ReserveTypeShape::QualifiedSymbol(mode_names[6]),
            ));
            modes
        },
    ];
    for index in 0..mode_names.len() {
        let mut missing = exact_modes();
        missing.remove(index);
        mode_near_misses.push(missing);
        let mut duplicate = exact_modes();
        duplicate.push(duplicate[index]);
        mode_near_misses.push(duplicate);
        let mut malformed = exact_modes();
        malformed[index].rhs_shape = if index == 0 {
            ReserveTypeShape::Builtin("set")
        } else {
            ReserveTypeShape::QualifiedSymbol(mode_names[(index + 1) % mode_names.len()])
        };
        mode_near_misses.push(malformed);
        let mut wrong_label = exact_modes();
        wrong_label[index].label = Some("WrongLongObjectModeDef");
        mode_near_misses.push(wrong_label);
        let mut wrong_spelling = exact_modes();
        wrong_spelling[index].pattern = "OtherLongObjectMode";
        mode_near_misses.push(wrong_spelling);
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
        "ChainObjectMode7",
        "ChainObjectMode7Def",
        ReserveTypeShape::QualifiedSymbol(mode_names[6]),
    ));
    source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
        source_id,
        deeper_modes,
        vec![reserve_item(
            vec!["x"],
            ReserveTypeShape::QualifiedSymbol("ChainObjectMode7"),
        )],
        IdentifierTypeAssertionTheoremSpec {
            asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[6]),
            ..theorem
        },
    ));
    for bad_reserve in [
        ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[6]),
        ReserveTypeShape::AttributedQualifiedSymbol(mode_names[6]),
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
        vec![reserve_item(
            vec!["y"],
            ReserveTypeShape::QualifiedSymbol(mode_names[6]),
        )],
        theorem,
    ));
    source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
        source_id,
        exact_modes(),
        vec![
            reserve_item(vec!["x"], ReserveTypeShape::QualifiedSymbol(mode_names[6])),
            reserve_item(vec!["y"], ReserveTypeShape::Builtin("object")),
        ],
        theorem,
    ));
    for bad_reserve in [
        vec![reserve_item(
            vec!["x"],
            ReserveTypeShape::QualifiedSymbol(mode_names[5]),
        )],
        vec![reserve_item(
            vec!["x"],
            ReserveTypeShape::QualifiedSymbol("OtherLongObjectMode"),
        )],
        vec![reserve_item(
            vec!["x", "y"],
            ReserveTypeShape::QualifiedSymbol(mode_names[6]),
        )],
    ] {
        source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            exact_modes(),
            bad_reserve,
            theorem,
        ));
    }
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
            asserted_type: ReserveTypeShape::Builtin("set"),
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
            asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[4]),
            ..theorem
        },
        IdentifierTypeAssertionTheoremSpec {
            asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[6]),
            ..theorem
        },
        IdentifierTypeAssertionTheoremSpec {
            asserted_type: ReserveTypeShape::QualifiedSymbol("OtherLongObjectMode"),
            ..theorem
        },
        IdentifierTypeAssertionTheoremSpec {
            asserted_type: ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[5]),
            ..theorem
        },
        IdentifierTypeAssertionTheoremSpec {
            asserted_type: ReserveTypeShape::AttributedQualifiedSymbol(mode_names[5]),
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
        for imported in [
            vec![(mode_names[imported_index], SymbolKind::Mode)],
            vec![
                (mode_names[imported_index], SymbolKind::Mode),
                (mode_names[imported_index], SymbolKind::Mode),
            ],
        ] {
            let imported_symbols = source_local_and_imported_symbols_env(
                module.clone(),
                &local_modes,
                imported.as_slice(),
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&exact, module.clone(), &imported_symbols),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
    }
    let unrelated_import = source_local_and_imported_symbols_env(
        module.clone(),
        &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
        &[("UnrelatedLongObjectMode", SymbolKind::Mode)],
    );
    assert!(
        super::extract_source_local_object_mode_long_chain_radix_asserted_head(
            &exact,
            module.clone(),
            &unrelated_import,
        )
        .is_some()
    );
    for (asserted_spelling, asserted_head_symbols) in [
        (
            "ImportedLongObjectMode",
            source_local_and_imported_symbols_env(
                module.clone(),
                &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
                &[("ImportedLongObjectMode", SymbolKind::Mode)],
            ),
        ),
        (
            "AmbiguousLongObjectMode",
            source_local_and_imported_symbols_env(
                module.clone(),
                &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
                &[
                    ("AmbiguousLongObjectMode", SymbolKind::Mode),
                    ("AmbiguousLongObjectMode", SymbolKind::Mode),
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
fn source_local_object_mode_long_chain_asserted_head_consumes_seven_expansions() {
    let source_id = source_id(200);
    let module = ResolverModuleId::new(
        PackageId::new("test"),
        ModulePath::new("local_object_mode_long_chain_asserted_head"),
    );
    let mode_names = [
        "BaseObjectMode",
        "ChainObjectMode1",
        "ChainObjectMode2",
        "ChainObjectMode3",
        "ChainObjectMode4",
        "ChainObjectMode5",
        "ChainObjectMode6",
    ];
    let mode_labels = [
        "BaseObjectModeDef",
        "ChainObjectMode1Def",
        "ChainObjectMode2Def",
        "ChainObjectMode3Def",
        "ChainObjectMode4Def",
        "ChainObjectMode5Def",
        "ChainObjectMode6Def",
    ];
    let symbols = source_local_symbols_env(
        module.clone(),
        &[
            (mode_names[0], SymbolKind::Mode),
            (mode_names[1], SymbolKind::Mode),
            (mode_names[2], SymbolKind::Mode),
            (mode_names[3], SymbolKind::Mode),
            (mode_names[4], SymbolKind::Mode),
            (mode_names[5], SymbolKind::Mode),
            (mode_names[6], SymbolKind::Mode),
            ("ChainObjectMode7", SymbolKind::Mode),
            ("OtherLongObjectMode", SymbolKind::Mode),
        ],
    );
    let theorem = IdentifierTypeAssertionTheoremSpec {
        status: None,
        label: "LongLocalObjectModeAssertedHeadPayloadBoundary",
        subject: "x",
        asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[6]),
        negated: false,
        recovered_label: false,
    };
    let exact_modes = || {
        vec![
            mode_definition_with_label(
                mode_names[0],
                mode_labels[0],
                ReserveTypeShape::Builtin("object"),
            ),
            mode_definition_with_label(
                mode_names[1],
                mode_labels[1],
                ReserveTypeShape::QualifiedSymbol(mode_names[0]),
            ),
            mode_definition_with_label(
                mode_names[2],
                mode_labels[2],
                ReserveTypeShape::QualifiedSymbol(mode_names[1]),
            ),
            mode_definition_with_label(
                mode_names[3],
                mode_labels[3],
                ReserveTypeShape::QualifiedSymbol(mode_names[2]),
            ),
            mode_definition_with_label(
                mode_names[4],
                mode_labels[4],
                ReserveTypeShape::QualifiedSymbol(mode_names[3]),
            ),
            mode_definition_with_label(
                mode_names[5],
                mode_labels[5],
                ReserveTypeShape::QualifiedSymbol(mode_names[4]),
            ),
            mode_definition_with_label(
                mode_names[6],
                mode_labels[6],
                ReserveTypeShape::QualifiedSymbol(mode_names[5]),
            ),
        ]
    };
    let reserve = || {
        vec![reserve_item(
            vec!["x"],
            ReserveTypeShape::QualifiedSymbol(mode_names[6]),
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
        extract_source_local_object_mode_long_chain_reserved_variable_type_assertion(
            &exact,
            module.clone(),
            &symbols,
        )
        .is_none()
    );
    assert!(
        super::extract_source_local_mode_long_chain_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .is_none()
    );
    assert!(
        super::extract_source_local_object_mode_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .is_none()
    );
    assert!(
        super::extract_source_chained_local_object_mode_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .is_none()
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
        super::extract_source_three_edge_local_object_mode_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .is_none()
    );
    assert!(
        super::extract_source_four_edge_local_object_mode_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .is_none()
    );
    let payload = super::extract_source_local_object_mode_long_chain_asserted_head(
        &exact,
        module.clone(),
        &symbols,
    )
    .expect("exact seven-expansion object same-head source should extract");
    assert_eq!(payload.reserve.mode_expansions.len(), 7);
    assert_eq!(payload.reserve.bridge.bindings().len(), 1);
    assert_eq!(payload.subject_lookup_ordinal, 1);
    let source_binding = &payload.reserve.bridge.bindings()[0];
    assert_eq!(source_binding.type_spelling, mode_names[6]);
    assert_eq!(payload.asserted_type.spelling, mode_names[6]);
    assert_eq!(payload.asserted_type.head, source_binding.type_head);
    assert!(matches!(source_binding.type_head, TypeHeadInput::Symbol(_)));
    assert_ne!(payload.asserted_type_site, payload.subject_site);
    assert_ne!(payload.asserted_type.range, source_binding.type_range);

    let output = super::source_local_object_mode_long_chain_asserted_head_output(
        &exact,
        module.clone(),
        &symbols,
    )
    .expect("exact seven-expansion object asserted-head source should reach checker");
    assert_source_reserved_variable_type_assertion_output(&output)
        .expect("Task 200 checker payload invariants should hold");
    assert_eq!(output.subject_binding, BindingId::new(0));
    assert_eq!(output.payload.subject_lookup_ordinal, 1);
    assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
    assert_eq!(output.term_formula.type_entries().len(), 3);
    assert_eq!(output.term_formula.normalized_types().len(), 1);
    assert_eq!(output.subject_result_input.spelling, mode_names[6]);
    assert_eq!(output.asserted_type_input.spelling, mode_names[6]);
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
        .expect("BaseObjectMode terminal expansion should exist");
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
                super::TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
            ),
            vec![
                super::TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_ASSERTED_HEAD_INVALID_PAYLOAD_KEY
                    .to_owned()
            ]
        );
    };
    for removed in mode_names {
        let mut invalid = super::source_local_object_mode_long_chain_asserted_head_output(
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
        super::source_local_object_mode_long_chain_asserted_head_output(
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
    invalid.asserted_type_input.spelling = "OtherLongObjectMode".to_owned();
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
        vec![
            exact_modes()[6],
            exact_modes()[5],
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
                "UnusedLongObjectMode",
                "UnusedLongObjectModeDef",
                ReserveTypeShape::QualifiedSymbol(mode_names[6]),
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
        wrong_label[index].label = Some("WrongLongObjectModeDef");
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
            &format!("object mode near miss {index}"),
        );
    }

    let mut source_near_misses = Vec::new();
    let mut deeper_modes = exact_modes();
    deeper_modes.push(mode_definition_with_label(
        "ChainObjectMode7",
        "ChainObjectMode7Def",
        ReserveTypeShape::QualifiedSymbol(mode_names[6]),
    ));
    source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
        source_id,
        deeper_modes,
        vec![reserve_item(
            vec!["x"],
            ReserveTypeShape::QualifiedSymbol("ChainObjectMode7"),
        )],
        IdentifierTypeAssertionTheoremSpec {
            asserted_type: ReserveTypeShape::QualifiedSymbol("ChainObjectMode7"),
            ..theorem
        },
    ));
    for bad_reserve in [
        ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[6]),
        ReserveTypeShape::AttributedQualifiedSymbol(mode_names[6]),
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
            reserve_item(vec!["x"], ReserveTypeShape::QualifiedSymbol(mode_names[6])),
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
            asserted_type: ReserveTypeShape::Builtin("set"),
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
            asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[4]),
            ..theorem
        },
        IdentifierTypeAssertionTheoremSpec {
            asserted_type: ReserveTypeShape::QualifiedSymbol(mode_names[5]),
            ..theorem
        },
        IdentifierTypeAssertionTheoremSpec {
            asserted_type: ReserveTypeShape::QualifiedSymbol("OtherLongObjectMode"),
            ..theorem
        },
        IdentifierTypeAssertionTheoremSpec {
            asserted_type: ReserveTypeShape::QualifiedSymbolWithArgs(mode_names[6]),
            ..theorem
        },
        IdentifierTypeAssertionTheoremSpec {
            asserted_type: ReserveTypeShape::AttributedQualifiedSymbol(mode_names[6]),
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
        assert_extraction_gap(near_miss, &format!("object source near miss {index}"));
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
        &[("UnrelatedLongObjectMode", SymbolKind::Mode)],
    );
    assert!(
        super::extract_source_local_object_mode_long_chain_asserted_head(
            &exact,
            module.clone(),
            &unrelated_import,
        )
        .is_some()
    );
    for (asserted_spelling, asserted_head_symbols) in [
        (
            "ImportedLongObjectMode",
            source_local_and_imported_symbols_env(
                module.clone(),
                &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
                &[("ImportedLongObjectMode", SymbolKind::Mode)],
            ),
        ),
        (
            "AmbiguousLongObjectMode",
            source_local_and_imported_symbols_env(
                module.clone(),
                &mode_names.map(|spelling| (spelling, SymbolKind::Mode)),
                &[
                    ("AmbiguousLongObjectMode", SymbolKind::Mode),
                    ("AmbiguousLongObjectMode", SymbolKind::Mode),
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
fn source_local_object_mode_long_chain_reserved_variable_type_assertion_consumes_seven_expansions()
 {
    let source_id = source_id(179);
    let module = ResolverModuleId::new(
        PackageId::new("test"),
        ModulePath::new("local_object_mode_long_chain_reserved_variable_type_assertion"),
    );
    let symbols = source_local_symbols_env(
        module.clone(),
        &[
            ("BaseObjectMode", SymbolKind::Mode),
            ("ChainObjectMode1", SymbolKind::Mode),
            ("ChainObjectMode2", SymbolKind::Mode),
            ("ChainObjectMode3", SymbolKind::Mode),
            ("ChainObjectMode4", SymbolKind::Mode),
            ("ChainObjectMode5", SymbolKind::Mode),
            ("ChainObjectMode6", SymbolKind::Mode),
        ],
    );
    let theorem = IdentifierTypeAssertionTheoremSpec {
        status: None,
        label: "LongLocalObjectModeReservedVariableTypeAssertionPayloadBoundary",
        subject: "x",
        asserted_type: ReserveTypeShape::Builtin("object"),
        recovered_label: false,
        negated: false,
    };
    let exact_modes = || {
        vec![
            mode_definition_with_label(
                "BaseObjectMode",
                "BaseObjectModeDef",
                ReserveTypeShape::Builtin("object"),
            ),
            mode_definition_with_label(
                "ChainObjectMode1",
                "ChainObjectMode1Def",
                ReserveTypeShape::QualifiedSymbol("BaseObjectMode"),
            ),
            mode_definition_with_label(
                "ChainObjectMode2",
                "ChainObjectMode2Def",
                ReserveTypeShape::QualifiedSymbol("ChainObjectMode1"),
            ),
            mode_definition_with_label(
                "ChainObjectMode3",
                "ChainObjectMode3Def",
                ReserveTypeShape::QualifiedSymbol("ChainObjectMode2"),
            ),
            mode_definition_with_label(
                "ChainObjectMode4",
                "ChainObjectMode4Def",
                ReserveTypeShape::QualifiedSymbol("ChainObjectMode3"),
            ),
            mode_definition_with_label(
                "ChainObjectMode5",
                "ChainObjectMode5Def",
                ReserveTypeShape::QualifiedSymbol("ChainObjectMode4"),
            ),
            mode_definition_with_label(
                "ChainObjectMode6",
                "ChainObjectMode6Def",
                ReserveTypeShape::QualifiedSymbol("ChainObjectMode5"),
            ),
        ]
    };
    let reserve = || {
        vec![reserve_item(
            vec!["x"],
            ReserveTypeShape::QualifiedSymbol("ChainObjectMode6"),
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
    let payload = extract_source_local_object_mode_long_chain_reserved_variable_type_assertion(
        &exact,
        module.clone(),
        &symbols,
    )
    .expect("exact object-terminal long-chain type assertion should extract");
    assert_eq!(payload.reserve.mode_expansions.len(), 7);
    assert_eq!(payload.reserve.bridge.bindings().len(), 1);
    assert_eq!(payload.reserve.bridge.bindings()[0].spelling, "x");
    assert_eq!(
        payload.reserve.bridge.bindings()[0].type_spelling,
        "ChainObjectMode6"
    );
    assert_eq!(payload.asserted_type.spelling, "object");
    assert_eq!(payload.asserted_type.head, TypeHeadInput::BuiltinObject);
    assert_ne!(
        payload.reserve.bridge.bindings()[0].type_range,
        payload.asserted_type.range
    );
    assert_eq!(payload.subject_lookup_ordinal, 1);

    let output = source_local_object_mode_long_chain_reserved_variable_type_assertion_output(
        &exact,
        module.clone(),
        &symbols,
    )
    .expect("exact object-terminal long-chain type assertion should reach checker");
    assert_source_reserved_variable_type_assertion_output(&output)
        .expect("Task 179 checked payload invariants should hold");
    assert_eq!(output.subject_binding, BindingId::new(0));
    assert_eq!(output.subject_result_input.spelling, "ChainObjectMode6");
    assert!(matches!(
        output.subject_result_input.head,
        TypeHeadInput::Symbol(_)
    ));
    assert_eq!(output.asserted_type_input.spelling, "object");
    assert_eq!(
        output.asserted_type_input.head,
        TypeHeadInput::BuiltinObject
    );
    assert_ne!(
        output.subject_result_input.source_range,
        output.asserted_type_input.source_range
    );
    assert_eq!(output.term_formula.terms().len(), 1);
    let (_, term) = output
        .term_formula
        .terms()
        .iter()
        .next()
        .expect("Task 179 subject should be inferred");
    assert_eq!(term.status, TermStatus::Inferred);
    assert!(term.deferred.is_empty());
    let (_, formula) = output
        .term_formula
        .formulas()
        .iter()
        .next()
        .expect("Task 179 type assertion should be checked");
    assert_eq!(formula.kind, FormulaKind::TypeAssertion);
    assert_eq!(formula.status, FormulaStatus::Checked);
    assert!(formula.expected_types.is_empty());
    assert!(formula.facts.is_empty());
    assert!(formula.deferred.is_empty());
    let terminal = output
        .payload
        .reserve
        .mode_expansions
        .iter()
        .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("BaseObjectMode"))
        .map(|(_, expansion)| &expansion.radix)
        .expect("Task 179 BaseObjectMode terminal expansion should exist");
    let (_, normalized) = output
        .term_formula
        .normalized_types()
        .iter()
        .next()
        .expect("Task 179 normalized object type should exist");
    assert_eq!(output.term_formula.normalized_types().len(), 1);
    assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
    assert_eq!(normalized.source.range, terminal.source_range);
    assert_eq!(normalized.source.spelling, terminal.spelling);

    let invalid_key = || {
        vec![
            TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY
                .to_owned(),
        ]
    };
    for removed in [
        "BaseObjectMode",
        "ChainObjectMode1",
        "ChainObjectMode2",
        "ChainObjectMode3",
        "ChainObjectMode4",
        "ChainObjectMode5",
        "ChainObjectMode6",
    ] {
        let mut invalid =
            source_local_object_mode_long_chain_reserved_variable_type_assertion_output(
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
                TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY,
            ),
            invalid_key()
        );
    }
    let mut wrong_asserted =
        source_local_object_mode_long_chain_reserved_variable_type_assertion_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce an asserted-input corruption target");
    wrong_asserted.asserted_type_input.head = TypeHeadInput::BuiltinSet;
    let wrong_asserted_result =
        assert_source_reserved_variable_type_assertion_output(&wrong_asserted)
            .map(|()| wrong_asserted);
    assert_eq!(
        source_reserved_variable_type_assertion_result_detail_keys(
            wrong_asserted_result,
            TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY,
        ),
        invalid_key()
    );
    let mut wrong_subject =
        source_local_object_mode_long_chain_reserved_variable_type_assertion_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce a raw-subject corruption target");
    wrong_subject.subject_result_input.spelling = "object".to_owned();
    let wrong_subject_result =
        assert_source_reserved_variable_type_assertion_output(&wrong_subject)
            .map(|()| wrong_subject);
    assert_eq!(
        source_reserved_variable_type_assertion_result_detail_keys(
            wrong_subject_result,
            TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY,
        ),
        invalid_key()
    );

    let mut wrong_terminal = exact_modes();
    wrong_terminal[0] = mode_definition_with_label(
        "BaseObjectMode",
        "BaseObjectModeDef",
        ReserveTypeShape::Builtin("set"),
    );
    let mut wrong_link = exact_modes();
    wrong_link[3] = mode_definition_with_label(
        "ChainObjectMode3",
        "ChainObjectMode3Def",
        ReserveTypeShape::QualifiedSymbol("BaseObjectMode"),
    );
    let mut argument_bearing_link = exact_modes();
    argument_bearing_link[6] = mode_definition_with_label(
        "ChainObjectMode6",
        "ChainObjectMode6Def",
        ReserveTypeShape::QualifiedSymbolWithArgs("ChainObjectMode5"),
    );
    for near_miss in [
        mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            wrong_terminal,
            reserve(),
            theorem,
        ),
        mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            wrong_link,
            reserve(),
            theorem,
        ),
        mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            argument_bearing_link,
            reserve(),
            theorem,
        ),
        mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            exact_modes(),
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbolWithArgs("ChainObjectMode6"),
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
                asserted_type: ReserveTypeShape::QualifiedSymbol("ChainObjectMode6"),
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
    let unresolved_symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
    assert_eq!(
        source_type_elaboration_detail_keys(&exact, module, &unresolved_symbols),
        vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
    );
}
#[test]
fn source_local_object_mode_long_chain_reserved_variable_equality_consumes_seven_expansions() {
    let source_id = source_id(176);
    let module = ResolverModuleId::new(
        PackageId::new("test"),
        ModulePath::new("local_object_mode_long_chain_reserved_variable_equality"),
    );
    let symbols = source_local_symbols_env(
        module.clone(),
        &[
            ("BaseObjectMode", SymbolKind::Mode),
            ("ChainObjectMode1", SymbolKind::Mode),
            ("ChainObjectMode2", SymbolKind::Mode),
            ("ChainObjectMode3", SymbolKind::Mode),
            ("ChainObjectMode4", SymbolKind::Mode),
            ("ChainObjectMode5", SymbolKind::Mode),
            ("ChainObjectMode6", SymbolKind::Mode),
            ("ExtraObjectMode", SymbolKind::Mode),
        ],
    );
    let theorem = IdentifierBinaryTheoremSpec {
        status: None,
        label: "LongLocalObjectModeReservedVariableEqualityPayloadBoundary",
        left: "z",
        operator: "=",
        right: "z",
        recovered_label: false,
    };
    let exact_modes = || {
        vec![
            mode_definition_with_label(
                "BaseObjectMode",
                "BaseObjectModeDef",
                ReserveTypeShape::Builtin("object"),
            ),
            mode_definition_with_label(
                "ChainObjectMode1",
                "ChainObjectMode1Def",
                ReserveTypeShape::QualifiedSymbol("BaseObjectMode"),
            ),
            mode_definition_with_label(
                "ChainObjectMode2",
                "ChainObjectMode2Def",
                ReserveTypeShape::QualifiedSymbol("ChainObjectMode1"),
            ),
            mode_definition_with_label(
                "ChainObjectMode3",
                "ChainObjectMode3Def",
                ReserveTypeShape::QualifiedSymbol("ChainObjectMode2"),
            ),
            mode_definition_with_label(
                "ChainObjectMode4",
                "ChainObjectMode4Def",
                ReserveTypeShape::QualifiedSymbol("ChainObjectMode3"),
            ),
            mode_definition_with_label(
                "ChainObjectMode5",
                "ChainObjectMode5Def",
                ReserveTypeShape::QualifiedSymbol("ChainObjectMode4"),
            ),
            mode_definition_with_label(
                "ChainObjectMode6",
                "ChainObjectMode6Def",
                ReserveTypeShape::QualifiedSymbol("ChainObjectMode5"),
            ),
        ]
    };
    let reserve = || {
        vec![reserve_item(
            vec!["z"],
            ReserveTypeShape::QualifiedSymbol("ChainObjectMode6"),
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
    let payload = extract_source_local_object_mode_long_chain_reserved_variable_equality(
        &exact,
        module.clone(),
        &symbols,
    )
    .expect("exact local-object-mode long-chain equality should extract");
    assert_eq!(payload.reserve.mode_expansions.len(), 7);
    assert_eq!(payload.reserve.bridge.bindings().len(), 1);
    assert_eq!(payload.reserve.bridge.bindings()[0].spelling, "z");
    assert_eq!(
        payload.reserve.bridge.bindings()[0].type_spelling,
        "ChainObjectMode6"
    );
    assert_eq!(payload.left_lookup_ordinal, 1);
    assert_eq!(payload.right_lookup_ordinal, 2);

    let output = source_local_object_mode_long_chain_reserved_variable_equality_output(
        &exact,
        module.clone(),
        &symbols,
    )
    .expect("exact long-chain object equality should reach TermFormulaChecker");
    assert_source_reserved_variable_formula_output(&output)
        .expect("long-chain object equality invariants should hold");
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
        assert_eq!(input.spelling, "ChainObjectMode6");
        assert!(matches!(input.head, TypeHeadInput::Symbol(_)));
        assert_eq!(
            input.source_range,
            output.payload.reserve.bridge.bindings()[0].type_range
        );
    }
    let terminal = output
        .payload
        .reserve
        .mode_expansions
        .iter()
        .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("BaseObjectMode"))
        .map(|(_, expansion)| &expansion.radix)
        .expect("BaseObjectMode terminal expansion should exist");
    let (_, normalized) = output
        .term_formula
        .normalized_types()
        .iter()
        .next()
        .expect("one normalized terminal object should exist");
    assert_eq!(output.term_formula.normalized_types().len(), 1);
    assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
    assert_eq!(normalized.source.range, terminal.source_range);
    assert_eq!(normalized.source.spelling, terminal.spelling);
    assert_ne!(
        output.payload.reserve.bridge.bindings()[0].type_range,
        terminal.source_range
    );
    assert_eq!(output.term_formula.terms().len(), 2);
    let (_, formula) = output
        .term_formula
        .formulas()
        .iter()
        .next()
        .expect("long-chain object equality should be checked");
    assert_eq!(formula.kind, FormulaKind::Equality);
    assert_eq!(formula.status, FormulaStatus::Checked);
    assert_eq!(formula.expected_types.len(), 2);
    assert_eq!(
        formula.expected_types[0].source_range,
        output.payload.left_range
    );
    assert_eq!(
        formula.expected_types[1].source_range,
        output.payload.right_range
    );
    assert!(formula.facts.is_empty());
    assert!(formula.deferred.is_empty());
    assert!(output.term_formula.facts().is_empty());
    assert!(output.term_formula.diagnostics().is_empty());

    let invalid_key = || {
        vec![
            TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY
                .to_owned(),
        ]
    };
    for removed in [
        "BaseObjectMode",
        "ChainObjectMode1",
        "ChainObjectMode2",
        "ChainObjectMode3",
        "ChainObjectMode4",
        "ChainObjectMode5",
        "ChainObjectMode6",
    ] {
        let mut invalid =
            source_local_object_mode_long_chain_reserved_variable_equality_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce a seven-expansion corruption target");
        invalid
            .payload
            .reserve
            .mode_expansions
            .retain(|symbol, _| source_mode_symbol_spelling(symbol) != Some(removed));
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&invalid),
            invalid_key()
        );
    }
    for clear_left in [true, false] {
        let mut invalid =
            source_local_object_mode_long_chain_reserved_variable_equality_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce an expected-role corruption target");
        if clear_left {
            invalid.left_expected_input = None;
        } else {
            invalid.right_expected_input = None;
        }
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&invalid),
            invalid_key()
        );
    }
    let mut coerced = source_local_object_mode_long_chain_reserved_variable_equality_output(
        &exact,
        module.clone(),
        &symbols,
    )
    .expect("exact source should produce an object/set corruption target");
    coerced
        .left_expected_input
        .as_mut()
        .expect("left expected input should exist")
        .head = TypeHeadInput::BuiltinSet;
    assert_eq!(
        source_reserved_variable_formula_output_detail_keys(&coerced),
        invalid_key()
    );

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
    let mut set_terminal = exact_modes();
    set_terminal[0].rhs_shape = ReserveTypeShape::Builtin("set");
    assert_extraction_gap(set_terminal);
    let mut wrong_link = exact_modes();
    wrong_link[6].rhs_shape = ReserveTypeShape::QualifiedSymbol("ChainObjectMode4");
    assert_extraction_gap(wrong_link);
    let mut argument_bearing = exact_modes();
    argument_bearing[6].rhs_shape =
        ReserveTypeShape::QualifiedSymbolWithArgs("ChainObjectMode5");
    assert_extraction_gap(argument_bearing);

    for near_miss in [
        mode_then_reserve_identifier_binary_theorem_ast(
            source_id,
            exact_modes(),
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbolWithArgs("ChainObjectMode6"),
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
fn source_local_object_mode_long_chain_reserved_variable_inequality_consumes_seven_expansions()
{
    let source_id = source_id(177);
    let module = ResolverModuleId::new(
        PackageId::new("test"),
        ModulePath::new("local_object_mode_long_chain_reserved_variable_inequality"),
    );
    let symbols = source_local_symbols_env(
        module.clone(),
        &[
            ("BaseObjectMode", SymbolKind::Mode),
            ("ChainObjectMode1", SymbolKind::Mode),
            ("ChainObjectMode2", SymbolKind::Mode),
            ("ChainObjectMode3", SymbolKind::Mode),
            ("ChainObjectMode4", SymbolKind::Mode),
            ("ChainObjectMode5", SymbolKind::Mode),
            ("ChainObjectMode6", SymbolKind::Mode),
            ("ExtraObjectMode", SymbolKind::Mode),
        ],
    );
    let theorem = IdentifierBinaryTheoremSpec {
        status: None,
        label: "LongLocalObjectModeReservedVariableInequalityPayloadBoundary",
        left: "z",
        operator: "<>",
        right: "z",
        recovered_label: false,
    };
    let exact_modes = || {
        vec![
            mode_definition_with_label(
                "BaseObjectMode",
                "BaseObjectModeDef",
                ReserveTypeShape::Builtin("object"),
            ),
            mode_definition_with_label(
                "ChainObjectMode1",
                "ChainObjectMode1Def",
                ReserveTypeShape::QualifiedSymbol("BaseObjectMode"),
            ),
            mode_definition_with_label(
                "ChainObjectMode2",
                "ChainObjectMode2Def",
                ReserveTypeShape::QualifiedSymbol("ChainObjectMode1"),
            ),
            mode_definition_with_label(
                "ChainObjectMode3",
                "ChainObjectMode3Def",
                ReserveTypeShape::QualifiedSymbol("ChainObjectMode2"),
            ),
            mode_definition_with_label(
                "ChainObjectMode4",
                "ChainObjectMode4Def",
                ReserveTypeShape::QualifiedSymbol("ChainObjectMode3"),
            ),
            mode_definition_with_label(
                "ChainObjectMode5",
                "ChainObjectMode5Def",
                ReserveTypeShape::QualifiedSymbol("ChainObjectMode4"),
            ),
            mode_definition_with_label(
                "ChainObjectMode6",
                "ChainObjectMode6Def",
                ReserveTypeShape::QualifiedSymbol("ChainObjectMode5"),
            ),
        ]
    };
    let reserve = || {
        vec![reserve_item(
            vec!["z"],
            ReserveTypeShape::QualifiedSymbol("ChainObjectMode6"),
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
    let payload = extract_source_local_object_mode_long_chain_reserved_variable_inequality(
        &exact,
        module.clone(),
        &symbols,
    )
    .expect("exact local-object-mode long-chain inequality should extract");
    assert_eq!(payload.reserve.mode_expansions.len(), 7);
    assert_eq!(payload.reserve.bridge.bindings().len(), 1);
    assert_eq!(payload.reserve.bridge.bindings()[0].spelling, "z");
    assert_eq!(
        payload.reserve.bridge.bindings()[0].type_spelling,
        "ChainObjectMode6"
    );
    assert_eq!(payload.left_lookup_ordinal, 1);
    assert_eq!(payload.right_lookup_ordinal, 2);

    let output = source_local_object_mode_long_chain_reserved_variable_inequality_output(
        &exact,
        module.clone(),
        &symbols,
    )
    .expect("exact long-chain object inequality should reach TermFormulaChecker");
    assert_source_reserved_variable_formula_output(&output)
        .expect("long-chain object inequality invariants should hold");
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
        assert_eq!(input.spelling, "ChainObjectMode6");
        assert!(matches!(input.head, TypeHeadInput::Symbol(_)));
        assert_eq!(
            input.source_range,
            output.payload.reserve.bridge.bindings()[0].type_range
        );
    }
    let terminal = output
        .payload
        .reserve
        .mode_expansions
        .iter()
        .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("BaseObjectMode"))
        .map(|(_, expansion)| &expansion.radix)
        .expect("BaseObjectMode terminal expansion should exist");
    let (_, normalized) = output
        .term_formula
        .normalized_types()
        .iter()
        .next()
        .expect("one normalized terminal object should exist");
    assert_eq!(output.term_formula.normalized_types().len(), 1);
    assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
    assert_eq!(normalized.source.range, terminal.source_range);
    assert_eq!(normalized.source.spelling, terminal.spelling);
    assert_ne!(
        output.payload.reserve.bridge.bindings()[0].type_range,
        terminal.source_range
    );
    assert_eq!(output.term_formula.terms().len(), 2);
    let (_, formula) = output
        .term_formula
        .formulas()
        .iter()
        .next()
        .expect("long-chain object inequality should be checked");
    assert_eq!(formula.kind, FormulaKind::Inequality);
    assert_eq!(formula.status, FormulaStatus::Checked);
    assert_eq!(formula.expected_types.len(), 2);
    assert_eq!(
        formula.expected_types[0].source_range,
        output.payload.left_range
    );
    assert_eq!(
        formula.expected_types[1].source_range,
        output.payload.right_range
    );
    assert!(formula.facts.is_empty());
    assert!(formula.deferred.is_empty());
    assert!(output.term_formula.facts().is_empty());
    assert!(output.term_formula.diagnostics().is_empty());

    let invalid_key = || {
        vec![
            TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY
                .to_owned(),
        ]
    };
    for removed in [
        "BaseObjectMode",
        "ChainObjectMode1",
        "ChainObjectMode2",
        "ChainObjectMode3",
        "ChainObjectMode4",
        "ChainObjectMode5",
        "ChainObjectMode6",
    ] {
        let mut invalid =
            source_local_object_mode_long_chain_reserved_variable_inequality_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce a seven-expansion corruption target");
        invalid
            .payload
            .reserve
            .mode_expansions
            .retain(|symbol, _| source_mode_symbol_spelling(symbol) != Some(removed));
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&invalid),
            invalid_key()
        );
    }
    for clear_left in [true, false] {
        let mut invalid =
            source_local_object_mode_long_chain_reserved_variable_inequality_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("exact source should produce an expected-role corruption target");
        if clear_left {
            invalid.left_expected_input = None;
        } else {
            invalid.right_expected_input = None;
        }
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&invalid),
            invalid_key()
        );
    }
    let mut coerced = source_local_object_mode_long_chain_reserved_variable_inequality_output(
        &exact,
        module.clone(),
        &symbols,
    )
    .expect("exact source should produce an object/set corruption target");
    coerced
        .left_expected_input
        .as_mut()
        .expect("left expected input should exist")
        .head = TypeHeadInput::BuiltinSet;
    assert_eq!(
        source_reserved_variable_formula_output_detail_keys(&coerced),
        invalid_key()
    );

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
    let mut set_terminal = exact_modes();
    set_terminal[0].rhs_shape = ReserveTypeShape::Builtin("set");
    assert_extraction_gap(set_terminal);
    let mut wrong_link = exact_modes();
    wrong_link[6].rhs_shape = ReserveTypeShape::QualifiedSymbol("ChainObjectMode4");
    assert_extraction_gap(wrong_link);
    let mut argument_bearing = exact_modes();
    argument_bearing[6].rhs_shape =
        ReserveTypeShape::QualifiedSymbolWithArgs("ChainObjectMode5");
    assert_extraction_gap(argument_bearing);

    for near_miss in [
        mode_then_reserve_identifier_binary_theorem_ast(
            source_id,
            exact_modes(),
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbolWithArgs("ChainObjectMode6"),
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
fn source_local_object_mode_long_chain_reserved_variable_membership_consumes_seven_expansions()
{
    let source_id = source_id(178);
    let module = ResolverModuleId::new(
        PackageId::new("test"),
        ModulePath::new("local_object_mode_long_chain_reserved_variable_membership"),
    );
    let symbols = source_local_symbols_env(
        module.clone(),
        &[
            ("BaseObjectMode", SymbolKind::Mode),
            ("ChainObjectMode1", SymbolKind::Mode),
            ("ChainObjectMode2", SymbolKind::Mode),
            ("ChainObjectMode3", SymbolKind::Mode),
            ("ChainObjectMode4", SymbolKind::Mode),
            ("ChainObjectMode5", SymbolKind::Mode),
            ("ChainObjectMode6", SymbolKind::Mode),
            ("ExtraObjectMode", SymbolKind::Mode),
        ],
    );
    let exact_modes = || {
        vec![
            mode_definition_with_label(
                "BaseObjectMode",
                "BaseObjectModeDef",
                ReserveTypeShape::Builtin("object"),
            ),
            mode_definition_with_label(
                "ChainObjectMode1",
                "ChainObjectMode1Def",
                ReserveTypeShape::QualifiedSymbol("BaseObjectMode"),
            ),
            mode_definition_with_label(
                "ChainObjectMode2",
                "ChainObjectMode2Def",
                ReserveTypeShape::QualifiedSymbol("ChainObjectMode1"),
            ),
            mode_definition_with_label(
                "ChainObjectMode3",
                "ChainObjectMode3Def",
                ReserveTypeShape::QualifiedSymbol("ChainObjectMode2"),
            ),
            mode_definition_with_label(
                "ChainObjectMode4",
                "ChainObjectMode4Def",
                ReserveTypeShape::QualifiedSymbol("ChainObjectMode3"),
            ),
            mode_definition_with_label(
                "ChainObjectMode5",
                "ChainObjectMode5Def",
                ReserveTypeShape::QualifiedSymbol("ChainObjectMode4"),
            ),
            mode_definition_with_label(
                "ChainObjectMode6",
                "ChainObjectMode6Def",
                ReserveTypeShape::QualifiedSymbol("ChainObjectMode5"),
            ),
        ]
    };
    let reserves = || {
        vec![
            reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("ChainObjectMode6"),
            ),
            reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
        ]
    };
    let theorem = IdentifierBinaryTheoremSpec {
        status: None,
        label: "LongLocalObjectModeReservedVariableMembershipPayloadBoundary",
        left: "x",
        operator: "in",
        right: "y",
        recovered_label: false,
    };
    let exact = mode_then_reserve_identifier_binary_theorem_ast(
        source_id,
        exact_modes(),
        reserves(),
        theorem,
    );
    assert_eq!(
        source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
        Vec::<String>::new()
    );

    let payload = extract_source_local_object_mode_long_chain_reserved_variable_membership(
        &exact,
        module.clone(),
        &symbols,
    )
    .expect("exact long-chain object membership should extract");
    assert_eq!(payload.reserve.mode_expansions.len(), 7);
    assert_eq!(payload.reserve.bridge.bindings().len(), 2);
    assert_eq!(payload.reserve.bridge.bindings()[0].spelling, "x");
    assert_eq!(
        payload.reserve.bridge.bindings()[0].type_spelling,
        "ChainObjectMode6"
    );
    assert_eq!(payload.reserve.bridge.bindings()[1].spelling, "y");
    assert_eq!(payload.reserve.bridge.bindings()[1].type_spelling, "set");
    assert_ne!(
        payload.reserve.bridge.bindings()[0].type_range,
        payload.reserve.bridge.bindings()[1].type_range
    );
    assert_eq!(payload.left_lookup_ordinal, 2);
    assert_eq!(payload.right_lookup_ordinal, 3);

    let output = source_local_object_mode_long_chain_reserved_variable_membership_output(
        &exact,
        module.clone(),
        &symbols,
    )
    .expect("exact long-chain object membership should reach TermFormulaChecker");
    assert_source_reserved_variable_formula_output(&output)
        .expect("long-chain object membership invariants should hold");
    assert_eq!(output.left_binding, BindingId::new(0));
    assert_eq!(output.right_binding, BindingId::new(1));
    assert_eq!(output.left_result_input.spelling, "ChainObjectMode6");
    assert!(matches!(
        output.left_result_input.head,
        TypeHeadInput::Symbol(_)
    ));
    assert_eq!(
        output.left_result_input.source_range,
        output.payload.reserve.bridge.bindings()[0].type_range
    );
    assert!(output.left_expected_input.is_none());
    assert_eq!(output.right_result_input.spelling, "set");
    assert_eq!(output.right_result_input.head, TypeHeadInput::BuiltinSet);
    let right_expected = output
        .right_expected_input
        .as_ref()
        .expect("right expected-set input should exist");
    assert_eq!(right_expected.spelling, "set");
    assert_eq!(right_expected.head, TypeHeadInput::BuiltinSet);
    assert_eq!(
        output.right_result_input.source_range,
        output.payload.reserve.bridge.bindings()[1].type_range
    );
    assert_eq!(
        right_expected.source_range,
        output.payload.reserve.bridge.bindings()[1].type_range
    );
    let terminal = output
        .payload
        .reserve
        .mode_expansions
        .iter()
        .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("BaseObjectMode"))
        .map(|(_, expansion)| &expansion.radix)
        .expect("BaseObjectMode terminal expansion should exist");
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
    let (_, formula) = output
        .term_formula
        .formulas()
        .iter()
        .next()
        .expect("long-chain object membership should be checked");
    assert_eq!(formula.kind, FormulaKind::Membership);
    assert_eq!(formula.status, FormulaStatus::Checked);
    assert_eq!(formula.expected_types.len(), 1);
    assert_eq!(formula.expected_types[0].term, output.payload.right_site);
    assert_eq!(
        formula.expected_types[0].source_range,
        output.payload.right_range
    );
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
            "long-local-object-mode-reserved-variable-membership-left-result".to_owned(),
            "long-local-object-mode-reserved-variable-membership-right-expected".to_owned(),
            "long-local-object-mode-reserved-variable-membership-right-result".to_owned(),
        ])
    );

    let invalid_key = || {
        vec![
            TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY
                .to_owned(),
        ]
    };
    for removed in [
        "BaseObjectMode",
        "ChainObjectMode1",
        "ChainObjectMode2",
        "ChainObjectMode3",
        "ChainObjectMode4",
        "ChainObjectMode5",
        "ChainObjectMode6",
    ] {
        let mut invalid =
            source_local_object_mode_long_chain_reserved_variable_membership_output(
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
        assert_eq!(
            source_reserved_variable_formula_output_detail_keys(&invalid),
            invalid_key()
        );
    }
    let mut unexpected_left =
        source_local_object_mode_long_chain_reserved_variable_membership_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce a left-expected corruption target");
    unexpected_left.left_expected_input = unexpected_left.right_expected_input.clone();
    assert_eq!(
        source_reserved_variable_formula_output_detail_keys(&unexpected_left),
        invalid_key()
    );
    let mut missing_right =
        source_local_object_mode_long_chain_reserved_variable_membership_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce a missing-right corruption target");
    missing_right.right_expected_input = None;
    assert_eq!(
        source_reserved_variable_formula_output_detail_keys(&missing_right),
        invalid_key()
    );
    let mut wrong_right =
        source_local_object_mode_long_chain_reserved_variable_membership_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact source should produce a right-type corruption target");
    wrong_right
        .right_expected_input
        .as_mut()
        .expect("right expected input should exist")
        .head = TypeHeadInput::BuiltinObject;
    assert_eq!(
        source_reserved_variable_formula_output_detail_keys(&wrong_right),
        invalid_key()
    );

    let assert_extraction_gap = |modes| {
        let near_miss = mode_then_reserve_identifier_binary_theorem_ast(
            source_id,
            modes,
            reserves(),
            theorem,
        );
        assert_eq!(
            source_type_elaboration_detail_keys(&near_miss, module.clone(), &symbols),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );
    };
    let mut set_terminal = exact_modes();
    set_terminal[0].rhs_shape = ReserveTypeShape::Builtin("set");
    assert_extraction_gap(set_terminal);
    let mut wrong_link = exact_modes();
    wrong_link[6].rhs_shape = ReserveTypeShape::QualifiedSymbol("ChainObjectMode4");
    assert_extraction_gap(wrong_link);
    let mut argument_bearing = exact_modes();
    argument_bearing[6].rhs_shape =
        ReserveTypeShape::QualifiedSymbolWithArgs("ChainObjectMode5");
    assert_extraction_gap(argument_bearing);

    for near_miss in [
        mode_then_reserve_identifier_binary_theorem_ast(
            source_id,
            exact_modes(),
            vec![
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbol("ChainObjectMode6"),
                ),
            ],
            theorem,
        ),
        mode_then_reserve_identifier_binary_theorem_ast(
            source_id,
            exact_modes(),
            vec![
                reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbolWithArgs("ChainObjectMode6"),
                ),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
            ],
            theorem,
        ),
        mode_then_reserve_identifier_binary_theorem_ast(
            source_id,
            exact_modes(),
            reserves(),
            IdentifierBinaryTheoremSpec {
                label: "OtherPayloadBoundary",
                ..theorem
            },
        ),
        mode_then_reserve_identifier_binary_theorem_ast(
            source_id,
            exact_modes(),
            reserves(),
            IdentifierBinaryTheoremSpec {
                left: "y",
                ..theorem
            },
        ),
        mode_then_reserve_identifier_binary_theorem_ast(
            source_id,
            exact_modes(),
            reserves(),
            IdentifierBinaryTheoremSpec {
                right: "x",
                ..theorem
            },
        ),
        mode_then_reserve_identifier_binary_theorem_ast(
            source_id,
            exact_modes(),
            reserves(),
            IdentifierBinaryTheoremSpec {
                operator: "=",
                ..theorem
            },
        ),
        mode_then_reserve_identifier_binary_theorem_ast(
            source_id,
            exact_modes(),
            reserves(),
            IdentifierBinaryTheoremSpec {
                status: Some("open"),
                ..theorem
            },
        ),
        mode_then_reserve_identifier_binary_theorem_ast(
            source_id,
            exact_modes(),
            reserves(),
            IdentifierBinaryTheoremSpec {
                recovered_label: true,
                ..theorem
            },
        ),
        modes_then_empty_definition_reserve_identifier_binary_theorem_ast(
            source_id,
            exact_modes(),
            reserves(),
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
