use super::*;
use crate::{
    backend::{
        BackendCandidatePayload, BackendCommand, BackendCounterexample, BackendDiagnostic,
        BackendKind, BackendObservation, BackendProfile, BackendProfileId, BackendResourceLimits,
        EncodedBackendProblem, EncodedBackendProblemParts, classify_backend_observation,
        synthetic_backend_result, synthetic_backend_result_with_diagnostics,
    },
    problem::{
        AtpAtom, AtpDeclaration, AtpDeclarationId, AtpDeclarationKind, AtpFingerprint, AtpFormula,
        AtpFormulaId, AtpFormulaTree, AtpProblemParts, AtpProvenance, AtpProvenanceId,
        AtpSourceBinding, AtpSourceRef, AtpSymbolMapEntry, AtpSymbolSource, AtpTypeContext,
        ConcreteFormat, EqualitySupport, ExpectedBackendResult, LogicFragment, LogicProfile,
        NativePropertySupport, QuantifierPolicy, SoftTypeStrategy,
    },
};
use std::collections::BTreeSet;

#[test]
fn plans_are_deterministic_under_shuffled_run_order() {
    let fixture = Fixture::new();
    let first = plan_portfolio(fixture.input(vec![
        fixture.run("run-b", "profile-b", 20, b"b"),
        fixture.run("run-a", "profile-a", 10, b"a"),
    ]))
    .expect("first plan");
    let second = plan_portfolio(fixture.input(vec![
        fixture.run("run-a", "profile-a", 10, b"a"),
        fixture.run("run-b", "profile-b", 20, b"b"),
    ]))
    .expect("second plan");

    assert_eq!(first.plan_hash(), second.plan_hash());
    assert_eq!(
        run_ids(first.scheduled_runs()),
        vec!["run-a".to_owned(), "run-b".to_owned()]
    );
    assert_eq!(
        run_ids(first.scheduled_runs()),
        run_ids(second.scheduled_runs())
    );
}

#[test]
fn shuffled_completion_produces_identical_candidate_sets_and_order() {
    let fixture = Fixture::new();
    let run_a = fixture.run("run-a", "profile-a", 20, b"a");
    let run_b = fixture.run("run-b", "profile-b", 10, b"b");
    let plan_a = plan_portfolio(fixture.input(vec![run_a.clone(), run_b.clone()])).expect("plan a");
    let plan_b = plan_portfolio(fixture.input(vec![run_b.clone(), run_a.clone()])).expect("plan b");
    let result_a = fixture.proved_result(run_a);
    let result_b = fixture.proved_result(run_b);

    let evidence_a = collect_portfolio_results(plan_a, vec![result_a.clone(), result_b.clone()])
        .expect("evidence a");
    let evidence_b =
        collect_portfolio_results(plan_b, vec![result_b, result_a]).expect("evidence b");

    assert_eq!(candidate_ids(&evidence_a), candidate_ids(&evidence_b));
    assert_eq!(
        candidate_ids(&evidence_a),
        vec![
            "run-b:candidate-run-b".to_owned(),
            "run-a:candidate-run-a".to_owned()
        ]
    );
    assert_eq!(
        evidence_a.evidence_set_hash(),
        evidence_b.evidence_set_hash()
    );
    assert_eq!(
        evidence_a.stop_summary().reason(),
        PortfolioStopReason::AllRunsCollected
    );
}

#[test]
fn cancellation_returns_no_partial_candidates_or_missing_result_error() {
    let fixture = Fixture::new();
    let cancellation = BackendCancellationToken::new();
    let run_a = fixture.run_with_cancellation("run-a", "profile-a", 10, b"a", &cancellation);
    let run_b = fixture.run_with_cancellation("run-b", "profile-b", 20, b"b", &cancellation);
    let plan = plan_portfolio(
        fixture.input_with_cancellation(vec![run_a.clone(), run_b], cancellation.clone()),
    )
    .expect("plan");
    let partial_result = fixture.proved_result(run_a);
    cancellation.cancel();

    let evidence =
        collect_portfolio_results(plan, vec![partial_result]).expect("cancelled evidence");

    assert_eq!(
        evidence.stop_summary().reason(),
        PortfolioStopReason::Cancelled
    );
    assert!(evidence.candidates().is_empty());
    assert_eq!(evidence.backend_results().len(), 1);
}

#[test]
fn collection_fail_closes_for_missing_unknown_and_duplicate_results() {
    let fixture = Fixture::new();
    let run_a = fixture.run("run-a", "profile-a", 10, b"a");
    let run_b = fixture.run("run-b", "profile-b", 20, b"b");
    let plan = plan_portfolio(fixture.input(vec![run_a.clone(), run_b.clone()])).expect("plan");
    let missing =
        collect_portfolio_results(plan.clone(), vec![fixture.proved_result(run_a.clone())]);
    assert!(matches!(
        missing,
        Err(PortfolioError::MissingBackendResult { run_id }) if run_id == "run-b"
    ));

    let unknown_run = fixture.run("run-z", "profile-z", 30, b"z");
    let unknown = collect_portfolio_results(
        plan.clone(),
        vec![
            fixture.proved_result(run_a.clone()),
            fixture.proved_result(run_b.clone()),
            fixture.proved_result(unknown_run),
        ],
    );
    assert!(matches!(
        unknown,
        Err(PortfolioError::UnknownBackendResult { run_id }) if run_id == "run-z"
    ));

    let duplicate_result = fixture.proved_result(run_b);
    let duplicate =
        collect_portfolio_results(plan, vec![duplicate_result.clone(), duplicate_result]);
    assert!(matches!(
        duplicate,
        Err(PortfolioError::DuplicateBackendResult { run_id }) if run_id == "run-b"
    ));
}

#[test]
fn planning_rejects_backend_runs_outside_the_target_problem() {
    let fixture = Fixture::new();
    let other_problem = minimal_problem_with_vc(8);
    let problem_mismatch = fixture.run_with_encoded_problem(
        "run-problem",
        "profile-a",
        10,
        fixture.encoded_problem_for(
            b"a",
            other_problem.problem_id(),
            fixture.problem.target_binding().clone(),
            ExpectedBackendResult::Unsat,
            ConcreteFormat::Tptp,
        ),
        ConcreteFormat::Tptp,
        &BackendCancellationToken::new(),
    );
    assert!(matches!(
        plan_portfolio(fixture.input(vec![problem_mismatch])),
        Err(PortfolioError::BackendRunProblemMismatch { run_id }) if run_id == "run-problem"
    ));

    let target_mismatch = fixture.run_with_encoded_problem(
        "run-target",
        "profile-a",
        10,
        fixture.encoded_problem_for(
            b"a",
            fixture.problem.problem_id(),
            target_binding(9),
            ExpectedBackendResult::Unsat,
            ConcreteFormat::Tptp,
        ),
        ConcreteFormat::Tptp,
        &BackendCancellationToken::new(),
    );
    assert!(matches!(
        plan_portfolio(fixture.input(vec![target_mismatch])),
        Err(PortfolioError::BackendRunTargetMismatch { run_id }) if run_id == "run-target"
    ));

    let format_mismatch = fixture.run_with_encoded_problem(
        "run-format",
        "profile-smt",
        10,
        fixture.encoded_problem_for(
            b"(assert true)",
            fixture.problem.problem_id(),
            fixture.problem.target_binding().clone(),
            ExpectedBackendResult::Unsat,
            ConcreteFormat::SmtLib,
        ),
        ConcreteFormat::SmtLib,
        &BackendCancellationToken::new(),
    );
    assert!(matches!(
        plan_portfolio(fixture.input(vec![format_mismatch])),
        Err(PortfolioError::BackendRunFormatMismatch { run_id }) if run_id == "run-format"
    ));
}

#[test]
fn result_metadata_mismatch_is_rejected() {
    let fixture = Fixture::new();
    let scheduled = fixture.run("run-a", "profile-a", 10, b"a");
    let mismatched_same_id = fixture.run("run-a", "profile-a", 10, b"different");
    let plan = plan_portfolio(fixture.input(vec![scheduled])).expect("plan");

    let error = collect_portfolio_results(plan, vec![fixture.proved_result(mismatched_same_id)]);

    assert!(matches!(
        error,
        Err(PortfolioError::BackendResultMetadataMismatch { run_id }) if run_id == "run-a"
    ));

    let scheduled = fixture.run("run-b", "profile-a", 10, b"a");
    let mismatched_command =
        fixture.run_with_command_id("run-b", "profile-a", 10, b"a", "different-command");
    let plan = plan_portfolio(fixture.input(vec![scheduled])).expect("plan");

    let error = collect_portfolio_results(plan, vec![fixture.proved_result(mismatched_command)]);

    assert!(matches!(
        error,
        Err(PortfolioError::BackendResultMetadataMismatch { run_id }) if run_id == "run-b"
    ));
}

#[test]
fn no_schedulable_or_budget_exhausted_plan_has_no_candidates() {
    let fixture = Fixture::new();
    let empty_plan = plan_portfolio(fixture.input(Vec::new())).expect("empty plan");
    let empty_evidence = collect_portfolio_results(empty_plan, Vec::new()).expect("empty evidence");
    assert_eq!(
        empty_evidence.stop_summary().reason(),
        PortfolioStopReason::NoSchedulableProfile
    );
    assert!(empty_evidence.candidates().is_empty());

    let over_budget = plan_portfolio(fixture.input_with_obligation_budget(
        vec![
            fixture.run("run-a", "profile-a", 10, b"a"),
            fixture.run("run-b", "profile-b", 20, b"b"),
        ],
        PortfolioBudget::with_max_scheduled_runs(1),
    ))
    .expect("budget plan");
    assert_eq!(
        over_budget.stop_summary().reason(),
        PortfolioStopReason::BudgetExhausted
    );
    assert!(over_budget.scheduled_runs().is_empty());
}

#[test]
fn stopped_plans_still_reject_unexpected_results() {
    let fixture = Fixture::new();
    let empty_plan = plan_portfolio(fixture.input(Vec::new())).expect("empty plan");
    let unexpected = fixture.run("run-z", "profile-z", 30, b"z");

    let error = collect_portfolio_results(empty_plan, vec![fixture.proved_result(unexpected)]);

    assert!(matches!(
        error,
        Err(PortfolioError::UnknownBackendResult { run_id }) if run_id == "run-z"
    ));

    let cancellation = BackendCancellationToken::new();
    cancellation.cancel();
    let cancelled_plan = plan_portfolio(fixture.input_with_cancellation(
        vec![fixture.run_with_cancellation("run-a", "profile-a", 10, b"a", &cancellation)],
        cancellation,
    ))
    .expect("cancelled plan");
    let unexpected = fixture.run("run-z", "profile-z", 30, b"z");

    let error = collect_portfolio_results(cancelled_plan, vec![fixture.proved_result(unexpected)]);

    assert!(matches!(
        error,
        Err(PortfolioError::UnknownBackendResult { run_id }) if run_id == "run-z"
    ));
}

#[test]
fn candidate_metadata_mismatch_fails_before_evidence_set() {
    let fixture = Fixture::new();
    for (index, reason) in [
        "unsupported_candidate_payload",
        "candidate_target_mismatch",
        "candidate_input_hash_mismatch",
        "candidate_provenance_mismatch",
        "candidate_label_mismatch",
        "candidate_symbol_mismatch",
    ]
    .into_iter()
    .enumerate()
    {
        let run_id = format!("run-{index}");
        let run = fixture.run(&run_id, "profile-a", 10, b"a");
        let candidate = candidate_with_mismatch(run.encoded_problem(), reason);
        let plan = plan_portfolio(fixture.input(vec![run.clone()])).expect("plan");
        let result = fixture.proved_result_with_unchecked_candidate(run, candidate);

        let error = collect_portfolio_results(plan, vec![result]);

        assert!(matches!(
            error,
            Err(PortfolioError::CandidateMetadataMismatch {
                run_id: rejected_run_id,
                reason: rejected_reason,
            }) if rejected_run_id == run_id && rejected_reason == reason
        ));
    }
}

#[test]
fn non_proof_backend_statuses_are_collected_without_formula_candidates() {
    let fixture = Fixture::new();
    let timeout = fixture.run("run-timeout", "profile-a", 10, b"a");
    let error = fixture.run("run-error", "profile-b", 20, b"b");
    let unknown = fixture.run("run-unknown", "profile-c", 30, b"c");
    let malformed = fixture.run("run-malformed", "profile-d", 40, b"d");
    let plan = plan_portfolio(fixture.input(vec![
        timeout.clone(),
        error.clone(),
        unknown.clone(),
        malformed.clone(),
    ]))
    .expect("plan");

    let evidence = collect_portfolio_results(
        plan,
        vec![
            fixture.status_result_with_diagnostic(
                timeout,
                BackendRunStatus::Timeout,
                "backend_timeout",
            ),
            fixture.status_result_with_diagnostic(error, BackendRunStatus::Error, "backend_error"),
            fixture.observed_unknown_result(unknown),
            fixture.malformed_result(malformed),
        ],
    )
    .expect("evidence");

    assert_eq!(
        evidence.stop_summary().reason(),
        PortfolioStopReason::AllRunsCollected
    );
    assert!(evidence.candidates().is_empty());
    assert_eq!(evidence.backend_results().len(), 4);
    assert_eq!(
        result_status(&evidence, "run-timeout"),
        BackendRunStatus::Timeout
    );
    assert_eq!(
        result_status(&evidence, "run-error"),
        BackendRunStatus::Error
    );
    assert_eq!(
        result_status(&evidence, "run-unknown"),
        BackendRunStatus::Unknown
    );
    assert_eq!(
        result_status(&evidence, "run-malformed"),
        BackendRunStatus::Error
    );
    assert!(result_has_diagnostic(
        &evidence,
        "run-timeout",
        "backend_timeout"
    ));
    assert!(result_has_diagnostic(
        &evidence,
        "run-error",
        "backend_error"
    ));
    assert!(result_has_diagnostic(
        &evidence,
        "run-unknown",
        "backend_result_unknown"
    ));
    assert!(result_has_diagnostic(
        &evidence,
        "run-malformed",
        "backend_result_malformed"
    ));
}

#[test]
fn counterexample_status_produces_untrusted_counterexample_candidate() {
    let fixture = Fixture::new();
    let run = fixture.run("run-counterexample", "profile-a", 10, b"a");
    let plan = plan_portfolio(fixture.input(vec![run.clone()])).expect("plan");

    let evidence = collect_portfolio_results(plan, vec![fixture.counterexample_result(run)])
        .expect("evidence");

    assert_eq!(evidence.candidates().len(), 1);
    assert_eq!(
        evidence.candidates()[0].candidate_kind(),
        PortfolioCandidateKind::Counterexample
    );
    assert_eq!(
        evidence.candidates()[0].evidence_format(),
        PortfolioEvidenceFormat::Counterexample
    );
}

#[test]
fn portfolio_source_excludes_trusted_or_downstream_boundary_strings() {
    let source = include_str!("../portfolio.rs");
    let prohibited = [
        ["mizar_", "kernel::"].concat(),
        ["kernel", "_verified"].concat(),
        ["Kernel", "CheckResult"].concat(),
        ["Proof", "Witness"].concat(),
        ["Proof", "WitnessDraft"].concat(),
        ["ProofPolicy", "Evaluator"].concat(),
        ["proof", "_cache"].concat(),
        ["cache", "_promotion"].concat(),
        ["accepted", "_proof_status"].concat(),
        ["caller", "_supplied"].concat(),
        ["instantiated", "_formulas"].concat(),
        ["Sat", "Problem"].concat(),
        ["DI", "MACS"].concat(),
        ["real", "_output_extraction"].concat(),
        ["Resolution", "Trace"].concat(),
        ["Mini", "SAT"].concat(),
    ];
    for prohibited in prohibited {
        assert!(
            !source.contains(&prohibited),
            "portfolio source must not contain prohibited boundary string {prohibited}"
        );
    }
}

#[derive(Clone)]
struct Fixture {
    problem: AtpProblem,
    vc_hash: Hash,
    provenance_hash: Hash,
}

impl Fixture {
    fn new() -> Self {
        Self {
            problem: minimal_problem(),
            vc_hash: hash_with_seed(3),
            provenance_hash: hash_with_seed(9),
        }
    }

    fn input(&self, backend_runs: Vec<BackendRunInput>) -> PortfolioInput {
        self.input_with_obligation_budget(backend_runs, PortfolioBudget::unbounded())
    }

    fn input_with_obligation_budget(
        &self,
        backend_runs: Vec<BackendRunInput>,
        obligation_budget: PortfolioBudget,
    ) -> PortfolioInput {
        self.input_parts(
            backend_runs,
            obligation_budget,
            BackendCancellationToken::new(),
        )
    }

    fn input_with_cancellation(
        &self,
        backend_runs: Vec<BackendRunInput>,
        cancellation: BackendCancellationToken,
    ) -> PortfolioInput {
        self.input_parts(backend_runs, PortfolioBudget::unbounded(), cancellation)
    }

    fn input_parts(
        &self,
        backend_runs: Vec<BackendRunInput>,
        obligation_budget: PortfolioBudget,
        cancellation: BackendCancellationToken,
    ) -> PortfolioInput {
        PortfolioInput::new(PortfolioInputParts {
            portfolio_id: PortfolioId::new("portfolio").expect("portfolio id"),
            vc_hash: self.vc_hash,
            atp_problem: self.problem.clone(),
            backend_runs,
            obligation_budget,
            scheduler_budget: PortfolioBudget::unbounded(),
            policy_constraints: PortfolioPolicyConstraints::new(),
            cancellation,
        })
    }

    fn run(
        &self,
        run_id: &str,
        profile_id: &str,
        priority: u32,
        input_text: &[u8],
    ) -> BackendRunInput {
        self.run_with_cancellation(
            run_id,
            profile_id,
            priority,
            input_text,
            &BackendCancellationToken::new(),
        )
    }

    fn run_with_cancellation(
        &self,
        run_id: &str,
        profile_id: &str,
        priority: u32,
        input_text: &[u8],
        cancellation: &BackendCancellationToken,
    ) -> BackendRunInput {
        BackendRunInput::new(
            BackendRunId::new(run_id).expect("run id"),
            self.encoded_problem(input_text),
            BackendProfile::new(
                BackendProfileId::new(profile_id).expect("profile id"),
                BackendKind::new("mock").expect("backend kind"),
                ConcreteFormat::Tptp,
            )
            .with_deterministic_priority(priority),
            BackendCommand::new("mock-backend", Vec::new()).expect("command"),
            BackendResourceLimits::new(),
            BackendIoMode::Stdin,
            cancellation.clone(),
        )
    }

    fn run_with_command_id(
        &self,
        run_id: &str,
        profile_id: &str,
        priority: u32,
        input_text: &[u8],
        command_id: &str,
    ) -> BackendRunInput {
        BackendRunInput::new(
            BackendRunId::new(run_id).expect("run id"),
            self.encoded_problem(input_text),
            BackendProfile::new(
                BackendProfileId::new(profile_id).expect("profile id"),
                BackendKind::new("mock").expect("backend kind"),
                ConcreteFormat::Tptp,
            )
            .with_deterministic_priority(priority),
            BackendCommand::new("mock-backend", Vec::new())
                .expect("command")
                .with_semantic_executable_id(command_id)
                .expect("semantic command id"),
            BackendResourceLimits::new(),
            BackendIoMode::Stdin,
            BackendCancellationToken::new(),
        )
    }

    fn run_with_encoded_problem(
        &self,
        run_id: &str,
        profile_id: &str,
        priority: u32,
        encoded_problem: EncodedBackendProblem,
        profile_format: ConcreteFormat,
        cancellation: &BackendCancellationToken,
    ) -> BackendRunInput {
        BackendRunInput::new(
            BackendRunId::new(run_id).expect("run id"),
            encoded_problem,
            BackendProfile::new(
                BackendProfileId::new(profile_id).expect("profile id"),
                BackendKind::new("mock").expect("backend kind"),
                profile_format,
            )
            .with_deterministic_priority(priority),
            BackendCommand::new("mock-backend", Vec::new()).expect("command"),
            BackendResourceLimits::new(),
            BackendIoMode::Stdin,
            cancellation.clone(),
        )
    }

    fn encoded_problem(&self, input_text: &[u8]) -> EncodedBackendProblem {
        self.encoded_problem_for(
            input_text,
            self.problem.problem_id(),
            self.problem.target_binding().clone(),
            self.problem.expected_result(),
            ConcreteFormat::Tptp,
        )
    }

    fn encoded_problem_for(
        &self,
        input_text: &[u8],
        problem_id: AtpProblemId,
        target_binding: AtpTargetBinding,
        expected_result: ExpectedBackendResult,
        concrete_format: ConcreteFormat,
    ) -> EncodedBackendProblem {
        EncodedBackendProblem::new(EncodedBackendProblemParts {
            problem_id,
            target_binding,
            expected_result,
            concrete_format,
            logic_profile_name: self.problem.logic_profile().name().as_str().to_owned(),
            logic_fragment: match concrete_format {
                ConcreteFormat::Tptp => "Fof",
                ConcreteFormat::SmtLib => "QF_UF",
            }
            .to_owned(),
            input_text: input_text.to_vec(),
            formula_labels: vec!["ax_1".to_owned(), "conj_1".to_owned()],
            symbol_bindings: vec!["P".to_owned()],
            provenance_hash: self.provenance_hash,
        })
        .expect("encoded problem")
    }

    fn proved_result(&self, run: BackendRunInput) -> BackendRunResult {
        let candidate = matching_candidate(
            run.encoded_problem(),
            format!("candidate-{}", run.run_id().as_str()),
        );
        self.proved_result_with_candidate(run, candidate)
    }

    fn proved_result_with_candidate(
        &self,
        run: BackendRunInput,
        candidate: BackendCandidateEvidence,
    ) -> BackendRunResult {
        let base = synthetic_backend_result(&run, BackendRunStatus::Unknown);
        classify_backend_observation(
            base,
            BackendObservation::new(BackendObservedResult::Unsat)
                .with_candidate_evidence(candidate),
        )
    }

    fn proved_result_with_unchecked_candidate(
        &self,
        run: BackendRunInput,
        candidate: BackendCandidateEvidence,
    ) -> BackendRunResult {
        classify_backend_observation(
            synthetic_backend_result(&run, BackendRunStatus::Proved),
            BackendObservation::new(BackendObservedResult::Unsat)
                .with_candidate_evidence(candidate),
        )
    }

    fn status_result_with_diagnostic(
        &self,
        run: BackendRunInput,
        status: BackendRunStatus,
        diagnostic_key: &'static str,
    ) -> BackendRunResult {
        synthetic_backend_result_with_diagnostics(
            &run,
            status,
            vec![BackendDiagnostic::new(
                diagnostic_key,
                "synthetic diagnostic",
            )],
        )
    }

    fn observed_unknown_result(&self, run: BackendRunInput) -> BackendRunResult {
        classify_backend_observation(
            synthetic_backend_result(&run, BackendRunStatus::Unknown),
            BackendObservation::new(BackendObservedResult::Unknown),
        )
    }

    fn malformed_result(&self, run: BackendRunInput) -> BackendRunResult {
        classify_backend_observation(
            synthetic_backend_result(&run, BackendRunStatus::Unknown),
            BackendObservation::new(BackendObservedResult::Malformed),
        )
    }

    fn counterexample_result(&self, run: BackendRunInput) -> BackendRunResult {
        classify_backend_observation(
            synthetic_backend_result(&run, BackendRunStatus::Unknown),
            BackendObservation::new(BackendObservedResult::Sat).with_counterexample(
                BackendCounterexample::new(b"model".to_vec(), true).expect("counterexample"),
            ),
        )
    }
}

fn run_ids(runs: &[BackendRunInput]) -> Vec<String> {
    runs.iter()
        .map(|run| run.run_id().as_str().to_owned())
        .collect()
}

fn candidate_ids(evidence: &PortfolioEvidenceSet) -> Vec<String> {
    evidence
        .candidates()
        .iter()
        .map(|candidate| candidate.candidate_id().as_str().to_owned())
        .collect()
}

fn result_status(evidence: &PortfolioEvidenceSet, run_id: &str) -> BackendRunStatus {
    evidence
        .backend_results()
        .iter()
        .find(|result| result.run_id().as_str() == run_id)
        .map(BackendRunResult::status)
        .expect("result exists")
}

fn result_has_diagnostic(
    evidence: &PortfolioEvidenceSet,
    run_id: &str,
    diagnostic_key: &str,
) -> bool {
    evidence
        .backend_results()
        .iter()
        .find(|result| result.run_id().as_str() == run_id)
        .expect("result exists")
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.key() == diagnostic_key)
}

fn matching_candidate(
    problem: &EncodedBackendProblem,
    candidate_id: impl Into<String>,
) -> BackendCandidateEvidence {
    BackendCandidateEvidence::new(
        candidate_id,
        BackendCandidatePayload::FormulaSubstitutionBytes(b"candidate".to_vec()),
        problem.target_binding().clone(),
        problem.input_hash(),
        problem.provenance_hash(),
        problem.formula_labels().to_vec(),
        problem.symbol_bindings().to_vec(),
    )
    .expect("candidate")
}

fn candidate_with_mismatch(
    problem: &EncodedBackendProblem,
    reason: &'static str,
) -> BackendCandidateEvidence {
    let payload = if reason == "unsupported_candidate_payload" {
        BackendCandidatePayload::BackendLog(vec![1])
    } else {
        BackendCandidatePayload::FormulaSubstitutionBytes(b"candidate".to_vec())
    };
    let target_binding = if reason == "candidate_target_mismatch" {
        target_binding(99)
    } else {
        problem.target_binding().clone()
    };
    let encoded_problem_hash = if reason == "candidate_input_hash_mismatch" {
        hash_with_seed(99)
    } else {
        problem.input_hash()
    };
    let provenance_hash = if reason == "candidate_provenance_mismatch" {
        hash_with_seed(98)
    } else {
        problem.provenance_hash()
    };
    let formula_label_refs = if reason == "candidate_label_mismatch" {
        vec!["different_label".to_owned()]
    } else {
        problem.formula_labels().to_vec()
    };
    let symbol_binding_refs = if reason == "candidate_symbol_mismatch" {
        vec!["DifferentSymbol".to_owned()]
    } else {
        problem.symbol_bindings().to_vec()
    };
    BackendCandidateEvidence::new(
        format!("candidate-{reason}"),
        payload,
        target_binding,
        encoded_problem_hash,
        provenance_hash,
        formula_label_refs,
        symbol_binding_refs,
    )
    .expect("candidate")
}

fn minimal_problem() -> AtpProblem {
    minimal_problem_with_vc(7)
}

fn minimal_problem_with_vc(vc_index: u32) -> AtpProblem {
    AtpProblem::try_new(AtpProblemParts {
        vc_id: VcId::new(vc_index as usize),
        target_binding: target_binding(vc_index),
        logic_profile: LogicProfile::try_new(
            "fof-fixture",
            LogicFragment::Fof,
            EqualitySupport::Supported,
            QuantifierPolicy::PropositionalOnly,
            SoftTypeStrategy::BackendSorts,
            NativePropertySupport::Unsupported,
            BTreeSet::from([ConcreteFormat::Tptp]),
        )
        .expect("logic profile"),
        expected_result: ExpectedBackendResult::Unsat,
        declarations: vec![AtpDeclaration::new(
            AtpDeclarationId::new(1),
            AtpDeclarationKind::Predicate,
            "P",
            0,
            AtpProvenanceId::new(1),
        )],
        axioms: vec![AtpFormula::new(
            AtpFormulaId::new(1),
            AtpFormulaTree::Atom(AtpAtom::new("P", Vec::new())),
            AtpProvenanceId::new(2),
        )],
        conjecture: AtpFormula::new(
            AtpFormulaId::new(2),
            AtpFormulaTree::Atom(AtpAtom::new("P", Vec::new())),
            AtpProvenanceId::new(3),
        ),
        type_context: AtpTypeContext::new(Vec::new()),
        properties: Vec::new(),
        symbol_map: vec![AtpSymbolMapEntry::new(
            "P",
            AtpSymbolSource::MizarSymbol(AtpSourceBinding::new("pred:P")),
        )],
        provenance: vec![
            AtpProvenance::new(
                AtpProvenanceId::new(1),
                AtpSourceRef::LocalHypothesis(AtpSourceBinding::new("decl:P")),
                "decl",
            ),
            AtpProvenance::new(
                AtpProvenanceId::new(2),
                AtpSourceRef::CitedPremise(AtpSourceBinding::new("premise:1")),
                "premise",
            ),
            AtpProvenance::new(
                AtpProvenanceId::new(3),
                AtpSourceRef::GeneratedVcFact(AtpSourceBinding::new("goal:1")),
                "goal",
            ),
        ],
        diagnostics: Vec::new(),
    })
    .expect("minimal problem")
}

fn target_binding(vc_index: u32) -> AtpTargetBinding {
    AtpTargetBinding::new(
        AtpFingerprint::new(18, format!("target-vc-{vc_index}").into_bytes()).expect("fingerprint"),
        AtpSourceBinding::new(format!("vc:{vc_index}")),
    )
    .expect("target binding")
}

fn hash_with_seed(seed: u8) -> Hash {
    Hash::from_bytes([seed; Hash::BYTE_LEN])
}
