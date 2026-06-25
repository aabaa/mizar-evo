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
mod tests {
    use super::*;
    use crate::{
        certificate_parser::{
            ClauseRefNamespace, ClauseTautologyPolicy, FinalGoalRef, Fingerprint, GeneratedClause,
            KernelProfileRecord, ParsedCertificateTestParts, ResolutionStep, SymbolManifestEntry,
            VariableManifestEntry,
        },
        clause::{Atom, ClauseForm, SymbolKey, SymbolKind, TautologyPolicy, Term, VariableId},
    };

    #[test]
    fn valid_replay_derives_empty_clause_and_reports_checked_steps_only() {
        let target = target();
        let generated = vec![
            generated(1, ordinary(vec![neg_p()])),
            generated(2, ordinary(vec![pos_p()])),
            generated(3, empty_clause()),
        ];
        let steps = vec![step(
            1,
            generated_ref(1),
            generated_ref(2),
            neg_p(),
            generated_ref(3),
        )];
        let certificate = certificate(generated, steps, final_step(1));

        let report = replay_resolution_trace(input(&target, &certificate, None, limits()))
            .expect("valid trace replays");

        assert_eq!(report.checked_steps().len(), 1);
        assert_eq!(report.checked_steps()[0].step_id, 1);
        assert_eq!(report.checked_steps()[0].generated_clause_id, 3);
        assert_eq!(report.checked_steps()[0].clause.form(), ClauseForm::Empty);
        let final_goal =
            checked_resolution_final_goal(input(&target, &certificate, None, limits()), &report)
                .expect("resolution final goal is checked")
                .expect("resolution final goal");
        assert_eq!(final_goal.form(), ClauseForm::Empty);
    }

    #[test]
    fn valid_replay_uses_imported_axiom_theorem_and_previous_step_parents() {
        let target = target();
        let generated = vec![
            generated(1, ordinary(vec![pos_p()])),
            generated(3, ordinary(vec![pos_q()])),
            generated(4, empty_clause()),
        ];
        let steps = vec![
            step(
                1,
                imported_axiom_ref(10),
                generated_ref(1),
                neg_p(),
                generated_ref(3),
            ),
            step(
                2,
                resolution_step_ref(1),
                imported_theorem_ref(20),
                pos_q(),
                generated_ref(4),
            ),
        ];
        let certificate = certificate(generated, steps, final_step(2));
        let context = ImportedClauseContext::new(
            Some(vec![9]),
            vec![ImportedClauseEntry::new(
                10,
                ordinary(vec![neg_p(), pos_q()]),
            )],
            vec![ImportedClauseEntry::new(20, ordinary(vec![neg_q()]))],
        )
        .expect("valid imported context");

        let report =
            replay_resolution_trace(input(&target, &certificate, Some(&context), limits()))
                .expect("valid mixed-parent trace replays");

        assert_eq!(report.checked_steps().len(), 2);
        assert_eq!(report.checked_steps()[0].generated_clause_id, 3);
        assert_eq!(report.checked_steps()[0].clause, ordinary(vec![pos_q()]));
        assert_eq!(report.checked_steps()[1].generated_clause_id, 4);
        assert_eq!(report.checked_steps()[1].clause.form(), ClauseForm::Empty);
    }

    #[test]
    fn rejects_pivot_polarity_and_resolvent_mismatch_with_stable_locations() {
        let target = target();
        let pivot_absent = certificate(
            vec![
                generated(1, ordinary(vec![neg_p()])),
                generated(2, ordinary(vec![pos_p()])),
                generated(3, empty_clause()),
            ],
            vec![step(
                1,
                generated_ref(1),
                generated_ref(2),
                pos_p(),
                generated_ref(3),
            )],
            final_step(1),
        );

        let record =
            replay_resolution_trace(input(&target, &pivot_absent, None, limits())).unwrap_err();

        assert_rejection(
            &record,
            RejectionDetail::InvalidSatProof,
            Some(1),
            Some(to_rejection_clause_ref(generated_ref(1))),
        );
        assert_eq!(record.location().field_path, Some("pivot_literal"));

        let opposite_absent = certificate(
            vec![
                generated(1, ordinary(vec![neg_p()])),
                generated(2, ordinary(vec![neg_p()])),
                generated(3, empty_clause()),
            ],
            vec![step(
                1,
                generated_ref(1),
                generated_ref(2),
                neg_p(),
                generated_ref(3),
            )],
            final_step(1),
        );

        let record =
            replay_resolution_trace(input(&target, &opposite_absent, None, limits())).unwrap_err();

        assert_rejection(
            &record,
            RejectionDetail::InvalidSatProof,
            Some(1),
            Some(to_rejection_clause_ref(generated_ref(2))),
        );
        assert_eq!(record.location().field_path, Some("pivot_literal"));

        let mismatch = certificate(
            vec![
                generated(1, ordinary(vec![neg_p()])),
                generated(2, ordinary(vec![pos_p()])),
                generated(3, ordinary(vec![pos_q()])),
            ],
            vec![step(
                1,
                generated_ref(1),
                generated_ref(2),
                neg_p(),
                generated_ref(3),
            )],
            final_step(1),
        );

        let record =
            replay_resolution_trace(input(&target, &mismatch, None, limits())).unwrap_err();

        assert_rejection(
            &record,
            RejectionDetail::InvalidSatProof,
            Some(1),
            Some(to_rejection_clause_ref(generated_ref(3))),
        );
    }

    #[test]
    fn rejects_swapped_orientation_and_resolvent_mismatch_variants() {
        let target = target();
        let swapped = certificate(
            vec![
                generated(1, ordinary(vec![pos_p()])),
                generated(2, ordinary(vec![neg_p()])),
                generated(3, empty_clause()),
            ],
            vec![step(
                1,
                generated_ref(1),
                generated_ref(2),
                neg_p(),
                generated_ref(3),
            )],
            final_step(1),
        );
        let record = replay_resolution_trace(input(&target, &swapped, None, limits()))
            .expect_err("parent orientation is semantic");
        assert_rejection(
            &record,
            RejectionDetail::InvalidSatProof,
            Some(1),
            Some(to_rejection_clause_ref(generated_ref(1))),
        );

        for (generated_clause, label) in [
            (empty_clause(), "missing literal"),
            (ordinary(vec![neg_q()]), "different polarity"),
            (ordinary(vec![pos_r()]), "different canonical literal bytes"),
            (ordinary(vec![pos_q(), pos_r()]), "extra literal"),
        ] {
            let mismatch = certificate(
                vec![
                    generated(1, ordinary(vec![neg_p(), pos_q()])),
                    generated(2, ordinary(vec![pos_p()])),
                    generated(3, generated_clause),
                ],
                vec![step(
                    1,
                    generated_ref(1),
                    generated_ref(2),
                    neg_p(),
                    generated_ref(3),
                )],
                final_generated(3),
            );
            let record = replay_resolution_trace(input(&target, &mismatch, None, limits()))
                .expect_err(label);
            assert_rejection(
                &record,
                RejectionDetail::InvalidSatProof,
                Some(1),
                Some(to_rejection_clause_ref(generated_ref(3))),
            );
        }
    }

    #[test]
    fn imported_context_is_explicit_sorted_and_provenance_checked() {
        let target = target();
        let certificate = certificate(
            vec![
                generated(1, ordinary(vec![pos_p()])),
                generated(2, empty_clause()),
            ],
            vec![step(
                1,
                imported_axiom_ref(10),
                generated_ref(1),
                neg_p(),
                generated_ref(2),
            )],
            final_step(1),
        );

        let record = replay_resolution_trace(input(&target, &certificate, None, limits()))
            .expect_err("missing context is provenance failure");
        assert_rejection(
            &record,
            RejectionDetail::MissingProvenance,
            Some(1),
            Some(to_rejection_clause_ref(imported_axiom_ref(10))),
        );

        let missing_provenance = ImportedClauseContext::new(
            None,
            vec![ImportedClauseEntry::new(10, ordinary(vec![neg_p()]))],
            Vec::new(),
        )
        .expect("sorted context");
        let record = replay_resolution_trace(input(
            &target,
            &certificate,
            Some(&missing_provenance),
            limits(),
        ))
        .expect_err("missing context provenance is rejected");
        assert_rejection(
            &record,
            RejectionDetail::MissingProvenance,
            Some(1),
            Some(to_rejection_clause_ref(imported_axiom_ref(10))),
        );

        let absent_id = ImportedClauseContext::new(
            Some(vec![9]),
            vec![ImportedClauseEntry::new(11, ordinary(vec![neg_p()]))],
            Vec::new(),
        )
        .expect("sorted context");
        let record =
            replay_resolution_trace(input(&target, &certificate, Some(&absent_id), limits()))
                .expect_err("absent imported id is rejected");
        assert_rejection(
            &record,
            RejectionDetail::MissingProvenance,
            Some(1),
            Some(to_rejection_clause_ref(imported_axiom_ref(10))),
        );

        let duplicate = ImportedClauseContext::new(
            Some(vec![9]),
            vec![
                ImportedClauseEntry::new(10, ordinary(vec![neg_p()])),
                ImportedClauseEntry::new(10, ordinary(vec![neg_p()])),
            ],
            Vec::new(),
        );
        assert_eq!(
            duplicate,
            Err(ImportedClauseContextError::DuplicateImportedClause {
                namespace: ClauseRefNamespace::ImportedAxiom,
                imported_fact_id: 10,
            })
        );
        let theorem_duplicate = ImportedClauseContext::new(
            Some(vec![9]),
            Vec::new(),
            vec![
                ImportedClauseEntry::new(20, ordinary(vec![neg_q()])),
                ImportedClauseEntry::new(20, ordinary(vec![neg_q()])),
            ],
        );
        assert_eq!(
            theorem_duplicate,
            Err(ImportedClauseContextError::DuplicateImportedClause {
                namespace: ClauseRefNamespace::ImportedTheorem,
                imported_fact_id: 20,
            })
        );

        let sorted = ImportedClauseContext::new(
            Some(vec![9]),
            vec![
                ImportedClauseEntry::new(11, ordinary(vec![neg_p()])),
                ImportedClauseEntry::new(10, ordinary(vec![neg_p()])),
            ],
            Vec::new(),
        )
        .expect("constructor canonicalizes order");
        assert_eq!(sorted.imported_axiom_clauses()[0].imported_fact_id, 10);
        assert_eq!(sorted.imported_axiom_clauses()[1].imported_fact_id, 11);

        let with_unused_invalid_extra = ImportedClauseContext::new(
            Some(vec![9]),
            vec![
                ImportedClauseEntry::new(10, ordinary(vec![neg_p()])),
                ImportedClauseEntry::new(99, wrong_profile_clause()),
            ],
            Vec::new(),
        )
        .expect("extra entries are allowed when ids are unique");
        replay_resolution_trace(input(
            &target,
            &certificate,
            Some(&with_unused_invalid_extra),
            limits(),
        ))
        .expect("unused invalid imported entries are not scanned by replay");
    }

    #[test]
    fn imported_context_compatibility_and_depth_are_checked_at_first_use() {
        let target = target();
        let certificate = certificate(
            vec![
                generated(1, ordinary(vec![pos_p()])),
                generated(2, empty_clause()),
            ],
            vec![step(
                1,
                imported_axiom_ref(10),
                generated_ref(1),
                neg_p(),
                generated_ref(2),
            )],
            final_step(1),
        );
        let incompatible = ImportedClauseContext::new(
            Some(vec![9]),
            vec![ImportedClauseEntry::new(
                10,
                ordinary_with_context(
                    vec![literal_with_variable(99)],
                    context_with_variable(99, 8),
                ),
            )],
            Vec::new(),
        )
        .expect("sorted context");

        let record =
            replay_resolution_trace(input(&target, &certificate, Some(&incompatible), limits()))
                .expect_err("variable outside replay context is missing provenance");

        assert_rejection(
            &record,
            RejectionDetail::MissingProvenance,
            Some(1),
            Some(to_rejection_clause_ref(imported_axiom_ref(10))),
        );

        for imported_clause in [
            wrong_profile_clause(),
            unknown_symbol_clause(),
            noncanonical_imported_clause(),
        ] {
            let context = ImportedClauseContext::new(
                Some(vec![9]),
                vec![ImportedClauseEntry::new(10, imported_clause)],
                Vec::new(),
            )
            .expect("sorted context");
            let record =
                replay_resolution_trace(input(&target, &certificate, Some(&context), limits()))
                    .expect_err("incompatible imported clause is missing provenance");
            assert_rejection(
                &record,
                RejectionDetail::MissingProvenance,
                Some(1),
                Some(to_rejection_clause_ref(imported_axiom_ref(10))),
            );
        }

        let deep_context = ImportedClauseContext::new(
            Some(vec![9]),
            vec![ImportedClauseEntry::new(
                10,
                ordinary_with_context(
                    vec![literal_with_term(deep_term(4))],
                    context_with_variable(1, 8),
                ),
            )],
            Vec::new(),
        )
        .expect("sorted context");
        let mut depth_limited = limits();
        depth_limited.max_term_recursion_depth = 1;

        let record = replay_resolution_trace(input(
            &target,
            &certificate,
            Some(&deep_context),
            depth_limited,
        ))
        .expect_err("deep imported terms are resource bounded");

        assert_rejection(
            &record,
            RejectionDetail::ResourceExhaustion,
            Some(1),
            Some(to_rejection_clause_ref(imported_axiom_ref(10))),
        );

        let term_sized_context = ImportedClauseContext::new(
            Some(vec![9]),
            vec![ImportedClauseEntry::new(
                10,
                ordinary(vec![literal_with_variable(1)]),
            )],
            Vec::new(),
        )
        .expect("sorted context");
        let mut term_size_limited = limits();
        term_size_limited.max_term_encoding_bytes = 1;

        let record = replay_resolution_trace(input(
            &target,
            &certificate,
            Some(&term_sized_context),
            term_size_limited,
        ))
        .expect_err("used imported term encoding is resource bounded");

        assert_rejection(
            &record,
            RejectionDetail::ResourceExhaustion,
            Some(1),
            Some(to_rejection_clause_ref(imported_axiom_ref(10))),
        );
    }

    #[test]
    fn replay_resource_limits_fire_before_unbounded_resolvent_collection() {
        let target = target();
        let certificate = certificate(
            vec![
                generated(1, ordinary(vec![neg_p(), pos_q()])),
                generated(2, ordinary(vec![pos_p()])),
                generated(3, ordinary(vec![pos_q()])),
            ],
            vec![step(
                1,
                generated_ref(1),
                generated_ref(2),
                neg_p(),
                generated_ref(3),
            )],
            final_generated(3),
        );
        let mut literal_limited = limits();
        literal_limited.max_resolvent_literals = 0;

        let record = replay_resolution_trace(input(&target, &certificate, None, literal_limited))
            .expect_err("resolvent upper bound is checked before collection");

        assert_rejection(
            &record,
            RejectionDetail::ResourceExhaustion,
            Some(1),
            Some(to_rejection_clause_ref(generated_ref(3))),
        );

        let mut byte_limited = limits();
        byte_limited.max_resolvent_canonical_bytes = 1;
        let record = replay_resolution_trace(input(&target, &certificate, None, byte_limited))
            .expect_err("resolvent byte limit is checked during bounded accumulation");

        assert_rejection(&record, RejectionDetail::ResourceExhaustion, Some(1), None);
    }

    #[test]
    fn replay_step_parent_and_term_encoding_limits_are_resource_exhaustion() {
        let target = target();
        let valid = certificate(
            vec![
                generated(1, ordinary(vec![neg_p()])),
                generated(2, ordinary(vec![pos_p()])),
                generated(3, empty_clause()),
            ],
            vec![step(
                1,
                generated_ref(1),
                generated_ref(2),
                neg_p(),
                generated_ref(3),
            )],
            final_step(1),
        );
        let mut step_limited = limits();
        step_limited.max_checked_steps = 0;
        let record = replay_resolution_trace(input(&target, &valid, None, step_limited))
            .expect_err("step count limit is enforced");
        assert_rejection(&record, RejectionDetail::ResourceExhaustion, Some(1), None);

        let parent_heavy = certificate(
            vec![
                generated(1, ordinary(vec![neg_p(), pos_q()])),
                generated(2, ordinary(vec![pos_p()])),
                generated(3, ordinary(vec![pos_q()])),
            ],
            vec![step(
                1,
                generated_ref(1),
                generated_ref(2),
                neg_p(),
                generated_ref(3),
            )],
            final_generated(3),
        );
        let mut parent_limited = limits();
        parent_limited.max_parent_literals = 1;
        let record = replay_resolution_trace(input(&target, &parent_heavy, None, parent_limited))
            .expect_err("parent literal limit is enforced");
        assert_rejection(
            &record,
            RejectionDetail::ResourceExhaustion,
            Some(1),
            Some(to_rejection_clause_ref(generated_ref(1))),
        );

        let previous_step_parent_heavy = certificate(
            vec![
                generated(1, normalized_ordinary(vec![neg_p(), pos_q(), pos_r()])),
                generated(
                    2,
                    normalized_ordinary(vec![
                        pos_p(),
                        pos_p_with_variable(1),
                        literal_with_term(deep_term(1)),
                    ]),
                ),
                generated(
                    3,
                    normalized_ordinary(vec![
                        pos_q(),
                        pos_r(),
                        pos_p_with_variable(1),
                        literal_with_term(deep_term(1)),
                    ]),
                ),
                generated(4, ordinary(vec![neg_q()])),
                generated(5, empty_clause()),
            ],
            vec![
                step(
                    1,
                    generated_ref(1),
                    generated_ref(2),
                    neg_p(),
                    generated_ref(3),
                ),
                step(
                    2,
                    resolution_step_ref(1),
                    generated_ref(4),
                    pos_q(),
                    generated_ref(5),
                ),
            ],
            final_step(2),
        );
        let mut previous_parent_limited = limits();
        previous_parent_limited.max_parent_literals = 3;
        let record = replay_resolution_trace(input(
            &target,
            &previous_step_parent_heavy,
            None,
            previous_parent_limited,
        ))
        .expect_err("previous-step parent literal limit is enforced before clone");
        assert_rejection(
            &record,
            RejectionDetail::ResourceExhaustion,
            Some(2),
            Some(to_rejection_clause_ref(resolution_step_ref(1))),
        );

        let term_heavy = certificate(
            vec![
                generated(1, ordinary(vec![literal_with_variable(1)])),
                generated(2, ordinary(vec![pos_p()])),
                generated(3, empty_clause()),
            ],
            vec![step(
                1,
                generated_ref(1),
                generated_ref(2),
                neg_p(),
                generated_ref(3),
            )],
            final_step(1),
        );
        let mut term_limited = limits();
        term_limited.max_term_encoding_bytes = 1;
        let record = replay_resolution_trace(input(&target, &term_heavy, None, term_limited))
            .expect_err("term encoding limit is enforced during replay");
        assert_rejection(
            &record,
            RejectionDetail::ResourceExhaustion,
            Some(1),
            Some(to_rejection_clause_ref(generated_ref(1))),
        );
    }

    #[test]
    fn tautology_outcomes_follow_the_active_clause_profile() {
        let target = target();
        let marker_certificate = certificate_with_policy(
            vec![
                generated(
                    1,
                    ordinary_with_context(
                        vec![neg_p(), pos_q()],
                        context_with_policy(TautologyPolicy::Marker, 1, 16),
                    ),
                ),
                generated(
                    2,
                    ordinary_with_context(
                        vec![neg_q(), pos_p()],
                        context_with_policy(TautologyPolicy::Marker, 1, 16),
                    ),
                ),
                generated(3, tautology_marker()),
            ],
            vec![step(
                1,
                generated_ref(1),
                generated_ref(2),
                neg_p(),
                generated_ref(3),
            )],
            final_generated(3),
            ClauseTautologyPolicy::Marker,
        );

        let report = replay_resolution_trace(input(&target, &marker_certificate, None, limits()))
            .expect("marker profile accepts tautology marker replay");

        assert_eq!(
            report.checked_steps()[0].clause.form(),
            ClauseForm::Tautology
        );

        let reject_certificate = certificate(
            vec![
                generated(1, ordinary(vec![neg_p(), pos_q()])),
                generated(2, ordinary(vec![neg_q(), pos_p()])),
                generated(3, ordinary(vec![pos_q()])),
            ],
            vec![step(
                1,
                generated_ref(1),
                generated_ref(2),
                neg_p(),
                generated_ref(3),
            )],
            final_generated(3),
        );

        let record = replay_resolution_trace(input(&target, &reject_certificate, None, limits()))
            .expect_err("reject profile rejects tautological replay outcome");

        assert_rejection(
            &record,
            RejectionDetail::InvalidSatProof,
            Some(1),
            Some(to_rejection_clause_ref(generated_ref(3))),
        );
    }

    #[test]
    fn defensive_replay_rejects_broken_generated_and_step_invariants() {
        let target = target();
        let bad_generated_namespace = certificate(
            vec![
                generated(1, ordinary(vec![neg_p()])),
                generated(2, ordinary(vec![pos_p()])),
                generated(3, empty_clause()),
            ],
            vec![step(
                1,
                generated_ref(1),
                generated_ref(2),
                neg_p(),
                resolution_step_ref(3),
            )],
            final_step(1),
        );
        let record =
            replay_resolution_trace(input(&target, &bad_generated_namespace, None, limits()))
                .expect_err("generated output namespace must be generated_clause");
        assert_rejection(
            &record,
            RejectionDetail::InvalidSatProof,
            Some(1),
            Some(to_rejection_clause_ref(resolution_step_ref(3))),
        );

        let missing_generated_output = certificate(
            vec![
                generated(1, ordinary(vec![neg_p()])),
                generated(2, ordinary(vec![pos_p()])),
            ],
            vec![step(
                1,
                generated_ref(1),
                generated_ref(2),
                neg_p(),
                generated_ref(99),
            )],
            final_step(1),
        );
        let record =
            replay_resolution_trace(input(&target, &missing_generated_output, None, limits()))
                .expect_err("missing generated output is invalid replay");
        assert_rejection(
            &record,
            RejectionDetail::InvalidSatProof,
            Some(1),
            Some(to_rejection_clause_ref(generated_ref(99))),
        );

        let missing_generated_parent = certificate(
            vec![
                generated(1, ordinary(vec![pos_p()])),
                generated(2, empty_clause()),
            ],
            vec![step(
                1,
                generated_ref(99),
                generated_ref(1),
                neg_p(),
                generated_ref(2),
            )],
            final_step(1),
        );
        let record =
            replay_resolution_trace(input(&target, &missing_generated_parent, None, limits()))
                .expect_err("missing generated parent is invalid replay");
        assert_rejection(
            &record,
            RejectionDetail::InvalidSatProof,
            Some(1),
            Some(to_rejection_clause_ref(generated_ref(99))),
        );

        let unchecked_step_parent = certificate(
            vec![
                generated(1, ordinary(vec![pos_p()])),
                generated(2, empty_clause()),
            ],
            vec![step(
                1,
                resolution_step_ref(99),
                generated_ref(1),
                neg_p(),
                generated_ref(2),
            )],
            final_step(1),
        );
        let record =
            replay_resolution_trace(input(&target, &unchecked_step_parent, None, limits()))
                .expect_err("unchecked previous step parent is invalid replay");
        assert_rejection(
            &record,
            RejectionDetail::InvalidSatProof,
            Some(1),
            Some(to_rejection_clause_ref(resolution_step_ref(99))),
        );
    }

    #[test]
    fn final_goal_helper_requires_checked_empty_resolution_outputs() {
        let target = target();
        let produced_empty = certificate(
            vec![
                generated(1, ordinary(vec![neg_p()])),
                generated(2, ordinary(vec![pos_p()])),
                generated(3, empty_clause()),
            ],
            vec![step(
                1,
                generated_ref(1),
                generated_ref(2),
                neg_p(),
                generated_ref(3),
            )],
            final_generated(3),
        );
        let report = replay_resolution_trace(input(&target, &produced_empty, None, limits()))
            .expect("trace replays");
        let final_goal =
            checked_resolution_final_goal(input(&target, &produced_empty, None, limits()), &report)
                .expect("produced generated final goal is checked")
                .expect("generated resolution final goal");
        assert_eq!(final_goal.form(), ClauseForm::Empty);

        let mismatched_same_ids = certificate(
            vec![
                generated(1, ordinary(vec![neg_p()])),
                generated(2, ordinary(vec![pos_p()])),
                generated(3, empty_clause()),
                generated(4, ordinary(vec![pos_q()])),
            ],
            vec![step(
                1,
                generated_ref(1),
                generated_ref(2),
                neg_p(),
                generated_ref(3),
            )],
            final_generated(3),
        );
        let record = checked_resolution_final_goal(
            input(&target, &mismatched_same_ids, None, limits()),
            &report,
        )
        .expect_err("report is bound to its replayed certificate");
        assert_rejection_detail(&record, RejectionDetail::InvalidSatProof);
        assert!(record.location().final_goal);

        let other_target = TargetVcFingerprint::new(1, vec![43]);
        let record = checked_resolution_final_goal(
            input(&other_target, &produced_empty, None, limits()),
            &report,
        )
        .expect_err("report is bound to its replay target");
        assert_eq!(record.category(), RejectionCategory::KernelRejection);
        assert_eq!(record.detail(), RejectionDetail::InvalidSatProof);
        assert_eq!(record.target_vc_fingerprint(), &other_target);
        assert!(record.location().final_goal);

        let not_produced = certificate(
            vec![
                generated(1, ordinary(vec![neg_p()])),
                generated(2, ordinary(vec![pos_p()])),
                generated(3, empty_clause()),
            ],
            vec![step(
                1,
                generated_ref(1),
                generated_ref(2),
                neg_p(),
                generated_ref(3),
            )],
            final_generated(2),
        );
        let report = replay_resolution_trace(input(&target, &not_produced, None, limits()))
            .expect("trace replays");
        let record =
            checked_resolution_final_goal(input(&target, &not_produced, None, limits()), &report)
                .expect_err("generated final goal must be replay-produced");
        assert_rejection_detail(&record, RejectionDetail::InvalidSatProof);
        assert!(record.location().final_goal);

        let non_empty = certificate(
            vec![
                generated(1, ordinary(vec![neg_p(), pos_q()])),
                generated(2, ordinary(vec![pos_p()])),
                generated(3, ordinary(vec![pos_q()])),
            ],
            vec![step(
                1,
                generated_ref(1),
                generated_ref(2),
                neg_p(),
                generated_ref(3),
            )],
            final_generated(3),
        );
        let report = replay_resolution_trace(input(&target, &non_empty, None, limits()))
            .expect("trace replays");
        let record =
            checked_resolution_final_goal(input(&target, &non_empty, None, limits()), &report)
                .expect_err("non-empty final goal is not accepted");
        assert_rejection_detail(&record, RejectionDetail::InvalidSatProof);
        assert!(record.location().final_goal);

        let unchecked_step = certificate(
            vec![
                generated(1, ordinary(vec![neg_p()])),
                generated(2, ordinary(vec![pos_p()])),
                generated(3, empty_clause()),
            ],
            vec![step(
                1,
                generated_ref(1),
                generated_ref(2),
                neg_p(),
                generated_ref(3),
            )],
            final_step(99),
        );
        let report = replay_resolution_trace(input(&target, &unchecked_step, None, limits()))
            .expect("trace replays");
        let record =
            checked_resolution_final_goal(input(&target, &unchecked_step, None, limits()), &report)
                .expect_err("unchecked resolution-step final goal is rejected");
        assert_rejection_detail(&record, RejectionDetail::InvalidSatProof);
        assert!(record.location().final_goal);

        let non_empty_step = certificate(
            vec![
                generated(1, ordinary(vec![neg_p(), pos_q()])),
                generated(2, ordinary(vec![pos_p()])),
                generated(3, ordinary(vec![pos_q()])),
            ],
            vec![step(
                1,
                generated_ref(1),
                generated_ref(2),
                neg_p(),
                generated_ref(3),
            )],
            final_step(1),
        );
        let report = replay_resolution_trace(input(&target, &non_empty_step, None, limits()))
            .expect("trace replays");
        let record =
            checked_resolution_final_goal(input(&target, &non_empty_step, None, limits()), &report)
                .expect_err("non-empty resolution-step final goal is rejected");
        assert_rejection_detail(&record, RejectionDetail::InvalidSatProof);
        assert!(record.location().final_goal);
    }

    #[test]
    fn deterministic_report_uses_trace_order_and_canonicalized_context_order() {
        let target = target();
        let generated = vec![
            generated(1, ordinary(vec![pos_p()])),
            generated(3, ordinary(vec![pos_q()])),
            generated(4, empty_clause()),
        ];
        let steps = vec![
            step(
                1,
                imported_axiom_ref(10),
                generated_ref(1),
                neg_p(),
                generated_ref(3),
            ),
            step(
                2,
                resolution_step_ref(1),
                imported_theorem_ref(20),
                pos_q(),
                generated_ref(4),
            ),
        ];
        let certificate = certificate(generated, steps, final_step(2));
        let context = ImportedClauseContext::new(
            Some(vec![9]),
            vec![
                ImportedClauseEntry::new(11, ordinary(vec![neg_p()])),
                ImportedClauseEntry::new(10, ordinary(vec![neg_p(), pos_q()])),
            ],
            vec![
                ImportedClauseEntry::new(21, ordinary(vec![neg_q()])),
                ImportedClauseEntry::new(20, ordinary(vec![neg_q()])),
            ],
        )
        .expect("context order is canonicalized");

        let report =
            replay_resolution_trace(input(&target, &certificate, Some(&context), limits()))
                .expect("trace replays deterministically");

        assert_eq!(
            report
                .checked_steps()
                .iter()
                .map(|step| (step.step_id, step.generated_clause_id))
                .collect::<Vec<_>>(),
            vec![(1, 3), (2, 4)]
        );
        assert_eq!(context.imported_axiom_clauses()[0].imported_fact_id, 10);
        assert_eq!(context.imported_theorem_clauses()[0].imported_fact_id, 20);
    }

    #[test]
    fn deterministic_rejection_order_uses_stable_locations() {
        let target = target();
        let failing_step_one = certificate(
            vec![
                generated(1, ordinary(vec![pos_p()])),
                generated(2, empty_clause()),
            ],
            vec![step(
                1,
                imported_axiom_ref(10),
                generated_ref(1),
                neg_p(),
                generated_ref(2),
            )],
            final_step(1),
        );
        let failing_step_two = certificate(
            vec![
                generated(1, ordinary(vec![pos_p()])),
                generated(2, empty_clause()),
            ],
            vec![step(
                2,
                imported_axiom_ref(10),
                generated_ref(1),
                neg_p(),
                generated_ref(2),
            )],
            final_step(2),
        );
        let mut records = [
            *replay_resolution_trace(input(&target, &failing_step_two, None, limits()))
                .expect_err("missing context"),
            *replay_resolution_trace(input(&target, &failing_step_one, None, limits()))
                .expect_err("missing context"),
        ];

        records.sort();

        assert_eq!(records[0].location().resolution_step_id, Some(1));
        assert_eq!(records[1].location().resolution_step_id, Some(2));
    }

    #[test]
    fn clause_depth_limit_and_literal_length_helpers_are_clause_owned() {
        let mut context = base_context().with_max_term_recursion_depth(1);
        context = context.with_limits(usize::MAX, usize::MAX);
        let err = Clause::from_canonical_parts(
            ClauseForm::Ordinary,
            vec![literal_with_term(deep_term(3))],
            &context,
        )
        .expect_err("term depth is bounded by clause context");
        assert_eq!(
            err,
            ClauseError::TermRecursionDepthExceeded { max: 1, actual: 2 }
        );
        let literal = pos_p();
        assert_eq!(
            literal.canonical_len().expect("literal length"),
            literal.canonical_bytes().expect("literal bytes").len()
        );
        let nested_literal = literal_with_term(Term::Application {
            symbol: q_symbol(),
            arguments: vec![deep_term(2)],
        });
        assert_eq!(
            nested_literal
                .canonical_len()
                .expect("nested literal length"),
            nested_literal
                .canonical_bytes()
                .expect("nested literal bytes")
                .len()
        );
    }

    fn input<'a>(
        target: &'a TargetVcFingerprint,
        certificate: &'a crate::certificate_parser::ParsedCertificate,
        imported_clause_context: Option<&'a ImportedClauseContext>,
        limits: ResolutionReplayLimits,
    ) -> ResolutionTraceInput<'a> {
        ResolutionTraceInput {
            target_vc_fingerprint: target,
            certificate,
            imported_clause_context,
            limits,
        }
    }

    fn assert_rejection(
        record: &RejectionRecord,
        detail: RejectionDetail,
        step_id: Option<u32>,
        clause_ref: Option<RejectionClauseRef>,
    ) {
        assert_rejection_detail(record, detail);
        assert_eq!(record.location().resolution_step_id, step_id);
        assert_eq!(record.location().clause_ref, clause_ref);
    }

    fn assert_rejection_detail(record: &RejectionRecord, detail: RejectionDetail) {
        assert_eq!(record.category(), RejectionCategory::KernelRejection);
        assert_eq!(record.detail(), detail);
        assert_eq!(record.detail().stable_key(), detail.stable_key());
        assert_eq!(record.target_vc_fingerprint(), &target());
    }

    fn target() -> TargetVcFingerprint {
        TargetVcFingerprint::new(1, vec![42])
    }

    fn limits() -> ResolutionReplayLimits {
        ResolutionReplayLimits {
            max_checked_steps: 16,
            max_parent_literals: 8,
            max_resolvent_literals: 8,
            max_resolvent_canonical_bytes: 4096,
            max_term_encoding_bytes: 4096,
            max_term_recursion_depth: 16,
        }
    }

    fn certificate(
        generated_clauses: Vec<GeneratedClause>,
        resolution_trace: Vec<ResolutionStep>,
        final_goal: FinalGoalRef,
    ) -> crate::certificate_parser::ParsedCertificate {
        certificate_with_policy(
            generated_clauses,
            resolution_trace,
            final_goal,
            ClauseTautologyPolicy::Reject,
        )
    }

    fn certificate_with_policy(
        generated_clauses: Vec<GeneratedClause>,
        resolution_trace: Vec<ResolutionStep>,
        final_goal: FinalGoalRef,
        clause_tautology_policy: ClauseTautologyPolicy,
    ) -> crate::certificate_parser::ParsedCertificate {
        let canonical_hash_input = test_canonical_hash_input(
            &generated_clauses,
            &resolution_trace,
            final_goal,
            clause_tautology_policy,
        );
        crate::certificate_parser::ParsedCertificate::new_for_kernel_tests(
            ParsedCertificateTestParts {
                schema_version: 1,
                encoding_version: 1,
                kernel_profile: KernelProfileRecord::v1(1, clause_tautology_policy),
                target_vc: Fingerprint::new(1, vec![42]),
                symbol_manifest: vec![
                    SymbolManifestEntry { symbol: p_symbol() },
                    SymbolManifestEntry { symbol: q_symbol() },
                    SymbolManifestEntry { symbol: r_symbol() },
                ],
                variable_manifest: vec![VariableManifestEntry {
                    variable_id: VariableId(1),
                }],
                imported_axioms: Vec::new(),
                imported_theorems: Vec::new(),
                generated_clauses,
                substitutions: Vec::new(),
                resolution_trace,
                derived_facts: Vec::new(),
                final_goal,
                canonical_hash_input,
            },
        )
    }

    fn test_canonical_hash_input(
        generated_clauses: &[GeneratedClause],
        resolution_trace: &[ResolutionStep],
        final_goal: FinalGoalRef,
        clause_tautology_policy: ClauseTautologyPolicy,
    ) -> Vec<u8> {
        let mut bytes = vec![clause_tautology_policy.tag()];
        for generated_clause in generated_clauses {
            bytes.extend_from_slice(&generated_clause.clause_id.to_be_bytes());
            bytes.extend(
                generated_clause
                    .clause
                    .canonical_hash_input()
                    .expect("test clause canonical hash input"),
            );
        }
        for step in resolution_trace {
            bytes.extend_from_slice(&step.step_id.to_be_bytes());
            push_clause_ref_bytes(&mut bytes, step.parent_a);
            push_clause_ref_bytes(&mut bytes, step.parent_b);
            bytes.extend(
                step.pivot_literal
                    .canonical_bytes()
                    .expect("test pivot canonical bytes"),
            );
            push_clause_ref_bytes(&mut bytes, step.generated_clause);
        }
        bytes.push(final_goal_namespace_tag(final_goal.namespace));
        bytes.extend_from_slice(&final_goal.id.to_be_bytes());
        bytes
    }

    fn push_clause_ref_bytes(bytes: &mut Vec<u8>, clause_ref: ParsedClauseRef) {
        bytes.push(clause_ref_namespace_tag(clause_ref.namespace));
        bytes.extend_from_slice(&clause_ref.id.to_be_bytes());
    }

    const fn clause_ref_namespace_tag(namespace: ClauseRefNamespace) -> u8 {
        match namespace {
            ClauseRefNamespace::GeneratedClause => 1,
            ClauseRefNamespace::ResolutionStep => 2,
            ClauseRefNamespace::ImportedAxiom => 3,
            ClauseRefNamespace::ImportedTheorem => 4,
        }
    }

    const fn final_goal_namespace_tag(namespace: FinalGoalNamespace) -> u8 {
        match namespace {
            FinalGoalNamespace::GeneratedClause => 1,
            FinalGoalNamespace::ResolutionStep => 2,
            FinalGoalNamespace::DerivedFact => 3,
        }
    }

    fn generated(clause_id: u32, clause: Clause) -> GeneratedClause {
        GeneratedClause { clause_id, clause }
    }

    fn step(
        step_id: u32,
        parent_a: ParsedClauseRef,
        parent_b: ParsedClauseRef,
        pivot_literal: Literal,
        generated_clause: ParsedClauseRef,
    ) -> ResolutionStep {
        ResolutionStep {
            step_id,
            parent_a,
            parent_b,
            pivot_literal,
            generated_clause,
        }
    }

    fn generated_ref(id: u32) -> ParsedClauseRef {
        ParsedClauseRef {
            namespace: ClauseRefNamespace::GeneratedClause,
            id,
        }
    }

    fn imported_axiom_ref(id: u32) -> ParsedClauseRef {
        ParsedClauseRef {
            namespace: ClauseRefNamespace::ImportedAxiom,
            id,
        }
    }

    fn imported_theorem_ref(id: u32) -> ParsedClauseRef {
        ParsedClauseRef {
            namespace: ClauseRefNamespace::ImportedTheorem,
            id,
        }
    }

    fn resolution_step_ref(id: u32) -> ParsedClauseRef {
        ParsedClauseRef {
            namespace: ClauseRefNamespace::ResolutionStep,
            id,
        }
    }

    fn final_step(id: u32) -> FinalGoalRef {
        FinalGoalRef {
            namespace: FinalGoalNamespace::ResolutionStep,
            id,
        }
    }

    fn final_generated(id: u32) -> FinalGoalRef {
        FinalGoalRef {
            namespace: FinalGoalNamespace::GeneratedClause,
            id,
        }
    }

    fn empty_clause() -> Clause {
        Clause::from_canonical_parts(ClauseForm::Empty, Vec::new(), &base_context())
            .expect("empty clause")
    }

    fn ordinary(literals: Vec<Literal>) -> Clause {
        ordinary_with_context(literals, base_context())
    }

    fn normalized_ordinary(literals: Vec<Literal>) -> Clause {
        Clause::normalize(literals, &base_context()).expect("normalized ordinary clause")
    }

    fn tautology_marker() -> Clause {
        Clause::from_canonical_parts(
            ClauseForm::Tautology,
            Vec::new(),
            &context_with_policy(TautologyPolicy::Marker, 1, 16),
        )
        .expect("tautology marker")
    }

    fn ordinary_with_context(literals: Vec<Literal>, context: ClauseValidationContext) -> Clause {
        Clause::from_canonical_parts(ClauseForm::Ordinary, literals, &context)
            .expect("ordinary clause")
    }

    fn wrong_profile_clause() -> Clause {
        let context =
            ClauseValidationContext::new(ClauseProfile::new(1, 2, TautologyPolicy::Reject))
                .with_known_symbol(p_symbol())
                .with_canonical_variable(VariableId(1))
                .with_limits(16, 4096)
                .with_max_term_recursion_depth(16);
        ordinary_with_context(vec![neg_p()], context)
    }

    fn unknown_symbol_clause() -> Clause {
        let symbol = unknown_symbol();
        let context =
            ClauseValidationContext::new(ClauseProfile::new(1, 1, TautologyPolicy::Reject))
                .with_known_symbol(symbol)
                .with_canonical_variable(VariableId(1))
                .with_limits(16, 4096)
                .with_max_term_recursion_depth(16);
        ordinary_with_context(
            vec![Literal::new(
                Polarity::Negative,
                Atom::new(symbol, Vec::new()),
            )],
            context,
        )
    }

    fn noncanonical_imported_clause() -> Clause {
        Clause::new_unchecked_for_kernel_tests(
            ClauseProfile::new(1, 1, TautologyPolicy::Reject),
            ClauseForm::Ordinary,
            vec![pos_q(), neg_p()],
        )
    }

    fn base_context() -> ClauseValidationContext {
        context_with_variable(1, 16)
    }

    fn context_with_variable(variable_id: u32, max_depth: usize) -> ClauseValidationContext {
        context_with_policy(TautologyPolicy::Reject, variable_id, max_depth)
    }

    fn context_with_policy(
        tautology_policy: TautologyPolicy,
        variable_id: u32,
        max_depth: usize,
    ) -> ClauseValidationContext {
        ClauseValidationContext::new(ClauseProfile::new(1, 1, tautology_policy))
            .with_known_symbol(p_symbol())
            .with_known_symbol(q_symbol())
            .with_known_symbol(r_symbol())
            .with_canonical_variable(VariableId(variable_id))
            .with_limits(16, 4096)
            .with_max_term_recursion_depth(max_depth)
    }

    fn neg_p() -> Literal {
        Literal::new(Polarity::Negative, Atom::new(p_symbol(), Vec::new()))
    }

    fn pos_p() -> Literal {
        Literal::new(Polarity::Positive, Atom::new(p_symbol(), Vec::new()))
    }

    fn pos_p_with_variable(variable_id: u32) -> Literal {
        Literal::new(
            Polarity::Positive,
            Atom::new(p_symbol(), vec![Term::Variable(VariableId(variable_id))]),
        )
    }

    fn pos_q() -> Literal {
        Literal::new(Polarity::Positive, Atom::new(q_symbol(), Vec::new()))
    }

    fn neg_q() -> Literal {
        Literal::new(Polarity::Negative, Atom::new(q_symbol(), Vec::new()))
    }

    fn pos_r() -> Literal {
        Literal::new(Polarity::Positive, Atom::new(r_symbol(), Vec::new()))
    }

    fn literal_with_variable(variable_id: u32) -> Literal {
        literal_with_term(Term::Variable(VariableId(variable_id)))
    }

    fn literal_with_term(term: Term) -> Literal {
        Literal::new(Polarity::Negative, Atom::new(p_symbol(), vec![term]))
    }

    fn deep_term(depth: u32) -> Term {
        if depth == 0 {
            return Term::Variable(VariableId(1));
        }
        Term::BinderNormalized {
            binder_id: depth,
            body: Box::new(deep_term(depth - 1)),
        }
    }

    fn p_symbol() -> SymbolKey {
        SymbolKey::new(SymbolKind::Predicate, 1)
    }

    fn q_symbol() -> SymbolKey {
        SymbolKey::new(SymbolKind::Predicate, 2)
    }

    fn r_symbol() -> SymbolKey {
        SymbolKey::new(SymbolKind::Predicate, 3)
    }

    fn unknown_symbol() -> SymbolKey {
        SymbolKey::new(SymbolKind::Predicate, 99)
    }
}
