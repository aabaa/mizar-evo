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
    pub tokens: Vec<TokenExpectation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenExpectation {
    pub kind: String,
    pub lexeme: String,
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
    let (table, token_tables) = toml_lite::parse_expectation_tables(content)
        .map_err(|error| format!("TOML parse error on line {}: {}", error.line, error.message))?;
    expectation_from_table(&table, &token_tables)
}

fn expectation_from_table(
    table: &TomlTable,
    token_tables: &[TomlTable],
) -> Result<Expectation, String> {
    validate_known_fields(table)?;
    let tokens = parse_token_expectations(token_tables)?;

    let schema_version = toml_lite::required_u32(table, "schema_version")?;
    if schema_version != 1 {
        return Err(format!("unsupported schema_version `{schema_version}`"));
    }

    let id = TestCaseId(toml_lite::required_string(table, "id")?);
    if id.0.is_empty() {
        return Err("`id` must not be empty".to_owned());
    }
    let kind = toml_lite::required_string(table, "kind")?.parse()?;
    let stage = toml_lite::required_string(table, "stage")?.parse()?;
    let domain = toml_lite::required_string(table, "domain")?;
    if domain.is_empty() {
        return Err("`domain` must not be empty".to_owned());
    }
    let source = PathBuf::from(toml_lite::required_string(table, "source")?);
    let expected_outcome = toml_lite::required_string(table, "expected_outcome")?.parse()?;
    let spec_refs = toml_lite::string_array(table, "spec_refs")?
        .into_iter()
        .map(|spec_ref| {
            if spec_ref.is_empty() {
                Err("`spec_refs` entries must not be empty".to_owned())
            } else {
                Ok(SpecRequirementId(spec_ref))
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    let expected_phase = toml_lite::optional_string(table, "expected_phase")?
        .map(|phase| {
            if phase.is_empty() {
                Err("`expected_phase` must not be empty".to_owned())
            } else {
                phase.parse()
            }
        })
        .transpose()?;
    let failure_category = toml_lite::optional_string(table, "failure_category")?;
    let rejection_reason = toml_lite::optional_string(table, "rejection_reason")?;
    let diagnostic_codes = toml_lite::string_array(table, "diagnostic_codes")?
        .into_iter()
        .map(|diagnostic_code| {
            if diagnostic_code.is_empty() {
                Err("`diagnostic_codes` entries must not be empty".to_owned())
            } else {
                Ok(diagnostic_code)
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    let stable_detail_key = toml_lite::optional_string(table, "stable_detail_key")?;
    for (field, value) in [
        ("failure_category", failure_category.as_deref()),
        ("rejection_reason", rejection_reason.as_deref()),
        ("stable_detail_key", stable_detail_key.as_deref()),
    ] {
        if value.is_some_and(str::is_empty) {
            return Err(format!("`{field}` must not be empty when present"));
        }
    }
    validate_optional_metadata_fields(table)?;

    validate_kind_outcome(kind, expected_outcome)?;
    if !tokens.is_empty() && stage != Stage::Lexical {
        return Err("token expectations are only valid for `stage = \"lexical\"`".to_owned());
    }
    if matches!(
        expected_outcome,
        ExpectedOutcome::Pass | ExpectedOutcome::Fail
    ) && expected_phase.is_none()
    {
        return Err("pass and fail expectations require `expected_phase`".to_owned());
    }
    if expected_outcome == ExpectedOutcome::Fail {
        if failure_category.as_deref().is_none_or(str::is_empty) {
            return Err("fail expectations require `failure_category`".to_owned());
        }
        if stable_detail_key.as_deref().is_none_or(str::is_empty) {
            return Err("fail expectations require `stable_detail_key`".to_owned());
        }
    }
    if expected_outcome == ExpectedOutcome::MetadataOnly
        && source
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| {
                name.ends_with(".miz") || name.ends_with(".src") || name.ends_with(".cert.json")
            })
    {
        return Err(
            "`metadata_only` is not valid for .miz, .src, or .cert.json payloads".to_owned(),
        );
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
        tokens,
    })
}

fn parse_token_expectations(token_tables: &[TomlTable]) -> Result<Vec<TokenExpectation>, String> {
    let mut tokens = Vec::with_capacity(token_tables.len());
    for table in token_tables {
        validate_token_fields(table)?;
        let kind = toml_lite::required_string(table, "kind")?;
        if kind.is_empty() {
            return Err("`tokens.kind` must not be empty".to_owned());
        }
        let lexeme = toml_lite::required_string(table, "lexeme")?;
        if lexeme.is_empty() {
            return Err("`tokens.lexeme` must not be empty".to_owned());
        }
        tokens.push(TokenExpectation { kind, lexeme });
    }
    Ok(tokens)
}

fn validate_token_fields(table: &TomlTable) -> Result<(), String> {
    const KNOWN_TOKEN_FIELDS: &[&str] = &["kind", "lexeme"];

    for key in table.keys() {
        if !KNOWN_TOKEN_FIELDS.contains(&key.as_str()) {
            return Err(format!("unknown token expectation field `{key}`"));
        }
    }
    Ok(())
}

fn validate_known_fields(table: &TomlTable) -> Result<(), String> {
    const KNOWN_FIELDS: &[&str] = &[
        "schema_version",
        "id",
        "kind",
        "stage",
        "domain",
        "source",
        "expected_outcome",
        "spec_refs",
        "profiles",
        "tags",
        "notes",
        "expected_phase",
        "diagnostic_codes",
        "snapshots",
        "failure_category",
        "rejection_reason",
        "stable_detail_key",
        "ast_profile",
        "snapshot_profiles",
    ];

    for key in table.keys() {
        if !KNOWN_FIELDS.contains(&key.as_str()) {
            return Err(format!("unknown expectation field `{key}`"));
        }
    }
    Ok(())
}

fn validate_optional_metadata_fields(table: &TomlTable) -> Result<(), String> {
    for key in ["profiles", "tags", "snapshot_profiles"] {
        if table.contains_key(key) {
            toml_lite::string_array(table, key)?;
        }
    }
    for key in ["notes", "snapshots", "ast_profile"] {
        if table.contains_key(key) {
            toml_lite::optional_string(table, key)?;
        }
    }
    Ok(())
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

    let clean_source_path = clean_relative_path(&expectation.source);
    if !clean_source_path {
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
    }

    let source_path = path.parent().unwrap_or_else(|| Path::new(""));
    if clean_source_path && !source_path.join(&expectation.source).is_file() {
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
