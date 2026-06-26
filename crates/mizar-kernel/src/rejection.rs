use std::cmp::Ordering;

use crate::certificate_parser::{
    CertificateParseError, CertificateParseLocation, CertificateRejectionDetail, FailureCategory,
    Fingerprint, SectionTag,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetVcFingerprint {
    pub algorithm_id: u8,
    pub digest: Vec<u8>,
}

impl TargetVcFingerprint {
    #[must_use]
    pub fn new(algorithm_id: u8, digest: Vec<u8>) -> Self {
        Self {
            algorithm_id,
            digest,
        }
    }

    #[must_use]
    pub fn from_certificate_fingerprint(fingerprint: &Fingerprint) -> Self {
        Self::new(fingerprint.algorithm_id, fingerprint.digest.clone())
    }

    #[must_use]
    pub fn sort_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![self.algorithm_id];
        let digest_len = u32::try_from(self.digest.len()).unwrap_or(u32::MAX);
        bytes.extend_from_slice(&digest_len.to_be_bytes());
        bytes.extend_from_slice(&self.digest);
        bytes
    }
}

impl Ord for TargetVcFingerprint {
    fn cmp(&self, other: &Self) -> Ordering {
        self.sort_bytes().cmp(&other.sort_bytes())
    }
}

impl PartialOrd for TargetVcFingerprint {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RejectionCategory {
    CertificateRejection,
    KernelRejection,
}

impl RejectionCategory {
    #[must_use]
    pub const fn stable_key(self) -> &'static str {
        match self {
            Self::CertificateRejection => "certificate_rejection",
            Self::KernelRejection => "kernel_rejection",
        }
    }

    const fn rank(self) -> u8 {
        match self {
            Self::CertificateRejection => 0,
            Self::KernelRejection => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RejectionDetail {
    UnsupportedCertificateFormat,
    ContextMismatch,
    MalformedCertificate,
    MalformedWitnessData,
    MissingProvenance,
    ResourceExhaustion,
    InvalidSubstitution,
    InvalidSatProof,
    InvalidSatRefutation,
    InvalidClusterTrace,
    UnresolvedSymbol,
    Timeout,
}

impl RejectionDetail {
    #[must_use]
    pub const fn stable_key(self) -> &'static str {
        match self {
            Self::UnsupportedCertificateFormat => "unsupported_certificate_format",
            Self::ContextMismatch => "context_mismatch",
            Self::MalformedCertificate => "malformed_certificate",
            Self::MalformedWitnessData => "malformed_witness_data",
            Self::MissingProvenance => "missing_provenance",
            Self::ResourceExhaustion => "resource_exhaustion",
            Self::InvalidSubstitution => "invalid_substitution",
            Self::InvalidSatProof => "invalid_sat_proof",
            Self::InvalidSatRefutation => "invalid_sat_refutation",
            Self::InvalidClusterTrace => "invalid_cluster_trace",
            Self::UnresolvedSymbol => "unresolved_symbol",
            Self::Timeout => "timeout",
        }
    }

    #[must_use]
    pub const fn is_allowed_for(self, category: RejectionCategory) -> bool {
        match self {
            Self::UnsupportedCertificateFormat
            | Self::ContextMismatch
            | Self::MalformedCertificate
            | Self::MalformedWitnessData => {
                matches!(category, RejectionCategory::CertificateRejection)
            }
            Self::MissingProvenance
            | Self::InvalidSubstitution
            | Self::InvalidSatProof
            | Self::InvalidSatRefutation
            | Self::InvalidClusterTrace
            | Self::UnresolvedSymbol
            | Self::Timeout => matches!(category, RejectionCategory::KernelRejection),
            Self::ResourceExhaustion => true,
        }
    }

    #[must_use]
    pub const fn is_acceptance(self) -> bool {
        false
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[non_exhaustive]
pub enum ClauseRefNamespace {
    GeneratedClause,
    ResolutionStep,
    ImportedAxiom,
    ImportedTheorem,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ClauseRef {
    pub namespace: ClauseRefNamespace,
    pub id: u32,
}

impl ClauseRef {
    #[must_use]
    pub const fn new(namespace: ClauseRefNamespace, id: u32) -> Self {
        Self { namespace, id }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct RejectionLocation {
    pub certificate_byte_offset: Option<usize>,
    pub section_tag: Option<SectionTag>,
    pub item_index: Option<u32>,
    pub field_path: Option<&'static str>,
    pub clause_ref: Option<ClauseRef>,
    pub resolution_step_id: Option<u32>,
    pub substitution_id: Option<u32>,
    pub imported_fact_id: Option<u32>,
    pub cluster_trace_step_id: Option<u32>,
    pub reduction_step_id: Option<u32>,
    pub derived_fact_id: Option<u32>,
    pub final_goal: bool,
}

impl RejectionLocation {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn from_certificate_parse_location(location: &CertificateParseLocation) -> Self {
        Self {
            certificate_byte_offset: Some(location.byte_offset),
            section_tag: location.section_tag,
            item_index: location.item_index,
            field_path: location.field_path,
            ..Self::default()
        }
    }

    #[must_use]
    pub const fn with_certificate_byte_offset(mut self, byte_offset: usize) -> Self {
        self.certificate_byte_offset = Some(byte_offset);
        self
    }

    #[must_use]
    pub const fn with_section_tag(mut self, section_tag: SectionTag) -> Self {
        self.section_tag = Some(section_tag);
        self
    }

    #[must_use]
    pub const fn with_item_index(mut self, item_index: u32) -> Self {
        self.item_index = Some(item_index);
        self
    }

    #[must_use]
    pub const fn with_field_path(mut self, field_path: &'static str) -> Self {
        self.field_path = Some(field_path);
        self
    }

    #[must_use]
    pub const fn with_clause_ref(mut self, clause_ref: ClauseRef) -> Self {
        self.clause_ref = Some(clause_ref);
        self
    }

    #[must_use]
    pub const fn with_resolution_step_id(mut self, resolution_step_id: u32) -> Self {
        self.resolution_step_id = Some(resolution_step_id);
        self
    }

    #[must_use]
    pub const fn with_substitution_id(mut self, substitution_id: u32) -> Self {
        self.substitution_id = Some(substitution_id);
        self
    }

    #[must_use]
    pub const fn with_imported_fact_id(mut self, imported_fact_id: u32) -> Self {
        self.imported_fact_id = Some(imported_fact_id);
        self
    }

    #[must_use]
    pub const fn with_cluster_trace_step_id(mut self, cluster_trace_step_id: u32) -> Self {
        self.cluster_trace_step_id = Some(cluster_trace_step_id);
        self
    }

    #[must_use]
    pub const fn with_reduction_step_id(mut self, reduction_step_id: u32) -> Self {
        self.reduction_step_id = Some(reduction_step_id);
        self
    }

    #[must_use]
    pub const fn with_derived_fact_id(mut self, derived_fact_id: u32) -> Self {
        self.derived_fact_id = Some(derived_fact_id);
        self
    }

    #[must_use]
    pub const fn with_final_goal(mut self) -> Self {
        self.final_goal = true;
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RejectionRecord {
    target_vc_fingerprint: TargetVcFingerprint,
    category: RejectionCategory,
    detail: RejectionDetail,
    location: RejectionLocation,
}

impl RejectionRecord {
    pub fn new(
        target_vc_fingerprint: TargetVcFingerprint,
        category: RejectionCategory,
        detail: RejectionDetail,
        location: RejectionLocation,
    ) -> Result<Self, RejectionRecordError> {
        if !detail.is_allowed_for(category) {
            return Err(RejectionRecordError::InvalidCategoryDetail { category, detail });
        }
        Ok(Self {
            target_vc_fingerprint,
            category,
            detail,
            location,
        })
    }

    pub fn from_certificate_parse_error(
        target_vc_fingerprint: TargetVcFingerprint,
        error: CertificateParseError,
    ) -> Result<Self, RejectionRecordError> {
        let category = match error.category {
            FailureCategory::CertificateRejection => RejectionCategory::CertificateRejection,
        };
        Self::new(
            target_vc_fingerprint,
            category,
            map_certificate_detail(error.detail),
            RejectionLocation::from_certificate_parse_location(&error.location),
        )
    }

    #[must_use]
    pub const fn category(&self) -> RejectionCategory {
        self.category
    }

    #[must_use]
    pub const fn detail(&self) -> RejectionDetail {
        self.detail
    }

    #[must_use]
    pub const fn target_vc_fingerprint(&self) -> &TargetVcFingerprint {
        &self.target_vc_fingerprint
    }

    #[must_use]
    pub const fn location(&self) -> &RejectionLocation {
        &self.location
    }

    #[must_use]
    pub const fn stable_detail_key(&self) -> &'static str {
        self.detail.stable_key()
    }

    #[must_use]
    pub const fn is_acceptance(&self) -> bool {
        false
    }
}

impl Ord for RejectionRecord {
    fn cmp(&self, other: &Self) -> Ordering {
        self.target_vc_fingerprint
            .cmp(&other.target_vc_fingerprint)
            .then_with(|| self.category.rank().cmp(&other.category.rank()))
            .then_with(|| {
                option_usize_key(self.location.certificate_byte_offset)
                    .cmp(&option_usize_key(other.location.certificate_byte_offset))
            })
            .then_with(|| {
                option_copy_key(self.location.section_tag)
                    .cmp(&option_copy_key(other.location.section_tag))
            })
            .then_with(|| {
                option_copy_key(self.location.item_index)
                    .cmp(&option_copy_key(other.location.item_index))
            })
            .then_with(|| {
                option_str_key(self.location.field_path)
                    .cmp(&option_str_key(other.location.field_path))
            })
            .then_with(|| {
                option_copy_key(self.location.imported_fact_id)
                    .cmp(&option_copy_key(other.location.imported_fact_id))
            })
            .then_with(|| {
                option_copy_key(imported_clause_ref_key(self.location.clause_ref)).cmp(
                    &option_copy_key(imported_clause_ref_key(other.location.clause_ref)),
                )
            })
            .then_with(|| {
                option_copy_key(generated_clause_ref_key(self.location.clause_ref)).cmp(
                    &option_copy_key(generated_clause_ref_key(other.location.clause_ref)),
                )
            })
            .then_with(|| {
                option_copy_key(self.location.resolution_step_id)
                    .cmp(&option_copy_key(other.location.resolution_step_id))
            })
            .then_with(|| {
                option_copy_key(resolution_clause_ref_key(self.location.clause_ref)).cmp(
                    &option_copy_key(resolution_clause_ref_key(other.location.clause_ref)),
                )
            })
            .then_with(|| {
                option_copy_key(self.location.substitution_id)
                    .cmp(&option_copy_key(other.location.substitution_id))
            })
            .then_with(|| {
                option_copy_key(self.location.cluster_trace_step_id)
                    .cmp(&option_copy_key(other.location.cluster_trace_step_id))
            })
            .then_with(|| {
                option_copy_key(self.location.reduction_step_id)
                    .cmp(&option_copy_key(other.location.reduction_step_id))
            })
            .then_with(|| {
                option_copy_key(self.location.derived_fact_id)
                    .cmp(&option_copy_key(other.location.derived_fact_id))
            })
            .then_with(|| self.location.final_goal.cmp(&other.location.final_goal))
            .then_with(|| self.detail.stable_key().cmp(other.detail.stable_key()))
    }
}

impl PartialOrd for RejectionRecord {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RejectionRecordError {
    InvalidCategoryDetail {
        category: RejectionCategory,
        detail: RejectionDetail,
    },
}

fn map_certificate_detail(detail: CertificateRejectionDetail) -> RejectionDetail {
    match detail {
        CertificateRejectionDetail::UnsupportedCertificateFormat => {
            RejectionDetail::UnsupportedCertificateFormat
        }
        CertificateRejectionDetail::ContextMismatch => RejectionDetail::ContextMismatch,
        CertificateRejectionDetail::MalformedCertificate => RejectionDetail::MalformedCertificate,
        CertificateRejectionDetail::ResourceExhaustion => RejectionDetail::ResourceExhaustion,
    }
}

fn option_usize_key(value: Option<usize>) -> (bool, usize) {
    value.map_or((true, 0), |value| (false, value))
}

fn option_copy_key<T: Copy + Ord>(value: Option<T>) -> (bool, Option<T>) {
    value.map_or((true, None), |value| (false, Some(value)))
}

fn option_str_key(value: Option<&'static str>) -> (bool, &'static str) {
    value.map_or((true, ""), |value| (false, value))
}

fn imported_clause_ref_key(clause_ref: Option<ClauseRef>) -> Option<(u8, u32)> {
    match clause_ref {
        Some(ClauseRef { namespace, id }) => match namespace {
            ClauseRefNamespace::ImportedAxiom => Some((0, id)),
            ClauseRefNamespace::ImportedTheorem => Some((1, id)),
            ClauseRefNamespace::GeneratedClause | ClauseRefNamespace::ResolutionStep => None,
        },
        None => None,
    }
}

fn generated_clause_ref_key(clause_ref: Option<ClauseRef>) -> Option<u32> {
    match clause_ref {
        Some(ClauseRef { namespace, id }) => match namespace {
            ClauseRefNamespace::GeneratedClause => Some(id),
            ClauseRefNamespace::ResolutionStep
            | ClauseRefNamespace::ImportedAxiom
            | ClauseRefNamespace::ImportedTheorem => None,
        },
        None => None,
    }
}

fn resolution_clause_ref_key(clause_ref: Option<ClauseRef>) -> Option<u32> {
    match clause_ref {
        Some(ClauseRef { namespace, id }) => match namespace {
            ClauseRefNamespace::ResolutionStep => Some(id),
            ClauseRefNamespace::GeneratedClause
            | ClauseRefNamespace::ImportedAxiom
            | ClauseRefNamespace::ImportedTheorem => None,
        },
        None => None,
    }
}

#[cfg(test)]
mod tests;
