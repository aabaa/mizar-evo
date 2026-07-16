    include!("tests/support.rs");

    include!("tests/parse_only.rs");

    include!("tests/type_elaboration/source_extraction.rs");

    include!("tests/type_elaboration/reserved_binary.rs");

    include!("tests/type_elaboration/mode_chain.rs");

    include!("tests/type_elaboration/reserved_direct.rs");

    include!("tests/type_elaboration/asserted_head_base.rs");

    include!("tests/type_elaboration/asserted_head_four_edge_radix.rs");

    include!("tests/type_elaboration/asserted_head_three_edge_object_radix.rs");

    include!("tests/type_elaboration/asserted_head_two_edge_object_radix.rs");

    include!("tests/type_elaboration/asserted_head_type_assertion.rs");

    include!("tests/type_elaboration/binary_route_fixtures.rs");

    #[test]
    fn active_reserved_object_variable_equality_fixture_preserves_real_checker_payload() {
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
        let plan = build_test_plan(&config).expect("Task 188 repository plan should build");
        let (ordinal, case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| {
                case.id.0 == "pass_type_elaboration_reserved_object_variable_equality_001"
            })
            .expect("Task 188 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 188 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 188 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output =
            super::source_reserved_object_variable_equality_output(&ast, resolver.module, &symbols)
                .expect("Task 188 real AST should reach the builtin-object equality seam");
        assert_source_reserved_variable_formula_output(&output)
            .expect("Task 188 real AST should preserve every checked payload invariant");
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 188 normalized object identity should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
    }

    #[test]
    fn active_reserved_object_variable_inequality_fixture_preserves_real_checker_payload() {
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
        let plan = build_test_plan(&config).expect("Task 190 repository plan should build");
        let (ordinal, case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| {
                case.id.0 == "pass_type_elaboration_reserved_object_variable_inequality_001"
            })
            .expect("Task 190 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 190 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 190 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = super::source_reserved_object_variable_inequality_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 190 real AST should reach the builtin-object inequality seam");
        assert_source_reserved_variable_formula_output(&output)
            .expect("Task 190 real AST should preserve every checked payload invariant");
        assert_eq!(output.term_formula.type_entries().len(), 6);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 190 normalized object identity should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        let (_, formula) = output
            .term_formula
            .formulas()
            .iter()
            .next()
            .expect("Task 190 checked inequality should exist");
        assert_eq!(formula.kind, FormulaKind::Inequality);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert_eq!(formula.expected_types.len(), 2);
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
    }

    #[test]
    fn active_reserved_object_variable_type_assertion_fixture_preserves_real_checker_payload() {
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
        let plan = build_test_plan(&config).expect("Task 189 repository plan should build");
        let (ordinal, case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| {
                case.id.0 == "pass_type_elaboration_reserved_object_variable_type_assertion_001"
            })
            .expect("Task 189 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 189 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 189 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = super::source_reserved_object_variable_type_assertion_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 189 real AST should reach the builtin-object type-assertion seam");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 189 real AST should preserve every checked payload invariant");
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 189 normalized object identity should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
    }

    #[test]
    fn active_contradiction_formula_constant_fixture_preserves_real_checker_payload() {
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
                case.id.0 == "pass_type_elaboration_contradiction_formula_constant_001"
            })
            .expect("Task 180 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 180 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 180 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_contradiction_formula_output(&ast, resolver.module, &symbols)
            .expect("Task 180 real AST should reach the standalone contradiction checker seam");
        assert!(output.terms().is_empty());
        assert_eq!(output.formulas().len(), 1);
        assert!(output.candidate_sets().is_empty());
        assert!(output.facts().is_empty());
        assert!(output.diagnostics().is_empty());
        let (_, formula) = output
            .formulas()
            .iter()
            .next()
            .expect("Task 180 contradiction formula should be checked");
        assert_eq!(formula.kind, FormulaKind::Contradiction);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert_eq!(formula.context, BindingContextId::new(0));
        assert!(formula.terms.is_empty());
        assert!(formula.asserted_type.is_none());
        assert!(formula.expected_types.is_empty());
        assert!(formula.candidate_set.is_none());
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
    }

    #[test]
    fn active_distinct_reserved_variable_equality_fixture_preserves_real_checker_payload() {
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
                case.id.0 == "pass_type_elaboration_distinct_reserved_variable_equality_001"
            })
            .expect("Task 123 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 123 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 123 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output =
            source_distinct_reserved_variable_equality_output(&ast, resolver.module, &symbols)
                .expect("Task 123 real AST should reach the distinct-binding equality seam");
        assert_source_reserved_variable_formula_output(&output)
            .expect("Task 123 real AST should preserve every checked payload invariant");
        assert_ne!(output.left_binding, output.right_binding);
        assert_eq!(
            output.payload.reserve.bridge.bindings()[0].type_range,
            output.payload.reserve.bridge.bindings()[1].type_range
        );
    }

    #[test]
    fn active_distinct_reserved_object_variable_equality_fixture_preserves_real_checker_payload() {
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
        let plan = build_test_plan(&config).expect("Task 191 repository plan should build");
        let (ordinal, case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| {
                case.id.0 == "pass_type_elaboration_distinct_reserved_object_variable_equality_001"
            })
            .expect("Task 191 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 191 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 191 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = super::source_distinct_reserved_object_variable_equality_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 191 real AST should reach the distinct builtin-object equality seam");
        assert_source_reserved_variable_formula_output(&output)
            .expect("Task 191 real AST should preserve every checked payload invariant");
        assert_eq!(output.left_binding, BindingId::new(0));
        assert_eq!(output.right_binding, BindingId::new(1));
        assert_eq!(output.payload.left_lookup_ordinal, 2);
        assert_eq!(output.payload.right_lookup_ordinal, 3);
        assert_eq!(
            output.payload.reserve.bridge.bindings()[0].type_range,
            output.payload.reserve.bridge.bindings()[1].type_range
        );
        assert_eq!(output.term_formula.type_entries().len(), 6);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 191 normalized object identity should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        let (_, formula) = output
            .term_formula
            .formulas()
            .iter()
            .next()
            .expect("Task 191 checked equality should exist");
        assert_eq!(formula.kind, FormulaKind::Equality);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert_eq!(formula.expected_types.len(), 2);
        assert!(formula.candidate_set.is_none());
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
        assert!(output.term_formula.candidate_sets().is_empty());
        assert!(output.term_formula.facts().is_empty());
        assert!(output.term_formula.diagnostics().is_empty());
    }

    #[test]
    fn active_distinct_reserved_object_variable_inequality_fixture_preserves_real_checker_payload()
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
        let plan = build_test_plan(&config).expect("Task 192 repository plan should build");
        let (ordinal, case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| {
                case.id.0
                    == "pass_type_elaboration_distinct_reserved_object_variable_inequality_001"
            })
            .expect("Task 192 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 192 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 192 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = super::source_distinct_reserved_object_variable_inequality_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 192 real AST should reach the distinct builtin-object inequality seam");
        assert_source_reserved_variable_formula_output(&output)
            .expect("Task 192 real AST should preserve every checked payload invariant");
        assert_eq!(output.left_binding, BindingId::new(0));
        assert_eq!(output.right_binding, BindingId::new(1));
        assert_eq!(output.payload.left_lookup_ordinal, 2);
        assert_eq!(output.payload.right_lookup_ordinal, 3);
        assert_eq!(
            output.payload.reserve.bridge.bindings()[0].type_range,
            output.payload.reserve.bridge.bindings()[1].type_range
        );
        assert_eq!(output.term_formula.type_entries().len(), 6);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 192 normalized object identity should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        let (_, formula) = output
            .term_formula
            .formulas()
            .iter()
            .next()
            .expect("Task 192 checked inequality should exist");
        assert_eq!(formula.kind, FormulaKind::Inequality);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert_eq!(formula.expected_types.len(), 2);
        assert_eq!(formula.expected_types[0].term, output.payload.left_site);
        assert_eq!(formula.expected_types[1].term, output.payload.right_site);
        assert!(formula.candidate_set.is_none());
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
        assert!(output.term_formula.candidate_sets().is_empty());
        assert!(output.term_formula.facts().is_empty());
        assert!(output.term_formula.diagnostics().is_empty());
    }

    #[test]
    fn active_distinct_reserved_variable_inequality_fixture_preserves_real_checker_payload() {
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
                case.id.0 == "pass_type_elaboration_distinct_reserved_variable_inequality_001"
            })
            .expect("Task 160 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 160 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 160 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output =
            source_distinct_reserved_variable_inequality_output(&ast, resolver.module, &symbols)
                .expect("Task 160 real AST should reach the distinct-binding inequality seam");
        assert_source_reserved_variable_formula_output(&output)
            .expect("Task 160 real AST should preserve every checked payload invariant");
        assert_eq!(output.left_binding, BindingId::new(0));
        assert_eq!(output.right_binding, BindingId::new(1));
        assert_eq!(output.payload.left_lookup_ordinal, 2);
        assert_eq!(output.payload.right_lookup_ordinal, 3);
        assert_eq!(
            output.payload.reserve.bridge.bindings()[0].type_range,
            output.payload.reserve.bridge.bindings()[1].type_range
        );
        assert_eq!(output.term_formula.type_entries().len(), 6);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        let formula = output
            .term_formula
            .formulas()
            .iter()
            .map(|(_, formula)| formula)
            .next()
            .expect("Task 160 inequality formula should be checked");
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
    }

    #[test]
    fn active_distinct_reserved_variable_membership_fixture_preserves_real_checker_payload() {
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
                case.id.0 == "pass_type_elaboration_distinct_reserved_variable_membership_001"
            })
            .expect("Task 159 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 159 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 159 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output =
            source_distinct_reserved_variable_membership_output(&ast, resolver.module, &symbols)
                .expect("Task 159 real AST should reach the distinct-binding membership seam");
        assert_source_reserved_variable_formula_output(&output)
            .expect("Task 159 real AST should preserve every checked payload invariant");
        assert_eq!(output.left_binding, BindingId::new(0));
        assert_eq!(output.right_binding, BindingId::new(1));
        assert_eq!(
            output.payload.reserve.bridge.bindings()[0].type_range,
            output.payload.reserve.bridge.bindings()[1].type_range
        );
        assert!(output.left_expected_input.is_none());
        assert_eq!(output.term_formula.normalized_types().len(), 1);
    }

    #[test]
    fn active_multiple_object_reserve_declaration_inequality_fixture_preserves_real_checker_payload()
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
        let plan = build_test_plan(&config).expect("Task 194 repository plan should build");
        let (ordinal, case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| {
                case.id.0
                    == "pass_type_elaboration_multiple_object_reserve_declaration_inequality_001"
            })
            .expect("Task 194 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 194 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 194 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = super::source_multiple_object_reserve_declaration_inequality_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 194 real AST should reach the multiple-object-reserve inequality seam");
        assert_source_reserved_variable_formula_output(&output)
            .expect("Task 194 real AST should preserve every checked payload invariant");
        assert_eq!(output.left_binding, BindingId::new(0));
        assert_eq!(output.right_binding, BindingId::new(1));
        assert_eq!(output.payload.left_lookup_ordinal, 2);
        assert_eq!(output.payload.right_lookup_ordinal, 3);
        let left_range = output.payload.reserve.bridge.bindings()[0].type_range;
        let right_range = output.payload.reserve.bridge.bindings()[1].type_range;
        assert!(
            (left_range.start, left_range.end) < (right_range.start, right_range.end),
            "the two declaration-owned object ranges should preserve source order"
        );
        assert_eq!(output.left_result_input.source_range, left_range);
        assert_eq!(
            output.left_expected_input.as_ref().unwrap().source_range,
            left_range
        );
        assert_eq!(output.right_result_input.source_range, right_range);
        assert_eq!(
            output.right_expected_input.as_ref().unwrap().source_range,
            right_range
        );
        for input in [
            &output.left_result_input,
            output.left_expected_input.as_ref().unwrap(),
            &output.right_result_input,
            output.right_expected_input.as_ref().unwrap(),
        ] {
            assert_eq!(input.head, TypeHeadInput::BuiltinObject);
            assert!(input.args.is_empty());
            assert!(input.attributes.is_empty());
        }
        assert_eq!(output.term_formula.type_entries().len(), 6);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 194 normalized object identity should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_eq!(normalized.source.range, left_range);
        let (_, formula) = output
            .term_formula
            .formulas()
            .iter()
            .next()
            .expect("Task 194 checked inequality should exist");
        assert_eq!(formula.kind, FormulaKind::Inequality);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert_eq!(formula.expected_types.len(), 2);
        assert!(formula.candidate_set.is_none());
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
        assert!(output.term_formula.candidate_sets().is_empty());
        assert!(output.term_formula.facts().is_empty());
        assert!(output.term_formula.diagnostics().is_empty());
    }

    #[test]
    fn active_multiple_object_reserve_declaration_equality_fixture_preserves_real_checker_payload()
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
        let plan = build_test_plan(&config).expect("Task 193 repository plan should build");
        let (ordinal, case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| {
                case.id.0
                    == "pass_type_elaboration_multiple_object_reserve_declaration_equality_001"
            })
            .expect("Task 193 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 193 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 193 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = super::source_multiple_object_reserve_declaration_equality_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 193 real AST should reach the multiple-object-reserve equality seam");
        assert_source_reserved_variable_formula_output(&output)
            .expect("Task 193 real AST should preserve every checked payload invariant");
        assert_eq!(output.left_binding, BindingId::new(0));
        assert_eq!(output.right_binding, BindingId::new(1));
        assert_eq!(output.payload.left_lookup_ordinal, 2);
        assert_eq!(output.payload.right_lookup_ordinal, 3);
        let left_range = output.payload.reserve.bridge.bindings()[0].type_range;
        let right_range = output.payload.reserve.bridge.bindings()[1].type_range;
        assert_ne!(left_range, right_range);
        assert_eq!(output.left_result_input.source_range, left_range);
        assert_eq!(
            output.left_expected_input.as_ref().unwrap().source_range,
            left_range
        );
        assert_eq!(output.right_result_input.source_range, right_range);
        assert_eq!(
            output.right_expected_input.as_ref().unwrap().source_range,
            right_range
        );
        assert_eq!(output.term_formula.type_entries().len(), 6);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 193 normalized object identity should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_eq!(normalized.source.range, left_range);
        let (_, formula) = output
            .term_formula
            .formulas()
            .iter()
            .next()
            .expect("Task 193 checked equality should exist");
        assert_eq!(formula.kind, FormulaKind::Equality);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert_eq!(formula.expected_types.len(), 2);
        assert!(formula.candidate_set.is_none());
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
        assert!(output.term_formula.candidate_sets().is_empty());
        assert!(output.term_formula.facts().is_empty());
        assert!(output.term_formula.diagnostics().is_empty());
    }

    #[test]
    fn active_multiple_reserve_declaration_equality_fixture_preserves_real_checker_payload() {
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
                case.id.0 == "pass_type_elaboration_multiple_reserve_declaration_equality_001"
            })
            .expect("Task 124 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 124 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 124 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output =
            source_multiple_reserve_declaration_equality_output(&ast, resolver.module, &symbols)
                .expect("Task 124 real AST should reach the multiple-reserve equality seam");
        assert_source_reserved_variable_formula_output(&output)
            .expect("Task 124 real AST should preserve every checked payload invariant");
        assert_ne!(output.left_binding, output.right_binding);
        assert_ne!(
            output.payload.reserve.bridge.bindings()[0].type_range,
            output.payload.reserve.bridge.bindings()[1].type_range
        );
        assert_eq!(output.term_formula.normalized_types().len(), 1);
    }

    #[test]
    fn active_multiple_reserve_declaration_inequality_fixture_preserves_real_checker_payload() {
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
                case.id.0 == "pass_type_elaboration_multiple_reserve_declaration_inequality_001"
            })
            .expect("Task 161 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 161 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 161 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output =
            source_multiple_reserve_declaration_inequality_output(&ast, resolver.module, &symbols)
                .expect("Task 161 real AST should reach the multiple-reserve inequality seam");
        assert_source_reserved_variable_formula_output(&output)
            .expect("Task 161 real AST should preserve every checked payload invariant");
        assert_eq!(output.left_binding, BindingId::new(0));
        assert_eq!(output.right_binding, BindingId::new(1));
        assert_eq!(output.payload.left_lookup_ordinal, 2);
        assert_eq!(output.payload.right_lookup_ordinal, 3);
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
        assert_eq!(output.term_formula.type_entries().len(), 6);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 161 should have one canonical set identity");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_eq!(normalized.source.range, source_bindings[0].type_range);
        let formula = output
            .term_formula
            .formulas()
            .iter()
            .map(|(_, formula)| formula)
            .next()
            .expect("Task 161 inequality formula should be checked");
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
    }

    #[test]
    fn active_multiple_reserve_declaration_membership_fixture_preserves_real_checker_payload() {
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
                case.id.0 == "pass_type_elaboration_multiple_reserve_declaration_membership_001"
            })
            .expect("Task 162 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 162 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 162 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output =
            source_multiple_reserve_declaration_membership_output(&ast, resolver.module, &symbols)
                .expect("Task 162 real AST should reach the multiple-reserve membership seam");
        assert_source_reserved_variable_formula_output(&output)
            .expect("Task 162 real AST should preserve every checked payload invariant");
        assert_eq!(output.left_binding, BindingId::new(0));
        assert_eq!(output.right_binding, BindingId::new(1));
        assert_eq!(output.payload.left_lookup_ordinal, 2);
        assert_eq!(output.payload.right_lookup_ordinal, 3);
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
                .expect("Task 162 right expected input should exist")
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
            .expect("Task 162 should have one canonical set identity");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_eq!(normalized.source.range, source_bindings[0].type_range);
        let formula = output
            .term_formula
            .formulas()
            .iter()
            .map(|(_, formula)| formula)
            .next()
            .expect("Task 162 membership formula should be checked");
        assert_eq!(formula.kind, FormulaKind::Membership);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert_eq!(formula.expected_types.len(), 1);
        assert_eq!(formula.expected_types[0].term, output.payload.right_site);
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
        assert!(output.term_formula.candidate_sets().is_empty());
        assert!(output.term_formula.facts().is_empty());
        assert!(output.term_formula.diagnostics().is_empty());
    }

    #[test]
    fn active_heterogeneous_reserve_membership_fixture_preserves_real_checker_payload() {
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
                case.id.0 == "pass_type_elaboration_heterogeneous_reserve_membership_001"
            })
            .expect("Task 125 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 125 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 125 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output =
            source_heterogeneous_reserve_membership_output(&ast, resolver.module, &symbols)
                .expect("Task 125 real AST should reach the heterogeneous membership seam");
        assert_source_reserved_variable_formula_output(&output)
            .expect("Task 125 real AST should preserve every checked payload invariant");
        assert_ne!(output.left_binding, output.right_binding);
        assert_eq!(output.left_result_input.head, TypeHeadInput::BuiltinObject);
        assert_eq!(output.right_result_input.head, TypeHeadInput::BuiltinSet);
        assert!(output.left_expected_input.is_none());
        assert_eq!(output.term_formula.normalized_types().len(), 2);
    }

    #[test]
    fn active_local_mode_reserved_variable_membership_fixture_consumes_real_expansion() {
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
                case.id.0 == "pass_type_elaboration_local_mode_reserved_variable_membership_001"
            })
            .expect("Task 139 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 139 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 139 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output =
            source_local_mode_reserved_variable_membership_output(&ast, resolver.module, &symbols)
                .expect("Task 139 real AST should reach the local-mode membership seam");
        assert_source_reserved_variable_formula_output(&output)
            .expect("Task 139 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 1);
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
    }

    #[test]
    fn active_local_object_mode_reserved_variable_membership_fixture_consumes_real_expansion() {
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
                    == "pass_type_elaboration_local_object_mode_reserved_variable_membership_001"
            })
            .expect("Task 140 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 140 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 140 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_local_object_mode_reserved_variable_membership_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 140 real AST should reach the local object-mode membership seam");
        assert_source_reserved_variable_formula_output(&output)
            .expect("Task 140 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 1);
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
    }

    #[test]
    fn active_chained_local_mode_reserved_variable_membership_fixture_consumes_both_expansions() {
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
                    == "pass_type_elaboration_chained_local_mode_reserved_variable_membership_001"
            })
            .expect("Task 141 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 141 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 141 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_chained_local_mode_reserved_variable_membership_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 141 real AST should reach the chained local-mode membership seam");
        assert_source_reserved_variable_formula_output(&output)
            .expect("Task 141 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 2);
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
    }

    #[test]
    fn active_chained_local_object_mode_reserved_variable_membership_fixture_consumes_both_expansions()
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
                    == "pass_type_elaboration_chained_local_object_mode_reserved_variable_membership_001"
            })
            .expect("Task 142 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 142 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 142 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_chained_local_object_mode_reserved_variable_membership_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 142 real AST should reach the chained local object-mode membership seam");
        assert_source_reserved_variable_formula_output(&output)
            .expect("Task 142 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 2);
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
    }

    #[test]
    fn active_local_mode_reserved_variable_equality_fixture_consumes_real_expansion() {
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
                case.id.0 == "pass_type_elaboration_local_mode_reserved_variable_equality_001"
            })
            .expect("Task 126 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 126 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 126 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output =
            source_local_mode_reserved_variable_equality_output(&ast, resolver.module, &symbols)
                .expect("Task 126 real AST should reach the local-mode equality seam");
        assert!(
            output.term_formula.diagnostics().is_empty(),
            "{}",
            output.term_formula.debug_text()
        );
        assert_source_reserved_variable_formula_output(&output)
            .expect("Task 126 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 1);
        assert!(matches!(
            output.left_result_input.head,
            TypeHeadInput::Symbol(_)
        ));
        assert_eq!(output.term_formula.normalized_types().len(), 1);
    }

    #[test]
    fn active_chained_local_mode_reserved_variable_equality_fixture_consumes_both_expansions() {
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
                    == "pass_type_elaboration_chained_local_mode_reserved_variable_equality_001"
            })
            .expect("Task 127 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 127 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 127 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_chained_local_mode_reserved_variable_equality_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 127 real AST should reach the chained local-mode equality seam");
        assert_source_reserved_variable_formula_output(&output)
            .expect("Task 127 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 2);
        assert!(matches!(
            output.left_result_input.head,
            TypeHeadInput::Symbol(_)
        ));
        assert_eq!(output.term_formula.normalized_types().len(), 1);
    }

    #[test]
    fn active_two_edge_local_mode_reserved_variable_membership_fixture_consumes_three_expansions() {
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
                    == "pass_type_elaboration_two_edge_local_mode_reserved_variable_membership_001"
            })
            .expect("Task 143 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 143 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 143 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_two_edge_local_mode_reserved_variable_membership_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 143 real AST should reach the two-edge local-mode membership seam");
        assert_source_reserved_variable_formula_output(&output)
            .expect("Task 143 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 3);
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
    }

    #[test]
    fn active_two_edge_local_object_mode_reserved_variable_membership_fixture_consumes_three_expansions()
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
                    == "pass_type_elaboration_two_edge_local_object_mode_reserved_variable_membership_001"
            })
            .expect("Task 144 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 144 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 144 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_two_edge_local_object_mode_reserved_variable_membership_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 144 real AST should reach the two-edge local-object-mode membership seam");
        assert_source_reserved_variable_formula_output(&output)
            .expect("Task 144 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 3);
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
    }

    #[test]
    fn active_three_edge_local_mode_reserved_variable_equality_fixture_consumes_four_expansions() {
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
                    == "pass_type_elaboration_three_edge_local_mode_reserved_variable_equality_001"
            })
            .expect("Task 154 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 154 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 154 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_three_edge_local_mode_reserved_variable_equality_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 154 real AST should reach the three-edge local-mode equality seam");
        assert_source_reserved_variable_formula_output(&output)
            .expect("Task 154 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 4);
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
                source_mode_symbol_spelling(symbol) == Some("BaseThreeEdgeModeEquality")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("Task 154 base terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 154 normalized set type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
    }

    #[test]
    fn active_three_edge_local_mode_reserved_variable_inequality_fixture_consumes_four_expansions()
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
                    == "pass_type_elaboration_three_edge_local_mode_reserved_variable_inequality_001"
            })
            .expect("Task 156 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 156 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 156 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_three_edge_local_mode_reserved_variable_inequality_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 156 real AST should reach the three-edge local-mode inequality seam");
        assert_source_reserved_variable_formula_output(&output)
            .expect("Task 156 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 4);
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
                source_mode_symbol_spelling(symbol) == Some("BaseThreeEdgeModeInequality")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("Task 156 base terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 156 normalized set type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
    }

    #[test]
    fn active_three_edge_local_object_mode_reserved_variable_inequality_fixture_consumes_four_expansions()
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
                    == "pass_type_elaboration_three_edge_local_object_mode_reserved_variable_inequality_001"
            })
            .expect("Task 157 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 157 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 157 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_three_edge_local_object_mode_reserved_variable_inequality_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 157 real AST should reach the three-edge local-object-mode inequality seam");
        assert_source_reserved_variable_formula_output(&output)
            .expect("Task 157 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 4);
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
                source_mode_symbol_spelling(symbol) == Some("BaseThreeEdgeObjectModeInequality")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("Task 157 base object terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 157 normalized object type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
    }

    #[test]
    fn active_three_edge_local_mode_reserved_variable_membership_fixture_consumes_four_expansions()
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
                    == "pass_type_elaboration_three_edge_local_mode_reserved_variable_membership_001"
            })
            .expect("Task 158 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 158 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 158 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_three_edge_local_mode_reserved_variable_membership_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 158 real AST should reach the three-edge local-mode membership seam");
        assert_source_reserved_variable_formula_output(&output)
            .expect("Task 158 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 4);
        assert_eq!(output.left_binding, BindingId::new(0));
        assert_eq!(output.right_binding, BindingId::new(1));
        assert_eq!(output.payload.left_lookup_ordinal, 2);
        assert_eq!(output.payload.right_lookup_ordinal, 3);
        assert_ne!(
            output.payload.reserve.bridge.bindings()[0].type_range,
            output.payload.reserve.bridge.bindings()[1].type_range
        );
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
        assert_eq!(output.term_formula.terms().len(), 2);
        let formula = output
            .term_formula
            .formulas()
            .iter()
            .map(|(_, formula)| formula)
            .next()
            .expect("Task 158 membership formula should be checked");
        assert_eq!(formula.kind, FormulaKind::Membership);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert_eq!(formula.expected_types.len(), 1);
        assert_eq!(formula.expected_types[0].term, output.payload.right_site);
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
    }

    #[test]
    fn active_four_edge_local_mode_reserved_variable_membership_fixture_consumes_five_expansions() {
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
                    == "pass_type_elaboration_four_edge_local_mode_reserved_variable_membership_001"
            })
            .expect("Task 164 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 164 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 164 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_four_edge_local_mode_reserved_variable_membership_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 164 real AST should reach the four-edge local-mode membership seam");
        assert_source_reserved_variable_formula_output(&output)
            .expect("Task 164 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 5);
        assert_eq!(output.left_binding, BindingId::new(0));
        assert_eq!(output.right_binding, BindingId::new(1));
        assert_eq!(output.payload.left_lookup_ordinal, 2);
        assert_eq!(output.payload.right_lookup_ordinal, 3);
        assert_ne!(
            output.payload.reserve.bridge.bindings()[0].type_range,
            output.payload.reserve.bridge.bindings()[1].type_range
        );
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
        assert_eq!(output.term_formula.terms().len(), 2);
        let formula = output
            .term_formula
            .formulas()
            .iter()
            .map(|(_, formula)| formula)
            .next()
            .expect("Task 164 membership formula should be checked");
        assert_eq!(formula.kind, FormulaKind::Membership);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert_eq!(formula.expected_types.len(), 1);
        assert_eq!(formula.expected_types[0].term, output.payload.right_site);
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
    }
    #[test]
    fn active_four_edge_local_object_mode_reserved_variable_membership_fixture_consumes_five_expansions()
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
                    == "pass_type_elaboration_four_edge_local_object_mode_reserved_variable_membership_001"
            })
            .expect("Task 165 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 165 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 165 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_four_edge_local_object_mode_reserved_variable_membership_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 165 real AST should reach the four-edge local-object-mode membership seam");
        assert_source_reserved_variable_formula_output(&output)
            .expect("Task 165 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 5);
        assert_eq!(output.left_binding, BindingId::new(0));
        assert_eq!(output.right_binding, BindingId::new(1));
        assert_eq!(output.payload.left_lookup_ordinal, 2);
        assert_eq!(output.payload.right_lookup_ordinal, 3);
        assert_ne!(
            output.payload.reserve.bridge.bindings()[0].type_range,
            output.payload.reserve.bridge.bindings()[1].type_range
        );
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
        assert!(
            output
                .term_formula
                .normalized_types()
                .iter()
                .any(|(_, normalized)| normalized.head == TypeHeadRef::BuiltinObject)
        );
        assert!(
            output
                .term_formula
                .normalized_types()
                .iter()
                .any(|(_, normalized)| normalized.head == TypeHeadRef::BuiltinSet)
        );
        assert_eq!(output.term_formula.terms().len(), 2);
        let formula = output
            .term_formula
            .formulas()
            .iter()
            .map(|(_, formula)| formula)
            .next()
            .expect("Task 165 membership formula should be checked");
        assert_eq!(formula.kind, FormulaKind::Membership);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert_eq!(formula.expected_types.len(), 1);
        assert_eq!(formula.expected_types[0].term, output.payload.right_site);
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
    }
    #[test]
    fn active_three_edge_local_object_mode_reserved_variable_membership_fixture_consumes_four_expansions()
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
                    == "pass_type_elaboration_three_edge_local_object_mode_reserved_variable_membership_001"
            })
            .expect("Task 163 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 163 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 163 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_three_edge_local_object_mode_reserved_variable_membership_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 163 real AST should reach the three-edge local-object-mode membership seam");
        assert_source_reserved_variable_formula_output(&output)
            .expect("Task 163 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 4);
        assert_eq!(output.left_binding, BindingId::new(0));
        assert_eq!(output.right_binding, BindingId::new(1));
        assert_eq!(output.payload.left_lookup_ordinal, 2);
        assert_eq!(output.payload.right_lookup_ordinal, 3);
        assert_ne!(
            output.payload.reserve.bridge.bindings()[0].type_range,
            output.payload.reserve.bridge.bindings()[1].type_range
        );
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
        assert!(
            output
                .term_formula
                .normalized_types()
                .iter()
                .any(|(_, normalized)| normalized.head == TypeHeadRef::BuiltinObject)
        );
        assert!(
            output
                .term_formula
                .normalized_types()
                .iter()
                .any(|(_, normalized)| normalized.head == TypeHeadRef::BuiltinSet)
        );
        assert_eq!(output.term_formula.terms().len(), 2);
        let formula = output
            .term_formula
            .formulas()
            .iter()
            .map(|(_, formula)| formula)
            .next()
            .expect("Task 163 membership formula should be checked");
        assert_eq!(formula.kind, FormulaKind::Membership);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert_eq!(formula.expected_types.len(), 1);
        assert_eq!(formula.expected_types[0].term, output.payload.right_site);
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
    }

    #[test]
    fn active_three_edge_local_object_mode_reserved_variable_equality_fixture_consumes_four_expansions()
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
                    == "pass_type_elaboration_three_edge_local_object_mode_reserved_variable_equality_001"
            })
            .expect("Task 155 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 155 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 155 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_three_edge_local_object_mode_reserved_variable_equality_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 155 real AST should reach the three-edge local-object-mode equality seam");
        assert_source_reserved_variable_formula_output(&output)
            .expect("Task 155 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 4);
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
                source_mode_symbol_spelling(symbol) == Some("BaseThreeEdgeObjectModeEquality")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("Task 155 base object terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 155 normalized object type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
    }

    #[test]
    fn active_two_edge_local_mode_reserved_variable_equality_fixture_consumes_three_expansions() {
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
                    == "pass_type_elaboration_two_edge_local_mode_reserved_variable_equality_001"
            })
            .expect("Task 134 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 134 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 134 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_two_edge_local_mode_reserved_variable_equality_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 134 real AST should reach the two-edge local-mode equality seam");
        assert_source_reserved_variable_formula_output(&output)
            .expect("Task 134 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 3);
        assert!(matches!(
            output.left_result_input.head,
            TypeHeadInput::Symbol(_)
        ));
        assert_eq!(output.term_formula.normalized_types().len(), 1);
    }
    #[test]
    fn active_two_edge_local_mode_reserved_variable_inequality_fixture_consumes_three_expansions() {
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
                    == "pass_type_elaboration_two_edge_local_mode_reserved_variable_inequality_001"
            })
            .expect("Task 136 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 136 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 136 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_two_edge_local_mode_reserved_variable_inequality_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 136 real AST should reach the two-edge local-mode inequality seam");
        assert_source_reserved_variable_formula_output(&output)
            .expect("Task 136 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 3);
        assert!(matches!(
            output.left_result_input.head,
            TypeHeadInput::Symbol(_)
        ));
        assert_eq!(output.term_formula.normalized_types().len(), 1);
    }

    #[test]
    fn active_two_edge_local_object_mode_reserved_variable_inequality_fixture_consumes_three_expansions()
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
                    == "pass_type_elaboration_two_edge_local_object_mode_reserved_variable_inequality_001"
            })
            .expect("Task 137 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 137 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 137 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_two_edge_local_object_mode_reserved_variable_inequality_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 137 real AST should reach the two-edge local-object-mode inequality seam");
        assert_source_reserved_variable_formula_output(&output)
            .expect("Task 137 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 3);
        assert!(matches!(
            output.left_result_input.head,
            TypeHeadInput::Symbol(_)
        ));
        assert_eq!(output.term_formula.normalized_types().len(), 1);
    }

    #[test]
    fn active_two_edge_local_object_mode_reserved_variable_equality_fixture_consumes_three_expansions()
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
                    == "pass_type_elaboration_two_edge_local_object_mode_reserved_variable_equality_001"
            })
            .expect("Task 135 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 135 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 135 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_two_edge_local_object_mode_reserved_variable_equality_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 135 real AST should reach the two-edge local-object-mode equality seam");
        assert_source_reserved_variable_formula_output(&output)
            .expect("Task 135 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 3);
        assert!(matches!(
            output.left_result_input.head,
            TypeHeadInput::Symbol(_)
        ));
        assert_eq!(output.term_formula.normalized_types().len(), 1);
    }
    #[test]
    fn active_chained_local_mode_reserved_variable_inequality_fixture_consumes_both_expansions() {
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
                    == "pass_type_elaboration_chained_local_mode_reserved_variable_inequality_001"
            })
            .expect("Task 132 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 132 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 132 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_chained_local_mode_reserved_variable_inequality_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 132 real AST should reach the chained local-mode inequality seam");
        assert_source_reserved_variable_formula_output(&output)
            .expect("Task 132 real AST should preserve checked payload invariants");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 2);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        let formula = output
            .term_formula
            .formulas()
            .iter()
            .map(|(_, formula)| formula)
            .next()
            .expect("Task 132 checked inequality should exist");
        assert_eq!(formula.kind, FormulaKind::Inequality);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert!(formula.facts.is_empty());
    }

    #[test]
    fn active_chained_local_object_mode_reserved_variable_inequality_fixture_consumes_both_expansions()
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
                    == "pass_type_elaboration_chained_local_object_mode_reserved_variable_inequality_001"
            })
            .expect("Task 133 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 133 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 133 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_chained_local_object_mode_reserved_variable_inequality_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 133 real AST should reach the chained local-object-mode inequality seam");
        assert_source_reserved_variable_formula_output(&output)
            .expect("Task 133 real AST should preserve checked payload invariants");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 2);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        let formula = output
            .term_formula
            .formulas()
            .iter()
            .map(|(_, formula)| formula)
            .next()
            .expect("Task 133 checked inequality should exist");
        assert_eq!(formula.kind, FormulaKind::Inequality);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert!(formula.facts.is_empty());
    }
    #[test]
    fn active_local_object_mode_reserved_variable_equality_fixture_consumes_real_expansion() {
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
                    == "pass_type_elaboration_local_object_mode_reserved_variable_equality_001"
            })
            .expect("Task 128 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 128 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 128 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_local_object_mode_reserved_variable_equality_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 128 real AST should reach the local object-mode equality seam");
        assert_source_reserved_variable_formula_output(&output)
            .expect("Task 128 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 1);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 128 normalized object type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
    }

    #[test]
    fn active_chained_local_object_mode_reserved_variable_equality_fixture_consumes_real_expansions()
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
                    == "pass_type_elaboration_chained_local_object_mode_reserved_variable_equality_001"
            })
            .expect("Task 129 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 129 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 129 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = super::source_chained_local_object_mode_reserved_variable_equality_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 129 real AST should reach the chained object-mode equality seam");
        assert_source_reserved_variable_formula_output(&output)
            .expect("Task 129 real AST should preserve checked payload invariants");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 2);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
    }

    #[test]
    fn active_local_mode_reserved_variable_inequality_fixture_consumes_real_expansion() {
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
                case.id.0 == "pass_type_elaboration_local_mode_reserved_variable_inequality_001"
            })
            .expect("Task 130 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 130 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 130 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output =
            source_local_mode_reserved_variable_inequality_output(&ast, resolver.module, &symbols)
                .expect("Task 130 real AST should reach the local-mode inequality seam");
        assert_source_reserved_variable_formula_output(&output)
            .expect("Task 130 real AST should preserve checked payload invariants");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 1);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
    }

    #[test]
    fn active_local_object_mode_reserved_variable_inequality_fixture_consumes_real_expansion() {
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
                    == "pass_type_elaboration_local_object_mode_reserved_variable_inequality_001"
            })
            .expect("Task 131 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 131 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 131 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_local_object_mode_reserved_variable_inequality_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 131 real AST should reach the local object-mode inequality seam");
        assert_source_reserved_variable_formula_output(&output)
            .expect("Task 131 real AST should preserve checked payload invariants");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 1);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
    }

    #[test]
    fn active_reserved_variable_membership_fixture_preserves_real_checker_payload() {
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
            .find(|(_, case)| case.id.0 == "pass_type_elaboration_reserved_variable_membership_001")
            .expect("Task 120 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 120 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 120 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_reserved_variable_membership_output(&ast, resolver.module, &symbols)
            .expect("Task 120 real AST should reach the reserved-variable membership checker seam");
        assert_source_reserved_variable_formula_output(&output)
            .expect("Task 120 real AST should preserve every checked payload invariant");
    }

    #[test]
    fn active_reserved_variable_inequality_fixture_preserves_real_checker_payload() {
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
            .find(|(_, case)| case.id.0 == "pass_type_elaboration_reserved_variable_inequality_001")
            .expect("Task 121 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 121 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 121 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_reserved_variable_inequality_output(&ast, resolver.module, &symbols)
            .expect("Task 121 real AST should reach the reserved-variable inequality seam");
        assert_source_reserved_variable_formula_output(&output)
            .expect("Task 121 real AST should preserve every checked payload invariant");
    }

    #[test]
    fn active_reserved_variable_type_assertion_fixture_preserves_real_checker_payload() {
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
                case.id.0 == "pass_type_elaboration_reserved_variable_type_assertion_001"
            })
            .expect("Task 122 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 122 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 122 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output =
            source_reserved_variable_type_assertion_output(&ast, resolver.module, &symbols)
                .expect("Task 122 real AST should reach the reserved-variable type assertion seam");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 122 real AST should preserve every checked payload invariant");
    }

    #[test]
    fn active_local_mode_reserved_variable_type_assertion_fixture_consumes_real_expansion() {
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
                case.id.0 == "pass_type_elaboration_local_mode_reserved_variable_type_assertion_001"
            })
            .expect("Task 138 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 138 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 138 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_local_mode_reserved_variable_type_assertion_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 138 real AST should reach the local-mode type assertion seam");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 138 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 1);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .values()
            .next()
            .expect("Task 138 real expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 138 normalized set type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_eq!(normalized.source.range, terminal.radix.source_range);
    }

    #[test]
    fn active_local_mode_asserted_head_fixture_consumes_real_expansion() {
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
            .find(|(_, case)| case.id.0 == "pass_type_elaboration_local_mode_asserted_head_001")
            .expect("Task 182 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 182 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 182 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_local_mode_asserted_head_output(&ast, resolver.module, &symbols)
            .expect("Task 182 real AST should reach the formula-side local-mode seam");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 182 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 1);
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
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
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .values()
            .next()
            .expect("Task 182 real expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 182 normalized set type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_eq!(normalized.source.range, terminal.radix.source_range);
        assert_eq!(normalized.source.spelling, terminal.radix.spelling);
    }

    #[test]
    fn active_local_object_mode_asserted_head_fixture_consumes_real_expansion() {
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
                case.id.0 == "pass_type_elaboration_local_object_mode_asserted_head_001"
            })
            .expect("Task 183 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 183 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 183 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_local_object_mode_asserted_head_output(&ast, resolver.module, &symbols)
            .expect("Task 183 real AST should reach the object-terminal asserted-head seam");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 183 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 1);
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
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
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .values()
            .next()
            .expect("Task 183 real expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 183 normalized object type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_eq!(normalized.source.range, terminal.radix.source_range);
        assert_eq!(normalized.source.spelling, terminal.radix.spelling);
    }

    #[test]
    fn active_chained_local_mode_asserted_head_fixture_consumes_both_expansions() {
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
                case.id.0 == "pass_type_elaboration_chained_local_mode_asserted_head_001"
            })
            .expect("Task 184 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 184 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 184 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output =
            source_chained_local_mode_asserted_head_output(&ast, resolver.module, &symbols)
                .expect("Task 184 real AST should reach the one-edge asserted-head seam");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 184 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 2);
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
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
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some("BaseModeAssertedHead"))
            .map(|(_, expansion)| &expansion.radix)
            .expect("Task 184 real base expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 184 normalized set type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
        let (_, formula) = output
            .term_formula
            .formulas()
            .iter()
            .next()
            .expect("Task 184 checked formula should exist");
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
    }

    #[test]
    fn active_chained_local_mode_radix_asserted_head_fixture_consumes_immediate_expansion() {
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
        let plan = build_test_plan(&config).expect("Task 201 repository plan should build");
        let (ordinal, case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| {
                case.id.0 == "pass_type_elaboration_chained_local_mode_radix_asserted_head_001"
            })
            .expect("Task 201 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 201 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 201 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output =
            source_chained_local_mode_radix_asserted_head_output(&ast, resolver.module, &symbols)
                .expect("Task 201 real AST should reach the immediate-radix asserted-head seam");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 201 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 2);
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert_eq!(
            output.subject_result_input.spelling,
            "OuterModeRadixAssertedHead"
        );
        assert_eq!(
            output.asserted_type_input.spelling,
            "BaseModeRadixAssertedHead"
        );
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
        let outer_expansion = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("OuterModeRadixAssertedHead")
            })
            .map(|(_, expansion)| expansion)
            .expect("Task 201 outer binding expansion should exist");
        assert_eq!(outer_expansion.radix.spelling, "BaseModeRadixAssertedHead");
        assert_eq!(outer_expansion.radix.head, output.asserted_type_input.head);
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("BaseModeRadixAssertedHead")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("Task 201 base terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 201 normalized set type should exist");
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
    fn active_chained_local_object_mode_radix_asserted_head_fixture_consumes_immediate_expansion() {
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
        let plan = build_test_plan(&config).expect("Task 202 repository plan should build");
        let (ordinal, case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| {
                case.id.0
                    == "pass_type_elaboration_chained_local_object_mode_radix_asserted_head_001"
            })
            .expect("Task 202 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 202 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 202 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_chained_local_object_mode_radix_asserted_head_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 202 real AST should reach the object immediate-radix asserted-head seam");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 202 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 2);
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert_eq!(
            output.subject_result_input.spelling,
            "OuterObjectModeRadixAssertedHead"
        );
        assert_eq!(
            output.asserted_type_input.spelling,
            "BaseObjectModeRadixAssertedHead"
        );
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
        let outer_expansion = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("OuterObjectModeRadixAssertedHead")
            })
            .map(|(_, expansion)| expansion)
            .expect("Task 202 object outer binding expansion should exist");
        assert_eq!(
            outer_expansion.radix.spelling,
            "BaseObjectModeRadixAssertedHead"
        );
        assert_eq!(outer_expansion.radix.head, output.asserted_type_input.head);
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("BaseObjectModeRadixAssertedHead")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("Task 202 object base terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 202 normalized object type should exist");
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
    }

    #[test]
    fn active_two_edge_local_mode_radix_asserted_head_fixture_consumes_immediate_expansion() {
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
        let plan = build_test_plan(&config).expect("Task 203 repository plan should build");
        let (ordinal, case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| {
                case.id.0 == "pass_type_elaboration_two_edge_local_mode_radix_asserted_head_001"
            })
            .expect("Task 203 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 203 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 203 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output =
            source_two_edge_local_mode_radix_asserted_head_output(&ast, resolver.module, &symbols)
                .expect("Task 203 real AST should reach the two-edge immediate-radix seam");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 203 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 3);
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert_eq!(
            output.subject_result_input.spelling,
            "OuterTwoEdgeModeRadixAssertedHead"
        );
        assert_eq!(
            output.asserted_type_input.spelling,
            "MiddleTwoEdgeModeRadixAssertedHead"
        );
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
        let outer_expansion = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("OuterTwoEdgeModeRadixAssertedHead")
            })
            .map(|(_, expansion)| expansion)
            .expect("Task 203 outer binding expansion should exist");
        assert_eq!(
            outer_expansion.radix.spelling,
            "MiddleTwoEdgeModeRadixAssertedHead"
        );
        assert_eq!(outer_expansion.radix.head, output.asserted_type_input.head);
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("BaseTwoEdgeModeRadixAssertedHead")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("Task 203 base terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 203 normalized set type should exist");
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
    }

    #[test]
    fn active_two_edge_local_mode_two_hop_asserted_head_fixture_consumes_both_expansions() {
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
        let plan = build_test_plan(&config).expect("Task 211 repository plan should build");
        let (ordinal, case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| {
                case.id.0 == "pass_type_elaboration_two_edge_local_mode_two_hop_asserted_head_001"
            })
            .expect("Task 211 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 211 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 211 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_two_edge_local_mode_two_hop_asserted_head_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 211 real AST should reach the explicit two-hop asserted-head seam");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 211 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 3);
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert_eq!(
            output.subject_result_input.spelling,
            "OuterTwoHopModeAssertedHead"
        );
        assert_eq!(
            output.asserted_type_input.spelling,
            "BaseTwoHopModeAssertedHead"
        );
        let outer = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("OuterTwoHopModeAssertedHead")
            })
            .map(|(_, expansion)| expansion)
            .unwrap();
        let middle = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("MiddleTwoHopModeAssertedHead")
            })
            .map(|(_, expansion)| expansion)
            .unwrap();
        let base = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("BaseTwoHopModeAssertedHead")
            })
            .map(|(_, expansion)| expansion)
            .unwrap();
        assert_eq!(outer.radix.spelling, "MiddleTwoHopModeAssertedHead");
        assert_eq!(middle.radix.spelling, "BaseTwoHopModeAssertedHead");
        assert_eq!(middle.radix.head, output.asserted_type_input.head);
        assert_eq!(base.radix.spelling, "set");
        assert_eq!(base.radix.head, TypeHeadInput::BuiltinSet);
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .unwrap();
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_eq!(normalized.source.range, base.radix.source_range);
        assert_eq!(normalized.source.spelling, base.radix.spelling);
        let (_, term) = output.term_formula.terms().iter().next().unwrap();
        assert_eq!(term.status, TermStatus::Inferred);
        assert!(term.deferred.is_empty());
        let (_, formula) = output.term_formula.formulas().iter().next().unwrap();
        assert_eq!(formula.kind, FormulaKind::TypeAssertion);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert!(formula.deferred.is_empty());
        assert!(output.term_formula.diagnostics().is_empty());
    }

    #[test]
    fn two_hop_asserted_head_route_rejects_all_36_preexisting_owner_fixtures() {
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
        let plan = build_test_plan(&config).expect("Task 211 repository plan should build");
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
            "pass_type_elaboration_local_object_mode_long_chain_radix_asserted_head_001",
        ];
        assert_eq!(preexisting_owner_ids.len(), 36);
        for owner_id in preexisting_owner_ids {
            let (ordinal, case) = active_type_elaboration_cases(&plan)
                .enumerate()
                .find(|(_, case)| case.id.0 == owner_id)
                .unwrap_or_else(|| panic!("pre-existing owner fixture {owner_id} must be active"));
            let frontend = run_frontend(&workspace_root, case, ordinal)
                .unwrap_or_else(|error| panic!("owner fixture {owner_id} must parse: {error}"));
            assert!(
                frontend.diagnostics.is_empty(),
                "owner fixture {owner_id} must remain frontend-clean"
            );
            let ast = frontend
                .ast
                .unwrap_or_else(|| panic!("owner fixture {owner_id} must produce an AST"));
            let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
            assert!(
                resolver.detail_keys.is_empty(),
                "owner fixture {owner_id} must remain resolver-clean"
            );
            let symbols =
                augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
            assert_eq!(
                source_type_elaboration_detail_keys(&ast, resolver.module.clone(), &symbols),
                Vec::<String>::new(),
                "pre-existing owner fixture {owner_id} must still reach its active route"
            );
            assert!(
                extract_source_two_edge_local_mode_two_hop_asserted_head(
                    &ast,
                    resolver.module,
                    &symbols,
                )
                .is_none(),
                "Task 211 must reject pre-existing owner fixture {owner_id}"
            );
        }
    }

    #[test]
    fn active_two_edge_local_object_mode_radix_asserted_head_fixture_consumes_immediate_expansion()
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
        let plan = build_test_plan(&config).expect("Task 204 repository plan should build");
        let (ordinal, case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| {
                case.id.0
                    == "pass_type_elaboration_two_edge_local_object_mode_radix_asserted_head_001"
            })
            .expect("Task 204 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 204 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 204 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_two_edge_local_object_mode_radix_asserted_head_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 204 real AST should reach the two-edge object immediate-radix seam");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 204 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 3);
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert_eq!(
            output.subject_result_input.spelling,
            "OuterTwoEdgeObjectModeRadixAssertedHead"
        );
        assert_eq!(
            output.asserted_type_input.spelling,
            "MiddleTwoEdgeObjectModeRadixAssertedHead"
        );
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
        let outer_expansion = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol)
                    == Some("OuterTwoEdgeObjectModeRadixAssertedHead")
            })
            .map(|(_, expansion)| expansion)
            .expect("Task 204 outer binding expansion should exist");
        assert_eq!(
            outer_expansion.radix.spelling,
            "MiddleTwoEdgeObjectModeRadixAssertedHead"
        );
        assert_eq!(outer_expansion.radix.head, output.asserted_type_input.head);
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol)
                    == Some("BaseTwoEdgeObjectModeRadixAssertedHead")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("Task 204 base object terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 204 normalized object type should exist");
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
    }

    #[test]
    fn active_two_edge_local_object_mode_two_hop_asserted_head_fixture_consumes_both_expansions() {
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
        let plan = build_test_plan(&config).expect("Task 212 repository plan should build");
        let (ordinal, case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| {
                case.id.0
                    == "pass_type_elaboration_two_edge_local_object_mode_two_hop_asserted_head_001"
            })
            .expect("Task 212 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 212 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 212 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_two_edge_local_object_mode_two_hop_asserted_head_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 212 real AST should reach the explicit two-hop asserted-head seam");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 212 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 3);
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert_eq!(
            output.subject_result_input.spelling,
            "OuterTwoHopObjectModeAssertedHead"
        );
        assert_eq!(
            output.asserted_type_input.spelling,
            "BaseTwoHopObjectModeAssertedHead"
        );
        let outer = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("OuterTwoHopObjectModeAssertedHead")
            })
            .map(|(_, expansion)| expansion)
            .unwrap();
        let middle = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("MiddleTwoHopObjectModeAssertedHead")
            })
            .map(|(_, expansion)| expansion)
            .unwrap();
        let base = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("BaseTwoHopObjectModeAssertedHead")
            })
            .map(|(_, expansion)| expansion)
            .unwrap();
        assert_eq!(outer.radix.spelling, "MiddleTwoHopObjectModeAssertedHead");
        assert_eq!(middle.radix.spelling, "BaseTwoHopObjectModeAssertedHead");
        assert_eq!(middle.radix.head, output.asserted_type_input.head);
        assert_eq!(base.radix.spelling, "object");
        assert_eq!(base.radix.head, TypeHeadInput::BuiltinObject);
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .unwrap();
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_eq!(normalized.source.range, base.radix.source_range);
        assert_eq!(normalized.source.spelling, base.radix.spelling);
        let (_, term) = output.term_formula.terms().iter().next().unwrap();
        assert_eq!(term.status, TermStatus::Inferred);
        assert!(term.deferred.is_empty());
        let (_, formula) = output.term_formula.formulas().iter().next().unwrap();
        assert_eq!(formula.kind, FormulaKind::TypeAssertion);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert!(formula.deferred.is_empty());
        assert!(output.term_formula.diagnostics().is_empty());
    }

    #[test]
    fn active_three_edge_local_mode_two_hop_asserted_head_fixture_consumes_four_expansions() {
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
        let plan = build_test_plan(&config).expect("Task 213 repository plan should build");
        let (ordinal, case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| {
                case.id.0 == "pass_type_elaboration_three_edge_local_mode_two_hop_asserted_head_001"
            })
            .expect("Task 213 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 213 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 213 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_three_edge_local_mode_two_hop_asserted_head_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 213 real AST should reach the explicit two-link asserted-head seam");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 213 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 4);
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert_eq!(
            output.subject_result_input.spelling,
            "OuterThreeEdgeModeTwoHopAssertedHead"
        );
        assert_eq!(
            output.asserted_type_input.spelling,
            "InnerThreeEdgeModeTwoHopAssertedHead"
        );
        let expansion = |spelling| {
            output
                .payload
                .reserve
                .mode_expansions
                .iter()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(spelling))
                .map(|(_, expansion)| expansion)
                .unwrap()
        };
        let outer = expansion("OuterThreeEdgeModeTwoHopAssertedHead");
        let middle = expansion("MiddleThreeEdgeModeTwoHopAssertedHead");
        let inner = expansion("InnerThreeEdgeModeTwoHopAssertedHead");
        let base = expansion("BaseThreeEdgeModeTwoHopAssertedHead");
        assert_eq!(
            outer.radix.spelling,
            "MiddleThreeEdgeModeTwoHopAssertedHead"
        );
        assert_eq!(
            middle.radix.spelling,
            "InnerThreeEdgeModeTwoHopAssertedHead"
        );
        assert_eq!(middle.radix.head, output.asserted_type_input.head);
        assert_eq!(inner.radix.spelling, "BaseThreeEdgeModeTwoHopAssertedHead");
        assert_eq!(base.radix.spelling, "set");
        assert_eq!(base.radix.head, TypeHeadInput::BuiltinSet);
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .unwrap();
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_ne!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_eq!(normalized.source.range, base.radix.source_range);
        assert_eq!(normalized.source.spelling, base.radix.spelling);
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
    fn active_three_edge_local_object_mode_two_hop_asserted_head_fixture_consumes_four_expansions()
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
        let plan = build_test_plan(&config).expect("Task 214 repository plan should build");
        let (ordinal, case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| {
                case.id.0
                    == "pass_type_elaboration_three_edge_local_object_mode_two_hop_asserted_head_001"
            })
            .expect("Task 214 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 214 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 214 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_three_edge_local_object_mode_two_hop_asserted_head_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 214 real AST should reach the object two-link asserted-head seam");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 214 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 4);
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert_eq!(
            output.subject_result_input.spelling,
            "OuterThreeEdgeObjectModeTwoHopAssertedHead"
        );
        assert_eq!(
            output.asserted_type_input.spelling,
            "InnerThreeEdgeObjectModeTwoHopAssertedHead"
        );
        let expansion = |spelling| {
            output
                .payload
                .reserve
                .mode_expansions
                .iter()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(spelling))
                .map(|(_, expansion)| expansion)
                .unwrap()
        };
        let outer = expansion("OuterThreeEdgeObjectModeTwoHopAssertedHead");
        let middle = expansion("MiddleThreeEdgeObjectModeTwoHopAssertedHead");
        let inner = expansion("InnerThreeEdgeObjectModeTwoHopAssertedHead");
        let base = expansion("BaseThreeEdgeObjectModeTwoHopAssertedHead");
        assert_eq!(
            outer.radix.spelling,
            "MiddleThreeEdgeObjectModeTwoHopAssertedHead"
        );
        assert_eq!(
            middle.radix.spelling,
            "InnerThreeEdgeObjectModeTwoHopAssertedHead"
        );
        assert_eq!(middle.radix.head, output.asserted_type_input.head);
        assert_eq!(
            inner.radix.spelling,
            "BaseThreeEdgeObjectModeTwoHopAssertedHead"
        );
        assert_eq!(base.radix.spelling, "object");
        assert_eq!(base.radix.head, TypeHeadInput::BuiltinObject);
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .unwrap();
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_ne!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_eq!(normalized.source.range, base.radix.source_range);
        assert_eq!(normalized.source.spelling, base.radix.spelling);
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
    fn object_two_hop_asserted_head_route_rejects_all_37_preexisting_owner_fixtures() {
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
        let plan = build_test_plan(&config).expect("Task 212 repository plan should build");
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
            "pass_type_elaboration_local_object_mode_long_chain_radix_asserted_head_001",
            "pass_type_elaboration_two_edge_local_mode_two_hop_asserted_head_001",
        ];
        assert_eq!(preexisting_owner_ids.len(), 37);
        for owner_id in preexisting_owner_ids {
            let (ordinal, case) = active_type_elaboration_cases(&plan)
                .enumerate()
                .find(|(_, case)| case.id.0 == owner_id)
                .unwrap_or_else(|| panic!("existing owner fixture {owner_id} must be active"));
            let frontend = run_frontend(&workspace_root, case, ordinal)
                .unwrap_or_else(|error| panic!("owner fixture {owner_id} must parse: {error}"));
            assert!(
                frontend.diagnostics.is_empty(),
                "owner fixture {owner_id} must remain frontend-clean"
            );
            let ast = frontend
                .ast
                .unwrap_or_else(|| panic!("owner fixture {owner_id} must produce an AST"));
            let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
            assert!(
                resolver.detail_keys.is_empty(),
                "owner fixture {owner_id} must remain resolver-clean"
            );
            let symbols =
                augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
            assert_eq!(
                source_type_elaboration_detail_keys(&ast, resolver.module.clone(), &symbols),
                Vec::<String>::new(),
                "existing owner fixture {owner_id} must still reach its active route"
            );
            assert!(
                extract_source_two_edge_local_object_mode_two_hop_asserted_head(
                    &ast,
                    resolver.module,
                    &symbols,
                )
                .is_none(),
                "Task 212 must reject existing owner fixture {owner_id}"
            );
        }
    }

    #[test]
    fn three_edge_two_hop_asserted_head_route_rejects_all_38_prior_owner_fixtures() {
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
        let plan = build_test_plan(&config).expect("Task 213 repository plan should build");
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
        ];
        assert_eq!(prior_owner_ids.len(), 38);
        for owner_id in prior_owner_ids {
            let (ordinal, case) = active_type_elaboration_cases(&plan)
                .enumerate()
                .find(|(_, case)| case.id.0 == owner_id)
                .unwrap_or_else(|| panic!("prior owner fixture {owner_id} must be active"));
            let frontend = run_frontend(&workspace_root, case, ordinal)
                .unwrap_or_else(|error| panic!("owner fixture {owner_id} must parse: {error}"));
            assert!(
                frontend.diagnostics.is_empty(),
                "owner fixture {owner_id} must remain frontend-clean"
            );
            let ast = frontend
                .ast
                .unwrap_or_else(|| panic!("owner fixture {owner_id} must produce an AST"));
            let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
            assert!(
                resolver.detail_keys.is_empty(),
                "owner fixture {owner_id} must remain resolver-clean"
            );
            let symbols =
                augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
            assert_eq!(
                source_type_elaboration_detail_keys(&ast, resolver.module.clone(), &symbols),
                Vec::<String>::new(),
                "prior owner fixture {owner_id} must still reach its active route"
            );
            assert!(
                extract_source_three_edge_local_mode_two_hop_asserted_head(
                    &ast,
                    resolver.module,
                    &symbols,
                )
                .is_none(),
                "Task 213 must reject prior owner fixture {owner_id}"
            );
        }
    }

    #[test]
    fn object_three_edge_two_hop_asserted_head_route_rejects_all_39_prior_owner_fixtures() {
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
        let plan = build_test_plan(&config).expect("Task 214 repository plan should build");
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
        ];
        assert_eq!(prior_owner_ids.len(), 39);
        for owner_id in prior_owner_ids {
            let (ordinal, case) = active_type_elaboration_cases(&plan)
                .enumerate()
                .find(|(_, case)| case.id.0 == owner_id)
                .unwrap_or_else(|| panic!("prior owner fixture {owner_id} must be active"));
            let frontend = run_frontend(&workspace_root, case, ordinal)
                .unwrap_or_else(|error| panic!("owner fixture {owner_id} must parse: {error}"));
            assert!(
                frontend.diagnostics.is_empty(),
                "owner fixture {owner_id} must remain frontend-clean"
            );
            let ast = frontend
                .ast
                .unwrap_or_else(|| panic!("owner fixture {owner_id} must produce an AST"));
            let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
            assert!(
                resolver.detail_keys.is_empty(),
                "owner fixture {owner_id} must remain resolver-clean"
            );
            let symbols =
                augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
            assert_eq!(
                source_type_elaboration_detail_keys(&ast, resolver.module.clone(), &symbols),
                Vec::<String>::new(),
                "prior owner fixture {owner_id} must still reach its active route"
            );
            assert!(
                extract_source_three_edge_local_object_mode_two_hop_asserted_head(
                    &ast,
                    resolver.module,
                    &symbols,
                )
                .is_none(),
                "Task 214 must reject prior owner fixture {owner_id}"
            );
        }
    }

    #[test]
    fn active_three_edge_local_mode_radix_asserted_head_fixture_consumes_immediate_expansion() {
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
        let plan = build_test_plan(&config).expect("Task 205 repository plan should build");
        let (ordinal, case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| {
                case.id.0 == "pass_type_elaboration_three_edge_local_mode_radix_asserted_head_001"
            })
            .expect("Task 205 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 205 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 205 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_three_edge_local_mode_radix_asserted_head_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 205 real AST should reach the three-edge immediate-radix seam");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 205 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 4);
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert_eq!(
            output.subject_result_input.spelling,
            "OuterThreeEdgeModeRadixAssertedHead"
        );
        assert_eq!(
            output.asserted_type_input.spelling,
            "MiddleThreeEdgeModeRadixAssertedHead"
        );
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
        let outer_expansion = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("OuterThreeEdgeModeRadixAssertedHead")
            })
            .map(|(_, expansion)| expansion)
            .expect("Task 205 outer binding expansion should exist");
        assert_eq!(
            outer_expansion.radix.spelling,
            "MiddleThreeEdgeModeRadixAssertedHead"
        );
        assert_eq!(outer_expansion.radix.head, output.asserted_type_input.head);
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("BaseThreeEdgeModeRadixAssertedHead")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("Task 205 base set terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 205 normalized set type should exist");
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
    }

    #[test]
    fn active_four_edge_local_object_mode_radix_asserted_head_fixture_consumes_immediate_expansion()
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
        let plan = build_test_plan(&config).expect("Task 208 repository plan should build");
        let (ordinal, case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| {
                case.id.0
                    == "pass_type_elaboration_four_edge_local_object_mode_radix_asserted_head_001"
            })
            .expect("Task 208 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 208 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 208 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_four_edge_local_object_mode_radix_asserted_head_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 208 real AST should reach the four-edge immediate-radix seam");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 208 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 5);
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert_eq!(
            output.subject_result_input.spelling,
            "TooDeepFourEdgeObjectModeRadixAssertedHead"
        );
        assert_eq!(
            output.asserted_type_input.spelling,
            "OuterFourEdgeObjectModeRadixAssertedHead"
        );
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
        let outer_expansion = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol)
                    == Some("TooDeepFourEdgeObjectModeRadixAssertedHead")
            })
            .map(|(_, expansion)| expansion)
            .expect("Task 208 outer binding expansion should exist");
        assert_eq!(
            outer_expansion.radix.spelling,
            "OuterFourEdgeObjectModeRadixAssertedHead"
        );
        assert_eq!(outer_expansion.radix.head, output.asserted_type_input.head);
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol)
                    == Some("BaseFourEdgeObjectModeRadixAssertedHead")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("Task 208 base object terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 208 normalized object type should exist");
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
    }

    #[test]
    fn active_four_edge_local_mode_radix_asserted_head_fixture_consumes_immediate_expansion() {
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
        let plan = build_test_plan(&config).expect("Task 207 repository plan should build");
        let (ordinal, case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| {
                case.id.0 == "pass_type_elaboration_four_edge_local_mode_radix_asserted_head_001"
            })
            .expect("Task 207 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 207 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 207 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output =
            source_four_edge_local_mode_radix_asserted_head_output(&ast, resolver.module, &symbols)
                .expect("Task 207 real AST should reach the four-edge immediate-radix seam");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 207 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 5);
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert_eq!(
            output.subject_result_input.spelling,
            "TooDeepFourEdgeModeRadixAssertedHead"
        );
        assert_eq!(
            output.asserted_type_input.spelling,
            "OuterFourEdgeModeRadixAssertedHead"
        );
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
        let outer_expansion = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("TooDeepFourEdgeModeRadixAssertedHead")
            })
            .map(|(_, expansion)| expansion)
            .expect("Task 207 outer binding expansion should exist");
        assert_eq!(
            outer_expansion.radix.spelling,
            "OuterFourEdgeModeRadixAssertedHead"
        );
        assert_eq!(outer_expansion.radix.head, output.asserted_type_input.head);
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("BaseFourEdgeModeRadixAssertedHead")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("Task 207 base set terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 207 normalized set type should exist");
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
    }

    #[test]
    fn active_three_edge_local_object_mode_radix_asserted_head_fixture_consumes_immediate_expansion()
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
        let plan = build_test_plan(&config).expect("Task 206 repository plan should build");
        let (ordinal, case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| {
                case.id.0
                    == "pass_type_elaboration_three_edge_local_object_mode_radix_asserted_head_001"
            })
            .expect("Task 206 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 206 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 206 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_three_edge_local_object_mode_radix_asserted_head_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 206 real AST should reach the three-edge immediate-radix seam");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 206 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 4);
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert_eq!(
            output.subject_result_input.spelling,
            "OuterThreeEdgeObjectModeRadixAssertedHead"
        );
        assert_eq!(
            output.asserted_type_input.spelling,
            "MiddleThreeEdgeObjectModeRadixAssertedHead"
        );
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
        let outer_expansion = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol)
                    == Some("OuterThreeEdgeObjectModeRadixAssertedHead")
            })
            .map(|(_, expansion)| expansion)
            .expect("Task 206 outer binding expansion should exist");
        assert_eq!(
            outer_expansion.radix.spelling,
            "MiddleThreeEdgeObjectModeRadixAssertedHead"
        );
        assert_eq!(outer_expansion.radix.head, output.asserted_type_input.head);
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol)
                    == Some("BaseThreeEdgeObjectModeRadixAssertedHead")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("Task 206 base object terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 206 normalized object type should exist");
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
    }

    #[test]
    fn active_chained_local_object_mode_asserted_head_fixture_consumes_both_expansions() {
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
                case.id.0 == "pass_type_elaboration_chained_local_object_mode_asserted_head_001"
            })
            .expect("Task 185 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 185 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 185 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output =
            source_chained_local_object_mode_asserted_head_output(&ast, resolver.module, &symbols)
                .expect("Task 185 real AST should reach the one-edge asserted-head seam");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 185 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 2);
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
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
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("BaseObjectModeAssertedHead")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("Task 185 real base expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 185 normalized object type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
        let (_, formula) = output
            .term_formula
            .formulas()
            .iter()
            .next()
            .expect("Task 185 checked formula should exist");
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
    }

    #[test]
    fn active_local_object_mode_reserved_variable_type_assertion_fixture_consumes_real_expansion() {
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
                case.id.0 == "pass_type_elaboration_local_object_mode_reserved_variable_type_assertion_001"
            })
            .expect("Task 145 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 145 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 145 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_local_object_mode_reserved_variable_type_assertion_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 145 real AST should reach the local-object-mode type assertion seam");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 145 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 1);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .values()
            .next()
            .expect("Task 145 real expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 145 normalized object type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_eq!(normalized.source.range, terminal.radix.source_range);
    }

    #[test]
    fn active_chained_local_mode_reserved_variable_type_assertion_fixture_consumes_both_expansions()
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
                    == "pass_type_elaboration_chained_local_mode_reserved_variable_type_assertion_001"
            })
            .expect("Task 146 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 146 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 146 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_chained_local_mode_reserved_variable_type_assertion_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 146 real AST should reach the chained local-mode type assertion seam");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 146 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 2);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("BaseModeTypeAssertion")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("Task 146 base terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 146 normalized set type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
    }

    #[test]
    fn active_chained_local_object_mode_reserved_variable_type_assertion_fixture_consumes_both_expansions()
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
                    == "pass_type_elaboration_chained_local_object_mode_reserved_variable_type_assertion_001"
            })
            .expect("Task 147 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 147 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 147 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_chained_local_object_mode_reserved_variable_type_assertion_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 147 real AST should reach the chained local-object-mode type assertion seam");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 147 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 2);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("BaseObjectModeTypeAssertion")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("Task 147 base object terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 147 normalized object type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
    }

    #[test]
    fn active_two_edge_local_mode_reserved_variable_type_assertion_fixture_consumes_three_expansions()
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
                    == "pass_type_elaboration_two_edge_local_mode_reserved_variable_type_assertion_001"
            })
            .expect("Task 148 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 148 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 148 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_two_edge_local_mode_reserved_variable_type_assertion_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 148 real AST should reach the two-edge local-mode type assertion seam");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 148 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("BaseTwoEdgeModeTypeAssertion")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("Task 148 base terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 148 normalized set type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
    }

    #[test]
    fn active_three_edge_local_mode_asserted_head_fixture_consumes_four_expansions() {
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
        let plan = build_test_plan(&config).expect("Task 195 repository plan should build");
        let (ordinal, case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| {
                case.id.0 == "pass_type_elaboration_three_edge_local_mode_asserted_head_001"
            })
            .expect("Task 195 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 195 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 195 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = super::source_three_edge_local_mode_asserted_head_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 195 real AST should reach the three-edge asserted-head seam");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 195 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 4);
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert_eq!(
            output.subject_result_input.spelling,
            "OuterThreeEdgeModeAssertedHead"
        );
        assert_eq!(
            output.asserted_type_input.spelling,
            "OuterThreeEdgeModeAssertedHead"
        );
        assert_eq!(
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
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("BaseThreeEdgeModeAssertedHead")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("Task 195 base terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 195 normalized set type should exist");
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
    fn active_four_edge_local_mode_asserted_head_fixture_consumes_five_expansions() {
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
        let plan = build_test_plan(&config).expect("Task 197 repository plan should build");
        let (ordinal, case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| {
                case.id.0 == "pass_type_elaboration_four_edge_local_mode_asserted_head_001"
            })
            .expect("Task 197 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 197 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 197 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = super::source_four_edge_local_mode_asserted_head_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 197 real AST should reach the four-edge asserted-head seam");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 197 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 5);
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert_eq!(
            output.subject_result_input.spelling,
            "TooDeepFourEdgeModeAssertedHead"
        );
        assert_eq!(
            output.asserted_type_input.spelling,
            "TooDeepFourEdgeModeAssertedHead"
        );
        assert_eq!(
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
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("BaseFourEdgeModeAssertedHead")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("Task 197 base terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 197 normalized set type should exist");
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
    fn active_four_edge_local_object_mode_asserted_head_fixture_consumes_five_expansions() {
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
        let plan = build_test_plan(&config).expect("Task 198 repository plan should build");
        let (ordinal, case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| {
                case.id.0 == "pass_type_elaboration_four_edge_local_object_mode_asserted_head_001"
            })
            .expect("Task 198 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 198 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 198 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = super::source_four_edge_local_object_mode_asserted_head_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 198 real AST should reach the four-edge object asserted-head seam");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 198 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 5);
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert_eq!(
            output.subject_result_input.spelling,
            "TooDeepFourEdgeObjectModeAssertedHead"
        );
        assert_eq!(
            output.asserted_type_input.spelling,
            "TooDeepFourEdgeObjectModeAssertedHead"
        );
        assert_eq!(
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
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("BaseFourEdgeObjectModeAssertedHead")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("Task 198 base object terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 198 normalized object type should exist");
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
    fn active_three_edge_local_object_mode_asserted_head_fixture_consumes_four_expansions() {
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
        let plan = build_test_plan(&config).expect("Task 196 repository plan should build");
        let (ordinal, case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| {
                case.id.0 == "pass_type_elaboration_three_edge_local_object_mode_asserted_head_001"
            })
            .expect("Task 196 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 196 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 196 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = super::source_three_edge_local_object_mode_asserted_head_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 196 real AST should reach the three-edge asserted-head seam");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 196 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 4);
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert_eq!(
            output.subject_result_input.spelling,
            "OuterThreeEdgeObjectModeAssertedHead"
        );
        assert_eq!(
            output.asserted_type_input.spelling,
            "OuterThreeEdgeObjectModeAssertedHead"
        );
        assert_eq!(
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
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("BaseThreeEdgeObjectModeAssertedHead")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("Task 196 base terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 196 normalized object type should exist");
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
    fn active_two_edge_local_mode_asserted_head_fixture_consumes_three_expansions() {
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
                case.id.0 == "pass_type_elaboration_two_edge_local_mode_asserted_head_001"
            })
            .expect("Task 186 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 186 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 186 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output =
            source_two_edge_local_mode_asserted_head_output(&ast, resolver.module, &symbols)
                .expect(
                    "Task 186 real AST should reach the two-edge local-mode type assertion seam",
                );
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 186 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 3);
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
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
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("BaseTwoEdgeModeAssertedHead")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("Task 186 base terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 186 normalized set type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
        let (_, term) = output
            .term_formula
            .terms()
            .iter()
            .next()
            .expect("Task 186 inferred term should exist");
        assert_eq!(term.status, TermStatus::Inferred);
        assert!(term.deferred.is_empty());
        let (_, formula) = output
            .term_formula
            .formulas()
            .iter()
            .next()
            .expect("Task 186 checked formula should exist");
        assert_eq!(formula.kind, FormulaKind::TypeAssertion);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert!(formula.expected_types.is_empty());
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
    }

    #[test]
    fn active_two_edge_local_object_mode_asserted_head_fixture_consumes_three_expansions() {
        const CASE_ID: &str = "pass_type_elaboration_two_edge_local_object_mode_asserted_head_001";
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("mizar-test crate should live below the workspace root")
            .to_path_buf();
        let config = DiscoveryConfig {
            tests_root: workspace_root.join("tests"),
            manifest_path: workspace_root.join("tests/coverage/spec_trace.toml"),
            workspace_root: workspace_root.clone(),
            profile: TestProfile::Fast,
            validation_mode: ValidationMode::Metadata,
        };
        let plan = build_test_plan(&config).expect("Task 187 repository plan should build");
        let (ordinal, case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| case.id.0 == CASE_ID)
            .expect("Task 187 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 187 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 187 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output =
            source_two_edge_local_object_mode_asserted_head_output(&ast, resolver.module, &symbols)
                .expect("Task 187 real AST should reach the exact object-terminal seam");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 187 real AST should preserve checker payload invariants");
        assert_eq!(
            (
                output.payload.reserve.mode_expansions.len(),
                output.subject_binding,
                output.term_formula.type_entries().len(),
                output.term_formula.normalized_types().len(),
            ),
            (3, BindingId::new(0), 3, 1)
        );
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
        assert!(matches!(
            output.asserted_type_input.head,
            TypeHeadInput::Symbol(_)
        ));
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
            .expect("Task 187 terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .unwrap();
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
    }

    #[test]
    fn active_two_edge_local_object_mode_reserved_variable_type_assertion_fixture_consumes_three_expansions()
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
                    == "pass_type_elaboration_two_edge_local_object_mode_reserved_variable_type_assertion_001"
            })
            .expect("Task 149 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 149 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 149 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_two_edge_local_object_mode_reserved_variable_type_assertion_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect(
            "Task 149 real AST should reach the two-edge local-object-mode type assertion seam",
        );
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 149 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("BaseTwoEdgeObjectModeTypeAssertion")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("Task 149 base terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 149 normalized object type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
    }

    #[test]
    fn active_three_edge_local_mode_reserved_variable_type_assertion_fixture_consumes_four_expansions()
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
                    == "pass_type_elaboration_three_edge_local_mode_reserved_variable_type_assertion_001"
            })
            .expect("Task 150 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 150 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 150 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_three_edge_local_mode_reserved_variable_type_assertion_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 150 real AST should reach the three-edge local-mode type assertion seam");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 150 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 4);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("BaseThreeEdgeModeTypeAssertion")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("Task 150 base terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 150 normalized set type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
    }

    #[test]
    fn active_three_edge_local_object_mode_reserved_variable_type_assertion_fixture_consumes_four_expansions()
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
                    == "pass_type_elaboration_three_edge_local_object_mode_reserved_variable_type_assertion_001"
            })
            .expect("Task 151 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 151 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 151 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_three_edge_local_object_mode_reserved_variable_type_assertion_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect(
            "Task 151 real AST should reach the three-edge local-object-mode type assertion seam",
        );
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 151 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 4);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("BaseThreeEdgeObjectModeTypeAssertion")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("Task 151 base terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 151 normalized object type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
    }

    #[test]
    fn active_four_edge_local_mode_reserved_variable_type_assertion_fixture_consumes_five_expansions()
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
                    == "pass_type_elaboration_four_edge_local_mode_reserved_variable_type_assertion_001"
            })
            .expect("Task 152 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 152 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 152 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_four_edge_local_mode_reserved_variable_type_assertion_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 152 real AST should reach the four-edge local-mode type assertion seam");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 152 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 5);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("BaseFourEdgeModeTypeAssertion")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("Task 152 base terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 152 normalized set type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
    }

    #[test]
    fn active_four_edge_local_object_mode_reserved_variable_type_assertion_fixture_consumes_five_expansions()
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
                    == "pass_type_elaboration_four_edge_local_object_mode_reserved_variable_type_assertion_001"
            })
            .expect("Task 153 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 153 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 153 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_four_edge_local_object_mode_reserved_variable_type_assertion_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect(
            "Task 153 real AST should reach the four-edge local-object-mode type assertion seam",
        );
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 153 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 5);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        let terminal = output
            .payload
            .reserve
            .mode_expansions
            .iter()
            .find(|(symbol, _)| {
                source_mode_symbol_spelling(symbol) == Some("BaseFourEdgeObjectModeTypeAssertion")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("Task 153 base object terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 153 normalized object type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
    }

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
            Vec::<String>::new()
        );
        let contradiction_output =
            source_contradiction_formula_output(&contradiction_theorem, module.clone(), &symbols)
                .expect("exact standalone contradiction bridge should produce checker output");
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
        assert!(contradiction_output.terms().is_empty());
        assert_eq!(contradiction_output.formulas().len(), 1);
        assert!(contradiction_output.diagnostics().is_empty());
        let (_, checked_contradiction) = contradiction_output
            .formulas()
            .iter()
            .next()
            .expect("standalone contradiction should be checked");
        assert_eq!(
            checked_contradiction.site,
            contradiction_payload.formula_site
        );
        assert_eq!(checked_contradiction.context, BindingContextId::new(0));
        assert_eq!(checked_contradiction.kind, FormulaKind::Contradiction);
        assert_eq!(checked_contradiction.status, FormulaStatus::Checked);
        assert!(checked_contradiction.terms.is_empty());
        assert!(checked_contradiction.asserted_type.is_none());
        assert!(checked_contradiction.expected_types.is_empty());
        assert!(checked_contradiction.candidate_set.is_none());
        assert!(checked_contradiction.facts.is_empty());
        assert!(checked_contradiction.deferred.is_empty());
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
        let type_assertion_theorem = builtin_type_assertion_theorem_ast(
            source_id,
            "BuiltinTypeAssertionPayloadBoundary",
            "1",
            ReserveTypeShape::Builtin("set"),
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
        assert_eq!(type_assertion_output.terms().len(), 1);
        let (_, checked_subject) = type_assertion_output
            .terms()
            .iter()
            .next()
            .expect("subject term should be checked");
        assert_eq!(checked_subject.kind, TermKind::Numeral);
        assert_eq!(checked_subject.status, TermStatus::Partial);
        assert_eq!(checked_subject.site, type_assertion_payload.subject_site);
        assert_eq!(type_assertion_output.formulas().len(), 1);
        let (_, checked_formula) = type_assertion_output
            .formulas()
            .iter()
            .next()
            .expect("type assertion formula should be checked");
        assert_eq!(checked_formula.site, type_assertion_payload.formula_site);
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
            type_assertion_payload.asserted_type.range
        );
        let imported_predicate_functor_symbols =
            imported_predicate_functor_symbol_env(symbols.module_id().clone());
        let imported_predicate_functor_theorem = imported_predicate_functor_theorem_ast(
            source_id,
            &["parser.type_fixtures"],
            exact_imported_predicate_functor_theorem_spec(),
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
        let expected_formula_sites = surface_sites_for_kind_ranges(
            &set_enumeration_theorem,
            SurfaceNodeKind::BuiltinPredicateApplication,
            &[expected_formula_range],
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
            set_enumeration_payload.formula_site
        );
        assert_eq!(checked_set_formula.kind, FormulaKind::Equality);
        assert_eq!(checked_set_formula.status, FormulaStatus::Partial);
        assert_eq!(
            checked_set_formula.terms,
            vec![
                set_enumeration_payload.left_site.clone(),
                set_enumeration_payload.right_site.clone(),
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
        assert_eq!(
            imported_predicate_functor_payload
                .predicate_symbol
                .module()
                .path()
                .as_str(),
            "parser.type_fixtures"
        );
        assert_eq!(
            imported_predicate_functor_payload
                .functor_symbol
                .module()
                .path()
                .as_str(),
            "parser.type_fixtures"
        );
        assert_eq!(imported_predicate_functor_output.terms().len(), 4);
        let checked_left = imported_predicate_functor_output
            .terms()
            .iter()
            .map(|(_, term)| term)
            .find(|term| term.site == imported_predicate_functor_payload.left_site)
            .expect("left numeral term should be checked");
        assert_eq!(checked_left.kind, TermKind::Numeral);
        assert_eq!(checked_left.status, TermStatus::Partial);
        let checked_functor_left = imported_predicate_functor_output
            .terms()
            .iter()
            .map(|(_, term)| term)
            .find(|term| term.site == imported_predicate_functor_payload.functor_left_site)
            .expect("functor left numeral term should be checked");
        assert_eq!(checked_functor_left.kind, TermKind::Numeral);
        assert_eq!(checked_functor_left.status, TermStatus::Partial);
        let checked_functor_right = imported_predicate_functor_output
            .terms()
            .iter()
            .map(|(_, term)| term)
            .find(|term| term.site == imported_predicate_functor_payload.functor_right_site)
            .expect("functor right numeral term should be checked");
        assert_eq!(checked_functor_right.kind, TermKind::Numeral);
        assert_eq!(checked_functor_right.status, TermStatus::Partial);
        let checked_functor = imported_predicate_functor_output
            .terms()
            .iter()
            .map(|(_, term)| term)
            .find(|term| term.site == imported_predicate_functor_payload.functor_site)
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
            imported_predicate_functor_payload.formula_site
        );
        assert_eq!(
            checked_predicate_formula.kind,
            FormulaKind::PredicateApplication
        );
        assert_eq!(checked_predicate_formula.status, FormulaStatus::Partial);
        assert_eq!(
            checked_predicate_formula.terms,
            vec![
                imported_predicate_functor_payload.left_site.clone(),
                imported_predicate_functor_payload.functor_site.clone(),
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
            assert_eq!(
                source_type_elaboration_detail_keys(
                    &gap_case,
                    imported_predicate_functor_symbols.module_id().clone(),
                    &imported_predicate_functor_symbols
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
        ] {
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
        let imported_attribute_assertion_theorem = imported_attribute_assertion_theorem_ast(
            source_id,
            &["parser.type_fixtures"],
            "ImportedAttributeAssertionPayloadBoundary",
            "1",
            "empty",
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
        assert_eq!(
            imported_attribute_assertion_payload
                .attribute_symbol
                .module()
                .path()
                .as_str(),
            "parser.type_fixtures"
        );
        assert_eq!(imported_attribute_assertion_output.terms().len(), 1);
        let (_, checked_attribute_subject) = imported_attribute_assertion_output
            .terms()
            .iter()
            .next()
            .expect("attribute assertion subject term should be checked");
        assert_eq!(checked_attribute_subject.kind, TermKind::Numeral);
        assert_eq!(checked_attribute_subject.status, TermStatus::Partial);
        assert_eq!(
            checked_attribute_subject.site,
            imported_attribute_assertion_payload.subject_site
        );
        assert_eq!(checked_attribute_subject.context, BindingContextId::new(0));
        assert!(checked_attribute_subject.candidate_set.is_none());
        assert_eq!(imported_attribute_assertion_output.formulas().len(), 1);
        let (_, checked_attribute_formula) = imported_attribute_assertion_output
            .formulas()
            .iter()
            .next()
            .expect("attribute assertion formula should be checked");
        assert_eq!(
            checked_attribute_formula.site,
            imported_attribute_assertion_payload.formula_site
        );
        assert_eq!(
            checked_attribute_formula.kind,
            FormulaKind::AttributeAssertion
        );
        assert_eq!(checked_attribute_formula.status, FormulaStatus::Partial);
        assert_eq!(checked_attribute_formula.context, BindingContextId::new(0));
        assert_eq!(
            checked_attribute_formula.terms,
            vec![imported_attribute_assertion_payload.subject_site.clone()]
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
            assert_eq!(
                source_type_elaboration_detail_keys(
                    &gap_case,
                    imported_attribute_assertion_symbols.module_id().clone(),
                    &imported_attribute_assertion_symbols
                ),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
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
        ] {
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
        let imported_non_empty_attribute_assertion_theorem =
            imported_non_empty_attribute_assertion_theorem_ast(
                source_id,
                &["parser.type_fixtures"],
                "ImportedNonEmptyAttributeAssertionPayloadBoundary",
                "1",
                "empty",
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
        assert_eq!(
            imported_non_empty_attribute_assertion_payload
                .attribute_symbol
                .module()
                .path()
                .as_str(),
            "parser.type_fixtures"
        );
        assert_eq!(
            imported_non_empty_attribute_assertion_output.terms().len(),
            1
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
            imported_non_empty_attribute_assertion_payload.subject_site
        );
        assert_eq!(checked_non_empty_subject.context, BindingContextId::new(0));
        assert!(checked_non_empty_subject.candidate_set.is_none());
        assert_eq!(
            imported_non_empty_attribute_assertion_output
                .formulas()
                .len(),
            1
        );
        let (_, checked_non_empty_formula) = imported_non_empty_attribute_assertion_output
            .formulas()
            .iter()
            .next()
            .expect("non-empty attribute assertion formula should be checked");
        assert_eq!(
            checked_non_empty_formula.site,
            imported_non_empty_attribute_assertion_payload.formula_site
        );
        assert_eq!(
            checked_non_empty_formula.kind,
            FormulaKind::AttributeAssertion
        );
        assert_eq!(checked_non_empty_formula.status, FormulaStatus::Partial);
        assert_eq!(checked_non_empty_formula.context, BindingContextId::new(0));
        assert_eq!(
            checked_non_empty_formula.terms,
            vec![
                imported_non_empty_attribute_assertion_payload
                    .subject_site
                    .clone()
            ]
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
        ] {
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
                ["2", "1"],
                "=",
                ["1", "2"],
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
    include!("tests/type_elaboration/long_chain.rs");
    #[test]
    fn source_four_edge_local_mode_reserved_variable_inequality_consumes_five_expansions() {
        let source_id = source_id(168);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("four_edge_local_mode_reserved_variable_inequality"),
        );
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseFourEdgeModeInequality", SymbolKind::Mode),
                ("InnerFourEdgeModeInequality", SymbolKind::Mode),
                ("MiddleFourEdgeModeInequality", SymbolKind::Mode),
                ("OuterFourEdgeModeInequality", SymbolKind::Mode),
                ("TooDeepFourEdgeModeInequality", SymbolKind::Mode),
                ("ExtraFourEdgeModeInequality", SymbolKind::Mode),
            ],
        );
        let theorem = IdentifierBinaryTheoremSpec {
            status: None,
            label: "FourEdgeLocalModeReservedVariableInequalityPayloadBoundary",
            left: "z",
            operator: "<>",
            right: "z",
            recovered_label: false,
        };
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    "BaseFourEdgeModeInequality",
                    "BaseFourEdgeModeInequalityDef",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition_with_label(
                    "InnerFourEdgeModeInequality",
                    "InnerFourEdgeModeInequalityDef",
                    ReserveTypeShape::QualifiedSymbol("BaseFourEdgeModeInequality"),
                ),
                mode_definition_with_label(
                    "MiddleFourEdgeModeInequality",
                    "MiddleFourEdgeModeInequalityDef",
                    ReserveTypeShape::QualifiedSymbol("InnerFourEdgeModeInequality"),
                ),
                mode_definition_with_label(
                    "OuterFourEdgeModeInequality",
                    "OuterFourEdgeModeInequalityDef",
                    ReserveTypeShape::QualifiedSymbol("MiddleFourEdgeModeInequality"),
                ),
                mode_definition_with_label(
                    "TooDeepFourEdgeModeInequality",
                    "TooDeepFourEdgeModeInequalityDef",
                    ReserveTypeShape::QualifiedSymbol("OuterFourEdgeModeInequality"),
                ),
            ]
        };
        let reserve = || {
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("TooDeepFourEdgeModeInequality"),
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
        let payload = extract_source_four_edge_local_mode_reserved_variable_inequality(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact four-edge local-mode reserved-variable inequality should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 5);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(payload.reserve.bridge.bindings()[0].spelling, "z");
        assert_eq!(
            payload.reserve.bridge.bindings()[0].type_spelling,
            "TooDeepFourEdgeModeInequality"
        );
        assert_eq!(payload.left_lookup_ordinal, 1);
        assert_eq!(payload.right_lookup_ordinal, 2);

        let output = source_four_edge_local_mode_reserved_variable_inequality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact four-edge local-mode inequality should reach TermFormulaChecker");
        assert_source_reserved_variable_formula_output(&output)
            .expect("four-edge local-mode inequality invariants should hold");
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
            assert_eq!(input.spelling, "TooDeepFourEdgeModeInequality");
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
                source_mode_symbol_spelling(symbol) == Some("BaseFourEdgeModeInequality")
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
            .expect("inequality formula should be checked");
        assert_eq!(formula.kind, FormulaKind::Inequality);
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
                TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY
                    .to_owned(),
            ]
        };
        let mut missing_left_expected =
            source_four_edge_local_mode_reserved_variable_inequality_output(
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
            source_four_edge_local_mode_reserved_variable_inequality_output(
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
            "BaseFourEdgeModeInequality",
            "InnerFourEdgeModeInequality",
            "MiddleFourEdgeModeInequality",
            "OuterFourEdgeModeInequality",
            "TooDeepFourEdgeModeInequality",
        ] {
            let mut invalid = source_four_edge_local_mode_reserved_variable_inequality_output(
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
                    TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY
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
            "BaseFourEdgeModeInequality",
            "InnerFourEdgeModeInequality",
            "MiddleFourEdgeModeInequality",
            "OuterFourEdgeModeInequality",
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
                ReserveTypeShape::QualifiedSymbol("ExtraFourEdgeModeInequality");
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
            ReserveTypeShape::QualifiedSymbol("BaseFourEdgeModeInequality");
        assert_extraction_gap(one_edge_outermost_radix);

        let mut two_edge_outermost_radix = exact_modes();
        two_edge_outermost_radix[4].rhs_shape =
            ReserveTypeShape::QualifiedSymbol("InnerFourEdgeModeInequality");
        assert_extraction_gap(two_edge_outermost_radix);

        let mut three_edge_outermost_radix = exact_modes();
        three_edge_outermost_radix[4].rhs_shape =
            ReserveTypeShape::QualifiedSymbol("MiddleFourEdgeModeInequality");
        assert_extraction_gap(three_edge_outermost_radix);

        let mut cyclic = exact_modes();
        cyclic[0].rhs_shape = ReserveTypeShape::QualifiedSymbol("TooDeepFourEdgeModeInequality");
        assert_extraction_gap(cyclic);

        assert_extraction_gap(vec![
            exact_modes()[0],
            mode_definition_with_label(
                "ExtraFourEdgeModeInequality",
                "ExtraFourEdgeModeInequalityDef",
                ReserveTypeShape::QualifiedSymbol("BaseFourEdgeModeInequality"),
            ),
            mode_definition_with_label(
                "InnerFourEdgeModeInequality",
                "InnerFourEdgeModeInequalityDef",
                ReserveTypeShape::QualifiedSymbol("ExtraFourEdgeModeInequality"),
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
                    ReserveTypeShape::QualifiedSymbolWithArgs("TooDeepFourEdgeModeInequality"),
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
    fn active_four_edge_local_mode_reserved_variable_inequality_fixture_consumes_five_expansions() {
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
                    == "pass_type_elaboration_four_edge_local_mode_reserved_variable_inequality_001"
            })
            .expect("Task 168 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 168 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 168 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_four_edge_local_mode_reserved_variable_inequality_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 168 real AST should reach the four-edge local-mode inequality seam");
        assert_source_reserved_variable_formula_output(&output)
            .expect("Task 168 real AST should preserve every checked payload invariant");
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
                source_mode_symbol_spelling(symbol) == Some("BaseFourEdgeModeInequality")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("Task 168 base terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 168 normalized set type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
    }
    #[test]
    fn source_four_edge_local_object_mode_reserved_variable_equality_consumes_five_expansions() {
        let source_id = source_id(167);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("four_edge_local_object_mode_reserved_variable_equality"),
        );
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseFourEdgeObjectModeEquality", SymbolKind::Mode),
                ("InnerFourEdgeObjectModeEquality", SymbolKind::Mode),
                ("MiddleFourEdgeObjectModeEquality", SymbolKind::Mode),
                ("OuterFourEdgeObjectModeEquality", SymbolKind::Mode),
                ("TooDeepFourEdgeObjectModeEquality", SymbolKind::Mode),
                ("ExtraFourEdgeObjectModeEquality", SymbolKind::Mode),
            ],
        );
        let theorem = IdentifierBinaryTheoremSpec {
            status: None,
            label: "FourEdgeLocalObjectModeReservedVariableEqualityPayloadBoundary",
            left: "z",
            operator: "=",
            right: "z",
            recovered_label: false,
        };
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    "BaseFourEdgeObjectModeEquality",
                    "BaseFourEdgeObjectModeEqualityDef",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition_with_label(
                    "InnerFourEdgeObjectModeEquality",
                    "InnerFourEdgeObjectModeEqualityDef",
                    ReserveTypeShape::QualifiedSymbol("BaseFourEdgeObjectModeEquality"),
                ),
                mode_definition_with_label(
                    "MiddleFourEdgeObjectModeEquality",
                    "MiddleFourEdgeObjectModeEqualityDef",
                    ReserveTypeShape::QualifiedSymbol("InnerFourEdgeObjectModeEquality"),
                ),
                mode_definition_with_label(
                    "OuterFourEdgeObjectModeEquality",
                    "OuterFourEdgeObjectModeEqualityDef",
                    ReserveTypeShape::QualifiedSymbol("MiddleFourEdgeObjectModeEquality"),
                ),
                mode_definition_with_label(
                    "TooDeepFourEdgeObjectModeEquality",
                    "TooDeepFourEdgeObjectModeEqualityDef",
                    ReserveTypeShape::QualifiedSymbol("OuterFourEdgeObjectModeEquality"),
                ),
            ]
        };
        let reserve = || {
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("TooDeepFourEdgeObjectModeEquality"),
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
        let payload = extract_source_four_edge_local_object_mode_reserved_variable_equality(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact four-edge local-object-mode reserved-variable equality should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 5);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(payload.reserve.bridge.bindings()[0].spelling, "z");
        assert_eq!(
            payload.reserve.bridge.bindings()[0].type_spelling,
            "TooDeepFourEdgeObjectModeEquality"
        );
        assert_eq!(payload.left_lookup_ordinal, 1);
        assert_eq!(payload.right_lookup_ordinal, 2);

        let output = source_four_edge_local_object_mode_reserved_variable_equality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact four-edge local-object-mode equality should reach TermFormulaChecker");
        assert_source_reserved_variable_formula_output(&output)
            .expect("four-edge local-object-mode equality invariants should hold");
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
            assert_eq!(input.spelling, "TooDeepFourEdgeObjectModeEquality");
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
                source_mode_symbol_spelling(symbol) == Some("BaseFourEdgeObjectModeEquality")
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
                TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY
                    .to_owned(),
            ]
        };
        let mut missing_left_expected =
            source_four_edge_local_object_mode_reserved_variable_equality_output(
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
            source_four_edge_local_object_mode_reserved_variable_equality_output(
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
            "BaseFourEdgeObjectModeEquality",
            "InnerFourEdgeObjectModeEquality",
            "MiddleFourEdgeObjectModeEquality",
            "OuterFourEdgeObjectModeEquality",
            "TooDeepFourEdgeObjectModeEquality",
        ] {
            let mut invalid = source_four_edge_local_object_mode_reserved_variable_equality_output(
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
                    TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY
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
            "BaseFourEdgeObjectModeEquality",
            "InnerFourEdgeObjectModeEquality",
            "MiddleFourEdgeObjectModeEquality",
            "OuterFourEdgeObjectModeEquality",
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
                ReserveTypeShape::QualifiedSymbol("ExtraFourEdgeObjectModeEquality");
            assert_extraction_gap(wrong_radix);
        }

        let mut duplicate = exact_modes();
        duplicate.push(duplicate[0]);
        assert_extraction_gap(duplicate);

        let mut attributed_terminal = exact_modes();
        attributed_terminal[0].rhs_shape = ReserveTypeShape::AttributedSet;
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
            ReserveTypeShape::QualifiedSymbol("BaseFourEdgeObjectModeEquality");
        assert_extraction_gap(one_edge_outermost_radix);

        let mut two_edge_outermost_radix = exact_modes();
        two_edge_outermost_radix[4].rhs_shape =
            ReserveTypeShape::QualifiedSymbol("InnerFourEdgeObjectModeEquality");
        assert_extraction_gap(two_edge_outermost_radix);

        let mut three_edge_outermost_radix = exact_modes();
        three_edge_outermost_radix[4].rhs_shape =
            ReserveTypeShape::QualifiedSymbol("MiddleFourEdgeObjectModeEquality");
        assert_extraction_gap(three_edge_outermost_radix);

        let mut cyclic = exact_modes();
        cyclic[0].rhs_shape =
            ReserveTypeShape::QualifiedSymbol("TooDeepFourEdgeObjectModeEquality");
        assert_extraction_gap(cyclic);

        assert_extraction_gap(vec![
            exact_modes()[0],
            mode_definition_with_label(
                "ExtraFourEdgeObjectModeEquality",
                "ExtraFourEdgeObjectModeEqualityDef",
                ReserveTypeShape::QualifiedSymbol("BaseFourEdgeObjectModeEquality"),
            ),
            mode_definition_with_label(
                "InnerFourEdgeObjectModeEquality",
                "InnerFourEdgeObjectModeEqualityDef",
                ReserveTypeShape::QualifiedSymbol("ExtraFourEdgeObjectModeEquality"),
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
                    ReserveTypeShape::QualifiedSymbolWithArgs("TooDeepFourEdgeObjectModeEquality"),
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
    fn active_four_edge_local_object_mode_reserved_variable_equality_fixture_consumes_five_expansions()
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
                    == "pass_type_elaboration_four_edge_local_object_mode_reserved_variable_equality_001"
            })
            .expect("Task 167 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 167 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 167 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_four_edge_local_object_mode_reserved_variable_equality_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 167 real AST should reach the four-edge local-object-mode equality seam");
        assert_source_reserved_variable_formula_output(&output)
            .expect("Task 167 real AST should preserve every checked payload invariant");
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
                source_mode_symbol_spelling(symbol) == Some("BaseFourEdgeObjectModeEquality")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("Task 167 base terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 167 normalized object type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
    }
    #[test]
    fn source_four_edge_local_object_mode_reserved_variable_inequality_consumes_five_expansions() {
        let source_id = source_id(169);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("four_edge_local_object_mode_reserved_variable_inequality"),
        );
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseFourEdgeObjectModeInequality", SymbolKind::Mode),
                ("InnerFourEdgeObjectModeInequality", SymbolKind::Mode),
                ("MiddleFourEdgeObjectModeInequality", SymbolKind::Mode),
                ("OuterFourEdgeObjectModeInequality", SymbolKind::Mode),
                ("TooDeepFourEdgeObjectModeInequality", SymbolKind::Mode),
                ("ExtraFourEdgeObjectModeInequality", SymbolKind::Mode),
            ],
        );
        let theorem = IdentifierBinaryTheoremSpec {
            status: None,
            label: "FourEdgeLocalObjectModeReservedVariableInequalityPayloadBoundary",
            left: "z",
            operator: "<>",
            right: "z",
            recovered_label: false,
        };
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    "BaseFourEdgeObjectModeInequality",
                    "BaseFourEdgeObjectModeInequalityDef",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition_with_label(
                    "InnerFourEdgeObjectModeInequality",
                    "InnerFourEdgeObjectModeInequalityDef",
                    ReserveTypeShape::QualifiedSymbol("BaseFourEdgeObjectModeInequality"),
                ),
                mode_definition_with_label(
                    "MiddleFourEdgeObjectModeInequality",
                    "MiddleFourEdgeObjectModeInequalityDef",
                    ReserveTypeShape::QualifiedSymbol("InnerFourEdgeObjectModeInequality"),
                ),
                mode_definition_with_label(
                    "OuterFourEdgeObjectModeInequality",
                    "OuterFourEdgeObjectModeInequalityDef",
                    ReserveTypeShape::QualifiedSymbol("MiddleFourEdgeObjectModeInequality"),
                ),
                mode_definition_with_label(
                    "TooDeepFourEdgeObjectModeInequality",
                    "TooDeepFourEdgeObjectModeInequalityDef",
                    ReserveTypeShape::QualifiedSymbol("OuterFourEdgeObjectModeInequality"),
                ),
            ]
        };
        let reserve = || {
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("TooDeepFourEdgeObjectModeInequality"),
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
        let payload = extract_source_four_edge_local_object_mode_reserved_variable_inequality(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact four-edge local-object-mode reserved-variable inequality should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 5);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(payload.reserve.bridge.bindings()[0].spelling, "z");
        assert_eq!(
            payload.reserve.bridge.bindings()[0].type_spelling,
            "TooDeepFourEdgeObjectModeInequality"
        );
        assert_eq!(payload.left_lookup_ordinal, 1);
        assert_eq!(payload.right_lookup_ordinal, 2);

        let output = source_four_edge_local_object_mode_reserved_variable_inequality_output(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("exact four-edge local-object-mode inequality should reach TermFormulaChecker");
        assert_source_reserved_variable_formula_output(&output)
            .expect("four-edge local-object-mode inequality invariants should hold");
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
            assert_eq!(input.spelling, "TooDeepFourEdgeObjectModeInequality");
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
                source_mode_symbol_spelling(symbol) == Some("BaseFourEdgeObjectModeInequality")
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
            .expect("inequality formula should be checked");
        assert_eq!(formula.kind, FormulaKind::Inequality);
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
                TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY
                    .to_owned(),
            ]
        };
        let mut missing_left_expected =
            source_four_edge_local_object_mode_reserved_variable_inequality_output(
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
            source_four_edge_local_object_mode_reserved_variable_inequality_output(
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
            "BaseFourEdgeObjectModeInequality",
            "InnerFourEdgeObjectModeInequality",
            "MiddleFourEdgeObjectModeInequality",
            "OuterFourEdgeObjectModeInequality",
            "TooDeepFourEdgeObjectModeInequality",
        ] {
            let mut invalid =
                source_four_edge_local_object_mode_reserved_variable_inequality_output(
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
                    TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY
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
            "BaseFourEdgeObjectModeInequality",
            "InnerFourEdgeObjectModeInequality",
            "MiddleFourEdgeObjectModeInequality",
            "OuterFourEdgeObjectModeInequality",
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
                ReserveTypeShape::QualifiedSymbol("ExtraFourEdgeObjectModeInequality");
            assert_extraction_gap(wrong_radix);
        }

        let mut duplicate = exact_modes();
        duplicate.push(duplicate[0]);
        assert_extraction_gap(duplicate);

        let mut attributed_terminal = exact_modes();
        attributed_terminal[0].rhs_shape = ReserveTypeShape::AttributedSet;
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
            ReserveTypeShape::QualifiedSymbol("BaseFourEdgeObjectModeInequality");
        assert_extraction_gap(one_edge_outermost_radix);

        let mut two_edge_outermost_radix = exact_modes();
        two_edge_outermost_radix[4].rhs_shape =
            ReserveTypeShape::QualifiedSymbol("InnerFourEdgeObjectModeInequality");
        assert_extraction_gap(two_edge_outermost_radix);

        let mut three_edge_outermost_radix = exact_modes();
        three_edge_outermost_radix[4].rhs_shape =
            ReserveTypeShape::QualifiedSymbol("MiddleFourEdgeObjectModeInequality");
        assert_extraction_gap(three_edge_outermost_radix);

        let mut cyclic = exact_modes();
        cyclic[0].rhs_shape =
            ReserveTypeShape::QualifiedSymbol("TooDeepFourEdgeObjectModeInequality");
        assert_extraction_gap(cyclic);

        assert_extraction_gap(vec![
            exact_modes()[0],
            mode_definition_with_label(
                "ExtraFourEdgeObjectModeInequality",
                "ExtraFourEdgeObjectModeInequalityDef",
                ReserveTypeShape::QualifiedSymbol("BaseFourEdgeObjectModeInequality"),
            ),
            mode_definition_with_label(
                "InnerFourEdgeObjectModeInequality",
                "InnerFourEdgeObjectModeInequalityDef",
                ReserveTypeShape::QualifiedSymbol("ExtraFourEdgeObjectModeInequality"),
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
                    ReserveTypeShape::QualifiedSymbolWithArgs(
                        "TooDeepFourEdgeObjectModeInequality",
                    ),
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
    fn active_four_edge_local_object_mode_reserved_variable_inequality_fixture_consumes_five_expansions()
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
                    == "pass_type_elaboration_four_edge_local_object_mode_reserved_variable_inequality_001"
            })
            .expect("Task 169 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 169 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 169 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_four_edge_local_object_mode_reserved_variable_inequality_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 169 real AST should reach the four-edge local-object-mode inequality seam");
        assert_source_reserved_variable_formula_output(&output)
            .expect("Task 169 real AST should preserve every checked payload invariant");
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
                source_mode_symbol_spelling(symbol) == Some("BaseFourEdgeObjectModeInequality")
            })
            .map(|(_, expansion)| &expansion.radix)
            .expect("Task 169 base terminal expansion should exist");
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 169 normalized object type should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_eq!(normalized.source.range, terminal.source_range);
        assert_eq!(normalized.source.spelling, terminal.spelling);
    }
    #[test]
    fn source_three_edge_local_object_mode_two_hop_asserted_head_consumes_four_expansions() {
        let source_id = source_id(214);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("three_edge_local_object_mode_two_hop_asserted_head"),
        );
        let base_mode = "BaseThreeEdgeObjectModeTwoHopAssertedHead";
        let inner_mode = "InnerThreeEdgeObjectModeTwoHopAssertedHead";
        let middle_mode = "MiddleThreeEdgeObjectModeTwoHopAssertedHead";
        let outer_mode = "OuterThreeEdgeObjectModeTwoHopAssertedHead";
        let deeper_mode = "DeeperThreeEdgeObjectModeTwoHopAssertedHead";
        let all_modes = [base_mode, inner_mode, middle_mode, outer_mode];
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                (base_mode, SymbolKind::Mode),
                (inner_mode, SymbolKind::Mode),
                (middle_mode, SymbolKind::Mode),
                (outer_mode, SymbolKind::Mode),
                (deeper_mode, SymbolKind::Mode),
                (
                    "OtherThreeEdgeObjectModeTwoHopAssertedHead",
                    SymbolKind::Mode,
                ),
            ],
        );
        let theorem = exact_three_edge_local_object_mode_two_hop_asserted_head_spec();
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    base_mode,
                    "BaseThreeEdgeObjectModeTwoHopAssertedHeadDef",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition_with_label(
                    inner_mode,
                    "InnerThreeEdgeObjectModeTwoHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(base_mode),
                ),
                mode_definition_with_label(
                    middle_mode,
                    "MiddleThreeEdgeObjectModeTwoHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(inner_mode),
                ),
                mode_definition_with_label(
                    outer_mode,
                    "OuterThreeEdgeObjectModeTwoHopAssertedHeadDef",
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
        let payload = extract_source_three_edge_local_object_mode_two_hop_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("the exact object-terminal three-edge two-hop source should extract");
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
        assert_eq!(base_expansion.radix.spelling, "object");
        assert_eq!(base_expansion.radix.head, TypeHeadInput::BuiltinObject);
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

        let exact_output = || {
            source_three_edge_local_object_mode_two_hop_asserted_head_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .unwrap()
        };
        let output = exact_output();
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 214 exact checker output must satisfy every invariant");
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
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_ne!(normalized.head, TypeHeadRef::BuiltinSet);
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
                    TYPE_ELABORATION_THREE_EDGE_LOCAL_OBJECT_MODE_TWO_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
                ),
                vec![
                    TYPE_ELABORATION_THREE_EDGE_LOCAL_OBJECT_MODE_TWO_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY
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
        inner.radix.spelling = "object".to_owned();
        inner.radix.head = TypeHeadInput::BuiltinObject;
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
            .expect("mutating cloned Task 214 outputs must not mutate the exact output");

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
                0 => "object",
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
            wrong_label[index].label = Some("WrongThreeEdgeObjectModeTwoHopAssertedHeadDef");
            near_misses.push(wrong_label);
            let mut wrong_spelling = exact_modes();
            wrong_spelling[index].pattern = "OtherThreeEdgeObjectModeTwoHopAssertedHead";
            near_misses.push(wrong_spelling);
            let mut wrong_radix = exact_modes();
            wrong_radix[index].rhs_shape = if index == 0 {
                ReserveTypeShape::Builtin("set")
            } else {
                ReserveTypeShape::QualifiedSymbol("OtherThreeEdgeObjectModeTwoHopAssertedHead")
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
                ReserveTypeShape::AttributedObject
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
        broken_tail_link[1].rhs_shape = ReserveTypeShape::Builtin("object");
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
                asserted_type: ReserveTypeShape::QualifiedSymbol(middle_mode),
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
                    "OtherThreeEdgeObjectModeTwoHopAssertedHead",
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
            "DeeperThreeEdgeObjectModeTwoHopAssertedHeadDef",
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
            &[(
                "UnrelatedThreeEdgeObjectModeTwoHopAssertedHead",
                SymbolKind::Mode,
            )],
        );
        assert!(
            extract_source_three_edge_local_object_mode_two_hop_asserted_head(
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
    fn task214_synthetic_is_rejected_by_all_39_prior_type_assertion_extractors() {
        let source_id = source_id(214);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("task214_prior_extractor_isolation"),
        );
        let names = [
            "BaseThreeEdgeObjectModeTwoHopAssertedHead",
            "InnerThreeEdgeObjectModeTwoHopAssertedHead",
            "MiddleThreeEdgeObjectModeTwoHopAssertedHead",
            "OuterThreeEdgeObjectModeTwoHopAssertedHead",
        ];
        let symbols = source_local_symbols_env(
            module.clone(),
            &names.map(|spelling| (spelling, SymbolKind::Mode)),
        );
        let exact = mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            vec![
                mode_definition_with_label(
                    names[0],
                    "BaseThreeEdgeObjectModeTwoHopAssertedHeadDef",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition_with_label(
                    names[1],
                    "InnerThreeEdgeObjectModeTwoHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(names[0]),
                ),
                mode_definition_with_label(
                    names[2],
                    "MiddleThreeEdgeObjectModeTwoHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(names[1]),
                ),
                mode_definition_with_label(
                    names[3],
                    "OuterThreeEdgeObjectModeTwoHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(names[2]),
                ),
            ],
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol(names[3]),
            )],
            exact_three_edge_local_object_mode_two_hop_asserted_head_spec(),
        );
        assert!(
            extract_source_three_edge_local_object_mode_two_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_some()
        );
        let prior_results = [
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
            extract_source_three_edge_local_mode_two_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
        ];
        assert_eq!(prior_results.len(), 39);
        assert!(prior_results.into_iter().all(|result| result.is_none()));
    }

    #[test]
    fn active_four_edge_local_mode_two_hop_asserted_head_fixture_consumes_five_expansions() {
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
        let plan = build_test_plan(&config).expect("Task 215 repository plan should build");
        let (ordinal, case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| {
                case.id.0 == "pass_type_elaboration_four_edge_local_mode_two_hop_asserted_head_001"
            })
            .expect("Task 215 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 215 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 215 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_four_edge_local_mode_two_hop_asserted_head_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 215 real AST should reach the set-terminal two-link asserted-head seam");
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 215 real AST should preserve every checked payload invariant");
        assert_eq!(output.payload.reserve.mode_expansions.len(), 5);
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert_eq!(
            output.subject_result_input.spelling,
            "TooDeepFourEdgeModeTwoHopAssertedHead"
        );
        assert_eq!(
            output.asserted_type_input.spelling,
            "MiddleFourEdgeModeTwoHopAssertedHead"
        );
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
        let expansion = |spelling| {
            output
                .payload
                .reserve
                .mode_expansions
                .iter()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(spelling))
                .map(|(_, expansion)| expansion)
                .unwrap()
        };
        let too_deep = expansion("TooDeepFourEdgeModeTwoHopAssertedHead");
        let outer = expansion("OuterFourEdgeModeTwoHopAssertedHead");
        let middle = expansion("MiddleFourEdgeModeTwoHopAssertedHead");
        let inner = expansion("InnerFourEdgeModeTwoHopAssertedHead");
        let base = expansion("BaseFourEdgeModeTwoHopAssertedHead");
        assert_eq!(
            too_deep.radix.spelling,
            "OuterFourEdgeModeTwoHopAssertedHead"
        );
        assert_eq!(outer.radix.spelling, "MiddleFourEdgeModeTwoHopAssertedHead");
        assert_eq!(outer.radix.head, output.asserted_type_input.head);
        assert_eq!(middle.radix.spelling, "InnerFourEdgeModeTwoHopAssertedHead");
        assert_eq!(inner.radix.spelling, "BaseFourEdgeModeTwoHopAssertedHead");
        assert_eq!(base.radix.spelling, "set");
        assert_eq!(base.radix.head, TypeHeadInput::BuiltinSet);
        let (_, normalized) = output
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .unwrap();
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_ne!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_eq!(normalized.source.range, base.radix.source_range);
        assert_eq!(normalized.source.spelling, base.radix.spelling);
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
    fn four_edge_two_hop_asserted_head_route_rejects_all_40_prior_owner_fixtures() {
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
        let plan = build_test_plan(&config).expect("Task 215 repository plan should build");
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
        ];
        assert_eq!(prior_owner_ids.len(), 40);
        for owner_id in prior_owner_ids {
            let (ordinal, case) = active_type_elaboration_cases(&plan)
                .enumerate()
                .find(|(_, case)| case.id.0 == owner_id)
                .unwrap_or_else(|| panic!("prior owner fixture {owner_id} must be active"));
            let frontend = run_frontend(&workspace_root, case, ordinal)
                .unwrap_or_else(|error| panic!("owner fixture {owner_id} must parse: {error}"));
            assert!(frontend.diagnostics.is_empty());
            let ast = frontend
                .ast
                .unwrap_or_else(|| panic!("owner fixture {owner_id} must produce an AST"));
            let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
            assert!(resolver.detail_keys.is_empty());
            let symbols =
                augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
            assert_eq!(
                source_type_elaboration_detail_keys(&ast, resolver.module.clone(), &symbols),
                Vec::<String>::new(),
                "prior owner fixture {owner_id} must still reach its active route"
            );
            assert!(
                extract_source_four_edge_local_mode_two_hop_asserted_head(
                    &ast,
                    resolver.module,
                    &symbols,
                )
                .is_none(),
                "Task 215 must reject prior owner fixture {owner_id}"
            );
        }
    }

    #[test]
    fn source_four_edge_local_mode_two_hop_asserted_head_consumes_five_expansions() {
        let source_id = source_id(215);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("four_edge_local_mode_two_hop_asserted_head"),
        );
        let base_mode = "BaseFourEdgeModeTwoHopAssertedHead";
        let inner_mode = "InnerFourEdgeModeTwoHopAssertedHead";
        let middle_mode = "MiddleFourEdgeModeTwoHopAssertedHead";
        let outer_mode = "OuterFourEdgeModeTwoHopAssertedHead";
        let too_deep_mode = "TooDeepFourEdgeModeTwoHopAssertedHead";
        let deeper_mode = "DeeperFourEdgeModeTwoHopAssertedHead";
        let other_mode = "OtherFourEdgeModeTwoHopAssertedHead";
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
                (deeper_mode, SymbolKind::Mode),
                (other_mode, SymbolKind::Mode),
            ],
        );
        let theorem = exact_four_edge_local_mode_two_hop_asserted_head_spec();
        let exact_modes = || {
            vec![
                mode_definition_with_label(
                    base_mode,
                    "BaseFourEdgeModeTwoHopAssertedHeadDef",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition_with_label(
                    inner_mode,
                    "InnerFourEdgeModeTwoHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(base_mode),
                ),
                mode_definition_with_label(
                    middle_mode,
                    "MiddleFourEdgeModeTwoHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(inner_mode),
                ),
                mode_definition_with_label(
                    outer_mode,
                    "OuterFourEdgeModeTwoHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(middle_mode),
                ),
                mode_definition_with_label(
                    too_deep_mode,
                    "TooDeepFourEdgeModeTwoHopAssertedHeadDef",
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
        let payload = extract_source_four_edge_local_mode_two_hop_asserted_head(
            &exact,
            module.clone(),
            &symbols,
        )
        .expect("the exact set-terminal four-edge two-hop source should extract");
        assert_eq!(payload.reserve.mode_expansions.len(), 5);
        assert_eq!(payload.reserve.bridge.bindings().len(), 1);
        assert_eq!(payload.subject_lookup_ordinal, 1);
        let source_binding = &payload.reserve.bridge.bindings()[0];
        assert_eq!(source_binding.type_spelling, too_deep_mode);
        assert_eq!(payload.asserted_type.spelling, middle_mode);
        let expansion = |spelling| {
            payload
                .reserve
                .mode_expansions
                .iter()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(spelling))
                .map(|(_, expansion)| expansion)
                .unwrap()
        };
        let too_deep_expansion = expansion(too_deep_mode);
        let outer_expansion = expansion(outer_mode);
        let middle_expansion = expansion(middle_mode);
        let inner_expansion = expansion(inner_mode);
        let base_expansion = expansion(base_mode);
        assert_eq!(too_deep_expansion.radix.spelling, outer_mode);
        assert_eq!(outer_expansion.radix.spelling, middle_mode);
        assert_eq!(outer_expansion.radix.head, payload.asserted_type.head);
        assert_eq!(middle_expansion.radix.spelling, inner_mode);
        assert_eq!(inner_expansion.radix.spelling, base_mode);
        assert_eq!(base_expansion.radix.spelling, "set");
        assert_eq!(base_expansion.radix.head, TypeHeadInput::BuiltinSet);
        let TypeHeadInput::Symbol(too_deep_symbol) = &source_binding.type_head else {
            panic!("TooDeep binding must resolve to a symbol")
        };
        let TypeHeadInput::Symbol(outer_symbol) = &too_deep_expansion.radix.head else {
            panic!("TooDeep expansion must resolve to the Outer symbol")
        };
        let TypeHeadInput::Symbol(middle_symbol) = &payload.asserted_type.head else {
            panic!("asserted head must resolve to the Middle symbol")
        };
        assert_ne!(too_deep_symbol, outer_symbol);
        assert_ne!(too_deep_symbol, middle_symbol);
        assert_ne!(outer_symbol, middle_symbol);

        let exact_output = || {
            source_four_edge_local_mode_two_hop_asserted_head_output(
                &exact,
                module.clone(),
                &symbols,
            )
            .unwrap()
        };
        let output = exact_output();
        assert_source_reserved_variable_type_assertion_output(&output)
            .expect("Task 215 exact checker output must satisfy every invariant");
        assert_eq!(output.subject_binding, BindingId::new(0));
        assert_eq!(output.payload.subject_lookup_ordinal, 1);
        assert_eq!(output.payload.reserve.mode_expansions.len(), 5);
        assert_eq!(output.term_formula.type_entries().len(), 3);
        assert_eq!(output.term_formula.normalized_types().len(), 1);
        assert_eq!(output.subject_result_input.spelling, too_deep_mode);
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
                    TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_TWO_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
                ),
                vec![
                    TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_TWO_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY
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
        invalid.subject_result_input.head = invalid.asserted_type_input.head.clone();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.site = invalid.asserted_type_input.site.clone();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.subject_result_input.source_range = invalid.asserted_type_input.source_range;
        corruptions.push(invalid);
        let mut invalid = exact_output();
        invalid.asserted_type_input.spelling = outer_mode.to_owned();
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
        let (_, too_deep) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(too_deep_mode))
            .unwrap();
        too_deep.radix.spelling = middle_mode.to_owned();
        too_deep.radix.head = invalid.asserted_type_input.head.clone();
        corruptions.push(invalid);
        let mut invalid = exact_output();
        let inner_symbol = invalid
            .payload
            .reserve
            .mode_expansions
            .keys()
            .find(|symbol| source_mode_symbol_spelling(symbol) == Some(inner_mode))
            .unwrap()
            .clone();
        let (_, outer) = invalid
            .payload
            .reserve
            .mode_expansions
            .iter_mut()
            .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(outer_mode))
            .unwrap();
        outer.radix.spelling = inner_mode.to_owned();
        outer.radix.head = TypeHeadInput::Symbol(inner_symbol);
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
            .expect("mutating cloned Task 215 outputs must not mutate the exact output");

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
                            if order.iter().enumerate().any(|(index, value)| {
                                order[..index].iter().any(|prior| prior == value)
                            }) {
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
            let expected_radix = match index {
                0 => "set",
                1 => base_mode,
                2 => inner_mode,
                3 => middle_mode,
                _ => outer_mode,
            };
            let mut near_misses = Vec::new();
            let mut missing = exact_modes();
            missing.remove(index);
            near_misses.push(missing);
            let mut duplicate = exact_modes();
            duplicate.push(duplicate[index]);
            near_misses.push(duplicate);
            let mut wrong_label = exact_modes();
            wrong_label[index].label = Some("WrongFourEdgeModeTwoHopAssertedHeadDef");
            near_misses.push(wrong_label);
            let mut wrong_spelling = exact_modes();
            wrong_spelling[index].pattern = other_mode;
            near_misses.push(wrong_spelling);
            let mut wrong_radix = exact_modes();
            wrong_radix[index].rhs_shape = if index == 0 {
                ReserveTypeShape::Builtin("object")
            } else {
                ReserveTypeShape::QualifiedSymbol(other_mode)
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

        let mut broken_too_deep_link = exact_modes();
        broken_too_deep_link[4].rhs_shape = ReserveTypeShape::QualifiedSymbol(middle_mode);
        let mut broken_outer_link = exact_modes();
        broken_outer_link[3].rhs_shape = ReserveTypeShape::QualifiedSymbol(inner_mode);
        let mut broken_middle_tail = exact_modes();
        broken_middle_tail[2].rhs_shape = ReserveTypeShape::QualifiedSymbol(base_mode);
        let mut broken_inner_tail = exact_modes();
        broken_inner_tail[1].rhs_shape = ReserveTypeShape::Builtin("set");
        let mut broken_terminal = exact_modes();
        broken_terminal[0].rhs_shape = ReserveTypeShape::Builtin("object");
        for (context, modes) in [
            ("TooDeep-to-Outer relation link", broken_too_deep_link),
            ("Outer-to-Middle relation link", broken_outer_link),
            ("Middle-to-Inner terminal tail", broken_middle_tail),
            ("Inner-to-Base terminal tail", broken_inner_tail),
            ("Base-to-set terminal", broken_terminal),
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
            ReserveTypeShape::QualifiedSymbol(outer_mode),
            ReserveTypeShape::QualifiedSymbolWithArgs(too_deep_mode),
            ReserveTypeShape::AttributedQualifiedSymbol(too_deep_mode),
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
                asserted_type: ReserveTypeShape::QualifiedSymbol(too_deep_mode),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(outer_mode),
                ..theorem
            },
            IdentifierTypeAssertionTheoremSpec {
                asserted_type: ReserveTypeShape::QualifiedSymbol(inner_mode),
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
                asserted_type: ReserveTypeShape::QualifiedSymbol(other_mode),
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
        let mut connected_deeper_modes = exact_modes();
        connected_deeper_modes.push(mode_definition_with_label(
            deeper_mode,
            "DeeperFourEdgeModeTwoHopAssertedHeadDef",
            ReserveTypeShape::QualifiedSymbol(too_deep_mode),
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
            &[("UnrelatedFourEdgeModeTwoHopAssertedHead", SymbolKind::Mode)],
        );
        assert!(
            extract_source_four_edge_local_mode_two_hop_asserted_head(
                &exact,
                module.clone(),
                &unrelated_import,
            )
            .is_some()
        );
        for imported_index in 0..5 {
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
        for imports in [
            all_modes
                .iter()
                .map(|spelling| (*spelling, SymbolKind::Mode))
                .collect::<Vec<_>>(),
            all_modes
                .iter()
                .flat_map(|spelling| [(*spelling, SymbolKind::Mode), (*spelling, SymbolKind::Mode)])
                .collect::<Vec<_>>(),
        ] {
            let provenance_near_miss =
                source_local_and_imported_symbols_env(module.clone(), &[], &imports);
            assert_eq!(
                source_type_elaboration_detail_keys(&exact, module.clone(), &provenance_near_miss,),
                vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
            );
        }
    }

    #[test]
    fn task215_synthetic_is_rejected_by_all_40_prior_type_assertion_extractors() {
        let source_id = source_id(215);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("task215_prior_extractor_isolation"),
        );
        let names = [
            "BaseFourEdgeModeTwoHopAssertedHead",
            "InnerFourEdgeModeTwoHopAssertedHead",
            "MiddleFourEdgeModeTwoHopAssertedHead",
            "OuterFourEdgeModeTwoHopAssertedHead",
            "TooDeepFourEdgeModeTwoHopAssertedHead",
        ];
        let symbols = source_local_symbols_env(
            module.clone(),
            &names.map(|spelling| (spelling, SymbolKind::Mode)),
        );
        let exact = mode_then_reserve_identifier_type_assertion_theorem_ast(
            source_id,
            vec![
                mode_definition_with_label(
                    names[0],
                    "BaseFourEdgeModeTwoHopAssertedHeadDef",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition_with_label(
                    names[1],
                    "InnerFourEdgeModeTwoHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(names[0]),
                ),
                mode_definition_with_label(
                    names[2],
                    "MiddleFourEdgeModeTwoHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(names[1]),
                ),
                mode_definition_with_label(
                    names[3],
                    "OuterFourEdgeModeTwoHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(names[2]),
                ),
                mode_definition_with_label(
                    names[4],
                    "TooDeepFourEdgeModeTwoHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(names[3]),
                ),
            ],
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol(names[4]),
            )],
            exact_four_edge_local_mode_two_hop_asserted_head_spec(),
        );
        assert!(
            extract_source_four_edge_local_mode_two_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .is_some()
        );
        let prior_results = [
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
            extract_source_three_edge_local_mode_two_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
            extract_source_three_edge_local_object_mode_two_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            ),
        ];
        assert_eq!(prior_results.len(), 40);
        assert!(prior_results.into_iter().all(|result| result.is_none()));
    }

    mod task_216_four_edge_object_two_hop_asserted_head {
        use super::*;

        fn exact_four_edge_local_object_mode_two_hop_asserted_head_spec()
        -> IdentifierTypeAssertionTheoremSpec<'static> {
            IdentifierTypeAssertionTheoremSpec {
                status: None,
                label: "FourEdgeLocalObjectModeTwoHopAssertedHeadPayloadBoundary",
                subject: "x",
                asserted_type: ReserveTypeShape::QualifiedSymbol(
                    "MiddleFourEdgeObjectModeTwoHopAssertedHead",
                ),
                recovered_label: false,
                negated: false,
            }
        }

        #[test]
        fn active_four_edge_local_object_mode_two_hop_asserted_head_fixture_consumes_five_expansions()
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
            let plan = build_test_plan(&config).expect("Task 216 repository plan should build");
            let (ordinal, case) = active_type_elaboration_cases(&plan)
                .enumerate()
                .find(|(_, case)| {
                    case.id.0 == "pass_type_elaboration_four_edge_local_object_mode_two_hop_asserted_head_001"
                })
                .expect("Task 216 active fixture should be discoverable");
            let frontend = run_frontend(&workspace_root, case, ordinal)
                .expect("Task 216 fixture should run through the real frontend");
            assert!(frontend.diagnostics.is_empty());
            let ast = frontend
                .ast
                .expect("Task 216 fixture should produce an AST");
            let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
            assert!(resolver.detail_keys.is_empty());
            let symbols =
                augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
            let output = source_four_edge_local_object_mode_two_hop_asserted_head_output(
                &ast,
                resolver.module,
                &symbols,
            )
            .expect(
                "Task 216 real AST should reach the object-terminal two-link asserted-head seam",
            );
            assert_source_reserved_variable_type_assertion_output(&output)
                .expect("Task 216 real AST should preserve every checked payload invariant");
            assert_eq!(output.payload.reserve.mode_expansions.len(), 5);
            assert_eq!(output.subject_binding, BindingId::new(0));
            assert_eq!(output.payload.subject_lookup_ordinal, 1);
            assert_eq!(output.term_formula.type_entries().len(), 3);
            assert_eq!(output.term_formula.normalized_types().len(), 1);
            assert_eq!(
                output.subject_result_input.spelling,
                "TooDeepFourEdgeObjectModeTwoHopAssertedHead"
            );
            assert_eq!(
                output.asserted_type_input.spelling,
                "MiddleFourEdgeObjectModeTwoHopAssertedHead"
            );
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
            let expansion = |spelling| {
                output
                    .payload
                    .reserve
                    .mode_expansions
                    .iter()
                    .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(spelling))
                    .map(|(_, expansion)| expansion)
                    .unwrap()
            };
            let too_deep = expansion("TooDeepFourEdgeObjectModeTwoHopAssertedHead");
            let outer = expansion("OuterFourEdgeObjectModeTwoHopAssertedHead");
            let middle = expansion("MiddleFourEdgeObjectModeTwoHopAssertedHead");
            let inner = expansion("InnerFourEdgeObjectModeTwoHopAssertedHead");
            let base = expansion("BaseFourEdgeObjectModeTwoHopAssertedHead");
            assert_eq!(
                too_deep.radix.spelling,
                "OuterFourEdgeObjectModeTwoHopAssertedHead"
            );
            assert_eq!(
                outer.radix.spelling,
                "MiddleFourEdgeObjectModeTwoHopAssertedHead"
            );
            assert_eq!(outer.radix.head, output.asserted_type_input.head);
            assert_eq!(
                middle.radix.spelling,
                "InnerFourEdgeObjectModeTwoHopAssertedHead"
            );
            assert_eq!(
                inner.radix.spelling,
                "BaseFourEdgeObjectModeTwoHopAssertedHead"
            );
            assert_eq!(base.radix.spelling, "object");
            assert_eq!(base.radix.head, TypeHeadInput::BuiltinObject);
            let (_, normalized) = output
                .term_formula
                .normalized_types()
                .iter()
                .next()
                .unwrap();
            assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
            assert_ne!(normalized.head, TypeHeadRef::BuiltinSet);
            assert_eq!(normalized.source.range, base.radix.source_range);
            assert_eq!(normalized.source.spelling, base.radix.spelling);
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
        fn four_edge_object_two_hop_asserted_head_route_rejects_all_41_prior_owner_fixtures() {
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
            let plan = build_test_plan(&config).expect("Task 216 repository plan should build");
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
            ];
            assert_eq!(prior_owner_ids.len(), 41);
            for owner_id in prior_owner_ids {
                let (ordinal, case) = active_type_elaboration_cases(&plan)
                    .enumerate()
                    .find(|(_, case)| case.id.0 == owner_id)
                    .unwrap_or_else(|| panic!("prior owner fixture {owner_id} must be active"));
                let frontend = run_frontend(&workspace_root, case, ordinal)
                    .unwrap_or_else(|error| panic!("owner fixture {owner_id} must parse: {error}"));
                assert!(frontend.diagnostics.is_empty());
                let ast = frontend
                    .ast
                    .unwrap_or_else(|| panic!("owner fixture {owner_id} must produce an AST"));
                let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
                assert!(resolver.detail_keys.is_empty());
                let symbols =
                    augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
                assert_eq!(
                    source_type_elaboration_detail_keys(&ast, resolver.module.clone(), &symbols),
                    Vec::<String>::new(),
                    "prior owner fixture {owner_id} must still reach its active route"
                );
                assert!(
                    extract_source_four_edge_local_object_mode_two_hop_asserted_head(
                        &ast,
                        resolver.module,
                        &symbols,
                    )
                    .is_none(),
                    "Task 216 must reject prior owner fixture {owner_id}"
                );
            }
        }

        #[test]
        fn source_four_edge_local_object_mode_two_hop_asserted_head_consumes_five_expansions() {
            let source_id = source_id(216);
            let module = ResolverModuleId::new(
                PackageId::new("test"),
                ModulePath::new("four_edge_local_object_mode_two_hop_asserted_head"),
            );
            let base_mode = "BaseFourEdgeObjectModeTwoHopAssertedHead";
            let inner_mode = "InnerFourEdgeObjectModeTwoHopAssertedHead";
            let middle_mode = "MiddleFourEdgeObjectModeTwoHopAssertedHead";
            let outer_mode = "OuterFourEdgeObjectModeTwoHopAssertedHead";
            let too_deep_mode = "TooDeepFourEdgeObjectModeTwoHopAssertedHead";
            let deeper_mode = "DeeperFourEdgeObjectModeTwoHopAssertedHead";
            let other_mode = "OtherFourEdgeObjectModeTwoHopAssertedHead";
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
                    (deeper_mode, SymbolKind::Mode),
                    (other_mode, SymbolKind::Mode),
                ],
            );
            let theorem = exact_four_edge_local_object_mode_two_hop_asserted_head_spec();
            let exact_modes = || {
                vec![
                    mode_definition_with_label(
                        base_mode,
                        "BaseFourEdgeObjectModeTwoHopAssertedHeadDef",
                        ReserveTypeShape::Builtin("object"),
                    ),
                    mode_definition_with_label(
                        inner_mode,
                        "InnerFourEdgeObjectModeTwoHopAssertedHeadDef",
                        ReserveTypeShape::QualifiedSymbol(base_mode),
                    ),
                    mode_definition_with_label(
                        middle_mode,
                        "MiddleFourEdgeObjectModeTwoHopAssertedHeadDef",
                        ReserveTypeShape::QualifiedSymbol(inner_mode),
                    ),
                    mode_definition_with_label(
                        outer_mode,
                        "OuterFourEdgeObjectModeTwoHopAssertedHeadDef",
                        ReserveTypeShape::QualifiedSymbol(middle_mode),
                    ),
                    mode_definition_with_label(
                        too_deep_mode,
                        "TooDeepFourEdgeObjectModeTwoHopAssertedHeadDef",
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
            let payload = extract_source_four_edge_local_object_mode_two_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("the exact object-terminal four-edge two-hop source should extract");
            assert_eq!(payload.reserve.mode_expansions.len(), 5);
            assert_eq!(payload.reserve.bridge.bindings().len(), 1);
            assert_eq!(payload.subject_lookup_ordinal, 1);
            let source_binding = &payload.reserve.bridge.bindings()[0];
            assert_eq!(source_binding.type_spelling, too_deep_mode);
            assert_eq!(payload.asserted_type.spelling, middle_mode);
            let expansion = |spelling| {
                payload
                    .reserve
                    .mode_expansions
                    .iter()
                    .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(spelling))
                    .map(|(_, expansion)| expansion)
                    .unwrap()
            };
            let too_deep_expansion = expansion(too_deep_mode);
            let outer_expansion = expansion(outer_mode);
            let middle_expansion = expansion(middle_mode);
            let inner_expansion = expansion(inner_mode);
            let base_expansion = expansion(base_mode);
            assert_eq!(too_deep_expansion.radix.spelling, outer_mode);
            assert_eq!(outer_expansion.radix.spelling, middle_mode);
            assert_eq!(outer_expansion.radix.head, payload.asserted_type.head);
            assert_eq!(middle_expansion.radix.spelling, inner_mode);
            assert_eq!(inner_expansion.radix.spelling, base_mode);
            assert_eq!(base_expansion.radix.spelling, "object");
            assert_eq!(base_expansion.radix.head, TypeHeadInput::BuiltinObject);
            let TypeHeadInput::Symbol(too_deep_symbol) = &source_binding.type_head else {
                panic!("TooDeep binding must resolve to a symbol")
            };
            let TypeHeadInput::Symbol(outer_symbol) = &too_deep_expansion.radix.head else {
                panic!("TooDeep expansion must resolve to the Outer symbol")
            };
            let TypeHeadInput::Symbol(middle_symbol) = &payload.asserted_type.head else {
                panic!("asserted head must resolve to the Middle symbol")
            };
            assert_ne!(too_deep_symbol, outer_symbol);
            assert_ne!(too_deep_symbol, middle_symbol);
            assert_ne!(outer_symbol, middle_symbol);

            let exact_output = || {
                source_four_edge_local_object_mode_two_hop_asserted_head_output(
                    &exact,
                    module.clone(),
                    &symbols,
                )
                .unwrap()
            };
            let output = exact_output();
            assert_source_reserved_variable_type_assertion_output(&output)
                .expect("Task 216 exact checker output must satisfy every invariant");
            assert_eq!(output.subject_binding, BindingId::new(0));
            assert_eq!(output.payload.subject_lookup_ordinal, 1);
            assert_eq!(output.payload.reserve.mode_expansions.len(), 5);
            assert_eq!(output.term_formula.type_entries().len(), 3);
            assert_eq!(output.term_formula.normalized_types().len(), 1);
            assert_eq!(output.subject_result_input.spelling, too_deep_mode);
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
            let (_, normalized) = output
                .term_formula
                .normalized_types()
                .iter()
                .next()
                .unwrap();
            assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
            assert_ne!(normalized.head, TypeHeadRef::BuiltinSet);
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
                    assert_source_reserved_variable_type_assertion_output(&invalid)
                        .map(|()| invalid);
                assert_eq!(
                    source_reserved_variable_type_assertion_result_detail_keys(
                        invalid_result,
                        TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_TWO_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
                    ),
                    vec![
                        TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_TWO_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY
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
            invalid.subject_result_input.head = invalid.asserted_type_input.head.clone();
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.subject_result_input.site = invalid.asserted_type_input.site.clone();
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.subject_result_input.source_range = invalid.asserted_type_input.source_range;
            corruptions.push(invalid);
            let mut invalid = exact_output();
            invalid.asserted_type_input.spelling = outer_mode.to_owned();
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
            let (_, too_deep) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(too_deep_mode))
                .unwrap();
            too_deep.radix.spelling = middle_mode.to_owned();
            too_deep.radix.head = invalid.asserted_type_input.head.clone();
            corruptions.push(invalid);
            let mut invalid = exact_output();
            let inner_symbol = invalid
                .payload
                .reserve
                .mode_expansions
                .keys()
                .find(|symbol| source_mode_symbol_spelling(symbol) == Some(inner_mode))
                .unwrap()
                .clone();
            let (_, outer) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(outer_mode))
                .unwrap();
            outer.radix.spelling = inner_mode.to_owned();
            outer.radix.head = TypeHeadInput::Symbol(inner_symbol);
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
            inner.radix.spelling = "object".to_owned();
            inner.radix.head = TypeHeadInput::BuiltinObject;
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
                .expect("mutating cloned Task 216 outputs must not mutate the exact output");

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
                                if order.iter().enumerate().any(|(index, value)| {
                                    order[..index].iter().any(|prior| prior == value)
                                }) {
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
                                            ordered[a], ordered[b], ordered[c], ordered[d],
                                            ordered[e],
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
                let expected_radix = match index {
                    0 => "object",
                    1 => base_mode,
                    2 => inner_mode,
                    3 => middle_mode,
                    _ => outer_mode,
                };
                let mut near_misses = Vec::new();
                let mut missing = exact_modes();
                missing.remove(index);
                near_misses.push(missing);
                let mut duplicate = exact_modes();
                duplicate.push(duplicate[index]);
                near_misses.push(duplicate);
                let mut wrong_label = exact_modes();
                wrong_label[index].label = Some("WrongFourEdgeObjectModeTwoHopAssertedHeadDef");
                near_misses.push(wrong_label);
                let mut wrong_spelling = exact_modes();
                wrong_spelling[index].pattern = other_mode;
                near_misses.push(wrong_spelling);
                let mut wrong_radix = exact_modes();
                wrong_radix[index].rhs_shape = if index == 0 {
                    ReserveTypeShape::Builtin("set")
                } else {
                    ReserveTypeShape::QualifiedSymbol(other_mode)
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
                    ReserveTypeShape::AttributedObject
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

            let mut broken_too_deep_link = exact_modes();
            broken_too_deep_link[4].rhs_shape = ReserveTypeShape::QualifiedSymbol(middle_mode);
            let mut broken_outer_link = exact_modes();
            broken_outer_link[3].rhs_shape = ReserveTypeShape::QualifiedSymbol(inner_mode);
            let mut broken_middle_tail = exact_modes();
            broken_middle_tail[2].rhs_shape = ReserveTypeShape::QualifiedSymbol(base_mode);
            let mut broken_inner_tail = exact_modes();
            broken_inner_tail[1].rhs_shape = ReserveTypeShape::Builtin("object");
            let mut broken_terminal = exact_modes();
            broken_terminal[0].rhs_shape = ReserveTypeShape::Builtin("set");
            for (context, modes) in [
                ("TooDeep-to-Outer relation link", broken_too_deep_link),
                ("Outer-to-Middle relation link", broken_outer_link),
                ("Middle-to-Inner terminal tail", broken_middle_tail),
                ("Inner-to-Base terminal tail", broken_inner_tail),
                ("Base-to-object terminal", broken_terminal),
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
                ReserveTypeShape::QualifiedSymbol(outer_mode),
                ReserveTypeShape::QualifiedSymbolWithArgs(too_deep_mode),
                ReserveTypeShape::AttributedQualifiedSymbol(too_deep_mode),
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
                    asserted_type: ReserveTypeShape::QualifiedSymbol(too_deep_mode),
                    ..theorem
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(outer_mode),
                    ..theorem
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(inner_mode),
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
                    asserted_type: ReserveTypeShape::QualifiedSymbol(other_mode),
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
            let mut connected_deeper_modes = exact_modes();
            connected_deeper_modes.push(mode_definition_with_label(
                deeper_mode,
                "DeeperFourEdgeObjectModeTwoHopAssertedHeadDef",
                ReserveTypeShape::QualifiedSymbol(too_deep_mode),
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
                &[(
                    "UnrelatedFourEdgeObjectModeTwoHopAssertedHead",
                    SymbolKind::Mode,
                )],
            );
            assert!(
                extract_source_four_edge_local_object_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &unrelated_import,
                )
                .is_some()
            );
            for imported_index in 0..5 {
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
            for imports in [
                all_modes
                    .iter()
                    .map(|spelling| (*spelling, SymbolKind::Mode))
                    .collect::<Vec<_>>(),
                all_modes
                    .iter()
                    .flat_map(|spelling| {
                        [(*spelling, SymbolKind::Mode), (*spelling, SymbolKind::Mode)]
                    })
                    .collect::<Vec<_>>(),
            ] {
                let provenance_near_miss =
                    source_local_and_imported_symbols_env(module.clone(), &[], &imports);
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

        #[test]
        fn task216_synthetic_is_rejected_by_all_41_prior_type_assertion_extractors() {
            let source_id = source_id(216);
            let module = ResolverModuleId::new(
                PackageId::new("test"),
                ModulePath::new("task216_prior_extractor_isolation"),
            );
            let names = [
                "BaseFourEdgeObjectModeTwoHopAssertedHead",
                "InnerFourEdgeObjectModeTwoHopAssertedHead",
                "MiddleFourEdgeObjectModeTwoHopAssertedHead",
                "OuterFourEdgeObjectModeTwoHopAssertedHead",
                "TooDeepFourEdgeObjectModeTwoHopAssertedHead",
            ];
            let symbols = source_local_symbols_env(
                module.clone(),
                &names.map(|spelling| (spelling, SymbolKind::Mode)),
            );
            let exact = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                vec![
                    mode_definition_with_label(
                        names[0],
                        "BaseFourEdgeObjectModeTwoHopAssertedHeadDef",
                        ReserveTypeShape::Builtin("object"),
                    ),
                    mode_definition_with_label(
                        names[1],
                        "InnerFourEdgeObjectModeTwoHopAssertedHeadDef",
                        ReserveTypeShape::QualifiedSymbol(names[0]),
                    ),
                    mode_definition_with_label(
                        names[2],
                        "MiddleFourEdgeObjectModeTwoHopAssertedHeadDef",
                        ReserveTypeShape::QualifiedSymbol(names[1]),
                    ),
                    mode_definition_with_label(
                        names[3],
                        "OuterFourEdgeObjectModeTwoHopAssertedHeadDef",
                        ReserveTypeShape::QualifiedSymbol(names[2]),
                    ),
                    mode_definition_with_label(
                        names[4],
                        "TooDeepFourEdgeObjectModeTwoHopAssertedHeadDef",
                        ReserveTypeShape::QualifiedSymbol(names[3]),
                    ),
                ],
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbol(names[4]),
                )],
                exact_four_edge_local_object_mode_two_hop_asserted_head_spec(),
            );
            assert!(
                extract_source_four_edge_local_object_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                )
                .is_some()
            );
            let prior_results = [
                extract_source_reserved_variable_type_assertion(&exact, module.clone(), &symbols),
                super::super::extract_source_reserved_object_variable_type_assertion(
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
                extract_source_chained_local_mode_radix_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
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
                extract_source_three_edge_local_mode_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
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
                super::super::extract_source_four_edge_local_mode_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                super::super::extract_source_four_edge_local_object_mode_asserted_head(
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
                super::super::extract_source_local_mode_long_chain_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                super::super::extract_source_local_object_mode_long_chain_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_local_mode_long_chain_radix_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                super::super::extract_source_local_object_mode_long_chain_radix_asserted_head(
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
                extract_source_three_edge_local_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_three_edge_local_object_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_four_edge_local_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
            ];
            assert_eq!(prior_results.len(), 41);
            assert!(prior_results.into_iter().all(|result| result.is_none()));
        }
    }

    mod task_217_three_edge_three_hop_asserted_head {
        use super::*;

        const BASE: &str = "BaseThreeEdgeModeThreeHopAssertedHead";
        const INNER: &str = "InnerThreeEdgeModeThreeHopAssertedHead";
        const MIDDLE: &str = "MiddleThreeEdgeModeThreeHopAssertedHead";
        const OUTER: &str = "OuterThreeEdgeModeThreeHopAssertedHead";
        const OTHER: &str = "OtherThreeEdgeModeThreeHopAssertedHead";

        fn theorem() -> IdentifierTypeAssertionTheoremSpec<'static> {
            IdentifierTypeAssertionTheoremSpec {
                status: None,
                label: "ThreeEdgeLocalModeThreeHopAssertedHeadPayloadBoundary",
                subject: "x",
                asserted_type: ReserveTypeShape::QualifiedSymbol(BASE),
                recovered_label: false,
                negated: false,
            }
        }

        fn exact_modes() -> Vec<ModeDefinitionSpec> {
            vec![
                mode_definition_with_label(
                    BASE,
                    "BaseThreeEdgeModeThreeHopAssertedHeadDef",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition_with_label(
                    INNER,
                    "InnerThreeEdgeModeThreeHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(BASE),
                ),
                mode_definition_with_label(
                    MIDDLE,
                    "MiddleThreeEdgeModeThreeHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(INNER),
                ),
                mode_definition_with_label(
                    OUTER,
                    "OuterThreeEdgeModeThreeHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(MIDDLE),
                ),
            ]
        }

        fn reserve() -> Vec<ReserveItemSpec> {
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol(OUTER),
            )]
        }

        #[test]
        fn active_fixture_consumes_four_real_expansions() {
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
            let plan = build_test_plan(&config).expect("Task 217 repository plan should build");
            let (ordinal, case) = active_type_elaboration_cases(&plan)
                .enumerate()
                .find(|(_, case)| {
                    case.id.0
                        == "pass_type_elaboration_three_edge_local_mode_three_hop_asserted_head_001"
                })
                .expect("Task 217 active fixture should be discoverable");
            let frontend = run_frontend(&workspace_root, case, ordinal)
                .expect("Task 217 fixture should run through the real frontend");
            assert!(frontend.diagnostics.is_empty());
            let ast = frontend
                .ast
                .expect("Task 217 fixture should produce an AST");
            let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
            assert!(resolver.detail_keys.is_empty());
            let symbols =
                augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
            assert_eq!(
                source_type_elaboration_detail_keys(&ast, resolver.module.clone(), &symbols),
                Vec::<String>::new()
            );
            let output = source_three_edge_local_mode_three_hop_asserted_head_output(
                &ast,
                resolver.module,
                &symbols,
            )
            .expect("Task 217 real AST should reach the exact three-link asserted-head seam");
            assert_source_reserved_variable_type_assertion_output(&output)
                .expect("Task 217 real AST should preserve every checked payload invariant");
            assert_eq!(output.payload.reserve.mode_expansions.len(), 4);
            assert_eq!(output.subject_binding, BindingId::new(0));
            assert_eq!(output.payload.subject_lookup_ordinal, 1);
            assert_eq!(output.term_formula.type_entries().len(), 3);
            assert_eq!(output.term_formula.normalized_types().len(), 1);
            assert_eq!(output.subject_result_input.spelling, OUTER);
            assert_eq!(output.asserted_type_input.spelling, BASE);
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
            let expansion = |spelling| {
                output
                    .payload
                    .reserve
                    .mode_expansions
                    .iter()
                    .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(spelling))
                    .map(|(_, expansion)| expansion)
                    .unwrap()
            };
            let outer = expansion(OUTER);
            let middle = expansion(MIDDLE);
            let inner = expansion(INNER);
            let base = expansion(BASE);
            assert_eq!(outer.radix.spelling, MIDDLE);
            assert_eq!(middle.radix.spelling, INNER);
            assert_eq!(inner.radix.spelling, BASE);
            assert_eq!(inner.radix.head, output.asserted_type_input.head);
            assert_eq!(base.radix.spelling, "set");
            assert_eq!(base.radix.head, TypeHeadInput::BuiltinSet);
            let (_, normalized) = output
                .term_formula
                .normalized_types()
                .iter()
                .next()
                .unwrap();
            assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
            assert_eq!(normalized.source.range, base.radix.source_range);
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
        }

        #[test]
        fn synthetic_exactness_matrix_rejects_every_near_miss() {
            let source_id = source_id(217);
            let module = ResolverModuleId::new(
                PackageId::new("test"),
                ModulePath::new("three_edge_local_mode_three_hop_asserted_head"),
            );
            let all_modes = [BASE, INNER, MIDDLE, OUTER];
            let symbols = source_local_symbols_env(
                module.clone(),
                &[
                    (BASE, SymbolKind::Mode),
                    (INNER, SymbolKind::Mode),
                    (MIDDLE, SymbolKind::Mode),
                    (OUTER, SymbolKind::Mode),
                    (OTHER, SymbolKind::Mode),
                ],
            );
            let exact = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                theorem(),
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
                Vec::<String>::new()
            );
            let payload = extract_source_three_edge_local_mode_three_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("the exact set-terminal three-edge three-hop source should extract");
            assert_eq!(payload.reserve.mode_expansions.len(), 4);
            assert_eq!(payload.reserve.bridge.bindings().len(), 1);
            assert_eq!(payload.subject_lookup_ordinal, 1);
            assert_eq!(payload.reserve.bridge.bindings()[0].type_spelling, OUTER);
            assert_eq!(payload.asserted_type.spelling, BASE);

            let exact_output = || {
                source_three_edge_local_mode_three_hop_asserted_head_output(
                    &exact,
                    module.clone(),
                    &symbols,
                )
                .unwrap()
            };
            let output = exact_output();
            assert_source_reserved_variable_type_assertion_output(&output)
                .expect("Task 217 exact checker output must satisfy every invariant");

            let assert_invalid_output = |invalid| {
                let invalid_result =
                    assert_source_reserved_variable_type_assertion_output(&invalid)
                        .map(|()| invalid);
                assert_eq!(
                    source_reserved_variable_type_assertion_result_detail_keys(
                        invalid_result,
                        TYPE_ELABORATION_THREE_EDGE_LOCAL_MODE_THREE_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
                    ),
                    vec![
                        TYPE_ELABORATION_THREE_EDGE_LOCAL_MODE_THREE_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY
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
            invalid.subject_result_input.spelling = MIDDLE.to_owned();
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
            invalid.asserted_type_input.spelling = INNER.to_owned();
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
            let inner_symbol = invalid
                .payload
                .reserve
                .mode_expansions
                .keys()
                .find(|symbol| source_mode_symbol_spelling(symbol) == Some(INNER))
                .unwrap()
                .clone();
            let (_, outer) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(OUTER))
                .unwrap();
            outer.radix.spelling = INNER.to_owned();
            outer.radix.head = TypeHeadInput::Symbol(inner_symbol);
            corruptions.push(invalid);
            let mut invalid = exact_output();
            let base_symbol = invalid
                .payload
                .reserve
                .mode_expansions
                .keys()
                .find(|symbol| source_mode_symbol_spelling(symbol) == Some(BASE))
                .unwrap()
                .clone();
            let (_, middle) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(MIDDLE))
                .unwrap();
            middle.radix.spelling = BASE.to_owned();
            middle.radix.head = TypeHeadInput::Symbol(base_symbol);
            corruptions.push(invalid);
            let mut invalid = exact_output();
            let (_, inner) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(INNER))
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
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(BASE))
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
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(BASE))
                .unwrap();
            base.radix.source_range = range(source_id, 0, 1);
            corruptions.push(invalid);
            for invalid in corruptions {
                assert_invalid_output(invalid);
            }
            assert_source_reserved_variable_type_assertion_output(&exact_output())
                .expect("mutating cloned Task 217 outputs must not mutate the exact output");

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
                                    theorem(),
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
                    1 => BASE,
                    2 => INNER,
                    _ => MIDDLE,
                };
                let mut near_misses = Vec::new();
                let mut missing = exact_modes();
                missing.remove(index);
                near_misses.push(missing);
                let mut duplicate = exact_modes();
                duplicate.push(duplicate[index]);
                near_misses.push(duplicate);
                let mut wrong_label = exact_modes();
                wrong_label[index].label = Some("WrongThreeEdgeModeThreeHopAssertedHeadDef");
                near_misses.push(wrong_label);
                let mut wrong_pattern = exact_modes();
                wrong_pattern[index].pattern = OTHER;
                near_misses.push(wrong_pattern);
                let mut wrong_radix = exact_modes();
                wrong_radix[index].rhs_shape = if index == 0 {
                    ReserveTypeShape::Builtin("object")
                } else {
                    ReserveTypeShape::QualifiedSymbol(OTHER)
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
                let mut attributed = exact_modes();
                attributed[index].rhs_shape = if index == 0 {
                    ReserveTypeShape::AttributedSet
                } else {
                    ReserveTypeShape::AttributedQualifiedSymbol(expected_radix)
                };
                near_misses.push(attributed);
                for (variant, modes) in near_misses.into_iter().enumerate() {
                    assert_extraction_gap(
                        mode_then_reserve_identifier_type_assertion_theorem_ast(
                            source_id,
                            modes,
                            reserve(),
                            theorem(),
                        ),
                        &format!("definition {index} near miss {variant}"),
                    );
                }
            }

            for (context, index, wrong_radix) in [
                ("Outer-to-Middle relation link", 3, INNER),
                ("Middle-to-Inner relation link", 2, BASE),
                ("Inner-to-Base relation link", 1, MIDDLE),
            ] {
                let mut modes = exact_modes();
                modes[index].rhs_shape = ReserveTypeShape::QualifiedSymbol(wrong_radix);
                assert_extraction_gap(
                    mode_then_reserve_identifier_type_assertion_theorem_ast(
                        source_id,
                        modes,
                        reserve(),
                        theorem(),
                    ),
                    context,
                );
            }

            let mut source_near_misses = Vec::new();
            for bad_reserve in [
                ReserveTypeShape::QualifiedSymbol(BASE),
                ReserveTypeShape::QualifiedSymbol(INNER),
                ReserveTypeShape::QualifiedSymbol(MIDDLE),
                ReserveTypeShape::QualifiedSymbolWithArgs(OUTER),
                ReserveTypeShape::AttributedQualifiedSymbol(OUTER),
                ReserveTypeShape::Builtin("set"),
                ReserveTypeShape::Builtin("object"),
            ] {
                source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    exact_modes(),
                    vec![reserve_item(vec!["x"], bad_reserve)],
                    theorem(),
                ));
            }
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::QualifiedSymbol(OUTER)),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                ],
                theorem(),
            ));
            for near_miss_theorem in [
                IdentifierTypeAssertionTheoremSpec {
                    label: "OtherPayloadBoundary",
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    subject: "y",
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(INNER),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(MIDDLE),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(OUTER),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::Builtin("set"),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::Builtin("object"),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(OTHER),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbolWithArgs(BASE),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::AttributedQualifiedSymbol(BASE),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    negated: true,
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    status: Some("registration"),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    recovered_label: true,
                    ..theorem()
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
                    theorem(),
                ),
            );
            for (index, near_miss) in source_near_misses.into_iter().enumerate() {
                assert_extraction_gap(near_miss, &format!("source near miss {index}"));
            }

            let unrelated_import = source_local_and_imported_symbols_env(
                module.clone(),
                &all_modes.map(|spelling| (spelling, SymbolKind::Mode)),
                &[("UnrelatedThreeHopMode", SymbolKind::Mode)],
            );
            assert!(
                extract_source_three_edge_local_mode_three_hop_asserted_head(
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
            for imports in [
                all_modes
                    .iter()
                    .map(|spelling| (*spelling, SymbolKind::Mode))
                    .collect::<Vec<_>>(),
                all_modes
                    .iter()
                    .flat_map(|spelling| {
                        [(*spelling, SymbolKind::Mode), (*spelling, SymbolKind::Mode)]
                    })
                    .collect::<Vec<_>>(),
            ] {
                let provenance_near_miss =
                    source_local_and_imported_symbols_env(module.clone(), &[], &imports);
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

        #[test]
        fn route_rejects_all_42_prior_owner_fixtures_including_tasks_211_through_216() {
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
            let plan = build_test_plan(&config).expect("Task 217 repository plan should build");
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
            ];
            assert_eq!(prior_owner_ids.len(), 42);
            assert_eq!(
                &prior_owner_ids[36..],
                &[
                    "pass_type_elaboration_two_edge_local_mode_two_hop_asserted_head_001",
                    "pass_type_elaboration_two_edge_local_object_mode_two_hop_asserted_head_001",
                    "pass_type_elaboration_three_edge_local_mode_two_hop_asserted_head_001",
                    "pass_type_elaboration_three_edge_local_object_mode_two_hop_asserted_head_001",
                    "pass_type_elaboration_four_edge_local_mode_two_hop_asserted_head_001",
                    "pass_type_elaboration_four_edge_local_object_mode_two_hop_asserted_head_001",
                ]
            );
            for owner_id in prior_owner_ids {
                let (ordinal, case) = active_type_elaboration_cases(&plan)
                    .enumerate()
                    .find(|(_, case)| case.id.0 == owner_id)
                    .unwrap_or_else(|| panic!("prior owner fixture {owner_id} must be active"));
                let frontend = run_frontend(&workspace_root, case, ordinal)
                    .unwrap_or_else(|error| panic!("owner fixture {owner_id} must parse: {error}"));
                assert!(frontend.diagnostics.is_empty());
                let ast = frontend
                    .ast
                    .unwrap_or_else(|| panic!("owner fixture {owner_id} must produce an AST"));
                let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
                assert!(resolver.detail_keys.is_empty());
                let symbols =
                    augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
                assert_eq!(
                    source_type_elaboration_detail_keys(&ast, resolver.module.clone(), &symbols),
                    Vec::<String>::new(),
                    "prior owner fixture {owner_id} must still reach its active route"
                );
                assert!(
                    extract_source_three_edge_local_mode_three_hop_asserted_head(
                        &ast,
                        resolver.module,
                        &symbols,
                    )
                    .is_none(),
                    "Task 217 must reject prior owner fixture {owner_id}"
                );
            }
        }

        #[test]
        fn task217_synthetic_is_rejected_by_all_42_prior_type_assertion_extractors() {
            let source_id = source_id(217);
            let module = ResolverModuleId::new(
                PackageId::new("test"),
                ModulePath::new("task217_prior_extractor_isolation"),
            );
            let symbols = source_local_symbols_env(
                module.clone(),
                &[
                    (BASE, SymbolKind::Mode),
                    (INNER, SymbolKind::Mode),
                    (MIDDLE, SymbolKind::Mode),
                    (OUTER, SymbolKind::Mode),
                ],
            );
            let exact = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                theorem(),
            );
            assert!(
                extract_source_three_edge_local_mode_three_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                )
                .is_some()
            );
            let prior_results = [
                extract_source_reserved_variable_type_assertion(&exact, module.clone(), &symbols),
                super::super::extract_source_reserved_object_variable_type_assertion(
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
                extract_source_chained_local_mode_radix_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
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
                extract_source_three_edge_local_mode_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
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
                super::super::extract_source_four_edge_local_mode_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                super::super::extract_source_four_edge_local_object_mode_asserted_head(
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
                super::super::extract_source_local_mode_long_chain_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                super::super::extract_source_local_object_mode_long_chain_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_local_mode_long_chain_radix_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                super::super::extract_source_local_object_mode_long_chain_radix_asserted_head(
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
                extract_source_three_edge_local_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_three_edge_local_object_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_four_edge_local_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_four_edge_local_object_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
            ];
            assert_eq!(prior_results.len(), 42);
            assert!(prior_results.into_iter().all(|result| result.is_none()));
        }
    }
    mod task_218_three_edge_object_three_hop_asserted_head {
        use super::*;

        const BASE: &str = "BaseThreeEdgeObjectModeThreeHopAssertedHead";
        const INNER: &str = "InnerThreeEdgeObjectModeThreeHopAssertedHead";
        const MIDDLE: &str = "MiddleThreeEdgeObjectModeThreeHopAssertedHead";
        const OUTER: &str = "OuterThreeEdgeObjectModeThreeHopAssertedHead";
        const OTHER: &str = "OtherThreeEdgeObjectModeThreeHopAssertedHead";
        const DEEPER: &str = "DeeperThreeEdgeObjectModeThreeHopAssertedHead";

        fn theorem() -> IdentifierTypeAssertionTheoremSpec<'static> {
            IdentifierTypeAssertionTheoremSpec {
                status: None,
                label: "ThreeEdgeLocalObjectModeThreeHopAssertedHeadPayloadBoundary",
                subject: "x",
                asserted_type: ReserveTypeShape::QualifiedSymbol(BASE),
                recovered_label: false,
                negated: false,
            }
        }

        fn exact_modes() -> Vec<ModeDefinitionSpec> {
            vec![
                mode_definition_with_label(
                    BASE,
                    "BaseThreeEdgeObjectModeThreeHopAssertedHeadDef",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition_with_label(
                    INNER,
                    "InnerThreeEdgeObjectModeThreeHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(BASE),
                ),
                mode_definition_with_label(
                    MIDDLE,
                    "MiddleThreeEdgeObjectModeThreeHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(INNER),
                ),
                mode_definition_with_label(
                    OUTER,
                    "OuterThreeEdgeObjectModeThreeHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(MIDDLE),
                ),
            ]
        }

        fn reserve() -> Vec<ReserveItemSpec> {
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol(OUTER),
            )]
        }

        #[test]
        fn active_fixture_consumes_four_real_expansions() {
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
            let plan = build_test_plan(&config).expect("Task 218 repository plan should build");
            let (ordinal, case) = active_type_elaboration_cases(&plan)
                .enumerate()
                .find(|(_, case)| {
                    case.id.0
                        == "pass_type_elaboration_three_edge_local_object_mode_three_hop_asserted_head_001"
                })
                .expect("Task 218 active fixture should be discoverable");
            let frontend = run_frontend(&workspace_root, case, ordinal)
                .expect("Task 218 fixture should run through the real frontend");
            assert!(frontend.diagnostics.is_empty());
            let ast = frontend
                .ast
                .expect("Task 218 fixture should produce an AST");
            let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
            assert!(resolver.detail_keys.is_empty());
            let symbols =
                augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
            assert_eq!(
                source_type_elaboration_detail_keys(&ast, resolver.module.clone(), &symbols),
                Vec::<String>::new()
            );
            let output = source_three_edge_local_object_mode_three_hop_asserted_head_output(
                &ast,
                resolver.module,
                &symbols,
            )
            .expect("Task 218 real AST should reach the exact three-link asserted-head seam");
            assert_source_reserved_variable_type_assertion_output(&output)
                .expect("Task 218 real AST should preserve every checked payload invariant");
            assert_eq!(output.payload.reserve.mode_expansions.len(), 4);
            assert_eq!(output.subject_binding, BindingId::new(0));
            assert_eq!(output.payload.subject_lookup_ordinal, 1);
            assert_eq!(output.term_formula.type_entries().len(), 3);
            assert_eq!(output.term_formula.normalized_types().len(), 1);
            assert_eq!(output.subject_result_input.spelling, OUTER);
            assert_eq!(output.asserted_type_input.spelling, BASE);
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
            let expansion = |spelling| {
                output
                    .payload
                    .reserve
                    .mode_expansions
                    .iter()
                    .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(spelling))
                    .map(|(_, expansion)| expansion)
                    .unwrap()
            };
            let outer = expansion(OUTER);
            let middle = expansion(MIDDLE);
            let inner = expansion(INNER);
            let base = expansion(BASE);
            assert_eq!(outer.radix.spelling, MIDDLE);
            assert_eq!(middle.radix.spelling, INNER);
            assert_eq!(inner.radix.spelling, BASE);
            assert_eq!(inner.radix.head, output.asserted_type_input.head);
            assert_eq!(base.radix.spelling, "object");
            assert_eq!(base.radix.head, TypeHeadInput::BuiltinObject);
            let (_, normalized) = output
                .term_formula
                .normalized_types()
                .iter()
                .next()
                .unwrap();
            assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
            assert_eq!(normalized.source.range, base.radix.source_range);
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
        }

        #[test]
        fn synthetic_exactness_matrix_rejects_every_near_miss() {
            let source_id = source_id(218);
            let module = ResolverModuleId::new(
                PackageId::new("test"),
                ModulePath::new("three_edge_local_object_mode_three_hop_asserted_head"),
            );
            let all_modes = [BASE, INNER, MIDDLE, OUTER];
            let symbols = source_local_symbols_env(
                module.clone(),
                &[
                    (BASE, SymbolKind::Mode),
                    (INNER, SymbolKind::Mode),
                    (MIDDLE, SymbolKind::Mode),
                    (OUTER, SymbolKind::Mode),
                    (OTHER, SymbolKind::Mode),
                    (DEEPER, SymbolKind::Mode),
                ],
            );
            let exact = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                theorem(),
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
                Vec::<String>::new()
            );
            let payload = extract_source_three_edge_local_object_mode_three_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("the exact object-terminal three-edge three-hop source should extract");
            assert_eq!(payload.reserve.mode_expansions.len(), 4);
            assert_eq!(payload.reserve.bridge.bindings().len(), 1);
            assert_eq!(payload.subject_lookup_ordinal, 1);
            assert_eq!(payload.reserve.bridge.bindings()[0].type_spelling, OUTER);
            assert_eq!(payload.asserted_type.spelling, BASE);

            let exact_output = || {
                source_three_edge_local_object_mode_three_hop_asserted_head_output(
                    &exact,
                    module.clone(),
                    &symbols,
                )
                .unwrap()
            };
            let output = exact_output();
            assert_source_reserved_variable_type_assertion_output(&output)
                .expect("Task 218 exact checker output must satisfy every invariant");

            let assert_invalid_output = |invalid| {
                let invalid_result =
                    assert_source_reserved_variable_type_assertion_output(&invalid)
                        .map(|()| invalid);
                assert_eq!(
                    source_reserved_variable_type_assertion_result_detail_keys(
                        invalid_result,
                        TYPE_ELABORATION_THREE_EDGE_LOCAL_OBJECT_MODE_THREE_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
                    ),
                    vec![
                        TYPE_ELABORATION_THREE_EDGE_LOCAL_OBJECT_MODE_THREE_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY
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
            invalid.subject_result_input.spelling = MIDDLE.to_owned();
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
            invalid.asserted_type_input.spelling = INNER.to_owned();
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
            let inner_symbol = invalid
                .payload
                .reserve
                .mode_expansions
                .keys()
                .find(|symbol| source_mode_symbol_spelling(symbol) == Some(INNER))
                .unwrap()
                .clone();
            let (_, outer) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(OUTER))
                .unwrap();
            outer.radix.spelling = INNER.to_owned();
            outer.radix.head = TypeHeadInput::Symbol(inner_symbol);
            corruptions.push(invalid);
            let mut invalid = exact_output();
            let base_symbol = invalid
                .payload
                .reserve
                .mode_expansions
                .keys()
                .find(|symbol| source_mode_symbol_spelling(symbol) == Some(BASE))
                .unwrap()
                .clone();
            let (_, middle) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(MIDDLE))
                .unwrap();
            middle.radix.spelling = BASE.to_owned();
            middle.radix.head = TypeHeadInput::Symbol(base_symbol);
            corruptions.push(invalid);
            let mut invalid = exact_output();
            let (_, inner) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(INNER))
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
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(BASE))
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
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(BASE))
                .unwrap();
            base.radix.source_range = range(source_id, 0, 1);
            corruptions.push(invalid);
            for invalid in corruptions {
                assert_invalid_output(invalid);
            }
            assert_source_reserved_variable_type_assertion_output(&exact_output())
                .expect("mutating cloned Task 218 outputs must not mutate the exact output");

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
                                    theorem(),
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
                    0 => "object",
                    1 => BASE,
                    2 => INNER,
                    _ => MIDDLE,
                };
                let mut near_misses = Vec::new();
                let mut missing = exact_modes();
                missing.remove(index);
                near_misses.push(missing);
                let mut duplicate = exact_modes();
                duplicate.push(duplicate[index]);
                near_misses.push(duplicate);
                let mut wrong_label = exact_modes();
                wrong_label[index].label = Some("WrongThreeEdgeObjectModeThreeHopAssertedHeadDef");
                near_misses.push(wrong_label);
                let mut wrong_pattern = exact_modes();
                wrong_pattern[index].pattern = OTHER;
                near_misses.push(wrong_pattern);
                let mut wrong_radix = exact_modes();
                wrong_radix[index].rhs_shape = if index == 0 {
                    ReserveTypeShape::Builtin("set")
                } else {
                    ReserveTypeShape::QualifiedSymbol(OTHER)
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
                let mut attributed = exact_modes();
                attributed[index].rhs_shape = if index == 0 {
                    ReserveTypeShape::AttributedObject
                } else {
                    ReserveTypeShape::AttributedQualifiedSymbol(expected_radix)
                };
                near_misses.push(attributed);
                for (variant, modes) in near_misses.into_iter().enumerate() {
                    assert_extraction_gap(
                        mode_then_reserve_identifier_type_assertion_theorem_ast(
                            source_id,
                            modes,
                            reserve(),
                            theorem(),
                        ),
                        &format!("definition {index} near miss {variant}"),
                    );
                }
            }

            for (context, index, wrong_radix) in [
                ("Outer-to-Middle relation link", 3, INNER),
                ("Middle-to-Inner relation link", 2, BASE),
                ("Inner-to-Base relation link", 1, MIDDLE),
            ] {
                let mut modes = exact_modes();
                modes[index].rhs_shape = ReserveTypeShape::QualifiedSymbol(wrong_radix);
                assert_extraction_gap(
                    mode_then_reserve_identifier_type_assertion_theorem_ast(
                        source_id,
                        modes,
                        reserve(),
                        theorem(),
                    ),
                    context,
                );
            }

            let mut source_near_misses = Vec::new();
            for bad_reserve in [
                ReserveTypeShape::QualifiedSymbol(BASE),
                ReserveTypeShape::QualifiedSymbol(INNER),
                ReserveTypeShape::QualifiedSymbol(MIDDLE),
                ReserveTypeShape::QualifiedSymbolWithArgs(OUTER),
                ReserveTypeShape::AttributedQualifiedSymbol(OUTER),
                ReserveTypeShape::Builtin("set"),
                ReserveTypeShape::Builtin("object"),
            ] {
                source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    exact_modes(),
                    vec![reserve_item(vec!["x"], bad_reserve)],
                    theorem(),
                ));
            }
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::QualifiedSymbol(OUTER)),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                ],
                theorem(),
            ));
            for near_miss_theorem in [
                IdentifierTypeAssertionTheoremSpec {
                    label: "OtherPayloadBoundary",
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    subject: "y",
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(INNER),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(MIDDLE),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(OUTER),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::Builtin("set"),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::Builtin("object"),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(OTHER),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbolWithArgs(BASE),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::AttributedQualifiedSymbol(BASE),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    negated: true,
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    status: Some("registration"),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    recovered_label: true,
                    ..theorem()
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
                DEEPER,
                "DeeperThreeEdgeObjectModeThreeHopAssertedHeadDef",
                ReserveTypeShape::QualifiedSymbol(OUTER),
            ));
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                connected_deeper_modes,
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(DEEPER),
                    ..theorem()
                },
            ));
            source_near_misses.push(
                modes_then_empty_definition_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    exact_modes(),
                    reserve(),
                    theorem(),
                ),
            );
            for (index, near_miss) in source_near_misses.into_iter().enumerate() {
                assert_extraction_gap(near_miss, &format!("source near miss {index}"));
            }

            let unrelated_import = source_local_and_imported_symbols_env(
                module.clone(),
                &all_modes.map(|spelling| (spelling, SymbolKind::Mode)),
                &[("UnrelatedThreeHopMode", SymbolKind::Mode)],
            );
            assert!(
                extract_source_three_edge_local_object_mode_three_hop_asserted_head(
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
            for imports in [
                all_modes
                    .iter()
                    .map(|spelling| (*spelling, SymbolKind::Mode))
                    .collect::<Vec<_>>(),
                all_modes
                    .iter()
                    .flat_map(|spelling| {
                        [(*spelling, SymbolKind::Mode), (*spelling, SymbolKind::Mode)]
                    })
                    .collect::<Vec<_>>(),
            ] {
                let provenance_near_miss =
                    source_local_and_imported_symbols_env(module.clone(), &[], &imports);
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

        #[test]
        fn route_rejects_all_43_prior_owner_fixtures_including_tasks_211_through_217() {
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
            let plan = build_test_plan(&config).expect("Task 218 repository plan should build");
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
            ];
            assert_eq!(prior_owner_ids.len(), 43);
            assert_eq!(
                &prior_owner_ids[36..],
                &[
                    "pass_type_elaboration_two_edge_local_mode_two_hop_asserted_head_001",
                    "pass_type_elaboration_two_edge_local_object_mode_two_hop_asserted_head_001",
                    "pass_type_elaboration_three_edge_local_mode_two_hop_asserted_head_001",
                    "pass_type_elaboration_three_edge_local_object_mode_two_hop_asserted_head_001",
                    "pass_type_elaboration_four_edge_local_mode_two_hop_asserted_head_001",
                    "pass_type_elaboration_four_edge_local_object_mode_two_hop_asserted_head_001",
                    "pass_type_elaboration_three_edge_local_mode_three_hop_asserted_head_001",
                ]
            );
            for owner_id in prior_owner_ids {
                let (ordinal, case) = active_type_elaboration_cases(&plan)
                    .enumerate()
                    .find(|(_, case)| case.id.0 == owner_id)
                    .unwrap_or_else(|| panic!("prior owner fixture {owner_id} must be active"));
                let frontend = run_frontend(&workspace_root, case, ordinal)
                    .unwrap_or_else(|error| panic!("owner fixture {owner_id} must parse: {error}"));
                assert!(frontend.diagnostics.is_empty());
                let ast = frontend
                    .ast
                    .unwrap_or_else(|| panic!("owner fixture {owner_id} must produce an AST"));
                let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
                assert!(resolver.detail_keys.is_empty());
                let symbols =
                    augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
                assert_eq!(
                    source_type_elaboration_detail_keys(&ast, resolver.module.clone(), &symbols),
                    Vec::<String>::new(),
                    "prior owner fixture {owner_id} must still reach its active route"
                );
                assert!(
                    extract_source_three_edge_local_object_mode_three_hop_asserted_head(
                        &ast,
                        resolver.module,
                        &symbols,
                    )
                    .is_none(),
                    "Task 218 must reject prior owner fixture {owner_id}"
                );
            }
        }

        #[test]
        fn task218_synthetic_is_rejected_by_all_43_prior_type_assertion_extractors() {
            let source_id = source_id(218);
            let module = ResolverModuleId::new(
                PackageId::new("test"),
                ModulePath::new("task218_prior_extractor_isolation"),
            );
            let symbols = source_local_symbols_env(
                module.clone(),
                &[
                    (BASE, SymbolKind::Mode),
                    (INNER, SymbolKind::Mode),
                    (MIDDLE, SymbolKind::Mode),
                    (OUTER, SymbolKind::Mode),
                ],
            );
            let exact = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                theorem(),
            );
            assert!(
                extract_source_three_edge_local_object_mode_three_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                )
                .is_some()
            );
            let prior_results = [
                extract_source_reserved_variable_type_assertion(&exact, module.clone(), &symbols),
                super::super::extract_source_reserved_object_variable_type_assertion(
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
                extract_source_chained_local_mode_radix_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
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
                extract_source_three_edge_local_mode_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
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
                super::super::extract_source_four_edge_local_mode_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                super::super::extract_source_four_edge_local_object_mode_asserted_head(
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
                super::super::extract_source_local_mode_long_chain_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                super::super::extract_source_local_object_mode_long_chain_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_local_mode_long_chain_radix_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                super::super::extract_source_local_object_mode_long_chain_radix_asserted_head(
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
                extract_source_three_edge_local_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_three_edge_local_object_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_four_edge_local_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_four_edge_local_object_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_three_edge_local_mode_three_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
            ];
            assert_eq!(prior_results.len(), 43);
            assert!(prior_results.into_iter().all(|result| result.is_none()));
        }
    }
    mod task_219_four_edge_three_hop_asserted_head {
        use super::*;

        const BASE: &str = "BaseFourEdgeModeThreeHopAssertedHead";
        const INNER: &str = "InnerFourEdgeModeThreeHopAssertedHead";
        const MIDDLE: &str = "MiddleFourEdgeModeThreeHopAssertedHead";
        const OUTER: &str = "OuterFourEdgeModeThreeHopAssertedHead";
        const TOO_DEEP: &str = "TooDeepFourEdgeModeThreeHopAssertedHead";
        const OTHER: &str = "OtherFourEdgeModeThreeHopAssertedHead";
        const DEEPER: &str = "DeeperFourEdgeModeThreeHopAssertedHead";

        fn theorem() -> IdentifierTypeAssertionTheoremSpec<'static> {
            IdentifierTypeAssertionTheoremSpec {
                status: None,
                label: "FourEdgeLocalModeThreeHopAssertedHeadPayloadBoundary",
                subject: "x",
                asserted_type: ReserveTypeShape::QualifiedSymbol(INNER),
                recovered_label: false,
                negated: false,
            }
        }

        fn exact_modes() -> Vec<ModeDefinitionSpec> {
            vec![
                mode_definition_with_label(
                    BASE,
                    "BaseFourEdgeModeThreeHopAssertedHeadDef",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition_with_label(
                    INNER,
                    "InnerFourEdgeModeThreeHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(BASE),
                ),
                mode_definition_with_label(
                    MIDDLE,
                    "MiddleFourEdgeModeThreeHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(INNER),
                ),
                mode_definition_with_label(
                    OUTER,
                    "OuterFourEdgeModeThreeHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(MIDDLE),
                ),
                mode_definition_with_label(
                    TOO_DEEP,
                    "TooDeepFourEdgeModeThreeHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(OUTER),
                ),
            ]
        }

        fn reserve() -> Vec<ReserveItemSpec> {
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol(TOO_DEEP),
            )]
        }

        #[test]
        fn active_fixture_consumes_five_real_expansions() {
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
            let plan = build_test_plan(&config).expect("Task 219 repository plan should build");
            let (ordinal, case) = active_type_elaboration_cases(&plan)
                .enumerate()
                .find(|(_, case)| {
                    case.id.0
                        == "pass_type_elaboration_four_edge_local_mode_three_hop_asserted_head_001"
                })
                .expect("Task 219 active fixture should be discoverable");
            let frontend = run_frontend(&workspace_root, case, ordinal)
                .expect("Task 219 fixture should run through the real frontend");
            assert!(frontend.diagnostics.is_empty());
            let ast = frontend
                .ast
                .expect("Task 219 fixture should produce an AST");
            let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
            assert!(resolver.detail_keys.is_empty());
            let symbols =
                augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
            assert_eq!(
                source_type_elaboration_detail_keys(&ast, resolver.module.clone(), &symbols),
                Vec::<String>::new()
            );
            let output = source_four_edge_local_mode_three_hop_asserted_head_output(
                &ast,
                resolver.module,
                &symbols,
            )
            .expect("Task 219 real AST should reach the exact three-link asserted-head seam");
            assert_source_reserved_variable_type_assertion_output(&output)
                .expect("Task 219 real AST should preserve every checked payload invariant");
            assert_eq!(output.payload.reserve.mode_expansions.len(), 5);
            assert_eq!(output.subject_binding, BindingId::new(0));
            assert_eq!(output.payload.subject_lookup_ordinal, 1);
            assert_eq!(output.term_formula.type_entries().len(), 3);
            assert_eq!(output.term_formula.normalized_types().len(), 1);
            assert_eq!(output.subject_result_input.spelling, TOO_DEEP);
            assert_eq!(output.asserted_type_input.spelling, INNER);
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
            let expansion = |spelling| {
                output
                    .payload
                    .reserve
                    .mode_expansions
                    .iter()
                    .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(spelling))
                    .map(|(_, expansion)| expansion)
                    .unwrap()
            };
            let too_deep = expansion(TOO_DEEP);
            let outer = expansion(OUTER);
            let middle = expansion(MIDDLE);
            let inner = expansion(INNER);
            let base = expansion(BASE);
            assert_eq!(too_deep.radix.spelling, OUTER);
            assert_eq!(outer.radix.spelling, MIDDLE);
            assert_eq!(middle.radix.spelling, INNER);
            assert_eq!(inner.radix.spelling, BASE);
            assert_eq!(middle.radix.head, output.asserted_type_input.head);
            assert_eq!(base.radix.spelling, "set");
            assert_eq!(base.radix.head, TypeHeadInput::BuiltinSet);
            let (_, normalized) = output
                .term_formula
                .normalized_types()
                .iter()
                .next()
                .unwrap();
            assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
            assert_eq!(normalized.source.range, base.radix.source_range);
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
        }

        #[test]
        fn synthetic_exactness_matrix_rejects_every_near_miss() {
            let source_id = source_id(219);
            let module = ResolverModuleId::new(
                PackageId::new("test"),
                ModulePath::new("four_edge_local_mode_three_hop_asserted_head"),
            );
            let all_modes = [BASE, INNER, MIDDLE, OUTER, TOO_DEEP];
            let symbols = source_local_symbols_env(
                module.clone(),
                &[
                    (BASE, SymbolKind::Mode),
                    (INNER, SymbolKind::Mode),
                    (MIDDLE, SymbolKind::Mode),
                    (OUTER, SymbolKind::Mode),
                    (TOO_DEEP, SymbolKind::Mode),
                    (OTHER, SymbolKind::Mode),
                    (DEEPER, SymbolKind::Mode),
                ],
            );
            let exact = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                theorem(),
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
                Vec::<String>::new()
            );
            let payload = extract_source_four_edge_local_mode_three_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("the exact set-terminal four-edge three-hop source should extract");
            assert_eq!(payload.reserve.mode_expansions.len(), 5);
            assert_eq!(payload.reserve.bridge.bindings().len(), 1);
            assert_eq!(payload.subject_lookup_ordinal, 1);
            assert_eq!(payload.reserve.bridge.bindings()[0].type_spelling, TOO_DEEP);
            assert_eq!(payload.asserted_type.spelling, INNER);

            let exact_output = || {
                source_four_edge_local_mode_three_hop_asserted_head_output(
                    &exact,
                    module.clone(),
                    &symbols,
                )
                .unwrap()
            };
            let output = exact_output();
            assert_source_reserved_variable_type_assertion_output(&output)
                .expect("Task 219 exact checker output must satisfy every invariant");

            let assert_invalid_output = |invalid| {
                let invalid_result =
                    assert_source_reserved_variable_type_assertion_output(&invalid)
                        .map(|()| invalid);
                assert_eq!(
                    source_reserved_variable_type_assertion_result_detail_keys(
                        invalid_result,
                        TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_THREE_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
                    ),
                    vec![
                        TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_THREE_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY
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
            invalid.subject_result_input.spelling = OUTER.to_owned();
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
            invalid.asserted_type_input.spelling = BASE.to_owned();
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
            let middle_symbol = invalid
                .payload
                .reserve
                .mode_expansions
                .keys()
                .find(|symbol| source_mode_symbol_spelling(symbol) == Some(MIDDLE))
                .unwrap()
                .clone();
            let (_, too_deep) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(TOO_DEEP))
                .unwrap();
            too_deep.radix.spelling = MIDDLE.to_owned();
            too_deep.radix.head = TypeHeadInput::Symbol(middle_symbol);
            corruptions.push(invalid);
            let mut invalid = exact_output();
            let inner_symbol = invalid
                .payload
                .reserve
                .mode_expansions
                .keys()
                .find(|symbol| source_mode_symbol_spelling(symbol) == Some(INNER))
                .unwrap()
                .clone();
            let (_, outer) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(OUTER))
                .unwrap();
            outer.radix.spelling = INNER.to_owned();
            outer.radix.head = TypeHeadInput::Symbol(inner_symbol);
            corruptions.push(invalid);
            let mut invalid = exact_output();
            let base_symbol = invalid
                .payload
                .reserve
                .mode_expansions
                .keys()
                .find(|symbol| source_mode_symbol_spelling(symbol) == Some(BASE))
                .unwrap()
                .clone();
            let (_, middle) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(MIDDLE))
                .unwrap();
            middle.radix.spelling = BASE.to_owned();
            middle.radix.head = TypeHeadInput::Symbol(base_symbol);
            corruptions.push(invalid);
            let mut invalid = exact_output();
            let (_, inner) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(INNER))
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
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(BASE))
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
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(BASE))
                .unwrap();
            base.radix.source_range = range(source_id, 0, 1);
            corruptions.push(invalid);
            for invalid in corruptions {
                assert_invalid_output(invalid);
            }
            assert_source_reserved_variable_type_assertion_output(&exact_output())
                .expect("mutating cloned Task 219 outputs must not mutate the exact output");

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
                                    .any(|(index, value)| order[index + 1..].contains(value))
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
                                            ordered[a], ordered[b], ordered[c], ordered[d],
                                            ordered[e],
                                        ],
                                        reserve(),
                                        theorem(),
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
                let expected_radix = match index {
                    0 => "set",
                    1 => BASE,
                    2 => INNER,
                    3 => MIDDLE,
                    _ => OUTER,
                };
                let mut near_misses = Vec::new();
                let mut missing = exact_modes();
                missing.remove(index);
                near_misses.push(missing);
                let mut duplicate = exact_modes();
                duplicate.push(duplicate[index]);
                near_misses.push(duplicate);
                let mut wrong_label = exact_modes();
                wrong_label[index].label = Some("WrongFourEdgeModeThreeHopAssertedHeadDef");
                near_misses.push(wrong_label);
                let mut wrong_pattern = exact_modes();
                wrong_pattern[index].pattern = OTHER;
                near_misses.push(wrong_pattern);
                let mut wrong_radix = exact_modes();
                wrong_radix[index].rhs_shape = if index == 0 {
                    ReserveTypeShape::Builtin("object")
                } else {
                    ReserveTypeShape::QualifiedSymbol(OTHER)
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
                let mut attributed = exact_modes();
                attributed[index].rhs_shape = if index == 0 {
                    ReserveTypeShape::AttributedSet
                } else {
                    ReserveTypeShape::AttributedQualifiedSymbol(expected_radix)
                };
                near_misses.push(attributed);
                for (variant, modes) in near_misses.into_iter().enumerate() {
                    assert_extraction_gap(
                        mode_then_reserve_identifier_type_assertion_theorem_ast(
                            source_id,
                            modes,
                            reserve(),
                            theorem(),
                        ),
                        &format!("definition {index} near miss {variant}"),
                    );
                }
            }

            for (context, index, wrong_radix) in [
                ("TooDeep-to-Outer relation link", 4, MIDDLE),
                ("Outer-to-Middle relation link", 3, INNER),
                ("Middle-to-Inner relation link", 2, BASE),
                ("Inner-to-Base terminal-normalization link", 1, MIDDLE),
            ] {
                let mut modes = exact_modes();
                modes[index].rhs_shape = ReserveTypeShape::QualifiedSymbol(wrong_radix);
                assert_extraction_gap(
                    mode_then_reserve_identifier_type_assertion_theorem_ast(
                        source_id,
                        modes,
                        reserve(),
                        theorem(),
                    ),
                    context,
                );
            }

            let mut source_near_misses = Vec::new();
            for bad_reserve in [
                ReserveTypeShape::QualifiedSymbol(BASE),
                ReserveTypeShape::QualifiedSymbol(INNER),
                ReserveTypeShape::QualifiedSymbol(MIDDLE),
                ReserveTypeShape::QualifiedSymbol(OUTER),
                ReserveTypeShape::QualifiedSymbolWithArgs(TOO_DEEP),
                ReserveTypeShape::AttributedQualifiedSymbol(TOO_DEEP),
                ReserveTypeShape::Builtin("set"),
                ReserveTypeShape::Builtin("object"),
            ] {
                source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    exact_modes(),
                    vec![reserve_item(vec!["x"], bad_reserve)],
                    theorem(),
                ));
            }
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::QualifiedSymbol(TOO_DEEP)),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                ],
                theorem(),
            ));
            for near_miss_theorem in [
                IdentifierTypeAssertionTheoremSpec {
                    label: "OtherPayloadBoundary",
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    subject: "y",
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(TOO_DEEP),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(OUTER),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(MIDDLE),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(BASE),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::Builtin("set"),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::Builtin("object"),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(OTHER),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(DEEPER),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbolWithArgs(INNER),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::AttributedQualifiedSymbol(INNER),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    negated: true,
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    status: Some("registration"),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    recovered_label: true,
                    ..theorem()
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
                DEEPER,
                "DeeperFourEdgeModeThreeHopAssertedHeadDef",
                ReserveTypeShape::QualifiedSymbol(TOO_DEEP),
            ));
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                connected_deeper_modes,
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(DEEPER),
                    ..theorem()
                },
            ));
            source_near_misses.push(
                modes_then_empty_definition_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    exact_modes(),
                    reserve(),
                    theorem(),
                ),
            );
            for (index, near_miss) in source_near_misses.into_iter().enumerate() {
                assert_extraction_gap(near_miss, &format!("source near miss {index}"));
            }

            let unrelated_import = source_local_and_imported_symbols_env(
                module.clone(),
                &all_modes.map(|spelling| (spelling, SymbolKind::Mode)),
                &[("UnrelatedThreeHopMode", SymbolKind::Mode)],
            );
            assert!(
                extract_source_four_edge_local_mode_three_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &unrelated_import,
                )
                .is_some()
            );
            for imported_index in 0..5 {
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
            for imports in [
                all_modes
                    .iter()
                    .map(|spelling| (*spelling, SymbolKind::Mode))
                    .collect::<Vec<_>>(),
                all_modes
                    .iter()
                    .flat_map(|spelling| {
                        [(*spelling, SymbolKind::Mode), (*spelling, SymbolKind::Mode)]
                    })
                    .collect::<Vec<_>>(),
            ] {
                let provenance_near_miss =
                    source_local_and_imported_symbols_env(module.clone(), &[], &imports);
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

        #[test]
        fn route_rejects_all_44_prior_owner_fixtures_including_tasks_211_through_218() {
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
            let plan = build_test_plan(&config).expect("Task 219 repository plan should build");
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
            ];
            assert_eq!(prior_owner_ids.len(), 44);
            assert_eq!(
                &prior_owner_ids[36..],
                &[
                    "pass_type_elaboration_two_edge_local_mode_two_hop_asserted_head_001",
                    "pass_type_elaboration_two_edge_local_object_mode_two_hop_asserted_head_001",
                    "pass_type_elaboration_three_edge_local_mode_two_hop_asserted_head_001",
                    "pass_type_elaboration_three_edge_local_object_mode_two_hop_asserted_head_001",
                    "pass_type_elaboration_four_edge_local_mode_two_hop_asserted_head_001",
                    "pass_type_elaboration_four_edge_local_object_mode_two_hop_asserted_head_001",
                    "pass_type_elaboration_three_edge_local_mode_three_hop_asserted_head_001",
                    "pass_type_elaboration_three_edge_local_object_mode_three_hop_asserted_head_001",
                ]
            );
            for owner_id in prior_owner_ids {
                let (ordinal, case) = active_type_elaboration_cases(&plan)
                    .enumerate()
                    .find(|(_, case)| case.id.0 == owner_id)
                    .unwrap_or_else(|| panic!("prior owner fixture {owner_id} must be active"));
                let frontend = run_frontend(&workspace_root, case, ordinal)
                    .unwrap_or_else(|error| panic!("owner fixture {owner_id} must parse: {error}"));
                assert!(frontend.diagnostics.is_empty());
                let ast = frontend
                    .ast
                    .unwrap_or_else(|| panic!("owner fixture {owner_id} must produce an AST"));
                let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
                assert!(resolver.detail_keys.is_empty());
                let symbols =
                    augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
                assert_eq!(
                    source_type_elaboration_detail_keys(&ast, resolver.module.clone(), &symbols),
                    Vec::<String>::new(),
                    "prior owner fixture {owner_id} must still reach its active route"
                );
                assert!(
                    extract_source_four_edge_local_mode_three_hop_asserted_head(
                        &ast,
                        resolver.module,
                        &symbols,
                    )
                    .is_none(),
                    "Task 219 must reject prior owner fixture {owner_id}"
                );
            }
        }

        #[test]
        fn task219_synthetic_is_rejected_by_all_44_prior_type_assertion_extractors() {
            let source_id = source_id(219);
            let module = ResolverModuleId::new(
                PackageId::new("test"),
                ModulePath::new("task219_prior_extractor_isolation"),
            );
            let symbols = source_local_symbols_env(
                module.clone(),
                &[
                    (BASE, SymbolKind::Mode),
                    (INNER, SymbolKind::Mode),
                    (MIDDLE, SymbolKind::Mode),
                    (OUTER, SymbolKind::Mode),
                    (TOO_DEEP, SymbolKind::Mode),
                ],
            );
            let exact = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                theorem(),
            );
            assert!(
                extract_source_four_edge_local_mode_three_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                )
                .is_some()
            );
            let prior_results = [
                extract_source_reserved_variable_type_assertion(&exact, module.clone(), &symbols),
                super::super::extract_source_reserved_object_variable_type_assertion(
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
                extract_source_chained_local_mode_radix_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
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
                extract_source_three_edge_local_mode_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
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
                super::super::extract_source_four_edge_local_mode_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                super::super::extract_source_four_edge_local_object_mode_asserted_head(
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
                super::super::extract_source_local_mode_long_chain_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                super::super::extract_source_local_object_mode_long_chain_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_local_mode_long_chain_radix_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                super::super::extract_source_local_object_mode_long_chain_radix_asserted_head(
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
                extract_source_three_edge_local_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_three_edge_local_object_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_four_edge_local_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_four_edge_local_object_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_three_edge_local_mode_three_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_three_edge_local_object_mode_three_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
            ];
            assert_eq!(prior_results.len(), 44);
            assert!(prior_results.into_iter().all(|result| result.is_none()));
        }
    }
    mod task_220_four_edge_object_three_hop_asserted_head {
        use super::*;

        const BASE: &str = "BaseFourEdgeObjectModeThreeHopAssertedHead";
        const INNER: &str = "InnerFourEdgeObjectModeThreeHopAssertedHead";
        const MIDDLE: &str = "MiddleFourEdgeObjectModeThreeHopAssertedHead";
        const OUTER: &str = "OuterFourEdgeObjectModeThreeHopAssertedHead";
        const TOO_DEEP: &str = "TooDeepFourEdgeObjectModeThreeHopAssertedHead";
        const OTHER: &str = "OtherFourEdgeObjectModeThreeHopAssertedHead";
        const DEEPER: &str = "DeeperFourEdgeObjectModeThreeHopAssertedHead";

        fn theorem() -> IdentifierTypeAssertionTheoremSpec<'static> {
            IdentifierTypeAssertionTheoremSpec {
                status: None,
                label: "FourEdgeLocalObjectModeThreeHopAssertedHeadPayloadBoundary",
                subject: "x",
                asserted_type: ReserveTypeShape::QualifiedSymbol(INNER),
                recovered_label: false,
                negated: false,
            }
        }

        fn exact_modes() -> Vec<ModeDefinitionSpec> {
            vec![
                mode_definition_with_label(
                    BASE,
                    "BaseFourEdgeObjectModeThreeHopAssertedHeadDef",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition_with_label(
                    INNER,
                    "InnerFourEdgeObjectModeThreeHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(BASE),
                ),
                mode_definition_with_label(
                    MIDDLE,
                    "MiddleFourEdgeObjectModeThreeHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(INNER),
                ),
                mode_definition_with_label(
                    OUTER,
                    "OuterFourEdgeObjectModeThreeHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(MIDDLE),
                ),
                mode_definition_with_label(
                    TOO_DEEP,
                    "TooDeepFourEdgeObjectModeThreeHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(OUTER),
                ),
            ]
        }

        fn reserve() -> Vec<ReserveItemSpec> {
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol(TOO_DEEP),
            )]
        }

        #[test]
        fn active_fixture_consumes_five_real_object_expansions() {
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
            let plan = build_test_plan(&config).expect("Task 220 repository plan should build");
            let (ordinal, case) = active_type_elaboration_cases(&plan)
                .enumerate()
                .find(|(_, case)| {
                    case.id.0
                        == "pass_type_elaboration_four_edge_local_object_mode_three_hop_asserted_head_001"
                })
                .expect("Task 220 active fixture should be discoverable");
            let frontend = run_frontend(&workspace_root, case, ordinal)
                .expect("Task 220 fixture should run through the real frontend");
            assert!(frontend.diagnostics.is_empty());
            let ast = frontend
                .ast
                .expect("Task 220 fixture should produce an AST");
            let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
            assert!(resolver.detail_keys.is_empty());
            let symbols =
                augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
            assert_eq!(
                source_type_elaboration_detail_keys(&ast, resolver.module.clone(), &symbols),
                Vec::<String>::new()
            );
            let output = source_four_edge_local_object_mode_three_hop_asserted_head_output(
                &ast,
                resolver.module,
                &symbols,
            )
            .expect(
                "Task 220 real AST should reach the exact object three-link asserted-head seam",
            );
            assert_source_reserved_variable_type_assertion_output(&output)
                .expect("Task 220 real AST should preserve every checked payload invariant");
            assert_eq!(output.payload.reserve.mode_expansions.len(), 5);
            assert_eq!(output.subject_binding, BindingId::new(0));
            assert_eq!(output.payload.subject_lookup_ordinal, 1);
            assert_eq!(output.term_formula.type_entries().len(), 3);
            assert_eq!(output.term_formula.normalized_types().len(), 1);
            assert_eq!(output.subject_result_input.spelling, TOO_DEEP);
            assert_eq!(output.asserted_type_input.spelling, INNER);
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
            let expansion = |spelling| {
                output
                    .payload
                    .reserve
                    .mode_expansions
                    .iter()
                    .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(spelling))
                    .map(|(_, expansion)| expansion)
                    .unwrap()
            };
            let too_deep = expansion(TOO_DEEP);
            let outer = expansion(OUTER);
            let middle = expansion(MIDDLE);
            let inner = expansion(INNER);
            let base = expansion(BASE);
            assert_eq!(too_deep.radix.spelling, OUTER);
            assert_eq!(outer.radix.spelling, MIDDLE);
            assert_eq!(middle.radix.spelling, INNER);
            assert_eq!(inner.radix.spelling, BASE);
            assert_eq!(middle.radix.head, output.asserted_type_input.head);
            assert_eq!(base.radix.spelling, "object");
            assert_eq!(base.radix.head, TypeHeadInput::BuiltinObject);
            let (_, normalized) = output
                .term_formula
                .normalized_types()
                .iter()
                .next()
                .unwrap();
            assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
            assert_eq!(normalized.source.range, base.radix.source_range);
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
        }

        #[test]
        fn synthetic_exactness_matrix_rejects_every_near_miss() {
            let source_id = source_id(220);
            let module = ResolverModuleId::new(
                PackageId::new("test"),
                ModulePath::new("four_edge_local_object_mode_three_hop_asserted_head"),
            );
            let all_modes = [BASE, INNER, MIDDLE, OUTER, TOO_DEEP];
            let symbols = source_local_symbols_env(
                module.clone(),
                &[
                    (BASE, SymbolKind::Mode),
                    (INNER, SymbolKind::Mode),
                    (MIDDLE, SymbolKind::Mode),
                    (OUTER, SymbolKind::Mode),
                    (TOO_DEEP, SymbolKind::Mode),
                    (OTHER, SymbolKind::Mode),
                    (DEEPER, SymbolKind::Mode),
                ],
            );
            let exact = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                theorem(),
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
                Vec::<String>::new()
            );
            let payload = extract_source_four_edge_local_object_mode_three_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("the exact object-terminal four-edge three-hop source should extract");
            assert_eq!(payload.reserve.mode_expansions.len(), 5);
            assert_eq!(payload.reserve.bridge.bindings().len(), 1);
            assert_eq!(payload.subject_lookup_ordinal, 1);
            assert_eq!(payload.reserve.bridge.bindings()[0].type_spelling, TOO_DEEP);
            assert_eq!(payload.asserted_type.spelling, INNER);

            let exact_output = || {
                source_four_edge_local_object_mode_three_hop_asserted_head_output(
                    &exact,
                    module.clone(),
                    &symbols,
                )
                .unwrap()
            };
            let output = exact_output();
            assert_source_reserved_variable_type_assertion_output(&output)
                .expect("Task 220 exact checker output must satisfy every invariant");

            let assert_invalid_output = |invalid| {
                let invalid_result =
                    assert_source_reserved_variable_type_assertion_output(&invalid)
                        .map(|()| invalid);
                assert_eq!(
                    source_reserved_variable_type_assertion_result_detail_keys(
                        invalid_result,
                        TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_THREE_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
                    ),
                    vec![
                        TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_THREE_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY
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
            invalid.subject_result_input.spelling = OUTER.to_owned();
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
            invalid.asserted_type_input.spelling = BASE.to_owned();
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
            let middle_symbol = invalid
                .payload
                .reserve
                .mode_expansions
                .keys()
                .find(|symbol| source_mode_symbol_spelling(symbol) == Some(MIDDLE))
                .unwrap()
                .clone();
            let (_, too_deep) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(TOO_DEEP))
                .unwrap();
            too_deep.radix.spelling = MIDDLE.to_owned();
            too_deep.radix.head = TypeHeadInput::Symbol(middle_symbol);
            corruptions.push(invalid);
            let mut invalid = exact_output();
            let inner_symbol = invalid
                .payload
                .reserve
                .mode_expansions
                .keys()
                .find(|symbol| source_mode_symbol_spelling(symbol) == Some(INNER))
                .unwrap()
                .clone();
            let (_, outer) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(OUTER))
                .unwrap();
            outer.radix.spelling = INNER.to_owned();
            outer.radix.head = TypeHeadInput::Symbol(inner_symbol);
            corruptions.push(invalid);
            let mut invalid = exact_output();
            let base_symbol = invalid
                .payload
                .reserve
                .mode_expansions
                .keys()
                .find(|symbol| source_mode_symbol_spelling(symbol) == Some(BASE))
                .unwrap()
                .clone();
            let (_, middle) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(MIDDLE))
                .unwrap();
            middle.radix.spelling = BASE.to_owned();
            middle.radix.head = TypeHeadInput::Symbol(base_symbol);
            corruptions.push(invalid);
            let mut invalid = exact_output();
            let (_, inner) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(INNER))
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
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(BASE))
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
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(BASE))
                .unwrap();
            base.radix.source_range = range(source_id, 0, 1);
            corruptions.push(invalid);
            for invalid in corruptions {
                assert_invalid_output(invalid);
            }
            assert_source_reserved_variable_type_assertion_output(&exact_output())
                .expect("mutating cloned Task 220 outputs must not mutate the exact output");

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
                                    .any(|(index, value)| order[index + 1..].contains(value))
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
                                            ordered[a], ordered[b], ordered[c], ordered[d],
                                            ordered[e],
                                        ],
                                        reserve(),
                                        theorem(),
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
                let expected_radix = match index {
                    0 => "object",
                    1 => BASE,
                    2 => INNER,
                    3 => MIDDLE,
                    _ => OUTER,
                };
                let mut near_misses = Vec::new();
                let mut missing = exact_modes();
                missing.remove(index);
                near_misses.push(missing);
                let mut duplicate = exact_modes();
                duplicate.push(duplicate[index]);
                near_misses.push(duplicate);
                let mut wrong_label = exact_modes();
                wrong_label[index].label = Some("WrongFourEdgeObjectModeThreeHopAssertedHeadDef");
                near_misses.push(wrong_label);
                let mut wrong_pattern = exact_modes();
                wrong_pattern[index].pattern = OTHER;
                near_misses.push(wrong_pattern);
                let mut wrong_radix = exact_modes();
                wrong_radix[index].rhs_shape = if index == 0 {
                    ReserveTypeShape::Builtin("set")
                } else {
                    ReserveTypeShape::QualifiedSymbol(OTHER)
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
                let mut attributed = exact_modes();
                attributed[index].rhs_shape = if index == 0 {
                    ReserveTypeShape::AttributedObject
                } else {
                    ReserveTypeShape::AttributedQualifiedSymbol(expected_radix)
                };
                near_misses.push(attributed);
                for (variant, modes) in near_misses.into_iter().enumerate() {
                    assert_extraction_gap(
                        mode_then_reserve_identifier_type_assertion_theorem_ast(
                            source_id,
                            modes,
                            reserve(),
                            theorem(),
                        ),
                        &format!("definition {index} near miss {variant}"),
                    );
                }
            }

            for (context, index, wrong_radix) in [
                ("TooDeep-to-Outer relation link", 4, MIDDLE),
                ("Outer-to-Middle relation link", 3, INNER),
                ("Middle-to-Inner relation link", 2, BASE),
                ("Inner-to-Base terminal-normalization link", 1, MIDDLE),
            ] {
                let mut modes = exact_modes();
                modes[index].rhs_shape = ReserveTypeShape::QualifiedSymbol(wrong_radix);
                assert_extraction_gap(
                    mode_then_reserve_identifier_type_assertion_theorem_ast(
                        source_id,
                        modes,
                        reserve(),
                        theorem(),
                    ),
                    context,
                );
            }

            let mut source_near_misses = Vec::new();
            for bad_reserve in [
                ReserveTypeShape::QualifiedSymbol(BASE),
                ReserveTypeShape::QualifiedSymbol(INNER),
                ReserveTypeShape::QualifiedSymbol(MIDDLE),
                ReserveTypeShape::QualifiedSymbol(OUTER),
                ReserveTypeShape::QualifiedSymbolWithArgs(TOO_DEEP),
                ReserveTypeShape::AttributedQualifiedSymbol(TOO_DEEP),
                ReserveTypeShape::Builtin("set"),
                ReserveTypeShape::Builtin("object"),
            ] {
                source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    exact_modes(),
                    vec![reserve_item(vec!["x"], bad_reserve)],
                    theorem(),
                ));
            }
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::QualifiedSymbol(TOO_DEEP)),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("object")),
                ],
                theorem(),
            ));
            for near_miss_theorem in [
                IdentifierTypeAssertionTheoremSpec {
                    label: "OtherPayloadBoundary",
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    subject: "y",
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(TOO_DEEP),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(OUTER),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(MIDDLE),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(BASE),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::Builtin("set"),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::Builtin("object"),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(OTHER),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(DEEPER),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbolWithArgs(INNER),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::AttributedQualifiedSymbol(INNER),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    negated: true,
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    status: Some("registration"),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    recovered_label: true,
                    ..theorem()
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
                DEEPER,
                "DeeperFourEdgeObjectModeThreeHopAssertedHeadDef",
                ReserveTypeShape::QualifiedSymbol(TOO_DEEP),
            ));
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                connected_deeper_modes,
                reserve(),
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(DEEPER),
                    ..theorem()
                },
            ));
            source_near_misses.push(
                modes_then_empty_definition_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    exact_modes(),
                    reserve(),
                    theorem(),
                ),
            );
            for (index, near_miss) in source_near_misses.into_iter().enumerate() {
                assert_extraction_gap(near_miss, &format!("source near miss {index}"));
            }

            let unrelated_import = source_local_and_imported_symbols_env(
                module.clone(),
                &all_modes.map(|spelling| (spelling, SymbolKind::Mode)),
                &[("UnrelatedThreeHopMode", SymbolKind::Mode)],
            );
            assert!(
                extract_source_four_edge_local_object_mode_three_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &unrelated_import,
                )
                .is_some()
            );
            for imported_index in 0..5 {
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
            for imports in [
                all_modes
                    .iter()
                    .map(|spelling| (*spelling, SymbolKind::Mode))
                    .collect::<Vec<_>>(),
                all_modes
                    .iter()
                    .flat_map(|spelling| {
                        [(*spelling, SymbolKind::Mode), (*spelling, SymbolKind::Mode)]
                    })
                    .collect::<Vec<_>>(),
            ] {
                let provenance_near_miss =
                    source_local_and_imported_symbols_env(module.clone(), &[], &imports);
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

        #[test]
        fn route_rejects_all_45_prior_owner_fixtures_including_task_208_and_tasks_211_through_219()
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
            let plan = build_test_plan(&config).expect("Task 220 repository plan should build");
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
            ];
            assert_eq!(prior_owner_ids.len(), 45);
            assert_eq!(
                prior_owner_ids[29],
                "pass_type_elaboration_four_edge_local_object_mode_radix_asserted_head_001"
            );
            assert_eq!(
                &prior_owner_ids[36..],
                &[
                    "pass_type_elaboration_two_edge_local_mode_two_hop_asserted_head_001",
                    "pass_type_elaboration_two_edge_local_object_mode_two_hop_asserted_head_001",
                    "pass_type_elaboration_three_edge_local_mode_two_hop_asserted_head_001",
                    "pass_type_elaboration_three_edge_local_object_mode_two_hop_asserted_head_001",
                    "pass_type_elaboration_four_edge_local_mode_two_hop_asserted_head_001",
                    "pass_type_elaboration_four_edge_local_object_mode_two_hop_asserted_head_001",
                    "pass_type_elaboration_three_edge_local_mode_three_hop_asserted_head_001",
                    "pass_type_elaboration_three_edge_local_object_mode_three_hop_asserted_head_001",
                    "pass_type_elaboration_four_edge_local_mode_three_hop_asserted_head_001",
                ]
            );
            for owner_id in prior_owner_ids {
                let (ordinal, case) = active_type_elaboration_cases(&plan)
                    .enumerate()
                    .find(|(_, case)| case.id.0 == owner_id)
                    .unwrap_or_else(|| panic!("prior owner fixture {owner_id} must be active"));
                let frontend = run_frontend(&workspace_root, case, ordinal)
                    .unwrap_or_else(|error| panic!("owner fixture {owner_id} must parse: {error}"));
                assert!(frontend.diagnostics.is_empty());
                let ast = frontend
                    .ast
                    .unwrap_or_else(|| panic!("owner fixture {owner_id} must produce an AST"));
                let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
                assert!(resolver.detail_keys.is_empty());
                let symbols =
                    augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
                assert_eq!(
                    source_type_elaboration_detail_keys(&ast, resolver.module.clone(), &symbols),
                    Vec::<String>::new(),
                    "prior owner fixture {owner_id} must still reach its active route"
                );
                assert!(
                    extract_source_four_edge_local_object_mode_three_hop_asserted_head(
                        &ast,
                        resolver.module,
                        &symbols,
                    )
                    .is_none(),
                    "Task 220 must reject prior owner fixture {owner_id}"
                );
            }
        }

        #[test]
        fn task220_synthetic_is_rejected_by_all_45_prior_type_assertion_extractors() {
            let source_id = source_id(220);
            let module = ResolverModuleId::new(
                PackageId::new("test"),
                ModulePath::new("task220_prior_extractor_isolation"),
            );
            let symbols = source_local_symbols_env(
                module.clone(),
                &[
                    (BASE, SymbolKind::Mode),
                    (INNER, SymbolKind::Mode),
                    (MIDDLE, SymbolKind::Mode),
                    (OUTER, SymbolKind::Mode),
                    (TOO_DEEP, SymbolKind::Mode),
                ],
            );
            let exact = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                theorem(),
            );
            assert!(
                extract_source_four_edge_local_object_mode_three_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                )
                .is_some()
            );
            let prior_results = [
                extract_source_reserved_variable_type_assertion(&exact, module.clone(), &symbols),
                super::super::extract_source_reserved_object_variable_type_assertion(
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
                extract_source_chained_local_mode_radix_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
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
                extract_source_three_edge_local_mode_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
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
                super::super::extract_source_four_edge_local_mode_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                super::super::extract_source_four_edge_local_object_mode_asserted_head(
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
                super::super::extract_source_local_mode_long_chain_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                super::super::extract_source_local_object_mode_long_chain_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_local_mode_long_chain_radix_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                super::super::extract_source_local_object_mode_long_chain_radix_asserted_head(
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
                extract_source_three_edge_local_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_three_edge_local_object_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_four_edge_local_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_four_edge_local_object_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_three_edge_local_mode_three_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_three_edge_local_object_mode_three_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_four_edge_local_mode_three_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
            ];
            assert_eq!(prior_results.len(), 45);
            assert!(prior_results.into_iter().all(|result| result.is_none()));
        }
    }
    mod task_221_four_edge_four_hop_asserted_head {
        use super::*;

        const BASE: &str = "BaseFourEdgeModeFourHopAssertedHead";
        const INNER: &str = "InnerFourEdgeModeFourHopAssertedHead";
        const MIDDLE: &str = "MiddleFourEdgeModeFourHopAssertedHead";
        const OUTER: &str = "OuterFourEdgeModeFourHopAssertedHead";
        const TOO_DEEP: &str = "TooDeepFourEdgeModeFourHopAssertedHead";
        const OTHER: &str = "OtherFourEdgeModeFourHopAssertedHead";
        const DEEPER: &str = "DeeperFourEdgeModeFourHopAssertedHead";

        fn theorem() -> IdentifierTypeAssertionTheoremSpec<'static> {
            IdentifierTypeAssertionTheoremSpec {
                status: None,
                label: "FourEdgeLocalModeFourHopAssertedHeadPayloadBoundary",
                subject: "x",
                asserted_type: ReserveTypeShape::QualifiedSymbol(BASE),
                recovered_label: false,
                negated: false,
            }
        }

        fn exact_modes() -> Vec<ModeDefinitionSpec> {
            vec![
                mode_definition_with_label(
                    BASE,
                    "BaseFourEdgeModeFourHopAssertedHeadDef",
                    ReserveTypeShape::Builtin("set"),
                ),
                mode_definition_with_label(
                    INNER,
                    "InnerFourEdgeModeFourHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(BASE),
                ),
                mode_definition_with_label(
                    MIDDLE,
                    "MiddleFourEdgeModeFourHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(INNER),
                ),
                mode_definition_with_label(
                    OUTER,
                    "OuterFourEdgeModeFourHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(MIDDLE),
                ),
                mode_definition_with_label(
                    TOO_DEEP,
                    "TooDeepFourEdgeModeFourHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(OUTER),
                ),
            ]
        }

        fn reserve() -> Vec<ReserveItemSpec> {
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol(TOO_DEEP),
            )]
        }

        #[test]
        fn active_fixture_consumes_five_real_expansions() {
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
            let plan = build_test_plan(&config).expect("Task 221 repository plan should build");
            let (ordinal, case) = active_type_elaboration_cases(&plan)
                .enumerate()
                .find(|(_, case)| {
                    case.id.0
                        == "pass_type_elaboration_four_edge_local_mode_four_hop_asserted_head_001"
                })
                .expect("Task 221 active fixture should be discoverable");
            let frontend = run_frontend(&workspace_root, case, ordinal)
                .expect("Task 221 fixture should run through the real frontend");
            assert!(frontend.diagnostics.is_empty());
            let ast = frontend
                .ast
                .expect("Task 221 fixture should produce an AST");
            let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
            assert!(resolver.detail_keys.is_empty());
            let symbols =
                augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
            assert_eq!(
                source_type_elaboration_detail_keys(&ast, resolver.module.clone(), &symbols),
                Vec::<String>::new()
            );
            let output = source_four_edge_local_mode_four_hop_asserted_head_output(
                &ast,
                resolver.module,
                &symbols,
            )
            .expect("Task 221 real AST should reach the exact four-link asserted-head seam");
            assert_source_reserved_variable_type_assertion_output(&output)
                .expect("Task 221 real AST should preserve every checked payload invariant");
            assert_eq!(output.payload.reserve.mode_expansions.len(), 5);
            assert_eq!(output.subject_binding, BindingId::new(0));
            assert_eq!(output.payload.subject_lookup_ordinal, 1);
            assert_eq!(output.term_formula.type_entries().len(), 3);
            assert_eq!(output.term_formula.normalized_types().len(), 1);
            assert_eq!(output.subject_result_input.spelling, TOO_DEEP);
            assert_eq!(output.asserted_type_input.spelling, BASE);
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
            let expansion = |spelling| {
                output
                    .payload
                    .reserve
                    .mode_expansions
                    .iter()
                    .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(spelling))
                    .map(|(_, expansion)| expansion)
                    .unwrap()
            };
            let too_deep = expansion(TOO_DEEP);
            let outer = expansion(OUTER);
            let middle = expansion(MIDDLE);
            let inner = expansion(INNER);
            let base = expansion(BASE);
            assert_eq!(too_deep.radix.spelling, OUTER);
            assert_eq!(outer.radix.spelling, MIDDLE);
            assert_eq!(middle.radix.spelling, INNER);
            assert_eq!(inner.radix.spelling, BASE);
            assert_eq!(inner.radix.head, output.asserted_type_input.head);
            assert_eq!(base.radix.spelling, "set");
            assert_eq!(base.radix.head, TypeHeadInput::BuiltinSet);
            let (_, normalized) = output
                .term_formula
                .normalized_types()
                .iter()
                .next()
                .unwrap();
            assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
            assert_eq!(normalized.source.range, base.radix.source_range);
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
        }

        #[test]
        fn synthetic_exactness_matrix_rejects_every_near_miss() {
            let source_id = source_id(221);
            let module = ResolverModuleId::new(
                PackageId::new("test"),
                ModulePath::new("four_edge_local_mode_four_hop_asserted_head"),
            );
            let all_modes = [BASE, INNER, MIDDLE, OUTER, TOO_DEEP];
            let symbols = source_local_symbols_env(
                module.clone(),
                &[
                    (BASE, SymbolKind::Mode),
                    (INNER, SymbolKind::Mode),
                    (MIDDLE, SymbolKind::Mode),
                    (OUTER, SymbolKind::Mode),
                    (TOO_DEEP, SymbolKind::Mode),
                    (OTHER, SymbolKind::Mode),
                    (DEEPER, SymbolKind::Mode),
                ],
            );
            let exact = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                theorem(),
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
                Vec::<String>::new()
            );
            let payload = extract_source_four_edge_local_mode_four_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("the exact set-terminal four-edge four-hop source should extract");
            assert_eq!(payload.reserve.mode_expansions.len(), 5);
            assert_eq!(payload.reserve.bridge.bindings().len(), 1);
            assert_eq!(payload.subject_lookup_ordinal, 1);
            assert_eq!(payload.reserve.bridge.bindings()[0].type_spelling, TOO_DEEP);
            assert_eq!(payload.asserted_type.spelling, BASE);

            let exact_output = || {
                source_four_edge_local_mode_four_hop_asserted_head_output(
                    &exact,
                    module.clone(),
                    &symbols,
                )
                .unwrap()
            };
            let output = exact_output();
            assert_source_reserved_variable_type_assertion_output(&output)
                .expect("Task 221 exact checker output must satisfy every invariant");

            let assert_invalid_output = |invalid| {
                let invalid_result =
                    assert_source_reserved_variable_type_assertion_output(&invalid)
                        .map(|()| invalid);
                assert_eq!(
                    source_reserved_variable_type_assertion_result_detail_keys(
                        invalid_result,
                        TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_FOUR_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
                    ),
                    vec![
                        TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_FOUR_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY
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
            invalid.subject_result_input.spelling = OUTER.to_owned();
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
            invalid.asserted_type_input.spelling = INNER.to_owned();
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
            let middle_symbol = invalid
                .payload
                .reserve
                .mode_expansions
                .keys()
                .find(|symbol| source_mode_symbol_spelling(symbol) == Some(MIDDLE))
                .unwrap()
                .clone();
            let (_, too_deep) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(TOO_DEEP))
                .unwrap();
            too_deep.radix.spelling = MIDDLE.to_owned();
            too_deep.radix.head = TypeHeadInput::Symbol(middle_symbol);
            corruptions.push(invalid);
            let mut invalid = exact_output();
            let inner_symbol = invalid
                .payload
                .reserve
                .mode_expansions
                .keys()
                .find(|symbol| source_mode_symbol_spelling(symbol) == Some(INNER))
                .unwrap()
                .clone();
            let (_, outer) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(OUTER))
                .unwrap();
            outer.radix.spelling = INNER.to_owned();
            outer.radix.head = TypeHeadInput::Symbol(inner_symbol);
            corruptions.push(invalid);
            let mut invalid = exact_output();
            let base_symbol = invalid
                .payload
                .reserve
                .mode_expansions
                .keys()
                .find(|symbol| source_mode_symbol_spelling(symbol) == Some(BASE))
                .unwrap()
                .clone();
            let (_, middle) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(MIDDLE))
                .unwrap();
            middle.radix.spelling = BASE.to_owned();
            middle.radix.head = TypeHeadInput::Symbol(base_symbol);
            corruptions.push(invalid);
            let mut invalid = exact_output();
            let (_, inner) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(INNER))
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
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(BASE))
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
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(BASE))
                .unwrap();
            base.radix.source_range = range(source_id, 0, 1);
            corruptions.push(invalid);
            for invalid in corruptions {
                assert_invalid_output(invalid);
            }
            assert_source_reserved_variable_type_assertion_output(&exact_output())
                .expect("mutating cloned Task 221 outputs must not mutate the exact output");

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
                                    .any(|(index, value)| order[index + 1..].contains(value))
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
                                            ordered[a], ordered[b], ordered[c], ordered[d],
                                            ordered[e],
                                        ],
                                        reserve(),
                                        theorem(),
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
                let expected_radix = match index {
                    0 => "set",
                    1 => BASE,
                    2 => INNER,
                    3 => MIDDLE,
                    _ => OUTER,
                };
                let mut near_misses = Vec::new();
                let mut missing = exact_modes();
                missing.remove(index);
                near_misses.push(missing);
                let mut duplicate = exact_modes();
                duplicate.push(duplicate[index]);
                near_misses.push(duplicate);
                let mut wrong_label = exact_modes();
                wrong_label[index].label = Some("WrongFourEdgeModeFourHopAssertedHeadDef");
                near_misses.push(wrong_label);
                let mut wrong_pattern = exact_modes();
                wrong_pattern[index].pattern = OTHER;
                near_misses.push(wrong_pattern);
                let mut wrong_radix = exact_modes();
                wrong_radix[index].rhs_shape = if index == 0 {
                    ReserveTypeShape::Builtin("object")
                } else {
                    ReserveTypeShape::QualifiedSymbol(OTHER)
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
                let mut attributed = exact_modes();
                attributed[index].rhs_shape = if index == 0 {
                    ReserveTypeShape::AttributedSet
                } else {
                    ReserveTypeShape::AttributedQualifiedSymbol(expected_radix)
                };
                near_misses.push(attributed);
                for (variant, modes) in near_misses.into_iter().enumerate() {
                    assert_extraction_gap(
                        mode_then_reserve_identifier_type_assertion_theorem_ast(
                            source_id,
                            modes,
                            reserve(),
                            theorem(),
                        ),
                        &format!("definition {index} near miss {variant}"),
                    );
                }
            }

            for (context, index, wrong_radix) in [
                ("TooDeep-to-Outer relation link", 4, MIDDLE),
                ("Outer-to-Middle relation link", 3, INNER),
                ("Middle-to-Inner relation link", 2, BASE),
                ("Inner-to-Base relation link", 1, MIDDLE),
            ] {
                let mut modes = exact_modes();
                modes[index].rhs_shape = ReserveTypeShape::QualifiedSymbol(wrong_radix);
                assert_extraction_gap(
                    mode_then_reserve_identifier_type_assertion_theorem_ast(
                        source_id,
                        modes,
                        reserve(),
                        theorem(),
                    ),
                    context,
                );
            }

            let mut source_near_misses = Vec::new();
            for bad_reserve in [
                ReserveTypeShape::QualifiedSymbol(BASE),
                ReserveTypeShape::QualifiedSymbol(INNER),
                ReserveTypeShape::QualifiedSymbol(MIDDLE),
                ReserveTypeShape::QualifiedSymbol(OUTER),
                ReserveTypeShape::QualifiedSymbolWithArgs(TOO_DEEP),
                ReserveTypeShape::AttributedQualifiedSymbol(TOO_DEEP),
                ReserveTypeShape::Builtin("set"),
                ReserveTypeShape::Builtin("object"),
            ] {
                source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    exact_modes(),
                    vec![reserve_item(vec!["x"], bad_reserve)],
                    theorem(),
                ));
            }
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::QualifiedSymbol(TOO_DEEP)),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
                ],
                theorem(),
            ));
            for near_miss_theorem in [
                IdentifierTypeAssertionTheoremSpec {
                    label: "OtherPayloadBoundary",
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    subject: "y",
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(TOO_DEEP),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(OUTER),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(MIDDLE),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(INNER),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::Builtin("set"),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::Builtin("object"),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(OTHER),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(DEEPER),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbolWithArgs(BASE),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::AttributedQualifiedSymbol(BASE),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    negated: true,
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    status: Some("registration"),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    recovered_label: true,
                    ..theorem()
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
                DEEPER,
                "DeeperFourEdgeModeFourHopAssertedHeadDef",
                ReserveTypeShape::QualifiedSymbol(TOO_DEEP),
            ));
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                connected_deeper_modes,
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbol(DEEPER),
                )],
                theorem(),
            ));
            source_near_misses.push(
                modes_then_empty_definition_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    exact_modes(),
                    reserve(),
                    theorem(),
                ),
            );
            for (index, near_miss) in source_near_misses.into_iter().enumerate() {
                assert_extraction_gap(near_miss, &format!("source near miss {index}"));
            }

            let unrelated_import = source_local_and_imported_symbols_env(
                module.clone(),
                &all_modes.map(|spelling| (spelling, SymbolKind::Mode)),
                &[("UnrelatedFourHopMode", SymbolKind::Mode)],
            );
            assert!(
                extract_source_four_edge_local_mode_four_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &unrelated_import,
                )
                .is_some()
            );
            for imported_index in 0..5 {
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
            for imports in [
                all_modes
                    .iter()
                    .map(|spelling| (*spelling, SymbolKind::Mode))
                    .collect::<Vec<_>>(),
                all_modes
                    .iter()
                    .flat_map(|spelling| {
                        [(*spelling, SymbolKind::Mode), (*spelling, SymbolKind::Mode)]
                    })
                    .collect::<Vec<_>>(),
            ] {
                let provenance_near_miss =
                    source_local_and_imported_symbols_env(module.clone(), &[], &imports);
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

        #[test]
        fn route_rejects_all_46_prior_owner_fixtures_including_task_207_and_tasks_211_through_220()
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
            let plan = build_test_plan(&config).expect("Task 221 repository plan should build");
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
            ];
            assert_eq!(prior_owner_ids.len(), 46);
            assert_eq!(
                &prior_owner_ids[36..],
                &[
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
                ]
            );
            for owner_id in prior_owner_ids {
                let (ordinal, case) = active_type_elaboration_cases(&plan)
                    .enumerate()
                    .find(|(_, case)| case.id.0 == owner_id)
                    .unwrap_or_else(|| panic!("prior owner fixture {owner_id} must be active"));
                let frontend = run_frontend(&workspace_root, case, ordinal)
                    .unwrap_or_else(|error| panic!("owner fixture {owner_id} must parse: {error}"));
                assert!(frontend.diagnostics.is_empty());
                let ast = frontend
                    .ast
                    .unwrap_or_else(|| panic!("owner fixture {owner_id} must produce an AST"));
                let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
                assert!(resolver.detail_keys.is_empty());
                let symbols =
                    augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
                assert_eq!(
                    source_type_elaboration_detail_keys(&ast, resolver.module.clone(), &symbols),
                    Vec::<String>::new(),
                    "prior owner fixture {owner_id} must still reach its active route"
                );
                assert!(
                    extract_source_four_edge_local_mode_four_hop_asserted_head(
                        &ast,
                        resolver.module,
                        &symbols,
                    )
                    .is_none(),
                    "Task 221 must reject prior owner fixture {owner_id}"
                );
            }
        }

        #[test]
        fn task221_synthetic_is_rejected_by_all_46_prior_type_assertion_extractors() {
            let source_id = source_id(221);
            let module = ResolverModuleId::new(
                PackageId::new("test"),
                ModulePath::new("task221_prior_extractor_isolation"),
            );
            let symbols = source_local_symbols_env(
                module.clone(),
                &[
                    (BASE, SymbolKind::Mode),
                    (INNER, SymbolKind::Mode),
                    (MIDDLE, SymbolKind::Mode),
                    (OUTER, SymbolKind::Mode),
                    (TOO_DEEP, SymbolKind::Mode),
                ],
            );
            let exact = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                theorem(),
            );
            assert!(
                extract_source_four_edge_local_mode_four_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                )
                .is_some()
            );
            let prior_results = [
                extract_source_reserved_variable_type_assertion(&exact, module.clone(), &symbols),
                super::super::extract_source_reserved_object_variable_type_assertion(
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
                extract_source_chained_local_mode_radix_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
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
                extract_source_three_edge_local_mode_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
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
                super::super::extract_source_four_edge_local_mode_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                super::super::extract_source_four_edge_local_object_mode_asserted_head(
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
                super::super::extract_source_local_mode_long_chain_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                super::super::extract_source_local_object_mode_long_chain_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_local_mode_long_chain_radix_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                super::super::extract_source_local_object_mode_long_chain_radix_asserted_head(
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
                extract_source_three_edge_local_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_three_edge_local_object_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_four_edge_local_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_four_edge_local_object_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_three_edge_local_mode_three_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_three_edge_local_object_mode_three_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_four_edge_local_mode_three_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_four_edge_local_object_mode_three_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
            ];
            assert_eq!(prior_results.len(), 46);
            assert!(prior_results.into_iter().all(|result| result.is_none()));
        }
    }
    mod task_222_four_edge_object_four_hop_asserted_head {
        use super::*;

        const BASE: &str = "BaseFourEdgeObjectModeFourHopAssertedHead";
        const INNER: &str = "InnerFourEdgeObjectModeFourHopAssertedHead";
        const MIDDLE: &str = "MiddleFourEdgeObjectModeFourHopAssertedHead";
        const OUTER: &str = "OuterFourEdgeObjectModeFourHopAssertedHead";
        const TOO_DEEP: &str = "TooDeepFourEdgeObjectModeFourHopAssertedHead";
        const OTHER: &str = "OtherFourEdgeObjectModeFourHopAssertedHead";
        const DEEPER: &str = "DeeperFourEdgeObjectModeFourHopAssertedHead";

        fn theorem() -> IdentifierTypeAssertionTheoremSpec<'static> {
            IdentifierTypeAssertionTheoremSpec {
                status: None,
                label: "FourEdgeLocalObjectModeFourHopAssertedHeadPayloadBoundary",
                subject: "x",
                asserted_type: ReserveTypeShape::QualifiedSymbol(BASE),
                recovered_label: false,
                negated: false,
            }
        }

        fn exact_modes() -> Vec<ModeDefinitionSpec> {
            vec![
                mode_definition_with_label(
                    BASE,
                    "BaseFourEdgeObjectModeFourHopAssertedHeadDef",
                    ReserveTypeShape::Builtin("object"),
                ),
                mode_definition_with_label(
                    INNER,
                    "InnerFourEdgeObjectModeFourHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(BASE),
                ),
                mode_definition_with_label(
                    MIDDLE,
                    "MiddleFourEdgeObjectModeFourHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(INNER),
                ),
                mode_definition_with_label(
                    OUTER,
                    "OuterFourEdgeObjectModeFourHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(MIDDLE),
                ),
                mode_definition_with_label(
                    TOO_DEEP,
                    "TooDeepFourEdgeObjectModeFourHopAssertedHeadDef",
                    ReserveTypeShape::QualifiedSymbol(OUTER),
                ),
            ]
        }

        fn reserve() -> Vec<ReserveItemSpec> {
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol(TOO_DEEP),
            )]
        }

        #[test]
        fn active_fixture_consumes_five_real_object_expansions() {
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
            let plan = build_test_plan(&config).expect("Task 222 repository plan should build");
            let (ordinal, case) = active_type_elaboration_cases(&plan)
                .enumerate()
                .find(|(_, case)| {
                    case.id.0
                        == "pass_type_elaboration_four_edge_local_object_mode_four_hop_asserted_head_001"
                })
                .expect("Task 222 active fixture should be discoverable");
            let frontend = run_frontend(&workspace_root, case, ordinal)
                .expect("Task 222 fixture should run through the real frontend");
            assert!(frontend.diagnostics.is_empty());
            let ast = frontend
                .ast
                .expect("Task 222 fixture should produce an AST");
            let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
            assert!(resolver.detail_keys.is_empty());
            let symbols =
                augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
            assert_eq!(
                source_type_elaboration_detail_keys(&ast, resolver.module.clone(), &symbols),
                Vec::<String>::new()
            );
            let output = source_four_edge_local_object_mode_four_hop_asserted_head_output(
                &ast,
                resolver.module,
                &symbols,
            )
            .expect("Task 222 real AST should reach the exact four-link asserted-head seam");
            assert_source_reserved_variable_type_assertion_output(&output)
                .expect("Task 222 real AST should preserve every checked payload invariant");
            assert_eq!(output.payload.reserve.mode_expansions.len(), 5);
            assert_eq!(output.subject_binding, BindingId::new(0));
            assert_eq!(output.payload.subject_lookup_ordinal, 1);
            assert_eq!(output.term_formula.type_entries().len(), 3);
            assert_eq!(output.term_formula.normalized_types().len(), 1);
            assert_eq!(output.subject_result_input.spelling, TOO_DEEP);
            assert_eq!(output.asserted_type_input.spelling, BASE);
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
            let expansion = |spelling| {
                output
                    .payload
                    .reserve
                    .mode_expansions
                    .iter()
                    .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(spelling))
                    .map(|(_, expansion)| expansion)
                    .unwrap()
            };
            let too_deep = expansion(TOO_DEEP);
            let outer = expansion(OUTER);
            let middle = expansion(MIDDLE);
            let inner = expansion(INNER);
            let base = expansion(BASE);
            assert_eq!(too_deep.radix.spelling, OUTER);
            assert_eq!(outer.radix.spelling, MIDDLE);
            assert_eq!(middle.radix.spelling, INNER);
            assert_eq!(inner.radix.spelling, BASE);
            assert_eq!(inner.radix.head, output.asserted_type_input.head);
            assert_eq!(base.radix.spelling, "object");
            assert_eq!(base.radix.head, TypeHeadInput::BuiltinObject);
            let (_, normalized) = output
                .term_formula
                .normalized_types()
                .iter()
                .next()
                .unwrap();
            assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
            assert_eq!(normalized.source.range, base.radix.source_range);
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
        }

        #[test]
        fn synthetic_exactness_matrix_rejects_every_near_miss() {
            let source_id = source_id(222);
            let module = ResolverModuleId::new(
                PackageId::new("test"),
                ModulePath::new("four_edge_local_object_mode_four_hop_asserted_head"),
            );
            let all_modes = [BASE, INNER, MIDDLE, OUTER, TOO_DEEP];
            let symbols = source_local_symbols_env(
                module.clone(),
                &[
                    (BASE, SymbolKind::Mode),
                    (INNER, SymbolKind::Mode),
                    (MIDDLE, SymbolKind::Mode),
                    (OUTER, SymbolKind::Mode),
                    (TOO_DEEP, SymbolKind::Mode),
                    (OTHER, SymbolKind::Mode),
                    (DEEPER, SymbolKind::Mode),
                ],
            );
            let exact = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                theorem(),
            );
            assert_eq!(
                source_type_elaboration_detail_keys(&exact, module.clone(), &symbols),
                Vec::<String>::new()
            );
            let payload = extract_source_four_edge_local_object_mode_four_hop_asserted_head(
                &exact,
                module.clone(),
                &symbols,
            )
            .expect("the exact object-terminal four-edge four-hop source should extract");
            assert_eq!(payload.reserve.mode_expansions.len(), 5);
            assert_eq!(payload.reserve.bridge.bindings().len(), 1);
            assert_eq!(payload.subject_lookup_ordinal, 1);
            assert_eq!(payload.reserve.bridge.bindings()[0].type_spelling, TOO_DEEP);
            assert_eq!(payload.asserted_type.spelling, BASE);

            let exact_output = || {
                source_four_edge_local_object_mode_four_hop_asserted_head_output(
                    &exact,
                    module.clone(),
                    &symbols,
                )
                .unwrap()
            };
            let output = exact_output();
            assert_source_reserved_variable_type_assertion_output(&output)
                .expect("Task 222 exact checker output must satisfy every invariant");

            let assert_invalid_output = |invalid| {
                let invalid_result =
                    assert_source_reserved_variable_type_assertion_output(&invalid)
                        .map(|()| invalid);
                assert_eq!(
                    source_reserved_variable_type_assertion_result_detail_keys(
                        invalid_result,
                        TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_FOUR_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
                    ),
                    vec![
                        TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_FOUR_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY
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
            invalid.subject_result_input.spelling = OUTER.to_owned();
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
            invalid.asserted_type_input.spelling = INNER.to_owned();
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
            let middle_symbol = invalid
                .payload
                .reserve
                .mode_expansions
                .keys()
                .find(|symbol| source_mode_symbol_spelling(symbol) == Some(MIDDLE))
                .unwrap()
                .clone();
            let (_, too_deep) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(TOO_DEEP))
                .unwrap();
            too_deep.radix.spelling = MIDDLE.to_owned();
            too_deep.radix.head = TypeHeadInput::Symbol(middle_symbol);
            corruptions.push(invalid);
            let mut invalid = exact_output();
            let inner_symbol = invalid
                .payload
                .reserve
                .mode_expansions
                .keys()
                .find(|symbol| source_mode_symbol_spelling(symbol) == Some(INNER))
                .unwrap()
                .clone();
            let (_, outer) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(OUTER))
                .unwrap();
            outer.radix.spelling = INNER.to_owned();
            outer.radix.head = TypeHeadInput::Symbol(inner_symbol);
            corruptions.push(invalid);
            let mut invalid = exact_output();
            let base_symbol = invalid
                .payload
                .reserve
                .mode_expansions
                .keys()
                .find(|symbol| source_mode_symbol_spelling(symbol) == Some(BASE))
                .unwrap()
                .clone();
            let (_, middle) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(MIDDLE))
                .unwrap();
            middle.radix.spelling = BASE.to_owned();
            middle.radix.head = TypeHeadInput::Symbol(base_symbol);
            corruptions.push(invalid);
            let mut invalid = exact_output();
            let (_, inner) = invalid
                .payload
                .reserve
                .mode_expansions
                .iter_mut()
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(INNER))
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
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(BASE))
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
                .find(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(BASE))
                .unwrap();
            base.radix.source_range = range(source_id, 0, 1);
            corruptions.push(invalid);
            for invalid in corruptions {
                assert_invalid_output(invalid);
            }
            assert_source_reserved_variable_type_assertion_output(&exact_output())
                .expect("mutating cloned Task 222 outputs must not mutate the exact output");

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
                                    .any(|(index, value)| order[index + 1..].contains(value))
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
                                            ordered[a], ordered[b], ordered[c], ordered[d],
                                            ordered[e],
                                        ],
                                        reserve(),
                                        theorem(),
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
                let expected_radix = match index {
                    0 => "object",
                    1 => BASE,
                    2 => INNER,
                    3 => MIDDLE,
                    _ => OUTER,
                };
                let mut near_misses = Vec::new();
                let mut missing = exact_modes();
                missing.remove(index);
                near_misses.push(missing);
                let mut duplicate = exact_modes();
                duplicate.push(duplicate[index]);
                near_misses.push(duplicate);
                let mut wrong_label = exact_modes();
                wrong_label[index].label = Some("WrongFourEdgeObjectModeFourHopAssertedHeadDef");
                near_misses.push(wrong_label);
                let mut wrong_pattern = exact_modes();
                wrong_pattern[index].pattern = OTHER;
                near_misses.push(wrong_pattern);
                let mut wrong_radix = exact_modes();
                wrong_radix[index].rhs_shape = if index == 0 {
                    ReserveTypeShape::Builtin("set")
                } else {
                    ReserveTypeShape::QualifiedSymbol(OTHER)
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
                let mut attributed = exact_modes();
                attributed[index].rhs_shape = if index == 0 {
                    ReserveTypeShape::AttributedObject
                } else {
                    ReserveTypeShape::AttributedQualifiedSymbol(expected_radix)
                };
                near_misses.push(attributed);
                for (variant, modes) in near_misses.into_iter().enumerate() {
                    assert_extraction_gap(
                        mode_then_reserve_identifier_type_assertion_theorem_ast(
                            source_id,
                            modes,
                            reserve(),
                            theorem(),
                        ),
                        &format!("definition {index} near miss {variant}"),
                    );
                }
            }

            for (context, index, wrong_radix) in [
                ("TooDeep-to-Outer relation link", 4, MIDDLE),
                ("Outer-to-Middle relation link", 3, INNER),
                ("Middle-to-Inner relation link", 2, BASE),
                ("Inner-to-Base relation link", 1, MIDDLE),
            ] {
                let mut modes = exact_modes();
                modes[index].rhs_shape = ReserveTypeShape::QualifiedSymbol(wrong_radix);
                assert_extraction_gap(
                    mode_then_reserve_identifier_type_assertion_theorem_ast(
                        source_id,
                        modes,
                        reserve(),
                        theorem(),
                    ),
                    context,
                );
            }

            let mut source_near_misses = Vec::new();
            for bad_reserve in [
                ReserveTypeShape::QualifiedSymbol(BASE),
                ReserveTypeShape::QualifiedSymbol(INNER),
                ReserveTypeShape::QualifiedSymbol(MIDDLE),
                ReserveTypeShape::QualifiedSymbol(OUTER),
                ReserveTypeShape::QualifiedSymbolWithArgs(TOO_DEEP),
                ReserveTypeShape::AttributedQualifiedSymbol(TOO_DEEP),
                ReserveTypeShape::Builtin("set"),
                ReserveTypeShape::Builtin("object"),
            ] {
                source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    exact_modes(),
                    vec![reserve_item(vec!["x"], bad_reserve)],
                    theorem(),
                ));
            }
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                vec![
                    reserve_item(vec!["x"], ReserveTypeShape::QualifiedSymbol(TOO_DEEP)),
                    reserve_item(vec!["y"], ReserveTypeShape::Builtin("object")),
                ],
                theorem(),
            ));
            for near_miss_theorem in [
                IdentifierTypeAssertionTheoremSpec {
                    label: "OtherPayloadBoundary",
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    subject: "y",
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(TOO_DEEP),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(OUTER),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(MIDDLE),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(INNER),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::Builtin("set"),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::Builtin("object"),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(OTHER),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbol(DEEPER),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::QualifiedSymbolWithArgs(BASE),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    asserted_type: ReserveTypeShape::AttributedQualifiedSymbol(BASE),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    negated: true,
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    status: Some("registration"),
                    ..theorem()
                },
                IdentifierTypeAssertionTheoremSpec {
                    recovered_label: true,
                    ..theorem()
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
                DEEPER,
                "DeeperFourEdgeObjectModeFourHopAssertedHeadDef",
                ReserveTypeShape::QualifiedSymbol(TOO_DEEP),
            ));
            source_near_misses.push(mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                connected_deeper_modes,
                vec![reserve_item(
                    vec!["x"],
                    ReserveTypeShape::QualifiedSymbol(DEEPER),
                )],
                theorem(),
            ));
            source_near_misses.push(
                modes_then_empty_definition_reserve_identifier_type_assertion_theorem_ast(
                    source_id,
                    exact_modes(),
                    reserve(),
                    theorem(),
                ),
            );
            for (index, near_miss) in source_near_misses.into_iter().enumerate() {
                assert_extraction_gap(near_miss, &format!("source near miss {index}"));
            }

            let unrelated_import = source_local_and_imported_symbols_env(
                module.clone(),
                &all_modes.map(|spelling| (spelling, SymbolKind::Mode)),
                &[("UnrelatedFourHopMode", SymbolKind::Mode)],
            );
            assert!(
                extract_source_four_edge_local_object_mode_four_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &unrelated_import,
                )
                .is_some()
            );
            for imported_index in 0..5 {
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
            for imports in [
                all_modes
                    .iter()
                    .map(|spelling| (*spelling, SymbolKind::Mode))
                    .collect::<Vec<_>>(),
                all_modes
                    .iter()
                    .flat_map(|spelling| {
                        [(*spelling, SymbolKind::Mode), (*spelling, SymbolKind::Mode)]
                    })
                    .collect::<Vec<_>>(),
            ] {
                let provenance_near_miss =
                    source_local_and_imported_symbols_env(module.clone(), &[], &imports);
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

        #[test]
        fn route_rejects_all_47_prior_owner_fixtures_including_task_208_and_tasks_211_through_221()
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
            let plan = build_test_plan(&config).expect("Task 222 repository plan should build");
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
            ];
            assert_eq!(prior_owner_ids.len(), 47);
            assert_eq!(
                &prior_owner_ids[36..],
                &[
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
                ]
            );
            for owner_id in prior_owner_ids {
                let (ordinal, case) = active_type_elaboration_cases(&plan)
                    .enumerate()
                    .find(|(_, case)| case.id.0 == owner_id)
                    .unwrap_or_else(|| panic!("prior owner fixture {owner_id} must be active"));
                let frontend = run_frontend(&workspace_root, case, ordinal)
                    .unwrap_or_else(|error| panic!("owner fixture {owner_id} must parse: {error}"));
                assert!(frontend.diagnostics.is_empty());
                let ast = frontend
                    .ast
                    .unwrap_or_else(|| panic!("owner fixture {owner_id} must produce an AST"));
                let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
                assert!(resolver.detail_keys.is_empty());
                let symbols =
                    augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
                assert_eq!(
                    source_type_elaboration_detail_keys(&ast, resolver.module.clone(), &symbols),
                    Vec::<String>::new(),
                    "prior owner fixture {owner_id} must still reach its active route"
                );
                assert!(
                    extract_source_four_edge_local_object_mode_four_hop_asserted_head(
                        &ast,
                        resolver.module,
                        &symbols,
                    )
                    .is_none(),
                    "Task 222 must reject prior owner fixture {owner_id}"
                );
            }
        }

        #[test]
        fn task222_synthetic_is_rejected_by_all_47_prior_type_assertion_extractors() {
            let source_id = source_id(222);
            let module = ResolverModuleId::new(
                PackageId::new("test"),
                ModulePath::new("task222_prior_extractor_isolation"),
            );
            let symbols = source_local_symbols_env(
                module.clone(),
                &[
                    (BASE, SymbolKind::Mode),
                    (INNER, SymbolKind::Mode),
                    (MIDDLE, SymbolKind::Mode),
                    (OUTER, SymbolKind::Mode),
                    (TOO_DEEP, SymbolKind::Mode),
                ],
            );
            let exact = mode_then_reserve_identifier_type_assertion_theorem_ast(
                source_id,
                exact_modes(),
                reserve(),
                theorem(),
            );
            assert!(
                extract_source_four_edge_local_object_mode_four_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                )
                .is_some()
            );
            let prior_results = [
                extract_source_reserved_variable_type_assertion(&exact, module.clone(), &symbols),
                super::super::extract_source_reserved_object_variable_type_assertion(
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
                extract_source_chained_local_mode_radix_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
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
                extract_source_three_edge_local_mode_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
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
                super::super::extract_source_four_edge_local_mode_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                super::super::extract_source_four_edge_local_object_mode_asserted_head(
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
                super::super::extract_source_local_mode_long_chain_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                super::super::extract_source_local_object_mode_long_chain_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_local_mode_long_chain_radix_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                super::super::extract_source_local_object_mode_long_chain_radix_asserted_head(
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
                extract_source_three_edge_local_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_three_edge_local_object_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_four_edge_local_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_four_edge_local_object_mode_two_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_three_edge_local_mode_three_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_three_edge_local_object_mode_three_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_four_edge_local_mode_three_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_four_edge_local_object_mode_three_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
                extract_source_four_edge_local_mode_four_hop_asserted_head(
                    &exact,
                    module.clone(),
                    &symbols,
                ),
            ];
            assert_eq!(prior_results.len(), 47);
            assert!(prior_results.into_iter().all(|result| result.is_none()));
        }
    }
