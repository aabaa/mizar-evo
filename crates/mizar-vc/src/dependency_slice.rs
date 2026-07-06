//! Conservative dependency slices for verification conditions.
//!
//! This module implements the task-14 slice specified in
//! [dependency_slice.md](../../../doc/design/mizar-vc/en/dependency_slice.md):
//! deterministic, prover-independent dependency slices for validated `VcSet`
//! data.

use crate::{
    discharge::{
        DischargeEvidenceRecord, DischargeEvidenceReplay, DischargeEvidenceSource, DischargeOutput,
    },
    kernel_evidence_handoff::VcKernelEvidenceHandoff,
    vc_ir::{
        AnchorCompleteness, AnchorIngredient, ComputationHint, ContextEntry, ContextEntryId,
        DefinitionUnfoldRequest, DischargeEvidenceRef, HashMarker, ObligationAnchor, PolicyKey,
        PremiseRef, PremiseRestriction, ProofHint, VcFormulaRef, VcGeneratedFormulaId,
        VcGeneratedFormulaShape, VcId, VcIr, VcKind, VcSet, VcStatus, VcText,
        stable_fingerprint_hash,
    },
};
use mizar_core::control_flow::ObligationHandoffId;
use mizar_session::Hash;
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt::{self, Write as _},
};

pub const DEPENDENCY_SLICE_SCHEMA_VERSION: &str = "mizar-vc-dependency-slice-v1";

#[derive(Debug, Clone, Copy)]
pub struct DependencySliceInput<'a> {
    pub vc_set: &'a VcSet,
    pub discharge_output: Option<&'a DischargeOutput>,
}

#[derive(Debug, Clone, Copy)]
pub struct KernelEvidenceDependencyInput<'a> {
    pub vc: VcId,
    pub handoff: &'a VcKernelEvidenceHandoff,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencySliceSet {
    schema_version: &'static str,
    slices: Vec<DependencySlice>,
}

impl DependencySliceSet {
    pub const fn schema_version(&self) -> &'static str {
        self.schema_version
    }

    pub fn slices(&self) -> &[DependencySlice] {
        &self.slices
    }

    pub fn slice_for(&self, vc: VcId) -> Option<&DependencySlice> {
        self.slices.iter().find(|slice| slice.vc == vc)
    }

    pub fn proof_reuse_key_for(
        &self,
        discharge_output: &DischargeOutput,
        vc: VcId,
    ) -> Option<ProofReuseCandidateKey> {
        let _ = (discharge_output, vc);
        None
    }

    pub fn proof_reuse_key_for_kernel_handoff(
        &self,
        discharge_output: &DischargeOutput,
        vc: VcId,
        kernel_handoff: &VcKernelEvidenceHandoff,
    ) -> Option<ProofReuseCandidateKey> {
        let current_slices = try_compute_dependency_slices_with_kernel_evidence(
            DependencySliceInput {
                vc_set: discharge_output.vc_set(),
                discharge_output: Some(discharge_output),
            },
            &[KernelEvidenceDependencyInput {
                vc,
                handoff: kernel_handoff,
            }],
        )
        .ok()?;
        let provided_slice = self.slice_for(vc)?;
        let current_slice = current_slices.slice_for(vc)?;
        if provided_slice.fingerprint != current_slice.fingerprint
            || provided_slice.completeness != current_slice.completeness
            || provided_slice.kind != current_slice.kind
            || provided_slice.status != current_slice.status
        {
            return None;
        }
        let current_vc = discharge_output.vc_set().vc(vc)?;
        proof_reuse_key(current_vc, current_slice, discharge_output, kernel_handoff)
    }

    pub fn debug_text(&self) -> String {
        let mut output = String::from("dependency-slice-set-debug-v1\n");
        writeln!(&mut output, "schema-version: {}", self.schema_version).expect("write string");
        for slice in &self.slices {
            writeln!(
                &mut output,
                "slice {:?}: kind={:?}; status={:?}; completeness={:?}; \
                 fingerprint={}",
                slice.vc,
                slice.kind,
                slice.status,
                slice.completeness,
                hex(slice.fingerprint.hash().as_bytes())
            )
            .expect("write string");
            writeln!(&mut output, "  [entries]").expect("write string");
            for entry in &slice.entries {
                writeln!(
                    &mut output,
                    "  {:?}: {} => {}",
                    entry.class, entry.local_key, entry.payload
                )
                .expect("write string");
            }
            writeln!(&mut output, "  [unknowns]").expect("write string");
            for unknown in &slice.unknowns {
                writeln!(
                    &mut output,
                    "  {:?}: {} => {}",
                    unknown.family, unknown.local_key, unknown.reason
                )
                .expect("write string");
            }
        }
        output
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencySlice {
    vc: VcId,
    kind: VcKind,
    status: VcStatus,
    entries: Vec<DependencyEntry>,
    unknowns: Vec<DependencyUnknown>,
    completeness: DependencySliceCompleteness,
    fingerprint: DependencySliceFingerprint,
}

impl DependencySlice {
    pub const fn vc(&self) -> VcId {
        self.vc
    }

    pub const fn kind(&self) -> &VcKind {
        &self.kind
    }

    pub const fn status(&self) -> &VcStatus {
        &self.status
    }

    pub fn entries(&self) -> &[DependencyEntry] {
        &self.entries
    }

    pub fn unknowns(&self) -> &[DependencyUnknown] {
        &self.unknowns
    }

    pub const fn completeness(&self) -> DependencySliceCompleteness {
        self.completeness
    }

    pub const fn is_complete(&self) -> bool {
        matches!(self.completeness, DependencySliceCompleteness::Complete)
    }

    pub const fn requires_cache_miss(&self) -> bool {
        !self.is_complete()
    }

    pub const fn fingerprint(&self) -> DependencySliceFingerprint {
        self.fingerprint
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DependencySliceFingerprint(Hash);

impl DependencySliceFingerprint {
    pub const fn hash(self) -> Hash {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProofReuseCandidateKey(Hash);

impl ProofReuseCandidateKey {
    pub const fn hash(self) -> Hash {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum DependencySliceCompleteness {
    Complete,
    IncompleteUncacheable,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DependencyEntry {
    class: DependencyEntryClass,
    local_key: String,
    payload: String,
    fingerprint_payload: String,
}

impl DependencyEntry {
    pub fn class(&self) -> DependencyEntryClass {
        self.class
    }

    pub fn local_key(&self) -> &str {
        &self.local_key
    }

    pub fn payload(&self) -> &str {
        &self.payload
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum DependencyEntryClass {
    LocalContext,
    GeneratedFormula,
    CoreFormula,
    Definition,
    ImportedFact,
    Trace,
    Policy,
    Anchor,
    DischargeEvidence,
    KernelEvidence,
    Seed,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DependencyUnknown {
    family: DependencyUnknownFamily,
    local_key: String,
    reason: String,
}

impl DependencyUnknown {
    pub fn family(&self) -> DependencyUnknownFamily {
        self.family
    }

    pub fn local_key(&self) -> &str {
        &self.local_key
    }

    pub fn reason(&self) -> &str {
        &self.reason
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum DependencyUnknownFamily {
    Premise,
    Anchor,
    Trace,
    Import,
    Definition,
    Computation,
    DischargeEvidence,
    UpstreamPayload,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DependencySliceError {
    MismatchedDischargeOutput,
    DuplicateKernelEvidence { vc: VcId },
    MismatchedKernelEvidence { vc: VcId },
    UnknownKernelEvidenceVc { vc: VcId },
}

impl fmt::Display for DependencySliceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MismatchedDischargeOutput => {
                formatter.write_str("discharge output does not match dependency-slice input VcSet")
            }
            Self::DuplicateKernelEvidence { vc } => {
                write!(formatter, "duplicate kernel evidence handoff for {vc:?}")
            }
            Self::MismatchedKernelEvidence { vc } => {
                write!(formatter, "kernel evidence handoff does not match {vc:?}")
            }
            Self::UnknownKernelEvidenceVc { vc } => {
                write!(
                    formatter,
                    "kernel evidence handoff references unknown {vc:?}"
                )
            }
        }
    }
}

impl Error for DependencySliceError {}

pub fn try_compute_dependency_slices(
    input: DependencySliceInput<'_>,
) -> Result<DependencySliceSet, DependencySliceError> {
    try_compute_dependency_slices_with_kernel_evidence(input, &[])
}

pub fn try_compute_dependency_slices_with_kernel_evidence(
    input: DependencySliceInput<'_>,
    kernel_evidence: &[KernelEvidenceDependencyInput<'_>],
) -> Result<DependencySliceSet, DependencySliceError> {
    let discharge_output = input.discharge_output;
    if let Some(output) = discharge_output
        && output.vc_set() != input.vc_set
    {
        return Err(DependencySliceError::MismatchedDischargeOutput);
    }

    let evidence_by_vc = discharge_output.map_or_else(BTreeMap::new, evidence_by_vc);
    let explanations_by_vc = discharge_output.map_or_else(BTreeMap::new, explanations_by_vc);
    let kernel_evidence_by_vc = kernel_evidence_by_vc(input.vc_set, kernel_evidence)?;

    let slices = input
        .vc_set
        .vcs()
        .iter()
        .map(|vc| {
            let records = evidence_by_vc
                .get(&vc.id)
                .map_or([].as_slice(), Vec::as_slice);
            let explanations = explanations_by_vc
                .get(&vc.id)
                .map_or([].as_slice(), Vec::as_slice);
            compute_slice(
                input.vc_set,
                vc,
                records,
                explanations,
                kernel_evidence_by_vc.get(&vc.id).copied(),
            )
        })
        .collect();

    Ok(DependencySliceSet {
        schema_version: DEPENDENCY_SLICE_SCHEMA_VERSION,
        slices,
    })
}

fn kernel_evidence_by_vc<'a>(
    vc_set: &VcSet,
    inputs: &'a [KernelEvidenceDependencyInput<'a>],
) -> Result<BTreeMap<VcId, &'a VcKernelEvidenceHandoff>, DependencySliceError> {
    let mut by_vc = BTreeMap::new();
    for input in inputs {
        let Some(vc) = vc_set.vc(input.vc) else {
            return Err(DependencySliceError::UnknownKernelEvidenceVc { vc: input.vc });
        };
        let handoff = input.handoff;
        if !handoff.targets_vc(vc_set, input.vc).unwrap_or(false)
            || handoff
                .canonical_evidence()
                .final_goal()
                .producer_formula_ref
                != vc.goal
            || !handoff.context_identity_is_consistent()
        {
            return Err(DependencySliceError::MismatchedKernelEvidence { vc: input.vc });
        }
        if by_vc.insert(input.vc, handoff).is_some() {
            return Err(DependencySliceError::DuplicateKernelEvidence { vc: input.vc });
        }
    }
    Ok(by_vc)
}

fn evidence_by_vc(output: &DischargeOutput) -> BTreeMap<VcId, Vec<&DischargeEvidenceRecord>> {
    let mut by_vc = BTreeMap::new();
    for record in output.evidence_records() {
        by_vc.entry(record.vc).or_insert_with(Vec::new).push(record);
    }
    by_vc
}

fn explanations_by_vc(
    output: &DischargeOutput,
) -> BTreeMap<VcId, Vec<&crate::discharge::DischargeExplanation>> {
    let mut by_vc = BTreeMap::new();
    for explanation in output.explanations() {
        by_vc
            .entry(explanation.vc)
            .or_insert_with(Vec::new)
            .push(explanation);
    }
    by_vc
}

fn compute_slice(
    vc_set: &VcSet,
    vc: &VcIr,
    evidence_records: &[&DischargeEvidenceRecord],
    explanations: &[&crate::discharge::DischargeExplanation],
    kernel_evidence: Option<&VcKernelEvidenceHandoff>,
) -> DependencySlice {
    let mut builder = SliceBuilder::new(vc_set, vc);
    builder.collect_status(&vc.status, evidence_records.is_empty());
    builder.collect_formula(vc.goal);
    builder.collect_anchor(&vc.anchor);
    builder.collect_seed_accounting(vc.seed.handoff);

    for input in vc.local_context.policy_inputs() {
        builder.add_entry(
            DependencyEntryClass::Policy,
            format!("policy-input:{}", input.sort_key.as_str()),
            format!("key={:?}; value={:?}", input.key, input.value),
        );
    }

    for premise in &vc.premises {
        builder.collect_premise(premise);
    }

    if let Some(hint) = &vc.proof_hint {
        builder.collect_proof_hint(hint);
    }

    for record in evidence_records {
        builder.collect_discharge_record(record);
    }

    for explanation in explanations {
        builder.add_entry(
            DependencyEntryClass::DischargeEvidence,
            "discharge:explanation".to_owned(),
            format!(
                "category={:?}; rule={:?}; detail={:?}",
                explanation.category, explanation.rule, explanation.detail
            ),
        );
    }

    if let Some(handoff) = kernel_evidence {
        builder.collect_kernel_evidence(handoff);
    }

    builder.finish()
}

struct SliceBuilder<'a> {
    vc_set: &'a VcSet,
    vc: &'a VcIr,
    entries: BTreeSet<DependencyEntry>,
    unknowns: BTreeSet<DependencyUnknown>,
}

impl<'a> SliceBuilder<'a> {
    fn new(vc_set: &'a VcSet, vc: &'a VcIr) -> Self {
        Self {
            vc_set,
            vc,
            entries: BTreeSet::new(),
            unknowns: BTreeSet::new(),
        }
    }

    fn finish(self) -> DependencySlice {
        let entries = self.entries.into_iter().collect::<Vec<_>>();
        let unknowns = self.unknowns.into_iter().collect::<Vec<_>>();
        let completeness = if unknowns.is_empty() {
            DependencySliceCompleteness::Complete
        } else {
            DependencySliceCompleteness::IncompleteUncacheable
        };
        let fingerprint = fingerprint_for(
            &self.vc.kind,
            &self.vc.status,
            &entries,
            &unknowns,
            completeness,
        );

        DependencySlice {
            vc: self.vc.id,
            kind: self.vc.kind.clone(),
            status: self.vc.status.clone(),
            entries,
            unknowns,
            completeness,
            fingerprint,
        }
    }

    fn add_entry(
        &mut self,
        class: DependencyEntryClass,
        local_key: impl Into<String>,
        payload: impl Into<String>,
    ) {
        let payload = payload.into();
        self.add_entry_with_fingerprint_payload(class, local_key, payload.clone(), payload);
    }

    fn add_entry_with_fingerprint_payload(
        &mut self,
        class: DependencyEntryClass,
        local_key: impl Into<String>,
        payload: impl Into<String>,
        fingerprint_payload: impl Into<String>,
    ) {
        self.entries.insert(DependencyEntry {
            class,
            local_key: local_key.into(),
            payload: payload.into(),
            fingerprint_payload: fingerprint_payload.into(),
        });
    }

    fn add_unknown(
        &mut self,
        family: DependencyUnknownFamily,
        local_key: impl Into<String>,
        reason: impl Into<String>,
    ) {
        self.unknowns.insert(DependencyUnknown {
            family,
            local_key: local_key.into(),
            reason: reason.into(),
        });
    }

    fn collect_status(&mut self, status: &VcStatus, missing_replay: bool) {
        self.add_entry_with_fingerprint_payload(
            DependencyEntryClass::Policy,
            "status:boundary",
            format!("{status:?}"),
            status_fingerprint_payload(status),
        );

        match status {
            VcStatus::PolicyOpen { policy } => self.add_policy_ref("status:policy-open", policy),
            VcStatus::AssumedByPolicy { policy, marker } => {
                self.add_policy_ref("status:assumed-by-policy", policy);
                self.collect_premise(marker);
            }
            VcStatus::Discharged { evidence } => {
                self.collect_discharge_evidence_ref("status", evidence);
                if missing_replay {
                    self.add_unknown(
                        DependencyUnknownFamily::DischargeEvidence,
                        "discharge:missing-replay",
                        "discharged status has no matching DischargeOutput replay data",
                    );
                }
            }
            VcStatus::Open
            | VcStatus::NeedsAtp
            | VcStatus::SkippedDueToInvalidInput { .. }
            | VcStatus::DeferredExternal { .. }
            | VcStatus::Error { .. } => {}
        }
    }

    fn collect_formula(&mut self, formula: VcFormulaRef) {
        self.collect_formula_inner(formula, &mut BTreeSet::new(), &mut BTreeSet::new());
    }

    fn collect_formula_inner(
        &mut self,
        formula: VcFormulaRef,
        active_formulas: &mut BTreeSet<VcGeneratedFormulaId>,
        active_context: &mut BTreeSet<ContextEntryId>,
    ) {
        match formula {
            VcFormulaRef::Core(formula) => {
                self.add_entry_with_fingerprint_payload(
                    DependencyEntryClass::CoreFormula,
                    format!("core-formula:{}", formula.index()),
                    format!("{formula:?}"),
                    "core-formula:unresolved",
                );
                self.add_unknown(
                    DependencyUnknownFamily::UpstreamPayload,
                    "core-formula:unresolved",
                    "core formula payload is unavailable",
                );
            }
            VcFormulaRef::Generated(id) => {
                self.add_entry_with_fingerprint_payload(
                    DependencyEntryClass::GeneratedFormula,
                    format!("generated-formula:{}", id.index()),
                    format!("{id:?}"),
                    "generated-formula:ref",
                );
                if !active_formulas.insert(id) {
                    self.add_unknown(
                        DependencyUnknownFamily::UpstreamPayload,
                        format!("generated-formula:{}", id.index()),
                        "generated formula recursion was already active",
                    );
                    return;
                }

                let Some(generated) = self.vc_set.generated_formula(id) else {
                    self.add_unknown(
                        DependencyUnknownFamily::UpstreamPayload,
                        format!("generated-formula:{}", id.index()),
                        "generated formula payload is unavailable",
                    );
                    active_formulas.remove(&id);
                    return;
                };
                let fingerprint_payload =
                    formula_fingerprint_payload(self.vc_set, VcFormulaRef::Generated(generated.id));
                if fingerprint_payload.is_none() {
                    self.add_unknown(
                        DependencyUnknownFamily::UpstreamPayload,
                        "generated-formula:unresolved",
                        "generated formula payload is unavailable for reuse fingerprinting",
                    );
                }
                self.add_entry_with_fingerprint_payload(
                    DependencyEntryClass::GeneratedFormula,
                    format!("generated-formula:{}:payload", id.index()),
                    format!("kind={:?}; shape={:?}", generated.kind, generated.shape),
                    fingerprint_payload
                        .unwrap_or_else(|| "generated-formula:unresolved".to_owned()),
                );
                match &generated.shape {
                    VcGeneratedFormulaShape::True
                    | VcGeneratedFormulaShape::False
                    | VcGeneratedFormulaShape::Diagnostic(_) => {}
                    VcGeneratedFormulaShape::Ref(inner) | VcGeneratedFormulaShape::Not(inner) => {
                        self.collect_formula_inner(*inner, active_formulas, active_context);
                    }
                    VcGeneratedFormulaShape::And(formulas)
                    | VcGeneratedFormulaShape::Or(formulas) => {
                        for formula in formulas {
                            self.collect_formula_inner(*formula, active_formulas, active_context);
                        }
                    }
                    VcGeneratedFormulaShape::Implies {
                        premise,
                        conclusion,
                    } => {
                        self.collect_formula_inner(*premise, active_formulas, active_context);
                        self.collect_formula_inner(*conclusion, active_formulas, active_context);
                    }
                    VcGeneratedFormulaShape::Quantified { binders, body, .. } => {
                        for binder in binders {
                            self.collect_context_entry_inner(
                                *binder,
                                active_formulas,
                                active_context,
                            );
                        }
                        self.collect_formula_inner(*body, active_formulas, active_context);
                    }
                }
                active_formulas.remove(&id);
            }
        }
    }

    fn collect_context_entry(&mut self, id: ContextEntryId) {
        self.collect_context_entry_inner(id, &mut BTreeSet::new(), &mut BTreeSet::new());
    }

    fn collect_context_entry_inner(
        &mut self,
        id: ContextEntryId,
        active_formulas: &mut BTreeSet<VcGeneratedFormulaId>,
        active_context: &mut BTreeSet<ContextEntryId>,
    ) {
        if !active_context.insert(id) {
            self.add_unknown(
                DependencyUnknownFamily::UpstreamPayload,
                format!("context:{}", id.index()),
                "local context recursion was already active",
            );
            return;
        }

        let Some(entry) = context_entry(self.vc, id) else {
            self.add_unknown(
                DependencyUnknownFamily::UpstreamPayload,
                format!("context:{}", id.index()),
                "local context payload is unavailable",
            );
            active_context.remove(&id);
            return;
        };

        self.add_context_entry_inner(entry, active_formulas, active_context);
        active_context.remove(&id);
    }

    fn add_context_entry_inner(
        &mut self,
        entry: &ContextEntry,
        active_formulas: &mut BTreeSet<VcGeneratedFormulaId>,
        active_context: &mut BTreeSet<ContextEntryId>,
    ) {
        let mut fingerprint_payload = format!(
            "sort-key={:?}; kind={}; provenance={:?}",
            entry.sort_key,
            context_entry_kind_fingerprint_payload(&entry.kind),
            entry.provenance
        );
        if let Some(formula) = entry.formula {
            let formula_payload = formula_fingerprint_payload(self.vc_set, formula)
                .unwrap_or_else(|| "formula:unresolved".to_owned());
            write!(&mut fingerprint_payload, "; formula={formula_payload}").expect("write string");
        } else {
            fingerprint_payload.push_str("; formula=<none>");
        }
        if matches!(
            entry.kind,
            crate::vc_ir::ContextEntryKind::BinderDeclaration { .. }
        ) {
            self.add_unknown(
                DependencyUnknownFamily::UpstreamPayload,
                "context:binder-declaration",
                "binder declaration payload is unavailable",
            );
        }

        self.add_entry_with_fingerprint_payload(
            DependencyEntryClass::LocalContext,
            format!("context:{}", entry.id.index()),
            format!(
                "sort-key={:?}; kind={:?}; formula={:?}; provenance={:?}",
                entry.sort_key, entry.kind, entry.formula, entry.provenance
            ),
            fingerprint_payload,
        );
        if let Some(formula) = entry.formula {
            self.collect_formula_inner(formula, active_formulas, active_context);
        }
    }

    fn collect_premise(&mut self, premise: &PremiseRef) {
        match premise {
            PremiseRef::LocalContext(id) => self.collect_context_entry(*id),
            PremiseRef::LocalLabel { label } => self.add_entry(
                DependencyEntryClass::ImportedFact,
                format!("local-label:{label:?}"),
                format!("{premise:?}"),
            ),
            PremiseRef::ImportedFact { symbol } => {
                self.add_entry(
                    DependencyEntryClass::ImportedFact,
                    format!("imported-fact:{}", symbol.as_str()),
                    format!("{premise:?}"),
                );
                self.add_unknown(
                    DependencyUnknownFamily::Import,
                    format!("imported-fact:{}", symbol.as_str()),
                    "imported fact is represented only by an opaque symbol marker",
                );
            }
            PremiseRef::DefinitionBoundary { definition }
            | PremiseRef::PermittedUnfolding { definition } => {
                self.add_definition_ref(*definition, format!("{premise:?}"));
            }
            PremiseRef::CheckerFact { formula } | PremiseRef::TypePredicate { formula } => {
                self.collect_formula(VcFormulaRef::Core(*formula));
            }
            PremiseRef::RegistrationTrace { trace } => {
                self.collect_trace("registration", trace);
            }
            PremiseRef::ClusterTrace { trace } => {
                self.collect_trace("cluster", trace);
            }
            PremiseRef::ReductionTrace { trace } => {
                self.collect_trace("reduction", trace);
            }
            PremiseRef::GeneratedFact { formula } => self.collect_formula(*formula),
            PremiseRef::PolicyAssumption { marker } => {
                self.add_policy_ref("premise:policy-assumption", marker);
            }
            PremiseRef::ConservativeUnknown { reason } => self.add_unknown(
                DependencyUnknownFamily::Premise,
                format!("premise:unknown:{}", reason.as_str()),
                reason.as_str().to_owned(),
            ),
        }
    }

    fn collect_trace(&mut self, family: &str, trace: &VcText) {
        self.add_entry(
            DependencyEntryClass::Trace,
            format!("{family}-trace:{}", trace.as_str()),
            format!("family={family}; marker={trace:?}"),
        );
        self.add_unknown(
            DependencyUnknownFamily::Trace,
            format!("{family}-trace:{}", trace.as_str()),
            "trace dependency is represented only by an opaque textual marker",
        );
    }

    fn collect_proof_hint(&mut self, hint: &ProofHint) {
        if let Some(solver) = &hint.solver {
            self.add_entry(
                DependencyEntryClass::Policy,
                format!("proof-hint:solver:{}", solver.as_str()),
                format!("{solver:?}"),
            );
        }
        if let Some(max_axioms) = hint.max_axioms {
            self.add_entry(
                DependencyEntryClass::Policy,
                "proof-hint:max-axioms",
                max_axioms.to_string(),
            );
        }
        if let Some(timeout) = &hint.timeout {
            self.add_entry(
                DependencyEntryClass::Policy,
                format!("proof-hint:timeout:{}", timeout.as_str()),
                format!("{timeout:?}"),
            );
        }
        if let Some(computation) = &hint.computation {
            self.collect_computation_hint(computation);
        }

        for premise in &hint.citations {
            self.collect_premise(premise);
        }
        for request in &hint.unfold_requests {
            self.collect_unfold_request(request);
        }
        for restriction in &hint.premise_restrictions {
            match restriction {
                PremiseRestriction::Only(premises) | PremiseRestriction::Exclude(premises) => {
                    for premise in premises {
                        self.collect_premise(premise);
                    }
                }
                PremiseRestriction::Intent(intent) => self.add_entry(
                    DependencyEntryClass::Policy,
                    format!("proof-hint:restriction-intent:{}", intent.as_str()),
                    format!("{intent:?}"),
                ),
            }
        }
    }

    fn collect_computation_hint(&mut self, computation: &ComputationHint) {
        self.add_entry(
            DependencyEntryClass::Policy,
            "proof-hint:computation",
            format!("{computation:?}"),
        );
        if let ComputationHint::LimitPolicy(policy) = computation {
            self.add_policy_ref("proof-hint:computation-limit", policy);
        }
        if let ComputationHint::SymbolicRequest(request) = computation {
            self.add_unknown(
                DependencyUnknownFamily::Computation,
                format!("computation:symbolic-request:{}", request.as_str()),
                "symbolic computation request is represented only by an opaque marker",
            );
        }
    }

    fn collect_unfold_request(&mut self, request: &DefinitionUnfoldRequest) {
        self.add_definition_ref(
            request.definition,
            format!("unfold-request opacity={:?}", request.opacity_override),
        );
    }

    fn collect_anchor(&mut self, anchor: &ObligationAnchor) {
        self.add_entry_with_fingerprint_payload(
            DependencyEntryClass::Anchor,
            "anchor:owner",
            format!("{:?}", anchor.owner),
            stable_anchor_owner_payload(&anchor.owner),
        );
        self.add_entry(
            DependencyEntryClass::Anchor,
            "anchor:kind",
            format!("{:?}", anchor.kind),
        );
        self.add_entry(
            DependencyEntryClass::Anchor,
            "anchor:local-path",
            format!("{:?}", anchor.local_path),
        );
        self.add_entry(
            DependencyEntryClass::Anchor,
            "anchor:label",
            format!("{:?}", anchor.label),
        );
        self.add_entry(
            DependencyEntryClass::Anchor,
            "anchor:semantic-origin",
            format!("{:?}", anchor.semantic_origin),
        );
        self.add_entry(
            DependencyEntryClass::Anchor,
            "anchor:generation-schema",
            format!("{:?}", anchor.generation_schema_version),
        );
        self.collect_anchor_hash_marker(
            AnchorIngredient::SourceShapeHash,
            "anchor:source-shape-hash",
            &anchor.source_shape_hash,
        );
        self.collect_anchor_hash_marker(
            AnchorIngredient::CanonicalGoalHash,
            "anchor:canonical-goal-hash",
            &anchor.canonical_goal_hash,
        );
        self.collect_anchor_hash_marker(
            AnchorIngredient::CanonicalContextHash,
            "anchor:canonical-context-hash",
            &anchor.canonical_context_hash,
        );

        if let AnchorCompleteness::Incomplete { missing } = &anchor.completeness {
            for ingredient in missing {
                self.add_unknown(
                    DependencyUnknownFamily::Anchor,
                    format!("anchor:missing:{ingredient:?}"),
                    "anchor ingredient is incomplete or unavailable",
                );
            }
        }
    }

    fn collect_anchor_hash_marker(
        &mut self,
        ingredient: AnchorIngredient,
        local_key: &str,
        marker: &HashMarker,
    ) {
        self.add_entry(
            DependencyEntryClass::Anchor,
            local_key.to_owned(),
            hash_marker_payload(marker),
        );
        match marker {
            HashMarker::Available(_) => {}
            HashMarker::Unavailable { reason } | HashMarker::ConservativeUnknown { reason } => {
                self.add_unknown(
                    DependencyUnknownFamily::Anchor,
                    local_key.to_owned(),
                    format!("{ingredient:?}: {}", reason.as_str()),
                );
            }
        }
    }

    fn collect_seed_accounting(&mut self, handoff: ObligationHandoffId) {
        let Some(row) = self.vc_set.seed_accounting_for(handoff) else {
            self.add_unknown(
                DependencyUnknownFamily::UpstreamPayload,
                format!("seed:{handoff:?}"),
                "seed accounting row is unavailable",
            );
            return;
        };

        self.add_entry_with_fingerprint_payload(
            DependencyEntryClass::Seed,
            "seed:current-obligation",
            format!(
                "handoff={handoff:?}; origin={:?}; status={:?}; mapping={}",
                row.origin,
                row.seed_status,
                seed_mapping_payload(row, self.vc.id)
            ),
            format!(
                "status={:?}; mapping={}",
                row.seed_status,
                seed_mapping_payload(row, self.vc.id)
            ),
        );
    }

    fn collect_discharge_record(&mut self, record: &DischargeEvidenceRecord) {
        let payload = format!(
            "source={:?}; rule={:?}; status-evidence={}",
            record.source,
            record.rule,
            discharge_evidence_payload(&record.status_evidence)
        );
        let fingerprint_payload = format!(
            "source={:?}; rule={:?}; status-evidence={}",
            record.source,
            record.rule,
            discharge_evidence_fingerprint_payload(&record.status_evidence)
        );
        self.add_entry_with_fingerprint_payload(
            DependencyEntryClass::DischargeEvidence,
            format!("discharge:record:{}", record.rule_name.as_str()),
            payload,
            fingerprint_payload,
        );
        self.collect_discharge_evidence_ref("record", &record.status_evidence);
        self.collect_formula(record.inputs.goal);
        for id in &record.inputs.local_context {
            self.collect_context_entry(*id);
        }
        for premise in &record.inputs.premises {
            self.collect_premise(premise);
        }
        for formula in &record.inputs.generated_formulas {
            self.collect_formula(VcFormulaRef::Generated(*formula));
        }
        for policy in &record.inputs.policy_inputs {
            self.add_entry(
                DependencyEntryClass::Policy,
                format!("discharge:policy:{}", policy.key.as_str()),
                format!("key={:?}; value={:?}", policy.key, policy.value),
            );
        }
        for request in &record.inputs.unfold_requests {
            self.collect_unfold_request(request);
        }
        if let Some(computation) = &record.inputs.computation {
            self.add_entry(
                DependencyEntryClass::Policy,
                "discharge:computation",
                format!(
                    "hint={:?}; active-policy={:?}; max-steps={}; requested-steps={:?}; timeout={:?}",
                    computation.hint,
                    computation.active_policy,
                    computation.max_steps,
                    computation.requested_steps,
                    computation.timeout
                ),
            );
        }
        if let DischargeEvidenceReplay::PreservedStatusOnly { reason } = &record.replay {
            self.add_unknown(
                DependencyUnknownFamily::DischargeEvidence,
                "discharge:preserved-status-only",
                reason.as_str().to_owned(),
            );
        }
    }

    fn collect_kernel_evidence(&mut self, handoff: &VcKernelEvidenceHandoff) {
        let context_requirements = handoff
            .formula_context_requirements()
            .map_or_else(|| "<none>".to_owned(), |context| format!("{context:?}"));
        let envelope = handoff.canonical_evidence();
        let payload = format!(
            "schema={}; encoding={}; target={:?}; profile={:?}; canonical-hash={}; formula-context={}",
            envelope.schema_version(),
            envelope.encoding_version(),
            envelope.target_vc(),
            envelope.kernel_profile(),
            hex(handoff.canonical_hash().as_bytes()),
            context_requirements
        );
        self.add_entry_with_fingerprint_payload(
            DependencyEntryClass::KernelEvidence,
            "kernel-evidence:canonical-handoff",
            payload.clone(),
            payload,
        );
        let context_identity = handoff.context_identity();
        let payload = format!(
            "schema={}; target={:?}; canonical-hash={}; context-identity-hash={}; entries={:?}",
            context_identity.schema_version(),
            context_identity.target_vc(),
            hex(context_identity.canonical_evidence_hash().as_bytes()),
            hex(context_identity.hash().as_bytes()),
            context_identity.entries()
        );
        self.add_entry_with_fingerprint_payload(
            DependencyEntryClass::KernelEvidence,
            "kernel-evidence:context-identity",
            payload.clone(),
            payload,
        );
    }

    fn collect_discharge_evidence_ref(&mut self, role: &str, evidence: &DischargeEvidenceRef) {
        self.add_entry_with_fingerprint_payload(
            DependencyEntryClass::DischargeEvidence,
            format!("discharge:evidence:{role}:{}", evidence.rule.as_str()),
            discharge_evidence_payload(evidence),
            discharge_evidence_fingerprint_payload(evidence),
        );
        match &evidence.evidence_hash {
            HashMarker::Available(_) => {}
            HashMarker::Unavailable { reason } => {
                self.add_unknown(
                    DependencyUnknownFamily::DischargeEvidence,
                    format!("discharge:evidence:{role}:unavailable"),
                    reason.as_str().to_owned(),
                );
            }
            HashMarker::ConservativeUnknown { reason } => {
                self.add_unknown(
                    DependencyUnknownFamily::DischargeEvidence,
                    format!("discharge:evidence:{role}:unknown"),
                    reason.as_str().to_owned(),
                );
            }
        }
    }

    fn add_definition_ref(
        &mut self,
        definition: mizar_core::core_ir::CoreDefinitionId,
        payload: String,
    ) {
        self.add_entry_with_fingerprint_payload(
            DependencyEntryClass::Definition,
            format!("definition:{}", definition.index()),
            payload,
            "definition:unresolved",
        );
        self.add_unknown(
            DependencyUnknownFamily::Definition,
            "definition:unresolved",
            "definition payload is unavailable",
        );
    }

    fn add_policy_ref(&mut self, role: &str, policy: &PolicyKey) {
        self.add_entry(
            DependencyEntryClass::Policy,
            format!("{role}:{}", policy.as_str()),
            format!("{policy:?}"),
        );
    }
}

fn context_entry(vc: &VcIr, id: ContextEntryId) -> Option<&ContextEntry> {
    vc.local_context
        .entries()
        .get(id.index())
        .filter(|entry| entry.id == id)
}

fn seed_mapping_payload(row: &crate::vc_ir::SeedAccounting, vc: VcId) -> String {
    match &row.mapping {
        crate::vc_ir::SeedVcMapping::NoConcreteVc { reason } => {
            format!("no-concrete-vc reason={reason:?}")
        }
        crate::vc_ir::SeedVcMapping::One { vc: mapped } if *mapped == vc => {
            "one current-vc".to_owned()
        }
        crate::vc_ir::SeedVcMapping::One { .. } => "one other-vc".to_owned(),
        crate::vc_ir::SeedVcMapping::Expanded {
            vcs,
            expansion_schema,
        } => {
            let expansion_index = vcs
                .iter()
                .find(|entry| entry.vc == vc)
                .map(|entry| entry.expansion_index);
            format!(
                "expanded index={expansion_index:?}; count={}; schema={expansion_schema:?}",
                vcs.len()
            )
        }
    }
}

fn discharge_evidence_payload(evidence: &DischargeEvidenceRef) -> String {
    format!(
        "rule={:?}; evidence-hash={}",
        evidence.rule,
        hash_marker_payload(&evidence.evidence_hash)
    )
}

fn discharge_evidence_fingerprint_payload(evidence: &DischargeEvidenceRef) -> String {
    format!(
        "rule={:?}; evidence-hash={}",
        evidence.rule,
        hash_marker_fingerprint_payload(&evidence.evidence_hash)
    )
}

fn hash_marker_payload(marker: &HashMarker) -> String {
    match marker {
        HashMarker::Available(hash) => format!("available:{}", hex(hash.as_bytes())),
        HashMarker::Unavailable { reason } => format!("unavailable:{}", reason.as_str()),
        HashMarker::ConservativeUnknown { reason } => {
            format!("conservative-unknown:{}", reason.as_str())
        }
    }
}

fn hash_marker_fingerprint_payload(marker: &HashMarker) -> String {
    match marker {
        HashMarker::Available(_) => "available".to_owned(),
        HashMarker::Unavailable { reason } => format!("unavailable:{}", reason.as_str()),
        HashMarker::ConservativeUnknown { reason } => {
            format!("conservative-unknown:{}", reason.as_str())
        }
    }
}

fn context_entry_kind_fingerprint_payload(kind: &crate::vc_ir::ContextEntryKind) -> String {
    match kind {
        crate::vc_ir::ContextEntryKind::BinderDeclaration { .. } => "BinderDeclaration".to_owned(),
        other => format!("{other:?}"),
    }
}

fn formula_fingerprint_payload(vc_set: &VcSet, formula: VcFormulaRef) -> Option<String> {
    formula_fingerprint_payload_inner(vc_set, formula, &mut BTreeSet::new())
}

fn formula_fingerprint_payload_inner(
    vc_set: &VcSet,
    formula: VcFormulaRef,
    active: &mut BTreeSet<VcGeneratedFormulaId>,
) -> Option<String> {
    match formula {
        VcFormulaRef::Core(_) => None,
        VcFormulaRef::Generated(id) => {
            if !active.insert(id) {
                return None;
            }
            let Some(generated) = vc_set.generated_formula(id) else {
                active.remove(&id);
                return None;
            };
            let Some(shape) = formula_shape_fingerprint_payload(vc_set, &generated.shape, active)
            else {
                active.remove(&id);
                return None;
            };
            active.remove(&id);
            Some(format!(
                "generated kind={:?}; provenance={:?}; {shape}",
                generated.kind, generated.provenance
            ))
        }
    }
}

fn formula_shape_fingerprint_payload(
    vc_set: &VcSet,
    shape: &VcGeneratedFormulaShape,
    active: &mut BTreeSet<VcGeneratedFormulaId>,
) -> Option<String> {
    match shape {
        VcGeneratedFormulaShape::True => Some("shape=true".to_owned()),
        VcGeneratedFormulaShape::False => Some("shape=false".to_owned()),
        VcGeneratedFormulaShape::Diagnostic(_) => None,
        VcGeneratedFormulaShape::Ref(formula) => {
            formula_fingerprint_payload_inner(vc_set, *formula, active)
                .map(|inner| format!("shape=ref({inner})"))
        }
        VcGeneratedFormulaShape::Not(formula) => {
            formula_fingerprint_payload_inner(vc_set, *formula, active)
                .map(|inner| format!("shape=not({inner})"))
        }
        VcGeneratedFormulaShape::And(formulas) => formulas
            .iter()
            .map(|formula| formula_fingerprint_payload_inner(vc_set, *formula, active))
            .collect::<Option<Vec<_>>>()
            .map(|payloads| format!("shape=and({})", payloads.join(";"))),
        VcGeneratedFormulaShape::Or(formulas) => formulas
            .iter()
            .map(|formula| formula_fingerprint_payload_inner(vc_set, *formula, active))
            .collect::<Option<Vec<_>>>()
            .map(|payloads| format!("shape=or({})", payloads.join(";"))),
        VcGeneratedFormulaShape::Implies {
            premise,
            conclusion,
        } => {
            let premise = formula_fingerprint_payload_inner(vc_set, *premise, active)?;
            let conclusion = formula_fingerprint_payload_inner(vc_set, *conclusion, active)?;
            Some(format!("shape=implies({premise};{conclusion})"))
        }
        VcGeneratedFormulaShape::Quantified { .. } => None,
    }
}

fn status_fingerprint_payload(status: &VcStatus) -> String {
    match status {
        VcStatus::Open => "Open".to_owned(),
        VcStatus::NeedsAtp => "NeedsAtp".to_owned(),
        VcStatus::Discharged { evidence } => format!(
            "Discharged {{ evidence={} }}",
            discharge_evidence_fingerprint_payload(evidence)
        ),
        VcStatus::PolicyOpen { policy } => format!("PolicyOpen {{ policy={policy:?} }}"),
        VcStatus::AssumedByPolicy { policy, marker } => {
            format!("AssumedByPolicy {{ policy={policy:?}; marker={marker:?} }}")
        }
        VcStatus::SkippedDueToInvalidInput { reason } => {
            format!("SkippedDueToInvalidInput {{ reason={reason:?} }}")
        }
        VcStatus::DeferredExternal { reason } => {
            format!("DeferredExternal {{ reason={reason:?} }}")
        }
        VcStatus::Error { diagnostic } => format!("Error {{ diagnostic={diagnostic:?} }}"),
    }
}

fn fingerprint_for(
    kind: &VcKind,
    status: &VcStatus,
    entries: &[DependencyEntry],
    unknowns: &[DependencyUnknown],
    completeness: DependencySliceCompleteness,
) -> DependencySliceFingerprint {
    let mut payload = String::from("dependency-slice-fingerprint-v1\n");
    writeln!(
        &mut payload,
        "schema-version: {DEPENDENCY_SLICE_SCHEMA_VERSION}"
    )
    .expect("write string");
    writeln!(&mut payload, "kind: {kind:?}").expect("write string");
    writeln!(
        &mut payload,
        "status: {}",
        status_fingerprint_payload(status)
    )
    .expect("write string");
    writeln!(&mut payload, "completeness: {completeness:?}").expect("write string");
    writeln!(&mut payload, "[entries]").expect("write string");
    let mut entry_payloads = entries
        .iter()
        .map(|entry| (entry.class, entry.fingerprint_payload.as_str()))
        .collect::<Vec<_>>();
    entry_payloads.sort();
    for (class, fingerprint_payload) in entry_payloads {
        writeln!(&mut payload, "{class:?}\t{fingerprint_payload}").expect("write string");
    }
    writeln!(&mut payload, "[unknowns]").expect("write string");
    let mut unknown_payloads = unknowns
        .iter()
        .map(|unknown| (unknown.family, unknown.reason.as_str()))
        .collect::<Vec<_>>();
    unknown_payloads.sort();
    for (family, reason) in unknown_payloads {
        writeln!(&mut payload, "{family:?}\t{reason}").expect("write string");
    }
    DependencySliceFingerprint(stable_hash(payload.as_bytes()))
}

fn proof_reuse_key(
    vc: &VcIr,
    slice: &DependencySlice,
    discharge_output: &DischargeOutput,
    kernel_handoff: &VcKernelEvidenceHandoff,
) -> Option<ProofReuseCandidateKey> {
    let payload = proof_reuse_key_payload(vc, slice, discharge_output, kernel_handoff)?;
    Some(ProofReuseCandidateKey(stable_fingerprint_hash(
        "mizar-vc-proof-reuse-key",
        payload.as_bytes(),
    )))
}

fn proof_reuse_key_payload(
    vc: &VcIr,
    slice: &DependencySlice,
    discharge_output: &DischargeOutput,
    kernel_handoff: &VcKernelEvidenceHandoff,
) -> Option<String> {
    if vc.id != slice.vc || vc.kind != slice.kind || vc.status != slice.status {
        return None;
    }
    if !vc.anchor.is_complete() || !slice.is_complete() {
        return None;
    }
    let VcStatus::Discharged { evidence } = &vc.status else {
        return None;
    };
    let record = discharge_output.evidence_records().iter().find(|record| {
        record.vc == vc.id
            && matches!(record.source, DischargeEvidenceSource::NewlyProduced)
            && matches!(&record.replay, DischargeEvidenceReplay::ExplicitInputRefs)
    })?;
    if &record.status_evidence != evidence {
        return None;
    }
    let HashMarker::Available(evidence_hash) = record.status_evidence.evidence_hash else {
        return None;
    };
    let canonical_vc = discharge_output.vc_set().canonical_vc_fingerprint(vc.id)?;
    let local_context = discharge_output.vc_set().local_context_fingerprint(vc.id)?;
    let policy = policy_fingerprint_for(vc);

    let mut payload = String::from("proof-reuse-candidate-key-v1\n");
    write_reuse_anchor_payload(&mut payload, &vc.anchor);
    writeln!(&mut payload, "canonical-vc: {:?}", canonical_vc.hash()).expect("write string");
    writeln!(&mut payload, "local-context: {:?}", local_context.hash()).expect("write string");
    writeln!(
        &mut payload,
        "dependency-slice: {:?}",
        slice.fingerprint.hash()
    )
    .expect("write string");
    writeln!(&mut payload, "verifier-policy: {policy:?}").expect("write string");
    writeln!(
        &mut payload,
        "kernel-evidence: canonical-hash={:?}; context-identity-hash={:?}; formula-context={:?}",
        kernel_handoff.canonical_hash(),
        kernel_handoff.context_identity_hash(),
        kernel_handoff.formula_context_requirements()
    )
    .expect("write string");
    writeln!(
        &mut payload,
        "deterministic-evidence: rule={:?}; hash={evidence_hash:?}",
        record.status_evidence.rule
    )
    .expect("write string");

    Some(payload)
}

fn write_reuse_anchor_payload(output: &mut String, anchor: &ObligationAnchor) {
    writeln!(output, "[anchor]").expect("write string");
    writeln!(
        output,
        "owner: {}",
        stable_anchor_owner_payload(&anchor.owner)
    )
    .expect("write string");
    writeln!(output, "kind: {:?}", anchor.kind).expect("write string");
    writeln!(output, "local-path: {:?}", anchor.local_path).expect("write string");
    writeln!(output, "label: {:?}", anchor.label).expect("write string");
    writeln!(output, "semantic-origin: {:?}", anchor.semantic_origin).expect("write string");
    writeln!(
        output,
        "source-shape: {}",
        hash_marker_fingerprint_payload(&anchor.source_shape_hash)
    )
    .expect("write string");
    writeln!(
        output,
        "canonical-goal: {}",
        hash_marker_fingerprint_payload(&anchor.canonical_goal_hash)
    )
    .expect("write string");
    writeln!(
        output,
        "canonical-context: {}",
        hash_marker_fingerprint_payload(&anchor.canonical_context_hash)
    )
    .expect("write string");
    writeln!(
        output,
        "generation-schema: {:?}",
        anchor.generation_schema_version
    )
    .expect("write string");
}

fn policy_fingerprint_for(vc: &VcIr) -> Hash {
    let mut payload = String::from("verifier-policy-fingerprint-v1\n");
    for input in vc.local_context.policy_inputs() {
        writeln!(
            &mut payload,
            "policy-input: sort={:?}; key={:?}; value={:?}",
            input.sort_key, input.key, input.value
        )
        .expect("write string");
    }
    match &vc.status {
        VcStatus::PolicyOpen { policy } => {
            writeln!(&mut payload, "status-policy-open: {policy:?}").expect("write string");
        }
        VcStatus::AssumedByPolicy { policy, marker } => {
            writeln!(
                &mut payload,
                "status-assumed-by-policy: policy={policy:?}; marker={marker:?}"
            )
            .expect("write string");
        }
        VcStatus::Discharged { evidence } => {
            writeln!(&mut payload, "status-discharged-rule: {:?}", evidence.rule)
                .expect("write string");
        }
        other => {
            writeln!(&mut payload, "status-boundary: {other:?}").expect("write string");
        }
    }
    stable_fingerprint_hash("mizar-vc-verifier-policy", payload.as_bytes())
}

fn stable_anchor_owner_payload(owner: &crate::vc_ir::AnchorOwner) -> String {
    match owner {
        crate::vc_ir::AnchorOwner::Theorem(_) => "theorem".to_owned(),
        crate::vc_ir::AnchorOwner::Definition(_) => "definition".to_owned(),
        crate::vc_ir::AnchorOwner::Registration(_) => "registration".to_owned(),
        crate::vc_ir::AnchorOwner::GeneratedSymbol(_) => "generated-symbol".to_owned(),
        crate::vc_ir::AnchorOwner::Algorithm(_) => "algorithm".to_owned(),
        crate::vc_ir::AnchorOwner::ProofBlock(_) => "proof-block".to_owned(),
        crate::vc_ir::AnchorOwner::CheckerOrigin(origin) => {
            format!("checker-origin:{}", origin.as_str())
        }
    }
}

fn stable_hash(bytes: &[u8]) -> Hash {
    let mut lanes = [
        0x6d_69_7a_61_72_2d_76_63_u64,
        0x64_65_70_2d_73_6c_69_63_u64,
        0x65_2d_66_69_6e_67_65_72_u64,
        0x70_72_69_6e_74_2d_76_31_u64,
    ];

    for (index, byte) in bytes.iter().copied().enumerate() {
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
    lanes[1] ^= (bytes.len() as u64).rotate_left(17);
    lanes[2] ^= lanes[0].rotate_left(7);
    lanes[3] ^= lanes[1].rotate_left(13);

    let mut output = [0_u8; Hash::BYTE_LEN];
    for (chunk, lane) in output.chunks_exact_mut(8).zip(lanes) {
        chunk.copy_from_slice(&lane.to_be_bytes());
    }
    Hash::from_bytes(output)
}

fn hex(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut encoded, "{byte:02x}").expect("write string");
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        discharge::{DischargeInput, DischargePolicy, try_discharge},
        kernel_evidence_handoff::{
            KERNEL_FORMULA_FINGERPRINT_ALGORITHM_ID, KernelClauseTautologyPolicy,
            KernelEvidenceFingerprint, KernelEvidenceHandoffInput, KernelEvidenceProfile,
            KernelFormulaPayload, KernelFormulaProjection, KernelGoalPolarity,
            VcKernelEvidenceHandoff, build_kernel_evidence_handoff,
        },
        vc_ir::{
            AnchorLabel, AnchorLabelRole, AnchorOwner, AnchorUnavailableReason, CanonicalSortKey,
            ContextEntryKind, DefinitionOpacityOverride, GenerationSchemaVersion, LocalContext,
            PolicyValue, ProofHintKey, SeedAccounting, SeedOriginRef, SeedVcMapping, SeedVcRef,
            VcGeneratedFormula, VcGeneratedFormulaKind, VcModuleRef, VcProvenance,
            VcProvenancePhase, VcSchemaVersion, VcSetParts, VcSourceRef, VerifierPolicyInput,
        },
    };
    use mizar_core::{
        control_flow::ObligationHandoffId,
        core_ir::{
            CoreDefinitionId, CoreDiagnosticId, CoreFormulaId, CoreItemId, CoreLabelRef,
            CoreNodeRef, CoreProvenance, CoreProvenanceKey, CoreProvenancePhase, CoreSourceRef,
            CoreVarId, GeneratedFrom, GeneratedOriginKey, GeneratedOriginKind,
            LocalProofOrProgramPath, NormalizedSemanticOrigin, ObligationSeedId,
            ObligationSeedStatus,
        },
    };
    use mizar_session::{
        BuildSnapshotId, InMemorySessionIdAllocator, SessionIdAllocator, SourceId, SourceRange,
    };

    #[test]
    fn collects_dependency_classes_from_vc_ir_inputs() {
        let mut parts = fixture_parts(
            VcStatus::NeedsAtp,
            VcFormulaRef::Generated(VcGeneratedFormulaId::new(0)),
            generated_chain(),
            complete_anchor_fixture(),
        );
        parts.vcs[0].local_context = rich_context(parts.source);
        parts.vcs[0].premises = vec![
            PremiseRef::ReductionTrace {
                trace: VcText::new("reduce/R"),
            },
            PremiseRef::RegistrationTrace {
                trace: VcText::new("registration/R"),
            },
            PremiseRef::ClusterTrace {
                trace: VcText::new("cluster/C"),
            },
            PremiseRef::ImportedFact {
                symbol: VcText::new("ORDERS_1:1"),
            },
            PremiseRef::DefinitionBoundary {
                definition: CoreDefinitionId::new(0),
            },
            PremiseRef::PermittedUnfolding {
                definition: CoreDefinitionId::new(1),
            },
            PremiseRef::CheckerFact {
                formula: CoreFormulaId::new(12),
            },
            PremiseRef::TypePredicate {
                formula: CoreFormulaId::new(13),
            },
            PremiseRef::LocalContext(ContextEntryId::new(0)),
            PremiseRef::PolicyAssumption {
                marker: PolicyKey::new("task-14-policy"),
            },
        ];
        parts.vcs[0].proof_hint = Some(rich_hint());

        let set = fixture_set(parts);
        let output = try_compute_dependency_slices(DependencySliceInput {
            vc_set: &set,
            discharge_output: None,
        })
        .expect("slice computation");
        let slice = only_slice(&output);

        assert!(has_entry(
            slice,
            DependencyEntryClass::CoreFormula,
            "core-formula:10"
        ));
        assert!(has_entry(
            slice,
            DependencyEntryClass::CoreFormula,
            "core-formula:11"
        ));
        assert!(has_entry(
            slice,
            DependencyEntryClass::GeneratedFormula,
            "generated-formula:0"
        ));
        assert!(has_entry(
            slice,
            DependencyEntryClass::GeneratedFormula,
            "generated-formula:1"
        ));
        assert!(has_entry(
            slice,
            DependencyEntryClass::LocalContext,
            "context:0"
        ));
        assert!(
            entry_payload(slice, DependencyEntryClass::LocalContext, "context:0")
                .is_some_and(|payload| payload.contains("provenance="))
        );
        assert!(has_entry(
            slice,
            DependencyEntryClass::Definition,
            "definition:0"
        ));
        assert!(has_entry(
            slice,
            DependencyEntryClass::Definition,
            "definition:1"
        ));
        assert!(has_entry(
            slice,
            DependencyEntryClass::Definition,
            "definition:2"
        ));
        assert!(has_entry(
            slice,
            DependencyEntryClass::ImportedFact,
            "imported-fact:ORDERS_1:1"
        ));
        assert!(has_entry(
            slice,
            DependencyEntryClass::Trace,
            "registration-trace:registration/R"
        ));
        assert!(has_entry(
            slice,
            DependencyEntryClass::Trace,
            "cluster-trace:cluster/C"
        ));
        assert!(has_entry(
            slice,
            DependencyEntryClass::Trace,
            "reduction-trace:reduce/R"
        ));
        assert!(has_entry(
            slice,
            DependencyEntryClass::Policy,
            "policy-input:000-policy"
        ));
        assert!(has_entry(
            slice,
            DependencyEntryClass::Policy,
            "proof-hint:solver:task-14-solver"
        ));
        assert!(has_entry(
            slice,
            DependencyEntryClass::Policy,
            "proof-hint:timeout:task-14-timeout"
        ));
        assert!(has_entry(
            slice,
            DependencyEntryClass::Policy,
            "proof-hint:max-axioms"
        ));
        assert!(has_entry(
            slice,
            DependencyEntryClass::Policy,
            "proof-hint:restriction-intent:task-14-intent"
        ));
        assert!(has_entry(
            slice,
            DependencyEntryClass::Policy,
            "proof-hint:computation"
        ));
        assert!(has_entry(
            slice,
            DependencyEntryClass::Anchor,
            "anchor:canonical-goal-hash"
        ));
        assert!(has_entry(
            slice,
            DependencyEntryClass::Seed,
            "seed:current-obligation"
        ));
        assert!(slice.requires_cache_miss());
        assert!(
            slice
                .unknowns()
                .iter()
                .any(|unknown| unknown.family() == DependencyUnknownFamily::Trace)
        );
        assert!(
            slice
                .unknowns()
                .iter()
                .any(|unknown| unknown.family() == DependencyUnknownFamily::Import)
        );
        assert!(
            slice
                .unknowns()
                .iter()
                .any(|unknown| unknown.family() == DependencyUnknownFamily::Computation)
        );
    }

    #[test]
    fn unused_local_context_entries_are_excluded() {
        let mut parts = fixture_parts(
            VcStatus::NeedsAtp,
            VcFormulaRef::Core(CoreFormulaId::new(0)),
            Vec::new(),
            complete_anchor_fixture(),
        );
        parts.vcs[0].local_context = rich_context(parts.source);
        parts.vcs[0].premises = vec![PremiseRef::LocalContext(ContextEntryId::new(0))];
        let set = fixture_set(parts);

        let output = try_compute_dependency_slices(DependencySliceInput {
            vc_set: &set,
            discharge_output: None,
        })
        .expect("slice computation");
        let slice = only_slice(&output);

        assert!(has_entry(
            slice,
            DependencyEntryClass::LocalContext,
            "context:0"
        ));
        assert!(!has_entry(
            slice,
            DependencyEntryClass::LocalContext,
            "context:1"
        ));
    }

    #[test]
    fn quantified_binder_context_cycles_become_conservative_unknowns() {
        let mut parts = fixture_parts(
            VcStatus::NeedsAtp,
            VcFormulaRef::Generated(VcGeneratedFormulaId::new(0)),
            vec![generated_formula(
                0,
                VcGeneratedFormulaShape::Quantified {
                    kind: crate::vc_ir::QuantifierKind::Forall,
                    binders: vec![ContextEntryId::new(0)],
                    body: VcFormulaRef::Core(CoreFormulaId::new(0)),
                },
            )],
            complete_anchor_fixture(),
        );
        parts.vcs[0].local_context = LocalContext::try_new(
            vec![ContextEntry {
                id: ContextEntryId::new(0),
                sort_key: CanonicalSortKey::new("000-binder"),
                kind: ContextEntryKind::BinderDeclaration {
                    var: CoreVarId::new(0),
                    role: VcText::new("binder"),
                },
                formula: Some(VcFormulaRef::Generated(VcGeneratedFormulaId::new(0))),
                provenance: vec![provenance("recursive-binder")],
            }],
            Vec::new(),
        )
        .expect("recursive binder context");
        let set = fixture_set(parts);

        let slices = try_compute_dependency_slices(DependencySliceInput {
            vc_set: &set,
            discharge_output: None,
        })
        .expect("slice computation");
        let slice = only_slice(&slices);

        assert!(slice.requires_cache_miss());
        assert!(slice.unknowns().iter().any(|unknown| {
            unknown.family() == DependencyUnknownFamily::UpstreamPayload
                && unknown.local_key() == "generated-formula:0"
                && unknown.reason().contains("recursion")
        }));
    }

    #[test]
    fn discharge_evidence_is_collected_and_mismatches_fail_closed() {
        let original = fixture_set(fixture_parts(
            VcStatus::Open,
            VcFormulaRef::Generated(VcGeneratedFormulaId::new(0)),
            vec![generated_formula(0, VcGeneratedFormulaShape::True)],
            complete_anchor_fixture(),
        ));
        let discharge = try_discharge(DischargeInput {
            vc_set: &original,
            policy: &DischargePolicy::default(),
        })
        .expect("discharge");

        assert_eq!(
            try_compute_dependency_slices(DependencySliceInput {
                vc_set: &original,
                discharge_output: Some(&discharge),
            }),
            Err(DependencySliceError::MismatchedDischargeOutput)
        );

        let slices = try_compute_dependency_slices(DependencySliceInput {
            vc_set: discharge.vc_set(),
            discharge_output: Some(&discharge),
        })
        .expect("slice computation");
        let slice = only_slice(&slices);

        assert!(matches!(slice.status(), VcStatus::Discharged { .. }));
        assert!(has_entry(
            slice,
            DependencyEntryClass::DischargeEvidence,
            "discharge:record:task-11-generated-tautology-v1"
        ));
        assert!(has_entry(
            slice,
            DependencyEntryClass::DischargeEvidence,
            "discharge:explanation"
        ));
        assert!(!slice.unknowns().iter().any(|unknown| {
            unknown.family() == DependencyUnknownFamily::DischargeEvidence
                && unknown.local_key().contains("missing-replay")
        }));
    }

    #[test]
    fn kernel_evidence_identity_participates_in_slice_and_reuse_key() {
        let original = fixture_set(fixture_parts(
            VcStatus::Open,
            VcFormulaRef::Generated(VcGeneratedFormulaId::new(0)),
            vec![generated_formula(0, VcGeneratedFormulaShape::True)],
            complete_anchor_fixture(),
        ));
        let discharge = try_discharge(DischargeInput {
            vc_set: &original,
            policy: &DischargePolicy::default(),
        })
        .expect("discharge");
        let handoff = kernel_handoff_for(&discharge, VcId::new(0));
        let plain = try_compute_dependency_slices(DependencySliceInput {
            vc_set: discharge.vc_set(),
            discharge_output: Some(&discharge),
        })
        .expect("plain slices");
        let with_kernel = try_compute_dependency_slices_with_kernel_evidence(
            DependencySliceInput {
                vc_set: discharge.vc_set(),
                discharge_output: Some(&discharge),
            },
            &[KernelEvidenceDependencyInput {
                vc: VcId::new(0),
                handoff: &handoff,
            }],
        )
        .expect("kernel slices");
        let plain_slice = only_slice(&plain);
        let kernel_slice = only_slice(&with_kernel);

        assert_ne!(plain_slice.fingerprint(), kernel_slice.fingerprint());
        assert!(has_entry(
            kernel_slice,
            DependencyEntryClass::KernelEvidence,
            "kernel-evidence:canonical-handoff"
        ));
        assert!(has_entry(
            kernel_slice,
            DependencyEntryClass::KernelEvidence,
            "kernel-evidence:context-identity"
        ));
        assert!(
            plain
                .proof_reuse_key_for(&discharge, VcId::new(0))
                .is_none()
        );
        assert!(
            with_kernel
                .proof_reuse_key_for_kernel_handoff(&discharge, VcId::new(0), &handoff)
                .is_some()
        );

        assert_eq!(
            try_compute_dependency_slices_with_kernel_evidence(
                DependencySliceInput {
                    vc_set: discharge.vc_set(),
                    discharge_output: Some(&discharge),
                },
                &[
                    KernelEvidenceDependencyInput {
                        vc: VcId::new(0),
                        handoff: &handoff,
                    },
                    KernelEvidenceDependencyInput {
                        vc: VcId::new(0),
                        handoff: &handoff,
                    },
                ],
            ),
            Err(DependencySliceError::DuplicateKernelEvidence { vc: VcId::new(0) })
        );
        assert_eq!(
            try_compute_dependency_slices_with_kernel_evidence(
                DependencySliceInput {
                    vc_set: discharge.vc_set(),
                    discharge_output: Some(&discharge),
                },
                &[KernelEvidenceDependencyInput {
                    vc: VcId::new(1),
                    handoff: &handoff,
                }],
            ),
            Err(DependencySliceError::UnknownKernelEvidenceVc { vc: VcId::new(1) })
        );
        assert_eq!(
            try_compute_dependency_slices_with_kernel_evidence(
                DependencySliceInput {
                    vc_set: &fixture_set(fixture_parts(
                        VcStatus::Open,
                        VcFormulaRef::Core(CoreFormulaId::new(42)),
                        vec![generated_formula(0, VcGeneratedFormulaShape::True)],
                        complete_anchor_fixture(),
                    )),
                    discharge_output: None,
                },
                &[KernelEvidenceDependencyInput {
                    vc: VcId::new(0),
                    handoff: &handoff,
                }],
            ),
            Err(DependencySliceError::MismatchedKernelEvidence { vc: VcId::new(0) })
        );
        let two_vc = two_vc_same_goal_fixture();
        let two_vc_discharge = try_discharge(DischargeInput {
            vc_set: &two_vc,
            policy: &DischargePolicy::default(),
        })
        .expect("two-vc discharge");
        let handoff_for_vc0 = kernel_handoff_for(&two_vc_discharge, VcId::new(0));
        let handoff_for_vc1 = kernel_handoff_for(&two_vc_discharge, VcId::new(1));
        assert_eq!(
            two_vc_discharge.vc_set().vcs()[0].goal,
            two_vc_discharge.vc_set().vcs()[1].goal
        );
        assert_eq!(
            try_compute_dependency_slices_with_kernel_evidence(
                DependencySliceInput {
                    vc_set: two_vc_discharge.vc_set(),
                    discharge_output: Some(&two_vc_discharge),
                },
                &[KernelEvidenceDependencyInput {
                    vc: VcId::new(1),
                    handoff: &handoff_for_vc0,
                }],
            ),
            Err(DependencySliceError::MismatchedKernelEvidence { vc: VcId::new(1) })
        );
        let vc1_slices = try_compute_dependency_slices_with_kernel_evidence(
            DependencySliceInput {
                vc_set: two_vc_discharge.vc_set(),
                discharge_output: Some(&two_vc_discharge),
            },
            &[KernelEvidenceDependencyInput {
                vc: VcId::new(1),
                handoff: &handoff_for_vc1,
            }],
        )
        .expect("vc1 kernel slices");
        assert!(
            vc1_slices
                .proof_reuse_key_for_kernel_handoff(
                    &two_vc_discharge,
                    VcId::new(1),
                    &handoff_for_vc0,
                )
                .is_none(),
            "handoff target binding must match the selected VC before reuse"
        );
    }

    #[test]
    fn context_identity_hash_participates_independently_in_slice_and_reuse_key() {
        let mut parts = fixture_parts(
            VcStatus::Open,
            VcFormulaRef::Generated(VcGeneratedFormulaId::new(1)),
            vec![
                generated_formula(0, VcGeneratedFormulaShape::True),
                generated_formula(1, VcGeneratedFormulaShape::True),
            ],
            complete_anchor_fixture(),
        );
        parts.vcs[0].premises = vec![PremiseRef::GeneratedFact {
            formula: VcFormulaRef::Generated(VcGeneratedFormulaId::new(0)),
        }];
        let original = fixture_set(parts);
        let discharge = try_discharge(DischargeInput {
            vc_set: &original,
            policy: &DischargePolicy::default(),
        })
        .expect("discharge");
        let handoff = kernel_handoff_for(&discharge, VcId::new(0));
        let context_changed = handoff.with_test_context_identity_producer_ref(
            0,
            VcFormulaRef::Generated(VcGeneratedFormulaId::new(1)),
        );

        assert_eq!(handoff.canonical_hash(), context_changed.canonical_hash());
        assert_ne!(
            handoff.context_identity_hash(),
            context_changed.context_identity_hash()
        );
        assert!(context_changed.context_identity_is_consistent());

        let base_slices = try_compute_dependency_slices_with_kernel_evidence(
            DependencySliceInput {
                vc_set: discharge.vc_set(),
                discharge_output: Some(&discharge),
            },
            &[KernelEvidenceDependencyInput {
                vc: VcId::new(0),
                handoff: &handoff,
            }],
        )
        .expect("base slices");
        let context_changed_slices = try_compute_dependency_slices_with_kernel_evidence(
            DependencySliceInput {
                vc_set: discharge.vc_set(),
                discharge_output: Some(&discharge),
            },
            &[KernelEvidenceDependencyInput {
                vc: VcId::new(0),
                handoff: &context_changed,
            }],
        )
        .expect("context-changed slices");
        assert_ne!(
            only_slice(&base_slices).fingerprint(),
            only_slice(&context_changed_slices).fingerprint()
        );
        let discharged_vc = discharge.vc_set().vc(VcId::new(0)).expect("discharged vc");
        let base_key_payload = proof_reuse_key_payload(
            discharged_vc,
            only_slice(&base_slices),
            &discharge,
            &handoff,
        )
        .expect("base key payload");
        let context_changed_key_payload = proof_reuse_key_payload(
            discharged_vc,
            only_slice(&context_changed_slices),
            &discharge,
            &context_changed,
        )
        .expect("context-changed key payload");
        let base_kernel_line = base_key_payload
            .lines()
            .find(|line| line.starts_with("kernel-evidence: "))
            .expect("base kernel line");
        let context_changed_kernel_line = context_changed_key_payload
            .lines()
            .find(|line| line.starts_with("kernel-evidence: "))
            .expect("context-changed kernel line");
        assert!(base_kernel_line.contains(&format!(
            "context-identity-hash={:?}",
            handoff.context_identity_hash()
        )));
        assert_ne!(base_kernel_line, context_changed_kernel_line);
        assert_ne!(
            base_slices
                .proof_reuse_key_for_kernel_handoff(&discharge, VcId::new(0), &handoff)
                .expect("base key"),
            context_changed_slices
                .proof_reuse_key_for_kernel_handoff(&discharge, VcId::new(0), &context_changed,)
                .expect("context-changed key")
        );
    }

    #[test]
    fn pre_existing_discharge_evidence_inputs_are_slice_dependencies() {
        let mut parts = fixture_parts(
            VcStatus::Discharged {
                evidence: DischargeEvidenceRef {
                    rule: VcText::new("task-11-definitional-reduction-v1"),
                    evidence_hash: HashMarker::Available(Hash::from_bytes([8; Hash::BYTE_LEN])),
                },
            },
            VcFormulaRef::Generated(VcGeneratedFormulaId::new(0)),
            generated_chain(),
            complete_anchor_fixture(),
        );
        parts.vcs[0].local_context = rich_context(parts.source);
        parts.vcs[0].premises = vec![
            PremiseRef::LocalContext(ContextEntryId::new(0)),
            PremiseRef::PermittedUnfolding {
                definition: CoreDefinitionId::new(1),
            },
        ];
        parts.vcs[0].proof_hint = Some(rich_hint());
        let original = fixture_set(parts);
        let discharge = try_discharge(DischargeInput {
            vc_set: &original,
            policy: &DischargePolicy::default(),
        })
        .expect("preserved discharge");

        let slices = try_compute_dependency_slices(DependencySliceInput {
            vc_set: discharge.vc_set(),
            discharge_output: Some(&discharge),
        })
        .expect("slice computation");
        let slice = only_slice(&slices);

        assert!(has_entry(
            slice,
            DependencyEntryClass::DischargeEvidence,
            "discharge:record:task-11-definitional-reduction-v1"
        ));
        assert!(has_entry(
            slice,
            DependencyEntryClass::LocalContext,
            "context:1"
        ));
        assert!(has_entry(
            slice,
            DependencyEntryClass::GeneratedFormula,
            "generated-formula:1"
        ));
        assert!(has_entry(
            slice,
            DependencyEntryClass::Definition,
            "definition:1"
        ));
        assert!(has_entry(
            slice,
            DependencyEntryClass::Definition,
            "definition:2"
        ));
        assert!(has_entry(
            slice,
            DependencyEntryClass::Policy,
            "discharge:policy:task-14-policy"
        ));
        assert!(has_entry(
            slice,
            DependencyEntryClass::Policy,
            "discharge:computation"
        ));
        assert_eq!(
            slice.completeness(),
            DependencySliceCompleteness::IncompleteUncacheable
        );
        assert!(slice.requires_cache_miss());
        assert!(slice.unknowns().iter().any(|unknown| {
            unknown.family() == DependencyUnknownFamily::DischargeEvidence
                && unknown.local_key() == "discharge:preserved-status-only"
        }));
    }

    #[test]
    fn discharged_status_without_replay_is_incomplete_uncacheable() {
        let set = fixture_set(fixture_parts(
            VcStatus::Discharged {
                evidence: DischargeEvidenceRef {
                    rule: VcText::new("manual"),
                    evidence_hash: HashMarker::Available(Hash::from_bytes([9; Hash::BYTE_LEN])),
                },
            },
            VcFormulaRef::Core(CoreFormulaId::new(0)),
            Vec::new(),
            complete_anchor_fixture(),
        ));

        let slices = try_compute_dependency_slices(DependencySliceInput {
            vc_set: &set,
            discharge_output: None,
        })
        .expect("slice computation");
        let slice = only_slice(&slices);

        assert_eq!(
            slice.completeness(),
            DependencySliceCompleteness::IncompleteUncacheable
        );
        assert!(slice.requires_cache_miss());
        assert!(slice.unknowns().iter().any(|unknown| {
            unknown.family() == DependencyUnknownFamily::DischargeEvidence
                && unknown.local_key() == "discharge:missing-replay"
        }));
    }

    #[test]
    fn unknown_markers_are_cache_miss_and_fingerprint_inputs() {
        let complete_set = fixture_set(fixture_parts(
            VcStatus::NeedsAtp,
            VcFormulaRef::Generated(VcGeneratedFormulaId::new(0)),
            vec![generated_formula(0, VcGeneratedFormulaShape::True)],
            complete_anchor_fixture(),
        ));
        let mut unknown_parts = fixture_parts(
            VcStatus::NeedsAtp,
            VcFormulaRef::Generated(VcGeneratedFormulaId::new(0)),
            vec![generated_formula(0, VcGeneratedFormulaShape::True)],
            complete_anchor_fixture(),
        );
        unknown_parts.vcs[0].premises = vec![PremiseRef::ConservativeUnknown {
            reason: VcText::new("opaque registration view"),
        }];
        let unknown_set = fixture_set(unknown_parts);

        let complete = try_compute_dependency_slices(DependencySliceInput {
            vc_set: &complete_set,
            discharge_output: None,
        })
        .expect("complete slice");
        let unknown = try_compute_dependency_slices(DependencySliceInput {
            vc_set: &unknown_set,
            discharge_output: None,
        })
        .expect("unknown slice");
        let complete_slice = only_slice(&complete);
        let unknown_slice = only_slice(&unknown);

        assert!(complete_slice.is_complete());
        assert!(!complete_slice.requires_cache_miss());
        assert_eq!(
            unknown_slice.completeness(),
            DependencySliceCompleteness::IncompleteUncacheable
        );
        assert!(unknown_slice.requires_cache_miss());
        assert_ne!(complete_slice.fingerprint(), unknown_slice.fingerprint());
    }

    #[test]
    fn unresolved_core_formula_payloads_are_independently_incomplete() {
        let set = fixture_set(fixture_parts(
            VcStatus::NeedsAtp,
            VcFormulaRef::Core(CoreFormulaId::new(0)),
            Vec::new(),
            complete_anchor_fixture(),
        ));

        let slices = try_compute_dependency_slices(DependencySliceInput {
            vc_set: &set,
            discharge_output: None,
        })
        .expect("core formula slice");
        let slice = only_slice(&slices);

        assert_eq!(
            slice.completeness(),
            DependencySliceCompleteness::IncompleteUncacheable
        );
        assert!(slice.unknowns().iter().any(|unknown| {
            unknown.family() == DependencyUnknownFamily::UpstreamPayload
                && unknown.local_key() == "core-formula:unresolved"
                && unknown.reason().contains("core formula payload")
        }));
    }

    #[test]
    fn unresolved_definition_payloads_are_independently_incomplete() {
        let mut parts = fixture_parts(
            VcStatus::NeedsAtp,
            VcFormulaRef::Generated(VcGeneratedFormulaId::new(0)),
            vec![generated_formula(0, VcGeneratedFormulaShape::True)],
            complete_anchor_fixture(),
        );
        parts.vcs[0].premises = vec![PremiseRef::DefinitionBoundary {
            definition: CoreDefinitionId::new(0),
        }];
        let set = fixture_set(parts);

        let slices = try_compute_dependency_slices(DependencySliceInput {
            vc_set: &set,
            discharge_output: None,
        })
        .expect("definition slice");
        let slice = only_slice(&slices);

        assert_eq!(
            slice.completeness(),
            DependencySliceCompleteness::IncompleteUncacheable
        );
        assert!(slice.unknowns().iter().any(|unknown| {
            unknown.family() == DependencyUnknownFamily::Definition
                && unknown.local_key() == "definition:unresolved"
                && unknown.reason().contains("definition payload")
        }));
    }

    #[test]
    fn unresolved_generated_payloads_are_independently_incomplete() {
        let set = fixture_set(fixture_parts(
            VcStatus::NeedsAtp,
            VcFormulaRef::Generated(VcGeneratedFormulaId::new(0)),
            vec![generated_formula(
                0,
                VcGeneratedFormulaShape::Diagnostic(CoreDiagnosticId::new(0)),
            )],
            complete_anchor_fixture(),
        ));

        let slices = try_compute_dependency_slices(DependencySliceInput {
            vc_set: &set,
            discharge_output: None,
        })
        .expect("generated diagnostic slice");
        let slice = only_slice(&slices);

        assert_eq!(
            slice.completeness(),
            DependencySliceCompleteness::IncompleteUncacheable
        );
        assert!(slice.unknowns().iter().any(|unknown| {
            unknown.family() == DependencyUnknownFamily::UpstreamPayload
                && unknown.local_key() == "generated-formula:unresolved"
                && unknown.reason().contains("generated formula payload")
        }));
    }

    #[test]
    fn quantified_binder_payloads_are_independently_incomplete() {
        let mut parts = fixture_parts(
            VcStatus::NeedsAtp,
            VcFormulaRef::Generated(VcGeneratedFormulaId::new(0)),
            vec![
                generated_formula(
                    0,
                    VcGeneratedFormulaShape::Quantified {
                        kind: crate::vc_ir::QuantifierKind::Forall,
                        binders: vec![ContextEntryId::new(0)],
                        body: VcFormulaRef::Generated(VcGeneratedFormulaId::new(1)),
                    },
                ),
                generated_formula(1, VcGeneratedFormulaShape::True),
            ],
            complete_anchor_fixture(),
        );
        parts.vcs[0].local_context = LocalContext::try_new(
            vec![ContextEntry {
                id: ContextEntryId::new(0),
                sort_key: CanonicalSortKey::new("000-binder"),
                kind: ContextEntryKind::BinderDeclaration {
                    var: CoreVarId::new(0),
                    role: VcText::new("binder"),
                },
                formula: None,
                provenance: vec![provenance("binder")],
            }],
            Vec::new(),
        )
        .expect("binder context");
        let set = fixture_set(parts);

        let slices = try_compute_dependency_slices(DependencySliceInput {
            vc_set: &set,
            discharge_output: None,
        })
        .expect("quantified binder slice");
        let slice = only_slice(&slices);

        assert_eq!(
            slice.completeness(),
            DependencySliceCompleteness::IncompleteUncacheable
        );
        assert!(slice.unknowns().iter().any(|unknown| {
            unknown.family() == DependencyUnknownFamily::UpstreamPayload
                && unknown.local_key() == "generated-formula:unresolved"
        }));
        assert!(slice.unknowns().iter().any(|unknown| {
            unknown.family() == DependencyUnknownFamily::UpstreamPayload
                && unknown.local_key() == "context:binder-declaration"
        }));
    }

    #[test]
    fn incomplete_anchor_unknowns_are_cache_miss_and_fingerprint_inputs() {
        let complete_set = fixture_set(fixture_parts(
            VcStatus::NeedsAtp,
            VcFormulaRef::Core(CoreFormulaId::new(0)),
            Vec::new(),
            complete_anchor_fixture(),
        ));
        let incomplete_set = fixture_set(fixture_parts(
            VcStatus::NeedsAtp,
            VcFormulaRef::Core(CoreFormulaId::new(0)),
            Vec::new(),
            incomplete_anchor_fixture(),
        ));

        let complete = try_compute_dependency_slices(DependencySliceInput {
            vc_set: &complete_set,
            discharge_output: None,
        })
        .expect("complete slice");
        let incomplete = try_compute_dependency_slices(DependencySliceInput {
            vc_set: &incomplete_set,
            discharge_output: None,
        })
        .expect("incomplete slice");
        let complete_slice = only_slice(&complete);
        let incomplete_slice = only_slice(&incomplete);

        assert!(incomplete_slice.requires_cache_miss());
        assert!(
            incomplete_slice
                .unknowns()
                .iter()
                .any(|unknown| unknown.family() == DependencyUnknownFamily::Anchor)
        );
        assert_ne!(complete_slice.fingerprint(), incomplete_slice.fingerprint());
    }

    #[test]
    fn reusable_fingerprint_excludes_snapshot_local_vc_id() {
        let entries = vec![DependencyEntry {
            class: DependencyEntryClass::Policy,
            local_key: "policy:stable".to_owned(),
            payload: "stable policy payload".to_owned(),
            fingerprint_payload: "stable policy payload".to_owned(),
        }];
        let first = DependencySlice {
            vc: VcId::new(0),
            kind: VcKind::TheoremProofStep,
            status: VcStatus::NeedsAtp,
            entries: entries.clone(),
            unknowns: Vec::new(),
            completeness: DependencySliceCompleteness::Complete,
            fingerprint: fingerprint_for(
                &VcKind::TheoremProofStep,
                &VcStatus::NeedsAtp,
                &entries,
                &[],
                DependencySliceCompleteness::Complete,
            ),
        };
        let second = DependencySlice {
            vc: VcId::new(1),
            fingerprint: fingerprint_for(
                &VcKind::TheoremProofStep,
                &VcStatus::NeedsAtp,
                &entries,
                &[],
                DependencySliceCompleteness::Complete,
            ),
            ..first.clone()
        };

        assert_ne!(first.vc(), second.vc());
        assert_eq!(first.fingerprint(), second.fingerprint());
    }

    #[test]
    fn reusable_fingerprint_normalizes_snapshot_local_discharge_hashes() {
        let first_status = VcStatus::Discharged {
            evidence: DischargeEvidenceRef {
                rule: VcText::new("task-11-generated-tautology-v1"),
                evidence_hash: HashMarker::Available(Hash::from_bytes([1; Hash::BYTE_LEN])),
            },
        };
        let second_status = VcStatus::Discharged {
            evidence: DischargeEvidenceRef {
                rule: VcText::new("task-11-generated-tautology-v1"),
                evidence_hash: HashMarker::Available(Hash::from_bytes([2; Hash::BYTE_LEN])),
            },
        };
        let first_entries = vec![DependencyEntry {
            class: DependencyEntryClass::DischargeEvidence,
            local_key: "discharge:evidence:status:task-11-generated-tautology-v1".to_owned(),
            payload: "evidence-hash=available:01".to_owned(),
            fingerprint_payload: "evidence-hash=available".to_owned(),
        }];
        let second_entries = vec![DependencyEntry {
            class: DependencyEntryClass::DischargeEvidence,
            local_key: "discharge:evidence:status:task-11-generated-tautology-v1".to_owned(),
            payload: "evidence-hash=available:02".to_owned(),
            fingerprint_payload: "evidence-hash=available".to_owned(),
        }];

        assert_eq!(
            fingerprint_for(
                &VcKind::TheoremProofStep,
                &first_status,
                &first_entries,
                &[],
                DependencySliceCompleteness::Complete,
            ),
            fingerprint_for(
                &VcKind::TheoremProofStep,
                &second_status,
                &second_entries,
                &[],
                DependencySliceCompleteness::Complete,
            )
        );
    }

    #[test]
    fn available_anchor_hash_values_are_fingerprint_inputs() {
        let first_set = fixture_set(fixture_parts(
            VcStatus::NeedsAtp,
            VcFormulaRef::Core(CoreFormulaId::new(0)),
            Vec::new(),
            complete_anchor_with_hashes(1, 2, 3),
        ));
        let second_set = fixture_set(fixture_parts(
            VcStatus::NeedsAtp,
            VcFormulaRef::Core(CoreFormulaId::new(0)),
            Vec::new(),
            complete_anchor_with_hashes(4, 2, 3),
        ));

        let first = try_compute_dependency_slices(DependencySliceInput {
            vc_set: &first_set,
            discharge_output: None,
        })
        .expect("first slice");
        let second = try_compute_dependency_slices(DependencySliceInput {
            vc_set: &second_set,
            discharge_output: None,
        })
        .expect("second slice");

        assert_ne!(
            only_slice(&first).fingerprint(),
            only_slice(&second).fingerprint()
        );
    }

    #[test]
    fn dependency_ordering_debug_and_fingerprints_are_deterministic() {
        let first = fixture_set(parts_with_premises(vec![
            PremiseRef::ReductionTrace {
                trace: VcText::new("z"),
            },
            PremiseRef::RegistrationTrace {
                trace: VcText::new("a"),
            },
        ]));
        let second = fixture_set(parts_with_premises(vec![
            PremiseRef::RegistrationTrace {
                trace: VcText::new("a"),
            },
            PremiseRef::ReductionTrace {
                trace: VcText::new("z"),
            },
        ]));

        let first_slices = try_compute_dependency_slices(DependencySliceInput {
            vc_set: &first,
            discharge_output: None,
        })
        .expect("first slice");
        let second_slices = try_compute_dependency_slices(DependencySliceInput {
            vc_set: &second,
            discharge_output: None,
        })
        .expect("second slice");

        assert_eq!(first_slices.slices(), second_slices.slices());
        assert_eq!(first_slices.debug_text(), second_slices.debug_text());
    }

    #[test]
    fn status_boundary_participates_in_fingerprint() {
        let statuses = vec![
            VcStatus::NeedsAtp,
            VcStatus::PolicyOpen {
                policy: PolicyKey::new("manual-review"),
            },
            VcStatus::AssumedByPolicy {
                policy: PolicyKey::new("trusted-import"),
                marker: PremiseRef::PolicyAssumption {
                    marker: PolicyKey::new("trusted-import"),
                },
            },
            VcStatus::SkippedDueToInvalidInput {
                reason: VcText::new("invalid source"),
            },
            VcStatus::DeferredExternal {
                reason: VcText::new("resolver unavailable"),
            },
            VcStatus::Error {
                diagnostic: CoreDiagnosticId::new(0),
            },
            VcStatus::Discharged {
                evidence: DischargeEvidenceRef {
                    rule: VcText::new("manual"),
                    evidence_hash: HashMarker::Available(Hash::from_bytes([7; Hash::BYTE_LEN])),
                },
            },
        ];
        let mut fingerprints = Vec::new();

        for status in statuses {
            let set = fixture_set(fixture_parts(
                status.clone(),
                VcFormulaRef::Core(CoreFormulaId::new(0)),
                Vec::new(),
                complete_anchor_fixture(),
            ));
            let slices = try_compute_dependency_slices(DependencySliceInput {
                vc_set: &set,
                discharge_output: None,
            })
            .expect("status slice");
            let slice = only_slice(&slices);

            assert_eq!(slice.status(), &status);
            let expected_status = format!("{status:?}");
            assert_eq!(
                entry_payload(slice, DependencyEntryClass::Policy, "status:boundary"),
                Some(expected_status.as_str())
            );
            fingerprints.push(slice.fingerprint());
        }

        for (index, fingerprint) in fingerprints.iter().enumerate() {
            assert!(
                fingerprints[index + 1..]
                    .iter()
                    .all(|other| other != fingerprint),
                "status fingerprint at index {index} should be unique"
            );
        }
    }

    fn fixture_set(parts: VcSetParts) -> VcSet {
        VcSet::try_new(parts).expect("valid dependency-slice fixture")
    }

    fn two_vc_same_goal_fixture() -> VcSet {
        let mut parts = fixture_parts(
            VcStatus::Open,
            VcFormulaRef::Generated(VcGeneratedFormulaId::new(0)),
            vec![generated_formula(0, VcGeneratedFormulaShape::True)],
            complete_anchor_fixture(),
        );
        let mut second_vc = parts.vcs[0].clone();
        second_vc.id = VcId::new(1);
        second_vc.seed.handoff = ObligationHandoffId::new(1);
        second_vc.anchor = complete_anchor_fixture().with_source(parts.source);
        second_vc.provenance = vec![provenance("vc-second")];
        parts.vcs.push(second_vc);
        parts.seed_accounting.push(SeedAccounting {
            handoff: ObligationHandoffId::new(1),
            origin: SeedOriginRef::ExistingCore {
                seed: ObligationSeedId::new(1),
            },
            seed_status: ObligationSeedStatus::Active,
            mapping: SeedVcMapping::One { vc: VcId::new(1) },
        });
        fixture_set(parts)
    }

    fn fixture_parts(
        status: VcStatus,
        goal: VcFormulaRef,
        generated_formulas: Vec<VcGeneratedFormula>,
        anchor: ObligationAnchor,
    ) -> VcSetParts {
        let snapshot = sample_snapshot_id();
        let source = InMemorySessionIdAllocator::new()
            .next_source_id(snapshot)
            .expect("source id");
        let source_ref = source_ref(source);
        let handoff = ObligationHandoffId::new(0);

        VcSetParts {
            schema_version: VcSchemaVersion::new("dependency-slice-test-v1"),
            snapshot,
            source,
            module: VcModuleRef::new("sample"),
            generated_formulas,
            vcs: vec![VcIr {
                id: VcId::new(0),
                kind: VcKind::TheoremProofStep,
                source: VcSourceRef {
                    primary: source_ref,
                    related: vec![generated_source_ref()],
                },
                seed: SeedVcRef { handoff },
                anchor: anchor.with_source(source),
                local_context: LocalContext::try_new(Vec::new(), Vec::new())
                    .expect("empty context"),
                premises: Vec::new(),
                goal,
                proof_hint: None,
                status,
                provenance: vec![provenance("vc")],
            }],
            seed_accounting: vec![SeedAccounting {
                handoff,
                origin: SeedOriginRef::ExistingCore {
                    seed: ObligationSeedId::new(0),
                },
                seed_status: ObligationSeedStatus::Active,
                mapping: SeedVcMapping::One { vc: VcId::new(0) },
            }],
        }
    }

    fn parts_with_premises(premises: Vec<PremiseRef>) -> VcSetParts {
        let mut parts = fixture_parts(
            VcStatus::NeedsAtp,
            VcFormulaRef::Core(CoreFormulaId::new(0)),
            Vec::new(),
            complete_anchor_fixture(),
        );
        parts.vcs[0].premises = premises;
        parts
    }

    fn generated_chain() -> Vec<VcGeneratedFormula> {
        vec![
            generated_formula(
                0,
                VcGeneratedFormulaShape::And(vec![
                    VcFormulaRef::Core(CoreFormulaId::new(10)),
                    VcFormulaRef::Generated(VcGeneratedFormulaId::new(1)),
                ]),
            ),
            generated_formula(
                1,
                VcGeneratedFormulaShape::Ref(VcFormulaRef::Core(CoreFormulaId::new(11))),
            ),
        ]
    }

    fn generated_formula(index: usize, shape: VcGeneratedFormulaShape) -> VcGeneratedFormula {
        VcGeneratedFormula {
            id: VcGeneratedFormulaId::new(index),
            kind: VcGeneratedFormulaKind::GeneratedTypeObligation,
            shape,
            provenance: vec![provenance("generated")],
        }
    }

    fn rich_context(source: SourceId) -> LocalContext {
        LocalContext::try_new(
            vec![
                ContextEntry {
                    id: ContextEntryId::new(0),
                    sort_key: CanonicalSortKey::new("000-used"),
                    kind: ContextEntryKind::ProofAssumption,
                    formula: Some(VcFormulaRef::Core(CoreFormulaId::new(20))),
                    provenance: vec![provenance("context-used")],
                },
                ContextEntry {
                    id: ContextEntryId::new(1),
                    sort_key: CanonicalSortKey::new("001-unused"),
                    kind: ContextEntryKind::ProofAssumption,
                    formula: Some(VcFormulaRef::Core(CoreFormulaId::new(99))),
                    provenance: vec![provenance("context-unused")],
                },
                ContextEntry {
                    id: ContextEntryId::new(2),
                    sort_key: CanonicalSortKey::new("002-policy"),
                    kind: ContextEntryKind::VerifierPolicyInput {
                        key: PolicyKey::new("task-14-context-policy"),
                    },
                    formula: None,
                    provenance: vec![VcProvenance {
                        phase: VcProvenancePhase::CoreHandoff,
                        key: VcText::new("source"),
                        core: source_ref(source).provenance.first().cloned(),
                    }],
                },
            ],
            vec![VerifierPolicyInput {
                sort_key: CanonicalSortKey::new("000-policy"),
                key: PolicyKey::new("task-14-policy"),
                value: PolicyValue::new("enabled"),
            }],
        )
        .expect("rich context")
    }

    fn rich_hint() -> ProofHint {
        ProofHint {
            citations: vec![
                PremiseRef::GeneratedFact {
                    formula: VcFormulaRef::Generated(VcGeneratedFormulaId::new(1)),
                },
                PremiseRef::LocalLabel {
                    label: CoreLabelRef::new("A1"),
                },
            ],
            unfold_requests: vec![DefinitionUnfoldRequest {
                definition: CoreDefinitionId::new(2),
                opacity_override: Some(DefinitionOpacityOverride::Transparent),
            }],
            premise_restrictions: vec![
                PremiseRestriction::Only(vec![PremiseRef::LocalContext(ContextEntryId::new(0))]),
                PremiseRestriction::Intent(ProofHintKey::new("task-14-intent")),
            ],
            solver: Some(ProofHintKey::new("task-14-solver")),
            max_axioms: Some(32),
            timeout: Some(ProofHintKey::new("task-14-timeout")),
            computation: Some(ComputationHint::SymbolicRequest(ProofHintKey::new(
                "task-14-symbolic-computation",
            ))),
            provenance: vec![provenance("hint")],
        }
    }

    fn complete_anchor_fixture() -> ObligationAnchor {
        complete_anchor_with_hashes(1, 2, 3)
    }

    fn complete_anchor_with_hashes(
        source_shape_hash: u8,
        canonical_goal_hash: u8,
        canonical_context_hash: u8,
    ) -> ObligationAnchor {
        ObligationAnchor {
            owner: AnchorOwner::Theorem(CoreItemId::new(0)),
            kind: VcKind::TheoremProofStep,
            local_path: LocalProofOrProgramPath::new("proof/step/0"),
            label: Some(AnchorLabel {
                role: AnchorLabelRole::UserLabel,
                hint: Some(CoreLabelRef::new("A1")),
            }),
            semantic_origin: NormalizedSemanticOrigin::new("theorem:sample:proof-step:0"),
            source_range: None,
            provenance: vec![provenance("anchor")],
            source_shape_hash: available_hash(source_shape_hash),
            canonical_goal_hash: available_hash(canonical_goal_hash),
            canonical_context_hash: available_hash(canonical_context_hash),
            generation_schema_version: GenerationSchemaVersion::new("task-14-test"),
            completeness: AnchorCompleteness::Complete,
        }
    }

    fn incomplete_anchor_fixture() -> ObligationAnchor {
        let mut anchor = complete_anchor_fixture();
        anchor.source_shape_hash = HashMarker::Unavailable {
            reason: AnchorUnavailableReason::new("task-14 fixture lacks source-shape hash"),
        };
        anchor.canonical_goal_hash = HashMarker::ConservativeUnknown {
            reason: AnchorUnavailableReason::new("task-14 fixture has opaque goal hash"),
        };
        anchor.completeness = AnchorCompleteness::Incomplete {
            missing: vec![
                AnchorIngredient::SourceShapeHash,
                AnchorIngredient::CanonicalGoalHash,
            ],
        };
        anchor
    }

    trait TestAnchorExt {
        fn with_source(self, source: SourceId) -> Self;
    }

    impl TestAnchorExt for ObligationAnchor {
        fn with_source(mut self, source: SourceId) -> Self {
            self.source_range = Some(SourceRange {
                source_id: source,
                start: 0,
                end: 10,
            });
            self.provenance.push(VcProvenance {
                phase: VcProvenancePhase::CoreHandoff,
                key: VcText::new("source-ref"),
                core: source_ref(source).provenance.first().cloned(),
            });
            self
        }
    }

    fn kernel_handoff_for(discharge: &DischargeOutput, vc: VcId) -> VcKernelEvidenceHandoff {
        let vc_set = discharge.vc_set();
        let payloads = vc_set
            .generated_formulas()
            .iter()
            .map(|formula| KernelFormulaPayload {
                formula_ref: VcFormulaRef::Generated(formula.id),
                projection: KernelFormulaProjection {
                    formula_fingerprint: KernelEvidenceFingerprint::new(
                        KERNEL_FORMULA_FINGERPRINT_ALGORITHM_ID,
                        format!("formula-{}", formula.id.index()).into_bytes(),
                    )
                    .expect("formula fingerprint"),
                    formula_bytes: format!("kernel-formula-{}", formula.id.index()).into_bytes(),
                    provenance_payload: format!("provenance-{}", formula.id.index()).into_bytes(),
                },
            })
            .collect::<Vec<_>>();
        build_kernel_evidence_handoff(KernelEvidenceHandoffInput {
            vc_set,
            vc,
            goal_polarity: KernelGoalPolarity::AssertFalseForRefutation,
            kernel_profile: KernelEvidenceProfile::v1(1, KernelClauseTautologyPolicy::Reject),
            symbol_manifest: &[],
            variable_manifest: &[],
            formula_payloads: &payloads,
            imported_formula_payloads: &[],
            substitutions: &[],
            formula_context: None,
            discharge_output: Some(discharge),
        })
        .expect("kernel evidence handoff")
    }

    fn only_slice(output: &DependencySliceSet) -> &DependencySlice {
        assert_eq!(output.slices().len(), 1);
        &output.slices()[0]
    }

    fn has_entry(slice: &DependencySlice, class: DependencyEntryClass, local_key: &str) -> bool {
        entry_payload(slice, class, local_key).is_some()
    }

    fn entry_payload<'a>(
        slice: &'a DependencySlice,
        class: DependencyEntryClass,
        local_key: &str,
    ) -> Option<&'a str> {
        slice
            .entries()
            .iter()
            .find(|entry| entry.class() == class && entry.local_key() == local_key)
            .map(DependencyEntry::payload)
    }

    fn sample_snapshot_id() -> BuildSnapshotId {
        BuildSnapshotId::from_published_schema_str(
            "mizar-session-build-snapshot-v1:\
             1111111111111111111111111111111111111111111111111111111111111111",
        )
        .expect("snapshot id")
    }

    fn source_ref(source: SourceId) -> CoreSourceRef {
        CoreSourceRef::direct(SourceRange {
            source_id: source,
            start: 0,
            end: 10,
        })
        .with_provenance(vec![CoreProvenance::new(
            CoreProvenancePhase::ProofSkeleton,
            "direct-source",
        )])
    }

    fn generated_source_ref() -> CoreSourceRef {
        CoreSourceRef::generated(GeneratedFrom {
            owner: CoreNodeRef::Item(CoreItemId::new(0)),
            kind: GeneratedOriginKind::TypePredicate,
            key: GeneratedOriginKey::new("generated-type"),
            reason: CoreProvenanceKey::new("task-14-test"),
        })
    }

    fn provenance(key: &str) -> VcProvenance {
        VcProvenance {
            phase: VcProvenancePhase::Generator,
            key: VcText::new(key),
            core: None,
        }
    }

    fn available_hash(byte: u8) -> HashMarker {
        HashMarker::Available(Hash::from_bytes([byte; Hash::BYTE_LEN]))
    }
}
