use super::*;

#[test]
fn constructs_minimal_problem_and_renders_unsat_contract() {
    let problem = minimal_problem();

    assert_eq!(problem.vc_id(), VcId::new(7));
    assert_eq!(problem.expected_result(), ExpectedBackendResult::Unsat);
    assert_eq!(problem.declarations().len(), 2);
    assert_eq!(problem.axioms().len(), 1);
    assert!(problem.debug_text().contains("expected-result: Unsat"));
    assert!(problem.debug_text().contains("[diagnostics:non-semantic]"));
}

#[test]
fn deterministic_identity_sorts_shuffled_inputs_and_excludes_diagnostics() {
    let first = populated_problem(false);
    let second = populated_problem(true);
    let different_diagnostic =
        AtpProblem::try_new(populated_parts_with_diagnostic(false, "z-note"))
            .expect("diagnostic variant");

    assert_eq!(first.problem_id(), second.problem_id());
    assert_eq!(first.debug_text(), second.debug_text());
    assert_eq!(first.problem_id(), different_diagnostic.problem_id());
    assert_ne!(first.debug_text(), different_diagnostic.debug_text());
    assert_eq!(
        first
            .declarations()
            .iter()
            .map(AtpDeclaration::id)
            .collect::<Vec<_>>(),
        vec![AtpDeclarationId::new(1), AtpDeclarationId::new(2)]
    );
    assert_eq!(
        first
            .symbol_map()
            .iter()
            .map(|entry| entry.backend_symbol().as_str())
            .collect::<Vec<_>>(),
        vec!["P", "a", "guard1", "x"]
    );
    assert_eq!(
        first
            .axioms()
            .iter()
            .map(AtpFormula::id)
            .collect::<Vec<_>>(),
        vec![AtpFormulaId::new(1), AtpFormulaId::new(3)]
    );
    assert_eq!(
        first
            .type_context()
            .guards()
            .iter()
            .map(AtpTypeGuard::id)
            .collect::<Vec<_>>(),
        vec![AtpTypeGuardId::new(1), AtpTypeGuardId::new(2)]
    );
    assert_eq!(
        first
            .properties()
            .iter()
            .map(EncodedProperty::id)
            .collect::<Vec<_>>(),
        vec![AtpPropertyId::new(1), AtpPropertyId::new(2)]
    );
    assert!(first.debug_text().contains("diagnostics:non-semantic"));
}

#[test]
fn rejects_missing_required_inputs_fail_closed() {
    let mut parts = minimal_parts();
    parts.declarations[0] = AtpDeclaration::new(
        AtpDeclarationId::new(1),
        AtpDeclarationKind::Predicate,
        "P",
        1,
        AtpProvenanceId::new(99),
    );
    assert_eq!(
        AtpProblem::try_new(parts).unwrap_err(),
        AtpProblemError::MissingProvenance {
            owner: "declaration",
            provenance_id: AtpProvenanceId::new(99)
        }
    );

    assert_eq!(
        AtpFingerprint::new(18, Vec::new()).unwrap_err(),
        AtpProblemError::EmptyFingerprint { algorithm_id: 18 }
    );
    assert_eq!(
        AtpTargetBinding::new(
            AtpFingerprint::new(18, b"target".to_vec()).expect("fingerprint"),
            ""
        )
        .unwrap_err(),
        AtpProblemError::EmptyField {
            field: "target_binding.producer_binding"
        }
    );

    let mut parts = minimal_parts();
    parts.symbol_map.clear();
    assert_eq!(
        AtpProblem::try_new(parts).unwrap_err(),
        AtpProblemError::MissingSymbolMap {
            symbol: AtpSymbolName::new("P")
        }
    );

    let mut parts = minimal_parts();
    parts.logic_profile = profile_with_soft_types(SoftTypeStrategy::GuardPredicates);
    assert_eq!(
        AtpProblem::try_new(parts).unwrap_err(),
        AtpProblemError::MissingTypeContextBinding {
            strategy: SoftTypeStrategy::GuardPredicates
        }
    );
}

#[test]
fn symbol_references_fail_closed_for_each_problem_section() {
    let mut parts = minimal_parts();
    parts.axioms[0] = AtpFormula::new(
        AtpFormulaId::new(1),
        atom("Q", vec![constant("a")]),
        AtpProvenanceId::new(2),
    );
    assert_eq!(
        AtpProblem::try_new(parts).unwrap_err(),
        AtpProblemError::MissingSymbolMap {
            symbol: AtpSymbolName::new("Q")
        }
    );

    let mut parts = minimal_parts();
    parts.symbol_map.push(AtpSymbolMapEntry::new(
        "Q",
        AtpSymbolSource::MizarSymbol(AtpSourceBinding::new("pred:Q")),
    ));
    parts.axioms[0] = AtpFormula::new(
        AtpFormulaId::new(1),
        atom("Q", vec![constant("a")]),
        AtpProvenanceId::new(2),
    );
    assert_eq!(
        AtpProblem::try_new(parts).unwrap_err(),
        AtpProblemError::MissingDeclarationSymbol {
            symbol: AtpSymbolName::new("Q")
        }
    );

    let mut parts = minimal_parts();
    parts
        .symbol_map
        .retain(|entry| entry.backend_symbol().as_str() != "a");
    assert_eq!(
        AtpProblem::try_new(parts).unwrap_err(),
        AtpProblemError::MissingSymbolMap {
            symbol: AtpSymbolName::new("a")
        }
    );

    let mut parts = minimal_parts();
    parts.axioms[0] = AtpFormula::new(
        AtpFormulaId::new(1),
        atom(
            "P",
            vec![AtpTerm::Function {
                function: AtpSymbolName::new("f"),
                arguments: vec![constant("a")],
            }],
        ),
        AtpProvenanceId::new(2),
    );
    assert_eq!(
        AtpProblem::try_new(parts).unwrap_err(),
        AtpProblemError::MissingSymbolMap {
            symbol: AtpSymbolName::new("f")
        }
    );

    let mut parts = minimal_parts();
    parts.symbol_map.push(AtpSymbolMapEntry::new(
        "f",
        AtpSymbolSource::MizarSymbol(AtpSourceBinding::new("fun:f")),
    ));
    parts.axioms[0] = AtpFormula::new(
        AtpFormulaId::new(1),
        atom(
            "P",
            vec![AtpTerm::Function {
                function: AtpSymbolName::new("f"),
                arguments: vec![constant("a")],
            }],
        ),
        AtpProvenanceId::new(2),
    );
    assert_eq!(
        AtpProblem::try_new(parts).unwrap_err(),
        AtpProblemError::MissingDeclarationSymbol {
            symbol: AtpSymbolName::new("f")
        }
    );

    let mut parts = minimal_parts();
    parts.logic_profile = profile_first_order();
    parts.axioms[0] = AtpFormula::new(
        AtpFormulaId::new(1),
        AtpFormulaTree::Forall {
            binders: vec![AtpBinder::new("x", None)],
            body: Box::new(AtpFormulaTree::True),
        },
        AtpProvenanceId::new(2),
    );
    assert_eq!(
        AtpProblem::try_new(parts).unwrap_err(),
        AtpProblemError::MissingSymbolMap {
            symbol: AtpSymbolName::new("x")
        }
    );

    let mut parts = minimal_parts();
    parts.logic_profile = profile_first_order();
    parts.axioms[0] = AtpFormula::new(
        AtpFormulaId::new(1),
        AtpFormulaTree::Forall {
            binders: vec![AtpBinder::new("x", None)],
            body: Box::new(AtpFormulaTree::True),
        },
        AtpProvenanceId::new(2),
    );
    parts.symbol_map.push(AtpSymbolMapEntry::new(
        "x",
        AtpSymbolSource::GeneratedBinder(AtpSourceBinding::new("binder:x")),
    ));
    assert_eq!(
        AtpProblem::try_new(parts).unwrap_err(),
        AtpProblemError::MissingDeclarationSymbol {
            symbol: AtpSymbolName::new("x")
        }
    );

    let mut parts = minimal_parts();
    parts.logic_profile = profile_first_order();
    parts.axioms[0] = AtpFormula::new(
        AtpFormulaId::new(1),
        AtpFormulaTree::Forall {
            binders: vec![AtpBinder::new("x", Some(AtpSymbolName::new("S")))],
            body: Box::new(AtpFormulaTree::True),
        },
        AtpProvenanceId::new(2),
    );
    parts.symbol_map.push(AtpSymbolMapEntry::new(
        "x",
        AtpSymbolSource::GeneratedBinder(AtpSourceBinding::new("binder:x")),
    ));
    parts.declarations.push(AtpDeclaration::new(
        AtpDeclarationId::new(3),
        AtpDeclarationKind::GeneratedBinder,
        "x",
        0,
        AtpProvenanceId::new(2),
    ));
    assert_eq!(
        AtpProblem::try_new(parts).unwrap_err(),
        AtpProblemError::MissingSymbolMap {
            symbol: AtpSymbolName::new("S")
        }
    );

    let mut parts = minimal_parts();
    parts.logic_profile = profile_first_order();
    parts.axioms[0] = AtpFormula::new(
        AtpFormulaId::new(1),
        AtpFormulaTree::Forall {
            binders: vec![AtpBinder::new("x", Some(AtpSymbolName::new("S")))],
            body: Box::new(AtpFormulaTree::True),
        },
        AtpProvenanceId::new(2),
    );
    parts.symbol_map.push(AtpSymbolMapEntry::new(
        "x",
        AtpSymbolSource::GeneratedBinder(AtpSourceBinding::new("binder:x")),
    ));
    parts.declarations.push(AtpDeclaration::new(
        AtpDeclarationId::new(3),
        AtpDeclarationKind::GeneratedBinder,
        "x",
        0,
        AtpProvenanceId::new(2),
    ));
    parts.symbol_map.push(AtpSymbolMapEntry::new(
        "S",
        AtpSymbolSource::MizarSymbol(AtpSourceBinding::new("sort:S")),
    ));
    assert_eq!(
        AtpProblem::try_new(parts).unwrap_err(),
        AtpProblemError::MissingDeclarationSymbol {
            symbol: AtpSymbolName::new("S")
        }
    );

    let mut parts = minimal_parts();
    parts.properties.push(EncodedProperty::axiom(
        AtpPropertyId::new(1),
        "property-target",
        AtpFormulaTree::True,
        AtpProvenanceId::new(2),
    ));
    parts.symbol_map.push(AtpSymbolMapEntry::new(
        "property-target",
        AtpSymbolSource::MizarSymbol(AtpSourceBinding::new("property-target")),
    ));
    assert_eq!(
        AtpProblem::try_new(parts).unwrap_err(),
        AtpProblemError::MissingDeclarationSymbol {
            symbol: AtpSymbolName::new("property-target")
        }
    );

    let mut parts = minimal_parts();
    parts.properties.push(EncodedProperty::axiom(
        AtpPropertyId::new(1),
        "missing-property-target",
        AtpFormulaTree::True,
        AtpProvenanceId::new(2),
    ));
    assert_eq!(
        AtpProblem::try_new(parts).unwrap_err(),
        AtpProblemError::MissingSymbolMap {
            symbol: AtpSymbolName::new("missing-property-target")
        }
    );
}

#[test]
fn every_problem_formula_source_requires_existing_provenance() {
    let mut parts = minimal_parts();
    parts.axioms[0] = AtpFormula::new(
        AtpFormulaId::new(1),
        atom("P", vec![constant("a")]),
        AtpProvenanceId::new(99),
    );
    assert_eq!(
        AtpProblem::try_new(parts).unwrap_err(),
        AtpProblemError::MissingProvenance {
            owner: "axiom",
            provenance_id: AtpProvenanceId::new(99)
        }
    );

    let mut parts = minimal_parts();
    parts.conjecture = AtpFormula::new(
        AtpFormulaId::new(2),
        atom("P", vec![constant("a")]),
        AtpProvenanceId::new(99),
    );
    assert_eq!(
        AtpProblem::try_new(parts).unwrap_err(),
        AtpProblemError::MissingProvenance {
            owner: "conjecture",
            provenance_id: AtpProvenanceId::new(99)
        }
    );

    let mut parts = populated_parts(false);
    parts.type_context.guards[0] = AtpTypeGuard::new(
        AtpTypeGuardId::new(2),
        atom("P", vec![constant("a")]),
        AtpProvenanceId::new(99),
    );
    assert_eq!(
        AtpProblem::try_new(parts).unwrap_err(),
        AtpProblemError::MissingProvenance {
            owner: "type-guard",
            provenance_id: AtpProvenanceId::new(99)
        }
    );

    let mut parts = populated_parts(false);
    parts.properties[0] = EncodedProperty::axiom(
        AtpPropertyId::new(2),
        "P",
        atom("P", vec![constant("a")]),
        AtpProvenanceId::new(99),
    );
    assert_eq!(
        AtpProblem::try_new(parts).unwrap_err(),
        AtpProblemError::MissingProvenance {
            owner: "property",
            provenance_id: AtpProvenanceId::new(99)
        }
    );
}

#[test]
fn provenance_and_symbol_map_sources_fail_closed_when_incomplete() {
    let mut parts = minimal_parts();
    parts.provenance[0] = AtpProvenance::new(
        AtpProvenanceId::new(1),
        AtpSourceRef::LocalHypothesis(AtpSourceBinding::new("")),
        "payload",
    );
    assert_eq!(
        AtpProblem::try_new(parts).unwrap_err(),
        AtpProblemError::EmptyField {
            field: "provenance.source"
        }
    );

    for (source, field) in [
        (
            AtpSourceRef::ImportedAxiom {
                package: AtpSourceBinding::new(""),
                module: AtpSourceBinding::new("mod"),
                item: AtpSourceBinding::new("ax"),
                statement_fingerprint: AtpFingerprint::new(2, b"stmt".to_vec())
                    .expect("fingerprint"),
                required_status: AtpRequiredProofStatus::new("kernel-checked"),
                context_requirement: AtpSourceBinding::new("ctx"),
            },
            "imported.package",
        ),
        (
            AtpSourceRef::ImportedAxiom {
                package: AtpSourceBinding::new("pkg"),
                module: AtpSourceBinding::new(""),
                item: AtpSourceBinding::new("ax"),
                statement_fingerprint: AtpFingerprint::new(2, b"stmt".to_vec())
                    .expect("fingerprint"),
                required_status: AtpRequiredProofStatus::new("kernel-checked"),
                context_requirement: AtpSourceBinding::new("ctx"),
            },
            "imported.module",
        ),
        (
            AtpSourceRef::ImportedAxiom {
                package: AtpSourceBinding::new("pkg"),
                module: AtpSourceBinding::new("mod"),
                item: AtpSourceBinding::new(""),
                statement_fingerprint: AtpFingerprint::new(2, b"stmt".to_vec())
                    .expect("fingerprint"),
                required_status: AtpRequiredProofStatus::new("kernel-checked"),
                context_requirement: AtpSourceBinding::new("ctx"),
            },
            "imported.item",
        ),
        (
            AtpSourceRef::ImportedAxiom {
                package: AtpSourceBinding::new("pkg"),
                module: AtpSourceBinding::new("mod"),
                item: AtpSourceBinding::new("ax"),
                statement_fingerprint: AtpFingerprint::new(2, b"stmt".to_vec())
                    .expect("fingerprint"),
                required_status: AtpRequiredProofStatus::new(""),
                context_requirement: AtpSourceBinding::new("ctx"),
            },
            "imported.required_status",
        ),
        (
            AtpSourceRef::ImportedAxiom {
                package: AtpSourceBinding::new("pkg"),
                module: AtpSourceBinding::new("mod"),
                item: AtpSourceBinding::new("ax"),
                statement_fingerprint: AtpFingerprint::new(2, b"stmt".to_vec())
                    .expect("fingerprint"),
                required_status: AtpRequiredProofStatus::new("kernel-checked"),
                context_requirement: AtpSourceBinding::new(""),
            },
            "imported.context_requirement",
        ),
    ] {
        let mut parts = minimal_parts();
        parts.provenance[0] = AtpProvenance::new(AtpProvenanceId::new(1), source, "payload");
        assert_eq!(
            AtpProblem::try_new(parts).unwrap_err(),
            AtpProblemError::EmptyField { field }
        );
    }

    let mut parts = minimal_parts();
    parts.provenance[0] = AtpProvenance::new(
        AtpProvenanceId::new(1),
        AtpSourceRef::ImportedTheorem {
            package: AtpSourceBinding::new("pkg"),
            module: AtpSourceBinding::new("mod"),
            item: AtpSourceBinding::new("thm"),
            statement_fingerprint: AtpFingerprint::new(2, b"stmt".to_vec()).expect("fingerprint"),
            required_status: AtpRequiredProofStatus::new("kernel-checked"),
            context_requirement: AtpSourceBinding::new(""),
        },
        "payload",
    );
    assert_eq!(
        AtpProblem::try_new(parts).unwrap_err(),
        AtpProblemError::EmptyField {
            field: "imported.context_requirement"
        }
    );

    let mut parts = minimal_parts();
    parts.symbol_map[0] =
        AtpSymbolMapEntry::new("P", AtpSymbolSource::MizarSymbol(AtpSourceBinding::new("")));
    assert_eq!(
        AtpProblem::try_new(parts).unwrap_err(),
        AtpProblemError::EmptyField {
            field: "symbol_map.source"
        }
    );

    let mut parts = populated_parts(false);
    parts.symbol_map.push(AtpSymbolMapEntry::new(
        "guard-missing",
        AtpSymbolSource::TypeGuard(AtpTypeGuardId::new(99)),
    ));
    assert_eq!(
        AtpProblem::try_new(parts).unwrap_err(),
        AtpProblemError::MissingTypeGuard {
            type_guard: AtpTypeGuardId::new(99)
        }
    );
}

#[test]
fn rejects_missing_formula_payloads_and_duplicate_ids() {
    let mut parts = minimal_parts();
    parts.axioms[0] = AtpFormula::missing(AtpFormulaId::new(1), AtpProvenanceId::new(2));
    assert_eq!(
        AtpProblem::try_new(parts).unwrap_err(),
        AtpProblemError::MissingFormulaPayload {
            formula_id: AtpFormulaId::new(1)
        }
    );

    let mut parts = minimal_parts();
    parts.conjecture = AtpFormula::missing(AtpFormulaId::new(2), AtpProvenanceId::new(3));
    assert_eq!(
        AtpProblem::try_new(parts).unwrap_err(),
        AtpProblemError::MissingFormulaPayload {
            formula_id: AtpFormulaId::new(2)
        }
    );

    let mut parts = minimal_parts();
    parts.axioms.push(AtpFormula::new(
        AtpFormulaId::new(1),
        atom("P", vec![constant("a")]),
        AtpProvenanceId::new(2),
    ));
    assert_eq!(
        AtpProblem::try_new(parts).unwrap_err(),
        AtpProblemError::DuplicateId {
            section: "axioms",
            id: 1
        }
    );

    let mut parts = minimal_parts();
    parts.symbol_map.push(AtpSymbolMapEntry::new(
        "P",
        AtpSymbolSource::MizarSymbol(AtpSourceBinding::new("duplicate")),
    ));
    assert_eq!(
        AtpProblem::try_new(parts).unwrap_err(),
        AtpProblemError::DuplicateSymbolMap {
            symbol: AtpSymbolName::new("P")
        }
    );

    let mut parts = minimal_parts();
    parts.declarations.push(AtpDeclaration::new(
        AtpDeclarationId::new(3),
        AtpDeclarationKind::Predicate,
        "P",
        1,
        AtpProvenanceId::new(1),
    ));
    assert_eq!(
        AtpProblem::try_new(parts).unwrap_err(),
        AtpProblemError::DuplicateDeclarationSymbol {
            symbol: AtpSymbolName::new("P")
        }
    );

    let mut parts = minimal_parts();
    parts.conjecture = AtpFormula::new(
        AtpFormulaId::new(1),
        atom("P", vec![constant("a")]),
        AtpProvenanceId::new(3),
    );
    assert_eq!(
        AtpProblem::try_new(parts).unwrap_err(),
        AtpProblemError::DuplicateId {
            section: "formulas",
            id: 1
        }
    );
}

#[test]
fn symbol_declarations_must_match_formula_kind_and_arity() {
    let mut parts = minimal_parts();
    parts.declarations[0] = AtpDeclaration::new(
        AtpDeclarationId::new(1),
        AtpDeclarationKind::Function,
        "P",
        1,
        AtpProvenanceId::new(1),
    );
    assert_eq!(
        AtpProblem::try_new(parts).unwrap_err(),
        AtpProblemError::InvalidSymbolDeclaration {
            symbol: AtpSymbolName::new("P"),
            expected: "predicate",
            actual: AtpDeclarationKind::Function
        }
    );

    let mut parts = minimal_parts();
    parts.declarations[0] = AtpDeclaration::new(
        AtpDeclarationId::new(1),
        AtpDeclarationKind::Predicate,
        "P",
        0,
        AtpProvenanceId::new(1),
    );
    assert_eq!(
        AtpProblem::try_new(parts).unwrap_err(),
        AtpProblemError::InvalidSymbolArity {
            symbol: AtpSymbolName::new("P"),
            expected: 1,
            actual: 0
        }
    );

    let mut parts = minimal_parts();
    parts.declarations[1] = AtpDeclaration::new(
        AtpDeclarationId::new(2),
        AtpDeclarationKind::Predicate,
        "a",
        0,
        AtpProvenanceId::new(1),
    );
    assert_eq!(
        AtpProblem::try_new(parts).unwrap_err(),
        AtpProblemError::InvalidSymbolDeclaration {
            symbol: AtpSymbolName::new("a"),
            expected: "function",
            actual: AtpDeclarationKind::Predicate
        }
    );

    let mut parts = minimal_parts();
    parts.declarations[1] = AtpDeclaration::new(
        AtpDeclarationId::new(2),
        AtpDeclarationKind::Function,
        "a",
        1,
        AtpProvenanceId::new(1),
    );
    assert_eq!(
        AtpProblem::try_new(parts).unwrap_err(),
        AtpProblemError::InvalidSymbolArity {
            symbol: AtpSymbolName::new("a"),
            expected: 0,
            actual: 1
        }
    );

    let mut parts = minimal_parts();
    parts.logic_profile = profile_first_order();
    parts.axioms[0] = AtpFormula::new(
        AtpFormulaId::new(1),
        AtpFormulaTree::Forall {
            binders: vec![AtpBinder::new("x", Some(AtpSymbolName::new("S")))],
            body: Box::new(atom("P", vec![variable("x")])),
        },
        AtpProvenanceId::new(2),
    );
    parts.symbol_map.push(AtpSymbolMapEntry::new(
        "x",
        AtpSymbolSource::GeneratedBinder(AtpSourceBinding::new("binder:x")),
    ));
    parts.declarations.push(AtpDeclaration::new(
        AtpDeclarationId::new(3),
        AtpDeclarationKind::Predicate,
        "x",
        0,
        AtpProvenanceId::new(2),
    ));
    parts.symbol_map.push(AtpSymbolMapEntry::new(
        "S",
        AtpSymbolSource::MizarSymbol(AtpSourceBinding::new("sort:S")),
    ));
    parts.declarations.push(AtpDeclaration::new(
        AtpDeclarationId::new(4),
        AtpDeclarationKind::Sort,
        "S",
        0,
        AtpProvenanceId::new(2),
    ));
    assert_eq!(
        AtpProblem::try_new(parts).unwrap_err(),
        AtpProblemError::InvalidSymbolDeclaration {
            symbol: AtpSymbolName::new("x"),
            expected: "generated binder",
            actual: AtpDeclarationKind::Predicate
        }
    );

    let mut parts = minimal_parts();
    parts.logic_profile = profile_first_order();
    parts.axioms[0] = AtpFormula::new(
        AtpFormulaId::new(1),
        AtpFormulaTree::Forall {
            binders: vec![AtpBinder::new("x", Some(AtpSymbolName::new("S")))],
            body: Box::new(AtpFormulaTree::True),
        },
        AtpProvenanceId::new(2),
    );
    parts.symbol_map.push(AtpSymbolMapEntry::new(
        "x",
        AtpSymbolSource::GeneratedBinder(AtpSourceBinding::new("binder:x")),
    ));
    parts.declarations.push(AtpDeclaration::new(
        AtpDeclarationId::new(3),
        AtpDeclarationKind::GeneratedBinder,
        "x",
        0,
        AtpProvenanceId::new(2),
    ));
    parts.symbol_map.push(AtpSymbolMapEntry::new(
        "S",
        AtpSymbolSource::MizarSymbol(AtpSourceBinding::new("sort:S")),
    ));
    parts.declarations.push(AtpDeclaration::new(
        AtpDeclarationId::new(4),
        AtpDeclarationKind::Predicate,
        "S",
        0,
        AtpProvenanceId::new(2),
    ));
    assert_eq!(
        AtpProblem::try_new(parts).unwrap_err(),
        AtpProblemError::InvalidSymbolDeclaration {
            symbol: AtpSymbolName::new("S"),
            expected: "sort",
            actual: AtpDeclarationKind::Predicate
        }
    );
}

#[test]
fn unsupported_profile_limitations_are_classified_separately() {
    let mut parts = minimal_parts();
    parts.axioms[0] = AtpFormula::new(
        AtpFormulaId::new(1),
        AtpFormulaTree::Equality {
            left: constant("a"),
            right: constant("a"),
        },
        AtpProvenanceId::new(2),
    );
    parts.logic_profile = profile_without_equality();
    assert_eq!(
        AtpProblem::try_new(parts).unwrap_err(),
        AtpProblemError::UnsupportedProfileFeature {
            feature: "equality"
        }
    );

    let mut parts = minimal_parts();
    parts.axioms[0] = AtpFormula::new(
        AtpFormulaId::new(1),
        AtpFormulaTree::Forall {
            binders: vec![AtpBinder::new("x", None)],
            body: Box::new(atom("P", vec![variable("x")])),
        },
        AtpProvenanceId::new(2),
    );
    parts.symbol_map.push(AtpSymbolMapEntry::new(
        "x",
        AtpSymbolSource::GeneratedBinder(AtpSourceBinding::new("binder:x")),
    ));
    assert_eq!(
        AtpProblem::try_new(parts).unwrap_err(),
        AtpProblemError::UnsupportedProfileFeature {
            feature: "quantifier"
        }
    );
}

#[test]
fn invalid_logic_profile_is_rejected_before_problem_construction() {
    assert_eq!(
        LogicProfile::try_new(
            "",
            LogicFragment::Fof,
            EqualitySupport::Supported,
            QuantifierPolicy::PropositionalOnly,
            SoftTypeStrategy::BackendSorts,
            NativePropertySupport::Unsupported,
            BTreeSet::from([ConcreteFormat::Tptp]),
        )
        .unwrap_err(),
        AtpProblemError::InvalidLogicProfile {
            reason: "empty profile name"
        }
    );
    assert_eq!(
        LogicProfile::try_new(
            "empty-formats",
            LogicFragment::Fof,
            EqualitySupport::Supported,
            QuantifierPolicy::PropositionalOnly,
            SoftTypeStrategy::BackendSorts,
            NativePropertySupport::Unsupported,
            BTreeSet::new(),
        )
        .unwrap_err(),
        AtpProblemError::InvalidLogicProfile {
            reason: "no concrete encoder format"
        }
    );
}

#[test]
fn expected_backend_result_has_only_unsat_variant() {
    let source = include_str!("../problem.rs");
    let start = source
        .find("pub enum ExpectedBackendResult")
        .expect("expected-result enum");
    let tail = &source[start..];
    let end = tail.find("\n}\n").expect("expected-result enum end");
    assert_eq!(
        &tail[..end + 3],
        "pub enum ExpectedBackendResult {\n    Unsat,\n}\n"
    );
}

#[test]
fn canonical_identity_length_prefixes_string_payloads() {
    let mut first = minimal_parts();
    first.provenance[0] = AtpProvenance::new(
        AtpProvenanceId::new(1),
        AtpSourceRef::LocalHypothesis(AtpSourceBinding::new("ctx:1")),
        "alpha\nbeta",
    );
    let first = AtpProblem::try_new(first).expect("newline payload problem");

    let mut second = minimal_parts();
    second.provenance[0] = AtpProvenance::new(
        AtpProvenanceId::new(1),
        AtpSourceRef::LocalHypothesis(AtpSourceBinding::new("ctx:1")),
        "alpha\\nbeta",
    );
    let second = AtpProblem::try_new(second).expect("escaped payload problem");

    assert_ne!(first.problem_id(), second.problem_id());
    assert!(first.debug_text().contains("616c7068610a62657461"));
}

#[test]
fn properties_and_type_guards_require_provenance_and_supported_profiles() {
    let problem = populated_problem(false);
    assert_eq!(problem.properties().len(), 2);
    assert_eq!(problem.type_context().guards().len(), 2);

    let mut parts = populated_parts(false);
    parts.properties[0] = EncodedProperty::native_declaration(
        AtpPropertyId::new(2),
        "P",
        AtpDeclarationId::new(1),
        AtpProvenanceId::new(2),
    );
    assert_eq!(
        AtpProblem::try_new(parts).unwrap_err(),
        AtpProblemError::UnsupportedProfileFeature {
            feature: "native property declaration"
        }
    );

    let mut parts = minimal_parts();
    parts.logic_profile = profile_with_native_properties();
    parts.properties.push(EncodedProperty::native_declaration(
        AtpPropertyId::new(1),
        "P",
        AtpDeclarationId::new(1),
        AtpProvenanceId::new(2),
    ));
    let problem = AtpProblem::try_new(parts).expect("native property declaration");
    assert_eq!(problem.properties().len(), 1);

    let mut parts = minimal_parts();
    parts.logic_profile = profile_with_native_properties();
    parts.properties.push(EncodedProperty::native_declaration(
        AtpPropertyId::new(1),
        "P",
        AtpDeclarationId::new(99),
        AtpProvenanceId::new(2),
    ));
    assert_eq!(
        AtpProblem::try_new(parts).unwrap_err(),
        AtpProblemError::MissingDeclaration {
            declaration: AtpDeclarationId::new(99)
        }
    );
}

#[test]
fn public_problem_rendering_excludes_prohibited_trusted_material() {
    let problem = populated_problem(false);
    let rendered = format!("{:?}\n{}", problem, problem.debug_text());
    for prohibited in [
        "SAT clause",
        "backend log",
        "backend used_axioms",
        "proof method",
        "accepted proof status",
    ] {
        assert!(
            !rendered.contains(prohibited),
            "prohibited trusted material leaked: {prohibited}"
        );
    }
}

fn minimal_problem() -> AtpProblem {
    AtpProblem::try_new(minimal_parts()).expect("minimal ATP problem")
}

fn populated_problem(reverse: bool) -> AtpProblem {
    AtpProblem::try_new(populated_parts(reverse)).expect("populated ATP problem")
}

fn minimal_parts() -> AtpProblemParts {
    let provenance = vec![
        provenance(
            1,
            AtpSourceRef::LocalHypothesis(AtpSourceBinding::new("ctx:1")),
        ),
        provenance(
            2,
            AtpSourceRef::CitedPremise(AtpSourceBinding::new("premise:1")),
        ),
        provenance(
            3,
            AtpSourceRef::GeneratedVcFact(AtpSourceBinding::new("goal:7")),
        ),
    ];
    AtpProblemParts {
        vc_id: VcId::new(7),
        target_binding: target_binding(),
        logic_profile: profile(),
        expected_result: ExpectedBackendResult::Unsat,
        declarations: vec![
            AtpDeclaration::new(
                AtpDeclarationId::new(1),
                AtpDeclarationKind::Predicate,
                "P",
                1,
                AtpProvenanceId::new(1),
            ),
            AtpDeclaration::new(
                AtpDeclarationId::new(2),
                AtpDeclarationKind::Function,
                "a",
                0,
                AtpProvenanceId::new(1),
            ),
        ],
        axioms: vec![AtpFormula::new(
            AtpFormulaId::new(1),
            atom("P", vec![constant("a")]),
            AtpProvenanceId::new(2),
        )],
        conjecture: AtpFormula::new(
            AtpFormulaId::new(2),
            atom("P", vec![constant("a")]),
            AtpProvenanceId::new(3),
        ),
        type_context: AtpTypeContext::new(Vec::new()),
        properties: Vec::new(),
        symbol_map: vec![
            AtpSymbolMapEntry::new(
                "P",
                AtpSymbolSource::MizarSymbol(AtpSourceBinding::new("pred:P")),
            ),
            AtpSymbolMapEntry::new(
                "a",
                AtpSymbolSource::MizarSymbol(AtpSourceBinding::new("const:a")),
            ),
        ],
        provenance,
        diagnostics: vec![AtpDiagnostic::new("note", "fixture diagnostic")],
    }
}

fn populated_parts(reverse: bool) -> AtpProblemParts {
    populated_parts_with_diagnostic(reverse, "a-note")
}

fn populated_parts_with_diagnostic(reverse: bool, diagnostic_key: &str) -> AtpProblemParts {
    let mut parts = minimal_parts();
    parts.logic_profile = profile_with_soft_types(SoftTypeStrategy::GuardPredicates);
    parts.axioms.push(AtpFormula::new(
        AtpFormulaId::new(3),
        atom("P", vec![constant("a")]),
        AtpProvenanceId::new(2),
    ));
    parts.type_context = AtpTypeContext::new(vec![
        AtpTypeGuard::new(
            AtpTypeGuardId::new(2),
            atom("P", vec![constant("a")]),
            AtpProvenanceId::new(1),
        ),
        AtpTypeGuard::new(
            AtpTypeGuardId::new(1),
            atom("P", vec![constant("a")]),
            AtpProvenanceId::new(1),
        ),
    ]);
    parts.properties = vec![
        EncodedProperty::axiom(
            AtpPropertyId::new(2),
            "P",
            atom("P", vec![constant("a")]),
            AtpProvenanceId::new(2),
        ),
        EncodedProperty::axiom(
            AtpPropertyId::new(1),
            "P",
            AtpFormulaTree::Implies(
                Box::new(atom("P", vec![constant("a")])),
                Box::new(atom("P", vec![constant("a")])),
            ),
            AtpProvenanceId::new(2),
        ),
    ];
    parts.symbol_map.push(AtpSymbolMapEntry::new(
        "x",
        AtpSymbolSource::GeneratedBinder(AtpSourceBinding::new("binder:x")),
    ));
    parts.symbol_map.push(AtpSymbolMapEntry::new(
        "guard1",
        AtpSymbolSource::TypeGuard(AtpTypeGuardId::new(1)),
    ));
    parts.diagnostics = vec![AtpDiagnostic::new(
        diagnostic_key,
        "non semantic diagnostic",
    )];
    if reverse {
        parts.declarations.reverse();
        parts.provenance.reverse();
        parts.symbol_map.reverse();
        parts.axioms.reverse();
        parts.properties.reverse();
        parts.type_context.guards.reverse();
    }
    parts
}

fn target_binding() -> AtpTargetBinding {
    AtpTargetBinding::new(
        AtpFingerprint::new(18, b"target-vc-7".to_vec()).expect("fingerprint"),
        AtpSourceBinding::new("vc:7"),
    )
    .expect("target binding")
}

fn profile() -> LogicProfile {
    profile_with_soft_types(SoftTypeStrategy::BackendSorts)
}

fn profile_with_soft_types(soft_types: SoftTypeStrategy) -> LogicProfile {
    LogicProfile::try_new(
        "fof-fixture",
        LogicFragment::Fof,
        EqualitySupport::Supported,
        QuantifierPolicy::PropositionalOnly,
        soft_types,
        NativePropertySupport::Unsupported,
        BTreeSet::from([ConcreteFormat::Tptp]),
    )
    .expect("profile")
}

fn profile_first_order() -> LogicProfile {
    LogicProfile::try_new(
        "fof-first-order",
        LogicFragment::Fof,
        EqualitySupport::Supported,
        QuantifierPolicy::FirstOrder,
        SoftTypeStrategy::BackendSorts,
        NativePropertySupport::Unsupported,
        BTreeSet::from([ConcreteFormat::Tptp]),
    )
    .expect("profile")
}

fn profile_with_native_properties() -> LogicProfile {
    LogicProfile::try_new(
        "fof-native-properties",
        LogicFragment::Fof,
        EqualitySupport::Supported,
        QuantifierPolicy::PropositionalOnly,
        SoftTypeStrategy::BackendSorts,
        NativePropertySupport::Supported,
        BTreeSet::from([ConcreteFormat::Tptp]),
    )
    .expect("profile")
}

fn profile_without_equality() -> LogicProfile {
    LogicProfile::try_new(
        "fof-no-equality",
        LogicFragment::Fof,
        EqualitySupport::Unsupported,
        QuantifierPolicy::PropositionalOnly,
        SoftTypeStrategy::BackendSorts,
        NativePropertySupport::Unsupported,
        BTreeSet::from([ConcreteFormat::Tptp]),
    )
    .expect("profile")
}

fn provenance(id: u32, source: AtpSourceRef) -> AtpProvenance {
    AtpProvenance::new(
        AtpProvenanceId::new(id),
        source,
        format!("provenance-payload-{id}"),
    )
}

fn atom(predicate: &str, arguments: Vec<AtpTerm>) -> AtpFormulaTree {
    AtpFormulaTree::Atom(AtpAtom::new(predicate, arguments))
}

fn variable(name: &str) -> AtpTerm {
    AtpTerm::Variable(AtpSymbolName::new(name))
}

fn constant(name: &str) -> AtpTerm {
    AtpTerm::Function {
        function: AtpSymbolName::new(name),
        arguments: Vec::new(),
    }
}
