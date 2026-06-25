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
mod tests {
    use super::*;

    fn target(byte: u8) -> TargetVcFingerprint {
        TargetVcFingerprint::new(1, vec![byte])
    }

    fn record(
        target_byte: u8,
        category: RejectionCategory,
        detail: RejectionDetail,
        location: RejectionLocation,
    ) -> RejectionRecord {
        RejectionRecord::new(target(target_byte), category, detail, location)
            .expect("valid rejection record")
    }

    #[test]
    fn stable_keys_and_category_detail_pairs_are_explicit() {
        let details = [
            (
                RejectionDetail::UnsupportedCertificateFormat,
                "unsupported_certificate_format",
                RejectionCategory::CertificateRejection,
            ),
            (
                RejectionDetail::ContextMismatch,
                "context_mismatch",
                RejectionCategory::CertificateRejection,
            ),
            (
                RejectionDetail::MalformedCertificate,
                "malformed_certificate",
                RejectionCategory::CertificateRejection,
            ),
            (
                RejectionDetail::MalformedWitnessData,
                "malformed_witness_data",
                RejectionCategory::CertificateRejection,
            ),
            (
                RejectionDetail::MissingProvenance,
                "missing_provenance",
                RejectionCategory::KernelRejection,
            ),
            (
                RejectionDetail::ResourceExhaustion,
                "resource_exhaustion",
                RejectionCategory::CertificateRejection,
            ),
            (
                RejectionDetail::InvalidSubstitution,
                "invalid_substitution",
                RejectionCategory::KernelRejection,
            ),
            (
                RejectionDetail::InvalidSatProof,
                "invalid_sat_proof",
                RejectionCategory::KernelRejection,
            ),
            (
                RejectionDetail::InvalidClusterTrace,
                "invalid_cluster_trace",
                RejectionCategory::KernelRejection,
            ),
            (
                RejectionDetail::UnresolvedSymbol,
                "unresolved_symbol",
                RejectionCategory::KernelRejection,
            ),
            (
                RejectionDetail::Timeout,
                "timeout",
                RejectionCategory::KernelRejection,
            ),
        ];

        assert_eq!(
            RejectionCategory::CertificateRejection.stable_key(),
            "certificate_rejection"
        );
        assert_eq!(
            RejectionCategory::KernelRejection.stable_key(),
            "kernel_rejection"
        );
        for (detail, key, category) in details {
            assert_eq!(detail.stable_key(), key);
            assert!(detail.is_allowed_for(category));
            assert!(!detail.is_acceptance());
        }
        assert!(
            RejectionDetail::ResourceExhaustion
                .is_allowed_for(RejectionCategory::CertificateRejection)
        );
        assert!(
            RejectionDetail::ResourceExhaustion.is_allowed_for(RejectionCategory::KernelRejection)
        );
    }

    #[test]
    fn invalid_category_detail_mappings_are_rejected() {
        for detail in [
            RejectionDetail::UnsupportedCertificateFormat,
            RejectionDetail::ContextMismatch,
            RejectionDetail::MalformedCertificate,
            RejectionDetail::MalformedWitnessData,
        ] {
            assert_eq!(
                RejectionRecord::new(
                    target(1),
                    RejectionCategory::KernelRejection,
                    detail,
                    RejectionLocation::new()
                ),
                Err(RejectionRecordError::InvalidCategoryDetail {
                    category: RejectionCategory::KernelRejection,
                    detail,
                })
            );
        }

        for detail in [
            RejectionDetail::MissingProvenance,
            RejectionDetail::InvalidSubstitution,
            RejectionDetail::InvalidSatProof,
            RejectionDetail::InvalidClusterTrace,
            RejectionDetail::UnresolvedSymbol,
            RejectionDetail::Timeout,
        ] {
            assert_eq!(
                RejectionRecord::new(
                    target(1),
                    RejectionCategory::CertificateRejection,
                    detail,
                    RejectionLocation::new()
                ),
                Err(RejectionRecordError::InvalidCategoryDetail {
                    category: RejectionCategory::CertificateRejection,
                    detail,
                })
            );
        }
    }

    #[test]
    fn parser_conversion_preserves_target_fallback_and_location() {
        for (parser_detail, shared_detail) in [
            (
                CertificateRejectionDetail::UnsupportedCertificateFormat,
                RejectionDetail::UnsupportedCertificateFormat,
            ),
            (
                CertificateRejectionDetail::ContextMismatch,
                RejectionDetail::ContextMismatch,
            ),
            (
                CertificateRejectionDetail::MalformedCertificate,
                RejectionDetail::MalformedCertificate,
            ),
            (
                CertificateRejectionDetail::ResourceExhaustion,
                RejectionDetail::ResourceExhaustion,
            ),
        ] {
            let parser_error = CertificateParseError {
                category: FailureCategory::CertificateRejection,
                detail: parser_detail,
                location: CertificateParseLocation {
                    byte_offset: 42,
                    section_tag: Some(SectionTag::VariableManifest),
                    item_index: Some(7),
                    field_path: Some("target_vc"),
                },
            };
            let expected_target = target(9);

            let record = RejectionRecord::from_certificate_parse_error(
                expected_target.clone(),
                parser_error,
            )
            .expect("parser detail maps to shared rejection");

            assert_eq!(record.target_vc_fingerprint(), &expected_target);
            assert_eq!(record.category(), RejectionCategory::CertificateRejection);
            assert_eq!(record.detail(), shared_detail);
            assert_ne!(record.detail(), RejectionDetail::Timeout);
            assert_eq!(record.location().certificate_byte_offset, Some(42));
            assert_eq!(
                record.location().section_tag,
                Some(SectionTag::VariableManifest)
            );
            assert_eq!(record.location().item_index, Some(7));
            assert_eq!(record.location().field_path, Some("target_vc"));
            assert!(!record.is_acceptance());
        }
    }

    #[test]
    fn checker_locations_cover_all_evidence_fields() {
        let location = RejectionLocation::new()
            .with_resolution_step_id(1)
            .with_substitution_id(2)
            .with_clause_ref(ClauseRef::new(ClauseRefNamespace::GeneratedClause, 3))
            .with_imported_fact_id(4)
            .with_cluster_trace_step_id(5)
            .with_reduction_step_id(6)
            .with_derived_fact_id(7)
            .with_final_goal();

        assert_eq!(location.resolution_step_id, Some(1));
        assert_eq!(location.substitution_id, Some(2));
        assert_eq!(
            location.clause_ref,
            Some(ClauseRef::new(ClauseRefNamespace::GeneratedClause, 3))
        );
        assert_eq!(location.imported_fact_id, Some(4));
        assert_eq!(location.cluster_trace_step_id, Some(5));
        assert_eq!(location.reduction_step_id, Some(6));
        assert_eq!(location.derived_fact_id, Some(7));
        assert!(location.final_goal);
    }

    #[test]
    fn checker_owner_mappings_use_expected_details() {
        assert!(
            RejectionRecord::new(
                target(1),
                RejectionCategory::KernelRejection,
                RejectionDetail::InvalidSatProof,
                RejectionLocation::new().with_resolution_step_id(1),
            )
            .is_ok()
        );
        assert!(
            RejectionRecord::new(
                target(1),
                RejectionCategory::KernelRejection,
                RejectionDetail::InvalidSubstitution,
                RejectionLocation::new().with_substitution_id(1),
            )
            .is_ok()
        );
        assert!(
            RejectionRecord::new(
                target(1),
                RejectionCategory::KernelRejection,
                RejectionDetail::InvalidClusterTrace,
                RejectionLocation::new().with_cluster_trace_step_id(1),
            )
            .is_ok()
        );
        assert!(
            RejectionRecord::new(
                target(1),
                RejectionCategory::KernelRejection,
                RejectionDetail::UnresolvedSymbol,
                RejectionLocation::new().with_imported_fact_id(1),
            )
            .is_ok()
        );
        assert!(
            RejectionRecord::new(
                target(1),
                RejectionCategory::KernelRejection,
                RejectionDetail::MissingProvenance,
                RejectionLocation::new(),
            )
            .is_ok()
        );
        assert!(
            RejectionRecord::new(
                target(1),
                RejectionCategory::CertificateRejection,
                RejectionDetail::MalformedWitnessData,
                RejectionLocation::new(),
            )
            .is_ok()
        );
    }

    #[test]
    fn deterministic_ordering_uses_documented_tie_breakers() {
        let mut records = [
            record(
                2,
                RejectionCategory::CertificateRejection,
                RejectionDetail::MalformedCertificate,
                RejectionLocation::new().with_certificate_byte_offset(1),
            ),
            record(
                1,
                RejectionCategory::KernelRejection,
                RejectionDetail::InvalidSubstitution,
                RejectionLocation::new().with_substitution_id(1),
            ),
            record(
                1,
                RejectionCategory::CertificateRejection,
                RejectionDetail::MalformedCertificate,
                RejectionLocation::new()
                    .with_certificate_byte_offset(1)
                    .with_section_tag(SectionTag::GeneratedClauses)
                    .with_item_index(1)
                    .with_field_path("b"),
            ),
            record(
                1,
                RejectionCategory::CertificateRejection,
                RejectionDetail::UnsupportedCertificateFormat,
                RejectionLocation::new()
                    .with_certificate_byte_offset(1)
                    .with_section_tag(SectionTag::GeneratedClauses)
                    .with_item_index(1)
                    .with_field_path("a"),
            ),
            record(
                1,
                RejectionCategory::CertificateRejection,
                RejectionDetail::ContextMismatch,
                RejectionLocation::new().with_certificate_byte_offset(2),
            ),
            record(
                1,
                RejectionCategory::KernelRejection,
                RejectionDetail::InvalidClusterTrace,
                RejectionLocation::new().with_cluster_trace_step_id(1),
            ),
            record(
                1,
                RejectionCategory::KernelRejection,
                RejectionDetail::InvalidClusterTrace,
                RejectionLocation::new().with_reduction_step_id(1),
            ),
            record(
                1,
                RejectionCategory::KernelRejection,
                RejectionDetail::InvalidSatProof,
                RejectionLocation::new().with_derived_fact_id(1),
            ),
            record(
                1,
                RejectionCategory::KernelRejection,
                RejectionDetail::InvalidSatProof,
                RejectionLocation::new().with_final_goal(),
            ),
            record(
                1,
                RejectionCategory::KernelRejection,
                RejectionDetail::InvalidSatProof,
                RejectionLocation::new()
                    .with_clause_ref(ClauseRef::new(ClauseRefNamespace::GeneratedClause, 1)),
            ),
            record(
                1,
                RejectionCategory::KernelRejection,
                RejectionDetail::UnresolvedSymbol,
                RejectionLocation::new()
                    .with_clause_ref(ClauseRef::new(ClauseRefNamespace::ImportedTheorem, 1)),
            ),
            record(
                1,
                RejectionCategory::KernelRejection,
                RejectionDetail::UnresolvedSymbol,
                RejectionLocation::new()
                    .with_clause_ref(ClauseRef::new(ClauseRefNamespace::ImportedAxiom, 1)),
            ),
            record(
                1,
                RejectionCategory::KernelRejection,
                RejectionDetail::UnresolvedSymbol,
                RejectionLocation::new().with_imported_fact_id(1),
            ),
            record(
                1,
                RejectionCategory::KernelRejection,
                RejectionDetail::InvalidSatProof,
                RejectionLocation::new().with_resolution_step_id(1),
            ),
            record(
                1,
                RejectionCategory::KernelRejection,
                RejectionDetail::InvalidSatProof,
                RejectionLocation::new()
                    .with_clause_ref(ClauseRef::new(ClauseRefNamespace::ResolutionStep, 1)),
            ),
        ];

        records.sort();

        assert_eq!(
            records[0].detail(),
            RejectionDetail::UnsupportedCertificateFormat
        );
        assert_eq!(records[1].detail(), RejectionDetail::MalformedCertificate);
        assert_eq!(records[2].detail(), RejectionDetail::ContextMismatch);
        assert_eq!(records[3].detail(), RejectionDetail::UnresolvedSymbol);
        assert_eq!(records[3].location().imported_fact_id, Some(1));
        assert_eq!(
            records[4]
                .location()
                .clause_ref
                .expect("imported axiom")
                .namespace,
            ClauseRefNamespace::ImportedAxiom
        );
        assert_eq!(
            records[5]
                .location()
                .clause_ref
                .expect("imported theorem")
                .namespace,
            ClauseRefNamespace::ImportedTheorem
        );
        assert_eq!(records[6].detail(), RejectionDetail::InvalidSatProof);
        assert_eq!(records[6].location().clause_ref.expect("clause ref").id, 1);
        assert_eq!(records[7].detail(), RejectionDetail::InvalidSatProof);
        assert_eq!(records[7].location().resolution_step_id, Some(1));
        assert_eq!(
            records[8]
                .location()
                .clause_ref
                .expect("resolution ref")
                .namespace,
            ClauseRefNamespace::ResolutionStep
        );
        assert_eq!(records[9].detail(), RejectionDetail::InvalidSubstitution);
        assert_eq!(records[10].detail(), RejectionDetail::InvalidClusterTrace);
        assert_eq!(records[10].location().cluster_trace_step_id, Some(1));
        assert_eq!(records[11].detail(), RejectionDetail::InvalidClusterTrace);
        assert_eq!(records[11].location().reduction_step_id, Some(1));
        assert_eq!(records[12].location().derived_fact_id, Some(1));
        assert!(records[13].location().final_goal);
        assert_eq!(records[14].target_vc_fingerprint(), &target(2));
    }

    #[test]
    fn deterministic_ordering_isolates_section_item_and_detail_ties() {
        let mut category_records = [
            record(
                1,
                RejectionCategory::KernelRejection,
                RejectionDetail::ResourceExhaustion,
                RejectionLocation::new(),
            ),
            record(
                1,
                RejectionCategory::CertificateRejection,
                RejectionDetail::ResourceExhaustion,
                RejectionLocation::new(),
            ),
        ];

        category_records.sort();

        assert_eq!(
            category_records[0].category(),
            RejectionCategory::CertificateRejection
        );
        assert_eq!(
            category_records[1].category(),
            RejectionCategory::KernelRejection
        );

        let mut byte_offset_records = [
            record(
                1,
                RejectionCategory::CertificateRejection,
                RejectionDetail::MalformedCertificate,
                RejectionLocation::new()
                    .with_certificate_byte_offset(2)
                    .with_section_tag(SectionTag::SymbolManifest)
                    .with_item_index(1)
                    .with_field_path("same"),
            ),
            record(
                1,
                RejectionCategory::CertificateRejection,
                RejectionDetail::MalformedCertificate,
                RejectionLocation::new()
                    .with_certificate_byte_offset(1)
                    .with_section_tag(SectionTag::SymbolManifest)
                    .with_item_index(1)
                    .with_field_path("same"),
            ),
        ];

        byte_offset_records.sort();

        assert_eq!(
            byte_offset_records[0].location().certificate_byte_offset,
            Some(1)
        );
        assert_eq!(
            byte_offset_records[1].location().certificate_byte_offset,
            Some(2)
        );

        let mut section_records = [
            record(
                1,
                RejectionCategory::CertificateRejection,
                RejectionDetail::MalformedCertificate,
                RejectionLocation::new()
                    .with_certificate_byte_offset(1)
                    .with_section_tag(SectionTag::FinalGoal)
                    .with_item_index(1)
                    .with_field_path("same"),
            ),
            record(
                1,
                RejectionCategory::CertificateRejection,
                RejectionDetail::MalformedCertificate,
                RejectionLocation::new()
                    .with_certificate_byte_offset(1)
                    .with_section_tag(SectionTag::SymbolManifest)
                    .with_item_index(1)
                    .with_field_path("same"),
            ),
        ];

        section_records.sort();

        assert_eq!(
            section_records[0].location().section_tag,
            Some(SectionTag::SymbolManifest)
        );
        assert_eq!(
            section_records[1].location().section_tag,
            Some(SectionTag::FinalGoal)
        );

        let mut records = [
            record(
                1,
                RejectionCategory::CertificateRejection,
                RejectionDetail::MalformedCertificate,
                RejectionLocation::new()
                    .with_certificate_byte_offset(1)
                    .with_section_tag(SectionTag::FinalGoal),
            ),
            record(
                1,
                RejectionCategory::CertificateRejection,
                RejectionDetail::MalformedCertificate,
                RejectionLocation::new()
                    .with_certificate_byte_offset(1)
                    .with_section_tag(SectionTag::SymbolManifest)
                    .with_item_index(2),
            ),
            record(
                1,
                RejectionCategory::CertificateRejection,
                RejectionDetail::ContextMismatch,
                RejectionLocation::new()
                    .with_certificate_byte_offset(1)
                    .with_section_tag(SectionTag::SymbolManifest)
                    .with_item_index(1),
            ),
            record(
                1,
                RejectionCategory::CertificateRejection,
                RejectionDetail::MalformedCertificate,
                RejectionLocation::new()
                    .with_certificate_byte_offset(1)
                    .with_section_tag(SectionTag::SymbolManifest)
                    .with_item_index(1),
            ),
        ];

        records.sort();

        assert_eq!(records[0].detail(), RejectionDetail::ContextMismatch);
        assert_eq!(records[0].location().item_index, Some(1));
        assert_eq!(records[1].detail(), RejectionDetail::MalformedCertificate);
        assert_eq!(records[1].location().item_index, Some(1));
        assert_eq!(records[2].location().item_index, Some(2));
        assert_eq!(
            records[3].location().section_tag,
            Some(SectionTag::FinalGoal)
        );
    }

    #[test]
    fn target_sort_bytes_use_fixed_width_length() {
        assert_eq!(
            TargetVcFingerprint::new(2, vec![3, 4]).sort_bytes(),
            vec![2, 0, 0, 0, 2, 3, 4]
        );
    }

    #[test]
    fn public_enums_are_forward_compatible() {
        let source = include_str!("rejection.rs");
        for enum_name in [
            "RejectionCategory",
            "RejectionDetail",
            "ClauseRefNamespace",
            "RejectionRecordError",
        ] {
            assert!(
                source.contains(&format!("#[non_exhaustive]\npub enum {enum_name}")),
                "{enum_name} must be #[non_exhaustive]"
            );
        }
    }
}
