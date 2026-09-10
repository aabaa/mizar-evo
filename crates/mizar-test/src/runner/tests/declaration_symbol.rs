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
    assert_eq!(active.len(), 11);

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

fn step5c6_plan() -> (std::path::PathBuf, crate::harness::TestPlan) {
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
    (
        workspace_root,
        build_test_plan(&config).expect("Step 5C.6 plan"),
    )
}

#[test]
fn step5c6_four_module_cases_have_exact_outcomes_and_keys() {
    let (workspace_root, plan) = step5c6_plan();
    let expected = [
        (
            "fail_declaration_symbol_import_duplicate_alias_001",
            ["modules.import.duplicate_alias"].as_slice(),
        ),
        ("pass_declaration_symbol_branch_import_form_001", [].as_slice()),
        (
            "fail_declaration_symbol_import_unknown_module_001",
            ["modules.import.unknown_module"].as_slice(),
        ),
        (
            "pass_declaration_symbol_private_theorem_visibility_001",
            [].as_slice(),
        ),
    ];
    for (id, detail_keys) in expected {
        let (ordinal, case) = super::active_declaration_symbol_cases(&plan)
            .enumerate()
            .find(|(_, case)| case.id.0 == id)
            .expect("active Step 5C.6 case");
        let result = super::declaration_symbol::run_declaration_symbol_case(
            &workspace_root,
            case,
            ordinal,
        );
        assert_eq!(result.status, super::DeclarationSymbolCaseStatus::Passed, "{id}");
        assert_eq!(result.actual_detail_keys, detail_keys, "{id}");
    }
}

#[test]
fn step5c6_admission_rejects_single_metadata_drift_and_keeps_three_gaps_inactive() {
    use crate::expectation::{ExpectedOutcome, PipelinePhase};

    let (workspace_root, plan) = step5c6_plan();
    let original = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "pass_declaration_symbol_branch_import_form_001")
        .expect("branch-import admission")
        .clone();
    assert!(super::is_active_declaration_symbol(&original));

    let mut missing_tag = original.clone();
    missing_tag.expectation.tags.clear();
    let mut extra_tag = original.clone();
    extra_tag.expectation.tags.push("extra".to_owned());
    let mut changed_id = original.clone();
    changed_id.id.0.push_str("_changed");
    let mut changed_path = original.clone();
    changed_path.source_path = workspace_root.join("tests/miz/pass/resolve/not-this-case.miz");
    let mut changed_outcome = original.clone();
    changed_outcome.expectation.expected_outcome = ExpectedOutcome::Fail;
    let mut changed_phase = original.clone();
    changed_phase.expectation.expected_phase = Some(PipelinePhase::TypeCheck);
    let mut changed_key = original.clone();
    changed_key.expectation.stable_detail_key = Some("modules.import.wrong_key".to_owned());
    for (label, case) in [
        ("missing tag", missing_tag),
        ("extra tag", extra_tag),
        ("changed id", changed_id.clone()),
        ("changed path", changed_path),
        ("changed outcome", changed_outcome),
        ("changed phase", changed_phase),
        ("changed key", changed_key),
    ] {
        assert!(!super::is_active_declaration_symbol(&case), "{label}");
    }
    let mut invalid_plan = plan.clone();
    invalid_plan.cases.push(changed_id);
    assert!(
        super::validate_active_declaration_symbol_tags(&workspace_root, &invalid_plan)
            .iter()
            .any(|diagnostic| diagnostic
                .detail_key
                .ends_with("pass_declaration_symbol_branch_import_form_001_changed"))
    );

    let inactive_gap_ids = [
        "pass_type_elaboration_antonym_predicate_001",
        "pass_type_elaboration_synonym_functor_001",
        "fail_type_elaboration_synonym_loci_mismatch_001",
    ];
    for id in inactive_gap_ids {
        let case = plan.cases.iter().find(|case| case.id.0 == id).expect("inactive gap");
        assert!(!super::is_active_declaration_symbol(case), "{id} must remain inactive");
    }
}

#[test]
fn step5c6_import_candidates_preserve_alias_branch_provenance_and_reject_recovery() {
    let (workspace_root, plan) = step5c6_plan();
    let run = |id: &str| {
        let (ordinal, case) = plan
            .cases
            .iter()
            .enumerate()
            .find(|(_, case)| case.id.0 == id)
            .expect("candidate source case");
        run_frontend(&workspace_root, case, ordinal).expect("frontend output")
    };

    let branch = run("pass_declaration_symbol_branch_import_form_001");
    let branch_ast = branch.ast.as_ref().expect("branch AST");
    let branch_candidates = super::declaration_symbol::import_path_candidates(branch_ast)
        .expect("branch candidates");
    assert_eq!(branch_candidates.len(), 1);
    let candidate = &branch_candidates[0];
    assert_eq!(candidate.components(), ["parser", "type_fixtures"]);
    assert_eq!(candidate.branch_base_range().unwrap().start, 7);
    assert_eq!(candidate.branch_base_range().unwrap().end, 13);
    assert_eq!(candidate.branch_member_range().unwrap().start, 15);
    assert_eq!(candidate.branch_member_range().unwrap().end, 28);
    let mut fixture_case = plan.cases[0].clone();
    fixture_case.source_path = workspace_root.join("crates/mizar-test/tests/testdata/parser/type_fixtures.miz");
    let fixture = run_frontend(&workspace_root, &fixture_case, 9001).expect("fixture frontend");
    assert!(fixture.diagnostics.is_empty());
    let fixture_ast = fixture.ast.as_ref().expect("fixture AST");
    let fixture_env = resolver_symbol_collection(&workspace_root, &fixture_case, fixture_ast).env;
    assert!(fixture_env.symbols().iter().any(|entry| {
        entry.primary_spelling() == "X divides Y"
            && entry.kind() == mizar_resolve::env::SymbolKind::Predicate
            && entry.visibility() == mizar_resolve::env::Visibility::Public
            && entry.export_status() == mizar_resolve::env::ExportStatus::Exported
            && entry.notation_spelling() == Some("X divides Y")
    }));

    let aliases = run("fail_declaration_symbol_import_duplicate_alias_001");
    let aliases_ast = aliases.ast.as_ref().expect("alias AST");
    let alias_candidates = super::declaration_symbol::import_path_candidates(aliases_ast)
        .expect("alias candidates");
    assert_eq!(alias_candidates.len(), 2);
    assert_eq!((alias_candidates[0].alias(), alias_candidates[1].alias()), (Some("dupx"), Some("dupx")));
    let first_alias = aliases.source_text.find("dupx").expect("first alias");
    let second_alias = aliases.source_text[first_alias + 4..]
        .find("dupx")
        .map(|offset| first_alias + 4 + offset)
        .expect("second alias");
    assert_eq!((alias_candidates[0].alias_range().unwrap().start, alias_candidates[1].alias_range().unwrap().start), (first_alias, second_alias));
    assert_eq!((alias_candidates[0].alias_range().unwrap().end, alias_candidates[1].alias_range().unwrap().end), (first_alias + 4, second_alias + 4));
    assert_eq!((alias_candidates[0].ordinal(), alias_candidates[1].ordinal()), (0, 1));

    let recovered_ast = rebuild_surface_ast_recovering_first_token_in_kind(
        branch_ast,
        mizar_syntax::SurfaceNodeKind::ImportItem,
    );
    assert!(recovered_ast.node_views().any(|node| node.is_recovered()));
    assert!(
        super::declaration_symbol::import_path_candidates(&recovered_ast).is_none(),
        "recovered import syntax must not reach module resolution"
    );
}

#[test]
fn step5c6_fixture_absence_and_private_citation_boundaries_fail_closed() {
    use mizar_resolve::env::{ExportStatus, SymbolEntry, SymbolEnv, SymbolIndex, Visibility};
    use mizar_resolve::resolved_ast::{ModuleId, SemanticOrigin};
    use mizar_session::{ModulePath, PackageId};

    let (workspace_root, plan) = step5c6_plan();
    let scratch = std::env::temp_dir().join(format!(
        "mizar-step5c6-citation-{}-{}",
        std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos(),
    ));
    std::fs::create_dir(&scratch).expect("fresh citation workspace");
    let branch = plan.cases.iter().find(|case| {
        case.id.0 == "pass_declaration_symbol_branch_import_form_001"
    }).expect("branch import case");
    let missing_fixture = super::declaration_symbol::run_declaration_symbol_case(&scratch, branch, 9002);
    assert_eq!(missing_fixture.status, super::DeclarationSymbolCaseStatus::Failed);
    assert_eq!(missing_fixture.actual_detail_keys, ["declaration_symbol.module_semantics.input"]);
    let (ordinal, case) = plan
        .cases
        .iter()
        .enumerate()
        .find(|(_, case)| case.id.0 == "pass_declaration_symbol_private_theorem_visibility_001")
        .expect("private theorem case");
    let output = run_frontend(&workspace_root, case, ordinal).expect("private theorem frontend");
    let ast = output.ast.as_ref().expect("private theorem AST");
    let resolver = resolver_symbol_collection(&workspace_root, case, ast);
    let private = resolver
        .env
        .symbols()
        .iter()
        .find(|entry| entry.primary_spelling() == "PrivT1")
        .expect("private theorem symbol");
    assert_eq!(private.visibility(), Visibility::Private);
    assert_eq!(private.export_status(), ExportStatus::LocalOnly);
    assert_eq!(private.origin().source_id(), ast.source_id);
    assert_eq!(private.origin().module_id(), &resolver.module);
    let reference_start = surface_nodes_with_kind(ast, mizar_syntax::SurfaceNodeKind::Reference)
        .first()
        .expect("private citation")
        .1
        .range
        .start;
    assert!(matches!(private.origin().anchor(), mizar_session::SourceAnchor::Range(range) if range.end < reference_start));
    let rebuild_env = |visibility, export_status, origin: SemanticOrigin| {
        let mut symbols = SymbolIndex::new();
        for entry in resolver.env.symbols().iter() {
            let is_private = entry.primary_spelling() == "PrivT1";
            symbols.insert(
                SymbolEntry::new(
                    entry.symbol().clone(),
                    entry.kind(),
                    entry.namespace().clone(),
                    entry.primary_spelling(),
                    if is_private { origin.clone() } else { entry.origin().clone() },
                    entry.contribution(),
                )
                .with_visibility(if is_private { visibility } else { entry.visibility() })
                .with_export_status(if is_private { export_status } else { entry.export_status() }),
            );
        }
        let mut indexes = super::import_fixtures::clone_symbol_env_indexes(&resolver.env);
        indexes.symbols = symbols;
        SymbolEnv::new(resolver.module.clone(), indexes)
    };
    let public_env = rebuild_env(Visibility::Public, ExportStatus::LocalOnly, private.origin().clone());
    assert!(!super::declaration_symbol::private_theorem_is_valid(
        ast, &resolver.module, &resolver.shells, &public_env,
    ));
    let exported_env = rebuild_env(Visibility::Private, ExportStatus::Exported, private.origin().clone());
    assert!(!super::declaration_symbol::private_theorem_is_valid(
        ast, &resolver.module, &resolver.shells, &exported_env,
    ));
    let foreign_module = ModuleId::new(PackageId::new("foreign"), ModulePath::new("foreign.module"));
    let foreign_origin = SemanticOrigin::new(
        private.origin().source_id(),
        foreign_module,
        private.origin().anchor().clone(),
        private.origin().structural_path().to_vec(),
    );
    let foreign_env = rebuild_env(Visibility::Private, ExportStatus::LocalOnly, foreign_origin);
    assert!(!super::declaration_symbol::private_theorem_is_valid(
        ast, &resolver.module, &resolver.shells, &foreign_env,
    ));

    let control = rebuild_env(Visibility::Private, ExportStatus::LocalOnly, private.origin().clone());
    assert!(super::declaration_symbol::private_theorem_is_valid(
        ast, &resolver.module, &resolver.shells, &control,
    ));
    let foreign_container = SymbolEnv::new(
        ModuleId::new(PackageId::new("foreign"), ModulePath::new("foreign")),
        super::import_fixtures::clone_symbol_env_indexes(&resolver.env),
    );
    assert!(!super::declaration_symbol::private_theorem_is_valid(
        ast, &resolver.module, &resolver.shells, &foreign_container,
    ));

    let mut probe = case.clone();
    probe.source_path = scratch.join("citation.miz");
    let source = output.source_text.to_string();
    let split = source.find("theorem UsePriv1:").expect("later theorem");
    for (text, expected) in [
        (source.clone(), true),
        (source.replace("by PrivT1", "by Other1"), false),
        (source.replace("  thus X = X by PrivT1;", "  PrivT1: X = X;\n  thus X = X by PrivT1;"), false),
        (format!("{}\n{}", &source[split..], &source[..split]), false),
    ] {
        std::fs::write(&probe.source_path, text).expect("private citation probe");
        let frontend = run_frontend(&scratch, &probe, ordinal).expect("probe frontend");
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend.ast.as_ref().expect("probe AST");
        let resolver = resolver_symbol_collection(&scratch, &probe, ast);
        assert!(resolver.detail_keys.is_empty());
        assert_eq!(
            super::declaration_symbol::private_theorem_is_valid(
                ast, &resolver.module, &resolver.shells, &resolver.env,
            ),
            expected,
            "citation probe: {}", frontend.source_text,
        );
    }
    std::fs::remove_file(&probe.source_path).expect("remove owned probe");
    std::fs::remove_dir(&scratch).expect("remove empty owned workspace");
}
