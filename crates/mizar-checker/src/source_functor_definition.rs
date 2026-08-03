//! Syntax-free functor-definition intake for checker phase 6.

use crate::{
    binding_env::{
        BindingContextId, BindingId, BindingKind, BindingRecoveryState, BindingStatus,
        BindingTypeSite,
    },
    source_application::{SourceFunctorApplicationHandoff, SourceFunctorApplicationId},
    source_atomic_formula::{
        SourceAtomicEdgeId, SourceAtomicEdgeRole, SourceAtomicFormulaHandoff,
        SourceAtomicFormulaId, SourceAtomicFormulaKind, SourceAtomicFormulaRecovery,
        SourceAtomicRequestKind, SourceAtomicTermTarget,
    },
    source_context::{
        SourceBindingContextHandoff, SourceBindingSiteRole, SourceItemRecovery, SourceItemRole,
        SourceItemVisibility,
    },
    source_set_term::{SourceSetTermHandoff, SourceSetTermId},
    source_structure::{SourceStructureHandoff, SourceStructureTermId},
    source_term::{
        SourcePrimaryTermHandoff, SourcePrimaryTermId, SourcePrimaryTermKind,
        SourcePrimaryTermRecovery, SourcePrimaryTermReferenceId, SourcePrimaryTermReferenceRole,
        SourcePrimaryTermRole,
    },
    source_type::{
        SourceTypeApplicationForm, SourceTypeApplicationHandoff, SourceTypeApplicationId,
        SourceTypeDefinitionReturnId, SourceTypeExpressionId, SourceTypeHead,
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

dense_id!(SourceFunctorDefinitionId);
dense_id!(SourceFunctorParameterId);
dense_id!(SourceFunctorGuardId);
dense_id!(SourceFunctorDefiniensId);
dense_id!(SourceFunctorCorrectnessId);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFunctorDefinitionHandoffInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub definitions: Vec<SourceFunctorDefinitionInput>,
    pub parameters: Vec<SourceFunctorParameterInput>,
    pub guards: Vec<SourceFunctorGuardInput>,
    pub definientia: Vec<SourceFunctorDefiniensInput>,
    pub correctness: Vec<SourceFunctorCorrectnessInput>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFunctorDefinitionInput {
    pub symbol: SymbolId,
    pub definition: DefinitionId,
    pub contribution: SourceContributionId,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub source_ordinal: usize,
    pub context: BindingContextId,
    pub recovery: SourceFunctorDefinitionRecovery,
    pub spelling: String,
    pub style: SourceFunctorDefinitionStyle,
    pub return_type: SourceTypeDefinitionReturnId,
    pub definiens: SourceFunctorDefiniensId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFunctorParameterInput {
    pub ordinal: usize,
    pub binding: BindingId,
    pub written_type: SourceTypeApplicationId,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub declaration_range: SourceRange,
    pub context: BindingContextId,
    pub recovery: SourceFunctorDefinitionRecovery,
    pub spelling: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFunctorGuardInput {
    pub ordinal: usize,
    pub formula: SourceAtomicFormulaId,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub context: BindingContextId,
    pub recovery: SourceFunctorDefinitionRecovery,
    pub spelling: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFunctorDefiniensInput {
    pub owner: SourceFunctorDefinitionId,
    pub ordinal: usize,
    pub target: SourceFunctorDefiniensTarget,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub context: BindingContextId,
    pub recovery: SourceFunctorDefinitionRecovery,
    pub spelling: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFunctorCorrectnessInput {
    pub owner: SourceFunctorDefinitionId,
    pub ordinal: usize,
    pub kind: SourceFunctorCorrectnessKind,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub justification: SourceAnchor,
    pub recovery: SourceFunctorDefinitionRecovery,
    pub spelling: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceFunctorDefinitionStyle {
    Equals,
    Means,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceFunctorDefiniensTarget {
    Primary(SourcePrimaryTermId),
    Application(SourceFunctorApplicationId),
    Structure(SourceStructureTermId),
    SetTerm(SourceSetTermId),
    AtomicFormula(SourceAtomicFormulaId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceFunctorCorrectnessKind {
    Existence,
    Uniqueness,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceFunctorDefinitionRecovery {
    Normal,
    Degraded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFunctorDefinition {
    id: SourceFunctorDefinitionId,
    symbol: SymbolId,
    definition: DefinitionId,
    contribution: SourceContributionId,
    site: TypedSiteRef,
    source_range: SourceRange,
    source_ordinal: usize,
    context: BindingContextId,
    recovery: SourceFunctorDefinitionRecovery,
    spelling: String,
    style: SourceFunctorDefinitionStyle,
    return_type: SourceTypeDefinitionReturnId,
    definiens: SourceFunctorDefiniensId,
    origin: SemanticOrigin,
}

impl SourceFunctorDefinition {
    pub const fn id(&self) -> SourceFunctorDefinitionId {
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
    pub const fn recovery(&self) -> SourceFunctorDefinitionRecovery {
        self.recovery
    }
    pub fn spelling(&self) -> &str {
        &self.spelling
    }
    pub const fn style(&self) -> SourceFunctorDefinitionStyle {
        self.style
    }
    pub const fn return_type(&self) -> SourceTypeDefinitionReturnId {
        self.return_type
    }
    pub const fn definiens(&self) -> SourceFunctorDefiniensId {
        self.definiens
    }
    pub const fn origin(&self) -> &SemanticOrigin {
        &self.origin
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFunctorParameter {
    id: SourceFunctorParameterId,
    ordinal: usize,
    binding: BindingId,
    written_type: SourceTypeApplicationId,
    site: TypedSiteRef,
    source_range: SourceRange,
    declaration_range: SourceRange,
    context: BindingContextId,
    recovery: SourceFunctorDefinitionRecovery,
    spelling: String,
}

impl SourceFunctorParameter {
    pub const fn id(&self) -> SourceFunctorParameterId {
        self.id
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
    pub const fn recovery(&self) -> SourceFunctorDefinitionRecovery {
        self.recovery
    }
    pub fn spelling(&self) -> &str {
        &self.spelling
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFunctorGuard {
    id: SourceFunctorGuardId,
    ordinal: usize,
    formula: SourceAtomicFormulaId,
    site: TypedSiteRef,
    source_range: SourceRange,
    context: BindingContextId,
    recovery: SourceFunctorDefinitionRecovery,
    spelling: String,
}

impl SourceFunctorGuard {
    pub const fn id(&self) -> SourceFunctorGuardId {
        self.id
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
    pub const fn recovery(&self) -> SourceFunctorDefinitionRecovery {
        self.recovery
    }
    pub fn spelling(&self) -> &str {
        &self.spelling
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFunctorDefiniens {
    id: SourceFunctorDefiniensId,
    owner: SourceFunctorDefinitionId,
    ordinal: usize,
    target: SourceFunctorDefiniensTarget,
    site: TypedSiteRef,
    source_range: SourceRange,
    context: BindingContextId,
    recovery: SourceFunctorDefinitionRecovery,
    spelling: String,
}

impl SourceFunctorDefiniens {
    pub const fn id(&self) -> SourceFunctorDefiniensId {
        self.id
    }
    pub const fn owner(&self) -> SourceFunctorDefinitionId {
        self.owner
    }
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }
    pub const fn target(&self) -> SourceFunctorDefiniensTarget {
        self.target
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
    pub const fn recovery(&self) -> SourceFunctorDefinitionRecovery {
        self.recovery
    }
    pub fn spelling(&self) -> &str {
        &self.spelling
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFunctorCorrectness {
    id: SourceFunctorCorrectnessId,
    owner: SourceFunctorDefinitionId,
    ordinal: usize,
    kind: SourceFunctorCorrectnessKind,
    site: TypedSiteRef,
    source_range: SourceRange,
    justification: SourceAnchor,
    recovery: SourceFunctorDefinitionRecovery,
    spelling: String,
    obligation: InitialObligationId,
}

impl SourceFunctorCorrectness {
    pub const fn id(&self) -> SourceFunctorCorrectnessId {
        self.id
    }
    pub const fn owner(&self) -> SourceFunctorDefinitionId {
        self.owner
    }
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }
    pub const fn kind(&self) -> SourceFunctorCorrectnessKind {
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
    pub const fn recovery(&self) -> SourceFunctorDefinitionRecovery {
        self.recovery
    }
    pub fn spelling(&self) -> &str {
        &self.spelling
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
    SourceFunctorDefinitionTable,
    SourceFunctorDefinition,
    SourceFunctorDefinitionId
);
table!(
    SourceFunctorParameterTable,
    SourceFunctorParameter,
    SourceFunctorParameterId
);
table!(
    SourceFunctorGuardTable,
    SourceFunctorGuard,
    SourceFunctorGuardId
);
table!(
    SourceFunctorDefiniensTable,
    SourceFunctorDefiniens,
    SourceFunctorDefiniensId
);
table!(
    SourceFunctorCorrectnessTable,
    SourceFunctorCorrectness,
    SourceFunctorCorrectnessId
);

#[derive(Debug, Clone, PartialEq, Eq)]
struct SourceFunctorResolverIdentity {
    symbol: SymbolId,
    definition: DefinitionId,
    contribution: SourceContributionId,
    origin: SemanticOrigin,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFunctorDefinitionHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    resolver_identities: Vec<SourceFunctorResolverIdentity>,
    source_context_fingerprint: String,
    source_type_fingerprint: String,
    source_term_fingerprint: String,
    application_fingerprint: Option<String>,
    structure_fingerprint: Option<String>,
    set_term_fingerprint: Option<String>,
    atomic_formula_fingerprint: Option<String>,
    definitions: SourceFunctorDefinitionTable,
    parameters: SourceFunctorParameterTable,
    guards: SourceFunctorGuardTable,
    definientia: SourceFunctorDefiniensTable,
    correctness: SourceFunctorCorrectnessTable,
}

impl SourceFunctorDefinitionHandoff {
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
    pub fn application_fingerprint(&self) -> Option<&str> {
        self.application_fingerprint.as_deref()
    }
    pub fn structure_fingerprint(&self) -> Option<&str> {
        self.structure_fingerprint.as_deref()
    }
    pub fn set_term_fingerprint(&self) -> Option<&str> {
        self.set_term_fingerprint.as_deref()
    }
    pub fn atomic_formula_fingerprint(&self) -> Option<&str> {
        self.atomic_formula_fingerprint.as_deref()
    }
    pub const fn definitions(&self) -> &SourceFunctorDefinitionTable {
        &self.definitions
    }
    pub const fn parameters(&self) -> &SourceFunctorParameterTable {
        &self.parameters
    }
    pub const fn guards(&self) -> &SourceFunctorGuardTable {
        &self.guards
    }
    pub const fn definientia(&self) -> &SourceFunctorDefiniensTable {
        &self.definientia
    }
    pub const fn correctness(&self) -> &SourceFunctorCorrectnessTable {
        &self.correctness
    }

    pub fn debug_text(&self) -> String {
        let mut output = String::from("source-functor-definition-debug-v1\n");
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
        write_optional_fingerprint(
            &mut output,
            "application-fingerprint",
            self.application_fingerprint.as_deref(),
        );
        write_optional_fingerprint(
            &mut output,
            "structure-fingerprint",
            self.structure_fingerprint.as_deref(),
        );
        write_optional_fingerprint(
            &mut output,
            "set-term-fingerprint",
            self.set_term_fingerprint.as_deref(),
        );
        write_optional_fingerprint(
            &mut output,
            "atomic-formula-fingerprint",
            self.atomic_formula_fingerprint.as_deref(),
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
                " origin_path={:?} spelling={:?} style={} return_type={} definiens={}",
                row.origin.structural_path(),
                row.spelling,
                style_key(row.style),
                row.return_type.index(),
                row.definiens.index(),
            );
        }
        for (id, row) in self.parameters.iter() {
            let _ = write!(
                output,
                "parameter#{} ordinal={} binding={} written_type={} range={}..{} declaration_range={}..{} site=",
                id.index(),
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
                "guard#{} ordinal={} formula={} range={}..{} site=",
                id.index(),
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
        for (id, row) in self.definientia.iter() {
            let _ = write!(
                output,
                "definiens#{} owner={} ordinal={} target=",
                id.index(),
                row.owner.index(),
                row.ordinal,
            );
            write_target(&mut output, row.target);
            let _ = write!(
                output,
                " range={}..{} site=",
                row.source_range.start, row.source_range.end
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
        for (id, row) in self.correctness.iter() {
            let _ = write!(
                output,
                "correctness#{} owner={} ordinal={} kind={} range={}..{} site=",
                id.index(),
                row.owner.index(),
                row.ordinal,
                correctness_key(row.kind),
                row.source_range.start,
                row.source_range.end,
            );
            write_site(&mut output, &row.site);
            output.push_str(" justification=");
            write_anchor(&mut output, &row.justification);
            let _ = writeln!(
                output,
                " recovery={} spelling={:?} obligation={}",
                recovery_key(row.recovery),
                row.spelling,
                row.obligation.index(),
            );
        }
        output
    }

    // Rationale: installation revalidates the frozen lower-owner bundle without hiding dependencies.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn validate_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        source_context: &SourceBindingContextHandoff,
        source_type: &SourceTypeApplicationHandoff,
        source_term: &SourcePrimaryTermHandoff,
        applications: Option<&SourceFunctorApplicationHandoff>,
        structures: Option<&SourceStructureHandoff>,
        set_terms: Option<&SourceSetTermHandoff>,
        atomic_formulas: Option<&SourceAtomicFormulaHandoff>,
        initial_obligations: &InitialObligationTable,
        arena: &TypedArena,
    ) -> Result<(), SourceFunctorDefinitionError> {
        validate_dependency_identity(
            source_id,
            module_id,
            source_context,
            source_type,
            source_term,
            applications,
            structures,
            set_terms,
            atomic_formulas,
            arena,
        )?;
        if self.source_id != source_id || &self.module_id != module_id {
            return Err(SourceFunctorDefinitionError::SourceIdentityMismatch);
        }
        if self.source_context_fingerprint != source_context.debug_text()
            || self.source_type_fingerprint != source_type.debug_text()
            || self.source_term_fingerprint != source_term.debug_text()
            || !fingerprint_matches(
                &self.application_fingerprint,
                applications.map(|row| row.debug_text()),
            )
            || !fingerprint_matches(
                &self.structure_fingerprint,
                structures.map(|row| row.debug_text()),
            )
            || !fingerprint_matches(
                &self.set_term_fingerprint,
                set_terms.map(|row| row.debug_text()),
            )
            || !fingerprint_matches(
                &self.atomic_formula_fingerprint,
                atomic_formulas.map(|row| row.debug_text()),
            )
        {
            return Err(SourceFunctorDefinitionError::DependencyMismatch);
        }
        validate_handoff_rows(
            self,
            source_context,
            source_type,
            source_term,
            applications,
            structures,
            set_terms,
            atomic_formulas,
            initial_obligations,
            arena,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFunctorDefinitionProjection {
    base_initial_obligations: InitialObligationTable,
    handoff: SourceFunctorDefinitionHandoff,
    initial_obligations: InitialObligationTable,
}

impl SourceFunctorDefinitionProjection {
    pub const fn base_initial_obligations(&self) -> &InitialObligationTable {
        &self.base_initial_obligations
    }
    pub const fn handoff(&self) -> &SourceFunctorDefinitionHandoff {
        &self.handoff
    }
    pub const fn initial_obligations(&self) -> &InitialObligationTable {
        &self.initial_obligations
    }
    pub fn into_parts(
        self,
    ) -> (
        InitialObligationTable,
        SourceFunctorDefinitionHandoff,
        InitialObligationTable,
    ) {
        (
            self.base_initial_obligations,
            self.handoff,
            self.initial_obligations,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceFunctorDefinitionError {
    SourceIdentityMismatch,
    DependencyMismatch,
    InvalidResolverDefinition { index: usize },
    InvalidDefinition { index: usize },
    InvalidParameter { index: usize },
    InvalidGuard { index: usize },
    InvalidDefiniens { index: usize },
    InvalidCorrectness { index: usize },
    InvalidObligation,
    InvalidArenaOwnership,
    UnsupportedTaskShape,
}

impl fmt::Display for SourceFunctorDefinitionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SourceIdentityMismatch => {
                formatter.write_str("functor-definition source identity mismatch")
            }
            Self::DependencyMismatch => {
                formatter.write_str("functor-definition dependency mismatch")
            }
            Self::InvalidResolverDefinition { index } => {
                write!(formatter, "invalid functor resolver definition {index}")
            }
            Self::InvalidDefinition { index } => {
                write!(formatter, "invalid source functor definition {index}")
            }
            Self::InvalidParameter { index } => {
                write!(formatter, "invalid source functor parameter {index}")
            }
            Self::InvalidGuard { index } => {
                write!(formatter, "invalid source functor guard {index}")
            }
            Self::InvalidDefiniens { index } => {
                write!(formatter, "invalid source functor definiens {index}")
            }
            Self::InvalidCorrectness { index } => {
                write!(formatter, "invalid source functor correctness {index}")
            }
            Self::InvalidObligation => {
                formatter.write_str("invalid functor correctness obligation")
            }
            Self::InvalidArenaOwnership => {
                formatter.write_str("invalid functor-definition typed-arena ownership")
            }
            Self::UnsupportedTaskShape => {
                formatter.write_str("unsupported functor-definition task shape")
            }
        }
    }
}

impl Error for SourceFunctorDefinitionError {}

pub struct SourceFunctorDefinitionProducer;

impl SourceFunctorDefinitionProducer {
    // Rationale: the public producer ABI names every independently frozen lower handoff.
    #[allow(clippy::too_many_arguments)]
    pub fn build(
        input: SourceFunctorDefinitionHandoffInput,
        env: &SymbolEnv,
        source_context: &SourceBindingContextHandoff,
        source_type: &SourceTypeApplicationHandoff,
        source_term: &SourcePrimaryTermHandoff,
        applications: Option<&SourceFunctorApplicationHandoff>,
        structures: Option<&SourceStructureHandoff>,
        set_terms: Option<&SourceSetTermHandoff>,
        atomic_formulas: Option<&SourceAtomicFormulaHandoff>,
        base_initial_obligations: &InitialObligationTable,
        arena: &TypedArena,
    ) -> Result<SourceFunctorDefinitionProjection, SourceFunctorDefinitionError> {
        validate_dependency_identity(
            input.source_id,
            &input.module_id,
            source_context,
            source_type,
            source_term,
            applications,
            structures,
            set_terms,
            atomic_formulas,
            arena,
        )?;
        if env.module_id() != &input.module_id {
            return Err(SourceFunctorDefinitionError::SourceIdentityMismatch);
        }
        validate_input_shape(&input)?;
        validate_baseline(base_initial_obligations)?;
        let origins = validate_resolver_definitions(&input, env)?;
        validate_input_rows(
            &input,
            source_context,
            source_type,
            source_term,
            applications,
            structures,
            set_terms,
            atomic_formulas,
            arena,
        )?;

        let mut initial_obligations = base_initial_obligations.clone();
        let mut obligation_ids = Vec::with_capacity(2);
        for (index, row) in input.correctness.iter().enumerate() {
            let (kind, goal, provenance) = match row.kind {
                SourceFunctorCorrectnessKind::Existence => (
                    InitialObligationKind::FunctorExistence,
                    "source.definition.functor.correctness:definition=1:existence",
                    "source.definition.functor:definition=1:correctness=0",
                ),
                SourceFunctorCorrectnessKind::Uniqueness => (
                    InitialObligationKind::FunctorUniqueness,
                    "source.definition.functor.correctness:definition=1:uniqueness",
                    "source.definition.functor:definition=1:correctness=1",
                ),
            };
            let id = initial_obligations.insert(InitialObligationDraft {
                kind,
                owner: row.site.clone(),
                source_range: row.source_range,
                assumptions: Vec::new(),
                goal: InitialObligationGoal::new(goal),
                provenance: InitialObligationProvenance::new(provenance),
                status: InitialObligationStatus::Pending,
            });
            if id.index() != base_initial_obligations.len() + index {
                return Err(SourceFunctorDefinitionError::InvalidObligation);
            }
            obligation_ids.push(id);
        }

        let definitions = SourceFunctorDefinitionTable {
            rows: input
                .definitions
                .into_iter()
                .zip(origins)
                .enumerate()
                .map(|(index, (row, origin))| SourceFunctorDefinition {
                    id: SourceFunctorDefinitionId::new(index),
                    symbol: row.symbol,
                    definition: row.definition,
                    contribution: row.contribution,
                    site: row.site,
                    source_range: row.source_range,
                    source_ordinal: row.source_ordinal,
                    context: row.context,
                    recovery: row.recovery,
                    spelling: row.spelling,
                    style: row.style,
                    return_type: row.return_type,
                    definiens: row.definiens,
                    origin,
                })
                .collect(),
        };
        let parameters = SourceFunctorParameterTable {
            rows: input
                .parameters
                .into_iter()
                .enumerate()
                .map(|(index, row)| SourceFunctorParameter {
                    id: SourceFunctorParameterId::new(index),
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
        let guards = SourceFunctorGuardTable {
            rows: input
                .guards
                .into_iter()
                .enumerate()
                .map(|(index, row)| SourceFunctorGuard {
                    id: SourceFunctorGuardId::new(index),
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
        let definientia = SourceFunctorDefiniensTable {
            rows: input
                .definientia
                .into_iter()
                .enumerate()
                .map(|(index, row)| SourceFunctorDefiniens {
                    id: SourceFunctorDefiniensId::new(index),
                    owner: row.owner,
                    ordinal: row.ordinal,
                    target: row.target,
                    site: row.site,
                    source_range: row.source_range,
                    context: row.context,
                    recovery: row.recovery,
                    spelling: row.spelling,
                })
                .collect(),
        };
        let correctness = SourceFunctorCorrectnessTable {
            rows: input
                .correctness
                .into_iter()
                .zip(obligation_ids)
                .enumerate()
                .map(|(index, (row, obligation))| SourceFunctorCorrectness {
                    id: SourceFunctorCorrectnessId::new(index),
                    owner: row.owner,
                    ordinal: row.ordinal,
                    kind: row.kind,
                    site: row.site,
                    source_range: row.source_range,
                    justification: row.justification,
                    recovery: row.recovery,
                    spelling: row.spelling,
                    obligation,
                })
                .collect(),
        };
        let resolver_identities = definitions
            .rows
            .iter()
            .map(|row| SourceFunctorResolverIdentity {
                symbol: row.symbol.clone(),
                definition: row.definition,
                contribution: row.contribution,
                origin: row.origin.clone(),
            })
            .collect();
        let handoff = SourceFunctorDefinitionHandoff {
            source_id: input.source_id,
            module_id: input.module_id,
            resolver_identities,
            source_context_fingerprint: source_context.debug_text(),
            source_type_fingerprint: source_type.debug_text(),
            source_term_fingerprint: source_term.debug_text(),
            application_fingerprint: applications.map(|row| row.debug_text()),
            structure_fingerprint: structures.map(|row| row.debug_text()),
            set_term_fingerprint: set_terms.map(|row| row.debug_text()),
            atomic_formula_fingerprint: atomic_formulas.map(|row| row.debug_text()),
            definitions,
            parameters,
            guards,
            definientia,
            correctness,
        };
        validate_handoff_rows(
            &handoff,
            source_context,
            source_type,
            source_term,
            applications,
            structures,
            set_terms,
            atomic_formulas,
            &initial_obligations,
            arena,
        )?;
        Ok(SourceFunctorDefinitionProjection {
            base_initial_obligations: base_initial_obligations.clone(),
            handoff,
            initial_obligations,
        })
    }
}

// Rationale: dependency identity is checked across the complete frozen lower-owner bundle.
#[allow(clippy::too_many_arguments)]
fn validate_dependency_identity(
    source_id: SourceId,
    module_id: &ModuleId,
    source_context: &SourceBindingContextHandoff,
    source_type: &SourceTypeApplicationHandoff,
    source_term: &SourcePrimaryTermHandoff,
    applications: Option<&SourceFunctorApplicationHandoff>,
    structures: Option<&SourceStructureHandoff>,
    set_terms: Option<&SourceSetTermHandoff>,
    atomic_formulas: Option<&SourceAtomicFormulaHandoff>,
    arena: &TypedArena,
) -> Result<(), SourceFunctorDefinitionError> {
    if source_context.source_id() != source_id
        || source_context.module_id() != module_id
        || source_type.source_id() != source_id
        || source_type.module_id() != module_id
        || source_term.source_id() != source_id
        || source_term.module_id() != module_id
        || applications
            .is_some_and(|row| row.source_id() != source_id || row.module_id() != module_id)
        || structures
            .is_some_and(|row| row.source_id() != source_id || row.module_id() != module_id)
        || set_terms.is_some_and(|row| row.source_id() != source_id || row.module_id() != module_id)
        || atomic_formulas
            .is_some_and(|row| row.source_id() != source_id || row.module_id() != module_id)
    {
        return Err(SourceFunctorDefinitionError::SourceIdentityMismatch);
    }
    source_type
        .validate_installation(source_id, module_id, arena)
        .map_err(|_| SourceFunctorDefinitionError::DependencyMismatch)?;
    source_term
        .validate_installation(source_id, module_id, arena)
        .map_err(|_| SourceFunctorDefinitionError::DependencyMismatch)?;
    if let Some(row) = applications {
        row.validate_installation(source_id, module_id, source_term)
            .map_err(|_| SourceFunctorDefinitionError::DependencyMismatch)?;
    }
    if let Some(row) = structures {
        row.validate_installation(source_id, module_id, source_term, applications, arena)
            .map_err(|_| SourceFunctorDefinitionError::DependencyMismatch)?;
    }
    if let Some(row) = set_terms {
        row.validate_installation(
            source_id,
            module_id,
            source_term,
            applications,
            structures,
            arena,
        )
        .map_err(|_| SourceFunctorDefinitionError::DependencyMismatch)?;
    }
    let Some(atomic_formulas) = atomic_formulas else {
        return Err(SourceFunctorDefinitionError::DependencyMismatch);
    };
    atomic_formulas
        .validate_installation(
            source_id,
            module_id,
            source_term,
            applications,
            structures,
            set_terms,
            arena,
        )
        .map_err(|_| SourceFunctorDefinitionError::DependencyMismatch)?;
    Ok(())
}

fn validate_input_shape(
    input: &SourceFunctorDefinitionHandoffInput,
) -> Result<(), SourceFunctorDefinitionError> {
    if input.definitions.len() != 2
        || input.parameters.len() != 2
        || input.guards.len() != 1
        || input.definientia.len() != 2
        || input.correctness.len() != 2
    {
        return Err(SourceFunctorDefinitionError::UnsupportedTaskShape);
    }
    Ok(())
}

fn validate_baseline(
    baseline: &InitialObligationTable,
) -> Result<(), SourceFunctorDefinitionError> {
    if baseline.iter().any(|(_, row)| {
        matches!(
            row.kind,
            InitialObligationKind::PredicatePropertyCorrectness
                | InitialObligationKind::FunctorExistence
                | InitialObligationKind::FunctorUniqueness
        )
    }) {
        return Err(SourceFunctorDefinitionError::InvalidObligation);
    }
    Ok(())
}

fn validate_resolver_definitions(
    input: &SourceFunctorDefinitionHandoffInput,
    env: &SymbolEnv,
) -> Result<Vec<SemanticOrigin>, SourceFunctorDefinitionError> {
    if env.symbols().len() != 2 || env.definitions().len() != 2 || env.contributions().len() != 1 {
        return Err(SourceFunctorDefinitionError::InvalidResolverDefinition { index: 0 });
    }
    let expected_notations = ["task260_equals ( x )", "task260_means ( y )"];
    let expected_paths: [&[u32]; 2] = [&[4, 0, 9, 0], &[4, 0, 9, 1]];
    let expected_ranges = [(61, 118), (121, 179)];
    let expected_anchor = SourceAnchor::Range(range(input.source_id, 0, 261));
    let mut origins = Vec::with_capacity(2);
    for (index, row) in input.definitions.iter().enumerate() {
        let symbol = env
            .symbols()
            .get(&row.symbol)
            .ok_or(SourceFunctorDefinitionError::InvalidResolverDefinition { index })?;
        let definition = env
            .definitions()
            .get(row.definition)
            .ok_or(SourceFunctorDefinitionError::InvalidResolverDefinition { index })?;
        let contribution = env
            .contributions()
            .get(row.contribution)
            .ok_or(SourceFunctorDefinitionError::InvalidResolverDefinition { index })?;
        if row.definition.index() != index
            || row.symbol.module() != &input.module_id
            || symbol.symbol() != &row.symbol
            || symbol.kind() != SymbolKind::Functor
            || symbol.visibility() != Visibility::Public
            || symbol.export_status() != ExportStatus::Exported
            || symbol.primary_spelling() != expected_notations[index]
            || symbol.notation_spelling() != Some(expected_notations[index])
            || symbol.contribution() != row.contribution
            || symbol.origin() != definition.origin()
            || symbol.signature() != definition.signature()
            || !symbol.relations().is_empty()
            || definition.id() != row.definition
            || definition.symbol() != &row.symbol
            || definition.kind() != DefinitionKind::Functor
            || definition.visibility() != Visibility::Public
            || !definition.parameters().is_empty()
            || !definition.binders().is_empty()
            || definition.arity().is_some()
            || definition.notation_shape() != Some(expected_notations[index])
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
            || contribution.effects().symbols().len() != 2
            || contribution.effects().definitions().len() != 2
            || !contribution.effects().symbols().contains(&row.symbol)
            || !contribution
                .effects()
                .definitions()
                .contains(&row.definition)
            || !normal_origin(
                definition.origin(),
                input.source_id,
                &input.module_id,
                range(
                    input.source_id,
                    expected_ranges[index].0,
                    expected_ranges[index].1,
                ),
                expected_paths[index],
            )
        {
            return Err(SourceFunctorDefinitionError::InvalidResolverDefinition { index });
        }
        origins.push(definition.origin().clone());
    }
    Ok(origins)
}

// Rationale: row validation keeps each optional lower family explicit and independently testable.
#[allow(clippy::too_many_arguments)]
fn validate_input_rows(
    input: &SourceFunctorDefinitionHandoffInput,
    source_context: &SourceBindingContextHandoff,
    source_type: &SourceTypeApplicationHandoff,
    source_term: &SourcePrimaryTermHandoff,
    applications: Option<&SourceFunctorApplicationHandoff>,
    structures: Option<&SourceStructureHandoff>,
    set_terms: Option<&SourceSetTermHandoff>,
    atomic_formulas: Option<&SourceAtomicFormulaHandoff>,
    arena: &TypedArena,
) -> Result<(), SourceFunctorDefinitionError> {
    let Some(atomic_formulas) = atomic_formulas else {
        return Err(SourceFunctorDefinitionError::DependencyMismatch);
    };
    validate_optional_definiens_targets(input, applications, structures, set_terms, arena)?;
    if applications.is_some() || structures.is_some() || set_terms.is_some() {
        return Err(SourceFunctorDefinitionError::DependencyMismatch);
    }
    let source = input.source_id;
    let context = BindingContextId::new(1);
    let definition_ranges = [(61, 118), (121, 179)];
    let definition_sites = [84, 95];
    let definition_spellings = [
        "func Task260EqualsDef: task260_equals(x) -> set equals x;",
        "func Task260MeansDef: task260_means(y) -> set means x = y;",
    ];
    let styles = [
        SourceFunctorDefinitionStyle::Equals,
        SourceFunctorDefinitionStyle::Means,
    ];
    for (index, row) in input.definitions.iter().enumerate() {
        if row.source_range
            != range(
                source,
                definition_ranges[index].0,
                definition_ranges[index].1,
            )
            || row.source_ordinal != index
            || row.context != context
            || row.recovery != SourceFunctorDefinitionRecovery::Normal
            || row.spelling != definition_spellings[index]
            || row.style != styles[index]
            || row.return_type != SourceTypeDefinitionReturnId::new(index)
            || row.definiens != SourceFunctorDefiniensId::new(index)
            || row.site != TypedSiteRef::Node(TypedNodeId::new(definition_sites[index]))
        {
            return Err(SourceFunctorDefinitionError::InvalidDefinition { index });
        }
        if !valid_site(
            &row.site,
            row.source_range,
            "source.definition.functor",
            LocalTypeContextId::new(1),
            arena,
        ) {
            return Err(SourceFunctorDefinitionError::InvalidArenaOwnership);
        }
    }

    let local_context = validate_context_profile(input, source_context, arena)?;
    let parameter_ranges = [(13, 26, 17, 18), (29, 42, 33, 34)];
    let parameter_sites = [65, 69];
    let parameter_spellings = ["let x be set;", "let y be set;"];
    for (index, row) in input.parameters.iter().enumerate() {
        let (start, end, declaration_start, declaration_end) = parameter_ranges[index];
        if row.ordinal != index
            || row.binding != BindingId::new(index)
            || row.written_type != SourceTypeApplicationId::new(index)
            || row.site != TypedSiteRef::Node(TypedNodeId::new(parameter_sites[index]))
            || row.source_range != range(source, start, end)
            || row.declaration_range != range(source, declaration_start, declaration_end)
            || row.context != context
            || row.recovery != SourceFunctorDefinitionRecovery::Normal
            || row.spelling != parameter_spellings[index]
        {
            return Err(SourceFunctorDefinitionError::InvalidParameter { index });
        }
        if !valid_site(
            &row.site,
            row.declaration_range,
            "source.definition.functor.parameter",
            local_context,
            arena,
        ) {
            return Err(SourceFunctorDefinitionError::InvalidArenaOwnership);
        }
    }
    validate_type_profile(input, source_type, local_context, arena)?;
    validate_term_and_atomic_profile(input, source_term, atomic_formulas, local_context, arena)?;

    let guard = &input.guards[0];
    if guard.ordinal != 0
        || guard.formula != SourceAtomicFormulaId::new(0)
        || guard.site != TypedSiteRef::Node(TypedNodeId::new(77))
        || guard.source_range != range(source, 45, 58)
        || guard.context != context
        || guard.recovery != SourceFunctorDefinitionRecovery::Normal
        || guard.spelling != "assume x = x;"
    {
        return Err(SourceFunctorDefinitionError::InvalidGuard { index: 0 });
    }
    if !valid_site(
        &guard.site,
        guard.source_range,
        "source.definition.functor.guard",
        local_context,
        arena,
    ) {
        return Err(SourceFunctorDefinitionError::InvalidArenaOwnership);
    }

    let expected_targets = [
        SourceFunctorDefiniensTarget::Primary(SourcePrimaryTermId::new(2)),
        SourceFunctorDefiniensTarget::AtomicFormula(SourceAtomicFormulaId::new(1)),
    ];
    let definiens_ranges = [(116, 117), (173, 178)];
    let definiens_sites = [83, 94];
    let definiens_spellings = ["x", "x = y"];
    for (index, row) in input.definientia.iter().enumerate() {
        if row.owner != SourceFunctorDefinitionId::new(index)
            || row.ordinal != index
            || row.target != expected_targets[index]
            || row.site != TypedSiteRef::Node(TypedNodeId::new(definiens_sites[index]))
            || row.source_range
                != range(source, definiens_ranges[index].0, definiens_ranges[index].1)
            || row.context != context
            || row.recovery != SourceFunctorDefinitionRecovery::Normal
            || row.spelling != definiens_spellings[index]
        {
            return Err(SourceFunctorDefinitionError::InvalidDefiniens { index });
        }
        if !valid_site(
            &row.site,
            row.source_range,
            "source.definition.functor.definiens",
            local_context,
            arena,
        ) {
            return Err(SourceFunctorDefinitionError::InvalidArenaOwnership);
        }
    }

    let correctness_ranges = [(182, 217, 192, 216), (220, 256, 231, 255)];
    let correctness_sites = [99, 103];
    let correctness_kinds = [
        SourceFunctorCorrectnessKind::Existence,
        SourceFunctorCorrectnessKind::Uniqueness,
    ];
    let correctness_spellings = [
        "existence by computation(steps: 1);",
        "uniqueness by computation(steps: 1);",
    ];
    for (index, row) in input.correctness.iter().enumerate() {
        let (start, end, justification_start, justification_end) = correctness_ranges[index];
        if row.owner != SourceFunctorDefinitionId::new(1)
            || row.ordinal != index
            || row.kind != correctness_kinds[index]
            || row.site != TypedSiteRef::Node(TypedNodeId::new(correctness_sites[index]))
            || row.source_range != range(source, start, end)
            || row.justification
                != SourceAnchor::Range(range(source, justification_start, justification_end))
            || row.recovery != SourceFunctorDefinitionRecovery::Normal
            || row.spelling != correctness_spellings[index]
            || input.definitions[1].source_range.end >= row.source_range.start
            || (index == 1
                && input.correctness[0].source_range.end >= input.correctness[1].source_range.start)
        {
            return Err(SourceFunctorDefinitionError::InvalidCorrectness { index });
        }
        if !valid_site(
            &row.site,
            row.source_range,
            "source.definition.functor.correctness",
            local_context,
            arena,
        ) {
            return Err(SourceFunctorDefinitionError::InvalidArenaOwnership);
        }
    }
    Ok(())
}

// Rationale: deferred optional lower families are authenticated before Task 260 rejects their
// semantics, so stale IDs and arena owners cannot hide behind the frozen profile boundary.
fn validate_optional_definiens_targets(
    input: &SourceFunctorDefinitionHandoffInput,
    applications: Option<&SourceFunctorApplicationHandoff>,
    structures: Option<&SourceStructureHandoff>,
    set_terms: Option<&SourceSetTermHandoff>,
    arena: &TypedArena,
) -> Result<(), SourceFunctorDefinitionError> {
    for (index, definiens) in input.definientia.iter().enumerate() {
        let optional_owner = match definiens.target {
            SourceFunctorDefiniensTarget::Application(id) => {
                let row = applications
                    .and_then(|handoff| handoff.applications().get(id))
                    .ok_or(SourceFunctorDefinitionError::InvalidDefiniens { index })?;
                let kind = match row.kind() {
                    crate::source_application::SourceFunctorApplicationKind::Symbolic => {
                        "source.term.functor-application.symbolic"
                    }
                    crate::source_application::SourceFunctorApplicationKind::Inline => {
                        "source.term.functor-application.inline"
                    }
                };
                Some((row.site(), row.source_range(), row.context(), kind))
            }
            SourceFunctorDefiniensTarget::Structure(id) => {
                let row = structures
                    .and_then(|handoff| handoff.terms().get(id))
                    .ok_or(SourceFunctorDefinitionError::InvalidDefiniens { index })?;
                let kind = match row.kind() {
                    crate::source_structure::SourceStructureTermKind::Constructor => {
                        "source.term.structure.constructor"
                    }
                    crate::source_structure::SourceStructureTermKind::SelectorAccess => {
                        "source.term.structure.selector"
                    }
                    crate::source_structure::SourceStructureTermKind::FunctionalUpdate => {
                        "source.term.structure.update"
                    }
                };
                Some((row.site(), row.source_range(), row.context(), kind))
            }
            SourceFunctorDefiniensTarget::SetTerm(id) => {
                let row = set_terms
                    .and_then(|handoff| handoff.terms().get(id))
                    .ok_or(SourceFunctorDefinitionError::InvalidDefiniens { index })?;
                let kind = match row.kind() {
                    crate::source_set_term::SourceSetTermKind::Enumeration => {
                        "source.term.set.enumeration"
                    }
                    crate::source_set_term::SourceSetTermKind::Comprehension => {
                        "source.term.set.comprehension"
                    }
                    crate::source_set_term::SourceSetTermKind::Choice => "source.term.set.choice",
                    crate::source_set_term::SourceSetTermKind::Qua => "source.term.set.qua",
                };
                Some((row.site(), row.source_range(), row.context(), kind))
            }
            SourceFunctorDefiniensTarget::Primary(_)
            | SourceFunctorDefiniensTarget::AtomicFormula(_) => None,
        };
        if let Some((site, source_range, context, kind)) = optional_owner {
            if context != definiens.context {
                return Err(SourceFunctorDefinitionError::InvalidDefiniens { index });
            }
            if !valid_site(
                site,
                source_range,
                kind,
                LocalTypeContextId::new(context.index()),
                arena,
            ) {
                return Err(SourceFunctorDefinitionError::InvalidArenaOwnership);
            }
        }
    }
    Ok(())
}

fn validate_context_profile(
    input: &SourceFunctorDefinitionHandoffInput,
    source_context: &SourceBindingContextHandoff,
    arena: &TypedArena,
) -> Result<LocalTypeContextId, SourceFunctorDefinitionError> {
    let source = input.source_id;
    let binding_env = source_context.binding_env();
    if source_context.items().len() != 1
        || source_context.declarations().len() != 2
        || source_context.context_links().len() != 2
        || source_context.local_contexts().len() != 2
        || binding_env.contexts().len() != 2
        || binding_env.bindings().len() != 2
        || !binding_env.diagnostics().is_empty()
    {
        return Err(SourceFunctorDefinitionError::DependencyMismatch);
    }
    let item = source_context
        .items()
        .get(crate::source_context::SourceItemId::new(0))
        .ok_or(SourceFunctorDefinitionError::DependencyMismatch)?;
    let module_context = BindingContextId::new(0);
    let definition_context = BindingContextId::new(1);
    let module_local = LocalTypeContextId::new(0);
    let definition_local = LocalTypeContextId::new(1);
    let module_link = source_context
        .context_links()
        .get(module_context)
        .ok_or(SourceFunctorDefinitionError::DependencyMismatch)?;
    let definition_link = source_context
        .context_links()
        .get(definition_context)
        .ok_or(SourceFunctorDefinitionError::DependencyMismatch)?;
    if item.id != crate::source_context::SourceItemId::new(0)
        || item.shell.index() != 0
        || item.shell_ordinal != 0
        || item.role != SourceItemRole::DefinitionBlock
        || item.source_range != range(source, 0, 261)
        || item.parent.is_some()
        || item.visibility != SourceItemVisibility::Unspecified
        || item.site != TypedSiteRef::Node(TypedNodeId::new(104))
        || item.local_scope.is_none()
        || item.recovery != SourceItemRecovery::Normal
        || item.binding_context != definition_context
        || item.local_context != definition_local
        || item.predecessor.is_some()
        || module_link.binding_context != module_context
        || module_link.local_context != module_local
        || module_link.item.is_some()
        || definition_link.binding_context != definition_context
        || definition_link.local_context != definition_local
        || definition_link.item != Some(item.id)
    {
        return Err(SourceFunctorDefinitionError::DependencyMismatch);
    }
    if !valid_site(
        &item.site,
        item.source_range,
        "source.definition",
        definition_local,
        arena,
    ) {
        return Err(SourceFunctorDefinitionError::InvalidArenaOwnership);
    }
    for (index, row) in input.parameters.iter().enumerate() {
        let declaration = source_context
            .declarations()
            .get(crate::source_context::SourceDeclarationId::new(index))
            .ok_or(SourceFunctorDefinitionError::InvalidParameter { index })?;
        let written_range = if index == 0 {
            range(source, 22, 25)
        } else {
            range(source, 38, 41)
        };
        if declaration.item != item.id
            || declaration.binding != row.binding
            || declaration.source_ordinal != index
            || declaration.spelling != if index == 0 { "x" } else { "y" }
            || declaration.declaration_range != row.declaration_range
            || declaration.written_type_range != written_range
            || declaration.site != row.site
            || !matches!(
                declaration.role,
                SourceBindingSiteRole::DefinitionParameter { .. }
            )
            || declaration.binding_context != definition_context
            || declaration.local_context != definition_local
            || declaration.shadowed_binding.is_some()
            || declaration.predecessor.map(|id| id.index()) != index.checked_sub(1)
        {
            return Err(SourceFunctorDefinitionError::InvalidParameter { index });
        }
        let binding = binding_env
            .bindings()
            .get(row.binding)
            .ok_or(SourceFunctorDefinitionError::InvalidParameter { index })?;
        if binding.id != row.binding
            || binding.spelling != if index == 0 { "x" } else { "y" }
            || binding.kind != BindingKind::DefinitionParameter
            || binding.owner_context != definition_context
            || binding.declaration_range != row.declaration_range
            || binding.visible_after_ordinal != index
            || binding.type_site != BindingTypeSite::Source(written_range)
            || binding.status != BindingStatus::Active
            || !binding.captured.identities().is_empty()
            || !binding.diagnostics.is_empty()
            || binding.recovery != BindingRecoveryState::Normal
        {
            return Err(SourceFunctorDefinitionError::InvalidParameter { index });
        }
    }
    Ok(definition_local)
}

fn validate_type_profile(
    input: &SourceFunctorDefinitionHandoffInput,
    source_type: &SourceTypeApplicationHandoff,
    local_context: LocalTypeContextId,
    arena: &TypedArena,
) -> Result<(), SourceFunctorDefinitionError> {
    if source_type.applications().len() != 2
        || source_type.expressions().len() != 4
        || !source_type.arguments().is_empty()
        || source_type.definition_returns().len() != 2
    {
        return Err(SourceFunctorDefinitionError::DependencyMismatch);
    }
    let source = input.source_id;
    let ranges = [(22, 25), (38, 41), (105, 108), (163, 166)];
    let expression_sites = [63, 67, 80, 87];
    let head_sites = [62, 66, 79, 86];
    for index in 0..4 {
        let expression = source_type
            .expressions()
            .get(SourceTypeExpressionId::new(index))
            .ok_or(SourceFunctorDefinitionError::DependencyMismatch)?;
        let written_range = range(source, ranges[index].0, ranges[index].1);
        if expression.id() != SourceTypeExpressionId::new(index)
            || expression.source_id() != source
            || expression.module_id() != &input.module_id
            || expression.site() != &TypedSiteRef::Node(TypedNodeId::new(expression_sites[index]))
            || expression.source_range() != written_range
            || expression.spelling() != "set"
            || expression.head_site() != &TypedSiteRef::Node(TypedNodeId::new(head_sites[index]))
            || expression.head_range() != written_range
            || expression.head_spelling() != "set"
            || expression.form() != SourceTypeApplicationForm::Bare
            || expression.head() != &SourceTypeHead::BuiltinSet
            || expression.recovery() != NodeRecoveryState::Normal
        {
            return Err(SourceFunctorDefinitionError::DependencyMismatch);
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
            return Err(SourceFunctorDefinitionError::InvalidArenaOwnership);
        }
    }
    for index in 0..2 {
        let application = source_type
            .applications()
            .get(SourceTypeApplicationId::new(index))
            .ok_or(SourceFunctorDefinitionError::InvalidParameter { index })?;
        if application.id() != SourceTypeApplicationId::new(index)
            || application.binding() != input.parameters[index].binding
            || application.source_ordinal() != index
            || application.root() != SourceTypeExpressionId::new(index)
        {
            return Err(SourceFunctorDefinitionError::InvalidParameter { index });
        }
        let definition_return = source_type
            .definition_returns()
            .get(SourceTypeDefinitionReturnId::new(index))
            .ok_or(SourceFunctorDefinitionError::InvalidDefinition { index })?;
        if definition_return.id() != SourceTypeDefinitionReturnId::new(index)
            || definition_return.definition_site() != &input.definitions[index].site
            || definition_return.definition_range() != input.definitions[index].source_range
            || definition_return.source_ordinal() != index
            || definition_return.root() != SourceTypeExpressionId::new(index + 2)
        {
            return Err(SourceFunctorDefinitionError::InvalidDefinition { index });
        }
    }
    Ok(())
}

fn validate_term_and_atomic_profile(
    input: &SourceFunctorDefinitionHandoffInput,
    source_term: &SourcePrimaryTermHandoff,
    atomic_formulas: &SourceAtomicFormulaHandoff,
    local_context: LocalTypeContextId,
    arena: &TypedArena,
) -> Result<(), SourceFunctorDefinitionError> {
    if source_term.terms().len() != 5
        || source_term.references().len() != 5
        || !source_term.numeric_type_requests().is_empty()
        || atomic_formulas.formulas().len() != 2
        || !atomic_formulas.wrappers().is_empty()
        || !atomic_formulas.predicate_segments().is_empty()
        || !atomic_formulas.predicate_heads().is_empty()
        || !atomic_formulas.candidates().is_empty()
        || !atomic_formulas.type_sites().is_empty()
        || !atomic_formulas.attributes().is_empty()
        || atomic_formulas.edges().len() != 4
        || atomic_formulas.requests().len() != 4
    {
        return Err(SourceFunctorDefinitionError::DependencyMismatch);
    }
    let source = input.source_id;
    let context = BindingContextId::new(1);
    let term_ranges = [(52, 53), (56, 57), (116, 117), (173, 174), (177, 178)];
    let term_sites = [70, 72, 81, 88, 90];
    let term_spellings = ["x", "x", "x", "x", "y"];
    let binding_indexes = [0, 0, 0, 0, 1];
    for index in 0..5 {
        let term = source_term
            .terms()
            .get(SourcePrimaryTermId::new(index))
            .ok_or(SourceFunctorDefinitionError::DependencyMismatch)?;
        let reference = source_term
            .references()
            .get(SourcePrimaryTermReferenceId::new(index))
            .ok_or(SourceFunctorDefinitionError::DependencyMismatch)?;
        if term.site() != &TypedSiteRef::Node(TypedNodeId::new(term_sites[index]))
            || term.source_range() != range(source, term_ranges[index].0, term_ranges[index].1)
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
            return Err(SourceFunctorDefinitionError::DependencyMismatch);
        }
        if !valid_site(
            term.site(),
            term.source_range(),
            "source.term.variable-reference",
            local_context,
            arena,
        ) {
            return Err(SourceFunctorDefinitionError::InvalidArenaOwnership);
        }
    }
    let formula_ranges = [(52, 57), (173, 178)];
    let formula_sites = [75, 93];
    let formula_spellings = ["x = x", "x = y"];
    let target_indexes = [[0, 1], [3, 4]];
    for formula_index in 0..2 {
        let formula = atomic_formulas
            .formulas()
            .get(SourceAtomicFormulaId::new(formula_index))
            .ok_or(SourceFunctorDefinitionError::DependencyMismatch)?;
        if formula.site() != &TypedSiteRef::Node(TypedNodeId::new(formula_sites[formula_index]))
            || formula.source_range()
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
            return Err(SourceFunctorDefinitionError::DependencyMismatch);
        }
        if !valid_site(
            formula.site(),
            formula.source_range(),
            "source.formula.atomic.equality",
            local_context,
            arena,
        ) {
            return Err(SourceFunctorDefinitionError::InvalidArenaOwnership);
        }
        for (operand, target_index) in target_indexes[formula_index].iter().copied().enumerate() {
            let flat_index = formula_index * 2 + operand;
            let edge = atomic_formulas
                .edges()
                .get(SourceAtomicEdgeId::new(flat_index))
                .ok_or(SourceFunctorDefinitionError::DependencyMismatch)?;
            let request = atomic_formulas
                .requests()
                .get(crate::source_atomic_formula::SourceAtomicRequestId::new(
                    flat_index,
                ))
                .ok_or(SourceFunctorDefinitionError::DependencyMismatch)?;
            let role = if operand == 0 {
                SourceAtomicEdgeRole::BuiltinLeftOperand
            } else {
                SourceAtomicEdgeRole::BuiltinRightOperand
            };
            if edge.formula() != SourceAtomicFormulaId::new(formula_index)
                || edge.ordinal() != operand
                || edge.role() != role
                || edge.target()
                    != SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(target_index))
                || request.formula() != SourceAtomicFormulaId::new(formula_index)
                || request.ordinal() != operand
                || request.kind() != SourceAtomicRequestKind::OperandExpectedType
                || request.edge() != Some(SourceAtomicEdgeId::new(flat_index))
                || request.candidate().is_some()
                || request.type_site().is_some()
                || request.attribute().is_some()
            {
                return Err(SourceFunctorDefinitionError::DependencyMismatch);
            }
        }
    }
    Ok(())
}

// Rationale: replay validation authenticates the same complete lower bundle as construction.
#[allow(clippy::too_many_arguments)]
fn validate_handoff_rows(
    handoff: &SourceFunctorDefinitionHandoff,
    source_context: &SourceBindingContextHandoff,
    source_type: &SourceTypeApplicationHandoff,
    source_term: &SourcePrimaryTermHandoff,
    applications: Option<&SourceFunctorApplicationHandoff>,
    structures: Option<&SourceStructureHandoff>,
    set_terms: Option<&SourceSetTermHandoff>,
    atomic_formulas: Option<&SourceAtomicFormulaHandoff>,
    initial_obligations: &InitialObligationTable,
    arena: &TypedArena,
) -> Result<(), SourceFunctorDefinitionError> {
    validate_handoff_dense_ids(handoff)?;
    validate_handoff_resolver_identities(handoff)?;
    let input = SourceFunctorDefinitionHandoffInput {
        source_id: handoff.source_id,
        module_id: handoff.module_id.clone(),
        definitions: handoff
            .definitions
            .iter()
            .map(|(_, row)| SourceFunctorDefinitionInput {
                symbol: row.symbol.clone(),
                definition: row.definition,
                contribution: row.contribution,
                site: row.site.clone(),
                source_range: row.source_range,
                source_ordinal: row.source_ordinal,
                context: row.context,
                recovery: row.recovery,
                spelling: row.spelling.clone(),
                style: row.style,
                return_type: row.return_type,
                definiens: row.definiens,
            })
            .collect(),
        parameters: handoff
            .parameters
            .iter()
            .map(|(_, row)| SourceFunctorParameterInput {
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
            .map(|(_, row)| SourceFunctorGuardInput {
                ordinal: row.ordinal,
                formula: row.formula,
                site: row.site.clone(),
                source_range: row.source_range,
                context: row.context,
                recovery: row.recovery,
                spelling: row.spelling.clone(),
            })
            .collect(),
        definientia: handoff
            .definientia
            .iter()
            .map(|(_, row)| SourceFunctorDefiniensInput {
                owner: row.owner,
                ordinal: row.ordinal,
                target: row.target,
                site: row.site.clone(),
                source_range: row.source_range,
                context: row.context,
                recovery: row.recovery,
                spelling: row.spelling.clone(),
            })
            .collect(),
        correctness: handoff
            .correctness
            .iter()
            .map(|(_, row)| SourceFunctorCorrectnessInput {
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
    };
    validate_input_shape(&input)?;
    validate_input_rows(
        &input,
        source_context,
        source_type,
        source_term,
        applications,
        structures,
        set_terms,
        atomic_formulas,
        arena,
    )?;
    for (index, definition) in handoff.definitions.rows.iter().enumerate() {
        let paths: [&[u32]; 2] = [&[4, 0, 9, 0], &[4, 0, 9, 1]];
        if !normal_origin(
            &definition.origin,
            handoff.source_id,
            &handoff.module_id,
            definition.source_range,
            paths[index],
        ) {
            return Err(SourceFunctorDefinitionError::InvalidDefinition { index });
        }
    }
    validate_obligations(handoff, initial_obligations)
}

fn validate_handoff_dense_ids(
    handoff: &SourceFunctorDefinitionHandoff,
) -> Result<(), SourceFunctorDefinitionError> {
    for (index, row) in handoff.definitions.rows.iter().enumerate() {
        if row.id != SourceFunctorDefinitionId::new(index) {
            return Err(SourceFunctorDefinitionError::InvalidDefinition { index });
        }
    }
    for (index, row) in handoff.parameters.rows.iter().enumerate() {
        if row.id != SourceFunctorParameterId::new(index) {
            return Err(SourceFunctorDefinitionError::InvalidParameter { index });
        }
    }
    for (index, row) in handoff.guards.rows.iter().enumerate() {
        if row.id != SourceFunctorGuardId::new(index) {
            return Err(SourceFunctorDefinitionError::InvalidGuard { index });
        }
    }
    for (index, row) in handoff.definientia.rows.iter().enumerate() {
        if row.id != SourceFunctorDefiniensId::new(index) {
            return Err(SourceFunctorDefinitionError::InvalidDefiniens { index });
        }
    }
    for (index, row) in handoff.correctness.rows.iter().enumerate() {
        if row.id != SourceFunctorCorrectnessId::new(index) {
            return Err(SourceFunctorDefinitionError::InvalidCorrectness { index });
        }
    }
    Ok(())
}

fn validate_handoff_resolver_identities(
    handoff: &SourceFunctorDefinitionHandoff,
) -> Result<(), SourceFunctorDefinitionError> {
    if handoff.resolver_identities.len() != 2 {
        return Err(SourceFunctorDefinitionError::UnsupportedTaskShape);
    }
    for (index, (definition, identity)) in handoff
        .definitions
        .rows
        .iter()
        .zip(&handoff.resolver_identities)
        .enumerate()
    {
        let expected_fqn = format!(
            "{}::{}::{}",
            handoff.module_id.package().as_str(),
            handoff.module_id.path().as_str(),
            definition.symbol.local().as_str(),
        );
        if definition.symbol != identity.symbol
            || definition.definition != identity.definition
            || definition.contribution != identity.contribution
            || definition.origin != identity.origin
            || definition.symbol.module() != &handoff.module_id
            || definition.symbol.fqn().as_str() != expected_fqn
            || definition.definition.index() != index
            || definition.contribution.index() != 0
        {
            return Err(SourceFunctorDefinitionError::InvalidResolverDefinition { index });
        }
    }
    Ok(())
}

fn validate_obligations(
    handoff: &SourceFunctorDefinitionHandoff,
    initial_obligations: &InitialObligationTable,
) -> Result<(), SourceFunctorDefinitionError> {
    let base_len = handoff.correctness.rows[0].obligation.index();
    if initial_obligations.len() != base_len + 2 {
        return Err(SourceFunctorDefinitionError::InvalidObligation);
    }
    validate_baseline_prefix(initial_obligations, base_len)?;
    let kinds = [
        InitialObligationKind::FunctorExistence,
        InitialObligationKind::FunctorUniqueness,
    ];
    let goals = [
        "source.definition.functor.correctness:definition=1:existence",
        "source.definition.functor.correctness:definition=1:uniqueness",
    ];
    let provenances = [
        "source.definition.functor:definition=1:correctness=0",
        "source.definition.functor:definition=1:correctness=1",
    ];
    for index in 0..2 {
        let correctness = &handoff.correctness.rows[index];
        let expected_id = InitialObligationId::new(base_len + index);
        let obligation = initial_obligations
            .get(expected_id)
            .ok_or(SourceFunctorDefinitionError::InvalidObligation)?;
        if correctness.obligation != expected_id
            || obligation.id != expected_id
            || obligation.kind != kinds[index]
            || obligation.owner != correctness.site
            || obligation.source_range != correctness.source_range
            || !obligation.assumptions.is_empty()
            || obligation.goal.as_str() != goals[index]
            || obligation.provenance.as_str() != provenances[index]
            || obligation.status != InitialObligationStatus::Pending
        {
            return Err(SourceFunctorDefinitionError::InvalidObligation);
        }
    }
    Ok(())
}

fn validate_baseline_prefix(
    table: &InitialObligationTable,
    base_len: usize,
) -> Result<(), SourceFunctorDefinitionError> {
    if table.iter().take(base_len).any(|(_, row)| {
        matches!(
            row.kind,
            InitialObligationKind::PredicatePropertyCorrectness
                | InitialObligationKind::FunctorExistence
                | InitialObligationKind::FunctorUniqueness
        )
    }) {
        return Err(SourceFunctorDefinitionError::InvalidObligation);
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

fn fingerprint_matches(expected: &Option<String>, actual: Option<String>) -> bool {
    expected.as_ref() == actual.as_ref()
}

const fn range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
    SourceRange {
        source_id,
        start,
        end,
    }
}

const fn style_key(style: SourceFunctorDefinitionStyle) -> &'static str {
    match style {
        SourceFunctorDefinitionStyle::Equals => "equals",
        SourceFunctorDefinitionStyle::Means => "means",
    }
}

const fn correctness_key(kind: SourceFunctorCorrectnessKind) -> &'static str {
    match kind {
        SourceFunctorCorrectnessKind::Existence => "existence",
        SourceFunctorCorrectnessKind::Uniqueness => "uniqueness",
    }
}

const fn recovery_key(recovery: SourceFunctorDefinitionRecovery) -> &'static str {
    match recovery {
        SourceFunctorDefinitionRecovery::Normal => "normal",
        SourceFunctorDefinitionRecovery::Degraded => "degraded",
    }
}

fn write_optional_fingerprint(output: &mut String, label: &str, value: Option<&str>) {
    let _ = write!(output, "{label}: ");
    match value {
        Some(value) => {
            let _ = write!(output, "{value:?}");
        }
        None => output.push_str("none"),
    }
    output.push('\n');
}

fn write_target(output: &mut String, target: SourceFunctorDefiniensTarget) {
    match target {
        SourceFunctorDefiniensTarget::Primary(id) => {
            let _ = write!(output, "primary:{}", id.index());
        }
        SourceFunctorDefiniensTarget::Application(id) => {
            let _ = write!(output, "application:{}", id.index());
        }
        SourceFunctorDefiniensTarget::Structure(id) => {
            let _ = write!(output, "structure:{}", id.index());
        }
        SourceFunctorDefiniensTarget::SetTerm(id) => {
            let _ = write!(output, "set-term:{}", id.index());
        }
        SourceFunctorDefiniensTarget::AtomicFormula(id) => {
            let _ = write!(output, "atomic-formula:{}", id.index());
        }
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
#[path = "../tests/support/source_functor_definition_unit.rs"]
mod tests;
