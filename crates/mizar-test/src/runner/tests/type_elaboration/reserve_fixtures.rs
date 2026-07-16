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
