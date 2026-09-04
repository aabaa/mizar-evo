use std::path::Path;

use crate::diagnostic::ValidationDiagnostic;
use crate::expectation::{ExpectedOutcome, PipelinePhase};
use crate::harness::{TestCase, TestPlan};
use crate::staged_model::Stage;

use super::super::syntax_smoke::workspace_relative_source;

const ACTIVE_TYPE_ELABORATION_TAG: &str = "active_type_elaboration";

const STEP5C3_ATTRIBUTE_CASES: [(&str, &str, PipelinePhase, ExpectedOutcome); 5] = [
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
];

const STEP5C3_G1_CASE_IDS: [&str; 2] = [
    "fail_type_elaboration_argument_type_mismatch_functor_001",
    "pass_type_elaboration_argument_attribute_widening_001",
];

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
    let exact_step5c1 = step5c1_case(case).is_some()
        && case.expectation.tags.as_slice() == [ACTIVE_TYPE_ELABORATION_TAG];
    let exact_step5c2 = step5c2_case(case).is_some()
        && case.expectation.tags.as_slice() == [ACTIVE_TYPE_ELABORATION_TAG];
    let exact_step5c3 = step5c3_case(case).is_some()
        && case.expectation.tags.as_slice() == [ACTIVE_TYPE_ELABORATION_TAG];
    has_active_type_elaboration_tag(case)
        && !is_step5c3_g1_id(case)
        && case.expectation.stage == Stage::TypeElaboration
        && (case.expectation.expected_phase == Some(PipelinePhase::TypeCheck)
            || (exact_step5c1 || exact_step5c2 || exact_step5c3)
                && case.expectation.expected_phase == Some(PipelinePhase::Resolve))
        && matches!(
            case.expectation.expected_outcome,
            ExpectedOutcome::Pass | ExpectedOutcome::Fail
        )
        && (!is_step5c1_id(case) || exact_step5c1)
        && (!is_step5c2_id(case) || exact_step5c2)
        && (!is_step5c3_id(case) || exact_step5c3)
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
            || is_step5c1_id(case)
            || is_step5c2_id(case)
            || is_step5c3_id(case)
            || is_step5c3_g1_id(case) && has_active_type_elaboration_tag(case)
    }) {
        if !is_active_type_elaboration(case)
            || is_step5c1_id(case) && !is_step5c1_workspace_member(workspace_root, case)
            || is_step5c2_id(case) && !is_step5c2_workspace_member(workspace_root, case)
            || is_step5c3_id(case) && !is_step5c3_workspace_member(workspace_root, case)
            || is_step5c3_g1_id(case) && has_active_type_elaboration_tag(case)
        {
            diagnostics.push(
                ValidationDiagnostic::error(
                    &case.expectation_path,
                    "type_elaboration",
                    "E-TYPE-ELABORATION-ACTIVE-GATE",
                    format!("type_elaboration.active_gate.{}", case.id.0),
                "active_type_elaboration cases must be exact .miz pass/fail expectations at stage type_elaboration; only the frozen Step 5C.1/5C.2/5C.3 inventories may use phase resolve",
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
        &STEP5C3_ATTRIBUTE_CASES,
        "E-TYPE-ELABORATION-STEP5C3-INVENTORY",
        "step5c3_inventory",
    )
    .into_iter()
    .for_each(|diagnostic| diagnostics.push(diagnostic));
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
    STEP5C3_ATTRIBUTE_CASES
        .iter()
        .any(|(id, _, _, _)| case.id.0 == *id)
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
    step5c3_case(case).is_some_and(|(_, source, _, _)| {
        workspace_relative_source(workspace_root, &case.source_path)
            .is_some_and(|actual| actual == source)
    })
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
        STEP5C1_VARIABLE_CASES, STEP5C2_STRUCTURE_CASES, STEP5C3_ATTRIBUTE_CASES,
        STEP5C3_G1_CASE_IDS, is_active_type_elaboration, validate_active_type_elaboration_tags,
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
    fn step5c3_inventory_has_five_unique_id_source_pairs_and_excludes_g1() {
        assert_eq!(STEP5C3_ATTRIBUTE_CASES.len(), 5);
        assert_eq!(
            STEP5C3_ATTRIBUTE_CASES
                .iter()
                .map(|(id, source, _, _)| (*id, *source))
                .collect::<BTreeSet<_>>()
                .len(),
            5
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
