//! Type-expression normalization and declaration checking for checker phase 6.

use crate::{
    binding_env::{
        BindingContextId, BindingContextLayer, BindingContextRecovery, BindingEnv, BindingId,
        BindingKind, BindingStatus,
    },
    typed_ast::{
        BindingTypeRef, ContextRecoveryState, DiagnosticRecoveryState, FactProvenance, FactStatus,
        LocalTypeContextDraft, LocalTypeContextId, LocalTypeContextTable, NormalizedTypeId,
        OpenCandidateSetId, Polarity, SourceRangeKey as TypedSourceRangeKey, TypeAssumptionId,
        TypeContextLayer, TypeDiagnostic, TypeDiagnosticClass, TypeDiagnosticDraft,
        TypeDiagnosticId, TypeDiagnosticSeverity, TypeDiagnosticTable, TypeEntryActual,
        TypeEntryDraft, TypeEntryId, TypeFactDraft, TypeFactId, TypeFactTable, TypePredicateRef,
        TypeProvenance, TypeRuleId, TypeStatus, TypeTable, TypedSiteRef, TypedSubjectRef,
    },
};
use mizar_resolve::{
    env::{SymbolEnv, SymbolKind},
    resolved_ast::SymbolId,
};
use mizar_session::SourceRange;
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write as _,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeNormalizationOutput {
    normalized_types: NormalizedTypeTable,
    type_entries: TypeTable,
    diagnostics: TypeDiagnosticTable,
}

impl TypeNormalizationOutput {
    pub const fn normalized_types(&self) -> &NormalizedTypeTable {
        &self.normalized_types
    }

    pub const fn type_entries(&self) -> &TypeTable {
        &self.type_entries
    }

    pub const fn diagnostics(&self) -> &TypeDiagnosticTable {
        &self.diagnostics
    }

    pub fn debug_text(&self) -> String {
        let mut output = String::from("type-normalization-debug-v1\n");
        write_normalized_types(&mut output, &self.normalized_types);
        write_type_entries(&mut output, &self.type_entries);
        write_diagnostics(&mut output, &self.diagnostics);
        output
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TypeNormalizer {
    mode_expansions: BTreeMap<SymbolId, ModeExpansion>,
}

impl TypeNormalizer {
    pub fn new(mode_expansions: impl IntoIterator<Item = (SymbolId, ModeExpansion)>) -> Self {
        let mode_expansions = mode_expansions.into_iter().collect();
        Self { mode_expansions }
    }

    pub fn normalize(
        &self,
        symbols: &SymbolEnv,
        inputs: impl IntoIterator<Item = TypeExpressionInput>,
    ) -> TypeNormalizationOutput {
        let mut state = NormalizationState {
            symbols,
            mode_expansions: &self.mode_expansions,
            normalized_types: NormalizedTypeTable::new(),
            type_entries: TypeTable::new(),
            diagnostics: TypeDiagnosticTable::new(),
        };
        let mut seen_sites = BTreeSet::new();

        for input in inputs {
            let site = input.site.clone();
            let range = input.source_range;
            if !seen_sites.insert(site.clone()) {
                state.diagnostic(
                    Some(site),
                    range,
                    TypeDiagnosticClass::TypeExpression,
                    TypeDiagnosticSeverity::Error,
                    "checker.type.duplicate_site",
                    DiagnosticRecoveryState::Degraded,
                );
                continue;
            }

            let id = state.normalize_input(input);
            let normalized_status = state
                .normalized_types
                .get(id)
                .map(|normalized| normalized.status)
                .unwrap_or(NormalizedTypeStatus::Error);
            let (status, provenance) = match normalized_status {
                NormalizedTypeStatus::Known => (
                    TypeStatus::Known,
                    TypeProvenance::Inferred(TypeRuleId::new("type-expression-normalization")),
                ),
                NormalizedTypeStatus::Degraded => (
                    TypeStatus::Unknown,
                    TypeProvenance::Recovery(state.recovery_diagnostic(
                        site.clone(),
                        range,
                        "checker.type.recovery",
                    )),
                ),
                NormalizedTypeStatus::Error => (
                    TypeStatus::Error,
                    TypeProvenance::Recovery(state.recovery_diagnostic(
                        site.clone(),
                        range,
                        "checker.type.recovery",
                    )),
                ),
            };
            state.type_entries.insert(TypeEntryDraft {
                owner: site,
                expected: None,
                actual: TypeEntryActual::Known(id),
                status,
                provenance,
            });
        }
        let (normalized_types, type_remap) = state.normalized_types.into_canonical();
        let type_entries = remap_type_table(state.type_entries, &type_remap);

        TypeNormalizationOutput {
            normalized_types,
            type_entries,
            diagnostics: state.diagnostics,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclarationCheckingOutput {
    normalized_types: NormalizedTypeTable,
    declarations: CheckedDeclarationTable,
    contexts: LocalTypeContextTable,
    type_entries: TypeTable,
    facts: TypeFactTable,
    diagnostics: TypeDiagnosticTable,
}

impl DeclarationCheckingOutput {
    pub const fn normalized_types(&self) -> &NormalizedTypeTable {
        &self.normalized_types
    }

    pub const fn declarations(&self) -> &CheckedDeclarationTable {
        &self.declarations
    }

    pub const fn contexts(&self) -> &LocalTypeContextTable {
        &self.contexts
    }

    pub const fn type_entries(&self) -> &TypeTable {
        &self.type_entries
    }

    pub const fn facts(&self) -> &TypeFactTable {
        &self.facts
    }

    pub const fn diagnostics(&self) -> &TypeDiagnosticTable {
        &self.diagnostics
    }

    pub fn debug_text(&self) -> String {
        let mut output = String::from("declaration-checking-debug-v1\n");
        write_normalized_types(&mut output, &self.normalized_types);
        write_checked_declarations(&mut output, &self.declarations);
        write_local_contexts(&mut output, &self.contexts);
        write_type_entries(&mut output, &self.type_entries);
        write_type_facts(&mut output, &self.facts);
        write_diagnostics(&mut output, &self.diagnostics);
        output
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DeclarationChecker {
    normalizer: TypeNormalizer,
}

impl DeclarationChecker {
    pub fn new(normalizer: TypeNormalizer) -> Self {
        Self { normalizer }
    }

    pub fn check(
        &self,
        symbols: &SymbolEnv,
        binding_env: &BindingEnv,
        context_inputs: impl IntoIterator<Item = DeclarationContextInput>,
        declaration_inputs: impl IntoIterator<Item = DeclarationInput>,
    ) -> DeclarationCheckingOutput {
        let mut context_inputs = context_inputs.into_iter().collect::<Vec<_>>();
        context_inputs.sort_by_key(declaration_context_input_key);
        let mut declarations = declaration_inputs.into_iter().collect::<Vec<_>>();
        declarations.sort_by_key(declaration_input_key);

        let type_inputs = declarations
            .iter()
            .filter(|declaration| declaration.uses_explicit_type_payload())
            .filter_map(|declaration| declaration.type_expression.clone())
            .collect::<Vec<_>>();
        let normalized = self.normalizer.normalize(symbols, type_inputs);
        let type_entries_by_site = type_entries_by_site(normalized.type_entries());

        let mut state = DeclarationCheckingState {
            binding_env,
            normalized_types: normalized.normalized_types,
            declarations: CheckedDeclarationTable::new(),
            type_entries: normalized.type_entries,
            facts: TypeFactTable::new(),
            diagnostics: normalized.diagnostics,
            seen_sites: BTreeSet::new(),
            seen_bindings: BTreeSet::new(),
            context_bindings: BTreeMap::new(),
            context_facts: BTreeMap::new(),
        };

        for declaration in declarations {
            state.check_declaration(declaration, &type_entries_by_site);
        }

        let contexts = build_local_contexts(
            binding_env,
            &context_inputs,
            &state.context_bindings,
            &state.context_facts,
            &mut state.diagnostics,
        );

        DeclarationCheckingOutput {
            normalized_types: state.normalized_types,
            declarations: state.declarations,
            contexts,
            type_entries: state.type_entries,
            facts: state.facts,
            diagnostics: state.diagnostics,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TermFormulaInferenceOutput {
    normalized_types: NormalizedTypeTable,
    terms: CheckedTermTable,
    formulas: CheckedFormulaTable,
    candidate_sets: OpenCandidateSetTable,
    type_entries: TypeTable,
    facts: TypeFactTable,
    diagnostics: TypeDiagnosticTable,
}

impl TermFormulaInferenceOutput {
    pub const fn normalized_types(&self) -> &NormalizedTypeTable {
        &self.normalized_types
    }

    pub const fn terms(&self) -> &CheckedTermTable {
        &self.terms
    }

    pub const fn formulas(&self) -> &CheckedFormulaTable {
        &self.formulas
    }

    pub const fn candidate_sets(&self) -> &OpenCandidateSetTable {
        &self.candidate_sets
    }

    pub const fn type_entries(&self) -> &TypeTable {
        &self.type_entries
    }

    pub const fn facts(&self) -> &TypeFactTable {
        &self.facts
    }

    pub const fn diagnostics(&self) -> &TypeDiagnosticTable {
        &self.diagnostics
    }

    pub fn debug_text(&self) -> String {
        let mut output = String::from("term-formula-inference-debug-v1\n");
        write_normalized_types(&mut output, &self.normalized_types);
        write_checked_terms(&mut output, &self.terms);
        write_checked_formulas(&mut output, &self.formulas);
        write_candidate_sets(&mut output, &self.candidate_sets);
        write_type_entries(&mut output, &self.type_entries);
        write_type_facts(&mut output, &self.facts);
        write_diagnostics(&mut output, &self.diagnostics);
        output
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TermFormulaChecker {
    normalizer: TypeNormalizer,
}

impl TermFormulaChecker {
    pub fn new(normalizer: TypeNormalizer) -> Self {
        Self { normalizer }
    }

    pub fn infer(
        &self,
        symbols: &SymbolEnv,
        binding_env: &BindingEnv,
        term_inputs: impl IntoIterator<Item = TermInput>,
        formula_inputs: impl IntoIterator<Item = FormulaInput>,
    ) -> TermFormulaInferenceOutput {
        let mut terms = term_inputs.into_iter().collect::<Vec<_>>();
        terms.sort_by_key(term_input_key);
        let mut formulas = formula_inputs.into_iter().collect::<Vec<_>>();
        formulas.sort_by_key(formula_input_key);

        let mut type_inputs = Vec::new();
        for term in &terms {
            term.collect_type_inputs(&mut type_inputs);
        }
        for formula in &formulas {
            formula.collect_type_inputs(&mut type_inputs);
        }
        let normalized = self.normalizer.normalize(symbols, type_inputs);
        let type_entries_by_site = type_entries_by_site(normalized.type_entries());

        let mut state = TermFormulaCheckingState {
            symbols,
            binding_env,
            normalized_types: normalized.normalized_types,
            terms: CheckedTermTable::new(),
            formulas: CheckedFormulaTable::new(),
            candidate_sets: OpenCandidateSetTable::new(),
            type_entries: normalized.type_entries,
            facts: TypeFactTable::new(),
            diagnostics: normalized.diagnostics,
            seen_terms: BTreeSet::new(),
            seen_formulas: BTreeSet::new(),
        };

        for term in terms {
            state.check_term(term, &type_entries_by_site);
        }
        for formula in formulas {
            state.check_formula(formula, &type_entries_by_site);
        }

        TermFormulaInferenceOutput {
            normalized_types: state.normalized_types,
            terms: state.terms,
            formulas: state.formulas,
            candidate_sets: state.candidate_sets,
            type_entries: state.type_entries,
            facts: state.facts,
            diagnostics: state.diagnostics,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TermInput {
    pub site: TypedSiteRef,
    pub context: BindingContextId,
    pub source_range: SourceRange,
    pub kind: TermKind,
    pub reference: Option<TermReference>,
    pub result_type: Option<TypeExpressionInput>,
    pub expected_type: Option<TypeExpressionInput>,
    pub candidates: Vec<OpenCandidateInput>,
    pub deferred: Vec<TermDeferredReason>,
}

impl TermInput {
    pub fn new(
        site: TypedSiteRef,
        context: BindingContextId,
        source_range: SourceRange,
        kind: TermKind,
    ) -> Self {
        Self {
            site,
            context,
            source_range,
            kind,
            reference: None,
            result_type: None,
            expected_type: None,
            candidates: Vec::new(),
            deferred: Vec::new(),
        }
    }

    pub fn with_reference(mut self, reference: TermReference) -> Self {
        self.reference = Some(reference);
        self
    }

    pub fn with_result_type(mut self, result_type: TypeExpressionInput) -> Self {
        self.result_type = Some(result_type);
        self
    }

    pub fn with_expected_type(mut self, expected_type: TypeExpressionInput) -> Self {
        self.expected_type = Some(expected_type);
        self
    }

    pub fn with_candidates(mut self, candidates: Vec<OpenCandidateInput>) -> Self {
        self.candidates = candidates;
        self
    }

    pub fn with_deferred(mut self, deferred: Vec<TermDeferredReason>) -> Self {
        self.deferred = deferred;
        self
    }

    fn collect_type_inputs(&self, output: &mut Vec<TypeExpressionInput>) {
        if let Some(result_type) = &self.result_type {
            output.push(result_type.clone());
        }
        if let Some(expected_type) = &self.expected_type {
            output.push(expected_type.clone());
        }
        for candidate in &self.candidates {
            candidate.collect_type_inputs(output);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum TermKind {
    Variable,
    It,
    Numeral,
    FunctorApplication,
    SelectorAccess,
    StructureConstructor,
    SetEnumeration,
    SetComprehension,
    Choice,
    SourceQua,
    Parenthesized,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum TermReference {
    Binding(BindingId),
    Symbol(SymbolId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum TermDeferredReason {
    MissingReferencePayload,
    MissingNumericTypePayload,
    MissingSignaturePayload,
    MissingSelectorPayload,
    MissingStructurePayload,
    MissingResultTypePayload,
    SethoodRequirement,
    NonEmptinessRequirement,
    SourceQuaRequirement,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormulaInput {
    pub site: TypedSiteRef,
    pub context: BindingContextId,
    pub source_range: SourceRange,
    pub kind: FormulaKind,
    pub terms: Vec<TypedSiteRef>,
    pub asserted_type: Option<TypeExpressionInput>,
    pub expected_types: Vec<ExpectedTypeInput>,
    pub candidates: Vec<OpenCandidateInput>,
    pub facts: Vec<FormulaFactInput>,
    pub deferred: Vec<FormulaDeferredReason>,
}

impl FormulaInput {
    pub fn new(
        site: TypedSiteRef,
        context: BindingContextId,
        source_range: SourceRange,
        kind: FormulaKind,
    ) -> Self {
        Self {
            site,
            context,
            source_range,
            kind,
            terms: Vec::new(),
            asserted_type: None,
            expected_types: Vec::new(),
            candidates: Vec::new(),
            facts: Vec::new(),
            deferred: Vec::new(),
        }
    }

    pub fn with_terms(mut self, terms: Vec<TypedSiteRef>) -> Self {
        self.terms = terms;
        self
    }

    pub fn with_asserted_type(mut self, asserted_type: TypeExpressionInput) -> Self {
        self.asserted_type = Some(asserted_type);
        self
    }

    pub fn with_expected_types(mut self, expected_types: Vec<ExpectedTypeInput>) -> Self {
        self.expected_types = expected_types;
        self
    }

    pub fn with_candidates(mut self, candidates: Vec<OpenCandidateInput>) -> Self {
        self.candidates = candidates;
        self
    }

    pub fn with_facts(mut self, facts: Vec<FormulaFactInput>) -> Self {
        self.facts = facts;
        self
    }

    pub fn with_deferred(mut self, deferred: Vec<FormulaDeferredReason>) -> Self {
        self.deferred = deferred;
        self
    }

    fn collect_type_inputs(&self, output: &mut Vec<TypeExpressionInput>) {
        if let Some(asserted_type) = &self.asserted_type {
            output.push(asserted_type.clone());
        }
        for expected_type in &self.expected_types {
            output.push(expected_type.expected.clone());
        }
        for candidate in &self.candidates {
            candidate.collect_type_inputs(output);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum FormulaKind {
    PredicateApplication,
    Equality,
    Inequality,
    Membership,
    TypeAssertion,
    AttributeAssertion,
    Negation,
    Conjunction,
    Disjunction,
    Implication,
    Biconditional,
    Quantified,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum FormulaDeferredReason {
    MissingPredicateSignaturePayload,
    MissingExpectedTypePayload,
    MissingQuantifierPayload,
    MissingFormulaPayload,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpectedTypeInput {
    pub term: TypedSiteRef,
    pub expected: TypeExpressionInput,
    pub source_range: SourceRange,
}

impl ExpectedTypeInput {
    pub fn new(
        term: TypedSiteRef,
        expected: TypeExpressionInput,
        source_range: SourceRange,
    ) -> Self {
        Self {
            term,
            expected,
            source_range,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormulaFactInput {
    pub subject: TypedSubjectRef,
    pub predicate: TypePredicateRef,
    pub polarity: Polarity,
    pub source_range: SourceRange,
}

impl FormulaFactInput {
    pub fn new(
        subject: TypedSubjectRef,
        predicate: TypePredicateRef,
        polarity: Polarity,
        source_range: SourceRange,
    ) -> Self {
        Self {
            subject,
            predicate,
            polarity,
            source_range,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenCandidateInput {
    pub identity: CandidateIdentity,
    pub source_range: SourceRange,
    pub result_type: Option<TypeExpressionInput>,
    pub required_types: Vec<TypeExpressionInput>,
}

impl OpenCandidateInput {
    pub fn new(identity: CandidateIdentity, source_range: SourceRange) -> Self {
        Self {
            identity,
            source_range,
            result_type: None,
            required_types: Vec::new(),
        }
    }

    pub fn with_result_type(mut self, result_type: TypeExpressionInput) -> Self {
        self.result_type = Some(result_type);
        self
    }

    pub fn with_required_types(mut self, required_types: Vec<TypeExpressionInput>) -> Self {
        self.required_types = required_types;
        self
    }

    fn collect_type_inputs(&self, output: &mut Vec<TypeExpressionInput>) {
        if let Some(result_type) = &self.result_type {
            output.push(result_type.clone());
        }
        output.extend(self.required_types.iter().cloned());
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum CandidateIdentity {
    Symbol(SymbolId),
    Builtin(String),
    External(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OpenCandidateSetTable {
    entries: Vec<OpenCandidateSet>,
}

impl OpenCandidateSetTable {
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    fn insert(&mut self, draft: OpenCandidateSetDraft) -> OpenCandidateSetId {
        let id = OpenCandidateSetId::new(self.entries.len());
        let mut candidates = draft.candidates;
        candidates.sort_by_key(open_candidate_key);
        self.entries.push(OpenCandidateSet {
            id,
            owner: draft.owner,
            kind: draft.kind,
            candidates,
            status: draft.status,
            source_range: draft.source_range,
        });
        id
    }

    pub fn get(&self, id: OpenCandidateSetId) -> Option<&OpenCandidateSet> {
        self.entries.get(id.index())
    }

    pub fn iter(&self) -> impl Iterator<Item = (OpenCandidateSetId, &OpenCandidateSet)> {
        self.entries.iter().map(|entry| (entry.id, entry))
    }

    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenCandidateSet {
    pub id: OpenCandidateSetId,
    pub owner: TypedSiteRef,
    pub kind: CandidateSetKind,
    pub candidates: Vec<OpenCandidate>,
    pub status: CandidateSetStatus,
    pub source_range: SourceRange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct OpenCandidateSetDraft {
    owner: TypedSiteRef,
    kind: CandidateSetKind,
    candidates: Vec<OpenCandidate>,
    status: CandidateSetStatus,
    source_range: SourceRange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenCandidate {
    pub identity: CandidateIdentity,
    pub result_type: Option<NormalizedTypeId>,
    pub required_types: Vec<NormalizedTypeId>,
    pub source_range: SourceRange,
    pub status: CandidateStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum CandidateSetKind {
    Functor,
    Predicate,
    Selector,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum CandidateSetStatus {
    Open,
    Degraded,
    Rejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum CandidateStatus {
    Viable,
    Degraded,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CheckedTermTable {
    entries: Vec<CheckedTerm>,
}

impl CheckedTermTable {
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    fn insert(&mut self, draft: CheckedTermDraft) -> CheckedTermId {
        let id = CheckedTermId::new(self.entries.len());
        self.entries.push(CheckedTerm {
            id,
            site: draft.site,
            context: draft.context,
            kind: draft.kind,
            reference: draft.reference,
            type_entry: draft.type_entry,
            expected_type: draft.expected_type,
            candidate_set: draft.candidate_set,
            status: draft.status,
            deferred: draft.deferred,
        });
        id
    }

    pub fn get(&self, id: CheckedTermId) -> Option<&CheckedTerm> {
        self.entries.get(id.index())
    }

    pub fn iter(&self) -> impl Iterator<Item = (CheckedTermId, &CheckedTerm)> {
        self.entries.iter().map(|entry| (entry.id, entry))
    }

    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CheckedTermId(usize);

impl CheckedTermId {
    pub const fn new(index: usize) -> Self {
        Self(index)
    }

    pub const fn index(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckedTerm {
    pub id: CheckedTermId,
    pub site: TypedSiteRef,
    pub context: BindingContextId,
    pub kind: TermKind,
    pub reference: Option<TermReference>,
    pub type_entry: TypeEntryId,
    pub expected_type: Option<NormalizedTypeId>,
    pub candidate_set: Option<OpenCandidateSetId>,
    pub status: TermStatus,
    pub deferred: Vec<TermDeferredReason>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CheckedTermDraft {
    site: TypedSiteRef,
    context: BindingContextId,
    kind: TermKind,
    reference: Option<TermReference>,
    type_entry: TypeEntryId,
    expected_type: Option<NormalizedTypeId>,
    candidate_set: Option<OpenCandidateSetId>,
    status: TermStatus,
    deferred: Vec<TermDeferredReason>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum TermStatus {
    Inferred,
    Partial,
    Error,
    Skipped,
}

impl TermStatus {
    const fn max_partial(self) -> Self {
        match self {
            Self::Inferred => Self::Partial,
            Self::Partial | Self::Error | Self::Skipped => self,
        }
    }

    const fn max_error(self) -> Self {
        match self {
            Self::Skipped => Self::Skipped,
            Self::Inferred | Self::Partial | Self::Error => Self::Error,
        }
    }

    const fn skip(self) -> Self {
        match self {
            Self::Error => Self::Error,
            Self::Inferred | Self::Partial | Self::Skipped => Self::Skipped,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CheckedFormulaTable {
    entries: Vec<CheckedFormula>,
}

impl CheckedFormulaTable {
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    fn insert(&mut self, draft: CheckedFormulaDraft) -> CheckedFormulaId {
        let id = CheckedFormulaId::new(self.entries.len());
        self.entries.push(CheckedFormula {
            id,
            site: draft.site,
            context: draft.context,
            kind: draft.kind,
            terms: draft.terms,
            asserted_type: draft.asserted_type,
            expected_types: draft.expected_types,
            candidate_set: draft.candidate_set,
            facts: draft.facts,
            status: draft.status,
            deferred: draft.deferred,
        });
        id
    }

    pub fn get(&self, id: CheckedFormulaId) -> Option<&CheckedFormula> {
        self.entries.get(id.index())
    }

    pub fn iter(&self) -> impl Iterator<Item = (CheckedFormulaId, &CheckedFormula)> {
        self.entries.iter().map(|entry| (entry.id, entry))
    }

    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CheckedFormulaId(usize);

impl CheckedFormulaId {
    pub const fn new(index: usize) -> Self {
        Self(index)
    }

    pub const fn index(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckedFormula {
    pub id: CheckedFormulaId,
    pub site: TypedSiteRef,
    pub context: BindingContextId,
    pub kind: FormulaKind,
    pub terms: Vec<TypedSiteRef>,
    pub asserted_type: Option<NormalizedTypeId>,
    pub expected_types: Vec<ExpectedTypeConstraint>,
    pub candidate_set: Option<OpenCandidateSetId>,
    pub facts: Vec<TypeFactId>,
    pub status: FormulaStatus,
    pub deferred: Vec<FormulaDeferredReason>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CheckedFormulaDraft {
    site: TypedSiteRef,
    context: BindingContextId,
    kind: FormulaKind,
    terms: Vec<TypedSiteRef>,
    asserted_type: Option<NormalizedTypeId>,
    expected_types: Vec<ExpectedTypeConstraint>,
    candidate_set: Option<OpenCandidateSetId>,
    facts: Vec<TypeFactId>,
    status: FormulaStatus,
    deferred: Vec<FormulaDeferredReason>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum FormulaStatus {
    Checked,
    Partial,
    Error,
    Skipped,
}

impl FormulaStatus {
    const fn max_partial(self) -> Self {
        match self {
            Self::Checked => Self::Partial,
            Self::Partial | Self::Error | Self::Skipped => self,
        }
    }

    const fn max_error(self) -> Self {
        match self {
            Self::Skipped => Self::Skipped,
            Self::Checked | Self::Partial | Self::Error => Self::Error,
        }
    }

    const fn skip(self) -> Self {
        match self {
            Self::Error => Self::Error,
            Self::Checked | Self::Partial | Self::Skipped => Self::Skipped,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpectedTypeConstraint {
    pub term: TypedSiteRef,
    pub expected: NormalizedTypeId,
    pub status: TypeStatus,
    pub source_range: SourceRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CheckedTermState {
    context: BindingContextId,
    readiness: CheckedTermReadiness,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CheckedTermReadiness {
    Ready,
    Partial,
    NotWellFormed,
}

struct TermFormulaCheckingState<'a> {
    symbols: &'a SymbolEnv,
    binding_env: &'a BindingEnv,
    normalized_types: NormalizedTypeTable,
    terms: CheckedTermTable,
    formulas: CheckedFormulaTable,
    candidate_sets: OpenCandidateSetTable,
    type_entries: TypeTable,
    facts: TypeFactTable,
    diagnostics: TypeDiagnosticTable,
    seen_terms: BTreeSet<TypedSiteRef>,
    seen_formulas: BTreeSet<TypedSiteRef>,
}

impl TermFormulaCheckingState<'_> {
    fn check_term(
        &mut self,
        input: TermInput,
        type_entries_by_site: &BTreeMap<TypedSiteRef, (NormalizedTypeId, TypeStatus)>,
    ) {
        let mut status = TermStatus::Inferred;
        let mut recovery = None;
        let mut deferred = BTreeSet::new();

        if !self.seen_terms.insert(input.site.clone()) {
            recovery = Some(self.diagnostic(
                Some(input.site.clone()),
                input.source_range,
                TypeDiagnosticClass::TypeEntry,
                TypeDiagnosticSeverity::Error,
                "checker.term.duplicate_site",
                DiagnosticRecoveryState::Degraded,
            ));
            status = status.max_partial();
        }

        if self.binding_env.contexts().get(input.context).is_none() {
            recovery = Some(self.diagnostic(
                Some(input.site.clone()),
                input.source_range,
                TypeDiagnosticClass::Context,
                TypeDiagnosticSeverity::Error,
                "checker.term.unknown_context",
                DiagnosticRecoveryState::Degraded,
            ));
            status = status.max_error();
        }

        self.check_term_kind(&input, &mut status, &mut recovery);

        for reason in input.deferred.iter().copied() {
            deferred.insert(reason);
            recovery = Some(self.term_deferred_diagnostic(&input, reason));
            status = status.max_partial();
        }

        if input.kind == TermKind::Unsupported {
            recovery = Some(self.diagnostic(
                Some(input.site.clone()),
                input.source_range,
                TypeDiagnosticClass::Recovery,
                TypeDiagnosticSeverity::Error,
                "checker.term.unsupported_payload",
                DiagnosticRecoveryState::Degraded,
            ));
            status = status.skip();
        }

        let expected_type = input.expected_type.as_ref().and_then(|expected| {
            self.normalized_type_for_site(
                &input.site,
                expected,
                type_entries_by_site,
                "checker.term.missing_expected_type",
                &mut status,
                &mut recovery,
            )
        });

        let candidate_set = if input.candidates.is_empty() {
            None
        } else {
            let (id, candidate_status) = self.build_candidate_set(
                input.site.clone(),
                candidate_kind_for_term(input.kind),
                input.source_range,
                input.candidates,
                type_entries_by_site,
            );
            if candidate_status != CandidateSetStatus::Open || status == TermStatus::Inferred {
                status = status.max_partial();
            }
            Some(id)
        };

        let (actual, normalized_status) = if let Some(candidate_set) = candidate_set {
            (
                TypeEntryActual::CandidateSet(candidate_set),
                TypeStatus::Unknown,
            )
        } else if let Some(result_type) = &input.result_type {
            match type_entries_by_site.get(&result_type.site) {
                Some((id, type_status)) => (TypeEntryActual::Known(*id), *type_status),
                None => {
                    recovery = Some(self.diagnostic(
                        Some(input.site.clone()),
                        input.source_range,
                        TypeDiagnosticClass::TypeEntry,
                        TypeDiagnosticSeverity::Error,
                        "checker.term.missing_normalized_result_type",
                        DiagnosticRecoveryState::Degraded,
                    ));
                    status = status.max_partial();
                    (TypeEntryActual::Absent, TypeStatus::Unknown)
                }
            }
        } else {
            (TypeEntryActual::Absent, TypeStatus::Unknown)
        };

        let type_status = term_type_status(status, normalized_status, &actual, &deferred);
        let type_entry = self.type_entries.insert(TypeEntryDraft {
            owner: input.site.clone(),
            expected: expected_type,
            actual,
            status: type_status,
            provenance: term_provenance(input.kind, recovery),
        });

        self.terms.insert(CheckedTermDraft {
            site: input.site,
            context: input.context,
            kind: input.kind,
            reference: input.reference,
            type_entry,
            expected_type,
            candidate_set,
            status,
            deferred: deferred.into_iter().collect(),
        });
    }

    fn check_formula(
        &mut self,
        input: FormulaInput,
        type_entries_by_site: &BTreeMap<TypedSiteRef, (NormalizedTypeId, TypeStatus)>,
    ) {
        let mut status = FormulaStatus::Checked;
        let mut recovery = None;
        let mut deferred = BTreeSet::new();

        if !self.seen_formulas.insert(input.site.clone()) {
            recovery = Some(self.diagnostic(
                Some(input.site.clone()),
                input.source_range,
                TypeDiagnosticClass::TypeEntry,
                TypeDiagnosticSeverity::Error,
                "checker.formula.duplicate_site",
                DiagnosticRecoveryState::Degraded,
            ));
            status = status.max_partial();
        }

        if self.binding_env.contexts().get(input.context).is_none() {
            recovery = Some(self.diagnostic(
                Some(input.site.clone()),
                input.source_range,
                TypeDiagnosticClass::Context,
                TypeDiagnosticSeverity::Error,
                "checker.formula.unknown_context",
                DiagnosticRecoveryState::Degraded,
            ));
            status = status.max_error();
        }

        if input.kind == FormulaKind::Unsupported {
            recovery = Some(self.diagnostic(
                Some(input.site.clone()),
                input.source_range,
                TypeDiagnosticClass::Recovery,
                TypeDiagnosticSeverity::Error,
                "checker.formula.unsupported_payload",
                DiagnosticRecoveryState::Degraded,
            ));
            status = status.skip();
        }

        self.check_formula_terms(&input, &mut status, &mut recovery);

        for reason in input.deferred.iter().copied() {
            deferred.insert(reason);
            recovery = Some(self.formula_deferred_diagnostic(&input, reason));
            status = status.max_partial();
        }

        let asserted_type = input.asserted_type.as_ref().and_then(|asserted| {
            self.normalized_type_for_formula(
                &input.site,
                asserted,
                type_entries_by_site,
                "checker.formula.missing_asserted_type",
                &mut status,
                &mut recovery,
            )
        });

        let expected_types = input
            .expected_types
            .iter()
            .filter_map(|expected| {
                self.expected_type_constraint(
                    &input.site,
                    expected,
                    type_entries_by_site,
                    &mut status,
                    &mut recovery,
                )
            })
            .collect::<Vec<_>>();

        if input.kind == FormulaKind::PredicateApplication
            && input.candidates.is_empty()
            && !input
                .deferred
                .contains(&FormulaDeferredReason::MissingPredicateSignaturePayload)
        {
            recovery = Some(self.diagnostic(
                Some(input.site.clone()),
                input.source_range,
                TypeDiagnosticClass::TypeEntry,
                TypeDiagnosticSeverity::Note,
                "checker.formula.external.predicate_signature_payload",
                DiagnosticRecoveryState::Degraded,
            ));
            status = status.max_partial();
        }

        let candidate_set = if input.candidates.is_empty() {
            None
        } else {
            let (id, candidate_status) = self.build_candidate_set(
                input.site.clone(),
                candidate_kind_for_formula(input.kind),
                input.source_range,
                input.candidates,
                type_entries_by_site,
            );
            if candidate_status != CandidateSetStatus::Open || status == FormulaStatus::Checked {
                status = status.max_partial();
            }
            Some(id)
        };

        let mut facts = Vec::new();
        let fact_status = if status == FormulaStatus::Checked {
            FactStatus::Known
        } else {
            FactStatus::Degraded
        };
        for fact in &input.facts {
            facts.push(self.facts.insert(TypeFactDraft {
                subject: fact.subject.clone(),
                predicate: fact.predicate.clone(),
                polarity: fact.polarity,
                provenance: FactProvenance::Inferred(TypeRuleId::new(format!(
                    "formula-{}",
                    formula_kind_name(input.kind)
                ))),
                status: fact_status,
            }));
        }

        self.formulas.insert(CheckedFormulaDraft {
            site: input.site,
            context: input.context,
            kind: input.kind,
            terms: input.terms,
            asserted_type,
            expected_types,
            candidate_set,
            facts,
            status,
            deferred: deferred.into_iter().collect(),
        });
        let _ = recovery;
    }

    fn check_formula_terms(
        &mut self,
        input: &FormulaInput,
        status: &mut FormulaStatus,
        recovery: &mut Option<TypeDiagnosticId>,
    ) {
        for term in &input.terms {
            match self.checked_term_state(term) {
                Some(term_state) => {
                    if !self.context_can_see_term(input.context, term_state.context) {
                        *recovery = Some(self.diagnostic(
                            Some(input.site.clone()),
                            input.source_range,
                            TypeDiagnosticClass::Context,
                            TypeDiagnosticSeverity::Error,
                            "checker.formula.term.context_not_visible",
                            DiagnosticRecoveryState::Degraded,
                        ));
                        *status = status.max_error();
                    }
                    match term_state.readiness {
                        CheckedTermReadiness::Ready => {}
                        CheckedTermReadiness::Partial => {
                            *recovery = Some(self.diagnostic(
                                Some(input.site.clone()),
                                input.source_range,
                                TypeDiagnosticClass::TypeEntry,
                                TypeDiagnosticSeverity::Note,
                                "checker.formula.term.partial",
                                DiagnosticRecoveryState::Degraded,
                            ));
                            *status = status.max_partial();
                        }
                        CheckedTermReadiness::NotWellFormed => {
                            *recovery = Some(self.diagnostic(
                                Some(input.site.clone()),
                                input.source_range,
                                TypeDiagnosticClass::TypeEntry,
                                TypeDiagnosticSeverity::Error,
                                "checker.formula.term.not_well_formed",
                                DiagnosticRecoveryState::Degraded,
                            ));
                            *status = status.max_error();
                        }
                    }
                }
                None => {
                    *recovery = Some(self.diagnostic(
                        Some(input.site.clone()),
                        input.source_range,
                        TypeDiagnosticClass::TypeEntry,
                        TypeDiagnosticSeverity::Error,
                        "checker.formula.term.missing",
                        DiagnosticRecoveryState::Degraded,
                    ));
                    *status = status.max_error();
                }
            }
        }
    }

    fn checked_term_state(&self, site: &TypedSiteRef) -> Option<CheckedTermState> {
        let term = self
            .terms
            .iter()
            .map(|(_, term)| term)
            .find(|term| &term.site == site)?;
        let type_entry = self.type_entries.get(term.type_entry);
        let readiness = match (term.status, type_entry.map(|entry| entry.status)) {
            (TermStatus::Inferred, Some(TypeStatus::Known | TypeStatus::Assumed)) => {
                CheckedTermReadiness::Ready
            }
            (TermStatus::Inferred | TermStatus::Partial, Some(TypeStatus::Unknown)) => {
                CheckedTermReadiness::Partial
            }
            (TermStatus::Inferred | TermStatus::Partial, Some(TypeStatus::Error)) => {
                CheckedTermReadiness::NotWellFormed
            }
            (TermStatus::Inferred | TermStatus::Partial, Some(TypeStatus::Skipped) | None) => {
                CheckedTermReadiness::NotWellFormed
            }
            (TermStatus::Partial, Some(TypeStatus::Known | TypeStatus::Assumed)) => {
                CheckedTermReadiness::Partial
            }
            (TermStatus::Error | TermStatus::Skipped, _) => CheckedTermReadiness::NotWellFormed,
        };
        Some(CheckedTermState {
            context: term.context,
            readiness,
        })
    }

    fn context_can_see_term(
        &self,
        formula_context: BindingContextId,
        term_context: BindingContextId,
    ) -> bool {
        let mut cursor = Some(formula_context);
        while let Some(context) = cursor {
            if context == term_context {
                return true;
            }
            cursor = self
                .binding_env
                .contexts()
                .get(context)
                .and_then(|context| context.parent);
        }
        false
    }

    fn check_term_kind(
        &mut self,
        input: &TermInput,
        status: &mut TermStatus,
        recovery: &mut Option<TypeDiagnosticId>,
    ) {
        match input.kind {
            TermKind::Variable => match &input.reference {
                Some(TermReference::Binding(binding)) => {
                    self.check_term_binding_reference(input, *binding, status, recovery);
                }
                Some(TermReference::Symbol(symbol)) => {
                    if self.symbols.symbols().get(symbol).is_none() {
                        *recovery = Some(self.diagnostic(
                            Some(input.site.clone()),
                            input.source_range,
                            TypeDiagnosticClass::TypeEntry,
                            TypeDiagnosticSeverity::Error,
                            "checker.term.reference.unknown_symbol",
                            DiagnosticRecoveryState::Degraded,
                        ));
                        *status = status.max_error();
                    }
                }
                None => {
                    *recovery = Some(self.diagnostic(
                        Some(input.site.clone()),
                        input.source_range,
                        TypeDiagnosticClass::TypeEntry,
                        TypeDiagnosticSeverity::Note,
                        "checker.term.external.reference_payload",
                        DiagnosticRecoveryState::Degraded,
                    ));
                    *status = status.max_partial();
                }
            },
            TermKind::It if input.result_type.is_none() => {
                *recovery = Some(self.diagnostic(
                    Some(input.site.clone()),
                    input.source_range,
                    TypeDiagnosticClass::TypeEntry,
                    TypeDiagnosticSeverity::Error,
                    "checker.term.it.missing_current_result_type",
                    DiagnosticRecoveryState::Degraded,
                ));
                *status = status.max_error();
            }
            TermKind::Numeral if input.result_type.is_none() => {
                *recovery = Some(self.diagnostic(
                    Some(input.site.clone()),
                    input.source_range,
                    TypeDiagnosticClass::TypeEntry,
                    TypeDiagnosticSeverity::Note,
                    "checker.term.external.numeric_type_payload",
                    DiagnosticRecoveryState::Degraded,
                ));
                *status = status.max_partial();
            }
            TermKind::FunctorApplication
            | TermKind::SelectorAccess
            | TermKind::StructureConstructor
                if input.result_type.is_none() && input.candidates.is_empty() =>
            {
                *recovery = Some(self.diagnostic(
                    Some(input.site.clone()),
                    input.source_range,
                    TypeDiagnosticClass::TypeEntry,
                    TypeDiagnosticSeverity::Note,
                    "checker.term.external.signature_payload",
                    DiagnosticRecoveryState::Degraded,
                ));
                *status = status.max_partial();
            }
            TermKind::Parenthesized
            | TermKind::SetEnumeration
            | TermKind::SetComprehension
            | TermKind::Choice
            | TermKind::SourceQua
                if input.result_type.is_none() && input.candidates.is_empty() =>
            {
                *recovery = Some(self.diagnostic(
                    Some(input.site.clone()),
                    input.source_range,
                    TypeDiagnosticClass::TypeEntry,
                    TypeDiagnosticSeverity::Note,
                    "checker.term.external.result_type_payload",
                    DiagnosticRecoveryState::Degraded,
                ));
                *status = status.max_partial();
            }
            _ => {}
        }
    }

    fn check_term_binding_reference(
        &mut self,
        input: &TermInput,
        binding: BindingId,
        status: &mut TermStatus,
        recovery: &mut Option<TypeDiagnosticId>,
    ) {
        if self.binding_env.bindings().get(binding).is_none() {
            *recovery = Some(self.diagnostic(
                Some(input.site.clone()),
                input.source_range,
                TypeDiagnosticClass::TypeEntry,
                TypeDiagnosticSeverity::Error,
                "checker.term.reference.unknown_binding",
                DiagnosticRecoveryState::Degraded,
            ));
            *status = status.max_error();
            return;
        }
        let is_visible = self
            .binding_env
            .contexts()
            .get(input.context)
            .is_some_and(|context| context.visible_bindings.contains(&binding));
        if !is_visible {
            *recovery = Some(self.diagnostic(
                Some(input.site.clone()),
                input.source_range,
                TypeDiagnosticClass::Context,
                TypeDiagnosticSeverity::Error,
                "checker.term.reference.binding_not_visible",
                DiagnosticRecoveryState::Degraded,
            ));
            *status = status.max_error();
        }
    }

    fn normalized_type_for_site(
        &mut self,
        owner: &TypedSiteRef,
        input: &TypeExpressionInput,
        type_entries_by_site: &BTreeMap<TypedSiteRef, (NormalizedTypeId, TypeStatus)>,
        message_key: &str,
        status: &mut TermStatus,
        recovery: &mut Option<TypeDiagnosticId>,
    ) -> Option<NormalizedTypeId> {
        match type_entries_by_site.get(&input.site) {
            Some((id, type_status)) => {
                if *type_status != TypeStatus::Known {
                    *status = status.max_partial();
                }
                Some(*id)
            }
            None => {
                *recovery = Some(self.diagnostic(
                    Some(owner.clone()),
                    input.source_range,
                    TypeDiagnosticClass::TypeEntry,
                    TypeDiagnosticSeverity::Error,
                    message_key,
                    DiagnosticRecoveryState::Degraded,
                ));
                *status = status.max_partial();
                None
            }
        }
    }

    fn normalized_type_for_formula(
        &mut self,
        owner: &TypedSiteRef,
        input: &TypeExpressionInput,
        type_entries_by_site: &BTreeMap<TypedSiteRef, (NormalizedTypeId, TypeStatus)>,
        message_key: &str,
        status: &mut FormulaStatus,
        recovery: &mut Option<TypeDiagnosticId>,
    ) -> Option<NormalizedTypeId> {
        match type_entries_by_site.get(&input.site) {
            Some((id, type_status)) => {
                if *type_status != TypeStatus::Known {
                    *status = status.max_partial();
                }
                Some(*id)
            }
            None => {
                *recovery = Some(self.diagnostic(
                    Some(owner.clone()),
                    input.source_range,
                    TypeDiagnosticClass::TypeEntry,
                    TypeDiagnosticSeverity::Error,
                    message_key,
                    DiagnosticRecoveryState::Degraded,
                ));
                *status = status.max_partial();
                None
            }
        }
    }

    fn expected_type_constraint(
        &mut self,
        owner: &TypedSiteRef,
        input: &ExpectedTypeInput,
        type_entries_by_site: &BTreeMap<TypedSiteRef, (NormalizedTypeId, TypeStatus)>,
        status: &mut FormulaStatus,
        recovery: &mut Option<TypeDiagnosticId>,
    ) -> Option<ExpectedTypeConstraint> {
        match type_entries_by_site.get(&input.expected.site) {
            Some((id, type_status)) => {
                if *type_status != TypeStatus::Known {
                    *status = status.max_partial();
                }
                Some(ExpectedTypeConstraint {
                    term: input.term.clone(),
                    expected: *id,
                    status: *type_status,
                    source_range: input.source_range,
                })
            }
            None => {
                *recovery = Some(self.diagnostic(
                    Some(owner.clone()),
                    input.source_range,
                    TypeDiagnosticClass::TypeEntry,
                    TypeDiagnosticSeverity::Error,
                    "checker.formula.missing_expected_type",
                    DiagnosticRecoveryState::Degraded,
                ));
                *status = status.max_partial();
                None
            }
        }
    }

    fn build_candidate_set(
        &mut self,
        owner: TypedSiteRef,
        kind: CandidateSetKind,
        source_range: SourceRange,
        candidates: Vec<OpenCandidateInput>,
        type_entries_by_site: &BTreeMap<TypedSiteRef, (NormalizedTypeId, TypeStatus)>,
    ) -> (OpenCandidateSetId, CandidateSetStatus) {
        let mut set_status = CandidateSetStatus::Open;
        let mut open_candidates = Vec::new();
        for candidate in candidates {
            let mut candidate_status = CandidateStatus::Viable;
            let result_type = candidate.result_type.as_ref().and_then(|result_type| {
                match type_entries_by_site.get(&result_type.site) {
                    Some((id, status)) => {
                        if *status != TypeStatus::Known {
                            candidate_status = CandidateStatus::Degraded;
                            set_status = CandidateSetStatus::Degraded;
                        }
                        Some(*id)
                    }
                    None => {
                        candidate_status = CandidateStatus::Degraded;
                        set_status = CandidateSetStatus::Degraded;
                        self.diagnostic(
                            Some(owner.clone()),
                            result_type.source_range,
                            TypeDiagnosticClass::TypeEntry,
                            TypeDiagnosticSeverity::Error,
                            "checker.candidate.missing_result_type",
                            DiagnosticRecoveryState::Degraded,
                        );
                        None
                    }
                }
            });
            let mut required_types = Vec::new();
            for required in &candidate.required_types {
                match type_entries_by_site.get(&required.site) {
                    Some((id, status)) => {
                        if *status != TypeStatus::Known {
                            candidate_status = CandidateStatus::Degraded;
                            set_status = CandidateSetStatus::Degraded;
                        }
                        required_types.push(*id);
                    }
                    None => {
                        candidate_status = CandidateStatus::Degraded;
                        set_status = CandidateSetStatus::Degraded;
                        self.diagnostic(
                            Some(owner.clone()),
                            required.source_range,
                            TypeDiagnosticClass::TypeEntry,
                            TypeDiagnosticSeverity::Error,
                            "checker.candidate.missing_required_type",
                            DiagnosticRecoveryState::Degraded,
                        );
                    }
                }
            }
            open_candidates.push(OpenCandidate {
                identity: candidate.identity,
                result_type,
                required_types,
                source_range: candidate.source_range,
                status: candidate_status,
            });
        }
        let id = self.candidate_sets.insert(OpenCandidateSetDraft {
            owner,
            kind,
            candidates: open_candidates,
            status: set_status,
            source_range,
        });
        (id, set_status)
    }

    fn term_deferred_diagnostic(
        &mut self,
        input: &TermInput,
        reason: TermDeferredReason,
    ) -> TypeDiagnosticId {
        self.diagnostic(
            Some(input.site.clone()),
            input.source_range,
            TypeDiagnosticClass::TypeEntry,
            TypeDiagnosticSeverity::Note,
            term_deferred_message_key(reason),
            DiagnosticRecoveryState::Recovery,
        )
    }

    fn formula_deferred_diagnostic(
        &mut self,
        input: &FormulaInput,
        reason: FormulaDeferredReason,
    ) -> TypeDiagnosticId {
        self.diagnostic(
            Some(input.site.clone()),
            input.source_range,
            TypeDiagnosticClass::TypeFact,
            TypeDiagnosticSeverity::Note,
            formula_deferred_message_key(reason),
            DiagnosticRecoveryState::Recovery,
        )
    }

    fn diagnostic(
        &mut self,
        owner: Option<TypedSiteRef>,
        source_range: SourceRange,
        class: TypeDiagnosticClass,
        severity: TypeDiagnosticSeverity,
        message_key: &str,
        recovery: DiagnosticRecoveryState,
    ) -> TypeDiagnosticId {
        self.diagnostics.insert(TypeDiagnosticDraft {
            owner,
            source_range,
            class,
            severity,
            message_key: message_key.to_owned(),
            recovery,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclarationContextInput {
    pub binding_context: BindingContextId,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
}

impl DeclarationContextInput {
    pub fn new(
        binding_context: BindingContextId,
        site: TypedSiteRef,
        source_range: SourceRange,
    ) -> Self {
        Self {
            binding_context,
            site,
            source_range,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclarationInput {
    pub binding: BindingId,
    pub context: BindingContextId,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub kind: DeclarationKind,
    pub type_expression: Option<TypeExpressionInput>,
    pub reserved_default: Option<ReservedDefaultPayload>,
    pub deferred: Vec<DeclarationDeferredReason>,
    pub assumptions: Vec<DeclarationAssumptionInput>,
}

impl DeclarationInput {
    pub fn new(
        binding: BindingId,
        context: BindingContextId,
        site: TypedSiteRef,
        source_range: SourceRange,
        kind: DeclarationKind,
    ) -> Self {
        Self {
            binding,
            context,
            site,
            source_range,
            kind,
            type_expression: None,
            reserved_default: None,
            deferred: Vec::new(),
            assumptions: Vec::new(),
        }
    }

    pub fn with_type_expression(mut self, type_expression: TypeExpressionInput) -> Self {
        self.type_expression = Some(type_expression);
        self
    }

    pub fn with_reserved_default(mut self, reserved_default: ReservedDefaultPayload) -> Self {
        self.reserved_default = Some(reserved_default);
        self
    }

    pub fn with_deferred(mut self, deferred: Vec<DeclarationDeferredReason>) -> Self {
        self.deferred = deferred;
        self
    }

    pub fn with_assumptions(mut self, assumptions: Vec<DeclarationAssumptionInput>) -> Self {
        self.assumptions = assumptions;
        self
    }

    fn uses_explicit_type_payload(&self) -> bool {
        if self.kind == DeclarationKind::ReservedVariable
            && self
                .reserved_default
                .as_ref()
                .is_some_and(|payload| payload.shadowed_by_local)
        {
            return false;
        }
        self.type_expression.is_some()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum DeclarationKind {
    Let,
    ReservedVariable,
    QuantifiedVariable,
    Given,
    Consider,
    Take,
    Set,
    DefinitionParameter,
    DefFuncFormal,
    DefPredFormal,
    ReconsiderExisting,
    ReconsiderNew,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReservedDefaultPayload {
    pub type_site: TypedSiteRef,
    pub shadowed_by_local: bool,
}

impl ReservedDefaultPayload {
    pub const fn new(type_site: TypedSiteRef, shadowed_by_local: bool) -> Self {
        Self {
            type_site,
            shadowed_by_local,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum DeclarationDeferredReason {
    MissingTypePayload,
    MissingReservedDefaultPayload,
    MissingRightHandSidePayload,
    MissingDefinitionBodyPayload,
    MissingEvidenceQuery,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclarationAssumptionInput {
    pub predicate: TypePredicateRef,
    pub source_range: SourceRange,
}

impl DeclarationAssumptionInput {
    pub fn new(predicate: TypePredicateRef, source_range: SourceRange) -> Self {
        Self {
            predicate,
            source_range,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CheckedDeclarationTable {
    entries: Vec<CheckedDeclaration>,
}

impl CheckedDeclarationTable {
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    fn insert(&mut self, draft: CheckedDeclarationDraft) -> CheckedDeclarationId {
        let id = CheckedDeclarationId::new(self.entries.len());
        self.entries.push(CheckedDeclaration {
            id,
            binding: draft.binding,
            context: draft.context,
            site: draft.site,
            kind: draft.kind,
            type_entry: draft.type_entry,
            type_site: draft.type_site,
            facts: draft.facts,
            status: draft.status,
            deferred: draft.deferred,
        });
        id
    }

    pub fn get(&self, id: CheckedDeclarationId) -> Option<&CheckedDeclaration> {
        self.entries.get(id.index())
    }

    pub fn iter(&self) -> impl Iterator<Item = (CheckedDeclarationId, &CheckedDeclaration)> {
        self.entries.iter().map(|entry| (entry.id, entry))
    }

    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CheckedDeclarationId(usize);

impl CheckedDeclarationId {
    pub const fn new(index: usize) -> Self {
        Self(index)
    }

    pub const fn index(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckedDeclaration {
    pub id: CheckedDeclarationId,
    pub binding: BindingId,
    pub context: BindingContextId,
    pub site: TypedSiteRef,
    pub kind: DeclarationKind,
    pub type_entry: Option<TypeEntryId>,
    pub type_site: Option<TypedSiteRef>,
    pub facts: Vec<TypeFactId>,
    pub status: DeclarationStatus,
    pub deferred: Vec<DeclarationDeferredReason>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CheckedDeclarationDraft {
    binding: BindingId,
    context: BindingContextId,
    site: TypedSiteRef,
    kind: DeclarationKind,
    type_entry: Option<TypeEntryId>,
    type_site: Option<TypedSiteRef>,
    facts: Vec<TypeFactId>,
    status: DeclarationStatus,
    deferred: Vec<DeclarationDeferredReason>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum DeclarationStatus {
    Checked,
    Partial,
    Error,
}

struct DeclarationCheckingState<'a> {
    binding_env: &'a BindingEnv,
    normalized_types: NormalizedTypeTable,
    declarations: CheckedDeclarationTable,
    type_entries: TypeTable,
    facts: TypeFactTable,
    diagnostics: TypeDiagnosticTable,
    seen_sites: BTreeSet<TypedSiteRef>,
    seen_bindings: BTreeSet<BindingId>,
    context_bindings: BTreeMap<BindingContextId, BTreeSet<BindingTypeRef>>,
    context_facts: BTreeMap<BindingContextId, BTreeSet<TypeFactId>>,
}

impl DeclarationCheckingState<'_> {
    fn check_declaration(
        &mut self,
        input: DeclarationInput,
        type_entries_by_site: &BTreeMap<TypedSiteRef, (NormalizedTypeId, TypeStatus)>,
    ) {
        let mut status = DeclarationStatus::Checked;
        let mut recovery = None;
        let mut type_entry = None;
        let mut type_site = None;
        let mut deferred = BTreeSet::new();
        let mut facts = Vec::new();

        if !self.seen_sites.insert(input.site.clone()) {
            recovery = Some(self.diagnostic(
                Some(input.site.clone()),
                input.source_range,
                TypeDiagnosticClass::TypeEntry,
                TypeDiagnosticSeverity::Error,
                "checker.declaration.duplicate_site",
                DiagnosticRecoveryState::Degraded,
            ));
            status = DeclarationStatus::Partial;
        }

        if input.kind != DeclarationKind::ReconsiderExisting
            && !self.seen_bindings.insert(input.binding)
        {
            recovery = Some(self.diagnostic(
                Some(input.site.clone()),
                input.source_range,
                TypeDiagnosticClass::TypeEntry,
                TypeDiagnosticSeverity::Error,
                "checker.declaration.duplicate_binding",
                DiagnosticRecoveryState::Degraded,
            ));
            status = DeclarationStatus::Partial;
        }

        match self.binding_env.bindings().get(input.binding) {
            Some(binding) => {
                if !binding_context_matches_declaration(
                    self.binding_env,
                    input.binding,
                    binding.owner_context,
                    input.context,
                    input.kind,
                ) {
                    recovery = Some(self.diagnostic(
                        Some(input.site.clone()),
                        input.source_range,
                        TypeDiagnosticClass::Context,
                        TypeDiagnosticSeverity::Error,
                        "checker.declaration.binding_context_mismatch",
                        DiagnosticRecoveryState::Degraded,
                    ));
                    status = DeclarationStatus::Partial;
                }
                if !binding_kind_matches(input.kind, binding.kind) {
                    recovery = Some(self.diagnostic(
                        Some(input.site.clone()),
                        binding.declaration_range,
                        TypeDiagnosticClass::TypeEntry,
                        TypeDiagnosticSeverity::Error,
                        "checker.declaration.illegal_binding_kind",
                        DiagnosticRecoveryState::Degraded,
                    ));
                    status = DeclarationStatus::Partial;
                }
                if binding.status == BindingStatus::Omitted {
                    recovery = Some(self.diagnostic(
                        Some(input.site.clone()),
                        binding.declaration_range,
                        TypeDiagnosticClass::TypeEntry,
                        TypeDiagnosticSeverity::Error,
                        "checker.declaration.omitted_binding",
                        DiagnosticRecoveryState::Degraded,
                    ));
                    status = DeclarationStatus::Partial;
                }
            }
            None => {
                recovery = Some(self.diagnostic(
                    Some(input.site.clone()),
                    input.source_range,
                    TypeDiagnosticClass::TypeEntry,
                    TypeDiagnosticSeverity::Error,
                    "checker.declaration.unknown_binding",
                    DiagnosticRecoveryState::Degraded,
                ));
                status = DeclarationStatus::Error;
            }
        }

        if self.binding_env.contexts().get(input.context).is_none() {
            recovery = Some(self.diagnostic(
                Some(input.site.clone()),
                input.source_range,
                TypeDiagnosticClass::Context,
                TypeDiagnosticSeverity::Error,
                "checker.declaration.unknown_context",
                DiagnosticRecoveryState::Degraded,
            ));
            status = DeclarationStatus::Error;
        }

        for reason in input.deferred.iter().copied() {
            deferred.insert(reason);
            recovery = Some(self.deferred_diagnostic(&input, reason));
            status = status.max_partial();
        }

        let reserved_shadowed = input.kind == DeclarationKind::ReservedVariable
            && input
                .reserved_default
                .as_ref()
                .is_some_and(|payload| payload.shadowed_by_local);
        let reserved_default_type_site = input
            .reserved_default
            .as_ref()
            .filter(|_| input.kind == DeclarationKind::ReservedVariable)
            .filter(|payload| !payload.shadowed_by_local)
            .map(|payload| payload.type_site.clone());
        if input.kind == DeclarationKind::ReservedVariable {
            match &input.reserved_default {
                Some(payload) if payload.shadowed_by_local => {
                    deferred.insert(DeclarationDeferredReason::MissingReservedDefaultPayload);
                    recovery = Some(self.diagnostic(
                        Some(input.site.clone()),
                        input.source_range,
                        TypeDiagnosticClass::TypeEntry,
                        TypeDiagnosticSeverity::Note,
                        "checker.declaration.reserved_default_shadowed",
                        DiagnosticRecoveryState::Recovery,
                    ));
                    status = status.max_partial();
                }
                None => {
                    deferred.insert(DeclarationDeferredReason::MissingReservedDefaultPayload);
                    recovery = Some(self.deferred_diagnostic(
                        &input,
                        DeclarationDeferredReason::MissingReservedDefaultPayload,
                    ));
                    status = status.max_partial();
                }
                Some(_) => {}
            }
        }

        if let Some(type_expression) = &input.type_expression {
            if !reserved_shadowed {
                let has_reserved_default_mismatch = reserved_default_type_site
                    .as_ref()
                    .is_some_and(|reserved_site| reserved_site != &type_expression.site);
                if has_reserved_default_mismatch {
                    recovery = Some(self.diagnostic(
                        Some(input.site.clone()),
                        input.source_range,
                        TypeDiagnosticClass::TypeEntry,
                        TypeDiagnosticSeverity::Error,
                        "checker.declaration.reserved_default_type_site_mismatch",
                        DiagnosticRecoveryState::Degraded,
                    ));
                    status = status.max_partial();
                }
                let effective_type_site = if has_reserved_default_mismatch {
                    type_expression.site.clone()
                } else {
                    reserved_default_type_site
                        .clone()
                        .unwrap_or_else(|| type_expression.site.clone())
                };
                type_site = Some(effective_type_site.clone());
                match type_entries_by_site.get(&effective_type_site) {
                    Some((actual, type_status)) => {
                        let entry_status =
                            declaration_entry_status(status, *type_status, recovery, &deferred);
                        if entry_status != TypeStatus::Known {
                            status = status.max_partial();
                        }
                        type_entry = Some(self.type_entries.insert(TypeEntryDraft {
                            owner: input.site.clone(),
                            expected: None,
                            actual: TypeEntryActual::Known(*actual),
                            status: entry_status,
                            provenance: declaration_provenance(input.source_range, recovery),
                        }));
                    }
                    None => {
                        recovery = Some(self.diagnostic(
                            Some(input.site.clone()),
                            input.source_range,
                            TypeDiagnosticClass::TypeEntry,
                            TypeDiagnosticSeverity::Error,
                            "checker.declaration.missing_normalized_type",
                            DiagnosticRecoveryState::Degraded,
                        ));
                        status = DeclarationStatus::Partial;
                    }
                }
            }
        } else if should_defer_missing_type_payload(&input, reserved_shadowed) {
            deferred.insert(DeclarationDeferredReason::MissingTypePayload);
            recovery = Some(
                self.deferred_diagnostic(&input, DeclarationDeferredReason::MissingTypePayload),
            );
            status = status.max_partial();
        }

        if type_entry.is_none() {
            type_entry = Some(self.type_entries.insert(TypeEntryDraft {
                owner: input.site.clone(),
                expected: None,
                actual: TypeEntryActual::Absent,
                status: if status == DeclarationStatus::Error {
                    TypeStatus::Error
                } else {
                    TypeStatus::Unknown
                },
                provenance: declaration_provenance(input.source_range, recovery),
            }));
        }

        if status != DeclarationStatus::Checked && !input.assumptions.is_empty() {
            self.diagnostic(
                Some(input.site.clone()),
                input.source_range,
                TypeDiagnosticClass::TypeFact,
                TypeDiagnosticSeverity::Error,
                "checker.declaration.assumption_dropped_after_recovery",
                DiagnosticRecoveryState::Degraded,
            );
        } else {
            for assumption in &input.assumptions {
                let fact = self.facts.insert(TypeFactDraft {
                    subject: input.site.clone(),
                    predicate: assumption.predicate.clone(),
                    polarity: Polarity::Positive,
                    provenance: FactProvenance::Assumed(TypeAssumptionId::new(format!(
                        "declaration:{}",
                        declaration_kind_name(input.kind)
                    ))),
                    status: FactStatus::Assumed,
                });
                facts.push(fact);
                self.context_facts
                    .entry(input.context)
                    .or_default()
                    .insert(fact);
            }
        }

        if status != DeclarationStatus::Error {
            self.context_bindings
                .entry(input.context)
                .or_default()
                .insert(BindingTypeRef::Site(input.site.clone()));
        }

        self.declarations.insert(CheckedDeclarationDraft {
            binding: input.binding,
            context: input.context,
            site: input.site,
            kind: input.kind,
            type_entry,
            type_site,
            facts,
            status,
            deferred: deferred.into_iter().collect(),
        });
    }

    fn deferred_diagnostic(
        &mut self,
        input: &DeclarationInput,
        reason: DeclarationDeferredReason,
    ) -> TypeDiagnosticId {
        self.diagnostic(
            Some(input.site.clone()),
            input.source_range,
            TypeDiagnosticClass::TypeEntry,
            TypeDiagnosticSeverity::Note,
            deferred_message_key(reason),
            DiagnosticRecoveryState::Recovery,
        )
    }

    fn diagnostic(
        &mut self,
        owner: Option<TypedSiteRef>,
        source_range: SourceRange,
        class: TypeDiagnosticClass,
        severity: TypeDiagnosticSeverity,
        message_key: &str,
        recovery: DiagnosticRecoveryState,
    ) -> TypeDiagnosticId {
        self.diagnostics.insert(TypeDiagnosticDraft {
            owner,
            source_range,
            class,
            severity,
            message_key: message_key.to_owned(),
            recovery,
        })
    }
}

impl DeclarationStatus {
    fn max_partial(self) -> Self {
        match self {
            Self::Checked => Self::Partial,
            Self::Partial | Self::Error => self,
        }
    }
}

fn term_input_key(input: &TermInput) -> (String, TermKind) {
    (site_key(&input.site), input.kind)
}

fn formula_input_key(input: &FormulaInput) -> (String, FormulaKind) {
    (site_key(&input.site), input.kind)
}

fn declaration_context_input_key(input: &DeclarationContextInput) -> (usize, String) {
    (input.binding_context.index(), site_key(&input.site))
}

fn declaration_input_key(input: &DeclarationInput) -> (usize, String, DeclarationKind) {
    (input.binding.index(), site_key(&input.site), input.kind)
}

fn type_entries_by_site(
    entries: &TypeTable,
) -> BTreeMap<TypedSiteRef, (NormalizedTypeId, TypeStatus)> {
    entries
        .iter()
        .filter_map(|(_, entry)| match entry.actual {
            TypeEntryActual::Known(id) => Some((entry.owner.clone(), (id, entry.status))),
            TypeEntryActual::CandidateSet(_) | TypeEntryActual::Absent => None,
        })
        .collect()
}

fn declaration_provenance(
    source_range: SourceRange,
    recovery: Option<TypeDiagnosticId>,
) -> TypeProvenance {
    match recovery {
        Some(diagnostic) => TypeProvenance::Recovery(diagnostic),
        None => TypeProvenance::Declared(TypedSourceRangeKey::from(source_range)),
    }
}

fn term_provenance(kind: TermKind, recovery: Option<TypeDiagnosticId>) -> TypeProvenance {
    match recovery {
        Some(diagnostic) => TypeProvenance::Recovery(diagnostic),
        None => TypeProvenance::Inferred(TypeRuleId::new(format!("term-{}", term_kind_name(kind)))),
    }
}

fn term_type_status(
    status: TermStatus,
    normalized_status: TypeStatus,
    actual: &TypeEntryActual,
    deferred: &BTreeSet<TermDeferredReason>,
) -> TypeStatus {
    match status {
        TermStatus::Error => TypeStatus::Error,
        TermStatus::Skipped => TypeStatus::Skipped,
        TermStatus::Partial => TypeStatus::Unknown,
        TermStatus::Inferred => {
            if normalized_status == TypeStatus::Error {
                TypeStatus::Error
            } else if !deferred.is_empty()
                || normalized_status != TypeStatus::Known
                || !matches!(actual, TypeEntryActual::Known(_))
            {
                TypeStatus::Unknown
            } else {
                TypeStatus::Known
            }
        }
    }
}

fn candidate_kind_for_term(kind: TermKind) -> CandidateSetKind {
    match kind {
        TermKind::SelectorAccess => CandidateSetKind::Selector,
        _ => CandidateSetKind::Functor,
    }
}

fn candidate_kind_for_formula(_kind: FormulaKind) -> CandidateSetKind {
    CandidateSetKind::Predicate
}

fn should_defer_missing_type_payload(input: &DeclarationInput, reserved_shadowed: bool) -> bool {
    !(reserved_shadowed
        || input.kind == DeclarationKind::ReservedVariable && input.reserved_default.is_none())
}

fn declaration_entry_status(
    declaration_status: DeclarationStatus,
    normalized_status: TypeStatus,
    recovery: Option<TypeDiagnosticId>,
    deferred: &BTreeSet<DeclarationDeferredReason>,
) -> TypeStatus {
    if declaration_status == DeclarationStatus::Error || normalized_status == TypeStatus::Error {
        TypeStatus::Error
    } else if recovery.is_some() || normalized_status != TypeStatus::Known || !deferred.is_empty() {
        TypeStatus::Unknown
    } else {
        TypeStatus::Known
    }
}

fn binding_kind_matches(kind: DeclarationKind, binding_kind: BindingKind) -> bool {
    match kind {
        DeclarationKind::Let => binding_kind == BindingKind::LetBinding,
        DeclarationKind::ReservedVariable => binding_kind == BindingKind::ReservedVariable,
        DeclarationKind::QuantifiedVariable
        | DeclarationKind::Given
        | DeclarationKind::Consider
        | DeclarationKind::Take => binding_kind == BindingKind::QuantifierBinder,
        DeclarationKind::Set => binding_kind == BindingKind::LocalAbbreviation,
        DeclarationKind::DefinitionParameter
        | DeclarationKind::DefFuncFormal
        | DeclarationKind::DefPredFormal => binding_kind == BindingKind::DefinitionParameter,
        DeclarationKind::ReconsiderExisting => {
            !matches!(binding_kind, BindingKind::ReservedVariable)
        }
        DeclarationKind::ReconsiderNew => {
            matches!(
                binding_kind,
                BindingKind::LetBinding | BindingKind::LocalAbbreviation | BindingKind::Generated
            )
        }
    }
}

fn binding_context_matches_declaration(
    binding_env: &BindingEnv,
    binding: BindingId,
    owner_context: BindingContextId,
    declaration_context: BindingContextId,
    kind: DeclarationKind,
) -> bool {
    owner_context == declaration_context
        || (kind == DeclarationKind::ReconsiderExisting
            && binding_env
                .contexts()
                .get(declaration_context)
                .is_some_and(|context| context.visible_bindings.contains(&binding)))
}

fn build_local_contexts(
    binding_env: &BindingEnv,
    inputs: &[DeclarationContextInput],
    context_bindings: &BTreeMap<BindingContextId, BTreeSet<BindingTypeRef>>,
    context_facts: &BTreeMap<BindingContextId, BTreeSet<TypeFactId>>,
    diagnostics: &mut TypeDiagnosticTable,
) -> LocalTypeContextTable {
    let mut local_contexts = LocalTypeContextTable::new();
    let mut remap = BTreeMap::<BindingContextId, LocalTypeContextId>::new();
    let input_by_context = inputs
        .iter()
        .map(|input| (input.binding_context, input))
        .collect::<BTreeMap<_, _>>();
    let mut remaining = input_by_context.keys().copied().collect::<BTreeSet<_>>();

    while !remaining.is_empty() {
        let mut inserted = Vec::new();
        for binding_context_id in remaining.iter().copied().collect::<Vec<_>>() {
            let Some(input) = input_by_context.get(&binding_context_id).copied() else {
                inserted.push(binding_context_id);
                continue;
            };
            let Some(binding_context) = binding_env.contexts().get(binding_context_id) else {
                diagnostics.insert(TypeDiagnosticDraft {
                    owner: Some(input.site.clone()),
                    source_range: input.source_range,
                    class: TypeDiagnosticClass::Context,
                    severity: TypeDiagnosticSeverity::Error,
                    message_key: "checker.declaration.unknown_context".to_owned(),
                    recovery: DiagnosticRecoveryState::Degraded,
                });
                inserted.push(binding_context_id);
                continue;
            };
            let parent_is_selected = binding_context
                .parent
                .is_some_and(|parent| input_by_context.contains_key(&parent));
            if parent_is_selected
                && !binding_context
                    .parent
                    .is_some_and(|parent| remap.contains_key(&parent))
            {
                continue;
            }

            insert_local_context(
                &mut local_contexts,
                &mut remap,
                binding_context_id,
                input,
                binding_context,
                context_bindings,
                context_facts,
            );
            inserted.push(binding_context_id);
        }

        if inserted.is_empty() {
            for binding_context_id in remaining.iter().copied().collect::<Vec<_>>() {
                let Some(input) = input_by_context.get(&binding_context_id).copied() else {
                    continue;
                };
                diagnostics.insert(TypeDiagnosticDraft {
                    owner: Some(input.site.clone()),
                    source_range: input.source_range,
                    class: TypeDiagnosticClass::Context,
                    severity: TypeDiagnosticSeverity::Error,
                    message_key: "checker.declaration.context_parent_cycle".to_owned(),
                    recovery: DiagnosticRecoveryState::Degraded,
                });
                inserted.push(binding_context_id);
            }
        }

        for binding_context_id in inserted {
            remaining.remove(&binding_context_id);
        }
    }

    local_contexts
}

fn insert_local_context(
    local_contexts: &mut LocalTypeContextTable,
    remap: &mut BTreeMap<BindingContextId, LocalTypeContextId>,
    binding_context_id: BindingContextId,
    input: &DeclarationContextInput,
    binding_context: &crate::binding_env::BindingContext,
    context_bindings: &BTreeMap<BindingContextId, BTreeSet<BindingTypeRef>>,
    context_facts: &BTreeMap<BindingContextId, BTreeSet<TypeFactId>>,
) {
    let parent = binding_context
        .parent
        .and_then(|parent| remap.get(&parent).copied());
    let introduced = context_facts
        .get(&binding_context_id)
        .map(|facts| facts.iter().copied().collect::<Vec<_>>())
        .unwrap_or_default();
    let mut visible = parent
        .and_then(|parent| local_contexts.get(parent))
        .map(|context| {
            context
                .visible_facts
                .iter()
                .copied()
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    visible.extend(introduced.iter().copied());
    let id = local_contexts.insert(LocalTypeContextDraft {
        owner: input.site.clone(),
        parent,
        layer: type_context_layer(binding_context.layer),
        bindings: context_bindings
            .get(&binding_context_id)
            .map(|bindings| bindings.iter().cloned().collect())
            .unwrap_or_default(),
        introduced_assumptions: introduced,
        visible_facts: visible.into_iter().collect(),
        recovery: context_recovery(binding_context.recovery),
    });
    remap.insert(binding_context_id, id);
}

fn type_context_layer(layer: BindingContextLayer) -> TypeContextLayer {
    match layer {
        BindingContextLayer::Module => TypeContextLayer::Module,
        BindingContextLayer::Declaration => TypeContextLayer::Declaration,
        BindingContextLayer::Proof => TypeContextLayer::Proof,
        BindingContextLayer::Block => TypeContextLayer::Block,
        BindingContextLayer::Expression => TypeContextLayer::Expression,
    }
}

fn context_recovery(recovery: BindingContextRecovery) -> ContextRecoveryState {
    match recovery {
        BindingContextRecovery::Normal => ContextRecoveryState::Normal,
        BindingContextRecovery::Recovered => ContextRecoveryState::Recovered,
        BindingContextRecovery::Degraded => ContextRecoveryState::Degraded,
    }
}

fn deferred_message_key(reason: DeclarationDeferredReason) -> &'static str {
    match reason {
        DeclarationDeferredReason::MissingTypePayload => {
            "checker.declaration.deferred.type_payload"
        }
        DeclarationDeferredReason::MissingReservedDefaultPayload => {
            "checker.declaration.deferred.reserved_default_payload"
        }
        DeclarationDeferredReason::MissingRightHandSidePayload => {
            "checker.declaration.deferred.rhs_payload"
        }
        DeclarationDeferredReason::MissingDefinitionBodyPayload => {
            "checker.declaration.deferred.definition_body_payload"
        }
        DeclarationDeferredReason::MissingEvidenceQuery => {
            "checker.declaration.deferred.evidence_query"
        }
    }
}

fn term_deferred_message_key(reason: TermDeferredReason) -> &'static str {
    match reason {
        TermDeferredReason::MissingReferencePayload => "checker.term.external.reference_payload",
        TermDeferredReason::MissingNumericTypePayload => {
            "checker.term.external.numeric_type_payload"
        }
        TermDeferredReason::MissingSignaturePayload => "checker.term.external.signature_payload",
        TermDeferredReason::MissingSelectorPayload => "checker.term.external.selector_payload",
        TermDeferredReason::MissingStructurePayload => "checker.term.external.structure_payload",
        TermDeferredReason::MissingResultTypePayload => "checker.term.external.result_type_payload",
        TermDeferredReason::SethoodRequirement => "checker.term.deferred.sethood_requirement",
        TermDeferredReason::NonEmptinessRequirement => {
            "checker.term.deferred.non_empty_requirement"
        }
        TermDeferredReason::SourceQuaRequirement => "checker.term.deferred.source_qua_requirement",
    }
}

fn formula_deferred_message_key(reason: FormulaDeferredReason) -> &'static str {
    match reason {
        FormulaDeferredReason::MissingPredicateSignaturePayload => {
            "checker.formula.external.predicate_signature_payload"
        }
        FormulaDeferredReason::MissingExpectedTypePayload => {
            "checker.formula.external.expected_type_payload"
        }
        FormulaDeferredReason::MissingQuantifierPayload => {
            "checker.formula.external.quantifier_payload"
        }
        FormulaDeferredReason::MissingFormulaPayload => "checker.formula.external.formula_payload",
    }
}

fn term_kind_name(kind: TermKind) -> &'static str {
    match kind {
        TermKind::Variable => "variable",
        TermKind::It => "it",
        TermKind::Numeral => "numeral",
        TermKind::FunctorApplication => "functor_application",
        TermKind::SelectorAccess => "selector_access",
        TermKind::StructureConstructor => "structure_constructor",
        TermKind::SetEnumeration => "set_enumeration",
        TermKind::SetComprehension => "set_comprehension",
        TermKind::Choice => "choice",
        TermKind::SourceQua => "source_qua",
        TermKind::Parenthesized => "parenthesized",
        TermKind::Unsupported => "unsupported",
    }
}

fn formula_kind_name(kind: FormulaKind) -> &'static str {
    match kind {
        FormulaKind::PredicateApplication => "predicate_application",
        FormulaKind::Equality => "equality",
        FormulaKind::Inequality => "inequality",
        FormulaKind::Membership => "membership",
        FormulaKind::TypeAssertion => "type_assertion",
        FormulaKind::AttributeAssertion => "attribute_assertion",
        FormulaKind::Negation => "negation",
        FormulaKind::Conjunction => "conjunction",
        FormulaKind::Disjunction => "disjunction",
        FormulaKind::Implication => "implication",
        FormulaKind::Biconditional => "biconditional",
        FormulaKind::Quantified => "quantified",
        FormulaKind::Unsupported => "unsupported",
    }
}

fn term_status_name(status: TermStatus) -> &'static str {
    match status {
        TermStatus::Inferred => "inferred",
        TermStatus::Partial => "partial",
        TermStatus::Error => "error",
        TermStatus::Skipped => "skipped",
    }
}

fn formula_status_name(status: FormulaStatus) -> &'static str {
    match status {
        FormulaStatus::Checked => "checked",
        FormulaStatus::Partial => "partial",
        FormulaStatus::Error => "error",
        FormulaStatus::Skipped => "skipped",
    }
}

fn candidate_set_kind_name(kind: CandidateSetKind) -> &'static str {
    match kind {
        CandidateSetKind::Functor => "functor",
        CandidateSetKind::Predicate => "predicate",
        CandidateSetKind::Selector => "selector",
    }
}

fn candidate_set_status_name(status: CandidateSetStatus) -> &'static str {
    match status {
        CandidateSetStatus::Open => "open",
        CandidateSetStatus::Degraded => "degraded",
        CandidateSetStatus::Rejected => "rejected",
    }
}

fn candidate_status_name(status: CandidateStatus) -> &'static str {
    match status {
        CandidateStatus::Viable => "viable",
        CandidateStatus::Degraded => "degraded",
        CandidateStatus::Rejected => "rejected",
    }
}

fn declaration_kind_name(kind: DeclarationKind) -> &'static str {
    match kind {
        DeclarationKind::Let => "let",
        DeclarationKind::ReservedVariable => "reserved_variable",
        DeclarationKind::QuantifiedVariable => "quantified_variable",
        DeclarationKind::Given => "given",
        DeclarationKind::Consider => "consider",
        DeclarationKind::Take => "take",
        DeclarationKind::Set => "set",
        DeclarationKind::DefinitionParameter => "definition_parameter",
        DeclarationKind::DefFuncFormal => "deffunc_formal",
        DeclarationKind::DefPredFormal => "defpred_formal",
        DeclarationKind::ReconsiderExisting => "reconsider_existing",
        DeclarationKind::ReconsiderNew => "reconsider_new",
    }
}

fn declaration_status_name(status: DeclarationStatus) -> &'static str {
    match status {
        DeclarationStatus::Checked => "checked",
        DeclarationStatus::Partial => "partial",
        DeclarationStatus::Error => "error",
    }
}

fn deferred_reason_name(reason: DeclarationDeferredReason) -> &'static str {
    match reason {
        DeclarationDeferredReason::MissingTypePayload => "missing_type_payload",
        DeclarationDeferredReason::MissingReservedDefaultPayload => {
            "missing_reserved_default_payload"
        }
        DeclarationDeferredReason::MissingRightHandSidePayload => "missing_rhs_payload",
        DeclarationDeferredReason::MissingDefinitionBodyPayload => {
            "missing_definition_body_payload"
        }
        DeclarationDeferredReason::MissingEvidenceQuery => "missing_evidence_query",
    }
}

fn term_deferred_reason_name(reason: TermDeferredReason) -> &'static str {
    match reason {
        TermDeferredReason::MissingReferencePayload => "missing_reference_payload",
        TermDeferredReason::MissingNumericTypePayload => "missing_numeric_type_payload",
        TermDeferredReason::MissingSignaturePayload => "missing_signature_payload",
        TermDeferredReason::MissingSelectorPayload => "missing_selector_payload",
        TermDeferredReason::MissingStructurePayload => "missing_structure_payload",
        TermDeferredReason::MissingResultTypePayload => "missing_result_type_payload",
        TermDeferredReason::SethoodRequirement => "sethood_requirement",
        TermDeferredReason::NonEmptinessRequirement => "non_empty_requirement",
        TermDeferredReason::SourceQuaRequirement => "source_qua_requirement",
    }
}

fn formula_deferred_reason_name(reason: FormulaDeferredReason) -> &'static str {
    match reason {
        FormulaDeferredReason::MissingPredicateSignaturePayload => {
            "missing_predicate_signature_payload"
        }
        FormulaDeferredReason::MissingExpectedTypePayload => "missing_expected_type_payload",
        FormulaDeferredReason::MissingQuantifierPayload => "missing_quantifier_payload",
        FormulaDeferredReason::MissingFormulaPayload => "missing_formula_payload",
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeExpressionInput {
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub spelling: String,
    pub head: TypeHeadInput,
    pub args: Vec<TypeExpressionInput>,
    pub attributes: Vec<AttributeInput>,
}

impl TypeExpressionInput {
    pub fn new(
        site: TypedSiteRef,
        source_range: SourceRange,
        spelling: impl Into<String>,
        head: TypeHeadInput,
    ) -> Self {
        Self {
            site,
            source_range,
            spelling: spelling.into(),
            head,
            args: Vec::new(),
            attributes: Vec::new(),
        }
    }

    pub fn with_args(mut self, args: Vec<TypeExpressionInput>) -> Self {
        self.args = args;
        self
    }

    pub fn with_attributes(mut self, attributes: Vec<AttributeInput>) -> Self {
        self.attributes = attributes;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum TypeHeadInput {
    BuiltinObject,
    BuiltinSet,
    Symbol(SymbolId),
    Unresolved(String),
    Ambiguous(Vec<SymbolId>),
    Unsupported(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttributeInput {
    pub symbol: SymbolId,
    pub polarity: AttributePolarity,
    pub args: Vec<TypeExpressionInput>,
    pub source_range: SourceRange,
    pub spelling: String,
}

impl AttributeInput {
    pub fn new(
        symbol: SymbolId,
        polarity: AttributePolarity,
        source_range: SourceRange,
        spelling: impl Into<String>,
    ) -> Self {
        Self {
            symbol,
            polarity,
            args: Vec::new(),
            source_range,
            spelling: spelling.into(),
        }
    }

    pub fn with_args(mut self, args: Vec<TypeExpressionInput>) -> Self {
        self.args = args;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum AttributePolarity {
    Positive,
    Negative,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModeExpansion {
    pub radix: TypeExpressionInput,
    pub attributes: Vec<AttributeInput>,
}

impl ModeExpansion {
    pub fn new(radix: TypeExpressionInput, attributes: Vec<AttributeInput>) -> Self {
        Self { radix, attributes }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NormalizedTypeTable {
    entries: Vec<NormalizedType>,
    ids_by_key: BTreeMap<NormalizedTypeKey, NormalizedTypeId>,
}

impl NormalizedTypeTable {
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
            ids_by_key: BTreeMap::new(),
        }
    }

    fn intern(&mut self, draft: NormalizedTypeDraft) -> NormalizedTypeId {
        let key = NormalizedTypeKey::from(&draft);
        if let Some(id) = self.ids_by_key.get(&key) {
            let entry = &mut self.entries[id.index()];
            entry.source = canonical_source(entry.source.clone(), draft.source);
            entry.status = merge_status(entry.status, draft.status);
            return *id;
        }

        let id = NormalizedTypeId::new(self.entries.len());
        self.entries.push(NormalizedType {
            id,
            head: draft.head,
            args: draft.args,
            attributes: draft.attributes,
            source: draft.source,
            status: draft.status,
        });
        self.ids_by_key.insert(key, id);
        id
    }

    pub fn get(&self, id: NormalizedTypeId) -> Option<&NormalizedType> {
        self.entries.get(id.index())
    }

    pub fn iter(&self) -> impl Iterator<Item = (NormalizedTypeId, &NormalizedType)> {
        self.entries.iter().map(|entry| (entry.id, entry))
    }

    pub fn canonical_iter(&self) -> impl Iterator<Item = (NormalizedTypeId, &NormalizedType)> {
        self.ids_by_key
            .values()
            .copied()
            .map(|id| (id, &self.entries[id.index()]))
    }

    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedType {
    pub id: NormalizedTypeId,
    pub head: TypeHeadRef,
    pub args: Vec<NormalizedTypeId>,
    pub attributes: AttributeSet,
    pub source: TypeSource,
    pub status: NormalizedTypeStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NormalizedTypeDraft {
    head: TypeHeadRef,
    args: Vec<NormalizedTypeId>,
    attributes: AttributeSet,
    source: TypeSource,
    status: NormalizedTypeStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum TypeHeadRef {
    BuiltinObject,
    BuiltinSet,
    Mode(SymbolId),
    Structure(SymbolId),
    Error(TypeHeadErrorKind),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum TypeHeadErrorKind {
    Unknown,
    WrongKind,
    Ambiguous,
    Unsupported,
    Recovery,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttributeSet {
    positive: Vec<AttributeInstance>,
    negative: Vec<AttributeInstance>,
}

impl AttributeSet {
    pub const fn empty() -> Self {
        Self {
            positive: Vec::new(),
            negative: Vec::new(),
        }
    }

    pub fn positive(&self) -> &[AttributeInstance] {
        &self.positive
    }

    pub fn negative(&self) -> &[AttributeInstance] {
        &self.negative
    }

    fn is_empty(&self) -> bool {
        self.positive.is_empty() && self.negative.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AttributeInstance {
    pub symbol: SymbolId,
    pub args: Vec<NormalizedTypeId>,
    pub source_range: SourceRangeKey,
    pub spelling: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeSource {
    pub spelling: String,
    pub range: SourceRange,
}

impl TypeSource {
    pub fn new(spelling: impl Into<String>, range: SourceRange) -> Self {
        Self {
            spelling: spelling.into(),
            range,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum NormalizedTypeStatus {
    Known,
    Degraded,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceRangeKey {
    pub start: usize,
    pub end: usize,
}

impl From<SourceRange> for SourceRangeKey {
    fn from(range: SourceRange) -> Self {
        Self {
            start: range.start,
            end: range.end,
        }
    }
}

struct NormalizationState<'a> {
    symbols: &'a SymbolEnv,
    mode_expansions: &'a BTreeMap<SymbolId, ModeExpansion>,
    normalized_types: NormalizedTypeTable,
    type_entries: TypeTable,
    diagnostics: TypeDiagnosticTable,
}

impl NormalizationState<'_> {
    fn normalize_input(&mut self, input: TypeExpressionInput) -> NormalizedTypeId {
        let TypeExpressionInput {
            site,
            source_range,
            spelling,
            head,
            args,
            attributes,
        } = input;

        let mut status = NormalizedTypeStatus::Known;
        let normalized_args = self.normalize_type_args(args, &site, source_range, &mut status);
        let (head, head_failed) = self.normalize_head(&site, source_range, head);
        if head_failed {
            status = degrade(status);
        }

        let mode_expansion = match &head {
            TypeHeadRef::Mode(symbol) => self.mode_expansions.get(symbol).cloned(),
            _ => None,
        };

        if matches!(head, TypeHeadRef::Mode(_)) && mode_expansion.is_none() {
            self.diagnostic(
                Some(site.clone()),
                source_range,
                TypeDiagnosticClass::TypeExpression,
                TypeDiagnosticSeverity::Note,
                "checker.type.external.mode_expansion_payload",
                DiagnosticRecoveryState::Degraded,
            );
            status = degrade(status);
        }

        let mut all_attributes = attributes;
        if let Some(expansion) = mode_expansion {
            let radix_id = self.normalize_input(expansion.radix);
            let (radix_head, radix_args, radix_attributes, radix_status) =
                match self.normalized_types.get(radix_id).cloned() {
                    Some(normalized) => (
                        normalized.head,
                        normalized.args,
                        normalized.attributes,
                        normalized.status,
                    ),
                    None => (
                        {
                            self.recovery_diagnostic(
                                site.clone(),
                                source_range,
                                "checker.type.recovery.mode_radix",
                            );
                            TypeHeadRef::Error(TypeHeadErrorKind::Recovery)
                        },
                        Vec::new(),
                        AttributeSet::empty(),
                        NormalizedTypeStatus::Error,
                    ),
                };
            if radix_status != NormalizedTypeStatus::Known {
                status = degrade(status);
            }
            if !normalized_args.is_empty() {
                self.diagnostic(
                    Some(site.clone()),
                    source_range,
                    TypeDiagnosticClass::TypeExpression,
                    TypeDiagnosticSeverity::Error,
                    "checker.type.wrong_mode_arity",
                    DiagnosticRecoveryState::Degraded,
                );
                status = degrade(status);
            };
            all_attributes.extend(expansion.attributes);
            let extra_attributes =
                self.normalize_attributes(all_attributes, &site, source_range, &mut status);
            let attribute_set = self.merge_attribute_sets(
                radix_attributes,
                extra_attributes,
                &site,
                source_range,
                &mut status,
            );
            return finish_type(
                &mut self.normalized_types,
                NormalizedTypeDraft {
                    head: radix_head,
                    args: radix_args,
                    attributes: attribute_set,
                    source: TypeSource::new(spelling, source_range),
                    status,
                },
            );
        }

        if !normalized_args.is_empty() {
            match head {
                TypeHeadRef::BuiltinObject | TypeHeadRef::BuiltinSet => {
                    self.diagnostic(
                        Some(site.clone()),
                        source_range,
                        TypeDiagnosticClass::TypeExpression,
                        TypeDiagnosticSeverity::Error,
                        "checker.type.wrong_builtin_arity",
                        DiagnosticRecoveryState::Degraded,
                    );
                    status = degrade(status);
                }
                TypeHeadRef::Structure(_) => {
                    self.diagnostic(
                        Some(site.clone()),
                        source_range,
                        TypeDiagnosticClass::TypeExpression,
                        TypeDiagnosticSeverity::Error,
                        "checker.type.wrong_structure_arity",
                        DiagnosticRecoveryState::Degraded,
                    );
                    status = degrade(status);
                }
                TypeHeadRef::Mode(_) | TypeHeadRef::Error(_) => {}
            }
        }

        let attribute_set =
            self.normalize_attributes(all_attributes, &site, source_range, &mut status);
        finish_type(
            &mut self.normalized_types,
            NormalizedTypeDraft {
                head,
                args: normalized_args,
                attributes: attribute_set,
                source: TypeSource::new(spelling, source_range),
                status,
            },
        )
    }

    fn normalize_type_args(
        &mut self,
        args: Vec<TypeExpressionInput>,
        site: &TypedSiteRef,
        source_range: SourceRange,
        status: &mut NormalizedTypeStatus,
    ) -> Vec<NormalizedTypeId> {
        args.into_iter()
            .map(|arg| {
                let id = self.normalize_input(arg);
                if self
                    .normalized_types
                    .get(id)
                    .is_some_and(|normalized| normalized.status != NormalizedTypeStatus::Known)
                {
                    *status = degrade(*status);
                    self.diagnostic(
                        Some(site.clone()),
                        source_range,
                        TypeDiagnosticClass::TypeExpression,
                        TypeDiagnosticSeverity::Error,
                        "checker.type.argument_degraded",
                        DiagnosticRecoveryState::Degraded,
                    );
                }
                id
            })
            .collect()
    }

    fn normalize_head(
        &mut self,
        site: &TypedSiteRef,
        source_range: SourceRange,
        head: TypeHeadInput,
    ) -> (TypeHeadRef, bool) {
        match head {
            TypeHeadInput::BuiltinObject => (TypeHeadRef::BuiltinObject, false),
            TypeHeadInput::BuiltinSet => (TypeHeadRef::BuiltinSet, false),
            TypeHeadInput::Symbol(symbol) => match self.symbols.symbols().get(&symbol) {
                Some(entry) => match entry.kind() {
                    SymbolKind::Mode => (TypeHeadRef::Mode(symbol), false),
                    SymbolKind::Structure => (TypeHeadRef::Structure(symbol), false),
                    _ => {
                        self.diagnostic(
                            Some(site.clone()),
                            source_range,
                            TypeDiagnosticClass::TypeExpression,
                            TypeDiagnosticSeverity::Error,
                            "checker.type.wrong_head_kind",
                            DiagnosticRecoveryState::Degraded,
                        );
                        (TypeHeadRef::Error(TypeHeadErrorKind::WrongKind), true)
                    }
                },
                None => {
                    self.diagnostic(
                        Some(site.clone()),
                        source_range,
                        TypeDiagnosticClass::TypeExpression,
                        TypeDiagnosticSeverity::Error,
                        "checker.type.unknown_head",
                        DiagnosticRecoveryState::Degraded,
                    );
                    (TypeHeadRef::Error(TypeHeadErrorKind::Unknown), true)
                }
            },
            TypeHeadInput::Unresolved(_) => {
                self.diagnostic(
                    Some(site.clone()),
                    source_range,
                    TypeDiagnosticClass::TypeExpression,
                    TypeDiagnosticSeverity::Error,
                    "checker.type.unknown_head",
                    DiagnosticRecoveryState::Degraded,
                );
                (TypeHeadRef::Error(TypeHeadErrorKind::Unknown), true)
            }
            TypeHeadInput::Ambiguous(_) => {
                self.diagnostic(
                    Some(site.clone()),
                    source_range,
                    TypeDiagnosticClass::TypeExpression,
                    TypeDiagnosticSeverity::Error,
                    "checker.type.ambiguous_head",
                    DiagnosticRecoveryState::Degraded,
                );
                (TypeHeadRef::Error(TypeHeadErrorKind::Ambiguous), true)
            }
            TypeHeadInput::Unsupported(_) => {
                self.diagnostic(
                    Some(site.clone()),
                    source_range,
                    TypeDiagnosticClass::TypeExpression,
                    TypeDiagnosticSeverity::Error,
                    "checker.type.unsupported_payload",
                    DiagnosticRecoveryState::Degraded,
                );
                (TypeHeadRef::Error(TypeHeadErrorKind::Unsupported), true)
            }
        }
    }

    fn normalize_attributes(
        &mut self,
        attributes: Vec<AttributeInput>,
        site: &TypedSiteRef,
        source_range: SourceRange,
        status: &mut NormalizedTypeStatus,
    ) -> AttributeSet {
        let mut positive = BTreeMap::<AttributeSemanticKey, AttributeInstance>::new();
        let mut negative = BTreeMap::<AttributeSemanticKey, AttributeInstance>::new();

        for attribute in attributes {
            let AttributeInput {
                symbol,
                polarity,
                args,
                source_range: attribute_range,
                spelling,
            } = attribute;

            if !matches!(
                self.symbols
                    .symbols()
                    .get(&symbol)
                    .map(|entry| entry.kind()),
                Some(SymbolKind::Attribute)
            ) {
                self.diagnostic(
                    Some(site.clone()),
                    attribute_range,
                    TypeDiagnosticClass::TypeExpression,
                    TypeDiagnosticSeverity::Error,
                    "checker.type.wrong_attribute_kind",
                    DiagnosticRecoveryState::Degraded,
                );
                *status = degrade(*status);
            }

            let args = self.normalize_type_args(args, site, source_range, status);
            let instance = AttributeInstance {
                symbol,
                args,
                source_range: attribute_range.into(),
                spelling,
            };
            let key = AttributeSemanticKey {
                symbol: instance.symbol.clone(),
                args: instance.args.clone(),
            };
            match polarity {
                AttributePolarity::Positive => {
                    insert_canonical_attribute(&mut positive, key, instance);
                }
                AttributePolarity::Negative => {
                    insert_canonical_attribute(&mut negative, key, instance);
                }
            }
        }

        let contradictions = positive
            .keys()
            .filter(|key| negative.contains_key(*key))
            .cloned()
            .collect::<Vec<_>>();
        for _ in contradictions {
            self.diagnostic(
                Some(site.clone()),
                source_range,
                TypeDiagnosticClass::TypeExpression,
                TypeDiagnosticSeverity::Error,
                "checker.type.contradictory_attribute",
                DiagnosticRecoveryState::Degraded,
            );
            *status = degrade(*status);
        }

        AttributeSet {
            positive: positive.into_values().collect(),
            negative: negative.into_values().collect(),
        }
    }

    fn merge_attribute_sets(
        &mut self,
        base: AttributeSet,
        extra: AttributeSet,
        site: &TypedSiteRef,
        source_range: SourceRange,
        status: &mut NormalizedTypeStatus,
    ) -> AttributeSet {
        let mut positive = BTreeMap::<AttributeSemanticKey, AttributeInstance>::new();
        let mut negative = BTreeMap::<AttributeSemanticKey, AttributeInstance>::new();
        for instance in base.positive.into_iter().chain(extra.positive) {
            insert_canonical_attribute(&mut positive, attribute_semantic_key(&instance), instance);
        }
        for instance in base.negative.into_iter().chain(extra.negative) {
            insert_canonical_attribute(&mut negative, attribute_semantic_key(&instance), instance);
        }
        let contradictions = positive
            .keys()
            .filter(|key| negative.contains_key(*key))
            .cloned()
            .collect::<Vec<_>>();
        for _ in contradictions {
            self.diagnostic(
                Some(site.clone()),
                source_range,
                TypeDiagnosticClass::TypeExpression,
                TypeDiagnosticSeverity::Error,
                "checker.type.contradictory_attribute",
                DiagnosticRecoveryState::Degraded,
            );
            *status = degrade(*status);
        }
        AttributeSet {
            positive: positive.into_values().collect(),
            negative: negative.into_values().collect(),
        }
    }

    fn recovery_diagnostic(
        &mut self,
        site: TypedSiteRef,
        source_range: SourceRange,
        message_key: &str,
    ) -> TypeDiagnosticId {
        self.diagnostic(
            Some(site),
            source_range,
            TypeDiagnosticClass::Recovery,
            TypeDiagnosticSeverity::Note,
            message_key,
            DiagnosticRecoveryState::Recovery,
        )
    }

    fn diagnostic(
        &mut self,
        owner: Option<TypedSiteRef>,
        source_range: SourceRange,
        class: TypeDiagnosticClass,
        severity: TypeDiagnosticSeverity,
        message_key: &str,
        recovery: DiagnosticRecoveryState,
    ) -> TypeDiagnosticId {
        self.diagnostics.insert(TypeDiagnosticDraft {
            owner,
            source_range,
            class,
            severity,
            message_key: message_key.to_owned(),
            recovery,
        })
    }
}

fn finish_type(
    normalized_types: &mut NormalizedTypeTable,
    draft: NormalizedTypeDraft,
) -> NormalizedTypeId {
    normalized_types.intern(draft)
}

fn degrade(status: NormalizedTypeStatus) -> NormalizedTypeStatus {
    match status {
        NormalizedTypeStatus::Known | NormalizedTypeStatus::Degraded => {
            NormalizedTypeStatus::Degraded
        }
        NormalizedTypeStatus::Error => NormalizedTypeStatus::Error,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct AttributeSemanticKey {
    symbol: SymbolId,
    args: Vec<NormalizedTypeId>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct NormalizedTypeKey {
    head: TypeHeadRef,
    args: Vec<NormalizedTypeId>,
    positive: Vec<AttributeSemanticKey>,
    negative: Vec<AttributeSemanticKey>,
}

impl From<&NormalizedTypeDraft> for NormalizedTypeKey {
    fn from(draft: &NormalizedTypeDraft) -> Self {
        Self {
            head: draft.head.clone(),
            args: draft.args.clone(),
            positive: draft
                .attributes
                .positive
                .iter()
                .map(attribute_semantic_key)
                .collect(),
            negative: draft
                .attributes
                .negative
                .iter()
                .map(attribute_semantic_key)
                .collect(),
        }
    }
}

impl From<&NormalizedType> for NormalizedTypeKey {
    fn from(normalized: &NormalizedType) -> Self {
        Self {
            head: normalized.head.clone(),
            args: normalized.args.clone(),
            positive: normalized
                .attributes
                .positive
                .iter()
                .map(attribute_semantic_key)
                .collect(),
            negative: normalized
                .attributes
                .negative
                .iter()
                .map(attribute_semantic_key)
                .collect(),
        }
    }
}

fn attribute_semantic_key(instance: &AttributeInstance) -> AttributeSemanticKey {
    AttributeSemanticKey {
        symbol: instance.symbol.clone(),
        args: instance.args.clone(),
    }
}

fn insert_canonical_attribute(
    attributes: &mut BTreeMap<AttributeSemanticKey, AttributeInstance>,
    key: AttributeSemanticKey,
    instance: AttributeInstance,
) {
    let should_insert = match attributes.get(&key) {
        Some(current) => {
            attribute_instance_order_key(&instance) < attribute_instance_order_key(current)
        }
        None => true,
    };
    if should_insert {
        attributes.insert(key, instance);
    }
}

fn attribute_instance_order_key(instance: &AttributeInstance) -> (SourceRangeKey, &str) {
    (instance.source_range, instance.spelling.as_str())
}

fn canonical_source(left: TypeSource, right: TypeSource) -> TypeSource {
    if type_source_order_key(&right) < type_source_order_key(&left) {
        right
    } else {
        left
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct TypeSourceOrderKey {
    source_id: String,
    range: SourceRangeKey,
    spelling: String,
}

fn type_source_order_key(source: &TypeSource) -> TypeSourceOrderKey {
    TypeSourceOrderKey {
        source_id: format!("{:?}", source.range.source_id),
        range: source.range.into(),
        spelling: source.spelling.clone(),
    }
}

fn merge_status(left: NormalizedTypeStatus, right: NormalizedTypeStatus) -> NormalizedTypeStatus {
    if status_rank(right) > status_rank(left) {
        right
    } else {
        left
    }
}

fn status_rank(status: NormalizedTypeStatus) -> u8 {
    match status {
        NormalizedTypeStatus::Known => 0,
        NormalizedTypeStatus::Degraded => 1,
        NormalizedTypeStatus::Error => 2,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct StructuralTypeKey {
    head: TypeHeadRef,
    args: Vec<StructuralTypeKey>,
    positive: Vec<StructuralAttributeKey>,
    negative: Vec<StructuralAttributeKey>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct StructuralAttributeKey {
    symbol: SymbolId,
    args: Vec<StructuralTypeKey>,
}

impl NormalizedTypeTable {
    fn into_canonical(self) -> (Self, BTreeMap<NormalizedTypeId, NormalizedTypeId>) {
        let mut memo = BTreeMap::new();
        let mut by_key = BTreeMap::<StructuralTypeKey, Vec<NormalizedTypeId>>::new();
        for (old_id, _) in self.iter() {
            by_key
                .entry(structural_type_key(old_id, &self.entries, &mut memo))
                .or_default()
                .push(old_id);
        }

        let mut remap = BTreeMap::new();
        let groups = by_key.values().cloned().collect::<Vec<_>>();
        for (new_index, old_ids) in groups.iter().enumerate() {
            let new_id = NormalizedTypeId::new(new_index);
            for old_id in old_ids {
                remap.insert(*old_id, new_id);
            }
        }

        let mut entries = Vec::new();
        let mut ids_by_key = BTreeMap::new();
        for (new_index, old_ids) in groups.into_iter().enumerate() {
            let representative = old_ids
                .iter()
                .copied()
                .min_by(|left, right| {
                    type_source_order_key(&self.entries[left.index()].source)
                        .cmp(&type_source_order_key(&self.entries[right.index()].source))
                })
                .expect("canonical groups are never empty");
            let mut entry = self.entries[representative.index()].clone();
            entry.id = NormalizedTypeId::new(new_index);
            entry.args = entry
                .args
                .into_iter()
                .map(|id| remapped_type_id(id, &remap))
                .collect();
            entry.attributes = canonical_attribute_set_for_group(&old_ids, &self.entries, &remap);
            entry.source = canonical_source_for_group(&old_ids, &self.entries);
            entry.status = merged_status_for_group(&old_ids, &self.entries);
            ids_by_key.insert(NormalizedTypeKey::from(&entry), entry.id);
            entries.push(entry);
        }

        (
            Self {
                entries,
                ids_by_key,
            },
            remap,
        )
    }
}

fn canonical_attribute_set_for_group(
    old_ids: &[NormalizedTypeId],
    entries: &[NormalizedType],
    remap: &BTreeMap<NormalizedTypeId, NormalizedTypeId>,
) -> AttributeSet {
    let mut positive = BTreeMap::<AttributeSemanticKey, AttributeInstance>::new();
    let mut negative = BTreeMap::<AttributeSemanticKey, AttributeInstance>::new();

    for old_id in old_ids {
        let entry = &entries[old_id.index()];
        for attribute in entry.attributes.positive.iter().cloned() {
            let attribute = remap_attribute_instance(attribute, remap);
            insert_canonical_attribute(
                &mut positive,
                attribute_semantic_key(&attribute),
                attribute,
            );
        }
        for attribute in entry.attributes.negative.iter().cloned() {
            let attribute = remap_attribute_instance(attribute, remap);
            insert_canonical_attribute(
                &mut negative,
                attribute_semantic_key(&attribute),
                attribute,
            );
        }
    }

    AttributeSet {
        positive: positive.into_values().collect(),
        negative: negative.into_values().collect(),
    }
}

fn canonical_source_for_group(
    old_ids: &[NormalizedTypeId],
    entries: &[NormalizedType],
) -> TypeSource {
    let mut sources = old_ids
        .iter()
        .map(|old_id| entries[old_id.index()].source.clone());
    let mut source = sources.next().expect("canonical groups are never empty");
    for candidate in sources {
        source = canonical_source(source, candidate);
    }
    source
}

fn merged_status_for_group(
    old_ids: &[NormalizedTypeId],
    entries: &[NormalizedType],
) -> NormalizedTypeStatus {
    old_ids
        .iter()
        .fold(NormalizedTypeStatus::Known, |status, old_id| {
            merge_status(status, entries[old_id.index()].status)
        })
}

fn structural_type_key(
    id: NormalizedTypeId,
    entries: &[NormalizedType],
    memo: &mut BTreeMap<NormalizedTypeId, StructuralTypeKey>,
) -> StructuralTypeKey {
    if let Some(key) = memo.get(&id) {
        return key.clone();
    }
    let entry = &entries[id.index()];
    let key = StructuralTypeKey {
        head: entry.head.clone(),
        args: entry
            .args
            .iter()
            .map(|arg| structural_type_key(*arg, entries, memo))
            .collect(),
        positive: entry
            .attributes
            .positive
            .iter()
            .map(|attribute| structural_attribute_key(attribute, entries, memo))
            .collect(),
        negative: entry
            .attributes
            .negative
            .iter()
            .map(|attribute| structural_attribute_key(attribute, entries, memo))
            .collect(),
    };
    memo.insert(id, key.clone());
    key
}

fn structural_attribute_key(
    attribute: &AttributeInstance,
    entries: &[NormalizedType],
    memo: &mut BTreeMap<NormalizedTypeId, StructuralTypeKey>,
) -> StructuralAttributeKey {
    StructuralAttributeKey {
        symbol: attribute.symbol.clone(),
        args: attribute
            .args
            .iter()
            .map(|arg| structural_type_key(*arg, entries, memo))
            .collect(),
    }
}

fn remap_type_table(
    type_entries: TypeTable,
    remap: &BTreeMap<NormalizedTypeId, NormalizedTypeId>,
) -> TypeTable {
    let mut remapped = TypeTable::new();
    for (_, entry) in type_entries.iter() {
        remapped.insert(TypeEntryDraft {
            owner: entry.owner.clone(),
            expected: entry.expected.map(|id| remapped_type_id(id, remap)),
            actual: remap_type_actual(entry.actual, remap),
            status: entry.status,
            provenance: entry.provenance.clone(),
        });
    }
    remapped
}

fn remap_type_actual(
    actual: TypeEntryActual,
    remap: &BTreeMap<NormalizedTypeId, NormalizedTypeId>,
) -> TypeEntryActual {
    match actual {
        TypeEntryActual::Known(id) => TypeEntryActual::Known(remapped_type_id(id, remap)),
        TypeEntryActual::CandidateSet(id) => TypeEntryActual::CandidateSet(id),
        TypeEntryActual::Absent => TypeEntryActual::Absent,
    }
}

fn remap_attribute_instance(
    mut attribute: AttributeInstance,
    remap: &BTreeMap<NormalizedTypeId, NormalizedTypeId>,
) -> AttributeInstance {
    attribute.args = attribute
        .args
        .into_iter()
        .map(|id| remapped_type_id(id, remap))
        .collect();
    attribute
}

fn remapped_type_id(
    id: NormalizedTypeId,
    remap: &BTreeMap<NormalizedTypeId, NormalizedTypeId>,
) -> NormalizedTypeId {
    remap.get(&id).copied().unwrap_or(id)
}

fn open_candidate_key(
    candidate: &OpenCandidate,
) -> (
    CandidateIdentity,
    Vec<usize>,
    Option<usize>,
    SourceRangeKey,
    CandidateStatus,
) {
    (
        candidate.identity.clone(),
        candidate
            .required_types
            .iter()
            .map(|id| id.index())
            .collect(),
        candidate.result_type.map(|id| id.index()),
        candidate.source_range.into(),
        candidate.status,
    )
}

fn write_checked_terms(output: &mut String, terms: &CheckedTermTable) {
    output.push_str("terms:\n");
    if terms.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    let mut entries = terms.iter().map(|(_, term)| term).collect::<Vec<_>>();
    entries.sort_by_key(|term| (site_key(&term.site), term.kind));
    for (ordinal, term) in entries.into_iter().enumerate() {
        let _ = write!(
            output,
            "  term#{} context#{} kind={} site={} status={} reference=",
            ordinal,
            term.context.index(),
            term_kind_name(term.kind),
            site_key(&term.site),
            term_status_name(term.status)
        );
        write_term_reference(output, term.reference.as_ref());
        let _ = write!(
            output,
            " type_entry=type_entry_id#{}",
            term.type_entry.index()
        );
        output.push_str(" expected=");
        match term.expected_type {
            Some(id) => {
                let _ = write!(output, "normalized_type#{}", id.index());
            }
            None => output.push_str("<none>"),
        }
        output.push_str(" candidate_set=");
        match term.candidate_set {
            Some(id) => {
                let _ = write!(output, "candidate_set#{}", id.index());
            }
            None => output.push_str("<none>"),
        }
        output.push_str(" deferred=");
        write_term_deferred_reasons(output, &term.deferred);
        output.push('\n');
    }
}

fn write_checked_formulas(output: &mut String, formulas: &CheckedFormulaTable) {
    output.push_str("formulas:\n");
    if formulas.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    let mut entries = formulas
        .iter()
        .map(|(_, formula)| formula)
        .collect::<Vec<_>>();
    entries.sort_by_key(|formula| (site_key(&formula.site), formula.kind));
    for (ordinal, formula) in entries.into_iter().enumerate() {
        let _ = write!(
            output,
            "  formula#{} context#{} kind={} site={} status={} terms=",
            ordinal,
            formula.context.index(),
            formula_kind_name(formula.kind),
            site_key(&formula.site),
            formula_status_name(formula.status)
        );
        write_sites(output, &formula.terms);
        output.push_str(" asserted=");
        match formula.asserted_type {
            Some(id) => {
                let _ = write!(output, "normalized_type#{}", id.index());
            }
            None => output.push_str("<none>"),
        }
        output.push_str(" expected=");
        write_expected_type_constraints(output, &formula.expected_types);
        output.push_str(" candidate_set=");
        match formula.candidate_set {
            Some(id) => {
                let _ = write!(output, "candidate_set#{}", id.index());
            }
            None => output.push_str("<none>"),
        }
        output.push_str(" facts=");
        write_fact_ids(output, &formula.facts);
        output.push_str(" deferred=");
        write_formula_deferred_reasons(output, &formula.deferred);
        output.push('\n');
    }
}

fn write_candidate_sets(output: &mut String, candidate_sets: &OpenCandidateSetTable) {
    output.push_str("candidate_sets:\n");
    if candidate_sets.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    let mut entries = candidate_sets
        .iter()
        .map(|(_, candidate_set)| candidate_set)
        .collect::<Vec<_>>();
    entries.sort_by_key(|candidate_set| {
        (
            site_key(&candidate_set.owner),
            candidate_set.kind,
            candidate_set.status,
            SourceRangeKey::from(candidate_set.source_range),
        )
    });
    for candidate_set in entries {
        let _ = write!(
            output,
            "  candidate_set#{} owner={} kind={} status={} range={}..{} candidates=",
            candidate_set.id.index(),
            site_key(&candidate_set.owner),
            candidate_set_kind_name(candidate_set.kind),
            candidate_set_status_name(candidate_set.status),
            candidate_set.source_range.start,
            candidate_set.source_range.end
        );
        write_open_candidates(output, &candidate_set.candidates);
        output.push('\n');
    }
}

fn write_checked_declarations(output: &mut String, declarations: &CheckedDeclarationTable) {
    output.push_str("declarations:\n");
    if declarations.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    let mut entries = declarations
        .iter()
        .map(|(_, declaration)| declaration)
        .collect::<Vec<_>>();
    entries.sort_by_key(|declaration| {
        (
            declaration.binding.index(),
            site_key(&declaration.site),
            declaration.kind,
        )
    });
    for (ordinal, declaration) in entries.into_iter().enumerate() {
        let _ = write!(
            output,
            "  declaration#{} binding#{} context#{} kind={} site={} status={} type_entry=",
            ordinal,
            declaration.binding.index(),
            declaration.context.index(),
            declaration_kind_name(declaration.kind),
            site_key(&declaration.site),
            declaration_status_name(declaration.status)
        );
        match declaration.type_entry {
            Some(type_entry) => {
                let _ = write!(output, "type_entry_id#{}", type_entry.index());
            }
            None => output.push_str("<none>"),
        }
        output.push_str(" type_site=");
        match &declaration.type_site {
            Some(site) => output.push_str(&site_key(site)),
            None => output.push_str("<none>"),
        }
        output.push_str(" facts=");
        write_fact_ids(output, &declaration.facts);
        output.push_str(" deferred=");
        write_deferred_reasons(output, &declaration.deferred);
        output.push('\n');
    }
}

fn write_local_contexts(output: &mut String, contexts: &LocalTypeContextTable) {
    output.push_str("local_contexts:\n");
    if contexts.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    for (id, context) in contexts.iter() {
        let _ = write!(
            output,
            "  local_context#{} owner={} parent=",
            id.index(),
            site_key(&context.owner)
        );
        match context.parent {
            Some(parent) => {
                let _ = write!(output, "local_context#{}", parent.index());
            }
            None => output.push_str("<none>"),
        }
        let _ = write!(
            output,
            " layer={} recovery={} bindings=",
            type_context_layer_name(context.layer),
            context_recovery_name(context.recovery)
        );
        write_binding_refs(output, &context.bindings);
        output.push_str(" introduced=");
        write_fact_ids(output, &context.introduced_assumptions);
        output.push_str(" visible=");
        write_fact_ids(output, &context.visible_facts);
        output.push('\n');
    }
}

fn write_type_facts(output: &mut String, facts: &TypeFactTable) {
    output.push_str("facts:\n");
    if facts.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    for (id, fact) in facts.canonical_iter() {
        let _ = write!(
            output,
            "  fact#{} subject={} predicate=\"",
            id.index(),
            site_key(&fact.subject)
        );
        write_escaped(output, fact.predicate.as_str());
        let _ = write!(
            output,
            "\" polarity={} status={} provenance=",
            polarity_name(fact.polarity),
            fact_status_name(fact.status)
        );
        write_fact_provenance(output, &fact.provenance);
        output.push('\n');
    }
}

fn write_deferred_reasons(output: &mut String, deferred: &[DeclarationDeferredReason]) {
    output.push('[');
    for (index, reason) in deferred.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        output.push_str(deferred_reason_name(*reason));
    }
    output.push(']');
}

fn write_term_deferred_reasons(output: &mut String, deferred: &[TermDeferredReason]) {
    output.push('[');
    for (index, reason) in deferred.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        output.push_str(term_deferred_reason_name(*reason));
    }
    output.push(']');
}

fn write_formula_deferred_reasons(output: &mut String, deferred: &[FormulaDeferredReason]) {
    output.push('[');
    for (index, reason) in deferred.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        output.push_str(formula_deferred_reason_name(*reason));
    }
    output.push(']');
}

fn write_term_reference(output: &mut String, reference: Option<&TermReference>) {
    match reference {
        Some(TermReference::Binding(binding)) => {
            let _ = write!(output, "binding#{}", binding.index());
        }
        Some(TermReference::Symbol(symbol)) => {
            output.push_str("symbol=");
            write_symbol_id(output, symbol);
        }
        None => output.push_str("<none>"),
    }
}

fn write_sites(output: &mut String, sites: &[TypedSiteRef]) {
    output.push('[');
    for (index, site) in sites.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        output.push_str(&site_key(site));
    }
    output.push(']');
}

fn write_expected_type_constraints(output: &mut String, constraints: &[ExpectedTypeConstraint]) {
    output.push('[');
    for (index, constraint) in constraints.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        let _ = write!(
            output,
            "{}=>normalized_type#{}({:?}, range={}..{})",
            site_key(&constraint.term),
            constraint.expected.index(),
            constraint.status,
            constraint.source_range.start,
            constraint.source_range.end
        );
    }
    output.push(']');
}

fn write_open_candidates(output: &mut String, candidates: &[OpenCandidate]) {
    output.push('[');
    for (index, candidate) in candidates.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        write_candidate_identity(output, &candidate.identity);
        output.push_str(" result=");
        match candidate.result_type {
            Some(id) => {
                let _ = write!(output, "normalized_type#{}", id.index());
            }
            None => output.push_str("<none>"),
        }
        output.push_str(" required=");
        write_type_ids(output, &candidate.required_types);
        let _ = write!(
            output,
            " status={} range={}..{}",
            candidate_status_name(candidate.status),
            candidate.source_range.start,
            candidate.source_range.end
        );
    }
    output.push(']');
}

fn write_candidate_identity(output: &mut String, identity: &CandidateIdentity) {
    match identity {
        CandidateIdentity::Symbol(symbol) => {
            output.push_str("symbol=");
            write_symbol_id(output, symbol);
        }
        CandidateIdentity::Builtin(name) => {
            output.push_str("builtin=\"");
            write_escaped(output, name);
            output.push('"');
        }
        CandidateIdentity::External(name) => {
            output.push_str("external=\"");
            write_escaped(output, name);
            output.push('"');
        }
    }
}

fn write_binding_refs(output: &mut String, bindings: &[BindingTypeRef]) {
    output.push('[');
    for (index, binding) in bindings.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        match binding {
            BindingTypeRef::Symbol(symbol) => {
                output.push_str("symbol=");
                write_symbol_id(output, symbol);
            }
            BindingTypeRef::Site(site) => {
                output.push_str("site=");
                output.push_str(&site_key(site));
            }
        }
    }
    output.push(']');
}

fn write_fact_ids(output: &mut String, facts: &[TypeFactId]) {
    output.push('[');
    for (index, fact) in facts.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        let _ = write!(output, "fact#{}", fact.index());
    }
    output.push(']');
}

fn type_context_layer_name(layer: TypeContextLayer) -> &'static str {
    match layer {
        TypeContextLayer::Module => "module",
        TypeContextLayer::Declaration => "declaration",
        TypeContextLayer::Proof => "proof",
        TypeContextLayer::Block => "block",
        TypeContextLayer::Expression => "expression",
    }
}

fn context_recovery_name(recovery: ContextRecoveryState) -> &'static str {
    match recovery {
        ContextRecoveryState::Normal => "normal",
        ContextRecoveryState::Recovered => "recovered",
        ContextRecoveryState::Degraded => "degraded",
    }
}

fn polarity_name(polarity: Polarity) -> &'static str {
    match polarity {
        Polarity::Positive => "positive",
        Polarity::Negative => "negative",
    }
}

fn fact_status_name(status: FactStatus) -> &'static str {
    match status {
        FactStatus::Known => "known",
        FactStatus::Assumed => "assumed",
        FactStatus::PendingObligation => "pending_obligation",
        FactStatus::Degraded => "degraded",
        FactStatus::Rejected => "rejected",
    }
}

fn write_fact_provenance(output: &mut String, provenance: &FactProvenance) {
    match provenance {
        FactProvenance::Declared(range) => {
            let _ = write!(output, "declared:{}..{}", range.start, range.end);
        }
        FactProvenance::Assumed(assumption) => {
            output.push_str("assumed=\"");
            write_escaped(output, assumption.as_str());
            output.push('"');
        }
        FactProvenance::Inferred(rule) => {
            output.push_str("inferred=\"");
            write_escaped(output, rule.as_str());
            output.push('"');
        }
        FactProvenance::Obligation(obligation) => {
            let _ = write!(output, "obligation#{}", obligation.index());
        }
        FactProvenance::Builtin(rule) => {
            output.push_str("builtin=\"");
            write_escaped(output, rule.as_str());
            output.push('"');
        }
        FactProvenance::Registration(step) => {
            output.push_str("registration=\"");
            write_escaped(output, step.as_str());
            output.push('"');
        }
    }
}

fn write_normalized_types(output: &mut String, types: &NormalizedTypeTable) {
    output.push_str("normalized_types:\n");
    if types.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    for (id, ty) in types.canonical_iter() {
        let _ = write!(output, "  normalized_type#{} head=", id.index());
        write_head(output, &ty.head);
        output.push_str(" args=");
        write_type_ids(output, &ty.args);
        output.push_str(" attributes=");
        write_attributes(output, &ty.attributes);
        let _ = writeln!(
            output,
            " status={} source=\"{}\" range={}..{}",
            normalized_status_name(ty.status),
            escaped_display(&ty.source.spelling),
            ty.source.range.start,
            ty.source.range.end
        );
    }
}

fn write_type_entries(output: &mut String, types: &TypeTable) {
    output.push_str("type_entries:\n");
    if types.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    for (ordinal, (_, entry)) in types.canonical_iter().enumerate() {
        let _ = write!(
            output,
            "  type_entry#{} owner={} status={:?} actual=",
            ordinal,
            site_key(&entry.owner),
            entry.status
        );
        match entry.actual {
            TypeEntryActual::Known(id) => {
                let _ = write!(output, "normalized_type#{}", id.index());
            }
            TypeEntryActual::CandidateSet(id) => {
                let _ = write!(output, "candidate_set#{}", id.index());
            }
            TypeEntryActual::Absent => output.push_str("<absent>"),
        }
        output.push('\n');
    }
}

fn write_diagnostics(output: &mut String, diagnostics: &TypeDiagnosticTable) {
    output.push_str("diagnostics:\n");
    if diagnostics.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    let mut diagnostics = diagnostics
        .iter()
        .map(|(_, diagnostic)| diagnostic)
        .collect::<Vec<_>>();
    diagnostics.sort_by_key(|diagnostic| diagnostic_debug_key(diagnostic));
    for (ordinal, diagnostic) in diagnostics.into_iter().enumerate() {
        let _ = write!(output, "  diagnostic#{} owner=", ordinal);
        match &diagnostic.owner {
            Some(owner) => output.push_str(&site_key(owner)),
            None => output.push_str("<none>"),
        }
        let _ = writeln!(
            output,
            " range={}..{} class={:?} severity={:?} message_key=\"{}\" recovery={:?}",
            diagnostic.source_range.start,
            diagnostic.source_range.end,
            diagnostic.class,
            diagnostic.severity,
            escaped_display(&diagnostic.message_key),
            diagnostic.recovery
        );
    }
}

fn diagnostic_debug_key(
    diagnostic: &TypeDiagnostic,
) -> (
    String,
    usize,
    usize,
    TypeDiagnosticClass,
    TypeDiagnosticSeverity,
    String,
    DiagnosticRecoveryState,
) {
    (
        diagnostic
            .owner
            .as_ref()
            .map(site_key)
            .unwrap_or_else(|| "<none>".to_owned()),
        diagnostic.source_range.start,
        diagnostic.source_range.end,
        diagnostic.class,
        diagnostic.severity,
        diagnostic.message_key.clone(),
        diagnostic.recovery,
    )
}

fn write_head(output: &mut String, head: &TypeHeadRef) {
    match head {
        TypeHeadRef::BuiltinObject => output.push_str("builtin_object"),
        TypeHeadRef::BuiltinSet => output.push_str("builtin_set"),
        TypeHeadRef::Mode(symbol) => {
            output.push_str("mode=");
            write_symbol_id(output, symbol);
        }
        TypeHeadRef::Structure(symbol) => {
            output.push_str("structure=");
            write_symbol_id(output, symbol);
        }
        TypeHeadRef::Error(kind) => {
            let _ = write!(output, "error={}", type_head_error_kind_name(*kind));
        }
    }
}

fn type_head_error_kind_name(kind: TypeHeadErrorKind) -> &'static str {
    match kind {
        TypeHeadErrorKind::Unknown => "unknown",
        TypeHeadErrorKind::WrongKind => "wrong_kind",
        TypeHeadErrorKind::Ambiguous => "ambiguous",
        TypeHeadErrorKind::Unsupported => "unsupported",
        TypeHeadErrorKind::Recovery => "recovery",
    }
}

fn write_attributes(output: &mut String, attributes: &AttributeSet) {
    if attributes.is_empty() {
        output.push_str("[]");
        return;
    }
    output.push('[');
    let mut first = true;
    for (polarity, instances) in [
        ("positive", attributes.positive.as_slice()),
        ("negative", attributes.negative.as_slice()),
    ] {
        for instance in instances {
            if !first {
                output.push_str(", ");
            }
            first = false;
            let _ = write!(output, "{polarity}:");
            write_symbol_id(output, &instance.symbol);
            output.push('(');
            write_type_ids(output, &instance.args);
            let _ = write!(
                output,
                ", range={}..{}, spelling=\"{}\")",
                instance.source_range.start,
                instance.source_range.end,
                escaped_display(&instance.spelling)
            );
        }
    }
    output.push(']');
}

fn write_type_ids(output: &mut String, ids: &[NormalizedTypeId]) {
    output.push('[');
    for (index, id) in ids.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        let _ = write!(output, "normalized_type#{}", id.index());
    }
    output.push(']');
}

fn write_symbol_id(output: &mut String, symbol: &SymbolId) {
    output.push_str("{fqn=\"");
    write_escaped(output, symbol.fqn().as_str());
    output.push_str("\" local=\"");
    write_escaped(output, symbol.local().as_str());
    output.push_str("\"}");
}

fn site_key(site: &TypedSiteRef) -> String {
    match site {
        TypedSiteRef::Node(node) => format!("node#{}", node.index()),
        TypedSiteRef::Role { node, role } => format!("node#{}:{}", node.index(), role.as_str()),
    }
}

fn normalized_status_name(status: NormalizedTypeStatus) -> &'static str {
    match status {
        NormalizedTypeStatus::Known => "known",
        NormalizedTypeStatus::Degraded => "degraded",
        NormalizedTypeStatus::Error => "error",
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binding_env::{
        BinderIdentity, BindingContextDraft, BindingContextOwner, BindingContextTable,
        BindingDiagnosticTable, BindingDraft, BindingEnvParts, BindingRecoveryState, BindingTable,
        BindingTypeSite, CapturedFreeVariables,
    };
    use crate::typed_ast::{TypeRole, TypedNodeId};
    use mizar_resolve::{
        env::{
            ContributionKind, DefinitionIndex, ExportStatus, LabelIndex, ModuleLexicalSummaryIndex,
            ModuleSummaryIndex, NamespaceGraph, NamespacePath, OverloadIndex, RegistrationIndex,
            ResolvedExportIndex, ResolvedImportIndex, SourceContributionIndex, SymbolEntry,
            SymbolEnvIndexes, SymbolIndex, Visibility,
        },
        names::LocalTermScope,
        resolved_ast::{FullyQualifiedName, LocalSymbolId, ModuleId, SemanticOrigin},
    };
    use mizar_session::{
        BuildSnapshotId, InMemorySessionIdAllocator, ModulePath, PackageId, SessionIdAllocator,
        SourceAnchor, SourceId,
    };

    #[test]
    fn declarations_attach_types_and_context_snapshots_are_deterministic() {
        let source = source_id();
        let symbols = symbol_env(Vec::new());
        let binding_env = binding_env_for_declarations(
            source,
            vec![
                binding_spec("let_x", BindingKind::LetBinding, BindingStatus::Active),
                binding_spec(
                    "reserved_y",
                    BindingKind::ReservedVariable,
                    BindingStatus::Reserved,
                ),
                binding_spec(
                    "quant_z",
                    BindingKind::QuantifierBinder,
                    BindingStatus::Active,
                ),
                binding_spec(
                    "given_w",
                    BindingKind::QuantifierBinder,
                    BindingStatus::Active,
                ),
                binding_spec(
                    "set_a",
                    BindingKind::LocalAbbreviation,
                    BindingStatus::Active,
                ),
                binding_spec(
                    "param_p",
                    BindingKind::DefinitionParameter,
                    BindingStatus::Active,
                ),
                binding_spec(
                    "deffunc_arg",
                    BindingKind::DefinitionParameter,
                    BindingStatus::Active,
                ),
                binding_spec(
                    "defpred_arg",
                    BindingKind::DefinitionParameter,
                    BindingStatus::Active,
                ),
                binding_spec(
                    "reconsider_old",
                    BindingKind::LetBinding,
                    BindingStatus::Active,
                ),
                binding_spec(
                    "reconsider_new",
                    BindingKind::Generated,
                    BindingStatus::Active,
                ),
                binding_spec(
                    "consider_c",
                    BindingKind::QuantifierBinder,
                    BindingStatus::Active,
                ),
                binding_spec(
                    "take_t",
                    BindingKind::QuantifierBinder,
                    BindingStatus::Active,
                ),
            ],
        );
        let contexts = vec![
            DeclarationContextInput::new(BindingContextId::new(0), site(200), range(source, 0, 1)),
            DeclarationContextInput::new(BindingContextId::new(1), site(201), range(source, 1, 2)),
        ];
        let declarations = vec![
            declaration_with_type(source, 0, DeclarationKind::Let, 10, 100),
            declaration_with_type_in_context(
                source,
                1,
                BindingContextId::new(0),
                DeclarationKind::ReservedVariable,
                11,
                101,
            )
            .with_reserved_default(ReservedDefaultPayload::new(site(101), false))
            .with_assumptions(vec![DeclarationAssumptionInput::new(
                TypePredicateRef::new("reserved_y_is_set"),
                range(source, 11, 12),
            )]),
            declaration_with_type(source, 2, DeclarationKind::QuantifiedVariable, 12, 102),
            declaration_with_type(source, 3, DeclarationKind::Given, 13, 103).with_assumptions(
                vec![DeclarationAssumptionInput::new(
                    TypePredicateRef::new("given_w_is_set"),
                    range(source, 13, 14),
                )],
            ),
            declaration_with_type(source, 4, DeclarationKind::Set, 14, 104)
                .with_deferred(vec![DeclarationDeferredReason::MissingRightHandSidePayload]),
            declaration_with_type(source, 5, DeclarationKind::DefinitionParameter, 15, 105),
            declaration_with_type(source, 6, DeclarationKind::DefFuncFormal, 16, 106)
                .with_deferred(vec![
                    DeclarationDeferredReason::MissingDefinitionBodyPayload,
                ]),
            declaration_with_type(source, 7, DeclarationKind::DefPredFormal, 17, 107)
                .with_deferred(vec![
                    DeclarationDeferredReason::MissingDefinitionBodyPayload,
                ]),
            declaration_with_type(source, 8, DeclarationKind::ReconsiderExisting, 18, 108),
            declaration_with_type(source, 9, DeclarationKind::ReconsiderNew, 19, 109),
            declaration_with_type(source, 10, DeclarationKind::Consider, 20, 110),
            declaration_with_type(source, 11, DeclarationKind::Take, 21, 111),
        ];

        let first = DeclarationChecker::default().check(
            &symbols,
            &binding_env,
            contexts.clone(),
            declarations.clone(),
        );
        let second = DeclarationChecker::default().check(
            &symbols,
            &binding_env,
            contexts.into_iter().rev(),
            declarations.into_iter().rev(),
        );

        assert_eq!(first.debug_text(), second.debug_text());
        assert_eq!(first.declarations().len(), 12);
        assert_eq!(first.contexts().len(), 2);
        assert_eq!(first.facts().len(), 2);
        let declarations_by_binding = declarations_by_binding(&first);
        for binding_index in 0..12 {
            let declaration = declarations_by_binding
                .get(&BindingId::new(binding_index))
                .copied()
                .unwrap();
            assert_eq!(declaration.binding, BindingId::new(binding_index));
            assert_eq!(declaration.site, site(10 + binding_index));
            assert_eq!(declaration.type_site, Some(site(100 + binding_index)));
            let type_entry = first
                .type_entries()
                .get(declaration.type_entry.unwrap())
                .unwrap();
            assert_eq!(type_entry.owner, declaration.site);
            assert!(matches!(type_entry.actual, TypeEntryActual::Known(_)));
            if matches!(
                declaration.kind,
                DeclarationKind::Set
                    | DeclarationKind::DefFuncFormal
                    | DeclarationKind::DefPredFormal
            ) {
                assert_eq!(declaration.status, DeclarationStatus::Partial);
                assert_eq!(type_entry.status, TypeStatus::Unknown);
            } else {
                assert_eq!(declaration.status, DeclarationStatus::Checked);
                assert_eq!(type_entry.status, TypeStatus::Known);
            }
        }
        assert_eq!(
            declarations_by_binding
                .get(&BindingId::new(1))
                .unwrap()
                .facts
                .len(),
            1
        );
        assert_eq!(
            declarations_by_binding
                .get(&BindingId::new(3))
                .unwrap()
                .facts
                .len(),
            1
        );
        let module_context = first.contexts().get(LocalTypeContextId::new(0)).unwrap();
        let block_context = first.contexts().get(LocalTypeContextId::new(1)).unwrap();
        assert_eq!(module_context.parent, None);
        assert_eq!(block_context.parent, Some(LocalTypeContextId::new(0)));
        assert_eq!(
            module_context.bindings,
            vec![BindingTypeRef::Site(site(11))]
        );
        assert_eq!(
            module_context.introduced_assumptions,
            vec![TypeFactId::new(0)]
        );
        assert_eq!(module_context.visible_facts, vec![TypeFactId::new(0)]);
        assert_eq!(
            block_context.introduced_assumptions,
            vec![TypeFactId::new(1)]
        );
        assert_eq!(
            block_context.visible_facts,
            vec![TypeFactId::new(0), TypeFactId::new(1)]
        );
        assert_eq!(
            block_context.bindings,
            vec![
                BindingTypeRef::Site(site(10)),
                BindingTypeRef::Site(site(12)),
                BindingTypeRef::Site(site(13)),
                BindingTypeRef::Site(site(14)),
                BindingTypeRef::Site(site(15)),
                BindingTypeRef::Site(site(16)),
                BindingTypeRef::Site(site(17)),
                BindingTypeRef::Site(site(18)),
                BindingTypeRef::Site(site(19)),
                BindingTypeRef::Site(site(20)),
                BindingTypeRef::Site(site(21)),
            ]
        );
        let module_fact = first.facts().get(TypeFactId::new(0)).unwrap();
        assert_eq!(module_fact.subject, site(11));
        assert_eq!(module_fact.predicate.as_str(), "reserved_y_is_set");
        assert_eq!(module_fact.polarity, Polarity::Positive);
        assert_eq!(module_fact.status, FactStatus::Assumed);
        assert!(matches!(module_fact.provenance, FactProvenance::Assumed(_)));
        let block_fact = first.facts().get(TypeFactId::new(1)).unwrap();
        assert_eq!(block_fact.subject, site(13));
        assert_eq!(block_fact.predicate.as_str(), "given_w_is_set");
        assert_eq!(block_fact.polarity, Polarity::Positive);
        assert_eq!(block_fact.status, FactStatus::Assumed);
        assert!(matches!(block_fact.provenance, FactProvenance::Assumed(_)));
        assert!(first.debug_text().contains("kind=let"));
        assert!(first.debug_text().contains("kind=reserved_variable"));
        assert!(first.debug_text().contains("kind=quantified_variable"));
        assert!(first.debug_text().contains("kind=given"));
        assert!(first.debug_text().contains("kind=consider"));
        assert!(first.debug_text().contains("kind=take"));
        assert!(first.debug_text().contains("kind=set"));
        assert!(first.debug_text().contains("kind=definition_parameter"));
        assert!(first.debug_text().contains("kind=deffunc_formal"));
        assert!(first.debug_text().contains("kind=defpred_formal"));
        assert!(first.debug_text().contains("kind=reconsider_existing"));
        assert!(first.debug_text().contains("kind=reconsider_new"));
        assert!(
            first
                .debug_text()
                .contains("checker.declaration.deferred.rhs_payload")
        );
        assert!(
            first
                .debug_text()
                .contains("checker.declaration.deferred.definition_body_payload")
        );
        assert_eq!(
            diagnostic_ranges(&first, "checker.declaration.deferred.rhs_payload"),
            vec![(70, 75)]
        );
        assert_eq!(
            diagnostic_ranges(
                &first,
                "checker.declaration.deferred.definition_body_payload"
            ),
            vec![(80, 85), (85, 90)]
        );
        assert!(!first.debug_text().contains("obligation#"));
        assert!(!first.debug_text().contains("initial_obligation"));
        assert!(!first.debug_text().contains(concat!("V", "cId")));
        assert!(!first.debug_text().contains(concat!("Proof", "Witness")));
        assert!(
            !first
                .debug_text()
                .contains(concat!("active", "_refinement"))
        );
    }

    #[test]
    fn reserved_shadowing_and_missing_payloads_are_partial_with_ranges() {
        let source = source_id();
        let symbols = symbol_env(Vec::new());
        let binding_env = binding_env_for_declarations(
            source,
            vec![
                binding_spec(
                    "reserved_y",
                    BindingKind::ReservedVariable,
                    BindingStatus::Reserved,
                ),
                binding_spec(
                    "reserved_z",
                    BindingKind::ReservedVariable,
                    BindingStatus::Reserved,
                ),
                binding_spec(
                    "reserved_bad",
                    BindingKind::ReservedVariable,
                    BindingStatus::Reserved,
                ),
                binding_spec("let_x", BindingKind::LetBinding, BindingStatus::Active),
            ],
        );
        let contexts = vec![
            DeclarationContextInput::new(BindingContextId::new(0), site(199), range(source, 0, 1)),
            DeclarationContextInput::new(BindingContextId::new(1), site(200), range(source, 1, 2)),
        ];
        let shadowed = declaration_with_type_in_context(
            source,
            0,
            BindingContextId::new(0),
            DeclarationKind::ReservedVariable,
            10,
            100,
        )
        .with_reserved_default(ReservedDefaultPayload::new(site(100), true));
        let missing = DeclarationInput::new(
            BindingId::new(3),
            BindingContextId::new(1),
            site(11),
            range(source, 40, 45),
            DeclarationKind::Let,
        );
        let missing_reserved_default = DeclarationInput::new(
            BindingId::new(1),
            BindingContextId::new(0),
            site(12),
            range(source, 60, 65),
            DeclarationKind::ReservedVariable,
        );
        let mismatched_default = declaration_with_type_in_context(
            source,
            2,
            BindingContextId::new(0),
            DeclarationKind::ReservedVariable,
            13,
            103,
        )
        .with_reserved_default(ReservedDefaultPayload::new(site(999), false));

        let output = DeclarationChecker::default().check(
            &symbols,
            &binding_env,
            contexts,
            [
                shadowed,
                missing,
                missing_reserved_default,
                mismatched_default,
            ],
        );

        assert_eq!(output.normalized_types().len(), 1);
        assert_eq!(output.declarations().len(), 4);
        assert!(
            output
                .declarations()
                .iter()
                .all(|(_, declaration)| { declaration.status == DeclarationStatus::Partial })
        );
        let declarations = declarations_by_binding(&output);
        let shadowed_declaration = declarations.get(&BindingId::new(0)).copied().unwrap();
        assert_eq!(shadowed_declaration.type_site, None);
        let shadowed_entry = output
            .type_entries()
            .get(shadowed_declaration.type_entry.unwrap())
            .unwrap();
        assert_eq!(shadowed_entry.owner, site(10));
        assert_eq!(shadowed_entry.actual, TypeEntryActual::Absent);
        assert_eq!(shadowed_entry.status, TypeStatus::Unknown);
        assert_eq!(
            diagnostic_ranges(&output, "checker.declaration.reserved_default_shadowed"),
            vec![(50, 55)]
        );
        assert_eq!(
            diagnostic_ranges(&output, "checker.declaration.deferred.type_payload"),
            vec![(40, 45)]
        );
        assert_eq!(
            diagnostic_ranges(
                &output,
                "checker.declaration.deferred.reserved_default_payload"
            ),
            vec![(60, 65)]
        );
        assert_eq!(
            diagnostic_ranges(
                &output,
                "checker.declaration.reserved_default_type_site_mismatch"
            ),
            vec![(65, 70)]
        );
        let mismatch_declaration = declarations_by_binding(&output)
            .get(&BindingId::new(2))
            .copied()
            .unwrap();
        assert_eq!(mismatch_declaration.type_site, Some(site(103)));
        let mismatch_type_entry = output
            .type_entries()
            .get(mismatch_declaration.type_entry.unwrap())
            .unwrap();
        assert!(matches!(
            mismatch_type_entry.actual,
            TypeEntryActual::Known(_)
        ));
        assert_eq!(mismatch_type_entry.status, TypeStatus::Unknown);
    }

    #[test]
    fn invalid_declarations_keep_partial_output_and_deterministic_diagnostics() {
        let source = source_id();
        let symbols = symbol_env(Vec::new());
        let binding_env = binding_env_for_declarations(
            source,
            vec![
                binding_spec("let_x", BindingKind::LetBinding, BindingStatus::Active),
                binding_spec(
                    "set_a",
                    BindingKind::LocalAbbreviation,
                    BindingStatus::Omitted,
                ),
                binding_spec("let_y", BindingKind::LetBinding, BindingStatus::Active),
            ],
        );
        let contexts = vec![DeclarationContextInput::new(
            BindingContextId::new(1),
            site(200),
            range(source, 0, 1),
        )];
        let declarations = vec![
            declaration_with_type(source, 0, DeclarationKind::Set, 10, 100),
            declaration_with_type(source, 1, DeclarationKind::Set, 11, 101),
            declaration_with_type(source, 99, DeclarationKind::Let, 12, 102).with_assumptions(
                vec![DeclarationAssumptionInput::new(
                    TypePredicateRef::new("bad_assumption"),
                    range(source, 60, 65),
                )],
            ),
            DeclarationInput::new(
                BindingId::new(0),
                BindingContextId::new(99),
                site(10),
                range(source, 60, 65),
                DeclarationKind::Let,
            ),
            declaration_with_type_in_context(
                source,
                2,
                BindingContextId::new(0),
                DeclarationKind::Let,
                13,
                103,
            ),
        ];

        let output =
            DeclarationChecker::default().check(&symbols, &binding_env, contexts, declarations);

        assert_eq!(output.declarations().len(), 5);
        assert!(output.type_entries().len() >= 4);
        let declarations = declarations_by_binding(&output);
        assert_eq!(
            declarations.get(&BindingId::new(0)).unwrap().status,
            DeclarationStatus::Partial
        );
        assert_eq!(
            declarations.get(&BindingId::new(1)).unwrap().status,
            DeclarationStatus::Partial
        );
        assert_eq!(
            declarations.get(&BindingId::new(99)).unwrap().status,
            DeclarationStatus::Error
        );
        for declaration in declarations.values().copied() {
            let type_entry = output
                .type_entries()
                .get(declaration.type_entry.unwrap())
                .unwrap();
            assert_eq!(type_entry.owner, declaration.site);
            if declaration.status == DeclarationStatus::Error {
                assert_eq!(type_entry.status, TypeStatus::Error);
            }
        }
        let unknown_context_declaration = output
            .declarations()
            .iter()
            .map(|(_, declaration)| declaration)
            .find(|declaration| {
                declaration.binding == BindingId::new(0)
                    && declaration.context == BindingContextId::new(99)
                    && declaration.site == site(10)
                    && declaration.kind == DeclarationKind::Let
            })
            .unwrap();
        assert_eq!(unknown_context_declaration.status, DeclarationStatus::Error);
        let unknown_context_type_entry = output
            .type_entries()
            .get(unknown_context_declaration.type_entry.unwrap())
            .unwrap();
        assert_eq!(unknown_context_type_entry.actual, TypeEntryActual::Absent);
        assert_eq!(unknown_context_type_entry.status, TypeStatus::Error);
        let duplicate_set_declaration = output
            .declarations()
            .iter()
            .map(|(_, declaration)| declaration)
            .find(|declaration| {
                declaration.binding == BindingId::new(0)
                    && declaration.context == BindingContextId::new(1)
                    && declaration.site == site(10)
                    && declaration.kind == DeclarationKind::Set
            })
            .unwrap();
        assert_eq!(duplicate_set_declaration.status, DeclarationStatus::Partial);
        let duplicate_set_type_entry = output
            .type_entries()
            .get(duplicate_set_declaration.type_entry.unwrap())
            .unwrap();
        assert!(matches!(
            duplicate_set_type_entry.actual,
            TypeEntryActual::Known(_)
        ));
        assert_eq!(duplicate_set_type_entry.status, TypeStatus::Unknown);
        let context_mismatch_declaration = output
            .declarations()
            .iter()
            .map(|(_, declaration)| declaration)
            .find(|declaration| {
                declaration.binding == BindingId::new(2)
                    && declaration.context == BindingContextId::new(0)
                    && declaration.site == site(13)
                    && declaration.kind == DeclarationKind::Let
            })
            .unwrap();
        assert_eq!(
            context_mismatch_declaration.status,
            DeclarationStatus::Partial
        );
        let context_mismatch_type_entry = output
            .type_entries()
            .get(context_mismatch_declaration.type_entry.unwrap())
            .unwrap();
        assert!(matches!(
            context_mismatch_type_entry.actual,
            TypeEntryActual::Known(_)
        ));
        assert_eq!(context_mismatch_type_entry.status, TypeStatus::Unknown);
        assert!(output.facts().is_empty());
        for expected in [
            "checker.declaration.illegal_binding_kind",
            "checker.declaration.omitted_binding",
            "checker.declaration.unknown_binding",
            "checker.declaration.unknown_context",
            "checker.declaration.binding_context_mismatch",
            "checker.declaration.duplicate_site",
            "checker.declaration.duplicate_binding",
            "checker.declaration.assumption_dropped_after_recovery",
        ] {
            assert!(output.debug_text().contains(expected), "{expected}");
        }
        assert_eq!(
            diagnostic_ranges(&output, "checker.declaration.unknown_context"),
            vec![(60, 65)]
        );
        assert_eq!(
            diagnostic_ranges(&output, "checker.declaration.illegal_binding_kind"),
            vec![(0, 5)]
        );
        assert_eq!(
            diagnostic_ranges(&output, "checker.declaration.omitted_binding"),
            vec![(10, 15)]
        );
        assert_eq!(
            diagnostic_ranges(&output, "checker.declaration.unknown_binding"),
            vec![(60, 65)]
        );
        assert_eq!(
            diagnostic_ranges(&output, "checker.declaration.binding_context_mismatch"),
            vec![(60, 65), (65, 70)]
        );
        assert_eq!(
            diagnostic_ranges(&output, "checker.declaration.duplicate_site"),
            vec![(50, 55)]
        );
        assert_eq!(
            diagnostic_ranges(&output, "checker.declaration.duplicate_binding"),
            vec![(50, 55)]
        );
        assert_eq!(
            diagnostic_ranges(
                &output,
                "checker.declaration.assumption_dropped_after_recovery"
            ),
            vec![(60, 65)]
        );
        assert_eq!(output.contexts().len(), 1);
    }

    #[test]
    fn constrained_declarations_defer_obligations_without_fabricating_facts() {
        let source = source_id();
        let symbols = symbol_env(Vec::new());
        let binding_env = binding_env_for_declarations(
            source,
            vec![binding_spec(
                "let_x",
                BindingKind::LetBinding,
                BindingStatus::Active,
            )],
        );
        let declaration = declaration_with_type(source, 0, DeclarationKind::Let, 10, 100)
            .with_deferred(vec![DeclarationDeferredReason::MissingEvidenceQuery])
            .with_assumptions(vec![DeclarationAssumptionInput::new(
                TypePredicateRef::new("partial_assumption"),
                range(source, 50, 55),
            )]);

        let output = DeclarationChecker::default().check(
            &symbols,
            &binding_env,
            [DeclarationContextInput::new(
                BindingContextId::new(1),
                site(200),
                range(source, 0, 1),
            )],
            [declaration],
        );

        assert!(output.facts().is_empty());
        assert!(
            output
                .debug_text()
                .contains("checker.declaration.deferred.evidence_query")
        );
        assert_eq!(
            diagnostic_ranges(&output, "checker.declaration.deferred.evidence_query"),
            vec![(50, 55)]
        );
        assert_eq!(
            diagnostic_ranges(
                &output,
                "checker.declaration.assumption_dropped_after_recovery"
            ),
            vec![(50, 55)]
        );
        assert!(output.debug_text().contains("status=partial"));
        assert!(!output.debug_text().contains("obligation#"));
        assert!(!output.debug_text().contains("registration"));
    }

    #[test]
    fn set_declaration_missing_rhs_payload_is_partial_without_fabricated_type() {
        let source = source_id();
        let symbols = symbol_env(Vec::new());
        let binding_env = binding_env_for_declarations(
            source,
            vec![binding_spec(
                "set_a",
                BindingKind::LocalAbbreviation,
                BindingStatus::Active,
            )],
        );
        let declaration = DeclarationInput::new(
            BindingId::new(0),
            BindingContextId::new(1),
            site(10),
            range(source, 50, 55),
            DeclarationKind::Set,
        )
        .with_deferred(vec![DeclarationDeferredReason::MissingRightHandSidePayload]);

        let output = DeclarationChecker::default().check(
            &symbols,
            &binding_env,
            [DeclarationContextInput::new(
                BindingContextId::new(1),
                site(200),
                range(source, 0, 1),
            )],
            [declaration],
        );

        let declaration = output.declarations().iter().next().unwrap().1;
        assert_eq!(declaration.status, DeclarationStatus::Partial);
        assert!(
            declaration
                .deferred
                .contains(&DeclarationDeferredReason::MissingRightHandSidePayload)
        );
        assert_eq!(declaration.type_site, None);
        let type_entry = output
            .type_entries()
            .get(declaration.type_entry.unwrap())
            .unwrap();
        assert_eq!(type_entry.owner, site(10));
        assert_eq!(type_entry.actual, TypeEntryActual::Absent);
        assert_eq!(type_entry.status, TypeStatus::Unknown);
        assert!(output.facts().is_empty());
        assert_eq!(
            diagnostic_ranges(&output, "checker.declaration.deferred.rhs_payload"),
            vec![(50, 55)]
        );
        assert!(!output.debug_text().contains("obligation#"));
        assert!(!output.debug_text().contains("registration"));
    }

    #[test]
    fn attributed_declaration_defers_evidence_without_losing_normalized_type() {
        let source = source_id();
        let attr = symbol_id("Attr/1", "pkg::main::Attr/1");
        let symbols = symbol_env(vec![symbol_entry(attr.clone(), SymbolKind::Attribute)]);
        let binding_env = binding_env_for_declarations(
            source,
            vec![binding_spec(
                "let_x",
                BindingKind::LetBinding,
                BindingStatus::Active,
            )],
        );
        let declaration = DeclarationInput::new(
            BindingId::new(0),
            BindingContextId::new(1),
            site(10),
            range(source, 50, 55),
            DeclarationKind::Let,
        )
        .with_type_expression(
            TypeExpressionInput::new(
                site(100),
                range(source, 60, 70),
                "set attr",
                TypeHeadInput::BuiltinSet,
            )
            .with_attributes(vec![AttributeInput::new(
                attr.clone(),
                AttributePolarity::Positive,
                range(source, 64, 68),
                "Attr",
            )]),
        )
        .with_deferred(vec![DeclarationDeferredReason::MissingEvidenceQuery]);

        let output = DeclarationChecker::default().check(
            &symbols,
            &binding_env,
            [DeclarationContextInput::new(
                BindingContextId::new(1),
                site(200),
                range(source, 0, 1),
            )],
            [declaration],
        );

        let declaration = output.declarations().iter().next().unwrap().1;
        assert_eq!(declaration.status, DeclarationStatus::Partial);
        let type_entry = output
            .type_entries()
            .get(declaration.type_entry.unwrap())
            .unwrap();
        assert_eq!(type_entry.owner, site(10));
        assert_eq!(type_entry.status, TypeStatus::Unknown);
        let TypeEntryActual::Known(normalized_id) = type_entry.actual else {
            panic!("attributed declaration should keep its normalized type");
        };
        let normalized = output.normalized_types().get(normalized_id).unwrap();
        assert_eq!(normalized.status, NormalizedTypeStatus::Known);
        assert_eq!(normalized.attributes.positive().len(), 1);
        assert_eq!(normalized.attributes.positive()[0].symbol, attr);
        assert!(normalized.attributes.negative().is_empty());
        assert!(output.facts().is_empty());
        assert_eq!(
            diagnostic_ranges(&output, "checker.declaration.deferred.evidence_query"),
            vec![(50, 55)]
        );
        assert!(!output.debug_text().contains("obligation#"));
        assert!(!output.debug_text().contains("registration"));
    }

    #[test]
    fn reconsider_existing_allows_additional_type_view_for_same_binding() {
        let source = source_id();
        let symbols = symbol_env(Vec::new());
        let binding_env = binding_env_for_declarations(
            source,
            vec![binding_spec(
                "let_x",
                BindingKind::LetBinding,
                BindingStatus::Active,
            )],
        );
        let original = declaration_with_type(source, 0, DeclarationKind::Let, 10, 100);
        let reconsider =
            declaration_with_type(source, 0, DeclarationKind::ReconsiderExisting, 11, 101);

        let output = DeclarationChecker::default().check(
            &symbols,
            &binding_env,
            [DeclarationContextInput::new(
                BindingContextId::new(1),
                site(200),
                range(source, 0, 1),
            )],
            [original, reconsider],
        );

        assert_eq!(output.declarations().len(), 2);
        assert_eq!(
            diagnostic_ranges(&output, "checker.declaration.duplicate_binding"),
            Vec::<(usize, usize)>::new()
        );
        let reconsider = output
            .declarations()
            .iter()
            .map(|(_, declaration)| declaration)
            .find(|declaration| declaration.kind == DeclarationKind::ReconsiderExisting)
            .unwrap();
        assert_eq!(reconsider.binding, BindingId::new(0));
        assert_eq!(reconsider.status, DeclarationStatus::Checked);
        let type_entry = output
            .type_entries()
            .get(reconsider.type_entry.unwrap())
            .unwrap();
        assert_eq!(type_entry.owner, site(11));
        assert_eq!(type_entry.status, TypeStatus::Known);
        assert!(matches!(type_entry.actual, TypeEntryActual::Known(_)));
    }

    #[test]
    fn reconsider_existing_accepts_visible_outer_binding_from_child_context() {
        let source = source_id();
        let symbols = symbol_env(Vec::new());
        let binding_env = binding_env_with_outer_visible_binding(source);
        let reconsider = declaration_with_type_in_context(
            source,
            0,
            BindingContextId::new(1),
            DeclarationKind::ReconsiderExisting,
            11,
            101,
        );

        let output = DeclarationChecker::default().check(
            &symbols,
            &binding_env,
            [
                DeclarationContextInput::new(
                    BindingContextId::new(0),
                    site(199),
                    range(source, 0, 1),
                ),
                DeclarationContextInput::new(
                    BindingContextId::new(1),
                    site(200),
                    range(source, 1, 2),
                ),
            ],
            [reconsider],
        );

        assert_eq!(
            diagnostic_ranges(&output, "checker.declaration.binding_context_mismatch"),
            Vec::<(usize, usize)>::new()
        );
        let declaration = output.declarations().iter().next().unwrap().1;
        assert_eq!(declaration.binding, BindingId::new(0));
        assert_eq!(declaration.context, BindingContextId::new(1));
        assert_eq!(declaration.status, DeclarationStatus::Checked);
        let type_entry = output
            .type_entries()
            .get(declaration.type_entry.unwrap())
            .unwrap();
        assert_eq!(type_entry.owner, site(11));
        assert_eq!(type_entry.status, TypeStatus::Known);
        assert!(matches!(type_entry.actual, TypeEntryActual::Known(_)));
    }

    #[test]
    fn term_inference_covers_term_kinds_and_open_candidates_deterministically() {
        let source = source_id();
        let functor = symbol_id("F/1", "pkg::main::F/1");
        let selector = symbol_id("sel", "pkg::main::sel");
        let structure = symbol_id("Struct", "pkg::main::Struct");
        let symbols = symbol_env(vec![
            symbol_entry(functor.clone(), SymbolKind::Functor),
            symbol_entry(selector.clone(), SymbolKind::Selector),
            symbol_entry(structure.clone(), SymbolKind::Structure),
        ]);
        let binding_env = binding_env_for_declarations(
            source,
            vec![binding_spec(
                "let_x",
                BindingKind::LetBinding,
                BindingStatus::Active,
            )],
        );
        let terms = vec![
            term_with_type(source, 10, TermKind::Variable, 100)
                .with_reference(TermReference::Binding(BindingId::new(0))),
            term_with_type(source, 11, TermKind::Numeral, 101),
            term_with_type(source, 12, TermKind::It, 102),
            TermInput::new(
                site(13),
                BindingContextId::new(1),
                range(source, 65, 70),
                TermKind::FunctorApplication,
            )
            .with_candidates(vec![
                OpenCandidateInput::new(
                    CandidateIdentity::Builtin("zeta".to_owned()),
                    range(source, 80, 81),
                )
                .with_result_type(type_expression(
                    source,
                    103,
                    TypeHeadInput::BuiltinSet,
                )),
                OpenCandidateInput::new(
                    CandidateIdentity::Builtin("alpha".to_owned()),
                    range(source, 70, 71),
                )
                .with_result_type(type_expression(
                    source,
                    104,
                    TypeHeadInput::BuiltinObject,
                )),
            ]),
            TermInput::new(
                site(14),
                BindingContextId::new(1),
                range(source, 70, 75),
                TermKind::SelectorAccess,
            )
            .with_candidates(vec![
                OpenCandidateInput::new(CandidateIdentity::Symbol(selector), range(source, 75, 76))
                    .with_result_type(type_expression(source, 105, TypeHeadInput::BuiltinSet)),
            ]),
            term_with_type(source, 15, TermKind::StructureConstructor, 106).with_result_type(
                type_expression(source, 106, TypeHeadInput::Symbol(structure)),
            ),
            term_with_type(source, 16, TermKind::SetEnumeration, 107)
                .with_deferred(vec![TermDeferredReason::SethoodRequirement]),
            term_with_type(source, 17, TermKind::SetComprehension, 108)
                .with_deferred(vec![TermDeferredReason::SethoodRequirement]),
            term_with_type(source, 18, TermKind::Choice, 109)
                .with_deferred(vec![TermDeferredReason::NonEmptinessRequirement]),
            term_with_type(source, 19, TermKind::SourceQua, 110)
                .with_deferred(vec![TermDeferredReason::SourceQuaRequirement]),
            term_with_type(source, 20, TermKind::Parenthesized, 111),
        ];

        let first =
            TermFormulaChecker::default().infer(&symbols, &binding_env, terms.clone(), Vec::new());
        let second = TermFormulaChecker::default().infer(
            &symbols,
            &binding_env,
            terms.into_iter().rev(),
            Vec::new(),
        );

        assert_eq!(first.debug_text(), second.debug_text());
        assert_eq!(first.terms().len(), 11);
        assert_eq!(first.candidate_sets().len(), 2);
        let variable = term_by_site(&first, site(10));
        assert_eq!(variable.status, TermStatus::Inferred);
        assert!(matches!(
            first
                .type_entries()
                .get(variable.type_entry)
                .unwrap()
                .actual,
            TypeEntryActual::Known(_)
        ));
        let functor = term_by_site(&first, site(13));
        assert_eq!(functor.status, TermStatus::Partial);
        assert!(matches!(
            first.type_entries().get(functor.type_entry).unwrap().actual,
            TypeEntryActual::CandidateSet(_)
        ));
        let set_enum = term_by_site(&first, site(16));
        assert_eq!(
            set_enum.deferred,
            vec![TermDeferredReason::SethoodRequirement]
        );
        let choice = term_by_site(&first, site(18));
        assert_eq!(
            choice.deferred,
            vec![TermDeferredReason::NonEmptinessRequirement]
        );
        let source_qua = term_by_site(&first, site(19));
        assert_eq!(
            source_qua.deferred,
            vec![TermDeferredReason::SourceQuaRequirement]
        );
        let debug = first.debug_text();
        assert!(debug.contains("kind=selector_access"));
        assert!(debug.contains("kind=structure_constructor"));
        assert!(debug.contains("kind=parenthesized"));
        assert!(debug.contains("candidate_set#"));
        assert!(debug.find("builtin=\"alpha\"") < debug.find("builtin=\"zeta\""));
        assert!(!debug.contains("obligation#"));
        assert!(!debug.contains("registration"));
    }

    #[test]
    fn formula_inference_records_expected_constraints_facts_and_open_candidates() {
        let source = source_id();
        let predicate = symbol_id("P/1", "pkg::main::P/1");
        let attr = symbol_id("Attr/1", "pkg::main::Attr/1");
        let symbols = symbol_env(vec![
            symbol_entry(predicate.clone(), SymbolKind::Predicate),
            symbol_entry(attr.clone(), SymbolKind::Attribute),
        ]);
        let binding_env = binding_env_for_declarations(
            source,
            vec![binding_spec(
                "let_x",
                BindingKind::LetBinding,
                BindingStatus::Active,
            )],
        );
        let terms = vec![
            term_with_type(source, 10, TermKind::Variable, 100)
                .with_reference(TermReference::Binding(BindingId::new(0))),
            term_with_type(source, 11, TermKind::Numeral, 101),
        ];
        let formulas = vec![
            FormulaInput::new(
                site(30),
                BindingContextId::new(1),
                range(source, 150, 155),
                FormulaKind::Equality,
            )
            .with_terms(vec![site(10), site(11)])
            .with_expected_types(vec![
                ExpectedTypeInput::new(
                    site(10),
                    type_expression(source, 200, TypeHeadInput::BuiltinSet),
                    range(source, 150, 151),
                ),
                ExpectedTypeInput::new(
                    site(11),
                    type_expression(source, 201, TypeHeadInput::BuiltinSet),
                    range(source, 152, 153),
                ),
            ]),
            FormulaInput::new(
                site(31),
                BindingContextId::new(1),
                range(source, 155, 160),
                FormulaKind::Inequality,
            )
            .with_terms(vec![site(10), site(11)])
            .with_expected_types(vec![
                ExpectedTypeInput::new(
                    site(10),
                    type_expression(source, 205, TypeHeadInput::BuiltinSet),
                    range(source, 155, 156),
                ),
                ExpectedTypeInput::new(
                    site(11),
                    type_expression(source, 206, TypeHeadInput::BuiltinSet),
                    range(source, 157, 158),
                ),
            ]),
            FormulaInput::new(
                site(32),
                BindingContextId::new(1),
                range(source, 160, 165),
                FormulaKind::Membership,
            )
            .with_terms(vec![site(10), site(11)])
            .with_expected_types(vec![ExpectedTypeInput::new(
                site(11),
                type_expression(source, 202, TypeHeadInput::BuiltinSet),
                range(source, 162, 163),
            )]),
            FormulaInput::new(
                site(33),
                BindingContextId::new(1),
                range(source, 165, 170),
                FormulaKind::PredicateApplication,
            )
            .with_terms(vec![site(10)])
            .with_candidates(vec![
                OpenCandidateInput::new(
                    CandidateIdentity::Builtin("z-pred".to_owned()),
                    range(source, 166, 167),
                ),
                OpenCandidateInput::new(
                    CandidateIdentity::Symbol(predicate),
                    range(source, 165, 166),
                ),
            ]),
            FormulaInput::new(
                site(42),
                BindingContextId::new(1),
                range(source, 190, 195),
                FormulaKind::PredicateApplication,
            )
            .with_terms(vec![site(10)]),
            FormulaInput::new(
                site(34),
                BindingContextId::new(1),
                range(source, 170, 175),
                FormulaKind::TypeAssertion,
            )
            .with_terms(vec![site(10)])
            .with_asserted_type(type_expression(source, 203, TypeHeadInput::BuiltinSet))
            .with_facts(vec![FormulaFactInput::new(
                site(10),
                TypePredicateRef::new("type_asserted_set"),
                Polarity::Positive,
                range(source, 170, 175),
            )]),
            FormulaInput::new(
                site(35),
                BindingContextId::new(1),
                range(source, 175, 180),
                FormulaKind::AttributeAssertion,
            )
            .with_terms(vec![site(10)])
            .with_asserted_type(
                type_expression(source, 204, TypeHeadInput::BuiltinSet).with_attributes(vec![
                    AttributeInput::new(
                        attr,
                        AttributePolarity::Positive,
                        range(source, 176, 177),
                        "Attr",
                    ),
                ]),
            )
            .with_facts(vec![FormulaFactInput::new(
                site(10),
                TypePredicateRef::new("attr_asserted"),
                Polarity::Positive,
                range(source, 175, 180),
            )]),
            FormulaInput::new(
                site(36),
                BindingContextId::new(1),
                range(source, 180, 181),
                FormulaKind::Negation,
            ),
            FormulaInput::new(
                site(37),
                BindingContextId::new(1),
                range(source, 181, 182),
                FormulaKind::Conjunction,
            ),
            FormulaInput::new(
                site(38),
                BindingContextId::new(1),
                range(source, 182, 183),
                FormulaKind::Disjunction,
            ),
            FormulaInput::new(
                site(39),
                BindingContextId::new(1),
                range(source, 183, 184),
                FormulaKind::Implication,
            ),
            FormulaInput::new(
                site(40),
                BindingContextId::new(1),
                range(source, 184, 185),
                FormulaKind::Biconditional,
            ),
            FormulaInput::new(
                site(41),
                BindingContextId::new(1),
                range(source, 185, 190),
                FormulaKind::Quantified,
            )
            .with_deferred(vec![FormulaDeferredReason::MissingQuantifierPayload]),
        ];

        let first = TermFormulaChecker::default().infer(
            &symbols,
            &binding_env,
            terms.clone(),
            formulas.clone(),
        );
        let second = TermFormulaChecker::default().infer(
            &symbols,
            &binding_env,
            terms.into_iter().rev(),
            formulas.into_iter().rev(),
        );
        assert_eq!(first.debug_text(), second.debug_text());
        let output = first;

        assert_eq!(output.formulas().len(), 13);
        assert_eq!(output.facts().len(), 2);
        let equality = formula_by_site(&output, site(30));
        assert_eq!(equality.expected_types.len(), 2);
        assert_eq!(equality.status, FormulaStatus::Checked);
        let inequality = formula_by_site(&output, site(31));
        assert_eq!(inequality.expected_types.len(), 2);
        assert_eq!(inequality.status, FormulaStatus::Checked);
        let membership = formula_by_site(&output, site(32));
        assert_eq!(membership.expected_types.len(), 1);
        assert_eq!(membership.expected_types[0].term, site(11));
        assert_eq!(membership.status, FormulaStatus::Checked);
        let predicate = formula_by_site(&output, site(33));
        assert_eq!(predicate.status, FormulaStatus::Partial);
        assert!(predicate.candidate_set.is_some());
        let missing_predicate_payload = formula_by_site(&output, site(42));
        assert_eq!(missing_predicate_payload.status, FormulaStatus::Partial);
        assert_eq!(
            diagnostic_ranges(
                &output,
                "checker.formula.external.predicate_signature_payload"
            ),
            vec![(190, 195)]
        );
        let type_assertion = formula_by_site(&output, site(34));
        assert_eq!(type_assertion.facts.len(), 1);
        let fact = output.facts().get(type_assertion.facts[0]).unwrap();
        assert_eq!(fact.status, FactStatus::Known);
        assert!(matches!(fact.provenance, FactProvenance::Inferred(_)));
        let quantified = formula_by_site(&output, site(41));
        assert_eq!(quantified.status, FormulaStatus::Partial);
        assert_eq!(
            diagnostic_ranges(&output, "checker.formula.external.quantifier_payload"),
            vec![(185, 190)]
        );
        let debug = output.debug_text();
        for expected in [
            "kind=equality",
            "kind=inequality",
            "kind=membership",
            "kind=predicate_application",
            "kind=type_assertion",
            "kind=attribute_assertion",
            "kind=negation",
            "kind=conjunction",
            "kind=disjunction",
            "kind=implication",
            "kind=biconditional",
            "kind=quantified",
        ] {
            assert!(debug.contains(expected), "{expected}");
        }
        assert!(!debug.contains("obligation#"));
        assert!(!debug.contains("registration"));
    }

    #[test]
    fn term_formula_recovery_keeps_partial_entries_without_successful_types() {
        let source = source_id();
        let symbols = symbol_env(Vec::new());
        let binding_env = binding_env_for_declarations(
            source,
            vec![binding_spec(
                "let_x",
                BindingKind::LetBinding,
                BindingStatus::Active,
            )],
        );
        let terms = vec![
            TermInput::new(
                site(50),
                BindingContextId::new(1),
                range(source, 250, 255),
                TermKind::It,
            ),
            TermInput::new(
                site(51),
                BindingContextId::new(1),
                range(source, 255, 260),
                TermKind::Unsupported,
            ),
            TermInput::new(
                site(52),
                BindingContextId::new(1),
                range(source, 260, 265),
                TermKind::Variable,
            )
            .with_reference(TermReference::Binding(BindingId::new(99))),
            TermInput::new(
                site(53),
                BindingContextId::new(1),
                range(source, 265, 270),
                TermKind::Numeral,
            ),
            TermInput::new(
                site(54),
                BindingContextId::new(1),
                range(source, 270, 275),
                TermKind::Variable,
            )
            .with_reference(TermReference::Binding(BindingId::new(0)))
            .with_result_type(type_expression(
                source,
                120,
                TypeHeadInput::Unresolved("MissingMode".to_owned()),
            )),
        ];
        let formulas = vec![
            FormulaInput::new(
                site(60),
                BindingContextId::new(1),
                range(source, 300, 305),
                FormulaKind::Unsupported,
            ),
            FormulaInput::new(
                site(61),
                BindingContextId::new(1),
                range(source, 305, 310),
                FormulaKind::Equality,
            )
            .with_terms(vec![site(50), site(99)]),
            FormulaInput::new(
                site(62),
                BindingContextId::new(1),
                range(source, 310, 315),
                FormulaKind::Membership,
            )
            .with_terms(vec![site(51)]),
            FormulaInput::new(
                site(63),
                BindingContextId::new(1),
                range(source, 315, 320),
                FormulaKind::Inequality,
            )
            .with_terms(vec![site(53)]),
            FormulaInput::new(
                site(64),
                BindingContextId::new(1),
                range(source, 320, 325),
                FormulaKind::TypeAssertion,
            )
            .with_terms(vec![site(54)])
            .with_facts(vec![FormulaFactInput::new(
                site(54),
                TypePredicateRef::new("degraded_term_fact"),
                Polarity::Positive,
                range(source, 320, 325),
            )]),
        ];

        let output = TermFormulaChecker::default().infer(&symbols, &binding_env, terms, formulas);

        let it = term_by_site(&output, site(50));
        assert_eq!(it.status, TermStatus::Error);
        let it_entry = output.type_entries().get(it.type_entry).unwrap();
        assert_eq!(it_entry.actual, TypeEntryActual::Absent);
        assert_eq!(it_entry.status, TypeStatus::Error);
        let unsupported = term_by_site(&output, site(51));
        assert_eq!(unsupported.status, TermStatus::Skipped);
        assert_eq!(
            output
                .type_entries()
                .get(unsupported.type_entry)
                .unwrap()
                .status,
            TypeStatus::Skipped
        );
        let unknown_binding = term_by_site(&output, site(52));
        assert_eq!(unknown_binding.status, TermStatus::Error);
        let numeral = term_by_site(&output, site(53));
        assert_eq!(numeral.status, TermStatus::Partial);
        assert_eq!(
            output
                .type_entries()
                .get(numeral.type_entry)
                .unwrap()
                .actual,
            TypeEntryActual::Absent
        );
        let degraded_type_term = term_by_site(&output, site(54));
        assert_eq!(degraded_type_term.status, TermStatus::Inferred);
        assert_eq!(
            output
                .type_entries()
                .get(degraded_type_term.type_entry)
                .unwrap()
                .status,
            TypeStatus::Unknown
        );
        assert_eq!(
            formula_by_site(&output, site(60)).status,
            FormulaStatus::Skipped
        );
        assert_eq!(
            formula_by_site(&output, site(61)).status,
            FormulaStatus::Error
        );
        assert_eq!(
            formula_by_site(&output, site(62)).status,
            FormulaStatus::Error
        );
        assert_eq!(
            formula_by_site(&output, site(63)).status,
            FormulaStatus::Partial
        );
        let degraded_fact_formula = formula_by_site(&output, site(64));
        assert_eq!(degraded_fact_formula.status, FormulaStatus::Partial);
        assert_eq!(degraded_fact_formula.facts.len(), 1);
        assert_eq!(
            output
                .facts()
                .get(degraded_fact_formula.facts[0])
                .unwrap()
                .status,
            FactStatus::Degraded
        );
        assert_eq!(
            diagnostic_ranges(&output, "checker.formula.term.not_well_formed"),
            vec![(305, 310), (310, 315)]
        );
        assert_eq!(
            diagnostic_ranges(&output, "checker.formula.term.missing"),
            vec![(305, 310)]
        );
        assert_eq!(
            diagnostic_ranges(&output, "checker.formula.term.partial"),
            vec![(315, 320), (320, 325)]
        );
        for key in [
            "checker.term.it.missing_current_result_type",
            "checker.term.unsupported_payload",
            "checker.term.reference.unknown_binding",
            "checker.term.external.numeric_type_payload",
            "checker.formula.unsupported_payload",
            "checker.formula.term.not_well_formed",
            "checker.formula.term.missing",
            "checker.formula.term.partial",
        ] {
            assert!(output.debug_text().contains(key), "{key}");
        }
        assert_eq!(output.facts().len(), 1);
        assert!(
            !output
                .debug_text()
                .contains("status=Known actual=normalized_type")
        );
    }

    #[test]
    fn attributes_are_sorted_deduplicated_and_contradictions_are_diagnosed() {
        let source = source_id();
        let attr_a = symbol_id("AttrA/1", "pkg::main::AttrA/1");
        let attr_b = symbol_id("AttrB/1", "pkg::main::AttrB/1");
        let symbols = symbol_env(vec![
            symbol_entry(attr_b.clone(), SymbolKind::Attribute),
            symbol_entry(attr_a.clone(), SymbolKind::Attribute),
        ]);
        let input = TypeExpressionInput::new(
            site(0),
            range(source, 0, 10),
            "set attr",
            TypeHeadInput::BuiltinSet,
        )
        .with_attributes(vec![
            AttributeInput::new(
                attr_b.clone(),
                AttributePolarity::Positive,
                range(source, 8, 9),
                "B",
            ),
            AttributeInput::new(
                attr_a.clone(),
                AttributePolarity::Negative,
                range(source, 4, 5),
                "non A",
            ),
            AttributeInput::new(
                attr_a.clone(),
                AttributePolarity::Positive,
                range(source, 2, 3),
                "A",
            ),
            AttributeInput::new(
                attr_b,
                AttributePolarity::Positive,
                range(source, 6, 7),
                "B",
            ),
        ]);

        let output = TypeNormalizer::default().normalize(&symbols, [input]);
        let normalized = output
            .normalized_types()
            .get(NormalizedTypeId::new(0))
            .unwrap();

        assert_eq!(normalized.status, NormalizedTypeStatus::Degraded);
        assert_eq!(normalized.attributes.positive().len(), 2);
        assert_eq!(normalized.attributes.negative().len(), 1);
        assert_eq!(
            normalized
                .attributes
                .positive()
                .iter()
                .map(|attribute| attribute.symbol.local().as_str())
                .collect::<Vec<_>>(),
            vec!["AttrA/1", "AttrB/1"]
        );
        assert_eq!(output.diagnostics().len(), 2);
        assert!(
            output
                .debug_text()
                .contains("checker.type.contradictory_attribute")
        );
        assert_eq!(
            diagnostic_ranges(&output, "checker.type.contradictory_attribute"),
            vec![(0, 10)]
        );
    }

    #[test]
    fn attribute_arguments_and_duplicate_ranges_are_canonicalized() {
        let source = source_id();
        let attr = symbol_id("Attr/1", "pkg::main::Attr/1");
        let symbols = symbol_env(vec![symbol_entry(attr.clone(), SymbolKind::Attribute)]);
        let input = TypeExpressionInput::new(
            site(0),
            range(source, 0, 50),
            "attributed set",
            TypeHeadInput::BuiltinSet,
        )
        .with_attributes(vec![
            AttributeInput::new(
                attr.clone(),
                AttributePolarity::Positive,
                range(source, 30, 35),
                "Attr set late",
            )
            .with_args(vec![TypeExpressionInput::new(
                site(1),
                range(source, 36, 39),
                "set",
                TypeHeadInput::BuiltinSet,
            )]),
            AttributeInput::new(
                attr.clone(),
                AttributePolarity::Positive,
                range(source, 20, 25),
                "Attr object",
            )
            .with_args(vec![TypeExpressionInput::new(
                site(2),
                range(source, 26, 32),
                "object",
                TypeHeadInput::BuiltinObject,
            )]),
            AttributeInput::new(
                attr.clone(),
                AttributePolarity::Positive,
                range(source, 10, 15),
                "Attr set early",
            )
            .with_args(vec![TypeExpressionInput::new(
                site(3),
                range(source, 16, 19),
                "set",
                TypeHeadInput::BuiltinSet,
            )]),
        ]);

        let output = TypeNormalizer::default().normalize(&symbols, [input]);
        let normalized = output
            .type_entries()
            .iter()
            .find_map(|(_, entry)| match entry.actual {
                TypeEntryActual::Known(id) if entry.owner == site(0) => {
                    output.normalized_types().get(id)
                }
                _ => None,
            })
            .unwrap();

        assert_eq!(normalized.status, NormalizedTypeStatus::Known);
        assert_eq!(normalized.attributes.positive().len(), 2);
        assert_eq!(
            normalized
                .attributes
                .positive()
                .iter()
                .map(|attribute| attribute.args.clone())
                .collect::<Vec<_>>(),
            vec![
                vec![NormalizedTypeId::new(0)],
                vec![NormalizedTypeId::new(1)]
            ]
        );
        assert_eq!(normalized.attributes.positive()[1].source_range.start, 10);
        assert_eq!(normalized.attributes.positive()[1].source_range.end, 15);
    }

    #[test]
    fn equivalent_inputs_have_order_independent_debug_rendering() {
        let (source, alternate_source) = source_ids_pair();
        let symbols = symbol_env(Vec::new());
        let known_later = TypeExpressionInput::new(
            site(2),
            range(source, 20, 23),
            "set shared",
            TypeHeadInput::BuiltinSet,
        );
        let known_earlier = TypeExpressionInput::new(
            site(1),
            range(alternate_source, 20, 23),
            "set shared",
            TypeHeadInput::BuiltinSet,
        );
        let bad_later = TypeExpressionInput::new(
            site(4),
            range(source, 40, 47),
            "missing later",
            TypeHeadInput::Unresolved("missing".to_owned()),
        );
        let bad_earlier = TypeExpressionInput::new(
            site(3),
            range(source, 30, 37),
            "missing earlier",
            TypeHeadInput::Unresolved("missing".to_owned()),
        );

        let first = TypeNormalizer::default().normalize(
            &symbols,
            [
                known_later.clone(),
                bad_later.clone(),
                known_earlier.clone(),
                bad_earlier.clone(),
            ],
        );
        let second = TypeNormalizer::default().normalize(
            &symbols,
            [bad_earlier, known_earlier, bad_later, known_later],
        );

        assert_eq!(first.debug_text(), second.debug_text());
        assert_eq!(first.normalized_types().len(), 2);
        assert!(first.debug_text().contains("source=\"set shared\""));
        assert!(first.debug_text().contains("error=unknown"));
        assert!(!first.debug_text().contains("error=diagnostic#"));
        assert_eq!(
            first
                .normalized_types()
                .get(NormalizedTypeId::new(0))
                .unwrap()
                .source
                .range
                .source_id,
            second
                .normalized_types()
                .get(NormalizedTypeId::new(0))
                .unwrap()
                .source
                .range
                .source_id
        );
    }

    #[test]
    fn builtins_structures_and_recursive_arguments_have_deterministic_ids() {
        let source = source_id();
        let structure = symbol_id("Struct/0", "pkg::main::Struct/0");
        let mode = symbol_id("Mode/1", "pkg::main::Mode/1");
        let symbols = symbol_env(vec![
            symbol_entry(structure.clone(), SymbolKind::Structure),
            symbol_entry(mode.clone(), SymbolKind::Mode),
        ]);
        let arg = TypeExpressionInput::new(
            site(1),
            range(source, 2, 5),
            "set",
            TypeHeadInput::BuiltinSet,
        );
        let input = TypeExpressionInput::new(
            site(0),
            range(source, 0, 6),
            "Mode of set",
            TypeHeadInput::Symbol(mode.clone()),
        )
        .with_args(vec![arg]);
        let structure_input = TypeExpressionInput::new(
            site(2),
            range(source, 7, 13),
            "Struct",
            TypeHeadInput::Symbol(structure),
        );
        let object_input = TypeExpressionInput::new(
            site(3),
            range(source, 14, 20),
            "object",
            TypeHeadInput::BuiltinObject,
        );
        let mode_with_object_arg = TypeExpressionInput::new(
            site(4),
            range(source, 21, 35),
            "Mode of object",
            TypeHeadInput::Symbol(mode.clone()),
        )
        .with_args(vec![TypeExpressionInput::new(
            site(5),
            range(source, 29, 35),
            "object",
            TypeHeadInput::BuiltinObject,
        )]);

        let first = TypeNormalizer::default().normalize(
            &symbols,
            [
                input.clone(),
                structure_input.clone(),
                object_input.clone(),
                mode_with_object_arg.clone(),
            ],
        );
        let second = TypeNormalizer::default().normalize(
            &symbols,
            [input, structure_input, object_input, mode_with_object_arg],
        );

        assert_eq!(first.debug_text(), second.debug_text());
        assert_eq!(first.normalized_types().len(), 5);
        assert!(matches!(
            first
                .normalized_types()
                .get(NormalizedTypeId::new(0))
                .unwrap()
                .head,
            TypeHeadRef::BuiltinObject
        ));
        assert!(matches!(
            first
                .normalized_types()
                .get(NormalizedTypeId::new(1))
                .unwrap()
                .head,
            TypeHeadRef::BuiltinSet
        ));
        assert!(matches!(
            first.normalized_types().get(NormalizedTypeId::new(2)).unwrap().head,
            TypeHeadRef::Mode(ref actual) if actual == &mode
        ));
        assert!(
            first
                .normalized_types()
                .iter()
                .any(|(_, normalized)| matches!(normalized.head, TypeHeadRef::Structure(_)))
        );
        let mode_arg_sets = first
            .normalized_types()
            .iter()
            .filter_map(|(id, normalized)| match normalized.head {
                TypeHeadRef::Mode(ref actual) if actual == &mode => Some((id, &normalized.args)),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(mode_arg_sets.len(), 2);
        assert_ne!(mode_arg_sets[0].0, mode_arg_sets[1].0);
        assert_ne!(mode_arg_sets[0].1, mode_arg_sets[1].1);
        assert!(
            mode_arg_sets
                .iter()
                .any(|(_, args)| args.as_slice() == [NormalizedTypeId::new(0)])
        );
        assert!(
            mode_arg_sets
                .iter()
                .any(|(_, args)| args.as_slice() == [NormalizedTypeId::new(1)])
        );
        assert_eq!(
            first
                .type_entries()
                .canonical_iter()
                .map(|(id, _)| id.index())
                .collect::<Vec<_>>(),
            vec![0, 1, 2, 3]
        );
        assert!(
            first
                .debug_text()
                .starts_with("type-normalization-debug-v1\n")
        );
        assert!(!first.debug_text().contains("SourceId"));
        assert!(!first.debug_text().contains(concat!("V", "cId")));
        assert!(!first.debug_text().contains(concat!("Proof", "Witness")));
        assert!(!first.debug_text().contains("accepted_verifier_status"));
        assert!(!first.debug_text().contains(concat!("inserted", "_qua")));
        assert!(
            !first
                .debug_text()
                .contains(concat!("active", "_refinement"))
        );
        assert!(!first.debug_text().contains(concat!("overload", "_root")));
    }

    #[test]
    fn mode_expansion_provider_unfolds_radix_and_attributes_idempotently() {
        let source = source_id();
        let mode = symbol_id("Mode/0", "pkg::main::Mode/0");
        let attr = symbol_id("Expandable/0", "pkg::main::Expandable/0");
        let symbols = symbol_env(vec![
            symbol_entry(mode.clone(), SymbolKind::Mode),
            symbol_entry(attr.clone(), SymbolKind::Attribute),
        ]);
        let expansion = ModeExpansion::new(
            TypeExpressionInput::new(
                site(10),
                range(source, 20, 23),
                "set",
                TypeHeadInput::BuiltinSet,
            ),
            vec![AttributeInput::new(
                attr.clone(),
                AttributePolarity::Positive,
                range(source, 24, 25),
                "expandable",
            )],
        );
        let input = TypeExpressionInput::new(
            site(0),
            range(source, 0, 4),
            "Mode",
            TypeHeadInput::Symbol(mode.clone()),
        );
        let repeated = TypeExpressionInput::new(
            site(1),
            range(source, 5, 9),
            "Mode",
            TypeHeadInput::Symbol(mode.clone()),
        );

        let first = TypeNormalizer::new([(mode.clone(), expansion.clone())])
            .normalize(&symbols, [input.clone(), repeated.clone()]);
        let second =
            TypeNormalizer::new([(mode, expansion)]).normalize(&symbols, [input, repeated]);
        assert_eq!(first.debug_text(), second.debug_text());
        assert_eq!(first.normalized_types().len(), 2);
        assert_eq!(
            first
                .type_entries()
                .canonical_iter()
                .map(|(_, entry)| entry.actual)
                .collect::<Vec<_>>(),
            vec![
                TypeEntryActual::Known(NormalizedTypeId::new(1)),
                TypeEntryActual::Known(NormalizedTypeId::new(1)),
            ]
        );
        let expanded = first
            .normalized_types()
            .canonical_iter()
            .map(|(_, normalized)| normalized)
            .find(|normalized| !normalized.attributes.positive().is_empty())
            .unwrap();

        assert!(matches!(expanded.head, TypeHeadRef::BuiltinSet));
        assert_eq!(expanded.attributes.positive()[0].symbol, attr);
        assert_eq!(first.diagnostics().len(), 0);
    }

    #[test]
    fn mode_expansion_preserves_radix_arguments_and_attributes() {
        let source = source_id();
        let outer = symbol_id("Outer/0", "pkg::main::Outer/0");
        let inner = symbol_id("Inner/1", "pkg::main::Inner/1");
        let base_attr = symbol_id("Base/0", "pkg::main::Base/0");
        let extra_attr = symbol_id("Extra/0", "pkg::main::Extra/0");
        let symbols = symbol_env(vec![
            symbol_entry(outer.clone(), SymbolKind::Mode),
            symbol_entry(inner.clone(), SymbolKind::Mode),
            symbol_entry(base_attr.clone(), SymbolKind::Attribute),
            symbol_entry(extra_attr.clone(), SymbolKind::Attribute),
        ]);
        let radix = TypeExpressionInput::new(
            site(10),
            range(source, 20, 30),
            "Inner of object",
            TypeHeadInput::Symbol(inner.clone()),
        )
        .with_args(vec![TypeExpressionInput::new(
            site(11),
            range(source, 29, 35),
            "object",
            TypeHeadInput::BuiltinObject,
        )])
        .with_attributes(vec![AttributeInput::new(
            base_attr.clone(),
            AttributePolarity::Positive,
            range(source, 36, 37),
            "base",
        )]);
        let expansion = ModeExpansion::new(
            radix,
            vec![AttributeInput::new(
                extra_attr.clone(),
                AttributePolarity::Positive,
                range(source, 38, 39),
                "extra",
            )],
        );
        let input = TypeExpressionInput::new(
            site(0),
            range(source, 0, 5),
            "Outer",
            TypeHeadInput::Symbol(outer.clone()),
        );

        let output = TypeNormalizer::new([(outer, expansion)]).normalize(&symbols, [input]);
        let expanded = output
            .type_entries()
            .iter()
            .find_map(|(_, entry)| match entry.actual {
                TypeEntryActual::Known(id) if entry.owner == site(0) => {
                    output.normalized_types().get(id)
                }
                _ => None,
            })
            .unwrap();

        assert!(matches!(expanded.head, TypeHeadRef::Mode(ref actual) if actual == &inner));
        assert_eq!(expanded.args.len(), 1);
        assert_eq!(
            expanded
                .attributes
                .positive()
                .iter()
                .map(|attribute| attribute.symbol.local().as_str())
                .collect::<Vec<_>>(),
            vec!["Base/0", "Extra/0"]
        );
    }

    #[test]
    fn mode_expansion_with_arguments_reports_wrong_mode_arity() {
        let source = source_id();
        let mode = symbol_id("Mode/0", "pkg::main::Mode/0");
        let symbols = symbol_env(vec![symbol_entry(mode.clone(), SymbolKind::Mode)]);
        let expansion = ModeExpansion::new(
            TypeExpressionInput::new(
                site(10),
                range(source, 20, 23),
                "set",
                TypeHeadInput::BuiltinSet,
            ),
            Vec::new(),
        );
        let input = TypeExpressionInput::new(
            site(0),
            range(source, 0, 12),
            "Mode of object",
            TypeHeadInput::Symbol(mode.clone()),
        )
        .with_args(vec![TypeExpressionInput::new(
            site(1),
            range(source, 8, 14),
            "object",
            TypeHeadInput::BuiltinObject,
        )]);

        let output = TypeNormalizer::new([(mode, expansion)]).normalize(&symbols, [input]);
        let normalized = output
            .type_entries()
            .iter()
            .find_map(|(_, entry)| match entry.actual {
                TypeEntryActual::Known(id) if entry.owner == site(0) => {
                    output.normalized_types().get(id)
                }
                _ => None,
            })
            .unwrap();

        assert!(matches!(normalized.head, TypeHeadRef::BuiltinSet));
        assert_eq!(normalized.status, NormalizedTypeStatus::Degraded);
        assert_eq!(
            diagnostic_ranges(&output, "checker.type.wrong_mode_arity"),
            vec![(0, 12)]
        );
    }

    #[test]
    fn missing_mode_payload_degrades_without_cluster_repair() {
        let source = source_id();
        let mode = symbol_id("Mode/0", "pkg::main::Mode/0");
        let symbols = symbol_env(vec![symbol_entry(mode.clone(), SymbolKind::Mode)]);
        let input = TypeExpressionInput::new(
            site(0),
            range(source, 0, 4),
            "Mode",
            TypeHeadInput::Symbol(mode.clone()),
        );

        let output = TypeNormalizer::default().normalize(&symbols, [input]);
        let normalized = output
            .normalized_types()
            .get(NormalizedTypeId::new(0))
            .unwrap();

        assert!(matches!(normalized.head, TypeHeadRef::Mode(ref actual) if actual == &mode));
        assert_eq!(normalized.status, NormalizedTypeStatus::Degraded);
        assert!(
            output
                .debug_text()
                .contains("checker.type.external.mode_expansion_payload")
        );
        assert_eq!(
            diagnostic_ranges(&output, "checker.type.external.mode_expansion_payload"),
            vec![(0, 4)]
        );
        assert!(!output.debug_text().contains("cluster"));
        assert!(!output.debug_text().contains("registration"));
    }

    #[test]
    fn unknown_wrong_kind_ambiguous_and_unsupported_heads_are_partial_entries() {
        let source = source_id();
        let attr = symbol_id("Attr/0", "pkg::main::Attr/0");
        let candidate_a = symbol_id("A/0", "pkg::main::A/0");
        let candidate_b = symbol_id("B/0", "pkg::main::B/0");
        let symbols = symbol_env(vec![symbol_entry(attr.clone(), SymbolKind::Attribute)]);
        let inputs = [
            TypeExpressionInput::new(
                site(0),
                range(source, 0, 1),
                "missing",
                TypeHeadInput::Unresolved("missing".to_owned()),
            ),
            TypeExpressionInput::new(
                site(1),
                range(source, 2, 3),
                "Attr",
                TypeHeadInput::Symbol(attr),
            ),
            TypeExpressionInput::new(
                site(2),
                range(source, 4, 5),
                "ambiguous",
                TypeHeadInput::Ambiguous(vec![candidate_b, candidate_a]),
            ),
            TypeExpressionInput::new(
                site(3),
                range(source, 6, 7),
                "unsupported",
                TypeHeadInput::Unsupported("raw".to_owned()),
            ),
        ];

        let output = TypeNormalizer::default().normalize(&symbols, inputs);

        assert_eq!(output.type_entries().len(), 4);
        assert!(
            output
                .type_entries()
                .iter()
                .all(|(_, entry)| entry.status == TypeStatus::Unknown)
        );
        for expected in [
            "checker.type.unknown_head",
            "checker.type.wrong_head_kind",
            "checker.type.ambiguous_head",
            "checker.type.unsupported_payload",
            "checker.type.recovery",
        ] {
            assert!(output.debug_text().contains(expected), "{expected}");
        }
        assert_eq!(
            diagnostic_ranges(&output, "checker.type.unknown_head"),
            vec![(0, 1)]
        );
        assert_eq!(
            diagnostic_ranges(&output, "checker.type.wrong_head_kind"),
            vec![(2, 3)]
        );
        assert_eq!(
            diagnostic_ranges(&output, "checker.type.ambiguous_head"),
            vec![(4, 5)]
        );
        assert_eq!(
            diagnostic_ranges(&output, "checker.type.unsupported_payload"),
            vec![(6, 7)]
        );
    }

    #[test]
    fn structure_arity_and_wrong_attribute_kind_emit_degraded_diagnostics() {
        let source = source_id();
        let structure = symbol_id("Struct/0", "pkg::main::Struct/0");
        let mode = symbol_id("Mode/0", "pkg::main::Mode/0");
        let wrong_attribute = symbol_id("Wrong/0", "pkg::main::Wrong/0");
        let symbols = symbol_env(vec![
            symbol_entry(structure, SymbolKind::Structure),
            symbol_entry(mode.clone(), SymbolKind::Mode),
            symbol_entry(wrong_attribute.clone(), SymbolKind::Mode),
        ]);
        let input = TypeExpressionInput::new(
            site(0),
            range(source, 0, 6),
            "Struct of Mode",
            TypeHeadInput::Symbol(symbol_id("Struct/0", "pkg::main::Struct/0")),
        )
        .with_args(vec![TypeExpressionInput::new(
            site(1),
            range(source, 7, 11),
            "Mode",
            TypeHeadInput::Symbol(mode),
        )])
        .with_attributes(vec![AttributeInput::new(
            wrong_attribute,
            AttributePolarity::Positive,
            range(source, 12, 13),
            "Wrong",
        )]);
        let builtin_with_arg = TypeExpressionInput::new(
            site(2),
            range(source, 14, 20),
            "set of Mode",
            TypeHeadInput::BuiltinSet,
        )
        .with_args(vec![TypeExpressionInput::new(
            site(3),
            range(source, 21, 25),
            "Mode",
            TypeHeadInput::Symbol(symbol_id("Mode/0", "pkg::main::Mode/0")),
        )]);

        let output = TypeNormalizer::default().normalize(&symbols, [input, builtin_with_arg]);

        assert!(
            output
                .debug_text()
                .contains("checker.type.wrong_structure_arity")
        );
        assert!(
            output
                .debug_text()
                .contains("checker.type.wrong_attribute_kind")
        );
        assert!(
            output
                .debug_text()
                .contains("checker.type.wrong_builtin_arity")
        );
        assert_eq!(
            diagnostic_ranges(&output, "checker.type.wrong_structure_arity"),
            vec![(0, 6)]
        );
        assert_eq!(
            diagnostic_ranges(&output, "checker.type.wrong_attribute_kind"),
            vec![(12, 13)]
        );
        assert_eq!(
            diagnostic_ranges(&output, "checker.type.wrong_builtin_arity"),
            vec![(14, 20)]
        );
    }

    #[test]
    fn duplicate_type_expression_sites_are_diagnosed_without_duplicate_entries() {
        let source = source_id();
        let symbols = symbol_env(Vec::new());
        let first = TypeExpressionInput::new(
            site(0),
            range(source, 0, 3),
            "set",
            TypeHeadInput::BuiltinSet,
        );
        let duplicate = TypeExpressionInput::new(
            site(0),
            range(source, 4, 7),
            "object",
            TypeHeadInput::BuiltinObject,
        );

        let output = TypeNormalizer::default().normalize(&symbols, [first, duplicate]);

        assert_eq!(output.type_entries().len(), 1);
        assert_eq!(
            diagnostic_ranges(&output, "checker.type.duplicate_site"),
            vec![(4, 7)]
        );
    }

    trait HasDiagnostics {
        fn diagnostics(&self) -> &TypeDiagnosticTable;
    }

    impl HasDiagnostics for TypeNormalizationOutput {
        fn diagnostics(&self) -> &TypeDiagnosticTable {
            self.diagnostics()
        }
    }

    impl HasDiagnostics for DeclarationCheckingOutput {
        fn diagnostics(&self) -> &TypeDiagnosticTable {
            self.diagnostics()
        }
    }

    impl HasDiagnostics for TermFormulaInferenceOutput {
        fn diagnostics(&self) -> &TypeDiagnosticTable {
            self.diagnostics()
        }
    }

    fn declarations_by_binding(
        output: &DeclarationCheckingOutput,
    ) -> BTreeMap<BindingId, &CheckedDeclaration> {
        output
            .declarations()
            .iter()
            .map(|(_, declaration)| (declaration.binding, declaration))
            .collect()
    }

    fn term_by_site(output: &TermFormulaInferenceOutput, site: TypedSiteRef) -> &CheckedTerm {
        output
            .terms()
            .iter()
            .map(|(_, term)| term)
            .find(|term| term.site == site)
            .unwrap()
    }

    fn formula_by_site(output: &TermFormulaInferenceOutput, site: TypedSiteRef) -> &CheckedFormula {
        output
            .formulas()
            .iter()
            .map(|(_, formula)| formula)
            .find(|formula| formula.site == site)
            .unwrap()
    }

    fn diagnostic_ranges(output: &impl HasDiagnostics, message_key: &str) -> Vec<(usize, usize)> {
        output
            .diagnostics()
            .iter()
            .filter_map(|(_, diagnostic)| {
                (diagnostic.message_key == message_key)
                    .then_some((diagnostic.source_range.start, diagnostic.source_range.end))
            })
            .collect()
    }

    #[derive(Debug, Clone, Copy)]
    struct BindingSpec {
        spelling: &'static str,
        kind: BindingKind,
        status: BindingStatus,
    }

    fn binding_spec(
        spelling: &'static str,
        kind: BindingKind,
        status: BindingStatus,
    ) -> BindingSpec {
        BindingSpec {
            spelling,
            kind,
            status,
        }
    }

    fn binding_env_for_declarations(source: SourceId, specs: Vec<BindingSpec>) -> BindingEnv {
        let reserved = specs
            .iter()
            .enumerate()
            .filter_map(|(index, spec)| {
                (spec.kind == BindingKind::ReservedVariable).then_some(BindingId::new(index))
            })
            .collect::<Vec<_>>();
        let local = specs
            .iter()
            .enumerate()
            .filter_map(|(index, spec)| {
                (spec.kind != BindingKind::ReservedVariable).then_some(BindingId::new(index))
            })
            .collect::<Vec<_>>();
        let mut visible = reserved.clone();
        visible.extend(local.iter().copied());

        let mut contexts = BindingContextTable::new();
        contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::Module,
            parent: None,
            layer: BindingContextLayer::Module,
            lexical_scope: None,
            bindings: reserved.clone(),
            visible_bindings: reserved,
            recovery: BindingContextRecovery::Normal,
        });
        contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::Generated("block".to_owned()),
            parent: Some(BindingContextId::new(0)),
            layer: BindingContextLayer::Block,
            lexical_scope: Some(LocalTermScope::new(vec![1])),
            bindings: local,
            visible_bindings: visible,
            recovery: BindingContextRecovery::Normal,
        });

        let mut bindings = BindingTable::new();
        for (index, spec) in specs.into_iter().enumerate() {
            let owner_context = if spec.kind == BindingKind::ReservedVariable {
                BindingContextId::new(0)
            } else {
                BindingContextId::new(1)
            };
            bindings.insert(binding_draft_for_spec(source, index, owner_context, spec));
        }

        BindingEnv::try_new(BindingEnvParts {
            source_id: source,
            module_id: module_id(),
            contexts,
            bindings,
            diagnostics: BindingDiagnosticTable::new(),
        })
        .unwrap()
    }

    fn binding_env_with_outer_visible_binding(source: SourceId) -> BindingEnv {
        let mut contexts = BindingContextTable::new();
        contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::Module,
            parent: None,
            layer: BindingContextLayer::Module,
            lexical_scope: None,
            bindings: vec![BindingId::new(0)],
            visible_bindings: vec![BindingId::new(0)],
            recovery: BindingContextRecovery::Normal,
        });
        contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::Generated("block".to_owned()),
            parent: Some(BindingContextId::new(0)),
            layer: BindingContextLayer::Block,
            lexical_scope: Some(LocalTermScope::new(vec![1])),
            bindings: Vec::new(),
            visible_bindings: vec![BindingId::new(0)],
            recovery: BindingContextRecovery::Normal,
        });

        let mut bindings = BindingTable::new();
        bindings.insert(binding_draft_for_spec(
            source,
            0,
            BindingContextId::new(0),
            binding_spec("let_x", BindingKind::LetBinding, BindingStatus::Active),
        ));

        BindingEnv::try_new(BindingEnvParts {
            source_id: source,
            module_id: module_id(),
            contexts,
            bindings,
            diagnostics: BindingDiagnosticTable::new(),
        })
        .unwrap()
    }

    fn binding_draft_for_spec(
        source: SourceId,
        index: usize,
        owner_context: BindingContextId,
        spec: BindingSpec,
    ) -> BindingDraft {
        let declaration_range = range(source, index * 10, index * 10 + spec.spelling.len());
        let identity = match spec.kind {
            BindingKind::ReservedVariable => BinderIdentity::ReservedVariable {
                spelling: spec.spelling.to_owned(),
                declaration_range,
            },
            BindingKind::Generated => BinderIdentity::Generated {
                context: owner_context,
                counter: index as u32,
            },
            _ => BinderIdentity::ResolverLocal {
                scope: LocalTermScope::new(vec![1]),
                ordinal: index,
                declaration_range,
            },
        };
        BindingDraft {
            spelling: spec.spelling.to_owned(),
            kind: spec.kind,
            identity,
            owner_context,
            declaration_range,
            visible_after_ordinal: index,
            type_site: BindingTypeSite::Source(declaration_range),
            status: spec.status,
            captured: CapturedFreeVariables::default(),
            diagnostics: Vec::new(),
            recovery: BindingRecoveryState::Normal,
        }
    }

    fn declaration_with_type(
        source: SourceId,
        binding_index: usize,
        kind: DeclarationKind,
        declaration_node: usize,
        type_node: usize,
    ) -> DeclarationInput {
        declaration_with_type_in_context(
            source,
            binding_index,
            BindingContextId::new(1),
            kind,
            declaration_node,
            type_node,
        )
    }

    fn declaration_with_type_in_context(
        source: SourceId,
        binding_index: usize,
        context: BindingContextId,
        kind: DeclarationKind,
        declaration_node: usize,
        type_node: usize,
    ) -> DeclarationInput {
        DeclarationInput::new(
            BindingId::new(binding_index),
            context,
            site(declaration_node),
            range(source, declaration_node * 5, declaration_node * 5 + 5),
            kind,
        )
        .with_type_expression(TypeExpressionInput::new(
            site(type_node),
            range(source, type_node * 5, type_node * 5 + 3),
            "set",
            TypeHeadInput::BuiltinSet,
        ))
    }

    fn term_with_type(
        source: SourceId,
        term_node: usize,
        kind: TermKind,
        type_node: usize,
    ) -> TermInput {
        TermInput::new(
            site(term_node),
            BindingContextId::new(1),
            range(source, term_node * 5, term_node * 5 + 5),
            kind,
        )
        .with_result_type(type_expression(
            source,
            type_node,
            TypeHeadInput::BuiltinSet,
        ))
    }

    fn type_expression(
        source: SourceId,
        type_node: usize,
        head: TypeHeadInput,
    ) -> TypeExpressionInput {
        TypeExpressionInput::new(
            site(type_node),
            range(source, type_node * 5, type_node * 5 + 3),
            "set",
            head,
        )
    }

    fn symbol_env(entries: Vec<SymbolEntry>) -> SymbolEnv {
        let mut symbols = SymbolIndex::new();
        for entry in entries {
            symbols.insert(entry);
        }
        let mut contributions = SourceContributionIndex::new();
        let source = source_id();
        contributions.insert(
            module_id(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 1)),
        );
        SymbolEnv::new(
            module_id(),
            SymbolEnvIndexes {
                imports: ResolvedImportIndex::new(),
                exports: ResolvedExportIndex::new(),
                symbols,
                labels: LabelIndex::new(),
                definitions: DefinitionIndex::new(),
                overloads: OverloadIndex::new(),
                registrations: RegistrationIndex::new(),
                lexical_summaries: ModuleLexicalSummaryIndex::new(),
                namespace_graph: NamespaceGraph::new(),
                declaration_dependencies: Default::default(),
                contributions,
                module_summaries: ModuleSummaryIndex::new(),
            },
        )
    }

    fn symbol_entry(symbol: SymbolId, kind: SymbolKind) -> SymbolEntry {
        let source = source_id();
        let mut contributions = SourceContributionIndex::new();
        let contribution = contributions.insert(
            module_id(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 1)),
        );
        SymbolEntry::new(
            symbol,
            kind,
            NamespacePath::new("main"),
            "symbol",
            SemanticOrigin::new(
                source,
                module_id(),
                SourceAnchor::Range(range(source, 0, 1)),
                Vec::new(),
            ),
            contribution,
        )
        .with_visibility(Visibility::Public)
        .with_export_status(ExportStatus::Exported)
    }

    fn site(index: usize) -> TypedSiteRef {
        TypedSiteRef::Role {
            node: TypedNodeId::new(index),
            role: TypeRole::new("type"),
        }
    }

    fn source_id() -> SourceId {
        source_ids_pair().0
    }

    fn source_ids_pair() -> (SourceId, SourceId) {
        let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{}",
            "17".repeat(32)
        ))
        .unwrap();
        let allocator = InMemorySessionIdAllocator::new();
        (
            allocator.next_source_id(snapshot).unwrap(),
            allocator.next_source_id(snapshot).unwrap(),
        )
    }

    fn range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
        SourceRange {
            source_id,
            start,
            end,
        }
    }

    fn module_id() -> ModuleId {
        ModuleId::new(PackageId::new("pkg"), ModulePath::new("main"))
    }

    fn symbol_id(local: &str, fqn: &str) -> SymbolId {
        SymbolId::new(
            module_id(),
            LocalSymbolId::new(local),
            FullyQualifiedName::new(fqn),
        )
    }
}
