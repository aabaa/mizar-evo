use std::collections::BTreeSet;

use crate::{
    certificate_parser::{
        CertificateHashInputAlgorithm, ClauseTautologyPolicy, Fingerprint, KernelProfileRecord,
        RequiredProofStatus, SymbolManifestEntry, VariableManifestEntry,
    },
    clause::{
        Atom, Clause, ClauseError, ClauseProfile, ClauseValidationContext, Literal, Polarity,
        SymbolKey, SymbolKind, Term, VariableId,
    },
    rejection::{
        RejectionCategory, RejectionDetail, RejectionLocation, RejectionRecord, TargetVcFingerprint,
    },
    substitution_checker::{
        FreeVariableConstraint, FreshnessWitness, Replacement, SubstitutionPayload, TermPath,
        TermPathSegment,
    },
};

pub const SUPPORTED_FORMULA_FINGERPRINT_ALGORITHM_ID: u8 = 2;

const EVIDENCE_DOMAIN_SEPARATOR: &[u8] = b"MIZAR_KERNEL_EVIDENCE\0";
const FORMULA_DOMAIN_SEPARATOR: &[u8] = b"MIZAR_KERNEL_FORMULA\0";
const ENTRY_DOMAIN_SEPARATOR: &[u8] = b"MIZAR_KERNEL_FORMULA_ENTRY\0";
const SCHEMA_VERSION_V1: u16 = 1;
const ENCODING_VERSION_V1: u16 = 1;
const PROFILE_LEN: usize = 8;
const FORMULA_FRAME_TAG: u8 = 11;
const ENTRY_FRAME_TAG: u8 = 12;
const REQUIRED_SECTIONS: [EvidenceSectionTag; 6] = [
    EvidenceSectionTag::SymbolManifest,
    EvidenceSectionTag::VariableManifest,
    EvidenceSectionTag::Formulas,
    EvidenceSectionTag::Substitutions,
    EvidenceSectionTag::Provenance,
    EvidenceSectionTag::FinalGoal,
];

const PAYLOAD_KIND_FORMAL_TO_ACTUAL_MAP: u8 = 1;
const PAYLOAD_KIND_LOCAL_ABBREVIATION_EXPANSION: u8 = 2;
const REPLACEMENT_ROLE_TERM_ARGUMENT: u8 = 1;
const REPLACEMENT_ROLE_PREDICATE_ARGUMENT: u8 = 2;
const REPLACEMENT_ROLE_CAPTURED_FREE_VARIABLE: u8 = 3;
const EDGE_KIND_APPLICATION_ARGUMENT: u8 = 1;
const EDGE_KIND_BINDER_BODY: u8 = 2;
const CONSTRAINT_KIND_MUST_REMAIN_FREE_AT_PATH: u8 = 1;
const CONSTRAINT_KIND_MUST_BE_ABSENT_FROM_CAPTURE_SET: u8 = 2;
const DEFAULT_MAX_EVIDENCE_BYTES: usize = 16 * 1024 * 1024;
const DEFAULT_MAX_SECTION_BYTES: usize = 8 * 1024 * 1024;
const DEFAULT_MAX_SYMBOLS: usize = 16_384;
const DEFAULT_MAX_VARIABLES: usize = 65_536;
const DEFAULT_MAX_FORMULAS: usize = 32_768;
const DEFAULT_MAX_SUBSTITUTIONS: usize = 32_768;
const DEFAULT_MAX_PROVENANCE_ENTRIES: usize = 65_536;
const DEFAULT_MAX_FORMULA_NODES: usize = 1_000_000;
const DEFAULT_MAX_FORMULA_CHILDREN: usize = 4_096;
const DEFAULT_MAX_TERM_ENCODING_BYTES: usize = 1_048_576;
const DEFAULT_MAX_SUBSTITUTION_REPLACEMENTS: usize = 4_096;
const DEFAULT_MAX_FRESHNESS_WITNESSES: usize = 16_384;
const DEFAULT_MAX_FREE_VARIABLE_CONSTRAINTS: usize = 16_384;
const DEFAULT_MAX_TERM_PATH_SEGMENTS: usize = 4_096;
const DEFAULT_MAX_VARIABLE_LIST_ENTRIES: usize = 16_384;
const DEFAULT_MAX_OPAQUE_BYTES: usize = 1_048_576;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FormulaEvidenceParseContext {
    pub accepted_schema_versions: BTreeSet<u16>,
    pub accepted_encoding_versions: BTreeSet<u16>,
    pub accepted_kernel_profiles: BTreeSet<KernelProfileRecord>,
    pub expected_target_vc: Fingerprint,
    pub limits: FormulaEvidenceParseLimits,
}

impl FormulaEvidenceParseContext {
    #[must_use]
    pub fn v1(expected_target_vc: Fingerprint, profile: KernelProfileRecord) -> Self {
        Self {
            accepted_schema_versions: BTreeSet::from([SCHEMA_VERSION_V1]),
            accepted_encoding_versions: BTreeSet::from([ENCODING_VERSION_V1]),
            accepted_kernel_profiles: BTreeSet::from([profile]),
            expected_target_vc,
            limits: FormulaEvidenceParseLimits::default(),
        }
    }

    #[must_use]
    pub const fn with_limits(mut self, limits: FormulaEvidenceParseLimits) -> Self {
        self.limits = limits;
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FormulaEvidenceParseLimits {
    pub max_evidence_bytes: usize,
    pub max_section_bytes: usize,
    pub max_symbols: usize,
    pub max_variables: usize,
    pub max_formulas: usize,
    pub max_substitutions: usize,
    pub max_provenance_entries: usize,
    pub max_formula_depth: usize,
    pub max_formula_nodes: usize,
    pub max_formula_children: usize,
    pub max_term_encoding_bytes: usize,
    pub max_term_recursion_depth: usize,
    pub max_substitution_replacements: usize,
    pub max_freshness_witnesses: usize,
    pub max_free_variable_constraints: usize,
    pub max_term_path_segments: usize,
    pub max_variable_list_entries: usize,
    pub max_opaque_bytes: usize,
}

impl Default for FormulaEvidenceParseLimits {
    fn default() -> Self {
        Self {
            max_evidence_bytes: DEFAULT_MAX_EVIDENCE_BYTES,
            max_section_bytes: DEFAULT_MAX_SECTION_BYTES,
            max_symbols: DEFAULT_MAX_SYMBOLS,
            max_variables: DEFAULT_MAX_VARIABLES,
            max_formulas: DEFAULT_MAX_FORMULAS,
            max_substitutions: DEFAULT_MAX_SUBSTITUTIONS,
            max_provenance_entries: DEFAULT_MAX_PROVENANCE_ENTRIES,
            max_formula_depth: 64,
            max_formula_nodes: DEFAULT_MAX_FORMULA_NODES,
            max_formula_children: DEFAULT_MAX_FORMULA_CHILDREN,
            max_term_encoding_bytes: DEFAULT_MAX_TERM_ENCODING_BYTES,
            max_term_recursion_depth: 64,
            max_substitution_replacements: DEFAULT_MAX_SUBSTITUTION_REPLACEMENTS,
            max_freshness_witnesses: DEFAULT_MAX_FRESHNESS_WITNESSES,
            max_free_variable_constraints: DEFAULT_MAX_FREE_VARIABLE_CONSTRAINTS,
            max_term_path_segments: DEFAULT_MAX_TERM_PATH_SEGMENTS,
            max_variable_list_entries: DEFAULT_MAX_VARIABLE_LIST_ENTRIES,
            max_opaque_bytes: DEFAULT_MAX_OPAQUE_BYTES,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedKernelEvidence {
    pub schema_version: u16,
    pub encoding_version: u16,
    pub kernel_profile: KernelProfileRecord,
    pub target_vc: Fingerprint,
    pub symbol_manifest: Vec<SymbolManifestEntry>,
    pub variable_manifest: Vec<VariableManifestEntry>,
    pub formulas: Vec<FormulaEvidenceEntry>,
    pub substitutions: Vec<FormulaSubstitutionEvidence>,
    pub provenance: Vec<FormulaProvenance>,
    pub final_goal: FinalGoalEvidence,
    canonical_hash_input: Vec<u8>,
}

impl ParsedKernelEvidence {
    #[must_use]
    pub fn canonical_hash_input(&self) -> &[u8] {
        &self.canonical_hash_input
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FormulaEvidenceEntry {
    pub formula_id: u32,
    pub source: FormulaSource,
    pub formula: Formula,
    pub formula_fingerprint: Fingerprint,
    pub provenance_id: u32,
}

impl FormulaEvidenceEntry {
    #[must_use = "entry hash input must be consumed or checked"]
    pub fn entry_hash_input(&self) -> Result<Vec<u8>, FormulaEvidenceError> {
        let mut payload = Vec::new();
        payload.extend_from_slice(&self.formula_id.to_be_bytes());
        payload.push(self.source.source_class().tag());
        payload.extend(fingerprint_bytes(&self.formula_fingerprint)?);
        payload.extend(self.source.canonical_bytes()?);
        payload.extend_from_slice(&self.provenance_id.to_be_bytes());

        let mut bytes = Vec::from(ENTRY_DOMAIN_SEPARATOR);
        bytes.extend(frame(
            ENTRY_FRAME_TAG,
            self.source.source_class().tag(),
            payload,
        )?);
        Ok(bytes)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[non_exhaustive]
pub enum FormulaSourceClass {
    LocalHypothesis,
    CitedPremise,
    GeneratedVcFact,
    AcceptedImportedAxiom,
    AcceptedImportedTheorem,
    PolicyBoundedBuiltin,
}

impl FormulaSourceClass {
    const fn from_tag(tag: u8) -> Option<Self> {
        match tag {
            1 => Some(Self::LocalHypothesis),
            2 => Some(Self::CitedPremise),
            3 => Some(Self::GeneratedVcFact),
            4 => Some(Self::AcceptedImportedAxiom),
            5 => Some(Self::AcceptedImportedTheorem),
            6 => Some(Self::PolicyBoundedBuiltin),
            _ => None,
        }
    }

    #[must_use]
    pub const fn tag(self) -> u8 {
        match self {
            Self::LocalHypothesis => 1,
            Self::CitedPremise => 2,
            Self::GeneratedVcFact => 3,
            Self::AcceptedImportedAxiom => 4,
            Self::AcceptedImportedTheorem => 5,
            Self::PolicyBoundedBuiltin => 6,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum FormulaSource {
    LocalHypothesis { local_context_id: u32 },
    CitedPremise { local_context_id: u32 },
    GeneratedVcFact { vc_fact_id: u32 },
    AcceptedImportedAxiom(ImportedFormulaSource),
    AcceptedImportedTheorem(ImportedFormulaSource),
    PolicyBoundedBuiltin { built_in_id: Vec<u8> },
}

impl FormulaSource {
    #[must_use]
    pub const fn source_class(&self) -> FormulaSourceClass {
        match self {
            Self::LocalHypothesis { .. } => FormulaSourceClass::LocalHypothesis,
            Self::CitedPremise { .. } => FormulaSourceClass::CitedPremise,
            Self::GeneratedVcFact { .. } => FormulaSourceClass::GeneratedVcFact,
            Self::AcceptedImportedAxiom(_) => FormulaSourceClass::AcceptedImportedAxiom,
            Self::AcceptedImportedTheorem(_) => FormulaSourceClass::AcceptedImportedTheorem,
            Self::PolicyBoundedBuiltin { .. } => FormulaSourceClass::PolicyBoundedBuiltin,
        }
    }

    fn canonical_bytes(&self) -> Result<Vec<u8>, FormulaEvidenceError> {
        let mut payload = Vec::new();
        match self {
            Self::LocalHypothesis { local_context_id }
            | Self::CitedPremise { local_context_id } => {
                payload.extend_from_slice(&local_context_id.to_be_bytes());
            }
            Self::GeneratedVcFact { vc_fact_id } => {
                payload.extend_from_slice(&vc_fact_id.to_be_bytes());
            }
            Self::AcceptedImportedAxiom(source) | Self::AcceptedImportedTheorem(source) => {
                payload.extend(imported_source_bytes(source)?);
            }
            Self::PolicyBoundedBuiltin { built_in_id } => {
                write_len(built_in_id.len(), &mut payload)?;
                payload.extend_from_slice(built_in_id);
            }
        }
        frame(ENTRY_FRAME_TAG, self.source_class().tag(), payload)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportedFormulaSource {
    pub package_id: Vec<u8>,
    pub module_path: Vec<u8>,
    pub exported_item_id: Vec<u8>,
    pub statement_fingerprint: Fingerprint,
    pub required_proof_status: RequiredProofStatus,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Formula {
    Atom(Atom),
    Not(Box<Formula>),
    And(Vec<Formula>),
    Or(Vec<Formula>),
}

impl Formula {
    #[must_use]
    pub fn render(&self) -> String {
        match self {
            Self::Atom(atom) => atom.render(),
            Self::Not(formula) => format!("not({})", formula.render()),
            Self::And(children) => render_children("and", children),
            Self::Or(children) => render_children("or", children),
        }
    }

    #[must_use = "canonical formula hash input must be consumed or checked"]
    pub fn canonical_hash_input(&self) -> Result<Vec<u8>, FormulaEvidenceError> {
        let mut bytes = Vec::from(FORMULA_DOMAIN_SEPARATOR);
        bytes.extend(self.canonical_bytes()?);
        Ok(bytes)
    }

    fn canonical_bytes(&self) -> Result<Vec<u8>, FormulaEvidenceError> {
        match self {
            Self::Atom(atom) => frame(
                FORMULA_FRAME_TAG,
                1,
                atom.canonical_bytes()
                    .map_err(FormulaEvidenceError::Clause)?,
            ),
            Self::Not(formula) => frame(FORMULA_FRAME_TAG, 2, formula.canonical_bytes()?),
            Self::And(children) => child_frame(3, children),
            Self::Or(children) => child_frame(4, children),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FormulaSubstitutionEvidence {
    pub substitution_id: u32,
    pub source_formula_id: u32,
    pub binder_context_encoding: Vec<u8>,
    pub payload: SubstitutionPayload,
    pub freshness_witnesses: Vec<FreshnessWitness>,
    pub free_variable_constraints: Vec<FreeVariableConstraint>,
    pub provenance_id: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FormulaProvenance {
    pub provenance_id: u32,
    pub target_vc: Fingerprint,
    pub formula_fingerprint: Fingerprint,
    pub payload: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FinalGoalEvidence {
    pub polarity: GoalPolarity,
    pub formula: Formula,
    pub formula_fingerprint: Fingerprint,
    pub provenance_id: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum GoalPolarity {
    AssertFalseForRefutation,
    AssertTrueForConsistency,
}

impl GoalPolarity {
    const fn from_tag(tag: u8) -> Option<Self> {
        match tag {
            1 => Some(Self::AssertFalseForRefutation),
            2 => Some(Self::AssertTrueForConsistency),
            _ => None,
        }
    }

    #[must_use]
    pub const fn tag(self) -> u8 {
        match self {
            Self::AssertFalseForRefutation => 1,
            Self::AssertTrueForConsistency => 2,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum FormulaEvidenceError {
    Clause(ClauseError),
    ResourceExhaustion,
}

pub type FormulaEvidenceCheckResult<T> = Result<T, Box<RejectionRecord>>;

pub fn parse_formula_evidence(
    bytes: &[u8],
    context: &FormulaEvidenceParseContext,
) -> FormulaEvidenceCheckResult<ParsedKernelEvidence> {
    let target = TargetVcFingerprint::from_certificate_fingerprint(&context.expected_target_vc);
    if bytes.len() > context.limits.max_evidence_bytes {
        return Err(certificate_rejection(
            &target,
            RejectionDetail::ResourceExhaustion,
            RejectionLocation::new().with_field_path("evidence"),
        ));
    }

    let mut reader = Reader::new(bytes, 0, &target, None);
    if bytes.len() < EVIDENCE_DOMAIN_SEPARATOR.len()
        || &bytes[..EVIDENCE_DOMAIN_SEPARATOR.len()] != EVIDENCE_DOMAIN_SEPARATOR
    {
        return Err(reader.unsupported_at(0, "domain_separator"));
    }
    reader.read_exact(EVIDENCE_DOMAIN_SEPARATOR.len(), "domain_separator")?;

    let schema_version = reader.read_u16("schema_version")?;
    if schema_version != SCHEMA_VERSION_V1
        || !context.accepted_schema_versions.contains(&schema_version)
    {
        return Err(reader.unsupported_at(EVIDENCE_DOMAIN_SEPARATOR.len(), "schema_version"));
    }
    let encoding_version = reader.read_u16("encoding_version")?;
    if encoding_version != ENCODING_VERSION_V1
        || !context
            .accepted_encoding_versions
            .contains(&encoding_version)
    {
        return Err(reader.unsupported_at(EVIDENCE_DOMAIN_SEPARATOR.len() + 2, "encoding_version"));
    }

    let kernel_profile = read_kernel_profile(&mut reader)?;
    if !context.accepted_kernel_profiles.contains(&kernel_profile) {
        return Err(reader.unsupported_at(EVIDENCE_DOMAIN_SEPARATOR.len() + 4, "kernel_profile"));
    }
    let target_vc = read_fingerprint(&mut reader, "target_vc")?;
    if target_vc != context.expected_target_vc {
        return Err(certificate_rejection(
            &target,
            RejectionDetail::ContextMismatch,
            RejectionLocation::new()
                .with_certificate_byte_offset(EVIDENCE_DOMAIN_SEPARATOR.len() + 4 + PROFILE_LEN)
                .with_field_path("target_vc"),
        ));
    }

    let directory_count = reader.read_u32("directory_entry_count")?;
    if directory_count as usize != REQUIRED_SECTIONS.len() {
        return Err(reader.malformed_at(reader.offset().saturating_sub(4), "directory_entry_count"));
    }

    let mut directory = Vec::with_capacity(REQUIRED_SECTIONS.len());
    for expected in REQUIRED_SECTIONS {
        let entry_offset = reader.offset();
        let tag_offset = reader.offset();
        let tag_byte = reader.read_u8("directory.section_tag")?;
        let Some(section_tag) = EvidenceSectionTag::from_byte(tag_byte) else {
            return Err(reader.unsupported_at(tag_offset, "directory.section_tag"));
        };
        if section_tag != expected {
            return Err(reader.malformed_at(entry_offset, "directory.section_tag"));
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
        validate_section_limit(&entry, context)?;
        directory.push(entry);
    }

    let payload_base = reader.offset();
    for entry in &mut directory {
        entry.payload_base_offset = payload_base;
    }
    let payload_bytes = &bytes[payload_base..];
    validate_directory_ranges(&directory, payload_bytes.len(), &target)?;

    let symbol_manifest = parse_symbol_manifest(
        section_slice(payload_bytes, &directory[0]),
        &directory[0],
        &target,
    )?;
    let variable_manifest = parse_variable_manifest(
        section_slice(payload_bytes, &directory[1]),
        &directory[1],
        &target,
    )?;
    let formula_context = formula_validation_context(
        &kernel_profile,
        &symbol_manifest,
        &variable_manifest,
        context,
    );
    let formulas = parse_formulas(
        section_slice(payload_bytes, &directory[2]),
        &directory[2],
        &target,
        &formula_context,
        context,
    )?;
    let formula_ids = formulas
        .iter()
        .map(|formula| formula.formula_id)
        .collect::<BTreeSet<_>>();
    let substitutions = parse_substitutions(
        section_slice(payload_bytes, &directory[3]),
        &directory[3],
        &target,
        &formula_context,
        &formula_ids,
        context,
    )?;
    let provenance = parse_provenance(
        section_slice(payload_bytes, &directory[4]),
        &directory[4],
        &target,
        context,
    )?;
    let final_goal = parse_final_goal(
        section_slice(payload_bytes, &directory[5]),
        &directory[5],
        &target,
        &formula_context,
        context,
    )?;

    validate_provenance_bindings(
        &target,
        &target_vc,
        &formulas,
        &substitutions,
        &provenance,
        &final_goal,
    )?;

    Ok(ParsedKernelEvidence {
        schema_version,
        encoding_version,
        kernel_profile,
        target_vc,
        symbol_manifest,
        variable_manifest,
        formulas,
        substitutions,
        provenance,
        final_goal,
        canonical_hash_input: bytes.to_vec(),
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EvidenceSectionTag {
    SymbolManifest,
    VariableManifest,
    Formulas,
    Substitutions,
    Provenance,
    FinalGoal,
}

impl EvidenceSectionTag {
    const fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            1 => Some(Self::SymbolManifest),
            2 => Some(Self::VariableManifest),
            3 => Some(Self::Formulas),
            4 => Some(Self::Substitutions),
            5 => Some(Self::Provenance),
            6 => Some(Self::FinalGoal),
            _ => None,
        }
    }

    const fn byte(self) -> u8 {
        match self {
            Self::SymbolManifest => 1,
            Self::VariableManifest => 2,
            Self::Formulas => 3,
            Self::Substitutions => 4,
            Self::Provenance => 5,
            Self::FinalGoal => 6,
        }
    }
}

#[derive(Clone, Copy)]
struct DirectoryEntry {
    section_tag: EvidenceSectionTag,
    item_count: u32,
    payload_offset: u32,
    payload_length: u32,
    entry_offset: usize,
    payload_base_offset: usize,
}

fn validate_section_limit(
    entry: &DirectoryEntry,
    context: &FormulaEvidenceParseContext,
) -> FormulaEvidenceCheckResult<()> {
    let target = TargetVcFingerprint::from_certificate_fingerprint(&context.expected_target_vc);
    if entry.section_tag == EvidenceSectionTag::FinalGoal && entry.item_count != 1 {
        return Err(certificate_rejection(
            &target,
            RejectionDetail::MalformedWitnessData,
            RejectionLocation::new()
                .with_certificate_byte_offset(entry.entry_offset + 1)
                .with_field_path("directory.item_count"),
        ));
    }
    let max_count = match entry.section_tag {
        EvidenceSectionTag::SymbolManifest => context.limits.max_symbols,
        EvidenceSectionTag::VariableManifest => context.limits.max_variables,
        EvidenceSectionTag::Formulas => context.limits.max_formulas,
        EvidenceSectionTag::Substitutions => context.limits.max_substitutions,
        EvidenceSectionTag::Provenance => context.limits.max_provenance_entries,
        EvidenceSectionTag::FinalGoal => 1,
    };
    if entry.item_count as usize > max_count {
        return Err(certificate_rejection(
            &target,
            RejectionDetail::ResourceExhaustion,
            RejectionLocation::new()
                .with_certificate_byte_offset(entry.entry_offset + 1)
                .with_field_path("directory.item_count"),
        ));
    }
    if entry.payload_length as usize > context.limits.max_section_bytes {
        return Err(certificate_rejection(
            &target,
            RejectionDetail::ResourceExhaustion,
            RejectionLocation::new()
                .with_certificate_byte_offset(entry.entry_offset + 9)
                .with_field_path("directory.payload_length"),
        ));
    }
    Ok(())
}

fn validate_directory_ranges(
    directory: &[DirectoryEntry],
    payload_len: usize,
    target: &TargetVcFingerprint,
) -> FormulaEvidenceCheckResult<()> {
    let mut expected_offset = 0usize;
    for entry in directory {
        let payload_offset = entry.payload_offset as usize;
        let payload_length = entry.payload_length as usize;
        if payload_offset != expected_offset {
            return Err(certificate_rejection(
                target,
                RejectionDetail::MalformedWitnessData,
                RejectionLocation::new()
                    .with_certificate_byte_offset(entry.entry_offset + 5)
                    .with_field_path("directory.payload_offset"),
            ));
        }
        let Some(end) = payload_offset.checked_add(payload_length) else {
            return Err(certificate_rejection(
                target,
                RejectionDetail::ResourceExhaustion,
                RejectionLocation::new()
                    .with_certificate_byte_offset(entry.entry_offset + 5)
                    .with_field_path("directory.range"),
            ));
        };
        if end > payload_len {
            return Err(certificate_rejection(
                target,
                RejectionDetail::MalformedWitnessData,
                RejectionLocation::new()
                    .with_certificate_byte_offset(entry.entry_offset + 9)
                    .with_field_path("directory.payload_length"),
            ));
        }
        expected_offset = end;
    }
    if expected_offset != payload_len {
        let byte_offset = directory.first().map_or(expected_offset, |entry| {
            entry.payload_base_offset + expected_offset
        });
        return Err(certificate_rejection(
            target,
            RejectionDetail::MalformedWitnessData,
            RejectionLocation::new()
                .with_certificate_byte_offset(byte_offset)
                .with_field_path("section_payloads.trailing_bytes"),
        ));
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
    target: &TargetVcFingerprint,
) -> FormulaEvidenceCheckResult<Vec<SymbolManifestEntry>> {
    let frames = read_section_frames(bytes, entry, target)?;
    let mut entries = Vec::with_capacity(frames.len());
    let mut seen = BTreeSet::new();
    for frame in frames {
        let mut reader = frame.reader(target);
        let kind = read_symbol_kind(&mut reader, "symbol_manifest.symbol_kind")?;
        let id = reader.read_u32("symbol_manifest.symbol_id")?;
        reader.finish()?;
        let symbol = SymbolKey::new(kind, id);
        if !seen.insert(symbol) {
            return Err(frame.malformed(target, "symbol_manifest.duplicate"));
        }
        entries.push(SymbolManifestEntry { symbol });
    }
    ensure_sorted_by(
        entries.iter().map(|entry| entry.symbol),
        entry,
        target,
        "symbol_manifest",
    )?;
    Ok(entries)
}

fn parse_variable_manifest(
    bytes: &[u8],
    entry: &DirectoryEntry,
    target: &TargetVcFingerprint,
) -> FormulaEvidenceCheckResult<Vec<VariableManifestEntry>> {
    let frames = read_section_frames(bytes, entry, target)?;
    let mut entries = Vec::with_capacity(frames.len());
    let mut seen = BTreeSet::new();
    for frame in frames {
        let mut reader = frame.reader(target);
        let variable_id = VariableId(reader.read_u32("variable_manifest.variable_id")?);
        reader.finish()?;
        if !seen.insert(variable_id) {
            return Err(frame.malformed(target, "variable_manifest.duplicate"));
        }
        entries.push(VariableManifestEntry { variable_id });
    }
    ensure_sorted_by(
        entries.iter().map(|entry| entry.variable_id),
        entry,
        target,
        "variable_manifest",
    )?;
    Ok(entries)
}

fn parse_formulas(
    bytes: &[u8],
    entry: &DirectoryEntry,
    target: &TargetVcFingerprint,
    formula_context: &ClauseValidationContext,
    context: &FormulaEvidenceParseContext,
) -> FormulaEvidenceCheckResult<Vec<FormulaEvidenceEntry>> {
    let frames = read_section_frames(bytes, entry, target)?;
    let mut formulas = Vec::with_capacity(frames.len());
    let mut seen = BTreeSet::new();
    for frame in frames {
        let mut reader = frame.reader(target);
        let formula_id = reader.read_u32("formula.formula_id")?;
        let source_class_offset = reader.offset();
        let source_class_tag = reader.read_u8("formula.source_class")?;
        let Some(source_class) = FormulaSourceClass::from_tag(source_class_tag) else {
            return Err(reader.malformed_at(source_class_offset, "formula.source_class"));
        };
        let formula_fingerprint = read_fingerprint(&mut reader, "formula.fingerprint")?;
        let provenance_id = reader.read_u32("formula.provenance_id")?;
        let source = read_formula_source(&mut reader, source_class, &formula_fingerprint)?;
        let mut node_count = 0usize;
        let formula = read_formula(
            &mut reader,
            formula_context,
            context,
            0,
            &mut node_count,
            "formula",
        )?;
        reader.finish()?;
        if !seen.insert(formula_id) {
            return Err(frame.malformed(target, "formula.duplicate_id"));
        }
        validate_formula_fingerprint(
            target,
            &formula,
            &formula_fingerprint,
            "formula.fingerprint",
        )?;
        formulas.push(FormulaEvidenceEntry {
            formula_id,
            source,
            formula,
            formula_fingerprint,
            provenance_id,
        });
    }
    ensure_sorted_by(
        formulas.iter().map(|entry| entry.formula_id),
        entry,
        target,
        "formula",
    )?;
    Ok(formulas)
}

fn read_formula_source(
    reader: &mut Reader<'_>,
    source_class: FormulaSourceClass,
    formula_fingerprint: &Fingerprint,
) -> FormulaEvidenceCheckResult<FormulaSource> {
    match source_class {
        FormulaSourceClass::LocalHypothesis => {
            let local_context_id = reader.read_u32("formula.local_context_id")?;
            if local_context_id == 0 {
                return Err(reader.missing_at(
                    reader.offset().saturating_sub(4),
                    "formula.local_context_id",
                ));
            }
            Ok(FormulaSource::LocalHypothesis { local_context_id })
        }
        FormulaSourceClass::CitedPremise => {
            let local_context_id = reader.read_u32("formula.local_context_id")?;
            if local_context_id == 0 {
                return Err(reader.missing_at(
                    reader.offset().saturating_sub(4),
                    "formula.local_context_id",
                ));
            }
            Ok(FormulaSource::CitedPremise { local_context_id })
        }
        FormulaSourceClass::GeneratedVcFact => {
            let vc_fact_id = reader.read_u32("formula.vc_fact_id")?;
            if vc_fact_id == 0 {
                return Err(
                    reader.missing_at(reader.offset().saturating_sub(4), "formula.vc_fact_id")
                );
            }
            Ok(FormulaSource::GeneratedVcFact { vc_fact_id })
        }
        FormulaSourceClass::AcceptedImportedAxiom => Ok(FormulaSource::AcceptedImportedAxiom(
            read_imported_source(reader, formula_fingerprint)?,
        )),
        FormulaSourceClass::AcceptedImportedTheorem => Ok(FormulaSource::AcceptedImportedTheorem(
            read_imported_source(reader, formula_fingerprint)?,
        )),
        FormulaSourceClass::PolicyBoundedBuiltin => {
            let built_in_id = reader.read_bounded_bytes("formula.built_in_id", usize::MAX)?;
            if built_in_id.is_empty() {
                return Err(reader.missing_at(reader.offset(), "formula.built_in_id"));
            }
            Ok(FormulaSource::PolicyBoundedBuiltin { built_in_id })
        }
    }
}

fn read_imported_source(
    reader: &mut Reader<'_>,
    formula_fingerprint: &Fingerprint,
) -> FormulaEvidenceCheckResult<ImportedFormulaSource> {
    let package_id = reader.read_bounded_bytes("formula.package_id", usize::MAX)?;
    let module_path = reader.read_bounded_bytes("formula.module_path", usize::MAX)?;
    let exported_item_id = reader.read_bounded_bytes("formula.exported_item_id", usize::MAX)?;
    if package_id.is_empty() || module_path.is_empty() || exported_item_id.is_empty() {
        return Err(reader.missing_at(reader.offset(), "formula.imported_source"));
    }
    let statement_fingerprint = read_fingerprint(reader, "formula.statement_fingerprint")?;
    if &statement_fingerprint != formula_fingerprint {
        return Err(reader.missing_at(reader.offset(), "formula.statement_fingerprint"));
    }
    let status_offset = reader.offset();
    let status_tag = reader.read_u8("formula.required_proof_status")?;
    let Some(required_proof_status) = required_status_from_tag(status_tag) else {
        return Err(reader.malformed_at(status_offset, "formula.required_proof_status"));
    };
    Ok(ImportedFormulaSource {
        package_id,
        module_path,
        exported_item_id,
        statement_fingerprint,
        required_proof_status,
    })
}

fn parse_substitutions(
    bytes: &[u8],
    entry: &DirectoryEntry,
    target: &TargetVcFingerprint,
    formula_context: &ClauseValidationContext,
    formula_ids: &BTreeSet<u32>,
    context: &FormulaEvidenceParseContext,
) -> FormulaEvidenceCheckResult<Vec<FormulaSubstitutionEvidence>> {
    let frames = read_section_frames(bytes, entry, target)?;
    let mut substitutions = Vec::with_capacity(frames.len());
    let mut seen = BTreeSet::new();
    for frame in frames {
        let mut reader = frame.reader(target);
        let substitution_id = reader.read_u32("substitution.substitution_id")?;
        let source_formula_id = reader.read_u32("substitution.source_formula_id")?;
        if !formula_ids.contains(&source_formula_id) {
            return Err(reader.missing_at(
                reader.offset().saturating_sub(4),
                "substitution.source_formula_id",
            ));
        }
        let provenance_id = reader.read_u32("substitution.provenance_id")?;
        let binder_context_encoding = reader.read_bounded_bytes(
            "substitution.binder_context",
            context.limits.max_opaque_bytes,
        )?;
        let payload =
            read_substitution_payload(&mut reader, substitution_id, formula_context, context)?;
        let freshness_witnesses = read_freshness_witnesses(&mut reader, substitution_id, context)?;
        let free_variable_constraints =
            read_free_variable_constraints(&mut reader, substitution_id, context)?;
        reader.finish()?;
        if !seen.insert(substitution_id) {
            return Err(frame.malformed(target, "substitution.duplicate_id"));
        }
        substitutions.push(FormulaSubstitutionEvidence {
            substitution_id,
            source_formula_id,
            binder_context_encoding,
            payload,
            freshness_witnesses,
            free_variable_constraints,
            provenance_id,
        });
    }
    ensure_sorted_by(
        substitutions.iter().map(|entry| entry.substitution_id),
        entry,
        target,
        "substitution",
    )?;
    Ok(substitutions)
}

fn parse_provenance(
    bytes: &[u8],
    entry: &DirectoryEntry,
    target: &TargetVcFingerprint,
    context: &FormulaEvidenceParseContext,
) -> FormulaEvidenceCheckResult<Vec<FormulaProvenance>> {
    let frames = read_section_frames(bytes, entry, target)?;
    let mut provenance = Vec::with_capacity(frames.len());
    let mut seen = BTreeSet::new();
    for frame in frames {
        let mut reader = frame.reader(target);
        let provenance_id = reader.read_u32("provenance.provenance_id")?;
        let target_vc = read_fingerprint(&mut reader, "provenance.target_vc")?;
        let formula_fingerprint = read_fingerprint(&mut reader, "provenance.formula_fingerprint")?;
        let payload =
            reader.read_bounded_bytes("provenance.payload", context.limits.max_opaque_bytes)?;
        reader.finish()?;
        if payload.is_empty() {
            return Err(frame.missing(target, "provenance.payload"));
        }
        if !seen.insert(provenance_id) {
            return Err(frame.malformed(target, "provenance.duplicate_id"));
        }
        provenance.push(FormulaProvenance {
            provenance_id,
            target_vc,
            formula_fingerprint,
            payload,
        });
    }
    ensure_sorted_by(
        provenance.iter().map(|entry| entry.provenance_id),
        entry,
        target,
        "provenance",
    )?;
    Ok(provenance)
}

fn parse_final_goal(
    bytes: &[u8],
    entry: &DirectoryEntry,
    target: &TargetVcFingerprint,
    formula_context: &ClauseValidationContext,
    context: &FormulaEvidenceParseContext,
) -> FormulaEvidenceCheckResult<FinalGoalEvidence> {
    let frames = read_section_frames(bytes, entry, target)?;
    let Some(frame) = frames.first() else {
        return Err(certificate_rejection(
            target,
            RejectionDetail::MalformedWitnessData,
            RejectionLocation::new()
                .with_certificate_byte_offset(entry.entry_offset)
                .with_field_path("final_goal"),
        ));
    };
    let mut reader = frame.reader(target);
    let polarity_offset = reader.offset();
    let polarity_tag = reader.read_u8("final_goal.polarity")?;
    let Some(polarity) = GoalPolarity::from_tag(polarity_tag) else {
        return Err(reader.malformed_at(polarity_offset, "final_goal.polarity"));
    };
    let formula_fingerprint = read_fingerprint(&mut reader, "final_goal.fingerprint")?;
    let provenance_id = reader.read_u32("final_goal.provenance_id")?;
    let mut node_count = 0usize;
    let formula = read_formula(
        &mut reader,
        formula_context,
        context,
        0,
        &mut node_count,
        "final_goal.formula",
    )?;
    reader.finish()?;
    validate_formula_fingerprint(
        target,
        &formula,
        &formula_fingerprint,
        "final_goal.fingerprint",
    )?;
    Ok(FinalGoalEvidence {
        polarity,
        formula,
        formula_fingerprint,
        provenance_id,
    })
}

fn validate_provenance_bindings(
    target: &TargetVcFingerprint,
    expected_target: &Fingerprint,
    formulas: &[FormulaEvidenceEntry],
    substitutions: &[FormulaSubstitutionEvidence],
    provenance: &[FormulaProvenance],
    final_goal: &FinalGoalEvidence,
) -> FormulaEvidenceCheckResult<()> {
    for formula in formulas {
        let location = RejectionLocation::new().with_field_path("formula.provenance_id");
        validate_provenance_ref(
            target,
            expected_target,
            provenance,
            formula.provenance_id,
            &formula.formula_fingerprint,
            location,
        )?;
    }
    for substitution in substitutions {
        let Some(source_formula) = formulas
            .iter()
            .find(|formula| formula.formula_id == substitution.source_formula_id)
        else {
            return Err(kernel_rejection(
                target,
                RejectionDetail::MissingProvenance,
                RejectionLocation::new()
                    .with_substitution_id(substitution.substitution_id)
                    .with_field_path("substitution.source_formula_id"),
            ));
        };
        validate_provenance_ref(
            target,
            expected_target,
            provenance,
            substitution.provenance_id,
            &source_formula.formula_fingerprint,
            RejectionLocation::new()
                .with_substitution_id(substitution.substitution_id)
                .with_field_path("substitution.provenance_id"),
        )?;
    }
    validate_provenance_ref(
        target,
        expected_target,
        provenance,
        final_goal.provenance_id,
        &final_goal.formula_fingerprint,
        RejectionLocation::new()
            .with_final_goal()
            .with_field_path("final_goal.provenance_id"),
    )
}

fn validate_provenance_ref(
    target: &TargetVcFingerprint,
    expected_target: &Fingerprint,
    provenance: &[FormulaProvenance],
    provenance_id: u32,
    expected_formula: &Fingerprint,
    location: RejectionLocation,
) -> FormulaEvidenceCheckResult<()> {
    let Some(entry) = provenance
        .binary_search_by_key(&provenance_id, |entry| entry.provenance_id)
        .ok()
        .map(|index| &provenance[index])
    else {
        return Err(kernel_rejection(
            target,
            RejectionDetail::MissingProvenance,
            location,
        ));
    };
    if entry.target_vc != *expected_target || entry.formula_fingerprint != *expected_formula {
        return Err(kernel_rejection(
            target,
            RejectionDetail::MissingProvenance,
            location,
        ));
    }
    Ok(())
}

fn validate_formula_fingerprint(
    target: &TargetVcFingerprint,
    formula: &Formula,
    supplied: &Fingerprint,
    field: &'static str,
) -> FormulaEvidenceCheckResult<()> {
    if supplied.algorithm_id != SUPPORTED_FORMULA_FINGERPRINT_ALGORITHM_ID {
        return Err(kernel_rejection(
            target,
            RejectionDetail::MissingProvenance,
            RejectionLocation::new().with_field_path(field),
        ));
    }
    let canonical = formula.canonical_hash_input().map_err(|error| {
        let detail = if matches!(error, FormulaEvidenceError::ResourceExhaustion) {
            RejectionDetail::ResourceExhaustion
        } else {
            RejectionDetail::MissingProvenance
        };
        kernel_rejection(
            target,
            detail,
            RejectionLocation::new().with_field_path(field),
        )
    })?;
    let expected = Fingerprint::new(SUPPORTED_FORMULA_FINGERPRINT_ALGORITHM_ID, canonical);
    if &expected != supplied {
        return Err(kernel_rejection(
            target,
            RejectionDetail::MissingProvenance,
            RejectionLocation::new().with_field_path(field),
        ));
    }
    Ok(())
}

fn read_formula(
    reader: &mut Reader<'_>,
    formula_context: &ClauseValidationContext,
    context: &FormulaEvidenceParseContext,
    depth: usize,
    node_count: &mut usize,
    field: &'static str,
) -> FormulaEvidenceCheckResult<Formula> {
    if depth > context.limits.max_formula_depth {
        return Err(reader.resource_at(reader.offset(), field));
    }
    *node_count = node_count
        .checked_add(1)
        .ok_or_else(|| reader.resource_at(reader.offset(), field))?;
    if *node_count > context.limits.max_formula_nodes {
        return Err(reader.resource_at(reader.offset(), field));
    }
    let tag_offset = reader.offset();
    let tag = reader.read_u8(field)?;
    match tag {
        1 => {
            let atom = read_atom(reader, formula_context, context, depth, field)?;
            validate_atom(reader, &atom, formula_context, field)?;
            Ok(Formula::Atom(atom))
        }
        2 => {
            let child = read_formula(
                reader,
                formula_context,
                context,
                next_depth(reader, depth, field)?,
                node_count,
                field,
            )?;
            Ok(Formula::Not(Box::new(child)))
        }
        3 | 4 => {
            let count_offset = reader.offset();
            let child_count = reader.read_u32(field)? as usize;
            if child_count == 0 || child_count > context.limits.max_formula_children {
                return Err(reader.resource_at(count_offset, field));
            }
            ensure_count_fits_remaining(reader, child_count, 1, count_offset, field)?;
            let mut children = Vec::with_capacity(child_count);
            for _ in 0..child_count {
                children.push(read_formula(
                    reader,
                    formula_context,
                    context,
                    next_depth(reader, depth, field)?,
                    node_count,
                    field,
                )?);
            }
            if tag == 3 {
                Ok(Formula::And(children))
            } else {
                Ok(Formula::Or(children))
            }
        }
        _ => Err(reader.malformed_at(tag_offset, field)),
    }
}

fn read_atom(
    reader: &mut Reader<'_>,
    formula_context: &ClauseValidationContext,
    context: &FormulaEvidenceParseContext,
    depth: usize,
    field: &'static str,
) -> FormulaEvidenceCheckResult<Atom> {
    let kind = read_symbol_kind(reader, field)?;
    let symbol_id = reader.read_u32(field)?;
    let arity = reader.read_u32(field)?;
    let count_offset = reader.offset();
    let term_count = reader.read_u32(field)?;
    if term_count as usize > context.limits.max_formula_children {
        return Err(reader.resource_at(count_offset, field));
    }
    ensure_count_fits_remaining(reader, term_count as usize, 5, count_offset, field)?;
    let mut arguments = Vec::with_capacity(term_count as usize);
    for _ in 0..term_count {
        arguments.push(read_term(
            reader,
            formula_context,
            context,
            next_depth(reader, depth, field)?,
            field,
        )?);
    }
    Ok(Atom::with_arity(
        SymbolKey::new(kind, symbol_id),
        arity,
        arguments,
    ))
}

fn read_term(
    reader: &mut Reader<'_>,
    formula_context: &ClauseValidationContext,
    context: &FormulaEvidenceParseContext,
    depth: usize,
    field: &'static str,
) -> FormulaEvidenceCheckResult<Term> {
    let term_start = reader.offset();
    if depth > context.limits.max_term_recursion_depth {
        return Err(reader.resource_at(reader.offset(), field));
    }
    let tag_offset = reader.offset();
    let tag = reader.read_u8(field)?;
    let term = match tag {
        1 => Term::Variable(VariableId(reader.read_u32(field)?)),
        2 => {
            let kind = read_symbol_kind(reader, field)?;
            let symbol_id = reader.read_u32(field)?;
            let count_offset = reader.offset();
            let term_count = reader.read_u32(field)?;
            if term_count as usize > context.limits.max_formula_children {
                return Err(reader.resource_at(count_offset, field));
            }
            ensure_count_fits_remaining(reader, term_count as usize, 5, count_offset, field)?;
            let mut arguments = Vec::with_capacity(term_count as usize);
            for _ in 0..term_count {
                arguments.push(read_term(
                    reader,
                    formula_context,
                    context,
                    next_depth(reader, depth, field)?,
                    field,
                )?);
            }
            Term::Application {
                symbol: SymbolKey::new(kind, symbol_id),
                arguments,
            }
        }
        3 => {
            let binder_id = reader.read_u32(field)?;
            let body = read_term(
                reader,
                formula_context,
                context,
                next_depth(reader, depth, field)?,
                field,
            )?;
            Term::BinderNormalized {
                binder_id,
                body: Box::new(body),
            }
        }
        _ => return Err(reader.malformed_at(tag_offset, field)),
    };
    validate_term_record_size(reader, term_start, context, field)?;
    term.validate_for_kernel(formula_context)
        .map_err(|error| clause_error_rejection(reader, error, field))?;
    Ok(term)
}

fn read_substitution_payload(
    reader: &mut Reader<'_>,
    substitution_id: u32,
    formula_context: &ClauseValidationContext,
    context: &FormulaEvidenceParseContext,
) -> FormulaEvidenceCheckResult<SubstitutionPayload> {
    let owner_substitution_id = reader.read_u32("substitution.payload.owner_substitution_id")?;
    if owner_substitution_id != substitution_id {
        return Err(reader.missing_at(
            reader.offset().saturating_sub(4),
            "substitution.payload.owner_substitution_id",
        ));
    }
    let payload_kind_offset = reader.offset();
    let payload_kind = reader.read_u8("substitution.payload.payload_kind")?;
    if !matches!(
        payload_kind,
        PAYLOAD_KIND_FORMAL_TO_ACTUAL_MAP | PAYLOAD_KIND_LOCAL_ABBREVIATION_EXPANSION
    ) {
        return Err(reader.malformed_at(payload_kind_offset, "substitution.payload.payload_kind"));
    }
    let rewrite_path = read_term_path(reader, context, "substitution.payload.rewrite_path")?;
    let count_offset = reader.offset();
    let replacement_count = reader.read_u32("substitution.payload.replacement_count")? as usize;
    if replacement_count > context.limits.max_substitution_replacements {
        return Err(reader.resource_at(count_offset, "substitution.payload.replacement_count"));
    }
    ensure_count_fits_remaining(
        reader,
        replacement_count,
        10,
        count_offset,
        "substitution.payload.replacement_count",
    )?;
    let mut replacements = Vec::with_capacity(replacement_count);
    let mut seen_formals = BTreeSet::new();
    for _ in 0..replacement_count {
        let formal_variable_id =
            VariableId(reader.read_u32("substitution.payload.formal_variable_id")?);
        if !seen_formals.insert(formal_variable_id) {
            return Err(reader.malformed_at(
                reader.offset().saturating_sub(4),
                "substitution.payload.formal_variable_id",
            ));
        }
        let actual_term = read_term(
            reader,
            formula_context,
            context,
            0,
            "substitution.payload.actual_term",
        )?;
        let role_offset = reader.offset();
        let replacement_role = reader.read_u8("substitution.payload.replacement_role")?;
        if !matches!(
            replacement_role,
            REPLACEMENT_ROLE_TERM_ARGUMENT
                | REPLACEMENT_ROLE_PREDICATE_ARGUMENT
                | REPLACEMENT_ROLE_CAPTURED_FREE_VARIABLE
        ) {
            return Err(reader.malformed_at(role_offset, "substitution.payload.replacement_role"));
        }
        replacements.push(Replacement::new(
            formal_variable_id,
            actual_term,
            replacement_role,
        ));
    }
    Ok(SubstitutionPayload::new(
        owner_substitution_id,
        payload_kind,
        rewrite_path,
        replacements,
    ))
}

fn read_freshness_witnesses(
    reader: &mut Reader<'_>,
    substitution_id: u32,
    context: &FormulaEvidenceParseContext,
) -> FormulaEvidenceCheckResult<Vec<FreshnessWitness>> {
    let count_offset = reader.offset();
    let count = reader.read_u32("substitution.freshness_witness_count")? as usize;
    if count > context.limits.max_freshness_witnesses {
        return Err(reader.resource_at(count_offset, "substitution.freshness_witness_count"));
    }
    ensure_count_fits_remaining(
        reader,
        count,
        24,
        count_offset,
        "substitution.freshness_witness_count",
    )?;
    let mut witnesses = Vec::with_capacity(count);
    let mut seen = BTreeSet::new();
    for _ in 0..count {
        let witness_id = reader.read_u32("substitution.freshness_witness.witness_id")?;
        if !seen.insert(witness_id) {
            return Err(reader.malformed_at(
                reader.offset().saturating_sub(4),
                "substitution.freshness_witness.witness_id",
            ));
        }
        let owner_substitution_id =
            reader.read_u32("substitution.freshness_witness.owner_substitution_id")?;
        if owner_substitution_id != substitution_id {
            return Err(reader.missing_at(
                reader.offset().saturating_sub(4),
                "substitution.freshness_witness.owner_substitution_id",
            ));
        }
        let generated_variable_id =
            VariableId(reader.read_u32("substitution.freshness_witness.generated_variable_id")?);
        let binder_path = read_term_path(
            reader,
            context,
            "substitution.freshness_witness.binder_path",
        )?;
        let avoided_variables = read_variable_list(
            reader,
            context,
            "substitution.freshness_witness.avoided_variables",
        )?;
        let deterministic_counter =
            reader.read_u32("substitution.freshness_witness.deterministic_counter")?;
        witnesses.push(FreshnessWitness::new(
            witness_id,
            owner_substitution_id,
            generated_variable_id,
            binder_path,
            avoided_variables,
            deterministic_counter,
        ));
    }
    ensure_sorted_values(
        witnesses.iter().map(|witness| witness.witness_id),
        reader,
        "substitution.freshness_witness",
    )?;
    Ok(witnesses)
}

fn read_free_variable_constraints(
    reader: &mut Reader<'_>,
    substitution_id: u32,
    context: &FormulaEvidenceParseContext,
) -> FormulaEvidenceCheckResult<Vec<FreeVariableConstraint>> {
    let count_offset = reader.offset();
    let count = reader.read_u32("substitution.free_variable_constraint_count")? as usize;
    if count > context.limits.max_free_variable_constraints {
        return Err(reader.resource_at(count_offset, "substitution.free_variable_constraint_count"));
    }
    ensure_count_fits_remaining(
        reader,
        count,
        21,
        count_offset,
        "substitution.free_variable_constraint_count",
    )?;
    let mut constraints = Vec::with_capacity(count);
    let mut seen = BTreeSet::new();
    for _ in 0..count {
        let constraint_id =
            reader.read_u32("substitution.free_variable_constraint.constraint_id")?;
        if !seen.insert(constraint_id) {
            return Err(reader.malformed_at(
                reader.offset().saturating_sub(4),
                "substitution.free_variable_constraint.constraint_id",
            ));
        }
        let owner_substitution_id =
            reader.read_u32("substitution.free_variable_constraint.owner_substitution_id")?;
        if owner_substitution_id != substitution_id {
            return Err(reader.missing_at(
                reader.offset().saturating_sub(4),
                "substitution.free_variable_constraint.owner_substitution_id",
            ));
        }
        let kind_offset = reader.offset();
        let constraint_kind =
            reader.read_u8("substitution.free_variable_constraint.constraint_kind")?;
        if !matches!(
            constraint_kind,
            CONSTRAINT_KIND_MUST_REMAIN_FREE_AT_PATH
                | CONSTRAINT_KIND_MUST_BE_ABSENT_FROM_CAPTURE_SET
        ) {
            return Err(reader.malformed_at(
                kind_offset,
                "substitution.free_variable_constraint.constraint_kind",
            ));
        }
        let variable_id =
            VariableId(reader.read_u32("substitution.free_variable_constraint.variable_id")?);
        let term_path = read_term_path(
            reader,
            context,
            "substitution.free_variable_constraint.term_path",
        )?;
        let capture_set = read_variable_list(
            reader,
            context,
            "substitution.free_variable_constraint.capture_set",
        )?;
        constraints.push(FreeVariableConstraint::new(
            constraint_id,
            owner_substitution_id,
            constraint_kind,
            variable_id,
            term_path,
            capture_set,
        ));
    }
    ensure_sorted_values(
        constraints
            .iter()
            .map(|constraint| constraint.constraint_id),
        reader,
        "substitution.free_variable_constraint",
    )?;
    Ok(constraints)
}

fn read_term_path(
    reader: &mut Reader<'_>,
    context: &FormulaEvidenceParseContext,
    field: &'static str,
) -> FormulaEvidenceCheckResult<TermPath> {
    let count_offset = reader.offset();
    let count = reader.read_u32(field)? as usize;
    if count > context.limits.max_term_path_segments {
        return Err(reader.resource_at(count_offset, field));
    }
    ensure_count_fits_remaining(reader, count, 5, count_offset, field)?;
    let mut segments = Vec::with_capacity(count);
    for _ in 0..count {
        let edge_offset = reader.offset();
        let edge_kind = reader.read_u8(field)?;
        let child_index = reader.read_u32(field)?;
        match edge_kind {
            EDGE_KIND_APPLICATION_ARGUMENT => {}
            EDGE_KIND_BINDER_BODY if child_index == 0 => {}
            EDGE_KIND_BINDER_BODY => return Err(reader.malformed_at(edge_offset, field)),
            _ => return Err(reader.malformed_at(edge_offset, field)),
        }
        segments.push(TermPathSegment::new(edge_kind, child_index));
    }
    Ok(TermPath::new(segments))
}

fn read_variable_list(
    reader: &mut Reader<'_>,
    context: &FormulaEvidenceParseContext,
    field: &'static str,
) -> FormulaEvidenceCheckResult<Vec<VariableId>> {
    let count_offset = reader.offset();
    let count = reader.read_u32(field)? as usize;
    if count > context.limits.max_variable_list_entries {
        return Err(reader.resource_at(count_offset, field));
    }
    ensure_count_fits_remaining(reader, count, 4, count_offset, field)?;
    let mut variables = Vec::with_capacity(count);
    let mut seen = BTreeSet::new();
    for _ in 0..count {
        let variable = VariableId(reader.read_u32(field)?);
        if !seen.insert(variable) {
            return Err(reader.malformed_at(reader.offset().saturating_sub(4), field));
        }
        variables.push(variable);
    }
    ensure_sorted_values(variables.iter().map(|variable| variable.0), reader, field)?;
    Ok(variables)
}

fn read_kernel_profile(reader: &mut Reader<'_>) -> FormulaEvidenceCheckResult<KernelProfileRecord> {
    let profile_id = reader.read_u16("kernel_profile.profile_id")?;
    let clause_schema_version = reader.read_u16("kernel_profile.clause_schema_version")?;
    let clause_encoding_version = reader.read_u16("kernel_profile.clause_encoding_version")?;
    let tautology_offset = reader.offset();
    let tautology_tag = reader.read_u8("kernel_profile.clause_tautology_policy")?;
    let Some(clause_tautology_policy) = clause_tautology_from_tag(tautology_tag) else {
        return Err(
            reader.unsupported_at(tautology_offset, "kernel_profile.clause_tautology_policy")
        );
    };
    let hash_offset = reader.offset();
    let hash_tag = reader.read_u8("kernel_profile.certificate_hash_input_algorithm")?;
    let Some(certificate_hash_input_algorithm) = hash_algorithm_from_tag(hash_tag) else {
        return Err(reader.unsupported_at(
            hash_offset,
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
) -> FormulaEvidenceCheckResult<Fingerprint> {
    let algorithm_id = reader.read_u8(field)?;
    let digest = reader.read_bounded_bytes(field, usize::MAX)?;
    Ok(Fingerprint {
        algorithm_id,
        digest,
    })
}

fn read_symbol_kind(
    reader: &mut Reader<'_>,
    field: &'static str,
) -> FormulaEvidenceCheckResult<SymbolKind> {
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

fn required_status_from_tag(tag: u8) -> Option<RequiredProofStatus> {
    match tag {
        1 => Some(RequiredProofStatus::KernelVerified),
        2 => Some(RequiredProofStatus::DischargedBuiltin),
        3 => Some(RequiredProofStatus::ExternallyAttestedPolicyPermitted),
        _ => None,
    }
}

fn clause_tautology_from_tag(tag: u8) -> Option<ClauseTautologyPolicy> {
    match tag {
        1 => Some(ClauseTautologyPolicy::Reject),
        2 => Some(ClauseTautologyPolicy::Marker),
        _ => None,
    }
}

fn hash_algorithm_from_tag(tag: u8) -> Option<CertificateHashInputAlgorithm> {
    match tag {
        1 => Some(CertificateHashInputAlgorithm::CanonicalEnvelopeV1),
        _ => None,
    }
}

fn formula_validation_context(
    profile: &KernelProfileRecord,
    symbols: &[SymbolManifestEntry],
    variables: &[VariableManifestEntry],
    context: &FormulaEvidenceParseContext,
) -> ClauseValidationContext {
    let profile = ClauseProfile::new(
        profile.clause_schema_version,
        profile.clause_encoding_version,
        profile.clause_tautology_policy.into(),
    );
    let mut clause_context = ClauseValidationContext::new(profile)
        .with_limits(usize::MAX, context.limits.max_term_encoding_bytes)
        .with_max_term_recursion_depth(context.limits.max_term_recursion_depth);
    for symbol in symbols {
        clause_context = clause_context.with_known_symbol(symbol.symbol);
    }
    for variable in variables {
        clause_context = clause_context.with_canonical_variable(variable.variable_id);
    }
    clause_context
}

fn validate_atom(
    reader: &Reader<'_>,
    atom: &Atom,
    context: &ClauseValidationContext,
    field: &'static str,
) -> FormulaEvidenceCheckResult<()> {
    Clause::normalize(
        vec![Literal::new(Polarity::Positive, atom.clone())],
        context,
    )
    .map(|_| ())
    .map_err(|error| clause_error_rejection(reader, error, field))
}

fn validate_term_record_size(
    reader: &Reader<'_>,
    term_start: usize,
    context: &FormulaEvidenceParseContext,
    field: &'static str,
) -> FormulaEvidenceCheckResult<()> {
    let Some(size) = reader.offset().checked_sub(term_start) else {
        return Err(reader.resource_at(reader.offset(), field));
    };
    if size > context.limits.max_term_encoding_bytes {
        return Err(reader.resource_at(term_start, field));
    }
    Ok(())
}

fn ensure_count_fits_remaining(
    reader: &Reader<'_>,
    count: usize,
    min_item_bytes: usize,
    count_offset: usize,
    field: &'static str,
) -> FormulaEvidenceCheckResult<()> {
    let Some(min_bytes) = count.checked_mul(min_item_bytes) else {
        return Err(reader.resource_at(count_offset, field));
    };
    if min_bytes > reader.remaining() {
        return Err(reader.resource_at(count_offset, field));
    }
    Ok(())
}

fn clause_error_rejection(
    reader: &Reader<'_>,
    error: ClauseError,
    field: &'static str,
) -> Box<RejectionRecord> {
    let detail = if is_resource_clause_error(&error) {
        RejectionDetail::ResourceExhaustion
    } else {
        RejectionDetail::MalformedWitnessData
    };
    certificate_rejection(
        reader.target,
        detail,
        RejectionLocation::new()
            .with_certificate_byte_offset(reader.offset())
            .with_field_path(field),
    )
}

fn is_resource_clause_error(error: &ClauseError) -> bool {
    matches!(
        error,
        ClauseError::LiteralCountExceeded { .. }
            | ClauseError::TermSizeExceeded { .. }
            | ClauseError::TermRecursionDepthExceeded { .. }
    )
}

fn next_depth(
    reader: &Reader<'_>,
    depth: usize,
    field: &'static str,
) -> FormulaEvidenceCheckResult<usize> {
    depth
        .checked_add(1)
        .ok_or_else(|| reader.resource_at(reader.offset(), field))
}

fn render_children(operator: &str, children: &[Formula]) -> String {
    let mut rendered = format!("{operator}(");
    for (index, child) in children.iter().enumerate() {
        if index > 0 {
            rendered.push(',');
        }
        rendered.push_str(&child.render());
    }
    rendered.push(')');
    rendered
}

fn child_frame(tag: u8, children: &[Formula]) -> Result<Vec<u8>, FormulaEvidenceError> {
    if children.is_empty() {
        return Err(FormulaEvidenceError::ResourceExhaustion);
    }
    let mut payload = Vec::new();
    write_len(children.len(), &mut payload)?;
    for child in children {
        payload.extend(child.canonical_bytes()?);
    }
    frame(FORMULA_FRAME_TAG, tag, payload)
}

fn imported_source_bytes(source: &ImportedFormulaSource) -> Result<Vec<u8>, FormulaEvidenceError> {
    let mut bytes = Vec::new();
    write_len(source.package_id.len(), &mut bytes)?;
    bytes.extend_from_slice(&source.package_id);
    write_len(source.module_path.len(), &mut bytes)?;
    bytes.extend_from_slice(&source.module_path);
    write_len(source.exported_item_id.len(), &mut bytes)?;
    bytes.extend_from_slice(&source.exported_item_id);
    bytes.extend(fingerprint_bytes(&source.statement_fingerprint)?);
    bytes.push(required_status_tag(source.required_proof_status));
    Ok(bytes)
}

fn required_status_tag(status: RequiredProofStatus) -> u8 {
    match status {
        RequiredProofStatus::KernelVerified => 1,
        RequiredProofStatus::DischargedBuiltin => 2,
        RequiredProofStatus::ExternallyAttestedPolicyPermitted => 3,
    }
}

fn fingerprint_bytes(fingerprint: &Fingerprint) -> Result<Vec<u8>, FormulaEvidenceError> {
    let mut bytes = vec![fingerprint.algorithm_id];
    write_len(fingerprint.digest.len(), &mut bytes)?;
    bytes.extend_from_slice(&fingerprint.digest);
    Ok(bytes)
}

fn frame(tag: u8, subtype: u8, payload: Vec<u8>) -> Result<Vec<u8>, FormulaEvidenceError> {
    let mut bytes = vec![tag, subtype];
    write_len(payload.len(), &mut bytes)?;
    bytes.extend(payload);
    Ok(bytes)
}

fn write_len(len: usize, bytes: &mut Vec<u8>) -> Result<(), FormulaEvidenceError> {
    let len = u32::try_from(len).map_err(|_| FormulaEvidenceError::ResourceExhaustion)?;
    bytes.extend_from_slice(&len.to_be_bytes());
    Ok(())
}

fn ensure_sorted_by<T>(
    values: impl Iterator<Item = T>,
    entry: &DirectoryEntry,
    target: &TargetVcFingerprint,
    field: &'static str,
) -> FormulaEvidenceCheckResult<()>
where
    T: Ord,
{
    let mut previous = None;
    for value in values {
        if previous.as_ref().is_some_and(|previous| previous >= &value) {
            return Err(certificate_rejection(
                target,
                RejectionDetail::MalformedWitnessData,
                RejectionLocation::new()
                    .with_certificate_byte_offset(entry.entry_offset)
                    .with_field_path(field),
            ));
        }
        previous = Some(value);
    }
    Ok(())
}

fn ensure_sorted_values(
    values: impl Iterator<Item = u32>,
    reader: &Reader<'_>,
    field: &'static str,
) -> FormulaEvidenceCheckResult<()> {
    let mut previous = None;
    for value in values {
        if previous.is_some_and(|previous| previous >= value) {
            return Err(reader.malformed_at(reader.offset(), field));
        }
        previous = Some(value);
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct ItemFrame<'a> {
    payload: &'a [u8],
    offset: usize,
    item_index: u32,
}

impl<'a> ItemFrame<'a> {
    fn reader(self, target: &'a TargetVcFingerprint) -> Reader<'a> {
        Reader::new(self.payload, self.offset + 6, target, Some(self.item_index))
    }

    fn malformed(self, target: &TargetVcFingerprint, field: &'static str) -> Box<RejectionRecord> {
        certificate_rejection(
            target,
            RejectionDetail::MalformedWitnessData,
            RejectionLocation::new()
                .with_certificate_byte_offset(self.offset)
                .with_item_index(self.item_index)
                .with_field_path(field),
        )
    }

    fn missing(self, target: &TargetVcFingerprint, field: &'static str) -> Box<RejectionRecord> {
        kernel_rejection(
            target,
            RejectionDetail::MissingProvenance,
            RejectionLocation::new()
                .with_certificate_byte_offset(self.offset)
                .with_item_index(self.item_index)
                .with_field_path(field),
        )
    }
}

fn read_section_frames<'a>(
    bytes: &'a [u8],
    entry: &DirectoryEntry,
    target: &'a TargetVcFingerprint,
) -> FormulaEvidenceCheckResult<Vec<ItemFrame<'a>>> {
    if entry.item_count as usize > bytes.len() / 6 {
        return Err(certificate_rejection(
            target,
            RejectionDetail::MalformedWitnessData,
            RejectionLocation::new()
                .with_certificate_byte_offset(entry.entry_offset + 1)
                .with_field_path("directory.item_count"),
        ));
    }
    let mut reader = Reader::new(
        bytes,
        entry.payload_base_offset + entry.payload_offset as usize,
        target,
        None,
    );
    let mut frames = Vec::new();
    for item_index in 0..entry.item_count {
        reader.item_index = Some(item_index);
        let frame_offset = reader.offset();
        let section_tag_byte = reader.read_u8("item.section_tag")?;
        if section_tag_byte != entry.section_tag.byte() {
            return Err(reader.malformed_at(frame_offset, "item.section_tag"));
        }
        let item_tag = reader.read_u8("item.item_tag")?;
        if item_tag != 1 {
            return Err(reader.malformed_at(frame_offset + 1, "item.item_tag"));
        }
        let length = reader.read_u32("item.length")? as usize;
        let payload = reader.read_exact(length, "item.payload")?;
        frames.push(ItemFrame {
            payload,
            offset: frame_offset,
            item_index,
        });
    }
    reader.item_index = None;
    reader.finish()?;
    Ok(frames)
}

struct Reader<'a> {
    bytes: &'a [u8],
    cursor: usize,
    base_offset: usize,
    target: &'a TargetVcFingerprint,
    item_index: Option<u32>,
}

impl<'a> Reader<'a> {
    const fn new(
        bytes: &'a [u8],
        base_offset: usize,
        target: &'a TargetVcFingerprint,
        item_index: Option<u32>,
    ) -> Self {
        Self {
            bytes,
            cursor: 0,
            base_offset,
            target,
            item_index,
        }
    }

    const fn offset(&self) -> usize {
        self.base_offset + self.cursor
    }

    const fn remaining(&self) -> usize {
        self.bytes.len().saturating_sub(self.cursor)
    }

    fn read_u8(&mut self, field: &'static str) -> FormulaEvidenceCheckResult<u8> {
        let bytes = self.read_exact(1, field)?;
        Ok(bytes[0])
    }

    fn read_u16(&mut self, field: &'static str) -> FormulaEvidenceCheckResult<u16> {
        let bytes = self.read_exact(2, field)?;
        Ok(u16::from_be_bytes([bytes[0], bytes[1]]))
    }

    fn read_u32(&mut self, field: &'static str) -> FormulaEvidenceCheckResult<u32> {
        let bytes = self.read_exact(4, field)?;
        Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn read_bounded_bytes(
        &mut self,
        field: &'static str,
        max: usize,
    ) -> FormulaEvidenceCheckResult<Vec<u8>> {
        let len_offset = self.offset();
        let len = self.read_u32(field)? as usize;
        if len > max {
            return Err(self.resource_at(len_offset, field));
        }
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
    ) -> FormulaEvidenceCheckResult<&'a [u8]> {
        let start = self.cursor;
        let Some(end) = start.checked_add(len) else {
            return Err(self.resource_at(self.offset(), field));
        };
        if end > self.bytes.len() {
            return Err(self.malformed_at(self.offset(), field));
        }
        self.cursor = end;
        Ok(&self.bytes[start..end])
    }

    fn finish(&self) -> FormulaEvidenceCheckResult<()> {
        if self.cursor == self.bytes.len() {
            Ok(())
        } else {
            Err(self.malformed_at(self.offset(), "trailing_bytes"))
        }
    }

    fn unsupported_at(&self, byte_offset: usize, field: &'static str) -> Box<RejectionRecord> {
        certificate_rejection(
            self.target,
            RejectionDetail::UnsupportedCertificateFormat,
            self.location(byte_offset, field),
        )
    }

    fn malformed_at(&self, byte_offset: usize, field: &'static str) -> Box<RejectionRecord> {
        certificate_rejection(
            self.target,
            RejectionDetail::MalformedWitnessData,
            self.location(byte_offset, field),
        )
    }

    fn missing_at(&self, byte_offset: usize, field: &'static str) -> Box<RejectionRecord> {
        kernel_rejection(
            self.target,
            RejectionDetail::MissingProvenance,
            self.location(byte_offset, field),
        )
    }

    fn resource_at(&self, byte_offset: usize, field: &'static str) -> Box<RejectionRecord> {
        certificate_rejection(
            self.target,
            RejectionDetail::ResourceExhaustion,
            self.location(byte_offset, field),
        )
    }

    fn location(&self, byte_offset: usize, field: &'static str) -> RejectionLocation {
        let mut location = RejectionLocation::new()
            .with_certificate_byte_offset(byte_offset)
            .with_field_path(field);
        if let Some(item_index) = self.item_index {
            location = location.with_item_index(item_index);
        }
        location
    }
}

fn certificate_rejection(
    target: &TargetVcFingerprint,
    detail: RejectionDetail,
    location: RejectionLocation,
) -> Box<RejectionRecord> {
    rejection(
        target,
        RejectionCategory::CertificateRejection,
        detail,
        location,
    )
}

fn kernel_rejection(
    target: &TargetVcFingerprint,
    detail: RejectionDetail,
    location: RejectionLocation,
) -> Box<RejectionRecord> {
    rejection(target, RejectionCategory::KernelRejection, detail, location)
}

fn rejection(
    target: &TargetVcFingerprint,
    category: RejectionCategory,
    detail: RejectionDetail,
    location: RejectionLocation,
) -> Box<RejectionRecord> {
    Box::new(
        RejectionRecord::new(target.clone(), category, detail, location)
            .expect("formula evidence parser uses valid rejection detail mappings"),
    )
}

#[cfg(test)]
mod tests;
