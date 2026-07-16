use std::path::Path;

use mizar_resolve::env::{DefinitionKind, ExportStatus, SymbolEnv, SymbolKind, Visibility};

use crate::diagnostic::ValidationDiagnostic;
use crate::expectation::ExpectedOutcome;
use crate::harness::TestCase;

use super::shared::{FrontendRun, frontend_detail_keys, resolver_symbol_collection, run_frontend};
use super::{DeclarationSymbolCaseResult, DeclarationSymbolCaseStatus};

pub(super) fn run_declaration_symbol_case(
    workspace_root: &Path,
    case: &TestCase,
    ordinal: usize,
) -> DeclarationSymbolCaseResult {
    let output = run_frontend(workspace_root, case, ordinal);
    let actual = match output {
        Ok(output) => declaration_symbol_observation(workspace_root, case, output),
        Err(error) => DeclarationSymbolObservation {
            detail_keys: vec![format!("frontend_error:{error}")],
            payload_keys: Vec::new(),
        },
    };
    let expected_detail_keys = expected_declaration_symbol_detail_keys(case);
    let expected_payload_keys = expected_declaration_symbol_payload_keys(case);
    let status = match case.expectation.expected_outcome {
        ExpectedOutcome::Pass
            if actual.detail_keys.is_empty()
                && (case.expectation.declaration_symbol_payloads.is_empty()
                    || actual.payload_keys == expected_payload_keys) =>
        {
            DeclarationSymbolCaseStatus::Passed
        }
        ExpectedOutcome::Fail if actual.detail_keys == expected_detail_keys => {
            DeclarationSymbolCaseStatus::Passed
        }
        _ => DeclarationSymbolCaseStatus::Failed,
    };

    DeclarationSymbolCaseResult {
        id: case.id.clone(),
        expectation_path: case.expectation_path.clone(),
        status,
        actual_detail_keys: actual.detail_keys,
        actual_payload_keys: actual.payload_keys,
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct DeclarationSymbolObservation {
    detail_keys: Vec<String>,
    payload_keys: Vec<String>,
}

fn declaration_symbol_observation(
    workspace_root: &Path,
    case: &TestCase,
    output: FrontendRun,
) -> DeclarationSymbolObservation {
    let frontend_diagnostic_keys = frontend_detail_keys(case, &output.diagnostics);
    if !frontend_diagnostic_keys.is_empty() {
        return DeclarationSymbolObservation {
            detail_keys: frontend_diagnostic_keys,
            payload_keys: Vec::new(),
        };
    }

    let Some(ast) = output.ast else {
        return DeclarationSymbolObservation {
            detail_keys: vec!["declaration_symbol.no_ast".to_owned()],
            payload_keys: Vec::new(),
        };
    };
    let resolver = resolver_symbol_collection(workspace_root, case, &ast);
    let payload_keys = if resolver.detail_keys.is_empty() {
        declaration_symbol_payload_keys(&resolver.env)
    } else {
        Vec::new()
    };
    DeclarationSymbolObservation {
        detail_keys: resolver.detail_keys,
        payload_keys,
    }
}

fn declaration_symbol_payload_keys(env: &SymbolEnv) -> Vec<String> {
    let mut payloads = Vec::new();
    for symbol in env.symbols().iter() {
        let spelling = declaration_symbol_payload_component(symbol.primary_spelling());
        payloads.push(format!(
            "declaration_symbol.symbol.kind.{spelling}.{}",
            symbol_kind_payload_key(symbol.kind())
        ));
        payloads.push(format!(
            "declaration_symbol.symbol.visibility.{spelling}.{}",
            visibility_payload_key(symbol.visibility())
        ));
        payloads.push(format!(
            "declaration_symbol.symbol.export.{spelling}.{}",
            export_status_payload_key(symbol.export_status())
        ));
        if let Some(definition) = env.definitions().by_symbol(symbol.symbol()) {
            payloads.push(format!(
                "declaration_symbol.definition.kind.{spelling}.{}",
                definition_kind_payload_key(definition.kind())
            ));
            payloads.push(format!(
                "declaration_symbol.definition.visibility.{spelling}.{}",
                visibility_payload_key(definition.visibility())
            ));
        }
    }
    payloads.sort();
    payloads
}

fn declaration_symbol_payload_component(value: &str) -> String {
    let mut escaped = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-') {
            escaped.push(byte as char);
        } else {
            escaped.push('%');
            escaped.push(hex_digit(byte >> 4));
            escaped.push(hex_digit(byte & 0x0f));
        }
    }
    escaped
}

const fn hex_digit(value: u8) -> char {
    match value {
        0..=9 => (b'0' + value) as char,
        10..=15 => (b'A' + (value - 10)) as char,
        _ => '?',
    }
}

const fn symbol_kind_payload_key(kind: SymbolKind) -> &'static str {
    match kind {
        SymbolKind::Predicate => "predicate",
        SymbolKind::Functor => "functor",
        SymbolKind::Mode => "mode",
        SymbolKind::Attribute => "attribute",
        SymbolKind::Structure => "structure",
        SymbolKind::Selector => "selector",
        SymbolKind::Registration => "registration",
        SymbolKind::Theorem => "theorem",
        SymbolKind::Lemma => "lemma",
        SymbolKind::Algorithm => "algorithm",
        SymbolKind::Scheme => "scheme",
        SymbolKind::Template => "template",
        SymbolKind::Synonym => "synonym",
        SymbolKind::Antonym => "antonym",
        SymbolKind::Redefinition => "redefinition",
        SymbolKind::Builtin => "builtin",
        _ => "unknown",
    }
}

const fn definition_kind_payload_key(kind: DefinitionKind) -> &'static str {
    match kind {
        DefinitionKind::Predicate => "predicate",
        DefinitionKind::Functor => "functor",
        DefinitionKind::Mode => "mode",
        DefinitionKind::Attribute => "attribute",
        DefinitionKind::Structure => "structure",
        DefinitionKind::Registration => "registration",
        DefinitionKind::Theorem => "theorem",
        DefinitionKind::Lemma => "lemma",
        DefinitionKind::Algorithm => "algorithm",
        DefinitionKind::Scheme => "scheme",
        DefinitionKind::Template => "template",
        DefinitionKind::Synonym => "synonym",
        DefinitionKind::Antonym => "antonym",
        DefinitionKind::Redefinition => "redefinition",
        DefinitionKind::Selector => "selector",
        _ => "unknown",
    }
}

const fn visibility_payload_key(visibility: Visibility) -> &'static str {
    match visibility {
        Visibility::Private => "private",
        Visibility::Public => "public",
        _ => "unknown",
    }
}

const fn export_status_payload_key(status: ExportStatus) -> &'static str {
    match status {
        ExportStatus::LocalOnly => "local_only",
        ExportStatus::Exported => "exported",
        ExportStatus::ReExported => "re_exported",
        _ => "unknown",
    }
}

fn expected_declaration_symbol_detail_keys(case: &TestCase) -> Vec<String> {
    if !case.expectation.diagnostic_payloads.is_empty() {
        return case.expectation.diagnostic_payloads.clone();
    }
    case.expectation.stable_detail_key.iter().cloned().collect()
}

fn expected_declaration_symbol_payload_keys(case: &TestCase) -> Vec<String> {
    let mut payloads = case.expectation.declaration_symbol_payloads.clone();
    payloads.sort();
    payloads
}

pub(super) fn declaration_symbol_failure_diagnostic(
    case: &TestCase,
    result: &DeclarationSymbolCaseResult,
) -> ValidationDiagnostic {
    ValidationDiagnostic::error(
        &case.expectation_path,
        "declaration_symbol",
        "E-DECLARATION-SYMBOL-ASSERT",
        format!("declaration_symbol.{}", case.id.0),
        format!(
            "declaration-symbol case `{}` expected detail keys {:?} but got {:?}; expected payload keys {:?} but got {:?}",
            case.id.0,
            expected_declaration_symbol_detail_keys(case),
            result.actual_detail_keys,
            expected_declaration_symbol_payload_keys(case),
            result.actual_payload_keys
        ),
    )
}
