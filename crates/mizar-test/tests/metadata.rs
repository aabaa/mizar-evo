use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use mizar_frontend::lexical_env::{
    FrontendLexicalEnvironmentError, LexicalEnvironmentRequest, LexicalSummaryProvider, ModuleId,
    ResolvedImports, build_active_lexical_environment,
};
use mizar_frontend::lexing::{ParserLexingPlan, TokenKind, TokenizeRequest, tokenize};
use mizar_frontend::preprocess::preprocess;
use mizar_frontend::source::{SourceUnit, register_source_unit};
use mizar_frontend::span_bridge::SpanBridge;
use mizar_session::{
    BuildSnapshotId, Edition, Hash, InMemorySessionIdAllocator, LineMap, ModulePath, PackageId,
    SessionIdAllocator, SourceOrigin, hash_text, normalize_path,
};
use mizar_test::{
    DiscoveryConfig, ExpectedOutcome, PipelinePhase, Stage, TestKind, TestPlan, TestProfile,
    ValidationMode, active_parse_only_cases, build_test_plan, run_parse_only_corpus,
};

static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
const TEMPLATE_ARGUMENTS_REQUIREMENT_ID: &str = "spec.en.syntax.template_arguments.parser";
const ALGORITHMS_CLAIMS_REQUIREMENT_ID: &str = "spec.en.20.algorithms_claims.parser";
const ALGORITHM_CONTROL_FLOW_REQUIREMENT_ID: &str = "spec.en.20.algorithm_control_flow.parser";
const ALGORITHM_VERIFICATION_REQUIREMENT_ID: &str = "spec.en.20.algorithm_verification.parser";
const ANNOTATIONS_REQUIREMENT_ID: &str = "spec.en.21.annotations.parser";
const OPERATOR_PRECEDENCE_REQUIREMENT_ID: &str = "spec.en.13.operator_precedence.parser";
const SET_EXPRESSION_REQUIREMENT_ID: &str = "spec.en.13.set_expressions.parser";
const ATOMIC_FORMULA_REQUIREMENT_ID: &str = "spec.en.14.atomic_formula.parser";
const FORMULA_CONNECTIVES_REQUIREMENT_ID: &str =
    "spec.en.14.formula_connectives_quantifiers.parser";
const ATTRIBUTE_DEFINITIONS_REQUIREMENT_ID: &str = "spec.en.06.attribute_definitions.parser";
const PREDICATE_DEFINITIONS_REQUIREMENT_ID: &str = "spec.en.09.predicate_definitions.parser";
const FUNCTOR_DEFINITIONS_REQUIREMENT_ID: &str = "spec.en.10.functor_definitions.parser";
const MODE_DEFINITIONS_REQUIREMENT_ID: &str = "spec.en.07.mode_definitions.parser";
const REDEFINITION_NOTATION_REQUIREMENT_ID: &str = "spec.en.syntax.redefinition_notation.parser";
const PROPERTY_CLAUSES_REQUIREMENT_ID: &str = "spec.en.syntax.property_clauses.parser";
const STRUCTURES_REQUIREMENT_ID: &str = "spec.en.05.structures.parser";
const CORRECTNESS_CONDITIONS_REQUIREMENT_ID: &str = "spec.en.16.correctness_conditions.parser";
const REGISTRATIONS_REQUIREMENT_ID: &str = "spec.en.17.clusters_and_registrations.parser";
const PARSER_DEFERRED_RESERVED_WORDS: &[&str] = &[
    "infix_operator",
    "left",
    "none",
    "postfix_operator",
    "prefix_operator",
    "right",
    "transitivity",
];

#[test]
fn empty_corpus_succeeds() {
    let corpus = Corpus::new();

    let plan = corpus.plan();

    assert_eq!(plan.error_count(), 0);
    assert_eq!(plan.cases.len(), 0);
    assert_eq!(plan.manifest.requirements.len(), 0);
}

#[test]
fn malformed_toml_fails() {
    let corpus = Corpus::new();
    corpus.write("tests/lexical/pass/bad.src", "");
    corpus.write(
        "tests/lexical/pass/bad.expect.toml",
        "schema_version = \"one\"\n",
    );

    let plan = corpus.plan();

    assert_has_code(&plan, "E-EXPECT-SCHEMA");
}

#[test]
fn duplicate_expectation_ids_fail() {
    let corpus = Corpus::new();
    corpus.add_requirement("spec.en.test.basic", &[]);
    corpus.add_case(
        "tests/lexical/pass/dup_one",
        "dup_shared",
        "spec.en.test.basic",
    );
    corpus.add_case(
        "tests/lexical/pass/dup_two",
        "dup_shared",
        "spec.en.test.basic",
    );

    let plan = corpus.plan();

    assert_has_code(&plan, "E-EXPECT-DUP-ID");
}

#[test]
fn missing_source_fails() {
    let corpus = Corpus::new();
    corpus.add_requirement("spec.en.test.basic", &[]);
    corpus.write(
        "tests/lexical/pass/missing.expect.toml",
        expectation("missing", "missing.src", "spec.en.test.basic"),
    );

    let plan = corpus.plan();

    assert_has_code(&plan, "E-EXPECT-MISSING-SOURCE");
}

#[test]
fn missing_sidecar_fails_for_payload() {
    let corpus = Corpus::new();
    corpus.write("tests/lexical/pass/orphan.src", "");

    let plan = corpus.plan();

    assert_has_code(&plan, "E-LAYOUT-MISSING-SIDECAR");
}

#[test]
fn unknown_spec_refs_fail() {
    let corpus = Corpus::new();
    corpus.add_case(
        "tests/lexical/pass/unknown_spec",
        "unknown_spec",
        "spec.en.test.unknown",
    );

    let plan = corpus.plan();

    assert_has_code(&plan, "E-TRACE-UNKNOWN-SPEC-REF");
}

#[test]
fn manifest_test_back_reference_succeeds() {
    let corpus = Corpus::new();
    corpus.add_case("tests/lexical/pass/linked", "linked", "spec.en.test.basic");
    corpus.add_requirement(
        "spec.en.test.basic",
        &["tests/lexical/pass/linked.expect.toml"],
    );

    let plan = corpus.plan();

    assert_eq!(plan.error_count(), 0, "{:#?}", plan.diagnostics);
    assert_eq!(plan.cases.len(), 1);
}

#[test]
fn expectation_source_must_be_clean_relative_path() {
    let corpus = Corpus::new();
    corpus.add_requirement("spec.en.test.basic", &[]);
    corpus.write("tests/lexical/pass/escape.src", "");
    corpus.write(
        "tests/lexical/pass/escape.expect.toml",
        expectation("escape", "../escape.src", "spec.en.test.basic"),
    );

    let plan = corpus.plan();

    assert_has_code(&plan, "E-EXPECT-SOURCE-PATH");
}

#[test]
fn expectation_source_path_error_does_not_hide_spec_ref_errors() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/lexical/pass/escape.expect.toml",
        r#"schema_version = 1
id = "escape"
kind = "pass"
stage = "lexical"
domain = "lexical"
source = "../escape.src"
expected_outcome = "pass"
expected_phase = "lex"
diagnostic_codes = []
spec_refs = []
"#,
    );

    let plan = corpus.plan();

    assert_has_code(&plan, "E-EXPECT-SOURCE-PATH");
    assert_has_code(&plan, "E-EXPECT-SPEC-REFS");
}

#[test]
fn expectation_source_must_use_payload_extension() {
    let corpus = Corpus::new();
    corpus.add_requirement("spec.en.test.basic", &[]);
    corpus.write("tests/lexical/pass/not_payload.txt", "");
    corpus.write(
        "tests/lexical/pass/not_payload.expect.toml",
        expectation("not_payload", "not_payload.txt", "spec.en.test.basic"),
    );

    let plan = corpus.plan();

    assert_has_code(&plan, "E-EXPECT-SOURCE-EXTENSION");
}

#[test]
fn expectation_unknown_fields_fail() {
    let corpus = Corpus::new();
    corpus.write("tests/lexical/pass/unknown_field.src", "");
    corpus.write(
        "tests/lexical/pass/unknown_field.expect.toml",
        r#"schema_version = 1
id = "unknown_field"
kind = "pass"
stage = "lexical"
domain = "lexical"
source = "unknown_field.src"
expected_outcome = "pass"
expected_phase = "lex"
diagnostic_codes = []
spec_refs = ["spec.en.test.basic"]
surprise = "nope"
"#,
    );

    let plan = corpus.plan();

    assert_has_code(&plan, "E-EXPECT-SCHEMA");
}

#[test]
fn pass_expectation_requires_expected_phase() {
    let corpus = Corpus::new();
    corpus.write("tests/lexical/pass/no_phase.src", "");
    corpus.write(
        "tests/lexical/pass/no_phase.expect.toml",
        r#"schema_version = 1
id = "no_phase"
kind = "pass"
stage = "lexical"
domain = "lexical"
source = "no_phase.src"
expected_outcome = "pass"
diagnostic_codes = []
spec_refs = ["spec.en.test.basic"]
"#,
    );

    let plan = corpus.plan();

    assert_has_code(&plan, "E-EXPECT-SCHEMA");
}

#[test]
fn expectation_requires_diagnostic_codes() {
    let corpus = Corpus::new();
    corpus.write("tests/lexical/pass/no_diagnostics.src", "");
    corpus.write(
        "tests/lexical/pass/no_diagnostics.expect.toml",
        r#"schema_version = 1
id = "no_diagnostics"
kind = "pass"
stage = "lexical"
domain = "lexical"
source = "no_diagnostics.src"
expected_outcome = "pass"
expected_phase = "lex"
spec_refs = ["spec.en.test.basic"]
"#,
    );

    let plan = corpus.plan();

    assert_has_code(&plan, "E-EXPECT-SCHEMA");
}

#[test]
fn expectation_identity_fields_must_not_be_empty() {
    let corpus = Corpus::new();
    corpus.write("tests/lexical/pass/empty_identity.src", "");
    corpus.write(
        "tests/lexical/pass/empty_identity.expect.toml",
        r#"schema_version = 1
id = ""
kind = "pass"
stage = "lexical"
domain = "lexical"
source = "empty_identity.src"
expected_outcome = "pass"
expected_phase = "lex"
diagnostic_codes = []
spec_refs = ["spec.en.test.basic"]
"#,
    );
    corpus.write("tests/lexical/pass/empty_domain.src", "");
    corpus.write(
        "tests/lexical/pass/empty_domain.expect.toml",
        r#"schema_version = 1
id = "empty_domain"
kind = "pass"
stage = "lexical"
domain = ""
source = "empty_domain.src"
expected_outcome = "pass"
expected_phase = "lex"
diagnostic_codes = []
spec_refs = ["spec.en.test.basic"]
"#,
    );
    corpus.write("tests/lexical/pass/empty_spec_ref.src", "");
    corpus.write(
        "tests/lexical/pass/empty_spec_ref.expect.toml",
        r#"schema_version = 1
id = "empty_spec_ref"
kind = "pass"
stage = "lexical"
domain = "lexical"
source = "empty_spec_ref.src"
expected_outcome = "pass"
expected_phase = "lex"
diagnostic_codes = []
spec_refs = [""]
"#,
    );

    let plan = corpus.plan();
    let schema_errors = plan
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code.0 == "E-EXPECT-SCHEMA")
        .count();

    assert_eq!(schema_errors, 3, "{:#?}", plan.diagnostics);
}

#[test]
fn metadata_only_is_not_valid_for_source_payloads() {
    let corpus = Corpus::new();
    corpus.write("tests/lexical/pass/metadata_only.src", "");
    corpus.write(
        "tests/lexical/pass/metadata_only.expect.toml",
        r#"schema_version = 1
id = "metadata_only"
kind = "property_seed"
stage = "lexical"
domain = "lexical"
source = "metadata_only.src"
expected_outcome = "metadata_only"
diagnostic_codes = []
spec_refs = ["spec.en.test.basic"]
"#,
    );

    let plan = corpus.plan();

    assert_has_code(&plan, "E-EXPECT-SCHEMA");
}

#[test]
fn fail_identity_fields_must_not_be_empty() {
    let corpus = Corpus::new();
    corpus.write("tests/lexical/fail/empty_failure_category.src", "");
    corpus.write(
        "tests/lexical/fail/empty_failure_category.expect.toml",
        r#"schema_version = 1
id = "empty_failure_category"
kind = "fail"
stage = "lexical"
domain = "lexical"
source = "empty_failure_category.src"
expected_outcome = "fail"
expected_phase = "lex"
failure_category = ""
diagnostic_codes = ["E-LEX-TEST"]
stable_detail_key = "lexical.test"
spec_refs = ["spec.en.test.basic"]
"#,
    );
    corpus.write("tests/lexical/fail/empty_detail_key.src", "");
    corpus.write(
        "tests/lexical/fail/empty_detail_key.expect.toml",
        r#"schema_version = 1
id = "empty_detail_key"
kind = "fail"
stage = "lexical"
domain = "lexical"
source = "empty_detail_key.src"
expected_outcome = "fail"
expected_phase = "lex"
failure_category = "lex_error"
diagnostic_codes = ["E-LEX-TEST"]
stable_detail_key = ""
spec_refs = ["spec.en.test.basic"]
"#,
    );
    corpus.write("tests/lexical/fail/empty_rejection_reason.src", "");
    corpus.write(
        "tests/lexical/fail/empty_rejection_reason.expect.toml",
        r#"schema_version = 1
id = "empty_rejection_reason"
kind = "fail"
stage = "lexical"
domain = "lexical"
source = "empty_rejection_reason.src"
expected_outcome = "fail"
expected_phase = "lex"
failure_category = "lex_error"
rejection_reason = ""
diagnostic_codes = ["E-LEX-TEST"]
stable_detail_key = "lexical.test"
spec_refs = ["spec.en.test.basic"]
"#,
    );

    let plan = corpus.plan();
    let schema_errors = plan
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code.0 == "E-EXPECT-SCHEMA")
        .count();

    assert_eq!(schema_errors, 3, "{:#?}", plan.diagnostics);
}

#[test]
fn manifest_paths_must_be_clean_relative_paths() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.test.basic"
source = "../doc/spec/en/test.md"
section = "Test"
stage = "lexical"
status = "planned"
required = true
coverage = "pass"
tests = ["../tests/lexical/pass/escape.expect.toml"]
"#,
    );

    let plan = corpus.plan();

    assert_has_code(&plan, "E-MANIFEST-SOURCE-PATH");
    assert_has_code(&plan, "E-MANIFEST-TEST-PATH");
}

#[test]
fn manifest_paths_must_not_contain_current_dir_components() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.test.basic"
source = "doc/./spec/en/test.md"
section = "Test"
stage = "lexical"
status = "planned"
required = true
coverage = "pass"
tests = ["tests/./lexical/pass/linked.expect.toml"]
"#,
    );

    let plan = corpus.plan();

    assert_has_code(&plan, "E-MANIFEST-SOURCE-PATH");
    assert_has_code(&plan, "E-MANIFEST-TEST-PATH");
}

#[test]
fn manifest_duplicate_ids_fail() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.test.basic"
source = "doc/spec/en/test.md"
section = "Test"
stage = "lexical"
status = "planned"
required = true
coverage = "pass"
tests = []

[[requirement]]
id = "spec.en.test.basic"
source = "doc/spec/en/test.md"
section = "Test"
stage = "lexical"
status = "planned"
required = true
coverage = "pass"
tests = []
"#,
    );
    corpus.write("doc/spec/en/test.md", "# Test\n");

    let plan = corpus.plan();

    assert_has_code(&plan, "E-MANIFEST-DUP-ID");
}

#[test]
fn plan_order_is_deterministic_by_expectation_path() {
    let corpus = Corpus::new();
    corpus.add_requirement("spec.en.test.basic", &[]);
    corpus.add_case("tests/lexical/pass/z_case", "z_case", "spec.en.test.basic");
    corpus.add_case("tests/lexical/pass/a_case", "a_case", "spec.en.test.basic");

    let plan = corpus.plan();
    let paths = plan
        .cases
        .iter()
        .map(|case| rel(&corpus.root, &case.expectation_path))
        .collect::<Vec<_>>();

    assert_eq!(
        paths,
        vec![
            PathBuf::from("tests/lexical/pass/a_case.expect.toml"),
            PathBuf::from("tests/lexical/pass/z_case.expect.toml"),
        ]
    );
}

#[test]
fn repository_corpus_plan_succeeds() {
    let plan = repository_plan();

    assert_eq!(plan.error_count(), 0, "{:#?}", plan.diagnostics);
    assert!(plan.cases.iter().any(|case| {
        case.id.0 == "pass_lexical_identifier_basic_001"
            && case
                .expectation
                .tokens
                .iter()
                .any(|token| token.kind == "identifier" && token.lexeme == "alpha")
            && case
                .expectation
                .spec_refs
                .iter()
                .any(|spec_ref| spec_ref.0 == "spec.en.02.lexical.identifiers.basic")
    }));
}

#[test]
fn repository_parse_only_cases_separate_active_runner_seeds_from_future_metadata() {
    let plan = repository_plan();

    let pass_case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "pass_parser_template_arguments_001")
        .expect("parse-only pass seed should be discovered");
    assert_eq!(pass_case.expectation.kind, TestKind::Pass);
    assert_eq!(pass_case.expectation.stage, Stage::ParseOnly);
    assert_eq!(
        pass_case.expectation.expected_outcome,
        ExpectedOutcome::Pass
    );
    assert_eq!(
        pass_case.expectation.expected_phase,
        Some(PipelinePhase::Parse)
    );
    assert!(pass_case.expectation.tokens.is_empty());
    assert!(
        pass_case
            .expectation
            .spec_refs
            .iter()
            .any(|spec_ref| spec_ref.0 == TEMPLATE_ARGUMENTS_REQUIREMENT_ID)
    );
    assert!(
        pass_case
            .expectation
            .tags
            .iter()
            .any(|tag| tag == "active_parse_only")
    );

    let reference_pass = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "pass_parser_template_references_001")
        .expect("parse-only template reference pass seed should be discovered");
    assert_eq!(reference_pass.expectation.kind, TestKind::Pass);
    assert_eq!(reference_pass.expectation.stage, Stage::ParseOnly);
    assert_eq!(
        reference_pass.expectation.expected_outcome,
        ExpectedOutcome::Pass
    );
    assert_eq!(
        reference_pass.expectation.expected_phase,
        Some(PipelinePhase::Parse)
    );
    assert!(
        reference_pass
            .expectation
            .spec_refs
            .iter()
            .any(|spec_ref| spec_ref.0 == TEMPLATE_ARGUMENTS_REQUIREMENT_ID)
    );
    assert!(
        reference_pass
            .expectation
            .tags
            .iter()
            .any(|tag| tag == "active_parse_only")
    );

    let fail_case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "fail_parser_template_arguments_chained_iff_001")
        .expect("parse-only fail seed should be discovered");
    assert_eq!(fail_case.expectation.kind, TestKind::Fail);
    assert_eq!(fail_case.expectation.stage, Stage::ParseOnly);
    assert_eq!(
        fail_case.expectation.expected_outcome,
        ExpectedOutcome::Fail
    );
    assert_eq!(
        fail_case.expectation.expected_phase,
        Some(PipelinePhase::Parse)
    );
    assert_eq!(
        fail_case.expectation.failure_category.as_deref(),
        Some("syntax_error")
    );
    assert_eq!(
        fail_case.expectation.diagnostic_codes,
        vec!["non_associative_operator_chain".to_owned()]
    );
    assert!(
        fail_case
            .expectation
            .tags
            .iter()
            .any(|tag| tag == "active_parse_only")
    );

    let operator_pass = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "pass_parser_operator_terms_001")
        .expect("operator parse-only pass case should be discovered");
    assert_eq!(operator_pass.expectation.kind, TestKind::Pass);
    assert_eq!(operator_pass.expectation.stage, Stage::ParseOnly);
    assert_eq!(
        operator_pass.expectation.expected_phase,
        Some(PipelinePhase::Parse)
    );
    assert!(
        operator_pass
            .expectation
            .spec_refs
            .iter()
            .any(|spec_ref| spec_ref.0 == OPERATOR_PRECEDENCE_REQUIREMENT_ID)
    );
    assert!(
        operator_pass
            .expectation
            .tags
            .iter()
            .any(|tag| tag == "active_parse_only")
    );

    let operator_fail = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "fail_parser_operator_nonassoc_001")
        .expect("operator parse-only fail case should be discovered");
    assert_eq!(operator_fail.expectation.kind, TestKind::Fail);
    assert_eq!(operator_fail.expectation.stage, Stage::ParseOnly);
    assert_eq!(
        operator_fail.expectation.diagnostic_codes,
        vec!["non_associative_operator_chain".to_owned()]
    );
    assert!(
        operator_fail
            .expectation
            .spec_refs
            .iter()
            .any(|spec_ref| spec_ref.0 == OPERATOR_PRECEDENCE_REQUIREMENT_ID)
    );

    let set_comprehension_pass = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "pass_parser_set_comprehensions_001")
        .expect("set-comprehension parse-only pass case should be discovered");
    assert_eq!(set_comprehension_pass.expectation.kind, TestKind::Pass);
    assert_eq!(set_comprehension_pass.expectation.stage, Stage::ParseOnly);
    assert!(
        set_comprehension_pass
            .expectation
            .spec_refs
            .iter()
            .any(|spec_ref| spec_ref.0 == SET_EXPRESSION_REQUIREMENT_ID)
    );
    assert!(
        set_comprehension_pass
            .expectation
            .tags
            .iter()
            .any(|tag| tag == "active_parse_only")
    );

    let atomic_pass = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "pass_parser_atomic_formulas_001")
        .expect("atomic formula parse-only pass case should be discovered");
    assert_eq!(atomic_pass.expectation.kind, TestKind::Pass);
    assert_eq!(atomic_pass.expectation.stage, Stage::ParseOnly);
    assert!(
        atomic_pass
            .expectation
            .spec_refs
            .iter()
            .any(|spec_ref| spec_ref.0 == ATOMIC_FORMULA_REQUIREMENT_ID)
    );
    assert!(
        atomic_pass
            .expectation
            .tags
            .iter()
            .any(|tag| tag == "active_parse_only")
    );

    let atomic_fail = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "fail_parser_atomic_formula_mixed_chain_001")
        .expect("atomic formula parse-only fail case should be discovered");
    assert_eq!(atomic_fail.expectation.kind, TestKind::Fail);
    assert_eq!(atomic_fail.expectation.stage, Stage::ParseOnly);
    assert_eq!(
        atomic_fail.expectation.diagnostic_codes,
        vec!["malformed_term_expression".to_owned()]
    );
    assert!(
        atomic_fail
            .expectation
            .spec_refs
            .iter()
            .any(|spec_ref| spec_ref.0 == ATOMIC_FORMULA_REQUIREMENT_ID)
    );

    let active_cases = active_parse_only_cases(&plan)
        .map(|case| case.id.0.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        active_cases,
        vec![
            "fail_parser_algorithm_control_flow_recovery_001",
            "fail_parser_algorithm_verification_recovery_001",
            "fail_parser_algorithms_claims_recovery_001",
            "fail_parser_annotations_recovery_001",
            "fail_parser_atomic_formula_missing_rhs_001",
            "fail_parser_atomic_formula_mixed_chain_001",
            "fail_parser_block_statements_recovery_001",
            "fail_parser_conclusions_iterative_recovery_001",
            "fail_parser_consider_reconsider_recovery_001",
            "fail_parser_definition_attributes_recovery_001",
            "fail_parser_export_late_001",
            "fail_parser_export_missing_path_001",
            "fail_parser_export_missing_semicolon_001",
            "fail_parser_export_trailing_comma_001",
            "fail_parser_formula_missing_holds_001",
            "fail_parser_formula_missing_not_001",
            "fail_parser_formula_missing_operand_001",
            "fail_parser_formula_missing_quantifier_type_001",
            "fail_parser_formula_missing_st_001",
            "fail_parser_formula_nonassoc_iff_001",
            "fail_parser_formula_unmatched_grouping_001",
            "fail_parser_functor_definitions_recovery_001",
            "fail_parser_import_after_export_001",
            "fail_parser_import_late_001",
            "fail_parser_import_missing_alias_001",
            "fail_parser_import_missing_branch_close_001",
            "fail_parser_import_missing_semicolon_001",
            "fail_parser_inline_definitions_recovery_001",
            "fail_parser_justifications_recovery_001",
            "fail_parser_missing_block_semicolon_001",
            "fail_parser_missing_definition_end_001",
            "fail_parser_missing_semicolon_001",
            "fail_parser_mode_definitions_recovery_001",
            "fail_parser_operator_dangling_001",
            "fail_parser_operator_nonassoc_001",
            "fail_parser_predicate_definitions_recovery_001",
            "fail_parser_primary_terms_missing_delimiter_001",
            "fail_parser_primary_terms_missing_term_001",
            "fail_parser_property_clauses_recovery_001",
            "fail_parser_qua_missing_type_001",
            "fail_parser_recovery_consolidation_001",
            "fail_parser_redefinition_notation_recovery_001",
            "fail_parser_registrations_recovery_001",
            "fail_parser_selector_call_missing_close_001",
            "fail_parser_selector_missing_name_001",
            "fail_parser_set_comprehension_missing_close_001",
            "fail_parser_set_comprehension_missing_condition_001",
            "fail_parser_set_comprehension_missing_generator_001",
            "fail_parser_set_comprehension_missing_is_001",
            "fail_parser_set_comprehension_missing_type_001",
            "fail_parser_simple_statements_recovery_001",
            "fail_parser_stray_end_001",
            "fail_parser_structure_update_missing_close_001",
            "fail_parser_structure_update_missing_value_001",
            "fail_parser_structures_recovery_001",
            "fail_parser_template_arguments_chained_iff_001",
            "fail_parser_theorems_proofs_recovery_001",
            "fail_parser_type_expression_malformed_001",
            "fail_parser_type_expression_missing_bracket_001",
            "fail_parser_unexpected_top_level_token_001",
            "fail_parser_visibility_dangling_001",
            "fail_parser_visibility_duplicate_001",
            "fail_parser_visibility_invalid_target_001",
            "pass_parser_algorithm_control_flow_001",
            "pass_parser_algorithm_verification_001",
            "pass_parser_algorithms_claims_001",
            "pass_parser_annotations_001",
            "pass_parser_atomic_formulas_001",
            "pass_parser_block_statements_001",
            "pass_parser_conclusions_iterative_001",
            "pass_parser_consider_reconsider_001",
            "pass_parser_definition_attributes_001",
            "pass_parser_export_visibility_001",
            "pass_parser_formula_connectives_001",
            "pass_parser_functor_definitions_001",
            "pass_parser_import_items_001",
            "pass_parser_inline_definitions_001",
            "pass_parser_justifications_001",
            "pass_parser_minimal_token_stream_001",
            "pass_parser_mode_definitions_001",
            "pass_parser_module_skeleton_001",
            "pass_parser_operator_terms_001",
            "pass_parser_predicate_definitions_001",
            "pass_parser_primary_terms_001",
            "pass_parser_property_clauses_001",
            "pass_parser_qua_terms_001",
            "pass_parser_redefinition_notation_001",
            "pass_parser_registrations_001",
            "pass_parser_selector_updates_001",
            "pass_parser_set_comprehensions_001",
            "pass_parser_simple_statements_001",
            "pass_parser_structures_001",
            "pass_parser_template_arguments_001",
            "pass_parser_template_references_001",
            "pass_parser_theorems_proofs_001",
            "pass_parser_type_expressions_001",
        ]
    );

    let template_requirement = plan
        .manifest
        .requirements
        .iter()
        .find(|requirement| requirement.id.0 == TEMPLATE_ARGUMENTS_REQUIREMENT_ID)
        .expect("template parse-only requirement should exist");
    assert_eq!(template_requirement.stage, Stage::ParseOnly);
    assert_eq!(
        template_requirement.tests,
        vec![
            PathBuf::from("tests/miz/pass/parser/pass_parser_template_arguments_001.expect.toml"),
            PathBuf::from("tests/miz/pass/parser/pass_parser_template_references_001.expect.toml"),
            PathBuf::from(
                "tests/miz/fail/parser/fail_parser_template_arguments_chained_iff_001.expect.toml"
            ),
        ]
    );

    let operator_requirement = plan
        .manifest
        .requirements
        .iter()
        .find(|requirement| requirement.id.0 == OPERATOR_PRECEDENCE_REQUIREMENT_ID)
        .expect("operator parse-only requirement should exist");
    assert_eq!(operator_requirement.stage, Stage::ParseOnly);
    assert_eq!(
        operator_requirement.tests,
        vec![
            PathBuf::from("tests/miz/pass/parser/pass_parser_operator_terms_001.expect.toml"),
            PathBuf::from("tests/miz/fail/parser/fail_parser_operator_dangling_001.expect.toml"),
            PathBuf::from("tests/miz/fail/parser/fail_parser_operator_nonassoc_001.expect.toml"),
        ]
    );

    let set_expression_requirement = plan
        .manifest
        .requirements
        .iter()
        .find(|requirement| requirement.id.0 == SET_EXPRESSION_REQUIREMENT_ID)
        .expect("set-expression parse-only requirement should exist");
    assert_eq!(set_expression_requirement.stage, Stage::ParseOnly);
    assert_eq!(
        set_expression_requirement.tests,
        vec![
            PathBuf::from("tests/miz/pass/parser/pass_parser_primary_terms_001.expect.toml"),
            PathBuf::from("tests/miz/pass/parser/pass_parser_set_comprehensions_001.expect.toml"),
            PathBuf::from(
                "tests/miz/fail/parser/fail_parser_set_comprehension_missing_close_001.expect.toml"
            ),
            PathBuf::from(
                "tests/miz/fail/parser/fail_parser_set_comprehension_missing_condition_001.expect.toml"
            ),
            PathBuf::from(
                "tests/miz/fail/parser/fail_parser_set_comprehension_missing_generator_001.expect.toml"
            ),
            PathBuf::from(
                "tests/miz/fail/parser/fail_parser_set_comprehension_missing_is_001.expect.toml"
            ),
            PathBuf::from(
                "tests/miz/fail/parser/fail_parser_set_comprehension_missing_type_001.expect.toml"
            ),
        ]
    );

    let atomic_requirement = plan
        .manifest
        .requirements
        .iter()
        .find(|requirement| requirement.id.0 == ATOMIC_FORMULA_REQUIREMENT_ID)
        .expect("atomic formula parse-only requirement should exist");
    assert_eq!(atomic_requirement.stage, Stage::ParseOnly);
    assert_eq!(
        atomic_requirement.tests,
        vec![
            PathBuf::from("tests/miz/pass/parser/pass_parser_atomic_formulas_001.expect.toml"),
            PathBuf::from(
                "tests/miz/fail/parser/fail_parser_atomic_formula_missing_rhs_001.expect.toml"
            ),
            PathBuf::from(
                "tests/miz/fail/parser/fail_parser_atomic_formula_mixed_chain_001.expect.toml"
            ),
        ]
    );

    let formula_connectives_requirement = plan
        .manifest
        .requirements
        .iter()
        .find(|requirement| requirement.id.0 == FORMULA_CONNECTIVES_REQUIREMENT_ID)
        .expect("formula connective parse-only requirement should exist");
    assert_eq!(formula_connectives_requirement.stage, Stage::ParseOnly);
    assert_eq!(
        formula_connectives_requirement.tests,
        vec![
            PathBuf::from("tests/miz/pass/parser/pass_parser_formula_connectives_001.expect.toml"),
            PathBuf::from(
                "tests/miz/fail/parser/fail_parser_formula_missing_holds_001.expect.toml"
            ),
            PathBuf::from("tests/miz/fail/parser/fail_parser_formula_missing_not_001.expect.toml"),
            PathBuf::from(
                "tests/miz/fail/parser/fail_parser_formula_missing_operand_001.expect.toml"
            ),
            PathBuf::from(
                "tests/miz/fail/parser/fail_parser_formula_missing_quantifier_type_001.expect.toml"
            ),
            PathBuf::from("tests/miz/fail/parser/fail_parser_formula_missing_st_001.expect.toml"),
            PathBuf::from("tests/miz/fail/parser/fail_parser_formula_nonassoc_iff_001.expect.toml"),
            PathBuf::from(
                "tests/miz/fail/parser/fail_parser_formula_unmatched_grouping_001.expect.toml"
            ),
        ]
    );

    let attribute_definitions_requirement = plan
        .manifest
        .requirements
        .iter()
        .find(|requirement| requirement.id.0 == ATTRIBUTE_DEFINITIONS_REQUIREMENT_ID)
        .expect("attribute-definition parse-only requirement should exist");
    assert_eq!(attribute_definitions_requirement.stage, Stage::ParseOnly);
    assert_eq!(
        attribute_definitions_requirement.tests,
        vec![
            PathBuf::from(
                "tests/miz/pass/parser/pass_parser_definition_attributes_001.expect.toml"
            ),
            PathBuf::from(
                "tests/miz/fail/parser/fail_parser_definition_attributes_recovery_001.expect.toml"
            ),
        ]
    );

    let predicate_definitions_requirement = plan
        .manifest
        .requirements
        .iter()
        .find(|requirement| requirement.id.0 == PREDICATE_DEFINITIONS_REQUIREMENT_ID)
        .expect("predicate-definition parse-only requirement should exist");
    assert_eq!(predicate_definitions_requirement.stage, Stage::ParseOnly);
    assert_eq!(
        predicate_definitions_requirement.tests,
        vec![
            PathBuf::from(
                "tests/miz/pass/parser/pass_parser_predicate_definitions_001.expect.toml"
            ),
            PathBuf::from(
                "tests/miz/fail/parser/fail_parser_predicate_definitions_recovery_001.expect.toml"
            ),
        ]
    );

    let functor_definitions_requirement = plan
        .manifest
        .requirements
        .iter()
        .find(|requirement| requirement.id.0 == FUNCTOR_DEFINITIONS_REQUIREMENT_ID)
        .expect("functor-definition parse-only requirement should exist");
    assert_eq!(functor_definitions_requirement.stage, Stage::ParseOnly);
    assert_eq!(
        functor_definitions_requirement.tests,
        vec![
            PathBuf::from("tests/miz/pass/parser/pass_parser_functor_definitions_001.expect.toml"),
            PathBuf::from(
                "tests/miz/fail/parser/fail_parser_functor_definitions_recovery_001.expect.toml"
            ),
        ]
    );

    let mode_definitions_requirement = plan
        .manifest
        .requirements
        .iter()
        .find(|requirement| requirement.id.0 == MODE_DEFINITIONS_REQUIREMENT_ID)
        .expect("mode-definition parse-only requirement should exist");
    assert_eq!(mode_definitions_requirement.stage, Stage::ParseOnly);
    assert_eq!(
        mode_definitions_requirement.tests,
        vec![
            PathBuf::from("tests/miz/pass/parser/pass_parser_mode_definitions_001.expect.toml"),
            PathBuf::from(
                "tests/miz/fail/parser/fail_parser_mode_definitions_recovery_001.expect.toml"
            ),
        ]
    );

    let redefinition_notation_requirement = plan
        .manifest
        .requirements
        .iter()
        .find(|requirement| requirement.id.0 == REDEFINITION_NOTATION_REQUIREMENT_ID)
        .expect("redefinition/notation parse-only requirement should exist");
    assert_eq!(redefinition_notation_requirement.stage, Stage::ParseOnly);
    assert_eq!(
        redefinition_notation_requirement.tests,
        vec![
            PathBuf::from(
                "tests/miz/pass/parser/pass_parser_redefinition_notation_001.expect.toml"
            ),
            PathBuf::from(
                "tests/miz/fail/parser/fail_parser_redefinition_notation_recovery_001.expect.toml"
            ),
        ]
    );

    let property_clauses_requirement = plan
        .manifest
        .requirements
        .iter()
        .find(|requirement| requirement.id.0 == PROPERTY_CLAUSES_REQUIREMENT_ID)
        .expect("property-clause parse-only requirement should exist");
    assert_eq!(property_clauses_requirement.stage, Stage::ParseOnly);
    assert_eq!(
        property_clauses_requirement.tests,
        vec![
            PathBuf::from("tests/miz/pass/parser/pass_parser_property_clauses_001.expect.toml"),
            PathBuf::from(
                "tests/miz/fail/parser/fail_parser_property_clauses_recovery_001.expect.toml"
            ),
        ]
    );

    let structures_requirement = plan
        .manifest
        .requirements
        .iter()
        .find(|requirement| requirement.id.0 == STRUCTURES_REQUIREMENT_ID)
        .expect("structure-definition parse-only requirement should exist");
    assert_eq!(structures_requirement.stage, Stage::ParseOnly);
    assert_eq!(
        structures_requirement.tests,
        vec![
            PathBuf::from("tests/miz/pass/parser/pass_parser_structures_001.expect.toml"),
            PathBuf::from("tests/miz/fail/parser/fail_parser_structures_recovery_001.expect.toml"),
        ]
    );

    let correctness_conditions_requirement = plan
        .manifest
        .requirements
        .iter()
        .find(|requirement| requirement.id.0 == CORRECTNESS_CONDITIONS_REQUIREMENT_ID)
        .expect("correctness-condition parse-only requirement should exist");
    assert_eq!(correctness_conditions_requirement.stage, Stage::ParseOnly);
    assert_eq!(
        correctness_conditions_requirement.tests,
        vec![
            PathBuf::from(
                "tests/miz/pass/parser/pass_parser_definition_attributes_001.expect.toml"
            ),
            PathBuf::from(
                "tests/miz/fail/parser/fail_parser_definition_attributes_recovery_001.expect.toml"
            ),
        ]
    );

    let registrations_requirement = plan
        .manifest
        .requirements
        .iter()
        .find(|requirement| requirement.id.0 == REGISTRATIONS_REQUIREMENT_ID)
        .expect("registration parse-only requirement should exist");
    assert_eq!(registrations_requirement.stage, Stage::ParseOnly);
    assert_eq!(
        registrations_requirement.tests,
        vec![
            PathBuf::from("tests/miz/pass/parser/pass_parser_registrations_001.expect.toml"),
            PathBuf::from(
                "tests/miz/fail/parser/fail_parser_registrations_recovery_001.expect.toml"
            ),
        ]
    );

    let algorithms_claims_requirement = plan
        .manifest
        .requirements
        .iter()
        .find(|requirement| requirement.id.0 == ALGORITHMS_CLAIMS_REQUIREMENT_ID)
        .expect("algorithm/claim parse-only requirement should exist");
    assert_eq!(algorithms_claims_requirement.stage, Stage::ParseOnly);
    assert_eq!(
        algorithms_claims_requirement.tests,
        vec![
            PathBuf::from("tests/miz/pass/parser/pass_parser_algorithms_claims_001.expect.toml"),
            PathBuf::from(
                "tests/miz/fail/parser/fail_parser_algorithms_claims_recovery_001.expect.toml"
            ),
        ]
    );

    let algorithm_control_flow_requirement = plan
        .manifest
        .requirements
        .iter()
        .find(|requirement| requirement.id.0 == ALGORITHM_CONTROL_FLOW_REQUIREMENT_ID)
        .expect("algorithm control-flow parse-only requirement should exist");
    assert_eq!(algorithm_control_flow_requirement.stage, Stage::ParseOnly);
    assert_eq!(
        algorithm_control_flow_requirement.tests,
        vec![
            PathBuf::from(
                "tests/miz/pass/parser/pass_parser_algorithm_control_flow_001.expect.toml"
            ),
            PathBuf::from(
                "tests/miz/fail/parser/fail_parser_algorithm_control_flow_recovery_001.expect.toml"
            ),
        ]
    );

    let algorithm_verification_requirement = plan
        .manifest
        .requirements
        .iter()
        .find(|requirement| requirement.id.0 == ALGORITHM_VERIFICATION_REQUIREMENT_ID)
        .expect("algorithm verification parse-only requirement should exist");
    assert_eq!(algorithm_verification_requirement.stage, Stage::ParseOnly);
    assert_eq!(
        algorithm_verification_requirement.tests,
        vec![
            PathBuf::from(
                "tests/miz/pass/parser/pass_parser_algorithm_verification_001.expect.toml"
            ),
            PathBuf::from(
                "tests/miz/fail/parser/fail_parser_algorithm_verification_recovery_001.expect.toml"
            ),
        ]
    );

    let annotations_requirement = plan
        .manifest
        .requirements
        .iter()
        .find(|requirement| requirement.id.0 == ANNOTATIONS_REQUIREMENT_ID)
        .expect("annotation parse-only requirement should exist");
    assert_eq!(annotations_requirement.stage, Stage::ParseOnly);
    assert_eq!(
        annotations_requirement.tests,
        vec![
            PathBuf::from("tests/miz/pass/parser/pass_parser_annotations_001.expect.toml"),
            PathBuf::from("tests/miz/fail/parser/fail_parser_annotations_recovery_001.expect.toml"),
        ]
    );

    for requirement_id in [
        "spec.en.elaboration.choice_comprehension.lowering",
        "spec.en.binding.substitution.capture_avoidance",
        "spec.en.algorithm.vc.assignment_loop_exits",
        "spec.en.type_soundness.escape_and_guard_failures",
    ] {
        let requirement = plan
            .manifest
            .requirements
            .iter()
            .find(|requirement| requirement.id.0 == requirement_id)
            .unwrap_or_else(|| panic!("planned requirement `{requirement_id}` should exist"));
        assert!(
            requirement.tests.is_empty(),
            "`{requirement_id}` should remain unlinked until its semantic stage exists"
        );
        assert!(
            requirement.deferred_reason.is_some(),
            "`{requirement_id}` should keep a deferral reason"
        );
    }
}

#[test]
fn repository_parser_reserved_words_are_covered_or_explicitly_deferred() {
    let config = repository_config();
    let plan = repository_plan();
    let reserved_words = reserved_words_from_appendix_a(&config.workspace_root);
    let covered_words = active_parser_corpus_reserved_words(&plan, &reserved_words);
    let deferred_reserved_words = PARSER_DEFERRED_RESERVED_WORDS
        .iter()
        .map(|word| (*word).to_owned())
        .collect::<BTreeSet<_>>();

    let missing_words = reserved_words
        .difference(&covered_words)
        .filter(|word| !deferred_reserved_words.contains(*word))
        .cloned()
        .collect::<Vec<_>>();
    assert!(
        missing_words.is_empty(),
        "reserved words from appendix A must appear as frontend ReservedWord \
         tokens in active parser corpus sources or be listed as parser-deferred \
         reserved words: {missing_words:?}"
    );

    let unknown_deferred_words = deferred_reserved_words
        .difference(&reserved_words)
        .cloned()
        .collect::<Vec<_>>();
    assert!(
        unknown_deferred_words.is_empty(),
        "parser-deferred reserved words must still exist in appendix A: {unknown_deferred_words:?}"
    );

    let stale_deferred_words = deferred_reserved_words
        .intersection(&covered_words)
        .cloned()
        .collect::<Vec<_>>();
    assert!(
        stale_deferred_words.is_empty(),
        "parser-deferred reserved words now appear as frontend ReservedWord tokens \
         in active parser corpus sources; remove them from \
         PARSER_DEFERRED_RESERVED_WORDS and update the task-43 audit: \
         {stale_deferred_words:?}"
    );
}

#[test]
fn repository_parse_only_runner_executes_active_minimal_parser_seeds() {
    let config = repository_config();
    let report = run_parse_only_corpus(&config).unwrap();

    assert_eq!(report.error_count(), 0, "{:#?}", report.diagnostics);
    assert_eq!(report.results.len(), 96);
    assert_eq!(report.passed_count(), 96);
    assert_eq!(report.failed_count(), 0);
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_parser_algorithm_control_flow_001"
            && result.actual_diagnostic_codes.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_parser_algorithm_verification_001"
            && result.actual_diagnostic_codes.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_parser_annotations_001" && result.actual_diagnostic_codes.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_parser_algorithms_claims_001"
            && result.actual_diagnostic_codes.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_parser_atomic_formulas_001"
            && result.actual_diagnostic_codes.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_parser_block_statements_001"
            && result.actual_diagnostic_codes.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_parser_conclusions_iterative_001"
            && result.actual_diagnostic_codes.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_parser_definition_attributes_001"
            && result.actual_diagnostic_codes.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_parser_predicate_definitions_001"
            && result.actual_diagnostic_codes.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_parser_consider_reconsider_001"
            && result.actual_diagnostic_codes.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_parser_formula_connectives_001"
            && result.actual_diagnostic_codes.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_parser_functor_definitions_001"
            && result.actual_diagnostic_codes.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_parser_mode_definitions_001"
            && result.actual_diagnostic_codes.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_parser_redefinition_notation_001"
            && result.actual_diagnostic_codes.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_parser_registrations_001" && result.actual_diagnostic_codes.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_parser_property_clauses_001"
            && result.actual_diagnostic_codes.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_parser_structures_001" && result.actual_diagnostic_codes.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_parser_template_arguments_001"
            && result.actual_diagnostic_codes.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_parser_template_references_001"
            && result.actual_diagnostic_codes.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_parser_simple_statements_001"
            && result.actual_diagnostic_codes.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_parser_justifications_001" && result.actual_diagnostic_codes.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_atomic_formula_missing_rhs_001"
            && result.actual_diagnostic_codes == vec!["malformed_term_expression".to_owned()]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_atomic_formula_mixed_chain_001"
            && result.actual_diagnostic_codes == vec!["malformed_term_expression".to_owned()]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_block_statements_recovery_001"
            && result.actual_diagnostic_codes
                == vec![
                    "lexing:ScopeSkeleton(MissingEnd)".to_owned(),
                    "lexing:ScopeSkeleton(MissingEnd)".to_owned(),
                    "malformed_formula_expression".to_owned(),
                    "missing_semicolon".to_owned(),
                    "malformed_formula_expression".to_owned(),
                    "missing_semicolon".to_owned(),
                    "malformed_justification".to_owned(),
                    "missing_semicolon".to_owned(),
                    "malformed_formula_expression".to_owned(),
                    "missing_end".to_owned(),
                    "missing_semicolon".to_owned(),
                    "unexpected_top_level_token".to_owned(),
                    "missing_end".to_owned(),
                    "missing_end".to_owned(),
                ]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_definition_attributes_recovery_001"
            && result.actual_diagnostic_codes
                == vec![
                    "lexing:ScopeSkeleton(MissingEnd)".to_owned(),
                    "malformed_term_expression".to_owned(),
                    "malformed_term_expression".to_owned(),
                    "malformed_term_expression".to_owned(),
                    "malformed_formula_expression".to_owned(),
                    "malformed_justification".to_owned(),
                    "missing_end".to_owned(),
                    "missing_semicolon".to_owned(),
                ]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_predicate_definitions_recovery_001"
            && result.actual_diagnostic_codes
                == vec![
                    "malformed_term_expression".to_owned(),
                    "malformed_term_expression".to_owned(),
                    "malformed_formula_expression".to_owned(),
                    "malformed_term_expression".to_owned(),
                    "malformed_term_expression".to_owned(),
                    "malformed_term_expression".to_owned(),
                    "malformed_term_expression".to_owned(),
                    "malformed_formula_expression".to_owned(),
                    "malformed_formula_expression".to_owned(),
                    "malformed_term_expression".to_owned(),
                ]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_functor_definitions_recovery_001"
            && result.actual_diagnostic_codes
                == vec![
                    "malformed_term_expression".to_owned(),
                    "malformed_term_expression".to_owned(),
                    "malformed_type_expression".to_owned(),
                    "malformed_term_expression".to_owned(),
                    "malformed_term_expression".to_owned(),
                    "malformed_formula_expression".to_owned(),
                    "malformed_term_expression".to_owned(),
                    "malformed_type_expression".to_owned(),
                    "malformed_type_expression".to_owned(),
                    "malformed_formula_expression".to_owned(),
                    "malformed_formula_expression".to_owned(),
                    "malformed_formula_expression".to_owned(),
                    "malformed_formula_expression".to_owned(),
                    "malformed_formula_expression".to_owned(),
                    "malformed_formula_expression".to_owned(),
                    "malformed_term_expression".to_owned(),
                    "malformed_formula_expression".to_owned(),
                    "malformed_term_expression".to_owned(),
                ]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_redefinition_notation_recovery_001"
            && result.actual_diagnostic_codes
                == vec![
                    "malformed_term_expression".to_owned(),
                    "malformed_term_expression".to_owned(),
                    "malformed_justification".to_owned(),
                    "malformed_term_expression".to_owned(),
                    "malformed_term_expression".to_owned(),
                    "malformed_justification".to_owned(),
                    "malformed_type_expression".to_owned(),
                    "malformed_justification".to_owned(),
                    "malformed_term_expression".to_owned(),
                    "malformed_formula_expression".to_owned(),
                    "malformed_term_expression".to_owned(),
                ]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_registrations_recovery_001"
            && result.actual_diagnostic_codes
                == vec![
                    "malformed_term_expression".to_owned(),
                    "malformed_type_expression".to_owned(),
                    "malformed_term_expression".to_owned(),
                    "malformed_justification".to_owned(),
                    "malformed_justification".to_owned(),
                    "malformed_type_expression".to_owned(),
                    "missing_semicolon".to_owned(),
                    "malformed_justification".to_owned(),
                    "malformed_justification".to_owned(),
                    "missing_semicolon".to_owned(),
                    "malformed_type_expression".to_owned(),
                    "malformed_term_expression".to_owned(),
                    "malformed_term_expression".to_owned(),
                    "malformed_type_expression".to_owned(),
                    "malformed_type_expression".to_owned(),
                    "malformed_term_expression".to_owned(),
                    "malformed_justification".to_owned(),
                    "missing_end".to_owned(),
                    "missing_semicolon".to_owned(),
                ]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_property_clauses_recovery_001"
            && result.actual_diagnostic_codes
                == vec![
                    "malformed_justification".to_owned(),
                    "malformed_justification".to_owned(),
                    "malformed_justification".to_owned(),
                    "missing_semicolon".to_owned(),
                ]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_structures_recovery_001"
            && result.actual_diagnostic_codes
                == vec![
                    "malformed_term_expression".to_owned(),
                    "missing_semicolon".to_owned(),
                    "malformed_type_expression".to_owned(),
                    "malformed_term_expression".to_owned(),
                    "malformed_term_expression".to_owned(),
                    "malformed_term_expression".to_owned(),
                    "malformed_term_expression".to_owned(),
                    "malformed_term_expression".to_owned(),
                    "malformed_type_expression".to_owned(),
                    "malformed_term_expression".to_owned(),
                    "malformed_justification".to_owned(),
                ]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_parser_inline_definitions_001"
            && result.actual_diagnostic_codes.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_parser_theorems_proofs_001"
            && result.actual_diagnostic_codes.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_inline_definitions_recovery_001"
            && result.actual_diagnostic_codes
                == vec![
                    "lexing:ScopeSkeleton(UnsupportedBinderShape)".to_owned(),
                    "malformed_formula_expression".to_owned(),
                    "missing_semicolon".to_owned(),
                    "malformed_formula_expression".to_owned(),
                    "missing_semicolon".to_owned(),
                    "malformed_term_expression".to_owned(),
                    "malformed_term_expression".to_owned(),
                    "malformed_type_expression".to_owned(),
                    "malformed_type_expression".to_owned(),
                    "malformed_type_expression".to_owned(),
                    "malformed_term_expression".to_owned(),
                    "malformed_type_expression".to_owned(),
                    "malformed_type_expression".to_owned(),
                    "malformed_term_expression".to_owned(),
                    "malformed_term_expression".to_owned(),
                    "malformed_formula_expression".to_owned(),
                    "malformed_term_expression".to_owned(),
                    "malformed_formula_expression".to_owned(),
                    "missing_semicolon".to_owned(),
                ]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_theorems_proofs_recovery_001"
            && result.actual_diagnostic_codes
                == vec![
                    "lexing:ScopeSkeleton(MissingEnd)".to_owned(),
                    "lexing:ScopeSkeleton(MissingEnd)".to_owned(),
                    "malformed_term_expression".to_owned(),
                    "malformed_formula_expression".to_owned(),
                    "malformed_formula_expression".to_owned(),
                    "missing_end".to_owned(),
                    "missing_semicolon".to_owned(),
                    "missing_end".to_owned(),
                    "missing_semicolon".to_owned(),
                    "missing_end".to_owned(),
                    "missing_end".to_owned(),
                ]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_conclusions_iterative_recovery_001"
            && result.actual_diagnostic_codes
                == vec![
                    "malformed_formula_expression".to_owned(),
                    "missing_semicolon".to_owned(),
                    "malformed_formula_expression".to_owned(),
                    "malformed_term_expression".to_owned(),
                    "malformed_term_expression".to_owned(),
                    "malformed_justification".to_owned(),
                ]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_consider_reconsider_recovery_001"
            && result.actual_diagnostic_codes
                == vec![
                    "malformed_type_expression".to_owned(),
                    "malformed_formula_expression".to_owned(),
                    "malformed_formula_expression".to_owned(),
                    "malformed_justification".to_owned(),
                    "malformed_justification".to_owned(),
                    "malformed_term_expression".to_owned(),
                    "malformed_type_expression".to_owned(),
                    "malformed_type_expression".to_owned(),
                    "malformed_justification".to_owned(),
                    "malformed_term_expression".to_owned(),
                ]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_formula_missing_operand_001"
            && result.actual_diagnostic_codes == vec!["malformed_formula_expression".to_owned()]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_formula_missing_quantifier_type_001"
            && result.actual_diagnostic_codes == vec!["malformed_type_expression".to_owned()]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_formula_missing_not_001"
            && result.actual_diagnostic_codes == vec!["malformed_formula_expression".to_owned()]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_formula_missing_st_001"
            && result.actual_diagnostic_codes == vec!["malformed_formula_expression".to_owned()]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_formula_missing_holds_001"
            && result.actual_diagnostic_codes == vec!["malformed_formula_expression".to_owned()]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_simple_statements_recovery_001"
            && result.actual_diagnostic_codes
                == vec![
                    "malformed_type_expression".to_owned(),
                    "malformed_formula_expression".to_owned(),
                    "malformed_term_expression".to_owned(),
                    "malformed_term_expression".to_owned(),
                    "malformed_term_expression".to_owned(),
                    "malformed_term_expression".to_owned(),
                    "malformed_term_expression".to_owned(),
                    "missing_semicolon".to_owned(),
                    "missing_semicolon".to_owned(),
                ]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_justifications_recovery_001"
            && result.actual_diagnostic_codes
                == vec![
                    "malformed_justification".to_owned(),
                    "malformed_justification".to_owned(),
                    "malformed_justification".to_owned(),
                    "malformed_justification".to_owned(),
                    "malformed_justification".to_owned(),
                    "malformed_justification".to_owned(),
                    "malformed_justification".to_owned(),
                    "malformed_justification".to_owned(),
                    "malformed_justification".to_owned(),
                    "malformed_justification".to_owned(),
                    "malformed_justification".to_owned(),
                ]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_algorithm_control_flow_recovery_001"
            && result.actual_diagnostic_codes
                == vec![
                    "malformed_formula_expression".to_owned(),
                    "malformed_term_expression".to_owned(),
                    "malformed_term_expression".to_owned(),
                    "malformed_term_expression".to_owned(),
                    "malformed_term_expression".to_owned(),
                ]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_algorithm_verification_recovery_001"
            && result.actual_diagnostic_codes
                == vec![
                    "malformed_formula_expression".to_owned(),
                    "malformed_formula_expression".to_owned(),
                    "malformed_formula_expression".to_owned(),
                    "malformed_formula_expression".to_owned(),
                    "malformed_formula_expression".to_owned(),
                    "malformed_formula_expression".to_owned(),
                    "malformed_formula_expression".to_owned(),
                    "malformed_term_expression".to_owned(),
                    "malformed_term_expression".to_owned(),
                    "malformed_term_expression".to_owned(),
                    "malformed_formula_expression".to_owned(),
                    "malformed_formula_expression".to_owned(),
                ]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_algorithms_claims_recovery_001"
            && result.actual_diagnostic_codes
                == vec![
                    "malformed_term_expression".to_owned(),
                    "malformed_term_expression".to_owned(),
                    "malformed_type_expression".to_owned(),
                    "malformed_formula_expression".to_owned(),
                    "malformed_term_expression".to_owned(),
                    "malformed_term_expression".to_owned(),
                    "malformed_term_expression".to_owned(),
                    "malformed_term_expression".to_owned(),
                    "malformed_term_expression".to_owned(),
                    "malformed_term_expression".to_owned(),
                    "malformed_term_expression".to_owned(),
                    "missing_semicolon".to_owned(),
                    "malformed_term_expression".to_owned(),
                    "malformed_formula_expression".to_owned(),
                    "missing_semicolon".to_owned(),
                ]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_annotations_recovery_001"
            && result.actual_diagnostic_codes
                == vec![
                    "malformed_annotation".to_owned(),
                    "malformed_annotation".to_owned(),
                    "malformed_annotation".to_owned(),
                    "malformed_annotation".to_owned(),
                    "malformed_annotation".to_owned(),
                    "malformed_term_expression".to_owned(),
                    "malformed_annotation".to_owned(),
                    "malformed_annotation".to_owned(),
                    "malformed_annotation".to_owned(),
                    "malformed_annotation".to_owned(),
                    "malformed_annotation".to_owned(),
                    "malformed_annotation".to_owned(),
                    "malformed_annotation".to_owned(),
                    "malformed_annotation".to_owned(),
                ]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_formula_nonassoc_iff_001"
            && result.actual_diagnostic_codes == vec!["non_associative_operator_chain".to_owned()]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_template_arguments_chained_iff_001"
            && result.actual_diagnostic_codes == vec!["non_associative_operator_chain".to_owned()]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_formula_unmatched_grouping_001"
            && result.actual_diagnostic_codes == vec!["malformed_formula_expression".to_owned()]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_parser_export_visibility_001"
            && result.actual_diagnostic_codes.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_export_late_001"
            && result.actual_diagnostic_codes == vec!["unexpected_top_level_token".to_owned()]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_import_after_export_001"
            && result.actual_diagnostic_codes == vec!["unexpected_top_level_token".to_owned()]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_export_missing_path_001"
            && result.actual_diagnostic_codes == vec!["malformed_export".to_owned()]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_export_trailing_comma_001"
            && result.actual_diagnostic_codes == vec!["malformed_export".to_owned()]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_export_missing_semicolon_001"
            && result.actual_diagnostic_codes == vec!["missing_semicolon".to_owned()]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_visibility_dangling_001"
            && result.actual_diagnostic_codes == vec!["malformed_visibility".to_owned()]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_visibility_duplicate_001"
            && result.actual_diagnostic_codes == vec!["malformed_visibility".to_owned()]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_visibility_invalid_target_001"
            && result.actual_diagnostic_codes == vec!["malformed_visibility".to_owned()]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_parser_import_items_001" && result.actual_diagnostic_codes.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_import_late_001"
            && result.actual_diagnostic_codes == vec!["unexpected_top_level_token".to_owned()]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_import_missing_alias_001"
            && result.actual_diagnostic_codes == vec!["malformed_import".to_owned()]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_import_missing_branch_close_001"
            && result.actual_diagnostic_codes == vec!["malformed_import".to_owned()]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_import_missing_semicolon_001"
            && result.actual_diagnostic_codes == vec!["missing_semicolon".to_owned()]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_parser_minimal_token_stream_001"
            && result.actual_diagnostic_codes.is_empty()
            && result.snapshot_failure.is_none()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_missing_block_semicolon_001"
            && result.actual_diagnostic_codes == vec!["missing_semicolon".to_owned()]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_missing_definition_end_001"
            && result.actual_diagnostic_codes
                == vec!["missing_end".to_owned(), "missing_semicolon".to_owned()]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_missing_semicolon_001"
            && result.actual_diagnostic_codes == vec!["missing_semicolon".to_owned()]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_operator_dangling_001"
            && result.actual_diagnostic_codes == vec!["dangling_operator".to_owned()]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_operator_nonassoc_001"
            && result.actual_diagnostic_codes == vec!["non_associative_operator_chain".to_owned()]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_primary_terms_missing_delimiter_001"
            && result.actual_diagnostic_codes == vec!["malformed_term_expression".to_owned()]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_primary_terms_missing_term_001"
            && result.actual_diagnostic_codes == vec!["malformed_term_expression".to_owned()]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_qua_missing_type_001"
            && result.actual_diagnostic_codes == vec!["malformed_type_expression".to_owned()]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_selector_call_missing_close_001"
            && result.actual_diagnostic_codes == vec!["malformed_term_expression".to_owned()]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_selector_missing_name_001"
            && result.actual_diagnostic_codes == vec!["malformed_term_expression".to_owned()]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_set_comprehension_missing_close_001"
            && result.actual_diagnostic_codes == vec!["malformed_term_expression".to_owned()]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_set_comprehension_missing_condition_001"
            && result.actual_diagnostic_codes == vec!["malformed_formula_expression".to_owned()]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_set_comprehension_missing_generator_001"
            && result.actual_diagnostic_codes == vec!["malformed_term_expression".to_owned()]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_set_comprehension_missing_is_001"
            && result.actual_diagnostic_codes == vec!["malformed_term_expression".to_owned()]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_set_comprehension_missing_type_001"
            && result.actual_diagnostic_codes == vec!["malformed_type_expression".to_owned()]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_stray_end_001"
            && result.actual_diagnostic_codes == vec!["unrecoverable_input".to_owned()]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_structure_update_missing_close_001"
            && result.actual_diagnostic_codes == vec!["malformed_term_expression".to_owned()]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_structure_update_missing_value_001"
            && result.actual_diagnostic_codes == vec!["malformed_term_expression".to_owned()]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_unexpected_top_level_token_001"
            && result.actual_diagnostic_codes == vec!["unexpected_top_level_token".to_owned()]
            && result.snapshot_failure.is_none()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_parser_module_skeleton_001"
            && result.actual_diagnostic_codes.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_parser_operator_terms_001" && result.actual_diagnostic_codes.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_parser_primary_terms_001" && result.actual_diagnostic_codes.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_parser_qua_terms_001" && result.actual_diagnostic_codes.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_parser_selector_updates_001"
            && result.actual_diagnostic_codes.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_parser_set_comprehensions_001"
            && result.actual_diagnostic_codes.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_parser_type_expressions_001"
            && result.actual_diagnostic_codes.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_type_expression_malformed_001"
            && result.actual_diagnostic_codes == vec!["malformed_type_expression".to_owned()]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_parser_type_expression_missing_bracket_001"
            && result.actual_diagnostic_codes == vec!["malformed_type_expression".to_owned()]
    }));
}

#[test]
fn parse_only_runner_reports_mismatched_active_expectation() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/miz/pass/parser/pass_parser_mismatch_001.miz",
        "alpha;\n",
    );
    corpus.write(
        "tests/miz/pass/parser/pass_parser_mismatch_001.expect.toml",
        r#"schema_version = 1
id = "pass_parser_mismatch_001"
kind = "pass"
stage = "parse_only"
domain = "parser.test"
source = "pass_parser_mismatch_001.miz"
expected_outcome = "pass"
expected_phase = "parse"
diagnostic_codes = ["missing_end"]
tags = ["active_parse_only"]
spec_refs = ["spec.en.test.parse"]
"#,
    );
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.test.parse"
source = "doc/spec/en/test.md"
section = "Test"
stage = "parse_only"
status = "covered"
required = true
coverage = "pass"
tests = ["tests/miz/pass/parser/pass_parser_mismatch_001.expect.toml"]
"#,
    );
    corpus.write("doc/spec/en/test.md", "# Test\n");

    let report = run_parse_only_corpus(&corpus.config()).unwrap();

    assert_eq!(report.failed_count(), 1);
    assert_has_report_code(&report, "E-PARSE-ONLY-ASSERT");
}

#[test]
fn parse_only_runner_rejects_active_tag_on_non_parse_case() {
    let corpus = Corpus::new();
    corpus.write("tests/lexical/pass/tagged_lexical.src", "alpha");
    corpus.write(
        "tests/lexical/pass/tagged_lexical.expect.toml",
        r#"schema_version = 1
id = "tagged_lexical"
kind = "pass"
stage = "lexical"
domain = "lexical"
source = "tagged_lexical.src"
expected_outcome = "pass"
expected_phase = "lex"
diagnostic_codes = []
tags = ["active_parse_only"]
spec_refs = ["spec.en.test.lexical"]
"#,
    );
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.test.lexical"
source = "doc/spec/en/test.md"
section = "Test"
stage = "lexical"
status = "covered"
required = true
coverage = "pass"
tests = ["tests/lexical/pass/tagged_lexical.expect.toml"]
"#,
    );
    corpus.write("doc/spec/en/test.md", "# Test\n");

    let report = run_parse_only_corpus(&corpus.config()).unwrap();

    assert_eq!(report.results.len(), 0);
    assert_has_report_code(&report, "E-PARSE-ONLY-ACTIVE-GATE");
}

#[test]
fn parse_only_cli_reports_active_runner_summary() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_mizar-test"))
        .arg("parse-only")
        .arg("--workspace-root")
        .arg(repository_config().workspace_root)
        .output()
        .expect("mizar-test parse-only should run");

    assert!(
        output.status.success(),
        "parse-only CLI failed: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("parse-only cases: 96"));
    assert!(stdout.contains("passed: 96"));
    assert!(stdout.contains("failed: 0"));
}

#[test]
fn lexical_token_expectations_parse() {
    let corpus = Corpus::new();
    corpus.add_requirement("spec.en.test.basic", &[]);
    corpus.write("tests/lexical/pass/tokenized.src", "alpha");
    corpus.write(
        "tests/lexical/pass/tokenized.expect.toml",
        r#"schema_version = 1
id = "tokenized"
kind = "pass"
stage = "lexical"
domain = "lexical"
source = "tokenized.src"
expected_outcome = "pass"
expected_phase = "lex"
diagnostic_codes = []
spec_refs = ["spec.en.test.basic"]

[[tokens]]
kind = "identifier"
lexeme = "alpha"
span_start_line = 1
span_start_col = 1
span_end_line = 1
span_end_col = 6
"#,
    );

    let plan = corpus.plan();

    assert_eq!(plan.error_count(), 0, "{:#?}", plan.diagnostics);
    assert_eq!(plan.cases[0].expectation.tokens.len(), 1);
    assert_eq!(plan.cases[0].expectation.tokens[0].kind, "identifier");
    assert_eq!(plan.cases[0].expectation.tokens[0].lexeme, "alpha");
    assert_eq!(plan.cases[0].expectation.tokens[0].span_start_line, Some(1));
    assert_eq!(plan.cases[0].expectation.tokens[0].span_start_col, Some(1));
    assert_eq!(plan.cases[0].expectation.tokens[0].span_end_line, Some(1));
    assert_eq!(plan.cases[0].expectation.tokens[0].span_end_col, Some(6));
}

#[test]
fn diagnostic_payload_expectations_parse_and_reject_empty_entries() {
    let corpus = Corpus::new();
    corpus.add_requirement("spec.en.test.basic", &[]);
    corpus.write("tests/lexical/fail/diagnostic_payload.src", "\"alpha\"");
    corpus.write(
        "tests/lexical/fail/diagnostic_payload.expect.toml",
        r#"schema_version = 1
id = "diagnostic_payload"
kind = "fail"
stage = "lexical"
domain = "disambiguator"
source = "diagnostic_payload.src"
expected_outcome = "fail"
expected_phase = "lex"
failure_category = "lexical_diagnostic"
stable_detail_key = "diagnostic_payload"
diagnostic_codes = ["parser_context_rejected_candidate"]
diagnostic_payloads = ["parser_context_rejected_candidate:string_literal"]
spec_refs = ["spec.en.test.basic"]
"#,
    );

    let plan = corpus.plan();

    assert_eq!(plan.error_count(), 0, "{:#?}", plan.diagnostics);
    assert_eq!(
        plan.cases[0].expectation.diagnostic_payloads,
        vec!["parser_context_rejected_candidate:string_literal"]
    );

    let corpus = Corpus::new();
    corpus.add_requirement("spec.en.test.basic", &[]);
    corpus.write(
        "tests/lexical/fail/empty_diagnostic_payload.src",
        "\"alpha\"",
    );
    corpus.write(
        "tests/lexical/fail/empty_diagnostic_payload.expect.toml",
        r#"schema_version = 1
id = "empty_diagnostic_payload"
kind = "fail"
stage = "lexical"
domain = "disambiguator"
source = "empty_diagnostic_payload.src"
expected_outcome = "fail"
expected_phase = "lex"
failure_category = "lexical_diagnostic"
stable_detail_key = "empty_diagnostic_payload"
diagnostic_codes = ["parser_context_rejected_candidate"]
diagnostic_payloads = [""]
spec_refs = ["spec.en.test.basic"]
"#,
    );

    let plan = corpus.plan();

    assert_has_code(&plan, "E-EXPECT-SCHEMA");
}

#[test]
fn token_expectations_reject_unknown_or_empty_fields() {
    let corpus = Corpus::new();
    corpus.write("tests/lexical/pass/bad_token_unknown.src", "alpha");
    corpus.write(
        "tests/lexical/pass/bad_token_unknown.expect.toml",
        r#"schema_version = 1
id = "bad_token_unknown"
kind = "pass"
stage = "lexical"
domain = "lexical"
source = "bad_token_unknown.src"
expected_outcome = "pass"
expected_phase = "lex"
diagnostic_codes = []
spec_refs = ["spec.en.test.basic"]

[[tokens]]
kind = "identifier"
lexeme = "alpha"
span = "0..5"
"#,
    );
    corpus.write("tests/lexical/pass/bad_token_empty.src", "alpha");
    corpus.write(
        "tests/lexical/pass/bad_token_empty.expect.toml",
        r#"schema_version = 1
id = "bad_token_empty"
kind = "pass"
stage = "lexical"
domain = "lexical"
source = "bad_token_empty.src"
expected_outcome = "pass"
expected_phase = "lex"
diagnostic_codes = []
spec_refs = ["spec.en.test.basic"]

[[tokens]]
kind = ""
lexeme = "alpha"
"#,
    );

    let plan = corpus.plan();
    let schema_errors = plan
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code.0 == "E-EXPECT-SCHEMA")
        .count();

    assert_eq!(schema_errors, 2, "{:#?}", plan.diagnostics);
}

#[test]
fn token_expectations_are_lexical_only() {
    let corpus = Corpus::new();
    corpus.write("tests/miz/pass/parser/token_in_parse_stage.miz", "alpha");
    corpus.write(
        "tests/miz/pass/parser/token_in_parse_stage.expect.toml",
        r#"schema_version = 1
id = "token_in_parse_stage"
kind = "pass"
stage = "parse_only"
domain = "parser"
source = "token_in_parse_stage.miz"
expected_outcome = "pass"
expected_phase = "parse"
diagnostic_codes = []
spec_refs = ["spec.en.test.basic"]

[[tokens]]
kind = "identifier"
lexeme = "alpha"
"#,
    );

    let plan = corpus.plan();

    assert_has_code(&plan, "E-EXPECT-SCHEMA");
}

#[test]
fn parse_only_surface_ast_snapshot_path_is_retained() {
    let corpus = Corpus::new();
    corpus.add_requirement("spec.en.test.basic", &[]);
    corpus.write("tests/miz/pass/parser/snapshot_case.miz", "alpha;");
    corpus.write(
        "tests/miz/pass/parser/snapshot_case.expect.toml",
        r#"schema_version = 1
id = "snapshot_case"
kind = "pass"
stage = "parse_only"
domain = "parser"
source = "snapshot_case.miz"
expected_outcome = "pass"
expected_phase = "parse"
diagnostic_codes = []
snapshots = "snapshots/parser/snapshot_case.surface_ast.snap"
tags = ["active_parse_only"]
spec_refs = ["spec.en.test.basic"]
"#,
    );

    let plan = corpus.plan();

    assert_eq!(plan.error_count(), 0, "{:#?}", plan.diagnostics);
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "snapshot_case")
        .expect("snapshot case should be discovered");
    assert_eq!(
        case.expectation.snapshots.as_deref(),
        Some(Path::new("snapshots/parser/snapshot_case.surface_ast.snap"))
    );
}

#[test]
fn parse_only_surface_ast_snapshot_path_is_validated() {
    let corpus = Corpus::new();
    corpus.add_requirement("spec.en.test.basic", &[]);
    corpus.write("tests/miz/pass/parser/snapshot_escape.miz", "alpha;");
    corpus.write(
        "tests/miz/pass/parser/snapshot_escape.expect.toml",
        r#"schema_version = 1
id = "snapshot_escape"
kind = "pass"
stage = "parse_only"
domain = "parser"
source = "snapshot_escape.miz"
expected_outcome = "pass"
expected_phase = "parse"
diagnostic_codes = []
snapshots = "../snapshot_escape.surface_ast.snap"
tags = ["active_parse_only"]
spec_refs = ["spec.en.test.basic"]
"#,
    );
    corpus.write("tests/miz/pass/parser/snapshot_bad_ext.miz", "alpha;");
    corpus.write(
        "tests/miz/pass/parser/snapshot_bad_ext.expect.toml",
        r#"schema_version = 1
id = "snapshot_bad_ext"
kind = "pass"
stage = "parse_only"
domain = "parser"
source = "snapshot_bad_ext.miz"
expected_outcome = "pass"
expected_phase = "parse"
diagnostic_codes = []
snapshots = "snapshots/parser/snapshot_bad_ext.txt"
tags = ["active_parse_only"]
spec_refs = ["spec.en.test.basic"]
"#,
    );
    corpus.write("tests/lexical/pass/snapshot_scope.src", "alpha");
    corpus.write(
        "tests/lexical/pass/snapshot_scope.expect.toml",
        r#"schema_version = 1
id = "snapshot_scope"
kind = "pass"
stage = "lexical"
domain = "lexical"
source = "snapshot_scope.src"
expected_outcome = "pass"
expected_phase = "lex"
diagnostic_codes = []
snapshots = "snapshots/parser/snapshot_scope.surface_ast.snap"
spec_refs = ["spec.en.test.basic"]
"#,
    );

    let plan = corpus.plan();

    assert_has_code(&plan, "E-EXPECT-SNAPSHOT-PATH");
    assert_has_code(&plan, "E-EXPECT-SNAPSHOT-EXTENSION");
    assert_has_code(&plan, "E-EXPECT-SNAPSHOT-SCOPE");
}

#[test]
fn parse_only_runner_reports_surface_ast_snapshot_mismatch() {
    let corpus = Corpus::new();
    corpus.add_requirement("spec.en.test.basic", &[]);
    corpus.write("tests/miz/pass/parser/snapshot_mismatch.miz", "alpha;");
    corpus.write(
        "tests/snapshots/parser/snapshot_mismatch.surface_ast.snap",
        "wrong\n",
    );
    corpus.write(
        "tests/miz/pass/parser/snapshot_mismatch.expect.toml",
        r#"schema_version = 1
id = "snapshot_mismatch"
kind = "pass"
stage = "parse_only"
domain = "parser"
source = "snapshot_mismatch.miz"
expected_outcome = "pass"
expected_phase = "parse"
diagnostic_codes = []
snapshots = "snapshots/parser/snapshot_mismatch.surface_ast.snap"
tags = ["active_parse_only"]
spec_refs = ["spec.en.test.basic"]
"#,
    );

    let report = run_parse_only_corpus(&corpus.config()).unwrap();

    assert_eq!(report.failed_count(), 1);
    assert_has_report_code(&report, "E-PARSE-ONLY-SNAPSHOT");
}

#[test]
fn parse_only_runner_reports_missing_surface_ast_snapshot() {
    let corpus = Corpus::new();
    corpus.add_requirement("spec.en.test.basic", &[]);
    corpus.write("tests/miz/pass/parser/snapshot_missing.miz", "alpha;");
    corpus.write(
        "tests/miz/pass/parser/snapshot_missing.expect.toml",
        r#"schema_version = 1
id = "snapshot_missing"
kind = "pass"
stage = "parse_only"
domain = "parser"
source = "snapshot_missing.miz"
expected_outcome = "pass"
expected_phase = "parse"
diagnostic_codes = []
snapshots = "snapshots/parser/snapshot_missing.surface_ast.snap"
tags = ["active_parse_only"]
spec_refs = ["spec.en.test.basic"]
"#,
    );

    let report = run_parse_only_corpus(&corpus.config()).unwrap();

    assert_eq!(report.failed_count(), 1);
    assert_has_report_code(&report, "E-PARSE-ONLY-SNAPSHOT");
}

#[test]
fn parse_only_runner_reports_snapshot_request_without_ast() {
    let corpus = Corpus::new();
    corpus.add_requirement("spec.en.test.basic", &[]);
    corpus.write("tests/miz/fail/parser/snapshot_no_ast.miz", "end;");
    corpus.write(
        "tests/snapshots/parser/snapshot_no_ast.surface_ast.snap",
        "surface-ast-snapshot-v1\n",
    );
    corpus.write(
        "tests/miz/fail/parser/snapshot_no_ast.expect.toml",
        r#"schema_version = 1
id = "snapshot_no_ast"
kind = "fail"
stage = "parse_only"
domain = "parser"
source = "snapshot_no_ast.miz"
expected_outcome = "fail"
expected_phase = "parse"
failure_category = "syntax_error"
rejection_reason = "stray_end"
stable_detail_key = "parser.snapshot.no_ast"
diagnostic_codes = [
  "unrecoverable_input",
]
snapshots = "snapshots/parser/snapshot_no_ast.surface_ast.snap"
tags = ["active_parse_only", "allow_frontend_recovery_diagnostics"]
spec_refs = ["spec.en.test.basic"]
"#,
    );

    let report = run_parse_only_corpus(&corpus.config()).unwrap();

    assert_eq!(report.failed_count(), 1);
    assert_has_report_code(&report, "E-PARSE-ONLY-SNAPSHOT");
}

struct Corpus {
    root: PathBuf,
}

impl Corpus {
    fn new() -> Self {
        let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
        let root =
            std::env::temp_dir().join(format!("mizar-test-metadata-{}-{id}", std::process::id()));
        if root.exists() {
            fs::remove_dir_all(&root).unwrap();
        }
        fs::create_dir_all(&root).unwrap();
        let corpus = Self { root };
        corpus.create_standard_roots();
        corpus.write("tests/coverage/spec_trace.toml", "");
        corpus
    }

    fn create_standard_roots(&self) {
        for dir in [
            "tests/miz",
            "tests/lexical",
            "tests/certificates",
            "tests/generated",
            "tests/fuzz",
            "tests/property",
            "tests/snapshots",
            "tests/coverage",
            "doc/spec/en",
        ] {
            fs::create_dir_all(self.root.join(dir)).unwrap();
        }
    }

    fn write(&self, path: impl AsRef<Path>, content: impl AsRef<[u8]>) {
        let path = self.root.join(path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, content).unwrap();
    }

    fn add_requirement(&self, id: &str, tests: &[&str]) {
        let tests = tests
            .iter()
            .map(|test| format!("\"{test}\""))
            .collect::<Vec<_>>()
            .join(", ");
        self.write(
            "tests/coverage/spec_trace.toml",
            format!(
                r#"
[[requirement]]
id = "{id}"
source = "doc/spec/en/test.md"
section = "Test"
stage = "lexical"
status = "planned"
required = true
coverage = "pass"
tests = [{tests}]
"#
            ),
        );
        self.write("doc/spec/en/test.md", "# Test\n");
    }

    fn add_case(&self, stem_path: &str, id: &str, spec_ref: &str) {
        self.write(format!("{stem_path}.src"), "");
        let source = Path::new(stem_path)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned()
            + ".src";
        self.write(
            format!("{stem_path}.expect.toml"),
            expectation(id, &source, spec_ref),
        );
    }

    fn plan(&self) -> mizar_test::TestPlan {
        build_test_plan(&self.config()).unwrap()
    }

    fn config(&self) -> DiscoveryConfig {
        DiscoveryConfig {
            workspace_root: self.root.clone(),
            tests_root: self.root.join("tests"),
            manifest_path: self.root.join("tests/coverage/spec_trace.toml"),
            profile: TestProfile::Fast,
            validation_mode: ValidationMode::Metadata,
        }
    }
}

impl Drop for Corpus {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn expectation(id: &str, source: &str, spec_ref: &str) -> String {
    format!(
        r#"schema_version = 1
id = "{id}"
kind = "pass"
stage = "lexical"
domain = "lexical"
source = "{source}"
expected_outcome = "pass"
expected_phase = "lex"
diagnostic_codes = []
spec_refs = ["{spec_ref}"]
"#
    )
}

fn assert_has_code(plan: &mizar_test::TestPlan, code: &str) {
    assert!(
        plan.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code.0 == code),
        "expected diagnostic {code}, got {:#?}",
        plan.diagnostics
    );
}

fn assert_has_report_code(report: &mizar_test::ParseOnlyRunReport, code: &str) {
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code.0 == code),
        "expected diagnostic {code}, got {:#?}",
        report.diagnostics
    );
}

fn repository_plan() -> TestPlan {
    build_test_plan(&repository_config()).unwrap()
}

fn repository_config() -> DiscoveryConfig {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap()
        .to_path_buf();
    DiscoveryConfig {
        workspace_root: workspace_root.clone(),
        tests_root: workspace_root.join("tests"),
        manifest_path: workspace_root.join("tests/coverage/spec_trace.toml"),
        profile: TestProfile::Fast,
        validation_mode: ValidationMode::Metadata,
    }
}

fn reserved_words_from_appendix_a(workspace_root: &Path) -> BTreeSet<String> {
    let path = workspace_root.join("doc/spec/en/appendix_a.grammar_summary.md");
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read `{}`: {error}", path.display()));
    let (_, after_heading) = content
        .split_once("Reserved words are case-sensitive")
        .expect("appendix A should contain the reserved-word contract");
    let (_, after_fence) = after_heading
        .split_once("```text\n")
        .expect("reserved-word contract should use a text fence");
    let (word_block, _) = after_fence
        .split_once("\n```")
        .expect("reserved-word text fence should close");

    word_block.split_whitespace().map(str::to_owned).collect()
}

fn active_parser_corpus_reserved_words(
    plan: &TestPlan,
    reserved_words: &BTreeSet<String>,
) -> BTreeSet<String> {
    let mut covered_words = BTreeSet::new();
    let ids = InMemorySessionIdAllocator::new();
    let snapshot = snapshot_id(43);

    for (ordinal, case) in active_parse_only_cases(plan).enumerate() {
        let source = fs::read_to_string(&case.source_path).unwrap_or_else(|error| {
            panic!(
                "failed to read active parse-only source `{}`: {error}",
                case.source_path.display()
            )
        });
        let source_id = ids
            .next_source_id(snapshot)
            .expect("reserved-word audit source id should allocate");
        let source_unit =
            source_unit_for_reserved_word_audit(&case.source_path, source_id, ordinal, &source);
        covered_words.extend(reserved_words_in_frontend_tokens(
            &source_unit,
            reserved_words,
        ));
    }

    covered_words
}

fn source_unit_for_reserved_word_audit(
    source_path: &Path,
    source_id: mizar_session::SourceId,
    ordinal: usize,
    source: &str,
) -> SourceUnit {
    let audit_root = std::env::temp_dir().join(format!(
        "mizar-test-parser-reserved-word-audit-{}-{ordinal}",
        std::process::id()
    ));
    let audit_path = audit_root
        .join("src")
        .join("parser")
        .join("audit")
        .join(format!("case_{ordinal}.miz"));
    fs::create_dir_all(
        audit_path
            .parent()
            .expect("audit path should have a parent"),
    )
    .expect("reserved-word audit temp directory should be created");
    fs::write(&audit_path, source).expect("reserved-word audit temp source should be written");
    let normalized_path = normalize_path(&audit_root, &audit_path)
        .expect("reserved-word audit temp path should normalize");
    let _ = fs::remove_dir_all(&audit_root);

    SourceUnit {
        source_id,
        package_id: PackageId::new("parser_reserved_word_audit"),
        module_path: ModulePath::new(format!("parser.audit.case_{ordinal}")),
        normalized_path,
        edition: Edition::new("2026"),
        file_path: source_path.to_path_buf(),
        source_text: Arc::from(source),
        source_hash: hash_text(source),
        line_map: LineMap::with_source(source_id, source),
        loading_map: None,
        origin: SourceOrigin::Disk,
        generated_anchor: None,
    }
}

fn reserved_words_in_frontend_tokens(
    source: &SourceUnit,
    reserved_words: &BTreeSet<String>,
) -> BTreeSet<String> {
    let mut bridge = SpanBridge::new();
    register_source_unit(&mut bridge, source)
        .expect("reserved-word audit source registration should succeed");
    let preprocessed =
        preprocess(source, &mut bridge).expect("reserved-word audit preprocessing should succeed");
    let environment = build_active_lexical_environment(
        &LexicalEnvironmentRequest {
            source_id: source.source_id,
            import_stubs: &preprocessed.import_stubs,
            edition: source.edition.clone(),
        },
        &EmptyProvider,
    )
    .expect("reserved-word audit lexical environment should build")
    .environment;
    let plan = ParserLexingPlan::for_lexical_text(preprocessed.lexical_text.as_str());
    let token_stream = tokenize(
        TokenizeRequest::with_plan(&preprocessed, &environment, plan)
            .with_current_module(ModuleId::new("parser.reserved_word_audit")),
        &bridge,
    )
    .expect("reserved-word audit tokenization should succeed");

    token_stream
        .tokens()
        .iter()
        .filter(|token| token.kind == TokenKind::ReservedWord)
        .map(|token| token.text.to_string())
        .filter(|word| reserved_words.contains(word))
        .collect()
}

struct EmptyProvider;

impl LexicalSummaryProvider for EmptyProvider {
    fn resolve_imports(
        &self,
        _request: &LexicalEnvironmentRequest<'_>,
    ) -> Result<ResolvedImports, FrontendLexicalEnvironmentError> {
        Ok(ResolvedImports {
            imports: Vec::new(),
            summaries: Vec::new(),
            diagnostics: Vec::new(),
        })
    }
}

fn snapshot_id(byte: u8) -> BuildSnapshotId {
    let hex = format!("{byte:02x}").repeat(Hash::BYTE_LEN);
    BuildSnapshotId::from_published_schema_str(&format!("mizar-session-build-snapshot-v1:{hex}"))
        .expect("static snapshot id should parse")
}

fn rel(root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(root).unwrap().to_path_buf()
}
