use std::collections::{BTreeMap, BTreeSet};

use crate::{
    certificate_parser::{Fingerprint, ImportedFactRef, ParsedCertificate, RequiredProofStatus},
    clause::{Clause, ClauseError, ClauseProfile, ClauseValidationContext},
    rejection::{
        RejectionCategory, RejectionDetail, RejectionLocation, RejectionRecord, TargetVcFingerprint,
    },
    resolution_trace::{ImportedClauseContext, ImportedClauseContextError, ImportedClauseEntry},
};

pub const SUPPORTED_NORMALIZED_CLAUSE_FINGERPRINT_ALGORITHM_ID: u8 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ImportedFactCheckLimits {
    pub max_imported_facts: usize,
    pub max_imported_context_entries: usize,
    pub max_imported_clause_literals: usize,
    pub max_imported_clause_canonical_bytes: usize,
    pub max_imported_term_encoding_bytes: usize,
    pub max_imported_term_recursion_depth: usize,
}

impl Default for ImportedFactCheckLimits {
    fn default() -> Self {
        Self {
            max_imported_facts: usize::MAX,
            max_imported_context_entries: usize::MAX,
            max_imported_clause_literals: usize::MAX,
            max_imported_clause_canonical_bytes: usize::MAX,
            max_imported_term_encoding_bytes: usize::MAX,
            max_imported_term_recursion_depth: usize::MAX,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ImportedFactCheckInput<'a> {
    pub target_vc_fingerprint: &'a TargetVcFingerprint,
    pub certificate: &'a ParsedCertificate,
    pub imported_fact_context: Option<&'a ImportedFactContext>,
    pub policy: ImportedFactPolicy,
    pub limits: ImportedFactCheckLimits,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ImportedFactPolicy {
    pub allow_externally_attested: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ImportedFactContextLimits {
    pub max_imported_context_entries: usize,
}

impl Default for ImportedFactContextLimits {
    fn default() -> Self {
        Self {
            max_imported_context_entries: usize::MAX,
        }
    }
}

impl From<ImportedFactCheckLimits> for ImportedFactContextLimits {
    fn from(value: ImportedFactCheckLimits) -> Self {
        Self {
            max_imported_context_entries: value.max_imported_context_entries,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportedFactContext {
    provenance_fingerprint: Option<Vec<u8>>,
    imported_axioms: Vec<ImportedFactEvidence>,
    imported_theorems: Vec<ImportedFactEvidence>,
}

impl ImportedFactContext {
    pub fn new(
        provenance_fingerprint: Option<Vec<u8>>,
        imported_axioms: Vec<ImportedFactEvidence>,
        imported_theorems: Vec<ImportedFactEvidence>,
        limits: ImportedFactContextLimits,
    ) -> Result<Self, ImportedFactContextError> {
        validate_context_entry_count(&imported_axioms, &imported_theorems, limits)?;
        Ok(Self {
            provenance_fingerprint,
            imported_axioms: canonical_imported_evidence(
                ImportedFactNamespace::ImportedAxiom,
                imported_axioms,
            )?,
            imported_theorems: canonical_imported_evidence(
                ImportedFactNamespace::ImportedTheorem,
                imported_theorems,
            )?,
        })
    }

    #[must_use]
    pub fn provenance_fingerprint(&self) -> Option<&[u8]> {
        self.provenance_fingerprint.as_deref()
    }

    #[must_use]
    pub fn imported_axioms(&self) -> &[ImportedFactEvidence] {
        &self.imported_axioms
    }

    #[must_use]
    pub fn imported_theorems(&self) -> &[ImportedFactEvidence] {
        &self.imported_theorems
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ImportedFactContextError {
    ImportedFactCountExceeded {
        max: usize,
        actual: usize,
    },
    DuplicateImportedFact {
        namespace: ImportedFactNamespace,
        imported_fact_id: u32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportedFactEvidence {
    pub imported_fact_id: u32,
    pub package_id: Vec<u8>,
    pub module_path: Vec<u8>,
    pub exported_item_id: Vec<u8>,
    pub statement_fingerprint: Fingerprint,
    pub accepted_proof_status: AcceptedProofStatus,
    pub normalized_clause_fingerprint: Fingerprint,
    pub clause: Clause,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ImportedFactNamespace {
    ImportedAxiom,
    ImportedTheorem,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AcceptedProofStatus {
    KernelVerified,
    DischargedBuiltin,
    ExternallyAttestedPolicyPermitted,
}

impl Ord for AcceptedProofStatus {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.strength_rank().cmp(&other.strength_rank())
    }
}

impl PartialOrd for AcceptedProofStatus {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl AcceptedProofStatus {
    #[must_use]
    pub const fn satisfies(self, required: RequiredProofStatus) -> bool {
        match required {
            RequiredProofStatus::KernelVerified => matches!(self, Self::KernelVerified),
            RequiredProofStatus::DischargedBuiltin => {
                matches!(self, Self::KernelVerified | Self::DischargedBuiltin)
            }
            RequiredProofStatus::ExternallyAttestedPolicyPermitted => true,
        }
    }

    #[must_use]
    pub const fn policy_taint(self) -> bool {
        matches!(self, Self::ExternallyAttestedPolicyPermitted)
    }

    const fn strength_rank(self) -> u8 {
        match self {
            Self::ExternallyAttestedPolicyPermitted => 0,
            Self::DischargedBuiltin => 1,
            Self::KernelVerified => 2,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportedFactCheckReport {
    checked_imports: Vec<CheckedImportedFact>,
    imported_clause_context: ImportedClauseContext,
    policy_taint: bool,
}

impl ImportedFactCheckReport {
    #[must_use]
    pub fn checked_imports(&self) -> &[CheckedImportedFact] {
        &self.checked_imports
    }

    #[must_use]
    pub const fn imported_clause_context(&self) -> &ImportedClauseContext {
        &self.imported_clause_context
    }

    #[must_use]
    pub const fn policy_taint(&self) -> bool {
        self.policy_taint
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedImportedFact {
    pub namespace: ImportedFactNamespace,
    pub imported_fact_id: u32,
    pub statement_fingerprint: Fingerprint,
    pub accepted_proof_status: AcceptedProofStatus,
    pub policy_taint: bool,
}

pub type ImportedFactCheckResult<T> = Result<T, Box<RejectionRecord>>;

pub fn check_imported_facts(
    input: ImportedFactCheckInput<'_>,
) -> ImportedFactCheckResult<ImportedFactCheckReport> {
    let total_imports = total_import_count(input)?;
    if total_imports > input.limits.max_imported_facts {
        return Err(resource_rejection(
            input.target_vc_fingerprint,
            RejectionLocation::new().with_field_path("imported_fact_context.imported_fact_count"),
        ));
    }

    let context = checked_context(input)?;
    let validation_context = imported_clause_validation_context(input.certificate, input.limits);
    let provenance_fingerprint = context
        .provenance_fingerprint()
        .expect("checked_context rejects missing provenance")
        .to_vec();

    let mut checked_imports = Vec::with_capacity(total_imports);
    let mut imported_axiom_clauses = Vec::with_capacity(input.certificate.imported_axioms.len());
    let mut imported_theorem_clauses =
        Vec::with_capacity(input.certificate.imported_theorems.len());

    for imported in &input.certificate.imported_axioms {
        let checked = check_one_imported_fact(
            input,
            context,
            ImportedFactNamespace::ImportedAxiom,
            imported,
            &validation_context,
        )?;
        imported_axiom_clauses.push(ImportedClauseEntry::new(
            imported.imported_fact_id,
            checked.clause,
        ));
        checked_imports.push(checked.report);
    }

    for imported in &input.certificate.imported_theorems {
        let checked = check_one_imported_fact(
            input,
            context,
            ImportedFactNamespace::ImportedTheorem,
            imported,
            &validation_context,
        )?;
        imported_theorem_clauses.push(ImportedClauseEntry::new(
            imported.imported_fact_id,
            checked.clause,
        ));
        checked_imports.push(checked.report);
    }

    let policy_taint = checked_imports.iter().any(|import| import.policy_taint);
    let imported_clause_context = ImportedClauseContext::new(
        Some(provenance_fingerprint),
        imported_axiom_clauses,
        imported_theorem_clauses,
    )
    .map_err(|error| imported_clause_context_rejection(input.target_vc_fingerprint, error))?;

    Ok(ImportedFactCheckReport {
        checked_imports,
        imported_clause_context,
        policy_taint,
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CheckedImportWithClause {
    report: CheckedImportedFact,
    clause: Clause,
}

fn check_one_imported_fact(
    input: ImportedFactCheckInput<'_>,
    context: &ImportedFactContext,
    namespace: ImportedFactNamespace,
    imported: &ImportedFactRef,
    validation_context: &ClauseValidationContext,
) -> ImportedFactCheckResult<CheckedImportWithClause> {
    let location = imported_location(imported.imported_fact_id);
    let evidence =
        lookup_evidence(context, namespace, imported.imported_fact_id).ok_or_else(|| {
            rejection(
                input.target_vc_fingerprint,
                RejectionDetail::UnresolvedSymbol,
                location.clone(),
            )
        })?;

    validate_import_identity(input, imported, evidence, location.clone())?;
    validate_proof_status(input, imported, evidence, location.clone())?;
    validate_imported_clause(input, imported, evidence, validation_context, location)?;

    Ok(CheckedImportWithClause {
        report: CheckedImportedFact {
            namespace,
            imported_fact_id: imported.imported_fact_id,
            statement_fingerprint: imported.statement_fingerprint.clone(),
            accepted_proof_status: evidence.accepted_proof_status,
            policy_taint: evidence.accepted_proof_status.policy_taint(),
        },
        clause: evidence.clause.clone(),
    })
}

fn validate_import_identity(
    input: ImportedFactCheckInput<'_>,
    imported: &ImportedFactRef,
    evidence: &ImportedFactEvidence,
    location: RejectionLocation,
) -> ImportedFactCheckResult<()> {
    if evidence.imported_fact_id != imported.imported_fact_id {
        return Err(unresolved(
            input,
            location.with_field_path("imported_fact_id"),
        ));
    }
    if evidence.package_id != imported.package_id {
        return Err(unresolved(input, location.with_field_path("package_id")));
    }
    if evidence.module_path != imported.module_path {
        return Err(unresolved(input, location.with_field_path("module_path")));
    }
    if evidence.exported_item_id != imported.exported_item_id {
        return Err(unresolved(
            input,
            location.with_field_path("exported_item_id"),
        ));
    }
    if evidence.statement_fingerprint != imported.statement_fingerprint {
        return Err(unresolved(
            input,
            location.with_field_path("statement_fingerprint"),
        ));
    }
    Ok(())
}

fn validate_proof_status(
    input: ImportedFactCheckInput<'_>,
    imported: &ImportedFactRef,
    evidence: &ImportedFactEvidence,
    location: RejectionLocation,
) -> ImportedFactCheckResult<()> {
    if evidence.accepted_proof_status == AcceptedProofStatus::ExternallyAttestedPolicyPermitted {
        if imported.required_proof_status != RequiredProofStatus::ExternallyAttestedPolicyPermitted
            || !input.policy.allow_externally_attested
        {
            return Err(unresolved(
                input,
                location.with_field_path("required_proof_status"),
            ));
        }
        return Ok(());
    }
    if evidence
        .accepted_proof_status
        .satisfies(imported.required_proof_status)
    {
        return Ok(());
    }
    Err(unresolved(
        input,
        location.with_field_path("required_proof_status"),
    ))
}

fn validate_imported_clause(
    input: ImportedFactCheckInput<'_>,
    imported: &ImportedFactRef,
    evidence: &ImportedFactEvidence,
    validation_context: &ClauseValidationContext,
    location: RejectionLocation,
) -> ImportedFactCheckResult<()> {
    if evidence.clause.profile() != &validation_context.profile {
        return Err(missing_provenance(
            input,
            location.clone().with_field_path("clause.profile"),
        ));
    }
    Clause::validate_canonical_parts(
        evidence.clause.form(),
        evidence.clause.literals(),
        validation_context,
    )
    .map_err(|error| {
        clause_error_rejection(
            input,
            error,
            location.clone().with_field_path("clause"),
            RejectionDetail::MissingProvenance,
        )
    })?;

    if imported.statement_fingerprint.algorithm_id
        != SUPPORTED_NORMALIZED_CLAUSE_FINGERPRINT_ALGORITHM_ID
    {
        return Err(unresolved(
            input,
            location
                .clone()
                .with_field_path("statement_fingerprint.algorithm_id"),
        ));
    }
    if evidence.normalized_clause_fingerprint.algorithm_id
        != SUPPORTED_NORMALIZED_CLAUSE_FINGERPRINT_ALGORITHM_ID
    {
        return Err(unresolved(
            input,
            location
                .clone()
                .with_field_path("normalized_clause_fingerprint.algorithm_id"),
        ));
    }
    let canonical_hash_input_len = evidence
        .clause
        .canonical_hash_input_len_for_kernel()
        .map_err(|error| {
            clause_error_rejection(
                input,
                error,
                location
                    .clone()
                    .with_field_path("normalized_clause_fingerprint"),
                RejectionDetail::MissingProvenance,
            )
        })?;
    if canonical_hash_input_len > input.limits.max_imported_clause_canonical_bytes {
        return Err(resource_rejection(
            input.target_vc_fingerprint,
            location
                .clone()
                .with_field_path("normalized_clause_fingerprint"),
        ));
    }
    let canonical_hash_input = evidence.clause.canonical_hash_input().map_err(|error| {
        clause_error_rejection(
            input,
            error,
            location
                .clone()
                .with_field_path("normalized_clause_fingerprint"),
            RejectionDetail::MissingProvenance,
        )
    })?;

    let recomputed = Fingerprint::new(
        SUPPORTED_NORMALIZED_CLAUSE_FINGERPRINT_ALGORITHM_ID,
        canonical_hash_input,
    );
    if evidence.normalized_clause_fingerprint != recomputed {
        return Err(unresolved(
            input,
            location
                .clone()
                .with_field_path("normalized_clause_fingerprint"),
        ));
    }
    if evidence.normalized_clause_fingerprint != imported.statement_fingerprint {
        return Err(unresolved(
            input,
            location.with_field_path("statement_fingerprint"),
        ));
    }

    Ok(())
}

fn lookup_evidence(
    context: &ImportedFactContext,
    namespace: ImportedFactNamespace,
    imported_fact_id: u32,
) -> Option<&ImportedFactEvidence> {
    let entries = match namespace {
        ImportedFactNamespace::ImportedAxiom => context.imported_axioms(),
        ImportedFactNamespace::ImportedTheorem => context.imported_theorems(),
    };
    entries
        .binary_search_by_key(&imported_fact_id, |entry| entry.imported_fact_id)
        .ok()
        .map(|index| &entries[index])
}

fn checked_context<'a>(
    input: ImportedFactCheckInput<'a>,
) -> ImportedFactCheckResult<&'a ImportedFactContext> {
    let location = RejectionLocation::new().with_field_path("imported_fact_context");
    let Some(context) = input.imported_fact_context else {
        return Err(missing_provenance(input, location));
    };
    if context
        .provenance_fingerprint()
        .is_none_or(|fingerprint| fingerprint.is_empty())
    {
        return Err(missing_provenance(input, location));
    }
    Ok(context)
}

fn total_import_count(input: ImportedFactCheckInput<'_>) -> ImportedFactCheckResult<usize> {
    input
        .certificate
        .imported_axioms
        .len()
        .checked_add(input.certificate.imported_theorems.len())
        .ok_or_else(|| {
            resource_rejection(
                input.target_vc_fingerprint,
                RejectionLocation::new()
                    .with_field_path("imported_fact_context.imported_fact_count"),
            )
        })
}

fn imported_clause_validation_context(
    certificate: &ParsedCertificate,
    limits: ImportedFactCheckLimits,
) -> ClauseValidationContext {
    let profile = ClauseProfile::new(
        certificate.kernel_profile.clause_schema_version,
        certificate.kernel_profile.clause_encoding_version,
        certificate.kernel_profile.clause_tautology_policy.into(),
    );
    let mut context = ClauseValidationContext::new(profile)
        .with_limits(
            limits.max_imported_clause_literals,
            limits.max_imported_term_encoding_bytes,
        )
        .with_max_term_recursion_depth(limits.max_imported_term_recursion_depth);
    for symbol in &certificate.symbol_manifest {
        context = context.with_known_symbol(symbol.symbol);
    }
    for variable in &certificate.variable_manifest {
        context = context.with_canonical_variable(variable.variable_id);
    }
    context
}

fn canonical_imported_evidence(
    namespace: ImportedFactNamespace,
    mut evidence: Vec<ImportedFactEvidence>,
) -> Result<Vec<ImportedFactEvidence>, ImportedFactContextError> {
    evidence.sort_by_key(|entry| entry.imported_fact_id);
    for window in evidence.windows(2) {
        if window[0].imported_fact_id == window[1].imported_fact_id {
            return Err(ImportedFactContextError::DuplicateImportedFact {
                namespace,
                imported_fact_id: window[0].imported_fact_id,
            });
        }
    }
    Ok(evidence)
}

fn validate_context_entry_count(
    imported_axioms: &[ImportedFactEvidence],
    imported_theorems: &[ImportedFactEvidence],
    limits: ImportedFactContextLimits,
) -> Result<(), ImportedFactContextError> {
    let actual = imported_axioms
        .len()
        .saturating_add(imported_theorems.len());
    if actual > limits.max_imported_context_entries {
        return Err(ImportedFactContextError::ImportedFactCountExceeded {
            max: limits.max_imported_context_entries,
            actual,
        });
    }
    Ok(())
}

fn imported_clause_context_rejection(
    target: &TargetVcFingerprint,
    error: ImportedClauseContextError,
) -> Box<RejectionRecord> {
    let ImportedClauseContextError::DuplicateImportedClause {
        imported_fact_id, ..
    } = error;
    rejection(
        target,
        RejectionDetail::MissingProvenance,
        imported_location(imported_fact_id),
    )
}

fn clause_error_rejection(
    input: ImportedFactCheckInput<'_>,
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

fn unresolved(
    input: ImportedFactCheckInput<'_>,
    location: RejectionLocation,
) -> Box<RejectionRecord> {
    rejection(
        input.target_vc_fingerprint,
        RejectionDetail::UnresolvedSymbol,
        location,
    )
}

fn missing_provenance(
    input: ImportedFactCheckInput<'_>,
    location: RejectionLocation,
) -> Box<RejectionRecord> {
    rejection(
        input.target_vc_fingerprint,
        RejectionDetail::MissingProvenance,
        location,
    )
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
        .expect("imported fact checker uses valid kernel rejection detail mappings"),
    )
}

fn imported_location(imported_fact_id: u32) -> RejectionLocation {
    RejectionLocation::new().with_imported_fact_id(imported_fact_id)
}

const CLUSTER_FACT_DOMAIN_SEPARATOR: &[u8] = b"MIZAR_KERNEL_CLUSTER_FACT\0";
const REDUCTION_RESULT_DOMAIN_SEPARATOR: &[u8] = b"MIZAR_KERNEL_REDUCTION_RESULT\0";
const REDUCTION_AUDIT_DOMAIN_SEPARATOR: &[u8] = b"MIZAR_KERNEL_REDUCTION_AUDIT\0";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ClusterTraceReplayLimits {
    pub max_cluster_steps: usize,
    pub max_reduction_steps: usize,
    pub max_trace_steps: usize,
    pub max_guard_evidence: usize,
    pub max_reduction_bindings: usize,
    pub max_trace_field_bytes: usize,
    pub max_commitment_bytes: usize,
}

impl Default for ClusterTraceReplayLimits {
    fn default() -> Self {
        Self {
            max_cluster_steps: usize::MAX,
            max_reduction_steps: usize::MAX,
            max_trace_steps: usize::MAX,
            max_guard_evidence: usize::MAX,
            max_reduction_bindings: usize::MAX,
            max_trace_field_bytes: usize::MAX,
            max_commitment_bytes: usize::MAX,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ClusterTraceReplayInput<'a> {
    pub target_vc_fingerprint: &'a TargetVcFingerprint,
    pub checked_fact_context: &'a CheckedFactContext,
    pub cluster_trace_context: Option<&'a ClusterTraceContext>,
    pub requested_trace_steps: &'a [u32],
    pub limits: ClusterTraceReplayLimits,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedFactContext {
    imported_facts: Vec<u32>,
    generated_clauses: Vec<u32>,
}

impl CheckedFactContext {
    pub fn new(
        imported_facts: Vec<u32>,
        generated_clauses: Vec<u32>,
    ) -> Result<Self, ClusterTraceContextError> {
        Ok(Self {
            imported_facts: canonical_ids(BaseFactNamespace::ImportedFact, imported_facts)?,
            generated_clauses: canonical_ids(
                BaseFactNamespace::GeneratedClause,
                generated_clauses,
            )?,
        })
    }

    #[must_use]
    pub fn imported_facts(&self) -> &[u32] {
        &self.imported_facts
    }

    #[must_use]
    pub fn generated_clauses(&self) -> &[u32] {
        &self.generated_clauses
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClusterTraceContext {
    provenance_fingerprint: Option<Vec<u8>>,
    cluster_steps: Vec<ClusterStepEvidence>,
    reduction_steps: Vec<ReductionStepEvidence>,
}

impl ClusterTraceContext {
    pub fn new(
        provenance_fingerprint: Option<Vec<u8>>,
        cluster_steps: Vec<ClusterStepEvidence>,
        reduction_steps: Vec<ReductionStepEvidence>,
        limits: ClusterTraceReplayLimits,
    ) -> Result<Self, ClusterTraceContextError> {
        validate_trace_context_counts(&cluster_steps, &reduction_steps, limits)?;
        let cluster_steps = canonical_cluster_steps(cluster_steps)?;
        let reduction_steps = canonical_reduction_steps(reduction_steps)?;
        reject_cross_namespace_trace_ids(&cluster_steps, &reduction_steps)?;
        Ok(Self {
            provenance_fingerprint,
            cluster_steps,
            reduction_steps,
        })
    }

    #[must_use]
    pub fn provenance_fingerprint(&self) -> Option<&[u8]> {
        self.provenance_fingerprint.as_deref()
    }

    #[must_use]
    pub fn cluster_steps(&self) -> &[ClusterStepEvidence] {
        &self.cluster_steps
    }

    #[must_use]
    pub fn reduction_steps(&self) -> &[ReductionStepEvidence] {
        &self.reduction_steps
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClusterTraceContextError {
    TraceStepCountExceeded {
        max: usize,
        actual: usize,
    },
    DuplicateBaseFact {
        namespace: BaseFactNamespace,
        id: u32,
    },
    DuplicateClusterStep {
        step_id: u32,
    },
    DuplicateReductionStep {
        step_id: u32,
    },
    DuplicateTraceStep {
        step_id: u32,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BaseFactNamespace {
    ImportedFact,
    GeneratedClause,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClusterStepEvidence {
    pub cluster_trace_step_id: u32,
    pub source_type: Vec<u8>,
    pub applied_cluster: Vec<u8>,
    pub generated_attribute: Vec<u8>,
    pub generated_type: Vec<u8>,
    pub dependency: CheckedFactRef,
    pub generated_fact_fingerprint: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReductionStepEvidence {
    pub reduction_step_id: u32,
    pub applied_reduction: Vec<u8>,
    pub rule_fqn: Vec<u8>,
    pub enclosing_term_before: Vec<u8>,
    pub redex_path: Vec<u8>,
    pub source_redex: Vec<u8>,
    pub target_term: Vec<u8>,
    pub substitution: Vec<ReductionBindingEvidence>,
    pub required_guard_ids: Vec<u32>,
    pub discharged_guards: Vec<GuardEvidence>,
    pub rule_view: Vec<u8>,
    pub selection_key: Vec<u8>,
    pub strategy_audit_key: Vec<u8>,
    pub result_fingerprint: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReductionBindingEvidence {
    pub variable: Vec<u8>,
    pub replacement: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GuardEvidence {
    pub guard_id: u32,
    pub source_fact_ref: CheckedFactRef,
    pub checked_dependency_ref: CheckedFactRef,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CheckedFactRef {
    ImportedFact(u32),
    GeneratedClause(u32),
    TraceStep(u32),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClusterTraceReplayReport {
    checked_cluster_steps: Vec<CheckedClusterStep>,
    checked_reduction_steps: Vec<CheckedReductionStep>,
}

impl ClusterTraceReplayReport {
    #[must_use]
    pub fn checked_cluster_steps(&self) -> &[CheckedClusterStep] {
        &self.checked_cluster_steps
    }

    #[must_use]
    pub fn checked_reduction_steps(&self) -> &[CheckedReductionStep] {
        &self.checked_reduction_steps
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedClusterStep {
    pub cluster_trace_step_id: u32,
    pub generated_fact_fingerprint: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedReductionStep {
    pub reduction_step_id: u32,
    pub result_fingerprint: Vec<u8>,
}

pub type ClusterTraceReplayResult<T> = Result<T, Box<RejectionRecord>>;

pub fn replay_cluster_trace(
    input: ClusterTraceReplayInput<'_>,
) -> ClusterTraceReplayResult<ClusterTraceReplayReport> {
    if input.requested_trace_steps.is_empty() {
        return Ok(ClusterTraceReplayReport {
            checked_cluster_steps: Vec::new(),
            checked_reduction_steps: Vec::new(),
        });
    }
    let context = checked_cluster_trace_context(input)?;
    let required_steps = required_trace_steps(input, context)?;
    validate_runtime_trace_counts(input, context, &required_steps)?;

    let mut checked_trace_steps = BTreeSet::new();
    let mut checked_cluster_steps = Vec::new();
    let mut checked_reduction_steps = Vec::new();
    for step_id in required_steps {
        match lookup_trace_step(context, step_id) {
            Some(TraceStepEvidenceRef::Cluster(cluster)) => {
                replay_cluster_step(input, cluster, &checked_trace_steps)?;
                checked_trace_steps.insert(cluster.cluster_trace_step_id);
                checked_cluster_steps.push(CheckedClusterStep {
                    cluster_trace_step_id: cluster.cluster_trace_step_id,
                    generated_fact_fingerprint: cluster.generated_fact_fingerprint.clone(),
                });
            }
            Some(TraceStepEvidenceRef::Reduction(reduction)) => {
                replay_reduction_step(input, reduction, &checked_trace_steps)?;
                checked_trace_steps.insert(reduction.reduction_step_id);
                checked_reduction_steps.push(CheckedReductionStep {
                    reduction_step_id: reduction.reduction_step_id,
                    result_fingerprint: reduction.result_fingerprint.clone(),
                });
            }
            None => {
                return Err(invalid_cluster_trace(
                    input,
                    RejectionLocation::new().with_field_path("requested_trace_steps"),
                ));
            }
        }
    }

    Ok(ClusterTraceReplayReport {
        checked_cluster_steps,
        checked_reduction_steps,
    })
}

fn replay_cluster_step(
    input: ClusterTraceReplayInput<'_>,
    step: &ClusterStepEvidence,
    checked_trace_steps: &BTreeSet<u32>,
) -> ClusterTraceReplayResult<()> {
    let location = RejectionLocation::new().with_cluster_trace_step_id(step.cluster_trace_step_id);
    validate_trace_field(
        input,
        &step.source_type,
        location.clone().with_field_path("source_type"),
    )?;
    validate_trace_field(
        input,
        &step.applied_cluster,
        location.clone().with_field_path("applied_cluster"),
    )?;
    validate_trace_field(
        input,
        &step.generated_attribute,
        location.clone().with_field_path("generated_attribute"),
    )?;
    validate_trace_field(
        input,
        &step.generated_type,
        location.clone().with_field_path("generated_type"),
    )?;
    validate_trace_field(
        input,
        &step.generated_fact_fingerprint,
        location
            .clone()
            .with_field_path("generated_fact_fingerprint"),
    )?;
    require_checked_dependency(
        input,
        step.dependency,
        step.cluster_trace_step_id,
        checked_trace_steps,
        location.clone(),
    )?;
    validate_commitment_budget(
        input,
        expected_cluster_fact_commitment_len(step),
        location
            .clone()
            .with_field_path("generated_fact_fingerprint"),
    )?;
    let expected = expected_cluster_fact_fingerprint(step);
    if step.generated_fact_fingerprint != expected {
        return Err(invalid_cluster_trace(
            input,
            location.with_field_path("generated_fact_fingerprint"),
        ));
    }
    Ok(())
}

fn replay_reduction_step(
    input: ClusterTraceReplayInput<'_>,
    step: &ReductionStepEvidence,
    checked_trace_steps: &BTreeSet<u32>,
) -> ClusterTraceReplayResult<()> {
    let location = RejectionLocation::new().with_reduction_step_id(step.reduction_step_id);
    for (field, value) in [
        ("applied_reduction", &step.applied_reduction),
        ("rule_fqn", &step.rule_fqn),
        ("enclosing_term_before", &step.enclosing_term_before),
        ("redex_path", &step.redex_path),
        ("source_redex", &step.source_redex),
        ("target_term", &step.target_term),
        ("rule_view", &step.rule_view),
        ("selection_key", &step.selection_key),
    ] {
        validate_trace_field(input, value, location.clone().with_field_path(field))?;
    }
    validate_reduction_bindings(input, step)?;
    validate_required_guards(input, step, checked_trace_steps)?;

    validate_trace_field(
        input,
        &step.strategy_audit_key,
        location.clone().with_field_path("strategy_audit_key"),
    )?;
    validate_commitment_budget(
        input,
        expected_strategy_audit_commitment_len(step),
        location.clone().with_field_path("strategy_audit_key"),
    )?;
    let expected_audit = expected_strategy_audit_key(step);
    if step.strategy_audit_key != expected_audit {
        return Err(invalid_cluster_trace(
            input,
            location.clone().with_field_path("strategy_audit_key"),
        ));
    }

    validate_trace_field(
        input,
        &step.result_fingerprint,
        location.clone().with_field_path("result_fingerprint"),
    )?;
    validate_commitment_budget(
        input,
        expected_reduction_result_commitment_len(step),
        location.clone().with_field_path("result_fingerprint"),
    )?;
    let expected_result = expected_reduction_result_fingerprint(step);
    if step.result_fingerprint != expected_result {
        return Err(invalid_cluster_trace(
            input,
            location.with_field_path("result_fingerprint"),
        ));
    }
    Ok(())
}

fn validate_reduction_bindings(
    input: ClusterTraceReplayInput<'_>,
    step: &ReductionStepEvidence,
) -> ClusterTraceReplayResult<()> {
    let location = RejectionLocation::new().with_reduction_step_id(step.reduction_step_id);
    if step.substitution.len() > input.limits.max_reduction_bindings {
        return Err(cluster_resource_rejection(
            input,
            location.clone().with_field_path("substitution"),
        ));
    }
    for binding in &step.substitution {
        validate_trace_field(
            input,
            &binding.variable,
            location.clone().with_field_path("substitution.variable"),
        )?;
        validate_trace_field(
            input,
            &binding.replacement,
            location.clone().with_field_path("substitution.replacement"),
        )?;
    }
    Ok(())
}

fn validate_required_guards(
    input: ClusterTraceReplayInput<'_>,
    step: &ReductionStepEvidence,
    checked_trace_steps: &BTreeSet<u32>,
) -> ClusterTraceReplayResult<()> {
    let location = RejectionLocation::new().with_reduction_step_id(step.reduction_step_id);
    if step.discharged_guards.len() > input.limits.max_guard_evidence {
        return Err(cluster_resource_rejection(
            input,
            location.clone().with_field_path("discharged_guards"),
        ));
    }
    if step.required_guard_ids.len() > input.limits.max_guard_evidence {
        return Err(cluster_resource_rejection(
            input,
            location.clone().with_field_path("required_guard_ids"),
        ));
    }
    let mut required = step.required_guard_ids.clone();
    required.sort_unstable();
    let mut discharged: Vec<u32> = step
        .discharged_guards
        .iter()
        .map(|guard| guard.guard_id)
        .collect();
    discharged.sort_unstable();
    if required != discharged || has_duplicate_sorted_u32(&discharged) {
        return Err(invalid_cluster_trace(
            input,
            location.clone().with_field_path("discharged_guards"),
        ));
    }
    for guard in &step.discharged_guards {
        require_checked_dependency(
            input,
            guard.source_fact_ref,
            step.reduction_step_id,
            checked_trace_steps,
            location.clone().with_field_path("guard.source_fact_ref"),
        )?;
        require_checked_dependency(
            input,
            guard.checked_dependency_ref,
            step.reduction_step_id,
            checked_trace_steps,
            location
                .clone()
                .with_field_path("guard.checked_dependency_ref"),
        )?;
    }
    Ok(())
}

fn require_checked_dependency(
    input: ClusterTraceReplayInput<'_>,
    dependency: CheckedFactRef,
    current_step_id: u32,
    checked_trace_steps: &BTreeSet<u32>,
    location: RejectionLocation,
) -> ClusterTraceReplayResult<()> {
    let checked = match dependency {
        CheckedFactRef::ImportedFact(id) => input
            .checked_fact_context
            .imported_facts()
            .binary_search(&id)
            .is_ok(),
        CheckedFactRef::GeneratedClause(id) => input
            .checked_fact_context
            .generated_clauses()
            .binary_search(&id)
            .is_ok(),
        CheckedFactRef::TraceStep(id) => id < current_step_id && checked_trace_steps.contains(&id),
    };
    if checked {
        Ok(())
    } else {
        Err(invalid_cluster_trace(input, location))
    }
}

fn checked_cluster_trace_context<'a>(
    input: ClusterTraceReplayInput<'a>,
) -> ClusterTraceReplayResult<&'a ClusterTraceContext> {
    let location = RejectionLocation::new().with_field_path("cluster_trace_context");
    let Some(context) = input.cluster_trace_context else {
        return Err(cluster_missing_provenance(input, location));
    };
    if context
        .provenance_fingerprint()
        .is_none_or(|fingerprint| fingerprint.is_empty())
    {
        return Err(cluster_missing_provenance(input, location));
    }
    Ok(context)
}

fn required_trace_steps(
    input: ClusterTraceReplayInput<'_>,
    context: &ClusterTraceContext,
) -> ClusterTraceReplayResult<BTreeSet<u32>> {
    if input.requested_trace_steps.len() > input.limits.max_trace_steps {
        return Err(cluster_resource_rejection(
            input,
            RejectionLocation::new().with_field_path("requested_trace_steps"),
        ));
    }
    let mut required = BTreeSet::new();
    let mut origins = BTreeMap::new();
    let mut stack = Vec::new();
    for step_id in input.requested_trace_steps {
        if !required.insert(*step_id) {
            return Err(invalid_cluster_trace(
                input,
                RejectionLocation::new().with_field_path("requested_trace_steps"),
            ));
        }
        origins.insert(
            *step_id,
            RejectionLocation::new().with_field_path("requested_trace_steps"),
        );
        stack.push(*step_id);
    }

    while let Some(step_id) = stack.pop() {
        let Some(step) = lookup_trace_step(context, step_id) else {
            return Err(invalid_cluster_trace(
                input,
                origins.get(&step_id).cloned().unwrap_or_else(|| {
                    RejectionLocation::new().with_field_path("requested_trace_steps")
                }),
            ));
        };
        match step {
            TraceStepEvidenceRef::Cluster(cluster) => push_required_trace_dependency(
                input,
                &mut required,
                &mut origins,
                &mut stack,
                cluster.cluster_trace_step_id,
                cluster.dependency,
                RejectionLocation::new()
                    .with_cluster_trace_step_id(cluster.cluster_trace_step_id)
                    .with_field_path("dependency"),
            )?,
            TraceStepEvidenceRef::Reduction(reduction) => {
                if reduction.discharged_guards.len() > input.limits.max_guard_evidence {
                    return Err(cluster_resource_rejection(
                        input,
                        RejectionLocation::new()
                            .with_reduction_step_id(reduction.reduction_step_id)
                            .with_field_path("discharged_guards"),
                    ));
                }
                for guard in &reduction.discharged_guards {
                    push_required_trace_dependency(
                        input,
                        &mut required,
                        &mut origins,
                        &mut stack,
                        reduction.reduction_step_id,
                        guard.source_fact_ref,
                        RejectionLocation::new()
                            .with_reduction_step_id(reduction.reduction_step_id)
                            .with_field_path("guard.source_fact_ref"),
                    )?;
                    push_required_trace_dependency(
                        input,
                        &mut required,
                        &mut origins,
                        &mut stack,
                        reduction.reduction_step_id,
                        guard.checked_dependency_ref,
                        RejectionLocation::new()
                            .with_reduction_step_id(reduction.reduction_step_id)
                            .with_field_path("guard.checked_dependency_ref"),
                    )?;
                }
            }
        }
    }

    if required.len() > input.limits.max_trace_steps {
        return Err(cluster_resource_rejection(
            input,
            RejectionLocation::new().with_field_path("requested_trace_steps"),
        ));
    }
    Ok(required)
}

fn push_required_trace_dependency(
    input: ClusterTraceReplayInput<'_>,
    required: &mut BTreeSet<u32>,
    origins: &mut BTreeMap<u32, RejectionLocation>,
    stack: &mut Vec<u32>,
    current_step_id: u32,
    dependency: CheckedFactRef,
    location: RejectionLocation,
) -> ClusterTraceReplayResult<()> {
    let CheckedFactRef::TraceStep(dependency_id) = dependency else {
        return Ok(());
    };
    if dependency_id >= current_step_id {
        return Err(invalid_cluster_trace(input, location));
    }
    if required.insert(dependency_id) {
        if required.len() > input.limits.max_trace_steps {
            return Err(cluster_resource_rejection(
                input,
                RejectionLocation::new().with_field_path("requested_trace_steps"),
            ));
        }
        origins.insert(dependency_id, location);
        stack.push(dependency_id);
    }
    Ok(())
}

enum TraceStepEvidenceRef<'a> {
    Cluster(&'a ClusterStepEvidence),
    Reduction(&'a ReductionStepEvidence),
}

fn lookup_trace_step(
    context: &ClusterTraceContext,
    step_id: u32,
) -> Option<TraceStepEvidenceRef<'_>> {
    if let Ok(index) = context
        .cluster_steps()
        .binary_search_by_key(&step_id, |step| step.cluster_trace_step_id)
    {
        return Some(TraceStepEvidenceRef::Cluster(
            &context.cluster_steps()[index],
        ));
    }
    context
        .reduction_steps()
        .binary_search_by_key(&step_id, |step| step.reduction_step_id)
        .ok()
        .map(|index| TraceStepEvidenceRef::Reduction(&context.reduction_steps()[index]))
}

fn validate_runtime_trace_counts(
    input: ClusterTraceReplayInput<'_>,
    context: &ClusterTraceContext,
    required_steps: &BTreeSet<u32>,
) -> ClusterTraceReplayResult<()> {
    let mut cluster_steps = 0usize;
    let mut reduction_steps = 0usize;
    for step_id in required_steps {
        match lookup_trace_step(context, *step_id) {
            Some(TraceStepEvidenceRef::Cluster(_)) => {
                cluster_steps = cluster_steps.saturating_add(1);
            }
            Some(TraceStepEvidenceRef::Reduction(_)) => {
                reduction_steps = reduction_steps.saturating_add(1);
            }
            None => {
                return Err(invalid_cluster_trace(
                    input,
                    RejectionLocation::new().with_field_path("requested_trace_steps"),
                ));
            }
        }
    }
    if cluster_steps > input.limits.max_cluster_steps {
        return Err(cluster_resource_rejection(
            input,
            RejectionLocation::new().with_field_path("cluster_trace_context.cluster_steps"),
        ));
    }
    if reduction_steps > input.limits.max_reduction_steps {
        return Err(cluster_resource_rejection(
            input,
            RejectionLocation::new().with_field_path("cluster_trace_context.reduction_steps"),
        ));
    }
    if required_steps.len() > input.limits.max_trace_steps {
        return Err(cluster_resource_rejection(
            input,
            RejectionLocation::new().with_field_path("cluster_trace_context.steps"),
        ));
    }
    Ok(())
}

fn validate_trace_field(
    input: ClusterTraceReplayInput<'_>,
    value: &[u8],
    location: RejectionLocation,
) -> ClusterTraceReplayResult<()> {
    if value.is_empty() {
        return Err(invalid_cluster_trace(input, location));
    }
    if value.len() > u32::MAX as usize {
        return Err(cluster_resource_rejection(input, location));
    }
    if value.len() > input.limits.max_trace_field_bytes {
        return Err(cluster_resource_rejection(input, location));
    }
    Ok(())
}

fn validate_commitment_budget(
    input: ClusterTraceReplayInput<'_>,
    byte_len: Option<usize>,
    location: RejectionLocation,
) -> ClusterTraceReplayResult<()> {
    let Some(byte_len) = byte_len else {
        return Err(cluster_resource_rejection(input, location));
    };
    if byte_len > input.limits.max_commitment_bytes {
        return Err(cluster_resource_rejection(input, location));
    }
    Ok(())
}

fn expected_cluster_fact_fingerprint(step: &ClusterStepEvidence) -> Vec<u8> {
    let mut bytes = Vec::from(CLUSTER_FACT_DOMAIN_SEPARATOR);
    push_bytes_field(&mut bytes, &step.source_type);
    push_bytes_field(&mut bytes, &step.applied_cluster);
    push_bytes_field(&mut bytes, &step.generated_attribute);
    push_bytes_field(&mut bytes, &step.generated_type);
    push_dependency_ref(&mut bytes, step.dependency);
    bytes
}

fn expected_cluster_fact_commitment_len(step: &ClusterStepEvidence) -> Option<usize> {
    let mut len = CLUSTER_FACT_DOMAIN_SEPARATOR.len();
    len = checked_add_field_len(len, &step.source_type)?;
    len = checked_add_field_len(len, &step.applied_cluster)?;
    len = checked_add_field_len(len, &step.generated_attribute)?;
    len = checked_add_field_len(len, &step.generated_type)?;
    len.checked_add(dependency_ref_len())
}

fn expected_reduction_result_fingerprint(step: &ReductionStepEvidence) -> Vec<u8> {
    let mut bytes = Vec::from(REDUCTION_RESULT_DOMAIN_SEPARATOR);
    push_bytes_field(&mut bytes, &step.applied_reduction);
    push_bytes_field(&mut bytes, &step.rule_fqn);
    push_bytes_field(&mut bytes, &step.source_redex);
    push_bytes_field(&mut bytes, &step.target_term);
    for binding in &step.substitution {
        push_bytes_field(&mut bytes, &binding.variable);
        push_bytes_field(&mut bytes, &binding.replacement);
    }
    let mut required_guard_ids = step.required_guard_ids.clone();
    required_guard_ids.sort_unstable();
    for guard_id in required_guard_ids {
        bytes.extend_from_slice(&guard_id.to_be_bytes());
    }
    for guard in &step.discharged_guards {
        bytes.extend_from_slice(&guard.guard_id.to_be_bytes());
        push_dependency_ref(&mut bytes, guard.source_fact_ref);
        push_dependency_ref(&mut bytes, guard.checked_dependency_ref);
    }
    bytes
}

fn expected_reduction_result_commitment_len(step: &ReductionStepEvidence) -> Option<usize> {
    let mut len = REDUCTION_RESULT_DOMAIN_SEPARATOR.len();
    len = checked_add_field_len(len, &step.applied_reduction)?;
    len = checked_add_field_len(len, &step.rule_fqn)?;
    len = checked_add_field_len(len, &step.source_redex)?;
    len = checked_add_field_len(len, &step.target_term)?;
    for binding in &step.substitution {
        len = checked_add_field_len(len, &binding.variable)?;
        len = checked_add_field_len(len, &binding.replacement)?;
    }
    len = len.checked_add(step.required_guard_ids.len().checked_mul(4)?)?;
    for _guard in &step.discharged_guards {
        len = len.checked_add(4)?;
        len = len.checked_add(dependency_ref_len())?;
        len = len.checked_add(dependency_ref_len())?;
    }
    Some(len)
}

fn expected_strategy_audit_key(step: &ReductionStepEvidence) -> Vec<u8> {
    let mut bytes = Vec::from(REDUCTION_AUDIT_DOMAIN_SEPARATOR);
    push_bytes_field(&mut bytes, &step.enclosing_term_before);
    push_bytes_field(&mut bytes, &step.redex_path);
    push_bytes_field(&mut bytes, &step.rule_view);
    push_bytes_field(&mut bytes, &step.selection_key);
    bytes
}

fn expected_strategy_audit_commitment_len(step: &ReductionStepEvidence) -> Option<usize> {
    let mut len = REDUCTION_AUDIT_DOMAIN_SEPARATOR.len();
    len = checked_add_field_len(len, &step.enclosing_term_before)?;
    len = checked_add_field_len(len, &step.redex_path)?;
    len = checked_add_field_len(len, &step.rule_view)?;
    checked_add_field_len(len, &step.selection_key)
}

fn checked_add_field_len(total: usize, value: &[u8]) -> Option<usize> {
    total.checked_add(4)?.checked_add(value.len())
}

const fn dependency_ref_len() -> usize {
    5
}

fn push_bytes_field(bytes: &mut Vec<u8>, value: &[u8]) {
    let len = u32::try_from(value.len()).unwrap_or(u32::MAX);
    bytes.extend_from_slice(&len.to_be_bytes());
    bytes.extend_from_slice(value);
}

fn push_dependency_ref(bytes: &mut Vec<u8>, dependency: CheckedFactRef) {
    match dependency {
        CheckedFactRef::ImportedFact(id) => {
            bytes.push(1);
            bytes.extend_from_slice(&id.to_be_bytes());
        }
        CheckedFactRef::GeneratedClause(id) => {
            bytes.push(2);
            bytes.extend_from_slice(&id.to_be_bytes());
        }
        CheckedFactRef::TraceStep(id) => {
            bytes.push(3);
            bytes.extend_from_slice(&id.to_be_bytes());
        }
    }
}

fn validate_trace_context_counts(
    cluster_steps: &[ClusterStepEvidence],
    reduction_steps: &[ReductionStepEvidence],
    limits: ClusterTraceReplayLimits,
) -> Result<(), ClusterTraceContextError> {
    if cluster_steps.len() > limits.max_cluster_steps {
        return Err(ClusterTraceContextError::TraceStepCountExceeded {
            max: limits.max_cluster_steps,
            actual: cluster_steps.len(),
        });
    }
    if reduction_steps.len() > limits.max_reduction_steps {
        return Err(ClusterTraceContextError::TraceStepCountExceeded {
            max: limits.max_reduction_steps,
            actual: reduction_steps.len(),
        });
    }
    let total = cluster_steps.len().saturating_add(reduction_steps.len());
    if total > limits.max_trace_steps {
        return Err(ClusterTraceContextError::TraceStepCountExceeded {
            max: limits.max_trace_steps,
            actual: total,
        });
    }
    Ok(())
}

fn canonical_cluster_steps(
    mut steps: Vec<ClusterStepEvidence>,
) -> Result<Vec<ClusterStepEvidence>, ClusterTraceContextError> {
    steps.sort_by_key(|step| step.cluster_trace_step_id);
    for window in steps.windows(2) {
        if window[0].cluster_trace_step_id == window[1].cluster_trace_step_id {
            return Err(ClusterTraceContextError::DuplicateClusterStep {
                step_id: window[0].cluster_trace_step_id,
            });
        }
    }
    Ok(steps)
}

fn canonical_reduction_steps(
    mut steps: Vec<ReductionStepEvidence>,
) -> Result<Vec<ReductionStepEvidence>, ClusterTraceContextError> {
    steps.sort_by_key(|step| step.reduction_step_id);
    for window in steps.windows(2) {
        if window[0].reduction_step_id == window[1].reduction_step_id {
            return Err(ClusterTraceContextError::DuplicateReductionStep {
                step_id: window[0].reduction_step_id,
            });
        }
    }
    Ok(steps)
}

fn reject_cross_namespace_trace_ids(
    cluster_steps: &[ClusterStepEvidence],
    reduction_steps: &[ReductionStepEvidence],
) -> Result<(), ClusterTraceContextError> {
    let mut reduction_ids: Vec<u32> = reduction_steps
        .iter()
        .map(|step| step.reduction_step_id)
        .collect();
    reduction_ids.sort_unstable();
    for cluster in cluster_steps {
        if reduction_ids
            .binary_search(&cluster.cluster_trace_step_id)
            .is_ok()
        {
            return Err(ClusterTraceContextError::DuplicateTraceStep {
                step_id: cluster.cluster_trace_step_id,
            });
        }
    }
    Ok(())
}

fn canonical_ids(
    namespace: BaseFactNamespace,
    mut ids: Vec<u32>,
) -> Result<Vec<u32>, ClusterTraceContextError> {
    ids.sort_unstable();
    for window in ids.windows(2) {
        if window[0] == window[1] {
            return Err(ClusterTraceContextError::DuplicateBaseFact {
                namespace,
                id: window[0],
            });
        }
    }
    Ok(ids)
}

fn has_duplicate_sorted_u32(values: &[u32]) -> bool {
    values.windows(2).any(|window| window[0] == window[1])
}

fn invalid_cluster_trace(
    input: ClusterTraceReplayInput<'_>,
    location: RejectionLocation,
) -> Box<RejectionRecord> {
    rejection(
        input.target_vc_fingerprint,
        RejectionDetail::InvalidClusterTrace,
        location,
    )
}

fn cluster_missing_provenance(
    input: ClusterTraceReplayInput<'_>,
    location: RejectionLocation,
) -> Box<RejectionRecord> {
    rejection(
        input.target_vc_fingerprint,
        RejectionDetail::MissingProvenance,
        location,
    )
}

fn cluster_resource_rejection(
    input: ClusterTraceReplayInput<'_>,
    location: RejectionLocation,
) -> Box<RejectionRecord> {
    rejection(
        input.target_vc_fingerprint,
        RejectionDetail::ResourceExhaustion,
        location,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        certificate_parser::{
            ClauseTautologyPolicy, FinalGoalNamespace, FinalGoalRef, KernelProfileRecord,
            ParsedCertificateTestParts, SymbolManifestEntry, VariableManifestEntry,
        },
        clause::{
            Atom, ClauseForm, Literal, Polarity, SymbolId, SymbolKey, SymbolKind, TautologyPolicy,
            Term, VariableId,
        },
    };

    #[test]
    fn valid_cluster_and_reduction_trace_replays_in_trace_order() {
        let target = TargetVcFingerprint::new(1, vec![42]);
        let facts = checked_fact_context();
        let mut cluster = cluster_step(2, CheckedFactRef::ImportedFact(1));
        cluster.generated_fact_fingerprint = expected_cluster_fact_fingerprint(&cluster);
        let mut reduction = reduction_step(4, CheckedFactRef::TraceStep(2));
        reduction.strategy_audit_key = expected_strategy_audit_key(&reduction);
        reduction.result_fingerprint = expected_reduction_result_fingerprint(&reduction);
        let context = ClusterTraceContext::new(
            Some(vec![1]),
            vec![cluster.clone()],
            vec![reduction.clone()],
            cluster_limits(),
        )
        .expect("cluster trace context");

        let report = replay_cluster_trace(cluster_input(&target, &facts, Some(&context)))
            .expect("valid cluster trace");

        assert_eq!(report.checked_cluster_steps().len(), 1);
        assert_eq!(report.checked_cluster_steps()[0].cluster_trace_step_id, 2);
        assert_eq!(report.checked_reduction_steps().len(), 1);
        assert_eq!(report.checked_reduction_steps()[0].reduction_step_id, 4);
    }

    #[test]
    fn cluster_trace_missing_context_or_provenance_is_missing_provenance() {
        let target = TargetVcFingerprint::new(1, vec![42]);
        let facts = checked_fact_context();

        let no_request = replay_cluster_trace(cluster_input_with_requested(
            &target,
            &facts,
            None,
            Vec::new(),
        ))
        .expect("empty request does not require context");
        assert!(no_request.checked_cluster_steps().is_empty());

        let missing = replay_cluster_trace(cluster_input(&target, &facts, None))
            .expect_err("missing cluster trace context");
        assert_eq!(missing.detail(), RejectionDetail::MissingProvenance);
        assert_eq!(missing.location().field_path, Some("cluster_trace_context"));

        let mut cluster = cluster_step(1, CheckedFactRef::ImportedFact(1));
        cluster.generated_fact_fingerprint = expected_cluster_fact_fingerprint(&cluster);
        let context = ClusterTraceContext::new(
            Some(Vec::new()),
            vec![cluster],
            Vec::new(),
            cluster_limits(),
        )
        .expect("context");
        let missing_provenance =
            replay_cluster_trace(cluster_input(&target, &facts, Some(&context)))
                .expect_err("missing provenance");
        assert_eq!(
            missing_provenance.detail(),
            RejectionDetail::MissingProvenance
        );
    }

    #[test]
    fn cluster_trace_rejects_hidden_or_future_dependencies_and_mutated_facts() {
        let target = TargetVcFingerprint::new(1, vec![42]);
        let facts = checked_fact_context();

        let mut future_dependency = cluster_step(2, CheckedFactRef::TraceStep(3));
        future_dependency.generated_fact_fingerprint =
            expected_cluster_fact_fingerprint(&future_dependency);
        let context = ClusterTraceContext::new(
            Some(vec![1]),
            vec![future_dependency],
            Vec::new(),
            cluster_limits(),
        )
        .expect("context");
        let dependency_error = replay_cluster_trace(cluster_input(&target, &facts, Some(&context)))
            .expect_err("future dependency");
        assert_eq!(
            dependency_error.detail(),
            RejectionDetail::InvalidClusterTrace
        );

        let mut missing_transitive = cluster_step(4, CheckedFactRef::TraceStep(2));
        missing_transitive.generated_fact_fingerprint =
            expected_cluster_fact_fingerprint(&missing_transitive);
        let context = ClusterTraceContext::new(
            Some(vec![1]),
            vec![missing_transitive],
            Vec::new(),
            cluster_limits(),
        )
        .expect("context");
        let missing_transitive_error = replay_cluster_trace(cluster_input_with_requested(
            &target,
            &facts,
            Some(&context),
            vec![4],
        ))
        .expect_err("missing transitive dependency");
        assert_eq!(
            missing_transitive_error.detail(),
            RejectionDetail::InvalidClusterTrace
        );
        assert_eq!(
            missing_transitive_error.location().cluster_trace_step_id,
            Some(4)
        );
        assert_eq!(
            missing_transitive_error.location().field_path,
            Some("dependency")
        );

        let mut mutated_fact = cluster_step(2, CheckedFactRef::ImportedFact(1));
        mutated_fact.generated_fact_fingerprint = b"wrong".to_vec();
        let context = ClusterTraceContext::new(
            Some(vec![1]),
            vec![mutated_fact],
            Vec::new(),
            cluster_limits(),
        )
        .expect("context");
        let fact_error = replay_cluster_trace(cluster_input(&target, &facts, Some(&context)))
            .expect_err("mutated generated fact");
        assert_eq!(fact_error.detail(), RejectionDetail::InvalidClusterTrace);
        assert_eq!(
            fact_error.location().field_path,
            Some("generated_fact_fingerprint")
        );
    }

    #[test]
    fn reduction_trace_rejects_guard_strategy_and_result_mismatches() {
        let target = TargetVcFingerprint::new(1, vec![42]);
        let facts = checked_fact_context();

        let mut missing_guard = reduction_step(1, CheckedFactRef::ImportedFact(1));
        missing_guard.required_guard_ids = vec![1, 2];
        missing_guard.strategy_audit_key = expected_strategy_audit_key(&missing_guard);
        missing_guard.result_fingerprint = expected_reduction_result_fingerprint(&missing_guard);
        let context = ClusterTraceContext::new(
            Some(vec![1]),
            Vec::new(),
            vec![missing_guard],
            cluster_limits(),
        )
        .expect("context");
        let guard_error = replay_cluster_trace(cluster_input(&target, &facts, Some(&context)))
            .expect_err("missing guard");
        assert_eq!(guard_error.detail(), RejectionDetail::InvalidClusterTrace);
        assert_eq!(guard_error.location().field_path, Some("discharged_guards"));

        let mut bad_audit = reduction_step(1, CheckedFactRef::ImportedFact(1));
        bad_audit.strategy_audit_key = b"bad-audit".to_vec();
        bad_audit.result_fingerprint = expected_reduction_result_fingerprint(&bad_audit);
        let context =
            ClusterTraceContext::new(Some(vec![1]), Vec::new(), vec![bad_audit], cluster_limits())
                .expect("context");
        let audit_error = replay_cluster_trace(cluster_input(&target, &facts, Some(&context)))
            .expect_err("strategy audit mismatch");
        assert_eq!(
            audit_error.location().field_path,
            Some("strategy_audit_key")
        );

        let mut bad_result = reduction_step(1, CheckedFactRef::ImportedFact(1));
        bad_result.strategy_audit_key = expected_strategy_audit_key(&bad_result);
        bad_result.result_fingerprint = b"bad-result".to_vec();
        let context = ClusterTraceContext::new(
            Some(vec![1]),
            Vec::new(),
            vec![bad_result],
            cluster_limits(),
        )
        .expect("context");
        let result_error = replay_cluster_trace(cluster_input(&target, &facts, Some(&context)))
            .expect_err("result mismatch");
        assert_eq!(
            result_error.location().field_path,
            Some("result_fingerprint")
        );
    }

    #[test]
    fn cluster_trace_context_is_bounded_sorted_and_unique() {
        let mut first = cluster_step(1, CheckedFactRef::ImportedFact(1));
        first.generated_fact_fingerprint = expected_cluster_fact_fingerprint(&first);
        let mut second = cluster_step(3, CheckedFactRef::GeneratedClause(7));
        second.generated_fact_fingerprint = expected_cluster_fact_fingerprint(&second);
        let context = ClusterTraceContext::new(
            Some(vec![1]),
            vec![second.clone(), first.clone()],
            Vec::new(),
            cluster_limits(),
        )
        .expect("context canonicalizes order");
        assert_eq!(
            context
                .cluster_steps()
                .iter()
                .map(|step| step.cluster_trace_step_id)
                .collect::<Vec<_>>(),
            [1, 3]
        );

        let duplicate = ClusterTraceContext::new(
            Some(vec![1]),
            vec![first],
            vec![reduction_step(1, CheckedFactRef::ImportedFact(1))],
            cluster_limits(),
        )
        .expect_err("cross namespace duplicate");
        assert_eq!(
            duplicate,
            ClusterTraceContextError::DuplicateTraceStep { step_id: 1 }
        );

        let over_budget = ClusterTraceContext::new(
            Some(vec![1]),
            vec![second],
            Vec::new(),
            ClusterTraceReplayLimits {
                max_cluster_steps: 0,
                ..cluster_limits()
            },
        )
        .expect_err("cluster count limit");
        assert_eq!(
            over_budget,
            ClusterTraceContextError::TraceStepCountExceeded { max: 0, actual: 1 }
        );
    }

    #[test]
    fn cluster_trace_replays_only_requested_steps_and_dependencies() {
        let target = TargetVcFingerprint::new(1, vec![42]);
        let facts = checked_fact_context();
        let mut requested = cluster_step(1, CheckedFactRef::ImportedFact(1));
        requested.generated_fact_fingerprint = expected_cluster_fact_fingerprint(&requested);
        let mut unused_malformed = cluster_step(3, CheckedFactRef::ImportedFact(1));
        unused_malformed.generated_fact_fingerprint = b"wrong".to_vec();
        let context = ClusterTraceContext::new(
            Some(vec![1]),
            vec![unused_malformed, requested],
            Vec::new(),
            cluster_limits(),
        )
        .expect("context");

        let report = replay_cluster_trace(cluster_input_with_requested(
            &target,
            &facts,
            Some(&context),
            vec![1],
        ))
        .expect("unrequested malformed evidence is ignored");

        assert_eq!(report.checked_cluster_steps().len(), 1);
        assert_eq!(report.checked_cluster_steps()[0].cluster_trace_step_id, 1);
    }

    #[test]
    fn cluster_trace_closes_requested_dependencies_in_global_order() {
        let target = TargetVcFingerprint::new(1, vec![42]);
        let facts = checked_fact_context();
        let mut base_cluster = cluster_step(2, CheckedFactRef::ImportedFact(1));
        base_cluster.generated_fact_fingerprint = expected_cluster_fact_fingerprint(&base_cluster);
        let mut reduction = reduction_step(4, CheckedFactRef::TraceStep(2));
        reduction.strategy_audit_key = expected_strategy_audit_key(&reduction);
        reduction.result_fingerprint = expected_reduction_result_fingerprint(&reduction);
        let mut requested_cluster = cluster_step(6, CheckedFactRef::TraceStep(4));
        requested_cluster.generated_fact_fingerprint =
            expected_cluster_fact_fingerprint(&requested_cluster);
        let context = ClusterTraceContext::new(
            Some(vec![1]),
            vec![requested_cluster, base_cluster],
            vec![reduction],
            cluster_limits(),
        )
        .expect("context");

        let report = replay_cluster_trace(cluster_input_with_requested(
            &target,
            &facts,
            Some(&context),
            vec![6],
        ))
        .expect("transitive dependencies replay before requested id");

        assert_eq!(
            report
                .checked_cluster_steps()
                .iter()
                .map(|step| step.cluster_trace_step_id)
                .collect::<Vec<_>>(),
            [2, 6]
        );
        assert_eq!(
            report
                .checked_reduction_steps()
                .iter()
                .map(|step| step.reduction_step_id)
                .collect::<Vec<_>>(),
            [4]
        );
    }

    #[test]
    fn cluster_trace_rejects_unchecked_base_fact_dependencies() {
        let target = TargetVcFingerprint::new(1, vec![42]);
        let facts = checked_fact_context();

        let mut missing_import = cluster_step(1, CheckedFactRef::ImportedFact(99));
        missing_import.generated_fact_fingerprint =
            expected_cluster_fact_fingerprint(&missing_import);
        let context = ClusterTraceContext::new(
            Some(vec![1]),
            vec![missing_import],
            Vec::new(),
            cluster_limits(),
        )
        .expect("context");
        let import_error = replay_cluster_trace(cluster_input(&target, &facts, Some(&context)))
            .expect_err("unchecked imported fact");
        assert_eq!(import_error.detail(), RejectionDetail::InvalidClusterTrace);
        assert_eq!(import_error.location().cluster_trace_step_id, Some(1));

        let mut missing_generated = cluster_step(2, CheckedFactRef::GeneratedClause(99));
        missing_generated.generated_fact_fingerprint =
            expected_cluster_fact_fingerprint(&missing_generated);
        let context = ClusterTraceContext::new(
            Some(vec![1]),
            vec![missing_generated],
            Vec::new(),
            cluster_limits(),
        )
        .expect("context");
        let generated_error = replay_cluster_trace(cluster_input(&target, &facts, Some(&context)))
            .expect_err("unchecked generated clause");
        assert_eq!(
            generated_error.detail(),
            RejectionDetail::InvalidClusterTrace
        );
        assert_eq!(generated_error.location().cluster_trace_step_id, Some(2));
    }

    #[test]
    fn reduction_guards_match_exactly_and_dependencies_are_checked() {
        let target = TargetVcFingerprint::new(1, vec![42]);
        let facts = checked_fact_context();

        let mut order_insensitive = reduction_step(1, CheckedFactRef::ImportedFact(1));
        order_insensitive.required_guard_ids = vec![2, 1];
        order_insensitive.discharged_guards = vec![
            GuardEvidence {
                guard_id: 1,
                source_fact_ref: CheckedFactRef::ImportedFact(1),
                checked_dependency_ref: CheckedFactRef::GeneratedClause(7),
            },
            GuardEvidence {
                guard_id: 2,
                source_fact_ref: CheckedFactRef::GeneratedClause(7),
                checked_dependency_ref: CheckedFactRef::ImportedFact(1),
            },
        ];
        order_insensitive.strategy_audit_key = expected_strategy_audit_key(&order_insensitive);
        order_insensitive.result_fingerprint =
            expected_reduction_result_fingerprint(&order_insensitive);
        let context = ClusterTraceContext::new(
            Some(vec![1]),
            Vec::new(),
            vec![order_insensitive],
            cluster_limits(),
        )
        .expect("context");
        replay_cluster_trace(cluster_input(&target, &facts, Some(&context)))
            .expect("guard ids match independent of evidence order");

        let mut extra_guard = reduction_step(1, CheckedFactRef::ImportedFact(1));
        extra_guard.discharged_guards.push(GuardEvidence {
            guard_id: 2,
            source_fact_ref: CheckedFactRef::ImportedFact(1),
            checked_dependency_ref: CheckedFactRef::ImportedFact(1),
        });
        extra_guard.strategy_audit_key = expected_strategy_audit_key(&extra_guard);
        extra_guard.result_fingerprint = expected_reduction_result_fingerprint(&extra_guard);
        let context = ClusterTraceContext::new(
            Some(vec![1]),
            Vec::new(),
            vec![extra_guard],
            cluster_limits(),
        )
        .expect("context");
        let extra_error = replay_cluster_trace(cluster_input(&target, &facts, Some(&context)))
            .expect_err("extra guard id");
        assert_eq!(extra_error.location().field_path, Some("discharged_guards"));

        let mut duplicate_guard = reduction_step(1, CheckedFactRef::ImportedFact(1));
        duplicate_guard.required_guard_ids = vec![1, 1];
        duplicate_guard.discharged_guards.push(GuardEvidence {
            guard_id: 1,
            source_fact_ref: CheckedFactRef::ImportedFact(1),
            checked_dependency_ref: CheckedFactRef::ImportedFact(1),
        });
        duplicate_guard.strategy_audit_key = expected_strategy_audit_key(&duplicate_guard);
        duplicate_guard.result_fingerprint =
            expected_reduction_result_fingerprint(&duplicate_guard);
        let context = ClusterTraceContext::new(
            Some(vec![1]),
            Vec::new(),
            vec![duplicate_guard],
            cluster_limits(),
        )
        .expect("context");
        let duplicate_error = replay_cluster_trace(cluster_input(&target, &facts, Some(&context)))
            .expect_err("duplicate guard id");
        assert_eq!(
            duplicate_error.location().field_path,
            Some("discharged_guards")
        );

        let mut bad_source = reduction_step(1, CheckedFactRef::ImportedFact(99));
        bad_source.strategy_audit_key = expected_strategy_audit_key(&bad_source);
        bad_source.result_fingerprint = expected_reduction_result_fingerprint(&bad_source);
        let context = ClusterTraceContext::new(
            Some(vec![1]),
            Vec::new(),
            vec![bad_source],
            cluster_limits(),
        )
        .expect("context");
        let source_error = replay_cluster_trace(cluster_input(&target, &facts, Some(&context)))
            .expect_err("unchecked guard source");
        assert_eq!(
            source_error.location().field_path,
            Some("guard.source_fact_ref")
        );

        let mut bad_checked = reduction_step(1, CheckedFactRef::ImportedFact(1));
        bad_checked.discharged_guards[0].checked_dependency_ref =
            CheckedFactRef::GeneratedClause(99);
        bad_checked.strategy_audit_key = expected_strategy_audit_key(&bad_checked);
        bad_checked.result_fingerprint = expected_reduction_result_fingerprint(&bad_checked);
        let context = ClusterTraceContext::new(
            Some(vec![1]),
            Vec::new(),
            vec![bad_checked],
            cluster_limits(),
        )
        .expect("context");
        let checked_error = replay_cluster_trace(cluster_input(&target, &facts, Some(&context)))
            .expect_err("unchecked guard dependency");
        assert_eq!(
            checked_error.location().field_path,
            Some("guard.checked_dependency_ref")
        );
    }

    #[test]
    fn unused_context_entries_are_ignored_after_bounded_construction() {
        let target = TargetVcFingerprint::new(1, vec![42]);
        let facts = checked_fact_context();
        let mut requested = cluster_step(1, CheckedFactRef::ImportedFact(1));
        requested.generated_fact_fingerprint = expected_cluster_fact_fingerprint(&requested);
        let unused_malformed_reduction = reduction_step(3, CheckedFactRef::ImportedFact(99));
        let context = ClusterTraceContext::new(
            Some(vec![1]),
            vec![requested],
            vec![unused_malformed_reduction],
            cluster_limits(),
        )
        .expect("context");
        let mut input = cluster_input_with_requested(&target, &facts, Some(&context), vec![1]);
        input.limits.max_reduction_steps = 0;

        let report = replay_cluster_trace(input).expect("unused reduction is ignored");

        assert_eq!(report.checked_cluster_steps().len(), 1);
        assert!(report.checked_reduction_steps().is_empty());
    }

    #[test]
    fn cluster_trace_context_rejects_duplicates_and_canonicalizes_reductions() {
        let duplicate_cluster = ClusterTraceContext::new(
            Some(vec![1]),
            vec![
                cluster_step(1, CheckedFactRef::ImportedFact(1)),
                cluster_step(1, CheckedFactRef::GeneratedClause(7)),
            ],
            Vec::new(),
            cluster_limits(),
        )
        .expect_err("duplicate cluster id");
        assert_eq!(
            duplicate_cluster,
            ClusterTraceContextError::DuplicateClusterStep { step_id: 1 }
        );

        let duplicate_reduction = ClusterTraceContext::new(
            Some(vec![1]),
            Vec::new(),
            vec![
                reduction_step(1, CheckedFactRef::ImportedFact(1)),
                reduction_step(1, CheckedFactRef::GeneratedClause(7)),
            ],
            cluster_limits(),
        )
        .expect_err("duplicate reduction id");
        assert_eq!(
            duplicate_reduction,
            ClusterTraceContextError::DuplicateReductionStep { step_id: 1 }
        );

        let base_duplicate = CheckedFactContext::new(vec![1, 1], Vec::new())
            .expect_err("duplicate imported base fact");
        assert_eq!(
            base_duplicate,
            ClusterTraceContextError::DuplicateBaseFact {
                namespace: BaseFactNamespace::ImportedFact,
                id: 1
            }
        );

        let mut first = reduction_step(1, CheckedFactRef::ImportedFact(1));
        first.strategy_audit_key = expected_strategy_audit_key(&first);
        first.result_fingerprint = expected_reduction_result_fingerprint(&first);
        let mut second = reduction_step(3, CheckedFactRef::ImportedFact(1));
        second.strategy_audit_key = expected_strategy_audit_key(&second);
        second.result_fingerprint = expected_reduction_result_fingerprint(&second);
        let context = ClusterTraceContext::new(
            Some(vec![1]),
            Vec::new(),
            vec![second, first],
            cluster_limits(),
        )
        .expect("reduction context canonicalizes order");
        assert_eq!(
            context
                .reduction_steps()
                .iter()
                .map(|step| step.reduction_step_id)
                .collect::<Vec<_>>(),
            [1, 3]
        );
    }

    #[test]
    fn cluster_trace_runtime_limits_are_resource_exhaustion() {
        let target = TargetVcFingerprint::new(1, vec![42]);
        let facts = checked_fact_context();
        let mut cluster = cluster_step(1, CheckedFactRef::ImportedFact(1));
        cluster.generated_fact_fingerprint = expected_cluster_fact_fingerprint(&cluster);
        let context =
            ClusterTraceContext::new(Some(vec![1]), vec![cluster], Vec::new(), cluster_limits())
                .expect("context");
        let mut input = cluster_input(&target, &facts, Some(&context));
        input.limits.max_trace_field_bytes = 1;

        let error = replay_cluster_trace(input).expect_err("field byte limit");

        assert_eq!(error.detail(), RejectionDetail::ResourceExhaustion);

        let mut reduction = reduction_step(1, CheckedFactRef::ImportedFact(1));
        reduction.required_guard_ids = vec![1];
        reduction.discharged_guards.clear();
        reduction.strategy_audit_key = expected_strategy_audit_key(&reduction);
        reduction.result_fingerprint = expected_reduction_result_fingerprint(&reduction);
        let context =
            ClusterTraceContext::new(Some(vec![1]), Vec::new(), vec![reduction], cluster_limits())
                .expect("reduction context");
        let mut input = cluster_input(&target, &facts, Some(&context));
        input.limits.max_guard_evidence = 0;

        let guard_limit = replay_cluster_trace(input).expect_err("guard count limit");

        assert_eq!(guard_limit.detail(), RejectionDetail::ResourceExhaustion);
        assert_eq!(
            guard_limit.location().field_path,
            Some("required_guard_ids")
        );

        let empty_context =
            ClusterTraceContext::new(Some(vec![1]), Vec::new(), Vec::new(), cluster_limits())
                .expect("empty context");
        let mut requested_over_budget =
            cluster_input_with_requested(&target, &facts, Some(&empty_context), vec![1, 2]);
        requested_over_budget.limits.max_trace_steps = 1;
        let requested_error =
            replay_cluster_trace(requested_over_budget).expect_err("requested id count limit");
        assert_eq!(
            requested_error.detail(),
            RejectionDetail::ResourceExhaustion
        );
        assert_eq!(
            requested_error.location().field_path,
            Some("requested_trace_steps")
        );

        let mut dependency = cluster_step(1, CheckedFactRef::ImportedFact(1));
        dependency.generated_fact_fingerprint = expected_cluster_fact_fingerprint(&dependency);
        let mut requested = cluster_step(2, CheckedFactRef::TraceStep(1));
        requested.generated_fact_fingerprint = expected_cluster_fact_fingerprint(&requested);
        let context = ClusterTraceContext::new(
            Some(vec![1]),
            vec![dependency, requested],
            Vec::new(),
            cluster_limits(),
        )
        .expect("context");
        let mut closure_limited =
            cluster_input_with_requested(&target, &facts, Some(&context), vec![2]);
        closure_limited.limits.max_trace_steps = 1;
        let closure_error = replay_cluster_trace(closure_limited).expect_err("closure count limit");
        assert_eq!(closure_error.detail(), RejectionDetail::ResourceExhaustion);
        assert_eq!(
            closure_error.location().field_path,
            Some("requested_trace_steps")
        );

        let mut reduction = reduction_step(1, CheckedFactRef::ImportedFact(1));
        reduction.strategy_audit_key = expected_strategy_audit_key(&reduction);
        reduction.result_fingerprint = expected_reduction_result_fingerprint(&reduction);
        let context =
            ClusterTraceContext::new(Some(vec![1]), Vec::new(), vec![reduction], cluster_limits())
                .expect("context");
        let mut runtime_limited = cluster_input(&target, &facts, Some(&context));
        runtime_limited.limits.max_reduction_steps = 0;
        let runtime_error =
            replay_cluster_trace(runtime_limited).expect_err("runtime context limit");
        assert_eq!(runtime_error.detail(), RejectionDetail::ResourceExhaustion);
        assert_eq!(
            runtime_error.location().field_path,
            Some("cluster_trace_context.reduction_steps")
        );

        let mut reduction = reduction_step(1, CheckedFactRef::ImportedFact(1));
        reduction.strategy_audit_key = expected_strategy_audit_key(&reduction);
        reduction.result_fingerprint = expected_reduction_result_fingerprint(&reduction);
        let context =
            ClusterTraceContext::new(Some(vec![1]), Vec::new(), vec![reduction], cluster_limits())
                .expect("context");
        let mut binding_limited = cluster_input(&target, &facts, Some(&context));
        binding_limited.limits.max_reduction_bindings = 0;
        let binding_error = replay_cluster_trace(binding_limited).expect_err("binding count limit");
        assert_eq!(binding_error.detail(), RejectionDetail::ResourceExhaustion);
        assert_eq!(binding_error.location().field_path, Some("substitution"));

        let mut cluster = cluster_step(1, CheckedFactRef::ImportedFact(1));
        cluster.generated_fact_fingerprint = expected_cluster_fact_fingerprint(&cluster);
        let context =
            ClusterTraceContext::new(Some(vec![1]), vec![cluster], Vec::new(), cluster_limits())
                .expect("context");
        let mut commitment_limited = cluster_input(&target, &facts, Some(&context));
        commitment_limited.limits.max_commitment_bytes = 1;
        let commitment_error =
            replay_cluster_trace(commitment_limited).expect_err("commitment byte limit");
        assert_eq!(
            commitment_error.detail(),
            RejectionDetail::ResourceExhaustion
        );
        assert_eq!(
            commitment_error.location().field_path,
            Some("generated_fact_fingerprint")
        );
    }

    #[test]
    fn valid_imports_build_resolution_context_and_policy_taint() {
        let axiom_clause = ordinary(vec![neg_p()]);
        let theorem_clause = ordinary(vec![pos_q()]);
        let certificate = make_certificate(
            vec![imported_ref(
                1,
                b"pkg",
                b"mod",
                b"axiom",
                clause_fingerprint(&axiom_clause),
                RequiredProofStatus::DischargedBuiltin,
            )],
            vec![imported_ref(
                2,
                b"pkg",
                b"mod",
                b"theorem",
                clause_fingerprint(&theorem_clause),
                RequiredProofStatus::ExternallyAttestedPolicyPermitted,
            )],
        );
        let context = ImportedFactContext::new(
            Some(vec![9]),
            vec![evidence(
                &certificate.imported_axioms[0],
                AcceptedProofStatus::KernelVerified,
                axiom_clause.clone(),
            )],
            vec![evidence(
                &certificate.imported_theorems[0],
                AcceptedProofStatus::ExternallyAttestedPolicyPermitted,
                theorem_clause.clone(),
            )],
            context_limits(),
        )
        .expect("context");

        let report = check_imported_facts(input_with_policy(
            &certificate,
            Some(&context),
            ImportedFactPolicy {
                allow_externally_attested: true,
            },
            limits(),
        ))
        .expect("valid imports");

        assert_eq!(report.checked_imports().len(), 2);
        assert_eq!(
            report.checked_imports()[0].namespace,
            ImportedFactNamespace::ImportedAxiom
        );
        assert_eq!(
            report.checked_imports()[1].namespace,
            ImportedFactNamespace::ImportedTheorem
        );
        assert!(report.checked_imports()[1].policy_taint);
        assert!(report.policy_taint());
        assert_eq!(
            report.imported_clause_context().imported_axiom_clauses()[0].clause,
            axiom_clause
        );
        assert_eq!(
            report.imported_clause_context().imported_theorem_clauses()[0].clause,
            theorem_clause
        );
    }

    #[test]
    fn missing_context_or_provenance_is_missing_provenance() {
        let clause = ordinary(vec![neg_p()]);
        let certificate = make_certificate(
            vec![imported_ref(
                1,
                b"pkg",
                b"mod",
                b"axiom",
                clause_fingerprint(&clause),
                RequiredProofStatus::KernelVerified,
            )],
            Vec::new(),
        );

        let missing =
            check_imported_facts(input(&certificate, None, limits())).expect_err("missing context");
        assert_eq!(missing.detail(), RejectionDetail::MissingProvenance);
        assert_eq!(missing.location().field_path, Some("imported_fact_context"));

        let context = ImportedFactContext::new(
            Some(Vec::new()),
            vec![evidence(
                &certificate.imported_axioms[0],
                AcceptedProofStatus::KernelVerified,
                clause,
            )],
            Vec::new(),
            context_limits(),
        )
        .expect("context");
        let missing_provenance =
            check_imported_facts(input(&certificate, Some(&context), limits()))
                .expect_err("empty provenance");
        assert_eq!(
            missing_provenance.detail(),
            RejectionDetail::MissingProvenance
        );
    }

    #[test]
    fn identity_status_and_missing_evidence_fail_as_unresolved_symbol() {
        let clause = ordinary(vec![neg_p()]);
        let certificate = make_certificate(
            vec![imported_ref(
                1,
                b"pkg",
                b"mod",
                b"axiom",
                clause_fingerprint(&clause),
                RequiredProofStatus::KernelVerified,
            )],
            Vec::new(),
        );

        let context =
            ImportedFactContext::new(Some(vec![1]), Vec::new(), Vec::new(), context_limits())
                .expect("context");
        let missing_evidence = check_imported_facts(input(&certificate, Some(&context), limits()))
            .expect_err("missing evidence");
        assert_eq!(missing_evidence.detail(), RejectionDetail::UnresolvedSymbol);
        assert_eq!(missing_evidence.location().imported_fact_id, Some(1));

        let mut wrong_identity = evidence(
            &certificate.imported_axioms[0],
            AcceptedProofStatus::KernelVerified,
            clause.clone(),
        );
        wrong_identity.package_id = b"other".to_vec();
        let context = ImportedFactContext::new(
            Some(vec![1]),
            vec![wrong_identity],
            Vec::new(),
            context_limits(),
        )
        .expect("context");
        let identity_error = check_imported_facts(input(&certificate, Some(&context), limits()))
            .expect_err("identity mismatch");
        assert_eq!(identity_error.detail(), RejectionDetail::UnresolvedSymbol);
        assert_eq!(identity_error.location().field_path, Some("package_id"));

        let mut wrong_statement = evidence(
            &certificate.imported_axioms[0],
            AcceptedProofStatus::KernelVerified,
            clause.clone(),
        );
        wrong_statement.statement_fingerprint = Fingerprint::new(1, vec![9, 9, 9]);
        let context = ImportedFactContext::new(
            Some(vec![1]),
            vec![wrong_statement],
            Vec::new(),
            context_limits(),
        )
        .expect("context");
        let statement_error = check_imported_facts(input(&certificate, Some(&context), limits()))
            .expect_err("evidence statement fingerprint mismatch");
        assert_eq!(statement_error.detail(), RejectionDetail::UnresolvedSymbol);
        assert_eq!(
            statement_error.location().field_path,
            Some("statement_fingerprint")
        );

        let builtin_status = evidence(
            &certificate.imported_axioms[0],
            AcceptedProofStatus::DischargedBuiltin,
            clause.clone(),
        );
        let context = ImportedFactContext::new(
            Some(vec![1]),
            vec![builtin_status],
            Vec::new(),
            context_limits(),
        )
        .expect("context");
        let builtin_status_error =
            check_imported_facts(input(&certificate, Some(&context), limits()))
                .expect_err("builtin status is weaker than kernel verified");
        assert_eq!(
            builtin_status_error.detail(),
            RejectionDetail::UnresolvedSymbol
        );
        assert_eq!(
            builtin_status_error.location().field_path,
            Some("required_proof_status")
        );

        let weak_status = evidence(
            &certificate.imported_axioms[0],
            AcceptedProofStatus::ExternallyAttestedPolicyPermitted,
            clause.clone(),
        );
        let context = ImportedFactContext::new(
            Some(vec![1]),
            vec![weak_status],
            Vec::new(),
            context_limits(),
        )
        .expect("context");
        let status_error = check_imported_facts(input(&certificate, Some(&context), limits()))
            .expect_err("status mismatch");
        assert_eq!(status_error.detail(), RejectionDetail::UnresolvedSymbol);
        assert_eq!(
            status_error.location().field_path,
            Some("required_proof_status")
        );

        let external_certificate = make_certificate(
            vec![imported_ref(
                1,
                b"pkg",
                b"mod",
                b"axiom",
                clause_fingerprint(&clause),
                RequiredProofStatus::ExternallyAttestedPolicyPermitted,
            )],
            Vec::new(),
        );
        let external = evidence(
            &external_certificate.imported_axioms[0],
            AcceptedProofStatus::ExternallyAttestedPolicyPermitted,
            clause,
        );
        let context =
            ImportedFactContext::new(Some(vec![1]), vec![external], Vec::new(), context_limits())
                .expect("context");
        let policy_error =
            check_imported_facts(input(&external_certificate, Some(&context), limits()))
                .expect_err("external attestation is disabled by policy");
        assert_eq!(policy_error.detail(), RejectionDetail::UnresolvedSymbol);
        assert_eq!(
            policy_error.location().field_path,
            Some("required_proof_status")
        );
    }

    #[test]
    fn imported_clause_fingerprint_binding_is_checked_before_replay() {
        let clause = ordinary(vec![neg_p()]);
        let certificate = make_certificate(
            vec![imported_ref(
                1,
                b"pkg",
                b"mod",
                b"axiom",
                clause_fingerprint(&clause),
                RequiredProofStatus::KernelVerified,
            )],
            Vec::new(),
        );

        let mut wrong_normalized = evidence(
            &certificate.imported_axioms[0],
            AcceptedProofStatus::KernelVerified,
            clause.clone(),
        );
        wrong_normalized.normalized_clause_fingerprint = Fingerprint::new(1, vec![99]);
        let context = ImportedFactContext::new(
            Some(vec![1]),
            vec![wrong_normalized],
            Vec::new(),
            context_limits(),
        )
        .expect("context");
        let normalized_error = check_imported_facts(input(&certificate, Some(&context), limits()))
            .expect_err("normalized fingerprint mismatch");
        assert_eq!(
            normalized_error.location().field_path,
            Some("normalized_clause_fingerprint")
        );
        assert_eq!(normalized_error.detail(), RejectionDetail::UnresolvedSymbol);

        let wrong_statement = Fingerprint::new(1, vec![7, 7, 7]);
        let mismatched_certificate = make_certificate(
            vec![imported_ref(
                1,
                b"pkg",
                b"mod",
                b"axiom",
                wrong_statement.clone(),
                RequiredProofStatus::KernelVerified,
            )],
            Vec::new(),
        );
        let mut mismatched_evidence = evidence(
            &mismatched_certificate.imported_axioms[0],
            AcceptedProofStatus::KernelVerified,
            clause,
        );
        mismatched_evidence.normalized_clause_fingerprint = Fingerprint::new(
            1,
            mismatched_evidence.clause.canonical_hash_input().unwrap(),
        );
        let context = ImportedFactContext::new(
            Some(vec![1]),
            vec![mismatched_evidence],
            Vec::new(),
            context_limits(),
        )
        .expect("context");
        let statement_error =
            check_imported_facts(input(&mismatched_certificate, Some(&context), limits()))
                .expect_err("statement fingerprint must bind to clause content");
        assert_eq!(
            statement_error.location().field_path,
            Some("statement_fingerprint")
        );

        let unsupported_certificate = make_certificate(
            vec![imported_ref(
                1,
                b"pkg",
                b"mod",
                b"axiom",
                Fingerprint::new(9, clause_fingerprint(&ordinary(vec![neg_p()])).digest),
                RequiredProofStatus::KernelVerified,
            )],
            Vec::new(),
        );
        let unsupported = evidence(
            &unsupported_certificate.imported_axioms[0],
            AcceptedProofStatus::KernelVerified,
            ordinary(vec![neg_p()]),
        );
        let context = ImportedFactContext::new(
            Some(vec![1]),
            vec![unsupported],
            Vec::new(),
            context_limits(),
        )
        .expect("context");
        let unsupported_error =
            check_imported_facts(input(&unsupported_certificate, Some(&context), limits()))
                .expect_err("unsupported fingerprint algorithm");
        assert_eq!(
            unsupported_error.location().field_path,
            Some("statement_fingerprint.algorithm_id")
        );

        let mut unsupported_evidence_algorithm = evidence(
            &certificate.imported_axioms[0],
            AcceptedProofStatus::KernelVerified,
            ordinary(vec![neg_p()]),
        );
        unsupported_evidence_algorithm
            .normalized_clause_fingerprint
            .algorithm_id = 9;
        let context = ImportedFactContext::new(
            Some(vec![1]),
            vec![unsupported_evidence_algorithm],
            Vec::new(),
            context_limits(),
        )
        .expect("context");
        let unsupported_evidence_error =
            check_imported_facts(input(&certificate, Some(&context), limits()))
                .expect_err("unsupported evidence fingerprint algorithm");
        assert_eq!(
            unsupported_evidence_error.location().field_path,
            Some("normalized_clause_fingerprint.algorithm_id")
        );
    }

    #[test]
    fn imported_clause_profile_manifest_and_resource_limits_are_checked() {
        let clause = ordinary(vec![neg_p()]);
        let certificate = make_certificate(
            vec![imported_ref(
                1,
                b"pkg",
                b"mod",
                b"axiom",
                clause_fingerprint(&clause),
                RequiredProofStatus::KernelVerified,
            )],
            Vec::new(),
        );

        let context = ImportedFactContext::new(
            Some(vec![1]),
            vec![evidence(
                &certificate.imported_axioms[0],
                AcceptedProofStatus::KernelVerified,
                wrong_profile_clause(),
            )],
            Vec::new(),
            context_limits(),
        )
        .expect("context");
        let profile_error = check_imported_facts(input(&certificate, Some(&context), limits()))
            .expect_err("profile mismatch");
        assert_eq!(profile_error.detail(), RejectionDetail::MissingProvenance);
        assert_eq!(profile_error.location().field_path, Some("clause.profile"));

        let unknown_symbol = unknown_symbol_clause();
        let unknown_symbol_fingerprint = clause_fingerprint(&unknown_symbol);
        let unknown_certificate = make_certificate(
            vec![imported_ref(
                1,
                b"pkg",
                b"mod",
                b"axiom",
                unknown_symbol_fingerprint,
                RequiredProofStatus::KernelVerified,
            )],
            Vec::new(),
        );
        let context = ImportedFactContext::new(
            Some(vec![1]),
            vec![evidence(
                &unknown_certificate.imported_axioms[0],
                AcceptedProofStatus::KernelVerified,
                unknown_symbol,
            )],
            Vec::new(),
            context_limits(),
        )
        .expect("context");
        let manifest_error =
            check_imported_facts(input(&unknown_certificate, Some(&context), limits()))
                .expect_err("manifest mismatch");
        assert_eq!(manifest_error.detail(), RejectionDetail::MissingProvenance);
        assert_eq!(manifest_error.location().field_path, Some("clause"));

        let variable_clause = variable_clause(99);
        let variable_certificate = make_certificate(
            vec![imported_ref(
                1,
                b"pkg",
                b"mod",
                b"axiom",
                clause_fingerprint(&variable_clause),
                RequiredProofStatus::KernelVerified,
            )],
            Vec::new(),
        );
        let context = ImportedFactContext::new(
            Some(vec![1]),
            vec![evidence(
                &variable_certificate.imported_axioms[0],
                AcceptedProofStatus::KernelVerified,
                variable_clause,
            )],
            Vec::new(),
            context_limits(),
        )
        .expect("context");
        let variable_error =
            check_imported_facts(input(&variable_certificate, Some(&context), limits()))
                .expect_err("variable manifest mismatch");
        assert_eq!(variable_error.detail(), RejectionDetail::MissingProvenance);
        assert_eq!(variable_error.location().field_path, Some("clause"));

        let context = ImportedFactContext::new(
            Some(vec![1]),
            vec![evidence(
                &certificate.imported_axioms[0],
                AcceptedProofStatus::KernelVerified,
                clause,
            )],
            Vec::new(),
            context_limits(),
        )
        .expect("context");
        let mut tiny_limits = limits();
        tiny_limits.max_imported_clause_literals = 0;
        let resource_error = check_imported_facts(input(&certificate, Some(&context), tiny_limits))
            .expect_err("literal limit");
        assert_eq!(resource_error.detail(), RejectionDetail::ResourceExhaustion);
    }

    #[test]
    fn duplicate_context_ids_are_rejected_and_unused_malformed_entries_ignored() {
        let clause = ordinary(vec![neg_p()]);
        let certificate = make_certificate(
            vec![imported_ref(
                1,
                b"pkg",
                b"mod",
                b"axiom",
                clause_fingerprint(&clause),
                RequiredProofStatus::KernelVerified,
            )],
            Vec::new(),
        );
        let first = evidence(
            &certificate.imported_axioms[0],
            AcceptedProofStatus::KernelVerified,
            clause.clone(),
        );
        let duplicate = first.clone();
        let duplicate_error = ImportedFactContext::new(
            Some(vec![1]),
            vec![first, duplicate],
            Vec::new(),
            context_limits(),
        )
        .expect_err("duplicate context id");
        assert_eq!(
            duplicate_error,
            ImportedFactContextError::DuplicateImportedFact {
                namespace: ImportedFactNamespace::ImportedAxiom,
                imported_fact_id: 1
            }
        );

        let mut unused_malformed = evidence(
            &imported_ref(
                99,
                b"pkg",
                b"mod",
                b"unused",
                Fingerprint::new(1, vec![99]),
                RequiredProofStatus::KernelVerified,
            ),
            AcceptedProofStatus::KernelVerified,
            wrong_profile_clause(),
        );
        unused_malformed.normalized_clause_fingerprint = Fingerprint::new(1, vec![99]);
        let context = ImportedFactContext::new(
            Some(vec![1]),
            vec![
                unused_malformed,
                evidence(
                    &certificate.imported_axioms[0],
                    AcceptedProofStatus::KernelVerified,
                    clause,
                ),
            ],
            Vec::new(),
            context_limits(),
        )
        .expect("context canonicalizes order");

        check_imported_facts(input(&certificate, Some(&context), limits()))
            .expect("unused malformed context entry is ignored");
    }

    #[test]
    fn context_and_reports_are_canonical_under_shuffled_evidence() {
        let first_clause = ordinary(vec![neg_p()]);
        let second_clause = ordinary(vec![pos_q()]);
        let certificate = make_certificate(
            vec![
                imported_ref(
                    1,
                    b"pkg",
                    b"mod",
                    b"first",
                    clause_fingerprint(&first_clause),
                    RequiredProofStatus::KernelVerified,
                ),
                imported_ref(
                    2,
                    b"pkg",
                    b"mod",
                    b"second",
                    clause_fingerprint(&second_clause),
                    RequiredProofStatus::KernelVerified,
                ),
            ],
            Vec::new(),
        );
        let context = ImportedFactContext::new(
            Some(vec![1]),
            vec![
                evidence(
                    &certificate.imported_axioms[1],
                    AcceptedProofStatus::KernelVerified,
                    second_clause,
                ),
                evidence(
                    &certificate.imported_axioms[0],
                    AcceptedProofStatus::KernelVerified,
                    first_clause,
                ),
            ],
            Vec::new(),
            context_limits(),
        )
        .expect("context canonicalizes evidence order");

        assert_eq!(
            context
                .imported_axioms()
                .iter()
                .map(|entry| entry.imported_fact_id)
                .collect::<Vec<_>>(),
            [1, 2]
        );

        let report = check_imported_facts(input(&certificate, Some(&context), limits()))
            .expect("valid shuffled context");

        assert_eq!(
            report
                .checked_imports()
                .iter()
                .map(|entry| entry.imported_fact_id)
                .collect::<Vec<_>>(),
            [1, 2]
        );
        assert_eq!(
            report
                .imported_clause_context()
                .imported_axiom_clauses()
                .iter()
                .map(|entry| entry.imported_fact_id)
                .collect::<Vec<_>>(),
            [1, 2]
        );
    }

    #[test]
    fn context_constructor_rejects_entry_count_before_sorting() {
        let clause = ordinary(vec![neg_p()]);
        let imported = imported_ref(
            1,
            b"pkg",
            b"mod",
            b"axiom",
            clause_fingerprint(&clause),
            RequiredProofStatus::KernelVerified,
        );

        let error = ImportedFactContext::new(
            Some(vec![1]),
            vec![evidence(
                &imported,
                AcceptedProofStatus::KernelVerified,
                clause,
            )],
            Vec::new(),
            ImportedFactContextLimits {
                max_imported_context_entries: 0,
            },
        )
        .expect_err("context entry count limit");

        assert_eq!(
            error,
            ImportedFactContextError::ImportedFactCountExceeded { max: 0, actual: 1 }
        );
    }

    #[test]
    fn imported_fact_count_limit_rejects_before_context_lookup() {
        let clause = ordinary(vec![neg_p()]);
        let certificate = make_certificate(
            vec![imported_ref(
                1,
                b"pkg",
                b"mod",
                b"axiom",
                clause_fingerprint(&clause),
                RequiredProofStatus::KernelVerified,
            )],
            Vec::new(),
        );
        let mut tiny_limits = limits();
        tiny_limits.max_imported_facts = 0;

        let error = check_imported_facts(input(&certificate, None, tiny_limits))
            .expect_err("count limit should fire before missing context");

        assert_eq!(error.detail(), RejectionDetail::ResourceExhaustion);
        assert_eq!(
            error.location().field_path,
            Some("imported_fact_context.imported_fact_count")
        );
    }

    fn cluster_input<'a>(
        target: &'a TargetVcFingerprint,
        facts: &'a CheckedFactContext,
        context: Option<&'a ClusterTraceContext>,
    ) -> ClusterTraceReplayInput<'a> {
        let requested = context.map_or_else(
            || vec![1],
            |context| {
                let mut ids: Vec<u32> = context
                    .cluster_steps()
                    .iter()
                    .map(|step| step.cluster_trace_step_id)
                    .chain(
                        context
                            .reduction_steps()
                            .iter()
                            .map(|step| step.reduction_step_id),
                    )
                    .collect();
                ids.sort_unstable();
                ids
            },
        );
        cluster_input_with_requested(target, facts, context, requested)
    }

    fn cluster_input_with_requested<'a>(
        target: &'a TargetVcFingerprint,
        facts: &'a CheckedFactContext,
        context: Option<&'a ClusterTraceContext>,
        requested: Vec<u32>,
    ) -> ClusterTraceReplayInput<'a> {
        ClusterTraceReplayInput {
            target_vc_fingerprint: target,
            checked_fact_context: facts,
            cluster_trace_context: context,
            requested_trace_steps: Box::leak(requested.into_boxed_slice()),
            limits: cluster_limits(),
        }
    }

    fn checked_fact_context() -> CheckedFactContext {
        CheckedFactContext::new(vec![1], vec![7]).expect("checked fact context")
    }

    fn cluster_limits() -> ClusterTraceReplayLimits {
        ClusterTraceReplayLimits {
            max_cluster_steps: 8,
            max_reduction_steps: 8,
            max_trace_steps: 16,
            max_guard_evidence: 8,
            max_reduction_bindings: 8,
            max_trace_field_bytes: 4096,
            max_commitment_bytes: 8192,
        }
    }

    fn cluster_step(cluster_trace_step_id: u32, dependency: CheckedFactRef) -> ClusterStepEvidence {
        ClusterStepEvidence {
            cluster_trace_step_id,
            source_type: b"type:T".to_vec(),
            applied_cluster: b"cluster:C".to_vec(),
            generated_attribute: b"attr:A".to_vec(),
            generated_type: b"type:T+A".to_vec(),
            dependency,
            generated_fact_fingerprint: Vec::new(),
        }
    }

    fn reduction_step(
        reduction_step_id: u32,
        guard_dependency: CheckedFactRef,
    ) -> ReductionStepEvidence {
        ReductionStepEvidence {
            reduction_step_id,
            applied_reduction: b"reduction:R".to_vec(),
            rule_fqn: b"pkg::module::R".to_vec(),
            enclosing_term_before: b"term:before:R".to_vec(),
            redex_path: b"path:0.1".to_vec(),
            source_redex: b"term:redex:R".to_vec(),
            target_term: b"term:target:R".to_vec(),
            substitution: vec![ReductionBindingEvidence {
                variable: b"x".to_vec(),
                replacement: b"replacement".to_vec(),
            }],
            required_guard_ids: vec![1],
            discharged_guards: vec![GuardEvidence {
                guard_id: 1,
                source_fact_ref: guard_dependency,
                checked_dependency_ref: CheckedFactRef::ImportedFact(1),
            }],
            rule_view: b"fingerprint:R".to_vec(),
            selection_key: b"selection:R".to_vec(),
            strategy_audit_key: Vec::new(),
            result_fingerprint: Vec::new(),
        }
    }

    fn input<'a>(
        certificate: &'a ParsedCertificate,
        context: Option<&'a ImportedFactContext>,
        limits: ImportedFactCheckLimits,
    ) -> ImportedFactCheckInput<'a> {
        input_with_policy(certificate, context, ImportedFactPolicy::default(), limits)
    }

    fn input_with_policy<'a>(
        certificate: &'a ParsedCertificate,
        context: Option<&'a ImportedFactContext>,
        policy: ImportedFactPolicy,
        limits: ImportedFactCheckLimits,
    ) -> ImportedFactCheckInput<'a> {
        ImportedFactCheckInput {
            target_vc_fingerprint: Box::leak(Box::new(TargetVcFingerprint::new(1, vec![42]))),
            certificate,
            imported_fact_context: context,
            policy,
            limits,
        }
    }

    fn make_certificate(
        imported_axioms: Vec<ImportedFactRef>,
        imported_theorems: Vec<ImportedFactRef>,
    ) -> ParsedCertificate {
        ParsedCertificate::new_for_kernel_tests(ParsedCertificateTestParts {
            schema_version: 1,
            encoding_version: 1,
            kernel_profile: KernelProfileRecord::v1(1, ClauseTautologyPolicy::Reject),
            target_vc: Fingerprint::new(1, vec![42]),
            symbol_manifest: vec![
                SymbolManifestEntry { symbol: p_symbol() },
                SymbolManifestEntry { symbol: q_symbol() },
            ],
            variable_manifest: vec![VariableManifestEntry {
                variable_id: VariableId(1),
            }],
            imported_axioms,
            imported_theorems,
            generated_clauses: Vec::new(),
            substitutions: Vec::new(),
            resolution_trace: Vec::new(),
            derived_facts: Vec::new(),
            final_goal: FinalGoalRef {
                namespace: FinalGoalNamespace::GeneratedClause,
                id: 0,
            },
            canonical_hash_input: vec![1, 2, 3],
        })
    }

    fn imported_ref(
        imported_fact_id: u32,
        package_id: &[u8],
        module_path: &[u8],
        exported_item_id: &[u8],
        statement_fingerprint: Fingerprint,
        required_proof_status: RequiredProofStatus,
    ) -> ImportedFactRef {
        ImportedFactRef {
            imported_fact_id,
            package_id: package_id.to_vec(),
            module_path: module_path.to_vec(),
            exported_item_id: exported_item_id.to_vec(),
            statement_fingerprint,
            required_proof_status,
        }
    }

    fn evidence(
        imported: &ImportedFactRef,
        accepted_proof_status: AcceptedProofStatus,
        clause: Clause,
    ) -> ImportedFactEvidence {
        let normalized_clause_fingerprint = clause_fingerprint(&clause);
        ImportedFactEvidence {
            imported_fact_id: imported.imported_fact_id,
            package_id: imported.package_id.clone(),
            module_path: imported.module_path.clone(),
            exported_item_id: imported.exported_item_id.clone(),
            statement_fingerprint: imported.statement_fingerprint.clone(),
            accepted_proof_status,
            normalized_clause_fingerprint,
            clause,
        }
    }

    fn clause_fingerprint(clause: &Clause) -> Fingerprint {
        Fingerprint::new(
            1,
            clause
                .canonical_hash_input()
                .expect("test clause canonical hash input"),
        )
    }

    fn limits() -> ImportedFactCheckLimits {
        ImportedFactCheckLimits {
            max_imported_facts: 16,
            max_imported_context_entries: 16,
            max_imported_clause_literals: 8,
            max_imported_clause_canonical_bytes: 4096,
            max_imported_term_encoding_bytes: 4096,
            max_imported_term_recursion_depth: 16,
        }
    }

    fn context_limits() -> ImportedFactContextLimits {
        ImportedFactContextLimits {
            max_imported_context_entries: 16,
        }
    }

    fn ordinary(literals: Vec<Literal>) -> Clause {
        Clause::from_canonical_parts(ClauseForm::Ordinary, literals, &base_context())
            .expect("ordinary clause")
    }

    fn wrong_profile_clause() -> Clause {
        let context =
            ClauseValidationContext::new(ClauseProfile::new(1, 2, TautologyPolicy::Reject))
                .with_known_symbol(p_symbol())
                .with_canonical_variable(VariableId(1))
                .with_limits(8, 4096)
                .with_max_term_recursion_depth(16);
        Clause::from_canonical_parts(ClauseForm::Ordinary, vec![neg_p()], &context)
            .expect("wrong profile clause")
    }

    fn unknown_symbol_clause() -> Clause {
        let symbol = SymbolKey {
            kind: SymbolKind::Predicate,
            id: SymbolId(99),
        };
        let context =
            ClauseValidationContext::new(ClauseProfile::new(1, 1, TautologyPolicy::Reject))
                .with_known_symbol(symbol)
                .with_canonical_variable(VariableId(1))
                .with_limits(8, 4096)
                .with_max_term_recursion_depth(16);
        Clause::from_canonical_parts(
            ClauseForm::Ordinary,
            vec![Literal::new(
                Polarity::Negative,
                Atom::new(symbol, Vec::new()),
            )],
            &context,
        )
        .expect("unknown symbol clause")
    }

    fn variable_clause(variable_id: u32) -> Clause {
        let context =
            ClauseValidationContext::new(ClauseProfile::new(1, 1, TautologyPolicy::Reject))
                .with_known_symbol(p_symbol())
                .with_canonical_variable(VariableId(variable_id))
                .with_limits(8, 4096)
                .with_max_term_recursion_depth(16);
        Clause::from_canonical_parts(
            ClauseForm::Ordinary,
            vec![Literal::new(
                Polarity::Negative,
                Atom::new(p_symbol(), vec![Term::Variable(VariableId(variable_id))]),
            )],
            &context,
        )
        .expect("variable clause")
    }

    fn base_context() -> ClauseValidationContext {
        ClauseValidationContext::new(ClauseProfile::new(1, 1, TautologyPolicy::Reject))
            .with_known_symbol(p_symbol())
            .with_known_symbol(q_symbol())
            .with_canonical_variable(VariableId(1))
            .with_limits(8, 4096)
            .with_max_term_recursion_depth(16)
    }

    fn neg_p() -> Literal {
        Literal::new(Polarity::Negative, Atom::new(p_symbol(), Vec::new()))
    }

    fn pos_q() -> Literal {
        Literal::new(Polarity::Positive, Atom::new(q_symbol(), Vec::new()))
    }

    const fn p_symbol() -> SymbolKey {
        SymbolKey {
            kind: SymbolKind::Predicate,
            id: SymbolId(1),
        }
    }

    const fn q_symbol() -> SymbolKey {
        SymbolKey {
            kind: SymbolKind::Predicate,
            id: SymbolId(2),
        }
    }
}
