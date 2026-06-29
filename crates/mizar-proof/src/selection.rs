//! Deterministic proof-evidence winner selection.
//!
//! This module orders already-classified proof evidence. It does not run ATP
//! backends, schedule kernel checks, query caches, stage witnesses, or turn
//! policy evidence into trusted acceptance.

use std::{collections::BTreeMap, error::Error, fmt};

use mizar_kernel::checker::KernelCheckStatus;
use mizar_session::Hash;
use mizar_vc::vc_ir::VcId;

use crate::policy::{
    CandidatePolicyClass, ExternalEvidenceMode, ExternalEvidencePublicationStatus,
    KernelEvidenceOrigin, KernelPolicyInput, OpenObligationMode, PolicyDecision, PolicyDiagnostic,
    PolicyFingerprint, PolicyReasonCode, VerifierPolicy,
};

const SELECTION_TIE_BREAK_HASH_DOMAIN: &str = "mizar-proof-selection-tie-break-v1";
const SELECTION_DIAGNOSTIC_HASH_DOMAIN: &str = "mizar-proof-selection-diagnostic-v1";
const SELECTION_PAYLOAD_HASH_DOMAIN: &str = "mizar-proof-selection-candidate-payload-v1";
const MISSING_PRIORITY_SENTINEL: u32 = u32::MAX;

/// Required stable producer-owned candidate source id.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, std::hash::Hash)]
pub struct CandidateSourceId(String);

impl CandidateSourceId {
    /// Creates a non-empty stable source id.
    ///
    /// This id is part of the final tie-break key. It must be stable across
    /// worker scheduling and backend completion order.
    pub fn new(id: impl Into<String>) -> Result<Self, SelectionInputError> {
        let id = id.into();
        if id.is_empty() {
            return Err(SelectionInputError::EmptyCandidateSourceId);
        }
        Ok(Self(id))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CandidateSourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Stable diagnostic reference hash supplied by diagnostics/status owners.
#[derive(Clone, Copy, Debug, Eq, PartialEq, std::hash::Hash)]
pub struct DiagnosticRef(Hash);

impl DiagnosticRef {
    #[must_use]
    pub const fn new(hash: Hash) -> Self {
        Self(hash)
    }

    #[must_use]
    pub const fn hash(self) -> Hash {
        self.0
    }
}

impl Ord for DiagnosticRef {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.as_bytes().cmp(other.0.as_bytes())
    }
}

impl PartialOrd for DiagnosticRef {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Selection input error.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum SelectionInputError {
    EmptyCandidateSourceId,
}

impl fmt::Display for SelectionInputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyCandidateSourceId => f.write_str("candidate source id must be non-empty"),
        }
    }
}

impl Error for SelectionInputError {}

/// Kernel-derived trusted evidence marker.
///
/// The only public constructor takes `KernelPolicyInput`, which public callers
/// can create only from a real `KernelCheckResult` plus explicit origin.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustedKernelEvidence {
    selected_class: ProofWinnerClass,
    accepted_evidence_hash: Hash,
}

impl TrustedKernelEvidence {
    #[must_use]
    pub fn from_policy_input(input: &KernelPolicyInput) -> Option<Self> {
        if input.status() != KernelCheckStatus::Accepted || input.policy_taint() {
            return None;
        }
        let accepted_evidence_hash = input.accepted_evidence_hash()?;

        let selected_class = match input.origin() {
            KernelEvidenceOrigin::AtpFormulaSubstitution => ProofWinnerClass::KernelVerified,
            KernelEvidenceOrigin::BuiltinDischarge | KernelEvidenceOrigin::KernelPrimitive => {
                ProofWinnerClass::DischargedBuiltin
            }
        };
        Some(Self {
            selected_class,
            accepted_evidence_hash,
        })
    }

    #[must_use]
    pub const fn selected_class(&self) -> ProofWinnerClass {
        self.selected_class
    }

    #[must_use]
    pub const fn accepted_evidence_hash(&self) -> Hash {
        self.accepted_evidence_hash
    }

    #[must_use]
    pub const fn trusted_used_axioms_available(&self) -> bool {
        true
    }
}

/// One normalized evidence candidate considered by selection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProofEvidenceCandidate {
    source_id: CandidateSourceId,
    decision: PolicyDecision,
    trusted_kernel_evidence: Option<TrustedKernelEvidence>,
    backend_profile_priority: Option<u32>,
    evidence_format_priority: Option<u32>,
    evidence_hash: Option<Hash>,
    deterministic_discharge_hash: Option<Hash>,
    selected_proof_witness_hash: Option<Hash>,
    provenance_hash: Option<Hash>,
    diagnostic_refs: Vec<DiagnosticRef>,
}

impl ProofEvidenceCandidate {
    #[must_use]
    pub fn new(source_id: CandidateSourceId, decision: PolicyDecision) -> Self {
        Self {
            source_id,
            decision,
            trusted_kernel_evidence: None,
            backend_profile_priority: None,
            evidence_format_priority: None,
            evidence_hash: None,
            deterministic_discharge_hash: None,
            selected_proof_witness_hash: None,
            provenance_hash: None,
            diagnostic_refs: Vec::new(),
        }
    }

    #[must_use]
    pub fn from_trusted_kernel_input(
        source_id: CandidateSourceId,
        input: &KernelPolicyInput,
    ) -> Option<Self> {
        let trusted_kernel_evidence = TrustedKernelEvidence::from_policy_input(input)?;
        let class = match trusted_kernel_evidence.selected_class() {
            ProofWinnerClass::KernelVerified => CandidatePolicyClass::KernelVerified,
            ProofWinnerClass::DischargedBuiltin => CandidatePolicyClass::DischargedBuiltin,
            ProofWinnerClass::PolicyPermittedExternal
            | ProofWinnerClass::PolicyAssumed
            | ProofWinnerClass::PolicyOpen
            | ProofWinnerClass::Rejected
            | ProofWinnerClass::NoSelectableEvidence => return None,
        };

        Some(Self {
            source_id,
            decision: PolicyDecision {
                class,
                can_schedule_kernel_check: false,
                diagnostic: None,
                kernel_rejections: Vec::new(),
                external_admission: None,
            },
            trusted_kernel_evidence: Some(trusted_kernel_evidence.clone()),
            backend_profile_priority: None,
            evidence_format_priority: None,
            evidence_hash: Some(trusted_kernel_evidence.accepted_evidence_hash()),
            deterministic_discharge_hash: None,
            selected_proof_witness_hash: None,
            provenance_hash: None,
            diagnostic_refs: Vec::new(),
        })
    }

    #[cfg(test)]
    fn with_trusted_kernel_evidence(mut self, evidence: TrustedKernelEvidence) -> Self {
        self.trusted_kernel_evidence = Some(evidence);
        self
    }

    #[must_use]
    pub const fn with_backend_profile_priority(mut self, priority: u32) -> Self {
        self.backend_profile_priority = Some(priority);
        self
    }

    #[must_use]
    pub const fn with_evidence_format_priority(mut self, priority: u32) -> Self {
        self.evidence_format_priority = Some(priority);
        self
    }

    #[must_use]
    pub const fn with_evidence_hash(mut self, hash: Hash) -> Self {
        self.evidence_hash = Some(hash);
        self
    }

    #[must_use]
    pub const fn with_deterministic_discharge_hash(mut self, hash: Hash) -> Self {
        self.deterministic_discharge_hash = Some(hash);
        self
    }

    #[must_use]
    pub const fn with_selected_proof_witness_hash(mut self, hash: Hash) -> Self {
        self.selected_proof_witness_hash = Some(hash);
        self
    }

    #[must_use]
    pub const fn with_provenance_hash(mut self, hash: Hash) -> Self {
        self.provenance_hash = Some(hash);
        self
    }

    #[must_use]
    pub fn with_diagnostic_ref(mut self, diagnostic_ref: DiagnosticRef) -> Self {
        self.diagnostic_refs.push(diagnostic_ref);
        self
    }

    #[must_use]
    pub fn with_diagnostic_refs(
        mut self,
        diagnostic_refs: impl IntoIterator<Item = DiagnosticRef>,
    ) -> Self {
        self.diagnostic_refs.extend(diagnostic_refs);
        self
    }

    #[must_use]
    pub const fn source_id(&self) -> &CandidateSourceId {
        &self.source_id
    }

    #[must_use]
    pub const fn decision(&self) -> &PolicyDecision {
        &self.decision
    }

    #[must_use]
    pub const fn trusted_kernel_evidence(&self) -> Option<&TrustedKernelEvidence> {
        self.trusted_kernel_evidence.as_ref()
    }

    #[must_use]
    pub const fn backend_profile_priority(&self) -> Option<u32> {
        self.backend_profile_priority
    }

    #[must_use]
    pub const fn evidence_format_priority(&self) -> Option<u32> {
        self.evidence_format_priority
    }

    #[must_use]
    pub const fn evidence_hash(&self) -> Option<Hash> {
        self.evidence_hash
    }

    #[must_use]
    pub const fn deterministic_discharge_hash(&self) -> Option<Hash> {
        self.deterministic_discharge_hash
    }

    #[must_use]
    pub const fn selected_proof_witness_hash(&self) -> Option<Hash> {
        self.selected_proof_witness_hash
    }

    #[must_use]
    pub const fn provenance_hash(&self) -> Option<Hash> {
        self.provenance_hash
    }

    #[must_use]
    pub fn diagnostic_refs(&self) -> &[DiagnosticRef] {
        &self.diagnostic_refs
    }
}

/// Evidence set for one obligation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProofEvidenceSet {
    obligation_identity: Vec<u8>,
    encoded_problem_hash: Hash,
    policy: VerifierPolicy,
    candidates: Vec<ProofEvidenceCandidate>,
}

impl ProofEvidenceSet {
    #[must_use]
    pub fn new(
        obligation_identity: impl Into<Vec<u8>>,
        encoded_problem_hash: Hash,
        policy: VerifierPolicy,
    ) -> Self {
        Self {
            obligation_identity: obligation_identity.into(),
            encoded_problem_hash,
            policy,
            candidates: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_candidates(
        mut self,
        candidates: impl IntoIterator<Item = ProofEvidenceCandidate>,
    ) -> Self {
        self.candidates.extend(candidates);
        self
    }

    pub fn push_candidate(&mut self, candidate: ProofEvidenceCandidate) {
        self.candidates.push(candidate);
    }

    #[must_use]
    pub fn obligation_identity(&self) -> &[u8] {
        &self.obligation_identity
    }

    #[must_use]
    pub const fn encoded_problem_hash(&self) -> Hash {
        self.encoded_problem_hash
    }

    #[must_use]
    pub const fn policy(&self) -> &VerifierPolicy {
        &self.policy
    }

    #[must_use]
    pub fn candidates(&self) -> &[ProofEvidenceCandidate] {
        &self.candidates
    }

    #[must_use]
    pub fn policy_fingerprint(&self) -> PolicyFingerprint {
        self.policy.policy_fingerprint()
    }
}

/// Selected winner class or diagnostic outcome.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, std::hash::Hash)]
#[non_exhaustive]
pub enum ProofWinnerClass {
    KernelVerified,
    DischargedBuiltin,
    PolicyPermittedExternal,
    PolicyAssumed,
    PolicyOpen,
    Rejected,
    NoSelectableEvidence,
}

impl ProofWinnerClass {
    #[must_use]
    pub const fn is_trusted(self) -> bool {
        matches!(self, Self::KernelVerified | Self::DischargedBuiltin)
    }

    const fn rank(self) -> u8 {
        match self {
            Self::KernelVerified => 0,
            Self::DischargedBuiltin => 1,
            Self::PolicyPermittedExternal => 2,
            Self::PolicyAssumed => 3,
            Self::PolicyOpen => 4,
            Self::Rejected => 5,
            Self::NoSelectableEvidence => 6,
        }
    }

    const fn stable_key(self) -> &'static str {
        match self {
            Self::KernelVerified => "kernel_verified",
            Self::DischargedBuiltin => "discharged_builtin",
            Self::PolicyPermittedExternal => "policy_permitted_external",
            Self::PolicyAssumed => "policy_assumed",
            Self::PolicyOpen => "policy_open",
            Self::Rejected => "rejected",
            Self::NoSelectableEvidence => "no_selectable_evidence",
        }
    }
}

/// Stable reuse metadata emitted by selection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedReuseMetadata {
    selected_class: ProofWinnerClass,
    policy_fingerprint: PolicyFingerprint,
    encoded_problem_hash: Hash,
    selected_evidence_hash: Option<Hash>,
    selected_proof_witness_hash: Option<Hash>,
    deterministic_discharge_hash: Option<Hash>,
    external_admission_status: Option<ExternalEvidencePublicationStatus>,
    proof_witness_publication: ProofWitnessPublication,
    tie_break_key_hash: Hash,
}

impl SelectedReuseMetadata {
    #[must_use]
    pub const fn selected_class(&self) -> ProofWinnerClass {
        self.selected_class
    }

    #[must_use]
    pub const fn policy_fingerprint(&self) -> PolicyFingerprint {
        self.policy_fingerprint
    }

    #[must_use]
    pub const fn encoded_problem_hash(&self) -> Hash {
        self.encoded_problem_hash
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
    pub const fn external_admission_status(&self) -> Option<ExternalEvidencePublicationStatus> {
        self.external_admission_status
    }

    #[must_use]
    pub const fn proof_witness_publication(&self) -> ProofWitnessPublication {
        self.proof_witness_publication
    }

    #[must_use]
    pub const fn tie_break_key_hash(&self) -> Hash {
        self.tie_break_key_hash
    }
}

/// Whether selected proof witness publication is available for the winner.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, std::hash::Hash)]
#[non_exhaustive]
pub enum ProofWitnessPublication {
    Available,
    ExternalDependencyGap,
    NotApplicable,
}

/// Final deterministic selection result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProofSelection {
    selected_candidate_id: Option<CandidateSourceId>,
    selected_class: ProofWinnerClass,
    reuse_metadata: SelectedReuseMetadata,
    ordered_diagnostic_refs: Vec<DiagnosticRef>,
    trusted_used_axioms_available: bool,
    diagnostic_result_id: Option<Hash>,
}

/// Proof-selection input associated with one VC.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VcProofSelection {
    vc: VcId,
    selection: ProofSelection,
}

impl VcProofSelection {
    #[must_use]
    pub const fn new(vc: VcId, selection: ProofSelection) -> Self {
        Self { vc, selection }
    }

    #[must_use]
    pub const fn vc(&self) -> VcId {
        self.vc
    }

    #[must_use]
    pub const fn selection(&self) -> &ProofSelection {
        &self.selection
    }
}

/// Source of a merged artifact-facing proof selection.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, std::hash::Hash)]
#[non_exhaustive]
pub enum ProofSelectionSource {
    Portfolio,
    BuiltinDischarge,
}

impl ProofSelectionSource {
    const fn rank(self) -> u8 {
        match self {
            Self::Portfolio => 0,
            Self::BuiltinDischarge => 1,
        }
    }

    const fn allows_class(self, class: ProofWinnerClass) -> bool {
        match self {
            Self::Portfolio => !matches!(class, ProofWinnerClass::DischargedBuiltin),
            Self::BuiltinDischarge => matches!(
                class,
                ProofWinnerClass::DischargedBuiltin
                    | ProofWinnerClass::Rejected
                    | ProofWinnerClass::NoSelectableEvidence
            ),
        }
    }
}

/// Artifact-facing selected proof outcome before status projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArtifactProofSelection {
    vc: VcId,
    source: ProofSelectionSource,
    selection: ProofSelection,
}

impl ArtifactProofSelection {
    #[must_use]
    pub const fn vc(&self) -> VcId {
        self.vc
    }

    #[must_use]
    pub const fn source(&self) -> ProofSelectionSource {
        self.source
    }

    #[must_use]
    pub const fn selection(&self) -> &ProofSelection {
        &self.selection
    }

    #[must_use]
    pub const fn selected_class(&self) -> ProofWinnerClass {
        self.selection.selected_class()
    }
}

/// Artifact proof-selection merge error.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ArtifactProofSelectionError {
    DuplicateSelection {
        vc: VcId,
        source: ProofSelectionSource,
    },
    InvalidSourceClass {
        vc: VcId,
        source: ProofSelectionSource,
        selected_class: ProofWinnerClass,
    },
}

impl fmt::Display for ArtifactProofSelectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateSelection { vc, source } => {
                write!(f, "duplicate {source:?} proof selection for VC {vc:?}")
            }
            Self::InvalidSourceClass {
                vc,
                source,
                selected_class,
            } => write!(
                f,
                "invalid {source:?} proof selection class {selected_class:?} for VC {vc:?}"
            ),
        }
    }
}

impl Error for ArtifactProofSelectionError {}

impl ProofSelection {
    #[must_use]
    pub fn selected_candidate_id(&self) -> Option<&CandidateSourceId> {
        self.selected_candidate_id.as_ref()
    }

    #[must_use]
    pub const fn selected_class(&self) -> ProofWinnerClass {
        self.selected_class
    }

    #[must_use]
    pub const fn reuse_metadata(&self) -> &SelectedReuseMetadata {
        &self.reuse_metadata
    }

    #[must_use]
    pub fn ordered_diagnostic_refs(&self) -> &[DiagnosticRef] {
        &self.ordered_diagnostic_refs
    }

    #[must_use]
    pub const fn trusted_used_axioms_available(&self) -> bool {
        self.trusted_used_axioms_available
    }

    #[must_use]
    pub const fn diagnostic_result_id(&self) -> Option<Hash> {
        self.diagnostic_result_id
    }
}

/// Stateless deterministic selector.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ProofSelector;

impl ProofSelector {
    #[must_use]
    pub fn select(self, evidence_set: &ProofEvidenceSet) -> ProofSelection {
        select_winner(evidence_set)
    }
}

#[must_use]
pub fn select_winner(evidence_set: &ProofEvidenceSet) -> ProofSelection {
    let conflicted_ids = conflicting_candidate_ids(evidence_set);
    let generated_diagnostic_refs = generated_invalid_input_refs(evidence_set, &conflicted_ids);
    let has_pending_kernel_checkable = evidence_set
        .candidates()
        .iter()
        .any(|candidate| candidate.decision().class == CandidatePolicyClass::KernelCheckable);
    let mut selectable = Vec::new();
    let mut rejected = Vec::new();

    for candidate in evidence_set.candidates() {
        if conflicted_ids.contains_key(candidate.source_id()) {
            rejected.push(rejected_candidate(
                candidate,
                evidence_set,
                RejectionSourceRank::InvalidSelectionInput,
                "invalid_selection_input",
            ));
            continue;
        }

        if let Some(class) = selectable_class(
            candidate,
            evidence_set.policy(),
            has_pending_kernel_checkable,
        ) {
            selectable.push(selectable_candidate(candidate, evidence_set, class));
        } else if let Some(rejected_candidate) =
            rejected_candidate_from_policy(candidate, evidence_set)
        {
            rejected.push(rejected_candidate);
        }
    }

    selectable.sort_by(|left, right| left.key.cmp(&right.key));
    rejected.sort_by(|left, right| left.key.cmp(&right.key));

    if let Some(choice) = selectable.first() {
        return selected_candidate(
            evidence_set,
            choice.candidate,
            choice.class,
            choice.key_hash,
            &generated_diagnostic_refs,
        );
    }
    if let Some(choice) = rejected.first() {
        return selected_candidate(
            evidence_set,
            choice.candidate,
            ProofWinnerClass::Rejected,
            choice.key_hash,
            &generated_diagnostic_refs,
        );
    }

    no_selectable_evidence(evidence_set, &generated_diagnostic_refs)
}

pub fn merge_artifact_proof_selections(
    portfolio: impl IntoIterator<Item = VcProofSelection>,
    builtin_discharge: impl IntoIterator<Item = VcProofSelection>,
) -> Result<Vec<ArtifactProofSelection>, ArtifactProofSelectionError> {
    let mut by_vc = BTreeMap::<VcId, Vec<ArtifactMergeCandidate>>::new();

    for selection in portfolio {
        push_artifact_selection(&mut by_vc, selection, ProofSelectionSource::Portfolio)?;
    }
    for selection in builtin_discharge {
        push_artifact_selection(
            &mut by_vc,
            selection,
            ProofSelectionSource::BuiltinDischarge,
        )?;
    }

    Ok(by_vc
        .into_iter()
        .filter_map(|(vc, mut candidates)| {
            candidates.sort_by_key(ArtifactMergeCandidate::key);
            candidates
                .into_iter()
                .next()
                .map(|candidate| ArtifactProofSelection {
                    vc,
                    source: candidate.source,
                    selection: candidate.selection,
                })
        })
        .collect())
}

fn push_artifact_selection(
    by_vc: &mut BTreeMap<VcId, Vec<ArtifactMergeCandidate>>,
    selection: VcProofSelection,
    source: ProofSelectionSource,
) -> Result<(), ArtifactProofSelectionError> {
    if !source.allows_class(selection.selection().selected_class()) {
        return Err(ArtifactProofSelectionError::InvalidSourceClass {
            vc: selection.vc(),
            source,
            selected_class: selection.selection().selected_class(),
        });
    }

    let entries = by_vc.entry(selection.vc()).or_default();
    if entries.iter().any(|candidate| candidate.source == source) {
        return Err(ArtifactProofSelectionError::DuplicateSelection {
            vc: selection.vc(),
            source,
        });
    }
    entries.push(ArtifactMergeCandidate {
        source,
        selection: selection.selection,
    });
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ArtifactMergeCandidate {
    source: ProofSelectionSource,
    selection: ProofSelection,
}

impl ArtifactMergeCandidate {
    fn key(&self) -> ArtifactMergeKey {
        ArtifactMergeKey {
            class_rank: self.selection.selected_class().rank(),
            tie_break_key_hash: HashKey::from(self.selection.reuse_metadata().tie_break_key_hash()),
            source_rank: self.source.rank(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct ArtifactMergeKey {
    class_rank: u8,
    tie_break_key_hash: HashKey,
    source_rank: u8,
}

fn selectable_class(
    candidate: &ProofEvidenceCandidate,
    policy: &VerifierPolicy,
    has_pending_kernel_checkable: bool,
) -> Option<ProofWinnerClass> {
    match candidate.decision().class {
        CandidatePolicyClass::KernelVerified => {
            trusted_kernel_class(candidate, ProofWinnerClass::KernelVerified)
        }
        CandidatePolicyClass::DischargedBuiltin => {
            trusted_kernel_class(candidate, ProofWinnerClass::DischargedBuiltin)
        }
        CandidatePolicyClass::ExternallyAttested => candidate
            .decision()
            .external_admission
            .as_ref()
            .filter(|admission| admission.may_win_selection())
            .filter(|_| policy.external_evidence() == ExternalEvidenceMode::PermitNonTrustedWinner)
            .filter(|_| !policy.require_kernel_certificates())
            .filter(|_| !has_pending_kernel_checkable)
            .map(|_| ProofWinnerClass::PolicyPermittedExternal),
        CandidatePolicyClass::AssumedByPolicy
            if !policy.require_kernel_certificates()
                && !has_pending_kernel_checkable
                && policy.policy_assumption() == crate::policy::PolicyAssumptionMode::Record =>
        {
            Some(ProofWinnerClass::PolicyAssumed)
        }
        CandidatePolicyClass::OpenAllowed
            if !has_pending_kernel_checkable
                && policy.open_obligation() == OpenObligationMode::AllowPolicyOpen =>
        {
            Some(ProofWinnerClass::PolicyOpen)
        }
        CandidatePolicyClass::KernelRejected
        | CandidatePolicyClass::KernelCheckable
        | CandidatePolicyClass::OpenAllowed
        | CandidatePolicyClass::AssumedByPolicy
        | CandidatePolicyClass::RejectedByPolicy
        | CandidatePolicyClass::DiagnosticOnly => None,
    }
}

fn trusted_kernel_class(
    candidate: &ProofEvidenceCandidate,
    expected_class: ProofWinnerClass,
) -> Option<ProofWinnerClass> {
    candidate
        .trusted_kernel_evidence()
        .filter(|evidence| evidence.selected_class() == expected_class)
        .filter(|evidence| candidate.evidence_hash() == Some(evidence.accepted_evidence_hash()))
        .map(TrustedKernelEvidence::selected_class)
}

fn rejected_candidate_from_policy<'a>(
    candidate: &'a ProofEvidenceCandidate,
    evidence_set: &ProofEvidenceSet,
) -> Option<RejectedCandidate<'a>> {
    match candidate.decision().class {
        CandidatePolicyClass::KernelVerified | CandidatePolicyClass::DischargedBuiltin
            if trusted_kernel_class(
                candidate,
                match candidate.decision().class {
                    CandidatePolicyClass::KernelVerified => ProofWinnerClass::KernelVerified,
                    CandidatePolicyClass::DischargedBuiltin => ProofWinnerClass::DischargedBuiltin,
                    _ => unreachable!("guard restricts trusted kernel classes"),
                },
            )
            .is_none() =>
        {
            Some(rejected_candidate(
                candidate,
                evidence_set,
                RejectionSourceRank::InvalidSelectionInput,
                "trusted_kernel_evidence_mismatch",
            ))
        }
        CandidatePolicyClass::KernelRejected => Some(rejected_candidate(
            candidate,
            evidence_set,
            RejectionSourceRank::KernelRejection,
            kernel_rejection_reason(candidate),
        )),
        CandidatePolicyClass::KernelCheckable => Some(rejected_candidate(
            candidate,
            evidence_set,
            RejectionSourceRank::InvalidSelectionInput,
            "kernel_checkable_input_at_selection",
        )),
        CandidatePolicyClass::AssumedByPolicy
            if evidence_set.policy().require_kernel_certificates() =>
        {
            Some(rejected_candidate(
                candidate,
                evidence_set,
                RejectionSourceRank::PolicyRejection,
                "policy_assumption_requires_kernel_certificate",
            ))
        }
        CandidatePolicyClass::RejectedByPolicy => Some(rejected_candidate(
            candidate,
            evidence_set,
            rejection_source_from_policy(candidate.decision().diagnostic.as_ref()),
            policy_rejection_reason(candidate),
        )),
        CandidatePolicyClass::KernelVerified
        | CandidatePolicyClass::DischargedBuiltin
        | CandidatePolicyClass::ExternallyAttested
        | CandidatePolicyClass::OpenAllowed
        | CandidatePolicyClass::AssumedByPolicy
        | CandidatePolicyClass::DiagnosticOnly => None,
    }
}

fn selected_candidate(
    evidence_set: &ProofEvidenceSet,
    candidate: &ProofEvidenceCandidate,
    selected_class: ProofWinnerClass,
    tie_break_key_hash: Hash,
    generated_diagnostic_refs: &[DiagnosticRef],
) -> ProofSelection {
    let trusted_used_axioms_available = selected_class.is_trusted()
        && candidate
            .trusted_kernel_evidence()
            .is_some_and(TrustedKernelEvidence::trusted_used_axioms_available);
    let selected_candidate_id = Some(candidate.source_id().clone());
    let ordered_diagnostic_refs = ordered_diagnostic_refs(
        evidence_set
            .candidates()
            .iter()
            .filter(|other| other.source_id() != candidate.source_id()),
        generated_diagnostic_refs,
    );
    let proof_witness_publication = proof_witness_publication(candidate, selected_class);

    let reuse_metadata = SelectedReuseMetadata {
        selected_class,
        policy_fingerprint: evidence_set.policy_fingerprint(),
        encoded_problem_hash: evidence_set.encoded_problem_hash(),
        selected_evidence_hash: selected_evidence_hash(candidate, selected_class),
        selected_proof_witness_hash: (proof_witness_publication
            == ProofWitnessPublication::Available)
            .then_some(candidate.selected_proof_witness_hash())
            .flatten(),
        deterministic_discharge_hash: (selected_class == ProofWinnerClass::DischargedBuiltin)
            .then_some(candidate.deterministic_discharge_hash())
            .flatten(),
        external_admission_status: candidate
            .decision()
            .external_admission
            .as_ref()
            .map(|admission| admission.publication_status()),
        proof_witness_publication,
        tie_break_key_hash,
    };

    ProofSelection {
        selected_candidate_id,
        selected_class,
        reuse_metadata,
        ordered_diagnostic_refs,
        trusted_used_axioms_available,
        diagnostic_result_id: None,
    }
}

fn no_selectable_evidence(
    evidence_set: &ProofEvidenceSet,
    generated_diagnostic_refs: &[DiagnosticRef],
) -> ProofSelection {
    let diagnostic_result_id = diagnostic_result_id(evidence_set);
    let reuse_metadata = SelectedReuseMetadata {
        selected_class: ProofWinnerClass::NoSelectableEvidence,
        policy_fingerprint: evidence_set.policy_fingerprint(),
        encoded_problem_hash: evidence_set.encoded_problem_hash(),
        selected_evidence_hash: None,
        selected_proof_witness_hash: None,
        deterministic_discharge_hash: None,
        external_admission_status: None,
        proof_witness_publication: ProofWitnessPublication::NotApplicable,
        tie_break_key_hash: diagnostic_result_id,
    };

    ProofSelection {
        selected_candidate_id: None,
        selected_class: ProofWinnerClass::NoSelectableEvidence,
        reuse_metadata,
        ordered_diagnostic_refs: ordered_diagnostic_refs(
            evidence_set.candidates().iter(),
            generated_diagnostic_refs,
        ),
        trusted_used_axioms_available: false,
        diagnostic_result_id: Some(diagnostic_result_id),
    }
}

fn proof_witness_publication(
    candidate: &ProofEvidenceCandidate,
    selected_class: ProofWinnerClass,
) -> ProofWitnessPublication {
    match selected_class {
        ProofWinnerClass::KernelVerified if candidate.selected_proof_witness_hash().is_some() => {
            ProofWitnessPublication::Available
        }
        ProofWinnerClass::DischargedBuiltin => ProofWitnessPublication::ExternalDependencyGap,
        ProofWinnerClass::KernelVerified
        | ProofWinnerClass::PolicyPermittedExternal
        | ProofWinnerClass::PolicyAssumed
        | ProofWinnerClass::PolicyOpen
        | ProofWinnerClass::Rejected
        | ProofWinnerClass::NoSelectableEvidence => ProofWitnessPublication::NotApplicable,
    }
}

fn selectable_candidate<'a>(
    candidate: &'a ProofEvidenceCandidate,
    evidence_set: &ProofEvidenceSet,
    class: ProofWinnerClass,
) -> SelectableCandidate<'a> {
    let key = TieBreakKey::new(candidate, evidence_set, class);
    let key_hash = hash_tie_break_key(&key);
    SelectableCandidate {
        candidate,
        class,
        key,
        key_hash,
    }
}

fn rejected_candidate<'a>(
    candidate: &'a ProofEvidenceCandidate,
    evidence_set: &ProofEvidenceSet,
    source_rank: RejectionSourceRank,
    reason_code: &'static str,
) -> RejectedCandidate<'a> {
    let key = RejectedKey::new(candidate, evidence_set, source_rank, reason_code);
    let key_hash = hash_rejected_key(&key);
    RejectedCandidate {
        candidate,
        key,
        key_hash,
    }
}

fn selected_evidence_hash(
    candidate: &ProofEvidenceCandidate,
    selected_class: ProofWinnerClass,
) -> Option<Hash> {
    match selected_class {
        ProofWinnerClass::DischargedBuiltin => candidate
            .evidence_hash()
            .or_else(|| candidate.deterministic_discharge_hash()),
        ProofWinnerClass::NoSelectableEvidence => None,
        ProofWinnerClass::KernelVerified
        | ProofWinnerClass::PolicyPermittedExternal
        | ProofWinnerClass::PolicyAssumed
        | ProofWinnerClass::PolicyOpen
        | ProofWinnerClass::Rejected => candidate
            .evidence_hash()
            .or_else(|| candidate.deterministic_discharge_hash()),
    }
}

fn ordered_diagnostic_refs<'a>(
    candidates: impl IntoIterator<Item = &'a ProofEvidenceCandidate>,
    generated_diagnostic_refs: &[DiagnosticRef],
) -> Vec<DiagnosticRef> {
    let mut refs = candidates
        .into_iter()
        .flat_map(ProofEvidenceCandidate::diagnostic_refs)
        .copied()
        .collect::<Vec<_>>();
    refs.extend_from_slice(generated_diagnostic_refs);
    refs.sort();
    refs.dedup();
    refs
}

fn conflicting_candidate_ids(evidence_set: &ProofEvidenceSet) -> BTreeMap<CandidateSourceId, Hash> {
    let mut seen = BTreeMap::new();
    let mut conflicted = BTreeMap::new();

    for candidate in evidence_set.candidates() {
        let payload_hash = canonical_payload_hash(candidate);
        match seen.insert(candidate.source_id().clone(), payload_hash) {
            Some(previous) if previous != payload_hash => {
                conflicted.insert(candidate.source_id().clone(), payload_hash);
            }
            _ => {}
        }
    }

    conflicted
}

fn generated_invalid_input_refs(
    evidence_set: &ProofEvidenceSet,
    conflicted_ids: &BTreeMap<CandidateSourceId, Hash>,
) -> Vec<DiagnosticRef> {
    conflicted_ids
        .keys()
        .map(|source_id| {
            let mut hash = StableHasher::new(SELECTION_DIAGNOSTIC_HASH_DOMAIN);
            hash.field_bytes("obligation_identity", evidence_set.obligation_identity());
            hash.field_hash("encoded_problem_hash", evidence_set.encoded_problem_hash());
            hash.field_hash(
                "policy_fingerprint",
                evidence_set.policy_fingerprint().hash(),
            );
            hash.field_str("reason", "invalid_selection_input");
            hash.field_str("candidate_source_id", source_id.as_str());
            DiagnosticRef::new(hash.finalize())
        })
        .collect()
}

fn canonical_payload_hash(candidate: &ProofEvidenceCandidate) -> Hash {
    let mut hash = StableHasher::new(SELECTION_PAYLOAD_HASH_DOMAIN);
    hash.field_str("class", candidate_class_str(candidate.decision().class));
    hash.field_bool(
        "can_schedule_kernel_check",
        candidate.decision().can_schedule_kernel_check,
    );
    hash_policy_diagnostic(
        &mut hash,
        "diagnostic",
        candidate.decision().diagnostic.as_ref(),
    );
    hash.field_bool(
        "has_external_admission",
        candidate.decision().external_admission.is_some(),
    );
    hash.field_bool(
        "has_trusted_kernel_evidence",
        candidate.trusted_kernel_evidence().is_some(),
    );
    if let Some(trusted_kernel_evidence) = candidate.trusted_kernel_evidence() {
        hash.field_str(
            "trusted_kernel_class",
            trusted_kernel_evidence.selected_class().stable_key(),
        );
        hash.field_hash(
            "trusted_kernel_evidence_hash",
            trusted_kernel_evidence.accepted_evidence_hash(),
        );
    }
    if let Some(admission) = &candidate.decision().external_admission {
        hash.field_bool(
            "external_record_as_development",
            admission.record_as_development_evidence(),
        );
        hash.field_bool("external_may_win", admission.may_win_selection());
        hash.field_str(
            "external_publication_status",
            external_publication_status_str(admission.publication_status()),
        );
    }
    hash.field_optional_u32(
        "backend_profile_priority",
        candidate.backend_profile_priority(),
    );
    hash.field_optional_u32(
        "evidence_format_priority",
        candidate.evidence_format_priority(),
    );
    hash.field_optional_hash("evidence_hash", candidate.evidence_hash());
    hash.field_optional_hash(
        "deterministic_discharge_hash",
        candidate.deterministic_discharge_hash(),
    );
    hash.field_optional_hash(
        "proof_witness_hash",
        candidate.selected_proof_witness_hash(),
    );
    hash.field_optional_hash("provenance_hash", candidate.provenance_hash());
    let mut diagnostic_refs = candidate.diagnostic_refs().to_vec();
    diagnostic_refs.sort();
    for diagnostic_ref in diagnostic_refs {
        hash.field_hash("diagnostic_ref", diagnostic_ref.hash());
    }
    hash.finalize()
}

fn hash_policy_diagnostic(
    hash: &mut StableHasher,
    label: &str,
    diagnostic: Option<&PolicyDiagnostic>,
) {
    hash.field_bool(&format!("{label}_present"), diagnostic.is_some());
    if let Some(diagnostic) = diagnostic {
        hash.field_str(&format!("{label}_category"), diagnostic.category.as_str());
        hash.field_str(&format!("{label}_reason"), diagnostic.reason.as_str());
    }
}

fn diagnostic_result_id(evidence_set: &ProofEvidenceSet) -> Hash {
    let mut hash = StableHasher::new(SELECTION_DIAGNOSTIC_HASH_DOMAIN);
    hash.field_bytes("obligation_identity", evidence_set.obligation_identity());
    hash.field_hash("encoded_problem_hash", evidence_set.encoded_problem_hash());
    hash.field_hash(
        "policy_fingerprint",
        evidence_set.policy_fingerprint().hash(),
    );
    hash.field_str("reason", "no_selectable_evidence");
    hash.finalize()
}

fn hash_tie_break_key(key: &TieBreakKey) -> Hash {
    let mut hash = StableHasher::new(SELECTION_TIE_BREAK_HASH_DOMAIN);
    hash.field_u8("winner_rank", key.winner_rank);
    hash.field_u32("backend_profile_priority", key.backend_profile_priority);
    hash.field_u32("evidence_format_priority", key.evidence_format_priority);
    hash.field_hash_key("encoded_problem_hash", key.encoded_problem_hash);
    hash.field_str("policy_profile_id", &key.policy_profile_id);
    hash.field_optional_hash_key("evidence_hash", key.evidence_hash);
    hash.field_optional_hash_key("provenance_hash", key.provenance_hash);
    hash.field_str("candidate_source_id", key.candidate_source_id.as_str());
    hash.finalize()
}

fn hash_rejected_key(key: &RejectedKey) -> Hash {
    let mut hash = StableHasher::new(SELECTION_TIE_BREAK_HASH_DOMAIN);
    hash.field_u8("rejection_source_rank", key.source_rank);
    hash.field_u8("failure_category_rank", key.failure_category_rank);
    hash.field_u8("severity_rank", key.severity_rank);
    hash.field_str("reason_code", &key.reason_code);
    hash.field_hash_key("encoded_problem_hash", key.encoded_problem_hash);
    hash.field_optional_hash_key("evidence_hash", key.evidence_hash);
    hash.field_optional_hash_key("provenance_hash", key.provenance_hash);
    hash.field_str("candidate_source_id", key.candidate_source_id.as_str());
    hash.finalize()
}

fn kernel_rejection_reason(candidate: &ProofEvidenceCandidate) -> &'static str {
    candidate.decision().kernel_rejections.iter().min().map_or(
        "kernel_rejection",
        mizar_kernel::rejection::RejectionRecord::stable_detail_key,
    )
}

fn policy_rejection_reason(candidate: &ProofEvidenceCandidate) -> &'static str {
    candidate
        .decision()
        .diagnostic
        .as_ref()
        .map_or("policy_rejection", |diagnostic| diagnostic.reason.as_str())
}

fn rejection_source_from_policy(diagnostic: Option<&PolicyDiagnostic>) -> RejectionSourceRank {
    match diagnostic.map(|diagnostic| diagnostic.reason) {
        Some(
            PolicyReasonCode::KernelEvidenceTargetMismatch
            | PolicyReasonCode::KernelEvidenceFormatDisabled
            | PolicyReasonCode::MissingBuiltinKernelRepresentation
            | PolicyReasonCode::KernelPrimitiveNotAllowed
            | PolicyReasonCode::LegacyReplayRejected,
        ) => RejectionSourceRank::EvidenceFormatRejection,
        _ => RejectionSourceRank::PolicyRejection,
    }
}

fn candidate_class_str(class: CandidatePolicyClass) -> &'static str {
    match class {
        CandidatePolicyClass::KernelVerified => "kernel_verified",
        CandidatePolicyClass::DischargedBuiltin => "discharged_builtin",
        CandidatePolicyClass::KernelRejected => "kernel_rejected",
        CandidatePolicyClass::KernelCheckable => "kernel_checkable",
        CandidatePolicyClass::ExternallyAttested => "externally_attested",
        CandidatePolicyClass::OpenAllowed => "open_allowed",
        CandidatePolicyClass::AssumedByPolicy => "assumed_by_policy",
        CandidatePolicyClass::RejectedByPolicy => "rejected_by_policy",
        CandidatePolicyClass::DiagnosticOnly => "diagnostic_only",
    }
}

fn external_publication_status_str(status: ExternalEvidencePublicationStatus) -> &'static str {
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

#[derive(Clone, Debug, Eq, PartialEq)]
struct SelectableCandidate<'a> {
    candidate: &'a ProofEvidenceCandidate,
    class: ProofWinnerClass,
    key: TieBreakKey,
    key_hash: Hash,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RejectedCandidate<'a> {
    candidate: &'a ProofEvidenceCandidate,
    key: RejectedKey,
    key_hash: Hash,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct TieBreakKey {
    winner_rank: u8,
    backend_profile_priority: u32,
    evidence_format_priority: u32,
    encoded_problem_hash: HashKey,
    policy_profile_id: String,
    evidence_hash: OptionalHashKey,
    provenance_hash: OptionalHashKey,
    candidate_source_id: CandidateSourceId,
}

impl TieBreakKey {
    fn new(
        candidate: &ProofEvidenceCandidate,
        evidence_set: &ProofEvidenceSet,
        class: ProofWinnerClass,
    ) -> Self {
        Self {
            winner_rank: class.rank(),
            backend_profile_priority: candidate
                .backend_profile_priority()
                .unwrap_or(MISSING_PRIORITY_SENTINEL),
            evidence_format_priority: candidate
                .evidence_format_priority()
                .unwrap_or(MISSING_PRIORITY_SENTINEL),
            encoded_problem_hash: HashKey::from(evidence_set.encoded_problem_hash()),
            policy_profile_id: evidence_set.policy().profile_id().to_owned(),
            evidence_hash: OptionalHashKey::from_hash(selected_evidence_hash(candidate, class)),
            provenance_hash: OptionalHashKey::from_hash(candidate.provenance_hash()),
            candidate_source_id: candidate.source_id().clone(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct RejectedKey {
    source_rank: u8,
    failure_category_rank: u8,
    severity_rank: u8,
    reason_code: String,
    encoded_problem_hash: HashKey,
    evidence_hash: OptionalHashKey,
    provenance_hash: OptionalHashKey,
    candidate_source_id: CandidateSourceId,
}

impl RejectedKey {
    fn new(
        candidate: &ProofEvidenceCandidate,
        evidence_set: &ProofEvidenceSet,
        source_rank: RejectionSourceRank,
        reason_code: &'static str,
    ) -> Self {
        Self {
            source_rank: source_rank.rank(),
            failure_category_rank: failure_category_rank(candidate, source_rank),
            severity_rank: 0,
            reason_code: reason_code.to_owned(),
            encoded_problem_hash: HashKey::from(evidence_set.encoded_problem_hash()),
            evidence_hash: OptionalHashKey::from_hash(
                candidate
                    .evidence_hash()
                    .or_else(|| candidate.deterministic_discharge_hash()),
            ),
            provenance_hash: OptionalHashKey::from_hash(candidate.provenance_hash()),
            candidate_source_id: candidate.source_id().clone(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RejectionSourceRank {
    KernelRejection,
    EvidenceFormatRejection,
    PolicyRejection,
    InvalidSelectionInput,
}

impl RejectionSourceRank {
    const fn rank(self) -> u8 {
        match self {
            Self::KernelRejection => 0,
            Self::EvidenceFormatRejection => 1,
            Self::PolicyRejection => 2,
            Self::InvalidSelectionInput => 3,
        }
    }
}

fn failure_category_rank(
    candidate: &ProofEvidenceCandidate,
    source_rank: RejectionSourceRank,
) -> u8 {
    if source_rank == RejectionSourceRank::KernelRejection {
        return candidate
            .decision()
            .kernel_rejections
            .iter()
            .map(|record| match record.category().stable_key() {
                "certificate_rejection" => 0,
                "kernel_rejection" => 1,
                _ => 2,
            })
            .min()
            .unwrap_or(0);
    }

    match source_rank {
        RejectionSourceRank::KernelRejection => 0,
        RejectionSourceRank::EvidenceFormatRejection => 1,
        RejectionSourceRank::PolicyRejection => 2,
        RejectionSourceRank::InvalidSelectionInput => 3,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct HashKey([u8; Hash::BYTE_LEN]);

impl From<Hash> for HashKey {
    fn from(hash: Hash) -> Self {
        Self(*hash.as_bytes())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
enum OptionalHashKey {
    Present(HashKey),
    Missing,
}

impl OptionalHashKey {
    fn from_hash(hash: Option<Hash>) -> Self {
        hash.map_or(Self::Missing, |hash| Self::Present(HashKey::from(hash)))
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
                0x6f_6f_66_2d_73_65_6c_65,
                0x63_74_69_6f_6e_2d_68_31,
                0x77_69_6e_6e_65_72_2d_31,
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

    fn field_u32(&mut self, label: &str, value: u32) {
        self.field_bytes(label, &value.to_le_bytes());
    }

    fn field_optional_u32(&mut self, label: &str, value: Option<u32>) {
        match value {
            Some(value) => {
                self.field_u8(&format!("{label}_tag"), 0);
                self.field_u32(label, value);
            }
            None => self.field_u8(&format!("{label}_tag"), 1),
        }
    }

    fn field_hash(&mut self, label: &str, value: Hash) {
        self.field_bytes(label, value.as_bytes());
    }

    fn field_hash_key(&mut self, label: &str, value: HashKey) {
        self.field_bytes(label, &value.0);
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

    fn field_optional_hash_key(&mut self, label: &str, value: OptionalHashKey) {
        match value {
            OptionalHashKey::Present(value) => {
                self.field_u8(&format!("{label}_tag"), 0);
                self.field_hash_key(label, value);
            }
            OptionalHashKey::Missing => self.field_u8(&format!("{label}_tag"), 1),
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
        for (chunk, lane) in bytes.chunks_exact_mut(8).zip(self.lanes) {
            chunk.copy_from_slice(&lane.to_be_bytes());
        }
        Hash::from_bytes(bytes)
    }
}

#[cfg(test)]
mod tests {
    use crate::policy::{
        ExternalEvidenceMode, KernelEvidenceOrigin, PolicyAssumptionMode, PolicyCandidate,
        PolicyDiagnosticCategory, ProofPolicyEvaluator,
    };
    use mizar_kernel::rejection::{
        RejectionCategory, RejectionDetail, RejectionLocation, RejectionRecord, TargetVcFingerprint,
    };

    use super::*;

    #[test]
    fn candidate_source_id_rejects_empty_id() {
        assert_eq!(
            CandidateSourceId::new(""),
            Err(SelectionInputError::EmptyCandidateSourceId)
        );
        assert_eq!(
            CandidateSourceId::new("stable-candidate")
                .expect("stable id")
                .as_str(),
            "stable-candidate"
        );
    }

    #[test]
    fn winner_order_prefers_trusted_then_policy_classes() {
        let policy = VerifierPolicy::interactive()
            .with_external_evidence(ExternalEvidenceMode::PermitNonTrustedWinner)
            .with_policy_assumption(PolicyAssumptionMode::Record);
        let evaluator = ProofPolicyEvaluator::new(policy.clone());
        let set = base_set(policy).with_candidates([
            candidate("rejected", rejected_policy_decision()).with_evidence_hash(hash(6)),
            candidate(
                "open",
                evaluator.evaluate_candidate(&PolicyCandidate::OpenObligation),
            )
            .with_evidence_hash(hash(5)),
            candidate(
                "assumed",
                evaluator.evaluate_candidate(&PolicyCandidate::PolicyAssumption),
            )
            .with_evidence_hash(hash(4)),
            candidate(
                "external",
                evaluator.evaluate_candidate(&PolicyCandidate::ExternallyAttested),
            )
            .with_evidence_hash(hash(3)),
            trusted_candidate(
                "builtin",
                CandidatePolicyClass::DischargedBuiltin,
                KernelEvidenceOrigin::BuiltinDischarge,
                hash(22),
            )
            .with_deterministic_discharge_hash(hash(2)),
            trusted_candidate(
                "kernel",
                CandidatePolicyClass::KernelVerified,
                KernelEvidenceOrigin::AtpFormulaSubstitution,
                hash(1),
            ),
        ]);

        let selection = select_winner(&set);

        assert_eq!(selection.selected_class(), ProofWinnerClass::KernelVerified);
        assert_eq!(
            selection
                .selected_candidate_id()
                .map(CandidateSourceId::as_str),
            Some("kernel")
        );
        assert!(selection.trusted_used_axioms_available());
    }

    #[test]
    fn winner_order_exercises_adjacent_boundaries() {
        let policy = VerifierPolicy::interactive()
            .with_external_evidence(ExternalEvidenceMode::PermitNonTrustedWinner)
            .with_policy_assumption(PolicyAssumptionMode::Record);
        let evaluator = ProofPolicyEvaluator::new(policy.clone());
        let builtin = trusted_candidate(
            "builtin",
            CandidatePolicyClass::DischargedBuiltin,
            KernelEvidenceOrigin::BuiltinDischarge,
            hash(21),
        )
        .with_deterministic_discharge_hash(hash(22));
        let external = candidate(
            "external",
            evaluator.evaluate_candidate(&PolicyCandidate::ExternallyAttested),
        )
        .with_evidence_hash(hash(23));
        let assumed = candidate(
            "assumed",
            evaluator.evaluate_candidate(&PolicyCandidate::PolicyAssumption),
        )
        .with_evidence_hash(hash(24));
        let open = candidate(
            "open",
            evaluator.evaluate_candidate(&PolicyCandidate::OpenObligation),
        )
        .with_evidence_hash(hash(25));
        let rejected =
            candidate("rejected", rejected_policy_decision()).with_evidence_hash(hash(26));

        assert_selected(
            &base_set(policy.clone()).with_candidates([
                external.clone(),
                assumed.clone(),
                open.clone(),
                rejected.clone(),
                builtin,
            ]),
            ProofWinnerClass::DischargedBuiltin,
            "builtin",
        );
        assert_selected(
            &base_set(policy.clone()).with_candidates([
                assumed.clone(),
                open.clone(),
                rejected.clone(),
                external,
            ]),
            ProofWinnerClass::PolicyPermittedExternal,
            "external",
        );
        assert_selected(
            &base_set(policy.clone()).with_candidates([open.clone(), rejected.clone(), assumed]),
            ProofWinnerClass::PolicyAssumed,
            "assumed",
        );
        assert_selected(
            &base_set(policy.clone()).with_candidates([rejected.clone(), open]),
            ProofWinnerClass::PolicyOpen,
            "open",
        );
        assert_selected(
            &base_set(policy).with_candidates([rejected]),
            ProofWinnerClass::Rejected,
            "rejected",
        );
    }

    #[test]
    fn shuffled_candidate_arrival_never_changes_the_winner() {
        let policy = VerifierPolicy::release();
        let left = trusted_candidate(
            "preferred",
            CandidatePolicyClass::KernelVerified,
            KernelEvidenceOrigin::AtpFormulaSubstitution,
            hash(2),
        )
        .with_backend_profile_priority(0)
        .with_evidence_format_priority(0)
        .with_provenance_hash(hash(9));
        let right = trusted_candidate(
            "later",
            CandidatePolicyClass::KernelVerified,
            KernelEvidenceOrigin::AtpFormulaSubstitution,
            hash(1),
        )
        .with_backend_profile_priority(1)
        .with_evidence_format_priority(0)
        .with_provenance_hash(hash(8));

        let first =
            select_winner(&base_set(policy.clone()).with_candidates([left.clone(), right.clone()]));
        let second = select_winner(&base_set(policy).with_candidates([right, left]));

        assert_eq!(
            first.selected_candidate_id(),
            second.selected_candidate_id()
        );
        assert_eq!(
            first.selected_candidate_id().map(CandidateSourceId::as_str),
            Some("preferred")
        );
        assert_eq!(
            first.reuse_metadata().tie_break_key_hash(),
            second.reuse_metadata().tie_break_key_hash()
        );
    }

    #[test]
    fn tie_break_uses_present_hashes_and_priority_sentinels() {
        let policy = VerifierPolicy::interactive()
            .with_external_evidence(ExternalEvidenceMode::PermitNonTrustedWinner);
        let evaluator = ProofPolicyEvaluator::new(policy.clone());
        let external = evaluator.evaluate_candidate(&PolicyCandidate::ExternallyAttested);
        let with_present_hash =
            candidate("with-hash", external.clone()).with_evidence_hash(hash(9));
        let missing_hash = candidate("missing-hash", external.clone());
        let explicit_priority = candidate("explicit-priority", external)
            .with_backend_profile_priority(1)
            .with_evidence_hash(hash(10));

        let selection = select_winner(&base_set(policy).with_candidates([
            missing_hash,
            with_present_hash,
            explicit_priority,
        ]));

        assert_eq!(
            selection
                .selected_candidate_id()
                .map(CandidateSourceId::as_str),
            Some("explicit-priority")
        );
        assert_eq!(
            selection.reuse_metadata().selected_evidence_hash(),
            Some(hash(10))
        );
    }

    #[test]
    fn tie_break_covers_format_hash_provenance_and_source_id() {
        let explicit_format = trusted_candidate(
            "explicit-format",
            CandidatePolicyClass::KernelVerified,
            KernelEvidenceOrigin::AtpFormulaSubstitution,
            hash(31),
        )
        .with_backend_profile_priority(0)
        .with_evidence_format_priority(0);
        let missing_format = trusted_candidate(
            "missing-format",
            CandidatePolicyClass::KernelVerified,
            KernelEvidenceOrigin::AtpFormulaSubstitution,
            hash(30),
        )
        .with_backend_profile_priority(0);
        assert_selected(
            &base_set(VerifierPolicy::release()).with_candidates([missing_format, explicit_format]),
            ProofWinnerClass::KernelVerified,
            "explicit-format",
        );

        let policy = VerifierPolicy::interactive();
        let evaluator = ProofPolicyEvaluator::new(policy.clone());
        let open_decision = evaluator.evaluate_candidate(&PolicyCandidate::OpenObligation);
        let present_hash =
            candidate("present-hash", open_decision.clone()).with_evidence_hash(hash(40));
        let missing_hash = candidate("missing-hash", open_decision.clone());
        assert_selected(
            &base_set(policy.clone()).with_candidates([missing_hash, present_hash]),
            ProofWinnerClass::PolicyOpen,
            "present-hash",
        );

        let provenance_low = candidate("prov-low", open_decision.clone())
            .with_evidence_hash(hash(41))
            .with_provenance_hash(hash(1));
        let provenance_high = candidate("prov-high", open_decision.clone())
            .with_evidence_hash(hash(41))
            .with_provenance_hash(hash(2));
        assert_selected(
            &base_set(policy.clone()).with_candidates([provenance_high, provenance_low]),
            ProofWinnerClass::PolicyOpen,
            "prov-low",
        );

        let source_a = candidate("source-a", open_decision.clone()).with_evidence_hash(hash(42));
        let source_b = candidate("source-b", open_decision).with_evidence_hash(hash(42));
        assert_selected(
            &base_set(policy).with_candidates([source_b, source_a]),
            ProofWinnerClass::PolicyOpen,
            "source-a",
        );
    }

    #[test]
    fn conflicting_duplicate_candidate_ids_are_reported_deterministically() {
        let policy = VerifierPolicy::interactive();
        let evaluator = ProofPolicyEvaluator::new(policy.clone());
        let first = candidate(
            "duplicate",
            evaluator.evaluate_candidate(&PolicyCandidate::OpenObligation),
        )
        .with_evidence_hash(hash(50));
        let second = candidate(
            "duplicate",
            evaluator.evaluate_candidate(&PolicyCandidate::OpenObligation),
        )
        .with_evidence_hash(hash(51));

        let conflict_only = select_winner(
            &base_set(policy.clone()).with_candidates([first.clone(), second.clone()]),
        );
        assert_eq!(conflict_only.selected_class(), ProofWinnerClass::Rejected);
        assert_eq!(
            conflict_only
                .selected_candidate_id()
                .map(CandidateSourceId::as_str),
            Some("duplicate")
        );
        assert!(!conflict_only.ordered_diagnostic_refs().is_empty());

        let trusted = trusted_candidate(
            "trusted",
            CandidatePolicyClass::KernelVerified,
            KernelEvidenceOrigin::AtpFormulaSubstitution,
            hash(52),
        );
        let with_trusted =
            select_winner(&base_set(policy).with_candidates([first, trusted, second]));
        assert_eq!(
            with_trusted.selected_class(),
            ProofWinnerClass::KernelVerified
        );
        assert_eq!(
            with_trusted
                .selected_candidate_id()
                .map(CandidateSourceId::as_str),
            Some("trusted")
        );
        assert!(!with_trusted.ordered_diagnostic_refs().is_empty());
    }

    #[test]
    fn external_and_policy_assumption_do_not_win_when_kernel_certificates_are_required() {
        let policy = VerifierPolicy::interactive()
            .with_external_evidence(ExternalEvidenceMode::PermitNonTrustedWinner)
            .with_require_kernel_certificates(true)
            .with_policy_assumption(PolicyAssumptionMode::Record);
        let evaluator = ProofPolicyEvaluator::new(policy.clone());
        let external = candidate(
            "external",
            evaluator.evaluate_candidate(&PolicyCandidate::ExternallyAttested),
        )
        .with_evidence_hash(hash(1));
        let assumption = candidate(
            "assumption",
            evaluator.evaluate_candidate(&PolicyCandidate::PolicyAssumption),
        )
        .with_evidence_hash(hash(2));

        let selection = select_winner(&base_set(policy).with_candidates([external, assumption]));

        assert_eq!(selection.selected_class(), ProofWinnerClass::Rejected);
        assert_eq!(
            selection
                .selected_candidate_id()
                .map(CandidateSourceId::as_str),
            Some("assumption")
        );
        assert!(!selection.trusted_used_axioms_available());
    }

    #[test]
    fn pending_kernel_checkable_blocks_non_trusted_winners() {
        let policy = VerifierPolicy::interactive()
            .with_external_evidence(ExternalEvidenceMode::PermitNonTrustedWinner)
            .with_policy_assumption(PolicyAssumptionMode::Record);
        let evaluator = ProofPolicyEvaluator::new(policy.clone());
        let external = candidate(
            "external",
            evaluator.evaluate_candidate(&PolicyCandidate::ExternallyAttested),
        )
        .with_evidence_hash(hash(61));
        let open = candidate(
            "open",
            evaluator.evaluate_candidate(&PolicyCandidate::OpenObligation),
        )
        .with_evidence_hash(hash(62));
        let kernel_checkable = candidate(
            "kernel-checkable",
            evaluator.evaluate_candidate(&PolicyCandidate::UncheckedFormulaSubstitution {
                encoded_problem_matches: true,
            }),
        )
        .with_evidence_hash(hash(63));

        let selection =
            select_winner(&base_set(policy).with_candidates([external, open, kernel_checkable]));

        assert_eq!(selection.selected_class(), ProofWinnerClass::Rejected);
        assert_eq!(
            selection
                .selected_candidate_id()
                .map(CandidateSourceId::as_str),
            Some("kernel-checkable")
        );
    }

    #[test]
    fn diagnostic_only_and_empty_sets_have_no_selected_winner() {
        let empty = select_winner(&base_set(VerifierPolicy::development()));
        assert_eq!(
            empty.selected_class(),
            ProofWinnerClass::NoSelectableEvidence
        );
        assert!(empty.selected_candidate_id().is_none());
        assert!(empty.diagnostic_result_id().is_some());
        assert!(!empty.trusted_used_axioms_available());

        let evaluator = ProofPolicyEvaluator::new(VerifierPolicy::development());
        let diagnostic = candidate(
            "diagnostic",
            evaluator.evaluate_candidate(&PolicyCandidate::BackendDiagnostic),
        )
        .with_diagnostic_ref(DiagnosticRef::new(hash(44)));
        let selection =
            select_winner(&base_set(evaluator.policy().clone()).with_candidates([diagnostic]));

        assert_eq!(
            selection.selected_class(),
            ProofWinnerClass::NoSelectableEvidence
        );
        assert!(selection.selected_candidate_id().is_none());
        assert_eq!(
            selection.ordered_diagnostic_refs(),
            &[DiagnosticRef::new(hash(44))]
        );
    }

    #[test]
    fn rejected_candidates_use_diagnostic_order_not_arrival() {
        let kernel_rejected = candidate(
            "kernel-rejected",
            PolicyDecision {
                class: CandidatePolicyClass::KernelRejected,
                can_schedule_kernel_check: false,
                diagnostic: None,
                kernel_rejections: Vec::new(),
                external_admission: None,
            },
        )
        .with_evidence_hash(hash(9));
        let policy_rejected =
            candidate("policy-rejected", rejected_policy_decision()).with_evidence_hash(hash(1));

        let left = select_winner(
            &base_set(VerifierPolicy::release())
                .with_candidates([policy_rejected.clone(), kernel_rejected.clone()]),
        );
        let right = select_winner(
            &base_set(VerifierPolicy::release())
                .with_candidates([kernel_rejected, policy_rejected]),
        );

        assert_eq!(left.selected_class(), ProofWinnerClass::Rejected);
        assert_eq!(left.selected_candidate_id(), right.selected_candidate_id());
        assert_eq!(
            left.selected_candidate_id().map(CandidateSourceId::as_str),
            Some("kernel-rejected")
        );
    }

    #[test]
    fn rejected_order_covers_source_category_hash_and_id_ties() {
        let evidence_rejected = candidate(
            "evidence-rejected",
            rejected_policy_decision_with_reason(PolicyReasonCode::KernelEvidenceFormatDisabled),
        )
        .with_evidence_hash(hash(70));
        let policy_rejected =
            candidate("policy-rejected", rejected_policy_decision()).with_evidence_hash(hash(69));
        let invalid_rejected = candidate(
            "invalid-rejected",
            PolicyDecision {
                class: CandidatePolicyClass::KernelCheckable,
                can_schedule_kernel_check: true,
                diagnostic: None,
                kernel_rejections: Vec::new(),
                external_admission: None,
            },
        )
        .with_evidence_hash(hash(68));

        assert_selected(
            &base_set(VerifierPolicy::release())
                .with_candidates([policy_rejected.clone(), evidence_rejected.clone()]),
            ProofWinnerClass::Rejected,
            "evidence-rejected",
        );
        assert_selected(
            &base_set(VerifierPolicy::release())
                .with_candidates([invalid_rejected.clone(), policy_rejected]),
            ProofWinnerClass::Rejected,
            "policy-rejected",
        );

        let certificate_rejected = candidate(
            "certificate-rejected",
            kernel_rejected_decision_with_record(rejection_record(
                RejectionCategory::CertificateRejection,
                RejectionDetail::MalformedCertificate,
            )),
        );
        let kernel_rejected = candidate(
            "kernel-rejected",
            kernel_rejected_decision_with_record(rejection_record(
                RejectionCategory::KernelRejection,
                RejectionDetail::InvalidSubstitution,
            )),
        );
        assert_selected(
            &base_set(VerifierPolicy::release())
                .with_candidates([kernel_rejected, certificate_rejected]),
            ProofWinnerClass::Rejected,
            "certificate-rejected",
        );

        let missing_hash = candidate("missing-hash", rejected_policy_decision());
        let present_hash =
            candidate("present-hash", rejected_policy_decision()).with_evidence_hash(hash(1));
        assert_selected(
            &base_set(VerifierPolicy::release()).with_candidates([missing_hash, present_hash]),
            ProofWinnerClass::Rejected,
            "present-hash",
        );

        let source_a =
            candidate("source-a", rejected_policy_decision()).with_evidence_hash(hash(2));
        let source_b =
            candidate("source-b", rejected_policy_decision()).with_evidence_hash(hash(2));
        assert_selected(
            &base_set(VerifierPolicy::release()).with_candidates([source_b, source_a]),
            ProofWinnerClass::Rejected,
            "source-a",
        );
    }

    #[test]
    fn trusted_classes_require_kernel_derived_marker() {
        let spoofed = candidate(
            "spoofed",
            PolicyDecision {
                class: CandidatePolicyClass::KernelVerified,
                can_schedule_kernel_check: false,
                diagnostic: None,
                kernel_rejections: Vec::new(),
                external_admission: None,
            },
        );

        let selection =
            select_winner(&base_set(VerifierPolicy::release()).with_candidates([spoofed]));

        assert_eq!(selection.selected_class(), ProofWinnerClass::Rejected);
        assert_eq!(
            selection
                .selected_candidate_id()
                .map(CandidateSourceId::as_str),
            Some("spoofed")
        );
        assert!(!selection.trusted_used_axioms_available());

        let mismatched_kernel_input = KernelPolicyInput::for_test(
            KernelCheckStatus::Accepted,
            KernelEvidenceOrigin::BuiltinDischarge,
            false,
            Some(hash(80)),
        );
        let mismatched = candidate(
            "mismatched",
            PolicyDecision {
                class: CandidatePolicyClass::DischargedBuiltin,
                can_schedule_kernel_check: false,
                diagnostic: None,
                kernel_rejections: Vec::new(),
                external_admission: None,
            },
        )
        .with_evidence_hash(hash(81))
        .with_trusted_kernel_evidence(
            TrustedKernelEvidence::from_policy_input(&mismatched_kernel_input)
                .expect("accepted input with evidence hash"),
        );

        let mismatched_selection =
            select_winner(&base_set(VerifierPolicy::release()).with_candidates([mismatched]));
        assert_eq!(
            mismatched_selection.selected_class(),
            ProofWinnerClass::Rejected
        );
        assert!(!mismatched_selection.trusted_used_axioms_available());
    }

    #[test]
    fn discharged_builtin_trust_and_witness_gap_are_explicit() {
        let builtin = trusted_candidate(
            "builtin",
            CandidatePolicyClass::DischargedBuiltin,
            KernelEvidenceOrigin::BuiltinDischarge,
            hash(90),
        )
        .with_deterministic_discharge_hash(hash(91))
        .with_selected_proof_witness_hash(hash(92));

        let selection =
            select_winner(&base_set(VerifierPolicy::release()).with_candidates([builtin]));

        assert_eq!(
            selection.selected_class(),
            ProofWinnerClass::DischargedBuiltin
        );
        assert!(selection.trusted_used_axioms_available());
        assert_eq!(
            selection.reuse_metadata().selected_evidence_hash(),
            Some(hash(90))
        );
        assert_eq!(
            selection.reuse_metadata().deterministic_discharge_hash(),
            Some(hash(91))
        );
        assert_eq!(
            selection.reuse_metadata().selected_proof_witness_hash(),
            None
        );
        assert_eq!(
            selection.reuse_metadata().proof_witness_publication(),
            ProofWitnessPublication::ExternalDependencyGap
        );
    }

    #[test]
    fn non_trusted_winners_never_report_trusted_used_axioms() {
        let external_policy = VerifierPolicy::interactive()
            .with_external_evidence(ExternalEvidenceMode::PermitNonTrustedWinner);
        let external_evaluator = ProofPolicyEvaluator::new(external_policy.clone());
        let external = candidate(
            "external",
            external_evaluator.evaluate_candidate(&PolicyCandidate::ExternallyAttested),
        )
        .with_evidence_hash(hash(101));
        let external_selection =
            select_winner(&base_set(external_policy).with_candidates([external]));
        assert_eq!(
            external_selection.selected_class(),
            ProofWinnerClass::PolicyPermittedExternal
        );
        assert!(!external_selection.trusted_used_axioms_available());

        let assumed_policy = VerifierPolicy::development()
            .with_external_evidence(ExternalEvidenceMode::Reject)
            .with_policy_assumption(PolicyAssumptionMode::Record);
        let assumed_evaluator = ProofPolicyEvaluator::new(assumed_policy.clone());
        let assumed = candidate(
            "assumed",
            assumed_evaluator.evaluate_candidate(&PolicyCandidate::PolicyAssumption),
        )
        .with_evidence_hash(hash(102));
        let assumed_selection = select_winner(&base_set(assumed_policy).with_candidates([assumed]));
        assert_eq!(
            assumed_selection.selected_class(),
            ProofWinnerClass::PolicyAssumed
        );
        assert!(!assumed_selection.trusted_used_axioms_available());

        let open_policy = VerifierPolicy::interactive()
            .with_external_evidence(ExternalEvidenceMode::Reject)
            .with_policy_assumption(PolicyAssumptionMode::Reject);
        let open_evaluator = ProofPolicyEvaluator::new(open_policy.clone());
        let open = candidate(
            "open",
            open_evaluator.evaluate_candidate(&PolicyCandidate::OpenObligation),
        )
        .with_evidence_hash(hash(103));
        let open_selection = select_winner(&base_set(open_policy).with_candidates([open]));
        assert_eq!(
            open_selection.selected_class(),
            ProofWinnerClass::PolicyOpen
        );
        assert!(!open_selection.trusted_used_axioms_available());
    }

    #[test]
    fn forged_external_admission_cannot_override_active_policy_mode() {
        let forged_external = candidate(
            "forged-external",
            PolicyDecision {
                class: CandidatePolicyClass::ExternallyAttested,
                can_schedule_kernel_check: false,
                diagnostic: None,
                kernel_rejections: Vec::new(),
                external_admission: Some(crate::policy::ExternalEvidenceAdmission::new(
                    true,
                    true,
                    ExternalEvidencePublicationStatus::ExternallyAttestedPolicyPermitted,
                    None,
                )),
            },
        )
        .with_evidence_hash(hash(110));

        let selection = select_winner(
            &base_set(VerifierPolicy::development()).with_candidates([forged_external]),
        );

        assert_eq!(
            selection.selected_class(),
            ProofWinnerClass::NoSelectableEvidence
        );
        assert!(selection.selected_candidate_id().is_none());
    }

    #[test]
    fn artifact_merge_emits_canonical_vc_order() {
        let merged = merge_artifact_proof_selections(
            [
                VcProofSelection::new(VcId::new(2), selection_for_open("vc2")),
                VcProofSelection::new(VcId::new(0), selection_for_open("vc0")),
            ],
            [VcProofSelection::new(
                VcId::new(1),
                selection_for_builtin("vc1"),
            )],
        )
        .expect("merge succeeds");

        assert_eq!(
            merged
                .iter()
                .map(ArtifactProofSelection::vc)
                .collect::<Vec<_>>(),
            vec![VcId::new(0), VcId::new(1), VcId::new(2)]
        );
    }

    #[test]
    fn artifact_merge_preserves_trusted_class_precedence() {
        let builtin_over_external = merge_artifact_proof_selections(
            [VcProofSelection::new(
                VcId::new(0),
                selection_for_external("external"),
            )],
            [VcProofSelection::new(
                VcId::new(0),
                selection_for_builtin("builtin"),
            )],
        )
        .expect("merge succeeds");
        assert_eq!(
            builtin_over_external[0].source(),
            ProofSelectionSource::BuiltinDischarge
        );
        assert_eq!(
            builtin_over_external[0].selected_class(),
            ProofWinnerClass::DischargedBuiltin
        );

        let kernel_over_builtin = merge_artifact_proof_selections(
            [VcProofSelection::new(
                VcId::new(0),
                selection_for_kernel("kernel"),
            )],
            [VcProofSelection::new(
                VcId::new(0),
                selection_for_builtin("builtin"),
            )],
        )
        .expect("merge succeeds");
        assert_eq!(
            kernel_over_builtin[0].source(),
            ProofSelectionSource::Portfolio
        );
        assert_eq!(
            kernel_over_builtin[0].selected_class(),
            ProofWinnerClass::KernelVerified
        );
    }

    #[test]
    fn artifact_merge_uses_tie_break_hash_before_source_rank() {
        let first = selection_for_rejected("same-class-a");
        let second = selection_for_rejected("same-class-b");
        assert_ne!(tie_hash_bytes(&first), tie_hash_bytes(&second));

        let (lower_hash, higher_hash) = if tie_hash_bytes(&first) < tie_hash_bytes(&second) {
            (first, second)
        } else {
            (second, first)
        };
        let expected_candidate_id = lower_hash.selected_candidate_id().cloned();

        let merged = merge_artifact_proof_selections(
            [VcProofSelection::new(VcId::new(0), higher_hash)],
            [VcProofSelection::new(VcId::new(0), lower_hash)],
        )
        .expect("merge succeeds");

        assert_eq!(merged[0].source(), ProofSelectionSource::BuiltinDischarge);
        assert_eq!(merged[0].selected_class(), ProofWinnerClass::Rejected);
        assert_eq!(
            merged[0].selection().selected_candidate_id(),
            expected_candidate_id.as_ref()
        );
    }

    #[test]
    fn artifact_merge_uses_source_rank_for_equal_tie_break_hash() {
        let shared = selection_for_rejected("same-class-shared");

        let merged = merge_artifact_proof_selections(
            [VcProofSelection::new(VcId::new(0), shared.clone())],
            [VcProofSelection::new(VcId::new(0), shared)],
        )
        .expect("merge succeeds");

        assert_eq!(merged[0].source(), ProofSelectionSource::Portfolio);
        assert_eq!(merged[0].selected_class(), ProofWinnerClass::Rejected);
    }

    #[test]
    fn artifact_merge_preserves_non_trusted_outcomes() {
        let merged = merge_artifact_proof_selections(
            [
                VcProofSelection::new(VcId::new(0), selection_for_external("external")),
                VcProofSelection::new(VcId::new(1), selection_for_assumed("assumed")),
                VcProofSelection::new(VcId::new(2), selection_for_open("open")),
                VcProofSelection::new(VcId::new(3), selection_for_rejected("rejected")),
                VcProofSelection::new(VcId::new(4), selection_for_no_selectable()),
            ],
            [],
        )
        .expect("merge succeeds");

        assert_eq!(
            merged
                .iter()
                .map(ArtifactProofSelection::selected_class)
                .collect::<Vec<_>>(),
            vec![
                ProofWinnerClass::PolicyPermittedExternal,
                ProofWinnerClass::PolicyAssumed,
                ProofWinnerClass::PolicyOpen,
                ProofWinnerClass::Rejected,
                ProofWinnerClass::NoSelectableEvidence,
            ]
        );
        assert!(
            merged
                .iter()
                .all(|selection| !selection.selection().trusted_used_axioms_available())
        );
    }

    #[test]
    fn artifact_merge_rejects_duplicate_source_for_one_vc() {
        let portfolio_error = merge_artifact_proof_selections(
            [
                VcProofSelection::new(VcId::new(0), selection_for_open("left")),
                VcProofSelection::new(VcId::new(0), selection_for_open("right")),
            ],
            [],
        )
        .expect_err("duplicate portfolio selection is rejected");

        assert_eq!(
            portfolio_error,
            ArtifactProofSelectionError::DuplicateSelection {
                vc: VcId::new(0),
                source: ProofSelectionSource::Portfolio,
            }
        );

        let builtin_error = merge_artifact_proof_selections(
            [],
            [
                VcProofSelection::new(VcId::new(0), selection_for_builtin("left")),
                VcProofSelection::new(VcId::new(0), selection_for_builtin("right")),
            ],
        )
        .expect_err("duplicate built-in discharge selection is rejected");

        assert_eq!(
            builtin_error,
            ArtifactProofSelectionError::DuplicateSelection {
                vc: VcId::new(0),
                source: ProofSelectionSource::BuiltinDischarge,
            }
        );
    }

    #[test]
    fn artifact_merge_rejects_invalid_source_class_pairs() {
        let portfolio_error = merge_artifact_proof_selections(
            [VcProofSelection::new(
                VcId::new(0),
                selection_for_builtin("builtin"),
            )],
            [],
        )
        .expect_err("portfolio cannot publish built-in discharge selections");

        assert_eq!(
            portfolio_error,
            ArtifactProofSelectionError::InvalidSourceClass {
                vc: VcId::new(0),
                source: ProofSelectionSource::Portfolio,
                selected_class: ProofWinnerClass::DischargedBuiltin,
            }
        );

        let builtin_error = merge_artifact_proof_selections(
            [],
            [VcProofSelection::new(
                VcId::new(0),
                selection_for_external("external"),
            )],
        )
        .expect_err("built-in discharge source cannot publish external selections");

        assert_eq!(
            builtin_error,
            ArtifactProofSelectionError::InvalidSourceClass {
                vc: VcId::new(0),
                source: ProofSelectionSource::BuiltinDischarge,
                selected_class: ProofWinnerClass::PolicyPermittedExternal,
            }
        );
    }

    fn base_set(policy: VerifierPolicy) -> ProofEvidenceSet {
        ProofEvidenceSet::new(b"obligation".to_vec(), hash(100), policy)
    }

    fn selection_for_kernel(id: &str) -> ProofSelection {
        select_winner(
            &base_set(VerifierPolicy::release()).with_candidates([trusted_candidate(
                id,
                CandidatePolicyClass::KernelVerified,
                KernelEvidenceOrigin::AtpFormulaSubstitution,
                hash(120),
            )]),
        )
    }

    fn selection_for_builtin(id: &str) -> ProofSelection {
        select_winner(
            &base_set(VerifierPolicy::release()).with_candidates([trusted_candidate(
                id,
                CandidatePolicyClass::DischargedBuiltin,
                KernelEvidenceOrigin::BuiltinDischarge,
                hash(121),
            )
            .with_deterministic_discharge_hash(hash(122))]),
        )
    }

    fn selection_for_external(id: &str) -> ProofSelection {
        let policy = VerifierPolicy::interactive()
            .with_external_evidence(ExternalEvidenceMode::PermitNonTrustedWinner);
        let evaluator = ProofPolicyEvaluator::new(policy.clone());
        select_winner(
            &base_set(policy).with_candidates([candidate(
                id,
                evaluator.evaluate_candidate(&PolicyCandidate::ExternallyAttested),
            )
            .with_evidence_hash(hash(123))]),
        )
    }

    fn selection_for_assumed(id: &str) -> ProofSelection {
        let policy = VerifierPolicy::development()
            .with_external_evidence(ExternalEvidenceMode::Reject)
            .with_policy_assumption(PolicyAssumptionMode::Record);
        let evaluator = ProofPolicyEvaluator::new(policy.clone());
        select_winner(
            &base_set(policy).with_candidates([candidate(
                id,
                evaluator.evaluate_candidate(&PolicyCandidate::PolicyAssumption),
            )
            .with_evidence_hash(hash(125))]),
        )
    }

    fn selection_for_open(id: &str) -> ProofSelection {
        let policy = VerifierPolicy::interactive()
            .with_external_evidence(ExternalEvidenceMode::Reject)
            .with_policy_assumption(PolicyAssumptionMode::Reject);
        let evaluator = ProofPolicyEvaluator::new(policy.clone());
        select_winner(
            &base_set(policy).with_candidates([candidate(
                id,
                evaluator.evaluate_candidate(&PolicyCandidate::OpenObligation),
            )
            .with_evidence_hash(hash(124))]),
        )
    }

    fn selection_for_rejected(id: &str) -> ProofSelection {
        select_winner(
            &base_set(VerifierPolicy::release())
                .with_candidates([candidate(id, rejected_policy_decision())]),
        )
    }

    fn selection_for_no_selectable() -> ProofSelection {
        select_winner(&base_set(VerifierPolicy::development()))
    }

    fn tie_hash_bytes(selection: &ProofSelection) -> [u8; Hash::BYTE_LEN] {
        *selection.reuse_metadata().tie_break_key_hash().as_bytes()
    }

    fn candidate(id: &str, decision: PolicyDecision) -> ProofEvidenceCandidate {
        ProofEvidenceCandidate::new(CandidateSourceId::new(id).expect("stable id"), decision)
    }

    fn trusted_candidate(
        id: &str,
        class: CandidatePolicyClass,
        origin: KernelEvidenceOrigin,
        evidence_hash: Hash,
    ) -> ProofEvidenceCandidate {
        let kernel_input = KernelPolicyInput::for_test(
            KernelCheckStatus::Accepted,
            origin,
            false,
            Some(evidence_hash),
        );
        let candidate = ProofEvidenceCandidate::from_trusted_kernel_input(
            CandidateSourceId::new(id).expect("stable id"),
            &kernel_input,
        )
        .expect("accepted kernel input is trusted evidence");
        assert_eq!(candidate.decision().class, class);
        candidate
    }

    fn rejected_policy_decision() -> PolicyDecision {
        rejected_policy_decision_with_reason(PolicyReasonCode::OpenObligationRejected)
    }

    fn rejected_policy_decision_with_reason(reason: PolicyReasonCode) -> PolicyDecision {
        PolicyDecision {
            class: CandidatePolicyClass::RejectedByPolicy,
            can_schedule_kernel_check: false,
            diagnostic: Some(PolicyDiagnostic::new(
                PolicyDiagnosticCategory::PolicyRejection,
                reason,
            )),
            kernel_rejections: Vec::new(),
            external_admission: None,
        }
    }

    fn kernel_rejected_decision_with_record(record: RejectionRecord) -> PolicyDecision {
        PolicyDecision {
            class: CandidatePolicyClass::KernelRejected,
            can_schedule_kernel_check: false,
            diagnostic: None,
            kernel_rejections: vec![record],
            external_admission: None,
        }
    }

    fn rejection_record(category: RejectionCategory, detail: RejectionDetail) -> RejectionRecord {
        RejectionRecord::new(
            TargetVcFingerprint::new(1, vec![1, 2, 3]),
            category,
            detail,
            RejectionLocation::new(),
        )
        .expect("valid rejection record")
    }

    fn assert_selected(
        evidence_set: &ProofEvidenceSet,
        expected_class: ProofWinnerClass,
        expected_id: &str,
    ) {
        let selection = select_winner(evidence_set);
        assert_eq!(selection.selected_class(), expected_class);
        assert_eq!(
            selection
                .selected_candidate_id()
                .map(CandidateSourceId::as_str),
            Some(expected_id)
        );
    }

    fn hash(seed: u8) -> Hash {
        Hash::from_bytes([seed; Hash::BYTE_LEN])
    }
}
