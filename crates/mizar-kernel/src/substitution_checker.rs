use std::collections::BTreeSet;

use crate::{
    certificate_parser::{ParsedCertificate, SubstitutionEntry},
    clause::{
        ClauseError, ClauseProfile, ClauseValidationContext, Term, VariableId,
        application_term_len_for_kernel, binder_term_len_for_kernel,
    },
    rejection::{
        RejectionCategory, RejectionDetail, RejectionLocation, RejectionRecord, TargetVcFingerprint,
    },
};

const BINDER_CONTEXT_SCHEMA_VERSION: u16 = 1;
const PAYLOAD_KIND_FORMAL_TO_ACTUAL_MAP: u8 = 1;
const PAYLOAD_KIND_LOCAL_ABBREVIATION_EXPANSION: u8 = 2;
const REPLACEMENT_ROLE_TERM_ARGUMENT: u8 = 1;
const REPLACEMENT_ROLE_PREDICATE_ARGUMENT: u8 = 2;
const REPLACEMENT_ROLE_CAPTURED_FREE_VARIABLE: u8 = 3;
const EDGE_KIND_APPLICATION_ARGUMENT: u8 = 1;
const EDGE_KIND_BINDER_BODY: u8 = 2;
const CONSTRAINT_KIND_MUST_REMAIN_FREE_AT_PATH: u8 = 1;
const CONSTRAINT_KIND_MUST_BE_ABSENT_FROM_CAPTURE_SET: u8 = 2;
const BINDER_FRAME_ENCODED_BYTES: usize = 13;
const BINDER_VARIABLE_ENCODED_BYTES: usize = 4;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SubstitutionReplayLimits {
    pub max_substitutions: usize,
    pub max_binder_context_bytes: usize,
    pub max_binder_frames: usize,
    pub max_freshness_witnesses: usize,
    pub max_free_variable_constraints: usize,
    pub max_term_encoding_bytes: usize,
    pub max_term_recursion_depth: usize,
    pub max_alpha_renames: usize,
    pub max_payload_replacements: usize,
    pub max_term_path_segments: usize,
    pub max_avoided_variables: usize,
    pub max_capture_set_variables: usize,
}

impl Default for SubstitutionReplayLimits {
    fn default() -> Self {
        Self {
            max_substitutions: usize::MAX,
            max_binder_context_bytes: usize::MAX,
            max_binder_frames: usize::MAX,
            max_freshness_witnesses: usize::MAX,
            max_free_variable_constraints: usize::MAX,
            max_term_encoding_bytes: usize::MAX,
            max_term_recursion_depth: usize::MAX,
            max_alpha_renames: usize::MAX,
            max_payload_replacements: usize::MAX,
            max_term_path_segments: usize::MAX,
            max_avoided_variables: usize::MAX,
            max_capture_set_variables: usize::MAX,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SubstitutionCheckInput<'a> {
    pub target_vc_fingerprint: &'a TargetVcFingerprint,
    pub certificate: &'a ParsedCertificate,
    pub substitution_context: Option<&'a SubstitutionContext>,
    pub limits: SubstitutionReplayLimits,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubstitutionPayloadEntry {
    pub substitution_id: u32,
    pub payload: SubstitutionPayload,
}

impl SubstitutionPayloadEntry {
    #[must_use]
    pub const fn new(substitution_id: u32, payload: SubstitutionPayload) -> Self {
        Self {
            substitution_id,
            payload,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubstitutionPayload {
    pub owner_substitution_id: u32,
    pub payload_kind: u8,
    pub rewrite_path: TermPath,
    pub replacements: Vec<Replacement>,
}

impl SubstitutionPayload {
    #[must_use]
    pub const fn new(
        owner_substitution_id: u32,
        payload_kind: u8,
        rewrite_path: TermPath,
        replacements: Vec<Replacement>,
    ) -> Self {
        Self {
            owner_substitution_id,
            payload_kind,
            rewrite_path,
            replacements,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Replacement {
    pub formal_variable_id: VariableId,
    pub actual_term: Term,
    pub replacement_role: u8,
}

impl Replacement {
    #[must_use]
    pub const fn new(
        formal_variable_id: VariableId,
        actual_term: Term,
        replacement_role: u8,
    ) -> Self {
        Self {
            formal_variable_id,
            actual_term,
            replacement_role,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FreshnessWitness {
    pub witness_id: u32,
    pub owner_substitution_id: u32,
    pub generated_variable_id: VariableId,
    pub binder_path: TermPath,
    pub avoided_variables: Vec<VariableId>,
    pub deterministic_counter: u32,
}

impl FreshnessWitness {
    #[must_use]
    pub const fn new(
        witness_id: u32,
        owner_substitution_id: u32,
        generated_variable_id: VariableId,
        binder_path: TermPath,
        avoided_variables: Vec<VariableId>,
        deterministic_counter: u32,
    ) -> Self {
        Self {
            witness_id,
            owner_substitution_id,
            generated_variable_id,
            binder_path,
            avoided_variables,
            deterministic_counter,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FreeVariableConstraint {
    pub constraint_id: u32,
    pub owner_substitution_id: u32,
    pub constraint_kind: u8,
    pub variable_id: VariableId,
    pub term_path: TermPath,
    pub capture_set: Vec<VariableId>,
}

impl FreeVariableConstraint {
    #[must_use]
    pub const fn new(
        constraint_id: u32,
        owner_substitution_id: u32,
        constraint_kind: u8,
        variable_id: VariableId,
        term_path: TermPath,
        capture_set: Vec<VariableId>,
    ) -> Self {
        Self {
            constraint_id,
            owner_substitution_id,
            constraint_kind,
            variable_id,
            term_path,
            capture_set,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct TermPath {
    pub segments: Vec<TermPathSegment>,
}

impl TermPath {
    #[must_use]
    pub const fn new(segments: Vec<TermPathSegment>) -> Self {
        Self { segments }
    }

    #[must_use]
    pub const fn root() -> Self {
        Self {
            segments: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct TermPathSegment {
    pub edge_kind: u8,
    pub child_index: u32,
}

impl TermPathSegment {
    #[must_use]
    pub const fn new(edge_kind: u8, child_index: u32) -> Self {
        Self {
            edge_kind,
            child_index,
        }
    }

    #[must_use]
    pub const fn application_argument(child_index: u32) -> Self {
        Self::new(EDGE_KIND_APPLICATION_ARGUMENT, child_index)
    }

    #[must_use]
    pub const fn binder_body() -> Self {
        Self::new(EDGE_KIND_BINDER_BODY, 0)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubstitutionContext {
    provenance_fingerprint: Option<Vec<u8>>,
    substitution_payloads: Vec<SubstitutionPayloadEntry>,
    freshness_witnesses: Vec<FreshnessWitness>,
    free_variable_constraints: Vec<FreeVariableConstraint>,
    canonical_shape: bool,
}

impl SubstitutionContext {
    pub fn new(
        provenance_fingerprint: Option<Vec<u8>>,
        substitution_payloads: Vec<SubstitutionPayloadEntry>,
        freshness_witnesses: Vec<FreshnessWitness>,
        free_variable_constraints: Vec<FreeVariableConstraint>,
    ) -> Result<Self, SubstitutionContextError> {
        Ok(Self {
            provenance_fingerprint,
            substitution_payloads: canonical_payloads(substitution_payloads)?,
            freshness_witnesses: canonical_freshness_witnesses(freshness_witnesses)?,
            free_variable_constraints: canonical_free_variable_constraints(
                free_variable_constraints,
            )?,
            canonical_shape: true,
        })
    }

    #[must_use]
    pub fn provenance_fingerprint(&self) -> Option<&[u8]> {
        self.provenance_fingerprint.as_deref()
    }

    fn payload_for(&self, substitution_id: u32) -> Result<&SubstitutionPayloadEntry, ()> {
        if self.canonical_shape {
            unique_binary_lookup(&self.substitution_payloads, substitution_id, |entry| {
                entry.substitution_id
            })
        } else {
            unique_linear_lookup(&self.substitution_payloads, substitution_id, |entry| {
                entry.substitution_id
            })
        }
    }

    fn freshness_witness(&self, witness_id: u32) -> Result<&FreshnessWitness, ()> {
        if self.canonical_shape {
            unique_binary_lookup(&self.freshness_witnesses, witness_id, |witness| {
                witness.witness_id
            })
        } else {
            unique_linear_lookup(&self.freshness_witnesses, witness_id, |witness| {
                witness.witness_id
            })
        }
    }

    fn free_variable_constraint(&self, constraint_id: u32) -> Result<&FreeVariableConstraint, ()> {
        if self.canonical_shape {
            unique_binary_lookup(
                &self.free_variable_constraints,
                constraint_id,
                |constraint| constraint.constraint_id,
            )
        } else {
            unique_linear_lookup(
                &self.free_variable_constraints,
                constraint_id,
                |constraint| constraint.constraint_id,
            )
        }
    }
}

fn unique_binary_lookup<T>(entries: &[T], id: u32, key: impl Fn(&T) -> u32) -> Result<&T, ()> {
    let index = entries.binary_search_by_key(&id, &key).map_err(|_| ())?;
    if index > 0 && key(&entries[index - 1]) == id {
        return Err(());
    }
    if entries
        .get(index + 1)
        .is_some_and(|next_entry| key(next_entry) == id)
    {
        return Err(());
    }
    Ok(&entries[index])
}

fn unique_linear_lookup<T>(entries: &[T], id: u32, key: impl Fn(&T) -> u32) -> Result<&T, ()> {
    let mut found = None;
    for entry in entries {
        if key(entry) == id && found.replace(entry).is_some() {
            return Err(());
        }
    }
    found.ok_or(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SubstitutionContextError {
    DuplicateSubstitutionPayload { substitution_id: u32 },
    DuplicateFreshnessWitness { witness_id: u32 },
    DuplicateFreeVariableConstraint { constraint_id: u32 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubstitutionCheckReport<'a> {
    checked_substitutions: Vec<CheckedSubstitution>,
    target_vc_fingerprint: &'a TargetVcFingerprint,
    certificate_hash_input: &'a [u8],
    substitution_context_provenance: Option<&'a [u8]>,
}

impl SubstitutionCheckReport<'_> {
    #[must_use]
    pub fn checked_substitutions(&self) -> &[CheckedSubstitution] {
        &self.checked_substitutions
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedSubstitution {
    pub substitution_id: u32,
    pub source_term: Term,
    pub target_term: Term,
}

pub type SubstitutionCheckResult<T> = Result<T, Box<RejectionRecord>>;

pub fn replay_substitutions<'a>(
    input: SubstitutionCheckInput<'a>,
) -> SubstitutionCheckResult<SubstitutionCheckReport<'a>> {
    if input.certificate.substitutions.len() > input.limits.max_substitutions {
        let location = input
            .certificate
            .substitutions
            .first()
            .map_or_else(RejectionLocation::new, substitution_location);
        return Err(resource_rejection(input.target_vc_fingerprint, location));
    }

    let replay_context = ReplayContext::new(input.certificate, input.limits);
    let mut checked_substitutions = Vec::with_capacity(input.certificate.substitutions.len());
    for entry in &input.certificate.substitutions {
        let binder_context = decode_binder_context(input, &replay_context, entry)?;
        validate_term_for_entry(
            input,
            &replay_context,
            &binder_context,
            entry,
            &entry.source_term,
            "source_term",
        )?;
        validate_term_for_entry(
            input,
            &replay_context,
            &binder_context,
            entry,
            &entry.target_term,
            "target_term",
        )?;
        let context = substitution_context_for(input, entry)?;
        let payload = validate_payload(input, &replay_context, &binder_context, context, entry)?;
        validate_binder_context_usage(input, &binder_context, entry, payload)?;
        validate_side_conditions(input, &replay_context, &binder_context, context, entry)?;
        measure_direct_substitution(input, &binder_context, entry, payload)?;
        let replayed = replay_direct_substitution(input, &binder_context, entry, payload)?;
        validate_term_for_entry(
            input,
            &replay_context,
            &binder_context,
            entry,
            &replayed,
            "target_term",
        )?;
        if replayed != entry.target_term {
            return Err(invalid_rejection(
                input.target_vc_fingerprint,
                substitution_location(entry).with_field_path("target_term"),
            ));
        }
        checked_substitutions.push(CheckedSubstitution {
            substitution_id: entry.substitution_id,
            source_term: entry.source_term.clone(),
            target_term: entry.target_term.clone(),
        });
    }

    Ok(SubstitutionCheckReport {
        checked_substitutions,
        target_vc_fingerprint: input.target_vc_fingerprint,
        certificate_hash_input: input.certificate.canonical_hash_input(),
        substitution_context_provenance: input_context_provenance(input),
    })
}

pub fn checked_substitutions_for_input<'a>(
    input: SubstitutionCheckInput<'_>,
    report: &'a SubstitutionCheckReport<'_>,
) -> SubstitutionCheckResult<&'a [CheckedSubstitution]> {
    if report.target_vc_fingerprint != input.target_vc_fingerprint
        || report.certificate_hash_input != input.certificate.canonical_hash_input()
        || report.substitution_context_provenance != input_context_provenance(input)
    {
        return Err(invalid_rejection(
            input.target_vc_fingerprint,
            RejectionLocation::new().with_field_path("substitution_report_binding"),
        ));
    }
    Ok(report.checked_substitutions())
}

struct ReplayContext {
    term: ClauseValidationContext,
    canonical_variables: BTreeSet<VariableId>,
}

impl ReplayContext {
    fn new(certificate: &ParsedCertificate, limits: SubstitutionReplayLimits) -> Self {
        let profile = ClauseProfile::new(
            certificate.kernel_profile.clause_schema_version,
            certificate.kernel_profile.clause_encoding_version,
            certificate.kernel_profile.clause_tautology_policy.into(),
        );
        let mut term = ClauseValidationContext::new(profile)
            .with_limits(usize::MAX, limits.max_term_encoding_bytes)
            .with_max_term_recursion_depth(limits.max_term_recursion_depth);
        for symbol in &certificate.symbol_manifest {
            term = term.with_known_symbol(symbol.symbol);
        }
        let mut canonical_variables = BTreeSet::new();
        for variable in &certificate.variable_manifest {
            term = term.with_canonical_variable(variable.variable_id);
            canonical_variables.insert(variable.variable_id);
        }
        Self {
            term,
            canonical_variables,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct BinderContext {
    frames: Vec<BinderFrame>,
    free_variables: Vec<VariableId>,
    schematic_variables: Vec<VariableId>,
}

impl BinderContext {
    fn frame_for_binder(&self, binder_id: u32) -> Option<&BinderFrame> {
        self.frames
            .binary_search_by_key(&binder_id, |frame| frame.binder_id)
            .ok()
            .map(|index| &self.frames[index])
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct BinderFrame {
    binder_id: u32,
    canonical_index: u32,
    variable_id: VariableId,
    binder_role: u8,
}

fn decode_binder_context(
    input: SubstitutionCheckInput<'_>,
    replay_context: &ReplayContext,
    entry: &SubstitutionEntry,
) -> SubstitutionCheckResult<BinderContext> {
    let bytes = &entry.binder_context_encoding;
    if bytes.len() > input.limits.max_binder_context_bytes {
        return Err(resource_rejection(
            input.target_vc_fingerprint,
            substitution_location(entry).with_field_path("binder_context"),
        ));
    }
    let mut reader = BinderReader::new(bytes);
    let schema_version = reader
        .read_u16()
        .map_err(|()| malformed_binder(input, entry, "binder_context.schema_version"))?;
    if schema_version != BINDER_CONTEXT_SCHEMA_VERSION {
        return Err(malformed_binder(
            input,
            entry,
            "binder_context.schema_version",
        ));
    }
    let frame_count =
        checked_binder_count(input, entry, reader.read_u32(), "binder_context.frames")?;
    ensure_binder_items_available(
        input,
        entry,
        &reader,
        frame_count,
        BINDER_FRAME_ENCODED_BYTES,
        "binder_context.frames",
    )?;
    let mut frames = Vec::with_capacity(frame_count);
    for _ in 0..frame_count {
        let binder_id = reader
            .read_u32()
            .map_err(|()| malformed_binder(input, entry, "binder_context.binder_id"))?;
        let canonical_index = reader
            .read_u32()
            .map_err(|()| malformed_binder(input, entry, "binder_context.canonical_index"))?;
        let variable_id = VariableId(
            reader
                .read_u32()
                .map_err(|()| malformed_binder(input, entry, "binder_context.variable_id"))?,
        );
        let binder_role = reader
            .read_u8()
            .map_err(|()| malformed_binder(input, entry, "binder_context.binder_role"))?;
        if !matches!(binder_role, 1..=5) {
            return Err(malformed_binder(input, entry, "binder_context.binder_role"));
        }
        frames.push(BinderFrame {
            binder_id,
            canonical_index,
            variable_id,
            binder_role,
        });
    }
    let free_variable_count = checked_binder_count(
        input,
        entry,
        reader.read_u32(),
        "binder_context.free_variables",
    )?;
    ensure_binder_items_available(
        input,
        entry,
        &reader,
        free_variable_count,
        BINDER_VARIABLE_ENCODED_BYTES,
        "binder_context.free_variables",
    )?;
    let mut free_variables = Vec::with_capacity(free_variable_count);
    for _ in 0..free_variable_count {
        free_variables.push(VariableId(reader.read_u32().map_err(|()| {
            malformed_binder(input, entry, "binder_context.free_variables")
        })?));
    }
    let schematic_variable_count = checked_binder_count(
        input,
        entry,
        reader.read_u32(),
        "binder_context.schematic_variables",
    )?;
    ensure_binder_items_available(
        input,
        entry,
        &reader,
        schematic_variable_count,
        BINDER_VARIABLE_ENCODED_BYTES,
        "binder_context.schematic_variables",
    )?;
    let mut schematic_variables = Vec::with_capacity(schematic_variable_count);
    for _ in 0..schematic_variable_count {
        schematic_variables.push(VariableId(reader.read_u32().map_err(|()| {
            malformed_binder(input, entry, "binder_context.schematic_variables")
        })?));
    }
    if !reader.is_finished() {
        return Err(malformed_binder(
            input,
            entry,
            "binder_context.trailing_bytes",
        ));
    }
    validate_binder_frames(input, replay_context, entry, &mut frames)?;
    validate_sorted_unique_variables(
        input,
        replay_context,
        entry,
        &free_variables,
        "binder_context.free_variables",
    )?;
    validate_sorted_unique_variables(
        input,
        replay_context,
        entry,
        &schematic_variables,
        "binder_context.schematic_variables",
    )?;
    Ok(BinderContext {
        frames,
        free_variables,
        schematic_variables,
    })
}

fn checked_binder_count(
    input: SubstitutionCheckInput<'_>,
    entry: &SubstitutionEntry,
    count: Result<u32, ()>,
    field_path: &'static str,
) -> SubstitutionCheckResult<usize> {
    let count = count.map_err(|()| malformed_binder(input, entry, field_path))?;
    let count = usize::try_from(count).map_err(|_| {
        resource_rejection(
            input.target_vc_fingerprint,
            substitution_location(entry).with_field_path(field_path),
        )
    })?;
    if count > input.limits.max_binder_frames {
        return Err(resource_rejection(
            input.target_vc_fingerprint,
            substitution_location(entry).with_field_path(field_path),
        ));
    }
    Ok(count)
}

fn ensure_binder_items_available(
    input: SubstitutionCheckInput<'_>,
    entry: &SubstitutionEntry,
    reader: &BinderReader<'_>,
    count: usize,
    item_width: usize,
    field_path: &'static str,
) -> SubstitutionCheckResult<()> {
    let needed = count.checked_mul(item_width).ok_or_else(|| {
        resource_rejection(
            input.target_vc_fingerprint,
            substitution_location(entry).with_field_path(field_path),
        )
    })?;
    if needed > reader.remaining() {
        return Err(malformed_binder(input, entry, field_path));
    }
    Ok(())
}

fn validate_binder_frames(
    input: SubstitutionCheckInput<'_>,
    replay_context: &ReplayContext,
    entry: &SubstitutionEntry,
    frames: &mut [BinderFrame],
) -> SubstitutionCheckResult<()> {
    let mut binder_ids = BTreeSet::new();
    let mut variable_ids = BTreeSet::new();
    for frame in frames.iter() {
        if !binder_ids.insert(frame.binder_id) || !variable_ids.insert(frame.variable_id) {
            return Err(malformed_binder(input, entry, "binder_context.frames"));
        }
        if !replay_context
            .canonical_variables
            .contains(&frame.variable_id)
        {
            return Err(malformed_binder(input, entry, "binder_context.frames"));
        }
    }
    for (expected, frame) in frames.iter().enumerate() {
        let expected = u32::try_from(expected)
            .map_err(|_| malformed_binder(input, entry, "binder_context.frames"))?;
        if frame.canonical_index != expected {
            return Err(malformed_binder(input, entry, "binder_context.frames"));
        }
    }
    if !frames
        .windows(2)
        .all(|window| window[0].canonical_index < window[1].canonical_index)
    {
        return Err(malformed_binder(input, entry, "binder_context.frames"));
    }
    frames.sort_by_key(|frame| frame.binder_id);
    Ok(())
}

fn validate_sorted_unique_variables(
    input: SubstitutionCheckInput<'_>,
    replay_context: &ReplayContext,
    entry: &SubstitutionEntry,
    variables: &[VariableId],
    field_path: &'static str,
) -> SubstitutionCheckResult<()> {
    if !variables.windows(2).all(|window| window[0] < window[1]) {
        return Err(invalid_rejection(
            input.target_vc_fingerprint,
            substitution_location(entry).with_field_path(field_path),
        ));
    }
    for variable in variables {
        if !replay_context.canonical_variables.contains(variable) {
            return Err(invalid_rejection(
                input.target_vc_fingerprint,
                substitution_location(entry).with_field_path(field_path),
            ));
        }
    }
    Ok(())
}

fn substitution_context_for<'a>(
    input: SubstitutionCheckInput<'a>,
    entry: &SubstitutionEntry,
) -> SubstitutionCheckResult<&'a SubstitutionContext> {
    let Some(context) = input.substitution_context else {
        return Err(missing_rejection(
            input.target_vc_fingerprint,
            substitution_location(entry).with_field_path("substitution_context"),
        ));
    };
    if context
        .provenance_fingerprint()
        .is_none_or(|fingerprint| fingerprint.is_empty())
    {
        return Err(missing_rejection(
            input.target_vc_fingerprint,
            substitution_location(entry).with_field_path("substitution_context.provenance"),
        ));
    }
    Ok(context)
}

fn validate_payload<'a>(
    input: SubstitutionCheckInput<'_>,
    replay_context: &ReplayContext,
    binder_context: &BinderContext,
    context: &'a SubstitutionContext,
    entry: &SubstitutionEntry,
) -> SubstitutionCheckResult<&'a SubstitutionPayload> {
    let payload_entry = context.payload_for(entry.substitution_id).map_err(|()| {
        missing_rejection(
            input.target_vc_fingerprint,
            substitution_location(entry).with_field_path("substitution_payload"),
        )
    })?;
    let payload = &payload_entry.payload;
    if payload.owner_substitution_id != entry.substitution_id {
        return Err(invalid_rejection(
            input.target_vc_fingerprint,
            substitution_location(entry).with_field_path("payload.owner_substitution_id"),
        ));
    }
    match payload.payload_kind {
        PAYLOAD_KIND_FORMAL_TO_ACTUAL_MAP => {}
        PAYLOAD_KIND_LOCAL_ABBREVIATION_EXPANSION => {
            return Err(invalid_rejection(
                input.target_vc_fingerprint,
                substitution_location(entry).with_field_path("payload.payload_kind"),
            ));
        }
        _ => {
            return Err(invalid_rejection(
                input.target_vc_fingerprint,
                substitution_location(entry).with_field_path("payload.payload_kind"),
            ));
        }
    }
    if payload.replacements.len() > input.limits.max_payload_replacements {
        return Err(resource_rejection(
            input.target_vc_fingerprint,
            substitution_location(entry).with_field_path("payload.replacements"),
        ));
    }
    validate_term_path(
        input,
        entry,
        &entry.source_term,
        &payload.rewrite_path,
        "payload.rewrite_path",
    )?;
    let mut seen = BTreeSet::new();
    for replacement in &payload.replacements {
        if !seen.insert(replacement.formal_variable_id) {
            return Err(invalid_rejection(
                input.target_vc_fingerprint,
                substitution_location(entry).with_field_path("payload.replacements"),
            ));
        }
        if !replay_context
            .canonical_variables
            .contains(&replacement.formal_variable_id)
        {
            return Err(invalid_rejection(
                input.target_vc_fingerprint,
                substitution_location(entry).with_field_path("payload.formal_variable_id"),
            ));
        }
        match replacement.replacement_role {
            REPLACEMENT_ROLE_TERM_ARGUMENT | REPLACEMENT_ROLE_PREDICATE_ARGUMENT => {}
            REPLACEMENT_ROLE_CAPTURED_FREE_VARIABLE => {
                return Err(invalid_rejection(
                    input.target_vc_fingerprint,
                    substitution_location(entry).with_field_path("payload.replacement_role"),
                ));
            }
            _ => {
                return Err(invalid_rejection(
                    input.target_vc_fingerprint,
                    substitution_location(entry).with_field_path("payload.replacement_role"),
                ));
            }
        }
        validate_term_for_entry(
            input,
            replay_context,
            binder_context,
            entry,
            &replacement.actual_term,
            "payload.actual_term",
        )?;
    }
    if !payload
        .replacements
        .windows(2)
        .all(|window| window[0].formal_variable_id < window[1].formal_variable_id)
    {
        return Err(invalid_rejection(
            input.target_vc_fingerprint,
            substitution_location(entry).with_field_path("payload.replacements"),
        ));
    }
    Ok(payload)
}

fn validate_side_conditions(
    input: SubstitutionCheckInput<'_>,
    replay_context: &ReplayContext,
    _binder_context: &BinderContext,
    context: &SubstitutionContext,
    entry: &SubstitutionEntry,
) -> SubstitutionCheckResult<()> {
    if entry.freshness_witness_refs.len() > input.limits.max_freshness_witnesses {
        return Err(resource_rejection(
            input.target_vc_fingerprint,
            substitution_location(entry).with_field_path("freshness_witness_refs"),
        ));
    }
    if entry.free_variable_constraint_refs.len() > input.limits.max_free_variable_constraints {
        return Err(resource_rejection(
            input.target_vc_fingerprint,
            substitution_location(entry).with_field_path("free_variable_constraint_refs"),
        ));
    }
    for witness_id in &entry.freshness_witness_refs {
        let witness = context.freshness_witness(*witness_id).map_err(|()| {
            missing_rejection(
                input.target_vc_fingerprint,
                substitution_location(entry).with_field_path("freshness_witness_refs"),
            )
        })?;
        validate_freshness_witness(input, replay_context, entry, witness)?;
    }
    for constraint_id in &entry.free_variable_constraint_refs {
        let constraint = context
            .free_variable_constraint(*constraint_id)
            .map_err(|()| {
                missing_rejection(
                    input.target_vc_fingerprint,
                    substitution_location(entry).with_field_path("free_variable_constraint_refs"),
                )
            })?;
        validate_free_variable_constraint(input, replay_context, entry, constraint)?;
    }
    Ok(())
}

fn validate_binder_context_usage(
    input: SubstitutionCheckInput<'_>,
    binder_context: &BinderContext,
    entry: &SubstitutionEntry,
    payload: &SubstitutionPayload,
) -> SubstitutionCheckResult<()> {
    let mut used_binders = BTreeSet::new();
    collect_binder_ids(&entry.source_term, &mut used_binders);
    collect_binder_ids(&entry.target_term, &mut used_binders);
    for replacement in &payload.replacements {
        collect_binder_ids(&replacement.actual_term, &mut used_binders);
    }
    let frame_binders = binder_context
        .frames
        .iter()
        .map(|frame| frame.binder_id)
        .collect::<BTreeSet<_>>();
    if used_binders != frame_binders {
        return Err(invalid_rejection(
            input.target_vc_fingerprint,
            substitution_location(entry).with_field_path("binder_context"),
        ));
    }
    Ok(())
}

fn collect_binder_ids(term: &Term, used_binders: &mut BTreeSet<u32>) {
    match term {
        Term::Variable(_) | Term::Malformed => {}
        Term::Application { arguments, .. } => {
            for argument in arguments {
                collect_binder_ids(argument, used_binders);
            }
        }
        Term::BinderNormalized { binder_id, body } => {
            used_binders.insert(*binder_id);
            collect_binder_ids(body, used_binders);
        }
    }
}

fn validate_freshness_witness(
    input: SubstitutionCheckInput<'_>,
    replay_context: &ReplayContext,
    entry: &SubstitutionEntry,
    witness: &FreshnessWitness,
) -> SubstitutionCheckResult<()> {
    if witness.owner_substitution_id != entry.substitution_id {
        return Err(invalid_rejection(
            input.target_vc_fingerprint,
            substitution_location(entry).with_field_path("freshness_witness.owner_substitution_id"),
        ));
    }
    if witness.avoided_variables.len() > input.limits.max_avoided_variables {
        return Err(resource_rejection(
            input.target_vc_fingerprint,
            substitution_location(entry).with_field_path("freshness_witness.avoided_variables"),
        ));
    }
    validate_manifest_variable(
        input,
        replay_context,
        entry,
        witness.generated_variable_id,
        "freshness_witness.generated_variable_id",
    )?;
    validate_variable_list(
        input,
        replay_context,
        entry,
        &witness.avoided_variables,
        "freshness_witness.avoided_variables",
    )?;
    validate_term_path(
        input,
        entry,
        &entry.source_term,
        &witness.binder_path,
        "freshness_witness.binder_path",
    )?;
    Ok(())
}

fn validate_free_variable_constraint(
    input: SubstitutionCheckInput<'_>,
    replay_context: &ReplayContext,
    entry: &SubstitutionEntry,
    constraint: &FreeVariableConstraint,
) -> SubstitutionCheckResult<()> {
    if constraint.owner_substitution_id != entry.substitution_id {
        return Err(invalid_rejection(
            input.target_vc_fingerprint,
            substitution_location(entry)
                .with_field_path("free_variable_constraint.owner_substitution_id"),
        ));
    }
    if !matches!(
        constraint.constraint_kind,
        CONSTRAINT_KIND_MUST_REMAIN_FREE_AT_PATH | CONSTRAINT_KIND_MUST_BE_ABSENT_FROM_CAPTURE_SET
    ) {
        return Err(invalid_rejection(
            input.target_vc_fingerprint,
            substitution_location(entry)
                .with_field_path("free_variable_constraint.constraint_kind"),
        ));
    }
    validate_manifest_variable(
        input,
        replay_context,
        entry,
        constraint.variable_id,
        "free_variable_constraint.variable_id",
    )?;
    if constraint.capture_set.len() > input.limits.max_capture_set_variables {
        return Err(resource_rejection(
            input.target_vc_fingerprint,
            substitution_location(entry).with_field_path("free_variable_constraint.capture_set"),
        ));
    }
    validate_variable_list(
        input,
        replay_context,
        entry,
        &constraint.capture_set,
        "free_variable_constraint.capture_set",
    )?;
    validate_term_path(
        input,
        entry,
        &entry.target_term,
        &constraint.term_path,
        "free_variable_constraint.term_path",
    )?;
    Ok(())
}

fn replay_direct_substitution(
    input: SubstitutionCheckInput<'_>,
    binder_context: &BinderContext,
    entry: &SubstitutionEntry,
    payload: &SubstitutionPayload,
) -> SubstitutionCheckResult<Term> {
    apply_at_path(
        input,
        binder_context,
        entry,
        &entry.source_term,
        &payload.rewrite_path.segments,
        payload,
        &mut Vec::new(),
    )
}

fn measure_direct_substitution(
    input: SubstitutionCheckInput<'_>,
    binder_context: &BinderContext,
    entry: &SubstitutionEntry,
    payload: &SubstitutionPayload,
) -> SubstitutionCheckResult<usize> {
    let walk = MeasureReplay {
        input,
        binder_context,
        entry,
        payload,
    };
    measure_apply_at_path(
        walk,
        &entry.source_term,
        &payload.rewrite_path.segments,
        &mut Vec::new(),
        0,
    )
}

#[derive(Clone, Copy)]
struct MeasureReplay<'input, 'walk> {
    input: SubstitutionCheckInput<'input>,
    binder_context: &'walk BinderContext,
    entry: &'walk SubstitutionEntry,
    payload: &'walk SubstitutionPayload,
}

fn measure_apply_at_path(
    walk: MeasureReplay<'_, '_>,
    term: &Term,
    path: &[TermPathSegment],
    active_bound_variables: &mut Vec<VariableId>,
    depth: usize,
) -> SubstitutionCheckResult<usize> {
    validate_replay_depth(walk.input, walk.entry, depth)?;
    if path.is_empty() {
        return measure_substitute_subtree(walk, term, active_bound_variables, depth);
    }
    let (segment, rest) = path
        .split_first()
        .expect("path is known non-empty after check");
    match (segment.edge_kind, term) {
        (EDGE_KIND_APPLICATION_ARGUMENT, Term::Application { symbol, arguments }) => {
            let index = usize::try_from(segment.child_index).map_err(|_| {
                invalid_rejection(
                    walk.input.target_vc_fingerprint,
                    substitution_location(walk.entry).with_field_path("payload.rewrite_path"),
                )
            })?;
            if index >= arguments.len() {
                return Err(invalid_rejection(
                    walk.input.target_vc_fingerprint,
                    substitution_location(walk.entry).with_field_path("payload.rewrite_path"),
                ));
            }
            let child_depth = checked_replay_child_depth(walk.input, walk.entry, depth)?;
            let mut argument_lens = Vec::with_capacity(arguments.len());
            for (argument_index, argument) in arguments.iter().enumerate() {
                let argument_len = if argument_index == index {
                    measure_apply_at_path(
                        walk,
                        argument,
                        rest,
                        active_bound_variables,
                        child_depth,
                    )?
                } else {
                    measure_existing_term_at_depth(walk.input, walk.entry, argument, child_depth)?
                };
                argument_lens.push(argument_len);
            }
            checked_replay_len(
                walk.input,
                walk.entry,
                application_term_len_for_kernel(*symbol, &argument_lens),
            )
        }
        (EDGE_KIND_BINDER_BODY, Term::BinderNormalized { binder_id, body })
            if segment.child_index == 0 =>
        {
            let frame = walk
                .binder_context
                .frame_for_binder(*binder_id)
                .ok_or_else(|| {
                    invalid_rejection(
                        walk.input.target_vc_fingerprint,
                        substitution_location(walk.entry).with_field_path("binder_context"),
                    )
                })?;
            active_bound_variables.push(frame.variable_id);
            let child_depth = checked_replay_child_depth(walk.input, walk.entry, depth)?;
            let body_len =
                measure_apply_at_path(walk, body, rest, active_bound_variables, child_depth);
            active_bound_variables.pop();
            checked_replay_len(
                walk.input,
                walk.entry,
                binder_term_len_for_kernel(body_len?),
            )
        }
        _ => Err(invalid_rejection(
            walk.input.target_vc_fingerprint,
            substitution_location(walk.entry).with_field_path("payload.rewrite_path"),
        )),
    }
}

fn measure_substitute_subtree(
    walk: MeasureReplay<'_, '_>,
    term: &Term,
    active_bound_variables: &mut Vec<VariableId>,
    depth: usize,
) -> SubstitutionCheckResult<usize> {
    validate_replay_depth(walk.input, walk.entry, depth)?;
    match term {
        Term::Variable(variable) => {
            if active_bound_variables.contains(variable) {
                return measure_existing_term_at_depth(walk.input, walk.entry, term, depth);
            }
            let Some(replacement) = walk
                .payload
                .replacements
                .iter()
                .find(|replacement| replacement.formal_variable_id == *variable)
            else {
                return measure_existing_term_at_depth(walk.input, walk.entry, term, depth);
            };
            if actual_term_contains_any_active_bound_variable(
                walk.input,
                walk.binder_context,
                walk.entry,
                &replacement.actual_term,
                active_bound_variables,
            )? {
                return Err(invalid_rejection(
                    walk.input.target_vc_fingerprint,
                    substitution_location(walk.entry).with_field_path("payload.actual_term"),
                ));
            }
            if !active_bound_variables.is_empty() {
                return Err(invalid_rejection(
                    walk.input.target_vc_fingerprint,
                    substitution_location(walk.entry).with_field_path("payload.rewrite_path"),
                ));
            }
            measure_existing_term_at_depth(walk.input, walk.entry, &replacement.actual_term, depth)
        }
        Term::Application { symbol, arguments } => {
            let child_depth = checked_replay_child_depth(walk.input, walk.entry, depth)?;
            let mut argument_lens = Vec::with_capacity(arguments.len());
            for argument in arguments {
                argument_lens.push(measure_substitute_subtree(
                    walk,
                    argument,
                    active_bound_variables,
                    child_depth,
                )?);
            }
            checked_replay_len(
                walk.input,
                walk.entry,
                application_term_len_for_kernel(*symbol, &argument_lens),
            )
        }
        Term::BinderNormalized { binder_id, body } => {
            let frame = walk
                .binder_context
                .frame_for_binder(*binder_id)
                .ok_or_else(|| {
                    invalid_rejection(
                        walk.input.target_vc_fingerprint,
                        substitution_location(walk.entry).with_field_path("binder_context"),
                    )
                })?;
            active_bound_variables.push(frame.variable_id);
            let child_depth = checked_replay_child_depth(walk.input, walk.entry, depth)?;
            let body_len =
                measure_substitute_subtree(walk, body, active_bound_variables, child_depth);
            active_bound_variables.pop();
            checked_replay_len(
                walk.input,
                walk.entry,
                binder_term_len_for_kernel(body_len?),
            )
        }
        Term::Malformed => Err(invalid_rejection(
            walk.input.target_vc_fingerprint,
            substitution_location(walk.entry).with_field_path("source_term"),
        )),
    }
}

fn measure_existing_term_at_depth(
    input: SubstitutionCheckInput<'_>,
    entry: &SubstitutionEntry,
    term: &Term,
    depth: usize,
) -> SubstitutionCheckResult<usize> {
    validate_replay_depth(input, entry, depth)?;
    match term {
        Term::Variable(_) | Term::Malformed => {
            checked_replay_len(input, entry, term.canonical_len_for_kernel())
        }
        Term::Application { symbol, arguments } => {
            let child_depth = checked_replay_child_depth(input, entry, depth)?;
            let mut argument_lens = Vec::with_capacity(arguments.len());
            for argument in arguments {
                argument_lens.push(measure_existing_term_at_depth(
                    input,
                    entry,
                    argument,
                    child_depth,
                )?);
            }
            checked_replay_len(
                input,
                entry,
                application_term_len_for_kernel(*symbol, &argument_lens),
            )
        }
        Term::BinderNormalized { body, .. } => {
            let child_depth = checked_replay_child_depth(input, entry, depth)?;
            let body_len = measure_existing_term_at_depth(input, entry, body, child_depth)?;
            checked_replay_len(input, entry, binder_term_len_for_kernel(body_len))
        }
    }
}

fn validate_replay_depth(
    input: SubstitutionCheckInput<'_>,
    entry: &SubstitutionEntry,
    depth: usize,
) -> SubstitutionCheckResult<()> {
    if depth > input.limits.max_term_recursion_depth {
        return Err(resource_rejection(
            input.target_vc_fingerprint,
            substitution_location(entry).with_field_path("target_term"),
        ));
    }
    Ok(())
}

fn checked_replay_child_depth(
    input: SubstitutionCheckInput<'_>,
    entry: &SubstitutionEntry,
    depth: usize,
) -> SubstitutionCheckResult<usize> {
    depth.checked_add(1).ok_or_else(|| {
        resource_rejection(
            input.target_vc_fingerprint,
            substitution_location(entry).with_field_path("target_term"),
        )
    })
}

fn checked_replay_len(
    input: SubstitutionCheckInput<'_>,
    entry: &SubstitutionEntry,
    canonical_len: Result<usize, ClauseError>,
) -> SubstitutionCheckResult<usize> {
    let canonical_len = canonical_len
        .map_err(|error| clause_error_rejection(input, entry, error, "target_term"))?;
    if canonical_len > input.limits.max_term_encoding_bytes {
        return Err(resource_rejection(
            input.target_vc_fingerprint,
            substitution_location(entry).with_field_path("target_term"),
        ));
    }
    Ok(canonical_len)
}

fn apply_at_path(
    input: SubstitutionCheckInput<'_>,
    binder_context: &BinderContext,
    entry: &SubstitutionEntry,
    term: &Term,
    path: &[TermPathSegment],
    payload: &SubstitutionPayload,
    active_bound_variables: &mut Vec<VariableId>,
) -> SubstitutionCheckResult<Term> {
    if path.is_empty() {
        return substitute_subtree(
            input,
            binder_context,
            entry,
            term,
            payload,
            active_bound_variables,
        );
    }
    let (segment, rest) = path
        .split_first()
        .expect("path is known non-empty after check");
    match (segment.edge_kind, term) {
        (EDGE_KIND_APPLICATION_ARGUMENT, Term::Application { symbol, arguments }) => {
            let index = usize::try_from(segment.child_index).map_err(|_| {
                invalid_rejection(
                    input.target_vc_fingerprint,
                    substitution_location(entry).with_field_path("payload.rewrite_path"),
                )
            })?;
            let Some(argument) = arguments.get(index) else {
                return Err(invalid_rejection(
                    input.target_vc_fingerprint,
                    substitution_location(entry).with_field_path("payload.rewrite_path"),
                ));
            };
            let mut replayed_arguments = arguments.clone();
            replayed_arguments[index] = apply_at_path(
                input,
                binder_context,
                entry,
                argument,
                rest,
                payload,
                active_bound_variables,
            )?;
            Ok(Term::Application {
                symbol: *symbol,
                arguments: replayed_arguments,
            })
        }
        (EDGE_KIND_BINDER_BODY, Term::BinderNormalized { binder_id, body })
            if segment.child_index == 0 =>
        {
            let frame = binder_context.frame_for_binder(*binder_id).ok_or_else(|| {
                invalid_rejection(
                    input.target_vc_fingerprint,
                    substitution_location(entry).with_field_path("binder_context"),
                )
            })?;
            active_bound_variables.push(frame.variable_id);
            let replayed = apply_at_path(
                input,
                binder_context,
                entry,
                body,
                rest,
                payload,
                active_bound_variables,
            );
            active_bound_variables.pop();
            Ok(Term::BinderNormalized {
                binder_id: *binder_id,
                body: Box::new(replayed?),
            })
        }
        _ => Err(invalid_rejection(
            input.target_vc_fingerprint,
            substitution_location(entry).with_field_path("payload.rewrite_path"),
        )),
    }
}

fn substitute_subtree(
    input: SubstitutionCheckInput<'_>,
    binder_context: &BinderContext,
    entry: &SubstitutionEntry,
    term: &Term,
    payload: &SubstitutionPayload,
    active_bound_variables: &mut Vec<VariableId>,
) -> SubstitutionCheckResult<Term> {
    match term {
        Term::Variable(variable) => {
            if active_bound_variables.contains(variable) {
                return Ok(term.clone());
            }
            let Some(replacement) = payload
                .replacements
                .iter()
                .find(|replacement| replacement.formal_variable_id == *variable)
            else {
                return Ok(term.clone());
            };
            if actual_term_contains_any_active_bound_variable(
                input,
                binder_context,
                entry,
                &replacement.actual_term,
                active_bound_variables,
            )? {
                return Err(invalid_rejection(
                    input.target_vc_fingerprint,
                    substitution_location(entry).with_field_path("payload.actual_term"),
                ));
            }
            if !active_bound_variables.is_empty() {
                return Err(invalid_rejection(
                    input.target_vc_fingerprint,
                    substitution_location(entry).with_field_path("payload.rewrite_path"),
                ));
            }
            Ok(replacement.actual_term.clone())
        }
        Term::Application { symbol, arguments } => {
            let mut replayed_arguments = Vec::with_capacity(arguments.len());
            for argument in arguments {
                replayed_arguments.push(substitute_subtree(
                    input,
                    binder_context,
                    entry,
                    argument,
                    payload,
                    active_bound_variables,
                )?);
            }
            Ok(Term::Application {
                symbol: *symbol,
                arguments: replayed_arguments,
            })
        }
        Term::BinderNormalized { binder_id, body } => {
            let frame = binder_context.frame_for_binder(*binder_id).ok_or_else(|| {
                invalid_rejection(
                    input.target_vc_fingerprint,
                    substitution_location(entry).with_field_path("binder_context"),
                )
            })?;
            active_bound_variables.push(frame.variable_id);
            let replayed = substitute_subtree(
                input,
                binder_context,
                entry,
                body,
                payload,
                active_bound_variables,
            );
            active_bound_variables.pop();
            Ok(Term::BinderNormalized {
                binder_id: *binder_id,
                body: Box::new(replayed?),
            })
        }
        Term::Malformed => Err(invalid_rejection(
            input.target_vc_fingerprint,
            substitution_location(entry).with_field_path("source_term"),
        )),
    }
}

fn validate_term_for_entry(
    input: SubstitutionCheckInput<'_>,
    replay_context: &ReplayContext,
    binder_context: &BinderContext,
    entry: &SubstitutionEntry,
    term: &Term,
    field_path: &'static str,
) -> SubstitutionCheckResult<()> {
    term.validate_for_kernel(&replay_context.term)
        .map_err(|error| clause_error_rejection(input, entry, error, field_path))?;
    let mut seen_binders = BTreeSet::new();
    let mut previous_canonical_index = None;
    validate_term_binders(
        input,
        entry,
        binder_context,
        term,
        field_path,
        &mut seen_binders,
        &mut previous_canonical_index,
    )
}

fn validate_term_binders(
    input: SubstitutionCheckInput<'_>,
    entry: &SubstitutionEntry,
    binder_context: &BinderContext,
    term: &Term,
    field_path: &'static str,
    seen_binders: &mut BTreeSet<u32>,
    previous_canonical_index: &mut Option<u32>,
) -> SubstitutionCheckResult<()> {
    match term {
        Term::Variable(_) => Ok(()),
        Term::Application { arguments, .. } => {
            for argument in arguments {
                validate_term_binders(
                    input,
                    entry,
                    binder_context,
                    argument,
                    field_path,
                    seen_binders,
                    previous_canonical_index,
                )?;
            }
            Ok(())
        }
        Term::BinderNormalized { binder_id, body } => {
            let Some(frame) = binder_context.frame_for_binder(*binder_id) else {
                return Err(invalid_rejection(
                    input.target_vc_fingerprint,
                    substitution_location(entry).with_field_path(field_path),
                ));
            };
            if !seen_binders.insert(*binder_id) {
                return Err(invalid_rejection(
                    input.target_vc_fingerprint,
                    substitution_location(entry).with_field_path(field_path),
                ));
            }
            if previous_canonical_index.is_some_and(|previous| previous >= frame.canonical_index) {
                return Err(invalid_rejection(
                    input.target_vc_fingerprint,
                    substitution_location(entry).with_field_path(field_path),
                ));
            }
            *previous_canonical_index = Some(frame.canonical_index);
            validate_term_binders(
                input,
                entry,
                binder_context,
                body,
                field_path,
                seen_binders,
                previous_canonical_index,
            )
        }
        Term::Malformed => Err(invalid_rejection(
            input.target_vc_fingerprint,
            substitution_location(entry).with_field_path(field_path),
        )),
    }
}

fn validate_term_path(
    input: SubstitutionCheckInput<'_>,
    entry: &SubstitutionEntry,
    root: &Term,
    path: &TermPath,
    field_path: &'static str,
) -> SubstitutionCheckResult<()> {
    if path.segments.len() > input.limits.max_term_path_segments {
        return Err(resource_rejection(
            input.target_vc_fingerprint,
            substitution_location(entry).with_field_path(field_path),
        ));
    }
    let mut term = root;
    for segment in &path.segments {
        term = match (segment.edge_kind, term) {
            (EDGE_KIND_APPLICATION_ARGUMENT, Term::Application { arguments, .. }) => {
                let index = usize::try_from(segment.child_index).map_err(|_| {
                    invalid_rejection(
                        input.target_vc_fingerprint,
                        substitution_location(entry).with_field_path(field_path),
                    )
                })?;
                arguments.get(index).ok_or_else(|| {
                    invalid_rejection(
                        input.target_vc_fingerprint,
                        substitution_location(entry).with_field_path(field_path),
                    )
                })?
            }
            (EDGE_KIND_BINDER_BODY, Term::BinderNormalized { body, .. })
                if segment.child_index == 0 =>
            {
                body
            }
            (EDGE_KIND_BINDER_BODY | EDGE_KIND_APPLICATION_ARGUMENT, _) => {
                return Err(invalid_rejection(
                    input.target_vc_fingerprint,
                    substitution_location(entry).with_field_path(field_path),
                ));
            }
            _ => {
                return Err(invalid_rejection(
                    input.target_vc_fingerprint,
                    substitution_location(entry).with_field_path(field_path),
                ));
            }
        };
    }
    Ok(())
}

fn validate_manifest_variable(
    input: SubstitutionCheckInput<'_>,
    replay_context: &ReplayContext,
    entry: &SubstitutionEntry,
    variable: VariableId,
    field_path: &'static str,
) -> SubstitutionCheckResult<()> {
    if replay_context.canonical_variables.contains(&variable) {
        Ok(())
    } else {
        Err(invalid_rejection(
            input.target_vc_fingerprint,
            substitution_location(entry).with_field_path(field_path),
        ))
    }
}

fn validate_variable_list(
    input: SubstitutionCheckInput<'_>,
    replay_context: &ReplayContext,
    entry: &SubstitutionEntry,
    variables: &[VariableId],
    field_path: &'static str,
) -> SubstitutionCheckResult<()> {
    if !variables.windows(2).all(|window| window[0] < window[1]) {
        return Err(invalid_rejection(
            input.target_vc_fingerprint,
            substitution_location(entry).with_field_path(field_path),
        ));
    }
    for variable in variables {
        validate_manifest_variable(input, replay_context, entry, *variable, field_path)?;
    }
    Ok(())
}

fn actual_term_contains_any_active_bound_variable(
    input: SubstitutionCheckInput<'_>,
    binder_context: &BinderContext,
    entry: &SubstitutionEntry,
    actual_term: &Term,
    active_bound_variables: &[VariableId],
) -> SubstitutionCheckResult<bool> {
    for bound in active_bound_variables {
        if term_contains_free_variable(input, binder_context, entry, actual_term, *bound)? {
            return Ok(true);
        }
    }
    Ok(false)
}

fn term_contains_free_variable(
    input: SubstitutionCheckInput<'_>,
    binder_context: &BinderContext,
    entry: &SubstitutionEntry,
    term: &Term,
    needle: VariableId,
) -> SubstitutionCheckResult<bool> {
    let mut locally_bound = Vec::new();
    term_contains_free_variable_with_bound(
        input,
        binder_context,
        entry,
        term,
        needle,
        &mut locally_bound,
    )
}

fn term_contains_free_variable_with_bound(
    input: SubstitutionCheckInput<'_>,
    binder_context: &BinderContext,
    entry: &SubstitutionEntry,
    term: &Term,
    needle: VariableId,
    locally_bound: &mut Vec<VariableId>,
) -> SubstitutionCheckResult<bool> {
    match term {
        Term::Variable(variable) => Ok(*variable == needle && !locally_bound.contains(variable)),
        Term::Application { arguments, .. } => {
            for argument in arguments {
                if term_contains_free_variable_with_bound(
                    input,
                    binder_context,
                    entry,
                    argument,
                    needle,
                    locally_bound,
                )? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        Term::BinderNormalized { binder_id, body } => {
            let frame = binder_context.frame_for_binder(*binder_id).ok_or_else(|| {
                invalid_rejection(
                    input.target_vc_fingerprint,
                    substitution_location(entry).with_field_path("binder_context"),
                )
            })?;
            locally_bound.push(frame.variable_id);
            let contains = term_contains_free_variable_with_bound(
                input,
                binder_context,
                entry,
                body,
                needle,
                locally_bound,
            );
            locally_bound.pop();
            contains
        }
        Term::Malformed => Ok(false),
    }
}

fn clause_error_rejection(
    input: SubstitutionCheckInput<'_>,
    entry: &SubstitutionEntry,
    error: ClauseError,
    field_path: &'static str,
) -> Box<RejectionRecord> {
    let detail = if is_resource_clause_error(&error) {
        RejectionDetail::ResourceExhaustion
    } else {
        RejectionDetail::InvalidSubstitution
    };
    rejection(
        input.target_vc_fingerprint,
        detail,
        substitution_location(entry).with_field_path(field_path),
    )
}

fn is_resource_clause_error(error: &ClauseError) -> bool {
    matches!(
        error,
        ClauseError::TermSizeExceeded { .. } | ClauseError::TermRecursionDepthExceeded { .. }
    )
}

fn input_context_provenance<'a>(input: SubstitutionCheckInput<'a>) -> Option<&'a [u8]> {
    input
        .substitution_context
        .and_then(SubstitutionContext::provenance_fingerprint)
}

fn malformed_binder(
    input: SubstitutionCheckInput<'_>,
    entry: &SubstitutionEntry,
    field_path: &'static str,
) -> Box<RejectionRecord> {
    invalid_rejection(
        input.target_vc_fingerprint,
        substitution_location(entry).with_field_path(field_path),
    )
}

fn missing_rejection(
    target: &TargetVcFingerprint,
    location: RejectionLocation,
) -> Box<RejectionRecord> {
    rejection(target, RejectionDetail::MissingProvenance, location)
}

fn invalid_rejection(
    target: &TargetVcFingerprint,
    location: RejectionLocation,
) -> Box<RejectionRecord> {
    rejection(target, RejectionDetail::InvalidSubstitution, location)
}

fn resource_rejection(
    target: &TargetVcFingerprint,
    location: RejectionLocation,
) -> Box<RejectionRecord> {
    rejection(target, RejectionDetail::ResourceExhaustion, location)
}

fn rejection(
    target: &TargetVcFingerprint,
    detail: RejectionDetail,
    location: RejectionLocation,
) -> Box<RejectionRecord> {
    Box::new(
        RejectionRecord::new(
            target.clone(),
            RejectionCategory::KernelRejection,
            detail,
            location,
        )
        .expect("substitution checker uses valid kernel rejection detail mappings"),
    )
}

fn substitution_location(entry: &SubstitutionEntry) -> RejectionLocation {
    RejectionLocation::new().with_substitution_id(entry.substitution_id)
}

fn canonical_payloads(
    mut payloads: Vec<SubstitutionPayloadEntry>,
) -> Result<Vec<SubstitutionPayloadEntry>, SubstitutionContextError> {
    payloads.sort_by_key(|entry| entry.substitution_id);
    for window in payloads.windows(2) {
        if window[0].substitution_id == window[1].substitution_id {
            return Err(SubstitutionContextError::DuplicateSubstitutionPayload {
                substitution_id: window[0].substitution_id,
            });
        }
    }
    Ok(payloads)
}

fn canonical_freshness_witnesses(
    mut witnesses: Vec<FreshnessWitness>,
) -> Result<Vec<FreshnessWitness>, SubstitutionContextError> {
    witnesses.sort_by_key(|witness| witness.witness_id);
    for window in witnesses.windows(2) {
        if window[0].witness_id == window[1].witness_id {
            return Err(SubstitutionContextError::DuplicateFreshnessWitness {
                witness_id: window[0].witness_id,
            });
        }
    }
    Ok(witnesses)
}

fn canonical_free_variable_constraints(
    mut constraints: Vec<FreeVariableConstraint>,
) -> Result<Vec<FreeVariableConstraint>, SubstitutionContextError> {
    constraints.sort_by_key(|constraint| constraint.constraint_id);
    for window in constraints.windows(2) {
        if window[0].constraint_id == window[1].constraint_id {
            return Err(SubstitutionContextError::DuplicateFreeVariableConstraint {
                constraint_id: window[0].constraint_id,
            });
        }
    }
    Ok(constraints)
}

struct BinderReader<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> BinderReader<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    fn read_u8(&mut self) -> Result<u8, ()> {
        let Some(value) = self.bytes.get(self.position).copied() else {
            return Err(());
        };
        self.position += 1;
        Ok(value)
    }

    fn read_u16(&mut self) -> Result<u16, ()> {
        let bytes = self.read_array::<2>()?;
        Ok(u16::from_be_bytes(bytes))
    }

    fn read_u32(&mut self) -> Result<u32, ()> {
        let bytes = self.read_array::<4>()?;
        Ok(u32::from_be_bytes(bytes))
    }

    fn read_array<const N: usize>(&mut self) -> Result<[u8; N], ()> {
        let end = self.position.checked_add(N).ok_or(())?;
        let bytes = self.bytes.get(self.position..end).ok_or(())?;
        let array = bytes.try_into().map_err(|_| ())?;
        self.position = end;
        Ok(array)
    }

    const fn is_finished(&self) -> bool {
        self.position == self.bytes.len()
    }

    const fn remaining(&self) -> usize {
        self.bytes.len() - self.position
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        certificate_parser::{
            ClauseTautologyPolicy, FinalGoalNamespace, FinalGoalRef, Fingerprint,
            KernelProfileRecord, ParsedCertificateTestParts, SymbolManifestEntry,
            VariableManifestEntry,
        },
        clause::{SymbolKey, SymbolKind},
    };

    #[test]
    fn valid_direct_substitution_rewrites_only_recorded_path_and_reports_checked_data() {
        let target = target();
        let source = pair(var(1), var(1));
        let target_term = pair(var(2), var(1));
        let certificate = certificate(vec![substitution(1, source.clone(), target_term.clone())]);
        let context = context(vec![payload(
            1,
            path(vec![TermPathSegment::application_argument(0)]),
            vec![replacement(1, var(2), REPLACEMENT_ROLE_TERM_ARGUMENT)],
        )]);

        let report = replay_substitutions(input(&target, &certificate, Some(&context), limits()))
            .expect("valid direct substitution");

        assert_eq!(report.checked_substitutions().len(), 1);
        assert_eq!(report.checked_substitutions()[0].substitution_id, 1);
        assert_eq!(report.checked_substitutions()[0].source_term, source);
        assert_eq!(report.checked_substitutions()[0].target_term, target_term);
        assert_eq!(
            checked_substitutions_for_input(
                input(&target, &certificate, Some(&context), limits()),
                &report
            )
            .expect("report binding"),
            report.checked_substitutions()
        );
    }

    #[test]
    fn accepts_formal_map_payload_and_term_or_predicate_argument_roles_only() {
        for role in [
            REPLACEMENT_ROLE_TERM_ARGUMENT,
            REPLACEMENT_ROLE_PREDICATE_ARGUMENT,
        ] {
            let target = target();
            let certificate = certificate(vec![substitution(1, var(1), var(2))]);
            let context = context(vec![payload(
                1,
                TermPath::root(),
                vec![replacement(1, var(2), role)],
            )]);

            replay_substitutions(input(&target, &certificate, Some(&context), limits()))
                .expect("accepted direct payload role");
        }
    }

    #[test]
    fn rejects_missing_malformed_and_deferred_payload_evidence_without_diff_inference() {
        let target = target();
        let certificate = certificate(vec![substitution(1, var(1), var(2))]);
        let missing_payload = context(Vec::new());
        let record = replay_substitutions(input(
            &target,
            &certificate,
            Some(&missing_payload),
            limits(),
        ))
        .expect_err("missing payload");
        assert_rejection(
            &record,
            RejectionDetail::MissingProvenance,
            Some(1),
            Some("substitution_payload"),
        );
        assert_eq!(record.target_vc_fingerprint(), &target);

        let cases = [
            (
                payload(
                    1,
                    TermPath::root(),
                    vec![replacement(1, var(2), REPLACEMENT_ROLE_TERM_ARGUMENT)],
                )
                .with_kind(99),
                "payload.payload_kind",
            ),
            (
                payload(1, TermPath::root(), vec![replacement(1, var(2), 99)]),
                "payload.replacement_role",
            ),
            (
                payload(
                    1,
                    TermPath::root(),
                    vec![replacement(1, var(2), REPLACEMENT_ROLE_TERM_ARGUMENT)],
                )
                .with_kind(PAYLOAD_KIND_LOCAL_ABBREVIATION_EXPANSION),
                "payload.payload_kind",
            ),
            (
                payload(
                    1,
                    TermPath::root(),
                    vec![replacement(
                        1,
                        var(2),
                        REPLACEMENT_ROLE_CAPTURED_FREE_VARIABLE,
                    )],
                ),
                "payload.replacement_role",
            ),
            (
                payload(
                    1,
                    TermPath::root(),
                    vec![replacement(1, var(2), REPLACEMENT_ROLE_TERM_ARGUMENT)],
                )
                .with_owner(99),
                "payload.owner_substitution_id",
            ),
            (
                payload(
                    1,
                    TermPath::root(),
                    vec![
                        replacement(1, var(2), REPLACEMENT_ROLE_TERM_ARGUMENT),
                        replacement(1, var(3), REPLACEMENT_ROLE_TERM_ARGUMENT),
                    ],
                ),
                "payload.replacements",
            ),
            (
                payload(
                    1,
                    TermPath::root(),
                    vec![replacement(99, var(2), REPLACEMENT_ROLE_TERM_ARGUMENT)],
                ),
                "payload.formal_variable_id",
            ),
            (
                payload(
                    1,
                    TermPath::root(),
                    vec![replacement(1, var(99), REPLACEMENT_ROLE_TERM_ARGUMENT)],
                ),
                "payload.actual_term",
            ),
        ];

        for (bad_payload, field_path) in cases {
            let context = context(vec![bad_payload]);
            let record =
                replay_substitutions(input(&target, &certificate, Some(&context), limits()))
                    .expect_err("malformed or deferred payload");
            assert_rejection(
                &record,
                RejectionDetail::InvalidSubstitution,
                Some(1),
                Some(field_path),
            );
        }
    }

    #[test]
    fn rejects_target_mismatch_manifest_errors_and_capture_without_alpha_repair() {
        let target = target();
        let mismatch = certificate(vec![substitution(1, var(1), var(2))]);
        let no_rewrite = context(vec![payload(1, TermPath::root(), Vec::new())]);
        let record = replay_substitutions(input(&target, &mismatch, Some(&no_rewrite), limits()))
            .expect_err("target mismatch");
        assert_rejection(
            &record,
            RejectionDetail::InvalidSubstitution,
            Some(1),
            Some("target_term"),
        );

        let bad_source = certificate(vec![substitution(1, var(99), var(99))]);
        let bad_source_context = context(vec![payload(1, TermPath::root(), Vec::new())]);
        let record = replay_substitutions(input(
            &target,
            &bad_source,
            Some(&bad_source_context),
            limits(),
        ))
        .expect_err("manifest-incompatible source");
        assert_rejection(
            &record,
            RejectionDetail::InvalidSubstitution,
            Some(1),
            Some("source_term"),
        );

        let captured = certificate_with_binder(
            substitution(1, binder(10, var(1)), binder(10, var(2))),
            binder_context(vec![(10, 0, 2, 1)], vec![1, 2, 3], Vec::new()),
        );
        let capture_context = context(vec![payload(
            1,
            TermPath::root(),
            vec![replacement(1, var(2), REPLACEMENT_ROLE_TERM_ARGUMENT)],
        )]);
        let record =
            replay_substitutions(input(&target, &captured, Some(&capture_context), limits()))
                .expect_err("capture is rejected without alpha repair");
        assert_rejection(
            &record,
            RejectionDetail::InvalidSubstitution,
            Some(1),
            Some("payload.actual_term"),
        );

        let forged_under_binder = certificate_with_binder(
            substitution(1, binder(10, var(1)), binder(10, var(3))),
            binder_context(vec![(10, 0, 2, 1)], vec![1, 2, 3], Vec::new()),
        );
        let non_capturing_context = context(vec![payload(
            1,
            TermPath::root(),
            vec![replacement(1, var(3), REPLACEMENT_ROLE_TERM_ARGUMENT)],
        )]);
        let record = replay_substitutions(input(
            &target,
            &forged_under_binder,
            Some(&non_capturing_context),
            limits(),
        ))
        .expect_err("task 11 rejects under-binder substitution without semantic replay");
        assert_rejection(
            &record,
            RejectionDetail::InvalidSubstitution,
            Some(1),
            Some("payload.rewrite_path"),
        );
    }

    #[test]
    fn binder_context_decode_matrix_is_deterministic() {
        let target = target();
        let base = substitution(1, var(1), var(2));
        let invalid_cases = [
            (
                replace_schema_version(binder_context(Vec::new(), vec![1, 2, 3], Vec::new()), 2),
                "binder_context.schema_version",
            ),
            (
                binder_context(vec![(10, 0, 1, 9)], vec![1, 2, 3], Vec::new()),
                "binder_context.binder_role",
            ),
            (
                truncated_binder_context(),
                "binder_context.schematic_variables",
            ),
            (binder_context_with_frame_count(8), "binder_context.frames"),
            (
                binder_context_with_free_variable_count(8),
                "binder_context.free_variables",
            ),
            (
                binder_context_with_schematic_variable_count(8),
                "binder_context.schematic_variables",
            ),
            (
                {
                    let mut bytes = binder_context(Vec::new(), vec![1, 2, 3], Vec::new());
                    bytes.push(0);
                    bytes
                },
                "binder_context.trailing_bytes",
            ),
            (
                binder_context(
                    vec![(10, 0, 1, 1), (10, 1, 2, 1)],
                    vec![1, 2, 3],
                    Vec::new(),
                ),
                "binder_context.frames",
            ),
            (
                binder_context(
                    vec![(10, 0, 1, 1), (11, 1, 1, 1)],
                    vec![1, 2, 3],
                    Vec::new(),
                ),
                "binder_context.frames",
            ),
            (
                binder_context(
                    vec![(10, 2, 1, 1), (11, 1, 2, 1)],
                    vec![1, 2, 3],
                    Vec::new(),
                ),
                "binder_context.frames",
            ),
            (
                binder_context(Vec::new(), vec![2, 1], Vec::new()),
                "binder_context.free_variables",
            ),
            (
                binder_context(Vec::new(), vec![1, 2, 3], vec![3, 2]),
                "binder_context.schematic_variables",
            ),
            (
                binder_context(vec![(10, 0, 99, 1)], vec![1, 2, 3], Vec::new()),
                "binder_context.frames",
            ),
            (
                binder_context(Vec::new(), vec![1, 99], Vec::new()),
                "binder_context.free_variables",
            ),
            (
                binder_context(Vec::new(), vec![1, 2, 3], vec![99]),
                "binder_context.schematic_variables",
            ),
            (
                binder_context(vec![(10, 1, 1, 1)], vec![1, 2, 3], Vec::new()),
                "binder_context.frames",
            ),
        ];

        for (bytes, field_path) in invalid_cases {
            let certificate = certificate_with_binder(base.clone(), bytes);
            let context = context(vec![payload(
                1,
                TermPath::root(),
                vec![replacement(1, var(2), REPLACEMENT_ROLE_TERM_ARGUMENT)],
            )]);
            let record =
                replay_substitutions(input(&target, &certificate, Some(&context), limits()))
                    .expect_err("invalid binder context");
            assert_rejection(
                &record,
                RejectionDetail::InvalidSubstitution,
                Some(1),
                Some(field_path),
            );
        }

        let incompatible = certificate_with_binder(
            substitution(1, binder(99, var(1)), binder(99, var(2))),
            binder_context(Vec::new(), vec![1, 2, 3], Vec::new()),
        );
        let incompatible_context = context(vec![payload(
            1,
            TermPath::root(),
            vec![replacement(1, var(2), REPLACEMENT_ROLE_TERM_ARGUMENT)],
        )]);
        let record = replay_substitutions(input(
            &target,
            &incompatible,
            Some(&incompatible_context),
            limits(),
        ))
        .expect_err("frame/term incompatibility");
        assert_rejection(
            &record,
            RejectionDetail::InvalidSubstitution,
            Some(1),
            Some("source_term"),
        );

        let traversal_mismatch = certificate_with_binder(
            substitution(
                1,
                pair(binder(10, var(1)), binder(11, var(2))),
                pair(binder(10, var(1)), binder(11, var(2))),
            ),
            binder_context(
                vec![(11, 0, 2, 1), (10, 1, 1, 1)],
                vec![1, 2, 3],
                Vec::new(),
            ),
        );
        let identity_context = context(vec![payload(1, TermPath::root(), Vec::new())]);
        let record = replay_substitutions(input(
            &target,
            &traversal_mismatch,
            Some(&identity_context),
            limits(),
        ))
        .expect_err("term traversal order must match frame canonical indices");
        assert_rejection(
            &record,
            RejectionDetail::InvalidSubstitution,
            Some(1),
            Some("source_term"),
        );

        let unused_frame = certificate_with_binder(
            base.clone(),
            binder_context(vec![(10, 0, 1, 1)], vec![1, 2, 3], Vec::new()),
        );
        let record = replay_substitutions(input(
            &target,
            &unused_frame,
            Some(&identity_context),
            limits(),
        ))
        .expect_err("frame set must exactly match used normalized binders");
        assert_rejection(
            &record,
            RejectionDetail::InvalidSubstitution,
            Some(1),
            Some("binder_context"),
        );

        let resource_certificate = certificate_with_binder(
            base,
            binder_context(vec![(10, 0, 1, 1)], vec![1, 2, 3], Vec::new()),
        );
        let context = context(vec![payload(
            1,
            TermPath::root(),
            vec![replacement(1, var(2), REPLACEMENT_ROLE_TERM_ARGUMENT)],
        )]);
        let record = replay_substitutions(input(
            &target,
            &resource_certificate,
            Some(&context),
            SubstitutionReplayLimits {
                max_binder_frames: 0,
                ..limits()
            },
        ))
        .expect_err("frame limit");
        assert_rejection(
            &record,
            RejectionDetail::ResourceExhaustion,
            Some(1),
            Some("binder_context.frames"),
        );

        let record = replay_substitutions(input(
            &target,
            &resource_certificate,
            Some(&context),
            SubstitutionReplayLimits {
                max_binder_context_bytes: 1,
                ..limits()
            },
        ))
        .expect_err("byte limit");
        assert_rejection(
            &record,
            RejectionDetail::ResourceExhaustion,
            Some(1),
            Some("binder_context"),
        );
    }

    #[test]
    fn missing_provenance_and_side_condition_shape_are_rejected_at_first_use() {
        let target = target();
        let certificate = certificate_with_refs(vec![1], vec![2]);
        let no_context = replay_substitutions(input(&target, &certificate, None, limits()))
            .expect_err("missing context");
        assert_rejection(
            &no_context,
            RejectionDetail::MissingProvenance,
            Some(1),
            Some("substitution_context"),
        );

        let empty_provenance = SubstitutionContext::new(
            Some(Vec::new()),
            vec![payload(
                1,
                TermPath::root(),
                vec![replacement(1, var(2), REPLACEMENT_ROLE_TERM_ARGUMENT)],
            )],
            Vec::new(),
            Vec::new(),
        )
        .expect("shape-valid context");
        let record = replay_substitutions(input(
            &target,
            &certificate,
            Some(&empty_provenance),
            limits(),
        ))
        .expect_err("missing provenance");
        assert_rejection(
            &record,
            RejectionDetail::MissingProvenance,
            Some(1),
            Some("substitution_context.provenance"),
        );

        let context = context(vec![payload(
            1,
            TermPath::root(),
            vec![replacement(1, var(2), REPLACEMENT_ROLE_TERM_ARGUMENT)],
        )]);
        let record = replay_substitutions(input(&target, &certificate, Some(&context), limits()))
            .expect_err("missing witness");
        assert_rejection(
            &record,
            RejectionDetail::MissingProvenance,
            Some(1),
            Some("freshness_witness_refs"),
        );

        let only_constraint_missing = certificate_with_refs(Vec::new(), vec![2]);
        let record = replay_substitutions(input(
            &target,
            &only_constraint_missing,
            Some(&context),
            limits(),
        ))
        .expect_err("missing free-variable constraint");
        assert_rejection(
            &record,
            RejectionDetail::MissingProvenance,
            Some(1),
            Some("free_variable_constraint_refs"),
        );

        let payloads = || {
            vec![payload(
                1,
                TermPath::root(),
                vec![replacement(1, var(2), REPLACEMENT_ROLE_TERM_ARGUMENT)],
            )]
        };
        let side_cases = [
            (
                context_with_side(
                    payloads(),
                    vec![freshness(1).with_owner(99)],
                    vec![constraint(2)],
                ),
                "freshness_witness.owner_substitution_id",
            ),
            (
                context_with_side(
                    payloads(),
                    vec![freshness_with_path(
                        1,
                        path(vec![TermPathSegment::application_argument(0)]),
                        vec![VariableId(1), VariableId(2)],
                    )],
                    vec![constraint(2)],
                ),
                "freshness_witness.binder_path",
            ),
            (
                context_with_side(
                    payloads(),
                    vec![freshness_with_path(
                        1,
                        path(vec![TermPathSegment::new(9, 0)]),
                        vec![VariableId(1), VariableId(2)],
                    )],
                    vec![constraint(2)],
                ),
                "freshness_witness.binder_path",
            ),
            (
                context_with_side(
                    payloads(),
                    vec![freshness_with_path(
                        1,
                        TermPath::root(),
                        vec![VariableId(2), VariableId(1)],
                    )],
                    vec![constraint(2)],
                ),
                "freshness_witness.avoided_variables",
            ),
            (
                context_with_side(
                    payloads(),
                    vec![freshness(1)],
                    vec![constraint(2).with_owner(99)],
                ),
                "free_variable_constraint.owner_substitution_id",
            ),
            (
                context_with_side(
                    payloads(),
                    vec![freshness(1)],
                    vec![constraint(2).with_kind(9)],
                ),
                "free_variable_constraint.constraint_kind",
            ),
            (
                context_with_side(
                    payloads(),
                    vec![freshness(1)],
                    vec![constraint_with_path(
                        2,
                        path(vec![TermPathSegment::application_argument(0)]),
                        vec![VariableId(2), VariableId(3)],
                    )],
                ),
                "free_variable_constraint.term_path",
            ),
            (
                context_with_side(
                    payloads(),
                    vec![freshness(1)],
                    vec![constraint_with_path(
                        2,
                        path(vec![TermPathSegment::new(9, 0)]),
                        vec![VariableId(2), VariableId(3)],
                    )],
                ),
                "free_variable_constraint.term_path",
            ),
            (
                context_with_side(
                    payloads(),
                    vec![freshness(1)],
                    vec![constraint_with_path(
                        2,
                        TermPath::root(),
                        vec![VariableId(3), VariableId(2)],
                    )],
                ),
                "free_variable_constraint.capture_set",
            ),
        ];

        for (bad_context, field_path) in side_cases {
            let record =
                replay_substitutions(input(&target, &certificate, Some(&bad_context), limits()))
                    .expect_err("malformed side condition");
            assert_rejection(
                &record,
                RejectionDetail::InvalidSubstitution,
                Some(1),
                Some(field_path),
            );
        }
    }

    #[test]
    fn resource_limits_fire_before_unbounded_payload_or_side_condition_work() {
        let target = target();
        let count_certificate = certificate(vec![substitution(1, var(1), var(2))]);
        let count_context = context(vec![payload(
            1,
            TermPath::root(),
            vec![replacement(1, var(2), REPLACEMENT_ROLE_TERM_ARGUMENT)],
        )]);
        let record = replay_substitutions(input(
            &target,
            &count_certificate,
            Some(&count_context),
            SubstitutionReplayLimits {
                max_substitutions: 0,
                ..limits()
            },
        ))
        .expect_err("substitution count limit");
        assert_rejection(&record, RejectionDetail::ResourceExhaustion, Some(1), None);

        let record = replay_substitutions(input(
            &target,
            &count_certificate,
            Some(&count_context),
            SubstitutionReplayLimits {
                max_term_encoding_bytes: 1,
                ..limits()
            },
        ))
        .expect_err("term byte limit");
        assert_rejection(
            &record,
            RejectionDetail::ResourceExhaustion,
            Some(1),
            Some("source_term"),
        );

        let actual_size_context = context(vec![payload(
            1,
            TermPath::root(),
            vec![replacement(
                1,
                pair(var(2), var(2)),
                REPLACEMENT_ROLE_TERM_ARGUMENT,
            )],
        )]);
        let record = replay_substitutions(input(
            &target,
            &count_certificate,
            Some(&actual_size_context),
            SubstitutionReplayLimits {
                max_term_encoding_bytes: 20,
                ..limits()
            },
        ))
        .expect_err("actual term byte limit");
        assert_rejection(
            &record,
            RejectionDetail::ResourceExhaustion,
            Some(1),
            Some("payload.actual_term"),
        );

        let expansion_certificate =
            certificate(vec![substitution(1, pair(var(1), var(1)), var(2))]);
        let expansion_context = context(vec![payload(
            1,
            TermPath::root(),
            vec![replacement(
                1,
                pair(var(2), var(2)),
                REPLACEMENT_ROLE_TERM_ARGUMENT,
            )],
        )]);
        let record = replay_substitutions(input(
            &target,
            &expansion_certificate,
            Some(&expansion_context),
            SubstitutionReplayLimits {
                max_term_encoding_bytes: 40,
                ..limits()
            },
        ))
        .expect_err("replayed target byte limit is checked before cloning result");
        assert_rejection(
            &record,
            RejectionDetail::ResourceExhaustion,
            Some(1),
            Some("target_term"),
        );

        let depth_expansion_certificate = certificate(vec![substitution(
            1,
            pair(var(1), var(2)),
            pair(var(2), var(2)),
        )]);
        let depth_expansion_context = context(vec![payload(
            1,
            path(vec![TermPathSegment::application_argument(0)]),
            vec![replacement(
                1,
                pair(var(2), var(2)),
                REPLACEMENT_ROLE_TERM_ARGUMENT,
            )],
        )]);
        let record = replay_substitutions(input(
            &target,
            &depth_expansion_certificate,
            Some(&depth_expansion_context),
            SubstitutionReplayLimits {
                max_term_recursion_depth: 1,
                ..limits()
            },
        ))
        .expect_err("replayed target depth limit is checked before cloning result");
        assert_rejection(
            &record,
            RejectionDetail::ResourceExhaustion,
            Some(1),
            Some("target_term"),
        );

        let too_many_replacements = context(vec![payload(
            1,
            TermPath::root(),
            vec![replacement(1, var(2), REPLACEMENT_ROLE_TERM_ARGUMENT)],
        )]);
        let record = replay_substitutions(input(
            &target,
            &count_certificate,
            Some(&too_many_replacements),
            SubstitutionReplayLimits {
                max_payload_replacements: 0,
                ..limits()
            },
        ))
        .expect_err("payload replacement count limit");
        assert_rejection(
            &record,
            RejectionDetail::ResourceExhaustion,
            Some(1),
            Some("payload.replacements"),
        );

        let actual_context = context(vec![payload(
            1,
            TermPath::root(),
            vec![replacement(1, deep_term(3), REPLACEMENT_ROLE_TERM_ARGUMENT)],
        )]);
        let record = replay_substitutions(input(
            &target,
            &count_certificate,
            Some(&actual_context),
            SubstitutionReplayLimits {
                max_term_recursion_depth: 1,
                ..limits()
            },
        ))
        .expect_err("actual term depth limit");
        assert_rejection(
            &record,
            RejectionDetail::ResourceExhaustion,
            Some(1),
            Some("payload.actual_term"),
        );

        let context = context(vec![payload(
            1,
            path(vec![TermPathSegment::application_argument(0)]),
            vec![replacement(1, var(2), REPLACEMENT_ROLE_TERM_ARGUMENT)],
        )]);
        let path_certificate = certificate(vec![substitution(
            1,
            pair(var(1), var(1)),
            pair(var(2), var(1)),
        )]);
        let record = replay_substitutions(input(
            &target,
            &path_certificate,
            Some(&context),
            SubstitutionReplayLimits {
                max_term_path_segments: 0,
                ..limits()
            },
        ))
        .expect_err("path segment limit");
        assert_rejection(
            &record,
            RejectionDetail::ResourceExhaustion,
            Some(1),
            Some("payload.rewrite_path"),
        );

        let refs_certificate = certificate_with_refs(vec![1, 2], Vec::new());
        let refs_context = context_with_side(
            vec![payload(
                1,
                TermPath::root(),
                vec![replacement(1, var(2), REPLACEMENT_ROLE_TERM_ARGUMENT)],
            )],
            vec![freshness(1), freshness(2)],
            Vec::new(),
        );
        let record = replay_substitutions(input(
            &target,
            &refs_certificate,
            Some(&refs_context),
            SubstitutionReplayLimits {
                max_freshness_witnesses: 1,
                ..limits()
            },
        ))
        .expect_err("freshness ref count limit");
        assert_rejection(
            &record,
            RejectionDetail::ResourceExhaustion,
            Some(1),
            Some("freshness_witness_refs"),
        );

        let refs_certificate = certificate_with_refs(Vec::new(), vec![1, 2]);
        let refs_context = context_with_side(
            vec![payload(
                1,
                TermPath::root(),
                vec![replacement(1, var(2), REPLACEMENT_ROLE_TERM_ARGUMENT)],
            )],
            Vec::new(),
            vec![constraint(1), constraint(2)],
        );
        let record = replay_substitutions(input(
            &target,
            &refs_certificate,
            Some(&refs_context),
            SubstitutionReplayLimits {
                max_free_variable_constraints: 1,
                ..limits()
            },
        ))
        .expect_err("free-variable ref count limit");
        assert_rejection(
            &record,
            RejectionDetail::ResourceExhaustion,
            Some(1),
            Some("free_variable_constraint_refs"),
        );

        let refs_certificate = certificate_with_refs(vec![1], vec![2]);
        let refs_context = context_with_side(
            vec![payload(
                1,
                TermPath::root(),
                vec![replacement(1, var(2), REPLACEMENT_ROLE_TERM_ARGUMENT)],
            )],
            vec![freshness(1)],
            vec![constraint(2)],
        );
        let record = replay_substitutions(input(
            &target,
            &refs_certificate,
            Some(&refs_context),
            SubstitutionReplayLimits {
                max_avoided_variables: 1,
                ..limits()
            },
        ))
        .expect_err("avoided variable limit");
        assert_rejection(
            &record,
            RejectionDetail::ResourceExhaustion,
            Some(1),
            Some("freshness_witness.avoided_variables"),
        );
        let record = replay_substitutions(input(
            &target,
            &refs_certificate,
            Some(&refs_context),
            SubstitutionReplayLimits {
                max_capture_set_variables: 1,
                ..limits()
            },
        ))
        .expect_err("capture set limit");
        assert_rejection(
            &record,
            RejectionDetail::ResourceExhaustion,
            Some(1),
            Some("free_variable_constraint.capture_set"),
        );
    }

    #[test]
    fn context_constructor_canonicalizes_and_first_use_ignores_unused_malformed_entries() {
        let target = target();
        let certificate = certificate(vec![substitution(1, var(1), var(2))]);
        let context = context_with_side(
            vec![
                payload(
                    99,
                    TermPath::root(),
                    vec![replacement(
                        1,
                        var(99),
                        REPLACEMENT_ROLE_CAPTURED_FREE_VARIABLE,
                    )],
                )
                .with_kind(PAYLOAD_KIND_LOCAL_ABBREVIATION_EXPANSION),
                payload(
                    1,
                    TermPath::root(),
                    vec![replacement(1, var(2), REPLACEMENT_ROLE_TERM_ARGUMENT)],
                ),
            ],
            vec![freshness(99).with_owner(99)],
            vec![constraint(99).with_kind(9)],
        );

        replay_substitutions(input(&target, &certificate, Some(&context), limits()))
            .expect("unused malformed context entries are ignored");

        let side_order_certificate = certificate_with_refs(vec![1, 2], vec![2, 3]);
        let side_order_context = context_with_side(
            vec![payload(
                1,
                TermPath::root(),
                vec![replacement(1, var(2), REPLACEMENT_ROLE_TERM_ARGUMENT)],
            )],
            vec![freshness(2), freshness(1)],
            vec![constraint(3), constraint(2)],
        );
        replay_substitutions(input(
            &target,
            &side_order_certificate,
            Some(&side_order_context),
            limits(),
        ))
        .expect("constructor canonicalizes witness and constraint input order");

        let duplicate_payload = SubstitutionContext::new(
            Some(vec![7]),
            vec![
                payload(1, TermPath::root(), Vec::new()),
                payload(1, TermPath::root(), Vec::new()),
            ],
            Vec::new(),
            Vec::new(),
        )
        .expect_err("duplicate payload id");
        assert_eq!(
            duplicate_payload,
            SubstitutionContextError::DuplicateSubstitutionPayload { substitution_id: 1 }
        );

        let duplicate_witness = SubstitutionContext::new(
            Some(vec![7]),
            Vec::new(),
            vec![freshness(1), freshness(1)],
            Vec::new(),
        )
        .expect_err("duplicate witness id");
        assert_eq!(
            duplicate_witness,
            SubstitutionContextError::DuplicateFreshnessWitness { witness_id: 1 }
        );

        let duplicate_constraint = SubstitutionContext::new(
            Some(vec![7]),
            Vec::new(),
            Vec::new(),
            vec![constraint(1), constraint(1)],
        )
        .expect_err("duplicate constraint id");
        assert_eq!(
            duplicate_constraint,
            SubstitutionContextError::DuplicateFreeVariableConstraint { constraint_id: 1 }
        );

        let ambiguous_payload_context = unchecked_context(
            vec![
                payload(
                    1,
                    TermPath::root(),
                    vec![replacement(1, var(2), REPLACEMENT_ROLE_TERM_ARGUMENT)],
                ),
                payload(
                    99,
                    TermPath::root(),
                    vec![replacement(1, var(3), REPLACEMENT_ROLE_TERM_ARGUMENT)],
                ),
                payload(1, TermPath::root(), Vec::new()),
            ],
            Vec::new(),
            Vec::new(),
        );
        let record = replay_substitutions(input(
            &target,
            &certificate,
            Some(&ambiguous_payload_context),
            limits(),
        ))
        .expect_err("ambiguous payload id maps to missing provenance");
        assert_rejection(
            &record,
            RejectionDetail::MissingProvenance,
            Some(1),
            Some("substitution_payload"),
        );

        let refs_certificate = certificate_with_refs(vec![1], vec![2]);
        let ambiguous_witness_context = unchecked_context(
            vec![payload(
                1,
                TermPath::root(),
                vec![replacement(1, var(2), REPLACEMENT_ROLE_TERM_ARGUMENT)],
            )],
            vec![freshness(1), freshness(99), freshness(1)],
            vec![constraint(2)],
        );
        let record = replay_substitutions(input(
            &target,
            &refs_certificate,
            Some(&ambiguous_witness_context),
            limits(),
        ))
        .expect_err("ambiguous witness id maps to missing provenance");
        assert_rejection(
            &record,
            RejectionDetail::MissingProvenance,
            Some(1),
            Some("freshness_witness_refs"),
        );

        let ambiguous_constraint_context = unchecked_context(
            vec![payload(
                1,
                TermPath::root(),
                vec![replacement(1, var(2), REPLACEMENT_ROLE_TERM_ARGUMENT)],
            )],
            vec![freshness(1)],
            vec![constraint(2), constraint(99), constraint(2)],
        );
        let record = replay_substitutions(input(
            &target,
            &refs_certificate,
            Some(&ambiguous_constraint_context),
            limits(),
        ))
        .expect_err("ambiguous constraint id maps to missing provenance");
        assert_rejection(
            &record,
            RejectionDetail::MissingProvenance,
            Some(1),
            Some("free_variable_constraint_refs"),
        );
    }

    #[test]
    fn report_binding_rejects_target_certificate_and_context_mismatches() {
        let target = target();
        let certificate = certificate(vec![substitution(1, var(1), var(2))]);
        let context = context(vec![payload(
            1,
            TermPath::root(),
            vec![replacement(1, var(2), REPLACEMENT_ROLE_TERM_ARGUMENT)],
        )]);
        let report = replay_substitutions(input(&target, &certificate, Some(&context), limits()))
            .expect("valid report");

        let other_target = TargetVcFingerprint::new(1, vec![43]);
        let record = checked_substitutions_for_input(
            input(&other_target, &certificate, Some(&context), limits()),
            &report,
        )
        .expect_err("target mismatch");
        assert_eq!(record.target_vc_fingerprint(), &other_target);
        assert_rejection(
            &record,
            RejectionDetail::InvalidSubstitution,
            None,
            Some("substitution_report_binding"),
        );

        let other_certificate =
            certificate_with_hash(vec![substitution(1, var(1), var(2))], vec![99]);
        let record = checked_substitutions_for_input(
            input(&target, &other_certificate, Some(&context), limits()),
            &report,
        )
        .expect_err("certificate hash mismatch");
        assert_rejection(
            &record,
            RejectionDetail::InvalidSubstitution,
            None,
            Some("substitution_report_binding"),
        );

        let other_context = SubstitutionContext::new(
            Some(vec![8]),
            vec![payload(
                1,
                TermPath::root(),
                vec![replacement(1, var(2), REPLACEMENT_ROLE_TERM_ARGUMENT)],
            )],
            Vec::new(),
            Vec::new(),
        )
        .expect("valid context");
        let record = checked_substitutions_for_input(
            input(&target, &certificate, Some(&other_context), limits()),
            &report,
        )
        .expect_err("context provenance mismatch");
        assert_rejection(
            &record,
            RejectionDetail::InvalidSubstitution,
            None,
            Some("substitution_report_binding"),
        );
    }

    trait PayloadEntryTestExt {
        fn with_kind(self, payload_kind: u8) -> Self;
        fn with_owner(self, owner_substitution_id: u32) -> Self;
    }

    impl PayloadEntryTestExt for SubstitutionPayloadEntry {
        fn with_kind(mut self, payload_kind: u8) -> Self {
            self.payload.payload_kind = payload_kind;
            self
        }

        fn with_owner(mut self, owner_substitution_id: u32) -> Self {
            self.payload.owner_substitution_id = owner_substitution_id;
            self
        }
    }

    trait FreshnessTestExt {
        fn with_owner(self, owner_substitution_id: u32) -> Self;
    }

    impl FreshnessTestExt for FreshnessWitness {
        fn with_owner(mut self, owner_substitution_id: u32) -> Self {
            self.owner_substitution_id = owner_substitution_id;
            self
        }
    }

    trait ConstraintTestExt {
        fn with_kind(self, constraint_kind: u8) -> Self;
        fn with_owner(self, owner_substitution_id: u32) -> Self;
    }

    impl ConstraintTestExt for FreeVariableConstraint {
        fn with_kind(mut self, constraint_kind: u8) -> Self {
            self.constraint_kind = constraint_kind;
            self
        }

        fn with_owner(mut self, owner_substitution_id: u32) -> Self {
            self.owner_substitution_id = owner_substitution_id;
            self
        }
    }

    fn input<'a>(
        target: &'a TargetVcFingerprint,
        certificate: &'a ParsedCertificate,
        substitution_context: Option<&'a SubstitutionContext>,
        limits: SubstitutionReplayLimits,
    ) -> SubstitutionCheckInput<'a> {
        SubstitutionCheckInput {
            target_vc_fingerprint: target,
            certificate,
            substitution_context,
            limits,
        }
    }

    fn assert_rejection(
        record: &RejectionRecord,
        detail: RejectionDetail,
        substitution_id: Option<u32>,
        field_path: Option<&'static str>,
    ) {
        assert_eq!(record.category(), RejectionCategory::KernelRejection);
        assert_eq!(record.detail(), detail);
        assert_eq!(record.detail().stable_key(), detail.stable_key());
        assert_eq!(record.location().substitution_id, substitution_id);
        if let Some(field_path) = field_path {
            assert_eq!(record.location().field_path, Some(field_path));
        }
    }

    fn target() -> TargetVcFingerprint {
        TargetVcFingerprint::new(1, vec![42])
    }

    fn limits() -> SubstitutionReplayLimits {
        SubstitutionReplayLimits {
            max_substitutions: 8,
            max_binder_context_bytes: 512,
            max_binder_frames: 8,
            max_freshness_witnesses: 4,
            max_free_variable_constraints: 4,
            max_term_encoding_bytes: 4096,
            max_term_recursion_depth: 16,
            max_alpha_renames: 0,
            max_payload_replacements: 8,
            max_term_path_segments: 8,
            max_avoided_variables: 8,
            max_capture_set_variables: 8,
        }
    }

    fn certificate(substitutions: Vec<SubstitutionEntry>) -> ParsedCertificate {
        certificate_with_hash(substitutions, vec![1, 2, 3])
    }

    fn certificate_with_hash(
        substitutions: Vec<SubstitutionEntry>,
        canonical_hash_input: Vec<u8>,
    ) -> ParsedCertificate {
        ParsedCertificate::new_for_kernel_tests(ParsedCertificateTestParts {
            schema_version: 1,
            encoding_version: 1,
            kernel_profile: KernelProfileRecord::v1(1, ClauseTautologyPolicy::Reject),
            target_vc: Fingerprint::new(1, vec![42]),
            symbol_manifest: vec![SymbolManifestEntry { symbol: symbol() }],
            variable_manifest: (1..=4)
                .map(|id| VariableManifestEntry {
                    variable_id: VariableId(id),
                })
                .collect(),
            imported_axioms: Vec::new(),
            imported_theorems: Vec::new(),
            generated_clauses: Vec::new(),
            substitutions,
            resolution_trace: Vec::new(),
            derived_facts: Vec::new(),
            final_goal: FinalGoalRef {
                namespace: FinalGoalNamespace::GeneratedClause,
                id: 0,
            },
            canonical_hash_input,
        })
    }

    fn certificate_with_binder(
        mut substitution: SubstitutionEntry,
        binder_context_encoding: Vec<u8>,
    ) -> ParsedCertificate {
        substitution.binder_context_encoding = binder_context_encoding;
        certificate(vec![substitution])
    }

    fn certificate_with_refs(
        freshness_witness_refs: Vec<u32>,
        free_variable_constraint_refs: Vec<u32>,
    ) -> ParsedCertificate {
        let mut substitution = substitution(1, var(1), var(2));
        substitution.freshness_witness_refs = freshness_witness_refs;
        substitution.free_variable_constraint_refs = free_variable_constraint_refs;
        certificate(vec![substitution])
    }

    fn substitution(
        substitution_id: u32,
        source_term: Term,
        target_term: Term,
    ) -> SubstitutionEntry {
        SubstitutionEntry {
            substitution_id,
            source_term,
            target_term,
            binder_context_encoding: binder_context(Vec::new(), vec![1, 2, 3, 4], Vec::new()),
            freshness_witness_refs: Vec::new(),
            free_variable_constraint_refs: Vec::new(),
        }
    }

    fn context(payloads: Vec<SubstitutionPayloadEntry>) -> SubstitutionContext {
        context_with_side(payloads, Vec::new(), Vec::new())
    }

    fn context_with_side(
        payloads: Vec<SubstitutionPayloadEntry>,
        freshness_witnesses: Vec<FreshnessWitness>,
        free_variable_constraints: Vec<FreeVariableConstraint>,
    ) -> SubstitutionContext {
        SubstitutionContext::new(
            Some(vec![7]),
            payloads,
            freshness_witnesses,
            free_variable_constraints,
        )
        .expect("valid context shape")
    }

    fn unchecked_context(
        payloads: Vec<SubstitutionPayloadEntry>,
        freshness_witnesses: Vec<FreshnessWitness>,
        free_variable_constraints: Vec<FreeVariableConstraint>,
    ) -> SubstitutionContext {
        SubstitutionContext {
            provenance_fingerprint: Some(vec![7]),
            substitution_payloads: payloads,
            freshness_witnesses,
            free_variable_constraints,
            canonical_shape: false,
        }
    }

    fn payload(
        substitution_id: u32,
        rewrite_path: TermPath,
        replacements: Vec<Replacement>,
    ) -> SubstitutionPayloadEntry {
        SubstitutionPayloadEntry::new(
            substitution_id,
            SubstitutionPayload::new(
                substitution_id,
                PAYLOAD_KIND_FORMAL_TO_ACTUAL_MAP,
                rewrite_path,
                replacements,
            ),
        )
    }

    fn replacement(
        formal_variable_id: u32,
        actual_term: Term,
        replacement_role: u8,
    ) -> Replacement {
        Replacement::new(
            VariableId(formal_variable_id),
            actual_term,
            replacement_role,
        )
    }

    fn freshness(witness_id: u32) -> FreshnessWitness {
        freshness_with_path(
            witness_id,
            TermPath::root(),
            vec![VariableId(1), VariableId(2)],
        )
    }

    fn freshness_with_path(
        witness_id: u32,
        binder_path: TermPath,
        avoided_variables: Vec<VariableId>,
    ) -> FreshnessWitness {
        FreshnessWitness::new(
            witness_id,
            1,
            VariableId(3),
            binder_path,
            avoided_variables,
            0,
        )
    }

    fn constraint(constraint_id: u32) -> FreeVariableConstraint {
        constraint_with_path(
            constraint_id,
            TermPath::root(),
            vec![VariableId(2), VariableId(3)],
        )
    }

    fn constraint_with_path(
        constraint_id: u32,
        term_path: TermPath,
        capture_set: Vec<VariableId>,
    ) -> FreeVariableConstraint {
        FreeVariableConstraint::new(
            constraint_id,
            1,
            CONSTRAINT_KIND_MUST_REMAIN_FREE_AT_PATH,
            VariableId(1),
            term_path,
            capture_set,
        )
    }

    fn path(segments: Vec<TermPathSegment>) -> TermPath {
        TermPath::new(segments)
    }

    fn var(id: u32) -> Term {
        Term::Variable(VariableId(id))
    }

    fn pair(left: Term, right: Term) -> Term {
        Term::Application {
            symbol: symbol(),
            arguments: vec![left, right],
        }
    }

    fn deep_term(depth: u32) -> Term {
        if depth == 0 {
            return var(1);
        }
        pair(deep_term(depth - 1), var(1))
    }

    fn binder(binder_id: u32, body: Term) -> Term {
        Term::BinderNormalized {
            binder_id,
            body: Box::new(body),
        }
    }

    const fn symbol() -> SymbolKey {
        SymbolKey::new(SymbolKind::Predicate, 1)
    }

    fn binder_context(
        frames: Vec<(u32, u32, u32, u8)>,
        free_variables: Vec<u32>,
        schematic_variables: Vec<u32>,
    ) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&BINDER_CONTEXT_SCHEMA_VERSION.to_be_bytes());
        bytes.extend_from_slice(&(frames.len() as u32).to_be_bytes());
        for (binder_id, canonical_index, variable_id, binder_role) in frames {
            bytes.extend_from_slice(&binder_id.to_be_bytes());
            bytes.extend_from_slice(&canonical_index.to_be_bytes());
            bytes.extend_from_slice(&variable_id.to_be_bytes());
            bytes.push(binder_role);
        }
        bytes.extend_from_slice(&(free_variables.len() as u32).to_be_bytes());
        for variable in free_variables {
            bytes.extend_from_slice(&variable.to_be_bytes());
        }
        bytes.extend_from_slice(&(schematic_variables.len() as u32).to_be_bytes());
        for variable in schematic_variables {
            bytes.extend_from_slice(&variable.to_be_bytes());
        }
        bytes
    }

    fn binder_context_with_frame_count(frame_count: u32) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&BINDER_CONTEXT_SCHEMA_VERSION.to_be_bytes());
        bytes.extend_from_slice(&frame_count.to_be_bytes());
        bytes
    }

    fn binder_context_with_free_variable_count(free_variable_count: u32) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&BINDER_CONTEXT_SCHEMA_VERSION.to_be_bytes());
        bytes.extend_from_slice(&0u32.to_be_bytes());
        bytes.extend_from_slice(&free_variable_count.to_be_bytes());
        bytes
    }

    fn binder_context_with_schematic_variable_count(schematic_variable_count: u32) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&BINDER_CONTEXT_SCHEMA_VERSION.to_be_bytes());
        bytes.extend_from_slice(&0u32.to_be_bytes());
        bytes.extend_from_slice(&0u32.to_be_bytes());
        bytes.extend_from_slice(&schematic_variable_count.to_be_bytes());
        bytes
    }

    fn replace_schema_version(mut bytes: Vec<u8>, schema_version: u16) -> Vec<u8> {
        bytes[0..2].copy_from_slice(&schema_version.to_be_bytes());
        bytes
    }

    fn truncated_binder_context() -> Vec<u8> {
        let mut bytes = binder_context(Vec::new(), vec![1, 2, 3], Vec::new());
        bytes.pop();
        bytes
    }
}
