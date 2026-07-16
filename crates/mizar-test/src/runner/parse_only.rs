use std::fs;
use std::path::Path;

use crate::diagnostic::ValidationDiagnostic;
use crate::expectation::ExpectedOutcome;
use crate::harness::TestCase;

use super::shared::run_frontend;
use super::{
    ParseOnlyCaseResult, ParseOnlyCaseStatus, assertion_diagnostic_codes, frontend_error_code,
};

pub(super) fn run_parse_only_case(
    workspace_root: &Path,
    tests_root: &Path,
    case: &TestCase,
    ordinal: usize,
) -> ParseOnlyCaseResult {
    let output = run_frontend(workspace_root, case, ordinal);
    let (has_ast, actual_diagnostic_codes, ast_snapshot) = match output {
        Ok(output) => (
            output.ast.is_some(),
            assertion_diagnostic_codes(case, &output.diagnostics),
            output.ast_snapshot,
        ),
        Err(error) => (false, vec![frontend_error_code(&error)], None),
    };
    let expected_diagnostic_codes = &case.expectation.diagnostic_codes;
    let diagnostic_status = match case.expectation.expected_outcome {
        ExpectedOutcome::Pass
            if has_ast && actual_diagnostic_codes == *expected_diagnostic_codes =>
        {
            ParseOnlyCaseStatus::Passed
        }
        ExpectedOutcome::Fail if actual_diagnostic_codes == *expected_diagnostic_codes => {
            ParseOnlyCaseStatus::Passed
        }
        _ => ParseOnlyCaseStatus::Failed,
    };
    let snapshot_failure = if diagnostic_status == ParseOnlyCaseStatus::Passed {
        case.expectation
            .snapshots
            .as_ref()
            .and_then(|snapshot_path| {
                compare_surface_ast_snapshot(tests_root, snapshot_path, ast_snapshot.as_deref())
            })
    } else {
        None
    };
    let status = if snapshot_failure.is_some() {
        ParseOnlyCaseStatus::Failed
    } else {
        diagnostic_status
    };

    ParseOnlyCaseResult {
        id: case.id.clone(),
        expectation_path: case.expectation_path.clone(),
        status,
        actual_diagnostic_codes,
        snapshot_failure,
    }
}

pub(super) fn parse_only_failure_diagnostic(
    case: &TestCase,
    result: &ParseOnlyCaseResult,
) -> ValidationDiagnostic {
    if let Some(snapshot_failure) = &result.snapshot_failure {
        return ValidationDiagnostic::error(
            &case.expectation_path,
            "parse_only",
            "E-PARSE-ONLY-SNAPSHOT",
            format!("parse_only.snapshot.{}", case.id.0),
            format!("parse-only case `{}` {snapshot_failure}", case.id.0),
        );
    }
    ValidationDiagnostic::error(
        &case.expectation_path,
        "parse_only",
        "E-PARSE-ONLY-ASSERT",
        format!("parse_only.{}", case.id.0),
        format!(
            "parse-only case `{}` expected diagnostics {:?} but got {:?}",
            case.id.0, case.expectation.diagnostic_codes, result.actual_diagnostic_codes
        ),
    )
}

fn compare_surface_ast_snapshot(
    tests_root: &Path,
    snapshot_path: &Path,
    actual: Option<&str>,
) -> Option<String> {
    let Some(actual) = actual else {
        return Some(format!(
            "requested SurfaceAst snapshot `{}` but the parser produced no AST",
            snapshot_path.display()
        ));
    };
    let full_path = tests_root.join(snapshot_path);
    let expected = match fs::read_to_string(&full_path) {
        Ok(expected) => expected,
        Err(error) => {
            return Some(format!(
                "could not read SurfaceAst snapshot `{}`: {error}",
                snapshot_path.display()
            ));
        }
    };
    if expected == actual {
        None
    } else {
        Some(format!(
            "SurfaceAst snapshot `{}` differed (expected {} bytes, got {} bytes)",
            snapshot_path.display(),
            expected.len(),
            actual.len()
        ))
    }
}
