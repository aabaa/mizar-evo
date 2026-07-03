use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::diagnostic::{ValidationDiagnostic, ValidationSeverity};
use crate::expectation::{
    Architecture22Gate, Expectation, ExpectedOutcome, REQUIRED_SOUNDNESS_CASES, TestCaseId,
    architecture22_scenario_specs, parse_expectation_file, required_soundness_case_for,
    validate_expectation_path,
};
use crate::layout;
use crate::path_rules::{absolute_from, clean_relative_path};
use crate::traceability::{
    Architecture22MatrixReport, Architecture22ScenarioReport, CoverageEvidence, CoverageReport,
    CoverageShape, PassFailMix, RequirementCoverage, RequirementStatus, TraceManifest,
    parse_trace_manifest, validate_manifest,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveryConfig {
    pub workspace_root: PathBuf,
    pub tests_root: PathBuf,
    pub manifest_path: PathBuf,
    pub profile: TestProfile,
    pub validation_mode: ValidationMode,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum TestProfile {
    #[default]
    Fast,
    Full,
    Stress,
    FuzzRegression,
    SnapshotUpdate,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum ValidationMode {
    #[default]
    Metadata,
    Development,
    Release,
}

impl TestProfile {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Fast => "fast",
            Self::Full => "full",
            Self::Stress => "stress",
            Self::FuzzRegression => "fuzz_regression",
            Self::SnapshotUpdate => "snapshot_update",
        }
    }

    fn includes(self, profiles: &[String]) -> bool {
        if self == Self::Full {
            return true;
        }
        profiles
            .iter()
            .any(|profile| profile == self.as_str() || profile == self.hyphenated_str())
    }

    fn hyphenated_str(self) -> &'static str {
        match self {
            Self::FuzzRegression => "fuzz-regression",
            Self::SnapshotUpdate => "snapshot-update",
            _ => self.as_str(),
        }
    }
}

impl ValidationMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Metadata => "metadata",
            Self::Development => "development",
            Self::Release => "release",
        }
    }

    fn is_strict_layout(self) -> bool {
        matches!(self, Self::Development | Self::Release)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestPlan {
    pub cases: Vec<TestCase>,
    pub manifest: TraceManifest,
    pub coverage_report: CoverageReport,
    pub diagnostics: Vec<ValidationDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestCase {
    pub id: TestCaseId,
    pub source_path: PathBuf,
    pub expectation_path: PathBuf,
    pub expectation: Expectation,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum HarnessError {
    Infrastructure(String),
}

impl TestPlan {
    pub fn error_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == ValidationSeverity::Error)
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == ValidationSeverity::Warning)
            .count()
    }
}

pub fn build_test_plan(config: &DiscoveryConfig) -> Result<TestPlan, HarnessError> {
    let config = normalized_config(config)?;

    if !config.tests_root.is_dir() {
        return Err(HarnessError::Infrastructure(format!(
            "tests root `{}` is not a directory",
            config.tests_root.display()
        )));
    }

    let mut diagnostics = Vec::new();
    let manifest = match parse_trace_manifest(&config.manifest_path) {
        Ok(manifest) => manifest,
        Err(diagnostic) => {
            diagnostics.push(diagnostic);
            TraceManifest {
                requirements: Vec::new(),
            }
        }
    };
    diagnostics.extend(validate_manifest(
        &config.workspace_root,
        &config.manifest_path,
        &manifest,
    ));

    let discovered = layout::discover(&config.tests_root).map_err(|error| {
        HarnessError::Infrastructure(format!(
            "failed to discover tests under `{}`: {error}",
            config.tests_root.display()
        ))
    })?;
    diagnostics.extend(discovered.diagnostics);
    diagnostics.extend(validate_unknown_roots(&config)?);

    let mut all_cases = Vec::new();
    let mut cases = Vec::new();
    let mut id_paths: BTreeMap<TestCaseId, Vec<PathBuf>> = BTreeMap::new();
    let manifest_ids = manifest.requirement_ids();

    for sidecar in discovered.sidecars {
        let expectation = match parse_expectation_file(&sidecar) {
            Ok(expectation) => expectation,
            Err(diagnostic) => {
                diagnostics.push(diagnostic);
                continue;
            }
        };

        diagnostics.extend(validate_expectation_path(
            &sidecar,
            &expectation,
            &config.tests_root,
        ));

        for spec_ref in &expectation.spec_refs {
            if !manifest_ids.contains(spec_ref) {
                diagnostics.push(ValidationDiagnostic::error(
                    &sidecar,
                    "traceability",
                    "E-TRACE-UNKNOWN-SPEC-REF",
                    format!("trace.spec_ref.{}", spec_ref.0),
                    format!("unknown spec_ref `{}`", spec_ref.0),
                ));
            }
        }

        let source_path = sidecar
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .join(&expectation.source);
        id_paths
            .entry(expectation.id.clone())
            .or_default()
            .push(sidecar.clone());
        let case = TestCase {
            id: expectation.id.clone(),
            source_path,
            expectation_path: sidecar,
            expectation,
        };
        if config.profile.includes(&case.expectation.profiles) {
            cases.push(case.clone());
        }
        all_cases.push(case);
    }

    for (id, paths) in id_paths {
        if paths.len() > 1 {
            for path in paths {
                diagnostics.push(ValidationDiagnostic::error(
                    path,
                    "expectation",
                    "E-EXPECT-DUP-ID",
                    format!("expectation.id.{}", id.0),
                    format!("duplicate test id `{}`", id.0),
                ));
            }
        }
    }

    validate_manifest_test_links(
        &config.workspace_root,
        &config.manifest_path,
        &manifest,
        &all_cases,
        &mut diagnostics,
    );
    validate_obsolete_spec_refs(&manifest, &all_cases, &mut diagnostics);
    diagnostics.extend(validate_required_soundness_cases(
        &config.manifest_path,
        config.validation_mode,
        &manifest,
        &all_cases,
        &invalid_expectation_paths(&diagnostics),
    ));

    let base_invalid_expectation_paths = invalid_expectation_paths(&diagnostics);
    let base_coverage_evidence = coverage_evidence(
        &config.workspace_root,
        &manifest,
        &all_cases,
        &base_invalid_expectation_paths,
    );
    let base_coverage_report = manifest.coverage_report(
        &base_coverage_evidence,
        corpus_pass_fail_mix(&all_cases, &base_invalid_expectation_paths),
        architecture22_matrix_report(&all_cases, &base_invalid_expectation_paths),
    );
    diagnostics.extend(validate_stage_prerequisite_links(
        &config.workspace_root,
        &manifest,
        &all_cases,
        &base_invalid_expectation_paths,
        &base_coverage_report,
    ));

    let invalid_expectation_paths = invalid_expectation_paths(&diagnostics);
    let coverage_evidence = coverage_evidence(
        &config.workspace_root,
        &manifest,
        &all_cases,
        &invalid_expectation_paths,
    );
    let coverage_report = manifest.coverage_report(
        &coverage_evidence,
        corpus_pass_fail_mix(&all_cases, &invalid_expectation_paths),
        architecture22_matrix_report(&all_cases, &invalid_expectation_paths),
    );
    diagnostics.extend(validate_coverage_report(
        &config.manifest_path,
        config.validation_mode,
        &coverage_report,
    ));

    cases.sort_by(|left, right| left.expectation_path.cmp(&right.expectation_path));
    diagnostics.sort();

    Ok(TestPlan {
        cases,
        manifest,
        coverage_report,
        diagnostics,
    })
}

fn validate_unknown_roots(
    config: &DiscoveryConfig,
) -> Result<Vec<ValidationDiagnostic>, HarnessError> {
    if !config.validation_mode.is_strict_layout() {
        return Ok(Vec::new());
    }

    let unknown_roots = layout::unknown_roots(&config.tests_root).map_err(|error| {
        HarnessError::Infrastructure(format!(
            "failed to inspect test roots under `{}`: {error}",
            config.tests_root.display()
        ))
    })?;
    Ok(unknown_roots
        .into_iter()
        .map(|path| {
            let root_name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("<non-utf8>")
                .to_owned();
            ValidationDiagnostic::error(
                path,
                "layout",
                "E-LAYOUT-UNKNOWN-ROOT",
                format!("layout.root.{root_name}"),
                format!(
                    "unknown test root `{root_name}` is not allowed in `{}` validation mode",
                    config.validation_mode.as_str()
                ),
            )
        })
        .collect())
}

fn validate_manifest_test_links(
    workspace_root: &Path,
    manifest_path: &Path,
    manifest: &TraceManifest,
    cases: &[TestCase],
    diagnostics: &mut Vec<ValidationDiagnostic>,
) {
    let mut cases_by_rel_path = BTreeMap::new();
    for case in cases {
        if let Ok(rel_path) = case.expectation_path.strip_prefix(workspace_root) {
            cases_by_rel_path.insert(rel_path.to_path_buf(), case);
        }
    }

    for requirement in &manifest.requirements {
        let mut listed = BTreeSet::new();
        for test_path in &requirement.tests {
            if !listed.insert(test_path.clone()) {
                diagnostics.push(ValidationDiagnostic::error(
                    manifest_path,
                    "manifest",
                    "E-MANIFEST-DUP-TEST",
                    format!(
                        "manifest.tests.{}.{}",
                        requirement.id.0,
                        test_path.display()
                    ),
                    format!(
                        "requirement `{}` lists duplicate test `{}`",
                        requirement.id.0,
                        test_path.display()
                    ),
                ));
            }

            if !clean_relative_path(test_path) {
                continue;
            }

            if !workspace_root.join(test_path).is_file() {
                diagnostics.push(ValidationDiagnostic::error(
                    manifest_path,
                    "manifest",
                    "E-MANIFEST-MISSING-TEST",
                    format!(
                        "manifest.tests.{}.{}",
                        requirement.id.0,
                        test_path.display()
                    ),
                    format!(
                        "requirement `{}` lists missing test `{}`",
                        requirement.id.0,
                        test_path.display()
                    ),
                ));
                continue;
            }

            match cases_by_rel_path.get(test_path) {
                Some(case)
                    if case
                        .expectation
                        .spec_refs
                        .iter()
                        .any(|spec_ref| spec_ref == &requirement.id) => {}
                Some(_) => diagnostics.push(ValidationDiagnostic::error(
                    manifest_path,
                    "traceability",
                    "E-TRACE-MISSING-BACKREF",
                    format!("trace.backref.{}.{}", requirement.id.0, test_path.display()),
                    format!(
                        "test `{}` does not point back to requirement `{}`",
                        test_path.display(),
                        requirement.id.0
                    ),
                )),
                None => diagnostics.push(ValidationDiagnostic::error(
                    manifest_path,
                    "traceability",
                    "E-TRACE-UNPARSED-TEST",
                    format!("trace.test.{}.{}", requirement.id.0, test_path.display()),
                    format!(
                        "listed test `{}` was not discovered as a valid expectation",
                        test_path.display()
                    ),
                )),
            }
        }
    }
}

fn validate_obsolete_spec_refs(
    manifest: &TraceManifest,
    cases: &[TestCase],
    diagnostics: &mut Vec<ValidationDiagnostic>,
) {
    let requirements = manifest.by_id();
    for case in cases {
        for spec_ref in &case.expectation.spec_refs {
            let Some(requirement) = requirements.get(spec_ref) else {
                continue;
            };
            if requirement.status == RequirementStatus::Obsolete {
                diagnostics.push(ValidationDiagnostic::error(
                    &case.expectation_path,
                    "traceability",
                    "E-TRACE-OBSOLETE-SPEC-REF",
                    format!("trace.obsolete.{}", spec_ref.0),
                    format!("test points to obsolete requirement `{}`", spec_ref.0),
                ));
            }
        }
    }
}

fn validate_required_soundness_cases(
    manifest_path: &Path,
    validation_mode: ValidationMode,
    manifest: &TraceManifest,
    cases: &[TestCase],
    invalid_expectation_paths: &BTreeSet<PathBuf>,
) -> Vec<ValidationDiagnostic> {
    if !fail_soundness_bookkeeping_is_active(manifest, cases) {
        return Vec::new();
    }

    let present = cases
        .iter()
        .filter(|case| !invalid_expectation_paths.contains(&case.expectation_path))
        .filter_map(|case| required_soundness_case_for(&case.expectation))
        .map(|required_case| required_case.key)
        .collect::<BTreeSet<_>>();

    REQUIRED_SOUNDNESS_CASES
        .iter()
        .filter(|required_case| !present.contains(required_case.key))
        .map(|required_case| {
            let message = format!(
                "required fail/soundness case `{}` is not covered by expectation metadata",
                required_case.key
            );
            match validation_mode {
                ValidationMode::Metadata => ValidationDiagnostic::warning(
                    manifest_path,
                    "fail_soundness",
                    "W-SOUNDNESS-MISSING-CASE",
                    format!("fail_soundness.required_case.{}", required_case.key),
                    message,
                ),
                ValidationMode::Development | ValidationMode::Release => {
                    ValidationDiagnostic::error(
                        manifest_path,
                        "fail_soundness",
                        "E-SOUNDNESS-MISSING-CASE",
                        format!("fail_soundness.required_case.{}", required_case.key),
                        message,
                    )
                }
            }
        })
        .collect()
}

fn fail_soundness_bookkeeping_is_active(manifest: &TraceManifest, cases: &[TestCase]) -> bool {
    manifest.requirements.iter().any(|requirement| {
        requirement
            .source
            .to_string_lossy()
            .ends_with("doc/design/mizar-test/en/fail_soundness.md")
            || requirement.id.0.starts_with("spec.en.fail_soundness.")
            || requirement.id.0.starts_with("spec.en.soundness.")
    }) || cases
        .iter()
        .any(|case| required_soundness_case_for(&case.expectation).is_some())
}

fn validate_stage_prerequisite_links(
    workspace_root: &Path,
    manifest: &TraceManifest,
    cases: &[TestCase],
    invalid_expectation_paths: &BTreeSet<PathBuf>,
    report: &CoverageReport,
) -> Vec<ValidationDiagnostic> {
    let requirements = manifest.by_id();
    let mut diagnostics = Vec::new();
    for case in cases {
        if invalid_expectation_paths.contains(&case.expectation_path) {
            continue;
        }
        let Ok(relative_path) = case.expectation_path.strip_prefix(workspace_root) else {
            continue;
        };
        let relative_path = relative_path.to_path_buf();
        for spec_ref in &case.expectation.spec_refs {
            let Some(requirement) = requirements.get(spec_ref) else {
                continue;
            };
            if !requirement.tests.iter().any(|test| test == &relative_path) {
                continue;
            }
            if manifest.evidence_stage_can_credit(requirement, case.expectation.stage, report) {
                continue;
            }
            let missing_prerequisites = manifest.unsatisfied_prerequisites(requirement, report);
            if !missing_prerequisites.is_empty() {
                diagnostics.push(ValidationDiagnostic::error(
                    &case.expectation_path,
                    "traceability",
                    "E-TRACE-PREREQUISITE",
                    format!("trace.prerequisite.{}", spec_ref.0),
                    format!(
                        "test cannot credit requirement `{}` before prerequisite coverage is satisfied: {}",
                        spec_ref.0,
                        requirement_id_list(&missing_prerequisites)
                    ),
                ));
            } else {
                diagnostics.push(ValidationDiagnostic::error(
                    &case.expectation_path,
                    "traceability",
                    "E-TRACE-STAGE-MISMATCH",
                    format!("trace.stage.{}", spec_ref.0),
                    format!(
                        "test stage `{}` cannot credit requirement `{}` at stage `{}`",
                        case.expectation.stage.as_str(),
                        spec_ref.0,
                        requirement.stage.as_str()
                    ),
                ));
            }
        }
    }
    diagnostics
}

fn invalid_expectation_paths(diagnostics: &[ValidationDiagnostic]) -> BTreeSet<PathBuf> {
    diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic.severity == ValidationSeverity::Error
                && matches!(diagnostic.record_kind, "expectation" | "traceability")
        })
        .map(|diagnostic| diagnostic.path.clone())
        .collect()
}

fn coverage_evidence(
    workspace_root: &Path,
    manifest: &TraceManifest,
    cases: &[TestCase],
    invalid_expectation_paths: &BTreeSet<PathBuf>,
) -> Vec<CoverageEvidence> {
    let requirements = manifest.by_id();
    let mut evidence = Vec::new();
    for case in cases {
        if invalid_expectation_paths.contains(&case.expectation_path) {
            continue;
        }
        let Ok(relative_path) = case.expectation_path.strip_prefix(workspace_root) else {
            continue;
        };
        let relative_path = relative_path.to_path_buf();
        for spec_ref in &case.expectation.spec_refs {
            let Some(requirement) = requirements.get(spec_ref) else {
                continue;
            };
            if !requirement.tests.iter().any(|test| test == &relative_path) {
                continue;
            }
            evidence.push(CoverageEvidence {
                requirement_id: spec_ref.clone(),
                test_path: relative_path.clone(),
                stage: case.expectation.stage,
                kind: case.expectation.kind,
                expected_outcome: case.expectation.expected_outcome,
                has_diagnostic_evidence: has_diagnostic_evidence(&case.expectation),
                has_snapshot: case.expectation.snapshots.is_some(),
            });
        }
    }
    evidence.sort_by(|left, right| {
        left.requirement_id
            .cmp(&right.requirement_id)
            .then(left.test_path.cmp(&right.test_path))
    });
    evidence
}

fn has_diagnostic_evidence(expectation: &Expectation) -> bool {
    expectation.expected_outcome == ExpectedOutcome::Fail
        && (expectation.failure_category.is_some()
            || expectation.rejection_reason.is_some()
            || !expectation.diagnostic_codes.is_empty()
            || !expectation.diagnostic_payloads.is_empty()
            || expectation.stable_detail_key.is_some())
}

fn corpus_pass_fail_mix(
    cases: &[TestCase],
    invalid_expectation_paths: &BTreeSet<PathBuf>,
) -> PassFailMix {
    let pass = cases
        .iter()
        .filter(|case| !invalid_expectation_paths.contains(&case.expectation_path))
        .filter(|case| case.expectation.expected_outcome == ExpectedOutcome::Pass)
        .count();
    let fail = cases
        .iter()
        .filter(|case| !invalid_expectation_paths.contains(&case.expectation_path))
        .filter(|case| case.expectation.expected_outcome == ExpectedOutcome::Fail)
        .count();
    PassFailMix {
        pass,
        fail,
        total: pass + fail,
        target_pass_percent: 40,
        target_fail_percent: 60,
    }
}

fn architecture22_matrix_report(
    cases: &[TestCase],
    invalid_expectation_paths: &BTreeSet<PathBuf>,
) -> Architecture22MatrixReport {
    let mut by_scenario = architecture22_scenario_specs()
        .iter()
        .map(|scenario| {
            (
                scenario.id.to_owned(),
                Architecture22ScenarioReport {
                    scenario_id: scenario.id.to_owned(),
                    equivalence_class: scenario.equivalence_class.to_owned(),
                    planned: 0,
                    active: 0,
                },
            )
        })
        .collect::<BTreeMap<_, _>>();

    for case in cases {
        if invalid_expectation_paths.contains(&case.expectation_path) {
            continue;
        }
        let Some(metadata) = &case.expectation.architecture22 else {
            continue;
        };
        for scenario in &metadata.scenarios {
            let Some(row) = by_scenario.get_mut(scenario) else {
                continue;
            };
            match metadata.gate {
                Architecture22Gate::Planned => row.planned += 1,
                Architecture22Gate::Active => row.active += 1,
            }
        }
    }

    let scenarios = by_scenario.into_values().collect::<Vec<_>>();
    let missing_scenarios = scenarios
        .iter()
        .filter(|scenario| scenario.planned == 0 && scenario.active == 0)
        .map(|scenario| scenario.scenario_id.clone())
        .collect();

    Architecture22MatrixReport {
        scenarios,
        missing_scenarios,
    }
}

fn validate_coverage_report(
    manifest_path: &Path,
    validation_mode: ValidationMode,
    report: &CoverageReport,
) -> Vec<ValidationDiagnostic> {
    let mut diagnostics = Vec::new();
    for requirement in &report.requirements {
        if requirement.stored_status != requirement.computed_status {
            diagnostics.push(status_drift_diagnostic(
                manifest_path,
                validation_mode,
                requirement,
            ));
        }
        if !requirement.missing_shapes.is_empty()
            && missing_coverage_is_error(validation_mode, requirement)
        {
            diagnostics.push(ValidationDiagnostic::error(
                manifest_path,
                "traceability",
                "E-TRACE-MISSING-COVERAGE",
                format!("trace.coverage.{}", requirement.id.0),
                format!(
                    "requirement `{}` is missing {} coverage",
                    requirement.id.0,
                    coverage_shape_list(&requirement.missing_shapes)
                ),
            ));
        }
    }
    diagnostics
}

fn status_drift_diagnostic(
    manifest_path: &Path,
    validation_mode: ValidationMode,
    requirement: &RequirementCoverage,
) -> ValidationDiagnostic {
    let message = format!(
        "requirement `{}` stored status `{}` differs from computed status `{}`",
        requirement.id.0,
        requirement.stored_status.as_str(),
        requirement.computed_status.as_str()
    );
    if status_drift_is_error(validation_mode, requirement) {
        ValidationDiagnostic::error(
            manifest_path,
            "traceability",
            "E-TRACE-STATUS-DRIFT",
            format!("trace.status.{}", requirement.id.0),
            message,
        )
    } else {
        ValidationDiagnostic::warning(
            manifest_path,
            "traceability",
            "W-TRACE-STATUS-DRIFT",
            format!("trace.status.{}", requirement.id.0),
            message,
        )
    }
}

fn status_drift_is_error(
    validation_mode: ValidationMode,
    requirement: &RequirementCoverage,
) -> bool {
    match validation_mode {
        ValidationMode::Metadata => false,
        ValidationMode::Development => matches!(
            requirement.stored_status,
            RequirementStatus::Covered | RequirementStatus::Partial
        ),
        ValidationMode::Release => {
            requirement.required
                && !matches!(
                    requirement.stored_status,
                    RequirementStatus::Deferred | RequirementStatus::Obsolete
                )
        }
    }
}

fn missing_coverage_is_error(
    validation_mode: ValidationMode,
    requirement: &RequirementCoverage,
) -> bool {
    match validation_mode {
        ValidationMode::Metadata => false,
        ValidationMode::Development => matches!(
            requirement.stored_status,
            RequirementStatus::Covered | RequirementStatus::Partial
        ),
        ValidationMode::Release => {
            requirement.required
                && !matches!(
                    requirement.stored_status,
                    RequirementStatus::Deferred | RequirementStatus::Obsolete
                )
        }
    }
}

fn coverage_shape_list(shapes: &[CoverageShape]) -> String {
    shapes
        .iter()
        .map(|shape| shape.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

fn requirement_id_list(requirements: &[crate::SpecRequirementId]) -> String {
    requirements
        .iter()
        .map(|requirement| requirement.0.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

fn normalized_config(config: &DiscoveryConfig) -> Result<DiscoveryConfig, HarnessError> {
    let current_dir = std::env::current_dir().map_err(|error| {
        HarnessError::Infrastructure(format!("failed to read current directory: {error}"))
    })?;
    let workspace_root = absolute_from(&current_dir, &config.workspace_root);
    Ok(DiscoveryConfig {
        workspace_root: workspace_root.clone(),
        tests_root: absolute_from(&workspace_root, &config.tests_root),
        manifest_path: absolute_from(&workspace_root, &config.manifest_path),
        profile: config.profile,
        validation_mode: config.validation_mode,
    })
}

impl fmt::Display for HarnessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Infrastructure(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for HarnessError {}

impl FromStr for TestProfile {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "fast" => Ok(Self::Fast),
            "full" => Ok(Self::Full),
            "stress" => Ok(Self::Stress),
            "fuzz_regression" | "fuzz-regression" => Ok(Self::FuzzRegression),
            "snapshot_update" | "snapshot-update" => Ok(Self::SnapshotUpdate),
            other => Err(format!("unknown test profile `{other}`")),
        }
    }
}

impl FromStr for ValidationMode {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "metadata" => Ok(Self::Metadata),
            "development" => Ok(Self::Development),
            "release" => Ok(Self::Release),
            other => Err(format!("unknown validation mode `{other}`")),
        }
    }
}
