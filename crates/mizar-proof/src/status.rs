//! Artifact- and diagnostics-facing proof status projection.
//!
//! This module projects proof outcomes. Its bounded source-registration facades
//! own fresh normal kernel checks for classification or a local registration database.
//! It does not run ATP backends, query caches, or write artifact manifests.

use std::{error::Error, fmt};

use mizar_artifact::store::SchemaVersion;
use mizar_kernel::checker::{KernelCheckResult, KernelCheckStatus, UsedAxiom};
use mizar_session::Hash;
use mizar_vc::vc_ir::VcId;

use crate::{
    policy::{
        AcceptedGoalPolarity, ExternalEvidencePublicationStatus, KernelEvidenceOrigin,
        KernelPolicyInput, OpenObligationMode, PolicyFingerprint, VerifierPolicy,
    },
    selection::{
        ArtifactProofSelection, CandidateSourceId, DiagnosticRef, ProofWinnerClass,
        ProofWitnessPublication,
    },
};

const USED_AXIOMS_HASH_DOMAIN: &str = "mizar-proof-trusted-used-axioms-v1";
const PROOF_REUSE_VALIDATION_HASH_DOMAIN: &str = "mizar-proof-reuse-validation-v1";

/// Returns raw canonical theorem content and its full conservative dependency slice.
/// Source/snapshot freshness remains caller-owned; these inputs do not authorize acceptance or reuse.
pub fn source_theorem_canonical_inputs(
    check: &mizar_checker::type_checker::SourceTheoremCheck<'_>,
    snapshot: mizar_session::BuildSnapshotId,
    vc_id: VcId,
) -> Result<
    (
        mizar_vc::vc_ir::CanonicalVcFingerprint,
        Hash,
        mizar_vc::vc_ir::LocalContextFingerprint,
        mizar_vc::dependency_slice::DependencySlice,
    ),
    String,
> {
    use mizar_vc::{
        dependency_slice::{DependencySliceInput, try_compute_dependency_slices},
        vc_ir::{GenerationSchemaVersion, VcSchemaVersion},
    };

    let core = mizar_core::elaborator::lower_source_theorem_skeletons(check, None)?;
    let roots = core
        .obligation_seeds()
        .iter()
        .flat_map(|(_, seed)| seed.context.iter().copied().chain(seed.goal))
        .collect::<Vec<_>>();
    let formulas = mizar_core::elaborator::source_theorem_formula_payloads(check, &core, &roots)?;
    let vcs = mizar_vc::generator::generate_source_theorem_vcs(
        &core,
        snapshot,
        &GenerationSchemaVersion::new("source-theorem-local-v1"),
        &VcSchemaVersion::new("vc-v1"),
    )?;
    let (vc, goal, context) = vcs.source_theorem_canonical_components(vc_id, &core, &formulas)?;
    let slices = try_compute_dependency_slices(DependencySliceInput {
        vc_set: &vcs,
        discharge_output: None,
    })
    .map_err(|error| error.to_string())?;
    let slice = slices
        .slice_for(vc_id)
        .cloned()
        .ok_or_else(|| "theorem.proof.invalid_source_canonical_inputs".to_owned())?;
    Ok((vc, goal, context, slice))
}

/// Checks genuine original theorem reflexivity and returns its normal policy classification.
pub fn check_source_theorem_reflexivity(
    source: &mizar_resolve::resolved_ast::SurfaceResolvedArena,
    check: &mizar_checker::type_checker::SourceTheoremCheck<'_>,
    symbols: &mizar_resolve::env::SymbolEnv,
    snapshot: mizar_session::BuildSnapshotId,
    vc_id: VcId,
    policy: &VerifierPolicy,
) -> Result<(KernelCheckResult, crate::policy::PolicyDecision), String> {
    let (result, decision, _, _, _) = check_source_theorem_reflexivity_payload(
        source, check, symbols, snapshot, vc_id, policy, false,
    )?;
    Ok((result, decision))
}

#[expect(
    clippy::type_complexity,
    reason = "Keeps the shared checked payload explicit without a single-use alias"
)]
fn check_source_theorem_reflexivity_payload(
    source: &mizar_resolve::resolved_ast::SurfaceResolvedArena,
    check: &mizar_checker::type_checker::SourceTheoremCheck<'_>,
    symbols: &mizar_resolve::env::SymbolEnv,
    snapshot: mizar_session::BuildSnapshotId,
    vc_id: VcId,
    policy: &VerifierPolicy,
    include_metadata: bool,
) -> Result<
    (
        KernelCheckResult,
        crate::policy::PolicyDecision,
        mizar_vc::kernel_evidence_handoff::VcKernelEvidenceHandoff,
        Vec<u8>,
        Option<mizar_artifact::proof_witness::KernelAcceptanceMetadata>,
    ),
    String,
> {
    use crate::policy::{PolicyCandidate, ProofPolicyEvaluator};
    use mizar_kernel::{
        certificate_parser::{ClauseTautologyPolicy, Fingerprint, KernelProfileRecord},
        formula_evidence::{FormulaEvidenceParseContext, parse_formula_evidence},
    };
    use mizar_vc::{
        generator::generate_source_theorem_vcs,
        kernel_evidence_handoff::{
            KernelClauseTautologyPolicy, build_source_theorem_kernel_handoff,
        },
        vc_ir::{GenerationSchemaVersion, VcSchemaVersion},
    };

    let fresh =
        mizar_checker::type_checker::SourceVariableSemanticsChecker::check_theorem_skeletons(
            source,
            check.typed_ast(),
            check.scope(),
            symbols,
            check.labels(),
            check.resolved_labels(),
            None,
        )?;
    let invalid = || "theorem.proof.invalid_source_evidence".to_owned();
    if check.owners() != fresh.owners() {
        return Err(invalid());
    }
    let core = mizar_core::elaborator::lower_source_theorem_skeletons(&fresh, None)?;
    let vcs = generate_source_theorem_vcs(
        &core,
        snapshot,
        &GenerationSchemaVersion::new("source-theorem-local-v1"),
        &VcSchemaVersion::new("vc-v1"),
    )?;
    let handoff = build_source_theorem_kernel_handoff(&core, &vcs, vc_id)?;
    let envelope = handoff.canonical_evidence();
    let target = envelope.target_vc();
    let profile = envelope.kernel_profile();
    let bytes = handoff.encode_local_evidence(None)?;
    let tautology = match profile.clause_tautology_policy {
        KernelClauseTautologyPolicy::Reject => ClauseTautologyPolicy::Reject,
        KernelClauseTautologyPolicy::Marker => ClauseTautologyPolicy::Marker,
        _ => return Err(invalid()),
    };
    let mut parse_context = FormulaEvidenceParseContext::v1(
        Fingerprint::new(target.algorithm_id, target.digest.clone()),
        KernelProfileRecord::v1(profile.profile_id, tautology),
    );
    let parsed = parse_formula_evidence(&bytes, &parse_context)
        .map_err(|error| format!("theorem.proof.parse:{error:?}"))?;
    let names = mizar_atp::backend::encode_local_variable_reflexivity_names(&parsed)?;
    let bytes = handoff.encode_local_evidence(Some(&names))?;
    parse_context.accepted_schema_versions.insert(2);
    let parsed = parse_formula_evidence(&bytes, &parse_context)
        .map_err(|error| format!("theorem.proof.parse:{error:?}"))?;
    let origin = KernelEvidenceOrigin::BuiltinDischarge;
    let evaluator = ProofPolicyEvaluator::new(policy.clone());
    if !evaluator.can_schedule_kernel_check(&PolicyCandidate::UncheckedBuiltinDischarge {
        has_stable_kernel_representation: true,
    }) {
        return Err("theorem.proof.kernel_policy_rejected".to_owned());
    }
    let result =
        check_source_kernel_handoff(&handoff, &parsed, "theorem.proof.invalid_source_evidence")?;
    let input = KernelPolicyInput::from_kernel_result(&result, origin);
    let decision = evaluator.evaluate_candidate(&PolicyCandidate::KernelResult(input));
    let metadata = if include_metadata {
        use mizar_artifact::{
            proof_witness::KernelAcceptanceMetadata,
            registration_summary::{ArtifactHashClass, ArtifactHashRef},
        };

        let input = KernelPolicyInput::from_kernel_result(&result, origin);
        let accepted = input.accepted_evidence_hash();
        let polarity = input.accepted_goal_polarity();
        if decision.class == crate::policy::CandidatePolicyClass::DischargedBuiltin
            && result.status() == KernelCheckStatus::Accepted
            && !result.policy_taint()
            && result.evidence_check_kind()
                == Some(mizar_kernel::checker::KernelEvidenceCheckKind::ProofObligation)
            && polarity == Some(AcceptedGoalPolarity::AssertFalseForRefutation)
        {
            accepted
                .zip(handoff.local_evidence_component_hashes().ok())
                .map(|(accepted, components)| {
                    let [
                        kernel_profile_fingerprint,
                        formula_evidence_hash,
                        substitution_evidence_hash,
                        provenance_hash,
                    ] = [
                        ("profile", components[0]),
                        ("formula", components[1]),
                        ("substitutions", components[2]),
                        ("provenance", components[3]),
                    ]
                    .map(|(kind, digest)| ArtifactHashRef {
                        class: ArtifactHashClass::Interface,
                        schema_family: format!("mizar-vc/local-evidence-component/{kind}"),
                        schema_version: SchemaVersion::new(1, 0),
                        digest,
                    });
                    KernelAcceptanceMetadata {
                        kernel_profile_fingerprint,
                        verifier_policy_fingerprint: ArtifactHashRef {
                            class: ArtifactHashClass::Interface,
                            schema_family: "mizar-proof/verifier-policy".into(),
                            schema_version: SchemaVersion::new(policy.schema_version(), 0),
                            digest: policy.policy_fingerprint().hash(),
                        },
                        checker_schema_version: SchemaVersion::new(
                            policy.checker_schema_version(),
                            0,
                        ),
                        evidence_schema_version: SchemaVersion::new(parsed.schema_version(), 0),
                        target_binding_hash: ArtifactHashRef {
                            class: ArtifactHashClass::Interface,
                            schema_family: "mizar-vc/kernel-evidence-handoff-canonical".into(),
                            schema_version: SchemaVersion::new(1, 0),
                            digest: handoff.canonical_hash(),
                        },
                        formula_evidence_hash,
                        substitution_evidence_hash,
                        provenance_hash,
                        formula_context_hash: None,
                        accepted_result_hash: ArtifactHashRef {
                            class: ArtifactHashClass::Interface,
                            schema_family: "mizar-proof/kernel-accepted-evidence".into(),
                            schema_version: SchemaVersion::new(1, 0),
                            digest: accepted,
                        },
                    }
                })
        } else {
            None
        }
    } else {
        None
    };
    Ok((result, decision, handoff, bytes, metadata))
}

/// Returns ordinary acceptance metadata and the exact wire from one genuine theorem check.
pub fn check_source_theorem_reflexivity_metadata(
    source: &mizar_resolve::resolved_ast::SurfaceResolvedArena,
    check: &mizar_checker::type_checker::SourceTheoremCheck<'_>,
    symbols: &mizar_resolve::env::SymbolEnv,
    snapshot: mizar_session::BuildSnapshotId,
    vc_id: VcId,
    policy: &VerifierPolicy,
) -> Result<
    (
        KernelCheckResult,
        crate::policy::PolicyDecision,
        Option<mizar_artifact::proof_witness::KernelAcceptanceMetadata>,
        Vec<u8>,
    ),
    String,
> {
    check_source_theorem_reflexivity_payload(source, check, symbols, snapshot, vc_id, policy, true)
        .map(|(result, decision, _, bytes, metadata)| (result, decision, metadata, bytes))
}

/// Returns opaque trusted metadata and the exact wire from one fresh source theorem check.
pub fn source_theorem_reflexivity_trusted_witness_metadata_with_wire(
    source: &mizar_resolve::resolved_ast::SurfaceResolvedArena,
    check: &mizar_checker::type_checker::SourceTheoremCheck<'_>,
    symbols: &mizar_resolve::env::SymbolEnv,
    snapshot: mizar_session::BuildSnapshotId,
    vc_id: VcId,
    policy: &VerifierPolicy,
) -> Result<(crate::witness_store::TrustedKernelWitnessMetadata, Vec<u8>), String> {
    let origin = KernelEvidenceOrigin::BuiltinDischarge;
    let (result, decision, metadata, wire) =
        check_source_theorem_reflexivity_metadata(source, check, symbols, snapshot, vc_id, policy)?;
    let metadata = match (decision.class, metadata) {
        (crate::policy::CandidatePolicyClass::DischargedBuiltin, Some(metadata)) => metadata,
        _ => return Err("theorem.proof.invalid_source_trusted_witness_metadata".to_owned()),
    };
    let trusted = crate::witness_store::TrustedKernelWitnessMetadata::from_kernel_policy_input(
        &KernelPolicyInput::from_kernel_result(&result, origin),
        metadata,
    )
    .map_err(|error| error.to_string())?;
    Ok((trusted, wire))
}

/// Returns opaque trusted metadata from one fresh source theorem check.
pub fn source_theorem_reflexivity_trusted_witness_metadata(
    source: &mizar_resolve::resolved_ast::SurfaceResolvedArena,
    check: &mizar_checker::type_checker::SourceTheoremCheck<'_>,
    symbols: &mizar_resolve::env::SymbolEnv,
    snapshot: mizar_session::BuildSnapshotId,
    vc_id: VcId,
    policy: &VerifierPolicy,
) -> Result<crate::witness_store::TrustedKernelWitnessMetadata, String> {
    source_theorem_reflexivity_trusted_witness_metadata_with_wire(
        source, check, symbols, snapshot, vc_id, policy,
    )
    .map(|(metadata, _)| metadata)
}

/// Selects freshly checked original theorem reflexivity without an ATP encoding.
pub fn select_source_theorem_reflexivity(
    source: &mizar_resolve::resolved_ast::SurfaceResolvedArena,
    check: &mizar_checker::type_checker::SourceTheoremCheck<'_>,
    symbols: &mizar_resolve::env::SymbolEnv,
    snapshot: mizar_session::BuildSnapshotId,
    vc_id: VcId,
    policy: &VerifierPolicy,
) -> Result<crate::selection::ProofSelection, String> {
    let origin = KernelEvidenceOrigin::BuiltinDischarge;
    let (result, decision, _, _, _) = check_source_theorem_reflexivity_payload(
        source, check, symbols, snapshot, vc_id, policy, false,
    )?;
    select_source_theorem_reflexivity_result(&result, decision, origin, policy)
}

fn select_source_theorem_reflexivity_result(
    result: &KernelCheckResult,
    decision: crate::policy::PolicyDecision,
    origin: KernelEvidenceOrigin,
    policy: &VerifierPolicy,
) -> Result<crate::selection::ProofSelection, String> {
    use crate::selection::{ProofEvidenceCandidate, ProofEvidenceSet, select_winner};
    let invalid = || "theorem.proof.invalid_source_selection".to_owned();
    let target = result.target_vc_fingerprint().sort_bytes();
    let id = CandidateSourceId::new(format!(
        "source-theorem-reflexivity-v1:{}",
        source_discharge_hex(&target)
    ))
    .map_err(|_| invalid())?;
    let candidate = if decision.class == crate::policy::CandidatePolicyClass::DischargedBuiltin {
        ProofEvidenceCandidate::from_trusted_kernel_input(
            id,
            &KernelPolicyInput::from_kernel_result(result, origin),
        )
        .ok_or_else(invalid)?
    } else {
        ProofEvidenceCandidate::new(id, decision)
    };
    Ok(select_winner(
        &ProofEvidenceSet::new(target, None, policy.clone()).with_candidates([candidate]),
    ))
}

/// Projects original theorem identity, fresh selection and an unpublished checked-wire reference.
pub fn project_source_theorem_reflexivity_witness(
    source: &mizar_resolve::resolved_ast::SurfaceResolvedArena,
    check: &mizar_checker::type_checker::SourceTheoremCheck<'_>,
    symbols: &mizar_resolve::env::SymbolEnv,
    snapshot: mizar_session::BuildSnapshotId,
    vc_id: VcId,
    policy: &VerifierPolicy,
) -> Result<
    (
        crate::selection::ProofSelection,
        mizar_artifact::proof_witness::ProofWitnessRef,
        Vec<u8>,
    ),
    String,
> {
    use mizar_artifact::{
        proof_witness::{
            EvidenceKind, ProofStatus, ProofWitnessRef, builtin_proof_witness_ref_json,
        },
        registration_summary::{ArtifactHashClass, ArtifactHashRef},
        store::{CanonicalHashDomain, CanonicalJson, HashClass},
    };
    let (identity, _) =
        source_theorem_obligation_identity(source, check, symbols, snapshot, vc_id)?;
    let origin = KernelEvidenceOrigin::BuiltinDischarge;
    let (result, decision, handoff, bytes, metadata) = check_source_theorem_reflexivity_payload(
        source, check, symbols, snapshot, vc_id, policy, true,
    )?;
    let invalid = || "theorem.proof.invalid_source_witness".to_owned();
    if decision.class != crate::policy::CandidatePolicyClass::DischargedBuiltin {
        return Err(invalid());
    }
    let metadata = metadata.ok_or_else(invalid)?;
    let input = KernelPolicyInput::from_kernel_result(&result, origin);
    let polarity = input.accepted_goal_polarity().ok_or_else(invalid)?;
    let selection = select_source_theorem_reflexivity_result(&result, decision, origin, policy)
        .map_err(|_| invalid())?;
    if selection.selected_class() != ProofWinnerClass::DischargedBuiltin {
        return Err(invalid());
    }
    let target_hex = source_discharge_hex(&result.target_vc_fingerprint().sort_bytes());
    let payload_family = "mizar-proof/source-theorem-reflexivity-witness-payload";
    let witness_artifact_hash = ArtifactHashRef {
        class: ArtifactHashClass::Artifact,
        schema_family: payload_family.into(),
        schema_version: SchemaVersion::new(1, 0),
        digest: CanonicalHashDomain::new(
            HashClass::Artifact,
            payload_family,
            SchemaVersion::new(1, 0),
        )
        .hash(&CanonicalJson::string(source_discharge_hex(&bytes)), &[]),
    };
    let obligation = CanonicalJson::object([
        (
            "original_obligation_fingerprint",
            CanonicalJson::string(source_discharge_hex(
                identity.obligation_fingerprint().as_bytes(),
            )),
        ),
        ("checked_target_bytes", CanonicalJson::string(target_hex)),
        (
            "local_context_identity_hash",
            CanonicalJson::string(source_discharge_hex(
                handoff.context_identity_hash().as_bytes(),
            )),
        ),
        (
            "kernel_profile_fingerprint",
            CanonicalJson::string(
                metadata
                    .kernel_profile_fingerprint
                    .to_artifact_hash_string(),
            ),
        ),
        (
            "verifier_policy_fingerprint",
            CanonicalJson::string(
                metadata
                    .verifier_policy_fingerprint
                    .to_artifact_hash_string(),
            ),
        ),
        (
            "checker_schema_version",
            CanonicalJson::string(metadata.checker_schema_version.to_string()),
        ),
        (
            "evidence_schema_version",
            CanonicalJson::string(metadata.evidence_schema_version.to_string()),
        ),
        (
            "target_binding_hash",
            CanonicalJson::string(metadata.target_binding_hash.to_artifact_hash_string()),
        ),
        (
            "formula_evidence_hash",
            CanonicalJson::string(metadata.formula_evidence_hash.to_artifact_hash_string()),
        ),
        (
            "substitution_evidence_hash",
            CanonicalJson::string(
                metadata
                    .substitution_evidence_hash
                    .to_artifact_hash_string(),
            ),
        ),
        (
            "provenance_hash",
            CanonicalJson::string(metadata.provenance_hash.to_artifact_hash_string()),
        ),
        (
            "formula_context_hash",
            metadata
                .formula_context_hash
                .as_ref()
                .map_or_else(CanonicalJson::null, |hash| {
                    CanonicalJson::string(hash.to_artifact_hash_string())
                }),
        ),
        (
            "accepted_result_hash",
            CanonicalJson::string(metadata.accepted_result_hash.to_artifact_hash_string()),
        ),
        (
            "accepted_goal_polarity",
            CanonicalJson::string(polarity.as_str()),
        ),
        (
            "witness_artifact_hash",
            CanonicalJson::string(witness_artifact_hash.to_artifact_hash_string()),
        ),
    ])
    .map_err(|_| invalid())?;
    let obligation_family = "mizar-proof/source-theorem-reflexivity-obligation";
    let reference = ProofWitnessRef {
        schema_version: SchemaVersion::new(3, 0),
        obligation_id: identity.obligation_id().into(),
        obligation_fingerprint: ArtifactHashRef {
            class: ArtifactHashClass::Interface,
            schema_family: obligation_family.into(),
            schema_version: SchemaVersion::new(1, 0),
            digest: CanonicalHashDomain::new(
                HashClass::Interface,
                obligation_family,
                SchemaVersion::new(1, 0),
            )
            .hash(&obligation, &[]),
        },
        proof_status: ProofStatus::DischargedBuiltin,
        evidence_kind: EvidenceKind::FormulaSubstitutionKernelEvidence,
        witness_path: format!(
            "proof-witnesses/source-theorem-reflexivity-v1/{}.bin",
            source_discharge_hex(witness_artifact_hash.digest.as_bytes())
        ),
        witness_artifact_hash,
        kernel_acceptance: metadata,
    };
    builtin_proof_witness_ref_json(&reference).map_err(|_| invalid())?;
    Ok((selection, reference, bytes))
}

/// Acquires an internal draft through the original source's genuine proof pipeline.
pub fn source_theorem_reflexivity_witness_draft_from_original(
    source: &mizar_resolve::resolved_ast::SurfaceResolvedArena,
    symbols: &mizar_resolve::env::SymbolEnv,
    snapshot: &mizar_session::BuildSnapshot,
    policy: &VerifierPolicy,
    witness_path: &str,
) -> Result<
    (
        ProofStatusProjection,
        mizar_vc::dependency_slice::DependencySlice,
        crate::witness_store::ProofWitnessDraft,
    ),
    String,
> {
    use mizar_checker::type_checker::{SourceVariableSemanticsChecker, TermFormulaChecker};
    use mizar_vc::vc_ir::{GenerationSchemaVersion, VcSchemaVersion};

    let (scope, labels, resolved) = mizar_resolve::symbols::source_proof_inputs(source, symbols)?;
    let (_, _, typed) =
        TermFormulaChecker::check_source_theorem_equalities(source, symbols, &scope)?;
    let check = SourceVariableSemanticsChecker::check_theorem_skeletons(
        source, &typed, &scope, symbols, &labels, &resolved, None,
    )?;
    let core = mizar_core::elaborator::lower_source_theorem_skeletons(&check, None)?;
    let vcs = mizar_vc::generator::generate_source_theorem_vcs(
        &core,
        snapshot.id,
        &GenerationSchemaVersion::new("source-theorem-local-v1"),
        &VcSchemaVersion::new("vc-v1"),
    )?;
    let [vc] = vcs.vcs() else {
        return Err("theorem.proof.invalid_source_witness_draft".to_owned());
    };
    source_theorem_reflexivity_witness_draft(
        source,
        &check,
        symbols,
        snapshot,
        vc.id,
        policy,
        witness_path,
    )
}

/// Builds an internal draft from one fresh theorem check and retained snapshot metadata.
pub fn source_theorem_reflexivity_witness_draft(
    source: &mizar_resolve::resolved_ast::SurfaceResolvedArena,
    check: &mizar_checker::type_checker::SourceTheoremCheck<'_>,
    symbols: &mizar_resolve::env::SymbolEnv,
    snapshot: &mizar_session::BuildSnapshot,
    vc_id: VcId,
    policy: &VerifierPolicy,
    witness_path: &str,
) -> Result<
    (
        ProofStatusProjection,
        mizar_vc::dependency_slice::DependencySlice,
        crate::witness_store::ProofWitnessDraft,
    ),
    String,
> {
    use mizar_artifact::registration_summary::{ArtifactHashClass, ArtifactHashRef};

    use crate::{
        selection::{VcProofSelection, merge_artifact_proof_selections},
        witness_store::{
            ProofWitnessDraft, ProofWitnessPayloadSchema, ProofWitnessProvenance,
            TrustedKernelWitnessMetadata,
        },
    };

    let (identity, slice) =
        source_theorem_obligation_identity(source, check, symbols, snapshot.id, vc_id)?;
    let origin = KernelEvidenceOrigin::BuiltinDischarge;
    let (result, decision, metadata, wire) = check_source_theorem_reflexivity_metadata(
        source,
        check,
        symbols,
        snapshot.id,
        vc_id,
        policy,
    )?;
    let invalid = || "theorem.proof.invalid_source_witness_draft".to_owned();
    let metadata = match (decision.class, metadata) {
        (crate::policy::CandidatePolicyClass::DischargedBuiltin, Some(metadata)) => metadata,
        _ => return Err(invalid()),
    };
    let selection = select_source_theorem_reflexivity_result(&result, decision, origin, policy)?;
    let merged = merge_artifact_proof_selections([], [VcProofSelection::new(vc_id, selection)])
        .map_err(|error| error.to_string())?;
    let [selection] = <[_; 1]>::try_from(merged).map_err(|_| invalid())?;
    let projection = project_status(ProofStatusProjectionInput::new(
        selection,
        policy.clone(),
        identity,
    ))
    .map_err(|error| error.to_string())?;
    let reuse = projection.reuse_metadata();
    let provenance = ProofWitnessProvenance::new(
        snapshot.id.fingerprint(),
        "mizar-proof/source-theorem-reflexivity-witness-draft/v1",
        reuse.selected_candidate_id().cloned().ok_or_else(invalid)?,
        origin,
        reuse.vc_fingerprint(),
        reuse.dependency_slice_fingerprint(),
        snapshot.dependency_artifact_fingerprint(),
        policy.policy_fingerprint().hash(),
        metadata.checker_schema_version,
        metadata.evidence_schema_version,
    )
    .map_err(|error| error.to_string())?
    .with_accepted_result_hash(metadata.accepted_result_hash.digest);
    let payload_schema = ProofWitnessPayloadSchema::with_canonical_bytes_required(
        "mizar-proof/source-theorem-reflexivity-checked-wire",
        metadata.evidence_schema_version,
        true,
    )
    .map_err(|error| error.to_string())?;
    let trusted = TrustedKernelWitnessMetadata::from_kernel_policy_input(
        &KernelPolicyInput::from_kernel_result(&result, origin),
        metadata,
    )
    .map_err(|error| error.to_string())?;
    let draft = ProofWitnessDraft::new(
        projection.obligation_id(),
        reuse.obligation_anchor().clone(),
        ArtifactHashRef::new(
            ArtifactHashClass::Interface,
            "mizar-proof/source-theorem-original-obligation",
            SchemaVersion::new(1, 0),
            reuse.obligation_fingerprint(),
        ),
        &projection,
        trusted,
        payload_schema,
        wire,
        witness_path,
        provenance,
    )
    .map_err(|error| error.to_string())?;
    Ok((projection, slice, draft))
}

/// Projects genuine source theorem reflexivity with its full original dependency slice.
pub fn project_source_theorem_reflexivity_status(
    source: &mizar_resolve::resolved_ast::SurfaceResolvedArena,
    check: &mizar_checker::type_checker::SourceTheoremCheck<'_>,
    symbols: &mizar_resolve::env::SymbolEnv,
    snapshot: mizar_session::BuildSnapshotId,
    vc_id: VcId,
    policy: &VerifierPolicy,
) -> Result<
    (
        ProofStatusProjection,
        mizar_vc::dependency_slice::DependencySlice,
    ),
    String,
> {
    use crate::selection::{VcProofSelection, merge_artifact_proof_selections};
    let (identity, slice) =
        source_theorem_obligation_identity(source, check, symbols, snapshot, vc_id)?;
    let selection =
        select_source_theorem_reflexivity(source, check, symbols, snapshot, vc_id, policy)?;
    let merged = merge_artifact_proof_selections([], [VcProofSelection::new(vc_id, selection)])
        .map_err(|error| error.to_string())?;
    let [selection] =
        <[_; 1]>::try_from(merged).map_err(|_| "theorem.proof.invalid_source_status".to_owned())?;
    let projection = project_status(ProofStatusProjectionInput::new(
        selection,
        policy.clone(),
        identity,
    ))
    .map_err(|error| error.to_string())?;
    Ok((projection, slice))
}

/// Projects genuine source theorem reflexivity with same-result trusted used axioms.
pub fn project_source_theorem_reflexivity_used_axioms_status(
    source: &mizar_resolve::resolved_ast::SurfaceResolvedArena,
    check: &mizar_checker::type_checker::SourceTheoremCheck<'_>,
    symbols: &mizar_resolve::env::SymbolEnv,
    snapshot: mizar_session::BuildSnapshotId,
    vc_id: VcId,
    policy: &VerifierPolicy,
) -> Result<
    (
        ProofStatusProjection,
        mizar_vc::dependency_slice::DependencySlice,
    ),
    String,
> {
    use crate::selection::{VcProofSelection, merge_artifact_proof_selections};
    let (identity, slice) =
        source_theorem_obligation_identity(source, check, symbols, snapshot, vc_id)?;
    let origin = KernelEvidenceOrigin::BuiltinDischarge;
    let (result, decision, _, _, _) = check_source_theorem_reflexivity_payload(
        source, check, symbols, snapshot, vc_id, policy, false,
    )?;
    let invalid = || "theorem.proof.invalid_source_used_axioms_status".to_owned();
    if decision.class != crate::policy::CandidatePolicyClass::DischargedBuiltin {
        return Err(invalid());
    }
    let selection = select_source_theorem_reflexivity_result(&result, decision, origin, policy)?;
    if selection.selected_class() != ProofWinnerClass::DischargedBuiltin {
        return Err(invalid());
    }
    let reference = TrustedUsedAxiomsRef::from_kernel_result(&result, origin)
        .map_err(|error| error.to_string())?;
    let merged = merge_artifact_proof_selections([], [VcProofSelection::new(vc_id, selection)])
        .map_err(|error| error.to_string())?;
    let [selection] = <[_; 1]>::try_from(merged).map_err(|_| invalid())?;
    let projection = project_status(
        ProofStatusProjectionInput::new(selection, policy.clone(), identity)
            .with_trusted_used_axioms(reference),
    )
    .map_err(|error| error.to_string())?;
    Ok((projection, slice))
}

/// Binds genuine original theorem content to its full conservative Open dependency slice.
/// Source/snapshot freshness remains caller-owned; this grants no acceptance or reuse authority.
pub fn source_theorem_obligation_identity(
    source: &mizar_resolve::resolved_ast::SurfaceResolvedArena,
    check: &mizar_checker::type_checker::SourceTheoremCheck<'_>,
    symbols: &mizar_resolve::env::SymbolEnv,
    snapshot: mizar_session::BuildSnapshotId,
    vc_id: VcId,
) -> Result<
    (
        ProofObligationIdentity,
        mizar_vc::dependency_slice::DependencySlice,
    ),
    String,
> {
    use mizar_artifact::store::{
        CanonicalHashDomain, CanonicalJson, HashClass, canonical_json_bytes,
    };
    use mizar_checker::source_statement::{
        SourceStatementRecovery, SourceTheoremRole, SourceTheoremStatus,
    };

    let fresh =
        mizar_checker::type_checker::SourceVariableSemanticsChecker::check_theorem_skeletons(
            source,
            check.typed_ast(),
            check.scope(),
            symbols,
            check.labels(),
            check.resolved_labels(),
            None,
        )?;
    let invalid = || "theorem.proof.invalid_source_identity".to_owned();
    if check.owners() != fresh.owners() {
        return Err(invalid());
    }
    let (vc, goal, context, slice) = source_theorem_canonical_inputs(&fresh, snapshot, vc_id)?;
    let syntax =
        mizar_resolve::symbols::source_theorem_syntax_payloads_from_arena(source, symbols)?;
    let [(owner, visibility)] = fresh.owners() else {
        return Err(invalid());
    };
    if syntax.len() != 1
        || owner.role != SourceTheoremRole::Theorem
        || owner.status != SourceTheoremStatus::Unmodified
        || owner.recovery != SourceStatementRecovery::Normal
        || !matches!(*visibility, "public" | "private")
    {
        return Err(invalid());
    }
    let syntax = syntax.get(&owner.symbol).ok_or_else(invalid)?;
    let module = owner.symbol.module();
    let owner = CanonicalJson::object([
        (
            "package_id",
            CanonicalJson::string(module.package().as_str()),
        ),
        ("module_path", CanonicalJson::string(module.path().as_str())),
        ("namespace", CanonicalJson::string(module.path().as_str())),
        ("spelling", CanonicalJson::string(&owner.spelling)),
        ("role", CanonicalJson::string("theorem")),
        ("status", CanonicalJson::string("unmodified")),
        ("visibility", CanonicalJson::string(*visibility)),
    ])
    .map_err(|error| error.to_string())?;
    let candidate = mizar_vc::vc_ir::source_theorem_candidate_anchor_hash(
        &canonical_json_bytes(&owner),
        syntax,
        vc,
        goal,
        context,
    )?;
    let candidate = ObligationAnchor::new(format!(
        "source-theorem-candidate-anchor-v1:{}",
        source_discharge_hex(candidate.as_bytes()),
    ))
    .map_err(|error| error.to_string())?;
    let obligation = CanonicalJson::object([
        (
            "obligation_anchor",
            CanonicalJson::string(candidate.as_str()),
        ),
        (
            "vc_fingerprint",
            CanonicalJson::string(source_discharge_hex(vc.hash().as_bytes())),
        ),
        (
            "canonical_goal_hash",
            CanonicalJson::string(source_discharge_hex(goal.as_bytes())),
        ),
        (
            "local_context_fingerprint",
            CanonicalJson::string(source_discharge_hex(context.hash().as_bytes())),
        ),
        (
            "vc_local_dependency_slice_fingerprint",
            CanonicalJson::string(source_discharge_hex(slice.fingerprint().hash().as_bytes())),
        ),
    ])
    .map_err(|error| error.to_string())?;
    let fingerprint = CanonicalHashDomain::new(
        HashClass::Interface,
        "mizar-proof/source-theorem-original-obligation",
        SchemaVersion::new(1, 0),
    )
    .hash(&obligation, &[]);
    let identity = ProofObligationIdentity::new(
        candidate.as_str().to_owned(),
        candidate,
        fingerprint,
        vc.hash(),
        context.hash(),
        slice.fingerprint().hash(),
    )
    .map_err(|error| error.to_string())?;
    Ok((identity, slice))
}

/// Derives raw canonical VC, goal and visible-context content from one source call.
/// These components do not authorize acceptance, proof reuse or publication.
pub fn source_registration_canonical_components(
    source: &mizar_resolve::resolved_ast::SurfaceResolvedArena,
    nodes: &mizar_checker::typed_ast::TypedArena,
    symbols: &mizar_resolve::env::SymbolEnv,
    snapshot: mizar_session::BuildSnapshotId,
    vc_id: VcId,
) -> Result<(mizar_vc::vc_ir::CanonicalVcFingerprint, Hash, Hash), String> {
    use mizar_vc::vc_ir::{
        GenerationSchemaVersion, PremiseRef, VcFormulaRef, VcGeneratedFormulaShape, VcSchemaVersion,
    };
    use std::collections::BTreeSet;

    const INVALID: &str = "registration.proof.invalid_source_components";
    let check =
        mizar_checker::registration_resolution::check_source_existential_registration_proof(
            source, nodes, symbols,
        )?;
    let core = mizar_core::elaborator::lower_source_existential_registration(&check)?;
    let vcs = mizar_vc::generator::generate_source_existential_registration(
        &core,
        snapshot,
        &GenerationSchemaVersion::new("source-existential-registration-v1"),
        &VcSchemaVersion::new("vc-v1"),
    )?;
    let owners =
        mizar_resolve::symbols::source_registration_owner_payloads_from_arena(source, symbols)?;
    let mut references = Vec::new();
    for vc in vcs.vcs() {
        references.push(vc.goal);
        for premise in &vc.premises {
            let PremiseRef::GeneratedFact { formula } = premise else {
                return Err(INVALID.to_owned());
            };
            references.push(*formula);
        }
    }
    let mut roots = BTreeSet::new();
    let mut visited = BTreeSet::new();
    while let Some(reference) = references.pop() {
        match reference {
            VcFormulaRef::Core(id) => {
                roots.insert(id);
            }
            VcFormulaRef::Generated(id) => {
                if !visited.insert(id) {
                    continue;
                }
                match &vcs
                    .generated_formula(id)
                    .ok_or_else(|| INVALID.to_owned())?
                    .shape
                {
                    VcGeneratedFormulaShape::Ref(child) | VcGeneratedFormulaShape::Not(child) => {
                        references.push(*child);
                    }
                    VcGeneratedFormulaShape::And(children)
                    | VcGeneratedFormulaShape::Or(children) => {
                        references.extend(children);
                    }
                    _ => return Err(INVALID.to_owned()),
                }
            }
            _ => return Err(INVALID.to_owned()),
        }
    }
    let formulas = mizar_core::elaborator::source_registration_formula_payloads(
        &check,
        &core,
        &owners,
        &roots.into_iter().collect::<Vec<_>>(),
    )?;
    let fingerprints = vcs.source_registration_canonical_fingerprints(&core, &formulas)?;
    let full = fingerprints.get(&vc_id).ok_or_else(|| INVALID.to_owned())?;
    let (goal, context) =
        vcs.source_registration_canonical_goal_context(vc_id, &core, &formulas)?;
    Ok((*full, goal, context))
}

/// Binds genuine source producers into a candidate anchor and canonical content.
/// Matching this candidate alone does not authorize proof reuse or acceptance.
pub fn source_registration_candidate_anchor(
    source: &mizar_resolve::resolved_ast::SurfaceResolvedArena,
    nodes: &mizar_checker::typed_ast::TypedArena,
    symbols: &mizar_resolve::env::SymbolEnv,
    snapshot: mizar_session::BuildSnapshotId,
    vc_id: VcId,
) -> Result<
    (
        ObligationAnchor,
        mizar_vc::vc_ir::CanonicalVcFingerprint,
        Hash,
        Hash,
    ),
    String,
> {
    use mizar_vc::vc_ir::{
        GenerationSchemaVersion, PremiseRef, VcFormulaRef, VcGeneratedFormulaShape, VcSchemaVersion,
    };
    use std::collections::BTreeSet;

    const INVALID: &str = "registration.proof.invalid_source_anchor";
    let check =
        mizar_checker::registration_resolution::check_source_existential_registration_proof(
            source, nodes, symbols,
        )?;
    let core = mizar_core::elaborator::lower_source_existential_registration(&check)?;
    let vcs = mizar_vc::generator::generate_source_existential_registration(
        &core,
        snapshot,
        &GenerationSchemaVersion::new("source-existential-registration-v1"),
        &VcSchemaVersion::new("vc-v1"),
    )?;
    let owners =
        mizar_resolve::symbols::source_registration_owner_payloads_from_arena(source, symbols)?;
    let mut references = Vec::new();
    for vc in vcs.vcs() {
        references.push(vc.goal);
        for premise in &vc.premises {
            let PremiseRef::GeneratedFact { formula } = premise else {
                return Err(INVALID.to_owned());
            };
            references.push(*formula);
        }
    }
    let mut roots = BTreeSet::new();
    let mut visited = BTreeSet::new();
    while let Some(reference) = references.pop() {
        match reference {
            VcFormulaRef::Core(id) => {
                roots.insert(id);
            }
            VcFormulaRef::Generated(id) => {
                if !visited.insert(id) {
                    continue;
                }
                match &vcs
                    .generated_formula(id)
                    .ok_or_else(|| INVALID.to_owned())?
                    .shape
                {
                    VcGeneratedFormulaShape::Ref(child) | VcGeneratedFormulaShape::Not(child) => {
                        references.push(*child);
                    }
                    VcGeneratedFormulaShape::And(children)
                    | VcGeneratedFormulaShape::Or(children) => {
                        references.extend(children);
                    }
                    _ => return Err(INVALID.to_owned()),
                }
            }
            _ => return Err(INVALID.to_owned()),
        }
    }
    let formulas = mizar_core::elaborator::source_registration_formula_payloads(
        &check,
        &core,
        &owners,
        &roots.into_iter().collect::<Vec<_>>(),
    )?;
    let fingerprints = vcs.source_registration_canonical_fingerprints(&core, &formulas)?;
    let syntax =
        mizar_resolve::symbols::source_registration_syntax_payloads_from_arena(source, symbols)?;
    let mut item_owners = std::collections::BTreeMap::new();
    let mut item_syntax = std::collections::BTreeMap::new();
    for (id, item) in core.items().iter() {
        if item.kind != mizar_core::core_ir::CoreItemKind::Registration {
            continue;
        }
        item_owners.insert(
            id,
            owners
                .get(&item.symbol)
                .ok_or_else(|| INVALID.to_owned())?
                .clone(),
        );
        item_syntax.insert(
            id,
            syntax
                .get(&item.symbol)
                .ok_or_else(|| INVALID.to_owned())?
                .clone(),
        );
    }
    if item_syntax.len() != syntax.len() {
        return Err(INVALID.to_owned());
    }
    let candidates = vcs.source_registration_candidate_anchor_hashes(
        &core,
        &formulas,
        &item_syntax,
        &item_owners,
    )?;
    let candidate = candidates.get(&vc_id).ok_or_else(|| INVALID.to_owned())?;
    let full = fingerprints.get(&vc_id).ok_or_else(|| INVALID.to_owned())?;
    let (goal, context) =
        vcs.source_registration_canonical_goal_context(vc_id, &core, &formulas)?;
    let anchor = ObligationAnchor::new(format!(
        "source-registration-candidate-anchor-v1:{}",
        source_discharge_hex(candidate.as_bytes()),
    ))
    .map_err(|_| INVALID.to_owned())?;
    Ok((anchor, *full, goal, context))
}

/// Returns the original source VC's conservative dependency slice.
/// The numeric handle and incomplete fingerprint do not authorize reuse or publication.
pub fn source_registration_dependency_slice(
    source: &mizar_resolve::resolved_ast::SurfaceResolvedArena,
    nodes: &mizar_checker::typed_ast::TypedArena,
    symbols: &mizar_resolve::env::SymbolEnv,
    snapshot: mizar_session::BuildSnapshotId,
    vc_id: VcId,
) -> Result<mizar_vc::dependency_slice::DependencySlice, String> {
    use mizar_vc::{
        dependency_slice::{DependencySliceInput, try_compute_dependency_slices},
        vc_ir::{GenerationSchemaVersion, VcSchemaVersion},
    };

    let check =
        mizar_checker::registration_resolution::check_source_existential_registration_proof(
            source, nodes, symbols,
        )?;
    let core = mizar_core::elaborator::lower_source_existential_registration(&check)?;
    let vcs = mizar_vc::generator::generate_source_existential_registration(
        &core,
        snapshot,
        &GenerationSchemaVersion::new("source-existential-registration-v1"),
        &VcSchemaVersion::new("vc-v1"),
    )?;
    let slices = try_compute_dependency_slices(DependencySliceInput {
        vc_set: &vcs,
        discharge_output: None,
    })
    .map_err(|error| error.to_string())?;
    slices
        .slice_for(vc_id)
        .cloned()
        .ok_or_else(|| "registration.proof.invalid_source_dependency_slice".to_owned())
}

/// Assembles raw original-source identity with its full conservative Open slice.
/// This incomplete VC-local readout does not authorize acceptance, reuse or publication.
pub fn source_registration_obligation_identity(
    source: &mizar_resolve::resolved_ast::SurfaceResolvedArena,
    nodes: &mizar_checker::typed_ast::TypedArena,
    symbols: &mizar_resolve::env::SymbolEnv,
    snapshot: mizar_session::BuildSnapshotId,
    vc_id: VcId,
) -> Result<
    (
        ProofObligationIdentity,
        mizar_vc::dependency_slice::DependencySlice,
    ),
    String,
> {
    use mizar_artifact::store::{CanonicalHashDomain, CanonicalJson, HashClass};

    let (anchor, vc, goal, context) =
        source_registration_candidate_anchor(source, nodes, symbols, snapshot, vc_id)?;
    let slice = source_registration_dependency_slice(source, nodes, symbols, snapshot, vc_id)?;
    let obligation = CanonicalJson::object([
        ("obligation_anchor", CanonicalJson::string(anchor.as_str())),
        (
            "vc_fingerprint",
            CanonicalJson::string(source_discharge_hex(vc.hash().as_bytes())),
        ),
        (
            "canonical_goal_hash",
            CanonicalJson::string(source_discharge_hex(goal.as_bytes())),
        ),
        (
            "local_context_fingerprint",
            CanonicalJson::string(source_discharge_hex(context.as_bytes())),
        ),
        (
            "vc_local_dependency_slice_fingerprint",
            CanonicalJson::string(source_discharge_hex(slice.fingerprint().hash().as_bytes())),
        ),
    ])
    .map_err(|error| error.to_string())?;
    let fingerprint = CanonicalHashDomain::new(
        HashClass::Interface,
        "mizar-proof/source-registration-original-obligation",
        SchemaVersion::new(1, 0),
    )
    .hash(&obligation, &[]);
    let identity = ProofObligationIdentity::new(
        anchor.as_str().to_owned(),
        anchor,
        fingerprint,
        vc.hash(),
        context,
        slice.fingerprint().hash(),
    )
    .map_err(|error| error.to_string())?;
    Ok((identity, slice))
}

/// Freshly checks one authentic NeedsAtp registration candidate under the current policy.
/// The returned check and classification do not authorize activation or publication.
pub fn check_source_registration_candidate(
    source: &mizar_resolve::resolved_ast::SurfaceResolvedArena,
    nodes: &mizar_checker::typed_ast::TypedArena,
    symbols: &mizar_resolve::env::SymbolEnv,
    snapshot: mizar_session::BuildSnapshotId,
    vc_id: VcId,
    encoding: (
        mizar_atp::problem::LogicProfile,
        mizar_atp::problem::ConcreteFormat,
    ),
    policy: &VerifierPolicy,
) -> Result<(KernelCheckResult, crate::policy::PolicyDecision), String> {
    check_source_registration_candidate_payload(
        source,
        nodes,
        symbols,
        snapshot,
        vc_id,
        (encoding.0, encoding.1, false),
        policy,
    )?
    .ok_or_else(|| "registration.proof.invalid_source_candidate".to_owned())
    .map(|(result, decision, _, _, _)| (result, decision))
}

/// Attempts one policy-bounded source candidate screen and fresh normal Kernel check.
/// Absence supplies no checked result, activation or publication authority.
pub fn try_check_source_registration_candidate(
    source: &mizar_resolve::resolved_ast::SurfaceResolvedArena,
    nodes: &mizar_checker::typed_ast::TypedArena,
    symbols: &mizar_resolve::env::SymbolEnv,
    snapshot: mizar_session::BuildSnapshotId,
    vc_id: VcId,
    encoding: (
        mizar_atp::problem::LogicProfile,
        mizar_atp::problem::ConcreteFormat,
    ),
    policy: &VerifierPolicy,
) -> Result<Option<(KernelCheckResult, crate::policy::PolicyDecision)>, String> {
    check_source_registration_candidate_payload(
        source,
        nodes,
        symbols,
        snapshot,
        vc_id,
        (encoding.0, encoding.1, true),
        policy,
    )
    .map(|output| output.map(|(result, decision, _, _, _)| (result, decision)))
}

/// Freshly checks a source ATP candidate and retains raw same-call metadata.
/// The returned candidate and metadata do not authorize publication or proof reuse.
pub fn check_source_registration_candidate_metadata(
    source: &mizar_resolve::resolved_ast::SurfaceResolvedArena,
    nodes: &mizar_checker::typed_ast::TypedArena,
    symbols: &mizar_resolve::env::SymbolEnv,
    snapshot: mizar_session::BuildSnapshotId,
    vc_id: VcId,
    encoding: (
        mizar_atp::problem::LogicProfile,
        mizar_atp::problem::ConcreteFormat,
    ),
    policy: &VerifierPolicy,
) -> Result<
    (
        KernelCheckResult,
        crate::policy::PolicyDecision,
        Option<mizar_artifact::proof_witness::KernelAcceptanceMetadata>,
        mizar_atp::backend::BackendCandidateEvidence,
    ),
    String,
> {
    check_source_registration_candidate_payload(
        source,
        nodes,
        symbols,
        snapshot,
        vc_id,
        (encoding.0, encoding.1, false),
        policy,
    )?
    .ok_or_else(|| "registration.proof.invalid_source_candidate".to_owned())
    .map(|(result, decision, metadata, candidate, _)| (result, decision, metadata, candidate))
}

#[expect(
    clippy::type_complexity,
    reason = "Keeps the shared optional result explicit without a single-use alias"
)]
fn check_source_registration_candidate_payload(
    source: &mizar_resolve::resolved_ast::SurfaceResolvedArena,
    nodes: &mizar_checker::typed_ast::TypedArena,
    symbols: &mizar_resolve::env::SymbolEnv,
    snapshot: mizar_session::BuildSnapshotId,
    vc_id: VcId,
    encoding: (
        mizar_atp::problem::LogicProfile,
        mizar_atp::problem::ConcreteFormat,
        bool,
    ),
    policy: &VerifierPolicy,
) -> Result<
    Option<(
        KernelCheckResult,
        crate::policy::PolicyDecision,
        Option<mizar_artifact::proof_witness::KernelAcceptanceMetadata>,
        mizar_atp::backend::BackendCandidateEvidence,
        mizar_vc::kernel_evidence_handoff::VcKernelEvidenceHandoff,
    )>,
    String,
> {
    use crate::policy::{PolicyCandidate, ProofPolicyEvaluator};
    use mizar_artifact::{
        proof_witness::KernelAcceptanceMetadata,
        registration_summary::{ArtifactHashClass, ArtifactHashRef},
    };
    use mizar_atp::backend::{
        BackendCandidatePayload, build_source_registration_candidate_inputs,
        screen_source_registration_candidate,
    };
    use mizar_checker::registration_resolution::check_source_existential_registration_proof;
    use mizar_kernel::{
        certificate_parser::{ClauseTautologyPolicy, Fingerprint, KernelProfileRecord},
        formula_evidence::{FormulaEvidenceParseContext, parse_formula_evidence},
    };
    use mizar_vc::{
        discharge::{DischargeInput, DischargePolicy, try_discharge},
        generator::generate_source_existential_registration,
        kernel_evidence_handoff::{
            KernelClauseTautologyPolicy, build_source_existential_kernel_handoff,
        },
        vc_ir::{GenerationSchemaVersion, VcSchemaVersion},
    };
    let invalid = || "registration.proof.invalid_source_candidate".to_owned();
    let check = check_source_existential_registration_proof(source, nodes, symbols)?;
    let core = mizar_core::elaborator::lower_source_existential_registration(&check)?;
    let original = generate_source_existential_registration(
        &core,
        snapshot,
        &GenerationSchemaVersion::new("source-existential-registration-v1"),
        &VcSchemaVersion::new("vc-v1"),
    )?;
    let discharge = try_discharge(DischargeInput {
        vc_set: &original,
        policy: &DischargePolicy::default(),
    })
    .map_err(|error| format!("registration.proof.discharge:{error:?}"))?;
    let vcs = discharge.vc_set();
    let (encoded, candidate) = if encoding.2 {
        let Some(pair) = screen_source_registration_candidate(
            &core,
            vcs,
            vc_id,
            encoding.0,
            encoding.1,
            policy.instance_search_limits(),
        )?
        else {
            return Ok(None);
        };
        pair
    } else {
        build_source_registration_candidate_inputs(&core, vcs, vc_id, encoding.0, encoding.1)?
    };
    let BackendCandidatePayload::FormulaSubstitutionBytes(bytes) = candidate.payload() else {
        return Err(invalid());
    };
    let encoded_problem_matches = !bytes.is_empty()
        && candidate.target_binding() == encoded.target_binding()
        && candidate.encoded_problem_hash() == encoded.input_hash()
        && candidate.provenance_hash() == encoded.provenance_hash()
        && candidate.formula_label_refs() == encoded.formula_labels()
        && candidate.symbol_binding_refs() == encoded.symbol_bindings();
    let evaluator = ProofPolicyEvaluator::new(policy.clone());
    if !evaluator.can_schedule_kernel_check(&PolicyCandidate::UncheckedFormulaSubstitution {
        encoded_problem_matches,
    }) {
        return Err("registration.proof.kernel_policy_rejected".into());
    }
    let handoff = build_source_existential_kernel_handoff(&core, vcs, vc_id)?;
    let envelope = handoff.canonical_evidence();
    let profile = envelope.kernel_profile();
    let tautology = match profile.clause_tautology_policy {
        KernelClauseTautologyPolicy::Reject => ClauseTautologyPolicy::Reject,
        KernelClauseTautologyPolicy::Marker => ClauseTautologyPolicy::Marker,
        _ => return Err(invalid()),
    };
    let target = envelope.target_vc();
    let mut parse_context = FormulaEvidenceParseContext::v1(
        Fingerprint::new(target.algorithm_id, target.digest.clone()),
        KernelProfileRecord::v1(profile.profile_id, tautology),
    );
    parse_context.accepted_schema_versions.insert(2);
    let parsed = parse_formula_evidence(bytes, &parse_context)
        .map_err(|error| format!("registration.proof.parse:{error:?}"))?;
    let result = check_source_kernel_handoff(
        &handoff,
        &parsed,
        "registration.proof.invalid_source_candidate",
    )?;
    let input = KernelPolicyInput::from_kernel_result(
        &result,
        KernelEvidenceOrigin::AtpFormulaSubstitution,
    );
    let accepted = input.accepted_evidence_hash();
    let polarity = input.accepted_goal_polarity();
    let decision = evaluator.evaluate_candidate(&PolicyCandidate::KernelResult(input));
    let metadata = if decision.class == crate::policy::CandidatePolicyClass::KernelVerified
        && result.status() == KernelCheckStatus::Accepted
        && !result.policy_taint()
        && result.evidence_check_kind()
            == Some(mizar_kernel::checker::KernelEvidenceCheckKind::ProofObligation)
        && polarity == Some(AcceptedGoalPolarity::AssertFalseForRefutation)
    {
        accepted
            .zip(handoff.local_evidence_component_hashes().ok())
            .map(|(accepted, components)| {
                let [
                    kernel_profile_fingerprint,
                    formula_evidence_hash,
                    substitution_evidence_hash,
                    provenance_hash,
                ] = [
                    ("profile", components[0]),
                    ("formula", components[1]),
                    ("substitutions", components[2]),
                    ("provenance", components[3]),
                ]
                .map(|(kind, digest)| ArtifactHashRef {
                    class: ArtifactHashClass::Interface,
                    schema_family: format!("mizar-vc/local-evidence-component/{kind}"),
                    schema_version: SchemaVersion::new(1, 0),
                    digest,
                });
                KernelAcceptanceMetadata {
                    kernel_profile_fingerprint,
                    verifier_policy_fingerprint: ArtifactHashRef {
                        class: ArtifactHashClass::Interface,
                        schema_family: "mizar-proof/verifier-policy".into(),
                        schema_version: SchemaVersion::new(policy.schema_version(), 0),
                        digest: policy.policy_fingerprint().hash(),
                    },
                    checker_schema_version: SchemaVersion::new(policy.checker_schema_version(), 0),
                    evidence_schema_version: SchemaVersion::new(parsed.schema_version(), 0),
                    target_binding_hash: ArtifactHashRef {
                        class: ArtifactHashClass::Interface,
                        schema_family: "mizar-vc/kernel-evidence-handoff-canonical".into(),
                        schema_version: SchemaVersion::new(1, 0),
                        digest: handoff.canonical_hash(),
                    },
                    formula_evidence_hash,
                    substitution_evidence_hash,
                    provenance_hash,
                    formula_context_hash: None,
                    accepted_result_hash: ArtifactHashRef {
                        class: ArtifactHashClass::Interface,
                        schema_family: "mizar-proof/kernel-accepted-evidence".into(),
                        schema_version: SchemaVersion::new(1, 0),
                        digest: accepted,
                    },
                }
            })
    } else {
        None
    };
    Ok(Some((result, decision, metadata, candidate, handoff)))
}

/// Freshly checks one authentic direct-premise source discharge candidate.
/// The returned check and classification do not authorize activation or publication.
pub fn check_source_registration_discharge_candidate(
    source: &mizar_resolve::resolved_ast::SurfaceResolvedArena,
    nodes: &mizar_checker::typed_ast::TypedArena,
    symbols: &mizar_resolve::env::SymbolEnv,
    snapshot: mizar_session::BuildSnapshotId,
    vc_id: VcId,
    policy: &VerifierPolicy,
) -> Result<(KernelCheckResult, crate::policy::PolicyDecision), String> {
    check_source_registration_discharge_payload(source, nodes, symbols, snapshot, vc_id, policy)
        .map(|(result, decision, _, _)| (result, decision))
}

/// Freshly checks a source discharge candidate and associates raw same-call metadata.
/// The metadata does not authorize activation, publication, or proof reuse.
pub fn check_source_registration_discharge_metadata(
    source: &mizar_resolve::resolved_ast::SurfaceResolvedArena,
    nodes: &mizar_checker::typed_ast::TypedArena,
    symbols: &mizar_resolve::env::SymbolEnv,
    snapshot: mizar_session::BuildSnapshotId,
    vc_id: VcId,
    policy: &VerifierPolicy,
) -> Result<
    (
        KernelCheckResult,
        crate::policy::PolicyDecision,
        Option<mizar_artifact::proof_witness::KernelAcceptanceMetadata>,
    ),
    String,
> {
    check_source_registration_discharge_payload(source, nodes, symbols, snapshot, vc_id, policy)
        .map(|(result, decision, metadata, _)| (result, decision, metadata))
}

fn check_source_registration_discharge_payload(
    source: &mizar_resolve::resolved_ast::SurfaceResolvedArena,
    nodes: &mizar_checker::typed_ast::TypedArena,
    symbols: &mizar_resolve::env::SymbolEnv,
    snapshot: mizar_session::BuildSnapshotId,
    vc_id: VcId,
    policy: &VerifierPolicy,
) -> Result<
    (
        KernelCheckResult,
        crate::policy::PolicyDecision,
        Option<mizar_artifact::proof_witness::KernelAcceptanceMetadata>,
        Vec<u8>,
    ),
    String,
> {
    use crate::policy::{PolicyCandidate, ProofPolicyEvaluator};
    use mizar_artifact::{
        proof_witness::KernelAcceptanceMetadata,
        registration_summary::{ArtifactHashClass, ArtifactHashRef},
    };
    use mizar_kernel::{
        certificate_parser::{ClauseTautologyPolicy, Fingerprint, KernelProfileRecord},
        formula_evidence::{FormulaEvidenceParseContext, parse_formula_evidence},
    };
    use mizar_vc::{
        discharge::{DischargeInput, DischargePolicy, try_discharge},
        generator::generate_source_existential_registration,
        kernel_evidence_handoff::{
            KernelClauseTautologyPolicy, build_source_existential_discharge_candidate,
        },
        vc_ir::{GenerationSchemaVersion, VcSchemaVersion},
    };
    let checked =
        mizar_checker::registration_resolution::check_source_existential_registration_proof(
            source, nodes, symbols,
        )?;
    let core = mizar_core::elaborator::lower_source_existential_registration(&checked)?;
    let original = generate_source_existential_registration(
        &core,
        snapshot,
        &GenerationSchemaVersion::new("source-existential-registration-v1"),
        &VcSchemaVersion::new("vc-v1"),
    )?;
    let discharge = try_discharge(DischargeInput {
        vc_set: &original,
        policy: &DischargePolicy::default(),
    })
    .map_err(|error| format!("registration.proof.discharge:{error:?}"))?;
    let handoff =
        build_source_existential_discharge_candidate(&core, &original, vc_id, &discharge)?;
    let bytes = handoff
        .encode_local_evidence(None)
        .map_err(|_| "registration.proof.invalid_source_candidate".to_owned())?;
    let evaluator = ProofPolicyEvaluator::new(policy.clone());
    if !evaluator.can_schedule_kernel_check(&PolicyCandidate::UncheckedBuiltinDischarge {
        has_stable_kernel_representation: !bytes.is_empty()
            && handoff.context_identity_is_consistent(),
    }) {
        return Err("registration.proof.kernel_policy_rejected".into());
    }
    let envelope = handoff.canonical_evidence();
    let profile = envelope.kernel_profile();
    let tautology = match profile.clause_tautology_policy {
        KernelClauseTautologyPolicy::Reject => ClauseTautologyPolicy::Reject,
        KernelClauseTautologyPolicy::Marker => ClauseTautologyPolicy::Marker,
        _ => return Err("registration.proof.invalid_source_candidate".into()),
    };
    let target = envelope.target_vc();
    let parse_context = FormulaEvidenceParseContext::v1(
        Fingerprint::new(target.algorithm_id, target.digest.clone()),
        KernelProfileRecord::v1(profile.profile_id, tautology),
    );
    let parsed = parse_formula_evidence(&bytes, &parse_context)
        .map_err(|error| format!("registration.proof.parse:{error:?}"))?;
    let result = check_source_kernel_handoff(
        &handoff,
        &parsed,
        "registration.proof.invalid_source_candidate",
    )?;
    let input =
        KernelPolicyInput::from_kernel_result(&result, KernelEvidenceOrigin::BuiltinDischarge);
    let accepted = input.accepted_evidence_hash();
    let polarity = input.accepted_goal_polarity();
    let decision = evaluator.evaluate_candidate(&PolicyCandidate::KernelResult(input));
    let metadata = if decision.class == crate::policy::CandidatePolicyClass::DischargedBuiltin
        && result.status() == KernelCheckStatus::Accepted
        && !result.policy_taint()
        && result.evidence_check_kind()
            == Some(mizar_kernel::checker::KernelEvidenceCheckKind::ProofObligation)
        && polarity == Some(AcceptedGoalPolarity::AssertFalseForRefutation)
    {
        accepted
            .zip(handoff.local_evidence_component_hashes().ok())
            .map(|(accepted, components)| {
                let [
                    kernel_profile_fingerprint,
                    formula_evidence_hash,
                    substitution_evidence_hash,
                    provenance_hash,
                ] = [
                    ("profile", components[0]),
                    ("formula", components[1]),
                    ("substitutions", components[2]),
                    ("provenance", components[3]),
                ]
                .map(|(kind, digest)| ArtifactHashRef {
                    class: ArtifactHashClass::Interface,
                    schema_family: format!("mizar-vc/local-evidence-component/{kind}"),
                    schema_version: SchemaVersion::new(1, 0),
                    digest,
                });
                KernelAcceptanceMetadata {
                    kernel_profile_fingerprint,
                    verifier_policy_fingerprint: ArtifactHashRef {
                        class: ArtifactHashClass::Interface,
                        schema_family: "mizar-proof/verifier-policy".into(),
                        schema_version: SchemaVersion::new(policy.schema_version(), 0),
                        digest: policy.policy_fingerprint().hash(),
                    },
                    checker_schema_version: SchemaVersion::new(policy.checker_schema_version(), 0),
                    evidence_schema_version: SchemaVersion::new(envelope.schema_version(), 0),
                    target_binding_hash: ArtifactHashRef {
                        class: ArtifactHashClass::Interface,
                        schema_family: "mizar-vc/kernel-evidence-handoff-canonical".into(),
                        schema_version: SchemaVersion::new(1, 0),
                        digest: handoff.canonical_hash(),
                    },
                    formula_evidence_hash,
                    substitution_evidence_hash,
                    provenance_hash,
                    formula_context_hash: None,
                    accepted_result_hash: ArtifactHashRef {
                        class: ArtifactHashClass::Interface,
                        schema_family: "mizar-proof/kernel-accepted-evidence".into(),
                        schema_version: SchemaVersion::new(1, 0),
                        digest: accepted,
                    },
                }
            })
    } else {
        None
    };
    Ok((result, decision, metadata, bytes))
}

fn check_source_kernel_handoff(
    handoff: &mizar_vc::kernel_evidence_handoff::VcKernelEvidenceHandoff,
    parsed: &mizar_kernel::formula_evidence::ParsedKernelEvidence,
    invalid: &str,
) -> Result<KernelCheckResult, String> {
    use mizar_kernel::{
        certificate_parser::Fingerprint,
        checker::{
            FormulaEvidenceContext, ImportedFactContextLimits, KernelCheckPolicy,
            KernelContextIdentityEntry, KernelContextIdentityPayload, KernelContextIdentitySource,
            KernelEvidenceCheckInput, KernelEvidenceCheckKind, KernelEvidenceCheckLimits,
            KernelFormulaProducerRef, KernelVcGeneratedFormulaId, check_kernel_evidence,
        },
        rejection::TargetVcFingerprint,
    };
    use mizar_vc::{
        kernel_evidence_handoff::KernelContextIdentitySource as SourceContext, vc_ir::VcFormulaRef,
    };
    let target = handoff.canonical_evidence().target_vc();
    let target = TargetVcFingerprint::new(target.algorithm_id, target.digest.clone());
    let entries = handoff
        .context_identity()
        .entries()
        .iter()
        .map(|entry| {
            let source = match entry.source() {
                SourceContext::LocalHypothesis { local_context_id } => {
                    KernelContextIdentitySource::LocalHypothesis { local_context_id }
                }
                SourceContext::CitedPremise { local_context_id } => {
                    KernelContextIdentitySource::CitedPremise { local_context_id }
                }
                SourceContext::GeneratedVcFact { vc_fact_id } => {
                    KernelContextIdentitySource::GeneratedVcFact { vc_fact_id }
                }
                _ => return Err(invalid.to_owned()),
            };
            let producer = match entry.producer_formula_ref() {
                VcFormulaRef::Core(id) => KernelFormulaProducerRef::Core(id),
                VcFormulaRef::Generated(id) => {
                    KernelFormulaProducerRef::Generated(KernelVcGeneratedFormulaId::new(id.index()))
                }
                _ => return Err(invalid.to_owned()),
            };
            Ok(KernelContextIdentityEntry::new(
                source,
                entry.formula_id(),
                Fingerprint::new(
                    entry.formula_fingerprint().algorithm_id,
                    entry.formula_fingerprint().digest.clone(),
                ),
                producer,
            ))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let requirements = handoff.formula_context_requirements();
    if requirements.is_some_and(|context| {
        !context.imported_axioms.is_empty() || !context.imported_theorems.is_empty()
    }) {
        return Err(invalid.to_owned());
    }
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
    .map_err(|error| format!("registration.proof.context:{error:?}"))?;
    let result = check_kernel_evidence(KernelEvidenceCheckInput {
        target_vc_fingerprint: &target,
        evidence: parsed,
        formula_context: Some(&context),
        check_kind: KernelEvidenceCheckKind::ProofObligation,
        policy: KernelCheckPolicy::default(),
        limits: KernelEvidenceCheckLimits::default(),
    });
    Ok(result)
}

/// Freshly selects one source-bound candidate under the supplied policy.
/// The selection provides no activation, witness or publication authority.
pub fn select_source_registration_candidate(
    source: &mizar_resolve::resolved_ast::SurfaceResolvedArena,
    nodes: &mizar_checker::typed_ast::TypedArena,
    symbols: &mizar_resolve::env::SymbolEnv,
    snapshot: mizar_session::BuildSnapshotId,
    vc_id: VcId,
    encoding: (
        mizar_atp::problem::LogicProfile,
        mizar_atp::problem::ConcreteFormat,
    ),
    policy: &VerifierPolicy,
) -> Result<crate::selection::ProofSelection, String> {
    use crate::{
        policy::CandidatePolicyClass,
        selection::{ProofEvidenceCandidate, ProofEvidenceSet, select_winner},
    };
    use mizar_vc::{
        discharge::{DischargeInput, DischargePolicy, try_discharge},
        generator::generate_source_existential_registration,
        vc_ir::{GenerationSchemaVersion, VcSchemaVersion},
    };
    let (result, decision) = check_source_registration_candidate(
        source,
        nodes,
        symbols,
        snapshot,
        vc_id,
        encoding.clone(),
        policy,
    )?;
    let checked =
        mizar_checker::registration_resolution::check_source_existential_registration_proof(
            source, nodes, symbols,
        )?;
    let core = mizar_core::elaborator::lower_source_existential_registration(&checked)?;
    let original = generate_source_existential_registration(
        &core,
        snapshot,
        &GenerationSchemaVersion::new("source-existential-registration-v1"),
        &VcSchemaVersion::new("vc-v1"),
    )?;
    let discharge = try_discharge(DischargeInput {
        vc_set: &original,
        policy: &DischargePolicy::default(),
    })
    .map_err(|error| format!("registration.proof.discharge:{error:?}"))?;
    let (encoded, actual) = mizar_atp::backend::build_source_registration_candidate_inputs(
        &core,
        discharge.vc_set(),
        vc_id,
        encoding.0,
        encoding.1,
    )?;
    let target = result.target_vc_fingerprint();
    let fingerprint = encoded.target_binding().fingerprint();
    if target.algorithm_id != fingerprint.algorithm_id() || target.digest != fingerprint.digest() {
        return Err("registration.proof.invalid_source_candidate".into());
    }
    let id = CandidateSourceId::new(actual.candidate_id()).map_err(|error| error.to_string())?;
    let candidate = if decision.class == CandidatePolicyClass::KernelVerified {
        ProofEvidenceCandidate::from_trusted_kernel_input(
            id,
            &KernelPolicyInput::from_kernel_result(
                &result,
                KernelEvidenceOrigin::AtpFormulaSubstitution,
            ),
        )
        .ok_or_else(|| "registration.proof.invalid_source_candidate".to_owned())?
    } else {
        ProofEvidenceCandidate::new(id, decision)
    }
    .with_provenance_hash(actual.provenance_hash());
    Ok(select_winner(
        &ProofEvidenceSet::new(target.sort_bytes(), encoded.input_hash(), policy.clone())
            .with_candidates([candidate]),
    ))
}

/// Proves the authenticated local existential registration and returns its local database.
pub fn prove_source_existential_registration(
    source: &mizar_resolve::resolved_ast::SurfaceResolvedArena,
    nodes: &mizar_checker::typed_ast::TypedArena,
    symbols: &mizar_resolve::env::SymbolEnv,
    snapshot: mizar_session::BuildSnapshotId,
    policy: &VerifierPolicy,
) -> Result<mizar_checker::registration_resolution::RegistrationDatabase, String> {
    use crate::policy::{CandidatePolicyClass, PolicyCandidate, ProofPolicyEvaluator};
    use mizar_checker::registration_resolution::{
        ActivationInput, RegistrationDatabase, check_source_existential_registration_proof,
    };
    use mizar_kernel::{
        certificate_parser::{ClauseTautologyPolicy, Fingerprint, KernelProfileRecord},
        formula_evidence::{FormulaEvidenceParseContext, parse_formula_evidence},
    };
    use mizar_vc::{
        generator::generate_source_existential_registration,
        kernel_evidence_handoff::{
            KernelClauseTautologyPolicy, build_source_existential_kernel_handoff,
        },
        vc_ir::{GenerationSchemaVersion, VcSchemaVersion},
    };
    let invalid = || "registration.proof.invalid_source_evidence".to_owned();
    let check = check_source_existential_registration_proof(source, nodes, symbols)?;
    let core = mizar_core::elaborator::lower_source_existential_registration(&check)?;
    let vcs = generate_source_existential_registration(
        &core,
        snapshot,
        &GenerationSchemaVersion::new("source-existential-registration-v1"),
        &VcSchemaVersion::new("vc-v1"),
    )?;
    if vcs.vcs().len() != 2 * check.validations().len() {
        return Err(invalid());
    }
    let evaluator = ProofPolicyEvaluator::new(policy.clone());
    if !evaluator.can_schedule_kernel_check(&PolicyCandidate::UncheckedBuiltinDischarge {
        has_stable_kernel_representation: true,
    }) {
        return Err("registration.proof.kernel_policy_rejected".into());
    }
    let mut hashes = Vec::new();
    for vc in vcs.vcs() {
        let handoff = build_source_existential_kernel_handoff(&core, &vcs, vc.id)?;
        let envelope = handoff.canonical_evidence();
        let profile = envelope.kernel_profile();
        let tautology = match profile.clause_tautology_policy {
            KernelClauseTautologyPolicy::Reject => ClauseTautologyPolicy::Reject,
            KernelClauseTautologyPolicy::Marker => ClauseTautologyPolicy::Marker,
            _ => return Err(invalid()),
        };
        let kernel_profile = KernelProfileRecord::v1(profile.profile_id, tautology);
        let mut equality_instances = Vec::new();
        let target = envelope.target_vc();
        let mut schema = 1u16;
        let parsed = loop {
            let bytes = handoff
                .encode_local_evidence((schema == 2).then_some(equality_instances.as_slice()))
                .map_err(|_| invalid())?;
            let fingerprint = Fingerprint::new(target.algorithm_id, target.digest.clone());
            let mut context = FormulaEvidenceParseContext::v1(fingerprint, kernel_profile.clone());
            if schema == 2 {
                context.accepted_schema_versions.insert(2);
            }
            let parsed = parse_formula_evidence(&bytes, &context)
                .map_err(|error| format!("registration.proof.parse:{error:?}"))?;
            if schema == 2 {
                break parsed;
            }
            equality_instances =
                mizar_atp::backend::encode_closed_choice_reflexivity_names(&parsed)
                    .map_err(|_| invalid())?;
            schema = 2;
        };
        let result = check_source_kernel_handoff(
            &handoff,
            &parsed,
            "registration.proof.invalid_source_evidence",
        )?;
        let input =
            KernelPolicyInput::from_kernel_result(&result, KernelEvidenceOrigin::BuiltinDischarge);
        if input.status() != KernelCheckStatus::Accepted
            || input.policy_taint()
            || !input.is_proof_obligation()
            || input.accepted_goal_polarity()
                != Some(AcceptedGoalPolarity::AssertFalseForRefutation)
            || evaluator.candidate_class(&PolicyCandidate::KernelResult(input))
                != CandidatePolicyClass::DischargedBuiltin
        {
            return Err(format!("registration.proof.kernel_rejected:{result:?}"));
        }
        hashes.push(handoff.canonical_hash());
    }
    let mut activations = Vec::new();
    for validation in check.validations() {
        let entry = symbols
            .registrations()
            .iter()
            .find(|entry| entry.id() == validation.resolver_registration())
            .ok_or_else(invalid)?;
        let pattern = format!("{:?}", validation.pattern());
        let association = format!(
            "source-existential-registration-v1;source={:?};module={:?};snapshot={snapshot:?};validation={validation:?};core={core:?};vcs={vcs:?};handoffs={hashes:?};policy={:?}",
            source.source_id(),
            source.module(),
            policy.policy_fingerprint()
        );
        let mut fingerprint = StableHasher::new("source-existential-registration-v1");
        fingerprint.field_str("association", &association);
        let fingerprint = fingerprint.finalize();
        let activation = ActivationInput::accepted(
            entry.id(),
            entry.kind(),
            pattern.clone(),
            pattern,
            validation.correctness_provenance().as_str(),
            association,
        )
        .with_validation_kind(validation.pattern().kind())
        .with_fingerprint(format!("{fingerprint:?}"));
        activations.push(activation);
    }
    let database = RegistrationDatabase::from_symbol_env_with_validation(
        symbols,
        check.validations().to_vec(),
        activations,
    );
    if database.activated().len() != check.validations().len()
        || !database.pending().is_empty()
        || !database.rejected().is_empty()
        || !database.diagnostics().is_empty()
    {
        return Err(invalid());
    }
    Ok(database)
}

/// Stable source identity for proof-reuse candidates across edits.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, std::hash::Hash)]
pub struct ObligationAnchor(String);

impl ObligationAnchor {
    /// Creates a non-empty obligation anchor.
    pub fn new(anchor: impl Into<String>) -> Result<Self, StatusProjectionError> {
        let anchor = anchor.into();
        if anchor.is_empty() {
            return Err(StatusProjectionError::EmptyObligationAnchor);
        }
        Ok(Self(anchor))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ObligationAnchor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Stable proof-obligation identity supplied by the VC/artifact producer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProofObligationIdentity {
    obligation_id: String,
    obligation_anchor: ObligationAnchor,
    obligation_fingerprint: Hash,
    vc_fingerprint: Hash,
    local_context_fingerprint: Hash,
    dependency_slice_fingerprint: Hash,
}

impl ProofObligationIdentity {
    pub fn new(
        obligation_id: impl Into<String>,
        obligation_anchor: ObligationAnchor,
        obligation_fingerprint: Hash,
        vc_fingerprint: Hash,
        local_context_fingerprint: Hash,
        dependency_slice_fingerprint: Hash,
    ) -> Result<Self, StatusProjectionError> {
        let obligation_id = obligation_id.into();
        if obligation_id.is_empty() {
            return Err(StatusProjectionError::EmptyObligationId);
        }
        Ok(Self {
            obligation_id,
            obligation_anchor,
            obligation_fingerprint,
            vc_fingerprint,
            local_context_fingerprint,
            dependency_slice_fingerprint,
        })
    }

    #[must_use]
    pub fn obligation_id(&self) -> &str {
        &self.obligation_id
    }

    #[must_use]
    pub const fn obligation_anchor(&self) -> &ObligationAnchor {
        &self.obligation_anchor
    }

    #[must_use]
    pub const fn obligation_fingerprint(&self) -> Hash {
        self.obligation_fingerprint
    }

    #[must_use]
    pub const fn vc_fingerprint(&self) -> Hash {
        self.vc_fingerprint
    }

    #[must_use]
    pub const fn local_context_fingerprint(&self) -> Hash {
        self.local_context_fingerprint
    }

    #[must_use]
    pub const fn dependency_slice_fingerprint(&self) -> Hash {
        self.dependency_slice_fingerprint
    }
}

/// Stable diagnostics-owned explanation reference.
#[derive(Clone, Copy, Debug, Eq, PartialEq, std::hash::Hash)]
pub struct ExplanationRef(Hash);

impl ExplanationRef {
    #[must_use]
    pub const fn new(hash: Hash) -> Self {
        Self(hash)
    }

    #[must_use]
    pub const fn hash(self) -> Hash {
        self.0
    }
}

/// Trusted used-axiom reference derived from an accepted proof-obligation
/// kernel result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustedUsedAxiomsRef {
    accepted_evidence_hash: Hash,
    used_axioms_hash: Hash,
    used_axiom_count: usize,
}

impl TrustedUsedAxiomsRef {
    pub fn from_kernel_result(
        result: &KernelCheckResult,
        origin: KernelEvidenceOrigin,
    ) -> Result<Self, TrustedUsedAxiomsError> {
        trusted_used_axioms_from_kernel_policy_input(
            &KernelPolicyInput::from_kernel_result(result, origin),
            result.used_axioms(),
        )
    }

    #[must_use]
    pub const fn accepted_evidence_hash(&self) -> Hash {
        self.accepted_evidence_hash
    }

    #[must_use]
    pub const fn used_axioms_hash(&self) -> Hash {
        self.used_axioms_hash
    }

    #[must_use]
    pub const fn used_axiom_count(&self) -> usize {
        self.used_axiom_count
    }

    #[cfg(test)]
    pub(crate) const fn for_test(
        accepted_evidence_hash: Hash,
        used_axioms_hash: Hash,
        used_axiom_count: usize,
    ) -> Self {
        Self {
            accepted_evidence_hash,
            used_axioms_hash,
            used_axiom_count,
        }
    }
}

/// Error while deriving trusted used-axiom metadata from kernel output.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum TrustedUsedAxiomsError {
    KernelResultNotAccepted,
    KernelResultPolicyTainted,
    MissingAcceptedEvidenceHash,
}

impl fmt::Display for TrustedUsedAxiomsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::KernelResultNotAccepted => f.write_str("kernel result is not accepted"),
            Self::KernelResultPolicyTainted => f.write_str("kernel result is policy-tainted"),
            Self::MissingAcceptedEvidenceHash => {
                f.write_str("accepted kernel evidence hash is unavailable")
            }
        }
    }
}

impl Error for TrustedUsedAxiomsError {}

fn trusted_used_axioms_from_kernel_policy_input(
    input: &KernelPolicyInput,
    used_axioms: &[UsedAxiom],
) -> Result<TrustedUsedAxiomsRef, TrustedUsedAxiomsError> {
    if input.status() != KernelCheckStatus::Accepted {
        return Err(TrustedUsedAxiomsError::KernelResultNotAccepted);
    }
    if input.policy_taint() {
        return Err(TrustedUsedAxiomsError::KernelResultPolicyTainted);
    }
    if !input.is_proof_obligation() {
        return Err(TrustedUsedAxiomsError::MissingAcceptedEvidenceHash);
    }

    let accepted_evidence_hash = input
        .accepted_evidence_hash()
        .ok_or(TrustedUsedAxiomsError::MissingAcceptedEvidenceHash)?;

    Ok(TrustedUsedAxiomsRef {
        accepted_evidence_hash,
        used_axioms_hash: hash_used_axioms(used_axioms),
        used_axiom_count: used_axioms.len(),
    })
}

/// Input to proof status projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProofStatusProjectionInput {
    selection: ArtifactProofSelection,
    policy: VerifierPolicy,
    identity: ProofObligationIdentity,
    dependency_compatibility: Option<ProofReuseDependencyCompatibility>,
    trusted_used_axioms: Option<TrustedUsedAxiomsRef>,
    explanation_ref: Option<ExplanationRef>,
}

impl ProofStatusProjectionInput {
    #[must_use]
    pub const fn new(
        selection: ArtifactProofSelection,
        policy: VerifierPolicy,
        identity: ProofObligationIdentity,
    ) -> Self {
        Self {
            selection,
            policy,
            identity,
            dependency_compatibility: None,
            trusted_used_axioms: None,
            explanation_ref: None,
        }
    }

    #[must_use]
    pub fn with_dependency_compatibility(
        mut self,
        compatibility: ProofReuseDependencyCompatibility,
    ) -> Self {
        self.dependency_compatibility = Some(compatibility);
        self
    }

    #[must_use]
    pub fn with_trusted_used_axioms(mut self, trusted: TrustedUsedAxiomsRef) -> Self {
        self.trusted_used_axioms = Some(trusted);
        self
    }

    #[must_use]
    pub const fn with_explanation_ref(mut self, explanation_ref: ExplanationRef) -> Self {
        self.explanation_ref = Some(explanation_ref);
        self
    }
}

/// Internal projected proof status before final artifact publication.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, std::hash::Hash)]
#[non_exhaustive]
pub enum ProjectedProofStatus {
    Accepted,
    ExternallyAttested,
    PolicyAssumed,
    Open,
    Rejected,
    NotRequired,
}

impl ProjectedProofStatus {
    #[must_use]
    pub const fn is_trusted(self) -> bool {
        matches!(self, Self::Accepted)
    }
}

/// Obligation status names currently representable by `mizar-artifact`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, std::hash::Hash)]
#[non_exhaustive]
pub enum CurrentArtifactObligationStatus {
    Accepted,
    Open,
    Rejected,
    ExternallyAttested,
    NotRequired,
}

/// Why a projected status cannot be published into the current artifact schema.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, std::hash::Hash)]
#[non_exhaustive]
pub enum ArtifactPublicationGap {
    MissingKernelVerifiedWitness,
    DischargedBuiltinWitnessUnsupported,
    PolicyAssumedStatusUnsupported,
}

/// Publication availability for the current artifact schema.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, std::hash::Hash)]
#[non_exhaustive]
pub enum ArtifactStatusPublication {
    Publishable(CurrentArtifactObligationStatus),
    ExternalDependencyGap(ArtifactPublicationGap),
}

impl ArtifactStatusPublication {
    #[must_use]
    pub const fn status(self) -> Option<CurrentArtifactObligationStatus> {
        match self {
            Self::Publishable(status) => Some(status),
            Self::ExternalDependencyGap(_) => None,
        }
    }

    #[must_use]
    pub const fn gap(self) -> Option<ArtifactPublicationGap> {
        match self {
            Self::Publishable(_) => None,
            Self::ExternalDependencyGap(gap) => Some(gap),
        }
    }
}

/// Dependency artifact and schema compatibility metadata for proof reuse.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProofReuseDependencyCompatibility {
    dependency_artifact_fingerprint: Hash,
    dependency_schema_version: SchemaVersion,
    proof_reuse_schema_version: SchemaVersion,
}

impl ProofReuseDependencyCompatibility {
    #[must_use]
    pub const fn new(
        dependency_artifact_fingerprint: Hash,
        dependency_schema_version: SchemaVersion,
        proof_reuse_schema_version: SchemaVersion,
    ) -> Self {
        Self {
            dependency_artifact_fingerprint,
            dependency_schema_version,
            proof_reuse_schema_version,
        }
    }

    #[must_use]
    pub const fn dependency_artifact_fingerprint(self) -> Hash {
        self.dependency_artifact_fingerprint
    }

    #[must_use]
    pub const fn dependency_schema_version(self) -> SchemaVersion {
        self.dependency_schema_version
    }

    #[must_use]
    pub const fn proof_reuse_schema_version(self) -> SchemaVersion {
        self.proof_reuse_schema_version
    }
}

/// Stable identity for the proof evidence selected for possible reuse.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProofEvidenceReuseIdentity {
    selected_candidate_id: Option<CandidateSourceId>,
    selected_candidate_provenance_hash: Option<Hash>,
    selected_evidence_hash: Option<Hash>,
    selected_proof_witness_hash: Option<Hash>,
    deterministic_discharge_hash: Option<Hash>,
    accepted_goal_polarity: Option<AcceptedGoalPolarity>,
    tie_break_key_hash: Hash,
    selection_reason: &'static str,
}

impl ProofEvidenceReuseIdentity {
    #[must_use]
    pub fn selected_candidate_id(&self) -> Option<&CandidateSourceId> {
        self.selected_candidate_id.as_ref()
    }

    #[must_use]
    pub const fn selected_candidate_provenance_hash(&self) -> Option<Hash> {
        self.selected_candidate_provenance_hash
    }

    #[must_use]
    pub const fn selected_evidence_hash(&self) -> Option<Hash> {
        self.selected_evidence_hash
    }

    #[must_use]
    pub const fn selected_proof_witness_hash(&self) -> Option<Hash> {
        self.selected_proof_witness_hash
    }

    #[must_use]
    pub const fn deterministic_discharge_hash(&self) -> Option<Hash> {
        self.deterministic_discharge_hash
    }

    #[must_use]
    pub const fn accepted_goal_polarity(&self) -> Option<AcceptedGoalPolarity> {
        self.accepted_goal_polarity
    }

    #[must_use]
    pub const fn tie_break_key_hash(&self) -> Hash {
        self.tie_break_key_hash
    }

    #[must_use]
    pub const fn selection_reason(&self) -> &'static str {
        self.selection_reason
    }
}

/// Stable proof-reuse metadata emitted by status projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StatusReuseMetadata {
    selected_class: ProofWinnerClass,
    projected_status: ProjectedProofStatus,
    selected_candidate_id: Option<CandidateSourceId>,
    obligation_anchor: ObligationAnchor,
    obligation_fingerprint: Hash,
    vc_fingerprint: Hash,
    local_context_fingerprint: Hash,
    dependency_slice_fingerprint: Hash,
    policy_fingerprint: PolicyFingerprint,
    selected_evidence_hash: Option<Hash>,
    selected_proof_witness_hash: Option<Hash>,
    deterministic_discharge_hash: Option<Hash>,
    accepted_goal_polarity: Option<AcceptedGoalPolarity>,
    trusted_used_axioms_hash: Option<Hash>,
    external_admission_status: Option<ExternalEvidencePublicationStatus>,
    selected_candidate_provenance_hash: Option<Hash>,
    selection_reason: &'static str,
    tie_break_key_hash: Hash,
    dependency_compatibility: Option<ProofReuseDependencyCompatibility>,
    explanation_ref: Option<ExplanationRef>,
    diagnostic_result_id: Option<Hash>,
    proof_reuse_validation_hash: Hash,
}

impl StatusReuseMetadata {
    #[must_use]
    pub const fn selected_class(&self) -> ProofWinnerClass {
        self.selected_class
    }

    #[must_use]
    pub const fn projected_status(&self) -> ProjectedProofStatus {
        self.projected_status
    }

    #[must_use]
    pub fn selected_candidate_id(&self) -> Option<&CandidateSourceId> {
        self.selected_candidate_id.as_ref()
    }

    #[must_use]
    pub const fn obligation_anchor(&self) -> &ObligationAnchor {
        &self.obligation_anchor
    }

    #[must_use]
    pub const fn obligation_fingerprint(&self) -> Hash {
        self.obligation_fingerprint
    }

    #[must_use]
    pub const fn vc_fingerprint(&self) -> Hash {
        self.vc_fingerprint
    }

    #[must_use]
    pub const fn local_context_fingerprint(&self) -> Hash {
        self.local_context_fingerprint
    }

    #[must_use]
    pub const fn dependency_slice_fingerprint(&self) -> Hash {
        self.dependency_slice_fingerprint
    }

    #[must_use]
    pub const fn policy_fingerprint(&self) -> PolicyFingerprint {
        self.policy_fingerprint
    }

    #[must_use]
    pub const fn selected_evidence_hash(&self) -> Option<Hash> {
        self.selected_evidence_hash
    }

    #[must_use]
    pub const fn selected_proof_witness_hash(&self) -> Option<Hash> {
        self.selected_proof_witness_hash
    }

    #[must_use]
    pub const fn deterministic_discharge_hash(&self) -> Option<Hash> {
        self.deterministic_discharge_hash
    }

    #[must_use]
    pub const fn accepted_goal_polarity(&self) -> Option<AcceptedGoalPolarity> {
        self.accepted_goal_polarity
    }

    #[must_use]
    pub const fn trusted_used_axioms_hash(&self) -> Option<Hash> {
        self.trusted_used_axioms_hash
    }

    #[must_use]
    pub const fn external_admission_status(&self) -> Option<ExternalEvidencePublicationStatus> {
        self.external_admission_status
    }

    #[must_use]
    pub const fn selected_candidate_provenance_hash(&self) -> Option<Hash> {
        self.selected_candidate_provenance_hash
    }

    #[must_use]
    pub const fn selection_reason(&self) -> &'static str {
        self.selection_reason
    }

    #[must_use]
    pub const fn tie_break_key_hash(&self) -> Hash {
        self.tie_break_key_hash
    }

    #[must_use]
    pub const fn dependency_compatibility(&self) -> Option<ProofReuseDependencyCompatibility> {
        self.dependency_compatibility
    }

    #[must_use]
    pub const fn explanation_ref(&self) -> Option<ExplanationRef> {
        self.explanation_ref
    }

    #[must_use]
    pub const fn diagnostic_result_id(&self) -> Option<Hash> {
        self.diagnostic_result_id
    }

    #[must_use]
    pub fn proof_evidence_identity(&self) -> ProofEvidenceReuseIdentity {
        ProofEvidenceReuseIdentity {
            selected_candidate_id: self.selected_candidate_id.clone(),
            selected_candidate_provenance_hash: self.selected_candidate_provenance_hash,
            selected_evidence_hash: self.selected_evidence_hash,
            selected_proof_witness_hash: self.selected_proof_witness_hash,
            deterministic_discharge_hash: self.deterministic_discharge_hash,
            accepted_goal_polarity: self.accepted_goal_polarity,
            tie_break_key_hash: self.tie_break_key_hash,
            selection_reason: self.selection_reason,
        }
    }

    #[must_use]
    pub const fn proof_reuse_validation_hash(&self) -> Hash {
        self.proof_reuse_validation_hash
    }

    #[must_use]
    pub const fn cache_reuse_predicate_complete(&self) -> bool {
        if self.dependency_compatibility.is_none() {
            return false;
        }

        match self.selected_class {
            ProofWinnerClass::KernelVerified => self.selected_proof_witness_hash.is_some(),
            ProofWinnerClass::DischargedBuiltin => self.deterministic_discharge_hash.is_some(),
            ProofWinnerClass::PolicyPermittedExternal
            | ProofWinnerClass::PolicyAssumed
            | ProofWinnerClass::PolicyOpen
            | ProofWinnerClass::Rejected
            | ProofWinnerClass::NoSelectableEvidence => false,
        }
    }
}

/// Final status projection result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProofStatusProjection {
    vc: VcId,
    obligation_id: String,
    selected_class: ProofWinnerClass,
    projected_status: ProjectedProofStatus,
    artifact_publication: ArtifactStatusPublication,
    accepted_witness_obligation_id: Option<String>,
    trusted_used_axioms: Option<TrustedUsedAxiomsRef>,
    diagnostic_refs: Vec<DiagnosticRef>,
    explanation_ref: Option<ExplanationRef>,
    reuse_metadata: StatusReuseMetadata,
}

impl ProofStatusProjection {
    #[must_use]
    pub const fn vc(&self) -> VcId {
        self.vc
    }

    #[must_use]
    pub fn obligation_id(&self) -> &str {
        &self.obligation_id
    }

    #[must_use]
    pub const fn selected_class(&self) -> ProofWinnerClass {
        self.selected_class
    }

    #[must_use]
    pub const fn projected_status(&self) -> ProjectedProofStatus {
        self.projected_status
    }

    #[must_use]
    pub const fn artifact_publication(&self) -> ArtifactStatusPublication {
        self.artifact_publication
    }

    #[must_use]
    pub fn accepted_witness_obligation_id(&self) -> Option<&str> {
        self.accepted_witness_obligation_id.as_deref()
    }

    #[must_use]
    pub const fn trusted_used_axioms(&self) -> Option<&TrustedUsedAxiomsRef> {
        self.trusted_used_axioms.as_ref()
    }

    #[must_use]
    pub fn diagnostic_refs(&self) -> &[DiagnosticRef] {
        &self.diagnostic_refs
    }

    #[must_use]
    pub const fn explanation_ref(&self) -> Option<ExplanationRef> {
        self.explanation_ref
    }

    #[must_use]
    pub const fn reuse_metadata(&self) -> &StatusReuseMetadata {
        &self.reuse_metadata
    }
}

/// Projection error.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum StatusProjectionError {
    EmptyObligationId,
    EmptyObligationAnchor,
    TrustedUsedAxiomsForNonTrustedStatus {
        selected_class: ProofWinnerClass,
    },
    TrustedUsedAxiomsUnavailable {
        selected_class: ProofWinnerClass,
    },
    TrustedUsedAxiomsEvidenceMismatch {
        selected_evidence_hash: Option<Hash>,
        accepted_evidence_hash: Hash,
    },
    PolicyFingerprintMismatch {
        selected_policy_fingerprint: PolicyFingerprint,
        active_policy_fingerprint: PolicyFingerprint,
    },
}

impl fmt::Display for StatusProjectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyObligationId => f.write_str("obligation id must be non-empty"),
            Self::EmptyObligationAnchor => f.write_str("obligation anchor must be non-empty"),
            Self::TrustedUsedAxiomsForNonTrustedStatus { selected_class } => write!(
                f,
                "trusted used_axioms cannot be attached to non-trusted class {selected_class:?}"
            ),
            Self::TrustedUsedAxiomsUnavailable { selected_class } => write!(
                f,
                "selected class {selected_class:?} did not report trusted used_axioms availability"
            ),
            Self::TrustedUsedAxiomsEvidenceMismatch {
                selected_evidence_hash,
                accepted_evidence_hash,
            } => write!(
                f,
                "trusted used_axioms evidence hash {accepted_evidence_hash:?} does not match selected evidence {selected_evidence_hash:?}"
            ),
            Self::PolicyFingerprintMismatch {
                selected_policy_fingerprint,
                active_policy_fingerprint,
            } => write!(
                f,
                "selected policy fingerprint {selected_policy_fingerprint:?} does not match active policy fingerprint {active_policy_fingerprint:?}"
            ),
        }
    }
}

impl Error for StatusProjectionError {}

/// Projects one selected proof outcome into status metadata.
pub fn project_status(
    input: ProofStatusProjectionInput,
) -> Result<ProofStatusProjection, StatusProjectionError> {
    let selected_policy_fingerprint = input
        .selection
        .selection()
        .reuse_metadata()
        .policy_fingerprint();
    let active_policy_fingerprint = input.policy.policy_fingerprint();
    if selected_policy_fingerprint != active_policy_fingerprint {
        return Err(StatusProjectionError::PolicyFingerprintMismatch {
            selected_policy_fingerprint,
            active_policy_fingerprint,
        });
    }

    let selected_class = input.selection.selected_class();
    let projected_status = projected_status(selected_class, &input.policy);
    let artifact_publication = artifact_publication(&input.selection, projected_status);
    let trusted_used_axioms =
        validated_trusted_used_axioms(&input.selection, selected_class, input.trusted_used_axioms)?;
    let accepted_witness_obligation_id = matches!(
        artifact_publication,
        ArtifactStatusPublication::Publishable(CurrentArtifactObligationStatus::Accepted)
    )
    .then(|| input.identity.obligation_id.clone());
    let selection_metadata = input.selection.selection().reuse_metadata();
    let trusted_used_axioms_hash = trusted_used_axioms
        .as_ref()
        .map(TrustedUsedAxiomsRef::used_axioms_hash);

    let mut reuse_metadata = StatusReuseMetadata {
        selected_class,
        projected_status,
        selected_candidate_id: input.selection.selection().selected_candidate_id().cloned(),
        obligation_anchor: input.identity.obligation_anchor.clone(),
        obligation_fingerprint: input.identity.obligation_fingerprint,
        vc_fingerprint: input.identity.vc_fingerprint,
        local_context_fingerprint: input.identity.local_context_fingerprint,
        dependency_slice_fingerprint: input.identity.dependency_slice_fingerprint,
        policy_fingerprint: input.policy.policy_fingerprint(),
        selected_evidence_hash: selection_metadata.selected_evidence_hash(),
        selected_proof_witness_hash: selection_metadata.selected_proof_witness_hash(),
        deterministic_discharge_hash: selection_metadata.deterministic_discharge_hash(),
        accepted_goal_polarity: selection_metadata.accepted_goal_polarity(),
        trusted_used_axioms_hash,
        external_admission_status: selection_metadata.external_admission_status(),
        selected_candidate_provenance_hash: selection_metadata.selected_candidate_provenance_hash(),
        selection_reason: selection_metadata.selection_reason(),
        tie_break_key_hash: selection_metadata.tie_break_key_hash(),
        dependency_compatibility: input.dependency_compatibility,
        explanation_ref: input.explanation_ref,
        diagnostic_result_id: input.selection.selection().diagnostic_result_id(),
        proof_reuse_validation_hash: Hash::from_bytes([0; Hash::BYTE_LEN]),
    };
    reuse_metadata.proof_reuse_validation_hash = hash_status_reuse_metadata(&reuse_metadata);

    Ok(ProofStatusProjection {
        vc: input.selection.vc(),
        obligation_id: input.identity.obligation_id,
        selected_class,
        projected_status,
        artifact_publication,
        accepted_witness_obligation_id,
        trusted_used_axioms,
        diagnostic_refs: input
            .selection
            .selection()
            .ordered_diagnostic_refs()
            .to_vec(),
        explanation_ref: input.explanation_ref,
        reuse_metadata,
    })
}

fn projected_status(
    selected_class: ProofWinnerClass,
    policy: &VerifierPolicy,
) -> ProjectedProofStatus {
    match selected_class {
        ProofWinnerClass::KernelVerified | ProofWinnerClass::DischargedBuiltin => {
            ProjectedProofStatus::Accepted
        }
        ProofWinnerClass::PolicyPermittedExternal => ProjectedProofStatus::ExternallyAttested,
        ProofWinnerClass::PolicyAssumed => ProjectedProofStatus::PolicyAssumed,
        ProofWinnerClass::PolicyOpen => ProjectedProofStatus::Open,
        ProofWinnerClass::Rejected => ProjectedProofStatus::Rejected,
        ProofWinnerClass::NoSelectableEvidence => match policy.open_obligation() {
            OpenObligationMode::AllowPolicyOpen => ProjectedProofStatus::Open,
            OpenObligationMode::Reject | OpenObligationMode::RecordDiagnostic => {
                ProjectedProofStatus::Rejected
            }
        },
    }
}

fn artifact_publication(
    selection: &ArtifactProofSelection,
    projected_status: ProjectedProofStatus,
) -> ArtifactStatusPublication {
    match selection.selected_class() {
        ProofWinnerClass::KernelVerified => {
            let metadata = selection.selection().reuse_metadata();
            if metadata.proof_witness_publication() == ProofWitnessPublication::Available
                && metadata.selected_proof_witness_hash().is_some()
            {
                ArtifactStatusPublication::Publishable(CurrentArtifactObligationStatus::Accepted)
            } else {
                ArtifactStatusPublication::ExternalDependencyGap(
                    ArtifactPublicationGap::MissingKernelVerifiedWitness,
                )
            }
        }
        ProofWinnerClass::DischargedBuiltin => ArtifactStatusPublication::ExternalDependencyGap(
            ArtifactPublicationGap::DischargedBuiltinWitnessUnsupported,
        ),
        ProofWinnerClass::PolicyAssumed => ArtifactStatusPublication::ExternalDependencyGap(
            ArtifactPublicationGap::PolicyAssumedStatusUnsupported,
        ),
        ProofWinnerClass::PolicyPermittedExternal => ArtifactStatusPublication::Publishable(
            CurrentArtifactObligationStatus::ExternallyAttested,
        ),
        ProofWinnerClass::PolicyOpen => {
            ArtifactStatusPublication::Publishable(CurrentArtifactObligationStatus::Open)
        }
        ProofWinnerClass::Rejected => {
            ArtifactStatusPublication::Publishable(CurrentArtifactObligationStatus::Rejected)
        }
        ProofWinnerClass::NoSelectableEvidence => match projected_status {
            ProjectedProofStatus::Open => {
                ArtifactStatusPublication::Publishable(CurrentArtifactObligationStatus::Open)
            }
            ProjectedProofStatus::Rejected => {
                ArtifactStatusPublication::Publishable(CurrentArtifactObligationStatus::Rejected)
            }
            ProjectedProofStatus::Accepted
            | ProjectedProofStatus::ExternallyAttested
            | ProjectedProofStatus::PolicyAssumed
            | ProjectedProofStatus::NotRequired => {
                unreachable!("no-selectable evidence projects only to open or rejected statuses")
            }
        },
    }
}

fn validated_trusted_used_axioms(
    selection: &ArtifactProofSelection,
    selected_class: ProofWinnerClass,
    trusted_used_axioms: Option<TrustedUsedAxiomsRef>,
) -> Result<Option<TrustedUsedAxiomsRef>, StatusProjectionError> {
    let Some(trusted_used_axioms) = trusted_used_axioms else {
        return Ok(None);
    };

    if !selected_class.is_trusted() {
        return Err(StatusProjectionError::TrustedUsedAxiomsForNonTrustedStatus { selected_class });
    }
    if !selection.selection().trusted_used_axioms_available() {
        return Err(StatusProjectionError::TrustedUsedAxiomsUnavailable { selected_class });
    }

    let selected_evidence_hash = selection
        .selection()
        .reuse_metadata()
        .selected_evidence_hash();
    if selected_evidence_hash != Some(trusted_used_axioms.accepted_evidence_hash()) {
        return Err(StatusProjectionError::TrustedUsedAxiomsEvidenceMismatch {
            selected_evidence_hash,
            accepted_evidence_hash: trusted_used_axioms.accepted_evidence_hash(),
        });
    }

    Ok(Some(trusted_used_axioms))
}

fn hash_status_reuse_metadata(metadata: &StatusReuseMetadata) -> Hash {
    let mut hash = StableHasher::new(PROOF_REUSE_VALIDATION_HASH_DOMAIN);
    hash.field_str(
        "selected_class",
        proof_winner_class_key(metadata.selected_class),
    );
    hash.field_str(
        "projected_status",
        projected_proof_status_key(metadata.projected_status),
    );
    hash.field_optional_str(
        "selected_candidate_id",
        metadata
            .selected_candidate_id
            .as_ref()
            .map(CandidateSourceId::as_str),
    );
    hash.field_str("obligation_anchor", metadata.obligation_anchor.as_str());
    hash.field_hash("obligation_fingerprint", metadata.obligation_fingerprint);
    hash.field_hash("vc_fingerprint", metadata.vc_fingerprint);
    hash.field_hash(
        "local_context_fingerprint",
        metadata.local_context_fingerprint,
    );
    hash.field_hash(
        "dependency_slice_fingerprint",
        metadata.dependency_slice_fingerprint,
    );
    hash.field_hash("policy_fingerprint", metadata.policy_fingerprint.hash());
    hash.field_optional_hash("selected_evidence_hash", metadata.selected_evidence_hash);
    hash.field_optional_hash(
        "selected_proof_witness_hash",
        metadata.selected_proof_witness_hash,
    );
    hash.field_optional_hash(
        "deterministic_discharge_hash",
        metadata.deterministic_discharge_hash,
    );
    hash.field_optional_str(
        "accepted_goal_polarity",
        metadata
            .accepted_goal_polarity
            .map(AcceptedGoalPolarity::as_str),
    );
    hash.field_optional_hash(
        "trusted_used_axioms_hash",
        metadata.trusted_used_axioms_hash,
    );
    hash.field_optional_str(
        "external_admission_status",
        metadata
            .external_admission_status
            .map(external_publication_status_key),
    );
    hash.field_optional_hash(
        "selected_candidate_provenance_hash",
        metadata.selected_candidate_provenance_hash,
    );
    hash.field_str("selection_reason", metadata.selection_reason);
    hash.field_hash("tie_break_key_hash", metadata.tie_break_key_hash);
    hash_dependency_compatibility(&mut hash, metadata.dependency_compatibility);
    hash.field_optional_hash(
        "explanation_ref",
        metadata.explanation_ref.map(ExplanationRef::hash),
    );
    hash.field_optional_hash("diagnostic_result_id", metadata.diagnostic_result_id);
    hash.finalize()
}

fn hash_dependency_compatibility(
    hash: &mut StableHasher,
    compatibility: Option<ProofReuseDependencyCompatibility>,
) {
    hash.field_bool("dependency_compatibility_present", compatibility.is_some());
    if let Some(compatibility) = compatibility {
        hash.field_hash(
            "dependency_artifact_fingerprint",
            compatibility.dependency_artifact_fingerprint(),
        );
        hash_schema_version(
            hash,
            "dependency_schema_version",
            compatibility.dependency_schema_version(),
        );
        hash_schema_version(
            hash,
            "proof_reuse_schema_version",
            compatibility.proof_reuse_schema_version(),
        );
    }
}

fn hash_schema_version(hash: &mut StableHasher, label: &str, version: SchemaVersion) {
    hash.field_u16(&format!("{label}_major"), version.major());
    hash.field_u16(&format!("{label}_minor"), version.minor());
}

fn proof_winner_class_key(class: ProofWinnerClass) -> &'static str {
    match class {
        ProofWinnerClass::KernelVerified => "kernel_verified",
        ProofWinnerClass::DischargedBuiltin => "discharged_builtin",
        ProofWinnerClass::PolicyPermittedExternal => "policy_permitted_external",
        ProofWinnerClass::PolicyAssumed => "policy_assumed",
        ProofWinnerClass::PolicyOpen => "policy_open",
        ProofWinnerClass::Rejected => "rejected",
        ProofWinnerClass::NoSelectableEvidence => "no_selectable_evidence",
    }
}

fn projected_proof_status_key(status: ProjectedProofStatus) -> &'static str {
    match status {
        ProjectedProofStatus::Accepted => "accepted",
        ProjectedProofStatus::ExternallyAttested => "externally_attested",
        ProjectedProofStatus::PolicyAssumed => "policy_assumed",
        ProjectedProofStatus::Open => "open",
        ProjectedProofStatus::Rejected => "rejected",
        ProjectedProofStatus::NotRequired => "not_required",
    }
}

fn external_publication_status_key(status: ExternalEvidencePublicationStatus) -> &'static str {
    match status {
        ExternalEvidencePublicationStatus::RejectedByPolicy => "rejected_by_policy",
        ExternalEvidencePublicationStatus::ExternallyAttestedDevelopment => {
            "externally_attested_development"
        }
        ExternalEvidencePublicationStatus::ExternallyAttestedOpenDiagnostic => {
            "externally_attested_open_diagnostic"
        }
        ExternalEvidencePublicationStatus::ExternallyAttestedPolicyPermitted => {
            "externally_attested_policy_permitted"
        }
    }
}

fn hash_used_axioms(used_axioms: &[UsedAxiom]) -> Hash {
    let mut hash = StableHasher::new(USED_AXIOMS_HASH_DOMAIN);
    hash.field_u64("used_axiom_count", used_axioms.len() as u64);
    for axiom in used_axioms {
        hash.field_str("namespace", imported_fact_namespace_key(axiom.namespace));
        hash.field_u32("imported_fact_id", axiom.imported_fact_id);
        hash.field_u8(
            "statement_fingerprint_algorithm",
            axiom.statement_fingerprint.algorithm_id,
        );
        hash.field_bytes(
            "statement_fingerprint_digest",
            &axiom.statement_fingerprint.digest,
        );
    }
    hash.finalize()
}

fn imported_fact_namespace_key(
    namespace: mizar_kernel::checker::ImportedFactNamespace,
) -> &'static str {
    match namespace {
        mizar_kernel::checker::ImportedFactNamespace::ImportedAxiom => "imported_axiom",
        mizar_kernel::checker::ImportedFactNamespace::ImportedTheorem => "imported_theorem",
        _ => "unknown",
    }
}

struct StableHasher {
    lanes: [u64; 4],
    length: u64,
}

impl StableHasher {
    fn new(domain: &str) -> Self {
        let mut hasher = Self {
            lanes: [
                0x6d_69_7a_61_72_2d_70_72,
                0x6f_6f_66_2d_73_74_61_74,
                0x75_73_2d_61_78_69_6f,
                0x6d_73_2d_68_61_73_68,
            ],
            length: 0,
        };
        hasher.field_str("domain", domain);
        hasher
    }

    fn field_str(&mut self, label: &str, value: &str) {
        self.field_bytes(label, value.as_bytes());
    }

    fn field_bool(&mut self, label: &str, value: bool) {
        self.field_bytes(label, &[u8::from(value)]);
    }

    fn field_u8(&mut self, label: &str, value: u8) {
        self.field_bytes(label, &[value]);
    }

    fn field_u16(&mut self, label: &str, value: u16) {
        self.field_bytes(label, &value.to_le_bytes());
    }

    fn field_u32(&mut self, label: &str, value: u32) {
        self.field_bytes(label, &value.to_le_bytes());
    }

    fn field_u64(&mut self, label: &str, value: u64) {
        self.field_bytes(label, &value.to_le_bytes());
    }

    fn field_hash(&mut self, label: &str, value: Hash) {
        self.field_bytes(label, value.as_bytes());
    }

    fn field_optional_hash(&mut self, label: &str, value: Option<Hash>) {
        match value {
            Some(value) => {
                self.field_u8(&format!("{label}_tag"), 0);
                self.field_hash(label, value);
            }
            None => self.field_u8(&format!("{label}_tag"), 1),
        }
    }

    fn field_optional_str(&mut self, label: &str, value: Option<&str>) {
        match value {
            Some(value) => {
                self.field_u8(&format!("{label}_tag"), 0);
                self.field_str(label, value);
            }
            None => self.field_u8(&format!("{label}_tag"), 1),
        }
    }

    fn field_bytes(&mut self, label: &str, value: &[u8]) {
        self.feed_bytes(&(label.len() as u64).to_le_bytes());
        self.feed_bytes(label.as_bytes());
        self.feed_bytes(&(value.len() as u64).to_le_bytes());
        self.feed_bytes(value);
    }

    fn feed_bytes(&mut self, bytes: &[u8]) {
        for byte in bytes {
            let lane = self.length as usize % self.lanes.len();
            let mixed = self.length.rotate_left((lane as u32) + 7);
            self.lanes[lane] ^= u64::from(*byte)
                .wrapping_add(0x9e37_79b9_7f4a_7c15)
                .wrapping_add(mixed);
            self.lanes[lane] = self.lanes[lane]
                .rotate_left(13 + lane as u32)
                .wrapping_mul(0x1000_0000_01b3);
            self.length = self.length.wrapping_add(1);
        }
    }

    fn finalize(mut self) -> Hash {
        self.lanes[0] ^= self.length;
        self.lanes[1] ^= self.length.rotate_left(17);
        self.lanes[2] ^= self.lanes[0].rotate_left(9);
        self.lanes[3] ^= self.lanes[1].rotate_left(13);

        let mut bytes = [0_u8; Hash::BYTE_LEN];
        for (chunk, lane) in bytes.as_chunks_mut::<8>().0.iter_mut().zip(self.lanes) {
            chunk.copy_from_slice(&lane.to_be_bytes());
        }
        Hash::from_bytes(bytes)
    }
}

/// Selects a freshly checked source discharge candidate without an ATP encoding.
pub fn select_source_registration_discharge_candidate(
    source: &mizar_resolve::resolved_ast::SurfaceResolvedArena,
    nodes: &mizar_checker::typed_ast::TypedArena,
    symbols: &mizar_resolve::env::SymbolEnv,
    snapshot: mizar_session::BuildSnapshotId,
    vc_id: VcId,
    policy: &VerifierPolicy,
) -> Result<crate::selection::ProofSelection, String> {
    use crate::selection::{ProofEvidenceCandidate, ProofEvidenceSet, select_winner};
    let invalid = || "registration.proof.invalid_source_candidate".to_owned();
    let (result, decision, metadata) = check_source_registration_discharge_metadata(
        source, nodes, symbols, snapshot, vc_id, policy,
    )?;
    let target = result.target_vc_fingerprint().sort_bytes();
    let mut id = String::from("source-registration-discharge-v1:");
    for byte in &target {
        use std::fmt::Write;
        write!(id, "{byte:02x}").map_err(|_| invalid())?;
    }
    let id = CandidateSourceId::new(id).map_err(|_| invalid())?;
    let candidate = if decision.class == crate::policy::CandidatePolicyClass::DischargedBuiltin {
        let metadata = metadata.ok_or_else(invalid)?;
        ProofEvidenceCandidate::from_trusted_kernel_input(
            id,
            &KernelPolicyInput::from_kernel_result(&result, KernelEvidenceOrigin::BuiltinDischarge),
        )
        .ok_or_else(invalid)?
        .with_provenance_hash(metadata.provenance_hash.digest)
    } else {
        ProofEvidenceCandidate::new(id, decision)
    };
    Ok(select_winner(
        &ProofEvidenceSet::new(target, None, policy.clone()).with_candidates([candidate]),
    ))
}

/// Projects fresh source discharge selection, an unpublished reference and checked bytes.
pub fn project_source_registration_discharge_witness(
    source: &mizar_resolve::resolved_ast::SurfaceResolvedArena,
    nodes: &mizar_checker::typed_ast::TypedArena,
    symbols: &mizar_resolve::env::SymbolEnv,
    snapshot: mizar_session::BuildSnapshotId,
    vc_id: VcId,
    policy: &VerifierPolicy,
) -> Result<
    (
        crate::selection::ProofSelection,
        mizar_artifact::proof_witness::ProofWitnessRef,
        Vec<u8>,
    ),
    String,
> {
    use crate::selection::{ProofEvidenceCandidate, ProofEvidenceSet, select_winner};
    use mizar_artifact::{
        proof_witness::{
            EvidenceKind, ProofStatus, ProofWitnessRef, builtin_proof_witness_ref_json,
        },
        registration_summary::{ArtifactHashClass, ArtifactHashRef},
        store::{CanonicalHashDomain, CanonicalJson, HashClass},
    };
    let invalid = || "registration.proof.invalid_source_candidate".to_owned();
    let (result, decision, metadata, bytes) = check_source_registration_discharge_payload(
        source, nodes, symbols, snapshot, vc_id, policy,
    )?;
    if decision.class != crate::policy::CandidatePolicyClass::DischargedBuiltin {
        return Err(invalid());
    }
    let metadata = metadata.ok_or_else(invalid)?;
    let input =
        KernelPolicyInput::from_kernel_result(&result, KernelEvidenceOrigin::BuiltinDischarge);
    let polarity = input.accepted_goal_polarity().ok_or_else(invalid)?;
    let target = result.target_vc_fingerprint().sort_bytes();
    let target_hex = source_discharge_hex(&target);
    let id = CandidateSourceId::new(format!("source-registration-discharge-v1:{target_hex}"))
        .map_err(|_| invalid())?;
    let candidate = ProofEvidenceCandidate::from_trusted_kernel_input(id.clone(), &input)
        .ok_or_else(invalid)?
        .with_provenance_hash(metadata.provenance_hash.digest);
    let selection = select_winner(
        &ProofEvidenceSet::new(target, None, policy.clone()).with_candidates([candidate]),
    );
    let payload_family = "mizar-proof/source-registration-discharge-witness-payload";
    let witness_artifact_hash = ArtifactHashRef {
        class: ArtifactHashClass::Artifact,
        schema_family: payload_family.into(),
        schema_version: SchemaVersion::new(1, 0),
        digest: CanonicalHashDomain::new(
            HashClass::Artifact,
            payload_family,
            SchemaVersion::new(1, 0),
        )
        .hash(&CanonicalJson::string(source_discharge_hex(&bytes)), &[]),
    };
    let obligation = CanonicalJson::object([
        ("original_target_bytes", CanonicalJson::string(target_hex)),
        (
            "kernel_profile_fingerprint",
            CanonicalJson::string(
                metadata
                    .kernel_profile_fingerprint
                    .to_artifact_hash_string(),
            ),
        ),
        (
            "verifier_policy_fingerprint",
            CanonicalJson::string(
                metadata
                    .verifier_policy_fingerprint
                    .to_artifact_hash_string(),
            ),
        ),
        (
            "checker_schema_version",
            CanonicalJson::string(metadata.checker_schema_version.to_string()),
        ),
        (
            "evidence_schema_version",
            CanonicalJson::string(metadata.evidence_schema_version.to_string()),
        ),
        (
            "target_binding_hash",
            CanonicalJson::string(metadata.target_binding_hash.to_artifact_hash_string()),
        ),
        (
            "formula_evidence_hash",
            CanonicalJson::string(metadata.formula_evidence_hash.to_artifact_hash_string()),
        ),
        (
            "substitution_evidence_hash",
            CanonicalJson::string(
                metadata
                    .substitution_evidence_hash
                    .to_artifact_hash_string(),
            ),
        ),
        (
            "provenance_hash",
            CanonicalJson::string(metadata.provenance_hash.to_artifact_hash_string()),
        ),
        (
            "formula_context_hash",
            metadata
                .formula_context_hash
                .as_ref()
                .map_or_else(CanonicalJson::null, |hash| {
                    CanonicalJson::string(hash.to_artifact_hash_string())
                }),
        ),
        (
            "accepted_result_hash",
            CanonicalJson::string(metadata.accepted_result_hash.to_artifact_hash_string()),
        ),
        (
            "accepted_goal_polarity",
            CanonicalJson::string(polarity.as_str()),
        ),
        (
            "witness_artifact_hash",
            CanonicalJson::string(witness_artifact_hash.to_artifact_hash_string()),
        ),
    ])
    .map_err(|_| invalid())?;
    let obligation_family = "mizar-proof/source-registration-discharge-obligation";
    let reference = ProofWitnessRef {
        schema_version: SchemaVersion::new(3, 0),
        obligation_id: id.as_str().into(),
        obligation_fingerprint: ArtifactHashRef {
            class: ArtifactHashClass::Interface,
            schema_family: obligation_family.into(),
            schema_version: SchemaVersion::new(1, 0),
            digest: CanonicalHashDomain::new(
                HashClass::Interface,
                obligation_family,
                SchemaVersion::new(1, 0),
            )
            .hash(&obligation, &[]),
        },
        proof_status: ProofStatus::DischargedBuiltin,
        evidence_kind: EvidenceKind::FormulaSubstitutionKernelEvidence,
        witness_path: format!(
            "proof-witnesses/source-registration-discharge-v1/{}.bin",
            source_discharge_hex(witness_artifact_hash.digest.as_bytes())
        ),
        witness_artifact_hash,
        kernel_acceptance: metadata,
    };
    builtin_proof_witness_ref_json(&reference).map_err(|_| invalid())?;
    Ok((selection, reference, bytes))
}

/// Projects fresh source ATP selection, an unpublished reference and checked bytes.
pub fn project_source_registration_candidate_witness(
    source: &mizar_resolve::resolved_ast::SurfaceResolvedArena,
    nodes: &mizar_checker::typed_ast::TypedArena,
    symbols: &mizar_resolve::env::SymbolEnv,
    snapshot: mizar_session::BuildSnapshotId,
    vc_id: VcId,
    encoding: (
        mizar_atp::problem::LogicProfile,
        mizar_atp::problem::ConcreteFormat,
    ),
    policy: &VerifierPolicy,
) -> Result<
    (
        crate::selection::ProofSelection,
        mizar_artifact::proof_witness::ProofWitnessRef,
        Vec<u8>,
    ),
    String,
> {
    use crate::selection::{ProofEvidenceCandidate, ProofEvidenceSet, select_winner};
    use mizar_artifact::{
        proof_witness::{EvidenceKind, ProofStatus, ProofWitnessRef, proof_witness_ref_json},
        registration_summary::{ArtifactHashClass, ArtifactHashRef},
        store::{CanonicalHashDomain, CanonicalJson, HashClass},
    };
    use mizar_atp::backend::BackendCandidatePayload;
    let invalid = || "registration.proof.invalid_source_candidate".to_owned();
    let (result, decision, metadata, actual, handoff) =
        check_source_registration_candidate_payload(
            source,
            nodes,
            symbols,
            snapshot,
            vc_id,
            (encoding.0, encoding.1, false),
            policy,
        )?
        .ok_or_else(invalid)?;
    if decision.class != crate::policy::CandidatePolicyClass::KernelVerified {
        return Err(invalid());
    }
    let metadata = metadata.ok_or_else(invalid)?;
    let input = KernelPolicyInput::from_kernel_result(
        &result,
        KernelEvidenceOrigin::AtpFormulaSubstitution,
    );
    let polarity = input.accepted_goal_polarity().ok_or_else(invalid)?;
    let target = result.target_vc_fingerprint();
    let fingerprint = actual.target_binding().fingerprint();
    if target.algorithm_id != fingerprint.algorithm_id() || target.digest != fingerprint.digest() {
        return Err(invalid());
    }
    let id = CandidateSourceId::new(actual.candidate_id()).map_err(|_| invalid())?;
    let candidate = ProofEvidenceCandidate::from_trusted_kernel_input(id.clone(), &input)
        .ok_or_else(invalid)?
        .with_provenance_hash(actual.provenance_hash());
    let target = target.sort_bytes();
    let selection = select_winner(
        &ProofEvidenceSet::new(
            target.clone(),
            actual.encoded_problem_hash(),
            policy.clone(),
        )
        .with_candidates([candidate]),
    );
    if selection.selected_class() != ProofWinnerClass::KernelVerified {
        return Err(invalid());
    }
    let BackendCandidatePayload::FormulaSubstitutionBytes(bytes) = actual.payload() else {
        return Err(invalid());
    };
    let bytes = bytes.clone();
    let payload_family = "mizar-proof/source-registration-candidate-witness-payload";
    let witness_artifact_hash = ArtifactHashRef {
        class: ArtifactHashClass::Artifact,
        schema_family: payload_family.into(),
        schema_version: SchemaVersion::new(1, 0),
        digest: CanonicalHashDomain::new(
            HashClass::Artifact,
            payload_family,
            SchemaVersion::new(1, 0),
        )
        .hash(&CanonicalJson::string(source_discharge_hex(&bytes)), &[]),
    };
    let obligation = CanonicalJson::object([
        (
            "checked_target_bytes",
            CanonicalJson::string(source_discharge_hex(&target)),
        ),
        (
            "local_context_identity_hash",
            CanonicalJson::string(source_discharge_hex(
                handoff.context_identity_hash().as_bytes(),
            )),
        ),
        ("candidate_id", CanonicalJson::string(actual.candidate_id())),
        (
            "encoded_problem_hash",
            CanonicalJson::string(source_discharge_hex(
                actual.encoded_problem_hash().as_bytes(),
            )),
        ),
        (
            "atp_provenance_hash",
            CanonicalJson::string(source_discharge_hex(actual.provenance_hash().as_bytes())),
        ),
        (
            "kernel_profile_fingerprint",
            CanonicalJson::string(
                metadata
                    .kernel_profile_fingerprint
                    .to_artifact_hash_string(),
            ),
        ),
        (
            "verifier_policy_fingerprint",
            CanonicalJson::string(
                metadata
                    .verifier_policy_fingerprint
                    .to_artifact_hash_string(),
            ),
        ),
        (
            "checker_schema_version",
            CanonicalJson::string(metadata.checker_schema_version.to_string()),
        ),
        (
            "evidence_schema_version",
            CanonicalJson::string(metadata.evidence_schema_version.to_string()),
        ),
        (
            "target_binding_hash",
            CanonicalJson::string(metadata.target_binding_hash.to_artifact_hash_string()),
        ),
        (
            "formula_evidence_hash",
            CanonicalJson::string(metadata.formula_evidence_hash.to_artifact_hash_string()),
        ),
        (
            "substitution_evidence_hash",
            CanonicalJson::string(
                metadata
                    .substitution_evidence_hash
                    .to_artifact_hash_string(),
            ),
        ),
        (
            "provenance_hash",
            CanonicalJson::string(metadata.provenance_hash.to_artifact_hash_string()),
        ),
        (
            "formula_context_hash",
            metadata
                .formula_context_hash
                .as_ref()
                .map_or_else(CanonicalJson::null, |hash| {
                    CanonicalJson::string(hash.to_artifact_hash_string())
                }),
        ),
        (
            "accepted_result_hash",
            CanonicalJson::string(metadata.accepted_result_hash.to_artifact_hash_string()),
        ),
        (
            "accepted_goal_polarity",
            CanonicalJson::string(polarity.as_str()),
        ),
        (
            "witness_artifact_hash",
            CanonicalJson::string(witness_artifact_hash.to_artifact_hash_string()),
        ),
    ])
    .map_err(|_| invalid())?;
    let obligation_family = "mizar-proof/source-registration-candidate-obligation";
    let reference = ProofWitnessRef {
        schema_version: SchemaVersion::new(2, 0),
        obligation_id: id.as_str().into(),
        obligation_fingerprint: ArtifactHashRef {
            class: ArtifactHashClass::Interface,
            schema_family: obligation_family.into(),
            schema_version: SchemaVersion::new(1, 0),
            digest: CanonicalHashDomain::new(
                HashClass::Interface,
                obligation_family,
                SchemaVersion::new(1, 0),
            )
            .hash(&obligation, &[]),
        },
        proof_status: ProofStatus::KernelVerified,
        evidence_kind: EvidenceKind::FormulaSubstitutionKernelEvidence,
        witness_path: format!(
            "proof-witnesses/source-registration-candidate-v1/{}.bin",
            source_discharge_hex(witness_artifact_hash.digest.as_bytes())
        ),
        witness_artifact_hash,
        kernel_acceptance: metadata,
    };
    proof_witness_ref_json(&reference).map_err(|_| invalid())?;
    Ok((selection, reference, bytes))
}

fn source_discharge_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests;
