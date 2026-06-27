//! Policy-neutral ATP portfolio planning and candidate collection.
//!
//! This module implements the task-18 no-early-stop slice specified in
//! [portfolio.md](../../../doc/design/mizar-atp/en/portfolio.md). It builds
//! deterministic plans from prebuilt backend runs and collects terminal backend
//! results into untrusted evidence sets. It does not run backends directly,
//! call the kernel, evaluate proof policy, select artifact winners, publish
//! witnesses, update caches, or trust backend proof material.

use crate::{
    backend::{
        BackendCancellationToken, BackendCandidateEvidence, BackendCandidatePayload,
        BackendDiagnostic, BackendIoMode, BackendLimitRequirement, BackendObservedResult,
        BackendRunId, BackendRunInput, BackendRunResult, BackendRunStatus,
        BackendWorkingDirectoryPolicy, backend_run_command_fingerprint,
    },
    problem::{AtpProblem, AtpProblemId, AtpTargetBinding},
};
use mizar_session::Hash;
use mizar_vc::vc_ir::VcId;
use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt,
    time::Duration,
};

const PLAN_HASH_DOMAIN: &str = "mizar-atp/portfolio-plan/v1";
const CANDIDATE_HASH_DOMAIN: &str = "mizar-atp/portfolio-candidate/v1";
const RESULT_HASH_DOMAIN: &str = "mizar-atp/portfolio-result/v1";
const EVIDENCE_SET_HASH_DOMAIN: &str = "mizar-atp/portfolio-evidence-set/v1";
const PAYLOAD_HASH_DOMAIN: &str = "mizar-atp/portfolio-payload/v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortfolioId(String);

impl PortfolioId {
    pub fn new(value: impl Into<String>) -> Result<Self, PortfolioError> {
        let value = value.into();
        reject_empty("portfolio_id", &value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct PortfolioInputParts {
    pub portfolio_id: PortfolioId,
    pub vc_hash: Hash,
    pub atp_problem: AtpProblem,
    pub backend_runs: Vec<BackendRunInput>,
    pub obligation_budget: PortfolioBudget,
    pub scheduler_budget: PortfolioBudget,
    pub policy_constraints: PortfolioPolicyConstraints,
    pub cancellation: BackendCancellationToken,
}

#[derive(Debug, Clone)]
pub struct PortfolioInput {
    portfolio_id: PortfolioId,
    vc_hash: Hash,
    atp_problem: AtpProblem,
    backend_runs: Vec<BackendRunInput>,
    obligation_budget: PortfolioBudget,
    scheduler_budget: PortfolioBudget,
    policy_constraints: PortfolioPolicyConstraints,
    cancellation: BackendCancellationToken,
}

impl PortfolioInput {
    pub fn new(parts: PortfolioInputParts) -> Self {
        Self {
            portfolio_id: parts.portfolio_id,
            vc_hash: parts.vc_hash,
            atp_problem: parts.atp_problem,
            backend_runs: parts.backend_runs,
            obligation_budget: parts.obligation_budget,
            scheduler_budget: parts.scheduler_budget,
            policy_constraints: parts.policy_constraints,
            cancellation: parts.cancellation,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PortfolioBudget {
    max_scheduled_runs: Option<usize>,
}

impl PortfolioBudget {
    pub const fn unbounded() -> Self {
        Self {
            max_scheduled_runs: None,
        }
    }

    pub const fn with_max_scheduled_runs(max_scheduled_runs: usize) -> Self {
        Self {
            max_scheduled_runs: Some(max_scheduled_runs),
        }
    }

    pub const fn max_scheduled_runs(self) -> Option<usize> {
        self.max_scheduled_runs
    }
}

impl Default for PortfolioBudget {
    fn default() -> Self {
        Self::unbounded()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PortfolioPolicyConstraints {
    record_externally_attested: bool,
}

impl PortfolioPolicyConstraints {
    pub const fn new() -> Self {
        Self {
            record_externally_attested: false,
        }
    }

    pub const fn with_externally_attested_records(mut self, allowed: bool) -> Self {
        self.record_externally_attested = allowed;
        self
    }

    pub const fn record_externally_attested(self) -> bool {
        self.record_externally_attested
    }
}

impl Default for PortfolioPolicyConstraints {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct PortfolioPlan {
    portfolio_id: PortfolioId,
    vc_id: VcId,
    vc_hash: Hash,
    atp_problem_id: AtpProblemId,
    target_binding: AtpTargetBinding,
    backend_runs: Vec<BackendRunInput>,
    plan_hash: Hash,
    stop_summary: PortfolioStopSummary,
    diagnostics: Vec<PortfolioDiagnostic>,
    policy_constraints: PortfolioPolicyConstraints,
    cancellation: BackendCancellationToken,
}

impl PortfolioPlan {
    pub const fn portfolio_id(&self) -> &PortfolioId {
        &self.portfolio_id
    }

    pub const fn vc_id(&self) -> VcId {
        self.vc_id
    }

    pub const fn vc_hash(&self) -> Hash {
        self.vc_hash
    }

    pub const fn atp_problem_id(&self) -> AtpProblemId {
        self.atp_problem_id
    }

    pub const fn target_binding(&self) -> &AtpTargetBinding {
        &self.target_binding
    }

    pub fn scheduled_runs(&self) -> &[BackendRunInput] {
        &self.backend_runs
    }

    pub const fn plan_hash(&self) -> Hash {
        self.plan_hash
    }

    pub const fn stop_summary(&self) -> &PortfolioStopSummary {
        &self.stop_summary
    }

    pub fn diagnostics(&self) -> &[PortfolioDiagnostic] {
        &self.diagnostics
    }

    pub const fn policy_constraints(&self) -> PortfolioPolicyConstraints {
        self.policy_constraints
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortfolioEvidenceSet {
    portfolio_id: PortfolioId,
    vc_id: VcId,
    vc_hash: Hash,
    atp_problem_id: AtpProblemId,
    plan_hash: Hash,
    evidence_set_hash: Hash,
    backend_results: Vec<BackendRunResult>,
    candidates: Vec<PortfolioCandidate>,
    stop_summary: PortfolioStopSummary,
    diagnostics: Vec<PortfolioDiagnostic>,
}

impl PortfolioEvidenceSet {
    pub const fn portfolio_id(&self) -> &PortfolioId {
        &self.portfolio_id
    }

    pub const fn vc_id(&self) -> VcId {
        self.vc_id
    }

    pub const fn vc_hash(&self) -> Hash {
        self.vc_hash
    }

    pub const fn atp_problem_id(&self) -> AtpProblemId {
        self.atp_problem_id
    }

    pub const fn plan_hash(&self) -> Hash {
        self.plan_hash
    }

    pub const fn evidence_set_hash(&self) -> Hash {
        self.evidence_set_hash
    }

    pub fn backend_results(&self) -> &[BackendRunResult] {
        &self.backend_results
    }

    pub fn candidates(&self) -> &[PortfolioCandidate] {
        &self.candidates
    }

    pub const fn stop_summary(&self) -> &PortfolioStopSummary {
        &self.stop_summary
    }

    pub fn diagnostics(&self) -> &[PortfolioDiagnostic] {
        &self.diagnostics
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortfolioCandidateId(String);

impl PortfolioCandidateId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortfolioCandidate {
    candidate_id: PortfolioCandidateId,
    source_run_id: BackendRunId,
    backend_profile_id: String,
    backend_profile_priority: u32,
    encoded_problem_hash: Hash,
    target_binding: AtpTargetBinding,
    candidate_kind: PortfolioCandidateKind,
    evidence_format: PortfolioEvidenceFormat,
    evidence_payload_hash: Option<Hash>,
    observed_result: Option<BackendObservedResult>,
    provenance_hash: Hash,
    candidate_hash: Hash,
    diagnostics: Vec<PortfolioDiagnostic>,
}

impl PortfolioCandidate {
    pub const fn candidate_id(&self) -> &PortfolioCandidateId {
        &self.candidate_id
    }

    pub const fn source_run_id(&self) -> &BackendRunId {
        &self.source_run_id
    }

    pub fn backend_profile_id(&self) -> &str {
        &self.backend_profile_id
    }

    pub const fn encoded_problem_hash(&self) -> Hash {
        self.encoded_problem_hash
    }

    pub const fn target_binding(&self) -> &AtpTargetBinding {
        &self.target_binding
    }

    pub const fn candidate_kind(&self) -> PortfolioCandidateKind {
        self.candidate_kind
    }

    pub const fn evidence_format(&self) -> PortfolioEvidenceFormat {
        self.evidence_format
    }

    pub const fn evidence_payload_hash(&self) -> Option<Hash> {
        self.evidence_payload_hash
    }

    pub const fn observed_result(&self) -> Option<BackendObservedResult> {
        self.observed_result
    }

    pub const fn provenance_hash(&self) -> Hash {
        self.provenance_hash
    }

    pub const fn candidate_hash(&self) -> Hash {
        self.candidate_hash
    }

    pub fn diagnostics(&self) -> &[PortfolioDiagnostic] {
        &self.diagnostics
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum PortfolioCandidateKind {
    FormulaSubstitution,
    ExternallyAttested,
    Counterexample,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum PortfolioEvidenceFormat {
    FormulaSubstitutionBytes,
    FormulaSubstitutionRef,
    ExternallyAttested,
    Counterexample,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortfolioStopSummary {
    reason: PortfolioStopReason,
    message: String,
}

impl PortfolioStopSummary {
    pub fn new(reason: PortfolioStopReason, message: impl Into<String>) -> Self {
        Self {
            reason,
            message: message.into(),
        }
    }

    pub const fn reason(&self) -> PortfolioStopReason {
        self.reason
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum PortfolioStopReason {
    NotStopped,
    AllRunsCollected,
    NoSchedulableProfile,
    BudgetExhausted,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortfolioDiagnostic {
    key: String,
    message: String,
}

impl PortfolioDiagnostic {
    pub fn new(key: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            message: message.into(),
        }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PortfolioError {
    EmptyField {
        field: &'static str,
    },
    DuplicateBackendRun {
        run_id: String,
    },
    DuplicateBackendResult {
        run_id: String,
    },
    UnknownBackendResult {
        run_id: String,
    },
    MissingBackendResult {
        run_id: String,
    },
    BackendRunProblemMismatch {
        run_id: String,
    },
    BackendRunTargetMismatch {
        run_id: String,
    },
    BackendRunExpectedResultMismatch {
        run_id: String,
    },
    BackendRunFormatMismatch {
        run_id: String,
    },
    BackendResultMetadataMismatch {
        run_id: String,
    },
    CandidateMetadataMismatch {
        run_id: String,
        reason: &'static str,
    },
}

impl fmt::Display for PortfolioError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField { field } => write!(formatter, "empty portfolio field `{field}`"),
            Self::DuplicateBackendRun { run_id } => {
                write!(formatter, "duplicate backend run `{run_id}`")
            }
            Self::DuplicateBackendResult { run_id } => {
                write!(formatter, "duplicate backend result `{run_id}`")
            }
            Self::UnknownBackendResult { run_id } => {
                write!(formatter, "backend result `{run_id}` is not in the plan")
            }
            Self::MissingBackendResult { run_id } => {
                write!(formatter, "missing backend result `{run_id}`")
            }
            Self::BackendRunProblemMismatch { run_id } => {
                write!(
                    formatter,
                    "backend run `{run_id}` targets a different problem"
                )
            }
            Self::BackendRunTargetMismatch { run_id } => {
                write!(
                    formatter,
                    "backend run `{run_id}` has a mismatched target binding"
                )
            }
            Self::BackendRunExpectedResultMismatch { run_id } => {
                write!(
                    formatter,
                    "backend run `{run_id}` has a mismatched expected result"
                )
            }
            Self::BackendRunFormatMismatch { run_id } => {
                write!(formatter, "backend run `{run_id}` has a mismatched format")
            }
            Self::BackendResultMetadataMismatch { run_id } => {
                write!(
                    formatter,
                    "backend result `{run_id}` does not match the scheduled run"
                )
            }
            Self::CandidateMetadataMismatch { run_id, reason } => {
                write!(
                    formatter,
                    "candidate from backend result `{run_id}` failed metadata check `{reason}`"
                )
            }
        }
    }
}

impl Error for PortfolioError {}

pub fn plan_portfolio(input: PortfolioInput) -> Result<PortfolioPlan, PortfolioError> {
    let mut backend_runs = input.backend_runs;
    validate_backend_runs(&input.atp_problem, &mut backend_runs)?;
    backend_runs.sort_by(compare_backend_run);

    let mut diagnostics = Vec::new();
    let stop_summary = if input.cancellation.is_cancelled() {
        diagnostics.push(PortfolioDiagnostic::new(
            "portfolio_cancelled_before_plan",
            "portfolio was cancelled before backend scheduling",
        ));
        backend_runs.clear();
        PortfolioStopSummary::new(
            PortfolioStopReason::Cancelled,
            "portfolio cancelled before scheduling",
        )
    } else if backend_runs.is_empty() {
        diagnostics.push(PortfolioDiagnostic::new(
            "no_schedulable_profile",
            "no backend run was available for this ATP portfolio",
        ));
        PortfolioStopSummary::new(
            PortfolioStopReason::NoSchedulableProfile,
            "no backend run was scheduled",
        )
    } else if let Some(max_runs) =
        max_scheduled_runs(input.obligation_budget, input.scheduler_budget)
        && backend_runs.len() > max_runs
    {
        diagnostics.push(PortfolioDiagnostic::new(
            "portfolio_budget_exhausted",
            format!(
                "portfolio requested {} backend runs but budget permits {max_runs}",
                backend_runs.len()
            ),
        ));
        backend_runs.clear();
        PortfolioStopSummary::new(
            PortfolioStopReason::BudgetExhausted,
            "backend run budget exhausted before scheduling",
        )
    } else {
        PortfolioStopSummary::new(PortfolioStopReason::NotStopped, "backend runs scheduled")
    };

    let plan_hash = hash_plan(PlanHashInput {
        portfolio_id: &input.portfolio_id,
        vc_hash: input.vc_hash,
        problem: &input.atp_problem,
        backend_runs: &backend_runs,
        obligation_budget: input.obligation_budget,
        scheduler_budget: input.scheduler_budget,
        policy_constraints: input.policy_constraints,
        stop_summary: &stop_summary,
    });

    Ok(PortfolioPlan {
        portfolio_id: input.portfolio_id,
        vc_id: input.atp_problem.vc_id(),
        vc_hash: input.vc_hash,
        atp_problem_id: input.atp_problem.problem_id(),
        target_binding: input.atp_problem.target_binding().clone(),
        backend_runs,
        plan_hash,
        stop_summary,
        diagnostics,
        policy_constraints: input.policy_constraints,
        cancellation: input.cancellation,
    })
}

pub fn collect_portfolio_results(
    plan: PortfolioPlan,
    backend_results: Vec<BackendRunResult>,
) -> Result<PortfolioEvidenceSet, PortfolioError> {
    let stopped_before_collection = plan.stop_summary.reason() != PortfolioStopReason::NotStopped;
    let cancelled = plan.cancellation.is_cancelled()
        || plan.stop_summary.reason() == PortfolioStopReason::Cancelled;
    let normalized_results = normalize_backend_results(&plan, backend_results, cancelled)?;
    if stopped_before_collection {
        let diagnostics = plan.diagnostics.clone();
        return Ok(evidence_set(
            &plan,
            normalized_results,
            Vec::new(),
            plan.stop_summary.clone(),
            diagnostics,
        ));
    }

    if cancelled {
        let diagnostics = vec![PortfolioDiagnostic::new(
            "portfolio_cancelled_before_collection",
            "portfolio cancellation suppressed candidate handoff",
        )];
        return Ok(evidence_set(
            &plan,
            normalized_results,
            Vec::new(),
            PortfolioStopSummary::new(
                PortfolioStopReason::Cancelled,
                "portfolio cancelled before candidate handoff",
            ),
            diagnostics,
        ));
    }

    let candidates = collect_candidates(&plan, &normalized_results)?;
    Ok(evidence_set(
        &plan,
        normalized_results,
        candidates,
        PortfolioStopSummary::new(
            PortfolioStopReason::AllRunsCollected,
            "all scheduled backend runs were collected",
        ),
        Vec::new(),
    ))
}

fn validate_backend_runs(
    problem: &AtpProblem,
    backend_runs: &mut [BackendRunInput],
) -> Result<(), PortfolioError> {
    let mut seen = BTreeSet::new();
    for run in backend_runs {
        let run_id = run.run_id().as_str().to_owned();
        if !seen.insert(run_id.clone()) {
            return Err(PortfolioError::DuplicateBackendRun { run_id });
        }
        if run.encoded_problem().problem_id() != problem.problem_id() {
            return Err(PortfolioError::BackendRunProblemMismatch { run_id });
        }
        if run.encoded_problem().target_binding() != problem.target_binding() {
            return Err(PortfolioError::BackendRunTargetMismatch { run_id });
        }
        if run.encoded_problem().expected_result() != problem.expected_result() {
            return Err(PortfolioError::BackendRunExpectedResultMismatch { run_id });
        }
        if run.profile().concrete_format() != run.encoded_problem().concrete_format()
            || !problem
                .logic_profile()
                .concrete_formats()
                .contains(&run.encoded_problem().concrete_format())
        {
            return Err(PortfolioError::BackendRunFormatMismatch { run_id });
        }
    }
    Ok(())
}

fn normalize_backend_results(
    plan: &PortfolioPlan,
    backend_results: Vec<BackendRunResult>,
    allow_missing: bool,
) -> Result<Vec<BackendRunResult>, PortfolioError> {
    let mut by_run_id = BTreeMap::new();
    for result in backend_results {
        let run_id = result.run_id().as_str().to_owned();
        if by_run_id.insert(run_id.clone(), result).is_some() {
            return Err(PortfolioError::DuplicateBackendResult { run_id });
        }
    }

    let mut normalized = Vec::new();
    for run in &plan.backend_runs {
        let Some(result) = by_run_id.remove(run.run_id().as_str()) else {
            if allow_missing {
                continue;
            }
            return Err(PortfolioError::MissingBackendResult {
                run_id: run.run_id().as_str().to_owned(),
            });
        };
        validate_result_matches_run(run, &result)?;
        normalized.push(result);
    }

    if let Some((run_id, _)) = by_run_id.into_iter().next() {
        return Err(PortfolioError::UnknownBackendResult { run_id });
    }

    Ok(normalized)
}

fn validate_result_matches_run(
    run: &BackendRunInput,
    result: &BackendRunResult,
) -> Result<(), PortfolioError> {
    let matches = result.encoded_problem().problem_id() == run.encoded_problem().problem_id()
        && result.encoded_problem().input_hash() == run.encoded_problem().input_hash()
        && result.encoded_problem().metadata_hash() == run.encoded_problem().metadata_hash()
        && result.command_fingerprint() == backend_run_command_fingerprint(run)
        && result.profile_id() == run.profile().profile_id()
        && result.backend_kind() == run.profile().backend_kind();
    if matches {
        Ok(())
    } else {
        Err(PortfolioError::BackendResultMetadataMismatch {
            run_id: run.run_id().as_str().to_owned(),
        })
    }
}

fn collect_candidates(
    plan: &PortfolioPlan,
    backend_results: &[BackendRunResult],
) -> Result<Vec<PortfolioCandidate>, PortfolioError> {
    let mut candidates = Vec::new();
    for result in backend_results {
        match result.status() {
            BackendRunStatus::Proved => {
                let Some(candidate) = result.candidate_evidence() else {
                    continue;
                };
                candidates.push(formula_candidate(plan, result, candidate)?);
            }
            BackendRunStatus::Counterexample => {
                if result.counterexample().is_some() {
                    candidates.push(counterexample_candidate(plan, result)?);
                }
            }
            BackendRunStatus::Timeout
            | BackendRunStatus::Unknown
            | BackendRunStatus::Error
            | BackendRunStatus::Cancelled => {}
        }
    }
    candidates.sort_by(compare_candidate);
    Ok(candidates)
}

fn formula_candidate(
    plan: &PortfolioPlan,
    result: &BackendRunResult,
    candidate: &BackendCandidateEvidence,
) -> Result<PortfolioCandidate, PortfolioError> {
    if !candidate.payload().is_formula_substitution_candidate() {
        return Err(PortfolioError::CandidateMetadataMismatch {
            run_id: result.run_id().as_str().to_owned(),
            reason: "unsupported_candidate_payload",
        });
    }
    if candidate.target_binding() != result.encoded_problem().target_binding() {
        return Err(PortfolioError::CandidateMetadataMismatch {
            run_id: result.run_id().as_str().to_owned(),
            reason: "candidate_target_mismatch",
        });
    }
    if candidate.encoded_problem_hash() != result.encoded_problem().input_hash() {
        return Err(PortfolioError::CandidateMetadataMismatch {
            run_id: result.run_id().as_str().to_owned(),
            reason: "candidate_input_hash_mismatch",
        });
    }
    if candidate.provenance_hash() != result.encoded_problem().provenance_hash() {
        return Err(PortfolioError::CandidateMetadataMismatch {
            run_id: result.run_id().as_str().to_owned(),
            reason: "candidate_provenance_mismatch",
        });
    }
    if candidate.formula_label_refs() != result.encoded_problem().formula_labels() {
        return Err(PortfolioError::CandidateMetadataMismatch {
            run_id: result.run_id().as_str().to_owned(),
            reason: "candidate_label_mismatch",
        });
    }
    if candidate.symbol_binding_refs() != result.encoded_problem().symbol_bindings() {
        return Err(PortfolioError::CandidateMetadataMismatch {
            run_id: result.run_id().as_str().to_owned(),
            reason: "candidate_symbol_mismatch",
        });
    }

    let backend_profile_priority = planned_run_priority(plan, result.run_id())?;
    let (evidence_format, payload_hash) = candidate_payload_hash(candidate.payload());
    let candidate_id = PortfolioCandidateId(format!(
        "{}:{}",
        result.run_id().as_str(),
        candidate.candidate_id()
    ));
    let candidate_hash = hash_candidate(CandidateHashInput {
        portfolio_id: plan.portfolio_id.as_str(),
        source_run_id: result.run_id().as_str(),
        backend_profile_id: result.profile_id().as_str(),
        candidate_id: candidate_id.as_str(),
        candidate_kind: PortfolioCandidateKind::FormulaSubstitution,
        evidence_format,
        payload_hash,
        target_binding: candidate.target_binding(),
        encoded_problem_hash: candidate.encoded_problem_hash(),
        provenance_hash: candidate.provenance_hash(),
        observed_result: result.observed_result(),
    });

    Ok(PortfolioCandidate {
        candidate_id,
        source_run_id: result.run_id().clone(),
        backend_profile_id: result.profile_id().as_str().to_owned(),
        backend_profile_priority,
        encoded_problem_hash: candidate.encoded_problem_hash(),
        target_binding: candidate.target_binding().clone(),
        candidate_kind: PortfolioCandidateKind::FormulaSubstitution,
        evidence_format,
        evidence_payload_hash: Some(payload_hash),
        observed_result: result.observed_result(),
        provenance_hash: candidate.provenance_hash(),
        candidate_hash,
        diagnostics: diagnostics_from_backend(result.diagnostics()),
    })
}

fn counterexample_candidate(
    plan: &PortfolioPlan,
    result: &BackendRunResult,
) -> Result<PortfolioCandidate, PortfolioError> {
    let backend_profile_priority = planned_run_priority(plan, result.run_id())?;
    let payload_hash = result.counterexample().map_or_else(
        || hash_payload("counterexample-missing", &[]),
        |counterexample| hash_payload("counterexample", counterexample.payload()),
    );
    let candidate_id = PortfolioCandidateId(format!("{}:counterexample", result.run_id().as_str()));
    let candidate_hash = hash_candidate(CandidateHashInput {
        portfolio_id: plan.portfolio_id.as_str(),
        source_run_id: result.run_id().as_str(),
        backend_profile_id: result.profile_id().as_str(),
        candidate_id: candidate_id.as_str(),
        candidate_kind: PortfolioCandidateKind::Counterexample,
        evidence_format: PortfolioEvidenceFormat::Counterexample,
        payload_hash,
        target_binding: result.encoded_problem().target_binding(),
        encoded_problem_hash: result.encoded_problem().input_hash(),
        provenance_hash: result.encoded_problem().provenance_hash(),
        observed_result: result.observed_result(),
    });

    Ok(PortfolioCandidate {
        candidate_id,
        source_run_id: result.run_id().clone(),
        backend_profile_id: result.profile_id().as_str().to_owned(),
        backend_profile_priority,
        encoded_problem_hash: result.encoded_problem().input_hash(),
        target_binding: result.encoded_problem().target_binding().clone(),
        candidate_kind: PortfolioCandidateKind::Counterexample,
        evidence_format: PortfolioEvidenceFormat::Counterexample,
        evidence_payload_hash: Some(payload_hash),
        observed_result: result.observed_result(),
        provenance_hash: result.encoded_problem().provenance_hash(),
        candidate_hash,
        diagnostics: diagnostics_from_backend(result.diagnostics()),
    })
}

fn evidence_set(
    plan: &PortfolioPlan,
    backend_results: Vec<BackendRunResult>,
    candidates: Vec<PortfolioCandidate>,
    stop_summary: PortfolioStopSummary,
    diagnostics: Vec<PortfolioDiagnostic>,
) -> PortfolioEvidenceSet {
    let evidence_set_hash = hash_evidence_set(plan, &backend_results, &candidates, &stop_summary);
    PortfolioEvidenceSet {
        portfolio_id: plan.portfolio_id.clone(),
        vc_id: plan.vc_id,
        vc_hash: plan.vc_hash,
        atp_problem_id: plan.atp_problem_id,
        plan_hash: plan.plan_hash,
        evidence_set_hash,
        backend_results,
        candidates,
        stop_summary,
        diagnostics,
    }
}

fn max_scheduled_runs(
    obligation_budget: PortfolioBudget,
    scheduler_budget: PortfolioBudget,
) -> Option<usize> {
    match (
        obligation_budget.max_scheduled_runs(),
        scheduler_budget.max_scheduled_runs(),
    ) {
        (Some(left), Some(right)) => Some(left.min(right)),
        (Some(left), None) | (None, Some(left)) => Some(left),
        (None, None) => None,
    }
}

fn compare_backend_run(left: &BackendRunInput, right: &BackendRunInput) -> Ordering {
    left.profile()
        .deterministic_priority()
        .cmp(&right.profile().deterministic_priority())
        .then_with(|| {
            left.profile()
                .profile_id()
                .as_str()
                .cmp(right.profile().profile_id().as_str())
        })
        .then_with(|| {
            left.encoded_problem()
                .input_hash()
                .as_bytes()
                .cmp(right.encoded_problem().input_hash().as_bytes())
        })
        .then_with(|| left.run_id().as_str().cmp(right.run_id().as_str()))
}

fn compare_candidate(left: &PortfolioCandidate, right: &PortfolioCandidate) -> Ordering {
    candidate_kind_rank(left.candidate_kind())
        .cmp(&candidate_kind_rank(right.candidate_kind()))
        .then_with(|| {
            left.backend_profile_priority
                .cmp(&right.backend_profile_priority)
        })
        .then_with(|| {
            evidence_format_rank(left.evidence_format())
                .cmp(&evidence_format_rank(right.evidence_format()))
        })
        .then_with(|| {
            left.encoded_problem_hash()
                .as_bytes()
                .cmp(right.encoded_problem_hash().as_bytes())
        })
        .then_with(|| {
            left.candidate_hash()
                .as_bytes()
                .cmp(right.candidate_hash().as_bytes())
        })
        .then_with(|| left.backend_profile_id().cmp(right.backend_profile_id()))
        .then_with(|| {
            left.source_run_id()
                .as_str()
                .cmp(right.source_run_id().as_str())
        })
}

fn candidate_kind_rank(kind: PortfolioCandidateKind) -> u8 {
    match kind {
        PortfolioCandidateKind::FormulaSubstitution => 0,
        PortfolioCandidateKind::ExternallyAttested => 1,
        PortfolioCandidateKind::Counterexample => 2,
    }
}

fn evidence_format_rank(format: PortfolioEvidenceFormat) -> u8 {
    match format {
        PortfolioEvidenceFormat::FormulaSubstitutionBytes => 0,
        PortfolioEvidenceFormat::FormulaSubstitutionRef => 1,
        PortfolioEvidenceFormat::ExternallyAttested => 2,
        PortfolioEvidenceFormat::Counterexample => 3,
    }
}

fn diagnostics_from_backend(diagnostics: &[BackendDiagnostic]) -> Vec<PortfolioDiagnostic> {
    diagnostics
        .iter()
        .map(|diagnostic| PortfolioDiagnostic::new(diagnostic.key(), diagnostic.message()))
        .collect()
}

fn candidate_payload_hash(payload: &BackendCandidatePayload) -> (PortfolioEvidenceFormat, Hash) {
    match payload {
        BackendCandidatePayload::FormulaSubstitutionBytes(bytes) => (
            PortfolioEvidenceFormat::FormulaSubstitutionBytes,
            hash_payload("formula-substitution-bytes", bytes),
        ),
        BackendCandidatePayload::FormulaSubstitutionRef(reference) => (
            PortfolioEvidenceFormat::FormulaSubstitutionRef,
            hash_payload("formula-substitution-ref", reference.as_bytes()),
        ),
        _ => (
            PortfolioEvidenceFormat::ExternallyAttested,
            hash_payload("unsupported-backend-material", &[]),
        ),
    }
}

fn planned_run_priority(
    plan: &PortfolioPlan,
    run_id: &BackendRunId,
) -> Result<u32, PortfolioError> {
    plan.backend_runs
        .iter()
        .find(|run| run.run_id() == run_id)
        .map(|run| run.profile().deterministic_priority())
        .ok_or_else(|| PortfolioError::UnknownBackendResult {
            run_id: run_id.as_str().to_owned(),
        })
}

struct CandidateHashInput<'a> {
    portfolio_id: &'a str,
    source_run_id: &'a str,
    backend_profile_id: &'a str,
    candidate_id: &'a str,
    candidate_kind: PortfolioCandidateKind,
    evidence_format: PortfolioEvidenceFormat,
    payload_hash: Hash,
    target_binding: &'a AtpTargetBinding,
    encoded_problem_hash: Hash,
    provenance_hash: Hash,
    observed_result: Option<BackendObservedResult>,
}

fn hash_candidate(input: CandidateHashInput<'_>) -> Hash {
    let mut hash = StableHasher::new(CANDIDATE_HASH_DOMAIN);
    hash.field_str("portfolio-id", input.portfolio_id);
    hash.field_str("source-run-id", input.source_run_id);
    hash.field_str("backend-profile-id", input.backend_profile_id);
    hash.field_str("candidate-id", input.candidate_id);
    hash.field_str("candidate-kind", candidate_kind_name(input.candidate_kind));
    hash.field_str(
        "evidence-format",
        evidence_format_name(input.evidence_format),
    );
    hash.field_bytes("payload-hash", input.payload_hash.as_bytes());
    hash.field_bytes(
        "target-fingerprint",
        input.target_binding.fingerprint().digest(),
    );
    hash.field_str(
        "target-producer",
        input.target_binding.producer_binding().as_str(),
    );
    hash.field_bytes(
        "encoded-problem-hash",
        input.encoded_problem_hash.as_bytes(),
    );
    hash.field_bytes("provenance-hash", input.provenance_hash.as_bytes());
    hash.field_str(
        "observed-result",
        input.observed_result.map_or("none", observed_result_name),
    );
    hash.finalize()
}

struct PlanHashInput<'a> {
    portfolio_id: &'a PortfolioId,
    vc_hash: Hash,
    problem: &'a AtpProblem,
    backend_runs: &'a [BackendRunInput],
    obligation_budget: PortfolioBudget,
    scheduler_budget: PortfolioBudget,
    policy_constraints: PortfolioPolicyConstraints,
    stop_summary: &'a PortfolioStopSummary,
}

fn hash_plan(input: PlanHashInput<'_>) -> Hash {
    let mut hash = StableHasher::new(PLAN_HASH_DOMAIN);
    hash.field_str("portfolio-id", input.portfolio_id.as_str());
    hash.field_u64("vc-id", input.problem.vc_id().index() as u64);
    hash.field_bytes("vc-hash", input.vc_hash.as_bytes());
    hash.field_bytes("problem-id", input.problem.problem_id().hash().as_bytes());
    hash.field_str("stop-reason", stop_reason_name(input.stop_summary.reason()));
    hash.field_str("stop-message", input.stop_summary.message());
    hash_budget(&mut hash, "obligation", input.obligation_budget);
    hash_budget(&mut hash, "scheduler", input.scheduler_budget);
    hash.field_bool(
        "record-externally-attested",
        input.policy_constraints.record_externally_attested(),
    );
    for run in input.backend_runs {
        hash_backend_run(&mut hash, run);
    }
    hash.finalize()
}

fn hash_evidence_set(
    plan: &PortfolioPlan,
    backend_results: &[BackendRunResult],
    candidates: &[PortfolioCandidate],
    stop_summary: &PortfolioStopSummary,
) -> Hash {
    let mut hash = StableHasher::new(EVIDENCE_SET_HASH_DOMAIN);
    hash.field_str("portfolio-id", plan.portfolio_id.as_str());
    hash.field_u64("vc-id", plan.vc_id.index() as u64);
    hash.field_bytes("vc-hash", plan.vc_hash.as_bytes());
    hash.field_bytes("plan-hash", plan.plan_hash.as_bytes());
    hash.field_str("stop-reason", stop_reason_name(stop_summary.reason()));
    hash.field_str("stop-message", stop_summary.message());
    let mut result_hashes = backend_results
        .iter()
        .map(hash_result_metadata)
        .collect::<Vec<_>>();
    result_hashes.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
    for result_hash in result_hashes {
        hash.field_bytes("backend-result", result_hash.as_bytes());
    }
    let mut candidate_hashes = candidates
        .iter()
        .map(PortfolioCandidate::candidate_hash)
        .collect::<Vec<_>>();
    candidate_hashes.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
    for candidate_hash in candidate_hashes {
        hash.field_bytes("candidate", candidate_hash.as_bytes());
    }
    hash.finalize()
}

fn hash_backend_run(hash: &mut StableHasher, run: &BackendRunInput) {
    hash.field_str("run-id", run.run_id().as_str());
    hash.field_str("backend-kind", run.profile().backend_kind().as_str());
    hash.field_str("profile-id", run.profile().profile_id().as_str());
    hash.field_str(
        "format",
        concrete_format_name(run.profile().concrete_format()),
    );
    hash.field_u64(
        "profile-priority",
        u64::from(run.profile().deterministic_priority()),
    );
    hash.field_bytes("input-hash", run.encoded_problem().input_hash().as_bytes());
    hash.field_bytes(
        "metadata-hash",
        run.encoded_problem().metadata_hash().as_bytes(),
    );
    hash.field_str(
        "command-executable",
        &run.command().semantic_executable_id(),
    );
    for arg in run.command().args() {
        hash.field_str("command-arg", arg);
    }
    for (key, value) in run.command().environment().vars() {
        hash.field_str("env-key", key);
        hash.field_str("env-value", value);
    }
    hash.field_str(
        "working-directory-policy",
        working_directory_policy_name(run.command().working_directory()),
    );
    hash.field_str("io-mode", io_mode_name(run.io_mode()));
    hash.field_u64(
        "wall-timeout-ms",
        duration_millis_u64(run.resource_limits().wall_timeout()),
    );
    hash.field_u64(
        "kill-grace-ms",
        duration_millis_u64(run.resource_limits().kill_grace()),
    );
    hash.field_u64("stdout-limit", run.resource_limits().stdout_bytes() as u64);
    hash.field_u64("stderr-limit", run.resource_limits().stderr_bytes() as u64);
    for (name, requirement) in run.resource_limits().platform_limits() {
        hash.field_str("platform-limit", name);
        hash.field_str(
            "platform-limit-requirement",
            limit_requirement_name(*requirement),
        );
    }
}

fn hash_result_metadata(result: &BackendRunResult) -> Hash {
    let mut hash = StableHasher::new(RESULT_HASH_DOMAIN);
    hash.field_str("run-id", result.run_id().as_str());
    hash.field_str("backend-kind", result.backend_kind().as_str());
    hash.field_str("profile-id", result.profile_id().as_str());
    hash.field_str("status", run_status_name(result.status()));
    hash.field_str(
        "observed-result",
        result
            .observed_result()
            .map_or("none", observed_result_name),
    );
    hash.field_bytes(
        "encoded-input-hash",
        result.encoded_problem().input_hash().as_bytes(),
    );
    hash.field_bytes(
        "encoded-metadata-hash",
        result.encoded_problem().metadata_hash().as_bytes(),
    );
    hash.field_bytes(
        "command-fingerprint",
        result.command_fingerprint().as_bytes(),
    );
    hash.field_bytes("stdout-hash", result.stdout().hash().as_bytes());
    hash.field_bytes("stderr-hash", result.stderr().hash().as_bytes());
    hash.field_str("termination", termination_name(result.termination()));
    for diagnostic in result.diagnostics() {
        hash.field_str("diagnostic-key", diagnostic.key());
        hash.field_str("diagnostic-message", diagnostic.message());
    }
    hash.finalize()
}

fn hash_budget(hash: &mut StableHasher, label: &str, budget: PortfolioBudget) {
    hash.field_bool(
        &format!("{label}-max-scheduled-present"),
        budget.max_scheduled_runs().is_some(),
    );
    hash.field_u64(
        &format!("{label}-max-scheduled"),
        budget.max_scheduled_runs().unwrap_or(usize::MAX) as u64,
    );
}

fn hash_payload(kind: &str, bytes: &[u8]) -> Hash {
    let mut hash = StableHasher::new(PAYLOAD_HASH_DOMAIN);
    hash.field_str("payload-kind", kind);
    hash.field_bytes("payload", bytes);
    hash.finalize()
}

struct StableHasher {
    lanes: [u64; 4],
    length: u64,
}

impl StableHasher {
    fn new(domain: &str) -> Self {
        let mut hasher = Self {
            lanes: [
                0x6d_69_7a_61_72_2d_61_74,
                0x70_2d_70_6f_72_74_66_6f,
                0x6c_69_6f_2d_68_61_73_68,
                0x2d_76_31_2d_63_61_6e_6f,
            ],
            length: 0,
        };
        hasher.field_str("domain", domain);
        hasher
    }

    fn field_str(&mut self, label: &str, value: &str) {
        self.field_bytes(label, value.as_bytes());
    }

    fn field_u64(&mut self, label: &str, value: u64) {
        self.field_bytes(label, &value.to_le_bytes());
    }

    fn field_bool(&mut self, label: &str, value: bool) {
        self.field_bytes(label, &[u8::from(value)]);
    }

    fn field_bytes(&mut self, label: &str, value: &[u8]) {
        self.feed_bytes(&(label.len() as u64).to_le_bytes());
        self.feed_bytes(label.as_bytes());
        self.feed_bytes(&(value.len() as u64).to_le_bytes());
        self.feed_bytes(value);
    }

    fn feed_bytes(&mut self, bytes: &[u8]) {
        for byte in bytes {
            let lane = self.length as usize % self.lanes.len();
            let mixed = self.length.rotate_left((lane as u32) + 7);
            self.lanes[lane] ^= u64::from(*byte)
                .wrapping_add(0x9e37_79b9_7f4a_7c15)
                .wrapping_add(mixed);
            self.lanes[lane] = self.lanes[lane]
                .rotate_left(9 + lane as u32)
                .wrapping_mul(0x1000_0000_01b3);
            self.length = self.length.wrapping_add(1);
        }
    }

    fn finalize(mut self) -> Hash {
        self.lanes[0] ^= self.length;
        self.lanes[1] ^= self.length.rotate_left(23);
        self.lanes[2] ^= self.lanes[0].rotate_left(5);
        self.lanes[3] ^= self.lanes[1].rotate_left(17);

        let mut bytes = [0_u8; Hash::BYTE_LEN];
        for (chunk, lane) in bytes.chunks_exact_mut(8).zip(self.lanes) {
            chunk.copy_from_slice(&lane.to_be_bytes());
        }
        Hash::from_bytes(bytes)
    }
}

fn reject_empty(field: &'static str, value: &str) -> Result<(), PortfolioError> {
    if value.trim().is_empty() {
        Err(PortfolioError::EmptyField { field })
    } else {
        Ok(())
    }
}

fn concrete_format_name(format: crate::problem::ConcreteFormat) -> &'static str {
    match format {
        crate::problem::ConcreteFormat::Tptp => "tptp",
        crate::problem::ConcreteFormat::SmtLib => "smtlib",
    }
}

fn io_mode_name(mode: BackendIoMode) -> &'static str {
    match mode {
        BackendIoMode::Stdin => "stdin",
        BackendIoMode::PrivateProblemFile => "private-problem-file",
    }
}

fn working_directory_policy_name(policy: &BackendWorkingDirectoryPolicy) -> &'static str {
    match policy {
        BackendWorkingDirectoryPolicy::Inherit => "inherit",
        BackendWorkingDirectoryPolicy::Directory(_) => "explicit-directory",
    }
}

fn limit_requirement_name(requirement: BackendLimitRequirement) -> &'static str {
    match requirement {
        BackendLimitRequirement::BestEffort => "best-effort",
        BackendLimitRequirement::Required => "required",
    }
}

fn candidate_kind_name(kind: PortfolioCandidateKind) -> &'static str {
    match kind {
        PortfolioCandidateKind::FormulaSubstitution => "formula-substitution",
        PortfolioCandidateKind::ExternallyAttested => "externally-attested",
        PortfolioCandidateKind::Counterexample => "counterexample",
    }
}

fn evidence_format_name(format: PortfolioEvidenceFormat) -> &'static str {
    match format {
        PortfolioEvidenceFormat::FormulaSubstitutionBytes => "formula-substitution-bytes",
        PortfolioEvidenceFormat::FormulaSubstitutionRef => "formula-substitution-ref",
        PortfolioEvidenceFormat::ExternallyAttested => "externally-attested",
        PortfolioEvidenceFormat::Counterexample => "counterexample",
    }
}

fn run_status_name(status: BackendRunStatus) -> &'static str {
    match status {
        BackendRunStatus::Proved => "proved",
        BackendRunStatus::Counterexample => "counterexample",
        BackendRunStatus::Timeout => "timeout",
        BackendRunStatus::Unknown => "unknown",
        BackendRunStatus::Error => "error",
        BackendRunStatus::Cancelled => "cancelled",
    }
}

fn observed_result_name(result: BackendObservedResult) -> &'static str {
    match result {
        BackendObservedResult::Unsat => "unsat",
        BackendObservedResult::Sat => "sat",
        BackendObservedResult::CounterSatisfiable => "counter-satisfiable",
        BackendObservedResult::Unknown => "unknown",
        BackendObservedResult::Malformed => "malformed",
        BackendObservedResult::Unsupported => "unsupported",
    }
}

fn termination_name(termination: crate::backend::BackendTermination) -> &'static str {
    match termination {
        crate::backend::BackendTermination::NotStarted => "not-started",
        crate::backend::BackendTermination::Exited => "exited",
        crate::backend::BackendTermination::Timeout => "timeout",
        crate::backend::BackendTermination::Cancelled => "cancelled",
        crate::backend::BackendTermination::SpawnFailure => "spawn-failure",
        crate::backend::BackendTermination::ProcessError => "process-error",
    }
}

fn stop_reason_name(reason: PortfolioStopReason) -> &'static str {
    match reason {
        PortfolioStopReason::NotStopped => "not-stopped",
        PortfolioStopReason::AllRunsCollected => "all-runs-collected",
        PortfolioStopReason::NoSchedulableProfile => "no-schedulable-profile",
        PortfolioStopReason::BudgetExhausted => "budget-exhausted",
        PortfolioStopReason::Cancelled => "cancelled",
    }
}

fn duration_millis_u64(duration: Duration) -> u64 {
    duration.as_millis().try_into().unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests;
