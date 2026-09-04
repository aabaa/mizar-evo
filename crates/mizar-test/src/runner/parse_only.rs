use std::fs;
use std::path::Path;

use crate::diagnostic::ValidationDiagnostic;
use crate::expectation::ExpectedOutcome;
use crate::harness::TestCase;
use mizar_syntax::{SurfaceAst, SurfaceNode, SurfaceNodeKind};

use super::shared::{assertion_diagnostic_codes, frontend_error_code, run_frontend};
use super::{ParseOnlyCaseResult, ParseOnlyCaseStatus};

const STEP5C3_PARSE_ONLY_CASE: (&str, &str, ExpectedOutcome) = (
    "fail_type_elaboration_attr_param_prefix_unbound_001",
    "tests/miz/fail/attributes/fail_type_elaboration_attr_param_prefix_unbound_001.miz",
    ExpectedOutcome::Fail,
);
const ACTIVE_PARSE_ONLY_TAG: &str = "active_parse_only";

pub(super) fn is_step5c3_parse_only_case(case: &TestCase) -> bool {
    case.id.0 == STEP5C3_PARSE_ONLY_CASE.0
        && case.source_path.ends_with(STEP5C3_PARSE_ONLY_CASE.1)
        && case.expectation.tags.as_slice() == [ACTIVE_PARSE_ONLY_TAG]
        && case.expectation.stage == crate::staged_model::Stage::ParseOnly
        && case.expectation.expected_phase == Some(crate::expectation::PipelinePhase::Parse)
        && case.expectation.expected_outcome == STEP5C3_PARSE_ONLY_CASE.2
}

pub(super) fn is_step5c3_parse_only_workspace_member(
    workspace_root: &Path,
    case: &TestCase,
) -> bool {
    is_step5c3_parse_only_case(case)
        && super::syntax_smoke::workspace_relative_source(workspace_root, &case.source_path)
            .is_some_and(|source| source == STEP5C3_PARSE_ONLY_CASE.1)
}

pub(super) fn validate_step5c3_parse_only_inventory(
    workspace_root: &Path,
    plan: &crate::harness::TestPlan,
) -> Vec<ValidationDiagnostic> {
    if !workspace_root.join(STEP5C3_PARSE_ONLY_CASE.1).is_file() {
        return Vec::new();
    }
    let count = plan
        .cases
        .iter()
        .filter(|case| {
            case.id.0 == STEP5C3_PARSE_ONLY_CASE.0
                && super::syntax_smoke::workspace_relative_source(workspace_root, &case.source_path)
                    .is_some_and(|source| source == STEP5C3_PARSE_ONLY_CASE.1)
        })
        .count();
    if count == 1 {
        Vec::new()
    } else {
        vec![ValidationDiagnostic::error(
            Path::new(STEP5C3_PARSE_ONLY_CASE.1),
            "parse_only",
            "E-PARSE-ONLY-STEP5C3-INVENTORY",
            format!("parse_only.step5c3_inventory.{}", STEP5C3_PARSE_ONLY_CASE.0),
            format!(
                "Step 5C.3 parse-only route row `{}` must occur exactly once; found {count}",
                STEP5C3_PARSE_ONLY_CASE.0
            ),
        )]
    }
}

pub(super) fn run_parse_only_case(
    workspace_root: &Path,
    tests_root: &Path,
    case: &TestCase,
    ordinal: usize,
) -> ParseOnlyCaseResult {
    let output = run_frontend(workspace_root, case, ordinal);
    let (has_ast, actual_diagnostic_codes, ast_snapshot) = match output {
        Ok(output) => {
            let step5c3 = is_step5c3_parse_only_workspace_member(workspace_root, case);
            (
                output.ast.is_some(),
                if step5c3 {
                    if output.ast.as_ref().is_some_and(step5c3_parse_shape) {
                        Vec::new()
                    } else {
                        vec!["step5c3_parse_shape_mismatch".to_owned()]
                    }
                } else {
                    assertion_diagnostic_codes(case, &output.diagnostics)
                },
                output.ast_snapshot,
            )
        }
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

pub(in crate::runner) fn step5c3_parse_shape(ast: &SurfaceAst) -> bool {
    if ast.token_texts()
        != [
            "definition",
            "let",
            "X",
            "be",
            "set",
            ";",
            "attr",
            "PDef",
            ":",
            "X",
            "is",
            "k",
            "-",
            "scaled",
            "means",
            "X",
            "=",
            "X",
            ";",
            "end",
            ";",
        ]
    {
        return false;
    }
    let patterns = super::type_elaboration::surface_nodes_with_kind_for_parse(
        ast,
        SurfaceNodeKind::AttributePattern,
    );
    let definitions = super::type_elaboration::surface_nodes_with_kind_for_parse(
        ast,
        SurfaceNodeKind::AttributeDefinition,
    );
    let recovered = super::type_elaboration::surface_nodes_with_kind_for_parse(
        ast,
        SurfaceNodeKind::ErrorRecovery(mizar_syntax::SyntaxRecoveryKind::SkippedToken),
    );
    patterns.len() == 1
        && patterns[0].1.children.len() == 1
        && patterns[0]
            .1
            .children
            .first()
            .and_then(|child| ast.node(*child))
            .and_then(SurfaceNode::token_text)
            == Some("k")
        && definitions.len() == 1
        && super::type_elaboration::subtree_has_recovery_for_parse(ast, definitions[0].1)
        && recovered.len() == 1
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
