    use std::path::PathBuf;

    #[test]
    fn task31_repository_vc_debug_is_deterministic() {
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
        let (ordinal, case) = super::active_proof_verification_cases(&plan)
            .enumerate()
            .next()
            .expect("exact Task-180 proof-verification case");
        let first = super::proof_verification::generate_case_vc(&workspace_root, case, ordinal)
            .expect("first exact Task-180 VC generation");
        let second = super::proof_verification::generate_case_vc(&workspace_root, case, ordinal)
            .expect("second exact Task-180 VC generation");
        assert_eq!(first, second);
        assert_eq!(first.debug_text(), second.debug_text());
        assert_eq!(
            first.debug_text(),
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../tests/snapshots/vc/pass_proof_verification_contradiction_formula_constant_001.vc_ir.snap"
            ))
        );
    }

    #[test]
    fn task31_admission_rejects_every_reserved_case_mismatch() {
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
        let plan = build_test_plan(&config).expect("repository plan");
        let exact = plan
            .cases
            .iter()
            .find(|case| {
                case.id.0 == "pass_proof_verification_contradiction_formula_constant_001"
            })
            .expect("exact Task-180 case")
            .clone();
        let type_sidecar = plan
            .cases
            .iter()
            .find(|case| {
                case.id.0 == "pass_type_elaboration_contradiction_formula_constant_001"
            })
            .expect("unchanged type sidecar");
        assert!(!super::proof_verification::is_active_proof_verification(
            type_sidecar
        ));

        let mut variants = Vec::new();
        let mut wrong_stage = exact.clone();
        wrong_stage.expectation.stage = crate::Stage::TypeElaboration;
        variants.push(wrong_stage);
        let mut missing_tag = exact.clone();
        missing_tag.expectation.tags.clear();
        variants.push(missing_tag);
        let mut duplicate_tag = exact.clone();
        duplicate_tag
            .expectation
            .tags
            .push("active_proof_verification".to_owned());
        variants.push(duplicate_tag);
        let mut wrong_tag = exact.clone();
        wrong_tag.expectation.tags = vec!["active_type_elaboration".to_owned()];
        variants.push(wrong_tag);
        let mut wrong_phase = exact.clone();
        wrong_phase.expectation.expected_phase = Some(crate::PipelinePhase::Verification);
        variants.push(wrong_phase);
        let mut wrong_outcome = exact.clone();
        wrong_outcome.expectation.expected_outcome = crate::ExpectedOutcome::Fail;
        variants.push(wrong_outcome);
        let mut absent_snapshot = exact;
        absent_snapshot.expectation.snapshots = None;
        variants.push(absent_snapshot);

        for variant in variants {
            let mut mutated = plan.clone();
            mutated.cases = vec![variant];
            let diagnostics =
                super::proof_verification::validate_active_proof_verification_tags(&mutated);
            assert_eq!(diagnostics.len(), 1, "{:#?}", mutated.cases[0]);
            assert!(format!("{:?}", diagnostics[0]).contains("E-PROOF-VERIFICATION-ACTIVE-GATE"));
            assert_eq!(super::active_proof_verification_cases(&mutated).count(), 0);
        }
    }

    #[test]
    fn task31_snapshot_failures_and_report_projection_fail_closed() {
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
        let plan = build_test_plan(&config).expect("repository plan");
        let (ordinal, case) = super::active_proof_verification_cases(&plan)
            .enumerate()
            .next()
            .expect("exact Task-180 case");
        let vc = super::proof_verification::generate_case_vc(&workspace_root, case, ordinal)
            .expect("exact VC");
        let mut absent = case.clone();
        absent.expectation.snapshots = None;
        let absent_result = super::proof_verification::run_proof_verification_case(
            &workspace_root,
            &workspace_root.join("tests"),
            &absent,
            ordinal,
        );
        assert_eq!(
            absent_result.status,
            super::ProofVerificationCaseStatus::Failed
        );
        assert!(
            absent_result
                .failure
                .as_deref()
                .is_some_and(|failure| failure.contains("absent"))
        );

        static NEXT_VC31_TEMP_ID: std::sync::atomic::AtomicUsize =
            std::sync::atomic::AtomicUsize::new(0);
        let temp = std::env::temp_dir().join(format!(
            "mizar-test-vc31-snapshot-{}-{}",
            std::process::id(),
            NEXT_VC31_TEMP_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&temp).expect("temp directory");
        let mut missing = case.clone();
        missing.expectation.snapshots = Some(PathBuf::from("missing.snap"));
        let missing_result = super::proof_verification::run_proof_verification_case(
            &workspace_root,
            &temp,
            &missing,
            ordinal,
        );
        assert_eq!(
            missing_result.status,
            super::ProofVerificationCaseStatus::Failed
        );
        assert!(
            missing_result
                .failure
                .as_deref()
                .is_some_and(|failure| failure.contains("could not read"))
        );
        std::fs::create_dir_all(temp.join("unreadable.snap")).expect("unreadable directory");
        let mut unreadable = case.clone();
        unreadable.expectation.snapshots = Some(PathBuf::from("unreadable.snap"));
        let unreadable_result = super::proof_verification::run_proof_verification_case(
            &workspace_root,
            &temp,
            &unreadable,
            ordinal,
        );
        assert_eq!(
            unreadable_result.status,
            super::ProofVerificationCaseStatus::Failed
        );
        std::fs::write(temp.join("mismatch.snap"), "wrong\n").expect("mismatch baseline");
        let mut mismatch_plan = plan.clone();
        let mismatch_case = mismatch_plan
            .cases
            .iter_mut()
            .find(|candidate| candidate.id == case.id)
            .expect("exact case in cloned plan");
        mismatch_case.expectation.snapshots = Some(PathBuf::from("mismatch.snap"));
        let mismatch_report = super::run_proof_verification_plan(
            &workspace_root,
            &temp,
            &mismatch_plan,
        );
        assert_eq!(mismatch_report.results.len(), 1);
        assert_eq!(
            mismatch_report.results[0].status,
            super::ProofVerificationCaseStatus::Failed
        );
        assert!(
            mismatch_report.results[0]
                .failure
                .as_deref()
                .is_some_and(|failure| failure.contains("differed"))
        );
        assert!(mismatch_report.diagnostics.iter().any(|diagnostic| {
            diagnostic.code.0 == "E-PROOF-VERIFICATION-CASE"
                && diagnostic.detail_key
                    == "proof_verification.pass_proof_verification_contradiction_formula_constant_001"
        }));
        assert_eq!(vc.vcs().len(), 1);
        std::fs::remove_dir_all(temp).expect("remove temp directory");
    }

    #[test]
    fn task31_repository_proof_verification_report_passes_exactly_one_case() {
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
        let report = super::run_proof_verification_corpus(&config).expect("proof report");
        assert_eq!(report.results.len(), 1);
        assert_eq!(report.passed_count(), 1);
        assert_eq!(report.failed_count(), 0);
        assert_eq!(report.error_count(), 0, "{:#?}", report.diagnostics);
    }
