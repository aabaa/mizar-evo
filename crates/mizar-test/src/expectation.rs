use std::collections::BTreeSet;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::diagnostic::ValidationDiagnostic;
use crate::path_rules::{clean_relative_path, executable_payload_stem};
use crate::staged_model::Stage;
use crate::toml_lite::{self, TomlTable};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TestCaseId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SpecRequirementId(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestKind {
    Pass,
    Fail,
    Snapshot,
    Generated,
    FuzzSeed,
    PropertySeed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpectedOutcome {
    Pass,
    Fail,
    Snapshot,
    MetadataOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelinePhase {
    Lex,
    Parse,
    Resolve,
    TypeCheck,
    Elaboration,
    ClusterResolution,
    OverloadResolution,
    StatementCheck,
    VcGeneration,
    Verification,
    CertificateCheck,
    KernelCheck,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expectation {
    pub schema_version: u32,
    pub id: TestCaseId,
    pub kind: TestKind,
    pub stage: Stage,
    pub domain: String,
    pub source: PathBuf,
    pub expected_outcome: ExpectedOutcome,
    pub spec_refs: Vec<SpecRequirementId>,
    pub expected_phase: Option<PipelinePhase>,
    pub failure_category: Option<String>,
    pub rejection_reason: Option<String>,
    pub diagnostic_codes: Vec<String>,
    pub stable_detail_key: Option<String>,
}

pub fn parse_expectation_file(path: &Path) -> Result<Expectation, ValidationDiagnostic> {
    let content = fs::read_to_string(path).map_err(|error| {
        ValidationDiagnostic::error(
            path,
            "expectation",
            "E-EXPECT-READ",
            "expectation.read",
            format!("failed to read expectation: {error}"),
        )
    })?;
    parse_expectation_str(&content).map_err(|message| {
        ValidationDiagnostic::error(
            path,
            "expectation",
            "E-EXPECT-SCHEMA",
            "expectation.schema",
            message,
        )
    })
}

pub fn parse_expectation_str(content: &str) -> Result<Expectation, String> {
    let table = toml_lite::parse_table(content)
        .map_err(|error| format!("TOML parse error on line {}: {}", error.line, error.message))?;
    expectation_from_table(&table)
}

fn expectation_from_table(table: &TomlTable) -> Result<Expectation, String> {
    let schema_version = toml_lite::required_u32(table, "schema_version")?;
    if schema_version != 1 {
        return Err(format!("unsupported schema_version `{schema_version}`"));
    }

    let id = TestCaseId(toml_lite::required_string(table, "id")?);
    let kind = toml_lite::required_string(table, "kind")?.parse()?;
    let stage = toml_lite::required_string(table, "stage")?.parse()?;
    let domain = toml_lite::required_string(table, "domain")?;
    let source = PathBuf::from(toml_lite::required_string(table, "source")?);
    let expected_outcome = toml_lite::required_string(table, "expected_outcome")?.parse()?;
    let spec_refs = toml_lite::string_array(table, "spec_refs")?
        .into_iter()
        .map(SpecRequirementId)
        .collect();
    let expected_phase = toml_lite::optional_string(table, "expected_phase")?
        .map(|phase| phase.parse())
        .transpose()?;
    let failure_category = toml_lite::optional_string(table, "failure_category")?;
    let rejection_reason = toml_lite::optional_string(table, "rejection_reason")?;
    let diagnostic_codes = match table.get("diagnostic_codes") {
        Some(_) => toml_lite::string_array(table, "diagnostic_codes")?,
        None => Vec::new(),
    };
    let stable_detail_key = toml_lite::optional_string(table, "stable_detail_key")?;

    validate_kind_outcome(kind, expected_outcome)?;
    if expected_outcome == ExpectedOutcome::Fail {
        if failure_category.is_none() {
            return Err("fail expectations require `failure_category`".to_owned());
        }
        if stable_detail_key.is_none() {
            return Err("fail expectations require `stable_detail_key`".to_owned());
        }
    }

    Ok(Expectation {
        schema_version,
        id,
        kind,
        stage,
        domain,
        source,
        expected_outcome,
        spec_refs,
        expected_phase,
        failure_category,
        rejection_reason,
        diagnostic_codes,
        stable_detail_key,
    })
}

pub fn validate_expectation_path(
    path: &Path,
    expectation: &Expectation,
) -> Vec<ValidationDiagnostic> {
    let mut diagnostics = Vec::new();
    let sidecar_stem = expectation_stem(path).unwrap_or_default();
    if expectation.id.0 != sidecar_stem {
        diagnostics.push(ValidationDiagnostic::error(
            path,
            "expectation",
            "E-EXPECT-ID-MISMATCH",
            "expectation.id",
            format!(
                "expectation id `{}` does not match sidecar stem `{sidecar_stem}`",
                expectation.id.0
            ),
        ));
    }

    if !clean_relative_path(&expectation.source) {
        diagnostics.push(ValidationDiagnostic::error(
            path,
            "expectation",
            "E-EXPECT-SOURCE-PATH",
            "expectation.source_path",
            format!(
                "source `{}` must be a clean relative path",
                expectation.source.display()
            ),
        ));
        return diagnostics;
    }

    let source_path = path
        .parent()
        .unwrap_or_else(|| Path::new(""))
        .join(&expectation.source);
    if !source_path.is_file() {
        diagnostics.push(ValidationDiagnostic::error(
            path,
            "expectation",
            "E-EXPECT-MISSING-SOURCE",
            "expectation.source",
            format!("source `{}` does not exist", expectation.source.display()),
        ));
    }

    match executable_payload_stem(&expectation.source) {
        Some(source_stem) if source_stem != sidecar_stem => {
            diagnostics.push(ValidationDiagnostic::error(
                path,
                "expectation",
                "E-EXPECT-SOURCE-STEM",
                "expectation.source_stem",
                format!("source stem `{source_stem}` does not match sidecar stem `{sidecar_stem}`"),
            ));
        }
        Some(_) => {}
        None => diagnostics.push(ValidationDiagnostic::error(
            path,
            "expectation",
            "E-EXPECT-SOURCE-EXTENSION",
            "expectation.source_extension",
            format!(
                "source `{}` must use .miz, .src, .cert.json, or .fixture.toml",
                expectation.source.display()
            ),
        )),
    }

    if expectation.spec_refs.is_empty() {
        diagnostics.push(ValidationDiagnostic::error(
            path,
            "expectation",
            "E-EXPECT-SPEC-REFS",
            "expectation.spec_refs",
            "committed executable tests must list at least one spec_ref",
        ));
    }

    let mut seen = BTreeSet::new();
    for spec_ref in &expectation.spec_refs {
        if !seen.insert(spec_ref.0.clone()) {
            diagnostics.push(ValidationDiagnostic::error(
                path,
                "expectation",
                "E-EXPECT-DUP-SPEC-REF",
                format!("expectation.spec_refs.{}", spec_ref.0),
                format!("duplicate spec_ref `{}`", spec_ref.0),
            ));
        }
    }

    diagnostics
}

pub fn expectation_stem(path: &Path) -> Option<String> {
    let name = path.file_name()?.to_str()?;
    name.strip_suffix(".expect.toml").map(str::to_owned)
}

pub fn payload_stem(path: &Path) -> Option<String> {
    let name = path.file_name()?.to_str()?;
    for suffix in [
        ".fixture.toml",
        ".cert.json",
        ".expect.toml",
        ".miz",
        ".src",
    ] {
        if let Some(stem) = name.strip_suffix(suffix) {
            return Some(stem.to_owned());
        }
    }
    None
}

fn validate_kind_outcome(kind: TestKind, outcome: ExpectedOutcome) -> Result<(), String> {
    let allowed = match kind {
        TestKind::Pass => matches!(outcome, ExpectedOutcome::Pass | ExpectedOutcome::Snapshot),
        TestKind::Fail => matches!(outcome, ExpectedOutcome::Fail | ExpectedOutcome::Snapshot),
        TestKind::Snapshot => matches!(outcome, ExpectedOutcome::Snapshot),
        TestKind::Generated => matches!(
            outcome,
            ExpectedOutcome::Pass | ExpectedOutcome::Fail | ExpectedOutcome::Snapshot
        ),
        TestKind::FuzzSeed => matches!(
            outcome,
            ExpectedOutcome::Fail | ExpectedOutcome::MetadataOnly
        ),
        TestKind::PropertySeed => matches!(
            outcome,
            ExpectedOutcome::Pass | ExpectedOutcome::Fail | ExpectedOutcome::MetadataOnly
        ),
    };
    if allowed {
        Ok(())
    } else {
        Err(format!(
            "kind `{kind}` is not compatible with expected_outcome `{outcome}`"
        ))
    }
}

impl fmt::Display for TestKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Pass => "pass",
            Self::Fail => "fail",
            Self::Snapshot => "snapshot",
            Self::Generated => "generated",
            Self::FuzzSeed => "fuzz_seed",
            Self::PropertySeed => "property_seed",
        })
    }
}

impl fmt::Display for ExpectedOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Pass => "pass",
            Self::Fail => "fail",
            Self::Snapshot => "snapshot",
            Self::MetadataOnly => "metadata_only",
        })
    }
}

impl FromStr for TestKind {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "pass" => Ok(Self::Pass),
            "fail" => Ok(Self::Fail),
            "snapshot" => Ok(Self::Snapshot),
            "generated" => Ok(Self::Generated),
            "fuzz_seed" => Ok(Self::FuzzSeed),
            "property_seed" => Ok(Self::PropertySeed),
            other => Err(format!("unknown kind `{other}`")),
        }
    }
}

impl FromStr for ExpectedOutcome {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "pass" => Ok(Self::Pass),
            "fail" => Ok(Self::Fail),
            "snapshot" => Ok(Self::Snapshot),
            "metadata_only" => Ok(Self::MetadataOnly),
            other => Err(format!("unknown expected_outcome `{other}`")),
        }
    }
}

impl FromStr for PipelinePhase {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "lex" => Ok(Self::Lex),
            "parse" => Ok(Self::Parse),
            "resolve" => Ok(Self::Resolve),
            "type_check" => Ok(Self::TypeCheck),
            "elaboration" => Ok(Self::Elaboration),
            "cluster_resolution" => Ok(Self::ClusterResolution),
            "overload_resolution" => Ok(Self::OverloadResolution),
            "statement_check" => Ok(Self::StatementCheck),
            "vc_generation" => Ok(Self::VcGeneration),
            "verification" => Ok(Self::Verification),
            "certificate_check" => Ok(Self::CertificateCheck),
            "kernel_check" => Ok(Self::KernelCheck),
            other => Err(format!("unknown expected_phase `{other}`")),
        }
    }
}
