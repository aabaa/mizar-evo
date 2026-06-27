use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::PathBuf,
};

#[test]
fn atp_manifest_opts_into_workspace_lints() {
    let manifest_path = crate_root().join("Cargo.toml");
    let manifest = read_to_string(&manifest_path);
    let lints = section(&manifest, "lints");

    assert!(
        lints
            .iter()
            .any(|line| assignment_is(line, "workspace", "true")),
        "{} must keep [lints] workspace = true so cargo build/test and clippy \
         use the shared lint policy",
        manifest_path.display()
    );
}

#[test]
fn workspace_manifest_includes_mizar_atp_once() {
    let manifest_path = workspace_root().join("Cargo.toml");
    let manifest = read_to_string(&manifest_path);
    let members = workspace_members(&manifest);
    let occurrences = members
        .iter()
        .filter(|member| **member == "crates/mizar-atp")
        .count();

    assert_eq!(
        occurrences,
        1,
        "{} must list crates/mizar-atp exactly once in [workspace].members",
        manifest_path.display()
    );
}

#[test]
fn atp_manifest_keeps_task_one_package_metadata() {
    let manifest_path = crate_root().join("Cargo.toml");
    let manifest = read_to_string(&manifest_path);
    let package = section(&manifest, "package");
    let lib = section(&manifest, "lib");

    assert!(
        package
            .iter()
            .any(|line| assignment_is(line, "name", "mizar-atp")),
        "{} must keep the package name stable",
        manifest_path.display()
    );
    assert!(
        package
            .iter()
            .any(|line| assignment_is(line, "version", "0.1.0")),
        "{} must keep the task-1 crate version explicit until a release-policy \
         task changes it",
        manifest_path.display()
    );
    for key in [
        "edition.workspace",
        "license.workspace",
        "repository.workspace",
    ] {
        assert!(
            package.iter().any(|line| assignment_is(line, key, "true")),
            "{} must inherit {key} from the workspace",
            manifest_path.display()
        );
    }
    assert!(
        lib.iter()
            .any(|line| assignment_is(line, "name", "mizar_atp")),
        "{} must keep the library crate name stable",
        manifest_path.display()
    );
    assert!(
        lib.iter()
            .any(|line| assignment_is(line, "path", "src/lib.rs")),
        "{} must keep the task-1 library entry point at src/lib.rs",
        manifest_path.display()
    );
}

#[test]
fn atp_manifest_dependency_boundary_is_task_one_minimal() {
    let manifest_path = crate_root().join("Cargo.toml");
    let manifest = read_to_string(&manifest_path);
    let dependency_sections = dependency_sections(&manifest);

    assert_eq!(
        dependency_sections,
        [(
            "dependencies".to_owned(),
            vec![
                "mizar-core = { path = \"../mizar-core\" }",
                "mizar-kernel = { path = \"../mizar-kernel\" }",
                "mizar-session = { path = \"../mizar-session\" }",
                "mizar-vc = { path = \"../mizar-vc\" }",
            ],
        )],
        "{} must keep task-1 production dependencies limited to mizar-core, \
         mizar-kernel, mizar-session, and mizar-vc; dev/build/target \
         dependency sections require a later explicit task",
        manifest_path.display()
    );
}

#[test]
fn atp_lib_exposes_only_spec_backed_modules() {
    let lib_path = crate_root().join("src/lib.rs");
    let source = read_to_string(&lib_path);
    let expected_source = r#"//! ATP candidate-evidence production boundary.
//!
//! `mizar-atp` owns pipeline phase 13: translating `NeedsAtp` VC obligations
//! into backend-neutral ATP problems, running untrusted backends, and
//! collecting formula/substitution evidence candidates for `mizar-kernel`.
//!
//! This crate does not accept proofs, select trusted winners, call the kernel
//! as proof authority, or expose backend proof methods as trusted material.

#![forbid(unsafe_code)]

pub mod backend;
pub mod portfolio;
pub mod problem;
pub mod property_encoding;
pub mod smtlib_encoder;
pub mod tptp_encoder;
pub mod translator;
"#;
    let source_files = rust_source_files(&crate_root().join("src"))
        .into_iter()
        .map(|path| {
            path.strip_prefix(crate_root())
                .expect("source path lives in crate root")
                .display()
                .to_string()
        })
        .collect::<Vec<_>>();

    assert_eq!(
        source,
        expected_source,
        "{} must expose only spec-backed mizar-atp modules",
        lib_path.display()
    );
    assert_eq!(
        source_files,
        [
            "src/backend/tests.rs",
            "src/backend.rs",
            "src/lib.rs",
            "src/portfolio/tests.rs",
            "src/portfolio.rs",
            "src/problem/tests.rs",
            "src/problem.rs",
            "src/property_encoding/tests.rs",
            "src/property_encoding.rs",
            "src/smtlib_encoder/tests.rs",
            "src/smtlib_encoder.rs",
            "src/tptp_encoder/tests.rs",
            "src/tptp_encoder.rs",
            "src/translator/tests.rs",
            "src/translator.rs"
        ],
        "ATP production modules require paired specs before source, and task-27 \
         private test modules must stay cfg(test) children of those modules; \
         found {source_files:?}"
    );
}

#[test]
fn atp_crate_tree_contains_only_current_spec_backed_files() {
    let mut files = crate_files()
        .into_iter()
        .filter(|file| file != "Cargo.lock")
        .collect::<Vec<_>>();
    files.sort();

    assert_eq!(
        files,
        [
            "Cargo.toml",
            "src/backend.rs",
            "src/backend/tests.rs",
            "src/lib.rs",
            "src/portfolio.rs",
            "src/portfolio/tests.rs",
            "src/problem.rs",
            "src/problem/tests.rs",
            "src/property_encoding.rs",
            "src/property_encoding/tests.rs",
            "src/smtlib_encoder.rs",
            "src/smtlib_encoder/tests.rs",
            "src/tptp_encoder.rs",
            "src/tptp_encoder/tests.rs",
            "src/translator.rs",
            "src/translator/tests.rs",
            "tests/determinism_suite.rs",
            "tests/lint_policy.rs",
            "tests/mock_backend_corpus.rs"
        ],
        "mizar-atp crate files must stay limited to current spec-backed sources; \
         build scripts, examples, benches, extra tests beyond the task-21 \
         determinism suite, kernel/proof behavior, or other crate-root files \
         require explicit spec \
         tasks; found {files:?}"
    );
}

#[test]
fn atp_backend_module_has_paired_specs_and_excludes_trusted_acceptance() {
    let en_spec = workspace_root().join("doc/design/mizar-atp/en/backend.md");
    let ja_spec = workspace_root().join("doc/design/mizar-atp/ja/backend.md");
    let source_path = crate_root().join("src/backend.rs");
    let en = read_to_string(&en_spec);
    let ja = read_to_string(&ja_spec);
    let source = read_to_string(&source_path);
    let tests = read_to_string(&crate_root().join("src/backend/tests.rs"));

    for marker in [
        "Stable Hashes And Fingerprints",
        "Task-14 source must represent unsupported limits as either",
        "payload bytes or an explicit payload reference",
        "Task-14 Test Expectations",
    ] {
        assert!(
            en.contains(marker),
            "{} must keep task-13/task-14 backend spec marker `{marker}`",
            en_spec.display()
        );
    }
    for marker in [
        "stable hash と fingerprint",
        "unsupported limit を `best_effort` または `required`",
        "payload bytes または明示的な payload reference",
        "Task-14 Test Expectations",
    ] {
        assert!(
            ja.contains(marker),
            "{} must keep task-13/task-14 Japanese backend spec marker `{marker}`",
            ja_spec.display()
        );
    }
    for marker in [
        "pub fn run_backend",
        "pub fn classify_backend_observation",
        "process::{Command, ExitStatus, Stdio}",
        "FormulaSubstitutionBytes",
        "required_resource_limit_unsupported",
        "mizar-atp/backend-command/v1",
        "PrivateProblemFile::create",
        "proved_rejected_process_status",
    ] {
        assert!(
            source.contains(marker),
            "{} must implement backend.md task-14 marker `{marker}`",
            source_path.display()
        );
    }
    for marker in [
        "BackendCandidatePayload::FormulaSubstitutionBytes",
        "proved_rejected_process_status",
    ] {
        assert!(
            tests.contains(marker),
            "{} must keep task-14 backend unit-test marker `{marker}`",
            crate_root().join("src/backend/tests.rs").display()
        );
    }
    for prohibited in [
        "mizar_kernel::",
        "kernel_verified",
        "ProofWitness",
        "ProofPolicy",
        "pub mod proof_policy",
        "cache_promotion",
        "trusted_used_axioms",
        "accepted_proof_status",
    ] {
        assert!(
            !source.contains(prohibited),
            "{} must not expose prohibited trusted/downstream material `{prohibited}`",
            source_path.display()
        );
    }
}

#[test]
fn atp_backend_public_api_surface_is_task_fourteen_allowlist() {
    let source_path = crate_root().join("src/backend.rs");
    let source = read_to_string(&source_path);
    let public_items = public_api_items(&source);
    let public_fields = public_struct_fields(&source);
    let public_functions = public_api_functions(&source);
    let expected = [
        "BackendCancellationToken",
        "BackendCandidateEvidence",
        "BackendCandidatePayload",
        "BackendCommand",
        "BackendConfigError",
        "BackendCounterexample",
        "BackendDiagnostic",
        "BackendEnvironmentPolicy",
        "BackendExitStatus",
        "BackendIoMode",
        "BackendKind",
        "BackendLimitRequirement",
        "BackendObservation",
        "BackendObservedResult",
        "BackendProfile",
        "BackendProfileId",
        "BackendResourceLimits",
        "BackendRunId",
        "BackendRunInput",
        "BackendRunMetadata",
        "BackendRunResult",
        "BackendRunStatus",
        "BackendStreamCapture",
        "BackendTermination",
        "BackendVersionProbe",
        "BackendVersionRecord",
        "BackendWorkingDirectoryPolicy",
        "EncodedBackendProblem",
        "EncodedBackendProblemParts",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();

    assert_eq!(
        public_items,
        expected,
        "{} public API must stay limited to task-14 generic backend runner, \
         task-19 run metadata projection, mock classification, deterministic \
         run metadata, and fail-closed error/status shapes; real backend \
         adapters, proof parsers, kernel checks, witnesses, proof policy, and \
         cache handles require later specs",
        source_path.display()
    );

    assert_eq!(
        public_fields,
        [
            "EncodedBackendProblemParts::concrete_format",
            "EncodedBackendProblemParts::expected_result",
            "EncodedBackendProblemParts::formula_labels",
            "EncodedBackendProblemParts::input_text",
            "EncodedBackendProblemParts::logic_fragment",
            "EncodedBackendProblemParts::logic_profile_name",
            "EncodedBackendProblemParts::problem_id",
            "EncodedBackendProblemParts::provenance_hash",
            "EncodedBackendProblemParts::symbol_bindings",
            "EncodedBackendProblemParts::target_binding",
        ],
        "{} public struct fields must stay limited to the task-14 encoded \
         problem construction parts; run results, candidates, statuses, and \
         process metadata must remain opaque",
        source_path.display()
    );

    assert_eq!(
        public_functions,
        [
            "BackendCancellationToken::cancel",
            "BackendCancellationToken::is_cancelled",
            "BackendCancellationToken::new",
            "BackendCandidateEvidence::candidate_id",
            "BackendCandidateEvidence::encoded_problem_hash",
            "BackendCandidateEvidence::formula_label_refs",
            "BackendCandidateEvidence::new",
            "BackendCandidateEvidence::payload",
            "BackendCandidateEvidence::provenance_hash",
            "BackendCandidateEvidence::symbol_binding_refs",
            "BackendCandidateEvidence::target_binding",
            "BackendCandidatePayload::is_formula_substitution_candidate",
            "BackendCommand::args",
            "BackendCommand::environment",
            "BackendCommand::executable",
            "BackendCommand::new",
            "BackendCommand::semantic_executable_id",
            "BackendCommand::with_environment",
            "BackendCommand::with_semantic_executable_id",
            "BackendCommand::with_working_directory",
            "BackendCommand::working_directory",
            "BackendCounterexample::new",
            "BackendCounterexample::payload",
            "BackendCounterexample::provenance_mapped",
            "BackendDiagnostic::key",
            "BackendDiagnostic::message",
            "BackendDiagnostic::new",
            "BackendEnvironmentPolicy::new",
            "BackendEnvironmentPolicy::vars",
            "BackendExitStatus::code",
            "BackendExitStatus::success",
            "BackendKind::as_str",
            "BackendKind::new",
            "BackendObservation::new",
            "BackendObservation::with_candidate_evidence",
            "BackendObservation::with_counterexample",
            "BackendObservation::without_complete_output_requirement",
            "BackendProfile::backend_kind",
            "BackendProfile::concrete_format",
            "BackendProfile::deterministic_priority",
            "BackendProfile::new",
            "BackendProfile::profile_id",
            "BackendProfile::requires_candidate_evidence",
            "BackendProfile::version_probe",
            "BackendProfile::with_deterministic_priority",
            "BackendProfile::with_version_probe",
            "BackendProfileId::as_str",
            "BackendProfileId::new",
            "BackendResourceLimits::kill_grace",
            "BackendResourceLimits::new",
            "BackendResourceLimits::platform_limits",
            "BackendResourceLimits::stderr_bytes",
            "BackendResourceLimits::stdout_bytes",
            "BackendResourceLimits::wall_timeout",
            "BackendResourceLimits::with_kill_grace",
            "BackendResourceLimits::with_stderr_limit",
            "BackendResourceLimits::with_stdout_limit",
            "BackendResourceLimits::with_unsupported_platform_limit",
            "BackendResourceLimits::with_wall_timeout",
            "BackendRunId::as_str",
            "BackendRunId::new",
            "BackendRunInput::command",
            "BackendRunInput::encoded_problem",
            "BackendRunInput::io_mode",
            "BackendRunInput::new",
            "BackendRunInput::profile",
            "BackendRunInput::resource_limits",
            "BackendRunInput::run_id",
            "BackendRunInput::with_random_seed",
            "BackendRunMetadata::args",
            "BackendRunMetadata::backend_kind",
            "BackendRunMetadata::child_reaped",
            "BackendRunMetadata::command_fingerprint",
            "BackendRunMetadata::concrete_format",
            "BackendRunMetadata::diagnostics",
            "BackendRunMetadata::elapsed",
            "BackendRunMetadata::encoded_input_hash",
            "BackendRunMetadata::encoded_metadata_hash",
            "BackendRunMetadata::environment",
            "BackendRunMetadata::exit_status",
            "BackendRunMetadata::io_mode",
            "BackendRunMetadata::observed_result",
            "BackendRunMetadata::problem_id",
            "BackendRunMetadata::profile_id",
            "BackendRunMetadata::random_seed",
            "BackendRunMetadata::resource_limits",
            "BackendRunMetadata::run_id",
            "BackendRunMetadata::semantic_executable_id",
            "BackendRunMetadata::status",
            "BackendRunMetadata::stderr",
            "BackendRunMetadata::stdout",
            "BackendRunMetadata::termination",
            "BackendRunMetadata::version_record",
            "BackendRunMetadata::working_directory_policy_kind",
            "BackendRunResult::backend_kind",
            "BackendRunResult::candidate_evidence",
            "BackendRunResult::child_reaped",
            "BackendRunResult::command_fingerprint",
            "BackendRunResult::counterexample",
            "BackendRunResult::diagnostics",
            "BackendRunResult::elapsed",
            "BackendRunResult::encoded_problem",
            "BackendRunResult::exit_status",
            "BackendRunResult::metadata",
            "BackendRunResult::observed_result",
            "BackendRunResult::profile_id",
            "BackendRunResult::run_id",
            "BackendRunResult::status",
            "BackendRunResult::stderr",
            "BackendRunResult::stdout",
            "BackendRunResult::termination",
            "BackendRunResult::version_record",
            "BackendStreamCapture::hash",
            "BackendStreamCapture::incomplete",
            "BackendStreamCapture::retained_bytes",
            "BackendStreamCapture::total_bytes",
            "BackendStreamCapture::truncated",
            "BackendVersionProbe::args",
            "BackendVersionProbe::executable",
            "BackendVersionProbe::new",
            "BackendVersionProbe::timeout",
            "BackendVersionRecord::diagnostics",
            "BackendVersionRecord::exit_status",
            "BackendVersionRecord::parsed_version",
            "BackendVersionRecord::stderr",
            "BackendVersionRecord::stdout",
            "BackendVersionRecord::success",
            "EncodedBackendProblem::concrete_format",
            "EncodedBackendProblem::expected_result",
            "EncodedBackendProblem::formula_labels",
            "EncodedBackendProblem::input_hash",
            "EncodedBackendProblem::input_text",
            "EncodedBackendProblem::logic_fragment",
            "EncodedBackendProblem::logic_profile_name",
            "EncodedBackendProblem::metadata_hash",
            "EncodedBackendProblem::new",
            "EncodedBackendProblem::problem_id",
            "EncodedBackendProblem::provenance_hash",
            "EncodedBackendProblem::symbol_bindings",
            "EncodedBackendProblem::target_binding",
            "classify_backend_observation",
            "run_backend",
        ],
        "{} public functions and methods must stay limited to task-14 generic \
         runner, task-19 run metadata projection, and mock-classification \
         construction/accessors",
        source_path.display()
    );
}

#[test]
fn atp_portfolio_module_has_paired_specs_and_excludes_trusted_material() {
    let en_spec = workspace_root().join("doc/design/mizar-atp/en/portfolio.md");
    let ja_spec = workspace_root().join("doc/design/mizar-atp/ja/portfolio.md");
    let source_path = crate_root().join("src/portfolio.rs");
    let en = read_to_string(&en_spec);
    let ja = read_to_string(&ja_spec);
    let source = read_to_string(&source_path);
    let tests = read_to_string(&crate_root().join("src/portfolio/tests.rs"));

    for marker in [
        "Task 18 implements only the already-built run slice",
        "Task-20 Corpus And Mock-Backend Coverage",
        "Task-21 Determinism Suite",
        "## Result Matching",
        "Task-18 Test Coverage",
    ] {
        assert!(
            en.contains(marker),
            "{} must keep task-17/task-18 portfolio spec marker `{marker}`",
            en_spec.display()
        );
    }
    for marker in [
        "task 18 は planning のうち、already-built run slice だけを実装する",
        "task-20 corpus and mock-backend coverage",
        "task-21 determinism suite",
        "## result matching",
        "task-18 test coverage",
    ] {
        assert!(
            ja.contains(marker),
            "{} must keep task-17/task-18 Japanese portfolio spec marker `{marker}`",
            ja_spec.display()
        );
    }
    for marker in [
        "pub fn plan_portfolio",
        "pub fn collect_portfolio_results",
        "PortfolioStopReason::Cancelled",
        "BackendRunInput",
        "BackendRunResult",
        "mizar-atp/portfolio-plan/v1",
    ] {
        assert!(
            source.contains(marker),
            "{} must implement portfolio.md task-18 marker `{marker}`",
            source_path.display()
        );
    }
    assert!(
        tests.contains("portfolio_source_excludes_trusted_or_downstream_boundary_strings"),
        "{} must keep the task-18 prohibited-boundary unit-test marker",
        crate_root().join("src/portfolio/tests.rs").display()
    );
    for prohibited in [
        "mizar_kernel::",
        "kernel_verified",
        "KernelCheckResult",
        "ProofWitness",
        "ProofWitnessDraft",
        "ProofPolicyEvaluator",
        "proof_cache",
        "cache_promotion",
        "accepted_proof_status",
        "caller_supplied",
        "instantiated_formulas",
        "SatProblem",
        "DIMACS",
        "real_output_extraction",
        "MiniSAT",
    ] {
        assert!(
            !source.contains(prohibited),
            "{} must not expose prohibited trusted/downstream material `{prohibited}`",
            source_path.display()
        );
    }
}

#[test]
fn atp_portfolio_public_api_surface_is_task_eighteen_allowlist() {
    let source_path = crate_root().join("src/portfolio.rs");
    let source = read_to_string(&source_path);
    let public_items = public_api_items(&source);
    let public_fields = public_struct_fields(&source);
    let public_functions = public_api_functions(&source);
    let expected = [
        "PortfolioBudget",
        "PortfolioCandidate",
        "PortfolioCandidateId",
        "PortfolioCandidateKind",
        "PortfolioDiagnostic",
        "PortfolioError",
        "PortfolioEvidenceFormat",
        "PortfolioEvidenceSet",
        "PortfolioId",
        "PortfolioInput",
        "PortfolioInputParts",
        "PortfolioPlan",
        "PortfolioPolicyConstraints",
        "PortfolioStopReason",
        "PortfolioStopSummary",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();

    assert_eq!(
        public_items,
        expected,
        "{} public API must stay limited to task-18 portfolio planning, \
         candidate collection, deterministic evidence-set metadata, and \
         fail-closed error/status shapes; kernel checks, proof policy, \
         witnesses, cache handles, and real backend extractors require later \
         owner specs",
        source_path.display()
    );

    assert_eq!(
        public_fields,
        [
            "PortfolioInputParts::atp_problem",
            "PortfolioInputParts::backend_runs",
            "PortfolioInputParts::cancellation",
            "PortfolioInputParts::obligation_budget",
            "PortfolioInputParts::policy_constraints",
            "PortfolioInputParts::portfolio_id",
            "PortfolioInputParts::scheduler_budget",
            "PortfolioInputParts::vc_hash",
        ],
        "{} public struct fields must stay limited to explicit task-18 \
         portfolio construction inputs",
        source_path.display()
    );

    assert_eq!(
        public_functions,
        [
            "PortfolioBudget::max_scheduled_runs",
            "PortfolioBudget::unbounded",
            "PortfolioBudget::with_max_scheduled_runs",
            "PortfolioCandidate::backend_profile_id",
            "PortfolioCandidate::candidate_hash",
            "PortfolioCandidate::candidate_id",
            "PortfolioCandidate::candidate_kind",
            "PortfolioCandidate::diagnostics",
            "PortfolioCandidate::encoded_problem_hash",
            "PortfolioCandidate::evidence_format",
            "PortfolioCandidate::evidence_payload_hash",
            "PortfolioCandidate::observed_result",
            "PortfolioCandidate::provenance_hash",
            "PortfolioCandidate::source_run_id",
            "PortfolioCandidate::target_binding",
            "PortfolioCandidateId::as_str",
            "PortfolioDiagnostic::key",
            "PortfolioDiagnostic::message",
            "PortfolioDiagnostic::new",
            "PortfolioEvidenceSet::atp_problem_id",
            "PortfolioEvidenceSet::backend_results",
            "PortfolioEvidenceSet::candidates",
            "PortfolioEvidenceSet::diagnostics",
            "PortfolioEvidenceSet::evidence_set_hash",
            "PortfolioEvidenceSet::plan_hash",
            "PortfolioEvidenceSet::portfolio_id",
            "PortfolioEvidenceSet::stop_summary",
            "PortfolioEvidenceSet::vc_hash",
            "PortfolioEvidenceSet::vc_id",
            "PortfolioId::as_str",
            "PortfolioId::new",
            "PortfolioInput::new",
            "PortfolioPlan::atp_problem_id",
            "PortfolioPlan::diagnostics",
            "PortfolioPlan::plan_hash",
            "PortfolioPlan::policy_constraints",
            "PortfolioPlan::portfolio_id",
            "PortfolioPlan::scheduled_runs",
            "PortfolioPlan::stop_summary",
            "PortfolioPlan::target_binding",
            "PortfolioPlan::vc_hash",
            "PortfolioPlan::vc_id",
            "PortfolioPolicyConstraints::new",
            "PortfolioPolicyConstraints::record_externally_attested",
            "PortfolioPolicyConstraints::with_externally_attested_records",
            "PortfolioStopSummary::message",
            "PortfolioStopSummary::new",
            "PortfolioStopSummary::reason",
            "collect_portfolio_results",
            "plan_portfolio",
        ],
        "{} public functions and methods must stay limited to task-18 \
         portfolio construction/accessors",
        source_path.display()
    );
}

#[test]
fn atp_property_encoding_module_has_paired_specs_and_excludes_trusted_material() {
    let en_spec = workspace_root().join("doc/design/mizar-atp/en/property_encoding.md");
    let ja_spec = workspace_root().join("doc/design/mizar-atp/ja/property_encoding.md");
    let source_path = crate_root().join("src/property_encoding.rs");
    let en = read_to_string(&en_spec);
    let ja = read_to_string(&ja_spec);
    let source = read_to_string(&source_path);

    for marker in [
        "Generated Binders",
        "Task 8 must not emit native declarations yet",
        "`connectedness` is permitted to use `AtpFormulaTree::Or`",
        "Task-8 Test Expectations",
    ] {
        assert!(
            en.contains(marker),
            "{} must keep task-7 property spec marker `{marker}`",
            en_spec.display()
        );
    }
    for marker in [
        "Generated binder",
        "Task 8 はまだ native declaration を emit してはならない",
        "`AtpFormulaTree::Or` を使ってよい",
        "Task 8 test expectations",
    ] {
        assert!(
            ja.contains(marker),
            "{} must keep task-7 Japanese property spec marker `{marker}`",
            ja_spec.display()
        );
    }
    for marker in [
        "pub fn encode_properties",
        "AtpPropertyEncodingStrategy::NativeDeclaration",
        "AtpPropertyEncodingError::NativeDeclarationDeferred",
        "AtpDeclarationKind::GeneratedBinder",
        "AtpSymbolSource::GeneratedBinder",
        "AtpSourceRef::EncodedProperty",
        "AtpFormulaTree::Or",
        "EncodedProperty::axiom",
    ] {
        assert!(
            source.contains(marker),
            "{} must implement property_encoding.md task-8 marker `{marker}`",
            source_path.display()
        );
    }
    for prohibited in [
        "std::process::Command",
        "mizar_kernel::",
        "resolution_trace",
        "MiniSAT",
        "DIMACS",
        "instantiated_formula",
        "backend_used_axioms",
        "kernel_verified",
    ] {
        assert!(
            !source.contains(prohibited),
            "{} must not expose prohibited trusted/backend material `{prohibited}`",
            source_path.display()
        );
    }
}

#[test]
fn atp_property_encoding_public_api_surface_is_task_eight_allowlist() {
    let source_path = crate_root().join("src/property_encoding.rs");
    let source = read_to_string(&source_path);
    let public_items = public_api_items(&source);
    let public_fields = public_struct_fields(&source);
    let public_functions = public_api_functions(&source);
    let expected = [
        "AtpPropertyBinderSort",
        "AtpPropertyEncodingBundle",
        "AtpPropertyEncodingError",
        "AtpPropertyEncodingInput",
        "AtpPropertyEncodingStrategy",
        "AtpPropertyFamily",
        "AtpPropertyProjection",
        "AtpPropertyTargetKind",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();

    assert_eq!(
        public_items,
        expected,
        "{} public API must stay limited to task-8 property projection, \
         axiom encoding, generated-binder bundle, and fail-closed error \
         shapes; concrete encoders, backend runners, proof policy, accepted \
         statuses, witnesses, and cache handles require explicit later specs",
        source_path.display()
    );

    assert_eq!(
        public_fields,
        [
            "AtpPropertyBinderSort::source",
            "AtpPropertyBinderSort::symbol",
            "AtpPropertyEncodingInput::existing_declarations",
            "AtpPropertyEncodingInput::existing_provenance",
            "AtpPropertyEncodingInput::existing_symbol_map",
            "AtpPropertyEncodingInput::logic_profile",
            "AtpPropertyEncodingInput::next_declaration_id",
            "AtpPropertyEncodingInput::next_property_id",
            "AtpPropertyEncodingInput::next_provenance_id",
            "AtpPropertyEncodingInput::property_projections",
            "AtpPropertyProjection::binder_sort",
            "AtpPropertyProjection::encoding_strategy",
            "AtpPropertyProjection::family",
            "AtpPropertyProjection::provenance_payload",
            "AtpPropertyProjection::source_property",
            "AtpPropertyProjection::target_arity",
            "AtpPropertyProjection::target_kind",
            "AtpPropertyProjection::target_source",
            "AtpPropertyProjection::target_symbol",
        ],
        "{} public struct fields must stay limited to structured property \
         inputs and id ranges; backend material, accepted statuses, witnesses, \
         and cache handles require explicit later specs",
        source_path.display()
    );

    assert_eq!(
        public_functions,
        [
            "AtpPropertyEncodingBundle::declarations",
            "AtpPropertyEncodingBundle::properties",
            "AtpPropertyEncodingBundle::provenance",
            "AtpPropertyEncodingBundle::symbol_map",
            "AtpPropertyFamily::as_str",
            "AtpPropertyFamily::is_empty",
            "AtpPropertyFamily::new",
            "encode_properties",
        ],
        "{} public functions and methods must stay limited to task-8 property \
         encoding construction/accessors",
        source_path.display()
    );
}

#[test]
fn atp_translator_module_has_paired_specs_and_excludes_trusted_material() {
    let en_spec = workspace_root().join("doc/design/mizar-atp/en/translator.md");
    let ja_spec = workspace_root().join("doc/design/mizar-atp/ja/translator.md");
    let source_path = crate_root().join("src/translator.rs");
    let en = read_to_string(&en_spec);
    let ja = read_to_string(&ja_spec);
    let source = read_to_string(&source_path);

    for marker in [
        "structured formula, declaration, and soft-type projection tables",
        "The source `VcIr` must be status `NeedsAtp`",
        "Duplicate declarations",
        "task 5 defines Rust projection input structs",
    ] {
        assert!(
            en.contains(marker),
            "{} must keep task-4 translator spec marker `{marker}`",
            en_spec.display()
        );
    }
    for marker in [
        "structured formula / declaration / soft-type projection table",
        "`NeedsAtp` status",
        "duplicate declaration",
        "task 5 は declaration / soft-type payload 用の Rust projection input",
    ] {
        assert!(
            ja.contains(marker),
            "{} must keep task-4 Japanese translator spec marker `{marker}`",
            ja_spec.display()
        );
    }
    for marker in [
        "pub fn translate_declarations",
        "pub fn translate_problem",
        "VcStatus::NeedsAtp",
        "targets_vc",
        "AtpDeclarationProjection",
        "AtpFormulaProjection",
        "AtpSymbolSourceProjection",
        "AtpSoftTypeProjection",
        "MissingSoftTypeGuard",
        "ExpectedBackendResult::Unsat",
        "AtpProblem::try_new",
    ] {
        assert!(
            source.contains(marker),
            "{} must implement translator.md task-5/task-6 marker `{marker}`",
            source_path.display()
        );
    }
    for prohibited in [
        "std::process::Command",
        "mizar_kernel::",
        "resolution_trace",
        "MiniSAT",
        "DIMACS",
        "instantiated_formula",
        "backend_used_axioms",
        "kernel_verified",
    ] {
        assert!(
            !source.contains(prohibited),
            "{} must not expose prohibited trusted/backend material `{prohibited}`",
            source_path.display()
        );
    }
}

#[test]
fn atp_translator_public_api_surface_is_task_six_allowlist() {
    let source_path = crate_root().join("src/translator.rs");
    let source = read_to_string(&source_path);
    let public_items = public_api_items(&source);
    let public_fields = public_struct_fields(&source);
    let public_functions = public_api_functions(&source);
    let expected = [
        "AtpDeclarationProjection",
        "AtpDeclarationTranslation",
        "AtpDeclarationTranslationInput",
        "AtpFormulaProjection",
        "AtpFormulaProjectionTarget",
        "AtpProjectionKey",
        "AtpProjectionProvenance",
        "AtpSoftTypeProjection",
        "AtpSoftTypeRepresentation",
        "AtpSymbolSourceProjection",
        "AtpTranslationError",
        "AtpTranslationInput",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();

    assert_eq!(
        public_items,
        expected,
        "{} public API must stay limited to task-5/task-6 translator \
         structured-projection and problem-construction shapes; backend \
         runners, proof methods, SAT \
         clauses, accepted statuses, witness types, and cache handles require \
         explicit later specs",
        source_path.display()
    );

    assert_eq!(
        public_fields,
        [
            "AtpDeclarationProjection::arity",
            "AtpDeclarationProjection::key",
            "AtpDeclarationProjection::kind",
            "AtpDeclarationProjection::provenance",
            "AtpDeclarationProjection::symbol",
            "AtpDeclarationProjection::symbol_source",
            "AtpDeclarationTranslationInput::declaration_projections",
            "AtpDeclarationTranslationInput::diagnostics",
            "AtpDeclarationTranslationInput::kernel_handoff",
            "AtpDeclarationTranslationInput::logic_profile",
            "AtpDeclarationTranslationInput::soft_type_projections",
            "AtpDeclarationTranslationInput::vc",
            "AtpDeclarationTranslationInput::vc_set",
            "AtpFormulaProjection::formula",
            "AtpFormulaProjection::handoff_formula_fingerprint",
            "AtpFormulaProjection::handoff_provenance_payload",
            "AtpFormulaProjection::provenance",
            "AtpFormulaProjection::source_identity",
            "AtpFormulaProjection::target",
            "AtpProjectionProvenance::payload",
            "AtpProjectionProvenance::source",
            "AtpSoftTypeProjection::key",
            "AtpSoftTypeProjection::provenance",
            "AtpSoftTypeProjection::representation",
            "AtpTranslationInput::declaration_projections",
            "AtpTranslationInput::diagnostics",
            "AtpTranslationInput::formula_projections",
            "AtpTranslationInput::kernel_handoff",
            "AtpTranslationInput::logic_profile",
            "AtpTranslationInput::soft_type_projections",
            "AtpTranslationInput::vc",
            "AtpTranslationInput::vc_set",
        ],
        "{} public struct fields must stay limited to structured translator \
         inputs and formula projection agreement material; backend material, \
         SAT material, accepted statuses, witnesses, and cache handles require \
         explicit later specs",
        source_path.display()
    );

    assert_eq!(
        public_functions,
        [
            "AtpDeclarationTranslation::declarations",
            "AtpDeclarationTranslation::diagnostics",
            "AtpDeclarationTranslation::logic_profile",
            "AtpDeclarationTranslation::provenance",
            "AtpDeclarationTranslation::symbol_map",
            "AtpDeclarationTranslation::target_binding",
            "AtpDeclarationTranslation::type_context",
            "AtpDeclarationTranslation::vc_id",
            "AtpFormulaProjectionTarget::imported",
            "AtpFormulaProjectionTarget::vc_formula",
            "AtpProjectionKey::as_str",
            "AtpProjectionKey::is_empty",
            "AtpProjectionKey::new",
            "AtpProjectionProvenance::new",
            "translate_declarations",
            "translate_problem",
        ],
        "{} public functions and methods must stay limited to task-5/task-6 \
         translator construction/accessors; backend runners, proof methods, \
         SAT/kernel checks, witnesses, and cache APIs require explicit later \
         specs",
        source_path.display()
    );
}

#[test]
fn atp_tptp_encoder_module_has_paired_specs_and_excludes_trusted_material() {
    let en_spec = workspace_root().join("doc/design/mizar-atp/en/tptp_encoder.md");
    let ja_spec = workspace_root().join("doc/design/mizar-atp/ja/tptp_encoder.md");
    let source_path = crate_root().join("src/tptp_encoder.rs");
    let en = read_to_string(&en_spec);
    let ja = read_to_string(&ja_spec);
    let source = read_to_string(&source_path);

    for marker in [
        "Task 10 supports this fail-closed subset",
        "The encoder emits entries in this deterministic order",
        "The encoder must track active quantifier scope",
        "raw-name",
        "Task-10 Test Expectations",
    ] {
        assert!(
            en.contains(marker),
            "{} must keep task-9 TPTP spec marker `{marker}`",
            en_spec.display()
        );
    }
    for marker in [
        "Task 10 は次の fail-closed subset",
        "encoder は次の deterministic order",
        "encoder は active quantifier scope",
        "raw-name",
        "Task 10 は focused Rust coverage",
    ] {
        assert!(
            ja.contains(marker),
            "{} must keep task-9 Japanese TPTP spec marker `{marker}`",
            ja_spec.display()
        );
    }
    for marker in [
        "pub fn encode_tptp",
        "TptpDialect::Fof",
        "ConcreteFormat::Tptp",
        "LogicFragment::Fof",
        "SoftTypeStrategy::GuardPredicates",
        "PropertyEncoding::Axiom",
        "NativePropertyDeclaration",
        "FreeVariable",
        "BinderShadowing",
        "DuplicateTptpName",
    ] {
        assert!(
            source.contains(marker),
            "{} must implement tptp_encoder.md task-10 marker `{marker}`",
            source_path.display()
        );
    }
    for prohibited in [
        "std::process::Command",
        "mizar_kernel::",
        "resolution_trace",
        "MiniSAT",
        "DIMACS",
        "instantiated_formula",
        "backend_used_axioms",
        "kernel_verified",
    ] {
        assert!(
            !source.contains(prohibited),
            "{} must not expose prohibited trusted/backend material `{prohibited}`",
            source_path.display()
        );
    }
}

#[test]
fn atp_tptp_encoder_public_api_surface_is_task_ten_allowlist() {
    let source_path = crate_root().join("src/tptp_encoder.rs");
    let source = read_to_string(&source_path);
    let public_items = public_api_items(&source);
    let public_fields = public_struct_fields(&source);
    let public_functions = public_api_functions(&source);
    let expected = [
        "TptpDialect",
        "TptpEncodingError",
        "TptpEncodingOutput",
        "TptpFormulaItem",
        "TptpFormulaLabel",
        "TptpSymbolBinding",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();

    assert_eq!(
        public_items,
        expected,
        "{} public API must stay limited to task-10 deterministic TPTP FOF \
         emission, side metadata, and fail-closed error shapes; backend \
         runners, proof methods, SAT clauses, accepted statuses, witness \
         types, and cache handles require explicit later specs",
        source_path.display()
    );

    assert!(
        public_fields.is_empty(),
        "{} must keep task-10 TPTP structs opaque; public fields would expand \
         the evidence handoff surface without a spec",
        source_path.display()
    );

    assert_eq!(
        public_functions,
        [
            "TptpEncodingOutput::formula_labels",
            "TptpEncodingOutput::symbol_bindings",
            "TptpEncodingOutput::text",
            "TptpFormulaLabel::item",
            "TptpFormulaLabel::label",
            "TptpFormulaLabel::provenance",
            "TptpFormulaLabel::target_symbol",
            "TptpSymbolBinding::atp_symbol",
            "TptpSymbolBinding::source",
            "TptpSymbolBinding::tptp_name",
            "encode_tptp",
        ],
        "{} public functions and methods must stay limited to deterministic \
         TPTP text and side-metadata accessors; backend runners, proof \
         methods, SAT/kernel checks, witnesses, and cache APIs require \
         explicit later specs",
        source_path.display()
    );
}

#[test]
fn atp_smtlib_encoder_module_has_paired_specs_and_excludes_trusted_material() {
    let en_spec = workspace_root().join("doc/design/mizar-atp/en/smtlib_encoder.md");
    let ja_spec = workspace_root().join("doc/design/mizar-atp/ja/smtlib_encoder.md");
    let source_path = crate_root().join("src/smtlib_encoder.rs");
    let en = read_to_string(&en_spec);
    let ja = read_to_string(&ja_spec);
    let source = read_to_string(&source_path);

    for marker in [
        "Task 12 supports this fail-closed subset",
        "The encoder emits commands in this deterministic order",
        "The encoder must track active quantifier scope",
        "unused `AtpDeclarationKind::Sort` rows are accepted",
        "Task-12 Test Expectations",
    ] {
        assert!(
            en.contains(marker),
            "{} must keep task-11 SMT-LIB spec marker `{marker}`",
            en_spec.display()
        );
    }
    for marker in [
        "Task 12 は次の fail-closed subset",
        "encoder は次の deterministic order",
        "encoder は active quantifier scope",
        "未使用の `AtpDeclarationKind::Sort` row",
        "Task 12 は次の focused Rust coverage",
    ] {
        assert!(
            ja.contains(marker),
            "{} must keep task-11 Japanese SMT-LIB spec marker `{marker}`",
            ja_spec.display()
        );
    }
    for marker in [
        "pub fn encode_smtlib",
        "SmtLibDialect::Uninterpreted",
        "ConcreteFormat::SmtLib",
        "LogicFragment::SmtLibUninterpreted",
        "SoftTypeStrategy::GuardPredicates",
        "PropertyEncoding::Axiom",
        "NativePropertyDeclaration",
        "FreeVariable",
        "BinderShadowing",
        "DuplicateSmtLibSymbol",
        "NegatedConjecture",
    ] {
        assert!(
            source.contains(marker),
            "{} must implement smtlib_encoder.md task-12 marker `{marker}`",
            source_path.display()
        );
    }
    for prohibited in [
        "std::process::Command",
        "mizar_kernel::",
        "resolution_trace",
        "MiniSAT",
        "DIMACS",
        "instantiated_formula",
        "backend_used_axioms",
        "kernel_verified",
        "get-proof",
        "get-unsat-core",
    ] {
        assert!(
            !source.contains(prohibited),
            "{} must not expose prohibited trusted/backend material `{prohibited}`",
            source_path.display()
        );
    }
}

#[test]
fn atp_smtlib_encoder_public_api_surface_is_task_twelve_allowlist() {
    let source_path = crate_root().join("src/smtlib_encoder.rs");
    let source = read_to_string(&source_path);
    let public_items = public_api_items(&source);
    let public_fields = public_struct_fields(&source);
    let public_functions = public_api_functions(&source);
    let expected = [
        "SmtLibAssertionItem",
        "SmtLibAssertionLabel",
        "SmtLibDialect",
        "SmtLibEncodingError",
        "SmtLibEncodingOutput",
        "SmtLibSymbolBinding",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();

    assert_eq!(
        public_items,
        expected,
        "{} public API must stay limited to task-12 deterministic \
         uninterpreted SMT-LIB emission, side metadata, and fail-closed error \
         shapes; backend runners, proof methods, SAT clauses, accepted \
         statuses, witness types, and cache handles require explicit later \
         specs",
        source_path.display()
    );

    assert!(
        public_fields.is_empty(),
        "{} must keep task-12 SMT-LIB structs opaque; public fields would \
         expand the evidence handoff surface without a spec",
        source_path.display()
    );

    assert_eq!(
        public_functions,
        [
            "SmtLibAssertionLabel::is_negated",
            "SmtLibAssertionLabel::item",
            "SmtLibAssertionLabel::label",
            "SmtLibAssertionLabel::provenance",
            "SmtLibAssertionLabel::target_symbol",
            "SmtLibEncodingOutput::assertion_labels",
            "SmtLibEncodingOutput::symbol_bindings",
            "SmtLibEncodingOutput::text",
            "SmtLibSymbolBinding::atp_symbol",
            "SmtLibSymbolBinding::smtlib_symbol",
            "SmtLibSymbolBinding::source",
            "encode_smtlib",
        ],
        "{} public functions and methods must stay limited to deterministic \
         SMT-LIB text and side-metadata accessors; backend runners, proof \
         methods, SAT/kernel checks, witnesses, and cache APIs require \
         explicit later specs",
        source_path.display()
    );
}

#[test]
fn atp_problem_module_has_paired_specs_and_excludes_trusted_backend_material() {
    let en_spec = workspace_root().join("doc/design/mizar-atp/en/problem.md");
    let ja_spec = workspace_root().join("doc/design/mizar-atp/ja/problem.md");
    let source_path = crate_root().join("src/problem.rs");
    let en = read_to_string(&en_spec);
    let ja = read_to_string(&ja_spec);
    let source = read_to_string(&source_path);

    for marker in [
        "backend-neutral ATP problem data model",
        "not kernel evidence",
        "not a SAT problem",
        "ExpectedBackendResult::Unsat",
        "not a provenance substitute",
    ] {
        assert!(
            en.contains(marker),
            "{} must keep task-2 problem spec marker `{marker}`",
            en_spec.display()
        );
    }
    for marker in [
        "backend-neutral ATP problem data model",
        "kernel evidence",
        "SAT problem",
        "ExpectedBackendResult::Unsat",
        "provenance の代替ではない",
    ] {
        assert!(
            ja.contains(marker),
            "{} must keep task-2 Japanese problem spec marker `{marker}`",
            ja_spec.display()
        );
    }
    for marker in [
        "pub struct AtpProblem",
        "ExpectedBackendResult::Unsat",
        "MissingSymbolMap",
        "MissingTypeContextBinding",
        "UnsupportedProfileFeature",
    ] {
        assert!(
            source.contains(marker),
            "{} must implement problem.md marker `{marker}`",
            source_path.display()
        );
    }
    for prohibited in [
        "std::process::Command",
        "mizar_kernel::",
        "resolution_trace",
        "MiniSAT",
        "DIMACS",
        "instantiated_formula",
        "backend_used_axioms",
        "kernel_verified",
    ] {
        assert!(
            !source.contains(prohibited),
            "{} must not expose prohibited trusted/backend material `{prohibited}`",
            source_path.display()
        );
    }
}

#[test]
fn atp_problem_public_api_surface_is_spec_backed_allowlist() {
    let source_path = crate_root().join("src/problem.rs");
    let source = read_to_string(&source_path);
    let public_items = public_api_items(&source);
    let public_fields = public_struct_fields(&source);
    let expected = [
        "AtpAtom",
        "AtpBinder",
        "AtpDeclaration",
        "AtpDeclarationId",
        "AtpDeclarationKind",
        "AtpDiagnostic",
        "AtpDiagnosticKey",
        "AtpDiagnosticMessage",
        "AtpFingerprint",
        "AtpFormula",
        "AtpFormulaId",
        "AtpFormulaTree",
        "AtpPayload",
        "AtpProblem",
        "AtpProblemError",
        "AtpProblemId",
        "AtpProblemParts",
        "AtpProfileName",
        "AtpPropertyId",
        "AtpProvenance",
        "AtpProvenanceId",
        "AtpRequiredProofStatus",
        "AtpSourceBinding",
        "AtpSourceRef",
        "AtpSymbolMapEntry",
        "AtpSymbolName",
        "AtpSymbolSource",
        "AtpTargetBinding",
        "AtpTerm",
        "AtpTypeContext",
        "AtpTypeGuard",
        "AtpTypeGuardId",
        "ConcreteFormat",
        "EncodedProperty",
        "EqualitySupport",
        "ExpectedBackendResult",
        "LogicFragment",
        "LogicProfile",
        "NativePropertySupport",
        "PropertyEncoding",
        "QuantifierPolicy",
        "SoftTypeStrategy",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();

    assert_eq!(
        public_items,
        expected,
        "{} public API must stay limited to problem.md data shapes; backend \
         logs, proof methods, SAT clauses, legacy certificates, accepted \
         statuses, used-axiom material, kernel checks, cache handles, and \
         witness types require explicit later specs",
        source_path.display()
    );

    assert_eq!(
        public_fields,
        [
            "AtpProblemParts::axioms",
            "AtpProblemParts::conjecture",
            "AtpProblemParts::declarations",
            "AtpProblemParts::diagnostics",
            "AtpProblemParts::expected_result",
            "AtpProblemParts::logic_profile",
            "AtpProblemParts::properties",
            "AtpProblemParts::provenance",
            "AtpProblemParts::symbol_map",
            "AtpProblemParts::target_binding",
            "AtpProblemParts::type_context",
            "AtpProblemParts::vc_id",
        ],
        "{} public struct fields must stay limited to problem.md construction \
         inputs; backend logs, proof methods, SAT clauses, legacy \
         certificates, accepted statuses, used-axiom material, kernel checks, \
         cache handles, and witness payloads require explicit later specs",
        source_path.display()
    );

    assert!(
        source.contains(
            "#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]\n\
             #[non_exhaustive]\n\
             pub enum ExpectedBackendResult {\n\
             \x20   Unsat,\n\
             }\n"
        ),
        "{} must keep `Unsat` as the only task-3 expected-result variant",
        source_path.display()
    );
}

#[test]
fn atp_public_enums_are_non_exhaustive_and_documented() {
    for (source_file, spec_file) in [
        ("problem.rs", "problem.md"),
        ("translator.rs", "translator.md"),
        ("property_encoding.rs", "property_encoding.md"),
        ("tptp_encoder.rs", "tptp_encoder.md"),
        ("smtlib_encoder.rs", "smtlib_encoder.md"),
        ("backend.rs", "backend.md"),
        ("portfolio.rs", "portfolio.md"),
    ] {
        let source_path = crate_root().join("src").join(source_file);
        let source = read_to_string(&source_path);
        let public_enums = public_enums(&source);

        assert!(
            !public_enums.is_empty(),
            "{} must expose the enum inventory guarded by task 22",
            source_path.display()
        );
        assert_public_enums_are_non_exhaustive(&source_path, &source);

        for spec_path in [
            workspace_root()
                .join("doc/design/mizar-atp/en")
                .join(spec_file),
            workspace_root()
                .join("doc/design/mizar-atp/ja")
                .join(spec_file),
        ] {
            let spec = read_to_string(&spec_path);
            assert!(
                spec.contains("Public enum inventory:"),
                "{} must record task-22 public enum inventory",
                spec_path.display()
            );
            assert_eq!(
                documented_enum_inventory(&spec),
                public_enums,
                "{} must keep its task-22 enum inventory synchronized with {}",
                spec_path.display(),
                source_path.display()
            );
        }
    }
}

#[test]
fn atp_source_spec_audit_covers_public_modules_and_deferred_gaps() {
    let modules = [
        "backend",
        "portfolio",
        "problem",
        "property_encoding",
        "smtlib_encoder",
        "tptp_encoder",
        "translator",
    ];
    let expected_modules = modules.iter().copied().collect::<BTreeSet<_>>();
    let expected_gaps = [
        (
            "ATP-AUDIT-G001".to_owned(),
            "external_dependency_gap".to_owned(),
        ),
        (
            "ATP-AUDIT-G002".to_owned(),
            "external_dependency_gap".to_owned(),
        ),
        ("ATP-AUDIT-G003".to_owned(), "deferred".to_owned()),
        ("ATP-AUDIT-G004".to_owned(), "deferred".to_owned()),
        (
            "ATP-AUDIT-G005".to_owned(),
            "external_dependency_gap".to_owned(),
        ),
    ]
    .into_iter()
    .collect::<BTreeMap<_, _>>();
    let mut gaps_by_language = BTreeMap::new();
    let mut violations = Vec::new();

    for language in ["en", "ja"] {
        let audit_path = workspace_root()
            .join("doc/design/mizar-atp")
            .join(language)
            .join("source_spec_audit.md");
        let audit = read_to_string(&audit_path);
        let audit_modules = audit_public_module_sections(&audit);
        if audit_modules != expected_modules {
            violations.push(format!(
                "{}: source/spec audit module sections must exactly match public modules; source={expected_modules:?}, audit={audit_modules:?}",
                audit_path.display()
            ));
        }

        let audit_gaps = audit_gap_classes(&audit);
        if audit_gaps != expected_gaps {
            violations.push(format!(
                "{}: source/spec audit gap ids/classes must exactly match expected follow-up register; expected={expected_gaps:?}, audit={audit_gaps:?}",
                audit_path.display()
            ));
        }
        gaps_by_language.insert(language, audit_gaps);

        for module in modules {
            let module_source_path = crate_root().join("src").join(format!("{module}.rs"));
            let source = read_to_string(&module_source_path);
            let module_section = audit_module_section(&audit, module).unwrap_or_else(|| {
                violations.push(format!(
                    "{}: source/spec audit must include a section for `{module}`",
                    audit_path.display()
                ));
                ""
            });

            for marker in [
                format!("`{module}`"),
                format!("crates/mizar-atp/src/{module}.rs"),
                format!("{module}.md"),
            ] {
                if !audit.contains(&marker) {
                    violations.push(format!(
                        "{}: source/spec audit must mention {marker}",
                        audit_path.display()
                    ));
                }
            }

            for public_item in public_api_items(&source) {
                let marker = format!("`{public_item}`");
                if !module_section.contains(&marker) {
                    violations.push(format!(
                        "{}: section `{module}` must mention public item {marker} from {}",
                        audit_path.display(),
                        module_source_path.display()
                    ));
                }
            }

            for public_function in public_top_level_functions(&source) {
                let marker = format!("`{public_function}`");
                if !module_section.contains(&marker) {
                    violations.push(format!(
                        "{}: section `{module}` must mention public entry function {marker} from {}",
                        audit_path.display(),
                        module_source_path.display()
                    ));
                }
            }
        }

        for gap_detail in audit_gap_detail_violations(&audit) {
            violations.push(format!("{}: {gap_detail}", audit_path.display()));
        }

        for marker in [
            "No source/spec drift",
            "source_undocumented_behavior",
            "repo_metadata_conflict",
            "real-output extraction",
            "advanced_semantics",
            "source-derived ATP extraction",
            "mizar-kernel",
            "mizar-artifact",
            "ProofWitnessRef",
            "VerifiedArtifact",
            "mizar-proof",
            "mizar-cache",
            "resolution trace",
            "backend proof method",
            "SMT proof object",
            "SAT problem payload",
        ] {
            if !audit.contains(marker) {
                violations.push(format!(
                    "{}: source/spec audit must record marker `{marker}`",
                    audit_path.display()
                ));
            }
        }
    }

    if gaps_by_language.get("en") != gaps_by_language.get("ja") {
        violations.push(format!(
            "EN/JA source/spec audit gap ids/classes must stay synchronized; en={:?}, ja={:?}",
            gaps_by_language.get("en"),
            gaps_by_language.get("ja")
        ));
    }

    assert!(
        violations.is_empty(),
        "mizar-atp source/spec audit drift:\n{}",
        violations.join("\n")
    );
}

#[test]
fn atp_bilingual_sync_audit_covers_design_doc_pairs() {
    let expected_docs = [
        "00.crate_plan.md",
        "backend.md",
        "bilingual_sync_audit.md",
        "crate_exit_report.md",
        "module_boundary_audit.md",
        "portfolio.md",
        "problem.md",
        "property_encoding.md",
        "smtlib_encoder.md",
        "source_spec_audit.md",
        "todo.md",
        "tptp_encoder.md",
        "translator.md",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<BTreeSet<_>>();
    let mut docs_by_language = BTreeMap::new();
    let mut violations = Vec::new();

    for language in ["en", "ja"] {
        let doc_dir = workspace_root().join("doc/design/mizar-atp").join(language);
        let docs = markdown_doc_names(&doc_dir);
        if docs != expected_docs {
            violations.push(format!(
                "{} must contain exactly the bilingual-sync-audited design docs; expected={expected_docs:?}, found={docs:?}",
                doc_dir.display()
            ));
        }
        docs_by_language.insert(language, docs);

        let audit_path = doc_dir.join("bilingual_sync_audit.md");
        let audit = read_to_string(&audit_path);
        let inventory_docs = bilingual_pair_inventory_docs(&audit);
        if inventory_docs.iter().cloned().collect::<BTreeSet<_>>() != expected_docs {
            violations.push(format!(
                "{} pair inventory must list exactly current design docs; expected={expected_docs:?}, found={:?}",
                audit_path.display(),
                inventory_docs.iter().cloned().collect::<BTreeSet<_>>()
            ));
        }
        let mut inventory_counts = BTreeMap::<String, usize>::new();
        for document in &inventory_docs {
            *inventory_counts.entry(document.clone()).or_insert(0) += 1;
        }
        for (document, count) in inventory_counts {
            if count != 1 {
                violations.push(format!(
                    "{} pair inventory must list `{document}` exactly once, found {count}",
                    audit_path.display()
                ));
            }
        }
        if inventory_docs.len() != expected_docs.len() {
            violations.push(format!(
                "{} pair inventory must have exactly {} rows, found {}",
                audit_path.display(),
                expected_docs.len(),
                inventory_docs.len()
            ));
        }
        for marker in [
            "bilingual drift",
            "repo_metadata_conflict",
            "external_dependency_gap",
            "deferred",
            "ProofWitnessRef",
            "VerifiedArtifact",
            "real-output extraction",
            "advanced-semantics",
            "source-derived ATP extraction",
            "resolution trace",
            "SMT proof object",
            "SAT problem payload",
            "mizar-proof",
            "mizar-cache",
        ] {
            if !audit.contains(marker) {
                violations.push(format!(
                    "{} must record bilingual sync marker `{marker}`",
                    audit_path.display()
                ));
            }
        }
    }

    if docs_by_language.get("en") != docs_by_language.get("ja") {
        violations.push(format!(
            "EN/JA mizar-atp design doc filenames must stay synchronized; en={:?}, ja={:?}",
            docs_by_language.get("en"),
            docs_by_language.get("ja")
        ));
    }

    assert!(
        violations.is_empty(),
        "mizar-atp bilingual sync audit drift:\n{}",
        violations.join("\n")
    );
}

#[test]
fn atp_task_twenty_five_policy_gap_is_documented_and_guarded() {
    let expectations = [
        (
            "en",
            vec![
                (
                    "todo.md",
                    vec![
                        "25. **Portfolio completion-order independence gate.** [x]",
                        "Status: deferred/external_dependency_gap completion",
                    ],
                ),
                (
                    "00.crate_plan.md",
                    vec![
                        "ATP-G-025",
                        "Task 25: Portfolio Completion-Order Independence Gate",
                        "complete as a deferred/external_dependency_gap gate",
                        "mock proof-policy oracle",
                    ],
                ),
                (
                    "portfolio.md",
                    vec![
                        "Task 25 re-evaluates the portfolio completion-order independence gate",
                        "Task 25 Re-evaluation",
                        "Task-25 Deferred Gate",
                    ],
                ),
                (
                    "source_spec_audit.md",
                    vec![
                        "ATP-AUDIT-G005",
                        "Task 25 re-evaluates the portfolio completion-order independence gate",
                        "task-18/task-21 cover only no-early-stop deterministic handoff",
                    ],
                ),
                (
                    "bilingual_sync_audit.md",
                    vec![
                        "Task 25 Sync Edits",
                        "portfolio completion-order independence gate",
                        "winner/early-stop gate",
                    ],
                ),
            ],
        ),
        (
            "ja",
            vec![
                (
                    "todo.md",
                    vec![
                        "25. **portfolio 完了順独立性 gate。** [x]",
                        "Status: deferred/external_dependency_gap completion",
                    ],
                ),
                (
                    "00.crate_plan.md",
                    vec![
                        "ATP-G-025",
                        "Task 25: portfolio completion-order independence gate",
                        "deferred/external_dependency_gap gate として完了",
                        "mock proof-policy oracle",
                    ],
                ),
                (
                    "portfolio.md",
                    vec![
                        "task 25 は portfolio completion-order independence gate を再評価する",
                        "Task 25 Re-evaluation",
                        "Task-25 Deferred Gate",
                    ],
                ),
                (
                    "source_spec_audit.md",
                    vec![
                        "ATP-AUDIT-G005",
                        "task 25 は portfolio completion-order independence gate を再評価する",
                        "no-early-stop deterministic handoff だけを cover",
                    ],
                ),
                (
                    "bilingual_sync_audit.md",
                    vec![
                        "Task 25 Sync Edits",
                        "portfolio completion-order independence gate",
                        "winner/early-stop gate",
                    ],
                ),
            ],
        ),
    ];

    let mut violations = Vec::new();
    for (language, files) in expectations {
        let doc_dir = workspace_root().join("doc/design/mizar-atp").join(language);
        for (file_name, markers) in files {
            let path = doc_dir.join(file_name);
            let document = read_to_string(&path);
            for marker in markers {
                if !document.contains(marker) {
                    violations.push(format!(
                        "{} must document task-25 marker `{marker}`",
                        path.display()
                    ));
                }
            }
        }
    }

    let source_path = crate_root().join("src/portfolio.rs");
    let source = read_to_string(&source_path);
    for prohibited in [
        "ProofPolicyAdapter",
        "ProofPolicyDecision",
        "ProofPolicyEvaluator",
        "ProofPolicyFinality",
        "ProofPolicyOracle",
        "ProofPolicySelector",
        "ProofPolicyWinner",
        "EarlyStopOracle",
        "PolicyOracle",
        "WinnerSelection",
        "candidate_displacement",
        "early_stop_finality",
        "early_stop_oracle",
        "kernel_check",
        "mizar_proof",
        "mock_proof_policy",
        "policy_adapter",
        "policy_oracle",
        "policy_selector",
        "proof_policy",
        "proof_policy_select",
        "selected_class",
        "winner_selection",
        "accepted_state",
        "accepted_status",
        "accepted_proof_state",
        "ProofWitness",
        "publish_witness",
        "witness_output",
        "witness_writer",
        "proof_cache",
        "cache_promotion",
        "cache_output",
        "KernelCheckResult",
        "trusted_backend_proof",
        "backend_proof_material",
        "trusted_proof_material",
    ] {
        if source.contains(prohibited) {
            violations.push(format!(
                "{} must not add task-25 placeholder policy material `{prohibited}`",
                source_path.display()
            ));
        }
    }

    assert!(
        violations.is_empty(),
        "mizar-atp task-25 policy gap drift:\n{}",
        violations.join("\n")
    );
}

#[test]
fn atp_task_twenty_six_architecture_follow_up_audit_is_documented() {
    let expectations = [
        (
            "en",
            vec![
                (
                    "todo.md",
                    vec![
                        "26. **Architecture-22 follow-up audit.** [x]",
                        "Status: complete as an audit-only follow-up",
                        "`ATP-AUDIT-G005` remains the single policy-boundary",
                    ],
                ),
                (
                    "00.crate_plan.md",
                    vec![
                        "ATP-G-026",
                        "Task 26: Architecture-22 Follow-Up Audit",
                        "No new follow-up id is required",
                    ],
                ),
                (
                    "source_spec_audit.md",
                    vec![
                        "Task 26 Architecture-22 Follow-Up Audit",
                        "\"First backend to finish\" must not become the",
                        "semantic winner rule",
                        "exactly ATP-AUDIT-G005",
                    ],
                ),
                (
                    "bilingual_sync_audit.md",
                    vec![
                        "Task 26 Sync Edits",
                        "semantic proof identity",
                        "ATP-AUDIT-G005 remains the single policy-boundary",
                    ],
                ),
            ],
        ),
        (
            "ja",
            vec![
                (
                    "todo.md",
                    vec![
                        "26. **architecture-22 フォローアップ監査。** [x]",
                        "Status: audit-only follow-up として完了",
                        "`ATP-AUDIT-G005` は",
                    ],
                ),
                (
                    "00.crate_plan.md",
                    vec![
                        "ATP-G-026",
                        "Task 26: architecture-22 follow-up audit",
                        "新しい follow-up id は不要",
                    ],
                ),
                (
                    "source_spec_audit.md",
                    vec![
                        "Task 26 Architecture-22 Follow-Up Audit",
                        "\"first backend to finish\" は semantic winner rule",
                        "ATP-AUDIT-G005 だけ",
                    ],
                ),
                (
                    "bilingual_sync_audit.md",
                    vec![
                        "Task 26 Sync Edits",
                        "semantic proof identity",
                        "ATP-AUDIT-G005 は単一の policy-boundary",
                    ],
                ),
            ],
        ),
    ];

    let mut violations = Vec::new();
    for (language, files) in expectations {
        let doc_dir = workspace_root().join("doc/design/mizar-atp").join(language);
        for (file_name, markers) in files {
            let path = doc_dir.join(file_name);
            let document = read_to_string(&path);
            for marker in markers {
                if !document.contains(marker) {
                    violations.push(format!(
                        "{} must document task-26 marker `{marker}`",
                        path.display()
                    ));
                }
            }
        }
    }

    for language in ["en", "ja"] {
        let audit_path = workspace_root()
            .join("doc/design/mizar-atp")
            .join(language)
            .join("source_spec_audit.md");
        let audit = read_to_string(&audit_path);
        let audit_gaps = audit_gap_classes(&audit);
        if audit_gaps.keys().cloned().collect::<Vec<_>>()
            != [
                "ATP-AUDIT-G001".to_owned(),
                "ATP-AUDIT-G002".to_owned(),
                "ATP-AUDIT-G003".to_owned(),
                "ATP-AUDIT-G004".to_owned(),
                "ATP-AUDIT-G005".to_owned(),
            ]
        {
            violations.push(format!(
                "{} must keep task-26 follow-up audit on the existing ATP-AUDIT gap set; found {audit_gaps:?}",
                audit_path.display()
            ));
        }
    }

    assert!(
        violations.is_empty(),
        "mizar-atp task-26 architecture follow-up audit drift:\n{}",
        violations.join("\n")
    );
}

#[test]
fn atp_task_twenty_seven_module_boundary_layout_is_documented() {
    let expectations = [
        (
            "en",
            vec![
                (
                    "todo.md",
                    vec![
                        "27. **Module-boundary refactor gate.** [x]",
                        "Status: complete as a behavior-preserving layout refactor",
                        "moved the existing unit suites into private",
                        "`cfg(test)` child modules",
                    ],
                ),
                (
                    "00.crate_plan.md",
                    vec![
                        "ATP-G-027",
                        "Task 27: Module-Boundary Refactor Gate",
                        "private test module split",
                        "No public API, production behavior, diagnostics, deterministic rendering, artifact schema, or trust-boundary behavior changes",
                    ],
                ),
                (
                    "module_boundary_audit.md",
                    vec![
                        "Task 27 Module-Boundary Refactor Gate",
                        "private test module split",
                        "src/backend/tests.rs",
                        "src/portfolio/tests.rs",
                        "src/problem/tests.rs",
                        "src/property_encoding/tests.rs",
                        "src/smtlib_encoder/tests.rs",
                        "src/tptp_encoder/tests.rs",
                        "src/translator/tests.rs",
                        "No public API change",
                        "No new ATP-AUDIT gap",
                    ],
                ),
                (
                    "source_spec_audit.md",
                    vec![
                        "Task 27 Module-Boundary Layout Audit",
                        "src/backend/tests.rs",
                        "src/portfolio/tests.rs",
                        "src/problem/tests.rs",
                        "src/property_encoding/tests.rs",
                        "src/smtlib_encoder/tests.rs",
                        "src/tptp_encoder/tests.rs",
                        "src/translator/tests.rs",
                        "ATP-AUDIT gap is required",
                    ],
                ),
                (
                    "bilingual_sync_audit.md",
                    vec![
                        "Task 27 Sync Edits",
                        "module_boundary_audit.md",
                        "private test module split",
                    ],
                ),
            ],
        ),
        (
            "ja",
            vec![
                (
                    "todo.md",
                    vec![
                        "27. **module 境界リファクタリング gate。** [x]",
                        "Status: behavior-preserving layout refactor として完了",
                        "private `cfg(test)` child module",
                    ],
                ),
                (
                    "00.crate_plan.md",
                    vec![
                        "ATP-G-027",
                        "Task 27: module-boundary refactor gate",
                        "private test module split",
                        "public API、production behavior、diagnostic、deterministic rendering、artifact schema、trust-boundary behavior は変更しない",
                    ],
                ),
                (
                    "module_boundary_audit.md",
                    vec![
                        "Task 27 Module-Boundary Refactor Gate",
                        "private test module split",
                        "src/backend/tests.rs",
                        "src/portfolio/tests.rs",
                        "src/problem/tests.rs",
                        "src/property_encoding/tests.rs",
                        "src/smtlib_encoder/tests.rs",
                        "src/tptp_encoder/tests.rs",
                        "src/translator/tests.rs",
                        "public API change はない",
                        "No new ATP-AUDIT gap",
                    ],
                ),
                (
                    "source_spec_audit.md",
                    vec![
                        "Task 27 Module-Boundary Layout Audit",
                        "src/backend/tests.rs",
                        "src/portfolio/tests.rs",
                        "src/problem/tests.rs",
                        "src/property_encoding/tests.rs",
                        "src/smtlib_encoder/tests.rs",
                        "src/tptp_encoder/tests.rs",
                        "src/translator/tests.rs",
                        "ATP-AUDIT gap is required",
                    ],
                ),
                (
                    "bilingual_sync_audit.md",
                    vec![
                        "Task 27 Sync Edits",
                        "module_boundary_audit.md",
                        "private test module split",
                    ],
                ),
            ],
        ),
    ];

    let mut violations = Vec::new();
    for (language, files) in expectations {
        let doc_dir = workspace_root().join("doc/design/mizar-atp").join(language);
        for (file_name, markers) in files {
            let path = doc_dir.join(file_name);
            let document = read_to_string(&path);
            for marker in markers {
                if !document.contains(marker) {
                    violations.push(format!(
                        "{} must document task-27 marker `{marker}`",
                        path.display()
                    ));
                }
            }
        }
    }

    for module in [
        "backend",
        "portfolio",
        "problem",
        "property_encoding",
        "smtlib_encoder",
        "tptp_encoder",
        "translator",
    ] {
        let source_path = crate_root().join("src").join(format!("{module}.rs"));
        let source = read_to_string(&source_path);
        let child_module_marker = if module == "backend" {
            "#[cfg(all(test, unix))]\nmod tests;"
        } else {
            "#[cfg(test)]\nmod tests;"
        };
        if !source.contains(child_module_marker) {
            violations.push(format!(
                "{} must keep the task-27 private test child module marker",
                source_path.display()
            ));
        }

        let test_path = crate_root().join("src").join(module).join("tests.rs");
        let tests = read_to_string(&test_path);
        for prohibited in [
            "pub struct ",
            "pub enum ",
            "pub fn ",
            "pub const fn ",
            "pub mod ",
        ] {
            if tests
                .lines()
                .map(str::trim_start)
                .any(|line| line.starts_with(prohibited))
            {
                violations.push(format!(
                    "{} must remain a private unit-test child and not expose `{prohibited}`",
                    test_path.display()
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "mizar-atp task-27 module-boundary layout drift:\n{}",
        violations.join("\n")
    );
}

#[test]
fn atp_task_twenty_eight_crate_exit_report_is_documented() {
    let expectations = [
        (
            "en",
            vec![
                (
                    "todo.md",
                    vec![
                        "28. **Crate exit report and quality review.** [x]",
                        "94/100",
                        "external_dependency_gap / deferred",
                    ],
                ),
                (
                    "00.crate_plan.md",
                    vec![
                        "ATP-G-028",
                        "Task 28: Crate Exit Report And Quality Review",
                        "quality score 94/100",
                    ],
                ),
                (
                    "bilingual_sync_audit.md",
                    vec![
                        "Task 28 Sync Edits",
                        "crate_exit_report.md",
                        "quality score 94/100",
                    ],
                ),
                (
                    "crate_exit_report.md",
                    vec![
                        "Status: complete for the current candidate-evidence producer milestone.",
                        "Quality score: 94/100.",
                        "ATP-CLOSEOUT-G001",
                        "ATP-CLOSEOUT-G002",
                        "ATP-CLOSEOUT-G003",
                        "`cargo test -p mizar-proof` and `cargo test -p mizar-cache` were not run",
                        "Placeholder `mizar-proof`",
                    ],
                ),
            ],
        ),
        (
            "ja",
            vec![
                (
                    "todo.md",
                    vec![
                        "28. **crate exit report と quality review。** [x]",
                        "94/100",
                        "external_dependency_gap / deferred",
                    ],
                ),
                (
                    "00.crate_plan.md",
                    vec![
                        "ATP-G-028",
                        "Task 28: crate exit report and quality review",
                        "quality score 94/100",
                    ],
                ),
                (
                    "bilingual_sync_audit.md",
                    vec![
                        "Task 28 Sync Edits",
                        "crate_exit_report.md",
                        "quality score 94/100",
                    ],
                ),
                (
                    "crate_exit_report.md",
                    vec![
                        "Status: 現在の candidate-evidence producer milestone として完了。",
                        "Quality score: 94/100",
                        "ATP-CLOSEOUT-G001",
                        "ATP-CLOSEOUT-G002",
                        "ATP-CLOSEOUT-G003",
                        "`cargo test -p mizar-proof` と `cargo test -p mizar-cache`",
                        "placeholder `mizar-proof` または `mizar-cache` crate",
                    ],
                ),
            ],
        ),
    ];

    let mut violations = Vec::new();
    for (language, files) in expectations {
        let doc_dir = workspace_root().join("doc/design/mizar-atp").join(language);
        for (file_name, markers) in files {
            let path = doc_dir.join(file_name);
            let document = read_to_string(&path);
            for marker in markers {
                if !document.contains(marker) {
                    violations.push(format!(
                        "{} must document task-28 closeout marker `{marker}`",
                        path.display()
                    ));
                }
            }
        }
    }

    for language in ["en", "ja"] {
        let report_path = workspace_root()
            .join("doc/design/mizar-atp")
            .join(language)
            .join("crate_exit_report.md");
        let report = read_to_string(&report_path);
        let closeout_gaps = closeout_gap_rows(&report)
            .into_iter()
            .map(|(id, cells)| {
                let class = cells
                    .get(1)
                    .map(|class| class.replace('`', ""))
                    .unwrap_or_default();
                (id, (class, cells))
            })
            .collect::<BTreeMap<_, _>>();
        let expected_classes = [
            ("ATP-CLOSEOUT-G001", "external_dependency_gap"),
            ("ATP-CLOSEOUT-G002", "external_dependency_gap"),
            ("ATP-CLOSEOUT-G003", "external_dependency_gap"),
            ("ATP-CLOSEOUT-G004", "external_dependency_gap / deferred"),
            ("ATP-CLOSEOUT-G005", "deferred"),
            ("ATP-CLOSEOUT-G006", "deferred"),
        ]
        .into_iter()
        .map(|(id, class)| (id.to_owned(), class.to_owned()))
        .collect::<BTreeMap<_, _>>();
        let actual_classes = closeout_gaps
            .iter()
            .map(|(id, (class, _))| (id.clone(), class.clone()))
            .collect::<BTreeMap<_, _>>();
        if actual_classes != expected_classes {
            violations.push(format!(
                "{} must keep the task-28 closeout gap classes synchronized; expected {expected_classes:?}, found {actual_classes:?}",
                report_path.display()
            ));
        }

        let required_gap_markers = if language == "en" {
            [
                (
                    "ATP-CLOSEOUT-G001",
                    [
                        "real-output extraction",
                        "backend-specific EN/JA extraction specs",
                    ],
                ),
                (
                    "ATP-CLOSEOUT-G002",
                    ["`mizar-proof` is design-only", "do not add proof policy"],
                ),
                (
                    "ATP-CLOSEOUT-G003",
                    [
                        "`mizar-cache` is design-only",
                        "cache reuse must never upgrade evidence",
                    ],
                ),
                (
                    "ATP-CLOSEOUT-G004",
                    [
                        "`mizar-artifact` owns witness schema/projection",
                        "Downstream proof/cache/artifact owners",
                    ],
                ),
                (
                    "ATP-CLOSEOUT-G005",
                    ["metadata-only corpus anchors", "staged runner"],
                ),
                (
                    "ATP-CLOSEOUT-G006",
                    ["Typed TPTP/CNF/include routes", "paired specs and tests"],
                ),
            ]
        } else {
            [
                (
                    "ATP-CLOSEOUT-G001",
                    [
                        "real-output extraction spec/source module",
                        "backend-specific EN/JA extraction spec",
                    ],
                ),
                (
                    "ATP-CLOSEOUT-G002",
                    [
                        "`mizar-proof` は design-only",
                        "proof policy を `mizar-atp` に追加しない",
                    ],
                ),
                (
                    "ATP-CLOSEOUT-G003",
                    [
                        "`mizar-cache` は design-only",
                        "cache reuse は evidence を upgrade してはならない",
                    ],
                ),
                (
                    "ATP-CLOSEOUT-G004",
                    [
                        "`mizar-artifact` は witness schema/projection を所有",
                        "downstream proof/cache/artifact owner",
                    ],
                ),
                (
                    "ATP-CLOSEOUT-G005",
                    ["metadata-only corpus anchor", "staged runner"],
                ),
                (
                    "ATP-CLOSEOUT-G006",
                    ["Typed TPTP/CNF/include route", "paired spec と test"],
                ),
            ]
        };
        for (gap_id, markers) in required_gap_markers {
            let Some((_, cells)) = closeout_gaps.get(gap_id) else {
                violations.push(format!(
                    "{} must include closeout gap row `{gap_id}`",
                    report_path.display()
                ));
                continue;
            };
            let row = cells.join(" | ");
            for marker in markers {
                if !row.contains(marker) {
                    violations.push(format!(
                        "{} closeout gap row `{gap_id}` must include marker `{marker}`",
                        report_path.display()
                    ));
                }
            }
        }

        for marker in [
            "cargo clippy --all-targets --all-features --offline -- -D warnings",
            "cargo test --offline",
            "git diff --cached --check",
        ] {
            if !report.contains(marker) {
                violations.push(format!(
                    "{} must keep closeout verification marker `{marker}`",
                    report_path.display()
                ));
            }
        }
    }

    let workspace_manifest = read_to_string(&workspace_root().join("Cargo.toml"));
    let members = workspace_members(&workspace_manifest);
    let atp_manifest = read_to_string(&crate_root().join("Cargo.toml"));
    for crate_name in ["mizar-proof", "mizar-cache"] {
        let member = format!("crates/{crate_name}");
        if members
            .iter()
            .any(|workspace_member| workspace_member == &member)
        {
            violations.push(format!(
                "{} must not add placeholder workspace member `{member}` in task 28",
                workspace_root().join("Cargo.toml").display()
            ));
        }
        if workspace_root().join("crates").join(crate_name).exists() {
            violations.push(format!(
                "task 28 must not add placeholder crate directory `{member}`"
            ));
        }
        if atp_manifest.contains(crate_name) {
            violations.push(format!(
                "{} must not depend on placeholder `{crate_name}` in task 28",
                crate_root().join("Cargo.toml").display()
            ));
        }
    }

    assert!(
        violations.is_empty(),
        "mizar-atp task-28 crate exit report drift:\n{}",
        violations.join("\n")
    );
}

fn closeout_gap_rows(document: &str) -> Vec<(String, Vec<String>)> {
    document
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if !trimmed.starts_with("| ATP-CLOSEOUT-") {
                return None;
            }
            let cells = trimmed
                .trim_matches('|')
                .split('|')
                .map(str::trim)
                .map(str::to_owned)
                .collect::<Vec<_>>();
            let id = cells.first()?.clone();
            Some((id, cells))
        })
        .collect()
}

#[test]
fn workspace_lint_baseline_denies_rustc_warnings_and_clippy_all() {
    let manifest_path = workspace_root().join("Cargo.toml");
    let manifest = read_to_string(&manifest_path);
    let rust_lints = section(&manifest, "workspace.lints.rust");
    let clippy_lints = section(&manifest, "workspace.lints.clippy");

    assert!(
        rust_lints
            .iter()
            .any(|line| assignment_is(line, "warnings", "deny")),
        "{} must deny rustc warnings in the shared lint baseline",
        manifest_path.display()
    );
    assert!(
        clippy_lints
            .iter()
            .any(|line| assignment_is(line, "all", "deny")),
        "{} must deny clippy::all in the shared lint baseline",
        manifest_path.display()
    );
}

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn workspace_root() -> PathBuf {
    crate_root()
        .parent()
        .and_then(|path| path.parent())
        .expect("crate lives under crates/<name>")
        .to_path_buf()
}

fn read_to_string(path: &std::path::Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn section<'a>(manifest: &'a str, section: &str) -> Vec<&'a str> {
    let header = format!("[{section}]");
    let mut lines = Vec::new();
    let mut active = false;
    for line in manifest.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            active = trimmed == header;
            continue;
        }
        if active && !trimmed.is_empty() && !trimmed.starts_with('#') {
            lines.push(trimmed);
        }
    }
    lines
}

fn workspace_members(manifest: &str) -> Vec<String> {
    let workspace = section(manifest, "workspace");
    let mut members = Vec::new();
    let mut in_members = false;

    for line in workspace {
        if !in_members {
            if let Some((lhs, rhs)) = line.split_once('=')
                && lhs.trim() == "members"
            {
                in_members = true;
                push_member_entries(rhs, &mut members);
                if rhs.contains(']') {
                    break;
                }
            }
            continue;
        }

        push_member_entries(line, &mut members);
        if line.contains(']') {
            break;
        }
    }

    members
}

fn push_member_entries(line: &str, members: &mut Vec<String>) {
    for entry in line.split(',') {
        let member = entry
            .trim()
            .trim_start_matches('[')
            .trim_end_matches(']')
            .trim()
            .trim_matches('"');
        if !member.is_empty() {
            members.push(member.to_owned());
        }
    }
}

fn dependency_sections(manifest: &str) -> Vec<(String, Vec<&str>)> {
    let mut sections = Vec::new();
    let mut active = None;

    for line in manifest.lines() {
        let trimmed = line.trim();
        if let Some(section_name) = section_name(trimmed) {
            if let Some(section) = active.take() {
                sections.push(section);
            }
            active =
                dependency_section(section_name).then(|| (section_name.to_owned(), Vec::new()));
            continue;
        }

        if let Some((_, lines)) = &mut active
            && !trimmed.is_empty()
            && !trimmed.starts_with('#')
        {
            lines.push(trimmed);
        }
    }

    if let Some(section) = active {
        sections.push(section);
    }

    sections
}

fn section_name(line: &str) -> Option<&str> {
    line.strip_prefix('[')?.strip_suffix(']')
}

fn dependency_section(section_name: &str) -> bool {
    matches!(
        section_name,
        "dependencies" | "dev-dependencies" | "build-dependencies"
    ) || section_name.starts_with("dependencies.")
        || section_name.starts_with("dev-dependencies.")
        || section_name.starts_with("build-dependencies.")
        || section_name.ends_with(".dependencies")
        || section_name.ends_with(".dev-dependencies")
        || section_name.ends_with(".build-dependencies")
        || section_name.contains(".dependencies.")
        || section_name.contains(".dev-dependencies.")
        || section_name.contains(".build-dependencies.")
}

fn assignment_is(line: &str, key: &str, value: &str) -> bool {
    let Some((lhs, rhs)) = line.split_once('=') else {
        return false;
    };
    lhs.trim() == key && rhs.trim().trim_matches('"') == value
}

fn public_api_items(source: &str) -> Vec<String> {
    let mut items = Vec::new();
    for line in source.lines() {
        let trimmed = line.trim();
        if let Some(name) = item_name(trimmed.strip_prefix("pub struct ")) {
            items.push(name.to_owned());
        } else if let Some(name) = item_name(trimmed.strip_prefix("pub enum ")) {
            items.push(name.to_owned());
        } else if let Some(name) = macro_item_name(trimmed, "dense_id!(") {
            items.push(name.to_owned());
        } else if let Some(name) = macro_item_name(trimmed, "string_key!(") {
            items.push(name.to_owned());
        }
    }
    items.sort();
    items
}

fn public_enums(source: &str) -> Vec<String> {
    source
        .lines()
        .filter_map(|line| item_name(line.trim().strip_prefix("pub enum ")))
        .map(str::to_owned)
        .collect()
}

fn assert_public_enums_are_non_exhaustive(source_path: &std::path::Path, source: &str) {
    let mut attributes = Vec::new();

    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("#[") {
            attributes.push(trimmed.to_owned());
            continue;
        }

        if let Some(name) = item_name(trimmed.strip_prefix("pub enum ")) {
            assert!(
                attributes
                    .iter()
                    .any(|attribute| attribute == "#[non_exhaustive]"),
                "{} public enum `{name}` must be #[non_exhaustive] for task 22",
                source_path.display()
            );
        }

        attributes.clear();
    }
}

fn documented_enum_inventory(spec: &str) -> Vec<String> {
    let line = spec
        .lines()
        .find(|line| line.trim_start().starts_with("Public enum inventory:"))
        .expect("task-22 public enum inventory marker");
    backtick_items(line)
}

fn backtick_items(line: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut rest = line;

    while let Some(start) = rest.find('`') {
        let after_start = &rest[start + 1..];
        let Some(end) = after_start.find('`') else {
            break;
        };
        items.push(after_start[..end].to_owned());
        rest = &after_start[end + 1..];
    }

    items
}

fn audit_public_module_sections(document: &str) -> BTreeSet<&str> {
    document
        .lines()
        .filter_map(|line| {
            let module = line.trim().strip_prefix("### `")?.split('`').next()?;
            (!module.is_empty()).then_some(module)
        })
        .collect()
}

fn audit_module_section<'a>(document: &'a str, module: &str) -> Option<&'a str> {
    let header = format!("### `{module}`");
    let start = document.find(&header)?;
    let rest = &document[start..];
    let end = rest
        .find("\n### `")
        .or_else(|| rest.find("\n## "))
        .map_or(document.len(), |offset| start + offset);
    Some(&document[start..end])
}

fn audit_gap_classes(document: &str) -> BTreeMap<String, String> {
    let mut classes = BTreeMap::new();
    for (id, cells) in audit_gap_rows(document) {
        if let Some(class) = cells.get(1) {
            classes.insert(id, class.trim_matches('`').to_owned());
        }
    }
    classes
}

fn audit_gap_detail_violations(document: &str) -> Vec<String> {
    let rows = audit_gap_rows(document);
    let mut counts = BTreeMap::<String, usize>::new();
    for (id, _) in &rows {
        *counts.entry(id.clone()).or_insert(0) += 1;
    }

    let mut violations = counts
        .into_iter()
        .filter_map(|(id, count)| {
            (count != 1)
                .then(|| format!("follow-up row {id} must appear exactly once, found {count}"))
        })
        .collect::<Vec<_>>();

    violations.extend(
        rows.into_iter()
        .flat_map(|(id, cells)| {
            let mut violations = Vec::new();
            if cells.len() != 6 {
                violations.push(format!(
                    "follow-up row {id} must have exactly ID, Class, Evidence, Owner, Unblock condition, and Target follow-up / downstream phase cells"
                ));
                return violations;
            }
            for (index, label) in [
                "Evidence",
                "Owner",
                "Unblock condition",
                "Target follow-up / downstream phase",
            ]
            .iter()
            .enumerate()
            {
                let cell = cells[index + 2].trim();
                if cell.is_empty() || cell == "-" {
                    violations.push(format!(
                        "follow-up row {id} must have a non-empty {label} cell"
                    ));
                }
            }
            violations
        }),
    );

    violations
}

fn audit_gap_rows(document: &str) -> Vec<(String, Vec<String>)> {
    document
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if !trimmed.starts_with("| ATP-AUDIT-") {
                return None;
            }
            let cells = trimmed
                .trim_matches('|')
                .split('|')
                .map(str::trim)
                .collect::<Vec<_>>();
            let id = cells.first()?;
            Some((
                (*id).to_owned(),
                cells.into_iter().map(str::to_owned).collect(),
            ))
        })
        .collect()
}

fn markdown_doc_names(root: &std::path::Path) -> BTreeSet<String> {
    fs::read_dir(root)
        .unwrap_or_else(|error| panic!("{}: {error}", root.display()))
        .map(|entry| entry.expect("directory entry").path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "md"))
        .map(|path| {
            path.file_name()
                .expect("markdown file has file name")
                .to_string_lossy()
                .into_owned()
        })
        .collect()
}

fn bilingual_pair_inventory_docs(document: &str) -> Vec<String> {
    let section = audit_section(document, "Pair Inventory")
        .expect("bilingual sync audit must have Pair Inventory section");
    let mut docs = section
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if !trimmed.starts_with("| `") {
                return None;
            }
            let document = trimmed.trim_matches('|').split('|').next()?.trim();
            document
                .strip_prefix('`')?
                .strip_suffix('`')
                .map(str::to_owned)
        })
        .collect::<Vec<_>>();
    docs.sort();
    docs
}

fn audit_section<'a>(document: &'a str, title: &str) -> Option<&'a str> {
    let header = format!("## {title}");
    let start = document.find(&header)?;
    let rest = &document[start + header.len()..];
    let end = rest.find("\n## ").unwrap_or(rest.len());
    Some(&rest[..end])
}

fn public_struct_fields(source: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current_struct = None;

    for line in source.lines() {
        let trimmed = line.trim();
        if let Some(name) = item_name(trimmed.strip_prefix("pub struct ")) {
            if trimmed.ends_with('{') {
                current_struct = Some(name.to_owned());
            }
            continue;
        }

        if trimmed == "}" {
            current_struct = None;
            continue;
        }

        if let Some(struct_name) = &current_struct
            && let Some(field) = item_name(trimmed.strip_prefix("pub "))
        {
            fields.push(format!("{struct_name}::{field}"));
        }
    }

    fields.sort();
    fields
}

fn public_api_functions(source: &str) -> Vec<String> {
    let mut functions = Vec::new();
    let mut current_impl = None;
    let mut depth = 0usize;

    for line in source.lines() {
        let trimmed = line.trim();
        if current_impl.is_none()
            && let Some(rest) = trimmed.strip_prefix("impl ")
            && !rest.contains(" for ")
            && trimmed.ends_with('{')
        {
            current_impl = Some((item_name(Some(rest)).expect("impl type").to_owned(), depth));
        }

        if let Some(name) = item_name(
            trimmed
                .strip_prefix("pub const fn ")
                .or_else(|| trimmed.strip_prefix("pub fn ")),
        ) {
            if let Some((impl_type, _)) = &current_impl {
                functions.push(format!("{impl_type}::{name}"));
            } else {
                functions.push(name.to_owned());
            }
        }

        depth = depth
            .saturating_add(line.chars().filter(|character| *character == '{').count())
            .saturating_sub(line.chars().filter(|character| *character == '}').count());
        if let Some((_, impl_start_depth)) = &current_impl
            && depth == *impl_start_depth
        {
            current_impl = None;
        }
    }

    functions.sort();
    functions
}

fn public_top_level_functions(source: &str) -> Vec<String> {
    let mut functions = Vec::new();
    let mut depth = 0usize;

    for line in source.lines() {
        let trimmed = line.trim();
        if depth == 0
            && let Some(name) = item_name(
                trimmed
                    .strip_prefix("pub const fn ")
                    .or_else(|| trimmed.strip_prefix("pub fn ")),
            )
        {
            functions.push(name.to_owned());
        }

        depth = depth
            .saturating_add(line.chars().filter(|character| *character == '{').count())
            .saturating_sub(line.chars().filter(|character| *character == '}').count());
    }

    functions.sort();
    functions
}

fn item_name(rest: Option<&str>) -> Option<&str> {
    let rest = rest?;
    let end = rest
        .find(|character: char| !(character == '_' || character.is_ascii_alphanumeric()))
        .unwrap_or(rest.len());
    (end > 0).then_some(&rest[..end])
}

fn macro_item_name<'a>(line: &'a str, prefix: &str) -> Option<&'a str> {
    let rest = line.strip_prefix(prefix)?;
    rest.strip_suffix(");")
}

fn rust_source_files(root: &std::path::Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_rust_source_files(root, &mut files);
    files.sort();
    files
}

fn collect_rust_source_files(path: &std::path::Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(path).unwrap_or_else(|error| panic!("{}: {error}", path.display())) {
        let entry = entry.expect("directory entry");
        let path = entry.path();
        if path.is_dir() {
            collect_rust_source_files(&path, files);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
}

fn crate_files() -> Vec<String> {
    let root = crate_root();
    let mut files = Vec::new();
    collect_crate_files(&root, &root, &mut files);
    files
}

fn collect_crate_files(root: &std::path::Path, path: &std::path::Path, files: &mut Vec<String>) {
    for entry in fs::read_dir(path).unwrap_or_else(|error| panic!("{}: {error}", path.display())) {
        let entry = entry.expect("directory entry");
        let path = entry.path();
        let name = entry.file_name();
        if name == "target" {
            continue;
        }

        if path.is_dir() {
            collect_crate_files(root, &path, files);
        } else {
            files.push(
                path.strip_prefix(root)
                    .expect("crate file lives in crate root")
                    .display()
                    .to_string(),
            );
        }
    }
}
