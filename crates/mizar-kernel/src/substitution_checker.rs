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
const BINDER_ROLE_GENERATED_FRESH: u8 = 4;
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

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Default)]
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
#[non_exhaustive]
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
        let side_conditions =
            validate_side_conditions(input, &replay_context, &binder_context, context, entry)?;
        let alpha_plan = build_alpha_replay_plan(
            input,
            &replay_context,
            &binder_context,
            entry,
            payload,
            &side_conditions.freshness_witnesses,
        )?;
        measure_direct_substitution(input, &binder_context, entry, payload, &alpha_plan)?;
        let replayed =
            replay_direct_substitution(input, &binder_context, entry, payload, &alpha_plan)?;
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
        validate_free_variable_constraints(
            input,
            &binder_context,
            entry,
            &side_conditions.free_variable_constraints,
        )?;
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
    canonical_variable_order: Vec<VariableId>,
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
        let mut canonical_variable_order = Vec::with_capacity(certificate.variable_manifest.len());
        for variable in &certificate.variable_manifest {
            term = term.with_canonical_variable(variable.variable_id);
            canonical_variables.insert(variable.variable_id);
            canonical_variable_order.push(variable.variable_id);
        }
        Self {
            term,
            canonical_variables,
            canonical_variable_order,
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

struct SideConditionEvidence<'a> {
    freshness_witnesses: Vec<&'a FreshnessWitness>,
    free_variable_constraints: Vec<&'a FreeVariableConstraint>,
}

fn validate_side_conditions<'a>(
    input: SubstitutionCheckInput<'_>,
    replay_context: &ReplayContext,
    _binder_context: &BinderContext,
    context: &'a SubstitutionContext,
    entry: &SubstitutionEntry,
) -> SubstitutionCheckResult<SideConditionEvidence<'a>> {
    if entry.freshness_witness_refs.len() > input.limits.max_freshness_witnesses {
        return Err(resource_rejection(
            input.target_vc_fingerprint,
            substitution_location(entry).with_field_path("freshness_witness_refs"),
        ));
    }
    if entry.freshness_witness_refs.len() > input.limits.max_alpha_renames {
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
    let mut freshness_witnesses = Vec::with_capacity(entry.freshness_witness_refs.len());
    for witness_id in &entry.freshness_witness_refs {
        let witness = context.freshness_witness(*witness_id).map_err(|()| {
            missing_rejection(
                input.target_vc_fingerprint,
                substitution_location(entry).with_field_path("freshness_witness_refs"),
            )
        })?;
        validate_freshness_witness(input, replay_context, entry, witness)?;
        freshness_witnesses.push(witness);
    }
    let mut free_variable_constraints =
        Vec::with_capacity(entry.free_variable_constraint_refs.len());
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
        free_variable_constraints.push(constraint);
    }
    Ok(SideConditionEvidence {
        freshness_witnesses,
        free_variable_constraints,
    })
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

#[derive(Clone, Debug, Eq, PartialEq)]
struct AlphaReplayPlan {
    renames: Vec<AlphaRename>,
}

impl AlphaReplayPlan {
    fn empty() -> Self {
        Self {
            renames: Vec::new(),
        }
    }

    fn rename_for_path(&self, path: &[TermPathSegment]) -> Option<&AlphaRename> {
        self.renames
            .iter()
            .find(|rename| rename.source_path.segments == path)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct AlphaRename {
    source_path: TermPath,
    target_binder_id: u32,
    generated_variable_id: VariableId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ActiveBinder {
    source_variable_id: VariableId,
    target_variable_id: VariableId,
}

fn build_alpha_replay_plan(
    input: SubstitutionCheckInput<'_>,
    replay_context: &ReplayContext,
    binder_context: &BinderContext,
    entry: &SubstitutionEntry,
    payload: &SubstitutionPayload,
    witnesses: &[&FreshnessWitness],
) -> SubstitutionCheckResult<AlphaReplayPlan> {
    if witnesses.is_empty() {
        return Ok(AlphaReplayPlan::empty());
    }
    if witnesses.len() > input.limits.max_alpha_renames {
        return Err(resource_rejection(
            input.target_vc_fingerprint,
            substitution_location(entry).with_field_path("freshness_witness_refs"),
        ));
    }

    let mut seen_paths = BTreeSet::new();
    let mut seen_generated = BTreeSet::new();
    let mut renames = Vec::with_capacity(witnesses.len());
    for witness in witnesses {
        if !seen_paths.insert(witness.binder_path.clone())
            || !seen_generated.insert(witness.generated_variable_id)
        {
            return Err(invalid_rejection(
                input.target_vc_fingerprint,
                substitution_location(entry).with_field_path("freshness_witness.binder_path"),
            ));
        }
        let (source_binder_id, source_body) = binder_at_path(
            input,
            entry,
            &entry.source_term,
            &witness.binder_path,
            "freshness_witness.binder_path",
        )?;
        let (target_binder_id, _) = binder_at_path(
            input,
            entry,
            &entry.target_term,
            &witness.binder_path,
            "freshness_witness.binder_path",
        )?;
        let source_frame = binder_context
            .frame_for_binder(source_binder_id)
            .ok_or_else(|| {
                invalid_rejection(
                    input.target_vc_fingerprint,
                    substitution_location(entry).with_field_path("binder_context"),
                )
            })?;
        let target_frame = binder_context
            .frame_for_binder(target_binder_id)
            .ok_or_else(|| {
                invalid_rejection(
                    input.target_vc_fingerprint,
                    substitution_location(entry).with_field_path("binder_context"),
                )
            })?;
        if target_frame.binder_role != BINDER_ROLE_GENERATED_FRESH
            || target_frame.variable_id != witness.generated_variable_id
            || source_frame.variable_id == witness.generated_variable_id
        {
            return Err(invalid_rejection(
                input.target_vc_fingerprint,
                substitution_location(entry)
                    .with_field_path("freshness_witness.generated_variable_id"),
            ));
        }
        let expected_avoided = expected_avoided_variables(
            input,
            binder_context,
            entry,
            payload,
            &witness.binder_path,
            source_body,
            source_frame.variable_id,
        )?;
        if witness.avoided_variables != expected_avoided {
            return Err(invalid_rejection(
                input.target_vc_fingerprint,
                substitution_location(entry).with_field_path("freshness_witness.avoided_variables"),
            ));
        }
        if expected_avoided.contains(&witness.generated_variable_id) {
            return Err(invalid_rejection(
                input.target_vc_fingerprint,
                substitution_location(entry)
                    .with_field_path("freshness_witness.generated_variable_id"),
            ));
        }
        let expected_counter = deterministic_counter(
            replay_context,
            &expected_avoided,
            witness.generated_variable_id,
        )
        .ok_or_else(|| {
            invalid_rejection(
                input.target_vc_fingerprint,
                substitution_location(entry)
                    .with_field_path("freshness_witness.generated_variable_id"),
            )
        })?;
        if witness.deterministic_counter != expected_counter {
            return Err(invalid_rejection(
                input.target_vc_fingerprint,
                substitution_location(entry)
                    .with_field_path("freshness_witness.deterministic_counter"),
            ));
        }
        renames.push(AlphaRename {
            source_path: witness.binder_path.clone(),
            target_binder_id,
            generated_variable_id: witness.generated_variable_id,
        });
    }
    renames.sort_by(|left, right| left.source_path.cmp(&right.source_path));
    Ok(AlphaReplayPlan { renames })
}

fn expected_avoided_variables(
    input: SubstitutionCheckInput<'_>,
    binder_context: &BinderContext,
    entry: &SubstitutionEntry,
    payload: &SubstitutionPayload,
    binder_path: &TermPath,
    source_body: &Term,
    original_variable_id: VariableId,
) -> SubstitutionCheckResult<Vec<VariableId>> {
    let mut avoided = BTreeSet::new();
    let source_body_limit = input.limits.max_avoided_variables.saturating_add(1);
    let free_walk = FreeVariableWalk {
        input,
        binder_context,
        entry,
        limit: source_body_limit,
        field_path: "freshness_witness.avoided_variables",
    };
    collect_free_variables(free_walk, source_body, &mut Vec::new(), &mut avoided)?;
    avoided.remove(&original_variable_id);
    collect_inserted_actual_free_variables_below_binder(
        input,
        binder_context,
        entry,
        payload,
        binder_path,
        &mut avoided,
    )?;
    let avoided = avoided.into_iter().collect::<Vec<_>>();
    if avoided.len() > input.limits.max_avoided_variables {
        return Err(resource_rejection(
            input.target_vc_fingerprint,
            substitution_location(entry).with_field_path("freshness_witness.avoided_variables"),
        ));
    }
    Ok(avoided)
}

fn deterministic_counter(
    replay_context: &ReplayContext,
    avoided_variables: &[VariableId],
    generated_variable_id: VariableId,
) -> Option<u32> {
    let avoided = avoided_variables.iter().copied().collect::<BTreeSet<_>>();
    let mut counter = 0u32;
    for variable in &replay_context.canonical_variable_order {
        if avoided.contains(variable) {
            continue;
        }
        if *variable == generated_variable_id {
            return Some(counter);
        }
        counter = counter.checked_add(1)?;
    }
    None
}

fn replay_direct_substitution(
    input: SubstitutionCheckInput<'_>,
    binder_context: &BinderContext,
    entry: &SubstitutionEntry,
    payload: &SubstitutionPayload,
    alpha_plan: &AlphaReplayPlan,
) -> SubstitutionCheckResult<Term> {
    let walk = ReplayWalk {
        input,
        binder_context,
        entry,
        payload,
        alpha_plan,
    };
    apply_at_path(
        walk,
        &entry.source_term,
        &payload.rewrite_path.segments,
        &mut Vec::new(),
        &mut Vec::new(),
    )
}

#[derive(Clone, Copy)]
struct ReplayWalk<'input, 'walk> {
    input: SubstitutionCheckInput<'input>,
    binder_context: &'walk BinderContext,
    entry: &'walk SubstitutionEntry,
    payload: &'walk SubstitutionPayload,
    alpha_plan: &'walk AlphaReplayPlan,
}

fn measure_direct_substitution(
    input: SubstitutionCheckInput<'_>,
    binder_context: &BinderContext,
    entry: &SubstitutionEntry,
    payload: &SubstitutionPayload,
    alpha_plan: &AlphaReplayPlan,
) -> SubstitutionCheckResult<usize> {
    let walk = MeasureReplay {
        input,
        binder_context,
        entry,
        payload,
        alpha_plan,
    };
    measure_apply_at_path(
        walk,
        &entry.source_term,
        &payload.rewrite_path.segments,
        &mut Vec::new(),
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
    alpha_plan: &'walk AlphaReplayPlan,
}

fn measure_apply_at_path(
    walk: MeasureReplay<'_, '_>,
    term: &Term,
    path: &[TermPathSegment],
    active_binders: &mut Vec<ActiveBinder>,
    current_path: &mut Vec<TermPathSegment>,
    depth: usize,
) -> SubstitutionCheckResult<usize> {
    validate_replay_depth(walk.input, walk.entry, depth)?;
    if path.is_empty() {
        return measure_substitute_subtree(walk, term, active_binders, current_path, depth);
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
                    let child_index = u32::try_from(argument_index).map_err(|_| {
                        resource_rejection(
                            walk.input.target_vc_fingerprint,
                            substitution_location(walk.entry).with_field_path("target_term"),
                        )
                    })?;
                    current_path.push(TermPathSegment::application_argument(child_index));
                    let measured = measure_apply_at_path(
                        walk,
                        argument,
                        rest,
                        active_binders,
                        current_path,
                        child_depth,
                    );
                    current_path.pop();
                    measured?
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
            let rename = walk.alpha_plan.rename_for_path(current_path);
            let active = ActiveBinder {
                source_variable_id: frame.variable_id,
                target_variable_id: rename
                    .map_or(frame.variable_id, |rename| rename.generated_variable_id),
            };
            active_binders.push(active);
            current_path.push(TermPathSegment::binder_body());
            let child_depth = checked_replay_child_depth(walk.input, walk.entry, depth)?;
            let body_len =
                measure_apply_at_path(walk, body, rest, active_binders, current_path, child_depth);
            current_path.pop();
            active_binders.pop();
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
    active_binders: &mut Vec<ActiveBinder>,
    current_path: &mut Vec<TermPathSegment>,
    depth: usize,
) -> SubstitutionCheckResult<usize> {
    validate_replay_depth(walk.input, walk.entry, depth)?;
    match term {
        Term::Variable(variable) => {
            if active_binders
                .iter()
                .rev()
                .any(|active| active.source_variable_id == *variable)
            {
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
                &active_binders
                    .iter()
                    .map(|active| active.target_variable_id)
                    .collect::<Vec<_>>(),
            )? {
                return Err(invalid_rejection(
                    walk.input.target_vc_fingerprint,
                    substitution_location(walk.entry).with_field_path("payload.actual_term"),
                ));
            }
            measure_existing_term_at_depth(walk.input, walk.entry, &replacement.actual_term, depth)
        }
        Term::Application { symbol, arguments } => {
            let child_depth = checked_replay_child_depth(walk.input, walk.entry, depth)?;
            let mut argument_lens = Vec::with_capacity(arguments.len());
            for (argument_index, argument) in arguments.iter().enumerate() {
                let child_index = u32::try_from(argument_index).map_err(|_| {
                    resource_rejection(
                        walk.input.target_vc_fingerprint,
                        substitution_location(walk.entry).with_field_path("target_term"),
                    )
                })?;
                current_path.push(TermPathSegment::application_argument(child_index));
                let argument_len = measure_substitute_subtree(
                    walk,
                    argument,
                    active_binders,
                    current_path,
                    child_depth,
                );
                current_path.pop();
                argument_lens.push(argument_len?);
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
            let rename = walk.alpha_plan.rename_for_path(current_path);
            let active = ActiveBinder {
                source_variable_id: frame.variable_id,
                target_variable_id: rename
                    .map_or(frame.variable_id, |rename| rename.generated_variable_id),
            };
            active_binders.push(active);
            current_path.push(TermPathSegment::binder_body());
            let child_depth = checked_replay_child_depth(walk.input, walk.entry, depth)?;
            let body_len =
                measure_substitute_subtree(walk, body, active_binders, current_path, child_depth);
            current_path.pop();
            active_binders.pop();
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
    walk: ReplayWalk<'_, '_>,
    term: &Term,
    path: &[TermPathSegment],
    active_binders: &mut Vec<ActiveBinder>,
    current_path: &mut Vec<TermPathSegment>,
) -> SubstitutionCheckResult<Term> {
    if path.is_empty() {
        return substitute_subtree(walk, term, active_binders, current_path);
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
            let Some(argument) = arguments.get(index) else {
                return Err(invalid_rejection(
                    walk.input.target_vc_fingerprint,
                    substitution_location(walk.entry).with_field_path("payload.rewrite_path"),
                ));
            };
            let mut replayed_arguments = arguments.clone();
            current_path.push(TermPathSegment::application_argument(segment.child_index));
            let replayed = apply_at_path(walk, argument, rest, active_binders, current_path);
            current_path.pop();
            replayed_arguments[index] = replayed?;
            Ok(Term::Application {
                symbol: *symbol,
                arguments: replayed_arguments,
            })
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
            let rename = walk.alpha_plan.rename_for_path(current_path);
            let active = ActiveBinder {
                source_variable_id: frame.variable_id,
                target_variable_id: rename
                    .map_or(frame.variable_id, |rename| rename.generated_variable_id),
            };
            active_binders.push(active);
            current_path.push(TermPathSegment::binder_body());
            let replayed = apply_at_path(walk, body, rest, active_binders, current_path);
            current_path.pop();
            active_binders.pop();
            Ok(Term::BinderNormalized {
                binder_id: rename.map_or(*binder_id, |rename| rename.target_binder_id),
                body: Box::new(replayed?),
            })
        }
        _ => Err(invalid_rejection(
            walk.input.target_vc_fingerprint,
            substitution_location(walk.entry).with_field_path("payload.rewrite_path"),
        )),
    }
}

fn substitute_subtree(
    walk: ReplayWalk<'_, '_>,
    term: &Term,
    active_binders: &mut Vec<ActiveBinder>,
    current_path: &mut Vec<TermPathSegment>,
) -> SubstitutionCheckResult<Term> {
    match term {
        Term::Variable(variable) => {
            if let Some(active) = active_binders
                .iter()
                .rev()
                .find(|active| active.source_variable_id == *variable)
            {
                return Ok(Term::Variable(active.target_variable_id));
            }
            let Some(replacement) = walk
                .payload
                .replacements
                .iter()
                .find(|replacement| replacement.formal_variable_id == *variable)
            else {
                return Ok(term.clone());
            };
            if actual_term_contains_any_active_bound_variable(
                walk.input,
                walk.binder_context,
                walk.entry,
                &replacement.actual_term,
                &active_binders
                    .iter()
                    .map(|active| active.target_variable_id)
                    .collect::<Vec<_>>(),
            )? {
                return Err(invalid_rejection(
                    walk.input.target_vc_fingerprint,
                    substitution_location(walk.entry).with_field_path("payload.actual_term"),
                ));
            }
            Ok(replacement.actual_term.clone())
        }
        Term::Application { symbol, arguments } => {
            let mut replayed_arguments = Vec::with_capacity(arguments.len());
            for (argument_index, argument) in arguments.iter().enumerate() {
                let child_index = u32::try_from(argument_index).map_err(|_| {
                    resource_rejection(
                        walk.input.target_vc_fingerprint,
                        substitution_location(walk.entry).with_field_path("target_term"),
                    )
                })?;
                current_path.push(TermPathSegment::application_argument(child_index));
                let replayed = substitute_subtree(walk, argument, active_binders, current_path);
                current_path.pop();
                replayed_arguments.push(replayed?);
            }
            Ok(Term::Application {
                symbol: *symbol,
                arguments: replayed_arguments,
            })
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
            let rename = walk.alpha_plan.rename_for_path(current_path);
            let active = ActiveBinder {
                source_variable_id: frame.variable_id,
                target_variable_id: rename
                    .map_or(frame.variable_id, |rename| rename.generated_variable_id),
            };
            active_binders.push(active);
            current_path.push(TermPathSegment::binder_body());
            let replayed = substitute_subtree(walk, body, active_binders, current_path);
            current_path.pop();
            active_binders.pop();
            Ok(Term::BinderNormalized {
                binder_id: rename.map_or(*binder_id, |rename| rename.target_binder_id),
                body: Box::new(replayed?),
            })
        }
        Term::Malformed => Err(invalid_rejection(
            walk.input.target_vc_fingerprint,
            substitution_location(walk.entry).with_field_path("source_term"),
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

fn term_at_path<'a>(
    input: SubstitutionCheckInput<'_>,
    entry: &SubstitutionEntry,
    root: &'a Term,
    path: &TermPath,
    field_path: &'static str,
) -> SubstitutionCheckResult<&'a Term> {
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
    Ok(term)
}

fn binder_at_path<'a>(
    input: SubstitutionCheckInput<'_>,
    entry: &SubstitutionEntry,
    root: &'a Term,
    path: &TermPath,
    field_path: &'static str,
) -> SubstitutionCheckResult<(u32, &'a Term)> {
    match term_at_path(input, entry, root, path, field_path)? {
        Term::BinderNormalized { binder_id, body } => Ok((*binder_id, body)),
        _ => Err(invalid_rejection(
            input.target_vc_fingerprint,
            substitution_location(entry).with_field_path(field_path),
        )),
    }
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

fn validate_free_variable_constraints(
    input: SubstitutionCheckInput<'_>,
    binder_context: &BinderContext,
    entry: &SubstitutionEntry,
    constraints: &[&FreeVariableConstraint],
) -> SubstitutionCheckResult<()> {
    for constraint in constraints {
        let (capture_set, target_subtree) =
            target_capture_set_at_path(input, binder_context, entry, &constraint.term_path)?;
        if constraint.capture_set != capture_set {
            return Err(invalid_rejection(
                input.target_vc_fingerprint,
                substitution_location(entry)
                    .with_field_path("free_variable_constraint.capture_set"),
            ));
        }
        match constraint.constraint_kind {
            CONSTRAINT_KIND_MUST_REMAIN_FREE_AT_PATH => {
                let mut locally_bound = capture_set.clone();
                if !term_contains_free_variable_with_bound(
                    input,
                    binder_context,
                    entry,
                    target_subtree,
                    constraint.variable_id,
                    &mut locally_bound,
                )? {
                    return Err(invalid_rejection(
                        input.target_vc_fingerprint,
                        substitution_location(entry)
                            .with_field_path("free_variable_constraint.variable_id"),
                    ));
                }
            }
            CONSTRAINT_KIND_MUST_BE_ABSENT_FROM_CAPTURE_SET => {
                if capture_set.contains(&constraint.variable_id) {
                    return Err(invalid_rejection(
                        input.target_vc_fingerprint,
                        substitution_location(entry)
                            .with_field_path("free_variable_constraint.capture_set"),
                    ));
                }
            }
            _ => {
                return Err(invalid_rejection(
                    input.target_vc_fingerprint,
                    substitution_location(entry)
                        .with_field_path("free_variable_constraint.constraint_kind"),
                ));
            }
        }
    }
    Ok(())
}

fn target_capture_set_at_path<'a>(
    input: SubstitutionCheckInput<'_>,
    binder_context: &BinderContext,
    entry: &'a SubstitutionEntry,
    path: &TermPath,
) -> SubstitutionCheckResult<(Vec<VariableId>, &'a Term)> {
    let mut capture_set = Vec::new();
    let mut term = &entry.target_term;
    for segment in &path.segments {
        term = match (segment.edge_kind, term) {
            (EDGE_KIND_APPLICATION_ARGUMENT, Term::Application { arguments, .. }) => {
                let index = usize::try_from(segment.child_index).map_err(|_| {
                    invalid_rejection(
                        input.target_vc_fingerprint,
                        substitution_location(entry)
                            .with_field_path("free_variable_constraint.term_path"),
                    )
                })?;
                arguments.get(index).ok_or_else(|| {
                    invalid_rejection(
                        input.target_vc_fingerprint,
                        substitution_location(entry)
                            .with_field_path("free_variable_constraint.term_path"),
                    )
                })?
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
                capture_set.push(frame.variable_id);
                if capture_set.len() > input.limits.max_capture_set_variables {
                    return Err(resource_rejection(
                        input.target_vc_fingerprint,
                        substitution_location(entry)
                            .with_field_path("free_variable_constraint.capture_set"),
                    ));
                }
                body
            }
            _ => {
                return Err(invalid_rejection(
                    input.target_vc_fingerprint,
                    substitution_location(entry)
                        .with_field_path("free_variable_constraint.term_path"),
                ));
            }
        };
    }
    capture_set.sort();
    Ok((capture_set, term))
}

fn collect_inserted_actual_free_variables_below_binder(
    input: SubstitutionCheckInput<'_>,
    binder_context: &BinderContext,
    entry: &SubstitutionEntry,
    payload: &SubstitutionPayload,
    binder_path: &TermPath,
    avoided: &mut BTreeSet<VariableId>,
) -> SubstitutionCheckResult<()> {
    let subtree = term_at_path(
        input,
        entry,
        &entry.source_term,
        &payload.rewrite_path,
        "payload.rewrite_path",
    )?;
    let mut current_path = payload.rewrite_path.segments.clone();
    let mut active_bound_variables =
        source_active_variables_at_path(input, binder_context, entry, &payload.rewrite_path)?;
    let walk = InsertedActualWalk {
        input,
        binder_context,
        entry,
        payload,
        binder_path,
    };
    collect_inserted_actual_free_variables_in_subtree(
        walk,
        subtree,
        &mut current_path,
        &mut active_bound_variables,
        avoided,
    )
}

#[derive(Clone, Copy)]
struct InsertedActualWalk<'input, 'walk> {
    input: SubstitutionCheckInput<'input>,
    binder_context: &'walk BinderContext,
    entry: &'walk SubstitutionEntry,
    payload: &'walk SubstitutionPayload,
    binder_path: &'walk TermPath,
}

fn collect_inserted_actual_free_variables_in_subtree(
    walk: InsertedActualWalk<'_, '_>,
    term: &Term,
    current_path: &mut Vec<TermPathSegment>,
    active_bound_variables: &mut Vec<VariableId>,
    avoided: &mut BTreeSet<VariableId>,
) -> SubstitutionCheckResult<()> {
    match term {
        Term::Variable(variable) => {
            if active_bound_variables.contains(variable)
                || !is_below_binder_body(walk.binder_path, current_path)
            {
                return Ok(());
            }
            if let Some(replacement) = walk
                .payload
                .replacements
                .iter()
                .find(|replacement| replacement.formal_variable_id == *variable)
            {
                let free_walk = FreeVariableWalk {
                    input: walk.input,
                    binder_context: walk.binder_context,
                    entry: walk.entry,
                    limit: walk.input.limits.max_avoided_variables,
                    field_path: "freshness_witness.avoided_variables",
                };
                collect_free_variables(
                    free_walk,
                    &replacement.actual_term,
                    &mut Vec::new(),
                    avoided,
                )?;
            }
            Ok(())
        }
        Term::Application { arguments, .. } => {
            for (argument_index, argument) in arguments.iter().enumerate() {
                let child_index = u32::try_from(argument_index).map_err(|_| {
                    resource_rejection(
                        walk.input.target_vc_fingerprint,
                        substitution_location(walk.entry).with_field_path("target_term"),
                    )
                })?;
                current_path.push(TermPathSegment::application_argument(child_index));
                let result = collect_inserted_actual_free_variables_in_subtree(
                    walk,
                    argument,
                    current_path,
                    active_bound_variables,
                    avoided,
                );
                current_path.pop();
                result?;
            }
            Ok(())
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
            current_path.push(TermPathSegment::binder_body());
            let result = collect_inserted_actual_free_variables_in_subtree(
                walk,
                body,
                current_path,
                active_bound_variables,
                avoided,
            );
            current_path.pop();
            active_bound_variables.pop();
            result
        }
        Term::Malformed => Ok(()),
    }
}

fn source_active_variables_at_path(
    input: SubstitutionCheckInput<'_>,
    binder_context: &BinderContext,
    entry: &SubstitutionEntry,
    path: &TermPath,
) -> SubstitutionCheckResult<Vec<VariableId>> {
    let mut active = Vec::new();
    let mut term = &entry.source_term;
    for segment in &path.segments {
        term = match (segment.edge_kind, term) {
            (EDGE_KIND_APPLICATION_ARGUMENT, Term::Application { arguments, .. }) => {
                let index = usize::try_from(segment.child_index).map_err(|_| {
                    invalid_rejection(
                        input.target_vc_fingerprint,
                        substitution_location(entry).with_field_path("payload.rewrite_path"),
                    )
                })?;
                arguments.get(index).ok_or_else(|| {
                    invalid_rejection(
                        input.target_vc_fingerprint,
                        substitution_location(entry).with_field_path("payload.rewrite_path"),
                    )
                })?
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
                active.push(frame.variable_id);
                body
            }
            _ => {
                return Err(invalid_rejection(
                    input.target_vc_fingerprint,
                    substitution_location(entry).with_field_path("payload.rewrite_path"),
                ));
            }
        };
    }
    Ok(active)
}

fn is_below_binder_body(binder_path: &TermPath, current_path: &[TermPathSegment]) -> bool {
    let prefix_len = binder_path.segments.len();
    current_path.len() > prefix_len
        && current_path[..prefix_len] == binder_path.segments
        && current_path[prefix_len] == TermPathSegment::binder_body()
}

#[derive(Clone, Copy)]
struct FreeVariableWalk<'input, 'walk> {
    input: SubstitutionCheckInput<'input>,
    binder_context: &'walk BinderContext,
    entry: &'walk SubstitutionEntry,
    limit: usize,
    field_path: &'static str,
}

fn collect_free_variables(
    walk: FreeVariableWalk<'_, '_>,
    term: &Term,
    locally_bound: &mut Vec<VariableId>,
    free_variables: &mut BTreeSet<VariableId>,
) -> SubstitutionCheckResult<()> {
    match term {
        Term::Variable(variable) => {
            if !locally_bound.contains(variable) {
                insert_limited_variable(walk, free_variables, *variable)?;
            }
            Ok(())
        }
        Term::Application { arguments, .. } => {
            for argument in arguments {
                collect_free_variables(walk, argument, locally_bound, free_variables)?;
            }
            Ok(())
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
            locally_bound.push(frame.variable_id);
            let result = collect_free_variables(walk, body, locally_bound, free_variables);
            locally_bound.pop();
            result
        }
        Term::Malformed => Ok(()),
    }
}

fn insert_limited_variable(
    walk: FreeVariableWalk<'_, '_>,
    variables: &mut BTreeSet<VariableId>,
    variable: VariableId,
) -> SubstitutionCheckResult<()> {
    if variables.insert(variable) && variables.len() > walk.limit {
        return Err(resource_rejection(
            walk.input.target_vc_fingerprint,
            substitution_location(walk.entry).with_field_path(walk.field_path),
        ));
    }
    Ok(())
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
mod tests;
