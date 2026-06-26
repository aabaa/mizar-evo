use std::collections::BTreeMap;

use crate::{
    certificate_parser::{
        CertificateHashInputAlgorithm, ClauseTautologyPolicy, Fingerprint, KernelProfileRecord,
    },
    clause::{Atom, SymbolKey, SymbolKind, Term, VariableId},
    formula_evidence::{
        FormulaEvidenceParseContext, FormulaSourceClass, GoalPolarity, parse_formula_evidence,
    },
    substitution_checker::{TermPath, TermPathSegment},
};

use super::*;

#[test]
fn encodes_stable_tseitin_problem_and_goal_polarity() {
    let premise = Formula::Or(vec![atom_formula(1), atom_formula(2)]);
    let goal = atom_formula(1);
    let parsed = parsed_evidence(
        vec![formula_item(1, 11, &premise)],
        Vec::new(),
        goal_item(20, GoalPolarity::AssertFalseForRefutation, &goal),
    );

    let problem =
        encode_formula_evidence(&parsed, &SatEncodingContext::v1()).expect("encoding succeeds");

    assert!(
        problem
            .canonical_bytes()
            .starts_with(SAT_PROBLEM_DOMAIN_SEPARATOR)
    );
    assert_eq!(problem.atom_variables.len(), 2);
    assert_eq!(problem.atom_variables[0].variable, SatVariable(1));
    assert_eq!(problem.atom_variables[0].atom, atom(1, Vec::new()));
    assert_eq!(problem.atom_variables[1].variable, SatVariable(2));
    assert_eq!(problem.atom_variables[1].atom, atom(2, Vec::new()));
    assert_eq!(
        problem
            .assertions
            .iter()
            .map(|entry| entry.assertion_kind)
            .collect::<Vec<_>>(),
        vec![ASSERTION_KIND_PREMISE, ASSERTION_KIND_FINAL_GOAL]
    );
    assert_eq!(problem.clauses.len(), 5);
    assert_eq!(
        problem.clauses[0].literals,
        vec![
            SatLiteral::negative(SatVariable(1)),
            SatLiteral::positive(SatVariable(3))
        ]
    );
    assert_eq!(
        problem.clauses[1].literals,
        vec![
            SatLiteral::negative(SatVariable(2)),
            SatLiteral::positive(SatVariable(3))
        ]
    );
    assert_eq!(
        problem.clauses[2].literals,
        vec![
            SatLiteral::positive(SatVariable(1)),
            SatLiteral::positive(SatVariable(2)),
            SatLiteral::negative(SatVariable(3)),
        ]
    );
    assert_eq!(
        problem.clauses[3].literals,
        vec![SatLiteral::positive(SatVariable(3))]
    );
    assert_eq!(
        problem.clauses[4].literals,
        vec![SatLiteral::negative(SatVariable(1))]
    );
    assert_eq!(problem.canonical_bytes(), problem.canonical_bytes());
}

#[test]
fn formula_wide_substitution_adds_recomputed_instance_assertion() {
    let source = Formula::Atom(atom(
        1,
        vec![
            Term::Variable(VariableId(1)),
            Term::Variable(VariableId(1)),
            Term::Variable(VariableId(2)),
        ],
    ));
    let parsed = parsed_evidence(
        vec![formula_item(1, 11, &source)],
        vec![substitution_item(
            7,
            1,
            12,
            empty_binder_context(),
            vec![
                replacement(
                    1,
                    Term::Variable(VariableId(2)),
                    REPLACEMENT_ROLE_TERM_ARGUMENT,
                ),
                replacement(
                    2,
                    Term::Variable(VariableId(3)),
                    REPLACEMENT_ROLE_TERM_ARGUMENT,
                ),
            ],
            PAYLOAD_KIND_FORMAL_TO_ACTUAL_MAP,
            TermPath::root(),
        )],
        goal_item(20, GoalPolarity::AssertFalseForRefutation, &atom_formula(2)),
    );

    let problem = encode_formula_evidence(&parsed, &SatEncodingContext::v1())
        .expect("substitution instance is encoded");

    let instance = problem
        .assertions
        .iter()
        .find(|assertion| assertion.assertion_kind == ASSERTION_KIND_SUBSTITUTION_INSTANCE)
        .expect("derived substitution assertion");
    assert_eq!(instance.source_formula_id, Some(1));
    assert_eq!(instance.substitution_id, Some(7));
    assert_eq!(
        instance.formula,
        Formula::Atom(atom(
            1,
            vec![
                Term::Variable(VariableId(2)),
                Term::Variable(VariableId(2)),
                Term::Variable(VariableId(3)),
            ]
        ))
    );
    assert_eq!(
        instance.formula_fingerprint,
        formula_fingerprint(&instance.formula)
    );
    assert!(
        problem
            .assertions
            .iter()
            .any(
                |assertion| assertion.assertion_kind == ASSERTION_KIND_PREMISE
                    && assertion.formula == source
            ),
        "source premise remains asserted separately"
    );
}

#[test]
fn substitution_rewrites_only_unbound_formal_occurrences() {
    let source = Formula::Atom(atom(
        1,
        vec![
            Term::Variable(VariableId(1)),
            Term::BinderNormalized {
                binder_id: 5,
                body: Box::new(Term::Variable(VariableId(1))),
            },
        ],
    ));
    let parsed = parsed_evidence(
        vec![formula_item(1, 11, &source)],
        vec![substitution_item(
            7,
            1,
            12,
            binder_context(vec![(5, 0, 1, 1)], Vec::new(), Vec::new()),
            vec![replacement(
                1,
                Term::Variable(VariableId(2)),
                REPLACEMENT_ROLE_TERM_ARGUMENT,
            )],
            PAYLOAD_KIND_FORMAL_TO_ACTUAL_MAP,
            TermPath::root(),
        )],
        goal_item(20, GoalPolarity::AssertFalseForRefutation, &atom_formula(2)),
    );

    let problem = encode_formula_evidence(&parsed, &SatEncodingContext::v1())
        .expect("substitution instance is encoded");
    let instance = problem
        .assertions
        .iter()
        .find(|assertion| assertion.assertion_kind == ASSERTION_KIND_SUBSTITUTION_INSTANCE)
        .expect("derived substitution assertion");

    assert_eq!(
        instance.formula,
        Formula::Atom(atom(
            1,
            vec![
                Term::Variable(VariableId(2)),
                Term::BinderNormalized {
                    binder_id: 5,
                    body: Box::new(Term::Variable(VariableId(1))),
                },
            ],
        ))
    );
}

#[test]
fn nested_binder_prescan_ignores_inner_bound_formal_occurrences() {
    let source = Formula::Atom(atom(
        1,
        vec![
            Term::Variable(VariableId(1)),
            Term::BinderNormalized {
                binder_id: 5,
                body: Box::new(Term::BinderNormalized {
                    binder_id: 6,
                    body: Box::new(Term::Variable(VariableId(1))),
                }),
            },
        ],
    ));
    let parsed = parsed_evidence(
        vec![formula_item(1, 11, &source)],
        vec![substitution_item(
            7,
            1,
            12,
            binder_context(vec![(5, 0, 2, 1), (6, 1, 1, 1)], Vec::new(), Vec::new()),
            vec![replacement(
                1,
                Term::Variable(VariableId(2)),
                REPLACEMENT_ROLE_TERM_ARGUMENT,
            )],
            PAYLOAD_KIND_FORMAL_TO_ACTUAL_MAP,
            TermPath::root(),
        )],
        goal_item(20, GoalPolarity::AssertFalseForRefutation, &atom_formula(2)),
    );

    let problem = encode_formula_evidence(&parsed, &SatEncodingContext::v1())
        .expect("inner-bound formal does not trigger outer capture precheck");
    let instance = problem
        .assertions
        .iter()
        .find(|assertion| assertion.assertion_kind == ASSERTION_KIND_SUBSTITUTION_INSTANCE)
        .expect("derived substitution assertion");

    assert_eq!(
        instance.formula,
        Formula::Atom(atom(
            1,
            vec![
                Term::Variable(VariableId(2)),
                Term::BinderNormalized {
                    binder_id: 5,
                    body: Box::new(Term::BinderNormalized {
                        binder_id: 6,
                        body: Box::new(Term::Variable(VariableId(1))),
                    }),
                },
            ],
        ))
    );
}

#[test]
fn nested_and_not_encoding_uses_preorder_aux_variables_and_positive_goal_unit() {
    let premise = Formula::And(vec![
        Formula::Not(Box::new(atom_formula(1))),
        Formula::Or(vec![atom_formula(2), atom_formula(3)]),
    ]);
    let parsed = parsed_evidence(
        vec![formula_item(1, 11, &premise)],
        Vec::new(),
        goal_item(20, GoalPolarity::AssertTrueForConsistency, &atom_formula(1)),
    );

    let problem = encode_formula_evidence(&parsed, &SatEncodingContext::v1())
        .expect("nested encoding succeeds");

    assert_eq!(problem.atom_variables.len(), 3);
    assert_eq!(problem.clauses.len(), 8);
    assert_eq!(
        problem.clauses[0].literals,
        vec![
            SatLiteral::negative(SatVariable(1)),
            SatLiteral::negative(SatVariable(4)),
        ]
    );
    assert_eq!(
        problem.clauses[1].literals,
        vec![
            SatLiteral::negative(SatVariable(2)),
            SatLiteral::positive(SatVariable(5)),
        ]
    );
    assert_eq!(
        problem.clauses[2].literals,
        vec![
            SatLiteral::negative(SatVariable(3)),
            SatLiteral::positive(SatVariable(5)),
        ]
    );
    assert_eq!(
        problem.clauses[3].literals,
        vec![
            SatLiteral::positive(SatVariable(2)),
            SatLiteral::positive(SatVariable(3)),
            SatLiteral::negative(SatVariable(5)),
        ]
    );
    assert_eq!(
        problem.clauses[4].literals,
        vec![
            SatLiteral::negative(SatVariable(4)),
            SatLiteral::positive(SatVariable(5)),
        ]
    );
    assert_eq!(
        problem.clauses[5].literals,
        vec![
            SatLiteral::positive(SatVariable(1)),
            SatLiteral::positive(SatVariable(4)),
            SatLiteral::negative(SatVariable(5)),
        ]
    );
    assert_eq!(
        problem.clauses[6].literals,
        vec![SatLiteral::positive(SatVariable(4))]
    );
    assert_eq!(
        problem.clauses[7].literals,
        vec![SatLiteral::positive(SatVariable(1))]
    );
}

#[test]
fn atom_variables_are_sorted_by_canonical_atom_bytes_not_formula_ids() {
    let parsed = parsed_evidence(
        vec![
            formula_item(1, 11, &atom_formula(2)),
            formula_item(2, 12, &atom_formula(1)),
        ],
        Vec::new(),
        goal_item(20, GoalPolarity::AssertTrueForConsistency, &atom_formula(3)),
    );

    let first = encode_formula_evidence(&parsed, &SatEncodingContext::v1())
        .expect("first encoding succeeds");
    let second = encode_formula_evidence(&parsed, &SatEncodingContext::v1())
        .expect("second encoding succeeds");

    assert_eq!(first.canonical_bytes(), second.canonical_bytes());
    assert_eq!(
        first
            .atom_variables
            .iter()
            .map(|entry| entry.atom.symbol.id.0)
            .collect::<Vec<_>>(),
        vec![1, 2, 3]
    );
    let final_goal = first
        .assertions
        .iter()
        .find(|assertion| assertion.assertion_kind == ASSERTION_KIND_FINAL_GOAL)
        .expect("final goal assertion");
    assert!(final_goal.asserted_true);
}

#[test]
fn unsupported_substitution_shapes_reject_fail_closed() {
    let source = Formula::Atom(atom(1, vec![Term::Variable(VariableId(1))]));
    let cases = [
        (
            substitution_item(
                7,
                1,
                12,
                empty_binder_context(),
                vec![replacement(
                    1,
                    Term::Variable(VariableId(2)),
                    REPLACEMENT_ROLE_TERM_ARGUMENT,
                )],
                PAYLOAD_KIND_FORMAL_TO_ACTUAL_MAP,
                TermPath::new(vec![TermPathSegment::application_argument(0)]),
            ),
            "substitution.payload.rewrite_path",
        ),
        (
            substitution_item(
                7,
                1,
                12,
                empty_binder_context(),
                vec![replacement(
                    1,
                    Term::Variable(VariableId(2)),
                    REPLACEMENT_ROLE_TERM_ARGUMENT,
                )],
                2,
                TermPath::root(),
            ),
            "substitution.payload.payload_kind",
        ),
        (
            substitution_item(
                7,
                1,
                12,
                empty_binder_context(),
                vec![replacement(1, Term::Variable(VariableId(2)), 3)],
                PAYLOAD_KIND_FORMAL_TO_ACTUAL_MAP,
                TermPath::root(),
            ),
            "substitution.payload.replacement_role",
        ),
        (
            substitution_item_with_tail(
                substitution_item_prefix(
                    7,
                    1,
                    12,
                    empty_binder_context(),
                    vec![replacement(
                        1,
                        Term::Variable(VariableId(2)),
                        REPLACEMENT_ROLE_TERM_ARGUMENT,
                    )],
                    PAYLOAD_KIND_FORMAL_TO_ACTUAL_MAP,
                    TermPath::root(),
                ),
                substitution_tail_with_freshness(7),
            ),
            "substitution.freshness_witnesses",
        ),
        (
            substitution_item_with_tail(
                substitution_item_prefix(
                    7,
                    1,
                    12,
                    empty_binder_context(),
                    vec![replacement(
                        1,
                        Term::Variable(VariableId(2)),
                        REPLACEMENT_ROLE_TERM_ARGUMENT,
                    )],
                    PAYLOAD_KIND_FORMAL_TO_ACTUAL_MAP,
                    TermPath::root(),
                ),
                substitution_tail_with_free_variable_constraint(7),
            ),
            "substitution.free_variable_constraints",
        ),
        (
            substitution_item(
                7,
                1,
                12,
                empty_binder_context(),
                vec![replacement(
                    3,
                    Term::Variable(VariableId(2)),
                    REPLACEMENT_ROLE_TERM_ARGUMENT,
                )],
                PAYLOAD_KIND_FORMAL_TO_ACTUAL_MAP,
                TermPath::root(),
            ),
            "substitution.payload.formal_variable_id",
        ),
    ];

    for (substitution, field_path) in cases {
        let parsed = parsed_evidence(
            vec![formula_item(1, 11, &source)],
            vec![substitution],
            goal_item(20, GoalPolarity::AssertFalseForRefutation, &atom_formula(2)),
        );
        let error = encode_formula_evidence(&parsed, &SatEncodingContext::v1())
            .expect_err("unsupported substitution shape rejects");

        assert_eq!(error.category(), RejectionCategory::KernelRejection);
        assert_eq!(error.detail(), RejectionDetail::InvalidSubstitution);
        assert_eq!(error.location().substitution_id, Some(7));
        assert_eq!(error.location().field_path, Some(field_path));
    }
}

#[test]
fn duplicate_formal_ids_reject_before_encoding() {
    let source = Formula::Atom(atom(1, vec![Term::Variable(VariableId(1))]));
    let bytes = evidence_bytes(
        vec![formula_item(1, 11, &source)],
        vec![substitution_item(
            7,
            1,
            12,
            empty_binder_context(),
            vec![
                replacement(
                    1,
                    Term::Variable(VariableId(2)),
                    REPLACEMENT_ROLE_TERM_ARGUMENT,
                ),
                replacement(
                    1,
                    Term::Variable(VariableId(3)),
                    REPLACEMENT_ROLE_TERM_ARGUMENT,
                ),
            ],
            PAYLOAD_KIND_FORMAL_TO_ACTUAL_MAP,
            TermPath::root(),
        )],
        goal_item(20, GoalPolarity::AssertFalseForRefutation, &atom_formula(2)),
    );

    let error = parse_formula_evidence(
        &bytes,
        &FormulaEvidenceParseContext::v1(target(), profile()),
    )
    .expect_err("duplicate formal ids reject during parser-owned structural validation");

    assert_eq!(error.category(), RejectionCategory::CertificateRejection);
    assert_eq!(error.detail(), RejectionDetail::MalformedWitnessData);
    assert_eq!(
        error.location().field_path,
        Some("substitution.payload.formal_variable_id")
    );
}

#[test]
fn canonical_sat_bytes_pin_header_assertions_and_clause_encoding() {
    let premise = atom_formula(1);
    let parsed = parsed_evidence(
        vec![formula_item(1, 11, &premise)],
        Vec::new(),
        goal_item(20, GoalPolarity::AssertFalseForRefutation, &atom_formula(1)),
    );
    let problem =
        encode_formula_evidence(&parsed, &SatEncodingContext::v1()).expect("encoding succeeds");
    let mut reader = CanonicalReader::new(problem.canonical_bytes());

    reader.expect_bytes(SAT_PROBLEM_DOMAIN_SEPARATOR);
    assert_eq!(reader.read_u16(), SAT_PROBLEM_SCHEMA_VERSION);
    assert_eq!(reader.read_u16(), SAT_PROBLEM_ENCODING_VERSION);
    assert_eq!(reader.read_fingerprint(), target());

    assert_eq!(reader.read_u32(), 1);
    assert_eq!(reader.read_u32(), 1);
    assert_eq!(
        reader.read_bytes(),
        atom(1, Vec::new()).canonical_bytes().unwrap()
    );

    assert_eq!(reader.read_u32(), 2);
    assert_eq!(reader.read_u8(), ASSERTION_KIND_PREMISE);
    assert_eq!(reader.read_u8(), 1);
    assert_eq!(reader.read_option_u32(), Some(1));
    assert_eq!(reader.read_option_u32(), None);
    assert_eq!(reader.read_fingerprint(), formula_fingerprint(&premise));
    assert_eq!(reader.read_bytes(), premise.canonical_hash_input().unwrap());
    assert_eq!(reader.read_u8(), ASSERTION_KIND_FINAL_GOAL);
    assert_eq!(reader.read_u8(), 0);
    assert_eq!(reader.read_option_u32(), None);
    assert_eq!(reader.read_option_u32(), None);
    assert_eq!(reader.read_fingerprint(), formula_fingerprint(&premise));
    assert_eq!(reader.read_bytes(), premise.canonical_hash_input().unwrap());

    assert_eq!(reader.read_u32(), 2);
    assert_eq!(reader.read_u32(), 1);
    assert_eq!(reader.read_u32(), 1);
    assert_eq!(reader.read_u8(), 1);
    assert_eq!(reader.read_u32(), 1);
    assert_eq!(reader.read_u32(), 1);
    assert_eq!(reader.read_u8(), 0);
    reader.finish();
}

#[test]
fn canonical_sat_bytes_include_substitution_assertion_identity() {
    let source = Formula::Atom(atom(1, vec![Term::Variable(VariableId(1))]));
    let instance = Formula::Atom(atom(1, vec![Term::Variable(VariableId(2))]));
    let parsed = parsed_evidence(
        vec![formula_item(1, 11, &source)],
        vec![substitution_item(
            7,
            1,
            12,
            empty_binder_context(),
            vec![replacement(
                1,
                Term::Variable(VariableId(2)),
                REPLACEMENT_ROLE_TERM_ARGUMENT,
            )],
            PAYLOAD_KIND_FORMAL_TO_ACTUAL_MAP,
            TermPath::root(),
        )],
        goal_item(20, GoalPolarity::AssertFalseForRefutation, &atom_formula(2)),
    );
    let problem =
        encode_formula_evidence(&parsed, &SatEncodingContext::v1()).expect("encoding succeeds");
    let mut reader = CanonicalReader::new(problem.canonical_bytes());

    reader.expect_bytes(SAT_PROBLEM_DOMAIN_SEPARATOR);
    assert_eq!(reader.read_u16(), SAT_PROBLEM_SCHEMA_VERSION);
    assert_eq!(reader.read_u16(), SAT_PROBLEM_ENCODING_VERSION);
    assert_eq!(reader.read_fingerprint(), target());
    reader.skip_atom_manifest();

    assert_eq!(reader.read_u32(), 3);
    reader.skip_assertion();
    assert_eq!(reader.read_u8(), ASSERTION_KIND_SUBSTITUTION_INSTANCE);
    assert_eq!(reader.read_u8(), 1);
    assert_eq!(reader.read_option_u32(), Some(1));
    assert_eq!(reader.read_option_u32(), Some(7));
    assert_eq!(reader.read_fingerprint(), formula_fingerprint(&instance));
    assert_eq!(
        reader.read_bytes(),
        instance.canonical_hash_input().unwrap()
    );
    reader.skip_assertion();
    reader.skip_clauses();
    reader.finish();
}

#[test]
fn capture_under_binder_rejects_fail_closed_without_alpha_repair() {
    let source = Formula::Atom(atom(
        1,
        vec![Term::BinderNormalized {
            binder_id: 5,
            body: Box::new(Term::Variable(VariableId(2))),
        }],
    ));
    let parsed = parsed_evidence(
        vec![formula_item(1, 11, &source)],
        vec![substitution_item(
            7,
            1,
            12,
            binder_context(vec![(5, 0, 1, 1)], Vec::new(), Vec::new()),
            vec![replacement(
                2,
                Term::Variable(VariableId(1)),
                REPLACEMENT_ROLE_TERM_ARGUMENT,
            )],
            PAYLOAD_KIND_FORMAL_TO_ACTUAL_MAP,
            TermPath::root(),
        )],
        goal_item(20, GoalPolarity::AssertFalseForRefutation, &atom_formula(2)),
    );

    let error = encode_formula_evidence(&parsed, &SatEncodingContext::v1())
        .expect_err("capture must reject");

    assert_eq!(error.category(), RejectionCategory::KernelRejection);
    assert_eq!(error.detail(), RejectionDetail::InvalidSubstitution);
    assert_eq!(error.location().substitution_id, Some(7));
    assert_eq!(error.location().field_path, Some("substitution.capture"));
}

#[test]
fn actual_term_binder_ids_must_be_declared_in_binder_context() {
    let source = Formula::Atom(atom(1, vec![Term::Variable(VariableId(1))]));
    let parsed = parsed_evidence(
        vec![formula_item(1, 11, &source)],
        vec![substitution_item(
            7,
            1,
            12,
            empty_binder_context(),
            vec![replacement(
                1,
                Term::BinderNormalized {
                    binder_id: 9,
                    body: Box::new(Term::Variable(VariableId(2))),
                },
                REPLACEMENT_ROLE_TERM_ARGUMENT,
            )],
            PAYLOAD_KIND_FORMAL_TO_ACTUAL_MAP,
            TermPath::root(),
        )],
        goal_item(20, GoalPolarity::AssertFalseForRefutation, &atom_formula(2)),
    );

    let error = encode_formula_evidence(&parsed, &SatEncodingContext::v1())
        .expect_err("actual term binder id must be context-backed");

    assert_eq!(error.category(), RejectionCategory::KernelRejection);
    assert_eq!(error.detail(), RejectionDetail::InvalidSubstitution);
    assert_eq!(
        error.location().field_path,
        Some("substitution.payload.actual_term")
    );
}

#[test]
fn binder_context_rejects_empty_noncanonical_and_unused_frames() {
    let source = Formula::Atom(atom(1, vec![Term::Variable(VariableId(1))]));
    let cases = [
        (Vec::new(), "substitution.binder_context"),
        (
            binder_context(vec![(5, 1, 1, 1), (6, 0, 2, 1)], Vec::new(), Vec::new()),
            "substitution.binder_context.frames",
        ),
        (
            binder_context(vec![(5, 0, 3, 1)], Vec::new(), Vec::new()),
            "substitution.binder_context",
        ),
    ];

    for (binder_context_encoding, field_path) in cases {
        let parsed = parsed_evidence(
            vec![formula_item(1, 11, &source)],
            vec![substitution_item(
                7,
                1,
                12,
                binder_context_encoding,
                vec![replacement(
                    1,
                    Term::Variable(VariableId(2)),
                    REPLACEMENT_ROLE_TERM_ARGUMENT,
                )],
                PAYLOAD_KIND_FORMAL_TO_ACTUAL_MAP,
                TermPath::root(),
            )],
            goal_item(20, GoalPolarity::AssertFalseForRefutation, &atom_formula(2)),
        );

        let error = encode_formula_evidence(&parsed, &SatEncodingContext::v1())
            .expect_err("binder context shape rejects");

        assert_eq!(error.category(), RejectionCategory::KernelRejection);
        assert_eq!(error.detail(), RejectionDetail::InvalidSubstitution);
        assert_eq!(error.location().field_path, Some(field_path));
    }
}

#[test]
fn resource_limits_reject_before_sat_checking() {
    let parsed = parsed_evidence(
        vec![
            formula_item(1, 11, &atom_formula(1)),
            formula_item(2, 12, &atom_formula(2)),
        ],
        Vec::new(),
        goal_item(20, GoalPolarity::AssertFalseForRefutation, &atom_formula(3)),
    );
    let limits = SatEncodingLimits {
        max_atoms: 1,
        ..SatEncodingLimits::default()
    };

    let error = encode_formula_evidence(&parsed, &SatEncodingContext::v1().with_limits(limits))
        .expect_err("atom variable limit rejects");

    assert_eq!(error.category(), RejectionCategory::KernelRejection);
    assert_eq!(error.detail(), RejectionDetail::ResourceExhaustion);
    assert_eq!(
        error.location().field_path,
        Some("sat_encoding.atom_variables")
    );
}

fn parsed_evidence(
    formulas: Vec<Vec<u8>>,
    substitutions: Vec<Vec<u8>>,
    goal: Vec<u8>,
) -> ParsedKernelEvidence {
    let bytes = evidence_bytes(formulas, substitutions, goal);
    parse_formula_evidence(
        &bytes,
        &FormulaEvidenceParseContext::v1(target(), profile()),
    )
    .expect("test evidence parses")
}

fn evidence_bytes(formulas: Vec<Vec<u8>>, substitutions: Vec<Vec<u8>>, goal: Vec<u8>) -> Vec<u8> {
    let mut provenance = Vec::new();
    let formula_fingerprints = formulas
        .iter()
        .map(|formula| (formula_id(formula), fingerprint_from_slice(&formula[5..])))
        .collect::<BTreeMap<_, _>>();
    for formula in &formulas {
        provenance.push(provenance_item(
            formula_provenance_id(formula),
            &fingerprint_from_slice(&formula[5..]),
        ));
    }
    for substitution in &substitutions {
        let source_formula_id = substitution_source_formula_id(substitution);
        let source_fingerprint = formula_fingerprints
            .get(&source_formula_id)
            .expect("test substitution references a formula");
        provenance.push(provenance_item(
            substitution_provenance_id(substitution),
            source_fingerprint,
        ));
    }
    provenance.push(provenance_item(
        goal_provenance_id(&goal),
        &fingerprint_from_slice(&goal[1..]),
    ));
    envelope(vec![
        (1, symbol_items()),
        (2, variable_items()),
        (3, formulas),
        (4, substitutions),
        (5, provenance),
        (6, vec![goal]),
    ])
}

fn envelope(sections: Vec<(u8, Vec<Vec<u8>>)>) -> Vec<u8> {
    let mut payloads = Vec::new();
    let mut directory = Vec::new();
    let mut offset = 0u32;
    for (section, items) in &sections {
        let mut payload = Vec::new();
        for item in items {
            payload.push(*section);
            payload.push(1);
            put_u32(item.len() as u32, &mut payload);
            payload.extend_from_slice(item);
        }
        let length = payload.len() as u32;
        directory.push((*section, items.len() as u32, offset, length));
        offset += length;
        payloads.push(payload);
    }

    let mut bytes = b"MIZAR_KERNEL_EVIDENCE\0".to_vec();
    put_u16(1, &mut bytes);
    put_u16(1, &mut bytes);
    put_profile(&mut bytes);
    put_fingerprint(&target(), &mut bytes);
    put_u32(sections.len() as u32, &mut bytes);
    for (section, count, payload_offset, payload_length) in directory {
        bytes.push(section);
        put_u32(count, &mut bytes);
        put_u32(payload_offset, &mut bytes);
        put_u32(payload_length, &mut bytes);
    }
    for payload in payloads {
        bytes.extend(payload);
    }
    bytes
}

fn symbol_items() -> Vec<Vec<u8>> {
    (1..=3)
        .map(|id| {
            let mut item = Vec::new();
            item.push(symbol_kind_tag(SymbolKind::Predicate));
            put_u32(id, &mut item);
            item
        })
        .collect()
}

fn variable_items() -> Vec<Vec<u8>> {
    (1..=3)
        .map(|id| {
            let mut item = Vec::new();
            put_u32(id, &mut item);
            item
        })
        .collect()
}

fn formula_item(formula_id: u32, provenance_id: u32, formula: &Formula) -> Vec<u8> {
    let fingerprint = formula_fingerprint(formula);
    let mut item = Vec::new();
    put_u32(formula_id, &mut item);
    item.push(FormulaSourceClass::LocalHypothesis.tag());
    put_fingerprint(&fingerprint, &mut item);
    put_u32(provenance_id, &mut item);
    put_u32(1, &mut item);
    put_formula(formula, &mut item);
    item
}

fn substitution_item(
    substitution_id: u32,
    source_formula_id: u32,
    provenance_id: u32,
    binder_context_encoding: Vec<u8>,
    replacements: Vec<Replacement>,
    payload_kind: u8,
    rewrite_path: TermPath,
) -> Vec<u8> {
    substitution_item_with_tail(
        substitution_item_prefix(
            substitution_id,
            source_formula_id,
            provenance_id,
            binder_context_encoding,
            replacements,
            payload_kind,
            rewrite_path,
        ),
        empty_substitution_tail(),
    )
}

fn substitution_item_prefix(
    substitution_id: u32,
    source_formula_id: u32,
    provenance_id: u32,
    binder_context_encoding: Vec<u8>,
    replacements: Vec<Replacement>,
    payload_kind: u8,
    rewrite_path: TermPath,
) -> Vec<u8> {
    let mut item = Vec::new();
    put_u32(substitution_id, &mut item);
    put_u32(source_formula_id, &mut item);
    put_u32(provenance_id, &mut item);
    put_bytes(&binder_context_encoding, &mut item);
    put_u32(substitution_id, &mut item);
    item.push(payload_kind);
    put_term_path(&rewrite_path, &mut item);
    put_u32(replacements.len() as u32, &mut item);
    for replacement in replacements {
        put_u32(replacement.formal_variable_id.0, &mut item);
        put_term(&replacement.actual_term, &mut item);
        item.push(replacement.replacement_role);
    }
    item
}

fn substitution_item_with_tail(mut item: Vec<u8>, tail: Vec<u8>) -> Vec<u8> {
    item.extend(tail);
    item
}

fn empty_substitution_tail() -> Vec<u8> {
    let mut tail = Vec::new();
    put_u32(0, &mut tail);
    put_u32(0, &mut tail);
    tail
}

fn substitution_tail_with_freshness(substitution_id: u32) -> Vec<u8> {
    let mut tail = Vec::new();
    put_u32(1, &mut tail);
    put_u32(1, &mut tail);
    put_u32(substitution_id, &mut tail);
    put_u32(3, &mut tail);
    put_term_path(&TermPath::root(), &mut tail);
    put_u32(0, &mut tail);
    put_u32(0, &mut tail);
    put_u32(0, &mut tail);
    tail
}

fn substitution_tail_with_free_variable_constraint(substitution_id: u32) -> Vec<u8> {
    let mut tail = Vec::new();
    put_u32(0, &mut tail);
    put_u32(1, &mut tail);
    put_u32(1, &mut tail);
    put_u32(substitution_id, &mut tail);
    tail.push(1);
    put_u32(1, &mut tail);
    put_term_path(&TermPath::root(), &mut tail);
    put_u32(0, &mut tail);
    tail
}

fn replacement(formal_variable_id: u32, actual_term: Term, replacement_role: u8) -> Replacement {
    Replacement::new(
        VariableId(formal_variable_id),
        actual_term,
        replacement_role,
    )
}

fn goal_item(provenance_id: u32, polarity: GoalPolarity, formula: &Formula) -> Vec<u8> {
    let fingerprint = formula_fingerprint(formula);
    let mut item = Vec::new();
    item.push(polarity.tag());
    put_fingerprint(&fingerprint, &mut item);
    put_u32(provenance_id, &mut item);
    put_formula(formula, &mut item);
    item
}

fn provenance_item(provenance_id: u32, fingerprint: &Fingerprint) -> Vec<u8> {
    let mut item = Vec::new();
    put_u32(provenance_id, &mut item);
    put_fingerprint(&target(), &mut item);
    put_fingerprint(fingerprint, &mut item);
    put_bytes(b"producer", &mut item);
    item
}

fn atom_formula(symbol_id: u32) -> Formula {
    Formula::Atom(atom(symbol_id, Vec::new()))
}

fn atom(symbol_id: u32, arguments: Vec<Term>) -> Atom {
    Atom::with_arity(
        SymbolKey::new(SymbolKind::Predicate, symbol_id),
        arguments.len() as u32,
        arguments,
    )
}

fn formula_fingerprint(formula: &Formula) -> Fingerprint {
    Fingerprint::new(
        SUPPORTED_FORMULA_FINGERPRINT_ALGORITHM_ID,
        formula
            .canonical_hash_input()
            .expect("test formula is canonical"),
    )
}

fn fingerprint_from_slice(bytes: &[u8]) -> Fingerprint {
    let algorithm_id = bytes[0];
    let len = u32::from_be_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]) as usize;
    Fingerprint::new(algorithm_id, bytes[5..5 + len].to_vec())
}

fn formula_provenance_id(item: &[u8]) -> u32 {
    let start = 10 + fingerprint_len(item);
    u32::from_be_bytes([
        item[start],
        item[start + 1],
        item[start + 2],
        item[start + 3],
    ])
}

fn formula_id(item: &[u8]) -> u32 {
    u32::from_be_bytes([item[0], item[1], item[2], item[3]])
}

fn substitution_source_formula_id(item: &[u8]) -> u32 {
    u32::from_be_bytes([item[4], item[5], item[6], item[7]])
}

fn substitution_provenance_id(item: &[u8]) -> u32 {
    u32::from_be_bytes([item[8], item[9], item[10], item[11]])
}

fn goal_provenance_id(item: &[u8]) -> u32 {
    let start = 1 + fingerprint_item_len(&item[1..]);
    u32::from_be_bytes([
        item[start],
        item[start + 1],
        item[start + 2],
        item[start + 3],
    ])
}

fn fingerprint_len(item: &[u8]) -> usize {
    u32::from_be_bytes([item[6], item[7], item[8], item[9]]) as usize
}

fn fingerprint_item_len(bytes: &[u8]) -> usize {
    5 + u32::from_be_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]) as usize
}

fn put_formula(formula: &Formula, bytes: &mut Vec<u8>) {
    match formula {
        Formula::Atom(atom) => {
            bytes.push(1);
            put_atom(atom, bytes);
        }
        Formula::Not(child) => {
            bytes.push(2);
            put_formula(child, bytes);
        }
        Formula::And(children) => {
            bytes.push(3);
            put_u32(children.len() as u32, bytes);
            for child in children {
                put_formula(child, bytes);
            }
        }
        Formula::Or(children) => {
            bytes.push(4);
            put_u32(children.len() as u32, bytes);
            for child in children {
                put_formula(child, bytes);
            }
        }
    }
}

fn put_atom(atom: &Atom, bytes: &mut Vec<u8>) {
    bytes.push(symbol_kind_tag(atom.symbol.kind));
    put_u32(atom.symbol.id.0, bytes);
    put_u32(atom.arity, bytes);
    put_u32(atom.arguments.len() as u32, bytes);
    for argument in &atom.arguments {
        put_term(argument, bytes);
    }
}

fn put_term(term: &Term, bytes: &mut Vec<u8>) {
    match term {
        Term::Variable(variable) => {
            bytes.push(1);
            put_u32(variable.0, bytes);
        }
        Term::Application { symbol, arguments } => {
            bytes.push(2);
            bytes.push(symbol_kind_tag(symbol.kind));
            put_u32(symbol.id.0, bytes);
            put_u32(arguments.len() as u32, bytes);
            for argument in arguments {
                put_term(argument, bytes);
            }
        }
        Term::BinderNormalized { binder_id, body } => {
            bytes.push(3);
            put_u32(*binder_id, bytes);
            put_term(body, bytes);
        }
        Term::Malformed => bytes.push(255),
    }
}

fn put_term_path(path: &TermPath, bytes: &mut Vec<u8>) {
    put_u32(path.segments.len() as u32, bytes);
    for segment in &path.segments {
        bytes.push(segment.edge_kind);
        put_u32(segment.child_index, bytes);
    }
}

fn binder_context(
    frames: Vec<(u32, u32, u32, u8)>,
    free_variables: Vec<u32>,
    schematic_variables: Vec<u32>,
) -> Vec<u8> {
    let mut bytes = Vec::new();
    put_u16(1, &mut bytes);
    put_u32(frames.len() as u32, &mut bytes);
    for (binder_id, canonical_index, variable_id, binder_role) in frames {
        put_u32(binder_id, &mut bytes);
        put_u32(canonical_index, &mut bytes);
        put_u32(variable_id, &mut bytes);
        bytes.push(binder_role);
    }
    put_u32(free_variables.len() as u32, &mut bytes);
    for variable in free_variables {
        put_u32(variable, &mut bytes);
    }
    put_u32(schematic_variables.len() as u32, &mut bytes);
    for variable in schematic_variables {
        put_u32(variable, &mut bytes);
    }
    bytes
}

fn empty_binder_context() -> Vec<u8> {
    binder_context(Vec::new(), Vec::new(), Vec::new())
}

fn target() -> Fingerprint {
    Fingerprint::new(9, b"target".to_vec())
}

fn profile() -> KernelProfileRecord {
    KernelProfileRecord::v1(7, ClauseTautologyPolicy::Reject)
}

fn put_profile(bytes: &mut Vec<u8>) {
    let profile = profile();
    put_u16(profile.profile_id, bytes);
    put_u16(profile.clause_schema_version, bytes);
    put_u16(profile.clause_encoding_version, bytes);
    bytes.push(profile.clause_tautology_policy.tag());
    bytes.push(CertificateHashInputAlgorithm::CanonicalEnvelopeV1.tag());
}

fn put_fingerprint(fingerprint: &Fingerprint, bytes: &mut Vec<u8>) {
    bytes.push(fingerprint.algorithm_id);
    put_bytes(&fingerprint.digest, bytes);
}

fn put_bytes(payload: &[u8], bytes: &mut Vec<u8>) {
    put_u32(payload.len() as u32, bytes);
    bytes.extend_from_slice(payload);
}

fn put_u16(value: u16, bytes: &mut Vec<u8>) {
    bytes.extend_from_slice(&value.to_be_bytes());
}

fn put_u32(value: u32, bytes: &mut Vec<u8>) {
    bytes.extend_from_slice(&value.to_be_bytes());
}

fn symbol_kind_tag(kind: SymbolKind) -> u8 {
    match kind {
        SymbolKind::Predicate => 1,
        SymbolKind::FunctorPredicate => 2,
        SymbolKind::Equality => 3,
        SymbolKind::BuiltinRelation => 4,
    }
}

struct CanonicalReader<'a> {
    bytes: &'a [u8],
    cursor: usize,
}

impl<'a> CanonicalReader<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, cursor: 0 }
    }

    fn expect_bytes(&mut self, expected: &[u8]) {
        let actual = self.read_exact(expected.len());
        assert_eq!(actual, expected);
    }

    fn read_u8(&mut self) -> u8 {
        self.read_exact(1)[0]
    }

    fn read_u16(&mut self) -> u16 {
        let bytes = self.read_exact(2);
        u16::from_be_bytes([bytes[0], bytes[1]])
    }

    fn read_u32(&mut self) -> u32 {
        let bytes = self.read_exact(4);
        u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
    }

    fn read_option_u32(&mut self) -> Option<u32> {
        let tag = self.read_u8();
        let value = self.read_u32();
        match tag {
            0 => {
                assert_eq!(value, 0);
                None
            }
            1 => Some(value),
            _ => panic!("bad option tag {tag}"),
        }
    }

    fn read_fingerprint(&mut self) -> Fingerprint {
        let algorithm_id = self.read_u8();
        let digest = self.read_bytes();
        Fingerprint::new(algorithm_id, digest)
    }

    fn skip_atom_manifest(&mut self) {
        let count = self.read_u32();
        for _ in 0..count {
            self.read_u32();
            self.read_bytes();
        }
    }

    fn skip_assertion(&mut self) {
        self.read_u8();
        self.read_u8();
        self.read_option_u32();
        self.read_option_u32();
        self.read_fingerprint();
        self.read_bytes();
    }

    fn skip_clauses(&mut self) {
        let clause_count = self.read_u32();
        for _ in 0..clause_count {
            let literal_count = self.read_u32();
            for _ in 0..literal_count {
                self.read_u32();
                self.read_u8();
            }
        }
    }

    fn read_bytes(&mut self) -> Vec<u8> {
        let len = self.read_u32() as usize;
        self.read_exact(len).to_vec()
    }

    fn read_exact(&mut self, len: usize) -> &'a [u8] {
        let end = self.cursor.checked_add(len).expect("test read length fits");
        let bytes = self
            .bytes
            .get(self.cursor..end)
            .expect("canonical bytes contain expected field");
        self.cursor = end;
        bytes
    }

    fn finish(&self) {
        assert_eq!(
            self.cursor,
            self.bytes.len(),
            "canonical reader must consume every byte"
        );
    }
}
