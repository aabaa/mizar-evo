use crate::diagnostic::ValidationDiagnostic;
use crate::harness::TestCase;

use super::super::TypeElaborationCaseResult;

pub(in crate::runner) fn expected_type_elaboration_detail_keys(case: &TestCase) -> Vec<String> {
    if !case.expectation.diagnostic_payloads.is_empty() {
        return case.expectation.diagnostic_payloads.clone();
    }
    case.expectation.stable_detail_key.iter().cloned().collect()
}

pub(in crate::runner) fn type_elaboration_failure_diagnostic(
    case: &TestCase,
    result: &TypeElaborationCaseResult,
) -> ValidationDiagnostic {
    if let Some(snapshot_failure) = &result.snapshot_failure {
        return ValidationDiagnostic::error(
            &case.expectation_path,
            "type_elaboration",
            "E-TYPE-ELABORATION-SNAPSHOT",
            format!("type_elaboration.snapshot.{}", case.id.0),
            format!("type-elaboration case `{}` {snapshot_failure}", case.id.0),
        );
    }
    ValidationDiagnostic::error(
        &case.expectation_path,
        "type_elaboration",
        "E-TYPE-ELABORATION-ASSERT",
        format!("type_elaboration.{}", case.id.0),
        format!(
            "type-elaboration case `{}` expected detail keys {:?} but got {:?}",
            case.id.0,
            expected_type_elaboration_detail_keys(case),
            result.actual_detail_keys
        ),
    )
}
