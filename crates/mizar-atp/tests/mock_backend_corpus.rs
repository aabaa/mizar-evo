#![cfg(unix)]

use mizar_atp::{
    backend::{
        BackendCancellationToken, BackendCandidateEvidence, BackendCandidatePayload,
        BackendCommand, BackendCounterexample, BackendIoMode, BackendKind, BackendObservation,
        BackendObservedResult, BackendProfile, BackendProfileId, BackendResourceLimits,
        BackendRunId, BackendRunInput, BackendRunResult, BackendRunStatus, EncodedBackendProblem,
        EncodedBackendProblemParts, run_backend,
    },
    portfolio::{
        PortfolioBudget, PortfolioCandidateKind, PortfolioEvidenceFormat, PortfolioId,
        PortfolioInput, PortfolioInputParts, PortfolioPolicyConstraints, PortfolioStopReason,
        collect_portfolio_results, plan_portfolio,
    },
    problem::{
        AtpAtom, AtpDeclaration, AtpDeclarationId, AtpDeclarationKind, AtpFingerprint, AtpFormula,
        AtpFormulaId, AtpFormulaTree, AtpProblem, AtpProblemParts, AtpProvenance, AtpProvenanceId,
        AtpSourceBinding, AtpSourceRef, AtpSymbolMapEntry, AtpSymbolSource, AtpTargetBinding,
        AtpTypeContext, ConcreteFormat, EqualitySupport, ExpectedBackendResult, LogicFragment,
        LogicProfile, NativePropertySupport, QuantifierPolicy, SoftTypeStrategy,
    },
};
use mizar_session::Hash;
use mizar_vc::vc_ir::VcId;
use std::{
    collections::BTreeSet,
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

static NEXT_TEMP_DIR: AtomicU64 = AtomicU64::new(1);

const EXPECTATION_PATH: &str = "tests/property/atp_mock_backend_integration_001.expect.toml";
const FIXTURE_PATH: &str = "tests/property/atp_mock_backend_integration_001.fixture.toml";
const TRACE_MANIFEST_PATH: &str = "tests/coverage/spec_trace.toml";
const TASK20_SPEC_REF: &str = "spec.en.mizar_atp.advanced_semantics.mock_backend_integration";

#[test]
fn metadata_only_corpus_cases_drive_mock_backend_portfolio() {
    let sidecar = read_workspace_file(EXPECTATION_PATH);
    assert!(sidecar.contains(r#"stage = "advanced_semantics""#));
    assert!(sidecar.contains(r#"expected_outcome = "metadata_only""#));
    assert!(sidecar.contains(TASK20_SPEC_REF));
    assert!(!sidecar.contains("active_advanced_semantics"));

    let manifest = read_workspace_file(TRACE_MANIFEST_PATH);
    assert_manifest_links_task20_fixture(&manifest);

    let fixture = read_workspace_file(FIXTURE_PATH);
    assert!(fixture.contains("Metadata-only advanced_semantics corpus anchor"));
    assert!(!fixture.contains("candidate_payload"));
    assert!(!fixture.contains("BackendProofMethod"));
    assert!(!fixture.contains("ResolutionTrace"));
    assert!(!fixture.contains("MiniSAT"));

    let fixture_cases = parse_fixture_cases(&fixture);
    assert_eq!(
        fixture_cases,
        vec![
            FixtureCase::FormulaSubstitution,
            FixtureCase::Counterexample,
            FixtureCase::Unknown,
        ]
    );

    let problem = minimal_problem();
    let temp = TestDir::new();
    let runs = fixture_cases
        .iter()
        .map(|case| run_input_for_case(&temp, &problem, *case))
        .collect::<Vec<_>>();
    let results = runs
        .iter()
        .zip(fixture_cases.iter())
        .map(|(run, case)| run_mock_case(run.clone(), *case))
        .collect::<Vec<_>>();

    let evidence = collect_from_runs(problem.clone(), runs.clone(), results.clone());
    let mut reversed_results = results;
    reversed_results.reverse();
    let reversed_evidence = collect_from_runs(problem, runs, reversed_results);

    assert_eq!(evidence.backend_results().len(), 3);
    assert_eq!(
        evidence.stop_summary().reason(),
        PortfolioStopReason::AllRunsCollected
    );
    assert_eq!(evidence.diagnostics(), &[]);
    assert_eq!(evidence.candidates().len(), 2);
    assert_eq!(
        evidence
            .candidates()
            .iter()
            .map(|candidate| (
                candidate.candidate_kind(),
                candidate.evidence_format(),
                candidate.observed_result()
            ))
            .collect::<Vec<_>>(),
        vec![
            (
                PortfolioCandidateKind::FormulaSubstitution,
                PortfolioEvidenceFormat::FormulaSubstitutionBytes,
                Some(BackendObservedResult::Unsat),
            ),
            (
                PortfolioCandidateKind::Counterexample,
                PortfolioEvidenceFormat::Counterexample,
                Some(BackendObservedResult::Sat),
            ),
        ]
    );
    assert!(evidence.candidates()[0].evidence_payload_hash().is_some());
    assert!(evidence.candidates()[1].evidence_payload_hash().is_some());
    assert!(evidence.backend_results().iter().any(|result| {
        result.status() == BackendRunStatus::Unknown
            && result.observed_result() == Some(BackendObservedResult::Unknown)
            && result.candidate_evidence().is_none()
            && result.counterexample().is_none()
    }));

    assert_eq!(
        candidate_ids(&evidence),
        candidate_ids(&reversed_evidence),
        "candidate handoff must be independent of backend completion order"
    );
    assert_eq!(
        evidence.evidence_set_hash(),
        reversed_evidence.evidence_set_hash(),
        "same backend result set must hash identically under shuffled completion order"
    );
}

fn collect_from_runs(
    problem: AtpProblem,
    runs: Vec<BackendRunInput>,
    results: Vec<BackendRunResult>,
) -> mizar_atp::portfolio::PortfolioEvidenceSet {
    let plan = plan_portfolio(PortfolioInput::new(PortfolioInputParts {
        portfolio_id: PortfolioId::new("task20-mock-corpus").expect("portfolio id"),
        vc_hash: hash_with_seed(20),
        atp_problem: problem,
        backend_runs: runs,
        obligation_budget: PortfolioBudget::unbounded(),
        scheduler_budget: PortfolioBudget::unbounded(),
        policy_constraints: PortfolioPolicyConstraints::new(),
        cancellation: BackendCancellationToken::new(),
    }))
    .expect("portfolio plan");
    collect_portfolio_results(plan, results).expect("portfolio results")
}

fn run_mock_case(run: BackendRunInput, case: FixtureCase) -> BackendRunResult {
    let result = run_backend(run);
    assert_eq!(result.status(), BackendRunStatus::Unknown);
    match case {
        FixtureCase::FormulaSubstitution => {
            let candidate = BackendCandidateEvidence::new(
                format!("candidate-{}", case.name()),
                BackendCandidatePayload::FormulaSubstitutionBytes(
                    format!(
                        "formula-substitution-candidate:{}",
                        result.run_id().as_str()
                    )
                    .into_bytes(),
                ),
                result.encoded_problem().target_binding().clone(),
                result.encoded_problem().input_hash(),
                result.encoded_problem().provenance_hash(),
                result.encoded_problem().formula_labels().to_vec(),
                result.encoded_problem().symbol_bindings().to_vec(),
            )
            .expect("formula/substitution candidate");
            mizar_atp::backend::classify_backend_observation(
                result,
                BackendObservation::new(BackendObservedResult::Unsat)
                    .with_candidate_evidence(candidate),
            )
        }
        FixtureCase::Counterexample => {
            let counterexample =
                BackendCounterexample::new(format!("model:{}", case.name()).into_bytes(), true)
                    .expect("counterexample");
            mizar_atp::backend::classify_backend_observation(
                result,
                BackendObservation::new(BackendObservedResult::Sat)
                    .with_counterexample(counterexample),
            )
        }
        FixtureCase::Unknown => mizar_atp::backend::classify_backend_observation(
            result,
            BackendObservation::new(BackendObservedResult::Unknown),
        ),
    }
}

fn run_input_for_case(temp: &TestDir, problem: &AtpProblem, case: FixtureCase) -> BackendRunInput {
    let script = temp.executable(
        &format!("backend-{}", case.name()),
        &format!(
            "#!/bin/sh\ncat >/dev/null\nprintf '%s\\n' '{}'\n",
            case.name()
        ),
    );
    let command = BackendCommand::new(script, Vec::new())
        .expect("command")
        .with_semantic_executable_id("mizar-atp-task20-mock-backend")
        .expect("semantic executable id");
    let profile = BackendProfile::new(
        BackendProfileId::new(format!("mock-profile-{}", case.name())).expect("profile id"),
        BackendKind::new("task20-mock-backend").expect("backend kind"),
        ConcreteFormat::Tptp,
    )
    .with_deterministic_priority(case.priority());

    BackendRunInput::new(
        BackendRunId::new(format!("run-{}", case.name())).expect("run id"),
        encoded_problem(problem, case),
        profile,
        command,
        BackendResourceLimits::new().with_wall_timeout(Duration::from_secs(2)),
        BackendIoMode::Stdin,
        BackendCancellationToken::new(),
    )
    .with_random_seed(u64::from(20 + case.priority()))
}

fn encoded_problem(problem: &AtpProblem, case: FixtureCase) -> EncodedBackendProblem {
    EncodedBackendProblem::new(EncodedBackendProblemParts {
        problem_id: problem.problem_id(),
        target_binding: problem.target_binding().clone(),
        expected_result: problem.expected_result(),
        concrete_format: ConcreteFormat::Tptp,
        logic_profile_name: problem.logic_profile().name().as_str().to_owned(),
        logic_fragment: "Fof".to_owned(),
        input_text: format!("fof(task20_{}, conjecture, p).\n", case.name()).into_bytes(),
        formula_labels: vec!["ax_p".to_owned(), "conjecture_p".to_owned()],
        symbol_bindings: vec!["P".to_owned()],
        provenance_hash: hash_with_seed(90 + case.priority() as u8),
    })
    .expect("encoded backend problem")
}

fn minimal_problem() -> AtpProblem {
    AtpProblem::try_new(AtpProblemParts {
        vc_id: VcId::new(20),
        target_binding: target_binding("task20:vc"),
        logic_profile: LogicProfile::try_new(
            "task20-fof",
            LogicFragment::Fof,
            EqualitySupport::Unsupported,
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
            AtpSymbolSource::MizarSymbol(AtpSourceBinding::new("predicate:P")),
        )],
        provenance: vec![
            AtpProvenance::new(
                AtpProvenanceId::new(1),
                AtpSourceRef::LocalHypothesis(AtpSourceBinding::new("decl:P")),
                "decl",
            ),
            AtpProvenance::new(
                AtpProvenanceId::new(2),
                AtpSourceRef::CitedPremise(AtpSourceBinding::new("premise:P")),
                "premise",
            ),
            AtpProvenance::new(
                AtpProvenanceId::new(3),
                AtpSourceRef::GeneratedVcFact(AtpSourceBinding::new("goal:P")),
                "goal",
            ),
        ],
        diagnostics: Vec::new(),
    })
    .expect("minimal ATP problem")
}

fn target_binding(source: &str) -> AtpTargetBinding {
    AtpTargetBinding::new(
        AtpFingerprint::new(20, source.as_bytes().to_vec()).expect("fingerprint"),
        AtpSourceBinding::new(source),
    )
    .expect("target binding")
}

fn parse_fixture_cases(fixture: &str) -> Vec<FixtureCase> {
    let mut cases = Vec::new();
    let mut in_case = false;
    let mut current_name: Option<String> = None;

    for line in fixture.lines().map(str::trim) {
        if line == "[[case]]" {
            if in_case {
                push_case_name(&mut cases, current_name.take());
            }
            in_case = true;
            continue;
        }

        if line.starts_with('[') {
            if in_case {
                push_case_name(&mut cases, current_name.take());
            }
            in_case = false;
            continue;
        }

        if in_case && let Some(name) = parse_string_assignment(line, "name") {
            assert!(
                current_name.replace(name).is_none(),
                "fixture case must not repeat `name`"
            );
        }
    }

    if in_case {
        push_case_name(&mut cases, current_name);
    }

    cases
}

fn push_case_name(cases: &mut Vec<FixtureCase>, name: Option<String>) {
    let name = name.expect("fixture case is missing `name`");
    let case = FixtureCase::from_name(&name)
        .unwrap_or_else(|| panic!("unknown task-20 fixture case `{name}`"));
    cases.push(case);
}

fn assert_manifest_links_task20_fixture(manifest: &str) {
    let task20_record = requirement_record(manifest, TASK20_SPEC_REF)
        .expect("trace manifest must contain the task-20 requirement");
    assert!(task20_record.contains(r#"stage = "advanced_semantics""#));
    assert!(task20_record.contains(r#"status = "covered""#));
    assert!(task20_record.contains(r#"coverage = "property""#));
    assert!(
        task20_record.contains(&format!(r#""{EXPECTATION_PATH}""#)),
        "trace manifest task-20 requirement must list the metadata-only sidecar"
    );
}

fn requirement_record<'a>(manifest: &'a str, requirement_id: &str) -> Option<&'a str> {
    manifest
        .split("[[requirement]]")
        .find(|record| record.contains(&format!(r#"id = "{requirement_id}""#)))
}

fn parse_string_assignment(line: &str, key: &str) -> Option<String> {
    let prefix = format!(r#"{key} = ""#);
    line.strip_prefix(&prefix)
        .and_then(|value| value.strip_suffix('"'))
        .map(str::to_owned)
}

fn candidate_ids(
    evidence: &mizar_atp::portfolio::PortfolioEvidenceSet,
) -> Vec<(String, BackendObservedResult)> {
    evidence
        .candidates()
        .iter()
        .map(|candidate| {
            (
                candidate.candidate_id().as_str().to_owned(),
                candidate
                    .observed_result()
                    .expect("candidate observed result"),
            )
        })
        .collect()
}

fn read_workspace_file(relative: &str) -> String {
    let path = workspace_root().join(relative);
    fs::read_to_string(&path).unwrap_or_else(|error| panic!("failed to read {relative}: {error}"))
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf()
}

fn hash_with_seed(seed: u8) -> Hash {
    Hash::from_bytes([seed; Hash::BYTE_LEN])
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FixtureCase {
    FormulaSubstitution,
    Counterexample,
    Unknown,
}

impl FixtureCase {
    const fn name(self) -> &'static str {
        match self {
            Self::FormulaSubstitution => "formula-substitution-candidate",
            Self::Counterexample => "counterexample-record",
            Self::Unknown => "unknown-open-result",
        }
    }

    const fn priority(self) -> u32 {
        match self {
            Self::FormulaSubstitution => 0,
            Self::Counterexample => 1,
            Self::Unknown => 2,
        }
    }

    fn from_name(name: &str) -> Option<Self> {
        match name {
            "formula-substitution-candidate" => Some(Self::FormulaSubstitution),
            "counterexample-record" => Some(Self::Counterexample),
            "unknown-open-result" => Some(Self::Unknown),
            _ => None,
        }
    }
}

struct TestDir {
    path: PathBuf,
}

impl TestDir {
    fn new() -> Self {
        let id = NEXT_TEMP_DIR.fetch_add(1, Ordering::SeqCst);
        let path = std::env::temp_dir().join(format!(
            "mizar-atp-task20-mock-backend-{}-{id}",
            std::process::id()
        ));
        if path.exists() {
            fs::remove_dir_all(&path).expect("remove stale temp dir");
        }
        fs::create_dir_all(&path).expect("create temp dir");
        Self { path }
    }

    fn executable(&self, name: &str, script: &str) -> PathBuf {
        let path = self.path.join(name);
        fs::write(&path, script).expect("write script");
        let mut permissions = fs::metadata(&path).expect("script metadata").permissions();
        permissions.set_mode(0o700);
        fs::set_permissions(&path, permissions).expect("set executable bit");
        path
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
