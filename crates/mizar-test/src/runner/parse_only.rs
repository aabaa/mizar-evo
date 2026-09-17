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
const STEP5C4_PARSE_ONLY_CASE: (&str, &str, ExpectedOutcome) = (
    "fail_parse_only_mode_property_impl_missing_correctness_001",
    "tests/miz/fail/modes/fail_parse_only_mode_property_impl_missing_correctness_001.miz",
    ExpectedOutcome::Fail,
);
const STEP5C4_PARSE_ONLY_DETAIL_KEY: &str =
    "modes.property_implementation.missing_existence_uniqueness";
const ACTIVE_PARSE_ONLY_TAG: &str = "active_parse_only";

const STEP5C11_PARSE_ID: &str = "fail_parse_only_cluster_adjective_argument_list_001";
const STEP5C11_PARSE_SOURCE: &str =
    "tests/miz/fail/clusters/fail_parse_only_cluster_adjective_argument_list_001.miz";

pub(super) fn is_step5c11_parse_candidate(case: &TestCase) -> bool {
    case.id.0 == STEP5C11_PARSE_ID
        || case.expectation.id.0 == STEP5C11_PARSE_ID
        || case.source_path.file_name() == Path::new(STEP5C11_PARSE_SOURCE).file_name()
        || case.expectation_path.file_name()
            == Path::new(STEP5C11_PARSE_SOURCE)
                .with_extension("expect.toml")
                .file_name()
}

pub(super) fn step5c11_parse_admitted(root: Option<&Path>, case: &TestCase) -> bool {
    let sidecar = Path::new(STEP5C11_PARSE_SOURCE).with_extension("expect.toml");
    let paths_match = [
        (&case.source_path, Path::new(STEP5C11_PARSE_SOURCE)),
        (&case.expectation_path, sidecar.as_path()),
    ]
    .into_iter()
    .all(|(actual, expected)| match root {
        Some(root) => super::syntax_smoke::workspace_relative_source(root, actual)
            .is_some_and(|path| Path::new(&path) == expected),
        None => actual.ends_with(expected),
    });
    paths_match
        && case.id.0 == STEP5C11_PARSE_ID
        && case.expectation.id == case.id
        && case.expectation.source == Path::new(STEP5C11_PARSE_SOURCE).file_name().unwrap()
        && case.expectation.stage == crate::staged_model::Stage::ParseOnly
        && case.expectation.expected_phase == Some(crate::expectation::PipelinePhase::Parse)
        && case.expectation.kind == crate::expectation::TestKind::Fail
        && case.expectation.expected_outcome == ExpectedOutcome::Fail
        && case.expectation.failure_category.as_deref() == Some("syntax_error")
        && case.expectation.stable_detail_key.as_deref()
            == Some("clusters.adjective.argument_list_form")
        && case.expectation.diagnostic_codes.is_empty()
        && case.expectation.diagnostic_payloads.is_empty()
        && case.expectation.declaration_symbol_payloads.is_empty()
        && case.expectation.snapshots.is_none()
        && case.expectation.tags.as_slice() == [ACTIVE_PARSE_ONLY_TAG]
}

pub(super) fn is_step5c4_parse_only_case(case: &TestCase) -> bool {
    case.id.0 == STEP5C4_PARSE_ONLY_CASE.0
        && case.source_path.ends_with(STEP5C4_PARSE_ONLY_CASE.1)
        && case.expectation.tags.as_slice() == [ACTIVE_PARSE_ONLY_TAG]
        && case.expectation.stage == crate::staged_model::Stage::ParseOnly
        && case.expectation.expected_phase == Some(crate::expectation::PipelinePhase::Parse)
        && case.expectation.expected_outcome == STEP5C4_PARSE_ONLY_CASE.2
        && case.expectation.stable_detail_key.as_deref() == Some(STEP5C4_PARSE_ONLY_DETAIL_KEY)
        && case.expectation.diagnostic_payloads.is_empty()
}

pub(super) fn is_step5c4_parse_only_workspace_member(
    workspace_root: &Path,
    case: &TestCase,
) -> bool {
    is_step5c4_parse_only_case(case)
        && super::syntax_smoke::workspace_relative_source(workspace_root, &case.source_path)
            .is_some_and(|source| source == STEP5C4_PARSE_ONLY_CASE.1)
}

pub(super) fn validate_step5c4_parse_only_inventory(
    workspace_root: &Path,
    plan: &crate::harness::TestPlan,
) -> Vec<ValidationDiagnostic> {
    if !workspace_root.join(STEP5C4_PARSE_ONLY_CASE.1).is_file() {
        return Vec::new();
    }
    let count = plan
        .cases
        .iter()
        .filter(|case| {
            case.id.0 == STEP5C4_PARSE_ONLY_CASE.0
                && super::syntax_smoke::workspace_relative_source(workspace_root, &case.source_path)
                    .is_some_and(|source| source == STEP5C4_PARSE_ONLY_CASE.1)
        })
        .count();
    if count == 1 {
        Vec::new()
    } else {
        vec![ValidationDiagnostic::error(
            Path::new(STEP5C4_PARSE_ONLY_CASE.1),
            "parse_only",
            "E-PARSE-ONLY-STEP5C4-INVENTORY",
            format!("parse_only.step5c4_inventory.{}", STEP5C4_PARSE_ONLY_CASE.0),
            format!(
                "Step 5C.4 parse-only route row `{}` must occur exactly once; found {count}",
                STEP5C4_PARSE_ONLY_CASE.0
            ),
        )]
    }
}

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
            let step5c4 = is_step5c4_parse_only_workspace_member(workspace_root, case);
            (
                output.ast.is_some(),
                if is_step5c11_parse_candidate(case) {
                    if step5c11_parse_admitted(Some(workspace_root), case)
                        && step5c11_registration_diagnostics(&output)
                    {
                        Vec::new()
                    } else {
                        vec!["clusters.adjective.invalid_diagnostic_provenance".to_owned()]
                    }
                } else if super::formula_statement::is_step5c8_candidate(case) {
                    if super::formula_statement::step5_formula_admitted(Some(workspace_root), case)
                        && step5c8_iff_diagnostic(&output)
                    {
                        Vec::new()
                    } else {
                        vec!["formulas.iff.invalid_diagnostic_provenance".to_owned()]
                    }
                } else if step5c3 {
                    if output.ast.as_ref().is_some_and(step5c3_parse_shape) {
                        Vec::new()
                    } else {
                        vec!["step5c3_parse_shape_mismatch".to_owned()]
                    }
                } else if step5c4 {
                    if output.ast.as_ref().is_some_and(step5c4_parse_shape) {
                        Vec::new()
                    } else {
                        vec![STEP5C4_PARSE_ONLY_DETAIL_KEY.to_owned()]
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

fn step5c11_registration_diagnostics(output: &super::shared::FrontendRun) -> bool {
    use mizar_frontend::orchestration::{DiagnosticClass, DiagnosticCode, DiagnosticLocation};
    let (Some(ast), [adjective_diagnostic, argument_diagnostic]) =
        (&output.ast, output.diagnostics.as_slice())
    else {
        return false;
    };
    if output.diagnostics.iter().any(|diagnostic| {
        diagnostic.code != DiagnosticCode::Syntax("malformed_type_expression".into())
            || diagnostic.class != DiagnosticClass::Syntax
    }) || adjective_diagnostic.message.as_ref()
        != "expected existential registration adjective before type"
        || argument_diagnostic.message.as_ref() != "registration adjectives cannot have arguments"
        || ast.nodes().iter().any(|node| node.recovered)
    {
        return false;
    }
    let mut nodes = ast
        .nodes()
        .iter()
        .filter(|node| node.kind == SurfaceNodeKind::RegistrationBlockItem)
        .collect::<Vec<_>>();
    for kind in [
        SurfaceNodeKind::ExistentialRegistration,
        SurfaceNodeKind::TypeExpression,
        SurfaceNodeKind::AttributeChain,
        SurfaceNodeKind::AttributeRef,
    ] {
        nodes = nodes
            .into_iter()
            .flat_map(|node| node.children.iter())
            .filter_map(|child| ast.node(*child))
            .filter(|node| node.kind == kind)
            .collect();
    }
    nodes.into_iter().any(|attribute| {
        let first = ast.token_views().find(|token| {
            token.range().start >= attribute.range.start && token.range().end <= attribute.range.end
        });
        first.is_some_and(|token| {
            adjective_diagnostic.location == DiagnosticLocation::SourceRange(token.range())
        }) && attribute
            .children
            .iter()
            .filter_map(|child| ast.node(*child))
            .any(|child| {
                child.token_text() == Some("(")
                    && argument_diagnostic.location == DiagnosticLocation::SourceRange(child.range)
            })
    })
}

fn step5c8_iff_diagnostic(output: &super::shared::FrontendRun) -> bool {
    use mizar_frontend::orchestration::{DiagnosticCode, DiagnosticLocation};
    use mizar_syntax::SurfaceFormulaConnective;
    let (Some(ast), [diagnostic]) = (&output.ast, output.diagnostics.as_slice()) else {
        return false;
    };
    if diagnostic.code != DiagnosticCode::Syntax("non_associative_operator_chain".into())
        || ast.nodes().iter().any(|node| node.recovered)
    {
        return false;
    }
    ast.nodes().iter().any(|node| {
        if !matches!(node.kind, SurfaceNodeKind::BinaryFormula(operator) if operator.connective == SurfaceFormulaConnective::Iff) {
            return false;
        }
        let [left, operator, _right] = node.children.as_slice() else { return false; };
        ast.node(*left).is_some_and(|left| matches!(left.kind, SurfaceNodeKind::BinaryFormula(operator) if operator.connective == SurfaceFormulaConnective::Iff))
            && ast.node(*operator).is_some_and(|operator| {
                operator.token_text() == Some("iff")
                    && diagnostic.location == DiagnosticLocation::SourceRange(operator.range)
            })
    })
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

pub(in crate::runner) fn step5c4_parse_shape(ast: &SurfaceAst) -> bool {
    if ast.token_texts()
        != [
            "definition",
            "struct",
            "U2Box",
            "where",
            "field",
            "data",
            "->",
            "set",
            ";",
            "property",
            "mark2",
            "->",
            "set",
            ";",
            "end",
            ";",
            "end",
            ";",
            "definition",
            "let",
            "B",
            "be",
            "U2Box",
            ";",
            "property",
            "B",
            ".",
            "mark2",
            "means",
            "it",
            "=",
            "B",
            ".",
            "data",
            ";",
            "end",
            ";",
        ]
    {
        return false;
    }
    ast.root()
        .and_then(|root| ast.node(root))
        .is_some_and(|root| {
            matches!(root.kind, SurfaceNodeKind::Root)
                && !root.recovered
                && super::type_elaboration::surface_nodes_with_kind_for_parse(
                    ast,
                    SurfaceNodeKind::StructureDefinition,
                )
                .len()
                    == 1
                && super::type_elaboration::surface_nodes_with_kind_for_parse(
                    ast,
                    SurfaceNodeKind::StructureField,
                )
                .len()
                    == 1
                && super::type_elaboration::surface_nodes_with_kind_for_parse(
                    ast,
                    SurfaceNodeKind::StructureProperty,
                )
                .len()
                    == 1
                && super::type_elaboration::surface_nodes_with_kind_for_parse(
                    ast,
                    SurfaceNodeKind::PropertyImplementation,
                )
                .len()
                    == 1
                && super::type_elaboration::surface_nodes_with_kind_for_parse(
                    ast,
                    SurfaceNodeKind::CorrectnessCondition,
                )
                .len()
                    == 2
                && super::type_elaboration::surface_nodes_with_kind_for_parse(
                    ast,
                    SurfaceNodeKind::CorrectnessCondition,
                )
                .iter()
                .all(|(_, node)| super::type_elaboration::subtree_has_recovery_for_parse(ast, node))
        })
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runner::formula_statement::step5c8_test_frontend;

    #[test]
    fn step5c11_registration_rejection_authenticates_source_and_diagnostics() {
        let source = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/miz/fail/clusters/fail_parse_only_cluster_adjective_argument_list_001.miz"
        ));
        let original = step5c8_test_frontend(source);
        assert!(
            step5c11_registration_diagnostics(&original),
            "{:?}",
            original.diagnostics
        );
        let renamed = source
            .replace("graded2", "ranked")
            .replace("CBad1", "Renamed")
            .replace("G2Def", "RenamedDef")
            .replace("X", "Y");
        assert!(step5c11_registration_diagnostics(&step5c8_test_frontend(
            &renamed
        )));
        for source in [
            source.replace("graded2(0)", "0-graded2"),
            source.replace("graded2(0)", "graded2"),
            source.replace("graded2(0)", "set"),
            source.replace("graded2(0)", "graded2("),
            source.replace("X = X", "X ="),
            "theorem F: for x being set holds x = ;".into(),
            "definition attr A: X is graded2 means X = X; end; theorem F: for X being graded2(0) set holds X = X;".into(),
        ] {
            let mut output = step5c8_test_frontend(&source);
            assert!(!step5c11_registration_diagnostics(&output), "{source}");
            output.diagnostics = original.diagnostics.clone();
            assert!(!step5c11_registration_diagnostics(&output), "substitution: {source}");
        }
        let mutations: &[fn(&mut super::super::shared::FrontendRun)] = &[
            |output| output.diagnostics.clear(),
            |output| {
                output.diagnostics.remove(0);
            },
            |output| output.diagnostics.push(output.diagnostics[0].clone()),
            |output| output.diagnostics.swap(0, 1),
            |output| {
                output.diagnostics[0].code = mizar_frontend::orchestration::DiagnosticCode::Syntax(
                    "missing_semicolon".into(),
                )
            },
            |output| {
                output.diagnostics[1].code = mizar_frontend::orchestration::DiagnosticCode::Syntax(
                    "missing_semicolon".into(),
                )
            },
            |output| output.diagnostics[0].message = "unrelated error".into(),
            |output| output.diagnostics[1].message = "unrelated error".into(),
            |output| output.diagnostics[0].location = output.diagnostics[1].location.clone(),
            |output| output.diagnostics[1].location = output.diagnostics[0].location.clone(),
            |output| output.ast = None,
        ];
        for mutate in mutations {
            let mut output = step5c8_test_frontend(source);
            mutate(&mut output);
            assert!(!step5c11_registration_diagnostics(&output));
        }
    }

    #[test]
    fn step5c11_parse_admission_rejects_metadata_and_cross_stage_fallback() {
        use crate::{DiscoveryConfig, PipelinePhase, Stage, TestProfile, ValidationMode};
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        let plan = crate::harness::build_test_plan(&DiscoveryConfig {
            workspace_root: root.into(),
            tests_root: root.join("tests"),
            manifest_path: root.join("tests/coverage/spec_trace.toml"),
            profile: TestProfile::Fast,
            validation_mode: ValidationMode::Metadata,
        })
        .unwrap();
        let original = plan
            .cases
            .iter()
            .find(|case| case.id.0 == STEP5C11_PARSE_ID)
            .unwrap();
        assert!(step5c11_parse_admitted(Some(root), original));
        assert!(super::super::is_active_parse_only(original));
        assert_eq!(
            run_parse_only_case(root, &root.join("tests"), original, 511).status,
            ParseOnlyCaseStatus::Passed
        );
        let mutations: &[fn(&mut TestCase)] = &[
            |case| case.id.0.push_str("_forged"),
            |case| case.expectation.id.0.push_str("_forged"),
            |case| case.source_path = case.source_path.with_file_name("wrong.miz"),
            |case| {
                case.expectation_path = case.expectation_path.with_file_name("wrong.expect.toml")
            },
            |case| case.expectation.source = "wrong.miz".into(),
            |case| case.expectation.stage = Stage::TypeElaboration,
            |case| case.expectation.expected_phase = Some(PipelinePhase::TypeCheck),
            |case| case.expectation.expected_outcome = ExpectedOutcome::Pass,
            |case| case.expectation.kind = crate::expectation::TestKind::Pass,
            |case| case.expectation.failure_category = Some("unrelated".into()),
            |case| case.expectation.stable_detail_key = Some("unrelated".into()),
            |case| case.expectation.diagnostic_codes.push("E-FORGED".into()),
            |case| case.expectation.diagnostic_payloads.push("forged".into()),
            |case| {
                case.expectation
                    .declaration_symbol_payloads
                    .push("forged".into())
            },
            |case| case.expectation.snapshots = Some("forged.snap".into()),
            |case| case.expectation.tags.clear(),
            |case| case.expectation.tags.push("active_parse_only".into()),
        ];
        for mutate in mutations {
            let mut case = original.clone();
            mutate(&mut case);
            assert!(is_step5c11_parse_candidate(&case));
            assert!(!step5c11_parse_admitted(Some(root), &case), "{case:?}");
            assert!(!super::super::is_active_parse_only(&case));
            let mut changed_plan = plan.clone();
            changed_plan.cases = vec![case];
            assert!(!super::super::validate_active_parse_only_tags(root, &changed_plan).is_empty());
        }
        for path in ["source", "sidecar"] {
            let mut case = original.clone();
            let actual = if path == "source" {
                &mut case.source_path
            } else {
                &mut case.expectation_path
            };
            *actual = root.join("alias").join(actual.strip_prefix(root).unwrap());
            assert!(!step5c11_parse_admitted(Some(root), &case));
            assert_eq!(
                run_parse_only_case(root, &root.join("tests"), &case, 512).status,
                ParseOnlyCaseStatus::Failed
            );
        }
        for (stage, phase, tag) in [
            (
                Stage::DeclarationSymbol,
                PipelinePhase::Resolve,
                "active_declaration_symbol",
            ),
            (
                Stage::TypeElaboration,
                PipelinePhase::TypeCheck,
                "active_type_elaboration",
            ),
            (
                Stage::FormulaStatement,
                PipelinePhase::StatementCheck,
                "active_formula_statement",
            ),
            (
                Stage::ProofVerification,
                PipelinePhase::VcGeneration,
                "active_proof_verification",
            ),
        ] {
            let mut case = original.clone();
            case.expectation.stage = stage;
            case.expectation.expected_phase = Some(phase);
            case.expectation.tags = vec![tag.into()];
            assert!(!super::super::is_active_parse_only(&case));
            assert!(!super::super::is_active_declaration_symbol(&case));
            assert!(!super::super::is_active_type_elaboration(&case));
            assert!(!super::super::formula_statement::is_active_formula_statement(root, &case));
            assert!(!super::super::is_active_proof_verification(&case));
        }
    }

    #[test]
    fn step5c8_iff_rejection_requires_the_real_chain_diagnostic_and_location() {
        let source = "theorem F: for x being set holds x = x iff x = x iff x = x;";
        let mut output = step5c8_test_frontend(source);
        assert!(step5c8_iff_diagnostic(&output), "{:?}", output.diagnostics);
        let diagnostic = output.diagnostics[0].clone();
        output.diagnostics.clear();
        assert!(!step5c8_iff_diagnostic(&output));
        output.diagnostics = vec![diagnostic.clone(), diagnostic.clone()];
        assert!(!step5c8_iff_diagnostic(&output));
        output.diagnostics = vec![diagnostic.clone()];
        output.diagnostics[0].code =
            mizar_frontend::orchestration::DiagnosticCode::Syntax("missing_semicolon".into());
        assert!(!step5c8_iff_diagnostic(&output));
        output.diagnostics = vec![diagnostic.clone()];
        output.diagnostics[0].location =
            mizar_frontend::orchestration::DiagnosticLocation::SourceRange(
                output.ast.as_ref().unwrap().nodes()[0].range,
            );
        assert!(!step5c8_iff_diagnostic(&output));
        for source in [
            "theorem F: for x being set holds (x = x iff x = x) iff x = x;",
            "theorem F: for x being set holds x = x iff (x = x iff x = x);",
            "theorem F: for x being set holds x = ;",
        ] {
            let mut unrelated = step5c8_test_frontend(source);
            assert!(!step5c8_iff_diagnostic(&unrelated));
            unrelated.diagnostics = vec![diagnostic.clone()];
            assert!(!step5c8_iff_diagnostic(&unrelated));
        }
    }
}
