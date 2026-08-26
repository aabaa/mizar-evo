//! Syntax-free cross-family source formula composition transport.

use crate::{
    binding_env::{
        BinderIdentity, BindingContextDraft, BindingContextId, BindingContextLayer,
        BindingContextOwner, BindingContextRecovery, BindingContextTable, BindingDraft, BindingEnv,
        BindingEnvParts, BindingId, BindingKind, BindingLookupResult, BindingLookupSite,
        BindingRecoveryState, BindingStatus, BindingTable, BindingTypeSite, CapturedFreeVariables,
    },
    source_application::{
        SourceFunctorApplicationForm, SourceFunctorApplicationHandoff, SourceFunctorApplicationId,
        SourceFunctorApplicationKind, SourceFunctorApplicationRecovery, SourceFunctorArgumentId,
        SourceFunctorArgumentTarget, SourceFunctorCandidateId, SourceFunctorHeadSite,
        SourceFunctorTypeRequestKind,
    },
    source_atomic_formula::{
        SourceAtomicEdgeId, SourceAtomicEdgeRole, SourceAtomicFormulaHandoff,
        SourceAtomicFormulaId, SourceAtomicFormulaKind, SourceAtomicFormulaRecovery,
        SourceAtomicRequestId, SourceAtomicRequestKind, SourceAtomicTermTarget,
        SourcePredicateCandidateId, SourcePredicateHeadId, SourcePredicateSegmentId,
        SourcePredicateSegmentPolarityInput,
    },
    source_composite_formula::{
        SourceCompositeFormulaHandoff, SourceCompositeFormulaId, SourceCompositeFormulaKind,
        SourceCompositeFormulaRecovery, SourceQuantifierBinderId,
    },
    source_set_term::{
        SourceSetConditionId, SourceSetEdgeId, SourceSetEdgeRole, SourceSetGeneratorId,
        SourceSetRequestKind, SourceSetTarget, SourceSetTermHandoff, SourceSetTermId,
        SourceSetTermKind, SourceSetTermRecovery, SourceSetTypeHead, SourceSetTypeOwner,
        SourceSetTypeSiteId,
    },
    source_template_type_parameter_association::{
        SourceTemplateFraenkelStructuralComposition,
        SourceTemplateFraenkelStructuralCompositionHandoff,
        SourceTemplateFraenkelStructuralCompositionId,
    },
    source_term::{
        SourceNestedFraenkelMapperPrimaryHandoff, SourceNumericTypeRequestId,
        SourcePrimaryTermHandoff, SourcePrimaryTermId, SourcePrimaryTermKind,
        SourcePrimaryTermRecovery, SourcePrimaryTermReferenceId, SourcePrimaryTermReferenceRole,
        SourcePrimaryTermRole,
    },
    typed_ast::{NodeRecoveryState, TypedArena, TypedAst, TypedNode, TypedNodeId},
};
use mizar_resolve::{
    names::{
        FraenkelGeneratorVariableBinding, FraenkelGeneratorVariableBindingId,
        FraenkelGeneratorVariableSourceCollection, FraenkelGeneratorVariableUseLink,
        FraenkelGeneratorVariableUseRole,
    },
    resolved_ast::{ModuleId, ResolvedNodeId},
};
use mizar_session::{SourceAnchor, SourceId, SourceRange};
use std::{
    error::Error,
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

dense_id!(SourceFormulaAtomicEdgeId);
dense_id!(SourceQuantifierBoundUseId);
dense_id!(SourceConditionFormulaEdgeId);
dense_id!(SourcePredicateChainConjunctionId);
dense_id!(SourcePredicateChainNegationId);

/// Complete input for one cross-family source formula composition transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFormulaCompositionHandoffInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub atomic_edges: Vec<SourceFormulaAtomicEdgeInput>,
    pub bound_uses: Vec<SourceQuantifierBoundUseInput>,
}

/// One composite-formula-to-atomic-formula association.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFormulaAtomicEdgeInput {
    pub formula: SourceCompositeFormulaId,
    pub ordinal: usize,
    pub role: SourceFormulaAtomicEdgeRole,
    pub child: SourceAtomicFormulaId,
}

/// One ordinary bound-variable occurrence selected by lexical lookup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceQuantifierBoundUseInput {
    pub binder: SourceQuantifierBinderId,
    pub ordinal: usize,
    pub body_edge: SourceFormulaAtomicEdgeId,
    pub term: SourcePrimaryTermId,
    pub reference: SourcePrimaryTermReferenceId,
}

/// Complete input for one condition-to-atomic-formula association transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceConditionFormulaCompositionHandoffInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub edges: Vec<SourceConditionFormulaEdgeInput>,
}

/// One source-set condition to atomic-formula association.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceConditionFormulaEdgeInput {
    pub condition: SourceSetConditionId,
    pub ordinal: usize,
    pub formula: SourceAtomicFormulaId,
}

/// Complete input for one predicate-chain composition transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePredicateChainCompositionHandoffInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub conjunctions: Vec<SourcePredicateChainConjunctionInput>,
    pub negations: Vec<SourcePredicateChainNegationInput>,
}

/// One association between adjacent predicate-chain segments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePredicateChainConjunctionInput {
    pub formula: SourceAtomicFormulaId,
    pub ordinal: usize,
    pub left_segment: SourcePredicateSegmentId,
    pub right_segment: SourcePredicateSegmentId,
    pub boundary: SourceAtomicEdgeId,
}

/// One association between a predicate chain and a negated segment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePredicateChainNegationInput {
    pub formula: SourceAtomicFormulaId,
    pub ordinal: usize,
    pub segment: SourcePredicateSegmentId,
}

/// Cross-family role of an atomic formula under a composite formula.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceFormulaAtomicEdgeRole {
    UniversalBody,
    UniversalRestriction,
    ConjunctionLeft,
    ConjunctionRight,
    DisjunctionLeft,
    DisjunctionRight,
}

/// Immutable validated formula composition handoff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFormulaCompositionHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    primary_term_fingerprint: String,
    atomic_formula_fingerprint: String,
    composite_formula_fingerprint: String,
    atomic_edges: SourceFormulaAtomicEdgeTable,
    bound_uses: SourceQuantifierBoundUseTable,
}

/// Immutable validated condition-to-atomic-formula composition handoff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceConditionFormulaCompositionHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    primary_term_fingerprint: String,
    application_fingerprint: String,
    set_term_fingerprint: String,
    atomic_formula_fingerprint: String,
    edges: SourceConditionFormulaEdgeTable,
}

/// Immutable validated predicate-chain composition handoff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePredicateChainCompositionHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    primary_term_fingerprint: String,
    atomic_formula_fingerprint: String,
    conjunctions: SourcePredicateChainConjunctionTable,
    negations: SourcePredicateChainNegationTable,
}

impl SourcePredicateChainCompositionHandoff {
    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    pub fn primary_term_fingerprint(&self) -> &str {
        &self.primary_term_fingerprint
    }

    pub fn atomic_formula_fingerprint(&self) -> &str {
        &self.atomic_formula_fingerprint
    }

    pub const fn conjunctions(&self) -> &SourcePredicateChainConjunctionTable {
        &self.conjunctions
    }

    pub const fn negations(&self) -> &SourcePredicateChainNegationTable {
        &self.negations
    }

    pub fn debug_text(&self) -> String {
        let mut output = String::from("source-predicate-chain-composition-debug-v1\n");
        let _ = writeln!(
            output,
            "module: {}::{}",
            self.module_id.package().as_str(),
            self.module_id.path().as_str()
        );
        let _ = writeln!(
            output,
            "primary-term-fingerprint: {:?}",
            self.primary_term_fingerprint
        );
        let _ = writeln!(
            output,
            "atomic-formula-fingerprint: {:?}",
            self.atomic_formula_fingerprint
        );
        let _ = writeln!(output, "conjunctions: {}", self.conjunctions.len());
        for (id, row) in self.conjunctions.iter() {
            let _ = writeln!(
                output,
                "  conjunction#{} formula={} ordinal={} left_segment={} right_segment={} boundary={}",
                id.index(),
                row.formula.index(),
                row.ordinal,
                row.left_segment.index(),
                row.right_segment.index(),
                row.boundary.index(),
            );
        }
        let _ = writeln!(output, "negations: {}", self.negations.len());
        for (id, row) in self.negations.iter() {
            let _ = writeln!(
                output,
                "  negation#{} formula={} ordinal={} segment={}",
                id.index(),
                row.formula.index(),
                row.ordinal,
                row.segment.index(),
            );
        }
        output
    }

    pub(crate) fn validate_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        primary_terms: &SourcePrimaryTermHandoff,
        atomic_formulas: &SourceAtomicFormulaHandoff,
        arena: &TypedArena,
    ) -> Result<(), SourcePredicateChainCompositionError> {
        if self.source_id != source_id
            || &self.module_id != module_id
            || self.primary_term_fingerprint.is_empty()
            || self.atomic_formula_fingerprint.is_empty()
            || self.primary_term_fingerprint != primary_terms.debug_text()
            || self.atomic_formula_fingerprint != atomic_formulas.debug_text()
        {
            return Err(SourcePredicateChainCompositionError::DependencyMismatch);
        }
        validate_predicate_chain_transaction(
            &SourcePredicateChainCompositionHandoffInput {
                source_id: self.source_id,
                module_id: self.module_id.clone(),
                conjunctions: self.conjunctions.rows.iter().map(Into::into).collect(),
                negations: self.negations.rows.iter().map(Into::into).collect(),
            },
            primary_terms,
            atomic_formulas,
            arena,
        )
    }
}

impl SourceConditionFormulaCompositionHandoff {
    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    pub fn primary_term_fingerprint(&self) -> &str {
        &self.primary_term_fingerprint
    }

    pub fn application_fingerprint(&self) -> &str {
        &self.application_fingerprint
    }

    pub fn set_term_fingerprint(&self) -> &str {
        &self.set_term_fingerprint
    }

    pub fn atomic_formula_fingerprint(&self) -> &str {
        &self.atomic_formula_fingerprint
    }

    pub const fn edges(&self) -> &SourceConditionFormulaEdgeTable {
        &self.edges
    }

    pub fn debug_text(&self) -> String {
        let mut output = String::from("source-condition-formula-composition-debug-v1\n");
        let _ = writeln!(
            output,
            "module: {}::{}",
            self.module_id.package().as_str(),
            self.module_id.path().as_str()
        );
        let _ = writeln!(
            output,
            "primary-term-fingerprint: {:?}",
            self.primary_term_fingerprint
        );
        let _ = writeln!(
            output,
            "application-fingerprint: {:?}",
            self.application_fingerprint
        );
        let _ = writeln!(
            output,
            "set-term-fingerprint: {:?}",
            self.set_term_fingerprint
        );
        let _ = writeln!(
            output,
            "atomic-formula-fingerprint: {:?}",
            self.atomic_formula_fingerprint
        );
        let _ = writeln!(output, "edges: {}", self.edges.len());
        for (id, edge) in self.edges.iter() {
            let _ = writeln!(
                output,
                "  edge#{} condition={} ordinal={} formula={}",
                id.index(),
                edge.condition.index(),
                edge.ordinal,
                edge.formula.index(),
            );
        }
        output
    }

    #[allow(clippy::too_many_arguments)] // Rationale: installation must reauthenticate every frozen lower-family dependency explicitly.
    pub(crate) fn validate_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        primary_terms: &SourcePrimaryTermHandoff,
        applications: &SourceFunctorApplicationHandoff,
        set_terms: &SourceSetTermHandoff,
        atomic_formulas: &SourceAtomicFormulaHandoff,
        arena: &TypedArena,
    ) -> Result<(), SourceConditionFormulaCompositionError> {
        if self.source_id != source_id
            || &self.module_id != module_id
            || self.primary_term_fingerprint.is_empty()
            || self.application_fingerprint.is_empty()
            || self.set_term_fingerprint.is_empty()
            || self.atomic_formula_fingerprint.is_empty()
            || self.primary_term_fingerprint != primary_terms.debug_text()
            || self.application_fingerprint != applications.debug_text()
            || self.set_term_fingerprint != set_terms.debug_text()
            || self.atomic_formula_fingerprint != atomic_formulas.debug_text()
        {
            return Err(SourceConditionFormulaCompositionError::DependencyMismatch);
        }
        validate_condition_transaction(
            &SourceConditionFormulaCompositionHandoffInput {
                source_id: self.source_id,
                module_id: self.module_id.clone(),
                edges: self.edges.rows.iter().map(Into::into).collect(),
            },
            primary_terms,
            applications,
            set_terms,
            atomic_formulas,
            arena,
        )
    }
}

impl SourceFormulaCompositionHandoff {
    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    pub fn primary_term_fingerprint(&self) -> &str {
        &self.primary_term_fingerprint
    }

    pub fn atomic_formula_fingerprint(&self) -> &str {
        &self.atomic_formula_fingerprint
    }

    pub fn composite_formula_fingerprint(&self) -> &str {
        &self.composite_formula_fingerprint
    }

    pub const fn atomic_edges(&self) -> &SourceFormulaAtomicEdgeTable {
        &self.atomic_edges
    }

    pub const fn bound_uses(&self) -> &SourceQuantifierBoundUseTable {
        &self.bound_uses
    }

    pub fn debug_text(&self) -> String {
        let mut output = String::from("source-formula-composition-debug-v1\n");
        let _ = writeln!(
            output,
            "module: {}::{}",
            self.module_id.package().as_str(),
            self.module_id.path().as_str()
        );
        let _ = writeln!(
            output,
            "primary-term-fingerprint: {:?}",
            self.primary_term_fingerprint
        );
        let _ = writeln!(
            output,
            "atomic-formula-fingerprint: {:?}",
            self.atomic_formula_fingerprint
        );
        let _ = writeln!(
            output,
            "composite-formula-fingerprint: {:?}",
            self.composite_formula_fingerprint
        );
        let _ = writeln!(output, "atomic-edges: {}", self.atomic_edges.len());
        for (id, row) in self.atomic_edges.iter() {
            let _ = writeln!(
                output,
                "  atomic-edge#{} formula={} ordinal={} role={} child={}",
                id.index(),
                row.formula.index(),
                row.ordinal,
                atomic_edge_role_key(row.role),
                row.child.index(),
            );
        }
        let _ = writeln!(output, "bound-uses: {}", self.bound_uses.len());
        for (id, row) in self.bound_uses.iter() {
            let _ = writeln!(
                output,
                "  bound-use#{} binder={} ordinal={} body-edge={} term={} reference={}",
                id.index(),
                row.binder.index(),
                row.ordinal,
                row.body_edge.index(),
                row.term.index(),
                row.reference.index(),
            );
        }
        output
    }

    pub(crate) fn validate_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        primary_terms: &SourcePrimaryTermHandoff,
        atomic_formulas: &SourceAtomicFormulaHandoff,
        composite_formulas: &SourceCompositeFormulaHandoff,
        arena: &TypedArena,
    ) -> Result<(), SourceFormulaCompositionError> {
        if self.source_id != source_id
            || &self.module_id != module_id
            || self.primary_term_fingerprint.is_empty()
            || self.atomic_formula_fingerprint.is_empty()
            || self.composite_formula_fingerprint.is_empty()
            || self.primary_term_fingerprint != primary_terms.debug_text()
            || self.atomic_formula_fingerprint != atomic_formulas.debug_text()
            || self.composite_formula_fingerprint != composite_formulas.debug_text()
        {
            return Err(SourceFormulaCompositionError::DependencyMismatch);
        }
        let input = SourceFormulaCompositionHandoffInput {
            source_id: self.source_id,
            module_id: self.module_id.clone(),
            atomic_edges: self.atomic_edges.rows.iter().map(Into::into).collect(),
            bound_uses: self.bound_uses.rows.iter().map(Into::into).collect(),
        };
        validate_transaction(
            &input,
            primary_terms,
            atomic_formulas,
            composite_formulas,
            arena,
        )
    }
}

macro_rules! table {
    ($name:ident, $row:ident, $id:ident) => {
        #[derive(Debug, Clone, Default, PartialEq, Eq)]
        pub struct $name {
            rows: Vec<$row>,
        }

        impl $name {
            pub fn get(&self, id: $id) -> Option<&$row> {
                self.rows.get(id.index())
            }

            pub fn iter(&self) -> impl Iterator<Item = ($id, &$row)> {
                self.rows
                    .iter()
                    .enumerate()
                    .map(|(index, row)| ($id::new(index), row))
            }

            pub const fn len(&self) -> usize {
                self.rows.len()
            }

            pub const fn is_empty(&self) -> bool {
                self.rows.is_empty()
            }
        }
    };
}

table!(
    SourceFormulaAtomicEdgeTable,
    SourceFormulaAtomicEdge,
    SourceFormulaAtomicEdgeId
);
table!(
    SourceQuantifierBoundUseTable,
    SourceQuantifierBoundUse,
    SourceQuantifierBoundUseId
);
table!(
    SourceConditionFormulaEdgeTable,
    SourceConditionFormulaEdge,
    SourceConditionFormulaEdgeId
);
table!(
    SourcePredicateChainConjunctionTable,
    SourcePredicateChainConjunction,
    SourcePredicateChainConjunctionId
);
table!(
    SourcePredicateChainNegationTable,
    SourcePredicateChainNegation,
    SourcePredicateChainNegationId
);

/// One validated composite-formula-to-atomic-formula association.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFormulaAtomicEdge {
    formula: SourceCompositeFormulaId,
    ordinal: usize,
    role: SourceFormulaAtomicEdgeRole,
    child: SourceAtomicFormulaId,
}

impl SourceFormulaAtomicEdge {
    pub const fn formula(&self) -> SourceCompositeFormulaId {
        self.formula
    }

    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }

    pub const fn role(&self) -> SourceFormulaAtomicEdgeRole {
        self.role
    }

    pub const fn child(&self) -> SourceAtomicFormulaId {
        self.child
    }
}

/// One validated quantifier bound-variable use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceQuantifierBoundUse {
    binder: SourceQuantifierBinderId,
    ordinal: usize,
    body_edge: SourceFormulaAtomicEdgeId,
    term: SourcePrimaryTermId,
    reference: SourcePrimaryTermReferenceId,
}

/// One validated source-set condition to atomic-formula association.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceConditionFormulaEdge {
    condition: SourceSetConditionId,
    ordinal: usize,
    formula: SourceAtomicFormulaId,
}

impl SourceConditionFormulaEdge {
    pub const fn condition(&self) -> SourceSetConditionId {
        self.condition
    }

    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }

    pub const fn formula(&self) -> SourceAtomicFormulaId {
        self.formula
    }
}

/// One validated association between adjacent predicate-chain segments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePredicateChainConjunction {
    formula: SourceAtomicFormulaId,
    ordinal: usize,
    left_segment: SourcePredicateSegmentId,
    right_segment: SourcePredicateSegmentId,
    boundary: SourceAtomicEdgeId,
}

impl SourcePredicateChainConjunction {
    pub const fn formula(&self) -> SourceAtomicFormulaId {
        self.formula
    }

    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }

    pub const fn left_segment(&self) -> SourcePredicateSegmentId {
        self.left_segment
    }

    pub const fn right_segment(&self) -> SourcePredicateSegmentId {
        self.right_segment
    }

    pub const fn boundary(&self) -> SourceAtomicEdgeId {
        self.boundary
    }
}

/// One validated association between a predicate chain and a negated segment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePredicateChainNegation {
    formula: SourceAtomicFormulaId,
    ordinal: usize,
    segment: SourcePredicateSegmentId,
}

impl SourcePredicateChainNegation {
    pub const fn formula(&self) -> SourceAtomicFormulaId {
        self.formula
    }

    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }

    pub const fn segment(&self) -> SourcePredicateSegmentId {
        self.segment
    }
}

impl SourceQuantifierBoundUse {
    pub const fn binder(&self) -> SourceQuantifierBinderId {
        self.binder
    }

    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }

    pub const fn body_edge(&self) -> SourceFormulaAtomicEdgeId {
        self.body_edge
    }

    pub const fn term(&self) -> SourcePrimaryTermId {
        self.term
    }

    pub const fn reference(&self) -> SourcePrimaryTermReferenceId {
        self.reference
    }
}

impl From<SourceFormulaAtomicEdgeInput> for SourceFormulaAtomicEdge {
    fn from(input: SourceFormulaAtomicEdgeInput) -> Self {
        Self {
            formula: input.formula,
            ordinal: input.ordinal,
            role: input.role,
            child: input.child,
        }
    }
}

impl From<&SourceFormulaAtomicEdge> for SourceFormulaAtomicEdgeInput {
    fn from(row: &SourceFormulaAtomicEdge) -> Self {
        Self {
            formula: row.formula,
            ordinal: row.ordinal,
            role: row.role,
            child: row.child,
        }
    }
}

impl From<SourceQuantifierBoundUseInput> for SourceQuantifierBoundUse {
    fn from(input: SourceQuantifierBoundUseInput) -> Self {
        Self {
            binder: input.binder,
            ordinal: input.ordinal,
            body_edge: input.body_edge,
            term: input.term,
            reference: input.reference,
        }
    }
}

impl From<&SourceQuantifierBoundUse> for SourceQuantifierBoundUseInput {
    fn from(row: &SourceQuantifierBoundUse) -> Self {
        Self {
            binder: row.binder,
            ordinal: row.ordinal,
            body_edge: row.body_edge,
            term: row.term,
            reference: row.reference,
        }
    }
}

impl From<SourceConditionFormulaEdgeInput> for SourceConditionFormulaEdge {
    fn from(input: SourceConditionFormulaEdgeInput) -> Self {
        Self {
            condition: input.condition,
            ordinal: input.ordinal,
            formula: input.formula,
        }
    }
}

impl From<&SourceConditionFormulaEdge> for SourceConditionFormulaEdgeInput {
    fn from(row: &SourceConditionFormulaEdge) -> Self {
        Self {
            condition: row.condition,
            ordinal: row.ordinal,
            formula: row.formula,
        }
    }
}

impl From<SourcePredicateChainConjunctionInput> for SourcePredicateChainConjunction {
    fn from(input: SourcePredicateChainConjunctionInput) -> Self {
        Self {
            formula: input.formula,
            ordinal: input.ordinal,
            left_segment: input.left_segment,
            right_segment: input.right_segment,
            boundary: input.boundary,
        }
    }
}

impl From<&SourcePredicateChainConjunction> for SourcePredicateChainConjunctionInput {
    fn from(row: &SourcePredicateChainConjunction) -> Self {
        Self {
            formula: row.formula,
            ordinal: row.ordinal,
            left_segment: row.left_segment,
            right_segment: row.right_segment,
            boundary: row.boundary,
        }
    }
}

impl From<SourcePredicateChainNegationInput> for SourcePredicateChainNegation {
    fn from(input: SourcePredicateChainNegationInput) -> Self {
        Self {
            formula: input.formula,
            ordinal: input.ordinal,
            segment: input.segment,
        }
    }
}

impl From<&SourcePredicateChainNegation> for SourcePredicateChainNegationInput {
    fn from(row: &SourcePredicateChainNegation) -> Self {
        Self {
            formula: row.formula,
            ordinal: row.ordinal,
            segment: row.segment,
        }
    }
}

/// Validates and publishes the exact Task 257B1/B2/B3 cross-family associations.
#[derive(Debug, Clone, Copy, Default)]
pub struct SourceFormulaCompositionProducer;

impl SourceFormulaCompositionProducer {
    pub fn build(
        input: SourceFormulaCompositionHandoffInput,
        primary_terms: &SourcePrimaryTermHandoff,
        atomic_formulas: &SourceAtomicFormulaHandoff,
        composite_formulas: &SourceCompositeFormulaHandoff,
        arena: &TypedArena,
    ) -> Result<SourceFormulaCompositionHandoff, SourceFormulaCompositionError> {
        validate_transaction(
            &input,
            primary_terms,
            atomic_formulas,
            composite_formulas,
            arena,
        )?;
        let primary_term_fingerprint = primary_terms.debug_text();
        let atomic_formula_fingerprint = atomic_formulas.debug_text();
        let composite_formula_fingerprint = composite_formulas.debug_text();
        if primary_term_fingerprint.is_empty()
            || atomic_formula_fingerprint.is_empty()
            || composite_formula_fingerprint.is_empty()
        {
            return Err(SourceFormulaCompositionError::DependencyMismatch);
        }
        Ok(SourceFormulaCompositionHandoff {
            source_id: input.source_id,
            module_id: input.module_id,
            primary_term_fingerprint,
            atomic_formula_fingerprint,
            composite_formula_fingerprint,
            atomic_edges: SourceFormulaAtomicEdgeTable {
                rows: input.atomic_edges.into_iter().map(Into::into).collect(),
            },
            bound_uses: SourceQuantifierBoundUseTable {
                rows: input.bound_uses.into_iter().map(Into::into).collect(),
            },
        })
    }
}

/// Validates and publishes the exact Task 257C2 condition/formula association.
#[derive(Debug, Clone, Copy, Default)]
pub struct SourceConditionFormulaCompositionProducer;

impl SourceConditionFormulaCompositionProducer {
    pub fn build(
        input: SourceConditionFormulaCompositionHandoffInput,
        primary_terms: &SourcePrimaryTermHandoff,
        applications: &SourceFunctorApplicationHandoff,
        set_terms: &SourceSetTermHandoff,
        atomic_formulas: &SourceAtomicFormulaHandoff,
        arena: &TypedArena,
    ) -> Result<SourceConditionFormulaCompositionHandoff, SourceConditionFormulaCompositionError>
    {
        validate_condition_transaction(
            &input,
            primary_terms,
            applications,
            set_terms,
            atomic_formulas,
            arena,
        )?;
        let primary_term_fingerprint = primary_terms.debug_text();
        let application_fingerprint = applications.debug_text();
        let set_term_fingerprint = set_terms.debug_text();
        let atomic_formula_fingerprint = atomic_formulas.debug_text();
        if primary_term_fingerprint.is_empty()
            || application_fingerprint.is_empty()
            || set_term_fingerprint.is_empty()
            || atomic_formula_fingerprint.is_empty()
        {
            return Err(SourceConditionFormulaCompositionError::DependencyMismatch);
        }
        Ok(SourceConditionFormulaCompositionHandoff {
            source_id: input.source_id,
            module_id: input.module_id,
            primary_term_fingerprint,
            application_fingerprint,
            set_term_fingerprint,
            atomic_formula_fingerprint,
            edges: SourceConditionFormulaEdgeTable {
                rows: input.edges.into_iter().map(Into::into).collect(),
            },
        })
    }
}

/// Validates and publishes the exact Task 257C3 predicate-chain composition.
#[derive(Debug, Clone, Copy, Default)]
pub struct SourcePredicateChainCompositionProducer;

impl SourcePredicateChainCompositionProducer {
    pub fn build(
        input: SourcePredicateChainCompositionHandoffInput,
        primary_terms: &SourcePrimaryTermHandoff,
        atomic_formulas: &SourceAtomicFormulaHandoff,
        arena: &TypedArena,
    ) -> Result<SourcePredicateChainCompositionHandoff, SourcePredicateChainCompositionError> {
        validate_predicate_chain_transaction(&input, primary_terms, atomic_formulas, arena)?;
        let primary_term_fingerprint = primary_terms.debug_text();
        let atomic_formula_fingerprint = atomic_formulas.debug_text();
        if primary_term_fingerprint.is_empty() || atomic_formula_fingerprint.is_empty() {
            return Err(SourcePredicateChainCompositionError::DependencyMismatch);
        }
        Ok(SourcePredicateChainCompositionHandoff {
            source_id: input.source_id,
            module_id: input.module_id,
            primary_term_fingerprint,
            atomic_formula_fingerprint,
            conjunctions: SourcePredicateChainConjunctionTable {
                rows: input.conjunctions.into_iter().map(Into::into).collect(),
            },
            negations: SourcePredicateChainNegationTable {
                rows: input.negations.into_iter().map(Into::into).collect(),
            },
        })
    }
}

/// Formula composition transaction validation failure.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceFormulaCompositionError {
    DependencyMismatch,
    InvalidAtomicEdge {
        edge: SourceFormulaAtomicEdgeId,
    },
    InvalidBoundUse {
        bound_use: SourceQuantifierBoundUseId,
    },
    InvalidAggregate,
}

impl fmt::Display for SourceFormulaCompositionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DependencyMismatch => {
                formatter.write_str("source formula composition dependency mismatch")
            }
            Self::InvalidAtomicEdge { edge } => write!(
                formatter,
                "source formula atomic edge {} is invalid",
                edge.index()
            ),
            Self::InvalidBoundUse { bound_use } => write!(
                formatter,
                "source quantifier bound use {} is invalid",
                bound_use.index()
            ),
            Self::InvalidAggregate => {
                formatter.write_str("source formula composition aggregate is invalid")
            }
        }
    }
}

impl Error for SourceFormulaCompositionError {}

/// Condition/formula composition transaction validation failure.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceConditionFormulaCompositionError {
    DependencyMismatch,
    InvalidEdge { edge: SourceConditionFormulaEdgeId },
    InvalidAggregate,
}

impl fmt::Display for SourceConditionFormulaCompositionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DependencyMismatch => {
                formatter.write_str("source condition/formula composition dependency mismatch")
            }
            Self::InvalidEdge { edge } => write!(
                formatter,
                "source condition/formula edge {} is invalid",
                edge.index()
            ),
            Self::InvalidAggregate => {
                formatter.write_str("source condition/formula composition aggregate is invalid")
            }
        }
    }
}

impl Error for SourceConditionFormulaCompositionError {}

/// Predicate-chain composition transaction validation failure.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourcePredicateChainCompositionError {
    DependencyMismatch,
    InvalidConjunction {
        conjunction: SourcePredicateChainConjunctionId,
    },
    InvalidNegation {
        negation: SourcePredicateChainNegationId,
    },
    InvalidAggregate,
}

impl fmt::Display for SourcePredicateChainCompositionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DependencyMismatch => {
                formatter.write_str("source predicate-chain composition dependency mismatch")
            }
            Self::InvalidConjunction { conjunction } => write!(
                formatter,
                "source predicate-chain conjunction {} is invalid",
                conjunction.index()
            ),
            Self::InvalidNegation { negation } => write!(
                formatter,
                "source predicate-chain negation {} is invalid",
                negation.index()
            ),
            Self::InvalidAggregate => {
                formatter.write_str("source predicate-chain composition aggregate is invalid")
            }
        }
    }
}

impl Error for SourcePredicateChainCompositionError {}

fn validate_predicate_chain_transaction(
    input: &SourcePredicateChainCompositionHandoffInput,
    primary_terms: &SourcePrimaryTermHandoff,
    atomic_formulas: &SourceAtomicFormulaHandoff,
    arena: &TypedArena,
) -> Result<(), SourcePredicateChainCompositionError> {
    if input.source_id != primary_terms.source_id()
        || input.source_id != atomic_formulas.source_id()
        || &input.module_id != primary_terms.module_id()
        || &input.module_id != atomic_formulas.module_id()
    {
        return Err(SourcePredicateChainCompositionError::DependencyMismatch);
    }
    primary_terms
        .validate_installation(input.source_id, &input.module_id, arena)
        .map_err(|_| SourcePredicateChainCompositionError::DependencyMismatch)?;
    atomic_formulas
        .validate_installation(
            input.source_id,
            &input.module_id,
            primary_terms,
            None,
            None,
            None,
            arena,
        )
        .map_err(|_| SourcePredicateChainCompositionError::DependencyMismatch)?;
    validate_predicate_chain_dependency_profiles(primary_terms, atomic_formulas)?;

    let ([conjunction], [negation]) = (input.conjunctions.as_slice(), input.negations.as_slice())
    else {
        return Err(SourcePredicateChainCompositionError::InvalidAggregate);
    };

    let conjunction_id = SourcePredicateChainConjunctionId::new(0);
    let left = atomic_formulas
        .predicate_segments()
        .get(conjunction.left_segment)
        .ok_or(SourcePredicateChainCompositionError::InvalidConjunction {
            conjunction: conjunction_id,
        })?;
    let right = atomic_formulas
        .predicate_segments()
        .get(conjunction.right_segment)
        .ok_or(SourcePredicateChainCompositionError::InvalidConjunction {
            conjunction: conjunction_id,
        })?;
    let boundary = atomic_formulas.edges().get(conjunction.boundary).ok_or(
        SourcePredicateChainCompositionError::InvalidConjunction {
            conjunction: conjunction_id,
        },
    )?;
    if conjunction.formula != SourceAtomicFormulaId::new(0)
        || conjunction.ordinal != 0
        || conjunction.left_segment != SourcePredicateSegmentId::new(0)
        || conjunction.right_segment != SourcePredicateSegmentId::new(1)
        || conjunction.boundary != SourceAtomicEdgeId::new(1)
        || left.formula() != conjunction.formula
        || right.formula() != conjunction.formula
        || left.right_edge() != conjunction.boundary
        || right.left_edge() != conjunction.boundary
        || boundary.formula() != conjunction.formula
        || boundary.ordinal() != 1
        || boundary.role() != SourceAtomicEdgeRole::PredicateChainBoundary
        || boundary.target() != SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(1))
    {
        return Err(SourcePredicateChainCompositionError::InvalidConjunction {
            conjunction: conjunction_id,
        });
    }

    let negation_id = SourcePredicateChainNegationId::new(0);
    let segment = atomic_formulas
        .predicate_segments()
        .get(negation.segment)
        .ok_or(SourcePredicateChainCompositionError::InvalidNegation {
            negation: negation_id,
        })?;
    if negation.formula != SourceAtomicFormulaId::new(0)
        || negation.ordinal != 0
        || negation.segment != SourcePredicateSegmentId::new(1)
        || segment.formula() != negation.formula
        || !matches!(
            segment.polarity(),
            SourcePredicateSegmentPolarityInput::Negative { .. }
        )
    {
        return Err(SourcePredicateChainCompositionError::InvalidNegation {
            negation: negation_id,
        });
    }
    Ok(())
}

fn validate_predicate_chain_dependency_profiles(
    primary_terms: &SourcePrimaryTermHandoff,
    atomic_formulas: &SourceAtomicFormulaHandoff,
) -> Result<(), SourcePredicateChainCompositionError> {
    if primary_terms.terms().len() != 3
        || !primary_terms.references().is_empty()
        || primary_terms.numeric_type_requests().len() != 3
        || atomic_formulas.formulas().len() != 1
        || !atomic_formulas.wrappers().is_empty()
        || atomic_formulas.predicate_segments().len() != 2
        || atomic_formulas.predicate_heads().len() != 2
        || atomic_formulas.candidates().len() != 2
        || !atomic_formulas.type_sites().is_empty()
        || !atomic_formulas.attributes().is_empty()
        || atomic_formulas.edges().len() != 3
        || atomic_formulas.requests().len() != 2
    {
        return Err(SourcePredicateChainCompositionError::DependencyMismatch);
    }

    for (index, (start, end, spelling)) in [(75, 76, "1"), (85, 86, "2"), (104, 105, "3")]
        .into_iter()
        .enumerate()
    {
        let term = primary_terms
            .terms()
            .get(SourcePrimaryTermId::new(index))
            .ok_or(SourcePredicateChainCompositionError::DependencyMismatch)?;
        let request = primary_terms
            .numeric_type_requests()
            .get(SourceNumericTypeRequestId::new(index))
            .ok_or(SourcePredicateChainCompositionError::DependencyMismatch)?;
        if term.source_ordinal() != index
            || term.source_range().start != start
            || term.source_range().end != end
            || term.context() != crate::binding_env::BindingContextId::new(0)
            || term.recovery() != SourcePrimaryTermRecovery::Normal
            || term.spelling() != spelling
            || term.kind() != SourcePrimaryTermKind::Numeral
            || term.role() != SourcePrimaryTermRole::Value
            || term.parent().is_some()
            || request.term() != SourcePrimaryTermId::new(index)
            || request.owner() != term.site()
            || request.source_range() != term.source_range()
            || request.spelling() != term.spelling()
            || request.request_ordinal() != index
        {
            return Err(SourcePredicateChainCompositionError::DependencyMismatch);
        }
    }

    let formula = atomic_formulas
        .formulas()
        .get(SourceAtomicFormulaId::new(0))
        .ok_or(SourcePredicateChainCompositionError::DependencyMismatch)?;
    if formula.source_ordinal() != 0
        || formula.source_range().start != 75
        || formula.source_range().end != 105
        || formula.context() != crate::binding_env::BindingContextId::new(0)
        || formula.recovery() != SourceAtomicFormulaRecovery::Normal
        || formula.spelling() != "1 divides 2 does not divides 3"
        || formula.kind() != SourceAtomicFormulaKind::PredicateApplication
    {
        return Err(SourcePredicateChainCompositionError::DependencyMismatch);
    }

    for (index, (start, end, spelling, head, left_edge, right_edge)) in [
        (75, 86, "1 divides 2", 0, 0, 1),
        (87, 105, "does not divides 3", 1, 1, 2),
    ]
    .into_iter()
    .enumerate()
    {
        let segment = atomic_formulas
            .predicate_segments()
            .get(SourcePredicateSegmentId::new(index))
            .ok_or(SourcePredicateChainCompositionError::DependencyMismatch)?;
        if segment.formula() != SourceAtomicFormulaId::new(0)
            || segment.ordinal() != index
            || segment.source_range().start != start
            || segment.source_range().end != end
            || segment.context() != crate::binding_env::BindingContextId::new(0)
            || segment.recovery() != SourceAtomicFormulaRecovery::Normal
            || segment.spelling() != spelling
            || segment.head() != SourcePredicateHeadId::new(head)
            || segment.left_edge() != SourceAtomicEdgeId::new(left_edge)
            || segment.right_edge() != SourceAtomicEdgeId::new(right_edge)
        {
            return Err(SourcePredicateChainCompositionError::DependencyMismatch);
        }
    }
    if !matches!(
        atomic_formulas
            .predicate_segments()
            .get(SourcePredicateSegmentId::new(0))
            .map(|segment| segment.polarity()),
        Some(SourcePredicateSegmentPolarityInput::Positive)
    ) {
        return Err(SourcePredicateChainCompositionError::DependencyMismatch);
    }
    let Some(SourcePredicateSegmentPolarityInput::Negative {
        verb_range,
        verb_spelling,
        verb_recovery,
        not_range,
        not_spelling,
        not_recovery,
        ..
    }) = atomic_formulas
        .predicate_segments()
        .get(SourcePredicateSegmentId::new(1))
        .map(|segment| segment.polarity())
    else {
        return Err(SourcePredicateChainCompositionError::DependencyMismatch);
    };
    if verb_range.start != 87
        || verb_range.end != 91
        || verb_spelling != "does"
        || *verb_recovery != SourceAtomicFormulaRecovery::Normal
        || not_range.start != 92
        || not_range.end != 95
        || not_spelling != "not"
        || *not_recovery != SourceAtomicFormulaRecovery::Normal
    {
        return Err(SourcePredicateChainCompositionError::DependencyMismatch);
    }

    for (index, (start, end)) in [(77, 84), (96, 103)].into_iter().enumerate() {
        let head = atomic_formulas
            .predicate_heads()
            .get(SourcePredicateHeadId::new(index))
            .ok_or(SourcePredicateChainCompositionError::DependencyMismatch)?;
        let candidate = atomic_formulas
            .candidates()
            .get(SourcePredicateCandidateId::new(index))
            .ok_or(SourcePredicateChainCompositionError::DependencyMismatch)?;
        if head.formula() != SourceAtomicFormulaId::new(0)
            || head.source_range().start != start
            || head.source_range().end != end
            || head.context() != crate::binding_env::BindingContextId::new(0)
            || head.recovery() != SourceAtomicFormulaRecovery::Normal
            || head.spelling() != "divides"
            || head.left_arity() != 1
            || head.right_arity() != 1
            || candidate.head() != SourcePredicateHeadId::new(index)
            || candidate.ordinal() != 0
        {
            return Err(SourcePredicateChainCompositionError::DependencyMismatch);
        }
    }
    let candidate0 = atomic_formulas
        .candidates()
        .get(SourcePredicateCandidateId::new(0))
        .ok_or(SourcePredicateChainCompositionError::DependencyMismatch)?;
    let candidate1 = atomic_formulas
        .candidates()
        .get(SourcePredicateCandidateId::new(1))
        .ok_or(SourcePredicateChainCompositionError::DependencyMismatch)?;
    if candidate0.symbol() != candidate1.symbol()
        || candidate0.contribution() != candidate1.contribution()
    {
        return Err(SourcePredicateChainCompositionError::DependencyMismatch);
    }

    for (index, role, term) in [
        (0, SourceAtomicEdgeRole::PredicateLeftArgument, 0),
        (1, SourceAtomicEdgeRole::PredicateChainBoundary, 1),
        (2, SourceAtomicEdgeRole::PredicateRightArgument, 2),
    ] {
        let edge = atomic_formulas
            .edges()
            .get(SourceAtomicEdgeId::new(index))
            .ok_or(SourcePredicateChainCompositionError::DependencyMismatch)?;
        if edge.formula() != SourceAtomicFormulaId::new(0)
            || edge.ordinal() != index
            || edge.role() != role
            || edge.target() != SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(term))
        {
            return Err(SourcePredicateChainCompositionError::DependencyMismatch);
        }
    }
    for index in 0..2 {
        let request = atomic_formulas
            .requests()
            .get(SourceAtomicRequestId::new(index))
            .ok_or(SourcePredicateChainCompositionError::DependencyMismatch)?;
        if request.formula() != SourceAtomicFormulaId::new(0)
            || request.ordinal() != index
            || request.kind() != SourceAtomicRequestKind::PredicateCandidateSignature
            || request.edge().is_some()
            || request.candidate() != Some(SourcePredicateCandidateId::new(index))
            || request.type_site().is_some()
            || request.attribute().is_some()
        {
            return Err(SourcePredicateChainCompositionError::DependencyMismatch);
        }
    }
    Ok(())
}

fn validate_condition_transaction(
    input: &SourceConditionFormulaCompositionHandoffInput,
    primary_terms: &SourcePrimaryTermHandoff,
    applications: &SourceFunctorApplicationHandoff,
    set_terms: &SourceSetTermHandoff,
    atomic_formulas: &SourceAtomicFormulaHandoff,
    arena: &TypedArena,
) -> Result<(), SourceConditionFormulaCompositionError> {
    if input.source_id != primary_terms.source_id()
        || input.source_id != applications.source_id()
        || input.source_id != set_terms.source_id()
        || input.source_id != atomic_formulas.source_id()
        || &input.module_id != primary_terms.module_id()
        || &input.module_id != applications.module_id()
        || &input.module_id != set_terms.module_id()
        || &input.module_id != atomic_formulas.module_id()
    {
        return Err(SourceConditionFormulaCompositionError::DependencyMismatch);
    }
    primary_terms
        .validate_installation(input.source_id, &input.module_id, arena)
        .map_err(|_| SourceConditionFormulaCompositionError::DependencyMismatch)?;
    applications
        .validate_installation(input.source_id, &input.module_id, primary_terms)
        .map_err(|_| SourceConditionFormulaCompositionError::DependencyMismatch)?;
    set_terms
        .validate_installation(
            input.source_id,
            &input.module_id,
            primary_terms,
            Some(applications),
            None,
            arena,
        )
        .map_err(|_| SourceConditionFormulaCompositionError::DependencyMismatch)?;
    atomic_formulas
        .validate_installation(
            input.source_id,
            &input.module_id,
            primary_terms,
            Some(applications),
            None,
            Some(set_terms),
            arena,
        )
        .map_err(|_| SourceConditionFormulaCompositionError::DependencyMismatch)?;

    validate_condition_dependency_profiles(
        primary_terms,
        applications,
        set_terms,
        atomic_formulas,
        arena,
    )?;

    let [edge] = input.edges.as_slice() else {
        return Err(SourceConditionFormulaCompositionError::InvalidAggregate);
    };
    let condition = set_terms.conditions().get(edge.condition).ok_or(
        SourceConditionFormulaCompositionError::InvalidEdge {
            edge: SourceConditionFormulaEdgeId::new(0),
        },
    )?;
    let formula = atomic_formulas.formulas().get(edge.formula).ok_or(
        SourceConditionFormulaCompositionError::InvalidEdge {
            edge: SourceConditionFormulaEdgeId::new(0),
        },
    )?;
    if edge.condition != SourceSetConditionId::new(0)
        || edge.ordinal != 0
        || edge.formula != SourceAtomicFormulaId::new(0)
        || condition.ordinal() != edge.ordinal
        || condition.source_range() != formula.source_range()
        || condition.spelling() != formula.spelling()
        || condition.recovery() != SourceSetTermRecovery::Normal
        || formula.recovery() != SourceAtomicFormulaRecovery::Normal
        || condition.condition_site() == formula.site()
        || !arena
            .node(condition.condition_site().node())
            .is_some_and(|node| node.children.contains(&formula.site().node()))
    {
        return Err(SourceConditionFormulaCompositionError::InvalidEdge {
            edge: SourceConditionFormulaEdgeId::new(0),
        });
    }
    Ok(())
}

fn validate_condition_dependency_profiles(
    primary_terms: &SourcePrimaryTermHandoff,
    applications: &SourceFunctorApplicationHandoff,
    set_terms: &SourceSetTermHandoff,
    atomic_formulas: &SourceAtomicFormulaHandoff,
    arena: &TypedArena,
) -> Result<(), SourceConditionFormulaCompositionError> {
    if primary_terms.terms().len() != 4
        || !primary_terms.references().is_empty()
        || primary_terms.numeric_type_requests().len() != 4
        || applications.applications().len() != 1
        || !applications.wrappers().is_empty()
        || applications.candidates().len() != 1
        || applications.arguments().len() != 2
        || applications.type_requests().len() != 2
        || set_terms.terms().len() != 1
        || !set_terms.wrappers().is_empty()
        || set_terms.generators().len() != 1
        || set_terms.type_sites().len() != 1
        || set_terms.conditions().len() != 1
        || set_terms.edges().len() != 1
        || set_terms.requests().len() != 2
        || atomic_formulas.formulas().len() != 1
        || !atomic_formulas.wrappers().is_empty()
        || !atomic_formulas.predicate_segments().is_empty()
        || !atomic_formulas.predicate_heads().is_empty()
        || !atomic_formulas.candidates().is_empty()
        || !atomic_formulas.type_sites().is_empty()
        || !atomic_formulas.attributes().is_empty()
        || atomic_formulas.edges().len() != 2
        || atomic_formulas.requests().len() != 2
    {
        return Err(SourceConditionFormulaCompositionError::InvalidAggregate);
    }

    for (index, (start, end, spelling)) in [
        (141, 142, "1"),
        (146, 147, "2"),
        (177, 178, "3"),
        (181, 182, "4"),
    ]
    .into_iter()
    .enumerate()
    {
        let term = primary_terms
            .terms()
            .get(SourcePrimaryTermId::new(index))
            .ok_or(SourceConditionFormulaCompositionError::InvalidAggregate)?;
        let request = primary_terms
            .numeric_type_requests()
            .get(SourceNumericTypeRequestId::new(index))
            .ok_or(SourceConditionFormulaCompositionError::InvalidAggregate)?;
        if term.source_ordinal() != index
            || term.source_range().start != start
            || term.source_range().end != end
            || term.context() != crate::binding_env::BindingContextId::new(0)
            || term.recovery() != SourcePrimaryTermRecovery::Normal
            || term.spelling() != spelling
            || term.kind() != SourcePrimaryTermKind::Numeral
            || term.role() != SourcePrimaryTermRole::Value
            || term.parent().is_some()
            || request.term() != SourcePrimaryTermId::new(index)
            || request.owner() != term.site()
            || request.source_range() != term.source_range()
            || request.spelling() != term.spelling()
            || request.request_ordinal() != index
        {
            return Err(SourceConditionFormulaCompositionError::InvalidAggregate);
        }
    }

    let application = applications
        .applications()
        .get(SourceFunctorApplicationId::new(0))
        .ok_or(SourceConditionFormulaCompositionError::InvalidAggregate)?;
    if application.source_range().start != 141
        || application.source_range().end != 147
        || application.source_ordinal() != 0
        || application.context() != crate::binding_env::BindingContextId::new(0)
        || application.recovery() != SourceFunctorApplicationRecovery::Normal
        || application.spelling() != "1 ++ 2"
        || application.kind() != SourceFunctorApplicationKind::Symbolic
        || application.form() != SourceFunctorApplicationForm::Infix
        || application.head_ordinal() != 1
    {
        return Err(SourceConditionFormulaCompositionError::InvalidAggregate);
    }
    match application.head() {
        SourceFunctorHeadSite::Single {
            source_range,
            spelling,
            ..
        } if source_range.start == 143 && source_range.end == 145 && spelling == "++" => {}
        _ => return Err(SourceConditionFormulaCompositionError::InvalidAggregate),
    }
    let candidate = applications
        .candidates()
        .get(SourceFunctorCandidateId::new(0))
        .ok_or(SourceConditionFormulaCompositionError::InvalidAggregate)?;
    if candidate.application() != SourceFunctorApplicationId::new(0) || candidate.ordinal() != 0 {
        return Err(SourceConditionFormulaCompositionError::InvalidAggregate);
    }
    for (index, primary) in [0, 1].into_iter().enumerate() {
        let argument = applications
            .arguments()
            .get(SourceFunctorArgumentId::new(index))
            .ok_or(SourceConditionFormulaCompositionError::InvalidAggregate)?;
        if argument.application() != SourceFunctorApplicationId::new(0)
            || argument.ordinal() != index
            || argument.target()
                != SourceFunctorArgumentTarget::Primary(SourcePrimaryTermId::new(primary))
        {
            return Err(SourceConditionFormulaCompositionError::InvalidAggregate);
        }
    }
    for (index, request) in applications.type_requests().iter() {
        let (candidate, kind) = match index.index() {
            0 => (
                Some(SourceFunctorCandidateId::new(0)),
                SourceFunctorTypeRequestKind::CandidateSignature,
            ),
            1 => (None, SourceFunctorTypeRequestKind::ApplicationResultType),
            _ => return Err(SourceConditionFormulaCompositionError::InvalidAggregate),
        };
        if request.application() != SourceFunctorApplicationId::new(0)
            || request.candidate() != candidate
            || request.request_ordinal() != index.index()
            || request.kind() != kind
        {
            return Err(SourceConditionFormulaCompositionError::InvalidAggregate);
        }
    }

    let set_term = set_terms
        .terms()
        .get(SourceSetTermId::new(0))
        .ok_or(SourceConditionFormulaCompositionError::InvalidAggregate)?;
    let condition = set_terms
        .conditions()
        .get(SourceSetConditionId::new(0))
        .ok_or(SourceConditionFormulaCompositionError::InvalidAggregate)?;
    let generator = set_terms
        .generators()
        .get(SourceSetGeneratorId::new(0))
        .ok_or(SourceConditionFormulaCompositionError::InvalidAggregate)?;
    let type_site = set_terms
        .type_sites()
        .get(SourceSetTypeSiteId::new(0))
        .ok_or(SourceConditionFormulaCompositionError::InvalidAggregate)?;
    let set_edge = set_terms
        .edges()
        .get(SourceSetEdgeId::new(0))
        .ok_or(SourceConditionFormulaCompositionError::InvalidAggregate)?;
    let formula = atomic_formulas
        .formulas()
        .get(SourceAtomicFormulaId::new(0))
        .ok_or(SourceConditionFormulaCompositionError::InvalidAggregate)?;
    if set_term.source_ordinal() != 0
        || set_term.context() != crate::binding_env::BindingContextId::new(0)
        || set_term.kind() != SourceSetTermKind::Comprehension
        || set_term.recovery() != SourceSetTermRecovery::Normal
        || set_term.source_range().start != 139
        || set_term.source_range().end != 184
        || set_term.spelling() != "{ 1 ++ 2 where candidate255c is set : 3 = 4 }"
        || generator.term() != SourceSetTermId::new(0)
        || generator.ordinal() != 0
        || generator.source_range().start != 154
        || generator.source_range().end != 167
        || generator.spelling() != "candidate255c"
        || generator.context() != set_term.context()
        || generator.recovery() != SourceSetTermRecovery::Normal
        || generator.type_site() != SourceSetTypeSiteId::new(0)
        || type_site.owner() != SourceSetTypeOwner::Generator(SourceSetGeneratorId::new(0))
        || type_site.source_range().start != 171
        || type_site.source_range().end != 174
        || type_site.spelling() != "set"
        || type_site.head_range().start != 171
        || type_site.head_range().end != 174
        || type_site.head_spelling() != "set"
        || type_site.context() != set_term.context()
        || type_site.recovery() != SourceSetTermRecovery::Normal
        || type_site.head() != SourceSetTypeHead::BuiltinSet
        || condition.term() != SourceSetTermId::new(0)
        || condition.ordinal() != 0
        || condition.colon_range().start != 175
        || condition.colon_range().end != 176
        || condition.colon_spelling() != ":"
        || condition.source_range().start != 177
        || condition.source_range().end != 182
        || condition.spelling() != "3 = 4"
        || set_edge.term() != SourceSetTermId::new(0)
        || set_edge.ordinal() != 0
        || set_edge.role() != SourceSetEdgeRole::ComprehensionMapper
        || set_edge.target() != SourceSetTarget::Application(SourceFunctorApplicationId::new(0))
        || formula.source_ordinal() != 0
        || formula.kind() != SourceAtomicFormulaKind::Equality
        || formula.recovery() != SourceAtomicFormulaRecovery::Normal
        || formula.source_range() != condition.source_range()
        || formula.spelling() != condition.spelling()
        || formula.context() != set_term.context()
        || !properly_contains(set_term.source_range(), formula.source_range())
        || condition.condition_site() == formula.site()
        || !arena
            .node(condition.condition_site().node())
            .is_some_and(|node| node.children.contains(&formula.site().node()))
    {
        return Err(SourceConditionFormulaCompositionError::InvalidAggregate);
    }
    for (index, request) in set_terms.requests().iter() {
        let (kind, generator, type_site) = match index.index() {
            0 => (
                SourceSetRequestKind::GeneratorSethood,
                Some(SourceSetGeneratorId::new(0)),
                Some(SourceSetTypeSiteId::new(0)),
            ),
            1 => (SourceSetRequestKind::ResultType, None, None),
            _ => return Err(SourceConditionFormulaCompositionError::InvalidAggregate),
        };
        if request.term() != SourceSetTermId::new(0)
            || request.ordinal() != index.index()
            || request.kind() != kind
            || request.generator() != generator
            || request.type_site() != type_site
        {
            return Err(SourceConditionFormulaCompositionError::InvalidAggregate);
        }
    }

    for (index, role, term) in [
        (0, SourceAtomicEdgeRole::BuiltinLeftOperand, 2),
        (1, SourceAtomicEdgeRole::BuiltinRightOperand, 3),
    ] {
        let edge = atomic_formulas
            .edges()
            .get(SourceAtomicEdgeId::new(index))
            .ok_or(SourceConditionFormulaCompositionError::InvalidAggregate)?;
        let request = atomic_formulas
            .requests()
            .get(crate::source_atomic_formula::SourceAtomicRequestId::new(
                index,
            ))
            .ok_or(SourceConditionFormulaCompositionError::InvalidAggregate)?;
        if edge.formula() != SourceAtomicFormulaId::new(0)
            || edge.ordinal() != index
            || edge.role() != role
            || edge.target() != SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(term))
            || request.formula() != SourceAtomicFormulaId::new(0)
            || request.ordinal() != index
            || request.kind() != SourceAtomicRequestKind::OperandExpectedType
            || request.edge() != Some(SourceAtomicEdgeId::new(index))
            || request.candidate().is_some()
            || request.type_site().is_some()
            || request.attribute().is_some()
        {
            return Err(SourceConditionFormulaCompositionError::InvalidAggregate);
        }
    }
    Ok(())
}

fn validate_transaction(
    input: &SourceFormulaCompositionHandoffInput,
    primary_terms: &SourcePrimaryTermHandoff,
    atomic_formulas: &SourceAtomicFormulaHandoff,
    composite_formulas: &SourceCompositeFormulaHandoff,
    arena: &TypedArena,
) -> Result<(), SourceFormulaCompositionError> {
    if input.source_id != primary_terms.source_id()
        || input.source_id != atomic_formulas.source_id()
        || input.source_id != composite_formulas.source_id()
        || &input.module_id != primary_terms.module_id()
        || &input.module_id != atomic_formulas.module_id()
        || &input.module_id != composite_formulas.module_id()
    {
        return Err(SourceFormulaCompositionError::DependencyMismatch);
    }
    primary_terms
        .validate_installation(input.source_id, &input.module_id, arena)
        .map_err(|_| SourceFormulaCompositionError::DependencyMismatch)?;
    atomic_formulas
        .validate_installation(
            input.source_id,
            &input.module_id,
            primary_terms,
            None,
            None,
            None,
            arena,
        )
        .map_err(|_| SourceFormulaCompositionError::DependencyMismatch)?;
    composite_formulas
        .validate_installation(input.source_id, &input.module_id, arena)
        .map_err(|_| SourceFormulaCompositionError::DependencyMismatch)?;

    validate_dependency_profiles(primary_terms, atomic_formulas, composite_formulas)?;
    validate_atomic_edge(input, atomic_formulas, composite_formulas)?;
    validate_bound_uses(input, primary_terms, atomic_formulas, composite_formulas)
}

fn validate_dependency_profiles(
    primary_terms: &SourcePrimaryTermHandoff,
    atomic_formulas: &SourceAtomicFormulaHandoff,
    composite_formulas: &SourceCompositeFormulaHandoff,
) -> Result<(), SourceFormulaCompositionError> {
    if composite_formulas.is_task_257b3_profile() {
        return validate_task_257b3_dependency_profiles(
            primary_terms,
            atomic_formulas,
            composite_formulas,
        );
    }
    if composite_formulas.is_task_257b2_profile() {
        return validate_task_257b2_dependency_profiles(
            primary_terms,
            atomic_formulas,
            composite_formulas,
        );
    }
    if primary_terms.terms().len() != 2
        || primary_terms.references().len() != 2
        || !primary_terms.numeric_type_requests().is_empty()
        || atomic_formulas.formulas().len() != 1
        || !atomic_formulas.wrappers().is_empty()
        || !atomic_formulas.predicate_heads().is_empty()
        || !atomic_formulas.candidates().is_empty()
        || !atomic_formulas.type_sites().is_empty()
        || !atomic_formulas.attributes().is_empty()
        || atomic_formulas.edges().len() != 2
        || atomic_formulas.requests().len() != 2
        || !composite_formulas.is_task_257b1_profile()
    {
        return Err(SourceFormulaCompositionError::InvalidAggregate);
    }

    let Some(atomic) = atomic_formulas
        .formulas()
        .get(SourceAtomicFormulaId::new(0))
    else {
        return Err(SourceFormulaCompositionError::InvalidAggregate);
    };
    let Some(composite) = composite_formulas
        .formulas()
        .get(SourceCompositeFormulaId::new(0))
    else {
        return Err(SourceFormulaCompositionError::InvalidAggregate);
    };
    let Some(binder) = composite_formulas
        .binders()
        .get(SourceQuantifierBinderId::new(0))
    else {
        return Err(SourceFormulaCompositionError::InvalidAggregate);
    };
    if atomic.kind() != SourceAtomicFormulaKind::Equality
        || atomic.recovery() != SourceAtomicFormulaRecovery::Normal
        || atomic.context() != binder.body_context()
        || composite.kind() != SourceCompositeFormulaKind::Universal
        || composite.recovery() != SourceCompositeFormulaRecovery::Normal
        || composite.context().index() != 0
        || binder.formula() != SourceCompositeFormulaId::new(0)
        || !properly_contains(composite.source_range(), atomic.source_range())
    {
        return Err(SourceFormulaCompositionError::InvalidAggregate);
    }

    for (index, role) in [
        SourceAtomicEdgeRole::BuiltinLeftOperand,
        SourceAtomicEdgeRole::BuiltinRightOperand,
    ]
    .into_iter()
    .enumerate()
    {
        let Some(edge) = atomic_formulas.edges().get(SourceAtomicEdgeId::new(index)) else {
            return Err(SourceFormulaCompositionError::InvalidAggregate);
        };
        let Some(request) = atomic_formulas.requests().get(
            crate::source_atomic_formula::SourceAtomicRequestId::new(index),
        ) else {
            return Err(SourceFormulaCompositionError::InvalidAggregate);
        };
        if edge.formula() != SourceAtomicFormulaId::new(0)
            || edge.ordinal() != index
            || edge.role() != role
            || edge.target() != SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(index))
            || request.formula() != SourceAtomicFormulaId::new(0)
            || request.ordinal() != index
            || request.kind() != SourceAtomicRequestKind::OperandExpectedType
            || request.edge() != Some(SourceAtomicEdgeId::new(index))
            || request.candidate().is_some()
            || request.type_site().is_some()
            || request.attribute().is_some()
        {
            return Err(SourceFormulaCompositionError::InvalidAggregate);
        }
    }
    Ok(())
}

fn validate_task_257b3_dependency_profiles(
    primary_terms: &SourcePrimaryTermHandoff,
    atomic_formulas: &SourceAtomicFormulaHandoff,
    composite_formulas: &SourceCompositeFormulaHandoff,
) -> Result<(), SourceFormulaCompositionError> {
    if primary_terms.terms().len() != 6
        || primary_terms.references().len() != 6
        || !primary_terms.numeric_type_requests().is_empty()
        || atomic_formulas.formulas().len() != 3
        || !atomic_formulas.wrappers().is_empty()
        || !atomic_formulas.predicate_heads().is_empty()
        || !atomic_formulas.candidates().is_empty()
        || !atomic_formulas.type_sites().is_empty()
        || !atomic_formulas.attributes().is_empty()
        || atomic_formulas.edges().len() != 6
        || atomic_formulas.requests().len() != 6
    {
        return Err(SourceFormulaCompositionError::InvalidAggregate);
    }
    let binders = (0..3)
        .map(|index| {
            composite_formulas
                .binders()
                .get(SourceQuantifierBinderId::new(index))
                .ok_or(SourceFormulaCompositionError::InvalidAggregate)
        })
        .collect::<Result<Vec<_>, _>>()?;
    if composite_formulas
        .binding_env()
        .bindings()
        .iter()
        .any(|(_, binding)| !binding.captured.identities().is_empty())
    {
        return Err(SourceFormulaCompositionError::InvalidAggregate);
    }

    let term_spellings = ["x", "x", "r", "y", "x", "r"];
    let term_contexts = [1, 1, 3, 3, 3, 3];
    let term_bindings = [1, 1, 3, 2, 1, 3];
    let use_ordinals = [2, 2, 4, 4, 4, 4];
    let use_scopes = [
        &[0_u32][..],
        &[0_u32][..],
        &[0_u32, 0, 0][..],
        &[0_u32, 0, 0][..],
        &[0_u32, 0, 0][..],
        &[0_u32, 0, 0][..],
    ];
    for index in 0..6 {
        let term = primary_terms
            .terms()
            .get(SourcePrimaryTermId::new(index))
            .ok_or(SourceFormulaCompositionError::InvalidAggregate)?;
        let reference = primary_terms
            .references()
            .get(SourcePrimaryTermReferenceId::new(index))
            .ok_or(SourceFormulaCompositionError::InvalidAggregate)?;
        if term.kind() != SourcePrimaryTermKind::VariableReference
            || term.role() != SourcePrimaryTermRole::Value
            || term.recovery() != SourcePrimaryTermRecovery::Normal
            || term.spelling() != term_spellings[index]
            || term.context().index() != term_contexts[index]
            || term.parent().is_some()
            || reference.term() != SourcePrimaryTermId::new(index)
            || reference.binding().index() != term_bindings[index]
            || reference.role() != SourcePrimaryTermReferenceRole::Variable
            || reference
                .lexical_scope()
                .is_none_or(|scope| scope.path() != use_scopes[index])
            || reference.use_ordinal() != use_ordinals[index]
        {
            return Err(SourceFormulaCompositionError::InvalidAggregate);
        }
    }

    let equality_spellings = ["x = x", "r = y", "x = r"];
    let equality_contexts = [1, 3, 3];
    for formula_index in 0..3 {
        let formula = atomic_formulas
            .formulas()
            .get(SourceAtomicFormulaId::new(formula_index))
            .ok_or(SourceFormulaCompositionError::InvalidAggregate)?;
        if formula.kind() != SourceAtomicFormulaKind::Equality
            || formula.recovery() != SourceAtomicFormulaRecovery::Normal
            || formula.context().index() != equality_contexts[formula_index]
            || formula.spelling() != equality_spellings[formula_index]
        {
            return Err(SourceFormulaCompositionError::InvalidAggregate);
        }
        for (ordinal, role) in [
            SourceAtomicEdgeRole::BuiltinLeftOperand,
            SourceAtomicEdgeRole::BuiltinRightOperand,
        ]
        .into_iter()
        .enumerate()
        {
            let index = formula_index * 2 + ordinal;
            let edge = atomic_formulas
                .edges()
                .get(SourceAtomicEdgeId::new(index))
                .ok_or(SourceFormulaCompositionError::InvalidAggregate)?;
            let request = atomic_formulas
                .requests()
                .get(crate::source_atomic_formula::SourceAtomicRequestId::new(
                    index,
                ))
                .ok_or(SourceFormulaCompositionError::InvalidAggregate)?;
            if edge.formula() != SourceAtomicFormulaId::new(formula_index)
                || edge.ordinal() != ordinal
                || edge.role() != role
                || edge.target() != SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(index))
                || request.formula() != SourceAtomicFormulaId::new(formula_index)
                || request.ordinal() != ordinal
                || request.kind() != SourceAtomicRequestKind::OperandExpectedType
                || request.edge() != Some(SourceAtomicEdgeId::new(index))
                || request.candidate().is_some()
                || request.type_site().is_some()
                || request.attribute().is_some()
            {
                return Err(SourceFormulaCompositionError::InvalidAggregate);
            }
        }
    }
    if binders[0].binding().index() != 1
        || binders[1].binding().index() != 2
        || binders[2].binding().index() != 3
    {
        return Err(SourceFormulaCompositionError::InvalidAggregate);
    }
    Ok(())
}

fn validate_task_257b2_dependency_profiles(
    primary_terms: &SourcePrimaryTermHandoff,
    atomic_formulas: &SourceAtomicFormulaHandoff,
    composite_formulas: &SourceCompositeFormulaHandoff,
) -> Result<(), SourceFormulaCompositionError> {
    if primary_terms.terms().len() != 16
        || !primary_terms.references().is_empty()
        || primary_terms.numeric_type_requests().len() != 16
        || atomic_formulas.formulas().len() != 8
        || !atomic_formulas.wrappers().is_empty()
        || !atomic_formulas.predicate_heads().is_empty()
        || !atomic_formulas.candidates().is_empty()
        || !atomic_formulas.type_sites().is_empty()
        || !atomic_formulas.attributes().is_empty()
        || atomic_formulas.edges().len() != 16
        || atomic_formulas.requests().len() != 16
    {
        return Err(SourceFormulaCompositionError::InvalidAggregate);
    }
    let binder = composite_formulas
        .binders()
        .get(SourceQuantifierBinderId::new(0))
        .ok_or(SourceFormulaCompositionError::InvalidAggregate)?;
    let binding = composite_formulas
        .binding_env()
        .bindings()
        .get(binder.binding())
        .ok_or(SourceFormulaCompositionError::InvalidAggregate)?;
    if !binding.captured.identities().is_empty() {
        return Err(SourceFormulaCompositionError::InvalidAggregate);
    }

    let numeral_spellings = [
        "0", "0", "0", "3", "0", "0", "0", "3", "0", "0", "0", "0", "0", "0", "0", "0",
    ];
    for (index, spelling) in numeral_spellings.into_iter().enumerate() {
        let term = primary_terms
            .terms()
            .get(SourcePrimaryTermId::new(index))
            .ok_or(SourceFormulaCompositionError::InvalidAggregate)?;
        let request = primary_terms
            .numeric_type_requests()
            .get(crate::source_term::SourceNumericTypeRequestId::new(index))
            .ok_or(SourceFormulaCompositionError::InvalidAggregate)?;
        if term.kind() != SourcePrimaryTermKind::Numeral
            || term.role() != SourcePrimaryTermRole::Value
            || term.recovery() != SourcePrimaryTermRecovery::Normal
            || term.spelling() != spelling
            || term.context() != binder.body_context()
            || term.parent().is_some()
            || request.term() != SourcePrimaryTermId::new(index)
            || request.owner() != term.site()
            || request.source_range() != term.source_range()
            || request.spelling() != spelling
            || request.request_ordinal() != index
        {
            return Err(SourceFormulaCompositionError::InvalidAggregate);
        }
    }

    let equality_spellings = [
        "0 = 0", "0 = 3", "0 = 0", "0 = 3", "0 = 0", "0 = 0", "0 = 0", "0 = 0",
    ];
    for (formula_index, spelling) in equality_spellings.into_iter().enumerate() {
        let formula = atomic_formulas
            .formulas()
            .get(SourceAtomicFormulaId::new(formula_index))
            .ok_or(SourceFormulaCompositionError::InvalidAggregate)?;
        if formula.kind() != SourceAtomicFormulaKind::Equality
            || formula.recovery() != SourceAtomicFormulaRecovery::Normal
            || formula.context() != binder.body_context()
            || formula.spelling() != spelling
        {
            return Err(SourceFormulaCompositionError::InvalidAggregate);
        }
        for (ordinal, role) in [
            SourceAtomicEdgeRole::BuiltinLeftOperand,
            SourceAtomicEdgeRole::BuiltinRightOperand,
        ]
        .into_iter()
        .enumerate()
        {
            let index = formula_index * 2 + ordinal;
            let edge = atomic_formulas
                .edges()
                .get(SourceAtomicEdgeId::new(index))
                .ok_or(SourceFormulaCompositionError::InvalidAggregate)?;
            let request = atomic_formulas
                .requests()
                .get(crate::source_atomic_formula::SourceAtomicRequestId::new(
                    index,
                ))
                .ok_or(SourceFormulaCompositionError::InvalidAggregate)?;
            if edge.formula() != SourceAtomicFormulaId::new(formula_index)
                || edge.ordinal() != ordinal
                || edge.role() != role
                || edge.target() != SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(index))
                || request.formula() != SourceAtomicFormulaId::new(formula_index)
                || request.ordinal() != ordinal
                || request.kind() != SourceAtomicRequestKind::OperandExpectedType
                || request.edge() != Some(SourceAtomicEdgeId::new(index))
                || request.candidate().is_some()
                || request.type_site().is_some()
                || request.attribute().is_some()
            {
                return Err(SourceFormulaCompositionError::InvalidAggregate);
            }
        }
    }
    Ok(())
}

fn validate_atomic_edge(
    input: &SourceFormulaCompositionHandoffInput,
    atomic_formulas: &SourceAtomicFormulaHandoff,
    composite_formulas: &SourceCompositeFormulaHandoff,
) -> Result<(), SourceFormulaCompositionError> {
    if composite_formulas.is_task_257b3_profile() {
        return validate_task_257b3_atomic_edges(input, atomic_formulas, composite_formulas);
    }
    if composite_formulas.is_task_257b2_profile() {
        return validate_task_257b2_atomic_edges(input, atomic_formulas, composite_formulas);
    }
    let [edge] = input.atomic_edges.as_slice() else {
        return Err(SourceFormulaCompositionError::InvalidAggregate);
    };
    let Some(parent) = composite_formulas.formulas().get(edge.formula) else {
        return Err(SourceFormulaCompositionError::InvalidAtomicEdge {
            edge: SourceFormulaAtomicEdgeId::new(0),
        });
    };
    let Some(child) = atomic_formulas.formulas().get(edge.child) else {
        return Err(SourceFormulaCompositionError::InvalidAtomicEdge {
            edge: SourceFormulaAtomicEdgeId::new(0),
        });
    };
    if edge.formula != SourceCompositeFormulaId::new(0)
        || edge.ordinal != 0
        || edge.role != SourceFormulaAtomicEdgeRole::UniversalBody
        || edge.child != SourceAtomicFormulaId::new(0)
        || !properly_contains(parent.source_range(), child.source_range())
    {
        return Err(SourceFormulaCompositionError::InvalidAtomicEdge {
            edge: SourceFormulaAtomicEdgeId::new(0),
        });
    }
    Ok(())
}

fn validate_task_257b3_atomic_edges(
    input: &SourceFormulaCompositionHandoffInput,
    atomic_formulas: &SourceAtomicFormulaHandoff,
    composite_formulas: &SourceCompositeFormulaHandoff,
) -> Result<(), SourceFormulaCompositionError> {
    let expected = [
        (0, 0, SourceFormulaAtomicEdgeRole::UniversalRestriction, 0),
        (2, 0, SourceFormulaAtomicEdgeRole::UniversalRestriction, 1),
        (2, 1, SourceFormulaAtomicEdgeRole::UniversalBody, 2),
    ];
    if input.atomic_edges.len() != expected.len() {
        return Err(SourceFormulaCompositionError::InvalidAggregate);
    }
    for (index, (row, (formula, ordinal, role, child))) in
        input.atomic_edges.iter().zip(expected).enumerate()
    {
        let parent = composite_formulas
            .formulas()
            .get(SourceCompositeFormulaId::new(formula))
            .ok_or(SourceFormulaCompositionError::InvalidAtomicEdge {
                edge: SourceFormulaAtomicEdgeId::new(index),
            })?;
        let atomic = atomic_formulas
            .formulas()
            .get(SourceAtomicFormulaId::new(child))
            .ok_or(SourceFormulaCompositionError::InvalidAtomicEdge {
                edge: SourceFormulaAtomicEdgeId::new(index),
            })?;
        let parent_id = SourceCompositeFormulaId::new(formula);
        let has_deeper_composite_owner =
            composite_formulas
                .formulas()
                .iter()
                .any(|(candidate_id, candidate)| {
                    candidate_id != parent_id
                        && properly_contains(parent.source_range(), candidate.source_range())
                        && properly_contains(candidate.source_range(), atomic.source_range())
                });
        if row.formula != SourceCompositeFormulaId::new(formula)
            || row.ordinal != ordinal
            || row.role != role
            || row.child != SourceAtomicFormulaId::new(child)
            || !properly_contains(parent.source_range(), atomic.source_range())
            || has_deeper_composite_owner
        {
            return Err(SourceFormulaCompositionError::InvalidAtomicEdge {
                edge: SourceFormulaAtomicEdgeId::new(index),
            });
        }
    }
    Ok(())
}

fn validate_task_257b2_atomic_edges(
    input: &SourceFormulaCompositionHandoffInput,
    atomic_formulas: &SourceAtomicFormulaHandoff,
    composite_formulas: &SourceCompositeFormulaHandoff,
) -> Result<(), SourceFormulaCompositionError> {
    let expected = [
        (3, 0, SourceFormulaAtomicEdgeRole::ConjunctionLeft, 0),
        (3, 1, SourceFormulaAtomicEdgeRole::ConjunctionRight, 1),
        (4, 0, SourceFormulaAtomicEdgeRole::DisjunctionLeft, 2),
        (4, 1, SourceFormulaAtomicEdgeRole::DisjunctionRight, 3),
        (6, 0, SourceFormulaAtomicEdgeRole::ConjunctionLeft, 4),
        (6, 1, SourceFormulaAtomicEdgeRole::ConjunctionRight, 5),
        (7, 0, SourceFormulaAtomicEdgeRole::DisjunctionLeft, 6),
        (7, 1, SourceFormulaAtomicEdgeRole::DisjunctionRight, 7),
    ];
    if input.atomic_edges.len() != expected.len() {
        return Err(SourceFormulaCompositionError::InvalidAggregate);
    }
    for (index, (row, (formula, ordinal, role, child))) in
        input.atomic_edges.iter().zip(expected).enumerate()
    {
        let parent = composite_formulas
            .formulas()
            .get(SourceCompositeFormulaId::new(formula))
            .ok_or(SourceFormulaCompositionError::InvalidAtomicEdge {
                edge: SourceFormulaAtomicEdgeId::new(index),
            })?;
        let atomic = atomic_formulas
            .formulas()
            .get(SourceAtomicFormulaId::new(child))
            .ok_or(SourceFormulaCompositionError::InvalidAtomicEdge {
                edge: SourceFormulaAtomicEdgeId::new(index),
            })?;
        if row.formula != SourceCompositeFormulaId::new(formula)
            || row.ordinal != ordinal
            || row.role != role
            || row.child != SourceAtomicFormulaId::new(child)
            || !properly_contains(parent.source_range(), atomic.source_range())
        {
            return Err(SourceFormulaCompositionError::InvalidAtomicEdge {
                edge: SourceFormulaAtomicEdgeId::new(index),
            });
        }
    }
    Ok(())
}

fn validate_bound_uses(
    input: &SourceFormulaCompositionHandoffInput,
    primary_terms: &SourcePrimaryTermHandoff,
    atomic_formulas: &SourceAtomicFormulaHandoff,
    composite_formulas: &SourceCompositeFormulaHandoff,
) -> Result<(), SourceFormulaCompositionError> {
    if composite_formulas.is_task_257b3_profile() {
        return validate_task_257b3_bound_uses(
            input,
            primary_terms,
            atomic_formulas,
            composite_formulas,
        );
    }
    if composite_formulas.is_task_257b2_profile() {
        let binder = composite_formulas
            .binders()
            .get(SourceQuantifierBinderId::new(0))
            .ok_or(SourceFormulaCompositionError::InvalidAggregate)?;
        let binding = composite_formulas
            .binding_env()
            .bindings()
            .get(binder.binding())
            .ok_or(SourceFormulaCompositionError::InvalidAggregate)?;
        return if input.bound_uses.is_empty()
            && primary_terms.references().is_empty()
            && binding.captured.identities().is_empty()
        {
            Ok(())
        } else {
            Err(SourceFormulaCompositionError::InvalidAggregate)
        };
    }
    if input.bound_uses.len() != 2 {
        return Err(SourceFormulaCompositionError::InvalidAggregate);
    }
    let binder = composite_formulas
        .binders()
        .get(SourceQuantifierBinderId::new(0))
        .ok_or(SourceFormulaCompositionError::InvalidAggregate)?;
    let equality = atomic_formulas
        .formulas()
        .get(SourceAtomicFormulaId::new(0))
        .ok_or(SourceFormulaCompositionError::InvalidAggregate)?;

    let mut previous_range: Option<SourceRange> = None;
    for (index, row) in input.bound_uses.iter().enumerate() {
        let id = SourceQuantifierBoundUseId::new(index);
        let Some(term) = primary_terms.terms().get(row.term) else {
            return Err(SourceFormulaCompositionError::InvalidBoundUse { bound_use: id });
        };
        let Some(reference) = primary_terms.references().get(row.reference) else {
            return Err(SourceFormulaCompositionError::InvalidBoundUse { bound_use: id });
        };
        if row.binder != SourceQuantifierBinderId::new(0)
            || row.ordinal != index
            || row.body_edge != SourceFormulaAtomicEdgeId::new(0)
            || row.term != SourcePrimaryTermId::new(index)
            || row.reference != SourcePrimaryTermReferenceId::new(index)
            || term.kind() != SourcePrimaryTermKind::VariableReference
            || term.role() != SourcePrimaryTermRole::Value
            || term.recovery() != SourcePrimaryTermRecovery::Normal
            || term.spelling() != "x"
            || term.context() != binder.body_context()
            || term.parent().is_some()
            || !properly_contains(equality.source_range(), term.source_range())
            || reference.term() != row.term
            || reference.binding() != binder.binding()
            || reference.role() != SourcePrimaryTermReferenceRole::Variable
            || reference.lexical_scope() != Some(binder.local().scope())
            || reference.use_ordinal() != 1
        {
            return Err(SourceFormulaCompositionError::InvalidBoundUse { bound_use: id });
        }
        let lookup = BindingLookupSite::new(
            term.spelling(),
            term.context(),
            reference.lexical_scope().cloned(),
            reference.use_ordinal(),
        );
        if composite_formulas.binding_env().lookup(&lookup)
            != Ok(BindingLookupResult::Local(binder.binding()))
        {
            return Err(SourceFormulaCompositionError::InvalidBoundUse { bound_use: id });
        }
        if previous_range.is_some_and(|previous| previous.end > term.source_range().start) {
            return Err(SourceFormulaCompositionError::InvalidBoundUse { bound_use: id });
        }
        previous_range = Some(term.source_range());
    }
    Ok(())
}

fn validate_task_257b3_bound_uses(
    input: &SourceFormulaCompositionHandoffInput,
    primary_terms: &SourcePrimaryTermHandoff,
    atomic_formulas: &SourceAtomicFormulaHandoff,
    composite_formulas: &SourceCompositeFormulaHandoff,
) -> Result<(), SourceFormulaCompositionError> {
    let binder_ids = [0, 0, 2, 1, 0, 2];
    let ordinals = [0, 1, 0, 0, 2, 1];
    let owning_edges = [0, 0, 1, 1, 2, 2];
    let use_ordinals = [2, 2, 4, 4, 4, 4];
    if input.bound_uses.len() != 6 {
        return Err(SourceFormulaCompositionError::InvalidAggregate);
    }
    let mut previous_range: Option<SourceRange> = None;
    for (index, row) in input.bound_uses.iter().enumerate() {
        let id = SourceQuantifierBoundUseId::new(index);
        let binder = composite_formulas
            .binders()
            .get(SourceQuantifierBinderId::new(binder_ids[index]))
            .ok_or(SourceFormulaCompositionError::InvalidBoundUse { bound_use: id })?;
        let owner_edge = input
            .atomic_edges
            .get(owning_edges[index])
            .ok_or(SourceFormulaCompositionError::InvalidBoundUse { bound_use: id })?;
        let owner = atomic_formulas
            .formulas()
            .get(owner_edge.child)
            .ok_or(SourceFormulaCompositionError::InvalidBoundUse { bound_use: id })?;
        let term = primary_terms
            .terms()
            .get(row.term)
            .ok_or(SourceFormulaCompositionError::InvalidBoundUse { bound_use: id })?;
        let reference = primary_terms
            .references()
            .get(row.reference)
            .ok_or(SourceFormulaCompositionError::InvalidBoundUse { bound_use: id })?;
        let context = composite_formulas
            .binding_env()
            .contexts()
            .get(term.context())
            .ok_or(SourceFormulaCompositionError::InvalidBoundUse { bound_use: id })?;
        if row.binder != SourceQuantifierBinderId::new(binder_ids[index])
            || row.ordinal != ordinals[index]
            || row.body_edge != SourceFormulaAtomicEdgeId::new(owning_edges[index])
            || row.term != SourcePrimaryTermId::new(index)
            || row.reference != SourcePrimaryTermReferenceId::new(index)
            || term.kind() != SourcePrimaryTermKind::VariableReference
            || term.role() != SourcePrimaryTermRole::Value
            || term.recovery() != SourcePrimaryTermRecovery::Normal
            || term.spelling() != binder.identifier_spelling()
            || term.parent().is_some()
            || !properly_contains(owner.source_range(), term.source_range())
            || reference.term() != row.term
            || reference.binding() != binder.binding()
            || reference.role() != SourcePrimaryTermReferenceRole::Variable
            || reference.lexical_scope() != context.lexical_scope.as_ref()
            || reference.use_ordinal() != use_ordinals[index]
        {
            return Err(SourceFormulaCompositionError::InvalidBoundUse { bound_use: id });
        }
        let lookup = BindingLookupSite::new(
            term.spelling(),
            term.context(),
            reference.lexical_scope().cloned(),
            reference.use_ordinal(),
        );
        if composite_formulas.binding_env().lookup(&lookup)
            != Ok(BindingLookupResult::Local(binder.binding()))
        {
            return Err(SourceFormulaCompositionError::InvalidBoundUse { bound_use: id });
        }
        if previous_range.is_some_and(|previous| previous.end > term.source_range().start) {
            return Err(SourceFormulaCompositionError::InvalidBoundUse { bound_use: id });
        }
        previous_range = Some(term.source_range());
    }
    Ok(())
}

fn properly_contains(parent: SourceRange, child: SourceRange) -> bool {
    parent.source_id == child.source_id
        && parent != child
        && parent.start <= child.start
        && child.end <= parent.end
}

fn atomic_edge_role_key(role: SourceFormulaAtomicEdgeRole) -> &'static str {
    match role {
        SourceFormulaAtomicEdgeRole::UniversalBody => "universal-body",
        SourceFormulaAtomicEdgeRole::UniversalRestriction => "universal-restriction",
        SourceFormulaAtomicEdgeRole::ConjunctionLeft => "conjunction-left",
        SourceFormulaAtomicEdgeRole::ConjunctionRight => "conjunction-right",
        SourceFormulaAtomicEdgeRole::DisjunctionLeft => "disjunction-left",
        SourceFormulaAtomicEdgeRole::DisjunctionRight => "disjunction-right",
    }
}

dense_id!(SourceFraenkelGeneratorBindingContextId);
dense_id!(SourceFraenkelGeneratorUsePositionId);
dense_id!(SourceNestedFraenkelBinderUseId);
dense_id!(SourceNestedFraenkelCaptureIdentityId);

/// One resolved nested Fraenkel mapper use and its distinct outer binder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceNestedFraenkelBinderUse {
    resolver_use_index: usize,
    resolver_binding: FraenkelGeneratorVariableBindingId,
    outer_binder: TypedNodeId,
    inner_mapper_use: TypedNodeId,
    source_ordinal: usize,
}

impl SourceNestedFraenkelBinderUse {
    #[must_use]
    pub const fn resolver_use_index(&self) -> usize {
        self.resolver_use_index
    }

    #[must_use]
    pub const fn resolver_binding(&self) -> FraenkelGeneratorVariableBindingId {
        self.resolver_binding
    }

    #[must_use]
    pub const fn outer_binder(&self) -> TypedNodeId {
        self.outer_binder
    }

    #[must_use]
    pub const fn inner_mapper_use(&self) -> TypedNodeId {
        self.inner_mapper_use
    }

    #[must_use]
    pub const fn source_ordinal(&self) -> usize {
        self.source_ordinal
    }
}

/// Dense nested Fraenkel binder-use rows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceNestedFraenkelBinderUseTable {
    rows: Vec<SourceNestedFraenkelBinderUse>,
}

impl SourceNestedFraenkelBinderUseTable {
    #[must_use]
    pub fn get(
        &self,
        id: SourceNestedFraenkelBinderUseId,
    ) -> Option<&SourceNestedFraenkelBinderUse> {
        self.rows.get(id.index())
    }

    pub fn iter(
        &self,
    ) -> impl Iterator<
        Item = (
            SourceNestedFraenkelBinderUseId,
            &SourceNestedFraenkelBinderUse,
        ),
    > {
        self.rows
            .iter()
            .enumerate()
            .map(|(index, row)| (SourceNestedFraenkelBinderUseId::new(index), row))
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.rows.len()
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

/// A rejected nested Fraenkel binder-use handoff.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceNestedFraenkelBinderUseError {
    EnvironmentMismatch,
    InvalidResolverDependency,
    InvalidTypedDependency,
    InvalidBinderUse {
        binder_use: SourceNestedFraenkelBinderUseId,
    },
}

impl fmt::Display for SourceNestedFraenkelBinderUseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EnvironmentMismatch => {
                formatter.write_str("nested Fraenkel binder-use environment mismatch")
            }
            Self::InvalidResolverDependency => {
                formatter.write_str("nested Fraenkel binder-use resolver dependency is invalid")
            }
            Self::InvalidTypedDependency => {
                formatter.write_str("nested Fraenkel binder-use typed dependency is invalid")
            }
            Self::InvalidBinderUse { binder_use } => write!(
                formatter,
                "nested Fraenkel binder use {} is invalid",
                binder_use.index()
            ),
        }
    }
}

impl Error for SourceNestedFraenkelBinderUseError {}

const NESTED_FRAENKEL_BINDER_USE_SNAPSHOT_VERSION: &str =
    "source-nested-fraenkel-binder-use-dependencies-v1";
const NESTED_FRAENKEL_BINDER_USE_SNAPSHOT_DOMAIN: &str = "source-nested-fraenkel-binder-use";

#[derive(Clone, PartialEq, Eq)]
struct SourceNestedFraenkelBinderUseDependencies {
    version: &'static str,
    domain: &'static str,
    resolver: FraenkelGeneratorVariableSourceCollection,
    typed_ast: TypedAst,
}

/// Immutable nested Fraenkel binder-use identity transport.
#[derive(Clone, PartialEq, Eq)]
pub struct SourceNestedFraenkelBinderUseHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    resolver_summary: String,
    binder_uses: SourceNestedFraenkelBinderUseTable,
    dependencies: SourceNestedFraenkelBinderUseDependencies,
}

impl SourceNestedFraenkelBinderUseHandoff {
    #[must_use]
    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    #[must_use]
    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    #[must_use]
    pub fn resolver_summary(&self) -> &str {
        &self.resolver_summary
    }

    #[must_use]
    pub const fn binder_uses(&self) -> &SourceNestedFraenkelBinderUseTable {
        &self.binder_uses
    }

    #[must_use]
    pub fn debug_text(&self) -> String {
        format!(
            "source-nested-fraenkel-binder-use-v1|module={}.{}|binder-uses={}",
            self.module_id.package().as_str(),
            self.module_id.path().as_str(),
            self.binder_uses.len(),
        )
    }

    fn validate(&self) -> Result<(), SourceNestedFraenkelBinderUseError> {
        let dependencies = &self.dependencies;
        if self.source_id != dependencies.resolver.source_id()
            || self.source_id != dependencies.typed_ast.source_id()
            || &self.module_id != dependencies.resolver.module()
            || &self.module_id != dependencies.typed_ast.module_id()
        {
            return Err(SourceNestedFraenkelBinderUseError::EnvironmentMismatch);
        }
        let profile = validate_nested_fraenkel_resolver(&dependencies.resolver)
            .ok_or(SourceNestedFraenkelBinderUseError::InvalidResolverDependency)?;
        if dependencies.version != NESTED_FRAENKEL_BINDER_USE_SNAPSHOT_VERSION
            || dependencies.domain != NESTED_FRAENKEL_BINDER_USE_SNAPSHOT_DOMAIN
            || self.resolver_summary != dependencies.resolver.debug_text()
        {
            return Err(SourceNestedFraenkelBinderUseError::InvalidResolverDependency);
        }
        let typed = validate_nested_fraenkel_typed(&dependencies.typed_ast, profile)
            .ok_or(SourceNestedFraenkelBinderUseError::InvalidTypedDependency)?;
        validate_nested_fraenkel_binder_use_rows(&self.binder_uses, profile, typed)
    }

    /// Reauthenticates the complete retained resolver and typed snapshots.
    ///
    /// This is intentionally crate-private: later structural transports may
    /// depend on the C4C3 identity relation, but must not inspect or expose its
    /// retained resolver or typed-AST snapshots.
    pub(crate) fn validate_complete(&self) -> Result<(), SourceNestedFraenkelBinderUseError> {
        self.validate()
    }
}

/// Builds the single default-deny nested Fraenkel binder-use identity handoff.
#[derive(Debug, Clone, Copy)]
pub struct SourceNestedFraenkelBinderUseProducer;

impl SourceNestedFraenkelBinderUseProducer {
    pub fn build(
        resolver: &FraenkelGeneratorVariableSourceCollection,
        typed_ast: &TypedAst,
    ) -> Result<SourceNestedFraenkelBinderUseHandoff, SourceNestedFraenkelBinderUseError> {
        if resolver.source_id() != typed_ast.source_id()
            || resolver.module() != typed_ast.module_id()
        {
            return Err(SourceNestedFraenkelBinderUseError::EnvironmentMismatch);
        }
        let dependencies = SourceNestedFraenkelBinderUseDependencies {
            version: NESTED_FRAENKEL_BINDER_USE_SNAPSHOT_VERSION,
            domain: NESTED_FRAENKEL_BINDER_USE_SNAPSHOT_DOMAIN,
            resolver: resolver.clone(),
            typed_ast: typed_ast.clone(),
        };
        let profile = validate_nested_fraenkel_resolver(&dependencies.resolver)
            .ok_or(SourceNestedFraenkelBinderUseError::InvalidResolverDependency)?;
        let typed = validate_nested_fraenkel_typed(&dependencies.typed_ast, profile)
            .ok_or(SourceNestedFraenkelBinderUseError::InvalidTypedDependency)?;
        let handoff = SourceNestedFraenkelBinderUseHandoff {
            source_id: dependencies.resolver.source_id(),
            module_id: dependencies.resolver.module().clone(),
            resolver_summary: dependencies.resolver.debug_text(),
            binder_uses: SourceNestedFraenkelBinderUseTable {
                rows: vec![SourceNestedFraenkelBinderUse {
                    resolver_use_index: 0,
                    resolver_binding: profile.outer_binding,
                    outer_binder: typed.outer_binder,
                    inner_mapper_use: typed.inner_mapper_use,
                    source_ordinal: 0,
                }],
            },
            dependencies,
        };
        handoff.validate()?;
        Ok(handoff)
    }
}

/// One exact C4C4 mapper-to-resolved-binding identity association.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceNestedFraenkelCaptureIdentity {
    owner_context: BindingContextId,
    owner_range: SourceRange,
    mapper_term: SourcePrimaryTermId,
    mapper_reference: SourcePrimaryTermReferenceId,
    projected_binding: BindingId,
    resolver_use_index: usize,
    resolver_binding: FraenkelGeneratorVariableBindingId,
    source_ordinal: usize,
}

impl SourceNestedFraenkelCaptureIdentity {
    #[must_use]
    pub const fn owner_context(&self) -> BindingContextId {
        self.owner_context
    }

    #[must_use]
    pub const fn owner_range(&self) -> SourceRange {
        self.owner_range
    }

    #[must_use]
    pub const fn mapper_term(&self) -> SourcePrimaryTermId {
        self.mapper_term
    }

    #[must_use]
    pub const fn mapper_reference(&self) -> SourcePrimaryTermReferenceId {
        self.mapper_reference
    }

    #[must_use]
    pub const fn projected_binding(&self) -> BindingId {
        self.projected_binding
    }

    #[must_use]
    pub const fn resolver_use_index(&self) -> usize {
        self.resolver_use_index
    }

    #[must_use]
    pub const fn resolver_binding(&self) -> FraenkelGeneratorVariableBindingId {
        self.resolver_binding
    }

    #[must_use]
    pub const fn source_ordinal(&self) -> usize {
        self.source_ordinal
    }
}

/// Dense source-ordered C4C4 capture-identity associations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceNestedFraenkelCaptureIdentityTable {
    rows: Vec<SourceNestedFraenkelCaptureIdentity>,
}

impl SourceNestedFraenkelCaptureIdentityTable {
    #[must_use]
    pub fn get(
        &self,
        id: SourceNestedFraenkelCaptureIdentityId,
    ) -> Option<&SourceNestedFraenkelCaptureIdentity> {
        self.rows.get(id.index())
    }

    pub fn iter(
        &self,
    ) -> impl Iterator<
        Item = (
            SourceNestedFraenkelCaptureIdentityId,
            &SourceNestedFraenkelCaptureIdentity,
        ),
    > {
        self.rows
            .iter()
            .enumerate()
            .map(|(index, row)| (SourceNestedFraenkelCaptureIdentityId::new(index), row))
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.rows.len()
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

/// A rejected nested Fraenkel capture-identity receipt.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceNestedFraenkelCaptureIdentityError {
    InvalidDependency,
    InvalidCaptureIdentity {
        capture_identity: SourceNestedFraenkelCaptureIdentityId,
    },
}

impl fmt::Display for SourceNestedFraenkelCaptureIdentityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDependency => {
                formatter.write_str("nested Fraenkel capture-identity dependency is invalid")
            }
            Self::InvalidCaptureIdentity { capture_identity } => write!(
                formatter,
                "nested Fraenkel capture identity {} is invalid",
                capture_identity.index()
            ),
        }
    }
}

impl Error for SourceNestedFraenkelCaptureIdentityError {}

/// Immutable first receipt of the exact nested Fraenkel capture identity.
#[derive(Clone, PartialEq, Eq)]
pub struct SourceNestedFraenkelCaptureIdentityHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    dependency: SourceNestedFraenkelMapperPrimaryHandoff,
    dependency_fingerprint: String,
    identities: SourceNestedFraenkelCaptureIdentityTable,
}

impl SourceNestedFraenkelCaptureIdentityHandoff {
    #[must_use]
    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    #[must_use]
    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    #[must_use]
    pub const fn dependency(&self) -> &SourceNestedFraenkelMapperPrimaryHandoff {
        &self.dependency
    }

    #[must_use]
    pub fn dependency_fingerprint(&self) -> &str {
        &self.dependency_fingerprint
    }

    #[must_use]
    pub const fn identities(&self) -> &SourceNestedFraenkelCaptureIdentityTable {
        &self.identities
    }

    #[must_use]
    pub fn debug_text(&self) -> String {
        format!(
            "source-nested-fraenkel-capture-identity-v1|module={}.{}|identities=1|dependency-fingerprint={:?}",
            self.module_id.package().as_str(),
            self.module_id.path().as_str(),
            self.dependency_fingerprint,
        )
    }

    fn validate(&self) -> Result<(), SourceNestedFraenkelCaptureIdentityError> {
        if self.source_id != self.dependency.source_id()
            || &self.module_id != self.dependency.module_id()
            || self.dependency_fingerprint != self.dependency.debug_text()
            || self.dependency.validate_complete().is_err()
        {
            return Err(SourceNestedFraenkelCaptureIdentityError::InvalidDependency);
        }

        let identity_id = SourceNestedFraenkelCaptureIdentityId::new(0);
        if self.identities.len() != 1
            || self
                .identities
                .get(SourceNestedFraenkelCaptureIdentityId::new(1))
                .is_some()
        {
            return Err(
                SourceNestedFraenkelCaptureIdentityError::InvalidCaptureIdentity {
                    capture_identity: identity_id,
                },
            );
        }
        let identity = self.identities.get(identity_id).ok_or(
            SourceNestedFraenkelCaptureIdentityError::InvalidCaptureIdentity {
                capture_identity: identity_id,
            },
        )?;
        if !validate_nested_fraenkel_capture_identity(&self.dependency, identity) {
            return Err(
                SourceNestedFraenkelCaptureIdentityError::InvalidCaptureIdentity {
                    capture_identity: identity_id,
                },
            );
        }
        Ok(())
    }

    /// Reauthenticates the complete retained C4C4 dependency and receipt.
    pub(crate) fn validate_complete(&self) -> Result<(), SourceNestedFraenkelCaptureIdentityError> {
        self.validate()
    }

    pub(crate) fn validate_typed_owner(&self, typed_ast: &TypedAst) -> bool {
        self.validate_complete().is_ok()
            && typed_ast.matches_source_nested_fraenkel_capture_identity_snapshot(
                &self.dependency.dependency().dependencies.typed_ast,
            )
    }
}

/// Builds only the exact nested Fraenkel capture-identity receipt.
#[derive(Debug, Clone, Copy)]
pub struct SourceNestedFraenkelCaptureIdentityProducer;

impl SourceNestedFraenkelCaptureIdentityProducer {
    pub fn build(
        dependency: SourceNestedFraenkelMapperPrimaryHandoff,
    ) -> Result<SourceNestedFraenkelCaptureIdentityHandoff, SourceNestedFraenkelCaptureIdentityError>
    {
        dependency
            .validate_complete()
            .map_err(|_| SourceNestedFraenkelCaptureIdentityError::InvalidDependency)?;
        let source_id = dependency.source_id();
        let module_id = dependency.module_id().clone();
        let handoff = SourceNestedFraenkelCaptureIdentityHandoff {
            source_id,
            module_id,
            dependency_fingerprint: dependency.debug_text(),
            dependency,
            identities: SourceNestedFraenkelCaptureIdentityTable {
                rows: vec![SourceNestedFraenkelCaptureIdentity {
                    owner_context: BindingContextId::new(2),
                    owner_range: SourceRange {
                        source_id,
                        start: 92,
                        end: 123,
                    },
                    mapper_term: SourcePrimaryTermId::new(0),
                    mapper_reference: SourcePrimaryTermReferenceId::new(0),
                    projected_binding: BindingId::new(0),
                    resolver_use_index: 0,
                    resolver_binding: FraenkelGeneratorVariableBindingId::new(1),
                    source_ordinal: 0,
                }],
            },
        };
        handoff.validate_complete()?;
        Ok(handoff)
    }
}

fn validate_nested_fraenkel_capture_identity(
    dependency: &SourceNestedFraenkelMapperPrimaryHandoff,
    identity: &SourceNestedFraenkelCaptureIdentity,
) -> bool {
    let owner_context = BindingContextId::new(2);
    let projected_binding = BindingId::new(0);
    let mapper_term = SourcePrimaryTermId::new(0);
    let mapper_reference = SourcePrimaryTermReferenceId::new(0);
    if identity.owner_context != owner_context
        || identity.mapper_term != mapper_term
        || identity.mapper_reference != mapper_reference
        || identity.projected_binding != projected_binding
        || identity.resolver_use_index != 0
        || identity.resolver_binding != FraenkelGeneratorVariableBindingId::new(1)
        || identity.source_ordinal != 0
        || !exact_nested_resolver_range(identity.owner_range, dependency.source_id(), 92, 123)
    {
        return false;
    }

    let Some(owner) = dependency.binding_env().contexts().get(owner_context) else {
        return false;
    };
    if owner.id != owner_context
        || !matches!(
            &owner.owner,
            BindingContextOwner::SourceComprehension { source_range }
                if *source_range == identity.owner_range
        )
        || owner.parent != Some(BindingContextId::new(1))
        || owner.layer != BindingContextLayer::Expression
        || owner.lexical_scope.is_some()
        || !owner.bindings.is_empty()
        || owner.visible_bindings.as_slice() != [projected_binding]
        || owner.recovery != BindingContextRecovery::Normal
    {
        return false;
    }

    let Some(term) = dependency.source_term().terms().get(mapper_term) else {
        return false;
    };
    let Some(reference) = dependency.source_term().references().get(mapper_reference) else {
        return false;
    };
    let Some(binding) = dependency.binding_env().bindings().get(projected_binding) else {
        return false;
    };
    let binder_use_id = SourceNestedFraenkelBinderUseId::new(0);
    let Some(binder_use) = dependency.dependency().binder_uses().get(binder_use_id) else {
        return false;
    };

    term.context() == owner_context
        && exact_nested_resolver_range(term.source_range(), dependency.source_id(), 94, 95)
        && term.source_ordinal() == 0
        && term.kind() == SourcePrimaryTermKind::VariableReference
        && term.role() == SourcePrimaryTermRole::Value
        && term.recovery() == SourcePrimaryTermRecovery::Normal
        && term.parent().is_none()
        && reference.term() == mapper_term
        && reference.binding() == projected_binding
        && reference.role() == SourcePrimaryTermReferenceRole::Variable
        && reference.lexical_scope().is_none()
        && reference.use_ordinal() == 1
        && binding.id == projected_binding
        && matches!(
            &binding.identity,
            BinderIdentity::SourceBound { context, ordinal }
                if *context == BindingContextId::new(1) && *ordinal == 0
        )
        && binding.owner_context == BindingContextId::new(1)
        && binding.captured.identities().is_empty()
        && binder_use.resolver_use_index() == identity.resolver_use_index
        && binder_use.resolver_binding() == identity.resolver_binding
        && binder_use.source_ordinal() == identity.source_ordinal
        && dependency
            .dependency()
            .binder_uses()
            .get(SourceNestedFraenkelBinderUseId::new(1))
            .is_none()
}

#[derive(Clone, Copy)]
struct NestedFraenkelResolverProfile {
    definition_block: ResolvedNodeId,
    functor_definition: ResolvedNodeId,
    inner_comprehension: ResolvedNodeId,
    inner_segment: ResolvedNodeId,
    inner_binder: ResolvedNodeId,
    outer_comprehension: ResolvedNodeId,
    outer_segment: ResolvedNodeId,
    outer_binder: ResolvedNodeId,
    mapper_owner: ResolvedNodeId,
    mapper_reference: ResolvedNodeId,
    mapper_identifier: ResolvedNodeId,
    outer_binding: FraenkelGeneratorVariableBindingId,
}

fn validate_nested_fraenkel_resolver(
    resolver: &FraenkelGeneratorVariableSourceCollection,
) -> Option<NestedFraenkelResolverProfile> {
    let bindings = resolver.bindings().iter().collect::<Vec<_>>();
    let [(inner_id, inner), (outer_id, outer)] = bindings.as_slice() else {
        return None;
    };
    let uses = resolver.uses().iter().collect::<Vec<_>>();
    let [mapper] = uses.as_slice() else {
        return None;
    };
    if inner_id.index() != 0
        || outer_id.index() != 1
        || resolver.bindings().get(*inner_id) != Some(*inner)
        || resolver.bindings().get(*outer_id) != Some(*outer)
        || resolver
            .bindings()
            .get(FraenkelGeneratorVariableBindingId::new(2))
            .is_some()
        || resolver.uses().get(1).is_some()
        || inner.spelling() != "y"
        || outer.spelling() != "x"
        || inner.source_ordinal() != 0
        || outer.source_ordinal() != 1
        || !exact_nested_resolver_range(inner.segment_range(), resolver.source_id(), 102, 121)
        || !exact_nested_resolver_range(inner.binder_range(), resolver.source_id(), 102, 103)
        || !exact_nested_resolver_range(outer.segment_range(), resolver.source_id(), 136, 155)
        || !exact_nested_resolver_range(outer.binder_range(), resolver.source_id(), 136, 137)
        || inner.definition_block() != outer.definition_block()
        || inner.functor_definition() != outer.functor_definition()
        || inner.comprehension() == outer.comprehension()
        || inner.binder() == outer.binder()
        || mapper.definition_block() != inner.definition_block()
        || mapper.functor_definition() != inner.functor_definition()
        || mapper.comprehension() != inner.comprehension()
        || mapper.binding() != *outer_id
        || mapper.role() != FraenkelGeneratorVariableUseRole::Mapper
        || mapper.source_ordinal() != 0
        || mapper.role_source_ordinal() != 0
        || !exact_nested_resolver_range(mapper.identifier_range(), resolver.source_id(), 94, 95)
    {
        return None;
    }
    Some(NestedFraenkelResolverProfile {
        definition_block: inner.definition_block(),
        functor_definition: inner.functor_definition(),
        inner_comprehension: inner.comprehension(),
        inner_segment: inner.segment(),
        inner_binder: inner.binder(),
        outer_comprehension: outer.comprehension(),
        outer_segment: outer.segment(),
        outer_binder: outer.binder(),
        mapper_owner: mapper.role_owner(),
        mapper_reference: mapper.term_reference(),
        mapper_identifier: mapper.identifier(),
        outer_binding: *outer_id,
    })
}

fn exact_nested_resolver_range(
    range: SourceRange,
    source_id: SourceId,
    start: usize,
    end: usize,
) -> bool {
    range
        == SourceRange {
            source_id,
            start,
            end,
        }
}

#[derive(Clone, Copy)]
struct NestedFraenkelTypedProfile {
    outer_binder: TypedNodeId,
    inner_mapper_use: TypedNodeId,
}

fn validate_nested_fraenkel_typed(
    typed_ast: &TypedAst,
    profile: NestedFraenkelResolverProfile,
) -> Option<NestedFraenkelTypedProfile> {
    if typed_ast
        .source_nested_fraenkel_capture_identity()
        .is_some()
        || typed_ast.resolved_root().is_some()
        || !typed_ast.contexts().is_empty()
        || !typed_ast.types().is_empty()
        || !typed_ast.facts().is_empty()
        || !typed_ast.coercions().is_empty()
        || !typed_ast.initial_obligations().is_empty()
        || !typed_ast.diagnostics().is_empty()
        || !has_complete_unique_normal_typed_projection(typed_ast)
    {
        return None;
    }
    let definition =
        exact_nested_typed_id(typed_ast, profile.definition_block, "DefinitionBlockItem")?;
    let functor =
        exact_nested_typed_id(typed_ast, profile.functor_definition, "FunctorDefinition")?;
    let inner_comprehension = exact_nested_typed_node(
        typed_ast,
        profile.inner_comprehension,
        "SetComprehension",
        92,
        123,
    )?;
    let inner_segment = exact_nested_typed_node(
        typed_ast,
        profile.inner_segment,
        "ComprehensionVariableSegment",
        102,
        121,
    )?;
    let inner_binder =
        exact_nested_typed_node(typed_ast, profile.inner_binder, "Identifier", 102, 103)?;
    let outer_comprehension = exact_nested_typed_node(
        typed_ast,
        profile.outer_comprehension,
        "SetComprehension",
        90,
        157,
    )?;
    let outer_segment = exact_nested_typed_node(
        typed_ast,
        profile.outer_segment,
        "ComprehensionVariableSegment",
        136,
        155,
    )?;
    let outer_binder =
        exact_nested_typed_node(typed_ast, profile.outer_binder, "Identifier", 136, 137)?;
    let mapper_owner =
        exact_nested_typed_node(typed_ast, profile.mapper_owner, "TermExpression", 94, 95)?;
    let mapper_reference =
        exact_nested_typed_node(typed_ast, profile.mapper_reference, "TermReference", 94, 95)?;
    let mapper_identifier =
        exact_nested_typed_node(typed_ast, profile.mapper_identifier, "Identifier", 94, 95)?;
    if !exact_nested_typed_root(typed_ast, definition) {
        return None;
    }
    let typed = [
        definition,
        functor,
        inner_comprehension,
        inner_segment,
        inner_binder,
        outer_comprehension,
        outer_segment,
        outer_binder,
        mapper_owner,
        mapper_reference,
        mapper_identifier,
    ];
    let outer_mapper = exact_typed_child_containing(
        typed_ast,
        outer_comprehension,
        "TermExpression",
        inner_comprehension,
    )?;
    let definiens = exact_typed_child(typed_ast, functor, "TermDefiniens")?;
    let definiens_expression =
        exact_typed_child_containing(typed_ast, definiens, "TermExpression", outer_comprehension)?;
    let inner_type = exact_nested_typed_type(typed_ast, inner_segment, 107, 121)?;
    let outer_type = exact_nested_typed_type(typed_ast, outer_segment, 141, 155)?;
    let definition_children = typed_children(typed_ast, definition)?;
    let functor_children = typed_children(typed_ast, functor)?;
    let definiens_children = typed_children(typed_ast, definiens)?;
    let definiens_expression_children = typed_children(typed_ast, definiens_expression)?;
    let outer_children = typed_children(typed_ast, outer_comprehension)?;
    let outer_mapper_children = typed_children(typed_ast, outer_mapper)?;
    let inner_children = typed_children(typed_ast, inner_comprehension)?;
    let inner_segment_children = typed_children(typed_ast, inner_segment)?;
    let outer_segment_children = typed_children(typed_ast, outer_segment)?;
    let mapper_owner_children = typed_children(typed_ast, mapper_owner)?;
    let mapper_reference_children = typed_children(typed_ast, mapper_reference)?;
    if typed
        .iter()
        .enumerate()
        .any(|(index, id)| typed[..index].contains(id))
        || !has_typed_edge(typed_ast, definition, functor)
        || !range_contains(
            typed_range(typed_ast, definition)?,
            typed_range(typed_ast, functor)?,
        )
        || !range_contains(
            typed_range(typed_ast, functor)?,
            typed_range(typed_ast, definiens)?,
        )
        || !range_contains(
            typed_range(typed_ast, definiens)?,
            typed_range(typed_ast, definiens_expression)?,
        )
        || !range_contains(
            typed_range(typed_ast, definiens_expression)?,
            typed_range(typed_ast, outer_comprehension)?,
        )
        || !has_typed_edge(typed_ast, functor, definiens)
        || !has_typed_edge(typed_ast, definiens, definiens_expression)
        || !has_typed_edge(typed_ast, definiens_expression, outer_comprehension)
        || typed_range(typed_ast, outer_mapper)? != typed_range(typed_ast, inner_comprehension)?
        || !has_typed_edge(typed_ast, outer_comprehension, outer_mapper)
        || !has_typed_edge(typed_ast, outer_mapper, inner_comprehension)
        || !has_typed_edge(typed_ast, inner_comprehension, inner_segment)
        || !has_typed_edge(typed_ast, inner_segment, inner_binder)
        || !has_typed_edge(typed_ast, inner_segment, inner_type)
        || !has_typed_edge(typed_ast, outer_comprehension, outer_segment)
        || !has_typed_edge(typed_ast, outer_segment, outer_binder)
        || !has_typed_edge(typed_ast, outer_segment, outer_type)
        || !has_typed_edge(typed_ast, inner_comprehension, mapper_owner)
        || !has_typed_edge(typed_ast, mapper_owner, mapper_reference)
        || !has_typed_edge(typed_ast, mapper_reference, mapper_identifier)
        || !exact_nested_definition_children(typed_ast, definition_children, functor)
        || !exact_nested_functor_children(typed_ast, functor_children, definiens)
        || definiens_children != [definiens_expression]
        || definiens_expression_children != [outer_comprehension]
        || outer_children.len() != 5
        || !exact_nested_token(typed_ast, outer_children[0], "ReservedSymbol", "{", 90, 91)
        || outer_children[1] != outer_mapper
        || !exact_nested_token(
            typed_ast,
            outer_children[2],
            "ReservedWord",
            "where",
            130,
            135,
        )
        || outer_children[3] != outer_segment
        || !exact_nested_token(
            typed_ast,
            outer_children[4],
            "ReservedSymbol",
            "}",
            156,
            157,
        )
        || outer_mapper_children != [inner_comprehension]
        || inner_children.len() != 5
        || !exact_nested_token(typed_ast, inner_children[0], "ReservedSymbol", "{", 92, 93)
        || inner_children[1] != mapper_owner
        || !exact_nested_token(
            typed_ast,
            inner_children[2],
            "ReservedWord",
            "where",
            96,
            101,
        )
        || inner_children[3] != inner_segment
        || !exact_nested_token(
            typed_ast,
            inner_children[4],
            "ReservedSymbol",
            "}",
            122,
            123,
        )
        || inner_segment_children.len() != 3
        || inner_segment_children != [inner_binder, inner_segment_children[1], inner_type]
        || !exact_nested_token(
            typed_ast,
            inner_segment_children[1],
            "ReservedWord",
            "is",
            104,
            106,
        )
        || outer_segment_children.len() != 3
        || outer_segment_children != [outer_binder, outer_segment_children[1], outer_type]
        || !exact_nested_token(
            typed_ast,
            outer_segment_children[1],
            "ReservedWord",
            "is",
            138,
            140,
        )
        || mapper_owner_children != [mapper_reference]
        || mapper_reference_children != [mapper_identifier]
    {
        return None;
    }
    Some(NestedFraenkelTypedProfile {
        outer_binder,
        inner_mapper_use: mapper_identifier,
    })
}

fn exact_nested_definition_children(
    typed_ast: &TypedAst,
    children: &[TypedNodeId],
    functor: TypedNodeId,
) -> bool {
    match children {
        [definition, found] => {
            *found == functor
                && exact_nested_token(typed_ast, *definition, "ReservedWord", "definition", 39, 49)
        }
        [definition, found, end, semicolon] => {
            *found == functor
                && exact_nested_token(typed_ast, *definition, "ReservedWord", "definition", 40, 50)
                && exact_nested_token(typed_ast, *end, "ReservedWord", "end", 159, 162)
                && exact_nested_token(typed_ast, *semicolon, "ReservedSymbol", ";", 162, 163)
        }
        _ => false,
    }
}

fn exact_nested_functor_children(
    typed_ast: &TypedAst,
    children: &[TypedNodeId],
    definiens: TypedNodeId,
) -> bool {
    match children {
        [func, found] => {
            *found == definiens
                && exact_nested_token(typed_ast, *func, "ReservedWord", "func", 52, 56)
        }
        [func, pattern, arrow, result_type, equals, found, semicolon] => {
            *found == definiens
                && exact_nested_imported_functor_scaffold(typed_ast, *pattern, *result_type)
                && exact_nested_token(typed_ast, *func, "ReservedWord", "func", 53, 57)
                && exact_nested_token(typed_ast, *arrow, "ReservedSymbol", "->", 72, 74)
                && typed_ast
                    .nodes()
                    .node(*result_type)
                    .is_some_and(|node| node.kind.as_str() == "TypeExpression")
                && exact_nested_token(typed_ast, *equals, "ReservedWord", "equals", 79, 85)
                && exact_nested_token(typed_ast, *semicolon, "ReservedSymbol", ";", 157, 158)
        }
        _ => false,
    }
}

fn exact_nested_imported_functor_scaffold(
    typed_ast: &TypedAst,
    pattern: TypedNodeId,
    result_type: TypedNodeId,
) -> bool {
    let Some([pattern_identifier]) = typed_children(typed_ast, pattern) else {
        return false;
    };
    let Some([head]) = typed_children(typed_ast, result_type) else {
        return false;
    };
    let Some([set]) = typed_children(typed_ast, *head) else {
        return false;
    };
    typed_ast
        .nodes()
        .node(pattern)
        .is_some_and(|node| node.kind.as_str() == "FunctorPattern")
        && typed_ast
            .nodes()
            .node(*pattern_identifier)
            .is_some_and(|node| node.kind.as_str() == "Identifier")
        && typed_range(typed_ast, *pattern_identifier)
            == Some(SourceRange {
                source_id: typed_ast.source_id(),
                start: 58,
                end: 71,
            })
        && typed_ast
            .nodes()
            .node(result_type)
            .is_some_and(|node| node.kind.as_str() == "TypeExpression")
        && typed_ast
            .nodes()
            .node(*head)
            .is_some_and(|node| node.kind.as_str() == "TypeHead")
        && exact_nested_token(typed_ast, *set, "ReservedWord", "set", 75, 78)
}

fn typed_children(typed_ast: &TypedAst, parent: TypedNodeId) -> Option<&[TypedNodeId]> {
    Some(typed_ast.nodes().node(parent)?.children.as_slice())
}

fn exact_nested_typed_root(typed_ast: &TypedAst, definition: TypedNodeId) -> bool {
    let Some(root) = typed_ast.nodes().root() else {
        return false;
    };
    let Some(root_node) = typed_ast.nodes().node(root) else {
        return false;
    };
    if root_node.kind.as_str() != "Root"
        || root_node.recovery != NodeRecoveryState::Normal
        || !all_typed_nodes_reachable_from_root(typed_ast, root)
    {
        return false;
    }
    match typed_range(typed_ast, root) {
        Some(SourceRange {
            source_id,
            start: 0,
            end: 164,
        }) if source_id == typed_ast.source_id() => root_node.children.as_slice() == [definition],
        Some(SourceRange {
            source_id,
            start: 0,
            end: 163,
        }) if source_id == typed_ast.source_id() => {
            let Some((compilation_unit, raw_tokens)) = root_node.children.split_last() else {
                return false;
            };
            if raw_tokens.len() != 31
                || raw_tokens.iter().any(|child| {
                    typed_ast
                        .nodes()
                        .node(*child)
                        .is_none_or(|node| !node.children.is_empty())
                })
            {
                return false;
            }
            let Some([item_list]) = typed_children(typed_ast, *compilation_unit) else {
                return false;
            };
            let Some([import_item, found_definition]) = typed_children(typed_ast, *item_list)
            else {
                return false;
            };
            *found_definition == definition
                && typed_ast
                    .nodes()
                    .node(*compilation_unit)
                    .is_some_and(|node| {
                        node.kind.as_str() == "CompilationUnit"
                            && typed_range(typed_ast, *compilation_unit)
                                == Some(SourceRange {
                                    source_id,
                                    start: 0,
                                    end: 163,
                                })
                    })
                && typed_ast.nodes().node(*item_list).is_some_and(|node| {
                    node.kind.as_str() == "ItemList"
                        && typed_range(typed_ast, *item_list)
                            == Some(SourceRange {
                                source_id,
                                start: 0,
                                end: 163,
                            })
                })
                && typed_ast
                    .nodes()
                    .node(*import_item)
                    .is_some_and(|node| node.kind.as_str() == "ImportItem")
        }
        _ => false,
    }
}

fn all_typed_nodes_reachable_from_root(typed_ast: &TypedAst, root: TypedNodeId) -> bool {
    let mut reachable = std::collections::BTreeSet::new();
    let mut pending = vec![root];
    while let Some(id) = pending.pop() {
        if !reachable.insert(id) {
            continue;
        }
        let Some(node) = typed_ast.nodes().node(id) else {
            return false;
        };
        pending.extend(node.children.iter().copied());
    }
    reachable.len() == typed_ast.nodes().len()
}

fn exact_nested_token(
    typed_ast: &TypedAst,
    id: TypedNodeId,
    token_kind: &str,
    text: &str,
    start: usize,
    end: usize,
) -> bool {
    typed_ast.nodes().node(id).is_some_and(|node| {
        node.kind.as_str()
            == format!(r#"Token(SurfaceToken {{ kind: {token_kind}, text: "{text}" }})"#)
            && node.recovery == NodeRecoveryState::Normal
            && typed_range(typed_ast, id)
                == Some(SourceRange {
                    source_id: typed_ast.source_id(),
                    start,
                    end,
                })
    })
}

fn has_complete_unique_normal_typed_projection(typed_ast: &TypedAst) -> bool {
    let mut resolved = std::collections::BTreeSet::new();
    !typed_ast.nodes().is_empty()
        && typed_ast.nodes().iter().all(|(_, node)| {
            node.recovery == NodeRecoveryState::Normal
                && node.resolved_node.is_some_and(|id| resolved.insert(id))
        })
}

fn exact_nested_typed_type(
    typed_ast: &TypedAst,
    segment: TypedNodeId,
    start: usize,
    end: usize,
) -> Option<TypedNodeId> {
    let type_expression = exact_typed_child(typed_ast, segment, "TypeExpression")?;
    let type_head = exact_typed_child(typed_ast, type_expression, "TypeHead")?;
    let element_symbol = exact_typed_child(typed_ast, type_head, "QualifiedSymbol")?;
    let arguments = exact_typed_child(typed_ast, type_head, "TypeArguments")?;
    let nat_term = exact_typed_child(typed_ast, arguments, "TermExpression")?;
    let nat_reference = exact_typed_child(typed_ast, nat_term, "TermReference")?;
    let nat_symbol = exact_typed_child(typed_ast, nat_reference, "QualifiedSymbol")?;
    let element_path = exact_typed_child(typed_ast, element_symbol, "PathSegment")?;
    let nat_path = exact_typed_child(typed_ast, nat_symbol, "PathSegment")?;
    let element_token = *typed_children(typed_ast, element_path)?.first()?;
    let of_token = *typed_children(typed_ast, arguments)?.first()?;
    let nat_token = *typed_children(typed_ast, nat_path)?.first()?;
    let checks = [
        (type_expression, "TypeExpression", start, end),
        (type_head, "TypeHead", start, end),
        (element_symbol, "QualifiedSymbol", start, start + 7),
        (element_path, "PathSegment", start, start + 7),
        (arguments, "TypeArguments", start + 8, end),
        (nat_term, "TermExpression", start + 11, end),
        (nat_reference, "TermReference", start + 11, end),
        (nat_symbol, "QualifiedSymbol", start + 11, end),
        (nat_path, "PathSegment", start + 11, end),
    ];
    (checks
        .into_iter()
        .all(|(id, kind, range_start, range_end)| {
            typed_ast.nodes().node(id).is_some_and(|node| {
                node.kind.as_str() == kind
                    && node.recovery == NodeRecoveryState::Normal
                    && typed_range(typed_ast, id)
                        == Some(SourceRange {
                            source_id: typed_ast.source_id(),
                            start: range_start,
                            end: range_end,
                        })
            })
        })
        && typed_children(typed_ast, type_expression)? == [type_head]
        && typed_children(typed_ast, type_head)? == [element_symbol, arguments]
        && typed_children(typed_ast, element_symbol)? == [element_path]
        && typed_children(typed_ast, element_path)? == [element_token]
        && exact_nested_token(
            typed_ast,
            element_token,
            "UserSymbol",
            "Element",
            start,
            start + 7,
        )
        && typed_children(typed_ast, arguments)? == [of_token, nat_term]
        && exact_nested_token(
            typed_ast,
            of_token,
            "ReservedWord",
            "of",
            start + 8,
            start + 10,
        )
        && typed_children(typed_ast, nat_term)? == [nat_reference]
        && typed_children(typed_ast, nat_reference)? == [nat_symbol]
        && typed_children(typed_ast, nat_symbol)? == [nat_path]
        && typed_children(typed_ast, nat_path)? == [nat_token]
        && exact_nested_token(typed_ast, nat_token, "UserSymbol", "NAT", start + 11, end))
    .then_some(type_expression)
}

fn exact_nested_typed_node(
    typed_ast: &TypedAst,
    resolved: ResolvedNodeId,
    kind: &str,
    start: usize,
    end: usize,
) -> Option<TypedNodeId> {
    let id = typed_for_resolved(typed_ast, resolved)?;
    let node = exact_stored_typed_node(typed_ast, id, kind)?;
    let range = typed_range(typed_ast, id)?;
    (node.resolved_node == Some(resolved) && range.start == start && range.end == end).then_some(id)
}

fn exact_nested_typed_id(
    typed_ast: &TypedAst,
    resolved: ResolvedNodeId,
    kind: &str,
) -> Option<TypedNodeId> {
    let id = typed_for_resolved(typed_ast, resolved)?;
    (exact_stored_typed_node(typed_ast, id, kind)?.resolved_node == Some(resolved)).then_some(id)
}

fn validate_nested_fraenkel_binder_use_rows(
    rows: &SourceNestedFraenkelBinderUseTable,
    profile: NestedFraenkelResolverProfile,
    typed: NestedFraenkelTypedProfile,
) -> Result<(), SourceNestedFraenkelBinderUseError> {
    if rows.len() != 1 {
        return Err(SourceNestedFraenkelBinderUseError::InvalidBinderUse {
            binder_use: SourceNestedFraenkelBinderUseId::new(0),
        });
    }
    let id = SourceNestedFraenkelBinderUseId::new(0);
    let Some(row) = rows.get(id) else {
        return Err(SourceNestedFraenkelBinderUseError::InvalidBinderUse { binder_use: id });
    };
    if rows.get(SourceNestedFraenkelBinderUseId::new(1)).is_some()
        || row.resolver_use_index() != 0
        || row.resolver_binding() != profile.outer_binding
        || row.outer_binder() != typed.outer_binder
        || row.inner_mapper_use() != typed.inner_mapper_use
        || row.source_ordinal() != 0
    {
        return Err(SourceNestedFraenkelBinderUseError::InvalidBinderUse { binder_use: id });
    }
    Ok(())
}

/// One checked Fraenkel generator binding context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFraenkelGeneratorBindingContext {
    composition: SourceTemplateFraenkelStructuralCompositionId,
    resolver_binding: FraenkelGeneratorVariableBindingId,
    context: BindingContextId,
    binding: BindingId,
    source_ordinal: usize,
}

impl SourceFraenkelGeneratorBindingContext {
    #[must_use]
    pub const fn composition(&self) -> SourceTemplateFraenkelStructuralCompositionId {
        self.composition
    }

    #[must_use]
    pub const fn resolver_binding(&self) -> FraenkelGeneratorVariableBindingId {
        self.resolver_binding
    }

    #[must_use]
    pub const fn context(&self) -> BindingContextId {
        self.context
    }

    #[must_use]
    pub const fn binding(&self) -> BindingId {
        self.binding
    }

    #[must_use]
    pub const fn source_ordinal(&self) -> usize {
        self.source_ordinal
    }
}

/// Dense checked Fraenkel generator binding contexts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFraenkelGeneratorBindingContextTable {
    rows: Vec<SourceFraenkelGeneratorBindingContext>,
}

impl SourceFraenkelGeneratorBindingContextTable {
    #[must_use]
    pub fn get(
        &self,
        id: SourceFraenkelGeneratorBindingContextId,
    ) -> Option<&SourceFraenkelGeneratorBindingContext> {
        self.rows.get(id.index())
    }

    pub fn iter(
        &self,
    ) -> impl Iterator<
        Item = (
            SourceFraenkelGeneratorBindingContextId,
            &SourceFraenkelGeneratorBindingContext,
        ),
    > {
        self.rows
            .iter()
            .enumerate()
            .map(|(index, row)| (SourceFraenkelGeneratorBindingContextId::new(index), row))
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.rows.len()
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

/// One normalized lookup position for a checked Fraenkel generator use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFraenkelGeneratorUsePosition {
    binding_context: SourceFraenkelGeneratorBindingContextId,
    resolver_use_index: usize,
    source_ordinal: usize,
    lookup_ordinal: usize,
}

impl SourceFraenkelGeneratorUsePosition {
    #[must_use]
    pub const fn binding_context(&self) -> SourceFraenkelGeneratorBindingContextId {
        self.binding_context
    }

    #[must_use]
    pub const fn resolver_use_index(&self) -> usize {
        self.resolver_use_index
    }

    #[must_use]
    pub const fn source_ordinal(&self) -> usize {
        self.source_ordinal
    }

    #[must_use]
    pub const fn lookup_ordinal(&self) -> usize {
        self.lookup_ordinal
    }
}

/// Dense normalized lookup positions for a checked Fraenkel generator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFraenkelGeneratorUsePositionTable {
    rows: Vec<SourceFraenkelGeneratorUsePosition>,
}

impl SourceFraenkelGeneratorUsePositionTable {
    #[must_use]
    pub fn get(
        &self,
        id: SourceFraenkelGeneratorUsePositionId,
    ) -> Option<&SourceFraenkelGeneratorUsePosition> {
        self.rows.get(id.index())
    }

    pub fn iter(
        &self,
    ) -> impl Iterator<
        Item = (
            SourceFraenkelGeneratorUsePositionId,
            &SourceFraenkelGeneratorUsePosition,
        ),
    > {
        self.rows
            .iter()
            .enumerate()
            .map(|(index, row)| (SourceFraenkelGeneratorUsePositionId::new(index), row))
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.rows.len()
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

/// A rejected checked Fraenkel generator binding-context handoff.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceFraenkelGeneratorBindingContextError {
    EnvironmentMismatch,
    InvalidStructuralDependency,
    InvalidResolverDependency,
    InvalidBindingContext {
        binding_context: SourceFraenkelGeneratorBindingContextId,
    },
    InvalidUsePosition {
        use_position: SourceFraenkelGeneratorUsePositionId,
    },
    InvalidEnvironment,
}

impl fmt::Display for SourceFraenkelGeneratorBindingContextError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EnvironmentMismatch => {
                formatter.write_str("Fraenkel generator binding-context environment mismatch")
            }
            Self::InvalidStructuralDependency => formatter
                .write_str("Fraenkel generator binding-context structural dependency is invalid"),
            Self::InvalidResolverDependency => formatter
                .write_str("Fraenkel generator binding-context resolver dependency is invalid"),
            Self::InvalidBindingContext { binding_context } => write!(
                formatter,
                "Fraenkel generator binding context {} is invalid",
                binding_context.index()
            ),
            Self::InvalidUsePosition { use_position } => write!(
                formatter,
                "Fraenkel generator use position {} is invalid",
                use_position.index()
            ),
            Self::InvalidEnvironment => {
                formatter.write_str("Fraenkel generator binding environment is invalid")
            }
        }
    }
}

impl Error for SourceFraenkelGeneratorBindingContextError {}

const FRAENKEL_GENERATOR_BINDING_SNAPSHOT_VERSION: &str =
    "source-fraenkel-generator-binding-context-dependencies-v1";
const FRAENKEL_GENERATOR_BINDING_SNAPSHOT_DOMAIN: &str =
    "source-fraenkel-generator-binding-context";

#[derive(Clone, PartialEq, Eq)]
struct SourceFraenkelGeneratorBindingDependencies {
    version: &'static str,
    domain: &'static str,
    structural: SourceTemplateFraenkelStructuralCompositionHandoff,
    resolver: FraenkelGeneratorVariableSourceCollection,
    typed_ast: TypedAst,
}

/// Opaque handoff for the checked Fraenkel generator binding context.
#[derive(Clone, PartialEq, Eq)]
pub struct SourceFraenkelGeneratorBindingContextHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    structural_summary: String,
    resolver_summary: String,
    binding_env: BindingEnv,
    bindings: SourceFraenkelGeneratorBindingContextTable,
    use_positions: SourceFraenkelGeneratorUsePositionTable,
    dependencies: SourceFraenkelGeneratorBindingDependencies,
}

impl SourceFraenkelGeneratorBindingContextHandoff {
    #[must_use]
    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    #[must_use]
    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    #[must_use]
    pub fn structural_summary(&self) -> &str {
        &self.structural_summary
    }

    #[must_use]
    pub fn resolver_summary(&self) -> &str {
        &self.resolver_summary
    }

    #[must_use]
    pub const fn binding_env(&self) -> &BindingEnv {
        &self.binding_env
    }

    #[must_use]
    pub const fn bindings(&self) -> &SourceFraenkelGeneratorBindingContextTable {
        &self.bindings
    }

    #[must_use]
    pub const fn use_positions(&self) -> &SourceFraenkelGeneratorUsePositionTable {
        &self.use_positions
    }

    #[must_use]
    pub fn debug_text(&self) -> String {
        format!(
            "source-fraenkel-generator-binding-context-v1|module={}.{}|bindings={}|use-positions={}",
            self.module_id.package().as_str(),
            self.module_id.path().as_str(),
            self.bindings.len(),
            self.use_positions.len(),
        )
    }

    fn validate(&self) -> Result<(), SourceFraenkelGeneratorBindingContextError> {
        let dependencies = &self.dependencies;
        if self.source_id != dependencies.structural.source_id()
            || self.source_id != dependencies.resolver.source_id()
            || self.source_id != dependencies.typed_ast.source_id()
            || &self.module_id != dependencies.structural.module_id()
            || &self.module_id != dependencies.resolver.module()
            || &self.module_id != dependencies.typed_ast.module_id()
        {
            return Err(SourceFraenkelGeneratorBindingContextError::EnvironmentMismatch);
        }

        validate_fraenkel_generator_dependency_header(dependencies)?;
        let structural =
            validate_structural_dependency(&dependencies.structural, &dependencies.typed_ast)
                .ok_or(SourceFraenkelGeneratorBindingContextError::InvalidStructuralDependency)?;
        if self.structural_summary != dependencies.structural.debug_text() {
            return Err(SourceFraenkelGeneratorBindingContextError::InvalidStructuralDependency);
        }
        let profile = validate_resolver_dependency(
            &dependencies.resolver,
            &dependencies.typed_ast,
            structural,
        )
        .ok_or(SourceFraenkelGeneratorBindingContextError::InvalidResolverDependency)?;
        if self.resolver_summary != dependencies.resolver.debug_text() {
            return Err(SourceFraenkelGeneratorBindingContextError::InvalidResolverDependency);
        }
        validate_binding_context_rows(&self.bindings, &profile)?;
        validate_use_position_rows(&self.use_positions, &profile)?;
        validate_fraenkel_generator_binding_env(
            &self.binding_env,
            self.source_id,
            &self.module_id,
            &profile,
        )
    }
}

/// Produces the one default-deny Fraenkel generator binding context.
#[derive(Debug, Clone, Copy, Default)]
pub struct SourceFraenkelGeneratorBindingContextProducer;

impl SourceFraenkelGeneratorBindingContextProducer {
    pub fn build(
        structural: &SourceTemplateFraenkelStructuralCompositionHandoff,
        resolver: &FraenkelGeneratorVariableSourceCollection,
        typed_ast: &TypedAst,
    ) -> Result<
        SourceFraenkelGeneratorBindingContextHandoff,
        SourceFraenkelGeneratorBindingContextError,
    > {
        if structural.source_id() != resolver.source_id()
            || structural.source_id() != typed_ast.source_id()
            || structural.module_id() != resolver.module()
            || structural.module_id() != typed_ast.module_id()
        {
            return Err(SourceFraenkelGeneratorBindingContextError::EnvironmentMismatch);
        }

        let dependencies = SourceFraenkelGeneratorBindingDependencies {
            version: FRAENKEL_GENERATOR_BINDING_SNAPSHOT_VERSION,
            domain: FRAENKEL_GENERATOR_BINDING_SNAPSHOT_DOMAIN,
            structural: structural.clone(),
            resolver: resolver.clone(),
            typed_ast: typed_ast.clone(),
        };
        let profile = validate_fraenkel_generator_dependencies(&dependencies)?;
        let binding_env = build_fraenkel_generator_binding_env(
            dependencies.structural.source_id(),
            dependencies.structural.module_id().clone(),
            &profile,
        )?;
        let binding_context = SourceFraenkelGeneratorBindingContext {
            composition: profile.composition,
            resolver_binding: profile.resolver_binding,
            context: BindingContextId::new(1),
            binding: BindingId::new(0),
            source_ordinal: 0,
        };
        let use_positions = SourceFraenkelGeneratorUsePositionTable {
            rows: (0..3)
                .map(|index| SourceFraenkelGeneratorUsePosition {
                    binding_context: SourceFraenkelGeneratorBindingContextId::new(0),
                    resolver_use_index: index,
                    source_ordinal: index,
                    lookup_ordinal: index + 1,
                })
                .collect(),
        };
        let handoff = SourceFraenkelGeneratorBindingContextHandoff {
            source_id: dependencies.structural.source_id(),
            module_id: dependencies.structural.module_id().clone(),
            structural_summary: dependencies.structural.debug_text(),
            resolver_summary: dependencies.resolver.debug_text(),
            binding_env,
            bindings: SourceFraenkelGeneratorBindingContextTable {
                rows: vec![binding_context],
            },
            use_positions,
            dependencies,
        };
        handoff.validate()?;
        Ok(handoff)
    }
}

dense_id!(SourceFraenkelGeneratorBoundUseId);

/// One checked Fraenkel generator use mapped to its checker binding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFraenkelGeneratorBoundUse {
    use_position: SourceFraenkelGeneratorUsePositionId,
    binding_context: SourceFraenkelGeneratorBindingContextId,
    resolver_use_index: usize,
    source_ordinal: usize,
    lookup_ordinal: usize,
    context: BindingContextId,
    binding: BindingId,
}

impl SourceFraenkelGeneratorBoundUse {
    #[must_use]
    pub const fn use_position(&self) -> SourceFraenkelGeneratorUsePositionId {
        self.use_position
    }

    #[must_use]
    pub const fn binding_context(&self) -> SourceFraenkelGeneratorBindingContextId {
        self.binding_context
    }

    #[must_use]
    pub const fn resolver_use_index(&self) -> usize {
        self.resolver_use_index
    }

    #[must_use]
    pub const fn source_ordinal(&self) -> usize {
        self.source_ordinal
    }

    #[must_use]
    pub const fn lookup_ordinal(&self) -> usize {
        self.lookup_ordinal
    }

    #[must_use]
    pub const fn context(&self) -> BindingContextId {
        self.context
    }

    #[must_use]
    pub const fn binding(&self) -> BindingId {
        self.binding
    }
}

/// Dense checked Fraenkel generator bound uses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFraenkelGeneratorBoundUseTable {
    rows: Vec<SourceFraenkelGeneratorBoundUse>,
}

impl SourceFraenkelGeneratorBoundUseTable {
    #[must_use]
    pub fn get(
        &self,
        id: SourceFraenkelGeneratorBoundUseId,
    ) -> Option<&SourceFraenkelGeneratorBoundUse> {
        self.rows.get(id.index())
    }

    pub fn iter(
        &self,
    ) -> impl Iterator<
        Item = (
            SourceFraenkelGeneratorBoundUseId,
            &SourceFraenkelGeneratorBoundUse,
        ),
    > {
        self.rows
            .iter()
            .enumerate()
            .map(|(index, row)| (SourceFraenkelGeneratorBoundUseId::new(index), row))
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.rows.len()
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

/// A rejected Fraenkel generator bound-use handoff.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceFraenkelGeneratorBoundUseError {
    EnvironmentMismatch,
    InvalidBindingContextDependency,
    InvalidBoundUse {
        bound_use: SourceFraenkelGeneratorBoundUseId,
    },
}

impl fmt::Display for SourceFraenkelGeneratorBoundUseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EnvironmentMismatch => {
                formatter.write_str("Fraenkel generator bound-use environment mismatch")
            }
            Self::InvalidBindingContextDependency => formatter
                .write_str("Fraenkel generator bound-use binding-context dependency is invalid"),
            Self::InvalidBoundUse { bound_use } => write!(
                formatter,
                "Fraenkel generator bound use {} is invalid",
                bound_use.index()
            ),
        }
    }
}

impl Error for SourceFraenkelGeneratorBoundUseError {}

const FRAENKEL_GENERATOR_BOUND_USE_DEPENDENCY_VERSION: &str =
    "source-fraenkel-generator-bound-use-dependency-v1";
const FRAENKEL_GENERATOR_BOUND_USE_DEPENDENCY_DOMAIN: &str = "source-fraenkel-generator-bound-use";

#[derive(Clone, PartialEq, Eq)]
struct SourceFraenkelGeneratorBoundUseDependencies {
    version: &'static str,
    domain: &'static str,
    binding_context: SourceFraenkelGeneratorBindingContextHandoff,
}

/// Opaque handoff for checked Fraenkel generator bound uses.
#[derive(Clone, PartialEq, Eq)]
pub struct SourceFraenkelGeneratorBoundUseHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    dependency_summary: String,
    bound_uses: SourceFraenkelGeneratorBoundUseTable,
    dependencies: SourceFraenkelGeneratorBoundUseDependencies,
}

impl SourceFraenkelGeneratorBoundUseHandoff {
    #[must_use]
    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    #[must_use]
    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    #[must_use]
    pub fn dependency_summary(&self) -> &str {
        &self.dependency_summary
    }

    #[must_use]
    pub const fn bound_uses(&self) -> &SourceFraenkelGeneratorBoundUseTable {
        &self.bound_uses
    }

    #[must_use]
    pub fn debug_text(&self) -> String {
        format!(
            "source-fraenkel-generator-bound-use-v1|module={}.{}|bound-uses={}",
            self.module_id.package().as_str(),
            self.module_id.path().as_str(),
            self.bound_uses.len(),
        )
    }

    fn validate(&self) -> Result<(), SourceFraenkelGeneratorBoundUseError> {
        let dependency = &self.dependencies.binding_context;
        if self.source_id != dependency.source_id() || &self.module_id != dependency.module_id() {
            return Err(SourceFraenkelGeneratorBoundUseError::EnvironmentMismatch);
        }
        validate_fraenkel_generator_bound_use_dependency(&self.dependencies)?;
        if self.dependency_summary != dependency.debug_text() {
            return Err(SourceFraenkelGeneratorBoundUseError::InvalidBindingContextDependency);
        }
        validate_fraenkel_generator_bound_use_rows(&self.bound_uses, dependency)
    }
}

/// Produces the default-deny Fraenkel generator bound-use association.
#[derive(Debug, Clone, Copy, Default)]
pub struct SourceFraenkelGeneratorBoundUseProducer;

impl SourceFraenkelGeneratorBoundUseProducer {
    pub fn build(
        binding_context: &SourceFraenkelGeneratorBindingContextHandoff,
    ) -> Result<SourceFraenkelGeneratorBoundUseHandoff, SourceFraenkelGeneratorBoundUseError> {
        binding_context
            .validate()
            .map_err(|_| SourceFraenkelGeneratorBoundUseError::InvalidBindingContextDependency)?;
        let dependencies = SourceFraenkelGeneratorBoundUseDependencies {
            version: FRAENKEL_GENERATOR_BOUND_USE_DEPENDENCY_VERSION,
            domain: FRAENKEL_GENERATOR_BOUND_USE_DEPENDENCY_DOMAIN,
            binding_context: binding_context.clone(),
        };
        let bound_uses = build_fraenkel_generator_bound_use_rows(binding_context)?;
        let handoff = SourceFraenkelGeneratorBoundUseHandoff {
            source_id: binding_context.source_id(),
            module_id: binding_context.module_id().clone(),
            dependency_summary: binding_context.debug_text(),
            bound_uses,
            dependencies,
        };
        handoff.validate()?;
        Ok(handoff)
    }
}

fn validate_fraenkel_generator_bound_use_dependency(
    dependencies: &SourceFraenkelGeneratorBoundUseDependencies,
) -> Result<(), SourceFraenkelGeneratorBoundUseError> {
    if dependencies.version != FRAENKEL_GENERATOR_BOUND_USE_DEPENDENCY_VERSION
        || dependencies.domain != FRAENKEL_GENERATOR_BOUND_USE_DEPENDENCY_DOMAIN
    {
        return Err(SourceFraenkelGeneratorBoundUseError::InvalidBindingContextDependency);
    }
    dependencies
        .binding_context
        .validate()
        .map_err(|_| SourceFraenkelGeneratorBoundUseError::InvalidBindingContextDependency)
}

fn build_fraenkel_generator_bound_use_rows(
    binding_context: &SourceFraenkelGeneratorBindingContextHandoff,
) -> Result<SourceFraenkelGeneratorBoundUseTable, SourceFraenkelGeneratorBoundUseError> {
    let rows = binding_context
        .use_positions()
        .iter()
        .map(|(use_position, position)| {
            let binding = binding_context
                .bindings()
                .get(position.binding_context())
                .ok_or(SourceFraenkelGeneratorBoundUseError::InvalidBoundUse {
                    bound_use: SourceFraenkelGeneratorBoundUseId::new(use_position.index()),
                })?;
            let lookup = binding_context.binding_env().lookup(&BindingLookupSite::new(
                "x",
                binding.context(),
                None,
                position.lookup_ordinal(),
            ));
            if !matches!(lookup, Ok(BindingLookupResult::Local(found)) if found == binding.binding())
            {
                return Err(SourceFraenkelGeneratorBoundUseError::InvalidBoundUse {
                    bound_use: SourceFraenkelGeneratorBoundUseId::new(use_position.index()),
                });
            }
            Ok(SourceFraenkelGeneratorBoundUse {
                use_position,
                binding_context: position.binding_context(),
                resolver_use_index: position.resolver_use_index(),
                source_ordinal: position.source_ordinal(),
                lookup_ordinal: position.lookup_ordinal(),
                context: binding.context(),
                binding: binding.binding(),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let table = SourceFraenkelGeneratorBoundUseTable { rows };
    validate_fraenkel_generator_bound_use_rows(&table, binding_context)?;
    Ok(table)
}

fn validate_fraenkel_generator_bound_use_rows(
    rows: &SourceFraenkelGeneratorBoundUseTable,
    dependency: &SourceFraenkelGeneratorBindingContextHandoff,
) -> Result<(), SourceFraenkelGeneratorBoundUseError> {
    if rows.len() != 3 {
        return Err(SourceFraenkelGeneratorBoundUseError::InvalidBoundUse {
            bound_use: SourceFraenkelGeneratorBoundUseId::new(0),
        });
    }
    for (id, row) in rows.iter() {
        let invalid = || SourceFraenkelGeneratorBoundUseError::InvalidBoundUse { bound_use: id };
        let use_position_id = SourceFraenkelGeneratorUsePositionId::new(id.index());
        let position = dependency
            .use_positions()
            .get(use_position_id)
            .ok_or_else(invalid)?;
        let binding = dependency
            .bindings()
            .get(position.binding_context())
            .ok_or_else(invalid)?;
        let lookup = dependency.binding_env().lookup(&BindingLookupSite::new(
            "x",
            binding.context(),
            None,
            position.lookup_ordinal(),
        ));
        if rows.get(id) != Some(row)
            || row.use_position() != use_position_id
            || row.binding_context() != position.binding_context()
            || row.resolver_use_index() != position.resolver_use_index()
            || row.source_ordinal() != position.source_ordinal()
            || row.lookup_ordinal() != position.lookup_ordinal()
            || row.context() != binding.context()
            || row.binding() != binding.binding()
            || !matches!(lookup, Ok(BindingLookupResult::Local(found)) if found == binding.binding())
        {
            return Err(invalid());
        }
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct FraenkelGeneratorBindingProfile {
    composition: SourceTemplateFraenkelStructuralCompositionId,
    resolver_binding: FraenkelGeneratorVariableBindingId,
    comprehension_range: SourceRange,
    binder_range: SourceRange,
    type_range: SourceRange,
}

fn validate_fraenkel_generator_dependencies(
    dependencies: &SourceFraenkelGeneratorBindingDependencies,
) -> Result<FraenkelGeneratorBindingProfile, SourceFraenkelGeneratorBindingContextError> {
    validate_fraenkel_generator_dependency_header(dependencies)?;
    let structural =
        validate_structural_dependency(&dependencies.structural, &dependencies.typed_ast)
            .ok_or(SourceFraenkelGeneratorBindingContextError::InvalidStructuralDependency)?;
    validate_resolver_dependency(&dependencies.resolver, &dependencies.typed_ast, structural)
        .ok_or(SourceFraenkelGeneratorBindingContextError::InvalidResolverDependency)
}

fn validate_fraenkel_generator_dependency_header(
    dependencies: &SourceFraenkelGeneratorBindingDependencies,
) -> Result<(), SourceFraenkelGeneratorBindingContextError> {
    if dependencies.structural.source_id() != dependencies.resolver.source_id()
        || dependencies.structural.source_id() != dependencies.typed_ast.source_id()
        || dependencies.structural.module_id() != dependencies.resolver.module()
        || dependencies.structural.module_id() != dependencies.typed_ast.module_id()
    {
        return Err(SourceFraenkelGeneratorBindingContextError::EnvironmentMismatch);
    }
    if dependencies.version != FRAENKEL_GENERATOR_BINDING_SNAPSHOT_VERSION
        || dependencies.domain != FRAENKEL_GENERATOR_BINDING_SNAPSHOT_DOMAIN
    {
        return Err(SourceFraenkelGeneratorBindingContextError::InvalidStructuralDependency);
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct StructuralDependencyProfile {
    composition: SourceTemplateFraenkelStructuralCompositionId,
    definition_block: TypedNodeId,
    functor_definition: TypedNodeId,
    comprehension: TypedNodeId,
    segment: TypedNodeId,
    binder: TypedNodeId,
    type_expression: TypedNodeId,
    mapper_role_owner: TypedNodeId,
    mapper_term_reference: TypedNodeId,
    mapper_identifier: TypedNodeId,
    first_condition_role_owner: TypedNodeId,
    first_condition_term_reference: TypedNodeId,
    first_condition_identifier: TypedNodeId,
    second_condition_role_owner: TypedNodeId,
    second_condition_term_reference: TypedNodeId,
    second_condition_identifier: TypedNodeId,
}

fn validate_structural_dependency(
    handoff: &SourceTemplateFraenkelStructuralCompositionHandoff,
    typed_ast: &TypedAst,
) -> Option<StructuralDependencyProfile> {
    let rows = handoff.compositions().iter().collect::<Vec<_>>();
    let [(composition, row)] = rows.as_slice() else {
        return None;
    };
    if composition.index() != 0
        || handoff.compositions().get(*composition) != Some(*row)
        || handoff
            .compositions()
            .get(SourceTemplateFraenkelStructuralCompositionId::new(1))
            .is_some()
        || row.template_association().index() != 0
        || row.template_binding().index() != 0
        || row.generator_binding().index() != 0
        || !validate_structural_nodes(typed_ast, row)
        || !validate_structural_edges(typed_ast, row)
        || (
            row.mapper_source_ordinal(),
            row.mapper_role_source_ordinal(),
            row.first_condition_source_ordinal(),
            row.first_condition_role_source_ordinal(),
            row.second_condition_source_ordinal(),
            row.second_condition_role_source_ordinal(),
        ) != (0, 0, 1, 0, 2, 1)
    {
        return None;
    }
    Some(StructuralDependencyProfile {
        composition: *composition,
        definition_block: row.definition_block(),
        functor_definition: row.functor_definition(),
        comprehension: row.comprehension(),
        segment: row.segment(),
        binder: row.generator_binder(),
        type_expression: row.type_expression(),
        mapper_role_owner: row.mapper_role_owner(),
        mapper_term_reference: row.mapper_term_reference(),
        mapper_identifier: row.mapper_identifier(),
        first_condition_role_owner: row.first_condition_role_owner(),
        first_condition_term_reference: row.first_condition_term_reference(),
        first_condition_identifier: row.first_condition_identifier(),
        second_condition_role_owner: row.second_condition_role_owner(),
        second_condition_term_reference: row.second_condition_term_reference(),
        second_condition_identifier: row.second_condition_identifier(),
    })
}

fn validate_structural_nodes(
    typed_ast: &TypedAst,
    row: &SourceTemplateFraenkelStructuralComposition,
) -> bool {
    [
        (row.definition_block(), "DefinitionBlockItem"),
        (row.parameter(), "TemplateParameter"),
        (row.template_binder(), "Identifier"),
        (row.type_head(), "TypeHead"),
        (row.template_identifier(), "Identifier"),
        (row.functor_definition(), "FunctorDefinition"),
        (row.comprehension(), "SetComprehension"),
        (row.segment(), "ComprehensionVariableSegment"),
        (row.generator_binder(), "Identifier"),
        (row.type_expression(), "TypeExpression"),
        (row.mapper_role_owner(), "TermExpression"),
        (row.mapper_term_reference(), "TermReference"),
        (row.mapper_identifier(), "Identifier"),
        (row.first_condition_role_owner(), "FormulaExpression"),
        (row.first_condition_term_reference(), "TermReference"),
        (row.first_condition_identifier(), "Identifier"),
        (row.second_condition_role_owner(), "FormulaExpression"),
        (row.second_condition_term_reference(), "TermReference"),
        (row.second_condition_identifier(), "Identifier"),
    ]
    .into_iter()
    .all(|(id, kind)| exact_stored_typed_node(typed_ast, id, kind).is_some())
}

fn validate_structural_edges(
    typed_ast: &TypedAst,
    row: &SourceTemplateFraenkelStructuralComposition,
) -> bool {
    let Some(definition_range) = typed_range(typed_ast, row.definition_block()) else {
        return false;
    };
    let Some(parameter_range) = typed_range(typed_ast, row.parameter()) else {
        return false;
    };
    let Some(template_binder_range) = typed_range(typed_ast, row.template_binder()) else {
        return false;
    };
    let Some(type_head_range) = typed_range(typed_ast, row.type_head()) else {
        return false;
    };
    let Some(template_identifier_range) = typed_range(typed_ast, row.template_identifier()) else {
        return false;
    };
    let Some(functor_range) = typed_range(typed_ast, row.functor_definition()) else {
        return false;
    };
    let Some(comprehension_range) = typed_range(typed_ast, row.comprehension()) else {
        return false;
    };
    let Some(segment_range) = typed_range(typed_ast, row.segment()) else {
        return false;
    };
    let Some(generator_binder_range) = typed_range(typed_ast, row.generator_binder()) else {
        return false;
    };
    let Some(type_expression_range) = typed_range(typed_ast, row.type_expression()) else {
        return false;
    };
    let Some(mapper_range) = typed_range(typed_ast, row.mapper_role_owner()) else {
        return false;
    };
    let Some(mapper_reference_range) = typed_range(typed_ast, row.mapper_term_reference()) else {
        return false;
    };
    let Some(mapper_identifier_range) = typed_range(typed_ast, row.mapper_identifier()) else {
        return false;
    };
    if !range_contains(definition_range, parameter_range)
        || !range_contains(parameter_range, template_binder_range)
        || !range_contains(definition_range, type_head_range)
        || !range_contains(type_head_range, template_identifier_range)
        || !range_contains(definition_range, functor_range)
        || !range_contains(functor_range, comprehension_range)
        || !range_contains(comprehension_range, segment_range)
        || !range_contains(segment_range, generator_binder_range)
        || !range_contains(segment_range, type_expression_range)
        || !range_contains(type_expression_range, type_head_range)
        || !range_contains(comprehension_range, mapper_range)
        || !range_contains(mapper_range, mapper_reference_range)
        || !range_contains(mapper_reference_range, mapper_identifier_range)
        || !has_typed_edge(typed_ast, row.definition_block(), row.parameter())
        || !has_typed_edge(typed_ast, row.definition_block(), row.functor_definition())
        || !has_typed_edge(typed_ast, row.parameter(), row.template_binder())
        || !has_typed_edge(typed_ast, row.type_head(), row.template_identifier())
        || !has_typed_edge(typed_ast, row.comprehension(), row.segment())
        || !has_typed_edge(typed_ast, row.segment(), row.generator_binder())
        || !has_typed_edge(typed_ast, row.segment(), row.type_expression())
        || !has_typed_edge(typed_ast, row.type_expression(), row.type_head())
        || !has_typed_edge(typed_ast, row.comprehension(), row.mapper_role_owner())
        || !has_typed_edge(
            typed_ast,
            row.mapper_role_owner(),
            row.mapper_term_reference(),
        )
        || !has_typed_edge(
            typed_ast,
            row.mapper_term_reference(),
            row.mapper_identifier(),
        )
    {
        return false;
    }
    let Some(term_definiens) =
        exact_typed_child(typed_ast, row.functor_definition(), "TermDefiniens")
    else {
        return false;
    };
    let Some(term_expression) = exact_typed_child(typed_ast, term_definiens, "TermExpression")
    else {
        return false;
    };
    if !has_typed_edge(typed_ast, term_expression, row.comprehension()) {
        return false;
    }
    validate_condition_structure(
        typed_ast,
        row.comprehension(),
        row.first_condition_role_owner(),
        row.first_condition_term_reference(),
        row.first_condition_identifier(),
    ) && validate_condition_structure(
        typed_ast,
        row.comprehension(),
        row.second_condition_role_owner(),
        row.second_condition_term_reference(),
        row.second_condition_identifier(),
    )
}

fn validate_condition_structure(
    typed_ast: &TypedAst,
    comprehension: TypedNodeId,
    owner: TypedNodeId,
    term_reference: TypedNodeId,
    identifier: TypedNodeId,
) -> bool {
    let Some(owner_range) = typed_range(typed_ast, owner) else {
        return false;
    };
    let Some(comprehension_range) = typed_range(typed_ast, comprehension) else {
        return false;
    };
    let Some(reference_range) = typed_range(typed_ast, term_reference) else {
        return false;
    };
    let Some(identifier_range) = typed_range(typed_ast, identifier) else {
        return false;
    };
    let Some(prefix) = exact_typed_child(typed_ast, owner, "PrefixFormula(Not)") else {
        return false;
    };
    let Some(predicate) = exact_typed_child(typed_ast, prefix, "BuiltinPredicateApplication")
    else {
        return false;
    };
    let Some(term_expression) =
        exact_typed_child_containing(typed_ast, predicate, "TermExpression", term_reference)
    else {
        return false;
    };
    let Some(prefix_range) = typed_range(typed_ast, prefix) else {
        return false;
    };
    let Some(predicate_range) = typed_range(typed_ast, predicate) else {
        return false;
    };
    let Some(term_range) = typed_range(typed_ast, term_expression) else {
        return false;
    };
    range_contains(comprehension_range, owner_range)
        && range_contains(owner_range, prefix_range)
        && range_contains(prefix_range, predicate_range)
        && range_contains(predicate_range, term_range)
        && range_contains(term_range, reference_range)
        && range_contains(reference_range, identifier_range)
        && has_typed_edge(typed_ast, predicate, term_expression)
        && has_typed_edge(typed_ast, term_expression, term_reference)
        && has_typed_edge(typed_ast, term_reference, identifier)
        && has_typed_edge(typed_ast, comprehension, owner)
}

fn validate_resolver_dependency(
    resolver: &FraenkelGeneratorVariableSourceCollection,
    typed_ast: &TypedAst,
    structural: StructuralDependencyProfile,
) -> Option<FraenkelGeneratorBindingProfile> {
    let binding_rows = resolver.bindings().iter().collect::<Vec<_>>();
    let [(binding_id, binding)] = binding_rows.as_slice() else {
        return None;
    };
    if binding_id.index() != 0
        || resolver.bindings().get(*binding_id) != Some(*binding)
        || resolver
            .bindings()
            .get(FraenkelGeneratorVariableBindingId::new(1))
            .is_some()
        || binding.source_ordinal() != 0
        || binding.spelling() != "x"
        || !binding_matches_structural(typed_ast, binding, structural)
    {
        return None;
    }
    let uses = resolver.uses().iter().collect::<Vec<_>>();
    if uses.len() != 3 || resolver.uses().get(3).is_some() {
        return None;
    }
    let expected = [
        (
            FraenkelGeneratorVariableUseRole::Mapper,
            0,
            0,
            structural.mapper_role_owner,
            structural.mapper_term_reference,
            structural.mapper_identifier,
        ),
        (
            FraenkelGeneratorVariableUseRole::Condition,
            1,
            0,
            structural.first_condition_role_owner,
            structural.first_condition_term_reference,
            structural.first_condition_identifier,
        ),
        (
            FraenkelGeneratorVariableUseRole::Condition,
            2,
            1,
            structural.second_condition_role_owner,
            structural.second_condition_term_reference,
            structural.second_condition_identifier,
        ),
    ];
    if !uses.iter().zip(expected).all(
        |(link, (role, source_ordinal, role_source_ordinal, owner, reference, identifier))| {
            use_matches_structural(
                typed_ast,
                link,
                *binding_id,
                role,
                source_ordinal,
                role_source_ordinal,
                structural,
                owner,
                reference,
                identifier,
            )
        },
    ) {
        return None;
    }
    Some(FraenkelGeneratorBindingProfile {
        composition: structural.composition,
        resolver_binding: *binding_id,
        comprehension_range: typed_range(typed_ast, structural.comprehension)?,
        binder_range: typed_range(typed_ast, structural.binder)?,
        type_range: typed_range(typed_ast, structural.type_expression)?,
    })
}

fn binding_matches_structural(
    typed_ast: &TypedAst,
    binding: &FraenkelGeneratorVariableBinding,
    structural: StructuralDependencyProfile,
) -> bool {
    typed_for_resolved(typed_ast, binding.definition_block()) == Some(structural.definition_block)
        && typed_for_resolved(typed_ast, binding.functor_definition())
            == Some(structural.functor_definition)
        && typed_for_resolved(typed_ast, binding.comprehension()) == Some(structural.comprehension)
        && typed_for_resolved(typed_ast, binding.segment()) == Some(structural.segment)
        && typed_for_resolved(typed_ast, binding.binder()) == Some(structural.binder)
        && Some(binding.segment_range()) == typed_range(typed_ast, structural.segment)
        && Some(binding.binder_range()) == typed_range(typed_ast, structural.binder)
}

// Rationale: each frozen resolver getter maps to an independently authenticated typed field.
#[allow(clippy::too_many_arguments)]
fn use_matches_structural(
    typed_ast: &TypedAst,
    link: &FraenkelGeneratorVariableUseLink,
    binding: FraenkelGeneratorVariableBindingId,
    expected_role: FraenkelGeneratorVariableUseRole,
    expected_source_ordinal: usize,
    expected_role_source_ordinal: usize,
    structural: StructuralDependencyProfile,
    expected_owner: TypedNodeId,
    expected_reference: TypedNodeId,
    expected_identifier: TypedNodeId,
) -> bool {
    link.binding() == binding
        && link.role() == expected_role
        && link.source_ordinal() == expected_source_ordinal
        && link.role_source_ordinal() == expected_role_source_ordinal
        && typed_for_resolved(typed_ast, link.definition_block())
            == Some(structural.definition_block)
        && typed_for_resolved(typed_ast, link.functor_definition())
            == Some(structural.functor_definition)
        && typed_for_resolved(typed_ast, link.comprehension()) == Some(structural.comprehension)
        && typed_for_resolved(typed_ast, link.role_owner()) == Some(expected_owner)
        && typed_for_resolved(typed_ast, link.term_reference()) == Some(expected_reference)
        && typed_for_resolved(typed_ast, link.identifier()) == Some(expected_identifier)
        && Some(link.identifier_range()) == typed_range(typed_ast, expected_identifier)
}

fn build_fraenkel_generator_binding_env(
    source_id: SourceId,
    module_id: ModuleId,
    profile: &FraenkelGeneratorBindingProfile,
) -> Result<BindingEnv, SourceFraenkelGeneratorBindingContextError> {
    let mut bindings = BindingTable::new();
    let binding = bindings.insert(BindingDraft {
        spelling: "x".to_owned(),
        kind: BindingKind::QuantifierBinder,
        identity: BinderIdentity::SourceBound {
            context: BindingContextId::new(1),
            ordinal: 0,
        },
        owner_context: BindingContextId::new(1),
        declaration_range: profile.binder_range,
        visible_after_ordinal: 0,
        type_site: BindingTypeSite::Source(profile.type_range),
        status: BindingStatus::Active,
        captured: CapturedFreeVariables::default(),
        diagnostics: Vec::new(),
        recovery: BindingRecoveryState::Normal,
    });
    if binding != BindingId::new(0) {
        return Err(SourceFraenkelGeneratorBindingContextError::InvalidEnvironment);
    }
    let mut contexts = BindingContextTable::new();
    let root = contexts.insert(BindingContextDraft {
        owner: BindingContextOwner::Module,
        parent: None,
        layer: BindingContextLayer::Module,
        lexical_scope: None,
        bindings: Vec::new(),
        visible_bindings: Vec::new(),
        recovery: BindingContextRecovery::Normal,
    });
    let context = contexts.insert(BindingContextDraft {
        owner: BindingContextOwner::SourceComprehension {
            source_range: profile.comprehension_range,
        },
        parent: Some(root),
        layer: BindingContextLayer::Expression,
        lexical_scope: None,
        bindings: vec![binding],
        visible_bindings: vec![binding],
        recovery: BindingContextRecovery::Normal,
    });
    if root != BindingContextId::new(0) || context != BindingContextId::new(1) {
        return Err(SourceFraenkelGeneratorBindingContextError::InvalidEnvironment);
    }
    BindingEnv::try_new(BindingEnvParts {
        source_id,
        module_id,
        contexts,
        bindings,
        diagnostics: Default::default(),
    })
    .map_err(|_| SourceFraenkelGeneratorBindingContextError::InvalidEnvironment)
}

fn validate_binding_context_rows(
    rows: &SourceFraenkelGeneratorBindingContextTable,
    profile: &FraenkelGeneratorBindingProfile,
) -> Result<(), SourceFraenkelGeneratorBindingContextError> {
    let entries = rows.iter().collect::<Vec<_>>();
    let Some((id, row)) = entries.first().copied() else {
        return Err(
            SourceFraenkelGeneratorBindingContextError::InvalidBindingContext {
                binding_context: SourceFraenkelGeneratorBindingContextId::new(0),
            },
        );
    };
    if entries.len() != 1
        || id.index() != 0
        || rows.get(id) != Some(row)
        || rows
            .get(SourceFraenkelGeneratorBindingContextId::new(1))
            .is_some()
        || row.composition() != profile.composition
        || row.resolver_binding() != profile.resolver_binding
        || row.context() != BindingContextId::new(1)
        || row.binding() != BindingId::new(0)
        || row.source_ordinal() != 0
    {
        return Err(
            SourceFraenkelGeneratorBindingContextError::InvalidBindingContext {
                binding_context: id,
            },
        );
    }
    Ok(())
}

fn validate_use_position_rows(
    rows: &SourceFraenkelGeneratorUsePositionTable,
    _profile: &FraenkelGeneratorBindingProfile,
) -> Result<(), SourceFraenkelGeneratorBindingContextError> {
    if rows.len() != 3 {
        return Err(
            SourceFraenkelGeneratorBindingContextError::InvalidUsePosition {
                use_position: SourceFraenkelGeneratorUsePositionId::new(0),
            },
        );
    }
    for (id, row) in rows.iter() {
        let expected = id.index();
        if rows.get(id) != Some(row)
            || row.binding_context() != SourceFraenkelGeneratorBindingContextId::new(0)
            || row.resolver_use_index() != expected
            || row.source_ordinal() != expected
            || row.lookup_ordinal() != expected + 1
        {
            return Err(
                SourceFraenkelGeneratorBindingContextError::InvalidUsePosition { use_position: id },
            );
        }
    }
    Ok(())
}

fn validate_fraenkel_generator_binding_env(
    binding_env: &BindingEnv,
    source_id: SourceId,
    module_id: &ModuleId,
    profile: &FraenkelGeneratorBindingProfile,
) -> Result<(), SourceFraenkelGeneratorBindingContextError> {
    if binding_env.source_id() != source_id || binding_env.module_id() != module_id {
        return Err(SourceFraenkelGeneratorBindingContextError::InvalidEnvironment);
    }
    let expected = format!(
        "binding-env-debug-v1\nmodule: {}::{}\ncontexts:\n  context#0 owner=module parent=none layer=module scope=none bindings=[] visible=[] recovery=normal\n  context#1 owner=source-comprehension({}..{}) parent=context#0 layer=expression scope=none bindings=[binding#0] visible=[binding#0] recovery=normal\nbindings:\n  binding#0 spelling=\"x\" kind=quantifier_binder owner=context#1 identity=source_bound(context#1, ordinal=0) range={}..{} visible_after=0 type=source({}..{}) status=active captured=[] diagnostics=[] recovery=normal\ndiagnostics:\n",
        module_id.package().as_str(),
        module_id.path().as_str(),
        profile.comprehension_range.start,
        profile.comprehension_range.end,
        profile.binder_range.start,
        profile.binder_range.end,
        profile.type_range.start,
        profile.type_range.end,
    );
    if binding_env.debug_text() != expected
        || !matches!(
            binding_env.lookup(&BindingLookupSite::new(
                "x",
                BindingContextId::new(1),
                None,
                0,
            )),
            Ok(BindingLookupResult::ForwardReference { candidates, .. })
                if candidates == vec![BindingId::new(0)]
        )
        || !(1..=3).all(|ordinal| {
            matches!(
                binding_env.lookup(&BindingLookupSite::new(
                    "x",
                    BindingContextId::new(1),
                    None,
                    ordinal,
                )),
                Ok(BindingLookupResult::Local(binding)) if binding == BindingId::new(0)
            )
        })
    {
        return Err(SourceFraenkelGeneratorBindingContextError::InvalidEnvironment);
    }
    Ok(())
}

fn exact_stored_typed_node<'a>(
    typed_ast: &'a TypedAst,
    id: TypedNodeId,
    kind: &str,
) -> Option<&'a TypedNode> {
    let node = typed_ast.nodes().node(id)?;
    let resolved = node.resolved_node?;
    (node.kind.as_str() == kind
        && node.recovery == NodeRecoveryState::Normal
        && typed_for_resolved(typed_ast, resolved) == Some(id))
    .then_some(node)
}

fn typed_for_resolved(typed_ast: &TypedAst, resolved_node: ResolvedNodeId) -> Option<TypedNodeId> {
    let mut match_id = None;
    for (typed_id, node) in typed_ast.nodes().iter() {
        if node.resolved_node == Some(resolved_node) && match_id.replace(typed_id).is_some() {
            return None;
        }
    }
    match_id
}

fn typed_range(typed_ast: &TypedAst, id: TypedNodeId) -> Option<SourceRange> {
    let SourceAnchor::Range(range) = typed_ast.nodes().node(id)?.anchor else {
        return None;
    };
    (range.source_id == typed_ast.source_id() && range.start < range.end).then_some(range)
}

fn range_contains(parent: SourceRange, child: SourceRange) -> bool {
    parent.source_id == child.source_id && parent.start <= child.start && child.end <= parent.end
}

fn has_typed_edge(typed_ast: &TypedAst, parent: TypedNodeId, child: TypedNodeId) -> bool {
    typed_ast
        .nodes()
        .node(parent)
        .is_some_and(|node| node.children.contains(&child))
}

fn exact_typed_child(typed_ast: &TypedAst, parent: TypedNodeId, kind: &str) -> Option<TypedNodeId> {
    let parent = typed_ast.nodes().node(parent)?;
    let mut matches = parent.children.iter().filter_map(|child| {
        let node = typed_ast.nodes().node(*child)?;
        (node.kind.as_str() == kind).then_some(*child)
    });
    let child = matches.next()?;
    (matches.next().is_none()
        && typed_ast
            .nodes()
            .node(child)
            .is_some_and(|node| node.recovery == NodeRecoveryState::Normal))
    .then_some(child)
}

fn exact_typed_child_containing(
    typed_ast: &TypedAst,
    parent: TypedNodeId,
    kind: &str,
    descendant: TypedNodeId,
) -> Option<TypedNodeId> {
    let parent = typed_ast.nodes().node(parent)?;
    let mut matches = parent.children.iter().filter_map(|child| {
        let node = typed_ast.nodes().node(*child)?;
        (node.kind.as_str() == kind && node.children.contains(&descendant)).then_some(*child)
    });
    let child = matches.next()?;
    (matches.next().is_none()
        && typed_ast
            .nodes()
            .node(child)
            .is_some_and(|node| node.recovery == NodeRecoveryState::Normal))
    .then_some(child)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::{
        binding_env::{
            BindingContextDraft, BindingContextId, BindingContextLayer, BindingContextOwner,
            BindingContextRecovery, BindingContextTable, BindingDiagnosticClass,
            BindingDiagnosticDraft, BindingDiagnosticRecovery, BindingDiagnosticSeverity,
            BindingDiagnosticTable, BindingEnv, BindingEnvError, BindingEnvParts, BindingId,
            BindingTable,
        },
        cluster_trace::ClusterFactTable,
        overload_resolution::{
            CandidateViabilityInput, CandidateViabilityOutput, OverloadCandidateInput,
            OverloadCollectionOutput, OverloadSelectionOutput, OverloadSiteInput,
            OverloadSiteResolutionInput, SpecificityComparisonInput, SpecificityGraphOutput,
            TemplateExpansionOutput,
        },
        resolved_typed_ast::{
            ResolvedNodeKindHint, ResolvedNodeKindHintKind, ResolvedTypedAst,
            ResolvedTypedAstError, ResolvedTypedAstInputs, SourceNodeRole,
        },
        source_atomic_formula::{
            SourceAtomicEdgeInput, SourceAtomicFormulaHandoffInput, SourceAtomicFormulaInput,
            SourceAtomicFormulaProducer, SourceAtomicRequestInput,
        },
        source_composite_formula::{
            SourceBinderTypeHead, SourceBinderTypeSiteId, SourceBinderTypeSiteInput,
            SourceCompositeFormulaHandoffInput, SourceCompositeFormulaInput,
            SourceCompositeFormulaProducer, SourceFormulaEdgeInput, SourceFormulaEdgeRole,
            SourceFormulaRequestInput, SourceFormulaRequestKind, SourceFormulaRootInput,
            SourceFormulaRootOwnership, SourceFormulaWrapperInput, SourceQuantifierBinderInput,
        },
        source_template_type_parameter_association::{
            SourceTemplateFraenkelStructuralCompositionHandoff,
            SourceTemplateFraenkelStructuralCompositionProducer,
            SourceTemplateTypeParameterAssociationProducer,
        },
        source_term::{
            SourceNestedFraenkelMapperPrimaryProducer, SourceNumericTypeRequestInput,
            SourcePrimaryTermHandoffInput, SourcePrimaryTermInput, SourcePrimaryTermProducer,
            SourcePrimaryTermReferenceInput,
        },
        typed_ast::{
            CoercionTable, InitialObligationTable, LocalTypeContextTable,
            StatementTransportTableForTest, TypeDiagnosticTable, TypeFactTable, TypeTable,
            TypedArenaBuilder, TypedAst, TypedAstError, TypedAstParts, TypedNode, TypedNodeId,
            TypedSiteRef,
        },
    };
    use mizar_resolve::{
        env::{SymbolEnv, SymbolEnvIndexes},
        names::{
            FraenkelGeneratorVariableSourceCollection, FraenkelGeneratorVariableSourceCollector,
            LocalTermBinding, LocalTermScope, TemplateTypeParameterSourceCollector,
        },
        resolved_ast::SurfaceResolvedArena,
    };
    use mizar_session::{
        BuildSnapshotId, InMemorySessionIdAllocator, ModulePath, PackageId,
        SessionIdAllocator as _, SourceAnchor,
    };
    use mizar_syntax as syntax;

    struct Fixture {
        source: SourceId,
        module: ModuleId,
        arena: TypedArena,
        composite_input: SourceCompositeFormulaHandoffInput,
        primary: SourcePrimaryTermHandoff,
        atomic: SourceAtomicFormulaHandoff,
        composite: SourceCompositeFormulaHandoff,
        input: SourceFormulaCompositionHandoffInput,
    }

    struct Task257B2Fixture {
        source: SourceId,
        module: ModuleId,
        arena: TypedArena,
        bindings: BindingEnv,
        primary_input: SourcePrimaryTermHandoffInput,
        atomic_input: SourceAtomicFormulaHandoffInput,
        primary: SourcePrimaryTermHandoff,
        atomic: SourceAtomicFormulaHandoff,
        composite: SourceCompositeFormulaHandoff,
        input: SourceFormulaCompositionHandoffInput,
    }

    struct Task257B3Fixture {
        source: SourceId,
        module: ModuleId,
        arena: TypedArena,
        bindings: BindingEnv,
        primary_input: SourcePrimaryTermHandoffInput,
        atomic_input: SourceAtomicFormulaHandoffInput,
        primary: SourcePrimaryTermHandoff,
        atomic: SourceAtomicFormulaHandoff,
        composite: SourceCompositeFormulaHandoff,
        input: SourceFormulaCompositionHandoffInput,
    }

    fn source_id() -> SourceId {
        let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{}",
            "f7".repeat(32)
        ))
        .expect("snapshot");
        InMemorySessionIdAllocator::new()
            .next_source_id(snapshot)
            .expect("source")
    }

    fn other_source_id() -> SourceId {
        let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{}",
            "f8".repeat(32)
        ))
        .expect("other snapshot");
        let allocator = InMemorySessionIdAllocator::new();
        let _ = allocator
            .next_source_id(snapshot)
            .expect("discarded first source");
        allocator.next_source_id(snapshot).expect("other source")
    }

    fn module() -> ModuleId {
        ModuleId::new(
            PackageId::new("pkg"),
            ModulePath::new("composition.fixture"),
        )
    }

    fn range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
        SourceRange {
            source_id,
            start,
            end,
        }
    }

    fn node(index: usize) -> TypedSiteRef {
        TypedSiteRef::Node(TypedNodeId::new(index))
    }

    fn base_bindings(source: SourceId, module: &ModuleId) -> BindingEnv {
        let mut contexts = BindingContextTable::new();
        contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::Module,
            parent: None,
            layer: BindingContextLayer::Module,
            lexical_scope: None,
            bindings: Vec::new(),
            visible_bindings: Vec::new(),
            recovery: BindingContextRecovery::Normal,
        });
        let mut diagnostics = BindingDiagnosticTable::new();
        for message_key in [
            "checker.binding.external.local_bindings",
            "checker.binding.external.use_site_scope",
            "checker.binding.external.reserve_payload",
            "checker.binding.external.closure_payload",
        ] {
            diagnostics.insert(BindingDiagnosticDraft {
                source_range: None,
                class: BindingDiagnosticClass::ExternalDependencyGap,
                severity: BindingDiagnosticSeverity::Note,
                message_key: message_key.to_owned(),
                recovery: BindingDiagnosticRecovery::Degraded,
            });
        }
        BindingEnv::try_new(BindingEnvParts {
            source_id: source,
            module_id: module.clone(),
            contexts,
            bindings: BindingTable::new(),
            diagnostics,
        })
        .expect("base bindings")
    }

    struct Task257c4aFixture {
        source: SourceId,
        module: ModuleId,
        structural: SourceTemplateFraenkelStructuralCompositionHandoff,
        resolver: FraenkelGeneratorVariableSourceCollection,
        typed_ast: TypedAst,
    }

    fn task257c4a_fixture() -> Task257c4aFixture {
        let source = source_id();
        let module = module();
        let ast = task257c4a_surface_ast(source);
        let resolved =
            SurfaceResolvedArena::lower(&ast, &module).expect("Task257C4A resolver arena");
        let templates = TemplateTypeParameterSourceCollector::new(&ast, &module, &resolved)
            .expect("Task257C4A template collector")
            .collect()
            .expect("Task257C4A template collection");
        let resolver = FraenkelGeneratorVariableSourceCollector::new(&ast, &module, &resolved)
            .expect("Task257C4A generator collector")
            .collect()
            .expect("Task257C4A generator collection");
        let typed_ast = task257c4a_typed_ast(&ast, module.clone(), &resolved);
        let template =
            SourceTemplateTypeParameterAssociationProducer::build(&templates, &typed_ast)
                .expect("Task257C4A template handoff");
        let structural = SourceTemplateFraenkelStructuralCompositionProducer::build(
            &template, &resolver, &typed_ast,
        )
        .expect("Task257C4A structural handoff");
        Task257c4aFixture {
            source,
            module,
            structural,
            resolver,
            typed_ast,
        }
    }

    struct Task257c4c3Fixture {
        source: SourceId,
        module: ModuleId,
        resolver: FraenkelGeneratorVariableSourceCollection,
        typed_ast: TypedAst,
    }

    fn task257c4c3_fixture() -> Task257c4c3Fixture {
        let source = source_id();
        let module = module();
        let ast = task257c4c3_surface_ast(source);
        let resolved = SurfaceResolvedArena::lower(&ast, &module).expect("Task257C4C3 resolver");
        let resolver = FraenkelGeneratorVariableSourceCollector::new(&ast, &module, &resolved)
            .expect("Task257C4C3 collector")
            .collect()
            .expect("Task257C4C3 collection");
        let typed_ast = task257c4a_typed_ast(&ast, module.clone(), &resolved);
        Task257c4c3Fixture {
            source,
            module,
            resolver,
            typed_ast,
        }
    }

    pub(crate) fn task257c4c3_handoff_for_test() -> SourceNestedFraenkelBinderUseHandoff {
        let fixture = task257c4c3_fixture();
        SourceNestedFraenkelBinderUseProducer::build(&fixture.resolver, &fixture.typed_ast)
            .expect("Task257C4C3 test dependency")
    }

    #[derive(Clone, Copy)]
    pub(crate) enum Task257c4c3HandoffCorruption {
        Source,
        Module,
        Summary,
        Row,
        RetainedResolver,
        RetainedTypedAst,
    }

    pub(crate) fn task257c4c3_corrupted_handoff_for_test(
        corruption: Task257c4c3HandoffCorruption,
    ) -> SourceNestedFraenkelBinderUseHandoff {
        let fixture = task257c4c3_fixture();
        let mut handoff =
            SourceNestedFraenkelBinderUseProducer::build(&fixture.resolver, &fixture.typed_ast)
                .expect("Task257C4C3 test dependency");
        match corruption {
            Task257c4c3HandoffCorruption::Source => handoff.source_id = other_source_id(),
            Task257c4c3HandoffCorruption::Module => {
                handoff.module_id =
                    ModuleId::new(PackageId::new("pkg"), ModulePath::new("composition.other"));
            }
            Task257c4c3HandoffCorruption::Summary => {
                handoff.resolver_summary = "stale".to_owned();
            }
            Task257c4c3HandoffCorruption::Row => handoff.binder_uses.rows.clear(),
            Task257c4c3HandoffCorruption::RetainedResolver => {
                handoff.dependencies.resolver =
                    task257c4a_empty_resolver(fixture.source, &fixture.module);
            }
            Task257c4c3HandoffCorruption::RetainedTypedAst => {
                handoff.dependencies.typed_ast = task257c4a_fixture().typed_ast;
            }
        }
        handoff
    }

    fn task257c4c3_type(
        builder: &mut syntax::SurfaceAstBuilder,
        source: SourceId,
        start: usize,
    ) -> syntax::SurfaceBuilderNodeId {
        let element = builder.add_token(
            syntax::SurfaceTokenKind::UserSymbol,
            "Element",
            range(source, start, start + 7),
        );
        let element_path = builder.add_node(
            syntax::SurfaceNodeKind::PathSegment,
            range(source, start, start + 7),
            vec![element],
        );
        let element_symbol = builder.add_node(
            syntax::SurfaceNodeKind::QualifiedSymbol,
            range(source, start, start + 7),
            vec![element_path],
        );
        let of = builder.add_token(
            syntax::SurfaceTokenKind::ReservedWord,
            "of",
            range(source, start + 8, start + 10),
        );
        let nat = builder.add_token(
            syntax::SurfaceTokenKind::UserSymbol,
            "NAT",
            range(source, start + 11, start + 14),
        );
        let nat_path = builder.add_node(
            syntax::SurfaceNodeKind::PathSegment,
            range(source, start + 11, start + 14),
            vec![nat],
        );
        let nat_symbol = builder.add_node(
            syntax::SurfaceNodeKind::QualifiedSymbol,
            range(source, start + 11, start + 14),
            vec![nat_path],
        );
        let nat_reference = builder.add_node(
            syntax::SurfaceNodeKind::TermReference,
            range(source, start + 11, start + 14),
            vec![nat_symbol],
        );
        let nat_term = builder.add_node(
            syntax::SurfaceNodeKind::TermExpression,
            range(source, start + 11, start + 14),
            vec![nat_reference],
        );
        let arguments = builder.add_node(
            syntax::SurfaceNodeKind::TypeArguments,
            range(source, start + 8, start + 14),
            vec![of, nat_term],
        );
        let head = builder.add_node(
            syntax::SurfaceNodeKind::TypeHead,
            range(source, start, start + 14),
            vec![element_symbol, arguments],
        );
        builder.add_node(
            syntax::SurfaceNodeKind::TypeExpression,
            range(source, start, start + 14),
            vec![head],
        )
    }

    fn task257c4c3_surface_ast(source: SourceId) -> syntax::SurfaceAst {
        let mut b = syntax::SurfaceAstBuilder::new(source);
        let definition = b.add_token(
            syntax::SurfaceTokenKind::ReservedWord,
            "definition",
            range(source, 39, 49),
        );
        let func = b.add_token(
            syntax::SurfaceTokenKind::ReservedWord,
            "func",
            range(source, 52, 56),
        );
        let outer_open = b.add_token(
            syntax::SurfaceTokenKind::ReservedSymbol,
            "{",
            range(source, 90, 91),
        );
        let inner_open = b.add_token(
            syntax::SurfaceTokenKind::ReservedSymbol,
            "{",
            range(source, 92, 93),
        );
        let mapper_identifier = b.add_token(
            syntax::SurfaceTokenKind::Identifier,
            "x",
            range(source, 94, 95),
        );
        let mapper_reference = b.add_node(
            syntax::SurfaceNodeKind::TermReference,
            range(source, 94, 95),
            vec![mapper_identifier],
        );
        let mapper = b.add_node(
            syntax::SurfaceNodeKind::TermExpression,
            range(source, 94, 95),
            vec![mapper_reference],
        );
        let inner_where = b.add_token(
            syntax::SurfaceTokenKind::ReservedWord,
            "where",
            range(source, 96, 101),
        );
        let inner_binder = b.add_token(
            syntax::SurfaceTokenKind::Identifier,
            "y",
            range(source, 102, 103),
        );
        let inner_is = b.add_token(
            syntax::SurfaceTokenKind::ReservedWord,
            "is",
            range(source, 104, 106),
        );
        let inner_type = task257c4c3_type(&mut b, source, 107);
        let inner_segment = b.add_node(
            syntax::SurfaceNodeKind::ComprehensionVariableSegment,
            range(source, 102, 121),
            vec![inner_binder, inner_is, inner_type],
        );
        let inner_close = b.add_token(
            syntax::SurfaceTokenKind::ReservedSymbol,
            "}",
            range(source, 122, 123),
        );
        let inner = b.add_node(
            syntax::SurfaceNodeKind::SetComprehension,
            range(source, 92, 123),
            vec![inner_open, mapper, inner_where, inner_segment, inner_close],
        );
        let outer_mapper = b.add_node(
            syntax::SurfaceNodeKind::TermExpression,
            range(source, 92, 123),
            vec![inner],
        );
        let outer_where = b.add_token(
            syntax::SurfaceTokenKind::ReservedWord,
            "where",
            range(source, 130, 135),
        );
        let outer_binder = b.add_token(
            syntax::SurfaceTokenKind::Identifier,
            "x",
            range(source, 136, 137),
        );
        let outer_is = b.add_token(
            syntax::SurfaceTokenKind::ReservedWord,
            "is",
            range(source, 138, 140),
        );
        let outer_type = task257c4c3_type(&mut b, source, 141);
        let outer_segment = b.add_node(
            syntax::SurfaceNodeKind::ComprehensionVariableSegment,
            range(source, 136, 155),
            vec![outer_binder, outer_is, outer_type],
        );
        let outer_close = b.add_token(
            syntax::SurfaceTokenKind::ReservedSymbol,
            "}",
            range(source, 156, 157),
        );
        let outer = b.add_node(
            syntax::SurfaceNodeKind::SetComprehension,
            range(source, 90, 157),
            vec![
                outer_open,
                outer_mapper,
                outer_where,
                outer_segment,
                outer_close,
            ],
        );
        let definiens_expression = b.add_node(
            syntax::SurfaceNodeKind::TermExpression,
            range(source, 90, 157),
            vec![outer],
        );
        let definiens = b.add_node(
            syntax::SurfaceNodeKind::TermDefiniens,
            range(source, 90, 157),
            vec![definiens_expression],
        );
        let functor = b.add_node(
            syntax::SurfaceNodeKind::FunctorDefinition,
            range(source, 52, 158),
            vec![func, definiens],
        );
        let block = b.add_node(
            syntax::SurfaceNodeKind::DefinitionBlockItem,
            range(source, 39, 164),
            vec![definition, functor],
        );
        let root = b.add_node(
            syntax::SurfaceNodeKind::Root,
            range(source, 0, 164),
            vec![block],
        );
        b.finish(Some(root), None)
    }

    #[test]
    fn task257c4c3_builds_exact_nested_binder_use_handoff() {
        let fixture = task257c4c3_fixture();
        let handoff =
            SourceNestedFraenkelBinderUseProducer::build(&fixture.resolver, &fixture.typed_ast)
                .unwrap();
        assert_eq!(handoff.source_id(), fixture.source);
        assert_eq!(handoff.module_id(), &fixture.module);
        assert_eq!(
            handoff.resolver_summary(),
            "fraenkel-generator-variable-source-v1|module=pkg.composition.fixture|bindings=2|uses=1"
        );
        assert_eq!(
            handoff.debug_text(),
            "source-nested-fraenkel-binder-use-v1|module=pkg.composition.fixture|binder-uses=1"
        );
        let row = handoff
            .binder_uses()
            .get(SourceNestedFraenkelBinderUseId::new(0))
            .unwrap();
        assert_eq!(row.resolver_use_index(), 0);
        assert_eq!(row.resolver_binding().index(), 1);
        assert_eq!(
            typed_range(&fixture.typed_ast, row.outer_binder()),
            Some(range(fixture.source, 136, 137))
        );
        assert_eq!(
            typed_range(&fixture.typed_ast, row.inner_mapper_use()),
            Some(range(fixture.source, 94, 95))
        );
        assert_eq!(row.source_ordinal(), 0);
        assert_eq!(
            handoff
                .binder_uses()
                .iter()
                .map(|(id, value)| (id.index(), value.resolver_use_index()))
                .collect::<Vec<_>>(),
            vec![(0, 0)]
        );
        assert!(
            handoff
                .binder_uses()
                .get(SourceNestedFraenkelBinderUseId::new(1))
                .is_none()
        );
    }

    #[test]
    fn task257c4c3_rejects_environment_resolver_and_typed_dependency_corruption() {
        let fixture = task257c4c3_fixture();
        let handoff =
            SourceNestedFraenkelBinderUseProducer::build(&fixture.resolver, &fixture.typed_ast)
                .unwrap();
        let mut environment = handoff.clone();
        environment.source_id = other_source_id();
        assert!(matches!(
            environment.validate(),
            Err(SourceNestedFraenkelBinderUseError::EnvironmentMismatch)
        ));
        let mut resolver = handoff.clone();
        resolver.resolver_summary = "stale".to_owned();
        assert!(matches!(
            resolver.validate(),
            Err(SourceNestedFraenkelBinderUseError::InvalidResolverDependency)
        ));
        let mut version = handoff.clone();
        version.dependencies.version = "stale";
        assert!(matches!(
            version.validate(),
            Err(SourceNestedFraenkelBinderUseError::InvalidResolverDependency)
        ));
        let mut domain = handoff.clone();
        domain.dependencies.domain = "stale";
        assert!(matches!(
            domain.validate(),
            Err(SourceNestedFraenkelBinderUseError::InvalidResolverDependency)
        ));
        let mut retained_empty_resolver = handoff.clone();
        retained_empty_resolver.dependencies.resolver =
            task257c4a_empty_resolver(fixture.source, &fixture.module);
        assert!(matches!(
            retained_empty_resolver.validate(),
            Err(SourceNestedFraenkelBinderUseError::InvalidResolverDependency)
        ));
        let mut retained_f5_resolver = handoff.clone();
        retained_f5_resolver.dependencies.resolver = task257c4a_fixture().resolver;
        assert!(matches!(
            retained_f5_resolver.validate(),
            Err(SourceNestedFraenkelBinderUseError::InvalidResolverDependency)
        ));
        let mut typed = handoff.clone();
        let ast = task257c4a_surface_ast(fixture.source);
        let resolved = SurfaceResolvedArena::lower(&ast, &fixture.module).unwrap();
        typed.dependencies.typed_ast =
            task257c4a_typed_ast(&ast, fixture.module.clone(), &resolved);
        assert!(matches!(
            typed.validate(),
            Err(SourceNestedFraenkelBinderUseError::InvalidTypedDependency)
        ));
        let profile = validate_nested_fraenkel_resolver(&fixture.resolver).unwrap();
        let mapper = typed_for_resolved(&fixture.typed_ast, profile.mapper_identifier).unwrap();
        let reference = typed_for_resolved(&fixture.typed_ast, profile.mapper_reference).unwrap();
        let make_typed = |mut nodes: Vec<TypedNode>| {
            task257c4a_typed_from_nodes(
                fixture.source,
                fixture.module.clone(),
                fixture.typed_ast.nodes().root(),
                std::mem::take(&mut nodes),
            )
        };
        let original = fixture
            .typed_ast
            .nodes()
            .iter()
            .map(|(_, node)| node.clone())
            .collect::<Vec<_>>();
        let mut recovered = original.clone();
        recovered[mapper.index()].recovery = NodeRecoveryState::Recovered;
        let mut wrong_kind = original.clone();
        wrong_kind[mapper.index()].kind = "TermReference".into();
        let mut wrong_range = original.clone();
        wrong_range[mapper.index()].anchor = SourceAnchor::Range(range(fixture.source, 94, 96));
        let mut detached = original.clone();
        detached[reference.index()].children.clear();
        let type_expression = fixture
            .typed_ast
            .nodes()
            .iter()
            .find_map(|(id, node)| {
                (node.kind.as_str() == "TypeExpression"
                    && typed_range(&fixture.typed_ast, id) == Some(range(fixture.source, 107, 121)))
                .then_some(id)
            })
            .unwrap();
        let mut broken_type = original.clone();
        broken_type[type_expression.index()].kind = "TermExpression".into();
        let element_token = fixture
            .typed_ast
            .nodes()
            .iter()
            .find_map(|(id, node)| {
                (node.kind.as_str()
                    == r#"Token(SurfaceToken { kind: UserSymbol, text: "Element" })"#
                    && typed_range(&fixture.typed_ast, id) == Some(range(fixture.source, 107, 114)))
                .then_some(id)
            })
            .unwrap();
        let functor = typed_for_resolved(&fixture.typed_ast, profile.functor_definition).unwrap();
        let mut replaced_token = original.clone();
        replaced_token[element_token.index()].kind =
            r#"Token(SurfaceToken { kind: UserSymbol, text: "Other" })"#.into();
        let mut extra_child = original.clone();
        extra_child[reference.index()]
            .children
            .push(TypedNodeId::new(0));
        let mut detached_functor = original.clone();
        detached_functor[functor.index()].children.pop();
        let definition = typed_for_resolved(&fixture.typed_ast, profile.definition_block).unwrap();
        let root = fixture.typed_ast.nodes().root().unwrap();
        let without_root = task257c4a_typed_from_nodes(
            fixture.source,
            fixture.module.clone(),
            None,
            original.clone(),
        );
        let wrong_root = task257c4a_typed_from_nodes(
            fixture.source,
            fixture.module.clone(),
            Some(definition),
            original.clone(),
        );
        let mut detached_definition_nodes = original.clone();
        detached_definition_nodes[root.index()].children.clear();
        let detached_definition = task257c4a_typed_from_nodes(
            fixture.source,
            fixture.module.clone(),
            Some(root),
            detached_definition_nodes,
        );
        for corrupted in [without_root, wrong_root, detached_definition] {
            let mut value = handoff.clone();
            value.dependencies.typed_ast = corrupted;
            assert!(matches!(
                value.validate(),
                Err(SourceNestedFraenkelBinderUseError::InvalidTypedDependency)
            ));
        }
        let mut duplicate = original;
        duplicate[0].resolved_node = duplicate[mapper.index()].resolved_node;
        for corrupted in [
            recovered,
            wrong_kind,
            wrong_range,
            detached,
            broken_type,
            replaced_token,
            extra_child,
            detached_functor,
            duplicate,
        ] {
            let mut value = handoff.clone();
            value.dependencies.typed_ast = make_typed(corrupted);
            assert!(matches!(
                value.validate(),
                Err(SourceNestedFraenkelBinderUseError::InvalidTypedDependency)
            ));
        }
        assert!(!exact_nested_resolver_range(
            range(other_source_id(), 102, 121),
            fixture.source,
            102,
            121
        ));
        let other = task257c4a_fixture();
        assert!(matches!(
            SourceNestedFraenkelBinderUseProducer::build(&fixture.resolver, &other.typed_ast),
            Err(SourceNestedFraenkelBinderUseError::InvalidTypedDependency)
        ));
        let empty = task257c4a_empty_resolver(fixture.source, &fixture.module);
        assert!(matches!(
            SourceNestedFraenkelBinderUseProducer::build(&empty, &fixture.typed_ast),
            Err(SourceNestedFraenkelBinderUseError::InvalidResolverDependency)
        ));
        let wrong_module =
            ModuleId::new(PackageId::new("pkg"), ModulePath::new("composition.other"));
        let module_mismatch = task257c4a_typed_from_nodes(
            fixture.source,
            wrong_module,
            fixture.typed_ast.nodes().root(),
            fixture
                .typed_ast
                .nodes()
                .iter()
                .map(|(_, node)| node.clone())
                .collect(),
        );
        assert!(matches!(
            SourceNestedFraenkelBinderUseProducer::build(&fixture.resolver, &module_mismatch),
            Err(SourceNestedFraenkelBinderUseError::EnvironmentMismatch)
        ));
        let mut resolver_precedence = handoff.clone();
        resolver_precedence.resolver_summary = "stale".to_owned();
        resolver_precedence.binder_uses.rows.clear();
        resolver_precedence.dependencies.typed_ast =
            task257c4a_typed_ast(&ast, fixture.module.clone(), &resolved);
        assert!(matches!(
            resolver_precedence.validate(),
            Err(SourceNestedFraenkelBinderUseError::InvalidResolverDependency)
        ));
        let mut typed_precedence = handoff.clone();
        typed_precedence.binder_uses.rows.clear();
        typed_precedence.dependencies.typed_ast =
            task257c4a_typed_ast(&ast, fixture.module.clone(), &resolved);
        assert!(matches!(
            typed_precedence.validate(),
            Err(SourceNestedFraenkelBinderUseError::InvalidTypedDependency)
        ));
        let mut precedence = handoff;
        precedence.source_id = other_source_id();
        precedence.resolver_summary = "stale".to_owned();
        precedence.binder_uses.rows.clear();
        assert!(matches!(
            precedence.validate(),
            Err(SourceNestedFraenkelBinderUseError::EnvironmentMismatch)
        ));
    }

    #[test]
    fn task257c4c3_rejects_row_cardinality_order_and_site_corruption() {
        let fixture = task257c4c3_fixture();
        let handoff =
            SourceNestedFraenkelBinderUseProducer::build(&fixture.resolver, &fixture.typed_ast)
                .unwrap();
        let mut missing = handoff.clone();
        missing.binder_uses.rows.clear();
        let mut extra = handoff.clone();
        extra
            .binder_uses
            .rows
            .push(extra.binder_uses.rows[0].clone());
        let mut order = handoff.clone();
        order.binder_uses.rows[0].resolver_use_index = 1;
        let mut site = handoff.clone();
        site.binder_uses.rows[0].outer_binder = TypedNodeId::new(0);
        let mut binding = handoff.clone();
        binding.binder_uses.rows[0].resolver_binding = FraenkelGeneratorVariableBindingId::new(0);
        let mut mapper = handoff.clone();
        mapper.binder_uses.rows[0].inner_mapper_use = TypedNodeId::new(0);
        let mut ordinal = handoff.clone();
        ordinal.binder_uses.rows[0].source_ordinal = 1;
        for corrupted in [missing, extra, order, site, binding, mapper, ordinal] {
            assert!(
                matches!(corrupted.validate(), Err(SourceNestedFraenkelBinderUseError::InvalidBinderUse { binder_use }) if binder_use.index() == 0)
            );
        }
    }

    #[test]
    fn task257c4c3_replays_deterministically_and_rejects_f5_profiles() {
        let fixture = task257c4c3_fixture();
        let first =
            SourceNestedFraenkelBinderUseProducer::build(&fixture.resolver, &fixture.typed_ast);
        let second =
            SourceNestedFraenkelBinderUseProducer::build(&fixture.resolver, &fixture.typed_ast);
        assert!(
            matches!((&first, &second), (Ok(left), Ok(right)) if left == right && left.debug_text() == right.debug_text())
        );
        let f5 = task257c4a_fixture();
        assert!(matches!(
            SourceNestedFraenkelBinderUseProducer::build(&f5.resolver, &f5.typed_ast),
            Err(SourceNestedFraenkelBinderUseError::InvalidResolverDependency)
        ));
    }

    fn task257c4c5_dependency() -> SourceNestedFraenkelMapperPrimaryHandoff {
        SourceNestedFraenkelMapperPrimaryProducer::build(task257c4c3_handoff_for_test())
            .expect("Task257C4C4 test dependency")
    }

    fn task257c4c6_handoff() -> SourceNestedFraenkelCaptureIdentityHandoff {
        SourceNestedFraenkelCaptureIdentityProducer::build(task257c4c5_dependency())
            .expect("Task257C4C5 test dependency")
    }

    fn task257c4c6_typed_and_handoff() -> (TypedAst, SourceNestedFraenkelCaptureIdentityHandoff) {
        let fixture = task257c4c3_fixture();
        let handoff = SourceNestedFraenkelCaptureIdentityProducer::build(
            SourceNestedFraenkelMapperPrimaryProducer::build(
                SourceNestedFraenkelBinderUseProducer::build(&fixture.resolver, &fixture.typed_ast)
                    .expect("C4C3 test dependency"),
            )
            .expect("C4C4 test dependency"),
        )
        .expect("C4C5 test dependency");
        (fixture.typed_ast, handoff)
    }

    fn task257c4c6_assemble_resolved(
        typed_ast: &TypedAst,
    ) -> Result<ResolvedTypedAst, ResolvedTypedAstError> {
        task257c4c6_assemble_resolved_with_hints(typed_ast, Vec::new())
    }

    fn task257c4c6_assemble_resolved_with_hints(
        typed_ast: &TypedAst,
        node_hints: Vec<ResolvedNodeKindHint>,
    ) -> Result<ResolvedTypedAst, ResolvedTypedAstError> {
        let cluster_facts = ClusterFactTable::new();
        let overload_collection = OverloadCollectionOutput::collect(
            Vec::<OverloadSiteInput>::new(),
            Vec::<OverloadCandidateInput>::new(),
        );
        let template_expansion = TemplateExpansionOutput::expand(&overload_collection);
        let viability = CandidateViabilityOutput::filter(
            &template_expansion,
            Vec::<CandidateViabilityInput>::new(),
        );
        let specificity =
            SpecificityGraphOutput::build(&viability, Vec::<SpecificityComparisonInput>::new());
        let overload_selection = OverloadSelectionOutput::resolve(
            &specificity,
            Vec::<OverloadSiteResolutionInput>::new(),
        );
        ResolvedTypedAst::assemble(ResolvedTypedAstInputs {
            typed_ast,
            cluster_facts: &cluster_facts,
            overload_collection: &overload_collection,
            template_expansion: &template_expansion,
            viability: &viability,
            specificity: &specificity,
            overload_selection: &overload_selection,
            expressions: Vec::new(),
            node_hints,
            statement_semantics: None,
            statement_proofs: None,
        })
    }

    fn task257c4c6_assert_captured_empty(handoff: &SourceNestedFraenkelCaptureIdentityHandoff) {
        assert!(
            handoff
                .dependency()
                .binding_env()
                .bindings()
                .get(BindingId::new(0))
                .is_some_and(|binding| binding.captured.identities().is_empty())
        );
    }

    fn task257c4c6_assert_resolved_empty(resolved: &ResolvedTypedAst) {
        assert!(resolved.expr_metadata().is_empty());
        assert!(resolved.collection_candidates().is_empty());
        assert!(resolved.expanded_candidates().is_empty());
        assert!(resolved.template_expansions().is_empty());
        assert!(resolved.viable_candidates().is_empty());
        assert!(resolved.viability_decisions().is_empty());
        assert!(resolved.specificity_graphs().is_empty());
        assert!(resolved.resolved_overloads().is_empty());
        assert!(resolved.inserted_coercions().is_empty());
        assert!(resolved.cluster_facts().is_empty());
        assert!(resolved.diagnostics().is_empty());
        assert!(resolved.checked_formulas().is_empty());
        assert!(resolved.statement_semantics().is_empty());
        assert!(resolved.checked_proofs().is_empty());
        assert!(resolved.checked_proof_nodes().is_empty());
        assert!(resolved.checked_terminal_goals().is_empty());
    }

    #[test]
    fn task257c4c6_installation_authenticates_exact_typed_snapshot() {
        let (typed_ast, handoff) = task257c4c6_typed_and_handoff();
        task257c4c6_assert_captured_empty(&handoff);
        let installed = typed_ast
            .with_source_nested_fraenkel_capture_identity(handoff.clone())
            .expect("C4C6 exact typed snapshot");
        assert!(
            installed
                .source_nested_fraenkel_capture_identity()
                .is_some_and(|actual| actual == &handoff)
        );
        let second = installed
            .clone()
            .with_source_nested_fraenkel_capture_identity(handoff)
            .expect_err("C4C6 is one-shot");
        assert_eq!(
            second,
            TypedAstError::InvalidSourceNestedFraenkelCaptureIdentity
        );
        assert!(
            installed
                .source_nested_fraenkel_capture_identity()
                .is_some()
        );
        task257c4c6_assert_captured_empty(
            installed
                .source_nested_fraenkel_capture_identity()
                .expect("C4C6 retained receipt"),
        );
    }

    #[test]
    fn task257c4c6_typed_installation_is_boxed_one_shot_and_debug_stable() {
        let (typed_ast, handoff) = task257c4c6_typed_and_handoff();
        task257c4c6_assert_captured_empty(&handoff);
        let absent_debug = typed_ast.debug_text();
        assert!(!absent_debug.contains(&handoff.debug_text()));
        let installed = typed_ast
            .with_source_nested_fraenkel_capture_identity(handoff.clone())
            .unwrap();
        let insertion = absent_debug.find("nodes:\n").unwrap();
        let expected = format!(
            "{}{}\n{}",
            &absent_debug[..insertion],
            handoff.debug_text(),
            &absent_debug[insertion..]
        );
        assert_eq!(installed.debug_text(), expected);
        assert_eq!(
            installed
                .debug_text()
                .matches(&handoff.debug_text())
                .count(),
            1
        );
        assert!(format!("{installed:?}").contains("InstalledSourceNestedFraenkelCaptureIdentity"));
        assert_eq!(installed, installed.clone());
        task257c4c6_assert_captured_empty(
            installed
                .clone()
                .source_nested_fraenkel_capture_identity()
                .expect("C4C6 cloned receipt"),
        );
    }

    #[test]
    fn task257c4c6_rejects_dependency_row_and_final_snapshot_corruption() {
        let (typed_ast, handoff) = task257c4c6_typed_and_handoff();
        let mut row_corrupt = handoff.clone();
        row_corrupt.identities.rows.clear();
        task257c4c6_assert_captured_empty(&row_corrupt);
        assert!(matches!(
            typed_ast
                .clone()
                .with_source_nested_fraenkel_capture_identity(row_corrupt),
            Err(TypedAstError::InvalidSourceNestedFraenkelCaptureIdentity)
        ));

        let mut snapshot_corrupt = typed_ast.clone();
        snapshot_corrupt
            .occupy_statement_transport_table_for_test(StatementTransportTableForTest::Context);
        let snapshot_handoff = handoff.clone();
        assert!(matches!(
            snapshot_corrupt.with_source_nested_fraenkel_capture_identity(snapshot_handoff.clone()),
            Err(TypedAstError::InvalidSourceNestedFraenkelCaptureIdentity)
        ));
        task257c4c6_assert_captured_empty(&snapshot_handoff);
    }

    #[test]
    fn task257c4c6_reciprocal_installation_exclusion_is_atomic() {
        let fixture = task257c4c3_fixture();
        let handoff = SourceNestedFraenkelCaptureIdentityProducer::build(
            SourceNestedFraenkelMapperPrimaryProducer::build(
                SourceNestedFraenkelBinderUseProducer::build(&fixture.resolver, &fixture.typed_ast)
                    .unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        task257c4c6_assert_captured_empty(&handoff);

        let mut injected = fixture.typed_ast.clone();
        injected.inject_source_nested_fraenkel_capture_identity_for_test(handoff.clone());
        assert!(matches!(
            SourceNestedFraenkelBinderUseProducer::build(&fixture.resolver, &injected),
            Err(SourceNestedFraenkelBinderUseError::InvalidTypedDependency)
        ));
        assert!(
            injected
                .source_nested_fraenkel_capture_identity()
                .is_some_and(|actual| actual == &handoff)
        );
        task257c4c6_assert_captured_empty(
            injected
                .source_nested_fraenkel_capture_identity()
                .expect("test-only pre-C4C3 receipt remains installed"),
        );

        for table in [
            StatementTransportTableForTest::Context,
            StatementTransportTableForTest::Type,
            StatementTransportTableForTest::Fact,
            StatementTransportTableForTest::Coercion,
            StatementTransportTableForTest::InitialObligation,
            StatementTransportTableForTest::Diagnostic,
        ] {
            let mut populated = fixture.typed_ast.clone();
            populated.occupy_statement_transport_table_for_test(table);
            assert!(matches!(
                SourceNestedFraenkelBinderUseProducer::build(&fixture.resolver, &populated),
                Err(SourceNestedFraenkelBinderUseError::InvalidTypedDependency)
            ));
            assert!(
                populated
                    .source_nested_fraenkel_capture_identity()
                    .is_none()
            );
        }

        let installed = fixture
            .typed_ast
            .clone()
            .with_source_nested_fraenkel_capture_identity(handoff.clone())
            .unwrap();
        let term = handoff.dependency().source_term().clone();
        assert!(matches!(
            installed.clone().with_source_term(term),
            Err(TypedAstError::InvalidSourceTerm)
        ));
        assert!(
            installed
                .source_nested_fraenkel_capture_identity()
                .is_some()
        );
        task257c4c6_assert_captured_empty(
            installed
                .source_nested_fraenkel_capture_identity()
                .expect("C4C6 typed receipt after reciprocal rejection"),
        );

        let c4c3 = SourceNestedFraenkelBinderUseProducer::build(&fixture.resolver, &installed);
        assert!(matches!(
            c4c3,
            Err(SourceNestedFraenkelBinderUseError::InvalidTypedDependency)
        ));
        assert!(
            installed
                .source_nested_fraenkel_capture_identity()
                .is_some()
        );
        task257c4c6_assert_captured_empty(
            installed
                .source_nested_fraenkel_capture_identity()
                .expect("C4C6 receipt remains after C4C3 rejection"),
        );
    }

    #[test]
    fn task257c4c6_resolved_clone_revalidates_and_preserves_receipt() {
        let (typed_ast, handoff) = task257c4c6_typed_and_handoff();
        task257c4c6_assert_captured_empty(&handoff);
        let typed_ast = typed_ast
            .with_source_nested_fraenkel_capture_identity(handoff.clone())
            .unwrap();
        task257c4c6_assert_captured_empty(
            typed_ast
                .source_nested_fraenkel_capture_identity()
                .expect("C4C6 typed receipt"),
        );
        let resolved = task257c4c6_assemble_resolved(&typed_ast).unwrap();
        assert!(
            resolved
                .source_nested_fraenkel_capture_identity()
                .is_some_and(|actual| actual == &handoff)
        );
        task257c4c6_assert_captured_empty(
            resolved
                .source_nested_fraenkel_capture_identity()
                .expect("C4C6 resolved receipt"),
        );
        task257c4c6_assert_resolved_empty(&resolved);
        let resolved_clone = resolved.clone();
        assert_eq!(resolved, resolved_clone);
        assert_eq!(resolved.debug_text(), resolved_clone.debug_text());
        task257c4c6_assert_captured_empty(
            resolved_clone
                .source_nested_fraenkel_capture_identity()
                .expect("C4C6 cloned resolved receipt"),
        );
        let replay = task257c4c6_assemble_resolved(&typed_ast).unwrap();
        assert_eq!(resolved, replay);
        task257c4c6_assert_captured_empty(
            replay
                .source_nested_fraenkel_capture_identity()
                .expect("C4C6 replayed resolved receipt"),
        );
        task257c4c6_assert_resolved_empty(&replay);
    }

    #[test]
    fn task257c4c6_resolved_rejects_injected_stale_or_mismatched_receipt() {
        let (typed_ast, handoff) = task257c4c6_typed_and_handoff();
        task257c4c6_assert_captured_empty(&handoff);
        let installed = typed_ast
            .clone()
            .with_source_nested_fraenkel_capture_identity(handoff.clone())
            .unwrap();
        task257c4c6_assert_captured_empty(
            installed
                .source_nested_fraenkel_capture_identity()
                .expect("C4C6 installed receipt"),
        );
        let mut stale = installed.clone();
        let mut stale_handoff = task257c4c6_handoff();
        stale_handoff.identities.rows[0].source_ordinal = 1;
        task257c4c6_assert_captured_empty(&stale_handoff);
        stale.inject_source_nested_fraenkel_capture_identity_for_test(stale_handoff);
        assert!(matches!(
            task257c4c6_assemble_resolved(&stale),
            Err(ResolvedTypedAstError::InvalidSourceNestedFraenkelCaptureIdentity)
        ));
        task257c4c6_assert_captured_empty(
            stale
                .source_nested_fraenkel_capture_identity()
                .expect("stale receipt remains installed after rejection"),
        );

        let mut mismatched = installed;
        let mut corrupted = handoff;
        corrupted.source_id = other_source_id();
        task257c4c6_assert_captured_empty(&corrupted);
        mismatched.inject_source_nested_fraenkel_capture_identity_for_test(corrupted);
        assert!(matches!(
            task257c4c6_assemble_resolved(&mismatched),
            Err(ResolvedTypedAstError::InvalidSourceNestedFraenkelCaptureIdentity)
        ));
        task257c4c6_assert_captured_empty(
            mismatched
                .source_nested_fraenkel_capture_identity()
                .expect("mismatched receipt remains installed after rejection"),
        );

        let external_input = typed_ast
            .with_source_nested_fraenkel_capture_identity(task257c4c6_handoff())
            .unwrap();
        let external_handoff = external_input
            .source_nested_fraenkel_capture_identity()
            .expect("external-input fixture receipt")
            .clone();
        task257c4c6_assert_captured_empty(&external_handoff);
        assert!(matches!(
            task257c4c6_assemble_resolved_with_hints(
                &external_input,
                vec![ResolvedNodeKindHint {
                    typed_node: TypedNodeId::new(0),
                    kind: ResolvedNodeKindHintKind::SourcePreserved {
                        role: SourceNodeRole::new("task257c4c6.external-input"),
                    },
                }],
            ),
            Err(ResolvedTypedAstError::InvalidSourceNestedFraenkelCaptureIdentity)
        ));
        task257c4c6_assert_captured_empty(
            external_input
                .source_nested_fraenkel_capture_identity()
                .expect("external-input receipt remains installed after rejection"),
        );
    }

    #[test]
    fn task257c4c5_builds_exact_capture_identity_handoff() {
        let dependency = task257c4c5_dependency();
        let expected_dependency = dependency.debug_text();
        let source = dependency.source_id();
        let module = dependency.module_id().clone();
        let handoff = SourceNestedFraenkelCaptureIdentityProducer::build(dependency).unwrap();

        assert_eq!(handoff.source_id(), source);
        assert_eq!(handoff.module_id(), &module);
        assert_eq!(handoff.dependency_fingerprint(), expected_dependency);
        assert_eq!(handoff.identities().len(), 1);
        assert!(!handoff.identities().is_empty());
        assert!(
            handoff
                .identities()
                .get(SourceNestedFraenkelCaptureIdentityId::new(1))
                .is_none()
        );
        let rows = handoff.identities().iter().collect::<Vec<_>>();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].0, SourceNestedFraenkelCaptureIdentityId::new(0));
        let identity = rows[0].1;
        assert_eq!(identity.owner_context(), BindingContextId::new(2));
        assert_eq!(
            identity.owner_range(),
            SourceRange {
                source_id: source,
                start: 92,
                end: 123,
            }
        );
        assert_eq!(identity.mapper_term(), SourcePrimaryTermId::new(0));
        assert_eq!(
            identity.mapper_reference(),
            SourcePrimaryTermReferenceId::new(0)
        );
        assert_eq!(identity.projected_binding(), BindingId::new(0));
        assert_eq!(identity.resolver_use_index(), 0);
        assert_eq!(
            identity.resolver_binding(),
            FraenkelGeneratorVariableBindingId::new(1)
        );
        assert_eq!(identity.source_ordinal(), 0);
        assert_eq!(
            handoff.debug_text(),
            format!(
                "source-nested-fraenkel-capture-identity-v1|module={}.{}|identities=1|dependency-fingerprint={expected_dependency:?}",
                module.package().as_str(),
                module.path().as_str(),
            )
        );
        assert!(handoff.validate_complete().is_ok());
    }

    #[test]
    fn task257c4c5_rejects_dependency_owner_and_precedence_corruption() {
        let handoff =
            SourceNestedFraenkelCaptureIdentityProducer::build(task257c4c5_dependency()).unwrap();

        let mut source = handoff.clone();
        source.source_id = other_source_id();
        assert_eq!(
            source.validate_complete().unwrap_err().to_string(),
            "nested Fraenkel capture-identity dependency is invalid"
        );
        assert!(matches!(
            source.validate_complete(),
            Err(SourceNestedFraenkelCaptureIdentityError::InvalidDependency)
        ));

        let mut module = handoff.clone();
        module.module_id =
            ModuleId::new(PackageId::new("pkg"), ModulePath::new("composition.other"));
        assert!(matches!(
            module.validate_complete(),
            Err(SourceNestedFraenkelCaptureIdentityError::InvalidDependency)
        ));

        let mut fingerprint = handoff.clone();
        fingerprint.dependency_fingerprint = "stale".to_owned();
        assert!(matches!(
            fingerprint.validate_complete(),
            Err(SourceNestedFraenkelCaptureIdentityError::InvalidDependency)
        ));

        let mut owner_context = handoff.clone();
        owner_context.identities.rows[0].owner_context = BindingContextId::new(1);
        let mut owner_range = handoff.clone();
        owner_range.identities.rows[0].owner_range.start = 91;
        assert_eq!(
            owner_context.validate_complete().unwrap_err().to_string(),
            "nested Fraenkel capture identity 0 is invalid"
        );
        for corrupted in [owner_context, owner_range] {
            assert!(matches!(
                corrupted.validate_complete(),
                Err(SourceNestedFraenkelCaptureIdentityError::InvalidCaptureIdentity { capture_identity })
                    if capture_identity == SourceNestedFraenkelCaptureIdentityId::new(0)
            ));
        }

        let mut precedence = handoff;
        precedence.source_id = other_source_id();
        precedence.identities.rows.clear();
        assert!(matches!(
            precedence.validate_complete(),
            Err(SourceNestedFraenkelCaptureIdentityError::InvalidDependency)
        ));
    }

    #[test]
    fn task257c4c5_rejects_identity_cardinality_order_and_field_corruption() {
        let handoff =
            SourceNestedFraenkelCaptureIdentityProducer::build(task257c4c5_dependency()).unwrap();
        let mut missing = handoff.clone();
        missing.identities.rows.clear();
        let mut extra = handoff.clone();
        extra.identities.rows.push(extra.identities.rows[0].clone());
        let mut mapper_term = handoff.clone();
        mapper_term.identities.rows[0].mapper_term = SourcePrimaryTermId::new(1);
        let mut mapper_reference = handoff.clone();
        mapper_reference.identities.rows[0].mapper_reference = SourcePrimaryTermReferenceId::new(1);
        let mut projected_binding = handoff.clone();
        projected_binding.identities.rows[0].projected_binding = BindingId::new(1);
        let mut resolver_use = handoff.clone();
        resolver_use.identities.rows[0].resolver_use_index = 1;
        let mut resolver_binding = handoff.clone();
        resolver_binding.identities.rows[0].resolver_binding =
            FraenkelGeneratorVariableBindingId::new(0);
        let mut source_ordinal = handoff;
        source_ordinal.identities.rows[0].source_ordinal = 1;

        for corrupted in [
            missing,
            extra,
            mapper_term,
            mapper_reference,
            projected_binding,
            resolver_use,
            resolver_binding,
            source_ordinal,
        ] {
            assert!(matches!(
                corrupted.validate_complete(),
                Err(SourceNestedFraenkelCaptureIdentityError::InvalidCaptureIdentity { capture_identity })
                    if capture_identity == SourceNestedFraenkelCaptureIdentityId::new(0)
            ));
        }
    }

    #[test]
    fn task257c4c5_replays_deterministically_and_preserves_empty_capture_and_installation() {
        let dependency = task257c4c5_dependency();
        let dependency_before = dependency.clone();
        let first = SourceNestedFraenkelCaptureIdentityProducer::build(dependency).unwrap();
        let second =
            SourceNestedFraenkelCaptureIdentityProducer::build(task257c4c5_dependency()).unwrap();

        assert!(first == second);
        assert_eq!(first.debug_text(), second.debug_text());
        assert!(first.dependency() == &dependency_before);
        assert_eq!(first.dependency().projection_arena().len(), 1);
        assert_eq!(first.dependency().source_term().terms().len(), 1);
        assert_eq!(first.dependency().source_term().references().len(), 1);
        assert!(
            first
                .dependency()
                .source_term()
                .numeric_type_requests()
                .is_empty()
        );
        assert!(
            first
                .dependency()
                .binding_env()
                .bindings()
                .get(BindingId::new(0))
                .expect("retained outer-x binding")
                .captured
                .identities()
                .is_empty()
        );
        assert!(first.validate_complete().is_ok());
    }

    fn task257c4a_surface_ast(source: SourceId) -> syntax::SurfaceAst {
        let mut builder = syntax::SurfaceAstBuilder::new(source);
        let definition = builder.add_token(
            syntax::SurfaceTokenKind::ReservedWord,
            "definition",
            range(source, 600, 610),
        );
        let let_keyword = builder.add_token(
            syntax::SurfaceTokenKind::ReservedWord,
            "let",
            range(source, 611, 614),
        );
        let template_binder = builder.add_token(
            syntax::SurfaceTokenKind::Identifier,
            "T",
            range(source, 615, 616),
        );
        let be_keyword = builder.add_token(
            syntax::SurfaceTokenKind::ReservedWord,
            "be",
            range(source, 617, 619),
        );
        let type_keyword = builder.add_token(
            syntax::SurfaceTokenKind::ReservedWord,
            "type",
            range(source, 620, 624),
        );
        let semicolon = builder.add_token(
            syntax::SurfaceTokenKind::ReservedSymbol,
            ";",
            range(source, 624, 625),
        );
        let parameter = builder.add_node(
            syntax::SurfaceNodeKind::TemplateParameter,
            range(source, 611, 625),
            vec![
                let_keyword,
                template_binder,
                be_keyword,
                type_keyword,
                semicolon,
            ],
        );
        let func = builder.add_token(
            syntax::SurfaceTokenKind::ReservedWord,
            "func",
            range(source, 630, 634),
        );
        let open = builder.add_token(
            syntax::SurfaceTokenKind::ReservedSymbol,
            "{",
            range(source, 663, 664),
        );
        let mapper_identifier = builder.add_token(
            syntax::SurfaceTokenKind::Identifier,
            "x",
            range(source, 665, 666),
        );
        let mapper_reference = builder.add_node(
            syntax::SurfaceNodeKind::TermReference,
            range(source, 665, 666),
            vec![mapper_identifier],
        );
        let mapper = builder.add_node(
            syntax::SurfaceNodeKind::TermExpression,
            range(source, 665, 666),
            vec![mapper_reference],
        );
        let where_keyword = builder.add_token(
            syntax::SurfaceTokenKind::ReservedWord,
            "where",
            range(source, 667, 672),
        );
        let generator_binder = builder.add_token(
            syntax::SurfaceTokenKind::Identifier,
            "x",
            range(source, 673, 674),
        );
        let is_keyword = builder.add_token(
            syntax::SurfaceTokenKind::ReservedWord,
            "is",
            range(source, 675, 677),
        );
        let template_identifier = builder.add_token(
            syntax::SurfaceTokenKind::Identifier,
            "T",
            range(source, 678, 679),
        );
        let type_head = builder.add_node(
            syntax::SurfaceNodeKind::TypeHead,
            range(source, 678, 679),
            vec![template_identifier],
        );
        let type_expression = builder.add_node(
            syntax::SurfaceNodeKind::TypeExpression,
            range(source, 678, 679),
            vec![type_head],
        );
        let segment = builder.add_node(
            syntax::SurfaceNodeKind::ComprehensionVariableSegment,
            range(source, 673, 679),
            vec![generator_binder, is_keyword, type_expression],
        );
        let colon = builder.add_token(
            syntax::SurfaceTokenKind::ReservedSymbol,
            ":",
            range(source, 680, 681),
        );
        let not_keyword = builder.add_token(
            syntax::SurfaceTokenKind::ReservedWord,
            "not",
            range(source, 682, 685),
        );
        let first_identifier = builder.add_token(
            syntax::SurfaceTokenKind::Identifier,
            "x",
            range(source, 686, 687),
        );
        let first_reference = builder.add_node(
            syntax::SurfaceNodeKind::TermReference,
            range(source, 686, 687),
            vec![first_identifier],
        );
        let first_term = builder.add_node(
            syntax::SurfaceNodeKind::TermExpression,
            range(source, 686, 687),
            vec![first_reference],
        );
        let membership = builder.add_token(
            syntax::SurfaceTokenKind::ReservedWord,
            "in",
            range(source, 688, 690),
        );
        let second_identifier = builder.add_token(
            syntax::SurfaceTokenKind::Identifier,
            "x",
            range(source, 691, 692),
        );
        let second_reference = builder.add_node(
            syntax::SurfaceNodeKind::TermReference,
            range(source, 691, 692),
            vec![second_identifier],
        );
        let second_term = builder.add_node(
            syntax::SurfaceNodeKind::TermExpression,
            range(source, 691, 692),
            vec![second_reference],
        );
        let predicate = builder.add_node(
            syntax::SurfaceNodeKind::BuiltinPredicateApplication,
            range(source, 686, 692),
            vec![first_term, membership, second_term],
        );
        let prefix = builder.add_node(
            syntax::SurfaceNodeKind::PrefixFormula(syntax::SurfaceFormulaPrefixOperator::Not),
            range(source, 682, 692),
            vec![not_keyword, predicate],
        );
        let condition = builder.add_node(
            syntax::SurfaceNodeKind::FormulaExpression,
            range(source, 682, 692),
            vec![prefix],
        );
        let close = builder.add_token(
            syntax::SurfaceTokenKind::ReservedSymbol,
            "}",
            range(source, 693, 694),
        );
        let comprehension = builder.add_node(
            syntax::SurfaceNodeKind::SetComprehension,
            range(source, 663, 694),
            vec![
                open,
                mapper,
                where_keyword,
                segment,
                colon,
                condition,
                close,
            ],
        );
        let term_expression = builder.add_node(
            syntax::SurfaceNodeKind::TermExpression,
            range(source, 663, 694),
            vec![comprehension],
        );
        let definiens = builder.add_node(
            syntax::SurfaceNodeKind::TermDefiniens,
            range(source, 663, 694),
            vec![term_expression],
        );
        let functor = builder.add_node(
            syntax::SurfaceNodeKind::FunctorDefinition,
            range(source, 630, 700),
            vec![func, definiens],
        );
        let definition_block = builder.add_node(
            syntax::SurfaceNodeKind::DefinitionBlockItem,
            range(source, 600, 700),
            vec![definition, parameter, functor],
        );
        let root = builder.add_node(
            syntax::SurfaceNodeKind::Root,
            range(source, 600, 700),
            vec![definition_block],
        );
        builder.finish(Some(root), None)
    }

    fn task257c4a_typed_ast(
        ast: &syntax::SurfaceAst,
        module: ModuleId,
        resolved: &SurfaceResolvedArena,
    ) -> TypedAst {
        let mut builder = TypedArenaBuilder::new();
        let mut typed_by_surface = std::collections::BTreeMap::new();
        for view in ast.node_views() {
            let surface = ast.node(view.id()).expect("Task257C4A surface node");
            let resolved_node = resolved
                .resolved_node_for(view.id())
                .expect("Task257C4A resolver mapping");
            let kind = match &surface.kind {
                syntax::SurfaceNodeKind::Token(token)
                    if token.kind == syntax::SurfaceTokenKind::Identifier =>
                {
                    "Identifier".to_owned()
                }
                _ => format!("{:?}", surface.kind),
            };
            let typed = TypedNode::new(kind, SourceAnchor::Range(surface.range))
                .with_children(
                    surface
                        .children
                        .iter()
                        .map(|child| {
                            *typed_by_surface
                                .get(child)
                                .expect("Task257C4A child before parent")
                        })
                        .collect(),
                )
                .with_resolved_node(resolved_node);
            let typed_id = builder.push(typed).expect("Task257C4A typed node");
            assert!(typed_by_surface.insert(view.id(), typed_id).is_none());
        }
        let root = ast
            .root()
            .and_then(|id| typed_by_surface.get(&id).copied())
            .expect("Task257C4A typed root");
        TypedAst::try_new(TypedAstParts {
            source_id: ast.source_id,
            module_id: module,
            resolved_root: None,
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes: builder.finish(Some(root)).expect("Task257C4A typed arena"),
            contexts: LocalTypeContextTable::new(),
            types: TypeTable::new(),
            facts: TypeFactTable::new(),
            coercions: CoercionTable::new(),
            initial_obligations: InitialObligationTable::new(),
            diagnostics: TypeDiagnosticTable::new(),
        })
        .expect("Task257C4A typed AST")
    }

    fn task257c4a_typed_from_nodes(
        source: SourceId,
        module: ModuleId,
        root: Option<TypedNodeId>,
        nodes: Vec<TypedNode>,
    ) -> TypedAst {
        TypedAst::try_new(TypedAstParts {
            source_id: source,
            module_id: module,
            resolved_root: None,
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes: TypedArena::try_new(root, nodes).expect("Task257C4A typed arena mutation"),
            contexts: LocalTypeContextTable::new(),
            types: TypeTable::new(),
            facts: TypeFactTable::new(),
            coercions: CoercionTable::new(),
            initial_obligations: InitialObligationTable::new(),
            diagnostics: TypeDiagnosticTable::new(),
        })
        .expect("Task257C4A typed AST mutation")
    }

    fn task257c4a_empty_resolver(
        source: SourceId,
        module: &ModuleId,
    ) -> FraenkelGeneratorVariableSourceCollection {
        let mut builder = syntax::SurfaceAstBuilder::new(source);
        let root = builder.add_node(
            syntax::SurfaceNodeKind::Root,
            range(source, 600, 601),
            Vec::new(),
        );
        let ast = builder.finish(Some(root), None);
        let resolved =
            SurfaceResolvedArena::lower(&ast, module).expect("Task257C4A empty resolver arena");
        FraenkelGeneratorVariableSourceCollector::new(&ast, module, &resolved)
            .expect("Task257C4A empty generator collector")
            .collect()
            .expect("Task257C4A empty generator collection")
    }

    #[test]
    fn task257c4a_builds_exact_fraenkel_generator_binding_context() {
        let fixture = task257c4a_fixture();
        let handoff = SourceFraenkelGeneratorBindingContextProducer::build(
            &fixture.structural,
            &fixture.resolver,
            &fixture.typed_ast,
        )
        .expect("Task257C4A binding context");

        assert_eq!(handoff.source_id(), fixture.source);
        assert_eq!(handoff.module_id(), &fixture.module);
        assert_eq!(handoff.bindings().len(), 1);
        assert!(!handoff.bindings().is_empty());
        assert_eq!(handoff.use_positions().len(), 3);
        let binding_rows = handoff.bindings().iter().collect::<Vec<_>>();
        let [(binding_context, binding)] = binding_rows.as_slice() else {
            panic!("Task257C4A requires one binding context");
        };
        assert_eq!(binding_context.index(), 0);
        assert_eq!(binding.composition().index(), 0);
        assert_eq!(binding.resolver_binding().index(), 0);
        assert_eq!(binding.context(), BindingContextId::new(1));
        assert_eq!(binding.binding(), BindingId::new(0));
        assert_eq!(binding.source_ordinal(), 0);
        assert_eq!(
            handoff
                .use_positions()
                .iter()
                .map(|(id, row)| (
                    id.index(),
                    row.binding_context().index(),
                    row.resolver_use_index(),
                    row.source_ordinal(),
                    row.lookup_ordinal(),
                ))
                .collect::<Vec<_>>(),
            vec![(0, 0, 0, 0, 1), (1, 0, 1, 1, 2), (2, 0, 2, 2, 3)]
        );
        assert_eq!(
            handoff.binding_env().debug_text(),
            "binding-env-debug-v1\nmodule: pkg::composition.fixture\ncontexts:\n  context#0 owner=module parent=none layer=module scope=none bindings=[] visible=[] recovery=normal\n  context#1 owner=source-comprehension(663..694) parent=context#0 layer=expression scope=none bindings=[binding#0] visible=[binding#0] recovery=normal\nbindings:\n  binding#0 spelling=\"x\" kind=quantifier_binder owner=context#1 identity=source_bound(context#1, ordinal=0) range=673..674 visible_after=0 type=source(678..679) status=active captured=[] diagnostics=[] recovery=normal\ndiagnostics:\n"
        );
        assert!(matches!(
            handoff.binding_env().lookup(&BindingLookupSite::new(
                "x",
                BindingContextId::new(1),
                None,
                0,
            )),
            Ok(BindingLookupResult::ForwardReference { candidates, .. })
                if candidates == vec![BindingId::new(0)]
        ));
        assert!(matches!(
            handoff.binding_env().lookup(&BindingLookupSite::new(
                "x",
                BindingContextId::new(1),
                None,
                3,
            )),
            Ok(BindingLookupResult::Local(binding)) if binding == BindingId::new(0)
        ));
        assert_eq!(
            handoff.debug_text(),
            "source-fraenkel-generator-binding-context-v1|module=pkg.composition.fixture|bindings=1|use-positions=3"
        );
    }

    #[test]
    fn task257c4a_rejects_environment_structural_and_resolver_corruption() {
        let fixture = task257c4a_fixture();
        let wrong_module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("other"));
        let wrong_environment = task257c4a_typed_from_nodes(
            fixture.source,
            wrong_module,
            fixture.typed_ast.nodes().root(),
            fixture
                .typed_ast
                .nodes()
                .iter()
                .map(|(_, node)| node.clone())
                .collect(),
        );
        assert!(matches!(
            SourceFraenkelGeneratorBindingContextProducer::build(
                &fixture.structural,
                &fixture.resolver,
                &wrong_environment,
            ),
            Err(SourceFraenkelGeneratorBindingContextError::EnvironmentMismatch)
        ));

        let parameter = fixture
            .structural
            .compositions()
            .get(SourceTemplateFraenkelStructuralCompositionId::new(0))
            .expect("Task257C4A composition")
            .parameter();
        let mut corrupted_nodes = fixture
            .typed_ast
            .nodes()
            .iter()
            .map(|(_, node)| node.clone())
            .collect::<Vec<_>>();
        corrupted_nodes[parameter.index()].resolved_node = None;
        let corrupted_structural = task257c4a_typed_from_nodes(
            fixture.source,
            fixture.module.clone(),
            fixture.typed_ast.nodes().root(),
            corrupted_nodes,
        );
        assert!(matches!(
            SourceFraenkelGeneratorBindingContextProducer::build(
                &fixture.structural,
                &fixture.resolver,
                &corrupted_structural,
            ),
            Err(SourceFraenkelGeneratorBindingContextError::InvalidStructuralDependency)
        ));

        let composition = fixture
            .structural
            .compositions()
            .get(SourceTemplateFraenkelStructuralCompositionId::new(0))
            .expect("Task257C4A composition");
        let mut detached_condition_nodes = fixture
            .typed_ast
            .nodes()
            .iter()
            .map(|(_, node)| node.clone())
            .collect::<Vec<_>>();
        detached_condition_nodes[composition.comprehension().index()]
            .children
            .retain(|child| *child != composition.first_condition_role_owner());
        let detached_condition = task257c4a_typed_from_nodes(
            fixture.source,
            fixture.module.clone(),
            fixture.typed_ast.nodes().root(),
            detached_condition_nodes,
        );
        assert!(matches!(
            SourceFraenkelGeneratorBindingContextProducer::build(
                &fixture.structural,
                &fixture.resolver,
                &detached_condition,
            ),
            Err(SourceFraenkelGeneratorBindingContextError::InvalidStructuralDependency)
        ));

        assert!(matches!(
            SourceFraenkelGeneratorBindingContextProducer::build(
                &fixture.structural,
                &task257c4a_empty_resolver(fixture.source, &fixture.module),
                &fixture.typed_ast,
            ),
            Err(SourceFraenkelGeneratorBindingContextError::InvalidResolverDependency)
        ));
        assert!(matches!(
            SourceFraenkelGeneratorBindingContextProducer::build(
                &fixture.structural,
                &task257c4a_empty_resolver(fixture.source, &fixture.module),
                &corrupted_structural,
            ),
            Err(SourceFraenkelGeneratorBindingContextError::InvalidStructuralDependency)
        ));
        assert!(matches!(
            SourceFraenkelGeneratorBindingContextProducer::build(
                &fixture.structural,
                &task257c4a_empty_resolver(fixture.source, &fixture.module),
                &wrong_environment,
            ),
            Err(SourceFraenkelGeneratorBindingContextError::EnvironmentMismatch)
        ));

        let mut snapshot_resolver = SourceFraenkelGeneratorBindingContextProducer::build(
            &fixture.structural,
            &fixture.resolver,
            &fixture.typed_ast,
        )
        .expect("Task257C4A snapshot handoff");
        snapshot_resolver.dependencies.resolver =
            task257c4a_empty_resolver(fixture.source, &fixture.module);
        assert_eq!(
            snapshot_resolver.validate(),
            Err(SourceFraenkelGeneratorBindingContextError::InvalidResolverDependency)
        );

        let mut structural_summary_precedence =
            SourceFraenkelGeneratorBindingContextProducer::build(
                &fixture.structural,
                &fixture.resolver,
                &fixture.typed_ast,
            )
            .expect("Task257C4A snapshot handoff");
        structural_summary_precedence
            .structural_summary
            .push_str("-stale");
        structural_summary_precedence.dependencies.resolver =
            task257c4a_empty_resolver(fixture.source, &fixture.module);
        assert_eq!(
            structural_summary_precedence.validate(),
            Err(SourceFraenkelGeneratorBindingContextError::InvalidStructuralDependency)
        );

        let mut snapshot_structural = SourceFraenkelGeneratorBindingContextProducer::build(
            &fixture.structural,
            &fixture.resolver,
            &fixture.typed_ast,
        )
        .expect("Task257C4A snapshot handoff");
        snapshot_structural.dependencies.typed_ast = corrupted_structural.clone();
        snapshot_structural.dependencies.resolver =
            task257c4a_empty_resolver(fixture.source, &fixture.module);
        assert_eq!(
            snapshot_structural.validate(),
            Err(SourceFraenkelGeneratorBindingContextError::InvalidStructuralDependency)
        );

        let mut snapshot_environment = SourceFraenkelGeneratorBindingContextProducer::build(
            &fixture.structural,
            &fixture.resolver,
            &fixture.typed_ast,
        )
        .expect("Task257C4A snapshot handoff");
        snapshot_environment.dependencies.typed_ast = wrong_environment;
        snapshot_environment.dependencies.resolver =
            task257c4a_empty_resolver(fixture.source, &fixture.module);
        assert_eq!(
            snapshot_environment.validate(),
            Err(SourceFraenkelGeneratorBindingContextError::EnvironmentMismatch)
        );
    }

    #[test]
    fn task257c4a_rejects_context_identity_range_position_and_profile_corruption() {
        let fixture = task257c4a_fixture();
        let build = || {
            SourceFraenkelGeneratorBindingContextProducer::build(
                &fixture.structural,
                &fixture.resolver,
                &fixture.typed_ast,
            )
            .expect("Task257C4A valid handoff")
        };

        let mut context = build();
        context.bindings.rows[0].context = BindingContextId::new(0);
        assert_eq!(
            context.validate(),
            Err(
                SourceFraenkelGeneratorBindingContextError::InvalidBindingContext {
                    binding_context: SourceFraenkelGeneratorBindingContextId::new(0),
                }
            )
        );

        let mut position = build();
        position.use_positions.rows[2].lookup_ordinal = 9;
        assert_eq!(
            position.validate(),
            Err(
                SourceFraenkelGeneratorBindingContextError::InvalidUsePosition {
                    use_position: SourceFraenkelGeneratorUsePositionId::new(2),
                }
            )
        );

        let mut identity_and_range = build();
        let binding = identity_and_range
            .binding_env
            .binding_mut_for_test(BindingId::new(0))
            .expect("Task257C4A binding");
        binding.identity = BinderIdentity::Generated {
            context: BindingContextId::new(1),
            counter: 0,
        };
        binding.declaration_range = range(fixture.source, 672, 674);
        assert_eq!(
            identity_and_range.validate(),
            Err(SourceFraenkelGeneratorBindingContextError::InvalidEnvironment)
        );

        let mut binding_context_precedence = build();
        binding_context_precedence.bindings.rows[0].context = BindingContextId::new(0);
        binding_context_precedence.use_positions.rows[0].lookup_ordinal = 9;
        binding_context_precedence
            .binding_env
            .binding_mut_for_test(BindingId::new(0))
            .expect("Task257C4A precedence binding")
            .declaration_range = range(fixture.source, 672, 674);
        assert_eq!(
            binding_context_precedence.validate(),
            Err(
                SourceFraenkelGeneratorBindingContextError::InvalidBindingContext {
                    binding_context: SourceFraenkelGeneratorBindingContextId::new(0),
                }
            )
        );

        let mut use_position_precedence = build();
        use_position_precedence.use_positions.rows[0].lookup_ordinal = 9;
        use_position_precedence
            .binding_env
            .binding_mut_for_test(BindingId::new(0))
            .expect("Task257C4A precedence binding")
            .declaration_range = range(fixture.source, 672, 674);
        assert_eq!(
            use_position_precedence.validate(),
            Err(
                SourceFraenkelGeneratorBindingContextError::InvalidUsePosition {
                    use_position: SourceFraenkelGeneratorUsePositionId::new(0),
                }
            )
        );

        let mut missing_binding = build();
        missing_binding.bindings.rows.clear();
        assert_eq!(
            missing_binding.validate(),
            Err(
                SourceFraenkelGeneratorBindingContextError::InvalidBindingContext {
                    binding_context: SourceFraenkelGeneratorBindingContextId::new(0),
                }
            )
        );

        let mut duplicate_binding = build();
        duplicate_binding
            .bindings
            .rows
            .push(duplicate_binding.bindings.rows[0].clone());
        assert_eq!(
            duplicate_binding.validate(),
            Err(
                SourceFraenkelGeneratorBindingContextError::InvalidBindingContext {
                    binding_context: SourceFraenkelGeneratorBindingContextId::new(0),
                }
            )
        );

        let mut missing_use = build();
        let _ = missing_use.use_positions.rows.pop();
        assert_eq!(
            missing_use.validate(),
            Err(
                SourceFraenkelGeneratorBindingContextError::InvalidUsePosition {
                    use_position: SourceFraenkelGeneratorUsePositionId::new(0),
                }
            )
        );

        let mut duplicate_use = build();
        duplicate_use
            .use_positions
            .rows
            .push(duplicate_use.use_positions.rows[0].clone());
        assert_eq!(
            duplicate_use.validate(),
            Err(
                SourceFraenkelGeneratorBindingContextError::InvalidUsePosition {
                    use_position: SourceFraenkelGeneratorUsePositionId::new(0),
                }
            )
        );

        let mut recovery = build();
        recovery
            .binding_env
            .binding_mut_for_test(BindingId::new(0))
            .expect("Task257C4A recovery binding")
            .recovery = BindingRecoveryState::Recovered;
        assert_eq!(
            recovery.validate(),
            Err(SourceFraenkelGeneratorBindingContextError::InvalidEnvironment)
        );

        let f5_parts = || BindingEnvParts {
            source_id: fixture.source,
            module_id: fixture.module.clone(),
            contexts: build().binding_env.contexts().clone(),
            bindings: build().binding_env.bindings().clone(),
            diagnostics: build().binding_env.diagnostics().clone(),
        };

        let mut wrong_owner = f5_parts();
        wrong_owner
            .bindings
            .get_mut_for_test(BindingId::new(0))
            .expect("Task257C4A source-bound binding")
            .identity = BinderIdentity::SourceBound {
            context: BindingContextId::new(0),
            ordinal: 0,
        };
        assert_eq!(
            BindingEnv::try_new(wrong_owner),
            Err(BindingEnvError::InvalidSourceBoundIdentityOwner {
                binding: BindingId::new(0),
                context: BindingContextId::new(0),
                owner: BindingContextId::new(1),
            })
        );

        let mut wrong_kind = f5_parts();
        wrong_kind
            .bindings
            .get_mut_for_test(BindingId::new(0))
            .expect("Task257C4A source-bound binding")
            .kind = BindingKind::LetBinding;
        assert_eq!(
            BindingEnv::try_new(wrong_kind),
            Err(BindingEnvError::InconsistentSourceBoundIdentity {
                binding: BindingId::new(0),
            })
        );

        let mut wrong_ordinal = f5_parts();
        wrong_ordinal
            .bindings
            .get_mut_for_test(BindingId::new(0))
            .expect("Task257C4A source-bound binding")
            .identity = BinderIdentity::SourceBound {
            context: BindingContextId::new(1),
            ordinal: 1,
        };
        assert_eq!(
            BindingEnv::try_new(wrong_ordinal),
            Err(BindingEnvError::InconsistentSourceBoundIdentity {
                binding: BindingId::new(0),
            })
        );

        let mut missing_context = f5_parts();
        missing_context
            .bindings
            .get_mut_for_test(BindingId::new(0))
            .expect("Task257C4A source-bound binding")
            .identity = BinderIdentity::SourceBound {
            context: BindingContextId::new(2),
            ordinal: 0,
        };
        assert_eq!(
            BindingEnv::try_new(missing_context),
            Err(BindingEnvError::InvalidSourceBoundIdentityContext {
                binding: BindingId::new(0),
                context: BindingContextId::new(2),
            })
        );

        let mut captured_missing_context = f5_parts();
        captured_missing_context
            .bindings
            .get_mut_for_test(BindingId::new(0))
            .expect("Task257C4A source-bound binding")
            .captured = CapturedFreeVariables::new(vec![BinderIdentity::SourceBound {
            context: BindingContextId::new(2),
            ordinal: 0,
        }]);
        assert_eq!(
            BindingEnv::try_new(captured_missing_context),
            Err(BindingEnvError::InvalidSourceBoundIdentityContext {
                binding: BindingId::new(0),
                context: BindingContextId::new(2),
            })
        );

        let mut invalid_contexts = BindingContextTable::new();
        let root = invalid_contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::Module,
            parent: None,
            layer: BindingContextLayer::Module,
            lexical_scope: None,
            bindings: Vec::new(),
            visible_bindings: Vec::new(),
            recovery: BindingContextRecovery::Normal,
        });
        let invalid_context = invalid_contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::SourceComprehension {
                source_range: range(other_source_id(), 663, 694),
            },
            parent: Some(root),
            layer: BindingContextLayer::Expression,
            lexical_scope: None,
            bindings: Vec::new(),
            visible_bindings: Vec::new(),
            recovery: BindingContextRecovery::Normal,
        });
        assert_eq!(
            BindingEnv::try_new(BindingEnvParts {
                source_id: fixture.source,
                module_id: fixture.module.clone(),
                contexts: invalid_contexts,
                bindings: BindingTable::new(),
                diagnostics: BindingDiagnosticTable::new(),
            }),
            Err(BindingEnvError::InvalidContextSourceRange {
                context: invalid_context,
            })
        );
    }

    #[test]
    fn task257c4a_rebuilds_deterministically_without_mutation() {
        let fixture = task257c4a_fixture();
        let structural_before = fixture.structural.clone();
        let resolver_before = fixture.resolver.clone();
        let typed_before = fixture.typed_ast.clone();
        let first = SourceFraenkelGeneratorBindingContextProducer::build(
            &fixture.structural,
            &fixture.resolver,
            &fixture.typed_ast,
        )
        .expect("Task257C4A first build");
        let second = SourceFraenkelGeneratorBindingContextProducer::build(
            &fixture.structural,
            &fixture.resolver,
            &fixture.typed_ast,
        )
        .expect("Task257C4A second build");
        assert!(first == second);
        assert_eq!(fixture.structural, structural_before);
        assert_eq!(fixture.resolver, resolver_before);
        assert_eq!(fixture.typed_ast, typed_before);
    }

    #[test]
    fn task257c4b_builds_exact_fraenkel_generator_bound_uses() {
        let fixture = task257c4a_fixture();
        let binding_context = SourceFraenkelGeneratorBindingContextProducer::build(
            &fixture.structural,
            &fixture.resolver,
            &fixture.typed_ast,
        )
        .expect("Task257C4B binding-context dependency");
        let handoff = SourceFraenkelGeneratorBoundUseProducer::build(&binding_context)
            .expect("Task257C4B bound uses");

        assert_eq!(handoff.source_id(), fixture.source);
        assert_eq!(handoff.module_id(), &fixture.module);
        assert_eq!(handoff.dependency_summary(), binding_context.debug_text());
        assert_eq!(handoff.bound_uses().len(), 3);
        assert!(!handoff.bound_uses().is_empty());
        assert_eq!(handoff.bound_uses().iter().count(), 3);
        assert_eq!(
            handoff
                .bound_uses()
                .iter()
                .map(|(id, row)| (
                    id.index(),
                    row.use_position().index(),
                    row.binding_context().index(),
                    row.resolver_use_index(),
                    row.source_ordinal(),
                    row.lookup_ordinal(),
                    row.context().index(),
                    row.binding().index(),
                ))
                .collect::<Vec<_>>(),
            vec![
                (0, 0, 0, 0, 0, 1, 1, 0),
                (1, 1, 0, 1, 1, 2, 1, 0),
                (2, 2, 0, 2, 2, 3, 1, 0),
            ]
        );
        for (id, row) in handoff.bound_uses().iter() {
            assert_eq!(handoff.bound_uses().get(id), Some(row));
            assert!(matches!(
                binding_context.binding_env().lookup(&BindingLookupSite::new(
                    "x",
                    row.context(),
                    None,
                    row.lookup_ordinal(),
                )),
                Ok(BindingLookupResult::Local(binding)) if binding == row.binding()
            ));
        }
        assert!(
            handoff
                .bound_uses()
                .get(SourceFraenkelGeneratorBoundUseId::new(3))
                .is_none()
        );
        assert!(matches!(
            binding_context.binding_env().lookup(&BindingLookupSite::new(
                "x",
                BindingContextId::new(1),
                None,
                0,
            )),
            Ok(BindingLookupResult::ForwardReference { candidates, .. })
                if candidates == vec![BindingId::new(0)]
        ));
        assert_eq!(
            handoff.debug_text(),
            "source-fraenkel-generator-bound-use-v1|module=pkg.composition.fixture|bound-uses=3"
        );
    }

    #[test]
    fn task257c4b_rejects_environment_and_binding_context_dependency_corruption() {
        let fixture = task257c4a_fixture();
        let binding_context = SourceFraenkelGeneratorBindingContextProducer::build(
            &fixture.structural,
            &fixture.resolver,
            &fixture.typed_ast,
        )
        .expect("Task257C4B binding-context dependency");
        let build = || {
            SourceFraenkelGeneratorBoundUseProducer::build(&binding_context)
                .expect("Task257C4B valid handoff")
        };

        let mut wrong_source = build();
        wrong_source.source_id = other_source_id();
        wrong_source.dependencies.version = "stale";
        wrong_source.bound_uses.rows[0].lookup_ordinal = 0;
        assert_eq!(
            wrong_source.validate(),
            Err(SourceFraenkelGeneratorBoundUseError::EnvironmentMismatch)
        );

        let mut wrong_module = build();
        wrong_module.module_id = ModuleId::new(PackageId::new("pkg"), ModulePath::new("other"));
        assert_eq!(
            wrong_module.validate(),
            Err(SourceFraenkelGeneratorBoundUseError::EnvironmentMismatch)
        );

        let mut stale_version = build();
        stale_version.dependencies.version = "stale";
        assert_eq!(
            stale_version.validate(),
            Err(SourceFraenkelGeneratorBoundUseError::InvalidBindingContextDependency)
        );

        let mut stale_domain = build();
        stale_domain.dependencies.domain = "stale";
        assert_eq!(
            stale_domain.validate(),
            Err(SourceFraenkelGeneratorBoundUseError::InvalidBindingContextDependency)
        );

        let mut stale_summary = build();
        stale_summary.dependency_summary.push_str("-stale");
        stale_summary.bound_uses.rows[0].lookup_ordinal = 0;
        assert_eq!(
            stale_summary.validate(),
            Err(SourceFraenkelGeneratorBoundUseError::InvalidBindingContextDependency)
        );

        let mut stale_resolver = build();
        stale_resolver
            .dependencies
            .binding_context
            .dependencies
            .resolver = task257c4a_empty_resolver(fixture.source, &fixture.module);
        stale_resolver.bound_uses.rows[0].lookup_ordinal = 0;
        assert_eq!(
            stale_resolver.validate(),
            Err(SourceFraenkelGeneratorBoundUseError::InvalidBindingContextDependency)
        );

        let other_source = other_source_id();
        let other_ast = task257c4a_surface_ast(other_source);
        let other_resolved = SurfaceResolvedArena::lower(&other_ast, &fixture.module)
            .expect("Task257C4B other resolver arena");
        let other_templates =
            TemplateTypeParameterSourceCollector::new(&other_ast, &fixture.module, &other_resolved)
                .expect("Task257C4B other template collector")
                .collect()
                .expect("Task257C4B other template collection");
        let other_resolver = FraenkelGeneratorVariableSourceCollector::new(
            &other_ast,
            &fixture.module,
            &other_resolved,
        )
        .expect("Task257C4B other generator collector")
        .collect()
        .expect("Task257C4B other generator collection");
        let other_typed = task257c4a_typed_ast(&other_ast, fixture.module.clone(), &other_resolved);
        let other_template =
            SourceTemplateTypeParameterAssociationProducer::build(&other_templates, &other_typed)
                .expect("Task257C4B other template handoff");
        let other_structural = SourceTemplateFraenkelStructuralCompositionProducer::build(
            &other_template,
            &other_resolver,
            &other_typed,
        )
        .expect("Task257C4B other structural handoff");
        let mut stale_structural = build();
        stale_structural
            .dependencies
            .binding_context
            .dependencies
            .structural = other_structural;
        stale_structural.bound_uses.rows[0].lookup_ordinal = 0;
        assert_eq!(
            stale_structural.validate(),
            Err(SourceFraenkelGeneratorBoundUseError::InvalidBindingContextDependency)
        );

        let parameter = binding_context
            .dependencies
            .structural
            .compositions()
            .get(SourceTemplateFraenkelStructuralCompositionId::new(0))
            .expect("Task257C4B structural composition")
            .parameter();
        let mut stale_nodes = binding_context
            .dependencies
            .typed_ast
            .nodes()
            .iter()
            .map(|(_, node)| node.clone())
            .collect::<Vec<_>>();
        stale_nodes[parameter.index()].resolved_node = None;
        let stale_typed = task257c4a_typed_from_nodes(
            fixture.source,
            fixture.module.clone(),
            binding_context.dependencies.typed_ast.nodes().root(),
            stale_nodes,
        );
        let mut stale_typed_dependency = build();
        stale_typed_dependency
            .dependencies
            .binding_context
            .dependencies
            .typed_ast = stale_typed;
        stale_typed_dependency.bound_uses.rows[0].lookup_ordinal = 0;
        assert_eq!(
            stale_typed_dependency.validate(),
            Err(SourceFraenkelGeneratorBoundUseError::InvalidBindingContextDependency)
        );

        let mut stale_position = build();
        stale_position
            .dependencies
            .binding_context
            .use_positions
            .rows[0]
            .lookup_ordinal = 9;
        assert_eq!(
            stale_position.validate(),
            Err(SourceFraenkelGeneratorBoundUseError::InvalidBindingContextDependency)
        );

        let mut stale_environment = build();
        stale_environment
            .dependencies
            .binding_context
            .binding_env
            .binding_mut_for_test(BindingId::new(0))
            .expect("Task257C4B retained binding")
            .recovery = BindingRecoveryState::Recovered;
        assert_eq!(
            stale_environment.validate(),
            Err(SourceFraenkelGeneratorBoundUseError::InvalidBindingContextDependency)
        );

        let mut invalid_input = binding_context.clone();
        invalid_input.use_positions.rows[0].lookup_ordinal = 9;
        assert!(matches!(
            SourceFraenkelGeneratorBoundUseProducer::build(&invalid_input),
            Err(SourceFraenkelGeneratorBoundUseError::InvalidBindingContextDependency)
        ));
    }

    #[test]
    fn task257c4b_rejects_bound_use_and_lookup_corruption() {
        let fixture = task257c4a_fixture();
        let binding_context = SourceFraenkelGeneratorBindingContextProducer::build(
            &fixture.structural,
            &fixture.resolver,
            &fixture.typed_ast,
        )
        .expect("Task257C4B binding-context dependency");
        let build = || {
            SourceFraenkelGeneratorBoundUseProducer::build(&binding_context)
                .expect("Task257C4B valid handoff")
        };
        let invalid = |candidate: &SourceFraenkelGeneratorBoundUseHandoff,
                       id: SourceFraenkelGeneratorBoundUseId| {
            assert_eq!(
                candidate.validate(),
                Err(SourceFraenkelGeneratorBoundUseError::InvalidBoundUse { bound_use: id })
            );
        };

        let mut missing = build();
        let _ = missing.bound_uses.rows.pop();
        invalid(&missing, SourceFraenkelGeneratorBoundUseId::new(0));

        let mut extra = build();
        extra.bound_uses.rows.push(extra.bound_uses.rows[0].clone());
        invalid(&extra, SourceFraenkelGeneratorBoundUseId::new(0));

        let mut reordered = build();
        reordered.bound_uses.rows.swap(0, 1);
        invalid(&reordered, SourceFraenkelGeneratorBoundUseId::new(0));

        let mut duplicate = build();
        duplicate.bound_uses.rows[1] = duplicate.bound_uses.rows[0].clone();
        invalid(&duplicate, SourceFraenkelGeneratorBoundUseId::new(1));

        let mut use_position = build();
        use_position.bound_uses.rows[0].use_position = SourceFraenkelGeneratorUsePositionId::new(2);
        invalid(&use_position, SourceFraenkelGeneratorBoundUseId::new(0));

        let mut binding_context_id = build();
        binding_context_id.bound_uses.rows[0].binding_context =
            SourceFraenkelGeneratorBindingContextId::new(1);
        invalid(
            &binding_context_id,
            SourceFraenkelGeneratorBoundUseId::new(0),
        );

        let mut resolver_use = build();
        resolver_use.bound_uses.rows[1].resolver_use_index = 99;
        invalid(&resolver_use, SourceFraenkelGeneratorBoundUseId::new(1));

        let mut source_ordinal = build();
        source_ordinal.bound_uses.rows[1].source_ordinal = 99;
        invalid(&source_ordinal, SourceFraenkelGeneratorBoundUseId::new(1));

        let mut non_local_lookup = build();
        non_local_lookup.bound_uses.rows[0].lookup_ordinal = 0;
        invalid(&non_local_lookup, SourceFraenkelGeneratorBoundUseId::new(0));

        let mut context = build();
        context.bound_uses.rows[2].context = BindingContextId::new(0);
        invalid(&context, SourceFraenkelGeneratorBoundUseId::new(2));

        let mut binding = build();
        binding.bound_uses.rows[2].binding = BindingId::new(1);
        invalid(&binding, SourceFraenkelGeneratorBoundUseId::new(2));

        let mut dependency_precedence = build();
        dependency_precedence.dependencies.domain = "stale";
        dependency_precedence.bound_uses.rows[0].lookup_ordinal = 0;
        assert_eq!(
            dependency_precedence.validate(),
            Err(SourceFraenkelGeneratorBoundUseError::InvalidBindingContextDependency)
        );
    }

    #[test]
    fn task257c4b_rebuilds_deterministically_without_mutation() {
        let fixture = task257c4a_fixture();
        let binding_context = SourceFraenkelGeneratorBindingContextProducer::build(
            &fixture.structural,
            &fixture.resolver,
            &fixture.typed_ast,
        )
        .expect("Task257C4B binding-context dependency");
        let before = binding_context.clone();
        let before_debug = binding_context.debug_text();
        let first = SourceFraenkelGeneratorBoundUseProducer::build(&binding_context)
            .expect("Task257C4B first build");
        let second = SourceFraenkelGeneratorBoundUseProducer::build(&binding_context)
            .expect("Task257C4B second build");

        assert!(first == second);
        assert!(binding_context == before);
        assert_eq!(binding_context.debug_text(), before_debug);
        assert!(first.dependencies.binding_context == binding_context);
        assert!(second.dependencies.binding_context == binding_context);
    }

    fn fixture() -> Fixture {
        let source = source_id();
        let module = module();
        let arena = TypedArena::try_new(
            None,
            [
                ("source.formula.composite.universal", range(source, 50, 77)),
                ("source.formula.quantifier-binder", range(source, 54, 65)),
                ("source.formula.quantifier-binder", range(source, 54, 55)),
                ("source.formula.binder-type", range(source, 62, 65)),
                ("source.formula.binder-type-head", range(source, 62, 65)),
                ("source.term.variable-reference", range(source, 72, 73)),
                ("source.term.variable-reference", range(source, 76, 77)),
                ("source.formula.atomic.equality", range(source, 72, 77)),
            ]
            .into_iter()
            .map(|(kind, source_range)| TypedNode::new(kind, SourceAnchor::Range(source_range)))
            .collect(),
        )
        .expect("arena");
        let composite_input = SourceCompositeFormulaHandoffInput {
            source_id: source,
            module_id: module.clone(),
            formulas: vec![SourceCompositeFormulaInput {
                site: node(0),
                source_range: range(source, 50, 77),
                source_ordinal: 0,
                context: BindingContextId::new(0),
                recovery: SourceCompositeFormulaRecovery::Normal,
                spelling: "for holds".to_owned(),
                kind: SourceCompositeFormulaKind::Universal,
            }],
            wrappers: Vec::new(),
            roots: vec![SourceFormulaRootInput {
                formula: SourceCompositeFormulaId::new(0),
                ordinal: 0,
                ownership: SourceFormulaRootOwnership::UnassignedStatement,
            }],
            binders: vec![SourceQuantifierBinderInput {
                formula: SourceCompositeFormulaId::new(0),
                ordinal: 0,
                segment_site: node(1),
                segment_range: range(source, 54, 65),
                segment_spelling: "x being".to_owned(),
                identifier_site: node(2),
                identifier_range: range(source, 54, 55),
                identifier_spelling: "x".to_owned(),
                local: LocalTermBinding::new(
                    "x",
                    LocalTermScope::new(vec![0]),
                    range(source, 54, 55),
                    0,
                ),
                binding: BindingId::new(0),
                body_context: BindingContextId::new(1),
                type_site: SourceBinderTypeSiteId::new(0),
                recovery: SourceCompositeFormulaRecovery::Normal,
            }],
            type_sites: vec![SourceBinderTypeSiteInput {
                binder: SourceQuantifierBinderId::new(0),
                site: node(3),
                source_range: range(source, 62, 65),
                spelling: "set".to_owned(),
                head_site: node(4),
                head_range: range(source, 62, 65),
                head_spelling: "set".to_owned(),
                context: BindingContextId::new(0),
                recovery: SourceCompositeFormulaRecovery::Normal,
                head: SourceBinderTypeHead::BuiltinSet,
            }],
            edges: Vec::new(),
            requests: vec![
                SourceFormulaRequestInput {
                    formula: SourceCompositeFormulaId::new(0),
                    ordinal: 0,
                    kind: SourceFormulaRequestKind::QuantifierSemantics,
                    binder: None,
                    type_site: None,
                },
                SourceFormulaRequestInput {
                    formula: SourceCompositeFormulaId::new(0),
                    ordinal: 1,
                    kind: SourceFormulaRequestKind::BinderType,
                    binder: Some(SourceQuantifierBinderId::new(0)),
                    type_site: Some(SourceBinderTypeSiteId::new(0)),
                },
            ],
        };
        let base = base_bindings(source, &module);
        let bindings =
            SourceCompositeFormulaProducer::extend_bindings(&composite_input, &base, &arena)
                .expect("extended bindings");
        let composite =
            SourceCompositeFormulaProducer::build(composite_input.clone(), &bindings, &arena)
                .expect("composite");
        let primary = SourcePrimaryTermProducer::build(
            SourcePrimaryTermHandoffInput {
                source_id: source,
                module_id: module.clone(),
                terms: [(5, 72, 73), (6, 76, 77)]
                    .into_iter()
                    .enumerate()
                    .map(|(ordinal, (site, start, end))| SourcePrimaryTermInput {
                        site: node(site),
                        source_range: range(source, start, end),
                        source_ordinal: ordinal,
                        context: BindingContextId::new(1),
                        recovery: SourcePrimaryTermRecovery::Normal,
                        spelling: "x".to_owned(),
                        kind: SourcePrimaryTermKind::VariableReference,
                        role: SourcePrimaryTermRole::Value,
                        parent: None,
                    })
                    .collect(),
                references: (0..2)
                    .map(|index| SourcePrimaryTermReferenceInput {
                        term: SourcePrimaryTermId::new(index),
                        binding: BindingId::new(0),
                        role: SourcePrimaryTermReferenceRole::Variable,
                    })
                    .collect(),
                numeric_type_requests: Vec::new(),
            },
            &bindings,
            &arena,
        )
        .expect("primary");
        let atomic = SourceAtomicFormulaProducer::build(
            SourceAtomicFormulaHandoffInput {
                source_id: source,
                module_id: module.clone(),
                formulas: vec![SourceAtomicFormulaInput {
                    site: node(7),
                    source_range: range(source, 72, 77),
                    source_ordinal: 0,
                    context: BindingContextId::new(1),
                    recovery: SourceAtomicFormulaRecovery::Normal,
                    spelling: "x = x".to_owned(),
                    kind: SourceAtomicFormulaKind::Equality,
                }],
                wrappers: Vec::new(),
                predicate_segments: Vec::new(),
                predicate_heads: Vec::new(),
                candidates: Vec::new(),
                type_sites: Vec::new(),
                attributes: Vec::new(),
                edges: vec![
                    SourceAtomicEdgeInput {
                        formula: SourceAtomicFormulaId::new(0),
                        ordinal: 0,
                        role: SourceAtomicEdgeRole::BuiltinLeftOperand,
                        target: SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(0)),
                    },
                    SourceAtomicEdgeInput {
                        formula: SourceAtomicFormulaId::new(0),
                        ordinal: 1,
                        role: SourceAtomicEdgeRole::BuiltinRightOperand,
                        target: SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(1)),
                    },
                ],
                requests: (0..2)
                    .map(|index| SourceAtomicRequestInput {
                        formula: SourceAtomicFormulaId::new(0),
                        ordinal: index,
                        kind: SourceAtomicRequestKind::OperandExpectedType,
                        edge: Some(SourceAtomicEdgeId::new(index)),
                        candidate: None,
                        type_site: None,
                        attribute: None,
                    })
                    .collect(),
            },
            &bindings,
            &SymbolEnv::new(module.clone(), SymbolEnvIndexes::default()),
            &primary,
            None,
            None,
            None,
            &arena,
        )
        .expect("atomic");
        let input = SourceFormulaCompositionHandoffInput {
            source_id: source,
            module_id: module.clone(),
            atomic_edges: vec![SourceFormulaAtomicEdgeInput {
                formula: SourceCompositeFormulaId::new(0),
                ordinal: 0,
                role: SourceFormulaAtomicEdgeRole::UniversalBody,
                child: SourceAtomicFormulaId::new(0),
            }],
            bound_uses: (0..2)
                .map(|index| SourceQuantifierBoundUseInput {
                    binder: SourceQuantifierBinderId::new(0),
                    ordinal: index,
                    body_edge: SourceFormulaAtomicEdgeId::new(0),
                    term: SourcePrimaryTermId::new(index),
                    reference: SourcePrimaryTermReferenceId::new(index),
                })
                .collect(),
        };
        Fixture {
            source,
            module,
            arena,
            composite_input,
            primary,
            atomic,
            composite,
            input,
        }
    }

    fn task_257b2_fixture() -> Task257B2Fixture {
        let composite_fixture =
            crate::source_composite_formula::tests::task_257b2_composite_fixture();
        let source = composite_fixture.source;
        let module = composite_fixture.module.clone();
        let arena = composite_fixture.arena.clone();
        let composite_input = composite_fixture.input.clone();
        let bindings = SourceCompositeFormulaProducer::extend_bindings(
            &composite_input,
            &composite_fixture.base,
            &arena,
        )
        .expect("Task 257B2 bindings");
        let composite =
            SourceCompositeFormulaProducer::build(composite_input.clone(), &bindings, &arena)
                .expect("Task 257B2 composite");
        let numeral_ranges = [
            (74, 75),
            (78, 79),
            (88, 89),
            (92, 93),
            (99, 100),
            (103, 104),
            (115, 116),
            (119, 120),
            (129, 130),
            (133, 134),
            (137, 138),
            (141, 142),
            (148, 149),
            (152, 153),
            (157, 158),
            (161, 162),
        ];
        let numeral_spellings = [
            "0", "0", "0", "3", "0", "0", "0", "3", "0", "0", "0", "0", "0", "0", "0", "0",
        ];
        let terms = numeral_ranges
            .into_iter()
            .zip(numeral_spellings)
            .enumerate()
            .map(|(index, ((start, end), spelling))| SourcePrimaryTermInput {
                site: node(26 + index),
                source_range: range(source, start, end),
                source_ordinal: index,
                context: BindingContextId::new(1),
                recovery: SourcePrimaryTermRecovery::Normal,
                spelling: spelling.to_owned(),
                kind: SourcePrimaryTermKind::Numeral,
                role: SourcePrimaryTermRole::Value,
                parent: None,
            })
            .collect::<Vec<_>>();
        let numeric_type_requests = terms
            .iter()
            .enumerate()
            .map(|(index, term)| SourceNumericTypeRequestInput {
                term: SourcePrimaryTermId::new(index),
                owner: term.site.clone(),
                source_range: term.source_range,
                spelling: term.spelling.clone(),
                request_ordinal: index,
            })
            .collect();
        let primary_input = SourcePrimaryTermHandoffInput {
            source_id: source,
            module_id: module.clone(),
            terms,
            references: Vec::new(),
            numeric_type_requests,
        };
        let primary = SourcePrimaryTermProducer::build(primary_input.clone(), &bindings, &arena)
            .expect("Task 257B2 primary terms");
        let equality_ranges = [
            (74, 79),
            (88, 93),
            (99, 104),
            (115, 120),
            (129, 134),
            (137, 142),
            (148, 153),
            (157, 162),
        ];
        let equality_spellings = [
            "0 = 0", "0 = 3", "0 = 0", "0 = 3", "0 = 0", "0 = 0", "0 = 0", "0 = 0",
        ];
        let formulas = equality_ranges
            .into_iter()
            .zip(equality_spellings)
            .enumerate()
            .map(
                |(index, ((start, end), spelling))| SourceAtomicFormulaInput {
                    site: node(18 + index),
                    source_range: range(source, start, end),
                    source_ordinal: index,
                    context: BindingContextId::new(1),
                    recovery: SourceAtomicFormulaRecovery::Normal,
                    spelling: spelling.to_owned(),
                    kind: SourceAtomicFormulaKind::Equality,
                },
            )
            .collect();
        let edges = (0..8)
            .flat_map(|formula| {
                [
                    SourceAtomicEdgeInput {
                        formula: SourceAtomicFormulaId::new(formula),
                        ordinal: 0,
                        role: SourceAtomicEdgeRole::BuiltinLeftOperand,
                        target: SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(
                            formula * 2,
                        )),
                    },
                    SourceAtomicEdgeInput {
                        formula: SourceAtomicFormulaId::new(formula),
                        ordinal: 1,
                        role: SourceAtomicEdgeRole::BuiltinRightOperand,
                        target: SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(
                            formula * 2 + 1,
                        )),
                    },
                ]
            })
            .collect::<Vec<_>>();
        let requests = edges
            .iter()
            .enumerate()
            .map(|(index, edge)| SourceAtomicRequestInput {
                formula: edge.formula,
                ordinal: edge.ordinal,
                kind: SourceAtomicRequestKind::OperandExpectedType,
                edge: Some(SourceAtomicEdgeId::new(index)),
                candidate: None,
                type_site: None,
                attribute: None,
            })
            .collect();
        let atomic_input = SourceAtomicFormulaHandoffInput {
            source_id: source,
            module_id: module.clone(),
            formulas,
            wrappers: Vec::new(),
            predicate_segments: Vec::new(),
            predicate_heads: Vec::new(),
            candidates: Vec::new(),
            type_sites: Vec::new(),
            attributes: Vec::new(),
            edges,
            requests,
        };
        let atomic = SourceAtomicFormulaProducer::build(
            atomic_input.clone(),
            &bindings,
            &SymbolEnv::new(module.clone(), SymbolEnvIndexes::default()),
            &primary,
            None,
            None,
            None,
            &arena,
        )
        .expect("Task 257B2 atomic formulas");
        let input = SourceFormulaCompositionHandoffInput {
            source_id: source,
            module_id: module.clone(),
            atomic_edges: [
                (3, 0, SourceFormulaAtomicEdgeRole::ConjunctionLeft, 0),
                (3, 1, SourceFormulaAtomicEdgeRole::ConjunctionRight, 1),
                (4, 0, SourceFormulaAtomicEdgeRole::DisjunctionLeft, 2),
                (4, 1, SourceFormulaAtomicEdgeRole::DisjunctionRight, 3),
                (6, 0, SourceFormulaAtomicEdgeRole::ConjunctionLeft, 4),
                (6, 1, SourceFormulaAtomicEdgeRole::ConjunctionRight, 5),
                (7, 0, SourceFormulaAtomicEdgeRole::DisjunctionLeft, 6),
                (7, 1, SourceFormulaAtomicEdgeRole::DisjunctionRight, 7),
            ]
            .into_iter()
            .map(
                |(formula, ordinal, role, child)| SourceFormulaAtomicEdgeInput {
                    formula: SourceCompositeFormulaId::new(formula),
                    ordinal,
                    role,
                    child: SourceAtomicFormulaId::new(child),
                },
            )
            .collect(),
            bound_uses: Vec::new(),
        };
        Task257B2Fixture {
            source,
            module,
            arena,
            bindings,
            primary_input,
            atomic_input,
            primary,
            atomic,
            composite,
            input,
        }
    }

    fn task_257b3_fixture() -> Task257B3Fixture {
        let composite_fixture =
            crate::source_composite_formula::tests::task_257b3_composite_fixture();
        let source = composite_fixture.source;
        let module = composite_fixture.module.clone();
        let mut nodes = composite_fixture
            .arena
            .iter()
            .map(|(_, node)| node.clone())
            .collect::<Vec<_>>();
        for (key, start, end) in [
            ("source.formula.atomic.equality", 86, 91),
            ("source.formula.atomic.equality", 119, 124),
            ("source.formula.atomic.equality", 131, 136),
            ("source.term.variable-reference", 86, 87),
            ("source.term.variable-reference", 90, 91),
            ("source.term.variable-reference", 119, 120),
            ("source.term.variable-reference", 123, 124),
            ("source.term.variable-reference", 131, 132),
            ("source.term.variable-reference", 135, 136),
        ] {
            nodes.push(TypedNode::new(
                key,
                SourceAnchor::Range(range(source, start, end)),
            ));
        }
        let arena = TypedArena::try_new(None, nodes).expect("Task 257B3 complete arena");
        let bindings = SourceCompositeFormulaProducer::extend_bindings(
            &composite_fixture.input,
            &composite_fixture.base,
            &arena,
        )
        .expect("Task 257B3 bindings");
        let composite =
            SourceCompositeFormulaProducer::build(composite_fixture.input, &bindings, &arena)
                .expect("Task 257B3 composite");
        let term_ranges = [
            (86, 87),
            (90, 91),
            (119, 120),
            (123, 124),
            (131, 132),
            (135, 136),
        ];
        let term_spellings = ["x", "x", "r", "y", "x", "r"];
        let term_contexts = [1, 1, 3, 3, 3, 3];
        let term_bindings = [1, 1, 3, 2, 1, 3];
        let terms = term_ranges
            .into_iter()
            .zip(term_spellings)
            .zip(term_contexts)
            .enumerate()
            .map(
                |(index, (((start, end), spelling), context))| SourcePrimaryTermInput {
                    site: node(18 + index),
                    source_range: range(source, start, end),
                    source_ordinal: index,
                    context: BindingContextId::new(context),
                    recovery: SourcePrimaryTermRecovery::Normal,
                    spelling: spelling.to_owned(),
                    kind: SourcePrimaryTermKind::VariableReference,
                    role: SourcePrimaryTermRole::Value,
                    parent: None,
                },
            )
            .collect();
        let references = term_bindings
            .into_iter()
            .enumerate()
            .map(|(index, binding)| SourcePrimaryTermReferenceInput {
                term: SourcePrimaryTermId::new(index),
                binding: BindingId::new(binding),
                role: SourcePrimaryTermReferenceRole::Variable,
            })
            .collect();
        let primary_input = SourcePrimaryTermHandoffInput {
            source_id: source,
            module_id: module.clone(),
            terms,
            references,
            numeric_type_requests: Vec::new(),
        };
        let primary = SourcePrimaryTermProducer::build(primary_input.clone(), &bindings, &arena)
            .expect("Task 257B3 primary terms");
        let equality_ranges = [(86, 91), (119, 124), (131, 136)];
        let equality_spellings = ["x = x", "r = y", "x = r"];
        let equality_contexts = [1, 3, 3];
        let formulas = equality_ranges
            .into_iter()
            .zip(equality_spellings)
            .zip(equality_contexts)
            .enumerate()
            .map(
                |(index, (((start, end), spelling), context))| SourceAtomicFormulaInput {
                    site: node(15 + index),
                    source_range: range(source, start, end),
                    source_ordinal: index,
                    context: BindingContextId::new(context),
                    recovery: SourceAtomicFormulaRecovery::Normal,
                    spelling: spelling.to_owned(),
                    kind: SourceAtomicFormulaKind::Equality,
                },
            )
            .collect();
        let edges = (0..3)
            .flat_map(|formula| {
                [
                    SourceAtomicEdgeInput {
                        formula: SourceAtomicFormulaId::new(formula),
                        ordinal: 0,
                        role: SourceAtomicEdgeRole::BuiltinLeftOperand,
                        target: SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(
                            formula * 2,
                        )),
                    },
                    SourceAtomicEdgeInput {
                        formula: SourceAtomicFormulaId::new(formula),
                        ordinal: 1,
                        role: SourceAtomicEdgeRole::BuiltinRightOperand,
                        target: SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(
                            formula * 2 + 1,
                        )),
                    },
                ]
            })
            .collect::<Vec<_>>();
        let requests = edges
            .iter()
            .enumerate()
            .map(|(index, edge)| SourceAtomicRequestInput {
                formula: edge.formula,
                ordinal: edge.ordinal,
                kind: SourceAtomicRequestKind::OperandExpectedType,
                edge: Some(SourceAtomicEdgeId::new(index)),
                candidate: None,
                type_site: None,
                attribute: None,
            })
            .collect();
        let atomic_input = SourceAtomicFormulaHandoffInput {
            source_id: source,
            module_id: module.clone(),
            formulas,
            wrappers: Vec::new(),
            predicate_segments: Vec::new(),
            predicate_heads: Vec::new(),
            candidates: Vec::new(),
            type_sites: Vec::new(),
            attributes: Vec::new(),
            edges,
            requests,
        };
        let atomic = SourceAtomicFormulaProducer::build(
            atomic_input.clone(),
            &bindings,
            &SymbolEnv::new(module.clone(), SymbolEnvIndexes::default()),
            &primary,
            None,
            None,
            None,
            &arena,
        )
        .expect("Task 257B3 atomic formulas");
        let input = SourceFormulaCompositionHandoffInput {
            source_id: source,
            module_id: module.clone(),
            atomic_edges: [
                (0, 0, SourceFormulaAtomicEdgeRole::UniversalRestriction, 0),
                (2, 0, SourceFormulaAtomicEdgeRole::UniversalRestriction, 1),
                (2, 1, SourceFormulaAtomicEdgeRole::UniversalBody, 2),
            ]
            .into_iter()
            .map(
                |(formula, ordinal, role, child)| SourceFormulaAtomicEdgeInput {
                    formula: SourceCompositeFormulaId::new(formula),
                    ordinal,
                    role,
                    child: SourceAtomicFormulaId::new(child),
                },
            )
            .collect(),
            bound_uses: [0, 0, 2, 1, 0, 2]
                .into_iter()
                .zip([0, 1, 0, 0, 2, 1])
                .zip([0, 0, 1, 1, 2, 2])
                .enumerate()
                .map(
                    |(index, ((binder, ordinal), body_edge))| SourceQuantifierBoundUseInput {
                        binder: SourceQuantifierBinderId::new(binder),
                        ordinal,
                        body_edge: SourceFormulaAtomicEdgeId::new(body_edge),
                        term: SourcePrimaryTermId::new(index),
                        reference: SourcePrimaryTermReferenceId::new(index),
                    },
                )
                .collect(),
        };
        Task257B3Fixture {
            source,
            module,
            arena,
            bindings,
            primary_input,
            atomic_input,
            primary,
            atomic,
            composite,
            input,
        }
    }

    fn build(
        fixture: &Fixture,
        input: SourceFormulaCompositionHandoffInput,
    ) -> Result<SourceFormulaCompositionHandoff, SourceFormulaCompositionError> {
        SourceFormulaCompositionProducer::build(
            input,
            &fixture.primary,
            &fixture.atomic,
            &fixture.composite,
            &fixture.arena,
        )
    }

    fn build_task_257b2(
        fixture: &Task257B2Fixture,
        input: SourceFormulaCompositionHandoffInput,
    ) -> Result<SourceFormulaCompositionHandoff, SourceFormulaCompositionError> {
        SourceFormulaCompositionProducer::build(
            input,
            &fixture.primary,
            &fixture.atomic,
            &fixture.composite,
            &fixture.arena,
        )
    }

    fn build_task_257b3(
        fixture: &Task257B3Fixture,
        input: SourceFormulaCompositionHandoffInput,
    ) -> Result<SourceFormulaCompositionHandoff, SourceFormulaCompositionError> {
        SourceFormulaCompositionProducer::build(
            input,
            &fixture.primary,
            &fixture.atomic,
            &fixture.composite,
            &fixture.arena,
        )
    }

    pub(crate) fn task_257b_installed_typed_ast() -> TypedAst {
        let fixture = task_257b2_fixture();
        let composition =
            build_task_257b2(&fixture, fixture.input.clone()).expect("Task 257B composition");
        empty_typed_ast(
            fixture.source,
            fixture.module.clone(),
            fixture.arena.clone(),
        )
        .with_source_term(fixture.primary.clone())
        .expect("Task 257B Task 252")
        .with_source_atomic_formula(fixture.atomic.clone())
        .expect("Task 257B Task 256")
        .with_source_formula_composition(fixture.composite.clone(), composition)
        .expect("Task 257B combined install")
    }

    fn debug_oracle(value: &str) -> (usize, u64, u64) {
        value
            .bytes()
            .enumerate()
            .fold((0, 0_u64, 0_u64), |(_, sum, weighted), (index, byte)| {
                (
                    index + 1,
                    sum.wrapping_add(u64::from(byte)),
                    weighted.wrapping_add((index as u64 + 1) * u64::from(byte)),
                )
            })
    }

    #[test]
    fn task_257b2_exact_composition_dependencies_edges_and_debug_publish() {
        let fixture = task_257b2_fixture();
        let first =
            build_task_257b2(&fixture, fixture.input.clone()).expect("Task 257B2 composition");
        let second = build_task_257b2(&fixture, fixture.input.clone()).expect("Task 257B2 replay");
        assert_eq!(first, second);
        assert_eq!(first.debug_text(), second.debug_text());
        assert_eq!(
            crate::source_composite_formula::tests::task_257b2_debug_oracle(&first.debug_text()),
            (15120, 6507344007032078667, 15846722718407534454),
            "Task 257B2 composition debug bytes changed"
        );
        assert_eq!(
            (
                fixture.primary.terms().len(),
                fixture.primary.references().len(),
                fixture.primary.numeric_type_requests().len(),
                fixture.atomic.formulas().len(),
                fixture.atomic.edges().len(),
                fixture.atomic.requests().len(),
                fixture.composite.formulas().len(),
                fixture.composite.wrappers().len(),
                first.atomic_edges().len(),
                first.bound_uses().len(),
            ),
            (16, 0, 16, 8, 16, 16, 8, 6, 8, 0)
        );
        assert_eq!(
            first
                .atomic_edges()
                .iter()
                .map(|(_, row)| {
                    (
                        row.formula().index(),
                        row.ordinal(),
                        row.role(),
                        row.child().index(),
                    )
                })
                .collect::<Vec<_>>(),
            [
                (3, 0, SourceFormulaAtomicEdgeRole::ConjunctionLeft, 0),
                (3, 1, SourceFormulaAtomicEdgeRole::ConjunctionRight, 1),
                (4, 0, SourceFormulaAtomicEdgeRole::DisjunctionLeft, 2),
                (4, 1, SourceFormulaAtomicEdgeRole::DisjunctionRight, 3),
                (6, 0, SourceFormulaAtomicEdgeRole::ConjunctionLeft, 4),
                (6, 1, SourceFormulaAtomicEdgeRole::ConjunctionRight, 5),
                (7, 0, SourceFormulaAtomicEdgeRole::DisjunctionLeft, 6),
                (7, 1, SourceFormulaAtomicEdgeRole::DisjunctionRight, 7),
            ]
        );
        assert_eq!(
            first.primary_term_fingerprint(),
            fixture.primary.debug_text()
        );
        assert_eq!(
            first.atomic_formula_fingerprint(),
            fixture.atomic.debug_text()
        );
        assert_eq!(
            first.composite_formula_fingerprint(),
            fixture.composite.debug_text()
        );
        assert!(first.debug_text().contains("role=conjunction-left"));
        assert!(first.debug_text().contains("role=disjunction-right"));
    }

    #[test]
    fn task_257b2_composition_and_dependency_corruptions_fail_closed_then_recover() {
        let fixture = task_257b2_fixture();
        for index in 0..8 {
            let mut input = fixture.input.clone();
            input.atomic_edges[index].formula = SourceCompositeFormulaId::new(0);
            assert!(build_task_257b2(&fixture, input).is_err());

            let mut input = fixture.input.clone();
            input.atomic_edges[index].ordinal += 1;
            assert!(build_task_257b2(&fixture, input).is_err());

            let mut input = fixture.input.clone();
            input.atomic_edges[index].role = SourceFormulaAtomicEdgeRole::UniversalBody;
            assert!(build_task_257b2(&fixture, input).is_err());

            let mut input = fixture.input.clone();
            input.atomic_edges[index].child = SourceAtomicFormulaId::new(7 - index);
            assert!(build_task_257b2(&fixture, input).is_err());
        }
        let mut reordered = fixture.input.clone();
        reordered.atomic_edges.swap(0, 1);
        assert!(build_task_257b2(&fixture, reordered).is_err());
        let mut missing = fixture.input.clone();
        missing.atomic_edges.pop();
        assert!(build_task_257b2(&fixture, missing).is_err());
        let mut extra = fixture.input.clone();
        extra.atomic_edges.push(extra.atomic_edges[0].clone());
        assert!(build_task_257b2(&fixture, extra).is_err());
        let mut fabricated_bound_use = fixture.input.clone();
        fabricated_bound_use
            .bound_uses
            .push(SourceQuantifierBoundUseInput {
                binder: SourceQuantifierBinderId::new(0),
                ordinal: 0,
                body_edge: SourceFormulaAtomicEdgeId::new(0),
                term: SourcePrimaryTermId::new(0),
                reference: SourcePrimaryTermReferenceId::new(0),
            });
        assert!(build_task_257b2(&fixture, fabricated_bound_use).is_err());

        let mut primary_input = fixture.primary_input.clone();
        primary_input.terms[3].spelling = "0".to_owned();
        primary_input.numeric_type_requests[3].spelling = "0".to_owned();
        let substituted_primary =
            SourcePrimaryTermProducer::build(primary_input, &fixture.bindings, &fixture.arena)
                .expect("structurally valid substituted primary");
        assert!(
            SourceFormulaCompositionProducer::build(
                fixture.input.clone(),
                &substituted_primary,
                &fixture.atomic,
                &fixture.composite,
                &fixture.arena,
            )
            .is_err()
        );

        let mut atomic_input = fixture.atomic_input.clone();
        atomic_input.formulas[1].spelling = "0 = 0".to_owned();
        let substituted_atomic = SourceAtomicFormulaProducer::build(
            atomic_input,
            &fixture.bindings,
            &SymbolEnv::new(fixture.module.clone(), SymbolEnvIndexes::default()),
            &substituted_primary,
            None,
            None,
            None,
            &fixture.arena,
        )
        .expect("structurally valid substituted atomic");
        assert!(
            SourceFormulaCompositionProducer::build(
                fixture.input.clone(),
                &substituted_primary,
                &substituted_atomic,
                &fixture.composite,
                &fixture.arena,
            )
            .is_err()
        );

        let b1 = self::fixture();
        assert!(
            SourceFormulaCompositionProducer::build(
                fixture.input.clone(),
                &b1.primary,
                &fixture.atomic,
                &fixture.composite,
                &fixture.arena,
            )
            .is_err()
        );
        assert!(
            build_task_257b2(&fixture, fixture.input.clone()).is_ok(),
            "valid replay recovers after rejected corruptions"
        );
    }

    #[test]
    fn task_257b2_combined_installation_ownership_and_resolved_clone_are_atomic() {
        let fixture = task_257b2_fixture();
        let composition =
            build_task_257b2(&fixture, fixture.input.clone()).expect("Task 257B2 composition");
        let base = empty_typed_ast(
            fixture.source,
            fixture.module.clone(),
            fixture.arena.clone(),
        );
        assert_eq!(
            base.clone()
                .with_source_composite_formula(fixture.composite.clone())
                .expect_err("legacy installer remains A-only"),
            TypedAstError::InvalidSourceCompositeFormula
        );
        let with_dependencies = base
            .with_source_term(fixture.primary.clone())
            .expect("Task 252 install")
            .with_source_atomic_formula(fixture.atomic.clone())
            .expect("Task 256 install");
        let installed = with_dependencies
            .clone()
            .with_source_formula_composition(fixture.composite.clone(), composition.clone())
            .expect("Task 257B2 atomic combined install");
        assert_eq!(
            installed.source_composite_formula(),
            Some(&fixture.composite)
        );
        assert_eq!(installed.source_formula_composition(), Some(&composition));
        assert_eq!(
            installed
                .clone()
                .with_source_formula_composition(fixture.composite.clone(), composition.clone())
                .expect_err("duplicate B2 install"),
            TypedAstError::InvalidSourceFormulaComposition
        );

        let b1 = self::fixture();
        let b1_composition = build(&b1, b1.input.clone()).expect("Task 257B1 composition");
        let b1_installed = empty_typed_ast(b1.source, b1.module.clone(), b1.arena.clone())
            .with_source_term(b1.primary.clone())
            .expect("B1 Task 252")
            .with_source_atomic_formula(b1.atomic.clone())
            .expect("B1 Task 256")
            .with_source_formula_composition(b1.composite.clone(), b1_composition)
            .expect("B1 composition");
        let before_b1 = b1_installed.debug_text();
        assert_eq!(
            b1_installed
                .clone()
                .with_source_formula_composition(fixture.composite.clone(), composition.clone())
                .expect_err("existing B1 ownership rejects B2"),
            TypedAstError::InvalidSourceFormulaComposition
        );
        assert_eq!(b1_installed.debug_text(), before_b1);

        let cluster_facts = ClusterFactTable::new();
        let collection = OverloadCollectionOutput::collect(
            Vec::<OverloadSiteInput>::new(),
            Vec::<OverloadCandidateInput>::new(),
        );
        let expansion = TemplateExpansionOutput::expand(&collection);
        let viability =
            CandidateViabilityOutput::filter(&expansion, Vec::<CandidateViabilityInput>::new());
        let specificity =
            SpecificityGraphOutput::build(&viability, Vec::<SpecificityComparisonInput>::new());
        let selection = OverloadSelectionOutput::resolve(
            &specificity,
            Vec::<OverloadSiteResolutionInput>::new(),
        );
        let mut orphaned = installed.clone();
        orphaned.remove_source_formula_composition_for_test();
        assert_eq!(
            ResolvedTypedAst::assemble(ResolvedTypedAstInputs {
                typed_ast: &orphaned,
                cluster_facts: &cluster_facts,
                overload_collection: &collection,
                template_expansion: &expansion,
                viability: &viability,
                specificity: &specificity,
                overload_selection: &selection,
                expressions: Vec::new(),
                node_hints: Vec::new(),
                statement_semantics: None,
                statement_proofs: None,
            })
            .expect_err("orphaned Task 257B2 composite must fail final assembly"),
            crate::resolved_typed_ast::ResolvedTypedAstError::InvalidSourceFormulaComposition
        );
        let resolved = ResolvedTypedAst::assemble(ResolvedTypedAstInputs {
            typed_ast: &installed,
            cluster_facts: &cluster_facts,
            overload_collection: &collection,
            template_expansion: &expansion,
            viability: &viability,
            specificity: &specificity,
            overload_selection: &selection,
            expressions: Vec::new(),
            node_hints: Vec::new(),
            statement_semantics: None,
            statement_proofs: None,
        })
        .expect("Task 257B2 resolved assembly");
        assert_eq!(resolved.source_formula_composition(), Some(&composition));
        assert_eq!(
            resolved.source_composite_formula(),
            Some(&fixture.composite)
        );
        assert_eq!(resolved.clone().debug_text(), resolved.debug_text());
        assert!(resolved.checked_formulas().is_empty());
        assert!(resolved.statement_semantics().is_empty());
    }

    #[test]
    fn task_257b3_exact_nested_composition_dependencies_edges_uses_and_debug_publish() {
        let fixture = task_257b3_fixture();
        let first =
            build_task_257b3(&fixture, fixture.input.clone()).expect("Task 257B3 composition");
        let second = build_task_257b3(&fixture, fixture.input.clone()).expect("Task 257B3 replay");
        assert_eq!(first, second);
        assert_eq!(first.debug_text(), second.debug_text());
        assert_eq!(
            first.debug_text(),
            include_str!("testdata/task257b3_composition.debug")
        );
        assert_eq!(
            debug_oracle(&first.debug_text()),
            (8752, 792413, 3442669489),
            "Task 257B3 composition debug bytes changed"
        );
        assert_eq!(
            (
                fixture.bindings.contexts().len(),
                fixture.bindings.bindings().len(),
                fixture.primary.terms().len(),
                fixture.primary.references().len(),
                fixture.atomic.formulas().len(),
                fixture.atomic.edges().len(),
                fixture.composite.formulas().len(),
                fixture.composite.edges().len(),
                first.atomic_edges().len(),
                first.bound_uses().len(),
            ),
            (4, 4, 6, 6, 3, 6, 3, 2, 3, 6)
        );
        assert_eq!(
            first
                .atomic_edges()
                .iter()
                .map(|(_, row)| {
                    (
                        row.formula().index(),
                        row.ordinal(),
                        row.role(),
                        row.child().index(),
                    )
                })
                .collect::<Vec<_>>(),
            [
                (0, 0, SourceFormulaAtomicEdgeRole::UniversalRestriction, 0),
                (2, 0, SourceFormulaAtomicEdgeRole::UniversalRestriction, 1),
                (2, 1, SourceFormulaAtomicEdgeRole::UniversalBody, 2),
            ]
        );
        assert_eq!(
            first
                .bound_uses()
                .iter()
                .map(|(_, row)| {
                    (
                        row.binder().index(),
                        row.ordinal(),
                        row.body_edge().index(),
                        row.term().index(),
                        row.reference().index(),
                    )
                })
                .collect::<Vec<_>>(),
            [
                (0, 0, 0, 0, 0),
                (0, 1, 0, 1, 1),
                (2, 0, 1, 2, 2),
                (1, 0, 1, 3, 3),
                (0, 2, 2, 4, 4),
                (2, 1, 2, 5, 5),
            ]
        );
        assert!(first.debug_text().contains("role=universal-restriction"));
        assert!(first.debug_text().contains("body-edge=2"));
    }

    #[test]
    fn task_257b3_composition_and_dependency_corruptions_fail_closed_then_recover() {
        let fixture = task_257b3_fixture();
        for index in 0..3 {
            let mut input = fixture.input.clone();
            input.atomic_edges[index].formula = SourceCompositeFormulaId::new(1);
            assert!(build_task_257b3(&fixture, input).is_err());

            let mut input = fixture.input.clone();
            input.atomic_edges[index].ordinal += 1;
            assert!(build_task_257b3(&fixture, input).is_err());

            let mut input = fixture.input.clone();
            input.atomic_edges[index].role = SourceFormulaAtomicEdgeRole::ConjunctionLeft;
            assert!(build_task_257b3(&fixture, input).is_err());

            let mut input = fixture.input.clone();
            input.atomic_edges[index].child = SourceAtomicFormulaId::new((index + 1) % 3);
            assert!(build_task_257b3(&fixture, input).is_err());
        }
        for index in 0..6 {
            let mut input = fixture.input.clone();
            input.bound_uses[index].binder =
                SourceQuantifierBinderId::new((input.bound_uses[index].binder.index() + 1) % 3);
            assert!(build_task_257b3(&fixture, input).is_err());

            let mut input = fixture.input.clone();
            input.bound_uses[index].ordinal += 1;
            assert!(build_task_257b3(&fixture, input).is_err());

            let mut input = fixture.input.clone();
            input.bound_uses[index].body_edge = SourceFormulaAtomicEdgeId::new((index / 2 + 1) % 3);
            assert!(build_task_257b3(&fixture, input).is_err());

            let mut input = fixture.input.clone();
            input.bound_uses[index].term = SourcePrimaryTermId::new((index + 1) % 6);
            assert!(build_task_257b3(&fixture, input).is_err());

            let mut input = fixture.input.clone();
            input.bound_uses[index].reference = SourcePrimaryTermReferenceId::new((index + 1) % 6);
            assert!(build_task_257b3(&fixture, input).is_err());
        }
        let mut reordered = fixture.input.clone();
        reordered.atomic_edges.swap(0, 1);
        assert!(build_task_257b3(&fixture, reordered).is_err());
        let mut missing = fixture.input.clone();
        missing.bound_uses.pop();
        assert!(build_task_257b3(&fixture, missing).is_err());
        let mut extra = fixture.input.clone();
        extra.bound_uses.push(extra.bound_uses[0].clone());
        assert!(build_task_257b3(&fixture, extra).is_err());

        let mut primary_input = fixture.primary_input.clone();
        primary_input.terms[0].context = BindingContextId::new(3);
        let substituted_primary =
            SourcePrimaryTermProducer::build(primary_input, &fixture.bindings, &fixture.arena)
                .expect("structurally valid use-context substitution");
        assert!(
            SourceFormulaCompositionProducer::build(
                fixture.input.clone(),
                &substituted_primary,
                &fixture.atomic,
                &fixture.composite,
                &fixture.arena,
            )
            .is_err()
        );

        let mut shifted_nodes = fixture
            .arena
            .iter()
            .map(|(id, node)| {
                let mut node = node.clone();
                node.anchor = match id.index() {
                    15 => SourceAnchor::Range(range(fixture.source, 112, 117)),
                    18 => SourceAnchor::Range(range(fixture.source, 112, 113)),
                    19 => SourceAnchor::Range(range(fixture.source, 116, 117)),
                    _ => node.anchor,
                };
                node
            })
            .collect::<Vec<_>>();
        let shifted_arena =
            TypedArena::try_new(None, std::mem::take(&mut shifted_nodes)).expect("shifted arena");
        let mut shifted_primary_input = fixture.primary_input.clone();
        shifted_primary_input.terms[0].source_range = range(fixture.source, 112, 113);
        shifted_primary_input.terms[1].source_range = range(fixture.source, 116, 117);
        let shifted_primary = SourcePrimaryTermProducer::build(
            shifted_primary_input,
            &fixture.bindings,
            &shifted_arena,
        )
        .expect("coherent shifted primary terms");
        let mut shifted_atomic_input = fixture.atomic_input.clone();
        shifted_atomic_input.formulas[0].source_range = range(fixture.source, 112, 117);
        let shifted_atomic = SourceAtomicFormulaProducer::build(
            shifted_atomic_input,
            &fixture.bindings,
            &SymbolEnv::new(fixture.module.clone(), SymbolEnvIndexes::default()),
            &shifted_primary,
            None,
            None,
            None,
            &shifted_arena,
        )
        .expect("coherent shifted atomic formula");
        assert!(
            SourceFormulaCompositionProducer::build(
                fixture.input.clone(),
                &shifted_primary,
                &shifted_atomic,
                &fixture.composite,
                &shifted_arena,
            )
            .is_err(),
            "atom moved beneath a deeper composite owner must not remain assigned to the outer formula"
        );

        let task_257b1 = self::fixture();
        assert!(
            SourceFormulaCompositionProducer::build(
                fixture.input.clone(),
                &fixture.primary,
                &task_257b1.atomic,
                &fixture.composite,
                &fixture.arena,
            )
            .is_err()
        );
        assert!(
            build_task_257b3(&fixture, fixture.input.clone()).is_ok(),
            "valid replay recovers after every rejected corruption"
        );
    }

    #[test]
    fn task_257b3_combined_installation_ownership_and_resolved_clone_are_atomic() {
        let fixture = task_257b3_fixture();
        let composition =
            build_task_257b3(&fixture, fixture.input.clone()).expect("Task 257B3 composition");
        let base = empty_typed_ast(
            fixture.source,
            fixture.module.clone(),
            fixture.arena.clone(),
        );
        assert_eq!(
            base.clone()
                .with_source_composite_formula(fixture.composite.clone())
                .expect_err("legacy installer rejects B3"),
            TypedAstError::InvalidSourceCompositeFormula
        );
        assert_combined_install_rejected_without_publication(
            &base,
            fixture.composite.clone(),
            composition.clone(),
        );
        let with_primary = base
            .with_source_term(fixture.primary.clone())
            .expect("Task 252 install");
        assert_combined_install_rejected_without_publication(
            &with_primary,
            fixture.composite.clone(),
            composition.clone(),
        );
        let with_atomic = with_primary
            .with_source_atomic_formula(fixture.atomic.clone())
            .expect("Task 256 install");
        let installed = with_atomic
            .clone()
            .with_source_formula_composition(fixture.composite.clone(), composition.clone())
            .expect("Task 257B3 atomic combined install");
        assert_eq!(
            installed.source_composite_formula(),
            Some(&fixture.composite)
        );
        assert_eq!(installed.source_formula_composition(), Some(&composition));
        assert_combined_install_rejected_preserving_state(
            &installed,
            fixture.composite.clone(),
            composition.clone(),
        );
        for mutate in [
            |candidate: &mut SourceFormulaCompositionHandoff| {
                candidate.primary_term_fingerprint.push_str("stale")
            },
            |candidate: &mut SourceFormulaCompositionHandoff| {
                candidate.atomic_formula_fingerprint.push_str("stale")
            },
            |candidate: &mut SourceFormulaCompositionHandoff| {
                candidate.composite_formula_fingerprint.push_str("stale")
            },
            |candidate: &mut SourceFormulaCompositionHandoff| {
                candidate.atomic_edges.rows.swap(0, 1)
            },
            |candidate: &mut SourceFormulaCompositionHandoff| candidate.bound_uses.rows.swap(0, 1),
        ] {
            let mut candidate = composition.clone();
            mutate(&mut candidate);
            assert_combined_install_rejected_without_publication(
                &with_atomic,
                fixture.composite.clone(),
                candidate,
            );
        }

        let task_248 = crate::source_context::tests::task_248_occupied_typed_ast(
            fixture.source,
            fixture.module.clone(),
        );
        assert!(task_248.source_context().is_some());
        assert_combined_install_rejected_without_publication(
            &task_248,
            fixture.composite.clone(),
            composition.clone(),
        );
        let task_257a = crate::source_composite_formula::tests::task_257a_installed_typed_ast();
        assert_combined_install_rejected_preserving_state(
            &task_257a,
            fixture.composite.clone(),
            composition.clone(),
        );
        let task_257b1 = self::fixture();
        let task_257b1_composition =
            build(&task_257b1, task_257b1.input.clone()).expect("Task 257B1 composition");
        let task_257b1_installed = empty_typed_ast(
            task_257b1.source,
            task_257b1.module.clone(),
            task_257b1.arena.clone(),
        )
        .with_source_term(task_257b1.primary.clone())
        .expect("Task 257B1 term")
        .with_source_atomic_formula(task_257b1.atomic.clone())
        .expect("Task 257B1 atomic")
        .with_source_formula_composition(task_257b1.composite.clone(), task_257b1_composition)
        .expect("Task 257B1 install");
        assert_combined_install_rejected_preserving_state(
            &task_257b1_installed,
            fixture.composite.clone(),
            composition.clone(),
        );
        let task_257b2 = task_257b2_fixture();
        let task_257b2_composition = build_task_257b2(&task_257b2, task_257b2.input.clone())
            .expect("Task 257B2 composition");
        let task_257b2_installed = empty_typed_ast(
            task_257b2.source,
            task_257b2.module.clone(),
            task_257b2.arena.clone(),
        )
        .with_source_term(task_257b2.primary.clone())
        .expect("Task 257B2 term")
        .with_source_atomic_formula(task_257b2.atomic.clone())
        .expect("Task 257B2 atomic")
        .with_source_formula_composition(task_257b2.composite.clone(), task_257b2_composition)
        .expect("Task 257B2 install");
        assert_combined_install_rejected_preserving_state(
            &task_257b2_installed,
            fixture.composite.clone(),
            composition.clone(),
        );

        let cluster_facts = ClusterFactTable::new();
        let collection = OverloadCollectionOutput::collect(
            Vec::<OverloadSiteInput>::new(),
            Vec::<OverloadCandidateInput>::new(),
        );
        let expansion = TemplateExpansionOutput::expand(&collection);
        let viability =
            CandidateViabilityOutput::filter(&expansion, Vec::<CandidateViabilityInput>::new());
        let specificity =
            SpecificityGraphOutput::build(&viability, Vec::<SpecificityComparisonInput>::new());
        let selection = OverloadSelectionOutput::resolve(
            &specificity,
            Vec::<OverloadSiteResolutionInput>::new(),
        );
        let mut orphaned = installed.clone();
        orphaned.remove_source_formula_composition_for_test();
        assert_eq!(
            ResolvedTypedAst::assemble(ResolvedTypedAstInputs {
                typed_ast: &orphaned,
                cluster_facts: &cluster_facts,
                overload_collection: &collection,
                template_expansion: &expansion,
                viability: &viability,
                specificity: &specificity,
                overload_selection: &selection,
                expressions: Vec::new(),
                node_hints: Vec::new(),
                statement_semantics: None,
                statement_proofs: None,
            })
            .expect_err("orphaned Task 257B3 composite must fail final assembly"),
            crate::resolved_typed_ast::ResolvedTypedAstError::InvalidSourceFormulaComposition
        );
        let resolved = ResolvedTypedAst::assemble(ResolvedTypedAstInputs {
            typed_ast: &installed,
            cluster_facts: &cluster_facts,
            overload_collection: &collection,
            template_expansion: &expansion,
            viability: &viability,
            specificity: &specificity,
            overload_selection: &selection,
            expressions: Vec::new(),
            node_hints: Vec::new(),
            statement_semantics: None,
            statement_proofs: None,
        })
        .expect("Task 257B3 resolved assembly");
        assert_eq!(resolved.source_formula_composition(), Some(&composition));
        assert_eq!(
            resolved.source_composite_formula(),
            Some(&fixture.composite)
        );
        assert_eq!(resolved.clone().debug_text(), resolved.debug_text());
        assert!(resolved.checked_formulas().is_empty());
        assert!(resolved.statement_semantics().is_empty());
    }

    #[test]
    fn second_composite_profile_and_exact_composition_publish() {
        let fixture = fixture();
        assert!(fixture.composite.is_task_257b1_profile());
        assert_eq!(
            (
                fixture.composite.formulas().len(),
                fixture.composite.wrappers().len(),
                fixture.composite.roots().len(),
                fixture.composite.binders().len(),
                fixture.composite.type_sites().len(),
                fixture.composite.edges().len(),
                fixture.composite.requests().len(),
            ),
            (1, 0, 1, 1, 1, 0, 2)
        );
        let handoff = build(&fixture, fixture.input.clone()).expect("composition");
        assert_eq!(
            (handoff.atomic_edges().len(), handoff.bound_uses().len()),
            (1, 2)
        );
        assert_eq!(
            handoff.primary_term_fingerprint(),
            fixture.primary.debug_text()
        );
        assert_eq!(
            handoff.atomic_formula_fingerprint(),
            fixture.atomic.debug_text()
        );
        assert_eq!(
            handoff.composite_formula_fingerprint(),
            fixture.composite.debug_text()
        );
        assert_eq!(
            handoff
                .bound_uses()
                .iter()
                .map(|(_, row)| (row.ordinal(), row.term().index(), row.reference().index()))
                .collect::<Vec<_>>(),
            vec![(0, 0, 0), (1, 1, 1)]
        );
    }

    #[test]
    fn debug_is_deterministic_and_has_the_complete_schema() {
        let fixture = fixture();
        let first = build(&fixture, fixture.input.clone()).expect("first");
        let second = build(&fixture, fixture.input.clone()).expect("second");
        assert_eq!(first, second);
        assert_eq!(first.debug_text(), second.debug_text());
        assert_eq!(first.debug_text(), EXPECTED_DEBUG);
    }

    #[test]
    fn every_composition_field_and_association_corruption_rejects() {
        type InputMutation = Box<dyn Fn(&mut SourceFormulaCompositionHandoffInput)>;

        let fixture = fixture();
        let mutations: Vec<InputMutation> = vec![
            Box::new(|input| input.source_id = other_source_id()),
            Box::new(|input| {
                input.module_id = ModuleId::new(
                    PackageId::new("other"),
                    ModulePath::new("composition.other"),
                )
            }),
            Box::new(|input| input.atomic_edges[0].formula = SourceCompositeFormulaId::new(1)),
            Box::new(|input| input.atomic_edges[0].ordinal = 1),
            Box::new(|input| input.atomic_edges[0].child = SourceAtomicFormulaId::new(1)),
            Box::new(|input| input.bound_uses[0].binder = SourceQuantifierBinderId::new(1)),
            Box::new(|input| input.bound_uses[0].ordinal = 1),
            Box::new(|input| input.bound_uses[0].body_edge = SourceFormulaAtomicEdgeId::new(1)),
            Box::new(|input| input.bound_uses[0].term = SourcePrimaryTermId::new(1)),
            Box::new(|input| input.bound_uses[0].reference = SourcePrimaryTermReferenceId::new(1)),
            Box::new(|input| input.bound_uses.swap(0, 1)),
            Box::new(|input| {
                input.bound_uses[1].term = SourcePrimaryTermId::new(0);
                input.bound_uses[1].reference = SourcePrimaryTermReferenceId::new(0);
            }),
        ];
        for (index, mutate) in mutations.into_iter().enumerate() {
            let mut input = fixture.input.clone();
            mutate(&mut input);
            assert!(
                build(&fixture, input).is_err(),
                "composition corruption #{index} was accepted"
            );
        }
        let mut missing_edge = fixture.input.clone();
        missing_edge.atomic_edges.clear();
        assert!(build(&fixture, missing_edge).is_err());
        let mut missing_use = fixture.input.clone();
        missing_use.bound_uses.pop();
        assert!(build(&fixture, missing_use).is_err());
        let mut extra_use = fixture.input.clone();
        extra_use.bound_uses.push(extra_use.bound_uses[1].clone());
        assert!(build(&fixture, extra_use).is_err());
        assert!(matches!(
            fixture.input.atomic_edges[0].role,
            SourceFormulaAtomicEdgeRole::UniversalBody
        ));
    }

    #[test]
    fn profile_hybrids_and_third_shapes_reject() {
        let fixture = fixture();
        for mutate in [
            |input: &mut SourceCompositeFormulaHandoffInput| input.formulas.clear(),
            |input: &mut SourceCompositeFormulaHandoffInput| input.requests.clear(),
            |input: &mut SourceCompositeFormulaHandoffInput| {
                input.requests.push(input.requests[0].clone())
            },
        ] {
            let mut input = fixture.composite_input.clone();
            mutate(&mut input);
            assert!(
                SourceCompositeFormulaProducer::extend_bindings(
                    &input,
                    &base_bindings(fixture.source, &fixture.module),
                    &fixture.arena,
                )
                .is_err()
            );
        }

        let mut b1_rows_at_a_cardinality = fixture.composite_input.clone();
        b1_rows_at_a_cardinality.formulas = (0..5)
            .map(|ordinal| {
                let mut row = b1_rows_at_a_cardinality.formulas[0].clone();
                row.source_ordinal = ordinal;
                row
            })
            .collect();
        b1_rows_at_a_cardinality.edges = (0..4)
            .map(|ordinal| SourceFormulaEdgeInput {
                parent: SourceCompositeFormulaId::new(0),
                ordinal,
                role: if ordinal == 0 {
                    SourceFormulaEdgeRole::ImplicationLeft
                } else {
                    SourceFormulaEdgeRole::ImplicationRight
                },
                child: SourceCompositeFormulaId::new(0),
            })
            .collect();
        b1_rows_at_a_cardinality.requests = (0..6)
            .map(|ordinal| {
                let mut row = b1_rows_at_a_cardinality.requests[0].clone();
                row.ordinal = ordinal;
                row
            })
            .collect();
        assert_eq!(
            (
                b1_rows_at_a_cardinality.formulas.len(),
                b1_rows_at_a_cardinality.wrappers.len(),
                b1_rows_at_a_cardinality.roots.len(),
                b1_rows_at_a_cardinality.binders.len(),
                b1_rows_at_a_cardinality.type_sites.len(),
                b1_rows_at_a_cardinality.edges.len(),
                b1_rows_at_a_cardinality.requests.len(),
            ),
            (5, 0, 1, 1, 1, 4, 6)
        );
        assert!(
            SourceCompositeFormulaProducer::extend_bindings(
                &b1_rows_at_a_cardinality,
                &base_bindings(fixture.source, &fixture.module),
                &fixture.arena,
            )
            .is_err()
        );

        let mut otherwise_valid_third_profile = fixture.composite_input.clone();
        otherwise_valid_third_profile
            .wrappers
            .push(SourceFormulaWrapperInput {
                formula: SourceCompositeFormulaId::new(0),
                ordinal: 0,
                site: node(7),
                source_range: range(fixture.source, 50, 77),
                context: BindingContextId::new(0),
                recovery: SourceCompositeFormulaRecovery::Normal,
                spelling: "(for holds)".to_owned(),
            });
        assert!(
            SourceCompositeFormulaProducer::extend_bindings(
                &otherwise_valid_third_profile,
                &base_bindings(fixture.source, &fixture.module),
                &fixture.arena,
            )
            .is_err()
        );
    }

    #[test]
    fn combined_installation_is_atomic_and_legacy_installer_rejects_b1() {
        let fixture = fixture();
        let composition = build(&fixture, fixture.input.clone()).expect("composition");
        let base = empty_typed_ast(
            fixture.source,
            fixture.module.clone(),
            fixture.arena.clone(),
        );
        assert_eq!(
            base.clone()
                .with_source_composite_formula(fixture.composite.clone())
                .expect_err("legacy installer rejects B1"),
            TypedAstError::InvalidSourceCompositeFormula
        );
        let with_term = base
            .with_source_term(fixture.primary.clone())
            .expect("term install");
        let before_missing = with_term.debug_text();
        assert_eq!(
            with_term
                .clone()
                .with_source_formula_composition(fixture.composite.clone(), composition.clone(),)
                .expect_err("atomic dependency is required"),
            TypedAstError::InvalidSourceFormulaComposition
        );
        assert_eq!(with_term.debug_text(), before_missing);

        let with_atomic = with_term
            .with_source_atomic_formula(fixture.atomic.clone())
            .expect("atomic install");
        let installed = with_atomic
            .clone()
            .with_source_formula_composition(fixture.composite.clone(), composition.clone())
            .expect("combined install");
        assert_eq!(
            installed.source_composite_formula(),
            Some(&fixture.composite)
        );
        assert_eq!(installed.source_formula_composition(), Some(&composition));
        assert!(installed.source_context().is_none());
        assert!(
            installed
                .debug_text()
                .find("source-atomic-formula-debug-v1")
                .is_some_and(|atomic| {
                    let composite = installed
                        .debug_text()
                        .find("source-composite-formula-debug-v1")
                        .expect("composite debug");
                    let composition = installed
                        .debug_text()
                        .find("source-formula-composition-debug-v1")
                        .expect("composition debug");
                    atomic < composite && composite < composition
                })
        );
        assert_eq!(
            installed
                .clone()
                .with_source_formula_composition(fixture.composite.clone(), composition)
                .expect_err("duplicate combined install"),
            TypedAstError::InvalidSourceFormulaComposition
        );

        let mut stale = build(&fixture, fixture.input.clone()).expect("stale candidate");
        stale.primary_term_fingerprint.push_str("stale");
        let before_stale = with_atomic.debug_text();
        assert_eq!(
            with_atomic
                .clone()
                .with_source_formula_composition(fixture.composite.clone(), stale)
                .expect_err("stale dependency"),
            TypedAstError::InvalidSourceFormulaComposition
        );
        assert_eq!(with_atomic.debug_text(), before_stale);

        for mutate in [
            |candidate: &mut SourceFormulaCompositionHandoff| {
                candidate.primary_term_fingerprint.push_str("substituted")
            },
            |candidate: &mut SourceFormulaCompositionHandoff| {
                candidate.atomic_formula_fingerprint.push_str("substituted")
            },
            |candidate: &mut SourceFormulaCompositionHandoff| {
                candidate
                    .composite_formula_fingerprint
                    .push_str("substituted")
            },
            |candidate: &mut SourceFormulaCompositionHandoff| candidate.bound_uses.rows.swap(0, 1),
        ] {
            let mut candidate = build(&fixture, fixture.input.clone()).expect("candidate");
            mutate(&mut candidate);
            assert_combined_install_rejected_without_publication(
                &with_atomic,
                fixture.composite.clone(),
                candidate,
            );
        }

        let task_248 = crate::source_context::tests::task_248_occupied_typed_ast(
            fixture.source,
            fixture.module.clone(),
        );
        assert!(task_248.source_context().is_some());
        assert_combined_install_rejected_without_publication(
            &task_248,
            fixture.composite.clone(),
            build(&fixture, fixture.input.clone()).expect("Task 248 candidate"),
        );

        let task_257a = crate::source_composite_formula::tests::task_257a_installed_typed_ast();
        let before_task_257a = task_257a.debug_text();
        let original_task_257a = task_257a
            .source_composite_formula()
            .expect("Task 257A is installed")
            .clone();
        assert_eq!(
            task_257a
                .clone()
                .with_source_formula_composition(
                    fixture.composite.clone(),
                    build(&fixture, fixture.input.clone()).expect("Task 257A candidate"),
                )
                .expect_err("Task 257A occupancy rejects B1"),
            TypedAstError::InvalidSourceFormulaComposition
        );
        assert_eq!(task_257a.debug_text(), before_task_257a);
        assert_eq!(
            task_257a.source_composite_formula(),
            Some(&original_task_257a)
        );
        assert!(task_257a.source_formula_composition().is_none());
    }

    #[test]
    fn resolved_assembly_revalidates_and_clone_preserves_composition() {
        let fixture = fixture();
        let composition = build(&fixture, fixture.input.clone()).expect("composition");
        let typed = empty_typed_ast(
            fixture.source,
            fixture.module.clone(),
            fixture.arena.clone(),
        )
        .with_source_term(fixture.primary.clone())
        .expect("term install")
        .with_source_atomic_formula(fixture.atomic.clone())
        .expect("atomic install")
        .with_source_formula_composition(fixture.composite.clone(), composition.clone())
        .expect("composition install");
        let cluster_facts = ClusterFactTable::new();
        let collection = OverloadCollectionOutput::collect(
            Vec::<OverloadSiteInput>::new(),
            Vec::<OverloadCandidateInput>::new(),
        );
        let expansion = TemplateExpansionOutput::expand(&collection);
        let viability =
            CandidateViabilityOutput::filter(&expansion, Vec::<CandidateViabilityInput>::new());
        let specificity =
            SpecificityGraphOutput::build(&viability, Vec::<SpecificityComparisonInput>::new());
        let selection = OverloadSelectionOutput::resolve(
            &specificity,
            Vec::<OverloadSiteResolutionInput>::new(),
        );
        let resolved = ResolvedTypedAst::assemble(ResolvedTypedAstInputs {
            typed_ast: &typed,
            cluster_facts: &cluster_facts,
            overload_collection: &collection,
            template_expansion: &expansion,
            viability: &viability,
            specificity: &specificity,
            overload_selection: &selection,
            expressions: Vec::new(),
            node_hints: Vec::new(),
            statement_semantics: None,
            statement_proofs: None,
        })
        .expect("resolved assembly");
        assert_eq!(resolved.source_formula_composition(), Some(&composition));
        assert_eq!(
            resolved.source_composite_formula(),
            Some(&fixture.composite)
        );
        let debug = resolved.debug_text();
        let atomic = debug
            .find("source-atomic-formula-debug-v1")
            .expect("atomic debug");
        let composite = debug
            .find("source-composite-formula-debug-v1")
            .expect("composite debug");
        let composition_debug = debug
            .find("source-formula-composition-debug-v1")
            .expect("composition debug");
        assert!(atomic < composite && composite < composition_debug);
    }

    struct Task257C2Fixture {
        lower: crate::source_atomic_formula::tests::ConditionContainerFixture,
        set: SourceSetTermHandoff,
        atomic: SourceAtomicFormulaHandoff,
        input: SourceConditionFormulaCompositionHandoffInput,
    }

    fn task_257c2_fixture() -> Task257C2Fixture {
        let lower = crate::source_atomic_formula::tests::exact_condition_container_fixture();
        let set = lower
            .build_set(lower.set_input.clone())
            .expect("Task 257C2 set handoff");
        let atomic = lower
            .build_atomic(lower.atomic_input.clone(), Some(&set))
            .expect("Task 257C2 atomic handoff");
        let input = SourceConditionFormulaCompositionHandoffInput {
            source_id: lower.source,
            module_id: lower.module.clone(),
            edges: vec![SourceConditionFormulaEdgeInput {
                condition: SourceSetConditionId::new(0),
                ordinal: 0,
                formula: SourceAtomicFormulaId::new(0),
            }],
        };
        Task257C2Fixture {
            lower,
            set,
            atomic,
            input,
        }
    }

    fn build_task_257c2(
        fixture: &Task257C2Fixture,
        input: SourceConditionFormulaCompositionHandoffInput,
    ) -> Result<SourceConditionFormulaCompositionHandoff, SourceConditionFormulaCompositionError>
    {
        SourceConditionFormulaCompositionProducer::build(
            input,
            &fixture.lower.primary,
            &fixture.lower.application,
            &fixture.set,
            &fixture.atomic,
            &fixture.lower.arena,
        )
    }

    pub(crate) fn task_257c2_installed_typed_ast() -> TypedAst {
        let fixture = task_257c2_fixture();
        let composition =
            build_task_257c2(&fixture, fixture.input.clone()).expect("Task 257C2 composition");
        fixture
            .lower
            .typed_ast()
            .with_source_set_term(fixture.set.clone())
            .expect("Task 257C2 Task 255")
            .with_source_atomic_formula(fixture.atomic.clone())
            .expect("Task 257C2 Task 256")
            .with_source_condition_formula_composition(composition)
            .expect("Task 257C2 install")
    }

    #[test]
    fn task_257c2_exact_association_dependencies_accessors_and_debug_publish() {
        let fixture = task_257c2_fixture();
        let handoff =
            build_task_257c2(&fixture, fixture.input.clone()).expect("Task 257C2 handoff");
        let replay = build_task_257c2(&fixture, fixture.input.clone()).expect("Task 257C2 replay");
        assert_eq!(handoff.source_id(), fixture.lower.source);
        assert_eq!(handoff.module_id(), &fixture.lower.module);
        assert_eq!(
            handoff.primary_term_fingerprint(),
            fixture.lower.primary.debug_text()
        );
        assert_eq!(
            handoff.application_fingerprint(),
            fixture.lower.application.debug_text()
        );
        assert_eq!(handoff.set_term_fingerprint(), fixture.set.debug_text());
        assert_eq!(
            handoff.atomic_formula_fingerprint(),
            fixture.atomic.debug_text()
        );
        assert_eq!(handoff.edges().len(), 1);
        assert!(!handoff.edges().is_empty());
        let edge = handoff
            .edges()
            .get(SourceConditionFormulaEdgeId::new(0))
            .expect("Task 257C2 edge");
        assert_eq!(edge.condition(), SourceSetConditionId::new(0));
        assert_eq!(edge.ordinal(), 0);
        assert_eq!(edge.formula(), SourceAtomicFormulaId::new(0));
        assert_eq!(
            handoff.edges().iter().collect::<Vec<_>>(),
            [(SourceConditionFormulaEdgeId::new(0), edge)]
        );
        let debug = handoff.debug_text();
        assert_eq!(debug, replay.debug_text());
        assert_eq!(
            debug,
            format!(
                concat!(
                    "source-condition-formula-composition-debug-v1\n",
                    "module: {}::{}\n",
                    "primary-term-fingerprint: {:?}\n",
                    "application-fingerprint: {:?}\n",
                    "set-term-fingerprint: {:?}\n",
                    "atomic-formula-fingerprint: {:?}\n",
                    "edges: 1\n",
                    "  edge#0 condition=0 ordinal=0 formula=0\n",
                ),
                fixture.lower.module.package().as_str(),
                fixture.lower.module.path().as_str(),
                fixture.lower.primary.debug_text(),
                fixture.lower.application.debug_text(),
                fixture.set.debug_text(),
                fixture.atomic.debug_text(),
            )
        );
    }

    #[test]
    fn task_257c2_edge_dependency_and_profile_corruptions_fail_closed() {
        let fixture = task_257c2_fixture();
        for mutate in [
            |input: &mut SourceConditionFormulaCompositionHandoffInput| input.edges.clear(),
            |input: &mut SourceConditionFormulaCompositionHandoffInput| {
                input.edges.push(input.edges[0].clone())
            },
            |input: &mut SourceConditionFormulaCompositionHandoffInput| {
                input.edges[0].condition = SourceSetConditionId::new(1)
            },
            |input: &mut SourceConditionFormulaCompositionHandoffInput| input.edges[0].ordinal = 1,
            |input: &mut SourceConditionFormulaCompositionHandoffInput| {
                input.edges[0].formula = SourceAtomicFormulaId::new(1)
            },
        ] {
            let mut input = fixture.input.clone();
            mutate(&mut input);
            assert!(build_task_257c2(&fixture, input).is_err());
            assert!(build_task_257c2(&fixture, fixture.input.clone()).is_ok());
        }

        let condition_site = fixture
            .set
            .conditions()
            .get(SourceSetConditionId::new(0))
            .expect("Task 257C2 condition")
            .condition_site()
            .node();
        let formula_site = fixture
            .atomic
            .formulas()
            .get(SourceAtomicFormulaId::new(0))
            .expect("Task 257C2 formula")
            .site()
            .node();
        let mut nodes = fixture
            .lower
            .arena
            .iter()
            .map(|(_, node)| node.clone())
            .collect::<Vec<_>>();
        nodes[condition_site.index()]
            .children
            .retain(|child| *child != formula_site);
        let stale_arena =
            TypedArena::try_new(fixture.lower.arena.root(), nodes).expect("stale arena");
        assert_eq!(
            SourceConditionFormulaCompositionProducer::build(
                fixture.input.clone(),
                &fixture.lower.primary,
                &fixture.lower.application,
                &fixture.set,
                &fixture.atomic,
                &stale_arena,
            ),
            Err(SourceConditionFormulaCompositionError::DependencyMismatch)
        );
        assert!(build_task_257c2(&fixture, fixture.input.clone()).is_ok());

        let legacy = self::fixture();
        assert!(
            SourceConditionFormulaCompositionProducer::build(
                fixture.input.clone(),
                &legacy.primary,
                &fixture.lower.application,
                &fixture.set,
                &fixture.atomic,
                &fixture.lower.arena,
            )
            .is_err()
        );
        let mut stale = build_task_257c2(&fixture, fixture.input.clone()).expect("stale candidate");
        stale.atomic_formula_fingerprint.push_str("stale");
        let base = fixture
            .lower
            .typed_ast()
            .with_source_set_term(fixture.set.clone())
            .expect("Task 255 install")
            .with_source_atomic_formula(fixture.atomic.clone())
            .expect("Task 256 install");
        let before = base.debug_text();
        assert_eq!(
            base.clone()
                .with_source_condition_formula_composition(stale)
                .expect_err("stale C2 handoff"),
            TypedAstError::InvalidSourceConditionFormulaComposition
        );
        assert_eq!(base.debug_text(), before);
    }

    #[test]
    fn task_257c2_installation_exclusion_and_resolved_clone_are_atomic() {
        let fixture = task_257c2_fixture();
        let composition =
            build_task_257c2(&fixture, fixture.input.clone()).expect("Task 257C2 composition");
        let base = fixture.lower.typed_ast();
        assert_eq!(
            base.clone()
                .with_source_condition_formula_composition(composition.clone())
                .expect_err("missing Task 255/256 dependencies"),
            TypedAstError::InvalidSourceConditionFormulaComposition
        );
        let with_set = base
            .with_source_set_term(fixture.set.clone())
            .expect("Task 255 install");
        assert_eq!(
            with_set
                .clone()
                .with_source_condition_formula_composition(composition.clone())
                .expect_err("missing Task 256 dependency"),
            TypedAstError::InvalidSourceConditionFormulaComposition
        );
        let with_atomic = with_set
            .with_source_atomic_formula(fixture.atomic.clone())
            .expect("Task 256 install");
        let installed = with_atomic
            .clone()
            .with_source_condition_formula_composition(composition.clone())
            .expect("Task 257C2 install");
        assert_eq!(
            installed.source_condition_formula_composition(),
            Some(&composition)
        );
        let before = installed.debug_text();
        assert_eq!(
            installed
                .clone()
                .with_source_condition_formula_composition(composition.clone())
                .expect_err("duplicate Task 257C2"),
            TypedAstError::InvalidSourceConditionFormulaComposition
        );
        assert_eq!(installed.debug_text(), before);

        let reverse_installed = fixture
            .lower
            .typed_ast()
            .with_source_atomic_formula(fixture.atomic.clone())
            .expect("Task 256 reverse-order install")
            .with_source_set_term(fixture.set.clone())
            .expect("Task 255 reverse-order install")
            .with_source_condition_formula_composition(composition.clone())
            .expect("Task 257C2 reverse lower-order install");
        assert_eq!(
            reverse_installed.source_condition_formula_composition(),
            Some(&composition)
        );

        let b1 = self::fixture();
        let b1_composition = build(&b1, b1.input.clone()).expect("Task 257B1 composition");
        let installed_before_b1 = installed.debug_text();
        assert_eq!(
            installed
                .clone()
                .with_source_formula_composition(b1.composite.clone(), b1_composition.clone())
                .expect_err("Task 257B after C2"),
            TypedAstError::InvalidSourceFormulaComposition
        );
        assert_eq!(installed.debug_text(), installed_before_b1);
        let b1_installed = empty_typed_ast(b1.source, b1.module.clone(), b1.arena.clone())
            .with_source_term(b1.primary.clone())
            .expect("Task 257B1 primary")
            .with_source_atomic_formula(b1.atomic.clone())
            .expect("Task 257B1 atomic")
            .with_source_formula_composition(b1.composite.clone(), b1_composition)
            .expect("Task 257B1 install");
        let b1_before_c2 = b1_installed.debug_text();
        assert_eq!(
            b1_installed
                .clone()
                .with_source_condition_formula_composition(composition.clone())
                .expect_err("C2 after Task 257B"),
            TypedAstError::InvalidSourceConditionFormulaComposition
        );
        assert_eq!(b1_installed.debug_text(), b1_before_c2);
        let task_257a = crate::source_composite_formula::tests::task_257a_installed_typed_ast();
        let task_257a_handoff = task_257a
            .source_composite_formula()
            .expect("Task 257A handoff")
            .clone();
        let task_257a_before_c2 = task_257a.debug_text();
        assert_eq!(
            task_257a
                .clone()
                .with_source_condition_formula_composition(composition.clone())
                .expect_err("C2 after Task 257A"),
            TypedAstError::InvalidSourceConditionFormulaComposition
        );
        assert_eq!(task_257a.debug_text(), task_257a_before_c2);
        let installed_before_a = installed.debug_text();
        assert_eq!(
            installed
                .clone()
                .with_source_composite_formula(task_257a_handoff)
                .expect_err("Task 257A after C2"),
            TypedAstError::InvalidSourceCompositeFormula
        );
        assert_eq!(installed.debug_text(), installed_before_a);

        let cluster_facts = ClusterFactTable::new();
        let collection = OverloadCollectionOutput::collect(
            Vec::<OverloadSiteInput>::new(),
            Vec::<OverloadCandidateInput>::new(),
        );
        let expansion = TemplateExpansionOutput::expand(&collection);
        let viability =
            CandidateViabilityOutput::filter(&expansion, Vec::<CandidateViabilityInput>::new());
        let specificity =
            SpecificityGraphOutput::build(&viability, Vec::<SpecificityComparisonInput>::new());
        let selection = OverloadSelectionOutput::resolve(
            &specificity,
            Vec::<OverloadSiteResolutionInput>::new(),
        );
        let resolved = ResolvedTypedAst::assemble(ResolvedTypedAstInputs {
            typed_ast: &installed,
            cluster_facts: &cluster_facts,
            overload_collection: &collection,
            template_expansion: &expansion,
            viability: &viability,
            specificity: &specificity,
            overload_selection: &selection,
            expressions: Vec::new(),
            node_hints: Vec::new(),
            statement_semantics: None,
            statement_proofs: None,
        })
        .expect("Task 257C2 resolved assembly");
        assert_eq!(
            resolved.source_condition_formula_composition(),
            Some(&composition)
        );
        assert!(resolved.source_composite_formula().is_none());
        assert!(resolved.source_formula_composition().is_none());
        let debug = resolved.debug_text();
        let installed_clone = installed.clone();
        assert_eq!(
            installed_clone.source_condition_formula_composition(),
            Some(&composition)
        );
        assert_eq!(installed_clone.debug_text(), installed.debug_text());
        let resolved_clone = resolved.clone();
        assert_eq!(
            resolved_clone.source_condition_formula_composition(),
            Some(&composition)
        );
        assert_eq!(resolved_clone, resolved);
        assert_eq!(resolved_clone.debug_text(), debug);
        let atomic = debug
            .find("source-atomic-formula-debug-v1")
            .expect("atomic debug");
        let condition = debug
            .find("source-condition-formula-composition-debug-v1")
            .expect("condition/formula debug");
        assert!(atomic < condition);

        let mut orphaned = installed;
        orphaned.remove_source_condition_formula_composition_for_test();
        assert!(
            ResolvedTypedAst::assemble(ResolvedTypedAstInputs {
                typed_ast: &orphaned,
                cluster_facts: &cluster_facts,
                overload_collection: &collection,
                template_expansion: &expansion,
                viability: &viability,
                specificity: &specificity,
                overload_selection: &selection,
                expressions: Vec::new(),
                node_hints: Vec::new(),
                statement_semantics: None,
                statement_proofs: None,
            })
            .is_ok()
        );
    }

    struct Task257C3Fixture {
        lower: crate::source_atomic_formula::tests::Fixture,
        atomic: SourceAtomicFormulaHandoff,
        input: SourcePredicateChainCompositionHandoffInput,
    }

    fn task_257c3_fixture() -> Task257C3Fixture {
        let lower = crate::source_atomic_formula::tests::predicate_chain_fixture();
        let atomic = SourceAtomicFormulaProducer::build(
            lower.input.clone(),
            &lower.bindings,
            &lower.symbols,
            &lower.primary,
            None,
            None,
            None,
            &lower.arena,
        )
        .expect("Task 257C3 atomic handoff");
        let input = SourcePredicateChainCompositionHandoffInput {
            source_id: lower.source,
            module_id: lower.module.clone(),
            conjunctions: vec![SourcePredicateChainConjunctionInput {
                formula: SourceAtomicFormulaId::new(0),
                ordinal: 0,
                left_segment: SourcePredicateSegmentId::new(0),
                right_segment: SourcePredicateSegmentId::new(1),
                boundary: SourceAtomicEdgeId::new(1),
            }],
            negations: vec![SourcePredicateChainNegationInput {
                formula: SourceAtomicFormulaId::new(0),
                ordinal: 0,
                segment: SourcePredicateSegmentId::new(1),
            }],
        };
        Task257C3Fixture {
            lower,
            atomic,
            input,
        }
    }

    fn build_task_257c3(
        fixture: &Task257C3Fixture,
        input: SourcePredicateChainCompositionHandoffInput,
    ) -> Result<SourcePredicateChainCompositionHandoff, SourcePredicateChainCompositionError> {
        SourcePredicateChainCompositionProducer::build(
            input,
            &fixture.lower.primary,
            &fixture.atomic,
            &fixture.lower.arena,
        )
    }

    pub(crate) fn task_257c3_installed_typed_ast() -> TypedAst {
        let fixture = task_257c3_fixture();
        let composition =
            build_task_257c3(&fixture, fixture.input.clone()).expect("Task 257C3 composition");
        empty_typed_ast(
            fixture.lower.source,
            fixture.lower.module.clone(),
            fixture.lower.arena.clone(),
        )
        .with_source_term(fixture.lower.primary.clone())
        .expect("Task 257C3 Task 252")
        .with_source_atomic_formula(fixture.atomic.clone())
        .expect("Task 257C3 Task 256")
        .with_source_predicate_chain_composition(composition)
        .expect("Task 257C3 install")
    }

    fn assemble_empty_resolved(
        typed_ast: &TypedAst,
    ) -> Result<ResolvedTypedAst, crate::resolved_typed_ast::ResolvedTypedAstError> {
        let cluster_facts = ClusterFactTable::new();
        let collection = OverloadCollectionOutput::collect(
            Vec::<OverloadSiteInput>::new(),
            Vec::<OverloadCandidateInput>::new(),
        );
        let expansion = TemplateExpansionOutput::expand(&collection);
        let viability =
            CandidateViabilityOutput::filter(&expansion, Vec::<CandidateViabilityInput>::new());
        let specificity =
            SpecificityGraphOutput::build(&viability, Vec::<SpecificityComparisonInput>::new());
        let selection = OverloadSelectionOutput::resolve(
            &specificity,
            Vec::<OverloadSiteResolutionInput>::new(),
        );
        ResolvedTypedAst::assemble(ResolvedTypedAstInputs {
            typed_ast,
            cluster_facts: &cluster_facts,
            overload_collection: &collection,
            template_expansion: &expansion,
            viability: &viability,
            specificity: &specificity,
            overload_selection: &selection,
            expressions: Vec::new(),
            node_hints: Vec::new(),
            statement_semantics: None,
            statement_proofs: None,
        })
    }

    #[test]
    fn task_257c3_exact_rows_accessors_debug_and_row_errors_are_frozen() {
        let fixture = task_257c3_fixture();
        let handoff =
            build_task_257c3(&fixture, fixture.input.clone()).expect("Task 257C3 handoff");
        let replay = build_task_257c3(&fixture, fixture.input.clone()).expect("Task 257C3 replay");
        assert_eq!(handoff.source_id(), fixture.lower.source);
        assert_eq!(handoff.module_id(), &fixture.lower.module);
        assert_eq!(
            handoff.primary_term_fingerprint(),
            fixture.lower.primary.debug_text()
        );
        assert_eq!(
            handoff.atomic_formula_fingerprint(),
            fixture.atomic.debug_text()
        );
        assert_eq!(handoff.conjunctions().len(), 1);
        assert!(!handoff.conjunctions().is_empty());
        let conjunction = handoff
            .conjunctions()
            .get(SourcePredicateChainConjunctionId::new(0))
            .expect("Task 257C3 conjunction");
        assert_eq!(conjunction.formula(), SourceAtomicFormulaId::new(0));
        assert_eq!(conjunction.ordinal(), 0);
        assert_eq!(conjunction.left_segment(), SourcePredicateSegmentId::new(0));
        assert_eq!(
            conjunction.right_segment(),
            SourcePredicateSegmentId::new(1)
        );
        assert_eq!(conjunction.boundary(), SourceAtomicEdgeId::new(1));
        assert_eq!(
            handoff.conjunctions().iter().collect::<Vec<_>>(),
            [(SourcePredicateChainConjunctionId::new(0), conjunction)]
        );
        assert_eq!(handoff.negations().len(), 1);
        assert!(!handoff.negations().is_empty());
        let negation = handoff
            .negations()
            .get(SourcePredicateChainNegationId::new(0))
            .expect("Task 257C3 negation");
        assert_eq!(negation.formula(), SourceAtomicFormulaId::new(0));
        assert_eq!(negation.ordinal(), 0);
        assert_eq!(negation.segment(), SourcePredicateSegmentId::new(1));
        assert_eq!(
            handoff.negations().iter().collect::<Vec<_>>(),
            [(SourcePredicateChainNegationId::new(0), negation)]
        );
        assert_eq!(handoff.debug_text(), replay.debug_text());
        assert_eq!(
            handoff.debug_text(),
            format!(
                concat!(
                    "source-predicate-chain-composition-debug-v1\n",
                    "module: {}::{}\n",
                    "primary-term-fingerprint: {:?}\n",
                    "atomic-formula-fingerprint: {:?}\n",
                    "conjunctions: 1\n",
                    "  conjunction#0 formula=0 ordinal=0 left_segment=0 right_segment=1 boundary=1\n",
                    "negations: 1\n",
                    "  negation#0 formula=0 ordinal=0 segment=1\n",
                ),
                fixture.lower.module.package().as_str(),
                fixture.lower.module.path().as_str(),
                fixture.lower.primary.debug_text(),
                fixture.atomic.debug_text(),
            )
        );

        for mutate in [
            |input: &mut SourcePredicateChainCompositionHandoffInput| {
                input.conjunctions[0].formula = SourceAtomicFormulaId::new(1)
            },
            |input: &mut SourcePredicateChainCompositionHandoffInput| {
                input.conjunctions[0].ordinal = 1
            },
            |input: &mut SourcePredicateChainCompositionHandoffInput| {
                input.conjunctions[0].left_segment = SourcePredicateSegmentId::new(1)
            },
            |input: &mut SourcePredicateChainCompositionHandoffInput| {
                input.conjunctions[0].right_segment = SourcePredicateSegmentId::new(0)
            },
            |input: &mut SourcePredicateChainCompositionHandoffInput| {
                input.conjunctions[0].boundary = SourceAtomicEdgeId::new(0)
            },
        ] {
            let mut input = fixture.input.clone();
            mutate(&mut input);
            assert_eq!(
                build_task_257c3(&fixture, input),
                Err(SourcePredicateChainCompositionError::InvalidConjunction {
                    conjunction: SourcePredicateChainConjunctionId::new(0),
                })
            );
            assert!(build_task_257c3(&fixture, fixture.input.clone()).is_ok());
        }
        for mutate in [
            |input: &mut SourcePredicateChainCompositionHandoffInput| {
                input.negations[0].formula = SourceAtomicFormulaId::new(1)
            },
            |input: &mut SourcePredicateChainCompositionHandoffInput| {
                input.negations[0].ordinal = 1
            },
            |input: &mut SourcePredicateChainCompositionHandoffInput| {
                input.negations[0].segment = SourcePredicateSegmentId::new(0)
            },
        ] {
            let mut input = fixture.input.clone();
            mutate(&mut input);
            assert_eq!(
                build_task_257c3(&fixture, input),
                Err(SourcePredicateChainCompositionError::InvalidNegation {
                    negation: SourcePredicateChainNegationId::new(0),
                })
            );
            assert!(build_task_257c3(&fixture, fixture.input.clone()).is_ok());
        }
        for mutate in [
            |input: &mut SourcePredicateChainCompositionHandoffInput| input.conjunctions.clear(),
            |input: &mut SourcePredicateChainCompositionHandoffInput| {
                input.conjunctions.push(input.conjunctions[0].clone())
            },
            |input: &mut SourcePredicateChainCompositionHandoffInput| input.negations.clear(),
            |input: &mut SourcePredicateChainCompositionHandoffInput| {
                input.negations.push(input.negations[0].clone())
            },
            |input: &mut SourcePredicateChainCompositionHandoffInput| {
                input.conjunctions[0].formula = SourceAtomicFormulaId::new(1);
                input.negations.clear();
            },
            |input: &mut SourcePredicateChainCompositionHandoffInput| {
                input.conjunctions.clear();
                input.negations[0].formula = SourceAtomicFormulaId::new(1);
            },
        ] {
            let mut input = fixture.input.clone();
            mutate(&mut input);
            assert_eq!(
                build_task_257c3(&fixture, input),
                Err(SourcePredicateChainCompositionError::InvalidAggregate)
            );
            assert!(build_task_257c3(&fixture, fixture.input.clone()).is_ok());
        }

        let mut invalid_dependency_and_rows = fixture.input.clone();
        invalid_dependency_and_rows.module_id =
            ModuleId::new(PackageId::new("other"), ModulePath::new("other.module"));
        invalid_dependency_and_rows.conjunctions.clear();
        invalid_dependency_and_rows.negations[0].formula = SourceAtomicFormulaId::new(1);
        assert_eq!(
            build_task_257c3(&fixture, invalid_dependency_and_rows),
            Err(SourcePredicateChainCompositionError::DependencyMismatch)
        );

        let mut invalid_conjunction_and_negation = fixture.input.clone();
        invalid_conjunction_and_negation.conjunctions[0].formula = SourceAtomicFormulaId::new(1);
        invalid_conjunction_and_negation.negations[0].formula = SourceAtomicFormulaId::new(1);
        assert_eq!(
            build_task_257c3(&fixture, invalid_conjunction_and_negation),
            Err(SourcePredicateChainCompositionError::InvalidConjunction {
                conjunction: SourcePredicateChainConjunctionId::new(0),
            })
        );
    }

    #[test]
    fn task_257c3_dependency_profiles_and_stale_arena_fail_closed() {
        let fixture = task_257c3_fixture();
        let mut mismatched = fixture.input.clone();
        mismatched.module_id =
            ModuleId::new(PackageId::new("other"), ModulePath::new("other.module"));
        assert_eq!(
            build_task_257c3(&fixture, mismatched),
            Err(SourcePredicateChainCompositionError::DependencyMismatch)
        );

        let wrong =
            crate::source_atomic_formula::tests::single_predicate_on_predicate_chain_arena_fixture(
            );
        let wrong_atomic = SourceAtomicFormulaProducer::build(
            wrong.input.clone(),
            &wrong.bindings,
            &wrong.symbols,
            &wrong.primary,
            None,
            None,
            None,
            &wrong.arena,
        )
        .expect("independently valid wrong-profile Task 256");
        assert!(
            wrong
                .primary
                .validate_installation(wrong.source, &wrong.module, &wrong.arena)
                .is_ok()
        );
        assert!(
            wrong_atomic
                .validate_installation(
                    wrong.source,
                    &wrong.module,
                    &wrong.primary,
                    None,
                    None,
                    None,
                    &wrong.arena,
                )
                .is_ok()
        );
        assert_eq!(
            SourcePredicateChainCompositionProducer::build(
                fixture.input.clone(),
                &wrong.primary,
                &wrong_atomic,
                &wrong.arena,
            ),
            Err(SourcePredicateChainCompositionError::DependencyMismatch)
        );

        let mut nodes = fixture
            .lower
            .arena
            .iter()
            .map(|(_, node)| node.clone())
            .collect::<Vec<_>>();
        nodes[0].anchor = SourceAnchor::Range(range(fixture.lower.source, 74, 75));
        let stale_arena =
            TypedArena::try_new(fixture.lower.arena.root(), nodes).expect("stale Task 257C3 arena");
        assert_eq!(
            SourcePredicateChainCompositionProducer::build(
                fixture.input.clone(),
                &fixture.lower.primary,
                &fixture.atomic,
                &stale_arena,
            ),
            Err(SourcePredicateChainCompositionError::DependencyMismatch)
        );

        let base = empty_typed_ast(
            fixture.lower.source,
            fixture.lower.module.clone(),
            fixture.lower.arena.clone(),
        )
        .with_source_term(fixture.lower.primary.clone())
        .expect("Task 252 install")
        .with_source_atomic_formula(fixture.atomic.clone())
        .expect("Task 256 install");
        for mutate in [
            |candidate: &mut SourcePredicateChainCompositionHandoff| {
                candidate.primary_term_fingerprint.push_str("stale")
            },
            |candidate: &mut SourcePredicateChainCompositionHandoff| {
                candidate.atomic_formula_fingerprint.push_str("stale")
            },
        ] {
            let mut stale =
                build_task_257c3(&fixture, fixture.input.clone()).expect("stale C3 candidate");
            mutate(&mut stale);
            let before = base.debug_text();
            assert_eq!(
                base.clone()
                    .with_source_predicate_chain_composition(stale)
                    .expect_err("stale C3 handoff"),
                TypedAstError::InvalidSourcePredicateChainComposition
            );
            assert_eq!(base.debug_text(), before);
        }
        assert!(
            base.with_source_predicate_chain_composition(
                build_task_257c3(&fixture, fixture.input.clone()).expect("valid C3 replay")
            )
            .is_ok()
        );
    }

    #[test]
    fn task_257c3_typed_resolved_ownership_is_atomic_and_clone_preserved() {
        let fixture = task_257c3_fixture();
        let composition =
            build_task_257c3(&fixture, fixture.input.clone()).expect("Task 257C3 composition");
        let empty = empty_typed_ast(
            fixture.lower.source,
            fixture.lower.module.clone(),
            fixture.lower.arena.clone(),
        );
        assert_eq!(
            empty
                .clone()
                .with_source_predicate_chain_composition(composition.clone())
                .expect_err("missing Task 252"),
            TypedAstError::InvalidSourcePredicateChainComposition
        );
        let with_term = empty
            .with_source_term(fixture.lower.primary.clone())
            .expect("Task 252 install");
        assert_eq!(
            with_term
                .clone()
                .with_source_predicate_chain_composition(composition.clone())
                .expect_err("missing Task 256"),
            TypedAstError::InvalidSourcePredicateChainComposition
        );
        let base = with_term
            .with_source_atomic_formula(fixture.atomic.clone())
            .expect("Task 256 install");
        let installed = base
            .clone()
            .with_source_predicate_chain_composition(composition.clone())
            .expect("Task 257C3 install");
        assert_eq!(
            installed.source_predicate_chain_composition(),
            Some(&composition)
        );
        assert!(installed.source_composite_formula().is_none());
        assert!(installed.source_formula_composition().is_none());
        assert!(installed.source_condition_formula_composition().is_none());
        let before_duplicate = installed.debug_text();
        let typed_term = before_duplicate
            .find("source-primary-term-debug-v1")
            .expect("typed term debug");
        let typed_atomic = before_duplicate
            .find("source-atomic-formula-debug-v1")
            .expect("typed atomic debug");
        let typed_composition = before_duplicate
            .find("source-predicate-chain-composition-debug-v1")
            .expect("typed C3 debug");
        let typed_nodes = before_duplicate.find("nodes:").expect("typed nodes");
        assert!(
            typed_term < typed_atomic
                && typed_atomic < typed_composition
                && typed_composition < typed_nodes
        );
        assert_eq!(
            installed
                .clone()
                .with_source_predicate_chain_composition(composition.clone())
                .expect_err("duplicate Task 257C3"),
            TypedAstError::InvalidSourcePredicateChainComposition
        );
        assert_eq!(installed.debug_text(), before_duplicate);

        let task_257a = crate::source_composite_formula::tests::task_257a_installed_typed_ast();
        let task_257a_handoff = task_257a
            .source_composite_formula()
            .expect("Task 257A handoff")
            .clone();
        let before_a = task_257a.debug_text();
        assert_eq!(
            task_257a
                .clone()
                .with_source_predicate_chain_composition(composition.clone())
                .expect_err("C3 after Task 257A"),
            TypedAstError::InvalidSourcePredicateChainComposition
        );
        assert_eq!(task_257a.debug_text(), before_a);

        let b1 = self::fixture();
        let b1_composition = build(&b1, b1.input.clone()).expect("Task 257B1 composition");
        let b1_installed = empty_typed_ast(b1.source, b1.module.clone(), b1.arena.clone())
            .with_source_term(b1.primary.clone())
            .expect("Task 257B1 term")
            .with_source_atomic_formula(b1.atomic.clone())
            .expect("Task 257B1 atomic")
            .with_source_formula_composition(b1.composite.clone(), b1_composition.clone())
            .expect("Task 257B1 install");
        let before_b = b1_installed.debug_text();
        assert_eq!(
            b1_installed
                .clone()
                .with_source_predicate_chain_composition(composition.clone())
                .expect_err("C3 after Task 257B"),
            TypedAstError::InvalidSourcePredicateChainComposition
        );
        assert_eq!(b1_installed.debug_text(), before_b);

        let c2 = task_257c2_fixture();
        let c2_composition =
            build_task_257c2(&c2, c2.input.clone()).expect("Task 257C2 composition");
        let c2_installed = c2
            .lower
            .typed_ast()
            .with_source_set_term(c2.set.clone())
            .expect("Task 255 install")
            .with_source_atomic_formula(c2.atomic.clone())
            .expect("Task 256 equality install")
            .with_source_condition_formula_composition(c2_composition.clone())
            .expect("Task 257C2 install");
        let before_c2 = c2_installed.debug_text();
        assert_eq!(
            c2_installed
                .clone()
                .with_source_predicate_chain_composition(composition.clone())
                .expect_err("C3 after Task 257C2"),
            TypedAstError::InvalidSourcePredicateChainComposition
        );
        assert_eq!(c2_installed.debug_text(), before_c2);

        let mut c3_base_with_a = base.clone();
        c3_base_with_a.inject_source_composite_formula_for_test(task_257a_handoff.clone());
        assert_eq!(
            c3_base_with_a
                .with_source_predicate_chain_composition(composition.clone())
                .expect_err("Task 257A field excludes C3"),
            TypedAstError::InvalidSourcePredicateChainComposition
        );
        let mut c3_base_with_b = base.clone();
        c3_base_with_b.inject_source_formula_composition_for_test(b1_composition.clone());
        assert_eq!(
            c3_base_with_b
                .with_source_predicate_chain_composition(composition.clone())
                .expect_err("Task 257B field excludes C3"),
            TypedAstError::InvalidSourcePredicateChainComposition
        );
        let mut c3_base_with_c2 = base.clone();
        c3_base_with_c2
            .inject_source_condition_formula_composition_for_test(c2_composition.clone());
        assert_eq!(
            c3_base_with_c2
                .with_source_predicate_chain_composition(composition.clone())
                .expect_err("Task 257C2 field excludes C3"),
            TypedAstError::InvalidSourcePredicateChainComposition
        );

        let mut a_base_with_c3 = task_257a;
        a_base_with_c3.remove_source_composite_formula_for_test();
        a_base_with_c3.inject_source_predicate_chain_composition_for_test(composition.clone());
        assert_eq!(
            a_base_with_c3
                .with_source_composite_formula(task_257a_handoff)
                .expect_err("C3 field excludes Task 257A"),
            TypedAstError::InvalidSourceCompositeFormula
        );
        let mut b_base_with_c3 = b1_installed;
        b_base_with_c3.remove_source_composite_formula_for_test();
        b_base_with_c3.remove_source_formula_composition_for_test();
        b_base_with_c3.inject_source_predicate_chain_composition_for_test(composition.clone());
        assert_eq!(
            b_base_with_c3
                .with_source_formula_composition(b1.composite.clone(), b1_composition)
                .expect_err("C3 field excludes Task 257B"),
            TypedAstError::InvalidSourceFormulaComposition
        );
        let mut c2_base_with_c3 = c2_installed;
        c2_base_with_c3.remove_source_condition_formula_composition_for_test();
        c2_base_with_c3.inject_source_predicate_chain_composition_for_test(composition.clone());
        assert_eq!(
            c2_base_with_c3
                .with_source_condition_formula_composition(c2_composition)
                .expect_err("C3 field excludes Task 257C2"),
            TypedAstError::InvalidSourceConditionFormulaComposition
        );
        assert!(
            base.clone()
                .with_source_predicate_chain_composition(composition.clone())
                .is_ok()
        );

        let resolved = assemble_empty_resolved(&installed).expect("Task 257C3 resolved assembly");
        assert_eq!(
            resolved.source_predicate_chain_composition(),
            Some(&composition)
        );
        let typed_clone = installed.clone();
        assert_eq!(
            typed_clone.source_predicate_chain_composition(),
            Some(&composition)
        );
        assert_eq!(typed_clone.debug_text(), installed.debug_text());
        let resolved_clone = resolved.clone();
        assert_eq!(resolved_clone, resolved);
        assert_eq!(
            resolved_clone.source_predicate_chain_composition(),
            Some(&composition)
        );
        assert_eq!(resolved_clone.debug_text(), resolved.debug_text());
        let debug = resolved.debug_text();
        let term = debug
            .find("source-primary-term-debug-v1")
            .expect("term debug");
        let atomic = debug
            .find("source-atomic-formula-debug-v1")
            .expect("atomic debug");
        let composition_debug = debug
            .find("source-predicate-chain-composition-debug-v1")
            .expect("C3 debug");
        let nodes = debug.find("nodes:").expect("resolved nodes");
        assert!(term < atomic && atomic < composition_debug && composition_debug < nodes);

        let mut orphaned = installed;
        orphaned.remove_source_atomic_formula_for_test();
        assert_eq!(
            assemble_empty_resolved(&orphaned).expect_err("resolved C3 revalidation"),
            crate::resolved_typed_ast::ResolvedTypedAstError::InvalidSourcePredicateChainComposition
        );
    }

    fn empty_typed_ast(source: SourceId, module: ModuleId, nodes: TypedArena) -> TypedAst {
        TypedAst::try_new(TypedAstParts {
            source_id: source,
            module_id: module,
            resolved_root: None,
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes,
            contexts: LocalTypeContextTable::new(),
            types: TypeTable::new(),
            facts: TypeFactTable::new(),
            coercions: CoercionTable::new(),
            initial_obligations: InitialObligationTable::new(),
            diagnostics: TypeDiagnosticTable::new(),
        })
        .expect("empty typed AST")
    }

    #[track_caller]
    fn assert_combined_install_rejected_without_publication(
        typed: &TypedAst,
        composite: SourceCompositeFormulaHandoff,
        composition: SourceFormulaCompositionHandoff,
    ) {
        assert_combined_install_rejected_preserving_state(typed, composite, composition);
        assert!(typed.source_composite_formula().is_none());
        assert!(typed.source_formula_composition().is_none());
    }

    #[track_caller]
    fn assert_combined_install_rejected_preserving_state(
        typed: &TypedAst,
        composite: SourceCompositeFormulaHandoff,
        composition: SourceFormulaCompositionHandoff,
    ) {
        let before = typed.debug_text();
        assert_eq!(
            typed
                .clone()
                .with_source_formula_composition(composite, composition)
                .expect_err("combined installation must fail closed"),
            TypedAstError::InvalidSourceFormulaComposition
        );
        assert_eq!(typed.debug_text(), before);
    }

    const EXPECTED_DEBUG: &str = r###"source-formula-composition-debug-v1
module: pkg::composition.fixture
primary-term-fingerprint: "source-primary-term-debug-v1\nmodule: composition.fixture\nterm#0 ordinal=0 kind=variable-reference role=value range=72..73 site=5 context=1 recovery=normal spelling=\"x\" parent=-\nterm#1 ordinal=1 kind=variable-reference role=value range=76..77 site=6 context=1 recovery=normal spelling=\"x\" parent=-\nreference#0 term=0 binding=0 role=variable use_ordinal=1 scope=[0]\nreference#1 term=1 binding=0 role=variable use_ordinal=1 scope=[0]\n"
atomic-formula-fingerprint: "source-atomic-formula-debug-v1\nmodule: composition.fixture\nprimary-term-fingerprint: \"source-primary-term-debug-v1\\nmodule: composition.fixture\\nterm#0 ordinal=0 kind=variable-reference role=value range=72..73 site=5 context=1 recovery=normal spelling=\\\"x\\\" parent=-\\nterm#1 ordinal=1 kind=variable-reference role=value range=76..77 site=6 context=1 recovery=normal spelling=\\\"x\\\" parent=-\\nreference#0 term=0 binding=0 role=variable use_ordinal=1 scope=[0]\\nreference#1 term=1 binding=0 role=variable use_ordinal=1 scope=[0]\\n\"\napplication-fingerprint: None\nstructure-fingerprint: None\nset-term-fingerprint: None\nformula#0 ordinal=0 kind=equality range=72..77 site=7 context=1 recovery=normal spelling=\"x = x\"\nedge#0 formula=0 ordinal=0 role=builtin-left target=primary:0\nedge#1 formula=0 ordinal=1 role=builtin-right target=primary:1\nrequest#0 formula=0 ordinal=0 kind=operand-expected-type edge=0 candidate=- type_site=- attribute=-\nrequest#1 formula=0 ordinal=1 kind=operand-expected-type edge=1 candidate=- type_site=- attribute=-\n"
composite-formula-fingerprint: "source-composite-formula-debug-v1\nmodule: pkg::composition.fixture\nbinding-env-debug-v1\nmodule: pkg::composition.fixture\ncontexts:\n  context#0 owner=module parent=none layer=module scope=none bindings=[] visible=[] recovery=normal\n  context#1 owner=source-formula(50..77) parent=context#0 layer=expression scope=[0] bindings=[binding#0] visible=[binding#0] recovery=normal\nbindings:\n  binding#0 spelling=\"x\" kind=quantifier_binder owner=context#1 identity=resolver_local(scope=[0], ordinal=0, range=54..55) range=54..55 visible_after=0 type=source(62..65) status=active captured=[] diagnostics=[] recovery=normal\ndiagnostics:\n  diagnostic#3 range=none class=external_dependency_gap severity=note key=\"checker.binding.external.closure_payload\" recovery=degraded\n  diagnostic#0 range=none class=external_dependency_gap severity=note key=\"checker.binding.external.local_bindings\" recovery=degraded\n  diagnostic#2 range=none class=external_dependency_gap severity=note key=\"checker.binding.external.reserve_payload\" recovery=degraded\n  diagnostic#1 range=none class=external_dependency_gap severity=note key=\"checker.binding.external.use_site_scope\" recovery=degraded\nformulas: 1\n  formula#0 site=0 range=50..77 ordinal=0 context=0 recovery=normal spelling=\"for holds\" kind=universal\nwrappers: 0\nroots: 1\n  root#0 formula=0 ordinal=0 ownership=unassigned-statement\nbinders: 1\n  binder#0 formula=0 ordinal=0 segment-site=1 segment-range=54..65 segment-spelling=\"x being\" identifier-site=2 identifier-range=54..55 identifier-spelling=\"x\" local-scope=[0] local-ordinal=0 binding=0 body-context=1 type-site=0 recovery=normal\ntype-sites: 1\n  type-site#0 binder=0 site=3 range=62..65 spelling=\"set\" head-site=4 head-range=62..65 head-spelling=\"set\" context=0 recovery=normal head=builtin-set\nedges: 0\nrequests: 2\n  request#0 formula=0 ordinal=0 kind=quantifier-semantics binder=- type-site=-\n  request#1 formula=0 ordinal=1 kind=binder-type binder=0 type-site=0\n"
atomic-edges: 1
  atomic-edge#0 formula=0 ordinal=0 role=universal-body child=0
bound-uses: 2
  bound-use#0 binder=0 ordinal=0 body-edge=0 term=0 reference=0
  bound-use#1 binder=0 ordinal=1 body-edge=0 term=1 reference=1
"###;
}
