use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::diagnostic::{ValidationDiagnostic, ValidationSeverity};
use crate::expectation::{
    Expectation, TestCaseId, parse_expectation_file, validate_expectation_path,
};
use crate::layout;
use crate::path_rules::{absolute_from, clean_relative_path};
use crate::traceability::{TraceManifest, parse_trace_manifest, validate_manifest};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveryConfig {
    pub workspace_root: PathBuf,
    pub tests_root: PathBuf,
    pub manifest_path: PathBuf,
    pub profile: TestProfile,
    pub validation_mode: ValidationMode,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TestProfile {
    #[default]
    Fast,
    Full,
    Stress,
    FuzzRegression,
    SnapshotUpdate,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
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

        diagnostics.extend(validate_expectation_path(&sidecar, &expectation));

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

    cases.sort_by(|left, right| left.expectation_path.cmp(&right.expectation_path));
    diagnostics.sort();

    Ok(TestPlan {
        cases,
        manifest,
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
