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
        KernelClauseTautologyPolicy, KernelEvidenceFingerprint, KernelEvidenceHandoffError,
        KernelEvidenceHandoffInput, KernelEvidenceProfile, KernelFormulaContextRequirements,
        KernelFormulaPayload, KernelFormulaProjection, KernelGoalPolarity,
        KernelImportedFactRequirement, KernelImportedFormulaClass, KernelImportedFormulaPayload,
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
fn task_29_consistency_polarity_proof_obligation_is_not_translated() {
    let set = fixture_set(VcStatus::NeedsAtp, "task-29-polarity");
    let payloads = formula_payloads(&set);

    let error = build_kernel_evidence_handoff(KernelEvidenceHandoffInput {
        vc_set: &set,
        vc: VcId::new(0),
        goal_polarity: KernelGoalPolarity::AssertTrueForConsistency,
        kernel_profile: KernelEvidenceProfile::v1(1, KernelClauseTautologyPolicy::Reject),
        symbol_manifest: &[],
        variable_manifest: &[],
        formula_payloads: &payloads,
        imported_formula_payloads: &[],
        substitutions: &[],
        formula_context: None,
        discharge_output: None,
    })
    .expect_err("consistency polarity proof obligation must not reach ATP translation");

    assert!(matches!(
        error,
        KernelEvidenceHandoffError::GoalPolarityMismatch {
            vc,
            requested: KernelGoalPolarity::AssertTrueForConsistency,
            required: KernelGoalPolarity::AssertFalseForRefutation,
        } if vc == VcId::new(0)
    ));
}

#[test]
fn task_29_relabelled_local_hypothesis_is_not_translated() {
    let set = fixture_set(VcStatus::NeedsAtp, "task-29-local-source");
    let handoff = handoff(&set);
    let mut input = basic_problem_input(&set, &handoff);
    input.formula_projections[0].provenance.source =
        AtpSourceRef::LocalHypothesis(AtpSourceBinding::new("local-context:99"));
    input.formula_projections[0].source_identity = AtpProjectionKey::new("local-context:99");

    assert!(matches!(
        translate_problem(input).expect_err("relabelled local hypothesis must fail closed"),
        AtpTranslationError::Problem {
            source: AtpProblemError::UnsupportedProfileFeature {
                feature: "formula provenance source class"
            }
        }
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

#[test]
fn instance_finder_emits_kernel_accepted_evidence_for_fol_and_both_formats() {
    use crate::{backend::EncodedBackendProblem, instance_finder::find_instance_evidence};
    use mizar_kernel::formula_evidence::Formula;

    let x = AtpTerm::Variable("x".into());
    let a = AtpTerm::Function {
        function: "a".into(),
        arguments: Vec::new(),
    };
    let p_x = AtpFormulaTree::Atom(AtpAtom::new("p", vec![x.clone()]));
    let q_x = AtpFormulaTree::Atom(AtpAtom::new("q", vec![x]));
    let p_a = AtpFormulaTree::Atom(AtpAtom::new("p", vec![a.clone()]));
    let q_a = AtpFormulaTree::Atom(AtpAtom::new("q", vec![a]));
    let premise_tree = AtpFormulaTree::Forall {
        binders: vec![AtpBinder::new("x", None)],
        body: Box::new(AtpFormulaTree::Implies(Box::new(p_x), Box::new(q_x))),
    };
    let premise_formula = kernel_or(vec![
        Formula::Not(Box::new(kernel_atom(1, vec![kernel_var(1)]))),
        kernel_atom(2, vec![kernel_var(1)]),
    ]);
    let second_formula = kernel_atom(1, vec![kernel_const(4)]);
    let goal_formula = kernel_atom(2, vec![kernel_const(4)]);
    let (problem, handoff) = finder_problem(
        (premise_tree, premise_formula),
        Some((p_a, second_formula)),
        (q_a, goal_formula),
        vec![
            finder_symbol(0, 1, 1),
            finder_symbol(1, 1, 2),
            finder_symbol(2, 2, 4),
        ],
        vec![1],
        vec![
            declaration_projection("p", "p", AtpDeclarationKind::Predicate, 1),
            declaration_projection("q", "q", AtpDeclarationKind::Predicate, 1),
            declaration_projection("a", "a", AtpDeclarationKind::Function, 0),
            declaration_projection("x", "x", AtpDeclarationKind::GeneratedBinder, 0),
        ],
    );
    let tptp = EncodedBackendProblem::encode(&problem, ConcreteFormat::Tptp).expect("TPTP");
    let smt_problem = reprofile_finder_problem(&problem, ConcreteFormat::SmtLib);
    let smt = EncodedBackendProblem::encode(&smt_problem, ConcreteFormat::SmtLib).expect("SMT");
    let first = find_instance_evidence(&problem, &tptp, &handoff, Some([1, 2, 0, 2]), None)
        .expect("TPTP search")
        .expect("TPTP evidence");
    let second = find_instance_evidence(&smt_problem, &smt, &handoff, Some([1, 2, 0, 2]), None)
        .expect("SMT search")
        .expect("SMT evidence");
    assert_eq!(first.payload(), second.payload());
    assert_eq!(first.formula_label_refs(), tptp.formula_labels());
    assert_eq!(second.formula_label_refs(), smt.formula_labels());
    assert_kernel_accepts_instance_candidate(&first, &handoff);
    assert_kernel_accepts_instance_candidate(&second, &handoff);
    let repeated = find_instance_evidence(&problem, &tptp, &handoff, Some([1, 2, 0, 2]), None)
        .expect("repeated search")
        .expect("repeated evidence");
    assert_eq!(first, repeated);

    let selected_labels = [
        format!("ax_{}", problem.axioms()[0].id().index()),
        format!("ax_{}", problem.axioms()[1].id().index()),
    ];
    let reported = find_instance_evidence(
        &problem,
        &tptp,
        &handoff,
        Some([1, 2, 0, 2]),
        Some(&selected_labels),
    )
    .expect("valid premise report")
    .expect("reported premises prove goal");
    assert_eq!(reported.formula_label_refs(), tptp.formula_labels());
    assert_kernel_accepts_instance_candidate(&reported, &handoff);
    let only_ground_fact = [selected_labels[1].clone()];
    assert!(
        find_instance_evidence(
            &problem,
            &tptp,
            &handoff,
            Some([1, 2, 0, 2]),
            Some(&only_ground_fact),
        )
        .expect("report omits universal premise")
        .is_none()
    );
}

#[test]
fn instance_finder_limits_stale_inputs_and_premise_reports_fail_closed() {
    use crate::{backend::EncodedBackendProblem, instance_finder::find_instance_evidence};
    let a = AtpTerm::Function {
        function: "a".into(),
        arguments: Vec::new(),
    };
    let p = AtpFormulaTree::Atom(AtpAtom::new("p", vec![a.clone()]));
    let pa = kernel_atom(1, vec![kernel_const(4)]);
    let (problem, handoff) = finder_problem(
        (p.clone(), pa.clone()),
        Some((p.clone(), pa.clone())),
        (p, pa),
        vec![finder_symbol(0, 1, 1), finder_symbol(1, 2, 4)],
        Vec::new(),
        vec![
            declaration_projection("p", "p", AtpDeclarationKind::Predicate, 1),
            declaration_projection("a", "a", AtpDeclarationKind::Function, 0),
        ],
    );
    let encoded = EncodedBackendProblem::encode(&problem, ConcreteFormat::Tptp).expect("encode");

    assert!(
        find_instance_evidence(&problem, &encoded, &handoff, None, None)
            .expect("unconfigured")
            .is_none()
    );
    assert!(
        find_instance_evidence(&problem, &encoded, &handoff, Some([0, 2, 0, 1]), None)
            .expect("ground-term limit")
            .is_none()
    );
    assert!(
        find_instance_evidence(&problem, &encoded, &handoff, Some([1, 1, 0, 1]), None)
            .expect("premise-instance limit")
            .is_none()
    );
    assert!(
        find_instance_evidence(&problem, &encoded, &handoff, Some([1, 2, 0, 0]), None)
            .expect("SAT-check limit")
            .is_none()
    );
    let exact = find_instance_evidence(&problem, &encoded, &handoff, Some([1, 2, 0, 1]), None)
        .expect("exact bounds")
        .expect("exact bounds include the proof");
    assert_kernel_accepts_instance_candidate(&exact, &handoff);
    assert!(
        find_instance_evidence(&problem, &encoded, &handoff, Some([0, 0, 0, 0]), None)
            .expect("literal zero")
            .is_none()
    );

    let quantified = AtpFormulaTree::Forall {
        binders: vec![AtpBinder::new("x", None)],
        body: Box::new(AtpFormulaTree::Atom(AtpAtom::new(
            "p",
            vec![AtpTerm::Variable("x".into())],
        ))),
    };
    let (no_terms_problem, no_terms_handoff) = finder_problem(
        (quantified, kernel_atom(1, vec![kernel_var(1)])),
        None,
        (
            AtpFormulaTree::Atom(AtpAtom::new("q", Vec::new())),
            kernel_atom(2, Vec::new()),
        ),
        vec![finder_symbol(0, 1, 1), finder_symbol(1, 1, 2)],
        vec![1],
        vec![
            declaration_projection("p", "p", AtpDeclarationKind::Predicate, 1),
            declaration_projection("q", "q", AtpDeclarationKind::Predicate, 0),
            declaration_projection("x", "x", AtpDeclarationKind::GeneratedBinder, 0),
        ],
    );
    let no_terms_encoded = EncodedBackendProblem::encode(&no_terms_problem, ConcreteFormat::Tptp)
        .expect("ground-free encoded problem");
    assert!(
        find_instance_evidence(
            &no_terms_problem,
            &no_terms_encoded,
            &no_terms_handoff,
            Some([10, 10, 10, 10]),
            None,
        )
        .expect("no observed ground terms")
        .is_none()
    );

    let stale_problem = finder_problem(
        (
            AtpFormulaTree::Atom(AtpAtom::new("p", Vec::new())),
            kernel_atom(1, Vec::new()),
        ),
        None,
        (
            AtpFormulaTree::Atom(AtpAtom::new("q", Vec::new())),
            kernel_atom(2, Vec::new()),
        ),
        vec![finder_symbol(0, 1, 1), finder_symbol(1, 1, 2)],
        Vec::new(),
        vec![
            declaration_projection("p", "p", AtpDeclarationKind::Predicate, 0),
            declaration_projection("q", "q", AtpDeclarationKind::Predicate, 0),
        ],
    );
    let stale_encoded = EncodedBackendProblem::encode(&stale_problem.0, ConcreteFormat::Tptp)
        .expect("stale problem encoding");
    assert!(
        find_instance_evidence(&problem, &stale_encoded, &handoff, Some([0, 2, 0, 1]), None)
            .is_err()
    );
    assert!(
        find_instance_evidence(
            &problem,
            &encoded,
            &stale_problem.1,
            Some([0, 2, 0, 1]),
            None
        )
        .is_err()
    );

    let mut mismatched_parts = AtpProblemParts {
        vc_id: problem.vc_id(),
        target_binding: problem.target_binding().clone(),
        logic_profile: problem.logic_profile().clone(),
        expected_result: problem.expected_result(),
        declarations: problem.declarations().to_vec(),
        axioms: problem.axioms().to_vec(),
        conjecture: problem.conjecture().clone(),
        type_context: problem.type_context().clone(),
        properties: problem.properties().to_vec(),
        symbol_map: problem.symbol_map().to_vec(),
        provenance: problem.provenance().to_vec(),
        diagnostics: problem.diagnostics().to_vec(),
    };
    let guard = &problem.type_context().guards()[0];
    mismatched_parts.type_context = AtpTypeContext::new(vec![AtpTypeGuard::new(
        guard.id(),
        AtpFormulaTree::True,
        guard.provenance(),
    )]);
    let mismatched_guard_problem = AtpProblem::try_new(mismatched_parts).expect("mutated guard");
    let mismatched_guard_encoded =
        EncodedBackendProblem::encode(&mismatched_guard_problem, ConcreteFormat::Tptp)
            .expect("mutated guard encoding");
    assert!(
        find_instance_evidence(
            &mismatched_guard_problem,
            &mismatched_guard_encoded,
            &handoff,
            Some([1, 2, 0, 1]),
            None,
        )
        .is_err()
    );

    let forged = ["ax_99".to_owned(), "goal:1".to_owned()];
    assert!(
        find_instance_evidence(
            &problem,
            &encoded,
            &handoff,
            Some([0, 2, 0, 1]),
            Some(&forged),
        )
        .is_err()
    );
    let subset = ["ax_0".to_owned()];
    let subset_candidate = find_instance_evidence(
        &problem,
        &encoded,
        &handoff,
        Some([1, 1, 0, 1]),
        Some(&subset),
    )
    .expect("valid premise subset")
    .expect("valid subset proves goal");
    assert_eq!(
        subset_candidate.formula_label_refs(),
        encoded.formula_labels()
    );
    assert_kernel_accepts_instance_candidate(&subset_candidate, &handoff);

    let false_goal = AtpFormulaTree::Atom(AtpAtom::new("q", Vec::new()));
    let (satisfiable_problem, satisfiable_handoff) = finder_problem(
        (
            AtpFormulaTree::Atom(AtpAtom::new("p", Vec::new())),
            kernel_atom(1, Vec::new()),
        ),
        None,
        (false_goal, kernel_atom(2, Vec::new())),
        vec![finder_symbol(0, 1, 1), finder_symbol(1, 1, 2)],
        Vec::new(),
        vec![
            declaration_projection("p", "p", AtpDeclarationKind::Predicate, 0),
            declaration_projection("q", "q", AtpDeclarationKind::Predicate, 0),
        ],
    );
    let satisfiable_encoded =
        EncodedBackendProblem::encode(&satisfiable_problem, ConcreteFormat::Tptp).expect("encode");
    assert!(
        find_instance_evidence(
            &satisfiable_problem,
            &satisfiable_encoded,
            &satisfiable_handoff,
            Some([0, 1, 0, 1]),
            None,
        )
        .expect("satisfiable instance problem")
        .is_none()
    );
}

#[test]
fn instance_finder_uses_equality_names_for_symmetry_transitivity_and_congruence() {
    use crate::instance_finder::find_instance_evidence;
    use mizar_kernel::clause::Term;

    let a = AtpTerm::Function {
        function: "a".into(),
        arguments: Vec::new(),
    };
    let b = AtpTerm::Function {
        function: "b".into(),
        arguments: Vec::new(),
    };
    let c = AtpTerm::Function {
        function: "c".into(),
        arguments: Vec::new(),
    };
    let fa = AtpTerm::Function {
        function: "f".into(),
        arguments: vec![a.clone()],
    };
    let fb = AtpTerm::Function {
        function: "f".into(),
        arguments: vec![b.clone()],
    };
    let local_tree = AtpFormulaTree::Equality {
        left: b.clone(),
        right: a.clone(),
    };
    let second_tree = AtpFormulaTree::Equality {
        left: c.clone(),
        right: fa,
    };
    let goal_tree = AtpFormulaTree::Equality { left: fb, right: c };
    let equality = |left, right| {
        mizar_kernel::formula_evidence::Formula::Atom(mizar_kernel::clause::Atom::new(
            mizar_kernel::clause::SymbolKey::new(mizar_kernel::clause::SymbolKind::Equality, 3),
            vec![left, right],
        ))
    };
    let function = |id, argument| Term::Application {
        symbol: mizar_kernel::clause::SymbolKey::new(
            mizar_kernel::clause::SymbolKind::FunctorPredicate,
            id,
        ),
        arguments: vec![argument],
    };
    let const_ = |id| Term::Application {
        symbol: mizar_kernel::clause::SymbolKey::new(
            mizar_kernel::clause::SymbolKind::FunctorPredicate,
            id,
        ),
        arguments: Vec::new(),
    };
    let (problem, handoff) = finder_problem(
        (local_tree, equality(const_(5), const_(4))),
        Some((second_tree, equality(const_(7), function(6, const_(4))))),
        (goal_tree, equality(function(6, const_(5)), const_(7))),
        vec![
            finder_symbol(0, 2, 4),
            finder_symbol(1, 2, 5),
            finder_symbol(2, 2, 6),
            finder_symbol(3, 2, 7),
            finder_symbol(4, 3, 3),
        ],
        Vec::new(),
        vec![
            declaration_projection("a", "a", AtpDeclarationKind::Function, 0),
            declaration_projection("b", "b", AtpDeclarationKind::Function, 0),
            declaration_projection("c", "c", AtpDeclarationKind::Function, 0),
            declaration_projection("f", "f", AtpDeclarationKind::Function, 1),
        ],
    );
    let encoded = crate::backend::EncodedBackendProblem::encode(&problem, ConcreteFormat::Tptp)
        .expect("equality problem encoding");
    assert!(
        find_instance_evidence(&problem, &encoded, &handoff, Some([5, 2, 0, 100]), None)
            .expect("no equality candidates allowed")
            .is_none()
    );
    let candidate = find_instance_evidence(
        &problem,
        &encoded,
        &handoff,
        Some([5, 2, 1_000, 1_000]),
        None,
    )
    .expect("equality search")
    .expect("congruence evidence");
    assert_kernel_accepts_instance_candidate(&candidate, &handoff);
    let crate::backend::BackendCandidatePayload::FormulaSubstitutionBytes(bytes) =
        candidate.payload()
    else {
        panic!("expected formula evidence");
    };
    let parsed = mizar_kernel::formula_evidence::parse_formula_evidence(
        bytes,
        &finder_parse_context(&handoff),
    )
    .expect("parse equality evidence");
    let names = u64::try_from(parsed.equality_instances().len()).expect("name count");
    assert!(names > 0);
    let exact = find_instance_evidence(
        &problem,
        &encoded,
        &handoff,
        Some([5, 2, names, names + 1]),
        None,
    )
    .expect("exact equality/SAT bounds")
    .expect("exact bounds retain the proof");
    assert_eq!(candidate, exact);
    for limits in [[5, 2, names - 1, names + 1], [5, 2, names, names]] {
        assert!(
            find_instance_evidence(&problem, &encoded, &handoff, Some(limits), None)
                .expect("one fewer equality name or SAT query")
                .is_none()
        );
    }
}

fn finder_symbol(
    entry_id: u32,
    kind_tag: u8,
    symbol_id: u32,
) -> mizar_vc::kernel_evidence_handoff::KernelManifestEntry {
    mizar_vc::kernel_evidence_handoff::KernelManifestEntry {
        entry_id,
        payload: [vec![kind_tag], symbol_id.to_be_bytes().to_vec()].concat(),
    }
}

fn finder_problem(
    local: (AtpFormulaTree, mizar_kernel::formula_evidence::Formula),
    generated: Option<(AtpFormulaTree, mizar_kernel::formula_evidence::Formula)>,
    goal: (AtpFormulaTree, mizar_kernel::formula_evidence::Formula),
    symbols: Vec<mizar_vc::kernel_evidence_handoff::KernelManifestEntry>,
    variables: Vec<u32>,
    declarations: Vec<AtpDeclarationProjection>,
) -> (AtpProblem, VcKernelEvidenceHandoff) {
    let (local_tree, local_formula) = local;
    let (goal_tree, goal_formula) = goal;
    use mizar_vc::kernel_evidence_handoff::{
        KernelEvidenceProfile, KernelFormulaPayload, KernelFormulaProjection, KernelGoalPolarity,
        KernelManifestEntry, build_kernel_evidence_handoff,
    };
    use mizar_vc::vc_ir::PremiseRef;

    let set = fixture_set_with(
        VcStatus::NeedsAtp,
        "instance-finder",
        if generated.is_some() {
            vec![
                PremiseRef::LocalContext(ContextEntryId::new(0)),
                PremiseRef::GeneratedFact {
                    formula: VcFormulaRef::Generated(VcGeneratedFormulaId::new(2)),
                },
            ]
        } else {
            vec![PremiseRef::LocalContext(ContextEntryId::new(0))]
        },
        None,
    );
    let generated_formula = generated
        .as_ref()
        .map(|(_, formula)| formula)
        .unwrap_or(&local_formula);
    let kernel_formulas = [&local_formula, &goal_formula, generated_formula];
    let provenance_payloads = [
        b"local-instance-premise".as_slice(),
        b"goal-instance".as_slice(),
        b"generated-instance-premise".as_slice(),
    ];
    let formula_payloads = kernel_formulas
        .iter()
        .enumerate()
        .map(|(index, formula)| KernelFormulaPayload {
            formula_ref: VcFormulaRef::Generated(VcGeneratedFormulaId::new(index)),
            projection: KernelFormulaProjection {
                formula_fingerprint: KernelEvidenceFingerprint::new(
                    KERNEL_FORMULA_FINGERPRINT_ALGORITHM_ID,
                    formula.canonical_hash_input().expect("canonical formula"),
                )
                .expect("formula fingerprint"),
                formula_bytes: finder_formula_wire(formula),
                provenance_payload: provenance_payloads[index].to_vec(),
            },
        })
        .collect::<Vec<_>>();
    let variable_manifest = variables
        .iter()
        .enumerate()
        .map(|(index, id)| KernelManifestEntry {
            entry_id: index as u32,
            payload: id.to_be_bytes().to_vec(),
        })
        .collect::<Vec<_>>();
    let handoff = build_kernel_evidence_handoff(KernelEvidenceHandoffInput {
        vc_set: &set,
        vc: VcId::new(0),
        goal_polarity: KernelGoalPolarity::AssertFalseForRefutation,
        kernel_profile: KernelEvidenceProfile::v1(1, KernelClauseTautologyPolicy::Reject),
        symbol_manifest: &symbols,
        variable_manifest: &variable_manifest,
        formula_payloads: &formula_payloads,
        imported_formula_payloads: &[],
        substitutions: &[],
        formula_context: None,
        discharge_output: None,
    })
    .expect("valid kernel handoff");
    let projection = |target, formula, source, identity: &str, index: usize| {
        formula_projection(
            target,
            formula,
            source,
            identity,
            format!("atp-provenance:{identity}"),
            &formula_payloads[index]
                .projection
                .formula_fingerprint
                .digest,
            &formula_payloads[index].projection.provenance_payload,
        )
    };
    let mut input = basic_problem_input(&set, &handoff);
    input.logic_profile = profile(SoftTypeStrategy::GuardPredicates);
    input.soft_type_projections = vec![soft_guard("finder-premise", local_tree.clone())];
    input.declaration_projections = declarations;
    input.formula_projections = vec![
        projection(
            AtpFormulaProjectionTarget::VcFormula(VcFormulaRef::Generated(
                VcGeneratedFormulaId::new(0),
            )),
            local_tree,
            AtpSourceRef::LocalHypothesis(AtpSourceBinding::new("local-context:1")),
            "local-context:1",
            0,
        ),
        projection(
            AtpFormulaProjectionTarget::VcFormula(VcFormulaRef::Generated(
                VcGeneratedFormulaId::new(1),
            )),
            goal_tree,
            AtpSourceRef::GeneratedVcFact(AtpSourceBinding::new("goal:1")),
            "goal:1",
            1,
        ),
    ];
    if let Some((tree, _)) = generated {
        input.formula_projections.push(projection(
            AtpFormulaProjectionTarget::VcFormula(VcFormulaRef::Generated(
                VcGeneratedFormulaId::new(2),
            )),
            tree,
            AtpSourceRef::GeneratedVcFact(AtpSourceBinding::new("generated:3")),
            "generated:3",
            2,
        ));
    }
    let translated = translate_problem(input).expect("ATP problem translation");
    (
        reprofile_finder_problem(&translated, ConcreteFormat::Tptp),
        handoff,
    )
}

fn reprofile_finder_problem(problem: &AtpProblem, format: ConcreteFormat) -> AtpProblem {
    let profile = LogicProfile::try_new(
        match format {
            ConcreteFormat::Tptp => "finder-fof",
            ConcreteFormat::SmtLib => "finder-smt",
        },
        match format {
            ConcreteFormat::Tptp => LogicFragment::Fof,
            ConcreteFormat::SmtLib => LogicFragment::SmtLibUninterpreted,
        },
        EqualitySupport::Supported,
        QuantifierPolicy::FirstOrder,
        SoftTypeStrategy::GuardPredicates,
        NativePropertySupport::Unsupported,
        BTreeSet::from([format]),
    )
    .expect("backend profile");
    AtpProblem::try_new(AtpProblemParts {
        vc_id: problem.vc_id(),
        target_binding: problem.target_binding().clone(),
        logic_profile: profile,
        expected_result: problem.expected_result(),
        declarations: problem.declarations().to_vec(),
        axioms: problem.axioms().to_vec(),
        conjecture: problem.conjecture().clone(),
        type_context: problem.type_context().clone(),
        properties: problem.properties().to_vec(),
        symbol_map: problem.symbol_map().to_vec(),
        provenance: problem.provenance().to_vec(),
        diagnostics: problem.diagnostics().to_vec(),
    })
    .expect("reprofiled backend problem")
}

fn finder_formula_wire(formula: &mizar_kernel::formula_evidence::Formula) -> Vec<u8> {
    use mizar_kernel::{clause::Term, formula_evidence::Formula};
    fn u32_bytes(value: u32, out: &mut Vec<u8>) {
        out.extend_from_slice(&value.to_be_bytes());
    }
    fn term_wire(term: &Term, out: &mut Vec<u8>) {
        match term {
            Term::Variable(id) => {
                out.push(1);
                u32_bytes(id.0, out);
            }
            Term::Application { symbol, arguments } => {
                out.push(2);
                out.push(match symbol.kind {
                    mizar_kernel::clause::SymbolKind::Predicate => 1,
                    mizar_kernel::clause::SymbolKind::FunctorPredicate => 2,
                    mizar_kernel::clause::SymbolKind::Equality => 3,
                    mizar_kernel::clause::SymbolKind::BuiltinRelation => 4,
                    _ => unreachable!(),
                });
                u32_bytes(symbol.id.0, out);
                u32_bytes(arguments.len() as u32, out);
                for argument in arguments {
                    term_wire(argument, out);
                }
            }
            _ => panic!("fixture terms are variables or applications"),
        }
    }
    fn write(formula: &Formula, out: &mut Vec<u8>) {
        match formula {
            Formula::Atom(atom) => {
                out.push(1);
                out.push(match atom.symbol.kind {
                    mizar_kernel::clause::SymbolKind::Predicate => 1,
                    mizar_kernel::clause::SymbolKind::FunctorPredicate => 2,
                    mizar_kernel::clause::SymbolKind::Equality => 3,
                    mizar_kernel::clause::SymbolKind::BuiltinRelation => 4,
                    _ => unreachable!(),
                });
                u32_bytes(atom.symbol.id.0, out);
                u32_bytes(atom.arity, out);
                u32_bytes(atom.arguments.len() as u32, out);
                for argument in &atom.arguments {
                    term_wire(argument, out);
                }
            }
            Formula::Not(child) => {
                out.push(2);
                write(child, out);
            }
            Formula::And(children) | Formula::Or(children) => {
                out.push(if matches!(formula, Formula::And(_)) {
                    3
                } else {
                    4
                });
                u32_bytes(children.len() as u32, out);
                for child in children {
                    write(child, out);
                }
            }
            _ => panic!("fixture formula is supported"),
        }
    }
    let mut out = Vec::new();
    write(formula, &mut out);
    out
}

fn kernel_atom(
    symbol_id: u32,
    arguments: Vec<mizar_kernel::clause::Term>,
) -> mizar_kernel::formula_evidence::Formula {
    mizar_kernel::formula_evidence::Formula::Atom(mizar_kernel::clause::Atom::new(
        mizar_kernel::clause::SymbolKey::new(
            mizar_kernel::clause::SymbolKind::Predicate,
            symbol_id,
        ),
        arguments,
    ))
}

fn kernel_var(id: u32) -> mizar_kernel::clause::Term {
    mizar_kernel::clause::Term::Variable(mizar_kernel::clause::VariableId(id))
}

fn kernel_const(id: u32) -> mizar_kernel::clause::Term {
    mizar_kernel::clause::Term::Application {
        symbol: mizar_kernel::clause::SymbolKey::new(
            mizar_kernel::clause::SymbolKind::FunctorPredicate,
            id,
        ),
        arguments: Vec::new(),
    }
}

fn kernel_or(
    children: Vec<mizar_kernel::formula_evidence::Formula>,
) -> mizar_kernel::formula_evidence::Formula {
    mizar_kernel::formula_evidence::Formula::Or(children)
}

fn finder_parse_context(
    handoff: &VcKernelEvidenceHandoff,
) -> mizar_kernel::formula_evidence::FormulaEvidenceParseContext {
    let target = handoff.canonical_evidence().target_vc();
    let profile = handoff.canonical_evidence().kernel_profile();
    let policy = match profile.clause_tautology_policy {
        KernelClauseTautologyPolicy::Reject => {
            mizar_kernel::certificate_parser::ClauseTautologyPolicy::Reject
        }
        KernelClauseTautologyPolicy::Marker => {
            mizar_kernel::certificate_parser::ClauseTautologyPolicy::Marker
        }
        _ => unreachable!(),
    };
    let mut context = mizar_kernel::formula_evidence::FormulaEvidenceParseContext::v1(
        mizar_kernel::certificate_parser::Fingerprint::new(
            target.algorithm_id,
            target.digest.clone(),
        ),
        mizar_kernel::certificate_parser::KernelProfileRecord::v1(profile.profile_id, policy),
    );
    context.accepted_schema_versions.insert(2);
    context
}

fn assert_kernel_accepts_instance_candidate(
    candidate: &crate::instance_finder::CandidateKernelEvidence,
    handoff: &VcKernelEvidenceHandoff,
) {
    use crate::backend::BackendCandidatePayload;
    use mizar_kernel::{
        checker::{
            FormulaEvidenceContext, ImportedFactContextLimits, KernelCheckPolicy,
            KernelContextIdentityEntry, KernelContextIdentityPayload, KernelContextIdentitySource,
            KernelEvidenceCheckInput, KernelEvidenceCheckKind, KernelEvidenceCheckLimits,
            KernelFormulaProducerRef, KernelVcGeneratedFormulaId, check_kernel_evidence,
        },
        formula_evidence::parse_formula_evidence,
        rejection::TargetVcFingerprint,
    };
    use mizar_vc::{
        kernel_evidence_handoff::KernelContextIdentitySource as VcSource, vc_ir::VcFormulaRef,
    };

    let BackendCandidatePayload::FormulaSubstitutionBytes(bytes) = candidate.payload() else {
        panic!("candidate payload is not formula substitution evidence")
    };
    let parsed =
        parse_formula_evidence(bytes, &finder_parse_context(handoff)).expect("parse candidate");
    let envelope = handoff.canonical_evidence();
    let target = TargetVcFingerprint::new(
        envelope.target_vc().algorithm_id,
        envelope.target_vc().digest.clone(),
    );
    let entries = handoff
        .context_identity()
        .entries()
        .iter()
        .map(|entry| {
            let source = match entry.source() {
                VcSource::LocalHypothesis { local_context_id } => {
                    KernelContextIdentitySource::LocalHypothesis { local_context_id }
                }
                VcSource::CitedPremise { local_context_id } => {
                    KernelContextIdentitySource::CitedPremise { local_context_id }
                }
                VcSource::GeneratedVcFact { vc_fact_id } => {
                    KernelContextIdentitySource::GeneratedVcFact { vc_fact_id }
                }
                _ => panic!("unsupported test source"),
            };
            let producer = match entry.producer_formula_ref() {
                VcFormulaRef::Core(id) => KernelFormulaProducerRef::Core(id),
                VcFormulaRef::Generated(id) => {
                    KernelFormulaProducerRef::Generated(KernelVcGeneratedFormulaId::new(id.index()))
                }
                _ => panic!("unsupported producer ref"),
            };
            KernelContextIdentityEntry::new(
                source,
                entry.formula_id(),
                mizar_kernel::certificate_parser::Fingerprint::new(
                    entry.formula_fingerprint().algorithm_id,
                    entry.formula_fingerprint().digest.clone(),
                ),
                producer,
            )
        })
        .collect();
    let context = FormulaEvidenceContext::with_context_identity(
        Some(handoff.context_identity_hash().as_bytes().to_vec()),
        Vec::new(),
        Vec::new(),
        Some(KernelContextIdentityPayload::new(
            target.clone(),
            handoff.canonical_hash(),
            handoff.context_identity_hash(),
            entries,
        )),
        ImportedFactContextLimits::default(),
    )
    .expect("formula context");
    let checked = check_kernel_evidence(KernelEvidenceCheckInput {
        target_vc_fingerprint: &target,
        evidence: &parsed,
        formula_context: Some(&context),
        check_kind: KernelEvidenceCheckKind::ProofObligation,
        policy: KernelCheckPolicy::default(),
        limits: KernelEvidenceCheckLimits::default(),
    });
    assert_eq!(
        checked.status(),
        mizar_kernel::checker::KernelCheckStatus::Accepted,
        "{checked:?}"
    );
}
