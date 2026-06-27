use super::*;
use crate::problem::{
    AtpDiagnostic, AtpFingerprint, AtpFormula, AtpFormulaId, AtpProblem, AtpProblemParts,
    AtpTargetBinding, AtpTypeContext, ConcreteFormat, ExpectedBackendResult, LogicFragment,
    NativePropertySupport, SoftTypeStrategy,
};

#[test]
fn encodes_every_supported_family_as_axiom_form() {
    for fixture in family_fixtures() {
        let bundle =
            encode_properties(input_for(vec![fixture.projection()])).expect("property bundle");

        assert_eq!(bundle.properties().len(), 1);
        assert_eq!(bundle.declarations().len(), fixture.binder_count);
        assert_eq!(bundle.symbol_map().len(), fixture.binder_count);
        assert_eq!(bundle.provenance().len(), fixture.binder_count + 1);
        assert_eq!(bundle.properties()[0].id(), AtpPropertyId::new(10));
        assert_eq!(
            bundle.properties()[0].target_symbol().as_str(),
            fixture.target
        );
        assert_eq!(
            bundle
                .declarations()
                .iter()
                .map(AtpDeclaration::kind)
                .collect::<Vec<_>>(),
            vec![AtpDeclarationKind::GeneratedBinder; fixture.binder_count]
        );

        let formula = match bundle.properties()[0].encoding() {
            crate::problem::PropertyEncoding::Axiom(formula) => formula,
            crate::problem::PropertyEncoding::NativeDeclaration(_) => {
                panic!("task 8 must emit axiom-form properties")
            }
        };
        let AtpFormulaTree::Forall { binders, body } = formula else {
            panic!("property must be universally quantified");
        };
        assert_eq!(binders.len(), fixture.binder_count);
        assert!(binders.iter().all(|binder| binder.sort().is_none()));
        let binder_symbols = binders
            .iter()
            .map(|binder| binder.variable().clone())
            .collect::<Vec<_>>();
        assert_eq!(
            binder_symbols
                .iter()
                .map(|symbol| symbol.as_str().to_owned())
                .collect::<Vec<_>>(),
            bundle
                .declarations()
                .iter()
                .map(|declaration| declaration.symbol().as_str().to_owned())
                .collect::<Vec<_>>()
        );
        assert_eq!(
            body.as_ref(),
            &fixture.expected_body(&AtpSymbolName::new(fixture.target), &binder_symbols)
        );
    }
}

#[test]
fn generated_binders_are_declared_traceable_and_canonical_under_shuffle() {
    let first = encode_properties(input_for(vec![
        projection(
            "prop:symmetry",
            FAMILY_SYMMETRY,
            "P2",
            "pred:P2",
            AtpPropertyTargetKind::Predicate,
            2,
        ),
        projection(
            "prop:idempotence",
            FAMILY_IDEMPOTENCE,
            "F2",
            "fun:F2",
            AtpPropertyTargetKind::Function,
            2,
        ),
    ]))
    .expect("property bundle");
    let second = encode_properties(input_for(vec![
        projection(
            "prop:idempotence",
            FAMILY_IDEMPOTENCE,
            "F2",
            "fun:F2",
            AtpPropertyTargetKind::Function,
            2,
        ),
        projection(
            "prop:symmetry",
            FAMILY_SYMMETRY,
            "P2",
            "pred:P2",
            AtpPropertyTargetKind::Predicate,
            2,
        ),
    ]))
    .expect("property bundle");

    assert_eq!(first, second);
    assert_eq!(
        first
            .properties()
            .iter()
            .map(EncodedProperty::id)
            .collect::<Vec<_>>(),
        vec![AtpPropertyId::new(10), AtpPropertyId::new(11)]
    );
    assert!(
        first
            .symbol_map()
            .iter()
            .all(|entry| matches!(entry.source(), AtpSymbolSource::GeneratedBinder(_)))
    );
    assert!(first.provenance().iter().all(|entry| {
        matches!(entry.source(), AtpSourceRef::EncodedProperty(_)) && !entry.payload().is_empty()
    }));
    assert!(
        first.declarations()[0].symbol().as_str().ends_with("_0"),
        "first generated binder records position 0"
    );
    assert!(
        first.declarations()[1].symbol().as_str().ends_with("_1"),
        "second generated binder records position 1"
    );
}

#[test]
fn property_bundle_integrates_with_atp_problem_parts() {
    let bundle = encode_properties(input_for(vec![projection(
        "prop:connectedness",
        FAMILY_CONNECTEDNESS,
        "P2",
        "pred:P2",
        AtpPropertyTargetKind::Predicate,
        2,
    )]))
    .expect("property bundle");
    let mut parts = minimal_parts();
    parts
        .declarations
        .extend(bundle.declarations().iter().cloned());
    parts.symbol_map.extend(bundle.symbol_map().iter().cloned());
    parts.provenance.extend(bundle.provenance().iter().cloned());
    parts.properties.extend(bundle.properties().iter().cloned());

    let problem = AtpProblem::try_new(parts).expect("problem with encoded property");
    assert_eq!(problem.properties().len(), 1);
    assert!(problem.debug_text().contains("[properties]"));
    assert!(problem.debug_text().contains("encoding=axiom:"));
    assert!(!problem.debug_text().contains("native-declaration"));
}

#[test]
fn rejects_wrong_target_kind_and_arity_for_function_and_predicate_groups() {
    let error = encode_properties(input_for(vec![projection(
        "prop:bad-function-kind",
        FAMILY_COMMUTATIVITY,
        "P2",
        "pred:P2",
        AtpPropertyTargetKind::Predicate,
        2,
    )]))
    .unwrap_err();
    assert_eq!(
        error,
        AtpPropertyEncodingError::InvalidPropertyTarget {
            family: AtpPropertyFamily::new(FAMILY_COMMUTATIVITY),
            expected_kind: AtpPropertyTargetKind::Function,
            expected_arity: 2,
            actual_kind: AtpPropertyTargetKind::Predicate,
            actual_arity: 2,
        }
    );

    let error = encode_properties(input_for(vec![projection(
        "prop:bad-predicate-arity",
        FAMILY_REFLEXIVITY,
        "P1",
        "pred:P1",
        AtpPropertyTargetKind::Predicate,
        1,
    )]))
    .unwrap_err();
    assert_eq!(
        error,
        AtpPropertyEncodingError::InvalidPropertyTarget {
            family: AtpPropertyFamily::new(FAMILY_REFLEXIVITY),
            expected_kind: AtpPropertyTargetKind::Predicate,
            expected_arity: 2,
            actual_kind: AtpPropertyTargetKind::Predicate,
            actual_arity: 1,
        }
    );
}

#[test]
fn rejects_missing_target_declaration_symbol_map_and_provenance_payload() {
    let mut input = input_for(vec![projection(
        "prop:missing-declaration",
        FAMILY_SYMMETRY,
        "P2",
        "pred:P2",
        AtpPropertyTargetKind::Predicate,
        2,
    )]);
    input
        .existing_declarations
        .retain(|declaration| declaration.symbol().as_str() != "P2");
    assert_eq!(
        encode_properties(input).unwrap_err(),
        AtpPropertyEncodingError::MissingDeclarationSymbol {
            symbol: AtpSymbolName::new("P2"),
        }
    );

    let mut input = input_for(vec![projection(
        "prop:missing-symbol",
        FAMILY_SYMMETRY,
        "P2",
        "pred:P2",
        AtpPropertyTargetKind::Predicate,
        2,
    )]);
    input
        .existing_symbol_map
        .retain(|entry| entry.backend_symbol().as_str() != "P2");
    assert_eq!(
        encode_properties(input).unwrap_err(),
        AtpPropertyEncodingError::MissingSymbolMap {
            symbol: AtpSymbolName::new("P2"),
        }
    );

    let mut bad_payload = projection(
        "prop:empty-payload",
        FAMILY_SYMMETRY,
        "P2",
        "pred:P2",
        AtpPropertyTargetKind::Predicate,
        2,
    );
    bad_payload.provenance_payload = AtpPayload::new("");
    assert_eq!(
        encode_properties(input_for(vec![bad_payload])).unwrap_err(),
        AtpPropertyEncodingError::EmptyField {
            field: "property.provenance_payload",
        }
    );

    let mut id_collision = input_for(vec![projection(
        "prop:id-collision",
        FAMILY_SYMMETRY,
        "P2",
        "pred:P2",
        AtpPropertyTargetKind::Predicate,
        2,
    )]);
    id_collision.next_provenance_id = AtpProvenanceId::new(1);
    assert_eq!(
        encode_properties(id_collision).unwrap_err(),
        AtpPropertyEncodingError::DuplicateId {
            section: "provenance",
            id: 1,
        }
    );
}

#[test]
fn rejects_mismatched_existing_target_declaration_and_source_identity() {
    let mut wrong_declaration_kind = input_for(vec![projection(
        "prop:wrong-declaration-kind",
        FAMILY_SYMMETRY,
        "P2",
        "pred:P2",
        AtpPropertyTargetKind::Predicate,
        2,
    )]);
    replace_declaration(
        &mut wrong_declaration_kind.existing_declarations,
        "P2",
        AtpDeclarationKind::Function,
        2,
    );
    assert_eq!(
        encode_properties(wrong_declaration_kind).unwrap_err(),
        AtpPropertyEncodingError::InvalidSymbolDeclaration {
            symbol: AtpSymbolName::new("P2"),
            expected: "predicate",
            actual: AtpDeclarationKind::Function,
        }
    );

    let mut wrong_declaration_arity = input_for(vec![projection(
        "prop:wrong-declaration-arity",
        FAMILY_SYMMETRY,
        "P2",
        "pred:P2",
        AtpPropertyTargetKind::Predicate,
        2,
    )]);
    replace_declaration(
        &mut wrong_declaration_arity.existing_declarations,
        "P2",
        AtpDeclarationKind::Predicate,
        1,
    );
    assert_eq!(
        encode_properties(wrong_declaration_arity).unwrap_err(),
        AtpPropertyEncodingError::InvalidSymbolArity {
            symbol: AtpSymbolName::new("P2"),
            expected: 2,
            actual: 1,
        }
    );

    let mut wrong_source = input_for(vec![projection(
        "prop:wrong-source",
        FAMILY_SYMMETRY,
        "P2",
        "pred:P2",
        AtpPropertyTargetKind::Predicate,
        2,
    )]);
    replace_symbol_source(&mut wrong_source.existing_symbol_map, "P2", "pred:other");
    assert_eq!(
        encode_properties(wrong_source).unwrap_err(),
        AtpPropertyEncodingError::InvalidSymbolSource {
            symbol: AtpSymbolName::new("P2"),
            expected: AtpSymbolSource::MizarSymbol(AtpSourceBinding::new("pred:P2")),
            actual: AtpSymbolSource::MizarSymbol(AtpSourceBinding::new("pred:other")),
        }
    );

    let mut duplicate_source = input_for(vec![projection(
        "prop:duplicate-symbol-source",
        FAMILY_SYMMETRY,
        "P2",
        "pred:P2",
        AtpPropertyTargetKind::Predicate,
        2,
    )]);
    duplicate_source
        .existing_symbol_map
        .push(AtpSymbolMapEntry::new(
            "P2_alias",
            AtpSymbolSource::MizarSymbol(AtpSourceBinding::new("pred:P2")),
        ));
    assert_eq!(
        encode_properties(duplicate_source).unwrap_err(),
        AtpPropertyEncodingError::DuplicateSymbolSource {
            source: AtpSymbolSource::MizarSymbol(AtpSourceBinding::new("pred:P2")),
        }
    );

    let mut duplicate_generated_binder_source = input_for(vec![projection(
        "prop:generated-source-collision",
        FAMILY_SYMMETRY,
        "P2",
        "pred:P2",
        AtpPropertyTargetKind::Predicate,
        2,
    )]);
    duplicate_generated_binder_source
        .existing_symbol_map
        .push(AtpSymbolMapEntry::new(
            "preexisting_binder",
            AtpSymbolSource::GeneratedBinder(AtpSourceBinding::new(
                "prop:generated-source-collision#target:pred:P2#binder:0",
            )),
        ));
    assert_eq!(
        encode_properties(duplicate_generated_binder_source).unwrap_err(),
        AtpPropertyEncodingError::DuplicateSymbolSource {
            source: AtpSymbolSource::GeneratedBinder(AtpSourceBinding::new(
                "prop:generated-source-collision#target:pred:P2#binder:0",
            )),
        }
    );
}

#[test]
fn rejects_profile_without_quantifiers_or_required_equality() {
    let mut no_quantifier = input_for(vec![projection(
        "prop:no-quantifier",
        FAMILY_SYMMETRY,
        "P2",
        "pred:P2",
        AtpPropertyTargetKind::Predicate,
        2,
    )]);
    no_quantifier.logic_profile = profile(QuantifierPolicy::PropositionalOnly, true, false);
    assert_eq!(
        encode_properties(no_quantifier).unwrap_err(),
        AtpPropertyEncodingError::UnsupportedProfileFeature {
            feature: "quantifier",
        }
    );

    let mut no_function_equality = input_for(vec![projection(
        "prop:no-function-equality",
        FAMILY_COMMUTATIVITY,
        "F2",
        "fun:F2",
        AtpPropertyTargetKind::Function,
        2,
    )]);
    no_function_equality.logic_profile = profile(QuantifierPolicy::FirstOrder, false, false);
    assert_eq!(
        encode_properties(no_function_equality).unwrap_err(),
        AtpPropertyEncodingError::UnsupportedProfileFeature {
            feature: "equality",
        }
    );

    let mut no_connectedness_equality = input_for(vec![projection(
        "prop:no-connectedness-equality",
        FAMILY_CONNECTEDNESS,
        "P2",
        "pred:P2",
        AtpPropertyTargetKind::Predicate,
        2,
    )]);
    no_connectedness_equality.logic_profile = profile(QuantifierPolicy::FirstOrder, false, false);
    assert_eq!(
        encode_properties(no_connectedness_equality).unwrap_err(),
        AtpPropertyEncodingError::UnsupportedProfileFeature {
            feature: "equality",
        }
    );
}

#[test]
fn native_declaration_requests_are_deferred_even_when_profile_supports_them() {
    let mut native = projection(
        "prop:native",
        FAMILY_SYMMETRY,
        "P2",
        "pred:P2",
        AtpPropertyTargetKind::Predicate,
        2,
    );
    native.encoding_strategy = AtpPropertyEncodingStrategy::NativeDeclaration;
    let mut input = input_for(vec![native]);
    input.logic_profile = profile(QuantifierPolicy::FirstOrder, true, true);

    assert_eq!(
        encode_properties(input).unwrap_err(),
        AtpPropertyEncodingError::NativeDeclarationDeferred
    );
}

#[test]
fn rejects_unsupported_family_and_duplicate_identities() {
    assert_eq!(
        encode_properties(input_for(vec![projection(
            "prop:assoc",
            "associativity",
            "F2",
            "fun:F2",
            AtpPropertyTargetKind::Function,
            2,
        )]))
        .unwrap_err(),
        AtpPropertyEncodingError::UnsupportedFamily {
            family: AtpPropertyFamily::new("associativity"),
        }
    );

    assert_eq!(
        encode_properties(input_for(vec![
            projection(
                "prop:duplicate",
                FAMILY_SYMMETRY,
                "P2",
                "pred:P2",
                AtpPropertyTargetKind::Predicate,
                2,
            ),
            projection(
                "prop:duplicate",
                FAMILY_ASYMMETRY,
                "P2",
                "pred:P2",
                AtpPropertyTargetKind::Predicate,
                2,
            ),
        ]))
        .unwrap_err(),
        AtpPropertyEncodingError::DuplicateSourceProperty {
            source_property: AtpSourceBinding::new("prop:duplicate"),
        }
    );

    let duplicate_encoded = encode_properties(input_for(vec![
        projection(
            "prop:symmetry-a",
            FAMILY_SYMMETRY,
            "P2",
            "pred:P2",
            AtpPropertyTargetKind::Predicate,
            2,
        ),
        projection(
            "prop:symmetry-b",
            FAMILY_SYMMETRY,
            "P2",
            "pred:P2",
            AtpPropertyTargetKind::Predicate,
            2,
        ),
    ]))
    .unwrap_err();
    assert!(matches!(
        duplicate_encoded,
        AtpPropertyEncodingError::DuplicateEncodedProperty { .. }
    ));
}

#[test]
fn binder_sort_is_validated_and_attached_to_generated_binders() {
    let mut sorted = projection(
        "prop:sorted-reflexivity",
        FAMILY_REFLEXIVITY,
        "P2",
        "pred:P2",
        AtpPropertyTargetKind::Predicate,
        2,
    );
    sorted.binder_sort = Some(AtpPropertyBinderSort {
        symbol: AtpSymbolName::new("S"),
        source: AtpSourceBinding::new("sort:S"),
    });

    let bundle = encode_properties(input_for(vec![sorted])).expect("sorted property bundle");
    let crate::problem::PropertyEncoding::Axiom(AtpFormulaTree::Forall { binders, .. }) =
        bundle.properties()[0].encoding()
    else {
        panic!("expected sorted quantified property");
    };
    assert_eq!(binders[0].sort().map(AtpSymbolName::as_str), Some("S"));

    let mut bad_sort = projection(
        "prop:bad-sort",
        FAMILY_REFLEXIVITY,
        "P2",
        "pred:P2",
        AtpPropertyTargetKind::Predicate,
        2,
    );
    bad_sort.binder_sort = Some(AtpPropertyBinderSort {
        symbol: AtpSymbolName::new("missing-sort"),
        source: AtpSourceBinding::new("sort:missing"),
    });
    assert_eq!(
        encode_properties(input_for(vec![bad_sort])).unwrap_err(),
        AtpPropertyEncodingError::MissingSymbolMap {
            symbol: AtpSymbolName::new("missing-sort"),
        }
    );
}

struct FamilyFixture {
    family: &'static str,
    target: &'static str,
    target_source: &'static str,
    target_kind: AtpPropertyTargetKind,
    target_arity: u32,
    binder_count: usize,
    expected_body: fn(&AtpSymbolName, &[AtpSymbolName]) -> AtpFormulaTree,
}

impl FamilyFixture {
    fn projection(&self) -> AtpPropertyProjection {
        projection(
            &format!("prop:{}", self.family),
            self.family,
            self.target,
            self.target_source,
            self.target_kind,
            self.target_arity,
        )
    }

    fn expected_body(
        &self,
        target_symbol: &AtpSymbolName,
        binder_symbols: &[AtpSymbolName],
    ) -> AtpFormulaTree {
        (self.expected_body)(target_symbol, binder_symbols)
    }
}

fn family_fixtures() -> Vec<FamilyFixture> {
    vec![
        FamilyFixture {
            family: FAMILY_COMMUTATIVITY,
            target: "F2",
            target_source: "fun:F2",
            target_kind: AtpPropertyTargetKind::Function,
            target_arity: 2,
            binder_count: 2,
            expected_body: expected_commutativity,
        },
        FamilyFixture {
            family: FAMILY_SYMMETRY,
            target: "P2",
            target_source: "pred:P2",
            target_kind: AtpPropertyTargetKind::Predicate,
            target_arity: 2,
            binder_count: 2,
            expected_body: expected_symmetry,
        },
        FamilyFixture {
            family: FAMILY_REFLEXIVITY,
            target: "P2",
            target_source: "pred:P2",
            target_kind: AtpPropertyTargetKind::Predicate,
            target_arity: 2,
            binder_count: 1,
            expected_body: expected_reflexivity,
        },
        FamilyFixture {
            family: FAMILY_IDEMPOTENCE,
            target: "F2",
            target_source: "fun:F2",
            target_kind: AtpPropertyTargetKind::Function,
            target_arity: 2,
            binder_count: 1,
            expected_body: expected_idempotence,
        },
        FamilyFixture {
            family: FAMILY_INVOLUTIVENESS,
            target: "F1",
            target_source: "fun:F1",
            target_kind: AtpPropertyTargetKind::Function,
            target_arity: 1,
            binder_count: 1,
            expected_body: expected_involutiveness,
        },
        FamilyFixture {
            family: FAMILY_PROJECTIVITY,
            target: "F1",
            target_source: "fun:F1",
            target_kind: AtpPropertyTargetKind::Function,
            target_arity: 1,
            binder_count: 1,
            expected_body: expected_projectivity,
        },
        FamilyFixture {
            family: FAMILY_ASYMMETRY,
            target: "P2",
            target_source: "pred:P2",
            target_kind: AtpPropertyTargetKind::Predicate,
            target_arity: 2,
            binder_count: 2,
            expected_body: expected_asymmetry,
        },
        FamilyFixture {
            family: FAMILY_CONNECTEDNESS,
            target: "P2",
            target_source: "pred:P2",
            target_kind: AtpPropertyTargetKind::Predicate,
            target_arity: 2,
            binder_count: 2,
            expected_body: expected_connectedness,
        },
        FamilyFixture {
            family: FAMILY_IRREFLEXIVITY,
            target: "P2",
            target_source: "pred:P2",
            target_kind: AtpPropertyTargetKind::Predicate,
            target_arity: 2,
            binder_count: 1,
            expected_body: expected_irreflexivity,
        },
    ]
}

fn expected_commutativity(target: &AtpSymbolName, binders: &[AtpSymbolName]) -> AtpFormulaTree {
    let a = variable(&binders[0]);
    let b = variable(&binders[1]);
    equality(
        function(target, vec![a.clone(), b.clone()]),
        function(target, vec![b, a]),
    )
}

fn expected_symmetry(target: &AtpSymbolName, binders: &[AtpSymbolName]) -> AtpFormulaTree {
    let a = variable(&binders[0]);
    let b = variable(&binders[1]);
    implies(
        atom(target, vec![a.clone(), b.clone()]),
        atom(target, vec![b, a]),
    )
}

fn expected_reflexivity(target: &AtpSymbolName, binders: &[AtpSymbolName]) -> AtpFormulaTree {
    let a = variable(&binders[0]);
    atom(target, vec![a.clone(), a])
}

fn expected_idempotence(target: &AtpSymbolName, binders: &[AtpSymbolName]) -> AtpFormulaTree {
    let a = variable(&binders[0]);
    equality(function(target, vec![a.clone(), a.clone()]), a)
}

fn expected_involutiveness(target: &AtpSymbolName, binders: &[AtpSymbolName]) -> AtpFormulaTree {
    let a = variable(&binders[0]);
    equality(function(target, vec![function(target, vec![a.clone()])]), a)
}

fn expected_projectivity(target: &AtpSymbolName, binders: &[AtpSymbolName]) -> AtpFormulaTree {
    let a = variable(&binders[0]);
    equality(
        function(target, vec![function(target, vec![a.clone()])]),
        function(target, vec![a]),
    )
}

fn expected_asymmetry(target: &AtpSymbolName, binders: &[AtpSymbolName]) -> AtpFormulaTree {
    let a = variable(&binders[0]);
    let b = variable(&binders[1]);
    implies(
        atom(target, vec![a.clone(), b.clone()]),
        AtpFormulaTree::Not(Box::new(atom(target, vec![b, a]))),
    )
}

fn expected_connectedness(target: &AtpSymbolName, binders: &[AtpSymbolName]) -> AtpFormulaTree {
    let a = variable(&binders[0]);
    let b = variable(&binders[1]);
    implies(
        AtpFormulaTree::Not(Box::new(equality(a.clone(), b.clone()))),
        AtpFormulaTree::Or(vec![
            atom(target, vec![a.clone(), b.clone()]),
            atom(target, vec![b, a]),
        ]),
    )
}

fn expected_irreflexivity(target: &AtpSymbolName, binders: &[AtpSymbolName]) -> AtpFormulaTree {
    let a = variable(&binders[0]);
    AtpFormulaTree::Not(Box::new(atom(target, vec![a.clone(), a])))
}

fn input_for(property_projections: Vec<AtpPropertyProjection>) -> AtpPropertyEncodingInput {
    AtpPropertyEncodingInput {
        logic_profile: profile(QuantifierPolicy::FirstOrder, true, false),
        existing_declarations: base_declarations(),
        existing_symbol_map: base_symbol_map(),
        existing_provenance: base_provenance(),
        property_projections,
        next_property_id: AtpPropertyId::new(10),
        next_declaration_id: AtpDeclarationId::new(100),
        next_provenance_id: AtpProvenanceId::new(200),
    }
}

fn projection(
    source_property: &str,
    family: &str,
    target_symbol: &str,
    target_source: &str,
    target_kind: AtpPropertyTargetKind,
    target_arity: u32,
) -> AtpPropertyProjection {
    AtpPropertyProjection {
        source_property: AtpSourceBinding::new(source_property),
        family: AtpPropertyFamily::new(family),
        target_symbol: AtpSymbolName::new(target_symbol),
        target_source: AtpSourceBinding::new(target_source),
        target_kind,
        target_arity,
        binder_sort: None,
        provenance_payload: AtpPayload::new(format!("payload:{source_property}")),
        encoding_strategy: AtpPropertyEncodingStrategy::Axiom,
    }
}

fn minimal_parts() -> AtpProblemParts {
    let mut declarations = base_declarations();
    declarations.retain(|declaration| {
        matches!(
            declaration.symbol().as_str(),
            "Q" | "P2" | "F1" | "F2" | "S"
        )
    });
    let mut symbol_map = base_symbol_map();
    symbol_map.retain(|entry| {
        matches!(
            entry.backend_symbol().as_str(),
            "Q" | "P2" | "F1" | "F2" | "S"
        )
    });
    AtpProblemParts {
        vc_id: mizar_vc::vc_ir::VcId::new(11),
        target_binding: AtpTargetBinding::new(
            AtpFingerprint::new(18, b"target-vc-11".to_vec()).expect("fingerprint"),
            AtpSourceBinding::new("vc:11"),
        )
        .expect("target binding"),
        logic_profile: profile(QuantifierPolicy::FirstOrder, true, false),
        expected_result: ExpectedBackendResult::Unsat,
        declarations,
        axioms: vec![AtpFormula::new(
            AtpFormulaId::new(1),
            AtpFormulaTree::Atom(AtpAtom::new("Q", Vec::new())),
            AtpProvenanceId::new(2),
        )],
        conjecture: AtpFormula::new(
            AtpFormulaId::new(2),
            AtpFormulaTree::Atom(AtpAtom::new("Q", Vec::new())),
            AtpProvenanceId::new(3),
        ),
        type_context: AtpTypeContext::new(Vec::new()),
        properties: Vec::new(),
        symbol_map,
        provenance: base_provenance(),
        diagnostics: vec![AtpDiagnostic::new("note", "property fixture")],
    }
}

fn base_declarations() -> Vec<AtpDeclaration> {
    vec![
        declaration(1, AtpDeclarationKind::Predicate, "Q", 0),
        declaration(2, AtpDeclarationKind::Predicate, "P2", 2),
        declaration(3, AtpDeclarationKind::Predicate, "P1", 1),
        declaration(4, AtpDeclarationKind::Function, "F2", 2),
        declaration(5, AtpDeclarationKind::Function, "F1", 1),
        declaration(6, AtpDeclarationKind::Sort, "S", 0),
    ]
}

fn declaration(id: u32, kind: AtpDeclarationKind, symbol: &str, arity: u32) -> AtpDeclaration {
    AtpDeclaration::new(
        AtpDeclarationId::new(id),
        kind,
        symbol,
        arity,
        AtpProvenanceId::new(1),
    )
}

fn base_symbol_map() -> Vec<AtpSymbolMapEntry> {
    vec![
        symbol("Q", "pred:Q"),
        symbol("P2", "pred:P2"),
        symbol("P1", "pred:P1"),
        symbol("F2", "fun:F2"),
        symbol("F1", "fun:F1"),
        symbol("S", "sort:S"),
    ]
}

fn symbol(symbol: &str, source: &str) -> AtpSymbolMapEntry {
    AtpSymbolMapEntry::new(
        symbol,
        AtpSymbolSource::MizarSymbol(AtpSourceBinding::new(source)),
    )
}

fn replace_declaration(
    declarations: &mut [AtpDeclaration],
    symbol: &str,
    kind: AtpDeclarationKind,
    arity: u32,
) {
    let declaration = declarations
        .iter_mut()
        .find(|declaration| declaration.symbol().as_str() == symbol)
        .expect("fixture declaration exists");
    *declaration = AtpDeclaration::new(
        declaration.id(),
        kind,
        symbol,
        arity,
        declaration.provenance(),
    );
}

fn replace_symbol_source(entries: &mut [AtpSymbolMapEntry], symbol: &str, source: &str) {
    let entry = entries
        .iter_mut()
        .find(|entry| entry.backend_symbol().as_str() == symbol)
        .expect("fixture symbol-map row exists");
    *entry = AtpSymbolMapEntry::new(
        symbol,
        AtpSymbolSource::MizarSymbol(AtpSourceBinding::new(source)),
    );
}

fn base_provenance() -> Vec<AtpProvenance> {
    vec![
        AtpProvenance::new(
            AtpProvenanceId::new(1),
            AtpSourceRef::LocalHypothesis(AtpSourceBinding::new("decls")),
            "decl payload",
        ),
        AtpProvenance::new(
            AtpProvenanceId::new(2),
            AtpSourceRef::CitedPremise(AtpSourceBinding::new("premise:1")),
            "axiom payload",
        ),
        AtpProvenance::new(
            AtpProvenanceId::new(3),
            AtpSourceRef::GeneratedVcFact(AtpSourceBinding::new("goal:1")),
            "goal payload",
        ),
    ]
}

fn profile(quantifiers: QuantifierPolicy, equality: bool, native_properties: bool) -> LogicProfile {
    LogicProfile::try_new(
        "property-fixture",
        LogicFragment::Fof,
        if equality {
            EqualitySupport::Supported
        } else {
            EqualitySupport::Unsupported
        },
        quantifiers,
        SoftTypeStrategy::BackendSorts,
        if native_properties {
            NativePropertySupport::Supported
        } else {
            NativePropertySupport::Unsupported
        },
        BTreeSet::from([ConcreteFormat::Tptp]),
    )
    .expect("profile")
}
