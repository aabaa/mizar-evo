//! Producer-side handoff from VC IR to kernel formula/substitution evidence.
//!
//! This module implements the task-25 slice specified in
//! [kernel_evidence_handoff.md](../../../doc/design/mizar-vc/en/kernel_evidence_handoff.md).
//! It builds an immutable, prover-independent handoff package and does not call
//! the kernel, SAT checkers, or ATP backends.

use crate::{
    discharge::{DischargeEvidenceReplay, DischargeEvidenceSource, DischargeOutput},
    vc_ir::{
        CollectionLoopObligation, ContextEntry, ContextEntryId, ContextEntryKind,
        LoopInvariantPhase, PremiseRef, RangeLoopObligation, RegistrationCorrectnessKind,
        VcFormulaRef, VcGeneratedFormulaId, VcId, VcIr, VcKind, VcSet, VcText,
        stable_fingerprint_hash,
    },
};
use mizar_session::Hash;
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt::{self, Write as _},
};

pub const KERNEL_EVIDENCE_SCHEMA_VERSION: u16 = 1;
pub const KERNEL_EVIDENCE_ENCODING_VERSION: u16 = 1;
pub const VC_KERNEL_HANDOFF_SCHEMA: &str = "mizar-vc-kernel-evidence-handoff-v1";
pub const VC_TARGET_FINGERPRINT_ALGORITHM_ID: u8 = 18;
pub const KERNEL_FORMULA_FINGERPRINT_ALGORITHM_ID: u8 = 2;

#[derive(Debug, Clone, Copy)]
pub struct KernelEvidenceHandoffInput<'a> {
    pub vc_set: &'a VcSet,
    pub vc: VcId,
    pub goal_polarity: KernelGoalPolarity,
    pub kernel_profile: KernelEvidenceProfile,
    pub symbol_manifest: &'a [KernelManifestEntry],
    pub variable_manifest: &'a [KernelManifestEntry],
    pub formula_payloads: &'a [KernelFormulaPayload],
    pub imported_formula_payloads: &'a [KernelImportedFormulaPayload],
    pub substitutions: &'a [KernelSubstitutionPayload],
    pub formula_context: Option<&'a KernelFormulaContextRequirements>,
    pub discharge_output: Option<&'a DischargeOutput>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VcKernelEvidenceHandoff {
    canonical_evidence: KernelEvidenceEnvelope,
    formula_context_requirements: Option<KernelFormulaContextRequirements>,
    diagnostics: KernelEvidenceDiagnosticInputs,
    canonical_hash_input: Vec<u8>,
    canonical_hash: Hash,
}

impl VcKernelEvidenceHandoff {
    pub const fn canonical_evidence(&self) -> &KernelEvidenceEnvelope {
        &self.canonical_evidence
    }

    pub const fn formula_context_requirements(&self) -> Option<&KernelFormulaContextRequirements> {
        self.formula_context_requirements.as_ref()
    }

    pub const fn diagnostics(&self) -> &KernelEvidenceDiagnosticInputs {
        &self.diagnostics
    }

    pub fn canonical_hash_input(&self) -> &[u8] {
        &self.canonical_hash_input
    }

    pub const fn canonical_hash(&self) -> Hash {
        self.canonical_hash
    }

    pub fn targets_vc(&self, vc_set: &VcSet, vc: VcId) -> Result<bool, KernelEvidenceHandoffError> {
        Ok(self.canonical_evidence.target_vc() == &target_fingerprint(vc_set, vc)?)
    }

    pub fn debug_text(&self) -> String {
        let mut output = String::from("vc-kernel-evidence-handoff-debug-v1\n");
        writeln!(
            &mut output,
            "canonical-hash: {}",
            hex(self.canonical_hash.as_bytes())
        )
        .expect("write string");
        output.push_str(&self.canonical_evidence.debug_text());
        if let Some(context) = &self.formula_context_requirements {
            output.push_str(&context.debug_text());
        } else {
            writeln!(&mut output, "[formula-context-requirements]\n<none>").expect("write string");
        }
        output.push_str(&self.diagnostics.debug_text());
        output
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KernelEvidenceEnvelope {
    schema_version: u16,
    encoding_version: u16,
    target_vc: KernelEvidenceFingerprint,
    kernel_profile: KernelEvidenceProfile,
    symbol_manifest: Vec<KernelManifestEntry>,
    variable_manifest: Vec<KernelManifestEntry>,
    formula_evidence: Vec<KernelFormulaEvidenceEntry>,
    substitutions: Vec<KernelSubstitutionEvidence>,
    provenance: Vec<KernelEvidenceProvenance>,
    final_goal: KernelFinalGoalEvidence,
}

impl KernelEvidenceEnvelope {
    pub const fn schema_version(&self) -> u16 {
        self.schema_version
    }

    pub const fn encoding_version(&self) -> u16 {
        self.encoding_version
    }

    pub const fn target_vc(&self) -> &KernelEvidenceFingerprint {
        &self.target_vc
    }

    pub const fn kernel_profile(&self) -> KernelEvidenceProfile {
        self.kernel_profile
    }

    pub fn symbol_manifest(&self) -> &[KernelManifestEntry] {
        &self.symbol_manifest
    }

    pub fn variable_manifest(&self) -> &[KernelManifestEntry] {
        &self.variable_manifest
    }

    pub fn formula_evidence(&self) -> &[KernelFormulaEvidenceEntry] {
        &self.formula_evidence
    }

    pub fn substitutions(&self) -> &[KernelSubstitutionEvidence] {
        &self.substitutions
    }

    pub fn provenance(&self) -> &[KernelEvidenceProvenance] {
        &self.provenance
    }

    pub const fn final_goal(&self) -> &KernelFinalGoalEvidence {
        &self.final_goal
    }

    fn debug_text(&self) -> String {
        let mut output = String::from("[canonical-evidence]\n");
        writeln!(&mut output, "schema-version: {}", self.schema_version).expect("write string");
        writeln!(&mut output, "encoding-version: {}", self.encoding_version).expect("write string");
        writeln!(&mut output, "target-vc: {}", self.target_vc.render()).expect("write string");
        writeln!(&mut output, "kernel-profile: {:?}", self.kernel_profile).expect("write string");
        write_manifest(&mut output, "symbol-manifest", &self.symbol_manifest);
        write_manifest(&mut output, "variable-manifest", &self.variable_manifest);
        writeln!(&mut output, "[formula-evidence]").expect("write string");
        for formula in &self.formula_evidence {
            writeln!(
                &mut output,
                "formula {}: source={:?}; fingerprint={}; provenance={}; bytes={}",
                formula.formula_id,
                formula.source,
                formula.formula_fingerprint.render(),
                formula.provenance_id,
                hex(&formula.formula_bytes)
            )
            .expect("write string");
        }
        writeln!(&mut output, "[substitutions]").expect("write string");
        for substitution in &self.substitutions {
            writeln!(
                &mut output,
                "substitution {}: source-formula={}; provenance={}; binder={}; payload={}; freshness={:?}; free-vars={:?}",
                substitution.substitution_id,
                substitution.source_formula_id,
                substitution.provenance_id,
                hex(&substitution.binder_context_encoding),
                hex(&substitution.payload),
                substitution
                    .freshness_witnesses
                    .iter()
                    .map(|bytes| hex(bytes))
                    .collect::<Vec<_>>(),
                substitution
                    .free_variable_constraints
                    .iter()
                    .map(|bytes| hex(bytes))
                    .collect::<Vec<_>>()
            )
            .expect("write string");
        }
        writeln!(&mut output, "[provenance]").expect("write string");
        for provenance in &self.provenance {
            writeln!(
                &mut output,
                "provenance {}: target={}; formula={}; payload={}",
                provenance.provenance_id,
                provenance.target_vc.render(),
                provenance.formula_fingerprint.render(),
                hex(&provenance.payload)
            )
            .expect("write string");
        }
        writeln!(
            &mut output,
            "[final-goal]\npolarity={:?}; fingerprint={}; provenance={}; bytes={}",
            self.final_goal.polarity,
            self.final_goal.formula_fingerprint.render(),
            self.final_goal.provenance_id,
            hex(&self.final_goal.formula_bytes)
        )
        .expect("write string");
        output
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelEvidenceProfile {
    pub profile_id: u16,
    pub clause_schema_version: u16,
    pub clause_encoding_version: u16,
    pub clause_tautology_policy: KernelClauseTautologyPolicy,
    pub certificate_hash_input_algorithm: KernelCertificateHashInputAlgorithm,
}

impl KernelEvidenceProfile {
    pub const fn v1(profile_id: u16, clause_tautology_policy: KernelClauseTautologyPolicy) -> Self {
        Self {
            profile_id,
            clause_schema_version: 1,
            clause_encoding_version: 1,
            clause_tautology_policy,
            certificate_hash_input_algorithm:
                KernelCertificateHashInputAlgorithm::CanonicalEnvelopeV1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum KernelClauseTautologyPolicy {
    Reject,
    Marker,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum KernelCertificateHashInputAlgorithm {
    CanonicalEnvelopeV1,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KernelEvidenceFingerprint {
    pub algorithm_id: u8,
    pub digest: Vec<u8>,
}

impl KernelEvidenceFingerprint {
    pub fn new(algorithm_id: u8, digest: Vec<u8>) -> Result<Self, KernelEvidenceHandoffError> {
        if digest.is_empty() {
            return Err(KernelEvidenceHandoffError::EmptyFingerprint { algorithm_id });
        }
        Ok(Self {
            algorithm_id,
            digest,
        })
    }

    fn render(&self) -> String {
        format!("{}:{}", self.algorithm_id, hex(&self.digest))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KernelManifestEntry {
    pub entry_id: u32,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KernelFormulaPayload {
    pub formula_ref: VcFormulaRef,
    pub projection: KernelFormulaProjection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KernelFormulaProjection {
    pub formula_fingerprint: KernelEvidenceFingerprint,
    pub formula_bytes: Vec<u8>,
    pub provenance_payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KernelImportedFormulaPayload {
    pub symbol: VcText,
    pub class: KernelImportedFormulaClass,
    pub requirement: KernelImportedFactRequirement,
    pub projection: KernelFormulaProjection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum KernelImportedFormulaClass {
    Axiom,
    Theorem,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct KernelImportedFactRequirement {
    pub imported_fact_id: u32,
    pub package_id: Vec<u8>,
    pub module_path: Vec<u8>,
    pub exported_item_id: Vec<u8>,
    pub statement_fingerprint: KernelEvidenceFingerprint,
    pub required_proof_status: KernelRequiredProofStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum KernelRequiredProofStatus {
    KernelVerified,
    DischargedBuiltin,
    ExternallyAttestedPolicyPermitted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KernelFormulaContextRequirements {
    pub provenance_fingerprint: KernelEvidenceFingerprint,
    pub imported_axioms: Vec<KernelImportedFactRequirement>,
    pub imported_theorems: Vec<KernelImportedFactRequirement>,
}

impl KernelFormulaContextRequirements {
    fn debug_text(&self) -> String {
        let mut output = String::from("[formula-context-requirements]\n");
        writeln!(
            &mut output,
            "provenance-fingerprint: {}",
            self.provenance_fingerprint.render()
        )
        .expect("write string");
        write_import_requirements(&mut output, "imported-axioms", &self.imported_axioms);
        write_import_requirements(&mut output, "imported-theorems", &self.imported_theorems);
        output
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KernelSubstitutionPayload {
    pub substitution_id: u32,
    pub source_formula: VcFormulaRef,
    pub binder_context_encoding: Vec<u8>,
    pub payload: Vec<u8>,
    pub freshness_witnesses: Vec<Vec<u8>>,
    pub free_variable_constraints: Vec<Vec<u8>>,
    pub provenance_payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KernelFormulaEvidenceEntry {
    formula_id: u32,
    source: KernelFormulaSource,
    formula_fingerprint: KernelEvidenceFingerprint,
    formula_bytes: Vec<u8>,
    provenance_id: u32,
    producer_formula_ref: Option<VcFormulaRef>,
}

impl KernelFormulaEvidenceEntry {
    pub const fn formula_id(&self) -> u32 {
        self.formula_id
    }

    pub const fn source(&self) -> &KernelFormulaSource {
        &self.source
    }

    pub const fn formula_fingerprint(&self) -> &KernelEvidenceFingerprint {
        &self.formula_fingerprint
    }

    pub fn formula_bytes(&self) -> &[u8] {
        &self.formula_bytes
    }

    pub const fn provenance_id(&self) -> u32 {
        self.provenance_id
    }

    pub const fn producer_formula_ref(&self) -> Option<VcFormulaRef> {
        self.producer_formula_ref
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum KernelFormulaSource {
    LocalHypothesis { local_context_id: u32 },
    CitedPremise { local_context_id: u32 },
    GeneratedVcFact { vc_fact_id: u32 },
    AcceptedImportedAxiom(KernelImportedFactRequirement),
    AcceptedImportedTheorem(KernelImportedFactRequirement),
    PolicyBoundedBuiltin { built_in_id: Vec<u8> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KernelSubstitutionEvidence {
    pub substitution_id: u32,
    pub source_formula_id: u32,
    pub binder_context_encoding: Vec<u8>,
    pub payload: Vec<u8>,
    pub freshness_witnesses: Vec<Vec<u8>>,
    pub free_variable_constraints: Vec<Vec<u8>>,
    pub provenance_id: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KernelEvidenceProvenance {
    pub provenance_id: u32,
    pub target_vc: KernelEvidenceFingerprint,
    pub formula_fingerprint: KernelEvidenceFingerprint,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KernelFinalGoalEvidence {
    pub polarity: KernelGoalPolarity,
    pub formula_fingerprint: KernelEvidenceFingerprint,
    pub formula_bytes: Vec<u8>,
    pub provenance_id: u32,
    pub producer_formula_ref: VcFormulaRef,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum KernelGoalPolarity {
    AssertFalseForRefutation,
    AssertTrueForConsistency,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KernelEvidenceDiagnosticInputs {
    pub vc: VcId,
    pub local_context_fingerprint: Option<Hash>,
    pub discharge_records: Vec<KernelDischargeDiagnostic>,
}

impl KernelEvidenceDiagnosticInputs {
    fn debug_text(&self) -> String {
        let mut output = String::from("[diagnostic-inputs]\n");
        writeln!(&mut output, "vc: {:?}", self.vc).expect("write string");
        match self.local_context_fingerprint {
            Some(hash) => writeln!(
                &mut output,
                "local-context-fingerprint: {}",
                hex(hash.as_bytes())
            )
            .expect("write string"),
            None => writeln!(&mut output, "local-context-fingerprint: <unavailable>")
                .expect("write string"),
        }
        writeln!(&mut output, "[discharge-diagnostics]").expect("write string");
        for record in &self.discharge_records {
            writeln!(
                &mut output,
                "discharge {:?}: source={:?}; replay={:?}; goal={:?}; local-context={:?}; premises={:?}; generated={:?}",
                record.vc,
                record.source,
                record.replay,
                record.goal,
                record.local_context,
                record.premises,
                record.generated_formulas
            )
            .expect("write string");
        }
        output
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KernelDischargeDiagnostic {
    pub vc: VcId,
    pub source: DischargeEvidenceSource,
    pub replay: DischargeEvidenceReplay,
    pub goal: VcFormulaRef,
    pub local_context: Vec<ContextEntryId>,
    pub premises: Vec<PremiseRef>,
    pub generated_formulas: Vec<VcGeneratedFormulaId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum KernelEvidenceHandoffError {
    UnknownVc {
        vc: VcId,
    },
    MissingTargetBinding {
        vc: VcId,
    },
    GoalPolarityMismatch {
        vc: VcId,
        requested: KernelGoalPolarity,
        required: KernelGoalPolarity,
    },
    MissingFormulaPayload {
        formula: VcFormulaRef,
        role: KernelEvidenceRole,
    },
    DuplicateFormulaPayload {
        formula: VcFormulaRef,
    },
    EmptyFormulaPayload {
        role: KernelEvidenceRole,
    },
    EmptyProvenancePayload {
        role: KernelEvidenceRole,
    },
    UnsupportedFormulaFingerprintAlgorithm {
        algorithm_id: u8,
    },
    ImportedFormulaFingerprintMismatch {
        symbol: VcText,
    },
    EmptyFingerprint {
        algorithm_id: u8,
    },
    EmptyFormulaContextProvenance,
    DuplicateManifestEntry {
        entry_id: u32,
    },
    EmptyManifestEntry {
        entry_id: u32,
    },
    MissingLocalContextFormula {
        context: ContextEntryId,
    },
    MissingImportedFormulaPayload {
        symbol: VcText,
    },
    DuplicateImportedFormulaPayload {
        symbol: VcText,
    },
    MissingFormulaContext {
        symbol: VcText,
    },
    ImportedFormulaContextMismatch {
        symbol: VcText,
    },
    UnsupportedPremise {
        premise: PremiseRef,
        reason: VcText,
    },
    DuplicateSubstitution {
        substitution_id: u32,
    },
    MissingSubstitutionSource {
        substitution_id: u32,
        source_formula: VcFormulaRef,
    },
    EmptySubstitutionPayload {
        substitution_id: u32,
        field: &'static str,
    },
    EmptySubstitutionSideCondition {
        substitution_id: u32,
        field: &'static str,
    },
    DuplicateSubstitutionSideCondition {
        substitution_id: u32,
        field: &'static str,
    },
    IdOverflow {
        field: &'static str,
    },
    MismatchedDischargeOutput {
        vc: VcId,
    },
}

impl fmt::Display for KernelEvidenceHandoffError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownVc { vc } => write!(formatter, "unknown VC {vc:?}"),
            Self::MissingTargetBinding { vc } => {
                write!(formatter, "missing stable target binding for {vc:?}")
            }
            Self::GoalPolarityMismatch {
                vc,
                requested,
                required,
            } => write!(
                formatter,
                "goal polarity {requested:?} does not match required {required:?} for {vc:?}"
            ),
            Self::MissingFormulaPayload { formula, role } => {
                write!(
                    formatter,
                    "missing formula payload for {role:?} {formula:?}"
                )
            }
            Self::DuplicateFormulaPayload { formula } => {
                write!(formatter, "duplicate formula payload for {formula:?}")
            }
            Self::EmptyFormulaPayload { role } => {
                write!(formatter, "empty formula payload for {role:?}")
            }
            Self::EmptyProvenancePayload { role } => {
                write!(formatter, "empty provenance payload for {role:?}")
            }
            Self::UnsupportedFormulaFingerprintAlgorithm { algorithm_id } => write!(
                formatter,
                "unsupported formula fingerprint algorithm {algorithm_id}"
            ),
            Self::ImportedFormulaFingerprintMismatch { symbol } => write!(
                formatter,
                "imported formula payload for {} does not match its statement fingerprint",
                symbol.as_str()
            ),
            Self::EmptyFingerprint { algorithm_id } => {
                write!(
                    formatter,
                    "empty fingerprint digest for algorithm {algorithm_id}"
                )
            }
            Self::EmptyFormulaContextProvenance => {
                write!(formatter, "formula context provenance fingerprint is empty")
            }
            Self::DuplicateManifestEntry { entry_id } => {
                write!(formatter, "duplicate manifest entry {entry_id}")
            }
            Self::EmptyManifestEntry { entry_id } => {
                write!(formatter, "empty manifest entry {entry_id}")
            }
            Self::MissingLocalContextFormula { context } => {
                write!(formatter, "local context {context:?} has no formula")
            }
            Self::MissingImportedFormulaPayload { symbol } => {
                write!(
                    formatter,
                    "missing imported formula payload for {}",
                    symbol.as_str()
                )
            }
            Self::DuplicateImportedFormulaPayload { symbol } => {
                write!(
                    formatter,
                    "duplicate imported formula payload for {}",
                    symbol.as_str()
                )
            }
            Self::MissingFormulaContext { symbol } => {
                write!(formatter, "missing formula context for {}", symbol.as_str())
            }
            Self::ImportedFormulaContextMismatch { symbol } => {
                write!(
                    formatter,
                    "formula context does not contain {}",
                    symbol.as_str()
                )
            }
            Self::UnsupportedPremise { premise, reason } => {
                write!(
                    formatter,
                    "unsupported premise {premise:?}: {}",
                    reason.as_str()
                )
            }
            Self::DuplicateSubstitution { substitution_id } => {
                write!(formatter, "duplicate substitution {substitution_id}")
            }
            Self::MissingSubstitutionSource {
                substitution_id,
                source_formula,
            } => write!(
                formatter,
                "substitution {substitution_id} references missing source formula {source_formula:?}"
            ),
            Self::EmptySubstitutionPayload {
                substitution_id,
                field,
            } => write!(
                formatter,
                "substitution {substitution_id} has empty {field}"
            ),
            Self::EmptySubstitutionSideCondition {
                substitution_id,
                field,
            } => write!(
                formatter,
                "substitution {substitution_id} has empty side-condition record in {field}"
            ),
            Self::DuplicateSubstitutionSideCondition {
                substitution_id,
                field,
            } => write!(
                formatter,
                "substitution {substitution_id} has duplicate side-condition record in {field}"
            ),
            Self::IdOverflow { field } => write!(formatter, "{field} does not fit in u32"),
            Self::MismatchedDischargeOutput { vc } => {
                write!(formatter, "discharge output does not contain {vc:?}")
            }
        }
    }
}

impl Error for KernelEvidenceHandoffError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum KernelEvidenceRole {
    LocalContext,
    GeneratedPremise,
    ImportedPremise,
    FinalGoal,
    Substitution,
}

pub fn build_kernel_evidence_handoff(
    input: KernelEvidenceHandoffInput<'_>,
) -> Result<VcKernelEvidenceHandoff, KernelEvidenceHandoffError> {
    let vc = input
        .vc_set
        .vc(input.vc)
        .ok_or(KernelEvidenceHandoffError::UnknownVc { vc: input.vc })?;
    let required_goal_polarity = required_goal_polarity(vc);
    if input.goal_polarity != required_goal_polarity {
        return Err(KernelEvidenceHandoffError::GoalPolarityMismatch {
            vc: input.vc,
            requested: input.goal_polarity,
            required: required_goal_polarity,
        });
    }
    let target_vc = target_fingerprint(input.vc_set, input.vc)?;
    let symbol_manifest = sorted_manifest(input.symbol_manifest)?;
    let variable_manifest = sorted_manifest(input.variable_manifest)?;
    let formula_payloads = formula_payload_map(input.formula_payloads)?;
    let imported_payloads = imported_payload_map(input.imported_formula_payloads)?;
    let mut assembly = HandoffAssembly::new(target_vc.clone());

    for entry in vc.local_context.entries() {
        if let Some(formula) = entry.formula {
            let source = local_context_source(entry)?;
            let projection =
                formula_payload(&formula_payloads, formula, KernelEvidenceRole::LocalContext)?;
            assembly.add_formula_ref(formula, source, projection)?;
        }
    }

    for premise in sorted_premise_refs(&vc.premises) {
        add_premise(
            &mut assembly,
            vc,
            &premise,
            &formula_payloads,
            &imported_payloads,
            input.formula_context,
        )?;
    }

    let substitutions = build_substitutions(&mut assembly, input.substitutions)?;
    let final_goal = build_final_goal(
        &mut assembly,
        vc.goal,
        input.goal_polarity,
        &formula_payloads,
    )?;
    let formula_context_requirements =
        context_requirements(input.formula_context, &assembly.imported_symbols)?;
    let diagnostics = diagnostics(input.vc_set, input.vc, input.discharge_output)?;

    let canonical_evidence = KernelEvidenceEnvelope {
        schema_version: KERNEL_EVIDENCE_SCHEMA_VERSION,
        encoding_version: KERNEL_EVIDENCE_ENCODING_VERSION,
        target_vc,
        kernel_profile: input.kernel_profile,
        symbol_manifest,
        variable_manifest,
        formula_evidence: assembly.formulas,
        substitutions,
        provenance: assembly.provenance,
        final_goal,
    };
    let canonical_hash_input = canonical_hash_input(&canonical_evidence);
    let canonical_hash =
        stable_fingerprint_hash("mizar-vc-kernel-evidence-handoff", &canonical_hash_input);

    Ok(VcKernelEvidenceHandoff {
        canonical_evidence,
        formula_context_requirements,
        diagnostics,
        canonical_hash_input,
        canonical_hash,
    })
}

fn required_goal_polarity(vc: &VcIr) -> KernelGoalPolarity {
    match &vc.kind {
        VcKind::TheoremProofStep | VcKind::TerminalProofGoal | VcKind::DefinitionCorrectness => {
            KernelGoalPolarity::AssertFalseForRefutation
        }
        VcKind::RegistrationStyleCorrectness {
            style:
                RegistrationCorrectnessKind::Registration
                | RegistrationCorrectnessKind::Redefinition
                | RegistrationCorrectnessKind::Reduction
                | RegistrationCorrectnessKind::ExplicitCoreSeed,
        }
        | VcKind::CheckerInitial
        | VcKind::GeneratedNonEmptiness
        | VcKind::GeneratedSethood
        | VcKind::FraenkelMembershipAxiom
        | VcKind::AlgorithmPrecondition
        | VcKind::AlgorithmPostcondition
        | VcKind::CallPrecondition
        | VcKind::AlgorithmAssertion => KernelGoalPolarity::AssertFalseForRefutation,
        VcKind::LoopInvariant {
            phase:
                LoopInvariantPhase::Entry
                | LoopInvariantPhase::Preservation
                | LoopInvariantPhase::Break
                | LoopInvariantPhase::Continue
                | LoopInvariantPhase::Exit,
        }
        | VcKind::RangeLoop {
            obligation:
                RangeLoopObligation::PositiveStep
                | RangeLoopObligation::RangeBound
                | RangeLoopObligation::HiddenIndex,
        }
        | VcKind::CollectionLoop {
            obligation:
                CollectionLoopObligation::Finiteness | CollectionLoopObligation::OrderIndependence,
        }
        | VcKind::Termination
        | VcKind::PartialTermination
        | VcKind::GhostErasureSafety
        | VcKind::PolicyDeferredTraceability => KernelGoalPolarity::AssertFalseForRefutation,
    }
}

struct HandoffAssembly {
    target_vc: KernelEvidenceFingerprint,
    formulas: Vec<KernelFormulaEvidenceEntry>,
    provenance: Vec<KernelEvidenceProvenance>,
    formula_id_by_ref: BTreeMap<VcFormulaRef, u32>,
    formula_id_by_source: BTreeMap<String, u32>,
    imported_symbols: BTreeSet<VcText>,
    next_formula_id: u32,
    next_provenance_id: u32,
}

impl HandoffAssembly {
    fn new(target_vc: KernelEvidenceFingerprint) -> Self {
        Self {
            target_vc,
            formulas: Vec::new(),
            provenance: Vec::new(),
            formula_id_by_ref: BTreeMap::new(),
            formula_id_by_source: BTreeMap::new(),
            imported_symbols: BTreeSet::new(),
            next_formula_id: 1,
            next_provenance_id: 1,
        }
    }

    fn add_formula_ref(
        &mut self,
        formula_ref: VcFormulaRef,
        source: KernelFormulaSource,
        projection: &KernelFormulaProjection,
    ) -> Result<u32, KernelEvidenceHandoffError> {
        let role = role_for_formula_source(&source);
        validate_projection(projection, role)?;
        let source_key = format!("{source:?}");
        if let Some(formula_id) = self.formula_id_by_source.get(&source_key) {
            self.formula_id_by_ref
                .entry(formula_ref)
                .or_insert(*formula_id);
            return Ok(*formula_id);
        }
        let formula_id = self.next_formula_id;
        self.next_formula_id =
            self.next_formula_id
                .checked_add(1)
                .ok_or(KernelEvidenceHandoffError::IdOverflow {
                    field: "formula_id",
                })?;
        let provenance_id = self.add_provenance(
            &projection.formula_fingerprint,
            &projection.provenance_payload,
            role,
        )?;
        self.formulas.push(KernelFormulaEvidenceEntry {
            formula_id,
            source,
            formula_fingerprint: projection.formula_fingerprint.clone(),
            formula_bytes: projection.formula_bytes.clone(),
            provenance_id,
            producer_formula_ref: Some(formula_ref),
        });
        self.formula_id_by_source.insert(source_key, formula_id);
        self.formula_id_by_ref
            .entry(formula_ref)
            .or_insert(formula_id);
        Ok(formula_id)
    }

    fn add_imported_formula(
        &mut self,
        source: KernelFormulaSource,
        symbol: &VcText,
        projection: &KernelFormulaProjection,
    ) -> Result<u32, KernelEvidenceHandoffError> {
        let source_key = format!("{source:?}");
        if let Some(formula_id) = self.formula_id_by_source.get(&source_key) {
            self.imported_symbols.insert(symbol.clone());
            return Ok(*formula_id);
        }
        let formula_id = self.next_formula_id;
        self.next_formula_id =
            self.next_formula_id
                .checked_add(1)
                .ok_or(KernelEvidenceHandoffError::IdOverflow {
                    field: "formula_id",
                })?;
        let provenance_id = self.add_provenance(
            &projection.formula_fingerprint,
            &projection.provenance_payload,
            KernelEvidenceRole::ImportedPremise,
        )?;
        self.formulas.push(KernelFormulaEvidenceEntry {
            formula_id,
            source,
            formula_fingerprint: projection.formula_fingerprint.clone(),
            formula_bytes: projection.formula_bytes.clone(),
            provenance_id,
            producer_formula_ref: None,
        });
        self.formula_id_by_source.insert(source_key, formula_id);
        self.imported_symbols.insert(symbol.clone());
        Ok(formula_id)
    }

    fn add_provenance(
        &mut self,
        formula_fingerprint: &KernelEvidenceFingerprint,
        payload: &[u8],
        role: KernelEvidenceRole,
    ) -> Result<u32, KernelEvidenceHandoffError> {
        if payload.is_empty() {
            return Err(KernelEvidenceHandoffError::EmptyProvenancePayload { role });
        }
        let provenance_id = self.next_provenance_id;
        self.next_provenance_id = self.next_provenance_id.checked_add(1).ok_or(
            KernelEvidenceHandoffError::IdOverflow {
                field: "provenance_id",
            },
        )?;
        self.provenance.push(KernelEvidenceProvenance {
            provenance_id,
            target_vc: self.target_vc.clone(),
            formula_fingerprint: formula_fingerprint.clone(),
            payload: payload.to_vec(),
        });
        Ok(provenance_id)
    }
}

fn target_fingerprint(
    vc_set: &VcSet,
    vc: VcId,
) -> Result<KernelEvidenceFingerprint, KernelEvidenceHandoffError> {
    let vc_ir = vc_set
        .vc(vc)
        .ok_or(KernelEvidenceHandoffError::UnknownVc { vc })?;
    let mut payload = String::from("vc-kernel-evidence-target-v1\n");
    writeln!(
        &mut payload,
        "schema-version: {:?}",
        vc_set.schema_version()
    )
    .expect("write string");
    writeln!(&mut payload, "snapshot: {:?}", vc_set.snapshot()).expect("write string");
    writeln!(&mut payload, "source: {:?}", vc_set.source()).expect("write string");
    writeln!(&mut payload, "module: {:?}", vc_set.module()).expect("write string");
    writeln!(&mut payload, "vc: {:?}", vc_ir.id).expect("write string");
    writeln!(&mut payload, "kind: {:?}", vc_ir.kind).expect("write string");
    writeln!(&mut payload, "source-ref: {:?}", vc_ir.source).expect("write string");
    writeln!(&mut payload, "seed: {:?}", vc_ir.seed).expect("write string");
    writeln!(&mut payload, "anchor: {:?}", vc_ir.anchor).expect("write string");
    writeln!(
        &mut payload,
        "local-context: {:?}",
        vc_ir.local_context.entries()
    )
    .expect("write string");
    writeln!(
        &mut payload,
        "policy-inputs: {:?}",
        vc_ir.local_context.policy_inputs()
    )
    .expect("write string");
    writeln!(
        &mut payload,
        "premises: {:?}",
        sorted_premise_refs(&vc_ir.premises)
    )
    .expect("write string");
    writeln!(&mut payload, "goal: {:?}", vc_ir.goal).expect("write string");
    writeln!(
        &mut payload,
        "status-category: {:?}",
        target_status_category(&vc_ir.status)
    )
    .expect("write string");
    writeln!(&mut payload, "provenance: {:?}", vc_ir.provenance).expect("write string");
    writeln!(
        &mut payload,
        "generated-formulas: {:?}",
        vc_set.generated_formulas()
    )
    .expect("write string");
    let fingerprint =
        stable_fingerprint_hash("mizar-vc-kernel-evidence-target", payload.as_bytes());
    KernelEvidenceFingerprint::new(
        VC_TARGET_FINGERPRINT_ALGORITHM_ID,
        fingerprint.as_bytes().to_vec(),
    )
}

fn sorted_premise_refs(premises: &[PremiseRef]) -> Vec<PremiseRef> {
    let mut sorted = premises.to_vec();
    sorted.sort();
    sorted
}

#[derive(Debug, Clone, Copy)]
enum TargetStatusCategory {
    Open,
    NeedsAtp,
    Discharged,
    PolicyOpen,
    AssumedByPolicy,
    SkippedDueToInvalidInput,
    DeferredExternal,
    Error,
}

fn target_status_category(status: &crate::vc_ir::VcStatus) -> TargetStatusCategory {
    match status {
        crate::vc_ir::VcStatus::Open => TargetStatusCategory::Open,
        crate::vc_ir::VcStatus::NeedsAtp => TargetStatusCategory::NeedsAtp,
        crate::vc_ir::VcStatus::Discharged { .. } => TargetStatusCategory::Discharged,
        crate::vc_ir::VcStatus::PolicyOpen { .. } => TargetStatusCategory::PolicyOpen,
        crate::vc_ir::VcStatus::AssumedByPolicy { .. } => TargetStatusCategory::AssumedByPolicy,
        crate::vc_ir::VcStatus::SkippedDueToInvalidInput { .. } => {
            TargetStatusCategory::SkippedDueToInvalidInput
        }
        crate::vc_ir::VcStatus::DeferredExternal { .. } => TargetStatusCategory::DeferredExternal,
        crate::vc_ir::VcStatus::Error { .. } => TargetStatusCategory::Error,
    }
}

fn role_for_formula_source(source: &KernelFormulaSource) -> KernelEvidenceRole {
    match source {
        KernelFormulaSource::LocalHypothesis { .. } | KernelFormulaSource::CitedPremise { .. } => {
            KernelEvidenceRole::LocalContext
        }
        KernelFormulaSource::GeneratedVcFact { .. } => KernelEvidenceRole::GeneratedPremise,
        KernelFormulaSource::AcceptedImportedAxiom(_)
        | KernelFormulaSource::AcceptedImportedTheorem(_) => KernelEvidenceRole::ImportedPremise,
        KernelFormulaSource::PolicyBoundedBuiltin { .. } => KernelEvidenceRole::GeneratedPremise,
    }
}

fn sorted_manifest(
    entries: &[KernelManifestEntry],
) -> Result<Vec<KernelManifestEntry>, KernelEvidenceHandoffError> {
    let mut sorted = entries.to_vec();
    sorted.sort_by_key(|entry| entry.entry_id);
    for entry in &sorted {
        if entry.payload.is_empty() {
            return Err(KernelEvidenceHandoffError::EmptyManifestEntry {
                entry_id: entry.entry_id,
            });
        }
    }
    for pair in sorted.windows(2) {
        if pair[0].entry_id == pair[1].entry_id {
            return Err(KernelEvidenceHandoffError::DuplicateManifestEntry {
                entry_id: pair[0].entry_id,
            });
        }
    }
    Ok(sorted)
}

fn formula_payload_map(
    payloads: &[KernelFormulaPayload],
) -> Result<BTreeMap<VcFormulaRef, KernelFormulaProjection>, KernelEvidenceHandoffError> {
    let mut map = BTreeMap::new();
    for payload in payloads {
        validate_fingerprint(&payload.projection.formula_fingerprint)?;
        if map
            .insert(payload.formula_ref, payload.projection.clone())
            .is_some()
        {
            return Err(KernelEvidenceHandoffError::DuplicateFormulaPayload {
                formula: payload.formula_ref,
            });
        }
    }
    Ok(map)
}

fn imported_payload_map(
    payloads: &[KernelImportedFormulaPayload],
) -> Result<BTreeMap<VcText, KernelImportedFormulaPayload>, KernelEvidenceHandoffError> {
    let mut map = BTreeMap::new();
    for payload in payloads {
        validate_projection(&payload.projection, KernelEvidenceRole::ImportedPremise)?;
        validate_import_requirement(&payload.requirement)?;
        if payload.requirement.statement_fingerprint != payload.projection.formula_fingerprint {
            return Err(
                KernelEvidenceHandoffError::ImportedFormulaFingerprintMismatch {
                    symbol: payload.symbol.clone(),
                },
            );
        }
        if map
            .insert(payload.symbol.clone(), payload.clone())
            .is_some()
        {
            return Err(
                KernelEvidenceHandoffError::DuplicateImportedFormulaPayload {
                    symbol: payload.symbol.clone(),
                },
            );
        }
    }
    Ok(map)
}

fn validate_projection(
    projection: &KernelFormulaProjection,
    role: KernelEvidenceRole,
) -> Result<(), KernelEvidenceHandoffError> {
    validate_fingerprint(&projection.formula_fingerprint)?;
    if projection.formula_bytes.is_empty() {
        return Err(KernelEvidenceHandoffError::EmptyFormulaPayload { role });
    }
    if projection.provenance_payload.is_empty() {
        return Err(KernelEvidenceHandoffError::EmptyProvenancePayload { role });
    }
    Ok(())
}

fn validate_fingerprint(
    fingerprint: &KernelEvidenceFingerprint,
) -> Result<(), KernelEvidenceHandoffError> {
    if fingerprint.digest.is_empty() {
        return Err(KernelEvidenceHandoffError::EmptyFingerprint {
            algorithm_id: fingerprint.algorithm_id,
        });
    }
    if fingerprint.algorithm_id != KERNEL_FORMULA_FINGERPRINT_ALGORITHM_ID {
        return Err(
            KernelEvidenceHandoffError::UnsupportedFormulaFingerprintAlgorithm {
                algorithm_id: fingerprint.algorithm_id,
            },
        );
    }
    Ok(())
}

fn validate_import_requirement(
    requirement: &KernelImportedFactRequirement,
) -> Result<(), KernelEvidenceHandoffError> {
    if requirement.package_id.is_empty()
        || requirement.module_path.is_empty()
        || requirement.exported_item_id.is_empty()
    {
        return Err(KernelEvidenceHandoffError::EmptyFormulaPayload {
            role: KernelEvidenceRole::ImportedPremise,
        });
    }
    if requirement.statement_fingerprint.digest.is_empty() {
        return Err(KernelEvidenceHandoffError::EmptyFingerprint {
            algorithm_id: requirement.statement_fingerprint.algorithm_id,
        });
    }
    if requirement.statement_fingerprint.algorithm_id != KERNEL_FORMULA_FINGERPRINT_ALGORITHM_ID {
        return Err(
            KernelEvidenceHandoffError::UnsupportedFormulaFingerprintAlgorithm {
                algorithm_id: requirement.statement_fingerprint.algorithm_id,
            },
        );
    }
    Ok(())
}

fn formula_payload(
    payloads: &BTreeMap<VcFormulaRef, KernelFormulaProjection>,
    formula: VcFormulaRef,
    role: KernelEvidenceRole,
) -> Result<&KernelFormulaProjection, KernelEvidenceHandoffError> {
    payloads
        .get(&formula)
        .ok_or(KernelEvidenceHandoffError::MissingFormulaPayload { formula, role })
}

fn local_context_source(
    entry: &ContextEntry,
) -> Result<KernelFormulaSource, KernelEvidenceHandoffError> {
    let local_context_id = kernel_index(entry.id.index(), "local_context_id")?;
    if matches!(entry.kind, ContextEntryKind::CitedPremise) {
        Ok(KernelFormulaSource::CitedPremise { local_context_id })
    } else {
        Ok(KernelFormulaSource::LocalHypothesis { local_context_id })
    }
}

fn add_premise(
    assembly: &mut HandoffAssembly,
    vc: &VcIr,
    premise: &PremiseRef,
    formula_payloads: &BTreeMap<VcFormulaRef, KernelFormulaProjection>,
    imported_payloads: &BTreeMap<VcText, KernelImportedFormulaPayload>,
    formula_context: Option<&KernelFormulaContextRequirements>,
) -> Result<(), KernelEvidenceHandoffError> {
    match premise {
        PremiseRef::LocalContext(context_id) => {
            let entry = vc
                .local_context
                .entries()
                .iter()
                .find(|entry| entry.id == *context_id)
                .ok_or(KernelEvidenceHandoffError::MissingLocalContextFormula {
                    context: *context_id,
                })?;
            let formula =
                entry
                    .formula
                    .ok_or(KernelEvidenceHandoffError::MissingLocalContextFormula {
                        context: *context_id,
                    })?;
            let source = local_context_source(entry)?;
            let projection =
                formula_payload(formula_payloads, formula, KernelEvidenceRole::LocalContext)?;
            assembly.add_formula_ref(formula, source, projection)?;
            Ok(())
        }
        PremiseRef::GeneratedFact { formula } => {
            let VcFormulaRef::Generated(generated) = formula else {
                return Err(KernelEvidenceHandoffError::UnsupportedPremise {
                    premise: premise.clone(),
                    reason: VcText::new("generated fact premise is not a generated VC formula"),
                });
            };
            let source = KernelFormulaSource::GeneratedVcFact {
                vc_fact_id: kernel_index(generated.index(), "vc_fact_id")?,
            };
            let projection = formula_payload(
                formula_payloads,
                *formula,
                KernelEvidenceRole::GeneratedPremise,
            )?;
            assembly.add_formula_ref(*formula, source, projection)?;
            Ok(())
        }
        PremiseRef::ImportedFact { symbol } => {
            let payload = imported_payloads.get(symbol).ok_or_else(|| {
                KernelEvidenceHandoffError::MissingImportedFormulaPayload {
                    symbol: symbol.clone(),
                }
            })?;
            let context = formula_context.ok_or_else(|| {
                KernelEvidenceHandoffError::MissingFormulaContext {
                    symbol: symbol.clone(),
                }
            })?;
            if !context_contains_requirement(context, payload.class, &payload.requirement) {
                return Err(KernelEvidenceHandoffError::ImportedFormulaContextMismatch {
                    symbol: symbol.clone(),
                });
            }
            let source = match payload.class {
                KernelImportedFormulaClass::Axiom => {
                    KernelFormulaSource::AcceptedImportedAxiom(payload.requirement.clone())
                }
                KernelImportedFormulaClass::Theorem => {
                    KernelFormulaSource::AcceptedImportedTheorem(payload.requirement.clone())
                }
            };
            assembly.add_imported_formula(source, symbol, &payload.projection)?;
            Ok(())
        }
        PremiseRef::ConservativeUnknown { reason } => {
            Err(KernelEvidenceHandoffError::UnsupportedPremise {
                premise: premise.clone(),
                reason: reason.clone(),
            })
        }
        _ => Err(KernelEvidenceHandoffError::UnsupportedPremise {
            premise: premise.clone(),
            reason: VcText::new("premise has no explicit kernel formula payload binding"),
        }),
    }
}

fn context_contains_requirement(
    context: &KernelFormulaContextRequirements,
    class: KernelImportedFormulaClass,
    requirement: &KernelImportedFactRequirement,
) -> bool {
    let requirements = match class {
        KernelImportedFormulaClass::Axiom => &context.imported_axioms,
        KernelImportedFormulaClass::Theorem => &context.imported_theorems,
    };
    requirements.contains(requirement)
}

fn build_substitutions(
    assembly: &mut HandoffAssembly,
    substitutions: &[KernelSubstitutionPayload],
) -> Result<Vec<KernelSubstitutionEvidence>, KernelEvidenceHandoffError> {
    let mut sorted = substitutions.to_vec();
    sorted.sort_by_key(|substitution| substitution.substitution_id);
    for pair in sorted.windows(2) {
        if pair[0].substitution_id == pair[1].substitution_id {
            return Err(KernelEvidenceHandoffError::DuplicateSubstitution {
                substitution_id: pair[0].substitution_id,
            });
        }
    }

    let mut output = Vec::with_capacity(sorted.len());
    for substitution in sorted {
        if substitution.binder_context_encoding.is_empty() {
            return Err(KernelEvidenceHandoffError::EmptySubstitutionPayload {
                substitution_id: substitution.substitution_id,
                field: "binder_context_encoding",
            });
        }
        if substitution.payload.is_empty() {
            return Err(KernelEvidenceHandoffError::EmptySubstitutionPayload {
                substitution_id: substitution.substitution_id,
                field: "payload",
            });
        }
        if substitution.provenance_payload.is_empty() {
            return Err(KernelEvidenceHandoffError::EmptySubstitutionPayload {
                substitution_id: substitution.substitution_id,
                field: "provenance_payload",
            });
        }
        let source_formula_id = *assembly
            .formula_id_by_ref
            .get(&substitution.source_formula)
            .ok_or(KernelEvidenceHandoffError::MissingSubstitutionSource {
                substitution_id: substitution.substitution_id,
                source_formula: substitution.source_formula,
            })?;
        let source_fingerprint = assembly
            .formulas
            .iter()
            .find(|formula| formula.formula_id == source_formula_id)
            .expect("source formula id comes from formulas table")
            .formula_fingerprint
            .clone();
        let provenance_id = assembly.add_provenance(
            &source_fingerprint,
            &substitution.provenance_payload,
            KernelEvidenceRole::Substitution,
        )?;
        let freshness_witnesses = canonical_side_conditions(
            substitution.substitution_id,
            "freshness_witnesses",
            substitution.freshness_witnesses,
        )?;
        let free_variable_constraints = canonical_side_conditions(
            substitution.substitution_id,
            "free_variable_constraints",
            substitution.free_variable_constraints,
        )?;
        output.push(KernelSubstitutionEvidence {
            substitution_id: substitution.substitution_id,
            source_formula_id,
            binder_context_encoding: substitution.binder_context_encoding,
            payload: substitution.payload,
            freshness_witnesses,
            free_variable_constraints,
            provenance_id,
        });
    }

    Ok(output)
}

fn build_final_goal(
    assembly: &mut HandoffAssembly,
    goal: VcFormulaRef,
    polarity: KernelGoalPolarity,
    formula_payloads: &BTreeMap<VcFormulaRef, KernelFormulaProjection>,
) -> Result<KernelFinalGoalEvidence, KernelEvidenceHandoffError> {
    let projection = formula_payload(formula_payloads, goal, KernelEvidenceRole::FinalGoal)?;
    validate_projection(projection, KernelEvidenceRole::FinalGoal)?;
    let provenance_id = assembly.add_provenance(
        &projection.formula_fingerprint,
        &projection.provenance_payload,
        KernelEvidenceRole::FinalGoal,
    )?;
    Ok(KernelFinalGoalEvidence {
        polarity,
        formula_fingerprint: projection.formula_fingerprint.clone(),
        formula_bytes: projection.formula_bytes.clone(),
        provenance_id,
        producer_formula_ref: goal,
    })
}

fn context_requirements(
    context: Option<&KernelFormulaContextRequirements>,
    used_symbols: &BTreeSet<VcText>,
) -> Result<Option<KernelFormulaContextRequirements>, KernelEvidenceHandoffError> {
    if used_symbols.is_empty() {
        return Ok(None);
    }
    let context = context.expect("used imported symbols require formula context");
    canonical_context_requirements(context).map(Some)
}

fn canonical_context_requirements(
    context: &KernelFormulaContextRequirements,
) -> Result<KernelFormulaContextRequirements, KernelEvidenceHandoffError> {
    if context.provenance_fingerprint.digest.is_empty() {
        return Err(KernelEvidenceHandoffError::EmptyFormulaContextProvenance);
    }
    let mut imported_axioms = canonical_import_requirements(&context.imported_axioms)?;
    let mut imported_theorems = canonical_import_requirements(&context.imported_theorems)?;
    imported_axioms.sort();
    imported_axioms.dedup();
    imported_theorems.sort();
    imported_theorems.dedup();
    Ok(KernelFormulaContextRequirements {
        provenance_fingerprint: context.provenance_fingerprint.clone(),
        imported_axioms,
        imported_theorems,
    })
}

fn canonical_import_requirements(
    requirements: &[KernelImportedFactRequirement],
) -> Result<Vec<KernelImportedFactRequirement>, KernelEvidenceHandoffError> {
    for requirement in requirements {
        validate_import_requirement(requirement)?;
    }
    Ok(requirements.to_vec())
}

fn canonical_side_conditions(
    substitution_id: u32,
    field: &'static str,
    mut records: Vec<Vec<u8>>,
) -> Result<Vec<Vec<u8>>, KernelEvidenceHandoffError> {
    records.sort();
    for record in &records {
        if record.is_empty() {
            return Err(KernelEvidenceHandoffError::EmptySubstitutionSideCondition {
                substitution_id,
                field,
            });
        }
    }
    for pair in records.windows(2) {
        if pair[0] == pair[1] {
            return Err(
                KernelEvidenceHandoffError::DuplicateSubstitutionSideCondition {
                    substitution_id,
                    field,
                },
            );
        }
    }
    Ok(records)
}

fn diagnostics(
    vc_set: &VcSet,
    vc: VcId,
    discharge_output: Option<&DischargeOutput>,
) -> Result<KernelEvidenceDiagnosticInputs, KernelEvidenceHandoffError> {
    let local_context_fingerprint = vc_set
        .local_context_fingerprint(vc)
        .map(|fingerprint| fingerprint.hash());
    let discharge_records = if let Some(output) = discharge_output {
        if !discharge_output_matches_input(output.vc_set(), vc_set) {
            return Err(KernelEvidenceHandoffError::MismatchedDischargeOutput { vc });
        }
        output
            .evidence_records()
            .iter()
            .filter(|record| record.vc == vc)
            .map(|record| KernelDischargeDiagnostic {
                vc: record.vc,
                source: record.source,
                replay: record.replay.clone(),
                goal: record.inputs.goal,
                local_context: record.inputs.local_context.clone(),
                premises: record.inputs.premises.clone(),
                generated_formulas: record.inputs.generated_formulas.clone(),
            })
            .collect()
    } else {
        Vec::new()
    };
    Ok(KernelEvidenceDiagnosticInputs {
        vc,
        local_context_fingerprint,
        discharge_records,
    })
}

fn discharge_output_matches_input(output_set: &VcSet, input_set: &VcSet) -> bool {
    output_set.schema_version() == input_set.schema_version()
        && output_set.snapshot() == input_set.snapshot()
        && output_set.source() == input_set.source()
        && output_set.module() == input_set.module()
        && output_set.generated_formulas() == input_set.generated_formulas()
        && output_set.seed_accounting() == input_set.seed_accounting()
        && output_set.vcs().len() == input_set.vcs().len()
        && output_set
            .vcs()
            .iter()
            .zip(input_set.vcs())
            .all(|(output_vc, input_vc)| discharge_vc_matches_input(output_vc, input_vc))
}

fn discharge_vc_matches_input(output_vc: &VcIr, input_vc: &VcIr) -> bool {
    output_vc.id == input_vc.id
        && output_vc.kind == input_vc.kind
        && output_vc.source == input_vc.source
        && output_vc.seed == input_vc.seed
        && output_vc.anchor == input_vc.anchor
        && output_vc.local_context == input_vc.local_context
        && output_vc.premises == input_vc.premises
        && output_vc.goal == input_vc.goal
        && output_vc.proof_hint == input_vc.proof_hint
}

fn canonical_hash_input(envelope: &KernelEvidenceEnvelope) -> Vec<u8> {
    let mut output = String::from("vc-kernel-evidence-handoff-canonical-v1\n");
    writeln!(&mut output, "schema-version={}", envelope.schema_version).expect("write string");
    writeln!(
        &mut output,
        "encoding-version={}",
        envelope.encoding_version
    )
    .expect("write string");
    writeln!(&mut output, "target-vc={}", envelope.target_vc.render()).expect("write string");
    writeln!(&mut output, "kernel-profile={:?}", envelope.kernel_profile).expect("write string");
    write_manifest(&mut output, "symbol-manifest", &envelope.symbol_manifest);
    write_manifest(
        &mut output,
        "variable-manifest",
        &envelope.variable_manifest,
    );
    writeln!(&mut output, "[formula-evidence]").expect("write string");
    for formula in &envelope.formula_evidence {
        writeln!(
            &mut output,
            "formula-id={}; source={:?}; fingerprint={}; provenance={}; formula-bytes={}",
            formula.formula_id,
            formula.source,
            formula.formula_fingerprint.render(),
            formula.provenance_id,
            hex(&formula.formula_bytes)
        )
        .expect("write string");
    }
    writeln!(&mut output, "[substitutions]").expect("write string");
    for substitution in &envelope.substitutions {
        writeln!(
            &mut output,
            "substitution-id={}; source-formula-id={}; binder={}; payload={}; freshness={}; free-vars={}; provenance={}",
            substitution.substitution_id,
            substitution.source_formula_id,
            hex(&substitution.binder_context_encoding),
            hex(&substitution.payload),
            joined_hex(&substitution.freshness_witnesses),
            joined_hex(&substitution.free_variable_constraints),
            substitution.provenance_id
        )
        .expect("write string");
    }
    writeln!(&mut output, "[provenance]").expect("write string");
    for provenance in &envelope.provenance {
        writeln!(
            &mut output,
            "provenance-id={}; target={}; formula={}; payload={}",
            provenance.provenance_id,
            provenance.target_vc.render(),
            provenance.formula_fingerprint.render(),
            hex(&provenance.payload)
        )
        .expect("write string");
    }
    writeln!(
        &mut output,
        "[final-goal]\npolarity={:?}; fingerprint={}; provenance={}; formula-bytes={}",
        envelope.final_goal.polarity,
        envelope.final_goal.formula_fingerprint.render(),
        envelope.final_goal.provenance_id,
        hex(&envelope.final_goal.formula_bytes)
    )
    .expect("write string");
    output.into_bytes()
}

fn write_manifest(output: &mut String, label: &str, entries: &[KernelManifestEntry]) {
    writeln!(output, "[{label}]").expect("write string");
    for entry in entries {
        writeln!(output, "entry {}: {}", entry.entry_id, hex(&entry.payload))
            .expect("write string");
    }
}

fn write_import_requirements(
    output: &mut String,
    label: &str,
    entries: &[KernelImportedFactRequirement],
) {
    writeln!(output, "[{label}]").expect("write string");
    for entry in entries {
        writeln!(
            output,
            "import {}: package={}; module={}; item={}; statement={}; required={:?}",
            entry.imported_fact_id,
            hex(&entry.package_id),
            hex(&entry.module_path),
            hex(&entry.exported_item_id),
            entry.statement_fingerprint.render(),
            entry.required_proof_status
        )
        .expect("write string");
    }
}

fn kernel_index(index: usize, field: &'static str) -> Result<u32, KernelEvidenceHandoffError> {
    let one_based = index
        .checked_add(1)
        .ok_or(KernelEvidenceHandoffError::IdOverflow { field })?;
    u32::try_from(one_based).map_err(|_| KernelEvidenceHandoffError::IdOverflow { field })
}

fn joined_hex(items: &[Vec<u8>]) -> String {
    items
        .iter()
        .map(|bytes| hex(bytes))
        .collect::<Vec<_>>()
        .join(",")
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        discharge::{DischargePolicy, try_discharge},
        vc_ir::{
            AnchorCompleteness, AnchorIngredient, AnchorLabel, AnchorLabelRole,
            AnchorUnavailableReason, CanonicalSortKey, CollectionLoopObligation,
            DefinitionOpacityOverride, DefinitionUnfoldRequest, GenerationSchemaVersion,
            HashMarker, LocalContext, LoopInvariantPhase, RangeLoopObligation,
            RegistrationCorrectnessKind, SeedAccounting, SeedOriginRef, SeedVcMapping, SeedVcRef,
            VcGeneratedFormula, VcGeneratedFormulaKind, VcGeneratedFormulaShape, VcKind,
            VcModuleRef, VcProvenance, VcProvenancePhase, VcSchemaVersion, VcSetParts, VcSourceRef,
            VcStatus,
        },
    };
    use mizar_core::{
        control_flow::ObligationHandoffId,
        core_ir::{
            CoreDefinitionId, CoreItemId, CoreLabelRef, CoreProvenance, CoreProvenancePhase,
            CoreSourceRef, LocalProofOrProgramPath, NormalizedSemanticOrigin, ObligationSeedId,
            ObligationSeedStatus,
        },
    };
    use mizar_session::{
        BuildSnapshotId, InMemorySessionIdAllocator, SessionIdAllocator, SourceId, SourceRange,
    };

    type SubstitutionMutation = fn(&mut KernelSubstitutionPayload);

    #[test]
    fn builds_deterministic_handoff_for_local_context_and_goal() {
        let set = fixture_set(FixtureShape::LocalContextPremise);
        let input = handoff_input(&set, &[], None, None);

        let first = build_kernel_evidence_handoff(input).expect("handoff");
        let second = build_kernel_evidence_handoff(input).expect("handoff");

        assert_eq!(first.canonical_hash_input(), second.canonical_hash_input());
        assert_eq!(first.canonical_hash(), second.canonical_hash());
        assert_eq!(first.debug_text(), second.debug_text());
        assert_eq!(first.canonical_evidence().schema_version(), 1);
        assert_eq!(first.canonical_evidence().encoding_version(), 1);
        assert_eq!(first.canonical_evidence().formula_evidence().len(), 1);
        assert!(matches!(
            first.canonical_evidence().formula_evidence()[0].source(),
            KernelFormulaSource::LocalHypothesis {
                local_context_id: 1
            }
        ));
        assert_eq!(
            first.canonical_evidence().final_goal().producer_formula_ref,
            VcFormulaRef::Generated(VcGeneratedFormulaId::new(1))
        );
        assert_eq!(
            first.canonical_evidence().final_goal().polarity,
            KernelGoalPolarity::AssertFalseForRefutation
        );
        assert!(
            first
                .debug_text()
                .contains("vc-kernel-evidence-handoff-debug-v1")
        );
    }

    #[test]
    fn generated_premises_and_provenance_are_mapped() {
        let set = fixture_set(FixtureShape::GeneratedPremise);
        let handoff =
            build_kernel_evidence_handoff(handoff_input(&set, &[], None, None)).expect("handoff");
        let evidence = handoff.canonical_evidence();

        assert_eq!(
            evidence.target_vc().algorithm_id,
            VC_TARGET_FINGERPRINT_ALGORITHM_ID
        );
        assert!(!evidence.target_vc().digest.is_empty());
        assert_eq!(evidence.formula_evidence().len(), 1);
        let premise = &evidence.formula_evidence()[0];
        assert_eq!(premise.formula_id(), 1);
        assert!(matches!(
            premise.source(),
            KernelFormulaSource::GeneratedVcFact { vc_fact_id: 1 }
        ));
        assert_eq!(
            premise.formula_fingerprint(),
            &fingerprint(KERNEL_FORMULA_FINGERPRINT_ALGORITHM_ID, b"formula-0")
        );
        assert_eq!(premise.formula_bytes(), b"kernel-formula-0");
        assert_eq!(premise.provenance_id(), 1);

        assert_eq!(evidence.provenance().len(), 2);
        assert_eq!(evidence.provenance()[0].provenance_id, 1);
        assert_eq!(evidence.provenance()[0].target_vc, *evidence.target_vc());
        assert_eq!(
            evidence.provenance()[0].formula_fingerprint,
            *premise.formula_fingerprint()
        );
        assert_eq!(evidence.provenance()[0].payload, b"provenance-0");
        assert_eq!(evidence.final_goal().provenance_id, 2);
        assert_eq!(
            evidence.final_goal().formula_fingerprint,
            fingerprint(KERNEL_FORMULA_FINGERPRINT_ALGORITHM_ID, b"formula-1")
        );
        assert_eq!(evidence.provenance()[1].payload, b"provenance-1");
    }

    #[test]
    fn canonical_handoff_sorts_manifest_and_substitution_inputs() {
        let set = fixture_set(FixtureShape::LocalContextPremise);
        let symbols_unsorted = vec![
            KernelManifestEntry {
                entry_id: 2,
                payload: b"symbol-b".to_vec(),
            },
            KernelManifestEntry {
                entry_id: 1,
                payload: b"symbol-a".to_vec(),
            },
        ];
        let symbols_sorted = vec![
            KernelManifestEntry {
                entry_id: 1,
                payload: b"symbol-a".to_vec(),
            },
            KernelManifestEntry {
                entry_id: 2,
                payload: b"symbol-b".to_vec(),
            },
        ];
        let variables_unsorted = vec![
            KernelManifestEntry {
                entry_id: 2,
                payload: b"variable-b".to_vec(),
            },
            KernelManifestEntry {
                entry_id: 1,
                payload: b"variable-a".to_vec(),
            },
        ];
        let variables_sorted = vec![
            KernelManifestEntry {
                entry_id: 1,
                payload: b"variable-a".to_vec(),
            },
            KernelManifestEntry {
                entry_id: 2,
                payload: b"variable-b".to_vec(),
            },
        ];
        let substitutions_unsorted = vec![
            substitution_payload(9, VcFormulaRef::Generated(VcGeneratedFormulaId::new(0))),
            substitution_payload_with_side_conditions(
                7,
                VcFormulaRef::Generated(VcGeneratedFormulaId::new(0)),
                vec![b"fresh-z".to_vec(), b"fresh-a".to_vec()],
                vec![b"free-z".to_vec(), b"free-a".to_vec()],
            ),
        ];
        let substitutions_sorted = vec![
            substitution_payload_with_side_conditions(
                7,
                VcFormulaRef::Generated(VcGeneratedFormulaId::new(0)),
                vec![b"fresh-a".to_vec(), b"fresh-z".to_vec()],
                vec![b"free-a".to_vec(), b"free-z".to_vec()],
            ),
            substitution_payload(9, VcFormulaRef::Generated(VcGeneratedFormulaId::new(0))),
        ];

        let unsorted = build_kernel_evidence_handoff(KernelEvidenceHandoffInput {
            symbol_manifest: &symbols_unsorted,
            variable_manifest: &variables_unsorted,
            substitutions: &substitutions_unsorted,
            ..handoff_input(&set, &[], None, None)
        })
        .expect("unsorted handoff");
        let sorted = build_kernel_evidence_handoff(KernelEvidenceHandoffInput {
            symbol_manifest: &symbols_sorted,
            variable_manifest: &variables_sorted,
            substitutions: &substitutions_sorted,
            ..handoff_input(&set, &[], None, None)
        })
        .expect("sorted handoff");

        assert_eq!(
            unsorted.canonical_hash_input(),
            sorted.canonical_hash_input()
        );
        assert_eq!(unsorted.canonical_hash(), sorted.canonical_hash());
        assert_eq!(
            unsorted
                .canonical_evidence()
                .symbol_manifest()
                .iter()
                .map(|entry| entry.entry_id)
                .collect::<Vec<_>>(),
            [1, 2]
        );
        assert_eq!(
            unsorted
                .canonical_evidence()
                .variable_manifest()
                .iter()
                .map(|entry| entry.entry_id)
                .collect::<Vec<_>>(),
            [1, 2]
        );
        assert_eq!(
            unsorted
                .canonical_evidence()
                .substitutions()
                .iter()
                .map(|substitution| substitution.substitution_id)
                .collect::<Vec<_>>(),
            [7, 9]
        );
    }

    #[test]
    fn canonical_handoff_sorts_premises_and_excludes_status_evidence() {
        let left = fixture_set(FixtureShape::TwoPremisesForward);
        let right = fixture_set(FixtureShape::TwoPremisesReverse);

        let left_handoff =
            build_kernel_evidence_handoff(handoff_input(&left, &[], None, None)).expect("left");
        let right_handoff =
            build_kernel_evidence_handoff(handoff_input(&right, &[], None, None)).expect("right");

        assert_eq!(
            left_handoff.canonical_hash_input(),
            right_handoff.canonical_hash_input()
        );
        assert_eq!(
            left_handoff.canonical_hash(),
            right_handoff.canonical_hash()
        );
        assert_eq!(
            left_handoff
                .canonical_evidence()
                .formula_evidence()
                .iter()
                .map(|formula| formula.formula_id())
                .collect::<Vec<_>>(),
            [1, 2]
        );

        let discharged = fixture_set(FixtureShape::DischargedWithEvidence);
        let handoff = build_kernel_evidence_handoff(handoff_input(&discharged, &[], None, None))
            .expect("discharged handoff");
        let canonical = String::from_utf8_lossy(handoff.canonical_hash_input());
        assert!(!canonical.contains("discharge-rule"));
        assert!(!canonical.contains("discharge-evidence"));
    }

    #[test]
    fn proof_hints_and_discharge_evidence_do_not_change_target_identity() {
        let no_hint = fixture_set(FixtureShape::GoalOneNoHint);
        let no_hint_handoff =
            build_kernel_evidence_handoff(handoff_input(&no_hint, &[], None, None))
                .expect("no hint handoff");

        for shape in [
            FixtureShape::HintOnlyCitation,
            FixtureShape::UnsupportedHintMetadata,
        ] {
            let hinted = fixture_set(shape);
            let hinted_handoff =
                build_kernel_evidence_handoff(handoff_input(&hinted, &[], None, None))
                    .expect("hinted handoff");

            assert_eq!(
                hinted_handoff.canonical_evidence().target_vc(),
                no_hint_handoff.canonical_evidence().target_vc()
            );
            assert_eq!(
                hinted_handoff.canonical_hash_input(),
                no_hint_handoff.canonical_hash_input()
            );
            assert_eq!(
                hinted_handoff.canonical_hash(),
                no_hint_handoff.canonical_hash()
            );
        }

        let discharged = fixture_set(FixtureShape::DischargedWithEvidence);
        let alternate = fixture_set(FixtureShape::DischargedWithAlternateEvidence);
        let discharged_handoff =
            build_kernel_evidence_handoff(handoff_input(&discharged, &[], None, None))
                .expect("discharged handoff");
        let alternate_handoff =
            build_kernel_evidence_handoff(handoff_input(&alternate, &[], None, None))
                .expect("alternate discharged handoff");

        assert_eq!(
            discharged_handoff.canonical_evidence().target_vc(),
            alternate_handoff.canonical_evidence().target_vc()
        );
        assert_eq!(
            discharged_handoff.canonical_hash_input(),
            alternate_handoff.canonical_hash_input()
        );
        assert_eq!(
            discharged_handoff.canonical_hash(),
            alternate_handoff.canonical_hash()
        );
    }

    #[test]
    fn proof_hint_metadata_does_not_block_target_binding() {
        let set = fixture_set(FixtureShape::UnsupportedHintMetadata);
        let handoff =
            build_kernel_evidence_handoff(handoff_input(&set, &[], None, None)).expect("handoff");

        assert!(handoff.canonical_evidence().formula_evidence().is_empty());
        assert_eq!(
            handoff
                .canonical_evidence()
                .final_goal()
                .producer_formula_ref,
            VcFormulaRef::Generated(VcGeneratedFormulaId::new(1))
        );
        let canonical = String::from_utf8_lossy(handoff.canonical_hash_input());
        assert!(!canonical.contains("unfold"));
        assert!(!canonical.contains("hint-only"));
    }

    #[test]
    fn missing_formula_payload_fails_closed() {
        let set = fixture_set(FixtureShape::GeneratedGoalOnly);
        let payloads = Vec::new();
        let input = KernelEvidenceHandoffInput {
            formula_payloads: &payloads,
            ..handoff_input(&set, &[], None, None)
        };

        let error = build_kernel_evidence_handoff(input).expect_err("missing payload");

        assert!(matches!(
            error,
            KernelEvidenceHandoffError::MissingFormulaPayload {
                formula: VcFormulaRef::Generated(_),
                role: KernelEvidenceRole::FinalGoal
            }
        ));
    }

    #[test]
    fn consistency_goal_polarity_for_proof_obligation_fails_closed() {
        let set = fixture_set(FixtureShape::GeneratedGoalOnly);
        let input = KernelEvidenceHandoffInput {
            goal_polarity: KernelGoalPolarity::AssertTrueForConsistency,
            ..handoff_input(&set, &[], None, None)
        };

        let error =
            build_kernel_evidence_handoff(input).expect_err("consistency polarity rejected");

        assert!(matches!(
            &error,
            KernelEvidenceHandoffError::GoalPolarityMismatch {
                vc,
                requested: KernelGoalPolarity::AssertTrueForConsistency,
                required: KernelGoalPolarity::AssertFalseForRefutation,
            } if *vc == VcId::new(0)
        ));
        assert_eq!(
            error.to_string(),
            "goal polarity AssertTrueForConsistency does not match required AssertFalseForRefutation for VcId(0)"
        );
    }

    #[test]
    fn goal_polarity_mismatch_precedes_payload_validation() {
        let set = fixture_set(FixtureShape::GeneratedGoalOnly);
        let payloads = Vec::new();
        let input = KernelEvidenceHandoffInput {
            goal_polarity: KernelGoalPolarity::AssertTrueForConsistency,
            formula_payloads: &payloads,
            ..handoff_input(&set, &[], None, None)
        };

        let error = build_kernel_evidence_handoff(input)
            .expect_err("polarity mismatch rejected before missing payload");

        assert!(matches!(
            error,
            KernelEvidenceHandoffError::GoalPolarityMismatch {
                vc,
                requested: KernelGoalPolarity::AssertTrueForConsistency,
                required: KernelGoalPolarity::AssertFalseForRefutation,
            } if vc == VcId::new(0)
        ));
    }

    #[test]
    fn every_current_vc_kind_requires_refutation_polarity() {
        for kind in current_vc_kinds() {
            let set = fixture_set_with_kind(FixtureShape::GeneratedGoalOnly, kind.clone());
            let input = KernelEvidenceHandoffInput {
                goal_polarity: KernelGoalPolarity::AssertTrueForConsistency,
                ..handoff_input(&set, &[], None, None)
            };

            let error = build_kernel_evidence_handoff(input)
                .expect_err("current VC kind requires refutation polarity");

            assert!(
                matches!(
                    error,
                    KernelEvidenceHandoffError::GoalPolarityMismatch {
                        vc,
                        requested: KernelGoalPolarity::AssertTrueForConsistency,
                        required: KernelGoalPolarity::AssertFalseForRefutation,
                    } if vc == VcId::new(0)
                ),
                "{kind:?} did not reject consistency polarity"
            );
        }
    }

    #[test]
    fn missing_premise_and_imported_payloads_fail_closed() {
        let generated_set = fixture_set(FixtureShape::GeneratedPremise);
        let payloads = vec![formula_payload_for(VcGeneratedFormulaId::new(1))];
        let generated_input = KernelEvidenceHandoffInput {
            formula_payloads: &payloads,
            ..handoff_input(&generated_set, &[], None, None)
        };
        let error =
            build_kernel_evidence_handoff(generated_input).expect_err("missing generated premise");
        assert!(matches!(
            error,
            KernelEvidenceHandoffError::MissingFormulaPayload {
                formula: VcFormulaRef::Generated(_),
                role: KernelEvidenceRole::GeneratedPremise
            }
        ));

        let set = fixture_set(FixtureShape::LocalContextPremise);
        let payloads = vec![formula_payload_for(VcGeneratedFormulaId::new(1))];
        let local_input = KernelEvidenceHandoffInput {
            formula_payloads: &payloads,
            ..handoff_input(&set, &[], None, None)
        };
        let error = build_kernel_evidence_handoff(local_input).expect_err("missing local payload");
        assert!(matches!(
            error,
            KernelEvidenceHandoffError::MissingFormulaPayload {
                formula: VcFormulaRef::Generated(_),
                role: KernelEvidenceRole::LocalContext
            }
        ));

        let imported_set = fixture_set(FixtureShape::ImportedPremise);
        let imported_payloads = Vec::new();
        let import_input = KernelEvidenceHandoffInput {
            imported_formula_payloads: &imported_payloads,
            ..handoff_input(&imported_set, &[], None, None)
        };
        let error =
            build_kernel_evidence_handoff(import_input).expect_err("missing imported payload");
        assert!(matches!(
            error,
            KernelEvidenceHandoffError::MissingImportedFormulaPayload { symbol }
                if symbol == VcText::new("Imported::A1")
        ));

        for mutate in [
            |payload: &mut KernelImportedFormulaPayload| payload.requirement.package_id.clear(),
            |payload: &mut KernelImportedFormulaPayload| payload.requirement.module_path.clear(),
            |payload: &mut KernelImportedFormulaPayload| {
                payload.requirement.exported_item_id.clear();
            },
        ] {
            let mut payload = imported_payload(&VcText::new("Imported::A1"));
            mutate(&mut payload);
            let payloads = vec![payload];
            let input = KernelEvidenceHandoffInput {
                imported_formula_payloads: &payloads,
                ..handoff_input(&imported_set, &[], None, None)
            };
            let error = build_kernel_evidence_handoff(input)
                .expect_err("empty imported identity component");
            assert!(matches!(
                error,
                KernelEvidenceHandoffError::EmptyFormulaPayload {
                    role: KernelEvidenceRole::ImportedPremise
                }
            ));
        }
    }

    #[test]
    fn invalid_formula_projection_payloads_fail_closed() {
        let goal_set = fixture_set(FixtureShape::GeneratedGoalOnly);
        let mut empty_goal_formula = formula_payloads(&goal_set);
        empty_goal_formula[0].projection.formula_bytes.clear();
        let input = KernelEvidenceHandoffInput {
            formula_payloads: &empty_goal_formula,
            ..handoff_input(&goal_set, &[], None, None)
        };
        let error = build_kernel_evidence_handoff(input).expect_err("empty formula bytes");
        assert!(matches!(
            error,
            KernelEvidenceHandoffError::EmptyFormulaPayload {
                role: KernelEvidenceRole::FinalGoal
            }
        ));

        let local_set = fixture_set(FixtureShape::LocalContextPremise);
        let mut empty_local_formula = formula_payloads(&local_set);
        empty_local_formula[0].projection.formula_bytes.clear();
        let input = KernelEvidenceHandoffInput {
            formula_payloads: &empty_local_formula,
            ..handoff_input(&local_set, &[], None, None)
        };
        let error = build_kernel_evidence_handoff(input).expect_err("empty local formula bytes");
        assert!(matches!(
            error,
            KernelEvidenceHandoffError::EmptyFormulaPayload {
                role: KernelEvidenceRole::LocalContext
            }
        ));

        let mut empty_provenance = formula_payloads(&goal_set);
        empty_provenance[0].projection.provenance_payload.clear();
        let input = KernelEvidenceHandoffInput {
            formula_payloads: &empty_provenance,
            ..handoff_input(&goal_set, &[], None, None)
        };
        let error = build_kernel_evidence_handoff(input).expect_err("empty provenance");
        assert!(matches!(
            error,
            KernelEvidenceHandoffError::EmptyProvenancePayload {
                role: KernelEvidenceRole::FinalGoal
            }
        ));

        let mut unsupported_algorithm = formula_payloads(&goal_set);
        unsupported_algorithm[0]
            .projection
            .formula_fingerprint
            .algorithm_id = 99;
        let input = KernelEvidenceHandoffInput {
            formula_payloads: &unsupported_algorithm,
            ..handoff_input(&goal_set, &[], None, None)
        };
        let error =
            build_kernel_evidence_handoff(input).expect_err("unsupported formula algorithm");
        assert!(matches!(
            error,
            KernelEvidenceHandoffError::UnsupportedFormulaFingerprintAlgorithm { algorithm_id: 99 }
        ));
    }

    #[test]
    fn proof_hints_do_not_select_extra_premises() {
        let set = fixture_set(FixtureShape::HintOnlyCitation);
        let input = handoff_input(&set, &[], None, None);

        let handoff = build_kernel_evidence_handoff(input).expect("handoff");

        assert!(handoff.canonical_evidence().formula_evidence().is_empty());
        assert_eq!(
            handoff
                .canonical_evidence()
                .final_goal()
                .producer_formula_ref,
            VcFormulaRef::Generated(VcGeneratedFormulaId::new(1))
        );
        assert!(!String::from_utf8_lossy(handoff.canonical_hash_input()).contains("hint-only"));
    }

    #[test]
    fn imported_premise_requires_formula_context() {
        let set = fixture_set(FixtureShape::ImportedPremise);
        let symbol = VcText::new("Imported::A1");
        let imported = imported_payload(&symbol);
        let imported_payloads = vec![imported.clone()];
        let input_without_context = handoff_input(&set, &imported_payloads, None, None);

        let error = build_kernel_evidence_handoff(input_without_context)
            .expect_err("missing imported context");
        assert!(matches!(
            error,
            KernelEvidenceHandoffError::MissingFormulaContext { symbol: found }
                if found == symbol
        ));

        let wrong_context = KernelFormulaContextRequirements {
            provenance_fingerprint: fingerprint(7, b"context"),
            imported_axioms: Vec::new(),
            imported_theorems: Vec::new(),
        };
        let input_wrong_context =
            handoff_input(&set, &imported_payloads, Some(&wrong_context), None);
        let error = build_kernel_evidence_handoff(input_wrong_context).expect_err("wrong context");
        assert!(matches!(
            error,
            KernelEvidenceHandoffError::ImportedFormulaContextMismatch { symbol: found }
                if found == symbol
        ));

        let mut extra_axiom = imported.requirement.clone();
        extra_axiom.imported_fact_id = 0;
        extra_axiom.package_id = b"aaa".to_vec();
        extra_axiom.statement_fingerprint =
            fingerprint(KERNEL_FORMULA_FINGERPRINT_ALGORITHM_ID, b"extra-axiom");
        let mut theorem_low = imported.requirement.clone();
        theorem_low.imported_fact_id = 10;
        theorem_low.package_id = b"theorem-a".to_vec();
        theorem_low.statement_fingerprint =
            fingerprint(KERNEL_FORMULA_FINGERPRINT_ALGORITHM_ID, b"theorem-a");
        let mut theorem_high = imported.requirement.clone();
        theorem_high.imported_fact_id = 11;
        theorem_high.package_id = b"theorem-b".to_vec();
        theorem_high.statement_fingerprint =
            fingerprint(KERNEL_FORMULA_FINGERPRINT_ALGORITHM_ID, b"theorem-b");
        let context = KernelFormulaContextRequirements {
            provenance_fingerprint: fingerprint(7, b"context"),
            imported_axioms: vec![
                imported.requirement.clone(),
                extra_axiom.clone(),
                imported.requirement.clone(),
            ],
            imported_theorems: vec![
                theorem_high.clone(),
                theorem_low.clone(),
                theorem_high.clone(),
            ],
        };
        let handoff = build_kernel_evidence_handoff(handoff_input(
            &set,
            &imported_payloads,
            Some(&context),
            None,
        ))
        .expect("imported handoff");
        assert!(matches!(
            handoff.canonical_evidence().formula_evidence()[0].source(),
            KernelFormulaSource::AcceptedImportedAxiom(_)
        ));
        assert!(handoff.formula_context_requirements().is_some());
        let mut expected_axioms = vec![imported.requirement.clone(), extra_axiom];
        expected_axioms.sort();
        expected_axioms.dedup();
        let mut expected_theorems = vec![theorem_high, theorem_low];
        expected_theorems.sort();
        expected_theorems.dedup();
        assert_eq!(
            handoff
                .formula_context_requirements()
                .expect("context")
                .imported_axioms,
            expected_axioms
        );
        assert_eq!(
            handoff
                .formula_context_requirements()
                .expect("context")
                .imported_theorems,
            expected_theorems
        );

        let empty_context_provenance = KernelFormulaContextRequirements {
            provenance_fingerprint: KernelEvidenceFingerprint {
                algorithm_id: 7,
                digest: Vec::new(),
            },
            imported_axioms: vec![imported.requirement.clone()],
            imported_theorems: Vec::new(),
        };
        let error = build_kernel_evidence_handoff(handoff_input(
            &set,
            &imported_payloads,
            Some(&empty_context_provenance),
            None,
        ))
        .expect_err("empty context provenance");
        assert!(matches!(
            error,
            KernelEvidenceHandoffError::EmptyFormulaContextProvenance
        ));

        let mut wrong_algorithm = imported.clone();
        wrong_algorithm
            .requirement
            .statement_fingerprint
            .algorithm_id = 99;
        let wrong_algorithm_payloads = vec![wrong_algorithm];
        let error = build_kernel_evidence_handoff(handoff_input(
            &set,
            &wrong_algorithm_payloads,
            Some(&context),
            None,
        ))
        .expect_err("unsupported imported statement algorithm");
        assert!(matches!(
            error,
            KernelEvidenceHandoffError::UnsupportedFormulaFingerprintAlgorithm { algorithm_id: 99 }
        ));

        let mut mismatched = imported.clone();
        mismatched.requirement.statement_fingerprint =
            fingerprint(KERNEL_FORMULA_FINGERPRINT_ALGORITHM_ID, b"different");
        let mismatched_payloads = vec![mismatched];
        let error = build_kernel_evidence_handoff(handoff_input(
            &set,
            &mismatched_payloads,
            Some(&context),
            None,
        ))
        .expect_err("mismatched imported fingerprint");
        assert!(matches!(
            error,
            KernelEvidenceHandoffError::ImportedFormulaFingerprintMismatch { symbol: found }
                if found == symbol
        ));
    }

    #[test]
    fn substitutions_reference_source_formula_without_instantiated_fields() {
        let set = fixture_set(FixtureShape::LocalContextPremise);
        let substitution =
            substitution_payload(7, VcFormulaRef::Generated(VcGeneratedFormulaId::new(0)));
        let substitutions = vec![substitution];
        let input = KernelEvidenceHandoffInput {
            substitutions: &substitutions,
            ..handoff_input(&set, &[], None, None)
        };

        let handoff = build_kernel_evidence_handoff(input).expect("handoff");

        assert_eq!(handoff.canonical_evidence().substitutions().len(), 1);
        assert_eq!(
            handoff.canonical_evidence().substitutions()[0].source_formula_id,
            1
        );
        let debug = handoff.debug_text();
        let canonical = String::from_utf8_lossy(handoff.canonical_hash_input());
        assert!(!debug.contains("instantiated"));
        assert!(!debug.contains("target_formula"));
        assert!(!canonical.contains("instantiated"));
        assert!(!canonical.contains("target_formula"));
    }

    #[test]
    fn substitution_payloads_fail_closed_on_missing_source_and_empty_fields() {
        let source = VcFormulaRef::Generated(VcGeneratedFormulaId::new(0));
        let no_source_set = fixture_set(FixtureShape::GeneratedGoalOnly);
        let substitutions = vec![substitution_payload(7, source)];
        let input = KernelEvidenceHandoffInput {
            substitutions: &substitutions,
            ..handoff_input(&no_source_set, &[], None, None)
        };
        let error = build_kernel_evidence_handoff(input).expect_err("missing source");
        assert!(matches!(
            error,
            KernelEvidenceHandoffError::MissingSubstitutionSource {
                substitution_id: 7,
                source_formula
            } if source_formula == source
        ));

        let set = fixture_set(FixtureShape::LocalContextPremise);
        let empty_field_cases: [(&str, SubstitutionMutation); 3] = [
            (
                "binder_context_encoding",
                |substitution: &mut KernelSubstitutionPayload| {
                    substitution.binder_context_encoding.clear();
                },
            ),
            ("payload", |substitution: &mut KernelSubstitutionPayload| {
                substitution.payload.clear();
            }),
            (
                "provenance_payload",
                |substitution: &mut KernelSubstitutionPayload| {
                    substitution.provenance_payload.clear();
                },
            ),
        ];
        for (field, mutation) in empty_field_cases {
            let mut substitution = substitution_payload(7, source);
            mutation(&mut substitution);
            let substitutions = vec![substitution];
            let input = KernelEvidenceHandoffInput {
                substitutions: &substitutions,
                ..handoff_input(&set, &[], None, None)
            };
            let error = build_kernel_evidence_handoff(input).expect_err(field);
            assert!(matches!(
                error,
                KernelEvidenceHandoffError::EmptySubstitutionPayload {
                    substitution_id: 7,
                    field: found
                } if found == field
            ));
        }

        let mut empty_side_condition = substitution_payload(7, source);
        empty_side_condition.freshness_witnesses.push(Vec::new());
        let substitutions = vec![empty_side_condition];
        let input = KernelEvidenceHandoffInput {
            substitutions: &substitutions,
            ..handoff_input(&set, &[], None, None)
        };
        let error = build_kernel_evidence_handoff(input).expect_err("empty side condition");
        assert!(matches!(
            error,
            KernelEvidenceHandoffError::EmptySubstitutionSideCondition {
                substitution_id: 7,
                field: "freshness_witnesses"
            }
        ));

        let mut duplicate_side_condition = substitution_payload(7, source);
        duplicate_side_condition
            .free_variable_constraints
            .push(b"free-vars-7".to_vec());
        let substitutions = vec![duplicate_side_condition];
        let input = KernelEvidenceHandoffInput {
            substitutions: &substitutions,
            ..handoff_input(&set, &[], None, None)
        };
        let error = build_kernel_evidence_handoff(input).expect_err("duplicate side condition");
        assert!(matches!(
            error,
            KernelEvidenceHandoffError::DuplicateSubstitutionSideCondition {
                substitution_id: 7,
                field: "free_variable_constraints"
            }
        ));
    }

    #[test]
    fn discharge_diagnostics_exclude_rule_names_and_evidence_hashes() {
        let set = fixture_set(FixtureShape::GeneratedGoalOnly);
        let discharge = try_discharge(crate::discharge::DischargeInput {
            vc_set: &set,
            policy: &DischargePolicy::default(),
        })
        .expect("discharge");
        let input = handoff_input(&set, &[], None, Some(&discharge));

        let handoff = build_kernel_evidence_handoff(input).expect("handoff");

        assert_eq!(handoff.diagnostics().discharge_records.len(), 1);
        assert_eq!(handoff.canonical_evidence().formula_evidence().len(), 0);
        let debug = handoff.debug_text();
        let canonical = String::from_utf8_lossy(handoff.canonical_hash_input());
        assert!(debug.contains("[discharge-diagnostics]"));
        assert!(!debug.contains("GeneratedTautology"));
        assert!(!debug.contains("evidence_hash"));
        assert!(!canonical.contains("discharge"));
    }

    #[test]
    fn discharge_diagnostics_reject_mismatched_vc_set() {
        let set = fixture_set(FixtureShape::LocalContextPremise);
        let other_set = fixture_set(FixtureShape::GeneratedGoalOnly);
        let discharge = try_discharge(crate::discharge::DischargeInput {
            vc_set: &other_set,
            policy: &DischargePolicy::default(),
        })
        .expect("discharge");
        let input = handoff_input(&set, &[], None, Some(&discharge));

        let error = build_kernel_evidence_handoff(input).expect_err("mismatched discharge output");

        assert!(matches!(
            error,
            KernelEvidenceHandoffError::MismatchedDischargeOutput { vc } if vc == VcId::new(0)
        ));
    }

    #[test]
    fn prohibited_backend_material_is_absent_from_handoff_renderings() {
        let set = fixture_set(FixtureShape::GeneratedGoalOnly);
        let handoff =
            build_kernel_evidence_handoff(handoff_input(&set, &[], None, None)).expect("handoff");
        let rendered = format!(
            "{}\n{}",
            handoff.debug_text(),
            String::from_utf8_lossy(handoff.canonical_hash_input())
        );

        for prohibited in [
            "TPTP",
            "SMT-LIB",
            "DIMACS",
            "resolution_trace",
            "MiniSAT",
            "backend_log",
            "used_axioms",
        ] {
            assert!(!rendered.contains(prohibited), "{prohibited} leaked");
        }
    }

    #[derive(Clone, Copy)]
    enum FixtureShape {
        GeneratedGoalOnly,
        GoalOneNoHint,
        LocalContextPremise,
        GeneratedPremise,
        TwoPremisesForward,
        TwoPremisesReverse,
        DischargedWithEvidence,
        DischargedWithAlternateEvidence,
        HintOnlyCitation,
        UnsupportedHintMetadata,
        ImportedPremise,
    }

    fn handoff_input<'a>(
        set: &'a VcSet,
        imported_formula_payloads: &'a [KernelImportedFormulaPayload],
        formula_context: Option<&'a KernelFormulaContextRequirements>,
        discharge_output: Option<&'a DischargeOutput>,
    ) -> KernelEvidenceHandoffInput<'a> {
        let payloads = formula_payloads(set);
        let leaked_payloads = Box::leak(Box::new(payloads));
        KernelEvidenceHandoffInput {
            vc_set: set,
            vc: VcId::new(0),
            goal_polarity: KernelGoalPolarity::AssertFalseForRefutation,
            kernel_profile: KernelEvidenceProfile::v1(1, KernelClauseTautologyPolicy::Reject),
            symbol_manifest: &[],
            variable_manifest: &[],
            formula_payloads: leaked_payloads,
            imported_formula_payloads,
            substitutions: &[],
            formula_context,
            discharge_output,
        }
    }

    fn formula_payloads(set: &VcSet) -> Vec<KernelFormulaPayload> {
        set.generated_formulas()
            .iter()
            .map(|formula| KernelFormulaPayload {
                formula_ref: VcFormulaRef::Generated(formula.id),
                projection: formula_projection(formula.id),
            })
            .collect()
    }

    fn formula_payload_for(formula: VcGeneratedFormulaId) -> KernelFormulaPayload {
        KernelFormulaPayload {
            formula_ref: VcFormulaRef::Generated(formula),
            projection: formula_projection(formula),
        }
    }

    fn formula_projection(formula: VcGeneratedFormulaId) -> KernelFormulaProjection {
        KernelFormulaProjection {
            formula_fingerprint: fingerprint(
                KERNEL_FORMULA_FINGERPRINT_ALGORITHM_ID,
                format!("formula-{}", formula.index()).as_bytes(),
            ),
            formula_bytes: format!("kernel-formula-{}", formula.index()).into_bytes(),
            provenance_payload: format!("provenance-{}", formula.index()).into_bytes(),
        }
    }

    fn substitution_payload(
        substitution_id: u32,
        source_formula: VcFormulaRef,
    ) -> KernelSubstitutionPayload {
        substitution_payload_with_side_conditions(
            substitution_id,
            source_formula,
            vec![format!("fresh-{substitution_id}").into_bytes()],
            vec![format!("free-vars-{substitution_id}").into_bytes()],
        )
    }

    fn substitution_payload_with_side_conditions(
        substitution_id: u32,
        source_formula: VcFormulaRef,
        freshness_witnesses: Vec<Vec<u8>>,
        free_variable_constraints: Vec<Vec<u8>>,
    ) -> KernelSubstitutionPayload {
        KernelSubstitutionPayload {
            substitution_id,
            source_formula,
            binder_context_encoding: format!("binder-context-{substitution_id}").into_bytes(),
            payload: format!("substitution-payload-{substitution_id}").into_bytes(),
            freshness_witnesses,
            free_variable_constraints,
            provenance_payload: format!("substitution-provenance-{substitution_id}").into_bytes(),
        }
    }

    fn imported_payload(symbol: &VcText) -> KernelImportedFormulaPayload {
        KernelImportedFormulaPayload {
            symbol: symbol.clone(),
            class: KernelImportedFormulaClass::Axiom,
            requirement: KernelImportedFactRequirement {
                imported_fact_id: 1,
                package_id: b"pkg".to_vec(),
                module_path: b"module".to_vec(),
                exported_item_id: b"item".to_vec(),
                statement_fingerprint: fingerprint(
                    KERNEL_FORMULA_FINGERPRINT_ALGORITHM_ID,
                    b"imported-formula",
                ),
                required_proof_status: KernelRequiredProofStatus::KernelVerified,
            },
            projection: KernelFormulaProjection {
                formula_fingerprint: fingerprint(
                    KERNEL_FORMULA_FINGERPRINT_ALGORITHM_ID,
                    b"imported-formula",
                ),
                formula_bytes: b"imported-kernel-formula".to_vec(),
                provenance_payload: b"imported-provenance".to_vec(),
            },
        }
    }

    fn fixture_set(shape: FixtureShape) -> VcSet {
        fixture_set_with_kind(shape, VcKind::TheoremProofStep)
    }

    fn fixture_set_with_kind(shape: FixtureShape, kind: VcKind) -> VcSet {
        let snapshot = BuildSnapshotId::from_published_schema_str(
            "mizar-session-build-snapshot-v1:\
             2222222222222222222222222222222222222222222222222222222222222222",
        )
        .expect("snapshot id");
        let source = InMemorySessionIdAllocator::new()
            .next_source_id(snapshot)
            .expect("source id");
        let generated_formulas = vec![
            VcGeneratedFormula {
                id: VcGeneratedFormulaId::new(0),
                kind: VcGeneratedFormulaKind::GeneratedTypeObligation,
                shape: VcGeneratedFormulaShape::True,
                provenance: vec![provenance("generated-0")],
            },
            VcGeneratedFormula {
                id: VcGeneratedFormulaId::new(1),
                kind: VcGeneratedFormulaKind::SplitGoal,
                shape: VcGeneratedFormulaShape::False,
                provenance: vec![provenance("generated-1")],
            },
        ];
        let (local_context, premises, proof_hint, goal, status) = match shape {
            FixtureShape::GeneratedGoalOnly => (
                LocalContext::try_new(Vec::new(), Vec::new()).expect("context"),
                Vec::new(),
                None,
                VcFormulaRef::Generated(VcGeneratedFormulaId::new(0)),
                VcStatus::Open,
            ),
            FixtureShape::GoalOneNoHint => (
                LocalContext::try_new(Vec::new(), Vec::new()).expect("context"),
                Vec::new(),
                None,
                VcFormulaRef::Generated(VcGeneratedFormulaId::new(1)),
                VcStatus::NeedsAtp,
            ),
            FixtureShape::LocalContextPremise => (
                LocalContext::try_new(
                    vec![ContextEntry {
                        id: ContextEntryId::new(0),
                        sort_key: CanonicalSortKey::new("000-local"),
                        kind: ContextEntryKind::ProofAssumption,
                        formula: Some(VcFormulaRef::Generated(VcGeneratedFormulaId::new(0))),
                        provenance: vec![provenance("local")],
                    }],
                    Vec::new(),
                )
                .expect("context"),
                vec![PremiseRef::LocalContext(ContextEntryId::new(0))],
                None,
                VcFormulaRef::Generated(VcGeneratedFormulaId::new(1)),
                VcStatus::NeedsAtp,
            ),
            FixtureShape::GeneratedPremise => (
                LocalContext::try_new(Vec::new(), Vec::new()).expect("context"),
                vec![PremiseRef::GeneratedFact {
                    formula: VcFormulaRef::Generated(VcGeneratedFormulaId::new(0)),
                }],
                None,
                VcFormulaRef::Generated(VcGeneratedFormulaId::new(1)),
                VcStatus::NeedsAtp,
            ),
            FixtureShape::TwoPremisesForward | FixtureShape::TwoPremisesReverse => {
                let context = LocalContext::try_new(Vec::new(), Vec::new()).expect("context");
                let premises = match shape {
                    FixtureShape::TwoPremisesForward => vec![
                        PremiseRef::GeneratedFact {
                            formula: VcFormulaRef::Generated(VcGeneratedFormulaId::new(0)),
                        },
                        PremiseRef::GeneratedFact {
                            formula: VcFormulaRef::Generated(VcGeneratedFormulaId::new(1)),
                        },
                    ],
                    FixtureShape::TwoPremisesReverse => vec![
                        PremiseRef::GeneratedFact {
                            formula: VcFormulaRef::Generated(VcGeneratedFormulaId::new(1)),
                        },
                        PremiseRef::GeneratedFact {
                            formula: VcFormulaRef::Generated(VcGeneratedFormulaId::new(0)),
                        },
                    ],
                    _ => unreachable!("shape matched above"),
                };
                (
                    context,
                    premises,
                    None,
                    VcFormulaRef::Generated(VcGeneratedFormulaId::new(1)),
                    VcStatus::NeedsAtp,
                )
            }
            FixtureShape::DischargedWithEvidence
            | FixtureShape::DischargedWithAlternateEvidence => {
                let (rule, evidence_hash) = match shape {
                    FixtureShape::DischargedWithEvidence => (
                        VcText::new("discharge-rule"),
                        stable_fingerprint_hash("discharge-evidence", b"discharge-evidence"),
                    ),
                    FixtureShape::DischargedWithAlternateEvidence => (
                        VcText::new("alternate-discharge-rule"),
                        stable_fingerprint_hash(
                            "alternate-discharge-evidence",
                            b"alternate-discharge-evidence",
                        ),
                    ),
                    _ => unreachable!("shape matched above"),
                };
                (
                    LocalContext::try_new(Vec::new(), Vec::new()).expect("context"),
                    Vec::new(),
                    None,
                    VcFormulaRef::Generated(VcGeneratedFormulaId::new(1)),
                    VcStatus::Discharged {
                        evidence: crate::vc_ir::DischargeEvidenceRef {
                            rule,
                            evidence_hash: HashMarker::Available(evidence_hash),
                        },
                    },
                )
            }
            FixtureShape::HintOnlyCitation => (
                LocalContext::try_new(Vec::new(), Vec::new()).expect("context"),
                Vec::new(),
                Some(crate::vc_ir::ProofHint {
                    citations: vec![PremiseRef::GeneratedFact {
                        formula: VcFormulaRef::Generated(VcGeneratedFormulaId::new(0)),
                    }],
                    unfold_requests: Vec::new(),
                    premise_restrictions: Vec::new(),
                    solver: Some(crate::vc_ir::ProofHintKey::new("hint-only")),
                    max_axioms: Some(1),
                    timeout: None,
                    computation: None,
                    provenance: vec![provenance("hint")],
                }),
                VcFormulaRef::Generated(VcGeneratedFormulaId::new(1)),
                VcStatus::NeedsAtp,
            ),
            FixtureShape::UnsupportedHintMetadata => (
                LocalContext::try_new(Vec::new(), Vec::new()).expect("context"),
                Vec::new(),
                Some(crate::vc_ir::ProofHint {
                    citations: Vec::new(),
                    unfold_requests: vec![DefinitionUnfoldRequest {
                        definition: CoreDefinitionId::new(0),
                        opacity_override: Some(DefinitionOpacityOverride::Transparent),
                    }],
                    premise_restrictions: Vec::new(),
                    solver: Some(crate::vc_ir::ProofHintKey::new("hint-only")),
                    max_axioms: Some(1),
                    timeout: None,
                    computation: None,
                    provenance: vec![provenance("hint")],
                }),
                VcFormulaRef::Generated(VcGeneratedFormulaId::new(1)),
                VcStatus::NeedsAtp,
            ),
            FixtureShape::ImportedPremise => (
                LocalContext::try_new(Vec::new(), Vec::new()).expect("context"),
                vec![PremiseRef::ImportedFact {
                    symbol: VcText::new("Imported::A1"),
                }],
                None,
                VcFormulaRef::Generated(VcGeneratedFormulaId::new(1)),
                VcStatus::NeedsAtp,
            ),
        };
        VcSet::try_new(VcSetParts {
            schema_version: VcSchemaVersion::new("kernel-handoff-test-v1"),
            snapshot,
            source,
            module: VcModuleRef::new("sample"),
            generated_formulas,
            vcs: vec![VcIr {
                id: VcId::new(0),
                kind: kind.clone(),
                source: VcSourceRef {
                    primary: source_ref(source),
                    related: Vec::new(),
                },
                seed: SeedVcRef {
                    handoff: ObligationHandoffId::new(0),
                },
                anchor: incomplete_anchor(source, kind),
                local_context,
                premises,
                goal,
                proof_hint,
                status,
                provenance: vec![provenance("vc")],
            }],
            seed_accounting: vec![SeedAccounting {
                handoff: ObligationHandoffId::new(0),
                origin: SeedOriginRef::ExistingCore {
                    seed: ObligationSeedId::new(0),
                },
                seed_status: ObligationSeedStatus::Active,
                mapping: SeedVcMapping::One { vc: VcId::new(0) },
            }],
        })
        .expect("vc set")
    }

    fn current_vc_kinds() -> Vec<VcKind> {
        vec![
            VcKind::TheoremProofStep,
            VcKind::TerminalProofGoal,
            VcKind::DefinitionCorrectness,
            VcKind::RegistrationStyleCorrectness {
                style: RegistrationCorrectnessKind::Registration,
            },
            VcKind::RegistrationStyleCorrectness {
                style: RegistrationCorrectnessKind::Redefinition,
            },
            VcKind::RegistrationStyleCorrectness {
                style: RegistrationCorrectnessKind::Reduction,
            },
            VcKind::RegistrationStyleCorrectness {
                style: RegistrationCorrectnessKind::ExplicitCoreSeed,
            },
            VcKind::CheckerInitial,
            VcKind::GeneratedNonEmptiness,
            VcKind::GeneratedSethood,
            VcKind::FraenkelMembershipAxiom,
            VcKind::AlgorithmPrecondition,
            VcKind::AlgorithmPostcondition,
            VcKind::CallPrecondition,
            VcKind::AlgorithmAssertion,
            VcKind::LoopInvariant {
                phase: LoopInvariantPhase::Entry,
            },
            VcKind::LoopInvariant {
                phase: LoopInvariantPhase::Preservation,
            },
            VcKind::LoopInvariant {
                phase: LoopInvariantPhase::Break,
            },
            VcKind::LoopInvariant {
                phase: LoopInvariantPhase::Continue,
            },
            VcKind::LoopInvariant {
                phase: LoopInvariantPhase::Exit,
            },
            VcKind::RangeLoop {
                obligation: RangeLoopObligation::PositiveStep,
            },
            VcKind::RangeLoop {
                obligation: RangeLoopObligation::RangeBound,
            },
            VcKind::RangeLoop {
                obligation: RangeLoopObligation::HiddenIndex,
            },
            VcKind::CollectionLoop {
                obligation: CollectionLoopObligation::Finiteness,
            },
            VcKind::CollectionLoop {
                obligation: CollectionLoopObligation::OrderIndependence,
            },
            VcKind::Termination,
            VcKind::PartialTermination,
            VcKind::GhostErasureSafety,
            VcKind::PolicyDeferredTraceability,
        ]
    }

    fn incomplete_anchor(source: SourceId, kind: VcKind) -> crate::vc_ir::ObligationAnchor {
        crate::vc_ir::ObligationAnchor {
            owner: crate::vc_ir::AnchorOwner::Theorem(CoreItemId::new(0)),
            kind,
            local_path: LocalProofOrProgramPath::new("proof/0"),
            label: Some(AnchorLabel {
                role: AnchorLabelRole::UserLabel,
                hint: Some(CoreLabelRef::new("A1")),
            }),
            semantic_origin: NormalizedSemanticOrigin::new("theorem:sample"),
            source_range: Some(SourceRange {
                source_id: source,
                start: 0,
                end: 4,
            }),
            provenance: vec![provenance("anchor")],
            source_shape_hash: HashMarker::Unavailable {
                reason: AnchorUnavailableReason::new("test fixture"),
            },
            canonical_goal_hash: HashMarker::Unavailable {
                reason: AnchorUnavailableReason::new("test fixture"),
            },
            canonical_context_hash: HashMarker::Unavailable {
                reason: AnchorUnavailableReason::new("test fixture"),
            },
            generation_schema_version: GenerationSchemaVersion::new("kernel-handoff-test"),
            completeness: AnchorCompleteness::Incomplete {
                missing: vec![
                    AnchorIngredient::SourceShapeHash,
                    AnchorIngredient::CanonicalGoalHash,
                    AnchorIngredient::CanonicalContextHash,
                ],
            },
        }
    }

    fn source_ref(source: SourceId) -> CoreSourceRef {
        CoreSourceRef::direct(SourceRange {
            source_id: source,
            start: 0,
            end: 4,
        })
        .with_provenance(vec![CoreProvenance::new(
            CoreProvenancePhase::ProofSkeleton,
            "kernel-handoff-test",
        )])
    }

    fn provenance(key: &str) -> VcProvenance {
        VcProvenance {
            phase: VcProvenancePhase::Generator,
            key: VcText::new(key),
            core: None,
        }
    }

    fn fingerprint(algorithm_id: u8, digest: &[u8]) -> KernelEvidenceFingerprint {
        KernelEvidenceFingerprint::new(algorithm_id, digest.to_vec()).expect("fingerprint")
    }
}
