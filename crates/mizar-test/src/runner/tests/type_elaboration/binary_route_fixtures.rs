    #[test]
    fn active_reserved_variable_equality_fixture_preserves_real_checker_payload() {
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
            .find(|(_, case)| case.id.0 == "pass_type_elaboration_reserved_variable_equality_001")
            .expect("Task 119 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 119 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 119 fixture should produce an AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = source_reserved_variable_equality_output(&ast, resolver.module, &symbols)
            .expect("Task 119 real AST should reach the reserved-variable equality checker seam");
        assert_source_reserved_variable_formula_output(&output)
            .expect("Task 119 real AST should preserve every checked payload invariant");
    }

    #[test]
    fn active_parenthesized_reserved_variable_equality_fixture_preserves_real_checker_payload() {
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
        let plan = build_test_plan(&config).expect("Task 223 repository plan should build");
        let (ordinal, case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| {
                case.id.0 == "pass_type_elaboration_parenthesized_reserved_variable_equality_001"
            })
            .expect("Task 223 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 223 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 223 fixture should produce a real AST");
        let parenthesized_terms = surface_nodes_with_kind(&ast, SurfaceNodeKind::ParenthesizedTerm);
        let [(wrapper_id, wrapper)] = parenthesized_terms.as_slice() else {
            panic!("Task 223 real AST should contain exactly one ParenthesizedTerm");
        };
        assert_eq!(super::direct_token_texts(&ast, wrapper), ["(", ")"]);
        assert_eq!(super::structural_child_ids(&ast, wrapper).len(), 1);
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        let output = super::source_parenthesized_reserved_variable_equality_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 223 real AST should reach the parenthesized equality seam");
        assert_eq!(output.wrapper_site, surface_site(*wrapper_id));
        assert_eq!(output.wrapper_range, wrapper.range);
        super::assert_source_parenthesized_reserved_variable_equality_output(&output)
            .expect("Task 223 real AST should preserve every checked payload invariant");
        assert_eq!(output.formula.payload.left_lookup_ordinal, 1);
        assert_eq!(output.formula.payload.right_lookup_ordinal, 2);
        assert_eq!(output.formula.left_binding, BindingId::new(0));
        assert_eq!(output.formula.right_binding, BindingId::new(0));
    }

    #[test]
    fn active_parenthesized_reserved_variable_inequality_fixture_preserves_real_checker_payload() {
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
        let plan = build_test_plan(&config).expect("Task 241 repository plan should build");
        let (ordinal, case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| {
                case.id.0 == "pass_type_elaboration_parenthesized_reserved_variable_inequality_001"
            })
            .expect("Task 241 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 241 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 241 fixture should produce a real AST");
        let parenthesized_terms = surface_nodes_with_kind(&ast, SurfaceNodeKind::ParenthesizedTerm);
        let [(wrapper_id, wrapper)] = parenthesized_terms.as_slice() else {
            panic!("Task 241 real AST should contain exactly one ParenthesizedTerm");
        };
        assert_eq!(super::direct_token_texts(&ast, wrapper), ["(", ")"]);
        assert_eq!(super::structural_child_ids(&ast, wrapper).len(), 1);
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        assert!(
            super::extract_source_parenthesized_reserved_variable_equality(
                &ast,
                resolver.module.clone(),
                &symbols,
            )
            .is_none()
        );
        assert!(
            super::extract_source_reserved_variable_inequality(
                &ast,
                resolver.module.clone(),
                &symbols,
            )
            .is_none()
        );
        let output = super::source_parenthesized_reserved_variable_inequality_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 241 real AST should reach the parenthesized inequality seam");
        assert_eq!(output.wrapper_site, surface_site(*wrapper_id));
        assert_eq!(output.wrapper_range, wrapper.range);
        super::assert_source_parenthesized_reserved_variable_inequality_output(&output)
            .expect("Task 241 real AST should preserve every checked payload invariant");
        assert_eq!(output.formula.payload.left_lookup_ordinal, 1);
        assert_eq!(output.formula.payload.right_lookup_ordinal, 2);
        assert_eq!(output.formula.left_binding, BindingId::new(0));
        assert_eq!(output.formula.right_binding, BindingId::new(0));
        let (_, formula) = output
            .formula
            .term_formula
            .formulas()
            .iter()
            .next()
            .expect("Task 241 real AST should produce one checked formula");
        assert_eq!(formula.kind, FormulaKind::Inequality);
        assert_eq!(formula.status, FormulaStatus::Checked);
    }

    #[test]
    fn active_parenthesized_reserved_object_variable_equality_fixture_preserves_real_checker_payload()
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
        let plan = build_test_plan(&config).expect("Task 233 repository plan should build");
        let (ordinal, case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| {
                case.id.0
                    == "pass_type_elaboration_parenthesized_reserved_object_variable_equality_001"
            })
            .expect("Task 233 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 233 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 233 fixture should produce a real AST");
        let parenthesized_terms = surface_nodes_with_kind(&ast, SurfaceNodeKind::ParenthesizedTerm);
        let [(wrapper_id, wrapper)] = parenthesized_terms.as_slice() else {
            panic!("Task 233 real AST should contain exactly one ParenthesizedTerm");
        };
        assert_eq!(super::direct_token_texts(&ast, wrapper), ["(", ")"]);
        assert_eq!(super::structural_child_ids(&ast, wrapper).len(), 1);
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        assert!(
            super::extract_source_parenthesized_reserved_variable_equality(
                &ast,
                resolver.module.clone(),
                &symbols,
            )
            .is_none()
        );
        assert!(
            super::extract_source_reserved_object_variable_equality(
                &ast,
                resolver.module.clone(),
                &symbols,
            )
            .is_none()
        );
        let output = super::source_parenthesized_reserved_object_variable_equality_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 233 real AST should reach the parenthesized object equality seam");
        assert_eq!(output.wrapper_site, surface_site(*wrapper_id));
        assert_eq!(output.wrapper_range, wrapper.range);
        super::assert_source_parenthesized_reserved_object_variable_equality_output(&output)
            .expect("Task 233 real AST should preserve every checked payload invariant");
        assert_eq!(output.formula.payload.left_lookup_ordinal, 1);
        assert_eq!(output.formula.payload.right_lookup_ordinal, 2);
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
        assert_eq!(output.formula.term_formula.normalized_types().len(), 1);
        let (_, normalized) = output
            .formula
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 233 real AST should preserve a canonical object identity");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_eq!(normalized.source.spelling, "object");
    }

    #[test]
    fn active_parenthesized_reserved_object_variable_inequality_fixture_preserves_real_checker_payload()
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
        let plan = build_test_plan(&config).expect("Task 242 repository plan should build");
        let (ordinal, case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| {
                case.id.0
                    == "pass_type_elaboration_parenthesized_reserved_object_variable_inequality_001"
            })
            .expect("Task 242 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 242 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 242 fixture should produce a real AST");
        let parenthesized_terms = surface_nodes_with_kind(&ast, SurfaceNodeKind::ParenthesizedTerm);
        let [(wrapper_id, wrapper)] = parenthesized_terms.as_slice() else {
            panic!("Task 242 real AST should contain exactly one ParenthesizedTerm");
        };
        assert_eq!(super::direct_token_texts(&ast, wrapper), ["(", ")"]);
        assert_eq!(super::structural_child_ids(&ast, wrapper).len(), 1);
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        assert!(
            super::extract_source_parenthesized_reserved_object_variable_equality(
                &ast,
                resolver.module.clone(),
                &symbols,
            )
            .is_none()
        );
        assert!(
            super::extract_source_parenthesized_reserved_variable_inequality(
                &ast,
                resolver.module.clone(),
                &symbols,
            )
            .is_none()
        );
        assert!(
            super::extract_source_reserved_object_variable_inequality(
                &ast,
                resolver.module.clone(),
                &symbols,
            )
            .is_none()
        );
        let output = super::source_parenthesized_reserved_object_variable_inequality_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 242 real AST should reach the parenthesized object inequality seam");
        assert_eq!(output.wrapper_site, surface_site(*wrapper_id));
        assert_eq!(output.wrapper_range, wrapper.range);
        super::assert_source_parenthesized_reserved_object_variable_inequality_output(&output)
            .expect("Task 242 real AST should preserve every checked payload invariant");
        assert_eq!(output.formula.payload.left_lookup_ordinal, 1);
        assert_eq!(output.formula.payload.right_lookup_ordinal, 2);
        assert_eq!(output.formula.left_binding, BindingId::new(0));
        assert_eq!(output.formula.right_binding, BindingId::new(0));
        assert_eq!(output.formula.term_formula.normalized_types().len(), 1);
        let (_, normalized) = output
            .formula
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 242 real AST should preserve a canonical object identity");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinObject);
        assert_eq!(normalized.source.spelling, "object");
        let (_, formula) = output
            .formula
            .term_formula
            .formulas()
            .iter()
            .next()
            .expect("Task 242 real AST should produce one checked formula");
        assert_eq!(formula.kind, FormulaKind::Inequality);
        assert_eq!(formula.status, FormulaStatus::Checked);
    }

    #[test]
    fn active_parenthesized_reserved_variable_membership_fixture_preserves_real_checker_payload() {
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
        let plan = build_test_plan(&config).expect("Task 243 repository plan should build");
        let (ordinal, case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| {
                case.id.0 == "pass_type_elaboration_parenthesized_reserved_variable_membership_001"
            })
            .expect("Task 243 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 243 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 243 fixture should produce a real AST");
        let parenthesized_terms = surface_nodes_with_kind(&ast, SurfaceNodeKind::ParenthesizedTerm);
        let [(wrapper_id, wrapper)] = parenthesized_terms.as_slice() else {
            panic!("Task 243 real AST should contain exactly one ParenthesizedTerm");
        };
        assert_eq!(super::direct_token_texts(&ast, wrapper), ["(", ")"]);
        assert_eq!(super::structural_child_ids(&ast, wrapper).len(), 1);
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        for extractor in [
            super::extract_source_parenthesized_reserved_variable_equality,
            super::extract_source_parenthesized_reserved_variable_inequality,
            super::extract_source_parenthesized_reserved_object_variable_equality,
            super::extract_source_parenthesized_reserved_object_variable_inequality,
        ] {
            assert!(extractor(&ast, resolver.module.clone(), &symbols).is_none());
        }
        assert!(
            super::extract_source_reserved_variable_membership(
                &ast,
                resolver.module.clone(),
                &symbols,
            )
            .is_none()
        );
        let output = super::source_parenthesized_reserved_variable_membership_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 243 real AST should reach the parenthesized membership seam");
        assert_eq!(output.wrapper_site, surface_site(*wrapper_id));
        assert_eq!(output.wrapper_range, wrapper.range);
        super::assert_source_parenthesized_reserved_variable_membership_output(&output)
            .expect("Task 243 real AST should preserve every checked payload invariant");
        assert_eq!(output.formula.payload.left_lookup_ordinal, 1);
        assert_eq!(output.formula.payload.right_lookup_ordinal, 2);
        assert_eq!(output.formula.left_binding, BindingId::new(0));
        assert_eq!(output.formula.right_binding, BindingId::new(0));
        assert!(output.formula.left_expected_input.is_none());
        assert_eq!(
            output.formula.right_expected_input.as_ref().unwrap().head,
            TypeHeadInput::BuiltinSet
        );
        assert_eq!(output.formula.term_formula.terms().len(), 2);
        assert_eq!(output.formula.term_formula.type_entries().len(), 5);
        assert_eq!(output.formula.term_formula.normalized_types().len(), 1);
        let (_, normalized) = output
            .formula
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 243 real AST should preserve a canonical set identity");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_eq!(normalized.source.spelling, "set");
        let (_, formula) = output
            .formula
            .term_formula
            .formulas()
            .iter()
            .next()
            .expect("Task 243 real AST should produce one checked formula");
        assert_eq!(formula.kind, FormulaKind::Membership);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert_eq!(formula.expected_types.len(), 1);
        assert_eq!(
            formula.expected_types[0].term,
            output.formula.payload.right_site
        );
    }

    #[test]
    fn active_parenthesized_heterogeneous_reserve_membership_fixture_preserves_real_checker_payload()
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
        let plan = build_test_plan(&config).expect("Task 244 repository plan should build");
        let (ordinal, case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| {
                case.id.0
                    == "pass_type_elaboration_parenthesized_heterogeneous_reserve_membership_001"
            })
            .expect("Task 244 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 244 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 244 fixture should produce a real AST");
        let parenthesized_terms = surface_nodes_with_kind(&ast, SurfaceNodeKind::ParenthesizedTerm);
        let [(wrapper_id, wrapper)] = parenthesized_terms.as_slice() else {
            panic!("Task 244 real AST should contain exactly one ParenthesizedTerm");
        };
        assert_eq!(super::direct_token_texts(&ast, wrapper), ["(", ")"]);
        assert_eq!(super::structural_child_ids(&ast, wrapper).len(), 1);
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        assert!(
            super::extract_source_heterogeneous_reserve_membership(
                &ast,
                resolver.module.clone(),
                &symbols,
            )
            .is_none()
        );
        for extractor in [
            super::extract_source_parenthesized_reserved_variable_equality,
            super::extract_source_parenthesized_reserved_object_variable_equality,
            super::extract_source_parenthesized_reserved_variable_inequality,
            super::extract_source_parenthesized_reserved_object_variable_inequality,
            super::extract_source_parenthesized_reserved_variable_membership,
        ] {
            assert!(extractor(&ast, resolver.module.clone(), &symbols).is_none());
        }
        let output = super::source_parenthesized_heterogeneous_reserve_membership_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 244 real AST should reach the parenthesized heterogeneous membership seam");
        assert_eq!(output.wrapper_site, surface_site(*wrapper_id));
        assert_eq!(output.wrapper_range, wrapper.range);
        super::assert_source_parenthesized_heterogeneous_reserve_membership_output(&output)
            .expect("Task 244 real AST should preserve every checked payload invariant");
        let bindings = output.formula.payload.reserve.bridge.bindings();
        assert_eq!(bindings.len(), 2);
        assert_eq!(bindings[0].spelling, "x");
        assert_eq!(bindings[0].type_head, TypeHeadInput::BuiltinObject);
        assert_eq!(bindings[1].spelling, "y");
        assert_eq!(bindings[1].type_head, TypeHeadInput::BuiltinSet);
        assert_ne!(bindings[0].type_range, bindings[1].type_range);
        assert!(bindings[0].type_range.start < bindings[1].type_range.start);
        assert_eq!(output.formula.payload.left_lookup_ordinal, 2);
        assert_eq!(output.formula.payload.right_lookup_ordinal, 3);
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
        let normalized = output
            .formula
            .term_formula
            .normalized_types()
            .iter()
            .map(|(_, normalized)| (normalized.head.clone(), normalized))
            .collect::<BTreeMap<_, _>>();
        assert_eq!(
            normalized
                .get(&TypeHeadRef::BuiltinObject)
                .expect("Task 244 object identity should remain distinct")
                .source
                .range,
            bindings[0].type_range
        );
        assert_eq!(
            normalized
                .get(&TypeHeadRef::BuiltinSet)
                .expect("Task 244 set identity should remain distinct")
                .source
                .range,
            bindings[1].type_range
        );
        let (_, formula) = output
            .formula
            .term_formula
            .formulas()
            .iter()
            .next()
            .expect("Task 244 real AST should produce one checked formula");
        assert_eq!(formula.kind, FormulaKind::Membership);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert_eq!(formula.expected_types.len(), 1);
        assert_eq!(
            formula.expected_types[0].term,
            output.formula.payload.right_site
        );
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
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
    }

    #[test]
    fn active_right_parenthesized_reserved_variable_membership_fixture_preserves_real_checker_payload()
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
        let plan = build_test_plan(&config).expect("Task 245 repository plan should build");
        let (ordinal, case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| {
                case.id.0
                    == "pass_type_elaboration_right_parenthesized_reserved_variable_membership_001"
            })
            .expect("Task 245 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 245 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 245 fixture should produce a real AST");
        let parenthesized_terms = surface_nodes_with_kind(&ast, SurfaceNodeKind::ParenthesizedTerm);
        let [(wrapper_id, wrapper)] = parenthesized_terms.as_slice() else {
            panic!("Task 245 real AST should contain exactly one ParenthesizedTerm");
        };
        assert_eq!(super::direct_token_texts(&ast, wrapper), ["(", ")"]);
        assert_eq!(super::structural_child_ids(&ast, wrapper).len(), 1);
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        for extractor in [
            super::extract_source_parenthesized_reserved_variable_equality,
            super::extract_source_parenthesized_reserved_variable_inequality,
            super::extract_source_parenthesized_reserved_variable_membership,
            super::extract_source_parenthesized_heterogeneous_reserve_membership,
            super::extract_source_parenthesized_reserved_object_variable_equality,
            super::extract_source_parenthesized_reserved_object_variable_inequality,
        ] {
            assert!(extractor(&ast, resolver.module.clone(), &symbols).is_none());
        }
        assert!(
            super::extract_source_reserved_variable_membership(
                &ast,
                resolver.module.clone(),
                &symbols,
            )
            .is_none()
        );
        let output = super::source_right_parenthesized_reserved_variable_membership_output(
            &ast,
            resolver.module,
            &symbols,
        )
        .expect("Task 245 real AST should reach the right-parenthesized membership seam");
        assert_eq!(output.wrapper_site, surface_site(*wrapper_id));
        assert_eq!(output.wrapper_range, wrapper.range);
        super::assert_source_right_parenthesized_reserved_variable_membership_output(&output)
            .expect("Task 245 real AST should preserve every checked payload invariant");
        assert_eq!(
            output.wrapper_side,
            super::SourceParenthesizedOperandSide::Right
        );
        assert_eq!(output.formula.payload.left_lookup_ordinal, 1);
        assert_eq!(output.formula.payload.right_lookup_ordinal, 2);
        assert_eq!(output.formula.left_binding, BindingId::new(0));
        assert_eq!(output.formula.right_binding, BindingId::new(0));
        assert!(output.formula.left_expected_input.is_none());
        assert_eq!(
            output.formula.right_expected_input.as_ref().unwrap().head,
            TypeHeadInput::BuiltinSet
        );
        assert_eq!(output.formula.term_formula.terms().len(), 2);
        assert_eq!(output.formula.term_formula.type_entries().len(), 5);
        assert_eq!(output.formula.term_formula.normalized_types().len(), 1);
        let (_, normalized) = output
            .formula
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 245 real AST should preserve a canonical set identity");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_eq!(normalized.source.spelling, "set");
        let (_, formula) = output
            .formula
            .term_formula
            .formulas()
            .iter()
            .next()
            .expect("Task 245 real AST should produce one checked formula");
        assert_eq!(formula.kind, FormulaKind::Membership);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert_eq!(formula.expected_types.len(), 1);
        assert_eq!(
            formula.expected_types[0].term,
            output.formula.payload.right_site
        );
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
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
    }

    #[test]
    fn active_parenthesized_two_edge_local_mode_reserved_variable_equality_fixture_preserves_real_checker_payload()
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
        let plan = build_test_plan(&config).expect("Task 246 repository plan should build");
        let (ordinal, case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| {
                case.id.0
                    == "pass_type_elaboration_parenthesized_two_edge_local_mode_reserved_variable_equality_001"
            })
            .expect("Task 246 active fixture should be discoverable");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("Task 246 fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("Task 246 fixture should produce a real AST");
        let parenthesized_terms = surface_nodes_with_kind(&ast, SurfaceNodeKind::ParenthesizedTerm);
        let [(wrapper_id, wrapper)] = parenthesized_terms.as_slice() else {
            panic!("Task 246 real AST should contain exactly one ParenthesizedTerm");
        };
        assert_eq!(super::direct_token_texts(&ast, wrapper), ["(", ")"]);
        assert_eq!(super::structural_child_ids(&ast, wrapper).len(), 1);
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        assert!(
            super::extract_source_two_edge_local_mode_reserved_variable_equality(
                &ast,
                resolver.module.clone(),
                &symbols,
            )
            .is_none()
        );
        assert!(
            super::extract_source_parenthesized_reserved_variable_equality(
                &ast,
                resolver.module.clone(),
                &symbols,
            )
            .is_none()
        );
        let output =
            super::source_parenthesized_two_edge_local_mode_reserved_variable_equality_output(
                &ast,
                resolver.module,
                &symbols,
            )
            .expect("Task 246 real AST should reach the exact composition seam");
        assert_eq!(output.wrapper_site, surface_site(*wrapper_id));
        assert_eq!(output.wrapper_range, wrapper.range);
        super::assert_source_parenthesized_two_edge_local_mode_reserved_variable_equality_output(
            &output,
        )
        .expect("Task 246 real AST should preserve every checked payload invariant");
        assert_eq!(
            output.wrapper_side,
            super::SourceParenthesizedOperandSide::Left
        );
        assert_eq!(output.formula.payload.left_lookup_ordinal, 1);
        assert_eq!(output.formula.payload.right_lookup_ordinal, 2);
        assert_eq!(output.formula.left_binding, BindingId::new(0));
        assert_eq!(output.formula.right_binding, BindingId::new(0));
        assert_eq!(output.formula.payload.reserve.mode_expansions.len(), 3);
        assert_eq!(output.formula.term_formula.terms().len(), 2);
        assert_eq!(output.formula.term_formula.type_entries().len(), 6);
        assert_eq!(output.formula.term_formula.normalized_types().len(), 1);
        for input in [
            &output.formula.left_result_input,
            &output.formula.right_result_input,
            output.formula.left_expected_input.as_ref().unwrap(),
            output.formula.right_expected_input.as_ref().unwrap(),
        ] {
            assert_eq!(input.spelling, "OuterTwoEdgeModeEquality");
            assert!(matches!(input.head, TypeHeadInput::Symbol(_)));
        }
        let (_, normalized) = output
            .formula
            .term_formula
            .normalized_types()
            .iter()
            .next()
            .expect("Task 246 normalized set identity should exist");
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_eq!(normalized.source.spelling, "set");
        let (_, formula) = output
            .formula
            .term_formula
            .formulas()
            .iter()
            .next()
            .expect("Task 246 checked equality should exist");
        assert_eq!(formula.kind, FormulaKind::Equality);
        assert_eq!(formula.status, FormulaStatus::Checked);
        assert_eq!(formula.expected_types.len(), 2);
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
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
    }

    #[test]
    fn parenthesized_heterogeneous_reserve_membership_route_preserves_real_imported_mode_gap() {
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
        let plan = build_test_plan(&config).expect("Task 244 imported guard plan should build");
        let (ordinal, case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| case.id.0 == "fail_type_elaboration_imported_mode_gap_001")
            .expect("the real imported mode gap fixture should remain active");
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .expect("the imported mode gap fixture should run through the real frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend
            .ast
            .expect("the imported mode gap fixture should produce a real AST");
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty());
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        assert!(
            super::extract_source_parenthesized_heterogeneous_reserve_membership(
                &ast,
                resolver.module.clone(),
                &symbols,
            )
            .is_none()
        );
        assert_eq!(
            source_type_elaboration_detail_keys(&ast, resolver.module, &symbols),
            vec![
                "type_elaboration.checker.checker.type.external.mode_expansion_payload".to_owned(),
                "type_elaboration.checker.checker.type.recovery".to_owned(),
            ]
        );
    }

    #[test]
    fn parenthesized_reserved_variable_equality_route_isolated_from_all_prior_binary_owners() {
        type PriorExtractor = fn(
            &SurfaceAst,
            ResolverModuleId,
            &SymbolEnv,
        ) -> Option<super::SourceReservedVariableBinaryFormula>;
        macro_rules! prior_extractors {
            ($($extractor:path),+ $(,)?) => {
                [$(
                    (stringify!($extractor), $extractor as PriorExtractor),
                )+]
            };
        }
        let prior_extractors: [(&str, PriorExtractor); 52] = prior_extractors![
            super::extract_source_reserved_variable_equality,
            super::extract_source_reserved_object_variable_equality,
            super::extract_source_distinct_reserved_object_variable_equality,
            super::extract_source_distinct_reserved_object_variable_inequality,
            super::extract_source_reserved_object_variable_inequality,
            super::extract_source_distinct_reserved_variable_equality,
            super::extract_source_distinct_reserved_variable_membership,
            super::extract_source_distinct_reserved_variable_inequality,
            super::extract_source_heterogeneous_reserve_membership,
            super::extract_source_local_mode_reserved_variable_membership,
            super::extract_source_chained_local_mode_reserved_variable_membership,
            super::extract_source_two_edge_local_mode_reserved_variable_membership,
            super::extract_source_three_edge_local_mode_reserved_variable_membership,
            super::extract_source_four_edge_local_mode_reserved_variable_membership,
            super::extract_source_four_edge_local_object_mode_reserved_variable_membership,
            super::extract_source_three_edge_local_object_mode_reserved_variable_membership,
            super::extract_source_two_edge_local_object_mode_reserved_variable_membership,
            super::extract_source_chained_local_object_mode_reserved_variable_membership,
            super::extract_source_local_object_mode_reserved_variable_membership,
            super::extract_source_local_mode_reserved_variable_equality,
            super::extract_source_local_mode_reserved_variable_inequality,
            super::extract_source_local_object_mode_reserved_variable_inequality,
            super::extract_source_chained_local_mode_reserved_variable_equality,
            super::extract_source_two_edge_local_mode_reserved_variable_equality,
            super::extract_source_three_edge_local_mode_reserved_variable_equality,
            super::extract_source_four_edge_local_mode_reserved_variable_equality,
            super::extract_source_local_mode_long_chain_reserved_variable_equality,
            super::extract_source_local_object_mode_long_chain_reserved_variable_equality,
            super::extract_source_local_object_mode_long_chain_reserved_variable_inequality,
            super::extract_source_local_object_mode_long_chain_reserved_variable_membership,
            super::extract_source_local_mode_long_chain_reserved_variable_inequality,
            super::extract_source_local_mode_long_chain_reserved_variable_membership,
            super::extract_source_four_edge_local_mode_reserved_variable_inequality,
            super::extract_source_four_edge_local_object_mode_reserved_variable_equality,
            super::extract_source_four_edge_local_object_mode_reserved_variable_inequality,
            super::extract_source_three_edge_local_object_mode_reserved_variable_equality,
            super::extract_source_three_edge_local_mode_reserved_variable_inequality,
            super::extract_source_three_edge_local_object_mode_reserved_variable_inequality,
            super::extract_source_two_edge_local_mode_reserved_variable_inequality,
            super::extract_source_two_edge_local_object_mode_reserved_variable_inequality,
            super::extract_source_two_edge_local_object_mode_reserved_variable_equality,
            super::extract_source_chained_local_mode_reserved_variable_inequality,
            super::extract_source_chained_local_object_mode_reserved_variable_equality,
            super::extract_source_chained_local_object_mode_reserved_variable_inequality,
            super::extract_source_local_object_mode_reserved_variable_equality,
            super::extract_source_multiple_reserve_declaration_equality,
            super::extract_source_multiple_object_reserve_declaration_equality,
            super::extract_source_multiple_object_reserve_declaration_inequality,
            super::extract_source_multiple_reserve_declaration_inequality,
            super::extract_source_multiple_reserve_declaration_membership,
            super::extract_source_reserved_variable_membership,
            super::extract_source_reserved_variable_inequality,
        ];

        let source_id = source_id(223);
        let task_223_module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("parenthesized_reserved_variable_equality_isolation"),
        );
        let task_223_symbols = SymbolEnv::new(task_223_module.clone(), SymbolEnvIndexes::default());
        let task_223_ast = reserve_then_parenthesized_identifier_binary_theorem_ast(
            source_id,
            vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
            ParenthesizedIdentifierBinaryTheoremSpec {
                status: None,
                label: "ParenthesizedReservedVariableEqualityPayloadBoundary",
                left: ParenthesizedIdentifierOperandShape::Identifier {
                    spelling: "x",
                    depth: 1,
                    recovered: false,
                    open: "(",
                    close: ")",
                },
                operator: "=",
                right: ParenthesizedIdentifierOperandShape::Direct("x"),
                recovered_label: false,
            },
        );
        assert!(
            super::extract_source_parenthesized_reserved_variable_equality(
                &task_223_ast,
                task_223_module.clone(),
                &task_223_symbols,
            )
            .is_some()
        );
        for (name, extractor) in prior_extractors {
            assert!(
                extractor(&task_223_ast, task_223_module.clone(), &task_223_symbols,).is_none(),
                "Task 223 source must not be owned by {name}",
            );
        }

        struct PriorOwnerFixture {
            id: String,
            ast: SurfaceAst,
            module: ResolverModuleId,
            symbols: SymbolEnv,
        }
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
        let plan = build_test_plan(&config).expect("Task 223 isolation plan should build");
        let mut candidates = Vec::new();
        for (ordinal, case) in active_type_elaboration_cases(&plan).enumerate() {
            let id = case.id.0.as_str();
            if id == "pass_type_elaboration_parenthesized_reserved_variable_equality_001"
                || !(id.contains("equality")
                    || id.contains("inequality")
                    || id.contains("membership"))
            {
                continue;
            }
            let frontend = run_frontend(&workspace_root, case, ordinal)
                .expect("prior binary owner fixture should run through the real frontend");
            assert!(
                frontend.diagnostics.is_empty(),
                "unexpected frontend diagnostics for {id}"
            );
            let ast = frontend
                .ast
                .expect("prior binary owner fixture should produce an AST");
            let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
            assert!(
                resolver.detail_keys.is_empty(),
                "unexpected resolver diagnostics for {id}"
            );
            let module = resolver.module;
            let symbols = augment_type_elaboration_import_summaries(&ast, &module, resolver.env);
            candidates.push(PriorOwnerFixture {
                id: id.to_owned(),
                ast,
                module,
                symbols,
            });
        }

        let mut matched_fixture_ids = BTreeSet::new();
        for (name, extractor) in prior_extractors {
            let matches = candidates
                .iter()
                .filter(|candidate| {
                    extractor(&candidate.ast, candidate.module.clone(), &candidate.symbols)
                        .is_some()
                })
                .collect::<Vec<_>>();
            assert_eq!(
                matches.len(),
                1,
                "{name} should retain exactly one real active owner fixture; matches={:?}",
                matches
                    .iter()
                    .map(|candidate| candidate.id.as_str())
                    .collect::<Vec<_>>()
            );
            let owner = matches[0];
            assert!(matched_fixture_ids.insert(owner.id.clone()));
            assert!(
                super::extract_source_parenthesized_reserved_variable_equality(
                    &owner.ast,
                    owner.module.clone(),
                    &owner.symbols,
                )
                .is_none(),
                "prior owner {} must not be captured by Task 223",
                owner.id,
            );
        }
        assert_eq!(matched_fixture_ids.len(), 52);
    }

    #[test]
    fn parenthesized_reserved_object_variable_equality_route_isolated_from_all_prior_binary_owners()
    {
        type PriorExtractor = fn(
            &SurfaceAst,
            ResolverModuleId,
            &SymbolEnv,
        ) -> Option<super::SourceReservedVariableBinaryFormula>;
        macro_rules! prior_extractors {
            ($($extractor:path),+ $(,)?) => {
                [$(
                    (stringify!($extractor), $extractor as PriorExtractor),
                )+]
            };
        }
        let prior_extractors: [(&str, PriorExtractor); 52] = prior_extractors![
            super::extract_source_reserved_variable_equality,
            super::extract_source_reserved_object_variable_equality,
            super::extract_source_distinct_reserved_object_variable_equality,
            super::extract_source_distinct_reserved_object_variable_inequality,
            super::extract_source_reserved_object_variable_inequality,
            super::extract_source_distinct_reserved_variable_equality,
            super::extract_source_distinct_reserved_variable_membership,
            super::extract_source_distinct_reserved_variable_inequality,
            super::extract_source_heterogeneous_reserve_membership,
            super::extract_source_local_mode_reserved_variable_membership,
            super::extract_source_chained_local_mode_reserved_variable_membership,
            super::extract_source_two_edge_local_mode_reserved_variable_membership,
            super::extract_source_three_edge_local_mode_reserved_variable_membership,
            super::extract_source_four_edge_local_mode_reserved_variable_membership,
            super::extract_source_four_edge_local_object_mode_reserved_variable_membership,
            super::extract_source_three_edge_local_object_mode_reserved_variable_membership,
            super::extract_source_two_edge_local_object_mode_reserved_variable_membership,
            super::extract_source_chained_local_object_mode_reserved_variable_membership,
            super::extract_source_local_object_mode_reserved_variable_membership,
            super::extract_source_local_mode_reserved_variable_equality,
            super::extract_source_local_mode_reserved_variable_inequality,
            super::extract_source_local_object_mode_reserved_variable_inequality,
            super::extract_source_chained_local_mode_reserved_variable_equality,
            super::extract_source_two_edge_local_mode_reserved_variable_equality,
            super::extract_source_three_edge_local_mode_reserved_variable_equality,
            super::extract_source_four_edge_local_mode_reserved_variable_equality,
            super::extract_source_local_mode_long_chain_reserved_variable_equality,
            super::extract_source_local_object_mode_long_chain_reserved_variable_equality,
            super::extract_source_local_object_mode_long_chain_reserved_variable_inequality,
            super::extract_source_local_object_mode_long_chain_reserved_variable_membership,
            super::extract_source_local_mode_long_chain_reserved_variable_inequality,
            super::extract_source_local_mode_long_chain_reserved_variable_membership,
            super::extract_source_four_edge_local_mode_reserved_variable_inequality,
            super::extract_source_four_edge_local_object_mode_reserved_variable_equality,
            super::extract_source_four_edge_local_object_mode_reserved_variable_inequality,
            super::extract_source_three_edge_local_object_mode_reserved_variable_equality,
            super::extract_source_three_edge_local_mode_reserved_variable_inequality,
            super::extract_source_three_edge_local_object_mode_reserved_variable_inequality,
            super::extract_source_two_edge_local_mode_reserved_variable_inequality,
            super::extract_source_two_edge_local_object_mode_reserved_variable_inequality,
            super::extract_source_two_edge_local_object_mode_reserved_variable_equality,
            super::extract_source_chained_local_mode_reserved_variable_inequality,
            super::extract_source_chained_local_object_mode_reserved_variable_equality,
            super::extract_source_chained_local_object_mode_reserved_variable_inequality,
            super::extract_source_local_object_mode_reserved_variable_equality,
            super::extract_source_multiple_reserve_declaration_equality,
            super::extract_source_multiple_object_reserve_declaration_equality,
            super::extract_source_multiple_object_reserve_declaration_inequality,
            super::extract_source_multiple_reserve_declaration_inequality,
            super::extract_source_multiple_reserve_declaration_membership,
            super::extract_source_reserved_variable_membership,
            super::extract_source_reserved_variable_inequality,
        ];

        let source_id = source_id(233);
        let task_233_module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("parenthesized_reserved_object_variable_equality_isolation"),
        );
        let task_233_symbols = SymbolEnv::new(task_233_module.clone(), SymbolEnvIndexes::default());
        let task_233_ast = reserve_then_parenthesized_identifier_binary_theorem_ast(
            source_id,
            vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("object"))],
            ParenthesizedIdentifierBinaryTheoremSpec {
                status: None,
                label: "ParenthesizedReservedObjectVariableEqualityPayloadBoundary",
                left: ParenthesizedIdentifierOperandShape::Identifier {
                    spelling: "x",
                    depth: 1,
                    recovered: false,
                    open: "(",
                    close: ")",
                },
                operator: "=",
                right: ParenthesizedIdentifierOperandShape::Direct("x"),
                recovered_label: false,
            },
        );
        assert!(
            super::extract_source_parenthesized_reserved_object_variable_equality(
                &task_233_ast,
                task_233_module.clone(),
                &task_233_symbols,
            )
            .is_some()
        );
        assert!(
            super::extract_source_parenthesized_reserved_variable_equality(
                &task_233_ast,
                task_233_module.clone(),
                &task_233_symbols,
            )
            .is_none()
        );
        for (name, extractor) in prior_extractors {
            assert!(
                extractor(&task_233_ast, task_233_module.clone(), &task_233_symbols,).is_none(),
                "Task 233 source must not be owned by {name}",
            );
        }

        struct PriorOwnerFixture {
            id: String,
            ast: SurfaceAst,
            module: ResolverModuleId,
            symbols: SymbolEnv,
        }
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
        let plan = build_test_plan(&config).expect("Task 233 isolation plan should build");
        let mut candidates = Vec::new();
        for (ordinal, case) in active_type_elaboration_cases(&plan).enumerate() {
            let id = case.id.0.as_str();
            if id == "pass_type_elaboration_parenthesized_reserved_object_variable_equality_001"
                || !(id.contains("equality")
                    || id.contains("inequality")
                    || id.contains("membership"))
            {
                continue;
            }
            let frontend = run_frontend(&workspace_root, case, ordinal)
                .expect("prior binary owner fixture should run through the real frontend");
            assert!(
                frontend.diagnostics.is_empty(),
                "unexpected frontend diagnostics for {id}"
            );
            let ast = frontend
                .ast
                .expect("prior binary owner fixture should produce an AST");
            let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
            assert!(
                resolver.detail_keys.is_empty(),
                "unexpected resolver diagnostics for {id}"
            );
            let module = resolver.module;
            let symbols = augment_type_elaboration_import_summaries(&ast, &module, resolver.env);
            candidates.push(PriorOwnerFixture {
                id: id.to_owned(),
                ast,
                module,
                symbols,
            });
        }

        let mut matched_fixture_ids = BTreeSet::new();
        for (name, extractor) in prior_extractors {
            let matches = candidates
                .iter()
                .filter(|candidate| {
                    extractor(&candidate.ast, candidate.module.clone(), &candidate.symbols)
                        .is_some()
                })
                .collect::<Vec<_>>();
            assert_eq!(
                matches.len(),
                1,
                "{name} should retain exactly one real active owner fixture; matches={:?}",
                matches
                    .iter()
                    .map(|candidate| candidate.id.as_str())
                    .collect::<Vec<_>>()
            );
            let owner = matches[0];
            assert!(matched_fixture_ids.insert(owner.id.clone()));
            assert!(
                super::extract_source_parenthesized_reserved_object_variable_equality(
                    &owner.ast,
                    owner.module.clone(),
                    &owner.symbols,
                )
                .is_none(),
                "prior owner {} must not be captured by Task 233",
                owner.id,
            );
        }
        let task_223_owner = candidates
            .iter()
            .find(|candidate| {
                candidate.id == "pass_type_elaboration_parenthesized_reserved_variable_equality_001"
            })
            .expect("Task 223 active owner should remain in the prior candidate set");
        assert!(
            super::extract_source_parenthesized_reserved_variable_equality(
                &task_223_owner.ast,
                task_223_owner.module.clone(),
                &task_223_owner.symbols,
            )
            .is_some()
        );
        assert!(
            super::extract_source_parenthesized_reserved_object_variable_equality(
                &task_223_owner.ast,
                task_223_owner.module.clone(),
                &task_223_owner.symbols,
            )
            .is_none()
        );
        assert!(matched_fixture_ids.insert(task_223_owner.id.clone()));
        assert_eq!(matched_fixture_ids.len(), 53);
    }

    #[test]
    fn parenthesized_reserved_variable_inequality_route_isolated_from_all_prior_binary_owners() {
        type PriorExtractor = fn(
            &SurfaceAst,
            ResolverModuleId,
            &SymbolEnv,
        ) -> Option<super::SourceReservedVariableBinaryFormula>;
        macro_rules! prior_extractors {
            ($($extractor:path),+ $(,)?) => {
                [$(
                    (stringify!($extractor), $extractor as PriorExtractor),
                )+]
            };
        }
        let prior_extractors: [(&str, PriorExtractor); 52] = prior_extractors![
            super::extract_source_reserved_variable_equality,
            super::extract_source_reserved_object_variable_equality,
            super::extract_source_distinct_reserved_object_variable_equality,
            super::extract_source_distinct_reserved_object_variable_inequality,
            super::extract_source_reserved_object_variable_inequality,
            super::extract_source_distinct_reserved_variable_equality,
            super::extract_source_distinct_reserved_variable_membership,
            super::extract_source_distinct_reserved_variable_inequality,
            super::extract_source_heterogeneous_reserve_membership,
            super::extract_source_local_mode_reserved_variable_membership,
            super::extract_source_chained_local_mode_reserved_variable_membership,
            super::extract_source_two_edge_local_mode_reserved_variable_membership,
            super::extract_source_three_edge_local_mode_reserved_variable_membership,
            super::extract_source_four_edge_local_mode_reserved_variable_membership,
            super::extract_source_four_edge_local_object_mode_reserved_variable_membership,
            super::extract_source_three_edge_local_object_mode_reserved_variable_membership,
            super::extract_source_two_edge_local_object_mode_reserved_variable_membership,
            super::extract_source_chained_local_object_mode_reserved_variable_membership,
            super::extract_source_local_object_mode_reserved_variable_membership,
            super::extract_source_local_mode_reserved_variable_equality,
            super::extract_source_local_mode_reserved_variable_inequality,
            super::extract_source_local_object_mode_reserved_variable_inequality,
            super::extract_source_chained_local_mode_reserved_variable_equality,
            super::extract_source_two_edge_local_mode_reserved_variable_equality,
            super::extract_source_three_edge_local_mode_reserved_variable_equality,
            super::extract_source_four_edge_local_mode_reserved_variable_equality,
            super::extract_source_local_mode_long_chain_reserved_variable_equality,
            super::extract_source_local_object_mode_long_chain_reserved_variable_equality,
            super::extract_source_local_object_mode_long_chain_reserved_variable_inequality,
            super::extract_source_local_object_mode_long_chain_reserved_variable_membership,
            super::extract_source_local_mode_long_chain_reserved_variable_inequality,
            super::extract_source_local_mode_long_chain_reserved_variable_membership,
            super::extract_source_four_edge_local_mode_reserved_variable_inequality,
            super::extract_source_four_edge_local_object_mode_reserved_variable_equality,
            super::extract_source_four_edge_local_object_mode_reserved_variable_inequality,
            super::extract_source_three_edge_local_object_mode_reserved_variable_equality,
            super::extract_source_three_edge_local_mode_reserved_variable_inequality,
            super::extract_source_three_edge_local_object_mode_reserved_variable_inequality,
            super::extract_source_two_edge_local_mode_reserved_variable_inequality,
            super::extract_source_two_edge_local_object_mode_reserved_variable_inequality,
            super::extract_source_two_edge_local_object_mode_reserved_variable_equality,
            super::extract_source_chained_local_mode_reserved_variable_inequality,
            super::extract_source_chained_local_object_mode_reserved_variable_equality,
            super::extract_source_chained_local_object_mode_reserved_variable_inequality,
            super::extract_source_local_object_mode_reserved_variable_equality,
            super::extract_source_multiple_reserve_declaration_equality,
            super::extract_source_multiple_object_reserve_declaration_equality,
            super::extract_source_multiple_object_reserve_declaration_inequality,
            super::extract_source_multiple_reserve_declaration_inequality,
            super::extract_source_multiple_reserve_declaration_membership,
            super::extract_source_reserved_variable_membership,
            super::extract_source_reserved_variable_inequality,
        ];

        let source_id = source_id(241);
        let task_241_module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("parenthesized_reserved_variable_inequality_isolation"),
        );
        let task_241_symbols = SymbolEnv::new(task_241_module.clone(), SymbolEnvIndexes::default());
        let task_241_ast = reserve_then_parenthesized_identifier_binary_theorem_ast(
            source_id,
            vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
            ParenthesizedIdentifierBinaryTheoremSpec {
                status: None,
                label: "ParenthesizedReservedVariableInequalityPayloadBoundary",
                left: ParenthesizedIdentifierOperandShape::Identifier {
                    spelling: "x",
                    depth: 1,
                    recovered: false,
                    open: "(",
                    close: ")",
                },
                operator: "<>",
                right: ParenthesizedIdentifierOperandShape::Direct("x"),
                recovered_label: false,
            },
        );
        assert!(
            super::extract_source_parenthesized_reserved_variable_inequality(
                &task_241_ast,
                task_241_module.clone(),
                &task_241_symbols,
            )
            .is_some()
        );
        assert!(
            super::extract_source_parenthesized_reserved_variable_equality(
                &task_241_ast,
                task_241_module.clone(),
                &task_241_symbols,
            )
            .is_none()
        );
        assert!(
            super::extract_source_parenthesized_reserved_object_variable_equality(
                &task_241_ast,
                task_241_module.clone(),
                &task_241_symbols,
            )
            .is_none()
        );
        for (name, extractor) in prior_extractors {
            assert!(
                extractor(&task_241_ast, task_241_module.clone(), &task_241_symbols,).is_none(),
                "Task 241 source must not be owned by {name}",
            );
        }

        struct PriorOwnerFixture {
            id: String,
            ast: SurfaceAst,
            module: ResolverModuleId,
            symbols: SymbolEnv,
        }
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
        let plan = build_test_plan(&config).expect("Task 241 isolation plan should build");
        let mut candidates = Vec::new();
        for (ordinal, case) in active_type_elaboration_cases(&plan).enumerate() {
            let id = case.id.0.as_str();
            if id == "pass_type_elaboration_parenthesized_reserved_variable_inequality_001"
                || !(id.contains("equality")
                    || id.contains("inequality")
                    || id.contains("membership"))
            {
                continue;
            }
            let frontend = run_frontend(&workspace_root, case, ordinal)
                .expect("prior binary owner fixture should run through the real frontend");
            assert!(
                frontend.diagnostics.is_empty(),
                "unexpected frontend diagnostics for {id}"
            );
            let ast = frontend
                .ast
                .expect("prior binary owner fixture should produce an AST");
            let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
            assert!(
                resolver.detail_keys.is_empty(),
                "unexpected resolver diagnostics for {id}"
            );
            let module = resolver.module;
            let symbols = augment_type_elaboration_import_summaries(&ast, &module, resolver.env);
            candidates.push(PriorOwnerFixture {
                id: id.to_owned(),
                ast,
                module,
                symbols,
            });
        }

        let mut matched_fixture_ids = BTreeSet::new();
        for (name, extractor) in prior_extractors {
            let matches = candidates
                .iter()
                .filter(|candidate| {
                    extractor(&candidate.ast, candidate.module.clone(), &candidate.symbols)
                        .is_some()
                })
                .collect::<Vec<_>>();
            assert_eq!(
                matches.len(),
                1,
                "{name} should retain exactly one real active owner fixture; matches={:?}",
                matches
                    .iter()
                    .map(|candidate| candidate.id.as_str())
                    .collect::<Vec<_>>()
            );
            let owner = matches[0];
            assert!(matched_fixture_ids.insert(owner.id.clone()));
            assert!(
                super::extract_source_parenthesized_reserved_variable_inequality(
                    &owner.ast,
                    owner.module.clone(),
                    &owner.symbols,
                )
                .is_none(),
                "prior owner {} must not be captured by Task 241",
                owner.id,
            );
        }
        for (id, extractor) in [
            (
                "pass_type_elaboration_parenthesized_reserved_variable_equality_001",
                super::extract_source_parenthesized_reserved_variable_equality
                    as fn(
                        &SurfaceAst,
                        ResolverModuleId,
                        &SymbolEnv,
                    )
                        -> Option<super::SourceParenthesizedReservedVariableBinaryFormula>,
            ),
            (
                "pass_type_elaboration_parenthesized_reserved_object_variable_equality_001",
                super::extract_source_parenthesized_reserved_object_variable_equality
                    as fn(
                        &SurfaceAst,
                        ResolverModuleId,
                        &SymbolEnv,
                    )
                        -> Option<super::SourceParenthesizedReservedVariableBinaryFormula>,
            ),
        ] {
            let owner = candidates
                .iter()
                .find(|candidate| candidate.id == id)
                .unwrap_or_else(|| panic!("prior parenthesized owner {id} should remain active"));
            assert!(extractor(&owner.ast, owner.module.clone(), &owner.symbols).is_some());
            assert!(
                super::extract_source_parenthesized_reserved_variable_inequality(
                    &owner.ast,
                    owner.module.clone(),
                    &owner.symbols,
                )
                .is_none()
            );
            assert!(matched_fixture_ids.insert(owner.id.clone()));
        }
        assert_eq!(matched_fixture_ids.len(), 54);
    }

    #[test]
    fn parenthesized_reserved_object_variable_inequality_route_isolated_from_all_prior_binary_owners()
     {
        type PriorExtractor = fn(
            &SurfaceAst,
            ResolverModuleId,
            &SymbolEnv,
        ) -> Option<super::SourceReservedVariableBinaryFormula>;
        macro_rules! prior_extractors {
            ($($extractor:path),+ $(,)?) => {
                [$(
                    (stringify!($extractor), $extractor as PriorExtractor),
                )+]
            };
        }
        let prior_extractors: [(&str, PriorExtractor); 52] = prior_extractors![
            super::extract_source_reserved_variable_equality,
            super::extract_source_reserved_object_variable_equality,
            super::extract_source_distinct_reserved_object_variable_equality,
            super::extract_source_distinct_reserved_object_variable_inequality,
            super::extract_source_reserved_object_variable_inequality,
            super::extract_source_distinct_reserved_variable_equality,
            super::extract_source_distinct_reserved_variable_membership,
            super::extract_source_distinct_reserved_variable_inequality,
            super::extract_source_heterogeneous_reserve_membership,
            super::extract_source_local_mode_reserved_variable_membership,
            super::extract_source_chained_local_mode_reserved_variable_membership,
            super::extract_source_two_edge_local_mode_reserved_variable_membership,
            super::extract_source_three_edge_local_mode_reserved_variable_membership,
            super::extract_source_four_edge_local_mode_reserved_variable_membership,
            super::extract_source_four_edge_local_object_mode_reserved_variable_membership,
            super::extract_source_three_edge_local_object_mode_reserved_variable_membership,
            super::extract_source_two_edge_local_object_mode_reserved_variable_membership,
            super::extract_source_chained_local_object_mode_reserved_variable_membership,
            super::extract_source_local_object_mode_reserved_variable_membership,
            super::extract_source_local_mode_reserved_variable_equality,
            super::extract_source_local_mode_reserved_variable_inequality,
            super::extract_source_local_object_mode_reserved_variable_inequality,
            super::extract_source_chained_local_mode_reserved_variable_equality,
            super::extract_source_two_edge_local_mode_reserved_variable_equality,
            super::extract_source_three_edge_local_mode_reserved_variable_equality,
            super::extract_source_four_edge_local_mode_reserved_variable_equality,
            super::extract_source_local_mode_long_chain_reserved_variable_equality,
            super::extract_source_local_object_mode_long_chain_reserved_variable_equality,
            super::extract_source_local_object_mode_long_chain_reserved_variable_inequality,
            super::extract_source_local_object_mode_long_chain_reserved_variable_membership,
            super::extract_source_local_mode_long_chain_reserved_variable_inequality,
            super::extract_source_local_mode_long_chain_reserved_variable_membership,
            super::extract_source_four_edge_local_mode_reserved_variable_inequality,
            super::extract_source_four_edge_local_object_mode_reserved_variable_equality,
            super::extract_source_four_edge_local_object_mode_reserved_variable_inequality,
            super::extract_source_three_edge_local_object_mode_reserved_variable_equality,
            super::extract_source_three_edge_local_mode_reserved_variable_inequality,
            super::extract_source_three_edge_local_object_mode_reserved_variable_inequality,
            super::extract_source_two_edge_local_mode_reserved_variable_inequality,
            super::extract_source_two_edge_local_object_mode_reserved_variable_inequality,
            super::extract_source_two_edge_local_object_mode_reserved_variable_equality,
            super::extract_source_chained_local_mode_reserved_variable_inequality,
            super::extract_source_chained_local_object_mode_reserved_variable_equality,
            super::extract_source_chained_local_object_mode_reserved_variable_inequality,
            super::extract_source_local_object_mode_reserved_variable_equality,
            super::extract_source_multiple_reserve_declaration_equality,
            super::extract_source_multiple_object_reserve_declaration_equality,
            super::extract_source_multiple_object_reserve_declaration_inequality,
            super::extract_source_multiple_reserve_declaration_inequality,
            super::extract_source_multiple_reserve_declaration_membership,
            super::extract_source_reserved_variable_membership,
            super::extract_source_reserved_variable_inequality,
        ];

        let source_id = source_id(242);
        let task_242_module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("parenthesized_reserved_object_variable_inequality_isolation"),
        );
        let task_242_symbols = SymbolEnv::new(task_242_module.clone(), SymbolEnvIndexes::default());
        let task_242_ast = reserve_then_parenthesized_identifier_binary_theorem_ast(
            source_id,
            vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("object"))],
            ParenthesizedIdentifierBinaryTheoremSpec {
                status: None,
                label: "ParenthesizedReservedObjectVariableInequalityPayloadBoundary",
                left: ParenthesizedIdentifierOperandShape::Identifier {
                    spelling: "x",
                    depth: 1,
                    recovered: false,
                    open: "(",
                    close: ")",
                },
                operator: "<>",
                right: ParenthesizedIdentifierOperandShape::Direct("x"),
                recovered_label: false,
            },
        );
        assert!(
            super::extract_source_parenthesized_reserved_object_variable_inequality(
                &task_242_ast,
                task_242_module.clone(),
                &task_242_symbols,
            )
            .is_some()
        );
        for (name, extractor) in prior_extractors {
            assert!(
                extractor(&task_242_ast, task_242_module.clone(), &task_242_symbols,).is_none(),
                "Task 242 source must not be owned by {name}",
            );
        }
        for (name, extractor) in [
            (
                "Task 223",
                super::extract_source_parenthesized_reserved_variable_equality
                    as fn(
                        &SurfaceAst,
                        ResolverModuleId,
                        &SymbolEnv,
                    )
                        -> Option<super::SourceParenthesizedReservedVariableBinaryFormula>,
            ),
            (
                "Task 233",
                super::extract_source_parenthesized_reserved_object_variable_equality
                    as fn(
                        &SurfaceAst,
                        ResolverModuleId,
                        &SymbolEnv,
                    )
                        -> Option<super::SourceParenthesizedReservedVariableBinaryFormula>,
            ),
            (
                "Task 241",
                super::extract_source_parenthesized_reserved_variable_inequality
                    as fn(
                        &SurfaceAst,
                        ResolverModuleId,
                        &SymbolEnv,
                    )
                        -> Option<super::SourceParenthesizedReservedVariableBinaryFormula>,
            ),
        ] {
            assert!(
                extractor(&task_242_ast, task_242_module.clone(), &task_242_symbols,).is_none(),
                "Task 242 source must not be owned by {name}",
            );
        }

        struct PriorOwnerFixture {
            id: String,
            ast: SurfaceAst,
            module: ResolverModuleId,
            symbols: SymbolEnv,
        }
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
        let plan = build_test_plan(&config).expect("Task 242 isolation plan should build");
        let mut candidates = Vec::new();
        for (ordinal, case) in active_type_elaboration_cases(&plan).enumerate() {
            let id = case.id.0.as_str();
            if id == "pass_type_elaboration_parenthesized_reserved_object_variable_inequality_001"
                || !(id.contains("equality")
                    || id.contains("inequality")
                    || id.contains("membership"))
            {
                continue;
            }
            let frontend = run_frontend(&workspace_root, case, ordinal)
                .expect("prior binary owner fixture should run through the real frontend");
            assert!(
                frontend.diagnostics.is_empty(),
                "unexpected frontend diagnostics for {id}"
            );
            let ast = frontend
                .ast
                .expect("prior binary owner fixture should produce an AST");
            let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
            assert!(
                resolver.detail_keys.is_empty(),
                "unexpected resolver diagnostics for {id}"
            );
            let module = resolver.module;
            let symbols = augment_type_elaboration_import_summaries(&ast, &module, resolver.env);
            candidates.push(PriorOwnerFixture {
                id: id.to_owned(),
                ast,
                module,
                symbols,
            });
        }

        let mut matched_fixture_ids = BTreeSet::new();
        for (name, extractor) in prior_extractors {
            let matches = candidates
                .iter()
                .filter(|candidate| {
                    extractor(&candidate.ast, candidate.module.clone(), &candidate.symbols)
                        .is_some()
                })
                .collect::<Vec<_>>();
            assert_eq!(
                matches.len(),
                1,
                "{name} should retain exactly one real active owner fixture; matches={:?}",
                matches
                    .iter()
                    .map(|candidate| candidate.id.as_str())
                    .collect::<Vec<_>>()
            );
            let owner = matches[0];
            assert!(matched_fixture_ids.insert(owner.id.clone()));
            assert!(
                super::extract_source_parenthesized_reserved_object_variable_inequality(
                    &owner.ast,
                    owner.module.clone(),
                    &owner.symbols,
                )
                .is_none(),
                "prior owner {} must not be captured by Task 242",
                owner.id,
            );
        }
        for (id, extractor) in [
            (
                "pass_type_elaboration_parenthesized_reserved_variable_equality_001",
                super::extract_source_parenthesized_reserved_variable_equality
                    as fn(
                        &SurfaceAst,
                        ResolverModuleId,
                        &SymbolEnv,
                    )
                        -> Option<super::SourceParenthesizedReservedVariableBinaryFormula>,
            ),
            (
                "pass_type_elaboration_parenthesized_reserved_object_variable_equality_001",
                super::extract_source_parenthesized_reserved_object_variable_equality
                    as fn(
                        &SurfaceAst,
                        ResolverModuleId,
                        &SymbolEnv,
                    )
                        -> Option<super::SourceParenthesizedReservedVariableBinaryFormula>,
            ),
            (
                "pass_type_elaboration_parenthesized_reserved_variable_inequality_001",
                super::extract_source_parenthesized_reserved_variable_inequality
                    as fn(
                        &SurfaceAst,
                        ResolverModuleId,
                        &SymbolEnv,
                    )
                        -> Option<super::SourceParenthesizedReservedVariableBinaryFormula>,
            ),
        ] {
            let owner = candidates
                .iter()
                .find(|candidate| candidate.id == id)
                .unwrap_or_else(|| panic!("prior parenthesized owner {id} should remain active"));
            assert!(extractor(&owner.ast, owner.module.clone(), &owner.symbols).is_some());
            assert!(
                super::extract_source_parenthesized_reserved_object_variable_inequality(
                    &owner.ast,
                    owner.module.clone(),
                    &owner.symbols,
                )
                .is_none()
            );
            assert!(matched_fixture_ids.insert(owner.id.clone()));
        }
        assert_eq!(matched_fixture_ids.len(), 55);
    }

    #[test]
    fn parenthesized_reserved_variable_membership_route_isolated_from_all_prior_binary_owners() {
        type PriorExtractor = fn(
            &SurfaceAst,
            ResolverModuleId,
            &SymbolEnv,
        ) -> Option<super::SourceReservedVariableBinaryFormula>;
        macro_rules! prior_extractors {
            ($($extractor:path),+ $(,)?) => {
                [$(
                    (stringify!($extractor), $extractor as PriorExtractor),
                )+]
            };
        }
        let prior_extractors: [(&str, PriorExtractor); 52] = prior_extractors![
            super::extract_source_reserved_variable_equality,
            super::extract_source_reserved_object_variable_equality,
            super::extract_source_distinct_reserved_object_variable_equality,
            super::extract_source_distinct_reserved_object_variable_inequality,
            super::extract_source_reserved_object_variable_inequality,
            super::extract_source_distinct_reserved_variable_equality,
            super::extract_source_distinct_reserved_variable_membership,
            super::extract_source_distinct_reserved_variable_inequality,
            super::extract_source_heterogeneous_reserve_membership,
            super::extract_source_local_mode_reserved_variable_membership,
            super::extract_source_chained_local_mode_reserved_variable_membership,
            super::extract_source_two_edge_local_mode_reserved_variable_membership,
            super::extract_source_three_edge_local_mode_reserved_variable_membership,
            super::extract_source_four_edge_local_mode_reserved_variable_membership,
            super::extract_source_four_edge_local_object_mode_reserved_variable_membership,
            super::extract_source_three_edge_local_object_mode_reserved_variable_membership,
            super::extract_source_two_edge_local_object_mode_reserved_variable_membership,
            super::extract_source_chained_local_object_mode_reserved_variable_membership,
            super::extract_source_local_object_mode_reserved_variable_membership,
            super::extract_source_local_mode_reserved_variable_equality,
            super::extract_source_local_mode_reserved_variable_inequality,
            super::extract_source_local_object_mode_reserved_variable_inequality,
            super::extract_source_chained_local_mode_reserved_variable_equality,
            super::extract_source_two_edge_local_mode_reserved_variable_equality,
            super::extract_source_three_edge_local_mode_reserved_variable_equality,
            super::extract_source_four_edge_local_mode_reserved_variable_equality,
            super::extract_source_local_mode_long_chain_reserved_variable_equality,
            super::extract_source_local_object_mode_long_chain_reserved_variable_equality,
            super::extract_source_local_object_mode_long_chain_reserved_variable_inequality,
            super::extract_source_local_object_mode_long_chain_reserved_variable_membership,
            super::extract_source_local_mode_long_chain_reserved_variable_inequality,
            super::extract_source_local_mode_long_chain_reserved_variable_membership,
            super::extract_source_four_edge_local_mode_reserved_variable_inequality,
            super::extract_source_four_edge_local_object_mode_reserved_variable_equality,
            super::extract_source_four_edge_local_object_mode_reserved_variable_inequality,
            super::extract_source_three_edge_local_object_mode_reserved_variable_equality,
            super::extract_source_three_edge_local_mode_reserved_variable_inequality,
            super::extract_source_three_edge_local_object_mode_reserved_variable_inequality,
            super::extract_source_two_edge_local_mode_reserved_variable_inequality,
            super::extract_source_two_edge_local_object_mode_reserved_variable_inequality,
            super::extract_source_two_edge_local_object_mode_reserved_variable_equality,
            super::extract_source_chained_local_mode_reserved_variable_inequality,
            super::extract_source_chained_local_object_mode_reserved_variable_equality,
            super::extract_source_chained_local_object_mode_reserved_variable_inequality,
            super::extract_source_local_object_mode_reserved_variable_equality,
            super::extract_source_multiple_reserve_declaration_equality,
            super::extract_source_multiple_object_reserve_declaration_equality,
            super::extract_source_multiple_object_reserve_declaration_inequality,
            super::extract_source_multiple_reserve_declaration_inequality,
            super::extract_source_multiple_reserve_declaration_membership,
            super::extract_source_reserved_variable_membership,
            super::extract_source_reserved_variable_inequality,
        ];

        let source_id = source_id(243);
        let task_243_module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("parenthesized_reserved_variable_membership_isolation"),
        );
        let task_243_symbols = SymbolEnv::new(task_243_module.clone(), SymbolEnvIndexes::default());
        let task_243_ast = reserve_then_parenthesized_identifier_binary_theorem_ast(
            source_id,
            vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
            ParenthesizedIdentifierBinaryTheoremSpec {
                status: None,
                label: "ParenthesizedReservedVariableMembershipPayloadBoundary",
                left: ParenthesizedIdentifierOperandShape::Identifier {
                    spelling: "x",
                    depth: 1,
                    recovered: false,
                    open: "(",
                    close: ")",
                },
                operator: "in",
                right: ParenthesizedIdentifierOperandShape::Direct("x"),
                recovered_label: false,
            },
        );
        assert!(
            super::extract_source_parenthesized_reserved_variable_membership(
                &task_243_ast,
                task_243_module.clone(),
                &task_243_symbols,
            )
            .is_some()
        );
        for (name, extractor) in prior_extractors {
            assert!(
                extractor(&task_243_ast, task_243_module.clone(), &task_243_symbols).is_none(),
                "Task 243 source must not be owned by {name}",
            );
        }
        for (name, extractor) in [
            (
                "Task 223",
                super::extract_source_parenthesized_reserved_variable_equality
                    as fn(
                        &SurfaceAst,
                        ResolverModuleId,
                        &SymbolEnv,
                    )
                        -> Option<super::SourceParenthesizedReservedVariableBinaryFormula>,
            ),
            (
                "Task 233",
                super::extract_source_parenthesized_reserved_object_variable_equality
                    as fn(
                        &SurfaceAst,
                        ResolverModuleId,
                        &SymbolEnv,
                    )
                        -> Option<super::SourceParenthesizedReservedVariableBinaryFormula>,
            ),
            (
                "Task 241",
                super::extract_source_parenthesized_reserved_variable_inequality
                    as fn(
                        &SurfaceAst,
                        ResolverModuleId,
                        &SymbolEnv,
                    )
                        -> Option<super::SourceParenthesizedReservedVariableBinaryFormula>,
            ),
            (
                "Task 242",
                super::extract_source_parenthesized_reserved_object_variable_inequality
                    as fn(
                        &SurfaceAst,
                        ResolverModuleId,
                        &SymbolEnv,
                    )
                        -> Option<super::SourceParenthesizedReservedVariableBinaryFormula>,
            ),
        ] {
            assert!(
                extractor(&task_243_ast, task_243_module.clone(), &task_243_symbols).is_none(),
                "Task 243 source must not be owned by {name}",
            );
        }

        struct PriorOwnerFixture {
            id: String,
            ast: SurfaceAst,
            module: ResolverModuleId,
            symbols: SymbolEnv,
        }
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
        let plan = build_test_plan(&config).expect("Task 243 isolation plan should build");
        let mut candidates = Vec::new();
        for (ordinal, case) in active_type_elaboration_cases(&plan).enumerate() {
            let id = case.id.0.as_str();
            if id == "pass_type_elaboration_parenthesized_reserved_variable_membership_001"
                || !(id.contains("equality")
                    || id.contains("inequality")
                    || id.contains("membership"))
            {
                continue;
            }
            let frontend = run_frontend(&workspace_root, case, ordinal)
                .expect("prior binary owner fixture should run through the real frontend");
            assert!(
                frontend.diagnostics.is_empty(),
                "unexpected frontend diagnostics for {id}"
            );
            let ast = frontend
                .ast
                .expect("prior binary owner fixture should produce an AST");
            let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
            assert!(
                resolver.detail_keys.is_empty(),
                "unexpected resolver diagnostics for {id}"
            );
            let module = resolver.module;
            let symbols = augment_type_elaboration_import_summaries(&ast, &module, resolver.env);
            candidates.push(PriorOwnerFixture {
                id: id.to_owned(),
                ast,
                module,
                symbols,
            });
        }

        let mut matched_fixture_ids = BTreeSet::new();
        for (name, extractor) in prior_extractors {
            let matches = candidates
                .iter()
                .filter(|candidate| {
                    extractor(&candidate.ast, candidate.module.clone(), &candidate.symbols)
                        .is_some()
                })
                .collect::<Vec<_>>();
            assert_eq!(
                matches.len(),
                1,
                "{name} should retain exactly one real active owner fixture; matches={:?}",
                matches
                    .iter()
                    .map(|candidate| candidate.id.as_str())
                    .collect::<Vec<_>>()
            );
            let owner = matches[0];
            assert!(matched_fixture_ids.insert(owner.id.clone()));
            assert!(
                super::extract_source_parenthesized_reserved_variable_membership(
                    &owner.ast,
                    owner.module.clone(),
                    &owner.symbols,
                )
                .is_none(),
                "prior owner {} must not be captured by Task 243",
                owner.id,
            );
        }
        for (id, extractor) in [
            (
                "pass_type_elaboration_parenthesized_reserved_variable_equality_001",
                super::extract_source_parenthesized_reserved_variable_equality
                    as fn(
                        &SurfaceAst,
                        ResolverModuleId,
                        &SymbolEnv,
                    )
                        -> Option<super::SourceParenthesizedReservedVariableBinaryFormula>,
            ),
            (
                "pass_type_elaboration_parenthesized_reserved_object_variable_equality_001",
                super::extract_source_parenthesized_reserved_object_variable_equality
                    as fn(
                        &SurfaceAst,
                        ResolverModuleId,
                        &SymbolEnv,
                    )
                        -> Option<super::SourceParenthesizedReservedVariableBinaryFormula>,
            ),
            (
                "pass_type_elaboration_parenthesized_reserved_variable_inequality_001",
                super::extract_source_parenthesized_reserved_variable_inequality
                    as fn(
                        &SurfaceAst,
                        ResolverModuleId,
                        &SymbolEnv,
                    )
                        -> Option<super::SourceParenthesizedReservedVariableBinaryFormula>,
            ),
            (
                "pass_type_elaboration_parenthesized_reserved_object_variable_inequality_001",
                super::extract_source_parenthesized_reserved_object_variable_inequality
                    as fn(
                        &SurfaceAst,
                        ResolverModuleId,
                        &SymbolEnv,
                    )
                        -> Option<super::SourceParenthesizedReservedVariableBinaryFormula>,
            ),
        ] {
            let owner = candidates
                .iter()
                .find(|candidate| candidate.id == id)
                .unwrap_or_else(|| panic!("prior parenthesized owner {id} should remain active"));
            assert!(extractor(&owner.ast, owner.module.clone(), &owner.symbols).is_some());
            assert!(
                super::extract_source_parenthesized_reserved_variable_membership(
                    &owner.ast,
                    owner.module.clone(),
                    &owner.symbols,
                )
                .is_none()
            );
            assert!(matched_fixture_ids.insert(owner.id.clone()));
        }
        assert_eq!(matched_fixture_ids.len(), 56);
    }

    #[test]
    fn parenthesized_heterogeneous_reserve_membership_route_isolated_from_all_prior_binary_owners()
    {
        type PriorExtractor = fn(
            &SurfaceAst,
            ResolverModuleId,
            &SymbolEnv,
        ) -> Option<super::SourceReservedVariableBinaryFormula>;
        macro_rules! prior_extractors {
            ($($extractor:path),+ $(,)?) => {
                [$(
                    (stringify!($extractor), $extractor as PriorExtractor),
                )+]
            };
        }
        let prior_extractors: [(&str, PriorExtractor); 52] = prior_extractors![
            super::extract_source_reserved_variable_equality,
            super::extract_source_reserved_object_variable_equality,
            super::extract_source_distinct_reserved_object_variable_equality,
            super::extract_source_distinct_reserved_object_variable_inequality,
            super::extract_source_reserved_object_variable_inequality,
            super::extract_source_distinct_reserved_variable_equality,
            super::extract_source_distinct_reserved_variable_membership,
            super::extract_source_distinct_reserved_variable_inequality,
            super::extract_source_heterogeneous_reserve_membership,
            super::extract_source_local_mode_reserved_variable_membership,
            super::extract_source_chained_local_mode_reserved_variable_membership,
            super::extract_source_two_edge_local_mode_reserved_variable_membership,
            super::extract_source_three_edge_local_mode_reserved_variable_membership,
            super::extract_source_four_edge_local_mode_reserved_variable_membership,
            super::extract_source_four_edge_local_object_mode_reserved_variable_membership,
            super::extract_source_three_edge_local_object_mode_reserved_variable_membership,
            super::extract_source_two_edge_local_object_mode_reserved_variable_membership,
            super::extract_source_chained_local_object_mode_reserved_variable_membership,
            super::extract_source_local_object_mode_reserved_variable_membership,
            super::extract_source_local_mode_reserved_variable_equality,
            super::extract_source_local_mode_reserved_variable_inequality,
            super::extract_source_local_object_mode_reserved_variable_inequality,
            super::extract_source_chained_local_mode_reserved_variable_equality,
            super::extract_source_two_edge_local_mode_reserved_variable_equality,
            super::extract_source_three_edge_local_mode_reserved_variable_equality,
            super::extract_source_four_edge_local_mode_reserved_variable_equality,
            super::extract_source_local_mode_long_chain_reserved_variable_equality,
            super::extract_source_local_object_mode_long_chain_reserved_variable_equality,
            super::extract_source_local_object_mode_long_chain_reserved_variable_inequality,
            super::extract_source_local_object_mode_long_chain_reserved_variable_membership,
            super::extract_source_local_mode_long_chain_reserved_variable_inequality,
            super::extract_source_local_mode_long_chain_reserved_variable_membership,
            super::extract_source_four_edge_local_mode_reserved_variable_inequality,
            super::extract_source_four_edge_local_object_mode_reserved_variable_equality,
            super::extract_source_four_edge_local_object_mode_reserved_variable_inequality,
            super::extract_source_three_edge_local_object_mode_reserved_variable_equality,
            super::extract_source_three_edge_local_mode_reserved_variable_inequality,
            super::extract_source_three_edge_local_object_mode_reserved_variable_inequality,
            super::extract_source_two_edge_local_mode_reserved_variable_inequality,
            super::extract_source_two_edge_local_object_mode_reserved_variable_inequality,
            super::extract_source_two_edge_local_object_mode_reserved_variable_equality,
            super::extract_source_chained_local_mode_reserved_variable_inequality,
            super::extract_source_chained_local_object_mode_reserved_variable_equality,
            super::extract_source_chained_local_object_mode_reserved_variable_inequality,
            super::extract_source_local_object_mode_reserved_variable_equality,
            super::extract_source_multiple_reserve_declaration_equality,
            super::extract_source_multiple_object_reserve_declaration_equality,
            super::extract_source_multiple_object_reserve_declaration_inequality,
            super::extract_source_multiple_reserve_declaration_inequality,
            super::extract_source_multiple_reserve_declaration_membership,
            super::extract_source_reserved_variable_membership,
            super::extract_source_reserved_variable_inequality,
        ];

        let source_id = source_id(244);
        let task_244_module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("parenthesized_heterogeneous_reserve_membership_isolation"),
        );
        let task_244_symbols = SymbolEnv::new(task_244_module.clone(), SymbolEnvIndexes::default());
        let task_244_ast = reserve_then_parenthesized_identifier_binary_theorem_ast(
            source_id,
            vec![
                reserve_item(vec!["x"], ReserveTypeShape::Builtin("object")),
                reserve_item(vec!["y"], ReserveTypeShape::Builtin("set")),
            ],
            ParenthesizedIdentifierBinaryTheoremSpec {
                status: None,
                label: "ParenthesizedHeterogeneousReserveMembershipPayloadBoundary",
                left: ParenthesizedIdentifierOperandShape::Identifier {
                    spelling: "x",
                    depth: 1,
                    recovered: false,
                    open: "(",
                    close: ")",
                },
                operator: "in",
                right: ParenthesizedIdentifierOperandShape::Direct("y"),
                recovered_label: false,
            },
        );
        assert!(
            super::extract_source_parenthesized_heterogeneous_reserve_membership(
                &task_244_ast,
                task_244_module.clone(),
                &task_244_symbols,
            )
            .is_some()
        );
        for (name, extractor) in prior_extractors {
            assert!(
                extractor(&task_244_ast, task_244_module.clone(), &task_244_symbols).is_none(),
                "Task 244 source must not be owned by {name}",
            );
        }
        for (name, extractor) in [
            (
                "Task 223",
                super::extract_source_parenthesized_reserved_variable_equality
                    as fn(
                        &SurfaceAst,
                        ResolverModuleId,
                        &SymbolEnv,
                    )
                        -> Option<super::SourceParenthesizedReservedVariableBinaryFormula>,
            ),
            (
                "Task 233",
                super::extract_source_parenthesized_reserved_object_variable_equality
                    as fn(
                        &SurfaceAst,
                        ResolverModuleId,
                        &SymbolEnv,
                    )
                        -> Option<super::SourceParenthesizedReservedVariableBinaryFormula>,
            ),
            (
                "Task 241",
                super::extract_source_parenthesized_reserved_variable_inequality
                    as fn(
                        &SurfaceAst,
                        ResolverModuleId,
                        &SymbolEnv,
                    )
                        -> Option<super::SourceParenthesizedReservedVariableBinaryFormula>,
            ),
            (
                "Task 242",
                super::extract_source_parenthesized_reserved_object_variable_inequality
                    as fn(
                        &SurfaceAst,
                        ResolverModuleId,
                        &SymbolEnv,
                    )
                        -> Option<super::SourceParenthesizedReservedVariableBinaryFormula>,
            ),
            (
                "Task 243",
                super::extract_source_parenthesized_reserved_variable_membership
                    as fn(
                        &SurfaceAst,
                        ResolverModuleId,
                        &SymbolEnv,
                    )
                        -> Option<super::SourceParenthesizedReservedVariableBinaryFormula>,
            ),
        ] {
            assert!(
                extractor(&task_244_ast, task_244_module.clone(), &task_244_symbols).is_none(),
                "Task 244 source must not be owned by {name}",
            );
        }

        struct PriorOwnerFixture {
            id: String,
            ast: SurfaceAst,
            module: ResolverModuleId,
            symbols: SymbolEnv,
        }
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
        let plan = build_test_plan(&config).expect("Task 244 isolation plan should build");
        let mut candidates = Vec::new();
        for (ordinal, case) in active_type_elaboration_cases(&plan).enumerate() {
            let id = case.id.0.as_str();
            if id == "pass_type_elaboration_parenthesized_heterogeneous_reserve_membership_001"
                || !(id.contains("equality")
                    || id.contains("inequality")
                    || id.contains("membership"))
            {
                continue;
            }
            let frontend = run_frontend(&workspace_root, case, ordinal)
                .expect("prior binary owner fixture should run through the real frontend");
            assert!(
                frontend.diagnostics.is_empty(),
                "unexpected frontend diagnostics for {id}"
            );
            let ast = frontend
                .ast
                .expect("prior binary owner fixture should produce an AST");
            let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
            assert!(
                resolver.detail_keys.is_empty(),
                "unexpected resolver diagnostics for {id}"
            );
            let module = resolver.module;
            let symbols = augment_type_elaboration_import_summaries(&ast, &module, resolver.env);
            candidates.push(PriorOwnerFixture {
                id: id.to_owned(),
                ast,
                module,
                symbols,
            });
        }

        let mut matched_fixture_ids = BTreeSet::new();
        for (name, extractor) in prior_extractors {
            let matches = candidates
                .iter()
                .filter(|candidate| {
                    extractor(&candidate.ast, candidate.module.clone(), &candidate.symbols)
                        .is_some()
                })
                .collect::<Vec<_>>();
            assert_eq!(
                matches.len(),
                1,
                "{name} should retain exactly one real active owner fixture; matches={:?}",
                matches
                    .iter()
                    .map(|candidate| candidate.id.as_str())
                    .collect::<Vec<_>>()
            );
            let owner = matches[0];
            assert!(matched_fixture_ids.insert(owner.id.clone()));
            assert!(
                super::extract_source_parenthesized_heterogeneous_reserve_membership(
                    &owner.ast,
                    owner.module.clone(),
                    &owner.symbols,
                )
                .is_none(),
                "prior owner {} must not be captured by Task 244",
                owner.id,
            );
        }
        for (id, extractor) in [
            (
                "pass_type_elaboration_parenthesized_reserved_variable_equality_001",
                super::extract_source_parenthesized_reserved_variable_equality
                    as fn(
                        &SurfaceAst,
                        ResolverModuleId,
                        &SymbolEnv,
                    )
                        -> Option<super::SourceParenthesizedReservedVariableBinaryFormula>,
            ),
            (
                "pass_type_elaboration_parenthesized_reserved_object_variable_equality_001",
                super::extract_source_parenthesized_reserved_object_variable_equality
                    as fn(
                        &SurfaceAst,
                        ResolverModuleId,
                        &SymbolEnv,
                    )
                        -> Option<super::SourceParenthesizedReservedVariableBinaryFormula>,
            ),
            (
                "pass_type_elaboration_parenthesized_reserved_variable_inequality_001",
                super::extract_source_parenthesized_reserved_variable_inequality
                    as fn(
                        &SurfaceAst,
                        ResolverModuleId,
                        &SymbolEnv,
                    )
                        -> Option<super::SourceParenthesizedReservedVariableBinaryFormula>,
            ),
            (
                "pass_type_elaboration_parenthesized_reserved_object_variable_inequality_001",
                super::extract_source_parenthesized_reserved_object_variable_inequality
                    as fn(
                        &SurfaceAst,
                        ResolverModuleId,
                        &SymbolEnv,
                    )
                        -> Option<super::SourceParenthesizedReservedVariableBinaryFormula>,
            ),
            (
                "pass_type_elaboration_parenthesized_reserved_variable_membership_001",
                super::extract_source_parenthesized_reserved_variable_membership
                    as fn(
                        &SurfaceAst,
                        ResolverModuleId,
                        &SymbolEnv,
                    )
                        -> Option<super::SourceParenthesizedReservedVariableBinaryFormula>,
            ),
        ] {
            let owner = candidates
                .iter()
                .find(|candidate| candidate.id == id)
                .unwrap_or_else(|| panic!("prior parenthesized owner {id} should remain active"));
            assert!(extractor(&owner.ast, owner.module.clone(), &owner.symbols).is_some());
            assert!(
                super::extract_source_parenthesized_heterogeneous_reserve_membership(
                    &owner.ast,
                    owner.module.clone(),
                    &owner.symbols,
                )
                .is_none()
            );
            assert!(matched_fixture_ids.insert(owner.id.clone()));
        }
        assert_eq!(matched_fixture_ids.len(), 57);
    }

    #[test]
    fn right_parenthesized_membership_route_isolated_from_all_58_prior_binary_owners() {
        type DirectExtractor = fn(
            &SurfaceAst,
            ResolverModuleId,
            &SymbolEnv,
        ) -> Option<super::SourceReservedVariableBinaryFormula>;
        macro_rules! direct_extractors {
            ($($extractor:path),+ $(,)?) => {
                [$(
                    (stringify!($extractor), $extractor as DirectExtractor),
                )+]
            };
        }
        let direct_extractors: [(&str, DirectExtractor); 52] = direct_extractors![
            super::extract_source_reserved_variable_equality,
            super::extract_source_reserved_object_variable_equality,
            super::extract_source_distinct_reserved_object_variable_equality,
            super::extract_source_distinct_reserved_object_variable_inequality,
            super::extract_source_reserved_object_variable_inequality,
            super::extract_source_distinct_reserved_variable_equality,
            super::extract_source_distinct_reserved_variable_membership,
            super::extract_source_distinct_reserved_variable_inequality,
            super::extract_source_heterogeneous_reserve_membership,
            super::extract_source_local_mode_reserved_variable_membership,
            super::extract_source_chained_local_mode_reserved_variable_membership,
            super::extract_source_two_edge_local_mode_reserved_variable_membership,
            super::extract_source_three_edge_local_mode_reserved_variable_membership,
            super::extract_source_four_edge_local_mode_reserved_variable_membership,
            super::extract_source_four_edge_local_object_mode_reserved_variable_membership,
            super::extract_source_three_edge_local_object_mode_reserved_variable_membership,
            super::extract_source_two_edge_local_object_mode_reserved_variable_membership,
            super::extract_source_chained_local_object_mode_reserved_variable_membership,
            super::extract_source_local_object_mode_reserved_variable_membership,
            super::extract_source_local_mode_reserved_variable_equality,
            super::extract_source_local_mode_reserved_variable_inequality,
            super::extract_source_local_object_mode_reserved_variable_inequality,
            super::extract_source_chained_local_mode_reserved_variable_equality,
            super::extract_source_two_edge_local_mode_reserved_variable_equality,
            super::extract_source_three_edge_local_mode_reserved_variable_equality,
            super::extract_source_four_edge_local_mode_reserved_variable_equality,
            super::extract_source_local_mode_long_chain_reserved_variable_equality,
            super::extract_source_local_object_mode_long_chain_reserved_variable_equality,
            super::extract_source_local_object_mode_long_chain_reserved_variable_inequality,
            super::extract_source_local_object_mode_long_chain_reserved_variable_membership,
            super::extract_source_local_mode_long_chain_reserved_variable_inequality,
            super::extract_source_local_mode_long_chain_reserved_variable_membership,
            super::extract_source_four_edge_local_mode_reserved_variable_inequality,
            super::extract_source_four_edge_local_object_mode_reserved_variable_equality,
            super::extract_source_four_edge_local_object_mode_reserved_variable_inequality,
            super::extract_source_three_edge_local_object_mode_reserved_variable_equality,
            super::extract_source_three_edge_local_mode_reserved_variable_inequality,
            super::extract_source_three_edge_local_object_mode_reserved_variable_inequality,
            super::extract_source_two_edge_local_mode_reserved_variable_inequality,
            super::extract_source_two_edge_local_object_mode_reserved_variable_inequality,
            super::extract_source_two_edge_local_object_mode_reserved_variable_equality,
            super::extract_source_chained_local_mode_reserved_variable_inequality,
            super::extract_source_chained_local_object_mode_reserved_variable_equality,
            super::extract_source_chained_local_object_mode_reserved_variable_inequality,
            super::extract_source_local_object_mode_reserved_variable_equality,
            super::extract_source_multiple_reserve_declaration_equality,
            super::extract_source_multiple_object_reserve_declaration_equality,
            super::extract_source_multiple_object_reserve_declaration_inequality,
            super::extract_source_multiple_reserve_declaration_inequality,
            super::extract_source_multiple_reserve_declaration_membership,
            super::extract_source_reserved_variable_membership,
            super::extract_source_reserved_variable_inequality,
        ];
        type ParenthesizedExtractor =
            fn(
                &SurfaceAst,
                ResolverModuleId,
                &SymbolEnv,
            ) -> Option<super::SourceParenthesizedReservedVariableBinaryFormula>;
        let parenthesized_extractors: [(&str, ParenthesizedExtractor); 6] = [
            (
                "Task 223",
                super::extract_source_parenthesized_reserved_variable_equality,
            ),
            (
                "Task 233",
                super::extract_source_parenthesized_reserved_object_variable_equality,
            ),
            (
                "Task 241",
                super::extract_source_parenthesized_reserved_variable_inequality,
            ),
            (
                "Task 242",
                super::extract_source_parenthesized_reserved_object_variable_inequality,
            ),
            (
                "Task 243",
                super::extract_source_parenthesized_reserved_variable_membership,
            ),
            (
                "Task 244",
                super::extract_source_parenthesized_heterogeneous_reserve_membership,
            ),
        ];

        let source_id = source_id(245);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("right_parenthesized_membership_owner_isolation"),
        );
        let symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        let ast = reserve_then_parenthesized_identifier_binary_theorem_ast(
            source_id,
            vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
            ParenthesizedIdentifierBinaryTheoremSpec {
                status: None,
                label: "RightParenthesizedReservedVariableMembershipPayloadBoundary",
                left: ParenthesizedIdentifierOperandShape::Direct("x"),
                operator: "in",
                right: ParenthesizedIdentifierOperandShape::Identifier {
                    spelling: "x",
                    depth: 1,
                    recovered: false,
                    open: "(",
                    close: ")",
                },
                recovered_label: false,
            },
        );
        assert!(
            super::extract_source_right_parenthesized_reserved_variable_membership(
                &ast,
                module.clone(),
                &symbols,
            )
            .is_some()
        );
        for (name, extractor) in &direct_extractors {
            assert!(
                extractor(&ast, module.clone(), &symbols).is_none(),
                "Task 245 source must not be owned by {name}",
            );
        }
        for (name, extractor) in &parenthesized_extractors {
            assert!(
                extractor(&ast, module.clone(), &symbols).is_none(),
                "Task 245 source must not be owned by {name}",
            );
        }

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
        let plan = build_test_plan(&config).expect("Task 245 isolation plan should build");
        let mut prior_owner_ids = BTreeSet::new();
        for (ordinal, case) in active_type_elaboration_cases(&plan).enumerate() {
            let id = case.id.0.as_str();
            if id == "pass_type_elaboration_right_parenthesized_reserved_variable_membership_001"
                || !(id.contains("equality")
                    || id.contains("inequality")
                    || id.contains("membership"))
            {
                continue;
            }
            let frontend = run_frontend(&workspace_root, case, ordinal)
                .expect("prior binary owner fixture should run through the real frontend");
            assert!(
                frontend.diagnostics.is_empty(),
                "unexpected frontend diagnostics for {id}"
            );
            let owner_ast = frontend
                .ast
                .expect("prior binary owner fixture should produce an AST");
            let resolver = resolver_symbol_collection(&workspace_root, case, &owner_ast);
            assert!(
                resolver.detail_keys.is_empty(),
                "unexpected resolver diagnostics for {id}"
            );
            let owner_symbols = augment_type_elaboration_import_summaries(
                &owner_ast,
                &resolver.module,
                resolver.env,
            );
            let owner_matches = direct_extractors
                .iter()
                .filter(|(_, extractor)| {
                    extractor(&owner_ast, resolver.module.clone(), &owner_symbols).is_some()
                })
                .count()
                + parenthesized_extractors
                    .iter()
                    .filter(|(_, extractor)| {
                        extractor(&owner_ast, resolver.module.clone(), &owner_symbols).is_some()
                    })
                    .count();
            if owner_matches == 0 {
                continue;
            }
            assert_eq!(
                owner_matches, 1,
                "prior fixture {id} should retain exactly one binary owner route",
            );
            assert!(
                super::extract_source_right_parenthesized_reserved_variable_membership(
                    &owner_ast,
                    resolver.module,
                    &owner_symbols,
                )
                .is_none(),
                "prior owner {id} must not be captured by Task 245",
            );
            assert!(prior_owner_ids.insert(id.to_owned()));
        }
        assert_eq!(prior_owner_ids.len(), 58);
    }

    #[test]
    fn parenthesized_two_edge_local_mode_equality_route_isolated_from_all_59_prior_binary_owners() {
        type DirectExtractor = fn(
            &SurfaceAst,
            ResolverModuleId,
            &SymbolEnv,
        ) -> Option<super::SourceReservedVariableBinaryFormula>;
        macro_rules! direct_extractors {
            ($($extractor:path),+ $(,)?) => {
                [$(
                    (stringify!($extractor), $extractor as DirectExtractor),
                )+]
            };
        }
        let direct_extractors: [(&str, DirectExtractor); 52] = direct_extractors![
            super::extract_source_reserved_variable_equality,
            super::extract_source_reserved_object_variable_equality,
            super::extract_source_distinct_reserved_object_variable_equality,
            super::extract_source_distinct_reserved_object_variable_inequality,
            super::extract_source_reserved_object_variable_inequality,
            super::extract_source_distinct_reserved_variable_equality,
            super::extract_source_distinct_reserved_variable_membership,
            super::extract_source_distinct_reserved_variable_inequality,
            super::extract_source_heterogeneous_reserve_membership,
            super::extract_source_local_mode_reserved_variable_membership,
            super::extract_source_chained_local_mode_reserved_variable_membership,
            super::extract_source_two_edge_local_mode_reserved_variable_membership,
            super::extract_source_three_edge_local_mode_reserved_variable_membership,
            super::extract_source_four_edge_local_mode_reserved_variable_membership,
            super::extract_source_four_edge_local_object_mode_reserved_variable_membership,
            super::extract_source_three_edge_local_object_mode_reserved_variable_membership,
            super::extract_source_two_edge_local_object_mode_reserved_variable_membership,
            super::extract_source_chained_local_object_mode_reserved_variable_membership,
            super::extract_source_local_object_mode_reserved_variable_membership,
            super::extract_source_local_mode_reserved_variable_equality,
            super::extract_source_local_mode_reserved_variable_inequality,
            super::extract_source_local_object_mode_reserved_variable_inequality,
            super::extract_source_chained_local_mode_reserved_variable_equality,
            super::extract_source_two_edge_local_mode_reserved_variable_equality,
            super::extract_source_three_edge_local_mode_reserved_variable_equality,
            super::extract_source_four_edge_local_mode_reserved_variable_equality,
            super::extract_source_local_mode_long_chain_reserved_variable_equality,
            super::extract_source_local_object_mode_long_chain_reserved_variable_equality,
            super::extract_source_local_object_mode_long_chain_reserved_variable_inequality,
            super::extract_source_local_object_mode_long_chain_reserved_variable_membership,
            super::extract_source_local_mode_long_chain_reserved_variable_inequality,
            super::extract_source_local_mode_long_chain_reserved_variable_membership,
            super::extract_source_four_edge_local_mode_reserved_variable_inequality,
            super::extract_source_four_edge_local_object_mode_reserved_variable_equality,
            super::extract_source_four_edge_local_object_mode_reserved_variable_inequality,
            super::extract_source_three_edge_local_object_mode_reserved_variable_equality,
            super::extract_source_three_edge_local_mode_reserved_variable_inequality,
            super::extract_source_three_edge_local_object_mode_reserved_variable_inequality,
            super::extract_source_two_edge_local_mode_reserved_variable_inequality,
            super::extract_source_two_edge_local_object_mode_reserved_variable_inequality,
            super::extract_source_two_edge_local_object_mode_reserved_variable_equality,
            super::extract_source_chained_local_mode_reserved_variable_inequality,
            super::extract_source_chained_local_object_mode_reserved_variable_equality,
            super::extract_source_chained_local_object_mode_reserved_variable_inequality,
            super::extract_source_local_object_mode_reserved_variable_equality,
            super::extract_source_multiple_reserve_declaration_equality,
            super::extract_source_multiple_object_reserve_declaration_equality,
            super::extract_source_multiple_object_reserve_declaration_inequality,
            super::extract_source_multiple_reserve_declaration_inequality,
            super::extract_source_multiple_reserve_declaration_membership,
            super::extract_source_reserved_variable_membership,
            super::extract_source_reserved_variable_inequality,
        ];
        type ParenthesizedExtractor =
            fn(
                &SurfaceAst,
                ResolverModuleId,
                &SymbolEnv,
            ) -> Option<super::SourceParenthesizedReservedVariableBinaryFormula>;
        let parenthesized_extractors: [(&str, ParenthesizedExtractor); 7] = [
            (
                "Task 223",
                super::extract_source_parenthesized_reserved_variable_equality,
            ),
            (
                "Task 233",
                super::extract_source_parenthesized_reserved_object_variable_equality,
            ),
            (
                "Task 241",
                super::extract_source_parenthesized_reserved_variable_inequality,
            ),
            (
                "Task 242",
                super::extract_source_parenthesized_reserved_object_variable_inequality,
            ),
            (
                "Task 243",
                super::extract_source_parenthesized_reserved_variable_membership,
            ),
            (
                "Task 244",
                super::extract_source_parenthesized_heterogeneous_reserve_membership,
            ),
            (
                "Task 245",
                super::extract_source_right_parenthesized_reserved_variable_membership,
            ),
        ];

        let source_id = source_id(246);
        let module = ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("parenthesized_two_edge_equality_owner_isolation"),
        );
        let symbols = source_local_symbols_env(
            module.clone(),
            &[
                ("BaseTwoEdgeModeEquality", SymbolKind::Mode),
                ("MiddleTwoEdgeModeEquality", SymbolKind::Mode),
                ("OuterTwoEdgeModeEquality", SymbolKind::Mode),
            ],
        );
        let ast = mode_then_reserve_parenthesized_identifier_binary_theorem_ast(
            source_id,
            [
                mode_definition("BaseTwoEdgeModeEquality", ReserveTypeShape::Builtin("set")),
                mode_definition(
                    "MiddleTwoEdgeModeEquality",
                    ReserveTypeShape::QualifiedSymbol("BaseTwoEdgeModeEquality"),
                ),
                mode_definition(
                    "OuterTwoEdgeModeEquality",
                    ReserveTypeShape::QualifiedSymbol("MiddleTwoEdgeModeEquality"),
                ),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("OuterTwoEdgeModeEquality"),
            )],
            ParenthesizedIdentifierBinaryTheoremSpec {
                status: None,
                label: "ParenthesizedTwoEdgeLocalModeReservedVariableEqualityPayloadBoundary",
                left: ParenthesizedIdentifierOperandShape::Identifier {
                    spelling: "z",
                    depth: 1,
                    recovered: false,
                    open: "(",
                    close: ")",
                },
                operator: "=",
                right: ParenthesizedIdentifierOperandShape::Direct("z"),
                recovered_label: false,
            },
        );
        assert!(
            super::extract_source_parenthesized_two_edge_local_mode_reserved_variable_equality(
                &ast,
                module.clone(),
                &symbols,
            )
            .is_some()
        );
        for (name, extractor) in &direct_extractors {
            assert!(
                extractor(&ast, module.clone(), &symbols).is_none(),
                "Task 246 source must not be owned by {name}",
            );
        }
        for (name, extractor) in &parenthesized_extractors {
            assert!(
                extractor(&ast, module.clone(), &symbols).is_none(),
                "Task 246 source must not be owned by {name}",
            );
        }

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
        let plan = build_test_plan(&config).expect("Task 246 isolation plan should build");
        let mut prior_owner_ids = BTreeSet::new();
        for (ordinal, case) in active_type_elaboration_cases(&plan).enumerate() {
            let id = case.id.0.as_str();
            if id
                == "pass_type_elaboration_parenthesized_two_edge_local_mode_reserved_variable_equality_001"
                || !(id.contains("equality")
                    || id.contains("inequality")
                    || id.contains("membership"))
            {
                continue;
            }
            let frontend = run_frontend(&workspace_root, case, ordinal)
                .expect("prior binary owner fixture should run through the real frontend");
            assert!(
                frontend.diagnostics.is_empty(),
                "unexpected frontend diagnostics for {id}"
            );
            let owner_ast = frontend
                .ast
                .expect("prior binary owner fixture should produce an AST");
            let resolver = resolver_symbol_collection(&workspace_root, case, &owner_ast);
            assert!(
                resolver.detail_keys.is_empty(),
                "unexpected resolver diagnostics for {id}"
            );
            let owner_symbols = augment_type_elaboration_import_summaries(
                &owner_ast,
                &resolver.module,
                resolver.env,
            );
            let owner_matches = direct_extractors
                .iter()
                .filter(|(_, extractor)| {
                    extractor(&owner_ast, resolver.module.clone(), &owner_symbols).is_some()
                })
                .count()
                + parenthesized_extractors
                    .iter()
                    .filter(|(_, extractor)| {
                        extractor(&owner_ast, resolver.module.clone(), &owner_symbols).is_some()
                    })
                    .count();
            if owner_matches == 0 {
                continue;
            }
            assert_eq!(
                owner_matches, 1,
                "prior fixture {id} should retain exactly one binary owner route",
            );
            assert!(
                super::extract_source_parenthesized_two_edge_local_mode_reserved_variable_equality(
                    &owner_ast,
                    resolver.module,
                    &owner_symbols,
                )
                .is_none(),
                "prior owner {id} must not be captured by Task 246",
            );
            assert!(prior_owner_ids.insert(id.to_owned()));
        }
        assert_eq!(prior_owner_ids.len(), 59);
    }
