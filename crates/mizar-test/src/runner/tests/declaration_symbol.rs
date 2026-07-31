#[test]
fn task258b5c_frozen_surface_profile_hashes_are_exact() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf();
    let config = DiscoveryConfig {
        workspace_root: workspace_root.clone(),
        tests_root: workspace_root.join("tests"),
        manifest_path: workspace_root.join("tests/coverage/spec_trace.toml"),
        profile: TestProfile::Fast,
        validation_mode: ValidationMode::Metadata,
    };
    let plan = build_test_plan(&config).expect("Task 258B5C repository plan");
    let ids = [
        "fail_resolve_proof_label_inner_to_outer_confinement_001",
        "fail_resolve_proof_label_sibling_confinement_001",
    ];
    let hashes = ids.map(|id| {
        let (ordinal, case) = plan
            .cases
            .iter()
            .enumerate()
            .find(|(_, case)| case.id.0 == id)
            .expect("exact Task 258B5C case");
        let output = run_frontend(&workspace_root, case, ordinal).expect("B5C frontend");
        assert!(output.diagnostics.is_empty());
        let ast = output.ast.as_ref().expect("B5C Surface AST");
        super::declaration_symbol::proof_label_dense_profile_hash_for_test(ast)
    });
    assert_eq!(hashes, [0x05f4_763c_ffe3_248b, 0x1226_7d7f_52fa_e5a8]);
}

#[test]
fn task258b5c_every_input_projection_reference_and_result_mutation_fails_closed() {
    use super::declaration_symbol::ProofLabelConfinementMutation as Mutation;

    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf();
    let config = DiscoveryConfig {
        workspace_root: workspace_root.clone(),
        tests_root: workspace_root.join("tests"),
        manifest_path: workspace_root.join("tests/coverage/spec_trace.toml"),
        profile: TestProfile::Fast,
        validation_mode: ValidationMode::Metadata,
    };
    let plan = build_test_plan(&config).expect("Task 258B5C repository plan");
    let (ordinal, case) = plan
        .cases
        .iter()
        .enumerate()
        .find(|(_, case)| {
            case.id.0 == "fail_resolve_proof_label_inner_to_outer_confinement_001"
        })
        .expect("exact Task 258B5C case");
    let output = run_frontend(&workspace_root, case, ordinal).expect("B5C frontend");
    assert!(output.diagnostics.is_empty());
    let ast = output.ast.as_ref().expect("B5C Surface AST");
    let resolver = resolver_symbol_collection(&workspace_root, case, ast);
    assert!(resolver.detail_keys.is_empty());

    let allocator = InMemorySessionIdAllocator::new();
    let _ = allocator
        .next_source_id(snapshot_id(0xf0))
        .expect("reserved source id");
    let foreign_source = allocator
        .next_source_id(snapshot_id(0xf1))
        .expect("foreign source id");

    let detail = |mutation| {
        super::declaration_symbol::proof_label_confinement_detail_with_mutation_for_test(
            &output.source_text,
            ast,
            &resolver.module,
            &resolver.env,
            foreign_source,
            mutation,
        )
    };
    assert_eq!(
        detail(Mutation::None),
        Some("declaration_symbol.label.proof_scope_confinement")
    );

    let mutations = [
        Mutation::EnvironmentModule,
        Mutation::SymbolCount,
        Mutation::LabelCount,
        Mutation::DefinitionCount,
        Mutation::NoContribution,
        Mutation::MultipleContributions,
        Mutation::ImportCount,
        Mutation::ContributionId,
        Mutation::ContributionImported,
        Mutation::ContributionSummary,
        Mutation::ContributionBuiltin,
        Mutation::ContributionModule,
        Mutation::ContributionSource,
        Mutation::ProjectionExtra,
        Mutation::ProjectionOriginPath,
        Mutation::ProjectionModule,
        Mutation::ProjectionNamespace,
        Mutation::ProjectionSpelling,
        Mutation::ProjectionKind,
        Mutation::ProjectionVisibility,
        Mutation::ProjectionExportStatus,
        Mutation::ProjectionRange,
        Mutation::ProjectionOriginSource,
        Mutation::ProjectionOriginModule,
        Mutation::ProjectionOriginAnchor,
        Mutation::ProjectionOriginStructuralPath,
        Mutation::ProjectionOriginRecovered,
        Mutation::ProjectionContribution,
        Mutation::ProjectionVisibleOrdinal,
        Mutation::ProjectionScope,
        Mutation::ProjectionImported,
        Mutation::ReferenceExtra,
        Mutation::ReferenceNode,
        Mutation::ReferenceRange,
        Mutation::ReferenceSpelling,
        Mutation::ReferenceOriginSource,
        Mutation::ReferenceOriginModule,
        Mutation::ReferenceOriginAnchor,
        Mutation::ReferenceOriginStructuralPath,
        Mutation::ReferenceOriginRecovered,
        Mutation::ReferenceOrdinal,
        Mutation::ReferenceExpectation,
        Mutation::ReferenceScope,
        Mutation::ReferenceQualified,
        Mutation::ReferenceFailedNamespace,
        Mutation::ResultResolved,
        Mutation::ResultAmbiguous,
        Mutation::ResultExtraReference,
        Mutation::ResultDiagnostic,
        Mutation::ResultId,
        Mutation::ResultIndexCount,
        Mutation::ResultTableCount,
        Mutation::ResultHasUnresolved,
        Mutation::ResultIndexOriginPath,
        Mutation::ResultIndexKind,
        Mutation::ResultIndexVisibility,
        Mutation::ResultIndexExportStatus,
        Mutation::ResultIndexNamespace,
        Mutation::ResultIndexSpelling,
        Mutation::ResultIndexOrigin,
        Mutation::ResultIndexContribution,
        Mutation::ResultIndexRecovery,
        Mutation::ResultTableSite,
        Mutation::ResultTableOrigin,
        Mutation::ResultTableRecovery,
        Mutation::ResultUnresolvedSpelling,
        Mutation::ResultUnresolvedRange,
        Mutation::ResultUnresolvedExpectation,
    ];
    for mutation in mutations {
        assert_eq!(
            detail(mutation),
            Some("declaration_symbol.label.proof_scope_input"),
            "{mutation:?} must fail closed"
        );
    }
}

#[test]
fn task258b5c_exact_cases_replay_and_reverse_order_without_disturbing_existing_cases() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf();
    let config = DiscoveryConfig {
        workspace_root: workspace_root.clone(),
        tests_root: workspace_root.join("tests"),
        manifest_path: workspace_root.join("tests/coverage/spec_trace.toml"),
        profile: TestProfile::Fast,
        validation_mode: ValidationMode::Metadata,
    };
    let plan = build_test_plan(&config).expect("Task 258B5C repository plan");
    let active = super::active_declaration_symbol_cases(&plan)
        .enumerate()
        .collect::<Vec<_>>();
    assert_eq!(active.len(), 7);

    let run = |entries: &[(usize, &crate::harness::TestCase)]| {
        entries
            .iter()
            .map(|(ordinal, case)| {
                super::declaration_symbol::run_declaration_symbol_case(
                    &workspace_root,
                    case,
                    *ordinal,
                )
            })
            .collect::<Vec<_>>()
    };
    let first = run(&active);
    assert!(
        first
            .iter()
            .all(|result| result.status == super::DeclarationSymbolCaseStatus::Passed)
    );
    let b5c = first
        .iter()
        .filter(|result| result.id.0.contains("proof_label_"))
        .collect::<Vec<_>>();
    assert_eq!(b5c.len(), 2);
    for result in b5c {
        assert_eq!(
            result.actual_detail_keys,
            ["declaration_symbol.label.proof_scope_confinement"]
        );
        assert!(result.actual_payload_keys.is_empty());
    }

    assert_eq!(run(&active), first, "same-order replay must be identical");
    let mut reversed_entries = active.clone();
    reversed_entries.reverse();
    let mut reversed = run(&reversed_entries);
    reversed.reverse();
    assert_eq!(
        reversed, first,
        "execution order must not affect any active declaration-symbol case"
    );
}

#[test]
fn task258b5c_source_and_normal_ast_select_the_route_but_expectations_do_not() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf();
    let config = DiscoveryConfig {
        workspace_root: workspace_root.clone(),
        tests_root: workspace_root.join("tests"),
        manifest_path: workspace_root.join("tests/coverage/spec_trace.toml"),
        profile: TestProfile::Fast,
        validation_mode: ValidationMode::Metadata,
    };
    let plan = build_test_plan(&config).expect("Task 258B5C repository plan");
    let active = super::active_declaration_symbol_cases(&plan)
        .enumerate()
        .collect::<Vec<_>>();
    let (b5c_ordinal, b5c_case) = active
        .iter()
        .copied()
        .find(|(_, case)| {
            case.id.0 == "fail_resolve_proof_label_inner_to_outer_confinement_001"
        })
        .expect("exact Task 258B5C case");
    let (sibling_ordinal, sibling_case) = active
        .iter()
        .copied()
        .find(|(_, case)| case.id.0 == "fail_resolve_proof_label_sibling_confinement_001")
        .expect("exact sibling Task 258B5C case");
    let (other_ordinal, other_case) = active
        .iter()
        .copied()
        .find(|(_, case)| !case.id.0.contains("proof_label_"))
        .expect("existing declaration-symbol control case");

    let output = run_frontend(&workspace_root, b5c_case, b5c_ordinal).expect("B5C frontend");
    let ast = output.ast.as_ref().expect("B5C Surface AST");
    let sibling_output =
        run_frontend(&workspace_root, sibling_case, sibling_ordinal).expect("sibling B5C frontend");
    let sibling_ast = sibling_output
        .ast
        .as_ref()
        .expect("sibling B5C Surface AST");
    assert!(
        super::declaration_symbol::proof_label_confinement_profile_for_test(
            &output.source_text,
            ast
        )
    );
    assert!(
        super::declaration_symbol::proof_label_confinement_profile_for_test(
            &sibling_output.source_text,
            sibling_ast
        )
    );
    assert!(
        !super::declaration_symbol::proof_label_confinement_profile_for_test(
            &output.source_text,
            sibling_ast
        ),
        "exact inner source must reject the exact sibling AST"
    );
    assert!(
        !super::declaration_symbol::proof_label_confinement_profile_for_test(
            &sibling_output.source_text,
            ast
        ),
        "exact sibling source must reject the exact inner AST"
    );
    let mut changed_source = output.source_text.to_string();
    changed_source.push('\n');
    assert!(
        !super::declaration_symbol::proof_label_confinement_profile_for_test(
            &changed_source,
            ast
        )
    );

    let mut mutated_expectation = b5c_case.clone();
    mutated_expectation.expectation.stable_detail_key =
        Some("copied.expectation.must.not.select".to_owned());
    mutated_expectation.expectation.diagnostic_payloads =
        vec!["copied.expectation.must.not.select".to_owned()];
    mutated_expectation.expectation.spec_refs.clear();
    mutated_expectation.expectation.tags.clear();
    mutated_expectation.expectation.rejection_reason =
        Some("copied_expectation_must_not_select".to_owned());
    let result = super::declaration_symbol::run_declaration_symbol_case(
        &workspace_root,
        &mutated_expectation,
        b5c_ordinal,
    );
    assert_eq!(
        result.actual_detail_keys,
        ["declaration_symbol.label.proof_scope_confinement"]
    );
    assert_eq!(result.status, super::DeclarationSymbolCaseStatus::Failed);

    let mut copied_expectation = other_case.clone();
    copied_expectation.expectation = b5c_case.expectation.clone();
    let result = super::declaration_symbol::run_declaration_symbol_case(
        &workspace_root,
        &copied_expectation,
        other_ordinal,
    );
    assert_ne!(
        result.actual_detail_keys,
        ["declaration_symbol.label.proof_scope_confinement"]
    );

    let mut public_code = b5c_case.clone();
    public_code.expectation.diagnostic_codes =
        vec!["E-UNSPECIFIED-PROOF-LABEL-SCOPE".to_owned()];
    let result = super::declaration_symbol::run_declaration_symbol_case(
        &workspace_root,
        &public_code,
        b5c_ordinal,
    );
    assert_eq!(
        result.actual_detail_keys,
        ["declaration_symbol.label.proof_scope_input"]
    );
    assert!(result.actual_payload_keys.is_empty());
}
