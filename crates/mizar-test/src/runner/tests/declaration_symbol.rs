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
fn step5c6_admission_rejects_metadata_drift_and_keeps_three_alias_rows_outside_declaration_stage() {
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

    let alias_ids = [
        "pass_type_elaboration_antonym_predicate_001",
        "pass_type_elaboration_synonym_functor_001",
        "fail_type_elaboration_synonym_loci_mismatch_001",
    ];
    for id in alias_ids {
        let case = plan.cases.iter().find(|case| case.id.0 == id).expect("alias case");
        assert!(!super::is_active_declaration_symbol(case), "{id} is outside declaration stage");
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
    let branch_candidates = mizar_resolve::imports::ImportPathCandidate::from_surface_ast(branch_ast)
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
    let alias_candidates = mizar_resolve::imports::ImportPathCandidate::from_surface_ast(aliases_ast)
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
        mizar_resolve::imports::ImportPathCandidate::from_surface_ast(&recovered_ast).is_none(),
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
#[test]
fn step5c6_synonym_mismatch_requires_actual_unique_target_and_loci() {
    use mizar_resolve::declarations::{DeclarationShellCollector, DeclarationShellKind};
    use mizar_resolve::symbols::{
        SignatureProjectionExtractor, SymbolCollector, SymbolDiagnosticClass,
        validate_source_symbol_env,
    };
    let exact = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/miz/fail/resolve/fail_type_elaboration_synonym_loci_mismatch_001.miz"
    ));
    let (base, tail) = exact.split_once("\n\ndefinition\n").unwrap();
    let alias = format!("definition\n{tail}");
    let module = ResolverModuleId::new(
        PackageId::new("synonym-source"),
        ModulePath::new("mismatch"),
    );
    let collect = |source: &str| {
        let output = super::formula_statement::step5c8_test_frontend(source);
        assert!(
            output.diagnostics.is_empty(),
            "{source}: {:?}",
            output.diagnostics
        );
        let ast = output.ast.unwrap();
        let shells = DeclarationShellCollector::new(&ast, &module).collect();
        let result = SignatureProjectionExtractor::new(
            &ast,
            &shells,
            NamespacePath::new(module.path().as_str()),
        )
        .collect(&module);
        (ast, shells, result)
    };
    let key = vec!["declaration_symbol.notation.synonym_loci_mismatch".to_owned()];
    let mapped = |ast: &SurfaceAst,
                  shells: &mizar_resolve::declarations::DeclarationShellSet,
                  env: &SymbolEnv| {
        super::declaration_symbol::step5c6_synonym_detail_keys(ast, &module, shells, env, &key)
    };
    let expected = Some(vec!["notation.synonym.loci_mismatch".to_owned()]);
    let other =
        "definition let P,Q be set; func OtherDef: P otherbase Q -> set equals Q; coherence; end;";
    for (source, target_start) in [
        (exact.to_owned(), "func SynBase2Def"),
        (
            format!(
                "{}\n{}",
                base.replace('X', "A").replace('Y', "B"),
                alias.replace('X', "P").replace('Y', "Q").replace('Z', "R")
            ),
            "func SynBase2Def",
        ),
        (
            exact
                .replace("SynBase2Def", "Renamed")
                .replace("equals X", "equals Y"),
            "func Renamed",
        ),
        (
            format!(
                "{base}\n{other}\n{}",
                alias.replace("synbase2", "otherbase")
            ),
            "func OtherDef",
        ),
    ] {
        let (ast, shells, result) = collect(&source);
        assert_eq!(mapped(&ast, &shells, result.env()), expected, "{source}");
        let replay = SignatureProjectionExtractor::new(
            &ast,
            &shells,
            NamespacePath::new(module.path().as_str()),
        )
        .collect(&module);
        assert_eq!(result, replay);
        let [diagnostic] = result.diagnostics() else {
            panic!("sole synonym diagnostic")
        };
        assert_eq!(
            diagnostic.class(),
            SymbolDiagnosticClass::SynonymLociMismatch
        );
        let alias_shell = shells.declaration(diagnostic.shell().unwrap()).unwrap();
        assert_eq!(alias_shell.kind(), DeclarationShellKind::NotationAlias);
        assert_eq!(diagnostic.range(), alias_shell.range());
        assert_eq!(diagnostic.range().start, source.find("synonym").unwrap());
        let [original] = diagnostic.candidates() else {
            panic!("actual unique original")
        };
        let definition = result.env().definitions().by_symbol(original).unwrap();
        assert_eq!(
            definition.kind(),
            mizar_resolve::env::DefinitionKind::Functor
        );
        assert!(definition.conflict().is_none());
        let SourceAnchor::Range(range) = definition.origin().anchor() else {
            panic!("source constructor")
        };
        assert_eq!(range.start, source.find(target_start).unwrap());
        assert!(range.end < diagnostic.range().start);
        let contribution = result
            .env()
            .contributions()
            .get(definition.contribution())
            .unwrap();
        assert_eq!(contribution.effects().diagnostics(), [diagnostic.id()]);
        assert!(contribution.effects().symbols().contains(original));
    }
    for source in [
        exact.replace("synbad(X, Y, Z)", "synbad(X, Y)"),
        exact.replace("synbad(X, Y, Z)", "synbad(Y, X)"),
        exact
            .replacen("let X, Y be set;", "let X, Y, Z be set;", 1)
            .replace("X synbase2 Y", "synbase2(X,Y,Z)"),
        exact.replacen("let X, Y be set;", "let X be object; let Y be set;", 1),
        exact.replace("for X synbase2 Y", "for X synbase2"),
        exact.replace("for X synbase2 Y", "for X missingbase Y"),
        alias.clone(),
        format!("{alias}\n{base}"),
        "definition let X,Y,Z be set; func SynBase2Def: X synbase2 Y -> set equals X; coherence; synonym synbad(X,Y,Z) for X synbase2 Y; end;".to_owned(),
        format!(
            "{base}\n{}\n{alias}",
            base.replace("SynBase2Def", "Duplicate")
        ),
        format!("definition let X,Y be set; pred X synbase2 Y means X=X; end;\n{alias}"),
        exact.replace("synbad(X, Y, Z)", "synbad(X, Y, X)"),
        exact.replace("synbad(X, Y, Z)", "synbad(X, Y, Missing)"),
        exact.replace("for X synbase2 Y", "for X synbase2 X"),
        exact.replace("let X, Y, Z be set;", "let X, Y, Z be set; assume X=X;"),
    ] {
        let (ast, shells, result) = collect(&source);
        assert!(
            result
                .diagnostics()
                .iter()
                .all(|d| d.class() != SymbolDiagnosticClass::SynonymLociMismatch),
            "{source}"
        );
        assert_eq!(mapped(&ast, &shells, result.env()), None, "{source}");
        assert!(
            result
                .env()
                .symbols()
                .iter()
                .filter(|entry| entry.kind() == SymbolKind::Synonym)
                .all(|entry| entry.relations().is_empty())
        );
    }
    for (source, frontend_error) in [
        (exact.replace("for X synbase2 Y", "for"), true),
        (
            exact.replace("let X, Y, Z be set;", "let X, Y, Z be Missing;"),
            true,
        ),
        (exact.replace("synbad(X, Y, Z)", "synbad()"), false),
        (exact.replace("synbad(X, Y, Z)", "synbad((X,Y,Z))"), false),
    ] {
        let output = super::formula_statement::step5c8_test_frontend(&source);
        if frontend_error {
            assert!(!output.diagnostics.is_empty(), "{source}");
        }
        let ast = output.ast.expect("source recovery AST");
        let shells = DeclarationShellCollector::new(&ast, &module).collect();
        let result = SignatureProjectionExtractor::new(
            &ast,
            &shells,
            NamespacePath::new(module.path().as_str()),
        )
        .collect(&module);
        assert!(
            result
                .diagnostics()
                .iter()
                .all(|diagnostic| diagnostic.class() != SymbolDiagnosticClass::SynonymLociMismatch),
            "{source}"
        );
        assert_eq!(mapped(&ast, &shells, result.env()), None, "{source}");
    }
    let (ast, shells, result) = collect(exact);
    let source = mizar_resolve::resolved_ast::SurfaceResolvedArena::lower(&ast, &module).unwrap();
    let projections = SignatureProjectionExtractor::new(
        &ast,
        &shells,
        NamespacePath::new(module.path().as_str()),
    )
    .extract();
    let legacy = SymbolCollector::new(ast.source_id, &module, &shells, &projections).collect();
    assert!(legacy.diagnostics().is_empty());
    assert!(validate_source_symbol_env(&source, legacy.env()).is_err());
    assert!(validate_source_symbol_env(&source, result.env()).is_err());
    let (healthy_ast, _, healthy) = collect(&exact.replace("synbad(X, Y, Z)", "synbad(X, Y)"));
    assert!(healthy.diagnostics().is_empty());
    let healthy_source =
        mizar_resolve::resolved_ast::SurfaceResolvedArena::lower(&healthy_ast, &module).unwrap();
    assert!(validate_source_symbol_env(&healthy_source, healthy.env()).is_ok());
    let recovered =
        rebuild_surface_ast_recovering_first_token_in_kind(&ast, SurfaceNodeKind::NotationAlias);
    assert_eq!(mapped(&recovered, &shells, result.env()), None);
    assert_eq!(
        mapped(
            &ast,
            &mizar_resolve::declarations::DeclarationShellSet::default(),
            result.env()
        ),
        None
    );
    let mut foreign = ast.clone();
    let ids = InMemorySessionIdAllocator::new();
    ids.next_source_id(snapshot_id(0)).unwrap();
    foreign.source_id = ids.next_source_id(snapshot_id(0)).unwrap();
    assert_eq!(mapped(&foreign, &shells, result.env()), None);
    let foreign_module =
        ResolverModuleId::new(PackageId::new("foreign"), ModulePath::new("mismatch"));
    assert_eq!(
        super::declaration_symbol::step5c6_synonym_detail_keys(
            &ast,
            &foreign_module,
            &shells,
            result.env(),
            &key
        ),
        None
    );
    for keys in [vec![], vec![key[0].clone(), "unrelated".into()]] {
        assert_eq!(
            super::declaration_symbol::step5c6_synonym_detail_keys(
                &ast,
                &module,
                &shells,
                result.env(),
                &keys
            ),
            None
        );
    }
    let (_, _, stale) = collect(&exact.replace("synbase2", "otherbase"));
    assert_eq!(mapped(&ast, &shells, stale.env()), None);
    for mutation in 0..3 {
        let mut indexes = super::import_fixtures::clone_symbol_env_indexes(result.env());
        match mutation {
            0 => indexes.contributions = Default::default(),
            1 => indexes.definitions = Default::default(),
            _ => {
                let target = result
                    .env()
                    .symbols()
                    .iter()
                    .find(|entry| entry.kind() == SymbolKind::Functor)
                    .unwrap();
                let alias_range = result.diagnostics()[0].range();
                let origin = SemanticOrigin::new(
                    ast.source_id,
                    module.clone(),
                    SourceAnchor::Range(alias_range),
                    target.origin().structural_path().to_vec(),
                );
                let mut changed = SymbolEntry::new(
                    target.symbol().clone(),
                    target.kind(),
                    target.namespace().clone(),
                    target.primary_spelling(),
                    origin,
                    target.contribution(),
                )
                .with_visibility(target.visibility())
                .with_export_status(target.export_status())
                .with_relations(target.relations().to_vec());
                if let Some(notation) = target.notation_spelling() {
                    changed = changed.with_notation_spelling(notation);
                }
                if let Some(signature) = target.signature() {
                    changed = changed.with_signature(signature.clone());
                }
                indexes.symbols = Default::default();
                for entry in result.env().symbols().iter() {
                    indexes
                        .symbols
                        .insert(if entry.symbol() == target.symbol() {
                            changed.clone()
                        } else {
                            entry.clone()
                        });
                }
                assert_eq!(indexes.symbols.len(), result.env().symbols().len());
            }
        }
        let changed = SymbolEnv::new(module.clone(), indexes);
        assert_ne!(&changed, result.env());
        assert_eq!(mapped(&ast, &shells, &changed), None);
    }
    let (unrelated_ast, unrelated_shells, unrelated) = collect(&format!(
        "{exact}\ntheorem Extra: for X being set holds X=X; theorem Extra: for X being set holds X=X;"
    ));
    assert!(unrelated.diagnostics().len() > 1);
    assert_eq!(
        mapped(&unrelated_ast, &unrelated_shells, unrelated.env()),
        None
    );
}
