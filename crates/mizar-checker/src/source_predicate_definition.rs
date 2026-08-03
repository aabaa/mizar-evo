//! Syntax-free predicate-definition intake for checker phase 6.

use crate::{
    binding_env::{
        BindingContextId, BindingId, BindingKind, BindingRecoveryState, BindingStatus,
        BindingTypeSite,
    },
    source_atomic_formula::{
        SourceAtomicEdgeId, SourceAtomicEdgeRole, SourceAtomicFormulaHandoff,
        SourceAtomicFormulaId, SourceAtomicFormulaKind, SourceAtomicFormulaRecovery,
        SourceAtomicRequestKind, SourceAtomicTermTarget,
    },
    source_context::{
        SourceBindingContextHandoff, SourceBindingSiteRole, SourceItemRecovery, SourceItemRole,
        SourceItemVisibility,
    },
    source_term::{
        SourcePrimaryTermHandoff, SourcePrimaryTermId, SourcePrimaryTermKind,
        SourcePrimaryTermRecovery, SourcePrimaryTermReferenceRole, SourcePrimaryTermRole,
    },
    source_type::{
        SourceTypeApplicationForm, SourceTypeApplicationHandoff, SourceTypeApplicationId,
        SourceTypeExpressionId, SourceTypeHead,
    },
    typed_ast::{
        InitialObligationDraft, InitialObligationGoal, InitialObligationId, InitialObligationKind,
        InitialObligationProvenance, InitialObligationStatus, InitialObligationTable,
        LocalTypeContextId, NodeRecoveryState, TypedArena, TypedNodeId, TypedSiteRef,
    },
};
use mizar_resolve::{
    env::{
        ContributionKind, DefinitionId, DefinitionKind, ExportStatus, SourceContributionId,
        SymbolEnv, SymbolKind, Visibility,
    },
    resolved_ast::{ModuleId, SemanticOrigin, SymbolId},
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

dense_id!(SourcePredicateDefinitionId);
dense_id!(SourcePredicateParameterId);
dense_id!(SourcePredicateGuardId);
dense_id!(SourcePredicatePropertyId);
dense_id!(SourcePredicateCorrectnessId);

/// Complete syntax-free input for one predicate-definition transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePredicateDefinitionHandoffInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub definitions: Vec<SourcePredicateDefinitionInput>,
    pub parameters: Vec<SourcePredicateParameterInput>,
    pub guards: Vec<SourcePredicateGuardInput>,
    pub properties: Vec<SourcePredicatePropertyInput>,
    pub correctness: Vec<SourcePredicateCorrectnessInput>,
}

/// One source predicate definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePredicateDefinitionInput {
    pub symbol: SymbolId,
    pub definition: DefinitionId,
    pub contribution: SourceContributionId,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub source_ordinal: usize,
    pub context: BindingContextId,
    pub recovery: SourcePredicateDefinitionRecovery,
    pub spelling: String,
    pub definiens: SourceAtomicFormulaId,
}

/// One source-written predicate parameter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePredicateParameterInput {
    pub owner: SourcePredicateDefinitionId,
    pub ordinal: usize,
    pub binding: BindingId,
    pub written_type: SourceTypeApplicationId,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub declaration_range: SourceRange,
    pub context: BindingContextId,
    pub recovery: SourcePredicateDefinitionRecovery,
    pub spelling: String,
}

/// One definition-local guard.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePredicateGuardInput {
    pub owner: SourcePredicateDefinitionId,
    pub ordinal: usize,
    pub formula: SourceAtomicFormulaId,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub context: BindingContextId,
    pub recovery: SourcePredicateDefinitionRecovery,
    pub spelling: String,
}

/// One source predicate-property clause.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePredicatePropertyInput {
    pub owner: SourcePredicateDefinitionId,
    pub ordinal: usize,
    pub kind: SourcePredicatePropertyKind,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub justification: SourceAnchor,
    pub recovery: SourcePredicateDefinitionRecovery,
    pub spelling: String,
}

/// One property-to-correctness association.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePredicateCorrectnessInput {
    pub owner: SourcePredicateDefinitionId,
    pub property: SourcePredicatePropertyId,
    pub ordinal: usize,
    pub source_anchor: SourceAnchor,
}

/// Predicate-property kinds admitted at this boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourcePredicatePropertyKind {
    Symmetry,
}

/// Recovery state retained at the predicate-definition boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourcePredicateDefinitionRecovery {
    Normal,
    Degraded,
}

/// One immutable validated predicate definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePredicateDefinition {
    id: SourcePredicateDefinitionId,
    symbol: SymbolId,
    definition: DefinitionId,
    contribution: SourceContributionId,
    site: TypedSiteRef,
    source_range: SourceRange,
    source_ordinal: usize,
    context: BindingContextId,
    recovery: SourcePredicateDefinitionRecovery,
    spelling: String,
    definiens: SourceAtomicFormulaId,
    origin: SemanticOrigin,
}

impl SourcePredicateDefinition {
    pub const fn id(&self) -> SourcePredicateDefinitionId {
        self.id
    }

    pub const fn symbol(&self) -> &SymbolId {
        &self.symbol
    }

    pub const fn definition(&self) -> DefinitionId {
        self.definition
    }

    pub const fn contribution(&self) -> SourceContributionId {
        self.contribution
    }

    pub const fn site(&self) -> &TypedSiteRef {
        &self.site
    }

    pub const fn source_range(&self) -> SourceRange {
        self.source_range
    }

    pub const fn source_ordinal(&self) -> usize {
        self.source_ordinal
    }

    pub const fn context(&self) -> BindingContextId {
        self.context
    }

    pub const fn recovery(&self) -> SourcePredicateDefinitionRecovery {
        self.recovery
    }

    pub fn spelling(&self) -> &str {
        &self.spelling
    }

    pub const fn definiens(&self) -> SourceAtomicFormulaId {
        self.definiens
    }

    pub const fn origin(&self) -> &SemanticOrigin {
        &self.origin
    }
}

/// One immutable validated predicate parameter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePredicateParameter {
    id: SourcePredicateParameterId,
    owner: SourcePredicateDefinitionId,
    ordinal: usize,
    binding: BindingId,
    written_type: SourceTypeApplicationId,
    site: TypedSiteRef,
    source_range: SourceRange,
    declaration_range: SourceRange,
    context: BindingContextId,
    recovery: SourcePredicateDefinitionRecovery,
    spelling: String,
}

impl SourcePredicateParameter {
    pub const fn id(&self) -> SourcePredicateParameterId {
        self.id
    }

    pub const fn owner(&self) -> SourcePredicateDefinitionId {
        self.owner
    }

    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }

    pub const fn binding(&self) -> BindingId {
        self.binding
    }

    pub const fn written_type(&self) -> SourceTypeApplicationId {
        self.written_type
    }

    pub const fn site(&self) -> &TypedSiteRef {
        &self.site
    }

    pub const fn source_range(&self) -> SourceRange {
        self.source_range
    }

    pub const fn declaration_range(&self) -> SourceRange {
        self.declaration_range
    }

    pub const fn context(&self) -> BindingContextId {
        self.context
    }

    pub const fn recovery(&self) -> SourcePredicateDefinitionRecovery {
        self.recovery
    }

    pub fn spelling(&self) -> &str {
        &self.spelling
    }
}

/// One immutable validated definition-local guard.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePredicateGuard {
    id: SourcePredicateGuardId,
    owner: SourcePredicateDefinitionId,
    ordinal: usize,
    formula: SourceAtomicFormulaId,
    site: TypedSiteRef,
    source_range: SourceRange,
    context: BindingContextId,
    recovery: SourcePredicateDefinitionRecovery,
    spelling: String,
}

impl SourcePredicateGuard {
    pub const fn id(&self) -> SourcePredicateGuardId {
        self.id
    }

    pub const fn owner(&self) -> SourcePredicateDefinitionId {
        self.owner
    }

    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }

    pub const fn formula(&self) -> SourceAtomicFormulaId {
        self.formula
    }

    pub const fn site(&self) -> &TypedSiteRef {
        &self.site
    }

    pub const fn source_range(&self) -> SourceRange {
        self.source_range
    }

    pub const fn context(&self) -> BindingContextId {
        self.context
    }

    pub const fn recovery(&self) -> SourcePredicateDefinitionRecovery {
        self.recovery
    }

    pub fn spelling(&self) -> &str {
        &self.spelling
    }
}

/// One immutable validated predicate property.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePredicateProperty {
    id: SourcePredicatePropertyId,
    owner: SourcePredicateDefinitionId,
    ordinal: usize,
    kind: SourcePredicatePropertyKind,
    site: TypedSiteRef,
    source_range: SourceRange,
    justification: SourceAnchor,
    recovery: SourcePredicateDefinitionRecovery,
    spelling: String,
}

impl SourcePredicateProperty {
    pub const fn id(&self) -> SourcePredicatePropertyId {
        self.id
    }

    pub const fn owner(&self) -> SourcePredicateDefinitionId {
        self.owner
    }

    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }

    pub const fn kind(&self) -> SourcePredicatePropertyKind {
        self.kind
    }

    pub const fn site(&self) -> &TypedSiteRef {
        &self.site
    }

    pub const fn source_range(&self) -> SourceRange {
        self.source_range
    }

    pub const fn justification(&self) -> &SourceAnchor {
        &self.justification
    }

    pub const fn recovery(&self) -> SourcePredicateDefinitionRecovery {
        self.recovery
    }

    pub fn spelling(&self) -> &str {
        &self.spelling
    }
}

/// One immutable validated predicate-property correctness link.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePredicateCorrectness {
    id: SourcePredicateCorrectnessId,
    owner: SourcePredicateDefinitionId,
    property: SourcePredicatePropertyId,
    ordinal: usize,
    source_anchor: SourceAnchor,
    obligation: InitialObligationId,
}

impl SourcePredicateCorrectness {
    pub const fn id(&self) -> SourcePredicateCorrectnessId {
        self.id
    }

    pub const fn owner(&self) -> SourcePredicateDefinitionId {
        self.owner
    }

    pub const fn property(&self) -> SourcePredicatePropertyId {
        self.property
    }

    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }

    pub const fn source_anchor(&self) -> &SourceAnchor {
        &self.source_anchor
    }

    pub const fn obligation(&self) -> InitialObligationId {
        self.obligation
    }
}

macro_rules! table {
    ($table:ident, $row:ident, $id:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $table {
            rows: Vec<$row>,
        }

        impl $table {
            pub fn get(&self, id: $id) -> Option<&$row> {
                self.rows.get(id.index())
            }

            pub fn iter(&self) -> impl Iterator<Item = ($id, &$row)> {
                self.rows.iter().enumerate().map(|(index, row)| {
                    debug_assert_eq!(row.id(), $id::new(index));
                    ($id::new(index), row)
                })
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
    SourcePredicateDefinitionTable,
    SourcePredicateDefinition,
    SourcePredicateDefinitionId
);
table!(
    SourcePredicateParameterTable,
    SourcePredicateParameter,
    SourcePredicateParameterId
);
table!(
    SourcePredicateGuardTable,
    SourcePredicateGuard,
    SourcePredicateGuardId
);
table!(
    SourcePredicatePropertyTable,
    SourcePredicateProperty,
    SourcePredicatePropertyId
);
table!(
    SourcePredicateCorrectnessTable,
    SourcePredicateCorrectness,
    SourcePredicateCorrectnessId
);

/// Immutable validated predicate-definition handoff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePredicateDefinitionHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    resolver_identity: SourcePredicateResolverIdentity,
    source_context_fingerprint: String,
    source_type_fingerprint: String,
    source_term_fingerprint: String,
    source_atomic_formula_fingerprint: String,
    definitions: SourcePredicateDefinitionTable,
    parameters: SourcePredicateParameterTable,
    guards: SourcePredicateGuardTable,
    properties: SourcePredicatePropertyTable,
    correctness: SourcePredicateCorrectnessTable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SourcePredicateResolverIdentity {
    symbol: SymbolId,
    definition: DefinitionId,
    contribution: SourceContributionId,
    origin: SemanticOrigin,
}

impl SourcePredicateDefinitionHandoff {
    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    pub fn source_context_fingerprint(&self) -> &str {
        &self.source_context_fingerprint
    }

    pub fn source_type_fingerprint(&self) -> &str {
        &self.source_type_fingerprint
    }

    pub fn source_term_fingerprint(&self) -> &str {
        &self.source_term_fingerprint
    }

    pub fn source_atomic_formula_fingerprint(&self) -> &str {
        &self.source_atomic_formula_fingerprint
    }

    pub const fn definitions(&self) -> &SourcePredicateDefinitionTable {
        &self.definitions
    }

    pub const fn parameters(&self) -> &SourcePredicateParameterTable {
        &self.parameters
    }

    pub const fn guards(&self) -> &SourcePredicateGuardTable {
        &self.guards
    }

    pub const fn properties(&self) -> &SourcePredicatePropertyTable {
        &self.properties
    }

    pub const fn correctness(&self) -> &SourcePredicateCorrectnessTable {
        &self.correctness
    }

    /// Stable, source-ordered representation used as a dependency fingerprint.
    pub fn debug_text(&self) -> String {
        let mut output = String::from("source-predicate-definition-debug-v1\n");
        let _ = writeln!(output, "module: {}", self.module_id.path().as_str());
        let _ = writeln!(
            output,
            "source-context-fingerprint: {:?}",
            self.source_context_fingerprint
        );
        let _ = writeln!(
            output,
            "source-type-fingerprint: {:?}",
            self.source_type_fingerprint
        );
        let _ = writeln!(
            output,
            "source-term-fingerprint: {:?}",
            self.source_term_fingerprint
        );
        let _ = writeln!(
            output,
            "source-atomic-formula-fingerprint: {:?}",
            self.source_atomic_formula_fingerprint
        );
        for (id, row) in self.definitions.iter() {
            let _ = write!(
                output,
                "definition#{} symbol={:?} definition={} contribution={} ordinal={} range={}..{} site=",
                id.index(),
                row.symbol.fqn().as_str(),
                row.definition.index(),
                row.contribution.index(),
                row.source_ordinal,
                row.source_range.start,
                row.source_range.end,
            );
            write_site(&mut output, &row.site);
            let _ = write!(
                output,
                " context={} recovery={} origin_range=",
                row.context.index(),
                recovery_key(row.recovery),
            );
            write_anchor_range(&mut output, row.origin.anchor());
            let _ = writeln!(
                output,
                " origin_path={:?} spelling={:?} definiens={}",
                row.origin.structural_path(),
                row.spelling,
                row.definiens.index(),
            );
        }
        for (id, row) in self.parameters.iter() {
            let _ = write!(
                output,
                "parameter#{} owner={} ordinal={} binding={} written_type={} range={}..{} declaration_range={}..{} site=",
                id.index(),
                row.owner.index(),
                row.ordinal,
                row.binding.index(),
                row.written_type.index(),
                row.source_range.start,
                row.source_range.end,
                row.declaration_range.start,
                row.declaration_range.end,
            );
            write_site(&mut output, &row.site);
            let _ = writeln!(
                output,
                " context={} recovery={} spelling={:?}",
                row.context.index(),
                recovery_key(row.recovery),
                row.spelling,
            );
        }
        for (id, row) in self.guards.iter() {
            let _ = write!(
                output,
                "guard#{} owner={} ordinal={} formula={} range={}..{} site=",
                id.index(),
                row.owner.index(),
                row.ordinal,
                row.formula.index(),
                row.source_range.start,
                row.source_range.end,
            );
            write_site(&mut output, &row.site);
            let _ = writeln!(
                output,
                " context={} recovery={} spelling={:?}",
                row.context.index(),
                recovery_key(row.recovery),
                row.spelling,
            );
        }
        for (id, row) in self.properties.iter() {
            let _ = write!(
                output,
                "property#{} owner={} ordinal={} kind={} range={}..{} site=",
                id.index(),
                row.owner.index(),
                row.ordinal,
                property_kind_key(row.kind),
                row.source_range.start,
                row.source_range.end,
            );
            write_site(&mut output, &row.site);
            output.push_str(" justification=");
            write_anchor(&mut output, &row.justification);
            let _ = writeln!(
                output,
                " recovery={} spelling={:?}",
                recovery_key(row.recovery),
                row.spelling,
            );
        }
        for (id, row) in self.correctness.iter() {
            let _ = write!(
                output,
                "correctness#{} owner={} property={} ordinal={} anchor=",
                id.index(),
                row.owner.index(),
                row.property.index(),
                row.ordinal,
            );
            write_anchor(&mut output, &row.source_anchor);
            let _ = writeln!(output, " obligation={}", row.obligation.index());
        }
        output
    }

    // Rationale: installation must reauthenticate all four lower handoffs and both owned outputs.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn validate_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        source_context: &SourceBindingContextHandoff,
        source_type: &SourceTypeApplicationHandoff,
        source_term: &SourcePrimaryTermHandoff,
        source_atomic_formula: &SourceAtomicFormulaHandoff,
        initial_obligations: &InitialObligationTable,
        arena: &TypedArena,
    ) -> Result<(), SourcePredicateDefinitionError> {
        validate_dependency_identity(
            source_id,
            module_id,
            source_context,
            source_type,
            source_term,
            source_atomic_formula,
            arena,
        )?;
        if self.source_id != source_id || &self.module_id != module_id {
            return Err(SourcePredicateDefinitionError::SourceIdentityMismatch);
        }
        if self.source_context_fingerprint != source_context.debug_text()
            || self.source_type_fingerprint != source_type.debug_text()
            || self.source_term_fingerprint != source_term.debug_text()
            || self.source_atomic_formula_fingerprint != source_atomic_formula.debug_text()
        {
            return Err(SourcePredicateDefinitionError::DependencyMismatch);
        }
        validate_handoff_rows(
            self,
            source_context,
            source_type,
            source_term,
            source_atomic_formula,
            initial_obligations,
            arena,
        )
    }
}

/// Transactional producer result retaining the obligation baseline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePredicateDefinitionProjection {
    base_initial_obligations: InitialObligationTable,
    handoff: SourcePredicateDefinitionHandoff,
    initial_obligations: InitialObligationTable,
}

impl SourcePredicateDefinitionProjection {
    pub const fn base_initial_obligations(&self) -> &InitialObligationTable {
        &self.base_initial_obligations
    }

    pub const fn handoff(&self) -> &SourcePredicateDefinitionHandoff {
        &self.handoff
    }

    pub const fn initial_obligations(&self) -> &InitialObligationTable {
        &self.initial_obligations
    }

    pub fn into_parts(
        self,
    ) -> (
        InitialObligationTable,
        SourcePredicateDefinitionHandoff,
        InitialObligationTable,
    ) {
        (
            self.base_initial_obligations,
            self.handoff,
            self.initial_obligations,
        )
    }
}

/// Fail-closed predicate-definition validation errors.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourcePredicateDefinitionError {
    SourceIdentityMismatch,
    DependencyMismatch,
    InvalidResolverDefinition,
    InvalidDefinition { index: usize },
    InvalidParameter { index: usize },
    InvalidGuard { index: usize },
    InvalidProperty { index: usize },
    InvalidCorrectness { index: usize },
    InvalidObligation,
    InvalidArenaOwnership,
    UnsupportedTaskShape,
}

impl fmt::Display for SourcePredicateDefinitionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SourceIdentityMismatch => {
                formatter.write_str("predicate-definition source identity mismatch")
            }
            Self::DependencyMismatch => {
                formatter.write_str("predicate-definition dependency mismatch")
            }
            Self::InvalidResolverDefinition => {
                formatter.write_str("invalid predicate resolver definition")
            }
            Self::InvalidDefinition { index } => {
                write!(formatter, "invalid source predicate definition {index}")
            }
            Self::InvalidParameter { index } => {
                write!(formatter, "invalid source predicate parameter {index}")
            }
            Self::InvalidGuard { index } => {
                write!(formatter, "invalid source predicate guard {index}")
            }
            Self::InvalidProperty { index } => {
                write!(formatter, "invalid source predicate property {index}")
            }
            Self::InvalidCorrectness { index } => {
                write!(
                    formatter,
                    "invalid source predicate correctness link {index}"
                )
            }
            Self::InvalidObligation => {
                formatter.write_str("invalid predicate-property correctness obligation")
            }
            Self::InvalidArenaOwnership => {
                formatter.write_str("invalid predicate-definition typed-arena ownership")
            }
            Self::UnsupportedTaskShape => {
                formatter.write_str("unsupported predicate-definition task shape")
            }
        }
    }
}

impl Error for SourcePredicateDefinitionError {}

/// Builds authenticated predicate-definition handoffs.
pub struct SourcePredicateDefinitionProducer;

impl SourcePredicateDefinitionProducer {
    // Rationale: the public transaction makes every frozen dependency explicit.
    #[allow(clippy::too_many_arguments)]
    pub fn build(
        input: SourcePredicateDefinitionHandoffInput,
        env: &SymbolEnv,
        source_context: &SourceBindingContextHandoff,
        source_type: &SourceTypeApplicationHandoff,
        source_term: &SourcePrimaryTermHandoff,
        source_atomic_formula: &SourceAtomicFormulaHandoff,
        base_initial_obligations: &InitialObligationTable,
        arena: &TypedArena,
    ) -> Result<SourcePredicateDefinitionProjection, SourcePredicateDefinitionError> {
        validate_dependency_identity(
            input.source_id,
            &input.module_id,
            source_context,
            source_type,
            source_term,
            source_atomic_formula,
            arena,
        )?;
        if env.module_id() != &input.module_id {
            return Err(SourcePredicateDefinitionError::SourceIdentityMismatch);
        }
        validate_input_shape(&input)?;
        let origin = validate_resolver_definition(&input, env)?;
        validate_input_rows(
            &input,
            source_context,
            source_type,
            source_term,
            source_atomic_formula,
            arena,
        )?;

        let mut initial_obligations = base_initial_obligations.clone();
        let property = &input.properties[0];
        let obligation = initial_obligations.insert(InitialObligationDraft {
            kind: InitialObligationKind::PredicatePropertyCorrectness,
            owner: property.site.clone(),
            source_range: property.source_range,
            assumptions: Vec::new(),
            goal: InitialObligationGoal::new("source.definition.predicate.correctness:property=0"),
            provenance: InitialObligationProvenance::new(
                "source.definition.predicate:definition=0:property=0",
            ),
            status: InitialObligationStatus::Pending,
        });
        if obligation.index() != base_initial_obligations.len() {
            return Err(SourcePredicateDefinitionError::InvalidObligation);
        }

        let definitions = SourcePredicateDefinitionTable {
            rows: input
                .definitions
                .into_iter()
                .enumerate()
                .map(|(index, row)| SourcePredicateDefinition {
                    id: SourcePredicateDefinitionId::new(index),
                    symbol: row.symbol,
                    definition: row.definition,
                    contribution: row.contribution,
                    site: row.site,
                    source_range: row.source_range,
                    source_ordinal: row.source_ordinal,
                    context: row.context,
                    recovery: row.recovery,
                    spelling: row.spelling,
                    definiens: row.definiens,
                    origin: origin.clone(),
                })
                .collect(),
        };
        let parameters = SourcePredicateParameterTable {
            rows: input
                .parameters
                .into_iter()
                .enumerate()
                .map(|(index, row)| SourcePredicateParameter {
                    id: SourcePredicateParameterId::new(index),
                    owner: row.owner,
                    ordinal: row.ordinal,
                    binding: row.binding,
                    written_type: row.written_type,
                    site: row.site,
                    source_range: row.source_range,
                    declaration_range: row.declaration_range,
                    context: row.context,
                    recovery: row.recovery,
                    spelling: row.spelling,
                })
                .collect(),
        };
        let guards = SourcePredicateGuardTable {
            rows: input
                .guards
                .into_iter()
                .enumerate()
                .map(|(index, row)| SourcePredicateGuard {
                    id: SourcePredicateGuardId::new(index),
                    owner: row.owner,
                    ordinal: row.ordinal,
                    formula: row.formula,
                    site: row.site,
                    source_range: row.source_range,
                    context: row.context,
                    recovery: row.recovery,
                    spelling: row.spelling,
                })
                .collect(),
        };
        let properties = SourcePredicatePropertyTable {
            rows: input
                .properties
                .into_iter()
                .enumerate()
                .map(|(index, row)| SourcePredicateProperty {
                    id: SourcePredicatePropertyId::new(index),
                    owner: row.owner,
                    ordinal: row.ordinal,
                    kind: row.kind,
                    site: row.site,
                    source_range: row.source_range,
                    justification: row.justification,
                    recovery: row.recovery,
                    spelling: row.spelling,
                })
                .collect(),
        };
        let correctness = SourcePredicateCorrectnessTable {
            rows: input
                .correctness
                .into_iter()
                .enumerate()
                .map(|(index, row)| SourcePredicateCorrectness {
                    id: SourcePredicateCorrectnessId::new(index),
                    owner: row.owner,
                    property: row.property,
                    ordinal: row.ordinal,
                    source_anchor: row.source_anchor,
                    obligation,
                })
                .collect(),
        };
        let handoff = SourcePredicateDefinitionHandoff {
            source_id: input.source_id,
            module_id: input.module_id,
            resolver_identity: SourcePredicateResolverIdentity {
                symbol: definitions.rows[0].symbol.clone(),
                definition: definitions.rows[0].definition,
                contribution: definitions.rows[0].contribution,
                origin: definitions.rows[0].origin.clone(),
            },
            source_context_fingerprint: source_context.debug_text(),
            source_type_fingerprint: source_type.debug_text(),
            source_term_fingerprint: source_term.debug_text(),
            source_atomic_formula_fingerprint: source_atomic_formula.debug_text(),
            definitions,
            parameters,
            guards,
            properties,
            correctness,
        };
        validate_handoff_rows(
            &handoff,
            source_context,
            source_type,
            source_term,
            source_atomic_formula,
            &initial_obligations,
            arena,
        )?;
        Ok(SourcePredicateDefinitionProjection {
            base_initial_obligations: base_initial_obligations.clone(),
            handoff,
            initial_obligations,
        })
    }
}

// Rationale: dependency identity is meaningful only across the complete lower bundle.
#[allow(clippy::too_many_arguments)]
fn validate_dependency_identity(
    source_id: SourceId,
    module_id: &ModuleId,
    source_context: &SourceBindingContextHandoff,
    source_type: &SourceTypeApplicationHandoff,
    source_term: &SourcePrimaryTermHandoff,
    source_atomic_formula: &SourceAtomicFormulaHandoff,
    arena: &TypedArena,
) -> Result<(), SourcePredicateDefinitionError> {
    if source_context.source_id() != source_id
        || source_context.module_id() != module_id
        || source_type.source_id() != source_id
        || source_type.module_id() != module_id
        || source_term.source_id() != source_id
        || source_term.module_id() != module_id
        || source_atomic_formula.source_id() != source_id
        || source_atomic_formula.module_id() != module_id
    {
        return Err(SourcePredicateDefinitionError::SourceIdentityMismatch);
    }
    source_type
        .validate_installation(source_id, module_id, arena)
        .map_err(|_| SourcePredicateDefinitionError::DependencyMismatch)?;
    source_term
        .validate_installation(source_id, module_id, arena)
        .map_err(|_| SourcePredicateDefinitionError::DependencyMismatch)?;
    source_atomic_formula
        .validate_installation(source_id, module_id, source_term, None, None, None, arena)
        .map_err(|_| SourcePredicateDefinitionError::DependencyMismatch)?;
    Ok(())
}

fn validate_input_shape(
    input: &SourcePredicateDefinitionHandoffInput,
) -> Result<(), SourcePredicateDefinitionError> {
    if input.definitions.len() != 1
        || input.parameters.len() != 2
        || input.guards.len() != 1
        || input.properties.len() != 1
        || input.correctness.len() != 1
    {
        return Err(SourcePredicateDefinitionError::UnsupportedTaskShape);
    }
    Ok(())
}

fn validate_resolver_definition(
    input: &SourcePredicateDefinitionHandoffInput,
    env: &SymbolEnv,
) -> Result<SemanticOrigin, SourcePredicateDefinitionError> {
    let row = &input.definitions[0];
    let symbol = env
        .symbols()
        .get(&row.symbol)
        .ok_or(SourcePredicateDefinitionError::InvalidResolverDefinition)?;
    let definition = env
        .definitions()
        .get(row.definition)
        .ok_or(SourcePredicateDefinitionError::InvalidResolverDefinition)?;
    let contribution = env
        .contributions()
        .get(row.contribution)
        .ok_or(SourcePredicateDefinitionError::InvalidResolverDefinition)?;
    let expected_anchor = SourceAnchor::Range(range(input.source_id, 0, 164));
    if row.definition.index() != 0
        || row.symbol.module() != &input.module_id
        || symbol.symbol() != &row.symbol
        || symbol.kind() != SymbolKind::Predicate
        || symbol.visibility() != Visibility::Public
        || symbol.export_status() != ExportStatus::Exported
        || symbol.primary_spelling() != "x task259_rel y"
        || symbol.notation_spelling() != Some("x task259_rel y")
        || symbol.contribution() != row.contribution
        || symbol.origin() != definition.origin()
        || symbol.signature() != definition.signature()
        || !symbol.relations().is_empty()
        || definition.id() != row.definition
        || definition.symbol() != &row.symbol
        || definition.kind() != DefinitionKind::Predicate
        || definition.visibility() != Visibility::Public
        || !definition.parameters().is_empty()
        || !definition.binders().is_empty()
        || definition.arity().is_some()
        || definition.notation_shape() != Some("x task259_rel y")
        || definition.doc_attachment().is_some()
        || definition.contribution() != row.contribution
        || definition.conflict().is_some()
        || !definition.dependencies().is_empty()
        || contribution.module() != &input.module_id
        || contribution.kind()
            != &(ContributionKind::LocalSource {
                source_id: input.source_id,
            })
        || contribution.anchor() != &expected_anchor
        || !contribution.effects().symbols().contains(&row.symbol)
        || !contribution
            .effects()
            .definitions()
            .contains(&row.definition)
        || !normal_origin(
            definition.origin(),
            input.source_id,
            &input.module_id,
            row.source_range,
            &[4, 0, 8, 0],
        )
    {
        return Err(SourcePredicateDefinitionError::InvalidResolverDefinition);
    }
    Ok(definition.origin().clone())
}

// Rationale: row validation cross-checks each input against every frozen lower owner.
#[allow(clippy::too_many_arguments)]
fn validate_input_rows(
    input: &SourcePredicateDefinitionHandoffInput,
    source_context: &SourceBindingContextHandoff,
    source_type: &SourceTypeApplicationHandoff,
    source_term: &SourcePrimaryTermHandoff,
    source_atomic_formula: &SourceAtomicFormulaHandoff,
    arena: &TypedArena,
) -> Result<(), SourcePredicateDefinitionError> {
    let source = input.source_id;
    let definition = &input.definitions[0];
    if definition.source_range != range(source, 61, 122)
        || definition.source_ordinal != 0
        || definition.recovery != SourcePredicateDefinitionRecovery::Normal
        || definition.spelling != "pred Task259PredicateDefinition: x task259_rel y means x = y;"
        || definition.definiens != SourceAtomicFormulaId::new(1)
    {
        return Err(SourcePredicateDefinitionError::InvalidDefinition { index: 0 });
    }
    let local_context = validate_context_profile(input, source_context)?;
    if !valid_site(
        &definition.site,
        definition.source_range,
        "source.definition.predicate",
        local_context,
        arena,
    ) {
        return Err(SourcePredicateDefinitionError::InvalidArenaOwnership);
    }
    let context = definition.context;
    let parameter_ranges = [(13, 26, 17, 18), (29, 42, 33, 34)];
    let parameter_spellings = ["let x be set;", "let y be set;"];
    for (index, row) in input.parameters.iter().enumerate() {
        let (start, end, declaration_start, declaration_end) = parameter_ranges[index];
        if row.owner != SourcePredicateDefinitionId::new(0)
            || row.ordinal != index
            || row.written_type != SourceTypeApplicationId::new(index)
            || row.source_range != range(source, start, end)
            || row.declaration_range != range(source, declaration_start, declaration_end)
            || row.context != context
            || row.recovery != SourcePredicateDefinitionRecovery::Normal
            || row.spelling != parameter_spellings[index]
        {
            return Err(SourcePredicateDefinitionError::InvalidParameter { index });
        }
        if !valid_site(
            &row.site,
            row.declaration_range,
            "source.definition.predicate.parameter",
            local_context,
            arena,
        ) {
            return Err(SourcePredicateDefinitionError::InvalidArenaOwnership);
        }
    }
    validate_type_profile(input, source_type, local_context, arena)?;
    validate_term_and_atomic_profile(input, source_term, source_atomic_formula)?;

    let guard = &input.guards[0];
    if guard.owner != SourcePredicateDefinitionId::new(0)
        || guard.ordinal != 0
        || guard.formula != SourceAtomicFormulaId::new(0)
        || guard.source_range != range(source, 45, 58)
        || guard.context != context
        || guard.recovery != SourcePredicateDefinitionRecovery::Normal
        || guard.spelling != "assume x = x;"
    {
        return Err(SourcePredicateDefinitionError::InvalidGuard { index: 0 });
    }
    if !valid_site(
        &guard.site,
        guard.source_range,
        "source.definition.predicate.guard",
        local_context,
        arena,
    ) {
        return Err(SourcePredicateDefinitionError::InvalidArenaOwnership);
    }
    let property = &input.properties[0];
    if property.owner != SourcePredicateDefinitionId::new(0)
        || property.ordinal != 0
        || property.kind != SourcePredicatePropertyKind::Symmetry
        || property.source_range != range(source, 125, 159)
        || property.justification != SourceAnchor::Range(range(source, 134, 158))
        || property.recovery != SourcePredicateDefinitionRecovery::Normal
        || property.spelling != "symmetry by computation(steps: 1);"
        || definition.source_range.end >= property.source_range.start
    {
        return Err(SourcePredicateDefinitionError::InvalidProperty { index: 0 });
    }
    if !valid_site(
        &property.site,
        property.source_range,
        "source.definition.predicate.property",
        local_context,
        arena,
    ) {
        return Err(SourcePredicateDefinitionError::InvalidArenaOwnership);
    }
    let correctness = &input.correctness[0];
    if correctness.owner != SourcePredicateDefinitionId::new(0)
        || correctness.property != SourcePredicatePropertyId::new(0)
        || correctness.ordinal != 0
        || correctness.source_anchor != SourceAnchor::Range(range(source, 125, 159))
    {
        return Err(SourcePredicateDefinitionError::InvalidCorrectness { index: 0 });
    }
    Ok(())
}

fn validate_context_profile(
    input: &SourcePredicateDefinitionHandoffInput,
    source_context: &SourceBindingContextHandoff,
) -> Result<LocalTypeContextId, SourcePredicateDefinitionError> {
    let source = input.source_id;
    let definition_context = input.definitions[0].context;
    let binding_env = source_context.binding_env();
    if source_context.items().len() != 1
        || source_context.declarations().len() != 2
        || source_context.context_links().len() != 2
        || source_context.local_contexts().len() != 2
        || binding_env.contexts().len() != 2
        || binding_env.bindings().len() != 2
        || !binding_env.diagnostics().is_empty()
    {
        return Err(SourcePredicateDefinitionError::DependencyMismatch);
    }
    let Some(item) = source_context
        .items()
        .get(crate::source_context::SourceItemId::new(0))
    else {
        return Err(SourcePredicateDefinitionError::DependencyMismatch);
    };
    let module_binding_context = BindingContextId::new(0);
    let module_local_context = LocalTypeContextId::new(0);
    let Some(module_link) = source_context.context_links().get(module_binding_context) else {
        return Err(SourcePredicateDefinitionError::DependencyMismatch);
    };
    let Some(definition_link) = source_context.context_links().get(definition_context) else {
        return Err(SourcePredicateDefinitionError::DependencyMismatch);
    };
    let definition_local_context = definition_link.local_context;
    if item.id != crate::source_context::SourceItemId::new(0)
        || item.shell_ordinal != 0
        || item.role != SourceItemRole::DefinitionBlock
        || item.source_range != range(source, 0, 164)
        || item.parent.is_some()
        || item.visibility != SourceItemVisibility::Unspecified
        || item.local_scope.is_none()
        || item.recovery != SourceItemRecovery::Normal
        || item.binding_context != definition_context
        || item.binding_context.index() != 1
        || definition_local_context != LocalTypeContextId::new(1)
        || item.local_context != definition_local_context
        || item.predecessor.is_some()
        || module_link.binding_context != module_binding_context
        || module_link.local_context != module_local_context
        || module_link.item.is_some()
        || definition_link.binding_context != definition_context
        || definition_link.local_context != definition_local_context
        || definition_link.local_context != item.local_context
        || definition_link.item != Some(item.id)
    {
        return Err(SourcePredicateDefinitionError::DependencyMismatch);
    }
    for (index, row) in input.parameters.iter().enumerate() {
        let declaration = source_context
            .declarations()
            .get(crate::source_context::SourceDeclarationId::new(index))
            .ok_or(SourcePredicateDefinitionError::InvalidParameter { index })?;
        if declaration.item.index() != 0
            || declaration.binding != row.binding
            || declaration.source_ordinal != index
            || declaration.spelling != if index == 0 { "x" } else { "y" }
            || declaration.declaration_range != row.declaration_range
            || declaration.written_type_range
                != if index == 0 {
                    range(source, 22, 25)
                } else {
                    range(source, 38, 41)
                }
            || declaration.site != row.site
            || !matches!(
                declaration.role,
                SourceBindingSiteRole::DefinitionParameter { .. }
            )
            || declaration.binding_context != definition_context
            || declaration.local_context != definition_local_context
            || declaration.shadowed_binding.is_some()
            || declaration.predecessor.map(|id| id.index()) != index.checked_sub(1)
        {
            return Err(SourcePredicateDefinitionError::InvalidParameter { index });
        }
        let binding = binding_env
            .bindings()
            .get(row.binding)
            .ok_or(SourcePredicateDefinitionError::InvalidParameter { index })?;
        if binding.id != row.binding
            || binding.spelling != if index == 0 { "x" } else { "y" }
            || binding.kind != BindingKind::DefinitionParameter
            || binding.owner_context != definition_context
            || binding.declaration_range != row.declaration_range
            || binding.visible_after_ordinal != index
            || binding.type_site
                != BindingTypeSite::Source(if index == 0 {
                    range(source, 22, 25)
                } else {
                    range(source, 38, 41)
                })
            || binding.status != BindingStatus::Active
            || !binding.captured.identities().is_empty()
            || !binding.diagnostics.is_empty()
            || binding.recovery != BindingRecoveryState::Normal
        {
            return Err(SourcePredicateDefinitionError::InvalidParameter { index });
        }
    }
    Ok(definition_local_context)
}

fn validate_type_profile(
    input: &SourcePredicateDefinitionHandoffInput,
    source_type: &SourceTypeApplicationHandoff,
    local_context: LocalTypeContextId,
    arena: &TypedArena,
) -> Result<(), SourcePredicateDefinitionError> {
    if source_type.applications().len() != 2
        || source_type.expressions().len() != 2
        || !source_type.arguments().is_empty()
    {
        return Err(SourcePredicateDefinitionError::DependencyMismatch);
    }
    let source = input.source_id;
    for index in 0..2 {
        let application = source_type
            .applications()
            .get(SourceTypeApplicationId::new(index))
            .ok_or(SourcePredicateDefinitionError::InvalidParameter { index })?;
        let expression = source_type
            .expressions()
            .get(SourceTypeExpressionId::new(index))
            .ok_or(SourcePredicateDefinitionError::InvalidParameter { index })?;
        let written_range = if index == 0 {
            range(source, 22, 25)
        } else {
            range(source, 38, 41)
        };
        let expression_site = TypedSiteRef::Node(TypedNodeId::new(if index == 0 { 1 } else { 4 }));
        let head_site = TypedSiteRef::Node(TypedNodeId::new(if index == 0 { 0 } else { 3 }));
        if application.binding() != input.parameters[index].binding
            || application.source_ordinal() != index
            || application.root() != SourceTypeExpressionId::new(index)
            || expression.id() != SourceTypeExpressionId::new(index)
            || expression.source_id() != source
            || expression.module_id() != &input.module_id
            || expression.site() != &expression_site
            || expression.source_range() != written_range
            || expression.spelling() != "set"
            || expression.head_site() != &head_site
            || expression.head_range() != written_range
            || expression.head_spelling() != "set"
            || expression.form() != SourceTypeApplicationForm::Bare
            || expression.head() != &SourceTypeHead::BuiltinSet
            || expression.recovery() != NodeRecoveryState::Normal
        {
            return Err(SourcePredicateDefinitionError::InvalidParameter { index });
        }
        if !valid_site(
            expression.site(),
            written_range,
            "source.type.expression",
            local_context,
            arena,
        ) || !valid_site(
            expression.head_site(),
            written_range,
            "source.type.head",
            local_context,
            arena,
        ) {
            return Err(SourcePredicateDefinitionError::InvalidArenaOwnership);
        }
    }
    Ok(())
}

fn validate_term_and_atomic_profile(
    input: &SourcePredicateDefinitionHandoffInput,
    source_term: &SourcePrimaryTermHandoff,
    source_atomic_formula: &SourceAtomicFormulaHandoff,
) -> Result<(), SourcePredicateDefinitionError> {
    if source_term.terms().len() != 4
        || source_term.references().len() != 4
        || !source_term.numeric_type_requests().is_empty()
        || source_atomic_formula.formulas().len() != 2
        || !source_atomic_formula.wrappers().is_empty()
        || !source_atomic_formula.predicate_segments().is_empty()
        || !source_atomic_formula.predicate_heads().is_empty()
        || !source_atomic_formula.candidates().is_empty()
        || !source_atomic_formula.type_sites().is_empty()
        || !source_atomic_formula.attributes().is_empty()
        || source_atomic_formula.edges().len() != 4
        || source_atomic_formula.requests().len() != 4
    {
        return Err(SourcePredicateDefinitionError::DependencyMismatch);
    }
    let source = input.source_id;
    let context = input.definitions[0].context;
    let term_ranges = [(52, 53), (56, 57), (116, 117), (120, 121)];
    let term_spellings = ["x", "x", "x", "y"];
    let binding_indexes = [0, 0, 0, 1];
    for index in 0..4 {
        let term = source_term
            .terms()
            .get(SourcePrimaryTermId::new(index))
            .ok_or(SourcePredicateDefinitionError::DependencyMismatch)?;
        let reference = source_term
            .references()
            .get(crate::source_term::SourcePrimaryTermReferenceId::new(index))
            .ok_or(SourcePredicateDefinitionError::DependencyMismatch)?;
        if term.source_range() != range(source, term_ranges[index].0, term_ranges[index].1)
            || term.source_ordinal() != index
            || term.context() != context
            || term.recovery() != SourcePrimaryTermRecovery::Normal
            || term.spelling() != term_spellings[index]
            || term.kind() != SourcePrimaryTermKind::VariableReference
            || term.role() != SourcePrimaryTermRole::Value
            || term.parent().is_some()
            || reference.term() != SourcePrimaryTermId::new(index)
            || reference.binding() != input.parameters[binding_indexes[index]].binding
            || reference.role() != SourcePrimaryTermReferenceRole::Variable
        {
            return Err(SourcePredicateDefinitionError::DependencyMismatch);
        }
    }
    let formula_ranges = [(52, 57), (116, 121)];
    let formula_spellings = ["x = x", "x = y"];
    for formula_index in 0..2 {
        let formula = source_atomic_formula
            .formulas()
            .get(SourceAtomicFormulaId::new(formula_index))
            .ok_or(SourcePredicateDefinitionError::DependencyMismatch)?;
        if formula.source_range()
            != range(
                source,
                formula_ranges[formula_index].0,
                formula_ranges[formula_index].1,
            )
            || formula.source_ordinal() != formula_index
            || formula.context() != context
            || formula.recovery() != SourceAtomicFormulaRecovery::Normal
            || formula.spelling() != formula_spellings[formula_index]
            || formula.kind() != SourceAtomicFormulaKind::Equality
        {
            return Err(SourcePredicateDefinitionError::DependencyMismatch);
        }
        for operand in 0..2 {
            let flat_index = formula_index * 2 + operand;
            let edge = source_atomic_formula
                .edges()
                .get(SourceAtomicEdgeId::new(flat_index))
                .ok_or(SourcePredicateDefinitionError::DependencyMismatch)?;
            let request = source_atomic_formula
                .requests()
                .get(crate::source_atomic_formula::SourceAtomicRequestId::new(
                    flat_index,
                ))
                .ok_or(SourcePredicateDefinitionError::DependencyMismatch)?;
            let role = if operand == 0 {
                SourceAtomicEdgeRole::BuiltinLeftOperand
            } else {
                SourceAtomicEdgeRole::BuiltinRightOperand
            };
            if edge.formula() != SourceAtomicFormulaId::new(formula_index)
                || edge.ordinal() != operand
                || edge.role() != role
                || edge.target()
                    != SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(flat_index))
                || request.formula() != SourceAtomicFormulaId::new(formula_index)
                || request.ordinal() != operand
                || request.kind() != SourceAtomicRequestKind::OperandExpectedType
                || request.edge() != Some(SourceAtomicEdgeId::new(flat_index))
                || request.candidate().is_some()
                || request.type_site().is_some()
                || request.attribute().is_some()
            {
                return Err(SourcePredicateDefinitionError::DependencyMismatch);
            }
        }
    }
    Ok(())
}

fn validate_handoff_rows(
    handoff: &SourcePredicateDefinitionHandoff,
    source_context: &SourceBindingContextHandoff,
    source_type: &SourceTypeApplicationHandoff,
    source_term: &SourcePrimaryTermHandoff,
    source_atomic_formula: &SourceAtomicFormulaHandoff,
    initial_obligations: &InitialObligationTable,
    arena: &TypedArena,
) -> Result<(), SourcePredicateDefinitionError> {
    validate_handoff_dense_ids(handoff)?;
    validate_handoff_resolver_identity(handoff)?;
    let input = SourcePredicateDefinitionHandoffInput {
        source_id: handoff.source_id,
        module_id: handoff.module_id.clone(),
        definitions: handoff
            .definitions
            .iter()
            .map(|(_, row)| SourcePredicateDefinitionInput {
                symbol: row.symbol.clone(),
                definition: row.definition,
                contribution: row.contribution,
                site: row.site.clone(),
                source_range: row.source_range,
                source_ordinal: row.source_ordinal,
                context: row.context,
                recovery: row.recovery,
                spelling: row.spelling.clone(),
                definiens: row.definiens,
            })
            .collect(),
        parameters: handoff
            .parameters
            .iter()
            .map(|(_, row)| SourcePredicateParameterInput {
                owner: row.owner,
                ordinal: row.ordinal,
                binding: row.binding,
                written_type: row.written_type,
                site: row.site.clone(),
                source_range: row.source_range,
                declaration_range: row.declaration_range,
                context: row.context,
                recovery: row.recovery,
                spelling: row.spelling.clone(),
            })
            .collect(),
        guards: handoff
            .guards
            .iter()
            .map(|(_, row)| SourcePredicateGuardInput {
                owner: row.owner,
                ordinal: row.ordinal,
                formula: row.formula,
                site: row.site.clone(),
                source_range: row.source_range,
                context: row.context,
                recovery: row.recovery,
                spelling: row.spelling.clone(),
            })
            .collect(),
        properties: handoff
            .properties
            .iter()
            .map(|(_, row)| SourcePredicatePropertyInput {
                owner: row.owner,
                ordinal: row.ordinal,
                kind: row.kind,
                site: row.site.clone(),
                source_range: row.source_range,
                justification: row.justification.clone(),
                recovery: row.recovery,
                spelling: row.spelling.clone(),
            })
            .collect(),
        correctness: handoff
            .correctness
            .iter()
            .map(|(_, row)| SourcePredicateCorrectnessInput {
                owner: row.owner,
                property: row.property,
                ordinal: row.ordinal,
                source_anchor: row.source_anchor.clone(),
            })
            .collect(),
    };
    validate_input_shape(&input)?;
    validate_input_rows(
        &input,
        source_context,
        source_type,
        source_term,
        source_atomic_formula,
        arena,
    )?;
    let definition = &handoff.definitions.rows[0];
    if !normal_origin(
        &definition.origin,
        handoff.source_id,
        &handoff.module_id,
        definition.source_range,
        &[4, 0, 8, 0],
    ) {
        return Err(SourcePredicateDefinitionError::InvalidDefinition { index: 0 });
    }
    let correctness = &handoff.correctness.rows[0];
    let property = &handoff.properties.rows[0];
    let obligation = initial_obligations
        .get(correctness.obligation)
        .ok_or(SourcePredicateDefinitionError::InvalidObligation)?;
    if correctness.obligation.index() + 1 != initial_obligations.len()
        || obligation.id != correctness.obligation
        || obligation.kind != InitialObligationKind::PredicatePropertyCorrectness
        || obligation.owner != property.site
        || obligation.source_range != property.source_range
        || !obligation.assumptions.is_empty()
        || obligation.goal.as_str() != "source.definition.predicate.correctness:property=0"
        || obligation.provenance.as_str() != "source.definition.predicate:definition=0:property=0"
        || obligation.status != InitialObligationStatus::Pending
    {
        return Err(SourcePredicateDefinitionError::InvalidObligation);
    }
    Ok(())
}

fn validate_handoff_dense_ids(
    handoff: &SourcePredicateDefinitionHandoff,
) -> Result<(), SourcePredicateDefinitionError> {
    for (index, row) in handoff.definitions.rows.iter().enumerate() {
        if row.id != SourcePredicateDefinitionId::new(index) {
            return Err(SourcePredicateDefinitionError::InvalidDefinition { index });
        }
    }
    for (index, row) in handoff.parameters.rows.iter().enumerate() {
        if row.id != SourcePredicateParameterId::new(index) {
            return Err(SourcePredicateDefinitionError::InvalidParameter { index });
        }
    }
    for (index, row) in handoff.guards.rows.iter().enumerate() {
        if row.id != SourcePredicateGuardId::new(index) {
            return Err(SourcePredicateDefinitionError::InvalidGuard { index });
        }
    }
    for (index, row) in handoff.properties.rows.iter().enumerate() {
        if row.id != SourcePredicatePropertyId::new(index) {
            return Err(SourcePredicateDefinitionError::InvalidProperty { index });
        }
    }
    for (index, row) in handoff.correctness.rows.iter().enumerate() {
        if row.id != SourcePredicateCorrectnessId::new(index) {
            return Err(SourcePredicateDefinitionError::InvalidCorrectness { index });
        }
    }
    Ok(())
}

fn validate_handoff_resolver_identity(
    handoff: &SourcePredicateDefinitionHandoff,
) -> Result<(), SourcePredicateDefinitionError> {
    let Some(definition) = handoff.definitions.rows.first() else {
        return Err(SourcePredicateDefinitionError::UnsupportedTaskShape);
    };
    let expected_fqn = format!(
        "{}::{}::{}",
        handoff.module_id.package().as_str(),
        handoff.module_id.path().as_str(),
        definition.symbol.local().as_str(),
    );
    if definition.symbol != handoff.resolver_identity.symbol
        || definition.definition != handoff.resolver_identity.definition
        || definition.contribution != handoff.resolver_identity.contribution
        || definition.origin != handoff.resolver_identity.origin
        || definition.symbol.module() != &handoff.module_id
        || definition.symbol.fqn().as_str() != expected_fqn
        || definition.definition.index() != 0
        || definition.contribution.index() != 0
    {
        return Err(SourcePredicateDefinitionError::InvalidResolverDefinition);
    }
    Ok(())
}

fn normal_origin(
    origin: &SemanticOrigin,
    source_id: SourceId,
    module_id: &ModuleId,
    source_range: SourceRange,
    structural_path: &[u32],
) -> bool {
    origin.source_id() == source_id
        && origin.module_id() == module_id
        && origin.anchor() == &SourceAnchor::Range(source_range)
        && origin.structural_path() == structural_path
        && origin.import_edge().is_none()
        && !origin.is_recovered()
}

fn valid_site(
    site: &TypedSiteRef,
    source_range: SourceRange,
    kind: &str,
    local_context: LocalTypeContextId,
    arena: &TypedArena,
) -> bool {
    let TypedSiteRef::Node(node_id) = site else {
        return false;
    };
    arena.node(*node_id).is_some_and(|node| {
        node.kind.as_str() == kind
            && node.anchor == SourceAnchor::Range(source_range)
            && node.recovery == NodeRecoveryState::Normal
            && node.links.context == Some(local_context)
    })
}

const fn range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
    SourceRange {
        source_id,
        start,
        end,
    }
}

const fn property_kind_key(kind: SourcePredicatePropertyKind) -> &'static str {
    match kind {
        SourcePredicatePropertyKind::Symmetry => "symmetry",
    }
}

const fn recovery_key(recovery: SourcePredicateDefinitionRecovery) -> &'static str {
    match recovery {
        SourcePredicateDefinitionRecovery::Normal => "normal",
        SourcePredicateDefinitionRecovery::Degraded => "degraded",
    }
}

fn write_site(output: &mut String, site: &TypedSiteRef) {
    match site {
        TypedSiteRef::Node(node) => {
            let _ = write!(output, "node#{}", node.index());
        }
        TypedSiteRef::Role { node, .. } => {
            let _ = write!(output, "role#{}", node.index());
        }
    }
}

fn write_anchor(output: &mut String, anchor: &SourceAnchor) {
    match anchor {
        SourceAnchor::Range(range) => {
            let _ = write!(output, "range:{}..{}", range.start, range.end);
        }
        SourceAnchor::Point { offset, .. } => {
            let _ = write!(output, "point:{offset}");
        }
        SourceAnchor::Generated(_) => output.push_str("generated"),
        _ => output.push_str("unsupported"),
    }
}

fn write_anchor_range(output: &mut String, anchor: &SourceAnchor) {
    match anchor {
        SourceAnchor::Range(range) => {
            let _ = write!(output, "{}..{}", range.start, range.end);
        }
        SourceAnchor::Point { offset, .. } => {
            let _ = write!(output, "{offset}..{offset}");
        }
        SourceAnchor::Generated(_) => output.push_str("generated"),
        _ => output.push_str("unsupported"),
    }
}

#[cfg(test)]
#[path = "../tests/support/source_predicate_definition_unit.rs"]
pub(crate) mod tests;
