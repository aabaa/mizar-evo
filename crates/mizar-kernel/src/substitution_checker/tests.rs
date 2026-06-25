use super::*;
use crate::{
    certificate_parser::{
        ClauseTautologyPolicy, FinalGoalNamespace, FinalGoalRef, Fingerprint, KernelProfileRecord,
        ParsedCertificateTestParts, SymbolManifestEntry, VariableManifestEntry,
    },
    clause::{SymbolKey, SymbolKind},
};

#[test]
fn valid_direct_substitution_rewrites_only_recorded_path_and_reports_checked_data() {
    let target = target();
    let source = pair(var(1), var(1));
    let target_term = pair(var(2), var(1));
    let certificate = certificate(vec![substitution(1, source.clone(), target_term.clone())]);
    let context = context(vec![payload(
        1,
        path(vec![TermPathSegment::application_argument(0)]),
        vec![replacement(1, var(2), REPLACEMENT_ROLE_TERM_ARGUMENT)],
    )]);

    let report = replay_substitutions(input(&target, &certificate, Some(&context), limits()))
        .expect("valid direct substitution");

    assert_eq!(report.checked_substitutions().len(), 1);
    assert_eq!(report.checked_substitutions()[0].substitution_id, 1);
    assert_eq!(report.checked_substitutions()[0].source_term, source);
    assert_eq!(report.checked_substitutions()[0].target_term, target_term);
    assert_eq!(
        checked_substitutions_for_input(
            input(&target, &certificate, Some(&context), limits()),
            &report
        )
        .expect("report binding"),
        report.checked_substitutions()
    );
}

#[test]
fn accepts_formal_map_payload_and_term_or_predicate_argument_roles_only() {
    for role in [
        REPLACEMENT_ROLE_TERM_ARGUMENT,
        REPLACEMENT_ROLE_PREDICATE_ARGUMENT,
    ] {
        let target = target();
        let certificate = certificate(vec![substitution(1, var(1), var(2))]);
        let context = context(vec![payload(
            1,
            TermPath::root(),
            vec![replacement(1, var(2), role)],
        )]);

        replay_substitutions(input(&target, &certificate, Some(&context), limits()))
            .expect("accepted direct payload role");
    }
}

#[test]
fn rejects_missing_malformed_and_deferred_payload_evidence_without_diff_inference() {
    let target = target();
    let certificate = certificate(vec![substitution(1, var(1), var(2))]);
    let missing_payload = context(Vec::new());
    let record = replay_substitutions(input(
        &target,
        &certificate,
        Some(&missing_payload),
        limits(),
    ))
    .expect_err("missing payload");
    assert_rejection(
        &record,
        RejectionDetail::MissingProvenance,
        Some(1),
        Some("substitution_payload"),
    );
    assert_eq!(record.target_vc_fingerprint(), &target);

    let cases = [
        (
            payload(
                1,
                TermPath::root(),
                vec![replacement(1, var(2), REPLACEMENT_ROLE_TERM_ARGUMENT)],
            )
            .with_kind(99),
            "payload.payload_kind",
        ),
        (
            payload(1, TermPath::root(), vec![replacement(1, var(2), 99)]),
            "payload.replacement_role",
        ),
        (
            payload(
                1,
                TermPath::root(),
                vec![replacement(1, var(2), REPLACEMENT_ROLE_TERM_ARGUMENT)],
            )
            .with_kind(PAYLOAD_KIND_LOCAL_ABBREVIATION_EXPANSION),
            "payload.payload_kind",
        ),
        (
            payload(
                1,
                TermPath::root(),
                vec![replacement(
                    1,
                    var(2),
                    REPLACEMENT_ROLE_CAPTURED_FREE_VARIABLE,
                )],
            ),
            "payload.replacement_role",
        ),
        (
            payload(
                1,
                TermPath::root(),
                vec![replacement(1, var(2), REPLACEMENT_ROLE_TERM_ARGUMENT)],
            )
            .with_owner(99),
            "payload.owner_substitution_id",
        ),
        (
            payload(
                1,
                TermPath::root(),
                vec![
                    replacement(1, var(2), REPLACEMENT_ROLE_TERM_ARGUMENT),
                    replacement(1, var(3), REPLACEMENT_ROLE_TERM_ARGUMENT),
                ],
            ),
            "payload.replacements",
        ),
        (
            payload(
                1,
                TermPath::root(),
                vec![replacement(99, var(2), REPLACEMENT_ROLE_TERM_ARGUMENT)],
            ),
            "payload.formal_variable_id",
        ),
        (
            payload(
                1,
                TermPath::root(),
                vec![replacement(1, var(99), REPLACEMENT_ROLE_TERM_ARGUMENT)],
            ),
            "payload.actual_term",
        ),
    ];

    for (bad_payload, field_path) in cases {
        let context = context(vec![bad_payload]);
        let record = replay_substitutions(input(&target, &certificate, Some(&context), limits()))
            .expect_err("malformed or deferred payload");
        assert_rejection(
            &record,
            RejectionDetail::InvalidSubstitution,
            Some(1),
            Some(field_path),
        );
    }
}

#[test]
fn rejects_target_mismatch_manifest_errors_and_capture_without_alpha_repair() {
    let target = target();
    let mismatch = certificate(vec![substitution(1, var(1), var(2))]);
    let no_rewrite = context(vec![payload(1, TermPath::root(), Vec::new())]);
    let record = replay_substitutions(input(&target, &mismatch, Some(&no_rewrite), limits()))
        .expect_err("target mismatch");
    assert_rejection(
        &record,
        RejectionDetail::InvalidSubstitution,
        Some(1),
        Some("target_term"),
    );

    let bad_source = certificate(vec![substitution(1, var(99), var(99))]);
    let bad_source_context = context(vec![payload(1, TermPath::root(), Vec::new())]);
    let record = replay_substitutions(input(
        &target,
        &bad_source,
        Some(&bad_source_context),
        limits(),
    ))
    .expect_err("manifest-incompatible source");
    assert_rejection(
        &record,
        RejectionDetail::InvalidSubstitution,
        Some(1),
        Some("source_term"),
    );

    let captured = certificate_with_binder(
        substitution(1, binder(10, var(1)), binder(10, var(2))),
        binder_context(vec![(10, 0, 2, 1)], vec![1, 2, 3], Vec::new()),
    );
    let capture_context = context(vec![payload(
        1,
        TermPath::root(),
        vec![replacement(1, var(2), REPLACEMENT_ROLE_TERM_ARGUMENT)],
    )]);
    let record = replay_substitutions(input(&target, &captured, Some(&capture_context), limits()))
        .expect_err("capture is rejected without alpha repair");
    assert_rejection(
        &record,
        RejectionDetail::InvalidSubstitution,
        Some(1),
        Some("payload.actual_term"),
    );

    let forged_under_binder = certificate_with_binder(
        substitution(1, binder(10, var(1)), binder(10, var(3))),
        binder_context(vec![(10, 0, 2, 1)], vec![1, 2, 3], Vec::new()),
    );
    let non_capturing_context = context(vec![payload(
        1,
        TermPath::root(),
        vec![replacement(1, var(3), REPLACEMENT_ROLE_TERM_ARGUMENT)],
    )]);
    replay_substitutions(input(
        &target,
        &forged_under_binder,
        Some(&non_capturing_context),
        limits(),
    ))
    .expect("task 12 accepts non-capturing under-binder substitution");
}

#[test]
fn alpha_conversion_and_freshness_witnesses_are_replayed_semantically() {
    let target = target();
    let source = binder(10, pair(var(1), var(2)));
    let target_term = binder(11, pair(var(2), var(3)));
    let mut entry = substitution(1, source, target_term);
    entry.freshness_witness_refs = vec![1];
    let certificate = certificate_with_binder(
        entry,
        binder_context(
            vec![(10, 0, 2, 1), (11, 1, 3, 4)],
            all_variables(),
            Vec::new(),
        ),
    );
    let context = context_with_side(
        vec![payload(
            1,
            TermPath::root(),
            vec![replacement(1, var(2), REPLACEMENT_ROLE_TERM_ARGUMENT)],
        )],
        vec![freshness_full(
            1,
            1,
            3,
            TermPath::root(),
            vec![VariableId(1), VariableId(2)],
            0,
        )],
        Vec::new(),
    );

    replay_substitutions(input(&target, &certificate, Some(&context), limits()))
        .expect("freshness witness justifies alpha-renamed capture avoidance and bound rename");

    let alpha_only_certificate = certificate_with_binder(
        {
            let mut entry = substitution(
                1,
                binder(10, pair(var(1), var(2))),
                binder(11, pair(var(1), var(3))),
            );
            entry.freshness_witness_refs = vec![1];
            entry
        },
        binder_context(
            vec![(10, 0, 2, 1), (11, 1, 3, 4)],
            all_variables(),
            Vec::new(),
        ),
    );
    let alpha_only_context = context_with_side(
        vec![payload(1, TermPath::root(), Vec::new())],
        vec![freshness_full(
            1,
            1,
            3,
            TermPath::root(),
            vec![VariableId(1)],
            1,
        )],
        Vec::new(),
    );
    replay_substitutions(input(
        &target,
        &alpha_only_certificate,
        Some(&alpha_only_context),
        SubstitutionReplayLimits {
            max_avoided_variables: 1,
            ..limits()
        },
    ))
    .expect("source binder variable is excluded before final avoided limit");

    let bad_target = certificate_with_binder(
        {
            let mut entry = substitution(
                1,
                binder(10, pair(var(1), var(2))),
                binder(11, pair(var(2), var(2))),
            );
            entry.freshness_witness_refs = vec![1];
            entry
        },
        binder_context(
            vec![(10, 0, 2, 1), (11, 1, 3, 4)],
            all_variables(),
            Vec::new(),
        ),
    );
    let record = replay_substitutions(input(&target, &bad_target, Some(&context), limits()))
        .expect_err("inconsistent alpha target");
    assert_rejection(
        &record,
        RejectionDetail::InvalidSubstitution,
        Some(1),
        Some("target_term"),
    );

    let unreferenced_witness = certificate_with_binder(
        substitution(
            1,
            binder(10, pair(var(1), var(2))),
            binder(11, pair(var(2), var(3))),
        ),
        binder_context(
            vec![(10, 0, 2, 1), (11, 1, 3, 4)],
            all_variables(),
            Vec::new(),
        ),
    );
    let record = replay_substitutions(input(
        &target,
        &unreferenced_witness,
        Some(&context),
        limits(),
    ))
    .expect_err("unreferenced freshness witness must not be consulted");
    assert_rejection(
        &record,
        RejectionDetail::InvalidSubstitution,
        Some(1),
        Some("payload.actual_term"),
    );

    let bad_counter = context_with_side(
        vec![payload(
            1,
            TermPath::root(),
            vec![replacement(1, var(2), REPLACEMENT_ROLE_TERM_ARGUMENT)],
        )],
        vec![freshness_full(
            1,
            1,
            3,
            TermPath::root(),
            vec![VariableId(1), VariableId(2)],
            1,
        )],
        Vec::new(),
    );
    let record = replay_substitutions(input(&target, &certificate, Some(&bad_counter), limits()))
        .expect_err("freshness counter mismatch");
    assert_rejection(
        &record,
        RejectionDetail::InvalidSubstitution,
        Some(1),
        Some("freshness_witness.deterministic_counter"),
    );

    let stale_avoided = context_with_side(
        vec![payload(
            1,
            TermPath::root(),
            vec![replacement(1, var(2), REPLACEMENT_ROLE_TERM_ARGUMENT)],
        )],
        vec![freshness_full(
            1,
            1,
            3,
            TermPath::root(),
            vec![VariableId(1)],
            0,
        )],
        Vec::new(),
    );
    let record = replay_substitutions(input(&target, &certificate, Some(&stale_avoided), limits()))
        .expect_err("stale sorted avoided set mismatch");
    assert_rejection(
        &record,
        RejectionDetail::InvalidSubstitution,
        Some(1),
        Some("freshness_witness.avoided_variables"),
    );

    let record = replay_substitutions(input(
        &target,
        &certificate,
        Some(&context),
        SubstitutionReplayLimits {
            max_avoided_variables: 1,
            ..limits()
        },
    ))
    .expect_err("recomputed avoided set respects resource limit");
    assert_rejection(
        &record,
        RejectionDetail::ResourceExhaustion,
        Some(1),
        Some("freshness_witness.avoided_variables"),
    );

    let bad_role = certificate_with_binder(
        {
            let mut entry = substitution(
                1,
                binder(10, pair(var(1), var(2))),
                binder(11, pair(var(2), var(3))),
            );
            entry.freshness_witness_refs = vec![1];
            entry
        },
        binder_context(
            vec![(10, 0, 2, 1), (11, 1, 3, 1)],
            all_variables(),
            Vec::new(),
        ),
    );
    let record = replay_substitutions(input(&target, &bad_role, Some(&context), limits()))
        .expect_err("target binder must be generated fresh");
    assert_rejection(
        &record,
        RejectionDetail::InvalidSubstitution,
        Some(1),
        Some("freshness_witness.generated_variable_id"),
    );
}

#[test]
fn free_variable_constraints_recompute_capture_sets() {
    let target = target();
    let certificate = certificate_with_refs(Vec::new(), vec![1, 2]);
    let context = context_with_side(
        vec![payload(
            1,
            TermPath::root(),
            vec![replacement(1, var(2), REPLACEMENT_ROLE_TERM_ARGUMENT)],
        )],
        Vec::new(),
        vec![
            free_constraint(
                1,
                1,
                CONSTRAINT_KIND_MUST_REMAIN_FREE_AT_PATH,
                2,
                TermPath::root(),
                Vec::new(),
            ),
            free_constraint(
                2,
                1,
                CONSTRAINT_KIND_MUST_BE_ABSENT_FROM_CAPTURE_SET,
                3,
                TermPath::root(),
                Vec::new(),
            ),
        ],
    );
    replay_substitutions(input(&target, &certificate, Some(&context), limits()))
        .expect("free variable constraints are replayed");

    let binder_certificate = certificate_with_binder(
        {
            let mut entry = substitution(1, binder(10, var(2)), binder(10, var(2)));
            entry.free_variable_constraint_refs = vec![1];
            entry
        },
        binder_context(vec![(10, 0, 2, 1)], all_variables(), Vec::new()),
    );
    let binder_context_ok = context_with_side(
        vec![payload(1, TermPath::root(), Vec::new())],
        Vec::new(),
        vec![free_constraint(
            1,
            1,
            CONSTRAINT_KIND_MUST_BE_ABSENT_FROM_CAPTURE_SET,
            3,
            path(vec![TermPathSegment::binder_body()]),
            vec![VariableId(2)],
        )],
    );
    replay_substitutions(input(
        &target,
        &binder_certificate,
        Some(&binder_context_ok),
        limits(),
    ))
    .expect("capture set is recomputed from active target binders");

    let bad_capture_set = context_with_side(
        vec![payload(1, TermPath::root(), Vec::new())],
        Vec::new(),
        vec![free_constraint(
            1,
            1,
            CONSTRAINT_KIND_MUST_BE_ABSENT_FROM_CAPTURE_SET,
            3,
            path(vec![TermPathSegment::binder_body()]),
            Vec::new(),
        )],
    );
    let record = replay_substitutions(input(
        &target,
        &binder_certificate,
        Some(&bad_capture_set),
        limits(),
    ))
    .expect_err("recorded capture set must not be self-attesting");
    assert_rejection(
        &record,
        RejectionDetail::InvalidSubstitution,
        Some(1),
        Some("free_variable_constraint.capture_set"),
    );

    let record = replay_substitutions(input(
        &target,
        &binder_certificate,
        Some(&binder_context_ok),
        SubstitutionReplayLimits {
            max_capture_set_variables: 0,
            ..limits()
        },
    ))
    .expect_err("recomputed capture set respects resource limit");
    assert_rejection(
        &record,
        RejectionDetail::ResourceExhaustion,
        Some(1),
        Some("free_variable_constraint.capture_set"),
    );

    let captured_variable_constraint = context_with_side(
        vec![payload(1, TermPath::root(), Vec::new())],
        Vec::new(),
        vec![free_constraint(
            1,
            1,
            CONSTRAINT_KIND_MUST_REMAIN_FREE_AT_PATH,
            2,
            path(vec![TermPathSegment::binder_body()]),
            vec![VariableId(2)],
        )],
    );
    let record = replay_substitutions(input(
        &target,
        &binder_certificate,
        Some(&captured_variable_constraint),
        limits(),
    ))
    .expect_err("syntactic occurrence captured by target binder is not free");
    assert_rejection(
        &record,
        RejectionDetail::InvalidSubstitution,
        Some(1),
        Some("free_variable_constraint.variable_id"),
    );

    let missing_free_variable = context_with_side(
        vec![payload(
            1,
            TermPath::root(),
            vec![replacement(1, var(2), REPLACEMENT_ROLE_TERM_ARGUMENT)],
        )],
        Vec::new(),
        vec![free_constraint(
            1,
            1,
            CONSTRAINT_KIND_MUST_REMAIN_FREE_AT_PATH,
            4,
            TermPath::root(),
            Vec::new(),
        )],
    );
    let one_constraint = certificate_with_refs(Vec::new(), vec![1]);
    let record = replay_substitutions(input(
        &target,
        &one_constraint,
        Some(&missing_free_variable),
        limits(),
    ))
    .expect_err("missing free variable at target path");
    assert_rejection(
        &record,
        RejectionDetail::InvalidSubstitution,
        Some(1),
        Some("free_variable_constraint.variable_id"),
    );
}

#[test]
fn alpha_rename_limits_and_shuffled_witness_contexts_are_deterministic() {
    let target = target();
    let mut first = substitution(1, binder(10, var(1)), binder(11, var(2)));
    first.binder_context_encoding = binder_context(
        vec![(10, 0, 2, 1), (11, 1, 5, 4)],
        all_variables(),
        Vec::new(),
    );
    first.freshness_witness_refs = vec![1];
    let mut second = substitution(2, binder(20, var(3)), binder(21, var(4)));
    second.binder_context_encoding = binder_context(
        vec![(20, 0, 4, 1), (21, 1, 6, 4)],
        all_variables(),
        Vec::new(),
    );
    second.freshness_witness_refs = vec![2];
    let parsed_certificate = certificate(vec![first.clone(), second]);
    let context = context_with_side(
        vec![
            payload(
                2,
                TermPath::root(),
                vec![replacement(3, var(4), REPLACEMENT_ROLE_TERM_ARGUMENT)],
            ),
            payload(
                1,
                TermPath::root(),
                vec![replacement(1, var(2), REPLACEMENT_ROLE_TERM_ARGUMENT)],
            ),
        ],
        vec![
            freshness_full(
                2,
                2,
                6,
                TermPath::root(),
                vec![VariableId(3), VariableId(4)],
                3,
            ),
            freshness_full(
                1,
                1,
                5,
                TermPath::root(),
                vec![VariableId(1), VariableId(2)],
                2,
            ),
        ],
        Vec::new(),
    );
    let report = replay_substitutions(input(
        &target,
        &parsed_certificate,
        Some(&context),
        limits(),
    ))
    .expect("shuffled context is canonicalized before witness replay");
    assert_eq!(
        report
            .checked_substitutions()
            .iter()
            .map(|checked| checked.substitution_id)
            .collect::<Vec<_>>(),
        vec![1, 2]
    );

    let limited = replay_substitutions(input(
        &target,
        &certificate(vec![first]),
        Some(&context),
        SubstitutionReplayLimits {
            max_alpha_renames: 0,
            ..limits()
        },
    ))
    .expect_err("alpha rename budget");
    assert_rejection(
        &limited,
        RejectionDetail::ResourceExhaustion,
        Some(1),
        Some("freshness_witness_refs"),
    );

    let invalid_refs = certificate_with_refs(Vec::new(), vec![1, 2]);
    let shuffled_invalid_context = context_with_side(
        vec![payload(
            1,
            TermPath::root(),
            vec![replacement(1, var(2), REPLACEMENT_ROLE_TERM_ARGUMENT)],
        )],
        Vec::new(),
        vec![
            free_constraint(
                2,
                1,
                CONSTRAINT_KIND_MUST_BE_ABSENT_FROM_CAPTURE_SET,
                3,
                TermPath::root(),
                vec![VariableId(2)],
            ),
            free_constraint(
                1,
                1,
                CONSTRAINT_KIND_MUST_REMAIN_FREE_AT_PATH,
                4,
                TermPath::root(),
                Vec::new(),
            ),
        ],
    );
    let record = replay_substitutions(input(
        &target,
        &invalid_refs,
        Some(&shuffled_invalid_context),
        limits(),
    ))
    .expect_err("first rejected constraint follows certificate ref order");
    assert_rejection(
        &record,
        RejectionDetail::InvalidSubstitution,
        Some(1),
        Some("free_variable_constraint.variable_id"),
    );
}

#[test]
fn binder_context_decode_matrix_is_deterministic() {
    let target = target();
    let base = substitution(1, var(1), var(2));
    let invalid_cases = [
        (
            replace_schema_version(binder_context(Vec::new(), vec![1, 2, 3], Vec::new()), 2),
            "binder_context.schema_version",
        ),
        (
            binder_context(vec![(10, 0, 1, 9)], vec![1, 2, 3], Vec::new()),
            "binder_context.binder_role",
        ),
        (
            truncated_binder_context(),
            "binder_context.schematic_variables",
        ),
        (binder_context_with_frame_count(8), "binder_context.frames"),
        (
            binder_context_with_free_variable_count(8),
            "binder_context.free_variables",
        ),
        (
            binder_context_with_schematic_variable_count(8),
            "binder_context.schematic_variables",
        ),
        (
            {
                let mut bytes = binder_context(Vec::new(), vec![1, 2, 3], Vec::new());
                bytes.push(0);
                bytes
            },
            "binder_context.trailing_bytes",
        ),
        (
            binder_context(
                vec![(10, 0, 1, 1), (10, 1, 2, 1)],
                vec![1, 2, 3],
                Vec::new(),
            ),
            "binder_context.frames",
        ),
        (
            binder_context(
                vec![(10, 0, 1, 1), (11, 1, 1, 1)],
                vec![1, 2, 3],
                Vec::new(),
            ),
            "binder_context.frames",
        ),
        (
            binder_context(
                vec![(10, 2, 1, 1), (11, 1, 2, 1)],
                vec![1, 2, 3],
                Vec::new(),
            ),
            "binder_context.frames",
        ),
        (
            binder_context(Vec::new(), vec![2, 1], Vec::new()),
            "binder_context.free_variables",
        ),
        (
            binder_context(Vec::new(), vec![1, 2, 3], vec![3, 2]),
            "binder_context.schematic_variables",
        ),
        (
            binder_context(vec![(10, 0, 99, 1)], vec![1, 2, 3], Vec::new()),
            "binder_context.frames",
        ),
        (
            binder_context(Vec::new(), vec![1, 99], Vec::new()),
            "binder_context.free_variables",
        ),
        (
            binder_context(Vec::new(), vec![1, 2, 3], vec![99]),
            "binder_context.schematic_variables",
        ),
        (
            binder_context(vec![(10, 1, 1, 1)], vec![1, 2, 3], Vec::new()),
            "binder_context.frames",
        ),
    ];

    for (bytes, field_path) in invalid_cases {
        let certificate = certificate_with_binder(base.clone(), bytes);
        let context = context(vec![payload(
            1,
            TermPath::root(),
            vec![replacement(1, var(2), REPLACEMENT_ROLE_TERM_ARGUMENT)],
        )]);
        let record = replay_substitutions(input(&target, &certificate, Some(&context), limits()))
            .expect_err("invalid binder context");
        assert_rejection(
            &record,
            RejectionDetail::InvalidSubstitution,
            Some(1),
            Some(field_path),
        );
    }

    let incompatible = certificate_with_binder(
        substitution(1, binder(99, var(1)), binder(99, var(2))),
        binder_context(Vec::new(), vec![1, 2, 3], Vec::new()),
    );
    let incompatible_context = context(vec![payload(
        1,
        TermPath::root(),
        vec![replacement(1, var(2), REPLACEMENT_ROLE_TERM_ARGUMENT)],
    )]);
    let record = replay_substitutions(input(
        &target,
        &incompatible,
        Some(&incompatible_context),
        limits(),
    ))
    .expect_err("frame/term incompatibility");
    assert_rejection(
        &record,
        RejectionDetail::InvalidSubstitution,
        Some(1),
        Some("source_term"),
    );

    let traversal_mismatch = certificate_with_binder(
        substitution(
            1,
            pair(binder(10, var(1)), binder(11, var(2))),
            pair(binder(10, var(1)), binder(11, var(2))),
        ),
        binder_context(
            vec![(11, 0, 2, 1), (10, 1, 1, 1)],
            vec![1, 2, 3],
            Vec::new(),
        ),
    );
    let identity_context = context(vec![payload(1, TermPath::root(), Vec::new())]);
    let record = replay_substitutions(input(
        &target,
        &traversal_mismatch,
        Some(&identity_context),
        limits(),
    ))
    .expect_err("term traversal order must match frame canonical indices");
    assert_rejection(
        &record,
        RejectionDetail::InvalidSubstitution,
        Some(1),
        Some("source_term"),
    );

    let unused_frame = certificate_with_binder(
        base.clone(),
        binder_context(vec![(10, 0, 1, 1)], vec![1, 2, 3], Vec::new()),
    );
    let record = replay_substitutions(input(
        &target,
        &unused_frame,
        Some(&identity_context),
        limits(),
    ))
    .expect_err("frame set must exactly match used normalized binders");
    assert_rejection(
        &record,
        RejectionDetail::InvalidSubstitution,
        Some(1),
        Some("binder_context"),
    );

    let resource_certificate = certificate_with_binder(
        base,
        binder_context(vec![(10, 0, 1, 1)], vec![1, 2, 3], Vec::new()),
    );
    let context = context(vec![payload(
        1,
        TermPath::root(),
        vec![replacement(1, var(2), REPLACEMENT_ROLE_TERM_ARGUMENT)],
    )]);
    let record = replay_substitutions(input(
        &target,
        &resource_certificate,
        Some(&context),
        SubstitutionReplayLimits {
            max_binder_frames: 0,
            ..limits()
        },
    ))
    .expect_err("frame limit");
    assert_rejection(
        &record,
        RejectionDetail::ResourceExhaustion,
        Some(1),
        Some("binder_context.frames"),
    );

    let record = replay_substitutions(input(
        &target,
        &resource_certificate,
        Some(&context),
        SubstitutionReplayLimits {
            max_binder_context_bytes: 1,
            ..limits()
        },
    ))
    .expect_err("byte limit");
    assert_rejection(
        &record,
        RejectionDetail::ResourceExhaustion,
        Some(1),
        Some("binder_context"),
    );
}

#[test]
fn missing_provenance_and_side_condition_shape_are_rejected_at_first_use() {
    let target = target();
    let certificate = certificate_with_refs(vec![1], vec![2]);
    let no_context = replay_substitutions(input(&target, &certificate, None, limits()))
        .expect_err("missing context");
    assert_rejection(
        &no_context,
        RejectionDetail::MissingProvenance,
        Some(1),
        Some("substitution_context"),
    );

    let empty_provenance = SubstitutionContext::new(
        Some(Vec::new()),
        vec![payload(
            1,
            TermPath::root(),
            vec![replacement(1, var(2), REPLACEMENT_ROLE_TERM_ARGUMENT)],
        )],
        Vec::new(),
        Vec::new(),
    )
    .expect("shape-valid context");
    let record = replay_substitutions(input(
        &target,
        &certificate,
        Some(&empty_provenance),
        limits(),
    ))
    .expect_err("missing provenance");
    assert_rejection(
        &record,
        RejectionDetail::MissingProvenance,
        Some(1),
        Some("substitution_context.provenance"),
    );

    let context = context(vec![payload(
        1,
        TermPath::root(),
        vec![replacement(1, var(2), REPLACEMENT_ROLE_TERM_ARGUMENT)],
    )]);
    let record = replay_substitutions(input(&target, &certificate, Some(&context), limits()))
        .expect_err("missing witness");
    assert_rejection(
        &record,
        RejectionDetail::MissingProvenance,
        Some(1),
        Some("freshness_witness_refs"),
    );

    let only_constraint_missing = certificate_with_refs(Vec::new(), vec![2]);
    let record = replay_substitutions(input(
        &target,
        &only_constraint_missing,
        Some(&context),
        limits(),
    ))
    .expect_err("missing free-variable constraint");
    assert_rejection(
        &record,
        RejectionDetail::MissingProvenance,
        Some(1),
        Some("free_variable_constraint_refs"),
    );

    let payloads = || {
        vec![payload(
            1,
            TermPath::root(),
            vec![replacement(1, var(2), REPLACEMENT_ROLE_TERM_ARGUMENT)],
        )]
    };
    let side_cases = [
        (
            context_with_side(
                payloads(),
                vec![freshness(1).with_owner(99)],
                vec![constraint(2)],
            ),
            "freshness_witness.owner_substitution_id",
        ),
        (
            context_with_side(
                payloads(),
                vec![freshness_with_path(
                    1,
                    path(vec![TermPathSegment::application_argument(0)]),
                    vec![VariableId(1), VariableId(2)],
                )],
                vec![constraint(2)],
            ),
            "freshness_witness.binder_path",
        ),
        (
            context_with_side(
                payloads(),
                vec![freshness_with_path(
                    1,
                    path(vec![TermPathSegment::new(9, 0)]),
                    vec![VariableId(1), VariableId(2)],
                )],
                vec![constraint(2)],
            ),
            "freshness_witness.binder_path",
        ),
        (
            context_with_side(
                payloads(),
                vec![freshness_with_path(
                    1,
                    TermPath::root(),
                    vec![VariableId(2), VariableId(1)],
                )],
                vec![constraint(2)],
            ),
            "freshness_witness.avoided_variables",
        ),
        (
            context_with_side(
                payloads(),
                vec![freshness(1)],
                vec![constraint(2).with_owner(99)],
            ),
            "free_variable_constraint.owner_substitution_id",
        ),
        (
            context_with_side(
                payloads(),
                vec![freshness(1)],
                vec![constraint(2).with_kind(9)],
            ),
            "free_variable_constraint.constraint_kind",
        ),
        (
            context_with_side(
                payloads(),
                vec![freshness(1)],
                vec![constraint_with_path(
                    2,
                    path(vec![TermPathSegment::application_argument(0)]),
                    vec![VariableId(2), VariableId(3)],
                )],
            ),
            "free_variable_constraint.term_path",
        ),
        (
            context_with_side(
                payloads(),
                vec![freshness(1)],
                vec![constraint_with_path(
                    2,
                    path(vec![TermPathSegment::new(9, 0)]),
                    vec![VariableId(2), VariableId(3)],
                )],
            ),
            "free_variable_constraint.term_path",
        ),
        (
            context_with_side(
                payloads(),
                vec![freshness(1)],
                vec![constraint_with_path(
                    2,
                    TermPath::root(),
                    vec![VariableId(3), VariableId(2)],
                )],
            ),
            "free_variable_constraint.capture_set",
        ),
    ];

    for (bad_context, field_path) in side_cases {
        let record =
            replay_substitutions(input(&target, &certificate, Some(&bad_context), limits()))
                .expect_err("malformed side condition");
        assert_rejection(
            &record,
            RejectionDetail::InvalidSubstitution,
            Some(1),
            Some(field_path),
        );
    }
}

#[test]
fn resource_limits_fire_before_unbounded_payload_or_side_condition_work() {
    let target = target();
    let count_certificate = certificate(vec![substitution(1, var(1), var(2))]);
    let count_context = context(vec![payload(
        1,
        TermPath::root(),
        vec![replacement(1, var(2), REPLACEMENT_ROLE_TERM_ARGUMENT)],
    )]);
    let record = replay_substitutions(input(
        &target,
        &count_certificate,
        Some(&count_context),
        SubstitutionReplayLimits {
            max_substitutions: 0,
            ..limits()
        },
    ))
    .expect_err("substitution count limit");
    assert_rejection(&record, RejectionDetail::ResourceExhaustion, Some(1), None);

    let record = replay_substitutions(input(
        &target,
        &count_certificate,
        Some(&count_context),
        SubstitutionReplayLimits {
            max_term_encoding_bytes: 1,
            ..limits()
        },
    ))
    .expect_err("term byte limit");
    assert_rejection(
        &record,
        RejectionDetail::ResourceExhaustion,
        Some(1),
        Some("source_term"),
    );

    let actual_size_context = context(vec![payload(
        1,
        TermPath::root(),
        vec![replacement(
            1,
            pair(var(2), var(2)),
            REPLACEMENT_ROLE_TERM_ARGUMENT,
        )],
    )]);
    let record = replay_substitutions(input(
        &target,
        &count_certificate,
        Some(&actual_size_context),
        SubstitutionReplayLimits {
            max_term_encoding_bytes: 20,
            ..limits()
        },
    ))
    .expect_err("actual term byte limit");
    assert_rejection(
        &record,
        RejectionDetail::ResourceExhaustion,
        Some(1),
        Some("payload.actual_term"),
    );

    let expansion_certificate = certificate(vec![substitution(1, pair(var(1), var(1)), var(2))]);
    let expansion_context = context(vec![payload(
        1,
        TermPath::root(),
        vec![replacement(
            1,
            pair(var(2), var(2)),
            REPLACEMENT_ROLE_TERM_ARGUMENT,
        )],
    )]);
    let record = replay_substitutions(input(
        &target,
        &expansion_certificate,
        Some(&expansion_context),
        SubstitutionReplayLimits {
            max_term_encoding_bytes: 40,
            ..limits()
        },
    ))
    .expect_err("replayed target byte limit is checked before cloning result");
    assert_rejection(
        &record,
        RejectionDetail::ResourceExhaustion,
        Some(1),
        Some("target_term"),
    );

    let depth_expansion_certificate = certificate(vec![substitution(
        1,
        pair(var(1), var(2)),
        pair(var(2), var(2)),
    )]);
    let depth_expansion_context = context(vec![payload(
        1,
        path(vec![TermPathSegment::application_argument(0)]),
        vec![replacement(
            1,
            pair(var(2), var(2)),
            REPLACEMENT_ROLE_TERM_ARGUMENT,
        )],
    )]);
    let record = replay_substitutions(input(
        &target,
        &depth_expansion_certificate,
        Some(&depth_expansion_context),
        SubstitutionReplayLimits {
            max_term_recursion_depth: 1,
            ..limits()
        },
    ))
    .expect_err("replayed target depth limit is checked before cloning result");
    assert_rejection(
        &record,
        RejectionDetail::ResourceExhaustion,
        Some(1),
        Some("target_term"),
    );

    let too_many_replacements = context(vec![payload(
        1,
        TermPath::root(),
        vec![replacement(1, var(2), REPLACEMENT_ROLE_TERM_ARGUMENT)],
    )]);
    let record = replay_substitutions(input(
        &target,
        &count_certificate,
        Some(&too_many_replacements),
        SubstitutionReplayLimits {
            max_payload_replacements: 0,
            ..limits()
        },
    ))
    .expect_err("payload replacement count limit");
    assert_rejection(
        &record,
        RejectionDetail::ResourceExhaustion,
        Some(1),
        Some("payload.replacements"),
    );

    let actual_context = context(vec![payload(
        1,
        TermPath::root(),
        vec![replacement(1, deep_term(3), REPLACEMENT_ROLE_TERM_ARGUMENT)],
    )]);
    let record = replay_substitutions(input(
        &target,
        &count_certificate,
        Some(&actual_context),
        SubstitutionReplayLimits {
            max_term_recursion_depth: 1,
            ..limits()
        },
    ))
    .expect_err("actual term depth limit");
    assert_rejection(
        &record,
        RejectionDetail::ResourceExhaustion,
        Some(1),
        Some("payload.actual_term"),
    );

    let context = context(vec![payload(
        1,
        path(vec![TermPathSegment::application_argument(0)]),
        vec![replacement(1, var(2), REPLACEMENT_ROLE_TERM_ARGUMENT)],
    )]);
    let path_certificate = certificate(vec![substitution(
        1,
        pair(var(1), var(1)),
        pair(var(2), var(1)),
    )]);
    let record = replay_substitutions(input(
        &target,
        &path_certificate,
        Some(&context),
        SubstitutionReplayLimits {
            max_term_path_segments: 0,
            ..limits()
        },
    ))
    .expect_err("path segment limit");
    assert_rejection(
        &record,
        RejectionDetail::ResourceExhaustion,
        Some(1),
        Some("payload.rewrite_path"),
    );

    let refs_certificate = certificate_with_refs(vec![1, 2], Vec::new());
    let refs_context = context_with_side(
        vec![payload(
            1,
            TermPath::root(),
            vec![replacement(1, var(2), REPLACEMENT_ROLE_TERM_ARGUMENT)],
        )],
        vec![freshness(1), freshness(2)],
        Vec::new(),
    );
    let record = replay_substitutions(input(
        &target,
        &refs_certificate,
        Some(&refs_context),
        SubstitutionReplayLimits {
            max_freshness_witnesses: 1,
            ..limits()
        },
    ))
    .expect_err("freshness ref count limit");
    assert_rejection(
        &record,
        RejectionDetail::ResourceExhaustion,
        Some(1),
        Some("freshness_witness_refs"),
    );

    let refs_certificate = certificate_with_refs(Vec::new(), vec![1, 2]);
    let refs_context = context_with_side(
        vec![payload(
            1,
            TermPath::root(),
            vec![replacement(1, var(2), REPLACEMENT_ROLE_TERM_ARGUMENT)],
        )],
        Vec::new(),
        vec![constraint(1), constraint(2)],
    );
    let record = replay_substitutions(input(
        &target,
        &refs_certificate,
        Some(&refs_context),
        SubstitutionReplayLimits {
            max_free_variable_constraints: 1,
            ..limits()
        },
    ))
    .expect_err("free-variable ref count limit");
    assert_rejection(
        &record,
        RejectionDetail::ResourceExhaustion,
        Some(1),
        Some("free_variable_constraint_refs"),
    );

    let refs_certificate = certificate_with_refs(vec![1], vec![2]);
    let refs_context = context_with_side(
        vec![payload(
            1,
            TermPath::root(),
            vec![replacement(1, var(2), REPLACEMENT_ROLE_TERM_ARGUMENT)],
        )],
        vec![freshness(1)],
        vec![constraint(2)],
    );
    let record = replay_substitutions(input(
        &target,
        &refs_certificate,
        Some(&refs_context),
        SubstitutionReplayLimits {
            max_avoided_variables: 1,
            ..limits()
        },
    ))
    .expect_err("avoided variable limit");
    assert_rejection(
        &record,
        RejectionDetail::ResourceExhaustion,
        Some(1),
        Some("freshness_witness.avoided_variables"),
    );
    let record = replay_substitutions(input(
        &target,
        &refs_certificate,
        Some(&refs_context),
        SubstitutionReplayLimits {
            max_capture_set_variables: 1,
            ..limits()
        },
    ))
    .expect_err("capture set limit");
    assert_rejection(
        &record,
        RejectionDetail::ResourceExhaustion,
        Some(1),
        Some("free_variable_constraint.capture_set"),
    );
}

#[test]
fn context_constructor_canonicalizes_and_first_use_ignores_unused_malformed_entries() {
    let target = target();
    let certificate = certificate(vec![substitution(1, var(1), var(2))]);
    let context = context_with_side(
        vec![
            payload(
                99,
                TermPath::root(),
                vec![replacement(
                    1,
                    var(99),
                    REPLACEMENT_ROLE_CAPTURED_FREE_VARIABLE,
                )],
            )
            .with_kind(PAYLOAD_KIND_LOCAL_ABBREVIATION_EXPANSION),
            payload(
                1,
                TermPath::root(),
                vec![replacement(1, var(2), REPLACEMENT_ROLE_TERM_ARGUMENT)],
            ),
        ],
        vec![freshness(99).with_owner(99)],
        vec![constraint(99).with_kind(9)],
    );

    replay_substitutions(input(&target, &certificate, Some(&context), limits()))
        .expect("unused malformed context entries are ignored");

    let side_order_certificate = certificate_with_refs(Vec::new(), vec![2, 3]);
    let side_order_context = context_with_side(
        vec![payload(
            1,
            TermPath::root(),
            vec![replacement(1, var(2), REPLACEMENT_ROLE_TERM_ARGUMENT)],
        )],
        Vec::new(),
        vec![
            free_constraint(
                3,
                1,
                CONSTRAINT_KIND_MUST_BE_ABSENT_FROM_CAPTURE_SET,
                3,
                TermPath::root(),
                Vec::new(),
            ),
            free_constraint(
                2,
                1,
                CONSTRAINT_KIND_MUST_REMAIN_FREE_AT_PATH,
                2,
                TermPath::root(),
                Vec::new(),
            ),
        ],
    );
    replay_substitutions(input(
        &target,
        &side_order_certificate,
        Some(&side_order_context),
        limits(),
    ))
    .expect("constructor canonicalizes referenced constraint input order");

    let duplicate_payload = SubstitutionContext::new(
        Some(vec![7]),
        vec![
            payload(1, TermPath::root(), Vec::new()),
            payload(1, TermPath::root(), Vec::new()),
        ],
        Vec::new(),
        Vec::new(),
    )
    .expect_err("duplicate payload id");
    assert_eq!(
        duplicate_payload,
        SubstitutionContextError::DuplicateSubstitutionPayload { substitution_id: 1 }
    );

    let duplicate_witness = SubstitutionContext::new(
        Some(vec![7]),
        Vec::new(),
        vec![freshness(1), freshness(1)],
        Vec::new(),
    )
    .expect_err("duplicate witness id");
    assert_eq!(
        duplicate_witness,
        SubstitutionContextError::DuplicateFreshnessWitness { witness_id: 1 }
    );

    let duplicate_constraint = SubstitutionContext::new(
        Some(vec![7]),
        Vec::new(),
        Vec::new(),
        vec![constraint(1), constraint(1)],
    )
    .expect_err("duplicate constraint id");
    assert_eq!(
        duplicate_constraint,
        SubstitutionContextError::DuplicateFreeVariableConstraint { constraint_id: 1 }
    );

    let ambiguous_payload_context = unchecked_context(
        vec![
            payload(
                1,
                TermPath::root(),
                vec![replacement(1, var(2), REPLACEMENT_ROLE_TERM_ARGUMENT)],
            ),
            payload(
                99,
                TermPath::root(),
                vec![replacement(1, var(3), REPLACEMENT_ROLE_TERM_ARGUMENT)],
            ),
            payload(1, TermPath::root(), Vec::new()),
        ],
        Vec::new(),
        Vec::new(),
    );
    let record = replay_substitutions(input(
        &target,
        &certificate,
        Some(&ambiguous_payload_context),
        limits(),
    ))
    .expect_err("ambiguous payload id maps to missing provenance");
    assert_rejection(
        &record,
        RejectionDetail::MissingProvenance,
        Some(1),
        Some("substitution_payload"),
    );

    let refs_certificate = certificate_with_refs(vec![1], vec![2]);
    let ambiguous_witness_context = unchecked_context(
        vec![payload(
            1,
            TermPath::root(),
            vec![replacement(1, var(2), REPLACEMENT_ROLE_TERM_ARGUMENT)],
        )],
        vec![freshness(1), freshness(99), freshness(1)],
        vec![constraint(2)],
    );
    let record = replay_substitutions(input(
        &target,
        &refs_certificate,
        Some(&ambiguous_witness_context),
        limits(),
    ))
    .expect_err("ambiguous witness id maps to missing provenance");
    assert_rejection(
        &record,
        RejectionDetail::MissingProvenance,
        Some(1),
        Some("freshness_witness_refs"),
    );

    let ambiguous_constraint_context = unchecked_context(
        vec![payload(
            1,
            TermPath::root(),
            vec![replacement(1, var(2), REPLACEMENT_ROLE_TERM_ARGUMENT)],
        )],
        vec![freshness(1)],
        vec![constraint(2), constraint(99), constraint(2)],
    );
    let record = replay_substitutions(input(
        &target,
        &refs_certificate,
        Some(&ambiguous_constraint_context),
        limits(),
    ))
    .expect_err("ambiguous constraint id maps to missing provenance");
    assert_rejection(
        &record,
        RejectionDetail::MissingProvenance,
        Some(1),
        Some("free_variable_constraint_refs"),
    );
}

#[test]
fn report_binding_rejects_target_certificate_and_context_mismatches() {
    let target = target();
    let certificate = certificate(vec![substitution(1, var(1), var(2))]);
    let context = context(vec![payload(
        1,
        TermPath::root(),
        vec![replacement(1, var(2), REPLACEMENT_ROLE_TERM_ARGUMENT)],
    )]);
    let report = replay_substitutions(input(&target, &certificate, Some(&context), limits()))
        .expect("valid report");

    let other_target = TargetVcFingerprint::new(1, vec![43]);
    let record = checked_substitutions_for_input(
        input(&other_target, &certificate, Some(&context), limits()),
        &report,
    )
    .expect_err("target mismatch");
    assert_eq!(record.target_vc_fingerprint(), &other_target);
    assert_rejection(
        &record,
        RejectionDetail::InvalidSubstitution,
        None,
        Some("substitution_report_binding"),
    );

    let other_certificate = certificate_with_hash(vec![substitution(1, var(1), var(2))], vec![99]);
    let record = checked_substitutions_for_input(
        input(&target, &other_certificate, Some(&context), limits()),
        &report,
    )
    .expect_err("certificate hash mismatch");
    assert_rejection(
        &record,
        RejectionDetail::InvalidSubstitution,
        None,
        Some("substitution_report_binding"),
    );

    let other_context = SubstitutionContext::new(
        Some(vec![8]),
        vec![payload(
            1,
            TermPath::root(),
            vec![replacement(1, var(2), REPLACEMENT_ROLE_TERM_ARGUMENT)],
        )],
        Vec::new(),
        Vec::new(),
    )
    .expect("valid context");
    let record = checked_substitutions_for_input(
        input(&target, &certificate, Some(&other_context), limits()),
        &report,
    )
    .expect_err("context provenance mismatch");
    assert_rejection(
        &record,
        RejectionDetail::InvalidSubstitution,
        None,
        Some("substitution_report_binding"),
    );
}

trait PayloadEntryTestExt {
    fn with_kind(self, payload_kind: u8) -> Self;
    fn with_owner(self, owner_substitution_id: u32) -> Self;
}

impl PayloadEntryTestExt for SubstitutionPayloadEntry {
    fn with_kind(mut self, payload_kind: u8) -> Self {
        self.payload.payload_kind = payload_kind;
        self
    }

    fn with_owner(mut self, owner_substitution_id: u32) -> Self {
        self.payload.owner_substitution_id = owner_substitution_id;
        self
    }
}

trait FreshnessTestExt {
    fn with_owner(self, owner_substitution_id: u32) -> Self;
}

impl FreshnessTestExt for FreshnessWitness {
    fn with_owner(mut self, owner_substitution_id: u32) -> Self {
        self.owner_substitution_id = owner_substitution_id;
        self
    }
}

trait ConstraintTestExt {
    fn with_kind(self, constraint_kind: u8) -> Self;
    fn with_owner(self, owner_substitution_id: u32) -> Self;
}

impl ConstraintTestExt for FreeVariableConstraint {
    fn with_kind(mut self, constraint_kind: u8) -> Self {
        self.constraint_kind = constraint_kind;
        self
    }

    fn with_owner(mut self, owner_substitution_id: u32) -> Self {
        self.owner_substitution_id = owner_substitution_id;
        self
    }
}

fn input<'a>(
    target: &'a TargetVcFingerprint,
    certificate: &'a ParsedCertificate,
    substitution_context: Option<&'a SubstitutionContext>,
    limits: SubstitutionReplayLimits,
) -> SubstitutionCheckInput<'a> {
    SubstitutionCheckInput {
        target_vc_fingerprint: target,
        certificate,
        substitution_context,
        limits,
    }
}

fn assert_rejection(
    record: &RejectionRecord,
    detail: RejectionDetail,
    substitution_id: Option<u32>,
    field_path: Option<&'static str>,
) {
    assert_eq!(record.category(), RejectionCategory::KernelRejection);
    assert_eq!(record.detail(), detail);
    assert_eq!(record.detail().stable_key(), detail.stable_key());
    assert_eq!(record.location().substitution_id, substitution_id);
    if let Some(field_path) = field_path {
        assert_eq!(record.location().field_path, Some(field_path));
    }
}

fn target() -> TargetVcFingerprint {
    TargetVcFingerprint::new(1, vec![42])
}

fn limits() -> SubstitutionReplayLimits {
    SubstitutionReplayLimits {
        max_substitutions: 8,
        max_binder_context_bytes: 512,
        max_binder_frames: 8,
        max_freshness_witnesses: 4,
        max_free_variable_constraints: 4,
        max_term_encoding_bytes: 4096,
        max_term_recursion_depth: 16,
        max_alpha_renames: 4,
        max_payload_replacements: 8,
        max_term_path_segments: 8,
        max_avoided_variables: 8,
        max_capture_set_variables: 8,
    }
}

fn certificate(substitutions: Vec<SubstitutionEntry>) -> ParsedCertificate {
    certificate_with_hash(substitutions, vec![1, 2, 3])
}

fn certificate_with_hash(
    substitutions: Vec<SubstitutionEntry>,
    canonical_hash_input: Vec<u8>,
) -> ParsedCertificate {
    ParsedCertificate::new_for_kernel_tests(ParsedCertificateTestParts {
        schema_version: 1,
        encoding_version: 1,
        kernel_profile: KernelProfileRecord::v1(1, ClauseTautologyPolicy::Reject),
        target_vc: Fingerprint::new(1, vec![42]),
        symbol_manifest: vec![SymbolManifestEntry { symbol: symbol() }],
        variable_manifest: (1..=8)
            .map(|id| VariableManifestEntry {
                variable_id: VariableId(id),
            })
            .collect(),
        imported_axioms: Vec::new(),
        imported_theorems: Vec::new(),
        generated_clauses: Vec::new(),
        substitutions,
        resolution_trace: Vec::new(),
        derived_facts: Vec::new(),
        final_goal: FinalGoalRef {
            namespace: FinalGoalNamespace::GeneratedClause,
            id: 0,
        },
        canonical_hash_input,
    })
}

fn certificate_with_binder(
    mut substitution: SubstitutionEntry,
    binder_context_encoding: Vec<u8>,
) -> ParsedCertificate {
    substitution.binder_context_encoding = binder_context_encoding;
    certificate(vec![substitution])
}

fn certificate_with_refs(
    freshness_witness_refs: Vec<u32>,
    free_variable_constraint_refs: Vec<u32>,
) -> ParsedCertificate {
    let mut substitution = substitution(1, var(1), var(2));
    substitution.freshness_witness_refs = freshness_witness_refs;
    substitution.free_variable_constraint_refs = free_variable_constraint_refs;
    certificate(vec![substitution])
}

fn substitution(substitution_id: u32, source_term: Term, target_term: Term) -> SubstitutionEntry {
    SubstitutionEntry {
        substitution_id,
        source_term,
        target_term,
        binder_context_encoding: binder_context(Vec::new(), vec![1, 2, 3, 4], Vec::new()),
        freshness_witness_refs: Vec::new(),
        free_variable_constraint_refs: Vec::new(),
    }
}

fn context(payloads: Vec<SubstitutionPayloadEntry>) -> SubstitutionContext {
    context_with_side(payloads, Vec::new(), Vec::new())
}

fn context_with_side(
    payloads: Vec<SubstitutionPayloadEntry>,
    freshness_witnesses: Vec<FreshnessWitness>,
    free_variable_constraints: Vec<FreeVariableConstraint>,
) -> SubstitutionContext {
    SubstitutionContext::new(
        Some(vec![7]),
        payloads,
        freshness_witnesses,
        free_variable_constraints,
    )
    .expect("valid context shape")
}

fn unchecked_context(
    payloads: Vec<SubstitutionPayloadEntry>,
    freshness_witnesses: Vec<FreshnessWitness>,
    free_variable_constraints: Vec<FreeVariableConstraint>,
) -> SubstitutionContext {
    SubstitutionContext {
        provenance_fingerprint: Some(vec![7]),
        substitution_payloads: payloads,
        freshness_witnesses,
        free_variable_constraints,
        canonical_shape: false,
    }
}

fn payload(
    substitution_id: u32,
    rewrite_path: TermPath,
    replacements: Vec<Replacement>,
) -> SubstitutionPayloadEntry {
    SubstitutionPayloadEntry::new(
        substitution_id,
        SubstitutionPayload::new(
            substitution_id,
            PAYLOAD_KIND_FORMAL_TO_ACTUAL_MAP,
            rewrite_path,
            replacements,
        ),
    )
}

fn replacement(formal_variable_id: u32, actual_term: Term, replacement_role: u8) -> Replacement {
    Replacement::new(
        VariableId(formal_variable_id),
        actual_term,
        replacement_role,
    )
}

fn freshness(witness_id: u32) -> FreshnessWitness {
    freshness_with_path(
        witness_id,
        TermPath::root(),
        vec![VariableId(1), VariableId(2)],
    )
}

fn freshness_with_path(
    witness_id: u32,
    binder_path: TermPath,
    avoided_variables: Vec<VariableId>,
) -> FreshnessWitness {
    freshness_full(witness_id, 1, 3, binder_path, avoided_variables, 0)
}

fn freshness_full(
    witness_id: u32,
    owner_substitution_id: u32,
    generated_variable_id: u32,
    binder_path: TermPath,
    avoided_variables: Vec<VariableId>,
    deterministic_counter: u32,
) -> FreshnessWitness {
    FreshnessWitness::new(
        witness_id,
        owner_substitution_id,
        VariableId(generated_variable_id),
        binder_path,
        avoided_variables,
        deterministic_counter,
    )
}

fn constraint(constraint_id: u32) -> FreeVariableConstraint {
    constraint_with_path(
        constraint_id,
        TermPath::root(),
        vec![VariableId(2), VariableId(3)],
    )
}

fn constraint_with_path(
    constraint_id: u32,
    term_path: TermPath,
    capture_set: Vec<VariableId>,
) -> FreeVariableConstraint {
    free_constraint(
        constraint_id,
        1,
        CONSTRAINT_KIND_MUST_REMAIN_FREE_AT_PATH,
        1,
        term_path,
        capture_set,
    )
}

fn free_constraint(
    constraint_id: u32,
    owner_substitution_id: u32,
    constraint_kind: u8,
    variable_id: u32,
    term_path: TermPath,
    capture_set: Vec<VariableId>,
) -> FreeVariableConstraint {
    FreeVariableConstraint::new(
        constraint_id,
        owner_substitution_id,
        constraint_kind,
        VariableId(variable_id),
        term_path,
        capture_set,
    )
}

fn path(segments: Vec<TermPathSegment>) -> TermPath {
    TermPath::new(segments)
}

fn var(id: u32) -> Term {
    Term::Variable(VariableId(id))
}

fn pair(left: Term, right: Term) -> Term {
    Term::Application {
        symbol: symbol(),
        arguments: vec![left, right],
    }
}

fn deep_term(depth: u32) -> Term {
    if depth == 0 {
        return var(1);
    }
    pair(deep_term(depth - 1), var(1))
}

fn binder(binder_id: u32, body: Term) -> Term {
    Term::BinderNormalized {
        binder_id,
        body: Box::new(body),
    }
}

const fn symbol() -> SymbolKey {
    SymbolKey::new(SymbolKind::Predicate, 1)
}

fn all_variables() -> Vec<u32> {
    (1..=8).collect()
}

fn binder_context(
    frames: Vec<(u32, u32, u32, u8)>,
    free_variables: Vec<u32>,
    schematic_variables: Vec<u32>,
) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&BINDER_CONTEXT_SCHEMA_VERSION.to_be_bytes());
    bytes.extend_from_slice(&(frames.len() as u32).to_be_bytes());
    for (binder_id, canonical_index, variable_id, binder_role) in frames {
        bytes.extend_from_slice(&binder_id.to_be_bytes());
        bytes.extend_from_slice(&canonical_index.to_be_bytes());
        bytes.extend_from_slice(&variable_id.to_be_bytes());
        bytes.push(binder_role);
    }
    bytes.extend_from_slice(&(free_variables.len() as u32).to_be_bytes());
    for variable in free_variables {
        bytes.extend_from_slice(&variable.to_be_bytes());
    }
    bytes.extend_from_slice(&(schematic_variables.len() as u32).to_be_bytes());
    for variable in schematic_variables {
        bytes.extend_from_slice(&variable.to_be_bytes());
    }
    bytes
}

fn binder_context_with_frame_count(frame_count: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&BINDER_CONTEXT_SCHEMA_VERSION.to_be_bytes());
    bytes.extend_from_slice(&frame_count.to_be_bytes());
    bytes
}

fn binder_context_with_free_variable_count(free_variable_count: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&BINDER_CONTEXT_SCHEMA_VERSION.to_be_bytes());
    bytes.extend_from_slice(&0u32.to_be_bytes());
    bytes.extend_from_slice(&free_variable_count.to_be_bytes());
    bytes
}

fn binder_context_with_schematic_variable_count(schematic_variable_count: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&BINDER_CONTEXT_SCHEMA_VERSION.to_be_bytes());
    bytes.extend_from_slice(&0u32.to_be_bytes());
    bytes.extend_from_slice(&0u32.to_be_bytes());
    bytes.extend_from_slice(&schematic_variable_count.to_be_bytes());
    bytes
}

fn replace_schema_version(mut bytes: Vec<u8>, schema_version: u16) -> Vec<u8> {
    bytes[0..2].copy_from_slice(&schema_version.to_be_bytes());
    bytes
}

fn truncated_binder_context() -> Vec<u8> {
    let mut bytes = binder_context(Vec::new(), vec![1, 2, 3], Vec::new());
    bytes.pop();
    bytes
}
