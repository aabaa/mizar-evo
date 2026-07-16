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
