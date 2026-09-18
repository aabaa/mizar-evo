use std::path::Path;

use crate::diagnostic::ValidationDiagnostic;
use crate::expectation::{ExpectedOutcome, PipelinePhase};
use crate::harness::{TestCase, TestPlan};
use crate::staged_model::Stage;

use super::super::syntax_smoke::workspace_relative_source;

const ACTIVE_TYPE_ELABORATION_TAG: &str = "active_type_elaboration";
const STEP5C5_ARGUMENT_ID: &str = "fail_type_elaboration_pred_argument_type_mismatch_001";
const STEP5C5_PROPERTY_ID: &str = "pass_type_elaboration_func_commutativity_property_001";
const STEP5C3_ARGUMENT_ID: &str = "fail_type_elaboration_argument_type_mismatch_functor_001";
const STEP5C4_DEPENDENT_ID: &str = "pass_type_elaboration_mode_dependent_of_params_001";

const STEP5C6_ALIAS_IDS: [&str; 3] = [
    "fail_type_elaboration_synonym_loci_mismatch_001",
    "pass_type_elaboration_synonym_functor_001",
    "pass_type_elaboration_antonym_predicate_001",
];

pub(in crate::runner) fn is_step5c5_argument_candidate(case: &TestCase) -> bool {
    [STEP5C5_ARGUMENT_ID, STEP5C5_PROPERTY_ID]
        .into_iter()
        .any(|id| {
            case.id.0 == id
                || case.expectation.id.0 == id
                || case.source_path.file_stem().is_some_and(|stem| stem == id)
                || case
                    .expectation_path
                    .file_name()
                    .is_some_and(|name| name == format!("{id}.expect.toml").as_str())
        })
}

pub(in crate::runner) fn step5c5_argument_admitted(root: Option<&Path>, case: &TestCase) -> bool {
    let property = case.id.0 == STEP5C5_PROPERTY_ID;
    if !property && case.id.0 != STEP5C5_ARGUMENT_ID {
        return false;
    }
    let directory = if property {
        "pass/functors"
    } else {
        "fail/predicates"
    };
    let path = format!("tests/miz/{directory}/{}.miz", case.id.0);
    let exact_path = |actual: &Path, expected: &Path| match root {
        Some(root) => {
            workspace_relative_source(root, actual).is_some_and(|path| Path::new(&path) == expected)
        }
        None => actual.ends_with(expected),
    };
    case.expectation.schema_version == 1
        && case.expectation.id == case.id
        && exact_path(&case.source_path, Path::new(&path))
        && exact_path(
            &case.expectation_path,
            &Path::new(&path).with_extension("expect.toml"),
        )
        && case.expectation.source == Path::new(&path).file_name().unwrap()
        && case.expectation.kind
            == if property {
                crate::expectation::TestKind::Pass
            } else {
                crate::expectation::TestKind::Fail
            }
        && case.expectation.stage == Stage::TypeElaboration
        && case.expectation.expected_phase == Some(PipelinePhase::TypeCheck)
        && case.expectation.expected_outcome
            == if property {
                ExpectedOutcome::Pass
            } else {
                ExpectedOutcome::Fail
            }
        && case.expectation.failure_category.as_deref() == (!property).then_some("type_error")
        && case.expectation.stable_detail_key.as_deref()
            == (!property).then_some("predicates.application.argument_type_mismatch")
        && case.expectation.domain
            == if property {
                "functors.properties"
            } else {
                "predicates.application"
            }
        && case.expectation.spec_refs.len() == 1
        && case.expectation.spec_refs[0].0
            == if property {
                "spec.en.10.functors.properties.declaration"
            } else {
                "spec.en.09.predicates.application.typing"
            }
        && case.expectation.profiles.as_slice() == ["fast"]
        && case.expectation.rejection_reason.is_none()
        && case.expectation.diagnostic_codes.is_empty()
        && case.expectation.diagnostic_payloads.is_empty()
        && case.expectation.declaration_symbol_payloads.is_empty()
        && case.expectation.snapshots.is_none()
        && case.expectation.ast_profile.is_none()
        && case.expectation.snapshot_profiles.is_empty()
        && case.expectation.tokens.is_empty()
        && case.expectation.origin.is_none()
        && case.expectation.architecture22.is_none()
        && case.expectation.tags.as_slice() == [ACTIVE_TYPE_ELABORATION_TAG]
}

pub(in crate::runner) fn is_step5c3_argument_candidate(case: &TestCase) -> bool {
    let id = STEP5C3_ARGUMENT_ID;
    case.id.0 == id
        || case.expectation.id.0 == id
        || case.source_path.file_stem().is_some_and(|stem| stem == id)
        || case
            .expectation_path
            .file_name()
            .is_some_and(|name| name == format!("{id}.expect.toml").as_str())
}

pub(in crate::runner) fn step5c3_argument_admitted(root: Option<&Path>, case: &TestCase) -> bool {
    let path = format!("tests/miz/fail/types/{STEP5C3_ARGUMENT_ID}.miz");
    let exact_path = |actual: &Path, expected: &Path| match root {
        Some(root) => {
            workspace_relative_source(root, actual).is_some_and(|path| Path::new(&path) == expected)
        }
        None => actual.ends_with(expected),
    };
    case.id.0 == STEP5C3_ARGUMENT_ID
        && case.expectation.schema_version == 1
        && case.expectation.id == case.id
        && exact_path(&case.source_path, Path::new(&path))
        && exact_path(
            &case.expectation_path,
            &Path::new(&path).with_extension("expect.toml"),
        )
        && case.expectation.source == Path::new(&path).file_name().unwrap()
        && case.expectation.kind == crate::expectation::TestKind::Fail
        && case.expectation.stage == Stage::TypeElaboration
        && case.expectation.expected_phase == Some(PipelinePhase::TypeCheck)
        && case.expectation.expected_outcome == ExpectedOutcome::Fail
        && case.expectation.failure_category.as_deref() == Some("type_error")
        && case.expectation.stable_detail_key.as_deref()
            == Some("types.application.argument_type_mismatch")
        && case.expectation.domain == "types.application"
        && case.expectation.spec_refs.len() == 1
        && case.expectation.spec_refs[0].0 == "spec.en.03.types.widening.argument_position"
        && case.expectation.profiles.as_slice() == ["fast"]
        && case.expectation.rejection_reason.is_none()
        && case.expectation.diagnostic_codes.is_empty()
        && case.expectation.diagnostic_payloads.is_empty()
        && case.expectation.declaration_symbol_payloads.is_empty()
        && case.expectation.snapshots.is_none()
        && case.expectation.ast_profile.is_none()
        && case.expectation.snapshot_profiles.is_empty()
        && case.expectation.tokens.is_empty()
        && case.expectation.origin.is_none()
        && case.expectation.architecture22.is_none()
        && case.expectation.tags.as_slice() == [ACTIVE_TYPE_ELABORATION_TAG]
}

pub(in crate::runner) fn is_step5c4_dependent_candidate(case: &TestCase) -> bool {
    let id = STEP5C4_DEPENDENT_ID;
    case.id.0 == id
        || case.expectation.id.0 == id
        || case.source_path.file_stem().is_some_and(|stem| stem == id)
        || case
            .expectation_path
            .file_name()
            .is_some_and(|name| name == format!("{id}.expect.toml").as_str())
}

pub(in crate::runner) fn step5c4_dependent_admitted(root: Option<&Path>, case: &TestCase) -> bool {
    let path = format!("tests/miz/pass/modes/{STEP5C4_DEPENDENT_ID}.miz");
    let exact_path = |actual: &Path, expected: &Path| match root {
        Some(root) => {
            workspace_relative_source(root, actual).is_some_and(|path| Path::new(&path) == expected)
        }
        None => actual.ends_with(expected),
    };
    case.id.0 == STEP5C4_DEPENDENT_ID
        && case.expectation.schema_version == 1
        && case.expectation.id == case.id
        && exact_path(&case.source_path, Path::new(&path))
        && exact_path(
            &case.expectation_path,
            &Path::new(&path).with_extension("expect.toml"),
        )
        && case.expectation.source == Path::new(&path).file_name().unwrap()
        && case.expectation.kind == crate::expectation::TestKind::Pass
        && case.expectation.stage == Stage::TypeElaboration
        && case.expectation.expected_phase == Some(PipelinePhase::TypeCheck)
        && case.expectation.expected_outcome == ExpectedOutcome::Pass
        && case.expectation.failure_category.is_none()
        && case.expectation.stable_detail_key.is_none()
        && case.expectation.domain == "modes.dependent"
        && case.expectation.spec_refs.len() == 1
        && case.expectation.spec_refs[0].0 == "spec.en.07.modes.dependent.of_parameters"
        && case.expectation.profiles.as_slice() == ["fast"]
        && case.expectation.rejection_reason.is_none()
        && case.expectation.diagnostic_codes.is_empty()
        && case.expectation.diagnostic_payloads.is_empty()
        && case.expectation.declaration_symbol_payloads.is_empty()
        && case.expectation.snapshots.is_none()
        && case.expectation.ast_profile.is_none()
        && case.expectation.snapshot_profiles.is_empty()
        && case.expectation.tokens.is_empty()
        && case.expectation.origin.is_none()
        && case.expectation.architecture22.is_none()
        && case.expectation.tags.as_slice() == [ACTIVE_TYPE_ELABORATION_TAG]
}

pub(in crate::runner) fn is_step5c6_alias_candidate(case: &TestCase) -> bool {
    STEP5C6_ALIAS_IDS.iter().any(|id| {
        case.id.0 == *id
            || case.expectation.id.0 == *id
            || case.source_path.file_stem().is_some_and(|stem| stem == *id)
            || case
                .expectation_path
                .file_name()
                .is_some_and(|name| name == format!("{id}.expect.toml").as_str())
    })
}

pub(in crate::runner) fn step5c6_synonym_admitted(root: Option<&Path>, case: &TestCase) -> bool {
    use crate::expectation::TestKind;
    let (directory, kind, phase, outcome, category, detail) = if case.id.0 == STEP5C6_ALIAS_IDS[0] {
        (
            "fail",
            TestKind::Fail,
            PipelinePhase::Resolve,
            ExpectedOutcome::Fail,
            Some("resolve_error"),
            Some("notation.synonym.loci_mismatch"),
        )
    } else if STEP5C6_ALIAS_IDS[1..].contains(&case.id.0.as_str()) {
        (
            "pass",
            TestKind::Pass,
            PipelinePhase::TypeCheck,
            ExpectedOutcome::Pass,
            None,
            None,
        )
    } else {
        return false;
    };
    let path = format!("tests/miz/{directory}/resolve/{}.miz", case.id.0);
    let exact_path = |actual: &Path, expected: &Path| match root {
        Some(root) => {
            workspace_relative_source(root, actual).is_some_and(|path| Path::new(&path) == expected)
        }
        None => actual.ends_with(expected),
    };
    case.expectation.schema_version == 1
        && case.expectation.id == case.id
        && exact_path(&case.source_path, Path::new(&path))
        && exact_path(
            &case.expectation_path,
            &Path::new(&path).with_extension("expect.toml"),
        )
        && case.expectation.source == Path::new(&path).file_name().unwrap()
        && case.expectation.kind == kind
        && case.expectation.stage == Stage::TypeElaboration
        && case.expectation.expected_phase == Some(phase)
        && case.expectation.expected_outcome == outcome
        && case.expectation.failure_category.as_deref() == category
        && case.expectation.stable_detail_key.as_deref() == detail
        && case.expectation.domain
            == if case.id.0 == STEP5C6_ALIAS_IDS[2] {
                "notation.antonym"
            } else {
                "notation.synonym"
            }
        && case.expectation.spec_refs.len() == 1
        && case.expectation.spec_refs[0].0
            == if case.id.0 == STEP5C6_ALIAS_IDS[2] {
                "spec.en.11.symbols.antonym.predicate"
            } else {
                "spec.en.11.symbols.synonym.functor"
            }
        && case.expectation.profiles.as_slice() == ["fast"]
        && case.expectation.rejection_reason.is_none()
        && case.expectation.diagnostic_codes.is_empty()
        && case.expectation.diagnostic_payloads.is_empty()
        && case.expectation.declaration_symbol_payloads.is_empty()
        && case.expectation.snapshots.is_none()
        && case.expectation.ast_profile.is_none()
        && case.expectation.snapshot_profiles.is_empty()
        && case.expectation.tokens.is_empty()
        && case.expectation.origin.is_none()
        && case.expectation.architecture22.is_none()
        && case.expectation.tags.as_slice() == [ACTIVE_TYPE_ELABORATION_TAG]
}

const STEP5C12_IDS: [&str; 5] = [
    "pass_type_elaboration_template_type_param_functor_001",
    "pass_type_elaboration_template_pred_param_001",
    "fail_type_elaboration_template_arity_mismatch_001",
    "pass_type_elaboration_template_extends_bound_001",
    "fail_type_elaboration_template_bound_violation_001",
];

pub(in crate::runner) fn is_step5c12_candidate(case: &TestCase) -> bool {
    STEP5C12_IDS.iter().any(|id| {
        case.id.0 == *id
            || case.expectation.id.0 == *id
            || case.source_path.file_stem().is_some_and(|stem| stem == *id)
            || case
                .expectation_path
                .file_name()
                .is_some_and(|name| name == format!("{id}.expect.toml").as_str())
    })
}

pub(in crate::runner) fn step5c12_admitted(root: Option<&Path>, case: &TestCase) -> bool {
    use crate::expectation::TestKind;
    let Some(index) = STEP5C12_IDS.iter().position(|id| *id == case.id.0) else {
        return false;
    };
    let (domain, spec_ref, detail) = match index {
        0 => (
            "templates.type_parameter",
            "spec.en.18.templates.type_parameter.functor",
            None,
        ),
        1 => (
            "templates.predicate_parameter",
            "spec.en.18.templates.predicate_parameter.declaration",
            None,
        ),
        2 => (
            "templates.instantiation",
            "spec.en.18.templates.instantiation.arity",
            Some("templates.argument.arity_mismatch"),
        ),
        3 => (
            "templates.bounded_parameter",
            "spec.en.18.templates.type_parameter.extends_bound",
            None,
        ),
        4 => (
            "templates.bounded_parameter",
            "spec.en.18.templates.type_parameter.extends_bound",
            Some("templates.argument.bound_violation"),
        ),
        _ => unreachable!(),
    };
    let negative = detail.is_some();
    let kind = if negative { "fail" } else { "pass" };
    let path = format!("tests/miz/{kind}/templates/{}.miz", case.id.0);
    let sidecar = Path::new(&path).with_extension("expect.toml");
    let exact_path = |actual: &Path, expected: &Path| match root {
        Some(root) => {
            workspace_relative_source(root, actual).is_some_and(|path| Path::new(&path) == expected)
        }
        None => actual.ends_with(expected),
    };
    case.expectation.id == case.id
        && case.expectation.schema_version == 1
        && exact_path(&case.source_path, Path::new(&path))
        && exact_path(&case.expectation_path, &sidecar)
        && case.expectation.source == Path::new(&path).file_name().unwrap()
        && case.expectation.kind
            == if negative {
                TestKind::Fail
            } else {
                TestKind::Pass
            }
        && case.expectation.stage == Stage::TypeElaboration
        && case.expectation.expected_phase == Some(PipelinePhase::TypeCheck)
        && case.expectation.expected_outcome
            == if negative {
                ExpectedOutcome::Fail
            } else {
                ExpectedOutcome::Pass
            }
        && case.expectation.failure_category.as_deref() == negative.then_some("type_error")
        && case.expectation.stable_detail_key.as_deref() == detail
        && case.expectation.domain == domain
        && case.expectation.spec_refs.len() == 1
        && case.expectation.spec_refs[0].0 == spec_ref
        && case.expectation.profiles.as_slice() == ["fast"]
        && case.expectation.rejection_reason.is_none()
        && case.expectation.diagnostic_codes.is_empty()
        && case.expectation.diagnostic_payloads.is_empty()
        && case.expectation.declaration_symbol_payloads.is_empty()
        && case.expectation.snapshots.is_none()
        && case.expectation.ast_profile.is_none()
        && case.expectation.snapshot_profiles.is_empty()
        && case.expectation.tokens.is_empty()
        && case.expectation.origin.is_none()
        && case.expectation.architecture22.is_none()
        && case.expectation.tags.as_slice() == [ACTIVE_TYPE_ELABORATION_TAG]
}

const STEP5C14_STATIC_CASES: [(&str, &str); 2] = [
    (
        "fail_type_elaboration_algorithm_break_outside_loop_001",
        "algorithms.control_flow.break_outside_loop",
    ),
    (
        "fail_type_elaboration_algorithm_ghost_isolation_001",
        "algorithms.ghost.isolation_violation",
    ),
];

pub(in crate::runner) fn is_step5c14_static_candidate(case: &TestCase) -> bool {
    STEP5C14_STATIC_CASES.iter().any(|(id, _)| {
        case.id.0 == *id
            || case.expectation.id.0 == *id
            || case.source_path.file_stem().is_some_and(|stem| stem == *id)
            || case
                .expectation_path
                .file_name()
                .is_some_and(|name| name == format!("{id}.expect.toml").as_str())
    })
}

pub(in crate::runner) fn step5c14_static_admitted(root: Option<&Path>, case: &TestCase) -> bool {
    let Some((id, key)) = STEP5C14_STATIC_CASES
        .iter()
        .find(|(id, _)| case.id.0 == *id)
    else {
        return false;
    };
    let path = format!("tests/miz/fail/algorithms/{id}.miz");
    let exact_path = |actual: &Path, expected: &Path| match root {
        Some(root) => {
            workspace_relative_source(root, actual).is_some_and(|path| Path::new(&path) == expected)
        }
        None => actual.ends_with(expected),
    };
    case.expectation.id == case.id
        && exact_path(&case.source_path, Path::new(&path))
        && exact_path(
            &case.expectation_path,
            &Path::new(&path).with_extension("expect.toml"),
        )
        && case.expectation.source == Path::new(&path).file_name().unwrap()
        && case.expectation.kind == crate::expectation::TestKind::Fail
        && case.expectation.stage == Stage::TypeElaboration
        && case.expectation.expected_phase == Some(PipelinePhase::Elaboration)
        && case.expectation.expected_outcome == ExpectedOutcome::Fail
        && case.expectation.failure_category.as_deref() == Some("type_error")
        && case.expectation.stable_detail_key.as_deref() == Some(*key)
        && case.expectation.rejection_reason.is_none()
        && case.expectation.diagnostic_codes.is_empty()
        && case.expectation.diagnostic_payloads.is_empty()
        && case.expectation.declaration_symbol_payloads.is_empty()
        && case.expectation.snapshots.is_none()
        && case.expectation.tags.as_slice() == [ACTIVE_TYPE_ELABORATION_TAG]
}

const STEP5C5_CASES: [(&str, &str, PipelinePhase, ExpectedOutcome); 10] = [
    (
        "fail_type_elaboration_pred_property_arity_mismatch_001",
        "tests/miz/fail/predicates/fail_type_elaboration_pred_property_arity_mismatch_001.miz",
        PipelinePhase::TypeCheck,
        ExpectedOutcome::Fail,
    ),
    (
        "pass_type_elaboration_pred_properties_declaration_001",
        "tests/miz/pass/predicates/pass_type_elaboration_pred_properties_declaration_001.miz",
        PipelinePhase::TypeCheck,
        ExpectedOutcome::Pass,
    ),
    (
        "pass_type_elaboration_func_builtin_bracket_pair_001",
        "tests/miz/pass/functors/pass_type_elaboration_func_builtin_bracket_pair_001.miz",
        PipelinePhase::TypeCheck,
        ExpectedOutcome::Pass,
    ),
    (
        "fail_type_elaboration_func_equals_result_type_mismatch_001",
        "tests/miz/fail/functors/fail_type_elaboration_func_equals_result_type_mismatch_001.miz",
        PipelinePhase::TypeCheck,
        ExpectedOutcome::Fail,
    ),
    (
        "fail_type_elaboration_func_means_missing_correctness_001",
        "tests/miz/fail/functors/fail_type_elaboration_func_means_missing_correctness_001.miz",
        PipelinePhase::TypeCheck,
        ExpectedOutcome::Fail,
    ),
    (
        "fail_type_elaboration_func_duplicate_same_signature_001",
        "tests/miz/fail/functors/fail_type_elaboration_func_duplicate_same_signature_001.miz",
        PipelinePhase::Resolve,
        ExpectedOutcome::Fail,
    ),
    (
        "fail_type_elaboration_func_property_arity_mismatch_001",
        "tests/miz/fail/functors/fail_type_elaboration_func_property_arity_mismatch_001.miz",
        PipelinePhase::TypeCheck,
        ExpectedOutcome::Fail,
    ),
    (
        "fail_type_elaboration_pred_duplicate_same_signature_001",
        "tests/miz/fail/predicates/fail_type_elaboration_pred_duplicate_same_signature_001.miz",
        PipelinePhase::Resolve,
        ExpectedOutcome::Fail,
    ),
    (
        STEP5C5_ARGUMENT_ID,
        "tests/miz/fail/predicates/fail_type_elaboration_pred_argument_type_mismatch_001.miz",
        PipelinePhase::TypeCheck,
        ExpectedOutcome::Fail,
    ),
    (
        STEP5C5_PROPERTY_ID,
        "tests/miz/pass/functors/pass_type_elaboration_func_commutativity_property_001.miz",
        PipelinePhase::TypeCheck,
        ExpectedOutcome::Pass,
    ),
];

const STEP5C5_BLOCKED_CASE_IDS: [&str; 7] = [
    "pass_formula_statement_pred_negated_application_001",
    "pass_proof_verification_pred_phrase_identifier_001",
    "pass_proof_verification_pred_symbolic_infix_001",
    "pass_type_elaboration_pred_redefine_narrower_loci_001",
    "pass_type_elaboration_func_dependent_return_type_001",
    "pass_proof_verification_func_equals_infix_operator_001",
    "pass_proof_verification_func_means_prefix_001",
];

const STEP5C7_TERM_CASES: [(&str, &str, PipelinePhase, ExpectedOutcome, Option<&str>); 5] = [
    (
        "fail_type_elaboration_term_choice_uninhabited_001",
        "tests/miz/fail/terms/fail_type_elaboration_term_choice_uninhabited_001.miz",
        PipelinePhase::TypeCheck,
        ExpectedOutcome::Fail,
        Some("terms.choice.missing_inhabitation"),
    ),
    (
        "pass_type_elaboration_term_choice_builtin_001",
        "tests/miz/pass/terms/pass_type_elaboration_term_choice_builtin_001.miz",
        PipelinePhase::TypeCheck,
        ExpectedOutcome::Pass,
        None,
    ),
    (
        "pass_type_elaboration_term_numeral_equality_001",
        "tests/miz/pass/terms/pass_type_elaboration_term_numeral_equality_001.miz",
        PipelinePhase::TypeCheck,
        ExpectedOutcome::Pass,
        None,
    ),
    (
        "pass_type_elaboration_term_qua_widening_001",
        "tests/miz/pass/terms/pass_type_elaboration_term_qua_widening_001.miz",
        PipelinePhase::TypeCheck,
        ExpectedOutcome::Pass,
        None,
    ),
    (
        "fail_type_elaboration_term_comprehension_unbound_mapper_001",
        "tests/miz/fail/terms/fail_type_elaboration_term_comprehension_unbound_mapper_001.miz",
        PipelinePhase::Resolve,
        ExpectedOutcome::Fail,
        Some("terms.comprehension.unbound_mapper_variable"),
    ),
];

const STEP5C4_MODE_CASES: [(&str, &str, PipelinePhase, ExpectedOutcome); 6] = [
    (
        "pass_type_elaboration_mode_attributed_struct_radix_001",
        "tests/miz/pass/modes/pass_type_elaboration_mode_attributed_struct_radix_001.miz",
        PipelinePhase::TypeCheck,
        ExpectedOutcome::Pass,
    ),
    (
        "fail_type_elaboration_mode_dependent_arity_mismatch_001",
        "tests/miz/fail/modes/fail_type_elaboration_mode_dependent_arity_mismatch_001.miz",
        PipelinePhase::TypeCheck,
        ExpectedOutcome::Fail,
    ),
    (
        "pass_type_elaboration_mode_property_impl_equals_001",
        "tests/miz/pass/modes/pass_type_elaboration_mode_property_impl_equals_001.miz",
        PipelinePhase::TypeCheck,
        ExpectedOutcome::Pass,
    ),
    (
        "pass_type_elaboration_mode_property_impl_means_001",
        "tests/miz/pass/modes/pass_type_elaboration_mode_property_impl_means_001.miz",
        PipelinePhase::TypeCheck,
        ExpectedOutcome::Pass,
    ),
    (
        "fail_type_elaboration_mode_property_impl_unknown_property_001",
        "tests/miz/fail/modes/fail_type_elaboration_mode_property_impl_unknown_property_001.miz",
        PipelinePhase::TypeCheck,
        ExpectedOutcome::Fail,
    ),
    (
        STEP5C4_DEPENDENT_ID,
        "tests/miz/pass/modes/pass_type_elaboration_mode_dependent_of_params_001.miz",
        PipelinePhase::TypeCheck,
        ExpectedOutcome::Pass,
    ),
];

const STEP5C3_ATTRIBUTE_CASES: [(&str, &str, PipelinePhase, ExpectedOutcome); 6] = [
    (
        "fail_type_elaboration_attr_duplicate_same_subject_001",
        "tests/miz/fail/attributes/fail_type_elaboration_attr_duplicate_same_subject_001.miz",
        PipelinePhase::Resolve,
        ExpectedOutcome::Fail,
    ),
    (
        "pass_type_elaboration_attr_struct_qualified_reference_001",
        "tests/miz/pass/attributes/pass_type_elaboration_attr_struct_qualified_reference_001.miz",
        PipelinePhase::TypeCheck,
        ExpectedOutcome::Pass,
    ),
    (
        "pass_type_elaboration_attr_param_prefix_declaration_001",
        "tests/miz/pass/attributes/pass_type_elaboration_attr_param_prefix_declaration_001.miz",
        PipelinePhase::TypeCheck,
        ExpectedOutcome::Pass,
    ),
    (
        "pass_type_elaboration_attr_redefine_narrower_subject_001",
        "tests/miz/pass/attributes/pass_type_elaboration_attr_redefine_narrower_subject_001.miz",
        PipelinePhase::TypeCheck,
        ExpectedOutcome::Pass,
    ),
    (
        "fail_type_elaboration_attr_non_attribute_symbol_001",
        "tests/miz/fail/attributes/fail_type_elaboration_attr_non_attribute_symbol_001.miz",
        PipelinePhase::TypeCheck,
        ExpectedOutcome::Fail,
    ),
    (
        STEP5C3_ARGUMENT_ID,
        "tests/miz/fail/types/fail_type_elaboration_argument_type_mismatch_functor_001.miz",
        PipelinePhase::TypeCheck,
        ExpectedOutcome::Fail,
    ),
];

const STEP5C3_G1_CASE_IDS: [&str; 1] = ["pass_type_elaboration_argument_attribute_widening_001"];

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

const STEP5C2_STRUCTURE_CASES: [(&str, &str, PipelinePhase, ExpectedOutcome); 12] = [
    (
        "fail_type_elaboration_struct_constructor_missing_field_001",
        "tests/miz/fail/structures/fail_type_elaboration_struct_constructor_missing_field_001.miz",
        PipelinePhase::TypeCheck,
        ExpectedOutcome::Fail,
    ),
    (
        "fail_type_elaboration_struct_duplicate_member_001",
        "tests/miz/fail/structures/fail_type_elaboration_struct_duplicate_member_001.miz",
        PipelinePhase::Resolve,
        ExpectedOutcome::Fail,
    ),
    (
        "pass_type_elaboration_struct_definition_basic_001",
        "tests/miz/pass/structures/pass_type_elaboration_struct_definition_basic_001.miz",
        PipelinePhase::TypeCheck,
        ExpectedOutcome::Pass,
    ),
    (
        "pass_type_elaboration_struct_property_member_001",
        "tests/miz/pass/structures/pass_type_elaboration_struct_property_member_001.miz",
        PipelinePhase::TypeCheck,
        ExpectedOutcome::Pass,
    ),
    (
        "pass_type_elaboration_struct_dependent_bracket_params_001",
        "tests/miz/pass/structures/pass_type_elaboration_struct_dependent_bracket_params_001.miz",
        PipelinePhase::TypeCheck,
        ExpectedOutcome::Pass,
    ),
    (
        "fail_type_elaboration_struct_diamond_inconsistent_001",
        "tests/miz/fail/structures/fail_type_elaboration_struct_diamond_inconsistent_001.miz",
        PipelinePhase::TypeCheck,
        ExpectedOutcome::Fail,
    ),
    (
        "pass_type_elaboration_struct_diamond_consistent_001",
        "tests/miz/pass/structures/pass_type_elaboration_struct_diamond_consistent_001.miz",
        PipelinePhase::TypeCheck,
        ExpectedOutcome::Pass,
    ),
    (
        "pass_type_elaboration_struct_inherit_from_set_001",
        "tests/miz/pass/structures/pass_type_elaboration_struct_inherit_from_set_001.miz",
        PipelinePhase::TypeCheck,
        ExpectedOutcome::Pass,
    ),
    (
        "fail_type_elaboration_struct_inherit_uncovered_member_001",
        "tests/miz/fail/structures/fail_type_elaboration_struct_inherit_uncovered_member_001.miz",
        PipelinePhase::TypeCheck,
        ExpectedOutcome::Fail,
    ),
    (
        "fail_type_elaboration_struct_inherit_unknown_source_001",
        "tests/miz/fail/structures/fail_type_elaboration_struct_inherit_unknown_source_001.miz",
        PipelinePhase::TypeCheck,
        ExpectedOutcome::Fail,
    ),
    (
        "pass_type_elaboration_struct_inherit_rename_001",
        "tests/miz/pass/structures/pass_type_elaboration_struct_inherit_rename_001.miz",
        PipelinePhase::TypeCheck,
        ExpectedOutcome::Pass,
    ),
    (
        "fail_type_elaboration_struct_unknown_selector_001",
        "tests/miz/fail/structures/fail_type_elaboration_struct_unknown_selector_001.miz",
        PipelinePhase::TypeCheck,
        ExpectedOutcome::Fail,
    ),
];

pub(in crate::runner) fn is_active_type_elaboration(case: &TestCase) -> bool {
    if super::super::formula_statement::is_step5c5_negated_candidate(case) {
        return false;
    }
    if is_step5c5_argument_candidate(case) {
        return step5c5_argument_admitted(None, case);
    }
    if is_step5c3_argument_candidate(case) {
        return step5c3_argument_admitted(None, case);
    }
    if is_step5c4_dependent_candidate(case) {
        return step5c4_dependent_admitted(None, case);
    }
    if is_step5c6_alias_candidate(case) {
        return step5c6_synonym_admitted(None, case);
    }
    if is_step5c14_static_candidate(case) {
        return step5c14_static_admitted(None, case);
    }
    if is_step5c12_candidate(case) {
        return step5c12_admitted(None, case);
    }
    if super::super::proof_verification::is_step5c14_return_candidate(case)
        || super::super::is_step5c13_overload_candidate(case)
        || super::super::is_step5c11_registration_candidate(case)
        || super::super::proof_verification::is_step5c11_proof_candidate(case)
    {
        return false;
    }
    if super::super::parse_only::is_step5c11_parse_candidate(case) {
        return false;
    }
    if super::super::formula_statement::is_step5c9_candidate(case) {
        return false;
    }
    if super::super::formula_statement::is_step5c8_candidate(case)
        || super::super::formula_statement::is_step5c10_candidate(case)
    {
        return case.expectation.stage == Stage::TypeElaboration
            && super::super::formula_statement::step5_formula_admitted(None, case);
    }
    let exact_step5c1 = step5c1_case(case).is_some()
        && case.expectation.tags.as_slice() == [ACTIVE_TYPE_ELABORATION_TAG];
    let exact_step5c2 = step5c2_case(case).is_some()
        && case.expectation.tags.as_slice() == [ACTIVE_TYPE_ELABORATION_TAG];
    let exact_step5c3 = step5c3_case(case).is_some()
        && case.expectation.tags.as_slice() == [ACTIVE_TYPE_ELABORATION_TAG];
    let exact_step5c4 = step5c4_case(case).is_some()
        && case.expectation.tags.as_slice() == [ACTIVE_TYPE_ELABORATION_TAG];
    let exact_step5c5 = step5c5_case(case).is_some()
        && case.expectation.tags.as_slice() == [ACTIVE_TYPE_ELABORATION_TAG];
    let exact_step5c7 = step5c7_case(case).is_some()
        && step5c7_exact_metadata(case)
        && case.expectation.tags.as_slice() == [ACTIVE_TYPE_ELABORATION_TAG];
    has_active_type_elaboration_tag(case)
        && !is_step5c3_g1_id(case)
        && !is_step5c5_blocked_id(case)
        && case.expectation.stage == Stage::TypeElaboration
        && (case.expectation.expected_phase == Some(PipelinePhase::TypeCheck)
            || (exact_step5c1 || exact_step5c2 || exact_step5c3 || exact_step5c5 || exact_step5c7)
                && case.expectation.expected_phase == Some(PipelinePhase::Resolve))
        && matches!(
            case.expectation.expected_outcome,
            ExpectedOutcome::Pass | ExpectedOutcome::Fail
        )
        && (!is_step5c1_id(case) || exact_step5c1)
        && (!is_step5c2_id(case) || exact_step5c2)
        && (!is_step5c3_id(case) || exact_step5c3)
        && (!is_step5c4_id(case) || exact_step5c4)
        && (!is_step5c5_id(case) || exact_step5c5)
        && (!is_step5c7_candidate(case) || exact_step5c7)
        && case.id.0 != "fail_type_elaboration_term_qua_invalid_narrowing_001"
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
    for case in plan.cases.iter().filter(|case| {
        has_active_type_elaboration_tag(case)
            || STEP5C6_ALIAS_IDS[..2]
                .iter()
                .any(|id| case.id.0 == *id || case.expectation.id.0 == *id)
            || is_step5c14_static_candidate(case)
            || STEP5C12_IDS[..3].iter().any(|id| case.id.0 == *id)
            || is_step5c1_id(case)
            || is_step5c2_id(case)
            || is_step5c3_id(case)
            || is_step5c4_id(case)
            || is_step5c5_id(case)
            || is_step5c7_candidate(case)
            || is_step5c5_blocked_id(case) && has_active_type_elaboration_tag(case)
            || is_step5c3_g1_id(case) && has_active_type_elaboration_tag(case)
    }) {
        if !is_active_type_elaboration(case)
            || is_step5c6_alias_candidate(case)
                && !step5c6_synonym_admitted(Some(workspace_root), case)
            || is_step5c14_static_candidate(case)
                && !step5c14_static_admitted(Some(workspace_root), case)
            || is_step5c12_candidate(case) && !step5c12_admitted(Some(workspace_root), case)
            || is_step5c1_id(case) && !is_step5c1_workspace_member(workspace_root, case)
            || is_step5c2_id(case) && !is_step5c2_workspace_member(workspace_root, case)
            || is_step5c3_id(case) && !is_step5c3_workspace_member(workspace_root, case)
            || is_step5c4_id(case) && !is_step5c4_workspace_member(workspace_root, case)
            || is_step5c5_id(case) && !is_step5c5_workspace_member(workspace_root, case)
            || is_step5c7_candidate(case) && !is_step5c7_workspace_member(workspace_root, case)
            || is_step5c5_blocked_id(case) && has_active_type_elaboration_tag(case)
            || is_step5c3_g1_id(case) && has_active_type_elaboration_tag(case)
        {
            diagnostics.push(
                ValidationDiagnostic::error(
                    &case.expectation_path,
                    "type_elaboration",
                    "E-TYPE-ELABORATION-ACTIVE-GATE",
                    format!("type_elaboration.active_gate.{}", case.id.0),
                "active_type_elaboration cases must be exact .miz pass/fail expectations at stage type_elaboration; only frozen exact inventories may use phase resolve",
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
    if workspace_root
        .join("tests/coverage/step5_activation_map.tsv")
        .is_file()
    {
        for id in &STEP5C6_ALIAS_IDS {
            if plan
                .cases
                .iter()
                .filter(|case| {
                    case.id.0 == *id && step5c6_synonym_admitted(Some(workspace_root), case)
                })
                .count()
                != 1
            {
                diagnostics.push(ValidationDiagnostic::error(
                    workspace_root,
                    "type_elaboration",
                    "E-TYPE-ELABORATION-STEP5C6-INVENTORY",
                    "type_elaboration.step5c6_inventory.synonym",
                    "each frozen alias case must have exactly one admitted source/sidecar pair",
                ));
            }
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
    validate_exact_inventory(
        workspace_root,
        plan,
        &STEP5C2_STRUCTURE_CASES,
        "E-TYPE-ELABORATION-STEP5C2-INVENTORY",
        "step5c2_inventory",
    )
    .into_iter()
    .for_each(|diagnostic| diagnostics.push(diagnostic));
    validate_exact_inventory(
        workspace_root,
        plan,
        &STEP5C4_MODE_CASES,
        "E-TYPE-ELABORATION-STEP5C4-INVENTORY",
        "step5c4_inventory",
    )
    .into_iter()
    .for_each(|diagnostic| diagnostics.push(diagnostic));
    validate_exact_inventory(
        workspace_root,
        plan,
        &STEP5C3_ATTRIBUTE_CASES,
        "E-TYPE-ELABORATION-STEP5C3-INVENTORY",
        "step5c3_inventory",
    )
    .into_iter()
    .for_each(|diagnostic| diagnostics.push(diagnostic));
    validate_exact_inventory(
        workspace_root,
        plan,
        &STEP5C5_CASES,
        "E-TYPE-ELABORATION-STEP5C5-INVENTORY",
        "step5c5_inventory",
    )
    .into_iter()
    .for_each(|diagnostic| diagnostics.push(diagnostic));
    if workspace_root
        .join("tests/coverage/step5_activation_map.tsv")
        .is_file()
        || plan.cases.iter().any(is_step5c12_candidate)
    {
        for id in &STEP5C12_IDS {
            if plan
                .cases
                .iter()
                .filter(|case| case.id.0 == *id && step5c12_admitted(Some(workspace_root), case))
                .count()
                != 1
            {
                diagnostics.push(ValidationDiagnostic::error(
                    workspace_root,
                    "type_elaboration",
                    "E-TYPE-ELABORATION-STEP5C12-INVENTORY",
                    format!("type_elaboration.step5c12_inventory.{id}"),
                    "each template row must have exactly one admitted source/sidecar pair",
                ));
            }
        }
    }
    if workspace_root
        .join("tests/coverage/step5_activation_map.tsv")
        .is_file()
        || plan.cases.iter().any(is_step5c14_static_candidate)
    {
        for (id, _) in STEP5C14_STATIC_CASES {
            if plan
                .cases
                .iter()
                .filter(|case| {
                    case.id.0 == id && step5c14_static_admitted(Some(workspace_root), case)
                })
                .count()
                != 1
            {
                diagnostics.push(ValidationDiagnostic::error(
                    workspace_root,
                    "type_elaboration",
                    "E-TYPE-ELABORATION-STEP5C14-INVENTORY",
                    format!("type_elaboration.step5c14_inventory.{id}"),
                    "each static algorithm row must have exactly one admitted source/sidecar pair",
                ));
            }
        }
    }
    if STEP5C7_TERM_CASES.iter().any(|(_, source, _, _, _)| {
        workspace_root.join(source).is_file()
            || workspace_root
                .join(Path::new(source).with_extension("expect.toml"))
                .is_file()
    }) {
        for (id, source, _, _, _) in STEP5C7_TERM_CASES {
            let count = plan
                .cases
                .iter()
                .filter(|case| case.id.0 == id && is_step5c7_workspace_member(workspace_root, case))
                .count();
            if count != 1 {
                diagnostics.push(ValidationDiagnostic::error(
                    std::path::Path::new(source),
                    "type_elaboration",
                    "E-TYPE-ELABORATION-STEP5C7-INVENTORY",
                    format!("type_elaboration.step5c7_inventory.{id}"),
                    format!("step5c7 term route row `{id}` must occur exactly once; found {count}"),
                ));
            }
        }
    }
    diagnostics
}

fn validate_exact_inventory(
    workspace_root: &Path,
    plan: &TestPlan,
    rows: &[(&str, &str, PipelinePhase, ExpectedOutcome)],
    code: &'static str,
    key_prefix: &str,
) -> Vec<ValidationDiagnostic> {
    if !rows
        .iter()
        .any(|(_, source, _, _)| workspace_root.join(source).is_file())
    {
        return Vec::new();
    }
    rows.iter()
        .filter_map(|(id, source, _, _)| {
            let count = plan
                .cases
                .iter()
                .filter(|case| {
                    case.id.0 == *id
                        && workspace_relative_source(workspace_root, &case.source_path)
                            .is_some_and(|actual| actual == *source)
                })
                .count();
            (count != 1).then(|| {
                ValidationDiagnostic::error(
                    std::path::Path::new(source),
                    "type_elaboration",
                    code,
                    format!("type_elaboration.{key_prefix}.{id}"),
                    format!("{key_prefix} route row `{id}` must occur exactly once; found {count}"),
                )
            })
        })
        .collect()
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

fn is_step5c2_id(case: &TestCase) -> bool {
    STEP5C2_STRUCTURE_CASES
        .iter()
        .any(|(id, _, _, _)| case.id.0 == *id)
}

fn is_step5c3_id(case: &TestCase) -> bool {
    is_step5c3_argument_candidate(case)
        || STEP5C3_ATTRIBUTE_CASES
            .iter()
            .any(|(id, _, _, _)| case.id.0 == *id)
}

fn is_step5c4_id(case: &TestCase) -> bool {
    is_step5c4_dependent_candidate(case)
        || STEP5C4_MODE_CASES
            .iter()
            .any(|(id, _, _, _)| case.id.0 == *id)
}

fn is_step5c5_id(case: &TestCase) -> bool {
    is_step5c5_argument_candidate(case)
        || STEP5C5_CASES.iter().any(|(id, _, _, _)| case.id.0 == *id)
        || is_step5c5_predicate_duplicate_candidate(case)
}

pub(in crate::runner) fn is_step5c5_predicate_duplicate_candidate(case: &TestCase) -> bool {
    let (id, source, _, _) = STEP5C5_CASES[7];
    case.id.0 == id
        || case.expectation.id.0 == id
        || case.source_path.file_name() == Path::new(source).file_name()
        || case.expectation_path.file_name()
            == Path::new(source).with_extension("expect.toml").file_name()
}

fn is_step5c7_candidate(case: &TestCase) -> bool {
    step5c7_case(case).is_some()
}

fn step5c7_case(
    case: &TestCase,
) -> Option<(
    &'static str,
    &'static str,
    PipelinePhase,
    ExpectedOutcome,
    Option<&'static str>,
)> {
    STEP5C7_TERM_CASES
        .iter()
        .copied()
        .find(|(id, source, _, _, _)| {
            case.id.0 == *id
                || case.source_path.file_name() == Path::new(source).file_name()
                || case.expectation_path.file_name()
                    == Path::new(source).with_extension("expect.toml").file_name()
        })
}

fn step5c7_exact_metadata(case: &TestCase) -> bool {
    let Some((_, source, phase, outcome, detail_key)) = STEP5C7_TERM_CASES
        .iter()
        .copied()
        .find(|(id, _, _, _, _)| case.id.0 == *id)
    else {
        return false;
    };
    case.expectation.id == case.id
        && case.source_path.ends_with(source)
        && case
            .expectation_path
            .ends_with(Path::new(source).with_extension("expect.toml"))
        && case.expectation.source == Path::new(source).file_name().unwrap()
        && case.expectation.stage == Stage::TypeElaboration
        && case.expectation.expected_phase == Some(phase)
        && case.expectation.expected_outcome == outcome
        && case.expectation.stable_detail_key.as_deref() == detail_key
        && case.expectation.diagnostic_codes.is_empty()
}

fn is_step5c5_blocked_id(case: &TestCase) -> bool {
    STEP5C5_BLOCKED_CASE_IDS.iter().any(|id| case.id.0 == *id)
}

fn is_step5c3_g1_id(case: &TestCase) -> bool {
    STEP5C3_G1_CASE_IDS.iter().any(|id| case.id.0 == *id)
}

fn step5c2_case(
    case: &TestCase,
) -> Option<(&'static str, &'static str, PipelinePhase, ExpectedOutcome)> {
    STEP5C2_STRUCTURE_CASES
        .iter()
        .copied()
        .find(|(id, source, phase, outcome)| {
            case.id.0 == *id
                && case.source_path.ends_with(source)
                && case.expectation.expected_phase == Some(*phase)
                && case.expectation.expected_outcome == *outcome
        })
}

fn step5c3_case(
    case: &TestCase,
) -> Option<(&'static str, &'static str, PipelinePhase, ExpectedOutcome)> {
    if is_step5c3_argument_candidate(case) && case.id.0 != STEP5C3_ARGUMENT_ID {
        return None;
    }
    STEP5C3_ATTRIBUTE_CASES
        .iter()
        .copied()
        .find(|(id, source, phase, outcome)| {
            case.id.0 == *id
                && case.source_path.ends_with(source)
                && case.expectation.expected_phase == Some(*phase)
                && case.expectation.expected_outcome == *outcome
        })
}

fn step5c4_case(
    case: &TestCase,
) -> Option<(&'static str, &'static str, PipelinePhase, ExpectedOutcome)> {
    STEP5C4_MODE_CASES
        .iter()
        .copied()
        .find(|(id, source, phase, outcome)| {
            case.id.0 == *id
                && case.source_path.ends_with(source)
                && case.expectation.expected_phase == Some(*phase)
                && case.expectation.expected_outcome == *outcome
        })
}

fn step5c5_case(
    case: &TestCase,
) -> Option<(&'static str, &'static str, PipelinePhase, ExpectedOutcome)> {
    if is_step5c5_argument_candidate(case) && !step5c5_argument_admitted(None, case) {
        return None;
    }
    if is_step5c5_predicate_duplicate_candidate(case) && case.id.0 != STEP5C5_CASES[7].0 {
        return None;
    }
    STEP5C5_CASES
        .iter()
        .copied()
        .find(|(id, source, phase, outcome)| {
            case.id.0 == *id
                && case.source_path.ends_with(source)
                && case.expectation.expected_phase == Some(*phase)
                && case.expectation.expected_outcome == *outcome
                && (*id != STEP5C5_CASES[7].0
                    || (case.expectation.id == case.id
                        && case.expectation.kind == crate::expectation::TestKind::Fail
                        && case.expectation.stage == Stage::TypeElaboration
                        && case
                            .expectation_path
                            .ends_with(Path::new(source).with_extension("expect.toml"))
                        && case.expectation.source == Path::new(source).file_name().unwrap()
                        && case.expectation.domain == "predicates.definition"
                        && case.expectation.failure_category.as_deref() == Some("resolve_error")
                        && case.expectation.stable_detail_key.as_deref()
                            == Some("predicates.definition.duplicate_same_signature")
                        && case.expectation.spec_refs.len() == 1
                        && case.expectation.spec_refs[0].0
                            == "spec.en.09.predicates.definition.uniqueness"
                        && case.expectation.diagnostic_codes.is_empty()
                        && case.expectation.diagnostic_payloads.is_empty()
                        && case.expectation.declaration_symbol_payloads.is_empty()
                        && case.expectation.rejection_reason.is_none()
                        && case.expectation.snapshots.is_none()
                        && case.expectation.ast_profile.is_none()
                        && case.expectation.snapshot_profiles.is_empty()
                        && case.expectation.tokens.is_empty()
                        && case.expectation.origin.is_none()
                        && case.expectation.architecture22.is_none()))
        })
}

pub(in crate::runner) fn is_step5c2_workspace_member(
    workspace_root: &Path,
    case: &TestCase,
) -> bool {
    step5c2_case(case).is_some_and(|(_, source, _, _)| {
        workspace_relative_source(workspace_root, &case.source_path)
            .is_some_and(|actual| actual == source)
    })
}

pub(in crate::runner) fn is_step5c3_workspace_member(
    workspace_root: &Path,
    case: &TestCase,
) -> bool {
    if is_step5c3_argument_candidate(case) {
        return step5c3_argument_admitted(Some(workspace_root), case);
    }
    step5c3_case(case).is_some_and(|(_, source, _, _)| {
        workspace_relative_source(workspace_root, &case.source_path)
            .is_some_and(|actual| actual == source)
    })
}

pub(in crate::runner) fn is_step5c4_workspace_member(
    workspace_root: &Path,
    case: &TestCase,
) -> bool {
    if is_step5c4_dependent_candidate(case) {
        return step5c4_dependent_admitted(Some(workspace_root), case);
    }
    step5c4_case(case).is_some_and(|(_, source, _, _)| {
        workspace_relative_source(workspace_root, &case.source_path)
            .is_some_and(|actual| actual == source)
    })
}

pub(in crate::runner) fn is_step5c5_workspace_member(
    workspace_root: &Path,
    case: &TestCase,
) -> bool {
    if is_step5c5_argument_candidate(case) {
        return step5c5_argument_admitted(Some(workspace_root), case);
    }
    step5c5_case(case).is_some_and(|(_, source, _, _)| {
        workspace_relative_source(workspace_root, &case.source_path)
            .is_some_and(|actual| actual == source)
            && (case.id.0 != STEP5C5_CASES[7].0
                || workspace_relative_source(workspace_root, &case.expectation_path).is_some_and(
                    |actual| actual == Path::new(source).with_extension("expect.toml"),
                ))
    })
}

pub(in crate::runner) fn is_step5c7_workspace_member(
    workspace_root: &Path,
    case: &TestCase,
) -> bool {
    let Some((_, source, _, _, _)) = step5c7_case(case) else {
        return false;
    };
    workspace_relative_source(workspace_root, &case.source_path)
        .is_some_and(|actual| actual == source)
        && workspace_relative_source(workspace_root, &case.expectation_path)
            .is_some_and(|actual| actual == Path::new(source).with_extension("expect.toml"))
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
        ACTIVE_TYPE_ELABORATION_TAG, STEP5C1_VARIABLE_CASES, STEP5C2_STRUCTURE_CASES,
        STEP5C3_ATTRIBUTE_CASES, STEP5C3_G1_CASE_IDS, STEP5C4_MODE_CASES, STEP5C5_BLOCKED_CASE_IDS,
        STEP5C5_CASES, STEP5C7_TERM_CASES, is_active_type_elaboration,
        validate_active_type_elaboration_tags,
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
    fn step5c2_inventory_has_twelve_unique_id_source_pairs() {
        assert_eq!(STEP5C2_STRUCTURE_CASES.len(), 12);
        assert_eq!(
            STEP5C2_STRUCTURE_CASES
                .iter()
                .map(|(id, source, _, _)| (*id, *source))
                .collect::<BTreeSet<_>>()
                .len(),
            12
        );
    }

    #[test]
    fn step5c3_inventory_has_six_unique_id_source_pairs_and_excludes_attributed_widening() {
        assert_eq!(STEP5C3_ATTRIBUTE_CASES.len(), 6);
        assert_eq!(
            STEP5C3_ATTRIBUTE_CASES
                .iter()
                .map(|(id, source, _, _)| (*id, *source))
                .collect::<BTreeSet<_>>()
                .len(),
            6
        );
        let plan = build_test_plan(&config()).unwrap();
        for id in STEP5C3_G1_CASE_IDS {
            let case = plan.cases.iter().find(|case| case.id.0 == id).unwrap();
            let mut tagged = case.clone();
            tagged
                .expectation
                .tags
                .push("active_type_elaboration".to_owned());
            assert!(!is_active_type_elaboration(&tagged), "{id}");
        }
    }

    #[test]
    fn step5c3_type_rows_execute_with_frozen_detail_keys() {
        let report = crate::runner::run_type_elaboration_corpus(&config()).expect("type report");
        assert_eq!(report.error_count(), 0, "{:?}", report.diagnostics);
        let expected = [
            (
                "fail_type_elaboration_attr_duplicate_same_subject_001",
                vec!["attributes.definition.duplicate_same_subject"],
            ),
            (
                "pass_type_elaboration_attr_struct_qualified_reference_001",
                Vec::new(),
            ),
            (
                "pass_type_elaboration_attr_param_prefix_declaration_001",
                Vec::new(),
            ),
            (
                "pass_type_elaboration_attr_redefine_narrower_subject_001",
                Vec::new(),
            ),
            (
                "fail_type_elaboration_attr_non_attribute_symbol_001",
                vec!["attributes.reference.non_attribute_symbol"],
            ),
            (
                super::STEP5C3_ARGUMENT_ID,
                vec!["types.application.argument_type_mismatch"],
            ),
        ];
        for (id, expected_keys) in expected {
            let result = report
                .results
                .iter()
                .find(|result| result.id.0 == id)
                .unwrap();
            assert_eq!(
                result.status,
                crate::runner::TypeElaborationCaseStatus::Passed,
                "{id}: {:?}",
                result.actual_detail_keys
            );
            assert_eq!(
                result.actual_detail_keys,
                expected_keys
                    .into_iter()
                    .map(str::to_owned)
                    .collect::<Vec<_>>(),
                "{id}"
            );
        }
    }

    #[test]
    fn step5c3_admission_and_inventory_fail_closed() {
        let root = workspace_root();
        let mut plan = build_test_plan(&config()).unwrap();
        assert!(validate_active_type_elaboration_tags(&root, &plan).is_empty());
        let original = plan
            .cases
            .iter()
            .find(|case| case.id.0 == STEP5C3_ATTRIBUTE_CASES[0].0)
            .unwrap()
            .clone();
        let mut drift = original.clone();
        drift
            .expectation
            .tags
            .push("active_type_elaboration".to_owned());
        assert!(!is_active_type_elaboration(&drift));
        let mut drift = original.clone();
        drift.expectation.expected_phase = Some(PipelinePhase::TypeCheck);
        assert!(!is_active_type_elaboration(&drift));
        let mut drift = original.clone();
        drift.expectation.expected_outcome = ExpectedOutcome::Pass;
        assert!(!is_active_type_elaboration(&drift));

        let mut alias = original;
        alias.source_path = root.join(format!("alias/{}", STEP5C3_ATTRIBUTE_CASES[0].1));
        plan.cases.push(alias);
        assert!(
            validate_active_type_elaboration_tags(&root, &plan)
                .iter()
                .any(|diagnostic| diagnostic.code.0 == "E-TYPE-ELABORATION-ACTIVE-GATE")
        );

        let duplicate = plan
            .cases
            .iter()
            .find(|case| case.id.0 == STEP5C3_ATTRIBUTE_CASES[1].0)
            .unwrap()
            .clone();
        plan.cases.push(duplicate);
        assert!(
            validate_active_type_elaboration_tags(&root, &plan)
                .iter()
                .any(|diagnostic| diagnostic.code.0 == "E-TYPE-ELABORATION-STEP5C3-INVENTORY")
        );

        plan.cases
            .retain(|case| case.id.0 != STEP5C3_ATTRIBUTE_CASES[2].0);
        assert!(
            validate_active_type_elaboration_tags(&root, &plan)
                .iter()
                .any(|diagnostic| diagnostic
                    .detail_key
                    .ends_with(STEP5C3_ATTRIBUTE_CASES[2].0))
        );
    }

    #[test]
    fn step5c4_inventory_admission_and_duplicate_tags_are_exact() {
        assert_eq!(STEP5C4_MODE_CASES.len(), 6);
        assert_eq!(
            STEP5C4_MODE_CASES
                .iter()
                .map(|(id, source, _, _)| (*id, *source))
                .collect::<BTreeSet<_>>()
                .len(),
            6
        );
        let root = workspace_root();
        let mut plan = build_test_plan(&config()).unwrap();
        assert!(validate_active_type_elaboration_tags(&root, &plan).is_empty());
        for (id, _, phase, outcome) in STEP5C4_MODE_CASES {
            let case = plan.cases.iter().find(|case| case.id.0 == id).unwrap();
            assert!(is_active_type_elaboration(case), "{id}");
            assert_eq!(case.expectation.expected_phase, Some(phase));
            assert_eq!(case.expectation.expected_outcome, outcome);
        }
        let dependent = plan
            .cases
            .iter()
            .find(|case| case.id.0 == "pass_type_elaboration_mode_dependent_of_params_001")
            .unwrap()
            .clone();
        let mut duplicate_dependent = dependent;
        duplicate_dependent
            .expectation
            .tags
            .push(ACTIVE_TYPE_ELABORATION_TAG.to_owned());
        assert!(!is_active_type_elaboration(&duplicate_dependent));

        let original = plan
            .cases
            .iter()
            .find(|case| case.id.0 == STEP5C4_MODE_CASES[0].0)
            .unwrap()
            .clone();
        let mut duplicate = original.clone();
        duplicate
            .expectation
            .tags
            .push(ACTIVE_TYPE_ELABORATION_TAG.to_owned());
        assert!(!is_active_type_elaboration(&duplicate));
        plan.cases.push(duplicate);
        assert!(
            validate_active_type_elaboration_tags(&root, &plan)
                .iter()
                .any(|diagnostic| diagnostic.code.0 == "E-TYPE-ELABORATION-ACTIVE-GATE")
        );
        plan.cases
            .retain(|case| case.id.0 != STEP5C4_MODE_CASES[1].0);
        assert!(
            validate_active_type_elaboration_tags(&root, &plan)
                .iter()
                .any(|diagnostic| diagnostic.code.0 == "E-TYPE-ELABORATION-STEP5C4-INVENTORY")
        );
    }

    #[test]
    fn step5c4_type_rows_execute_with_exact_details_and_outcomes() {
        let report = crate::runner::run_type_elaboration_corpus(&config()).expect("type report");
        assert_eq!(report.error_count(), 0, "{:?}", report.diagnostics);
        let expected = [
            (super::STEP5C4_DEPENDENT_ID, Vec::new()),
            (
                "pass_type_elaboration_mode_attributed_struct_radix_001",
                Vec::new(),
            ),
            (
                "fail_type_elaboration_mode_dependent_arity_mismatch_001",
                vec!["modes.dependent.argument_arity_mismatch"],
            ),
            (
                "pass_type_elaboration_mode_property_impl_equals_001",
                Vec::new(),
            ),
            (
                "pass_type_elaboration_mode_property_impl_means_001",
                Vec::new(),
            ),
            (
                "fail_type_elaboration_mode_property_impl_unknown_property_001",
                vec!["modes.property_implementation.unknown_property"],
            ),
        ];
        for (id, keys) in expected {
            let result = report
                .results
                .iter()
                .find(|result| result.id.0 == id)
                .unwrap();
            assert_eq!(
                result.status,
                crate::runner::TypeElaborationCaseStatus::Passed,
                "{id}"
            );
            assert_eq!(
                result.actual_detail_keys,
                keys.into_iter().map(str::to_owned).collect::<Vec<_>>(),
                "{id}"
            );
        }
    }

    #[test]
    fn step5c5_inventory_admission_and_blocked_rows_are_exact() {
        assert_eq!(STEP5C5_CASES.len(), 10);
        assert_eq!(
            STEP5C5_CASES
                .iter()
                .map(|(id, source, _, _)| (*id, *source))
                .collect::<BTreeSet<_>>()
                .len(),
            10
        );
        let root = workspace_root();
        let mut plan = build_test_plan(&config()).unwrap();
        assert!(validate_active_type_elaboration_tags(&root, &plan).is_empty());
        for (id, _, phase, outcome) in STEP5C5_CASES {
            let case = plan.cases.iter().find(|case| case.id.0 == id).unwrap();
            assert!(is_active_type_elaboration(case), "{id}");
            assert_eq!(case.expectation.expected_phase, Some(phase));
            assert_eq!(case.expectation.expected_outcome, outcome);
        }
        for id in STEP5C5_BLOCKED_CASE_IDS {
            let mut blocked = plan
                .cases
                .iter()
                .find(|case| case.id.0 == id)
                .unwrap()
                .clone();
            blocked
                .expectation
                .tags
                .push(ACTIVE_TYPE_ELABORATION_TAG.to_owned());
            assert!(!is_active_type_elaboration(&blocked), "{id}");
        }

        let mut duplicate = plan
            .cases
            .iter()
            .find(|case| case.id.0 == STEP5C5_CASES[0].0)
            .unwrap()
            .clone();
        duplicate
            .expectation
            .tags
            .push(ACTIVE_TYPE_ELABORATION_TAG.to_owned());
        plan.cases.push(duplicate);
        assert!(
            validate_active_type_elaboration_tags(&root, &plan)
                .iter()
                .any(|diagnostic| diagnostic.code.0 == "E-TYPE-ELABORATION-ACTIVE-GATE")
        );
        plan.cases.retain(|case| case.id.0 != STEP5C5_CASES[1].0);
        assert!(
            validate_active_type_elaboration_tags(&root, &plan)
                .iter()
                .any(|diagnostic| diagnostic.code.0 == "E-TYPE-ELABORATION-STEP5C5-INVENTORY")
        );
    }

    #[test]
    fn step5c5_rows_execute_with_frozen_details() {
        let report = crate::runner::run_type_elaboration_corpus(&config()).expect("type report");
        assert_eq!(report.error_count(), 0, "{:?}", report.diagnostics);
        let expected = [
            (super::STEP5C5_PROPERTY_ID, Vec::new()),
            (
                "fail_type_elaboration_pred_property_arity_mismatch_001",
                vec!["predicates.property.arity_mismatch"],
            ),
            (
                "pass_type_elaboration_pred_properties_declaration_001",
                Vec::new(),
            ),
            (
                "pass_type_elaboration_func_builtin_bracket_pair_001",
                Vec::new(),
            ),
            (
                "fail_type_elaboration_func_equals_result_type_mismatch_001",
                vec!["functors.equals.result_type_mismatch"],
            ),
            (
                "fail_type_elaboration_func_means_missing_correctness_001",
                vec!["functors.means.missing_correctness"],
            ),
            (
                "fail_type_elaboration_func_duplicate_same_signature_001",
                vec!["functors.definition.duplicate_same_signature"],
            ),
            (
                "fail_type_elaboration_func_property_arity_mismatch_001",
                vec!["functors.property.arity_mismatch"],
            ),
            (
                "fail_type_elaboration_pred_duplicate_same_signature_001",
                vec!["predicates.definition.duplicate_same_signature"],
            ),
            (
                super::STEP5C5_ARGUMENT_ID,
                vec!["predicates.application.argument_type_mismatch"],
            ),
        ];
        for (id, keys) in expected {
            let result = report
                .results
                .iter()
                .find(|result| result.id.0 == id)
                .unwrap();
            assert_eq!(
                result.status,
                crate::runner::TypeElaborationCaseStatus::Passed,
                "{id}"
            );
            assert_eq!(
                result.actual_detail_keys,
                keys.into_iter().map(str::to_owned).collect::<Vec<_>>(),
                "{id}"
            );
        }
    }

    #[test]
    fn step5c5_predicate_conflict_admission_rejects_aliases_and_payloads() {
        use crate::expectation::TestKind;
        use crate::harness::TestCase;
        use crate::staged_model::Stage;
        let root = workspace_root();
        let plan = build_test_plan(&config()).unwrap();
        let original = plan
            .cases
            .iter()
            .find(|case| case.id.0 == STEP5C5_CASES[7].0)
            .unwrap();
        assert!(super::is_step5c5_workspace_member(&root, original));
        let mut legacy_alias = plan
            .cases
            .iter()
            .find(|case| case.id.0 == STEP5C5_CASES[0].0)
            .unwrap()
            .clone();
        legacy_alias.expectation.id = original.expectation.id.clone();
        assert!(super::is_step5c5_predicate_duplicate_candidate(
            &legacy_alias
        ));
        assert!(!is_active_type_elaboration(&legacy_alias));
        for mutate in [
            (|case: &mut TestCase| case.id.0 = "alias".into()) as fn(&mut TestCase),
            |case| case.expectation.id.0 = "alias".into(),
            |case| case.source_path = "alias.miz".into(),
            |case| case.expectation_path = "alias.expect.toml".into(),
            |case| case.expectation.source = "alias.miz".into(),
            |case| case.expectation.kind = TestKind::Pass,
            |case| case.expectation.expected_phase = Some(PipelinePhase::TypeCheck),
            |case| case.expectation.expected_outcome = ExpectedOutcome::Pass,
            |case| case.expectation.failure_category = Some("type_error".into()),
            |case| case.expectation.stable_detail_key = Some("unrelated".into()),
            |case| case.expectation.domain = "other".into(),
            |case| case.expectation.spec_refs.clear(),
            |case| case.expectation.spec_refs[0].0.push_str("_forged"),
            |case| case.expectation.tags.clear(),
            |case| {
                case.expectation
                    .tags
                    .push(ACTIVE_TYPE_ELABORATION_TAG.into())
            },
            |case| case.expectation.diagnostic_codes.push("forged".into()),
            |case| case.expectation.diagnostic_payloads.push("forged".into()),
            |case| {
                case.expectation
                    .declaration_symbol_payloads
                    .push("forged".into())
            },
            |case| case.expectation.rejection_reason = Some("forged".into()),
            |case| case.expectation.snapshots = Some("forged.snap".into()),
            |case| case.expectation.ast_profile = Some("forged".into()),
            |case| case.expectation.snapshot_profiles.push("forged".into()),
            |case| {
                case.id.0 = "alias".into();
                case.source_path = "alias.miz".into();
                case.expectation_path = "alias.expect.toml".into();
                case.expectation.source = "alias.miz".into();
            },
        ] {
            let mut changed = original.clone();
            mutate(&mut changed);
            assert!(super::is_step5c5_predicate_duplicate_candidate(&changed));
            assert!(!is_active_type_elaboration(&changed));
            assert!(!crate::runner::is_active_parse_only(&changed));
            assert!(!crate::runner::is_active_declaration_symbol(&changed));
            assert!(
                !crate::runner::formula_statement::is_active_formula_statement(&root, &changed)
            );
            assert!(!crate::runner::is_active_proof_verification(&changed));
        }
        for (stage, phase, tag) in [
            (Stage::ParseOnly, PipelinePhase::Parse, "active_parse_only"),
            (
                Stage::DeclarationSymbol,
                PipelinePhase::Resolve,
                "active_declaration_symbol",
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
            let mut changed = original.clone();
            changed.expectation.stage = stage;
            changed.expectation.expected_phase = Some(phase);
            changed.expectation.expected_outcome = ExpectedOutcome::Pass;
            changed.expectation.kind = TestKind::Pass;
            changed.expectation.stable_detail_key = None;
            changed.expectation.failure_category = None;
            changed.expectation.tags = vec![tag.into()];
            assert!(!is_active_type_elaboration(&changed));
            assert!(!crate::runner::is_active_parse_only(&changed));
            assert!(!crate::runner::is_active_declaration_symbol(&changed));
            assert!(
                !crate::runner::formula_statement::is_active_formula_statement(&root, &changed)
            );
            assert!(!crate::runner::is_active_proof_verification(&changed));
        }
        for sidecar in [false, true] {
            let mut changed = original.clone();
            let path = if sidecar {
                &mut changed.expectation_path
            } else {
                &mut changed.source_path
            };
            *path = root.join("alias").join(path.strip_prefix(&root).unwrap());
            assert!(!super::is_step5c5_workspace_member(&root, &changed));
        }
    }

    #[test]
    fn step5c5_argument_admission_execution_and_inventory_are_exact() {
        use crate::expectation::{
            Architecture22Gate, Architecture22Metadata, OriginMetadata, TestKind, TokenExpectation,
        };
        use crate::harness::TestCase;
        use crate::staged_model::Stage;
        let root = workspace_root();
        let plan = build_test_plan(&config()).unwrap();
        for id in [super::STEP5C5_ARGUMENT_ID, super::STEP5C5_PROPERTY_ID] {
            let original = plan.cases.iter().find(|case| case.id.0 == id).unwrap();
            assert!(super::step5c5_argument_admitted(Some(&root), original));
            assert!(validate_active_type_elaboration_tags(&root, &plan).is_empty());
            let result = crate::runner::run_type_elaboration_case(
                &root,
                &root.join("tests"),
                original,
                5605,
            );
            assert_eq!(
                result.status,
                crate::runner::TypeElaborationCaseStatus::Passed
            );
            assert_eq!(
                result.actual_detail_keys,
                if id == super::STEP5C5_ARGUMENT_ID {
                    vec!["predicates.application.argument_type_mismatch"]
                } else {
                    Vec::new()
                }
            );
            let rejected = |case: &TestCase| {
                assert!(super::is_step5c5_argument_candidate(case));
                assert!(!super::step5c5_argument_admitted(Some(&root), case));
                assert!(!is_active_type_elaboration(case));
                assert!(!crate::runner::is_active_parse_only(case));
                assert!(!crate::runner::is_active_declaration_symbol(case));
                assert!(
                    !crate::runner::formula_statement::is_active_formula_statement(&root, case)
                );
                assert!(!crate::runner::is_active_proof_verification(case));
            };
            for mutate in [
                (|case: &mut TestCase| case.id.0 = "alias".into()) as fn(&mut TestCase),
                |case| case.expectation.id.0 = "alias".into(),
                |case| case.source_path = "alias.miz".into(),
                |case| case.expectation_path = "alias.expect.toml".into(),
                |case| case.expectation.source = "alias.miz".into(),
                |case| case.expectation.schema_version = 2,
                |case| {
                    case.expectation.kind = if case.expectation.kind == TestKind::Pass {
                        TestKind::Fail
                    } else {
                        TestKind::Pass
                    }
                },
                |case| case.expectation.expected_phase = Some(PipelinePhase::Resolve),
                |case| {
                    case.expectation.expected_outcome =
                        if case.expectation.expected_outcome == ExpectedOutcome::Pass {
                            ExpectedOutcome::Fail
                        } else {
                            ExpectedOutcome::Pass
                        }
                },
                |case| {
                    case.expectation.failure_category =
                        if case.expectation.failure_category.is_some() {
                            None
                        } else {
                            Some("forged".into())
                        }
                },
                |case| case.expectation.stable_detail_key = Some("unrelated".into()),
                |case| case.expectation.domain = "other".into(),
                |case| case.expectation.spec_refs.clear(),
                |case| case.expectation.spec_refs[0].0.push_str("_forged"),
                |case| case.expectation.profiles.push("other".into()),
                |case| case.expectation.tags.clear(),
                |case| {
                    case.expectation
                        .tags
                        .push(ACTIVE_TYPE_ELABORATION_TAG.into())
                },
                |case| case.expectation.diagnostic_codes.push("forged".into()),
                |case| case.expectation.diagnostic_payloads.push("forged".into()),
                |case| {
                    case.expectation
                        .declaration_symbol_payloads
                        .push("forged".into())
                },
                |case| case.expectation.rejection_reason = Some("forged".into()),
                |case| case.expectation.snapshots = Some("forged.snap".into()),
                |case| case.expectation.ast_profile = Some("forged".into()),
                |case| case.expectation.snapshot_profiles.push("forged".into()),
                |case| {
                    case.expectation.tokens.push(TokenExpectation {
                        kind: "Identifier".into(),
                        lexeme: "X".into(),
                        span_start: None,
                        span_end: None,
                        span_start_line: None,
                        span_start_col: None,
                        span_end_line: None,
                        span_end_col: None,
                    })
                },
                |case| {
                    case.expectation.origin = Some(OriginMetadata {
                        schema_version: 1,
                        kind: TestKind::Fail,
                        generator: "forged".into(),
                        generator_version: "1".into(),
                        seed: "0".into(),
                        profile: "forged".into(),
                        expected_outcome: ExpectedOutcome::Fail,
                        minimized: false,
                        original_failure_category: None,
                    })
                },
                |case| {
                    case.expectation.architecture22 = Some(Architecture22Metadata {
                        scenarios: vec!["forged".into()],
                        equivalence_class: None,
                        gate: Architecture22Gate::Planned,
                    })
                },
            ] {
                let mut changed = original.clone();
                mutate(&mut changed);
                rejected(&changed);
            }
            let mut legacy = plan
                .cases
                .iter()
                .find(|case| case.id.0 == STEP5C5_CASES[0].0)
                .unwrap()
                .clone();
            legacy.expectation.id = original.expectation.id.clone();
            rejected(&legacy);
            for (stage, phase, tag) in [
                (Stage::ParseOnly, PipelinePhase::Parse, "active_parse_only"),
                (
                    Stage::DeclarationSymbol,
                    PipelinePhase::Resolve,
                    "active_declaration_symbol",
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
                let mut changed = original.clone();
                changed.expectation.stage = stage;
                changed.expectation.expected_phase = Some(phase);
                changed.expectation.expected_outcome = ExpectedOutcome::Pass;
                changed.expectation.kind = TestKind::Pass;
                changed.expectation.failure_category = None;
                changed.expectation.stable_detail_key = None;
                changed.expectation.tags = vec![tag.into()];
                rejected(&changed);
            }
            for sidecar in [false, true] {
                let mut changed = original.clone();
                let path = if sidecar {
                    &mut changed.expectation_path
                } else {
                    &mut changed.source_path
                };
                *path = root.join("alias").join(path.strip_prefix(&root).unwrap());
                assert!(!super::step5c5_argument_admitted(Some(&root), &changed));
            }
            for duplicate in [false, true] {
                let mut changed = plan.clone();
                if duplicate {
                    changed.cases.push(original.clone());
                } else {
                    changed.cases.retain(|case| case.id != original.id);
                }
                assert!(
                    validate_active_type_elaboration_tags(&root, &changed)
                        .iter()
                        .any(|diagnostic| diagnostic.code.0
                            == "E-TYPE-ELABORATION-STEP5C5-INVENTORY")
                );
            }
        }
    }

    #[test]
    fn step5c3_argument_admission_execution_and_inventory_are_exact() {
        use crate::expectation::{
            Architecture22Gate, Architecture22Metadata, OriginMetadata, TestKind, TokenExpectation,
        };
        use crate::harness::TestCase;
        use crate::staged_model::Stage;
        let root = workspace_root();
        let plan = build_test_plan(&config()).unwrap();
        let original = plan
            .cases
            .iter()
            .find(|case| case.id.0 == super::STEP5C3_ARGUMENT_ID)
            .unwrap();
        assert!(super::step5c3_argument_admitted(Some(&root), original));
        assert!(validate_active_type_elaboration_tags(&root, &plan).is_empty());
        let result =
            crate::runner::run_type_elaboration_case(&root, &root.join("tests"), original, 5603);
        assert_eq!(
            result.status,
            crate::runner::TypeElaborationCaseStatus::Passed
        );
        assert_eq!(
            result.actual_detail_keys,
            ["types.application.argument_type_mismatch"]
        );
        let rejected = |case: &TestCase| {
            assert!(super::is_step5c3_argument_candidate(case));
            assert!(!super::step5c3_argument_admitted(Some(&root), case));
            assert!(!is_active_type_elaboration(case));
            assert!(!crate::runner::is_active_parse_only(case));
            assert!(!crate::runner::is_active_declaration_symbol(case));
            assert!(!crate::runner::formula_statement::is_active_formula_statement(&root, case));
            assert!(!crate::runner::is_active_proof_verification(case));
        };
        for mutate in [
            (|case: &mut TestCase| case.id.0 = "alias".into()) as fn(&mut TestCase),
            |case| case.expectation.id.0 = "alias".into(),
            |case| case.source_path = "alias.miz".into(),
            |case| case.expectation_path = "alias.expect.toml".into(),
            |case| case.expectation.source = "alias.miz".into(),
            |case| case.expectation.schema_version = 2,
            |case| case.expectation.kind = TestKind::Pass,
            |case| case.expectation.expected_phase = Some(PipelinePhase::Resolve),
            |case| case.expectation.expected_outcome = ExpectedOutcome::Pass,
            |case| case.expectation.failure_category = None,
            |case| case.expectation.stable_detail_key = Some("unrelated".into()),
            |case| case.expectation.domain = "other".into(),
            |case| case.expectation.spec_refs.clear(),
            |case| case.expectation.spec_refs[0].0.push_str("_forged"),
            |case| case.expectation.profiles.push("other".into()),
            |case| case.expectation.tags.clear(),
            |case| {
                case.expectation
                    .tags
                    .push(ACTIVE_TYPE_ELABORATION_TAG.into())
            },
            |case| case.expectation.diagnostic_codes.push("forged".into()),
            |case| case.expectation.diagnostic_payloads.push("forged".into()),
            |case| {
                case.expectation
                    .declaration_symbol_payloads
                    .push("forged".into())
            },
            |case| case.expectation.rejection_reason = Some("forged".into()),
            |case| case.expectation.snapshots = Some("forged.snap".into()),
            |case| case.expectation.ast_profile = Some("forged".into()),
            |case| case.expectation.snapshot_profiles.push("forged".into()),
            |case| {
                case.expectation.tokens.push(TokenExpectation {
                    kind: "Identifier".into(),
                    lexeme: "X".into(),
                    span_start: None,
                    span_end: None,
                    span_start_line: None,
                    span_start_col: None,
                    span_end_line: None,
                    span_end_col: None,
                })
            },
            |case| {
                case.expectation.origin = Some(OriginMetadata {
                    schema_version: 1,
                    kind: TestKind::Fail,
                    generator: "forged".into(),
                    generator_version: "1".into(),
                    seed: "0".into(),
                    profile: "forged".into(),
                    expected_outcome: ExpectedOutcome::Fail,
                    minimized: false,
                    original_failure_category: None,
                })
            },
            |case| {
                case.expectation.architecture22 = Some(Architecture22Metadata {
                    scenarios: vec!["forged".into()],
                    equivalence_class: None,
                    gate: Architecture22Gate::Planned,
                })
            },
        ] {
            let mut changed = original.clone();
            mutate(&mut changed);
            rejected(&changed);
        }
        let mut legacy = plan
            .cases
            .iter()
            .find(|case| case.id.0 == STEP5C3_ATTRIBUTE_CASES[0].0)
            .unwrap()
            .clone();
        legacy.expectation.id = original.expectation.id.clone();
        rejected(&legacy);
        for (stage, phase, tag) in [
            (Stage::ParseOnly, PipelinePhase::Parse, "active_parse_only"),
            (
                Stage::DeclarationSymbol,
                PipelinePhase::Resolve,
                "active_declaration_symbol",
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
            let mut changed = original.clone();
            changed.expectation.stage = stage;
            changed.expectation.expected_phase = Some(phase);
            changed.expectation.expected_outcome = ExpectedOutcome::Pass;
            changed.expectation.kind = TestKind::Pass;
            changed.expectation.failure_category = None;
            changed.expectation.stable_detail_key = None;
            changed.expectation.tags = vec![tag.into()];
            rejected(&changed);
        }
        for sidecar in [false, true] {
            let mut changed = original.clone();
            let path = if sidecar {
                &mut changed.expectation_path
            } else {
                &mut changed.source_path
            };
            *path = root.join("alias").join(path.strip_prefix(&root).unwrap());
            assert!(!super::step5c3_argument_admitted(Some(&root), &changed));
        }
        for duplicate in [false, true] {
            let mut changed = plan.clone();
            if duplicate {
                changed.cases.push(original.clone());
            } else {
                changed.cases.retain(|case| case.id != original.id);
            }
            assert!(
                validate_active_type_elaboration_tags(&root, &changed)
                    .iter()
                    .any(|diagnostic| diagnostic.code.0 == "E-TYPE-ELABORATION-STEP5C3-INVENTORY")
            );
        }
    }

    #[test]
    fn step5c4_dependent_admission_execution_and_inventory_are_exact() {
        use crate::expectation::{
            Architecture22Gate, Architecture22Metadata, OriginMetadata, TestKind, TokenExpectation,
        };
        use crate::harness::TestCase;
        use crate::staged_model::Stage;
        let root = workspace_root();
        let plan = build_test_plan(&config()).unwrap();
        let original = plan
            .cases
            .iter()
            .find(|case| case.id.0 == super::STEP5C4_DEPENDENT_ID)
            .unwrap();
        assert!(super::step5c4_dependent_admitted(Some(&root), original));
        assert!(validate_active_type_elaboration_tags(&root, &plan).is_empty());
        let result =
            crate::runner::run_type_elaboration_case(&root, &root.join("tests"), original, 5604);
        assert_eq!(
            result.status,
            crate::runner::TypeElaborationCaseStatus::Passed
        );
        assert!(result.actual_detail_keys.is_empty());
        let rejected = |case: &TestCase| {
            assert!(super::is_step5c4_dependent_candidate(case));
            assert!(!super::step5c4_dependent_admitted(Some(&root), case));
            assert!(!is_active_type_elaboration(case));
            assert!(!crate::runner::is_active_parse_only(case));
            assert!(!crate::runner::is_active_declaration_symbol(case));
            assert!(!crate::runner::formula_statement::is_active_formula_statement(&root, case));
            assert!(!crate::runner::is_active_proof_verification(case));
        };
        for mutate in [
            (|case: &mut TestCase| case.id.0 = "alias".into()) as fn(&mut TestCase),
            |case| case.expectation.id.0 = "alias".into(),
            |case| case.source_path = "alias.miz".into(),
            |case| case.expectation_path = "alias.expect.toml".into(),
            |case| case.expectation.source = "alias.miz".into(),
            |case| case.expectation.schema_version = 2,
            |case| case.expectation.kind = TestKind::Fail,
            |case| case.expectation.expected_phase = Some(PipelinePhase::Resolve),
            |case| case.expectation.expected_outcome = ExpectedOutcome::Fail,
            |case| case.expectation.failure_category = Some("type_error".into()),
            |case| case.expectation.stable_detail_key = Some("unrelated".into()),
            |case| case.expectation.domain = "other".into(),
            |case| case.expectation.spec_refs.clear(),
            |case| case.expectation.spec_refs[0].0.push_str("_forged"),
            |case| case.expectation.profiles.push("other".into()),
            |case| case.expectation.tags.clear(),
            |case| {
                case.expectation
                    .tags
                    .push(ACTIVE_TYPE_ELABORATION_TAG.into())
            },
            |case| case.expectation.diagnostic_codes.push("forged".into()),
            |case| case.expectation.diagnostic_payloads.push("forged".into()),
            |case| {
                case.expectation
                    .declaration_symbol_payloads
                    .push("forged".into())
            },
            |case| case.expectation.rejection_reason = Some("forged".into()),
            |case| case.expectation.snapshots = Some("forged.snap".into()),
            |case| case.expectation.ast_profile = Some("forged".into()),
            |case| case.expectation.snapshot_profiles.push("forged".into()),
            |case| {
                case.expectation.tokens.push(TokenExpectation {
                    kind: "Identifier".into(),
                    lexeme: "X".into(),
                    span_start: None,
                    span_end: None,
                    span_start_line: None,
                    span_start_col: None,
                    span_end_line: None,
                    span_end_col: None,
                })
            },
            |case| {
                case.expectation.origin = Some(OriginMetadata {
                    schema_version: 1,
                    kind: TestKind::Fail,
                    generator: "forged".into(),
                    generator_version: "1".into(),
                    seed: "0".into(),
                    profile: "forged".into(),
                    expected_outcome: ExpectedOutcome::Fail,
                    minimized: false,
                    original_failure_category: None,
                })
            },
            |case| {
                case.expectation.architecture22 = Some(Architecture22Metadata {
                    scenarios: vec!["forged".into()],
                    equivalence_class: None,
                    gate: Architecture22Gate::Planned,
                })
            },
        ] {
            let mut changed = original.clone();
            mutate(&mut changed);
            rejected(&changed);
        }
        let mut legacy = plan
            .cases
            .iter()
            .find(|case| case.id.0 == STEP5C4_MODE_CASES[0].0)
            .unwrap()
            .clone();
        legacy.expectation.id = original.expectation.id.clone();
        rejected(&legacy);
        for (stage, phase, tag) in [
            (Stage::ParseOnly, PipelinePhase::Parse, "active_parse_only"),
            (
                Stage::DeclarationSymbol,
                PipelinePhase::Resolve,
                "active_declaration_symbol",
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
            let mut changed = original.clone();
            changed.expectation.stage = stage;
            changed.expectation.expected_phase = Some(phase);
            changed.expectation.expected_outcome = ExpectedOutcome::Pass;
            changed.expectation.kind = TestKind::Pass;
            changed.expectation.failure_category = None;
            changed.expectation.stable_detail_key = None;
            changed.expectation.tags = vec![tag.into()];
            rejected(&changed);
        }
        for sidecar in [false, true] {
            let mut changed = original.clone();
            let path = if sidecar {
                &mut changed.expectation_path
            } else {
                &mut changed.source_path
            };
            *path = root.join("alias").join(path.strip_prefix(&root).unwrap());
            assert!(!super::step5c4_dependent_admitted(Some(&root), &changed));
        }
        for duplicate in [false, true] {
            let mut changed = plan.clone();
            if duplicate {
                changed.cases.push(original.clone());
            } else {
                changed.cases.retain(|case| case.id != original.id);
            }
            assert!(
                validate_active_type_elaboration_tags(&root, &changed)
                    .iter()
                    .any(|diagnostic| diagnostic.code.0 == "E-TYPE-ELABORATION-STEP5C4-INVENTORY")
            );
        }
    }

    #[test]
    fn step5c6_synonym_admission_execution_and_inventory_are_exact() {
        use crate::expectation::{
            Architecture22Gate, Architecture22Metadata, OriginMetadata, TestKind, TokenExpectation,
        };
        use crate::harness::TestCase;
        use crate::staged_model::Stage;
        let root = workspace_root();
        let plan = build_test_plan(&config()).unwrap();
        let original = plan
            .cases
            .iter()
            .find(|case| case.id.0 == super::STEP5C6_ALIAS_IDS[0])
            .unwrap();
        let positive = plan
            .cases
            .iter()
            .find(|case| case.id.0 == super::STEP5C6_ALIAS_IDS[1])
            .unwrap();
        let antonym = plan
            .cases
            .iter()
            .find(|case| case.id.0 == super::STEP5C6_ALIAS_IDS[2])
            .unwrap();
        let originals = [original, positive, antonym];
        assert!(validate_active_type_elaboration_tags(&root, &plan).is_empty());
        for original in originals {
            assert!(super::step5c6_synonym_admitted(Some(&root), original));
            let result = crate::runner::run_type_elaboration_case(
                &root,
                &root.join("tests"),
                original,
                5606,
            );
            assert_eq!(
                result.status,
                crate::runner::TypeElaborationCaseStatus::Passed
            );
            assert_eq!(
                result.actual_detail_keys,
                if original.expectation.kind == TestKind::Fail {
                    vec!["notation.synonym.loci_mismatch".to_owned()]
                } else {
                    vec![]
                }
            );
        }
        let rejected = |case: &TestCase| {
            assert!(super::is_step5c6_alias_candidate(case));
            assert!(!super::step5c6_synonym_admitted(Some(&root), case));
            assert!(!is_active_type_elaboration(case));
            assert!(!crate::runner::is_active_parse_only(case));
            assert!(!crate::runner::is_active_declaration_symbol(case));
            assert!(!crate::runner::formula_statement::is_active_formula_statement(&root, case));
            assert!(!crate::runner::is_active_proof_verification(case));
        };
        for mutate in [
            (|case: &mut TestCase| case.id.0 = "alias".into()) as fn(&mut TestCase),
            |case| case.expectation.id.0 = "alias".into(),
            |case| case.source_path = "alias.miz".into(),
            |case| case.expectation_path = "alias.expect.toml".into(),
            |case| case.expectation.source = "alias.miz".into(),
            |case| case.expectation.schema_version = 2,
            |case| {
                case.expectation.kind = if case.expectation.kind == TestKind::Pass {
                    TestKind::Fail
                } else {
                    TestKind::Pass
                }
            },
            |case| {
                case.expectation.expected_phase = Some(
                    if case.expectation.expected_phase == Some(PipelinePhase::TypeCheck) {
                        PipelinePhase::Resolve
                    } else {
                        PipelinePhase::TypeCheck
                    },
                )
            },
            |case| {
                case.expectation.expected_outcome =
                    if case.expectation.expected_outcome == ExpectedOutcome::Pass {
                        ExpectedOutcome::Fail
                    } else {
                        ExpectedOutcome::Pass
                    }
            },
            |case| case.expectation.failure_category = Some("type_error".into()),
            |case| case.expectation.stable_detail_key = Some("unrelated".into()),
            |case| case.expectation.domain = "other".into(),
            |case| case.expectation.spec_refs.clear(),
            |case| case.expectation.spec_refs[0].0.push_str("_forged"),
            |case| case.expectation.profiles.push("other".into()),
            |case| case.expectation.tags.clear(),
            |case| {
                case.expectation
                    .tags
                    .push(ACTIVE_TYPE_ELABORATION_TAG.into())
            },
            |case| case.expectation.diagnostic_codes.push("forged".into()),
            |case| case.expectation.diagnostic_payloads.push("forged".into()),
            |case| {
                case.expectation
                    .declaration_symbol_payloads
                    .push("forged".into())
            },
            |case| case.expectation.rejection_reason = Some("forged".into()),
            |case| case.expectation.snapshots = Some("forged.snap".into()),
            |case| case.expectation.ast_profile = Some("forged".into()),
            |case| case.expectation.snapshot_profiles.push("forged".into()),
            |case| {
                case.expectation.tokens.push(TokenExpectation {
                    kind: "Identifier".into(),
                    lexeme: "X".into(),
                    span_start: None,
                    span_end: None,
                    span_start_line: None,
                    span_start_col: None,
                    span_end_line: None,
                    span_end_col: None,
                })
            },
            |case| {
                case.expectation.origin = Some(OriginMetadata {
                    schema_version: 1,
                    kind: TestKind::Fail,
                    generator: "forged".into(),
                    generator_version: "1".into(),
                    seed: "0".into(),
                    profile: "forged".into(),
                    expected_outcome: ExpectedOutcome::Fail,
                    minimized: false,
                    original_failure_category: None,
                })
            },
            |case| {
                case.expectation.architecture22 = Some(Architecture22Metadata {
                    scenarios: vec!["forged".into()],
                    equivalence_class: None,
                    gate: Architecture22Gate::Planned,
                })
            },
        ] {
            for original in originals {
                let mut changed = original.clone();
                mutate(&mut changed);
                rejected(&changed);
            }
        }
        let mut legacy = plan
            .cases
            .iter()
            .find(|case| case.id.0 == STEP5C5_CASES[0].0)
            .unwrap()
            .clone();
        for original in originals {
            legacy.expectation.id = original.expectation.id.clone();
            rejected(&legacy);
        }
        for (stage, phase, tag) in [
            (Stage::ParseOnly, PipelinePhase::Parse, "active_parse_only"),
            (
                Stage::DeclarationSymbol,
                PipelinePhase::Resolve,
                "active_declaration_symbol",
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
            for original in originals {
                let mut changed = original.clone();
                changed.expectation.stage = stage;
                changed.expectation.expected_phase = Some(phase);
                changed.expectation.expected_outcome = ExpectedOutcome::Pass;
                changed.expectation.kind = TestKind::Pass;
                changed.expectation.failure_category = None;
                changed.expectation.stable_detail_key = None;
                changed.expectation.tags = vec![tag.into()];
                rejected(&changed);
            }
        }
        for sidecar in [false, true] {
            for original in originals {
                let mut changed = original.clone();
                let path = if sidecar {
                    &mut changed.expectation_path
                } else {
                    &mut changed.source_path
                };
                *path = root.join("alias").join(path.strip_prefix(&root).unwrap());
                assert!(!super::step5c6_synonym_admitted(Some(&root), &changed));
            }
        }
        for duplicate in [false, true] {
            for original in originals {
                let mut changed = plan.clone();
                if duplicate {
                    changed.cases.push(original.clone());
                } else {
                    changed.cases.retain(|case| case.id != original.id);
                }
                assert!(
                    validate_active_type_elaboration_tags(&root, &changed)
                        .iter()
                        .any(|diagnostic| diagnostic.code.0
                            == "E-TYPE-ELABORATION-STEP5C6-INVENTORY")
                );
            }
        }
    }

    #[test]
    fn step5c7_admission_and_inventory_are_exact() {
        assert_eq!(STEP5C7_TERM_CASES.len(), 5);
        assert_eq!(
            STEP5C7_TERM_CASES
                .iter()
                .map(|(id, source, _, _, _)| (*id, *source))
                .collect::<BTreeSet<_>>()
                .len(),
            5
        );
        let root = workspace_root();
        let mut plan = build_test_plan(&config()).unwrap();
        assert!(validate_active_type_elaboration_tags(&root, &plan).is_empty());
        let blocked_id = "fail_type_elaboration_term_qua_invalid_narrowing_001";
        let mut blocked = plan
            .cases
            .iter()
            .find(|case| case.id.0 == blocked_id)
            .unwrap()
            .clone();
        blocked.expectation.tags = vec![ACTIVE_TYPE_ELABORATION_TAG.to_owned()];
        assert!(!is_active_type_elaboration(&blocked));
        let mut blocked_plan = plan.clone();
        blocked_plan.cases.push(blocked);
        assert!(
            validate_active_type_elaboration_tags(&root, &blocked_plan)
                .iter()
                .any(|diagnostic| diagnostic.detail_key.ends_with(blocked_id))
        );
        for (id, _, phase, outcome, _) in STEP5C7_TERM_CASES {
            let case = plan.cases.iter().find(|case| case.id.0 == id).unwrap();
            assert!(is_active_type_elaboration(case), "{id}");
            assert_eq!(case.expectation.expected_phase, Some(phase));
            assert_eq!(case.expectation.expected_outcome, outcome);
        }

        let original = plan
            .cases
            .iter()
            .find(|case| case.id.0 == STEP5C7_TERM_CASES[0].0)
            .unwrap()
            .clone();
        let mut extra_tag = original.clone();
        extra_tag
            .expectation
            .tags
            .push(ACTIVE_TYPE_ELABORATION_TAG.to_owned());
        assert!(!is_active_type_elaboration(&extra_tag));
        let mut missing_tag = original.clone();
        missing_tag.expectation.tags.clear();
        assert!(!is_active_type_elaboration(&missing_tag));
        let mut wrong_phase = original.clone();
        wrong_phase.expectation.expected_phase = Some(PipelinePhase::Resolve);
        assert!(!is_active_type_elaboration(&wrong_phase));
        let mut wrong_outcome = original.clone();
        wrong_outcome.expectation.expected_outcome = ExpectedOutcome::Pass;
        assert!(!is_active_type_elaboration(&wrong_outcome));
        let mut wrong_key = original.clone();
        wrong_key.expectation.stable_detail_key = Some("terms.wrong".to_owned());
        assert!(!is_active_type_elaboration(&wrong_key));
        let mut wrong_id = original.clone();
        wrong_id.id.0 = "pass_type_elaboration_term_unlisted_001".to_owned();
        assert!(!is_active_type_elaboration(&wrong_id));
        let mut wrong_sidecar = original.clone();
        wrong_sidecar.expectation_path = root.join("alias/wrong.expect.toml");
        assert!(!is_active_type_elaboration(&wrong_sidecar));
        plan.cases.push(wrong_sidecar);
        let mut wrong_root = original.clone();
        wrong_root.source_path = root.join(format!("alias/{}", STEP5C7_TERM_CASES[0].1));
        plan.cases.push(wrong_root);
        assert!(
            validate_active_type_elaboration_tags(&root, &plan)
                .iter()
                .any(|diagnostic| diagnostic.code.0 == "E-TYPE-ELABORATION-ACTIVE-GATE")
        );

        let duplicate = plan
            .cases
            .iter()
            .find(|case| case.id.0 == STEP5C7_TERM_CASES[1].0)
            .unwrap()
            .clone();
        plan.cases.push(duplicate);
        assert!(
            validate_active_type_elaboration_tags(&root, &plan)
                .iter()
                .any(|diagnostic| diagnostic.code.0 == "E-TYPE-ELABORATION-STEP5C7-INVENTORY")
        );
        plan.cases
            .retain(|case| case.id.0 != STEP5C7_TERM_CASES[2].0);
        assert!(
            validate_active_type_elaboration_tags(&root, &plan)
                .iter()
                .any(|diagnostic| diagnostic.detail_key.ends_with(STEP5C7_TERM_CASES[2].0))
        );
    }

    #[test]
    fn step5c7_rows_execute_with_frozen_details() {
        let report = crate::runner::run_type_elaboration_corpus(&config()).expect("type report");
        assert_eq!(report.error_count(), 0, "{:?}", report.diagnostics);
        let expected = [
            (
                "fail_type_elaboration_term_choice_uninhabited_001",
                vec!["terms.choice.missing_inhabitation"],
            ),
            ("pass_type_elaboration_term_choice_builtin_001", Vec::new()),
            (
                "pass_type_elaboration_term_numeral_equality_001",
                Vec::new(),
            ),
            ("pass_type_elaboration_term_qua_widening_001", Vec::new()),
            (
                "fail_type_elaboration_term_comprehension_unbound_mapper_001",
                vec!["terms.comprehension.unbound_mapper_variable"],
            ),
        ];
        for (id, keys) in expected {
            let result = report
                .results
                .iter()
                .find(|result| result.id.0 == id)
                .unwrap();
            assert_eq!(
                result.status,
                crate::runner::TypeElaborationCaseStatus::Passed,
                "{id}: {:?}",
                result.actual_detail_keys
            );
            assert_eq!(
                result.actual_detail_keys,
                keys.into_iter().map(str::to_owned).collect::<Vec<_>>(),
                "{id}"
            );
        }
    }

    #[test]
    fn step5c2_exact_type_rows_execute_with_frozen_detail_keys() {
        let report = crate::runner::run_type_elaboration_corpus(&config()).expect("type report");
        let expected = [
            Some("structures.constructor.missing_field_argument"),
            Some("structures.definition.duplicate_member"),
            None,
            None,
            None,
            Some("structures.inherit.diamond_inconsistency"),
            None,
            None,
            Some("structures.inherit.uncovered_base_member"),
            Some("structures.inherit.unknown_source_member"),
            None,
            Some("structures.selector.unknown_field"),
        ];
        for ((id, _, _, _), detail_key) in STEP5C2_STRUCTURE_CASES.iter().zip(expected) {
            let matches = report
                .results
                .iter()
                .filter(|result| result.id.0 == *id)
                .collect::<Vec<_>>();
            assert_eq!(matches.len(), 1, "{id}");
            let result = matches[0];
            assert_eq!(
                result.status,
                crate::runner::TypeElaborationCaseStatus::Passed,
                "{id}: {:?}",
                result.actual_detail_keys
            );
            assert_eq!(
                result.actual_detail_keys,
                detail_key
                    .into_iter()
                    .map(str::to_owned)
                    .collect::<Vec<_>>(),
                "{id}"
            );
        }
    }

    #[test]
    fn step5c2_resolve_admission_and_inventory_fail_closed() {
        let root = workspace_root();
        let mut plan = build_test_plan(&config()).unwrap();
        let original = plan
            .cases
            .iter()
            .find(|case| case.id.0 == STEP5C2_STRUCTURE_CASES[1].0)
            .unwrap()
            .clone();
        assert!(is_active_type_elaboration(&original));

        let mut duplicate_tag = original.clone();
        duplicate_tag
            .expectation
            .tags
            .push("active_type_elaboration".to_owned());
        assert!(!is_active_type_elaboration(&duplicate_tag));

        let mut wrong_phase = original.clone();
        wrong_phase.expectation.expected_phase = Some(PipelinePhase::TypeCheck);
        assert!(!is_active_type_elaboration(&wrong_phase));

        let mut wrong_outcome = original.clone();
        wrong_outcome.expectation.expected_outcome = ExpectedOutcome::Pass;
        assert!(!is_active_type_elaboration(&wrong_outcome));

        let mut alias = original.clone();
        alias.source_path = root.join(format!("alias/{}", STEP5C2_STRUCTURE_CASES[1].1));
        assert!(alias.source_path.ends_with(STEP5C2_STRUCTURE_CASES[1].1));
        plan.cases.push(alias);
        assert!(
            validate_active_type_elaboration_tags(&root, &plan)
                .iter()
                .any(|diagnostic| diagnostic.code.0 == "E-TYPE-ELABORATION-ACTIVE-GATE")
        );

        let duplicate = plan
            .cases
            .iter()
            .find(|case| case.id.0 == STEP5C2_STRUCTURE_CASES[0].0)
            .unwrap()
            .clone();
        plan.cases.push(duplicate);
        assert!(
            validate_active_type_elaboration_tags(&root, &plan)
                .iter()
                .any(|diagnostic| { diagnostic.code.0 == "E-TYPE-ELABORATION-STEP5C2-INVENTORY" })
        );

        plan.cases
            .retain(|case| case.id.0 != STEP5C2_STRUCTURE_CASES[2].0);
        assert!(
            validate_active_type_elaboration_tags(&root, &plan)
                .iter()
                .any(|diagnostic| diagnostic
                    .detail_key
                    .ends_with(STEP5C2_STRUCTURE_CASES[2].0))
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
