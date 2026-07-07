//! Type-expression normalization and declaration checking for checker phase 6.

use crate::{
    binding_env::{
        BinderIdentity, BindingContextDraft, BindingContextId, BindingContextLayer,
        BindingContextRecovery, BindingContextTable, BindingDiagnosticTable, BindingDraft,
        BindingEnv, BindingEnvParts, BindingId, BindingKind, BindingRecoveryState, BindingStatus,
        BindingTable, BindingTypeSite, CapturedFreeVariables,
    },
    typed_ast::{
        BindingTypeRef, BuiltinRuleId, CoercionDraft, CoercionKind, CoercionProvenance,
        CoercionStatus, CoercionTable, ContextRecoveryState, DiagnosticRecoveryState,
        FactProvenance, FactStatus, InitialObligationDraft, InitialObligationGoal,
        InitialObligationId, InitialObligationKind, InitialObligationProvenance,
        InitialObligationStatus, InitialObligationTable, LocalTypeContextDraft, LocalTypeContextId,
        LocalTypeContextTable, NormalizedTypeId, OpenCandidateSetId, Polarity,
        SourceRangeKey as TypedSourceRangeKey, TypeAssumptionId, TypeContextLayer, TypeDiagnostic,
        TypeDiagnosticClass, TypeDiagnosticDraft, TypeDiagnosticId, TypeDiagnosticSeverity,
        TypeDiagnosticTable, TypeEntryActual, TypeEntryDraft, TypeEntryId, TypeFactDraft,
        TypeFactId, TypeFactTable, TypePredicateRef, TypeProvenance, TypeRuleId, TypeStatus,
        TypeTable, TypedNodeId, TypedSiteRef, TypedSubjectRef,
    },
};
use mizar_resolve::{
    env::{ContributionKind, SymbolEnv, SymbolKind},
    resolved_ast::{ModuleId, SymbolId},
};
use mizar_session::{SourceId, SourceRange};
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
pub struct CoercionCheckingOutput {
    normalized_types: NormalizedTypeTable,
    type_entries: TypeTable,
    coercions: CoercionTable,
    initial_obligations: InitialObligationTable,
    facts: TypeFactTable,
    diagnostics: TypeDiagnosticTable,
}

impl CoercionCheckingOutput {
    pub const fn normalized_types(&self) -> &NormalizedTypeTable {
        &self.normalized_types
    }

    pub const fn type_entries(&self) -> &TypeTable {
        &self.type_entries
    }

    pub const fn coercions(&self) -> &CoercionTable {
        &self.coercions
    }

    pub const fn initial_obligations(&self) -> &InitialObligationTable {
        &self.initial_obligations
    }

    pub const fn facts(&self) -> &TypeFactTable {
        &self.facts
    }

    pub const fn diagnostics(&self) -> &TypeDiagnosticTable {
        &self.diagnostics
    }

    pub fn debug_text(&self) -> String {
        let mut output = String::from("coercion-checking-debug-v1\n");
        write_normalized_types(&mut output, &self.normalized_types);
        write_type_entries(&mut output, &self.type_entries);
        write_coercions(&mut output, &self.coercions);
        write_initial_obligations(&mut output, &self.initial_obligations);
        write_type_facts(&mut output, &self.facts);
        write_diagnostics(&mut output, &self.diagnostics);
        output
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CoercionObligationChecker {
    normalizer: TypeNormalizer,
}

impl CoercionObligationChecker {
    pub fn new(normalizer: TypeNormalizer) -> Self {
        Self { normalizer }
    }

    pub fn check(
        &self,
        symbols: &SymbolEnv,
        input_facts: &TypeFactTable,
        coercion_inputs: impl IntoIterator<Item = CoercionInput>,
        obligation_inputs: impl IntoIterator<Item = InitialObligationInput>,
    ) -> CoercionCheckingOutput {
        let mut inputs = Vec::new();
        inputs.extend(
            obligation_inputs
                .into_iter()
                .map(|obligation| CoercionCheckingInput::Obligation(Box::new(obligation))),
        );
        inputs.extend(
            coercion_inputs
                .into_iter()
                .map(|coercion| CoercionCheckingInput::Coercion(Box::new(coercion))),
        );
        inputs.sort_by_key(coercion_checking_input_key);

        let mut type_inputs = Vec::new();
        for input in &inputs {
            input.collect_type_inputs(&mut type_inputs);
        }
        let normalized = self.normalizer.normalize(symbols, type_inputs);
        let type_entries_by_site = type_entries_by_site(normalized.type_entries());

        let mut state = CoercionCheckingState {
            input_facts,
            normalized_types: normalized.normalized_types,
            type_entries: normalized.type_entries,
            coercions: CoercionTable::new(),
            initial_obligations: InitialObligationTable::new(),
            facts: input_facts.clone(),
            diagnostics: normalized.diagnostics,
        };

        for input in inputs {
            match input {
                CoercionCheckingInput::Obligation(obligation) => {
                    state.check_initial_obligation(*obligation, &type_entries_by_site);
                }
                CoercionCheckingInput::Coercion(coercion) => {
                    state.check_coercion(*coercion, &type_entries_by_site);
                }
            }
        }

        CoercionCheckingOutput {
            normalized_types: state.normalized_types,
            type_entries: state.type_entries,
            coercions: state.coercions,
            initial_obligations: state.initial_obligations,
            facts: state.facts,
            diagnostics: state.diagnostics,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoercionInput {
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub kind: CoercionRequestKind,
    pub justification: CoercionJustification,
    pub evidence: CoercionEvidence,
    pub from_type: Option<TypeExpressionInput>,
    pub to_type: TypeExpressionInput,
    pub supporting_facts: Vec<TypeFactId>,
    pub obligation: Option<InitialObligationInput>,
    pub deferred: Vec<CoercionDeferredReason>,
}

impl CoercionInput {
    pub fn new(
        site: TypedSiteRef,
        source_range: SourceRange,
        kind: CoercionRequestKind,
        to_type: TypeExpressionInput,
    ) -> Self {
        Self {
            site,
            source_range,
            kind,
            justification: CoercionJustification::Explicit,
            evidence: CoercionEvidence::Missing,
            from_type: None,
            to_type,
            supporting_facts: Vec::new(),
            obligation: None,
            deferred: Vec::new(),
        }
    }

    pub fn with_from_type(mut self, from_type: TypeExpressionInput) -> Self {
        self.from_type = Some(from_type);
        self
    }

    pub const fn with_evidence(mut self, evidence: CoercionEvidence) -> Self {
        self.evidence = evidence;
        self
    }

    pub const fn with_justification(mut self, justification: CoercionJustification) -> Self {
        self.justification = justification;
        self
    }

    pub fn with_supporting_facts(mut self, supporting_facts: Vec<TypeFactId>) -> Self {
        self.supporting_facts = supporting_facts;
        self
    }

    pub fn with_obligation(mut self, obligation: InitialObligationInput) -> Self {
        self.obligation = Some(obligation);
        self
    }

    pub fn with_deferred(mut self, deferred: Vec<CoercionDeferredReason>) -> Self {
        self.deferred = deferred;
        self
    }

    fn collect_type_inputs(&self, output: &mut Vec<TypeExpressionInput>) {
        if let Some(from_type) = &self.from_type {
            output.push(from_type.clone());
        }
        output.push(self.to_type.clone());
        if let Some(obligation) = &self.obligation {
            obligation.collect_type_inputs(output);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum CoercionRequestKind {
    Widening,
    Narrowing,
    SourceQua,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum CoercionJustification {
    Explicit,
    Omitted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum CoercionEvidence {
    KnownFacts,
    BuiltinRadix,
    StructureInheritance,
    ActivatedSummary,
    StaticUpcast,
    CompatibleView,
    Missing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum CoercionDeferredReason {
    MissingWideningSupportPayload,
    MissingSourceQuaEvidencePayload,
    MissingInheritancePayload,
    MissingClusterEvidencePayload,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitialObligationInput {
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub kind: InitialRequirementKind,
    pub target_type: TypeExpressionInput,
    pub assumptions: Vec<TypeFactId>,
    pub goal: Option<InitialObligationGoal>,
    pub provenance: Option<InitialObligationProvenance>,
}

impl InitialObligationInput {
    pub fn new(
        site: TypedSiteRef,
        source_range: SourceRange,
        kind: InitialRequirementKind,
        target_type: TypeExpressionInput,
    ) -> Self {
        Self {
            site,
            source_range,
            kind,
            target_type,
            assumptions: Vec::new(),
            goal: None,
            provenance: None,
        }
    }

    pub fn with_assumptions(mut self, assumptions: Vec<TypeFactId>) -> Self {
        self.assumptions = assumptions;
        self
    }

    pub fn with_goal(mut self, goal: InitialObligationGoal) -> Self {
        self.goal = Some(goal);
        self
    }

    pub fn with_provenance(mut self, provenance: InitialObligationProvenance) -> Self {
        self.provenance = Some(provenance);
        self
    }

    fn collect_type_inputs(&self, output: &mut Vec<TypeExpressionInput>) {
        output.push(self.target_type.clone());
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum InitialRequirementKind {
    Sethood,
    NonEmptiness,
    Narrowing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypeFactQueryEngine<'a> {
    facts: &'a TypeFactTable,
    contexts: Option<&'a LocalTypeContextTable>,
}

impl<'a> TypeFactQueryEngine<'a> {
    pub const fn new(facts: &'a TypeFactTable) -> Self {
        Self {
            facts,
            contexts: None,
        }
    }

    pub const fn with_contexts(
        facts: &'a TypeFactTable,
        contexts: &'a LocalTypeContextTable,
    ) -> Self {
        Self {
            facts,
            contexts: Some(contexts),
        }
    }

    pub fn query(&self, query: TypeFactQuery) -> TypeFactQueryOutput {
        let mut matched = Vec::new();
        let mut opposite = Vec::new();
        let mut active_same_predicate = Vec::new();
        let mut diagnostics = TypeDiagnosticTable::new();
        for (id, fact) in self.facts.canonical_iter() {
            if fact.subject != query.subject || fact.predicate != query.predicate {
                continue;
            }
            if !self.fact_is_active(id, fact, query.context) {
                continue;
            }
            active_same_predicate.push(id);
            if fact.polarity == query.polarity {
                matched.push(id);
            } else {
                opposite.push(id);
            }
        }

        let status = if !matched.is_empty() && opposite.is_empty() {
            TypeFactQueryStatus::Satisfied
        } else if matched.is_empty() && opposite.is_empty() {
            TypeFactQueryStatus::Missing
        } else {
            diagnostics.insert(TypeDiagnosticDraft {
                owner: Some(query.subject.clone()),
                source_range: query.source_range,
                class: TypeDiagnosticClass::TypeFact,
                severity: TypeDiagnosticSeverity::Error,
                message_key: "checker.fact.contradiction".to_owned(),
                recovery: DiagnosticRecoveryState::Degraded,
            });
            matched = active_same_predicate;
            TypeFactQueryStatus::Contradicted
        };

        TypeFactQueryOutput {
            query,
            status,
            matched_facts: matched,
            diagnostics,
        }
    }

    pub fn active_facts(&self, context: Option<LocalTypeContextId>) -> Vec<TypeFactId> {
        self.facts
            .canonical_iter()
            .filter_map(|(id, fact)| self.fact_is_active(id, fact, context).then_some(id))
            .collect()
    }

    fn fact_is_active(
        &self,
        id: TypeFactId,
        fact: &crate::typed_ast::TypeFact,
        context: Option<LocalTypeContextId>,
    ) -> bool {
        match fact.status {
            FactStatus::Known => true,
            FactStatus::Assumed => context.is_some_and(|context| {
                self.contexts.is_some_and(|contexts| {
                    context_can_consume_fact(context, id, contexts, self.facts)
                })
            }),
            FactStatus::PendingObligation | FactStatus::Degraded | FactStatus::Rejected => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeFactQuery {
    pub subject: TypedSubjectRef,
    pub predicate: TypePredicateRef,
    pub polarity: Polarity,
    pub context: Option<LocalTypeContextId>,
    pub source_range: SourceRange,
}

impl TypeFactQuery {
    pub const fn new(
        subject: TypedSubjectRef,
        predicate: TypePredicateRef,
        polarity: Polarity,
        source_range: SourceRange,
    ) -> Self {
        Self {
            subject,
            predicate,
            polarity,
            context: None,
            source_range,
        }
    }

    pub const fn with_context(mut self, context: LocalTypeContextId) -> Self {
        self.context = Some(context);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeFactQueryOutput {
    query: TypeFactQuery,
    status: TypeFactQueryStatus,
    matched_facts: Vec<TypeFactId>,
    diagnostics: TypeDiagnosticTable,
}

impl TypeFactQueryOutput {
    pub const fn query(&self) -> &TypeFactQuery {
        &self.query
    }

    pub const fn status(&self) -> TypeFactQueryStatus {
        self.status
    }

    pub fn matched_facts(&self) -> &[TypeFactId] {
        &self.matched_facts
    }

    pub const fn diagnostics(&self) -> &TypeDiagnosticTable {
        &self.diagnostics
    }

    pub fn debug_text(&self) -> String {
        let mut output = String::from("type-fact-query-debug-v1\n");
        write_type_fact_query(&mut output, &self.query);
        let _ = write!(
            output,
            "status={} matched=",
            type_fact_query_status_name(self.status)
        );
        write_fact_ids(&mut output, &self.matched_facts);
        output.push('\n');
        write_diagnostics(&mut output, &self.diagnostics);
        output
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum TypeFactQueryStatus {
    Satisfied,
    Missing,
    Contradicted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CoercionCheckingInput {
    Obligation(Box<InitialObligationInput>),
    Coercion(Box<CoercionInput>),
}

impl CoercionCheckingInput {
    fn collect_type_inputs(&self, output: &mut Vec<TypeExpressionInput>) {
        match self {
            Self::Obligation(obligation) => obligation.collect_type_inputs(output),
            Self::Coercion(coercion) => coercion.collect_type_inputs(output),
        }
    }
}

struct CoercionCheckingState<'a> {
    input_facts: &'a TypeFactTable,
    normalized_types: NormalizedTypeTable,
    type_entries: TypeTable,
    coercions: CoercionTable,
    initial_obligations: InitialObligationTable,
    facts: TypeFactTable,
    diagnostics: TypeDiagnosticTable,
}

impl CoercionCheckingState<'_> {
    fn check_coercion(
        &mut self,
        input: CoercionInput,
        type_entries_by_site: &BTreeMap<TypedSiteRef, (NormalizedTypeId, TypeStatus)>,
    ) {
        let mut recovery = None;
        let mut status = CoercionStatus::Candidate;

        let from = input.from_type.as_ref().and_then(|from_type| {
            self.normalized_type_for_site(
                &input.site,
                from_type,
                type_entries_by_site,
                "checker.coercion.missing_from_type",
                &mut status,
                &mut recovery,
            )
        });
        let to = self.normalized_type_for_site(
            &input.site,
            &input.to_type,
            type_entries_by_site,
            "checker.coercion.missing_to_type",
            &mut status,
            &mut recovery,
        );

        let deferred = input.deferred.iter().copied().collect::<BTreeSet<_>>();
        for reason in deferred.iter().copied() {
            recovery = Some(self.coercion_deferred_diagnostic(&input, reason));
            status = merge_coercion_status(status, CoercionStatus::Blocked);
        }

        let mut supporting_facts =
            self.consumable_supporting_facts(&input, &mut status, &mut recovery);
        if input.kind == CoercionRequestKind::Widening
            && input.evidence == CoercionEvidence::BuiltinRadix
        {
            let fact = self.insert_builtin_widening_fact(&input);
            let builtin_fact_is_consumable = self
                .facts
                .get(fact)
                .is_some_and(|entry| entry.status.is_unconditionally_consumable());
            if builtin_fact_is_consumable {
                if !supporting_facts.contains(&fact) {
                    supporting_facts.push(fact);
                }
            } else {
                recovery = Some(self.diagnostic(
                    Some(input.site.clone()),
                    input.source_range,
                    TypeDiagnosticClass::Coercion,
                    TypeDiagnosticSeverity::Error,
                    "checker.coercion.builtin_widening_fact_not_consumable",
                    DiagnosticRecoveryState::Degraded,
                ));
                status = merge_coercion_status(status, CoercionStatus::Blocked);
            }
        }
        let mut obligation = None;
        if to.is_some() {
            match input.kind {
                CoercionRequestKind::Widening => {
                    if !widening_evidence_is_available(input.evidence, !supporting_facts.is_empty())
                        && !deferred
                            .contains(&CoercionDeferredReason::MissingWideningSupportPayload)
                    {
                        recovery = Some(self.diagnostic(
                            Some(input.site.clone()),
                            input.source_range,
                            TypeDiagnosticClass::Coercion,
                            TypeDiagnosticSeverity::Note,
                            "checker.coercion.external.widening_support_payload",
                            DiagnosticRecoveryState::Degraded,
                        ));
                        status = merge_coercion_status(status, CoercionStatus::Blocked);
                    }
                }
                CoercionRequestKind::SourceQua => {
                    if !source_qua_evidence_is_available(input.evidence) {
                        if deferred
                            .contains(&CoercionDeferredReason::MissingSourceQuaEvidencePayload)
                        {
                            status = merge_coercion_status(status, CoercionStatus::Blocked);
                        } else {
                            recovery = Some(self.diagnostic(
                                Some(input.site.clone()),
                                input.source_range,
                                TypeDiagnosticClass::Coercion,
                                TypeDiagnosticSeverity::Error,
                                "checker.coercion.invalid_source_qua_target",
                                DiagnosticRecoveryState::Degraded,
                            ));
                            status = merge_coercion_status(status, CoercionStatus::Rejected);
                        }
                    }
                }
                CoercionRequestKind::Narrowing => match input.justification {
                    CoercionJustification::Explicit => {
                        if !explicit_narrowing_evidence_is_available(
                            input.evidence,
                            !supporting_facts.is_empty(),
                        ) {
                            let obligation_input = input.obligation.clone().unwrap_or_else(|| {
                                InitialObligationInput::new(
                                    input.site.clone(),
                                    input.source_range,
                                    InitialRequirementKind::Narrowing,
                                    input.to_type.clone(),
                                )
                                .with_assumptions(input.supporting_facts.clone())
                            });
                            let (obligation_id, obligation_status) = self
                                .insert_initial_obligation(obligation_input, type_entries_by_site);
                            obligation = Some(obligation_id);
                            let obligation_coercion_status =
                                if obligation_status == InitialObligationStatus::Pending {
                                    CoercionStatus::RequiresObligation
                                } else {
                                    CoercionStatus::Blocked
                                };
                            status = merge_coercion_status(status, obligation_coercion_status);
                        }
                    }
                    CoercionJustification::Omitted => {
                        if !proof_free_reconsider_evidence_is_available(
                            input.evidence,
                            !supporting_facts.is_empty(),
                        ) {
                            recovery = Some(self.diagnostic(
                                Some(input.site.clone()),
                                input.source_range,
                                TypeDiagnosticClass::Coercion,
                                TypeDiagnosticSeverity::Error,
                                "type.narrowing_requires_proof",
                                DiagnosticRecoveryState::Degraded,
                            ));
                            status = merge_coercion_status(status, CoercionStatus::Rejected);
                        }
                    }
                },
            }
        } else {
            status = merge_coercion_status(status, CoercionStatus::Blocked);
        }

        let Some(to) = to else {
            let recovery = recovery.expect("missing target type creates diagnostic");
            let to = self.recovery_normalized_type(input.source_range);
            self.insert_recovered_coercion(input, from, to, supporting_facts, recovery, status);
            return;
        };

        let provenance = match (input.kind, recovery) {
            (_, Some(diagnostic)) => CoercionProvenance::Recovery(diagnostic),
            (CoercionRequestKind::Widening, None) => {
                CoercionProvenance::WideningRule(TypeRuleId::new(format!(
                    "coercion-widening-{}",
                    coercion_evidence_name(input.evidence)
                )))
            }
            (CoercionRequestKind::Narrowing, None) => {
                CoercionProvenance::NarrowingClaim(TypedSourceRangeKey::from(input.source_range))
            }
            (CoercionRequestKind::SourceQua, None) => {
                CoercionProvenance::SourceQua(TypedSourceRangeKey::from(input.source_range))
            }
        };

        self.coercions.insert(CoercionDraft {
            site: input.site,
            from,
            to,
            kind: coercion_kind(input.kind),
            status,
            supporting_facts,
            obligation,
            provenance,
        });
    }

    fn check_initial_obligation(
        &mut self,
        input: InitialObligationInput,
        type_entries_by_site: &BTreeMap<TypedSiteRef, (NormalizedTypeId, TypeStatus)>,
    ) {
        let _ = self.insert_initial_obligation(input, type_entries_by_site);
    }

    fn insert_initial_obligation(
        &mut self,
        input: InitialObligationInput,
        type_entries_by_site: &BTreeMap<TypedSiteRef, (NormalizedTypeId, TypeStatus)>,
    ) -> (InitialObligationId, InitialObligationStatus) {
        let mut status = InitialObligationStatus::Pending;
        let mut recovery = None;
        let target = self.normalized_type_for_obligation(
            &input,
            type_entries_by_site,
            &mut status,
            &mut recovery,
        );
        let assumptions =
            self.consumable_obligation_assumptions(&input, &mut status, &mut recovery);
        let goal = input.goal.unwrap_or_else(|| {
            InitialObligationGoal::new(format!(
                "{}:{}",
                initial_requirement_kind_name(input.kind),
                target
                    .map(|id| id.index().to_string())
                    .unwrap_or_else(|| "error".to_owned())
            ))
        });
        let provenance = input.provenance.unwrap_or_else(|| {
            InitialObligationProvenance::new(format!(
                "checker.obligation.{}",
                initial_requirement_kind_name(input.kind)
            ))
        });
        if target.is_none() && status == InitialObligationStatus::Pending {
            status = merge_initial_obligation_status(status, InitialObligationStatus::Blocked);
        }
        let id = self.initial_obligations.insert(InitialObligationDraft {
            kind: initial_obligation_kind(input.kind),
            owner: input.site.clone(),
            source_range: input.source_range,
            assumptions,
            goal,
            provenance,
            status,
        });
        self.facts.insert(TypeFactDraft {
            subject: input.site,
            predicate: TypePredicateRef::new(format!(
                "obligation.{}",
                initial_requirement_kind_name(input.kind)
            )),
            polarity: Polarity::Positive,
            provenance: FactProvenance::Obligation(id),
            status: if status == InitialObligationStatus::Pending {
                FactStatus::PendingObligation
            } else {
                FactStatus::Degraded
            },
        });
        let _ = recovery;
        (id, status)
    }

    fn normalized_type_for_site(
        &mut self,
        owner: &TypedSiteRef,
        input: &TypeExpressionInput,
        type_entries_by_site: &BTreeMap<TypedSiteRef, (NormalizedTypeId, TypeStatus)>,
        message_key: &str,
        status: &mut CoercionStatus,
        recovery: &mut Option<TypeDiagnosticId>,
    ) -> Option<NormalizedTypeId> {
        match type_entries_by_site.get(&input.site) {
            Some((id, type_status)) => {
                if *type_status != TypeStatus::Known {
                    *status = merge_coercion_status(*status, CoercionStatus::Blocked);
                }
                Some(*id)
            }
            None => {
                *recovery = Some(self.diagnostic(
                    Some(owner.clone()),
                    input.source_range,
                    TypeDiagnosticClass::Coercion,
                    TypeDiagnosticSeverity::Error,
                    message_key,
                    DiagnosticRecoveryState::Degraded,
                ));
                *status = merge_coercion_status(*status, CoercionStatus::Blocked);
                None
            }
        }
    }

    fn normalized_type_for_obligation(
        &mut self,
        input: &InitialObligationInput,
        type_entries_by_site: &BTreeMap<TypedSiteRef, (NormalizedTypeId, TypeStatus)>,
        status: &mut InitialObligationStatus,
        recovery: &mut Option<TypeDiagnosticId>,
    ) -> Option<NormalizedTypeId> {
        match type_entries_by_site.get(&input.target_type.site) {
            Some((id, type_status)) => {
                if *type_status != TypeStatus::Known {
                    *status =
                        merge_initial_obligation_status(*status, InitialObligationStatus::Blocked);
                }
                Some(*id)
            }
            None => {
                *recovery = Some(self.diagnostic(
                    Some(input.site.clone()),
                    input.target_type.source_range,
                    TypeDiagnosticClass::InitialObligation,
                    TypeDiagnosticSeverity::Error,
                    "checker.obligation.missing_target_type",
                    DiagnosticRecoveryState::Degraded,
                ));
                *status =
                    merge_initial_obligation_status(*status, InitialObligationStatus::Blocked);
                None
            }
        }
    }

    fn consumable_supporting_facts(
        &mut self,
        input: &CoercionInput,
        status: &mut CoercionStatus,
        recovery: &mut Option<TypeDiagnosticId>,
    ) -> Vec<TypeFactId> {
        let mut facts = Vec::new();
        for fact in &input.supporting_facts {
            match self.input_facts.get(*fact) {
                Some(entry) if entry.status.is_unconditionally_consumable() => facts.push(*fact),
                Some(_) => {
                    *recovery = Some(self.diagnostic(
                        Some(input.site.clone()),
                        input.source_range,
                        TypeDiagnosticClass::Coercion,
                        TypeDiagnosticSeverity::Error,
                        "checker.coercion.supporting_fact_not_consumable",
                        DiagnosticRecoveryState::Degraded,
                    ));
                    *status = merge_coercion_status(*status, CoercionStatus::Blocked);
                }
                None => {
                    *recovery = Some(self.diagnostic(
                        Some(input.site.clone()),
                        input.source_range,
                        TypeDiagnosticClass::Coercion,
                        TypeDiagnosticSeverity::Error,
                        "checker.coercion.unknown_supporting_fact",
                        DiagnosticRecoveryState::Degraded,
                    ));
                    *status = merge_coercion_status(*status, CoercionStatus::Blocked);
                }
            }
        }
        facts
    }

    fn consumable_obligation_assumptions(
        &mut self,
        input: &InitialObligationInput,
        status: &mut InitialObligationStatus,
        recovery: &mut Option<TypeDiagnosticId>,
    ) -> Vec<TypeFactId> {
        let mut facts = Vec::new();
        for fact in &input.assumptions {
            match self.input_facts.get(*fact) {
                Some(entry) if entry.status.is_unconditionally_consumable() => facts.push(*fact),
                Some(_) => {
                    *recovery = Some(self.diagnostic(
                        Some(input.site.clone()),
                        input.source_range,
                        TypeDiagnosticClass::InitialObligation,
                        TypeDiagnosticSeverity::Error,
                        "checker.obligation.assumption_not_consumable",
                        DiagnosticRecoveryState::Degraded,
                    ));
                    *status =
                        merge_initial_obligation_status(*status, InitialObligationStatus::Blocked);
                }
                None => {
                    *recovery = Some(self.diagnostic(
                        Some(input.site.clone()),
                        input.source_range,
                        TypeDiagnosticClass::InitialObligation,
                        TypeDiagnosticSeverity::Error,
                        "checker.obligation.unknown_assumption",
                        DiagnosticRecoveryState::Degraded,
                    ));
                    *status =
                        merge_initial_obligation_status(*status, InitialObligationStatus::Blocked);
                }
            }
        }
        facts
    }

    fn insert_recovered_coercion(
        &mut self,
        input: CoercionInput,
        from: Option<NormalizedTypeId>,
        to: NormalizedTypeId,
        supporting_facts: Vec<TypeFactId>,
        recovery: TypeDiagnosticId,
        status: CoercionStatus,
    ) {
        self.coercions.insert(CoercionDraft {
            site: input.site,
            from,
            to,
            kind: coercion_kind(input.kind),
            status,
            supporting_facts,
            obligation: None,
            provenance: CoercionProvenance::Recovery(recovery),
        });
    }

    fn insert_builtin_widening_fact(&mut self, input: &CoercionInput) -> TypeFactId {
        self.facts.insert(TypeFactDraft {
            subject: input.site.clone(),
            predicate: TypePredicateRef::new("coercion.widening.builtin_radix"),
            polarity: Polarity::Positive,
            provenance: FactProvenance::Builtin(BuiltinRuleId::new(
                "coercion-widening-builtin-radix",
            )),
            status: FactStatus::Known,
        })
    }

    fn recovery_normalized_type(&mut self, source_range: SourceRange) -> NormalizedTypeId {
        finish_type(
            &mut self.normalized_types,
            NormalizedTypeDraft {
                head: TypeHeadRef::Error(TypeHeadErrorKind::Recovery),
                args: Vec::new(),
                attributes: AttributeSet::empty(),
                source: TypeSource::new("<recovery>", source_range),
                status: NormalizedTypeStatus::Error,
            },
        )
    }

    fn coercion_deferred_diagnostic(
        &mut self,
        input: &CoercionInput,
        reason: CoercionDeferredReason,
    ) -> TypeDiagnosticId {
        self.diagnostic(
            Some(input.site.clone()),
            input.source_range,
            TypeDiagnosticClass::Coercion,
            TypeDiagnosticSeverity::Note,
            coercion_deferred_message_key(reason),
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
pub struct SourceReserveDeclarationBridge {
    source_id: SourceId,
    module_id: ModuleId,
    source_range: SourceRange,
    bindings: Vec<SourceReserveBindingInput>,
}

impl SourceReserveDeclarationBridge {
    pub fn new(
        source_id: SourceId,
        module_id: ModuleId,
        source_range: SourceRange,
        bindings: Vec<SourceReserveBindingInput>,
    ) -> Result<Self, String> {
        if bindings.is_empty() {
            return Err("source reserve bridge requires at least one binding".to_owned());
        }
        if source_range.source_id != source_id {
            return Err("source reserve bridge range uses a different source id".to_owned());
        }
        for (index, binding) in bindings.iter().enumerate() {
            binding.validate(source_id, index)?;
        }
        Ok(Self {
            source_id,
            module_id,
            source_range,
            bindings,
        })
    }

    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    pub const fn source_range(&self) -> SourceRange {
        self.source_range
    }

    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    pub fn bindings(&self) -> &[SourceReserveBindingInput] {
        &self.bindings
    }

    pub const fn module_context(&self) -> BindingContextId {
        BindingContextId::new(0)
    }

    pub fn root_node(&self) -> TypedNodeId {
        TypedNodeId::new(self.bindings.len() * 2)
    }

    pub const fn type_node(&self, index: usize) -> TypedNodeId {
        TypedNodeId::new(index * 2)
    }

    pub const fn declaration_node(&self, index: usize) -> TypedNodeId {
        TypedNodeId::new(index * 2 + 1)
    }

    pub fn check(&self, symbols: &SymbolEnv) -> Result<SourceReserveDeclarationHandoff, String> {
        self.check_with_type_normalizer(symbols, TypeNormalizer::default())
    }

    pub fn check_with_mode_expansions(
        &self,
        symbols: &SymbolEnv,
        mode_expansions: impl IntoIterator<Item = (SymbolId, ModeExpansion)>,
    ) -> Result<SourceReserveDeclarationHandoff, String> {
        self.check_with_type_normalizer(symbols, TypeNormalizer::new(mode_expansions))
    }

    fn check_with_type_normalizer(
        &self,
        symbols: &SymbolEnv,
        normalizer: TypeNormalizer,
    ) -> Result<SourceReserveDeclarationHandoff, String> {
        if symbols.module_id() != &self.module_id {
            return Err("source reserve bridge symbol environment module mismatch".to_owned());
        }
        self.validate_symbol_heads(symbols)?;
        let binding_env = self.binding_env()?;
        let context_inputs = vec![DeclarationContextInput::new(
            self.module_context(),
            TypedSiteRef::Node(self.root_node()),
            self.source_range,
        )];
        let declaration_inputs = self
            .bindings
            .iter()
            .enumerate()
            .map(|(index, binding)| {
                let type_site = TypedSiteRef::Node(self.type_node(index));
                let mut declaration = DeclarationInput::new(
                    BindingId::new(index),
                    self.module_context(),
                    TypedSiteRef::Node(self.declaration_node(index)),
                    binding.binding_range,
                    DeclarationKind::ReservedVariable,
                )
                .with_type_expression(
                    TypeExpressionInput::new(
                        type_site.clone(),
                        binding.type_range,
                        binding.type_spelling.clone(),
                        binding.type_head.clone(),
                    )
                    .with_attributes(binding.type_attributes.clone()),
                )
                .with_reserved_default(ReservedDefaultPayload::new(type_site, false));
                let symbol_head_kind = match &binding.type_head {
                    TypeHeadInput::Symbol(symbol) => {
                        symbols.symbols().get(symbol).map(|entry| entry.kind())
                    }
                    _ => None,
                };
                let symbol_head_has_mode_expansion = match &binding.type_head {
                    TypeHeadInput::Symbol(symbol)
                        if matches!(symbol_head_kind, Some(SymbolKind::Mode)) =>
                    {
                        normalizer.mode_expansions.contains_key(symbol)
                    }
                    _ => false,
                };
                let mode_expansion_requires_evidence = match &binding.type_head {
                    TypeHeadInput::Symbol(symbol)
                        if matches!(symbol_head_kind, Some(SymbolKind::Mode)) =>
                    {
                        mode_expansion_requires_evidence_query(
                            symbols,
                            &normalizer.mode_expansions,
                            symbol,
                        )
                    }
                    _ => false,
                };
                if matches!(symbol_head_kind, Some(SymbolKind::Structure))
                    || mode_expansion_requires_evidence
                    || (!binding.type_attributes.is_empty()
                        && (!matches!(symbol_head_kind, Some(SymbolKind::Mode))
                            || symbol_head_has_mode_expansion))
                {
                    declaration = declaration
                        .with_deferred(vec![DeclarationDeferredReason::MissingEvidenceQuery]);
                }
                declaration
            })
            .collect::<Vec<_>>();
        let declarations = DeclarationChecker::new(normalizer).check(
            symbols,
            &binding_env,
            context_inputs,
            declaration_inputs,
        );

        Ok(SourceReserveDeclarationHandoff {
            binding_env,
            declarations,
        })
    }

    fn validate_symbol_heads(&self, symbols: &SymbolEnv) -> Result<(), String> {
        for (index, binding) in self.bindings.iter().enumerate() {
            for (attribute_index, attribute) in binding.type_attributes.iter().enumerate() {
                if attribute.symbol.module() != &self.module_id {
                    return Err(format!(
                        "source reserve binding {index} attribute {attribute_index} is not local to the bridge module"
                    ));
                }
                let entry = symbols.symbols().get(&attribute.symbol).ok_or_else(|| {
                    format!(
                        "source reserve binding {index} attribute {attribute_index} is missing from SymbolEnv"
                    )
                })?;
                if !matches!(entry.kind(), SymbolKind::Attribute) {
                    return Err(format!(
                        "source reserve binding {index} attribute {attribute_index} is not a supported local attribute"
                    ));
                }
                let contribution =
                    symbols
                        .contributions()
                        .get(entry.contribution())
                        .ok_or_else(|| {
                            format!(
                                "source reserve binding {index} attribute {attribute_index} contribution is missing from SymbolEnv"
                            )
                        })?;
                if contribution.module() != &self.module_id
                    || !matches!(contribution.kind(), ContributionKind::LocalSource { .. })
                {
                    return Err(format!(
                        "source reserve binding {index} attribute {attribute_index} is not local source-backed"
                    ));
                }
            }
            let TypeHeadInput::Symbol(symbol) = &binding.type_head else {
                continue;
            };
            if symbol.module() != &self.module_id {
                return Err(format!(
                    "source reserve binding {index} symbol head is not local to the bridge module"
                ));
            }
            let entry = symbols.symbols().get(symbol).ok_or_else(|| {
                format!("source reserve binding {index} symbol head is missing from SymbolEnv")
            })?;
            if !matches!(entry.kind(), SymbolKind::Mode | SymbolKind::Structure) {
                return Err(format!(
                    "source reserve binding {index} symbol head is not a supported local type head"
                ));
            }
            let contribution = symbols
                .contributions()
                .get(entry.contribution())
                .ok_or_else(|| {
                    format!(
                        "source reserve binding {index} symbol head has an unknown source contribution"
                    )
                })?;
            if contribution.module() != &self.module_id
                || !matches!(contribution.kind(), ContributionKind::LocalSource { .. })
            {
                return Err(format!(
                    "source reserve binding {index} symbol head is not backed by local source"
                ));
            }
        }
        Ok(())
    }

    fn binding_env(&self) -> Result<BindingEnv, String> {
        let binding_ids = (0..self.bindings.len())
            .map(BindingId::new)
            .collect::<Vec<_>>();
        let mut contexts = BindingContextTable::new();
        contexts.insert(BindingContextDraft {
            owner: crate::binding_env::BindingContextOwner::Module,
            parent: None,
            layer: BindingContextLayer::Module,
            lexical_scope: None,
            bindings: binding_ids.clone(),
            visible_bindings: binding_ids,
            recovery: BindingContextRecovery::Normal,
        });

        let mut bindings = BindingTable::new();
        for (index, binding) in self.bindings.iter().enumerate() {
            bindings.insert(BindingDraft {
                spelling: binding.spelling.clone(),
                kind: BindingKind::ReservedVariable,
                identity: BinderIdentity::ReservedVariable {
                    spelling: binding.spelling.clone(),
                    declaration_range: binding.binding_range,
                },
                owner_context: self.module_context(),
                declaration_range: binding.binding_range,
                visible_after_ordinal: index,
                type_site: BindingTypeSite::Source(binding.type_range),
                status: BindingStatus::Reserved,
                captured: CapturedFreeVariables::default(),
                diagnostics: Vec::new(),
                recovery: BindingRecoveryState::Normal,
            });
        }

        BindingEnv::try_new(BindingEnvParts {
            source_id: self.source_id,
            module_id: self.module_id.clone(),
            contexts,
            bindings,
            diagnostics: BindingDiagnosticTable::new(),
        })
        .map_err(|error| error.to_string())
    }
}

fn mode_expansion_requires_evidence_query(
    symbols: &SymbolEnv,
    mode_expansions: &BTreeMap<SymbolId, ModeExpansion>,
    symbol: &SymbolId,
) -> bool {
    let mut visiting = BTreeSet::new();
    let mut current = symbol;
    while visiting.insert(current.clone()) {
        let Some(expansion) = mode_expansions.get(current) else {
            return false;
        };
        if !expansion.attributes.is_empty() || !expansion.radix.attributes.is_empty() {
            return true;
        }
        let TypeHeadInput::Symbol(radix) = &expansion.radix.head else {
            return false;
        };
        match symbols.symbols().get(radix).map(|entry| entry.kind()) {
            Some(SymbolKind::Structure) => return true,
            Some(SymbolKind::Mode) => current = radix,
            _ => return false,
        }
    }
    false
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceReserveBindingInput {
    pub spelling: String,
    pub binding_range: SourceRange,
    pub type_range: SourceRange,
    pub type_spelling: String,
    pub type_head: TypeHeadInput,
    pub type_attributes: Vec<AttributeInput>,
}

impl SourceReserveBindingInput {
    pub fn new(
        spelling: impl Into<String>,
        binding_range: SourceRange,
        type_range: SourceRange,
        type_spelling: impl Into<String>,
        type_head: TypeHeadInput,
    ) -> Self {
        Self {
            spelling: spelling.into(),
            binding_range,
            type_range,
            type_spelling: type_spelling.into(),
            type_head,
            type_attributes: Vec::new(),
        }
    }

    pub fn with_type_attributes(mut self, type_attributes: Vec<AttributeInput>) -> Self {
        self.type_attributes = type_attributes;
        self
    }

    fn validate(&self, source_id: SourceId, index: usize) -> Result<(), String> {
        if self.spelling.is_empty() {
            return Err(format!("source reserve binding {index} has empty spelling"));
        }
        if self.type_spelling.is_empty() {
            return Err(format!(
                "source reserve binding {index} has empty type spelling"
            ));
        }
        if self.binding_range.source_id != source_id || self.type_range.source_id != source_id {
            return Err(format!(
                "source reserve binding {index} range uses a different source id"
            ));
        }
        if !matches!(
            &self.type_head,
            TypeHeadInput::BuiltinSet | TypeHeadInput::BuiltinObject | TypeHeadInput::Symbol(_)
        ) {
            return Err(format!(
                "source reserve binding {index} is not a supported reserve type head"
            ));
        }
        for (attribute_index, attribute) in self.type_attributes.iter().enumerate() {
            if attribute.source_range.source_id != source_id {
                return Err(format!(
                    "source reserve binding {index} attribute {attribute_index} range uses a different source id"
                ));
            }
            if !attribute.args.is_empty() {
                return Err(format!(
                    "source reserve binding {index} attribute {attribute_index} has unsupported arguments"
                ));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceReserveDeclarationHandoff {
    binding_env: BindingEnv,
    declarations: DeclarationCheckingOutput,
}

impl SourceReserveDeclarationHandoff {
    pub const fn binding_env(&self) -> &BindingEnv {
        &self.binding_env
    }

    pub const fn declarations(&self) -> &DeclarationCheckingOutput {
        &self.declarations
    }

    pub fn into_parts(self) -> (BindingEnv, DeclarationCheckingOutput) {
        (self.binding_env, self.declarations)
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

type CoercionCheckingInputKey = (
    SourceRangeKey,
    String,
    u8,
    Option<CoercionRequestKind>,
    Option<InitialRequirementKind>,
    String,
    String,
    Vec<usize>,
    Option<CoercionEvidence>,
    Option<CoercionJustification>,
);

fn coercion_checking_input_key(input: &CoercionCheckingInput) -> CoercionCheckingInputKey {
    match input {
        CoercionCheckingInput::Obligation(obligation) => (
            SourceRangeKey::from(obligation.source_range),
            site_key(&obligation.site),
            0,
            None,
            Some(obligation.kind),
            type_expression_input_key(&obligation.target_type),
            String::new(),
            fact_id_indexes(&obligation.assumptions),
            None,
            None,
        ),
        CoercionCheckingInput::Coercion(coercion) => (
            SourceRangeKey::from(coercion.source_range),
            site_key(&coercion.site),
            1,
            Some(coercion.kind),
            None,
            type_expression_input_key(&coercion.to_type),
            coercion
                .from_type
                .as_ref()
                .map(type_expression_input_key)
                .unwrap_or_default(),
            fact_id_indexes(&coercion.supporting_facts),
            Some(coercion.evidence),
            Some(coercion.justification),
        ),
    }
}

fn type_expression_input_key(input: &TypeExpressionInput) -> String {
    format!(
        "{}:{}..{}:{}",
        site_key(&input.site),
        input.source_range.start,
        input.source_range.end,
        input.spelling
    )
}

fn fact_id_indexes(facts: &[TypeFactId]) -> Vec<usize> {
    facts.iter().map(|fact| fact.index()).collect()
}

fn coercion_kind(kind: CoercionRequestKind) -> CoercionKind {
    match kind {
        CoercionRequestKind::Widening => CoercionKind::Widening,
        CoercionRequestKind::Narrowing => CoercionKind::Narrowing,
        CoercionRequestKind::SourceQua => CoercionKind::SourceQua,
    }
}

fn initial_obligation_kind(kind: InitialRequirementKind) -> InitialObligationKind {
    match kind {
        InitialRequirementKind::Sethood => InitialObligationKind::Sethood,
        InitialRequirementKind::NonEmptiness => InitialObligationKind::NonEmptiness,
        InitialRequirementKind::Narrowing => InitialObligationKind::Narrowing,
    }
}

fn widening_evidence_is_available(evidence: CoercionEvidence, has_known_facts: bool) -> bool {
    match evidence {
        CoercionEvidence::KnownFacts
        | CoercionEvidence::StructureInheritance
        | CoercionEvidence::ActivatedSummary => has_known_facts,
        CoercionEvidence::BuiltinRadix => true,
        CoercionEvidence::StaticUpcast
        | CoercionEvidence::CompatibleView
        | CoercionEvidence::Missing => false,
    }
}

fn source_qua_evidence_is_available(evidence: CoercionEvidence) -> bool {
    matches!(
        evidence,
        CoercionEvidence::StaticUpcast | CoercionEvidence::CompatibleView
    )
}

fn explicit_narrowing_evidence_is_available(
    evidence: CoercionEvidence,
    has_known_facts: bool,
) -> bool {
    evidence == CoercionEvidence::KnownFacts && has_known_facts
}

fn proof_free_reconsider_evidence_is_available(
    evidence: CoercionEvidence,
    has_known_facts: bool,
) -> bool {
    has_known_facts
        && matches!(
            evidence,
            CoercionEvidence::KnownFacts
                | CoercionEvidence::BuiltinRadix
                | CoercionEvidence::StructureInheritance
                | CoercionEvidence::ActivatedSummary
                | CoercionEvidence::StaticUpcast
                | CoercionEvidence::CompatibleView
        )
}

fn merge_coercion_status(current: CoercionStatus, next: CoercionStatus) -> CoercionStatus {
    if coercion_status_rank(next) > coercion_status_rank(current) {
        next
    } else {
        current
    }
}

fn coercion_status_rank(status: CoercionStatus) -> u8 {
    match status {
        CoercionStatus::Candidate => 0,
        CoercionStatus::RequiresObligation => 1,
        CoercionStatus::Blocked => 2,
        CoercionStatus::Rejected => 3,
    }
}

fn merge_initial_obligation_status(
    current: InitialObligationStatus,
    next: InitialObligationStatus,
) -> InitialObligationStatus {
    if initial_obligation_status_rank(next) > initial_obligation_status_rank(current) {
        next
    } else {
        current
    }
}

fn initial_obligation_status_rank(status: InitialObligationStatus) -> u8 {
    match status {
        InitialObligationStatus::Pending => 0,
        InitialObligationStatus::Blocked => 1,
        InitialObligationStatus::Invalidated => 2,
    }
}

fn context_can_consume_fact(
    context_id: LocalTypeContextId,
    fact_id: TypeFactId,
    contexts: &LocalTypeContextTable,
    facts: &TypeFactTable,
) -> bool {
    let Some(fact) = facts.get(fact_id) else {
        return false;
    };
    if fact.status.is_unconditionally_consumable() {
        return true;
    }
    if fact.status != FactStatus::Assumed {
        return false;
    }

    let Some(query_context) = contexts.get(context_id) else {
        return false;
    };
    if query_context.visible_facts.binary_search(&fact_id).is_err() {
        return false;
    }

    let mut active = Some(context_id);
    while let Some(id) = active {
        let Some(context) = contexts.get(id) else {
            return false;
        };
        if context
            .introduced_assumptions
            .binary_search(&fact_id)
            .is_ok()
        {
            return true;
        }
        active = context.parent;
    }
    false
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

fn coercion_deferred_message_key(reason: CoercionDeferredReason) -> &'static str {
    match reason {
        CoercionDeferredReason::MissingWideningSupportPayload => {
            "checker.coercion.external.widening_support_payload"
        }
        CoercionDeferredReason::MissingSourceQuaEvidencePayload => {
            "checker.coercion.external.source_qua_evidence_payload"
        }
        CoercionDeferredReason::MissingInheritancePayload => {
            "checker.coercion.external.inheritance_payload"
        }
        CoercionDeferredReason::MissingClusterEvidencePayload => {
            "checker.coercion.external.cluster_evidence_payload"
        }
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

fn coercion_evidence_name(evidence: CoercionEvidence) -> &'static str {
    match evidence {
        CoercionEvidence::KnownFacts => "known_facts",
        CoercionEvidence::BuiltinRadix => "builtin_radix",
        CoercionEvidence::StructureInheritance => "structure_inheritance",
        CoercionEvidence::ActivatedSummary => "activated_summary",
        CoercionEvidence::StaticUpcast => "static_upcast",
        CoercionEvidence::CompatibleView => "compatible_view",
        CoercionEvidence::Missing => "missing",
    }
}

fn initial_requirement_kind_name(kind: InitialRequirementKind) -> &'static str {
    match kind {
        InitialRequirementKind::Sethood => "sethood",
        InitialRequirementKind::NonEmptiness => "non_emptiness",
        InitialRequirementKind::Narrowing => "narrowing",
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

fn write_coercions(output: &mut String, coercions: &CoercionTable) {
    output.push_str("coercions:\n");
    if coercions.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    for (id, coercion) in coercions.canonical_iter() {
        let _ = write!(
            output,
            "  coercion#{} site={} kind={} status={} from=",
            id.index(),
            site_key(&coercion.site),
            coercion_kind_name(coercion.kind),
            coercion_status_name(coercion.status)
        );
        write_optional_type_id(output, coercion.from);
        output.push_str(" to=");
        write_type_id(output, coercion.to);
        output.push_str(" facts=");
        write_fact_ids(output, &coercion.supporting_facts);
        output.push_str(" obligation=");
        write_optional_obligation_id(output, coercion.obligation);
        output.push_str(" provenance=");
        write_coercion_provenance(output, &coercion.provenance);
        output.push('\n');
    }
}

fn write_initial_obligations(output: &mut String, obligations: &InitialObligationTable) {
    output.push_str("initial_obligations:\n");
    if obligations.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    let mut entries = obligations
        .iter()
        .map(|(_, obligation)| obligation)
        .collect::<Vec<_>>();
    entries.sort_by_key(|obligation| {
        (
            site_key(&obligation.owner),
            obligation.kind,
            SourceRangeKey::from(obligation.source_range),
            obligation.id.index(),
        )
    });
    for obligation in entries {
        let _ = write!(
            output,
            "  obligation#{} kind={} owner={} range={}..{} assumptions=",
            obligation.id.index(),
            initial_obligation_kind_name(obligation.kind),
            site_key(&obligation.owner),
            obligation.source_range.start,
            obligation.source_range.end
        );
        write_fact_ids(output, &obligation.assumptions);
        output.push_str(" goal=\"");
        write_escaped(output, obligation.goal.as_str());
        output.push_str("\" provenance=\"");
        write_escaped(output, obligation.provenance.as_str());
        let _ = writeln!(
            output,
            "\" status={}",
            initial_obligation_status_name(obligation.status)
        );
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

fn write_type_id(output: &mut String, id: NormalizedTypeId) {
    let _ = write!(output, "normalized_type#{}", id.index());
}

fn write_optional_type_id(output: &mut String, id: Option<NormalizedTypeId>) {
    match id {
        Some(id) => write_type_id(output, id),
        None => output.push_str("<none>"),
    }
}

fn write_optional_obligation_id(output: &mut String, id: Option<InitialObligationId>) {
    match id {
        Some(id) => {
            let _ = write!(output, "obligation#{}", id.index());
        }
        None => output.push_str("<none>"),
    }
}

fn write_coercion_provenance(output: &mut String, provenance: &CoercionProvenance) {
    match provenance {
        CoercionProvenance::WideningRule(rule) => {
            output.push_str("widening_rule=\"");
            write_escaped(output, rule.as_str());
            output.push('"');
        }
        CoercionProvenance::NarrowingClaim(range) => {
            let _ = write!(output, "narrowing_claim:{}..{}", range.start, range.end);
        }
        CoercionProvenance::SourceQua(range) => {
            let _ = write!(output, "source_qua:{}..{}", range.start, range.end);
        }
        CoercionProvenance::Recovery(diagnostic) => {
            let _ = write!(output, "recovery=diagnostic#{}", diagnostic.index());
        }
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

fn write_type_fact_query(output: &mut String, query: &TypeFactQuery) {
    output.push_str("query: subject=");
    output.push_str(&site_key(&query.subject));
    output.push_str(" predicate=\"");
    write_escaped(output, query.predicate.as_str());
    let _ = write!(
        output,
        "\" polarity={} context=",
        polarity_name(query.polarity)
    );
    match query.context {
        Some(context) => {
            let _ = write!(output, "local_context#{}", context.index());
        }
        None => output.push_str("<none>"),
    }
    let _ = writeln!(
        output,
        " range={}..{}",
        query.source_range.start, query.source_range.end
    );
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

fn type_fact_query_status_name(status: TypeFactQueryStatus) -> &'static str {
    match status {
        TypeFactQueryStatus::Satisfied => "satisfied",
        TypeFactQueryStatus::Missing => "missing",
        TypeFactQueryStatus::Contradicted => "contradicted",
    }
}

fn coercion_kind_name(kind: CoercionKind) -> &'static str {
    match kind {
        CoercionKind::Widening => "widening",
        CoercionKind::Narrowing => "narrowing",
        CoercionKind::SourceQua => "source_qua",
    }
}

fn coercion_status_name(status: CoercionStatus) -> &'static str {
    match status {
        CoercionStatus::Candidate => "candidate",
        CoercionStatus::RequiresObligation => "requires_obligation",
        CoercionStatus::Blocked => "blocked",
        CoercionStatus::Rejected => "rejected",
    }
}

fn initial_obligation_kind_name(kind: InitialObligationKind) -> &'static str {
    match kind {
        InitialObligationKind::Sethood => "sethood",
        InitialObligationKind::NonEmptiness => "non_emptiness",
        InitialObligationKind::Narrowing => "narrowing",
        InitialObligationKind::RegistrationCorrectness => "registration_correctness",
    }
}

fn initial_obligation_status_name(status: InitialObligationStatus) -> &'static str {
    match status {
        InitialObligationStatus::Pending => "pending",
        InitialObligationStatus::Blocked => "blocked",
        InitialObligationStatus::Invalidated => "invalidated",
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
    fn source_reserve_declaration_bridge_builds_checker_owned_handoff() {
        let source = source_id();
        let module = module_id();
        let symbols = symbol_env(Vec::new());
        let bridge = SourceReserveDeclarationBridge::new(
            source,
            module,
            range(source, 0, 40),
            vec![
                SourceReserveBindingInput::new(
                    "x",
                    range(source, 8, 9),
                    range(source, 17, 20),
                    "set",
                    TypeHeadInput::BuiltinSet,
                ),
                SourceReserveBindingInput::new(
                    "y",
                    range(source, 11, 12),
                    range(source, 17, 20),
                    "set",
                    TypeHeadInput::BuiltinSet,
                ),
                SourceReserveBindingInput::new(
                    "z",
                    range(source, 29, 30),
                    range(source, 35, 41),
                    "object",
                    TypeHeadInput::BuiltinObject,
                ),
            ],
        )
        .expect("reserve bridge inputs should validate");

        let handoff = bridge
            .check(&symbols)
            .expect("checker-owned source reserve handoff should assemble");
        let binding_env = handoff.binding_env();
        let declarations = handoff.declarations();
        let module_context = binding_env
            .contexts()
            .get(bridge.module_context())
            .expect("module context should exist");

        assert_eq!(
            module_context.bindings,
            vec![BindingId::new(0), BindingId::new(1), BindingId::new(2)]
        );
        assert_eq!(module_context.visible_bindings, module_context.bindings);
        assert_eq!(binding_env.bindings().len(), 3);
        assert_eq!(declarations.declarations().len(), 3);
        assert_eq!(declarations.contexts().len(), 1);
        assert!(declarations.diagnostics().is_empty());
        assert_eq!(bridge.root_node(), TypedNodeId::new(6));
        assert_ne!(bridge.type_node(0), bridge.type_node(1));

        let checked = declarations_by_binding(declarations);
        for (index, source_binding) in bridge.bindings().iter().enumerate() {
            let binding_id = BindingId::new(index);
            let binding = binding_env
                .bindings()
                .get(binding_id)
                .expect("reserve binding should exist");
            assert_eq!(binding.spelling, source_binding.spelling);
            assert_eq!(binding.kind, BindingKind::ReservedVariable);
            assert_eq!(binding.owner_context, bridge.module_context());
            assert_eq!(
                binding.type_site,
                BindingTypeSite::Source(source_binding.type_range)
            );
            assert_eq!(binding.status, BindingStatus::Reserved);
            assert_eq!(binding.visible_after_ordinal, index);
            assert!(matches!(
                &binding.identity,
                BinderIdentity::ReservedVariable {
                    spelling,
                    declaration_range,
                } if spelling == &source_binding.spelling
                    && *declaration_range == source_binding.binding_range
            ));

            let declaration = checked
                .get(&binding_id)
                .expect("checked declaration should exist");
            assert_eq!(declaration.kind, DeclarationKind::ReservedVariable);
            assert_eq!(declaration.status, DeclarationStatus::Checked);
            assert_eq!(
                declaration.site,
                TypedSiteRef::Node(bridge.declaration_node(index))
            );
            assert_eq!(
                declaration.type_site,
                Some(TypedSiteRef::Node(bridge.type_node(index)))
            );
            assert!(declaration.type_entry.is_some());
        }
    }

    #[test]
    fn source_reserve_declaration_bridge_preserves_attributed_builtin_evidence_gap() {
        let source = source_id();
        let module = module_id();
        let attr = symbol_id("empty/0", "pkg::main::empty/0");
        let symbols = symbol_env(vec![symbol_entry(attr.clone(), SymbolKind::Attribute)]);
        let bridge = SourceReserveDeclarationBridge::new(
            source,
            module,
            range(source, 0, 30),
            vec![
                SourceReserveBindingInput::new(
                    "x",
                    range(source, 8, 9),
                    range(source, 14, 27),
                    "non empty set",
                    TypeHeadInput::BuiltinSet,
                )
                .with_type_attributes(vec![AttributeInput::new(
                    attr.clone(),
                    AttributePolarity::Negative,
                    range(source, 14, 23),
                    "non empty",
                )]),
            ],
        )
        .expect("attributed builtin reserve payload should validate");

        let handoff = bridge
            .check(&symbols)
            .expect("checker-owned attributed reserve handoff should assemble");
        let declaration = declarations_by_binding(handoff.declarations())
            .remove(&BindingId::new(0))
            .expect("checked declaration should exist");
        assert_eq!(declaration.status, DeclarationStatus::Partial);
        assert_eq!(
            declaration.deferred,
            vec![DeclarationDeferredReason::MissingEvidenceQuery]
        );
        let type_entry = handoff
            .declarations()
            .type_entries()
            .get(declaration.type_entry.unwrap())
            .expect("attributed reserve type entry should exist");
        assert_eq!(type_entry.status, TypeStatus::Unknown);
        let TypeEntryActual::Known(normalized_id) = type_entry.actual else {
            panic!("attributed reserve should keep a normalized type");
        };
        let normalized = handoff
            .declarations()
            .normalized_types()
            .get(normalized_id)
            .expect("normalized attributed reserve type should exist");
        assert_eq!(normalized.status, NormalizedTypeStatus::Known);
        assert_eq!(normalized.attributes.negative().len(), 1);
        assert_eq!(normalized.attributes.negative()[0].symbol, attr);
        assert!(normalized.attributes.positive().is_empty());
        assert_eq!(
            diagnostic_ranges(
                handoff.declarations(),
                "checker.declaration.deferred.evidence_query"
            ),
            vec![(8, 9)]
        );
        assert!(handoff.declarations().facts().is_empty());
    }

    #[test]
    fn source_reserve_declaration_bridge_validates_local_symbol_heads_and_mismatched_inputs() {
        let source = source_id();
        let module = module_id();
        let mode = symbol_id("Mode/0", "pkg::main::Mode/0");
        let mode_bridge = SourceReserveDeclarationBridge::new(
            source,
            module.clone(),
            range(source, 0, 10),
            vec![SourceReserveBindingInput::new(
                "x",
                range(source, 0, 1),
                range(source, 4, 8),
                "Mode",
                TypeHeadInput::Symbol(mode),
            )],
        )
        .expect("local mode reserve payload should validate source shape");
        let mode_symbols = symbol_env(vec![symbol_entry(
            symbol_id("Mode/0", "pkg::main::Mode/0"),
            SymbolKind::Mode,
        )]);
        let mode_handoff = mode_bridge
            .check(&mode_symbols)
            .expect("local mode reserve payload should reach declaration checking");
        assert_eq!(
            diagnostic_ranges(
                mode_handoff.declarations(),
                "checker.type.external.mode_expansion_payload"
            ),
            vec![(4, 8)]
        );

        let mode_expansion_bridge = SourceReserveDeclarationBridge::new(
            source,
            module.clone(),
            range(source, 0, 10),
            vec![SourceReserveBindingInput::new(
                "x",
                range(source, 0, 1),
                range(source, 4, 8),
                "Mode",
                TypeHeadInput::Symbol(symbol_id("Mode/0", "pkg::main::Mode/0")),
            )],
        )
        .expect("local mode reserve payload should validate source shape");
        let mode_expansion_handoff = mode_expansion_bridge
            .check_with_mode_expansions(
                &mode_symbols,
                [(
                    symbol_id("Mode/0", "pkg::main::Mode/0"),
                    ModeExpansion::new(
                        TypeExpressionInput::new(
                            site(777),
                            range(source, 20, 23),
                            "set",
                            TypeHeadInput::BuiltinSet,
                        ),
                        Vec::new(),
                    ),
                )],
            )
            .expect("local mode expansion payload should reach declaration checking");
        assert!(
            diagnostic_ranges(
                &mode_expansion_handoff.declarations,
                "checker.type.external.mode_expansion_payload"
            )
            .is_empty()
        );
        assert!(
            diagnostic_ranges(
                &mode_expansion_handoff.declarations,
                "checker.declaration.deferred.evidence_query"
            )
            .is_empty()
        );
        let expanded_mode_declaration =
            declarations_by_binding(mode_expansion_handoff.declarations())
                .remove(&BindingId::new(0))
                .expect("checked expanded mode declaration should exist");
        assert_eq!(expanded_mode_declaration.status, DeclarationStatus::Checked);
        let expanded_mode_type_entry = mode_expansion_handoff
            .declarations()
            .type_entries()
            .get(
                expanded_mode_declaration
                    .type_entry
                    .expect("expanded mode declaration should keep a type entry"),
            )
            .expect("expanded mode type entry should exist");
        assert_eq!(expanded_mode_type_entry.status, TypeStatus::Known);

        let struct_radix = symbol_id("Struct/0", "pkg::main::Struct/0");
        let mode_to_structure_symbols = symbol_env(vec![
            symbol_entry(symbol_id("Mode/0", "pkg::main::Mode/0"), SymbolKind::Mode),
            symbol_entry(struct_radix.clone(), SymbolKind::Structure),
        ]);
        let mode_to_structure_handoff = mode_expansion_bridge
            .check_with_mode_expansions(
                &mode_to_structure_symbols,
                [(
                    symbol_id("Mode/0", "pkg::main::Mode/0"),
                    ModeExpansion::new(
                        TypeExpressionInput::new(
                            site(778),
                            range(source, 20, 26),
                            "Struct",
                            TypeHeadInput::Symbol(struct_radix),
                        ),
                        Vec::new(),
                    ),
                )],
            )
            .expect("mode expansion to structure should reach declaration checking");
        assert!(
            diagnostic_ranges(
                &mode_to_structure_handoff.declarations,
                "checker.type.external.mode_expansion_payload"
            )
            .is_empty()
        );
        assert_eq!(
            diagnostic_ranges(
                &mode_to_structure_handoff.declarations,
                "checker.declaration.deferred.evidence_query"
            ),
            vec![(0, 1)]
        );
        let mode_to_structure_declaration =
            declarations_by_binding(mode_to_structure_handoff.declarations())
                .remove(&BindingId::new(0))
                .expect("checked mode-to-structure declaration should exist");
        assert_eq!(
            mode_to_structure_declaration.status,
            DeclarationStatus::Partial
        );
        let mode_to_structure_type_entry = mode_to_structure_handoff
            .declarations()
            .type_entries()
            .get(
                mode_to_structure_declaration
                    .type_entry
                    .expect("mode-to-structure declaration should keep a type entry"),
            )
            .expect("mode-to-structure type entry should exist");
        assert_eq!(mode_to_structure_type_entry.status, TypeStatus::Unknown);
        assert!(
            mode_to_structure_handoff.declarations().facts().is_empty(),
            "mode expansions with structure radixes must not seed facts without base-shape evidence"
        );

        let chained_struct_radix = symbol_id("Struct/0", "pkg::main::Struct/0");
        let chained_mode = symbol_id("Mode/0", "pkg::main::Mode/0");
        let chained_alias = symbol_id("Alias/0", "pkg::main::Alias/0");
        let chained_mode_to_structure_symbols = symbol_env(vec![
            symbol_entry(chained_mode.clone(), SymbolKind::Mode),
            symbol_entry(chained_alias.clone(), SymbolKind::Mode),
            symbol_entry(chained_struct_radix.clone(), SymbolKind::Structure),
        ]);
        let chained_mode_to_structure_handoff = mode_expansion_bridge
            .check_with_mode_expansions(
                &chained_mode_to_structure_symbols,
                [
                    (
                        chained_mode,
                        ModeExpansion::new(
                            TypeExpressionInput::new(
                                site(779),
                                range(source, 30, 35),
                                "Alias",
                                TypeHeadInput::Symbol(chained_alias.clone()),
                            ),
                            Vec::new(),
                        ),
                    ),
                    (
                        chained_alias,
                        ModeExpansion::new(
                            TypeExpressionInput::new(
                                site(780),
                                range(source, 40, 46),
                                "Struct",
                                TypeHeadInput::Symbol(chained_struct_radix),
                            ),
                            Vec::new(),
                        ),
                    ),
                ],
            )
            .expect("chained mode expansion to structure should reach declaration checking");
        assert!(
            diagnostic_ranges(
                &chained_mode_to_structure_handoff.declarations,
                "checker.type.external.mode_expansion_payload"
            )
            .is_empty()
        );
        assert_eq!(
            diagnostic_ranges(
                &chained_mode_to_structure_handoff.declarations,
                "checker.declaration.deferred.evidence_query"
            ),
            vec![(0, 1)]
        );
        assert!(
            chained_mode_to_structure_handoff
                .declarations()
                .facts()
                .is_empty(),
            "chained mode expansions ending in structure radixes must not seed facts"
        );

        let attributed_expansion_attr = symbol_id("marked/0", "pkg::main::marked/0");
        let attributed_expansion_symbols = symbol_env(vec![
            symbol_entry(symbol_id("Mode/0", "pkg::main::Mode/0"), SymbolKind::Mode),
            symbol_entry(attributed_expansion_attr.clone(), SymbolKind::Attribute),
        ]);
        let attributed_expansion_handoff = mode_expansion_bridge
            .check_with_mode_expansions(
                &attributed_expansion_symbols,
                [(
                    symbol_id("Mode/0", "pkg::main::Mode/0"),
                    ModeExpansion::new(
                        TypeExpressionInput::new(
                            site(781),
                            range(source, 50, 53),
                            "set",
                            TypeHeadInput::BuiltinSet,
                        ),
                        vec![AttributeInput::new(
                            attributed_expansion_attr.clone(),
                            AttributePolarity::Positive,
                            range(source, 45, 51),
                            "marked",
                        )],
                    ),
                )],
            )
            .expect("attributed mode expansion should reach declaration checking");
        assert!(
            diagnostic_ranges(
                &attributed_expansion_handoff.declarations,
                "checker.type.external.mode_expansion_payload"
            )
            .is_empty()
        );
        assert_eq!(
            diagnostic_ranges(
                &attributed_expansion_handoff.declarations,
                "checker.declaration.deferred.evidence_query"
            ),
            vec![(0, 1)]
        );
        let attributed_expansion_declaration =
            declarations_by_binding(attributed_expansion_handoff.declarations())
                .remove(&BindingId::new(0))
                .expect("checked attributed-expansion declaration should exist");
        assert_eq!(
            attributed_expansion_declaration.status,
            DeclarationStatus::Partial
        );
        let attributed_expansion_type_entry = attributed_expansion_handoff
            .declarations()
            .type_entries()
            .get(
                attributed_expansion_declaration
                    .type_entry
                    .expect("attributed-expansion declaration should keep a type entry"),
            )
            .expect("attributed-expansion type entry should exist");
        assert_eq!(attributed_expansion_type_entry.status, TypeStatus::Unknown);
        let TypeEntryActual::Known(attributed_expansion_normalized_id) =
            attributed_expansion_type_entry.actual
        else {
            panic!("attributed-expansion reserve should keep a normalized type");
        };
        let attributed_expansion_normalized = attributed_expansion_handoff
            .declarations()
            .normalized_types()
            .get(attributed_expansion_normalized_id)
            .expect("normalized attributed-expansion type should exist");
        assert_eq!(
            attributed_expansion_normalized.attributes.positive().len(),
            1
        );
        assert_eq!(
            attributed_expansion_normalized.attributes.positive()[0].symbol,
            attributed_expansion_attr
        );
        assert!(
            attributed_expansion_handoff
                .declarations()
                .facts()
                .is_empty(),
            "attributed mode expansions must not seed facts without existential evidence"
        );

        let chained_attributed_attr = symbol_id("marked/0", "pkg::main::marked/0");
        let chained_attributed_mode = symbol_id("Mode/0", "pkg::main::Mode/0");
        let chained_attributed_alias = symbol_id("Alias/0", "pkg::main::Alias/0");
        let chained_attributed_symbols = symbol_env(vec![
            symbol_entry(chained_attributed_mode.clone(), SymbolKind::Mode),
            symbol_entry(chained_attributed_alias.clone(), SymbolKind::Mode),
            symbol_entry(chained_attributed_attr.clone(), SymbolKind::Attribute),
        ]);
        let chained_attributed_handoff = mode_expansion_bridge
            .check_with_mode_expansions(
                &chained_attributed_symbols,
                [
                    (
                        chained_attributed_mode,
                        ModeExpansion::new(
                            TypeExpressionInput::new(
                                site(782),
                                range(source, 60, 65),
                                "Alias",
                                TypeHeadInput::Symbol(chained_attributed_alias.clone()),
                            ),
                            Vec::new(),
                        ),
                    ),
                    (
                        chained_attributed_alias,
                        ModeExpansion::new(
                            TypeExpressionInput::new(
                                site(783),
                                range(source, 70, 73),
                                "set",
                                TypeHeadInput::BuiltinSet,
                            )
                            .with_attributes(vec![
                                AttributeInput::new(
                                    chained_attributed_attr,
                                    AttributePolarity::Positive,
                                    range(source, 64, 70),
                                    "marked",
                                ),
                            ]),
                            Vec::new(),
                        ),
                    ),
                ],
            )
            .expect("chained attributed mode expansion should reach declaration checking");
        assert!(
            diagnostic_ranges(
                &chained_attributed_handoff.declarations,
                "checker.type.external.mode_expansion_payload"
            )
            .is_empty()
        );
        assert_eq!(
            diagnostic_ranges(
                &chained_attributed_handoff.declarations,
                "checker.declaration.deferred.evidence_query"
            ),
            vec![(0, 1)]
        );
        assert!(
            chained_attributed_handoff.declarations().facts().is_empty(),
            "chained attributed mode expansions must not seed facts"
        );

        let structure = symbol_id("Struct/0", "pkg::main::Struct/0");
        let structure_bridge = SourceReserveDeclarationBridge::new(
            source,
            module.clone(),
            range(source, 0, 10),
            vec![SourceReserveBindingInput::new(
                "x",
                range(source, 0, 1),
                range(source, 4, 10),
                "Struct",
                TypeHeadInput::Symbol(structure.clone()),
            )],
        )
        .expect("symbol reserve payload shape should validate before SymbolEnv checks");
        let structure_symbols = symbol_env(vec![symbol_entry(structure, SymbolKind::Structure)]);
        let structure_handoff = structure_bridge
            .check(&structure_symbols)
            .expect("local structure reserve payload should reach declaration checking");
        assert_eq!(
            diagnostic_ranges(
                structure_handoff.declarations(),
                "checker.declaration.deferred.evidence_query"
            ),
            vec![(0, 1)]
        );
        assert!(
            structure_handoff.declarations().facts().is_empty(),
            "local structure reserve heads must not seed facts without base-shape evidence"
        );

        let attributed_structure = symbol_id("Struct/0", "pkg::main::Struct/0");
        let attributed_structure_attr = symbol_id("empty/0", "pkg::main::empty/0");
        let attributed_structure_bridge = SourceReserveDeclarationBridge::new(
            source,
            module.clone(),
            range(source, 0, 25),
            vec![
                SourceReserveBindingInput::new(
                    "x",
                    range(source, 0, 1),
                    range(source, 4, 25),
                    "non empty Struct",
                    TypeHeadInput::Symbol(attributed_structure.clone()),
                )
                .with_type_attributes(vec![AttributeInput::new(
                    attributed_structure_attr.clone(),
                    AttributePolarity::Negative,
                    range(source, 4, 13),
                    "non empty",
                )]),
            ],
        )
        .expect("attributed local structure reserve payload should validate source shape");
        let attributed_structure_symbols = symbol_env(vec![
            symbol_entry(attributed_structure, SymbolKind::Structure),
            symbol_entry(attributed_structure_attr.clone(), SymbolKind::Attribute),
        ]);
        let attributed_structure_handoff = attributed_structure_bridge
            .check(&attributed_structure_symbols)
            .expect("attributed local structure reserve payload should reach declaration checking");
        assert_eq!(
            diagnostic_ranges(
                attributed_structure_handoff.declarations(),
                "checker.declaration.deferred.evidence_query"
            ),
            vec![(0, 1)]
        );
        let attributed_structure_declaration =
            declarations_by_binding(attributed_structure_handoff.declarations())
                .remove(&BindingId::new(0))
                .expect("checked attributed structure declaration should exist");
        let attributed_structure_type_entry = attributed_structure_handoff
            .declarations()
            .type_entries()
            .get(
                attributed_structure_declaration
                    .type_entry
                    .expect("attributed structure declaration should keep a type entry"),
            )
            .expect("attributed structure type entry should exist");
        let TypeEntryActual::Known(attributed_structure_normalized_id) =
            attributed_structure_type_entry.actual
        else {
            panic!("attributed local structure should keep a normalized type");
        };
        let attributed_structure_normalized = attributed_structure_handoff
            .declarations()
            .normalized_types()
            .get(attributed_structure_normalized_id)
            .expect("normalized attributed local structure type should exist");
        assert_eq!(
            attributed_structure_normalized.status,
            NormalizedTypeStatus::Known
        );
        assert_eq!(
            attributed_structure_normalized.attributes.negative().len(),
            1
        );
        assert_eq!(
            attributed_structure_normalized.attributes.negative()[0].symbol,
            attributed_structure_attr
        );
        assert!(
            attributed_structure_normalized
                .attributes
                .positive()
                .is_empty()
        );
        assert!(
            attributed_structure_handoff
                .declarations()
                .facts()
                .is_empty(),
            "attributed local structure reserve heads must not seed facts without existential evidence"
        );

        let imported_attribute = SymbolId::new(
            ModuleId::new(PackageId::new("pkg"), ModulePath::new("imported")),
            LocalSymbolId::new("empty/0"),
            FullyQualifiedName::new("pkg::imported::empty/0"),
        );
        let imported_attribute_bridge = SourceReserveDeclarationBridge::new(
            source,
            module.clone(),
            range(source, 0, 25),
            vec![
                SourceReserveBindingInput::new(
                    "x",
                    range(source, 0, 1),
                    range(source, 4, 25),
                    "non empty Struct",
                    TypeHeadInput::Symbol(symbol_id("Struct/0", "pkg::main::Struct/0")),
                )
                .with_type_attributes(vec![AttributeInput::new(
                    imported_attribute.clone(),
                    AttributePolarity::Negative,
                    range(source, 4, 13),
                    "non empty",
                )]),
            ],
        )
        .expect("imported attribute payload shape should validate before SymbolEnv checks");
        assert!(
            imported_attribute_bridge
                .check(&symbol_env_with_imported_attribute(
                    symbol_id("Struct/0", "pkg::main::Struct/0"),
                    imported_attribute.clone()
                ))
                .is_err(),
            "imported source attribute payloads must fail closed at the checker seam"
        );

        let wrong_kind_attribute = symbol_id("empty/0", "pkg::main::empty/0");
        let wrong_kind_attribute_bridge = SourceReserveDeclarationBridge::new(
            source,
            module.clone(),
            range(source, 0, 25),
            vec![
                SourceReserveBindingInput::new(
                    "x",
                    range(source, 0, 1),
                    range(source, 4, 25),
                    "non empty Struct",
                    TypeHeadInput::Symbol(symbol_id("Struct/0", "pkg::main::Struct/0")),
                )
                .with_type_attributes(vec![AttributeInput::new(
                    wrong_kind_attribute.clone(),
                    AttributePolarity::Negative,
                    range(source, 4, 13),
                    "non empty",
                )]),
            ],
        )
        .expect("wrong-kind attribute payload shape should validate before SymbolEnv checks");
        let wrong_kind_symbols = symbol_env(vec![
            symbol_entry(
                symbol_id("Struct/0", "pkg::main::Struct/0"),
                SymbolKind::Structure,
            ),
            symbol_entry(wrong_kind_attribute, SymbolKind::Mode),
        ]);
        assert!(
            wrong_kind_attribute_bridge
                .check(&wrong_kind_symbols)
                .is_err(),
            "non-attribute symbol payloads must fail closed at the checker seam"
        );

        let attribute_args = SourceReserveDeclarationBridge::new(
            source,
            module.clone(),
            range(source, 0, 25),
            vec![
                SourceReserveBindingInput::new(
                    "x",
                    range(source, 0, 1),
                    range(source, 4, 25),
                    "empty(x) Struct",
                    TypeHeadInput::Symbol(symbol_id("Struct/0", "pkg::main::Struct/0")),
                )
                .with_type_attributes(vec![
                    AttributeInput::new(
                        symbol_id("empty/0", "pkg::main::empty/0"),
                        AttributePolarity::Positive,
                        range(source, 4, 12),
                        "empty(x)",
                    )
                    .with_args(vec![TypeExpressionInput::new(
                        TypedSiteRef::Node(TypedNodeId::new(999)),
                        range(source, 10, 11),
                        "set",
                        TypeHeadInput::BuiltinSet,
                    )]),
                ]),
            ],
        );
        assert!(
            attribute_args.is_err(),
            "attribute arguments must stay on the extraction gap"
        );

        let attributed_mode = SourceReserveDeclarationBridge::new(
            source,
            module.clone(),
            range(source, 0, 18),
            vec![
                SourceReserveBindingInput::new(
                    "x",
                    range(source, 0, 1),
                    range(source, 4, 18),
                    "non empty Mode",
                    TypeHeadInput::Symbol(symbol_id("Mode/0", "pkg::main::Mode/0")),
                )
                .with_type_attributes(vec![AttributeInput::new(
                    symbol_id("empty/0", "pkg::main::empty/0"),
                    AttributePolarity::Negative,
                    range(source, 4, 13),
                    "non empty",
                )]),
            ],
        )
        .expect("attributed local mode payload should validate before SymbolEnv checks");
        let mode_attribute_symbols = symbol_env(vec![
            symbol_entry(symbol_id("Mode/0", "pkg::main::Mode/0"), SymbolKind::Mode),
            symbol_entry(
                symbol_id("empty/0", "pkg::main::empty/0"),
                SymbolKind::Attribute,
            ),
        ]);
        let attributed_mode_handoff = attributed_mode
            .check(&mode_attribute_symbols)
            .expect("attributed local mode reserve payload should reach type normalization");
        assert_eq!(
            diagnostic_ranges(
                attributed_mode_handoff.declarations(),
                "checker.type.external.mode_expansion_payload"
            ),
            vec![(4, 18)]
        );
        assert!(
            diagnostic_ranges(
                attributed_mode_handoff.declarations(),
                "checker.declaration.deferred.evidence_query"
            )
            .is_empty(),
            "attributed local modes must not receive an evidence-query diagnostic before real expansion payloads exist"
        );
        let attributed_mode_declaration =
            declarations_by_binding(attributed_mode_handoff.declarations())
                .remove(&BindingId::new(0))
                .expect("checked attributed mode declaration should exist");
        let attributed_mode_type_entry = attributed_mode_handoff
            .declarations()
            .type_entries()
            .get(
                attributed_mode_declaration
                    .type_entry
                    .expect("attributed mode declaration should keep a type entry"),
            )
            .expect("attributed mode type entry should exist");
        let TypeEntryActual::Known(attributed_mode_normalized_id) =
            attributed_mode_type_entry.actual
        else {
            panic!("attributed local mode should keep a normalized type");
        };
        let attributed_mode_normalized = attributed_mode_handoff
            .declarations()
            .normalized_types()
            .get(attributed_mode_normalized_id)
            .expect("normalized attributed local mode type should exist");
        assert_eq!(
            attributed_mode_normalized.status,
            NormalizedTypeStatus::Degraded
        );
        assert_eq!(attributed_mode_normalized.attributes.negative().len(), 1);
        assert_eq!(
            attributed_mode_normalized.attributes.negative()[0].symbol,
            symbol_id("empty/0", "pkg::main::empty/0")
        );
        assert!(attributed_mode_normalized.attributes.positive().is_empty());
        assert!(
            attributed_mode_handoff.declarations().facts().is_empty(),
            "attributed local mode reserve heads must not seed facts without mode expansion and existential evidence"
        );

        let attributed_mode_with_expansion = SourceReserveDeclarationBridge::new(
            source,
            module.clone(),
            range(source, 0, 18),
            vec![
                SourceReserveBindingInput::new(
                    "x",
                    range(source, 0, 1),
                    range(source, 4, 18),
                    "non empty Mode",
                    TypeHeadInput::Symbol(symbol_id("Mode/0", "pkg::main::Mode/0")),
                )
                .with_type_attributes(vec![AttributeInput::new(
                    symbol_id("empty/0", "pkg::main::empty/0"),
                    AttributePolarity::Negative,
                    range(source, 4, 13),
                    "non empty",
                )]),
            ],
        )
        .expect("attributed mode reserve payload should validate before SymbolEnv checks");
        let attributed_mode_with_expansion_handoff = attributed_mode_with_expansion
            .check_with_mode_expansions(
                &mode_attribute_symbols,
                [(
                    symbol_id("Mode/0", "pkg::main::Mode/0"),
                    ModeExpansion::new(
                        TypeExpressionInput::new(
                            site(778),
                            range(source, 20, 23),
                            "set",
                            TypeHeadInput::BuiltinSet,
                        ),
                        Vec::new(),
                    ),
                )],
            )
            .expect(
                "attributed mode with real expansion should fail closed in declaration checking",
            );
        assert_eq!(
            diagnostic_ranges(
                attributed_mode_with_expansion_handoff.declarations(),
                "checker.declaration.deferred.evidence_query"
            ),
            vec![(0, 1)]
        );
        assert!(
            diagnostic_ranges(
                attributed_mode_with_expansion_handoff.declarations(),
                "checker.type.external.mode_expansion_payload"
            )
            .is_empty(),
            "real mode expansion should move attributed mode uses past the missing-expansion gap"
        );
        assert!(
            attributed_mode_with_expansion_handoff
                .declarations()
                .facts()
                .is_empty(),
            "real mode expansion must not make attributed mode uses accepted without existential evidence"
        );

        let unresolved = SourceReserveDeclarationBridge::new(
            source,
            module.clone(),
            range(source, 0, 10),
            vec![SourceReserveBindingInput::new(
                "x",
                range(source, 0, 1),
                range(source, 4, 11),
                "Missing",
                TypeHeadInput::Unresolved("Missing".to_owned()),
            )],
        );
        assert!(
            unresolved.is_err(),
            "unresolved source reserve heads must stay on the extraction gap"
        );

        let ambiguous = SourceReserveDeclarationBridge::new(
            source,
            module.clone(),
            range(source, 0, 10),
            vec![SourceReserveBindingInput::new(
                "x",
                range(source, 0, 1),
                range(source, 4, 8),
                "Mode",
                TypeHeadInput::Ambiguous(vec![
                    symbol_id("Mode/0", "pkg::main::Mode/0"),
                    symbol_id("Mode/1", "pkg::main::Mode/1"),
                ]),
            )],
        );
        assert!(
            ambiguous.is_err(),
            "ambiguous source reserve heads must stay on the extraction gap"
        );

        let other_source = source_ids_pair().1;
        let mismatched_range = SourceReserveDeclarationBridge::new(
            source,
            module.clone(),
            range(source, 0, 10),
            vec![SourceReserveBindingInput::new(
                "x",
                range(other_source, 0, 1),
                range(source, 4, 7),
                "set",
                TypeHeadInput::BuiltinSet,
            )],
        );
        assert!(
            mismatched_range.is_err(),
            "source-derived payload ranges must belong to the bridge source"
        );

        let symbols = SymbolEnv::new(
            ModuleId::new(PackageId::new("test"), ModulePath::new("other")),
            SymbolEnvIndexes::default(),
        );
        let bridge = SourceReserveDeclarationBridge::new(
            source,
            module.clone(),
            range(source, 0, 10),
            vec![SourceReserveBindingInput::new(
                "x",
                range(source, 0, 1),
                range(source, 4, 7),
                "set",
                TypeHeadInput::BuiltinSet,
            )],
        )
        .expect("builtin payload should validate");
        assert!(
            bridge.check(&symbols).is_err(),
            "symbol environment module mismatches must fail closed"
        );

        let imported_mode = SymbolId::new(
            ModuleId::new(PackageId::new("pkg"), ModulePath::new("imported")),
            LocalSymbolId::new("ImportedMode/0"),
            FullyQualifiedName::new("pkg::imported::ImportedMode/0"),
        );
        let imported_bridge = SourceReserveDeclarationBridge::new(
            source,
            module,
            range(source, 0, 15),
            vec![SourceReserveBindingInput::new(
                "x",
                range(source, 0, 1),
                range(source, 4, 15),
                "ImportedMode",
                TypeHeadInput::Symbol(imported_mode),
            )],
        )
        .expect("symbol reserve payload shape should validate before local module checks");
        assert!(
            imported_bridge.check(&mode_symbols).is_err(),
            "non-local symbol heads must fail closed"
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
    fn coercion_checker_records_candidates_and_initial_obligations_deterministically() {
        let source = source_id();
        let symbols = symbol_env(Vec::new());
        let mut input_facts = TypeFactTable::new();
        let known_fact = input_facts.insert(TypeFactDraft {
            subject: site(90),
            predicate: TypePredicateRef::new("known_widening_support"),
            polarity: Polarity::Positive,
            provenance: FactProvenance::Inferred(TypeRuleId::new("fixture.known")),
            status: FactStatus::Known,
        });
        let coercions = vec![
            CoercionInput::new(
                site(12),
                range(source, 60, 65),
                CoercionRequestKind::Narrowing,
                type_expression(source, 112, TypeHeadInput::BuiltinSet),
            )
            .with_from_type(type_expression(source, 212, TypeHeadInput::BuiltinObject))
            .with_supporting_facts(vec![known_fact])
            .with_obligation(
                InitialObligationInput::new(
                    site(12),
                    range(source, 60, 65),
                    InitialRequirementKind::Narrowing,
                    type_expression(source, 312, TypeHeadInput::BuiltinSet),
                )
                .with_assumptions(vec![known_fact])
                .with_goal(InitialObligationGoal::new("reconsider.existing"))
                .with_provenance(InitialObligationProvenance::new("reconsider-existing")),
            ),
            CoercionInput::new(
                site(13),
                range(source, 65, 70),
                CoercionRequestKind::Narrowing,
                type_expression(source, 113, TypeHeadInput::BuiltinSet),
            )
            .with_from_type(type_expression(source, 213, TypeHeadInput::BuiltinObject))
            .with_obligation(
                InitialObligationInput::new(
                    site(13),
                    range(source, 65, 70),
                    InitialRequirementKind::Narrowing,
                    type_expression(source, 313, TypeHeadInput::BuiltinSet),
                )
                .with_goal(InitialObligationGoal::new("reconsider.new"))
                .with_provenance(InitialObligationProvenance::new("reconsider-new")),
            ),
            CoercionInput::new(
                site(10),
                range(source, 50, 55),
                CoercionRequestKind::SourceQua,
                type_expression(source, 110, TypeHeadInput::BuiltinSet),
            )
            .with_from_type(type_expression(source, 210, TypeHeadInput::BuiltinObject))
            .with_evidence(CoercionEvidence::StaticUpcast),
            CoercionInput::new(
                site(14),
                range(source, 70, 75),
                CoercionRequestKind::SourceQua,
                type_expression(source, 114, TypeHeadInput::BuiltinSet),
            )
            .with_from_type(type_expression(source, 214, TypeHeadInput::BuiltinObject))
            .with_evidence(CoercionEvidence::CompatibleView),
            CoercionInput::new(
                site(11),
                range(source, 55, 60),
                CoercionRequestKind::Widening,
                type_expression(source, 111, TypeHeadInput::BuiltinSet),
            )
            .with_from_type(type_expression(source, 211, TypeHeadInput::BuiltinObject))
            .with_evidence(CoercionEvidence::KnownFacts)
            .with_supporting_facts(vec![known_fact]),
            CoercionInput::new(
                site(15),
                range(source, 75, 80),
                CoercionRequestKind::Widening,
                type_expression(source, 115, TypeHeadInput::BuiltinSet),
            )
            .with_from_type(type_expression(source, 215, TypeHeadInput::BuiltinObject))
            .with_evidence(CoercionEvidence::BuiltinRadix),
            CoercionInput::new(
                site(16),
                range(source, 80, 85),
                CoercionRequestKind::Widening,
                type_expression(source, 116, TypeHeadInput::BuiltinSet),
            )
            .with_from_type(type_expression(source, 216, TypeHeadInput::BuiltinObject))
            .with_evidence(CoercionEvidence::StructureInheritance)
            .with_supporting_facts(vec![known_fact]),
            CoercionInput::new(
                site(17),
                range(source, 85, 90),
                CoercionRequestKind::Widening,
                type_expression(source, 117, TypeHeadInput::BuiltinSet),
            )
            .with_from_type(type_expression(source, 217, TypeHeadInput::BuiltinObject))
            .with_evidence(CoercionEvidence::ActivatedSummary)
            .with_supporting_facts(vec![known_fact]),
            CoercionInput::new(
                site(18),
                range(source, 100, 105),
                CoercionRequestKind::Narrowing,
                type_expression(source, 118, TypeHeadInput::BuiltinSet),
            )
            .with_from_type(type_expression(source, 218, TypeHeadInput::BuiltinObject))
            .with_evidence(CoercionEvidence::KnownFacts)
            .with_supporting_facts(vec![known_fact]),
        ];
        let obligations = vec![
            InitialObligationInput::new(
                site(21),
                range(source, 95, 100),
                InitialRequirementKind::NonEmptiness,
                type_expression(source, 121, TypeHeadInput::BuiltinSet),
            )
            .with_goal(InitialObligationGoal::new("choice.non_empty"))
            .with_provenance(InitialObligationProvenance::new("choice-term")),
            InitialObligationInput::new(
                site(20),
                range(source, 90, 95),
                InitialRequirementKind::Sethood,
                type_expression(source, 120, TypeHeadInput::BuiltinSet),
            )
            .with_assumptions(vec![known_fact]),
        ];

        let first = CoercionObligationChecker::default().check(
            &symbols,
            &input_facts,
            coercions.clone(),
            obligations.clone(),
        );
        let second = CoercionObligationChecker::default().check(
            &symbols,
            &input_facts,
            coercions.into_iter().rev(),
            obligations.into_iter().rev(),
        );
        assert_eq!(first.debug_text(), second.debug_text());

        assert_eq!(first.coercions().len(), 9);
        assert_eq!(first.initial_obligations().len(), 4);
        assert_eq!(first.facts().len(), 6);
        assert_eq!(
            first.facts().get(known_fact).unwrap().status,
            FactStatus::Known
        );
        let coercion_at = |site_ref: TypedSiteRef| {
            first
                .coercions()
                .iter()
                .map(|(_, coercion)| coercion)
                .find(|coercion| coercion.site == site_ref)
                .unwrap()
        };
        let widening = coercion_at(site(11));
        assert_eq!(widening.kind, CoercionKind::Widening);
        assert_eq!(widening.status, CoercionStatus::Candidate);
        assert_eq!(widening.supporting_facts, vec![known_fact]);
        let builtin_widening = coercion_at(site(15));
        assert_eq!(builtin_widening.kind, CoercionKind::Widening);
        assert_eq!(builtin_widening.status, CoercionStatus::Candidate);
        assert_eq!(builtin_widening.supporting_facts.len(), 1);
        let builtin_fact = first
            .facts()
            .get(builtin_widening.supporting_facts[0])
            .unwrap();
        assert_eq!(builtin_fact.status, FactStatus::Known);
        assert!(matches!(
            builtin_fact.provenance,
            FactProvenance::Builtin(_)
        ));
        for widening_site in [site(16), site(17)] {
            let widening = coercion_at(widening_site);
            assert_eq!(widening.kind, CoercionKind::Widening);
            assert_eq!(widening.status, CoercionStatus::Candidate);
            assert_eq!(widening.supporting_facts, vec![known_fact]);
        }
        let source_qua = coercion_at(site(10));
        assert_eq!(source_qua.kind, CoercionKind::SourceQua);
        assert_eq!(source_qua.status, CoercionStatus::Candidate);
        let compatible_qua = coercion_at(site(14));
        assert_eq!(compatible_qua.kind, CoercionKind::SourceQua);
        assert_eq!(compatible_qua.status, CoercionStatus::Candidate);

        let obligation_at = |site_ref: TypedSiteRef, kind: InitialObligationKind| {
            first
                .initial_obligations()
                .iter()
                .map(|(_, obligation)| obligation)
                .find(|obligation| obligation.owner == site_ref && obligation.kind == kind)
                .unwrap()
        };
        let existing_narrowing = coercion_at(site(12));
        assert_eq!(existing_narrowing.kind, CoercionKind::Narrowing);
        assert_eq!(
            existing_narrowing.status,
            CoercionStatus::RequiresObligation
        );
        let existing_obligation_id = existing_narrowing.obligation.unwrap();
        let existing_obligation = first
            .initial_obligations()
            .get(existing_obligation_id)
            .unwrap();
        assert_eq!(existing_obligation.kind, InitialObligationKind::Narrowing);
        assert_eq!(existing_obligation.source_range, range(source, 60, 65));
        assert_eq!(existing_obligation.assumptions, vec![known_fact]);
        assert_eq!(existing_obligation.goal.as_str(), "reconsider.existing");
        assert_eq!(
            existing_obligation.provenance.as_str(),
            "reconsider-existing"
        );
        assert_eq!(existing_obligation.status, InitialObligationStatus::Pending);
        let new_narrowing = coercion_at(site(13));
        assert_eq!(new_narrowing.kind, CoercionKind::Narrowing);
        assert_eq!(new_narrowing.status, CoercionStatus::RequiresObligation);
        let new_obligation_id = new_narrowing.obligation.unwrap();
        let new_obligation = first.initial_obligations().get(new_obligation_id).unwrap();
        assert_eq!(new_obligation.kind, InitialObligationKind::Narrowing);
        assert_eq!(new_obligation.source_range, range(source, 65, 70));
        assert!(new_obligation.assumptions.is_empty());
        assert_eq!(new_obligation.goal.as_str(), "reconsider.new");
        assert_eq!(new_obligation.provenance.as_str(), "reconsider-new");
        assert_eq!(new_obligation.status, InitialObligationStatus::Pending);
        let supported_narrowing = coercion_at(site(18));
        assert_eq!(supported_narrowing.kind, CoercionKind::Narrowing);
        assert_eq!(supported_narrowing.status, CoercionStatus::Candidate);
        assert_eq!(supported_narrowing.supporting_facts, vec![known_fact]);
        assert_eq!(supported_narrowing.obligation, None);

        let sethood = obligation_at(site(20), InitialObligationKind::Sethood);
        assert_eq!(
            sethood.id,
            first.initial_obligations().get(sethood.id).unwrap().id
        );
        assert_eq!(sethood.source_range, range(source, 90, 95));
        assert_eq!(sethood.assumptions, vec![known_fact]);
        let non_empty = obligation_at(site(21), InitialObligationKind::NonEmptiness);
        assert_eq!(
            non_empty.id,
            first.initial_obligations().get(non_empty.id).unwrap().id
        );
        assert_eq!(non_empty.source_range, range(source, 95, 100));
        assert!(non_empty.assumptions.is_empty());
        for (_, obligation) in first.initial_obligations().iter() {
            let fact = first
                .facts()
                .iter()
                .map(|(_, fact)| fact)
                .find(|fact| {
                    matches!(
                        &fact.provenance,
                        FactProvenance::Obligation(obligation_id)
                            if *obligation_id == obligation.id
                    )
                })
                .unwrap();
            assert_eq!(fact.subject, obligation.owner);
            assert_eq!(fact.status, FactStatus::PendingObligation);
        }
        assert_eq!(
            first
                .initial_obligations()
                .iter()
                .map(|(_, obligation)| obligation.kind)
                .collect::<Vec<_>>(),
            vec![
                InitialObligationKind::Narrowing,
                InitialObligationKind::Narrowing,
                InitialObligationKind::Sethood,
                InitialObligationKind::NonEmptiness,
            ]
        );
        let debug = first.debug_text();
        assert!(debug.contains("coercion-checking-debug-v1"));
        assert!(debug.contains("kind=widening status=candidate"));
        assert!(debug.contains("kind=source_qua status=candidate"));
        assert!(debug.contains("kind=narrowing status=requires_obligation"));
        assert!(debug.contains("kind=sethood"));
        assert!(debug.contains("kind=non_emptiness"));
        assert!(debug.contains("widening_rule=\"coercion-widening-builtin_radix\""));
        assert!(debug.contains("widening_rule=\"coercion-widening-structure_inheritance\""));
        assert!(debug.contains("widening_rule=\"coercion-widening-activated_summary\""));
        assert!(debug.contains("source_qua:70..75"));
        assert!(debug.contains("goal=\"reconsider.existing\""));
        assert!(debug.contains("goal=\"reconsider.new\""));
        assert!(debug.contains("obligation#"));
        assert!(!debug.contains(concat!("V", "cId")));
        assert!(!debug.contains(concat!("Proof", "Witness")));
        assert!(!debug.contains("ObligationAnchor"));
    }

    #[test]
    fn omitted_reconsider_accepts_only_proof_free_consumable_evidence() {
        let source = source_id();
        let symbols = symbol_env(Vec::new());
        let mut input_facts = TypeFactTable::new();
        let known_fact = input_facts.insert(TypeFactDraft {
            subject: site(90),
            predicate: TypePredicateRef::new("proof_free_reconsider_support"),
            polarity: Polarity::Positive,
            provenance: FactProvenance::Inferred(TypeRuleId::new("fixture.known")),
            status: FactStatus::Known,
        });
        let degraded_fact = input_facts.insert(TypeFactDraft {
            subject: site(91),
            predicate: TypePredicateRef::new("not_consumable_reconsider_support"),
            polarity: Polarity::Positive,
            provenance: FactProvenance::Inferred(TypeRuleId::new("fixture.degraded")),
            status: FactStatus::Degraded,
        });

        let accepted_evidence = [
            (30, CoercionEvidence::KnownFacts),
            (31, CoercionEvidence::BuiltinRadix),
            (32, CoercionEvidence::StructureInheritance),
            (33, CoercionEvidence::ActivatedSummary),
            (34, CoercionEvidence::StaticUpcast),
            (35, CoercionEvidence::CompatibleView),
        ]
        .into_iter()
        .map(|(offset, evidence)| {
            omitted_reconsider_input(source, offset, evidence)
                .with_supporting_facts(vec![known_fact])
        });
        let rejected_evidence = [
            omitted_reconsider_input(source, 40, CoercionEvidence::ActivatedSummary),
            omitted_reconsider_input(source, 41, CoercionEvidence::BuiltinRadix),
            omitted_reconsider_input(source, 42, CoercionEvidence::StaticUpcast),
            omitted_reconsider_input(source, 43, CoercionEvidence::CompatibleView)
                .with_supporting_facts(vec![degraded_fact]),
            omitted_reconsider_input(source, 44, CoercionEvidence::Missing)
                .with_supporting_facts(vec![known_fact]),
        ];
        let output = CoercionObligationChecker::default().check(
            &symbols,
            &input_facts,
            accepted_evidence
                .chain(rejected_evidence)
                .collect::<Vec<_>>(),
            Vec::new(),
        );

        assert!(output.initial_obligations().is_empty());
        let coercion_at = |site_ref: TypedSiteRef| {
            output
                .coercions()
                .iter()
                .map(|(_, coercion)| coercion)
                .find(|coercion| coercion.site == site_ref)
                .unwrap()
        };
        for offset in 30..=35 {
            let coercion = coercion_at(site(offset));
            assert_eq!(coercion.kind, CoercionKind::Narrowing);
            assert_eq!(coercion.status, CoercionStatus::Candidate);
            assert_eq!(coercion.supporting_facts, vec![known_fact]);
            assert_eq!(coercion.obligation, None);
        }
        for offset in 40..=44 {
            let coercion = coercion_at(site(offset));
            assert_eq!(coercion.kind, CoercionKind::Narrowing);
            assert_eq!(coercion.status, CoercionStatus::Rejected);
            assert_eq!(coercion.obligation, None);
        }
        assert_eq!(
            diagnostic_ranges(&output, "type.narrowing_requires_proof"),
            vec![(200, 205), (205, 210), (210, 215), (215, 220), (220, 225)]
        );
        let debug = output.debug_text();
        assert!(debug.contains("type.narrowing_requires_proof"));
        assert!(debug.contains("checker.coercion.supporting_fact_not_consumable"));
        assert!(!debug.contains("kind=narrowing status=requires_obligation"));
    }

    #[test]
    fn explicit_narrowing_preserves_task_ten_obligation_path() {
        let source = source_id();
        let symbols = symbol_env(Vec::new());
        let mut input_facts = TypeFactTable::new();
        let known_fact = input_facts.insert(TypeFactDraft {
            subject: site(92),
            predicate: TypePredicateRef::new("explicit_narrowing_support"),
            polarity: Polarity::Positive,
            provenance: FactProvenance::Inferred(TypeRuleId::new("fixture.explicit")),
            status: FactStatus::Known,
        });
        let explicit_markers = [
            (50, CoercionEvidence::BuiltinRadix),
            (51, CoercionEvidence::StructureInheritance),
            (52, CoercionEvidence::ActivatedSummary),
            (53, CoercionEvidence::StaticUpcast),
            (54, CoercionEvidence::CompatibleView),
        ]
        .into_iter()
        .map(|(offset, evidence)| {
            CoercionInput::new(
                site(offset),
                range(source, offset * 5, offset * 5 + 5),
                CoercionRequestKind::Narrowing,
                type_expression(source, offset + 100, TypeHeadInput::BuiltinSet),
            )
            .with_from_type(type_expression(
                source,
                offset + 200,
                TypeHeadInput::BuiltinObject,
            ))
            .with_evidence(evidence)
            .with_supporting_facts(vec![known_fact])
        })
        .collect::<Vec<_>>();

        let output = CoercionObligationChecker::default().check(
            &symbols,
            &input_facts,
            explicit_markers,
            Vec::new(),
        );

        assert_eq!(output.initial_obligations().len(), 5);
        for offset in 50..=54 {
            let coercion = output
                .coercions()
                .iter()
                .map(|(_, coercion)| coercion)
                .find(|coercion| coercion.site == site(offset))
                .unwrap();
            assert_eq!(coercion.kind, CoercionKind::Narrowing);
            assert_eq!(coercion.status, CoercionStatus::RequiresObligation);
            let obligation = output
                .initial_obligations()
                .get(coercion.obligation.unwrap())
                .unwrap();
            assert_eq!(obligation.kind, InitialObligationKind::Narrowing);
            assert_eq!(obligation.assumptions, vec![known_fact]);
        }
        assert!(diagnostic_ranges(&output, "type.narrowing_requires_proof").is_empty());
    }

    #[test]
    fn coercion_checker_blocks_missing_evidence_and_invalid_qua_without_fabrication() {
        let source = source_id();
        let symbols = symbol_env(Vec::new());
        let mut input_facts = TypeFactTable::new();
        let degraded_fact = input_facts.insert(TypeFactDraft {
            subject: site(91),
            predicate: TypePredicateRef::new("degraded_support"),
            polarity: Polarity::Positive,
            provenance: FactProvenance::Inferred(TypeRuleId::new("fixture.degraded")),
            status: FactStatus::Degraded,
        });
        let known_fact = input_facts.insert(TypeFactDraft {
            subject: site(92),
            predicate: TypePredicateRef::new("known_qua_is_not_qua_evidence"),
            polarity: Polarity::Positive,
            provenance: FactProvenance::Inferred(TypeRuleId::new("fixture.known_qua")),
            status: FactStatus::Known,
        });
        input_facts.insert(TypeFactDraft {
            subject: site(34),
            predicate: TypePredicateRef::new("coercion.widening.builtin_radix"),
            polarity: Polarity::Positive,
            provenance: FactProvenance::Builtin(BuiltinRuleId::new(
                "coercion-widening-builtin-radix",
            )),
            status: FactStatus::Degraded,
        });
        let coercions = vec![
            CoercionInput::new(
                site(30),
                range(source, 150, 155),
                CoercionRequestKind::Widening,
                type_expression(source, 130, TypeHeadInput::BuiltinSet),
            )
            .with_from_type(type_expression(source, 230, TypeHeadInput::BuiltinObject))
            .with_deferred(vec![CoercionDeferredReason::MissingWideningSupportPayload]),
            CoercionInput::new(
                site(31),
                range(source, 155, 160),
                CoercionRequestKind::SourceQua,
                type_expression(source, 131, TypeHeadInput::BuiltinSet),
            )
            .with_from_type(type_expression(source, 231, TypeHeadInput::BuiltinObject)),
            CoercionInput::new(
                site(32),
                range(source, 160, 165),
                CoercionRequestKind::Widening,
                type_expression(source, 132, TypeHeadInput::BuiltinSet),
            )
            .with_from_type(type_expression(source, 232, TypeHeadInput::BuiltinObject))
            .with_evidence(CoercionEvidence::KnownFacts)
            .with_supporting_facts(vec![degraded_fact]),
            CoercionInput::new(
                site(33),
                range(source, 165, 170),
                CoercionRequestKind::Widening,
                type_expression(source, 133, TypeHeadInput::BuiltinSet),
            )
            .with_from_type(type_expression(source, 233, TypeHeadInput::BuiltinObject))
            .with_evidence(CoercionEvidence::StructureInheritance),
            CoercionInput::new(
                site(34),
                range(source, 170, 175),
                CoercionRequestKind::Widening,
                type_expression(source, 134, TypeHeadInput::BuiltinSet),
            )
            .with_from_type(type_expression(source, 234, TypeHeadInput::BuiltinObject))
            .with_evidence(CoercionEvidence::BuiltinRadix),
            CoercionInput::new(
                site(35),
                range(source, 175, 180),
                CoercionRequestKind::SourceQua,
                type_expression(source, 135, TypeHeadInput::BuiltinSet),
            )
            .with_from_type(type_expression(source, 235, TypeHeadInput::BuiltinObject))
            .with_evidence(CoercionEvidence::KnownFacts)
            .with_supporting_facts(vec![known_fact]),
        ];
        let obligations = vec![
            InitialObligationInput::new(
                site(40),
                range(source, 200, 205),
                InitialRequirementKind::Sethood,
                type_expression(source, 140, TypeHeadInput::BuiltinSet),
            )
            .with_assumptions(vec![degraded_fact]),
            InitialObligationInput::new(
                site(41),
                range(source, 205, 210),
                InitialRequirementKind::NonEmptiness,
                type_expression(source, 141, TypeHeadInput::BuiltinSet),
            )
            .with_assumptions(vec![TypeFactId::new(99)]),
        ];

        let output = CoercionObligationChecker::default().check(
            &symbols,
            &input_facts,
            coercions.clone(),
            obligations.clone(),
        );
        let reversed = CoercionObligationChecker::default().check(
            &symbols,
            &input_facts,
            coercions.into_iter().rev(),
            obligations.into_iter().rev(),
        );
        assert_eq!(output.debug_text(), reversed.debug_text());

        assert_eq!(
            output
                .coercions()
                .iter()
                .map(|(_, coercion)| (coercion.site.clone(), coercion.status))
                .collect::<Vec<_>>(),
            vec![
                (site(30), CoercionStatus::Blocked),
                (site(31), CoercionStatus::Rejected),
                (site(32), CoercionStatus::Blocked),
                (site(33), CoercionStatus::Blocked),
                (site(34), CoercionStatus::Blocked),
                (site(35), CoercionStatus::Rejected),
            ]
        );
        assert!(
            output
                .initial_obligations()
                .iter()
                .all(|(_, obligation)| obligation.status == InitialObligationStatus::Blocked)
        );
        assert_eq!(
            diagnostic_ranges(
                &output,
                "checker.coercion.external.widening_support_payload"
            ),
            vec![(150, 155), (160, 165), (165, 170)]
        );
        assert_eq!(
            diagnostic_ranges(&output, "checker.coercion.invalid_source_qua_target"),
            vec![(155, 160), (175, 180)]
        );
        assert_eq!(
            diagnostic_ranges(&output, "checker.coercion.supporting_fact_not_consumable"),
            vec![(160, 165)]
        );
        assert_eq!(
            diagnostic_ranges(
                &output,
                "checker.coercion.builtin_widening_fact_not_consumable"
            ),
            vec![(170, 175)]
        );
        assert_eq!(
            diagnostic_ranges(&output, "checker.obligation.assumption_not_consumable"),
            vec![(200, 205)]
        );
        assert_eq!(
            diagnostic_ranges(&output, "checker.obligation.unknown_assumption"),
            vec![(205, 210)]
        );
        let debug = output.debug_text();
        assert!(debug.contains("kind=widening status=blocked"));
        assert!(debug.contains("kind=source_qua status=rejected"));
        assert!(debug.contains("kind=sethood"));
        assert!(debug.contains("kind=non_emptiness"));
        assert!(debug.contains("status=blocked"));
        assert!(!debug.contains("accepted_verifier_status"));
        assert!(!debug.contains(concat!("inserted", "_qua")));
    }

    #[test]
    fn coercion_checker_preserves_alternate_same_site_kind_candidates() {
        let source = source_id();
        let symbols = symbol_env(Vec::new());
        let mut input_facts = TypeFactTable::new();
        let support = input_facts.insert(TypeFactDraft {
            subject: site(92),
            predicate: TypePredicateRef::new("alternate_widening_support"),
            polarity: Polarity::Positive,
            provenance: FactProvenance::Inferred(TypeRuleId::new("fixture.alternate")),
            status: FactStatus::Known,
        });
        let first_input = CoercionInput::new(
            site(50),
            range(source, 250, 255),
            CoercionRequestKind::Widening,
            type_expression(source, 150, TypeHeadInput::BuiltinSet),
        )
        .with_from_type(type_expression(source, 250, TypeHeadInput::BuiltinObject))
        .with_evidence(CoercionEvidence::KnownFacts)
        .with_supporting_facts(vec![support]);
        let second_input = CoercionInput::new(
            site(50),
            range(source, 250, 255),
            CoercionRequestKind::Widening,
            TypeExpressionInput::new(
                site(151),
                range(source, 252, 255),
                "object",
                TypeHeadInput::BuiltinObject,
            ),
        )
        .with_from_type(type_expression(source, 251, TypeHeadInput::BuiltinObject))
        .with_evidence(CoercionEvidence::BuiltinRadix);

        let first = CoercionObligationChecker::default().check(
            &symbols,
            &input_facts,
            [first_input.clone(), second_input.clone()],
            Vec::new(),
        );
        let second = CoercionObligationChecker::default().check(
            &symbols,
            &input_facts,
            [second_input, first_input],
            Vec::new(),
        );

        assert_eq!(first.debug_text(), second.debug_text());
        assert_eq!(first.coercions().len(), 2);
        assert!(
            first
                .coercions()
                .iter()
                .all(|(_, coercion)| coercion.site == site(50)
                    && coercion.kind == CoercionKind::Widening
                    && coercion.status == CoercionStatus::Candidate)
        );
        assert_eq!(
            diagnostic_ranges(&first, "checker.coercion.duplicate_site"),
            Vec::<(usize, usize)>::new()
        );
    }

    #[test]
    fn type_fact_queries_are_deterministic_and_ignore_provenance_for_matching() {
        let source = source_id();
        let subject = site(70);
        let predicate = TypePredicateRef::new("is_set_like");
        let mut first_facts = TypeFactTable::new();
        let inferred = first_facts.insert(TypeFactDraft {
            subject: subject.clone(),
            predicate: predicate.clone(),
            polarity: Polarity::Positive,
            provenance: FactProvenance::Inferred(TypeRuleId::new("z-rule")),
            status: FactStatus::Known,
        });
        let builtin = first_facts.insert(TypeFactDraft {
            subject: subject.clone(),
            predicate: predicate.clone(),
            polarity: Polarity::Positive,
            provenance: FactProvenance::Builtin(BuiltinRuleId::new("a-builtin")),
            status: FactStatus::Known,
        });
        let duplicate = first_facts.insert(TypeFactDraft {
            subject: subject.clone(),
            predicate: predicate.clone(),
            polarity: Polarity::Positive,
            provenance: FactProvenance::Builtin(BuiltinRuleId::new("a-builtin")),
            status: FactStatus::Known,
        });
        assert_eq!(duplicate, builtin);
        assert_eq!(first_facts.len(), 2);

        let mut second_facts = TypeFactTable::new();
        second_facts.insert(TypeFactDraft {
            subject: subject.clone(),
            predicate: predicate.clone(),
            polarity: Polarity::Positive,
            provenance: FactProvenance::Builtin(BuiltinRuleId::new("a-builtin")),
            status: FactStatus::Known,
        });
        second_facts.insert(TypeFactDraft {
            subject: subject.clone(),
            predicate: predicate.clone(),
            polarity: Polarity::Positive,
            provenance: FactProvenance::Inferred(TypeRuleId::new("z-rule")),
            status: FactStatus::Known,
        });

        let query = TypeFactQuery::new(
            subject.clone(),
            predicate.clone(),
            Polarity::Positive,
            range(source, 350, 355),
        );
        let first = TypeFactQueryEngine::new(&first_facts).query(query.clone());
        let second = TypeFactQueryEngine::new(&second_facts).query(query);

        assert_eq!(first.status(), TypeFactQueryStatus::Satisfied);
        assert_eq!(first.matched_facts(), &[inferred, builtin]);
        assert_eq!(second.status(), TypeFactQueryStatus::Satisfied);
        assert_eq!(
            second.matched_facts(),
            &[TypeFactId::new(1), TypeFactId::new(0)]
        );
        assert!(first.debug_text().contains("status=satisfied"));
        assert!(first.debug_text().contains("matched=[fact#0, fact#1]"));
        assert!(first.diagnostics().is_empty());
        assert!(!first.debug_text().contains("registration"));
        assert!(!first.debug_text().contains(concat!("Proof", "Witness")));
    }

    #[test]
    fn type_fact_queries_respect_assumption_visibility_and_context_absence() {
        let source = source_id();
        let symbols = symbol_env(Vec::new());
        let binding_env = binding_env_for_declarations(
            source,
            vec![
                binding_spec(
                    "reserved_y",
                    BindingKind::ReservedVariable,
                    BindingStatus::Active,
                ),
                binding_spec(
                    "given_w",
                    BindingKind::QuantifierBinder,
                    BindingStatus::Active,
                ),
            ],
        );
        let output = DeclarationChecker::default().check(
            &symbols,
            &binding_env,
            vec![
                DeclarationContextInput::new(
                    BindingContextId::new(0),
                    site(300),
                    range(source, 0, 1),
                ),
                DeclarationContextInput::new(
                    BindingContextId::new(1),
                    site(301),
                    range(source, 1, 2),
                ),
            ],
            vec![
                declaration_with_type_in_context(
                    source,
                    0,
                    BindingContextId::new(0),
                    DeclarationKind::ReservedVariable,
                    80,
                    180,
                )
                .with_reserved_default(ReservedDefaultPayload::new(site(180), false))
                .with_assumptions(vec![DeclarationAssumptionInput::new(
                    TypePredicateRef::new("module_assumption"),
                    range(source, 350, 351),
                )]),
                declaration_with_type_in_context(
                    source,
                    1,
                    BindingContextId::new(1),
                    DeclarationKind::Given,
                    81,
                    181,
                )
                .with_assumptions(vec![DeclarationAssumptionInput::new(
                    TypePredicateRef::new("block_assumption"),
                    range(source, 351, 352),
                )]),
            ],
        );
        let module_query = TypeFactQuery::new(
            site(80),
            TypePredicateRef::new("module_assumption"),
            Polarity::Positive,
            range(source, 360, 361),
        );
        let block_query = TypeFactQuery::new(
            site(81),
            TypePredicateRef::new("block_assumption"),
            Polarity::Positive,
            range(source, 361, 362),
        );
        let contextless = TypeFactQueryEngine::new(output.facts()).query(module_query.clone());
        let no_query_context =
            TypeFactQueryEngine::with_contexts(output.facts(), output.contexts())
                .query(module_query.clone());
        let module_context = TypeFactQueryEngine::with_contexts(output.facts(), output.contexts())
            .query(
                module_query
                    .clone()
                    .with_context(LocalTypeContextId::new(0)),
            );
        let child_context = TypeFactQueryEngine::with_contexts(output.facts(), output.contexts())
            .query(module_query.with_context(LocalTypeContextId::new(1)));
        let block_from_parent =
            TypeFactQueryEngine::with_contexts(output.facts(), output.contexts())
                .query(block_query.clone().with_context(LocalTypeContextId::new(0)));
        let block_from_child =
            TypeFactQueryEngine::with_contexts(output.facts(), output.contexts())
                .query(block_query.with_context(LocalTypeContextId::new(1)));

        assert_eq!(contextless.status(), TypeFactQueryStatus::Missing);
        assert_eq!(no_query_context.status(), TypeFactQueryStatus::Missing);
        assert_eq!(module_context.status(), TypeFactQueryStatus::Satisfied);
        assert_eq!(child_context.status(), TypeFactQueryStatus::Satisfied);
        assert_eq!(block_from_parent.status(), TypeFactQueryStatus::Missing);
        assert_eq!(block_from_child.status(), TypeFactQueryStatus::Satisfied);
        assert_eq!(
            TypeFactQueryEngine::with_contexts(output.facts(), output.contexts())
                .active_facts(Some(LocalTypeContextId::new(1))),
            vec![TypeFactId::new(0), TypeFactId::new(1)]
        );
        assert_eq!(
            TypeFactQueryEngine::with_contexts(output.facts(), output.contexts())
                .active_facts(None),
            Vec::<TypeFactId>::new()
        );
        assert_eq!(
            TypeFactQueryEngine::new(output.facts()).active_facts(Some(LocalTypeContextId::new(1))),
            Vec::<TypeFactId>::new()
        );
        assert_eq!(
            TypeFactQueryEngine::with_contexts(output.facts(), output.contexts())
                .active_facts(Some(LocalTypeContextId::new(99))),
            Vec::<TypeFactId>::new()
        );

        let mut hidden_facts = TypeFactTable::new();
        let hidden_fact = hidden_facts.insert(TypeFactDraft {
            subject: site(82),
            predicate: TypePredicateRef::new("hidden_assumption"),
            polarity: Polarity::Positive,
            provenance: FactProvenance::Inferred(TypeRuleId::new("hidden")),
            status: FactStatus::Assumed,
        });
        let mut hidden_contexts = LocalTypeContextTable::new();
        let parent = hidden_contexts.insert(LocalTypeContextDraft {
            owner: site(302),
            parent: None,
            layer: TypeContextLayer::Block,
            bindings: Vec::new(),
            introduced_assumptions: vec![hidden_fact],
            visible_facts: vec![hidden_fact],
            recovery: ContextRecoveryState::Normal,
        });
        let hidden_child = hidden_contexts.insert(LocalTypeContextDraft {
            owner: site(303),
            parent: Some(parent),
            layer: TypeContextLayer::Block,
            bindings: Vec::new(),
            introduced_assumptions: Vec::new(),
            visible_facts: Vec::new(),
            recovery: ContextRecoveryState::Normal,
        });
        let hidden_query = TypeFactQuery::new(
            site(82),
            TypePredicateRef::new("hidden_assumption"),
            Polarity::Positive,
            range(source, 362, 363),
        );
        let hidden_engine = TypeFactQueryEngine::with_contexts(&hidden_facts, &hidden_contexts);

        assert_eq!(
            hidden_engine
                .query(hidden_query.clone().with_context(parent))
                .status(),
            TypeFactQueryStatus::Satisfied
        );
        assert_eq!(
            hidden_engine
                .query(hidden_query.with_context(hidden_child))
                .status(),
            TypeFactQueryStatus::Missing
        );
        assert_eq!(hidden_engine.active_facts(Some(parent)), vec![hidden_fact]);
        assert_eq!(
            hidden_engine.active_facts(Some(hidden_child)),
            Vec::<TypeFactId>::new()
        );

        let mut filtered_facts = TypeFactTable::new();
        let visible_positive = filtered_facts.insert(TypeFactDraft {
            subject: site(83),
            predicate: TypePredicateRef::new("visibility_filtered_opposite"),
            polarity: Polarity::Positive,
            provenance: FactProvenance::Inferred(TypeRuleId::new("visible")),
            status: FactStatus::Known,
        });
        let hidden_negative = filtered_facts.insert(TypeFactDraft {
            subject: site(83),
            predicate: TypePredicateRef::new("visibility_filtered_opposite"),
            polarity: Polarity::Negative,
            provenance: FactProvenance::Inferred(TypeRuleId::new("hidden-opposite")),
            status: FactStatus::Assumed,
        });
        let mut filtered_contexts = LocalTypeContextTable::new();
        let contradiction_parent = filtered_contexts.insert(LocalTypeContextDraft {
            owner: site(304),
            parent: None,
            layer: TypeContextLayer::Block,
            bindings: Vec::new(),
            introduced_assumptions: vec![hidden_negative],
            visible_facts: vec![hidden_negative],
            recovery: ContextRecoveryState::Normal,
        });
        let filtered_child = filtered_contexts.insert(LocalTypeContextDraft {
            owner: site(305),
            parent: Some(contradiction_parent),
            layer: TypeContextLayer::Block,
            bindings: Vec::new(),
            introduced_assumptions: Vec::new(),
            visible_facts: Vec::new(),
            recovery: ContextRecoveryState::Normal,
        });
        let filtered_query = TypeFactQuery::new(
            site(83),
            TypePredicateRef::new("visibility_filtered_opposite"),
            Polarity::Positive,
            range(source, 363, 364),
        );
        let filtered_engine =
            TypeFactQueryEngine::with_contexts(&filtered_facts, &filtered_contexts);

        assert_eq!(
            filtered_engine
                .query(filtered_query.clone().with_context(contradiction_parent))
                .status(),
            TypeFactQueryStatus::Contradicted
        );
        assert_eq!(
            filtered_engine
                .query(filtered_query.with_context(filtered_child))
                .status(),
            TypeFactQueryStatus::Satisfied
        );
        assert_eq!(
            filtered_engine.active_facts(Some(filtered_child)),
            vec![visible_positive]
        );
    }

    #[test]
    fn type_fact_queries_report_contradictions_without_mutating_facts() {
        let source = source_id();
        let subject = site(90);
        let predicate = TypePredicateRef::new("contradictory");
        let mut facts = TypeFactTable::new();
        let negative = facts.insert(TypeFactDraft {
            subject: subject.clone(),
            predicate: predicate.clone(),
            polarity: Polarity::Negative,
            provenance: FactProvenance::Builtin(BuiltinRuleId::new("negative")),
            status: FactStatus::Known,
        });
        let positive = facts.insert(TypeFactDraft {
            subject: subject.clone(),
            predicate: predicate.clone(),
            polarity: Polarity::Positive,
            provenance: FactProvenance::Inferred(TypeRuleId::new("positive")),
            status: FactStatus::Known,
        });
        facts.insert(TypeFactDraft {
            subject: subject.clone(),
            predicate: predicate.clone(),
            polarity: Polarity::Negative,
            provenance: FactProvenance::Inferred(TypeRuleId::new("degraded-negative")),
            status: FactStatus::Degraded,
        });
        facts.insert(TypeFactDraft {
            subject: subject.clone(),
            predicate: predicate.clone(),
            polarity: Polarity::Positive,
            provenance: FactProvenance::Obligation(InitialObligationId::new(7)),
            status: FactStatus::PendingObligation,
        });
        facts.insert(TypeFactDraft {
            subject: subject.clone(),
            predicate: predicate.clone(),
            polarity: Polarity::Negative,
            provenance: FactProvenance::Inferred(TypeRuleId::new("rejected-negative")),
            status: FactStatus::Rejected,
        });
        let query = TypeFactQuery::new(
            subject.clone(),
            predicate.clone(),
            Polarity::Positive,
            range(source, 400, 405),
        );

        let output = TypeFactQueryEngine::new(&facts).query(query);
        let negative_output = TypeFactQueryEngine::new(&facts).query(TypeFactQuery::new(
            subject.clone(),
            predicate.clone(),
            Polarity::Negative,
            range(source, 405, 410),
        ));

        assert_eq!(output.status(), TypeFactQueryStatus::Contradicted);
        assert_eq!(output.matched_facts(), &[positive, negative]);
        assert_eq!(negative_output.status(), TypeFactQueryStatus::Contradicted);
        assert_eq!(negative_output.matched_facts(), &[positive, negative]);
        assert_eq!(facts.len(), 5);
        assert_eq!(
            TypeFactQueryEngine::new(&facts).active_facts(None),
            vec![positive, negative]
        );
        assert_eq!(
            diagnostic_ranges(&output, "checker.fact.contradiction"),
            vec![(400, 405)]
        );
        assert!(output.debug_text().contains("status=contradicted"));
        assert!(output.debug_text().contains("class=TypeFact"));
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

    impl HasDiagnostics for CoercionCheckingOutput {
        fn diagnostics(&self) -> &TypeDiagnosticTable {
            self.diagnostics()
        }
    }

    impl HasDiagnostics for TypeFactQueryOutput {
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

    fn omitted_reconsider_input(
        source: SourceId,
        offset: usize,
        evidence: CoercionEvidence,
    ) -> CoercionInput {
        CoercionInput::new(
            site(offset),
            range(source, offset * 5, offset * 5 + 5),
            CoercionRequestKind::Narrowing,
            type_expression(source, offset + 100, TypeHeadInput::BuiltinSet),
        )
        .with_from_type(type_expression(
            source,
            offset + 200,
            TypeHeadInput::BuiltinObject,
        ))
        .with_justification(CoercionJustification::Omitted)
        .with_evidence(evidence)
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

    fn symbol_env_with_imported_attribute(
        structure: SymbolId,
        imported_attribute: SymbolId,
    ) -> SymbolEnv {
        let source = source_id();
        let imported_module = imported_attribute.module().clone();
        let mut contributions = SourceContributionIndex::new();
        let local_contribution = contributions.insert(
            module_id(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 1)),
        );
        let imported_contribution = contributions.insert(
            imported_module.clone(),
            ContributionKind::ImportedSource { source_id: source },
            SourceAnchor::Range(range(source, 1, 2)),
        );
        let mut symbols = SymbolIndex::new();
        symbols.insert(
            SymbolEntry::new(
                structure,
                SymbolKind::Structure,
                NamespacePath::new("main"),
                "Struct",
                SemanticOrigin::new(
                    source,
                    module_id(),
                    SourceAnchor::Range(range(source, 0, 1)),
                    Vec::new(),
                ),
                local_contribution,
            )
            .with_visibility(Visibility::Public)
            .with_export_status(ExportStatus::Exported),
        );
        symbols.insert(
            SymbolEntry::new(
                imported_attribute,
                SymbolKind::Attribute,
                NamespacePath::new("main"),
                "empty",
                SemanticOrigin::new(
                    source,
                    imported_module,
                    SourceAnchor::Range(range(source, 1, 2)),
                    Vec::new(),
                ),
                imported_contribution,
            )
            .with_visibility(Visibility::Public)
            .with_export_status(ExportStatus::Exported),
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
