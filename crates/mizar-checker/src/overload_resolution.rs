//! Checker-local overload site and candidate collection for phase 8.

use crate::typed_ast::{NormalizedTypeId, TypeFactId, TypedSiteRef};
use mizar_resolve::resolved_ast::{ModuleId, SymbolId};
use mizar_session::SourceRange;
use std::{
    collections::BTreeMap,
    fmt::{self, Write as _},
};

macro_rules! dense_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(usize);

        impl $name {
            pub const fn new(index: usize) -> Self {
                Self(index)
            }

            pub const fn index(self) -> usize {
                self.0
            }
        }
    };
}

macro_rules! string_key {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self::new(value)
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self::new(value)
            }
        }
    };
}

dense_id!(OverloadSiteId);
dense_id!(OverloadCandidateId);
dense_id!(OverloadDiagnosticId);
dense_id!(TemplateExpansionId);

string_key!(OverloadSiteKey);
string_key!(OverloadNameKey);
string_key!(OverloadDiagnosticMessageKey);
string_key!(CandidateProvenanceKey);
string_key!(TemplateInstantiationKey);
string_key!(TemplateParameterKey);
string_key!(QuaPathKey);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverloadCollectionOutput {
    sites: OverloadSiteTable,
    candidates: OverloadCandidateTable,
    diagnostics: OverloadDiagnosticTable,
}

impl OverloadCollectionOutput {
    pub fn collect(
        sites: impl IntoIterator<Item = OverloadSiteInput>,
        candidates: impl IntoIterator<Item = OverloadCandidateInput>,
    ) -> Self {
        OverloadCollectionBuilder::new(sites, candidates).finish()
    }

    pub const fn sites(&self) -> &OverloadSiteTable {
        &self.sites
    }

    pub const fn candidates(&self) -> &OverloadCandidateTable {
        &self.candidates
    }

    pub const fn diagnostics(&self) -> &OverloadDiagnosticTable {
        &self.diagnostics
    }

    pub fn debug_text(&self) -> String {
        let mut output = String::from("overload-collection-debug-v1\n");
        write_sites(&mut output, &self.sites);
        write_candidates(&mut output, &self.candidates);
        write_diagnostics(&mut output, &self.diagnostics);
        output
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateExpansionOutput {
    candidates: OverloadCandidateTable,
    expansions: TemplateExpansionTable,
    diagnostics: OverloadDiagnosticTable,
}

impl TemplateExpansionOutput {
    pub fn expand(collection: &OverloadCollectionOutput) -> Self {
        TemplateExpansionBuilder::new(collection).finish()
    }

    pub const fn candidates(&self) -> &OverloadCandidateTable {
        &self.candidates
    }

    pub const fn expansions(&self) -> &TemplateExpansionTable {
        &self.expansions
    }

    pub const fn diagnostics(&self) -> &OverloadDiagnosticTable {
        &self.diagnostics
    }

    pub fn debug_text(&self) -> String {
        let mut output = String::from("template-expansion-debug-v1\n");
        write_candidates(&mut output, &self.candidates);
        write_template_expansions(&mut output, &self.expansions);
        write_diagnostics(&mut output, &self.diagnostics);
        output
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverloadSiteInput {
    pub key: OverloadSiteKey,
    pub owner: TypedSiteRef,
    pub source_range: SourceRange,
    pub kind: OverloadSiteKind,
    pub name: OverloadNameKey,
    pub arguments: Vec<TypedSiteRef>,
    pub expected: Option<NormalizedTypeId>,
    pub source_qua: Vec<SourceQuaView>,
    pub recovery: OverloadSiteRecovery,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceQuaView {
    pub argument: TypedSiteRef,
    pub target: NormalizedTypeId,
    pub source_range: SourceRange,
    pub path: Option<QuaPathKey>,
    pub evidence_facts: Vec<TypeFactId>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum OverloadSiteKind {
    FunctorApplication,
    PredicateApplication,
    AttributeApplication,
    ModeApplication,
    SelectorApplication,
    StructureFieldApplication,
    TemplateName,
    Unsupported(UnsupportedOverloadRole),
}

impl OverloadSiteKind {
    pub const fn is_supported(&self) -> bool {
        !matches!(self, Self::Unsupported(_))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum UnsupportedOverloadRole {
    SchemeApplication,
    TheoremApplication,
    AlgorithmTemplate,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum OverloadSiteRecovery {
    Normal,
    Degraded {
        message_key: OverloadDiagnosticMessageKey,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverloadCandidateInput {
    pub site: OverloadSiteKey,
    pub symbol: SymbolId,
    pub ordinary_root: SymbolId,
    pub declaration_kind: CandidateDeclarationKind,
    pub parameters: Vec<NormalizedTypeId>,
    pub result: Option<NormalizedTypeId>,
    pub origin: CandidateOrigin,
    pub template: Option<TemplateCandidatePayload>,
    pub coherence: Option<CoherenceStatus>,
    pub provenance: CandidateProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum CandidateDeclarationKind {
    Functor,
    Predicate,
    Attribute,
    Mode,
    Selector,
    StructureField,
    Template,
    Redefinition,
    Unsupported(UnsupportedOverloadRole),
}

impl CandidateDeclarationKind {
    pub const fn is_supported(&self) -> bool {
        !matches!(self, Self::Unsupported(_))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum CandidateOrigin {
    Ordinary,
    Redefinition {
        refined: SymbolId,
    },
    TemplateDerived {
        template: SymbolId,
        instantiation: TemplateInstantiationKey,
    },
    Recovery(CandidateProvenanceKey),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum CoherenceStatus {
    Accepted,
    Pending,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateCandidatePayload {
    pub template: SymbolId,
    pub instantiation_key: TemplateInstantiationKey,
    pub parameters: Vec<TemplateParameterKey>,
    pub arguments: Vec<TemplateArgument>,
    pub inferred_arguments: Vec<TemplateArgumentInference>,
    pub constraints: Vec<TemplateConstraintEvidence>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum TemplateArgument {
    Explicit(NormalizedTypeId),
    Omitted(TemplateParameterKey),
    SourceQua {
        source: NormalizedTypeId,
        target: NormalizedTypeId,
        path: QuaPathKey,
        status: TemplateQuaStatus,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum TemplateQuaStatus {
    AcceptedWidening,
    RejectedNarrowing,
    DeferredExternalDependency,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateArgumentInference {
    pub parameter: TemplateParameterKey,
    pub inferred: NormalizedTypeId,
    pub evidence_key: CandidateProvenanceKey,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateConstraintEvidence {
    pub parameter: TemplateParameterKey,
    pub evidence_key: CandidateProvenanceKey,
    pub facts: Vec<TypeFactId>,
    pub status: TemplateConstraintEvidenceStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum TemplateConstraintEvidenceStatus {
    Accepted,
    Missing,
    DeferredExternalDependency,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateProvenance {
    pub stable_key: CandidateProvenanceKey,
    pub source_range: Option<SourceRange>,
    pub scope: CandidateScope,
    pub declaration_order: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum CandidateScope {
    Local,
    Imported {
        module: ModuleId,
        import_key: CandidateProvenanceKey,
    },
    DependencySummary {
        module: ModuleId,
        summary_key: CandidateProvenanceKey,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverloadSite {
    pub id: OverloadSiteId,
    pub key: OverloadSiteKey,
    pub owner: TypedSiteRef,
    pub source_range: SourceRange,
    pub kind: OverloadSiteKind,
    pub name: OverloadNameKey,
    pub arguments: Vec<TypedSiteRef>,
    pub expected: Option<NormalizedTypeId>,
    pub source_qua: Vec<SourceQuaView>,
    pub recovery: OverloadSiteRecovery,
    pub status: OverloadSiteStatus,
    pub diagnostics: Vec<OverloadDiagnosticId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum OverloadSiteStatus {
    Collected,
    Degraded,
    Deferred,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OverloadSiteTable {
    entries: Vec<OverloadSite>,
}

impl OverloadSiteTable {
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn get(&self, id: OverloadSiteId) -> Option<&OverloadSite> {
        self.entries.get(id.index())
    }

    pub fn iter(&self) -> impl Iterator<Item = (OverloadSiteId, &OverloadSite)> {
        self.entries.iter().map(|entry| (entry.id, entry))
    }

    pub fn canonical_iter(&self) -> impl Iterator<Item = (OverloadSiteId, &OverloadSite)> {
        let mut entries = self.entries.iter().collect::<Vec<_>>();
        entries.sort_by_key(|entry| site_output_key(entry));
        entries.into_iter().map(|entry| (entry.id, entry))
    }

    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn push(&mut self, site: OverloadSite) {
        self.entries.push(site);
    }

    fn get_mut(&mut self, id: OverloadSiteId) -> Option<&mut OverloadSite> {
        self.entries.get_mut(id.index())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverloadCandidate {
    pub id: OverloadCandidateId,
    pub site: OverloadSiteId,
    pub site_key: OverloadSiteKey,
    pub symbol: SymbolId,
    pub ordinary_root: SymbolId,
    pub declaration_kind: CandidateDeclarationKind,
    pub parameters: Vec<NormalizedTypeId>,
    pub result: Option<NormalizedTypeId>,
    pub origin: CandidateOrigin,
    pub template: Option<TemplateCandidatePayload>,
    pub coherence: Option<CoherenceStatus>,
    pub provenance: CandidateProvenance,
    pub status: OverloadCandidateStatus,
    pub diagnostics: Vec<OverloadDiagnosticId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum OverloadCandidateStatus {
    Collected,
    Deferred,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OverloadCandidateTable {
    entries: Vec<OverloadCandidate>,
}

impl OverloadCandidateTable {
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn get(&self, id: OverloadCandidateId) -> Option<&OverloadCandidate> {
        self.entries.get(id.index())
    }

    pub fn iter(&self) -> impl Iterator<Item = (OverloadCandidateId, &OverloadCandidate)> {
        self.entries.iter().map(|entry| (entry.id, entry))
    }

    pub fn canonical_iter(
        &self,
    ) -> impl Iterator<Item = (OverloadCandidateId, &OverloadCandidate)> {
        let mut entries = self.entries.iter().collect::<Vec<_>>();
        entries.sort_by_key(|entry| candidate_output_key(entry));
        entries.into_iter().map(|entry| (entry.id, entry))
    }

    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn push(&mut self, candidate: OverloadCandidate) {
        self.entries.push(candidate);
    }

    fn get_mut(&mut self, id: OverloadCandidateId) -> Option<&mut OverloadCandidate> {
        self.entries.get_mut(id.index())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateExpansion {
    pub id: TemplateExpansionId,
    pub source_candidate: OverloadCandidateId,
    pub site: OverloadSiteId,
    pub template: SymbolId,
    pub instantiation_key: TemplateInstantiationKey,
    pub substitutions: Vec<TemplateSubstitution>,
    pub instantiated_candidate: Option<OverloadCandidateId>,
    pub status: TemplateExpansionStatus,
    pub diagnostics: Vec<OverloadDiagnosticId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TemplateExpansionTable {
    entries: Vec<TemplateExpansion>,
}

impl TemplateExpansionTable {
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn get(&self, id: TemplateExpansionId) -> Option<&TemplateExpansion> {
        self.entries.get(id.index())
    }

    pub fn iter(&self) -> impl Iterator<Item = (TemplateExpansionId, &TemplateExpansion)> {
        self.entries.iter().map(|entry| (entry.id, entry))
    }

    pub fn canonical_iter(
        &self,
    ) -> impl Iterator<Item = (TemplateExpansionId, &TemplateExpansion)> {
        let mut entries = self.entries.iter().collect::<Vec<_>>();
        entries.sort_by_key(|entry| template_expansion_order_key(entry));
        entries.into_iter().map(|entry| (entry.id, entry))
    }

    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn push(&mut self, expansion: TemplateExpansion) {
        self.entries.push(expansion);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateSubstitution {
    pub parameter: TemplateParameterKey,
    pub value: NormalizedTypeId,
    pub source: TemplateSubstitutionSource,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum TemplateSubstitutionSource {
    Explicit,
    OmittedInference {
        evidence_key: CandidateProvenanceKey,
    },
    SourceQua {
        source: NormalizedTypeId,
        path: QuaPathKey,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum TemplateExpansionStatus {
    Instantiated,
    Rejected(TemplateExpansionFailure),
    Deferred(TemplateExpansionFailure),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum TemplateExpansionFailure {
    ArityMismatch,
    DuplicateParameter,
    ParameterMismatch,
    MissingInference,
    AmbiguousInference,
    UnknownConstraintParameter,
    MissingConstraintEvidence,
    DeferredConstraintEvidence,
    RejectedSourceQua,
    DeferredSourceQua,
    DeferredCandidate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverloadDiagnostic {
    pub id: OverloadDiagnosticId,
    pub site: Option<OverloadSiteId>,
    pub site_key: Option<OverloadSiteKey>,
    pub candidate: Option<OverloadCandidateId>,
    pub provenance: Option<OverloadDiagnosticProvenance>,
    pub class: OverloadDiagnosticClass,
    pub severity: OverloadDiagnosticSeverity,
    pub message_key: OverloadDiagnosticMessageKey,
    pub recovery: OverloadDiagnosticRecovery,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverloadDiagnosticDraft {
    pub site: Option<OverloadSiteId>,
    pub site_key: Option<OverloadSiteKey>,
    pub candidate: Option<OverloadCandidateId>,
    pub provenance: Option<OverloadDiagnosticProvenance>,
    pub class: OverloadDiagnosticClass,
    pub severity: OverloadDiagnosticSeverity,
    pub message_key: OverloadDiagnosticMessageKey,
    pub recovery: OverloadDiagnosticRecovery,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum OverloadDiagnosticProvenance {
    SiteInput {
        owner: TypedSiteRef,
        source_range: SourceRange,
        kind: OverloadSiteKind,
        name: OverloadNameKey,
        arguments: Vec<TypedSiteRef>,
        source_qua: Vec<SourceQuaView>,
        recovery: OverloadSiteRecovery,
    },
    CandidateInput {
        symbol: SymbolId,
        ordinary_root: SymbolId,
        declaration_kind: CandidateDeclarationKind,
        template: Option<Box<TemplateCandidatePayload>>,
        coherence: Option<CoherenceStatus>,
        provenance: CandidateProvenance,
    },
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OverloadDiagnosticTable {
    entries: Vec<OverloadDiagnostic>,
}

impl OverloadDiagnosticTable {
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn insert(&mut self, draft: OverloadDiagnosticDraft) -> OverloadDiagnosticId {
        let id = OverloadDiagnosticId::new(self.entries.len());
        self.entries.push(OverloadDiagnostic {
            id,
            site: draft.site,
            site_key: draft.site_key,
            candidate: draft.candidate,
            provenance: draft.provenance,
            class: draft.class,
            severity: draft.severity,
            message_key: draft.message_key,
            recovery: draft.recovery,
        });
        id
    }

    pub fn get(&self, id: OverloadDiagnosticId) -> Option<&OverloadDiagnostic> {
        self.entries.get(id.index())
    }

    pub fn iter(&self) -> impl Iterator<Item = (OverloadDiagnosticId, &OverloadDiagnostic)> {
        self.entries.iter().map(|entry| (entry.id, entry))
    }

    pub fn canonical_iter(
        &self,
    ) -> impl Iterator<Item = (OverloadDiagnosticId, &OverloadDiagnostic)> {
        let mut entries = self.entries.iter().collect::<Vec<_>>();
        entries.sort_by_key(|entry| diagnostic_output_key(entry));
        entries.into_iter().map(|entry| (entry.id, entry))
    }

    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum OverloadDiagnosticClass {
    DuplicateSiteKey,
    MissingSite,
    UnsupportedSiteRole,
    UnsupportedCandidateRole,
    TemplateExpansion,
    Recovery,
    ExternalDependencyGap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum OverloadDiagnosticSeverity {
    Error,
    Warning,
    Note,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum OverloadDiagnosticRecovery {
    Normal,
    Degraded,
}

type SiteRefOrderKey = (usize, u8, String);
type RangeOrderKey = (String, usize, usize);
type OptionalRangeOrderKey = (u8, String, usize, usize);
type SiteInputOrderKey = (
    RangeOrderKey,
    SiteRefOrderKey,
    OverloadSiteKind,
    String,
    String,
    usize,
);
type SiteOutputOrderKey = (
    RangeOrderKey,
    SiteRefOrderKey,
    OverloadSiteKind,
    String,
    String,
    usize,
);
type CandidateInputOrderKey = (
    usize,
    String,
    CandidateDeclarationKind,
    SymbolId,
    SymbolId,
    String,
    CandidateProvenanceOrderKey,
    usize,
    usize,
);
type CandidateOutputOrderKey = (
    usize,
    CandidateDeclarationKind,
    SymbolId,
    SymbolId,
    String,
    CandidateProvenanceOrderKey,
    usize,
);
type CandidateProvenanceOrderKey = (u8, String, String, OptionalRangeOrderKey, usize, String);
type DiagnosticOutputOrderKey = (
    u8,
    usize,
    String,
    u8,
    usize,
    OverloadDiagnosticClass,
    String,
    usize,
);
type TemplateExpansionOrderKey = (usize, String, usize);

struct SiteInputWithOrder {
    input: OverloadSiteInput,
    ordinal: usize,
}

struct CandidateInputWithOrder {
    input: OverloadCandidateInput,
    ordinal: usize,
}

struct OverloadCollectionBuilder {
    site_inputs: Vec<SiteInputWithOrder>,
    candidate_inputs: Vec<CandidateInputWithOrder>,
    sites: OverloadSiteTable,
    candidates: OverloadCandidateTable,
    diagnostics: OverloadDiagnosticTable,
    site_ids: BTreeMap<OverloadSiteKey, OverloadSiteId>,
}

impl OverloadCollectionBuilder {
    fn new(
        sites: impl IntoIterator<Item = OverloadSiteInput>,
        candidates: impl IntoIterator<Item = OverloadCandidateInput>,
    ) -> Self {
        Self {
            site_inputs: sites
                .into_iter()
                .enumerate()
                .map(|(ordinal, input)| SiteInputWithOrder { input, ordinal })
                .collect(),
            candidate_inputs: candidates
                .into_iter()
                .enumerate()
                .map(|(ordinal, input)| CandidateInputWithOrder { input, ordinal })
                .collect(),
            sites: OverloadSiteTable::new(),
            candidates: OverloadCandidateTable::new(),
            diagnostics: OverloadDiagnosticTable::new(),
            site_ids: BTreeMap::new(),
        }
    }

    fn finish(mut self) -> OverloadCollectionOutput {
        self.collect_sites();
        self.collect_candidates();
        OverloadCollectionOutput {
            sites: self.sites,
            candidates: self.candidates,
            diagnostics: self.diagnostics,
        }
    }

    fn collect_sites(&mut self) {
        self.site_inputs
            .sort_by_key(|input| site_input_order_key(&input.input, input.ordinal));
        let inputs = std::mem::take(&mut self.site_inputs);

        for input in inputs {
            if self.site_ids.contains_key(&input.input.key) {
                self.insert_diagnostic(OverloadDiagnosticDraft {
                    site: None,
                    site_key: Some(input.input.key.clone()),
                    candidate: None,
                    provenance: Some(site_diagnostic_provenance(&input.input)),
                    class: OverloadDiagnosticClass::DuplicateSiteKey,
                    severity: OverloadDiagnosticSeverity::Error,
                    message_key: OverloadDiagnosticMessageKey::new("overload.site.duplicate_key"),
                    recovery: OverloadDiagnosticRecovery::Degraded,
                });
                continue;
            }

            let site_id = OverloadSiteId::new(self.sites.len());
            let status = site_status(&input.input);
            let site_key = input.input.key.clone();
            let diagnostic_provenance = site_diagnostic_provenance(&input.input);
            self.site_ids.insert(site_key.clone(), site_id);
            self.sites.push(OverloadSite {
                id: site_id,
                key: site_key.clone(),
                owner: input.input.owner,
                source_range: input.input.source_range,
                kind: input.input.kind.clone(),
                name: input.input.name,
                arguments: input.input.arguments,
                expected: input.input.expected,
                source_qua: input.input.source_qua,
                recovery: input.input.recovery.clone(),
                status,
                diagnostics: Vec::new(),
            });

            if let Some(message_key) = unsupported_site_message_key(&input.input.kind) {
                let diagnostic = self.insert_diagnostic(OverloadDiagnosticDraft {
                    site: Some(site_id),
                    site_key: Some(site_key.clone()),
                    candidate: None,
                    provenance: Some(diagnostic_provenance.clone()),
                    class: OverloadDiagnosticClass::UnsupportedSiteRole,
                    severity: OverloadDiagnosticSeverity::Note,
                    message_key,
                    recovery: OverloadDiagnosticRecovery::Degraded,
                });
                self.attach_site_diagnostic(site_id, diagnostic);
            }

            if let OverloadSiteRecovery::Degraded { message_key } = input.input.recovery {
                let diagnostic = self.insert_diagnostic(OverloadDiagnosticDraft {
                    site: Some(site_id),
                    site_key: Some(site_key),
                    candidate: None,
                    provenance: Some(diagnostic_provenance),
                    class: OverloadDiagnosticClass::Recovery,
                    severity: OverloadDiagnosticSeverity::Note,
                    message_key,
                    recovery: OverloadDiagnosticRecovery::Degraded,
                });
                self.attach_site_diagnostic(site_id, diagnostic);
            }
        }
    }

    fn collect_candidates(&mut self) {
        let site_ids = self.site_ids.clone();
        self.candidate_inputs
            .sort_by_key(|input| candidate_input_order_key(&input.input, input.ordinal, &site_ids));
        let inputs = std::mem::take(&mut self.candidate_inputs);

        for input in inputs {
            let Some(site_id) = self.site_ids.get(&input.input.site).copied() else {
                self.insert_diagnostic(OverloadDiagnosticDraft {
                    site: None,
                    site_key: Some(input.input.site.clone()),
                    candidate: None,
                    provenance: Some(candidate_diagnostic_provenance(&input.input)),
                    class: OverloadDiagnosticClass::MissingSite,
                    severity: OverloadDiagnosticSeverity::Error,
                    message_key: OverloadDiagnosticMessageKey::new(
                        "overload.candidate.missing_site",
                    ),
                    recovery: OverloadDiagnosticRecovery::Degraded,
                });
                continue;
            };

            let diagnostic_provenance = candidate_diagnostic_provenance(&input.input);
            let site_status = self
                .sites
                .get(site_id)
                .map(|site| site.status)
                .unwrap_or(OverloadSiteStatus::Deferred);
            let candidate_id = OverloadCandidateId::new(self.candidates.len());
            let status = if site_status == OverloadSiteStatus::Deferred
                || !input.input.declaration_kind.is_supported()
            {
                OverloadCandidateStatus::Deferred
            } else {
                OverloadCandidateStatus::Collected
            };
            self.candidates.push(OverloadCandidate {
                id: candidate_id,
                site: site_id,
                site_key: input.input.site.clone(),
                symbol: input.input.symbol,
                ordinary_root: input.input.ordinary_root,
                declaration_kind: input.input.declaration_kind.clone(),
                parameters: input.input.parameters,
                result: input.input.result,
                origin: input.input.origin,
                template: input.input.template,
                coherence: input.input.coherence,
                provenance: input.input.provenance,
                status,
                diagnostics: Vec::new(),
            });

            if let Some(message_key) =
                unsupported_candidate_message_key(&input.input.declaration_kind)
            {
                let diagnostic = self.insert_diagnostic(OverloadDiagnosticDraft {
                    site: Some(site_id),
                    site_key: Some(input.input.site),
                    candidate: Some(candidate_id),
                    provenance: Some(diagnostic_provenance),
                    class: OverloadDiagnosticClass::UnsupportedCandidateRole,
                    severity: OverloadDiagnosticSeverity::Note,
                    message_key,
                    recovery: OverloadDiagnosticRecovery::Degraded,
                });
                self.attach_candidate_diagnostic(candidate_id, diagnostic);
            }
        }
    }

    fn insert_diagnostic(&mut self, draft: OverloadDiagnosticDraft) -> OverloadDiagnosticId {
        self.diagnostics.insert(draft)
    }

    fn attach_site_diagnostic(
        &mut self,
        site_id: OverloadSiteId,
        diagnostic: OverloadDiagnosticId,
    ) {
        if let Some(site) = self.sites.get_mut(site_id) {
            site.diagnostics.push(diagnostic);
        }
    }

    fn attach_candidate_diagnostic(
        &mut self,
        candidate_id: OverloadCandidateId,
        diagnostic: OverloadDiagnosticId,
    ) {
        if let Some(candidate) = self.candidates.get_mut(candidate_id) {
            candidate.diagnostics.push(diagnostic);
        }
    }
}

struct TemplateExpansionBuilder<'a> {
    collection: &'a OverloadCollectionOutput,
    candidates: OverloadCandidateTable,
    expansions: TemplateExpansionTable,
    diagnostics: OverloadDiagnosticTable,
}

impl<'a> TemplateExpansionBuilder<'a> {
    fn new(collection: &'a OverloadCollectionOutput) -> Self {
        Self {
            collection,
            candidates: OverloadCandidateTable::new(),
            expansions: TemplateExpansionTable::new(),
            diagnostics: OverloadDiagnosticTable::new(),
        }
    }

    fn finish(mut self) -> TemplateExpansionOutput {
        for (_, candidate) in self.collection.candidates().canonical_iter() {
            if let Some(payload) = &candidate.template {
                self.expand_template_candidate(candidate, payload);
            } else {
                self.copy_concrete_candidate(candidate);
            }
        }
        TemplateExpansionOutput {
            candidates: self.candidates,
            expansions: self.expansions,
            diagnostics: self.diagnostics,
        }
    }

    fn expand_template_candidate(
        &mut self,
        candidate: &OverloadCandidate,
        payload: &TemplateCandidatePayload,
    ) {
        let expansion_id = TemplateExpansionId::new(self.expansions.len());
        let evaluation = evaluate_template_candidate(candidate, payload);
        let (status, substitutions, instantiated_candidate, diagnostics) = match evaluation {
            TemplateEvaluation::Instantiated { substitutions } => {
                let concrete = self.instantiate_candidate(candidate, payload);
                (
                    TemplateExpansionStatus::Instantiated,
                    substitutions,
                    Some(concrete),
                    Vec::new(),
                )
            }
            TemplateEvaluation::Rejected { failure } => {
                let diagnostic = self.insert_template_diagnostic(candidate, &failure, false);
                (
                    TemplateExpansionStatus::Rejected(failure),
                    Vec::new(),
                    None,
                    vec![diagnostic],
                )
            }
            TemplateEvaluation::Deferred { failure } => {
                let diagnostic = self.insert_template_diagnostic(candidate, &failure, true);
                (
                    TemplateExpansionStatus::Deferred(failure),
                    Vec::new(),
                    None,
                    vec![diagnostic],
                )
            }
        };

        self.expansions.push(TemplateExpansion {
            id: expansion_id,
            source_candidate: candidate.id,
            site: candidate.site,
            template: payload.template.clone(),
            instantiation_key: payload.instantiation_key.clone(),
            substitutions,
            instantiated_candidate,
            status,
            diagnostics,
        });
    }

    fn instantiate_candidate(
        &mut self,
        candidate: &OverloadCandidate,
        payload: &TemplateCandidatePayload,
    ) -> OverloadCandidateId {
        let id = OverloadCandidateId::new(self.candidates.len());
        let mut concrete = candidate.clone();
        concrete.id = id;
        concrete.origin = CandidateOrigin::TemplateDerived {
            template: payload.template.clone(),
            instantiation: payload.instantiation_key.clone(),
        };
        concrete.template = None;
        concrete.diagnostics = self.remap_candidate_diagnostics(candidate, id);
        self.candidates.push(concrete);
        id
    }

    fn copy_concrete_candidate(&mut self, candidate: &OverloadCandidate) -> OverloadCandidateId {
        let id = OverloadCandidateId::new(self.candidates.len());
        let mut concrete = candidate.clone();
        concrete.id = id;
        concrete.diagnostics = self.remap_candidate_diagnostics(candidate, id);
        self.candidates.push(concrete);
        id
    }

    fn remap_candidate_diagnostics(
        &mut self,
        candidate: &OverloadCandidate,
        output_candidate: OverloadCandidateId,
    ) -> Vec<OverloadDiagnosticId> {
        candidate
            .diagnostics
            .iter()
            .filter_map(|diagnostic| {
                self.collection
                    .diagnostics()
                    .get(*diagnostic)
                    .map(|diagnostic| {
                        self.diagnostics.insert(OverloadDiagnosticDraft {
                            site: diagnostic.site,
                            site_key: diagnostic.site_key.clone(),
                            candidate: Some(output_candidate),
                            provenance: diagnostic.provenance.clone(),
                            class: diagnostic.class,
                            severity: diagnostic.severity,
                            message_key: diagnostic.message_key.clone(),
                            recovery: diagnostic.recovery,
                        })
                    })
            })
            .collect()
    }

    fn insert_template_diagnostic(
        &mut self,
        candidate: &OverloadCandidate,
        failure: &TemplateExpansionFailure,
        deferred: bool,
    ) -> OverloadDiagnosticId {
        self.diagnostics.insert(OverloadDiagnosticDraft {
            site: Some(candidate.site),
            site_key: Some(candidate.site_key.clone()),
            candidate: None,
            provenance: Some(candidate_diagnostic_provenance_from_candidate(candidate)),
            class: OverloadDiagnosticClass::TemplateExpansion,
            severity: if deferred {
                OverloadDiagnosticSeverity::Note
            } else {
                OverloadDiagnosticSeverity::Error
            },
            message_key: OverloadDiagnosticMessageKey::new(format!(
                "overload.template.{}",
                template_failure_name(failure)
            )),
            recovery: if deferred {
                OverloadDiagnosticRecovery::Degraded
            } else {
                OverloadDiagnosticRecovery::Normal
            },
        })
    }
}

enum TemplateEvaluation {
    Instantiated {
        substitutions: Vec<TemplateSubstitution>,
    },
    Rejected {
        failure: TemplateExpansionFailure,
    },
    Deferred {
        failure: TemplateExpansionFailure,
    },
}

fn evaluate_template_candidate(
    candidate: &OverloadCandidate,
    payload: &TemplateCandidatePayload,
) -> TemplateEvaluation {
    if candidate.status != OverloadCandidateStatus::Collected {
        return TemplateEvaluation::Deferred {
            failure: TemplateExpansionFailure::DeferredCandidate,
        };
    }
    if payload.parameters.len() != payload.arguments.len() {
        return TemplateEvaluation::Rejected {
            failure: TemplateExpansionFailure::ArityMismatch,
        };
    }

    let mut seen_parameters = BTreeMap::new();
    for parameter in &payload.parameters {
        if seen_parameters.insert(parameter.clone(), ()).is_some() {
            return TemplateEvaluation::Rejected {
                failure: TemplateExpansionFailure::DuplicateParameter,
            };
        }
    }

    let mut inferences = BTreeMap::new();
    for inference in &payload.inferred_arguments {
        if inferences
            .insert(inference.parameter.clone(), inference)
            .is_some()
        {
            return TemplateEvaluation::Rejected {
                failure: TemplateExpansionFailure::AmbiguousInference,
            };
        }
    }

    let mut substitutions = Vec::new();
    for (parameter, argument) in payload.parameters.iter().zip(&payload.arguments) {
        match argument {
            TemplateArgument::Explicit(value) => {
                substitutions.push(TemplateSubstitution {
                    parameter: parameter.clone(),
                    value: *value,
                    source: TemplateSubstitutionSource::Explicit,
                });
            }
            TemplateArgument::Omitted(omitted) => {
                if omitted != parameter {
                    return TemplateEvaluation::Rejected {
                        failure: TemplateExpansionFailure::ParameterMismatch,
                    };
                }
                let Some(inference) = inferences.get(parameter) else {
                    return TemplateEvaluation::Rejected {
                        failure: TemplateExpansionFailure::MissingInference,
                    };
                };
                substitutions.push(TemplateSubstitution {
                    parameter: parameter.clone(),
                    value: inference.inferred,
                    source: TemplateSubstitutionSource::OmittedInference {
                        evidence_key: inference.evidence_key.clone(),
                    },
                });
            }
            TemplateArgument::SourceQua {
                source,
                target,
                path,
                status,
            } => match status {
                TemplateQuaStatus::AcceptedWidening => {
                    substitutions.push(TemplateSubstitution {
                        parameter: parameter.clone(),
                        value: *target,
                        source: TemplateSubstitutionSource::SourceQua {
                            source: *source,
                            path: path.clone(),
                        },
                    });
                }
                TemplateQuaStatus::RejectedNarrowing => {
                    return TemplateEvaluation::Rejected {
                        failure: TemplateExpansionFailure::RejectedSourceQua,
                    };
                }
                TemplateQuaStatus::DeferredExternalDependency => {
                    return TemplateEvaluation::Deferred {
                        failure: TemplateExpansionFailure::DeferredSourceQua,
                    };
                }
            },
        }
    }

    for constraint in &payload.constraints {
        if !seen_parameters.contains_key(&constraint.parameter) {
            return TemplateEvaluation::Rejected {
                failure: TemplateExpansionFailure::UnknownConstraintParameter,
            };
        }
        match constraint.status {
            TemplateConstraintEvidenceStatus::Accepted if !constraint.facts.is_empty() => {}
            TemplateConstraintEvidenceStatus::Accepted
            | TemplateConstraintEvidenceStatus::Missing => {
                return TemplateEvaluation::Rejected {
                    failure: TemplateExpansionFailure::MissingConstraintEvidence,
                };
            }
            TemplateConstraintEvidenceStatus::DeferredExternalDependency => {
                return TemplateEvaluation::Deferred {
                    failure: TemplateExpansionFailure::DeferredConstraintEvidence,
                };
            }
        }
    }

    TemplateEvaluation::Instantiated { substitutions }
}

fn site_status(input: &OverloadSiteInput) -> OverloadSiteStatus {
    if !input.kind.is_supported() {
        OverloadSiteStatus::Deferred
    } else if matches!(input.recovery, OverloadSiteRecovery::Degraded { .. }) {
        OverloadSiteStatus::Degraded
    } else {
        OverloadSiteStatus::Collected
    }
}

fn unsupported_site_message_key(kind: &OverloadSiteKind) -> Option<OverloadDiagnosticMessageKey> {
    let OverloadSiteKind::Unsupported(role) = kind else {
        return None;
    };
    Some(OverloadDiagnosticMessageKey::new(format!(
        "overload.site.unsupported.{}",
        unsupported_role_name(role)
    )))
}

fn unsupported_candidate_message_key(
    kind: &CandidateDeclarationKind,
) -> Option<OverloadDiagnosticMessageKey> {
    let CandidateDeclarationKind::Unsupported(role) = kind else {
        return None;
    };
    Some(OverloadDiagnosticMessageKey::new(format!(
        "overload.candidate.unsupported.{}",
        unsupported_role_name(role)
    )))
}

fn site_diagnostic_provenance(input: &OverloadSiteInput) -> OverloadDiagnosticProvenance {
    OverloadDiagnosticProvenance::SiteInput {
        owner: input.owner.clone(),
        source_range: input.source_range,
        kind: input.kind.clone(),
        name: input.name.clone(),
        arguments: input.arguments.clone(),
        source_qua: input.source_qua.clone(),
        recovery: input.recovery.clone(),
    }
}

fn candidate_diagnostic_provenance(input: &OverloadCandidateInput) -> OverloadDiagnosticProvenance {
    OverloadDiagnosticProvenance::CandidateInput {
        symbol: input.symbol.clone(),
        ordinary_root: input.ordinary_root.clone(),
        declaration_kind: input.declaration_kind.clone(),
        template: input.template.clone().map(Box::new),
        coherence: input.coherence,
        provenance: input.provenance.clone(),
    }
}

fn candidate_diagnostic_provenance_from_candidate(
    candidate: &OverloadCandidate,
) -> OverloadDiagnosticProvenance {
    OverloadDiagnosticProvenance::CandidateInput {
        symbol: candidate.symbol.clone(),
        ordinary_root: candidate.ordinary_root.clone(),
        declaration_kind: candidate.declaration_kind.clone(),
        template: candidate.template.clone().map(Box::new),
        coherence: candidate.coherence,
        provenance: candidate.provenance.clone(),
    }
}

fn site_input_order_key(input: &OverloadSiteInput, ordinal: usize) -> SiteInputOrderKey {
    (
        range_order_key(input.source_range),
        site_ref_order_key(&input.owner),
        input.kind.clone(),
        input.name.as_str().to_owned(),
        input.key.as_str().to_owned(),
        ordinal,
    )
}

fn site_output_key(site: &OverloadSite) -> SiteOutputOrderKey {
    (
        range_order_key(site.source_range),
        site_ref_order_key(&site.owner),
        site.kind.clone(),
        site.name.as_str().to_owned(),
        site.key.as_str().to_owned(),
        site.id.index(),
    )
}

fn candidate_input_order_key(
    input: &OverloadCandidateInput,
    ordinal: usize,
    site_ids: &BTreeMap<OverloadSiteKey, OverloadSiteId>,
) -> CandidateInputOrderKey {
    let site_order = site_ids
        .get(&input.site)
        .map(|id| id.index())
        .unwrap_or(usize::MAX);
    (
        site_order,
        input.site.as_str().to_owned(),
        input.declaration_kind.clone(),
        input.ordinary_root.clone(),
        input.symbol.clone(),
        template_order_key(input.template.as_ref(), &input.origin),
        provenance_order_key(&input.provenance),
        input.provenance.declaration_order,
        ordinal,
    )
}

fn candidate_output_key(candidate: &OverloadCandidate) -> CandidateOutputOrderKey {
    (
        candidate.site.index(),
        candidate.declaration_kind.clone(),
        candidate.ordinary_root.clone(),
        candidate.symbol.clone(),
        template_order_key(candidate.template.as_ref(), &candidate.origin),
        provenance_order_key(&candidate.provenance),
        candidate.id.index(),
    )
}

fn template_expansion_order_key(expansion: &TemplateExpansion) -> TemplateExpansionOrderKey {
    (
        expansion.source_candidate.index(),
        expansion.instantiation_key.as_str().to_owned(),
        expansion.id.index(),
    )
}

fn diagnostic_output_key(diagnostic: &OverloadDiagnostic) -> DiagnosticOutputOrderKey {
    let (site_missing, site_index, site_key) = match (diagnostic.site, &diagnostic.site_key) {
        (Some(site), key) => (
            0,
            site.index(),
            key.as_ref()
                .map(|key| key.as_str().to_owned())
                .unwrap_or_default(),
        ),
        (None, Some(key)) => (1, usize::MAX, key.as_str().to_owned()),
        (None, None) => (2, usize::MAX, String::new()),
    };
    let (candidate_missing, candidate_index) = match diagnostic.candidate {
        Some(candidate) => (0, candidate.index()),
        None => (1, usize::MAX),
    };
    (
        site_missing,
        site_index,
        site_key,
        candidate_missing,
        candidate_index,
        diagnostic.class,
        diagnostic.message_key.as_str().to_owned(),
        diagnostic.id.index(),
    )
}

fn template_order_key(
    payload: Option<&TemplateCandidatePayload>,
    origin: &CandidateOrigin,
) -> String {
    payload
        .map(|payload| payload.instantiation_key.as_str().to_owned())
        .or_else(|| {
            if let CandidateOrigin::TemplateDerived { instantiation, .. } = origin {
                Some(instantiation.as_str().to_owned())
            } else {
                None
            }
        })
        .unwrap_or_default()
}

fn provenance_order_key(provenance: &CandidateProvenance) -> CandidateProvenanceOrderKey {
    match &provenance.scope {
        CandidateScope::Local => (
            0,
            String::new(),
            String::new(),
            optional_range_order_key(provenance.source_range),
            provenance.declaration_order,
            provenance.stable_key.as_str().to_owned(),
        ),
        CandidateScope::Imported { module, import_key } => (
            1,
            module_order_key(module),
            import_key.as_str().to_owned(),
            optional_range_order_key(provenance.source_range),
            provenance.declaration_order,
            provenance.stable_key.as_str().to_owned(),
        ),
        CandidateScope::DependencySummary {
            module,
            summary_key,
        } => (
            2,
            module_order_key(module),
            summary_key.as_str().to_owned(),
            optional_range_order_key(provenance.source_range),
            provenance.declaration_order,
            provenance.stable_key.as_str().to_owned(),
        ),
    }
}

fn optional_range_order_key(range: Option<SourceRange>) -> OptionalRangeOrderKey {
    range.map_or_else(
        || (1, String::new(), usize::MAX, usize::MAX),
        |range| {
            let (source, start, end) = range_order_key(range);
            (0, source, start, end)
        },
    )
}

fn range_order_key(range: SourceRange) -> RangeOrderKey {
    (source_order_key(range), range.start, range.end)
}

fn source_order_key(range: SourceRange) -> String {
    format!("{:?}", range.source_id)
}

fn site_ref_order_key(site: &TypedSiteRef) -> SiteRefOrderKey {
    match site {
        TypedSiteRef::Node(node) => (node.index(), 0, String::new()),
        TypedSiteRef::Role { node, role } => (node.index(), 1, role.as_str().to_owned()),
    }
}

fn site_ref_key(site: &TypedSiteRef) -> String {
    match site {
        TypedSiteRef::Node(node) => format!("node#{}", node.index()),
        TypedSiteRef::Role { node, role } => format!("node#{}:{}", node.index(), role.as_str()),
    }
}

fn module_order_key(module: &ModuleId) -> String {
    format!("{}::{}", module.package().as_str(), module.path().as_str())
}

fn write_sites(output: &mut String, sites: &OverloadSiteTable) {
    output.push_str("sites:\n");
    if sites.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    for (id, site) in sites.canonical_iter() {
        let _ = write!(
            output,
            "  site#{} key=\"{}\" kind={} status={} owner={} range=",
            id.index(),
            escaped_display(site.key.as_str()),
            site_kind_name(&site.kind),
            site_status_name(site.status),
            site_ref_key(&site.owner)
        );
        write_range(output, site.source_range);
        let _ = write!(
            output,
            " name=\"{}\" args=",
            escaped_display(site.name.as_str())
        );
        write_site_refs(output, &site.arguments);
        output.push_str(" expected=");
        write_optional_type(output, site.expected);
        output.push_str(" source_qua=");
        write_qua_views(output, &site.source_qua);
        output.push_str(" recovery=");
        write_site_recovery(output, &site.recovery);
        output.push_str(" diagnostics=");
        write_diagnostic_ids(output, &site.diagnostics);
        output.push('\n');
    }
}

fn write_candidates(output: &mut String, candidates: &OverloadCandidateTable) {
    output.push_str("candidates:\n");
    if candidates.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    for (id, candidate) in candidates.canonical_iter() {
        let _ = write!(
            output,
            "  candidate#{} site=site#{} site_key=\"{}\" status={} declaration={} symbol=",
            id.index(),
            candidate.site.index(),
            escaped_display(candidate.site_key.as_str()),
            candidate_status_name(candidate.status),
            candidate_declaration_kind_name(&candidate.declaration_kind)
        );
        write_symbol_id(output, &candidate.symbol);
        output.push_str(" root=");
        write_symbol_id(output, &candidate.ordinary_root);
        output.push_str(" parameters=");
        write_type_ids(output, &candidate.parameters);
        output.push_str(" result=");
        write_optional_type(output, candidate.result);
        output.push_str(" origin=");
        write_origin(output, &candidate.origin);
        output.push_str(" template=");
        write_template_payload(output, candidate.template.as_ref());
        output.push_str(" coherence=");
        write_optional_coherence(output, candidate.coherence);
        output.push_str(" provenance=");
        write_candidate_provenance(output, &candidate.provenance);
        output.push_str(" diagnostics=");
        write_diagnostic_ids(output, &candidate.diagnostics);
        output.push('\n');
    }
}

fn write_template_expansions(output: &mut String, expansions: &TemplateExpansionTable) {
    output.push_str("template_expansions:\n");
    if expansions.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    for (id, expansion) in expansions.canonical_iter() {
        let _ = write!(
            output,
            "  expansion#{} source=candidate#{} site=site#{} template=",
            id.index(),
            expansion.source_candidate.index(),
            expansion.site.index()
        );
        write_symbol_id(output, &expansion.template);
        let _ = write!(
            output,
            " instantiation=\"{}\" status=",
            escaped_display(expansion.instantiation_key.as_str())
        );
        write_template_expansion_status(output, &expansion.status);
        output.push_str(" substitutions=");
        write_template_substitutions(output, &expansion.substitutions);
        output.push_str(" instantiated=");
        write_optional_candidate_id(output, expansion.instantiated_candidate);
        output.push_str(" diagnostics=");
        write_diagnostic_ids(output, &expansion.diagnostics);
        output.push('\n');
    }
}

fn write_diagnostics(output: &mut String, diagnostics: &OverloadDiagnosticTable) {
    output.push_str("diagnostics:\n");
    if diagnostics.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    for (id, diagnostic) in diagnostics.canonical_iter() {
        let _ = write!(output, "  diagnostic#{} site=", id.index());
        write_optional_site_id(output, diagnostic.site);
        output.push_str(" site_key=");
        write_optional_site_key(output, diagnostic.site_key.as_ref());
        output.push_str(" candidate=");
        write_optional_candidate_id(output, diagnostic.candidate);
        output.push_str(" provenance=");
        write_diagnostic_provenance(output, diagnostic.provenance.as_ref());
        let _ = writeln!(
            output,
            " class={} severity={} message_key=\"{}\" recovery={}",
            diagnostic_class_name(diagnostic.class),
            diagnostic_severity_name(diagnostic.severity),
            escaped_display(diagnostic.message_key.as_str()),
            diagnostic_recovery_name(diagnostic.recovery)
        );
    }
}

fn write_diagnostic_provenance(
    output: &mut String,
    provenance: Option<&OverloadDiagnosticProvenance>,
) {
    let Some(provenance) = provenance else {
        output.push_str("<none>");
        return;
    };
    match provenance {
        OverloadDiagnosticProvenance::SiteInput {
            owner,
            source_range,
            kind,
            name,
            arguments,
            source_qua,
            recovery,
        } => {
            let _ = write!(output, "site_input(owner={}, range=", site_ref_key(owner));
            write_range(output, *source_range);
            let _ = write!(
                output,
                ", kind={}, name=\"{}\", args=",
                site_kind_name(kind),
                escaped_display(name.as_str())
            );
            write_site_refs(output, arguments);
            output.push_str(", source_qua=");
            write_qua_views(output, source_qua);
            output.push_str(", recovery=");
            write_site_recovery(output, recovery);
            output.push(')');
        }
        OverloadDiagnosticProvenance::CandidateInput {
            symbol,
            ordinary_root,
            declaration_kind,
            template,
            coherence,
            provenance,
        } => {
            output.push_str("candidate_input(symbol=");
            write_symbol_id(output, symbol);
            output.push_str(", root=");
            write_symbol_id(output, ordinary_root);
            let _ = write!(
                output,
                ", declaration={}, template=",
                candidate_declaration_kind_name(declaration_kind)
            );
            write_template_payload(output, template.as_deref());
            output.push_str(", coherence=");
            write_optional_coherence(output, *coherence);
            output.push_str(", provenance=");
            write_candidate_provenance(output, provenance);
            output.push(')');
        }
    }
}

fn write_qua_views(output: &mut String, views: &[SourceQuaView]) {
    output.push('[');
    for (index, view) in views.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        output.push('{');
        output.push_str("argument=");
        output.push_str(&site_ref_key(&view.argument));
        output.push_str(", target=");
        write_type_id(output, view.target);
        output.push_str(", range=");
        write_range(output, view.source_range);
        output.push_str(", path=");
        match &view.path {
            Some(path) => {
                let _ = write!(output, "\"{}\"", escaped_display(path.as_str()));
            }
            None => output.push_str("<none>"),
        }
        output.push_str(", facts=");
        write_fact_ids(output, &view.evidence_facts);
        output.push('}');
    }
    output.push(']');
}

fn write_template_expansion_status(output: &mut String, status: &TemplateExpansionStatus) {
    match status {
        TemplateExpansionStatus::Instantiated => output.push_str("instantiated"),
        TemplateExpansionStatus::Rejected(failure) => {
            let _ = write!(output, "rejected({})", template_failure_name(failure));
        }
        TemplateExpansionStatus::Deferred(failure) => {
            let _ = write!(output, "deferred({})", template_failure_name(failure));
        }
    }
}

fn write_template_substitutions(output: &mut String, substitutions: &[TemplateSubstitution]) {
    output.push('[');
    for (index, substitution) in substitutions.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        let _ = write!(
            output,
            "{{parameter=\"{}\", value=",
            escaped_display(substitution.parameter.as_str())
        );
        write_type_id(output, substitution.value);
        output.push_str(", source=");
        write_template_substitution_source(output, &substitution.source);
        output.push('}');
    }
    output.push(']');
}

fn write_template_substitution_source(output: &mut String, source: &TemplateSubstitutionSource) {
    match source {
        TemplateSubstitutionSource::Explicit => output.push_str("explicit"),
        TemplateSubstitutionSource::OmittedInference { evidence_key } => {
            let _ = write!(
                output,
                "omitted_inference(\"{}\")",
                escaped_display(evidence_key.as_str())
            );
        }
        TemplateSubstitutionSource::SourceQua { source, path } => {
            output.push_str("source_qua(source=");
            write_type_id(output, *source);
            let _ = write!(output, ", path=\"{}\")", escaped_display(path.as_str()));
        }
    }
}

fn write_template_payload(output: &mut String, payload: Option<&TemplateCandidatePayload>) {
    let Some(payload) = payload else {
        output.push_str("<none>");
        return;
    };
    output.push('{');
    output.push_str("template=");
    write_symbol_id(output, &payload.template);
    let _ = write!(
        output,
        ", instantiation=\"{}\", parameters=",
        escaped_display(payload.instantiation_key.as_str())
    );
    write_template_parameters(output, &payload.parameters);
    output.push_str(", arguments=");
    write_template_arguments(output, &payload.arguments);
    output.push_str(", inferred=");
    write_template_inferences(output, &payload.inferred_arguments);
    output.push_str(", constraints=");
    write_template_constraints(output, &payload.constraints);
    output.push('}');
}

fn write_template_parameters(output: &mut String, parameters: &[TemplateParameterKey]) {
    output.push('[');
    for (index, parameter) in parameters.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        let _ = write!(output, "\"{}\"", escaped_display(parameter.as_str()));
    }
    output.push(']');
}

fn write_template_arguments(output: &mut String, arguments: &[TemplateArgument]) {
    output.push('[');
    for (index, argument) in arguments.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        match argument {
            TemplateArgument::Explicit(id) => {
                output.push_str("explicit=");
                write_type_id(output, *id);
            }
            TemplateArgument::Omitted(parameter) => {
                let _ = write!(
                    output,
                    "omitted=\"{}\"",
                    escaped_display(parameter.as_str())
                );
            }
            TemplateArgument::SourceQua {
                source,
                target,
                path,
                status,
            } => {
                output.push_str("source_qua(source=");
                write_type_id(output, *source);
                output.push_str(", target=");
                write_type_id(output, *target);
                let _ = write!(
                    output,
                    ", path=\"{}\", status={})",
                    escaped_display(path.as_str()),
                    template_qua_status_name(*status)
                );
            }
        }
    }
    output.push(']');
}

fn write_template_inferences(output: &mut String, inferences: &[TemplateArgumentInference]) {
    output.push('[');
    for (index, inference) in inferences.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        let _ = write!(
            output,
            "{{parameter=\"{}\", inferred=",
            escaped_display(inference.parameter.as_str())
        );
        write_type_id(output, inference.inferred);
        let _ = write!(
            output,
            ", evidence=\"{}\"}}",
            escaped_display(inference.evidence_key.as_str())
        );
    }
    output.push(']');
}

fn write_template_constraints(output: &mut String, constraints: &[TemplateConstraintEvidence]) {
    output.push('[');
    for (index, constraint) in constraints.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        let _ = write!(
            output,
            "{{parameter=\"{}\", evidence=\"{}\", status={}, facts=",
            escaped_display(constraint.parameter.as_str()),
            escaped_display(constraint.evidence_key.as_str()),
            template_constraint_status_name(constraint.status)
        );
        write_fact_ids(output, &constraint.facts);
        output.push('}');
    }
    output.push(']');
}

fn write_candidate_provenance(output: &mut String, provenance: &CandidateProvenance) {
    output.push('{');
    let _ = write!(
        output,
        "key=\"{}\", scope=",
        escaped_display(provenance.stable_key.as_str())
    );
    write_candidate_scope(output, &provenance.scope);
    output.push_str(", range=");
    match provenance.source_range {
        Some(range) => write_range(output, range),
        None => output.push_str("<none>"),
    }
    let _ = write!(
        output,
        ", declaration_order={}",
        provenance.declaration_order
    );
    output.push('}');
}

fn write_candidate_scope(output: &mut String, scope: &CandidateScope) {
    match scope {
        CandidateScope::Local => output.push_str("local"),
        CandidateScope::Imported { module, import_key } => {
            output.push_str("imported(module=");
            write_module_id(output, module);
            let _ = write!(
                output,
                ", import=\"{}\")",
                escaped_display(import_key.as_str())
            );
        }
        CandidateScope::DependencySummary {
            module,
            summary_key,
        } => {
            output.push_str("dependency_summary(module=");
            write_module_id(output, module);
            let _ = write!(
                output,
                ", summary=\"{}\")",
                escaped_display(summary_key.as_str())
            );
        }
    }
}

fn write_origin(output: &mut String, origin: &CandidateOrigin) {
    match origin {
        CandidateOrigin::Ordinary => output.push_str("ordinary"),
        CandidateOrigin::Redefinition { refined } => {
            output.push_str("redefinition(refined=");
            write_symbol_id(output, refined);
            output.push(')');
        }
        CandidateOrigin::TemplateDerived {
            template,
            instantiation,
        } => {
            output.push_str("template_derived(template=");
            write_symbol_id(output, template);
            let _ = write!(
                output,
                ", instantiation=\"{}\")",
                escaped_display(instantiation.as_str())
            );
        }
        CandidateOrigin::Recovery(key) => {
            let _ = write!(output, "recovery(\"{}\")", escaped_display(key.as_str()));
        }
    }
}

fn write_site_recovery(output: &mut String, recovery: &OverloadSiteRecovery) {
    match recovery {
        OverloadSiteRecovery::Normal => output.push_str("normal"),
        OverloadSiteRecovery::Degraded { message_key } => {
            let _ = write!(
                output,
                "degraded(\"{}\")",
                escaped_display(message_key.as_str())
            );
        }
    }
}

fn write_optional_coherence(output: &mut String, coherence: Option<CoherenceStatus>) {
    match coherence {
        Some(status) => output.push_str(coherence_status_name(status)),
        None => output.push_str("<none>"),
    }
}

fn write_site_refs(output: &mut String, sites: &[TypedSiteRef]) {
    output.push('[');
    for (index, site) in sites.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        output.push_str(&site_ref_key(site));
    }
    output.push(']');
}

fn write_type_ids(output: &mut String, ids: &[NormalizedTypeId]) {
    output.push('[');
    for (index, id) in ids.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        write_type_id(output, *id);
    }
    output.push(']');
}

fn write_fact_ids(output: &mut String, ids: &[TypeFactId]) {
    output.push('[');
    for (index, id) in ids.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        let _ = write!(output, "fact#{}", id.index());
    }
    output.push(']');
}

fn write_diagnostic_ids(output: &mut String, ids: &[OverloadDiagnosticId]) {
    output.push('[');
    for (index, id) in ids.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        let _ = write!(output, "diagnostic#{}", id.index());
    }
    output.push(']');
}

fn write_optional_type(output: &mut String, id: Option<NormalizedTypeId>) {
    match id {
        Some(id) => write_type_id(output, id),
        None => output.push_str("<none>"),
    }
}

fn write_type_id(output: &mut String, id: NormalizedTypeId) {
    let _ = write!(output, "normalized_type#{}", id.index());
}

fn write_optional_site_id(output: &mut String, id: Option<OverloadSiteId>) {
    match id {
        Some(id) => {
            let _ = write!(output, "site#{}", id.index());
        }
        None => output.push_str("<none>"),
    }
}

fn write_optional_site_key(output: &mut String, key: Option<&OverloadSiteKey>) {
    match key {
        Some(key) => {
            let _ = write!(output, "\"{}\"", escaped_display(key.as_str()));
        }
        None => output.push_str("<none>"),
    }
}

fn write_optional_candidate_id(output: &mut String, id: Option<OverloadCandidateId>) {
    match id {
        Some(id) => {
            let _ = write!(output, "candidate#{}", id.index());
        }
        None => output.push_str("<none>"),
    }
}

fn write_range(output: &mut String, range: SourceRange) {
    let _ = write!(
        output,
        "source=\"{}\":{}..{}",
        escaped_display(&source_order_key(range)),
        range.start,
        range.end
    );
}

fn write_module_id(output: &mut String, module: &ModuleId) {
    write_escaped(output, module.package().as_str());
    output.push_str("::");
    write_escaped(output, module.path().as_str());
}

fn write_symbol_id(output: &mut String, symbol: &SymbolId) {
    output.push_str("{fqn=\"");
    write_escaped(output, symbol.fqn().as_str());
    output.push_str("\" module=");
    write_module_id(output, symbol.module());
    output.push_str(" local=\"");
    write_escaped(output, symbol.local().as_str());
    output.push_str("\"}");
}

fn site_kind_name(kind: &OverloadSiteKind) -> &'static str {
    match kind {
        OverloadSiteKind::FunctorApplication => "functor_application",
        OverloadSiteKind::PredicateApplication => "predicate_application",
        OverloadSiteKind::AttributeApplication => "attribute_application",
        OverloadSiteKind::ModeApplication => "mode_application",
        OverloadSiteKind::SelectorApplication => "selector_application",
        OverloadSiteKind::StructureFieldApplication => "structure_field_application",
        OverloadSiteKind::TemplateName => "template_name",
        OverloadSiteKind::Unsupported(_) => "unsupported",
    }
}

fn unsupported_role_name(role: &UnsupportedOverloadRole) -> &'static str {
    match role {
        UnsupportedOverloadRole::SchemeApplication => "scheme_application",
        UnsupportedOverloadRole::TheoremApplication => "theorem_application",
        UnsupportedOverloadRole::AlgorithmTemplate => "algorithm_template",
        UnsupportedOverloadRole::Unknown => "unknown",
    }
}

fn candidate_declaration_kind_name(kind: &CandidateDeclarationKind) -> &'static str {
    match kind {
        CandidateDeclarationKind::Functor => "functor",
        CandidateDeclarationKind::Predicate => "predicate",
        CandidateDeclarationKind::Attribute => "attribute",
        CandidateDeclarationKind::Mode => "mode",
        CandidateDeclarationKind::Selector => "selector",
        CandidateDeclarationKind::StructureField => "structure_field",
        CandidateDeclarationKind::Template => "template",
        CandidateDeclarationKind::Redefinition => "redefinition",
        CandidateDeclarationKind::Unsupported(_) => "unsupported",
    }
}

fn site_status_name(status: OverloadSiteStatus) -> &'static str {
    match status {
        OverloadSiteStatus::Collected => "collected",
        OverloadSiteStatus::Degraded => "degraded",
        OverloadSiteStatus::Deferred => "deferred",
    }
}

fn candidate_status_name(status: OverloadCandidateStatus) -> &'static str {
    match status {
        OverloadCandidateStatus::Collected => "collected",
        OverloadCandidateStatus::Deferred => "deferred",
    }
}

fn coherence_status_name(status: CoherenceStatus) -> &'static str {
    match status {
        CoherenceStatus::Accepted => "accepted",
        CoherenceStatus::Pending => "pending",
        CoherenceStatus::Rejected => "rejected",
    }
}

fn diagnostic_class_name(class: OverloadDiagnosticClass) -> &'static str {
    match class {
        OverloadDiagnosticClass::DuplicateSiteKey => "duplicate_site_key",
        OverloadDiagnosticClass::MissingSite => "missing_site",
        OverloadDiagnosticClass::UnsupportedSiteRole => "unsupported_site_role",
        OverloadDiagnosticClass::UnsupportedCandidateRole => "unsupported_candidate_role",
        OverloadDiagnosticClass::TemplateExpansion => "template_expansion",
        OverloadDiagnosticClass::Recovery => "recovery",
        OverloadDiagnosticClass::ExternalDependencyGap => "external_dependency_gap",
    }
}

fn template_failure_name(failure: &TemplateExpansionFailure) -> &'static str {
    match failure {
        TemplateExpansionFailure::ArityMismatch => "arity_mismatch",
        TemplateExpansionFailure::DuplicateParameter => "duplicate_parameter",
        TemplateExpansionFailure::ParameterMismatch => "parameter_mismatch",
        TemplateExpansionFailure::MissingInference => "missing_inference",
        TemplateExpansionFailure::AmbiguousInference => "ambiguous_inference",
        TemplateExpansionFailure::UnknownConstraintParameter => "unknown_constraint_parameter",
        TemplateExpansionFailure::MissingConstraintEvidence => "missing_constraint_evidence",
        TemplateExpansionFailure::DeferredConstraintEvidence => "deferred_constraint_evidence",
        TemplateExpansionFailure::RejectedSourceQua => "rejected_source_qua",
        TemplateExpansionFailure::DeferredSourceQua => "deferred_source_qua",
        TemplateExpansionFailure::DeferredCandidate => "deferred_candidate",
    }
}

fn template_qua_status_name(status: TemplateQuaStatus) -> &'static str {
    match status {
        TemplateQuaStatus::AcceptedWidening => "accepted_widening",
        TemplateQuaStatus::RejectedNarrowing => "rejected_narrowing",
        TemplateQuaStatus::DeferredExternalDependency => "deferred_external_dependency",
    }
}

fn template_constraint_status_name(status: TemplateConstraintEvidenceStatus) -> &'static str {
    match status {
        TemplateConstraintEvidenceStatus::Accepted => "accepted",
        TemplateConstraintEvidenceStatus::Missing => "missing",
        TemplateConstraintEvidenceStatus::DeferredExternalDependency => {
            "deferred_external_dependency"
        }
    }
}

fn diagnostic_severity_name(severity: OverloadDiagnosticSeverity) -> &'static str {
    match severity {
        OverloadDiagnosticSeverity::Error => "error",
        OverloadDiagnosticSeverity::Warning => "warning",
        OverloadDiagnosticSeverity::Note => "note",
    }
}

fn diagnostic_recovery_name(recovery: OverloadDiagnosticRecovery) -> &'static str {
    match recovery {
        OverloadDiagnosticRecovery::Normal => "normal",
        OverloadDiagnosticRecovery::Degraded => "degraded",
    }
}

fn escaped_display(value: &str) -> String {
    let mut escaped = String::new();
    write_escaped(&mut escaped, value);
    escaped
}

fn write_escaped(output: &mut String, value: &str) {
    for character in value.chars() {
        match character {
            '\\' => output.push_str("\\\\"),
            '"' => output.push_str("\\\""),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            character if character.is_control() => {
                let _ = write!(output, "\\u{{{:x}}}", character as u32);
            }
            character => output.push(character),
        }
    }
}

impl fmt::Display for OverloadSiteKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::typed_ast::{NormalizedTypeId, TypeRole, TypedNodeId, TypedSiteRef};
    use mizar_resolve::resolved_ast::{FullyQualifiedName, LocalSymbolId, ModuleId};
    use mizar_session::{
        BuildSnapshotId, InMemorySessionIdAllocator, ModulePath, PackageId, SessionIdAllocator,
        SourceId,
    };

    #[test]
    fn collects_supported_site_kinds_with_source_provenance() {
        let source_id = source_id(1);
        let sites = vec![
            site(
                "functor",
                OverloadSiteKind::FunctorApplication,
                source_id,
                10,
            ),
            site(
                "predicate",
                OverloadSiteKind::PredicateApplication,
                source_id,
                20,
            ),
            site(
                "attribute",
                OverloadSiteKind::AttributeApplication,
                source_id,
                30,
            ),
            site("mode", OverloadSiteKind::ModeApplication, source_id, 40),
            site(
                "selector",
                OverloadSiteKind::SelectorApplication,
                source_id,
                50,
            ),
            site(
                "structure",
                OverloadSiteKind::StructureFieldApplication,
                source_id,
                60,
            ),
            site("template", OverloadSiteKind::TemplateName, source_id, 70),
        ];
        let candidates = vec![
            candidate(
                "template",
                "template-root",
                "template-ref",
                CandidateScope::Local,
                0,
            ),
            candidate(
                "functor",
                "functor-root",
                "functor-ref",
                CandidateScope::Local,
                1,
            ),
        ];

        let output = OverloadCollectionOutput::collect(sites, candidates);

        assert_eq!(output.sites().len(), 7);
        assert_eq!(output.candidates().len(), 2);
        assert!(output.diagnostics().is_empty());
        assert_eq!(
            output
                .sites()
                .iter()
                .map(|(_, site)| site.key.as_str())
                .collect::<Vec<_>>(),
            [
                "functor",
                "predicate",
                "attribute",
                "mode",
                "selector",
                "structure",
                "template"
            ]
        );
        let debug = output.debug_text();
        assert!(debug.contains("key=\"functor\" kind=functor_application"));
        assert!(debug.contains("key=\"structure\" kind=structure_field_application"));
        assert!(debug.contains("range=source=\"SourceId"));
        assert!(debug.contains("provenance={key=\"template-ref-provenance\""));
    }

    #[test]
    fn preserves_already_filtered_candidate_sets_and_provenance() {
        let source_id = source_id(2);
        let imported_module = module("mathlib", "hidden/import");
        let output = OverloadCollectionOutput::collect(
            vec![site(
                "call",
                OverloadSiteKind::FunctorApplication,
                source_id,
                10,
            )],
            vec![
                candidate("call", "root-a", "local-a", CandidateScope::Local, 0),
                candidate(
                    "call",
                    "root-b",
                    "imported-b",
                    CandidateScope::Imported {
                        module: imported_module,
                        import_key: CandidateProvenanceKey::new("visible-import"),
                    },
                    1,
                ),
            ],
        );

        assert_eq!(output.candidates().len(), 2);
        let scopes = output
            .candidates()
            .iter()
            .map(|(_, candidate)| &candidate.provenance.scope)
            .collect::<Vec<_>>();
        assert!(matches!(scopes[0], CandidateScope::Local));
        assert!(matches!(scopes[1], CandidateScope::Imported { .. }));
        let (_, local) = output.candidates().iter().next().expect("local candidate");
        assert_eq!(local.provenance.stable_key.as_str(), "local-a-provenance");
        assert_range(local.provenance.source_range.expect("local range"), 0, 1);
        assert_eq!(local.provenance.declaration_order, 0);
        let (_, imported) = output
            .candidates()
            .iter()
            .nth(1)
            .expect("imported candidate");
        assert_eq!(
            imported.provenance.stable_key.as_str(),
            "imported-b-provenance"
        );
        assert_range(
            imported.provenance.source_range.expect("import range"),
            1,
            2,
        );
        assert_eq!(imported.provenance.declaration_order, 1);
        let debug = output.debug_text();
        assert!(debug.contains("scope=local"));
        assert!(debug.contains("scope=imported(module=mathlib::hidden/import"));
    }

    #[test]
    fn candidate_order_is_deterministic_by_site_and_declaration_keys() {
        let source_id = source_id(3);
        let sites = vec![
            site("later", OverloadSiteKind::FunctorApplication, source_id, 50),
            site(
                "earlier",
                OverloadSiteKind::FunctorApplication,
                source_id,
                10,
            ),
        ];
        let candidates = vec![
            candidate("later", "root-z", "symbol-z", CandidateScope::Local, 9),
            candidate("earlier", "root-b", "symbol-b", CandidateScope::Local, 2),
            candidate("earlier", "root-a", "symbol-c", CandidateScope::Local, 3),
            candidate("earlier", "root-a", "symbol-a", CandidateScope::Local, 1),
        ];

        let output = OverloadCollectionOutput::collect(sites.clone(), candidates.clone());
        let permuted_output = OverloadCollectionOutput::collect(
            sites.into_iter().rev().collect::<Vec<_>>(),
            candidates.into_iter().rev().collect::<Vec<_>>(),
        );

        assert_eq!(
            output
                .candidates()
                .iter()
                .map(|(_, candidate)| candidate.symbol.local().as_str())
                .collect::<Vec<_>>(),
            ["symbol-a", "symbol-c", "symbol-b", "symbol-z"]
        );
        assert_eq!(output.debug_text(), permuted_output.debug_text());
    }

    #[test]
    fn candidate_order_uses_provenance_and_declaration_order_tie_breakers() {
        let source_id = source_id(31);
        let mut later = candidate("call", "root-a", "symbol-a", CandidateScope::Local, 30);
        later.provenance.stable_key = CandidateProvenanceKey::new("later");
        later.provenance.source_range = Some(range(source_id, 80, 81));
        let mut earlier = candidate("call", "root-a", "symbol-a", CandidateScope::Local, 10);
        earlier.provenance.stable_key = CandidateProvenanceKey::new("earlier");
        earlier.provenance.source_range = Some(range(source_id, 40, 41));

        let output = OverloadCollectionOutput::collect(
            vec![site(
                "call",
                OverloadSiteKind::FunctorApplication,
                source_id,
                10,
            )],
            vec![later, earlier],
        );

        assert_eq!(
            output
                .candidates()
                .iter()
                .map(|(_, candidate)| candidate.provenance.stable_key.as_str())
                .collect::<Vec<_>>(),
            ["earlier", "later"]
        );
    }

    #[test]
    fn candidate_template_payload_and_coherence_are_retained() {
        let source_id = source_id(32);
        let template = symbol_id("template-source");
        let mut input = candidate(
            "call",
            "root",
            "template-candidate",
            CandidateScope::Local,
            0,
        );
        input.origin = CandidateOrigin::TemplateDerived {
            template: template.clone(),
            instantiation: TemplateInstantiationKey::new("T=mode"),
        };
        input.template = Some(TemplateCandidatePayload {
            template,
            instantiation_key: TemplateInstantiationKey::new("T=mode"),
            parameters: vec![
                TemplateParameterKey::new("T"),
                TemplateParameterKey::new("U"),
            ],
            arguments: vec![
                TemplateArgument::Explicit(NormalizedTypeId::new(7)),
                TemplateArgument::Omitted(TemplateParameterKey::new("U")),
            ],
            inferred_arguments: vec![TemplateArgumentInference {
                parameter: TemplateParameterKey::new("U"),
                inferred: NormalizedTypeId::new(10),
                evidence_key: CandidateProvenanceKey::new("inferred-U"),
            }],
            constraints: vec![TemplateConstraintEvidence {
                parameter: TemplateParameterKey::new("U"),
                evidence_key: CandidateProvenanceKey::new("constraint-visible"),
                facts: vec![TypeFactId::new(8)],
                status: TemplateConstraintEvidenceStatus::Accepted,
            }],
        });
        input.coherence = Some(CoherenceStatus::Accepted);

        let output = OverloadCollectionOutput::collect(
            vec![site("call", OverloadSiteKind::TemplateName, source_id, 10)],
            vec![input],
        );

        let (_, candidate) = output.candidates().iter().next().expect("candidate");
        assert_eq!(candidate.coherence, Some(CoherenceStatus::Accepted));
        let payload = candidate.template.as_ref().expect("template payload");
        assert_eq!(payload.instantiation_key.as_str(), "T=mode");
        assert_eq!(
            payload.arguments,
            [
                TemplateArgument::Explicit(NormalizedTypeId::new(7)),
                TemplateArgument::Omitted(TemplateParameterKey::new("U"))
            ]
        );
        assert_eq!(payload.constraints[0].parameter.as_str(), "U");
        assert_eq!(
            payload.constraints[0].evidence_key.as_str(),
            "constraint-visible"
        );
        assert_eq!(payload.constraints[0].facts, [TypeFactId::new(8)]);
        let debug = output.debug_text();
        assert!(debug.contains("coherence=accepted"));
        assert!(debug.contains("instantiation=\"T=mode\""));
    }

    #[test]
    fn explicit_template_candidates_expand_into_concrete_candidates_deterministically() {
        let source_id = source_id(33);
        let sites = vec![site(
            "call",
            OverloadSiteKind::FunctorApplication,
            source_id,
            10,
        )];
        let plain = candidate("call", "root", "same", CandidateScope::Local, 0);
        let mut templated = candidate("call", "root", "same", CandidateScope::Local, 1);
        templated.parameters = vec![NormalizedTypeId::new(7), NormalizedTypeId::new(8)];
        templated.result = Some(NormalizedTypeId::new(9));
        templated.template = Some(template_payload(
            "T=explicit",
            vec![TemplateParameterKey::new("T")],
            vec![TemplateArgument::Explicit(NormalizedTypeId::new(7))],
        ));
        let collection = OverloadCollectionOutput::collect(sites.clone(), vec![templated, plain]);
        let mut reversed_candidates = collection
            .candidates()
            .iter()
            .map(|(_, candidate)| OverloadCandidateInput {
                site: candidate.site_key.clone(),
                symbol: candidate.symbol.clone(),
                ordinary_root: candidate.ordinary_root.clone(),
                declaration_kind: candidate.declaration_kind.clone(),
                parameters: candidate.parameters.clone(),
                result: candidate.result,
                origin: candidate.origin.clone(),
                template: candidate.template.clone(),
                coherence: candidate.coherence,
                provenance: candidate.provenance.clone(),
            })
            .collect::<Vec<_>>();
        reversed_candidates.reverse();
        let permuted_collection = OverloadCollectionOutput::collect(sites, reversed_candidates);

        let output = TemplateExpansionOutput::expand(&collection);
        let permuted_output = TemplateExpansionOutput::expand(&permuted_collection);

        assert_eq!(output.debug_text(), permuted_output.debug_text());
        assert_eq!(output.expansions().len(), 1);
        let (_, expansion) = output.expansions().iter().next().expect("expansion");
        assert_eq!(expansion.status, TemplateExpansionStatus::Instantiated);
        assert_eq!(
            expansion.substitutions,
            [TemplateSubstitution {
                parameter: TemplateParameterKey::new("T"),
                value: NormalizedTypeId::new(7),
                source: TemplateSubstitutionSource::Explicit
            }]
        );
        let candidate_origins = output
            .candidates()
            .canonical_iter()
            .map(|(_, candidate)| &candidate.origin)
            .collect::<Vec<_>>();
        assert!(matches!(candidate_origins[0], CandidateOrigin::Ordinary));
        assert!(matches!(
            candidate_origins[1],
            CandidateOrigin::TemplateDerived { .. }
        ));
        let plain_candidate = output
            .candidates()
            .canonical_iter()
            .find_map(|(_, candidate)| {
                matches!(candidate.origin, CandidateOrigin::Ordinary).then_some(candidate)
            })
            .expect("plain candidate");
        assert_eq!(
            plain_candidate.parameters,
            [NormalizedTypeId::new(1), NormalizedTypeId::new(2)]
        );
        assert_eq!(plain_candidate.result, Some(NormalizedTypeId::new(3)));
        assert_eq!(plain_candidate.status, OverloadCandidateStatus::Collected);
        let expanded = output
            .candidates()
            .get(
                expansion
                    .instantiated_candidate
                    .expect("instantiated candidate"),
            )
            .expect("expanded candidate");
        assert!(expanded.template.is_none());
        assert_eq!(
            expanded.parameters,
            [NormalizedTypeId::new(7), NormalizedTypeId::new(8)]
        );
        assert_eq!(expanded.result, Some(NormalizedTypeId::new(9)));
        assert_eq!(expanded.status, OverloadCandidateStatus::Collected);
    }

    #[test]
    fn omitted_template_arguments_require_explicit_inference_payload() {
        let source_id = source_id(34);
        let mut inferred = candidate("call", "root", "inferred", CandidateScope::Local, 0);
        let mut inferred_payload = template_payload(
            "U=inferred",
            vec![TemplateParameterKey::new("U")],
            vec![TemplateArgument::Omitted(TemplateParameterKey::new("U"))],
        );
        inferred_payload.inferred_arguments = vec![TemplateArgumentInference {
            parameter: TemplateParameterKey::new("U"),
            inferred: NormalizedTypeId::new(11),
            evidence_key: CandidateProvenanceKey::new("exact-argument-type"),
        }];
        inferred.template = Some(inferred_payload);
        let mut missing = candidate("call", "root", "missing", CandidateScope::Local, 1);
        missing.template = Some(template_payload(
            "U=missing",
            vec![TemplateParameterKey::new("U")],
            vec![TemplateArgument::Omitted(TemplateParameterKey::new("U"))],
        ));
        let collection = OverloadCollectionOutput::collect(
            vec![site(
                "call",
                OverloadSiteKind::FunctorApplication,
                source_id,
                10,
            )],
            vec![missing, inferred],
        );

        let output = TemplateExpansionOutput::expand(&collection);

        assert_eq!(output.candidates().len(), 1);
        assert_eq!(
            output
                .expansions()
                .canonical_iter()
                .map(|(_, expansion)| &expansion.status)
                .collect::<Vec<_>>(),
            [
                &TemplateExpansionStatus::Instantiated,
                &TemplateExpansionStatus::Rejected(TemplateExpansionFailure::MissingInference)
            ]
        );
        assert!(
            output
                .debug_text()
                .contains("omitted_inference(\"exact-argument-type\")")
        );
        assert!(
            output
                .debug_text()
                .contains("message_key=\"overload.template.missing_inference\"")
        );
    }

    #[test]
    fn constraint_evidence_accepts_rejects_and_defers_templates() {
        let source_id = source_id(35);
        let accepted = constrained_template_candidate(
            "accepted",
            TemplateConstraintEvidenceStatus::Accepted,
            vec![TypeFactId::new(1)],
            0,
        );
        let missing = constrained_template_candidate(
            "missing",
            TemplateConstraintEvidenceStatus::Missing,
            Vec::new(),
            1,
        );
        let deferred = constrained_template_candidate(
            "deferred",
            TemplateConstraintEvidenceStatus::DeferredExternalDependency,
            Vec::new(),
            2,
        );
        let collection = OverloadCollectionOutput::collect(
            vec![site(
                "call",
                OverloadSiteKind::FunctorApplication,
                source_id,
                10,
            )],
            vec![deferred, accepted, missing],
        );

        let output = TemplateExpansionOutput::expand(&collection);

        assert_eq!(output.candidates().len(), 1);
        assert_eq!(
            output
                .expansions()
                .canonical_iter()
                .map(|(_, expansion)| &expansion.status)
                .collect::<Vec<_>>(),
            [
                &TemplateExpansionStatus::Instantiated,
                &TemplateExpansionStatus::Deferred(
                    TemplateExpansionFailure::DeferredConstraintEvidence
                ),
                &TemplateExpansionStatus::Rejected(
                    TemplateExpansionFailure::MissingConstraintEvidence
                )
            ]
        );
        let debug = output.debug_text();
        assert!(debug.contains("message_key=\"overload.template.missing_constraint_evidence\""));
        assert!(debug.contains("message_key=\"overload.template.deferred_constraint_evidence\""));
    }

    #[test]
    fn source_qua_template_arguments_must_be_accepted_widenings() {
        let source_id = source_id(36);
        let mut accepted = candidate("call", "root", "qua-ok", CandidateScope::Local, 0);
        accepted.template = Some(template_payload(
            "T=qua-ok",
            vec![TemplateParameterKey::new("T")],
            vec![TemplateArgument::SourceQua {
                source: NormalizedTypeId::new(3),
                target: NormalizedTypeId::new(4),
                path: QuaPathKey::new("widening-path"),
                status: TemplateQuaStatus::AcceptedWidening,
            }],
        ));
        let mut rejected = candidate("call", "root", "qua-bad", CandidateScope::Local, 1);
        rejected.template = Some(template_payload(
            "T=qua-bad",
            vec![TemplateParameterKey::new("T")],
            vec![TemplateArgument::SourceQua {
                source: NormalizedTypeId::new(4),
                target: NormalizedTypeId::new(3),
                path: QuaPathKey::new("narrowing-path"),
                status: TemplateQuaStatus::RejectedNarrowing,
            }],
        ));
        let mut deferred = candidate("call", "root", "qua-deferred", CandidateScope::Local, 2);
        deferred.template = Some(template_payload(
            "T=qua-deferred",
            vec![TemplateParameterKey::new("T")],
            vec![TemplateArgument::SourceQua {
                source: NormalizedTypeId::new(5),
                target: NormalizedTypeId::new(6),
                path: QuaPathKey::new("external-qua-path"),
                status: TemplateQuaStatus::DeferredExternalDependency,
            }],
        ));
        let collection = OverloadCollectionOutput::collect(
            vec![site(
                "call",
                OverloadSiteKind::FunctorApplication,
                source_id,
                10,
            )],
            vec![rejected, deferred, accepted],
        );

        let output = TemplateExpansionOutput::expand(&collection);

        assert_eq!(output.candidates().len(), 1);
        assert_eq!(
            output
                .expansions()
                .canonical_iter()
                .map(|(_, expansion)| &expansion.status)
                .collect::<Vec<_>>(),
            [
                &TemplateExpansionStatus::Rejected(TemplateExpansionFailure::RejectedSourceQua),
                &TemplateExpansionStatus::Deferred(TemplateExpansionFailure::DeferredSourceQua),
                &TemplateExpansionStatus::Instantiated
            ]
        );
        let debug = output.debug_text();
        assert!(debug.contains("source_qua(source=normalized_type#3, path=\"widening-path\")"));
        assert!(debug.contains("message_key=\"overload.template.rejected_source_qua\""));
        assert!(debug.contains("message_key=\"overload.template.deferred_source_qua\""));
    }

    #[test]
    fn template_expansion_diagnostics_do_not_reference_output_candidate_ids() {
        let source_id = source_id(37);
        let mut rejected = candidate("call", "root", "aaa-template", CandidateScope::Local, 0);
        rejected.template = Some(template_payload(
            "T=arity-mismatch",
            vec![TemplateParameterKey::new("T")],
            Vec::new(),
        ));
        let plain = candidate("call", "root", "zzz-plain", CandidateScope::Local, 1);
        let collection = OverloadCollectionOutput::collect(
            vec![site(
                "call",
                OverloadSiteKind::FunctorApplication,
                source_id,
                10,
            )],
            vec![rejected, plain],
        );

        let output = TemplateExpansionOutput::expand(&collection);

        assert_eq!(output.candidates().len(), 1);
        let (_, output_candidate) = output.candidates().iter().next().expect("candidate");
        assert_eq!(output_candidate.id, OverloadCandidateId::new(0));
        let (_, diagnostic) = output.diagnostics().iter().next().expect("diagnostic");
        assert_eq!(diagnostic.class, OverloadDiagnosticClass::TemplateExpansion);
        assert!(diagnostic.candidate.is_none());
        let (_, expansion) = output.expansions().iter().next().expect("expansion");
        assert_eq!(expansion.source_candidate, OverloadCandidateId::new(0));
    }

    #[test]
    fn non_template_candidate_diagnostics_are_remapped_into_expansion_output() {
        let source_id = source_id(38);
        let mut input = candidate("call", "root", "unsupported", CandidateScope::Local, 0);
        input.declaration_kind =
            CandidateDeclarationKind::Unsupported(UnsupportedOverloadRole::TheoremApplication);
        let collection = OverloadCollectionOutput::collect(
            vec![site(
                "call",
                OverloadSiteKind::FunctorApplication,
                source_id,
                10,
            )],
            vec![input],
        );

        let output = TemplateExpansionOutput::expand(&collection);

        let (_, candidate) = output.candidates().iter().next().expect("candidate");
        assert_eq!(candidate.id, OverloadCandidateId::new(0));
        assert_eq!(candidate.diagnostics.len(), 1);
        let diagnostic = output
            .diagnostics()
            .get(candidate.diagnostics[0])
            .expect("remapped diagnostic");
        assert_eq!(
            diagnostic.class,
            OverloadDiagnosticClass::UnsupportedCandidateRole
        );
        assert_eq!(diagnostic.candidate, Some(candidate.id));
        assert!(
            output
                .debug_text()
                .contains("class=unsupported_candidate_role")
        );
    }

    #[test]
    fn deferred_template_candidates_preserve_exclusion_reasons() {
        let source_id = source_id(39);
        let mut input = candidate("scheme", "root", "symbol", CandidateScope::Local, 0);
        input.template = Some(template_payload(
            "T=deferred",
            vec![TemplateParameterKey::new("T")],
            vec![TemplateArgument::Explicit(NormalizedTypeId::new(5))],
        ));
        let collection = OverloadCollectionOutput::collect(
            vec![site(
                "scheme",
                OverloadSiteKind::Unsupported(UnsupportedOverloadRole::SchemeApplication),
                source_id,
                10,
            )],
            vec![input],
        );

        let output = TemplateExpansionOutput::expand(&collection);

        assert!(output.candidates().is_empty());
        let (_, expansion) = output.expansions().iter().next().expect("expansion");
        assert_eq!(
            expansion.status,
            TemplateExpansionStatus::Deferred(TemplateExpansionFailure::DeferredCandidate)
        );
        assert_eq!(output.diagnostics().len(), 1);
        assert!(
            output
                .debug_text()
                .contains("message_key=\"overload.template.deferred_candidate\"")
        );
    }

    #[test]
    fn source_qua_and_recovery_site_provenance_are_retained() {
        let source_id = source_id(4);
        let mut input = site(
            "qua-site",
            OverloadSiteKind::FunctorApplication,
            source_id,
            10,
        );
        input.source_qua = vec![SourceQuaView {
            argument: TypedSiteRef::Role {
                node: TypedNodeId::new(2),
                role: TypeRole::new("argument"),
            },
            target: NormalizedTypeId::new(9),
            source_range: range(source_id, 12, 15),
            path: Some(QuaPathKey::new("mode-upcast")),
            evidence_facts: vec![TypeFactId::new(3)],
        }];
        input.recovery = OverloadSiteRecovery::Degraded {
            message_key: OverloadDiagnosticMessageKey::new("overload.site.recovered"),
        };

        let output = OverloadCollectionOutput::collect(vec![input], []);

        let (_, site) = output.sites().iter().next().expect("site collected");
        assert_eq!(site.status, OverloadSiteStatus::Degraded);
        assert_eq!(
            site.owner,
            TypedSiteRef::Node(TypedNodeId::new(1)),
            "owner is part of site provenance"
        );
        assert_range(site.source_range, 10, 15);
        assert_eq!(site.name.as_str(), "qua-site-name");
        assert_eq!(site.arguments, [TypedSiteRef::Node(TypedNodeId::new(101))]);
        assert_eq!(site.expected, Some(NormalizedTypeId::new(1)));
        assert_eq!(site.source_qua[0].target, NormalizedTypeId::new(9));
        assert_eq!(
            site.recovery,
            OverloadSiteRecovery::Degraded {
                message_key: OverloadDiagnosticMessageKey::new("overload.site.recovered")
            }
        );
        assert_eq!(output.diagnostics().len(), 1);
        let debug = output.debug_text();
        assert!(debug.contains("path=\"mode-upcast\""));
        assert!(debug.contains("facts=[fact#3]"));
        assert!(debug.contains("message_key=\"overload.site.recovered\""));
    }

    #[test]
    fn unknown_candidate_site_is_diagnosed_without_insertion() {
        let mut missing = candidate("missing", "root", "symbol", CandidateScope::Local, 0);
        missing.template = Some(TemplateCandidatePayload {
            template: symbol_id("missing-template"),
            instantiation_key: TemplateInstantiationKey::new("T=missing"),
            parameters: vec![TemplateParameterKey::new("T")],
            arguments: vec![TemplateArgument::Explicit(NormalizedTypeId::new(4))],
            inferred_arguments: Vec::new(),
            constraints: Vec::new(),
        });
        missing.coherence = Some(CoherenceStatus::Pending);
        let output = OverloadCollectionOutput::collect(Vec::new(), vec![missing]);

        assert!(output.sites().is_empty());
        assert!(output.candidates().is_empty());
        assert_eq!(output.diagnostics().len(), 1);
        let (_, diagnostic) = output
            .diagnostics()
            .iter()
            .next()
            .expect("missing site diagnostic");
        assert_eq!(diagnostic.class, OverloadDiagnosticClass::MissingSite);
        assert_eq!(
            diagnostic.site_key.as_ref().map(OverloadSiteKey::as_str),
            Some("missing")
        );
        let Some(OverloadDiagnosticProvenance::CandidateInput {
            declaration_kind,
            template,
            coherence,
            provenance,
            ..
        }) = &diagnostic.provenance
        else {
            panic!("missing candidate diagnostic keeps rejected candidate provenance");
        };
        assert_eq!(*declaration_kind, CandidateDeclarationKind::Functor);
        assert_eq!(*coherence, Some(CoherenceStatus::Pending));
        assert_eq!(
            template
                .as_ref()
                .map(|payload| payload.instantiation_key.as_str()),
            Some("T=missing")
        );
        assert_eq!(provenance.stable_key.as_str(), "symbol-provenance");
        assert_range(provenance.source_range.expect("candidate range"), 0, 1);
        assert_eq!(provenance.declaration_order, 0);
        assert!(output.debug_text().contains("coherence=pending"));
        assert!(output.debug_text().contains("instantiation=\"T=missing\""));
    }

    #[test]
    fn unsupported_roles_are_deferred_with_stable_diagnostics() {
        let source_id = source_id(5);
        let output = OverloadCollectionOutput::collect(
            vec![site(
                "scheme",
                OverloadSiteKind::Unsupported(UnsupportedOverloadRole::SchemeApplication),
                source_id,
                10,
            )],
            vec![OverloadCandidateInput {
                declaration_kind: CandidateDeclarationKind::Unsupported(
                    UnsupportedOverloadRole::TheoremApplication,
                ),
                ..candidate("scheme", "root", "symbol", CandidateScope::Local, 0)
            }],
        );

        let (_, site) = output.sites().iter().next().expect("site retained");
        assert_eq!(site.status, OverloadSiteStatus::Deferred);
        let (_, candidate) = output
            .candidates()
            .iter()
            .next()
            .expect("candidate retained");
        assert_eq!(candidate.status, OverloadCandidateStatus::Deferred);
        assert_eq!(output.diagnostics().len(), 2);
        let debug = output.debug_text();
        assert!(debug.contains("message_key=\"overload.site.unsupported.scheme_application\""));
        assert!(
            debug.contains("message_key=\"overload.candidate.unsupported.theorem_application\"")
        );
    }

    #[test]
    fn duplicate_site_keys_are_diagnosed_deterministically() {
        let source_id = source_id(6);
        let output = OverloadCollectionOutput::collect(
            vec![
                site("dup", OverloadSiteKind::FunctorApplication, source_id, 20),
                site("dup", OverloadSiteKind::PredicateApplication, source_id, 10),
            ],
            Vec::new(),
        );

        assert_eq!(output.sites().len(), 1);
        assert_eq!(output.diagnostics().len(), 1);
        let (_, diagnostic) = output.diagnostics().iter().next().expect("diagnostic");
        let Some(OverloadDiagnosticProvenance::SiteInput {
            source_range,
            kind,
            name,
            ..
        }) = &diagnostic.provenance
        else {
            panic!("duplicate site diagnostic keeps skipped site provenance");
        };
        assert_range(*source_range, 20, 25);
        assert_eq!(*kind, OverloadSiteKind::FunctorApplication);
        assert_eq!(name.as_str(), "dup-name");
        let debug = output.debug_text();
        assert!(debug.contains("key=\"dup\" kind=predicate_application"));
        assert!(debug.contains("class=duplicate_site_key"));
        assert!(debug.contains("provenance=site_input"));
    }

    fn site(
        key: &str,
        kind: OverloadSiteKind,
        source_id: SourceId,
        start: usize,
    ) -> OverloadSiteInput {
        OverloadSiteInput {
            key: OverloadSiteKey::new(key),
            owner: TypedSiteRef::Node(TypedNodeId::new(start / 10)),
            source_range: range(source_id, start, start + 5),
            kind,
            name: OverloadNameKey::new(format!("{key}-name")),
            arguments: vec![TypedSiteRef::Node(TypedNodeId::new((start / 10) + 100))],
            expected: Some(NormalizedTypeId::new(start / 10)),
            source_qua: Vec::new(),
            recovery: OverloadSiteRecovery::Normal,
        }
    }

    fn candidate(
        site: &str,
        root: &str,
        symbol: &str,
        scope: CandidateScope,
        declaration_order: usize,
    ) -> OverloadCandidateInput {
        OverloadCandidateInput {
            site: OverloadSiteKey::new(site),
            symbol: symbol_id(symbol),
            ordinary_root: symbol_id(root),
            declaration_kind: CandidateDeclarationKind::Functor,
            parameters: vec![NormalizedTypeId::new(1), NormalizedTypeId::new(2)],
            result: Some(NormalizedTypeId::new(3)),
            origin: CandidateOrigin::Ordinary,
            template: None,
            coherence: None,
            provenance: CandidateProvenance {
                stable_key: CandidateProvenanceKey::new(format!("{symbol}-provenance")),
                source_range: Some(range(
                    source_id(99),
                    declaration_order,
                    declaration_order + 1,
                )),
                scope,
                declaration_order,
            },
        }
    }

    fn template_payload(
        instantiation: &str,
        parameters: Vec<TemplateParameterKey>,
        arguments: Vec<TemplateArgument>,
    ) -> TemplateCandidatePayload {
        TemplateCandidatePayload {
            template: symbol_id("template-source"),
            instantiation_key: TemplateInstantiationKey::new(instantiation),
            parameters,
            arguments,
            inferred_arguments: Vec::new(),
            constraints: Vec::new(),
        }
    }

    fn constrained_template_candidate(
        symbol: &str,
        status: TemplateConstraintEvidenceStatus,
        facts: Vec<TypeFactId>,
        declaration_order: usize,
    ) -> OverloadCandidateInput {
        let mut input = candidate(
            "call",
            "root",
            symbol,
            CandidateScope::Local,
            declaration_order,
        );
        let mut payload = template_payload(
            &format!("T={symbol}"),
            vec![TemplateParameterKey::new("T")],
            vec![TemplateArgument::Explicit(NormalizedTypeId::new(7))],
        );
        payload.constraints = vec![TemplateConstraintEvidence {
            parameter: TemplateParameterKey::new("T"),
            evidence_key: CandidateProvenanceKey::new(format!("{symbol}-constraint")),
            facts,
            status,
        }];
        input.template = Some(payload);
        input
    }

    fn symbol_id(name: &str) -> SymbolId {
        SymbolId::new(
            module("work", "article"),
            LocalSymbolId::new(name),
            FullyQualifiedName::new(format!("work::article::{name}")),
        )
    }

    fn module(package: &str, path: &str) -> ModuleId {
        ModuleId::new(PackageId::new(package), ModulePath::new(path))
    }

    fn range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
        SourceRange {
            source_id,
            start,
            end,
        }
    }

    fn assert_range(range: SourceRange, start: usize, end: usize) {
        assert_eq!(range.start, start);
        assert_eq!(range.end, end);
    }

    fn source_id(seed: u8) -> SourceId {
        let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{seed:064x}"
        ))
        .expect("valid build snapshot id");
        InMemorySessionIdAllocator::new()
            .next_source_id(snapshot)
            .expect("source id allocation succeeds")
    }
}
