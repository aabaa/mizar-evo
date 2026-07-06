use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write as _,
};

use crate::{
    certificate_parser::{
        ClauseRef, FinalGoalNamespace, Fingerprint, ImportedFactRef, ParsedCertificate,
        RequiredProofStatus,
    },
    clause::{Clause, ClauseError, ClauseProfile, ClauseValidationContext},
    formula_evidence::{FormulaSource, GoalPolarity, ImportedFormulaSource, ParsedKernelEvidence},
    rejection::{
        ClauseRef as RejectionClauseRef, ClauseRefNamespace as RejectionClauseRefNamespace,
        RejectionCategory, RejectionDetail, RejectionLocation, RejectionRecord,
        TargetVcFingerprint,
    },
    resolution_trace::{
        CheckedResolutionStep, ImportedClauseContext, ImportedClauseContextError,
        ImportedClauseEntry, ResolutionReplayLimits, ResolutionReplayReport, ResolutionTraceInput,
        checked_resolution_final_goal, replay_resolution_trace,
    },
    sat_checker::{
        SatCheckContext, SatCheckLimits, SatCheckReport, SatCheckResult, check_sat_problem,
    },
    sat_encoding::{SatEncodingContext, SatEncodingLimits, encode_formula_evidence},
    substitution_checker::{
        CheckedSubstitution, SubstitutionCheckInput, SubstitutionContext, SubstitutionReplayLimits,
        checked_substitutions_for_input, replay_substitutions,
    },
};
use mizar_core::core_ir::CoreFormulaId;
use mizar_session::Hash;

pub const SUPPORTED_NORMALIZED_CLAUSE_FINGERPRINT_ALGORITHM_ID: u8 = 1;
pub const KERNEL_CONTEXT_IDENTITY_SCHEMA_VERSION: u16 = 1;
const KERNEL_CONTEXT_IDENTITY_HASH_DOMAIN: &str = "mizar-vc-kernel-context-identity";

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
    pub max_context_identity_entries: usize,
}

impl Default for ImportedFactContextLimits {
    fn default() -> Self {
        Self {
            max_imported_context_entries: usize::MAX,
            max_context_identity_entries: usize::MAX,
        }
    }
}

impl From<ImportedFactCheckLimits> for ImportedFactContextLimits {
    fn from(value: ImportedFactCheckLimits) -> Self {
        Self {
            max_imported_context_entries: value.max_imported_context_entries,
            max_context_identity_entries: usize::MAX,
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
pub struct FormulaEvidenceContext {
    provenance_fingerprint: Option<Vec<u8>>,
    imported_axioms: Vec<FormulaImportedFactEvidence>,
    imported_theorems: Vec<FormulaImportedFactEvidence>,
    context_identity: Option<KernelContextIdentityPayload>,
}

impl FormulaEvidenceContext {
    pub fn new(
        provenance_fingerprint: Option<Vec<u8>>,
        imported_axioms: Vec<FormulaImportedFactEvidence>,
        imported_theorems: Vec<FormulaImportedFactEvidence>,
        limits: ImportedFactContextLimits,
    ) -> Result<Self, ImportedFactContextError> {
        Self::with_context_identity(
            provenance_fingerprint,
            imported_axioms,
            imported_theorems,
            None,
            limits,
        )
    }

    pub fn with_context_identity(
        provenance_fingerprint: Option<Vec<u8>>,
        imported_axioms: Vec<FormulaImportedFactEvidence>,
        imported_theorems: Vec<FormulaImportedFactEvidence>,
        context_identity: Option<KernelContextIdentityPayload>,
        limits: ImportedFactContextLimits,
    ) -> Result<Self, ImportedFactContextError> {
        validate_context_entry_count(&imported_axioms, &imported_theorems, limits)?;
        validate_context_identity_entry_count(context_identity.as_ref(), limits)?;
        Ok(Self {
            provenance_fingerprint,
            imported_axioms: canonical_formula_imports(
                ImportedFactNamespace::ImportedAxiom,
                imported_axioms,
            )?,
            imported_theorems: canonical_formula_imports(
                ImportedFactNamespace::ImportedTheorem,
                imported_theorems,
            )?,
            context_identity,
        })
    }

    #[must_use]
    pub fn provenance_fingerprint(&self) -> Option<&[u8]> {
        self.provenance_fingerprint.as_deref()
    }

    #[must_use]
    pub fn imported_axioms(&self) -> &[FormulaImportedFactEvidence] {
        &self.imported_axioms
    }

    #[must_use]
    pub fn imported_theorems(&self) -> &[FormulaImportedFactEvidence] {
        &self.imported_theorems
    }

    #[must_use]
    pub const fn context_identity(&self) -> Option<&KernelContextIdentityPayload> {
        self.context_identity.as_ref()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ImportedFactContextError {
    ImportedFactCountExceeded {
        max: usize,
        actual: usize,
    },
    DuplicateImportedFact {
        namespace: ImportedFactNamespace,
        imported_fact_id: u32,
    },
    ContextIdentityCountExceeded {
        max: usize,
        actual: usize,
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FormulaImportedFactEvidence {
    pub imported_fact_id: u32,
    pub package_id: Vec<u8>,
    pub module_path: Vec<u8>,
    pub exported_item_id: Vec<u8>,
    pub statement_fingerprint: Fingerprint,
    pub accepted_proof_status: AcceptedProofStatus,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KernelContextIdentityPayload {
    schema_version: u16,
    target_vc: TargetVcFingerprint,
    canonical_handoff_hash: Hash,
    context_identity_hash: Hash,
    entries: Vec<KernelContextIdentityEntry>,
}

impl KernelContextIdentityPayload {
    #[must_use]
    pub fn new(
        target_vc: TargetVcFingerprint,
        canonical_handoff_hash: Hash,
        context_identity_hash: Hash,
        mut entries: Vec<KernelContextIdentityEntry>,
    ) -> Self {
        entries.sort();
        Self {
            schema_version: KERNEL_CONTEXT_IDENTITY_SCHEMA_VERSION,
            target_vc,
            canonical_handoff_hash,
            context_identity_hash,
            entries,
        }
    }

    #[must_use]
    pub const fn schema_version(&self) -> u16 {
        self.schema_version
    }

    #[must_use]
    pub const fn target_vc(&self) -> &TargetVcFingerprint {
        &self.target_vc
    }

    #[must_use]
    pub const fn canonical_handoff_hash(&self) -> Hash {
        self.canonical_handoff_hash
    }

    #[must_use]
    pub const fn context_identity_hash(&self) -> Hash {
        self.context_identity_hash
    }

    #[must_use]
    pub fn entries(&self) -> &[KernelContextIdentityEntry] {
        &self.entries
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct KernelContextIdentityEntry {
    source: KernelContextIdentitySource,
    formula_id: u32,
    formula_fingerprint: Fingerprint,
    producer_formula_ref: KernelFormulaProducerRef,
}

impl KernelContextIdentityEntry {
    #[must_use]
    pub fn new(
        source: KernelContextIdentitySource,
        formula_id: u32,
        formula_fingerprint: Fingerprint,
        producer_formula_ref: KernelFormulaProducerRef,
    ) -> Self {
        Self {
            source,
            formula_id,
            formula_fingerprint,
            producer_formula_ref,
        }
    }

    #[must_use]
    pub const fn source(&self) -> KernelContextIdentitySource {
        self.source
    }

    #[must_use]
    pub const fn formula_id(&self) -> u32 {
        self.formula_id
    }

    #[must_use]
    pub const fn formula_fingerprint(&self) -> &Fingerprint {
        &self.formula_fingerprint
    }

    #[must_use]
    pub const fn producer_formula_ref(&self) -> KernelFormulaProducerRef {
        self.producer_formula_ref
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[non_exhaustive]
pub enum KernelContextIdentitySource {
    LocalHypothesis { local_context_id: u32 },
    CitedPremise { local_context_id: u32 },
    GeneratedVcFact { vc_fact_id: u32 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[non_exhaustive]
pub enum KernelFormulaProducerRef {
    Core(CoreFormulaId),
    Generated(KernelVcGeneratedFormulaId),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct KernelVcGeneratedFormulaId(usize);

impl KernelVcGeneratedFormulaId {
    #[must_use]
    pub const fn new(index: usize) -> Self {
        Self(index)
    }

    #[must_use]
    pub const fn index(self) -> usize {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[non_exhaustive]
pub enum ImportedFactNamespace {
    ImportedAxiom,
    ImportedTheorem,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
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
struct FormulaImportCheckReport {
    checked_imports: Vec<CheckedImportedFact>,
    used_axioms: Vec<UsedAxiom>,
    policy_taint: bool,
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

fn canonical_formula_imports(
    namespace: ImportedFactNamespace,
    mut evidence: Vec<FormulaImportedFactEvidence>,
) -> Result<Vec<FormulaImportedFactEvidence>, ImportedFactContextError> {
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

fn validate_context_entry_count<T, U>(
    imported_axioms: &[T],
    imported_theorems: &[U],
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

fn validate_context_identity_entry_count(
    identity: Option<&KernelContextIdentityPayload>,
    limits: ImportedFactContextLimits,
) -> Result<(), ImportedFactContextError> {
    let actual = identity.map_or(0, |payload| payload.entries().len());
    if actual > limits.max_context_identity_entries {
        return Err(ImportedFactContextError::ContextIdentityCountExceeded {
            max: limits.max_context_identity_entries,
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
    rejection_with_category(target, RejectionCategory::KernelRejection, detail, location)
}

fn rejection_with_category(
    target: &TargetVcFingerprint,
    category: RejectionCategory,
    detail: RejectionDetail,
    location: RejectionLocation,
) -> Box<RejectionRecord> {
    Box::new(
        RejectionRecord::new(target.clone(), category, detail, location)
            .expect("checker uses valid rejection detail mappings"),
    )
}

fn imported_location(imported_fact_id: u32) -> RejectionLocation {
    RejectionLocation::new().with_imported_fact_id(imported_fact_id)
}

#[derive(Clone, Copy, Debug)]
pub struct KernelCheckInput<'a> {
    pub target_vc_fingerprint: &'a TargetVcFingerprint,
    pub certificate: &'a ParsedCertificate,
    pub imported_fact_context: Option<&'a ImportedFactContext>,
    pub substitution_context: Option<&'a SubstitutionContext>,
    pub cluster_trace_context: Option<&'a ClusterTraceContext>,
    pub requested_cluster_trace_steps: &'a [u32],
    pub policy: KernelCheckPolicy,
    pub limits: KernelCheckLimits,
}

#[derive(Clone, Copy, Debug)]
pub struct KernelEvidenceCheckInput<'a> {
    pub target_vc_fingerprint: &'a TargetVcFingerprint,
    pub evidence: &'a ParsedKernelEvidence,
    pub formula_context: Option<&'a FormulaEvidenceContext>,
    pub check_kind: KernelEvidenceCheckKind,
    pub policy: KernelCheckPolicy,
    pub limits: KernelEvidenceCheckLimits,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum KernelEvidenceCheckKind {
    ProofObligation,
    ConsistencyCheck,
}

impl KernelEvidenceCheckKind {
    const fn required_goal_polarity(self) -> GoalPolarity {
        match self {
            Self::ProofObligation => GoalPolarity::AssertFalseForRefutation,
            Self::ConsistencyCheck => GoalPolarity::AssertTrueForConsistency,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct KernelCheckPolicy {
    pub imported_fact_policy: ImportedFactPolicy,
    pub allow_legacy_certificate_audit: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KernelCheckLimits {
    pub imported_facts: ImportedFactCheckLimits,
    pub substitutions: SubstitutionReplayLimits,
    pub resolution: ResolutionReplayLimits,
    pub cluster_trace: ClusterTraceReplayLimits,
    pub max_pipeline_steps: usize,
    pub max_derived_facts: usize,
    pub max_report_records: usize,
}

impl Default for KernelCheckLimits {
    fn default() -> Self {
        Self {
            imported_facts: ImportedFactCheckLimits::default(),
            substitutions: SubstitutionReplayLimits::default(),
            resolution: ResolutionReplayLimits::default(),
            cluster_trace: ClusterTraceReplayLimits::default(),
            max_pipeline_steps: usize::MAX,
            max_derived_facts: usize::MAX,
            max_report_records: usize::MAX,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KernelEvidenceCheckLimits {
    pub formula_context: ImportedFactContextLimits,
    pub sat_encoding: SatEncodingLimits,
    pub sat_checker: SatCheckLimits,
    pub max_pipeline_steps: usize,
    pub max_report_records: usize,
}

impl Default for KernelEvidenceCheckLimits {
    fn default() -> Self {
        Self {
            formula_context: ImportedFactContextLimits::default(),
            sat_encoding: SatEncodingLimits::default(),
            sat_checker: SatCheckLimits::default(),
            max_pipeline_steps: usize::MAX,
            max_report_records: usize::MAX,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KernelCheckResult {
    target_vc_fingerprint: TargetVcFingerprint,
    status: KernelCheckStatus,
    checked_imports: Vec<CheckedImportedFact>,
    checked_substitutions: Vec<CheckedSubstitution>,
    checked_resolution_steps: Vec<CheckedResolutionStep>,
    checked_cluster_steps: Vec<CheckedClusterStep>,
    checked_reduction_steps: Vec<CheckedReductionStep>,
    checked_derived_facts: Vec<CheckedDerivedFact>,
    sat_check_report: Option<SatCheckReport>,
    final_goal: Option<CheckedFinalGoal>,
    evidence_check_kind: Option<KernelEvidenceCheckKind>,
    used_axioms: Vec<UsedAxiom>,
    policy_taint: bool,
    rejections: Vec<RejectionRecord>,
}

impl KernelCheckResult {
    #[must_use]
    pub const fn target_vc_fingerprint(&self) -> &TargetVcFingerprint {
        &self.target_vc_fingerprint
    }

    #[must_use]
    pub const fn status(&self) -> KernelCheckStatus {
        self.status
    }

    #[must_use]
    pub fn checked_imports(&self) -> &[CheckedImportedFact] {
        &self.checked_imports
    }

    #[must_use]
    pub fn checked_substitutions(&self) -> &[CheckedSubstitution] {
        &self.checked_substitutions
    }

    #[must_use]
    pub fn checked_resolution_steps(&self) -> &[CheckedResolutionStep] {
        &self.checked_resolution_steps
    }

    #[must_use]
    pub fn checked_cluster_steps(&self) -> &[CheckedClusterStep] {
        &self.checked_cluster_steps
    }

    #[must_use]
    pub fn checked_reduction_steps(&self) -> &[CheckedReductionStep] {
        &self.checked_reduction_steps
    }

    #[must_use]
    pub fn checked_derived_facts(&self) -> &[CheckedDerivedFact] {
        &self.checked_derived_facts
    }

    #[must_use]
    pub const fn sat_check_report(&self) -> Option<&SatCheckReport> {
        self.sat_check_report.as_ref()
    }

    #[must_use]
    pub const fn final_goal(&self) -> Option<CheckedFinalGoal> {
        self.final_goal
    }

    #[must_use]
    pub const fn evidence_check_kind(&self) -> Option<KernelEvidenceCheckKind> {
        self.evidence_check_kind
    }

    #[must_use]
    pub fn used_axioms(&self) -> &[UsedAxiom] {
        &self.used_axioms
    }

    #[must_use]
    pub const fn policy_taint(&self) -> bool {
        self.policy_taint
    }

    #[must_use]
    pub fn rejections(&self) -> &[RejectionRecord] {
        &self.rejections
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum KernelCheckStatus {
    Accepted,
    Rejected,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedDerivedFact {
    pub derived_fact_id: u32,
    pub source_clause_ref: ClauseRef,
    pub payload_fingerprint: Fingerprint,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CheckedFinalGoal {
    pub namespace: FinalGoalNamespace,
    pub id: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UsedAxiom {
    pub namespace: ImportedFactNamespace,
    pub imported_fact_id: u32,
    pub statement_fingerprint: Fingerprint,
}

pub type KernelCheckServiceResult<T> = Result<T, Box<RejectionRecord>>;

pub fn check_kernel_certificate(input: KernelCheckInput<'_>) -> KernelCheckResult {
    match check_kernel_certificate_inner(input) {
        Ok(result) => result,
        Err(rejection) => rejected_kernel_result(input.target_vc_fingerprint, *rejection),
    }
}

pub fn check_kernel_batch(inputs: &[KernelCheckInput<'_>]) -> Vec<KernelCheckResult> {
    let mut results: Vec<(usize, KernelCheckResult)> = inputs
        .iter()
        .copied()
        .enumerate()
        .map(|(input_order, input)| (input_order, check_kernel_certificate(input)))
        .collect();
    results.sort_by(|(left_order, left), (right_order, right)| {
        left.target_vc_fingerprint
            .cmp(&right.target_vc_fingerprint)
            .then_with(|| left_order.cmp(right_order))
    });
    results.into_iter().map(|(_, result)| result).collect()
}

pub fn check_kernel_evidence(input: KernelEvidenceCheckInput<'_>) -> KernelCheckResult {
    match check_kernel_evidence_inner(input) {
        Ok(result) => result,
        Err(rejection) => rejected_kernel_result(input.target_vc_fingerprint, *rejection),
    }
}

pub fn check_kernel_evidence_batch(
    inputs: &[KernelEvidenceCheckInput<'_>],
) -> Vec<KernelCheckResult> {
    let mut results: Vec<(usize, KernelCheckResult)> = inputs
        .iter()
        .copied()
        .enumerate()
        .map(|(input_order, input)| (input_order, check_kernel_evidence(input)))
        .collect();
    results.sort_by(|(left_order, left), (right_order, right)| {
        left.target_vc_fingerprint
            .cmp(&right.target_vc_fingerprint)
            .then_with(|| left_order.cmp(right_order))
    });
    results.into_iter().map(|(_, result)| result).collect()
}

fn check_kernel_evidence_inner(
    input: KernelEvidenceCheckInput<'_>,
) -> KernelCheckServiceResult<KernelCheckResult> {
    let mut budget = KernelPipelineBudget::new(input.limits.max_pipeline_steps);
    budget.step_target(input.target_vc_fingerprint, "target_vc")?;
    validate_evidence_target_binding(input)?;
    validate_evidence_goal_polarity(input)?;

    budget.step_target(input.target_vc_fingerprint, "formula_context")?;
    check_formula_context_identity(input)?;
    let formula_report = check_formula_imports(input)?;

    budget.step_target(input.target_vc_fingerprint, "sat_encoding")?;
    let encoded = encode_formula_evidence(
        input.evidence,
        &SatEncodingContext::v1().with_limits(input.limits.sat_encoding),
    )?;

    budget.step_target(input.target_vc_fingerprint, "sat_checker")?;
    let sat_report = match check_sat_problem(
        &encoded,
        &SatCheckContext::v1().with_limits(input.limits.sat_checker),
    ) {
        SatCheckResult::Unsat(report) => report,
        SatCheckResult::Sat(_) => {
            return Err(rejection(
                input.target_vc_fingerprint,
                RejectionDetail::InvalidSatRefutation,
                RejectionLocation::new().with_field_path("sat_checker.satisfiable"),
            ));
        }
        SatCheckResult::Rejected(rejection) => return Err(Box::new(rejection)),
    };

    validate_report_record_total(
        input.target_vc_fingerprint,
        input.limits.max_report_records,
        ReportRecordCounts {
            imports: formula_report.checked_imports.len(),
            // Formula substitutions are validated during SAT encoding; they are
            // not materialized as legacy CheckedSubstitution report records.
            substitutions: 0,
            resolution_steps: 0,
            cluster_steps: 0,
            reduction_steps: 0,
            derived_facts: 0,
            used_axioms: formula_report.used_axioms.len(),
            final_goals: 1,
            rejections: 0,
        },
    )?;

    Ok(KernelCheckResult {
        target_vc_fingerprint: input.target_vc_fingerprint.clone(),
        status: KernelCheckStatus::Accepted,
        checked_imports: formula_report.checked_imports,
        checked_substitutions: Vec::new(),
        checked_resolution_steps: Vec::new(),
        checked_cluster_steps: Vec::new(),
        checked_reduction_steps: Vec::new(),
        checked_derived_facts: Vec::new(),
        sat_check_report: Some(sat_report),
        final_goal: None,
        evidence_check_kind: Some(input.check_kind),
        used_axioms: formula_report.used_axioms,
        policy_taint: formula_report.policy_taint,
        rejections: Vec::new(),
    })
}

fn validate_evidence_target_binding(
    input: KernelEvidenceCheckInput<'_>,
) -> KernelCheckServiceResult<()> {
    let evidence_target =
        TargetVcFingerprint::from_certificate_fingerprint(input.evidence.target_vc());
    if &evidence_target == input.target_vc_fingerprint {
        Ok(())
    } else {
        Err(rejection_with_category(
            input.target_vc_fingerprint,
            RejectionCategory::CertificateRejection,
            RejectionDetail::ContextMismatch,
            RejectionLocation::new().with_field_path("target_vc"),
        ))
    }
}

fn validate_evidence_goal_polarity(
    input: KernelEvidenceCheckInput<'_>,
) -> KernelCheckServiceResult<()> {
    let required = input.check_kind.required_goal_polarity();
    if input.evidence.final_goal().polarity == required {
        Ok(())
    } else {
        Err(rejection_with_category(
            input.target_vc_fingerprint,
            RejectionCategory::CertificateRejection,
            RejectionDetail::ContextMismatch,
            RejectionLocation::new()
                .with_final_goal()
                .with_field_path("final_goal.polarity"),
        ))
    }
}

fn check_formula_imports(
    input: KernelEvidenceCheckInput<'_>,
) -> KernelCheckServiceResult<FormulaImportCheckReport> {
    if let Some(context) = input.formula_context {
        let actual = context
            .imported_axioms()
            .len()
            .saturating_add(context.imported_theorems().len());
        if actual > input.limits.formula_context.max_imported_context_entries {
            return Err(rejection(
                input.target_vc_fingerprint,
                RejectionDetail::ResourceExhaustion,
                RejectionLocation::new().with_field_path("formula_context.imported_context_count"),
            ));
        }
        if let Some(identity) = context.context_identity() {
            let actual = identity.entries().len();
            if actual > input.limits.formula_context.max_context_identity_entries {
                return Err(rejection(
                    input.target_vc_fingerprint,
                    RejectionDetail::ResourceExhaustion,
                    RejectionLocation::new()
                        .with_field_path("formula_context.context_identity.entries"),
                ));
            }
        }
    }

    let mut checked = BTreeMap::new();
    for formula in input.evidence.formulas() {
        match &formula.source {
            FormulaSource::AcceptedImportedAxiom(source) => {
                let import = check_formula_import_source(
                    input,
                    ImportedFactNamespace::ImportedAxiom,
                    source,
                )?;
                checked
                    .entry((
                        ImportedFactNamespace::ImportedAxiom,
                        import.imported_fact_id,
                    ))
                    .or_insert(import);
            }
            FormulaSource::AcceptedImportedTheorem(source) => {
                let import = check_formula_import_source(
                    input,
                    ImportedFactNamespace::ImportedTheorem,
                    source,
                )?;
                checked
                    .entry((
                        ImportedFactNamespace::ImportedTheorem,
                        import.imported_fact_id,
                    ))
                    .or_insert(import);
            }
            FormulaSource::LocalHypothesis { .. }
            | FormulaSource::CitedPremise { .. }
            | FormulaSource::GeneratedVcFact { .. }
            | FormulaSource::PolicyBoundedBuiltin { .. } => {}
        }
    }

    let checked_imports = checked.into_values().collect::<Vec<_>>();
    let used_axioms = checked_imports
        .iter()
        .map(|import| UsedAxiom {
            namespace: import.namespace,
            imported_fact_id: import.imported_fact_id,
            statement_fingerprint: import.statement_fingerprint.clone(),
        })
        .collect::<Vec<_>>();
    let policy_taint = checked_imports.iter().any(|import| import.policy_taint);

    Ok(FormulaImportCheckReport {
        checked_imports,
        used_axioms,
        policy_taint,
    })
}

fn check_formula_context_identity(
    input: KernelEvidenceCheckInput<'_>,
) -> KernelCheckServiceResult<()> {
    let required = input
        .evidence
        .formulas()
        .iter()
        .filter(|formula| context_identity_source(&formula.source).is_some())
        .collect::<Vec<_>>();
    if required.is_empty() {
        return Ok(());
    }

    let context = checked_formula_context(input)?;
    let Some(identity) = context.context_identity() else {
        return Err(rejection(
            input.target_vc_fingerprint,
            RejectionDetail::MissingProvenance,
            RejectionLocation::new().with_field_path("formula_context.context_identity"),
        ));
    };
    if identity.schema_version() != KERNEL_CONTEXT_IDENTITY_SCHEMA_VERSION {
        return Err(rejection(
            input.target_vc_fingerprint,
            RejectionDetail::MissingProvenance,
            RejectionLocation::new().with_field_path("formula_context.context_identity.schema"),
        ));
    }
    if identity.target_vc() != input.target_vc_fingerprint {
        return Err(rejection(
            input.target_vc_fingerprint,
            RejectionDetail::MissingProvenance,
            RejectionLocation::new().with_field_path("formula_context.context_identity.target_vc"),
        ));
    }
    validate_runtime_context_identity_limit(input, identity)?;
    if recompute_context_identity_hash(identity) != identity.context_identity_hash() {
        return Err(rejection(
            input.target_vc_fingerprint,
            RejectionDetail::MissingProvenance,
            RejectionLocation::new().with_field_path("formula_context.context_identity.hash"),
        ));
    }

    for formula in required {
        match matching_context_identity_entries(identity, formula).as_slice() {
            [_] => {}
            [] => {
                return Err(rejection(
                    input.target_vc_fingerprint,
                    RejectionDetail::MissingProvenance,
                    RejectionLocation::new().with_field_path("formula.context_identity"),
                ));
            }
            [_, ..] => {
                return Err(rejection(
                    input.target_vc_fingerprint,
                    RejectionDetail::MissingProvenance,
                    RejectionLocation::new().with_field_path("formula.context_identity"),
                ));
            }
        }
    }
    Ok(())
}

fn validate_runtime_context_identity_limit(
    input: KernelEvidenceCheckInput<'_>,
    identity: &KernelContextIdentityPayload,
) -> KernelCheckServiceResult<()> {
    if identity.entries().len() > input.limits.formula_context.max_context_identity_entries {
        return Err(rejection(
            input.target_vc_fingerprint,
            RejectionDetail::ResourceExhaustion,
            RejectionLocation::new().with_field_path("formula_context.context_identity.entries"),
        ));
    }
    Ok(())
}

fn matching_context_identity_entries<'a>(
    identity: &'a KernelContextIdentityPayload,
    formula: &crate::formula_evidence::FormulaEvidenceEntry,
) -> Vec<&'a KernelContextIdentityEntry> {
    let Some(source) = context_identity_source(&formula.source) else {
        return Vec::new();
    };
    identity
        .entries()
        .iter()
        .filter(|entry| {
            entry.source() == source
                && entry.formula_id() == formula.formula_id
                && entry.formula_fingerprint() == &formula.formula_fingerprint
        })
        .collect()
}

fn context_identity_source(source: &FormulaSource) -> Option<KernelContextIdentitySource> {
    match source {
        FormulaSource::LocalHypothesis { local_context_id } => {
            Some(KernelContextIdentitySource::LocalHypothesis {
                local_context_id: *local_context_id,
            })
        }
        FormulaSource::CitedPremise { local_context_id } => {
            Some(KernelContextIdentitySource::CitedPremise {
                local_context_id: *local_context_id,
            })
        }
        FormulaSource::GeneratedVcFact { vc_fact_id } => {
            Some(KernelContextIdentitySource::GeneratedVcFact {
                vc_fact_id: *vc_fact_id,
            })
        }
        FormulaSource::AcceptedImportedAxiom(_)
        | FormulaSource::AcceptedImportedTheorem(_)
        | FormulaSource::PolicyBoundedBuiltin { .. } => None,
    }
}

fn recompute_context_identity_hash(identity: &KernelContextIdentityPayload) -> Hash {
    stable_bridge_hash(
        KERNEL_CONTEXT_IDENTITY_HASH_DOMAIN,
        &context_identity_hash_input(identity),
    )
}

fn context_identity_hash_input(identity: &KernelContextIdentityPayload) -> Vec<u8> {
    let mut output = String::from("vc-kernel-context-identity-v1\n");
    writeln!(&mut output, "schema-version={}", identity.schema_version()).expect("write string");
    writeln!(
        &mut output,
        "target-vc={}",
        target_fingerprint_render(identity.target_vc())
    )
    .expect("write string");
    writeln!(
        &mut output,
        "canonical-evidence-hash={}",
        hex(identity.canonical_handoff_hash().as_bytes())
    )
    .expect("write string");
    writeln!(&mut output, "[entries]").expect("write string");
    for entry in identity.entries() {
        writeln!(
            &mut output,
            "source={}; formula-id={}; fingerprint={}; producer={}",
            context_identity_source_render(entry.source()),
            entry.formula_id(),
            fingerprint_render(entry.formula_fingerprint()),
            producer_ref_render(entry.producer_formula_ref())
        )
        .expect("write string");
    }
    output.into_bytes()
}

fn stable_bridge_hash(domain: &str, bytes: &[u8]) -> Hash {
    let mut lanes = [
        0x6d_69_7a_61_72_2d_76_63_u64,
        0x70_72_6f_6f_66_2d_69_64_u64,
        0x74_61_73_6b_32_30_2d_76_u64,
        0x66_69_6e_67_65_72_2d_31_u64,
    ];

    for (index, byte) in domain
        .as_bytes()
        .iter()
        .copied()
        .chain([0])
        .chain(bytes.iter().copied())
        .enumerate()
    {
        let lane = index % lanes.len();
        let mixed_index = (index as u64).rotate_left((lane as u32) + 1);
        lanes[lane] ^= u64::from(byte)
            .wrapping_add(0x9e37_79b9_7f4a_7c15)
            .wrapping_add(mixed_index);
        lanes[lane] = lanes[lane]
            .rotate_left(11 + lane as u32)
            .wrapping_mul(0x1000_0000_01b3);
    }

    lanes[0] ^= bytes.len() as u64;
    lanes[1] ^= (domain.len() as u64).rotate_left(17);
    lanes[2] ^= lanes[0].rotate_left(7);
    lanes[3] ^= lanes[1].rotate_left(13);

    let mut output = [0_u8; Hash::BYTE_LEN];
    for (chunk, lane) in output.chunks_exact_mut(8).zip(lanes) {
        chunk.copy_from_slice(&lane.to_be_bytes());
    }
    Hash::from_bytes(output)
}

fn context_identity_source_render(source: KernelContextIdentitySource) -> String {
    match source {
        KernelContextIdentitySource::LocalHypothesis { local_context_id } => {
            format!("LocalHypothesis {{ local_context_id: {local_context_id} }}")
        }
        KernelContextIdentitySource::CitedPremise { local_context_id } => {
            format!("CitedPremise {{ local_context_id: {local_context_id} }}")
        }
        KernelContextIdentitySource::GeneratedVcFact { vc_fact_id } => {
            format!("GeneratedVcFact {{ vc_fact_id: {vc_fact_id} }}")
        }
    }
}

fn producer_ref_render(producer: KernelFormulaProducerRef) -> String {
    match producer {
        KernelFormulaProducerRef::Core(formula) => {
            format!("Core(CoreFormulaId({}))", formula.index())
        }
        KernelFormulaProducerRef::Generated(formula) => {
            format!("Generated(VcGeneratedFormulaId({}))", formula.index())
        }
    }
}

fn target_fingerprint_render(fingerprint: &TargetVcFingerprint) -> String {
    format!("{}:{}", fingerprint.algorithm_id, hex(&fingerprint.digest))
}

fn fingerprint_render(fingerprint: &Fingerprint) -> String {
    format!("{}:{}", fingerprint.algorithm_id, hex(&fingerprint.digest))
}

fn hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

fn check_formula_import_source(
    input: KernelEvidenceCheckInput<'_>,
    namespace: ImportedFactNamespace,
    source: &ImportedFormulaSource,
) -> KernelCheckServiceResult<CheckedImportedFact> {
    let context = checked_formula_context(input)?;
    let evidence = lookup_formula_import(input.target_vc_fingerprint, context, namespace, source)?;
    let location = imported_location(evidence.imported_fact_id);
    validate_formula_import_status(input, source, evidence, location.clone())?;
    Ok(CheckedImportedFact {
        namespace,
        imported_fact_id: evidence.imported_fact_id,
        statement_fingerprint: evidence.statement_fingerprint.clone(),
        accepted_proof_status: evidence.accepted_proof_status,
        policy_taint: evidence.accepted_proof_status.policy_taint(),
    })
}

fn checked_formula_context<'a>(
    input: KernelEvidenceCheckInput<'a>,
) -> KernelCheckServiceResult<&'a FormulaEvidenceContext> {
    let location = RejectionLocation::new().with_field_path("formula_context");
    let Some(context) = input.formula_context else {
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
    Ok(context)
}

fn lookup_formula_import<'a>(
    target: &TargetVcFingerprint,
    context: &'a FormulaEvidenceContext,
    namespace: ImportedFactNamespace,
    source: &ImportedFormulaSource,
) -> KernelCheckServiceResult<&'a FormulaImportedFactEvidence> {
    let entries = match namespace {
        ImportedFactNamespace::ImportedAxiom => context.imported_axioms(),
        ImportedFactNamespace::ImportedTheorem => context.imported_theorems(),
    };
    let mut matches = entries
        .iter()
        .filter(|entry| formula_import_identity_matches(entry, source));
    let Some(first) = matches.next() else {
        return Err(rejection(
            target,
            RejectionDetail::UnresolvedSymbol,
            RejectionLocation::new().with_field_path("formula.imported_source"),
        ));
    };
    if matches.next().is_some() {
        return Err(rejection(
            target,
            RejectionDetail::UnresolvedSymbol,
            RejectionLocation::new()
                .with_imported_fact_id(first.imported_fact_id)
                .with_field_path("formula.imported_source"),
        ));
    }
    Ok(first)
}

fn formula_import_identity_matches(
    evidence: &FormulaImportedFactEvidence,
    source: &ImportedFormulaSource,
) -> bool {
    evidence.package_id == source.package_id
        && evidence.module_path == source.module_path
        && evidence.exported_item_id == source.exported_item_id
        && evidence.statement_fingerprint == source.statement_fingerprint
}

fn validate_formula_import_status(
    input: KernelEvidenceCheckInput<'_>,
    source: &ImportedFormulaSource,
    evidence: &FormulaImportedFactEvidence,
    location: RejectionLocation,
) -> KernelCheckServiceResult<()> {
    if evidence.accepted_proof_status == AcceptedProofStatus::ExternallyAttestedPolicyPermitted {
        if source.required_proof_status != RequiredProofStatus::ExternallyAttestedPolicyPermitted
            || !input.policy.imported_fact_policy.allow_externally_attested
        {
            return Err(rejection(
                input.target_vc_fingerprint,
                RejectionDetail::UnresolvedSymbol,
                location.with_field_path("formula.required_proof_status"),
            ));
        }
        return Ok(());
    }
    if evidence
        .accepted_proof_status
        .satisfies(source.required_proof_status)
    {
        return Ok(());
    }
    Err(rejection(
        input.target_vc_fingerprint,
        RejectionDetail::UnresolvedSymbol,
        location.with_field_path("formula.required_proof_status"),
    ))
}

fn check_kernel_certificate_inner(
    input: KernelCheckInput<'_>,
) -> KernelCheckServiceResult<KernelCheckResult> {
    if !input.policy.allow_legacy_certificate_audit {
        return Err(rejection_with_category(
            input.target_vc_fingerprint,
            RejectionCategory::CertificateRejection,
            RejectionDetail::UnsupportedCertificateFormat,
            RejectionLocation::new().with_field_path("policy.allow_legacy_certificate_audit"),
        ));
    }

    let mut budget = KernelPipelineBudget::new(input.limits.max_pipeline_steps);
    budget.step(input, "target_vc")?;
    validate_target_binding(input)?;

    budget.step(input, "imported_fact_context")?;
    let imported_report = check_imported_facts(ImportedFactCheckInput {
        target_vc_fingerprint: input.target_vc_fingerprint,
        certificate: input.certificate,
        imported_fact_context: input.imported_fact_context,
        policy: input.policy.imported_fact_policy,
        limits: input.limits.imported_facts,
    })?;

    budget.step(input, "substitution_context")?;
    let substitution_input = SubstitutionCheckInput {
        target_vc_fingerprint: input.target_vc_fingerprint,
        certificate: input.certificate,
        substitution_context: input.substitution_context,
        limits: input.limits.substitutions,
    };
    let substitution_report = replay_substitutions(substitution_input)?;
    let checked_substitutions =
        checked_substitutions_for_input(substitution_input, &substitution_report)?;

    budget.step(input, "resolution_trace")?;
    let resolution_input = ResolutionTraceInput {
        target_vc_fingerprint: input.target_vc_fingerprint,
        certificate: input.certificate,
        imported_clause_context: Some(imported_report.imported_clause_context()),
        limits: input.limits.resolution,
    };
    let resolution_report = replay_resolution_trace(resolution_input)?;

    budget.step(input, "cluster_trace_context")?;
    let checked_fact_context =
        checked_fact_context_for_cluster(input, &imported_report, &resolution_report)?;
    let cluster_report = replay_cluster_trace(ClusterTraceReplayInput {
        target_vc_fingerprint: input.target_vc_fingerprint,
        checked_fact_context: &checked_fact_context,
        cluster_trace_context: input.cluster_trace_context,
        requested_trace_steps: input.requested_cluster_trace_steps,
        limits: input.limits.cluster_trace,
    })?;

    budget.step(input, "derived_facts")?;
    validate_derived_facts(input)?;

    budget.step(input, "final_goal")?;
    let final_goal = checked_final_goal(input, resolution_input, &resolution_report)?;

    validate_report_record_count(
        input,
        ReportRecordCounts {
            imports: imported_report.checked_imports().len(),
            substitutions: checked_substitutions.len(),
            resolution_steps: resolution_report.checked_steps().len(),
            cluster_steps: cluster_report.checked_cluster_steps().len(),
            reduction_steps: cluster_report.checked_reduction_steps().len(),
            derived_facts: 0,
            used_axioms: 0,
            final_goals: 0,
            rejections: 1,
        },
    )?;
    let audit_rejection = *rejection_with_category(
        input.target_vc_fingerprint,
        RejectionCategory::CertificateRejection,
        RejectionDetail::UnsupportedCertificateFormat,
        final_goal.map_or_else(
            || RejectionLocation::new().with_field_path("certificate.final_goal"),
            legacy_final_goal_audit_location,
        ),
    );

    Ok(KernelCheckResult {
        target_vc_fingerprint: input.target_vc_fingerprint.clone(),
        status: KernelCheckStatus::Rejected,
        checked_imports: imported_report.checked_imports().to_vec(),
        checked_substitutions: checked_substitutions.to_vec(),
        checked_resolution_steps: resolution_report.checked_steps().to_vec(),
        checked_cluster_steps: cluster_report.checked_cluster_steps().to_vec(),
        checked_reduction_steps: cluster_report.checked_reduction_steps().to_vec(),
        checked_derived_facts: Vec::new(),
        sat_check_report: None,
        final_goal: None,
        evidence_check_kind: None,
        used_axioms: Vec::new(),
        policy_taint: imported_report.policy_taint(),
        rejections: vec![audit_rejection],
    })
}

fn legacy_final_goal_audit_location(final_goal: CheckedFinalGoal) -> RejectionLocation {
    let location = RejectionLocation::new().with_final_goal();
    match final_goal.namespace {
        FinalGoalNamespace::GeneratedClause => location.with_clause_ref(RejectionClauseRef::new(
            RejectionClauseRefNamespace::GeneratedClause,
            final_goal.id,
        )),
        FinalGoalNamespace::ResolutionStep => location.with_clause_ref(RejectionClauseRef::new(
            RejectionClauseRefNamespace::ResolutionStep,
            final_goal.id,
        )),
        FinalGoalNamespace::DerivedFact => location.with_derived_fact_id(final_goal.id),
    }
}

fn validate_target_binding(input: KernelCheckInput<'_>) -> KernelCheckServiceResult<()> {
    let certificate_target =
        TargetVcFingerprint::from_certificate_fingerprint(&input.certificate.target_vc);
    if &certificate_target == input.target_vc_fingerprint {
        Ok(())
    } else {
        Err(rejection_with_category(
            input.target_vc_fingerprint,
            RejectionCategory::CertificateRejection,
            RejectionDetail::ContextMismatch,
            RejectionLocation::new().with_field_path("target_vc"),
        ))
    }
}

fn checked_fact_context_for_cluster(
    input: KernelCheckInput<'_>,
    imported_report: &ImportedFactCheckReport,
    resolution_report: &ResolutionReplayReport<'_>,
) -> KernelCheckServiceResult<CheckedFactContext> {
    let imported_axioms = imported_report
        .checked_imports()
        .iter()
        .filter(|import| import.namespace == ImportedFactNamespace::ImportedAxiom)
        .map(|import| import.imported_fact_id)
        .collect();
    let imported_theorems = imported_report
        .checked_imports()
        .iter()
        .filter(|import| import.namespace == ImportedFactNamespace::ImportedTheorem)
        .map(|import| import.imported_fact_id)
        .collect();
    let generated_clauses = resolution_report
        .checked_steps()
        .iter()
        .map(|step| step.generated_clause_id)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    CheckedFactContext::new(imported_axioms, imported_theorems, generated_clauses).map_err(|_| {
        rejection(
            input.target_vc_fingerprint,
            RejectionDetail::InvalidSatProof,
            RejectionLocation::new().with_field_path("checked_fact_context"),
        )
    })
}

fn validate_derived_facts(input: KernelCheckInput<'_>) -> KernelCheckServiceResult<()> {
    if input.certificate.derived_facts.len() > input.limits.max_derived_facts {
        return Err(rejection(
            input.target_vc_fingerprint,
            RejectionDetail::ResourceExhaustion,
            RejectionLocation::new().with_field_path("derived_facts"),
        ));
    }
    if let Some(derived) = input.certificate.derived_facts.first() {
        return Err(rejection(
            input.target_vc_fingerprint,
            RejectionDetail::InvalidSatProof,
            RejectionLocation::new()
                .with_derived_fact_id(derived.derived_fact_id)
                .with_field_path("payload"),
        ));
    }
    Ok(())
}

fn checked_final_goal(
    input: KernelCheckInput<'_>,
    resolution_input: ResolutionTraceInput<'_>,
    resolution_report: &ResolutionReplayReport<'_>,
) -> KernelCheckServiceResult<Option<CheckedFinalGoal>> {
    if input.certificate.final_goal.namespace == FinalGoalNamespace::DerivedFact {
        checked_resolution_final_goal(resolution_input, resolution_report)?;
        return Err(rejection(
            input.target_vc_fingerprint,
            RejectionDetail::InvalidSatProof,
            RejectionLocation::new()
                .with_final_goal()
                .with_derived_fact_id(input.certificate.final_goal.id),
        ));
    }
    checked_resolution_final_goal(resolution_input, resolution_report)?;
    Ok(Some(CheckedFinalGoal {
        namespace: input.certificate.final_goal.namespace,
        id: input.certificate.final_goal.id,
    }))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ReportRecordCounts {
    imports: usize,
    substitutions: usize,
    resolution_steps: usize,
    cluster_steps: usize,
    reduction_steps: usize,
    derived_facts: usize,
    used_axioms: usize,
    final_goals: usize,
    rejections: usize,
}

fn validate_report_record_count(
    input: KernelCheckInput<'_>,
    counts: ReportRecordCounts,
) -> KernelCheckServiceResult<()> {
    validate_report_record_total(
        input.target_vc_fingerprint,
        input.limits.max_report_records,
        counts,
    )
}

fn validate_report_record_total(
    target: &TargetVcFingerprint,
    max_report_records: usize,
    counts: ReportRecordCounts,
) -> KernelCheckServiceResult<()> {
    let total = counts
        .imports
        .checked_add(counts.substitutions)
        .and_then(|value| value.checked_add(counts.resolution_steps))
        .and_then(|value| value.checked_add(counts.cluster_steps))
        .and_then(|value| value.checked_add(counts.reduction_steps))
        .and_then(|value| value.checked_add(counts.derived_facts))
        .and_then(|value| value.checked_add(counts.used_axioms))
        .and_then(|value| value.checked_add(counts.final_goals))
        .and_then(|value| value.checked_add(counts.rejections))
        .unwrap_or(usize::MAX);
    if total > max_report_records {
        return Err(rejection(
            target,
            RejectionDetail::ResourceExhaustion,
            RejectionLocation::new().with_field_path("checker_limits.max_report_records"),
        ));
    }
    Ok(())
}

fn rejected_kernel_result(
    target_vc_fingerprint: &TargetVcFingerprint,
    rejection: RejectionRecord,
) -> KernelCheckResult {
    KernelCheckResult {
        target_vc_fingerprint: target_vc_fingerprint.clone(),
        status: KernelCheckStatus::Rejected,
        checked_imports: Vec::new(),
        checked_substitutions: Vec::new(),
        checked_resolution_steps: Vec::new(),
        checked_cluster_steps: Vec::new(),
        checked_reduction_steps: Vec::new(),
        checked_derived_facts: Vec::new(),
        sat_check_report: None,
        final_goal: None,
        evidence_check_kind: None,
        used_axioms: Vec::new(),
        policy_taint: false,
        rejections: vec![rejection],
    }
}

struct KernelPipelineBudget {
    remaining: usize,
}

impl KernelPipelineBudget {
    const fn new(max_pipeline_steps: usize) -> Self {
        Self {
            remaining: max_pipeline_steps,
        }
    }

    fn step(
        &mut self,
        input: KernelCheckInput<'_>,
        field_path: &'static str,
    ) -> KernelCheckServiceResult<()> {
        self.step_target(input.target_vc_fingerprint, field_path)
    }

    fn step_target(
        &mut self,
        target: &TargetVcFingerprint,
        field_path: &'static str,
    ) -> KernelCheckServiceResult<()> {
        if self.remaining == 0 {
            return Err(rejection(
                target,
                RejectionDetail::Timeout,
                RejectionLocation::new().with_field_path(field_path),
            ));
        }
        self.remaining -= 1;
        Ok(())
    }
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
    imported_axioms: Vec<u32>,
    imported_theorems: Vec<u32>,
    generated_clauses: Vec<u32>,
}

impl CheckedFactContext {
    pub fn new(
        imported_axioms: Vec<u32>,
        imported_theorems: Vec<u32>,
        generated_clauses: Vec<u32>,
    ) -> Result<Self, ClusterTraceContextError> {
        Ok(Self {
            imported_axioms: canonical_ids(BaseFactNamespace::ImportedAxiom, imported_axioms)?,
            imported_theorems: canonical_ids(
                BaseFactNamespace::ImportedTheorem,
                imported_theorems,
            )?,
            generated_clauses: canonical_ids(
                BaseFactNamespace::GeneratedClause,
                generated_clauses,
            )?,
        })
    }

    #[must_use]
    pub fn imported_axioms(&self) -> &[u32] {
        &self.imported_axioms
    }

    #[must_use]
    pub fn imported_theorems(&self) -> &[u32] {
        &self.imported_theorems
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
#[non_exhaustive]
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
#[non_exhaustive]
pub enum BaseFactNamespace {
    ImportedAxiom,
    ImportedTheorem,
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
#[non_exhaustive]
pub enum CheckedFactRef {
    ImportedAxiom(u32),
    ImportedTheorem(u32),
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
        CheckedFactRef::ImportedAxiom(id) => input
            .checked_fact_context
            .imported_axioms()
            .binary_search(&id)
            .is_ok(),
        CheckedFactRef::ImportedTheorem(id) => input
            .checked_fact_context
            .imported_theorems()
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
        CheckedFactRef::ImportedAxiom(id) => {
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
        CheckedFactRef::ImportedTheorem(id) => {
            bytes.push(4);
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
mod tests;
