use mizar_atp::{
    backend::{
        BackendCancellationToken, BackendCandidateEvidence, BackendCandidatePayload,
        BackendCommand, BackendIoMode, BackendKind, BackendObservation, BackendObservedResult,
        BackendProfile, BackendProfileId, BackendResourceLimits, BackendRunId, BackendRunInput,
        BackendRunStatus, EncodedBackendProblem, EncodedBackendProblemParts,
        classify_backend_observation, run_backend,
    },
    portfolio::{
        PortfolioBudget, PortfolioEvidenceSet, PortfolioId, PortfolioInput, PortfolioInputParts,
        PortfolioPolicyConstraints, collect_portfolio_results, plan_portfolio,
    },
    problem::{
        AtpAtom, AtpDeclarationKind, AtpDiagnostic, AtpFingerprint, AtpFormulaTree, AtpProblem,
        AtpSourceBinding, AtpSourceRef, AtpSymbolName, ConcreteFormat, EqualitySupport,
        ExpectedBackendResult, LogicFragment, LogicProfile, NativePropertySupport,
        QuantifierPolicy, SoftTypeStrategy,
    },
    smtlib_encoder::{SmtLibDialect, encode_smtlib},
    tptp_encoder::{TptpDialect, encode_tptp},
    translator::{
        AtpDeclarationProjection, AtpFormulaProjection, AtpFormulaProjectionTarget,
        AtpProjectionKey, AtpProjectionProvenance, AtpSoftTypeProjection,
        AtpSoftTypeRepresentation, AtpSymbolSourceProjection, AtpTranslationInput,
        translate_problem,
    },
};
use mizar_core::{
    control_flow::ObligationHandoffId,
    core_ir::{
        CoreItemId, CoreLabelRef, CoreProvenance, CoreProvenancePhase, CoreSourceRef,
        LocalProofOrProgramPath, NormalizedSemanticOrigin, ObligationSeedId, ObligationSeedStatus,
    },
};
use mizar_session::{
    BuildSnapshotId, Hash, InMemorySessionIdAllocator, SessionIdAllocator, SourceId, SourceRange,
};
use mizar_vc::{
    kernel_evidence_handoff::{
        KERNEL_FORMULA_FINGERPRINT_ALGORITHM_ID, KernelClauseTautologyPolicy,
        KernelEvidenceFingerprint, KernelEvidenceHandoffInput, KernelEvidenceProfile,
        KernelFormulaPayload, KernelFormulaProjection, KernelGoalPolarity, VcKernelEvidenceHandoff,
        build_kernel_evidence_handoff,
    },
    vc_ir::{
        AnchorCompleteness, AnchorIngredient, AnchorLabel, AnchorLabelRole, AnchorOwner,
        AnchorUnavailableReason, CanonicalSortKey, ContextEntry, ContextEntryId, ContextEntryKind,
        GenerationSchemaVersion, HashMarker, LocalContext, PremiseRef, SeedAccounting,
        SeedOriginRef, SeedVcMapping, SeedVcRef, VcFormulaRef, VcGeneratedFormula,
        VcGeneratedFormulaId, VcGeneratedFormulaKind, VcGeneratedFormulaShape, VcId, VcIr, VcKind,
        VcModuleRef, VcProvenance, VcProvenancePhase, VcSchemaVersion, VcSet, VcSetParts,
        VcSourceRef, VcStatus, VcText,
    },
};
use std::{collections::BTreeSet, time::Duration};

#[test]
fn identical_vcir_inputs_produce_identical_problem_encodings_and_candidate_order() {
    for kind in [EncodingKind::Tptp, EncodingKind::SmtLib] {
        let first = encoded_fixture(kind);
        let second = encoded_fixture(kind);

        assert_eq!(first.vc_debug_text, second.vc_debug_text);
        assert_eq!(first.vc_hash, second.vc_hash);
        assert_eq!(first.handoff_hash, second.handoff_hash);
        assert_eq!(first.handoff_debug_text, second.handoff_debug_text);
        assert_eq!(first.problem.problem_id(), second.problem.problem_id());
        assert_eq!(first.problem.debug_text(), second.problem.debug_text());
        assert_eq!(
            first.encoded_problem.input_hash(),
            second.encoded_problem.input_hash()
        );
        assert_eq!(
            first.encoded_problem.metadata_hash(),
            second.encoded_problem.metadata_hash()
        );
        assert_eq!(first.encoder_text, second.encoder_text);
        assert_eq!(first.formula_labels, second.formula_labels);
        assert_eq!(first.symbol_bindings, second.symbol_bindings);
        assert_eq!(
            first.encoded_problem.formula_labels(),
            second.encoded_problem.formula_labels()
        );
        assert_eq!(
            first.encoded_problem.symbol_bindings(),
            second.encoded_problem.symbol_bindings()
        );

        assert_mock_portfolio_handoff_is_order_independent(kind, &first);
    }
}

#[derive(Debug, Clone, Copy)]
enum EncodingKind {
    Tptp,
    SmtLib,
}

impl EncodingKind {
    const fn concrete_format(self) -> ConcreteFormat {
        match self {
            Self::Tptp => ConcreteFormat::Tptp,
            Self::SmtLib => ConcreteFormat::SmtLib,
        }
    }

    const fn logic_fragment(self) -> LogicFragment {
        match self {
            Self::Tptp => LogicFragment::Fof,
            Self::SmtLib => LogicFragment::SmtLibUninterpreted,
        }
    }

    const fn profile_name(self) -> &'static str {
        match self {
            Self::Tptp => "determinism-fof",
            Self::SmtLib => "determinism-smtlib",
        }
    }

    const fn logic_fragment_name(self) -> &'static str {
        match self {
            Self::Tptp => "fof",
            Self::SmtLib => "smtlib-uninterpreted",
        }
    }

    const fn provenance_seed(self) -> u8 {
        match self {
            Self::Tptp => 0x31,
            Self::SmtLib => 0x32,
        }
    }
}

#[derive(Clone)]
struct EncodedFixture {
    vc_debug_text: String,
    vc_hash: Hash,
    handoff_hash: Hash,
    handoff_debug_text: String,
    problem: AtpProblem,
    encoded_problem: EncodedBackendProblem,
    encoder_text: Vec<u8>,
    formula_labels: Vec<String>,
    symbol_bindings: Vec<String>,
}

fn encoded_fixture(kind: EncodingKind) -> EncodedFixture {
    let set = fixture_set();
    let handoff = kernel_handoff(&set);
    let vc_debug_text = set.debug_text();
    let handoff_hash = handoff.canonical_hash();
    let handoff_debug_text = handoff.debug_text();
    let problem = translate_problem(translation_input(&set, &handoff, kind)).expect("problem");
    let vc_hash = set
        .canonical_vc_fingerprint(VcId::new(0))
        .expect("canonical VC fingerprint")
        .hash();

    match kind {
        EncodingKind::Tptp => {
            let output = encode_tptp(&problem, TptpDialect::Fof).expect("TPTP output");
            let formula_labels = output
                .formula_labels()
                .iter()
                .map(|label| label.label().to_owned())
                .collect::<Vec<_>>();
            let symbol_bindings = output
                .symbol_bindings()
                .iter()
                .map(|binding| {
                    format!(
                        "{}=>{}::{:?}",
                        binding.atp_symbol().as_str(),
                        binding.tptp_name(),
                        binding.source()
                    )
                })
                .collect::<Vec<_>>();
            let encoder_text = output.text().as_bytes().to_vec();
            let encoded_problem = encoded_problem(
                kind,
                &problem,
                encoder_text.clone(),
                formula_labels.clone(),
                symbol_bindings.clone(),
            );
            EncodedFixture {
                vc_debug_text,
                vc_hash,
                handoff_hash,
                handoff_debug_text,
                problem,
                encoded_problem,
                encoder_text,
                formula_labels,
                symbol_bindings,
            }
        }
        EncodingKind::SmtLib => {
            let output =
                encode_smtlib(&problem, SmtLibDialect::Uninterpreted).expect("SMT-LIB output");
            let formula_labels = output
                .assertion_labels()
                .iter()
                .map(|label| label.label().to_owned())
                .collect::<Vec<_>>();
            let symbol_bindings = output
                .symbol_bindings()
                .iter()
                .map(|binding| {
                    format!(
                        "{}=>{}::{:?}",
                        binding.atp_symbol().as_str(),
                        binding.smtlib_symbol(),
                        binding.source()
                    )
                })
                .collect::<Vec<_>>();
            let encoder_text = output.text().as_bytes().to_vec();
            let encoded_problem = encoded_problem(
                kind,
                &problem,
                encoder_text.clone(),
                formula_labels.clone(),
                symbol_bindings.clone(),
            );
            EncodedFixture {
                vc_debug_text,
                vc_hash,
                handoff_hash,
                handoff_debug_text,
                problem,
                encoded_problem,
                encoder_text,
                formula_labels,
                symbol_bindings,
            }
        }
    }
}

fn encoded_problem(
    kind: EncodingKind,
    problem: &AtpProblem,
    input_text: Vec<u8>,
    formula_labels: Vec<String>,
    symbol_bindings: Vec<String>,
) -> EncodedBackendProblem {
    EncodedBackendProblem::new(EncodedBackendProblemParts {
        problem_id: problem.problem_id(),
        target_binding: problem.target_binding().clone(),
        expected_result: ExpectedBackendResult::Unsat,
        concrete_format: kind.concrete_format(),
        logic_profile_name: problem.logic_profile().name().as_str().to_owned(),
        logic_fragment: kind.logic_fragment_name().to_owned(),
        input_text,
        formula_labels,
        symbol_bindings,
        provenance_hash: fixture_hash(kind.provenance_seed()),
    })
    .expect("encoded backend problem")
}

fn assert_mock_portfolio_handoff_is_order_independent(
    kind: EncodingKind,
    fixture: &EncodedFixture,
) {
    let run_zeta = backend_run(kind, fixture, "run-zeta", "profile-zeta", 20, 902);
    let run_alpha = backend_run(kind, fixture, "run-alpha", "profile-alpha", 10, 901);

    let plan_forward = plan_portfolio(portfolio_input(
        kind,
        fixture,
        vec![run_zeta.clone(), run_alpha.clone()],
    ))
    .expect("forward plan");
    let plan_reverse = plan_portfolio(portfolio_input(
        kind,
        fixture,
        vec![run_alpha.clone(), run_zeta.clone()],
    ))
    .expect("reverse plan");

    assert_eq!(plan_forward.plan_hash(), plan_reverse.plan_hash());
    assert_eq!(
        scheduled_run_ids(plan_forward.scheduled_runs()),
        ["run-alpha", "run-zeta"]
    );
    assert_eq!(
        scheduled_run_ids(plan_reverse.scheduled_runs()),
        ["run-alpha", "run-zeta"]
    );

    let result_zeta = proved_result(kind, &run_zeta, "candidate-zeta", b"zeta");
    let result_alpha = proved_result(kind, &run_alpha, "candidate-alpha", b"alpha");

    let evidence_forward = collect_portfolio_results(
        plan_forward,
        vec![result_zeta.clone(), result_alpha.clone()],
    )
    .expect("forward evidence");
    let evidence_reverse = collect_portfolio_results(plan_reverse, vec![result_alpha, result_zeta])
        .expect("reverse evidence");

    assert_eq!(
        candidate_ids(&evidence_forward),
        ["run-alpha:candidate-alpha", "run-zeta:candidate-zeta"]
    );
    assert_eq!(
        candidate_signatures(&evidence_forward),
        candidate_signatures(&evidence_reverse)
    );
    assert_eq!(
        evidence_forward.evidence_set_hash(),
        evidence_reverse.evidence_set_hash()
    );
    assert!(evidence_forward.diagnostics().is_empty());
    assert!(evidence_reverse.diagnostics().is_empty());
    assert!(
        evidence_forward
            .backend_results()
            .iter()
            .all(|result| result.status() == BackendRunStatus::Proved)
    );
}

fn backend_run(
    kind: EncodingKind,
    fixture: &EncodedFixture,
    run_id: &str,
    profile_id: &str,
    deterministic_priority: u32,
    seed: u64,
) -> BackendRunInput {
    let profile = BackendProfile::new(
        BackendProfileId::new(profile_id).expect("profile id"),
        BackendKind::new("mock-determinism-backend").expect("backend kind"),
        kind.concrete_format(),
    )
    .with_deterministic_priority(deterministic_priority);
    let command = BackendCommand::new(
        "/bin/sh",
        vec!["-c".to_owned(), "cat >/dev/null".to_owned()],
    )
    .expect("backend command")
    .with_semantic_executable_id("mock-determinism-backend")
    .expect("semantic executable id");

    BackendRunInput::new(
        BackendRunId::new(run_id).expect("run id"),
        fixture.encoded_problem.clone(),
        profile,
        command,
        BackendResourceLimits::new()
            .with_wall_timeout(Duration::from_secs(2))
            .with_kill_grace(Duration::from_millis(50)),
        BackendIoMode::Stdin,
        BackendCancellationToken::new(),
    )
    .with_random_seed(seed)
}

fn portfolio_input(
    kind: EncodingKind,
    fixture: &EncodedFixture,
    backend_runs: Vec<BackendRunInput>,
) -> PortfolioInput {
    PortfolioInput::new(PortfolioInputParts {
        portfolio_id: PortfolioId::new(format!("portfolio-{}", kind.profile_name()))
            .expect("portfolio id"),
        vc_hash: fixture.vc_hash,
        atp_problem: fixture.problem.clone(),
        backend_runs,
        obligation_budget: PortfolioBudget::unbounded(),
        scheduler_budget: PortfolioBudget::unbounded(),
        policy_constraints: PortfolioPolicyConstraints::new(),
        cancellation: BackendCancellationToken::new(),
    })
}

fn proved_result(
    kind: EncodingKind,
    run: &BackendRunInput,
    candidate_id: &str,
    payload_suffix: &[u8],
) -> mizar_atp::backend::BackendRunResult {
    let result = run_backend(run.clone());
    assert_eq!(result.status(), BackendRunStatus::Unknown);
    classify_backend_observation(
        result,
        BackendObservation::new(BackendObservedResult::Unsat).with_candidate_evidence(candidate(
            kind,
            run.encoded_problem(),
            candidate_id,
            payload_suffix,
        )),
    )
}

fn candidate(
    kind: EncodingKind,
    encoded_problem: &EncodedBackendProblem,
    candidate_id: &str,
    payload_suffix: &[u8],
) -> BackendCandidateEvidence {
    let mut payload = format!("formula-substitution-candidate:{}:", kind.profile_name())
        .as_bytes()
        .to_vec();
    payload.extend_from_slice(payload_suffix);
    BackendCandidateEvidence::new(
        candidate_id,
        BackendCandidatePayload::FormulaSubstitutionBytes(payload),
        encoded_problem.target_binding().clone(),
        encoded_problem.input_hash(),
        encoded_problem.provenance_hash(),
        encoded_problem.formula_labels().to_vec(),
        encoded_problem.symbol_bindings().to_vec(),
    )
    .expect("candidate evidence")
}

fn scheduled_run_ids(runs: &[BackendRunInput]) -> Vec<&str> {
    runs.iter().map(|run| run.run_id().as_str()).collect()
}

fn candidate_ids(evidence: &PortfolioEvidenceSet) -> Vec<&str> {
    evidence
        .candidates()
        .iter()
        .map(|candidate| candidate.candidate_id().as_str())
        .collect()
}

fn candidate_signatures(
    evidence: &PortfolioEvidenceSet,
) -> Vec<(String, String, String, Hash, Option<Hash>, Hash)> {
    evidence
        .candidates()
        .iter()
        .map(|candidate| {
            (
                candidate.candidate_id().as_str().to_owned(),
                candidate.backend_profile_id().to_owned(),
                format!(
                    "{:?}:{:?}",
                    candidate.candidate_kind(),
                    candidate.evidence_format()
                ),
                candidate.encoded_problem_hash(),
                candidate.evidence_payload_hash(),
                candidate.candidate_hash(),
            )
        })
        .collect()
}

fn translation_input<'a>(
    set: &'a VcSet,
    handoff: &'a VcKernelEvidenceHandoff,
    kind: EncodingKind,
) -> AtpTranslationInput<'a> {
    AtpTranslationInput {
        vc_set: set,
        vc: VcId::new(0),
        kernel_handoff: handoff,
        logic_profile: logic_profile(kind),
        declaration_projections: vec![
            declaration_projection("pred-type-guard", "type_guard"),
            declaration_projection("pred-p", "p"),
        ],
        soft_type_projections: vec![AtpSoftTypeProjection {
            key: AtpProjectionKey::new("type-guard"),
            representation: AtpSoftTypeRepresentation::GuardFormula(AtpFormulaTree::Atom(
                AtpAtom::new("type_guard", Vec::new()),
            )),
            provenance: AtpProjectionProvenance::new(
                AtpSourceRef::TypeFact(AtpSourceBinding::new("type-guard")),
                "atp-provenance:type-guard",
            ),
        }],
        formula_projections: vec![
            formula_projection(
                AtpFormulaProjectionTarget::VcFormula(VcFormulaRef::Generated(
                    VcGeneratedFormulaId::new(0),
                )),
                AtpFormulaTree::Atom(AtpAtom::new("p", Vec::new())),
                AtpSourceRef::LocalHypothesis(AtpSourceBinding::new("local-context:1")),
                "local-context:1",
                "atp-provenance:local:0",
                b"formula-0",
                b"provenance-0",
            ),
            formula_projection(
                AtpFormulaProjectionTarget::VcFormula(VcFormulaRef::Generated(
                    VcGeneratedFormulaId::new(1),
                )),
                AtpFormulaTree::False,
                AtpSourceRef::GeneratedVcFact(AtpSourceBinding::new("goal:1")),
                "goal:1",
                "atp-provenance:goal:1",
                b"formula-1",
                b"provenance-1",
            ),
        ],
        diagnostics: vec![AtpDiagnostic::new("determinism", kind.profile_name())],
    }
}

fn logic_profile(kind: EncodingKind) -> LogicProfile {
    LogicProfile::try_new(
        kind.profile_name(),
        kind.logic_fragment(),
        EqualitySupport::Supported,
        QuantifierPolicy::FirstOrder,
        SoftTypeStrategy::GuardPredicates,
        NativePropertySupport::Unsupported,
        BTreeSet::from([kind.concrete_format()]),
    )
    .expect("logic profile")
}

fn declaration_projection(key: &str, symbol: &str) -> AtpDeclarationProjection {
    AtpDeclarationProjection {
        key: AtpProjectionKey::new(key),
        kind: AtpDeclarationKind::Predicate,
        symbol: AtpSymbolName::new(symbol),
        arity: 0,
        provenance: AtpProjectionProvenance::new(
            AtpSourceRef::GeneratedVcFact(AtpSourceBinding::new(format!("decl:{key}"))),
            format!("decl-payload:{key}"),
        ),
        symbol_source: AtpSymbolSourceProjection::MizarSymbol(AtpSourceBinding::new(key)),
    }
}

fn formula_projection(
    target: AtpFormulaProjectionTarget,
    formula: AtpFormulaTree,
    source: AtpSourceRef,
    source_identity: impl Into<AtpProjectionKey>,
    provenance_payload: impl Into<String>,
    fingerprint_digest: &[u8],
    handoff_provenance_payload: &[u8],
) -> AtpFormulaProjection {
    AtpFormulaProjection {
        target,
        formula,
        provenance: AtpProjectionProvenance::new(source, provenance_payload.into()),
        source_identity: source_identity.into(),
        handoff_formula_fingerprint: AtpFingerprint::new(
            KERNEL_FORMULA_FINGERPRINT_ALGORITHM_ID,
            fingerprint_digest.to_vec(),
        )
        .expect("formula fingerprint"),
        handoff_provenance_payload: handoff_provenance_payload.to_vec(),
    }
}

fn kernel_handoff(set: &VcSet) -> VcKernelEvidenceHandoff {
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

fn formula_payloads(set: &VcSet) -> Vec<KernelFormulaPayload> {
    set.generated_formulas()
        .iter()
        .map(|formula| {
            let index = formula.id.index();
            KernelFormulaPayload {
                formula_ref: VcFormulaRef::Generated(formula.id),
                projection: KernelFormulaProjection {
                    formula_fingerprint: kernel_fingerprint(format!("formula-{index}").as_bytes()),
                    formula_bytes: format!("kernel-formula-{index}").into_bytes(),
                    provenance_payload: format!("provenance-{index}").into_bytes(),
                },
            }
        })
        .collect()
}

fn kernel_fingerprint(bytes: &[u8]) -> KernelEvidenceFingerprint {
    KernelEvidenceFingerprint::new(KERNEL_FORMULA_FINGERPRINT_ALGORITHM_ID, bytes.to_vec())
        .expect("kernel fingerprint")
}

fn fixture_set() -> VcSet {
    let snapshot = BuildSnapshotId::from_published_schema_str(
        "mizar-session-build-snapshot-v1:2222222222222222222222222222222222222222222222222222222222222222",
    )
    .expect("snapshot id");
    let source = InMemorySessionIdAllocator::new()
        .next_source_id(snapshot)
        .expect("source id");
    let handoff = ObligationHandoffId::new(0);
    let local_context = LocalContext::try_new(
        vec![ContextEntry {
            id: ContextEntryId::new(0),
            sort_key: CanonicalSortKey::new("000-local"),
            kind: ContextEntryKind::ProofAssumption,
            formula: Some(VcFormulaRef::Generated(VcGeneratedFormulaId::new(0))),
            provenance: vec![provenance("local")],
        }],
        Vec::new(),
    )
    .expect("local context");

    VcSet::try_new(VcSetParts {
        schema_version: VcSchemaVersion::new("atp-determinism-suite-v1"),
        snapshot,
        source,
        module: VcModuleRef::new("determinism"),
        generated_formulas: vec![
            VcGeneratedFormula {
                id: VcGeneratedFormulaId::new(0),
                kind: VcGeneratedFormulaKind::GeneratedTypeObligation,
                shape: VcGeneratedFormulaShape::True,
                provenance: vec![provenance("generated-0")],
            },
            VcGeneratedFormula {
                id: VcGeneratedFormulaId::new(1),
                kind: VcGeneratedFormulaKind::SplitGoal,
                shape: VcGeneratedFormulaShape::False,
                provenance: vec![provenance("generated-1")],
            },
        ],
        vcs: vec![VcIr {
            id: VcId::new(0),
            kind: VcKind::TheoremProofStep,
            source: VcSourceRef {
                primary: source_ref(source),
                related: Vec::new(),
            },
            seed: SeedVcRef { handoff },
            anchor: incomplete_anchor(source),
            local_context,
            premises: vec![PremiseRef::LocalContext(ContextEntryId::new(0))],
            goal: VcFormulaRef::Generated(VcGeneratedFormulaId::new(1)),
            proof_hint: None,
            status: VcStatus::NeedsAtp,
            provenance: vec![provenance("vc")],
        }],
        seed_accounting: vec![SeedAccounting {
            handoff,
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
        semantic_origin: NormalizedSemanticOrigin::new("theorem:determinism"),
        source_range: Some(SourceRange {
            source_id: source,
            start: 0,
            end: 4,
        }),
        provenance: vec![provenance("anchor")],
        source_shape_hash: unavailable_hash_marker(),
        canonical_goal_hash: unavailable_hash_marker(),
        canonical_context_hash: unavailable_hash_marker(),
        generation_schema_version: GenerationSchemaVersion::new("atp-determinism-suite"),
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
        "atp-determinism-suite",
    )])
}

fn provenance(key: &str) -> VcProvenance {
    VcProvenance {
        phase: VcProvenancePhase::Generator,
        key: VcText::new(key),
        core: None,
    }
}

fn unavailable_hash_marker() -> HashMarker {
    HashMarker::Unavailable {
        reason: AnchorUnavailableReason::new("determinism fixture"),
    }
}

fn fixture_hash(seed: u8) -> Hash {
    Hash::from_bytes([seed; Hash::BYTE_LEN])
}
