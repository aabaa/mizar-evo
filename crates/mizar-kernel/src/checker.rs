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
