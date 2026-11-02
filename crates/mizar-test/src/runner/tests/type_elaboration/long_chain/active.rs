#[test]
fn active_local_mode_long_chain_reserved_variable_equality_fixture_consumes_seven_expansions() {
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
                == "pass_type_elaboration_local_mode_long_chain_reserved_variable_equality_001"
        })
        .expect("Task 172 active fixture should be discoverable");
    let frontend = run_frontend(&workspace_root, case, ordinal)
        .expect("Task 172 fixture should run through the real frontend");
    assert!(frontend.diagnostics.is_empty());
    let ast = frontend
        .ast
        .expect("Task 172 fixture should produce an AST");
    let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
    assert!(resolver.detail_keys.is_empty());
    let symbols =
        augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
    let output = source_local_mode_long_chain_reserved_variable_equality_output(
        &ast,
        resolver.module,
        &symbols,
    )
    .expect("Task 172 real AST should reach the long-chain equality seam");
    assert_source_reserved_variable_formula_output(&output)
        .expect("Task 172 real AST should preserve every checked payload invariant");
    assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
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
        .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("BaseMode"))
        .map(|(_, expansion)| &expansion.radix)
        .expect("Task 172 BaseMode terminal expansion should exist");
    let (_, normalized) = output
        .term_formula
        .normalized_types()
        .iter()
        .next()
        .expect("Task 172 normalized set type should exist");
    assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
    assert_eq!(normalized.source.range, terminal.source_range);
    assert_eq!(normalized.source.spelling, terminal.spelling);
}
#[test]
fn active_local_mode_long_chain_reserved_variable_inequality_fixture_consumes_seven_expansions()
{
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
    let (ordinal, case) = active_type_elaboration_cases(&plan).enumerate()
        .find(|(_, case)| case.id.0 == "pass_type_elaboration_local_mode_long_chain_reserved_variable_inequality_001")
        .expect("Task 173 active fixture should be discoverable");
    let frontend = run_frontend(&workspace_root, case, ordinal)
        .expect("Task 173 fixture should run through the real frontend");
    assert!(frontend.diagnostics.is_empty());
    let ast = frontend
        .ast
        .expect("Task 173 fixture should produce an AST");
    let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
    assert!(resolver.detail_keys.is_empty());
    let symbols =
        augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
    let output = source_local_mode_long_chain_reserved_variable_inequality_output(
        &ast,
        resolver.module,
        &symbols,
    )
    .expect("Task 173 real AST should reach long-chain inequality");
    assert_source_reserved_variable_formula_output(&output).unwrap();
    assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
    let terminal = output
        .payload
        .reserve
        .mode_expansions
        .iter()
        .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("BaseMode"))
        .map(|(_, expansion)| &expansion.radix)
        .unwrap();
    let (_, normalized) = output
        .term_formula
        .normalized_types()
        .iter()
        .next()
        .unwrap();
    assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
    assert_eq!(normalized.source.range, terminal.source_range);
}
#[test]
fn active_local_mode_long_chain_reserved_variable_membership_fixture_consumes_seven_expansions()
{
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
                == "pass_type_elaboration_local_mode_long_chain_reserved_variable_membership_001"
        })
        .expect("Task 174 active fixture should be discoverable");
    let frontend = run_frontend(&workspace_root, case, ordinal)
        .expect("Task 174 fixture should run through the real frontend");
    assert!(frontend.diagnostics.is_empty());
    let ast = frontend
        .ast
        .expect("Task 174 fixture should produce an AST");
    let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
    assert!(resolver.detail_keys.is_empty());
    let symbols =
        augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
    let output = source_local_mode_long_chain_reserved_variable_membership_output(
        &ast,
        resolver.module,
        &symbols,
    )
    .expect("Task 174 real AST should reach long-chain membership");
    assert_source_reserved_variable_formula_output(&output)
        .expect("Task 174 real AST should preserve every checked payload invariant");
    assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
    assert_eq!(output.left_binding, BindingId::new(0));
    assert_eq!(output.right_binding, BindingId::new(1));
    assert_eq!(output.payload.left_lookup_ordinal, 2);
    assert_eq!(output.payload.right_lookup_ordinal, 3);
    assert!(output.left_expected_input.is_none());
    let (_, formula) = output
        .term_formula
        .formulas()
        .iter()
        .next()
        .expect("Task 174 membership formula should be checked");
    assert_eq!(formula.kind, FormulaKind::Membership);
    assert_eq!(formula.status, FormulaStatus::Checked);
    assert_eq!(formula.expected_types.len(), 1);
    assert!(formula.facts.is_empty());
    assert!(formula.deferred.is_empty());
    let terminal = output
        .payload
        .reserve
        .mode_expansions
        .iter()
        .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("BaseMode"))
        .map(|(_, expansion)| &expansion.radix)
        .expect("Task 174 BaseMode terminal expansion should exist");
    let (_, normalized) = output
        .term_formula
        .normalized_types()
        .iter()
        .next()
        .expect("Task 174 normalized set type should exist");
    assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
    assert_eq!(normalized.source.range, terminal.source_range);
    assert_eq!(normalized.source.spelling, terminal.spelling);
}
#[test]
fn active_local_mode_long_chain_reserved_variable_type_assertion_fixture_consumes_seven_expansions()
 {
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
                == "pass_type_elaboration_local_mode_long_chain_reserved_variable_type_assertion_001"
        })
        .expect("Task 175 active fixture should be discoverable");
    let frontend = run_frontend(&workspace_root, case, ordinal)
        .expect("Task 175 fixture should run through the real frontend");
    assert!(frontend.diagnostics.is_empty());
    let ast = frontend
        .ast
        .expect("Task 175 fixture should produce an AST");
    let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
    assert!(resolver.detail_keys.is_empty());
    let symbols =
        augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
    let output = source_local_mode_long_chain_reserved_variable_type_assertion_output(
        &ast,
        resolver.module,
        &symbols,
    )
    .expect("Task 175 real AST should reach long-chain type assertion");
    assert_source_reserved_variable_type_assertion_output(&output)
        .expect("Task 175 real AST should preserve every checked payload invariant");
    assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
    assert_eq!(output.subject_binding, BindingId::new(0));
    assert_eq!(output.payload.subject_lookup_ordinal, 1);
    assert_eq!(output.subject_result_input.spelling, "ChainMode6");
    assert!(matches!(
        output.subject_result_input.head,
        TypeHeadInput::Symbol(_)
    ));
    assert_eq!(output.asserted_type_input.spelling, "set");
    assert_eq!(output.asserted_type_input.head, TypeHeadInput::BuiltinSet);
    assert_ne!(
        output.subject_result_input.source_range,
        output.asserted_type_input.source_range
    );
    let (_, formula) = output
        .term_formula
        .formulas()
        .iter()
        .next()
        .expect("Task 175 type assertion should be checked");
    assert_eq!(formula.kind, FormulaKind::TypeAssertion);
    assert_eq!(formula.status, FormulaStatus::Checked);
    assert!(formula.facts.is_empty());
    assert!(formula.deferred.is_empty());
    let terminal = output
        .payload
        .reserve
        .mode_expansions
        .iter()
        .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("BaseMode"))
        .map(|(_, expansion)| &expansion.radix)
        .expect("Task 175 BaseMode terminal expansion should exist");
    let (_, normalized) = output
        .term_formula
        .normalized_types()
        .iter()
        .next()
        .expect("Task 175 normalized set type should exist");
    assert_eq!(output.term_formula.normalized_types().len(), 1);
    assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
    assert_eq!(normalized.source.range, terminal.source_range);
    assert_eq!(normalized.source.spelling, terminal.spelling);
}

#[test]
fn active_local_mode_long_chain_three_hop_asserted_head_fixture_consumes_seven_expansions() {
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
    let plan = build_test_plan(&config).expect("Task 226 repository plan should build");
    let (ordinal, case) = active_type_elaboration_cases(&plan)
        .enumerate()
        .find(|(_, case)| {
            case.id.0
                == "pass_type_elaboration_local_mode_long_chain_three_hop_asserted_head_001"
        })
        .expect("Task 226 active fixture should be discoverable");
    let frontend = run_frontend(&workspace_root, case, ordinal)
        .expect("Task 226 fixture should run through the real frontend");
    assert!(frontend.diagnostics.is_empty());
    let ast = frontend
        .ast
        .expect("Task 226 fixture should produce a real AST");
    let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
    assert!(resolver.detail_keys.is_empty());
    let symbols =
        augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
    let output = super::source_local_mode_long_chain_three_hop_asserted_head_output(
        &ast,
        resolver.module.clone(),
        &symbols,
    )
    .expect("Task 226 real AST should reach the seven-expansion three-hop seam");
    assert_source_reserved_variable_type_assertion_output(&output)
        .expect("Task 226 real AST should preserve every checked payload invariant");
    assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
    assert_eq!(output.subject_binding, BindingId::new(0));
    assert_eq!(output.payload.subject_lookup_ordinal, 1);
    assert_eq!(output.subject_result_input.spelling, "ChainMode6");
    assert_eq!(output.asserted_type_input.spelling, "ChainMode3");
    assert_ne!(
        output.subject_result_input.head,
        output.asserted_type_input.head
    );
    assert_eq!(output.term_formula.type_entries().len(), 3);
    assert_eq!(output.term_formula.normalized_types().len(), 1);
    assert!(output.term_formula.candidate_sets().is_empty());
    assert!(output.term_formula.facts().is_empty());
    assert!(output.term_formula.diagnostics().is_empty());
    let chain6 = output
        .payload
        .reserve
        .mode_expansions
        .iter()
        .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("ChainMode6"))
        .map(|(_, expansion)| expansion)
        .expect("Task 226 ChainMode6 expansion should exist");
    assert_eq!(chain6.radix.spelling, "ChainMode5");
    let TypeHeadInput::Symbol(chain5_symbol) = &chain6.radix.head else {
        panic!("Task 226 ChainMode5 first intermediate should be a symbol");
    };
    let chain5 = output
        .payload
        .reserve
        .mode_expansions
        .get(chain5_symbol)
        .expect("Task 226 ChainMode5 expansion should exist");
    assert_eq!(chain5.radix.spelling, "ChainMode4");
    assert_ne!(chain5.radix.head, output.asserted_type_input.head);
    let TypeHeadInput::Symbol(chain4_symbol) = &chain5.radix.head else {
        panic!("Task 226 ChainMode4 second intermediate should be a symbol");
    };
    let chain4 = output
        .payload
        .reserve
        .mode_expansions
        .get(chain4_symbol)
        .expect("Task 226 ChainMode4 expansion should exist");
    assert_eq!(chain4.radix.spelling, "ChainMode3");
    assert_eq!(chain4.radix.head, output.asserted_type_input.head);

    type OwnerExtractor = fn(
        &SurfaceAst,
        ResolverModuleId,
        &SymbolEnv,
    ) -> Option<super::SourceReservedVariableTypeAssertion>;
    let prior_extractors: [(&str, OwnerExtractor); 50] = [
        (
            "extract_source_reserved_variable_type_assertion",
            super::extract_source_reserved_variable_type_assertion,
        ),
        (
            "extract_source_reserved_object_variable_type_assertion",
            super::extract_source_reserved_object_variable_type_assertion,
        ),
        (
            "extract_source_local_mode_reserved_variable_type_assertion",
            super::extract_source_local_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_local_object_mode_reserved_variable_type_assertion",
            super::extract_source_local_object_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_local_mode_asserted_head",
            super::extract_source_local_mode_asserted_head,
        ),
        (
            "extract_source_local_object_mode_asserted_head",
            super::extract_source_local_object_mode_asserted_head,
        ),
        (
            "extract_source_chained_local_mode_reserved_variable_type_assertion",
            super::extract_source_chained_local_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_chained_local_object_mode_reserved_variable_type_assertion",
            super::extract_source_chained_local_object_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_chained_local_mode_asserted_head",
            super::extract_source_chained_local_mode_asserted_head,
        ),
        (
            "extract_source_chained_local_object_mode_asserted_head",
            super::extract_source_chained_local_object_mode_asserted_head,
        ),
        (
            "extract_source_chained_local_mode_radix_asserted_head",
            super::extract_source_chained_local_mode_radix_asserted_head,
        ),
        (
            "extract_source_chained_local_object_mode_radix_asserted_head",
            super::extract_source_chained_local_object_mode_radix_asserted_head,
        ),
        (
            "extract_source_two_edge_local_mode_reserved_variable_type_assertion",
            super::extract_source_two_edge_local_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_two_edge_local_object_mode_reserved_variable_type_assertion",
            super::extract_source_two_edge_local_object_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_two_edge_local_mode_asserted_head",
            super::extract_source_two_edge_local_mode_asserted_head,
        ),
        (
            "extract_source_two_edge_local_object_mode_asserted_head",
            super::extract_source_two_edge_local_object_mode_asserted_head,
        ),
        (
            "extract_source_two_edge_local_mode_radix_asserted_head",
            super::extract_source_two_edge_local_mode_radix_asserted_head,
        ),
        (
            "extract_source_two_edge_local_object_mode_radix_asserted_head",
            super::extract_source_two_edge_local_object_mode_radix_asserted_head,
        ),
        (
            "extract_source_three_edge_local_mode_reserved_variable_type_assertion",
            super::extract_source_three_edge_local_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_three_edge_local_object_mode_reserved_variable_type_assertion",
            super::extract_source_three_edge_local_object_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_three_edge_local_mode_asserted_head",
            super::extract_source_three_edge_local_mode_asserted_head,
        ),
        (
            "extract_source_three_edge_local_object_mode_asserted_head",
            super::extract_source_three_edge_local_object_mode_asserted_head,
        ),
        (
            "extract_source_three_edge_local_mode_radix_asserted_head",
            super::extract_source_three_edge_local_mode_radix_asserted_head,
        ),
        (
            "extract_source_three_edge_local_object_mode_radix_asserted_head",
            super::extract_source_three_edge_local_object_mode_radix_asserted_head,
        ),
        (
            "extract_source_four_edge_local_mode_reserved_variable_type_assertion",
            super::extract_source_four_edge_local_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_four_edge_local_object_mode_reserved_variable_type_assertion",
            super::extract_source_four_edge_local_object_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_four_edge_local_mode_asserted_head",
            super::extract_source_four_edge_local_mode_asserted_head,
        ),
        (
            "extract_source_four_edge_local_object_mode_asserted_head",
            super::extract_source_four_edge_local_object_mode_asserted_head,
        ),
        (
            "extract_source_four_edge_local_mode_radix_asserted_head",
            super::extract_source_four_edge_local_mode_radix_asserted_head,
        ),
        (
            "extract_source_four_edge_local_object_mode_radix_asserted_head",
            super::extract_source_four_edge_local_object_mode_radix_asserted_head,
        ),
        (
            "extract_source_local_mode_long_chain_reserved_variable_type_assertion",
            super::extract_source_local_mode_long_chain_reserved_variable_type_assertion,
        ),
        (
            "extract_source_local_object_mode_long_chain_reserved_variable_type_assertion",
            super::extract_source_local_object_mode_long_chain_reserved_variable_type_assertion,
        ),
        (
            "extract_source_local_mode_long_chain_asserted_head",
            super::extract_source_local_mode_long_chain_asserted_head,
        ),
        (
            "extract_source_local_object_mode_long_chain_asserted_head",
            super::extract_source_local_object_mode_long_chain_asserted_head,
        ),
        (
            "extract_source_local_mode_long_chain_radix_asserted_head",
            super::extract_source_local_mode_long_chain_radix_asserted_head,
        ),
        (
            "extract_source_local_object_mode_long_chain_radix_asserted_head",
            super::extract_source_local_object_mode_long_chain_radix_asserted_head,
        ),
        (
            "extract_source_local_mode_long_chain_two_hop_asserted_head",
            super::extract_source_local_mode_long_chain_two_hop_asserted_head,
        ),
        (
            "extract_source_local_object_mode_long_chain_two_hop_asserted_head",
            super::extract_source_local_object_mode_long_chain_two_hop_asserted_head,
        ),
        (
            "extract_source_two_edge_local_mode_two_hop_asserted_head",
            super::extract_source_two_edge_local_mode_two_hop_asserted_head,
        ),
        (
            "extract_source_two_edge_local_object_mode_two_hop_asserted_head",
            super::extract_source_two_edge_local_object_mode_two_hop_asserted_head,
        ),
        (
            "extract_source_three_edge_local_mode_two_hop_asserted_head",
            super::extract_source_three_edge_local_mode_two_hop_asserted_head,
        ),
        (
            "extract_source_three_edge_local_object_mode_two_hop_asserted_head",
            super::extract_source_three_edge_local_object_mode_two_hop_asserted_head,
        ),
        (
            "extract_source_four_edge_local_mode_two_hop_asserted_head",
            super::extract_source_four_edge_local_mode_two_hop_asserted_head,
        ),
        (
            "extract_source_four_edge_local_object_mode_two_hop_asserted_head",
            super::extract_source_four_edge_local_object_mode_two_hop_asserted_head,
        ),
        (
            "extract_source_three_edge_local_mode_three_hop_asserted_head",
            super::extract_source_three_edge_local_mode_three_hop_asserted_head,
        ),
        (
            "extract_source_three_edge_local_object_mode_three_hop_asserted_head",
            super::extract_source_three_edge_local_object_mode_three_hop_asserted_head,
        ),
        (
            "extract_source_four_edge_local_mode_three_hop_asserted_head",
            super::extract_source_four_edge_local_mode_three_hop_asserted_head,
        ),
        (
            "extract_source_four_edge_local_object_mode_three_hop_asserted_head",
            super::extract_source_four_edge_local_object_mode_three_hop_asserted_head,
        ),
        (
            "extract_source_four_edge_local_mode_four_hop_asserted_head",
            super::extract_source_four_edge_local_mode_four_hop_asserted_head,
        ),
        (
            "extract_source_four_edge_local_object_mode_four_hop_asserted_head",
            super::extract_source_four_edge_local_object_mode_four_hop_asserted_head,
        ),
    ];
    for (owner, extractor) in prior_extractors {
        assert!(
            extractor(&ast, resolver.module.clone(), &symbols).is_none(),
            "Task 226 source must not be owned by {owner}"
        );
    }

    let prior_owner_ids = [
        "pass_type_elaboration_reserved_variable_type_assertion_001",
        "pass_type_elaboration_reserved_object_variable_type_assertion_001",
        "pass_type_elaboration_local_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_local_object_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_local_mode_asserted_head_001",
        "pass_type_elaboration_local_object_mode_asserted_head_001",
        "pass_type_elaboration_chained_local_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_chained_local_object_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_chained_local_mode_asserted_head_001",
        "pass_type_elaboration_chained_local_object_mode_asserted_head_001",
        "pass_type_elaboration_chained_local_mode_radix_asserted_head_001",
        "pass_type_elaboration_chained_local_object_mode_radix_asserted_head_001",
        "pass_type_elaboration_two_edge_local_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_two_edge_local_object_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_two_edge_local_mode_asserted_head_001",
        "pass_type_elaboration_two_edge_local_object_mode_asserted_head_001",
        "pass_type_elaboration_two_edge_local_mode_radix_asserted_head_001",
        "pass_type_elaboration_two_edge_local_object_mode_radix_asserted_head_001",
        "pass_type_elaboration_three_edge_local_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_three_edge_local_object_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_three_edge_local_mode_asserted_head_001",
        "pass_type_elaboration_three_edge_local_object_mode_asserted_head_001",
        "pass_type_elaboration_three_edge_local_mode_radix_asserted_head_001",
        "pass_type_elaboration_three_edge_local_object_mode_radix_asserted_head_001",
        "pass_type_elaboration_four_edge_local_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_four_edge_local_object_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_four_edge_local_mode_asserted_head_001",
        "pass_type_elaboration_four_edge_local_object_mode_asserted_head_001",
        "pass_type_elaboration_four_edge_local_mode_radix_asserted_head_001",
        "pass_type_elaboration_four_edge_local_object_mode_radix_asserted_head_001",
        "pass_type_elaboration_local_mode_long_chain_reserved_variable_type_assertion_001",
        "pass_type_elaboration_local_object_mode_long_chain_reserved_variable_type_assertion_001",
        "pass_type_elaboration_local_mode_long_chain_asserted_head_001",
        "pass_type_elaboration_local_object_mode_long_chain_asserted_head_001",
        "pass_type_elaboration_local_mode_long_chain_radix_asserted_head_001",
        "pass_type_elaboration_local_object_mode_long_chain_radix_asserted_head_001",
        "pass_type_elaboration_local_mode_long_chain_two_hop_asserted_head_001",
        "pass_type_elaboration_local_object_mode_long_chain_two_hop_asserted_head_001",
        "pass_type_elaboration_two_edge_local_mode_two_hop_asserted_head_001",
        "pass_type_elaboration_two_edge_local_object_mode_two_hop_asserted_head_001",
        "pass_type_elaboration_three_edge_local_mode_two_hop_asserted_head_001",
        "pass_type_elaboration_three_edge_local_object_mode_two_hop_asserted_head_001",
        "pass_type_elaboration_four_edge_local_mode_two_hop_asserted_head_001",
        "pass_type_elaboration_four_edge_local_object_mode_two_hop_asserted_head_001",
        "pass_type_elaboration_three_edge_local_mode_three_hop_asserted_head_001",
        "pass_type_elaboration_three_edge_local_object_mode_three_hop_asserted_head_001",
        "pass_type_elaboration_four_edge_local_mode_three_hop_asserted_head_001",
        "pass_type_elaboration_four_edge_local_object_mode_three_hop_asserted_head_001",
        "pass_type_elaboration_four_edge_local_mode_four_hop_asserted_head_001",
        "pass_type_elaboration_four_edge_local_object_mode_four_hop_asserted_head_001",
    ];
    assert_eq!(prior_owner_ids.len(), 50);
    for owner_id in prior_owner_ids {
        let (owner_ordinal, owner_case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| case.id.0 == owner_id)
            .unwrap_or_else(|| panic!("prior owner fixture {owner_id} must be active"));
        let owner_frontend = run_frontend(&workspace_root, owner_case, owner_ordinal)
            .unwrap_or_else(|error| panic!("owner fixture {owner_id} must parse: {error}"));
        assert!(owner_frontend.diagnostics.is_empty());
        let owner_ast = owner_frontend
            .ast
            .unwrap_or_else(|| panic!("owner fixture {owner_id} must produce an AST"));
        let owner_resolver =
            resolver_symbol_collection(&workspace_root, owner_case, &owner_ast);
        assert!(owner_resolver.detail_keys.is_empty());
        let owner_symbols = augment_type_elaboration_import_summaries(
            &owner_ast,
            &owner_resolver.module,
            owner_resolver.env,
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &owner_ast,
                owner_resolver.module.clone(),
                &owner_symbols,
            ),
            Vec::<String>::new(),
            "prior owner fixture {owner_id} must still reach its active route"
        );
        assert!(
            super::extract_source_local_mode_long_chain_three_hop_asserted_head(
                &owner_ast,
                owner_resolver.module,
                &owner_symbols,
            )
            .is_none(),
            "Task 226 must reject prior owner fixture {owner_id}"
        );
    }
}

#[test]
fn active_local_mode_long_chain_four_hop_asserted_head_fixture_consumes_seven_expansions() {
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
    let plan = build_test_plan(&config).expect("Task 228 repository plan should build");
    let (ordinal, case) = active_type_elaboration_cases(&plan)
        .enumerate()
        .find(|(_, case)| {
            case.id.0
                == "pass_type_elaboration_local_mode_long_chain_four_hop_asserted_head_001"
        })
        .expect("Task 228 active fixture should be discoverable");
    let frontend = run_frontend(&workspace_root, case, ordinal)
        .expect("Task 228 fixture should run through the real frontend");
    assert!(frontend.diagnostics.is_empty());
    let ast = frontend
        .ast
        .expect("Task 228 fixture should produce a real AST");
    let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
    assert!(resolver.detail_keys.is_empty());
    let symbols =
        augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
    let output = super::source_local_mode_long_chain_four_hop_asserted_head_output(
        &ast,
        resolver.module.clone(),
        &symbols,
    )
    .expect("Task 228 real AST should reach the seven-expansion four-hop seam");
    assert_source_reserved_variable_type_assertion_output(&output)
        .expect("Task 228 real AST should preserve every checked payload invariant");
    assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
    assert_eq!(output.subject_binding, BindingId::new(0));
    assert_eq!(output.payload.subject_lookup_ordinal, 1);
    assert_eq!(output.subject_result_input.spelling, "ChainMode6");
    assert_eq!(output.asserted_type_input.spelling, "ChainMode2");
    assert_ne!(
        output.subject_result_input.head,
        output.asserted_type_input.head
    );
    assert_eq!(output.term_formula.type_entries().len(), 3);
    assert_eq!(output.term_formula.normalized_types().len(), 1);
    assert!(output.term_formula.candidate_sets().is_empty());
    assert!(output.term_formula.facts().is_empty());
    assert!(output.term_formula.diagnostics().is_empty());
    let chain6 = output
        .payload
        .reserve
        .mode_expansions
        .iter()
        .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("ChainMode6"))
        .map(|(_, expansion)| expansion)
        .expect("Task 228 ChainMode6 expansion should exist");
    assert_eq!(chain6.radix.spelling, "ChainMode5");
    let TypeHeadInput::Symbol(chain5_symbol) = &chain6.radix.head else {
        panic!("Task 228 ChainMode5 first intermediate should be a symbol");
    };
    let chain5 = output
        .payload
        .reserve
        .mode_expansions
        .get(chain5_symbol)
        .expect("Task 228 ChainMode5 expansion should exist");
    assert_eq!(chain5.radix.spelling, "ChainMode4");
    assert_ne!(chain5.radix.head, output.asserted_type_input.head);
    let TypeHeadInput::Symbol(chain4_symbol) = &chain5.radix.head else {
        panic!("Task 228 ChainMode4 second intermediate should be a symbol");
    };
    let chain4 = output
        .payload
        .reserve
        .mode_expansions
        .get(chain4_symbol)
        .expect("Task 228 ChainMode4 expansion should exist");
    assert_eq!(chain4.radix.spelling, "ChainMode3");
    assert_ne!(chain4.radix.head, output.asserted_type_input.head);
    let TypeHeadInput::Symbol(chain3_symbol) = &chain4.radix.head else {
        panic!("Task 228 ChainMode3 third intermediate should be a symbol");
    };
    let chain3 = output
        .payload
        .reserve
        .mode_expansions
        .get(chain3_symbol)
        .expect("Task 228 ChainMode3 expansion should exist");
    assert_eq!(chain3.radix.spelling, "ChainMode2");
    assert_eq!(chain3.radix.head, output.asserted_type_input.head);

    type OwnerExtractor = fn(
        &SurfaceAst,
        ResolverModuleId,
        &SymbolEnv,
    ) -> Option<super::SourceReservedVariableTypeAssertion>;
    let prior_extractors: [(&str, OwnerExtractor); 52] = [
        (
            "extract_source_reserved_variable_type_assertion",
            super::extract_source_reserved_variable_type_assertion,
        ),
        (
            "extract_source_reserved_object_variable_type_assertion",
            super::extract_source_reserved_object_variable_type_assertion,
        ),
        (
            "extract_source_local_mode_reserved_variable_type_assertion",
            super::extract_source_local_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_local_object_mode_reserved_variable_type_assertion",
            super::extract_source_local_object_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_local_mode_asserted_head",
            super::extract_source_local_mode_asserted_head,
        ),
        (
            "extract_source_local_object_mode_asserted_head",
            super::extract_source_local_object_mode_asserted_head,
        ),
        (
            "extract_source_chained_local_mode_reserved_variable_type_assertion",
            super::extract_source_chained_local_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_chained_local_object_mode_reserved_variable_type_assertion",
            super::extract_source_chained_local_object_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_chained_local_mode_asserted_head",
            super::extract_source_chained_local_mode_asserted_head,
        ),
        (
            "extract_source_chained_local_object_mode_asserted_head",
            super::extract_source_chained_local_object_mode_asserted_head,
        ),
        (
            "extract_source_chained_local_mode_radix_asserted_head",
            super::extract_source_chained_local_mode_radix_asserted_head,
        ),
        (
            "extract_source_chained_local_object_mode_radix_asserted_head",
            super::extract_source_chained_local_object_mode_radix_asserted_head,
        ),
        (
            "extract_source_two_edge_local_mode_reserved_variable_type_assertion",
            super::extract_source_two_edge_local_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_two_edge_local_object_mode_reserved_variable_type_assertion",
            super::extract_source_two_edge_local_object_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_two_edge_local_mode_asserted_head",
            super::extract_source_two_edge_local_mode_asserted_head,
        ),
        (
            "extract_source_two_edge_local_object_mode_asserted_head",
            super::extract_source_two_edge_local_object_mode_asserted_head,
        ),
        (
            "extract_source_two_edge_local_mode_radix_asserted_head",
            super::extract_source_two_edge_local_mode_radix_asserted_head,
        ),
        (
            "extract_source_two_edge_local_object_mode_radix_asserted_head",
            super::extract_source_two_edge_local_object_mode_radix_asserted_head,
        ),
        (
            "extract_source_three_edge_local_mode_reserved_variable_type_assertion",
            super::extract_source_three_edge_local_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_three_edge_local_object_mode_reserved_variable_type_assertion",
            super::extract_source_three_edge_local_object_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_three_edge_local_mode_asserted_head",
            super::extract_source_three_edge_local_mode_asserted_head,
        ),
        (
            "extract_source_three_edge_local_object_mode_asserted_head",
            super::extract_source_three_edge_local_object_mode_asserted_head,
        ),
        (
            "extract_source_three_edge_local_mode_radix_asserted_head",
            super::extract_source_three_edge_local_mode_radix_asserted_head,
        ),
        (
            "extract_source_three_edge_local_object_mode_radix_asserted_head",
            super::extract_source_three_edge_local_object_mode_radix_asserted_head,
        ),
        (
            "extract_source_four_edge_local_mode_reserved_variable_type_assertion",
            super::extract_source_four_edge_local_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_four_edge_local_object_mode_reserved_variable_type_assertion",
            super::extract_source_four_edge_local_object_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_four_edge_local_mode_asserted_head",
            super::extract_source_four_edge_local_mode_asserted_head,
        ),
        (
            "extract_source_four_edge_local_object_mode_asserted_head",
            super::extract_source_four_edge_local_object_mode_asserted_head,
        ),
        (
            "extract_source_four_edge_local_mode_radix_asserted_head",
            super::extract_source_four_edge_local_mode_radix_asserted_head,
        ),
        (
            "extract_source_four_edge_local_object_mode_radix_asserted_head",
            super::extract_source_four_edge_local_object_mode_radix_asserted_head,
        ),
        (
            "extract_source_local_mode_long_chain_reserved_variable_type_assertion",
            super::extract_source_local_mode_long_chain_reserved_variable_type_assertion,
        ),
        (
            "extract_source_local_object_mode_long_chain_reserved_variable_type_assertion",
            super::extract_source_local_object_mode_long_chain_reserved_variable_type_assertion,
        ),
        (
            "extract_source_local_mode_long_chain_asserted_head",
            super::extract_source_local_mode_long_chain_asserted_head,
        ),
        (
            "extract_source_local_object_mode_long_chain_asserted_head",
            super::extract_source_local_object_mode_long_chain_asserted_head,
        ),
        (
            "extract_source_local_mode_long_chain_radix_asserted_head",
            super::extract_source_local_mode_long_chain_radix_asserted_head,
        ),
        (
            "extract_source_local_object_mode_long_chain_radix_asserted_head",
            super::extract_source_local_object_mode_long_chain_radix_asserted_head,
        ),
        (
            "extract_source_local_mode_long_chain_two_hop_asserted_head",
            super::extract_source_local_mode_long_chain_two_hop_asserted_head,
        ),
        (
            "extract_source_local_object_mode_long_chain_two_hop_asserted_head",
            super::extract_source_local_object_mode_long_chain_two_hop_asserted_head,
        ),
        (
            "extract_source_two_edge_local_mode_two_hop_asserted_head",
            super::extract_source_two_edge_local_mode_two_hop_asserted_head,
        ),
        (
            "extract_source_two_edge_local_object_mode_two_hop_asserted_head",
            super::extract_source_two_edge_local_object_mode_two_hop_asserted_head,
        ),
        (
            "extract_source_three_edge_local_mode_two_hop_asserted_head",
            super::extract_source_three_edge_local_mode_two_hop_asserted_head,
        ),
        (
            "extract_source_three_edge_local_object_mode_two_hop_asserted_head",
            super::extract_source_three_edge_local_object_mode_two_hop_asserted_head,
        ),
        (
            "extract_source_four_edge_local_mode_two_hop_asserted_head",
            super::extract_source_four_edge_local_mode_two_hop_asserted_head,
        ),
        (
            "extract_source_four_edge_local_object_mode_two_hop_asserted_head",
            super::extract_source_four_edge_local_object_mode_two_hop_asserted_head,
        ),
        (
            "extract_source_three_edge_local_mode_three_hop_asserted_head",
            super::extract_source_three_edge_local_mode_three_hop_asserted_head,
        ),
        (
            "extract_source_three_edge_local_object_mode_three_hop_asserted_head",
            super::extract_source_three_edge_local_object_mode_three_hop_asserted_head,
        ),
        (
            "extract_source_four_edge_local_mode_three_hop_asserted_head",
            super::extract_source_four_edge_local_mode_three_hop_asserted_head,
        ),
        (
            "extract_source_four_edge_local_object_mode_three_hop_asserted_head",
            super::extract_source_four_edge_local_object_mode_three_hop_asserted_head,
        ),
        (
            "extract_source_four_edge_local_mode_four_hop_asserted_head",
            super::extract_source_four_edge_local_mode_four_hop_asserted_head,
        ),
        (
            "extract_source_four_edge_local_object_mode_four_hop_asserted_head",
            super::extract_source_four_edge_local_object_mode_four_hop_asserted_head,
        ),
        (
            "extract_source_local_mode_long_chain_three_hop_asserted_head",
            super::extract_source_local_mode_long_chain_three_hop_asserted_head,
        ),
        (
            "extract_source_local_object_mode_long_chain_three_hop_asserted_head",
            super::extract_source_local_object_mode_long_chain_three_hop_asserted_head,
        ),
    ];
    for (owner, extractor) in prior_extractors {
        assert!(
            extractor(&ast, resolver.module.clone(), &symbols).is_none(),
            "Task 228 source must not be owned by {owner}"
        );
    }

    let prior_owner_ids = [
        "pass_type_elaboration_reserved_variable_type_assertion_001",
        "pass_type_elaboration_reserved_object_variable_type_assertion_001",
        "pass_type_elaboration_local_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_local_object_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_local_mode_asserted_head_001",
        "pass_type_elaboration_local_object_mode_asserted_head_001",
        "pass_type_elaboration_chained_local_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_chained_local_object_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_chained_local_mode_asserted_head_001",
        "pass_type_elaboration_chained_local_object_mode_asserted_head_001",
        "pass_type_elaboration_chained_local_mode_radix_asserted_head_001",
        "pass_type_elaboration_chained_local_object_mode_radix_asserted_head_001",
        "pass_type_elaboration_two_edge_local_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_two_edge_local_object_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_two_edge_local_mode_asserted_head_001",
        "pass_type_elaboration_two_edge_local_object_mode_asserted_head_001",
        "pass_type_elaboration_two_edge_local_mode_radix_asserted_head_001",
        "pass_type_elaboration_two_edge_local_object_mode_radix_asserted_head_001",
        "pass_type_elaboration_three_edge_local_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_three_edge_local_object_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_three_edge_local_mode_asserted_head_001",
        "pass_type_elaboration_three_edge_local_object_mode_asserted_head_001",
        "pass_type_elaboration_three_edge_local_mode_radix_asserted_head_001",
        "pass_type_elaboration_three_edge_local_object_mode_radix_asserted_head_001",
        "pass_type_elaboration_four_edge_local_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_four_edge_local_object_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_four_edge_local_mode_asserted_head_001",
        "pass_type_elaboration_four_edge_local_object_mode_asserted_head_001",
        "pass_type_elaboration_four_edge_local_mode_radix_asserted_head_001",
        "pass_type_elaboration_four_edge_local_object_mode_radix_asserted_head_001",
        "pass_type_elaboration_local_mode_long_chain_reserved_variable_type_assertion_001",
        "pass_type_elaboration_local_object_mode_long_chain_reserved_variable_type_assertion_001",
        "pass_type_elaboration_local_mode_long_chain_asserted_head_001",
        "pass_type_elaboration_local_object_mode_long_chain_asserted_head_001",
        "pass_type_elaboration_local_mode_long_chain_radix_asserted_head_001",
        "pass_type_elaboration_local_object_mode_long_chain_radix_asserted_head_001",
        "pass_type_elaboration_local_mode_long_chain_two_hop_asserted_head_001",
        "pass_type_elaboration_local_object_mode_long_chain_two_hop_asserted_head_001",
        "pass_type_elaboration_two_edge_local_mode_two_hop_asserted_head_001",
        "pass_type_elaboration_two_edge_local_object_mode_two_hop_asserted_head_001",
        "pass_type_elaboration_three_edge_local_mode_two_hop_asserted_head_001",
        "pass_type_elaboration_three_edge_local_object_mode_two_hop_asserted_head_001",
        "pass_type_elaboration_four_edge_local_mode_two_hop_asserted_head_001",
        "pass_type_elaboration_four_edge_local_object_mode_two_hop_asserted_head_001",
        "pass_type_elaboration_three_edge_local_mode_three_hop_asserted_head_001",
        "pass_type_elaboration_three_edge_local_object_mode_three_hop_asserted_head_001",
        "pass_type_elaboration_four_edge_local_mode_three_hop_asserted_head_001",
        "pass_type_elaboration_four_edge_local_object_mode_three_hop_asserted_head_001",
        "pass_type_elaboration_four_edge_local_mode_four_hop_asserted_head_001",
        "pass_type_elaboration_four_edge_local_object_mode_four_hop_asserted_head_001",
        "pass_type_elaboration_local_mode_long_chain_three_hop_asserted_head_001",
        "pass_type_elaboration_local_object_mode_long_chain_three_hop_asserted_head_001",
    ];
    assert_eq!(prior_owner_ids.len(), 52);
    for owner_id in prior_owner_ids {
        let (owner_ordinal, owner_case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| case.id.0 == owner_id)
            .unwrap_or_else(|| panic!("prior owner fixture {owner_id} must be active"));
        let owner_frontend = run_frontend(&workspace_root, owner_case, owner_ordinal)
            .unwrap_or_else(|error| panic!("owner fixture {owner_id} must parse: {error}"));
        assert!(owner_frontend.diagnostics.is_empty());
        let owner_ast = owner_frontend
            .ast
            .unwrap_or_else(|| panic!("owner fixture {owner_id} must produce an AST"));
        let owner_resolver =
            resolver_symbol_collection(&workspace_root, owner_case, &owner_ast);
        assert!(owner_resolver.detail_keys.is_empty());
        let owner_symbols = augment_type_elaboration_import_summaries(
            &owner_ast,
            &owner_resolver.module,
            owner_resolver.env,
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &owner_ast,
                owner_resolver.module.clone(),
                &owner_symbols,
            ),
            Vec::<String>::new(),
            "prior owner fixture {owner_id} must still reach its active route"
        );
        assert!(
            super::extract_source_local_mode_long_chain_four_hop_asserted_head(
                &owner_ast,
                owner_resolver.module,
                &owner_symbols,
            )
            .is_none(),
            "Task 228 must reject prior owner fixture {owner_id}"
        );
    }
}

#[test]
fn active_local_mode_long_chain_five_hop_asserted_head_fixture_consumes_seven_expansions() {
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
    let plan = build_test_plan(&config).expect("Task 230 repository plan should build");
    let (ordinal, case) = active_type_elaboration_cases(&plan)
        .enumerate()
        .find(|(_, case)| {
            case.id.0
                == "pass_type_elaboration_local_mode_long_chain_five_hop_asserted_head_001"
        })
        .expect("Task 230 active fixture should be discoverable");
    let frontend = run_frontend(&workspace_root, case, ordinal)
        .expect("Task 230 fixture should run through the real frontend");
    assert!(frontend.diagnostics.is_empty());
    let ast = frontend
        .ast
        .expect("Task 230 fixture should produce a real AST");
    let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
    assert!(resolver.detail_keys.is_empty());
    let symbols =
        augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
    let output = super::source_local_mode_long_chain_five_hop_asserted_head_output(
        &ast,
        resolver.module.clone(),
        &symbols,
    )
    .expect("Task 230 real AST should reach the seven-expansion five-hop seam");
    assert_source_reserved_variable_type_assertion_output(&output)
        .expect("Task 230 real AST should preserve every checked payload invariant");
    assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
    assert_eq!(output.subject_binding, BindingId::new(0));
    assert_eq!(output.payload.subject_lookup_ordinal, 1);
    assert_eq!(output.subject_result_input.spelling, "ChainMode6");
    assert_eq!(output.asserted_type_input.spelling, "ChainMode1");
    assert_ne!(
        output.subject_result_input.head,
        output.asserted_type_input.head
    );
    assert_eq!(output.term_formula.type_entries().len(), 3);
    assert_eq!(output.term_formula.normalized_types().len(), 1);
    assert!(output.term_formula.candidate_sets().is_empty());
    assert!(output.term_formula.facts().is_empty());
    assert!(output.term_formula.diagnostics().is_empty());
    let chain6 = output
        .payload
        .reserve
        .mode_expansions
        .iter()
        .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("ChainMode6"))
        .map(|(_, expansion)| expansion)
        .expect("Task 230 ChainMode6 expansion should exist");
    assert_eq!(chain6.radix.spelling, "ChainMode5");
    let TypeHeadInput::Symbol(chain5_symbol) = &chain6.radix.head else {
        panic!("Task 230 ChainMode5 first intermediate should be a symbol");
    };
    let chain5 = output
        .payload
        .reserve
        .mode_expansions
        .get(chain5_symbol)
        .expect("Task 230 ChainMode5 expansion should exist");
    assert_eq!(chain5.radix.spelling, "ChainMode4");
    assert_ne!(chain5.radix.head, output.asserted_type_input.head);
    let TypeHeadInput::Symbol(chain4_symbol) = &chain5.radix.head else {
        panic!("Task 230 ChainMode4 second intermediate should be a symbol");
    };
    let chain4 = output
        .payload
        .reserve
        .mode_expansions
        .get(chain4_symbol)
        .expect("Task 230 ChainMode4 expansion should exist");
    assert_eq!(chain4.radix.spelling, "ChainMode3");
    assert_ne!(chain4.radix.head, output.asserted_type_input.head);
    let TypeHeadInput::Symbol(chain3_symbol) = &chain4.radix.head else {
        panic!("Task 230 ChainMode3 third intermediate should be a symbol");
    };
    let chain3 = output
        .payload
        .reserve
        .mode_expansions
        .get(chain3_symbol)
        .expect("Task 230 ChainMode3 expansion should exist");
    assert_eq!(chain3.radix.spelling, "ChainMode2");
    assert_ne!(chain3.radix.head, output.asserted_type_input.head);
    let TypeHeadInput::Symbol(chain2_symbol) = &chain3.radix.head else {
        panic!("Task 230 ChainMode2 fourth intermediate should be a symbol");
    };
    let chain2 = output
        .payload
        .reserve
        .mode_expansions
        .get(chain2_symbol)
        .expect("Task 230 ChainMode2 expansion should exist");
    assert_eq!(chain2.radix.spelling, "ChainMode1");
    assert_eq!(chain2.radix.head, output.asserted_type_input.head);

    type OwnerExtractor = fn(
        &SurfaceAst,
        ResolverModuleId,
        &SymbolEnv,
    ) -> Option<super::SourceReservedVariableTypeAssertion>;
    let prior_extractors: [(&str, OwnerExtractor); 54] = [
        (
            "extract_source_reserved_variable_type_assertion",
            super::extract_source_reserved_variable_type_assertion,
        ),
        (
            "extract_source_reserved_object_variable_type_assertion",
            super::extract_source_reserved_object_variable_type_assertion,
        ),
        (
            "extract_source_local_mode_reserved_variable_type_assertion",
            super::extract_source_local_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_local_object_mode_reserved_variable_type_assertion",
            super::extract_source_local_object_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_local_mode_asserted_head",
            super::extract_source_local_mode_asserted_head,
        ),
        (
            "extract_source_local_object_mode_asserted_head",
            super::extract_source_local_object_mode_asserted_head,
        ),
        (
            "extract_source_chained_local_mode_reserved_variable_type_assertion",
            super::extract_source_chained_local_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_chained_local_object_mode_reserved_variable_type_assertion",
            super::extract_source_chained_local_object_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_chained_local_mode_asserted_head",
            super::extract_source_chained_local_mode_asserted_head,
        ),
        (
            "extract_source_chained_local_object_mode_asserted_head",
            super::extract_source_chained_local_object_mode_asserted_head,
        ),
        (
            "extract_source_chained_local_mode_radix_asserted_head",
            super::extract_source_chained_local_mode_radix_asserted_head,
        ),
        (
            "extract_source_chained_local_object_mode_radix_asserted_head",
            super::extract_source_chained_local_object_mode_radix_asserted_head,
        ),
        (
            "extract_source_two_edge_local_mode_reserved_variable_type_assertion",
            super::extract_source_two_edge_local_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_two_edge_local_object_mode_reserved_variable_type_assertion",
            super::extract_source_two_edge_local_object_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_two_edge_local_mode_asserted_head",
            super::extract_source_two_edge_local_mode_asserted_head,
        ),
        (
            "extract_source_two_edge_local_object_mode_asserted_head",
            super::extract_source_two_edge_local_object_mode_asserted_head,
        ),
        (
            "extract_source_two_edge_local_mode_radix_asserted_head",
            super::extract_source_two_edge_local_mode_radix_asserted_head,
        ),
        (
            "extract_source_two_edge_local_object_mode_radix_asserted_head",
            super::extract_source_two_edge_local_object_mode_radix_asserted_head,
        ),
        (
            "extract_source_three_edge_local_mode_reserved_variable_type_assertion",
            super::extract_source_three_edge_local_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_three_edge_local_object_mode_reserved_variable_type_assertion",
            super::extract_source_three_edge_local_object_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_three_edge_local_mode_asserted_head",
            super::extract_source_three_edge_local_mode_asserted_head,
        ),
        (
            "extract_source_three_edge_local_object_mode_asserted_head",
            super::extract_source_three_edge_local_object_mode_asserted_head,
        ),
        (
            "extract_source_three_edge_local_mode_radix_asserted_head",
            super::extract_source_three_edge_local_mode_radix_asserted_head,
        ),
        (
            "extract_source_three_edge_local_object_mode_radix_asserted_head",
            super::extract_source_three_edge_local_object_mode_radix_asserted_head,
        ),
        (
            "extract_source_four_edge_local_mode_reserved_variable_type_assertion",
            super::extract_source_four_edge_local_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_four_edge_local_object_mode_reserved_variable_type_assertion",
            super::extract_source_four_edge_local_object_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_four_edge_local_mode_asserted_head",
            super::extract_source_four_edge_local_mode_asserted_head,
        ),
        (
            "extract_source_four_edge_local_object_mode_asserted_head",
            super::extract_source_four_edge_local_object_mode_asserted_head,
        ),
        (
            "extract_source_four_edge_local_mode_radix_asserted_head",
            super::extract_source_four_edge_local_mode_radix_asserted_head,
        ),
        (
            "extract_source_four_edge_local_object_mode_radix_asserted_head",
            super::extract_source_four_edge_local_object_mode_radix_asserted_head,
        ),
        (
            "extract_source_local_mode_long_chain_reserved_variable_type_assertion",
            super::extract_source_local_mode_long_chain_reserved_variable_type_assertion,
        ),
        (
            "extract_source_local_object_mode_long_chain_reserved_variable_type_assertion",
            super::extract_source_local_object_mode_long_chain_reserved_variable_type_assertion,
        ),
        (
            "extract_source_local_mode_long_chain_asserted_head",
            super::extract_source_local_mode_long_chain_asserted_head,
        ),
        (
            "extract_source_local_object_mode_long_chain_asserted_head",
            super::extract_source_local_object_mode_long_chain_asserted_head,
        ),
        (
            "extract_source_local_mode_long_chain_radix_asserted_head",
            super::extract_source_local_mode_long_chain_radix_asserted_head,
        ),
        (
            "extract_source_local_object_mode_long_chain_radix_asserted_head",
            super::extract_source_local_object_mode_long_chain_radix_asserted_head,
        ),
        (
            "extract_source_local_mode_long_chain_two_hop_asserted_head",
            super::extract_source_local_mode_long_chain_two_hop_asserted_head,
        ),
        (
            "extract_source_local_object_mode_long_chain_two_hop_asserted_head",
            super::extract_source_local_object_mode_long_chain_two_hop_asserted_head,
        ),
        (
            "extract_source_two_edge_local_mode_two_hop_asserted_head",
            super::extract_source_two_edge_local_mode_two_hop_asserted_head,
        ),
        (
            "extract_source_two_edge_local_object_mode_two_hop_asserted_head",
            super::extract_source_two_edge_local_object_mode_two_hop_asserted_head,
        ),
        (
            "extract_source_three_edge_local_mode_two_hop_asserted_head",
            super::extract_source_three_edge_local_mode_two_hop_asserted_head,
        ),
        (
            "extract_source_three_edge_local_object_mode_two_hop_asserted_head",
            super::extract_source_three_edge_local_object_mode_two_hop_asserted_head,
        ),
        (
            "extract_source_four_edge_local_mode_two_hop_asserted_head",
            super::extract_source_four_edge_local_mode_two_hop_asserted_head,
        ),
        (
            "extract_source_four_edge_local_object_mode_two_hop_asserted_head",
            super::extract_source_four_edge_local_object_mode_two_hop_asserted_head,
        ),
        (
            "extract_source_three_edge_local_mode_three_hop_asserted_head",
            super::extract_source_three_edge_local_mode_three_hop_asserted_head,
        ),
        (
            "extract_source_three_edge_local_object_mode_three_hop_asserted_head",
            super::extract_source_three_edge_local_object_mode_three_hop_asserted_head,
        ),
        (
            "extract_source_four_edge_local_mode_three_hop_asserted_head",
            super::extract_source_four_edge_local_mode_three_hop_asserted_head,
        ),
        (
            "extract_source_four_edge_local_object_mode_three_hop_asserted_head",
            super::extract_source_four_edge_local_object_mode_three_hop_asserted_head,
        ),
        (
            "extract_source_four_edge_local_mode_four_hop_asserted_head",
            super::extract_source_four_edge_local_mode_four_hop_asserted_head,
        ),
        (
            "extract_source_four_edge_local_object_mode_four_hop_asserted_head",
            super::extract_source_four_edge_local_object_mode_four_hop_asserted_head,
        ),
        (
            "extract_source_local_mode_long_chain_three_hop_asserted_head",
            super::extract_source_local_mode_long_chain_three_hop_asserted_head,
        ),
        (
            "extract_source_local_object_mode_long_chain_three_hop_asserted_head",
            super::extract_source_local_object_mode_long_chain_three_hop_asserted_head,
        ),
        (
            "extract_source_local_mode_long_chain_four_hop_asserted_head",
            super::extract_source_local_mode_long_chain_four_hop_asserted_head,
        ),
        (
            "extract_source_local_object_mode_long_chain_four_hop_asserted_head",
            super::extract_source_local_object_mode_long_chain_four_hop_asserted_head,
        ),
    ];
    for (owner, extractor) in prior_extractors {
        assert!(
            extractor(&ast, resolver.module.clone(), &symbols).is_none(),
            "Task 230 source must not be owned by {owner}"
        );
    }

    let prior_owner_ids = [
        "pass_type_elaboration_reserved_variable_type_assertion_001",
        "pass_type_elaboration_reserved_object_variable_type_assertion_001",
        "pass_type_elaboration_local_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_local_object_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_local_mode_asserted_head_001",
        "pass_type_elaboration_local_object_mode_asserted_head_001",
        "pass_type_elaboration_chained_local_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_chained_local_object_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_chained_local_mode_asserted_head_001",
        "pass_type_elaboration_chained_local_object_mode_asserted_head_001",
        "pass_type_elaboration_chained_local_mode_radix_asserted_head_001",
        "pass_type_elaboration_chained_local_object_mode_radix_asserted_head_001",
        "pass_type_elaboration_two_edge_local_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_two_edge_local_object_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_two_edge_local_mode_asserted_head_001",
        "pass_type_elaboration_two_edge_local_object_mode_asserted_head_001",
        "pass_type_elaboration_two_edge_local_mode_radix_asserted_head_001",
        "pass_type_elaboration_two_edge_local_object_mode_radix_asserted_head_001",
        "pass_type_elaboration_three_edge_local_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_three_edge_local_object_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_three_edge_local_mode_asserted_head_001",
        "pass_type_elaboration_three_edge_local_object_mode_asserted_head_001",
        "pass_type_elaboration_three_edge_local_mode_radix_asserted_head_001",
        "pass_type_elaboration_three_edge_local_object_mode_radix_asserted_head_001",
        "pass_type_elaboration_four_edge_local_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_four_edge_local_object_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_four_edge_local_mode_asserted_head_001",
        "pass_type_elaboration_four_edge_local_object_mode_asserted_head_001",
        "pass_type_elaboration_four_edge_local_mode_radix_asserted_head_001",
        "pass_type_elaboration_four_edge_local_object_mode_radix_asserted_head_001",
        "pass_type_elaboration_local_mode_long_chain_reserved_variable_type_assertion_001",
        "pass_type_elaboration_local_object_mode_long_chain_reserved_variable_type_assertion_001",
        "pass_type_elaboration_local_mode_long_chain_asserted_head_001",
        "pass_type_elaboration_local_object_mode_long_chain_asserted_head_001",
        "pass_type_elaboration_local_mode_long_chain_radix_asserted_head_001",
        "pass_type_elaboration_local_object_mode_long_chain_radix_asserted_head_001",
        "pass_type_elaboration_local_mode_long_chain_two_hop_asserted_head_001",
        "pass_type_elaboration_local_object_mode_long_chain_two_hop_asserted_head_001",
        "pass_type_elaboration_two_edge_local_mode_two_hop_asserted_head_001",
        "pass_type_elaboration_two_edge_local_object_mode_two_hop_asserted_head_001",
        "pass_type_elaboration_three_edge_local_mode_two_hop_asserted_head_001",
        "pass_type_elaboration_three_edge_local_object_mode_two_hop_asserted_head_001",
        "pass_type_elaboration_four_edge_local_mode_two_hop_asserted_head_001",
        "pass_type_elaboration_four_edge_local_object_mode_two_hop_asserted_head_001",
        "pass_type_elaboration_three_edge_local_mode_three_hop_asserted_head_001",
        "pass_type_elaboration_three_edge_local_object_mode_three_hop_asserted_head_001",
        "pass_type_elaboration_four_edge_local_mode_three_hop_asserted_head_001",
        "pass_type_elaboration_four_edge_local_object_mode_three_hop_asserted_head_001",
        "pass_type_elaboration_four_edge_local_mode_four_hop_asserted_head_001",
        "pass_type_elaboration_four_edge_local_object_mode_four_hop_asserted_head_001",
        "pass_type_elaboration_local_mode_long_chain_three_hop_asserted_head_001",
        "pass_type_elaboration_local_object_mode_long_chain_three_hop_asserted_head_001",
        "pass_type_elaboration_local_mode_long_chain_four_hop_asserted_head_001",
        "pass_type_elaboration_local_object_mode_long_chain_four_hop_asserted_head_001",
    ];
    assert_eq!(prior_owner_ids.len(), 54);
    for owner_id in prior_owner_ids {
        let (owner_ordinal, owner_case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| case.id.0 == owner_id)
            .unwrap_or_else(|| panic!("prior owner fixture {owner_id} must be active"));
        let owner_frontend = run_frontend(&workspace_root, owner_case, owner_ordinal)
            .unwrap_or_else(|error| panic!("owner fixture {owner_id} must parse: {error}"));
        assert!(owner_frontend.diagnostics.is_empty());
        let owner_ast = owner_frontend
            .ast
            .unwrap_or_else(|| panic!("owner fixture {owner_id} must produce an AST"));
        let owner_resolver =
            resolver_symbol_collection(&workspace_root, owner_case, &owner_ast);
        assert!(owner_resolver.detail_keys.is_empty());
        let owner_symbols = augment_type_elaboration_import_summaries(
            &owner_ast,
            &owner_resolver.module,
            owner_resolver.env,
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &owner_ast,
                owner_resolver.module.clone(),
                &owner_symbols,
            ),
            Vec::<String>::new(),
            "prior owner fixture {owner_id} must still reach its active route"
        );
        assert!(
            super::extract_source_local_mode_long_chain_five_hop_asserted_head(
                &owner_ast,
                owner_resolver.module,
                &owner_symbols,
            )
            .is_none(),
            "Task 230 must reject prior owner fixture {owner_id}"
        );
    }
}

#[test]
fn active_local_object_mode_long_chain_five_hop_asserted_head_fixture_consumes_seven_expansions()
 {
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
    let plan = build_test_plan(&config).expect("Task 231 repository plan should build");
    let (ordinal, case) = active_type_elaboration_cases(&plan)
        .enumerate()
        .find(|(_, case)| {
            case.id.0
                == "pass_type_elaboration_local_object_mode_long_chain_five_hop_asserted_head_001"
        })
        .expect("Task 231 active fixture should be discoverable");
    let frontend = run_frontend(&workspace_root, case, ordinal)
        .expect("Task 231 fixture should run through the real frontend");
    assert!(frontend.diagnostics.is_empty());
    let ast = frontend
        .ast
        .expect("Task 231 fixture should produce a real AST");
    let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
    assert!(resolver.detail_keys.is_empty());
    let symbols =
        augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
    let output = super::source_local_object_mode_long_chain_five_hop_asserted_head_output(
        &ast,
        resolver.module.clone(),
        &symbols,
    )
    .expect("Task 231 real AST should reach the seven-expansion five-hop seam");
    assert_source_reserved_variable_type_assertion_output(&output)
        .expect("Task 231 real AST should preserve every checked payload invariant");
    assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
    assert_eq!(output.subject_binding, BindingId::new(0));
    assert_eq!(output.payload.subject_lookup_ordinal, 1);
    assert_eq!(output.subject_result_input.spelling, "ChainObjectMode6");
    assert_eq!(output.asserted_type_input.spelling, "ChainObjectMode1");
    assert_ne!(
        output.subject_result_input.head,
        output.asserted_type_input.head
    );
    assert_eq!(output.term_formula.type_entries().len(), 3);
    assert_eq!(output.term_formula.normalized_types().len(), 1);
    assert!(output.term_formula.candidate_sets().is_empty());
    assert!(output.term_formula.facts().is_empty());
    assert!(output.term_formula.diagnostics().is_empty());
    let chain6 = output
        .payload
        .reserve
        .mode_expansions
        .iter()
        .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("ChainObjectMode6"))
        .map(|(_, expansion)| expansion)
        .expect("Task 231 ChainObjectMode6 expansion should exist");
    assert_eq!(chain6.radix.spelling, "ChainObjectMode5");
    let TypeHeadInput::Symbol(chain5_symbol) = &chain6.radix.head else {
        panic!("Task 231 ChainObjectMode5 first intermediate should be a symbol");
    };
    let chain5 = output
        .payload
        .reserve
        .mode_expansions
        .get(chain5_symbol)
        .expect("Task 231 ChainObjectMode5 expansion should exist");
    assert_eq!(chain5.radix.spelling, "ChainObjectMode4");
    assert_ne!(chain5.radix.head, output.asserted_type_input.head);
    let TypeHeadInput::Symbol(chain4_symbol) = &chain5.radix.head else {
        panic!("Task 231 ChainObjectMode4 second intermediate should be a symbol");
    };
    let chain4 = output
        .payload
        .reserve
        .mode_expansions
        .get(chain4_symbol)
        .expect("Task 231 ChainObjectMode4 expansion should exist");
    assert_eq!(chain4.radix.spelling, "ChainObjectMode3");
    assert_ne!(chain4.radix.head, output.asserted_type_input.head);
    let TypeHeadInput::Symbol(chain3_symbol) = &chain4.radix.head else {
        panic!("Task 231 ChainObjectMode3 third intermediate should be a symbol");
    };
    let chain3 = output
        .payload
        .reserve
        .mode_expansions
        .get(chain3_symbol)
        .expect("Task 231 ChainObjectMode3 expansion should exist");
    assert_eq!(chain3.radix.spelling, "ChainObjectMode2");
    assert_ne!(chain3.radix.head, output.asserted_type_input.head);
    let TypeHeadInput::Symbol(chain2_symbol) = &chain3.radix.head else {
        panic!("Task 231 ChainObjectMode2 fourth intermediate should be a symbol");
    };
    let chain2 = output
        .payload
        .reserve
        .mode_expansions
        .get(chain2_symbol)
        .expect("Task 231 ChainObjectMode2 expansion should exist");
    assert_eq!(chain2.radix.spelling, "ChainObjectMode1");
    assert_eq!(chain2.radix.head, output.asserted_type_input.head);

    type OwnerExtractor = fn(
        &SurfaceAst,
        ResolverModuleId,
        &SymbolEnv,
    ) -> Option<super::SourceReservedVariableTypeAssertion>;
    let prior_extractors: [(&str, OwnerExtractor); 55] = [
        (
            "extract_source_reserved_variable_type_assertion",
            super::extract_source_reserved_variable_type_assertion,
        ),
        (
            "extract_source_reserved_object_variable_type_assertion",
            super::extract_source_reserved_object_variable_type_assertion,
        ),
        (
            "extract_source_local_object_mode_reserved_variable_type_assertion",
            super::extract_source_local_object_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_local_mode_reserved_variable_type_assertion",
            super::extract_source_local_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_local_object_mode_asserted_head",
            super::extract_source_local_object_mode_asserted_head,
        ),
        (
            "extract_source_local_mode_asserted_head",
            super::extract_source_local_mode_asserted_head,
        ),
        (
            "extract_source_chained_local_object_mode_reserved_variable_type_assertion",
            super::extract_source_chained_local_object_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_chained_local_mode_reserved_variable_type_assertion",
            super::extract_source_chained_local_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_chained_local_object_mode_asserted_head",
            super::extract_source_chained_local_object_mode_asserted_head,
        ),
        (
            "extract_source_chained_local_mode_asserted_head",
            super::extract_source_chained_local_mode_asserted_head,
        ),
        (
            "extract_source_chained_local_object_mode_radix_asserted_head",
            super::extract_source_chained_local_object_mode_radix_asserted_head,
        ),
        (
            "extract_source_chained_local_mode_radix_asserted_head",
            super::extract_source_chained_local_mode_radix_asserted_head,
        ),
        (
            "extract_source_two_edge_local_object_mode_reserved_variable_type_assertion",
            super::extract_source_two_edge_local_object_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_two_edge_local_mode_reserved_variable_type_assertion",
            super::extract_source_two_edge_local_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_two_edge_local_object_mode_asserted_head",
            super::extract_source_two_edge_local_object_mode_asserted_head,
        ),
        (
            "extract_source_two_edge_local_mode_asserted_head",
            super::extract_source_two_edge_local_mode_asserted_head,
        ),
        (
            "extract_source_two_edge_local_object_mode_radix_asserted_head",
            super::extract_source_two_edge_local_object_mode_radix_asserted_head,
        ),
        (
            "extract_source_two_edge_local_mode_radix_asserted_head",
            super::extract_source_two_edge_local_mode_radix_asserted_head,
        ),
        (
            "extract_source_three_edge_local_object_mode_reserved_variable_type_assertion",
            super::extract_source_three_edge_local_object_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_three_edge_local_mode_reserved_variable_type_assertion",
            super::extract_source_three_edge_local_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_three_edge_local_object_mode_asserted_head",
            super::extract_source_three_edge_local_object_mode_asserted_head,
        ),
        (
            "extract_source_three_edge_local_mode_asserted_head",
            super::extract_source_three_edge_local_mode_asserted_head,
        ),
        (
            "extract_source_three_edge_local_object_mode_radix_asserted_head",
            super::extract_source_three_edge_local_object_mode_radix_asserted_head,
        ),
        (
            "extract_source_three_edge_local_mode_radix_asserted_head",
            super::extract_source_three_edge_local_mode_radix_asserted_head,
        ),
        (
            "extract_source_four_edge_local_object_mode_reserved_variable_type_assertion",
            super::extract_source_four_edge_local_object_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_four_edge_local_mode_reserved_variable_type_assertion",
            super::extract_source_four_edge_local_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_four_edge_local_object_mode_asserted_head",
            super::extract_source_four_edge_local_object_mode_asserted_head,
        ),
        (
            "extract_source_four_edge_local_mode_asserted_head",
            super::extract_source_four_edge_local_mode_asserted_head,
        ),
        (
            "extract_source_four_edge_local_object_mode_radix_asserted_head",
            super::extract_source_four_edge_local_object_mode_radix_asserted_head,
        ),
        (
            "extract_source_four_edge_local_mode_radix_asserted_head",
            super::extract_source_four_edge_local_mode_radix_asserted_head,
        ),
        (
            "extract_source_local_object_mode_long_chain_reserved_variable_type_assertion",
            super::extract_source_local_object_mode_long_chain_reserved_variable_type_assertion,
        ),
        (
            "extract_source_local_mode_long_chain_reserved_variable_type_assertion",
            super::extract_source_local_mode_long_chain_reserved_variable_type_assertion,
        ),
        (
            "extract_source_local_object_mode_long_chain_asserted_head",
            super::extract_source_local_object_mode_long_chain_asserted_head,
        ),
        (
            "extract_source_local_mode_long_chain_asserted_head",
            super::extract_source_local_mode_long_chain_asserted_head,
        ),
        (
            "extract_source_local_object_mode_long_chain_radix_asserted_head",
            super::extract_source_local_object_mode_long_chain_radix_asserted_head,
        ),
        (
            "extract_source_local_mode_long_chain_radix_asserted_head",
            super::extract_source_local_mode_long_chain_radix_asserted_head,
        ),
        (
            "extract_source_local_object_mode_long_chain_two_hop_asserted_head",
            super::extract_source_local_object_mode_long_chain_two_hop_asserted_head,
        ),
        (
            "extract_source_local_mode_long_chain_two_hop_asserted_head",
            super::extract_source_local_mode_long_chain_two_hop_asserted_head,
        ),
        (
            "extract_source_two_edge_local_object_mode_two_hop_asserted_head",
            super::extract_source_two_edge_local_object_mode_two_hop_asserted_head,
        ),
        (
            "extract_source_two_edge_local_mode_two_hop_asserted_head",
            super::extract_source_two_edge_local_mode_two_hop_asserted_head,
        ),
        (
            "extract_source_three_edge_local_object_mode_two_hop_asserted_head",
            super::extract_source_three_edge_local_object_mode_two_hop_asserted_head,
        ),
        (
            "extract_source_three_edge_local_mode_two_hop_asserted_head",
            super::extract_source_three_edge_local_mode_two_hop_asserted_head,
        ),
        (
            "extract_source_four_edge_local_object_mode_two_hop_asserted_head",
            super::extract_source_four_edge_local_object_mode_two_hop_asserted_head,
        ),
        (
            "extract_source_four_edge_local_mode_two_hop_asserted_head",
            super::extract_source_four_edge_local_mode_two_hop_asserted_head,
        ),
        (
            "extract_source_three_edge_local_object_mode_three_hop_asserted_head",
            super::extract_source_three_edge_local_object_mode_three_hop_asserted_head,
        ),
        (
            "extract_source_three_edge_local_mode_three_hop_asserted_head",
            super::extract_source_three_edge_local_mode_three_hop_asserted_head,
        ),
        (
            "extract_source_four_edge_local_object_mode_three_hop_asserted_head",
            super::extract_source_four_edge_local_object_mode_three_hop_asserted_head,
        ),
        (
            "extract_source_four_edge_local_mode_three_hop_asserted_head",
            super::extract_source_four_edge_local_mode_three_hop_asserted_head,
        ),
        (
            "extract_source_four_edge_local_object_mode_four_hop_asserted_head",
            super::extract_source_four_edge_local_object_mode_four_hop_asserted_head,
        ),
        (
            "extract_source_four_edge_local_mode_four_hop_asserted_head",
            super::extract_source_four_edge_local_mode_four_hop_asserted_head,
        ),
        (
            "extract_source_local_object_mode_long_chain_three_hop_asserted_head",
            super::extract_source_local_object_mode_long_chain_three_hop_asserted_head,
        ),
        (
            "extract_source_local_mode_long_chain_three_hop_asserted_head",
            super::extract_source_local_mode_long_chain_three_hop_asserted_head,
        ),
        (
            "extract_source_local_object_mode_long_chain_four_hop_asserted_head",
            super::extract_source_local_object_mode_long_chain_four_hop_asserted_head,
        ),
        (
            "extract_source_local_mode_long_chain_four_hop_asserted_head",
            super::extract_source_local_mode_long_chain_four_hop_asserted_head,
        ),
        (
            "extract_source_local_mode_long_chain_five_hop_asserted_head",
            super::extract_source_local_mode_long_chain_five_hop_asserted_head,
        ),
    ];
    for (owner, extractor) in prior_extractors {
        assert!(
            extractor(&ast, resolver.module.clone(), &symbols).is_none(),
            "Task 231 source must not be owned by {owner}"
        );
    }

    let prior_owner_ids = [
        "pass_type_elaboration_reserved_variable_type_assertion_001",
        "pass_type_elaboration_reserved_object_variable_type_assertion_001",
        "pass_type_elaboration_local_object_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_local_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_local_object_mode_asserted_head_001",
        "pass_type_elaboration_local_mode_asserted_head_001",
        "pass_type_elaboration_chained_local_object_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_chained_local_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_chained_local_object_mode_asserted_head_001",
        "pass_type_elaboration_chained_local_mode_asserted_head_001",
        "pass_type_elaboration_chained_local_object_mode_radix_asserted_head_001",
        "pass_type_elaboration_chained_local_mode_radix_asserted_head_001",
        "pass_type_elaboration_two_edge_local_object_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_two_edge_local_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_two_edge_local_object_mode_asserted_head_001",
        "pass_type_elaboration_two_edge_local_mode_asserted_head_001",
        "pass_type_elaboration_two_edge_local_object_mode_radix_asserted_head_001",
        "pass_type_elaboration_two_edge_local_mode_radix_asserted_head_001",
        "pass_type_elaboration_three_edge_local_object_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_three_edge_local_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_three_edge_local_object_mode_asserted_head_001",
        "pass_type_elaboration_three_edge_local_mode_asserted_head_001",
        "pass_type_elaboration_three_edge_local_object_mode_radix_asserted_head_001",
        "pass_type_elaboration_three_edge_local_mode_radix_asserted_head_001",
        "pass_type_elaboration_four_edge_local_object_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_four_edge_local_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_four_edge_local_object_mode_asserted_head_001",
        "pass_type_elaboration_four_edge_local_mode_asserted_head_001",
        "pass_type_elaboration_four_edge_local_object_mode_radix_asserted_head_001",
        "pass_type_elaboration_four_edge_local_mode_radix_asserted_head_001",
        "pass_type_elaboration_local_object_mode_long_chain_reserved_variable_type_assertion_001",
        "pass_type_elaboration_local_mode_long_chain_reserved_variable_type_assertion_001",
        "pass_type_elaboration_local_object_mode_long_chain_asserted_head_001",
        "pass_type_elaboration_local_mode_long_chain_asserted_head_001",
        "pass_type_elaboration_local_object_mode_long_chain_radix_asserted_head_001",
        "pass_type_elaboration_local_mode_long_chain_radix_asserted_head_001",
        "pass_type_elaboration_local_object_mode_long_chain_two_hop_asserted_head_001",
        "pass_type_elaboration_local_mode_long_chain_two_hop_asserted_head_001",
        "pass_type_elaboration_two_edge_local_object_mode_two_hop_asserted_head_001",
        "pass_type_elaboration_two_edge_local_mode_two_hop_asserted_head_001",
        "pass_type_elaboration_three_edge_local_object_mode_two_hop_asserted_head_001",
        "pass_type_elaboration_three_edge_local_mode_two_hop_asserted_head_001",
        "pass_type_elaboration_four_edge_local_object_mode_two_hop_asserted_head_001",
        "pass_type_elaboration_four_edge_local_mode_two_hop_asserted_head_001",
        "pass_type_elaboration_three_edge_local_object_mode_three_hop_asserted_head_001",
        "pass_type_elaboration_three_edge_local_mode_three_hop_asserted_head_001",
        "pass_type_elaboration_four_edge_local_object_mode_three_hop_asserted_head_001",
        "pass_type_elaboration_four_edge_local_mode_three_hop_asserted_head_001",
        "pass_type_elaboration_four_edge_local_object_mode_four_hop_asserted_head_001",
        "pass_type_elaboration_four_edge_local_mode_four_hop_asserted_head_001",
        "pass_type_elaboration_local_object_mode_long_chain_three_hop_asserted_head_001",
        "pass_type_elaboration_local_mode_long_chain_three_hop_asserted_head_001",
        "pass_type_elaboration_local_object_mode_long_chain_four_hop_asserted_head_001",
        "pass_type_elaboration_local_mode_long_chain_four_hop_asserted_head_001",
        "pass_type_elaboration_local_mode_long_chain_five_hop_asserted_head_001",
    ];
    assert_eq!(prior_owner_ids.len(), 55);
    for owner_id in prior_owner_ids {
        let (owner_ordinal, owner_case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| case.id.0 == owner_id)
            .unwrap_or_else(|| panic!("prior owner fixture {owner_id} must be active"));
        let owner_frontend = run_frontend(&workspace_root, owner_case, owner_ordinal)
            .unwrap_or_else(|error| panic!("owner fixture {owner_id} must parse: {error}"));
        assert!(owner_frontend.diagnostics.is_empty());
        let owner_ast = owner_frontend
            .ast
            .unwrap_or_else(|| panic!("owner fixture {owner_id} must produce an AST"));
        let owner_resolver =
            resolver_symbol_collection(&workspace_root, owner_case, &owner_ast);
        assert!(owner_resolver.detail_keys.is_empty());
        let owner_symbols = augment_type_elaboration_import_summaries(
            &owner_ast,
            &owner_resolver.module,
            owner_resolver.env,
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &owner_ast,
                owner_resolver.module.clone(),
                &owner_symbols,
            ),
            Vec::<String>::new(),
            "prior owner fixture {owner_id} must still reach its active route"
        );
        assert!(
            super::extract_source_local_object_mode_long_chain_five_hop_asserted_head(
                &owner_ast,
                owner_resolver.module,
                &owner_symbols,
            )
            .is_none(),
            "Task 231 must reject prior owner fixture {owner_id}"
        );
    }
}

#[test]
fn active_local_mode_long_chain_six_hop_asserted_head_fixture_consumes_seven_expansions() {
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
    let plan = build_test_plan(&config).expect("Task 234 repository plan should build");
    let (ordinal, case) = active_type_elaboration_cases(&plan)
        .enumerate()
        .find(|(_, case)| {
            case.id.0 == "pass_type_elaboration_local_mode_long_chain_six_hop_asserted_head_001"
        })
        .expect("Task 234 active fixture should be discoverable");
    let frontend = run_frontend(&workspace_root, case, ordinal)
        .expect("Task 234 fixture should run through the real frontend");
    assert!(frontend.diagnostics.is_empty());
    let ast = frontend
        .ast
        .expect("Task 234 fixture should produce a real AST");
    let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
    assert!(resolver.detail_keys.is_empty());
    let symbols =
        augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
    let output = super::source_local_mode_long_chain_six_hop_asserted_head_output(
        &ast,
        resolver.module.clone(),
        &symbols,
    )
    .expect("Task 234 real AST should reach the seven-expansion six-hop seam");
    assert_source_reserved_variable_type_assertion_output(&output)
        .expect("Task 234 real AST should preserve every checked payload invariant");
    assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
    assert_eq!(output.subject_binding, BindingId::new(0));
    assert_eq!(output.payload.subject_lookup_ordinal, 1);
    assert_eq!(output.subject_result_input.spelling, "ChainMode6");
    assert_eq!(output.asserted_type_input.spelling, "BaseMode");
    assert_ne!(
        output.subject_result_input.head,
        output.asserted_type_input.head
    );
    assert_eq!(output.term_formula.type_entries().len(), 3);
    assert_eq!(output.term_formula.normalized_types().len(), 1);
    assert!(output.term_formula.candidate_sets().is_empty());
    assert!(output.term_formula.facts().is_empty());
    assert!(output.term_formula.diagnostics().is_empty());
    let chain6 = output
        .payload
        .reserve
        .mode_expansions
        .iter()
        .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("ChainMode6"))
        .map(|(_, expansion)| expansion)
        .expect("Task 234 ChainMode6 expansion should exist");
    assert_eq!(chain6.radix.spelling, "ChainMode5");
    let TypeHeadInput::Symbol(chain5_symbol) = &chain6.radix.head else {
        panic!("Task 234 ChainMode5 first intermediate should be a symbol");
    };
    let chain5 = output
        .payload
        .reserve
        .mode_expansions
        .get(chain5_symbol)
        .expect("Task 234 ChainMode5 expansion should exist");
    assert_eq!(chain5.radix.spelling, "ChainMode4");
    assert_ne!(chain5.radix.head, output.asserted_type_input.head);
    let TypeHeadInput::Symbol(chain4_symbol) = &chain5.radix.head else {
        panic!("Task 234 ChainMode4 second intermediate should be a symbol");
    };
    let chain4 = output
        .payload
        .reserve
        .mode_expansions
        .get(chain4_symbol)
        .expect("Task 234 ChainMode4 expansion should exist");
    assert_eq!(chain4.radix.spelling, "ChainMode3");
    assert_ne!(chain4.radix.head, output.asserted_type_input.head);
    let TypeHeadInput::Symbol(chain3_symbol) = &chain4.radix.head else {
        panic!("Task 234 ChainMode3 third intermediate should be a symbol");
    };
    let chain3 = output
        .payload
        .reserve
        .mode_expansions
        .get(chain3_symbol)
        .expect("Task 234 ChainMode3 expansion should exist");
    assert_eq!(chain3.radix.spelling, "ChainMode2");
    assert_ne!(chain3.radix.head, output.asserted_type_input.head);
    let TypeHeadInput::Symbol(chain2_symbol) = &chain3.radix.head else {
        panic!("Task 234 ChainMode2 fourth intermediate should be a symbol");
    };
    let chain2 = output
        .payload
        .reserve
        .mode_expansions
        .get(chain2_symbol)
        .expect("Task 234 ChainMode2 expansion should exist");
    assert_eq!(chain2.radix.spelling, "ChainMode1");
    assert_ne!(chain2.radix.head, output.asserted_type_input.head);
    let TypeHeadInput::Symbol(chain1_symbol) = &chain2.radix.head else {
        panic!("Task 234 ChainMode1 fifth intermediate should be a symbol");
    };
    let chain1 = output
        .payload
        .reserve
        .mode_expansions
        .get(chain1_symbol)
        .expect("Task 234 ChainMode1 expansion should exist");
    assert_eq!(chain1.radix.spelling, "BaseMode");
    assert_eq!(chain1.radix.head, output.asserted_type_input.head);

    type OwnerExtractor = fn(
        &SurfaceAst,
        ResolverModuleId,
        &SymbolEnv,
    ) -> Option<super::SourceReservedVariableTypeAssertion>;
    let prior_extractors: [(&str, OwnerExtractor); 56] = [
        (
            "extract_source_reserved_variable_type_assertion",
            super::extract_source_reserved_variable_type_assertion,
        ),
        (
            "extract_source_reserved_object_variable_type_assertion",
            super::extract_source_reserved_object_variable_type_assertion,
        ),
        (
            "extract_source_local_object_mode_reserved_variable_type_assertion",
            super::extract_source_local_object_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_local_mode_reserved_variable_type_assertion",
            super::extract_source_local_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_local_object_mode_asserted_head",
            super::extract_source_local_object_mode_asserted_head,
        ),
        (
            "extract_source_local_mode_asserted_head",
            super::extract_source_local_mode_asserted_head,
        ),
        (
            "extract_source_chained_local_object_mode_reserved_variable_type_assertion",
            super::extract_source_chained_local_object_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_chained_local_mode_reserved_variable_type_assertion",
            super::extract_source_chained_local_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_chained_local_object_mode_asserted_head",
            super::extract_source_chained_local_object_mode_asserted_head,
        ),
        (
            "extract_source_chained_local_mode_asserted_head",
            super::extract_source_chained_local_mode_asserted_head,
        ),
        (
            "extract_source_chained_local_object_mode_radix_asserted_head",
            super::extract_source_chained_local_object_mode_radix_asserted_head,
        ),
        (
            "extract_source_chained_local_mode_radix_asserted_head",
            super::extract_source_chained_local_mode_radix_asserted_head,
        ),
        (
            "extract_source_two_edge_local_object_mode_reserved_variable_type_assertion",
            super::extract_source_two_edge_local_object_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_two_edge_local_mode_reserved_variable_type_assertion",
            super::extract_source_two_edge_local_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_two_edge_local_object_mode_asserted_head",
            super::extract_source_two_edge_local_object_mode_asserted_head,
        ),
        (
            "extract_source_two_edge_local_mode_asserted_head",
            super::extract_source_two_edge_local_mode_asserted_head,
        ),
        (
            "extract_source_two_edge_local_object_mode_radix_asserted_head",
            super::extract_source_two_edge_local_object_mode_radix_asserted_head,
        ),
        (
            "extract_source_two_edge_local_mode_radix_asserted_head",
            super::extract_source_two_edge_local_mode_radix_asserted_head,
        ),
        (
            "extract_source_three_edge_local_object_mode_reserved_variable_type_assertion",
            super::extract_source_three_edge_local_object_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_three_edge_local_mode_reserved_variable_type_assertion",
            super::extract_source_three_edge_local_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_three_edge_local_object_mode_asserted_head",
            super::extract_source_three_edge_local_object_mode_asserted_head,
        ),
        (
            "extract_source_three_edge_local_mode_asserted_head",
            super::extract_source_three_edge_local_mode_asserted_head,
        ),
        (
            "extract_source_three_edge_local_object_mode_radix_asserted_head",
            super::extract_source_three_edge_local_object_mode_radix_asserted_head,
        ),
        (
            "extract_source_three_edge_local_mode_radix_asserted_head",
            super::extract_source_three_edge_local_mode_radix_asserted_head,
        ),
        (
            "extract_source_four_edge_local_object_mode_reserved_variable_type_assertion",
            super::extract_source_four_edge_local_object_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_four_edge_local_mode_reserved_variable_type_assertion",
            super::extract_source_four_edge_local_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_four_edge_local_object_mode_asserted_head",
            super::extract_source_four_edge_local_object_mode_asserted_head,
        ),
        (
            "extract_source_four_edge_local_mode_asserted_head",
            super::extract_source_four_edge_local_mode_asserted_head,
        ),
        (
            "extract_source_four_edge_local_object_mode_radix_asserted_head",
            super::extract_source_four_edge_local_object_mode_radix_asserted_head,
        ),
        (
            "extract_source_four_edge_local_mode_radix_asserted_head",
            super::extract_source_four_edge_local_mode_radix_asserted_head,
        ),
        (
            "extract_source_local_object_mode_long_chain_reserved_variable_type_assertion",
            super::extract_source_local_object_mode_long_chain_reserved_variable_type_assertion,
        ),
        (
            "extract_source_local_mode_long_chain_reserved_variable_type_assertion",
            super::extract_source_local_mode_long_chain_reserved_variable_type_assertion,
        ),
        (
            "extract_source_local_object_mode_long_chain_asserted_head",
            super::extract_source_local_object_mode_long_chain_asserted_head,
        ),
        (
            "extract_source_local_mode_long_chain_asserted_head",
            super::extract_source_local_mode_long_chain_asserted_head,
        ),
        (
            "extract_source_local_object_mode_long_chain_radix_asserted_head",
            super::extract_source_local_object_mode_long_chain_radix_asserted_head,
        ),
        (
            "extract_source_local_mode_long_chain_radix_asserted_head",
            super::extract_source_local_mode_long_chain_radix_asserted_head,
        ),
        (
            "extract_source_local_object_mode_long_chain_two_hop_asserted_head",
            super::extract_source_local_object_mode_long_chain_two_hop_asserted_head,
        ),
        (
            "extract_source_local_mode_long_chain_two_hop_asserted_head",
            super::extract_source_local_mode_long_chain_two_hop_asserted_head,
        ),
        (
            "extract_source_two_edge_local_object_mode_two_hop_asserted_head",
            super::extract_source_two_edge_local_object_mode_two_hop_asserted_head,
        ),
        (
            "extract_source_two_edge_local_mode_two_hop_asserted_head",
            super::extract_source_two_edge_local_mode_two_hop_asserted_head,
        ),
        (
            "extract_source_three_edge_local_object_mode_two_hop_asserted_head",
            super::extract_source_three_edge_local_object_mode_two_hop_asserted_head,
        ),
        (
            "extract_source_three_edge_local_mode_two_hop_asserted_head",
            super::extract_source_three_edge_local_mode_two_hop_asserted_head,
        ),
        (
            "extract_source_four_edge_local_object_mode_two_hop_asserted_head",
            super::extract_source_four_edge_local_object_mode_two_hop_asserted_head,
        ),
        (
            "extract_source_four_edge_local_mode_two_hop_asserted_head",
            super::extract_source_four_edge_local_mode_two_hop_asserted_head,
        ),
        (
            "extract_source_three_edge_local_object_mode_three_hop_asserted_head",
            super::extract_source_three_edge_local_object_mode_three_hop_asserted_head,
        ),
        (
            "extract_source_three_edge_local_mode_three_hop_asserted_head",
            super::extract_source_three_edge_local_mode_three_hop_asserted_head,
        ),
        (
            "extract_source_four_edge_local_object_mode_three_hop_asserted_head",
            super::extract_source_four_edge_local_object_mode_three_hop_asserted_head,
        ),
        (
            "extract_source_four_edge_local_mode_three_hop_asserted_head",
            super::extract_source_four_edge_local_mode_three_hop_asserted_head,
        ),
        (
            "extract_source_four_edge_local_object_mode_four_hop_asserted_head",
            super::extract_source_four_edge_local_object_mode_four_hop_asserted_head,
        ),
        (
            "extract_source_four_edge_local_mode_four_hop_asserted_head",
            super::extract_source_four_edge_local_mode_four_hop_asserted_head,
        ),
        (
            "extract_source_local_object_mode_long_chain_three_hop_asserted_head",
            super::extract_source_local_object_mode_long_chain_three_hop_asserted_head,
        ),
        (
            "extract_source_local_mode_long_chain_three_hop_asserted_head",
            super::extract_source_local_mode_long_chain_three_hop_asserted_head,
        ),
        (
            "extract_source_local_object_mode_long_chain_four_hop_asserted_head",
            super::extract_source_local_object_mode_long_chain_four_hop_asserted_head,
        ),
        (
            "extract_source_local_mode_long_chain_four_hop_asserted_head",
            super::extract_source_local_mode_long_chain_four_hop_asserted_head,
        ),
        (
            "extract_source_local_mode_long_chain_five_hop_asserted_head",
            super::extract_source_local_mode_long_chain_five_hop_asserted_head,
        ),
        (
            "extract_source_local_object_mode_long_chain_five_hop_asserted_head",
            super::extract_source_local_object_mode_long_chain_five_hop_asserted_head,
        ),
    ];
    for (owner, extractor) in prior_extractors {
        assert!(
            extractor(&ast, resolver.module.clone(), &symbols).is_none(),
            "Task 234 source must not be owned by {owner}"
        );
    }

    let prior_owner_ids = [
        "pass_type_elaboration_reserved_variable_type_assertion_001",
        "pass_type_elaboration_reserved_object_variable_type_assertion_001",
        "pass_type_elaboration_local_object_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_local_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_local_object_mode_asserted_head_001",
        "pass_type_elaboration_local_mode_asserted_head_001",
        "pass_type_elaboration_chained_local_object_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_chained_local_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_chained_local_object_mode_asserted_head_001",
        "pass_type_elaboration_chained_local_mode_asserted_head_001",
        "pass_type_elaboration_chained_local_object_mode_radix_asserted_head_001",
        "pass_type_elaboration_chained_local_mode_radix_asserted_head_001",
        "pass_type_elaboration_two_edge_local_object_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_two_edge_local_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_two_edge_local_object_mode_asserted_head_001",
        "pass_type_elaboration_two_edge_local_mode_asserted_head_001",
        "pass_type_elaboration_two_edge_local_object_mode_radix_asserted_head_001",
        "pass_type_elaboration_two_edge_local_mode_radix_asserted_head_001",
        "pass_type_elaboration_three_edge_local_object_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_three_edge_local_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_three_edge_local_object_mode_asserted_head_001",
        "pass_type_elaboration_three_edge_local_mode_asserted_head_001",
        "pass_type_elaboration_three_edge_local_object_mode_radix_asserted_head_001",
        "pass_type_elaboration_three_edge_local_mode_radix_asserted_head_001",
        "pass_type_elaboration_four_edge_local_object_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_four_edge_local_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_four_edge_local_object_mode_asserted_head_001",
        "pass_type_elaboration_four_edge_local_mode_asserted_head_001",
        "pass_type_elaboration_four_edge_local_object_mode_radix_asserted_head_001",
        "pass_type_elaboration_four_edge_local_mode_radix_asserted_head_001",
        "pass_type_elaboration_local_object_mode_long_chain_reserved_variable_type_assertion_001",
        "pass_type_elaboration_local_mode_long_chain_reserved_variable_type_assertion_001",
        "pass_type_elaboration_local_object_mode_long_chain_asserted_head_001",
        "pass_type_elaboration_local_mode_long_chain_asserted_head_001",
        "pass_type_elaboration_local_object_mode_long_chain_radix_asserted_head_001",
        "pass_type_elaboration_local_mode_long_chain_radix_asserted_head_001",
        "pass_type_elaboration_local_object_mode_long_chain_two_hop_asserted_head_001",
        "pass_type_elaboration_local_mode_long_chain_two_hop_asserted_head_001",
        "pass_type_elaboration_two_edge_local_object_mode_two_hop_asserted_head_001",
        "pass_type_elaboration_two_edge_local_mode_two_hop_asserted_head_001",
        "pass_type_elaboration_three_edge_local_object_mode_two_hop_asserted_head_001",
        "pass_type_elaboration_three_edge_local_mode_two_hop_asserted_head_001",
        "pass_type_elaboration_four_edge_local_object_mode_two_hop_asserted_head_001",
        "pass_type_elaboration_four_edge_local_mode_two_hop_asserted_head_001",
        "pass_type_elaboration_three_edge_local_object_mode_three_hop_asserted_head_001",
        "pass_type_elaboration_three_edge_local_mode_three_hop_asserted_head_001",
        "pass_type_elaboration_four_edge_local_object_mode_three_hop_asserted_head_001",
        "pass_type_elaboration_four_edge_local_mode_three_hop_asserted_head_001",
        "pass_type_elaboration_four_edge_local_object_mode_four_hop_asserted_head_001",
        "pass_type_elaboration_four_edge_local_mode_four_hop_asserted_head_001",
        "pass_type_elaboration_local_object_mode_long_chain_three_hop_asserted_head_001",
        "pass_type_elaboration_local_mode_long_chain_three_hop_asserted_head_001",
        "pass_type_elaboration_local_object_mode_long_chain_four_hop_asserted_head_001",
        "pass_type_elaboration_local_mode_long_chain_four_hop_asserted_head_001",
        "pass_type_elaboration_local_mode_long_chain_five_hop_asserted_head_001",
        "pass_type_elaboration_local_object_mode_long_chain_five_hop_asserted_head_001",
    ];
    assert_eq!(prior_owner_ids.len(), 56);
    for owner_id in prior_owner_ids {
        let (owner_ordinal, owner_case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| case.id.0 == owner_id)
            .unwrap_or_else(|| panic!("prior owner fixture {owner_id} must be active"));
        let owner_frontend = run_frontend(&workspace_root, owner_case, owner_ordinal)
            .unwrap_or_else(|error| panic!("owner fixture {owner_id} must parse: {error}"));
        assert!(owner_frontend.diagnostics.is_empty());
        let owner_ast = owner_frontend
            .ast
            .unwrap_or_else(|| panic!("owner fixture {owner_id} must produce an AST"));
        let owner_resolver =
            resolver_symbol_collection(&workspace_root, owner_case, &owner_ast);
        assert!(owner_resolver.detail_keys.is_empty());
        let owner_symbols = augment_type_elaboration_import_summaries(
            &owner_ast,
            &owner_resolver.module,
            owner_resolver.env,
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &owner_ast,
                owner_resolver.module.clone(),
                &owner_symbols,
            ),
            Vec::<String>::new(),
            "prior owner fixture {owner_id} must still reach its active route"
        );
        assert!(
            super::extract_source_local_mode_long_chain_six_hop_asserted_head(
                &owner_ast,
                owner_resolver.module,
                &owner_symbols,
            )
            .is_none(),
            "Task 234 must reject prior owner fixture {owner_id}"
        );
    }
}

#[test]
fn active_local_object_mode_long_chain_six_hop_asserted_head_fixture_consumes_seven_expansions()
{
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
    let plan = build_test_plan(&config).expect("Task 236 repository plan should build");
    let (ordinal, case) = active_type_elaboration_cases(&plan)
        .enumerate()
        .find(|(_, case)| {
            case.id.0 == "pass_type_elaboration_local_object_mode_long_chain_six_hop_asserted_head_001"
        })
        .expect("Task 236 active fixture should be discoverable");
    let frontend = run_frontend(&workspace_root, case, ordinal)
        .expect("Task 236 fixture should run through the real frontend");
    assert!(frontend.diagnostics.is_empty());
    let ast = frontend
        .ast
        .expect("Task 236 fixture should produce a real AST");
    let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
    assert!(resolver.detail_keys.is_empty());
    let symbols =
        augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
    let output = super::source_local_object_mode_long_chain_six_hop_asserted_head_output(
        &ast,
        resolver.module.clone(),
        &symbols,
    )
    .expect("Task 236 real AST should reach the seven-expansion six-hop seam");
    assert_source_reserved_variable_type_assertion_output(&output)
        .expect("Task 236 real AST should preserve every checked payload invariant");
    assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
    assert_eq!(output.subject_binding, BindingId::new(0));
    assert_eq!(output.payload.subject_lookup_ordinal, 1);
    assert_eq!(output.subject_result_input.spelling, "ChainObjectMode6");
    assert_eq!(output.asserted_type_input.spelling, "BaseObjectMode");
    assert_ne!(
        output.subject_result_input.head,
        output.asserted_type_input.head
    );
    assert_eq!(output.term_formula.type_entries().len(), 3);
    assert_eq!(output.term_formula.normalized_types().len(), 1);
    assert!(output.term_formula.candidate_sets().is_empty());
    assert!(output.term_formula.facts().is_empty());
    assert!(output.term_formula.diagnostics().is_empty());
    let chain6 = output
        .payload
        .reserve
        .mode_expansions
        .iter()
        .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("ChainObjectMode6"))
        .map(|(_, expansion)| expansion)
        .expect("Task 236 ChainObjectMode6 expansion should exist");
    assert_eq!(chain6.radix.spelling, "ChainObjectMode5");
    let TypeHeadInput::Symbol(chain5_symbol) = &chain6.radix.head else {
        panic!("Task 236 ChainObjectMode5 first intermediate should be a symbol");
    };
    let chain5 = output
        .payload
        .reserve
        .mode_expansions
        .get(chain5_symbol)
        .expect("Task 236 ChainObjectMode5 expansion should exist");
    assert_eq!(chain5.radix.spelling, "ChainObjectMode4");
    assert_ne!(chain5.radix.head, output.asserted_type_input.head);
    let TypeHeadInput::Symbol(chain4_symbol) = &chain5.radix.head else {
        panic!("Task 236 ChainObjectMode4 second intermediate should be a symbol");
    };
    let chain4 = output
        .payload
        .reserve
        .mode_expansions
        .get(chain4_symbol)
        .expect("Task 236 ChainObjectMode4 expansion should exist");
    assert_eq!(chain4.radix.spelling, "ChainObjectMode3");
    assert_ne!(chain4.radix.head, output.asserted_type_input.head);
    let TypeHeadInput::Symbol(chain3_symbol) = &chain4.radix.head else {
        panic!("Task 236 ChainObjectMode3 third intermediate should be a symbol");
    };
    let chain3 = output
        .payload
        .reserve
        .mode_expansions
        .get(chain3_symbol)
        .expect("Task 236 ChainObjectMode3 expansion should exist");
    assert_eq!(chain3.radix.spelling, "ChainObjectMode2");
    assert_ne!(chain3.radix.head, output.asserted_type_input.head);
    let TypeHeadInput::Symbol(chain2_symbol) = &chain3.radix.head else {
        panic!("Task 236 ChainObjectMode2 fourth intermediate should be a symbol");
    };
    let chain2 = output
        .payload
        .reserve
        .mode_expansions
        .get(chain2_symbol)
        .expect("Task 236 ChainObjectMode2 expansion should exist");
    assert_eq!(chain2.radix.spelling, "ChainObjectMode1");
    assert_ne!(chain2.radix.head, output.asserted_type_input.head);
    let TypeHeadInput::Symbol(chain1_symbol) = &chain2.radix.head else {
        panic!("Task 236 ChainObjectMode1 fifth intermediate should be a symbol");
    };
    let chain1 = output
        .payload
        .reserve
        .mode_expansions
        .get(chain1_symbol)
        .expect("Task 236 ChainObjectMode1 expansion should exist");
    assert_eq!(chain1.radix.spelling, "BaseObjectMode");
    assert_eq!(chain1.radix.head, output.asserted_type_input.head);

    type OwnerExtractor = fn(
        &SurfaceAst,
        ResolverModuleId,
        &SymbolEnv,
    ) -> Option<super::SourceReservedVariableTypeAssertion>;
    let prior_extractors: [(&str, OwnerExtractor); 57] = [
        (
            "extract_source_reserved_variable_type_assertion",
            super::extract_source_reserved_variable_type_assertion,
        ),
        (
            "extract_source_reserved_object_variable_type_assertion",
            super::extract_source_reserved_object_variable_type_assertion,
        ),
        (
            "extract_source_local_object_mode_reserved_variable_type_assertion",
            super::extract_source_local_object_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_local_mode_reserved_variable_type_assertion",
            super::extract_source_local_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_local_object_mode_asserted_head",
            super::extract_source_local_object_mode_asserted_head,
        ),
        (
            "extract_source_local_mode_asserted_head",
            super::extract_source_local_mode_asserted_head,
        ),
        (
            "extract_source_chained_local_object_mode_reserved_variable_type_assertion",
            super::extract_source_chained_local_object_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_chained_local_mode_reserved_variable_type_assertion",
            super::extract_source_chained_local_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_chained_local_object_mode_asserted_head",
            super::extract_source_chained_local_object_mode_asserted_head,
        ),
        (
            "extract_source_chained_local_mode_asserted_head",
            super::extract_source_chained_local_mode_asserted_head,
        ),
        (
            "extract_source_chained_local_object_mode_radix_asserted_head",
            super::extract_source_chained_local_object_mode_radix_asserted_head,
        ),
        (
            "extract_source_chained_local_mode_radix_asserted_head",
            super::extract_source_chained_local_mode_radix_asserted_head,
        ),
        (
            "extract_source_two_edge_local_object_mode_reserved_variable_type_assertion",
            super::extract_source_two_edge_local_object_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_two_edge_local_mode_reserved_variable_type_assertion",
            super::extract_source_two_edge_local_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_two_edge_local_object_mode_asserted_head",
            super::extract_source_two_edge_local_object_mode_asserted_head,
        ),
        (
            "extract_source_two_edge_local_mode_asserted_head",
            super::extract_source_two_edge_local_mode_asserted_head,
        ),
        (
            "extract_source_two_edge_local_object_mode_radix_asserted_head",
            super::extract_source_two_edge_local_object_mode_radix_asserted_head,
        ),
        (
            "extract_source_two_edge_local_mode_radix_asserted_head",
            super::extract_source_two_edge_local_mode_radix_asserted_head,
        ),
        (
            "extract_source_three_edge_local_object_mode_reserved_variable_type_assertion",
            super::extract_source_three_edge_local_object_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_three_edge_local_mode_reserved_variable_type_assertion",
            super::extract_source_three_edge_local_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_three_edge_local_object_mode_asserted_head",
            super::extract_source_three_edge_local_object_mode_asserted_head,
        ),
        (
            "extract_source_three_edge_local_mode_asserted_head",
            super::extract_source_three_edge_local_mode_asserted_head,
        ),
        (
            "extract_source_three_edge_local_object_mode_radix_asserted_head",
            super::extract_source_three_edge_local_object_mode_radix_asserted_head,
        ),
        (
            "extract_source_three_edge_local_mode_radix_asserted_head",
            super::extract_source_three_edge_local_mode_radix_asserted_head,
        ),
        (
            "extract_source_four_edge_local_object_mode_reserved_variable_type_assertion",
            super::extract_source_four_edge_local_object_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_four_edge_local_mode_reserved_variable_type_assertion",
            super::extract_source_four_edge_local_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_four_edge_local_object_mode_asserted_head",
            super::extract_source_four_edge_local_object_mode_asserted_head,
        ),
        (
            "extract_source_four_edge_local_mode_asserted_head",
            super::extract_source_four_edge_local_mode_asserted_head,
        ),
        (
            "extract_source_four_edge_local_object_mode_radix_asserted_head",
            super::extract_source_four_edge_local_object_mode_radix_asserted_head,
        ),
        (
            "extract_source_four_edge_local_mode_radix_asserted_head",
            super::extract_source_four_edge_local_mode_radix_asserted_head,
        ),
        (
            "extract_source_local_object_mode_long_chain_reserved_variable_type_assertion",
            super::extract_source_local_object_mode_long_chain_reserved_variable_type_assertion,
        ),
        (
            "extract_source_local_mode_long_chain_reserved_variable_type_assertion",
            super::extract_source_local_mode_long_chain_reserved_variable_type_assertion,
        ),
        (
            "extract_source_local_object_mode_long_chain_asserted_head",
            super::extract_source_local_object_mode_long_chain_asserted_head,
        ),
        (
            "extract_source_local_mode_long_chain_asserted_head",
            super::extract_source_local_mode_long_chain_asserted_head,
        ),
        (
            "extract_source_local_object_mode_long_chain_radix_asserted_head",
            super::extract_source_local_object_mode_long_chain_radix_asserted_head,
        ),
        (
            "extract_source_local_mode_long_chain_radix_asserted_head",
            super::extract_source_local_mode_long_chain_radix_asserted_head,
        ),
        (
            "extract_source_local_object_mode_long_chain_two_hop_asserted_head",
            super::extract_source_local_object_mode_long_chain_two_hop_asserted_head,
        ),
        (
            "extract_source_local_mode_long_chain_two_hop_asserted_head",
            super::extract_source_local_mode_long_chain_two_hop_asserted_head,
        ),
        (
            "extract_source_two_edge_local_object_mode_two_hop_asserted_head",
            super::extract_source_two_edge_local_object_mode_two_hop_asserted_head,
        ),
        (
            "extract_source_two_edge_local_mode_two_hop_asserted_head",
            super::extract_source_two_edge_local_mode_two_hop_asserted_head,
        ),
        (
            "extract_source_three_edge_local_object_mode_two_hop_asserted_head",
            super::extract_source_three_edge_local_object_mode_two_hop_asserted_head,
        ),
        (
            "extract_source_three_edge_local_mode_two_hop_asserted_head",
            super::extract_source_three_edge_local_mode_two_hop_asserted_head,
        ),
        (
            "extract_source_four_edge_local_object_mode_two_hop_asserted_head",
            super::extract_source_four_edge_local_object_mode_two_hop_asserted_head,
        ),
        (
            "extract_source_four_edge_local_mode_two_hop_asserted_head",
            super::extract_source_four_edge_local_mode_two_hop_asserted_head,
        ),
        (
            "extract_source_three_edge_local_object_mode_three_hop_asserted_head",
            super::extract_source_three_edge_local_object_mode_three_hop_asserted_head,
        ),
        (
            "extract_source_three_edge_local_mode_three_hop_asserted_head",
            super::extract_source_three_edge_local_mode_three_hop_asserted_head,
        ),
        (
            "extract_source_four_edge_local_object_mode_three_hop_asserted_head",
            super::extract_source_four_edge_local_object_mode_three_hop_asserted_head,
        ),
        (
            "extract_source_four_edge_local_mode_three_hop_asserted_head",
            super::extract_source_four_edge_local_mode_three_hop_asserted_head,
        ),
        (
            "extract_source_four_edge_local_object_mode_four_hop_asserted_head",
            super::extract_source_four_edge_local_object_mode_four_hop_asserted_head,
        ),
        (
            "extract_source_four_edge_local_mode_four_hop_asserted_head",
            super::extract_source_four_edge_local_mode_four_hop_asserted_head,
        ),
        (
            "extract_source_local_object_mode_long_chain_three_hop_asserted_head",
            super::extract_source_local_object_mode_long_chain_three_hop_asserted_head,
        ),
        (
            "extract_source_local_mode_long_chain_three_hop_asserted_head",
            super::extract_source_local_mode_long_chain_three_hop_asserted_head,
        ),
        (
            "extract_source_local_object_mode_long_chain_four_hop_asserted_head",
            super::extract_source_local_object_mode_long_chain_four_hop_asserted_head,
        ),
        (
            "extract_source_local_mode_long_chain_four_hop_asserted_head",
            super::extract_source_local_mode_long_chain_four_hop_asserted_head,
        ),
        (
            "extract_source_local_mode_long_chain_five_hop_asserted_head",
            super::extract_source_local_mode_long_chain_five_hop_asserted_head,
        ),
        (
            "extract_source_local_object_mode_long_chain_five_hop_asserted_head",
            super::extract_source_local_object_mode_long_chain_five_hop_asserted_head,
        ),
        (
            "extract_source_local_mode_long_chain_six_hop_asserted_head",
            super::extract_source_local_mode_long_chain_six_hop_asserted_head,
        ),
    ];
    for (owner, extractor) in prior_extractors {
        assert!(
            extractor(&ast, resolver.module.clone(), &symbols).is_none(),
            "Task 236 source must not be owned by {owner}"
        );
    }

    let prior_owner_ids = [
        "pass_type_elaboration_reserved_variable_type_assertion_001",
        "pass_type_elaboration_reserved_object_variable_type_assertion_001",
        "pass_type_elaboration_local_object_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_local_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_local_object_mode_asserted_head_001",
        "pass_type_elaboration_local_mode_asserted_head_001",
        "pass_type_elaboration_chained_local_object_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_chained_local_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_chained_local_object_mode_asserted_head_001",
        "pass_type_elaboration_chained_local_mode_asserted_head_001",
        "pass_type_elaboration_chained_local_object_mode_radix_asserted_head_001",
        "pass_type_elaboration_chained_local_mode_radix_asserted_head_001",
        "pass_type_elaboration_two_edge_local_object_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_two_edge_local_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_two_edge_local_object_mode_asserted_head_001",
        "pass_type_elaboration_two_edge_local_mode_asserted_head_001",
        "pass_type_elaboration_two_edge_local_object_mode_radix_asserted_head_001",
        "pass_type_elaboration_two_edge_local_mode_radix_asserted_head_001",
        "pass_type_elaboration_three_edge_local_object_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_three_edge_local_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_three_edge_local_object_mode_asserted_head_001",
        "pass_type_elaboration_three_edge_local_mode_asserted_head_001",
        "pass_type_elaboration_three_edge_local_object_mode_radix_asserted_head_001",
        "pass_type_elaboration_three_edge_local_mode_radix_asserted_head_001",
        "pass_type_elaboration_four_edge_local_object_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_four_edge_local_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_four_edge_local_object_mode_asserted_head_001",
        "pass_type_elaboration_four_edge_local_mode_asserted_head_001",
        "pass_type_elaboration_four_edge_local_object_mode_radix_asserted_head_001",
        "pass_type_elaboration_four_edge_local_mode_radix_asserted_head_001",
        "pass_type_elaboration_local_object_mode_long_chain_reserved_variable_type_assertion_001",
        "pass_type_elaboration_local_mode_long_chain_reserved_variable_type_assertion_001",
        "pass_type_elaboration_local_object_mode_long_chain_asserted_head_001",
        "pass_type_elaboration_local_mode_long_chain_asserted_head_001",
        "pass_type_elaboration_local_object_mode_long_chain_radix_asserted_head_001",
        "pass_type_elaboration_local_mode_long_chain_radix_asserted_head_001",
        "pass_type_elaboration_local_object_mode_long_chain_two_hop_asserted_head_001",
        "pass_type_elaboration_local_mode_long_chain_two_hop_asserted_head_001",
        "pass_type_elaboration_two_edge_local_object_mode_two_hop_asserted_head_001",
        "pass_type_elaboration_two_edge_local_mode_two_hop_asserted_head_001",
        "pass_type_elaboration_three_edge_local_object_mode_two_hop_asserted_head_001",
        "pass_type_elaboration_three_edge_local_mode_two_hop_asserted_head_001",
        "pass_type_elaboration_four_edge_local_object_mode_two_hop_asserted_head_001",
        "pass_type_elaboration_four_edge_local_mode_two_hop_asserted_head_001",
        "pass_type_elaboration_three_edge_local_object_mode_three_hop_asserted_head_001",
        "pass_type_elaboration_three_edge_local_mode_three_hop_asserted_head_001",
        "pass_type_elaboration_four_edge_local_object_mode_three_hop_asserted_head_001",
        "pass_type_elaboration_four_edge_local_mode_three_hop_asserted_head_001",
        "pass_type_elaboration_four_edge_local_object_mode_four_hop_asserted_head_001",
        "pass_type_elaboration_four_edge_local_mode_four_hop_asserted_head_001",
        "pass_type_elaboration_local_object_mode_long_chain_three_hop_asserted_head_001",
        "pass_type_elaboration_local_mode_long_chain_three_hop_asserted_head_001",
        "pass_type_elaboration_local_object_mode_long_chain_four_hop_asserted_head_001",
        "pass_type_elaboration_local_mode_long_chain_four_hop_asserted_head_001",
        "pass_type_elaboration_local_mode_long_chain_five_hop_asserted_head_001",
        "pass_type_elaboration_local_object_mode_long_chain_five_hop_asserted_head_001",
        "pass_type_elaboration_local_mode_long_chain_six_hop_asserted_head_001",
    ];
    assert_eq!(prior_owner_ids.len(), 57);
    for owner_id in prior_owner_ids {
        let (owner_ordinal, owner_case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| case.id.0 == owner_id)
            .unwrap_or_else(|| panic!("prior owner fixture {owner_id} must be active"));
        let owner_frontend = run_frontend(&workspace_root, owner_case, owner_ordinal)
            .unwrap_or_else(|error| panic!("owner fixture {owner_id} must parse: {error}"));
        assert!(owner_frontend.diagnostics.is_empty());
        let owner_ast = owner_frontend
            .ast
            .unwrap_or_else(|| panic!("owner fixture {owner_id} must produce an AST"));
        let owner_resolver =
            resolver_symbol_collection(&workspace_root, owner_case, &owner_ast);
        assert!(owner_resolver.detail_keys.is_empty());
        let owner_symbols = augment_type_elaboration_import_summaries(
            &owner_ast,
            &owner_resolver.module,
            owner_resolver.env,
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &owner_ast,
                owner_resolver.module.clone(),
                &owner_symbols,
            ),
            Vec::<String>::new(),
            "prior owner fixture {owner_id} must still reach its active route"
        );
        assert!(
            super::extract_source_local_object_mode_long_chain_six_hop_asserted_head(
                &owner_ast,
                owner_resolver.module,
                &owner_symbols,
            )
            .is_none(),
            "Task 236 must reject prior owner fixture {owner_id}"
        );
    }
}

#[test]
fn active_local_object_mode_long_chain_four_hop_asserted_head_fixture_consumes_seven_expansions()
 {
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
    let plan = build_test_plan(&config).expect("Task 229 repository plan should build");
    let (ordinal, case) = active_type_elaboration_cases(&plan)
        .enumerate()
        .find(|(_, case)| {
            case.id.0
                == "pass_type_elaboration_local_object_mode_long_chain_four_hop_asserted_head_001"
        })
        .expect("Task 229 active fixture should be discoverable");
    let frontend = run_frontend(&workspace_root, case, ordinal)
        .expect("Task 229 fixture should run through the real frontend");
    assert!(frontend.diagnostics.is_empty());
    let ast = frontend
        .ast
        .expect("Task 229 fixture should produce a real AST");
    let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
    assert!(resolver.detail_keys.is_empty());
    let symbols =
        augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
    let output = super::source_local_object_mode_long_chain_four_hop_asserted_head_output(
        &ast,
        resolver.module.clone(),
        &symbols,
    )
    .expect("Task 229 real AST should reach the seven-expansion four-hop seam");
    assert_source_reserved_variable_type_assertion_output(&output)
        .expect("Task 229 real AST should preserve every checked payload invariant");
    assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
    assert_eq!(output.subject_binding, BindingId::new(0));
    assert_eq!(output.payload.subject_lookup_ordinal, 1);
    assert_eq!(output.subject_result_input.spelling, "ChainObjectMode6");
    assert_eq!(output.asserted_type_input.spelling, "ChainObjectMode2");
    assert_ne!(
        output.subject_result_input.head,
        output.asserted_type_input.head
    );
    assert_eq!(output.term_formula.type_entries().len(), 3);
    assert_eq!(output.term_formula.normalized_types().len(), 1);
    assert!(output.term_formula.candidate_sets().is_empty());
    assert!(output.term_formula.facts().is_empty());
    assert!(output.term_formula.diagnostics().is_empty());
    let chain6 = output
        .payload
        .reserve
        .mode_expansions
        .iter()
        .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("ChainObjectMode6"))
        .map(|(_, expansion)| expansion)
        .expect("Task 229 ChainObjectMode6 expansion should exist");
    assert_eq!(chain6.radix.spelling, "ChainObjectMode5");
    let TypeHeadInput::Symbol(chain5_symbol) = &chain6.radix.head else {
        panic!("Task 229 ChainObjectMode5 first intermediate should be a symbol");
    };
    let chain5 = output
        .payload
        .reserve
        .mode_expansions
        .get(chain5_symbol)
        .expect("Task 229 ChainObjectMode5 expansion should exist");
    assert_eq!(chain5.radix.spelling, "ChainObjectMode4");
    assert_ne!(chain5.radix.head, output.asserted_type_input.head);
    let TypeHeadInput::Symbol(chain4_symbol) = &chain5.radix.head else {
        panic!("Task 229 ChainObjectMode4 second intermediate should be a symbol");
    };
    let chain4 = output
        .payload
        .reserve
        .mode_expansions
        .get(chain4_symbol)
        .expect("Task 229 ChainObjectMode4 expansion should exist");
    assert_eq!(chain4.radix.spelling, "ChainObjectMode3");
    assert_ne!(chain4.radix.head, output.asserted_type_input.head);
    let TypeHeadInput::Symbol(chain3_symbol) = &chain4.radix.head else {
        panic!("Task 229 ChainObjectMode3 third intermediate should be a symbol");
    };
    let chain3 = output
        .payload
        .reserve
        .mode_expansions
        .get(chain3_symbol)
        .expect("Task 229 ChainObjectMode3 expansion should exist");
    assert_eq!(chain3.radix.spelling, "ChainObjectMode2");
    assert_eq!(chain3.radix.head, output.asserted_type_input.head);

    type OwnerExtractor = fn(
        &SurfaceAst,
        ResolverModuleId,
        &SymbolEnv,
    ) -> Option<super::SourceReservedVariableTypeAssertion>;
    let prior_extractors: [(&str, OwnerExtractor); 53] = [
        (
            "extract_source_reserved_variable_type_assertion",
            super::extract_source_reserved_variable_type_assertion,
        ),
        (
            "extract_source_reserved_object_variable_type_assertion",
            super::extract_source_reserved_object_variable_type_assertion,
        ),
        (
            "extract_source_local_mode_reserved_variable_type_assertion",
            super::extract_source_local_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_local_object_mode_reserved_variable_type_assertion",
            super::extract_source_local_object_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_local_mode_asserted_head",
            super::extract_source_local_mode_asserted_head,
        ),
        (
            "extract_source_local_object_mode_asserted_head",
            super::extract_source_local_object_mode_asserted_head,
        ),
        (
            "extract_source_chained_local_mode_reserved_variable_type_assertion",
            super::extract_source_chained_local_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_chained_local_object_mode_reserved_variable_type_assertion",
            super::extract_source_chained_local_object_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_chained_local_mode_asserted_head",
            super::extract_source_chained_local_mode_asserted_head,
        ),
        (
            "extract_source_chained_local_object_mode_asserted_head",
            super::extract_source_chained_local_object_mode_asserted_head,
        ),
        (
            "extract_source_chained_local_mode_radix_asserted_head",
            super::extract_source_chained_local_mode_radix_asserted_head,
        ),
        (
            "extract_source_chained_local_object_mode_radix_asserted_head",
            super::extract_source_chained_local_object_mode_radix_asserted_head,
        ),
        (
            "extract_source_two_edge_local_mode_reserved_variable_type_assertion",
            super::extract_source_two_edge_local_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_two_edge_local_object_mode_reserved_variable_type_assertion",
            super::extract_source_two_edge_local_object_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_two_edge_local_mode_asserted_head",
            super::extract_source_two_edge_local_mode_asserted_head,
        ),
        (
            "extract_source_two_edge_local_object_mode_asserted_head",
            super::extract_source_two_edge_local_object_mode_asserted_head,
        ),
        (
            "extract_source_two_edge_local_mode_radix_asserted_head",
            super::extract_source_two_edge_local_mode_radix_asserted_head,
        ),
        (
            "extract_source_two_edge_local_object_mode_radix_asserted_head",
            super::extract_source_two_edge_local_object_mode_radix_asserted_head,
        ),
        (
            "extract_source_three_edge_local_mode_reserved_variable_type_assertion",
            super::extract_source_three_edge_local_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_three_edge_local_object_mode_reserved_variable_type_assertion",
            super::extract_source_three_edge_local_object_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_three_edge_local_mode_asserted_head",
            super::extract_source_three_edge_local_mode_asserted_head,
        ),
        (
            "extract_source_three_edge_local_object_mode_asserted_head",
            super::extract_source_three_edge_local_object_mode_asserted_head,
        ),
        (
            "extract_source_three_edge_local_mode_radix_asserted_head",
            super::extract_source_three_edge_local_mode_radix_asserted_head,
        ),
        (
            "extract_source_three_edge_local_object_mode_radix_asserted_head",
            super::extract_source_three_edge_local_object_mode_radix_asserted_head,
        ),
        (
            "extract_source_four_edge_local_mode_reserved_variable_type_assertion",
            super::extract_source_four_edge_local_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_four_edge_local_object_mode_reserved_variable_type_assertion",
            super::extract_source_four_edge_local_object_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_four_edge_local_mode_asserted_head",
            super::extract_source_four_edge_local_mode_asserted_head,
        ),
        (
            "extract_source_four_edge_local_object_mode_asserted_head",
            super::extract_source_four_edge_local_object_mode_asserted_head,
        ),
        (
            "extract_source_four_edge_local_mode_radix_asserted_head",
            super::extract_source_four_edge_local_mode_radix_asserted_head,
        ),
        (
            "extract_source_four_edge_local_object_mode_radix_asserted_head",
            super::extract_source_four_edge_local_object_mode_radix_asserted_head,
        ),
        (
            "extract_source_local_mode_long_chain_reserved_variable_type_assertion",
            super::extract_source_local_mode_long_chain_reserved_variable_type_assertion,
        ),
        (
            "extract_source_local_object_mode_long_chain_reserved_variable_type_assertion",
            super::extract_source_local_object_mode_long_chain_reserved_variable_type_assertion,
        ),
        (
            "extract_source_local_mode_long_chain_asserted_head",
            super::extract_source_local_mode_long_chain_asserted_head,
        ),
        (
            "extract_source_local_object_mode_long_chain_asserted_head",
            super::extract_source_local_object_mode_long_chain_asserted_head,
        ),
        (
            "extract_source_local_mode_long_chain_radix_asserted_head",
            super::extract_source_local_mode_long_chain_radix_asserted_head,
        ),
        (
            "extract_source_local_object_mode_long_chain_radix_asserted_head",
            super::extract_source_local_object_mode_long_chain_radix_asserted_head,
        ),
        (
            "extract_source_local_mode_long_chain_two_hop_asserted_head",
            super::extract_source_local_mode_long_chain_two_hop_asserted_head,
        ),
        (
            "extract_source_local_object_mode_long_chain_two_hop_asserted_head",
            super::extract_source_local_object_mode_long_chain_two_hop_asserted_head,
        ),
        (
            "extract_source_two_edge_local_mode_two_hop_asserted_head",
            super::extract_source_two_edge_local_mode_two_hop_asserted_head,
        ),
        (
            "extract_source_two_edge_local_object_mode_two_hop_asserted_head",
            super::extract_source_two_edge_local_object_mode_two_hop_asserted_head,
        ),
        (
            "extract_source_three_edge_local_mode_two_hop_asserted_head",
            super::extract_source_three_edge_local_mode_two_hop_asserted_head,
        ),
        (
            "extract_source_three_edge_local_object_mode_two_hop_asserted_head",
            super::extract_source_three_edge_local_object_mode_two_hop_asserted_head,
        ),
        (
            "extract_source_four_edge_local_mode_two_hop_asserted_head",
            super::extract_source_four_edge_local_mode_two_hop_asserted_head,
        ),
        (
            "extract_source_four_edge_local_object_mode_two_hop_asserted_head",
            super::extract_source_four_edge_local_object_mode_two_hop_asserted_head,
        ),
        (
            "extract_source_three_edge_local_mode_three_hop_asserted_head",
            super::extract_source_three_edge_local_mode_three_hop_asserted_head,
        ),
        (
            "extract_source_three_edge_local_object_mode_three_hop_asserted_head",
            super::extract_source_three_edge_local_object_mode_three_hop_asserted_head,
        ),
        (
            "extract_source_four_edge_local_mode_three_hop_asserted_head",
            super::extract_source_four_edge_local_mode_three_hop_asserted_head,
        ),
        (
            "extract_source_four_edge_local_object_mode_three_hop_asserted_head",
            super::extract_source_four_edge_local_object_mode_three_hop_asserted_head,
        ),
        (
            "extract_source_four_edge_local_mode_four_hop_asserted_head",
            super::extract_source_four_edge_local_mode_four_hop_asserted_head,
        ),
        (
            "extract_source_four_edge_local_object_mode_four_hop_asserted_head",
            super::extract_source_four_edge_local_object_mode_four_hop_asserted_head,
        ),
        (
            "extract_source_local_mode_long_chain_three_hop_asserted_head",
            super::extract_source_local_mode_long_chain_three_hop_asserted_head,
        ),
        (
            "extract_source_local_object_mode_long_chain_three_hop_asserted_head",
            super::extract_source_local_object_mode_long_chain_three_hop_asserted_head,
        ),
        (
            "extract_source_local_mode_long_chain_four_hop_asserted_head",
            super::extract_source_local_mode_long_chain_four_hop_asserted_head,
        ),
    ];
    for (owner, extractor) in prior_extractors {
        assert!(
            extractor(&ast, resolver.module.clone(), &symbols).is_none(),
            "Task 229 source must not be owned by {owner}"
        );
    }

    let prior_owner_ids = [
        "pass_type_elaboration_reserved_variable_type_assertion_001",
        "pass_type_elaboration_reserved_object_variable_type_assertion_001",
        "pass_type_elaboration_local_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_local_object_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_local_mode_asserted_head_001",
        "pass_type_elaboration_local_object_mode_asserted_head_001",
        "pass_type_elaboration_chained_local_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_chained_local_object_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_chained_local_mode_asserted_head_001",
        "pass_type_elaboration_chained_local_object_mode_asserted_head_001",
        "pass_type_elaboration_chained_local_mode_radix_asserted_head_001",
        "pass_type_elaboration_chained_local_object_mode_radix_asserted_head_001",
        "pass_type_elaboration_two_edge_local_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_two_edge_local_object_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_two_edge_local_mode_asserted_head_001",
        "pass_type_elaboration_two_edge_local_object_mode_asserted_head_001",
        "pass_type_elaboration_two_edge_local_mode_radix_asserted_head_001",
        "pass_type_elaboration_two_edge_local_object_mode_radix_asserted_head_001",
        "pass_type_elaboration_three_edge_local_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_three_edge_local_object_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_three_edge_local_mode_asserted_head_001",
        "pass_type_elaboration_three_edge_local_object_mode_asserted_head_001",
        "pass_type_elaboration_three_edge_local_mode_radix_asserted_head_001",
        "pass_type_elaboration_three_edge_local_object_mode_radix_asserted_head_001",
        "pass_type_elaboration_four_edge_local_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_four_edge_local_object_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_four_edge_local_mode_asserted_head_001",
        "pass_type_elaboration_four_edge_local_object_mode_asserted_head_001",
        "pass_type_elaboration_four_edge_local_mode_radix_asserted_head_001",
        "pass_type_elaboration_four_edge_local_object_mode_radix_asserted_head_001",
        "pass_type_elaboration_local_mode_long_chain_reserved_variable_type_assertion_001",
        "pass_type_elaboration_local_object_mode_long_chain_reserved_variable_type_assertion_001",
        "pass_type_elaboration_local_mode_long_chain_asserted_head_001",
        "pass_type_elaboration_local_object_mode_long_chain_asserted_head_001",
        "pass_type_elaboration_local_mode_long_chain_radix_asserted_head_001",
        "pass_type_elaboration_local_object_mode_long_chain_radix_asserted_head_001",
        "pass_type_elaboration_local_mode_long_chain_two_hop_asserted_head_001",
        "pass_type_elaboration_local_object_mode_long_chain_two_hop_asserted_head_001",
        "pass_type_elaboration_two_edge_local_mode_two_hop_asserted_head_001",
        "pass_type_elaboration_two_edge_local_object_mode_two_hop_asserted_head_001",
        "pass_type_elaboration_three_edge_local_mode_two_hop_asserted_head_001",
        "pass_type_elaboration_three_edge_local_object_mode_two_hop_asserted_head_001",
        "pass_type_elaboration_four_edge_local_mode_two_hop_asserted_head_001",
        "pass_type_elaboration_four_edge_local_object_mode_two_hop_asserted_head_001",
        "pass_type_elaboration_three_edge_local_mode_three_hop_asserted_head_001",
        "pass_type_elaboration_three_edge_local_object_mode_three_hop_asserted_head_001",
        "pass_type_elaboration_four_edge_local_mode_three_hop_asserted_head_001",
        "pass_type_elaboration_four_edge_local_object_mode_three_hop_asserted_head_001",
        "pass_type_elaboration_four_edge_local_mode_four_hop_asserted_head_001",
        "pass_type_elaboration_four_edge_local_object_mode_four_hop_asserted_head_001",
        "pass_type_elaboration_local_mode_long_chain_three_hop_asserted_head_001",
        "pass_type_elaboration_local_object_mode_long_chain_three_hop_asserted_head_001",
        "pass_type_elaboration_local_mode_long_chain_four_hop_asserted_head_001",
    ];
    assert_eq!(prior_owner_ids.len(), 53);
    for owner_id in prior_owner_ids {
        let (owner_ordinal, owner_case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| case.id.0 == owner_id)
            .unwrap_or_else(|| panic!("prior owner fixture {owner_id} must be active"));
        let owner_frontend = run_frontend(&workspace_root, owner_case, owner_ordinal)
            .unwrap_or_else(|error| panic!("owner fixture {owner_id} must parse: {error}"));
        assert!(owner_frontend.diagnostics.is_empty());
        let owner_ast = owner_frontend
            .ast
            .unwrap_or_else(|| panic!("owner fixture {owner_id} must produce an AST"));
        let owner_resolver =
            resolver_symbol_collection(&workspace_root, owner_case, &owner_ast);
        assert!(owner_resolver.detail_keys.is_empty());
        let owner_symbols = augment_type_elaboration_import_summaries(
            &owner_ast,
            &owner_resolver.module,
            owner_resolver.env,
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &owner_ast,
                owner_resolver.module.clone(),
                &owner_symbols,
            ),
            Vec::<String>::new(),
            "prior owner fixture {owner_id} must still reach its active route"
        );
        assert!(
            super::extract_source_local_object_mode_long_chain_four_hop_asserted_head(
                &owner_ast,
                owner_resolver.module,
                &owner_symbols,
            )
            .is_none(),
            "Task 229 must reject prior owner fixture {owner_id}"
        );
    }
}

#[test]
fn active_local_object_mode_long_chain_three_hop_asserted_head_fixture_consumes_seven_expansions()
 {
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
    let plan = build_test_plan(&config).expect("Task 227 repository plan should build");
    let (ordinal, case) = active_type_elaboration_cases(&plan)
        .enumerate()
        .find(|(_, case)| {
            case.id.0 == "pass_type_elaboration_local_object_mode_long_chain_three_hop_asserted_head_001"
        })
        .expect("Task 227 active fixture should be discoverable");
    let frontend = run_frontend(&workspace_root, case, ordinal)
        .expect("Task 227 fixture should run through the real frontend");
    assert!(frontend.diagnostics.is_empty());
    let ast = frontend
        .ast
        .expect("Task 227 fixture should produce a real AST");
    let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
    assert!(resolver.detail_keys.is_empty());
    let symbols =
        augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
    let output = super::source_local_object_mode_long_chain_three_hop_asserted_head_output(
        &ast,
        resolver.module.clone(),
        &symbols,
    )
    .expect("Task 227 real AST should reach the seven-expansion object three-hop seam");
    assert_source_reserved_variable_type_assertion_output(&output)
        .expect("Task 227 real AST should preserve every checked payload invariant");
    assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
    assert_eq!(output.subject_binding, BindingId::new(0));
    assert_eq!(output.payload.subject_lookup_ordinal, 1);
    assert_eq!(output.subject_result_input.spelling, "ChainObjectMode6");
    assert_eq!(output.asserted_type_input.spelling, "ChainObjectMode3");
    assert_ne!(
        output.subject_result_input.head,
        output.asserted_type_input.head
    );
    assert_eq!(output.term_formula.type_entries().len(), 3);
    assert_eq!(output.term_formula.normalized_types().len(), 1);
    assert!(output.term_formula.candidate_sets().is_empty());
    assert!(output.term_formula.facts().is_empty());
    assert!(output.term_formula.diagnostics().is_empty());
    let chain6 = output
        .payload
        .reserve
        .mode_expansions
        .iter()
        .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("ChainObjectMode6"))
        .map(|(_, expansion)| expansion)
        .expect("Task 227 ChainObjectMode6 expansion should exist");
    assert_eq!(chain6.radix.spelling, "ChainObjectMode5");
    let TypeHeadInput::Symbol(chain5_symbol) = &chain6.radix.head else {
        panic!("Task 227 ChainObjectMode5 first intermediate should be a symbol");
    };
    let chain5 = output
        .payload
        .reserve
        .mode_expansions
        .get(chain5_symbol)
        .expect("Task 227 ChainObjectMode5 expansion should exist");
    assert_eq!(chain5.radix.spelling, "ChainObjectMode4");
    assert_ne!(chain5.radix.head, output.asserted_type_input.head);
    let TypeHeadInput::Symbol(chain4_symbol) = &chain5.radix.head else {
        panic!("Task 227 ChainObjectMode4 second intermediate should be a symbol");
    };
    let chain4 = output
        .payload
        .reserve
        .mode_expansions
        .get(chain4_symbol)
        .expect("Task 227 ChainObjectMode4 expansion should exist");
    assert_eq!(chain4.radix.spelling, "ChainObjectMode3");
    assert_eq!(chain4.radix.head, output.asserted_type_input.head);

    type OwnerExtractor = fn(
        &SurfaceAst,
        ResolverModuleId,
        &SymbolEnv,
    ) -> Option<super::SourceReservedVariableTypeAssertion>;
    let prior_extractors: [(&str, OwnerExtractor); 51] = [
        (
            "extract_source_reserved_variable_type_assertion",
            super::extract_source_reserved_variable_type_assertion,
        ),
        (
            "extract_source_reserved_object_variable_type_assertion",
            super::extract_source_reserved_object_variable_type_assertion,
        ),
        (
            "extract_source_local_mode_reserved_variable_type_assertion",
            super::extract_source_local_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_local_object_mode_reserved_variable_type_assertion",
            super::extract_source_local_object_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_local_mode_asserted_head",
            super::extract_source_local_mode_asserted_head,
        ),
        (
            "extract_source_local_object_mode_asserted_head",
            super::extract_source_local_object_mode_asserted_head,
        ),
        (
            "extract_source_chained_local_mode_reserved_variable_type_assertion",
            super::extract_source_chained_local_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_chained_local_object_mode_reserved_variable_type_assertion",
            super::extract_source_chained_local_object_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_chained_local_mode_asserted_head",
            super::extract_source_chained_local_mode_asserted_head,
        ),
        (
            "extract_source_chained_local_object_mode_asserted_head",
            super::extract_source_chained_local_object_mode_asserted_head,
        ),
        (
            "extract_source_chained_local_mode_radix_asserted_head",
            super::extract_source_chained_local_mode_radix_asserted_head,
        ),
        (
            "extract_source_chained_local_object_mode_radix_asserted_head",
            super::extract_source_chained_local_object_mode_radix_asserted_head,
        ),
        (
            "extract_source_two_edge_local_mode_reserved_variable_type_assertion",
            super::extract_source_two_edge_local_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_two_edge_local_object_mode_reserved_variable_type_assertion",
            super::extract_source_two_edge_local_object_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_two_edge_local_mode_asserted_head",
            super::extract_source_two_edge_local_mode_asserted_head,
        ),
        (
            "extract_source_two_edge_local_object_mode_asserted_head",
            super::extract_source_two_edge_local_object_mode_asserted_head,
        ),
        (
            "extract_source_two_edge_local_mode_radix_asserted_head",
            super::extract_source_two_edge_local_mode_radix_asserted_head,
        ),
        (
            "extract_source_two_edge_local_object_mode_radix_asserted_head",
            super::extract_source_two_edge_local_object_mode_radix_asserted_head,
        ),
        (
            "extract_source_three_edge_local_mode_reserved_variable_type_assertion",
            super::extract_source_three_edge_local_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_three_edge_local_object_mode_reserved_variable_type_assertion",
            super::extract_source_three_edge_local_object_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_three_edge_local_mode_asserted_head",
            super::extract_source_three_edge_local_mode_asserted_head,
        ),
        (
            "extract_source_three_edge_local_object_mode_asserted_head",
            super::extract_source_three_edge_local_object_mode_asserted_head,
        ),
        (
            "extract_source_three_edge_local_mode_radix_asserted_head",
            super::extract_source_three_edge_local_mode_radix_asserted_head,
        ),
        (
            "extract_source_three_edge_local_object_mode_radix_asserted_head",
            super::extract_source_three_edge_local_object_mode_radix_asserted_head,
        ),
        (
            "extract_source_four_edge_local_mode_reserved_variable_type_assertion",
            super::extract_source_four_edge_local_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_four_edge_local_object_mode_reserved_variable_type_assertion",
            super::extract_source_four_edge_local_object_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_four_edge_local_mode_asserted_head",
            super::extract_source_four_edge_local_mode_asserted_head,
        ),
        (
            "extract_source_four_edge_local_object_mode_asserted_head",
            super::extract_source_four_edge_local_object_mode_asserted_head,
        ),
        (
            "extract_source_four_edge_local_mode_radix_asserted_head",
            super::extract_source_four_edge_local_mode_radix_asserted_head,
        ),
        (
            "extract_source_four_edge_local_object_mode_radix_asserted_head",
            super::extract_source_four_edge_local_object_mode_radix_asserted_head,
        ),
        (
            "extract_source_local_mode_long_chain_reserved_variable_type_assertion",
            super::extract_source_local_mode_long_chain_reserved_variable_type_assertion,
        ),
        (
            "extract_source_local_object_mode_long_chain_reserved_variable_type_assertion",
            super::extract_source_local_object_mode_long_chain_reserved_variable_type_assertion,
        ),
        (
            "extract_source_local_mode_long_chain_asserted_head",
            super::extract_source_local_mode_long_chain_asserted_head,
        ),
        (
            "extract_source_local_object_mode_long_chain_asserted_head",
            super::extract_source_local_object_mode_long_chain_asserted_head,
        ),
        (
            "extract_source_local_mode_long_chain_radix_asserted_head",
            super::extract_source_local_mode_long_chain_radix_asserted_head,
        ),
        (
            "extract_source_local_object_mode_long_chain_radix_asserted_head",
            super::extract_source_local_object_mode_long_chain_radix_asserted_head,
        ),
        (
            "extract_source_local_mode_long_chain_two_hop_asserted_head",
            super::extract_source_local_mode_long_chain_two_hop_asserted_head,
        ),
        (
            "extract_source_local_object_mode_long_chain_two_hop_asserted_head",
            super::extract_source_local_object_mode_long_chain_two_hop_asserted_head,
        ),
        (
            "extract_source_two_edge_local_mode_two_hop_asserted_head",
            super::extract_source_two_edge_local_mode_two_hop_asserted_head,
        ),
        (
            "extract_source_two_edge_local_object_mode_two_hop_asserted_head",
            super::extract_source_two_edge_local_object_mode_two_hop_asserted_head,
        ),
        (
            "extract_source_three_edge_local_mode_two_hop_asserted_head",
            super::extract_source_three_edge_local_mode_two_hop_asserted_head,
        ),
        (
            "extract_source_three_edge_local_object_mode_two_hop_asserted_head",
            super::extract_source_three_edge_local_object_mode_two_hop_asserted_head,
        ),
        (
            "extract_source_four_edge_local_mode_two_hop_asserted_head",
            super::extract_source_four_edge_local_mode_two_hop_asserted_head,
        ),
        (
            "extract_source_four_edge_local_object_mode_two_hop_asserted_head",
            super::extract_source_four_edge_local_object_mode_two_hop_asserted_head,
        ),
        (
            "extract_source_three_edge_local_mode_three_hop_asserted_head",
            super::extract_source_three_edge_local_mode_three_hop_asserted_head,
        ),
        (
            "extract_source_three_edge_local_object_mode_three_hop_asserted_head",
            super::extract_source_three_edge_local_object_mode_three_hop_asserted_head,
        ),
        (
            "extract_source_four_edge_local_mode_three_hop_asserted_head",
            super::extract_source_four_edge_local_mode_three_hop_asserted_head,
        ),
        (
            "extract_source_four_edge_local_object_mode_three_hop_asserted_head",
            super::extract_source_four_edge_local_object_mode_three_hop_asserted_head,
        ),
        (
            "extract_source_four_edge_local_mode_four_hop_asserted_head",
            super::extract_source_four_edge_local_mode_four_hop_asserted_head,
        ),
        (
            "extract_source_four_edge_local_object_mode_four_hop_asserted_head",
            super::extract_source_four_edge_local_object_mode_four_hop_asserted_head,
        ),
        (
            "extract_source_local_mode_long_chain_three_hop_asserted_head",
            super::extract_source_local_mode_long_chain_three_hop_asserted_head,
        ),
    ];
    for (owner, extractor) in prior_extractors {
        assert!(
            extractor(&ast, resolver.module.clone(), &symbols).is_none(),
            "Task 227 source must not be owned by {owner}"
        );
    }

    let prior_owner_ids = [
        "pass_type_elaboration_reserved_variable_type_assertion_001",
        "pass_type_elaboration_reserved_object_variable_type_assertion_001",
        "pass_type_elaboration_local_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_local_object_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_local_mode_asserted_head_001",
        "pass_type_elaboration_local_object_mode_asserted_head_001",
        "pass_type_elaboration_chained_local_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_chained_local_object_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_chained_local_mode_asserted_head_001",
        "pass_type_elaboration_chained_local_object_mode_asserted_head_001",
        "pass_type_elaboration_chained_local_mode_radix_asserted_head_001",
        "pass_type_elaboration_chained_local_object_mode_radix_asserted_head_001",
        "pass_type_elaboration_two_edge_local_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_two_edge_local_object_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_two_edge_local_mode_asserted_head_001",
        "pass_type_elaboration_two_edge_local_object_mode_asserted_head_001",
        "pass_type_elaboration_two_edge_local_mode_radix_asserted_head_001",
        "pass_type_elaboration_two_edge_local_object_mode_radix_asserted_head_001",
        "pass_type_elaboration_three_edge_local_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_three_edge_local_object_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_three_edge_local_mode_asserted_head_001",
        "pass_type_elaboration_three_edge_local_object_mode_asserted_head_001",
        "pass_type_elaboration_three_edge_local_mode_radix_asserted_head_001",
        "pass_type_elaboration_three_edge_local_object_mode_radix_asserted_head_001",
        "pass_type_elaboration_four_edge_local_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_four_edge_local_object_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_four_edge_local_mode_asserted_head_001",
        "pass_type_elaboration_four_edge_local_object_mode_asserted_head_001",
        "pass_type_elaboration_four_edge_local_mode_radix_asserted_head_001",
        "pass_type_elaboration_four_edge_local_object_mode_radix_asserted_head_001",
        "pass_type_elaboration_local_mode_long_chain_reserved_variable_type_assertion_001",
        "pass_type_elaboration_local_object_mode_long_chain_reserved_variable_type_assertion_001",
        "pass_type_elaboration_local_mode_long_chain_asserted_head_001",
        "pass_type_elaboration_local_object_mode_long_chain_asserted_head_001",
        "pass_type_elaboration_local_mode_long_chain_radix_asserted_head_001",
        "pass_type_elaboration_local_object_mode_long_chain_radix_asserted_head_001",
        "pass_type_elaboration_local_mode_long_chain_two_hop_asserted_head_001",
        "pass_type_elaboration_local_object_mode_long_chain_two_hop_asserted_head_001",
        "pass_type_elaboration_two_edge_local_mode_two_hop_asserted_head_001",
        "pass_type_elaboration_two_edge_local_object_mode_two_hop_asserted_head_001",
        "pass_type_elaboration_three_edge_local_mode_two_hop_asserted_head_001",
        "pass_type_elaboration_three_edge_local_object_mode_two_hop_asserted_head_001",
        "pass_type_elaboration_four_edge_local_mode_two_hop_asserted_head_001",
        "pass_type_elaboration_four_edge_local_object_mode_two_hop_asserted_head_001",
        "pass_type_elaboration_three_edge_local_mode_three_hop_asserted_head_001",
        "pass_type_elaboration_three_edge_local_object_mode_three_hop_asserted_head_001",
        "pass_type_elaboration_four_edge_local_mode_three_hop_asserted_head_001",
        "pass_type_elaboration_four_edge_local_object_mode_three_hop_asserted_head_001",
        "pass_type_elaboration_four_edge_local_mode_four_hop_asserted_head_001",
        "pass_type_elaboration_four_edge_local_object_mode_four_hop_asserted_head_001",
        "pass_type_elaboration_local_mode_long_chain_three_hop_asserted_head_001",
    ];
    assert_eq!(prior_owner_ids.len(), 51);
    for owner_id in prior_owner_ids {
        let (owner_ordinal, owner_case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| case.id.0 == owner_id)
            .unwrap_or_else(|| panic!("prior owner fixture {owner_id} must be active"));
        let owner_frontend = run_frontend(&workspace_root, owner_case, owner_ordinal)
            .unwrap_or_else(|error| panic!("owner fixture {owner_id} must parse: {error}"));
        assert!(owner_frontend.diagnostics.is_empty());
        let owner_ast = owner_frontend
            .ast
            .unwrap_or_else(|| panic!("owner fixture {owner_id} must produce an AST"));
        let owner_resolver =
            resolver_symbol_collection(&workspace_root, owner_case, &owner_ast);
        assert!(owner_resolver.detail_keys.is_empty());
        let owner_symbols = augment_type_elaboration_import_summaries(
            &owner_ast,
            &owner_resolver.module,
            owner_resolver.env,
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &owner_ast,
                owner_resolver.module.clone(),
                &owner_symbols,
            ),
            Vec::<String>::new(),
            "prior owner fixture {owner_id} must still reach its active route"
        );
        assert!(
            super::extract_source_local_object_mode_long_chain_three_hop_asserted_head(
                &owner_ast,
                owner_resolver.module,
                &owner_symbols,
            )
            .is_none(),
            "Task 227 must reject prior owner fixture {owner_id}"
        );
    }
}

#[test]
fn active_local_mode_long_chain_two_hop_asserted_head_fixture_consumes_seven_expansions() {
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
    let plan = build_test_plan(&config).expect("Task 224 repository plan should build");
    let (ordinal, case) = active_type_elaboration_cases(&plan)
        .enumerate()
        .find(|(_, case)| {
            case.id.0 == "pass_type_elaboration_local_mode_long_chain_two_hop_asserted_head_001"
        })
        .expect("Task 224 active fixture should be discoverable");
    let frontend = run_frontend(&workspace_root, case, ordinal)
        .expect("Task 224 fixture should run through the real frontend");
    assert!(frontend.diagnostics.is_empty());
    let ast = frontend
        .ast
        .expect("Task 224 fixture should produce a real AST");
    let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
    assert!(resolver.detail_keys.is_empty());
    let symbols =
        augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
    let output = super::source_local_mode_long_chain_two_hop_asserted_head_output(
        &ast,
        resolver.module.clone(),
        &symbols,
    )
    .expect("Task 224 real AST should reach the seven-expansion two-hop seam");
    assert_source_reserved_variable_type_assertion_output(&output)
        .expect("Task 224 real AST should preserve every checked payload invariant");
    assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
    assert_eq!(output.subject_binding, BindingId::new(0));
    assert_eq!(output.payload.subject_lookup_ordinal, 1);
    assert_eq!(output.subject_result_input.spelling, "ChainMode6");
    assert_eq!(output.asserted_type_input.spelling, "ChainMode4");
    assert_ne!(
        output.subject_result_input.head,
        output.asserted_type_input.head
    );
    assert_eq!(output.term_formula.type_entries().len(), 3);
    assert_eq!(output.term_formula.normalized_types().len(), 1);
    assert!(output.term_formula.candidate_sets().is_empty());
    assert!(output.term_formula.facts().is_empty());
    assert!(output.term_formula.diagnostics().is_empty());
    let chain6 = output
        .payload
        .reserve
        .mode_expansions
        .iter()
        .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("ChainMode6"))
        .map(|(_, expansion)| expansion)
        .expect("Task 224 ChainMode6 expansion should exist");
    assert_eq!(chain6.radix.spelling, "ChainMode5");
    let TypeHeadInput::Symbol(chain5_symbol) = &chain6.radix.head else {
        panic!("Task 224 ChainMode5 intermediate should be a symbol");
    };
    let chain5 = output
        .payload
        .reserve
        .mode_expansions
        .get(chain5_symbol)
        .expect("Task 224 ChainMode5 expansion should exist");
    assert_eq!(chain5.radix.spelling, "ChainMode4");
    assert_eq!(chain5.radix.head, output.asserted_type_input.head);

    type OwnerExtractor = fn(
        &SurfaceAst,
        ResolverModuleId,
        &SymbolEnv,
    ) -> Option<super::SourceReservedVariableTypeAssertion>;
    let prior_extractors: [(&str, OwnerExtractor); 48] = [
        (
            "extract_source_reserved_variable_type_assertion",
            super::extract_source_reserved_variable_type_assertion,
        ),
        (
            "extract_source_reserved_object_variable_type_assertion",
            super::extract_source_reserved_object_variable_type_assertion,
        ),
        (
            "extract_source_local_mode_reserved_variable_type_assertion",
            super::extract_source_local_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_local_object_mode_reserved_variable_type_assertion",
            super::extract_source_local_object_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_local_mode_asserted_head",
            super::extract_source_local_mode_asserted_head,
        ),
        (
            "extract_source_local_object_mode_asserted_head",
            super::extract_source_local_object_mode_asserted_head,
        ),
        (
            "extract_source_chained_local_mode_reserved_variable_type_assertion",
            super::extract_source_chained_local_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_chained_local_object_mode_reserved_variable_type_assertion",
            super::extract_source_chained_local_object_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_chained_local_mode_asserted_head",
            super::extract_source_chained_local_mode_asserted_head,
        ),
        (
            "extract_source_chained_local_object_mode_asserted_head",
            super::extract_source_chained_local_object_mode_asserted_head,
        ),
        (
            "extract_source_chained_local_mode_radix_asserted_head",
            super::extract_source_chained_local_mode_radix_asserted_head,
        ),
        (
            "extract_source_chained_local_object_mode_radix_asserted_head",
            super::extract_source_chained_local_object_mode_radix_asserted_head,
        ),
        (
            "extract_source_two_edge_local_mode_reserved_variable_type_assertion",
            super::extract_source_two_edge_local_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_two_edge_local_object_mode_reserved_variable_type_assertion",
            super::extract_source_two_edge_local_object_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_two_edge_local_mode_asserted_head",
            super::extract_source_two_edge_local_mode_asserted_head,
        ),
        (
            "extract_source_two_edge_local_object_mode_asserted_head",
            super::extract_source_two_edge_local_object_mode_asserted_head,
        ),
        (
            "extract_source_two_edge_local_mode_radix_asserted_head",
            super::extract_source_two_edge_local_mode_radix_asserted_head,
        ),
        (
            "extract_source_two_edge_local_object_mode_radix_asserted_head",
            super::extract_source_two_edge_local_object_mode_radix_asserted_head,
        ),
        (
            "extract_source_three_edge_local_mode_reserved_variable_type_assertion",
            super::extract_source_three_edge_local_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_three_edge_local_object_mode_reserved_variable_type_assertion",
            super::extract_source_three_edge_local_object_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_three_edge_local_mode_asserted_head",
            super::extract_source_three_edge_local_mode_asserted_head,
        ),
        (
            "extract_source_three_edge_local_object_mode_asserted_head",
            super::extract_source_three_edge_local_object_mode_asserted_head,
        ),
        (
            "extract_source_three_edge_local_mode_radix_asserted_head",
            super::extract_source_three_edge_local_mode_radix_asserted_head,
        ),
        (
            "extract_source_three_edge_local_object_mode_radix_asserted_head",
            super::extract_source_three_edge_local_object_mode_radix_asserted_head,
        ),
        (
            "extract_source_four_edge_local_mode_reserved_variable_type_assertion",
            super::extract_source_four_edge_local_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_four_edge_local_object_mode_reserved_variable_type_assertion",
            super::extract_source_four_edge_local_object_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_four_edge_local_mode_asserted_head",
            super::extract_source_four_edge_local_mode_asserted_head,
        ),
        (
            "extract_source_four_edge_local_object_mode_asserted_head",
            super::extract_source_four_edge_local_object_mode_asserted_head,
        ),
        (
            "extract_source_four_edge_local_mode_radix_asserted_head",
            super::extract_source_four_edge_local_mode_radix_asserted_head,
        ),
        (
            "extract_source_four_edge_local_object_mode_radix_asserted_head",
            super::extract_source_four_edge_local_object_mode_radix_asserted_head,
        ),
        (
            "extract_source_local_mode_long_chain_reserved_variable_type_assertion",
            super::extract_source_local_mode_long_chain_reserved_variable_type_assertion,
        ),
        (
            "extract_source_local_object_mode_long_chain_reserved_variable_type_assertion",
            super::extract_source_local_object_mode_long_chain_reserved_variable_type_assertion,
        ),
        (
            "extract_source_local_mode_long_chain_asserted_head",
            super::extract_source_local_mode_long_chain_asserted_head,
        ),
        (
            "extract_source_local_object_mode_long_chain_asserted_head",
            super::extract_source_local_object_mode_long_chain_asserted_head,
        ),
        (
            "extract_source_local_mode_long_chain_radix_asserted_head",
            super::extract_source_local_mode_long_chain_radix_asserted_head,
        ),
        (
            "extract_source_local_object_mode_long_chain_radix_asserted_head",
            super::extract_source_local_object_mode_long_chain_radix_asserted_head,
        ),
        (
            "extract_source_two_edge_local_mode_two_hop_asserted_head",
            super::extract_source_two_edge_local_mode_two_hop_asserted_head,
        ),
        (
            "extract_source_two_edge_local_object_mode_two_hop_asserted_head",
            super::extract_source_two_edge_local_object_mode_two_hop_asserted_head,
        ),
        (
            "extract_source_three_edge_local_mode_two_hop_asserted_head",
            super::extract_source_three_edge_local_mode_two_hop_asserted_head,
        ),
        (
            "extract_source_three_edge_local_object_mode_two_hop_asserted_head",
            super::extract_source_three_edge_local_object_mode_two_hop_asserted_head,
        ),
        (
            "extract_source_four_edge_local_mode_two_hop_asserted_head",
            super::extract_source_four_edge_local_mode_two_hop_asserted_head,
        ),
        (
            "extract_source_four_edge_local_object_mode_two_hop_asserted_head",
            super::extract_source_four_edge_local_object_mode_two_hop_asserted_head,
        ),
        (
            "extract_source_three_edge_local_mode_three_hop_asserted_head",
            super::extract_source_three_edge_local_mode_three_hop_asserted_head,
        ),
        (
            "extract_source_three_edge_local_object_mode_three_hop_asserted_head",
            super::extract_source_three_edge_local_object_mode_three_hop_asserted_head,
        ),
        (
            "extract_source_four_edge_local_mode_three_hop_asserted_head",
            super::extract_source_four_edge_local_mode_three_hop_asserted_head,
        ),
        (
            "extract_source_four_edge_local_object_mode_three_hop_asserted_head",
            super::extract_source_four_edge_local_object_mode_three_hop_asserted_head,
        ),
        (
            "extract_source_four_edge_local_mode_four_hop_asserted_head",
            super::extract_source_four_edge_local_mode_four_hop_asserted_head,
        ),
        (
            "extract_source_four_edge_local_object_mode_four_hop_asserted_head",
            super::extract_source_four_edge_local_object_mode_four_hop_asserted_head,
        ),
    ];
    for (owner, extractor) in prior_extractors {
        assert!(
            extractor(&ast, resolver.module.clone(), &symbols).is_none(),
            "Task 224 source must not be owned by {owner}"
        );
    }

    let prior_owner_ids = [
        "pass_type_elaboration_reserved_variable_type_assertion_001",
        "pass_type_elaboration_reserved_object_variable_type_assertion_001",
        "pass_type_elaboration_local_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_local_object_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_local_mode_asserted_head_001",
        "pass_type_elaboration_local_object_mode_asserted_head_001",
        "pass_type_elaboration_chained_local_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_chained_local_object_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_chained_local_mode_asserted_head_001",
        "pass_type_elaboration_chained_local_object_mode_asserted_head_001",
        "pass_type_elaboration_chained_local_mode_radix_asserted_head_001",
        "pass_type_elaboration_chained_local_object_mode_radix_asserted_head_001",
        "pass_type_elaboration_two_edge_local_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_two_edge_local_object_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_two_edge_local_mode_asserted_head_001",
        "pass_type_elaboration_two_edge_local_object_mode_asserted_head_001",
        "pass_type_elaboration_two_edge_local_mode_radix_asserted_head_001",
        "pass_type_elaboration_two_edge_local_object_mode_radix_asserted_head_001",
        "pass_type_elaboration_three_edge_local_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_three_edge_local_object_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_three_edge_local_mode_asserted_head_001",
        "pass_type_elaboration_three_edge_local_object_mode_asserted_head_001",
        "pass_type_elaboration_three_edge_local_mode_radix_asserted_head_001",
        "pass_type_elaboration_three_edge_local_object_mode_radix_asserted_head_001",
        "pass_type_elaboration_four_edge_local_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_four_edge_local_object_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_four_edge_local_mode_asserted_head_001",
        "pass_type_elaboration_four_edge_local_object_mode_asserted_head_001",
        "pass_type_elaboration_four_edge_local_mode_radix_asserted_head_001",
        "pass_type_elaboration_four_edge_local_object_mode_radix_asserted_head_001",
        "pass_type_elaboration_local_mode_long_chain_reserved_variable_type_assertion_001",
        "pass_type_elaboration_local_object_mode_long_chain_reserved_variable_type_assertion_001",
        "pass_type_elaboration_local_mode_long_chain_asserted_head_001",
        "pass_type_elaboration_local_object_mode_long_chain_asserted_head_001",
        "pass_type_elaboration_local_mode_long_chain_radix_asserted_head_001",
        "pass_type_elaboration_local_object_mode_long_chain_radix_asserted_head_001",
        "pass_type_elaboration_two_edge_local_mode_two_hop_asserted_head_001",
        "pass_type_elaboration_two_edge_local_object_mode_two_hop_asserted_head_001",
        "pass_type_elaboration_three_edge_local_mode_two_hop_asserted_head_001",
        "pass_type_elaboration_three_edge_local_object_mode_two_hop_asserted_head_001",
        "pass_type_elaboration_four_edge_local_mode_two_hop_asserted_head_001",
        "pass_type_elaboration_four_edge_local_object_mode_two_hop_asserted_head_001",
        "pass_type_elaboration_three_edge_local_mode_three_hop_asserted_head_001",
        "pass_type_elaboration_three_edge_local_object_mode_three_hop_asserted_head_001",
        "pass_type_elaboration_four_edge_local_mode_three_hop_asserted_head_001",
        "pass_type_elaboration_four_edge_local_object_mode_three_hop_asserted_head_001",
        "pass_type_elaboration_four_edge_local_mode_four_hop_asserted_head_001",
        "pass_type_elaboration_four_edge_local_object_mode_four_hop_asserted_head_001",
    ];
    assert_eq!(prior_owner_ids.len(), 48);
    for owner_id in prior_owner_ids {
        let (owner_ordinal, owner_case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| case.id.0 == owner_id)
            .unwrap_or_else(|| panic!("prior owner fixture {owner_id} must be active"));
        let owner_frontend = run_frontend(&workspace_root, owner_case, owner_ordinal)
            .unwrap_or_else(|error| panic!("owner fixture {owner_id} must parse: {error}"));
        assert!(owner_frontend.diagnostics.is_empty());
        let owner_ast = owner_frontend
            .ast
            .unwrap_or_else(|| panic!("owner fixture {owner_id} must produce an AST"));
        let owner_resolver =
            resolver_symbol_collection(&workspace_root, owner_case, &owner_ast);
        assert!(owner_resolver.detail_keys.is_empty());
        let owner_symbols = augment_type_elaboration_import_summaries(
            &owner_ast,
            &owner_resolver.module,
            owner_resolver.env,
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &owner_ast,
                owner_resolver.module.clone(),
                &owner_symbols,
            ),
            Vec::<String>::new(),
            "prior owner fixture {owner_id} must still reach its active route"
        );
        assert!(
            super::extract_source_local_mode_long_chain_two_hop_asserted_head(
                &owner_ast,
                owner_resolver.module,
                &owner_symbols,
            )
            .is_none(),
            "Task 224 must reject prior owner fixture {owner_id}"
        );
    }
}

#[test]
fn active_local_object_mode_long_chain_two_hop_asserted_head_fixture_consumes_seven_expansions()
{
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
    let plan = build_test_plan(&config).expect("Task 225 repository plan should build");
    let (ordinal, case) = active_type_elaboration_cases(&plan)
        .enumerate()
        .find(|(_, case)| {
            case.id.0 == "pass_type_elaboration_local_object_mode_long_chain_two_hop_asserted_head_001"
        })
        .expect("Task 225 active fixture should be discoverable");
    let frontend = run_frontend(&workspace_root, case, ordinal)
        .expect("Task 225 fixture should run through the real frontend");
    assert!(frontend.diagnostics.is_empty());
    let ast = frontend
        .ast
        .expect("Task 225 fixture should produce a real AST");
    let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
    assert!(resolver.detail_keys.is_empty());
    let symbols =
        augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
    let output = super::source_local_object_mode_long_chain_two_hop_asserted_head_output(
        &ast,
        resolver.module.clone(),
        &symbols,
    )
    .expect("Task 225 real AST should reach the seven-expansion two-hop seam");
    assert_source_reserved_variable_type_assertion_output(&output)
        .expect("Task 225 real AST should preserve every checked payload invariant");
    assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
    assert_eq!(output.subject_binding, BindingId::new(0));
    assert_eq!(output.payload.subject_lookup_ordinal, 1);
    assert_eq!(output.subject_result_input.spelling, "ChainObjectMode6");
    assert_eq!(output.asserted_type_input.spelling, "ChainObjectMode4");
    assert_ne!(
        output.subject_result_input.head,
        output.asserted_type_input.head
    );
    assert_eq!(output.term_formula.type_entries().len(), 3);
    assert_eq!(output.term_formula.normalized_types().len(), 1);
    let (_, normalized) = output
        .term_formula
        .normalized_types()
        .iter()
        .next()
        .expect("Task 225 normalized object type should exist");
    assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
    assert!(output.term_formula.candidate_sets().is_empty());
    assert!(output.term_formula.facts().is_empty());
    assert!(output.term_formula.diagnostics().is_empty());
    let chain6 = output
        .payload
        .reserve
        .mode_expansions
        .iter()
        .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("ChainObjectMode6"))
        .map(|(_, expansion)| expansion)
        .expect("Task 225 ChainObjectMode6 expansion should exist");
    assert_eq!(chain6.radix.spelling, "ChainObjectMode5");
    let TypeHeadInput::Symbol(chain5_symbol) = &chain6.radix.head else {
        panic!("Task 225 ChainObjectMode5 intermediate should be a symbol");
    };
    let chain5 = output
        .payload
        .reserve
        .mode_expansions
        .get(chain5_symbol)
        .expect("Task 225 ChainObjectMode5 expansion should exist");
    assert_eq!(chain5.radix.spelling, "ChainObjectMode4");
    assert_eq!(chain5.radix.head, output.asserted_type_input.head);

    type OwnerExtractor = fn(
        &SurfaceAst,
        ResolverModuleId,
        &SymbolEnv,
    ) -> Option<super::SourceReservedVariableTypeAssertion>;
    let prior_extractors: [(&str, OwnerExtractor); 49] = [
        (
            "extract_source_reserved_variable_type_assertion",
            super::extract_source_reserved_variable_type_assertion,
        ),
        (
            "extract_source_reserved_object_variable_type_assertion",
            super::extract_source_reserved_object_variable_type_assertion,
        ),
        (
            "extract_source_local_mode_reserved_variable_type_assertion",
            super::extract_source_local_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_local_object_mode_reserved_variable_type_assertion",
            super::extract_source_local_object_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_local_mode_asserted_head",
            super::extract_source_local_mode_asserted_head,
        ),
        (
            "extract_source_local_object_mode_asserted_head",
            super::extract_source_local_object_mode_asserted_head,
        ),
        (
            "extract_source_chained_local_mode_reserved_variable_type_assertion",
            super::extract_source_chained_local_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_chained_local_object_mode_reserved_variable_type_assertion",
            super::extract_source_chained_local_object_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_chained_local_mode_asserted_head",
            super::extract_source_chained_local_mode_asserted_head,
        ),
        (
            "extract_source_chained_local_object_mode_asserted_head",
            super::extract_source_chained_local_object_mode_asserted_head,
        ),
        (
            "extract_source_chained_local_mode_radix_asserted_head",
            super::extract_source_chained_local_mode_radix_asserted_head,
        ),
        (
            "extract_source_chained_local_object_mode_radix_asserted_head",
            super::extract_source_chained_local_object_mode_radix_asserted_head,
        ),
        (
            "extract_source_two_edge_local_mode_reserved_variable_type_assertion",
            super::extract_source_two_edge_local_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_two_edge_local_object_mode_reserved_variable_type_assertion",
            super::extract_source_two_edge_local_object_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_two_edge_local_mode_asserted_head",
            super::extract_source_two_edge_local_mode_asserted_head,
        ),
        (
            "extract_source_two_edge_local_object_mode_asserted_head",
            super::extract_source_two_edge_local_object_mode_asserted_head,
        ),
        (
            "extract_source_two_edge_local_mode_radix_asserted_head",
            super::extract_source_two_edge_local_mode_radix_asserted_head,
        ),
        (
            "extract_source_two_edge_local_object_mode_radix_asserted_head",
            super::extract_source_two_edge_local_object_mode_radix_asserted_head,
        ),
        (
            "extract_source_three_edge_local_mode_reserved_variable_type_assertion",
            super::extract_source_three_edge_local_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_three_edge_local_object_mode_reserved_variable_type_assertion",
            super::extract_source_three_edge_local_object_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_three_edge_local_mode_asserted_head",
            super::extract_source_three_edge_local_mode_asserted_head,
        ),
        (
            "extract_source_three_edge_local_object_mode_asserted_head",
            super::extract_source_three_edge_local_object_mode_asserted_head,
        ),
        (
            "extract_source_three_edge_local_mode_radix_asserted_head",
            super::extract_source_three_edge_local_mode_radix_asserted_head,
        ),
        (
            "extract_source_three_edge_local_object_mode_radix_asserted_head",
            super::extract_source_three_edge_local_object_mode_radix_asserted_head,
        ),
        (
            "extract_source_four_edge_local_mode_reserved_variable_type_assertion",
            super::extract_source_four_edge_local_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_four_edge_local_object_mode_reserved_variable_type_assertion",
            super::extract_source_four_edge_local_object_mode_reserved_variable_type_assertion,
        ),
        (
            "extract_source_four_edge_local_mode_asserted_head",
            super::extract_source_four_edge_local_mode_asserted_head,
        ),
        (
            "extract_source_four_edge_local_object_mode_asserted_head",
            super::extract_source_four_edge_local_object_mode_asserted_head,
        ),
        (
            "extract_source_four_edge_local_mode_radix_asserted_head",
            super::extract_source_four_edge_local_mode_radix_asserted_head,
        ),
        (
            "extract_source_four_edge_local_object_mode_radix_asserted_head",
            super::extract_source_four_edge_local_object_mode_radix_asserted_head,
        ),
        (
            "extract_source_local_mode_long_chain_reserved_variable_type_assertion",
            super::extract_source_local_mode_long_chain_reserved_variable_type_assertion,
        ),
        (
            "extract_source_local_object_mode_long_chain_reserved_variable_type_assertion",
            super::extract_source_local_object_mode_long_chain_reserved_variable_type_assertion,
        ),
        (
            "extract_source_local_mode_long_chain_asserted_head",
            super::extract_source_local_mode_long_chain_asserted_head,
        ),
        (
            "extract_source_local_object_mode_long_chain_asserted_head",
            super::extract_source_local_object_mode_long_chain_asserted_head,
        ),
        (
            "extract_source_local_mode_long_chain_radix_asserted_head",
            super::extract_source_local_mode_long_chain_radix_asserted_head,
        ),
        (
            "extract_source_local_object_mode_long_chain_radix_asserted_head",
            super::extract_source_local_object_mode_long_chain_radix_asserted_head,
        ),
        (
            "extract_source_two_edge_local_mode_two_hop_asserted_head",
            super::extract_source_two_edge_local_mode_two_hop_asserted_head,
        ),
        (
            "extract_source_two_edge_local_object_mode_two_hop_asserted_head",
            super::extract_source_two_edge_local_object_mode_two_hop_asserted_head,
        ),
        (
            "extract_source_three_edge_local_mode_two_hop_asserted_head",
            super::extract_source_three_edge_local_mode_two_hop_asserted_head,
        ),
        (
            "extract_source_three_edge_local_object_mode_two_hop_asserted_head",
            super::extract_source_three_edge_local_object_mode_two_hop_asserted_head,
        ),
        (
            "extract_source_four_edge_local_mode_two_hop_asserted_head",
            super::extract_source_four_edge_local_mode_two_hop_asserted_head,
        ),
        (
            "extract_source_four_edge_local_object_mode_two_hop_asserted_head",
            super::extract_source_four_edge_local_object_mode_two_hop_asserted_head,
        ),
        (
            "extract_source_three_edge_local_mode_three_hop_asserted_head",
            super::extract_source_three_edge_local_mode_three_hop_asserted_head,
        ),
        (
            "extract_source_three_edge_local_object_mode_three_hop_asserted_head",
            super::extract_source_three_edge_local_object_mode_three_hop_asserted_head,
        ),
        (
            "extract_source_four_edge_local_mode_three_hop_asserted_head",
            super::extract_source_four_edge_local_mode_three_hop_asserted_head,
        ),
        (
            "extract_source_four_edge_local_object_mode_three_hop_asserted_head",
            super::extract_source_four_edge_local_object_mode_three_hop_asserted_head,
        ),
        (
            "extract_source_four_edge_local_mode_four_hop_asserted_head",
            super::extract_source_four_edge_local_mode_four_hop_asserted_head,
        ),
        (
            "extract_source_four_edge_local_object_mode_four_hop_asserted_head",
            super::extract_source_four_edge_local_object_mode_four_hop_asserted_head,
        ),
        (
            "extract_source_local_mode_long_chain_two_hop_asserted_head",
            super::extract_source_local_mode_long_chain_two_hop_asserted_head,
        ),
    ];
    for (owner, extractor) in prior_extractors {
        assert!(
            extractor(&ast, resolver.module.clone(), &symbols).is_none(),
            "Task 225 source must not be owned by {owner}"
        );
    }

    let prior_owner_ids = [
        "pass_type_elaboration_reserved_variable_type_assertion_001",
        "pass_type_elaboration_reserved_object_variable_type_assertion_001",
        "pass_type_elaboration_local_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_local_object_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_local_mode_asserted_head_001",
        "pass_type_elaboration_local_object_mode_asserted_head_001",
        "pass_type_elaboration_chained_local_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_chained_local_object_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_chained_local_mode_asserted_head_001",
        "pass_type_elaboration_chained_local_object_mode_asserted_head_001",
        "pass_type_elaboration_chained_local_mode_radix_asserted_head_001",
        "pass_type_elaboration_chained_local_object_mode_radix_asserted_head_001",
        "pass_type_elaboration_two_edge_local_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_two_edge_local_object_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_two_edge_local_mode_asserted_head_001",
        "pass_type_elaboration_two_edge_local_object_mode_asserted_head_001",
        "pass_type_elaboration_two_edge_local_mode_radix_asserted_head_001",
        "pass_type_elaboration_two_edge_local_object_mode_radix_asserted_head_001",
        "pass_type_elaboration_three_edge_local_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_three_edge_local_object_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_three_edge_local_mode_asserted_head_001",
        "pass_type_elaboration_three_edge_local_object_mode_asserted_head_001",
        "pass_type_elaboration_three_edge_local_mode_radix_asserted_head_001",
        "pass_type_elaboration_three_edge_local_object_mode_radix_asserted_head_001",
        "pass_type_elaboration_four_edge_local_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_four_edge_local_object_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_four_edge_local_mode_asserted_head_001",
        "pass_type_elaboration_four_edge_local_object_mode_asserted_head_001",
        "pass_type_elaboration_four_edge_local_mode_radix_asserted_head_001",
        "pass_type_elaboration_four_edge_local_object_mode_radix_asserted_head_001",
        "pass_type_elaboration_local_mode_long_chain_reserved_variable_type_assertion_001",
        "pass_type_elaboration_local_object_mode_long_chain_reserved_variable_type_assertion_001",
        "pass_type_elaboration_local_mode_long_chain_asserted_head_001",
        "pass_type_elaboration_local_object_mode_long_chain_asserted_head_001",
        "pass_type_elaboration_local_mode_long_chain_radix_asserted_head_001",
        "pass_type_elaboration_local_object_mode_long_chain_radix_asserted_head_001",
        "pass_type_elaboration_two_edge_local_mode_two_hop_asserted_head_001",
        "pass_type_elaboration_two_edge_local_object_mode_two_hop_asserted_head_001",
        "pass_type_elaboration_three_edge_local_mode_two_hop_asserted_head_001",
        "pass_type_elaboration_three_edge_local_object_mode_two_hop_asserted_head_001",
        "pass_type_elaboration_four_edge_local_mode_two_hop_asserted_head_001",
        "pass_type_elaboration_four_edge_local_object_mode_two_hop_asserted_head_001",
        "pass_type_elaboration_three_edge_local_mode_three_hop_asserted_head_001",
        "pass_type_elaboration_three_edge_local_object_mode_three_hop_asserted_head_001",
        "pass_type_elaboration_four_edge_local_mode_three_hop_asserted_head_001",
        "pass_type_elaboration_four_edge_local_object_mode_three_hop_asserted_head_001",
        "pass_type_elaboration_four_edge_local_mode_four_hop_asserted_head_001",
        "pass_type_elaboration_four_edge_local_object_mode_four_hop_asserted_head_001",
        "pass_type_elaboration_local_mode_long_chain_two_hop_asserted_head_001",
    ];
    assert_eq!(prior_owner_ids.len(), 49);
    for owner_id in prior_owner_ids {
        let (owner_ordinal, owner_case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| case.id.0 == owner_id)
            .unwrap_or_else(|| panic!("prior owner fixture {owner_id} must be active"));
        let owner_frontend = run_frontend(&workspace_root, owner_case, owner_ordinal)
            .unwrap_or_else(|error| panic!("owner fixture {owner_id} must parse: {error}"));
        assert!(owner_frontend.diagnostics.is_empty());
        let owner_ast = owner_frontend
            .ast
            .unwrap_or_else(|| panic!("owner fixture {owner_id} must produce an AST"));
        let owner_resolver =
            resolver_symbol_collection(&workspace_root, owner_case, &owner_ast);
        assert!(owner_resolver.detail_keys.is_empty());
        let owner_symbols = augment_type_elaboration_import_summaries(
            &owner_ast,
            &owner_resolver.module,
            owner_resolver.env,
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &owner_ast,
                owner_resolver.module.clone(),
                &owner_symbols,
            ),
            Vec::<String>::new(),
            "prior owner fixture {owner_id} must still reach its active route"
        );
        assert!(
            super::extract_source_local_object_mode_long_chain_two_hop_asserted_head(
                &owner_ast,
                owner_resolver.module,
                &owner_symbols,
            )
            .is_none(),
            "Task 225 must reject prior owner fixture {owner_id}"
        );
    }
}

#[test]
fn active_local_mode_long_chain_radix_asserted_head_fixture_consumes_seven_expansions() {
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
    let plan = build_test_plan(&config).expect("Task 209 repository plan should build");
    let (ordinal, case) = active_type_elaboration_cases(&plan)
        .enumerate()
        .find(|(_, case)| {
            case.id.0 == "pass_type_elaboration_local_mode_long_chain_radix_asserted_head_001"
        })
        .expect("Task 209 active fixture should be discoverable");
    let frontend = run_frontend(&workspace_root, case, ordinal)
        .expect("Task 209 fixture should run through the real frontend");
    assert!(frontend.diagnostics.is_empty());
    let ast = frontend
        .ast
        .expect("Task 209 fixture should produce an AST");
    let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
    assert!(resolver.detail_keys.is_empty());
    let symbols =
        augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
    let output = super::source_local_mode_long_chain_radix_asserted_head_output(
        &ast,
        resolver.module,
        &symbols,
    )
    .expect("Task 209 real AST should reach the seven-expansion asserted-head seam");
    assert_source_reserved_variable_type_assertion_output(&output)
        .expect("Task 209 real AST should preserve every checked payload invariant");
    assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
    assert_eq!(output.subject_binding, BindingId::new(0));
    assert_eq!(output.payload.subject_lookup_ordinal, 1);
    assert_eq!(output.term_formula.type_entries().len(), 3);
    assert_eq!(output.term_formula.normalized_types().len(), 1);
    assert_eq!(output.subject_result_input.spelling, "ChainMode6");
    assert_eq!(output.asserted_type_input.spelling, "ChainMode5");
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
    let binding_expansion = output
        .payload
        .reserve
        .mode_expansions
        .iter()
        .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("ChainMode6"))
        .map(|(_, expansion)| expansion)
        .expect("Task 209 ChainMode6 binding expansion should exist");
    assert_eq!(binding_expansion.radix.spelling, "ChainMode5");
    assert_eq!(
        binding_expansion.radix.head,
        output.asserted_type_input.head
    );
    let terminal = output
        .payload
        .reserve
        .mode_expansions
        .iter()
        .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("BaseMode"))
        .map(|(_, expansion)| &expansion.radix)
        .expect("Task 209 BaseMode terminal expansion should exist");
    let (_, normalized) = output
        .term_formula
        .normalized_types()
        .iter()
        .next()
        .expect("Task 209 normalized set type should exist");
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

    let preexisting_owner_ids = [
        "pass_type_elaboration_reserved_variable_type_assertion_001",
        "pass_type_elaboration_reserved_object_variable_type_assertion_001",
        "pass_type_elaboration_local_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_local_object_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_local_mode_asserted_head_001",
        "pass_type_elaboration_local_object_mode_asserted_head_001",
        "pass_type_elaboration_chained_local_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_chained_local_object_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_chained_local_mode_asserted_head_001",
        "pass_type_elaboration_chained_local_object_mode_asserted_head_001",
        "pass_type_elaboration_chained_local_mode_radix_asserted_head_001",
        "pass_type_elaboration_chained_local_object_mode_radix_asserted_head_001",
        "pass_type_elaboration_two_edge_local_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_two_edge_local_object_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_two_edge_local_mode_asserted_head_001",
        "pass_type_elaboration_two_edge_local_object_mode_asserted_head_001",
        "pass_type_elaboration_two_edge_local_mode_radix_asserted_head_001",
        "pass_type_elaboration_two_edge_local_object_mode_radix_asserted_head_001",
        "pass_type_elaboration_three_edge_local_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_three_edge_local_object_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_three_edge_local_mode_asserted_head_001",
        "pass_type_elaboration_three_edge_local_object_mode_asserted_head_001",
        "pass_type_elaboration_three_edge_local_mode_radix_asserted_head_001",
        "pass_type_elaboration_three_edge_local_object_mode_radix_asserted_head_001",
        "pass_type_elaboration_four_edge_local_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_four_edge_local_object_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_four_edge_local_mode_asserted_head_001",
        "pass_type_elaboration_four_edge_local_object_mode_asserted_head_001",
        "pass_type_elaboration_four_edge_local_mode_radix_asserted_head_001",
        "pass_type_elaboration_four_edge_local_object_mode_radix_asserted_head_001",
        "pass_type_elaboration_local_mode_long_chain_reserved_variable_type_assertion_001",
        "pass_type_elaboration_local_object_mode_long_chain_reserved_variable_type_assertion_001",
        "pass_type_elaboration_local_mode_long_chain_asserted_head_001",
        "pass_type_elaboration_local_object_mode_long_chain_asserted_head_001",
    ];
    assert_eq!(preexisting_owner_ids.len(), 34);
    for owner_id in preexisting_owner_ids {
        let (owner_ordinal, owner_case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| case.id.0 == owner_id)
            .unwrap_or_else(|| panic!("pre-existing owner fixture {owner_id} must be active"));
        let owner_frontend = run_frontend(&workspace_root, owner_case, owner_ordinal)
            .unwrap_or_else(|error| panic!("owner fixture {owner_id} must parse: {error}"));
        assert!(
            owner_frontend.diagnostics.is_empty(),
            "owner fixture {owner_id} must remain frontend-clean"
        );
        let owner_ast = owner_frontend
            .ast
            .unwrap_or_else(|| panic!("owner fixture {owner_id} must produce an AST"));
        let owner_resolver =
            resolver_symbol_collection(&workspace_root, owner_case, &owner_ast);
        assert!(
            owner_resolver.detail_keys.is_empty(),
            "owner fixture {owner_id} must remain resolver-clean"
        );
        let owner_symbols = augment_type_elaboration_import_summaries(
            &owner_ast,
            &owner_resolver.module,
            owner_resolver.env,
        );
        let owner_self_extracts = match owner_id {
            "pass_type_elaboration_reserved_variable_type_assertion_001" => {
                extract_source_reserved_variable_type_assertion(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_reserved_object_variable_type_assertion_001" => {
                super::extract_source_reserved_object_variable_type_assertion(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_local_mode_reserved_variable_type_assertion_001" => {
                extract_source_local_mode_reserved_variable_type_assertion(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_local_object_mode_reserved_variable_type_assertion_001" => {
                extract_source_local_object_mode_reserved_variable_type_assertion(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_local_mode_asserted_head_001" => {
                extract_source_local_mode_asserted_head(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_local_object_mode_asserted_head_001" => {
                extract_source_local_object_mode_asserted_head(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_chained_local_mode_reserved_variable_type_assertion_001" => {
                extract_source_chained_local_mode_reserved_variable_type_assertion(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_chained_local_object_mode_reserved_variable_type_assertion_001" => {
                extract_source_chained_local_object_mode_reserved_variable_type_assertion(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_chained_local_mode_asserted_head_001" => {
                extract_source_chained_local_mode_asserted_head(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_chained_local_object_mode_asserted_head_001" => {
                extract_source_chained_local_object_mode_asserted_head(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_chained_local_mode_radix_asserted_head_001" => {
                extract_source_chained_local_mode_radix_asserted_head(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_chained_local_object_mode_radix_asserted_head_001" => {
                extract_source_chained_local_object_mode_radix_asserted_head(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_two_edge_local_mode_reserved_variable_type_assertion_001" => {
                extract_source_two_edge_local_mode_reserved_variable_type_assertion(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_two_edge_local_object_mode_reserved_variable_type_assertion_001" => {
                extract_source_two_edge_local_object_mode_reserved_variable_type_assertion(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_two_edge_local_mode_asserted_head_001" => {
                extract_source_two_edge_local_mode_asserted_head(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_two_edge_local_object_mode_asserted_head_001" => {
                extract_source_two_edge_local_object_mode_asserted_head(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_two_edge_local_mode_radix_asserted_head_001" => {
                extract_source_two_edge_local_mode_radix_asserted_head(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_two_edge_local_object_mode_radix_asserted_head_001" => {
                extract_source_two_edge_local_object_mode_radix_asserted_head(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_three_edge_local_mode_reserved_variable_type_assertion_001" => {
                extract_source_three_edge_local_mode_reserved_variable_type_assertion(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_three_edge_local_object_mode_reserved_variable_type_assertion_001" => {
                extract_source_three_edge_local_object_mode_reserved_variable_type_assertion(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_three_edge_local_mode_asserted_head_001" => {
                extract_source_three_edge_local_mode_asserted_head(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_three_edge_local_object_mode_asserted_head_001" => {
                extract_source_three_edge_local_object_mode_asserted_head(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_three_edge_local_mode_radix_asserted_head_001" => {
                extract_source_three_edge_local_mode_radix_asserted_head(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_three_edge_local_object_mode_radix_asserted_head_001" => {
                extract_source_three_edge_local_object_mode_radix_asserted_head(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_four_edge_local_mode_reserved_variable_type_assertion_001" => {
                extract_source_four_edge_local_mode_reserved_variable_type_assertion(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_four_edge_local_object_mode_reserved_variable_type_assertion_001" => {
                extract_source_four_edge_local_object_mode_reserved_variable_type_assertion(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_four_edge_local_mode_asserted_head_001" => {
                super::extract_source_four_edge_local_mode_asserted_head(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_four_edge_local_object_mode_asserted_head_001" => {
                super::extract_source_four_edge_local_object_mode_asserted_head(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_four_edge_local_mode_radix_asserted_head_001" => {
                extract_source_four_edge_local_mode_radix_asserted_head(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_four_edge_local_object_mode_radix_asserted_head_001" => {
                extract_source_four_edge_local_object_mode_radix_asserted_head(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_local_mode_long_chain_reserved_variable_type_assertion_001" => {
                extract_source_local_mode_long_chain_reserved_variable_type_assertion(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_local_object_mode_long_chain_reserved_variable_type_assertion_001" => {
                extract_source_local_object_mode_long_chain_reserved_variable_type_assertion(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_local_mode_long_chain_asserted_head_001" => {
                super::extract_source_local_mode_long_chain_asserted_head(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_local_object_mode_long_chain_asserted_head_001" => {
                super::extract_source_local_object_mode_long_chain_asserted_head(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            _ => unreachable!("undeclared pre-existing owner fixture {owner_id}"),
        };
        assert!(
            owner_self_extracts,
            "pre-existing owner fixture {owner_id} must non-vacuously reach its own extractor"
        );
        assert!(
            extract_source_local_mode_long_chain_radix_asserted_head(
                &owner_ast,
                owner_resolver.module,
                &owner_symbols,
            )
            .is_none(),
            "Task 209 must reject pre-existing owner fixture {owner_id}"
        );
    }
}

#[test]
fn active_local_object_mode_long_chain_radix_asserted_head_fixture_consumes_seven_expansions() {
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
    let plan = build_test_plan(&config).expect("Task 210 repository plan should build");
    let (ordinal, case) = active_type_elaboration_cases(&plan)
        .enumerate()
        .find(|(_, case)| {
            case.id.0
                == "pass_type_elaboration_local_object_mode_long_chain_radix_asserted_head_001"
        })
        .expect("Task 210 active fixture should be discoverable");
    let frontend = run_frontend(&workspace_root, case, ordinal)
        .expect("Task 210 fixture should run through the real frontend");
    assert!(frontend.diagnostics.is_empty());
    let ast = frontend
        .ast
        .expect("Task 210 fixture should produce an AST");
    let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
    assert!(resolver.detail_keys.is_empty());
    let symbols =
        augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
    let output = super::source_local_object_mode_long_chain_radix_asserted_head_output(
        &ast,
        resolver.module,
        &symbols,
    )
    .expect("Task 210 real AST should reach the seven-expansion asserted-head seam");
    assert_source_reserved_variable_type_assertion_output(&output)
        .expect("Task 210 real AST should preserve every checked payload invariant");
    assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
    assert_eq!(output.subject_binding, BindingId::new(0));
    assert_eq!(output.payload.subject_lookup_ordinal, 1);
    assert_eq!(output.term_formula.type_entries().len(), 3);
    assert_eq!(output.term_formula.normalized_types().len(), 1);
    assert_eq!(output.subject_result_input.spelling, "ChainObjectMode6");
    assert_eq!(output.asserted_type_input.spelling, "ChainObjectMode5");
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
    let binding_expansion = output
        .payload
        .reserve
        .mode_expansions
        .iter()
        .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("ChainObjectMode6"))
        .map(|(_, expansion)| expansion)
        .expect("Task 210 ChainObjectMode6 binding expansion should exist");
    assert_eq!(binding_expansion.radix.spelling, "ChainObjectMode5");
    assert_eq!(
        binding_expansion.radix.head,
        output.asserted_type_input.head
    );
    let terminal = output
        .payload
        .reserve
        .mode_expansions
        .iter()
        .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("BaseObjectMode"))
        .map(|(_, expansion)| &expansion.radix)
        .expect("Task 210 BaseObjectMode terminal expansion should exist");
    let (_, normalized) = output
        .term_formula
        .normalized_types()
        .iter()
        .next()
        .expect("Task 210 normalized object type should exist");
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

    let preexisting_owner_ids = [
        "pass_type_elaboration_reserved_variable_type_assertion_001",
        "pass_type_elaboration_reserved_object_variable_type_assertion_001",
        "pass_type_elaboration_local_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_local_object_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_local_mode_asserted_head_001",
        "pass_type_elaboration_local_object_mode_asserted_head_001",
        "pass_type_elaboration_chained_local_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_chained_local_object_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_chained_local_mode_asserted_head_001",
        "pass_type_elaboration_chained_local_object_mode_asserted_head_001",
        "pass_type_elaboration_chained_local_mode_radix_asserted_head_001",
        "pass_type_elaboration_chained_local_object_mode_radix_asserted_head_001",
        "pass_type_elaboration_two_edge_local_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_two_edge_local_object_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_two_edge_local_mode_asserted_head_001",
        "pass_type_elaboration_two_edge_local_object_mode_asserted_head_001",
        "pass_type_elaboration_two_edge_local_mode_radix_asserted_head_001",
        "pass_type_elaboration_two_edge_local_object_mode_radix_asserted_head_001",
        "pass_type_elaboration_three_edge_local_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_three_edge_local_object_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_three_edge_local_mode_asserted_head_001",
        "pass_type_elaboration_three_edge_local_object_mode_asserted_head_001",
        "pass_type_elaboration_three_edge_local_mode_radix_asserted_head_001",
        "pass_type_elaboration_three_edge_local_object_mode_radix_asserted_head_001",
        "pass_type_elaboration_four_edge_local_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_four_edge_local_object_mode_reserved_variable_type_assertion_001",
        "pass_type_elaboration_four_edge_local_mode_asserted_head_001",
        "pass_type_elaboration_four_edge_local_object_mode_asserted_head_001",
        "pass_type_elaboration_four_edge_local_mode_radix_asserted_head_001",
        "pass_type_elaboration_four_edge_local_object_mode_radix_asserted_head_001",
        "pass_type_elaboration_local_mode_long_chain_reserved_variable_type_assertion_001",
        "pass_type_elaboration_local_object_mode_long_chain_reserved_variable_type_assertion_001",
        "pass_type_elaboration_local_mode_long_chain_asserted_head_001",
        "pass_type_elaboration_local_object_mode_long_chain_asserted_head_001",
        "pass_type_elaboration_local_mode_long_chain_radix_asserted_head_001",
    ];
    assert_eq!(preexisting_owner_ids.len(), 35);
    for owner_id in preexisting_owner_ids {
        let (owner_ordinal, owner_case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| case.id.0 == owner_id)
            .unwrap_or_else(|| panic!("pre-existing owner fixture {owner_id} must be active"));
        let owner_frontend = run_frontend(&workspace_root, owner_case, owner_ordinal)
            .unwrap_or_else(|error| panic!("owner fixture {owner_id} must parse: {error}"));
        assert!(
            owner_frontend.diagnostics.is_empty(),
            "owner fixture {owner_id} must remain frontend-clean"
        );
        let owner_ast = owner_frontend
            .ast
            .unwrap_or_else(|| panic!("owner fixture {owner_id} must produce an AST"));
        let owner_resolver =
            resolver_symbol_collection(&workspace_root, owner_case, &owner_ast);
        assert!(
            owner_resolver.detail_keys.is_empty(),
            "owner fixture {owner_id} must remain resolver-clean"
        );
        let owner_symbols = augment_type_elaboration_import_summaries(
            &owner_ast,
            &owner_resolver.module,
            owner_resolver.env,
        );
        let owner_self_extracts = match owner_id {
            "pass_type_elaboration_reserved_variable_type_assertion_001" => {
                extract_source_reserved_variable_type_assertion(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_reserved_object_variable_type_assertion_001" => {
                super::extract_source_reserved_object_variable_type_assertion(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_local_mode_reserved_variable_type_assertion_001" => {
                extract_source_local_mode_reserved_variable_type_assertion(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_local_object_mode_reserved_variable_type_assertion_001" => {
                extract_source_local_object_mode_reserved_variable_type_assertion(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_local_mode_asserted_head_001" => {
                extract_source_local_mode_asserted_head(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_local_object_mode_asserted_head_001" => {
                extract_source_local_object_mode_asserted_head(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_chained_local_mode_reserved_variable_type_assertion_001" => {
                extract_source_chained_local_mode_reserved_variable_type_assertion(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_chained_local_object_mode_reserved_variable_type_assertion_001" => {
                extract_source_chained_local_object_mode_reserved_variable_type_assertion(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_chained_local_mode_asserted_head_001" => {
                extract_source_chained_local_mode_asserted_head(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_chained_local_object_mode_asserted_head_001" => {
                extract_source_chained_local_object_mode_asserted_head(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_chained_local_mode_radix_asserted_head_001" => {
                extract_source_chained_local_mode_radix_asserted_head(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_chained_local_object_mode_radix_asserted_head_001" => {
                extract_source_chained_local_object_mode_radix_asserted_head(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_two_edge_local_mode_reserved_variable_type_assertion_001" => {
                extract_source_two_edge_local_mode_reserved_variable_type_assertion(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_two_edge_local_object_mode_reserved_variable_type_assertion_001" => {
                extract_source_two_edge_local_object_mode_reserved_variable_type_assertion(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_two_edge_local_mode_asserted_head_001" => {
                extract_source_two_edge_local_mode_asserted_head(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_two_edge_local_object_mode_asserted_head_001" => {
                extract_source_two_edge_local_object_mode_asserted_head(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_two_edge_local_mode_radix_asserted_head_001" => {
                extract_source_two_edge_local_mode_radix_asserted_head(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_two_edge_local_object_mode_radix_asserted_head_001" => {
                extract_source_two_edge_local_object_mode_radix_asserted_head(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_three_edge_local_mode_reserved_variable_type_assertion_001" => {
                extract_source_three_edge_local_mode_reserved_variable_type_assertion(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_three_edge_local_object_mode_reserved_variable_type_assertion_001" => {
                extract_source_three_edge_local_object_mode_reserved_variable_type_assertion(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_three_edge_local_mode_asserted_head_001" => {
                extract_source_three_edge_local_mode_asserted_head(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_three_edge_local_object_mode_asserted_head_001" => {
                extract_source_three_edge_local_object_mode_asserted_head(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_three_edge_local_mode_radix_asserted_head_001" => {
                extract_source_three_edge_local_mode_radix_asserted_head(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_three_edge_local_object_mode_radix_asserted_head_001" => {
                extract_source_three_edge_local_object_mode_radix_asserted_head(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_four_edge_local_mode_reserved_variable_type_assertion_001" => {
                extract_source_four_edge_local_mode_reserved_variable_type_assertion(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_four_edge_local_object_mode_reserved_variable_type_assertion_001" => {
                extract_source_four_edge_local_object_mode_reserved_variable_type_assertion(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_four_edge_local_mode_asserted_head_001" => {
                super::extract_source_four_edge_local_mode_asserted_head(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_four_edge_local_object_mode_asserted_head_001" => {
                super::extract_source_four_edge_local_object_mode_asserted_head(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_four_edge_local_mode_radix_asserted_head_001" => {
                extract_source_four_edge_local_mode_radix_asserted_head(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_four_edge_local_object_mode_radix_asserted_head_001" => {
                extract_source_four_edge_local_object_mode_radix_asserted_head(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_local_mode_long_chain_reserved_variable_type_assertion_001" => {
                extract_source_local_mode_long_chain_reserved_variable_type_assertion(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_local_object_mode_long_chain_reserved_variable_type_assertion_001" => {
                extract_source_local_object_mode_long_chain_reserved_variable_type_assertion(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_local_mode_long_chain_asserted_head_001" => {
                super::extract_source_local_mode_long_chain_asserted_head(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_local_object_mode_long_chain_asserted_head_001" => {
                super::extract_source_local_object_mode_long_chain_asserted_head(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            "pass_type_elaboration_local_mode_long_chain_radix_asserted_head_001" => {
                extract_source_local_mode_long_chain_radix_asserted_head(
                    &owner_ast,
                    owner_resolver.module.clone(),
                    &owner_symbols,
                )
                .is_some()
            }
            _ => unreachable!("undeclared pre-existing owner fixture {owner_id}"),
        };
        assert!(
            owner_self_extracts,
            "pre-existing owner fixture {owner_id} must non-vacuously reach its own extractor"
        );
        assert!(
            super::extract_source_local_object_mode_long_chain_radix_asserted_head(
                &owner_ast,
                owner_resolver.module,
                &owner_symbols,
            )
            .is_none(),
            "Task 210 must reject pre-existing owner fixture {owner_id}"
        );
    }
}

#[test]
fn active_local_mode_long_chain_asserted_head_fixture_consumes_seven_expansions() {
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
    let plan = build_test_plan(&config).expect("Task 199 repository plan should build");
    let (ordinal, case) = active_type_elaboration_cases(&plan)
        .enumerate()
        .find(|(_, case)| {
            case.id.0 == "pass_type_elaboration_local_mode_long_chain_asserted_head_001"
        })
        .expect("Task 199 active fixture should be discoverable");
    let frontend = run_frontend(&workspace_root, case, ordinal)
        .expect("Task 199 fixture should run through the real frontend");
    assert!(frontend.diagnostics.is_empty());
    let ast = frontend
        .ast
        .expect("Task 199 fixture should produce an AST");
    let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
    assert!(resolver.detail_keys.is_empty());
    let symbols =
        augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
    let output = super::source_local_mode_long_chain_asserted_head_output(
        &ast,
        resolver.module,
        &symbols,
    )
    .expect("Task 199 real AST should reach the seven-expansion asserted-head seam");
    assert_source_reserved_variable_type_assertion_output(&output)
        .expect("Task 199 real AST should preserve every checked payload invariant");
    assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
    assert_eq!(output.subject_binding, BindingId::new(0));
    assert_eq!(output.payload.subject_lookup_ordinal, 1);
    assert_eq!(output.term_formula.type_entries().len(), 3);
    assert_eq!(output.term_formula.normalized_types().len(), 1);
    assert_eq!(output.subject_result_input.spelling, "ChainMode6");
    assert_eq!(output.asserted_type_input.spelling, "ChainMode6");
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
        .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("BaseMode"))
        .map(|(_, expansion)| &expansion.radix)
        .expect("Task 199 BaseMode terminal expansion should exist");
    let (_, normalized) = output
        .term_formula
        .normalized_types()
        .iter()
        .next()
        .expect("Task 199 normalized set type should exist");
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
}

#[test]
fn active_local_object_mode_long_chain_asserted_head_fixture_consumes_seven_expansions() {
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
    let plan = build_test_plan(&config).expect("Task 200 repository plan should build");
    let (ordinal, case) = active_type_elaboration_cases(&plan)
        .enumerate()
        .find(|(_, case)| {
            case.id.0 == "pass_type_elaboration_local_object_mode_long_chain_asserted_head_001"
        })
        .expect("Task 200 active fixture should be discoverable");
    let frontend = run_frontend(&workspace_root, case, ordinal)
        .expect("Task 200 fixture should run through the real frontend");
    assert!(frontend.diagnostics.is_empty());
    let ast = frontend
        .ast
        .expect("Task 200 fixture should produce an AST");
    let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
    assert!(resolver.detail_keys.is_empty());
    let symbols =
        augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
    let output = super::source_local_object_mode_long_chain_asserted_head_output(
        &ast,
        resolver.module,
        &symbols,
    )
    .expect("Task 200 real AST should reach the seven-expansion object asserted-head seam");
    assert_source_reserved_variable_type_assertion_output(&output)
        .expect("Task 200 real AST should preserve every checked payload invariant");
    assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
    assert_eq!(output.subject_binding, BindingId::new(0));
    assert_eq!(output.payload.subject_lookup_ordinal, 1);
    assert_eq!(output.term_formula.type_entries().len(), 3);
    assert_eq!(output.term_formula.normalized_types().len(), 1);
    assert_eq!(output.subject_result_input.spelling, "ChainObjectMode6");
    assert_eq!(output.asserted_type_input.spelling, "ChainObjectMode6");
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
        .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("BaseObjectMode"))
        .map(|(_, expansion)| &expansion.radix)
        .expect("Task 200 BaseObjectMode terminal expansion should exist");
    let (_, normalized) = output
        .term_formula
        .normalized_types()
        .iter()
        .next()
        .expect("Task 200 normalized object type should exist");
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
}

#[test]
fn active_local_object_mode_long_chain_reserved_variable_type_assertion_fixture_consumes_seven_expansions()
 {
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
                == "pass_type_elaboration_local_object_mode_long_chain_reserved_variable_type_assertion_001"
        })
        .expect("Task 179 active fixture should be discoverable");
    let frontend = run_frontend(&workspace_root, case, ordinal)
        .expect("Task 179 fixture should run through the real frontend");
    assert!(frontend.diagnostics.is_empty());
    let ast = frontend
        .ast
        .expect("Task 179 fixture should produce an AST");
    let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
    assert!(resolver.detail_keys.is_empty());
    let symbols =
        augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
    let output = source_local_object_mode_long_chain_reserved_variable_type_assertion_output(
        &ast,
        resolver.module,
        &symbols,
    )
    .expect("Task 179 real AST should reach long-chain object type assertion");
    assert_source_reserved_variable_type_assertion_output(&output)
        .expect("Task 179 real AST should preserve every checked payload invariant");
    assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
    assert_eq!(output.subject_binding, BindingId::new(0));
    assert_eq!(output.payload.subject_lookup_ordinal, 1);
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
}
#[test]
fn active_local_object_mode_long_chain_reserved_variable_equality_fixture_consumes_seven_expansions()
 {
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
                == "pass_type_elaboration_local_object_mode_long_chain_reserved_variable_equality_001"
        })
        .expect("Task 176 active fixture should be discoverable");
    let frontend = run_frontend(&workspace_root, case, ordinal)
        .expect("Task 176 fixture should run through the real frontend");
    assert!(frontend.diagnostics.is_empty());
    let ast = frontend
        .ast
        .expect("Task 176 fixture should produce an AST");
    let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
    assert!(resolver.detail_keys.is_empty());
    let symbols =
        augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
    let output = source_local_object_mode_long_chain_reserved_variable_equality_output(
        &ast,
        resolver.module,
        &symbols,
    )
    .expect("Task 176 real AST should reach long-chain object equality");
    assert_source_reserved_variable_formula_output(&output)
        .expect("Task 176 real AST should preserve every checked payload invariant");
    assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
    assert_eq!(output.left_binding, BindingId::new(0));
    assert_eq!(output.right_binding, BindingId::new(0));
    assert_eq!(output.payload.left_lookup_ordinal, 1);
    assert_eq!(output.payload.right_lookup_ordinal, 2);
    for input in [
        &output.left_result_input,
        &output.right_result_input,
        output.left_expected_input.as_ref().unwrap(),
        output.right_expected_input.as_ref().unwrap(),
    ] {
        assert_eq!(input.spelling, "ChainObjectMode6");
        assert!(matches!(input.head, TypeHeadInput::Symbol(_)));
    }
    let (_, formula) = output
        .term_formula
        .formulas()
        .iter()
        .next()
        .expect("Task 176 equality should be checked");
    assert_eq!(formula.kind, FormulaKind::Equality);
    assert_eq!(formula.status, FormulaStatus::Checked);
    assert_eq!(formula.expected_types.len(), 2);
    assert!(formula.facts.is_empty());
    assert!(formula.deferred.is_empty());
    let terminal = output
        .payload
        .reserve
        .mode_expansions
        .iter()
        .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("BaseObjectMode"))
        .map(|(_, expansion)| &expansion.radix)
        .expect("Task 176 BaseObjectMode terminal expansion should exist");
    let (_, normalized) = output
        .term_formula
        .normalized_types()
        .iter()
        .next()
        .expect("Task 176 normalized object type should exist");
    assert_eq!(output.term_formula.normalized_types().len(), 1);
    assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
    assert_eq!(normalized.source.range, terminal.source_range);
    assert_eq!(normalized.source.spelling, terminal.spelling);
}
#[test]
fn active_local_object_mode_long_chain_reserved_variable_inequality_fixture_consumes_seven_expansions()
 {
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
                == "pass_type_elaboration_local_object_mode_long_chain_reserved_variable_inequality_001"
        })
        .expect("Task 177 active fixture should be discoverable");
    let frontend = run_frontend(&workspace_root, case, ordinal)
        .expect("Task 177 fixture should run through the real frontend");
    assert!(frontend.diagnostics.is_empty());
    let ast = frontend
        .ast
        .expect("Task 177 fixture should produce an AST");
    let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
    assert!(resolver.detail_keys.is_empty());
    let symbols =
        augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
    let output = source_local_object_mode_long_chain_reserved_variable_inequality_output(
        &ast,
        resolver.module,
        &symbols,
    )
    .expect("Task 177 real AST should reach long-chain object inequality");
    assert_source_reserved_variable_formula_output(&output)
        .expect("Task 177 real AST should preserve every checked payload invariant");
    assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
    assert_eq!(output.left_binding, BindingId::new(0));
    assert_eq!(output.right_binding, BindingId::new(0));
    assert_eq!(output.payload.left_lookup_ordinal, 1);
    assert_eq!(output.payload.right_lookup_ordinal, 2);
    for input in [
        &output.left_result_input,
        &output.right_result_input,
        output.left_expected_input.as_ref().unwrap(),
        output.right_expected_input.as_ref().unwrap(),
    ] {
        assert_eq!(input.spelling, "ChainObjectMode6");
        assert!(matches!(input.head, TypeHeadInput::Symbol(_)));
    }
    let (_, formula) = output
        .term_formula
        .formulas()
        .iter()
        .next()
        .expect("Task 177 inequality should be checked");
    assert_eq!(formula.kind, FormulaKind::Inequality);
    assert_eq!(formula.status, FormulaStatus::Checked);
    assert_eq!(formula.expected_types.len(), 2);
    assert!(formula.facts.is_empty());
    assert!(formula.deferred.is_empty());
    let terminal = output
        .payload
        .reserve
        .mode_expansions
        .iter()
        .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("BaseObjectMode"))
        .map(|(_, expansion)| &expansion.radix)
        .expect("Task 177 BaseObjectMode terminal expansion should exist");
    let (_, normalized) = output
        .term_formula
        .normalized_types()
        .iter()
        .next()
        .expect("Task 177 normalized object type should exist");
    assert_eq!(output.term_formula.normalized_types().len(), 1);
    assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
    assert_eq!(normalized.source.range, terminal.source_range);
    assert_eq!(normalized.source.spelling, terminal.spelling);
}
#[test]
fn active_local_object_mode_long_chain_reserved_variable_membership_fixture_consumes_seven_expansions()
 {
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
                == "pass_type_elaboration_local_object_mode_long_chain_reserved_variable_membership_001"
        })
        .expect("Task 178 active fixture should be discoverable");
    let frontend = run_frontend(&workspace_root, case, ordinal)
        .expect("Task 178 fixture should run through the real frontend");
    assert!(frontend.diagnostics.is_empty());
    let ast = frontend
        .ast
        .expect("Task 178 fixture should produce an AST");
    let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
    assert!(resolver.detail_keys.is_empty());
    let symbols =
        augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
    let output = source_local_object_mode_long_chain_reserved_variable_membership_output(
        &ast,
        resolver.module,
        &symbols,
    )
    .expect("Task 178 real AST should reach long-chain object membership");
    assert_source_reserved_variable_formula_output(&output)
        .expect("Task 178 real AST should preserve every checked payload invariant");
    assert_eq!(output.payload.reserve.mode_expansions.len(), 7);
    assert_eq!(output.left_binding, BindingId::new(0));
    assert_eq!(output.right_binding, BindingId::new(1));
    assert_eq!(output.payload.left_lookup_ordinal, 2);
    assert_eq!(output.payload.right_lookup_ordinal, 3);
    assert_eq!(output.left_result_input.spelling, "ChainObjectMode6");
    assert!(output.left_expected_input.is_none());
    assert_eq!(output.right_result_input.head, TypeHeadInput::BuiltinSet);
    assert_eq!(
        output.right_expected_input.as_ref().unwrap().head,
        TypeHeadInput::BuiltinSet
    );
    let (_, formula) = output
        .term_formula
        .formulas()
        .iter()
        .next()
        .expect("Task 178 membership formula should be checked");
    assert_eq!(formula.kind, FormulaKind::Membership);
    assert_eq!(formula.status, FormulaStatus::Checked);
    assert_eq!(formula.expected_types.len(), 1);
    assert!(formula.facts.is_empty());
    assert!(formula.deferred.is_empty());
    assert_eq!(output.term_formula.normalized_types().len(), 2);
    assert!(
        output
            .term_formula
            .normalized_types()
            .iter()
            .any(|(_, ty)| ty.head == TypeHeadRef::BuiltinObject)
    );
    assert!(
        output
            .term_formula
            .normalized_types()
            .iter()
            .any(|(_, ty)| ty.head == TypeHeadRef::BuiltinSet)
    );
}
