//! Verifier policy classification and fingerprinting.
//!
//! This module classifies normalized proof-policy inputs. It does not run
//! kernel checks, call ATP backends, search for proofs, or turn policy evidence
//! into trusted acceptance.

use std::collections::BTreeSet;

use mizar_kernel::{
    checker::{KernelCheckResult, KernelCheckStatus, KernelEvidenceCheckKind},
    rejection::RejectionRecord,
};
use mizar_session::Hash;

const POLICY_FINGERPRINT_DOMAIN: &str = "mizar-proof-policy-fingerprint-v1";

/// Current policy schema version.
pub const POLICY_SCHEMA_VERSION: u16 = 1;

/// Current checker-schema version expected by the default policies.
pub const DEFAULT_CHECKER_SCHEMA_VERSION: u16 = 1;

/// Active verifier policy used above evidence production and below publication.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifierPolicy {
    schema_version: u16,
    profile_id: String,
    build_mode: BuildMode,
    require_kernel_certificates: bool,
    external_evidence: ExternalEvidenceMode,
    open_obligation: OpenObligationMode,
    policy_assumption: PolicyAssumptionMode,
    kernel_evidence_formats: BTreeSet<KernelEvidenceFormat>,
    checker_schema_version: u16,
}

impl VerifierPolicy {
    /// Strict release policy.
    #[must_use]
    pub fn release() -> Self {
        Self {
            schema_version: POLICY_SCHEMA_VERSION,
            profile_id: "release".to_owned(),
            build_mode: BuildMode::Release,
            require_kernel_certificates: true,
            external_evidence: ExternalEvidenceMode::Reject,
            open_obligation: OpenObligationMode::Reject,
            policy_assumption: PolicyAssumptionMode::Reject,
            kernel_evidence_formats: default_kernel_evidence_formats(),
            checker_schema_version: DEFAULT_CHECKER_SCHEMA_VERSION,
        }
    }

    /// Development policy that may record non-trusted policy evidence.
    #[must_use]
    pub fn development() -> Self {
        Self {
            profile_id: "development".to_owned(),
            build_mode: BuildMode::Development,
            require_kernel_certificates: false,
            external_evidence: ExternalEvidenceMode::RecordDevelopment,
            open_obligation: OpenObligationMode::RecordDiagnostic,
            policy_assumption: PolicyAssumptionMode::Record,
            ..Self::release()
        }
    }

    /// Interactive policy for editor diagnostics and LSP feedback.
    #[must_use]
    pub fn interactive() -> Self {
        Self {
            profile_id: "interactive".to_owned(),
            build_mode: BuildMode::Interactive,
            require_kernel_certificates: false,
            external_evidence: ExternalEvidenceMode::RecordDevelopment,
            open_obligation: OpenObligationMode::AllowPolicyOpen,
            policy_assumption: PolicyAssumptionMode::Record,
            ..Self::release()
        }
    }

    #[must_use]
    pub const fn schema_version(&self) -> u16 {
        self.schema_version
    }

    #[must_use]
    pub fn profile_id(&self) -> &str {
        &self.profile_id
    }

    #[must_use]
    pub const fn build_mode(&self) -> BuildMode {
        self.build_mode
    }

    #[must_use]
    pub const fn require_kernel_certificates(&self) -> bool {
        self.require_kernel_certificates
    }

    #[must_use]
    pub const fn external_evidence(&self) -> ExternalEvidenceMode {
        self.external_evidence
    }

    #[must_use]
    pub const fn open_obligation(&self) -> OpenObligationMode {
        self.open_obligation
    }

    #[must_use]
    pub const fn policy_assumption(&self) -> PolicyAssumptionMode {
        self.policy_assumption
    }

    #[must_use]
    pub fn kernel_evidence_formats(&self) -> &BTreeSet<KernelEvidenceFormat> {
        &self.kernel_evidence_formats
    }

    #[must_use]
    pub const fn checker_schema_version(&self) -> u16 {
        self.checker_schema_version
    }

    #[must_use]
    pub fn with_schema_version(mut self, schema_version: u16) -> Self {
        self.schema_version = schema_version;
        self
    }

    #[must_use]
    pub fn with_profile_id(mut self, profile_id: impl Into<String>) -> Self {
        self.profile_id = profile_id.into();
        self
    }

    #[must_use]
    pub const fn with_build_mode(mut self, build_mode: BuildMode) -> Self {
        self.build_mode = build_mode;
        self
    }

    #[must_use]
    pub const fn with_require_kernel_certificates(
        mut self,
        require_kernel_certificates: bool,
    ) -> Self {
        self.require_kernel_certificates = require_kernel_certificates;
        self
    }

    #[must_use]
    pub const fn with_external_evidence(mut self, external_evidence: ExternalEvidenceMode) -> Self {
        self.external_evidence = external_evidence;
        self
    }

    #[must_use]
    pub const fn with_open_obligation(mut self, open_obligation: OpenObligationMode) -> Self {
        self.open_obligation = open_obligation;
        self
    }

    #[must_use]
    pub const fn with_policy_assumption(mut self, policy_assumption: PolicyAssumptionMode) -> Self {
        self.policy_assumption = policy_assumption;
        self
    }

    #[must_use]
    pub fn with_kernel_evidence_formats(
        mut self,
        formats: impl IntoIterator<Item = KernelEvidenceFormat>,
    ) -> Self {
        self.kernel_evidence_formats = formats.into_iter().collect();
        self
    }

    #[must_use]
    pub const fn with_checker_schema_version(mut self, checker_schema_version: u16) -> Self {
        self.checker_schema_version = checker_schema_version;
        self
    }

    #[must_use]
    pub fn policy_fingerprint(&self) -> PolicyFingerprint {
        let mut hash = StableHasher::new(POLICY_FINGERPRINT_DOMAIN);
        hash.field_u16("schema_version", self.schema_version);
        hash.field_str("profile_id", &self.profile_id);
        hash.field_str("build_mode", self.build_mode.as_str());
        hash.field_bool(
            "require_kernel_certificates",
            self.require_kernel_certificates,
        );
        hash.field_str("external_evidence", self.external_evidence.as_str());
        hash.field_str("open_obligation", self.open_obligation.as_str());
        hash.field_str("policy_assumption", self.policy_assumption.as_str());
        hash.field_u64(
            "kernel_evidence_format_count",
            self.kernel_evidence_formats.len() as u64,
        );
        for format in &self.kernel_evidence_formats {
            hash.field_str("kernel_evidence_format", format.as_str());
        }
        hash.field_u16("checker_schema_version", self.checker_schema_version);
        PolicyFingerprint(hash.finalize())
    }

    fn supports_kernel_format(&self, format: KernelEvidenceFormat) -> bool {
        self.kernel_evidence_formats.contains(&format)
    }
}

impl Default for VerifierPolicy {
    fn default() -> Self {
        Self::release()
    }
}

/// Stateless evaluator for a single active verifier policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProofPolicyEvaluator {
    policy: VerifierPolicy,
}

impl ProofPolicyEvaluator {
    #[must_use]
    pub const fn new(policy: VerifierPolicy) -> Self {
        Self { policy }
    }

    #[must_use]
    pub const fn policy(&self) -> &VerifierPolicy {
        &self.policy
    }

    #[must_use]
    pub fn policy_fingerprint(&self) -> PolicyFingerprint {
        self.policy.policy_fingerprint()
    }

    #[must_use]
    pub fn candidate_class(&self, candidate: &PolicyCandidate) -> CandidatePolicyClass {
        self.evaluate_candidate(candidate).class
    }

    #[must_use]
    pub fn can_schedule_kernel_check(&self, candidate: &PolicyCandidate) -> bool {
        match candidate {
            PolicyCandidate::UncheckedFormulaSubstitution {
                encoded_problem_matches,
            } => {
                *encoded_problem_matches
                    && self
                        .policy
                        .supports_kernel_format(KernelEvidenceFormat::FormulaSubstitution)
            }
            PolicyCandidate::UncheckedBuiltinDischarge {
                has_stable_kernel_representation,
            }
            | PolicyCandidate::KernelPrimitive {
                allowed_by_policy: has_stable_kernel_representation,
            } => {
                *has_stable_kernel_representation
                    && self
                        .policy
                        .supports_kernel_format(KernelEvidenceFormat::BuiltinKernelEvidence)
            }
            PolicyCandidate::KernelResult(_)
            | PolicyCandidate::ExternallyAttested
            | PolicyCandidate::OpenObligation
            | PolicyCandidate::PolicyAssumption
            | PolicyCandidate::BackendDiagnostic
            | PolicyCandidate::BackendProofPayload(_)
            | PolicyCandidate::BackendReportedUsedAxioms
            | PolicyCandidate::CacheRecord
            | PolicyCandidate::Counterexample
            | PolicyCandidate::TimingRecord
            | PolicyCandidate::UnsupportedProofPayload
            | PolicyCandidate::LegacyReplay => false,
        }
    }

    #[must_use]
    pub fn best_possible_early_stop_class(
        &self,
        candidate: &PolicyCandidate,
    ) -> Option<PortfolioEarlyStopClass> {
        match candidate {
            PolicyCandidate::KernelResult(input)
                if input.status() == KernelCheckStatus::Accepted
                    && input.is_proof_obligation()
                    && !input.policy_taint() =>
            {
                Some(match input.origin() {
                    KernelEvidenceOrigin::AtpFormulaSubstitution => {
                        PortfolioEarlyStopClass::KernelVerified
                    }
                    KernelEvidenceOrigin::BuiltinDischarge
                    | KernelEvidenceOrigin::KernelPrimitive => {
                        PortfolioEarlyStopClass::DischargedBuiltin
                    }
                })
            }
            PolicyCandidate::KernelResult(input)
                if input.status() == KernelCheckStatus::Accepted
                    && input.is_proof_obligation()
                    && input.policy_taint() =>
            {
                self.external_evidence_admission()
                    .may_win_selection()
                    .then_some(PortfolioEarlyStopClass::PolicyPermittedExternal)
            }
            PolicyCandidate::UncheckedFormulaSubstitution { .. }
                if self.can_schedule_kernel_check(candidate) =>
            {
                Some(PortfolioEarlyStopClass::KernelVerified)
            }
            PolicyCandidate::UncheckedBuiltinDischarge { .. }
            | PolicyCandidate::KernelPrimitive { .. }
                if self.can_schedule_kernel_check(candidate) =>
            {
                Some(PortfolioEarlyStopClass::DischargedBuiltin)
            }
            PolicyCandidate::ExternallyAttested => self
                .external_evidence_admission()
                .may_win_selection()
                .then_some(PortfolioEarlyStopClass::PolicyPermittedExternal),
            PolicyCandidate::PolicyAssumption
                if !self.policy.require_kernel_certificates()
                    && self.policy.policy_assumption() == PolicyAssumptionMode::Record =>
            {
                Some(PortfolioEarlyStopClass::PolicyAssumed)
            }
            PolicyCandidate::OpenObligation
                if self.policy.open_obligation() == OpenObligationMode::AllowPolicyOpen =>
            {
                Some(PortfolioEarlyStopClass::PolicyOpen)
            }
            PolicyCandidate::KernelResult(_)
            | PolicyCandidate::UncheckedFormulaSubstitution { .. }
            | PolicyCandidate::UncheckedBuiltinDischarge { .. }
            | PolicyCandidate::KernelPrimitive { .. }
            | PolicyCandidate::OpenObligation
            | PolicyCandidate::PolicyAssumption
            | PolicyCandidate::BackendDiagnostic
            | PolicyCandidate::BackendProofPayload(_)
            | PolicyCandidate::BackendReportedUsedAxioms
            | PolicyCandidate::CacheRecord
            | PolicyCandidate::Counterexample
            | PolicyCandidate::TimingRecord
            | PolicyCandidate::UnsupportedProofPayload
            | PolicyCandidate::LegacyReplay => None,
        }
    }

    #[must_use]
    pub fn portfolio_early_stop_decision(
        &self,
        input: &PortfolioEarlyStopInput,
    ) -> PortfolioEarlyStopDecision {
        let Some(observed_best_class) = input.observed_best_class() else {
            return PortfolioEarlyStopDecision::do_not_stop(
                PortfolioEarlyStopReason::NoObservedCandidate,
                None,
                None,
            );
        };
        if !self.early_stop_class_is_selectable(observed_best_class) {
            return PortfolioEarlyStopDecision::do_not_stop(
                PortfolioEarlyStopReason::ObservedClassNotSelectable,
                Some(observed_best_class),
                None,
            );
        }

        let mut blocking_pending_class = None;
        for pending_class in input.pending_best_possible_classes() {
            if !self.early_stop_class_is_selectable(*pending_class) {
                continue;
            }
            if pending_class.rank() <= observed_best_class.rank()
                && blocking_pending_class.is_none_or(|current: PortfolioEarlyStopClass| {
                    pending_class.rank() < current.rank()
                })
            {
                blocking_pending_class = Some(*pending_class);
            }
        }

        if let Some(blocking_pending_class) = blocking_pending_class {
            let reason = if blocking_pending_class.rank() < observed_best_class.rank() {
                PortfolioEarlyStopReason::BlockedByHigherClass
            } else {
                PortfolioEarlyStopReason::BlockedByEqualClass
            };
            return PortfolioEarlyStopDecision::do_not_stop(
                reason,
                Some(observed_best_class),
                Some(blocking_pending_class),
            );
        }

        PortfolioEarlyStopDecision::stop_allowed(observed_best_class)
    }

    #[must_use]
    pub fn evaluate_candidate(&self, candidate: &PolicyCandidate) -> PolicyDecision {
        let can_schedule_kernel_check = self.can_schedule_kernel_check(candidate);
        let (class, diagnostic, kernel_rejections, external_admission) = match candidate {
            PolicyCandidate::KernelResult(input) => match input.status {
                KernelCheckStatus::Accepted if !input.is_proof_obligation() => {
                    (CandidatePolicyClass::DiagnosticOnly, None, Vec::new(), None)
                }
                KernelCheckStatus::Accepted if input.policy_taint => {
                    self.externally_attested_decision()
                }
                KernelCheckStatus::Accepted => match input.origin {
                    KernelEvidenceOrigin::AtpFormulaSubstitution => {
                        (CandidatePolicyClass::KernelVerified, None, Vec::new(), None)
                    }
                    KernelEvidenceOrigin::BuiltinDischarge
                    | KernelEvidenceOrigin::KernelPrimitive => (
                        CandidatePolicyClass::DischargedBuiltin,
                        None,
                        Vec::new(),
                        None,
                    ),
                },
                KernelCheckStatus::Rejected => self.kernel_rejected_decision(input),
                _ => self.kernel_rejected_decision(input),
            },
            PolicyCandidate::UncheckedFormulaSubstitution {
                encoded_problem_matches,
            } => {
                if can_schedule_kernel_check {
                    (
                        CandidatePolicyClass::KernelCheckable,
                        None,
                        Vec::new(),
                        None,
                    )
                } else if *encoded_problem_matches {
                    self.rejection(PolicyReasonCode::KernelEvidenceFormatDisabled)
                } else {
                    self.rejection(PolicyReasonCode::KernelEvidenceTargetMismatch)
                }
            }
            PolicyCandidate::UncheckedBuiltinDischarge {
                has_stable_kernel_representation,
            } => {
                if can_schedule_kernel_check {
                    (
                        CandidatePolicyClass::KernelCheckable,
                        None,
                        Vec::new(),
                        None,
                    )
                } else if *has_stable_kernel_representation {
                    self.rejection(PolicyReasonCode::KernelEvidenceFormatDisabled)
                } else {
                    self.rejection(PolicyReasonCode::MissingBuiltinKernelRepresentation)
                }
            }
            PolicyCandidate::KernelPrimitive { allowed_by_policy } => {
                if can_schedule_kernel_check {
                    (
                        CandidatePolicyClass::KernelCheckable,
                        None,
                        Vec::new(),
                        None,
                    )
                } else if *allowed_by_policy {
                    self.rejection(PolicyReasonCode::KernelEvidenceFormatDisabled)
                } else {
                    self.rejection(PolicyReasonCode::KernelPrimitiveNotAllowed)
                }
            }
            PolicyCandidate::ExternallyAttested => self.externally_attested_decision(),
            PolicyCandidate::OpenObligation => match self.policy.open_obligation {
                OpenObligationMode::Reject => {
                    self.rejection(PolicyReasonCode::OpenObligationRejected)
                }
                OpenObligationMode::RecordDiagnostic | OpenObligationMode::AllowPolicyOpen => (
                    CandidatePolicyClass::OpenAllowed,
                    Some(PolicyDiagnostic::new(
                        PolicyDiagnosticCategory::PolicyOpen,
                        PolicyReasonCode::OpenObligationAllowed,
                    )),
                    Vec::new(),
                    None,
                ),
            },
            PolicyCandidate::PolicyAssumption => match self.policy.policy_assumption {
                PolicyAssumptionMode::Reject => {
                    self.rejection(PolicyReasonCode::PolicyAssumptionRejected)
                }
                PolicyAssumptionMode::Record => (
                    CandidatePolicyClass::AssumedByPolicy,
                    Some(PolicyDiagnostic::new(
                        PolicyDiagnosticCategory::PolicyOpen,
                        PolicyReasonCode::PolicyAssumptionRecorded,
                    )),
                    Vec::new(),
                    None,
                ),
            },
            PolicyCandidate::BackendDiagnostic
            | PolicyCandidate::BackendProofPayload(_)
            | PolicyCandidate::BackendReportedUsedAxioms
            | PolicyCandidate::CacheRecord
            | PolicyCandidate::Counterexample
            | PolicyCandidate::TimingRecord
            | PolicyCandidate::UnsupportedProofPayload => (
                CandidatePolicyClass::DiagnosticOnly,
                Some(PolicyDiagnostic::new(
                    PolicyDiagnosticCategory::DiagnosticOnly,
                    PolicyReasonCode::DiagnosticOnly,
                )),
                Vec::new(),
                None,
            ),
            PolicyCandidate::LegacyReplay => self.rejection(PolicyReasonCode::LegacyReplayRejected),
        };

        PolicyDecision {
            class,
            can_schedule_kernel_check,
            diagnostic,
            kernel_rejections,
            external_admission,
        }
    }

    #[must_use]
    pub fn external_evidence_admission(&self) -> ExternalEvidenceAdmission {
        match self.policy.external_evidence {
            ExternalEvidenceMode::Reject => ExternalEvidenceAdmission::new(
                false,
                false,
                ExternalEvidencePublicationStatus::RejectedByPolicy,
                Some(PolicyDiagnostic::new(
                    PolicyDiagnosticCategory::PolicyRejection,
                    PolicyReasonCode::ExternalEvidenceRejected,
                )),
            ),
            ExternalEvidenceMode::RecordDevelopment => self.external_recording_admission(false),
            ExternalEvidenceMode::PermitNonTrustedWinner => {
                if self.policy.require_kernel_certificates {
                    self.external_recording_admission(false)
                } else {
                    ExternalEvidenceAdmission::new(
                        true,
                        true,
                        ExternalEvidencePublicationStatus::ExternallyAttestedPolicyPermitted,
                        Some(PolicyDiagnostic::new(
                            PolicyDiagnosticCategory::PolicyOpen,
                            PolicyReasonCode::ExternalEvidencePolicyPermitted,
                        )),
                    )
                }
            }
        }
    }

    fn externally_attested_decision(
        &self,
    ) -> (
        CandidatePolicyClass,
        Option<PolicyDiagnostic>,
        Vec<RejectionRecord>,
        Option<ExternalEvidenceAdmission>,
    ) {
        let admission = self.external_evidence_admission();
        (
            admission.policy_class(),
            admission.diagnostic().cloned(),
            Vec::new(),
            Some(admission),
        )
    }

    fn external_recording_admission(&self, may_win_selection: bool) -> ExternalEvidenceAdmission {
        if self.policy.require_kernel_certificates {
            return match self.policy.build_mode {
                BuildMode::Interactive => ExternalEvidenceAdmission::new(
                    true,
                    false,
                    ExternalEvidencePublicationStatus::ExternallyAttestedOpenDiagnostic,
                    Some(PolicyDiagnostic::new(
                        PolicyDiagnosticCategory::PolicyOpen,
                        PolicyReasonCode::ExternalEvidenceRecorded,
                    )),
                ),
                BuildMode::Release | BuildMode::Development => ExternalEvidenceAdmission::new(
                    true,
                    false,
                    ExternalEvidencePublicationStatus::RejectedByPolicy,
                    Some(PolicyDiagnostic::new(
                        PolicyDiagnosticCategory::PolicyRejection,
                        PolicyReasonCode::ExternalEvidenceRequiresKernelCertificate,
                    )),
                ),
            };
        }

        match (may_win_selection, self.policy.build_mode) {
            (true, _) => ExternalEvidenceAdmission::new(
                true,
                true,
                ExternalEvidencePublicationStatus::ExternallyAttestedPolicyPermitted,
                Some(PolicyDiagnostic::new(
                    PolicyDiagnosticCategory::PolicyOpen,
                    PolicyReasonCode::ExternalEvidencePolicyPermitted,
                )),
            ),
            (false, BuildMode::Interactive) => ExternalEvidenceAdmission::new(
                true,
                false,
                ExternalEvidencePublicationStatus::ExternallyAttestedOpenDiagnostic,
                Some(PolicyDiagnostic::new(
                    PolicyDiagnosticCategory::PolicyOpen,
                    PolicyReasonCode::ExternalEvidenceRecorded,
                )),
            ),
            (false, BuildMode::Release | BuildMode::Development) => ExternalEvidenceAdmission::new(
                true,
                false,
                ExternalEvidencePublicationStatus::ExternallyAttestedDevelopment,
                Some(PolicyDiagnostic::new(
                    PolicyDiagnosticCategory::PolicyOpen,
                    PolicyReasonCode::ExternalEvidenceRecorded,
                )),
            ),
        }
    }

    fn kernel_rejected_decision(
        &self,
        input: &KernelPolicyInput,
    ) -> (
        CandidatePolicyClass,
        Option<PolicyDiagnostic>,
        Vec<RejectionRecord>,
        Option<ExternalEvidenceAdmission>,
    ) {
        (
            CandidatePolicyClass::KernelRejected,
            None,
            input.kernel_rejections.clone(),
            None,
        )
    }

    fn rejection(
        &self,
        reason: PolicyReasonCode,
    ) -> (
        CandidatePolicyClass,
        Option<PolicyDiagnostic>,
        Vec<RejectionRecord>,
        Option<ExternalEvidenceAdmission>,
    ) {
        (
            CandidatePolicyClass::RejectedByPolicy,
            Some(PolicyDiagnostic::new(
                PolicyDiagnosticCategory::PolicyRejection,
                reason,
            )),
            Vec::new(),
            None,
        )
    }

    fn early_stop_class_is_selectable(&self, class: PortfolioEarlyStopClass) -> bool {
        match class {
            PortfolioEarlyStopClass::KernelVerified
            | PortfolioEarlyStopClass::DischargedBuiltin => true,
            PortfolioEarlyStopClass::PolicyPermittedExternal => {
                self.external_evidence_admission().may_win_selection()
            }
            PortfolioEarlyStopClass::PolicyAssumed => {
                !self.policy.require_kernel_certificates()
                    && self.policy.policy_assumption() == PolicyAssumptionMode::Record
            }
            PortfolioEarlyStopClass::PolicyOpen => {
                self.policy.open_obligation() == OpenObligationMode::AllowPolicyOpen
            }
        }
    }
}

/// Stable hash over policy settings that affect proof policy behavior.
#[derive(Clone, Copy, Debug, Eq, PartialEq, std::hash::Hash)]
pub struct PolicyFingerprint(Hash);

impl PolicyFingerprint {
    #[must_use]
    pub const fn hash(self) -> Hash {
        self.0
    }

    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; Hash::BYTE_LEN] {
        self.0.as_bytes()
    }

    #[must_use]
    pub fn to_lower_hex(self) -> String {
        lower_hex(self.0.as_bytes())
    }
}

/// Build profile category.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, std::hash::Hash)]
#[non_exhaustive]
pub enum BuildMode {
    Release,
    Development,
    Interactive,
}

impl BuildMode {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Release => "release",
            Self::Development => "development",
            Self::Interactive => "interactive",
        }
    }
}

/// Policy for externally attested evidence records.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, std::hash::Hash)]
#[non_exhaustive]
pub enum ExternalEvidenceMode {
    Reject,
    RecordDevelopment,
    PermitNonTrustedWinner,
}

impl ExternalEvidenceMode {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Reject => "reject",
            Self::RecordDevelopment => "record-development",
            Self::PermitNonTrustedWinner => "permit-non-trusted-winner",
        }
    }
}

/// Policy for open obligations.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, std::hash::Hash)]
#[non_exhaustive]
pub enum OpenObligationMode {
    Reject,
    RecordDiagnostic,
    AllowPolicyOpen,
}

impl OpenObligationMode {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Reject => "reject",
            Self::RecordDiagnostic => "record-diagnostic",
            Self::AllowPolicyOpen => "allow-policy-open",
        }
    }
}

/// Policy for explicit assumptions.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, std::hash::Hash)]
#[non_exhaustive]
pub enum PolicyAssumptionMode {
    Reject,
    Record,
}

impl PolicyAssumptionMode {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Reject => "reject",
            Self::Record => "record",
        }
    }
}

/// Kernel-checkable evidence representations that policy may schedule.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, std::hash::Hash)]
#[non_exhaustive]
pub enum KernelEvidenceFormat {
    FormulaSubstitution,
    BuiltinKernelEvidence,
}

impl KernelEvidenceFormat {
    const fn as_str(self) -> &'static str {
        match self {
            Self::FormulaSubstitution => "formula-substitution",
            Self::BuiltinKernelEvidence => "builtin-kernel-evidence",
        }
    }
}

/// Policy-facing class used before deterministic winner selection.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, std::hash::Hash)]
#[non_exhaustive]
pub enum CandidatePolicyClass {
    KernelVerified,
    DischargedBuiltin,
    KernelRejected,
    KernelCheckable,
    ExternallyAttested,
    OpenAllowed,
    AssumedByPolicy,
    RejectedByPolicy,
    DiagnosticOnly,
}

/// Policy-normalized winner class used by ATP portfolio early-stop queries.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, std::hash::Hash)]
#[non_exhaustive]
pub enum PortfolioEarlyStopClass {
    KernelVerified,
    DischargedBuiltin,
    PolicyPermittedExternal,
    PolicyAssumed,
    PolicyOpen,
}

impl PortfolioEarlyStopClass {
    const fn rank(self) -> u8 {
        match self {
            Self::KernelVerified => 0,
            Self::DischargedBuiltin => 1,
            Self::PolicyPermittedExternal => 2,
            Self::PolicyAssumed => 3,
            Self::PolicyOpen => 4,
        }
    }
}

/// Stable reason for an ATP portfolio early-stop decision.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, std::hash::Hash)]
#[non_exhaustive]
pub enum PortfolioEarlyStopReason {
    NoObservedCandidate,
    ObservedClassNotSelectable,
    BlockedByHigherClass,
    BlockedByEqualClass,
    NoDisplacingPendingClass,
}

/// Class-level input for ATP portfolio early-stop finality.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PortfolioEarlyStopInput {
    observed_best_class: Option<PortfolioEarlyStopClass>,
    pending_best_possible_classes: Vec<PortfolioEarlyStopClass>,
}

impl PortfolioEarlyStopInput {
    #[must_use]
    pub fn new(
        observed_best_class: Option<PortfolioEarlyStopClass>,
        pending_best_possible_classes: impl IntoIterator<Item = PortfolioEarlyStopClass>,
    ) -> Self {
        Self {
            observed_best_class,
            pending_best_possible_classes: pending_best_possible_classes.into_iter().collect(),
        }
    }

    #[must_use]
    pub const fn observed_best_class(&self) -> Option<PortfolioEarlyStopClass> {
        self.observed_best_class
    }

    #[must_use]
    pub fn pending_best_possible_classes(&self) -> &[PortfolioEarlyStopClass] {
        &self.pending_best_possible_classes
    }
}

/// Policy decision for ATP portfolio early-stop finality.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PortfolioEarlyStopDecision {
    may_stop: bool,
    reason: PortfolioEarlyStopReason,
    observed_best_class: Option<PortfolioEarlyStopClass>,
    blocking_pending_class: Option<PortfolioEarlyStopClass>,
}

impl PortfolioEarlyStopDecision {
    const fn stop_allowed(observed_best_class: PortfolioEarlyStopClass) -> Self {
        Self {
            may_stop: true,
            reason: PortfolioEarlyStopReason::NoDisplacingPendingClass,
            observed_best_class: Some(observed_best_class),
            blocking_pending_class: None,
        }
    }

    const fn do_not_stop(
        reason: PortfolioEarlyStopReason,
        observed_best_class: Option<PortfolioEarlyStopClass>,
        blocking_pending_class: Option<PortfolioEarlyStopClass>,
    ) -> Self {
        Self {
            may_stop: false,
            reason,
            observed_best_class,
            blocking_pending_class,
        }
    }

    #[must_use]
    pub const fn may_stop(&self) -> bool {
        self.may_stop
    }

    #[must_use]
    pub const fn reason(&self) -> PortfolioEarlyStopReason {
        self.reason
    }

    #[must_use]
    pub const fn observed_best_class(&self) -> Option<PortfolioEarlyStopClass> {
        self.observed_best_class
    }

    #[must_use]
    pub const fn blocking_pending_class(&self) -> Option<PortfolioEarlyStopClass> {
        self.blocking_pending_class
    }
}

/// Normalized policy input for kernel results.
///
/// Public callers can construct this only from an actual kernel result plus an
/// explicit origin. This keeps trusted policy classes tied to
/// `KernelCheckResult` instead of to caller-synthesized status enums.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KernelPolicyInput {
    status: KernelCheckStatus,
    origin: KernelEvidenceOrigin,
    evidence_check_kind: Option<KernelEvidenceCheckKind>,
    policy_taint: bool,
    kernel_rejections: Vec<RejectionRecord>,
    accepted_evidence_hash: Option<Hash>,
}

impl KernelPolicyInput {
    #[must_use]
    pub fn from_kernel_result(result: &KernelCheckResult, origin: KernelEvidenceOrigin) -> Self {
        Self {
            status: result.status(),
            origin,
            evidence_check_kind: result.evidence_check_kind(),
            policy_taint: result.policy_taint(),
            kernel_rejections: result.rejections().to_vec(),
            accepted_evidence_hash: accepted_evidence_hash_from_kernel_result(result, origin),
        }
    }

    #[must_use]
    pub const fn status(&self) -> KernelCheckStatus {
        self.status
    }

    #[must_use]
    pub const fn origin(&self) -> KernelEvidenceOrigin {
        self.origin
    }

    #[must_use]
    pub const fn evidence_check_kind(&self) -> Option<KernelEvidenceCheckKind> {
        self.evidence_check_kind
    }

    #[must_use]
    pub const fn is_proof_obligation(&self) -> bool {
        matches!(
            self.evidence_check_kind,
            Some(KernelEvidenceCheckKind::ProofObligation)
        )
    }

    #[must_use]
    pub const fn policy_taint(&self) -> bool {
        self.policy_taint
    }

    #[must_use]
    pub fn kernel_rejections(&self) -> &[RejectionRecord] {
        &self.kernel_rejections
    }

    #[must_use]
    pub const fn accepted_evidence_hash(&self) -> Option<Hash> {
        self.accepted_evidence_hash
    }

    #[cfg(test)]
    pub(crate) fn for_test(
        status: KernelCheckStatus,
        origin: KernelEvidenceOrigin,
        policy_taint: bool,
        accepted_evidence_hash: Option<Hash>,
    ) -> Self {
        Self::for_test_with_check_kind(
            status,
            origin,
            Some(KernelEvidenceCheckKind::ProofObligation),
            policy_taint,
            accepted_evidence_hash,
        )
    }

    #[cfg(test)]
    pub(crate) fn for_test_with_check_kind(
        status: KernelCheckStatus,
        origin: KernelEvidenceOrigin,
        evidence_check_kind: Option<KernelEvidenceCheckKind>,
        policy_taint: bool,
        accepted_evidence_hash: Option<Hash>,
    ) -> Self {
        let accepted_evidence_hash = if status == KernelCheckStatus::Accepted
            && evidence_check_kind == Some(KernelEvidenceCheckKind::ProofObligation)
            && !policy_taint
        {
            accepted_evidence_hash
        } else {
            None
        };
        Self {
            status,
            origin,
            evidence_check_kind,
            policy_taint,
            kernel_rejections: Vec::new(),
            accepted_evidence_hash,
        }
    }
}

/// Explicit origin for a kernel result.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, std::hash::Hash)]
#[non_exhaustive]
pub enum KernelEvidenceOrigin {
    AtpFormulaSubstitution,
    BuiltinDischarge,
    KernelPrimitive,
}

impl KernelEvidenceOrigin {
    const fn as_str(self) -> &'static str {
        match self {
            Self::AtpFormulaSubstitution => "atp-formula-substitution",
            Self::BuiltinDischarge => "builtin-discharge",
            Self::KernelPrimitive => "kernel-primitive",
        }
    }
}

/// Normalized candidate accepted by the policy evaluator.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PolicyCandidate {
    KernelResult(KernelPolicyInput),
    UncheckedFormulaSubstitution {
        encoded_problem_matches: bool,
    },
    UncheckedBuiltinDischarge {
        has_stable_kernel_representation: bool,
    },
    KernelPrimitive {
        allowed_by_policy: bool,
    },
    ExternallyAttested,
    OpenObligation,
    PolicyAssumption,
    BackendDiagnostic,
    BackendProofPayload(BackendProofPayloadKind),
    BackendReportedUsedAxioms,
    CacheRecord,
    Counterexample,
    TimingRecord,
    UnsupportedProofPayload,
    LegacyReplay,
}

/// Backend-owned proof payloads that remain outside trusted acceptance.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, std::hash::Hash)]
#[non_exhaustive]
pub enum BackendProofPayloadKind {
    BackendProofMethod,
    ResolutionTrace,
    SmtProofObject,
    TstpTrace,
    UnsatCore,
}

/// Result of applying policy to one normalized candidate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyDecision {
    pub class: CandidatePolicyClass,
    pub can_schedule_kernel_check: bool,
    pub diagnostic: Option<PolicyDiagnostic>,
    pub kernel_rejections: Vec<RejectionRecord>,
    pub external_admission: Option<ExternalEvidenceAdmission>,
}

/// Admission result for externally attested evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExternalEvidenceAdmission {
    record_as_development_evidence: bool,
    may_win_selection: bool,
    publication_status: ExternalEvidencePublicationStatus,
    diagnostic: Option<PolicyDiagnostic>,
    trusted_used_axioms_allowed: bool,
}

impl ExternalEvidenceAdmission {
    #[must_use]
    pub const fn new(
        record_as_development_evidence: bool,
        may_win_selection: bool,
        publication_status: ExternalEvidencePublicationStatus,
        diagnostic: Option<PolicyDiagnostic>,
    ) -> Self {
        Self {
            record_as_development_evidence,
            may_win_selection,
            publication_status,
            diagnostic,
            trusted_used_axioms_allowed: false,
        }
    }

    #[must_use]
    pub const fn policy_class(&self) -> CandidatePolicyClass {
        match self.publication_status {
            ExternalEvidencePublicationStatus::RejectedByPolicy => {
                CandidatePolicyClass::RejectedByPolicy
            }
            ExternalEvidencePublicationStatus::ExternallyAttestedDevelopment
            | ExternalEvidencePublicationStatus::ExternallyAttestedOpenDiagnostic
            | ExternalEvidencePublicationStatus::ExternallyAttestedPolicyPermitted => {
                CandidatePolicyClass::ExternallyAttested
            }
        }
    }

    #[must_use]
    pub const fn record_as_development_evidence(&self) -> bool {
        self.record_as_development_evidence
    }

    #[must_use]
    pub const fn may_win_selection(&self) -> bool {
        self.may_win_selection
    }

    #[must_use]
    pub const fn publication_status(&self) -> ExternalEvidencePublicationStatus {
        self.publication_status
    }

    #[must_use]
    pub fn diagnostic(&self) -> Option<&PolicyDiagnostic> {
        self.diagnostic.as_ref()
    }

    #[must_use]
    pub const fn trusted_used_axioms_allowed(&self) -> bool {
        self.trusted_used_axioms_allowed
    }
}

/// Publication/status label for external evidence admission.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, std::hash::Hash)]
#[non_exhaustive]
pub enum ExternalEvidencePublicationStatus {
    RejectedByPolicy,
    ExternallyAttestedDevelopment,
    ExternallyAttestedOpenDiagnostic,
    ExternallyAttestedPolicyPermitted,
}

/// Stable policy diagnostic label.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyDiagnostic {
    pub category: PolicyDiagnosticCategory,
    pub reason: PolicyReasonCode,
}

impl PolicyDiagnostic {
    #[must_use]
    pub const fn new(category: PolicyDiagnosticCategory, reason: PolicyReasonCode) -> Self {
        Self { category, reason }
    }
}

/// Policy diagnostic category.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, std::hash::Hash)]
#[non_exhaustive]
pub enum PolicyDiagnosticCategory {
    PolicyRejection,
    PolicyOpen,
    DiagnosticOnly,
}

impl PolicyDiagnosticCategory {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PolicyRejection => "policy_rejection",
            Self::PolicyOpen => "policy_open",
            Self::DiagnosticOnly => "diagnostic_only",
        }
    }
}

/// Stable policy reason code.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, std::hash::Hash)]
#[non_exhaustive]
pub enum PolicyReasonCode {
    ExternalEvidenceRejected,
    ExternalEvidenceRequiresKernelCertificate,
    ExternalEvidenceRecorded,
    ExternalEvidencePolicyPermitted,
    KernelEvidenceTargetMismatch,
    KernelEvidenceFormatDisabled,
    MissingBuiltinKernelRepresentation,
    KernelPrimitiveNotAllowed,
    OpenObligationRejected,
    OpenObligationAllowed,
    PolicyAssumptionRejected,
    PolicyAssumptionRecorded,
    DiagnosticOnly,
    LegacyReplayRejected,
}

impl PolicyReasonCode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ExternalEvidenceRejected => "external_evidence_rejected",
            Self::ExternalEvidenceRequiresKernelCertificate => {
                "external_evidence_requires_kernel_certificate"
            }
            Self::ExternalEvidenceRecorded => "external_evidence_recorded",
            Self::ExternalEvidencePolicyPermitted => "external_evidence_policy_permitted",
            Self::KernelEvidenceTargetMismatch => "kernel_evidence_target_mismatch",
            Self::KernelEvidenceFormatDisabled => "kernel_evidence_format_disabled",
            Self::MissingBuiltinKernelRepresentation => "missing_builtin_kernel_representation",
            Self::KernelPrimitiveNotAllowed => "kernel_primitive_not_allowed",
            Self::OpenObligationRejected => "open_obligation_rejected",
            Self::OpenObligationAllowed => "open_obligation_allowed",
            Self::PolicyAssumptionRejected => "policy_assumption_rejected",
            Self::PolicyAssumptionRecorded => "policy_assumption_recorded",
            Self::DiagnosticOnly => "diagnostic_only",
            Self::LegacyReplayRejected => "legacy_replay_rejected",
        }
    }
}

fn default_kernel_evidence_formats() -> BTreeSet<KernelEvidenceFormat> {
    [
        KernelEvidenceFormat::FormulaSubstitution,
        KernelEvidenceFormat::BuiltinKernelEvidence,
    ]
    .into_iter()
    .collect()
}

fn accepted_evidence_hash_from_kernel_result(
    result: &KernelCheckResult,
    origin: KernelEvidenceOrigin,
) -> Option<Hash> {
    if result.status() != KernelCheckStatus::Accepted
        || result.policy_taint()
        || result.evidence_check_kind() != Some(KernelEvidenceCheckKind::ProofObligation)
    {
        return None;
    }

    let mut hash = StableHasher::new("mizar-proof-kernel-accepted-evidence-v1");
    hash.field_str("status", "accepted");
    hash.field_str("origin", origin.as_str());
    hash.field_str("evidence_check_kind", "proof-obligation");
    hash.field_bytes(
        "target_vc_fingerprint",
        &result.target_vc_fingerprint().sort_bytes(),
    );
    hash.field_u64(
        "checked_import_count",
        result.checked_imports().len() as u64,
    );
    hash.field_debug("checked_imports", result.checked_imports());
    hash.field_u64(
        "checked_substitution_count",
        result.checked_substitutions().len() as u64,
    );
    hash.field_debug("checked_substitutions", result.checked_substitutions());
    hash.field_u64(
        "checked_resolution_step_count",
        result.checked_resolution_steps().len() as u64,
    );
    hash.field_debug(
        "checked_resolution_steps",
        result.checked_resolution_steps(),
    );
    hash.field_u64(
        "checked_cluster_step_count",
        result.checked_cluster_steps().len() as u64,
    );
    hash.field_debug("checked_cluster_steps", result.checked_cluster_steps());
    hash.field_u64(
        "checked_reduction_step_count",
        result.checked_reduction_steps().len() as u64,
    );
    hash.field_debug("checked_reduction_steps", result.checked_reduction_steps());
    hash.field_u64(
        "checked_derived_fact_count",
        result.checked_derived_facts().len() as u64,
    );
    hash.field_debug("checked_derived_facts", result.checked_derived_facts());
    hash.field_bool("has_sat_report", result.sat_check_report().is_some());
    hash.field_debug("sat_check_report", result.sat_check_report());
    hash.field_bool("has_final_goal", result.final_goal().is_some());
    hash.field_debug("final_goal", result.final_goal());
    hash.field_u64("used_axiom_count", result.used_axioms().len() as u64);
    hash.field_debug("used_axioms", result.used_axioms());
    Some(hash.finalize())
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
                0x6f_6f_66_2d_70_6f_6c_69,
                0x63_79_2d_68_61_73_68_31,
                0x76_65_72_69_66_69_65_72,
            ],
            length: 0,
        };
        hasher.field_str("domain", domain);
        hasher
    }

    fn field_str(&mut self, label: &str, value: &str) {
        self.field_bytes(label, value.as_bytes());
    }

    fn field_debug(&mut self, label: &str, value: impl std::fmt::Debug) {
        self.field_str(label, &format!("{value:?}"));
    }

    fn field_u16(&mut self, label: &str, value: u16) {
        self.field_bytes(label, &value.to_le_bytes());
    }

    fn field_u64(&mut self, label: &str, value: u64) {
        self.field_bytes(label, &value.to_le_bytes());
    }

    fn field_bool(&mut self, label: &str, value: bool) {
        self.field_bytes(label, &[u8::from(value)]);
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

fn lower_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(char::from(HEX[(byte >> 4) as usize]));
        encoded.push(char::from(HEX[(byte & 0x0f) as usize]));
    }
    encoded
}

#[cfg(test)]
mod tests;
