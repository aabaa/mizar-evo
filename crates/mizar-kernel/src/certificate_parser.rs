use std::collections::BTreeSet;

use crate::clause::{
    Atom, Clause, ClauseForm, ClauseProfile, ClauseValidationContext, Literal, Polarity, SymbolKey,
    SymbolKind, TautologyPolicy, Term, VariableId,
};

const DOMAIN_SEPARATOR: &[u8] = b"MIZAR_KERNEL_CERT\0";
const SCHEMA_VERSION_V1: u16 = 1;
const ENCODING_VERSION_V1: u16 = 1;
const PROFILE_LEN: usize = 8;
const REQUIRED_SECTIONS: [SectionTag; 9] = [
    SectionTag::SymbolManifest,
    SectionTag::VariableManifest,
    SectionTag::ImportedAxioms,
    SectionTag::ImportedTheorems,
    SectionTag::GeneratedClauses,
    SectionTag::Substitutions,
    SectionTag::ResolutionTrace,
    SectionTag::DerivedFacts,
    SectionTag::FinalGoal,
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CertificateParseContext {
    pub accepted_schema_versions: BTreeSet<u16>,
    pub accepted_encoding_versions: BTreeSet<u16>,
    pub accepted_kernel_profiles: BTreeSet<KernelProfileRecord>,
    pub expected_target_vc: Fingerprint,
    pub clause_validation_policy: ClauseValidationPolicy,
    pub max_certificate_bytes: usize,
    pub max_section_bytes: usize,
    pub max_imported_facts: usize,
    pub max_generated_clauses: usize,
    pub max_substitutions: usize,
    pub max_resolution_steps: usize,
    pub max_derived_facts: usize,
    pub max_symbol_manifest_entries: usize,
    pub max_variable_manifest_entries: usize,
    pub max_term_recursion_depth: usize,
}

impl CertificateParseContext {
    #[must_use]
    pub fn v1(expected_target_vc: Fingerprint, profile: KernelProfileRecord) -> Self {
        Self {
            accepted_schema_versions: BTreeSet::from([SCHEMA_VERSION_V1]),
            accepted_encoding_versions: BTreeSet::from([ENCODING_VERSION_V1]),
            accepted_kernel_profiles: BTreeSet::from([profile]),
            expected_target_vc,
            clause_validation_policy: ClauseValidationPolicy::default(),
            max_certificate_bytes: usize::MAX,
            max_section_bytes: usize::MAX,
            max_imported_facts: usize::MAX,
            max_generated_clauses: usize::MAX,
            max_substitutions: usize::MAX,
            max_resolution_steps: usize::MAX,
            max_derived_facts: usize::MAX,
            max_symbol_manifest_entries: usize::MAX,
            max_variable_manifest_entries: usize::MAX,
            max_term_recursion_depth: 64,
        }
    }

    #[must_use]
    pub const fn with_limits(mut self, limits: CertificateParseLimits) -> Self {
        self.max_certificate_bytes = limits.max_certificate_bytes;
        self.max_section_bytes = limits.max_section_bytes;
        self.max_imported_facts = limits.max_imported_facts;
        self.max_generated_clauses = limits.max_generated_clauses;
        self.max_substitutions = limits.max_substitutions;
        self.max_resolution_steps = limits.max_resolution_steps;
        self.max_derived_facts = limits.max_derived_facts;
        self.max_symbol_manifest_entries = limits.max_symbol_manifest_entries;
        self.max_variable_manifest_entries = limits.max_variable_manifest_entries;
        self.max_term_recursion_depth = limits.max_term_recursion_depth;
        self
    }

    #[must_use]
    pub const fn with_clause_validation_policy(mut self, policy: ClauseValidationPolicy) -> Self {
        self.clause_validation_policy = policy;
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CertificateParseLimits {
    pub max_certificate_bytes: usize,
    pub max_section_bytes: usize,
    pub max_imported_facts: usize,
    pub max_generated_clauses: usize,
    pub max_substitutions: usize,
    pub max_resolution_steps: usize,
    pub max_derived_facts: usize,
    pub max_symbol_manifest_entries: usize,
    pub max_variable_manifest_entries: usize,
    pub max_term_recursion_depth: usize,
}

impl Default for CertificateParseLimits {
    fn default() -> Self {
        Self {
            max_certificate_bytes: usize::MAX,
            max_section_bytes: usize::MAX,
            max_imported_facts: usize::MAX,
            max_generated_clauses: usize::MAX,
            max_substitutions: usize::MAX,
            max_resolution_steps: usize::MAX,
            max_derived_facts: usize::MAX,
            max_symbol_manifest_entries: usize::MAX,
            max_variable_manifest_entries: usize::MAX,
            max_term_recursion_depth: 64,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ClauseValidationPolicy {
    pub max_literals: usize,
    pub max_term_encoding_bytes: usize,
}

impl Default for ClauseValidationPolicy {
    fn default() -> Self {
        Self {
            max_literals: usize::MAX,
            max_term_encoding_bytes: usize::MAX,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct KernelProfileRecord {
    pub profile_id: u16,
    pub clause_schema_version: u16,
    pub clause_encoding_version: u16,
    pub clause_tautology_policy: ClauseTautologyPolicy,
    pub certificate_hash_input_algorithm: CertificateHashInputAlgorithm,
}

impl KernelProfileRecord {
    #[must_use]
    pub const fn v1(profile_id: u16, clause_tautology_policy: ClauseTautologyPolicy) -> Self {
        Self {
            profile_id,
            clause_schema_version: SCHEMA_VERSION_V1,
            clause_encoding_version: ENCODING_VERSION_V1,
            clause_tautology_policy,
            certificate_hash_input_algorithm: CertificateHashInputAlgorithm::CanonicalEnvelopeV1,
        }
    }

    fn clause_profile(&self) -> ClauseProfile {
        ClauseProfile::new(
            self.clause_schema_version,
            self.clause_encoding_version,
            self.clause_tautology_policy.into(),
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[non_exhaustive]
pub enum ClauseTautologyPolicy {
    Reject,
    Marker,
}

impl ClauseTautologyPolicy {
    const fn from_tag(tag: u8) -> Option<Self> {
        match tag {
            1 => Some(Self::Reject),
            2 => Some(Self::Marker),
            _ => None,
        }
    }

    #[must_use]
    pub const fn tag(self) -> u8 {
        match self {
            Self::Reject => 1,
            Self::Marker => 2,
        }
    }
}

impl From<ClauseTautologyPolicy> for TautologyPolicy {
    fn from(value: ClauseTautologyPolicy) -> Self {
        match value {
            ClauseTautologyPolicy::Reject => Self::Reject,
            ClauseTautologyPolicy::Marker => Self::Marker,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[non_exhaustive]
pub enum CertificateHashInputAlgorithm {
    CanonicalEnvelopeV1,
}

impl CertificateHashInputAlgorithm {
    const fn from_tag(tag: u8) -> Option<Self> {
        match tag {
            1 => Some(Self::CanonicalEnvelopeV1),
            _ => None,
        }
    }

    #[must_use]
    pub const fn tag(self) -> u8 {
        match self {
            Self::CanonicalEnvelopeV1 => 1,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Fingerprint {
    pub algorithm_id: u8,
    pub digest: Vec<u8>,
}

impl Fingerprint {
    #[must_use]
    pub fn new(algorithm_id: u8, digest: Vec<u8>) -> Self {
        Self {
            algorithm_id,
            digest,
        }
    }

    #[must_use = "fingerprint canonical bytes must be consumed or checked"]
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CertificateParseError> {
        let mut bytes = vec![self.algorithm_id];
        write_len(self.digest.len(), &mut bytes)?;
        bytes.extend_from_slice(&self.digest);
        Ok(bytes)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedCertificate {
    pub schema_version: u16,
    pub encoding_version: u16,
    pub kernel_profile: KernelProfileRecord,
    pub target_vc: Fingerprint,
    pub symbol_manifest: Vec<SymbolManifestEntry>,
    pub variable_manifest: Vec<VariableManifestEntry>,
    pub imported_axioms: Vec<ImportedFactRef>,
    pub imported_theorems: Vec<ImportedFactRef>,
    pub generated_clauses: Vec<GeneratedClause>,
    pub substitutions: Vec<SubstitutionEntry>,
    pub resolution_trace: Vec<ResolutionStep>,
    pub derived_facts: Vec<DerivedFact>,
    pub final_goal: FinalGoalRef,
    canonical_hash_input: Vec<u8>,
}

impl ParsedCertificate {
    #[must_use]
    pub fn canonical_hash_input(&self) -> &[u8] {
        &self.canonical_hash_input
    }

    #[cfg(test)]
    pub(crate) fn new_for_kernel_tests(parts: ParsedCertificateTestParts) -> Self {
        Self {
            schema_version: parts.schema_version,
            encoding_version: parts.encoding_version,
            kernel_profile: parts.kernel_profile,
            target_vc: parts.target_vc,
            symbol_manifest: parts.symbol_manifest,
            variable_manifest: parts.variable_manifest,
            imported_axioms: parts.imported_axioms,
            imported_theorems: parts.imported_theorems,
            generated_clauses: parts.generated_clauses,
            substitutions: parts.substitutions,
            resolution_trace: parts.resolution_trace,
            derived_facts: parts.derived_facts,
            final_goal: parts.final_goal,
            canonical_hash_input: parts.canonical_hash_input,
        }
    }
}

#[cfg(test)]
pub(crate) struct ParsedCertificateTestParts {
    pub schema_version: u16,
    pub encoding_version: u16,
    pub kernel_profile: KernelProfileRecord,
    pub target_vc: Fingerprint,
    pub symbol_manifest: Vec<SymbolManifestEntry>,
    pub variable_manifest: Vec<VariableManifestEntry>,
    pub imported_axioms: Vec<ImportedFactRef>,
    pub imported_theorems: Vec<ImportedFactRef>,
    pub generated_clauses: Vec<GeneratedClause>,
    pub substitutions: Vec<SubstitutionEntry>,
    pub resolution_trace: Vec<ResolutionStep>,
    pub derived_facts: Vec<DerivedFact>,
    pub final_goal: FinalGoalRef,
    pub canonical_hash_input: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SymbolManifestEntry {
    pub symbol: SymbolKey,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct VariableManifestEntry {
    pub variable_id: VariableId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportedFactRef {
    pub imported_fact_id: u32,
    pub package_id: Vec<u8>,
    pub module_path: Vec<u8>,
    pub exported_item_id: Vec<u8>,
    pub statement_fingerprint: Fingerprint,
    pub required_proof_status: RequiredProofStatus,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[non_exhaustive]
pub enum RequiredProofStatus {
    KernelVerified,
    DischargedBuiltin,
    ExternallyAttestedPolicyPermitted,
}

impl RequiredProofStatus {
    const fn from_tag(tag: u8) -> Option<Self> {
        match tag {
            1 => Some(Self::KernelVerified),
            2 => Some(Self::DischargedBuiltin),
            3 => Some(Self::ExternallyAttestedPolicyPermitted),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GeneratedClause {
    pub clause_id: u32,
    pub clause: Clause,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubstitutionEntry {
    pub substitution_id: u32,
    pub source_term: Term,
    pub target_term: Term,
    pub binder_context_encoding: Vec<u8>,
    pub freshness_witness_refs: Vec<u32>,
    pub free_variable_constraint_refs: Vec<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolutionStep {
    pub step_id: u32,
    pub parent_a: ClauseRef,
    pub parent_b: ClauseRef,
    pub pivot_literal: Literal,
    pub generated_clause: ClauseRef,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DerivedFact {
    pub derived_fact_id: u32,
    pub source: ClauseRef,
    pub payload: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FinalGoalRef {
    pub namespace: FinalGoalNamespace,
    pub id: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ClauseRef {
    pub namespace: ClauseRefNamespace,
    pub id: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[non_exhaustive]
pub enum ClauseRefNamespace {
    GeneratedClause,
    ResolutionStep,
    ImportedAxiom,
    ImportedTheorem,
}

impl ClauseRefNamespace {
    const fn from_tag(tag: u8) -> Option<Self> {
        match tag {
            1 => Some(Self::GeneratedClause),
            2 => Some(Self::ResolutionStep),
            3 => Some(Self::ImportedAxiom),
            4 => Some(Self::ImportedTheorem),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum FinalGoalNamespace {
    GeneratedClause,
    ResolutionStep,
    DerivedFact,
}

impl FinalGoalNamespace {
    const fn from_tag(tag: u8) -> Option<Self> {
        match tag {
            1 => Some(Self::GeneratedClause),
            2 => Some(Self::ResolutionStep),
            3 => Some(Self::DerivedFact),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CertificateParseError {
    pub category: FailureCategory,
    pub detail: CertificateRejectionDetail,
    pub location: CertificateParseLocation,
}

impl CertificateParseError {
    fn new(detail: CertificateRejectionDetail, location: CertificateParseLocation) -> Self {
        Self {
            category: FailureCategory::CertificateRejection,
            detail,
            location,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum FailureCategory {
    CertificateRejection,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CertificateRejectionDetail {
    UnsupportedCertificateFormat,
    ContextMismatch,
    MalformedCertificate,
    ResourceExhaustion,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CertificateParseLocation {
    pub byte_offset: usize,
    pub section_tag: Option<SectionTag>,
    pub item_index: Option<u32>,
    pub field_path: Option<&'static str>,
}

impl CertificateParseLocation {
    const fn new(byte_offset: usize) -> Self {
        Self {
            byte_offset,
            section_tag: None,
            item_index: None,
            field_path: None,
        }
    }

    const fn with_field(mut self, field_path: &'static str) -> Self {
        self.field_path = Some(field_path);
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[non_exhaustive]
pub enum SectionTag {
    SymbolManifest,
    VariableManifest,
    ImportedAxioms,
    ImportedTheorems,
    GeneratedClauses,
    Substitutions,
    ResolutionTrace,
    DerivedFacts,
    FinalGoal,
}

impl SectionTag {
    const fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0x01 => Some(Self::SymbolManifest),
            0x02 => Some(Self::VariableManifest),
            0x03 => Some(Self::ImportedAxioms),
            0x04 => Some(Self::ImportedTheorems),
            0x05 => Some(Self::GeneratedClauses),
            0x06 => Some(Self::Substitutions),
            0x07 => Some(Self::ResolutionTrace),
            0x08 => Some(Self::DerivedFacts),
            0x09 => Some(Self::FinalGoal),
            _ => None,
        }
    }

    const fn byte(self) -> u8 {
        match self {
            Self::SymbolManifest => 0x01,
            Self::VariableManifest => 0x02,
            Self::ImportedAxioms => 0x03,
            Self::ImportedTheorems => 0x04,
            Self::GeneratedClauses => 0x05,
            Self::Substitutions => 0x06,
            Self::ResolutionTrace => 0x07,
            Self::DerivedFacts => 0x08,
            Self::FinalGoal => 0x09,
        }
    }
}

pub fn parse_certificate(
    bytes: &[u8],
    context: &CertificateParseContext,
) -> Result<ParsedCertificate, CertificateParseError> {
    if bytes.len() > context.max_certificate_bytes {
        return Err(resource_error(0, "certificate"));
    }

    let mut reader = Reader::new(bytes, 0, None, None);
    reader.read_exact(DOMAIN_SEPARATOR.len(), "domain_separator")?;
    if &bytes[..DOMAIN_SEPARATOR.len()] != DOMAIN_SEPARATOR {
        return Err(unsupported_error(0, "domain_separator"));
    }

    let schema_version = reader.read_u16("schema_version")?;
    if !context.accepted_schema_versions.contains(&schema_version) || schema_version != 1 {
        return Err(unsupported_error(DOMAIN_SEPARATOR.len(), "schema_version"));
    }
    let encoding_version = reader.read_u16("encoding_version")?;
    if !context
        .accepted_encoding_versions
        .contains(&encoding_version)
        || encoding_version != 1
    {
        return Err(unsupported_error(
            DOMAIN_SEPARATOR.len() + 2,
            "encoding_version",
        ));
    }

    let kernel_profile = read_kernel_profile(&mut reader)?;
    if !context.accepted_kernel_profiles.contains(&kernel_profile) {
        return Err(unsupported_error(
            DOMAIN_SEPARATOR.len() + 4,
            "kernel_profile",
        ));
    }
    let target_vc = read_fingerprint(&mut reader, "target_vc")?;
    if target_vc != context.expected_target_vc {
        return Err(CertificateParseError::new(
            CertificateRejectionDetail::ContextMismatch,
            CertificateParseLocation::new(DOMAIN_SEPARATOR.len() + 4 + PROFILE_LEN)
                .with_field("target_vc"),
        ));
    }

    let directory_count = reader.read_u32("directory_entry_count")?;
    if directory_count as usize != REQUIRED_SECTIONS.len() {
        return Err(malformed_at(
            reader.offset().saturating_sub(4),
            "directory_entry_count",
        ));
    }

    let mut directory = Vec::with_capacity(REQUIRED_SECTIONS.len());
    for expected in REQUIRED_SECTIONS {
        let entry_offset = reader.offset();
        let section_tag_byte = reader.read_u8("directory.section_tag")?;
        let Some(section_tag) = SectionTag::from_byte(section_tag_byte) else {
            return Err(unsupported_error(entry_offset, "directory.section_tag"));
        };
        if section_tag != expected {
            return Err(malformed_at(entry_offset, "directory.section_tag"));
        }
        let item_count = reader.read_u32("directory.item_count")?;
        let payload_offset = reader.read_u32("directory.payload_offset")?;
        let payload_length = reader.read_u32("directory.payload_length")?;
        let entry = DirectoryEntry {
            section_tag,
            item_count,
            payload_offset,
            payload_length,
            entry_offset,
            payload_base_offset: 0,
        };
        validate_section_limit(entry, context)?;
        directory.push(entry);
    }

    let payload_base = reader.offset();
    for entry in &mut directory {
        entry.payload_base_offset = payload_base;
    }
    let payload_bytes = &bytes[payload_base..];
    validate_directory_ranges(&directory, payload_bytes.len())?;

    let symbol_manifest =
        parse_symbol_manifest(section_slice(payload_bytes, &directory[0]), &directory[0])?;
    let variable_manifest =
        parse_variable_manifest(section_slice(payload_bytes, &directory[1]), &directory[1])?;
    let imported_axioms =
        parse_imported_facts(section_slice(payload_bytes, &directory[2]), &directory[2])?;
    let imported_theorems =
        parse_imported_facts(section_slice(payload_bytes, &directory[3]), &directory[3])?;
    let generated_clauses = parse_generated_clauses(
        section_slice(payload_bytes, &directory[4]),
        &directory[4],
        &kernel_profile,
        &symbol_manifest,
        &variable_manifest,
        context,
    )?;
    let substitutions = parse_substitutions(
        section_slice(payload_bytes, &directory[5]),
        &directory[5],
        context,
    )?;
    let resolution_trace = parse_resolution_trace(
        section_slice(payload_bytes, &directory[6]),
        &directory[6],
        &imported_axioms,
        &imported_theorems,
        &generated_clauses,
        context,
    )?;
    let derived_facts = parse_derived_facts(
        section_slice(payload_bytes, &directory[7]),
        &directory[7],
        &imported_axioms,
        &imported_theorems,
        &generated_clauses,
        &resolution_trace,
    )?;
    let final_goal = parse_final_goal(
        section_slice(payload_bytes, &directory[8]),
        &directory[8],
        &generated_clauses,
        &resolution_trace,
        &derived_facts,
    )?;

    Ok(ParsedCertificate {
        schema_version,
        encoding_version,
        kernel_profile,
        target_vc,
        symbol_manifest,
        variable_manifest,
        imported_axioms,
        imported_theorems,
        generated_clauses,
        substitutions,
        resolution_trace,
        derived_facts,
        final_goal,
        canonical_hash_input: bytes.to_vec(),
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct DirectoryEntry {
    section_tag: SectionTag,
    item_count: u32,
    payload_offset: u32,
    payload_length: u32,
    entry_offset: usize,
    payload_base_offset: usize,
}

fn validate_section_limit(
    entry: DirectoryEntry,
    context: &CertificateParseContext,
) -> Result<(), CertificateParseError> {
    let count = entry.item_count as usize;
    if entry.section_tag == SectionTag::FinalGoal && entry.item_count != 1 {
        return Err(malformed_at(entry.entry_offset + 1, "directory.item_count"));
    }
    let max_count = match entry.section_tag {
        SectionTag::SymbolManifest => context.max_symbol_manifest_entries,
        SectionTag::VariableManifest => context.max_variable_manifest_entries,
        SectionTag::ImportedAxioms | SectionTag::ImportedTheorems => context.max_imported_facts,
        SectionTag::GeneratedClauses => context.max_generated_clauses,
        SectionTag::Substitutions => context.max_substitutions,
        SectionTag::ResolutionTrace => context.max_resolution_steps,
        SectionTag::DerivedFacts => context.max_derived_facts,
        SectionTag::FinalGoal => 1,
    };
    if count > max_count {
        return Err(resource_error(
            entry.entry_offset + 1,
            "directory.item_count",
        ));
    }
    if entry.payload_length as usize > context.max_section_bytes {
        return Err(resource_error(
            entry.entry_offset + 9,
            "directory.payload_length",
        ));
    }
    Ok(())
}

fn validate_directory_ranges(
    directory: &[DirectoryEntry],
    payload_len: usize,
) -> Result<(), CertificateParseError> {
    let mut expected_offset = 0usize;
    for entry in directory {
        let payload_offset = entry.payload_offset as usize;
        let payload_length = entry.payload_length as usize;
        if payload_offset != expected_offset {
            return Err(malformed_at(
                entry.entry_offset + 5,
                "directory.payload_offset",
            ));
        }
        let Some(end) = payload_offset.checked_add(payload_length) else {
            return Err(resource_error(entry.entry_offset + 5, "directory.range"));
        };
        if end > payload_len {
            return Err(malformed_at(
                entry.entry_offset + 9,
                "directory.payload_length",
            ));
        }
        expected_offset = end;
    }
    if expected_offset != payload_len {
        let byte_offset = directory.first().map_or(expected_offset, |entry| {
            entry.payload_base_offset + expected_offset
        });
        return Err(malformed_at(byte_offset, "section_payloads.trailing_bytes"));
    }
    Ok(())
}

fn section_slice<'a>(payload_bytes: &'a [u8], entry: &DirectoryEntry) -> &'a [u8] {
    let start = entry.payload_offset as usize;
    let end = start + entry.payload_length as usize;
    &payload_bytes[start..end]
}

fn parse_symbol_manifest(
    bytes: &[u8],
    entry: &DirectoryEntry,
) -> Result<Vec<SymbolManifestEntry>, CertificateParseError> {
    let frames = read_section_frames(bytes, entry)?;
    let mut entries = Vec::with_capacity(frames.len());
    let mut seen = BTreeSet::new();
    for frame in frames {
        let mut reader = frame.reader();
        let kind = read_symbol_kind(&mut reader, "symbol_manifest.symbol_kind")?;
        let id = reader.read_u32("symbol_manifest.symbol_id")?;
        reader.finish()?;
        let symbol = SymbolKey::new(kind, id);
        if !seen.insert(symbol) {
            return Err(frame.malformed("symbol_manifest.duplicate"));
        }
        entries.push(SymbolManifestEntry { symbol });
    }
    ensure_sorted_by(
        entries.iter().map(|entry| entry.symbol),
        entry,
        "symbol_manifest",
    )?;
    Ok(entries)
}

fn parse_variable_manifest(
    bytes: &[u8],
    entry: &DirectoryEntry,
) -> Result<Vec<VariableManifestEntry>, CertificateParseError> {
    let frames = read_section_frames(bytes, entry)?;
    let mut entries = Vec::with_capacity(frames.len());
    let mut seen = BTreeSet::new();
    for frame in frames {
        let mut reader = frame.reader();
        let variable_id = VariableId(reader.read_u32("variable_manifest.variable_id")?);
        reader.finish()?;
        if !seen.insert(variable_id) {
            return Err(frame.malformed("variable_manifest.duplicate"));
        }
        entries.push(VariableManifestEntry { variable_id });
    }
    ensure_sorted_by(
        entries.iter().map(|entry| entry.variable_id),
        entry,
        "variable_manifest",
    )?;
    Ok(entries)
}

fn parse_imported_facts(
    bytes: &[u8],
    entry: &DirectoryEntry,
) -> Result<Vec<ImportedFactRef>, CertificateParseError> {
    let frames = read_section_frames(bytes, entry)?;
    let mut facts = Vec::with_capacity(frames.len());
    let mut seen_ids = BTreeSet::new();
    let mut seen_keys = BTreeSet::new();
    for frame in frames {
        let mut reader = frame.reader();
        let imported_fact_id = reader.read_u32("imported_fact.imported_fact_id")?;
        let package_id = reader.read_bytes("imported_fact.package_id")?;
        let module_path = reader.read_bytes("imported_fact.module_path")?;
        let exported_item_id = reader.read_bytes("imported_fact.exported_item_id")?;
        if package_id.is_empty() || module_path.is_empty() || exported_item_id.is_empty() {
            return Err(frame.malformed("imported_fact.stable_ref"));
        }
        let statement_fingerprint = read_fingerprint(&mut reader, "imported_fact.fingerprint")?;
        if statement_fingerprint.digest.is_empty() {
            return Err(frame.malformed("imported_fact.fingerprint"));
        }
        let status_tag = reader.read_u8("imported_fact.required_proof_status")?;
        let Some(required_proof_status) = RequiredProofStatus::from_tag(status_tag) else {
            return Err(reader.malformed_at(
                reader.offset().saturating_sub(1),
                "imported_fact.required_proof_status",
            ));
        };
        reader.finish()?;

        if !seen_ids.insert(imported_fact_id) {
            return Err(frame.malformed("imported_fact.duplicate_id"));
        }
        let stable_key = stable_import_key(&package_id, &module_path, &exported_item_id)?;
        if !seen_keys.insert(stable_key) {
            return Err(frame.malformed("imported_fact.duplicate_key"));
        }
        facts.push(ImportedFactRef {
            imported_fact_id,
            package_id,
            module_path,
            exported_item_id,
            statement_fingerprint,
            required_proof_status,
        });
    }
    ensure_sorted_by(
        facts.iter().map(|fact| fact.imported_fact_id),
        entry,
        "imported_fact",
    )?;
    Ok(facts)
}

fn parse_generated_clauses(
    bytes: &[u8],
    entry: &DirectoryEntry,
    profile: &KernelProfileRecord,
    symbols: &[SymbolManifestEntry],
    variables: &[VariableManifestEntry],
    context: &CertificateParseContext,
) -> Result<Vec<GeneratedClause>, CertificateParseError> {
    let frames = read_section_frames(bytes, entry)?;
    let mut generated = Vec::with_capacity(frames.len());
    let mut seen = BTreeSet::new();
    let clause_context = clause_context(profile, symbols, variables, context);
    for frame in frames {
        let mut reader = frame.reader();
        let clause_id = reader.read_u32("generated_clause.clause_id")?;
        let form = read_clause_form(&mut reader, "generated_clause.clause_form")?;
        let literal_count = reader.read_u32("generated_clause.literal_count")?;
        if literal_count as usize > context.clause_validation_policy.max_literals {
            return Err(frame.resource("generated_clause.literal_count"));
        }
        let mut literals = Vec::new();
        for _ in 0..literal_count {
            literals.push(read_literal(
                &mut reader,
                context,
                0,
                "generated_clause.literal",
            )?);
        }
        reader.finish()?;
        if !seen.insert(clause_id) {
            return Err(frame.malformed("generated_clause.duplicate_id"));
        }
        let clause = Clause::from_canonical_parts(form, literals, &clause_context)
            .map_err(|_| frame.malformed("generated_clause.clause"))?;
        generated.push(GeneratedClause { clause_id, clause });
    }
    ensure_sorted_by(
        generated.iter().map(|clause| clause.clause_id),
        entry,
        "generated_clause",
    )?;
    Ok(generated)
}

fn parse_substitutions(
    bytes: &[u8],
    entry: &DirectoryEntry,
    context: &CertificateParseContext,
) -> Result<Vec<SubstitutionEntry>, CertificateParseError> {
    let frames = read_section_frames(bytes, entry)?;
    let mut substitutions = Vec::with_capacity(frames.len());
    let mut seen = BTreeSet::new();
    for frame in frames {
        let mut reader = frame.reader();
        let substitution_id = reader.read_u32("substitution.substitution_id")?;
        let source_term = read_term(&mut reader, context, 0, "substitution.source_term")?;
        let target_term = read_term(&mut reader, context, 0, "substitution.target_term")?;
        let binder_context_encoding = reader.read_bytes("substitution.binder_context")?;
        let freshness_witness_refs =
            read_ref_list(&mut reader, "substitution.freshness_witness_refs")?;
        let free_variable_constraint_refs =
            read_ref_list(&mut reader, "substitution.free_variable_constraint_refs")?;
        reader.finish()?;
        if !seen.insert(substitution_id) {
            return Err(frame.malformed("substitution.duplicate_id"));
        }
        substitutions.push(SubstitutionEntry {
            substitution_id,
            source_term,
            target_term,
            binder_context_encoding,
            freshness_witness_refs,
            free_variable_constraint_refs,
        });
    }
    ensure_sorted_by(
        substitutions.iter().map(|entry| entry.substitution_id),
        entry,
        "substitution",
    )?;
    Ok(substitutions)
}

fn parse_resolution_trace(
    bytes: &[u8],
    entry: &DirectoryEntry,
    axioms: &[ImportedFactRef],
    theorems: &[ImportedFactRef],
    generated: &[GeneratedClause],
    context: &CertificateParseContext,
) -> Result<Vec<ResolutionStep>, CertificateParseError> {
    let frames = read_section_frames(bytes, entry)?;
    let axiom_ids = imported_ids(axioms);
    let theorem_ids = imported_ids(theorems);
    let generated_ids = generated_clause_ids(generated);
    let mut steps = Vec::with_capacity(frames.len());
    let mut seen_steps = BTreeSet::new();
    for frame in frames {
        let mut reader = frame.reader();
        let step_id = reader.read_u32("resolution.step_id")?;
        if !seen_steps.insert(step_id) {
            return Err(frame.malformed("resolution.duplicate_step"));
        }
        let parent_a = read_clause_ref(&mut reader, "resolution.parent_a")?;
        let parent_b = read_clause_ref(&mut reader, "resolution.parent_b")?;
        validate_parent_ref(
            parent_a,
            step_id,
            &axiom_ids,
            &theorem_ids,
            &generated_ids,
            &seen_steps,
        )
        .map_err(|_| frame.malformed("resolution.parent_a"))?;
        validate_parent_ref(
            parent_b,
            step_id,
            &axiom_ids,
            &theorem_ids,
            &generated_ids,
            &seen_steps,
        )
        .map_err(|_| frame.malformed("resolution.parent_b"))?;
        let pivot_literal = read_literal(&mut reader, context, 0, "resolution.pivot")?;
        let generated_clause = read_clause_ref(&mut reader, "resolution.generated_clause")?;
        if generated_clause.namespace != ClauseRefNamespace::GeneratedClause
            || !generated_ids.contains(&generated_clause.id)
        {
            return Err(frame.malformed("resolution.generated_clause"));
        }
        reader.finish()?;
        steps.push(ResolutionStep {
            step_id,
            parent_a,
            parent_b,
            pivot_literal,
            generated_clause,
        });
    }
    ensure_sorted_by(steps.iter().map(|step| step.step_id), entry, "resolution")?;
    Ok(steps)
}

fn parse_derived_facts(
    bytes: &[u8],
    entry: &DirectoryEntry,
    axioms: &[ImportedFactRef],
    theorems: &[ImportedFactRef],
    generated: &[GeneratedClause],
    steps: &[ResolutionStep],
) -> Result<Vec<DerivedFact>, CertificateParseError> {
    let frames = read_section_frames(bytes, entry)?;
    let axiom_ids = imported_ids(axioms);
    let theorem_ids = imported_ids(theorems);
    let generated_ids = generated_clause_ids(generated);
    let step_ids = resolution_step_ids(steps);
    let mut facts = Vec::with_capacity(frames.len());
    let mut seen = BTreeSet::new();
    for frame in frames {
        let mut reader = frame.reader();
        let derived_fact_id = reader.read_u32("derived_fact.derived_fact_id")?;
        let source = read_clause_ref(&mut reader, "derived_fact.source")?;
        validate_existing_clause_ref(source, &axiom_ids, &theorem_ids, &generated_ids, &step_ids)
            .map_err(|_| frame.malformed("derived_fact.source"))?;
        let payload = reader.read_bytes("derived_fact.payload")?;
        reader.finish()?;
        if !seen.insert(derived_fact_id) {
            return Err(frame.malformed("derived_fact.duplicate_id"));
        }
        facts.push(DerivedFact {
            derived_fact_id,
            source,
            payload,
        });
    }
    ensure_sorted_by(
        facts.iter().map(|fact| fact.derived_fact_id),
        entry,
        "derived_fact",
    )?;
    Ok(facts)
}

fn parse_final_goal(
    bytes: &[u8],
    entry: &DirectoryEntry,
    generated: &[GeneratedClause],
    steps: &[ResolutionStep],
    facts: &[DerivedFact],
) -> Result<FinalGoalRef, CertificateParseError> {
    let frames = read_section_frames(bytes, entry)?;
    let Some(frame) = frames.first() else {
        return Err(malformed_at(entry.entry_offset, "final_goal"));
    };
    let generated_ids = generated_clause_ids(generated);
    let step_ids = resolution_step_ids(steps);
    let fact_ids = derived_fact_ids(facts);
    let mut reader = frame.reader();
    let namespace_tag = reader.read_u8("final_goal.namespace")?;
    let Some(namespace) = FinalGoalNamespace::from_tag(namespace_tag) else {
        return Err(frame.malformed("final_goal.namespace"));
    };
    let id = reader.read_u32("final_goal.id")?;
    reader.finish()?;
    let exists = match namespace {
        FinalGoalNamespace::GeneratedClause => generated_ids.contains(&id),
        FinalGoalNamespace::ResolutionStep => step_ids.contains(&id),
        FinalGoalNamespace::DerivedFact => fact_ids.contains(&id),
    };
    if !exists {
        return Err(frame.malformed("final_goal.id"));
    }
    Ok(FinalGoalRef { namespace, id })
}

#[derive(Clone, Copy)]
struct ItemFrame<'a> {
    payload: &'a [u8],
    offset: usize,
    section_tag: SectionTag,
    item_index: u32,
}

impl<'a> ItemFrame<'a> {
    fn reader(self) -> Reader<'a> {
        Reader::new(
            self.payload,
            self.offset + 6,
            Some(self.section_tag),
            Some(self.item_index),
        )
    }

    fn malformed(self, field: &'static str) -> CertificateParseError {
        CertificateParseError::new(
            CertificateRejectionDetail::MalformedCertificate,
            CertificateParseLocation {
                byte_offset: self.offset,
                section_tag: Some(self.section_tag),
                item_index: Some(self.item_index),
                field_path: Some(field),
            },
        )
    }

    fn resource(self, field: &'static str) -> CertificateParseError {
        CertificateParseError::new(
            CertificateRejectionDetail::ResourceExhaustion,
            CertificateParseLocation {
                byte_offset: self.offset,
                section_tag: Some(self.section_tag),
                item_index: Some(self.item_index),
                field_path: Some(field),
            },
        )
    }
}

fn read_section_frames<'a>(
    bytes: &'a [u8],
    entry: &DirectoryEntry,
) -> Result<Vec<ItemFrame<'a>>, CertificateParseError> {
    let mut reader = Reader::new(
        bytes,
        entry.payload_base_offset + entry.payload_offset as usize,
        Some(entry.section_tag),
        None,
    );
    let mut frames = Vec::new();
    for item_index in 0..entry.item_count {
        reader.item_index = Some(item_index);
        let frame_offset = reader.offset();
        let section_tag_byte = reader.read_u8("item.section_tag")?;
        if section_tag_byte != entry.section_tag.byte() {
            return Err(CertificateParseError::new(
                CertificateRejectionDetail::MalformedCertificate,
                CertificateParseLocation {
                    byte_offset: frame_offset,
                    section_tag: Some(entry.section_tag),
                    item_index: Some(item_index),
                    field_path: Some("item.section_tag"),
                },
            ));
        }
        let item_tag = reader.read_u8("item.item_tag")?;
        if item_tag != 1 {
            return Err(CertificateParseError::new(
                CertificateRejectionDetail::MalformedCertificate,
                CertificateParseLocation {
                    byte_offset: frame_offset + 1,
                    section_tag: Some(entry.section_tag),
                    item_index: Some(item_index),
                    field_path: Some("item.item_tag"),
                },
            ));
        }
        let length = reader.read_u32("item.length")? as usize;
        let payload = reader.read_exact(length, "item.payload")?;
        frames.push(ItemFrame {
            payload,
            offset: frame_offset,
            section_tag: entry.section_tag,
            item_index,
        });
    }
    reader.item_index = None;
    reader.finish()?;
    Ok(frames)
}

fn read_kernel_profile(
    reader: &mut Reader<'_>,
) -> Result<KernelProfileRecord, CertificateParseError> {
    let profile_id = reader.read_u16("kernel_profile.profile_id")?;
    let clause_schema_version = reader.read_u16("kernel_profile.clause_schema_version")?;
    let clause_encoding_version = reader.read_u16("kernel_profile.clause_encoding_version")?;
    let tautology_tag = reader.read_u8("kernel_profile.clause_tautology_policy")?;
    let Some(clause_tautology_policy) = ClauseTautologyPolicy::from_tag(tautology_tag) else {
        return Err(unsupported_error(
            reader.offset().saturating_sub(1),
            "kernel_profile.clause_tautology_policy",
        ));
    };
    let hash_tag = reader.read_u8("kernel_profile.certificate_hash_input_algorithm")?;
    let Some(certificate_hash_input_algorithm) = CertificateHashInputAlgorithm::from_tag(hash_tag)
    else {
        return Err(unsupported_error(
            reader.offset().saturating_sub(1),
            "kernel_profile.certificate_hash_input_algorithm",
        ));
    };
    Ok(KernelProfileRecord {
        profile_id,
        clause_schema_version,
        clause_encoding_version,
        clause_tautology_policy,
        certificate_hash_input_algorithm,
    })
}

fn read_fingerprint(
    reader: &mut Reader<'_>,
    field: &'static str,
) -> Result<Fingerprint, CertificateParseError> {
    let algorithm_id = reader.read_u8(field)?;
    let digest = reader.read_bytes(field)?;
    Ok(Fingerprint {
        algorithm_id,
        digest,
    })
}

fn read_symbol_kind(
    reader: &mut Reader<'_>,
    field: &'static str,
) -> Result<SymbolKind, CertificateParseError> {
    let offset = reader.offset();
    let tag = reader.read_u8(field)?;
    let kind = match tag {
        1 => SymbolKind::Predicate,
        2 => SymbolKind::FunctorPredicate,
        3 => SymbolKind::Equality,
        4 => SymbolKind::BuiltinRelation,
        _ => return Err(reader.malformed_at(offset, field)),
    };
    Ok(kind)
}

fn read_clause_form(
    reader: &mut Reader<'_>,
    field: &'static str,
) -> Result<ClauseForm, CertificateParseError> {
    let offset = reader.offset();
    let tag = reader.read_u8(field)?;
    let form = match tag {
        1 => ClauseForm::Ordinary,
        2 => ClauseForm::Empty,
        3 => ClauseForm::Tautology,
        _ => return Err(reader.malformed_at(offset, field)),
    };
    Ok(form)
}

fn read_literal(
    reader: &mut Reader<'_>,
    context: &CertificateParseContext,
    depth: usize,
    field: &'static str,
) -> Result<Literal, CertificateParseError> {
    let polarity_offset = reader.offset();
    let polarity_tag = reader.read_u8(field)?;
    let polarity = match polarity_tag {
        1 => Polarity::Negative,
        2 => Polarity::Positive,
        _ => return Err(reader.malformed_at(polarity_offset, field)),
    };
    let atom = read_atom(reader, context, depth, field)?;
    Ok(Literal::new(polarity, atom))
}

fn read_atom(
    reader: &mut Reader<'_>,
    context: &CertificateParseContext,
    depth: usize,
    field: &'static str,
) -> Result<Atom, CertificateParseError> {
    let kind = read_symbol_kind(reader, field)?;
    let symbol_id = reader.read_u32(field)?;
    let arity = reader.read_u32(field)?;
    let term_count = reader.read_u32(field)?;
    let mut arguments = Vec::new();
    for _ in 0..term_count {
        arguments.push(read_term(reader, context, depth + 1, field)?);
    }
    Ok(Atom::with_arity(
        SymbolKey::new(kind, symbol_id),
        arity,
        arguments,
    ))
}

fn read_term(
    reader: &mut Reader<'_>,
    context: &CertificateParseContext,
    depth: usize,
    field: &'static str,
) -> Result<Term, CertificateParseError> {
    let term_start = reader.offset();
    if depth > context.max_term_recursion_depth {
        return Err(reader.resource_at(reader.offset(), field));
    }
    let tag_offset = reader.offset();
    let tag = reader.read_u8(field)?;
    match tag {
        1 => {
            let term = Term::Variable(VariableId(reader.read_u32(field)?));
            validate_term_record_size(reader, term_start, context, field)?;
            Ok(term)
        }
        2 => {
            let kind = read_symbol_kind(reader, field)?;
            let symbol_id = reader.read_u32(field)?;
            let term_count = reader.read_u32(field)?;
            ensure_child_term_budget(reader, term_count, 10, context, field)?;
            let mut arguments = Vec::new();
            for _ in 0..term_count {
                arguments.push(read_term(reader, context, depth + 1, field)?);
                validate_term_record_size(reader, term_start, context, field)?;
            }
            let term = Term::Application {
                symbol: SymbolKey::new(kind, symbol_id),
                arguments,
            };
            validate_term_record_size(reader, term_start, context, field)?;
            Ok(term)
        }
        3 => {
            let binder_id = reader.read_u32(field)?;
            let body = read_term(reader, context, depth + 1, field)?;
            let term = Term::BinderNormalized {
                binder_id,
                body: Box::new(body),
            };
            validate_term_record_size(reader, term_start, context, field)?;
            Ok(term)
        }
        _ => Err(reader.malformed_at(tag_offset, field)),
    }
}

fn read_ref_list(
    reader: &mut Reader<'_>,
    field: &'static str,
) -> Result<Vec<u32>, CertificateParseError> {
    let count = reader.read_u32(field)?;
    let mut ids = Vec::new();
    let mut seen = BTreeSet::new();
    for _ in 0..count {
        let id = reader.read_u32(field)?;
        if !seen.insert(id) {
            return Err(reader.malformed_at(reader.offset().saturating_sub(4), field));
        }
        ids.push(id);
    }
    ensure_sorted_values(&ids, reader, field)?;
    Ok(ids)
}

fn read_clause_ref(
    reader: &mut Reader<'_>,
    field: &'static str,
) -> Result<ClauseRef, CertificateParseError> {
    let namespace_offset = reader.offset();
    let namespace_tag = reader.read_u8(field)?;
    let Some(namespace) = ClauseRefNamespace::from_tag(namespace_tag) else {
        return Err(reader.malformed_at(namespace_offset, field));
    };
    let id = reader.read_u32(field)?;
    Ok(ClauseRef { namespace, id })
}

fn clause_context(
    profile: &KernelProfileRecord,
    symbols: &[SymbolManifestEntry],
    variables: &[VariableManifestEntry],
    context: &CertificateParseContext,
) -> ClauseValidationContext {
    let mut clause_context = ClauseValidationContext::new(profile.clause_profile()).with_limits(
        context.clause_validation_policy.max_literals,
        context.clause_validation_policy.max_term_encoding_bytes,
    );
    for symbol in symbols {
        clause_context = clause_context.with_known_symbol(symbol.symbol);
    }
    for variable in variables {
        clause_context = clause_context.with_canonical_variable(variable.variable_id);
    }
    clause_context
}

fn stable_import_key(
    package_id: &[u8],
    module_path: &[u8],
    exported_item_id: &[u8],
) -> Result<Vec<u8>, CertificateParseError> {
    let mut key = Vec::new();
    write_len(package_id.len(), &mut key)?;
    key.extend_from_slice(package_id);
    write_len(module_path.len(), &mut key)?;
    key.extend_from_slice(module_path);
    write_len(exported_item_id.len(), &mut key)?;
    key.extend_from_slice(exported_item_id);
    Ok(key)
}

fn validate_parent_ref(
    reference: ClauseRef,
    current_step_id: u32,
    axiom_ids: &BTreeSet<u32>,
    theorem_ids: &BTreeSet<u32>,
    generated_ids: &BTreeSet<u32>,
    earlier_step_ids: &BTreeSet<u32>,
) -> Result<(), ()> {
    match reference.namespace {
        ClauseRefNamespace::GeneratedClause => generated_ids.contains(&reference.id).then_some(()),
        ClauseRefNamespace::ResolutionStep => (reference.id < current_step_id
            && earlier_step_ids.contains(&reference.id))
        .then_some(()),
        ClauseRefNamespace::ImportedAxiom => axiom_ids.contains(&reference.id).then_some(()),
        ClauseRefNamespace::ImportedTheorem => theorem_ids.contains(&reference.id).then_some(()),
    }
    .ok_or(())
}

fn validate_existing_clause_ref(
    reference: ClauseRef,
    axiom_ids: &BTreeSet<u32>,
    theorem_ids: &BTreeSet<u32>,
    generated_ids: &BTreeSet<u32>,
    step_ids: &BTreeSet<u32>,
) -> Result<(), ()> {
    match reference.namespace {
        ClauseRefNamespace::GeneratedClause => generated_ids.contains(&reference.id).then_some(()),
        ClauseRefNamespace::ResolutionStep => step_ids.contains(&reference.id).then_some(()),
        ClauseRefNamespace::ImportedAxiom => axiom_ids.contains(&reference.id).then_some(()),
        ClauseRefNamespace::ImportedTheorem => theorem_ids.contains(&reference.id).then_some(()),
    }
    .ok_or(())
}

fn imported_ids(facts: &[ImportedFactRef]) -> BTreeSet<u32> {
    facts.iter().map(|fact| fact.imported_fact_id).collect()
}

fn generated_clause_ids(generated: &[GeneratedClause]) -> BTreeSet<u32> {
    generated.iter().map(|clause| clause.clause_id).collect()
}

fn resolution_step_ids(steps: &[ResolutionStep]) -> BTreeSet<u32> {
    steps.iter().map(|step| step.step_id).collect()
}

fn derived_fact_ids(facts: &[DerivedFact]) -> BTreeSet<u32> {
    facts.iter().map(|fact| fact.derived_fact_id).collect()
}

fn ensure_sorted_by<T>(
    values: impl Iterator<Item = T>,
    entry: &DirectoryEntry,
    field: &'static str,
) -> Result<(), CertificateParseError>
where
    T: Ord,
{
    let mut previous = None;
    for value in values {
        if previous.as_ref().is_some_and(|previous| previous >= &value) {
            return Err(malformed_at(entry.entry_offset, field));
        }
        previous = Some(value);
    }
    Ok(())
}

fn ensure_sorted_values(
    values: &[u32],
    reader: &Reader<'_>,
    field: &'static str,
) -> Result<(), CertificateParseError> {
    if values.windows(2).all(|window| window[0] < window[1]) {
        Ok(())
    } else {
        Err(reader.malformed_at(reader.offset(), field))
    }
}

fn validate_term_record_size(
    reader: &Reader<'_>,
    term_start: usize,
    context: &CertificateParseContext,
    field: &'static str,
) -> Result<(), CertificateParseError> {
    let Some(size) = reader.offset().checked_sub(term_start) else {
        return Err(reader.resource_at(reader.offset(), field));
    };
    if size > context.clause_validation_policy.max_term_encoding_bytes {
        return Err(reader.resource_at(term_start, field));
    }
    Ok(())
}

fn ensure_child_term_budget(
    reader: &Reader<'_>,
    term_count: u32,
    parent_header_bytes: usize,
    context: &CertificateParseContext,
    field: &'static str,
) -> Result<(), CertificateParseError> {
    let term_count = usize::try_from(term_count)
        .map_err(|_| reader.resource_at(reader.offset().saturating_sub(4), field))?;
    let Some(min_children_bytes) = term_count.checked_mul(5) else {
        return Err(reader.resource_at(reader.offset().saturating_sub(4), field));
    };
    let Some(min_record_bytes) = parent_header_bytes.checked_add(min_children_bytes) else {
        return Err(reader.resource_at(reader.offset().saturating_sub(4), field));
    };
    if min_record_bytes > context.clause_validation_policy.max_term_encoding_bytes {
        return Err(reader.resource_at(reader.offset().saturating_sub(4), field));
    }
    Ok(())
}

fn write_len(len: usize, bytes: &mut Vec<u8>) -> Result<(), CertificateParseError> {
    let len = u32::try_from(len).map_err(|_| resource_error(0, "length"))?;
    bytes.extend_from_slice(&len.to_be_bytes());
    Ok(())
}

#[derive(Clone)]
struct Reader<'a> {
    bytes: &'a [u8],
    cursor: usize,
    base_offset: usize,
    section_tag: Option<SectionTag>,
    item_index: Option<u32>,
}

impl<'a> Reader<'a> {
    const fn new(
        bytes: &'a [u8],
        base_offset: usize,
        section_tag: Option<SectionTag>,
        item_index: Option<u32>,
    ) -> Self {
        Self {
            bytes,
            cursor: 0,
            base_offset,
            section_tag,
            item_index,
        }
    }

    const fn offset(&self) -> usize {
        self.base_offset + self.cursor
    }

    fn read_u8(&mut self, field: &'static str) -> Result<u8, CertificateParseError> {
        let bytes = self.read_exact(1, field)?;
        Ok(bytes[0])
    }

    fn read_u16(&mut self, field: &'static str) -> Result<u16, CertificateParseError> {
        let bytes = self.read_exact(2, field)?;
        Ok(u16::from_be_bytes([bytes[0], bytes[1]]))
    }

    fn read_u32(&mut self, field: &'static str) -> Result<u32, CertificateParseError> {
        let bytes = self.read_exact(4, field)?;
        Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn read_bytes(&mut self, field: &'static str) -> Result<Vec<u8>, CertificateParseError> {
        let len_offset = self.offset();
        let len = self.read_u32(field)? as usize;
        let bytes = self.read_exact(len, field)?;
        if len > self.bytes.len() {
            return Err(self.resource_at(len_offset, field));
        }
        Ok(bytes.to_vec())
    }

    fn read_exact(
        &mut self,
        len: usize,
        field: &'static str,
    ) -> Result<&'a [u8], CertificateParseError> {
        let start = self.cursor;
        let Some(end) = start.checked_add(len) else {
            return Err(self.resource_at(self.offset(), field));
        };
        if end > self.bytes.len() {
            return Err(CertificateParseError::new(
                CertificateRejectionDetail::MalformedCertificate,
                CertificateParseLocation {
                    byte_offset: self.offset(),
                    section_tag: self.section_tag,
                    item_index: self.item_index,
                    field_path: Some(field),
                },
            ));
        }
        self.cursor = end;
        Ok(&self.bytes[start..end])
    }

    fn finish(&self) -> Result<(), CertificateParseError> {
        if self.cursor == self.bytes.len() {
            Ok(())
        } else {
            Err(CertificateParseError::new(
                CertificateRejectionDetail::MalformedCertificate,
                CertificateParseLocation {
                    byte_offset: self.offset(),
                    section_tag: self.section_tag,
                    item_index: self.item_index,
                    field_path: Some("trailing_bytes"),
                },
            ))
        }
    }

    fn malformed_at(&self, byte_offset: usize, field: &'static str) -> CertificateParseError {
        CertificateParseError::new(
            CertificateRejectionDetail::MalformedCertificate,
            CertificateParseLocation {
                byte_offset,
                section_tag: self.section_tag,
                item_index: self.item_index,
                field_path: Some(field),
            },
        )
    }

    fn resource_at(&self, byte_offset: usize, field: &'static str) -> CertificateParseError {
        CertificateParseError::new(
            CertificateRejectionDetail::ResourceExhaustion,
            CertificateParseLocation {
                byte_offset,
                section_tag: self.section_tag,
                item_index: self.item_index,
                field_path: Some(field),
            },
        )
    }
}

fn unsupported_error(offset: usize, field: &'static str) -> CertificateParseError {
    CertificateParseError::new(
        CertificateRejectionDetail::UnsupportedCertificateFormat,
        CertificateParseLocation::new(offset).with_field(field),
    )
}

fn malformed_at(offset: usize, field: &'static str) -> CertificateParseError {
    CertificateParseError::new(
        CertificateRejectionDetail::MalformedCertificate,
        CertificateParseLocation::new(offset).with_field(field),
    )
}

fn resource_error(offset: usize, field: &'static str) -> CertificateParseError {
    CertificateParseError::new(
        CertificateRejectionDetail::ResourceExhaustion,
        CertificateParseLocation::new(offset).with_field(field),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const DIRECTORY_ENTRY_LEN: usize = 13;

    #[test]
    fn parses_minimal_valid_certificate_and_hash_input() {
        let (bytes, context) = fixture();

        let parsed = parse_certificate(&bytes, &context).expect("valid certificate");

        assert_eq!(parsed.schema_version, 1);
        assert_eq!(parsed.encoding_version, 1);
        assert_eq!(parsed.generated_clauses.len(), 1);
        assert_eq!(parsed.final_goal.id, 1);
        assert_eq!(parsed.canonical_hash_input(), bytes.as_slice());
        assert!(parsed.canonical_hash_input().starts_with(DOMAIN_SEPARATOR));
        assert!(contains_subsequence(
            parsed.canonical_hash_input(),
            &1_u16.to_be_bytes()
        ));
        assert!(contains_subsequence(
            parsed.canonical_hash_input(),
            &context
                .expected_target_vc
                .canonical_bytes()
                .expect("fingerprint bytes")
        ));
    }

    #[test]
    fn rejects_unsupported_header_profile_and_context_mismatch() {
        let (bytes, context) = fixture();

        let mut bad_schema = bytes.clone();
        bad_schema[DOMAIN_SEPARATOR.len() + 1] = 2;
        assert_detail(
            parse_certificate(&bad_schema, &context),
            CertificateRejectionDetail::UnsupportedCertificateFormat,
        );

        let mut bad_encoding = bytes.clone();
        bad_encoding[DOMAIN_SEPARATOR.len() + 3] = 2;
        assert_detail(
            parse_certificate(&bad_encoding, &context),
            CertificateRejectionDetail::UnsupportedCertificateFormat,
        );

        let mut bad_profile = bytes.clone();
        bad_profile[DOMAIN_SEPARATOR.len() + 5] = 2;
        assert_detail(
            parse_certificate(&bad_profile, &context),
            CertificateRejectionDetail::UnsupportedCertificateFormat,
        );

        let mut bad_hash_algorithm = bytes.clone();
        let hash_algorithm_offset = DOMAIN_SEPARATOR.len() + 4 + 7;
        bad_hash_algorithm[hash_algorithm_offset] = 9;
        assert_detail(
            parse_certificate(&bad_hash_algorithm, &context),
            CertificateRejectionDetail::UnsupportedCertificateFormat,
        );

        let mismatched_context =
            CertificateParseContext::v1(Fingerprint::new(1, vec![9]), sample_profile());
        assert_detail(
            parse_certificate(&bytes, &mismatched_context),
            CertificateRejectionDetail::ContextMismatch,
        );
    }

    #[test]
    fn rejects_directory_and_item_canonicality_errors_with_locations() {
        let (bytes, context) = fixture();
        let directory_start = directory_start(&context.expected_target_vc);

        let mut unknown_section = bytes.clone();
        unknown_section[directory_start] = 0xff;
        assert_rejection_location(
            parse_certificate(&unknown_section, &context),
            CertificateRejectionDetail::UnsupportedCertificateFormat,
            directory_start,
            None,
            None,
            "directory.section_tag",
        );

        let mut out_of_order = bytes.clone();
        out_of_order[directory_start] = SectionTag::VariableManifest.byte();
        assert_rejection_location(
            parse_certificate(&out_of_order, &context),
            CertificateRejectionDetail::MalformedCertificate,
            directory_start,
            None,
            None,
            "directory.section_tag",
        );

        let mut missing = bytes.clone();
        set_u32(&mut missing, directory_start - 4, 8);
        assert_rejection_location(
            parse_certificate(&missing, &context),
            CertificateRejectionDetail::MalformedCertificate,
            directory_start - 4,
            None,
            None,
            "directory_entry_count",
        );

        let mut duplicate = bytes.clone();
        duplicate[directory_start + DIRECTORY_ENTRY_LEN] = SectionTag::SymbolManifest.byte();
        assert_rejection_location(
            parse_certificate(&duplicate, &context),
            CertificateRejectionDetail::MalformedCertificate,
            directory_start + DIRECTORY_ENTRY_LEN,
            None,
            None,
            "directory.section_tag",
        );

        let mut non_contiguous = bytes.clone();
        let variable_entry =
            directory_entry_start(&context.expected_target_vc, SectionTag::VariableManifest);
        set_u32(
            &mut non_contiguous,
            variable_entry + 5,
            section_payload_offset(
                &bytes,
                &context.expected_target_vc,
                SectionTag::VariableManifest,
            ) + 1,
        );
        assert_rejection_location(
            parse_certificate(&non_contiguous, &context),
            CertificateRejectionDetail::MalformedCertificate,
            variable_entry + 5,
            None,
            None,
            "directory.payload_offset",
        );

        let mut overlapping = bytes.clone();
        set_u32(&mut overlapping, variable_entry + 5, 0);
        assert_rejection_location(
            parse_certificate(&overlapping, &context),
            CertificateRejectionDetail::MalformedCertificate,
            variable_entry + 5,
            None,
            None,
            "directory.payload_offset",
        );

        let mut item_tag = bytes.clone();
        let payload_start = directory_start + REQUIRED_SECTIONS.len() * DIRECTORY_ENTRY_LEN;
        item_tag[payload_start + 1] = 2;
        assert_rejection_location(
            parse_certificate(&item_tag, &context),
            CertificateRejectionDetail::MalformedCertificate,
            payload_start + 1,
            Some(SectionTag::SymbolManifest),
            Some(0),
            "item.item_tag",
        );

        let mut section_tag = bytes.clone();
        section_tag[payload_start] = SectionTag::VariableManifest.byte();
        assert_rejection_location(
            parse_certificate(&section_tag, &context),
            CertificateRejectionDetail::MalformedCertificate,
            payload_start,
            Some(SectionTag::SymbolManifest),
            Some(0),
            "item.section_tag",
        );

        let mut item_count = bytes.clone();
        set_u32(&mut item_count, variable_entry + 1, 2);
        let variable_payload_end = payload_start
            + section_payload_offset(
                &bytes,
                &context.expected_target_vc,
                SectionTag::VariableManifest,
            ) as usize
            + section_payload_length(
                &bytes,
                &context.expected_target_vc,
                SectionTag::VariableManifest,
            ) as usize;
        assert_rejection_location(
            parse_certificate(&item_count, &context),
            CertificateRejectionDetail::MalformedCertificate,
            variable_payload_end,
            Some(SectionTag::VariableManifest),
            Some(1),
            "item.section_tag",
        );

        let mut malformed_field = CertificateBuilder::minimal();
        malformed_field.variable_manifest = vec![{
            let mut bytes = variable_manifest_entry(1);
            bytes.push(0);
            bytes
        }];
        assert_rejection_location(
            parse_certificate(&malformed_field.finish(), &context),
            CertificateRejectionDetail::MalformedCertificate,
            variable_payload_end,
            Some(SectionTag::VariableManifest),
            Some(0),
            "trailing_bytes",
        );

        let mut range_out_of_bounds = bytes.clone();
        set_u32(&mut range_out_of_bounds, variable_entry + 9, u32::MAX);
        let unbounded_section_context = context.clone().with_limits(CertificateParseLimits {
            max_section_bytes: usize::MAX,
            ..CertificateParseLimits::default()
        });
        assert_rejection_location(
            parse_certificate(&range_out_of_bounds, &unbounded_section_context),
            CertificateRejectionDetail::MalformedCertificate,
            variable_entry + 9,
            None,
            None,
            "directory.payload_length",
        );

        let mut truncated = bytes.clone();
        truncated.pop();
        assert_rejection_location(
            parse_certificate(&truncated, &context),
            CertificateRejectionDetail::MalformedCertificate,
            directory_entry_start(&context.expected_target_vc, SectionTag::FinalGoal) + 9,
            None,
            None,
            "directory.payload_length",
        );

        let mut trailing = bytes.clone();
        trailing.push(0);
        assert_rejection_location(
            parse_certificate(&trailing, &context),
            CertificateRejectionDetail::MalformedCertificate,
            bytes.len(),
            None,
            None,
            "section_payloads.trailing_bytes",
        );
    }

    #[test]
    fn rejects_resource_exhaustion_before_large_allocation() {
        let (bytes, context) = fixture();
        let limits = CertificateParseLimits {
            max_certificate_bytes: bytes.len() - 1,
            ..CertificateParseLimits::default()
        };
        assert_detail(
            parse_certificate(&bytes, &context.clone().with_limits(limits)),
            CertificateRejectionDetail::ResourceExhaustion,
        );

        let limits = CertificateParseLimits {
            max_section_bytes: 0,
            ..CertificateParseLimits::default()
        };
        assert_detail(
            parse_certificate(&bytes, &context.clone().with_limits(limits)),
            CertificateRejectionDetail::ResourceExhaustion,
        );

        let limits = CertificateParseLimits {
            max_symbol_manifest_entries: 1,
            ..CertificateParseLimits::default()
        };
        assert_detail(
            parse_certificate(&bytes, &context.clone().with_limits(limits)),
            CertificateRejectionDetail::ResourceExhaustion,
        );

        let mut huge_section = bytes.clone();
        set_u32(
            &mut huge_section,
            directory_entry_start(&context.expected_target_vc, SectionTag::SymbolManifest) + 9,
            u32::MAX,
        );
        let limits = CertificateParseLimits {
            max_section_bytes: 32,
            ..CertificateParseLimits::default()
        };
        assert_detail(
            parse_certificate(&huge_section, &context.clone().with_limits(limits)),
            CertificateRejectionDetail::ResourceExhaustion,
        );

        let deep = certificate_with_substitution_term_depth(3);
        let limits = CertificateParseLimits {
            max_term_recursion_depth: 1,
            ..CertificateParseLimits::default()
        };
        assert_detail(
            parse_certificate(&deep, &context.clone().with_limits(limits)),
            CertificateRejectionDetail::ResourceExhaustion,
        );

        let mut small_atom_arguments = CertificateBuilder::minimal();
        small_atom_arguments.generated_clauses = vec![generated_clause_with_literals(
            1,
            1,
            vec![literal_record(
                2,
                1,
                1,
                vec![variable_term(1), variable_term(1)],
            )],
        )];
        let small_term_context =
            context
                .clone()
                .with_clause_validation_policy(ClauseValidationPolicy {
                    max_literals: 8,
                    max_term_encoding_bytes: 16,
                });
        assert!(parse_certificate(&small_atom_arguments.finish(), &small_term_context).is_ok());

        let mut term_budget = CertificateBuilder::minimal();
        term_budget.generated_clauses = vec![generated_clause_with_literals(
            1,
            1,
            vec![literal_record(
                2,
                1,
                1,
                vec![application_term(
                    1,
                    1,
                    vec![
                        nested_term(1),
                        nested_term(1),
                        nested_term(1),
                        nested_term(1),
                    ],
                )],
            )],
        )];
        let parent_budget_context =
            context
                .clone()
                .with_clause_validation_policy(ClauseValidationPolicy {
                    max_literals: 8,
                    max_term_encoding_bytes: 35,
                });
        assert_detail(
            parse_certificate(&term_budget.finish(), &parent_budget_context),
            CertificateRejectionDetail::ResourceExhaustion,
        );
    }

    #[test]
    fn rejects_imported_fact_reference_errors() {
        let mut builder = CertificateBuilder::minimal();
        builder.imported_axioms = vec![imported_fact(1, b"pkg", b"mod", b"item")];
        let (bytes, context) = (builder.finish(), sample_context());
        let parsed = parse_certificate(&bytes, &context).expect("valid imported fact");
        assert_eq!(parsed.imported_axioms.len(), 1);

        let mut theorem_builder = CertificateBuilder::minimal();
        theorem_builder.imported_theorems = vec![imported_fact(1, b"pkg", b"mod", b"thm")];
        assert_eq!(
            parse_certificate(&theorem_builder.finish(), &context)
                .expect("valid theorem")
                .imported_theorems
                .len(),
            1
        );

        let mut unsorted_id = CertificateBuilder::minimal();
        unsorted_id.imported_axioms = vec![
            imported_fact(2, b"pkg", b"mod", b"item2"),
            imported_fact(1, b"pkg", b"mod", b"item1"),
        ];
        assert_detail(
            parse_certificate(&unsorted_id.finish(), &context),
            CertificateRejectionDetail::MalformedCertificate,
        );

        let mut duplicate_id = CertificateBuilder::minimal();
        duplicate_id.imported_axioms = vec![
            imported_fact(1, b"pkg", b"mod", b"item1"),
            imported_fact(1, b"pkg", b"mod", b"item2"),
        ];
        assert_detail(
            parse_certificate(&duplicate_id.finish(), &context),
            CertificateRejectionDetail::MalformedCertificate,
        );

        let mut duplicate_key = CertificateBuilder::minimal();
        duplicate_key.imported_axioms = vec![
            imported_fact(1, b"pkg", b"mod", b"item"),
            imported_fact(2, b"pkg", b"mod", b"item"),
        ];
        assert_detail(
            parse_certificate(&duplicate_key.finish(), &context),
            CertificateRejectionDetail::MalformedCertificate,
        );

        let malformed = imported_fact(1, b"", b"mod", b"item");
        let mut builder = CertificateBuilder::minimal();
        builder.imported_axioms = vec![malformed];
        assert_detail(
            parse_certificate(&builder.finish(), &context),
            CertificateRejectionDetail::MalformedCertificate,
        );

        for malformed in [
            imported_fact(1, b"pkg", b"", b"item"),
            imported_fact(1, b"pkg", b"mod", b""),
            imported_fact_with_fingerprint(1, b"pkg", b"mod", b"item", Vec::new(), 1),
            imported_fact_with_fingerprint(1, b"pkg", b"mod", b"item", vec![7], 9),
        ] {
            let mut builder = CertificateBuilder::minimal();
            builder.imported_axioms = vec![malformed];
            assert_detail(
                parse_certificate(&builder.finish(), &context),
                CertificateRejectionDetail::MalformedCertificate,
            );
        }
    }

    #[test]
    fn rejects_manifest_and_generated_clause_errors() {
        let (bytes, context) = fixture();
        assert!(parse_certificate(&bytes, &context).is_ok());

        let mut missing_variable = CertificateBuilder::minimal();
        missing_variable.variable_manifest.clear();
        assert_detail(
            parse_certificate(&missing_variable.finish(), &context),
            CertificateRejectionDetail::MalformedCertificate,
        );

        let mut duplicate_symbol = CertificateBuilder::minimal();
        duplicate_symbol
            .symbol_manifest
            .push(symbol_manifest_entry(1, 1));
        assert_detail(
            parse_certificate(&duplicate_symbol.finish(), &context),
            CertificateRejectionDetail::MalformedCertificate,
        );

        let mut unsorted_symbol = CertificateBuilder::minimal();
        unsorted_symbol.symbol_manifest =
            vec![symbol_manifest_entry(1, 2), symbol_manifest_entry(1, 1)];
        assert_detail(
            parse_certificate(&unsorted_symbol.finish(), &context),
            CertificateRejectionDetail::MalformedCertificate,
        );

        let mut duplicate_variable = CertificateBuilder::minimal();
        duplicate_variable
            .variable_manifest
            .push(variable_manifest_entry(1));
        assert_detail(
            parse_certificate(&duplicate_variable.finish(), &context),
            CertificateRejectionDetail::MalformedCertificate,
        );

        let mut unsorted_variable = CertificateBuilder::minimal();
        unsorted_variable.variable_manifest =
            vec![variable_manifest_entry(2), variable_manifest_entry(1)];
        assert_detail(
            parse_certificate(&unsorted_variable.finish(), &context),
            CertificateRejectionDetail::MalformedCertificate,
        );

        let mut unsupported_kind = CertificateBuilder::minimal();
        unsupported_kind.symbol_manifest = vec![symbol_manifest_entry(9, 1)];
        assert_detail(
            parse_certificate(&unsupported_kind.finish(), &context),
            CertificateRejectionDetail::MalformedCertificate,
        );

        let mut missing_symbol = CertificateBuilder::minimal();
        missing_symbol.symbol_manifest = vec![symbol_manifest_entry(1, 2)];
        assert_detail(
            parse_certificate(&missing_symbol.finish(), &context),
            CertificateRejectionDetail::MalformedCertificate,
        );

        let mut duplicate_clause = CertificateBuilder::minimal();
        duplicate_clause
            .generated_clauses
            .push(generated_clause_with_literals(
                1,
                1,
                vec![literal_record(2, 1, 2, vec![])],
            ));
        assert_detail(
            parse_certificate(&duplicate_clause.finish(), &context),
            CertificateRejectionDetail::MalformedCertificate,
        );

        let mut unsorted_clause = CertificateBuilder::minimal();
        unsorted_clause.generated_clauses = vec![
            generated_clause_with_literals(
                2,
                1,
                vec![literal_record(2, 1, 1, vec![variable_term(1)])],
            ),
            generated_clause_with_literals(1, 1, vec![literal_record(2, 1, 2, vec![])]),
        ];
        assert_detail(
            parse_certificate(&unsorted_clause.finish(), &context),
            CertificateRejectionDetail::MalformedCertificate,
        );

        let mut noncanonical = CertificateBuilder::minimal();
        noncanonical.generated_clauses = vec![generated_clause_with_literals(
            1,
            1,
            vec![
                literal_record(2, 1, 2, vec![]),
                literal_record(2, 1, 1, vec![]),
            ],
        )];
        assert_detail(
            parse_certificate(&noncanonical.finish(), &context),
            CertificateRejectionDetail::MalformedCertificate,
        );
    }

    #[test]
    fn validates_substitution_resolution_derived_and_final_refs() {
        let context = sample_context();

        let mut valid = CertificateBuilder::minimal();
        valid.imported_axioms = vec![imported_fact(3, b"pkg", b"mod", b"item")];
        valid.resolution_trace = vec![resolution_step(1, clause_ref(3, 3), clause_ref(1, 1), 1)];
        valid.derived_facts = vec![derived_fact(1, clause_ref(2, 1))];
        valid.final_goal = final_goal(3, 1);
        valid.substitutions = vec![substitution_entry(1, vec![1, 3])];
        assert!(parse_certificate(&valid.finish(), &context).is_ok());

        let mut unsorted_refs = CertificateBuilder::minimal();
        unsorted_refs.substitutions = vec![substitution_entry(1, vec![3, 1])];
        assert_detail(
            parse_certificate(&unsorted_refs.finish(), &context),
            CertificateRejectionDetail::MalformedCertificate,
        );

        let mut duplicate_substitution = CertificateBuilder::minimal();
        duplicate_substitution.substitutions = vec![
            substitution_entry(1, vec![1]),
            substitution_entry(1, vec![2]),
        ];
        assert_detail(
            parse_certificate(&duplicate_substitution.finish(), &context),
            CertificateRejectionDetail::MalformedCertificate,
        );

        let mut unsorted_substitution = CertificateBuilder::minimal();
        unsorted_substitution.substitutions = vec![
            substitution_entry(2, vec![1]),
            substitution_entry(1, vec![2]),
        ];
        assert_detail(
            parse_certificate(&unsorted_substitution.finish(), &context),
            CertificateRejectionDetail::MalformedCertificate,
        );

        let mut duplicate_free_variable_refs = CertificateBuilder::minimal();
        duplicate_free_variable_refs.substitutions =
            vec![substitution_entry_with_refs(1, vec![1], vec![2, 2])];
        assert_detail(
            parse_certificate(&duplicate_free_variable_refs.finish(), &context),
            CertificateRejectionDetail::MalformedCertificate,
        );

        let mut forward = CertificateBuilder::minimal();
        forward.resolution_trace = vec![resolution_step(1, clause_ref(2, 2), clause_ref(1, 1), 1)];
        assert_detail(
            parse_certificate(&forward.finish(), &context),
            CertificateRejectionDetail::MalformedCertificate,
        );

        let mut duplicate_step = CertificateBuilder::minimal();
        duplicate_step.resolution_trace = vec![
            resolution_step(1, clause_ref(1, 1), clause_ref(1, 1), 1),
            resolution_step(1, clause_ref(1, 1), clause_ref(1, 1), 1),
        ];
        assert_detail(
            parse_certificate(&duplicate_step.finish(), &context),
            CertificateRejectionDetail::MalformedCertificate,
        );

        let mut unsorted_step = CertificateBuilder::minimal();
        unsorted_step.resolution_trace = vec![
            resolution_step(2, clause_ref(1, 1), clause_ref(1, 1), 1),
            resolution_step(1, clause_ref(1, 1), clause_ref(1, 1), 1),
        ];
        assert_detail(
            parse_certificate(&unsorted_step.finish(), &context),
            CertificateRejectionDetail::MalformedCertificate,
        );

        let mut self_parent = CertificateBuilder::minimal();
        self_parent.resolution_trace =
            vec![resolution_step(1, clause_ref(2, 1), clause_ref(1, 1), 1)];
        assert_detail(
            parse_certificate(&self_parent.finish(), &context),
            CertificateRejectionDetail::MalformedCertificate,
        );

        let mut bad_parent_namespace = CertificateBuilder::minimal();
        bad_parent_namespace.resolution_trace =
            vec![resolution_step(1, clause_ref(9, 1), clause_ref(1, 1), 1)];
        assert_detail(
            parse_certificate(&bad_parent_namespace.finish(), &context),
            CertificateRejectionDetail::MalformedCertificate,
        );

        let mut bad_pivot = CertificateBuilder::minimal();
        bad_pivot.resolution_trace = vec![resolution_step_with_pivot(
            1,
            clause_ref(1, 1),
            clause_ref(1, 1),
            vec![9],
            1,
        )];
        assert_detail(
            parse_certificate(&bad_pivot.finish(), &context),
            CertificateRejectionDetail::MalformedCertificate,
        );

        let mut deep_pivot = CertificateBuilder::minimal();
        deep_pivot.resolution_trace = vec![resolution_step_with_pivot(
            1,
            clause_ref(1, 1),
            clause_ref(1, 1),
            literal_record(2, 1, 1, vec![nested_term(3)]),
            1,
        )];
        let limits = CertificateParseLimits {
            max_term_recursion_depth: 1,
            ..CertificateParseLimits::default()
        };
        assert_detail(
            parse_certificate(&deep_pivot.finish(), &context.clone().with_limits(limits)),
            CertificateRejectionDetail::ResourceExhaustion,
        );

        let mut bad_generated = CertificateBuilder::minimal();
        bad_generated.resolution_trace =
            vec![resolution_step(1, clause_ref(1, 1), clause_ref(1, 1), 99)];
        assert_detail(
            parse_certificate(&bad_generated.finish(), &context),
            CertificateRejectionDetail::MalformedCertificate,
        );

        let mut wrong_generated_namespace = CertificateBuilder::minimal();
        wrong_generated_namespace.resolution_trace = vec![resolution_step_with_generated_ref(
            1,
            clause_ref(1, 1),
            clause_ref(1, 1),
            clause_ref(2, 1),
        )];
        assert_detail(
            parse_certificate(&wrong_generated_namespace.finish(), &context),
            CertificateRejectionDetail::MalformedCertificate,
        );

        let mut bad_derived = CertificateBuilder::minimal();
        bad_derived.derived_facts = vec![derived_fact(1, clause_ref(2, 99))];
        assert_detail(
            parse_certificate(&bad_derived.finish(), &context),
            CertificateRejectionDetail::MalformedCertificate,
        );

        let mut final_generated = CertificateBuilder::minimal();
        final_generated.final_goal = final_goal(1, 1);
        assert!(parse_certificate(&final_generated.finish(), &context).is_ok());

        let mut final_resolution = CertificateBuilder::minimal();
        final_resolution.resolution_trace =
            vec![resolution_step(1, clause_ref(1, 1), clause_ref(1, 1), 1)];
        final_resolution.final_goal = final_goal(2, 1);
        assert!(parse_certificate(&final_resolution.finish(), &context).is_ok());

        let mut malformed_final_namespace = CertificateBuilder::minimal();
        malformed_final_namespace.final_goal = final_goal(9, 1);
        assert_detail(
            parse_certificate(&malformed_final_namespace.finish(), &context),
            CertificateRejectionDetail::MalformedCertificate,
        );

        let mut bad_final = CertificateBuilder::minimal();
        bad_final.final_goal = final_goal(3, 99);
        assert_detail(
            parse_certificate(&bad_final.finish(), &context),
            CertificateRejectionDetail::MalformedCertificate,
        );
    }

    #[test]
    fn preserves_deterministic_order_for_all_parsed_collections() {
        let context = sample_context();
        let mut builder = CertificateBuilder::minimal();
        builder.imported_axioms = vec![
            imported_fact(1, b"pkg", b"mod", b"axiom1"),
            imported_fact(2, b"pkg", b"mod", b"axiom2"),
        ];
        builder.imported_theorems = vec![imported_fact(1, b"pkg", b"mod", b"thm1")];
        builder
            .generated_clauses
            .push(generated_clause_with_literals(
                2,
                1,
                vec![
                    literal_record(2, 1, 1, vec![variable_term(1)]),
                    literal_record(2, 1, 2, vec![]),
                ],
            ));
        builder.substitutions = vec![
            substitution_entry(1, vec![1]),
            substitution_entry(2, vec![2]),
        ];
        builder.resolution_trace = vec![resolution_step(1, clause_ref(3, 1), clause_ref(1, 1), 1)];
        builder.derived_facts = vec![
            derived_fact(1, clause_ref(1, 1)),
            derived_fact(2, clause_ref(2, 1)),
        ];

        let parsed = parse_certificate(&builder.finish(), &context).expect("ordered certificate");

        assert_eq!(
            parsed
                .symbol_manifest
                .iter()
                .map(|entry| entry.symbol.id.0)
                .collect::<Vec<_>>(),
            [1, 2]
        );
        assert_eq!(
            parsed.generated_clauses[1]
                .clause
                .literals()
                .iter()
                .map(|literal| literal.atom.symbol.id.0)
                .collect::<Vec<_>>(),
            [1, 2]
        );

        let mut shuffled_child_bytes = CertificateBuilder::minimal();
        shuffled_child_bytes.generated_clauses = vec![generated_clause_with_literals(
            1,
            1,
            vec![
                literal_record(2, 1, 2, vec![]),
                literal_record(2, 1, 1, vec![variable_term(1)]),
            ],
        )];
        assert_detail(
            parse_certificate(&shuffled_child_bytes.finish(), &context),
            CertificateRejectionDetail::MalformedCertificate,
        );
        assert_eq!(
            parsed
                .variable_manifest
                .iter()
                .map(|entry| entry.variable_id.0)
                .collect::<Vec<_>>(),
            [1]
        );
        assert_eq!(
            parsed
                .imported_axioms
                .iter()
                .map(|fact| fact.imported_fact_id)
                .collect::<Vec<_>>(),
            [1, 2]
        );
        assert_eq!(
            parsed
                .imported_theorems
                .iter()
                .map(|fact| fact.imported_fact_id)
                .collect::<Vec<_>>(),
            [1]
        );
        assert_eq!(
            parsed
                .generated_clauses
                .iter()
                .map(|clause| clause.clause_id)
                .collect::<Vec<_>>(),
            [1, 2]
        );
        assert_eq!(
            parsed
                .substitutions
                .iter()
                .map(|entry| entry.substitution_id)
                .collect::<Vec<_>>(),
            [1, 2]
        );
        assert_eq!(
            parsed
                .resolution_trace
                .iter()
                .map(|step| step.step_id)
                .collect::<Vec<_>>(),
            [1]
        );
        assert_eq!(
            parsed
                .derived_facts
                .iter()
                .map(|fact| fact.derived_fact_id)
                .collect::<Vec<_>>(),
            [1, 2]
        );
    }

    #[test]
    fn deterministic_hash_input_excludes_nondeterministic_data() {
        let (bytes, context) = fixture();
        let parsed_a = parse_certificate(&bytes, &context).expect("valid a");
        let parsed_b = parse_certificate(&bytes, &context).expect("valid b");
        assert_eq!(parsed_a, parsed_b);
        assert_eq!(
            parsed_a.canonical_hash_input(),
            parsed_b.canonical_hash_input()
        );
        assert!(contains_subsequence(
            parsed_a.canonical_hash_input(),
            &1_u16.to_be_bytes()
        ));
        assert!(contains_subsequence(
            parsed_a.canonical_hash_input(),
            &profile_bytes(sample_profile())
        ));
        assert!(contains_subsequence(
            parsed_a.canonical_hash_input(),
            &item_frame(
                SectionTag::GeneratedClauses,
                generated_clause_with_literals(
                    1,
                    1,
                    vec![literal_record(2, 1, 1, vec![variable_term(1)])],
                )
            )
        ));
        let directory_start = directory_start(&context.expected_target_vc);
        let payload_start = directory_start + REQUIRED_SECTIONS.len() * DIRECTORY_ENTRY_LEN;
        assert!(contains_subsequence(
            parsed_a.canonical_hash_input(),
            &bytes[directory_start..payload_start]
        ));
        for excluded in [
            "source-path",
            "source-range",
            "display-name",
            "backend-log",
            "timestamp",
            "elapsed-time",
            "allocation-address",
            "allocation-order",
            "worker-completion-order",
            "cache-key",
            "artifact-path",
            "policy-projection",
        ] {
            assert!(!contains_subsequence(
                parsed_a.canonical_hash_input(),
                excluded.as_bytes()
            ));
        }
    }

    #[test]
    fn parser_errors_are_certificate_rejections_and_never_timeout() {
        let (mut bytes, context) = fixture();
        bytes[0] = 0;
        let error = parse_certificate(&bytes, &context).expect_err("invalid domain");
        assert_eq!(error.category, FailureCategory::CertificateRejection);
        assert_ne!(format!("{:?}", error.detail), "Timeout");
    }

    fn fixture() -> (Vec<u8>, CertificateParseContext) {
        (CertificateBuilder::minimal().finish(), sample_context())
    }

    fn sample_context() -> CertificateParseContext {
        CertificateParseContext::v1(sample_target(), sample_profile())
            .with_clause_validation_policy(ClauseValidationPolicy {
                max_literals: 8,
                max_term_encoding_bytes: 256,
            })
    }

    fn sample_profile() -> KernelProfileRecord {
        KernelProfileRecord::v1(1, ClauseTautologyPolicy::Reject)
    }

    fn sample_target() -> Fingerprint {
        Fingerprint::new(1, vec![1, 2, 3])
    }

    fn directory_start(target: &Fingerprint) -> usize {
        DOMAIN_SEPARATOR.len()
            + 2
            + 2
            + PROFILE_LEN
            + target.canonical_bytes().expect("target bytes").len()
            + 4
    }

    fn certificate_with_substitution_term_depth(depth: u32) -> Vec<u8> {
        let mut builder = CertificateBuilder::minimal();
        builder.substitutions = vec![substitution_entry_with_term(1, nested_term(depth))];
        builder.finish()
    }

    #[derive(Clone)]
    struct CertificateBuilder {
        symbol_manifest: Vec<Vec<u8>>,
        variable_manifest: Vec<Vec<u8>>,
        imported_axioms: Vec<Vec<u8>>,
        imported_theorems: Vec<Vec<u8>>,
        generated_clauses: Vec<Vec<u8>>,
        substitutions: Vec<Vec<u8>>,
        resolution_trace: Vec<Vec<u8>>,
        derived_facts: Vec<Vec<u8>>,
        final_goal: Vec<u8>,
    }

    impl CertificateBuilder {
        fn minimal() -> Self {
            Self {
                symbol_manifest: vec![symbol_manifest_entry(1, 1), symbol_manifest_entry(1, 2)],
                variable_manifest: vec![variable_manifest_entry(1)],
                imported_axioms: Vec::new(),
                imported_theorems: Vec::new(),
                generated_clauses: vec![generated_clause_with_literals(
                    1,
                    1,
                    vec![literal_record(2, 1, 1, vec![variable_term(1)])],
                )],
                substitutions: Vec::new(),
                resolution_trace: Vec::new(),
                derived_facts: Vec::new(),
                final_goal: final_goal(1, 1),
            }
        }

        fn finish(self) -> Vec<u8> {
            let sections = [
                (SectionTag::SymbolManifest, self.symbol_manifest),
                (SectionTag::VariableManifest, self.variable_manifest),
                (SectionTag::ImportedAxioms, self.imported_axioms),
                (SectionTag::ImportedTheorems, self.imported_theorems),
                (SectionTag::GeneratedClauses, self.generated_clauses),
                (SectionTag::Substitutions, self.substitutions),
                (SectionTag::ResolutionTrace, self.resolution_trace),
                (SectionTag::DerivedFacts, self.derived_facts),
                (SectionTag::FinalGoal, vec![self.final_goal]),
            ];
            let mut payloads = Vec::new();
            let mut directory = Vec::new();
            let mut offset = 0u32;
            for (section, items) in sections {
                let item_count = items.len() as u32;
                let section_payload = items
                    .into_iter()
                    .flat_map(|payload| item_frame(section, payload))
                    .collect::<Vec<_>>();
                directory.push((
                    section,
                    item_count,
                    offset,
                    items_len(section_payload.len()),
                ));
                offset += items_len(section_payload.len());
                payloads.extend(section_payload);
            }

            let mut bytes = Vec::from(DOMAIN_SEPARATOR);
            bytes.extend_from_slice(&1_u16.to_be_bytes());
            bytes.extend_from_slice(&1_u16.to_be_bytes());
            bytes.extend(profile_bytes(sample_profile()));
            bytes.extend(fingerprint_bytes(&sample_target()));
            bytes.extend_from_slice(&(REQUIRED_SECTIONS.len() as u32).to_be_bytes());
            for (section, item_count, payload_offset, payload_length) in directory {
                bytes.push(section.byte());
                bytes.extend_from_slice(&item_count.to_be_bytes());
                bytes.extend_from_slice(&payload_offset.to_be_bytes());
                bytes.extend_from_slice(&payload_length.to_be_bytes());
            }
            bytes.extend(payloads);
            bytes
        }
    }

    fn items_len(len: usize) -> u32 {
        u32::try_from(len).expect("fixture section fits")
    }

    fn item_frame(section: SectionTag, payload: Vec<u8>) -> Vec<u8> {
        let mut bytes = vec![section.byte(), 1];
        bytes.extend_from_slice(&(payload.len() as u32).to_be_bytes());
        bytes.extend(payload);
        bytes
    }

    fn profile_bytes(profile: KernelProfileRecord) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&profile.profile_id.to_be_bytes());
        bytes.extend_from_slice(&profile.clause_schema_version.to_be_bytes());
        bytes.extend_from_slice(&profile.clause_encoding_version.to_be_bytes());
        bytes.push(profile.clause_tautology_policy.tag());
        bytes.push(profile.certificate_hash_input_algorithm.tag());
        bytes
    }

    fn fingerprint_bytes(fingerprint: &Fingerprint) -> Vec<u8> {
        let mut bytes = vec![fingerprint.algorithm_id];
        bytes.extend_from_slice(&(fingerprint.digest.len() as u32).to_be_bytes());
        bytes.extend_from_slice(&fingerprint.digest);
        bytes
    }

    fn bytes_field(bytes: &[u8]) -> Vec<u8> {
        let mut encoded = Vec::new();
        encoded.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
        encoded.extend_from_slice(bytes);
        encoded
    }

    fn symbol_manifest_entry(kind: u8, id: u32) -> Vec<u8> {
        let mut bytes = vec![kind];
        bytes.extend_from_slice(&id.to_be_bytes());
        bytes
    }

    fn variable_manifest_entry(id: u32) -> Vec<u8> {
        id.to_be_bytes().to_vec()
    }

    fn imported_fact(id: u32, package: &[u8], module: &[u8], item: &[u8]) -> Vec<u8> {
        imported_fact_with_fingerprint(id, package, module, item, vec![7, 8], 1)
    }

    fn imported_fact_with_fingerprint(
        id: u32,
        package: &[u8],
        module: &[u8],
        item: &[u8],
        fingerprint: Vec<u8>,
        status: u8,
    ) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&id.to_be_bytes());
        bytes.extend(bytes_field(package));
        bytes.extend(bytes_field(module));
        bytes.extend(bytes_field(item));
        bytes.extend(fingerprint_bytes(&Fingerprint::new(1, fingerprint)));
        bytes.push(status);
        bytes
    }

    fn generated_clause_with_literals(id: u32, form: u8, literals: Vec<Vec<u8>>) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&id.to_be_bytes());
        bytes.push(form);
        bytes.extend_from_slice(&(literals.len() as u32).to_be_bytes());
        for literal in literals {
            bytes.extend(literal);
        }
        bytes
    }

    fn literal_record(polarity: u8, kind: u8, symbol_id: u32, terms: Vec<Vec<u8>>) -> Vec<u8> {
        let mut bytes = vec![polarity, kind];
        bytes.extend_from_slice(&symbol_id.to_be_bytes());
        bytes.extend_from_slice(&(terms.len() as u32).to_be_bytes());
        bytes.extend_from_slice(&(terms.len() as u32).to_be_bytes());
        for term in terms {
            bytes.extend(term);
        }
        bytes
    }

    fn variable_term(id: u32) -> Vec<u8> {
        let mut bytes = vec![1];
        bytes.extend_from_slice(&id.to_be_bytes());
        bytes
    }

    fn application_term(kind: u8, symbol_id: u32, arguments: Vec<Vec<u8>>) -> Vec<u8> {
        let mut bytes = vec![2, kind];
        bytes.extend_from_slice(&symbol_id.to_be_bytes());
        bytes.extend_from_slice(&(arguments.len() as u32).to_be_bytes());
        for argument in arguments {
            bytes.extend(argument);
        }
        bytes
    }

    fn nested_term(depth: u32) -> Vec<u8> {
        if depth == 0 {
            return variable_term(1);
        }
        let mut bytes = vec![3];
        bytes.extend_from_slice(&depth.to_be_bytes());
        bytes.extend(nested_term(depth - 1));
        bytes
    }

    fn substitution_entry(id: u32, freshness_refs: Vec<u32>) -> Vec<u8> {
        substitution_entry_with_refs(id, freshness_refs, vec![2])
    }

    fn substitution_entry_with_refs(
        id: u32,
        freshness_refs: Vec<u32>,
        free_variable_refs: Vec<u32>,
    ) -> Vec<u8> {
        substitution_entry_with_term(id, variable_term(1))
            .with_freshness(freshness_refs)
            .with_free_variables(free_variable_refs)
    }

    fn substitution_entry_with_term(id: u32, term: Vec<u8>) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&id.to_be_bytes());
        bytes.extend(term.clone());
        bytes.extend(term);
        bytes.extend(bytes_field(b"binder"));
        bytes.extend(ref_list(&[1]));
        bytes.extend(ref_list(&[2]));
        bytes
    }

    trait WithFreshness {
        fn with_freshness(self, refs: Vec<u32>) -> Self;
        fn with_free_variables(self, refs: Vec<u32>) -> Self;
    }

    impl WithFreshness for Vec<u8> {
        fn with_freshness(mut self, refs: Vec<u32>) -> Self {
            let mut replacement = Vec::new();
            replacement.extend(ref_list(&refs));
            let start = self.len() - ref_list(&[1]).len() - ref_list(&[2]).len();
            self.splice(start..start + ref_list(&[1]).len(), replacement);
            self
        }

        fn with_free_variables(mut self, refs: Vec<u32>) -> Self {
            let mut replacement = Vec::new();
            replacement.extend(ref_list(&refs));
            let start = self.len() - ref_list(&[2]).len();
            self.splice(start.., replacement);
            self
        }
    }

    fn ref_list(ids: &[u32]) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&(ids.len() as u32).to_be_bytes());
        for id in ids {
            bytes.extend_from_slice(&id.to_be_bytes());
        }
        bytes
    }

    fn clause_ref(namespace: u8, id: u32) -> Vec<u8> {
        let mut bytes = vec![namespace];
        bytes.extend_from_slice(&id.to_be_bytes());
        bytes
    }

    fn resolution_step(
        id: u32,
        parent_a: Vec<u8>,
        parent_b: Vec<u8>,
        generated_id: u32,
    ) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&id.to_be_bytes());
        bytes.extend(parent_a);
        bytes.extend(parent_b);
        bytes.extend(literal_record(2, 1, 1, vec![variable_term(1)]));
        bytes.extend(clause_ref(1, generated_id));
        bytes
    }

    fn resolution_step_with_pivot(
        id: u32,
        parent_a: Vec<u8>,
        parent_b: Vec<u8>,
        pivot: Vec<u8>,
        generated_id: u32,
    ) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&id.to_be_bytes());
        bytes.extend(parent_a);
        bytes.extend(parent_b);
        bytes.extend(pivot);
        bytes.extend(clause_ref(1, generated_id));
        bytes
    }

    fn resolution_step_with_generated_ref(
        id: u32,
        parent_a: Vec<u8>,
        parent_b: Vec<u8>,
        generated_ref: Vec<u8>,
    ) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&id.to_be_bytes());
        bytes.extend(parent_a);
        bytes.extend(parent_b);
        bytes.extend(literal_record(2, 1, 1, vec![variable_term(1)]));
        bytes.extend(generated_ref);
        bytes
    }

    fn derived_fact(id: u32, source: Vec<u8>) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&id.to_be_bytes());
        bytes.extend(source);
        bytes.extend(bytes_field(b"derived"));
        bytes
    }

    fn final_goal(namespace: u8, id: u32) -> Vec<u8> {
        let mut bytes = vec![namespace];
        bytes.extend_from_slice(&id.to_be_bytes());
        bytes
    }

    fn directory_entry_start(target: &Fingerprint, section: SectionTag) -> usize {
        directory_start(target) + section_index(section) * DIRECTORY_ENTRY_LEN
    }

    fn section_payload_offset(bytes: &[u8], target: &Fingerprint, section: SectionTag) -> u32 {
        let start = directory_entry_start(target, section) + 5;
        u32::from_be_bytes([
            bytes[start],
            bytes[start + 1],
            bytes[start + 2],
            bytes[start + 3],
        ])
    }

    fn section_payload_length(bytes: &[u8], target: &Fingerprint, section: SectionTag) -> u32 {
        let start = directory_entry_start(target, section) + 9;
        u32::from_be_bytes([
            bytes[start],
            bytes[start + 1],
            bytes[start + 2],
            bytes[start + 3],
        ])
    }

    fn section_index(section: SectionTag) -> usize {
        REQUIRED_SECTIONS
            .iter()
            .position(|candidate| *candidate == section)
            .expect("known section")
    }

    fn set_u32(bytes: &mut [u8], offset: usize, value: u32) {
        bytes[offset..offset + 4].copy_from_slice(&value.to_be_bytes());
    }

    fn assert_detail(
        result: Result<ParsedCertificate, CertificateParseError>,
        detail: CertificateRejectionDetail,
    ) {
        let error = result.expect_err("expected parser rejection");
        assert_eq!(error.category, FailureCategory::CertificateRejection);
        assert_eq!(error.detail, detail);
    }

    fn assert_rejection_location(
        result: Result<ParsedCertificate, CertificateParseError>,
        detail: CertificateRejectionDetail,
        byte_offset: usize,
        section_tag: Option<SectionTag>,
        item_index: Option<u32>,
        field_path: &'static str,
    ) {
        let error = result.expect_err("expected parser rejection");
        assert_eq!(error.category, FailureCategory::CertificateRejection);
        assert_eq!(error.detail, detail);
        assert_eq!(error.location.byte_offset, byte_offset);
        assert_eq!(error.location.section_tag, section_tag);
        assert_eq!(error.location.item_index, item_index);
        assert_eq!(error.location.field_path, Some(field_path));
    }

    fn contains_subsequence(bytes: &[u8], needle: &[u8]) -> bool {
        bytes.windows(needle.len()).any(|window| window == needle)
    }
}
