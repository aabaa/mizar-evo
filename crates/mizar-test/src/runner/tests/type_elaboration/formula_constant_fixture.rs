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
        let source_payload = extract_source_contradiction_formula(&ast)
            .expect("Task 180 real formula identity should extract");
        let handoff = assemble_source_contradiction_checker_handoff(
            &ast,
            resolver.module.clone(),
            &symbols,
        )
        .expect("Task 266 real AST should reach the final contradiction handoff");
        assert_source_contradiction_handoff(&handoff)
            .expect("Task 266 final contradiction handoff should be internally consistent");
        let output = &handoff.term_formula;
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
        assert_eq!(formula.site, source_payload.formula_site);
        assert!(formula.terms.is_empty());
        assert!(formula.asserted_type.is_none());
        assert!(formula.expected_types.is_empty());
        assert!(formula.candidate_set.is_none());
        assert!(formula.facts.is_empty());
        assert!(formula.deferred.is_empty());
        assert_eq!(formula.source_range.source_id, ast.source_id);
        assert_eq!(formula.recovery, mizar_checker::typed_ast::NodeRecoveryState::Normal);

        let statement = handoff
            .resolved
            .statement_semantics()
            .get(mizar_checker::resolved_typed_ast::StatementSemanticId::new(0))
            .expect("Task 266 should preserve one final statement row");
        assert_eq!(statement.formula, formula.id);
        assert_eq!(handoff.resolved.checked_formulas(), output.formulas());
        let owner = symbols
            .symbols()
            .get(&statement.owner)
            .expect("Task 266 owner should be the real resolver symbol");
        assert_eq!(owner.kind(), SymbolKind::Theorem);
        assert_eq!(owner.visibility(), mizar_resolve::env::Visibility::Public);
        assert_eq!(
            owner.export_status(),
            mizar_resolve::env::ExportStatus::Exported
        );
        assert_eq!(
            handoff.owner.visibility(),
            mizar_resolve::env::Visibility::Public
        );
        assert_eq!(
            handoff.owner.export_status(),
            mizar_resolve::env::ExportStatus::Exported
        );
        assert_eq!(statement.owner_origin, *owner.origin());
        assert_eq!(
            owner.origin().anchor(),
            &SourceAnchor::Range(statement.owner_range)
        );
        assert!(statement.owner_range.start < formula.source_range.start);
        assert!(formula.source_range.end < statement.owner_range.end);
        let owner_node = handoff
            .typed_ast
            .nodes()
            .node(statement.owner_node)
            .expect("Task 266 theorem typed node should exist");
        assert_eq!(owner_node.anchor, SourceAnchor::Range(statement.owner_range));
        assert_eq!(owner_node.children, vec![statement.formula_node]);
        assert_eq!(
            ast.node(source_payload.owner_site)
                .expect("Task 180 theorem source site")
                .range,
            statement.owner_range
        );

        let second = assemble_source_contradiction_checker_handoff(
            &ast,
            resolver.module.clone(),
            &symbols,
        )
        .expect("equivalent Task 266 handoff should assemble");
        assert_eq!(handoff.resolved, second.resolved);
        assert_eq!(handoff.resolved.debug_text(), second.resolved.debug_text());
        let core_debug = source_contradiction_core_ir_snapshot(
            &ast,
            resolver.module.clone(),
            &symbols,
        )
        .expect("Task 180 source should select the exact CoreIr snapshot consumer")
        .expect("Task 31 exact CoreIr snapshot should lower deterministically");
        assert!(core_debug.starts_with("core-ir-debug-v1\n"));
        assert!(core_debug.contains("status: PendingAutomaticProof"));
        assert!(core_debug.contains("kind: False"));
        assert!(core_debug.contains("kind: TheoremProof"));
        assert!(core_debug.contains("status: Active"));
        let extra_expression = source_contradiction_handoff_with_extra_expression(
            &ast,
            resolver.module.clone(),
            &symbols,
        )
        .expect("checker-valid Task 180 handoff with unrelated expression metadata should assemble");
        assert_eq!(extra_expression.expr_metadata().len(), 1);
        assert!(matches!(
            lower_exact_task180_handoff(&extra_expression),
            Err(ExactTask180LoweringError::InvalidCheckerBundle { reason })
                if reason == "unrelated checker payload must be empty"
        ));
        let proof_debug = handoff.resolved.debug_text();
        assert!(proof_debug.contains("checked-proofs:\n"));
        assert!(proof_debug.contains("status=PendingAutomaticProof"));
        assert!(proof_debug.contains("checked-proof-nodes:\n"));
        assert!(proof_debug.contains("checked-terminal-goals:\n"));
        assert!(proof_debug.contains("local_path=\"proof/0\" label=None"));
        assert_eq!(
            source_contradiction_formula_detail_keys(&ast, resolver.module.clone(), &symbols),
            Some(Vec::new())
        );

        for corruption in [
            SourceContradictionHandoffCorruption::MissingRow,
            SourceContradictionHandoffCorruption::DuplicateRow,
            SourceContradictionHandoffCorruption::InvalidFormula,
            SourceContradictionHandoffCorruption::WrongOwnerNode,
            SourceContradictionHandoffCorruption::MissingProofRow,
            SourceContradictionHandoffCorruption::DuplicateProofRow,
            SourceContradictionHandoffCorruption::NonzeroProofIntentId,
            SourceContradictionHandoffCorruption::NonzeroProofSourceOrder,
            SourceContradictionHandoffCorruption::NonzeroProofStatement,
            SourceContradictionHandoffCorruption::WrongProofSource,
            SourceContradictionHandoffCorruption::WrongProofModule,
            SourceContradictionHandoffCorruption::WrongProofOwner,
            SourceContradictionHandoffCorruption::WrongProofOwnerNode,
            SourceContradictionHandoffCorruption::WrongProofOwnerRange,
            SourceContradictionHandoffCorruption::WrongProofOwnerOrigin,
            SourceContradictionHandoffCorruption::PrivateProofOwner,
            SourceContradictionHandoffCorruption::LocalOnlyProofOwner,
            SourceContradictionHandoffCorruption::InvalidProofFormula,
            SourceContradictionHandoffCorruption::RoleProofFormulaSite,
            SourceContradictionHandoffCorruption::WrongProofFormulaNode,
            SourceContradictionHandoffCorruption::WrongProofFormulaRange,
            SourceContradictionHandoffCorruption::RecoveredProofIntent,
        ] {
            let error = source_contradiction_handoff_corruption_error(
                &ast,
                resolver.module.clone(),
                &symbols,
                corruption,
            )
            .expect("Task 266 real handoff corruption should fail closed");
            assert!(!error.is_empty());
        }
    }
