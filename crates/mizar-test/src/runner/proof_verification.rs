use std::fs;
use std::path::Path;

use mizar_core::elaborator::{SourceStructureCoreNormalizer, SourceStructureCoreReceipt};
use mizar_vc::generator::{ExactTask180VcInput, generate_exact_task180_vc};
use mizar_vc::vc_ir::{GenerationSchemaVersion, VcSchemaVersion, VcSet};

use crate::diagnostic::ValidationDiagnostic;
use crate::expectation::{ExpectedOutcome, PipelinePhase};
use crate::harness::{TestCase, TestPlan};
use crate::staged_model::Stage;

use super::import_fixtures::augment_type_elaboration_import_summaries;
use super::shared::{resolver_symbol_collection, run_frontend, snapshot_id};
use super::syntax_smoke::workspace_relative_source;
use super::type_elaboration::source_contradiction_core_ir;
use super::type_elaboration::source_structure_semantics::source_structure_semantics_output;
use super::{ProofVerificationCaseResult, ProofVerificationCaseStatus};

const ACTIVE_PROOF_VERIFICATION_TAG: &str = "active_proof_verification";
const EXACT_TASK180_CASE_ID: &str = "pass_proof_verification_contradiction_formula_constant_001";
pub(in crate::runner) const STEP5C2_PROOF_CASES: [(&str, &str); 2] = [
    (
        "pass_proof_verification_struct_constructor_access_001",
        "tests/miz/pass/structures/pass_proof_verification_struct_constructor_access_001.miz",
    ),
    (
        "pass_proof_verification_struct_with_update_001",
        "tests/miz/pass/structures/pass_proof_verification_struct_with_update_001.miz",
    ),
];
const GENERATION_SCHEMA: &str = "mizar-vc-generation-task31-v1";
const VC_SCHEMA: &str = "mizar-vc-vcset-task31-v1";

pub(super) fn is_active_proof_verification(case: &TestCase) -> bool {
    let task180 = case.id.0 == EXACT_TASK180_CASE_ID
        && active_tag_count(case) == 1
        && case.expectation.stage == Stage::ProofVerification
        && case.expectation.expected_phase == Some(PipelinePhase::VcGeneration)
        && case.expectation.expected_outcome == ExpectedOutcome::Pass
        && case.expectation.snapshots.is_some();
    let step5c2 = step5c2_proof_case(case).is_some()
        && case.expectation.tags.as_slice() == [ACTIVE_PROOF_VERIFICATION_TAG]
        && case.expectation.snapshots.is_none();
    (task180 || step5c2)
        && case
            .source_path
            .extension()
            .is_some_and(|extension| extension == "miz")
}

pub(super) fn validate_active_proof_verification_tags(
    workspace_root: &Path,
    plan: &TestPlan,
) -> Vec<ValidationDiagnostic> {
    let reserved_cases = plan
        .cases
        .iter()
        .filter(|case| {
            case.id.0 == EXACT_TASK180_CASE_ID
                || is_step5c2_proof_id(case)
                || case
                    .expectation
                    .tags
                    .iter()
                    .any(|tag| tag == ACTIVE_PROOF_VERIFICATION_TAG)
        })
        .collect::<Vec<_>>();
    let mut diagnostics = Vec::new();
    for case in reserved_cases {
        if !is_active_proof_verification(case)
            || is_step5c2_proof_id(case) && !is_step5c2_proof_workspace_member(workspace_root, case)
        {
            diagnostics.push(ValidationDiagnostic::error(
                &case.expectation_path,
                "proof_verification",
                "E-PROOF-VERIFICATION-ACTIVE-GATE",
                format!("proof_verification.active_gate.{}", case.id.0),
                "proof verification admits only exact .miz pass expectations: Task-180 requires its VcIr snapshot and Step 5C.2 structure cases require no snapshot",
            ));
        }
        if !case.expectation.diagnostic_codes.is_empty() {
            diagnostics.push(ValidationDiagnostic::error(
                &case.expectation_path,
                "proof_verification",
                "E-PROOF-VERIFICATION-PUBLIC-DIAGNOSTIC-CODES",
                format!("proof_verification.public_codes.{}", case.id.0),
                "active_proof_verification cases must keep diagnostic_codes empty until public proof diagnostic codes are specified",
            ));
        }
    }
    if STEP5C2_PROOF_CASES
        .iter()
        .any(|(_, source)| workspace_root.join(source).is_file())
    {
        for (id, source) in STEP5C2_PROOF_CASES {
            let count = plan
                .cases
                .iter()
                .filter(|case| {
                    case.id.0 == id
                        && workspace_relative_source(workspace_root, &case.source_path)
                            .is_some_and(|actual| actual == source)
                })
                .count();
            if count != 1 {
                diagnostics.push(ValidationDiagnostic::error(
                    Path::new(source),
                    "proof_verification",
                    "E-PROOF-VERIFICATION-STEP5C2-INVENTORY",
                    format!("proof_verification.step5c2_inventory.{id}"),
                    format!("Step 5C.2 proof row `{id}` must occur exactly once; found {count}"),
                ));
            }
        }
    }
    diagnostics
}

fn is_step5c2_proof_id(case: &TestCase) -> bool {
    STEP5C2_PROOF_CASES.iter().any(|(id, _)| case.id.0 == *id)
}

fn step5c2_proof_case(case: &TestCase) -> Option<(&'static str, &'static str)> {
    STEP5C2_PROOF_CASES.iter().copied().find(|(id, source)| {
        case.id.0 == *id
            && case.source_path.ends_with(source)
            && case.expectation.stage == Stage::ProofVerification
            && case.expectation.expected_phase == Some(PipelinePhase::VcGeneration)
            && case.expectation.expected_outcome == ExpectedOutcome::Pass
    })
}

fn is_step5c2_proof_workspace_member(workspace_root: &Path, case: &TestCase) -> bool {
    step5c2_proof_case(case).is_some_and(|(_, source)| {
        workspace_relative_source(workspace_root, &case.source_path)
            .is_some_and(|actual| actual == source)
    })
}

pub(super) fn run_proof_verification_case(
    workspace_root: &Path,
    tests_root: &Path,
    case: &TestCase,
    ordinal: usize,
) -> ProofVerificationCaseResult {
    if is_step5c2_proof_workspace_member(workspace_root, case) {
        let first = normalize_structure_case(workspace_root, case, ordinal);
        let second = normalize_structure_case(workspace_root, case, ordinal);
        let failure = match (first, second) {
            (Ok(first), Ok(second)) => {
                if first != second {
                    Some("Step 5C.2 structure normalization rerun was nondeterministic".to_owned())
                } else if !first.has_zero_residual_vcs() || first.residual_vc_count() != 0 {
                    Some("Step 5C.2 structure normalization retained residual VCs".to_owned())
                } else {
                    None
                }
            }
            (Err(error), _) | (_, Err(error)) => Some(error),
        };
        return ProofVerificationCaseResult {
            id: case.id.clone(),
            expectation_path: case.expectation_path.clone(),
            status: if failure.is_none() {
                ProofVerificationCaseStatus::Passed
            } else {
                ProofVerificationCaseStatus::Failed
            },
            failure,
        };
    }

    let first = generate_case_vc(workspace_root, case, ordinal);
    let second = generate_case_vc(workspace_root, case, ordinal);
    let failure = match (first, second) {
        (Ok(first), Ok(second)) => {
            if first != second || first.debug_text() != second.debug_text() {
                Some("exact Task-180 source-to-VC rerun was nondeterministic".to_owned())
            } else {
                compare_vc_snapshot(tests_root, case.expectation.snapshots.as_deref(), &first)
            }
        }
        (Err(error), _) | (_, Err(error)) => Some(error),
    };
    ProofVerificationCaseResult {
        id: case.id.clone(),
        expectation_path: case.expectation_path.clone(),
        status: if failure.is_none() {
            ProofVerificationCaseStatus::Passed
        } else {
            ProofVerificationCaseStatus::Failed
        },
        failure,
    }
}

pub(in crate::runner) fn normalize_structure_case(
    workspace_root: &Path,
    case: &TestCase,
    ordinal: usize,
) -> Result<SourceStructureCoreReceipt, String> {
    if !is_step5c2_proof_workspace_member(workspace_root, case) {
        return Err("case is not an exact Step 5C.2 proof workspace member".to_owned());
    }
    let frontend = run_frontend(workspace_root, case, ordinal)?;
    if !frontend.diagnostics.is_empty() {
        return Err("Step 5C.2 proof source produced frontend diagnostics".to_owned());
    }
    let ast = frontend
        .ast
        .ok_or_else(|| "Step 5C.2 proof source produced no AST".to_owned())?;
    let resolver = resolver_symbol_collection(workspace_root, case, &ast);
    if resolver
        .detail_keys
        .iter()
        .any(|key| key != "declaration_symbol.symbol.duplicate_declaration")
    {
        return Err("Step 5C.2 proof source produced resolver diagnostics".to_owned());
    }
    let output = source_structure_semantics_output(&ast, resolver.module, &resolver.env)?;
    SourceStructureCoreNormalizer::normalize(&output)
        .map_err(|error| format!("Step 5C.2 Core normalization failed: {error}"))
}

fn active_tag_count(case: &TestCase) -> usize {
    case.expectation
        .tags
        .iter()
        .filter(|tag| tag.as_str() == ACTIVE_PROOF_VERIFICATION_TAG)
        .count()
}

pub(in crate::runner) fn generate_case_vc(
    workspace_root: &Path,
    case: &TestCase,
    _ordinal: usize,
) -> Result<VcSet, String> {
    // Task-180 is the single snapshot-backed proof case.  Its source and
    // snapshot identities stay fixed when later snapshot-free proof routes
    // are admitted before it in corpus order.
    let ordinal = 0;
    let frontend = run_frontend(workspace_root, case, ordinal)?;
    if !frontend.diagnostics.is_empty() {
        return Err("exact Task-180 source produced frontend diagnostics".to_owned());
    }
    let ast = frontend
        .ast
        .ok_or_else(|| "exact Task-180 source produced no AST".to_owned())?;
    let resolver = resolver_symbol_collection(workspace_root, case, &ast);
    if !resolver.detail_keys.is_empty() {
        return Err("exact Task-180 source produced resolver diagnostics".to_owned());
    }
    let symbols = augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
    let core = source_contradiction_core_ir(&ast, resolver.module, &symbols)?;
    generate_exact_task180_vc(ExactTask180VcInput {
        core: &core,
        snapshot: snapshot_id(ordinal),
        generation_schema_version: &GenerationSchemaVersion::new(GENERATION_SCHEMA),
        vc_schema_version: &VcSchemaVersion::new(VC_SCHEMA),
    })
    .map_err(|error| error.to_string())
}

pub(in crate::runner) fn compare_vc_snapshot(
    tests_root: &Path,
    snapshot_path: Option<&Path>,
    vc_set: &VcSet,
) -> Option<String> {
    let Some(snapshot_path) = snapshot_path else {
        return Some("exact Task-180 VcIr snapshot path is absent".to_owned());
    };
    let expected = match fs::read_to_string(tests_root.join(snapshot_path)) {
        Ok(expected) => expected,
        Err(error) => {
            return Some(format!(
                "could not read exact Task-180 VcIr snapshot `{}`: {error}",
                snapshot_path.display()
            ));
        }
    };
    let actual = vc_set.debug_text();
    if expected == actual {
        None
    } else {
        Some(format!(
            "exact Task-180 VcIr snapshot `{}` differed (expected {} bytes, got {} bytes)",
            snapshot_path.display(),
            expected.len(),
            actual.len()
        ))
    }
}

pub(super) fn proof_verification_failure_diagnostic(
    case: &TestCase,
    result: &ProofVerificationCaseResult,
) -> ValidationDiagnostic {
    ValidationDiagnostic::error(
        &case.expectation_path,
        "proof_verification",
        "E-PROOF-VERIFICATION-CASE",
        format!("proof_verification.{}", case.id.0),
        result
            .failure
            .clone()
            .unwrap_or_else(|| "proof-verification case failed".to_owned()),
    )
}
