use std::collections::BTreeSet;
use std::fmt::Write as _;
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
    CoverageShape, DiscoveryConfig, ExpectedOutcome, PipelinePhase, RequirementStatus, Stage,
    TestKind, TestPlan, TestProfile, ValidationMode, active_parse_only_cases,
    active_type_elaboration_cases, architecture22_scenario_specs, build_test_plan,
    run_declaration_symbol_corpus, run_parse_only_corpus, run_type_elaboration_corpus,
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
fn unsupported_schema_versions_fail() {
    let corpus = Corpus::new();
    corpus.write("tests/lexical/pass/new_schema.src", "");
    corpus.write(
        "tests/lexical/pass/new_schema.expect.toml",
        r#"schema_version = 2
id = "new_schema"
kind = "pass"
stage = "lexical"
domain = "lexical"
source = "new_schema.src"
expected_outcome = "pass"
expected_phase = "lex"
diagnostic_codes = []
spec_refs = ["spec.en.test.basic"]
"#,
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
fn expectation_stems_must_match_sidecar_and_source() {
    let corpus = Corpus::new();
    corpus.add_requirement("spec.en.test.basic", &[]);
    corpus.write("tests/lexical/pass/nested/other.src", "");
    corpus.write(
        "tests/lexical/pass/nested/other.expect.toml",
        expectation("other", "other.src", "spec.en.test.basic"),
    );
    corpus.write("tests/lexical/pass/actual.src", "");
    corpus.write(
        "tests/lexical/pass/actual.expect.toml",
        r#"schema_version = 1
id = "different"
kind = "pass"
stage = "lexical"
domain = "lexical"
source = "nested/other.src"
expected_outcome = "pass"
expected_phase = "lex"
diagnostic_codes = []
spec_refs = ["spec.en.test.basic"]
"#,
    );

    let plan = corpus.plan();

    assert_has_code(&plan, "E-EXPECT-ID-MISMATCH");
    assert_has_code(&plan, "E-EXPECT-SOURCE-STEM");
}

#[test]
fn invalid_enum_and_kind_outcome_pairs_fail() {
    let corpus = Corpus::new();
    corpus.write("tests/lexical/pass/bad_kind.src", "");
    corpus.write(
        "tests/lexical/pass/bad_kind.expect.toml",
        r#"schema_version = 1
id = "bad_kind"
kind = "surprise"
stage = "lexical"
domain = "lexical"
source = "bad_kind.src"
expected_outcome = "pass"
expected_phase = "lex"
diagnostic_codes = []
spec_refs = ["spec.en.test.basic"]
"#,
    );
    corpus.write("tests/lexical/pass/bad_stage.src", "");
    corpus.write(
        "tests/lexical/pass/bad_stage.expect.toml",
        r#"schema_version = 1
id = "bad_stage"
kind = "pass"
stage = "surprise"
domain = "lexical"
source = "bad_stage.src"
expected_outcome = "pass"
expected_phase = "lex"
diagnostic_codes = []
spec_refs = ["spec.en.test.basic"]
"#,
    );
    corpus.write("tests/lexical/pass/bad_outcome.src", "");
    corpus.write(
        "tests/lexical/pass/bad_outcome.expect.toml",
        r#"schema_version = 1
id = "bad_outcome"
kind = "pass"
stage = "lexical"
domain = "lexical"
source = "bad_outcome.src"
expected_outcome = "surprise"
expected_phase = "lex"
diagnostic_codes = []
spec_refs = ["spec.en.test.basic"]
"#,
    );
    corpus.write("tests/lexical/pass/bad_pair.src", "");
    corpus.write(
        "tests/lexical/pass/bad_pair.expect.toml",
        r#"schema_version = 1
id = "bad_pair"
kind = "pass"
stage = "lexical"
domain = "lexical"
source = "bad_pair.src"
expected_outcome = "fail"
expected_phase = "lex"
diagnostic_codes = []
spec_refs = ["spec.en.test.basic"]
"#,
    );

    let plan = corpus.plan();
    let schema_errors = plan
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code.0 == "E-EXPECT-SCHEMA")
        .count();

    assert_eq!(schema_errors, 4, "{:#?}", plan.diagnostics);
}

#[test]
fn duplicate_sidecar_spec_refs_fail() {
    let corpus = Corpus::new();
    corpus.add_requirement("spec.en.test.basic", &[]);
    corpus.write("tests/lexical/pass/duplicate_spec_ref.src", "");
    corpus.write(
        "tests/lexical/pass/duplicate_spec_ref.expect.toml",
        r#"schema_version = 1
id = "duplicate_spec_ref"
kind = "pass"
stage = "lexical"
domain = "lexical"
source = "duplicate_spec_ref.src"
expected_outcome = "pass"
expected_phase = "lex"
diagnostic_codes = []
spec_refs = ["spec.en.test.basic", "spec.en.test.basic"]
"#,
    );

    let plan = corpus.plan();

    assert_has_code(&plan, "E-EXPECT-DUP-SPEC-REF");
}

#[test]
fn optional_metadata_is_retained_and_profiles_filter_cases() {
    let corpus = Corpus::new();
    corpus.add_requirement("spec.en.test.basic", &[]);
    corpus.write("tests/lexical/pass/fast_case.src", "");
    corpus.write(
        "tests/lexical/pass/fast_case.expect.toml",
        r#"schema_version = 1
id = "fast_case"
kind = "pass"
stage = "lexical"
domain = "lexical"
source = "fast_case.src"
expected_outcome = "pass"
expected_phase = "lex"
diagnostic_codes = []
spec_refs = ["spec.en.test.basic"]
notes = "retained review note"
ast_profile = "surface"
snapshot_profiles = ["surface_ast"]
"#,
    );
    corpus.write("tests/lexical/pass/stress_case.src", "");
    corpus.write(
        "tests/lexical/pass/stress_case.expect.toml",
        r#"schema_version = 1
id = "stress_case"
kind = "pass"
stage = "lexical"
domain = "lexical"
source = "stress_case.src"
expected_outcome = "pass"
expected_phase = "lex"
diagnostic_codes = []
spec_refs = ["spec.en.test.basic"]
profiles = ["stress"]
"#,
    );

    let fast_plan = corpus.plan();

    assert_eq!(fast_plan.error_count(), 0, "{:#?}", fast_plan.diagnostics);
    assert_eq!(fast_plan.cases.len(), 1);
    let fast_case = &fast_plan.cases[0].expectation;
    assert_eq!(fast_case.profiles, vec!["fast".to_owned()]);
    assert_eq!(fast_case.notes.as_deref(), Some("retained review note"));
    assert_eq!(fast_case.ast_profile.as_deref(), Some("surface"));
    assert_eq!(fast_case.snapshot_profiles, vec!["surface_ast".to_owned()]);

    let mut stress_config = corpus.config();
    stress_config.profile = TestProfile::Stress;
    let stress_plan = build_test_plan(&stress_config).unwrap();

    assert_eq!(
        stress_plan.error_count(),
        0,
        "{:#?}",
        stress_plan.diagnostics
    );
    assert_eq!(stress_plan.cases.len(), 1);
    assert_eq!(stress_plan.cases[0].id.0, "stress_case");

    let mut full_config = corpus.config();
    full_config.profile = TestProfile::Full;
    let full_plan = build_test_plan(&full_config).unwrap();

    assert_eq!(full_plan.error_count(), 0, "{:#?}", full_plan.diagnostics);
    let full_ids = full_plan
        .cases
        .iter()
        .map(|case| case.id.0.as_str())
        .collect::<Vec<_>>();
    assert_eq!(full_ids, vec!["fast_case", "stress_case"]);
}

#[test]
fn invalid_profiles_metadata_fails() {
    let corpus = Corpus::new();
    corpus.write("tests/lexical/pass/empty_profiles.src", "");
    corpus.write(
        "tests/lexical/pass/empty_profiles.expect.toml",
        r#"schema_version = 1
id = "empty_profiles"
kind = "pass"
stage = "lexical"
domain = "lexical"
source = "empty_profiles.src"
expected_outcome = "pass"
expected_phase = "lex"
diagnostic_codes = []
spec_refs = ["spec.en.test.basic"]
profiles = []
"#,
    );
    corpus.write("tests/lexical/pass/unknown_profile.src", "");
    corpus.write(
        "tests/lexical/pass/unknown_profile.expect.toml",
        r#"schema_version = 1
id = "unknown_profile"
kind = "pass"
stage = "lexical"
domain = "lexical"
source = "unknown_profile.src"
expected_outcome = "pass"
expected_phase = "lex"
diagnostic_codes = []
spec_refs = ["spec.en.test.basic"]
profiles = ["fasst"]
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
fn corpus_origin_metadata_is_retained_for_generated_fuzz_and_property_cases() {
    let corpus = Corpus::new();
    corpus.add_requirement("spec.en.test.basic", &[]);
    corpus.write(
        "tests/generated/generated_parser_001.miz",
        "reserve x for set;\nreserve y for set;\ntheorem x = x;\nproof\nend;\n",
    );
    corpus.write(
        "tests/generated/generated_parser_001.expect.toml",
        generated_expectation(
            "generated_parser_001",
            "generated_parser_001.miz",
            "spec.en.test.basic",
            r#"profiles = ["full"]"#,
            false,
        ),
    );
    corpus.write("tests/fuzz/fuzz_lexer_001.fixture.toml", "seed = \"abc\"\n");
    corpus.write(
        "tests/fuzz/fuzz_lexer_001.expect.toml",
        fuzz_seed_expectation(
            "fuzz_lexer_001",
            "fuzz_lexer_001.fixture.toml",
            "spec.en.test.basic",
            r#"profiles = ["fuzz_regression"]"#,
            Some("lex_error"),
        ),
    );
    corpus.write(
        "tests/property/property_lexer_001.fixture.toml",
        "seed = \"def\"\n",
    );
    corpus.write(
        "tests/property/property_lexer_001.expect.toml",
        property_seed_expectation(
            "property_lexer_001",
            "property_lexer_001.fixture.toml",
            "spec.en.test.basic",
            r#"profiles = ["full"]"#,
            false,
        ),
    );

    let mut config = corpus.config();
    config.profile = TestProfile::Full;
    let plan = build_test_plan(&config).unwrap();

    assert_eq!(plan.error_count(), 0, "{:#?}", plan.diagnostics);
    let generated = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "generated_parser_001")
        .unwrap();
    let generated_origin = generated.expectation.origin.as_ref().unwrap();
    assert_eq!(generated_origin.kind, TestKind::Generated);
    assert_eq!(generated_origin.generator, "grammar-smoke");
    assert_eq!(generated_origin.generator_version, "0.1.0");
    assert_eq!(generated_origin.seed, "generated_parser_001");
    assert_eq!(generated_origin.profile, "parser");
    assert_eq!(generated_origin.expected_outcome, ExpectedOutcome::Pass);
    assert!(!generated_origin.minimized);

    let fuzz = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "fuzz_lexer_001")
        .unwrap();
    let fuzz_origin = fuzz.expectation.origin.as_ref().unwrap();
    assert_eq!(fuzz_origin.kind, TestKind::FuzzSeed);
    assert_eq!(fuzz_origin.generator, "cargo-fuzz");
    assert_eq!(fuzz_origin.generator_version, "0.1.0");
    assert_eq!(fuzz_origin.seed, "fuzz_lexer_001");
    assert_eq!(fuzz_origin.profile, "lexical");
    assert_eq!(fuzz_origin.expected_outcome, ExpectedOutcome::MetadataOnly);
    assert_eq!(
        fuzz_origin.original_failure_category.as_deref(),
        Some("lex_error")
    );

    let property = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "property_lexer_001")
        .unwrap();
    let property_origin = property.expectation.origin.as_ref().unwrap();
    assert_eq!(property_origin.kind, TestKind::PropertySeed);
    assert_eq!(property_origin.generator, "proptest");
    assert_eq!(property_origin.generator_version, "0.1.0");
    assert_eq!(property_origin.seed, "property_lexer_001");
    assert_eq!(property_origin.profile, "lexical");
    assert_eq!(
        property_origin.expected_outcome,
        ExpectedOutcome::MetadataOnly
    );
}

#[test]
fn corpus_policy_violations_are_reported() {
    let corpus = Corpus::new();
    corpus.add_requirement("spec.en.test.basic", &[]);
    corpus.write("tests/generated/missing_origin.miz", "theorem x = x;\n");
    corpus.write(
        "tests/generated/missing_origin.expect.toml",
        r#"schema_version = 1
id = "missing_origin"
kind = "generated"
stage = "parse_only"
domain = "parser"
source = "missing_origin.miz"
expected_outcome = "pass"
expected_phase = "parse"
diagnostic_codes = []
spec_refs = ["spec.en.test.basic"]
profiles = ["full"]
"#,
    );
    corpus.write(
        "tests/fuzz/misplaced_property.fixture.toml",
        "seed = \"abc\"\n",
    );
    corpus.write(
        "tests/fuzz/misplaced_property.expect.toml",
        property_seed_expectation(
            "misplaced_property",
            "misplaced_property.fixture.toml",
            "spec.en.test.basic",
            r#"profiles = ["full"]"#,
            false,
        ),
    );
    corpus.write("tests/generated/unminimized_fast.miz", five_line_miz());
    corpus.write(
        "tests/generated/unminimized_fast.expect.toml",
        generated_expectation(
            "unminimized_fast",
            "unminimized_fast.miz",
            "spec.en.test.basic",
            "",
            false,
        ),
    );
    corpus.write("tests/generated/stress_fast_profile.miz", five_line_miz());
    corpus.write(
        "tests/generated/stress_fast_profile.expect.toml",
        generated_expectation(
            "stress_fast_profile",
            "stress_fast_profile.miz",
            "spec.en.test.basic",
            r#"profiles = ["fast", "stress"]"#,
            true,
        ),
    );
    corpus.write(
        "tests/fuzz/fuzz_missing_category.fixture.toml",
        "seed = \"abc\"\n",
    );
    corpus.write(
        "tests/fuzz/fuzz_missing_category.expect.toml",
        fuzz_seed_expectation(
            "fuzz_missing_category",
            "fuzz_missing_category.fixture.toml",
            "spec.en.test.basic",
            r#"profiles = ["full"]"#,
            None,
        ),
    );
    corpus.write("tests/stress/stress_without_profile.miz", five_line_miz());
    corpus.write(
        "tests/stress/stress_without_profile.expect.toml",
        generated_expectation(
            "stress_without_profile",
            "stress_without_profile.miz",
            "spec.en.test.basic",
            r#"profiles = ["full"]"#,
            false,
        ),
    );
    corpus.write(
        "tests/generated/generated_too_large.miz",
        "theorem x = x;\n".repeat(31),
    );
    corpus.write(
        "tests/generated/generated_too_large.expect.toml",
        generated_expectation(
            "generated_too_large",
            "generated_too_large.miz",
            "spec.en.test.basic",
            r#"profiles = ["full"]"#,
            false,
        ),
    );
    corpus.write("tests/miz/pass/parser/short_name.src", "");
    corpus.write(
        "tests/miz/pass/parser/short_name.expect.toml",
        parse_pass_expectation("short_name", "short_name.src", "spec.en.test.basic"),
    );

    let plan = corpus.plan();

    assert_has_code(&plan, "E-EXPECT-SCHEMA");
    assert_has_code(&plan, "E-CORPUS-PLACEMENT");
    assert_has_code(&plan, "E-CORPUS-UNMINIMIZED-FAST");
    assert_has_code(&plan, "E-CORPUS-STRESS-FAST-PROFILE");
    assert_has_code(&plan, "E-CORPUS-FUZZ-PROFILE");
    assert_has_code(&plan, "E-CORPUS-FUZZ-CATEGORY");
    assert_has_code(&plan, "E-CORPUS-STRESS-PROFILE");
    assert_has_code(&plan, "E-CORPUS-GENERATED-SIZE");
    assert_has_code(&plan, "W-CORPUS-NAMING");
}

#[test]
fn origin_metadata_schema_errors_fail() {
    let corpus = Corpus::new();
    corpus.add_requirement("spec.en.test.basic", &[]);
    corpus.write("tests/generated/origin_bad_schema_001.miz", five_line_miz());
    corpus.write(
        "tests/generated/origin_bad_schema_001.expect.toml",
        r#"schema_version = 1
id = "origin_bad_schema_001"
kind = "generated"
stage = "parse_only"
domain = "parser"
source = "origin_bad_schema_001.miz"
expected_outcome = "pass"
expected_phase = "parse"
diagnostic_codes = []
spec_refs = ["spec.en.test.basic"]
profiles = ["full"]

[origin]
schema_version = 2
kind = "generated"
generator = "grammar-smoke"
generator_version = "0.1.0"
seed = "origin_bad_schema_001"
profile = "parser"
expected_outcome = "pass"
minimized = false
"#,
    );
    corpus.write(
        "tests/generated/origin_unknown_field_001.miz",
        five_line_miz(),
    );
    corpus.write(
        "tests/generated/origin_unknown_field_001.expect.toml",
        r#"schema_version = 1
id = "origin_unknown_field_001"
kind = "generated"
stage = "parse_only"
domain = "parser"
source = "origin_unknown_field_001.miz"
expected_outcome = "pass"
expected_phase = "parse"
diagnostic_codes = []
spec_refs = ["spec.en.test.basic"]
profiles = ["full"]

[origin]
schema_version = 1
kind = "generated"
generator = "grammar-smoke"
generator_version = "0.1.0"
seed = "origin_unknown_field_001"
profile = "parser"
expected_outcome = "pass"
minimized = false
surprise = "nope"
"#,
    );
    corpus.write("tests/generated/origin_wrong_kind_001.miz", five_line_miz());
    corpus.write(
        "tests/generated/origin_wrong_kind_001.expect.toml",
        r#"schema_version = 1
id = "origin_wrong_kind_001"
kind = "generated"
stage = "parse_only"
domain = "parser"
source = "origin_wrong_kind_001.miz"
expected_outcome = "pass"
expected_phase = "parse"
diagnostic_codes = []
spec_refs = ["spec.en.test.basic"]
profiles = ["full"]

[origin]
schema_version = 1
kind = "property_seed"
generator = "grammar-smoke"
generator_version = "0.1.0"
seed = "origin_wrong_kind_001"
profile = "parser"
expected_outcome = "pass"
minimized = false
"#,
    );
    corpus.write(
        "tests/generated/origin_wrong_outcome_001.miz",
        five_line_miz(),
    );
    corpus.write(
        "tests/generated/origin_wrong_outcome_001.expect.toml",
        r#"schema_version = 1
id = "origin_wrong_outcome_001"
kind = "generated"
stage = "parse_only"
domain = "parser"
source = "origin_wrong_outcome_001.miz"
expected_outcome = "pass"
expected_phase = "parse"
diagnostic_codes = []
spec_refs = ["spec.en.test.basic"]
profiles = ["full"]

[origin]
schema_version = 1
kind = "generated"
generator = "grammar-smoke"
generator_version = "0.1.0"
seed = "origin_wrong_outcome_001"
profile = "parser"
expected_outcome = "fail"
minimized = false
"#,
    );
    corpus.write(
        "tests/generated/origin_missing_seed_001.miz",
        five_line_miz(),
    );
    corpus.write(
        "tests/generated/origin_missing_seed_001.expect.toml",
        r#"schema_version = 1
id = "origin_missing_seed_001"
kind = "generated"
stage = "parse_only"
domain = "parser"
source = "origin_missing_seed_001.miz"
expected_outcome = "pass"
expected_phase = "parse"
diagnostic_codes = []
spec_refs = ["spec.en.test.basic"]
profiles = ["full"]

[origin]
schema_version = 1
kind = "generated"
generator = "grammar-smoke"
generator_version = "0.1.0"
profile = "parser"
expected_outcome = "pass"
minimized = false
"#,
    );
    corpus.write(
        "tests/generated/origin_empty_generator_001.miz",
        five_line_miz(),
    );
    corpus.write(
        "tests/generated/origin_empty_generator_001.expect.toml",
        r#"schema_version = 1
id = "origin_empty_generator_001"
kind = "generated"
stage = "parse_only"
domain = "parser"
source = "origin_empty_generator_001.miz"
expected_outcome = "pass"
expected_phase = "parse"
diagnostic_codes = []
spec_refs = ["spec.en.test.basic"]
profiles = ["full"]

[origin]
schema_version = 1
kind = "generated"
generator = ""
generator_version = "0.1.0"
seed = "origin_empty_generator_001"
profile = "parser"
expected_outcome = "pass"
minimized = false
"#,
    );

    let plan = corpus.plan();
    let schema_errors = plan
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code.0 == "E-EXPECT-SCHEMA")
        .count();

    assert_eq!(schema_errors, 6, "{:#?}", plan.diagnostics);
}

#[test]
fn stress_generated_cases_are_valid_outside_fast_profile() {
    let corpus = Corpus::new();
    corpus.add_requirement("spec.en.test.basic", &[]);
    corpus.write(
        "tests/stress/generated_parser_stress_001.miz",
        "theorem x = x;\n".repeat(40),
    );
    corpus.write(
        "tests/stress/generated_parser_stress_001.expect.toml",
        generated_expectation(
            "generated_parser_stress_001",
            "generated_parser_stress_001.miz",
            "spec.en.test.basic",
            r#"profiles = ["stress"]"#,
            false,
        ),
    );

    let fast_plan = corpus.plan();

    assert_eq!(fast_plan.error_count(), 0, "{:#?}", fast_plan.diagnostics);
    assert_eq!(fast_plan.cases.len(), 0);

    let mut stress_config = corpus.config();
    stress_config.profile = TestProfile::Stress;
    let stress_plan = build_test_plan(&stress_config).unwrap();

    assert_eq!(
        stress_plan.error_count(),
        0,
        "{:#?}",
        stress_plan.diagnostics
    );
    assert_eq!(stress_plan.cases.len(), 1);
    assert_eq!(stress_plan.cases[0].id.0, "generated_parser_stress_001");
}

#[test]
fn generated_stress_profile_does_not_bypass_stress_root_size_gate() {
    let corpus = Corpus::new();
    corpus.add_requirement("spec.en.test.basic", &[]);
    corpus.write(
        "tests/generated/generated_profile_stress_large_001.miz",
        "theorem x = x;\n".repeat(40),
    );
    corpus.write(
        "tests/generated/generated_profile_stress_large_001.expect.toml",
        generated_expectation(
            "generated_profile_stress_large_001",
            "generated_profile_stress_large_001.miz",
            "spec.en.test.basic",
            r#"profiles = ["stress"]"#,
            true,
        ),
    );

    let plan = corpus.plan();

    assert_has_code(&plan, "E-CORPUS-GENERATED-SIZE");
}

#[test]
fn promoted_fuzz_failure_category_must_match_origin() {
    let corpus = Corpus::new();
    corpus.add_requirement("spec.en.test.basic", &[]);
    corpus.write(
        "tests/fuzz/fail_fuzz_lexer_mismatch_001.fixture.toml",
        "seed = \"abc\"\n",
    );
    corpus.write(
        "tests/fuzz/fail_fuzz_lexer_mismatch_001.expect.toml",
        r#"schema_version = 1
id = "fail_fuzz_lexer_mismatch_001"
kind = "fuzz_seed"
stage = "lexical"
domain = "lexical"
source = "fail_fuzz_lexer_mismatch_001.fixture.toml"
expected_outcome = "fail"
expected_phase = "lex"
failure_category = "lex_error"
stable_detail_key = "lexical.fuzz_mismatch"
diagnostic_codes = []
spec_refs = ["spec.en.test.basic"]
profiles = ["fuzz_regression"]

[origin]
schema_version = 1
kind = "fuzz_seed"
generator = "cargo-fuzz"
generator_version = "0.1.0"
seed = "fail_fuzz_lexer_mismatch_001"
profile = "lexical"
expected_outcome = "fail"
minimized = true
original_failure_category = "different_lex_error"
"#,
    );

    let plan = corpus.plan();

    assert_has_code(&plan, "E-CORPUS-FUZZ-CATEGORY");
}

#[test]
fn corpus_naming_rules_cover_snake_case_prefix_and_suffix() {
    let corpus = Corpus::new();
    corpus.add_requirement("spec.en.test.basic", &[]);
    corpus.write("tests/lexical/pass/PassCase001.src", "alpha");
    corpus.write(
        "tests/lexical/pass/PassCase001.expect.toml",
        r#"schema_version = 1
id = "PassCase001"
kind = "pass"
stage = "lexical"
domain = "lexical"
source = "PassCase001.src"
expected_outcome = "pass"
expected_phase = "lex"
diagnostic_codes = []
spec_refs = ["spec.en.test.basic"]
"#,
    );
    corpus.write("tests/lexical/pass/missing_prefix_001.src", "alpha");
    corpus.write(
        "tests/lexical/pass/missing_prefix_001.expect.toml",
        pass_expectation(
            "missing_prefix_001",
            "missing_prefix_001.src",
            "spec.en.test.basic",
        ),
    );
    corpus.write("tests/lexical/pass/pass_missing_suffix.src", "alpha");
    corpus.write(
        "tests/lexical/pass/pass_missing_suffix.expect.toml",
        pass_expectation(
            "pass_missing_suffix",
            "pass_missing_suffix.src",
            "spec.en.test.basic",
        ),
    );

    let plan = corpus.plan();

    assert_eq!(plan.error_count(), 0, "{:#?}", plan.diagnostics);
    let naming_warnings = plan
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code.0 == "W-CORPUS-NAMING")
        .count();
    assert_eq!(naming_warnings, 5, "{:#?}", plan.diagnostics);
    assert!(
        plan.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("stable snake_case stems"))
    );
    assert!(
        plan.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("`pass_` name prefix"))
    );
    assert!(
        plan.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("stable numeric suffix"))
    );
}

#[test]
fn corpus_size_guidelines_warn_for_oversized_handwritten_miz_without_failing() {
    let corpus = Corpus::new();
    corpus.add_requirement("spec.en.test.basic", &[]);
    corpus.write(
        "tests/miz/pass/parser/pass_large_parser_001.miz",
        "theorem x = x;\n".repeat(31),
    );
    corpus.write(
        "tests/miz/pass/parser/pass_large_parser_001.expect.toml",
        parse_pass_expectation(
            "pass_large_parser_001",
            "pass_large_parser_001.miz",
            "spec.en.test.basic",
        ),
    );

    let plan = corpus.plan();

    assert_eq!(plan.error_count(), 0, "{:#?}", plan.diagnostics);
    assert_has_code(&plan, "W-CORPUS-SIZE");
}

#[test]
fn corpus_pass_fail_directory_mismatches_are_errors() {
    let corpus = Corpus::new();
    corpus.add_requirement("spec.en.test.basic", &[]);
    corpus.write(
        "tests/miz/pass/parser/pass_dir_fail_case_001.miz",
        five_line_miz(),
    );
    corpus.write(
        "tests/miz/pass/parser/pass_dir_fail_case_001.expect.toml",
        r#"schema_version = 1
id = "pass_dir_fail_case_001"
kind = "fail"
stage = "parse_only"
domain = "parser"
source = "pass_dir_fail_case_001.miz"
expected_outcome = "fail"
expected_phase = "parse"
failure_category = "syntax_error"
stable_detail_key = "parser.synthetic"
diagnostic_codes = []
spec_refs = ["spec.en.test.basic"]
"#,
    );
    corpus.write(
        "tests/miz/fail/parser/fail_dir_pass_case_001.miz",
        five_line_miz(),
    );
    corpus.write(
        "tests/miz/fail/parser/fail_dir_pass_case_001.expect.toml",
        parse_pass_expectation(
            "fail_dir_pass_case_001",
            "fail_dir_pass_case_001.miz",
            "spec.en.test.basic",
        ),
    );

    let plan = corpus.plan();

    assert_has_code(&plan, "E-CORPUS-OUTCOME-PLACEMENT");
}

#[test]
fn corpus_policy_uses_configured_tests_root_for_path_classes() {
    let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
    let root = std::env::temp_dir().join("tests").join(format!(
        "pass-parent-mizar-test-{}-{id}",
        std::process::id()
    ));
    if root.exists() {
        fs::remove_dir_all(&root).unwrap();
    }
    fs::create_dir_all(&root).unwrap();
    let corpus = Corpus { root };
    corpus.create_standard_roots();
    corpus.write("tests/coverage/spec_trace.toml", "");
    corpus.add_requirement("spec.en.test.basic", &[]);
    corpus.write(
        "tests/generated/generated_root_anchor_001.miz",
        five_line_miz(),
    );
    corpus.write(
        "tests/generated/generated_root_anchor_001.expect.toml",
        generated_expectation(
            "generated_root_anchor_001",
            "generated_root_anchor_001.miz",
            "spec.en.test.basic",
            r#"profiles = ["full"]"#,
            false,
        ),
    );
    corpus.write("tests/lexical/pass/pass_root_anchor_001.src", "alpha");
    corpus.write(
        "tests/lexical/pass/pass_root_anchor_001.expect.toml",
        pass_expectation(
            "pass_root_anchor_001",
            "pass_root_anchor_001.src",
            "spec.en.test.basic",
        ),
    );

    let mut config = corpus.config();
    config.profile = TestProfile::Full;
    let plan = build_test_plan(&config).unwrap();

    assert_eq!(plan.error_count(), 0, "{:#?}", plan.diagnostics);
    assert_lacks_code(&plan, "E-CORPUS-PLACEMENT");
    assert_lacks_code(&plan, "E-CORPUS-OUTCOME-PLACEMENT");
    assert_lacks_code(&plan, "W-CORPUS-NAMING");
}

#[test]
fn filtered_sidecars_still_satisfy_manifest_backrefs() {
    let corpus = Corpus::new();
    corpus.write("tests/lexical/pass/stress_linked.src", "");
    corpus.write(
        "tests/lexical/pass/stress_linked.expect.toml",
        r#"schema_version = 1
id = "stress_linked"
kind = "pass"
stage = "lexical"
domain = "lexical"
source = "stress_linked.src"
expected_outcome = "pass"
expected_phase = "lex"
diagnostic_codes = []
spec_refs = ["spec.en.test.basic"]
profiles = ["stress"]
"#,
    );
    corpus.add_requirement(
        "spec.en.test.basic",
        &["tests/lexical/pass/stress_linked.expect.toml"],
    );

    let fast_plan = corpus.plan();

    assert_eq!(fast_plan.cases.len(), 0);
    assert_lacks_code(&fast_plan, "E-TRACE-UNPARSED-TEST");
    assert_lacks_code(&fast_plan, "E-TRACE-MISSING-BACKREF");
}

#[test]
fn profile_filtering_does_not_hide_sidecar_errors() {
    let corpus = Corpus::new();
    corpus.write("tests/lexical/pass/stress_unknown.src", "");
    corpus.write(
        "tests/lexical/pass/stress_unknown.expect.toml",
        r#"schema_version = 1
id = "stress_unknown"
kind = "pass"
stage = "lexical"
domain = "lexical"
source = "stress_unknown.src"
expected_outcome = "pass"
expected_phase = "lex"
diagnostic_codes = []
spec_refs = ["spec.en.test.unknown"]
profiles = ["stress"]
"#,
    );

    let fast_plan = corpus.plan();

    assert_eq!(fast_plan.cases.len(), 0);
    assert_has_code(&fast_plan, "E-TRACE-UNKNOWN-SPEC-REF");
}

#[test]
fn unknown_roots_are_permissive_in_metadata_and_strict_in_development() {
    let corpus = Corpus::new();
    corpus.write("tests/experimental/README.md", "not a corpus root\n");

    let metadata_plan = corpus.plan();

    assert_lacks_code(&metadata_plan, "E-LAYOUT-UNKNOWN-ROOT");

    let mut development_config = corpus.config();
    development_config.validation_mode = ValidationMode::Development;
    let development_plan = build_test_plan(&development_config).unwrap();

    assert_has_code(&development_plan, "E-LAYOUT-UNKNOWN-ROOT");

    let mut release_config = corpus.config();
    release_config.validation_mode = ValidationMode::Release;
    let release_plan = build_test_plan(&release_config).unwrap();

    assert_has_code(&release_plan, "E-LAYOUT-UNKNOWN-ROOT");
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
fn fail_contract_requires_certificate_and_kernel_rejection_reason() {
    let corpus = Corpus::new();
    corpus.write("tests/certificates/fail/phase_certificate.cert.json", "{}");
    corpus.write(
        "tests/certificates/fail/phase_certificate.expect.toml",
        r#"schema_version = 1
id = "phase_certificate"
kind = "fail"
stage = "advanced_semantics"
domain = "certificate"
source = "phase_certificate.cert.json"
expected_outcome = "fail"
expected_phase = "certificate_check"
failure_category = "proof_failure"
stable_detail_key = "certificate.phase_certificate"
diagnostic_codes = ["E-KERNEL-TEST"]
spec_refs = ["spec.en.test.basic"]
"#,
    );
    corpus.write(
        "tests/certificates/fail/category_certificate.cert.json",
        "{}",
    );
    corpus.write(
        "tests/certificates/fail/category_certificate.expect.toml",
        r#"schema_version = 1
id = "category_certificate"
kind = "fail"
stage = "advanced_semantics"
domain = "certificate"
source = "category_certificate.cert.json"
expected_outcome = "fail"
expected_phase = "verification"
failure_category = "certificate_rejection"
stable_detail_key = "certificate.category_certificate"
diagnostic_codes = ["E-CERT-TEST"]
spec_refs = ["spec.en.test.basic"]
"#,
    );
    corpus.write("tests/certificates/fail/phase_kernel.cert.json", "{}");
    corpus.write(
        "tests/certificates/fail/phase_kernel.expect.toml",
        r#"schema_version = 1
id = "phase_kernel"
kind = "fail"
stage = "advanced_semantics"
domain = "certificate"
source = "phase_kernel.cert.json"
expected_outcome = "fail"
expected_phase = "kernel_check"
failure_category = "proof_failure"
stable_detail_key = "certificate.phase_kernel"
diagnostic_codes = ["E-KERNEL-TEST"]
spec_refs = ["spec.en.test.basic"]
"#,
    );
    corpus.write("tests/certificates/fail/category_kernel.cert.json", "{}");
    corpus.write(
        "tests/certificates/fail/category_kernel.expect.toml",
        r#"schema_version = 1
id = "category_kernel"
kind = "fail"
stage = "advanced_semantics"
domain = "certificate"
source = "category_kernel.cert.json"
expected_outcome = "fail"
expected_phase = "verification"
failure_category = "kernel_rejection"
stable_detail_key = "certificate.category_kernel"
diagnostic_codes = ["E-KERNEL-TEST"]
spec_refs = ["spec.en.test.basic"]
"#,
    );
    corpus.write("tests/certificates/fail/proof_failure.cert.json", "{}");
    corpus.write(
        "tests/certificates/fail/proof_failure.expect.toml",
        r#"schema_version = 1
id = "proof_failure"
kind = "fail"
stage = "advanced_semantics"
domain = "certificate"
source = "proof_failure.cert.json"
expected_outcome = "fail"
expected_phase = "verification"
failure_category = "proof_failure"
stable_detail_key = "certificate.proof_failure"
diagnostic_codes = ["E-PROOF-TEST"]
spec_refs = ["spec.en.test.basic"]
"#,
    );

    let plan = corpus.plan();

    assert_has_code(&plan, "E-EXPECT-REJECTION-REASON");
    assert_eq!(
        plan.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.code.0 == "E-EXPECT-REJECTION-REASON")
            .count(),
        4,
        "{:#?}",
        plan.diagnostics
    );
}

#[test]
fn required_soundness_cases_validate_shape_and_profile() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/miz/fail/parser/weak_false_arithmetic.miz",
        "theorem 1 = 0;",
    );
    corpus.write(
        "tests/miz/fail/parser/weak_false_arithmetic.expect.toml",
        r#"schema_version = 1
id = "weak_false_arithmetic"
kind = "fail"
stage = "parse_only"
domain = "false_arithmetic"
source = "weak_false_arithmetic.miz"
expected_outcome = "fail"
expected_phase = "parse"
failure_category = "syntax_error"
rejection_reason = "syntax"
stable_detail_key = "soundness.false_arithmetic.one_eq_zero"
diagnostic_codes = ["missing_end"]
profiles = ["stress"]
spec_refs = ["spec.en.test.basic"]
"#,
    );

    let plan = corpus.plan();

    assert_has_code(&plan, "E-EXPECT-SOUNDNESS-DOMAIN");
    assert_has_code(&plan, "E-EXPECT-SOUNDNESS-STAGE");
    assert_has_code(&plan, "E-EXPECT-SOUNDNESS-PHASE");
    assert_has_code(&plan, "E-EXPECT-SOUNDNESS-PROFILE");
}

#[test]
fn required_soundness_cases_reject_unknown_keys_and_wrong_outcomes() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/miz/fail/proof/unknown_soundness.miz",
        "theorem 1 = 0;",
    );
    corpus.write(
        "tests/miz/fail/proof/unknown_soundness.expect.toml",
        r#"schema_version = 1
id = "unknown_soundness"
kind = "fail"
stage = "proof_verification"
domain = "soundness"
source = "unknown_soundness.miz"
expected_outcome = "fail"
expected_phase = "verification"
failure_category = "proof_failure"
stable_detail_key = "soundness.typo.case"
diagnostic_codes = ["E-PROOF-TEST"]
spec_refs = ["spec.en.test.basic"]
"#,
    );
    corpus.write(
        "tests/miz/pass/proof/pass_false_arithmetic.miz",
        "theorem 1 = 0;",
    );
    corpus.write(
        "tests/miz/pass/proof/pass_false_arithmetic.expect.toml",
        r#"schema_version = 1
id = "pass_false_arithmetic"
kind = "pass"
stage = "proof_verification"
domain = "soundness"
source = "pass_false_arithmetic.miz"
expected_outcome = "pass"
expected_phase = "verification"
stable_detail_key = "soundness.false_arithmetic.one_eq_zero"
diagnostic_codes = []
spec_refs = ["spec.en.test.basic"]
"#,
    );

    let plan = corpus.plan();

    assert_has_code(&plan, "E-EXPECT-SOUNDNESS-CASE");
    assert_has_code(&plan, "E-EXPECT-SOUNDNESS-OUTCOME");
}

#[test]
fn required_soundness_missing_cases_are_mode_aware() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.soundness.contract"
source = "doc/design/mizar-test/en/fail_soundness.md"
section = "Required Soundness Cases"
stage = "advanced_semantics"
status = "planned"
required = true
coverage = "fail"
tests = []
"#,
    );
    corpus.write(
        "doc/design/mizar-test/en/fail_soundness.md",
        "# Fail Soundness\n",
    );
    let metadata_plan = corpus.plan();

    assert_has_code(&metadata_plan, "W-SOUNDNESS-MISSING-CASE");

    let mut development_config = corpus.config();
    development_config.validation_mode = ValidationMode::Development;
    let development_plan = build_test_plan(&development_config).unwrap();

    assert_has_code(&development_plan, "E-SOUNDNESS-MISSING-CASE");
    assert!(development_plan.diagnostics.iter().any(|diagnostic| {
        diagnostic.detail_key
            == "fail_soundness.required_case.soundness.false_arithmetic.one_eq_zero"
    }));
    assert!(development_plan.diagnostics.iter().any(|diagnostic| {
        diagnostic.detail_key
            == "fail_soundness.required_case.soundness.certificate.invalid_sat_proof"
    }));
    assert!(development_plan.diagnostics.iter().any(|diagnostic| {
        diagnostic.detail_key
            == "fail_soundness.required_case.soundness.certificate.invalid_sat_refutation"
    }));
    assert!(development_plan.diagnostics.iter().any(|diagnostic| {
        diagnostic.detail_key
            == "fail_soundness.required_case.soundness.certificate.context_mismatch"
    }));
    assert!(development_plan.diagnostics.iter().any(|diagnostic| {
        diagnostic.detail_key
            == "fail_soundness.required_case.soundness.certificate.missing_provenance"
    }));
    assert!(development_plan.diagnostics.iter().any(|diagnostic| {
        diagnostic.detail_key
            == "fail_soundness.required_case.soundness.certificate.unsupported_legacy_certificate"
    }));

    let mut release_config = corpus.config();
    release_config.validation_mode = ValidationMode::Release;
    let release_plan = build_test_plan(&release_config).unwrap();

    assert_has_code(&release_plan, "E-SOUNDNESS-MISSING-CASE");
}

#[test]
fn recognized_soundness_sidecars_activate_missing_case_bookkeeping() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.test.proof"
source = "doc/spec/en/test.md"
section = "Test"
stage = "proof_verification"
status = "planned"
required = true
coverage = "fail"
tests = []
"#,
    );
    corpus.write("doc/spec/en/test.md", "# Test\n");
    corpus.write(
        "tests/miz/fail/proof/false_arithmetic.miz",
        "theorem 1 = 0;",
    );
    corpus.write(
        "tests/miz/fail/proof/false_arithmetic.expect.toml",
        r#"schema_version = 1
id = "false_arithmetic"
kind = "fail"
stage = "proof_verification"
domain = "soundness"
source = "false_arithmetic.miz"
expected_outcome = "fail"
expected_phase = "verification"
failure_category = "proof_failure"
stable_detail_key = "soundness.false_arithmetic.one_eq_zero"
diagnostic_codes = ["E-PROOF-TEST"]
spec_refs = ["spec.en.test.proof"]
"#,
    );

    let plan = corpus.plan();

    assert_has_code(&plan, "W-SOUNDNESS-MISSING-CASE");
    assert!(plan.diagnostics.iter().all(|diagnostic| {
        diagnostic.detail_key
            != "fail_soundness.required_case.soundness.false_arithmetic.one_eq_zero"
    }));
}

#[test]
fn corrected_certificate_soundness_keys_are_required_cases() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.test.certificate"
source = "doc/spec/en/test.md"
section = "Test"
stage = "advanced_semantics"
status = "planned"
required = true
coverage = "fail"
tests = []
"#,
    );
    corpus.write("doc/spec/en/test.md", "# Test\n");
    for (id, phase, category, reason, key) in [
        (
            "corrected_invalid_sat_refutation",
            "kernel_check",
            "kernel_rejection",
            "invalid_sat_refutation",
            "soundness.certificate.invalid_sat_refutation",
        ),
        (
            "corrected_context_mismatch",
            "certificate_check",
            "certificate_rejection",
            "context_mismatch",
            "soundness.certificate.context_mismatch",
        ),
        (
            "corrected_missing_provenance",
            "kernel_check",
            "kernel_rejection",
            "missing_provenance",
            "soundness.certificate.missing_provenance",
        ),
        (
            "corrected_unsupported_legacy_certificate",
            "certificate_check",
            "certificate_rejection",
            "unsupported_certificate_format",
            "soundness.certificate.unsupported_legacy_certificate",
        ),
    ] {
        corpus.write(format!("tests/certificates/fail/{id}.cert.json"), "{}");
        corpus.write(
            format!("tests/certificates/fail/{id}.expect.toml"),
            format!(
                r#"schema_version = 1
id = "{id}"
kind = "fail"
stage = "advanced_semantics"
domain = "certificate"
source = "{id}.cert.json"
expected_outcome = "fail"
expected_phase = "{phase}"
failure_category = "{category}"
rejection_reason = "{reason}"
stable_detail_key = "{key}"
diagnostic_codes = []
spec_refs = ["spec.en.test.certificate"]
"#
            ),
        );
    }

    let plan = corpus.plan();

    assert!(
        plan.diagnostics
            .iter()
            .all(|diagnostic| { !diagnostic.code.0.starts_with("E-EXPECT-SOUNDNESS") }),
        "{:#?}",
        plan.diagnostics
    );
}

#[test]
fn invalid_soundness_identity_does_not_satisfy_required_case_bookkeeping() {
    let corpus = Corpus::new();
    corpus.write("tests/certificates/fail/invalid_sat_proof.cert.json", "{}");
    corpus.write(
        "tests/certificates/fail/invalid_sat_proof.expect.toml",
        r#"schema_version = 1
id = "invalid_sat_proof"
kind = "fail"
stage = "advanced_semantics"
domain = "certificate"
source = "invalid_sat_proof.cert.json"
expected_outcome = "fail"
expected_phase = "kernel_check"
failure_category = "certificate_rejection"
rejection_reason = "malformed_certificate"
stable_detail_key = "soundness.certificate.invalid_sat_proof"
diagnostic_codes = ["E-KERNEL-TEST"]
spec_refs = ["spec.en.test.certificate"]
"#,
    );
    corpus.write(
        "tests/certificates/fail/invalid_sat_refutation.cert.json",
        "{}",
    );
    corpus.write(
        "tests/certificates/fail/invalid_sat_refutation.expect.toml",
        r#"schema_version = 1
id = "invalid_sat_refutation"
kind = "fail"
stage = "advanced_semantics"
domain = "certificate"
source = "invalid_sat_refutation.cert.json"
expected_outcome = "fail"
expected_phase = "kernel_check"
failure_category = "kernel_rejection"
rejection_reason = "invalid_sat_proof"
stable_detail_key = "soundness.certificate.invalid_sat_refutation"
diagnostic_codes = []
spec_refs = ["spec.en.test.certificate"]
"#,
    );
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.test.certificate"
source = "doc/spec/en/test.md"
section = "Test"
stage = "advanced_semantics"
status = "planned"
required = true
coverage = "fail"
tests = []
"#,
    );
    corpus.write("doc/spec/en/test.md", "# Test\n");
    let mut development_config = corpus.config();
    development_config.validation_mode = ValidationMode::Development;

    let plan = build_test_plan(&development_config).unwrap();

    assert_has_code(&plan, "E-EXPECT-SOUNDNESS-CATEGORY");
    assert_has_code(&plan, "E-EXPECT-SOUNDNESS-REJECTION-REASON");
    assert!(plan.diagnostics.iter().any(|diagnostic| {
        diagnostic.detail_key
            == "fail_soundness.required_case.soundness.certificate.invalid_sat_proof"
    }));
    assert!(plan.diagnostics.iter().any(|diagnostic| {
        diagnostic.detail_key
            == "expectation.soundness_rejection_reason.soundness.certificate.invalid_sat_refutation"
    }));
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
fn manifest_requirement_ids_must_be_sorted() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.test.z"
source = "doc/spec/en/test.md"
section = "Test"
stage = "lexical"
status = "planned"
required = true
coverage = "pass"
tests = []

[[requirement]]
id = "spec.en.test.a"
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

    assert_has_code(&plan, "E-MANIFEST-ID-ORDER");
}

#[test]
fn coverage_report_computes_shapes_status_and_pass_fail_mix() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.coverage.diagnostic"
source = "doc/spec/en/test.md"
section = "Test"
stage = "lexical"
status = "covered"
required = true
coverage = "diagnostic"
tests = ["tests/lexical/fail/fail_diagnostic_case_001.expect.toml"]

[[requirement]]
id = "spec.en.coverage.pass_and_fail"
source = "doc/spec/en/test.md"
section = "Test"
stage = "lexical"
status = "covered"
required = true
coverage = "pass_and_fail"
tests = [
  "tests/lexical/pass/pass_case_001.expect.toml",
  "tests/lexical/fail/fail_case_001.expect.toml",
]

[[requirement]]
id = "spec.en.coverage.property"
source = "doc/spec/en/test.md"
section = "Test"
stage = "lexical"
status = "covered"
required = true
coverage = "property"
tests = ["tests/property/property_case_001.expect.toml"]

[[requirement]]
id = "spec.en.coverage.snapshot"
source = "doc/spec/en/test.md"
section = "Test"
stage = "parse_only"
status = "covered"
required = true
coverage = "snapshot"
tests = ["tests/miz/pass/parser/pass_snapshot_case_001.expect.toml"]
"#,
    );
    corpus.write("doc/spec/en/test.md", "# Test\n");
    corpus.write("tests/lexical/pass/pass_case_001.src", "alpha");
    corpus.write(
        "tests/lexical/pass/pass_case_001.expect.toml",
        pass_expectation(
            "pass_case_001",
            "pass_case_001.src",
            "spec.en.coverage.pass_and_fail",
        ),
    );
    corpus.write("tests/lexical/fail/fail_case_001.src", "bad");
    corpus.write(
        "tests/lexical/fail/fail_case_001.expect.toml",
        fail_expectation(
            "fail_case_001",
            "fail_case_001.src",
            "spec.en.coverage.pass_and_fail",
        ),
    );
    corpus.write("tests/lexical/fail/fail_diagnostic_case_001.src", "bad");
    corpus.write(
        "tests/lexical/fail/fail_diagnostic_case_001.expect.toml",
        fail_expectation(
            "fail_diagnostic_case_001",
            "fail_diagnostic_case_001.src",
            "spec.en.coverage.diagnostic",
        ),
    );
    corpus.write(
        "tests/property/property_case_001.fixture.toml",
        "seed = \"1\"\n",
    );
    corpus.write(
        "tests/property/property_case_001.expect.toml",
        property_seed_expectation(
            "property_case_001",
            "property_case_001.fixture.toml",
            "spec.en.coverage.property",
            r#"profiles = ["full"]"#,
            false,
        ),
    );
    corpus.write(
        "tests/miz/pass/parser/pass_snapshot_case_001.miz",
        "alpha;\n",
    );
    corpus.write(
        "tests/miz/pass/parser/pass_snapshot_case_001.expect.toml",
        r#"schema_version = 1
id = "pass_snapshot_case_001"
kind = "pass"
stage = "parse_only"
domain = "parser"
source = "pass_snapshot_case_001.miz"
expected_outcome = "pass"
expected_phase = "parse"
diagnostic_codes = []
snapshots = "snapshots/parser/pass_snapshot_case_001.surface_ast.snap"
tags = ["active_parse_only"]
spec_refs = ["spec.en.coverage.snapshot"]
"#,
    );

    let plan = corpus.plan();

    assert_eq!(plan.error_count(), 0, "{:#?}", plan.diagnostics);
    assert_eq!(plan.coverage_report.pass_fail_mix.pass, 2);
    assert_eq!(plan.coverage_report.pass_fail_mix.fail, 2);
    assert_eq!(plan.coverage_report.pass_fail_mix.total, 4);
    let pass_and_fail = plan
        .coverage_report
        .requirements
        .iter()
        .find(|coverage| coverage.id.0 == "spec.en.coverage.pass_and_fail")
        .unwrap();
    assert_eq!(pass_and_fail.computed_status, RequirementStatus::Covered);
    assert!(pass_and_fail.missing_shapes.is_empty());
    let snapshot = plan
        .coverage_report
        .requirements
        .iter()
        .find(|coverage| coverage.id.0 == "spec.en.coverage.snapshot")
        .unwrap();
    assert_eq!(snapshot.coverage, CoverageShape::Snapshot);
    assert_eq!(snapshot.evidence.snapshot, 1);

    let lexical = plan
        .coverage_report
        .stages
        .iter()
        .find(|stage| stage.stage == Stage::Lexical)
        .unwrap();
    assert_eq!(lexical.requirements, 3);
    assert_eq!(lexical.covered, 3);
    assert_eq!(lexical.missing_shapes, 0);
    let parse_only = plan
        .coverage_report
        .stages
        .iter()
        .find(|stage| stage.stage == Stage::ParseOnly)
        .unwrap();
    assert_eq!(parse_only.requirements, 1);
    assert_eq!(parse_only.covered, 1);
}

#[test]
fn architecture22_matrix_metadata_reports_planned_rows() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.architecture_22.regression_matrix.metadata"
source = "doc/design/architecture/en/22.incremental_verification_contract.md"
section = "Regression matrix metadata"
stage = "advanced_semantics"
status = "partial"
required = true
coverage = "manual_review"
tests = ["tests/property/architecture22_matrix_001.expect.toml"]
"#,
    );
    corpus.write(
        "doc/design/architecture/en/22.incremental_verification_contract.md",
        "# Incremental Verification Contract\n",
    );
    corpus.write(
        "tests/property/architecture22_matrix_001.fixture.toml",
        "matrix = \"architecture22\"\n",
    );
    corpus.write(
        "tests/property/architecture22_matrix_001.expect.toml",
        architecture22_matrix_expectation(),
    );

    let plan = corpus.plan();

    assert_eq!(plan.error_count(), 0, "{:#?}", plan.diagnostics);
    let matrix = &plan.coverage_report.architecture22_matrix;
    assert!(matrix.missing_scenarios.is_empty());
    assert_eq!(
        matrix.scenarios.len(),
        architecture22_scenario_specs().len()
    );
    assert!(
        matrix
            .scenarios
            .iter()
            .all(|scenario| scenario.planned == 1)
    );
    assert!(matrix.scenarios.iter().all(|scenario| scenario.active == 0));
    let theorem_proof_body = matrix
        .scenarios
        .iter()
        .find(|scenario| scenario.scenario_id == "theorem_proof_body_invalidation")
        .unwrap();
    assert_eq!(theorem_proof_body.equivalence_class, "local_refresh_only");
    let coverage = plan
        .coverage_report
        .requirements
        .iter()
        .find(|coverage| coverage.id.0 == "spec.en.architecture_22.regression_matrix.metadata")
        .unwrap();
    assert_eq!(coverage.computed_status, RequirementStatus::Partial);
    assert_eq!(coverage.evidence.manual_review, 1);
}

#[test]
fn architecture22_matrix_defaults_gate_to_planned() {
    let corpus = Corpus::new();
    corpus.add_requirement("spec.en.test.basic", &[]);
    corpus.write(
        "tests/property/default_gate_001.fixture.toml",
        "seed = \"1\"\n",
    );
    corpus.write(
        "tests/property/default_gate_001.expect.toml",
        property_seed_expectation_with_extra(
            "default_gate_001",
            "default_gate_001.fixture.toml",
            "spec.en.test.basic",
            r#"profiles = ["full"]"#,
            true,
            r#"diagnostic_codes = []
architecture22_scenarios = ["cache_hit_miss_timing"]
architecture22_equivalence_class = "observable_outputs_equal""#,
        ),
    );

    let plan = corpus.plan();

    assert_eq!(plan.error_count(), 0, "{:#?}", plan.diagnostics);
    let cache_timing = plan
        .coverage_report
        .architecture22_matrix
        .scenarios
        .iter()
        .find(|scenario| scenario.scenario_id == "cache_hit_miss_timing")
        .unwrap();
    assert_eq!(cache_timing.planned, 1);
    assert_eq!(cache_timing.active, 0);
}

#[test]
fn architecture22_matrix_reports_missing_scenarios() {
    let corpus = Corpus::new();
    corpus.write("tests/coverage/spec_trace.toml", "");

    let plan = corpus.plan();

    let matrix = &plan.coverage_report.architecture22_matrix;
    assert_eq!(
        matrix.scenarios.len(),
        architecture22_scenario_specs().len()
    );
    assert_eq!(
        matrix.missing_scenarios.len(),
        architecture22_scenario_specs().len()
    );
    assert_eq!(
        matrix.missing_scenarios.first().map(String::as_str),
        Some("artifact_manifest_atomicity")
    );
}

#[test]
fn architecture22_matrix_rejects_unknown_or_orphan_metadata() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/property/unknown_scenario_001.fixture.toml",
        "seed = \"1\"\n",
    );
    corpus.write(
        "tests/property/unknown_scenario_001.expect.toml",
        property_seed_expectation_with_extra(
            "unknown_scenario_001",
            "unknown_scenario_001.fixture.toml",
            "spec.en.test.basic",
            r#"profiles = ["full"]"#,
            true,
            r#"diagnostic_codes = []
architecture22_scenarios = ["unknown_scenario"]
architecture22_gate = "planned""#,
        ),
    );
    corpus.write(
        "tests/property/orphan_gate_001.fixture.toml",
        "seed = \"2\"\n",
    );
    corpus.write(
        "tests/property/orphan_gate_001.expect.toml",
        property_seed_expectation_with_extra(
            "orphan_gate_001",
            "orphan_gate_001.fixture.toml",
            "spec.en.test.basic",
            r#"profiles = ["full"]"#,
            true,
            r#"diagnostic_codes = []
architecture22_gate = "planned""#,
        ),
    );
    corpus.write(
        "tests/property/orphan_class_001.fixture.toml",
        "seed = \"3\"\n",
    );
    corpus.write(
        "tests/property/orphan_class_001.expect.toml",
        property_seed_expectation_with_extra(
            "orphan_class_001",
            "orphan_class_001.fixture.toml",
            "spec.en.test.basic",
            r#"profiles = ["full"]"#,
            true,
            r#"diagnostic_codes = []
architecture22_equivalence_class = "cache_miss_only""#,
        ),
    );
    corpus.write(
        "tests/property/unknown_class_001.fixture.toml",
        "seed = \"4\"\n",
    );
    corpus.write(
        "tests/property/unknown_class_001.expect.toml",
        property_seed_expectation_with_extra(
            "unknown_class_001",
            "unknown_class_001.fixture.toml",
            "spec.en.test.basic",
            r#"profiles = ["full"]"#,
            true,
            r#"diagnostic_codes = []
architecture22_scenarios = ["cache_hit_miss_timing"]
architecture22_equivalence_class = "not_a_class"
architecture22_gate = "planned""#,
        ),
    );

    let plan = corpus.plan();

    assert_has_message(
        &plan,
        "unknown architecture22_scenarios entry `unknown_scenario`",
    );
    assert_has_message(
        &plan,
        "`architecture22_gate` requires `architecture22_scenarios`",
    );
    assert_has_message(
        &plan,
        "`architecture22_equivalence_class` requires `architecture22_scenarios`",
    );
    assert_has_message(
        &plan,
        "unknown architecture22_equivalence_class `not_a_class`",
    );
}

#[test]
fn architecture22_matrix_rejects_noncanonical_or_active_metadata() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/property/duplicate_scenario_001.fixture.toml",
        "seed = \"1\"\n",
    );
    corpus.write(
        "tests/property/duplicate_scenario_001.expect.toml",
        property_seed_expectation_with_extra(
            "duplicate_scenario_001",
            "duplicate_scenario_001.fixture.toml",
            "spec.en.test.basic",
            r#"profiles = ["full"]"#,
            true,
            r#"diagnostic_codes = []
architecture22_scenarios = ["cache_hit_miss_timing", "cache_hit_miss_timing"]
architecture22_gate = "planned""#,
        ),
    );
    corpus.write(
        "tests/property/unsorted_scenario_001.fixture.toml",
        "seed = \"2\"\n",
    );
    corpus.write(
        "tests/property/unsorted_scenario_001.expect.toml",
        property_seed_expectation_with_extra(
            "unsorted_scenario_001",
            "unsorted_scenario_001.fixture.toml",
            "spec.en.test.basic",
            r#"profiles = ["full"]"#,
            true,
            r#"diagnostic_codes = []
architecture22_scenarios = ["cache_hit_miss_timing", "artifact_manifest_atomicity"]
architecture22_gate = "planned""#,
        ),
    );
    corpus.write(
        "tests/property/class_mismatch_001.fixture.toml",
        "seed = \"3\"\n",
    );
    corpus.write(
        "tests/property/class_mismatch_001.expect.toml",
        property_seed_expectation_with_extra(
            "class_mismatch_001",
            "class_mismatch_001.fixture.toml",
            "spec.en.test.basic",
            r#"profiles = ["full"]"#,
            true,
            r#"diagnostic_codes = []
architecture22_scenarios = ["cache_hit_miss_timing"]
architecture22_equivalence_class = "cache_miss_only"
architecture22_gate = "planned""#,
        ),
    );
    corpus.write("tests/property/bad_gate_001.fixture.toml", "seed = \"4\"\n");
    corpus.write(
        "tests/property/bad_gate_001.expect.toml",
        property_seed_expectation_with_extra(
            "bad_gate_001",
            "bad_gate_001.fixture.toml",
            "spec.en.test.basic",
            r#"profiles = ["full"]"#,
            true,
            r#"diagnostic_codes = []
architecture22_scenarios = ["cache_hit_miss_timing"]
architecture22_gate = "ready""#,
        ),
    );
    corpus.write(
        "tests/property/active_gate_001.fixture.toml",
        "seed = \"5\"\n",
    );
    corpus.write(
        "tests/property/active_gate_001.expect.toml",
        property_seed_expectation_with_extra(
            "active_gate_001",
            "active_gate_001.fixture.toml",
            "spec.en.test.basic",
            r#"profiles = ["full"]"#,
            true,
            r#"diagnostic_codes = []
architecture22_scenarios = ["cache_hit_miss_timing"]
architecture22_gate = "active""#,
        ),
    );

    let plan = corpus.plan();

    assert_has_message(
        &plan,
        "duplicate architecture22_scenarios entry `cache_hit_miss_timing`",
    );
    assert_has_message(
        &plan,
        "architecture22_scenarios entry `artifact_manifest_atomicity` must be sorted after `cache_hit_miss_timing`",
    );
    assert_has_message(
        &plan,
        "`architecture22_equivalence_class` `cache_miss_only` does not match scenario `cache_hit_miss_timing` registry class `observable_outputs_equal`",
    );
    assert_has_message(&plan, "unknown architecture22_gate `ready`");
    assert_has_message(
        &plan,
        "`architecture22_gate = \"active\"` is not allowed for scenarios without active eligibility: cache_hit_miss_timing",
    );
}

#[test]
fn coverage_status_drift_is_warning_in_metadata_and_error_in_development() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.coverage.pass_and_fail"
source = "doc/spec/en/test.md"
section = "Test"
stage = "lexical"
status = "covered"
required = true
coverage = "pass_and_fail"
tests = ["tests/lexical/pass/pass_only.expect.toml"]
"#,
    );
    corpus.write("doc/spec/en/test.md", "# Test\n");
    corpus.write("tests/lexical/pass/pass_only.src", "alpha");
    corpus.write(
        "tests/lexical/pass/pass_only.expect.toml",
        pass_expectation(
            "pass_only",
            "pass_only.src",
            "spec.en.coverage.pass_and_fail",
        ),
    );

    let metadata_plan = corpus.plan();

    assert_eq!(
        metadata_plan.error_count(),
        0,
        "{:#?}",
        metadata_plan.diagnostics
    );
    assert_has_code(&metadata_plan, "W-TRACE-STATUS-DRIFT");
    let coverage = &metadata_plan.coverage_report.requirements[0];
    assert_eq!(coverage.computed_status, RequirementStatus::Partial);
    assert_eq!(coverage.missing_shapes, vec![CoverageShape::Fail]);

    let mut development_config = corpus.config();
    development_config.validation_mode = ValidationMode::Development;
    let development_plan = build_test_plan(&development_config).unwrap();

    assert_has_code(&development_plan, "E-TRACE-STATUS-DRIFT");
    assert_has_code(&development_plan, "E-TRACE-MISSING-COVERAGE");
}

#[test]
fn release_requires_required_coverage_but_accepts_deferred_reason() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.coverage.deferred"
source = "doc/spec/en/test.md"
section = "Test"
stage = "lexical"
status = "deferred"
required = true
coverage = "pass"
deferred_reason = "blocked until the owning runner exists"
tests = []

[[requirement]]
id = "spec.en.coverage.planned"
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
    let mut release_config = corpus.config();
    release_config.validation_mode = ValidationMode::Release;

    let release_plan = build_test_plan(&release_config).unwrap();

    assert_has_code(&release_plan, "E-TRACE-MISSING-COVERAGE");
    assert!(
        release_plan.diagnostics.iter().all(|diagnostic| {
            diagnostic.detail_key != "trace.coverage.spec.en.coverage.deferred"
        }),
        "{:#?}",
        release_plan.diagnostics
    );
}

#[test]
fn release_reports_required_status_drift() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.coverage.release_drift"
source = "doc/spec/en/test.md"
section = "Test"
stage = "lexical"
status = "covered"
required = true
coverage = "pass_and_fail"
tests = ["tests/lexical/pass/pass_only.expect.toml"]
"#,
    );
    corpus.write("doc/spec/en/test.md", "# Test\n");
    corpus.write("tests/lexical/pass/pass_only.src", "alpha");
    corpus.write(
        "tests/lexical/pass/pass_only.expect.toml",
        pass_expectation(
            "pass_only",
            "pass_only.src",
            "spec.en.coverage.release_drift",
        ),
    );
    let mut release_config = corpus.config();
    release_config.validation_mode = ValidationMode::Release;

    let release_plan = build_test_plan(&release_config).unwrap();

    assert_has_code(&release_plan, "E-TRACE-STATUS-DRIFT");
    assert_has_code(&release_plan, "E-TRACE-MISSING-COVERAGE");
}

#[test]
fn obsolete_requirements_reject_sidecar_refs() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.coverage.obsolete"
source = "doc/spec/en/test.md"
section = "Test"
stage = "lexical"
status = "obsolete"
required = false
coverage = "none"
tests = []
"#,
    );
    corpus.write("doc/spec/en/test.md", "# Test\n");
    corpus.write("tests/lexical/pass/obsolete_ref.src", "alpha");
    corpus.write(
        "tests/lexical/pass/obsolete_ref.expect.toml",
        pass_expectation(
            "obsolete_ref",
            "obsolete_ref.src",
            "spec.en.coverage.obsolete",
        ),
    );

    let plan = corpus.plan();

    assert_has_code(&plan, "E-TRACE-OBSOLETE-SPEC-REF");
}

#[test]
fn manifest_link_validator_reports_error_paths() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.link.backref"
source = "doc/spec/en/test.md"
section = "Test"
stage = "lexical"
status = "planned"
required = true
coverage = "pass"
tests = ["tests/lexical/pass/wrong_ref.expect.toml"]

[[requirement]]
id = "spec.en.link.deferred"
source = "doc/spec/en/test.md"
section = "Test"
stage = "lexical"
status = "deferred"
required = true
coverage = "pass"
tests = []

[[requirement]]
id = "spec.en.link.duplicate"
source = "doc/spec/en/test.md"
section = "Test"
stage = "lexical"
status = "covered"
required = true
coverage = "pass"
tests = [
  "tests/lexical/pass/duplicate.expect.toml",
  "tests/lexical/pass/duplicate.expect.toml",
]

[[requirement]]
id = "spec.en.link.missing_source"
source = "doc/spec/en/missing.md"
section = "Test"
stage = "lexical"
status = "planned"
required = true
coverage = "pass"
tests = []

[[requirement]]
id = "spec.en.link.missing_test"
source = "doc/spec/en/test.md"
section = "Test"
stage = "lexical"
status = "planned"
required = true
coverage = "pass"
tests = ["tests/lexical/pass/missing.expect.toml"]

[[requirement]]
id = "spec.en.link.other"
source = "doc/spec/en/test.md"
section = "Test"
stage = "lexical"
status = "covered"
required = true
coverage = "pass"
tests = ["tests/lexical/pass/wrong_ref.expect.toml"]

[[requirement]]
id = "spec.en.link.planned"
source = "doc/spec/en/test.md"
section = "Test"
stage = "lexical"
status = "planned"
required = true
coverage = "pass"
tests = []

[[requirement]]
id = "spec.en.link.unparsed"
source = "doc/spec/en/test.md"
section = "Test"
stage = "lexical"
status = "planned"
required = true
coverage = "pass"
tests = ["tests/lexical/pass/bad.expect.toml"]
"#,
    );
    corpus.write("doc/spec/en/test.md", "# Test\n");
    corpus.write("tests/lexical/pass/duplicate.src", "alpha");
    corpus.write(
        "tests/lexical/pass/duplicate.expect.toml",
        pass_expectation("duplicate", "duplicate.src", "spec.en.link.duplicate"),
    );
    corpus.write("tests/lexical/pass/wrong_ref.src", "alpha");
    corpus.write(
        "tests/lexical/pass/wrong_ref.expect.toml",
        pass_expectation("wrong_ref", "wrong_ref.src", "spec.en.link.other"),
    );
    corpus.write(
        "tests/lexical/pass/bad.expect.toml",
        "schema_version = \"one\"\n",
    );

    let plan = corpus.plan();

    assert_has_code(&plan, "E-MANIFEST-MISSING-SOURCE");
    assert_has_code(&plan, "E-MANIFEST-DUP-TEST");
    assert_has_code(&plan, "E-MANIFEST-MISSING-TEST");
    assert_has_code(&plan, "E-TRACE-MISSING-BACKREF");
    assert_has_code(&plan, "E-TRACE-UNPARSED-TEST");
    assert_has_code(&plan, "E-MANIFEST-DEFERRED-REASON");
    assert_has_code(&plan, "W-MANIFEST-PLANNED-NO-TESTS");
}

#[test]
fn manifest_depends_on_validation_reports_error_paths() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.dep.bad_stage"
source = "doc/spec/en/test.md"
section = "Test"
stage = "parse_only"
status = "planned"
required = true
coverage = "pass"
depends_on = ["spec.en.dep.later"]
tests = []

[[requirement]]
id = "spec.en.dep.duplicate"
source = "doc/spec/en/test.md"
section = "Test"
stage = "type_elaboration"
status = "planned"
required = true
coverage = "pass"
depends_on = ["spec.en.dep.later", "spec.en.dep.later"]
tests = []

[[requirement]]
id = "spec.en.dep.empty"
source = "doc/spec/en/test.md"
section = "Test"
stage = "parse_only"
status = "planned"
required = true
coverage = "pass"
depends_on = [""]
tests = []

[[requirement]]
id = "spec.en.dep.later"
source = "doc/spec/en/test.md"
section = "Test"
stage = "declaration_symbol"
status = "planned"
required = true
coverage = "pass"
tests = []

[[requirement]]
id = "spec.en.dep.self"
source = "doc/spec/en/test.md"
section = "Test"
stage = "parse_only"
status = "planned"
required = true
coverage = "pass"
depends_on = ["spec.en.dep.self"]
tests = []

[[requirement]]
id = "spec.en.dep.unknown"
source = "doc/spec/en/test.md"
section = "Test"
stage = "parse_only"
status = "planned"
required = true
coverage = "pass"
depends_on = ["spec.en.dep.missing"]
tests = []
"#,
    );
    corpus.write("doc/spec/en/test.md", "# Test\n");

    let plan = corpus.plan();

    assert_has_code(&plan, "E-MANIFEST-DEPENDS-ON-STAGE");
    assert_has_code(&plan, "E-MANIFEST-DEPENDS-ON");
    assert_has_code(&plan, "E-MANIFEST-DUP-DEPENDS-ON");
    assert_has_code(&plan, "E-MANIFEST-SELF-DEPENDS-ON");
    assert_has_code(&plan, "E-MANIFEST-UNKNOWN-DEPENDS-ON");
}

#[test]
fn stage_prerequisites_allow_covered_and_built_in_dependencies() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.prereq.builtin"
source = "doc/spec/en/test.md"
section = "Test"
stage = "lexical"
status = "covered"
required = true
coverage = "none"
built_in = true
tests = []

[[requirement]]
id = "spec.en.prereq.lexical"
source = "doc/spec/en/test.md"
section = "Test"
stage = "lexical"
status = "covered"
required = true
coverage = "pass"
tests = ["tests/lexical/pass/prereq_lexical.expect.toml"]

[[requirement]]
id = "spec.en.prereq.parse_builtin"
source = "doc/spec/en/test.md"
section = "Test"
stage = "parse_only"
status = "covered"
required = true
coverage = "pass"
depends_on = ["spec.en.prereq.builtin"]
tests = ["tests/miz/pass/parser/prereq_parse_builtin.expect.toml"]

[[requirement]]
id = "spec.en.prereq.parse_covered"
source = "doc/spec/en/test.md"
section = "Test"
stage = "parse_only"
status = "covered"
required = true
coverage = "pass"
depends_on = ["spec.en.prereq.lexical"]
tests = ["tests/miz/pass/parser/prereq_parse_covered.expect.toml"]
"#,
    );
    corpus.write("doc/spec/en/test.md", "# Test\n");
    corpus.write("tests/lexical/pass/prereq_lexical.src", "alpha");
    corpus.write(
        "tests/lexical/pass/prereq_lexical.expect.toml",
        pass_expectation(
            "prereq_lexical",
            "prereq_lexical.src",
            "spec.en.prereq.lexical",
        ),
    );
    corpus.write("tests/miz/pass/parser/prereq_parse_builtin.miz", "alpha;");
    corpus.write(
        "tests/miz/pass/parser/prereq_parse_builtin.expect.toml",
        parse_pass_expectation(
            "prereq_parse_builtin",
            "prereq_parse_builtin.miz",
            "spec.en.prereq.parse_builtin",
        ),
    );
    corpus.write("tests/miz/pass/parser/prereq_parse_covered.miz", "beta;");
    corpus.write(
        "tests/miz/pass/parser/prereq_parse_covered.expect.toml",
        parse_pass_expectation(
            "prereq_parse_covered",
            "prereq_parse_covered.miz",
            "spec.en.prereq.parse_covered",
        ),
    );

    let plan = corpus.plan();

    assert_eq!(plan.error_count(), 0, "{:#?}", plan.diagnostics);
    for requirement_id in [
        "spec.en.prereq.parse_builtin",
        "spec.en.prereq.parse_covered",
    ] {
        let coverage = plan
            .coverage_report
            .requirements
            .iter()
            .find(|coverage| coverage.id.0 == requirement_id)
            .unwrap_or_else(|| panic!("coverage for {requirement_id} should exist"));
        assert_eq!(coverage.computed_status, RequirementStatus::Covered);
    }
}

#[test]
fn unsatisfied_stage_prerequisite_blocks_coverage_credit() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.prereq.lower"
source = "doc/spec/en/test.md"
section = "Test"
stage = "lexical"
status = "planned"
required = true
coverage = "pass"
tests = []

[[requirement]]
id = "spec.en.prereq.upper"
source = "doc/spec/en/test.md"
section = "Test"
stage = "parse_only"
status = "covered"
required = true
coverage = "pass"
depends_on = ["spec.en.prereq.lower"]
tests = ["tests/miz/pass/parser/prereq_upper.expect.toml"]
"#,
    );
    corpus.write("doc/spec/en/test.md", "# Test\n");
    corpus.write("tests/miz/pass/parser/prereq_upper.miz", "alpha;");
    corpus.write(
        "tests/miz/pass/parser/prereq_upper.expect.toml",
        parse_pass_expectation("prereq_upper", "prereq_upper.miz", "spec.en.prereq.upper"),
    );

    let plan = corpus.plan();

    assert_has_code(&plan, "E-TRACE-PREREQUISITE");
    let coverage = plan
        .coverage_report
        .requirements
        .iter()
        .find(|coverage| coverage.id.0 == "spec.en.prereq.upper")
        .expect("upper requirement coverage should exist");
    assert_eq!(coverage.computed_status, RequirementStatus::Planned);
}

#[test]
fn stage_mismatch_blocks_coverage_credit() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.stage.lexical"
source = "doc/spec/en/test.md"
section = "Test"
stage = "lexical"
status = "covered"
required = true
coverage = "pass"
tests = ["tests/miz/pass/parser/stage_mismatch.expect.toml"]
"#,
    );
    corpus.write("doc/spec/en/test.md", "# Test\n");
    corpus.write("tests/miz/pass/parser/stage_mismatch.miz", "alpha;");
    corpus.write(
        "tests/miz/pass/parser/stage_mismatch.expect.toml",
        parse_pass_expectation(
            "stage_mismatch",
            "stage_mismatch.miz",
            "spec.en.stage.lexical",
        ),
    );

    let plan = corpus.plan();

    assert_has_code(&plan, "E-TRACE-STAGE-MISMATCH");
    let coverage = plan
        .coverage_report
        .requirements
        .iter()
        .find(|coverage| coverage.id.0 == "spec.en.stage.lexical")
        .expect("lexical requirement coverage should exist");
    assert_eq!(coverage.computed_status, RequirementStatus::Planned);
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
fn metadata_plans_diagnostics_and_coverage_are_byte_stable_across_repeated_builds() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.determinism.pass"
source = "doc/spec/en/test.md"
section = "Test"
stage = "lexical"
status = "covered"
required = true
coverage = "pass"
tests = [
  "tests/lexical/pass/pass_determinism_a_001.expect.toml",
  "tests/lexical/pass/pass_determinism_bad_name.expect.toml",
  "tests/lexical/pass/pass_determinism_z_001.expect.toml",
]

[[requirement]]
id = "spec.en.determinism.planned"
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
    corpus.write("tests/lexical/pass/pass_determinism_z_001.src", "zeta");
    corpus.write(
        "tests/lexical/pass/pass_determinism_z_001.expect.toml",
        pass_expectation(
            "pass_determinism_z_001",
            "pass_determinism_z_001.src",
            "spec.en.determinism.pass",
        ),
    );
    corpus.write("tests/lexical/pass/pass_determinism_a_001.src", "alpha");
    corpus.write(
        "tests/lexical/pass/pass_determinism_a_001.expect.toml",
        pass_expectation(
            "pass_determinism_a_001",
            "pass_determinism_a_001.src",
            "spec.en.determinism.pass",
        ),
    );
    corpus.write("tests/lexical/pass/pass_determinism_bad_name.src", "naming");
    corpus.write(
        "tests/lexical/pass/pass_determinism_bad_name.expect.toml",
        pass_expectation(
            "pass_determinism_bad_name",
            "pass_determinism_bad_name.src",
            "spec.en.determinism.pass",
        ),
    );
    corpus.write("tests/unknown_determinism/README.txt", "layout drift\n");

    let config = DiscoveryConfig {
        validation_mode: ValidationMode::Development,
        ..corpus.config()
    };
    let first = canonical_test_plan(&build_test_plan(&config).unwrap(), &corpus.root);
    let second = canonical_test_plan(&build_test_plan(&config).unwrap(), &corpus.root);
    let third = canonical_test_plan(&build_test_plan(&config).unwrap(), &corpus.root);

    assert_eq!(first, second);
    assert_eq!(second, third);
    assert!(first.contains("case|pass_determinism_a_001"));
    assert!(first.contains("case|pass_determinism_z_001"));
    assert!(
        first.find("case|pass_determinism_a_001").unwrap()
            < first.find("case|pass_determinism_z_001").unwrap()
    );
    let manifest_warning =
        "diagnostic|warning|tests/coverage/spec_trace.toml|manifest|W-MANIFEST-PLANNED-NO-TESTS"
            .to_owned();
    let naming_warning = "diagnostic|warning|tests/lexical/pass/pass_determinism_bad_name.expect.toml|corpus|W-CORPUS-NAMING|corpus.naming.pass_determinism_bad_name".to_owned();
    let layout_error =
        "diagnostic|error|tests/unknown_determinism|layout|E-LAYOUT-UNKNOWN-ROOT".to_owned();
    assert!(first.contains(&manifest_warning));
    assert!(first.contains(&naming_warning));
    assert!(first.contains(&layout_error));
    assert!(
        first.find(&manifest_warning).unwrap() < first.find(&naming_warning).unwrap()
            && first.find(&naming_warning).unwrap() < first.find(&layout_error).unwrap()
    );
    assert!(first.contains("coverage|spec.en.determinism.pass|lexical|pass|true|covered|covered"));
}

#[test]
fn active_runner_reports_are_byte_stable_across_repeated_runs() {
    let config = repository_config();
    let root = config.workspace_root.clone();

    let parse_first = canonical_parse_only_report(&run_parse_only_corpus(&config).unwrap(), &root);
    let parse_second = canonical_parse_only_report(&run_parse_only_corpus(&config).unwrap(), &root);
    assert_eq!(parse_first, parse_second);
    assert!(parse_first.contains("parse-only-result|pass_parser_template_arguments_001"));

    let declaration_first = canonical_declaration_symbol_report(
        &run_declaration_symbol_corpus(&config).unwrap(),
        &root,
    );
    let declaration_second = canonical_declaration_symbol_report(
        &run_declaration_symbol_corpus(&config).unwrap(),
        &root,
    );
    assert_eq!(declaration_first, declaration_second);
    assert!(
        declaration_first
            .contains("declaration-symbol-result|fail_resolve_duplicate_theorem_symbol_001")
    );

    let type_first =
        canonical_type_elaboration_report(&run_type_elaboration_corpus(&config).unwrap(), &root);
    let type_second =
        canonical_type_elaboration_report(&run_type_elaboration_corpus(&config).unwrap(), &root);
    assert_eq!(type_first, type_second);
    assert!(
        type_first
            .contains("type-elaboration-result|fail_type_elaboration_payload_extraction_gap_001")
    );
    assert!(
        type_first
            .contains("type-elaboration-result|fail_type_elaboration_non_builtin_type_gap_001")
    );
    assert!(
        type_first
            .contains("type-elaboration-result|pass_type_elaboration_builtin_type_expression_001")
    );
}

#[test]
fn active_runner_failure_reports_are_byte_stable_across_repeated_runs() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.determinism.snapshot"
source = "doc/spec/en/test.md"
section = "Test"
stage = "parse_only"
status = "covered"
required = true
coverage = "snapshot"
tests = ["tests/miz/pass/parser/pass_snapshot_failure_001.expect.toml"]
"#,
    );
    corpus.write("doc/spec/en/test.md", "# Test\n");
    corpus.write(
        "tests/miz/pass/parser/pass_snapshot_failure_001.miz",
        "alpha;",
    );
    corpus.write(
        "tests/snapshots/parser/pass_snapshot_failure_001.surface_ast.snap",
        "intentionally stale snapshot\n",
    );
    corpus.write(
        "tests/miz/pass/parser/pass_snapshot_failure_001.expect.toml",
        r#"schema_version = 1
id = "pass_snapshot_failure_001"
kind = "pass"
stage = "parse_only"
domain = "parser"
source = "pass_snapshot_failure_001.miz"
expected_outcome = "pass"
expected_phase = "parse"
diagnostic_codes = []
snapshots = "snapshots/parser/pass_snapshot_failure_001.surface_ast.snap"
tags = ["active_parse_only"]
spec_refs = ["spec.en.determinism.snapshot"]
"#,
    );

    let config = corpus.config();
    let root = config.workspace_root.clone();
    let first = canonical_parse_only_report(&run_parse_only_corpus(&config).unwrap(), &root);
    let second = canonical_parse_only_report(&run_parse_only_corpus(&config).unwrap(), &root);

    assert_eq!(first, second);
    assert!(first.contains("parse-only-counts|1|0|1|1"));
    assert!(
        first.contains(
            "parse-only-result|pass_snapshot_failure_001|tests/miz/pass/parser/pass_snapshot_failure_001.expect.toml|failed|codes=|snapshot=SurfaceAst snapshot"
        )
    );
    assert!(first.contains(
        "diagnostic|error|tests/miz/pass/parser/pass_snapshot_failure_001.expect.toml|parse_only|E-PARSE-ONLY-SNAPSHOT"
    ));
}

#[test]
fn repository_corpus_plan_succeeds() {
    let plan = repository_plan();

    assert_eq!(plan.error_count(), 0, "{:#?}", plan.diagnostics);
    for key in [
        "soundness.certificate.invalid_sat_refutation",
        "soundness.certificate.context_mismatch",
        "soundness.certificate.missing_provenance",
        "soundness.certificate.unsupported_legacy_certificate",
    ] {
        assert!(
            plan.cases
                .iter()
                .any(|case| case.expectation.stable_detail_key.as_deref() == Some(key)),
            "repository corpus should contain corrected required soundness case `{key}`"
        );
        let missing_detail_key = format!("fail_soundness.required_case.{key}");
        assert!(
            plan.diagnostics.iter().all(|diagnostic| {
                diagnostic.detail_key != missing_detail_key
                    || diagnostic.code.0 != "W-SOUNDNESS-MISSING-CASE"
            }),
            "corrected required soundness case `{key}` should not be reported missing: {:#?}",
            plan.diagnostics
        );
    }
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
    let matrix = &plan.coverage_report.architecture22_matrix;
    assert_eq!(
        matrix.scenarios.len(),
        architecture22_scenario_specs().len()
    );
    assert!(
        matrix.missing_scenarios.is_empty(),
        "{:?}",
        matrix.missing_scenarios
    );
    assert!(
        matrix
            .scenarios
            .iter()
            .all(|scenario| scenario.planned == 1 && scenario.active == 0),
        "{:?}",
        matrix.scenarios
    );
    let matrix_requirement = plan
        .coverage_report
        .requirements
        .iter()
        .find(|coverage| coverage.id.0 == "spec.en.architecture_22.regression_matrix.metadata")
        .expect("repository should include architecture-22 matrix trace row");
    assert_eq!(
        matrix_requirement.computed_status,
        RequirementStatus::Partial
    );
    assert_eq!(matrix_requirement.evidence.manual_review, 1);
}

#[test]
fn repository_declaration_symbol_runner_executes_active_resolver_seeds() {
    let report = run_declaration_symbol_corpus(&repository_config()).unwrap();

    assert_eq!(report.error_count(), 0, "{:#?}", report.diagnostics);
    assert_eq!(report.results.len(), 4);
    assert_eq!(report.passed_count(), 4);
    assert_eq!(report.failed_count(), 0);
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_resolve_declaration_symbol_smoke_001"
            && result.actual_detail_keys.is_empty()
            && result.actual_payload_keys == expected_declaration_symbol_smoke_payloads()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_resolve_parameterized_local_attribute_001"
            && result.actual_detail_keys.is_empty()
            && result.actual_payload_keys == parameterized_attribute_declaration_symbol_payloads()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_resolve_duplicate_theorem_symbol_001"
            && result.actual_detail_keys == ["declaration_symbol.symbol.duplicate_declaration"]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_resolve_same_signature_return_conflict_001"
            && result.actual_detail_keys
                == ["declaration_symbol.signature.same_signature_return_conflict"]
    }));
}

fn expected_declaration_symbol_smoke_payloads() -> Vec<String> {
    [
        "declaration_symbol.definition.kind.Carrier.structure",
        "declaration_symbol.definition.kind.CarrierMode.mode",
        "declaration_symbol.definition.kind.VisibleLemma.lemma",
        "declaration_symbol.definition.kind.VisibleTheorem.theorem",
        "declaration_symbol.definition.kind.carrier.selector",
        "declaration_symbol.definition.kind.flagged.attribute",
        "declaration_symbol.definition.kind.hidden_f%20x.functor",
        "declaration_symbol.definition.kind.x%20R%20y.predicate",
        "declaration_symbol.definition.visibility.Carrier.public",
        "declaration_symbol.definition.visibility.CarrierMode.public",
        "declaration_symbol.definition.visibility.VisibleLemma.public",
        "declaration_symbol.definition.visibility.VisibleTheorem.public",
        "declaration_symbol.definition.visibility.carrier.public",
        "declaration_symbol.definition.visibility.flagged.public",
        "declaration_symbol.definition.visibility.hidden_f%20x.private",
        "declaration_symbol.definition.visibility.x%20R%20y.public",
        "declaration_symbol.symbol.export.Carrier.exported",
        "declaration_symbol.symbol.export.CarrierMode.exported",
        "declaration_symbol.symbol.export.VisibleLemma.exported",
        "declaration_symbol.symbol.export.VisibleTheorem.exported",
        "declaration_symbol.symbol.export.carrier.exported",
        "declaration_symbol.symbol.export.flagged.exported",
        "declaration_symbol.symbol.export.hidden_f%20x.local_only",
        "declaration_symbol.symbol.export.x%20R%20y.exported",
        "declaration_symbol.symbol.kind.Carrier.structure",
        "declaration_symbol.symbol.kind.CarrierMode.mode",
        "declaration_symbol.symbol.kind.VisibleLemma.lemma",
        "declaration_symbol.symbol.kind.VisibleTheorem.theorem",
        "declaration_symbol.symbol.kind.carrier.selector",
        "declaration_symbol.symbol.kind.flagged.attribute",
        "declaration_symbol.symbol.kind.hidden_f%20x.functor",
        "declaration_symbol.symbol.kind.x%20R%20y.predicate",
        "declaration_symbol.symbol.visibility.Carrier.public",
        "declaration_symbol.symbol.visibility.CarrierMode.public",
        "declaration_symbol.symbol.visibility.VisibleLemma.public",
        "declaration_symbol.symbol.visibility.VisibleTheorem.public",
        "declaration_symbol.symbol.visibility.carrier.public",
        "declaration_symbol.symbol.visibility.flagged.public",
        "declaration_symbol.symbol.visibility.hidden_f%20x.private",
        "declaration_symbol.symbol.visibility.x%20R%20y.public",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

fn parameterized_attribute_declaration_symbol_payloads() -> Vec<String> {
    [
        "declaration_symbol.definition.kind.ranked.attribute",
        "declaration_symbol.definition.visibility.ranked.public",
        "declaration_symbol.symbol.export.ranked.exported",
        "declaration_symbol.symbol.kind.ranked.attribute",
        "declaration_symbol.symbol.visibility.ranked.public",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

fn visible_theorem_declaration_symbol_payloads() -> Vec<String> {
    [
        "declaration_symbol.definition.kind.VisibleTheorem.theorem",
        "declaration_symbol.definition.visibility.VisibleTheorem.public",
        "declaration_symbol.symbol.export.VisibleTheorem.exported",
        "declaration_symbol.symbol.kind.VisibleTheorem.theorem",
        "declaration_symbol.symbol.visibility.VisibleTheorem.public",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

fn plus_functor_declaration_symbol_payloads() -> Vec<String> {
    [
        "declaration_symbol.definition.kind.x%20%2B%2B%20y.functor",
        "declaration_symbol.definition.visibility.x%20%2B%2B%20y.public",
        "declaration_symbol.symbol.export.x%20%2B%2B%20y.exported",
        "declaration_symbol.symbol.kind.x%20%2B%2B%20y.functor",
        "declaration_symbol.symbol.visibility.x%20%2B%2B%20y.public",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

#[test]
fn repository_type_elaboration_runner_executes_active_source_derived_seeds() {
    let config = repository_config();
    let plan = build_test_plan(&config).unwrap();
    let inline_definition_case = active_type_elaboration_cases(&plan)
        .find(|case| case.id.0 == "fail_type_elaboration_inline_definition_gap_001")
        .expect("Task94 inline definition boundary should be active");
    assert_eq!(
        inline_definition_case
            .expectation
            .rejection_reason
            .as_deref(),
        Some("inline_definition_payload_extraction_gap")
    );
    let registration_block_case = active_type_elaboration_cases(&plan)
        .find(|case| case.id.0 == "fail_type_elaboration_registration_block_gap_001")
        .expect("Task95 registration block boundary should be active");
    assert_eq!(
        registration_block_case
            .expectation
            .rejection_reason
            .as_deref(),
        Some("registration_block_payload_extraction_gap")
    );
    let redefinition_notation_case = active_type_elaboration_cases(&plan)
        .find(|case| case.id.0 == "fail_type_elaboration_redefinition_notation_gap_001")
        .expect("Task96 redefinition/notation boundary should be active");
    assert_eq!(
        redefinition_notation_case
            .expectation
            .rejection_reason
            .as_deref(),
        Some("redefinition_notation_payload_extraction_gap")
    );
    let type_case_struct_case = active_type_elaboration_cases(&plan)
        .find(|case| case.id.0 == "fail_type_elaboration_imported_type_case_struct_gap_001")
        .expect("Task97 imported TypeCaseStruct boundary should be active");
    assert_eq!(
        type_case_struct_case
            .expectation
            .rejection_reason
            .as_deref(),
        Some("imported_structure_evidence_payload_gap")
    );
    let imported_predicate_functor_case = active_type_elaboration_cases(&plan)
        .find(|case| case.id.0 == "fail_type_elaboration_imported_predicate_functor_gap_001")
        .expect("Task110 imported predicate/functor checker bridge should be active");
    assert_eq!(
        imported_predicate_functor_case
            .expectation
            .rejection_reason
            .as_deref(),
        Some("imported_predicate_functor_signature_payload_gap")
    );
    let builtin_membership_case = active_type_elaboration_cases(&plan)
        .find(|case| case.id.0 == "fail_type_elaboration_builtin_membership_formula_gap_001")
        .expect("Task108 builtin membership checker bridge should be active");
    assert_eq!(
        builtin_membership_case
            .expectation
            .rejection_reason
            .as_deref(),
        Some("numeric_type_payload_extraction_gap")
    );
    let builtin_inequality_case = active_type_elaboration_cases(&plan)
        .find(|case| case.id.0 == "fail_type_elaboration_builtin_inequality_formula_gap_001")
        .expect("Task107 builtin inequality checker bridge should be active");
    assert_eq!(
        builtin_inequality_case
            .expectation
            .rejection_reason
            .as_deref(),
        Some("numeric_type_payload_extraction_gap")
    );
    let builtin_type_assertion_case = active_type_elaboration_cases(&plan)
        .find(|case| case.id.0 == "fail_type_elaboration_builtin_type_assertion_formula_gap_001")
        .expect("Task109 builtin type assertion checker bridge should be active");
    assert_eq!(
        builtin_type_assertion_case
            .expectation
            .rejection_reason
            .as_deref(),
        Some("numeric_type_payload_extraction_gap")
    );
    let imported_attribute_assertion_case = active_type_elaboration_cases(&plan)
        .find(|case| {
            case.id.0 == "fail_type_elaboration_imported_attribute_assertion_formula_gap_001"
        })
        .expect("Task113 imported attribute assertion formula checker bridge should be active");
    assert_eq!(
        imported_attribute_assertion_case
            .expectation
            .rejection_reason
            .as_deref(),
        Some("imported_attribute_assertion_formula_payload_gap")
    );
    let imported_non_empty_attribute_assertion_case = active_type_elaboration_cases(&plan)
        .find(|case| {
            case.id.0
                == "fail_type_elaboration_imported_non_empty_attribute_assertion_formula_gap_001"
        })
        .expect("Task114 imported non-empty attribute assertion checker bridge should be active");
    assert_eq!(
        imported_non_empty_attribute_assertion_case
            .expectation
            .rejection_reason
            .as_deref(),
        Some("imported_non_empty_attribute_assertion_formula_payload_gap")
    );
    let set_enumeration_formula_case = active_type_elaboration_cases(&plan)
        .find(|case| case.id.0 == "fail_type_elaboration_set_enumeration_formula_gap_001")
        .expect("Task111 set-enumeration formula bridge should be active");
    assert_eq!(
        set_enumeration_formula_case
            .expectation
            .rejection_reason
            .as_deref(),
        Some("set_enumeration_result_type_payload_gap")
    );
    let formula_connective_quantifier_case = active_type_elaboration_cases(&plan)
        .find(|case| case.id.0 == "fail_type_elaboration_formula_connective_quantifier_gap_001")
        .expect("Task112 formula connective/quantifier bridge should be active");
    assert_eq!(
        formula_connective_quantifier_case
            .expectation
            .rejection_reason
            .as_deref(),
        Some("formula_connective_quantifier_shell_payload_gap")
    );
    let contradiction_formula_constant_case = active_type_elaboration_cases(&plan)
        .find(|case| case.id.0 == "pass_type_elaboration_contradiction_formula_constant_001")
        .expect("Task180 contradiction formula constant bridge should be active");
    assert_eq!(
        contradiction_formula_constant_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let reserved_variable_equality_case = active_type_elaboration_cases(&plan)
        .find(|case| case.id.0 == "pass_type_elaboration_reserved_variable_equality_001")
        .expect("Task119 reserved-variable equality checker bridge should be active");
    assert_eq!(
        reserved_variable_equality_case.expectation.expected_outcome,
        ExpectedOutcome::Pass
    );
    let distinct_reserved_variable_equality_case = active_type_elaboration_cases(&plan)
        .find(|case| case.id.0 == "pass_type_elaboration_distinct_reserved_variable_equality_001")
        .expect("Task123 distinct reserved-variable equality checker bridge should be active");
    assert_eq!(
        distinct_reserved_variable_equality_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let multiple_reserve_declaration_equality_case = active_type_elaboration_cases(&plan)
        .find(|case| case.id.0 == "pass_type_elaboration_multiple_reserve_declaration_equality_001")
        .expect("Task124 multiple-reserve declaration equality checker bridge should be active");
    assert_eq!(
        multiple_reserve_declaration_equality_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let heterogeneous_reserve_membership_case = active_type_elaboration_cases(&plan)
        .find(|case| case.id.0 == "pass_type_elaboration_heterogeneous_reserve_membership_001")
        .expect("Task125 heterogeneous reserve membership checker bridge should be active");
    assert_eq!(
        heterogeneous_reserve_membership_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let local_mode_reserved_variable_membership_case = active_type_elaboration_cases(&plan)
        .find(|case| {
            case.id.0 == "pass_type_elaboration_local_mode_reserved_variable_membership_001"
        })
        .expect("Task139 local-mode reserved-variable membership bridge should be active");
    assert_eq!(
        local_mode_reserved_variable_membership_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let local_object_mode_reserved_variable_membership_case = active_type_elaboration_cases(&plan)
        .find(|case| {
            case.id.0 == "pass_type_elaboration_local_object_mode_reserved_variable_membership_001"
        })
        .expect("Task140 local-object-mode reserved-variable membership bridge should be active");
    assert_eq!(
        local_object_mode_reserved_variable_membership_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let chained_local_mode_reserved_variable_membership_case = active_type_elaboration_cases(&plan)
        .find(|case| {
            case.id.0 == "pass_type_elaboration_chained_local_mode_reserved_variable_membership_001"
        })
        .expect("Task141 chained local-mode reserved-variable membership bridge should be active");
    assert_eq!(
        chained_local_mode_reserved_variable_membership_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let chained_local_object_mode_reserved_variable_membership_case =
        active_type_elaboration_cases(&plan)
            .find(|case| {
                case.id.0
                    == "pass_type_elaboration_chained_local_object_mode_reserved_variable_membership_001"
            })
            .expect(
                "Task142 chained local-object-mode reserved-variable membership bridge should be active",
            );
    assert_eq!(
        chained_local_object_mode_reserved_variable_membership_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let two_edge_local_mode_reserved_variable_membership_case =
        active_type_elaboration_cases(&plan)
            .find(|case| {
                case.id.0
                    == "pass_type_elaboration_two_edge_local_mode_reserved_variable_membership_001"
            })
            .expect(
                "Task143 two-edge local-mode reserved-variable membership bridge should be active",
            );
    assert_eq!(
        two_edge_local_mode_reserved_variable_membership_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let two_edge_local_object_mode_reserved_variable_membership_case =
        active_type_elaboration_cases(&plan)
            .find(|case| {
                case.id.0
                    == "pass_type_elaboration_two_edge_local_object_mode_reserved_variable_membership_001"
            })
            .expect(
                "Task144 two-edge local-object-mode reserved-variable membership bridge should be active",
            );
    assert_eq!(
        two_edge_local_object_mode_reserved_variable_membership_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let chained_local_mode_reserved_variable_equality_case = active_type_elaboration_cases(&plan)
        .find(|case| {
            case.id.0 == "pass_type_elaboration_chained_local_mode_reserved_variable_equality_001"
        })
        .expect("Task127 chained local-mode reserved-variable equality bridge should be active");
    assert_eq!(
        chained_local_mode_reserved_variable_equality_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let chained_local_mode_reserved_variable_inequality_case = active_type_elaboration_cases(&plan)
        .find(|case| {
            case.id.0 == "pass_type_elaboration_chained_local_mode_reserved_variable_inequality_001"
        })
        .expect("Task132 chained local-mode reserved-variable inequality bridge should be active");
    assert_eq!(
        chained_local_mode_reserved_variable_inequality_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let local_mode_reserved_variable_equality_case = active_type_elaboration_cases(&plan)
        .find(|case| case.id.0 == "pass_type_elaboration_local_mode_reserved_variable_equality_001")
        .expect("Task126 local-mode reserved-variable equality bridge should be active");
    assert_eq!(
        local_mode_reserved_variable_equality_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let local_object_mode_reserved_variable_equality_case = active_type_elaboration_cases(&plan)
        .find(|case| {
            case.id.0 == "pass_type_elaboration_local_object_mode_reserved_variable_equality_001"
        })
        .expect("Task128 local object-mode reserved-variable equality bridge should be active");
    assert_eq!(
        local_object_mode_reserved_variable_equality_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let chained_local_object_mode_reserved_variable_equality_case = active_type_elaboration_cases(
        &plan,
    )
    .find(|case| {
        case.id.0
            == "pass_type_elaboration_chained_local_object_mode_reserved_variable_equality_001"
    })
    .expect("Task129 chained local object-mode equality bridge should be active");
    assert_eq!(
        chained_local_object_mode_reserved_variable_equality_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let local_mode_reserved_variable_inequality_case = active_type_elaboration_cases(&plan)
        .find(|case| {
            case.id.0 == "pass_type_elaboration_local_mode_reserved_variable_inequality_001"
        })
        .expect("Task130 local-mode reserved-variable inequality bridge should be active");
    assert_eq!(
        local_mode_reserved_variable_inequality_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let local_object_mode_reserved_variable_inequality_case = active_type_elaboration_cases(&plan)
        .find(|case| {
            case.id.0 == "pass_type_elaboration_local_object_mode_reserved_variable_inequality_001"
        })
        .expect("Task131 local object-mode reserved-variable inequality bridge should be active");
    assert_eq!(
        local_object_mode_reserved_variable_inequality_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let reserved_variable_membership_case = active_type_elaboration_cases(&plan)
        .find(|case| case.id.0 == "pass_type_elaboration_reserved_variable_membership_001")
        .expect("Task120 reserved-variable membership checker bridge should be active");
    assert_eq!(
        reserved_variable_membership_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let reserved_variable_inequality_case = active_type_elaboration_cases(&plan)
        .find(|case| case.id.0 == "pass_type_elaboration_reserved_variable_inequality_001")
        .expect("Task121 reserved-variable inequality checker bridge should be active");
    assert_eq!(
        reserved_variable_inequality_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let reserved_variable_type_assertion_case = active_type_elaboration_cases(&plan)
        .find(|case| case.id.0 == "pass_type_elaboration_reserved_variable_type_assertion_001")
        .expect("Task122 reserved-variable type assertion checker bridge should be active");
    assert_eq!(
        reserved_variable_type_assertion_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let local_mode_reserved_variable_type_assertion_case = active_type_elaboration_cases(&plan)
        .find(|case| {
            case.id.0 == "pass_type_elaboration_local_mode_reserved_variable_type_assertion_001"
        })
        .expect("Task138 local-mode reserved-variable type assertion bridge should be active");
    assert_eq!(
        local_mode_reserved_variable_type_assertion_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let local_object_mode_reserved_variable_type_assertion_case = active_type_elaboration_cases(
        &plan,
    )
    .find(|case| {
        case.id.0 == "pass_type_elaboration_local_object_mode_reserved_variable_type_assertion_001"
    })
    .expect("Task145 local-object-mode reserved-variable type assertion bridge should be active");
    assert_eq!(
        local_object_mode_reserved_variable_type_assertion_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let chained_local_mode_reserved_variable_type_assertion_case = active_type_elaboration_cases(
        &plan,
    )
    .find(|case| {
        case.id.0 == "pass_type_elaboration_chained_local_mode_reserved_variable_type_assertion_001"
    })
    .expect("Task146 chained local-mode reserved-variable type assertion bridge should be active");
    assert_eq!(
        chained_local_mode_reserved_variable_type_assertion_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let chained_local_object_mode_reserved_variable_type_assertion_case =
        active_type_elaboration_cases(&plan)
            .find(|case| {
                case.id.0
                    == "pass_type_elaboration_chained_local_object_mode_reserved_variable_type_assertion_001"
            })
            .expect(
                "Task147 chained local-object-mode reserved-variable type assertion bridge should be active",
            );
    assert_eq!(
        chained_local_object_mode_reserved_variable_type_assertion_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let two_edge_local_mode_reserved_variable_type_assertion_case = active_type_elaboration_cases(
        &plan,
    )
    .find(|case| {
        case.id.0
            == "pass_type_elaboration_two_edge_local_mode_reserved_variable_type_assertion_001"
    })
    .expect("Task148 two-edge local-mode reserved-variable type assertion bridge should be active");
    assert_eq!(
        two_edge_local_mode_reserved_variable_type_assertion_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let two_edge_local_object_mode_reserved_variable_type_assertion_case =
        active_type_elaboration_cases(&plan)
            .find(|case| {
                case.id.0
                    == "pass_type_elaboration_two_edge_local_object_mode_reserved_variable_type_assertion_001"
            })
            .expect(
                "Task149 two-edge local-object-mode reserved-variable type assertion bridge should be active",
            );
    assert_eq!(
        two_edge_local_object_mode_reserved_variable_type_assertion_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let three_edge_local_mode_reserved_variable_type_assertion_case =
        active_type_elaboration_cases(&plan)
            .find(|case| {
                case.id.0
                    == "pass_type_elaboration_three_edge_local_mode_reserved_variable_type_assertion_001"
            })
            .expect(
                "Task150 three-edge local-mode reserved-variable type assertion bridge should be active",
            );
    assert_eq!(
        three_edge_local_mode_reserved_variable_type_assertion_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let three_edge_local_object_mode_reserved_variable_type_assertion_case =
        active_type_elaboration_cases(&plan)
            .find(|case| {
                case.id.0
                    == "pass_type_elaboration_three_edge_local_object_mode_reserved_variable_type_assertion_001"
            })
            .expect(
                "Task151 three-edge local-object-mode reserved-variable type assertion bridge should be active",
            );
    assert_eq!(
        three_edge_local_object_mode_reserved_variable_type_assertion_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let four_edge_local_mode_reserved_variable_type_assertion_case = active_type_elaboration_cases(
        &plan,
    )
    .find(|case| {
        case.id.0
            == "pass_type_elaboration_four_edge_local_mode_reserved_variable_type_assertion_001"
    })
    .expect(
        "Task152 four-edge local-mode reserved-variable type assertion bridge should be active",
    );
    assert_eq!(
        four_edge_local_mode_reserved_variable_type_assertion_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let four_edge_local_object_mode_reserved_variable_type_assertion_case =
        active_type_elaboration_cases(&plan)
            .find(|case| {
                case.id.0
                    == "pass_type_elaboration_four_edge_local_object_mode_reserved_variable_type_assertion_001"
            })
            .expect(
                "Task153 four-edge local-object-mode reserved-variable type assertion bridge should be active",
            );
    assert_eq!(
        four_edge_local_object_mode_reserved_variable_type_assertion_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let three_edge_local_mode_reserved_variable_equality_case =
        active_type_elaboration_cases(&plan)
            .find(|case| {
                case.id.0
                    == "pass_type_elaboration_three_edge_local_mode_reserved_variable_equality_001"
            })
            .expect(
                "Task154 three-edge local-mode reserved-variable equality bridge should be active",
            );
    assert_eq!(
        three_edge_local_mode_reserved_variable_equality_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let three_edge_local_object_mode_reserved_variable_equality_case =
        active_type_elaboration_cases(&plan)
            .find(|case| {
                case.id.0
                    == "pass_type_elaboration_three_edge_local_object_mode_reserved_variable_equality_001"
            })
            .expect(
                "Task155 three-edge local-object-mode reserved-variable equality bridge should be active",
            );
    assert_eq!(
        three_edge_local_object_mode_reserved_variable_equality_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let three_edge_local_mode_reserved_variable_inequality_case = active_type_elaboration_cases(
        &plan,
    )
    .find(|case| {
        case.id.0 == "pass_type_elaboration_three_edge_local_mode_reserved_variable_inequality_001"
    })
    .expect("Task156 three-edge local-mode reserved-variable inequality bridge should be active");
    assert_eq!(
        three_edge_local_mode_reserved_variable_inequality_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let three_edge_local_object_mode_reserved_variable_inequality_case =
        active_type_elaboration_cases(&plan)
            .find(|case| {
                case.id.0
                    == "pass_type_elaboration_three_edge_local_object_mode_reserved_variable_inequality_001"
            })
            .expect(
                "Task157 three-edge local-object-mode reserved-variable inequality bridge should be active",
            );
    assert_eq!(
        three_edge_local_object_mode_reserved_variable_inequality_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let three_edge_local_mode_reserved_variable_membership_case = active_type_elaboration_cases(
        &plan,
    )
    .find(|case| {
        case.id.0 == "pass_type_elaboration_three_edge_local_mode_reserved_variable_membership_001"
    })
    .expect("Task158 three-edge local-mode reserved-variable membership bridge should be active");
    assert_eq!(
        three_edge_local_mode_reserved_variable_membership_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let three_edge_local_object_mode_reserved_variable_membership_case =
        active_type_elaboration_cases(&plan)
            .find(|case| {
                case.id.0
                    == "pass_type_elaboration_three_edge_local_object_mode_reserved_variable_membership_001"
            })
            .expect(
                "Task163 three-edge local-object-mode reserved-variable membership bridge should be active",
            );
    assert_eq!(
        three_edge_local_object_mode_reserved_variable_membership_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let four_edge_local_mode_reserved_variable_membership_case =
        active_type_elaboration_cases(&plan)
            .find(|case| {
                case.id.0
                    == "pass_type_elaboration_four_edge_local_mode_reserved_variable_membership_001"
            })
            .expect(
                "Task164 four-edge local-mode reserved-variable membership bridge should be active",
            );
    assert_eq!(
        four_edge_local_mode_reserved_variable_membership_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let four_edge_local_object_mode_reserved_variable_membership_case =
        active_type_elaboration_cases(&plan)
            .find(|case| {
                case.id.0
                    == "pass_type_elaboration_four_edge_local_object_mode_reserved_variable_membership_001"
            })
            .expect(
                "Task165 four-edge local-object-mode reserved-variable membership bridge should be active",
            );
    assert_eq!(
        four_edge_local_object_mode_reserved_variable_membership_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let four_edge_local_mode_reserved_variable_equality_case = active_type_elaboration_cases(&plan)
        .find(|case| {
            case.id.0 == "pass_type_elaboration_four_edge_local_mode_reserved_variable_equality_001"
        })
        .expect("Task166 four-edge local-mode reserved-variable equality bridge should be active");
    assert_eq!(
        four_edge_local_mode_reserved_variable_equality_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let four_edge_local_mode_reserved_variable_inequality_case =
        active_type_elaboration_cases(&plan)
            .find(|case| {
                case.id.0
                    == "pass_type_elaboration_four_edge_local_mode_reserved_variable_inequality_001"
            })
            .expect(
                "Task168 four-edge local-mode reserved-variable inequality bridge should be active",
            );
    assert_eq!(
        four_edge_local_mode_reserved_variable_inequality_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let four_edge_local_object_mode_reserved_variable_equality_case =
        active_type_elaboration_cases(&plan)
            .find(|case| {
                case.id.0
                    == "pass_type_elaboration_four_edge_local_object_mode_reserved_variable_equality_001"
            })
            .expect(
                "Task167 four-edge local-object-mode reserved-variable equality bridge should be active",
            );
    assert_eq!(
        four_edge_local_object_mode_reserved_variable_equality_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let four_edge_local_object_mode_reserved_variable_inequality_case =
        active_type_elaboration_cases(&plan)
            .find(|case| {
                case.id.0
                    == "pass_type_elaboration_four_edge_local_object_mode_reserved_variable_inequality_001"
            })
            .expect(
                "Task169 four-edge local-object-mode reserved-variable inequality bridge should be active",
            );
    assert_eq!(
        four_edge_local_object_mode_reserved_variable_inequality_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let local_mode_long_chain_reserved_variable_equality_case =
        active_type_elaboration_cases(&plan)
            .find(|case| {
                case.id.0
                    == "pass_type_elaboration_local_mode_long_chain_reserved_variable_equality_001"
            })
            .expect("Task172 local-mode long-chain equality bridge should be active");
    assert_eq!(
        local_mode_long_chain_reserved_variable_equality_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let local_mode_long_chain_reserved_variable_inequality_case = active_type_elaboration_cases(
        &plan,
    )
    .find(|case| {
        case.id.0 == "pass_type_elaboration_local_mode_long_chain_reserved_variable_inequality_001"
    })
    .expect("Task173 local-mode long-chain inequality bridge should be active");
    assert_eq!(
        local_mode_long_chain_reserved_variable_inequality_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let local_mode_long_chain_reserved_variable_membership_case = active_type_elaboration_cases(
        &plan,
    )
    .find(|case| {
        case.id.0 == "pass_type_elaboration_local_mode_long_chain_reserved_variable_membership_001"
    })
    .expect("Task174 local-mode long-chain membership bridge should be active");
    assert_eq!(
        local_mode_long_chain_reserved_variable_membership_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let local_mode_long_chain_reserved_variable_type_assertion_case =
        active_type_elaboration_cases(&plan)
            .find(|case| {
                case.id.0
                    == "pass_type_elaboration_local_mode_long_chain_reserved_variable_type_assertion_001"
            })
            .expect("Task175 local-mode long-chain type-assertion bridge should be active");
    assert_eq!(
        local_mode_long_chain_reserved_variable_type_assertion_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let local_mode_long_chain_asserted_head_case = active_type_elaboration_cases(&plan)
        .find(|case| case.id.0 == "pass_type_elaboration_local_mode_long_chain_asserted_head_001")
        .expect("Task199 local-mode long-chain asserted-head bridge should be active");
    assert_eq!(
        local_mode_long_chain_asserted_head_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let local_object_mode_long_chain_asserted_head_case = active_type_elaboration_cases(&plan)
        .find(|case| {
            case.id.0 == "pass_type_elaboration_local_object_mode_long_chain_asserted_head_001"
        })
        .expect("Task200 local-object-mode long-chain asserted-head bridge should be active");
    assert_eq!(
        local_object_mode_long_chain_asserted_head_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let local_object_mode_long_chain_reserved_variable_equality_case =
        active_type_elaboration_cases(&plan)
            .find(|case| {
                case.id.0
                    == "pass_type_elaboration_local_object_mode_long_chain_reserved_variable_equality_001"
            })
            .expect("Task176 local-object-mode long-chain equality bridge should be active");
    assert_eq!(
        local_object_mode_long_chain_reserved_variable_equality_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let local_object_mode_long_chain_reserved_variable_inequality_case =
        active_type_elaboration_cases(&plan)
            .find(|case| {
                case.id.0
                    == "pass_type_elaboration_local_object_mode_long_chain_reserved_variable_inequality_001"
            })
            .expect("Task177 local-object-mode long-chain inequality bridge should be active");
    assert_eq!(
        local_object_mode_long_chain_reserved_variable_inequality_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let local_object_mode_long_chain_reserved_variable_membership_case =
        active_type_elaboration_cases(&plan)
            .find(|case| {
                case.id.0
                    == "pass_type_elaboration_local_object_mode_long_chain_reserved_variable_membership_001"
            })
            .expect("Task178 local-object-mode long-chain membership bridge should be active");
    assert_eq!(
        local_object_mode_long_chain_reserved_variable_membership_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let local_object_mode_long_chain_reserved_variable_type_assertion_case =
        active_type_elaboration_cases(&plan)
            .find(|case| {
                case.id.0
                    == "pass_type_elaboration_local_object_mode_long_chain_reserved_variable_type_assertion_001"
            })
            .expect("Task179 local-object-mode long-chain type assertion bridge should be active");
    assert_eq!(
        local_object_mode_long_chain_reserved_variable_type_assertion_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let local_mode_asserted_head_case = active_type_elaboration_cases(&plan)
        .find(|case| case.id.0 == "pass_type_elaboration_local_mode_asserted_head_001")
        .expect("Task182 formula-side local-mode asserted-head bridge should be active");
    assert_eq!(
        local_mode_asserted_head_case.expectation.expected_outcome,
        ExpectedOutcome::Pass
    );
    let local_object_mode_asserted_head_case = active_type_elaboration_cases(&plan)
        .find(|case| case.id.0 == "pass_type_elaboration_local_object_mode_asserted_head_001")
        .expect("Task183 object-terminal formula-side asserted-head bridge should be active");
    assert_eq!(
        local_object_mode_asserted_head_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let chained_local_mode_asserted_head_case = active_type_elaboration_cases(&plan)
        .find(|case| case.id.0 == "pass_type_elaboration_chained_local_mode_asserted_head_001")
        .expect("Task184 one-edge formula-side local-mode asserted-head bridge should be active");
    assert_eq!(
        chained_local_mode_asserted_head_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let chained_local_mode_radix_asserted_head_case = active_type_elaboration_cases(&plan)
        .find(|case| {
            case.id.0
                == "pass_type_elaboration_chained_local_mode_radix_asserted_head_001"
        })
        .expect(
            "Task201 one-edge immediate-radix formula-side local-mode asserted-head bridge should be active",
        );
    assert_eq!(
        chained_local_mode_radix_asserted_head_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let chained_local_object_mode_radix_asserted_head_case =
        active_type_elaboration_cases(&plan)
            .find(|case| {
                case.id.0
                    == "pass_type_elaboration_chained_local_object_mode_radix_asserted_head_001"
            })
            .expect(
                "Task202 one-edge object-terminal immediate-radix asserted-head bridge should be active",
            );
    assert_eq!(
        chained_local_object_mode_radix_asserted_head_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let two_edge_local_mode_radix_asserted_head_case = active_type_elaboration_cases(&plan)
        .find(|case| {
            case.id.0 == "pass_type_elaboration_two_edge_local_mode_radix_asserted_head_001"
        })
        .expect(
            "Task203 two-edge set-terminal immediate-radix asserted-head bridge should be active",
        );
    assert_eq!(
        two_edge_local_mode_radix_asserted_head_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let two_edge_local_object_mode_radix_asserted_head_case =
        active_type_elaboration_cases(&plan)
            .find(|case| {
                case.id.0
                    == "pass_type_elaboration_two_edge_local_object_mode_radix_asserted_head_001"
            })
            .expect(
                "Task204 two-edge object-terminal immediate-radix asserted-head bridge should be active",
            );
    assert_eq!(
        two_edge_local_object_mode_radix_asserted_head_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let three_edge_local_mode_radix_asserted_head_case = active_type_elaboration_cases(&plan)
        .find(|case| {
            case.id.0 == "pass_type_elaboration_three_edge_local_mode_radix_asserted_head_001"
        })
        .expect(
            "Task205 three-edge set-terminal immediate-radix asserted-head bridge should be active",
        );
    assert_eq!(
        three_edge_local_mode_radix_asserted_head_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let three_edge_local_object_mode_radix_asserted_head_case = active_type_elaboration_cases(
        &plan,
    )
    .find(|case| {
        case.id.0 == "pass_type_elaboration_three_edge_local_object_mode_radix_asserted_head_001"
    })
    .expect(
        "Task206 three-edge object-terminal immediate-radix asserted-head bridge should be active",
    );
    assert_eq!(
        three_edge_local_object_mode_radix_asserted_head_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let chained_local_object_mode_asserted_head_case = active_type_elaboration_cases(&plan)
        .find(|case| {
            case.id.0 == "pass_type_elaboration_chained_local_object_mode_asserted_head_001"
        })
        .expect(
            "Task185 one-edge object-terminal formula-side local-mode asserted-head bridge should be active",
        );
    assert_eq!(
        chained_local_object_mode_asserted_head_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let two_edge_local_mode_asserted_head_case = active_type_elaboration_cases(&plan)
        .find(|case| case.id.0 == "pass_type_elaboration_two_edge_local_mode_asserted_head_001")
        .expect("Task186 two-edge formula-side local-mode asserted-head bridge should be active");
    assert_eq!(
        two_edge_local_mode_asserted_head_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let three_edge_local_mode_asserted_head_case = active_type_elaboration_cases(&plan)
        .find(|case| case.id.0 == "pass_type_elaboration_three_edge_local_mode_asserted_head_001")
        .expect("Task195 three-edge formula-side local-mode asserted-head bridge should be active");
    assert_eq!(
        three_edge_local_mode_asserted_head_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let four_edge_local_mode_asserted_head_case = active_type_elaboration_cases(&plan)
        .find(|case| case.id.0 == "pass_type_elaboration_four_edge_local_mode_asserted_head_001")
        .expect("Task197 four-edge formula-side local-mode asserted-head bridge should be active");
    assert_eq!(
        four_edge_local_mode_asserted_head_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let four_edge_local_mode_four_hop_asserted_head_case = active_type_elaboration_cases(&plan)
        .find(|case| {
            case.id.0 == "pass_type_elaboration_four_edge_local_mode_four_hop_asserted_head_001"
        })
        .expect("Task221 four-edge set-terminal four-hop asserted-head bridge should be active");
    assert_eq!(
        four_edge_local_mode_four_hop_asserted_head_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let four_edge_local_object_mode_four_hop_asserted_head_case = active_type_elaboration_cases(
        &plan,
    )
    .find(|case| {
        case.id.0 == "pass_type_elaboration_four_edge_local_object_mode_four_hop_asserted_head_001"
    })
    .expect("Task222 four-edge object-terminal four-hop asserted-head bridge should be active");
    assert_eq!(
        four_edge_local_object_mode_four_hop_asserted_head_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let parenthesized_reserved_variable_equality_case = active_type_elaboration_cases(&plan)
        .find(|case| {
            case.id.0 == "pass_type_elaboration_parenthesized_reserved_variable_equality_001"
        })
        .expect("Task223 parenthesized reserved-variable equality bridge should be active");
    assert_eq!(
        parenthesized_reserved_variable_equality_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let local_mode_long_chain_two_hop_asserted_head_case = active_type_elaboration_cases(&plan)
        .find(|case| {
            case.id.0 == "pass_type_elaboration_local_mode_long_chain_two_hop_asserted_head_001"
        })
        .expect(
            "Task224 seven-expansion set-terminal two-hop asserted-head bridge should be active",
        );
    assert_eq!(
        local_mode_long_chain_two_hop_asserted_head_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let four_edge_local_mode_three_hop_asserted_head_case = active_type_elaboration_cases(&plan)
        .find(|case| {
            case.id.0 == "pass_type_elaboration_four_edge_local_mode_three_hop_asserted_head_001"
        })
        .expect("Task219 four-edge set-terminal three-hop asserted-head bridge should be active");
    assert_eq!(
        four_edge_local_mode_three_hop_asserted_head_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let four_edge_local_mode_radix_asserted_head_case = active_type_elaboration_cases(&plan)
        .find(|case| {
            case.id.0 == "pass_type_elaboration_four_edge_local_mode_radix_asserted_head_001"
        })
        .expect(
            "Task207 four-edge set-terminal immediate-radix asserted-head bridge should be active",
        );
    assert_eq!(
        four_edge_local_mode_radix_asserted_head_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let four_edge_local_object_mode_radix_asserted_head_case =
        active_type_elaboration_cases(&plan)
            .find(|case| {
                case.id.0
                    == "pass_type_elaboration_four_edge_local_object_mode_radix_asserted_head_001"
            })
            .expect(
                "Task208 four-edge object-terminal immediate-radix asserted-head bridge should be active",
            );
    assert_eq!(
        four_edge_local_object_mode_radix_asserted_head_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let four_edge_local_object_mode_asserted_head_case = active_type_elaboration_cases(&plan)
        .find(|case| {
            case.id.0
                == "pass_type_elaboration_four_edge_local_object_mode_asserted_head_001"
        })
        .expect(
            "Task198 four-edge object-terminal formula-side local-mode asserted-head bridge should be active",
        );
    assert_eq!(
        four_edge_local_object_mode_asserted_head_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let four_edge_local_object_mode_three_hop_asserted_head_case = active_type_elaboration_cases(
        &plan,
    )
    .find(|case| {
        case.id.0 == "pass_type_elaboration_four_edge_local_object_mode_three_hop_asserted_head_001"
    })
    .expect("Task220 four-edge object-terminal three-hop asserted-head bridge should be active");
    assert_eq!(
        four_edge_local_object_mode_three_hop_asserted_head_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let three_edge_local_object_mode_asserted_head_case = active_type_elaboration_cases(&plan)
        .find(|case| {
            case.id.0
                == "pass_type_elaboration_three_edge_local_object_mode_asserted_head_001"
        })
        .expect(
            "Task196 three-edge object-terminal formula-side local-mode asserted-head bridge should be active",
        );
    assert_eq!(
        three_edge_local_object_mode_asserted_head_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let two_edge_local_object_mode_asserted_head_case = active_type_elaboration_cases(&plan)
        .find(|case| {
            case.id.0
                == "pass_type_elaboration_two_edge_local_object_mode_asserted_head_001"
        })
        .expect(
            "Task187 two-edge object-terminal formula-side local-mode asserted-head bridge should be active",
        );
    assert_eq!(
        two_edge_local_object_mode_asserted_head_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let reserved_object_variable_equality_case = active_type_elaboration_cases(&plan)
        .find(|case| case.id.0 == "pass_type_elaboration_reserved_object_variable_equality_001")
        .expect("Task188 builtin-object reserved-variable equality bridge should be active");
    assert_eq!(
        reserved_object_variable_equality_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let distinct_reserved_object_variable_equality_case = active_type_elaboration_cases(&plan)
        .find(|case| {
            case.id.0 == "pass_type_elaboration_distinct_reserved_object_variable_equality_001"
        })
        .expect("Task191 distinct reserved-object-variable equality bridge should be active");
    assert_eq!(
        distinct_reserved_object_variable_equality_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let distinct_reserved_object_variable_inequality_case = active_type_elaboration_cases(&plan)
        .find(|case| {
            case.id.0 == "pass_type_elaboration_distinct_reserved_object_variable_inequality_001"
        })
        .expect("Task192 distinct reserved-object-variable inequality bridge should be active");
    assert_eq!(
        distinct_reserved_object_variable_inequality_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let distinct_reserved_variable_membership_case = active_type_elaboration_cases(&plan)
        .find(|case| case.id.0 == "pass_type_elaboration_distinct_reserved_variable_membership_001")
        .expect("Task159 distinct reserved-variable membership bridge should be active");
    assert_eq!(
        distinct_reserved_variable_membership_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let distinct_reserved_variable_inequality_case = active_type_elaboration_cases(&plan)
        .find(|case| case.id.0 == "pass_type_elaboration_distinct_reserved_variable_inequality_001")
        .expect("Task160 distinct reserved-variable inequality bridge should be active");
    assert_eq!(
        distinct_reserved_variable_inequality_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let multiple_reserve_declaration_inequality_case = active_type_elaboration_cases(&plan)
        .find(|case| {
            case.id.0 == "pass_type_elaboration_multiple_reserve_declaration_inequality_001"
        })
        .expect("Task161 multiple-reserve-declaration inequality bridge should be active");
    assert_eq!(
        multiple_reserve_declaration_inequality_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let multiple_object_reserve_declaration_equality_case = active_type_elaboration_cases(&plan)
        .find(|case| {
            case.id.0 == "pass_type_elaboration_multiple_object_reserve_declaration_equality_001"
        })
        .expect("Task193 multiple-object-reserve-declaration equality bridge should be active");
    assert_eq!(
        multiple_object_reserve_declaration_equality_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let multiple_object_reserve_declaration_inequality_case = active_type_elaboration_cases(&plan)
        .find(|case| {
            case.id.0 == "pass_type_elaboration_multiple_object_reserve_declaration_inequality_001"
        })
        .expect("Task194 multiple-object-reserve-declaration inequality bridge should be active");
    assert_eq!(
        multiple_object_reserve_declaration_inequality_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );
    let multiple_reserve_declaration_membership_case = active_type_elaboration_cases(&plan)
        .find(|case| {
            case.id.0 == "pass_type_elaboration_multiple_reserve_declaration_membership_001"
        })
        .expect("Task162 multiple-reserve-declaration membership bridge should be active");
    assert_eq!(
        multiple_reserve_declaration_membership_case
            .expectation
            .expected_outcome,
        ExpectedOutcome::Pass
    );

    let report = run_type_elaboration_corpus(&config).unwrap();

    assert_eq!(report.error_count(), 0, "{:#?}", report.diagnostics);
    assert_eq!(report.results.len(), 172);
    assert_eq!(report.passed_count(), 172);
    assert_eq!(report.failed_count(), 0);
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_type_elaboration_non_builtin_type_gap_001"
            && result.actual_detail_keys
                == ["type_elaboration.external_dependency.ast_payload_extraction"]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_type_elaboration_attributed_reserve_gap_001"
            && result.actual_detail_keys
                == ["type_elaboration.checker.checker.declaration.deferred.evidence_query"]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_type_elaboration_mixed_reserve_gap_001"
            && result.actual_detail_keys
                == ["type_elaboration.checker.checker.declaration.deferred.evidence_query"]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_type_elaboration_structure_qualified_attribute_gap_001"
            && result.actual_detail_keys
                == ["type_elaboration.external_dependency.ast_payload_extraction"]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_type_elaboration_argument_bearing_mode_gap_001"
            && result.actual_detail_keys
                == ["type_elaboration.external_dependency.ast_payload_extraction"]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_type_elaboration_argument_bearing_attribute_gap_001"
            && result.actual_detail_keys
                == ["type_elaboration.external_dependency.ast_payload_extraction"]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_type_elaboration_attribute_definition_gap_001"
            && result.actual_detail_keys
                == ["type_elaboration.external_dependency.ast_payload_extraction"]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_type_elaboration_mode_structure_definition_gap_001"
            && result.actual_detail_keys
                == ["type_elaboration.external_dependency.ast_payload_extraction"]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_type_elaboration_argument_bearing_structure_gap_001"
            && result.actual_detail_keys
                == ["type_elaboration.external_dependency.ast_payload_extraction"]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_type_elaboration_imported_attribute_gap_001"
            && result.actual_detail_keys
                == ["type_elaboration.checker.checker.declaration.deferred.evidence_query"]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_type_elaboration_imported_empty_positive_gap_001"
            && result.actual_detail_keys
                == ["type_elaboration.checker.checker.declaration.deferred.evidence_query"]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_type_elaboration_imported_empty_object_gap_001"
            && result.actual_detail_keys
                == ["type_elaboration.checker.checker.declaration.deferred.evidence_query"]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_type_elaboration_imported_structure_gap_001"
            && result.actual_detail_keys
                == ["type_elaboration.checker.checker.declaration.deferred.evidence_query"]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_type_elaboration_imported_type_case_struct_gap_001"
            && result.actual_detail_keys
                == ["type_elaboration.checker.checker.declaration.deferred.evidence_query"]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_type_elaboration_imported_mode_gap_001"
            && result.actual_detail_keys
                == [
                    "type_elaboration.checker.checker.type.external.mode_expansion_payload",
                    "type_elaboration.checker.checker.type.recovery",
                ]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_type_elaboration_bracket_mode_argument_gap_001"
            && result.actual_detail_keys
                == ["type_elaboration.external_dependency.ast_payload_extraction"]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_type_elaboration_bracket_structure_argument_gap_001"
            && result.actual_detail_keys
                == ["type_elaboration.external_dependency.ast_payload_extraction"]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_type_elaboration_payload_extraction_gap_001"
            && result.actual_detail_keys
                == ["type_elaboration.external_dependency.ast_payload_extraction"]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_type_elaboration_formula_statement_gap_001"
            && result.actual_detail_keys
                == ["type_elaboration.checker.checker.formula.external.formula_payload"]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_type_elaboration_term_formula_gap_001"
            && result.actual_detail_keys
                == [
                    "type_elaboration.checker.checker.formula.term.partial",
                    "type_elaboration.checker.checker.term.external.numeric_type_payload",
                ]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_type_elaboration_imported_predicate_functor_gap_001"
            && result.actual_detail_keys
                == [
                    "type_elaboration.checker.checker.formula.external.predicate_signature_payload",
                    "type_elaboration.checker.checker.formula.term.partial",
                    "type_elaboration.checker.checker.term.external.numeric_type_payload",
                    "type_elaboration.checker.checker.term.external.signature_payload",
                ]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_type_elaboration_builtin_membership_formula_gap_001"
            && result.actual_detail_keys
                == [
                    "type_elaboration.checker.checker.formula.term.partial",
                    "type_elaboration.checker.checker.term.external.numeric_type_payload",
                ]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_type_elaboration_builtin_inequality_formula_gap_001"
            && result.actual_detail_keys
                == [
                    "type_elaboration.checker.checker.formula.term.partial",
                    "type_elaboration.checker.checker.term.external.numeric_type_payload",
                ]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_type_elaboration_builtin_type_assertion_formula_gap_001"
            && result.actual_detail_keys
                == [
                    "type_elaboration.checker.checker.formula.term.partial",
                    "type_elaboration.checker.checker.term.external.numeric_type_payload",
                ]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_type_elaboration_imported_attribute_assertion_formula_gap_001"
            && result.actual_detail_keys
                == [
                    "type_elaboration.checker.checker.formula.external.formula_payload",
                    "type_elaboration.checker.checker.formula.term.partial",
                    "type_elaboration.checker.checker.term.external.numeric_type_payload",
                ]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0
            == "fail_type_elaboration_imported_non_empty_attribute_assertion_formula_gap_001"
            && result.actual_detail_keys
                == [
                    "type_elaboration.checker.checker.formula.external.formula_payload",
                    "type_elaboration.checker.checker.formula.term.partial",
                    "type_elaboration.checker.checker.term.external.numeric_type_payload",
                ]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_type_elaboration_set_enumeration_formula_gap_001"
            && result.actual_detail_keys
                == [
                    "type_elaboration.checker.checker.formula.term.partial",
                    "type_elaboration.checker.checker.term.external.numeric_type_payload",
                    "type_elaboration.checker.checker.term.external.result_type_payload",
                ]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_type_elaboration_formula_connective_quantifier_gap_001"
            && result.actual_detail_keys
                == [
                    "type_elaboration.checker.checker.formula.external.formula_payload",
                    "type_elaboration.checker.checker.formula.external.quantifier_payload",
                ]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_type_elaboration_proof_skeleton_gap_001"
            && result.actual_detail_keys
                == ["type_elaboration.external_dependency.ast_payload_extraction"]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_type_elaboration_statement_proof_gap_001"
            && result.actual_detail_keys
                == ["type_elaboration.external_dependency.ast_payload_extraction"]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_type_elaboration_proof_local_declaration_gap_001"
            && result.actual_detail_keys
                == ["type_elaboration.external_dependency.ast_payload_extraction"]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_type_elaboration_inline_definition_gap_001"
            && result.actual_detail_keys
                == ["type_elaboration.external_dependency.ast_payload_extraction"]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_type_elaboration_registration_block_gap_001"
            && result.actual_detail_keys
                == ["type_elaboration.external_dependency.ast_payload_extraction"]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_type_elaboration_redefinition_notation_gap_001"
            && result.actual_detail_keys
                == ["type_elaboration.external_dependency.ast_payload_extraction"]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_type_elaboration_predicate_functor_definition_gap_001"
            && result.actual_detail_keys
                == ["type_elaboration.external_dependency.ast_payload_extraction"]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_type_elaboration_local_mode_forward_reference_gap_001"
            && result.actual_detail_keys
                == ["type_elaboration.lower_stage.frontend:malformed_type_expression"]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_type_elaboration_local_structure_forward_reference_gap_001"
            && result.actual_detail_keys
                == ["type_elaboration.lower_stage.frontend:malformed_type_expression"]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_type_elaboration_local_attribute_forward_reference_gap_001"
            && result.actual_detail_keys
                == ["type_elaboration.lower_stage.frontend:malformed_type_expression"]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_type_elaboration_local_attributed_reserve_evidence_gap_001"
            && result.actual_detail_keys
                == ["type_elaboration.checker.checker.declaration.deferred.evidence_query"]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_type_elaboration_local_mode_attributed_rhs_expansion_gap_001"
            && result.actual_detail_keys
                == ["type_elaboration.checker.checker.declaration.deferred.evidence_query"]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_type_elaboration_attributed_local_mode_attributed_rhs_evidence_gap_001"
            && result.actual_detail_keys
                == ["type_elaboration.checker.checker.declaration.deferred.evidence_query"]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0
            == "fail_type_elaboration_mixed_attributed_local_mode_attributed_rhs_expansion_gap_001"
            && result.actual_detail_keys
                == [
                    "type_elaboration.checker.checker.type.external.mode_expansion_payload",
                    "type_elaboration.checker.checker.type.recovery",
                ]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0
            == "fail_type_elaboration_attributed_local_mode_attributed_rhs_chain_evidence_gap_001"
            && result.actual_detail_keys
                == ["type_elaboration.checker.checker.declaration.deferred.evidence_query"]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_type_elaboration_local_mode_mixed_attributed_reserve_expansion_gap_001"
            && result.actual_detail_keys
                == [
                    "type_elaboration.checker.checker.type.external.mode_expansion_payload",
                    "type_elaboration.checker.checker.type.recovery",
                ]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0
            == "fail_type_elaboration_local_mode_chain_dependency_attributed_expansion_gap_001"
            && result.actual_detail_keys
                == [
                    "type_elaboration.checker.checker.type.external.mode_expansion_payload",
                    "type_elaboration.checker.checker.type.recovery",
                ]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0
            == "fail_type_elaboration_attributed_local_mode_structure_rhs_chain_evidence_gap_001"
            && result.actual_detail_keys
                == ["type_elaboration.checker.checker.declaration.deferred.evidence_query"]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_type_elaboration_local_attributed_mode_reserve_expansion_gap_001"
            && result.actual_detail_keys
                == ["type_elaboration.checker.checker.declaration.deferred.evidence_query"]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_type_elaboration_attributed_local_mode_structure_rhs_evidence_gap_001"
            && result.actual_detail_keys
                == ["type_elaboration.checker.checker.declaration.deferred.evidence_query"]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0
            == "fail_type_elaboration_mixed_attributed_local_mode_structure_rhs_expansion_gap_001"
            && result.actual_detail_keys
                == [
                    "type_elaboration.checker.checker.type.external.mode_expansion_payload",
                    "type_elaboration.checker.checker.type.recovery",
                ]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_type_elaboration_local_mode_structure_rhs_evidence_gap_001"
            && result.actual_detail_keys
                == ["type_elaboration.checker.checker.declaration.deferred.evidence_query"]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_type_elaboration_local_mode_structure_rhs_chain_evidence_gap_001"
            && result.actual_detail_keys
                == ["type_elaboration.checker.checker.declaration.deferred.evidence_query"]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_type_elaboration_local_mode_attributed_rhs_chain_evidence_gap_001"
            && result.actual_detail_keys
                == ["type_elaboration.checker.checker.declaration.deferred.evidence_query"]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_type_elaboration_attributed_local_mode_chain_evidence_gap_001"
            && result.actual_detail_keys
                == ["type_elaboration.checker.checker.declaration.deferred.evidence_query"]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_type_elaboration_local_structure_reserve_evidence_gap_001"
            && result.actual_detail_keys
                == ["type_elaboration.checker.checker.declaration.deferred.evidence_query"]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "fail_type_elaboration_local_attributed_structure_reserve_evidence_gap_001"
            && result.actual_detail_keys
                == ["type_elaboration.checker.checker.declaration.deferred.evidence_query"]
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_type_elaboration_builtin_type_expression_001"
            && result.actual_detail_keys.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_type_elaboration_multi_reserve_builtin_001"
            && result.actual_detail_keys.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_type_elaboration_reserved_variable_equality_001"
            && result.actual_detail_keys.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_type_elaboration_distinct_reserved_variable_equality_001"
            && result.actual_detail_keys.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_type_elaboration_distinct_reserved_variable_inequality_001"
            && result.actual_detail_keys.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_type_elaboration_multiple_reserve_declaration_equality_001"
            && result.actual_detail_keys.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_type_elaboration_multiple_reserve_declaration_inequality_001"
            && result.actual_detail_keys.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_type_elaboration_heterogeneous_reserve_membership_001"
            && result.actual_detail_keys.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_type_elaboration_local_mode_reserved_variable_membership_001"
            && result.actual_detail_keys.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_type_elaboration_local_object_mode_reserved_variable_membership_001"
            && result.actual_detail_keys.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_type_elaboration_chained_local_mode_reserved_variable_membership_001"
            && result.actual_detail_keys.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0
            == "pass_type_elaboration_chained_local_object_mode_reserved_variable_membership_001"
            && result.actual_detail_keys.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_type_elaboration_chained_local_mode_reserved_variable_equality_001"
            && result.actual_detail_keys.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_type_elaboration_chained_local_mode_reserved_variable_inequality_001"
            && result.actual_detail_keys.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_type_elaboration_local_mode_reserved_variable_equality_001"
            && result.actual_detail_keys.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_type_elaboration_local_object_mode_reserved_variable_equality_001"
            && result.actual_detail_keys.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0
            == "pass_type_elaboration_chained_local_object_mode_reserved_variable_equality_001"
            && result.actual_detail_keys.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_type_elaboration_local_mode_reserved_variable_inequality_001"
            && result.actual_detail_keys.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_type_elaboration_local_object_mode_reserved_variable_inequality_001"
            && result.actual_detail_keys.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_type_elaboration_reserved_variable_membership_001"
            && result.actual_detail_keys.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_type_elaboration_reserved_variable_inequality_001"
            && result.actual_detail_keys.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_type_elaboration_reserved_variable_type_assertion_001"
            && result.actual_detail_keys.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_type_elaboration_local_mode_expansion_001"
            && result.actual_detail_keys.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_type_elaboration_local_object_mode_expansion_001"
            && result.actual_detail_keys.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_type_elaboration_local_mode_chain_expansion_001"
            && result.actual_detail_keys.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_type_elaboration_local_object_mode_chain_expansion_001"
            && result.actual_detail_keys.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_type_elaboration_local_mode_two_edge_chain_expansion_001"
            && result.actual_detail_keys.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_type_elaboration_local_object_mode_two_edge_chain_expansion_001"
            && result.actual_detail_keys.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_type_elaboration_local_mode_three_edge_chain_expansion_001"
            && result.actual_detail_keys.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_type_elaboration_local_object_mode_three_edge_chain_expansion_001"
            && result.actual_detail_keys.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_type_elaboration_local_mode_four_edge_chain_expansion_001"
            && result.actual_detail_keys.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_type_elaboration_local_mode_cached_four_edge_chain_expansion_001"
            && result.actual_detail_keys.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_type_elaboration_local_object_mode_four_edge_chain_expansion_001"
            && result.actual_detail_keys.is_empty()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_type_elaboration_local_mode_long_chain_expansion_001"
            && result.actual_detail_keys.is_empty()
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
fn declaration_symbol_runner_rejects_active_tag_on_non_resolve_case() {
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
tags = ["active_declaration_symbol"]
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

    let report = run_declaration_symbol_corpus(&corpus.config()).unwrap();

    assert_eq!(report.results.len(), 0);
    assert_has_declaration_symbol_report_code(&report, "E-DECLARATION-SYMBOL-ACTIVE-GATE");
}

#[test]
fn declaration_symbol_runner_rejects_public_diagnostic_codes_until_range_exists() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/miz/fail/resolve/fail_duplicate_theorem.miz",
        "theorem Clash: thesis;\ntheorem Clash: thesis;\n",
    );
    corpus.write(
        "tests/miz/fail/resolve/fail_duplicate_theorem.expect.toml",
        r#"schema_version = 1
id = "fail_duplicate_theorem"
kind = "fail"
stage = "declaration_symbol"
domain = "resolve.declaration_symbol"
source = "fail_duplicate_theorem.miz"
expected_outcome = "fail"
expected_phase = "resolve"
failure_category = "resolve_error"
rejection_reason = "duplicate_theorem_symbol"
stable_detail_key = "declaration_symbol.symbol.duplicate_declaration"
diagnostic_codes = ["E-RESOLVE-BOGUS"]
tags = ["active_declaration_symbol"]
spec_refs = ["spec.en.test.resolve"]
"#,
    );
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.test.resolve"
source = "doc/spec/en/test.md"
section = "Test"
stage = "declaration_symbol"
status = "partial"
required = true
coverage = "manual_review"
tests = ["tests/miz/fail/resolve/fail_duplicate_theorem.expect.toml"]
"#,
    );
    corpus.write("doc/spec/en/test.md", "# Test\n");

    let report = run_declaration_symbol_corpus(&corpus.config()).unwrap();

    assert_has_declaration_symbol_report_code(
        &report,
        "E-DECLARATION-SYMBOL-PUBLIC-DIAGNOSTIC-CODES",
    );
}

#[test]
fn type_elaboration_runner_rejects_active_tag_on_non_type_case() {
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
tags = ["active_type_elaboration"]
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

    let report = run_type_elaboration_corpus(&corpus.config()).unwrap();

    assert_eq!(report.results.len(), 0);
    assert_has_type_elaboration_report_code(&report, "E-TYPE-ELABORATION-ACTIVE-GATE");
}

#[test]
fn type_elaboration_runner_rejects_public_diagnostic_codes_until_range_exists() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/miz/fail/types/fail_payload_gap.miz",
        "theorem VisibleTheorem: thesis;\n",
    );
    corpus.write(
        "tests/miz/fail/types/fail_payload_gap.expect.toml",
        r#"schema_version = 1
id = "fail_payload_gap"
kind = "fail"
stage = "type_elaboration"
domain = "checker.type_elaboration"
source = "fail_payload_gap.miz"
expected_outcome = "fail"
expected_phase = "type_check"
failure_category = "external_dependency_gap"
rejection_reason = "missing_ast_payload_extraction"
stable_detail_key = "type_elaboration.external_dependency.ast_payload_extraction"
diagnostic_codes = ["E-CHECKER-BOGUS"]
tags = ["active_type_elaboration"]
spec_refs = ["spec.en.test.type_elaboration"]
"#,
    );
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.test.type_elaboration"
source = "doc/spec/en/test.md"
section = "Test"
stage = "type_elaboration"
status = "partial"
required = true
coverage = "diagnostic"
tests = ["tests/miz/fail/types/fail_payload_gap.expect.toml"]
"#,
    );
    corpus.write("doc/spec/en/test.md", "# Test\n");

    let report = run_type_elaboration_corpus(&corpus.config()).unwrap();

    assert_has_type_elaboration_report_code(&report, "E-TYPE-ELABORATION-PUBLIC-DIAGNOSTIC-CODES");
}

#[test]
fn type_elaboration_runner_uses_stable_detail_key_fallback() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/miz/fail/types/fail_payload_gap.miz",
        "theorem VisibleTheorem: thesis;\n",
    );
    corpus.write(
        "tests/miz/fail/types/fail_payload_gap.expect.toml",
        r#"schema_version = 1
id = "fail_payload_gap"
kind = "fail"
stage = "type_elaboration"
domain = "checker.type_elaboration"
source = "fail_payload_gap.miz"
expected_outcome = "fail"
expected_phase = "type_check"
failure_category = "external_dependency_gap"
rejection_reason = "missing_ast_payload_extraction"
stable_detail_key = "type_elaboration.external_dependency.ast_payload_extraction"
diagnostic_codes = []
tags = ["active_type_elaboration"]
spec_refs = ["spec.en.test.type_elaboration"]
"#,
    );
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.test.type_elaboration"
source = "doc/spec/en/test.md"
section = "Test"
stage = "type_elaboration"
status = "partial"
required = true
coverage = "diagnostic"
tests = ["tests/miz/fail/types/fail_payload_gap.expect.toml"]
"#,
    );
    corpus.write("doc/spec/en/test.md", "# Test\n");

    let report = run_type_elaboration_corpus(&corpus.config()).unwrap();

    assert_eq!(report.error_count(), 0, "{:#?}", report.diagnostics);
    assert_eq!(report.results.len(), 1);
    assert_eq!(report.passed_count(), 1);
}

#[test]
fn type_elaboration_runner_accepts_source_derived_builtin_type_expressions() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/miz/pass/types/pass_builtin_types.miz",
        "reserve x for set;\nreserve y for object;\n",
    );
    corpus.write(
        "tests/miz/pass/types/pass_builtin_types.expect.toml",
        r#"schema_version = 1
id = "pass_builtin_types"
kind = "pass"
stage = "type_elaboration"
domain = "checker.type_elaboration"
source = "pass_builtin_types.miz"
expected_outcome = "pass"
expected_phase = "type_check"
diagnostic_codes = []
diagnostic_payloads = []
tags = ["active_type_elaboration"]
spec_refs = ["spec.en.test.type_elaboration.builtin"]
"#,
    );
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.test.type_elaboration.builtin"
source = "doc/spec/en/test.md"
section = "Test"
stage = "type_elaboration"
status = "covered"
required = true
coverage = "pass"
tests = ["tests/miz/pass/types/pass_builtin_types.expect.toml"]
"#,
    );
    corpus.write("doc/spec/en/test.md", "# Test\n");

    let report = run_type_elaboration_corpus(&corpus.config()).unwrap();

    assert_eq!(report.error_count(), 0, "{:#?}", report.diagnostics);
    assert_eq!(report.results.len(), 1);
    assert_eq!(report.passed_count(), 1);
    assert!(report.results[0].actual_detail_keys.is_empty());
}

#[test]
fn type_elaboration_runner_accepts_source_derived_local_mode_expansion() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/miz/pass/types/pass_local_mode_expansion.miz",
        "definition\n  mode LocalModeDef: LocalMode is set;\nend;\n\nreserve z for LocalMode;\n",
    );
    corpus.write(
        "tests/miz/pass/types/pass_local_mode_expansion.expect.toml",
        r#"schema_version = 1
id = "pass_local_mode_expansion"
kind = "pass"
stage = "type_elaboration"
domain = "checker.type_elaboration"
source = "pass_local_mode_expansion.miz"
expected_outcome = "pass"
expected_phase = "type_check"
diagnostic_codes = []
diagnostic_payloads = []
tags = ["active_type_elaboration"]
spec_refs = ["spec.en.test.type_elaboration.local_mode_expansion"]
"#,
    );
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.test.type_elaboration.local_mode_expansion"
source = "doc/spec/en/test.md"
section = "Test"
stage = "type_elaboration"
status = "covered"
required = true
coverage = "pass"
tests = ["tests/miz/pass/types/pass_local_mode_expansion.expect.toml"]
"#,
    );
    corpus.write("doc/spec/en/test.md", "# Test\n");

    let report = run_type_elaboration_corpus(&corpus.config()).unwrap();

    assert_eq!(report.error_count(), 0, "{:#?}", report.diagnostics);
    assert_eq!(report.results.len(), 1);
    assert_eq!(report.passed_count(), 1);
    assert!(report.results[0].actual_detail_keys.is_empty());
}

#[test]
fn type_elaboration_runner_accepts_source_derived_local_mode_two_edge_chain_expansion() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/miz/pass/types/pass_local_mode_two_edge_chain_expansion.miz",
        "definition\n  mode BaseModeDef: BaseMode is set;\nend;\n\ndefinition\n  mode MiddleModeDef: MiddleMode is BaseMode;\nend;\n\ndefinition\n  mode OuterModeDef: OuterMode is MiddleMode;\nend;\n\nreserve z for OuterMode;\n",
    );
    corpus.write(
        "tests/miz/pass/types/pass_local_mode_two_edge_chain_expansion.expect.toml",
        r#"schema_version = 1
id = "pass_local_mode_two_edge_chain_expansion"
kind = "pass"
stage = "type_elaboration"
domain = "checker.type_elaboration"
source = "pass_local_mode_two_edge_chain_expansion.miz"
expected_outcome = "pass"
expected_phase = "type_check"
diagnostic_codes = []
diagnostic_payloads = []
tags = ["active_type_elaboration"]
spec_refs = ["spec.en.test.type_elaboration.local_mode_two_edge_chain_expansion"]
"#,
    );
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.test.type_elaboration.local_mode_two_edge_chain_expansion"
source = "doc/spec/en/test.md"
section = "Test"
stage = "type_elaboration"
status = "covered"
required = true
coverage = "pass"
tests = ["tests/miz/pass/types/pass_local_mode_two_edge_chain_expansion.expect.toml"]
"#,
    );
    corpus.write("doc/spec/en/test.md", "# Test\n");

    let report = run_type_elaboration_corpus(&corpus.config()).unwrap();

    assert_eq!(report.error_count(), 0, "{:#?}", report.diagnostics);
    assert_eq!(report.results.len(), 1);
    assert_eq!(report.passed_count(), 1);
    assert!(report.results[0].actual_detail_keys.is_empty());
}

#[test]
fn type_elaboration_runner_accepts_source_derived_local_object_mode_two_edge_chain_expansion() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/miz/pass/types/pass_local_object_mode_two_edge_chain_expansion.miz",
        "definition\n  mode BaseObjectModeDef: BaseObjectMode is object;\nend;\n\ndefinition\n  mode MiddleObjectModeDef: MiddleObjectMode is BaseObjectMode;\nend;\n\ndefinition\n  mode OuterObjectModeDef: OuterObjectMode is MiddleObjectMode;\nend;\n\nreserve z for OuterObjectMode;\n",
    );
    corpus.write(
        "tests/miz/pass/types/pass_local_object_mode_two_edge_chain_expansion.expect.toml",
        r#"schema_version = 1
id = "pass_local_object_mode_two_edge_chain_expansion"
kind = "pass"
stage = "type_elaboration"
domain = "checker.type_elaboration"
source = "pass_local_object_mode_two_edge_chain_expansion.miz"
expected_outcome = "pass"
expected_phase = "type_check"
diagnostic_codes = []
diagnostic_payloads = []
tags = ["active_type_elaboration"]
spec_refs = ["spec.en.test.type_elaboration.local_object_mode_two_edge_chain_expansion"]
"#,
    );
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.test.type_elaboration.local_object_mode_two_edge_chain_expansion"
source = "doc/spec/en/test.md"
section = "Test"
stage = "type_elaboration"
status = "covered"
required = true
coverage = "pass"
tests = ["tests/miz/pass/types/pass_local_object_mode_two_edge_chain_expansion.expect.toml"]
"#,
    );
    corpus.write("doc/spec/en/test.md", "# Test\n");

    let report = run_type_elaboration_corpus(&corpus.config()).unwrap();

    assert_eq!(report.error_count(), 0, "{:#?}", report.diagnostics);
    assert_eq!(report.results.len(), 1);
    assert_eq!(report.passed_count(), 1);
    assert!(report.results[0].actual_detail_keys.is_empty());
}

#[test]
fn type_elaboration_runner_accepts_source_derived_local_mode_three_edge_chain_expansion() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/miz/pass/types/pass_local_mode_three_edge_chain_expansion.miz",
        "definition\n  mode BaseModeDef: BaseMode is set;\nend;\n\ndefinition\n  mode InnerModeDef: InnerMode is BaseMode;\nend;\n\ndefinition\n  mode MiddleModeDef: MiddleMode is InnerMode;\nend;\n\ndefinition\n  mode OuterModeDef: OuterMode is MiddleMode;\nend;\n\nreserve z for OuterMode;\n",
    );
    corpus.write(
        "tests/miz/pass/types/pass_local_mode_three_edge_chain_expansion.expect.toml",
        r#"schema_version = 1
id = "pass_local_mode_three_edge_chain_expansion"
kind = "pass"
stage = "type_elaboration"
domain = "checker.type_elaboration"
source = "pass_local_mode_three_edge_chain_expansion.miz"
expected_outcome = "pass"
expected_phase = "type_check"
diagnostic_codes = []
diagnostic_payloads = []
tags = ["active_type_elaboration"]
spec_refs = ["spec.en.test.type_elaboration.local_mode_three_edge_chain_expansion"]
"#,
    );
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.test.type_elaboration.local_mode_three_edge_chain_expansion"
source = "doc/spec/en/test.md"
section = "Test"
stage = "type_elaboration"
status = "covered"
required = true
coverage = "pass"
tests = ["tests/miz/pass/types/pass_local_mode_three_edge_chain_expansion.expect.toml"]
"#,
    );
    corpus.write("doc/spec/en/test.md", "# Test\n");

    let report = run_type_elaboration_corpus(&corpus.config()).unwrap();

    assert_eq!(report.error_count(), 0, "{:#?}", report.diagnostics);
    assert_eq!(report.results.len(), 1);
    assert_eq!(report.passed_count(), 1);
    assert!(report.results[0].actual_detail_keys.is_empty());
}

#[test]
fn type_elaboration_runner_accepts_source_derived_local_object_mode_three_edge_chain_expansion() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/miz/pass/types/pass_local_object_mode_three_edge_chain_expansion.miz",
        "definition\n  mode BaseObjectModeDef: BaseObjectMode is object;\nend;\n\ndefinition\n  mode InnerObjectModeDef: InnerObjectMode is BaseObjectMode;\nend;\n\ndefinition\n  mode MiddleObjectModeDef: MiddleObjectMode is InnerObjectMode;\nend;\n\ndefinition\n  mode OuterObjectModeDef: OuterObjectMode is MiddleObjectMode;\nend;\n\nreserve z for OuterObjectMode;\n",
    );
    corpus.write(
        "tests/miz/pass/types/pass_local_object_mode_three_edge_chain_expansion.expect.toml",
        r#"schema_version = 1
id = "pass_local_object_mode_three_edge_chain_expansion"
kind = "pass"
stage = "type_elaboration"
domain = "checker.type_elaboration"
source = "pass_local_object_mode_three_edge_chain_expansion.miz"
expected_outcome = "pass"
expected_phase = "type_check"
diagnostic_codes = []
diagnostic_payloads = []
tags = ["active_type_elaboration"]
spec_refs = ["spec.en.test.type_elaboration.local_object_mode_three_edge_chain_expansion"]
"#,
    );
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.test.type_elaboration.local_object_mode_three_edge_chain_expansion"
source = "doc/spec/en/test.md"
section = "Test"
stage = "type_elaboration"
status = "covered"
required = true
coverage = "pass"
tests = ["tests/miz/pass/types/pass_local_object_mode_three_edge_chain_expansion.expect.toml"]
"#,
    );
    corpus.write("doc/spec/en/test.md", "# Test\n");

    let report = run_type_elaboration_corpus(&corpus.config()).unwrap();

    assert_eq!(report.error_count(), 0, "{:#?}", report.diagnostics);
    assert_eq!(report.results.len(), 1);
    assert_eq!(report.passed_count(), 1);
    assert!(report.results[0].actual_detail_keys.is_empty());
}

#[test]
fn type_elaboration_runner_accepts_source_derived_local_mode_four_edge_chain_expansion() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/miz/pass/types/pass_four_edge_local_mode_chain.miz",
        "definition\n  mode BaseModeDef: BaseMode is set;\nend;\n\ndefinition\n  mode InnerModeDef: InnerMode is BaseMode;\nend;\n\ndefinition\n  mode MiddleModeDef: MiddleMode is InnerMode;\nend;\n\ndefinition\n  mode OuterModeDef: OuterMode is MiddleMode;\nend;\n\ndefinition\n  mode TooDeepModeDef: TooDeepMode is OuterMode;\nend;\n\nreserve z for TooDeepMode;\n",
    );
    corpus.write(
        "tests/miz/pass/types/pass_four_edge_local_mode_chain.expect.toml",
        r#"schema_version = 1
id = "pass_four_edge_local_mode_chain"
kind = "pass"
stage = "type_elaboration"
domain = "checker.type_elaboration"
source = "pass_four_edge_local_mode_chain.miz"
expected_outcome = "pass"
expected_phase = "type_check"
diagnostic_codes = []
diagnostic_payloads = []
tags = ["active_type_elaboration"]
spec_refs = ["spec.en.test.type_elaboration.local_mode_structural_chain_expansion"]
"#,
    );
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.test.type_elaboration.local_mode_structural_chain_expansion"
source = "doc/spec/en/test.md"
section = "Test"
stage = "type_elaboration"
status = "covered"
required = true
coverage = "pass"
tests = ["tests/miz/pass/types/pass_four_edge_local_mode_chain.expect.toml"]
"#,
    );
    corpus.write("doc/spec/en/test.md", "# Test\n");

    let report = run_type_elaboration_corpus(&corpus.config()).unwrap();

    assert_eq!(report.error_count(), 0, "{:#?}", report.diagnostics);
    assert_eq!(report.results.len(), 1);
    assert_eq!(report.passed_count(), 1);
    assert!(report.results[0].actual_detail_keys.is_empty());
}

#[test]
fn type_elaboration_runner_accepts_cached_local_mode_four_edge_chain_expansion() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/miz/pass/types/pass_cached_four_edge_local_mode_chain.miz",
        "definition\n  mode BaseModeDef: BaseMode is set;\nend;\n\ndefinition\n  mode InnerModeDef: InnerMode is BaseMode;\nend;\n\ndefinition\n  mode MiddleModeDef: MiddleMode is InnerMode;\nend;\n\ndefinition\n  mode OuterModeDef: OuterMode is MiddleMode;\nend;\n\ndefinition\n  mode TooDeepModeDef: TooDeepMode is OuterMode;\nend;\n\nreserve y for OuterMode;\nreserve z for TooDeepMode;\n",
    );
    corpus.write(
        "tests/miz/pass/types/pass_cached_four_edge_local_mode_chain.expect.toml",
        r#"schema_version = 1
id = "pass_cached_four_edge_local_mode_chain"
kind = "pass"
stage = "type_elaboration"
domain = "checker.type_elaboration"
source = "pass_cached_four_edge_local_mode_chain.miz"
expected_outcome = "pass"
expected_phase = "type_check"
diagnostic_codes = []
diagnostic_payloads = []
tags = ["active_type_elaboration"]
spec_refs = ["spec.en.test.type_elaboration.cached_local_mode_structural_chain_expansion"]
"#,
    );
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.test.type_elaboration.cached_local_mode_structural_chain_expansion"
source = "doc/spec/en/test.md"
section = "Test"
stage = "type_elaboration"
status = "covered"
required = true
coverage = "pass"
tests = ["tests/miz/pass/types/pass_cached_four_edge_local_mode_chain.expect.toml"]
"#,
    );
    corpus.write("doc/spec/en/test.md", "# Test\n");

    let report = run_type_elaboration_corpus(&corpus.config()).unwrap();

    assert_eq!(report.error_count(), 0, "{:#?}", report.diagnostics);
    assert_eq!(report.results.len(), 1);
    assert_eq!(report.passed_count(), 1);
    assert!(report.results[0].actual_detail_keys.is_empty());
}

#[test]
fn type_elaboration_runner_accepts_source_derived_local_mode_long_chain_expansion() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/miz/pass/types/pass_long_local_mode_chain.miz",
        "definition\n  mode BaseModeDef: BaseMode is set;\nend;\n\ndefinition\n  mode ChainMode1Def: ChainMode1 is BaseMode;\nend;\n\ndefinition\n  mode ChainMode2Def: ChainMode2 is ChainMode1;\nend;\n\ndefinition\n  mode ChainMode3Def: ChainMode3 is ChainMode2;\nend;\n\ndefinition\n  mode ChainMode4Def: ChainMode4 is ChainMode3;\nend;\n\ndefinition\n  mode ChainMode5Def: ChainMode5 is ChainMode4;\nend;\n\ndefinition\n  mode ChainMode6Def: ChainMode6 is ChainMode5;\nend;\n\nreserve z for ChainMode6;\n",
    );
    corpus.write(
        "tests/miz/pass/types/pass_long_local_mode_chain.expect.toml",
        r#"schema_version = 1
id = "pass_long_local_mode_chain"
kind = "pass"
stage = "type_elaboration"
domain = "checker.type_elaboration"
source = "pass_long_local_mode_chain.miz"
expected_outcome = "pass"
expected_phase = "type_check"
diagnostic_codes = []
diagnostic_payloads = []
tags = ["active_type_elaboration"]
spec_refs = ["spec.en.test.type_elaboration.local_mode_long_structural_chain_expansion"]
"#,
    );
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.test.type_elaboration.local_mode_long_structural_chain_expansion"
source = "doc/spec/en/test.md"
section = "Test"
stage = "type_elaboration"
status = "covered"
required = true
coverage = "pass"
tests = ["tests/miz/pass/types/pass_long_local_mode_chain.expect.toml"]
"#,
    );
    corpus.write("doc/spec/en/test.md", "# Test\n");

    let report = run_type_elaboration_corpus(&corpus.config()).unwrap();

    assert_eq!(report.error_count(), 0, "{:#?}", report.diagnostics);
    assert_eq!(report.results.len(), 1);
    assert_eq!(report.passed_count(), 1);
    assert!(report.results[0].actual_detail_keys.is_empty());
}

#[test]
fn type_elaboration_runner_does_not_expand_mixed_attributed_local_mode_uses() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/miz/fail/types/fail_mixed_local_mode_expansion.miz",
        "definition\n  let x be set;\n  attr MarkedDef: x is marked means thesis;\nend;\n\ndefinition\n  mode LocalModeDef: LocalMode is set;\nend;\n\nreserve x for LocalMode;\nreserve y for marked LocalMode;\n",
    );
    corpus.write(
        "tests/miz/fail/types/fail_mixed_local_mode_expansion.expect.toml",
        r#"schema_version = 1
id = "fail_mixed_local_mode_expansion"
kind = "fail"
stage = "type_elaboration"
domain = "checker.type_elaboration"
source = "fail_mixed_local_mode_expansion.miz"
expected_outcome = "fail"
expected_phase = "type_check"
failure_category = "external_dependency_gap"
rejection_reason = "missing_mode_expansion_payload"
stable_detail_key = "type_elaboration.checker.checker.type.external.mode_expansion_payload"
diagnostic_codes = []
diagnostic_payloads = [
  "type_elaboration.checker.checker.type.external.mode_expansion_payload",
  "type_elaboration.checker.checker.type.recovery",
]
tags = ["active_type_elaboration"]
spec_refs = ["spec.en.test.type_elaboration.local_mode_expansion_gap"]
"#,
    );
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.test.type_elaboration.local_mode_expansion_gap"
source = "doc/spec/en/test.md"
section = "Test"
stage = "type_elaboration"
status = "covered"
required = true
coverage = "diagnostic"
tests = ["tests/miz/fail/types/fail_mixed_local_mode_expansion.expect.toml"]
"#,
    );
    corpus.write("doc/spec/en/test.md", "# Test\n");

    let report = run_type_elaboration_corpus(&corpus.config()).unwrap();

    assert_eq!(report.error_count(), 0, "{:#?}", report.diagnostics);
    assert_eq!(report.results.len(), 1);
    assert_eq!(report.passed_count(), 1);
    assert_eq!(
        report.results[0].actual_detail_keys,
        [
            "type_elaboration.checker.checker.type.external.mode_expansion_payload",
            "type_elaboration.checker.checker.type.recovery",
        ]
    );
}

#[test]
fn type_elaboration_runner_expands_attributed_local_mode_reserve_to_evidence_gap() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/miz/fail/types/fail_attributed_local_mode_expansion.miz",
        "definition\n  let x be set;\n  attr MarkedDef: x is marked means thesis;\nend;\n\ndefinition\n  mode LocalModeDef: LocalMode is set;\nend;\n\nreserve y for marked LocalMode;\n",
    );
    corpus.write(
        "tests/miz/fail/types/fail_attributed_local_mode_expansion.expect.toml",
        r#"schema_version = 1
id = "fail_attributed_local_mode_expansion"
kind = "fail"
stage = "type_elaboration"
domain = "checker.type_elaboration"
source = "fail_attributed_local_mode_expansion.miz"
expected_outcome = "fail"
expected_phase = "type_check"
failure_category = "external_dependency_gap"
rejection_reason = "missing_attributed_mode_evidence_query"
stable_detail_key = "type_elaboration.checker.checker.declaration.deferred.evidence_query"
diagnostic_codes = []
diagnostic_payloads = [
  "type_elaboration.checker.checker.declaration.deferred.evidence_query",
]
tags = ["active_type_elaboration"]
spec_refs = ["spec.en.test.type_elaboration.attributed_local_mode_evidence_gap"]
"#,
    );
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.test.type_elaboration.attributed_local_mode_evidence_gap"
source = "doc/spec/en/test.md"
section = "Test"
stage = "type_elaboration"
status = "covered"
required = true
coverage = "diagnostic"
tests = ["tests/miz/fail/types/fail_attributed_local_mode_expansion.expect.toml"]
"#,
    );
    corpus.write("doc/spec/en/test.md", "# Test\n");

    let report = run_type_elaboration_corpus(&corpus.config()).unwrap();

    assert_eq!(report.error_count(), 0, "{:#?}", report.diagnostics);
    assert_eq!(report.results.len(), 1);
    assert_eq!(report.passed_count(), 1);
    assert_eq!(
        report.results[0].actual_detail_keys,
        ["type_elaboration.checker.checker.declaration.deferred.evidence_query"]
    );
}

#[test]
fn type_elaboration_runner_expands_attributed_local_mode_structure_rhs_to_evidence_gap() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/miz/fail/types/fail_attributed_local_mode_structure_rhs.miz",
        "definition\n  let x be set;\n  attr MarkedDef: x is marked means thesis;\nend;\n\ndefinition\n  struct LocalStruct where\n    field carrier -> set;\n  end;\nend;\n\ndefinition\n  mode LocalModeDef: LocalMode is LocalStruct;\nend;\n\nreserve y for marked LocalMode;\n",
    );
    corpus.write(
        "tests/miz/fail/types/fail_attributed_local_mode_structure_rhs.expect.toml",
        r#"schema_version = 1
id = "fail_attributed_local_mode_structure_rhs"
kind = "fail"
stage = "type_elaboration"
domain = "checker.type_elaboration"
source = "fail_attributed_local_mode_structure_rhs.miz"
expected_outcome = "fail"
expected_phase = "type_check"
failure_category = "external_dependency_gap"
rejection_reason = "missing_attributed_mode_structure_rhs_evidence_query"
stable_detail_key = "type_elaboration.checker.checker.declaration.deferred.evidence_query"
diagnostic_codes = []
diagnostic_payloads = [
  "type_elaboration.checker.checker.declaration.deferred.evidence_query",
]
tags = ["active_type_elaboration"]
spec_refs = ["spec.en.test.type_elaboration.attributed_local_mode_structure_rhs_evidence_gap"]
"#,
    );
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.test.type_elaboration.attributed_local_mode_structure_rhs_evidence_gap"
source = "doc/spec/en/test.md"
section = "Test"
stage = "type_elaboration"
status = "covered"
required = true
coverage = "diagnostic"
tests = ["tests/miz/fail/types/fail_attributed_local_mode_structure_rhs.expect.toml"]
"#,
    );
    corpus.write("doc/spec/en/test.md", "# Test\n");

    let report = run_type_elaboration_corpus(&corpus.config()).unwrap();

    assert_eq!(report.error_count(), 0, "{:#?}", report.diagnostics);
    assert_eq!(report.results.len(), 1);
    assert_eq!(report.passed_count(), 1);
    assert_eq!(
        report.results[0].actual_detail_keys,
        ["type_elaboration.checker.checker.declaration.deferred.evidence_query"]
    );
}

#[test]
fn type_elaboration_runner_expands_local_mode_structure_rhs_chain_to_evidence_gap() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/miz/fail/types/fail_local_mode_structure_rhs_chain.miz",
        "definition\n  struct LocalStruct where\n    field carrier -> set;\n  end;\nend;\n\ndefinition\n  mode StructModeDef: StructMode is LocalStruct;\nend;\n\ndefinition\n  mode ChainStructModeDef: ChainStructMode is StructMode;\nend;\n\nreserve z for ChainStructMode;\n",
    );
    corpus.write(
        "tests/miz/fail/types/fail_local_mode_structure_rhs_chain.expect.toml",
        r#"schema_version = 1
id = "fail_local_mode_structure_rhs_chain"
kind = "fail"
stage = "type_elaboration"
domain = "checker.type_elaboration"
source = "fail_local_mode_structure_rhs_chain.miz"
expected_outcome = "fail"
expected_phase = "type_check"
failure_category = "external_dependency_gap"
rejection_reason = "missing_structure_chain_inhabitation_evidence_payload"
stable_detail_key = "type_elaboration.checker.checker.declaration.deferred.evidence_query"
diagnostic_codes = []
diagnostic_payloads = [
  "type_elaboration.checker.checker.declaration.deferred.evidence_query",
]
tags = ["active_type_elaboration"]
spec_refs = ["spec.en.test.type_elaboration.local_mode_structure_rhs_chain_evidence_gap"]
"#,
    );
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.test.type_elaboration.local_mode_structure_rhs_chain_evidence_gap"
source = "doc/spec/en/test.md"
section = "Test"
stage = "type_elaboration"
status = "covered"
required = true
coverage = "diagnostic"
tests = ["tests/miz/fail/types/fail_local_mode_structure_rhs_chain.expect.toml"]
"#,
    );
    corpus.write("doc/spec/en/test.md", "# Test\n");

    let report = run_type_elaboration_corpus(&corpus.config()).unwrap();

    assert_eq!(report.error_count(), 0, "{:#?}", report.diagnostics);
    assert_eq!(report.results.len(), 1);
    assert_eq!(report.passed_count(), 1);
    assert_eq!(
        report.results[0].actual_detail_keys,
        ["type_elaboration.checker.checker.declaration.deferred.evidence_query"]
    );
}

#[test]
fn type_elaboration_runner_expands_attributed_local_mode_structure_rhs_chain_to_evidence_gap() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/miz/fail/types/fail_attributed_local_mode_structure_rhs_chain.miz",
        "definition\n  let x be set;\n  attr MarkedStructChainDef: x is marked means thesis;\nend;\n\ndefinition\n  struct LocalStruct where\n    field carrier -> set;\n  end;\nend;\n\ndefinition\n  mode BaseStructChainDef: BaseStructChain is LocalStruct;\nend;\n\ndefinition\n  mode AttributedStructChainDef: AttributedStructChain is BaseStructChain;\nend;\n\nreserve z for marked AttributedStructChain;\n",
    );
    corpus.write(
        "tests/miz/fail/types/fail_attributed_local_mode_structure_rhs_chain.expect.toml",
        r#"schema_version = 1
id = "fail_attributed_local_mode_structure_rhs_chain"
kind = "fail"
stage = "type_elaboration"
domain = "checker.type_elaboration"
source = "fail_attributed_local_mode_structure_rhs_chain.miz"
expected_outcome = "fail"
expected_phase = "type_check"
failure_category = "external_dependency_gap"
rejection_reason = "missing_attributed_structure_rhs_chain_evidence_query"
stable_detail_key = "type_elaboration.checker.checker.declaration.deferred.evidence_query"
diagnostic_codes = []
diagnostic_payloads = [
  "type_elaboration.checker.checker.declaration.deferred.evidence_query",
]
tags = ["active_type_elaboration"]
spec_refs = ["spec.en.test.type_elaboration.attributed_local_mode_structure_rhs_chain_evidence_gap"]
"#,
    );
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.test.type_elaboration.attributed_local_mode_structure_rhs_chain_evidence_gap"
source = "doc/spec/en/test.md"
section = "Test"
stage = "type_elaboration"
status = "covered"
required = true
coverage = "diagnostic"
tests = ["tests/miz/fail/types/fail_attributed_local_mode_structure_rhs_chain.expect.toml"]
"#,
    );
    corpus.write("doc/spec/en/test.md", "# Test\n");

    let report = run_type_elaboration_corpus(&corpus.config()).unwrap();

    assert_eq!(report.error_count(), 0, "{:#?}", report.diagnostics);
    assert_eq!(report.results.len(), 1);
    assert_eq!(report.passed_count(), 1);
    assert_eq!(
        report.results[0].actual_detail_keys,
        ["type_elaboration.checker.checker.declaration.deferred.evidence_query"]
    );
}

#[test]
fn type_elaboration_runner_expands_local_mode_attributed_rhs_chain_to_evidence_gap() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/miz/fail/types/fail_local_mode_attributed_rhs_chain.miz",
        "definition\n  let x be set;\n  attr MarkedDef: x is marked means thesis;\nend;\n\ndefinition\n  mode MarkedModeDef: MarkedMode is marked set;\nend;\n\ndefinition\n  mode ChainMarkedModeDef: ChainMarkedMode is MarkedMode;\nend;\n\nreserve z for ChainMarkedMode;\n",
    );
    corpus.write(
        "tests/miz/fail/types/fail_local_mode_attributed_rhs_chain.expect.toml",
        r#"schema_version = 1
id = "fail_local_mode_attributed_rhs_chain"
kind = "fail"
stage = "type_elaboration"
domain = "checker.type_elaboration"
source = "fail_local_mode_attributed_rhs_chain.miz"
expected_outcome = "fail"
expected_phase = "type_check"
failure_category = "external_dependency_gap"
rejection_reason = "missing_attributed_rhs_chain_evidence_query"
stable_detail_key = "type_elaboration.checker.checker.declaration.deferred.evidence_query"
diagnostic_codes = []
diagnostic_payloads = [
  "type_elaboration.checker.checker.declaration.deferred.evidence_query",
]
tags = ["active_type_elaboration"]
spec_refs = ["spec.en.test.type_elaboration.local_mode_attributed_rhs_chain_evidence_gap"]
"#,
    );
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.test.type_elaboration.local_mode_attributed_rhs_chain_evidence_gap"
source = "doc/spec/en/test.md"
section = "Test"
stage = "type_elaboration"
status = "covered"
required = true
coverage = "diagnostic"
tests = ["tests/miz/fail/types/fail_local_mode_attributed_rhs_chain.expect.toml"]
"#,
    );
    corpus.write("doc/spec/en/test.md", "# Test\n");

    let report = run_type_elaboration_corpus(&corpus.config()).unwrap();

    assert_eq!(report.error_count(), 0, "{:#?}", report.diagnostics);
    assert_eq!(report.results.len(), 1);
    assert_eq!(report.passed_count(), 1);
    assert_eq!(
        report.results[0].actual_detail_keys,
        ["type_elaboration.checker.checker.declaration.deferred.evidence_query"]
    );
}

#[test]
fn type_elaboration_runner_expands_attributed_local_mode_attributed_rhs_chain_to_evidence_gap() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/miz/fail/types/fail_attributed_local_mode_attributed_rhs_chain.miz",
        "definition\n  let x be set;\n  attr MarkedAttrChainDef: x is marked means thesis;\nend;\n\ndefinition\n  mode BaseMarkedChainDef: BaseMarkedChain is marked set;\nend;\n\ndefinition\n  mode AttributedMarkedChainDef: AttributedMarkedChain is BaseMarkedChain;\nend;\n\nreserve z for marked AttributedMarkedChain;\n",
    );
    corpus.write(
        "tests/miz/fail/types/fail_attributed_local_mode_attributed_rhs_chain.expect.toml",
        r#"schema_version = 1
id = "fail_attributed_local_mode_attributed_rhs_chain"
kind = "fail"
stage = "type_elaboration"
domain = "checker.type_elaboration"
source = "fail_attributed_local_mode_attributed_rhs_chain.miz"
expected_outcome = "fail"
expected_phase = "type_check"
failure_category = "external_dependency_gap"
rejection_reason = "missing_attributed_rhs_chain_evidence_query"
stable_detail_key = "type_elaboration.checker.checker.declaration.deferred.evidence_query"
diagnostic_codes = []
diagnostic_payloads = [
  "type_elaboration.checker.checker.declaration.deferred.evidence_query",
]
tags = ["active_type_elaboration"]
spec_refs = ["spec.en.test.type_elaboration.attributed_local_mode_attributed_rhs_chain_evidence_gap"]
"#,
    );
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.test.type_elaboration.attributed_local_mode_attributed_rhs_chain_evidence_gap"
source = "doc/spec/en/test.md"
section = "Test"
stage = "type_elaboration"
status = "covered"
required = true
coverage = "diagnostic"
tests = ["tests/miz/fail/types/fail_attributed_local_mode_attributed_rhs_chain.expect.toml"]
"#,
    );
    corpus.write("doc/spec/en/test.md", "# Test\n");

    let report = run_type_elaboration_corpus(&corpus.config()).unwrap();

    assert_eq!(report.error_count(), 0, "{:#?}", report.diagnostics);
    assert_eq!(report.results.len(), 1);
    assert_eq!(report.passed_count(), 1);
    assert_eq!(
        report.results[0].actual_detail_keys,
        ["type_elaboration.checker.checker.declaration.deferred.evidence_query"]
    );
}

#[test]
fn type_elaboration_runner_expands_attributed_local_mode_chain_to_evidence_gap() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/miz/fail/types/fail_attributed_local_mode_chain.miz",
        "definition\n  let x be set;\n  attr MarkedChainDef: x is marked means thesis;\nend;\n\ndefinition\n  mode BaseChainDef: BaseChain is set;\nend;\n\ndefinition\n  mode AttributedChainDef: AttributedChain is BaseChain;\nend;\n\nreserve z for marked AttributedChain;\n",
    );
    corpus.write(
        "tests/miz/fail/types/fail_attributed_local_mode_chain.expect.toml",
        r#"schema_version = 1
id = "fail_attributed_local_mode_chain"
kind = "fail"
stage = "type_elaboration"
domain = "checker.type_elaboration"
source = "fail_attributed_local_mode_chain.miz"
expected_outcome = "fail"
expected_phase = "type_check"
failure_category = "external_dependency_gap"
rejection_reason = "missing_attributed_mode_chain_evidence_query"
stable_detail_key = "type_elaboration.checker.checker.declaration.deferred.evidence_query"
diagnostic_codes = []
diagnostic_payloads = [
  "type_elaboration.checker.checker.declaration.deferred.evidence_query",
]
tags = ["active_type_elaboration"]
spec_refs = ["spec.en.test.type_elaboration.attributed_local_mode_chain_evidence_gap"]
"#,
    );
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.test.type_elaboration.attributed_local_mode_chain_evidence_gap"
source = "doc/spec/en/test.md"
section = "Test"
stage = "type_elaboration"
status = "covered"
required = true
coverage = "diagnostic"
tests = ["tests/miz/fail/types/fail_attributed_local_mode_chain.expect.toml"]
"#,
    );
    corpus.write("doc/spec/en/test.md", "# Test\n");

    let report = run_type_elaboration_corpus(&corpus.config()).unwrap();

    assert_eq!(report.error_count(), 0, "{:#?}", report.diagnostics);
    assert_eq!(report.results.len(), 1);
    assert_eq!(report.passed_count(), 1);
    assert_eq!(
        report.results[0].actual_detail_keys,
        ["type_elaboration.checker.checker.declaration.deferred.evidence_query"]
    );
}

#[test]
fn type_elaboration_runner_expands_attributed_local_mode_rhs_to_evidence_gap() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/miz/fail/types/fail_attributed_mode_rhs_expansion.miz",
        "definition\n  let x be set;\n  attr MarkedDef: x is marked means thesis;\nend;\n\ndefinition\n  mode LocalModeDef: LocalMode is marked set;\nend;\n\nreserve z for LocalMode;\n",
    );
    corpus.write(
        "tests/miz/fail/types/fail_attributed_mode_rhs_expansion.expect.toml",
        r#"schema_version = 1
id = "fail_attributed_mode_rhs_expansion"
kind = "fail"
stage = "type_elaboration"
domain = "checker.type_elaboration"
source = "fail_attributed_mode_rhs_expansion.miz"
expected_outcome = "fail"
expected_phase = "type_check"
failure_category = "external_dependency_gap"
rejection_reason = "missing_attributed_rhs_evidence_query"
stable_detail_key = "type_elaboration.checker.checker.declaration.deferred.evidence_query"
diagnostic_codes = []
diagnostic_payloads = [
  "type_elaboration.checker.checker.declaration.deferred.evidence_query",
]
tags = ["active_type_elaboration"]
spec_refs = ["spec.en.test.type_elaboration.local_mode_attributed_rhs_evidence_gap"]
"#,
    );
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.test.type_elaboration.local_mode_attributed_rhs_evidence_gap"
source = "doc/spec/en/test.md"
section = "Test"
stage = "type_elaboration"
status = "covered"
required = true
coverage = "diagnostic"
tests = ["tests/miz/fail/types/fail_attributed_mode_rhs_expansion.expect.toml"]
"#,
    );
    corpus.write("doc/spec/en/test.md", "# Test\n");

    let report = run_type_elaboration_corpus(&corpus.config()).unwrap();

    assert_eq!(report.error_count(), 0, "{:#?}", report.diagnostics);
    assert_eq!(report.results.len(), 1);
    assert_eq!(report.passed_count(), 1);
    assert_eq!(
        report.results[0].actual_detail_keys,
        ["type_elaboration.checker.checker.declaration.deferred.evidence_query"]
    );
}

#[test]
fn type_elaboration_runner_expands_attributed_local_mode_attributed_rhs_to_evidence_gap() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/miz/fail/types/fail_attributed_mode_attributed_rhs_expansion.miz",
        "definition\n  let x be set;\n  attr MarkedDef: x is marked means thesis;\nend;\n\ndefinition\n  mode LocalModeDef: LocalMode is marked set;\nend;\n\nreserve z for marked LocalMode;\n",
    );
    corpus.write(
        "tests/miz/fail/types/fail_attributed_mode_attributed_rhs_expansion.expect.toml",
        r#"schema_version = 1
id = "fail_attributed_mode_attributed_rhs_expansion"
kind = "fail"
stage = "type_elaboration"
domain = "checker.type_elaboration"
source = "fail_attributed_mode_attributed_rhs_expansion.miz"
expected_outcome = "fail"
expected_phase = "type_check"
failure_category = "external_dependency_gap"
rejection_reason = "missing_attributed_mode_attributed_rhs_evidence_query"
stable_detail_key = "type_elaboration.checker.checker.declaration.deferred.evidence_query"
diagnostic_codes = []
diagnostic_payloads = [
  "type_elaboration.checker.checker.declaration.deferred.evidence_query",
]
tags = ["active_type_elaboration"]
spec_refs = ["spec.en.test.type_elaboration.attributed_local_mode_attributed_rhs_evidence_gap"]
"#,
    );
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.test.type_elaboration.attributed_local_mode_attributed_rhs_evidence_gap"
source = "doc/spec/en/test.md"
section = "Test"
stage = "type_elaboration"
status = "covered"
required = true
coverage = "diagnostic"
tests = ["tests/miz/fail/types/fail_attributed_mode_attributed_rhs_expansion.expect.toml"]
"#,
    );
    corpus.write("doc/spec/en/test.md", "# Test\n");

    let report = run_type_elaboration_corpus(&corpus.config()).unwrap();

    assert_eq!(report.error_count(), 0, "{:#?}", report.diagnostics);
    assert_eq!(report.results.len(), 1);
    assert_eq!(report.passed_count(), 1);
    assert_eq!(
        report.results[0].actual_detail_keys,
        ["type_elaboration.checker.checker.declaration.deferred.evidence_query"]
    );
}

#[test]
fn type_elaboration_runner_keeps_structure_qualified_attributes_on_external_gap() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/miz/fail/types/fail_structure_qualified_attribute_gap.miz",
        "definition\n  let x be set;\n  attr MarkedDef: x is marked means thesis;\nend;\n\ndefinition\n  struct LocalStruct where\n    field carrier -> set;\n  end;\nend;\n\nreserve s for LocalStruct.marked LocalStruct;\n",
    );
    corpus.write(
        "tests/miz/fail/types/fail_structure_qualified_attribute_gap.expect.toml",
        r#"schema_version = 1
id = "fail_structure_qualified_attribute_gap"
kind = "fail"
stage = "type_elaboration"
domain = "checker.type_elaboration"
source = "fail_structure_qualified_attribute_gap.miz"
expected_outcome = "fail"
expected_phase = "type_check"
failure_category = "external_dependency_gap"
rejection_reason = "structure_qualified_attribute_payload_gap"
stable_detail_key = "type_elaboration.external_dependency.ast_payload_extraction"
diagnostic_codes = []
diagnostic_payloads = [
  "type_elaboration.external_dependency.ast_payload_extraction",
]
tags = ["active_type_elaboration"]
spec_refs = ["spec.en.test.type_elaboration.structure_qualified_attribute_gap"]
"#,
    );
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.test.type_elaboration.structure_qualified_attribute_gap"
source = "doc/spec/en/test.md"
section = "Test"
stage = "type_elaboration"
status = "covered"
required = true
coverage = "diagnostic"
tests = ["tests/miz/fail/types/fail_structure_qualified_attribute_gap.expect.toml"]
"#,
    );
    corpus.write("doc/spec/en/test.md", "# Test\n");

    let report = run_type_elaboration_corpus(&corpus.config()).unwrap();

    assert_eq!(report.error_count(), 0, "{:#?}", report.diagnostics);
    assert_eq!(report.results.len(), 1);
    assert_eq!(report.passed_count(), 1);
    assert_eq!(
        report.results[0].actual_detail_keys,
        ["type_elaboration.external_dependency.ast_payload_extraction"]
    );
}

#[test]
fn type_elaboration_runner_keeps_argument_bearing_attributes_on_external_gap() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/miz/fail/types/fail_argument_bearing_attribute_gap.miz",
        "definition\n  let x be set;\n  attr RankedDef: x is 2-ranked means thesis;\nend;\n\nreserve y for ranked(2) set;\n",
    );
    corpus.write(
        "tests/miz/fail/types/fail_argument_bearing_attribute_gap.expect.toml",
        r#"schema_version = 1
id = "fail_argument_bearing_attribute_gap"
kind = "fail"
stage = "type_elaboration"
domain = "checker.type_elaboration"
source = "fail_argument_bearing_attribute_gap.miz"
expected_outcome = "fail"
expected_phase = "type_check"
failure_category = "external_dependency_gap"
rejection_reason = "argument_bearing_attribute_payload_gap"
stable_detail_key = "type_elaboration.external_dependency.ast_payload_extraction"
diagnostic_codes = []
diagnostic_payloads = [
  "type_elaboration.external_dependency.ast_payload_extraction",
]
tags = ["active_type_elaboration"]
spec_refs = ["spec.en.test.type_elaboration.argument_bearing_attribute_gap"]
"#,
    );
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.test.type_elaboration.argument_bearing_attribute_gap"
source = "doc/spec/en/test.md"
section = "Test"
stage = "type_elaboration"
status = "covered"
required = true
coverage = "diagnostic"
tests = ["tests/miz/fail/types/fail_argument_bearing_attribute_gap.expect.toml"]
"#,
    );
    corpus.write("doc/spec/en/test.md", "# Test\n");

    let report = run_type_elaboration_corpus(&corpus.config()).unwrap();

    assert_eq!(report.error_count(), 0, "{:#?}", report.diagnostics);
    assert_eq!(report.results.len(), 1);
    assert_eq!(report.passed_count(), 1);
    assert_eq!(
        report.results[0].actual_detail_keys,
        ["type_elaboration.external_dependency.ast_payload_extraction"]
    );
}

#[test]
fn type_elaboration_runner_keeps_argument_bearing_mode_heads_on_external_gap() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/miz/fail/types/fail_argument_bearing_mode_gap.miz",
        "definition\n  mode ElementDef: Element of a is set;\nend;\n\nreserve e for Element of a;\n",
    );
    corpus.write(
        "tests/miz/fail/types/fail_argument_bearing_mode_gap.expect.toml",
        r#"schema_version = 1
id = "fail_argument_bearing_mode_gap"
kind = "fail"
stage = "type_elaboration"
domain = "checker.type_elaboration"
source = "fail_argument_bearing_mode_gap.miz"
expected_outcome = "fail"
expected_phase = "type_check"
failure_category = "external_dependency_gap"
rejection_reason = "argument_bearing_mode_type_head_payload_gap"
stable_detail_key = "type_elaboration.external_dependency.ast_payload_extraction"
diagnostic_codes = []
diagnostic_payloads = [
  "type_elaboration.external_dependency.ast_payload_extraction",
]
tags = ["active_type_elaboration"]
spec_refs = ["spec.en.test.type_elaboration.argument_bearing_mode_gap"]
"#,
    );
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.test.type_elaboration.argument_bearing_mode_gap"
source = "doc/spec/en/test.md"
section = "Test"
stage = "type_elaboration"
status = "covered"
required = true
coverage = "diagnostic"
tests = ["tests/miz/fail/types/fail_argument_bearing_mode_gap.expect.toml"]
"#,
    );
    corpus.write("doc/spec/en/test.md", "# Test\n");

    let report = run_type_elaboration_corpus(&corpus.config()).unwrap();

    assert_eq!(report.error_count(), 0, "{:#?}", report.diagnostics);
    assert_eq!(report.results.len(), 1);
    assert_eq!(report.passed_count(), 1);
    assert_eq!(
        report.results[0].actual_detail_keys,
        ["type_elaboration.external_dependency.ast_payload_extraction"]
    );
}

#[test]
fn type_elaboration_runner_keeps_argument_bearing_structure_heads_on_external_gap() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/miz/fail/types/fail_argument_bearing_structure_gap.miz",
        "definition\n  struct LocalStruct of p where\n    field carrier -> set;\n  end;\nend;\n\nreserve s for LocalStruct of a;\n",
    );
    corpus.write(
        "tests/miz/fail/types/fail_argument_bearing_structure_gap.expect.toml",
        r#"schema_version = 1
id = "fail_argument_bearing_structure_gap"
kind = "fail"
stage = "type_elaboration"
domain = "checker.type_elaboration"
source = "fail_argument_bearing_structure_gap.miz"
expected_outcome = "fail"
expected_phase = "type_check"
failure_category = "external_dependency_gap"
rejection_reason = "argument_bearing_structure_type_head_payload_gap"
stable_detail_key = "type_elaboration.external_dependency.ast_payload_extraction"
diagnostic_codes = []
diagnostic_payloads = [
  "type_elaboration.external_dependency.ast_payload_extraction",
]
tags = ["active_type_elaboration"]
spec_refs = ["spec.en.test.type_elaboration.argument_bearing_structure_gap"]
"#,
    );
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.test.type_elaboration.argument_bearing_structure_gap"
source = "doc/spec/en/test.md"
section = "Test"
stage = "type_elaboration"
status = "covered"
required = true
coverage = "diagnostic"
tests = ["tests/miz/fail/types/fail_argument_bearing_structure_gap.expect.toml"]
"#,
    );
    corpus.write("doc/spec/en/test.md", "# Test\n");

    let report = run_type_elaboration_corpus(&corpus.config()).unwrap();

    assert_eq!(report.error_count(), 0, "{:#?}", report.diagnostics);
    assert_eq!(report.results.len(), 1);
    assert_eq!(report.passed_count(), 1);
    assert_eq!(
        report.results[0].actual_detail_keys,
        ["type_elaboration.external_dependency.ast_payload_extraction"]
    );
}

#[test]
fn type_elaboration_runner_bridges_imported_attributes_to_evidence_gap() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/miz/fail/types/fail_imported_attribute_gap.miz",
        "import parser.type_fixtures;\nreserve a for TypeCaseAttr set;\n",
    );
    corpus.write(
        "tests/miz/fail/types/fail_imported_attribute_gap.expect.toml",
        r#"schema_version = 1
id = "fail_imported_attribute_gap"
kind = "fail"
stage = "type_elaboration"
domain = "checker.type_elaboration"
source = "fail_imported_attribute_gap.miz"
expected_outcome = "fail"
expected_phase = "type_check"
failure_category = "external_dependency_gap"
rejection_reason = "imported_attribute_evidence_payload_gap"
stable_detail_key = "type_elaboration.checker.checker.declaration.deferred.evidence_query"
diagnostic_codes = []
diagnostic_payloads = [
  "type_elaboration.checker.checker.declaration.deferred.evidence_query",
]
tags = ["active_type_elaboration"]
spec_refs = ["spec.en.test.type_elaboration.imported_attribute_provenance_bridge"]
"#,
    );
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.test.type_elaboration.imported_attribute_provenance_bridge"
source = "doc/spec/en/test.md"
section = "Test"
stage = "type_elaboration"
status = "covered"
required = true
coverage = "diagnostic"
tests = ["tests/miz/fail/types/fail_imported_attribute_gap.expect.toml"]
"#,
    );
    corpus.write("doc/spec/en/test.md", "# Test\n");

    let report = run_type_elaboration_corpus(&corpus.config()).unwrap();

    assert_eq!(report.error_count(), 0, "{:#?}", report.diagnostics);
    assert_eq!(report.results.len(), 1);
    assert_eq!(report.passed_count(), 1);
    assert_eq!(
        report.results[0].actual_detail_keys,
        ["type_elaboration.checker.checker.declaration.deferred.evidence_query"]
    );
}

#[test]
fn type_elaboration_runner_bridges_imported_structure_heads_to_evidence_gap() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/miz/fail/types/fail_imported_structure_gap.miz",
        "import parser.type_fixtures;\nreserve r for R;\n",
    );
    corpus.write(
        "tests/miz/fail/types/fail_imported_structure_gap.expect.toml",
        r#"schema_version = 1
id = "fail_imported_structure_gap"
kind = "fail"
stage = "type_elaboration"
domain = "checker.type_elaboration"
source = "fail_imported_structure_gap.miz"
expected_outcome = "fail"
expected_phase = "type_check"
failure_category = "external_dependency_gap"
rejection_reason = "imported_structure_evidence_payload_gap"
stable_detail_key = "type_elaboration.checker.checker.declaration.deferred.evidence_query"
diagnostic_codes = []
diagnostic_payloads = [
  "type_elaboration.checker.checker.declaration.deferred.evidence_query",
]
tags = ["active_type_elaboration"]
spec_refs = ["spec.en.test.type_elaboration.imported_structure_gap"]
"#,
    );
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.test.type_elaboration.imported_structure_gap"
source = "doc/spec/en/test.md"
section = "Test"
stage = "type_elaboration"
status = "covered"
required = true
coverage = "diagnostic"
tests = ["tests/miz/fail/types/fail_imported_structure_gap.expect.toml"]
"#,
    );
    corpus.write("doc/spec/en/test.md", "# Test\n");

    let report = run_type_elaboration_corpus(&corpus.config()).unwrap();

    assert_eq!(report.error_count(), 0, "{:#?}", report.diagnostics);
    assert_eq!(report.results.len(), 1);
    assert_eq!(report.passed_count(), 1);
    assert_eq!(
        report.results[0].actual_detail_keys,
        ["type_elaboration.checker.checker.declaration.deferred.evidence_query"]
    );
}

#[test]
fn type_elaboration_runner_bridges_imported_mode_heads_to_checker_gap() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/miz/fail/types/fail_imported_mode_gap.miz",
        "import parser.type_fixtures;\nreserve m for TypeCaseMode;\n",
    );
    corpus.write(
        "tests/miz/fail/types/fail_imported_mode_gap.expect.toml",
        r#"schema_version = 1
id = "fail_imported_mode_gap"
kind = "fail"
stage = "type_elaboration"
domain = "checker.type_elaboration"
source = "fail_imported_mode_gap.miz"
expected_outcome = "fail"
expected_phase = "type_check"
failure_category = "external_dependency_gap"
rejection_reason = "imported_mode_expansion_payload_gap"
stable_detail_key = "type_elaboration.checker.checker.type.external.mode_expansion_payload"
diagnostic_codes = []
diagnostic_payloads = [
  "type_elaboration.checker.checker.type.external.mode_expansion_payload",
  "type_elaboration.checker.checker.type.recovery",
]
tags = ["active_type_elaboration"]
spec_refs = ["spec.en.test.type_elaboration.imported_mode_gap"]
"#,
    );
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.test.type_elaboration.imported_mode_gap"
source = "doc/spec/en/test.md"
section = "Test"
stage = "type_elaboration"
status = "covered"
required = true
coverage = "diagnostic"
tests = ["tests/miz/fail/types/fail_imported_mode_gap.expect.toml"]
"#,
    );
    corpus.write("doc/spec/en/test.md", "# Test\n");

    let report = run_type_elaboration_corpus(&corpus.config()).unwrap();

    assert_eq!(report.error_count(), 0, "{:#?}", report.diagnostics);
    assert_eq!(report.results.len(), 1);
    assert_eq!(report.passed_count(), 1);
    assert_eq!(
        report.results[0].actual_detail_keys,
        [
            "type_elaboration.checker.checker.type.external.mode_expansion_payload",
            "type_elaboration.checker.checker.type.recovery",
        ]
    );
}

#[test]
fn type_elaboration_runner_keeps_bracket_mode_heads_on_external_gap() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/miz/fail/types/fail_bracket_mode_argument_gap.miz",
        "definition\n  mode FamilyDef: Family [a, b] is set;\nend;\n\nreserve f for Family[set];\n",
    );
    corpus.write(
        "tests/miz/fail/types/fail_bracket_mode_argument_gap.expect.toml",
        r#"schema_version = 1
id = "fail_bracket_mode_argument_gap"
kind = "fail"
stage = "type_elaboration"
domain = "checker.type_elaboration"
source = "fail_bracket_mode_argument_gap.miz"
expected_outcome = "fail"
expected_phase = "type_check"
failure_category = "external_dependency_gap"
rejection_reason = "bracket_mode_type_head_payload_gap"
stable_detail_key = "type_elaboration.external_dependency.ast_payload_extraction"
diagnostic_codes = []
diagnostic_payloads = [
  "type_elaboration.external_dependency.ast_payload_extraction",
]
tags = ["active_type_elaboration"]
spec_refs = ["spec.en.test.type_elaboration.bracket_mode_argument_gap"]
"#,
    );
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.test.type_elaboration.bracket_mode_argument_gap"
source = "doc/spec/en/test.md"
section = "Test"
stage = "type_elaboration"
status = "covered"
required = true
coverage = "diagnostic"
tests = ["tests/miz/fail/types/fail_bracket_mode_argument_gap.expect.toml"]
"#,
    );
    corpus.write("doc/spec/en/test.md", "# Test\n");

    let report = run_type_elaboration_corpus(&corpus.config()).unwrap();

    assert_eq!(report.error_count(), 0, "{:#?}", report.diagnostics);
    assert_eq!(report.results.len(), 1);
    assert_eq!(report.passed_count(), 1);
    assert_eq!(
        report.results[0].actual_detail_keys,
        ["type_elaboration.external_dependency.ast_payload_extraction"]
    );
}

#[test]
fn type_elaboration_runner_keeps_bracket_structure_heads_on_external_gap() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/miz/fail/types/fail_bracket_structure_argument_gap.miz",
        "definition\n  struct LocalStruct[r] where\n    field carrier -> set;\n  end;\nend;\n\nreserve s for LocalStruct[set];\n",
    );
    corpus.write(
        "tests/miz/fail/types/fail_bracket_structure_argument_gap.expect.toml",
        r#"schema_version = 1
id = "fail_bracket_structure_argument_gap"
kind = "fail"
stage = "type_elaboration"
domain = "checker.type_elaboration"
source = "fail_bracket_structure_argument_gap.miz"
expected_outcome = "fail"
expected_phase = "type_check"
failure_category = "external_dependency_gap"
rejection_reason = "bracket_structure_type_head_payload_gap"
stable_detail_key = "type_elaboration.external_dependency.ast_payload_extraction"
diagnostic_codes = []
diagnostic_payloads = [
  "type_elaboration.external_dependency.ast_payload_extraction",
]
tags = ["active_type_elaboration"]
spec_refs = ["spec.en.test.type_elaboration.bracket_structure_argument_gap"]
"#,
    );
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.test.type_elaboration.bracket_structure_argument_gap"
source = "doc/spec/en/test.md"
section = "Test"
stage = "type_elaboration"
status = "covered"
required = true
coverage = "diagnostic"
tests = ["tests/miz/fail/types/fail_bracket_structure_argument_gap.expect.toml"]
"#,
    );
    corpus.write("doc/spec/en/test.md", "# Test\n");

    let report = run_type_elaboration_corpus(&corpus.config()).unwrap();

    assert_eq!(report.error_count(), 0, "{:#?}", report.diagnostics);
    assert_eq!(report.results.len(), 1);
    assert_eq!(report.passed_count(), 1);
    assert_eq!(
        report.results[0].actual_detail_keys,
        ["type_elaboration.external_dependency.ast_payload_extraction"]
    );
}

#[test]
fn type_elaboration_runner_keeps_non_builtin_type_expressions_on_external_gap() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/miz/fail/types/fail_non_builtin_type_gap_001.miz",
        "import parser.type_fixtures;\nreserve x for T;\n",
    );
    corpus.write(
        "tests/miz/fail/types/fail_non_builtin_type_gap_001.expect.toml",
        r#"schema_version = 1
id = "fail_non_builtin_type_gap_001"
kind = "fail"
stage = "type_elaboration"
domain = "checker.type_elaboration"
source = "fail_non_builtin_type_gap_001.miz"
expected_outcome = "fail"
expected_phase = "type_check"
failure_category = "external_dependency_gap"
rejection_reason = "unsupported_type_expression_shape"
stable_detail_key = "type_elaboration.external_dependency.ast_payload_extraction"
diagnostic_codes = []
diagnostic_payloads = ["type_elaboration.external_dependency.ast_payload_extraction"]
tags = ["active_type_elaboration"]
spec_refs = ["spec.en.test.type_elaboration.gap"]
"#,
    );
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.test.type_elaboration.gap"
source = "doc/spec/en/test.md"
section = "Test"
stage = "type_elaboration"
status = "covered"
required = true
coverage = "diagnostic"
tests = ["tests/miz/fail/types/fail_non_builtin_type_gap_001.expect.toml"]
"#,
    );
    corpus.write("doc/spec/en/test.md", "# Test\n");

    let report = run_type_elaboration_corpus(&corpus.config()).unwrap();

    assert_eq!(report.error_count(), 0, "{:#?}", report.diagnostics);
    assert_eq!(report.results.len(), 1);
    assert_eq!(report.passed_count(), 1);
    assert_eq!(
        report.results[0].actual_detail_keys,
        ["type_elaboration.external_dependency.ast_payload_extraction"]
    );
}

#[test]
fn type_elaboration_runner_reports_lower_stage_symbol_failures_before_gap() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/miz/fail/types/fail_duplicate_theorem.miz",
        "theorem Clash: thesis;\ntheorem Clash: thesis;\n",
    );
    corpus.write(
        "tests/miz/fail/types/fail_duplicate_theorem.expect.toml",
        r#"schema_version = 1
id = "fail_duplicate_theorem"
kind = "fail"
stage = "type_elaboration"
domain = "checker.type_elaboration"
source = "fail_duplicate_theorem.miz"
expected_outcome = "fail"
expected_phase = "type_check"
failure_category = "lower_stage_error"
rejection_reason = "duplicate_theorem_symbol"
stable_detail_key = "type_elaboration.lower_stage.declaration_symbol.symbol.duplicate_declaration"
diagnostic_codes = []
tags = ["active_type_elaboration"]
spec_refs = ["spec.en.test.type_elaboration"]
"#,
    );
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.test.type_elaboration"
source = "doc/spec/en/test.md"
section = "Test"
stage = "type_elaboration"
status = "partial"
required = true
coverage = "diagnostic"
tests = ["tests/miz/fail/types/fail_duplicate_theorem.expect.toml"]
"#,
    );
    corpus.write("doc/spec/en/test.md", "# Test\n");

    let report = run_type_elaboration_corpus(&corpus.config()).unwrap();

    assert_eq!(report.error_count(), 0, "{:#?}", report.diagnostics);
    assert_eq!(report.results.len(), 1);
    assert_eq!(report.passed_count(), 1);
    assert_eq!(
        report.results[0].actual_detail_keys,
        ["type_elaboration.lower_stage.declaration_symbol.symbol.duplicate_declaration"]
    );
}

#[test]
fn type_elaboration_runner_keeps_forward_local_mode_references_on_lower_stage_gap() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/miz/fail/types/fail_forward_local_mode_reference.miz",
        "reserve z for LaterMode;\n\ndefinition\n  mode LaterModeDef: LaterMode is set;\nend;\n",
    );
    corpus.write(
        "tests/miz/fail/types/fail_forward_local_mode_reference.expect.toml",
        r#"schema_version = 1
id = "fail_forward_local_mode_reference"
kind = "fail"
stage = "type_elaboration"
domain = "checker.type_elaboration"
source = "fail_forward_local_mode_reference.miz"
expected_outcome = "fail"
expected_phase = "type_check"
failure_category = "lower_stage_error"
rejection_reason = "local_mode_forward_reference_active_range"
stable_detail_key = "type_elaboration.lower_stage.frontend:malformed_type_expression"
diagnostic_codes = []
diagnostic_payloads = [
  "type_elaboration.lower_stage.frontend:malformed_type_expression",
]
tags = ["active_type_elaboration"]
spec_refs = ["spec.en.test.type_elaboration.forward_local_mode_reference"]
"#,
    );
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.test.type_elaboration.forward_local_mode_reference"
source = "doc/spec/en/test.md"
section = "Test"
stage = "type_elaboration"
status = "covered"
required = true
coverage = "diagnostic"
tests = ["tests/miz/fail/types/fail_forward_local_mode_reference.expect.toml"]
"#,
    );
    corpus.write("doc/spec/en/test.md", "# Test\n");

    let report = run_type_elaboration_corpus(&corpus.config()).unwrap();

    assert_eq!(report.error_count(), 0, "{:#?}", report.diagnostics);
    assert_eq!(report.results.len(), 1);
    assert_eq!(report.passed_count(), 1);
    assert_eq!(
        report.results[0].actual_detail_keys,
        ["type_elaboration.lower_stage.frontend:malformed_type_expression"]
    );
}

#[test]
fn type_elaboration_runner_keeps_forward_local_structure_references_on_lower_stage_gap() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/miz/fail/types/fail_forward_local_structure_reference.miz",
        "reserve s for LaterStruct;\n\ndefinition\n  struct LaterStruct where\n    field carrier -> set;\n  end;\nend;\n",
    );
    corpus.write(
        "tests/miz/fail/types/fail_forward_local_structure_reference.expect.toml",
        r#"schema_version = 1
id = "fail_forward_local_structure_reference"
kind = "fail"
stage = "type_elaboration"
domain = "checker.type_elaboration"
source = "fail_forward_local_structure_reference.miz"
expected_outcome = "fail"
expected_phase = "type_check"
failure_category = "lower_stage_error"
rejection_reason = "local_structure_forward_reference_active_range"
stable_detail_key = "type_elaboration.lower_stage.frontend:malformed_type_expression"
diagnostic_codes = []
diagnostic_payloads = [
  "type_elaboration.lower_stage.frontend:malformed_type_expression",
]
tags = ["active_type_elaboration"]
spec_refs = ["spec.en.test.type_elaboration.forward_local_structure_reference"]
"#,
    );
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.test.type_elaboration.forward_local_structure_reference"
source = "doc/spec/en/test.md"
section = "Test"
stage = "type_elaboration"
status = "covered"
required = true
coverage = "diagnostic"
tests = ["tests/miz/fail/types/fail_forward_local_structure_reference.expect.toml"]
"#,
    );
    corpus.write("doc/spec/en/test.md", "# Test\n");

    let report = run_type_elaboration_corpus(&corpus.config()).unwrap();

    assert_eq!(report.error_count(), 0, "{:#?}", report.diagnostics);
    assert_eq!(report.results.len(), 1);
    assert_eq!(report.passed_count(), 1);
    assert_eq!(
        report.results[0].actual_detail_keys,
        ["type_elaboration.lower_stage.frontend:malformed_type_expression"]
    );
}

#[test]
fn type_elaboration_runner_keeps_forward_local_attribute_references_on_lower_stage_gap() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/miz/fail/types/fail_forward_local_attribute_reference.miz",
        "reserve x for marked set;\n\ndefinition\n  let x be set;\n  attr MarkedDef: x is marked means thesis;\nend;\n",
    );
    corpus.write(
        "tests/miz/fail/types/fail_forward_local_attribute_reference.expect.toml",
        r#"schema_version = 1
id = "fail_forward_local_attribute_reference"
kind = "fail"
stage = "type_elaboration"
domain = "checker.type_elaboration"
source = "fail_forward_local_attribute_reference.miz"
expected_outcome = "fail"
expected_phase = "type_check"
failure_category = "lower_stage_error"
rejection_reason = "local_attribute_forward_reference_active_range"
stable_detail_key = "type_elaboration.lower_stage.frontend:malformed_type_expression"
diagnostic_codes = []
diagnostic_payloads = [
  "type_elaboration.lower_stage.frontend:malformed_type_expression",
]
tags = ["active_type_elaboration"]
spec_refs = ["spec.en.test.type_elaboration.forward_local_attribute_reference"]
"#,
    );
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.test.type_elaboration.forward_local_attribute_reference"
source = "doc/spec/en/test.md"
section = "Test"
stage = "type_elaboration"
status = "covered"
required = true
coverage = "diagnostic"
tests = ["tests/miz/fail/types/fail_forward_local_attribute_reference.expect.toml"]
"#,
    );
    corpus.write("doc/spec/en/test.md", "# Test\n");

    let report = run_type_elaboration_corpus(&corpus.config()).unwrap();

    assert_eq!(report.error_count(), 0, "{:#?}", report.diagnostics);
    assert_eq!(report.results.len(), 1);
    assert_eq!(report.passed_count(), 1);
    assert_eq!(
        report.results[0].actual_detail_keys,
        ["type_elaboration.lower_stage.frontend:malformed_type_expression"]
    );
}

#[test]
fn type_elaboration_runner_rejects_active_tag_on_wrong_phase() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/miz/fail/types/fail_payload_gap.miz",
        "theorem VisibleTheorem: thesis;\n",
    );
    corpus.write(
        "tests/miz/fail/types/fail_payload_gap.expect.toml",
        r#"schema_version = 1
id = "fail_payload_gap"
kind = "fail"
stage = "type_elaboration"
domain = "checker.type_elaboration"
source = "fail_payload_gap.miz"
expected_outcome = "fail"
expected_phase = "resolve"
failure_category = "external_dependency_gap"
rejection_reason = "missing_ast_payload_extraction"
stable_detail_key = "type_elaboration.external_dependency.ast_payload_extraction"
diagnostic_codes = []
tags = ["active_type_elaboration"]
spec_refs = ["spec.en.test.type_elaboration"]
"#,
    );
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.test.type_elaboration"
source = "doc/spec/en/test.md"
section = "Test"
stage = "type_elaboration"
status = "partial"
required = true
coverage = "diagnostic"
tests = ["tests/miz/fail/types/fail_payload_gap.expect.toml"]
"#,
    );
    corpus.write("doc/spec/en/test.md", "# Test\n");

    let report = run_type_elaboration_corpus(&corpus.config()).unwrap();

    assert_eq!(report.results.len(), 0);
    assert_has_type_elaboration_report_code(&report, "E-TYPE-ELABORATION-ACTIVE-GATE");
}

#[test]
fn declaration_symbol_runner_uses_stable_detail_key_fallback() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/miz/fail/resolve/fail_duplicate_theorem.miz",
        "theorem Clash: thesis;\ntheorem Clash: thesis;\n",
    );
    corpus.write(
        "tests/miz/fail/resolve/fail_duplicate_theorem.expect.toml",
        r#"schema_version = 1
id = "fail_duplicate_theorem"
kind = "fail"
stage = "declaration_symbol"
domain = "resolve.declaration_symbol"
source = "fail_duplicate_theorem.miz"
expected_outcome = "fail"
expected_phase = "resolve"
failure_category = "resolve_error"
rejection_reason = "duplicate_theorem_symbol"
stable_detail_key = "declaration_symbol.symbol.duplicate_declaration"
diagnostic_codes = []
tags = ["active_declaration_symbol"]
spec_refs = ["spec.en.test.resolve"]
"#,
    );
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.test.resolve"
source = "doc/spec/en/test.md"
section = "Test"
stage = "declaration_symbol"
status = "partial"
required = true
coverage = "manual_review"
tests = ["tests/miz/fail/resolve/fail_duplicate_theorem.expect.toml"]
"#,
    );
    corpus.write("doc/spec/en/test.md", "# Test\n");

    let report = run_declaration_symbol_corpus(&corpus.config()).unwrap();

    assert_eq!(report.error_count(), 0, "{:#?}", report.diagnostics);
    assert_eq!(report.results.len(), 1);
    assert_eq!(report.passed_count(), 1);
}

#[test]
fn declaration_symbol_runner_records_parameterized_local_attribute_suffix() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/miz/pass/resolve/pass_parameterized_local_attribute_001.miz",
        "definition\n  let x be set;\n  attr RankedDef: x is 2-ranked means thesis;\nend;\n",
    );
    corpus.write(
        "tests/miz/pass/resolve/pass_parameterized_local_attribute_001.expect.toml",
        r#"schema_version = 1
id = "pass_parameterized_local_attribute_001"
kind = "pass"
stage = "declaration_symbol"
domain = "resolve.declaration_symbol"
source = "pass_parameterized_local_attribute_001.miz"
expected_outcome = "pass"
expected_phase = "resolve"
diagnostic_codes = []
declaration_symbol_payloads = [
  "declaration_symbol.definition.kind.ranked.attribute",
  "declaration_symbol.definition.visibility.ranked.public",
  "declaration_symbol.symbol.export.ranked.exported",
  "declaration_symbol.symbol.kind.ranked.attribute",
  "declaration_symbol.symbol.visibility.ranked.public",
]
tags = ["active_declaration_symbol"]
spec_refs = ["spec.en.test.resolve.parameterized_local_attribute"]
"#,
    );
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.test.resolve.parameterized_local_attribute"
source = "doc/spec/en/test.md"
section = "Test"
stage = "declaration_symbol"
status = "covered"
required = true
coverage = "pass"
tests = ["tests/miz/pass/resolve/pass_parameterized_local_attribute_001.expect.toml"]
"#,
    );
    corpus.write("doc/spec/en/test.md", "# Test\n");

    let report = run_declaration_symbol_corpus(&corpus.config()).unwrap();

    assert_eq!(report.error_count(), 0, "{:#?}", report.diagnostics);
    assert_eq!(report.results.len(), 1);
    assert_eq!(report.passed_count(), 1);
    assert_eq!(
        report.results[0].actual_payload_keys,
        [
            "declaration_symbol.definition.kind.ranked.attribute",
            "declaration_symbol.definition.visibility.ranked.public",
            "declaration_symbol.symbol.export.ranked.exported",
            "declaration_symbol.symbol.kind.ranked.attribute",
            "declaration_symbol.symbol.visibility.ranked.public",
        ]
    );
}

#[test]
fn declaration_symbol_runner_compares_payloads_exactly_and_sorts_expectations() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/miz/pass/resolve/pass_payload_unsorted.miz",
        "theorem VisibleTheorem: thesis;\n",
    );
    corpus.write(
        "tests/miz/pass/resolve/pass_payload_unsorted.expect.toml",
        r#"schema_version = 1
id = "pass_payload_unsorted"
kind = "pass"
stage = "declaration_symbol"
domain = "resolve.declaration_symbol"
source = "pass_payload_unsorted.miz"
expected_outcome = "pass"
expected_phase = "resolve"
diagnostic_codes = []
declaration_symbol_payloads = [
  "declaration_symbol.symbol.visibility.VisibleTheorem.public",
  "declaration_symbol.symbol.kind.VisibleTheorem.theorem",
  "declaration_symbol.symbol.export.VisibleTheorem.exported",
  "declaration_symbol.definition.visibility.VisibleTheorem.public",
  "declaration_symbol.definition.kind.VisibleTheorem.theorem",
]
tags = ["active_declaration_symbol"]
spec_refs = ["spec.en.test.resolve"]
"#,
    );
    corpus.write(
        "tests/miz/pass/resolve/pass_payload_mismatch.miz",
        "theorem VisibleTheorem: thesis;\n",
    );
    corpus.write(
        "tests/miz/pass/resolve/pass_payload_mismatch.expect.toml",
        r#"schema_version = 1
id = "pass_payload_mismatch"
kind = "pass"
stage = "declaration_symbol"
domain = "resolve.declaration_symbol"
source = "pass_payload_mismatch.miz"
expected_outcome = "pass"
expected_phase = "resolve"
diagnostic_codes = []
declaration_symbol_payloads = [
  "declaration_symbol.symbol.kind.VisibleTheorem.theorem",
]
tags = ["active_declaration_symbol"]
spec_refs = ["spec.en.test.resolve"]
"#,
    );
    corpus.write(
        "tests/miz/pass/resolve/pass_payload_extra.miz",
        "theorem VisibleTheorem: thesis;\n",
    );
    corpus.write(
        "tests/miz/pass/resolve/pass_payload_extra.expect.toml",
        r#"schema_version = 1
id = "pass_payload_extra"
kind = "pass"
stage = "declaration_symbol"
domain = "resolve.declaration_symbol"
source = "pass_payload_extra.miz"
expected_outcome = "pass"
expected_phase = "resolve"
diagnostic_codes = []
declaration_symbol_payloads = [
  "declaration_symbol.definition.kind.VisibleTheorem.theorem",
  "declaration_symbol.definition.visibility.VisibleTheorem.public",
  "declaration_symbol.symbol.export.VisibleTheorem.exported",
  "declaration_symbol.symbol.kind.VisibleTheorem.theorem",
  "declaration_symbol.symbol.kind.MissingTheorem.theorem",
  "declaration_symbol.symbol.visibility.VisibleTheorem.public",
]
tags = ["active_declaration_symbol"]
spec_refs = ["spec.en.test.resolve"]
"#,
    );
    corpus.write(
        "tests/miz/pass/resolve/pass_payload_escaped.miz",
        "import parser.type_fixtures;\n\ndefinition\n  let x, y be set;\n  func PlusDef: x ++ y -> set equals x;\nend;\n",
    );
    corpus.write(
        "tests/miz/pass/resolve/pass_payload_escaped.expect.toml",
        r#"schema_version = 1
id = "pass_payload_escaped"
kind = "pass"
stage = "declaration_symbol"
domain = "resolve.declaration_symbol"
source = "pass_payload_escaped.miz"
expected_outcome = "pass"
expected_phase = "resolve"
diagnostic_codes = []
declaration_symbol_payloads = [
  "declaration_symbol.definition.kind.x%20%2B%2B%20y.functor",
  "declaration_symbol.definition.visibility.x%20%2B%2B%20y.public",
  "declaration_symbol.symbol.export.x%20%2B%2B%20y.exported",
  "declaration_symbol.symbol.kind.x%20%2B%2B%20y.functor",
  "declaration_symbol.symbol.visibility.x%20%2B%2B%20y.public",
]
tags = ["active_declaration_symbol"]
spec_refs = ["spec.en.test.resolve"]
"#,
    );
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.test.resolve"
source = "doc/spec/en/test.md"
section = "Test"
stage = "declaration_symbol"
status = "partial"
required = true
coverage = "manual_review"
tests = [
  "tests/miz/pass/resolve/pass_payload_unsorted.expect.toml",
  "tests/miz/pass/resolve/pass_payload_mismatch.expect.toml",
  "tests/miz/pass/resolve/pass_payload_extra.expect.toml",
  "tests/miz/pass/resolve/pass_payload_escaped.expect.toml",
]
"#,
    );
    corpus.write("doc/spec/en/test.md", "# Test\n");

    let report = run_declaration_symbol_corpus(&corpus.config()).unwrap();

    assert_eq!(report.results.len(), 4, "{report:#?}");
    assert_eq!(report.passed_count(), 2, "{report:#?}");
    assert_eq!(report.failed_count(), 2, "{report:#?}");
    assert_has_declaration_symbol_report_code(&report, "E-DECLARATION-SYMBOL-ASSERT");
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_payload_unsorted"
            && result.status == mizar_test::DeclarationSymbolCaseStatus::Passed
            && result.actual_payload_keys == visible_theorem_declaration_symbol_payloads()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_payload_escaped"
            && result.status == mizar_test::DeclarationSymbolCaseStatus::Passed
            && result.actual_payload_keys == plus_functor_declaration_symbol_payloads()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_payload_mismatch"
            && result.status == mizar_test::DeclarationSymbolCaseStatus::Failed
            && result.actual_payload_keys == visible_theorem_declaration_symbol_payloads()
    }));
    assert!(report.results.iter().any(|result| {
        result.id.0 == "pass_payload_extra"
            && result.status == mizar_test::DeclarationSymbolCaseStatus::Failed
            && result.actual_payload_keys == visible_theorem_declaration_symbol_payloads()
    }));
}

#[test]
fn plan_cli_reports_deterministic_metadata_summary() {
    let corpus = Corpus::new();

    let output = plan_cli(&corpus)
        .output()
        .expect("mizar-test plan should run");

    assert!(
        output.status.success(),
        "plan CLI failed: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "test cases: 0\nrequirements: 0\nerrors: 0\nwarnings: 0\ncoverage stages: 0\npass/fail mix: pass=0 fail=0 total=0 target_pass=40 target_fail=60\narchitecture22 matrix: scenarios=18 planned=0 active=0 missing=18\n"
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn plan_cli_accepts_validation_modes() {
    for mode in ["metadata", "development", "release"] {
        let corpus = Corpus::new();

        let output = plan_cli(&corpus)
            .arg("--validation-mode")
            .arg(mode)
            .output()
            .unwrap_or_else(|error| panic!("mizar-test plan {mode} should run: {error}"));

        assert!(
            output.status.success(),
            "plan CLI mode {mode} failed: stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn plan_cli_warnings_exit_success() {
    let corpus = Corpus::new();
    corpus.add_requirement("spec.en.test.planned", &[]);

    let output = plan_cli(&corpus)
        .output()
        .expect("mizar-test plan with warnings should run");

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "test cases: 0\nrequirements: 1\nerrors: 0\nwarnings: 1\ncoverage stages: 1\ncoverage stage lexical: requirements=1 covered=0 partial=0 planned=1 deferred=0 obsolete=0 missing_shapes=1\npass/fail mix: pass=0 fail=0 total=0 target_pass=40 target_fail=60\narchitecture22 matrix: scenarios=18 planned=0 active=0 missing=18\n"
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("W-MANIFEST-PLANNED-NO-TESTS"));
}

#[test]
fn plan_cli_reports_deterministic_coverage_and_pass_fail_mix() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.cli.pass_fail"
source = "doc/spec/en/test.md"
section = "Test"
stage = "lexical"
status = "covered"
required = true
coverage = "pass_and_fail"
tests = [
  "tests/lexical/pass/pass_cli_001.expect.toml",
  "tests/lexical/fail/fail_cli_001.expect.toml",
]
"#,
    );
    corpus.write("doc/spec/en/test.md", "# Test\n");
    corpus.write("tests/lexical/pass/pass_cli_001.src", "alpha");
    corpus.write(
        "tests/lexical/pass/pass_cli_001.expect.toml",
        pass_expectation("pass_cli_001", "pass_cli_001.src", "spec.en.cli.pass_fail"),
    );
    corpus.write("tests/lexical/fail/fail_cli_001.src", "bad");
    corpus.write(
        "tests/lexical/fail/fail_cli_001.expect.toml",
        fail_expectation("fail_cli_001", "fail_cli_001.src", "spec.en.cli.pass_fail"),
    );

    let output = plan_cli(&corpus)
        .output()
        .expect("mizar-test plan with coverage report should run");

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "test cases: 2\nrequirements: 1\nerrors: 0\nwarnings: 0\ncoverage stages: 1\ncoverage stage lexical: requirements=1 covered=1 partial=0 planned=0 deferred=0 obsolete=0 missing_shapes=0\npass/fail mix: pass=1 fail=1 total=2 target_pass=40 target_fail=60\narchitecture22 matrix: scenarios=18 planned=0 active=0 missing=18\n"
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn plan_cli_reports_architecture22_matrix_rows() {
    let corpus = Corpus::new();
    corpus.write(
        "tests/coverage/spec_trace.toml",
        r#"
[[requirement]]
id = "spec.en.architecture_22.regression_matrix.metadata"
source = "doc/design/architecture/en/22.incremental_verification_contract.md"
section = "Regression matrix metadata"
stage = "advanced_semantics"
status = "partial"
required = true
coverage = "manual_review"
tests = ["tests/property/architecture22_matrix_001.expect.toml"]
"#,
    );
    corpus.write(
        "doc/design/architecture/en/22.incremental_verification_contract.md",
        "# Incremental Verification Contract\n",
    );
    corpus.write(
        "tests/property/architecture22_matrix_001.fixture.toml",
        "matrix = \"architecture22\"\n",
    );
    corpus.write(
        "tests/property/architecture22_matrix_001.expect.toml",
        architecture22_matrix_expectation(),
    );

    let output = plan_cli(&corpus)
        .output()
        .expect("mizar-test plan with architecture22 matrix should run");

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("architecture22 matrix: scenarios=18 planned=18 active=0 missing=0\n"));
    assert!(stdout.contains(
        "architecture22 scenario artifact_manifest_atomicity: class=atomic_publication planned=1 active=0\n"
    ));
    assert!(stdout.contains(
        "architecture22 scenario theorem_proof_body_invalidation: class=local_refresh_only planned=1 active=0\n"
    ));
    assert!(stdout.contains(
        "architecture22 scenario vcid_reorder_anchor_reuse: class=reuse_requires_full_identity planned=1 active=0\n"
    ));
    assert_eq!(stdout.matches("architecture22 scenario ").count(), 18);
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn plan_cli_validation_errors_exit_one() {
    let corpus = Corpus::new();
    corpus.write("tests/lexical/pass/orphan.src", "");

    let output = plan_cli(&corpus)
        .output()
        .expect("mizar-test plan with validation errors should run");

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "test cases: 0\nrequirements: 0\nerrors: 1\nwarnings: 0\ncoverage stages: 0\npass/fail mix: pass=0 fail=0 total=0 target_pass=40 target_fail=60\narchitecture22 matrix: scenarios=18 planned=0 active=0 missing=18\n"
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("E-LAYOUT-MISSING-SIDECAR"));
}

#[test]
fn plan_cli_release_unknown_roots_exit_one() {
    let corpus = Corpus::new();
    corpus.write("tests/experimental/README.md", "not a corpus root\n");

    let output = plan_cli(&corpus)
        .arg("--validation-mode")
        .arg("release")
        .output()
        .expect("mizar-test plan release with unknown root should run");

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "test cases: 0\nrequirements: 0\nerrors: 1\nwarnings: 0\ncoverage stages: 0\npass/fail mix: pass=0 fail=0 total=0 target_pass=40 target_fail=60\narchitecture22 matrix: scenarios=18 planned=0 active=0 missing=18\n"
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("E-LAYOUT-UNKNOWN-ROOT"));
}

#[test]
fn plan_cli_usage_and_infrastructure_errors_exit_two() {
    let corpus = Corpus::new();

    let bad_mode = plan_cli(&corpus)
        .arg("--validation-mode")
        .arg("strict")
        .output()
        .expect("mizar-test plan with bad validation mode should run");
    assert_eq!(bad_mode.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&bad_mode.stderr).contains("unknown validation mode `strict`"));

    let missing_root = plan_cli(&corpus)
        .arg("--tests-root")
        .arg("missing-tests")
        .output()
        .expect("mizar-test plan with missing root should run");
    assert_eq!(missing_root.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&missing_root.stderr).contains("is not a directory"),
        "stderr={}",
        String::from_utf8_lossy(&missing_root.stderr)
    );
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
fn declaration_symbol_cli_reports_active_runner_summary() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_mizar-test"))
        .arg("declaration-symbol")
        .arg("--workspace-root")
        .arg(repository_config().workspace_root)
        .output()
        .expect("mizar-test declaration-symbol should run");

    assert!(
        output.status.success(),
        "declaration-symbol CLI failed: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("declaration-symbol cases: 4"));
    assert!(stdout.contains("passed: 4"));
    assert!(stdout.contains("failed: 0"));
}

#[test]
fn type_elaboration_cli_reports_active_runner_summary() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_mizar-test"))
        .arg("type-elaboration")
        .arg("--workspace-root")
        .arg(repository_config().workspace_root)
        .output()
        .expect("mizar-test type-elaboration should run");

    assert!(
        output.status.success(),
        "type-elaboration CLI failed: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("type-elaboration cases: 172"));
    assert!(stdout.contains("passed: 172"));
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
fn declaration_symbol_payload_expectations_parse_and_validate_scope() {
    let corpus = Corpus::new();
    corpus.add_requirement("spec.en.test.resolve", &[]);
    corpus.write(
        "tests/miz/pass/resolve/pass_payloads.miz",
        "theorem VisibleTheorem: thesis;\n",
    );
    corpus.write(
        "tests/miz/pass/resolve/pass_payloads.expect.toml",
        r#"schema_version = 1
id = "pass_payloads"
kind = "pass"
stage = "declaration_symbol"
domain = "resolve.declaration_symbol"
source = "pass_payloads.miz"
expected_outcome = "pass"
expected_phase = "resolve"
diagnostic_codes = []
declaration_symbol_payloads = ["declaration_symbol.symbol.kind.VisibleTheorem.theorem"]
tags = ["active_declaration_symbol"]
spec_refs = ["spec.en.test.resolve"]
"#,
    );

    let plan = corpus.plan();

    assert_eq!(plan.error_count(), 0, "{:#?}", plan.diagnostics);
    assert_eq!(
        plan.cases[0].expectation.declaration_symbol_payloads,
        vec!["declaration_symbol.symbol.kind.VisibleTheorem.theorem"]
    );

    let corpus = Corpus::new();
    corpus.add_requirement("spec.en.test.resolve", &[]);
    corpus.write(
        "tests/miz/pass/resolve/empty_payload.miz",
        "theorem VisibleTheorem: thesis;\n",
    );
    corpus.write(
        "tests/miz/pass/resolve/empty_payload.expect.toml",
        r#"schema_version = 1
id = "empty_payload"
kind = "pass"
stage = "declaration_symbol"
domain = "resolve.declaration_symbol"
source = "empty_payload.miz"
expected_outcome = "pass"
expected_phase = "resolve"
diagnostic_codes = []
declaration_symbol_payloads = [""]
tags = ["active_declaration_symbol"]
spec_refs = ["spec.en.test.resolve"]
"#,
    );

    let plan = corpus.plan();

    assert_has_code(&plan, "E-EXPECT-SCHEMA");

    let corpus = Corpus::new();
    corpus.add_requirement("spec.en.test.resolve", &[]);
    corpus.write(
        "tests/miz/pass/resolve/missing_active_tag_payload.miz",
        "theorem VisibleTheorem: thesis;\n",
    );
    corpus.write(
        "tests/miz/pass/resolve/missing_active_tag_payload.expect.toml",
        r#"schema_version = 1
id = "missing_active_tag_payload"
kind = "pass"
stage = "declaration_symbol"
domain = "resolve.declaration_symbol"
source = "missing_active_tag_payload.miz"
expected_outcome = "pass"
expected_phase = "resolve"
diagnostic_codes = []
declaration_symbol_payloads = ["declaration_symbol.symbol.kind.VisibleTheorem.theorem"]
spec_refs = ["spec.en.test.resolve"]
"#,
    );

    let plan = corpus.plan();

    assert_has_code(&plan, "E-EXPECT-SCHEMA");

    let corpus = Corpus::new();
    corpus.add_requirement("spec.en.test.resolve", &[]);
    corpus.write(
        "tests/miz/fail/resolve/fail_payload_scope.miz",
        "theorem Clash: thesis;\ntheorem Clash: thesis;\n",
    );
    corpus.write(
        "tests/miz/fail/resolve/fail_payload_scope.expect.toml",
        r#"schema_version = 1
id = "fail_payload_scope"
kind = "fail"
stage = "declaration_symbol"
domain = "resolve.declaration_symbol"
source = "fail_payload_scope.miz"
expected_outcome = "fail"
expected_phase = "resolve"
failure_category = "resolve_error"
rejection_reason = "duplicate_theorem_symbol"
stable_detail_key = "declaration_symbol.symbol.duplicate_declaration"
diagnostic_codes = []
declaration_symbol_payloads = ["declaration_symbol.symbol.kind.Clash.theorem"]
tags = ["active_declaration_symbol"]
spec_refs = ["spec.en.test.resolve"]
"#,
    );

    let plan = corpus.plan();

    assert_has_code(&plan, "E-EXPECT-SCHEMA");

    let corpus = Corpus::new();
    corpus.add_requirement("spec.en.test.resolve", &[]);
    corpus.write(
        "tests/miz/pass/resolve/wrong_phase_payload.miz",
        "theorem VisibleTheorem: thesis;\n",
    );
    corpus.write(
        "tests/miz/pass/resolve/wrong_phase_payload.expect.toml",
        r#"schema_version = 1
id = "wrong_phase_payload"
kind = "pass"
stage = "declaration_symbol"
domain = "resolve.declaration_symbol"
source = "wrong_phase_payload.miz"
expected_outcome = "pass"
expected_phase = "lex"
diagnostic_codes = []
declaration_symbol_payloads = ["declaration_symbol.symbol.kind.VisibleTheorem.theorem"]
tags = ["active_declaration_symbol"]
spec_refs = ["spec.en.test.resolve"]
"#,
    );

    let plan = corpus.plan();

    assert_has_code(&plan, "E-EXPECT-SCHEMA");

    let corpus = Corpus::new();
    corpus.add_requirement("spec.en.test.resolve", &[]);
    corpus.write(
        "tests/miz/pass/resolve/wrong_source_payload.src",
        "theorem VisibleTheorem: thesis;\n",
    );
    corpus.write(
        "tests/miz/pass/resolve/wrong_source_payload.expect.toml",
        r#"schema_version = 1
id = "wrong_source_payload"
kind = "pass"
stage = "declaration_symbol"
domain = "resolve.declaration_symbol"
source = "wrong_source_payload.src"
expected_outcome = "pass"
expected_phase = "resolve"
diagnostic_codes = []
declaration_symbol_payloads = ["declaration_symbol.symbol.kind.VisibleTheorem.theorem"]
tags = ["active_declaration_symbol"]
spec_refs = ["spec.en.test.resolve"]
"#,
    );

    let plan = corpus.plan();

    assert_has_code(&plan, "E-EXPECT-SCHEMA");

    let corpus = Corpus::new();
    corpus.add_requirement("spec.en.test.resolve", &[]);
    corpus.write(
        "tests/miz/pass/resolve/wrong_stage_payload.miz",
        "theorem VisibleTheorem: thesis;\n",
    );
    corpus.write(
        "tests/miz/pass/resolve/wrong_stage_payload.expect.toml",
        r#"schema_version = 1
id = "wrong_stage_payload"
kind = "pass"
stage = "parse_only"
domain = "resolve.declaration_symbol"
source = "wrong_stage_payload.miz"
expected_outcome = "pass"
expected_phase = "resolve"
diagnostic_codes = []
declaration_symbol_payloads = ["declaration_symbol.symbol.kind.VisibleTheorem.theorem"]
tags = ["active_declaration_symbol"]
spec_refs = ["spec.en.test.resolve"]
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
            "tests/stress",
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
    pass_expectation(id, source, spec_ref)
}

fn pass_expectation(id: &str, source: &str, spec_ref: &str) -> String {
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

fn fail_expectation(id: &str, source: &str, spec_ref: &str) -> String {
    format!(
        r#"schema_version = 1
id = "{id}"
kind = "fail"
stage = "lexical"
domain = "lexical"
source = "{source}"
expected_outcome = "fail"
expected_phase = "lex"
failure_category = "lex_error"
stable_detail_key = "lexical.synthetic"
diagnostic_codes = []
diagnostic_payloads = ["lexical.synthetic"]
spec_refs = ["{spec_ref}"]
"#
    )
}

fn parse_pass_expectation(id: &str, source: &str, spec_ref: &str) -> String {
    format!(
        r#"schema_version = 1
id = "{id}"
kind = "pass"
stage = "parse_only"
domain = "parser"
source = "{source}"
expected_outcome = "pass"
expected_phase = "parse"
diagnostic_codes = []
spec_refs = ["{spec_ref}"]
"#
    )
}

fn generated_expectation(
    id: &str,
    source: &str,
    spec_ref: &str,
    profile_line: &str,
    minimized: bool,
) -> String {
    let profile_line = optional_line(profile_line);
    format!(
        r#"schema_version = 1
id = "{id}"
kind = "generated"
stage = "parse_only"
domain = "parser"
source = "{source}"
expected_outcome = "pass"
expected_phase = "parse"
diagnostic_codes = []
spec_refs = ["{spec_ref}"]
{profile_line}
[origin]
schema_version = 1
kind = "generated"
generator = "grammar-smoke"
generator_version = "0.1.0"
seed = "{id}"
profile = "parser"
expected_outcome = "pass"
minimized = {minimized}
"#
    )
}

fn fuzz_seed_expectation(
    id: &str,
    source: &str,
    spec_ref: &str,
    profile_line: &str,
    original_failure_category: Option<&str>,
) -> String {
    let profile_line = optional_line(profile_line);
    let original_failure_category = original_failure_category
        .map(|category| format!("original_failure_category = \"{category}\"\n"))
        .unwrap_or_default();
    format!(
        r#"schema_version = 1
id = "{id}"
kind = "fuzz_seed"
stage = "lexical"
domain = "lexical"
source = "{source}"
expected_outcome = "metadata_only"
diagnostic_codes = []
spec_refs = ["{spec_ref}"]
{profile_line}
[origin]
schema_version = 1
kind = "fuzz_seed"
generator = "cargo-fuzz"
generator_version = "0.1.0"
seed = "{id}"
profile = "lexical"
expected_outcome = "metadata_only"
minimized = true
{original_failure_category}"#
    )
}

fn property_seed_expectation(
    id: &str,
    source: &str,
    spec_ref: &str,
    profile_line: &str,
    minimized: bool,
) -> String {
    property_seed_expectation_with_extra(
        id,
        source,
        spec_ref,
        profile_line,
        minimized,
        "diagnostic_codes = []",
    )
}

fn property_seed_expectation_with_extra(
    id: &str,
    source: &str,
    spec_ref: &str,
    profile_line: &str,
    minimized: bool,
    extra: &str,
) -> String {
    let profile_line = optional_line(profile_line);
    let extra = optional_line(extra);
    format!(
        r#"schema_version = 1
id = "{id}"
kind = "property_seed"
stage = "lexical"
domain = "lexical"
source = "{source}"
expected_outcome = "metadata_only"
{extra}spec_refs = ["{spec_ref}"]
{profile_line}[origin]
schema_version = 1
kind = "property_seed"
generator = "proptest"
generator_version = "0.1.0"
seed = "{id}"
profile = "lexical"
expected_outcome = "metadata_only"
minimized = {minimized}
"#
    )
}

fn architecture22_matrix_expectation() -> String {
    let scenarios = architecture22_scenario_specs()
        .iter()
        .map(|scenario| format!("  \"{}\",", scenario.id))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        r#"schema_version = 1
id = "architecture22_matrix_001"
kind = "property_seed"
stage = "advanced_semantics"
domain = "incremental_verification"
source = "architecture22_matrix_001.fixture.toml"
expected_outcome = "metadata_only"
profiles = ["full"]
diagnostic_codes = []
spec_refs = ["{spec_ref}"]
architecture22_gate = "planned"
architecture22_scenarios = [
{scenarios}
]

[origin]
schema_version = 1
kind = "property_seed"
generator = "mizar-test-architecture22-matrix"
generator_version = "1"
seed = "architecture22_matrix_001"
profile = "architecture22-regression-matrix"
expected_outcome = "metadata_only"
minimized = true
"#,
        spec_ref = "spec.en.architecture_22.regression_matrix.metadata"
    )
}

fn optional_line(line: &str) -> String {
    if line.is_empty() {
        String::new()
    } else {
        format!("{line}\n")
    }
}

fn five_line_miz() -> &'static str {
    "reserve x for set;\nreserve y for set;\ntheorem x = x;\nproof\nend;\n"
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

fn assert_lacks_code(plan: &mizar_test::TestPlan, code: &str) {
    assert!(
        !plan
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code.0 == code),
        "unexpected diagnostic {code}, got {:#?}",
        plan.diagnostics
    );
}

fn assert_has_message(plan: &mizar_test::TestPlan, needle: &str) {
    assert!(
        plan.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains(needle)),
        "expected diagnostic containing {needle:?}, got {:#?}",
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

fn assert_has_declaration_symbol_report_code(
    report: &mizar_test::DeclarationSymbolRunReport,
    code: &str,
) {
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code.0 == code),
        "expected diagnostic {code}, got {:#?}",
        report.diagnostics
    );
}

fn assert_has_type_elaboration_report_code(
    report: &mizar_test::TypeElaborationRunReport,
    code: &str,
) {
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code.0 == code),
        "expected diagnostic {code}, got {:#?}",
        report.diagnostics
    );
}

fn canonical_test_plan(plan: &TestPlan, root: &Path) -> String {
    let mut output = String::new();
    writeln!(output, "cases={}", plan.cases.len()).unwrap();
    for case in &plan.cases {
        writeln!(
            output,
            "case|{}|{}|{}|{}|{}|{}|{}|profiles={}|tags={}|refs={}",
            case.id.0,
            rel_string(root, &case.expectation_path),
            rel_string(root, &case.source_path),
            case.expectation.kind,
            case.expectation.stage.as_str(),
            case.expectation.expected_outcome,
            case.expectation
                .expected_phase
                .map(|phase| phase.as_str())
                .unwrap_or("<none>"),
            case.expectation.profiles.join(","),
            case.expectation.tags.join(","),
            case.expectation
                .spec_refs
                .iter()
                .map(|spec_ref| spec_ref.0.as_str())
                .collect::<Vec<_>>()
                .join(",")
        )
        .unwrap();
    }

    writeln!(
        output,
        "pass-fail|{}|{}|{}|{}|{}",
        plan.coverage_report.pass_fail_mix.pass,
        plan.coverage_report.pass_fail_mix.fail,
        plan.coverage_report.pass_fail_mix.total,
        plan.coverage_report.pass_fail_mix.target_pass_percent,
        plan.coverage_report.pass_fail_mix.target_fail_percent
    )
    .unwrap();
    let matrix = &plan.coverage_report.architecture22_matrix;
    writeln!(
        output,
        "architecture22|scenarios={}|planned={}|active={}|missing={}",
        matrix.scenarios.len(),
        matrix
            .scenarios
            .iter()
            .map(|scenario| scenario.planned)
            .sum::<usize>(),
        matrix
            .scenarios
            .iter()
            .map(|scenario| scenario.active)
            .sum::<usize>(),
        matrix.missing_scenarios.len()
    )
    .unwrap();
    for scenario in &matrix.scenarios {
        if scenario.planned == 0 && scenario.active == 0 {
            continue;
        }
        writeln!(
            output,
            "architecture22-row|{}|{}|{}|{}",
            scenario.scenario_id, scenario.equivalence_class, scenario.planned, scenario.active
        )
        .unwrap();
    }
    for requirement in &plan.coverage_report.requirements {
        writeln!(
            output,
            "coverage|{}|{}|{}|{}|{}|{}|evidence={}/{}/{}/{}/{}/{}|missing={}",
            requirement.id.0,
            requirement.stage.as_str(),
            requirement.coverage.as_str(),
            requirement.required,
            requirement.stored_status.as_str(),
            requirement.computed_status.as_str(),
            requirement.evidence.pass,
            requirement.evidence.fail,
            requirement.evidence.diagnostic,
            requirement.evidence.snapshot,
            requirement.evidence.property,
            requirement.evidence.manual_review,
            requirement
                .missing_shapes
                .iter()
                .map(|shape| shape.as_str())
                .collect::<Vec<_>>()
                .join(",")
        )
        .unwrap();
    }
    for stage in &plan.coverage_report.stages {
        writeln!(
            output,
            "stage|{}|{}|{}|{}|{}|{}|{}|{}",
            stage.stage.as_str(),
            stage.requirements,
            stage.covered,
            stage.partial,
            stage.planned,
            stage.deferred,
            stage.obsolete,
            stage.missing_shapes
        )
        .unwrap();
    }
    push_canonical_diagnostics(&mut output, root, &plan.diagnostics);
    output
}

fn canonical_parse_only_report(report: &mizar_test::ParseOnlyRunReport, root: &Path) -> String {
    let mut output = String::new();
    writeln!(
        output,
        "parse-only-counts|{}|{}|{}|{}",
        report.results.len(),
        report.passed_count(),
        report.failed_count(),
        report.error_count()
    )
    .unwrap();
    for result in &report.results {
        writeln!(
            output,
            "parse-only-result|{}|{}|{}|codes={}|snapshot={}",
            result.id.0,
            rel_string(root, &result.expectation_path),
            parse_only_status(result.status),
            result.actual_diagnostic_codes.join(","),
            result.snapshot_failure.as_deref().unwrap_or("<none>")
        )
        .unwrap();
    }
    push_canonical_diagnostics(&mut output, root, &report.diagnostics);
    output
}

fn canonical_declaration_symbol_report(
    report: &mizar_test::DeclarationSymbolRunReport,
    root: &Path,
) -> String {
    let mut output = String::new();
    writeln!(
        output,
        "declaration-symbol-counts|{}|{}|{}|{}",
        report.results.len(),
        report.passed_count(),
        report.failed_count(),
        report.error_count()
    )
    .unwrap();
    for result in &report.results {
        writeln!(
            output,
            "declaration-symbol-result|{}|{}|{}|details={}|payloads={}",
            result.id.0,
            rel_string(root, &result.expectation_path),
            declaration_symbol_status(result.status),
            result.actual_detail_keys.join(","),
            result.actual_payload_keys.join(",")
        )
        .unwrap();
    }
    push_canonical_diagnostics(&mut output, root, &report.diagnostics);
    output
}

fn canonical_type_elaboration_report(
    report: &mizar_test::TypeElaborationRunReport,
    root: &Path,
) -> String {
    let mut output = String::new();
    writeln!(
        output,
        "type-elaboration-counts|{}|{}|{}|{}",
        report.results.len(),
        report.passed_count(),
        report.failed_count(),
        report.error_count()
    )
    .unwrap();
    for result in &report.results {
        writeln!(
            output,
            "type-elaboration-result|{}|{}|{}|details={}",
            result.id.0,
            rel_string(root, &result.expectation_path),
            type_elaboration_status(result.status),
            result.actual_detail_keys.join(",")
        )
        .unwrap();
    }
    push_canonical_diagnostics(&mut output, root, &report.diagnostics);
    output
}

fn push_canonical_diagnostics(
    output: &mut String,
    root: &Path,
    diagnostics: &[mizar_test::ValidationDiagnostic],
) {
    writeln!(output, "diagnostics={}", diagnostics.len()).unwrap();
    for diagnostic in diagnostics {
        writeln!(
            output,
            "diagnostic|{}|{}|{}|{}|{}|{}",
            severity(diagnostic.severity),
            rel_string(root, &diagnostic.path),
            diagnostic.record_kind,
            diagnostic.code.0,
            diagnostic.detail_key,
            diagnostic.message
        )
        .unwrap();
    }
}

fn severity(severity: mizar_test::ValidationSeverity) -> &'static str {
    match severity {
        mizar_test::ValidationSeverity::Error => "error",
        mizar_test::ValidationSeverity::Warning => "warning",
        _ => "unknown",
    }
}

fn parse_only_status(status: mizar_test::ParseOnlyCaseStatus) -> &'static str {
    match status {
        mizar_test::ParseOnlyCaseStatus::Passed => "passed",
        mizar_test::ParseOnlyCaseStatus::Failed => "failed",
        _ => "unknown",
    }
}

fn declaration_symbol_status(status: mizar_test::DeclarationSymbolCaseStatus) -> &'static str {
    match status {
        mizar_test::DeclarationSymbolCaseStatus::Passed => "passed",
        mizar_test::DeclarationSymbolCaseStatus::Failed => "failed",
        _ => "unknown",
    }
}

fn type_elaboration_status(status: mizar_test::TypeElaborationCaseStatus) -> &'static str {
    match status {
        mizar_test::TypeElaborationCaseStatus::Passed => "passed",
        mizar_test::TypeElaborationCaseStatus::Failed => "failed",
        _ => "unknown",
    }
}

fn rel_string(root: &Path, path: &Path) -> String {
    rel(root, path).to_string_lossy().replace('\\', "/")
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

fn plan_cli(corpus: &Corpus) -> std::process::Command {
    let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_mizar-test"));
    command
        .arg("plan")
        .arg("--workspace-root")
        .arg(&corpus.root);
    command
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
