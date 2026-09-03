use std::path::Path;

use crate::diagnostic::ValidationDiagnostic;
use crate::expectation::{ExpectedOutcome, PipelinePhase};
use crate::harness::{TestCase, TestPlan};
use crate::staged_model::Stage;

use super::super::syntax_smoke::workspace_relative_source;

const ACTIVE_TYPE_ELABORATION_TAG: &str = "active_type_elaboration";

const STEP5C1_VARIABLE_CASES: [(&str, &str, PipelinePhase, ExpectedOutcome); 6] = [
    (
        "pass_type_elaboration_reserve_shadow_explicit_type_001",
        "tests/miz/pass/variables/pass_type_elaboration_reserve_shadow_explicit_type_001.miz",
        PipelinePhase::TypeCheck,
        ExpectedOutcome::Pass,
    ),
    (
        "pass_type_elaboration_reserve_implicit_typing_001",
        "tests/miz/pass/variables/pass_type_elaboration_reserve_implicit_typing_001.miz",
        PipelinePhase::TypeCheck,
        ExpectedOutcome::Pass,
    ),
    (
        "fail_type_elaboration_reconsider_unjustified_narrowing_001",
        "tests/miz/fail/variables/fail_type_elaboration_reconsider_unjustified_narrowing_001.miz",
        PipelinePhase::TypeCheck,
        ExpectedOutcome::Fail,
    ),
    (
        "fail_type_elaboration_set_duplicate_local_constant_001",
        "tests/miz/fail/variables/fail_type_elaboration_set_duplicate_local_constant_001.miz",
        PipelinePhase::Resolve,
        ExpectedOutcome::Fail,
    ),
    (
        "fail_type_elaboration_set_forward_reference_001",
        "tests/miz/fail/variables/fail_type_elaboration_set_forward_reference_001.miz",
        PipelinePhase::Resolve,
        ExpectedOutcome::Fail,
    ),
    (
        "fail_type_elaboration_unreserved_implicit_variable_001",
        "tests/miz/fail/variables/fail_type_elaboration_unreserved_implicit_variable_001.miz",
        PipelinePhase::Resolve,
        ExpectedOutcome::Fail,
    ),
];

pub(in crate::runner) fn is_active_type_elaboration(case: &TestCase) -> bool {
    let exact_step5c1 = step5c1_case(case).is_some()
        && case.expectation.tags.as_slice() == [ACTIVE_TYPE_ELABORATION_TAG];
    has_active_type_elaboration_tag(case)
        && case.expectation.stage == Stage::TypeElaboration
        && (case.expectation.expected_phase == Some(PipelinePhase::TypeCheck)
            || exact_step5c1 && case.expectation.expected_phase == Some(PipelinePhase::Resolve))
        && matches!(
            case.expectation.expected_outcome,
            ExpectedOutcome::Pass | ExpectedOutcome::Fail
        )
        && (!is_step5c1_id(case) || exact_step5c1)
        && case
            .source_path
            .extension()
            .is_some_and(|extension| extension == "miz")
}

fn has_active_type_elaboration_tag(case: &TestCase) -> bool {
    case.expectation
        .tags
        .iter()
        .any(|tag| tag == ACTIVE_TYPE_ELABORATION_TAG)
}

pub(in crate::runner) fn validate_active_type_elaboration_tags(
    workspace_root: &Path,
    plan: &TestPlan,
) -> Vec<ValidationDiagnostic> {
    let mut diagnostics = Vec::new();
    for case in plan
        .cases
        .iter()
        .filter(|case| has_active_type_elaboration_tag(case) || is_step5c1_id(case))
    {
        if !is_active_type_elaboration(case)
            || is_step5c1_id(case) && !is_step5c1_workspace_member(workspace_root, case)
        {
            diagnostics.push(
                ValidationDiagnostic::error(
                    &case.expectation_path,
                    "type_elaboration",
                    "E-TYPE-ELABORATION-ACTIVE-GATE",
                    format!("type_elaboration.active_gate.{}", case.id.0),
                    "active_type_elaboration cases must be .miz pass/fail expectations at stage type_elaboration and phase type_check; only the exact Step 5C.1 fail inventory may use phase resolve",
                ),
            );
        }
        if !case.expectation.diagnostic_codes.is_empty() {
            diagnostics.push(ValidationDiagnostic::error(
                &case.expectation_path,
                "type_elaboration",
                "E-TYPE-ELABORATION-PUBLIC-DIAGNOSTIC-CODES",
                format!("type_elaboration.public_codes.{}", case.id.0),
                "active_type_elaboration cases must keep diagnostic_codes empty until public checker diagnostic codes are specified; use diagnostic_payloads or stable_detail_key for internal detail keys",
            ));
        }
    }
    if STEP5C1_VARIABLE_CASES
        .iter()
        .any(|(_, source, _, _)| workspace_root.join(source).is_file())
    {
        for (id, source, _, _) in STEP5C1_VARIABLE_CASES {
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
                    std::path::Path::new(source),
                    "type_elaboration",
                    "E-TYPE-ELABORATION-STEP5C1-INVENTORY",
                    format!("type_elaboration.step5c1_inventory.{id}"),
                    format!(
                        "Step 5C.1 variable route row `{id}` must occur exactly once; found {count}"
                    ),
                ));
            }
        }
    }
    diagnostics
}

fn is_step5c1_id(case: &TestCase) -> bool {
    STEP5C1_VARIABLE_CASES
        .iter()
        .any(|(id, _, _, _)| case.id.0 == *id)
}

fn step5c1_case(
    case: &TestCase,
) -> Option<(&'static str, &'static str, PipelinePhase, ExpectedOutcome)> {
    STEP5C1_VARIABLE_CASES
        .iter()
        .copied()
        .find(|(id, source, phase, outcome)| {
            case.id.0 == *id
                && case.source_path.ends_with(source)
                && case.expectation.expected_phase == Some(*phase)
                && case.expectation.expected_outcome == *outcome
        })
}

pub(in crate::runner) fn is_step5c1_workspace_member(
    workspace_root: &Path,
    case: &TestCase,
) -> bool {
    step5c1_case(case).is_some_and(|(_, source, _, _)| {
        workspace_relative_source(workspace_root, &case.source_path)
            .is_some_and(|actual| actual == source)
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    use crate::expectation::{ExpectedOutcome, PipelinePhase};
    use crate::harness::{DiscoveryConfig, TestProfile, ValidationMode, build_test_plan};

    use super::{
        STEP5C1_VARIABLE_CASES, is_active_type_elaboration, validate_active_type_elaboration_tags,
    };

    #[test]
    fn step5c1_inventory_has_six_unique_id_source_pairs() {
        assert_eq!(STEP5C1_VARIABLE_CASES.len(), 6);
        assert_eq!(
            STEP5C1_VARIABLE_CASES
                .iter()
                .map(|(id, source, _, _)| (*id, *source))
                .collect::<BTreeSet<_>>()
                .len(),
            6
        );
    }

    #[test]
    fn step5c1_resolve_admission_rejects_duplicate_tag_phase_and_outcome_drift() {
        let plan = build_test_plan(&config()).unwrap();
        let original = plan
            .cases
            .iter()
            .find(|case| case.id.0 == STEP5C1_VARIABLE_CASES[3].0)
            .unwrap();
        assert!(is_active_type_elaboration(original));

        let mut case = original.clone();
        case.expectation
            .tags
            .push("active_type_elaboration".to_owned());
        assert!(!is_active_type_elaboration(&case));
        let mut case = original.clone();
        case.expectation
            .tags
            .push("allow_frontend_recovery_diagnostics".to_owned());
        assert!(!is_active_type_elaboration(&case));
        let mut case = original.clone();
        case.expectation.expected_phase = Some(PipelinePhase::TypeCheck);
        assert!(!is_active_type_elaboration(&case));
        let mut case = original.clone();
        case.expectation.expected_outcome = ExpectedOutcome::Pass;
        assert!(!is_active_type_elaboration(&case));

        let mut case = original.clone();
        case.id.0 = "fail_type_elaboration_unlisted_resolve_extra_001".to_owned();
        case.source_path = workspace_root()
            .join("tests/miz/fail/variables/fail_type_elaboration_unlisted_resolve_extra_001.miz");
        assert!(!is_active_type_elaboration(&case));
        let root = workspace_root();
        let mut plan = plan;
        plan.cases.push(case);
        assert!(
            validate_active_type_elaboration_tags(&root, &plan)
                .iter()
                .any(|diagnostic| diagnostic
                    .detail_key
                    .ends_with("fail_type_elaboration_unlisted_resolve_extra_001"))
        );
    }

    #[test]
    fn step5c1_inventory_rejects_missing_duplicate_and_public_codes() {
        let root = workspace_root();
        let mut plan = build_test_plan(&config()).unwrap();
        assert!(validate_active_type_elaboration_tags(&root, &plan).is_empty());

        let duplicate = plan
            .cases
            .iter()
            .find(|case| case.id.0 == STEP5C1_VARIABLE_CASES[0].0)
            .unwrap()
            .clone();
        plan.cases.push(duplicate);
        assert!(
            validate_active_type_elaboration_tags(&root, &plan)
                .iter()
                .any(|diagnostic| { diagnostic.code.0 == "E-TYPE-ELABORATION-STEP5C1-INVENTORY" })
        );

        plan.cases
            .retain(|case| case.id.0 != STEP5C1_VARIABLE_CASES[1].0);
        assert!(
            validate_active_type_elaboration_tags(&root, &plan)
                .iter()
                .any(|diagnostic| diagnostic.detail_key.ends_with(STEP5C1_VARIABLE_CASES[1].0))
        );

        let case = plan
            .cases
            .iter_mut()
            .find(|case| case.id.0 == STEP5C1_VARIABLE_CASES[2].0)
            .unwrap();
        case.expectation
            .diagnostic_codes
            .push("E-FORBIDDEN".to_owned());
        assert!(
            validate_active_type_elaboration_tags(&root, &plan)
                .iter()
                .any(|diagnostic| {
                    diagnostic.code.0 == "E-TYPE-ELABORATION-PUBLIC-DIAGNOSTIC-CODES"
                })
        );
    }

    #[test]
    fn step5c1_inventory_rejects_workspace_alias_path() {
        let root = workspace_root();
        let mut plan = build_test_plan(&config()).unwrap();
        let mut alias = plan
            .cases
            .iter()
            .find(|case| case.id.0 == STEP5C1_VARIABLE_CASES[0].0)
            .unwrap()
            .clone();
        alias.source_path = root.join(format!("alias/{}", STEP5C1_VARIABLE_CASES[0].1));
        assert!(alias.source_path.ends_with(STEP5C1_VARIABLE_CASES[0].1));
        plan.cases.push(alias);
        assert!(
            validate_active_type_elaboration_tags(&root, &plan)
                .iter()
                .any(|diagnostic| { diagnostic.code.0 == "E-TYPE-ELABORATION-ACTIVE-GATE" })
        );
    }

    #[test]
    fn type_corpus_executes_exact_six_and_preserves_fallback_and_bare_keys() {
        let report = super::super::super::run_type_elaboration_corpus(&config()).unwrap();
        assert_eq!(report.error_count(), 0, "{:?}", report.diagnostics);
        let exact_ids = STEP5C1_VARIABLE_CASES
            .iter()
            .map(|(id, _, _, _)| *id)
            .collect::<BTreeSet<_>>();
        let exact = report
            .results
            .iter()
            .filter(|result| exact_ids.contains(result.id.0.as_str()))
            .collect::<Vec<_>>();
        assert_eq!(exact.len(), 6);
        assert!(exact.iter().all(|result| {
            result.status == super::super::super::TypeElaborationCaseStatus::Passed
        }));
        assert!(exact.iter().any(|result| {
            result.actual_detail_keys
                == ["variables.local_constant.duplicate_identifier".to_owned()]
        }));
        assert!(exact.iter().any(|result| {
            result.actual_detail_keys == ["variables.local_constant.forward_reference".to_owned()]
        }));
        assert!(exact.iter().any(|result| {
            result.actual_detail_keys
                == ["variables.reserve.unreserved_implicit_variable".to_owned()]
        }));
        assert!(report.results.iter().any(|result| {
            result.id.0 == "pass_type_elaboration_property_implementation_equals_payload_001"
                && result.status == super::super::super::TypeElaborationCaseStatus::Passed
        }));
    }

    fn config() -> DiscoveryConfig {
        DiscoveryConfig {
            workspace_root: workspace_root(),
            tests_root: PathBuf::from("tests"),
            manifest_path: PathBuf::from("tests/coverage/spec_trace.toml"),
            profile: TestProfile::Fast,
            validation_mode: ValidationMode::Metadata,
        }
    }

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf()
    }
}
