#[test]
fn syntax_smoke_frontend_panic_isolated_without_skipping_following_case() {
    let panic_id = crate::expectation::TestCaseId("panic_case".to_owned());
    let next_id = crate::expectation::TestCaseId("next_case".to_owned());
    let expectation_path = std::path::Path::new("tests/miz/panic.expect.toml");
    let executions = [
        super::syntax_smoke::isolate_syntax_smoke_case_for_test(
            &panic_id,
            expectation_path,
            || {
            panic!("synthetic frontend panic");
            },
        ),
        super::syntax_smoke::isolate_syntax_smoke_case_for_test(
            &next_id,
            expectation_path,
            || super::syntax_smoke::SyntaxSmokeCaseExecution {
                result: super::SyntaxSmokeCaseResult {
                    id: next_id.clone(),
                    expectation_path: expectation_path.to_owned(),
                    status: super::SyntaxSmokeCaseStatus::Passed,
                    actual_diagnostic_codes: Vec::new(),
                },
                syntax_diagnostic_codes: Vec::new(),
                completed: true,
                has_ast: true,
            },
        ),
    ];

    assert_eq!(executions[0].result.id, panic_id);
    assert_eq!(
        executions[0].result.status,
        super::SyntaxSmokeCaseStatus::Failed
    );
    assert_eq!(
        executions[0].result.actual_diagnostic_codes,
        ["frontend_panic"]
    );
    assert_eq!(executions[1].result.id, next_id);
    assert_eq!(
        executions[1].result.status,
        super::SyntaxSmokeCaseStatus::Passed
    );
}

#[test]
fn syntax_smoke_frontend_error_fails_with_stable_code() {
    let id = crate::expectation::TestCaseId("error_case".to_owned());
    let expectation_path = std::path::Path::new("tests/miz/error.expect.toml");

    let execution = super::syntax_smoke::frontend_error_execution_for_test(
        &id,
        expectation_path,
        "synthetic failure",
    );

    assert_eq!(execution.result.id, id);
    assert_eq!(
        execution.result.status,
        super::SyntaxSmokeCaseStatus::Failed
    );
    assert_eq!(
        execution.result.actual_diagnostic_codes,
        ["frontend_error:synthetic failure"]
    );
    assert!(!execution.completed);
    assert!(!execution.has_ast);
}
