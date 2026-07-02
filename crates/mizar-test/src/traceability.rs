use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::diagnostic::ValidationDiagnostic;
pub use crate::expectation::SpecRequirementId;
use crate::expectation::{ExpectedOutcome, TestKind};
use crate::path_rules::clean_relative_path;
use crate::staged_model::Stage;
use crate::toml_lite::{self, TomlTable};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequirementStatus {
    Planned,
    Covered,
    Partial,
    Deferred,
    Obsolete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoverageShape {
    None,
    Pass,
    Fail,
    PassAndFail,
    Diagnostic,
    Snapshot,
    Property,
    ManualReview,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecRequirement {
    pub id: SpecRequirementId,
    pub source: PathBuf,
    pub section: String,
    pub stage: Stage,
    pub status: RequirementStatus,
    pub required: bool,
    pub coverage: CoverageShape,
    pub tests: Vec<PathBuf>,
    pub deferred_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceManifest {
    pub requirements: Vec<SpecRequirement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoverageEvidence {
    pub requirement_id: SpecRequirementId,
    pub test_path: PathBuf,
    pub stage: Stage,
    pub kind: TestKind,
    pub expected_outcome: ExpectedOutcome,
    pub has_diagnostic_evidence: bool,
    pub has_snapshot: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoverageEvidenceSummary {
    pub pass: usize,
    pub fail: usize,
    pub diagnostic: usize,
    pub snapshot: usize,
    pub property: usize,
    pub manual_review: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequirementCoverage {
    pub id: SpecRequirementId,
    pub stage: Stage,
    pub coverage: CoverageShape,
    pub required: bool,
    pub stored_status: RequirementStatus,
    pub computed_status: RequirementStatus,
    pub evidence: CoverageEvidenceSummary,
    pub missing_shapes: Vec<CoverageShape>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageCoverage {
    pub stage: Stage,
    pub requirements: usize,
    pub covered: usize,
    pub partial: usize,
    pub planned: usize,
    pub deferred: usize,
    pub obsolete: usize,
    pub missing_shapes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PassFailMix {
    pub pass: usize,
    pub fail: usize,
    pub total: usize,
    pub target_pass_percent: u8,
    pub target_fail_percent: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoverageReport {
    pub requirements: Vec<RequirementCoverage>,
    pub stages: Vec<StageCoverage>,
    pub pass_fail_mix: PassFailMix,
}

impl TraceManifest {
    pub fn requirement_ids(&self) -> BTreeSet<SpecRequirementId> {
        self.requirements.iter().map(|req| req.id.clone()).collect()
    }

    pub fn by_id(&self) -> BTreeMap<SpecRequirementId, &SpecRequirement> {
        self.requirements
            .iter()
            .map(|req| (req.id.clone(), req))
            .collect()
    }

    pub fn coverage_report(
        &self,
        evidence: &[CoverageEvidence],
        pass_fail_mix: PassFailMix,
    ) -> CoverageReport {
        let evidence_by_requirement = evidence.iter().fold(
            BTreeMap::<SpecRequirementId, Vec<&CoverageEvidence>>::new(),
            |mut by_requirement, item| {
                by_requirement
                    .entry(item.requirement_id.clone())
                    .or_default()
                    .push(item);
                by_requirement
            },
        );

        let requirements = self
            .requirements
            .iter()
            .map(|requirement| {
                let evidence = evidence_by_requirement
                    .get(&requirement.id)
                    .map(Vec::as_slice)
                    .unwrap_or(&[]);
                requirement_coverage(requirement, evidence)
            })
            .collect::<Vec<_>>();
        let stages = stage_coverage(&requirements);
        CoverageReport {
            requirements,
            stages,
            pass_fail_mix,
        }
    }
}

pub fn parse_trace_manifest(path: &Path) -> Result<TraceManifest, ValidationDiagnostic> {
    let content = fs::read_to_string(path).map_err(|error| {
        ValidationDiagnostic::error(
            path,
            "manifest",
            "E-MANIFEST-READ",
            "manifest.read",
            format!("failed to read trace manifest: {error}"),
        )
    })?;
    parse_trace_manifest_str(&content).map_err(|message| {
        ValidationDiagnostic::error(
            path,
            "manifest",
            "E-MANIFEST-SCHEMA",
            "manifest.schema",
            message,
        )
    })
}

pub fn parse_trace_manifest_str(content: &str) -> Result<TraceManifest, String> {
    let records = toml_lite::parse_requirement_tables(content)
        .map_err(|error| format!("TOML parse error on line {}: {}", error.line, error.message))?;
    let requirements = records
        .iter()
        .map(requirement_from_table)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(TraceManifest { requirements })
}

pub fn validate_manifest(
    workspace_root: &Path,
    manifest_path: &Path,
    manifest: &TraceManifest,
) -> Vec<ValidationDiagnostic> {
    let mut diagnostics = Vec::new();
    let mut ids = BTreeSet::new();
    let mut previous_id: Option<&SpecRequirementId> = None;

    for requirement in &manifest.requirements {
        if !ids.insert(requirement.id.clone()) {
            diagnostics.push(ValidationDiagnostic::error(
                manifest_path,
                "manifest",
                "E-MANIFEST-DUP-ID",
                format!("manifest.id.{}", requirement.id.0),
                format!("duplicate requirement id `{}`", requirement.id.0),
            ));
        }
        if let Some(previous) = previous_id
            && previous > &requirement.id
        {
            diagnostics.push(ValidationDiagnostic::error(
                manifest_path,
                "manifest",
                "E-MANIFEST-ID-ORDER",
                format!("manifest.order.{}", requirement.id.0),
                format!(
                    "requirement id `{}` must be sorted after `{}`",
                    requirement.id.0, previous.0
                ),
            ));
        }
        previous_id = Some(&requirement.id);

        if !clean_relative_path(&requirement.source) {
            diagnostics.push(ValidationDiagnostic::error(
                manifest_path,
                "manifest",
                "E-MANIFEST-SOURCE-PATH",
                format!("manifest.source.{}", requirement.id.0),
                format!(
                    "requirement `{}` source `{}` must be a clean relative path",
                    requirement.id.0,
                    requirement.source.display()
                ),
            ));
        } else if !workspace_root.join(&requirement.source).is_file() {
            diagnostics.push(ValidationDiagnostic::error(
                manifest_path,
                "manifest",
                "E-MANIFEST-MISSING-SOURCE",
                format!("manifest.source.{}", requirement.id.0),
                format!(
                    "requirement `{}` source `{}` does not exist",
                    requirement.id.0,
                    requirement.source.display()
                ),
            ));
        }

        for test in &requirement.tests {
            if !clean_relative_path(test) {
                diagnostics.push(ValidationDiagnostic::error(
                    manifest_path,
                    "manifest",
                    "E-MANIFEST-TEST-PATH",
                    format!("manifest.tests.{}.{}", requirement.id.0, test.display()),
                    format!(
                        "requirement `{}` test `{}` must be a clean relative path",
                        requirement.id.0,
                        test.display()
                    ),
                ));
            }
        }

        if requirement.status == RequirementStatus::Deferred
            && requirement.required
            && requirement.deferred_reason.is_none()
        {
            diagnostics.push(ValidationDiagnostic::error(
                manifest_path,
                "manifest",
                "E-MANIFEST-DEFERRED-REASON",
                format!("manifest.deferred_reason.{}", requirement.id.0),
                format!(
                    "deferred required requirement `{}` needs a reason",
                    requirement.id.0
                ),
            ));
        }

        if requirement.status == RequirementStatus::Planned && requirement.tests.is_empty() {
            diagnostics.push(ValidationDiagnostic::warning(
                manifest_path,
                "manifest",
                "W-MANIFEST-PLANNED-NO-TESTS",
                format!("manifest.tests.{}", requirement.id.0),
                format!("planned requirement `{}` has no tests", requirement.id.0),
            ));
        }
    }

    diagnostics
}

fn requirement_coverage(
    requirement: &SpecRequirement,
    evidence: &[&CoverageEvidence],
) -> RequirementCoverage {
    let evidence_summary = summarize_evidence(evidence);
    let missing_shapes = missing_shapes(requirement.coverage, &evidence_summary);
    let computed_status = computed_status(requirement, &evidence_summary, &missing_shapes);
    RequirementCoverage {
        id: requirement.id.clone(),
        stage: requirement.stage,
        coverage: requirement.coverage,
        required: requirement.required,
        stored_status: requirement.status,
        computed_status,
        evidence: evidence_summary,
        missing_shapes,
    }
}

fn summarize_evidence(evidence: &[&CoverageEvidence]) -> CoverageEvidenceSummary {
    CoverageEvidenceSummary {
        pass: evidence
            .iter()
            .filter(|item| item.expected_outcome == ExpectedOutcome::Pass)
            .count(),
        fail: evidence
            .iter()
            .filter(|item| item.expected_outcome == ExpectedOutcome::Fail)
            .count(),
        diagnostic: evidence
            .iter()
            .filter(|item| item.has_diagnostic_evidence)
            .count(),
        snapshot: evidence
            .iter()
            .filter(|item| {
                item.has_snapshot
                    || item.kind == TestKind::Snapshot
                    || item.expected_outcome == ExpectedOutcome::Snapshot
            })
            .count(),
        property: evidence
            .iter()
            .filter(|item| item.kind == TestKind::PropertySeed)
            .count(),
        manual_review: evidence.len(),
    }
}

fn missing_shapes(
    coverage: CoverageShape,
    evidence: &CoverageEvidenceSummary,
) -> Vec<CoverageShape> {
    required_shapes(coverage)
        .into_iter()
        .filter(|shape| !shape_is_satisfied(*shape, evidence))
        .collect()
}

fn required_shapes(coverage: CoverageShape) -> Vec<CoverageShape> {
    match coverage {
        CoverageShape::None => Vec::new(),
        CoverageShape::PassAndFail => vec![CoverageShape::Pass, CoverageShape::Fail],
        other => vec![other],
    }
}

fn shape_is_satisfied(shape: CoverageShape, evidence: &CoverageEvidenceSummary) -> bool {
    match shape {
        CoverageShape::None => true,
        CoverageShape::Pass => evidence.pass > 0,
        CoverageShape::Fail => evidence.fail > 0,
        CoverageShape::PassAndFail => evidence.pass > 0 && evidence.fail > 0,
        CoverageShape::Diagnostic => evidence.diagnostic > 0,
        CoverageShape::Snapshot => evidence.snapshot > 0,
        CoverageShape::Property => evidence.property > 0,
        CoverageShape::ManualReview => evidence.manual_review > 0,
    }
}

fn computed_status(
    requirement: &SpecRequirement,
    evidence: &CoverageEvidenceSummary,
    missing_shapes: &[CoverageShape],
) -> RequirementStatus {
    if requirement.status == RequirementStatus::Deferred {
        return RequirementStatus::Deferred;
    }
    if requirement.status == RequirementStatus::Obsolete {
        return RequirementStatus::Obsolete;
    }
    if matches!(
        requirement.coverage,
        CoverageShape::None | CoverageShape::ManualReview
    ) {
        return if evidence.manual_review == 0 && requirement.coverage == CoverageShape::ManualReview
        {
            RequirementStatus::Planned
        } else {
            requirement.status
        };
    }

    let required_count = required_shapes(requirement.coverage).len();
    if missing_shapes.is_empty() {
        RequirementStatus::Covered
    } else if missing_shapes.len() < required_count {
        RequirementStatus::Partial
    } else {
        RequirementStatus::Planned
    }
}

fn stage_coverage(requirements: &[RequirementCoverage]) -> Vec<StageCoverage> {
    let mut by_stage = BTreeMap::<Stage, StageCoverage>::new();
    for requirement in requirements {
        let stage = by_stage.entry(requirement.stage).or_insert(StageCoverage {
            stage: requirement.stage,
            requirements: 0,
            covered: 0,
            partial: 0,
            planned: 0,
            deferred: 0,
            obsolete: 0,
            missing_shapes: 0,
        });
        stage.requirements += 1;
        stage.missing_shapes += requirement.missing_shapes.len();
        match requirement.computed_status {
            RequirementStatus::Planned => stage.planned += 1,
            RequirementStatus::Covered => stage.covered += 1,
            RequirementStatus::Partial => stage.partial += 1,
            RequirementStatus::Deferred => stage.deferred += 1,
            RequirementStatus::Obsolete => stage.obsolete += 1,
        }
    }
    by_stage.into_values().collect()
}

fn requirement_from_table(table: &TomlTable) -> Result<SpecRequirement, String> {
    let id = SpecRequirementId(toml_lite::required_string(table, "id")?);
    let source = PathBuf::from(toml_lite::required_string(table, "source")?);
    let section = toml_lite::required_string(table, "section")?;
    let stage = toml_lite::required_string(table, "stage")?.parse()?;
    let status = toml_lite::required_string(table, "status")?.parse()?;
    let required = toml_lite::required_bool(table, "required")?;
    let coverage = toml_lite::required_string(table, "coverage")?.parse()?;
    let tests = toml_lite::string_array(table, "tests")?
        .into_iter()
        .map(PathBuf::from)
        .collect();
    let deferred_reason = toml_lite::optional_string(table, "deferred_reason")?;

    Ok(SpecRequirement {
        id,
        source,
        section,
        stage,
        status,
        required,
        coverage,
        tests,
        deferred_reason,
    })
}

impl FromStr for RequirementStatus {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "planned" => Ok(Self::Planned),
            "covered" => Ok(Self::Covered),
            "partial" => Ok(Self::Partial),
            "deferred" => Ok(Self::Deferred),
            "obsolete" => Ok(Self::Obsolete),
            other => Err(format!("unknown requirement status `{other}`")),
        }
    }
}

impl RequirementStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Covered => "covered",
            Self::Partial => "partial",
            Self::Deferred => "deferred",
            Self::Obsolete => "obsolete",
        }
    }
}

impl FromStr for CoverageShape {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "none" => Ok(Self::None),
            "pass" => Ok(Self::Pass),
            "fail" => Ok(Self::Fail),
            "pass_and_fail" => Ok(Self::PassAndFail),
            "diagnostic" => Ok(Self::Diagnostic),
            "snapshot" => Ok(Self::Snapshot),
            "property" => Ok(Self::Property),
            "manual_review" => Ok(Self::ManualReview),
            other => Err(format!("unknown coverage shape `{other}`")),
        }
    }
}

impl CoverageShape {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Pass => "pass",
            Self::Fail => "fail",
            Self::PassAndFail => "pass_and_fail",
            Self::Diagnostic => "diagnostic",
            Self::Snapshot => "snapshot",
            Self::Property => "property",
            Self::ManualReview => "manual_review",
        }
    }
}
