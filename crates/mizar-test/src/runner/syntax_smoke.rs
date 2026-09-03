use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};

use mizar_frontend::orchestration::{DiagnosticCode, FrontendDiagnostic};

use crate::diagnostic::ValidationDiagnostic;
use crate::expectation::TestCaseId;
use crate::harness::{HarnessError, TestCase};
use crate::path_rules::clean_relative_path;

use super::shared::run_frontend;
use super::{SyntaxSmokeCaseResult, SyntaxSmokeCaseStatus};

const LEDGER_HEADER: &str = "case_id\tsource\tsyntax_diagnostic_codes\towner";
const LEDGER_RECORD_KIND: &str = "syntax_smoke";
const LEDGER_VALIDATION_CODE: &str = "E-SYNTAX-SMOKE-LEDGER";
const LEDGER_OWNER_TOKENS: [&str; 4] = ["template-spec-decision", "task-75", "task-76", "task-77"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SyntaxSmokeLedger {
    pub(super) path: PathBuf,
    pub(super) rows: Vec<SyntaxSmokeLedgerRow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SyntaxSmokeLedgerRow {
    pub(super) case_id: String,
    pub(super) source: String,
    pub(super) syntax_diagnostic_codes: Vec<String>,
    pub(super) owner: String,
    pub(super) selected_index: Option<usize>,
}

#[derive(Debug)]
pub(super) struct SyntaxSmokeCaseExecution {
    pub(super) result: SyntaxSmokeCaseResult,
    pub(super) syntax_diagnostic_codes: Vec<String>,
    pub(super) completed: bool,
    pub(super) has_ast: bool,
}

pub(super) fn run_syntax_smoke_case(
    workspace_root: &Path,
    case: &TestCase,
    ordinal: usize,
) -> SyntaxSmokeCaseExecution {
    isolate_syntax_smoke_case(&case.id, &case.expectation_path, || {
        run_syntax_smoke_case_without_panic_isolation(workspace_root, case, ordinal)
    })
}

fn isolate_syntax_smoke_case<F>(
    id: &TestCaseId,
    expectation_path: &Path,
    run: F,
) -> SyntaxSmokeCaseExecution
where
    F: FnOnce() -> SyntaxSmokeCaseExecution,
{
    match catch_unwind(AssertUnwindSafe(run)) {
        Ok(result) => result,
        Err(_) => SyntaxSmokeCaseExecution {
            result: SyntaxSmokeCaseResult {
                id: id.clone(),
                expectation_path: expectation_path.to_owned(),
                status: SyntaxSmokeCaseStatus::Failed,
                actual_diagnostic_codes: vec!["frontend_panic".to_owned()],
            },
            syntax_diagnostic_codes: Vec::new(),
            completed: false,
            has_ast: false,
        },
    }
}

fn run_syntax_smoke_case_without_panic_isolation(
    workspace_root: &Path,
    case: &TestCase,
    ordinal: usize,
) -> SyntaxSmokeCaseExecution {
    let output = match run_frontend(workspace_root, case, ordinal) {
        Ok(output) => output,
        Err(error) => {
            return frontend_error_execution(&case.id, &case.expectation_path, &error);
        }
    };
    let mut actual_diagnostic_codes = frontend_diagnostic_codes(&output.diagnostics);
    let syntax_diagnostic_codes = frontend_syntax_diagnostic_codes(&output.diagnostics);
    let has_ast = output.ast.is_some();
    if !has_ast {
        actual_diagnostic_codes.push("missing_ast".to_owned());
    }
    let status = if has_ast && syntax_diagnostic_codes.is_empty() {
        SyntaxSmokeCaseStatus::Passed
    } else {
        SyntaxSmokeCaseStatus::Failed
    };

    SyntaxSmokeCaseExecution {
        result: SyntaxSmokeCaseResult {
            id: case.id.clone(),
            expectation_path: case.expectation_path.clone(),
            status,
            actual_diagnostic_codes,
        },
        syntax_diagnostic_codes,
        completed: true,
        has_ast,
    }
}

fn frontend_error_execution(
    id: &TestCaseId,
    expectation_path: &Path,
    error: &str,
) -> SyntaxSmokeCaseExecution {
    SyntaxSmokeCaseExecution {
        result: SyntaxSmokeCaseResult {
            id: id.clone(),
            expectation_path: expectation_path.to_owned(),
            status: SyntaxSmokeCaseStatus::Failed,
            actual_diagnostic_codes: vec![format!("frontend_error:{error}")],
        },
        syntax_diagnostic_codes: Vec::new(),
        completed: false,
        has_ast: false,
    }
}

#[cfg(test)]
pub(super) fn isolate_syntax_smoke_case_for_test<F>(
    id: &TestCaseId,
    expectation_path: &Path,
    run: F,
) -> SyntaxSmokeCaseExecution
where
    F: FnOnce() -> SyntaxSmokeCaseExecution,
{
    isolate_syntax_smoke_case(id, expectation_path, run)
}

#[cfg(test)]
pub(super) fn frontend_error_execution_for_test(
    id: &TestCaseId,
    expectation_path: &Path,
    error: &str,
) -> SyntaxSmokeCaseExecution {
    frontend_error_execution(id, expectation_path, error)
}

pub(super) fn syntax_smoke_failure_diagnostic(
    case: &TestCase,
    result: &SyntaxSmokeCaseResult,
) -> ValidationDiagnostic {
    ValidationDiagnostic::error(
        &case.expectation_path,
        "syntax_smoke",
        "E-SYNTAX-SMOKE-ASSERT",
        format!("syntax_smoke.{}", case.id.0),
        format!(
            "syntax-smoke case `{}` failed with frontend result {:?}",
            case.id.0, result.actual_diagnostic_codes
        ),
    )
}

pub(super) fn syntax_smoke_ledger_diagnostic(
    path: &Path,
    detail_key: impl Into<String>,
    message: impl Into<String>,
) -> ValidationDiagnostic {
    ValidationDiagnostic::error(
        path,
        LEDGER_RECORD_KIND,
        LEDGER_VALIDATION_CODE,
        detail_key,
        message,
    )
}

pub(super) fn read_syntax_smoke_ledger(path: &Path) -> Result<Vec<u8>, HarnessError> {
    fs::read(path).map_err(|error| {
        HarnessError::Infrastructure(format!(
            "failed to read syntax smoke ledger `{}`: {error}",
            path.display()
        ))
    })
}

pub(super) fn parse_syntax_smoke_ledger(
    path: &Path,
    bytes: &[u8],
    workspace_root: &Path,
    selected_cases: &[&TestCase],
) -> Result<SyntaxSmokeLedger, Vec<ValidationDiagnostic>> {
    let text = match std::str::from_utf8(bytes) {
        Ok(text) => text,
        Err(error) => {
            return Err(vec![syntax_smoke_ledger_diagnostic(
                path,
                "syntax_smoke.ledger.utf8",
                format!("syntax smoke ledger is not valid UTF-8: {error}"),
            )]);
        }
    };

    let mut diagnostics = Vec::new();
    let mut lines = text.split('\n');
    match lines.next() {
        Some(header) if header == LEDGER_HEADER => {}
        Some(header) => diagnostics.push(syntax_smoke_ledger_diagnostic(
            path,
            "syntax_smoke.ledger.header",
            format!("syntax smoke ledger header must be `{LEDGER_HEADER}`, got `{header}`"),
        )),
        None => diagnostics.push(syntax_smoke_ledger_diagnostic(
            path,
            "syntax_smoke.ledger.header",
            format!("syntax smoke ledger is missing the exact header `{LEDGER_HEADER}`"),
        )),
    }

    let selected_index_by_id = selected_cases
        .iter()
        .enumerate()
        .map(|(index, case)| (case.id.0.as_str(), index))
        .collect::<BTreeMap<_, _>>();
    let mut selected_index_by_source = BTreeMap::<String, Vec<usize>>::new();
    for (index, case) in selected_cases.iter().enumerate() {
        if let Some(source) = workspace_relative_source(workspace_root, &case.source_path) {
            selected_index_by_source
                .entry(source)
                .or_default()
                .push(index);
        }
    }
    let mut seen_ids = BTreeSet::new();
    let mut seen_sources = BTreeSet::new();
    let mut rows = Vec::new();
    let mut previous_selected_index = None;

    for (line_index, line) in lines.enumerate() {
        let line_number = line_index + 2;
        if line.is_empty() {
            // A trailing LF terminates the final row; an empty line anywhere
            // else is a malformed content row.
            if line_index + 1 == text.split('\n').count() - 1 {
                continue;
            }
            diagnostics.push(syntax_smoke_ledger_diagnostic(
                path,
                format!("syntax_smoke.ledger.line.{line_number}"),
                format!("syntax smoke ledger line {line_number} is empty"),
            ));
            continue;
        }
        if line.contains('\r') {
            diagnostics.push(syntax_smoke_ledger_diagnostic(
                path,
                format!("syntax_smoke.ledger.line.{line_number}"),
                format!("syntax smoke ledger line {line_number} must use LF delimiters"),
            ));
            continue;
        }

        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() != 4 || fields.iter().any(|field| field.is_empty()) {
            diagnostics.push(syntax_smoke_ledger_diagnostic(
                path,
                format!("syntax_smoke.ledger.line.{line_number}"),
                format!(
                    "syntax smoke ledger line {line_number} must contain exactly four non-empty tab-separated fields"
                ),
            ));
            continue;
        }
        let [case_id, source, codes, owner] = [fields[0], fields[1], fields[2], fields[3]];
        if case_id.contains(',') || source.contains(',') || owner.contains(',') {
            diagnostics.push(syntax_smoke_ledger_diagnostic(
                path,
                format!("syntax_smoke.ledger.line.{line_number}"),
                format!(
                    "syntax smoke ledger line {line_number} contains a comma outside the syntax code field"
                ),
            ));
            continue;
        }
        if source.contains('\\') || !clean_relative_path(Path::new(source)) {
            diagnostics.push(syntax_smoke_ledger_diagnostic(
                path,
                format!("syntax_smoke.ledger.line.{line_number}"),
                format!(
                    "syntax smoke ledger line {line_number} source must be a clean forward-slash workspace-relative path"
                ),
            ));
            continue;
        }
        if case_id.chars().any(char::is_whitespace) || owner.chars().any(char::is_whitespace) {
            diagnostics.push(syntax_smoke_ledger_diagnostic(
                path,
                format!("syntax_smoke.ledger.line.{line_number}"),
                format!(
                    "syntax smoke ledger line {line_number} case_id and owner must be non-whitespace tokens"
                ),
            ));
            continue;
        }
        if !LEDGER_OWNER_TOKENS.contains(&owner) {
            diagnostics.push(syntax_smoke_ledger_diagnostic(
                path,
                format!("syntax_smoke.ledger.owner.{owner}"),
                format!("syntax smoke ledger owner `{owner}` is not a frozen owner token"),
            ));
            continue;
        }
        let syntax_diagnostic_codes = codes.split(',').collect::<Vec<_>>();
        if syntax_diagnostic_codes
            .iter()
            .any(|code| code.is_empty() || code.chars().any(char::is_whitespace))
        {
            diagnostics.push(syntax_smoke_ledger_diagnostic(
                path,
                format!("syntax_smoke.ledger.line.{line_number}"),
                format!(
                    "syntax smoke ledger line {line_number} must contain non-empty comma-separated syntax codes without whitespace"
                ),
            ));
            continue;
        }
        if !seen_ids.insert(case_id.to_owned()) {
            diagnostics.push(syntax_smoke_ledger_diagnostic(
                path,
                format!("syntax_smoke.ledger.duplicate_id.{case_id}"),
                format!("syntax smoke ledger contains duplicate case id `{case_id}`"),
            ));
        }
        if !seen_sources.insert(source.to_owned()) {
            diagnostics.push(syntax_smoke_ledger_diagnostic(
                path,
                format!("syntax_smoke.ledger.duplicate_source.{source}"),
                format!("syntax smoke ledger contains duplicate source `{source}`"),
            ));
        }

        let id_index = selected_index_by_id.get(case_id).copied();
        let source_indices = selected_index_by_source.get(source);
        let selected_index = match (id_index, source_indices) {
            // A known case id owns the execution slot even when its source
            // spelling is mismatched. The exact source identity is checked
            // at execution so the selected case can report the failure.
            (Some(id_index), Some(source_indices))
                if source_indices.len() == 1 && source_indices[0] == id_index =>
            {
                Some(id_index)
            }
            (Some(id_index), None) => Some(id_index),
            _ => {
                diagnostics.push(syntax_smoke_ledger_diagnostic(
                    path,
                    format!("syntax_smoke.ledger.membership.{case_id}"),
                    format!(
                        "syntax smoke ledger row `{case_id}` with source `{source}` must resolve to exactly one selected .miz case"
                    ),
                ));
                None
            }
        };
        if let Some(index) = selected_index {
            if previous_selected_index.is_some_and(|previous| index <= previous) {
                diagnostics.push(syntax_smoke_ledger_diagnostic(
                    path,
                    format!("syntax_smoke.ledger.order.{case_id}"),
                    format!(
                        "syntax smoke ledger rows must follow strictly increasing canonical selected-plan order; `{case_id}` is out of order"
                    ),
                ));
            }
            previous_selected_index = Some(index);
        }

        rows.push(SyntaxSmokeLedgerRow {
            case_id: case_id.to_owned(),
            source: source.to_owned(),
            syntax_diagnostic_codes: syntax_diagnostic_codes
                .into_iter()
                .map(str::to_owned)
                .collect(),
            owner: owner.to_owned(),
            selected_index,
        });
    }

    if diagnostics.is_empty() {
        Ok(SyntaxSmokeLedger {
            path: path.to_owned(),
            rows,
        })
    } else {
        Err(diagnostics)
    }
}

pub(super) fn workspace_relative_source(
    workspace_root: &Path,
    source_path: &Path,
) -> Option<String> {
    source_path
        .strip_prefix(workspace_root)
        .ok()
        .filter(|relative| clean_relative_path(relative))
        .map(|relative| relative.to_string_lossy().replace('\\', "/"))
}

fn frontend_diagnostic_codes(diagnostics: &[FrontendDiagnostic]) -> Vec<String> {
    diagnostics.iter().map(frontend_diagnostic_code).collect()
}

fn frontend_syntax_diagnostic_codes(diagnostics: &[FrontendDiagnostic]) -> Vec<String> {
    diagnostics
        .iter()
        .filter_map(|diagnostic| match &diagnostic.code {
            DiagnosticCode::Syntax(code) => Some(code.to_string()),
            _ => None,
        })
        .collect()
}

fn frontend_diagnostic_code(diagnostic: &FrontendDiagnostic) -> String {
    match &diagnostic.code {
        DiagnosticCode::SourceLoad => "source_load".to_owned(),
        DiagnosticCode::Preprocess(kind) => format!("preprocess:{kind:?}"),
        DiagnosticCode::LexicalEnvironment(code) => {
            format!("lexical_environment:{code:?}")
        }
        DiagnosticCode::Lexing(kind) => format!("lexing:{kind:?}"),
        DiagnosticCode::Syntax(code) => code.to_string(),
        _ => "frontend_diagnostic".to_owned(),
    }
}
