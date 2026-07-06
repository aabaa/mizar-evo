use super::*;
use crate::problem::{
    AtpAtom, AtpBinder, ConcreteFormat, EqualitySupport, LogicFragment, NativePropertySupport,
    QuantifierPolicy,
};
use mizar_core::{
    control_flow::ObligationHandoffId,
    core_ir::{
        CoreItemId, CoreLabelRef, CoreProvenance, CoreProvenancePhase, CoreSourceRef,
        LocalProofOrProgramPath, NormalizedSemanticOrigin, ObligationSeedId, ObligationSeedStatus,
    },
};
use mizar_session::{
    BuildSnapshotId, InMemorySessionIdAllocator, SessionIdAllocator, SourceId, SourceRange,
};
use mizar_vc::{
    kernel_evidence_handoff::{
        IMPORTED_STATEMENT_FINGERPRINT_ALGORITHM_ID, KERNEL_FORMULA_FINGERPRINT_ALGORITHM_ID,
        KernelClauseTautologyPolicy, KernelEvidenceFingerprint, KernelEvidenceHandoffInput,
        KernelEvidenceProfile, KernelFormulaContextRequirements, KernelFormulaPayload,
        KernelFormulaProjection, KernelGoalPolarity, KernelImportedFactRequirement,
        KernelImportedFormulaClass, KernelImportedFormulaPayload,
        KernelImportedStatementProjection, KernelRequiredProofStatus,
        build_kernel_evidence_handoff, canonical_imported_statement_projection_payload,
    },
    vc_ir::{
        AnchorCompleteness, AnchorIngredient, AnchorLabel, AnchorLabelRole, AnchorOwner,
        AnchorUnavailableReason, CanonicalSortKey, ContextEntry, ContextEntryId, ContextEntryKind,
        GenerationSchemaVersion, HashMarker, LocalContext, PremiseRef, PremiseRestriction,
        ProofHint, SeedAccounting, SeedOriginRef, SeedVcMapping, VcFormulaRef, VcGeneratedFormula,
        VcGeneratedFormulaId, VcGeneratedFormulaKind, VcGeneratedFormulaShape, VcIr, VcKind,
        VcModuleRef, VcProvenance, VcProvenancePhase, VcSchemaVersion, VcSet, VcSetParts,
        VcSourceRef, VcText,
    },
};
use std::collections::BTreeSet;

#[test]
fn imported_statement_projection_payload_matches_kernel_canonical_bytes() {
    let vc_statement = KernelEvidenceFingerprint::new(
        IMPORTED_STATEMENT_FINGERPRINT_ALGORITHM_ID,
        b"statement".to_vec(),
    )
    .expect("vc statement fingerprint");
    let vc_formula = KernelEvidenceFingerprint::new(
        KERNEL_FORMULA_FINGERPRINT_ALGORITHM_ID,
        b"formula".to_vec(),
    )
    .expect("vc formula fingerprint");
    let kernel_statement = mizar_kernel::certificate_parser::Fingerprint::new(
        mizar_kernel::formula_evidence::IMPORTED_STATEMENT_FINGERPRINT_ALGORITHM_ID,
        b"statement".to_vec(),
    );
    let kernel_formula = mizar_kernel::certificate_parser::Fingerprint::new(
        mizar_kernel::formula_evidence::SUPPORTED_FORMULA_FINGERPRINT_ALGORITHM_ID,
        b"formula".to_vec(),
    );

    let vc_payload = canonical_imported_statement_projection_payload(&vc_statement, &vc_formula)
        .expect("vc canonical payload");
    let kernel_payload =
        mizar_kernel::formula_evidence::canonical_imported_statement_projection_payload(
            &kernel_statement,
            &kernel_formula,
        )
        .expect("kernel canonical payload");

    let mut expected = b"MIZAR_KERNEL_IMPORTED_STATEMENT_PROJECTION\0".to_vec();
    expected.extend([18, 0, 0, 0, 9]);
    expected.extend(b"statement");
    expected.extend([2, 0, 0, 0, 7]);
    expected.extend(b"formula");
    assert_eq!(vc_payload, expected);
    assert_eq!(vc_payload, kernel_payload);
}

#[test]
fn translates_declarations_in_projection_key_order() {
    let set = fixture_set(VcStatus::NeedsAtp, "sample");
    let handoff = handoff(&set);
    let logic_profile = profile(SoftTypeStrategy::BackendSorts);
    let output = translate_declarations(AtpDeclarationTranslationInput {
        vc_set: &set,
        vc: VcId::new(0),
        kernel_handoff: &handoff,
        logic_profile: logic_profile.clone(),
        declaration_projections: vec![
            declaration_projection("zeta", "z", AtpDeclarationKind::Function, 0),
            declaration_projection("alpha", "a", AtpDeclarationKind::Predicate, 1),
        ],
        soft_type_projections: vec![],
        diagnostics: vec![AtpDiagnostic::new("note", "deterministic")],
    })
    .expect("translation");

    assert_eq!(output.vc_id(), VcId::new(0));
    assert_eq!(
        output
            .declarations()
            .iter()
            .map(|declaration| (declaration.id().index(), declaration.symbol().as_str()))
            .collect::<Vec<_>>(),
        [(0, "a"), (1, "z")]
    );
    assert_eq!(
        output
            .symbol_map()
            .iter()
            .map(|entry| entry.backend_symbol().as_str())
            .collect::<Vec<_>>(),
        ["a", "z"]
    );
    assert_eq!(output.logic_profile(), &logic_profile);
    assert!(
        output
            .target_binding()
            .producer_binding()
            .as_str()
            .contains("mizar-vc-kernel-evidence-handoff:")
    );
    assert_eq!(output.diagnostics().len(), 1);
}

#[test]
fn shuffled_equivalent_projection_inputs_have_identical_outputs() {
    let set = fixture_set(VcStatus::NeedsAtp, "sample");
    let handoff = handoff(&set);
    let mut first = empty_input(&set, &handoff);
    first.declaration_projections = vec![
        declaration_projection("beta", "b", AtpDeclarationKind::Function, 0),
        declaration_projection("alpha", "a", AtpDeclarationKind::Predicate, 1),
    ];
    let mut second = empty_input(&set, &handoff);
    second.declaration_projections = vec![
        declaration_projection("alpha", "a", AtpDeclarationKind::Predicate, 1),
        declaration_projection("beta", "b", AtpDeclarationKind::Function, 0),
    ];

    assert_eq!(
        translate_declarations(first).expect("first"),
        translate_declarations(second).expect("second")
    );
}

#[test]
fn shuffled_equivalent_soft_type_inputs_have_identical_outputs() {
    let set = fixture_set(VcStatus::NeedsAtp, "sample");
    let handoff = handoff(&set);
    let mut first = empty_input(&set, &handoff);
    first.logic_profile = profile(SoftTypeStrategy::GuardPredicates);
    first.declaration_projections = vec![
        declaration_projection("pred-a", "a", AtpDeclarationKind::Predicate, 0),
        declaration_projection("pred-b", "b", AtpDeclarationKind::Predicate, 0),
    ];
    first.soft_type_projections = vec![
        soft_guard(
            "z-guard",
            AtpFormulaTree::Atom(AtpAtom::new("b", Vec::new())),
        ),
        soft_guard(
            "a-guard",
            AtpFormulaTree::Atom(AtpAtom::new("a", Vec::new())),
        ),
    ];
    let mut second = empty_input(&set, &handoff);
    second.logic_profile = profile(SoftTypeStrategy::GuardPredicates);
    second.declaration_projections = vec![
        declaration_projection("pred-b", "b", AtpDeclarationKind::Predicate, 0),
        declaration_projection("pred-a", "a", AtpDeclarationKind::Predicate, 0),
    ];
    second.soft_type_projections = vec![
        soft_guard(
            "a-guard",
            AtpFormulaTree::Atom(AtpAtom::new("a", Vec::new())),
        ),
        soft_guard(
            "z-guard",
            AtpFormulaTree::Atom(AtpAtom::new("b", Vec::new())),
        ),
    ];

    assert_eq!(
        translate_declarations(first).expect("first"),
        translate_declarations(second).expect("second")
    );
}

#[test]
fn rejects_non_needs_atp_vcs() {
    let set = fixture_set(VcStatus::Open, "sample");
    let handoff = handoff(&set);
    let error =
        translate_declarations(empty_input(&set, &handoff)).expect_err("non-NeedsAtp status");

    assert!(matches!(
        error,
        AtpTranslationError::NonNeedsAtpStatus {
            status: VcStatus::Open,
            ..
        }
    ));
}

#[test]
fn rejects_mismatched_handoff_targets() {
    let handoff_set = fixture_set(VcStatus::NeedsAtp, "left");
    let input_set = fixture_set(VcStatus::NeedsAtp, "right");
    let handoff = handoff(&handoff_set);
    let error =
        translate_declarations(empty_input(&input_set, &handoff)).expect_err("mismatched target");

    assert!(matches!(
        error,
        AtpTranslationError::MismatchedTargetHandoff { vc } if vc == VcId::new(0)
    ));
}

#[test]
fn duplicate_projection_keys_fail_closed() {
    let set = fixture_set(VcStatus::NeedsAtp, "sample");
    let handoff = handoff(&set);
    let mut input = empty_input(&set, &handoff);
    input.declaration_projections = vec![
        declaration_projection("dup", "p", AtpDeclarationKind::Predicate, 0),
        declaration_projection("dup", "q", AtpDeclarationKind::Predicate, 0),
    ];
    let error = translate_declarations(input).expect_err("duplicate key");

    assert!(matches!(
        error,
        AtpTranslationError::DuplicateProjectionKey {
            section: "declaration projections",
            ..
        }
    ));
}

#[test]
fn malformed_projection_inputs_fail_closed() {
    let set = fixture_set(VcStatus::NeedsAtp, "sample");
    let handoff = handoff(&set);
    let mut empty_key = empty_input(&set, &handoff);
    empty_key.declaration_projections = vec![declaration_projection(
        "",
        "p",
        AtpDeclarationKind::Predicate,
        0,
    )];
    assert!(matches!(
        translate_declarations(empty_key).expect_err("empty projection key"),
        AtpTranslationError::EmptyProjectionKey {
            section: "declaration projections"
        }
    ));

    let mut empty_payload = empty_input(&set, &handoff);
    let mut projection = declaration_projection("p", "p", AtpDeclarationKind::Predicate, 0);
    projection.provenance.payload = AtpPayload::new("");
    empty_payload.declaration_projections = vec![projection];
    assert!(matches!(
        translate_declarations(empty_payload).expect_err("empty provenance payload"),
        AtpTranslationError::Problem {
            source: AtpProblemError::EmptyField {
                field: "projection.provenance.payload"
            }
        }
    ));

    let mut empty_source = empty_input(&set, &handoff);
    let mut projection = declaration_projection("q", "q", AtpDeclarationKind::Predicate, 0);
    projection.provenance.source = AtpSourceRef::GeneratedVcFact(AtpSourceBinding::new(""));
    empty_source.declaration_projections = vec![projection];
    assert!(matches!(
        translate_declarations(empty_source).expect_err("empty provenance source"),
        AtpTranslationError::Problem {
            source: AtpProblemError::EmptyField {
                field: "projection.provenance.source"
            }
        }
    ));
}

#[test]
fn malformed_soft_type_projection_inputs_fail_closed() {
    let set = fixture_set(VcStatus::NeedsAtp, "sample");
    let handoff = handoff(&set);
    let mut empty_key = empty_input(&set, &handoff);
    empty_key.soft_type_projections = vec![soft_guard("", AtpFormulaTree::True)];
    assert!(matches!(
        translate_declarations(empty_key).expect_err("empty soft-type projection key"),
        AtpTranslationError::EmptyProjectionKey {
            section: "soft-type projections"
        }
    ));

    let mut empty_payload = empty_input(&set, &handoff);
    let mut projection = soft_guard("guard", AtpFormulaTree::True);
    projection.provenance.payload = AtpPayload::new("");
    empty_payload.soft_type_projections = vec![projection];
    assert!(matches!(
        translate_declarations(empty_payload).expect_err("empty soft-type provenance payload"),
        AtpTranslationError::Problem {
            source: AtpProblemError::EmptyField {
                field: "projection.provenance.payload"
            }
        }
    ));

    let mut empty_source = empty_input(&set, &handoff);
    let mut projection = soft_guard("guard", AtpFormulaTree::True);
    projection.provenance.source = AtpSourceRef::TypeFact(AtpSourceBinding::new(""));
    empty_source.soft_type_projections = vec![projection];
    assert!(matches!(
        translate_declarations(empty_source).expect_err("empty soft-type provenance source"),
        AtpTranslationError::Problem {
            source: AtpProblemError::EmptyField {
                field: "projection.provenance.source"
            }
        }
    ));
}

#[test]
fn missing_type_guard_projection_fails_closed() {
    let set = fixture_set(VcStatus::NeedsAtp, "sample");
    let handoff = handoff(&set);
    let mut input = empty_input(&set, &handoff);
    input.declaration_projections = vec![AtpDeclarationProjection {
        key: AtpProjectionKey::new("type-predicate"),
        kind: AtpDeclarationKind::Predicate,
        symbol: AtpSymbolName::new("type_guard"),
        arity: 0,
        provenance: declaration_provenance("type-predicate"),
        symbol_source: AtpSymbolSourceProjection::TypeGuard(AtpProjectionKey::new("missing")),
    }];
    let error = translate_declarations(input).expect_err("missing type guard projection");

    assert!(matches!(
        error,
        AtpTranslationError::MissingTypeGuardProjection { .. }
    ));
}

#[test]
fn duplicate_symbols_and_missing_guard_symbols_fail_closed() {
    let set = fixture_set(VcStatus::NeedsAtp, "sample");
    let handoff = handoff(&set);
    let mut duplicate_symbol = empty_input(&set, &handoff);
    duplicate_symbol.declaration_projections = vec![
        declaration_projection("alpha", "p", AtpDeclarationKind::Predicate, 0),
        declaration_projection("beta", "p", AtpDeclarationKind::Predicate, 0),
    ];
    assert!(matches!(
        translate_declarations(duplicate_symbol).expect_err("duplicate declaration symbol"),
        AtpTranslationError::Problem {
            source: AtpProblemError::DuplicateSymbolMap { .. }
        }
    ));

    let mut missing_symbol = empty_input(&set, &handoff);
    missing_symbol.logic_profile = profile(SoftTypeStrategy::GuardPredicates);
    missing_symbol.soft_type_projections = vec![soft_guard(
        "guard",
        AtpFormulaTree::Atom(AtpAtom::new("missing", Vec::new())),
    )];
    assert!(matches!(
        translate_declarations(missing_symbol).expect_err("missing guard symbol"),
        AtpTranslationError::Problem {
            source: AtpProblemError::MissingSymbolMap { .. }
        }
    ));
}

#[test]
fn kind_and_arity_mismatches_are_rejected_by_translator_signature_validation() {
    let set = fixture_set(VcStatus::NeedsAtp, "sample");
    let handoff = handoff(&set);
    let mut input = empty_input(&set, &handoff);
    input.logic_profile = profile(SoftTypeStrategy::GuardPredicates);
    input.declaration_projections = vec![declaration_projection(
        "pred-p",
        "p",
        AtpDeclarationKind::Predicate,
        1,
    )];
    input.soft_type_projections = vec![soft_guard(
        "guard",
        AtpFormulaTree::Atom(AtpAtom::new("p", Vec::new())),
    )];
    let error = translate_declarations(input).expect_err("arity mismatch");

    assert!(matches!(
        error,
        AtpTranslationError::Problem {
            source: AtpProblemError::InvalidSymbolArity { .. }
        }
    ));
}

#[test]
fn kind_mismatches_are_rejected_by_translator_signature_validation() {
    let set = fixture_set(VcStatus::NeedsAtp, "sample");
    let handoff = handoff(&set);
    let mut input = empty_input(&set, &handoff);
    input.logic_profile = profile(SoftTypeStrategy::GuardPredicates);
    input.declaration_projections = vec![declaration_projection(
        "fn-f",
        "f",
        AtpDeclarationKind::Function,
        0,
    )];
    input.soft_type_projections = vec![soft_guard(
        "guard",
        AtpFormulaTree::Atom(AtpAtom::new("f", Vec::new())),
    )];
    let error = translate_declarations(input).expect_err("kind mismatch");

    assert!(matches!(
        error,
        AtpTranslationError::Problem {
            source: AtpProblemError::InvalidSymbolDeclaration { .. }
        }
    ));
}

#[test]
fn guard_profiles_do_not_accept_sort_only_soft_type_projection() {
    let set = fixture_set(VcStatus::NeedsAtp, "sample");
    let handoff = handoff(&set);
    let mut input = empty_input(&set, &handoff);
    input.logic_profile = profile(SoftTypeStrategy::GuardPredicates);
    input.soft_type_projections = vec![AtpSoftTypeProjection {
        key: AtpProjectionKey::new("type-fact"),
        representation: AtpSoftTypeRepresentation::BackendSortLossless,
        provenance: type_provenance("type-fact"),
    }];
    let error = translate_declarations(input).expect_err("missing guard");

    assert!(matches!(
        error,
        AtpTranslationError::MissingSoftTypeGuard { .. }
    ));
}

#[test]
fn explicit_profile_is_not_changed_to_accept_quantified_guard() {
    let set = fixture_set(VcStatus::NeedsAtp, "sample");
    let handoff = handoff(&set);
    let mut input = empty_input(&set, &handoff);
    input.logic_profile = propositional_guard_profile();
    input.declaration_projections = vec![declaration_projection(
        "binder-x",
        "x",
        AtpDeclarationKind::GeneratedBinder,
        0,
    )];
    input.soft_type_projections = vec![soft_guard(
        "guard",
        AtpFormulaTree::Forall {
            binders: vec![AtpBinder::new("x", None)],
            body: Box::new(AtpFormulaTree::True),
        },
    )];
    let error = translate_declarations(input).expect_err("unsupported quantifier");

    assert!(matches!(
        error,
        AtpTranslationError::Problem {
            source: AtpProblemError::UnsupportedProfileFeature {
                feature: "quantifier"
            }
        }
    ));
}

#[test]
fn type_guard_symbol_sources_resolve_to_generated_guard_ids() {
    let set = fixture_set(VcStatus::NeedsAtp, "sample");
    let handoff = handoff(&set);
    let mut input = empty_input(&set, &handoff);
    input.logic_profile = profile(SoftTypeStrategy::GuardPredicates);
    input.declaration_projections = vec![AtpDeclarationProjection {
        key: AtpProjectionKey::new("type-predicate"),
        kind: AtpDeclarationKind::Predicate,
        symbol: AtpSymbolName::new("type_guard"),
        arity: 0,
        provenance: declaration_provenance("type-predicate"),
        symbol_source: AtpSymbolSourceProjection::TypeGuard(AtpProjectionKey::new("guard")),
    }];
    input.soft_type_projections = vec![soft_guard(
        "guard",
        AtpFormulaTree::Atom(AtpAtom::new("type_guard", Vec::new())),
    )];
    let output = translate_declarations(input).expect("translation");

    assert!(matches!(
        output.symbol_map()[0].source(),
        AtpSymbolSource::TypeGuard(id) if *id == AtpTypeGuardId::new(0)
    ));
    assert_eq!(output.type_context().guards().len(), 1);
    assert_eq!(
        output.type_context().guards()[0].formula(),
        &AtpFormulaTree::Atom(AtpAtom::new("type_guard", Vec::new()))
    );
    assert_eq!(
        output.provenance()[0].source(),
        &AtpSourceRef::TypeFact(AtpSourceBinding::new("type:guard"))
    );
    assert_eq!(
        output.provenance()[0].payload().as_str(),
        "type-payload:guard"
    );
}

#[test]
fn imported_projection_missing_required_status_fails_closed() {
    let set = fixture_set(VcStatus::NeedsAtp, "sample");
    let handoff = handoff(&set);
    let mut input = empty_input(&set, &handoff);
    input.declaration_projections = vec![AtpDeclarationProjection {
        key: AtpProjectionKey::new("imported"),
        kind: AtpDeclarationKind::Predicate,
        symbol: AtpSymbolName::new("imported"),
        arity: 0,
        provenance: AtpProjectionProvenance::new(
            AtpSourceRef::ImportedAxiom {
                package: AtpSourceBinding::new("pkg"),
                module: AtpSourceBinding::new("module"),
                item: AtpSourceBinding::new("item"),
                statement_fingerprint: AtpFingerprint::new(2, vec![1]).expect("fingerprint"),
                required_status: AtpRequiredProofStatus::new(""),
                context_requirement: AtpSourceBinding::new("ctx"),
            },
            "imported",
        ),
        symbol_source: AtpSymbolSourceProjection::MizarSymbol(AtpSourceBinding::new("imported")),
    }];
    let error = translate_declarations(input).expect_err("empty imported status");

    assert!(matches!(
        error,
        AtpTranslationError::Problem {
            source: AtpProblemError::EmptyField {
                field: "imported.required_status"
            }
        }
    ));
}

#[test]
fn imported_projection_missing_context_requirement_fails_closed() {
    let set = fixture_set(VcStatus::NeedsAtp, "sample");
    let handoff = handoff(&set);
    let mut input = empty_input(&set, &handoff);
    input.declaration_projections = vec![AtpDeclarationProjection {
        key: AtpProjectionKey::new("imported"),
        kind: AtpDeclarationKind::Predicate,
        symbol: AtpSymbolName::new("imported"),
        arity: 0,
        provenance: AtpProjectionProvenance::new(
            AtpSourceRef::ImportedAxiom {
                package: AtpSourceBinding::new("pkg"),
                module: AtpSourceBinding::new("module"),
                item: AtpSourceBinding::new("item"),
                statement_fingerprint: AtpFingerprint::new(2, vec![1]).expect("fingerprint"),
                required_status: AtpRequiredProofStatus::new("translator-fixture-verified"),
                context_requirement: AtpSourceBinding::new(""),
            },
            "imported",
        ),
        symbol_source: AtpSymbolSourceProjection::MizarSymbol(AtpSourceBinding::new("imported")),
    }];
    let error = translate_declarations(input).expect_err("empty imported context");

    assert!(matches!(
        error,
        AtpTranslationError::Problem {
            source: AtpProblemError::EmptyField {
                field: "imported.context_requirement"
            }
        }
    ));
}

#[test]
fn translates_axioms_and_conjecture_with_unsat_polarity() {
    let set = fixture_set(VcStatus::NeedsAtp, "sample");
    let hinted_handoff = handoff(&set);
    let mut input = basic_problem_input(&set, &hinted_handoff);
    input.formula_projections[0].provenance.payload =
        AtpPayload::new("caller-selected-axiom-provenance");
    input.formula_projections[1].provenance.payload =
        AtpPayload::new("caller-selected-goal-provenance");
    let problem = translate_problem(input).expect("problem");

    assert_eq!(problem.vc_id(), VcId::new(0));
    assert_eq!(problem.expected_result(), ExpectedBackendResult::Unsat);
    assert_eq!(problem.axioms().len(), 1);
    assert_eq!(
        problem.axioms()[0].formula(),
        Some(&AtpFormulaTree::Atom(AtpAtom::new("p", Vec::new())))
    );
    assert_eq!(problem.conjecture().formula(), Some(&AtpFormulaTree::False));
    assert_eq!(problem.properties(), []);
    assert_eq!(
        problem.provenance()[1].payload().as_str(),
        "mizar-vc-handoff-provenance:70726f76656e616e63652d30"
    );
    assert_eq!(
        problem.provenance()[2].payload().as_str(),
        "mizar-vc-handoff-provenance:70726f76656e616e63652d31"
    );
    assert!(problem.debug_text().contains("expected-result: Unsat"));
}

#[test]
fn shuffled_formula_projection_inputs_have_identical_problem_output() {
    let set = fixture_set(VcStatus::NeedsAtp, "sample");
    let handoff = handoff(&set);
    let first = basic_problem_input(&set, &handoff);
    let mut second = basic_problem_input(&set, &handoff);
    second.formula_projections.reverse();

    assert_eq!(
        translate_problem(first).expect("first").debug_text(),
        translate_problem(second).expect("second").debug_text()
    );
}

#[test]
fn translate_problem_rejects_non_needs_atp_and_mismatched_handoff() {
    let open_set = fixture_set(VcStatus::Open, "sample");
    let open_handoff = handoff(&open_set);
    assert!(matches!(
        translate_problem(basic_problem_input(&open_set, &open_handoff)).expect_err("non-NeedsAtp"),
        AtpTranslationError::NonNeedsAtpStatus {
            status: VcStatus::Open,
            ..
        }
    ));

    let handoff_set = fixture_set(VcStatus::NeedsAtp, "left");
    let input_set = fixture_set(VcStatus::NeedsAtp, "right");
    let stale_handoff = handoff(&handoff_set);
    assert!(matches!(
        translate_problem(basic_problem_input(&input_set, &stale_handoff))
            .expect_err("stale handoff"),
        AtpTranslationError::MismatchedTargetHandoff { .. }
    ));
}

#[test]
fn missing_malformed_and_duplicate_formula_projections_fail_closed() {
    let set = fixture_set(VcStatus::NeedsAtp, "sample");
    let handoff = handoff(&set);
    let mut missing_goal = basic_problem_input(&set, &handoff);
    missing_goal.formula_projections.pop();
    assert!(matches!(
        translate_problem(missing_goal).expect_err("missing goal projection"),
        AtpTranslationError::MissingFormulaProjection { .. }
    ));

    let mut duplicate_target = basic_problem_input(&set, &handoff);
    let mut duplicate_projection = local_formula_projection(AtpFormulaTree::True);
    duplicate_projection.source_identity = AtpProjectionKey::new("local-context:duplicate");
    duplicate_target
        .formula_projections
        .push(duplicate_projection);
    assert!(matches!(
        translate_problem(duplicate_target).expect_err("duplicate target"),
        AtpTranslationError::DuplicateFormulaProjection { .. }
    ));

    let mut duplicate_identity = basic_problem_input(&set, &handoff);
    duplicate_identity.formula_projections[1].source_identity = duplicate_identity
        .formula_projections[0]
        .source_identity
        .clone();
    assert!(matches!(
        translate_problem(duplicate_identity).expect_err("duplicate source identity"),
        AtpTranslationError::DuplicateFormulaProjectionIdentity { .. }
    ));

    let mut empty_handoff_payload = basic_problem_input(&set, &handoff);
    empty_handoff_payload.formula_projections[0]
        .handoff_provenance_payload
        .clear();
    assert!(matches!(
        translate_problem(empty_handoff_payload).expect_err("empty handoff payload"),
        AtpTranslationError::Problem {
            source: AtpProblemError::EmptyField {
                field: "formula_projection.handoff_provenance_payload"
            }
        }
    ));

    let mut empty_identity = basic_problem_input(&set, &handoff);
    empty_identity.formula_projections[0].source_identity = AtpProjectionKey::new("");
    assert!(matches!(
        translate_problem(empty_identity).expect_err("empty source identity"),
        AtpTranslationError::EmptyProjectionKey {
            section: "formula projection source identities"
        }
    ));

    let mut empty_provenance_payload = basic_problem_input(&set, &handoff);
    empty_provenance_payload.formula_projections[0]
        .provenance
        .payload = AtpPayload::new("");
    assert!(matches!(
        translate_problem(empty_provenance_payload).expect_err("empty formula provenance payload"),
        AtpTranslationError::Problem {
            source: AtpProblemError::EmptyField {
                field: "projection.provenance.payload"
            }
        }
    ));

    let mut empty_provenance_source = basic_problem_input(&set, &handoff);
    empty_provenance_source.formula_projections[0]
        .provenance
        .source = AtpSourceRef::LocalHypothesis(AtpSourceBinding::new(""));
    assert!(matches!(
        translate_problem(empty_provenance_source).expect_err("empty formula provenance source"),
        AtpTranslationError::Problem {
            source: AtpProblemError::EmptyField {
                field: "projection.provenance.source"
            }
        }
    ));

    let imported_set = fixture_set_with(
        VcStatus::NeedsAtp,
        "empty-imported-symbol",
        vec![PremiseRef::ImportedFact {
            symbol: VcText::new("Imported::A1"),
        }],
        None,
    );
    let imported_handoff = handoff_with_imported(&imported_set);
    let mut empty_symbol = imported_problem_input(&imported_set, &imported_handoff);
    empty_symbol.formula_projections[0].target =
        AtpFormulaProjectionTarget::ImportedFact(VcText::new(""));
    assert!(matches!(
        translate_problem(empty_symbol).expect_err("empty imported symbol"),
        AtpTranslationError::Problem {
            source: AtpProblemError::EmptyField {
                field: "formula_projection.imported_symbol"
            }
        }
    ));
}

#[test]
fn formula_handoff_fingerprint_mismatch_fails_closed() {
    let set = fixture_set(VcStatus::NeedsAtp, "sample");
    let base_handoff = handoff(&set);
    let mut input = basic_problem_input(&set, &base_handoff);
    input.formula_projections[0].handoff_formula_fingerprint =
        AtpFingerprint::new(KERNEL_FORMULA_FINGERPRINT_ALGORITHM_ID, b"wrong".to_vec())
            .expect("fingerprint");

    assert!(matches!(
        translate_problem(input).expect_err("handoff mismatch"),
        AtpTranslationError::FormulaHandoffAgreement { .. }
    ));

    let mut payload_mismatch = basic_problem_input(&set, &base_handoff);
    payload_mismatch.formula_projections[0].handoff_provenance_payload =
        b"wrong-provenance".to_vec();
    assert!(matches!(
        translate_problem(payload_mismatch).expect_err("handoff payload mismatch"),
        AtpTranslationError::FormulaHandoffAgreement { .. }
    ));

    let mut wrong_local_source = basic_problem_input(&set, &base_handoff);
    wrong_local_source.formula_projections[0].provenance.source =
        AtpSourceRef::LocalHypothesis(AtpSourceBinding::new("local-context:99"));
    assert!(matches!(
        translate_problem(wrong_local_source).expect_err("wrong local source binding"),
        AtpTranslationError::Problem {
            source: AtpProblemError::UnsupportedProfileFeature {
                feature: "formula provenance source class"
            }
        }
    ));

    let mut wrong_goal_source_class = basic_problem_input(&set, &base_handoff);
    wrong_goal_source_class.formula_projections[1]
        .provenance
        .source = AtpSourceRef::LocalHypothesis(AtpSourceBinding::new("goal:1"));
    assert!(matches!(
        translate_problem(wrong_goal_source_class).expect_err("wrong goal source class"),
        AtpTranslationError::Problem {
            source: AtpProblemError::UnsupportedProfileFeature {
                feature: "conjecture provenance source class"
            }
        }
    ));

    let mut checker_goal_source = basic_problem_input(&set, &base_handoff);
    checker_goal_source.formula_projections[1].provenance.source =
        AtpSourceRef::CheckerOwnedFact(AtpSourceBinding::new("goal:1"));
    assert!(matches!(
        translate_problem(checker_goal_source).expect_err("checker-owned goal source"),
        AtpTranslationError::Problem {
            source: AtpProblemError::UnsupportedProfileFeature {
                feature: "conjecture provenance source class"
            }
        }
    ));

    let mut wrong_goal_binding = basic_problem_input(&set, &base_handoff);
    wrong_goal_binding.formula_projections[1].provenance.source =
        AtpSourceRef::GeneratedVcFact(AtpSourceBinding::new("generated:2"));
    wrong_goal_binding.formula_projections[1].source_identity =
        AtpProjectionKey::new("generated:2");
    assert!(matches!(
        translate_problem(wrong_goal_binding).expect_err("wrong goal binding"),
        AtpTranslationError::Problem {
            source: AtpProblemError::UnsupportedProfileFeature {
                feature: "conjecture provenance source class"
            }
        }
    ));

    let mut wrong_goal_identity = basic_problem_input(&set, &base_handoff);
    wrong_goal_identity.formula_projections[1].source_identity =
        AtpProjectionKey::new("goal:spoof");
    assert!(matches!(
        translate_problem(wrong_goal_identity).expect_err("wrong goal source identity"),
        AtpTranslationError::Problem {
            source: AtpProblemError::UnsupportedProfileFeature {
                feature: "conjecture provenance source class"
            }
        }
    ));

    let mut wrong_goal_fingerprint = basic_problem_input(&set, &base_handoff);
    wrong_goal_fingerprint.formula_projections[1].handoff_formula_fingerprint =
        AtpFingerprint::new(
            KERNEL_FORMULA_FINGERPRINT_ALGORITHM_ID,
            b"wrong-goal".to_vec(),
        )
        .expect("fingerprint");
    assert!(matches!(
        translate_problem(wrong_goal_fingerprint).expect_err("wrong goal fingerprint"),
        AtpTranslationError::FormulaHandoffAgreement { .. }
    ));

    let mut wrong_goal_payload = basic_problem_input(&set, &base_handoff);
    wrong_goal_payload.formula_projections[1].handoff_provenance_payload =
        b"wrong-goal-provenance".to_vec();
    assert!(matches!(
        translate_problem(wrong_goal_payload).expect_err("wrong goal payload"),
        AtpTranslationError::FormulaHandoffAgreement { .. }
    ));

    let generated_set = fixture_set_with(
        VcStatus::NeedsAtp,
        "generated-source-mismatch",
        vec![
            PremiseRef::LocalContext(ContextEntryId::new(0)),
            PremiseRef::GeneratedFact {
                formula: VcFormulaRef::Generated(VcGeneratedFormulaId::new(2)),
            },
        ],
        None,
    );
    let generated_handoff = handoff(&generated_set);
    let mut wrong_generated_source = two_premise_problem_input(&generated_set, &generated_handoff);
    wrong_generated_source.formula_projections[2]
        .provenance
        .source = AtpSourceRef::GeneratedVcFact(AtpSourceBinding::new("generated:99"));
    wrong_generated_source.formula_projections[2].source_identity =
        AtpProjectionKey::new("generated:99");
    assert!(matches!(
        translate_problem(wrong_generated_source).expect_err("wrong generated source"),
        AtpTranslationError::Problem {
            source: AtpProblemError::UnsupportedProfileFeature {
                feature: "formula provenance source class"
            }
        }
    ));

    let cited_set = fixture_set_with_local_entry_kind(
        VcStatus::NeedsAtp,
        "cited-source-mismatch",
        vec![PremiseRef::LocalContext(ContextEntryId::new(0))],
        None,
        ContextEntryKind::CitedPremise,
    );
    let cited_handoff = handoff(&cited_set);
    let mut cited_input = basic_problem_input(&cited_set, &cited_handoff);
    cited_input.formula_projections[0] =
        cited_formula_projection(AtpFormulaTree::Atom(AtpAtom::new("p", Vec::new())));
    translate_problem(cited_input).expect("cited premise source");

    let mut wrong_cited_source = basic_problem_input(&cited_set, &cited_handoff);
    wrong_cited_source.formula_projections[0] =
        cited_formula_projection(AtpFormulaTree::Atom(AtpAtom::new("p", Vec::new())));
    wrong_cited_source.formula_projections[0].provenance.source =
        AtpSourceRef::CitedPremise(AtpSourceBinding::new("cited-premise:99"));
    wrong_cited_source.formula_projections[0].source_identity =
        AtpProjectionKey::new("cited-premise:99");
    assert!(matches!(
        translate_problem(wrong_cited_source).expect_err("wrong cited source"),
        AtpTranslationError::Problem {
            source: AtpProblemError::UnsupportedProfileFeature {
                feature: "formula provenance source class"
            }
        }
    ));
}

#[test]
fn unsupported_formula_profile_features_are_not_silently_reprofiled() {
    let set = fixture_set(VcStatus::NeedsAtp, "sample");
    let handoff = handoff(&set);
    let mut input = basic_problem_input(&set, &handoff);
    input.logic_profile = profile_without_equality();
    input.declaration_projections = vec![declaration_projection(
        "fn-a",
        "a",
        AtpDeclarationKind::Function,
        0,
    )];
    input.formula_projections[0].formula = AtpFormulaTree::Equality {
        left: AtpTerm::Function {
            function: AtpSymbolName::new("a"),
            arguments: Vec::new(),
        },
        right: AtpTerm::Function {
            function: AtpSymbolName::new("a"),
            arguments: Vec::new(),
        },
    };

    assert!(matches!(
        translate_problem(input).expect_err("unsupported equality"),
        AtpTranslationError::Problem {
            source: AtpProblemError::UnsupportedProfileFeature {
                feature: "equality"
            }
        }
    ));
}

#[test]
fn duplicate_premise_refs_and_formula_identities_fail_closed() {
    let duplicate_ref_set = fixture_set_with(
        VcStatus::NeedsAtp,
        "dup-ref",
        vec![
            PremiseRef::LocalContext(ContextEntryId::new(0)),
            PremiseRef::LocalContext(ContextEntryId::new(0)),
        ],
        None,
    );
    let duplicate_ref_handoff = handoff(&duplicate_ref_set);
    assert!(matches!(
        translate_problem(basic_problem_input(
            &duplicate_ref_set,
            &duplicate_ref_handoff
        ))
        .expect_err("duplicate premise ref"),
        AtpTranslationError::DuplicatePremiseRef { .. }
    ));

    let duplicate_formula_set = fixture_set_with(
        VcStatus::NeedsAtp,
        "dup-formula",
        vec![
            PremiseRef::LocalContext(ContextEntryId::new(0)),
            PremiseRef::GeneratedFact {
                formula: VcFormulaRef::Generated(VcGeneratedFormulaId::new(0)),
            },
        ],
        None,
    );
    let duplicate_formula_handoff = handoff(&duplicate_formula_set);
    assert!(matches!(
        translate_problem(basic_problem_input(
            &duplicate_formula_set,
            &duplicate_formula_handoff
        ))
        .expect_err("duplicate formula ref"),
        AtpTranslationError::DuplicatePremiseFormula { .. }
    ));
}

#[test]
fn premise_formula_must_not_copy_goal_into_axioms() {
    let generated_goal_premise_set = fixture_set_with(
        VcStatus::NeedsAtp,
        "goal-generated-premise",
        vec![PremiseRef::GeneratedFact {
            formula: VcFormulaRef::Generated(VcGeneratedFormulaId::new(1)),
        }],
        None,
    );
    let generated_goal_handoff = handoff(&generated_goal_premise_set);
    assert!(matches!(
        translate_problem(basic_problem_input(
            &generated_goal_premise_set,
            &generated_goal_handoff
        ))
        .expect_err("goal copied as generated premise"),
        AtpTranslationError::PremiseCopiesGoal { .. }
    ));

    let local_goal_premise_set = fixture_set_with_local_formula(
        VcStatus::NeedsAtp,
        "goal-local-premise",
        vec![PremiseRef::LocalContext(ContextEntryId::new(0))],
        None,
        VcFormulaRef::Generated(VcGeneratedFormulaId::new(1)),
    );
    let local_goal_handoff = handoff(&local_goal_premise_set);
    assert!(matches!(
        translate_problem(basic_problem_input(
            &local_goal_premise_set,
            &local_goal_handoff
        ))
        .expect_err("goal copied as local premise"),
        AtpTranslationError::PremiseCopiesGoal { .. }
    ));
}

#[test]
fn unsupported_checker_owned_and_type_predicate_premises_fail_closed() {
    for premise in [
        PremiseRef::CheckerFact {
            formula: mizar_core::core_ir::CoreFormulaId::new(0),
        },
        PremiseRef::TypePredicate {
            formula: mizar_core::core_ir::CoreFormulaId::new(0),
        },
    ] {
        let set = fixture_set(VcStatus::NeedsAtp, "unsupported");
        let vc = set.vc(VcId::new(0)).expect("vc");
        assert!(matches!(
            premise_projection_target(vc, &premise).expect_err("unsupported premise family"),
            AtpTranslationError::UnsupportedPremiseRef { .. }
        ));
    }

    let unsupported_set = fixture_set_with(
        VcStatus::NeedsAtp,
        "unsupported-public",
        vec![PremiseRef::CheckerFact {
            formula: mizar_core::core_ir::CoreFormulaId::new(0),
        }],
        None,
    );
    let valid_set = fixture_set(VcStatus::NeedsAtp, "unsupported-public");
    let valid_handoff = handoff(&valid_set);
    assert!(matches!(
        translate_problem(basic_problem_input(&unsupported_set, &valid_handoff))
            .expect_err("public translator boundary fails closed"),
        AtpTranslationError::MismatchedTargetHandoff { .. }
    ));
}

#[test]
fn generated_fact_premises_materialize_and_premise_order_is_canonical() {
    let first = fixture_set_with(
        VcStatus::NeedsAtp,
        "generated-order",
        vec![
            PremiseRef::GeneratedFact {
                formula: VcFormulaRef::Generated(VcGeneratedFormulaId::new(2)),
            },
            PremiseRef::LocalContext(ContextEntryId::new(0)),
        ],
        None,
    );
    let second = fixture_set_with(
        VcStatus::NeedsAtp,
        "generated-order",
        vec![
            PremiseRef::LocalContext(ContextEntryId::new(0)),
            PremiseRef::GeneratedFact {
                formula: VcFormulaRef::Generated(VcGeneratedFormulaId::new(2)),
            },
        ],
        None,
    );
    let first_handoff = handoff(&first);
    let second_handoff = handoff(&second);
    let first_problem =
        translate_problem(two_premise_problem_input(&first, &first_handoff)).expect("first");
    let second_problem =
        translate_problem(two_premise_problem_input(&second, &second_handoff)).expect("second");

    assert_eq!(
        first_problem
            .axioms()
            .iter()
            .map(|axiom| axiom.formula().cloned())
            .collect::<Vec<_>>(),
        second_problem
            .axioms()
            .iter()
            .map(|axiom| axiom.formula().cloned())
            .collect::<Vec<_>>()
    );
    assert_eq!(first_problem.problem_id(), second_problem.problem_id());
    assert_eq!(first_problem.debug_text(), second_problem.debug_text());
    assert!(first_problem.provenance().iter().any(|provenance| {
        matches!(
            provenance.source(),
            AtpSourceRef::GeneratedVcFact(binding) if binding.as_str() == "generated:3"
        )
    }));
}

#[test]
fn proof_hint_restrictions_do_not_prune_premises() {
    let set = fixture_set_with(
        VcStatus::NeedsAtp,
        "hinted",
        vec![PremiseRef::LocalContext(ContextEntryId::new(0))],
        Some(ProofHint {
            citations: vec![],
            unfold_requests: vec![],
            premise_restrictions: vec![PremiseRestriction::Exclude(vec![
                PremiseRef::LocalContext(ContextEntryId::new(0)),
            ])],
            solver: None,
            max_axioms: None,
            timeout: None,
            computation: None,
            provenance: vec![vc_provenance("hint")],
        }),
    );
    let hinted_handoff = handoff(&set);
    let problem = translate_problem(basic_problem_input(&set, &hinted_handoff)).expect("problem");

    assert_eq!(problem.axioms().len(), 1);
    assert_eq!(
        problem.axioms()[0].formula(),
        Some(&AtpFormulaTree::Atom(AtpAtom::new("p", Vec::new())))
    );

    let only_set = fixture_set_with(
        VcStatus::NeedsAtp,
        "hinted-only",
        vec![PremiseRef::LocalContext(ContextEntryId::new(0))],
        Some(ProofHint {
            citations: vec![],
            unfold_requests: vec![],
            premise_restrictions: vec![PremiseRestriction::Only(Vec::new())],
            solver: None,
            max_axioms: None,
            timeout: None,
            computation: None,
            provenance: vec![vc_provenance("hint-only")],
        }),
    );
    let only_handoff = handoff(&only_set);
    let only_problem =
        translate_problem(basic_problem_input(&only_set, &only_handoff)).expect("problem");
    assert_eq!(only_problem.axioms().len(), 1);
}

#[test]
fn imported_formula_projection_requires_imported_provenance_fields() {
    let set = fixture_set_with(
        VcStatus::NeedsAtp,
        "imported",
        vec![PremiseRef::ImportedFact {
            symbol: VcText::new("Imported::A1"),
        }],
        None,
    );
    let handoff = handoff_with_imported(&set);
    let mut input = imported_problem_input(&set, &handoff);
    if let AtpSourceRef::ImportedAxiom {
        required_status, ..
    } = &mut input.formula_projections[0].provenance.source
    {
        *required_status = AtpRequiredProofStatus::new("");
    }

    assert!(matches!(
        translate_problem(input).expect_err("missing imported status"),
        AtpTranslationError::Problem {
            source: AtpProblemError::EmptyField {
                field: "imported.required_status"
            }
        }
    ));

    let mut wrong_package = imported_problem_input(&set, &handoff);
    if let AtpSourceRef::ImportedAxiom { package, .. } =
        &mut wrong_package.formula_projections[0].provenance.source
    {
        *package = AtpSourceBinding::new("other-pkg");
    }
    assert!(matches!(
        translate_problem(wrong_package).expect_err("wrong imported package"),
        AtpTranslationError::MissingFormulaHandoffEvidence { .. }
    ));

    let mut wrong_module = imported_problem_input(&set, &handoff);
    if let AtpSourceRef::ImportedAxiom { module, .. } =
        &mut wrong_module.formula_projections[0].provenance.source
    {
        *module = AtpSourceBinding::new("other-module");
    }
    assert!(matches!(
        translate_problem(wrong_module).expect_err("wrong imported module"),
        AtpTranslationError::MissingFormulaHandoffEvidence { .. }
    ));

    let mut wrong_item = imported_problem_input(&set, &handoff);
    if let AtpSourceRef::ImportedAxiom { item, .. } =
        &mut wrong_item.formula_projections[0].provenance.source
    {
        *item = AtpSourceBinding::new("other-item");
    }
    assert!(matches!(
        translate_problem(wrong_item).expect_err("wrong imported item"),
        AtpTranslationError::MissingFormulaHandoffEvidence { .. }
    ));

    let mut wrong_statement = imported_problem_input(&set, &handoff);
    if let AtpSourceRef::ImportedAxiom {
        statement_fingerprint,
        ..
    } = &mut wrong_statement.formula_projections[0].provenance.source
    {
        *statement_fingerprint = AtpFingerprint::new(
            IMPORTED_STATEMENT_FINGERPRINT_ALGORITHM_ID,
            b"other".to_vec(),
        )
        .expect("fingerprint");
    }
    assert!(matches!(
        translate_problem(wrong_statement).expect_err("wrong imported statement"),
        AtpTranslationError::MissingFormulaHandoffEvidence { .. }
    ));

    let mut wrong_status = imported_problem_input(&set, &handoff);
    if let AtpSourceRef::ImportedAxiom {
        required_status, ..
    } = &mut wrong_status.formula_projections[0].provenance.source
    {
        *required_status = AtpRequiredProofStatus::new("DischargedBuiltin");
    }
    assert!(matches!(
        translate_problem(wrong_status).expect_err("wrong imported status"),
        AtpTranslationError::MissingFormulaHandoffEvidence { .. }
    ));

    let mut wrong_class = imported_problem_input(&set, &handoff);
    wrong_class.formula_projections[0].provenance.source = imported_theorem_source();
    assert!(matches!(
        translate_problem(wrong_class).expect_err("wrong imported class"),
        AtpTranslationError::MissingFormulaHandoffEvidence { .. }
    ));

    let mut wrong_imported_fingerprint = imported_problem_input(&set, &handoff);
    wrong_imported_fingerprint.formula_projections[0].handoff_formula_fingerprint =
        AtpFingerprint::new(
            KERNEL_FORMULA_FINGERPRINT_ALGORITHM_ID,
            b"wrong-imported".to_vec(),
        )
        .expect("fingerprint");
    assert!(matches!(
        translate_problem(wrong_imported_fingerprint)
            .expect_err("wrong imported handoff fingerprint"),
        AtpTranslationError::MissingFormulaHandoffEvidence { .. }
    ));

    let mut wrong_imported_payload = imported_problem_input(&set, &handoff);
    wrong_imported_payload.formula_projections[0].handoff_provenance_payload =
        b"wrong-imported-provenance".to_vec();
    assert!(matches!(
        translate_problem(wrong_imported_payload).expect_err("wrong imported handoff payload"),
        AtpTranslationError::MissingFormulaHandoffEvidence { .. }
    ));

    let mut missing_context = imported_problem_input(&set, &handoff);
    if let AtpSourceRef::ImportedAxiom {
        context_requirement,
        ..
    } = &mut missing_context.formula_projections[0].provenance.source
    {
        *context_requirement = AtpSourceBinding::new("");
    }
    assert!(matches!(
        translate_problem(missing_context).expect_err("missing imported context"),
        AtpTranslationError::Problem {
            source: AtpProblemError::EmptyField {
                field: "imported.context_requirement"
            }
        }
    ));

    let mut wrong_context = imported_problem_input(&set, &handoff);
    if let AtpSourceRef::ImportedAxiom {
        context_requirement,
        ..
    } = &mut wrong_context.formula_projections[0].provenance.source
    {
        *context_requirement = AtpSourceBinding::new("wrong-context");
    }
    assert!(matches!(
        translate_problem(wrong_context).expect_err("wrong imported context"),
        AtpTranslationError::Problem {
            source: AtpProblemError::UnsupportedProfileFeature {
                feature: "imported formula context requirement"
            }
        }
    ));
}

#[test]
fn imported_duplicate_source_tuple_fails_closed_across_symbols() {
    let set = fixture_set_with(
        VcStatus::NeedsAtp,
        "imported-duplicate-source",
        vec![
            PremiseRef::ImportedFact {
                symbol: VcText::new("Imported::A1"),
            },
            PremiseRef::ImportedFact {
                symbol: VcText::new("Imported::A2"),
            },
        ],
        None,
    );
    let handoff = handoff_with_imported_symbols(&set, &["Imported::A1", "Imported::A2"]);
    let mut input = imported_problem_input(&set, &handoff);
    let mut second = imported_formula_projection(
        "Imported::A2",
        AtpFormulaTree::Atom(AtpAtom::new("p", Vec::new())),
    );
    second.handoff_provenance_payload = imported_provenance_payload("Imported::A1").into_bytes();
    input.formula_projections.push(second);

    assert!(matches!(
        translate_problem(input).expect_err("duplicate imported source tuple"),
        AtpTranslationError::DuplicatePremiseIdentity { .. }
    ));
}

#[test]
fn imported_formula_materializes_with_handoff_agreement() {
    let set = fixture_set_with(
        VcStatus::NeedsAtp,
        "imported-ok",
        vec![PremiseRef::ImportedFact {
            symbol: VcText::new("Imported::A1"),
        }],
        None,
    );
    let handoff = handoff_with_imported(&set);
    let problem = translate_problem(imported_problem_input(&set, &handoff)).expect("problem");

    assert_eq!(problem.axioms().len(), 1);
    assert!(matches!(
        problem.provenance()[1].source(),
        AtpSourceRef::ImportedAxiom { .. }
    ));
}

#[test]
fn soft_type_guards_are_preserved_in_full_problem_translation() {
    let set = fixture_set(VcStatus::NeedsAtp, "soft-type");
    let handoff = handoff(&set);
    let mut input = basic_problem_input(&set, &handoff);
    input.logic_profile = profile(SoftTypeStrategy::GuardPredicates);
    input
        .declaration_projections
        .push(AtpDeclarationProjection {
            key: AtpProjectionKey::new("type-predicate"),
            kind: AtpDeclarationKind::Predicate,
            symbol: AtpSymbolName::new("type_guard"),
            arity: 0,
            provenance: declaration_provenance("type-predicate"),
            symbol_source: AtpSymbolSourceProjection::TypeGuard(AtpProjectionKey::new("guard")),
        });
    input.soft_type_projections = vec![soft_guard(
        "guard",
        AtpFormulaTree::Atom(AtpAtom::new("type_guard", Vec::new())),
    )];
    let problem = translate_problem(input).expect("problem");

    assert_eq!(problem.type_context().guards().len(), 1);
    assert_eq!(
        problem.type_context().guards()[0].formula(),
        &AtpFormulaTree::Atom(AtpAtom::new("type_guard", Vec::new()))
    );
}

fn empty_input<'a>(
    set: &'a VcSet,
    handoff: &'a VcKernelEvidenceHandoff,
) -> AtpDeclarationTranslationInput<'a> {
    AtpDeclarationTranslationInput {
        vc_set: set,
        vc: VcId::new(0),
        kernel_handoff: handoff,
        logic_profile: profile(SoftTypeStrategy::BackendSorts),
        declaration_projections: Vec::new(),
        soft_type_projections: Vec::new(),
        diagnostics: Vec::new(),
    }
}

fn basic_problem_input<'a>(
    set: &'a VcSet,
    handoff: &'a VcKernelEvidenceHandoff,
) -> AtpTranslationInput<'a> {
    AtpTranslationInput {
        vc_set: set,
        vc: VcId::new(0),
        kernel_handoff: handoff,
        logic_profile: profile(SoftTypeStrategy::BackendSorts),
        declaration_projections: vec![declaration_projection(
            "pred-p",
            "p",
            AtpDeclarationKind::Predicate,
            0,
        )],
        soft_type_projections: Vec::new(),
        formula_projections: vec![
            local_formula_projection(AtpFormulaTree::Atom(AtpAtom::new("p", Vec::new()))),
            goal_formula_projection(AtpFormulaTree::False),
        ],
        diagnostics: Vec::new(),
    }
}

fn imported_problem_input<'a>(
    set: &'a VcSet,
    handoff: &'a VcKernelEvidenceHandoff,
) -> AtpTranslationInput<'a> {
    AtpTranslationInput {
        vc_set: set,
        vc: VcId::new(0),
        kernel_handoff: handoff,
        logic_profile: profile(SoftTypeStrategy::BackendSorts),
        declaration_projections: vec![declaration_projection(
            "pred-p",
            "p",
            AtpDeclarationKind::Predicate,
            0,
        )],
        soft_type_projections: Vec::new(),
        formula_projections: vec![
            imported_formula_projection(
                "Imported::A1",
                AtpFormulaTree::Atom(AtpAtom::new("p", Vec::new())),
            ),
            goal_formula_projection(AtpFormulaTree::False),
        ],
        diagnostics: Vec::new(),
    }
}

fn two_premise_problem_input<'a>(
    set: &'a VcSet,
    handoff: &'a VcKernelEvidenceHandoff,
) -> AtpTranslationInput<'a> {
    let mut input = basic_problem_input(set, handoff);
    input.formula_projections.push(generated_formula_projection(
        2,
        AtpFormulaTree::Atom(AtpAtom::new("q", Vec::new())),
    ));
    input.declaration_projections.push(declaration_projection(
        "pred-q",
        "q",
        AtpDeclarationKind::Predicate,
        0,
    ));
    input
}

fn declaration_projection(
    key: &str,
    symbol: &str,
    kind: AtpDeclarationKind,
    arity: u32,
) -> AtpDeclarationProjection {
    AtpDeclarationProjection {
        key: AtpProjectionKey::new(key),
        kind,
        symbol: AtpSymbolName::new(symbol),
        arity,
        provenance: declaration_provenance(key),
        symbol_source: match kind {
            AtpDeclarationKind::GeneratedBinder => {
                AtpSymbolSourceProjection::GeneratedBinder(AtpSourceBinding::new(key))
            }
            _ => AtpSymbolSourceProjection::MizarSymbol(AtpSourceBinding::new(key)),
        },
    }
}

fn local_formula_projection(formula: AtpFormulaTree) -> AtpFormulaProjection {
    formula_projection(
        AtpFormulaProjectionTarget::VcFormula(VcFormulaRef::Generated(VcGeneratedFormulaId::new(
            0,
        ))),
        formula,
        AtpSourceRef::LocalHypothesis(AtpSourceBinding::new("local-context:1")),
        "local-context:1",
        "atp-provenance:local:0",
        b"formula-0",
        b"provenance-0",
    )
}

fn cited_formula_projection(formula: AtpFormulaTree) -> AtpFormulaProjection {
    formula_projection(
        AtpFormulaProjectionTarget::VcFormula(VcFormulaRef::Generated(VcGeneratedFormulaId::new(
            0,
        ))),
        formula,
        AtpSourceRef::CitedPremise(AtpSourceBinding::new("cited-premise:1")),
        "cited-premise:1",
        "atp-provenance:cited:0",
        b"formula-0",
        b"provenance-0",
    )
}

fn goal_formula_projection(formula: AtpFormulaTree) -> AtpFormulaProjection {
    formula_projection(
        AtpFormulaProjectionTarget::VcFormula(VcFormulaRef::Generated(VcGeneratedFormulaId::new(
            1,
        ))),
        formula,
        AtpSourceRef::GeneratedVcFact(AtpSourceBinding::new("goal:1")),
        "goal:1",
        "atp-provenance:goal:1",
        b"formula-1",
        b"provenance-1",
    )
}

fn generated_formula_projection(index: usize, formula: AtpFormulaTree) -> AtpFormulaProjection {
    let source_index = index + 1;
    formula_projection(
        AtpFormulaProjectionTarget::VcFormula(VcFormulaRef::Generated(VcGeneratedFormulaId::new(
            index,
        ))),
        formula,
        AtpSourceRef::GeneratedVcFact(AtpSourceBinding::new(format!("generated:{source_index}"))),
        format!("generated:{source_index}"),
        format!("atp-provenance:generated:{index}"),
        format!("formula-{index}").as_bytes(),
        format!("provenance-{index}").as_bytes(),
    )
}

fn imported_formula_projection(symbol: &str, formula: AtpFormulaTree) -> AtpFormulaProjection {
    formula_projection(
        AtpFormulaProjectionTarget::ImportedFact(VcText::new(symbol)),
        formula,
        AtpSourceRef::ImportedAxiom {
            package: AtpSourceBinding::new("pkg"),
            module: AtpSourceBinding::new("module"),
            item: AtpSourceBinding::new("item"),
            statement_fingerprint: AtpFingerprint::new(
                IMPORTED_STATEMENT_FINGERPRINT_ALGORITHM_ID,
                b"statement".to_vec(),
            )
            .expect("statement fingerprint"),
            required_status: AtpRequiredProofStatus::new("KernelVerified"),
            context_requirement: AtpSourceBinding::new(fixture_formula_context_binding()),
        },
        format!("imported:{symbol}"),
        format!("atp-provenance:imported:{symbol}"),
        imported_fingerprint_digest(symbol).as_bytes(),
        imported_provenance_payload(symbol).as_bytes(),
    )
}

fn imported_theorem_source() -> AtpSourceRef {
    AtpSourceRef::ImportedTheorem {
        package: AtpSourceBinding::new("pkg"),
        module: AtpSourceBinding::new("module"),
        item: AtpSourceBinding::new("item"),
        statement_fingerprint: AtpFingerprint::new(
            IMPORTED_STATEMENT_FINGERPRINT_ALGORITHM_ID,
            b"statement".to_vec(),
        )
        .expect("statement fingerprint"),
        required_status: AtpRequiredProofStatus::new("KernelVerified"),
        context_requirement: AtpSourceBinding::new(fixture_formula_context_binding()),
    }
}

fn formula_projection(
    target: AtpFormulaProjectionTarget,
    formula: AtpFormulaTree,
    source: AtpSourceRef,
    source_identity: impl Into<AtpProjectionKey>,
    provenance_payload: impl Into<AtpPayload>,
    fingerprint_digest: &[u8],
    handoff_provenance_payload: &[u8],
) -> AtpFormulaProjection {
    AtpFormulaProjection {
        target,
        formula,
        provenance: AtpProjectionProvenance::new(source, provenance_payload),
        source_identity: source_identity.into(),
        handoff_formula_fingerprint: AtpFingerprint::new(
            KERNEL_FORMULA_FINGERPRINT_ALGORITHM_ID,
            fingerprint_digest.to_vec(),
        )
        .expect("formula fingerprint"),
        handoff_provenance_payload: handoff_provenance_payload.to_vec(),
    }
}

fn soft_guard(key: &str, formula: AtpFormulaTree) -> AtpSoftTypeProjection {
    AtpSoftTypeProjection {
        key: AtpProjectionKey::new(key),
        representation: AtpSoftTypeRepresentation::GuardFormula(formula),
        provenance: type_provenance(key),
    }
}

fn declaration_provenance(key: &str) -> AtpProjectionProvenance {
    AtpProjectionProvenance::new(
        AtpSourceRef::GeneratedVcFact(AtpSourceBinding::new(format!("decl:{key}"))),
        format!("decl-payload:{key}"),
    )
}

fn type_provenance(key: &str) -> AtpProjectionProvenance {
    AtpProjectionProvenance::new(
        AtpSourceRef::TypeFact(AtpSourceBinding::new(format!("type:{key}"))),
        format!("type-payload:{key}"),
    )
}

fn profile(soft_types: SoftTypeStrategy) -> LogicProfile {
    LogicProfile::try_new(
        "fof",
        LogicFragment::Fof,
        EqualitySupport::Supported,
        QuantifierPolicy::FirstOrder,
        soft_types,
        NativePropertySupport::Unsupported,
        BTreeSet::from([ConcreteFormat::Tptp]),
    )
    .expect("logic profile")
}

fn profile_without_equality() -> LogicProfile {
    LogicProfile::try_new(
        "fof-no-equality",
        LogicFragment::Fof,
        EqualitySupport::Unsupported,
        QuantifierPolicy::FirstOrder,
        SoftTypeStrategy::BackendSorts,
        NativePropertySupport::Unsupported,
        BTreeSet::from([ConcreteFormat::Tptp]),
    )
    .expect("logic profile")
}

fn propositional_guard_profile() -> LogicProfile {
    LogicProfile::try_new(
        "propositional-guards",
        LogicFragment::Fof,
        EqualitySupport::Supported,
        QuantifierPolicy::PropositionalOnly,
        SoftTypeStrategy::GuardPredicates,
        NativePropertySupport::Unsupported,
        BTreeSet::from([ConcreteFormat::Tptp]),
    )
    .expect("logic profile")
}

fn handoff(set: &VcSet) -> VcKernelEvidenceHandoff {
    let payloads = formula_payloads(set);
    build_kernel_evidence_handoff(KernelEvidenceHandoffInput {
        vc_set: set,
        vc: VcId::new(0),
        goal_polarity: KernelGoalPolarity::AssertFalseForRefutation,
        kernel_profile: KernelEvidenceProfile::v1(1, KernelClauseTautologyPolicy::Reject),
        symbol_manifest: &[],
        variable_manifest: &[],
        formula_payloads: &payloads,
        imported_formula_payloads: &[],
        substitutions: &[],
        formula_context: None,
        discharge_output: None,
    })
    .expect("kernel handoff")
}

fn handoff_with_imported(set: &VcSet) -> VcKernelEvidenceHandoff {
    handoff_with_imported_symbols(set, &["Imported::A1"])
}

fn handoff_with_imported_symbols(set: &VcSet, symbols: &[&str]) -> VcKernelEvidenceHandoff {
    let payloads = formula_payloads(set);
    let imported_payloads = symbols
        .iter()
        .map(|symbol| imported_payload_with_projection_source(&VcText::new(*symbol), symbols[0]))
        .collect::<Vec<_>>();
    let context = imported_context_for_payloads(&imported_payloads);
    build_kernel_evidence_handoff(KernelEvidenceHandoffInput {
        vc_set: set,
        vc: VcId::new(0),
        goal_polarity: KernelGoalPolarity::AssertFalseForRefutation,
        kernel_profile: KernelEvidenceProfile::v1(1, KernelClauseTautologyPolicy::Reject),
        symbol_manifest: &[],
        variable_manifest: &[],
        formula_payloads: &payloads,
        imported_formula_payloads: &imported_payloads,
        substitutions: &[],
        formula_context: Some(&context),
        discharge_output: None,
    })
    .expect("kernel handoff")
}

fn formula_payloads(set: &VcSet) -> Vec<KernelFormulaPayload> {
    set.generated_formulas()
        .iter()
        .map(|formula| KernelFormulaPayload {
            formula_ref: VcFormulaRef::Generated(formula.id),
            projection: KernelFormulaProjection {
                formula_fingerprint: fingerprint(
                    KERNEL_FORMULA_FINGERPRINT_ALGORITHM_ID,
                    format!("formula-{}", formula.id.index()).as_bytes(),
                ),
                formula_bytes: format!("kernel-formula-{}", formula.id.index()).into_bytes(),
                provenance_payload: format!("provenance-{}", formula.id.index()).into_bytes(),
            },
        })
        .collect()
}

fn imported_payload(symbol: &VcText) -> KernelImportedFormulaPayload {
    imported_payload_with_projection_source(symbol, symbol.as_str())
}

fn imported_payload_with_projection_source(
    symbol: &VcText,
    projection_source: &str,
) -> KernelImportedFormulaPayload {
    let statement_fingerprint =
        fingerprint(IMPORTED_STATEMENT_FINGERPRINT_ALGORITHM_ID, b"statement");
    let formula_fingerprint = fingerprint(
        KERNEL_FORMULA_FINGERPRINT_ALGORITHM_ID,
        imported_fingerprint_digest(projection_source).as_bytes(),
    );
    KernelImportedFormulaPayload {
        symbol: symbol.clone(),
        class: KernelImportedFormulaClass::Axiom,
        requirement: imported_requirement(),
        projection: KernelFormulaProjection {
            formula_fingerprint: formula_fingerprint.clone(),
            formula_bytes: format!("imported-formula-{projection_source}").into_bytes(),
            provenance_payload: imported_provenance_payload(projection_source).into_bytes(),
        },
        statement_projection: KernelImportedStatementProjection {
            statement_fingerprint: statement_fingerprint.clone(),
            formula_fingerprint: formula_fingerprint.clone(),
            payload: canonical_imported_statement_projection_payload(
                &statement_fingerprint,
                &formula_fingerprint,
            )
            .expect("canonical imported statement projection payload"),
        },
    }
}

fn imported_context(payload: &KernelImportedFormulaPayload) -> KernelFormulaContextRequirements {
    imported_context_for_payloads(std::slice::from_ref(payload))
}

fn imported_context_for_payloads(
    payloads: &[KernelImportedFormulaPayload],
) -> KernelFormulaContextRequirements {
    let mut imported_axioms = Vec::new();
    let mut imported_theorems = Vec::new();
    for payload in payloads {
        match payload.class {
            KernelImportedFormulaClass::Axiom => {
                imported_axioms.push(payload.requirement.clone());
            }
            KernelImportedFormulaClass::Theorem => {
                imported_theorems.push(payload.requirement.clone());
            }
            _ => panic!("unsupported imported formula class in fixture"),
        }
    }
    KernelFormulaContextRequirements {
        provenance_fingerprint: fingerprint(
            KERNEL_FORMULA_FINGERPRINT_ALGORITHM_ID,
            b"imported-context",
        ),
        imported_axioms,
        imported_theorems,
    }
}

fn fixture_formula_context_binding() -> String {
    let payload = imported_payload(&VcText::new("Imported::A1"));
    formula_context_binding(&imported_context(&payload))
}

fn imported_requirement() -> KernelImportedFactRequirement {
    KernelImportedFactRequirement {
        imported_fact_id: 0,
        package_id: b"pkg".to_vec(),
        module_path: b"module".to_vec(),
        exported_item_id: b"item".to_vec(),
        statement_fingerprint: fingerprint(
            IMPORTED_STATEMENT_FINGERPRINT_ALGORITHM_ID,
            b"statement",
        ),
        required_proof_status: KernelRequiredProofStatus::KernelVerified,
    }
}

fn imported_fingerprint_digest(_symbol: &str) -> String {
    "statement".to_owned()
}

fn imported_provenance_payload(symbol: &str) -> String {
    format!("imported-provenance-{symbol}")
}

fn fixture_set(status: VcStatus, module: &str) -> VcSet {
    fixture_set_with(
        status,
        module,
        vec![PremiseRef::LocalContext(ContextEntryId::new(0))],
        None,
    )
}

fn fixture_set_with(
    status: VcStatus,
    module: &str,
    premises: Vec<PremiseRef>,
    proof_hint: Option<ProofHint>,
) -> VcSet {
    fixture_set_with_local_formula(
        status,
        module,
        premises,
        proof_hint,
        VcFormulaRef::Generated(VcGeneratedFormulaId::new(0)),
    )
}

fn fixture_set_with_local_formula(
    status: VcStatus,
    module: &str,
    premises: Vec<PremiseRef>,
    proof_hint: Option<ProofHint>,
    local_formula: VcFormulaRef,
) -> VcSet {
    fixture_set_with_local_formula_and_kind(
        status,
        module,
        premises,
        proof_hint,
        local_formula,
        ContextEntryKind::ProofAssumption,
    )
}

fn fixture_set_with_local_entry_kind(
    status: VcStatus,
    module: &str,
    premises: Vec<PremiseRef>,
    proof_hint: Option<ProofHint>,
    local_kind: ContextEntryKind,
) -> VcSet {
    fixture_set_with_local_formula_and_kind(
        status,
        module,
        premises,
        proof_hint,
        VcFormulaRef::Generated(VcGeneratedFormulaId::new(0)),
        local_kind,
    )
}

fn fixture_set_with_local_formula_and_kind(
    status: VcStatus,
    module: &str,
    premises: Vec<PremiseRef>,
    proof_hint: Option<ProofHint>,
    local_formula: VcFormulaRef,
    local_kind: ContextEntryKind,
) -> VcSet {
    let snapshot = BuildSnapshotId::from_published_schema_str(
        "mizar-session-build-snapshot-v1:\
         3333333333333333333333333333333333333333333333333333333333333333",
    )
    .expect("snapshot id");
    let source = InMemorySessionIdAllocator::new()
        .next_source_id(snapshot)
        .expect("source id");
    let generated_formulas = vec![
        VcGeneratedFormula {
            id: VcGeneratedFormulaId::new(0),
            kind: VcGeneratedFormulaKind::GeneratedTypeObligation,
            shape: VcGeneratedFormulaShape::True,
            provenance: vec![vc_provenance("generated-0")],
        },
        VcGeneratedFormula {
            id: VcGeneratedFormulaId::new(1),
            kind: VcGeneratedFormulaKind::SplitGoal,
            shape: VcGeneratedFormulaShape::False,
            provenance: vec![vc_provenance("generated-1")],
        },
        VcGeneratedFormula {
            id: VcGeneratedFormulaId::new(2),
            kind: VcGeneratedFormulaKind::Conjunction,
            shape: VcGeneratedFormulaShape::True,
            provenance: vec![vc_provenance("generated-2")],
        },
    ];
    VcSet::try_new(VcSetParts {
        schema_version: VcSchemaVersion::new("atp-translator-test-v1"),
        snapshot,
        source,
        module: VcModuleRef::new(module),
        generated_formulas,
        vcs: vec![VcIr {
            id: VcId::new(0),
            kind: VcKind::TheoremProofStep,
            source: VcSourceRef {
                primary: source_ref(source),
                related: Vec::new(),
            },
            seed: mizar_vc::vc_ir::SeedVcRef {
                handoff: ObligationHandoffId::new(0),
            },
            anchor: incomplete_anchor(source),
            local_context: LocalContext::try_new(
                vec![ContextEntry {
                    id: ContextEntryId::new(0),
                    sort_key: CanonicalSortKey::new("000-local"),
                    kind: local_kind,
                    formula: Some(local_formula),
                    provenance: vec![vc_provenance("local")],
                }],
                Vec::new(),
            )
            .expect("context"),
            premises,
            goal: VcFormulaRef::Generated(VcGeneratedFormulaId::new(1)),
            proof_hint,
            status,
            provenance: vec![vc_provenance("vc")],
        }],
        seed_accounting: vec![SeedAccounting {
            handoff: ObligationHandoffId::new(0),
            origin: SeedOriginRef::ExistingCore {
                seed: ObligationSeedId::new(0),
            },
            seed_status: ObligationSeedStatus::Active,
            mapping: SeedVcMapping::One { vc: VcId::new(0) },
        }],
    })
    .expect("vc set")
}

fn incomplete_anchor(source: SourceId) -> mizar_vc::vc_ir::ObligationAnchor {
    mizar_vc::vc_ir::ObligationAnchor {
        owner: AnchorOwner::Theorem(CoreItemId::new(0)),
        kind: VcKind::TheoremProofStep,
        local_path: LocalProofOrProgramPath::new("proof/0"),
        label: Some(AnchorLabel {
            role: AnchorLabelRole::UserLabel,
            hint: Some(CoreLabelRef::new("A1")),
        }),
        semantic_origin: NormalizedSemanticOrigin::new("theorem:sample"),
        source_range: Some(SourceRange {
            source_id: source,
            start: 0,
            end: 4,
        }),
        provenance: vec![vc_provenance("anchor")],
        source_shape_hash: HashMarker::Unavailable {
            reason: AnchorUnavailableReason::new("test fixture"),
        },
        canonical_goal_hash: HashMarker::Unavailable {
            reason: AnchorUnavailableReason::new("test fixture"),
        },
        canonical_context_hash: HashMarker::Unavailable {
            reason: AnchorUnavailableReason::new("test fixture"),
        },
        generation_schema_version: GenerationSchemaVersion::new("atp-translator-test"),
        completeness: AnchorCompleteness::Incomplete {
            missing: vec![
                AnchorIngredient::SourceShapeHash,
                AnchorIngredient::CanonicalGoalHash,
                AnchorIngredient::CanonicalContextHash,
            ],
        },
    }
}

fn source_ref(source: SourceId) -> CoreSourceRef {
    CoreSourceRef::direct(SourceRange {
        source_id: source,
        start: 0,
        end: 4,
    })
    .with_provenance(vec![CoreProvenance::new(
        CoreProvenancePhase::ProofSkeleton,
        "atp-translator-test",
    )])
}

fn vc_provenance(key: &str) -> VcProvenance {
    VcProvenance {
        phase: VcProvenancePhase::Generator,
        key: VcText::new(key),
        core: None,
    }
}

fn fingerprint(algorithm_id: u8, digest: &[u8]) -> KernelEvidenceFingerprint {
    KernelEvidenceFingerprint::new(algorithm_id, digest.to_vec()).expect("fingerprint")
}
