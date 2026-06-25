use crate::{
    certificate_parser::{
        ClauseRef as ParsedClauseRef, ClauseRefNamespace as ParsedClauseRefNamespace,
        FinalGoalNamespace, ParsedCertificate,
    },
    clause::{Clause, ClauseError, ClauseProfile, ClauseValidationContext, Literal, Polarity},
    rejection::{
        ClauseRef as RejectionClauseRef, ClauseRefNamespace as RejectionClauseRefNamespace,
        RejectionCategory, RejectionDetail, RejectionLocation, RejectionRecord,
        TargetVcFingerprint,
    },
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResolutionReplayLimits {
    pub max_checked_steps: usize,
    pub max_parent_literals: usize,
    pub max_resolvent_literals: usize,
    pub max_resolvent_canonical_bytes: usize,
    pub max_term_encoding_bytes: usize,
    pub max_term_recursion_depth: usize,
}

impl Default for ResolutionReplayLimits {
    fn default() -> Self {
        Self {
            max_checked_steps: usize::MAX,
            max_parent_literals: usize::MAX,
            max_resolvent_literals: usize::MAX,
            max_resolvent_canonical_bytes: usize::MAX,
            max_term_encoding_bytes: usize::MAX,
            max_term_recursion_depth: usize::MAX,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ResolutionTraceInput<'a> {
    pub target_vc_fingerprint: &'a TargetVcFingerprint,
    pub certificate: &'a ParsedCertificate,
    pub imported_clause_context: Option<&'a ImportedClauseContext>,
    pub limits: ResolutionReplayLimits,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportedClauseEntry {
    pub imported_fact_id: u32,
    pub clause: Clause,
}

impl ImportedClauseEntry {
    #[must_use]
    pub const fn new(imported_fact_id: u32, clause: Clause) -> Self {
        Self {
            imported_fact_id,
            clause,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportedClauseContext {
    provenance_fingerprint: Option<Vec<u8>>,
    imported_axiom_clauses: Vec<ImportedClauseEntry>,
    imported_theorem_clauses: Vec<ImportedClauseEntry>,
}

impl ImportedClauseContext {
    pub fn new(
        provenance_fingerprint: Option<Vec<u8>>,
        imported_axiom_clauses: Vec<ImportedClauseEntry>,
        imported_theorem_clauses: Vec<ImportedClauseEntry>,
    ) -> Result<Self, ImportedClauseContextError> {
        Ok(Self {
            provenance_fingerprint,
            imported_axiom_clauses: canonical_imported_entries(
                ParsedClauseRefNamespace::ImportedAxiom,
                imported_axiom_clauses,
            )?,
            imported_theorem_clauses: canonical_imported_entries(
                ParsedClauseRefNamespace::ImportedTheorem,
                imported_theorem_clauses,
            )?,
        })
    }

    #[must_use]
    pub fn provenance_fingerprint(&self) -> Option<&[u8]> {
        self.provenance_fingerprint.as_deref()
    }

    #[must_use]
    pub fn imported_axiom_clauses(&self) -> &[ImportedClauseEntry] {
        &self.imported_axiom_clauses
    }

    #[must_use]
    pub fn imported_theorem_clauses(&self) -> &[ImportedClauseEntry] {
        &self.imported_theorem_clauses
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ImportedClauseContextError {
    DuplicateImportedClause {
        namespace: ParsedClauseRefNamespace,
        imported_fact_id: u32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolutionReplayReport<'a> {
    checked_steps: Vec<CheckedResolutionStep>,
    target_vc_fingerprint: &'a TargetVcFingerprint,
    certificate_hash_input: &'a [u8],
}

impl ResolutionReplayReport<'_> {
    #[must_use]
    pub fn checked_steps(&self) -> &[CheckedResolutionStep] {
        &self.checked_steps
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedResolutionStep {
    pub step_id: u32,
    pub generated_clause_id: u32,
    pub clause: Clause,
}

pub type ResolutionReplayResult<T> = Result<T, Box<RejectionRecord>>;

pub fn replay_resolution_trace<'a>(
    input: ResolutionTraceInput<'a>,
) -> ResolutionReplayResult<ResolutionReplayReport<'a>> {
    let contexts = ReplayContexts::new(input.certificate, input.limits);
    if input.certificate.resolution_trace.len() > input.limits.max_checked_steps {
        let location = input
            .certificate
            .resolution_trace
            .first()
            .map_or_else(RejectionLocation::new, |step| {
                RejectionLocation::new().with_resolution_step_id(step.step_id)
            });
        return Err(rejection(
            input.target_vc_fingerprint,
            RejectionDetail::ResourceExhaustion,
            location,
        ));
    }

    let mut checked_steps = Vec::with_capacity(input.certificate.resolution_trace.len());
    for step in &input.certificate.resolution_trace {
        let generated_clause_id = checked_generated_clause_id(input, step)?;
        let parent_a = lookup_parent_clause(input, &contexts, step, step.parent_a, &checked_steps)?;
        let parent_b = lookup_parent_clause(input, &contexts, step, step.parent_b, &checked_steps)?;
        validate_parent_size(input, step, step.parent_a, &parent_a)?;
        validate_parent_size(input, step, step.parent_b, &parent_b)?;

        let pivot = &step.pivot_literal;
        if !parent_a.literals().contains(pivot) {
            return Err(rejection(
                input.target_vc_fingerprint,
                RejectionDetail::InvalidSatProof,
                step_location(step.step_id)
                    .with_clause_ref(to_rejection_clause_ref(step.parent_a))
                    .with_field_path("pivot_literal"),
            ));
        }
        let opposite_pivot = opposite_literal(pivot);
        if !parent_b.literals().contains(&opposite_pivot) {
            return Err(rejection(
                input.target_vc_fingerprint,
                RejectionDetail::InvalidSatProof,
                step_location(step.step_id)
                    .with_clause_ref(to_rejection_clause_ref(step.parent_b))
                    .with_field_path("pivot_literal"),
            ));
        }

        let upper_bound = parent_a
            .literals()
            .len()
            .checked_add(parent_b.literals().len())
            .and_then(|count| count.checked_sub(2))
            .unwrap_or(usize::MAX);
        if upper_bound > input.limits.max_resolvent_literals {
            return Err(resource_rejection(
                input,
                step_location(step.step_id)
                    .with_clause_ref(to_rejection_clause_ref(step.generated_clause)),
            ));
        }

        let raw_resolvent =
            bounded_resolvent(input, step, &parent_a, pivot, &parent_b, &opposite_pivot)?;
        let replayed = Clause::normalize(raw_resolvent, &contexts.resolvent).map_err(|error| {
            clause_error_rejection(
                input,
                error,
                step_location(step.step_id)
                    .with_clause_ref(to_rejection_clause_ref(step.generated_clause)),
                RejectionDetail::InvalidSatProof,
            )
        })?;
        let generated_clause = lookup_generated_clause(input.certificate, generated_clause_id)
            .ok_or_else(|| {
                rejection(
                    input.target_vc_fingerprint,
                    RejectionDetail::InvalidSatProof,
                    step_location(step.step_id)
                        .with_clause_ref(to_rejection_clause_ref(step.generated_clause)),
                )
            })?;
        validate_clause_against_context(input, &replayed, &contexts.resolvent, step, None)?;
        validate_clause_against_context(
            input,
            generated_clause,
            &contexts.resolvent,
            step,
            Some(step.generated_clause),
        )?;
        if replayed != *generated_clause {
            return Err(rejection(
                input.target_vc_fingerprint,
                RejectionDetail::InvalidSatProof,
                step_location(step.step_id)
                    .with_clause_ref(to_rejection_clause_ref(step.generated_clause)),
            ));
        }
        checked_steps.push(CheckedResolutionStep {
            step_id: step.step_id,
            generated_clause_id,
            clause: replayed,
        });
    }

    Ok(ResolutionReplayReport {
        checked_steps,
        target_vc_fingerprint: input.target_vc_fingerprint,
        certificate_hash_input: input.certificate.canonical_hash_input(),
    })
}

pub fn checked_resolution_final_goal<'a>(
    input: ResolutionTraceInput<'_>,
    report: &'a ResolutionReplayReport<'_>,
) -> ResolutionReplayResult<Option<&'a Clause>> {
    if report.target_vc_fingerprint != input.target_vc_fingerprint
        || report.certificate_hash_input != input.certificate.canonical_hash_input()
    {
        return Err(final_goal_rejection(input.target_vc_fingerprint));
    }
    match input.certificate.final_goal.namespace {
        FinalGoalNamespace::DerivedFact => Ok(None),
        FinalGoalNamespace::ResolutionStep => {
            let Some(step) = report
                .checked_steps
                .iter()
                .find(|step| step.step_id == input.certificate.final_goal.id)
            else {
                return Err(final_goal_rejection(input.target_vc_fingerprint));
            };
            require_empty_final_goal(input.target_vc_fingerprint, &step.clause)?;
            Ok(Some(&step.clause))
        }
        FinalGoalNamespace::GeneratedClause => {
            let Some(step) = report
                .checked_steps
                .iter()
                .find(|step| step.generated_clause_id == input.certificate.final_goal.id)
            else {
                return Err(final_goal_rejection(input.target_vc_fingerprint));
            };
            require_empty_final_goal(input.target_vc_fingerprint, &step.clause)?;
            Ok(Some(&step.clause))
        }
    }
}

struct ReplayContexts {
    parent: ClauseValidationContext,
    resolvent: ClauseValidationContext,
}

impl ReplayContexts {
    fn new(certificate: &ParsedCertificate, limits: ResolutionReplayLimits) -> Self {
        Self {
            parent: replay_context(certificate, limits, limits.max_parent_literals),
            resolvent: replay_context(certificate, limits, limits.max_resolvent_literals),
        }
    }
}

fn replay_context(
    certificate: &ParsedCertificate,
    limits: ResolutionReplayLimits,
    max_literals: usize,
) -> ClauseValidationContext {
    let profile = ClauseProfile::new(
        certificate.kernel_profile.clause_schema_version,
        certificate.kernel_profile.clause_encoding_version,
        certificate.kernel_profile.clause_tautology_policy.into(),
    );
    let mut context = ClauseValidationContext::new(profile)
        .with_limits(max_literals, limits.max_term_encoding_bytes)
        .with_max_term_recursion_depth(limits.max_term_recursion_depth);
    for symbol in &certificate.symbol_manifest {
        context = context.with_known_symbol(symbol.symbol);
    }
    for variable in &certificate.variable_manifest {
        context = context.with_canonical_variable(variable.variable_id);
    }
    context
}

fn checked_generated_clause_id(
    input: ResolutionTraceInput<'_>,
    step: &crate::certificate_parser::ResolutionStep,
) -> ResolutionReplayResult<u32> {
    if step.generated_clause.namespace != ParsedClauseRefNamespace::GeneratedClause {
        return Err(rejection(
            input.target_vc_fingerprint,
            RejectionDetail::InvalidSatProof,
            step_location(step.step_id)
                .with_clause_ref(to_rejection_clause_ref(step.generated_clause)),
        ));
    }
    Ok(step.generated_clause.id)
}

fn lookup_parent_clause(
    input: ResolutionTraceInput<'_>,
    contexts: &ReplayContexts,
    step: &crate::certificate_parser::ResolutionStep,
    clause_ref: ParsedClauseRef,
    checked_steps: &[CheckedResolutionStep],
) -> ResolutionReplayResult<Clause> {
    match clause_ref.namespace {
        ParsedClauseRefNamespace::GeneratedClause => {
            let clause =
                lookup_generated_clause(input.certificate, clause_ref.id).ok_or_else(|| {
                    rejection(
                        input.target_vc_fingerprint,
                        RejectionDetail::InvalidSatProof,
                        step_location(step.step_id)
                            .with_clause_ref(to_rejection_clause_ref(clause_ref)),
                    )
                })?;
            validate_clause_against_context(
                input,
                clause,
                &contexts.parent,
                step,
                Some(clause_ref),
            )?;
            Ok(clause.clone())
        }
        ParsedClauseRefNamespace::ResolutionStep => {
            let checked = checked_steps
                .iter()
                .find(|checked| checked.step_id == clause_ref.id)
                .ok_or_else(|| {
                    rejection(
                        input.target_vc_fingerprint,
                        RejectionDetail::InvalidSatProof,
                        step_location(step.step_id)
                            .with_clause_ref(to_rejection_clause_ref(clause_ref)),
                    )
                })?;
            validate_clause_against_context(
                input,
                &checked.clause,
                &contexts.parent,
                step,
                Some(clause_ref),
            )?;
            Ok(checked.clause.clone())
        }
        ParsedClauseRefNamespace::ImportedAxiom | ParsedClauseRefNamespace::ImportedTheorem => {
            lookup_imported_clause(input, contexts, step, clause_ref)
        }
    }
}

fn lookup_generated_clause(certificate: &ParsedCertificate, clause_id: u32) -> Option<&Clause> {
    certificate
        .generated_clauses
        .binary_search_by_key(&clause_id, |clause| clause.clause_id)
        .ok()
        .map(|index| &certificate.generated_clauses[index].clause)
}

fn lookup_imported_clause(
    input: ResolutionTraceInput<'_>,
    contexts: &ReplayContexts,
    step: &crate::certificate_parser::ResolutionStep,
    clause_ref: ParsedClauseRef,
) -> ResolutionReplayResult<Clause> {
    let location = step_location(step.step_id).with_clause_ref(to_rejection_clause_ref(clause_ref));
    let Some(context) = input.imported_clause_context else {
        return Err(rejection(
            input.target_vc_fingerprint,
            RejectionDetail::MissingProvenance,
            location,
        ));
    };
    if context
        .provenance_fingerprint()
        .is_none_or(|fingerprint| fingerprint.is_empty())
    {
        return Err(rejection(
            input.target_vc_fingerprint,
            RejectionDetail::MissingProvenance,
            location,
        ));
    }
    let entries = match clause_ref.namespace {
        ParsedClauseRefNamespace::ImportedAxiom => context.imported_axiom_clauses(),
        ParsedClauseRefNamespace::ImportedTheorem => context.imported_theorem_clauses(),
        ParsedClauseRefNamespace::GeneratedClause | ParsedClauseRefNamespace::ResolutionStep => {
            unreachable!("imported lookup is called only for imported namespaces")
        }
    };
    let entry = entries
        .binary_search_by_key(&clause_ref.id, |entry| entry.imported_fact_id)
        .ok()
        .map(|index| &entries[index])
        .ok_or_else(|| {
            rejection(
                input.target_vc_fingerprint,
                RejectionDetail::MissingProvenance,
                location.clone(),
            )
        })?;
    if entry.clause.profile() != &contexts.parent.profile {
        return Err(rejection(
            input.target_vc_fingerprint,
            RejectionDetail::MissingProvenance,
            location,
        ));
    }
    Clause::validate_canonical_parts(
        entry.clause.form(),
        entry.clause.literals(),
        &contexts.parent,
    )
    .map_err(|error| {
        clause_error_rejection(input, error, location, RejectionDetail::MissingProvenance)
    })?;
    Ok(entry.clause.clone())
}

fn validate_parent_size(
    input: ResolutionTraceInput<'_>,
    step: &crate::certificate_parser::ResolutionStep,
    clause_ref: ParsedClauseRef,
    clause: &Clause,
) -> ResolutionReplayResult<()> {
    if clause.literals().len() > input.limits.max_parent_literals {
        return Err(resource_rejection(
            input,
            step_location(step.step_id).with_clause_ref(to_rejection_clause_ref(clause_ref)),
        ));
    }
    Ok(())
}

fn validate_clause_against_context(
    input: ResolutionTraceInput<'_>,
    clause: &Clause,
    context: &ClauseValidationContext,
    step: &crate::certificate_parser::ResolutionStep,
    clause_ref: Option<ParsedClauseRef>,
) -> ResolutionReplayResult<()> {
    let location = clause_ref.map_or_else(
        || step_location(step.step_id),
        |clause_ref| {
            step_location(step.step_id).with_clause_ref(to_rejection_clause_ref(clause_ref))
        },
    );
    Clause::validate_canonical_parts(clause.form(), clause.literals(), context).map_err(
        |error| clause_error_rejection(input, error, location, RejectionDetail::InvalidSatProof),
    )?;
    Ok(())
}

fn bounded_resolvent(
    input: ResolutionTraceInput<'_>,
    step: &crate::certificate_parser::ResolutionStep,
    parent_a: &Clause,
    pivot: &Literal,
    parent_b: &Clause,
    opposite_pivot: &Literal,
) -> ResolutionReplayResult<Vec<Literal>> {
    let mut raw = Vec::new();
    let mut canonical_bytes = 0usize;
    for literal in parent_a
        .literals()
        .iter()
        .filter(|literal| *literal != pivot)
    {
        push_resolvent_literal(input, step, &mut raw, &mut canonical_bytes, literal)?;
    }
    for literal in parent_b
        .literals()
        .iter()
        .filter(|literal| *literal != opposite_pivot)
    {
        push_resolvent_literal(input, step, &mut raw, &mut canonical_bytes, literal)?;
    }
    Ok(raw)
}

fn push_resolvent_literal(
    input: ResolutionTraceInput<'_>,
    step: &crate::certificate_parser::ResolutionStep,
    raw: &mut Vec<Literal>,
    canonical_bytes: &mut usize,
    literal: &Literal,
) -> ResolutionReplayResult<()> {
    if raw.len().saturating_add(1) > input.limits.max_resolvent_literals {
        return Err(resource_rejection(input, step_location(step.step_id)));
    }
    let literal_len = literal.canonical_len().map_err(|error| {
        clause_error_rejection(
            input,
            error,
            step_location(step.step_id),
            RejectionDetail::ResourceExhaustion,
        )
    })?;
    let Some(total) = canonical_bytes.checked_add(literal_len) else {
        return Err(resource_rejection(input, step_location(step.step_id)));
    };
    if total > input.limits.max_resolvent_canonical_bytes {
        return Err(resource_rejection(input, step_location(step.step_id)));
    }
    *canonical_bytes = total;
    raw.push(literal.clone());
    Ok(())
}

fn opposite_literal(literal: &Literal) -> Literal {
    let polarity = match literal.polarity {
        Polarity::Negative => Polarity::Positive,
        Polarity::Positive => Polarity::Negative,
    };
    Literal::new(polarity, literal.atom.clone())
}

fn require_empty_final_goal(
    target: &TargetVcFingerprint,
    clause: &Clause,
) -> ResolutionReplayResult<()> {
    if clause.form() == crate::clause::ClauseForm::Empty {
        return Ok(());
    }
    Err(final_goal_rejection(target))
}

fn final_goal_rejection(target: &TargetVcFingerprint) -> Box<RejectionRecord> {
    rejection(
        target,
        RejectionDetail::InvalidSatProof,
        RejectionLocation::new().with_final_goal(),
    )
}

fn clause_error_rejection(
    input: ResolutionTraceInput<'_>,
    error: ClauseError,
    location: RejectionLocation,
    non_resource_detail: RejectionDetail,
) -> Box<RejectionRecord> {
    let detail = if is_resource_clause_error(&error) {
        RejectionDetail::ResourceExhaustion
    } else {
        non_resource_detail
    };
    rejection(input.target_vc_fingerprint, detail, location)
}

fn is_resource_clause_error(error: &ClauseError) -> bool {
    matches!(
        error,
        ClauseError::LiteralCountExceeded { .. }
            | ClauseError::TermSizeExceeded { .. }
            | ClauseError::TermRecursionDepthExceeded { .. }
    )
}

fn resource_rejection(
    input: ResolutionTraceInput<'_>,
    location: RejectionLocation,
) -> Box<RejectionRecord> {
    rejection(
        input.target_vc_fingerprint,
        RejectionDetail::ResourceExhaustion,
        location,
    )
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
        .expect("resolution replay uses valid kernel rejection detail mappings"),
    )
}

fn step_location(step_id: u32) -> RejectionLocation {
    RejectionLocation::new().with_resolution_step_id(step_id)
}

fn to_rejection_clause_ref(clause_ref: ParsedClauseRef) -> RejectionClauseRef {
    let namespace = match clause_ref.namespace {
        ParsedClauseRefNamespace::GeneratedClause => RejectionClauseRefNamespace::GeneratedClause,
        ParsedClauseRefNamespace::ResolutionStep => RejectionClauseRefNamespace::ResolutionStep,
        ParsedClauseRefNamespace::ImportedAxiom => RejectionClauseRefNamespace::ImportedAxiom,
        ParsedClauseRefNamespace::ImportedTheorem => RejectionClauseRefNamespace::ImportedTheorem,
    };
    RejectionClauseRef::new(namespace, clause_ref.id)
}

fn canonical_imported_entries(
    namespace: ParsedClauseRefNamespace,
    mut entries: Vec<ImportedClauseEntry>,
) -> Result<Vec<ImportedClauseEntry>, ImportedClauseContextError> {
    entries.sort_by_key(|entry| entry.imported_fact_id);
    for window in entries.windows(2) {
        if window[0].imported_fact_id == window[1].imported_fact_id {
            return Err(ImportedClauseContextError::DuplicateImportedClause {
                namespace,
                imported_fact_id: window[0].imported_fact_id,
            });
        }
    }
    Ok(entries)
}

#[cfg(test)]
mod tests;
