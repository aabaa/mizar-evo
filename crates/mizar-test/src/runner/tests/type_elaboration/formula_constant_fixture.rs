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
