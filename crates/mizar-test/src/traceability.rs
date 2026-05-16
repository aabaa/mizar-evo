use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::diagnostic::ValidationDiagnostic;
pub use crate::expectation::SpecRequirementId;
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

        if !workspace_root.join(&requirement.source).is_file() {
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
