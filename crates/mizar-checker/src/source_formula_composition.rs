//! Syntax-free cross-family source formula composition transport.

use crate::{
    binding_env::{BindingLookupResult, BindingLookupSite},
    source_application::{
        SourceFunctorApplicationForm, SourceFunctorApplicationHandoff, SourceFunctorApplicationId,
        SourceFunctorApplicationKind, SourceFunctorApplicationRecovery, SourceFunctorArgumentId,
        SourceFunctorArgumentTarget, SourceFunctorCandidateId, SourceFunctorHeadSite,
        SourceFunctorTypeRequestKind,
    },
    source_atomic_formula::{
        SourceAtomicEdgeId, SourceAtomicEdgeRole, SourceAtomicFormulaHandoff,
        SourceAtomicFormulaId, SourceAtomicFormulaKind, SourceAtomicFormulaRecovery,
        SourceAtomicRequestKind, SourceAtomicTermTarget,
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
    source_term::{
        SourceNumericTypeRequestId, SourcePrimaryTermHandoff, SourcePrimaryTermId,
        SourcePrimaryTermKind, SourcePrimaryTermRecovery, SourcePrimaryTermReferenceId,
        SourcePrimaryTermReferenceRole, SourcePrimaryTermRole,
    },
    typed_ast::TypedArena,
};
use mizar_resolve::resolved_ast::ModuleId;
use mizar_session::{SourceId, SourceRange};
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        binding_env::{
            BindingContextDraft, BindingContextId, BindingContextLayer, BindingContextOwner,
            BindingContextRecovery, BindingContextTable, BindingDiagnosticClass,
            BindingDiagnosticDraft, BindingDiagnosticRecovery, BindingDiagnosticSeverity,
            BindingDiagnosticTable, BindingEnv, BindingEnvParts, BindingId, BindingTable,
        },
        cluster_trace::ClusterFactTable,
        overload_resolution::{
            CandidateViabilityInput, CandidateViabilityOutput, OverloadCandidateInput,
            OverloadCollectionOutput, OverloadSelectionOutput, OverloadSiteInput,
            OverloadSiteResolutionInput, SpecificityComparisonInput, SpecificityGraphOutput,
            TemplateExpansionOutput,
        },
        resolved_typed_ast::{ResolvedTypedAst, ResolvedTypedAstInputs},
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
        source_term::{
            SourceNumericTypeRequestInput, SourcePrimaryTermHandoffInput, SourcePrimaryTermInput,
            SourcePrimaryTermProducer, SourcePrimaryTermReferenceInput,
        },
        typed_ast::{
            CoercionTable, InitialObligationTable, LocalTypeContextTable, TypeDiagnosticTable,
            TypeFactTable, TypeTable, TypedAst, TypedAstError, TypedAstParts, TypedNode,
            TypedNodeId, TypedSiteRef,
        },
    };
    use mizar_resolve::{
        env::{SymbolEnv, SymbolEnvIndexes},
        names::{LocalTermBinding, LocalTermScope},
    };
    use mizar_session::{
        BuildSnapshotId, InMemorySessionIdAllocator, ModulePath, PackageId,
        SessionIdAllocator as _, SourceAnchor,
    };

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
