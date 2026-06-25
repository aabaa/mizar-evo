use super::*;

const DIRECTORY_ENTRY_LEN: usize = 13;

#[test]
fn parses_minimal_valid_certificate_and_hash_input() {
    let (bytes, context) = fixture();

    let parsed = parse_certificate(&bytes, &context).expect("valid certificate");

    assert_eq!(parsed.schema_version, 1);
    assert_eq!(parsed.encoding_version, 1);
    assert_eq!(parsed.generated_clauses.len(), 1);
    assert_eq!(parsed.final_goal.id, 1);
    assert_eq!(parsed.canonical_hash_input(), bytes.as_slice());
    assert!(parsed.canonical_hash_input().starts_with(DOMAIN_SEPARATOR));
    assert!(contains_subsequence(
        parsed.canonical_hash_input(),
        &1_u16.to_be_bytes()
    ));
    assert!(contains_subsequence(
        parsed.canonical_hash_input(),
        &context
            .expected_target_vc
            .canonical_bytes()
            .expect("fingerprint bytes")
    ));
}

#[test]
fn rejects_unsupported_header_profile_and_context_mismatch() {
    let (bytes, context) = fixture();

    let mut bad_schema = bytes.clone();
    bad_schema[DOMAIN_SEPARATOR.len() + 1] = 2;
    assert_detail(
        parse_certificate(&bad_schema, &context),
        CertificateRejectionDetail::UnsupportedCertificateFormat,
    );

    let mut bad_encoding = bytes.clone();
    bad_encoding[DOMAIN_SEPARATOR.len() + 3] = 2;
    assert_detail(
        parse_certificate(&bad_encoding, &context),
        CertificateRejectionDetail::UnsupportedCertificateFormat,
    );

    let mut bad_profile = bytes.clone();
    bad_profile[DOMAIN_SEPARATOR.len() + 5] = 2;
    assert_detail(
        parse_certificate(&bad_profile, &context),
        CertificateRejectionDetail::UnsupportedCertificateFormat,
    );

    let mut bad_hash_algorithm = bytes.clone();
    let hash_algorithm_offset = DOMAIN_SEPARATOR.len() + 4 + 7;
    bad_hash_algorithm[hash_algorithm_offset] = 9;
    assert_detail(
        parse_certificate(&bad_hash_algorithm, &context),
        CertificateRejectionDetail::UnsupportedCertificateFormat,
    );

    let mismatched_context =
        CertificateParseContext::v1(Fingerprint::new(1, vec![9]), sample_profile());
    assert_detail(
        parse_certificate(&bytes, &mismatched_context),
        CertificateRejectionDetail::ContextMismatch,
    );
}

#[test]
fn rejects_directory_and_item_canonicality_errors_with_locations() {
    let (bytes, context) = fixture();
    let directory_start = directory_start(&context.expected_target_vc);

    let mut unknown_section = bytes.clone();
    unknown_section[directory_start] = 0xff;
    assert_rejection_location(
        parse_certificate(&unknown_section, &context),
        CertificateRejectionDetail::UnsupportedCertificateFormat,
        directory_start,
        None,
        None,
        "directory.section_tag",
    );

    let mut out_of_order = bytes.clone();
    out_of_order[directory_start] = SectionTag::VariableManifest.byte();
    assert_rejection_location(
        parse_certificate(&out_of_order, &context),
        CertificateRejectionDetail::MalformedCertificate,
        directory_start,
        None,
        None,
        "directory.section_tag",
    );

    let mut missing = bytes.clone();
    set_u32(&mut missing, directory_start - 4, 8);
    assert_rejection_location(
        parse_certificate(&missing, &context),
        CertificateRejectionDetail::MalformedCertificate,
        directory_start - 4,
        None,
        None,
        "directory_entry_count",
    );

    let mut duplicate = bytes.clone();
    duplicate[directory_start + DIRECTORY_ENTRY_LEN] = SectionTag::SymbolManifest.byte();
    assert_rejection_location(
        parse_certificate(&duplicate, &context),
        CertificateRejectionDetail::MalformedCertificate,
        directory_start + DIRECTORY_ENTRY_LEN,
        None,
        None,
        "directory.section_tag",
    );

    let mut non_contiguous = bytes.clone();
    let variable_entry =
        directory_entry_start(&context.expected_target_vc, SectionTag::VariableManifest);
    set_u32(
        &mut non_contiguous,
        variable_entry + 5,
        section_payload_offset(
            &bytes,
            &context.expected_target_vc,
            SectionTag::VariableManifest,
        ) + 1,
    );
    assert_rejection_location(
        parse_certificate(&non_contiguous, &context),
        CertificateRejectionDetail::MalformedCertificate,
        variable_entry + 5,
        None,
        None,
        "directory.payload_offset",
    );

    let mut overlapping = bytes.clone();
    set_u32(&mut overlapping, variable_entry + 5, 0);
    assert_rejection_location(
        parse_certificate(&overlapping, &context),
        CertificateRejectionDetail::MalformedCertificate,
        variable_entry + 5,
        None,
        None,
        "directory.payload_offset",
    );

    let mut item_tag = bytes.clone();
    let payload_start = directory_start + REQUIRED_SECTIONS.len() * DIRECTORY_ENTRY_LEN;
    item_tag[payload_start + 1] = 2;
    assert_rejection_location(
        parse_certificate(&item_tag, &context),
        CertificateRejectionDetail::MalformedCertificate,
        payload_start + 1,
        Some(SectionTag::SymbolManifest),
        Some(0),
        "item.item_tag",
    );

    let mut section_tag = bytes.clone();
    section_tag[payload_start] = SectionTag::VariableManifest.byte();
    assert_rejection_location(
        parse_certificate(&section_tag, &context),
        CertificateRejectionDetail::MalformedCertificate,
        payload_start,
        Some(SectionTag::SymbolManifest),
        Some(0),
        "item.section_tag",
    );

    let mut item_count = bytes.clone();
    set_u32(&mut item_count, variable_entry + 1, 2);
    let variable_payload_end = payload_start
        + section_payload_offset(
            &bytes,
            &context.expected_target_vc,
            SectionTag::VariableManifest,
        ) as usize
        + section_payload_length(
            &bytes,
            &context.expected_target_vc,
            SectionTag::VariableManifest,
        ) as usize;
    assert_rejection_location(
        parse_certificate(&item_count, &context),
        CertificateRejectionDetail::MalformedCertificate,
        variable_payload_end,
        Some(SectionTag::VariableManifest),
        Some(1),
        "item.section_tag",
    );

    let mut malformed_field = CertificateBuilder::minimal();
    malformed_field.variable_manifest = vec![{
        let mut bytes = variable_manifest_entry(1);
        bytes.push(0);
        bytes
    }];
    assert_rejection_location(
        parse_certificate(&malformed_field.finish(), &context),
        CertificateRejectionDetail::MalformedCertificate,
        variable_payload_end,
        Some(SectionTag::VariableManifest),
        Some(0),
        "trailing_bytes",
    );

    let mut range_out_of_bounds = bytes.clone();
    set_u32(&mut range_out_of_bounds, variable_entry + 9, u32::MAX);
    let unbounded_section_context = context.clone().with_limits(CertificateParseLimits {
        max_section_bytes: usize::MAX,
        ..CertificateParseLimits::default()
    });
    assert_rejection_location(
        parse_certificate(&range_out_of_bounds, &unbounded_section_context),
        CertificateRejectionDetail::MalformedCertificate,
        variable_entry + 9,
        None,
        None,
        "directory.payload_length",
    );

    let mut truncated = bytes.clone();
    truncated.pop();
    assert_rejection_location(
        parse_certificate(&truncated, &context),
        CertificateRejectionDetail::MalformedCertificate,
        directory_entry_start(&context.expected_target_vc, SectionTag::FinalGoal) + 9,
        None,
        None,
        "directory.payload_length",
    );

    let mut trailing = bytes.clone();
    trailing.push(0);
    assert_rejection_location(
        parse_certificate(&trailing, &context),
        CertificateRejectionDetail::MalformedCertificate,
        bytes.len(),
        None,
        None,
        "section_payloads.trailing_bytes",
    );
}

#[test]
fn rejects_resource_exhaustion_before_large_allocation() {
    let (bytes, context) = fixture();
    let limits = CertificateParseLimits {
        max_certificate_bytes: bytes.len() - 1,
        ..CertificateParseLimits::default()
    };
    assert_detail(
        parse_certificate(&bytes, &context.clone().with_limits(limits)),
        CertificateRejectionDetail::ResourceExhaustion,
    );

    let limits = CertificateParseLimits {
        max_section_bytes: 0,
        ..CertificateParseLimits::default()
    };
    assert_detail(
        parse_certificate(&bytes, &context.clone().with_limits(limits)),
        CertificateRejectionDetail::ResourceExhaustion,
    );

    let limits = CertificateParseLimits {
        max_symbol_manifest_entries: 1,
        ..CertificateParseLimits::default()
    };
    assert_detail(
        parse_certificate(&bytes, &context.clone().with_limits(limits)),
        CertificateRejectionDetail::ResourceExhaustion,
    );

    let mut huge_section = bytes.clone();
    set_u32(
        &mut huge_section,
        directory_entry_start(&context.expected_target_vc, SectionTag::SymbolManifest) + 9,
        u32::MAX,
    );
    let limits = CertificateParseLimits {
        max_section_bytes: 32,
        ..CertificateParseLimits::default()
    };
    assert_detail(
        parse_certificate(&huge_section, &context.clone().with_limits(limits)),
        CertificateRejectionDetail::ResourceExhaustion,
    );

    let deep = certificate_with_substitution_term_depth(3);
    let limits = CertificateParseLimits {
        max_term_recursion_depth: 1,
        ..CertificateParseLimits::default()
    };
    assert_detail(
        parse_certificate(&deep, &context.clone().with_limits(limits)),
        CertificateRejectionDetail::ResourceExhaustion,
    );

    let mut small_atom_arguments = CertificateBuilder::minimal();
    small_atom_arguments.generated_clauses = vec![generated_clause_with_literals(
        1,
        1,
        vec![literal_record(
            2,
            1,
            1,
            vec![variable_term(1), variable_term(1)],
        )],
    )];
    let small_term_context =
        context
            .clone()
            .with_clause_validation_policy(ClauseValidationPolicy {
                max_literals: 8,
                max_term_encoding_bytes: 16,
            });
    assert!(parse_certificate(&small_atom_arguments.finish(), &small_term_context).is_ok());

    let mut term_budget = CertificateBuilder::minimal();
    term_budget.generated_clauses = vec![generated_clause_with_literals(
        1,
        1,
        vec![literal_record(
            2,
            1,
            1,
            vec![application_term(
                1,
                1,
                vec![
                    nested_term(1),
                    nested_term(1),
                    nested_term(1),
                    nested_term(1),
                ],
            )],
        )],
    )];
    let parent_budget_context =
        context
            .clone()
            .with_clause_validation_policy(ClauseValidationPolicy {
                max_literals: 8,
                max_term_encoding_bytes: 35,
            });
    assert_detail(
        parse_certificate(&term_budget.finish(), &parent_budget_context),
        CertificateRejectionDetail::ResourceExhaustion,
    );
}

#[test]
fn rejects_imported_fact_reference_errors() {
    let mut builder = CertificateBuilder::minimal();
    builder.imported_axioms = vec![imported_fact(1, b"pkg", b"mod", b"item")];
    let (bytes, context) = (builder.finish(), sample_context());
    let parsed = parse_certificate(&bytes, &context).expect("valid imported fact");
    assert_eq!(parsed.imported_axioms.len(), 1);

    let mut theorem_builder = CertificateBuilder::minimal();
    theorem_builder.imported_theorems = vec![imported_fact(1, b"pkg", b"mod", b"thm")];
    assert_eq!(
        parse_certificate(&theorem_builder.finish(), &context)
            .expect("valid theorem")
            .imported_theorems
            .len(),
        1
    );

    let mut unsorted_id = CertificateBuilder::minimal();
    unsorted_id.imported_axioms = vec![
        imported_fact(2, b"pkg", b"mod", b"item2"),
        imported_fact(1, b"pkg", b"mod", b"item1"),
    ];
    assert_detail(
        parse_certificate(&unsorted_id.finish(), &context),
        CertificateRejectionDetail::MalformedCertificate,
    );

    let mut duplicate_id = CertificateBuilder::minimal();
    duplicate_id.imported_axioms = vec![
        imported_fact(1, b"pkg", b"mod", b"item1"),
        imported_fact(1, b"pkg", b"mod", b"item2"),
    ];
    assert_detail(
        parse_certificate(&duplicate_id.finish(), &context),
        CertificateRejectionDetail::MalformedCertificate,
    );

    let mut duplicate_key = CertificateBuilder::minimal();
    duplicate_key.imported_axioms = vec![
        imported_fact(1, b"pkg", b"mod", b"item"),
        imported_fact(2, b"pkg", b"mod", b"item"),
    ];
    assert_detail(
        parse_certificate(&duplicate_key.finish(), &context),
        CertificateRejectionDetail::MalformedCertificate,
    );

    let malformed = imported_fact(1, b"", b"mod", b"item");
    let mut builder = CertificateBuilder::minimal();
    builder.imported_axioms = vec![malformed];
    assert_detail(
        parse_certificate(&builder.finish(), &context),
        CertificateRejectionDetail::MalformedCertificate,
    );

    for malformed in [
        imported_fact(1, b"pkg", b"", b"item"),
        imported_fact(1, b"pkg", b"mod", b""),
        imported_fact_with_fingerprint(1, b"pkg", b"mod", b"item", Vec::new(), 1),
        imported_fact_with_fingerprint(1, b"pkg", b"mod", b"item", vec![7], 9),
    ] {
        let mut builder = CertificateBuilder::minimal();
        builder.imported_axioms = vec![malformed];
        assert_detail(
            parse_certificate(&builder.finish(), &context),
            CertificateRejectionDetail::MalformedCertificate,
        );
    }
}

#[test]
fn rejects_manifest_and_generated_clause_errors() {
    let (bytes, context) = fixture();
    assert!(parse_certificate(&bytes, &context).is_ok());

    let mut missing_variable = CertificateBuilder::minimal();
    missing_variable.variable_manifest.clear();
    assert_detail(
        parse_certificate(&missing_variable.finish(), &context),
        CertificateRejectionDetail::MalformedCertificate,
    );

    let mut duplicate_symbol = CertificateBuilder::minimal();
    duplicate_symbol
        .symbol_manifest
        .push(symbol_manifest_entry(1, 1));
    assert_detail(
        parse_certificate(&duplicate_symbol.finish(), &context),
        CertificateRejectionDetail::MalformedCertificate,
    );

    let mut unsorted_symbol = CertificateBuilder::minimal();
    unsorted_symbol.symbol_manifest =
        vec![symbol_manifest_entry(1, 2), symbol_manifest_entry(1, 1)];
    assert_detail(
        parse_certificate(&unsorted_symbol.finish(), &context),
        CertificateRejectionDetail::MalformedCertificate,
    );

    let mut duplicate_variable = CertificateBuilder::minimal();
    duplicate_variable
        .variable_manifest
        .push(variable_manifest_entry(1));
    assert_detail(
        parse_certificate(&duplicate_variable.finish(), &context),
        CertificateRejectionDetail::MalformedCertificate,
    );

    let mut unsorted_variable = CertificateBuilder::minimal();
    unsorted_variable.variable_manifest =
        vec![variable_manifest_entry(2), variable_manifest_entry(1)];
    assert_detail(
        parse_certificate(&unsorted_variable.finish(), &context),
        CertificateRejectionDetail::MalformedCertificate,
    );

    let mut unsupported_kind = CertificateBuilder::minimal();
    unsupported_kind.symbol_manifest = vec![symbol_manifest_entry(9, 1)];
    assert_detail(
        parse_certificate(&unsupported_kind.finish(), &context),
        CertificateRejectionDetail::MalformedCertificate,
    );

    let mut missing_symbol = CertificateBuilder::minimal();
    missing_symbol.symbol_manifest = vec![symbol_manifest_entry(1, 2)];
    assert_detail(
        parse_certificate(&missing_symbol.finish(), &context),
        CertificateRejectionDetail::MalformedCertificate,
    );

    let mut duplicate_clause = CertificateBuilder::minimal();
    duplicate_clause
        .generated_clauses
        .push(generated_clause_with_literals(
            1,
            1,
            vec![literal_record(2, 1, 2, vec![])],
        ));
    assert_detail(
        parse_certificate(&duplicate_clause.finish(), &context),
        CertificateRejectionDetail::MalformedCertificate,
    );

    let mut unsorted_clause = CertificateBuilder::minimal();
    unsorted_clause.generated_clauses = vec![
        generated_clause_with_literals(2, 1, vec![literal_record(2, 1, 1, vec![variable_term(1)])]),
        generated_clause_with_literals(1, 1, vec![literal_record(2, 1, 2, vec![])]),
    ];
    assert_detail(
        parse_certificate(&unsorted_clause.finish(), &context),
        CertificateRejectionDetail::MalformedCertificate,
    );

    let mut noncanonical = CertificateBuilder::minimal();
    noncanonical.generated_clauses = vec![generated_clause_with_literals(
        1,
        1,
        vec![
            literal_record(2, 1, 2, vec![]),
            literal_record(2, 1, 1, vec![]),
        ],
    )];
    assert_detail(
        parse_certificate(&noncanonical.finish(), &context),
        CertificateRejectionDetail::MalformedCertificate,
    );
}

#[test]
fn validates_substitution_resolution_derived_and_final_refs() {
    let context = sample_context();

    let mut valid = CertificateBuilder::minimal();
    valid.imported_axioms = vec![imported_fact(3, b"pkg", b"mod", b"item")];
    valid.resolution_trace = vec![resolution_step(1, clause_ref(3, 3), clause_ref(1, 1), 1)];
    valid.derived_facts = vec![derived_fact(1, clause_ref(2, 1))];
    valid.final_goal = final_goal(3, 1);
    valid.substitutions = vec![substitution_entry(1, vec![1, 3])];
    assert!(parse_certificate(&valid.finish(), &context).is_ok());

    let mut unsorted_refs = CertificateBuilder::minimal();
    unsorted_refs.substitutions = vec![substitution_entry(1, vec![3, 1])];
    assert_detail(
        parse_certificate(&unsorted_refs.finish(), &context),
        CertificateRejectionDetail::MalformedCertificate,
    );

    let mut duplicate_substitution = CertificateBuilder::minimal();
    duplicate_substitution.substitutions = vec![
        substitution_entry(1, vec![1]),
        substitution_entry(1, vec![2]),
    ];
    assert_detail(
        parse_certificate(&duplicate_substitution.finish(), &context),
        CertificateRejectionDetail::MalformedCertificate,
    );

    let mut unsorted_substitution = CertificateBuilder::minimal();
    unsorted_substitution.substitutions = vec![
        substitution_entry(2, vec![1]),
        substitution_entry(1, vec![2]),
    ];
    assert_detail(
        parse_certificate(&unsorted_substitution.finish(), &context),
        CertificateRejectionDetail::MalformedCertificate,
    );

    let mut duplicate_free_variable_refs = CertificateBuilder::minimal();
    duplicate_free_variable_refs.substitutions =
        vec![substitution_entry_with_refs(1, vec![1], vec![2, 2])];
    assert_detail(
        parse_certificate(&duplicate_free_variable_refs.finish(), &context),
        CertificateRejectionDetail::MalformedCertificate,
    );

    let mut forward = CertificateBuilder::minimal();
    forward.resolution_trace = vec![resolution_step(1, clause_ref(2, 2), clause_ref(1, 1), 1)];
    assert_detail(
        parse_certificate(&forward.finish(), &context),
        CertificateRejectionDetail::MalformedCertificate,
    );

    let mut duplicate_step = CertificateBuilder::minimal();
    duplicate_step.resolution_trace = vec![
        resolution_step(1, clause_ref(1, 1), clause_ref(1, 1), 1),
        resolution_step(1, clause_ref(1, 1), clause_ref(1, 1), 1),
    ];
    assert_detail(
        parse_certificate(&duplicate_step.finish(), &context),
        CertificateRejectionDetail::MalformedCertificate,
    );

    let mut unsorted_step = CertificateBuilder::minimal();
    unsorted_step.resolution_trace = vec![
        resolution_step(2, clause_ref(1, 1), clause_ref(1, 1), 1),
        resolution_step(1, clause_ref(1, 1), clause_ref(1, 1), 1),
    ];
    assert_detail(
        parse_certificate(&unsorted_step.finish(), &context),
        CertificateRejectionDetail::MalformedCertificate,
    );

    let mut self_parent = CertificateBuilder::minimal();
    self_parent.resolution_trace = vec![resolution_step(1, clause_ref(2, 1), clause_ref(1, 1), 1)];
    assert_detail(
        parse_certificate(&self_parent.finish(), &context),
        CertificateRejectionDetail::MalformedCertificate,
    );

    let mut bad_parent_namespace = CertificateBuilder::minimal();
    bad_parent_namespace.resolution_trace =
        vec![resolution_step(1, clause_ref(9, 1), clause_ref(1, 1), 1)];
    assert_detail(
        parse_certificate(&bad_parent_namespace.finish(), &context),
        CertificateRejectionDetail::MalformedCertificate,
    );

    let mut bad_pivot = CertificateBuilder::minimal();
    bad_pivot.resolution_trace = vec![resolution_step_with_pivot(
        1,
        clause_ref(1, 1),
        clause_ref(1, 1),
        vec![9],
        1,
    )];
    assert_detail(
        parse_certificate(&bad_pivot.finish(), &context),
        CertificateRejectionDetail::MalformedCertificate,
    );

    let mut deep_pivot = CertificateBuilder::minimal();
    deep_pivot.resolution_trace = vec![resolution_step_with_pivot(
        1,
        clause_ref(1, 1),
        clause_ref(1, 1),
        literal_record(2, 1, 1, vec![nested_term(3)]),
        1,
    )];
    let limits = CertificateParseLimits {
        max_term_recursion_depth: 1,
        ..CertificateParseLimits::default()
    };
    assert_detail(
        parse_certificate(&deep_pivot.finish(), &context.clone().with_limits(limits)),
        CertificateRejectionDetail::ResourceExhaustion,
    );

    let mut bad_generated = CertificateBuilder::minimal();
    bad_generated.resolution_trace =
        vec![resolution_step(1, clause_ref(1, 1), clause_ref(1, 1), 99)];
    assert_detail(
        parse_certificate(&bad_generated.finish(), &context),
        CertificateRejectionDetail::MalformedCertificate,
    );

    let mut wrong_generated_namespace = CertificateBuilder::minimal();
    wrong_generated_namespace.resolution_trace = vec![resolution_step_with_generated_ref(
        1,
        clause_ref(1, 1),
        clause_ref(1, 1),
        clause_ref(2, 1),
    )];
    assert_detail(
        parse_certificate(&wrong_generated_namespace.finish(), &context),
        CertificateRejectionDetail::MalformedCertificate,
    );

    let mut bad_derived = CertificateBuilder::minimal();
    bad_derived.derived_facts = vec![derived_fact(1, clause_ref(2, 99))];
    assert_detail(
        parse_certificate(&bad_derived.finish(), &context),
        CertificateRejectionDetail::MalformedCertificate,
    );

    let mut final_generated = CertificateBuilder::minimal();
    final_generated.final_goal = final_goal(1, 1);
    assert!(parse_certificate(&final_generated.finish(), &context).is_ok());

    let mut final_resolution = CertificateBuilder::minimal();
    final_resolution.resolution_trace =
        vec![resolution_step(1, clause_ref(1, 1), clause_ref(1, 1), 1)];
    final_resolution.final_goal = final_goal(2, 1);
    assert!(parse_certificate(&final_resolution.finish(), &context).is_ok());

    let mut malformed_final_namespace = CertificateBuilder::minimal();
    malformed_final_namespace.final_goal = final_goal(9, 1);
    assert_detail(
        parse_certificate(&malformed_final_namespace.finish(), &context),
        CertificateRejectionDetail::MalformedCertificate,
    );

    let mut bad_final = CertificateBuilder::minimal();
    bad_final.final_goal = final_goal(3, 99);
    assert_detail(
        parse_certificate(&bad_final.finish(), &context),
        CertificateRejectionDetail::MalformedCertificate,
    );
}

#[test]
fn preserves_deterministic_order_for_all_parsed_collections() {
    let context = sample_context();
    let mut builder = CertificateBuilder::minimal();
    builder.imported_axioms = vec![
        imported_fact(1, b"pkg", b"mod", b"axiom1"),
        imported_fact(2, b"pkg", b"mod", b"axiom2"),
    ];
    builder.imported_theorems = vec![imported_fact(1, b"pkg", b"mod", b"thm1")];
    builder
        .generated_clauses
        .push(generated_clause_with_literals(
            2,
            1,
            vec![
                literal_record(2, 1, 1, vec![variable_term(1)]),
                literal_record(2, 1, 2, vec![]),
            ],
        ));
    builder.substitutions = vec![
        substitution_entry(1, vec![1]),
        substitution_entry(2, vec![2]),
    ];
    builder.resolution_trace = vec![resolution_step(1, clause_ref(3, 1), clause_ref(1, 1), 1)];
    builder.derived_facts = vec![
        derived_fact(1, clause_ref(1, 1)),
        derived_fact(2, clause_ref(2, 1)),
    ];

    let parsed = parse_certificate(&builder.finish(), &context).expect("ordered certificate");

    assert_eq!(
        parsed
            .symbol_manifest
            .iter()
            .map(|entry| entry.symbol.id.0)
            .collect::<Vec<_>>(),
        [1, 2]
    );
    assert_eq!(
        parsed.generated_clauses[1]
            .clause
            .literals()
            .iter()
            .map(|literal| literal.atom.symbol.id.0)
            .collect::<Vec<_>>(),
        [1, 2]
    );

    let mut shuffled_child_bytes = CertificateBuilder::minimal();
    shuffled_child_bytes.generated_clauses = vec![generated_clause_with_literals(
        1,
        1,
        vec![
            literal_record(2, 1, 2, vec![]),
            literal_record(2, 1, 1, vec![variable_term(1)]),
        ],
    )];
    assert_detail(
        parse_certificate(&shuffled_child_bytes.finish(), &context),
        CertificateRejectionDetail::MalformedCertificate,
    );
    assert_eq!(
        parsed
            .variable_manifest
            .iter()
            .map(|entry| entry.variable_id.0)
            .collect::<Vec<_>>(),
        [1]
    );
    assert_eq!(
        parsed
            .imported_axioms
            .iter()
            .map(|fact| fact.imported_fact_id)
            .collect::<Vec<_>>(),
        [1, 2]
    );
    assert_eq!(
        parsed
            .imported_theorems
            .iter()
            .map(|fact| fact.imported_fact_id)
            .collect::<Vec<_>>(),
        [1]
    );
    assert_eq!(
        parsed
            .generated_clauses
            .iter()
            .map(|clause| clause.clause_id)
            .collect::<Vec<_>>(),
        [1, 2]
    );
    assert_eq!(
        parsed
            .substitutions
            .iter()
            .map(|entry| entry.substitution_id)
            .collect::<Vec<_>>(),
        [1, 2]
    );
    assert_eq!(
        parsed
            .resolution_trace
            .iter()
            .map(|step| step.step_id)
            .collect::<Vec<_>>(),
        [1]
    );
    assert_eq!(
        parsed
            .derived_facts
            .iter()
            .map(|fact| fact.derived_fact_id)
            .collect::<Vec<_>>(),
        [1, 2]
    );
}

#[test]
fn deterministic_hash_input_excludes_nondeterministic_data() {
    let (bytes, context) = fixture();
    let parsed_a = parse_certificate(&bytes, &context).expect("valid a");
    let parsed_b = parse_certificate(&bytes, &context).expect("valid b");
    assert_eq!(parsed_a, parsed_b);
    assert_eq!(
        parsed_a.canonical_hash_input(),
        parsed_b.canonical_hash_input()
    );
    assert!(contains_subsequence(
        parsed_a.canonical_hash_input(),
        &1_u16.to_be_bytes()
    ));
    assert!(contains_subsequence(
        parsed_a.canonical_hash_input(),
        &profile_bytes(sample_profile())
    ));
    assert!(contains_subsequence(
        parsed_a.canonical_hash_input(),
        &item_frame(
            SectionTag::GeneratedClauses,
            generated_clause_with_literals(
                1,
                1,
                vec![literal_record(2, 1, 1, vec![variable_term(1)])],
            )
        )
    ));
    let directory_start = directory_start(&context.expected_target_vc);
    let payload_start = directory_start + REQUIRED_SECTIONS.len() * DIRECTORY_ENTRY_LEN;
    assert!(contains_subsequence(
        parsed_a.canonical_hash_input(),
        &bytes[directory_start..payload_start]
    ));
    for excluded in [
        "source-path",
        "source-range",
        "display-name",
        "backend-log",
        "timestamp",
        "elapsed-time",
        "allocation-address",
        "allocation-order",
        "worker-completion-order",
        "cache-key",
        "artifact-path",
        "policy-projection",
    ] {
        assert!(!contains_subsequence(
            parsed_a.canonical_hash_input(),
            excluded.as_bytes()
        ));
    }
}

#[test]
fn parser_errors_are_certificate_rejections_and_never_timeout() {
    let (mut bytes, context) = fixture();
    bytes[0] = 0;
    let error = parse_certificate(&bytes, &context).expect_err("invalid domain");
    assert_eq!(error.category, FailureCategory::CertificateRejection);
    assert_ne!(format!("{:?}", error.detail), "Timeout");
}

fn fixture() -> (Vec<u8>, CertificateParseContext) {
    (CertificateBuilder::minimal().finish(), sample_context())
}

fn sample_context() -> CertificateParseContext {
    CertificateParseContext::v1(sample_target(), sample_profile()).with_clause_validation_policy(
        ClauseValidationPolicy {
            max_literals: 8,
            max_term_encoding_bytes: 256,
        },
    )
}

fn sample_profile() -> KernelProfileRecord {
    KernelProfileRecord::v1(1, ClauseTautologyPolicy::Reject)
}

fn sample_target() -> Fingerprint {
    Fingerprint::new(1, vec![1, 2, 3])
}

fn directory_start(target: &Fingerprint) -> usize {
    DOMAIN_SEPARATOR.len()
        + 2
        + 2
        + PROFILE_LEN
        + target.canonical_bytes().expect("target bytes").len()
        + 4
}

fn certificate_with_substitution_term_depth(depth: u32) -> Vec<u8> {
    let mut builder = CertificateBuilder::minimal();
    builder.substitutions = vec![substitution_entry_with_term(1, nested_term(depth))];
    builder.finish()
}

#[derive(Clone)]
struct CertificateBuilder {
    symbol_manifest: Vec<Vec<u8>>,
    variable_manifest: Vec<Vec<u8>>,
    imported_axioms: Vec<Vec<u8>>,
    imported_theorems: Vec<Vec<u8>>,
    generated_clauses: Vec<Vec<u8>>,
    substitutions: Vec<Vec<u8>>,
    resolution_trace: Vec<Vec<u8>>,
    derived_facts: Vec<Vec<u8>>,
    final_goal: Vec<u8>,
}

impl CertificateBuilder {
    fn minimal() -> Self {
        Self {
            symbol_manifest: vec![symbol_manifest_entry(1, 1), symbol_manifest_entry(1, 2)],
            variable_manifest: vec![variable_manifest_entry(1)],
            imported_axioms: Vec::new(),
            imported_theorems: Vec::new(),
            generated_clauses: vec![generated_clause_with_literals(
                1,
                1,
                vec![literal_record(2, 1, 1, vec![variable_term(1)])],
            )],
            substitutions: Vec::new(),
            resolution_trace: Vec::new(),
            derived_facts: Vec::new(),
            final_goal: final_goal(1, 1),
        }
    }

    fn finish(self) -> Vec<u8> {
        let sections = [
            (SectionTag::SymbolManifest, self.symbol_manifest),
            (SectionTag::VariableManifest, self.variable_manifest),
            (SectionTag::ImportedAxioms, self.imported_axioms),
            (SectionTag::ImportedTheorems, self.imported_theorems),
            (SectionTag::GeneratedClauses, self.generated_clauses),
            (SectionTag::Substitutions, self.substitutions),
            (SectionTag::ResolutionTrace, self.resolution_trace),
            (SectionTag::DerivedFacts, self.derived_facts),
            (SectionTag::FinalGoal, vec![self.final_goal]),
        ];
        let mut payloads = Vec::new();
        let mut directory = Vec::new();
        let mut offset = 0u32;
        for (section, items) in sections {
            let item_count = items.len() as u32;
            let section_payload = items
                .into_iter()
                .flat_map(|payload| item_frame(section, payload))
                .collect::<Vec<_>>();
            directory.push((
                section,
                item_count,
                offset,
                items_len(section_payload.len()),
            ));
            offset += items_len(section_payload.len());
            payloads.extend(section_payload);
        }

        let mut bytes = Vec::from(DOMAIN_SEPARATOR);
        bytes.extend_from_slice(&1_u16.to_be_bytes());
        bytes.extend_from_slice(&1_u16.to_be_bytes());
        bytes.extend(profile_bytes(sample_profile()));
        bytes.extend(fingerprint_bytes(&sample_target()));
        bytes.extend_from_slice(&(REQUIRED_SECTIONS.len() as u32).to_be_bytes());
        for (section, item_count, payload_offset, payload_length) in directory {
            bytes.push(section.byte());
            bytes.extend_from_slice(&item_count.to_be_bytes());
            bytes.extend_from_slice(&payload_offset.to_be_bytes());
            bytes.extend_from_slice(&payload_length.to_be_bytes());
        }
        bytes.extend(payloads);
        bytes
    }
}

fn items_len(len: usize) -> u32 {
    u32::try_from(len).expect("fixture section fits")
}

fn item_frame(section: SectionTag, payload: Vec<u8>) -> Vec<u8> {
    let mut bytes = vec![section.byte(), 1];
    bytes.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    bytes.extend(payload);
    bytes
}

fn profile_bytes(profile: KernelProfileRecord) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&profile.profile_id.to_be_bytes());
    bytes.extend_from_slice(&profile.clause_schema_version.to_be_bytes());
    bytes.extend_from_slice(&profile.clause_encoding_version.to_be_bytes());
    bytes.push(profile.clause_tautology_policy.tag());
    bytes.push(profile.certificate_hash_input_algorithm.tag());
    bytes
}

fn fingerprint_bytes(fingerprint: &Fingerprint) -> Vec<u8> {
    let mut bytes = vec![fingerprint.algorithm_id];
    bytes.extend_from_slice(&(fingerprint.digest.len() as u32).to_be_bytes());
    bytes.extend_from_slice(&fingerprint.digest);
    bytes
}

fn bytes_field(bytes: &[u8]) -> Vec<u8> {
    let mut encoded = Vec::new();
    encoded.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
    encoded.extend_from_slice(bytes);
    encoded
}

fn symbol_manifest_entry(kind: u8, id: u32) -> Vec<u8> {
    let mut bytes = vec![kind];
    bytes.extend_from_slice(&id.to_be_bytes());
    bytes
}

fn variable_manifest_entry(id: u32) -> Vec<u8> {
    id.to_be_bytes().to_vec()
}

fn imported_fact(id: u32, package: &[u8], module: &[u8], item: &[u8]) -> Vec<u8> {
    imported_fact_with_fingerprint(id, package, module, item, vec![7, 8], 1)
}

fn imported_fact_with_fingerprint(
    id: u32,
    package: &[u8],
    module: &[u8],
    item: &[u8],
    fingerprint: Vec<u8>,
    status: u8,
) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&id.to_be_bytes());
    bytes.extend(bytes_field(package));
    bytes.extend(bytes_field(module));
    bytes.extend(bytes_field(item));
    bytes.extend(fingerprint_bytes(&Fingerprint::new(1, fingerprint)));
    bytes.push(status);
    bytes
}

fn generated_clause_with_literals(id: u32, form: u8, literals: Vec<Vec<u8>>) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&id.to_be_bytes());
    bytes.push(form);
    bytes.extend_from_slice(&(literals.len() as u32).to_be_bytes());
    for literal in literals {
        bytes.extend(literal);
    }
    bytes
}

fn literal_record(polarity: u8, kind: u8, symbol_id: u32, terms: Vec<Vec<u8>>) -> Vec<u8> {
    let mut bytes = vec![polarity, kind];
    bytes.extend_from_slice(&symbol_id.to_be_bytes());
    bytes.extend_from_slice(&(terms.len() as u32).to_be_bytes());
    bytes.extend_from_slice(&(terms.len() as u32).to_be_bytes());
    for term in terms {
        bytes.extend(term);
    }
    bytes
}

fn variable_term(id: u32) -> Vec<u8> {
    let mut bytes = vec![1];
    bytes.extend_from_slice(&id.to_be_bytes());
    bytes
}

fn application_term(kind: u8, symbol_id: u32, arguments: Vec<Vec<u8>>) -> Vec<u8> {
    let mut bytes = vec![2, kind];
    bytes.extend_from_slice(&symbol_id.to_be_bytes());
    bytes.extend_from_slice(&(arguments.len() as u32).to_be_bytes());
    for argument in arguments {
        bytes.extend(argument);
    }
    bytes
}

fn nested_term(depth: u32) -> Vec<u8> {
    if depth == 0 {
        return variable_term(1);
    }
    let mut bytes = vec![3];
    bytes.extend_from_slice(&depth.to_be_bytes());
    bytes.extend(nested_term(depth - 1));
    bytes
}

fn substitution_entry(id: u32, freshness_refs: Vec<u32>) -> Vec<u8> {
    substitution_entry_with_refs(id, freshness_refs, vec![2])
}

fn substitution_entry_with_refs(
    id: u32,
    freshness_refs: Vec<u32>,
    free_variable_refs: Vec<u32>,
) -> Vec<u8> {
    substitution_entry_with_term(id, variable_term(1))
        .with_freshness(freshness_refs)
        .with_free_variables(free_variable_refs)
}

fn substitution_entry_with_term(id: u32, term: Vec<u8>) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&id.to_be_bytes());
    bytes.extend(term.clone());
    bytes.extend(term);
    bytes.extend(bytes_field(b"binder"));
    bytes.extend(ref_list(&[1]));
    bytes.extend(ref_list(&[2]));
    bytes
}

trait WithFreshness {
    fn with_freshness(self, refs: Vec<u32>) -> Self;
    fn with_free_variables(self, refs: Vec<u32>) -> Self;
}

impl WithFreshness for Vec<u8> {
    fn with_freshness(mut self, refs: Vec<u32>) -> Self {
        let mut replacement = Vec::new();
        replacement.extend(ref_list(&refs));
        let start = self.len() - ref_list(&[1]).len() - ref_list(&[2]).len();
        self.splice(start..start + ref_list(&[1]).len(), replacement);
        self
    }

    fn with_free_variables(mut self, refs: Vec<u32>) -> Self {
        let mut replacement = Vec::new();
        replacement.extend(ref_list(&refs));
        let start = self.len() - ref_list(&[2]).len();
        self.splice(start.., replacement);
        self
    }
}

fn ref_list(ids: &[u32]) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&(ids.len() as u32).to_be_bytes());
    for id in ids {
        bytes.extend_from_slice(&id.to_be_bytes());
    }
    bytes
}

fn clause_ref(namespace: u8, id: u32) -> Vec<u8> {
    let mut bytes = vec![namespace];
    bytes.extend_from_slice(&id.to_be_bytes());
    bytes
}

fn resolution_step(id: u32, parent_a: Vec<u8>, parent_b: Vec<u8>, generated_id: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&id.to_be_bytes());
    bytes.extend(parent_a);
    bytes.extend(parent_b);
    bytes.extend(literal_record(2, 1, 1, vec![variable_term(1)]));
    bytes.extend(clause_ref(1, generated_id));
    bytes
}

fn resolution_step_with_pivot(
    id: u32,
    parent_a: Vec<u8>,
    parent_b: Vec<u8>,
    pivot: Vec<u8>,
    generated_id: u32,
) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&id.to_be_bytes());
    bytes.extend(parent_a);
    bytes.extend(parent_b);
    bytes.extend(pivot);
    bytes.extend(clause_ref(1, generated_id));
    bytes
}

fn resolution_step_with_generated_ref(
    id: u32,
    parent_a: Vec<u8>,
    parent_b: Vec<u8>,
    generated_ref: Vec<u8>,
) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&id.to_be_bytes());
    bytes.extend(parent_a);
    bytes.extend(parent_b);
    bytes.extend(literal_record(2, 1, 1, vec![variable_term(1)]));
    bytes.extend(generated_ref);
    bytes
}

fn derived_fact(id: u32, source: Vec<u8>) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&id.to_be_bytes());
    bytes.extend(source);
    bytes.extend(bytes_field(b"derived"));
    bytes
}

fn final_goal(namespace: u8, id: u32) -> Vec<u8> {
    let mut bytes = vec![namespace];
    bytes.extend_from_slice(&id.to_be_bytes());
    bytes
}

fn directory_entry_start(target: &Fingerprint, section: SectionTag) -> usize {
    directory_start(target) + section_index(section) * DIRECTORY_ENTRY_LEN
}

fn section_payload_offset(bytes: &[u8], target: &Fingerprint, section: SectionTag) -> u32 {
    let start = directory_entry_start(target, section) + 5;
    u32::from_be_bytes([
        bytes[start],
        bytes[start + 1],
        bytes[start + 2],
        bytes[start + 3],
    ])
}

fn section_payload_length(bytes: &[u8], target: &Fingerprint, section: SectionTag) -> u32 {
    let start = directory_entry_start(target, section) + 9;
    u32::from_be_bytes([
        bytes[start],
        bytes[start + 1],
        bytes[start + 2],
        bytes[start + 3],
    ])
}

fn section_index(section: SectionTag) -> usize {
    REQUIRED_SECTIONS
        .iter()
        .position(|candidate| *candidate == section)
        .expect("known section")
}

fn set_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_be_bytes());
}

fn assert_detail(
    result: Result<ParsedCertificate, CertificateParseError>,
    detail: CertificateRejectionDetail,
) {
    let error = result.expect_err("expected parser rejection");
    assert_eq!(error.category, FailureCategory::CertificateRejection);
    assert_eq!(error.detail, detail);
}

fn assert_rejection_location(
    result: Result<ParsedCertificate, CertificateParseError>,
    detail: CertificateRejectionDetail,
    byte_offset: usize,
    section_tag: Option<SectionTag>,
    item_index: Option<u32>,
    field_path: &'static str,
) {
    let error = result.expect_err("expected parser rejection");
    assert_eq!(error.category, FailureCategory::CertificateRejection);
    assert_eq!(error.detail, detail);
    assert_eq!(error.location.byte_offset, byte_offset);
    assert_eq!(error.location.section_tag, section_tag);
    assert_eq!(error.location.item_index, item_index);
    assert_eq!(error.location.field_path, Some(field_path));
}

fn contains_subsequence(bytes: &[u8], needle: &[u8]) -> bool {
    bytes.windows(needle.len()).any(|window| window == needle)
}
