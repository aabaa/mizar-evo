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
#[non_exhaustive]
pub enum TestKind {
    Pass,
    Fail,
    Snapshot,
    Generated,
    FuzzSeed,
    PropertySeed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ExpectedOutcome {
    Pass,
    Fail,
    Snapshot,
    MetadataOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Architecture22Gate {
    Planned,
    Active,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Architecture22ScenarioSpec {
    pub id: &'static str,
    pub summary: &'static str,
    pub equivalence_class: &'static str,
    pub active_eligibility: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Architecture22Metadata {
    pub scenarios: Vec<String>,
    pub equivalence_class: Option<String>,
    pub gate: Architecture22Gate,
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
    pub profiles: Vec<String>,
    pub expected_phase: Option<PipelinePhase>,
    pub failure_category: Option<String>,
    pub rejection_reason: Option<String>,
    pub diagnostic_codes: Vec<String>,
    pub diagnostic_payloads: Vec<String>,
    pub declaration_symbol_payloads: Vec<String>,
    pub stable_detail_key: Option<String>,
    pub tags: Vec<String>,
    pub notes: Option<String>,
    pub snapshots: Option<PathBuf>,
    pub ast_profile: Option<String>,
    pub snapshot_profiles: Vec<String>,
    pub tokens: Vec<TokenExpectation>,
    pub origin: Option<OriginMetadata>,
    pub architecture22: Option<Architecture22Metadata>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OriginMetadata {
    pub schema_version: u32,
    pub kind: TestKind,
    pub generator: String,
    pub generator_version: String,
    pub seed: String,
    pub profile: String,
    pub expected_outcome: ExpectedOutcome,
    pub minimized: bool,
    pub original_failure_category: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenExpectation {
    pub kind: String,
    pub lexeme: String,
    pub span_start: Option<u32>,
    pub span_end: Option<u32>,
    pub span_start_line: Option<u32>,
    pub span_start_col: Option<u32>,
    pub span_end_line: Option<u32>,
    pub span_end_col: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RequiredSoundnessCase {
    pub key: &'static str,
    pub domain: &'static str,
    pub expected_outcome: ExpectedOutcome,
    pub allowed_failure_categories: &'static [&'static str],
    pub allowed_rejection_reasons: &'static [&'static str],
    pub allowed_stages: &'static [Stage],
    pub allowed_phases: &'static [PipelinePhase],
    pub requires_fast_profile: bool,
}

const PROOF_OR_KERNEL_PHASES: &[PipelinePhase] = &[
    PipelinePhase::Verification,
    PipelinePhase::CertificateCheck,
    PipelinePhase::KernelCheck,
];

const CERTIFICATE_PHASES: &[PipelinePhase] =
    &[PipelinePhase::CertificateCheck, PipelinePhase::KernelCheck];

const ADVANCED_STAGES: &[Stage] = &[Stage::AdvancedSemantics];
const PROOF_OR_ADVANCED_STAGES: &[Stage] = &[Stage::ProofVerification, Stage::AdvancedSemantics];
const PROOF_CERT_OR_KERNEL_CATEGORIES: &[&str] =
    &["proof_failure", "certificate_rejection", "kernel_rejection"];
const CERTIFICATE_OR_KERNEL_CATEGORIES: &[&str] = &["certificate_rejection", "kernel_rejection"];
const KERNEL_REJECTION_CATEGORY: &[&str] = &["kernel_rejection"];
const CLUSTER_ERROR_CATEGORY: &[&str] = &["cluster_error"];
const OVERLOAD_ERROR_CATEGORY: &[&str] = &["overload_error"];

pub const ARCHITECTURE22_SCENARIOS: &[Architecture22ScenarioSpec] = &[
    Architecture22ScenarioSpec {
        id: "artifact_manifest_atomicity",
        summary: "artifact manifest commit remains atomic and deterministic",
        equivalence_class: "atomic_publication",
        active_eligibility: None,
    },
    Architecture22ScenarioSpec {
        id: "cache_hit_miss_timing",
        summary: "cache hit/miss timing does not affect diagnostics, proof acceptance, or artifact order",
        equivalence_class: "observable_outputs_equal",
        active_eligibility: None,
    },
    Architecture22ScenarioSpec {
        id: "cache_key_race",
        summary: "two workers racing on the same cache key cannot publish divergent contents",
        equivalence_class: "single_canonical_publication",
        active_eligibility: None,
    },
    Architecture22ScenarioSpec {
        id: "clean_incremental_artifact_equivalence",
        summary: "clean build equals incremental build for externally visible artifacts",
        equivalence_class: "observable_outputs_equal",
        active_eligibility: None,
    },
    Architecture22ScenarioSpec {
        id: "clean_parallel_equivalence",
        summary: "clean sequential build equals clean parallel build",
        equivalence_class: "observable_outputs_equal",
        active_eligibility: None,
    },
    Architecture22ScenarioSpec {
        id: "externally_attested_non_upgrade",
        summary: "externally attested evidence is never upgraded to kernel-verified by cache reuse",
        equivalence_class: "evidence_class_not_upgraded",
        active_eligibility: None,
    },
    Architecture22ScenarioSpec {
        id: "incremental_parallel_equivalence",
        summary: "sequential incremental build equals parallel incremental build",
        equivalence_class: "observable_outputs_equal",
        active_eligibility: None,
    },
    Architecture22ScenarioSpec {
        id: "missing_dependency_slice_cache_miss",
        summary: "missing dependency slice forces cache miss",
        equivalence_class: "cache_miss_only",
        active_eligibility: None,
    },
    Architecture22ScenarioSpec {
        id: "notation_operator_invalidation",
        summary: "notation/operator metadata edits invalidate affected token/AST views",
        equivalence_class: "downstream_invalidation",
        active_eligibility: None,
    },
    Architecture22ScenarioSpec {
        id: "proof_witness_mismatch",
        summary: "proof witness hash mismatch causes proof cache miss",
        equivalence_class: "cache_miss_only",
        active_eligibility: None,
    },
    Architecture22ScenarioSpec {
        id: "randomized_atp_completion_order",
        summary: "randomized ATP backend completion order does not change accepted proof status",
        equivalence_class: "deterministic_policy_selection",
        active_eligibility: None,
    },
    Architecture22ScenarioSpec {
        id: "randomized_ready_task_scheduling",
        summary: "randomized ready-task scheduling produces identical artifacts and canonical ordering",
        equivalence_class: "canonical_order_equal",
        active_eligibility: None,
    },
    Architecture22ScenarioSpec {
        id: "registration_cluster_invalidation",
        summary: "registration/coherence/reducibility/cluster changes invalidate active views and dependent VCs",
        equivalence_class: "downstream_invalidation",
        active_eligibility: None,
    },
    Architecture22ScenarioSpec {
        id: "registration_origin_deletion",
        summary: "deleting or renaming a registration removes stale cluster-db origins before cache hits",
        equivalence_class: "downstream_invalidation",
        active_eligibility: None,
    },
    Architecture22ScenarioSpec {
        id: "stale_snapshot_non_publication",
        summary: "stale snapshot diagnostics and obsolete results are not published as current",
        equivalence_class: "stale_result_not_published",
        active_eligibility: None,
    },
    Architecture22ScenarioSpec {
        id: "theorem_proof_body_invalidation",
        summary: "theorem proof-body-only edit refreshes local artifacts without downstream semantic rebuild when statement/status are unchanged",
        equivalence_class: "local_refresh_only",
        active_eligibility: None,
    },
    Architecture22ScenarioSpec {
        id: "theorem_status_invalidation",
        summary: "theorem accepted/open/failing status change invalidates visible importers",
        equivalence_class: "downstream_invalidation",
        active_eligibility: None,
    },
    Architecture22ScenarioSpec {
        id: "vcid_reorder_anchor_reuse",
        summary: "VcId reordering reuses only obligations matching anchor, VC/context/dependency/policy, and witness/discharge hashes",
        equivalence_class: "reuse_requires_full_identity",
        active_eligibility: None,
    },
];

pub fn architecture22_scenario_specs() -> &'static [Architecture22ScenarioSpec] {
    ARCHITECTURE22_SCENARIOS
}

pub fn architecture22_scenario_spec(id: &str) -> Option<&'static Architecture22ScenarioSpec> {
    ARCHITECTURE22_SCENARIOS
        .iter()
        .find(|scenario| scenario.id == id)
}

pub(crate) const REQUIRED_SOUNDNESS_CASES: &[RequiredSoundnessCase] = &[
    RequiredSoundnessCase {
        key: "soundness.false_arithmetic.one_eq_zero",
        domain: "soundness",
        expected_outcome: ExpectedOutcome::Fail,
        allowed_failure_categories: PROOF_CERT_OR_KERNEL_CATEGORIES,
        allowed_rejection_reasons: &[],
        allowed_stages: PROOF_OR_ADVANCED_STAGES,
        allowed_phases: PROOF_OR_KERNEL_PHASES,
        requires_fast_profile: true,
    },
    RequiredSoundnessCase {
        key: "soundness.substitution.variable_capture",
        domain: "substitution",
        expected_outcome: ExpectedOutcome::Fail,
        allowed_failure_categories: PROOF_CERT_OR_KERNEL_CATEGORIES,
        allowed_rejection_reasons: &["invalid_substitution"],
        allowed_stages: ADVANCED_STAGES,
        allowed_phases: PROOF_OR_KERNEL_PHASES,
        requires_fast_profile: true,
    },
    RequiredSoundnessCase {
        key: "soundness.substitution.binder_collision",
        domain: "substitution",
        expected_outcome: ExpectedOutcome::Fail,
        allowed_failure_categories: PROOF_CERT_OR_KERNEL_CATEGORIES,
        allowed_rejection_reasons: &["invalid_substitution"],
        allowed_stages: ADVANCED_STAGES,
        allowed_phases: PROOF_OR_KERNEL_PHASES,
        requires_fast_profile: true,
    },
    RequiredSoundnessCase {
        key: "soundness.substitution.malformed_substitution",
        domain: "substitution",
        expected_outcome: ExpectedOutcome::Fail,
        allowed_failure_categories: PROOF_CERT_OR_KERNEL_CATEGORIES,
        allowed_rejection_reasons: &["invalid_substitution"],
        allowed_stages: ADVANCED_STAGES,
        allowed_phases: PROOF_OR_KERNEL_PHASES,
        requires_fast_profile: true,
    },
    RequiredSoundnessCase {
        key: "soundness.substitution.alpha_conversion_failure",
        domain: "substitution",
        expected_outcome: ExpectedOutcome::Fail,
        allowed_failure_categories: PROOF_CERT_OR_KERNEL_CATEGORIES,
        allowed_rejection_reasons: &["invalid_substitution"],
        allowed_stages: ADVANCED_STAGES,
        allowed_phases: PROOF_OR_KERNEL_PHASES,
        requires_fast_profile: true,
    },
    RequiredSoundnessCase {
        key: "soundness.certificate.malformed_certificate",
        domain: "certificate",
        expected_outcome: ExpectedOutcome::Fail,
        allowed_failure_categories: &["certificate_rejection"],
        allowed_rejection_reasons: &["malformed_certificate"],
        allowed_stages: ADVANCED_STAGES,
        allowed_phases: CERTIFICATE_PHASES,
        requires_fast_profile: true,
    },
    RequiredSoundnessCase {
        key: "soundness.certificate.invalid_substitution",
        domain: "certificate",
        expected_outcome: ExpectedOutcome::Fail,
        allowed_failure_categories: KERNEL_REJECTION_CATEGORY,
        allowed_rejection_reasons: &["invalid_substitution"],
        allowed_stages: ADVANCED_STAGES,
        allowed_phases: CERTIFICATE_PHASES,
        requires_fast_profile: true,
    },
    RequiredSoundnessCase {
        key: "soundness.certificate.invalid_sat_proof",
        domain: "certificate",
        expected_outcome: ExpectedOutcome::Fail,
        allowed_failure_categories: KERNEL_REJECTION_CATEGORY,
        allowed_rejection_reasons: &["invalid_sat_proof"],
        allowed_stages: ADVANCED_STAGES,
        allowed_phases: CERTIFICATE_PHASES,
        requires_fast_profile: true,
    },
    RequiredSoundnessCase {
        key: "soundness.certificate.unresolved_symbol",
        domain: "certificate",
        expected_outcome: ExpectedOutcome::Fail,
        allowed_failure_categories: CERTIFICATE_OR_KERNEL_CATEGORIES,
        allowed_rejection_reasons: &["unresolved_symbol"],
        allowed_stages: ADVANCED_STAGES,
        allowed_phases: CERTIFICATE_PHASES,
        requires_fast_profile: true,
    },
    RequiredSoundnessCase {
        key: "soundness.certificate.timeout",
        domain: "certificate",
        expected_outcome: ExpectedOutcome::Fail,
        allowed_failure_categories: CERTIFICATE_OR_KERNEL_CATEGORIES,
        allowed_rejection_reasons: &["timeout"],
        allowed_stages: ADVANCED_STAGES,
        allowed_phases: CERTIFICATE_PHASES,
        requires_fast_profile: true,
    },
    RequiredSoundnessCase {
        key: "soundness.certificate.resource_exhaustion",
        domain: "certificate",
        expected_outcome: ExpectedOutcome::Fail,
        allowed_failure_categories: CERTIFICATE_OR_KERNEL_CATEGORIES,
        allowed_rejection_reasons: &["resource_exhaustion"],
        allowed_stages: ADVANCED_STAGES,
        allowed_phases: CERTIFICATE_PHASES,
        requires_fast_profile: true,
    },
    RequiredSoundnessCase {
        key: "soundness.cluster.infinite_chain",
        domain: "cluster",
        expected_outcome: ExpectedOutcome::Fail,
        allowed_failure_categories: CLUSTER_ERROR_CATEGORY,
        allowed_rejection_reasons: &["cluster_loop"],
        allowed_stages: ADVANCED_STAGES,
        allowed_phases: &[PipelinePhase::ClusterResolution],
        requires_fast_profile: true,
    },
    RequiredSoundnessCase {
        key: "soundness.cluster.cyclic_registration",
        domain: "cluster",
        expected_outcome: ExpectedOutcome::Fail,
        allowed_failure_categories: CLUSTER_ERROR_CATEGORY,
        allowed_rejection_reasons: &["cluster_loop"],
        allowed_stages: ADVANCED_STAGES,
        allowed_phases: &[PipelinePhase::ClusterResolution],
        requires_fast_profile: true,
    },
    RequiredSoundnessCase {
        key: "soundness.cluster.unintended_coercion",
        domain: "cluster",
        expected_outcome: ExpectedOutcome::Fail,
        allowed_failure_categories: CLUSTER_ERROR_CATEGORY,
        allowed_rejection_reasons: &["unintended_coercion"],
        allowed_stages: ADVANCED_STAGES,
        allowed_phases: &[PipelinePhase::ClusterResolution],
        requires_fast_profile: true,
    },
    RequiredSoundnessCase {
        key: "soundness.cluster.hidden_transitive_expansion",
        domain: "cluster",
        expected_outcome: ExpectedOutcome::Fail,
        allowed_failure_categories: CLUSTER_ERROR_CATEGORY,
        allowed_rejection_reasons: &["hidden_transitive_expansion"],
        allowed_stages: ADVANCED_STAGES,
        allowed_phases: &[PipelinePhase::ClusterResolution],
        requires_fast_profile: true,
    },
    RequiredSoundnessCase {
        key: "soundness.overload.ambiguous_notation",
        domain: "overload",
        expected_outcome: ExpectedOutcome::Fail,
        allowed_failure_categories: OVERLOAD_ERROR_CATEGORY,
        allowed_rejection_reasons: &["ambiguous_notation"],
        allowed_stages: ADVANCED_STAGES,
        allowed_phases: &[PipelinePhase::OverloadResolution],
        requires_fast_profile: true,
    },
    RequiredSoundnessCase {
        key: "soundness.overload.hidden_coercion",
        domain: "overload",
        expected_outcome: ExpectedOutcome::Fail,
        allowed_failure_categories: OVERLOAD_ERROR_CATEGORY,
        allowed_rejection_reasons: &["hidden_coercion"],
        allowed_stages: ADVANCED_STAGES,
        allowed_phases: &[PipelinePhase::OverloadResolution],
        requires_fast_profile: true,
    },
    RequiredSoundnessCase {
        key: "soundness.overload.unstable_resolution_order",
        domain: "overload",
        expected_outcome: ExpectedOutcome::Fail,
        allowed_failure_categories: OVERLOAD_ERROR_CATEGORY,
        allowed_rejection_reasons: &["unstable_resolution_order"],
        allowed_stages: ADVANCED_STAGES,
        allowed_phases: &[PipelinePhase::OverloadResolution],
        requires_fast_profile: true,
    },
    RequiredSoundnessCase {
        key: "soundness.dependency.stale_theorem_statement_fingerprint",
        domain: "dependency",
        expected_outcome: ExpectedOutcome::Fail,
        allowed_failure_categories: PROOF_CERT_OR_KERNEL_CATEGORIES,
        allowed_rejection_reasons: &["stale_theorem_statement_fingerprint"],
        allowed_stages: ADVANCED_STAGES,
        allowed_phases: PROOF_OR_KERNEL_PHASES,
        requires_fast_profile: true,
    },
    RequiredSoundnessCase {
        key: "soundness.dependency.stale_cluster_semantics",
        domain: "dependency",
        expected_outcome: ExpectedOutcome::Fail,
        allowed_failure_categories: PROOF_CERT_OR_KERNEL_CATEGORIES,
        allowed_rejection_reasons: &["stale_cluster_semantics"],
        allowed_stages: ADVANCED_STAGES,
        allowed_phases: PROOF_OR_KERNEL_PHASES,
        requires_fast_profile: true,
    },
    RequiredSoundnessCase {
        key: "soundness.dependency.stale_notation_parse_result",
        domain: "dependency",
        expected_outcome: ExpectedOutcome::Fail,
        allowed_failure_categories: PROOF_CERT_OR_KERNEL_CATEGORIES,
        allowed_rejection_reasons: &["stale_notation_parse_result"],
        allowed_stages: ADVANCED_STAGES,
        allowed_phases: PROOF_OR_KERNEL_PHASES,
        requires_fast_profile: true,
    },
    RequiredSoundnessCase {
        key: "soundness.policy.externally_attested_evidence_rejected",
        domain: "policy",
        expected_outcome: ExpectedOutcome::Fail,
        allowed_failure_categories: PROOF_CERT_OR_KERNEL_CATEGORIES,
        allowed_rejection_reasons: &["externally_attested_evidence_rejected"],
        allowed_stages: ADVANCED_STAGES,
        allowed_phases: PROOF_OR_KERNEL_PHASES,
        requires_fast_profile: true,
    },
];

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
    let (table, origin_table, token_tables) = toml_lite::parse_expectation_tables(content)
        .map_err(|error| format!("TOML parse error on line {}: {}", error.line, error.message))?;
    expectation_from_table(&table, origin_table.as_ref(), &token_tables)
}

fn expectation_from_table(
    table: &TomlTable,
    origin_table: Option<&TomlTable>,
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
    let profiles = if table.contains_key("profiles") {
        parse_non_empty_string_array(table, "profiles")?
    } else {
        vec!["fast".to_owned()]
    };
    validate_profile_ids(&profiles)?;
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
    let diagnostic_payloads = optional_string_array(table, "diagnostic_payloads")?
        .into_iter()
        .map(|diagnostic_payload| {
            if diagnostic_payload.is_empty() {
                Err("`diagnostic_payloads` entries must not be empty".to_owned())
            } else {
                Ok(diagnostic_payload)
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    let declaration_symbol_payloads = optional_string_array(table, "declaration_symbol_payloads")?
        .into_iter()
        .map(|payload| {
            if payload.is_empty() {
                Err("`declaration_symbol_payloads` entries must not be empty".to_owned())
            } else {
                Ok(payload)
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    let stable_detail_key = toml_lite::optional_string(table, "stable_detail_key")?;
    let notes = toml_lite::optional_string(table, "notes")?;
    let snapshots = toml_lite::optional_string(table, "snapshots")?
        .map(PathBuf::from)
        .map(|path| {
            if path.as_os_str().is_empty() {
                Err("`snapshots` must not be empty when present".to_owned())
            } else {
                Ok(path)
            }
        })
        .transpose()?;
    let ast_profile = toml_lite::optional_string(table, "ast_profile")?;
    let snapshot_profiles = parse_optional_non_empty_string_array(table, "snapshot_profiles")?;
    let tags = optional_string_array(table, "tags")?
        .into_iter()
        .map(|tag| {
            if tag.is_empty() {
                Err("`tags` entries must not be empty".to_owned())
            } else {
                Ok(tag)
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    for (field, value) in [
        ("failure_category", failure_category.as_deref()),
        ("rejection_reason", rejection_reason.as_deref()),
        ("stable_detail_key", stable_detail_key.as_deref()),
        ("notes", notes.as_deref()),
        ("ast_profile", ast_profile.as_deref()),
    ] {
        if value.is_some_and(str::is_empty) {
            return Err(format!("`{field}` must not be empty when present"));
        }
    }

    validate_kind_outcome(kind, expected_outcome)?;
    let origin = origin_table.map(parse_origin_metadata).transpose()?;
    validate_origin_metadata(kind, expected_outcome, origin.as_ref())?;
    if !tokens.is_empty() && stage != Stage::Lexical {
        return Err("token expectations are only valid for `stage = \"lexical\"`".to_owned());
    }
    let declaration_symbol_payloads_allowed = declaration_symbol_payloads.is_empty()
        || (stage == Stage::DeclarationSymbol
            && expected_outcome == ExpectedOutcome::Pass
            && expected_phase == Some(PipelinePhase::Resolve)
            && tags.iter().any(|tag| tag == "active_declaration_symbol")
            && source
                .extension()
                .is_some_and(|extension| extension == "miz"));
    if !declaration_symbol_payloads_allowed {
        return Err(
            "`declaration_symbol_payloads` is only valid for active declaration_symbol pass expectations"
                .to_owned(),
        );
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
    let architecture22 = parse_architecture22_metadata(table)?;

    Ok(Expectation {
        schema_version,
        id,
        kind,
        stage,
        domain,
        source,
        expected_outcome,
        spec_refs,
        profiles,
        expected_phase,
        failure_category,
        rejection_reason,
        diagnostic_codes,
        diagnostic_payloads,
        declaration_symbol_payloads,
        stable_detail_key,
        tags,
        notes,
        snapshots,
        ast_profile,
        snapshot_profiles,
        tokens,
        origin,
        architecture22,
    })
}

fn parse_architecture22_metadata(
    table: &TomlTable,
) -> Result<Option<Architecture22Metadata>, String> {
    let has_scenarios = table.contains_key("architecture22_scenarios");
    let has_equivalence_class = table.contains_key("architecture22_equivalence_class");
    let has_gate = table.contains_key("architecture22_gate");

    if !has_scenarios {
        if has_equivalence_class {
            return Err(
                "`architecture22_equivalence_class` requires `architecture22_scenarios`".to_owned(),
            );
        }
        if has_gate {
            return Err("`architecture22_gate` requires `architecture22_scenarios`".to_owned());
        }
        return Ok(None);
    }

    let scenarios = parse_non_empty_string_array(table, "architecture22_scenarios")?;
    validate_architecture22_scenarios(&scenarios)?;

    let equivalence_class = toml_lite::optional_string(table, "architecture22_equivalence_class")?;
    if let Some(equivalence_class) = &equivalence_class {
        if equivalence_class.is_empty() {
            return Err(
                "`architecture22_equivalence_class` must not be empty when present".to_owned(),
            );
        }
        if !architecture22_equivalence_class_is_known(equivalence_class) {
            return Err(format!(
                "unknown architecture22_equivalence_class `{equivalence_class}`"
            ));
        }
        for scenario in &scenarios {
            let spec = architecture22_scenario_spec(scenario)
                .expect("architecture22 scenario was validated as known");
            if spec.equivalence_class != equivalence_class {
                return Err(format!(
                    "`architecture22_equivalence_class` `{equivalence_class}` does not match scenario `{}` registry class `{}`",
                    spec.id, spec.equivalence_class
                ));
            }
        }
    }

    let gate = toml_lite::optional_string(table, "architecture22_gate")?
        .map(|gate| {
            if gate.is_empty() {
                Err("`architecture22_gate` must not be empty when present".to_owned())
            } else {
                gate.parse()
            }
        })
        .transpose()?
        .unwrap_or(Architecture22Gate::Planned);
    validate_architecture22_gate(gate, &scenarios)?;

    Ok(Some(Architecture22Metadata {
        scenarios,
        equivalence_class,
        gate,
    }))
}

fn validate_architecture22_scenarios(scenarios: &[String]) -> Result<(), String> {
    let mut seen = BTreeSet::new();
    let mut previous: Option<&str> = None;
    for scenario in scenarios {
        if architecture22_scenario_spec(scenario).is_none() {
            return Err(format!(
                "unknown architecture22_scenarios entry `{scenario}`"
            ));
        }
        if !seen.insert(scenario.as_str()) {
            return Err(format!(
                "duplicate architecture22_scenarios entry `{scenario}`"
            ));
        }
        if let Some(previous) = previous
            && previous > scenario.as_str()
        {
            return Err(format!(
                "architecture22_scenarios entry `{scenario}` must be sorted after `{previous}`"
            ));
        }
        previous = Some(scenario);
    }
    Ok(())
}

fn architecture22_equivalence_class_is_known(equivalence_class: &str) -> bool {
    ARCHITECTURE22_SCENARIOS
        .iter()
        .any(|scenario| scenario.equivalence_class == equivalence_class)
}

fn validate_architecture22_gate(
    gate: Architecture22Gate,
    scenarios: &[String],
) -> Result<(), String> {
    if gate == Architecture22Gate::Planned {
        return Ok(());
    }

    let ineligible = scenarios
        .iter()
        .filter(|scenario| {
            architecture22_scenario_spec(scenario)
                .expect("architecture22 scenario was validated as known")
                .active_eligibility
                .is_none()
        })
        .map(String::as_str)
        .collect::<Vec<_>>();
    if ineligible.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "`architecture22_gate = \"active\"` is not allowed for scenarios without active eligibility: {}",
            ineligible.join(", ")
        ))
    }
}

fn parse_origin_metadata(table: &TomlTable) -> Result<OriginMetadata, String> {
    validate_origin_fields(table)?;
    let schema_version = toml_lite::required_u32(table, "schema_version")?;
    if schema_version != 1 {
        return Err(format!(
            "unsupported origin.schema_version `{schema_version}`"
        ));
    }
    let kind: TestKind = toml_lite::required_string(table, "kind")?.parse()?;
    if !matches!(
        kind,
        TestKind::Generated | TestKind::FuzzSeed | TestKind::PropertySeed
    ) {
        return Err("`origin.kind` must be generated, fuzz_seed, or property_seed".to_owned());
    }
    let generator = required_non_empty_string(table, "generator")?;
    let generator_version = required_non_empty_string(table, "generator_version")?;
    let seed = required_non_empty_string(table, "seed")?;
    let profile = required_non_empty_string(table, "profile")?;
    let expected_outcome = toml_lite::required_string(table, "expected_outcome")?.parse()?;
    let minimized = toml_lite::required_bool(table, "minimized")?;
    let original_failure_category = toml_lite::optional_string(table, "original_failure_category")?;
    if original_failure_category
        .as_deref()
        .is_some_and(str::is_empty)
    {
        return Err("`origin.original_failure_category` must not be empty when present".to_owned());
    }
    Ok(OriginMetadata {
        schema_version,
        kind,
        generator,
        generator_version,
        seed,
        profile,
        expected_outcome,
        minimized,
        original_failure_category,
    })
}

fn required_non_empty_string(table: &TomlTable, key: &str) -> Result<String, String> {
    let value = toml_lite::required_string(table, key)?;
    if value.is_empty() {
        Err(format!("`origin.{key}` must not be empty"))
    } else {
        Ok(value)
    }
}

fn validate_origin_metadata(
    kind: TestKind,
    expected_outcome: ExpectedOutcome,
    origin: Option<&OriginMetadata>,
) -> Result<(), String> {
    match kind {
        TestKind::Generated | TestKind::FuzzSeed | TestKind::PropertySeed => {
            let Some(origin) = origin else {
                return Err(format!("kind `{kind}` requires `[origin]` metadata"));
            };
            if origin.kind != kind {
                return Err(format!(
                    "`origin.kind` `{}` must match sidecar kind `{kind}`",
                    origin.kind
                ));
            }
            if origin.expected_outcome != expected_outcome {
                return Err(format!(
                    "`origin.expected_outcome` `{}` must match sidecar expected_outcome `{expected_outcome}`",
                    origin.expected_outcome
                ));
            }
            Ok(())
        }
        TestKind::Pass | TestKind::Fail | TestKind::Snapshot => {
            if origin.is_some() {
                Err("`[origin]` metadata is only valid for generated, fuzz_seed, or property_seed cases".to_owned())
            } else {
                Ok(())
            }
        }
    }
}

fn optional_string_array(table: &TomlTable, key: &str) -> Result<Vec<String>, String> {
    if table.contains_key(key) {
        toml_lite::string_array(table, key)
    } else {
        Ok(Vec::new())
    }
}

fn parse_non_empty_string_array(table: &TomlTable, key: &str) -> Result<Vec<String>, String> {
    let values = toml_lite::string_array(table, key)?;
    if values.is_empty() {
        return Err(format!("`{key}` must not be empty when present"));
    }
    values
        .into_iter()
        .map(|value| {
            if value.is_empty() {
                Err(format!("`{key}` entries must not be empty"))
            } else {
                Ok(value)
            }
        })
        .collect()
}

fn validate_profile_ids(profiles: &[String]) -> Result<(), String> {
    for profile in profiles {
        if !matches!(
            profile.as_str(),
            "fast"
                | "full"
                | "stress"
                | "fuzz_regression"
                | "fuzz-regression"
                | "snapshot_update"
                | "snapshot-update"
        ) {
            return Err(format!("unknown profile `{profile}`"));
        }
    }
    Ok(())
}

fn parse_optional_non_empty_string_array(
    table: &TomlTable,
    key: &str,
) -> Result<Vec<String>, String> {
    if table.contains_key(key) {
        parse_non_empty_string_array(table, key)
    } else {
        Ok(Vec::new())
    }
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
        let span_start = toml_lite::optional_u32(table, "span_start")?;
        let span_end = toml_lite::optional_u32(table, "span_end")?;
        let span_start_line = toml_lite::optional_u32(table, "span_start_line")?;
        let span_start_col = toml_lite::optional_u32(table, "span_start_col")?;
        let span_end_line = toml_lite::optional_u32(table, "span_end_line")?;
        let span_end_col = toml_lite::optional_u32(table, "span_end_col")?;
        if span_start.is_some() != span_end.is_some() {
            return Err(
                "`tokens.span_start` and `tokens.span_end` must be provided together".to_owned(),
            );
        }
        let line_col_fields = [span_start_line, span_start_col, span_end_line, span_end_col];
        if line_col_fields.iter().any(Option::is_some)
            && !line_col_fields.iter().all(Option::is_some)
        {
            return Err(
                "`tokens.span_*_line` and `tokens.span_*_col` must be provided together".to_owned(),
            );
        }
        if span_start.is_some() && span_start_line.is_some() {
            return Err(
                "`tokens.span_start`/`span_end` and line/column spans are mutually exclusive"
                    .to_owned(),
            );
        }
        if let (Some(start), Some(end)) = (span_start, span_end)
            && start > end
        {
            return Err("`tokens.span_start` must not exceed `tokens.span_end`".to_owned());
        }
        if let (Some(start_line), Some(start_col), Some(end_line), Some(end_col)) =
            (span_start_line, span_start_col, span_end_line, span_end_col)
        {
            if start_line == 0 || start_col == 0 || end_line == 0 || end_col == 0 {
                return Err("line/column spans are 1-based and must be non-zero".to_owned());
            }
            if (start_line, start_col) > (end_line, end_col) {
                return Err(
                    "`tokens.span_start_line`/`span_start_col` must not exceed end line/column"
                        .to_owned(),
                );
            }
        }
        tokens.push(TokenExpectation {
            kind,
            lexeme,
            span_start,
            span_end,
            span_start_line,
            span_start_col,
            span_end_line,
            span_end_col,
        });
    }
    Ok(tokens)
}

fn validate_token_fields(table: &TomlTable) -> Result<(), String> {
    const KNOWN_TOKEN_FIELDS: &[&str] = &[
        "kind",
        "lexeme",
        "span_start",
        "span_end",
        "span_start_line",
        "span_start_col",
        "span_end_line",
        "span_end_col",
    ];

    for key in table.keys() {
        if !KNOWN_TOKEN_FIELDS.contains(&key.as_str()) {
            return Err(format!("unknown token expectation field `{key}`"));
        }
    }
    Ok(())
}

fn validate_origin_fields(table: &TomlTable) -> Result<(), String> {
    const KNOWN_ORIGIN_FIELDS: &[&str] = &[
        "schema_version",
        "kind",
        "generator",
        "generator_version",
        "seed",
        "profile",
        "expected_outcome",
        "minimized",
        "original_failure_category",
    ];

    for key in table.keys() {
        if !KNOWN_ORIGIN_FIELDS.contains(&key.as_str()) {
            return Err(format!("unknown origin field `{key}`"));
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
        "diagnostic_payloads",
        "declaration_symbol_payloads",
        "snapshots",
        "failure_category",
        "rejection_reason",
        "stable_detail_key",
        "ast_profile",
        "snapshot_profiles",
        "architecture22_scenarios",
        "architecture22_equivalence_class",
        "architecture22_gate",
    ];

    for key in table.keys() {
        if !KNOWN_FIELDS.contains(&key.as_str()) {
            return Err(format!("unknown expectation field `{key}`"));
        }
    }
    Ok(())
}

pub fn validate_expectation_path(
    path: &Path,
    expectation: &Expectation,
    tests_root: &Path,
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

    if let Some(snapshot_path) = &expectation.snapshots {
        if !clean_relative_path(snapshot_path) || !snapshot_path.starts_with("snapshots") {
            diagnostics.push(ValidationDiagnostic::error(
                path,
                "expectation",
                "E-EXPECT-SNAPSHOT-PATH",
                "expectation.snapshots",
                format!(
                    "snapshots `{}` must be a clean tests-root-relative path under snapshots/",
                    snapshot_path.display()
                ),
            ));
        }
        if snapshot_path
            .extension()
            .and_then(|extension| extension.to_str())
            != Some("snap")
        {
            diagnostics.push(ValidationDiagnostic::error(
                path,
                "expectation",
                "E-EXPECT-SNAPSHOT-EXTENSION",
                "expectation.snapshots",
                format!(
                    "snapshots `{}` must use the .snap extension",
                    snapshot_path.display()
                ),
            ));
        }
        let active_parse_only = expectation.stage == Stage::ParseOnly
            && expectation.expected_phase == Some(PipelinePhase::Parse)
            && matches!(
                expectation.expected_outcome,
                ExpectedOutcome::Pass | ExpectedOutcome::Fail
            )
            && expectation
                .tags
                .iter()
                .any(|tag| tag == "active_parse_only");
        if !active_parse_only {
            diagnostics.push(ValidationDiagnostic::error(
                path,
                "expectation",
                "E-EXPECT-SNAPSHOT-SCOPE",
                "expectation.snapshots",
                "snapshots are currently supported only for active parse-only pass/fail cases",
            ));
        }
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

    validate_fail_soundness_contract(path, expectation, &mut diagnostics);
    validate_corpus_policy(path, expectation, tests_root, &mut diagnostics);

    diagnostics
}

fn validate_corpus_policy(
    path: &Path,
    expectation: &Expectation,
    tests_root: &Path,
    diagnostics: &mut Vec<ValidationDiagnostic>,
) {
    let relative_path = path.strip_prefix(tests_root).ok();
    let root = relative_path.and_then(test_root_class);
    validate_corpus_class_placement(path, expectation, root, diagnostics);
    validate_corpus_profiles(path, expectation, root, diagnostics);
    validate_corpus_naming(path, expectation, relative_path, diagnostics);
    validate_corpus_file_size(path, expectation, root, diagnostics);
}

fn validate_corpus_class_placement(
    path: &Path,
    expectation: &Expectation,
    root: Option<&str>,
    diagnostics: &mut Vec<ValidationDiagnostic>,
) {
    let expected_root = match expectation.kind {
        TestKind::Generated => Some("generated"),
        TestKind::FuzzSeed => Some("fuzz"),
        TestKind::PropertySeed => Some("property"),
        TestKind::Pass | TestKind::Fail | TestKind::Snapshot => None,
    };

    if let Some(expected_root) = expected_root {
        let placed = root == Some(expected_root)
            || (expectation.kind == TestKind::Generated && root == Some("stress"));
        if !placed {
            diagnostics.push(ValidationDiagnostic::error(
                path,
                "corpus",
                "E-CORPUS-PLACEMENT",
                format!("corpus.placement.{}", expectation.id.0),
                format!(
                    "kind `{}` must be placed under tests/{expected_root}/{}",
                    expectation.kind,
                    if expectation.kind == TestKind::Generated {
                        " or tests/stress/"
                    } else {
                        ""
                    }
                ),
            ));
        }
    }

    for (root_name, expected_kind) in [
        ("generated", TestKind::Generated),
        ("fuzz", TestKind::FuzzSeed),
        ("property", TestKind::PropertySeed),
    ] {
        if root == Some(root_name) && expectation.kind != expected_kind {
            diagnostics.push(ValidationDiagnostic::error(
                path,
                "corpus",
                "E-CORPUS-PLACEMENT",
                format!("corpus.placement.{}", expectation.id.0),
                format!("tests/{root_name}/ sidecars must use kind `{expected_kind}`"),
            ));
        }
    }
}

fn validate_corpus_profiles(
    path: &Path,
    expectation: &Expectation,
    root: Option<&str>,
    diagnostics: &mut Vec<ValidationDiagnostic>,
) {
    let has_fast = has_profile(&expectation.profiles, "fast");
    let has_stress = has_profile(&expectation.profiles, "stress");
    if has_fast && has_stress {
        diagnostics.push(ValidationDiagnostic::error(
            path,
            "corpus",
            "E-CORPUS-STRESS-FAST-PROFILE",
            format!("corpus.profile.{}", expectation.id.0),
            "stress cases must stay outside the default fast profile",
        ));
    }
    if root == Some("stress") && !has_stress {
        diagnostics.push(ValidationDiagnostic::error(
            path,
            "corpus",
            "E-CORPUS-STRESS-PROFILE",
            format!("corpus.profile.{}", expectation.id.0),
            "tests/stress/ sidecars must include the stress profile",
        ));
    }

    let Some(origin) = expectation.origin.as_ref() else {
        return;
    };
    if !origin.minimized && has_fast {
        diagnostics.push(ValidationDiagnostic::error(
            path,
            "corpus",
            "E-CORPUS-UNMINIMIZED-FAST",
            format!("corpus.profile.{}", expectation.id.0),
            "unminimized generated, fuzz, and property seeds must stay outside the default fast profile",
        ));
    }
    if expectation.kind == TestKind::FuzzSeed
        && expectation.expected_outcome == ExpectedOutcome::MetadataOnly
        && !has_profile(&expectation.profiles, "fuzz_regression")
    {
        diagnostics.push(ValidationDiagnostic::error(
            path,
            "corpus",
            "E-CORPUS-FUZZ-PROFILE",
            format!("corpus.profile.{}", expectation.id.0),
            "metadata-only fuzz seeds must be gated by the fuzz_regression profile",
        ));
    }
    if expectation.kind == TestKind::FuzzSeed {
        match (
            origin.original_failure_category.as_deref(),
            expectation.failure_category.as_deref(),
        ) {
            (None, _) => diagnostics.push(ValidationDiagnostic::error(
                path,
                "corpus",
                "E-CORPUS-FUZZ-CATEGORY",
                format!("corpus.fuzz_category.{}", expectation.id.0),
                "fuzz seeds must preserve origin.original_failure_category",
            )),
            (Some(original), Some(current)) if original != current => {
                diagnostics.push(ValidationDiagnostic::error(
                    path,
                    "corpus",
                    "E-CORPUS-FUZZ-CATEGORY",
                    format!("corpus.fuzz_category.{}", expectation.id.0),
                    format!(
                        "fuzz failure_category `{current}` must match original failure category `{original}`"
                    ),
                ));
            }
            _ => {}
        }
    }
}

fn validate_corpus_naming(
    path: &Path,
    expectation: &Expectation,
    relative_path: Option<&Path>,
    diagnostics: &mut Vec<ValidationDiagnostic>,
) {
    let stem = expectation_stem(path).unwrap_or_default();
    if !is_snake_case_stem(&stem) {
        diagnostics.push(ValidationDiagnostic::warning(
            path,
            "corpus",
            "W-CORPUS-NAMING",
            format!("corpus.naming.{}", expectation.id.0),
            "corpus sidecar names should use stable snake_case stems",
        ));
    }
    if matches!(
        expectation.expected_outcome,
        ExpectedOutcome::Pass | ExpectedOutcome::Fail | ExpectedOutcome::Snapshot
    ) && !has_numeric_suffix(&stem)
    {
        diagnostics.push(ValidationDiagnostic::warning(
            path,
            "corpus",
            "W-CORPUS-NAMING",
            format!("corpus.naming.{}", expectation.id.0),
            "executable corpus sidecars should end with a stable numeric suffix",
        ));
    }
    if relative_path.is_some_and(|path| path_has_component(path, "pass"))
        && expectation.expected_outcome == ExpectedOutcome::Fail
    {
        diagnostics.push(ValidationDiagnostic::error(
            path,
            "corpus",
            "E-CORPUS-OUTCOME-PLACEMENT",
            format!("corpus.placement.{}", expectation.id.0),
            "fail expectations must not be placed under a pass corpus directory",
        ));
    }
    if relative_path.is_some_and(|path| path_has_component(path, "fail"))
        && expectation.expected_outcome == ExpectedOutcome::Pass
    {
        diagnostics.push(ValidationDiagnostic::error(
            path,
            "corpus",
            "E-CORPUS-OUTCOME-PLACEMENT",
            format!("corpus.placement.{}", expectation.id.0),
            "pass expectations must not be placed under a fail corpus directory",
        ));
    }
    let expected_prefix = match expectation.expected_outcome {
        ExpectedOutcome::Pass => Some("pass_"),
        ExpectedOutcome::Fail => Some("fail_"),
        ExpectedOutcome::Snapshot | ExpectedOutcome::MetadataOnly => None,
    };
    let Some(expected_prefix) = expected_prefix else {
        return;
    };
    if !stem.starts_with(expected_prefix)
        && relative_path
            .is_some_and(|path| path_has_component(path, expected_prefix.trim_end_matches('_')))
    {
        diagnostics.push(ValidationDiagnostic::warning(
            path,
            "corpus",
            "W-CORPUS-NAMING",
            format!("corpus.naming.{}", expectation.id.0),
            format!(
                "sidecars under a `{}` corpus directory should use the `{expected_prefix}` name prefix",
                expected_prefix.trim_end_matches('_')
            ),
        ));
    }
}

fn validate_corpus_file_size(
    path: &Path,
    expectation: &Expectation,
    root: Option<&str>,
    diagnostics: &mut Vec<ValidationDiagnostic>,
) {
    if expectation
        .source
        .extension()
        .and_then(|extension| extension.to_str())
        != Some("miz")
    {
        return;
    }
    let source_path = path
        .parent()
        .unwrap_or_else(|| Path::new(""))
        .join(&expectation.source);
    let Ok(content) = fs::read_to_string(&source_path) else {
        return;
    };
    let Some(guideline) = corpus_size_guideline(expectation, root) else {
        return;
    };
    let line_count = content.lines().count();
    if line_count >= guideline.min_lines && line_count <= guideline.max_lines {
        return;
    }
    if line_count <= guideline.max_lines {
        return;
    }

    if expectation.kind == TestKind::Generated && root != Some("stress") {
        diagnostics.push(ValidationDiagnostic::error(
            path,
            "corpus",
            "E-CORPUS-GENERATED-SIZE",
            format!("corpus.size.{}", expectation.id.0),
            format!(
                "generated .miz source has {line_count} lines, above the {} guideline of {}-{} lines; oversized generated cases must live under tests/stress/",
                guideline.label, guideline.min_lines, guideline.max_lines
            ),
        ));
    } else {
        diagnostics.push(ValidationDiagnostic::warning(
            path,
            "corpus",
            "W-CORPUS-SIZE",
            format!("corpus.size.{}", expectation.id.0),
            format!(
                ".miz source has {line_count} lines, above the {} guideline of {}-{} lines",
                guideline.label, guideline.min_lines, guideline.max_lines
            ),
        ));
    }
}

#[derive(Debug, Clone, Copy)]
struct CorpusSizeGuideline {
    label: &'static str,
    min_lines: usize,
    max_lines: usize,
}

fn corpus_size_guideline(
    expectation: &Expectation,
    root: Option<&str>,
) -> Option<CorpusSizeGuideline> {
    if root == Some("stress") {
        return Some(CorpusSizeGuideline {
            label: "stress test",
            min_lines: 500,
            max_lines: 1000,
        });
    }
    if expectation.domain.contains("integration") {
        return Some(CorpusSizeGuideline {
            label: "integration test",
            min_lines: 100,
            max_lines: 300,
        });
    }
    if expectation.domain.contains("cluster")
        || expectation.expected_phase == Some(PipelinePhase::ClusterResolution)
    {
        return Some(CorpusSizeGuideline {
            label: "cluster test",
            min_lines: 20,
            max_lines: 80,
        });
    }
    if matches!(
        expectation.expected_phase,
        Some(PipelinePhase::TypeCheck | PipelinePhase::Elaboration)
    ) {
        return Some(CorpusSizeGuideline {
            label: "type test",
            min_lines: 10,
            max_lines: 50,
        });
    }
    if matches!(
        expectation.stage,
        Stage::ProofVerification | Stage::AdvancedSemantics
    ) || matches!(
        expectation.expected_phase,
        Some(
            PipelinePhase::StatementCheck
                | PipelinePhase::VcGeneration
                | PipelinePhase::Verification
                | PipelinePhase::CertificateCheck
                | PipelinePhase::KernelCheck
        )
    ) {
        return Some(CorpusSizeGuideline {
            label: "theorem test",
            min_lines: 30,
            max_lines: 150,
        });
    }
    if expectation.stage == Stage::ParseOnly
        || expectation.expected_phase == Some(PipelinePhase::Parse)
    {
        return Some(CorpusSizeGuideline {
            label: "parser test",
            min_lines: 5,
            max_lines: 30,
        });
    }
    None
}

fn test_root_class(path: &Path) -> Option<&str> {
    path.components().next()?.as_os_str().to_str()
}

fn path_has_component(path: &Path, needle: &str) -> bool {
    path.components()
        .any(|component| component.as_os_str() == needle)
}

fn is_snake_case_stem(stem: &str) -> bool {
    !stem.is_empty()
        && !stem.starts_with('_')
        && !stem.ends_with('_')
        && !stem.contains("__")
        && stem
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

fn has_numeric_suffix(stem: &str) -> bool {
    stem.rsplit_once('_').is_some_and(|(_, suffix)| {
        !suffix.is_empty() && suffix.bytes().all(|byte| byte.is_ascii_digit())
    })
}

fn has_profile(profiles: &[String], profile: &str) -> bool {
    profiles.iter().any(|candidate| {
        candidate == profile
            || (profile == "fuzz_regression" && candidate == "fuzz-regression")
            || (profile == "snapshot_update" && candidate == "snapshot-update")
    })
}

pub(crate) fn required_soundness_case_for(
    expectation: &Expectation,
) -> Option<&'static RequiredSoundnessCase> {
    expectation
        .stable_detail_key
        .as_deref()
        .and_then(required_soundness_case_by_key)
}

pub(crate) fn required_soundness_case_by_key(key: &str) -> Option<&'static RequiredSoundnessCase> {
    REQUIRED_SOUNDNESS_CASES
        .iter()
        .find(|required_case| required_case.key == key)
}

fn validate_fail_soundness_contract(
    path: &Path,
    expectation: &Expectation,
    diagnostics: &mut Vec<ValidationDiagnostic>,
) {
    if expectation.expected_outcome == ExpectedOutcome::Fail
        && certificate_or_kernel_rejection(expectation)
        && expectation.rejection_reason.is_none()
    {
        diagnostics.push(ValidationDiagnostic::error(
            path,
            "expectation",
            "E-EXPECT-REJECTION-REASON",
            "expectation.rejection_reason",
            "certificate and kernel rejections require `rejection_reason`",
        ));
    }

    let Some(stable_detail_key) = expectation.stable_detail_key.as_deref() else {
        return;
    };
    if stable_detail_key.starts_with("soundness.")
        && required_soundness_case_by_key(stable_detail_key).is_none()
    {
        diagnostics.push(ValidationDiagnostic::error(
            path,
            "expectation",
            "E-EXPECT-SOUNDNESS-CASE",
            format!("expectation.soundness_case.{stable_detail_key}"),
            format!(
                "soundness stable_detail_key `{stable_detail_key}` is not a required fail/soundness case"
            ),
        ));
        return;
    }

    let Some(required_case) = required_soundness_case_by_key(stable_detail_key) else {
        return;
    };
    if expectation.domain != required_case.domain {
        diagnostics.push(ValidationDiagnostic::error(
            path,
            "expectation",
            "E-EXPECT-SOUNDNESS-DOMAIN",
            format!("expectation.soundness_domain.{}", required_case.key),
            format!(
                "soundness case `{}` must use domain `{}`",
                required_case.key, required_case.domain
            ),
        ));
    }
    if expectation.expected_outcome != required_case.expected_outcome {
        diagnostics.push(ValidationDiagnostic::error(
            path,
            "expectation",
            "E-EXPECT-SOUNDNESS-OUTCOME",
            format!("expectation.soundness_outcome.{}", required_case.key),
            format!(
                "soundness case `{}` must expect `{}`",
                required_case.key, required_case.expected_outcome
            ),
        ));
    }
    if expectation.expected_outcome == ExpectedOutcome::Fail {
        if !required_case.allowed_failure_categories.is_empty()
            && !expectation
                .failure_category
                .as_deref()
                .is_some_and(|category| {
                    required_case.allowed_failure_categories.contains(&category)
                })
        {
            diagnostics.push(ValidationDiagnostic::error(
                path,
                "expectation",
                "E-EXPECT-SOUNDNESS-CATEGORY",
                format!("expectation.soundness_category.{}", required_case.key),
                format!(
                    "soundness case `{}` must use one of these failure categories: {}",
                    required_case.key,
                    string_list(required_case.allowed_failure_categories)
                ),
            ));
        }
        if !required_case.allowed_rejection_reasons.is_empty()
            && !expectation
                .rejection_reason
                .as_deref()
                .is_some_and(|reason| required_case.allowed_rejection_reasons.contains(&reason))
        {
            diagnostics.push(ValidationDiagnostic::error(
                path,
                "expectation",
                "E-EXPECT-SOUNDNESS-REJECTION-REASON",
                format!(
                    "expectation.soundness_rejection_reason.{}",
                    required_case.key
                ),
                format!(
                    "soundness case `{}` must use one of these rejection reasons: {}",
                    required_case.key,
                    string_list(required_case.allowed_rejection_reasons)
                ),
            ));
        }
    }
    if !required_case.allowed_stages.contains(&expectation.stage) {
        diagnostics.push(ValidationDiagnostic::error(
            path,
            "expectation",
            "E-EXPECT-SOUNDNESS-STAGE",
            format!("expectation.soundness_stage.{}", required_case.key),
            format!(
                "soundness case `{}` must use one of these stages: {}",
                required_case.key,
                stage_list(required_case.allowed_stages)
            ),
        ));
    }
    if !expectation
        .expected_phase
        .is_some_and(|phase| required_case.allowed_phases.contains(&phase))
    {
        diagnostics.push(ValidationDiagnostic::error(
            path,
            "expectation",
            "E-EXPECT-SOUNDNESS-PHASE",
            format!("expectation.soundness_phase.{}", required_case.key),
            format!(
                "soundness case `{}` must use one of these expected phases: {}",
                required_case.key,
                phase_list(required_case.allowed_phases)
            ),
        ));
    }
    if required_case.requires_fast_profile
        && !expectation.profiles.iter().any(|profile| profile == "fast")
    {
        diagnostics.push(ValidationDiagnostic::error(
            path,
            "expectation",
            "E-EXPECT-SOUNDNESS-PROFILE",
            format!("expectation.soundness_profile.{}", required_case.key),
            format!(
                "soundness case `{}` must stay in the default fast profile",
                required_case.key
            ),
        ));
    }
}

fn certificate_or_kernel_rejection(expectation: &Expectation) -> bool {
    matches!(
        expectation.expected_phase,
        Some(PipelinePhase::CertificateCheck | PipelinePhase::KernelCheck)
    ) || matches!(
        expectation.failure_category.as_deref(),
        Some("certificate_rejection" | "kernel_rejection")
    )
}

fn stage_list(stages: &[Stage]) -> String {
    stages
        .iter()
        .map(|stage| stage.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

fn phase_list(phases: &[PipelinePhase]) -> String {
    phases
        .iter()
        .map(|phase| phase.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

fn string_list(values: &[&str]) -> String {
    values.join(", ")
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

impl fmt::Display for Architecture22Gate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
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

impl FromStr for Architecture22Gate {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "planned" => Ok(Self::Planned),
            "active" => Ok(Self::Active),
            other => Err(format!("unknown architecture22_gate `{other}`")),
        }
    }
}

impl Architecture22Gate {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Active => "active",
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

impl PipelinePhase {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Lex => "lex",
            Self::Parse => "parse",
            Self::Resolve => "resolve",
            Self::TypeCheck => "type_check",
            Self::Elaboration => "elaboration",
            Self::ClusterResolution => "cluster_resolution",
            Self::OverloadResolution => "overload_resolution",
            Self::StatementCheck => "statement_check",
            Self::VcGeneration => "vc_generation",
            Self::Verification => "verification",
            Self::CertificateCheck => "certificate_check",
            Self::KernelCheck => "kernel_check",
        }
    }
}
