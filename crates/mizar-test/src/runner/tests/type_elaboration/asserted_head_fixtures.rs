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
