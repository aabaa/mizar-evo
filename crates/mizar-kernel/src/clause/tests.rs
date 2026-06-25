use super::*;

#[test]
fn normalizes_valid_clauses_with_deterministic_order() {
    let context = sample_context(TautologyPolicy::Reject);
    let predicate_two = SymbolKey::new(SymbolKind::Predicate, 2);
    let predicate_one = SymbolKey::new(SymbolKind::Predicate, 1);
    let equality_one = SymbolKey::new(SymbolKind::Equality, 1);
    let raw = vec![
        lit(
            Polarity::Positive,
            predicate_two,
            vec![Term::Variable(VariableId(2))],
        ),
        lit(Polarity::Negative, equality_one, vec![]),
        lit(
            Polarity::Positive,
            predicate_one,
            vec![Term::Variable(VariableId(1))],
        ),
    ];

    let clause = Clause::normalize(raw, &context).expect("valid clause");

    assert_eq!(clause.form(), ClauseForm::Ordinary);
    assert_eq!(
        clause
            .literals()
            .iter()
            .map(Literal::render)
            .collect::<Vec<_>>(),
        [
            "-Equality#1[arity=0]()",
            "+Predicate#1[arity=1](v1)",
            "+Predicate#2[arity=1](v2)",
        ]
    );
}

#[test]
fn normalizes_valid_single_literal_clause() {
    let context = sample_context(TautologyPolicy::Reject);
    let literal = lit(
        Polarity::Positive,
        SymbolKey::new(SymbolKind::Predicate, 1),
        vec![Term::Variable(VariableId(1))],
    );

    let clause = Clause::normalize(vec![literal.clone()], &context).expect("valid clause");

    assert_eq!(clause.form(), ClauseForm::Ordinary);
    assert_eq!(clause.literals(), std::slice::from_ref(&literal));
    assert!(contains_subsequence(
        &hash_input(&clause),
        &literal.canonical_bytes().expect("valid literal bytes")
    ));
}

#[test]
fn symbol_kind_precedence_is_ordered_before_symbol_id() {
    let context = sample_context(TautologyPolicy::Reject);
    let predicate_high = SymbolKey::new(SymbolKind::Predicate, 9);
    let equality_low = SymbolKey::new(SymbolKind::Equality, 1);

    let clause = Clause::normalize(
        vec![
            lit(Polarity::Positive, equality_low, vec![]),
            lit(Polarity::Positive, predicate_high, vec![]),
        ],
        &context,
    )
    .expect("valid clause");

    assert_eq!(
        clause
            .literals()
            .iter()
            .map(Literal::render)
            .collect::<Vec<_>>(),
        ["+Predicate#9[arity=0]()", "+Equality#1[arity=0]()",]
    );
}

#[test]
fn orders_literals_by_symbol_id_arity_and_argument_encoding() {
    let context = sample_context(TautologyPolicy::Reject);
    let predicate_one = SymbolKey::new(SymbolKind::Predicate, 1);
    let predicate_two = SymbolKey::new(SymbolKind::Predicate, 2);

    let clause = Clause::normalize(
        vec![
            lit(
                Polarity::Positive,
                predicate_two,
                vec![Term::Variable(VariableId(1))],
            ),
            lit(
                Polarity::Positive,
                predicate_one,
                vec![Term::Variable(VariableId(2))],
            ),
            lit(Polarity::Positive, predicate_one, vec![]),
            lit(
                Polarity::Positive,
                predicate_one,
                vec![Term::Variable(VariableId(1))],
            ),
        ],
        &context,
    )
    .expect("valid clause");

    assert_eq!(
        clause
            .literals()
            .iter()
            .map(Literal::render)
            .collect::<Vec<_>>(),
        [
            "+Predicate#1[arity=0]()",
            "+Predicate#1[arity=1](v1)",
            "+Predicate#1[arity=1](v2)",
            "+Predicate#2[arity=1](v1)",
        ]
    );
}

#[test]
fn term_order_matches_canonical_byte_order() {
    let symbol = SymbolKey::new(SymbolKind::Predicate, 1);
    let empty_application = Term::Application {
        symbol,
        arguments: Vec::new(),
    };
    let variable_application = Term::Application {
        symbol,
        arguments: vec![Term::Variable(VariableId(1))],
    };

    assert_eq!(
        empty_application.cmp(&variable_application),
        empty_application
            .canonical_bytes()
            .expect("empty application bytes")
            .cmp(
                &variable_application
                    .canonical_bytes()
                    .expect("variable application bytes")
            )
    );
}

#[test]
fn removes_duplicate_literals_during_normalization() {
    let context = sample_context(TautologyPolicy::Reject);
    let symbol = SymbolKey::new(SymbolKind::Predicate, 1);
    let literal = lit(
        Polarity::Positive,
        symbol,
        vec![Term::Variable(VariableId(1))],
    );

    let clause =
        Clause::normalize(vec![literal.clone(), literal], &context).expect("deduped clause");

    assert_eq!(clause.literals().len(), 1);
}

#[test]
fn empty_clause_is_distinct_from_tautology() {
    let context = sample_context(TautologyPolicy::Marker);

    let clause = Clause::normalize(Vec::new(), &context).expect("empty clause");

    assert_eq!(clause.form(), ClauseForm::Empty);
    assert!(clause.literals().is_empty());
    assert!(clause.render().contains("form=Empty"));
    assert_eq!(hash_input(&clause), hash_input(&clause));
}

#[test]
fn rejects_or_marks_tautologies_according_to_profile() {
    let symbol = SymbolKey::new(SymbolKind::Predicate, 1);
    let positive = lit(
        Polarity::Positive,
        symbol,
        vec![Term::Variable(VariableId(1))],
    );
    let negative = lit(
        Polarity::Negative,
        symbol,
        vec![Term::Variable(VariableId(1))],
    );

    assert_eq!(
        Clause::normalize(
            vec![positive.clone(), negative.clone()],
            &sample_context(TautologyPolicy::Reject),
        ),
        Err(ClauseError::DisallowedTautology)
    );

    let marked = Clause::normalize(
        vec![
            positive,
            negative,
            lit(
                Polarity::Positive,
                SymbolKey::new(SymbolKind::Predicate, 2),
                vec![],
            ),
        ],
        &sample_context(TautologyPolicy::Marker),
    )
    .expect("tautology marker");
    assert_eq!(marked.form(), ClauseForm::Tautology);
    assert!(marked.literals().is_empty());
    assert!(marked.render().contains("literals=0"));
}

#[test]
fn rejects_malformed_atoms_terms_symbols_and_variables() {
    let context = sample_context(TautologyPolicy::Reject);
    let symbol = SymbolKey::new(SymbolKind::Predicate, 1);

    assert_eq!(
        Clause::normalize(
            vec![Literal::new(
                Polarity::Positive,
                Atom::with_arity(symbol, 2, vec![Term::Variable(VariableId(1))]),
            )],
            &context,
        ),
        Err(ClauseError::ArityMismatch {
            symbol,
            expected: 2,
            actual: 1,
        })
    );
    assert_eq!(
        Clause::normalize(
            vec![lit(Polarity::Positive, symbol, vec![Term::Malformed])],
            &context,
        ),
        Err(ClauseError::MalformedTermEncoding)
    );
    assert_eq!(
        Clause::normalize(
            vec![lit(
                Polarity::Positive,
                SymbolKey::new(SymbolKind::Predicate, 77),
                vec![],
            )],
            &context,
        ),
        Err(ClauseError::MissingSymbol(SymbolKey::new(
            SymbolKind::Predicate,
            77,
        )))
    );
    assert_eq!(
        Clause::normalize(
            vec![lit(
                Polarity::Positive,
                SymbolKey::new(SymbolKind::BuiltinRelation, 1),
                vec![],
            )],
            &context,
        ),
        Err(ClauseError::UnsupportedSymbolKind(
            SymbolKind::BuiltinRelation,
        ))
    );
    assert_eq!(
        Clause::normalize(
            vec![lit(
                Polarity::Positive,
                symbol,
                vec![Term::Variable(VariableId(99))],
            )],
            &context,
        ),
        Err(ClauseError::NoncanonicalVariableId(VariableId(99)))
    );
}

#[test]
fn enforces_profile_payload_and_resource_bounds() {
    let context = sample_context(TautologyPolicy::Reject).with_limits(1, 16);
    let symbol = SymbolKey::new(SymbolKind::Predicate, 1);
    let literal = lit(Polarity::Positive, symbol, vec![]);

    assert_eq!(
        Clause::from_canonical_parts(ClauseForm::Ordinary, Vec::new(), &context),
        Err(ClauseError::OrdinaryEmptyPayload)
    );
    assert_eq!(
        Clause::from_canonical_parts(ClauseForm::Empty, vec![literal.clone()], &context),
        Err(ClauseError::NonEmptyMarkerPayload {
            form: ClauseForm::Empty,
        })
    );
    assert_eq!(
        Clause::from_canonical_parts(ClauseForm::Tautology, Vec::new(), &context),
        Err(ClauseError::DisallowedTautology)
    );
    assert_eq!(
        Clause::from_canonical_parts(
            ClauseForm::Tautology,
            vec![literal.clone()],
            &sample_context(TautologyPolicy::Marker),
        ),
        Err(ClauseError::NonEmptyMarkerPayload {
            form: ClauseForm::Tautology,
        })
    );
    assert_eq!(
        Clause::from_canonical_parts(
            ClauseForm::Ordinary,
            vec![literal.clone(), literal.clone()],
            &sample_context(TautologyPolicy::Reject),
        ),
        Err(ClauseError::DuplicateLiteral)
    );
    assert_eq!(
        Clause::normalize(
            vec![
                literal.clone(),
                lit(
                    Polarity::Positive,
                    SymbolKey::new(SymbolKind::Predicate, 2),
                    vec![]
                ),
            ],
            &context,
        ),
        Err(ClauseError::LiteralCountExceeded { max: 1, actual: 2 })
    );
    assert_eq!(
        Clause::normalize(
            vec![lit(
                Polarity::Positive,
                symbol,
                vec![Term::Application {
                    symbol,
                    arguments: vec![Term::Variable(VariableId(1)), Term::Variable(VariableId(2)),],
                }],
            )],
            &context,
        ),
        Err(ClauseError::TermSizeExceeded {
            max: 16,
            actual: 35
        })
    );
}

#[test]
fn rejects_noncanonical_literal_order_in_canonical_constructor() {
    let context = sample_context(TautologyPolicy::Reject);
    let first = lit(
        Polarity::Positive,
        SymbolKey::new(SymbolKind::Predicate, 2),
        vec![],
    );
    let second = lit(
        Polarity::Positive,
        SymbolKey::new(SymbolKind::Predicate, 1),
        vec![],
    );

    assert_eq!(
        Clause::from_canonical_parts(ClauseForm::Ordinary, vec![first, second], &context),
        Err(ClauseError::NonCanonicalLiteralOrder)
    );
}

#[test]
fn rendering_and_hash_inputs_are_stable_and_exclude_display_data() {
    let context = sample_context(TautologyPolicy::Reject);
    let symbol_one = SymbolKey::new(SymbolKind::Predicate, 1);
    let symbol_two = SymbolKey::new(SymbolKind::Predicate, 2);
    let raw_a = vec![
        lit(
            Polarity::Positive,
            symbol_two,
            vec![Term::Variable(VariableId(2))],
        ),
        lit(
            Polarity::Negative,
            symbol_one,
            vec![Term::Variable(VariableId(1))],
        ),
    ];
    let raw_b = vec![raw_a[1].clone(), raw_a[0].clone()];

    let clause_a = Clause::normalize(raw_a, &context).expect("valid clause");
    let clause_b = Clause::normalize(raw_b.clone(), &context).expect("valid clause");

    assert_eq!(clause_a.render(), clause_b.render());
    let hash_a = hash_input(&clause_a);
    assert_eq!(hash_a, hash_input(&clause_b));
    assert!(hash_a.starts_with(CLAUSE_DOMAIN_SEPARATOR));
    assert!(
        contains_subsequence(&hash_a, &1_u16.to_be_bytes()),
        "schema/profile version participates in hash input"
    );
    assert!(
        hash_a.contains(&ClauseForm::Ordinary.tag()),
        "clause form participates in hash input"
    );
    assert_ne!(
        hash_a,
        hash_input(
            &Clause::normalize(
                vec![
                    lit(
                        Polarity::Positive,
                        symbol_two,
                        vec![Term::Variable(VariableId(2))]
                    ),
                    lit(
                        Polarity::Negative,
                        symbol_one,
                        vec![Term::Variable(VariableId(2))]
                    ),
                ],
                &context,
            )
            .expect("different canonical literal bytes")
        ),
        "canonical literal bytes participate in hash input"
    );
    assert_ne!(
        hash_a,
        hash_input(
            &Clause::normalize(raw_b.clone(), &sample_context(TautologyPolicy::Marker))
                .expect("tautology policy variant")
        ),
        "tautology policy participates in hash input"
    );
    assert_ne!(
        hash_a,
        hash_input(
            &Clause::normalize(
                raw_b,
                &sample_context_with_profile(ClauseProfile::new(2, 3, TautologyPolicy::Reject)),
            )
            .expect("profile version variant")
        ),
        "schema and encoding versions participate in hash input"
    );

    let marker_context = sample_context(TautologyPolicy::Marker);
    let empty = Clause::normalize(Vec::new(), &marker_context).expect("empty clause");
    let tautology = Clause::normalize(
        vec![
            lit(
                Polarity::Positive,
                symbol_one,
                vec![Term::Variable(VariableId(1))],
            ),
            lit(
                Polarity::Negative,
                symbol_one,
                vec![Term::Variable(VariableId(1))],
            ),
        ],
        &marker_context,
    )
    .expect("tautology marker");
    assert_ne!(
        hash_input(&empty),
        hash_input(&tautology),
        "empty and tautology forms have distinct hash inputs"
    );

    for excluded in [
        "display-name",
        "source-range",
        "/tmp/source.miz",
        "2026-06-25T00:00:00Z",
        "backend-log",
        "allocation-order",
        "worker-completion-order",
    ] {
        assert!(
            !contains_subsequence(&hash_a, excluded.as_bytes()),
            "{excluded} must not affect canonical bytes"
        );
    }
}

fn sample_context(policy: TautologyPolicy) -> ClauseValidationContext {
    sample_context_with_profile(ClauseProfile::new(1, 1, policy))
}

fn sample_context_with_profile(profile: ClauseProfile) -> ClauseValidationContext {
    let mut context = ClauseValidationContext::new(profile).with_limits(8, 256);
    for kind in [
        SymbolKind::Predicate,
        SymbolKind::FunctorPredicate,
        SymbolKind::Equality,
    ] {
        context = context.with_allowed_symbol_kind(kind);
    }
    for symbol in [
        SymbolKey::new(SymbolKind::Predicate, 1),
        SymbolKey::new(SymbolKind::Predicate, 2),
        SymbolKey::new(SymbolKind::Predicate, 9),
        SymbolKey::new(SymbolKind::FunctorPredicate, 1),
        SymbolKey::new(SymbolKind::Equality, 1),
    ] {
        context = context.with_known_symbol(symbol);
    }
    for variable in [VariableId(1), VariableId(2), VariableId(3)] {
        context = context.with_canonical_variable(variable);
    }
    context
}

fn lit(polarity: Polarity, symbol: SymbolKey, arguments: Vec<Term>) -> Literal {
    Literal::new(polarity, Atom::new(symbol, arguments))
}

fn hash_input(clause: &Clause) -> Vec<u8> {
    clause
        .canonical_hash_input()
        .expect("valid clause canonical hash input")
}

fn contains_subsequence(bytes: &[u8], needle: &[u8]) -> bool {
    bytes.windows(needle.len()).any(|window| window == needle)
}
