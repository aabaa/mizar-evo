//! Syntax-free struct-property-implementation intake for checker phase 6.

use crate::{
    binding_env::{BindingContextId, BindingId},
    source_application::{SourceFunctorApplicationHandoff, SourceFunctorApplicationId},
    source_atomic_formula::{
        SourceAtomicEdgeId, SourceAtomicEdgeRole, SourceAtomicFormulaHandoff,
        SourceAtomicFormulaId, SourceAtomicFormulaKind, SourceAtomicRequestKind,
        SourceAtomicTermTarget,
    },
    source_context::{
        SourceBindingContextHandoff, SourceItemId, SourceItemRecovery, SourceItemRole,
        SourceItemVisibility,
    },
    source_set_term::{SourceSetTermHandoff, SourceSetTermId},
    source_structure::{
        SourceStructureEdgeId, SourceStructureEdgeRole, SourceStructureHandoff,
        SourceStructureMemberId, SourceStructureMemberRole, SourceStructureRequestId,
        SourceStructureRequestKind, SourceStructureTarget, SourceStructureTermId,
        SourceStructureTermKind,
    },
    source_term::{
        SourcePrimaryTermHandoff, SourcePrimaryTermId, SourcePrimaryTermKind,
        SourcePrimaryTermRecovery, SourcePrimaryTermReferenceId, SourcePrimaryTermReferenceRole,
        SourcePrimaryTermRole,
    },
    source_type::{
        SourceTypeApplicationForm, SourceTypeApplicationHandoff, SourceTypeApplicationId,
        SourceTypeExpressionId, SourceTypeHead, SourceTypeStructureMemberId,
    },
    typed_ast::{
        InitialObligationDraft, InitialObligationGoal, InitialObligationId, InitialObligationKind,
        InitialObligationProvenance, InitialObligationStatus, InitialObligationTable,
        LocalTypeContextId, NodeRecoveryState, TypedArena, TypedNodeId, TypedSiteRef, TypingState,
    },
};
use mizar_resolve::{
    declarations::DeclarationShellId,
    env::{
        ContributionKind, DefinitionEntry, DefinitionId, DefinitionKind, ExportStatus,
        SourceContributionId, SymbolEntry, SymbolEnv, SymbolKind, Visibility,
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

dense_id!(SourcePropertyImplementationId);
dense_id!(SourcePropertyParameterId);
dense_id!(SourcePropertyTargetId);
dense_id!(SourcePropertyDefiniensId);
dense_id!(SourcePropertyCorrectnessId);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePropertyImplementationHandoffInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub implementations: Vec<SourcePropertyImplementationInput>,
    pub parameters: Vec<SourcePropertyParameterInput>,
    pub targets: Vec<SourcePropertyTargetInput>,
    pub definientia: Vec<SourcePropertyDefiniensInput>,
    pub correctness: Vec<SourcePropertyCorrectnessInput>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePropertyImplementationInput {
    pub shell: DeclarationShellId,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub source_ordinal: usize,
    pub context: BindingContextId,
    pub recovery: SourcePropertyImplementationRecovery,
    pub spelling: String,
    pub style: SourcePropertyImplementationStyle,
    pub parameter: SourcePropertyParameterId,
    pub target: SourcePropertyTargetId,
    pub definiens: SourcePropertyDefiniensId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePropertyParameterInput {
    pub owner: SourcePropertyImplementationId,
    pub ordinal: usize,
    pub binding: BindingId,
    pub written_type: SourceTypeApplicationId,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub declaration_range: SourceRange,
    pub context: BindingContextId,
    pub recovery: SourcePropertyImplementationRecovery,
    pub spelling: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePropertyTargetInput {
    pub owner: SourcePropertyImplementationId,
    pub ordinal: usize,
    pub subject: BindingId,
    pub symbol: SymbolId,
    pub definition: DefinitionId,
    pub contribution: SourceContributionId,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub subject_range: SourceRange,
    pub name_range: SourceRange,
    pub spelling: String,
    pub return_type: SourceTypeStructureMemberId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePropertyDefiniensInput {
    pub owner: SourcePropertyImplementationId,
    pub ordinal: usize,
    pub target: SourcePropertyDefiniensTarget,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub context: BindingContextId,
    pub recovery: SourcePropertyImplementationRecovery,
    pub spelling: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePropertyCorrectnessInput {
    pub owner: SourcePropertyImplementationId,
    pub ordinal: usize,
    pub kind: SourcePropertyCorrectnessKind,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub justification: SourceAnchor,
    pub recovery: SourcePropertyImplementationRecovery,
    pub spelling: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourcePropertyImplementationStyle {
    Equals,
    Means,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourcePropertyDefiniensTarget {
    Primary(SourcePrimaryTermId),
    Application(SourceFunctorApplicationId),
    Structure(SourceStructureTermId),
    SetTerm(SourceSetTermId),
    AtomicFormula(SourceAtomicFormulaId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourcePropertyCorrectnessKind {
    Existence,
    Uniqueness,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourcePropertyImplementationRecovery {
    Normal,
    Degraded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePropertyImplementation {
    id: SourcePropertyImplementationId,
    shell: DeclarationShellId,
    site: TypedSiteRef,
    source_range: SourceRange,
    source_ordinal: usize,
    context: BindingContextId,
    recovery: SourcePropertyImplementationRecovery,
    spelling: String,
    style: SourcePropertyImplementationStyle,
    parameter: SourcePropertyParameterId,
    target: SourcePropertyTargetId,
    definiens: SourcePropertyDefiniensId,
}

impl SourcePropertyImplementation {
    pub const fn id(&self) -> SourcePropertyImplementationId {
        self.id
    }
    pub const fn shell(&self) -> DeclarationShellId {
        self.shell
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
    pub const fn recovery(&self) -> SourcePropertyImplementationRecovery {
        self.recovery
    }
    pub fn spelling(&self) -> &str {
        &self.spelling
    }
    pub const fn style(&self) -> SourcePropertyImplementationStyle {
        self.style
    }
    pub const fn parameter(&self) -> SourcePropertyParameterId {
        self.parameter
    }
    pub const fn target(&self) -> SourcePropertyTargetId {
        self.target
    }
    pub const fn definiens(&self) -> SourcePropertyDefiniensId {
        self.definiens
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePropertyParameter {
    id: SourcePropertyParameterId,
    owner: SourcePropertyImplementationId,
    ordinal: usize,
    binding: BindingId,
    written_type: SourceTypeApplicationId,
    site: TypedSiteRef,
    source_range: SourceRange,
    declaration_range: SourceRange,
    context: BindingContextId,
    recovery: SourcePropertyImplementationRecovery,
    spelling: String,
}

impl SourcePropertyParameter {
    pub const fn id(&self) -> SourcePropertyParameterId {
        self.id
    }
    pub const fn owner(&self) -> SourcePropertyImplementationId {
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
    pub const fn recovery(&self) -> SourcePropertyImplementationRecovery {
        self.recovery
    }
    pub fn spelling(&self) -> &str {
        &self.spelling
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePropertyTarget {
    id: SourcePropertyTargetId,
    owner: SourcePropertyImplementationId,
    ordinal: usize,
    subject: BindingId,
    symbol: SymbolId,
    definition: DefinitionId,
    contribution: SourceContributionId,
    site: TypedSiteRef,
    source_range: SourceRange,
    subject_range: SourceRange,
    name_range: SourceRange,
    spelling: String,
    return_type: SourceTypeStructureMemberId,
    origin: SemanticOrigin,
}

impl SourcePropertyTarget {
    pub const fn id(&self) -> SourcePropertyTargetId {
        self.id
    }
    pub const fn owner(&self) -> SourcePropertyImplementationId {
        self.owner
    }
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }
    pub const fn subject(&self) -> BindingId {
        self.subject
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
    pub const fn subject_range(&self) -> SourceRange {
        self.subject_range
    }
    pub const fn name_range(&self) -> SourceRange {
        self.name_range
    }
    pub fn spelling(&self) -> &str {
        &self.spelling
    }
    pub const fn return_type(&self) -> SourceTypeStructureMemberId {
        self.return_type
    }
    pub const fn origin(&self) -> &SemanticOrigin {
        &self.origin
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePropertyDefiniens {
    id: SourcePropertyDefiniensId,
    owner: SourcePropertyImplementationId,
    ordinal: usize,
    target: SourcePropertyDefiniensTarget,
    site: TypedSiteRef,
    source_range: SourceRange,
    context: BindingContextId,
    recovery: SourcePropertyImplementationRecovery,
    spelling: String,
}

impl SourcePropertyDefiniens {
    pub const fn id(&self) -> SourcePropertyDefiniensId {
        self.id
    }
    pub const fn owner(&self) -> SourcePropertyImplementationId {
        self.owner
    }
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }
    pub const fn target(&self) -> SourcePropertyDefiniensTarget {
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
    pub const fn recovery(&self) -> SourcePropertyImplementationRecovery {
        self.recovery
    }
    pub fn spelling(&self) -> &str {
        &self.spelling
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePropertyCorrectness {
    id: SourcePropertyCorrectnessId,
    owner: SourcePropertyImplementationId,
    ordinal: usize,
    kind: SourcePropertyCorrectnessKind,
    site: TypedSiteRef,
    source_range: SourceRange,
    justification: SourceAnchor,
    recovery: SourcePropertyImplementationRecovery,
    spelling: String,
    obligation: InitialObligationId,
}

impl SourcePropertyCorrectness {
    pub const fn id(&self) -> SourcePropertyCorrectnessId {
        self.id
    }
    pub const fn owner(&self) -> SourcePropertyImplementationId {
        self.owner
    }
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }
    pub const fn kind(&self) -> SourcePropertyCorrectnessKind {
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
    pub const fn recovery(&self) -> SourcePropertyImplementationRecovery {
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
    SourcePropertyImplementationTable,
    SourcePropertyImplementation,
    SourcePropertyImplementationId
);
table!(
    SourcePropertyParameterTable,
    SourcePropertyParameter,
    SourcePropertyParameterId
);
table!(
    SourcePropertyTargetTable,
    SourcePropertyTarget,
    SourcePropertyTargetId
);
table!(
    SourcePropertyDefiniensTable,
    SourcePropertyDefiniens,
    SourcePropertyDefiniensId
);
table!(
    SourcePropertyCorrectnessTable,
    SourcePropertyCorrectness,
    SourcePropertyCorrectnessId
);

#[derive(Debug, Clone, PartialEq, Eq)]
struct SourcePropertyResolverIdentity {
    symbol: SymbolId,
    definition: DefinitionId,
    contribution: SourceContributionId,
    origin: SemanticOrigin,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePropertyCarrierIdentity {
    structure: SourcePropertyResolverIdentity,
    field: SourcePropertyResolverIdentity,
    property: SourcePropertyResolverIdentity,
}

impl SourcePropertyCarrierIdentity {
    pub fn structure_symbol(&self) -> &SymbolId {
        &self.structure.symbol
    }
    pub const fn structure_definition(&self) -> DefinitionId {
        self.structure.definition
    }
    pub const fn structure_contribution(&self) -> SourceContributionId {
        self.structure.contribution
    }
    pub const fn structure_origin(&self) -> &SemanticOrigin {
        &self.structure.origin
    }
    pub fn field_symbol(&self) -> &SymbolId {
        &self.field.symbol
    }
    pub const fn field_definition(&self) -> DefinitionId {
        self.field.definition
    }
    pub const fn field_contribution(&self) -> SourceContributionId {
        self.field.contribution
    }
    pub const fn field_origin(&self) -> &SemanticOrigin {
        &self.field.origin
    }
    pub fn property_symbol(&self) -> &SymbolId {
        &self.property.symbol
    }
    pub const fn property_definition(&self) -> DefinitionId {
        self.property.definition
    }
    pub const fn property_contribution(&self) -> SourceContributionId {
        self.property.contribution
    }
    pub const fn property_origin(&self) -> &SemanticOrigin {
        &self.property.origin
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePropertyImplementationHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    carrier_identity: SourcePropertyCarrierIdentity,
    source_context_fingerprint: String,
    source_type_fingerprint: String,
    source_term_fingerprint: String,
    source_functor_application_fingerprint: Option<String>,
    source_structure_fingerprint: Option<String>,
    source_set_term_fingerprint: Option<String>,
    source_atomic_formula_fingerprint: Option<String>,
    implementations: SourcePropertyImplementationTable,
    parameters: SourcePropertyParameterTable,
    targets: SourcePropertyTargetTable,
    definientia: SourcePropertyDefiniensTable,
    correctness: SourcePropertyCorrectnessTable,
}

impl SourcePropertyImplementationHandoff {
    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }
    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
    }
    pub const fn carrier_identity(&self) -> &SourcePropertyCarrierIdentity {
        &self.carrier_identity
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
    pub fn source_functor_application_fingerprint(&self) -> Option<&str> {
        self.source_functor_application_fingerprint.as_deref()
    }
    pub fn source_structure_fingerprint(&self) -> Option<&str> {
        self.source_structure_fingerprint.as_deref()
    }
    pub fn source_set_term_fingerprint(&self) -> Option<&str> {
        self.source_set_term_fingerprint.as_deref()
    }
    pub fn source_atomic_formula_fingerprint(&self) -> Option<&str> {
        self.source_atomic_formula_fingerprint.as_deref()
    }
    pub const fn implementations(&self) -> &SourcePropertyImplementationTable {
        &self.implementations
    }
    pub const fn parameters(&self) -> &SourcePropertyParameterTable {
        &self.parameters
    }
    pub const fn targets(&self) -> &SourcePropertyTargetTable {
        &self.targets
    }
    pub const fn definientia(&self) -> &SourcePropertyDefiniensTable {
        &self.definientia
    }
    pub const fn correctness(&self) -> &SourcePropertyCorrectnessTable {
        &self.correctness
    }

    pub fn debug_text(&self) -> String {
        let mut output = String::from("source-property-implementation-debug-v2\n");
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
            "source-functor-application-fingerprint",
            self.source_functor_application_fingerprint(),
        );
        write_optional_fingerprint(
            &mut output,
            "source-structure-fingerprint",
            self.source_structure_fingerprint(),
        );
        write_optional_fingerprint(
            &mut output,
            "source-set-term-fingerprint",
            self.source_set_term_fingerprint(),
        );
        write_optional_fingerprint(
            &mut output,
            "source-atomic-formula-fingerprint",
            self.source_atomic_formula_fingerprint(),
        );
        write_carrier_identity(
            &mut output,
            0,
            "structure",
            &self.carrier_identity.structure,
        );
        write_carrier_identity(&mut output, 1, "field", &self.carrier_identity.field);
        write_carrier_identity(&mut output, 2, "property", &self.carrier_identity.property);
        for (id, row) in self.implementations.iter() {
            let _ = write!(
                output,
                "implementation#{} shell={} range={}..{} site=",
                id.index(),
                row.shell.index(),
                row.source_range.start,
                row.source_range.end
            );
            write_site(&mut output, &row.site);
            let _ = writeln!(
                output,
                " ordinal={} context={} recovery={} spelling={:?} style={} parameter={} target={} definiens={}",
                row.source_ordinal,
                row.context.index(),
                recovery_key(row.recovery),
                row.spelling,
                style_key(row.style),
                row.parameter.index(),
                row.target.index(),
                row.definiens.index()
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
                row.declaration_range.end
            );
            write_site(&mut output, &row.site);
            let _ = writeln!(
                output,
                " context={} recovery={} spelling={:?}",
                row.context.index(),
                recovery_key(row.recovery),
                row.spelling
            );
        }
        for (id, row) in self.targets.iter() {
            let _ = write!(
                output,
                "target#{} owner={} ordinal={} subject={} symbol={:?} definition={} contribution={} range={}..{} subject_range={}..{} name_range={}..{} site=",
                id.index(),
                row.owner.index(),
                row.ordinal,
                row.subject.index(),
                row.symbol.fqn().as_str(),
                row.definition.index(),
                row.contribution.index(),
                row.source_range.start,
                row.source_range.end,
                row.subject_range.start,
                row.subject_range.end,
                row.name_range.start,
                row.name_range.end
            );
            write_site(&mut output, &row.site);
            let _ = write!(
                output,
                " spelling={:?} return_type={} origin_range=",
                row.spelling,
                row.return_type.index()
            );
            write_anchor_range(&mut output, row.origin.anchor());
            let _ = writeln!(output, " origin_path={:?}", row.origin.structural_path());
        }
        for (id, row) in self.definientia.iter() {
            let _ = write!(
                output,
                "definiens#{} owner={} ordinal={} target=",
                id.index(),
                row.owner.index(),
                row.ordinal
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
                row.spelling
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
                row.source_range.end
            );
            write_site(&mut output, &row.site);
            output.push_str(" justification=");
            write_anchor(&mut output, &row.justification);
            let _ = writeln!(
                output,
                " recovery={} spelling={:?} obligation={}",
                recovery_key(row.recovery),
                row.spelling,
                row.obligation.index()
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
    ) -> Result<(), SourcePropertyImplementationError> {
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
            return Err(SourcePropertyImplementationError::SourceIdentityMismatch);
        }
        if self.source_context_fingerprint != source_context.debug_text()
            || self.source_type_fingerprint != source_type.debug_text()
            || self.source_term_fingerprint != source_term.debug_text()
            || !fingerprint_matches(
                &self.source_functor_application_fingerprint,
                applications.map(|row| row.debug_text()),
            )
            || !fingerprint_matches(
                &self.source_structure_fingerprint,
                structures.map(|row| row.debug_text()),
            )
            || !fingerprint_matches(
                &self.source_set_term_fingerprint,
                set_terms.map(|row| row.debug_text()),
            )
            || !fingerprint_matches(
                &self.source_atomic_formula_fingerprint,
                atomic_formulas.map(|row| row.debug_text()),
            )
        {
            return Err(SourcePropertyImplementationError::DependencyMismatch);
        }
        validate_handoff(
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

/// Exact Task-264 equals selector occurrence associated with its authenticated field symbol.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePropertyEqualsSelectorAssociation {
    implementation: SourcePropertyImplementationId,
    definiens: SourcePropertyDefiniensId,
    structure_term: SourceStructureTermId,
    member: SourceStructureMemberId,
    member_request: SourceStructureRequestId,
    base_edge: SourceStructureEdgeId,
    base_term: SourcePrimaryTermId,
    base_reference: SourcePrimaryTermReferenceId,
    base_binding: BindingId,
    selector_symbol: SymbolId,
}

impl SourcePropertyEqualsSelectorAssociation {
    pub const fn implementation(&self) -> SourcePropertyImplementationId {
        self.implementation
    }

    pub const fn definiens(&self) -> SourcePropertyDefiniensId {
        self.definiens
    }

    pub const fn structure_term(&self) -> SourceStructureTermId {
        self.structure_term
    }

    pub const fn member(&self) -> SourceStructureMemberId {
        self.member
    }

    pub const fn member_request(&self) -> SourceStructureRequestId {
        self.member_request
    }

    pub const fn base_edge(&self) -> SourceStructureEdgeId {
        self.base_edge
    }

    pub const fn base_term(&self) -> SourcePrimaryTermId {
        self.base_term
    }

    pub const fn base_reference(&self) -> SourcePrimaryTermReferenceId {
        self.base_reference
    }

    pub const fn base_binding(&self) -> BindingId {
        self.base_binding
    }

    pub const fn selector_symbol(&self) -> &SymbolId {
        &self.selector_symbol
    }
}

/// Immutable Task-264 equals-only selector identity handoff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePropertyEqualsSelectorIdentityHandoff {
    property: SourcePropertyImplementationHandoff,
    terms: SourcePrimaryTermHandoff,
    structures: SourceStructureHandoff,
    association: SourcePropertyEqualsSelectorAssociation,
}

impl SourcePropertyEqualsSelectorIdentityHandoff {
    pub const fn source_id(&self) -> SourceId {
        self.property.source_id()
    }

    pub const fn module_id(&self) -> &ModuleId {
        self.property.module_id()
    }

    pub const fn property(&self) -> &SourcePropertyImplementationHandoff {
        &self.property
    }

    pub const fn terms(&self) -> &SourcePrimaryTermHandoff {
        &self.terms
    }

    pub const fn structures(&self) -> &SourceStructureHandoff {
        &self.structures
    }

    pub const fn association(&self) -> &SourcePropertyEqualsSelectorAssociation {
        &self.association
    }

    pub fn debug_text(&self) -> String {
        format!(
            concat!(
                "source-property-equals-selector-identity-debug-v1\n",
                "module: {}\n",
                "property-fingerprint: {:?}\n",
                "primary-term-fingerprint: {:?}\n",
                "structure-fingerprint: {:?}\n",
                "association implementation={} definiens={} structure-term={} member={} ",
                "member-request={} base-edge={} base-term={} base-reference={} base-binding={} selector={:?}\n",
            ),
            self.module_id().path().as_str(),
            self.property.debug_text(),
            self.terms.debug_text(),
            self.structures.debug_text(),
            self.association.implementation.index(),
            self.association.definiens.index(),
            self.association.structure_term.index(),
            self.association.member.index(),
            self.association.member_request.index(),
            self.association.base_edge.index(),
            self.association.base_term.index(),
            self.association.base_reference.index(),
            self.association.base_binding.index(),
            self.association.selector_symbol.fqn().as_str(),
        )
    }

    fn validate(&self) -> Result<(), SourcePropertyEqualsSelectorIdentityError> {
        validate_equals_selector_environment(&self.property, &self.terms, &self.structures)?;
        validate_equals_selector_profile(&self.property)?;
        validate_equals_selector_dependencies(&self.property, &self.terms, &self.structures)?;
        validate_equals_selector_association(
            &self.property,
            &self.terms,
            &self.structures,
            &self.association,
        )
    }
}

/// Errors raised while authenticating the exact Task-264 equals selector identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourcePropertyEqualsSelectorIdentityError {
    EnvironmentMismatch,
    UnsupportedProfile,
    DependencyMismatch,
    InvalidSelectorIdentity,
}

impl fmt::Display for SourcePropertyEqualsSelectorIdentityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EnvironmentMismatch => {
                formatter.write_str("property equals selector environment is invalid")
            }
            Self::UnsupportedProfile => {
                formatter.write_str("property equals selector profile is unsupported")
            }
            Self::DependencyMismatch => {
                formatter.write_str("property equals selector dependency is invalid")
            }
            Self::InvalidSelectorIdentity => {
                formatter.write_str("property equals selector identity is invalid")
            }
        }
    }
}

impl Error for SourcePropertyEqualsSelectorIdentityError {}

/// Builds the immutable Task-264 equals selector identity handoff.
pub struct SourcePropertyEqualsSelectorIdentityProducer;

impl SourcePropertyEqualsSelectorIdentityProducer {
    pub fn build(
        env: &SymbolEnv,
        property: SourcePropertyImplementationHandoff,
        terms: SourcePrimaryTermHandoff,
        structures: SourceStructureHandoff,
    ) -> Result<
        SourcePropertyEqualsSelectorIdentityHandoff,
        SourcePropertyEqualsSelectorIdentityError,
    > {
        validate_equals_selector_build_environment(env, &property)?;
        validate_equals_selector_environment(&property, &terms, &structures)?;
        validate_equals_selector_profile(&property)?;
        validate_equals_selector_dependencies(&property, &terms, &structures)?;
        validate_equals_selector_resolver(env, &property, &structures)?;
        let association = SourcePropertyEqualsSelectorAssociation {
            implementation: SourcePropertyImplementationId::new(0),
            definiens: SourcePropertyDefiniensId::new(0),
            structure_term: SourceStructureTermId::new(0),
            member: SourceStructureMemberId::new(0),
            member_request: SourceStructureRequestId::new(0),
            base_edge: SourceStructureEdgeId::new(0),
            base_term: SourcePrimaryTermId::new(0),
            base_reference: SourcePrimaryTermReferenceId::new(0),
            base_binding: BindingId::new(0),
            selector_symbol: property.carrier_identity().field_symbol().clone(),
        };
        validate_equals_selector_association(&property, &terms, &structures, &association)?;
        let handoff = SourcePropertyEqualsSelectorIdentityHandoff {
            property,
            terms,
            structures,
            association,
        };
        handoff.validate()?;
        Ok(handoff)
    }
}

fn validate_equals_selector_build_environment(
    env: &SymbolEnv,
    property: &SourcePropertyImplementationHandoff,
) -> Result<(), SourcePropertyEqualsSelectorIdentityError> {
    let contribution = env
        .contributions()
        .get(property.carrier_identity().field_contribution())
        .ok_or(SourcePropertyEqualsSelectorIdentityError::EnvironmentMismatch)?;
    if env.module_id() != property.module_id()
        || env.symbols().len() != 3
        || env.definitions().len() != 3
        || env.contributions().len() != 1
        || contribution.module() != property.module_id()
        || !matches!(
            contribution.kind(),
            ContributionKind::LocalSource { source_id } if *source_id == property.source_id()
        )
    {
        return Err(SourcePropertyEqualsSelectorIdentityError::EnvironmentMismatch);
    }
    Ok(())
}

fn validate_equals_selector_environment(
    property: &SourcePropertyImplementationHandoff,
    terms: &SourcePrimaryTermHandoff,
    structures: &SourceStructureHandoff,
) -> Result<(), SourcePropertyEqualsSelectorIdentityError> {
    if property.source_id() != terms.source_id()
        || property.source_id() != structures.source_id()
        || property.module_id() != terms.module_id()
        || property.module_id() != structures.module_id()
    {
        return Err(SourcePropertyEqualsSelectorIdentityError::EnvironmentMismatch);
    }
    Ok(())
}

fn validate_equals_selector_profile(
    property: &SourcePropertyImplementationHandoff,
) -> Result<(), SourcePropertyEqualsSelectorIdentityError> {
    let Some((implementation_id, implementation)) = property.implementations().iter().next() else {
        return Err(SourcePropertyEqualsSelectorIdentityError::DependencyMismatch);
    };
    if implementation.style() != SourcePropertyImplementationStyle::Equals {
        return Err(SourcePropertyEqualsSelectorIdentityError::UnsupportedProfile);
    }
    if property.implementations().len() != 1
        || property.parameters().len() != 1
        || property.targets().len() != 1
        || property.definientia().len() != 1
        || !property.correctness().is_empty()
        || implementation_id != SourcePropertyImplementationId::new(0)
        || implementation.id() != implementation_id
        || implementation.definiens() != SourcePropertyDefiniensId::new(0)
        || property.source_functor_application_fingerprint().is_some()
        || property.source_structure_fingerprint().is_none()
        || property.source_set_term_fingerprint().is_some()
        || property.source_atomic_formula_fingerprint().is_some()
    {
        return Err(SourcePropertyEqualsSelectorIdentityError::DependencyMismatch);
    }
    Ok(())
}

fn validate_equals_selector_dependencies(
    property: &SourcePropertyImplementationHandoff,
    terms: &SourcePrimaryTermHandoff,
    structures: &SourceStructureHandoff,
) -> Result<(), SourcePropertyEqualsSelectorIdentityError> {
    let term_fingerprint = terms.debug_text();
    let structure_fingerprint = structures.debug_text();
    if property.source_term_fingerprint() != term_fingerprint
        || property.source_structure_fingerprint() != Some(structure_fingerprint.as_str())
        || structures.primary_term_fingerprint() != term_fingerprint
        || structures.application_fingerprint().is_some()
    {
        return Err(SourcePropertyEqualsSelectorIdentityError::DependencyMismatch);
    }
    Ok(())
}

fn validate_equals_selector_resolver(
    env: &SymbolEnv,
    property: &SourcePropertyImplementationHandoff,
    structures: &SourceStructureHandoff,
) -> Result<(), SourcePropertyEqualsSelectorIdentityError> {
    let identity = property.carrier_identity();
    let symbol = env
        .symbols()
        .get(identity.field_symbol())
        .ok_or(SourcePropertyEqualsSelectorIdentityError::InvalidSelectorIdentity)?;
    let definition = env
        .definitions()
        .get(identity.field_definition())
        .ok_or(SourcePropertyEqualsSelectorIdentityError::InvalidSelectorIdentity)?;
    let member = structures
        .members()
        .get(SourceStructureMemberId::new(0))
        .ok_or(SourcePropertyEqualsSelectorIdentityError::InvalidSelectorIdentity)?;
    let contribution = env
        .contributions()
        .get(identity.field_contribution())
        .ok_or(SourcePropertyEqualsSelectorIdentityError::InvalidSelectorIdentity)?;
    if env.module_id() != property.module_id()
        || !resolver_identity_matches(
            symbol,
            definition,
            property.source_id(),
            property.module_id(),
            identity.field_contribution(),
            SymbolKind::Selector,
            DefinitionKind::Selector,
            member.spelling(),
            range(property.source_id(), 45, 66),
            &[4, 0, 11, 0, 18, 0],
        )
        || symbol.symbol() != identity.field_symbol()
        || definition.id() != identity.field_definition()
        || definition.contribution() != identity.field_contribution()
        || definition.origin() != identity.field_origin()
        || contribution.module() != property.module_id()
        || !matches!(
            contribution.kind(),
            ContributionKind::LocalSource { source_id } if *source_id == property.source_id()
        )
        || contribution.effects().symbols().len() != 3
        || contribution.effects().definitions().len() != 3
        || !contribution
            .effects()
            .symbols()
            .contains(identity.field_symbol())
        || !contribution
            .effects()
            .definitions()
            .contains(&identity.field_definition())
    {
        return Err(SourcePropertyEqualsSelectorIdentityError::InvalidSelectorIdentity);
    }
    Ok(())
}

fn validate_equals_selector_association(
    property: &SourcePropertyImplementationHandoff,
    terms: &SourcePrimaryTermHandoff,
    structures: &SourceStructureHandoff,
    association: &SourcePropertyEqualsSelectorAssociation,
) -> Result<(), SourcePropertyEqualsSelectorIdentityError> {
    let implementation = property
        .implementations()
        .get(association.implementation())
        .ok_or(SourcePropertyEqualsSelectorIdentityError::InvalidSelectorIdentity)?;
    let definiens = property
        .definientia()
        .get(association.definiens())
        .ok_or(SourcePropertyEqualsSelectorIdentityError::InvalidSelectorIdentity)?;
    let parameter = property
        .parameters()
        .iter()
        .next()
        .map(|(_, row)| row)
        .ok_or(SourcePropertyEqualsSelectorIdentityError::InvalidSelectorIdentity)?;
    let target = property
        .targets()
        .iter()
        .next()
        .map(|(_, row)| row)
        .ok_or(SourcePropertyEqualsSelectorIdentityError::InvalidSelectorIdentity)?;
    let base_term = terms
        .terms()
        .get(association.base_term())
        .ok_or(SourcePropertyEqualsSelectorIdentityError::InvalidSelectorIdentity)?;
    let base_reference = terms
        .references()
        .get(association.base_reference())
        .ok_or(SourcePropertyEqualsSelectorIdentityError::InvalidSelectorIdentity)?;
    let structure_term = structures
        .terms()
        .get(association.structure_term())
        .ok_or(SourcePropertyEqualsSelectorIdentityError::InvalidSelectorIdentity)?;
    let member = structures
        .members()
        .get(association.member())
        .ok_or(SourcePropertyEqualsSelectorIdentityError::InvalidSelectorIdentity)?;
    let edge = structures
        .edges()
        .get(association.base_edge())
        .ok_or(SourcePropertyEqualsSelectorIdentityError::InvalidSelectorIdentity)?;
    let member_request = structures
        .requests()
        .get(association.member_request())
        .ok_or(SourcePropertyEqualsSelectorIdentityError::InvalidSelectorIdentity)?;
    if terms.terms().len() != 1
        || terms.references().len() != 1
        || !terms.numeric_type_requests().is_empty()
        || structures.terms().len() != 1
        || !structures.wrappers().is_empty()
        || !structures.roots().is_empty()
        || structures.members().len() != 1
        || !structures.field_updates().is_empty()
        || structures.edges().len() != 1
        || structures.requests().len() != 3
        || association.implementation() != SourcePropertyImplementationId::new(0)
        || association.definiens() != SourcePropertyDefiniensId::new(0)
        || association.structure_term() != SourceStructureTermId::new(0)
        || association.member() != SourceStructureMemberId::new(0)
        || association.member_request() != SourceStructureRequestId::new(0)
        || association.base_edge() != SourceStructureEdgeId::new(0)
        || association.base_term() != SourcePrimaryTermId::new(0)
        || association.base_reference() != SourcePrimaryTermReferenceId::new(0)
        || association.base_binding() != BindingId::new(0)
        || association.selector_symbol() != property.carrier_identity().field_symbol()
        || implementation.definiens() != association.definiens()
        || definiens.owner() != association.implementation()
        || definiens.target()
            != SourcePropertyDefiniensTarget::Structure(association.structure_term())
        || parameter.binding() != association.base_binding()
        || target.subject() != association.base_binding()
        || base_term.kind() != SourcePrimaryTermKind::VariableReference
        || base_term.role() != SourcePrimaryTermRole::Value
        || base_term.recovery() != SourcePrimaryTermRecovery::Normal
        || base_term.parent().is_some()
        || base_term.source_ordinal() != 0
        || base_term.context() != BindingContextId::new(1)
        || base_term.site() != &TypedSiteRef::Node(TypedNodeId::new(48))
        || base_term.source_range() != range(property.source_id(), 173, 174)
        || base_term.spelling() != "M"
        || base_reference.term() != association.base_term()
        || base_reference.binding() != association.base_binding()
        || base_reference.role() != SourcePrimaryTermReferenceRole::Variable
        || base_reference
            .lexical_scope()
            .is_none_or(|scope| scope.path() != [4])
        || base_reference.use_ordinal() != 1
        || structure_term.kind() != SourceStructureTermKind::SelectorAccess
        || structure_term.recovery() != crate::source_structure::SourceStructureRecovery::Normal
        || structure_term.source_ordinal() != 0
        || structure_term.context() != BindingContextId::new(1)
        || structure_term.site() != &TypedSiteRef::Node(TypedNodeId::new(49))
        || structure_term.source_range() != range(property.source_id(), 173, 182)
        || structure_term.spelling() != "M.carrier"
        || member.term() != association.structure_term()
        || member.ordinal() != 0
        || member.role() != SourceStructureMemberRole::Selector
        || member.parent().is_some()
        || member.site() != &TypedSiteRef::Node(TypedNodeId::new(31))
        || member.source_range() != range(property.source_id(), 175, 182)
        || member.spelling() != "carrier"
        || edge.term() != association.structure_term()
        || edge.ordinal() != 0
        || edge.role() != SourceStructureEdgeRole::SelectorBase
        || edge.member().is_some()
        || edge.target() != SourceStructureTarget::Primary(association.base_term())
        || member_request.term() != association.structure_term()
        || member_request.member() != Some(association.member())
        || member_request.request_ordinal() != 0
        || member_request.kind() != SourceStructureRequestKind::MemberIdentity
    {
        return Err(SourcePropertyEqualsSelectorIdentityError::InvalidSelectorIdentity);
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePropertyImplementationProjection {
    base_initial_obligations: InitialObligationTable,
    handoff: SourcePropertyImplementationHandoff,
    initial_obligations: InitialObligationTable,
}

impl SourcePropertyImplementationProjection {
    pub const fn base_initial_obligations(&self) -> &InitialObligationTable {
        &self.base_initial_obligations
    }
    pub const fn handoff(&self) -> &SourcePropertyImplementationHandoff {
        &self.handoff
    }
    pub const fn initial_obligations(&self) -> &InitialObligationTable {
        &self.initial_obligations
    }
    pub fn into_parts(
        self,
    ) -> (
        InitialObligationTable,
        SourcePropertyImplementationHandoff,
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
pub enum SourcePropertyImplementationError {
    SourceIdentityMismatch,
    DependencyMismatch,
    InvalidResolverTarget { index: usize },
    InvalidImplementation { index: usize },
    InvalidParameter { index: usize },
    InvalidTarget { index: usize },
    InvalidDefiniens { index: usize },
    InvalidCorrectness { index: usize },
    InvalidObligation,
    InvalidArenaOwnership,
    UnsupportedTaskShape,
}

impl fmt::Display for SourcePropertyImplementationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SourceIdentityMismatch => {
                formatter.write_str("property-implementation source identity mismatch")
            }
            Self::DependencyMismatch => {
                formatter.write_str("property-implementation dependency mismatch")
            }
            Self::InvalidResolverTarget { index } => {
                write!(formatter, "invalid property resolver target {index}")
            }
            Self::InvalidImplementation { index } => {
                write!(formatter, "invalid source property implementation {index}")
            }
            Self::InvalidParameter { index } => {
                write!(formatter, "invalid source property parameter {index}")
            }
            Self::InvalidTarget { index } => {
                write!(formatter, "invalid source property target {index}")
            }
            Self::InvalidDefiniens { index } => {
                write!(formatter, "invalid source property definiens {index}")
            }
            Self::InvalidCorrectness { index } => {
                write!(formatter, "invalid source property correctness {index}")
            }
            Self::InvalidObligation => {
                formatter.write_str("invalid property-implementation correctness obligation")
            }
            Self::InvalidArenaOwnership => {
                formatter.write_str("invalid property-implementation typed-arena ownership")
            }
            Self::UnsupportedTaskShape => {
                formatter.write_str("unsupported property-implementation task shape")
            }
        }
    }
}

impl Error for SourcePropertyImplementationError {}

pub struct SourcePropertyImplementationProducer;

impl SourcePropertyImplementationProducer {
    // Rationale: the public producer ABI names every independently frozen lower handoff.
    #[allow(clippy::too_many_arguments)]
    pub fn build(
        input: SourcePropertyImplementationHandoffInput,
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
    ) -> Result<SourcePropertyImplementationProjection, SourcePropertyImplementationError> {
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
            return Err(SourcePropertyImplementationError::SourceIdentityMismatch);
        }
        validate_shape(&input)?;
        validate_baseline(base_initial_obligations)?;
        let carrier_identity = validate_resolver_target(&input, env, source_type)?;
        validate_input(
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
        let mut obligation_ids = Vec::with_capacity(input.correctness.len());
        for (index, row) in input.correctness.iter().enumerate() {
            let (kind, goal, provenance) = match row.kind {
                SourcePropertyCorrectnessKind::Existence => (
                    InitialObligationKind::PropertyImplementationExistence,
                    "source.definition.property-implementation.correctness:implementation=0:existence",
                    "source.definition.property-implementation:implementation=0:correctness=0",
                ),
                SourcePropertyCorrectnessKind::Uniqueness => (
                    InitialObligationKind::PropertyImplementationUniqueness,
                    "source.definition.property-implementation.correctness:implementation=0:uniqueness",
                    "source.definition.property-implementation:implementation=0:correctness=1",
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
                return Err(SourcePropertyImplementationError::InvalidObligation);
            }
            obligation_ids.push(id);
        }

        let implementations = SourcePropertyImplementationTable {
            rows: input
                .implementations
                .into_iter()
                .enumerate()
                .map(|(index, row)| SourcePropertyImplementation {
                    id: SourcePropertyImplementationId::new(index),
                    shell: row.shell,
                    site: row.site,
                    source_range: row.source_range,
                    source_ordinal: row.source_ordinal,
                    context: row.context,
                    recovery: row.recovery,
                    spelling: row.spelling,
                    style: row.style,
                    parameter: row.parameter,
                    target: row.target,
                    definiens: row.definiens,
                })
                .collect(),
        };
        let parameters = SourcePropertyParameterTable {
            rows: input
                .parameters
                .into_iter()
                .enumerate()
                .map(|(index, row)| SourcePropertyParameter {
                    id: SourcePropertyParameterId::new(index),
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
        let targets = SourcePropertyTargetTable {
            rows: input
                .targets
                .into_iter()
                .enumerate()
                .map(|(index, row)| SourcePropertyTarget {
                    id: SourcePropertyTargetId::new(index),
                    owner: row.owner,
                    ordinal: row.ordinal,
                    subject: row.subject,
                    symbol: row.symbol,
                    definition: row.definition,
                    contribution: row.contribution,
                    site: row.site,
                    source_range: row.source_range,
                    subject_range: row.subject_range,
                    name_range: row.name_range,
                    spelling: row.spelling,
                    return_type: row.return_type,
                    origin: carrier_identity.property_origin().clone(),
                })
                .collect(),
        };
        let definientia = SourcePropertyDefiniensTable {
            rows: input
                .definientia
                .into_iter()
                .enumerate()
                .map(|(index, row)| SourcePropertyDefiniens {
                    id: SourcePropertyDefiniensId::new(index),
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
        let correctness = SourcePropertyCorrectnessTable {
            rows: input
                .correctness
                .into_iter()
                .zip(obligation_ids)
                .enumerate()
                .map(|(index, (row, obligation))| SourcePropertyCorrectness {
                    id: SourcePropertyCorrectnessId::new(index),
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
        let handoff = SourcePropertyImplementationHandoff {
            source_id: input.source_id,
            module_id: input.module_id,
            carrier_identity,
            source_context_fingerprint: source_context.debug_text(),
            source_type_fingerprint: source_type.debug_text(),
            source_term_fingerprint: source_term.debug_text(),
            source_functor_application_fingerprint: applications.map(|row| row.debug_text()),
            source_structure_fingerprint: structures.map(|row| row.debug_text()),
            source_set_term_fingerprint: set_terms.map(|row| row.debug_text()),
            source_atomic_formula_fingerprint: atomic_formulas.map(|row| row.debug_text()),
            implementations,
            parameters,
            targets,
            definientia,
            correctness,
        };
        validate_handoff(
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
        Ok(SourcePropertyImplementationProjection {
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
) -> Result<(), SourcePropertyImplementationError> {
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
        return Err(SourcePropertyImplementationError::SourceIdentityMismatch);
    }
    source_type
        .validate_installation(source_id, module_id, arena)
        .map_err(|_| SourcePropertyImplementationError::DependencyMismatch)?;
    source_term
        .validate_installation(source_id, module_id, arena)
        .map_err(|_| {
            if atomic_formulas.is_some() {
                SourcePropertyImplementationError::InvalidDefiniens { index: 0 }
            } else {
                SourcePropertyImplementationError::DependencyMismatch
            }
        })?;
    if atomic_formulas.is_some() && source_term.terms().len() != 2 {
        return Err(SourcePropertyImplementationError::InvalidDefiniens { index: 0 });
    }
    if let Some(row) = applications {
        row.validate_installation(source_id, module_id, source_term)
            .map_err(|_| SourcePropertyImplementationError::DependencyMismatch)?;
    }
    if let Some(row) = structures {
        row.validate_installation(source_id, module_id, source_term, applications, arena)
            .map_err(|_| SourcePropertyImplementationError::DependencyMismatch)?;
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
        .map_err(|_| SourcePropertyImplementationError::DependencyMismatch)?;
    }
    if let Some(row) = atomic_formulas {
        row.validate_installation(
            source_id,
            module_id,
            source_term,
            applications,
            structures,
            set_terms,
            arena,
        )
        .map_err(|_| SourcePropertyImplementationError::InvalidDefiniens { index: 0 })?;
    }
    Ok(())
}

fn validate_shape(
    input: &SourcePropertyImplementationHandoffInput,
) -> Result<(), SourcePropertyImplementationError> {
    if input.implementations.len() != 1
        || input.parameters.len() != 1
        || input.targets.len() != 1
        || input.definientia.len() != 1
        || !matches!(input.correctness.len(), 0 | 2)
    {
        return Err(SourcePropertyImplementationError::UnsupportedTaskShape);
    }
    Ok(())
}

fn validate_baseline(
    table: &InitialObligationTable,
) -> Result<(), SourcePropertyImplementationError> {
    if table.iter().any(|(_, row)| property_family_kind(row.kind)) {
        return Err(SourcePropertyImplementationError::InvalidObligation);
    }
    Ok(())
}

fn validate_resolver_target(
    input: &SourcePropertyImplementationHandoffInput,
    env: &SymbolEnv,
    source_type: &SourceTypeApplicationHandoff,
) -> Result<SourcePropertyCarrierIdentity, SourcePropertyImplementationError> {
    if env.symbols().len() != 3 || env.definitions().len() != 3 || env.contributions().len() != 1 {
        return Err(SourcePropertyImplementationError::InvalidResolverTarget { index: 0 });
    }
    let row = &input.targets[0];
    let contribution = env
        .contributions()
        .get(row.contribution)
        .ok_or(SourcePropertyImplementationError::InvalidResolverTarget { index: 0 })?;
    let mut rows = Vec::with_capacity(3);
    for index in 0..3 {
        let definition = env
            .definitions()
            .iter()
            .find(|row| row.id().index() == index)
            .ok_or(SourcePropertyImplementationError::InvalidResolverTarget { index: 0 })?;
        let symbol = env
            .symbols()
            .get(definition.symbol())
            .ok_or(SourcePropertyImplementationError::InvalidResolverTarget { index: 0 })?;
        rows.push((symbol, definition));
    }
    let expected = [
        (
            SymbolKind::Structure,
            DefinitionKind::Structure,
            "Task264Carrier",
            range(input.source_id, 13, 101),
            &[4, 0, 11, 0][..],
        ),
        (
            SymbolKind::Selector,
            DefinitionKind::Selector,
            "carrier",
            range(input.source_id, 45, 66),
            &[4, 0, 11, 0, 18, 0][..],
        ),
        (
            SymbolKind::Selector,
            DefinitionKind::Selector,
            "marker",
            range(input.source_id, 71, 94),
            &[4, 0, 11, 0, 19, 1][..],
        ),
    ];
    if row.definition.index() != 2
        || row.contribution.index() != 0
        || contribution.module() != &input.module_id
        || contribution.kind()
            != &(ContributionKind::LocalSource {
                source_id: input.source_id,
            })
        || contribution.effects().symbols().len() != 3
        || contribution.effects().definitions().len() != 3
        || rows
            .iter()
            .enumerate()
            .any(|(index, (symbol, definition))| {
                let (symbol_kind, definition_kind, spelling, source_range, path) = expected[index];
                !resolver_identity_matches(
                    symbol,
                    definition,
                    input.source_id,
                    &input.module_id,
                    row.contribution,
                    symbol_kind,
                    definition_kind,
                    spelling,
                    source_range,
                    path,
                ) || !contribution
                    .effects()
                    .symbols()
                    .contains(definition.symbol())
                    || !contribution
                        .effects()
                        .definitions()
                        .contains(&definition.id())
            })
        || row.symbol != *rows[2].1.symbol()
        || row.definition != rows[2].1.id()
    {
        return Err(SourcePropertyImplementationError::InvalidResolverTarget { index: 0 });
    }
    let carrier_head_matches = matches!(
        source_type
            .expressions()
            .get(SourceTypeExpressionId::new(0))
            .map(|row| row.head()),
        Some(SourceTypeHead::Symbol {
            symbol,
            contribution,
        }) if symbol == rows[0].1.symbol()
            && *contribution == rows[0].1.contribution()
    );
    if !carrier_head_matches {
        return Err(SourcePropertyImplementationError::InvalidResolverTarget { index: 0 });
    }
    let identities = rows
        .into_iter()
        .map(|(symbol, definition)| SourcePropertyResolverIdentity {
            symbol: symbol.symbol().clone(),
            definition: definition.id(),
            contribution: definition.contribution(),
            origin: definition.origin().clone(),
        })
        .collect::<Vec<_>>();
    let [structure, field, property]: [SourcePropertyResolverIdentity; 3] =
        identities
            .try_into()
            .map_err(|_| SourcePropertyImplementationError::InvalidResolverTarget { index: 0 })?;
    Ok(SourcePropertyCarrierIdentity {
        structure,
        field,
        property,
    })
}

// Rationale: the fixed resolver role check keeps every authenticated field explicit.
#[allow(clippy::too_many_arguments)]
fn resolver_identity_matches(
    symbol: &SymbolEntry,
    definition: &DefinitionEntry,
    source_id: SourceId,
    module_id: &ModuleId,
    contribution: SourceContributionId,
    symbol_kind: SymbolKind,
    definition_kind: DefinitionKind,
    spelling: &str,
    source_range: SourceRange,
    path: &[u32],
) -> bool {
    symbol.symbol() == definition.symbol()
        && symbol.symbol().module() == module_id
        && symbol.kind() == symbol_kind
        && symbol.visibility() == Visibility::Public
        && symbol.export_status() == ExportStatus::Exported
        && symbol.primary_spelling() == spelling
        && symbol.contribution() == contribution
        && symbol.origin() == definition.origin()
        && definition.kind() == definition_kind
        && definition.visibility() == Visibility::Public
        && definition.contribution() == contribution
        && definition.conflict().is_none()
        && normal_origin(
            definition.origin(),
            source_id,
            module_id,
            source_range,
            path,
        )
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Profile {
    Means,
    Equals,
}

// Rationale: row validation keeps each optional lower family explicit and independently testable.
#[allow(clippy::too_many_arguments)]
fn validate_input(
    input: &SourcePropertyImplementationHandoffInput,
    source_context: &SourceBindingContextHandoff,
    source_type: &SourceTypeApplicationHandoff,
    source_term: &SourcePrimaryTermHandoff,
    applications: Option<&SourceFunctorApplicationHandoff>,
    structures: Option<&SourceStructureHandoff>,
    set_terms: Option<&SourceSetTermHandoff>,
    atomic_formulas: Option<&SourceAtomicFormulaHandoff>,
    arena: &TypedArena,
) -> Result<(), SourcePropertyImplementationError> {
    let implementation = &input.implementations[0];
    let profile = match (implementation.style, implementation.source_range.end) {
        (SourcePropertyImplementationStyle::Means, 262) => Profile::Means,
        (SourcePropertyImplementationStyle::Equals, 188) => Profile::Equals,
        _ => return Err(SourcePropertyImplementationError::InvalidImplementation { index: 0 }),
    };
    let (
        owner_node,
        implementation_end,
        implementation_spelling,
        parameter_node,
        definiens_node,
        definiens_range,
        definiens_spelling,
    ) = match profile {
        Profile::Means => (
            81,
            262,
            "definition\n  let M be Task264Carrier;\n  property M.marker means it = it;\n  existence by computation(steps: 1);\n  uniqueness by computation(steps: 1);\nend;",
            65,
            72,
            (172, 179),
            "it = it",
        ),
        Profile::Equals => (
            52,
            188,
            "definition\n  let M be Task264Carrier;\n  property M.marker equals M.carrier;\nend;",
            47,
            51,
            (173, 182),
            "M.carrier",
        ),
    };
    if implementation.shell.index() != 4
        || implementation.site != TypedSiteRef::Node(TypedNodeId::new(owner_node))
        || implementation.source_range != range(input.source_id, 108, implementation_end)
        || implementation.source_ordinal != 0
        || implementation.context != BindingContextId::new(1)
        || implementation.recovery != SourcePropertyImplementationRecovery::Normal
        || implementation.spelling != implementation_spelling
        || implementation.parameter != SourcePropertyParameterId::new(0)
        || implementation.target != SourcePropertyTargetId::new(0)
        || implementation.definiens != SourcePropertyDefiniensId::new(0)
    {
        return Err(SourcePropertyImplementationError::InvalidImplementation { index: 0 });
    }
    validate_context(
        input,
        source_context,
        owner_node,
        implementation_end,
        parameter_node,
    )?;

    let parameter = &input.parameters[0];
    if parameter.owner != SourcePropertyImplementationId::new(0)
        || parameter.ordinal != 0
        || parameter.binding != BindingId::new(0)
        || parameter.written_type != SourceTypeApplicationId::new(0)
        || parameter.site != TypedSiteRef::Node(TypedNodeId::new(parameter_node))
        || parameter.source_range != range(input.source_id, 121, 145)
        || parameter.declaration_range != range(input.source_id, 125, 126)
        || parameter.context != BindingContextId::new(1)
        || parameter.recovery != SourcePropertyImplementationRecovery::Normal
        || parameter.spelling != "let M be Task264Carrier;"
    {
        return Err(SourcePropertyImplementationError::InvalidParameter { index: 0 });
    }
    validate_source_type_profile(profile, input.source_id, source_type)?;

    let target = &input.targets[0];
    let expected_target_site = TypedSiteRef::Role {
        node: TypedNodeId::new(owner_node),
        role: crate::typed_ast::TypeRole::new("source.property-implementation.target"),
    };
    if target.owner != SourcePropertyImplementationId::new(0)
        || target.ordinal != 0
        || target.subject != BindingId::new(0)
        || target.definition.index() != 2
        || target.contribution.index() != 0
        || target.site != expected_target_site
        || target.source_range != range(input.source_id, 157, 165)
        || target.subject_range != range(input.source_id, 157, 158)
        || target.name_range != range(input.source_id, 159, 165)
        || target.spelling != "M.marker"
        || target.return_type != SourceTypeStructureMemberId::new(1)
        || source_type
            .structure_members()
            .get(target.return_type)
            .is_none()
    {
        return Err(SourcePropertyImplementationError::InvalidTarget { index: 0 });
    }

    let definiens = &input.definientia[0];
    let expected_target = match profile {
        Profile::Means => {
            SourcePropertyDefiniensTarget::AtomicFormula(SourceAtomicFormulaId::new(0))
        }
        Profile::Equals => SourcePropertyDefiniensTarget::Structure(SourceStructureTermId::new(0)),
    };
    if definiens.owner != SourcePropertyImplementationId::new(0)
        || definiens.ordinal != 0
        || definiens.target != expected_target
        || definiens.site != TypedSiteRef::Node(TypedNodeId::new(definiens_node))
        || definiens.source_range != range(input.source_id, definiens_range.0, definiens_range.1)
        || definiens.context != BindingContextId::new(1)
        || definiens.recovery != SourcePropertyImplementationRecovery::Normal
        || definiens.spelling != definiens_spelling
    {
        return Err(SourcePropertyImplementationError::InvalidDefiniens { index: 0 });
    }
    validate_profile_lower(
        profile,
        input.source_id,
        source_term,
        applications,
        structures,
        set_terms,
        atomic_formulas,
    )?;
    validate_correctness(profile, input)?;
    validate_arena(profile, input.source_id, arena)?;
    Ok(())
}

fn validate_context(
    input: &SourcePropertyImplementationHandoffInput,
    context: &SourceBindingContextHandoff,
    owner_node: usize,
    end: usize,
    parameter_node: usize,
) -> Result<(), SourcePropertyImplementationError> {
    let env = context.binding_env();
    if context.items().len() != 1
        || context.declarations().len() != 1
        || context.context_links().len() != 2
        || context.local_contexts().len() != 2
        || env.contexts().len() != 2
        || env.bindings().len() != 1
        || !env.diagnostics().is_empty()
    {
        return Err(SourcePropertyImplementationError::DependencyMismatch);
    }
    let item = context
        .items()
        .get(SourceItemId::new(0))
        .ok_or(SourcePropertyImplementationError::DependencyMismatch)?;
    let declaration = context
        .declarations()
        .iter()
        .next()
        .map(|(_, row)| row)
        .ok_or(SourcePropertyImplementationError::DependencyMismatch)?;
    if item.shell.index() != 4
        || item.shell_ordinal != 4
        || item.role != SourceItemRole::PropertyImplementation
        || item.source_range != range(input.source_id, 108, end)
        || item.parent.is_some()
        || item.visibility != SourceItemVisibility::Unspecified
        || item.site != TypedSiteRef::Node(TypedNodeId::new(owner_node))
        || item
            .local_scope
            .as_ref()
            .is_none_or(|scope| scope.path() != [4])
        || item.recovery != SourceItemRecovery::Normal
        || item.binding_context != BindingContextId::new(1)
        || item.local_context != LocalTypeContextId::new(1)
        || item.predecessor.is_some()
        || declaration.item != SourceItemId::new(0)
        || declaration.binding != BindingId::new(0)
        || declaration.source_ordinal != 0
        || declaration.spelling != "M"
        || declaration.declaration_range != range(input.source_id, 125, 126)
        || declaration.written_type_range != range(input.source_id, 130, 144)
        || declaration.site != TypedSiteRef::Node(TypedNodeId::new(parameter_node))
        || declaration.binding_context != BindingContextId::new(1)
        || declaration.local_context != LocalTypeContextId::new(1)
        || declaration.shadowed_binding.is_some()
        || declaration.predecessor.is_some()
    {
        return Err(SourcePropertyImplementationError::DependencyMismatch);
    }
    Ok(())
}

fn validate_source_type_profile(
    profile: Profile,
    source: SourceId,
    handoff: &SourceTypeApplicationHandoff,
) -> Result<(), SourcePropertyImplementationError> {
    if handoff.applications().len() != 1
        || handoff.expressions().len() != 3
        || !handoff.arguments().is_empty()
        || !handoff.definition_returns().is_empty()
        || !handoff.mode_rhs().is_empty()
        || handoff.structure_members().len() != 2
    {
        return Err(SourcePropertyImplementationError::DependencyMismatch);
    }
    let application = handoff
        .applications()
        .get(SourceTypeApplicationId::new(0))
        .ok_or(SourcePropertyImplementationError::DependencyMismatch)?;
    if application.binding() != BindingId::new(0)
        || application.source_ordinal() != 0
        || application.root() != SourceTypeExpressionId::new(0)
    {
        return Err(SourcePropertyImplementationError::DependencyMismatch);
    }
    let (application_node, head_node, member_nodes) = match profile {
        Profile::Means => (63, 64, [56, 59]),
        Profile::Equals => (45, 46, [38, 41]),
    };
    for (index, (site, head_site, start, end)) in [
        (application_node, head_node, 130, 144),
        (member_nodes[0] - 1, member_nodes[0] - 2, 62, 65),
        (member_nodes[1] - 1, member_nodes[1] - 2, 90, 93),
    ]
    .into_iter()
    .enumerate()
    {
        let expression = handoff
            .expressions()
            .get(SourceTypeExpressionId::new(index))
            .ok_or(SourcePropertyImplementationError::DependencyMismatch)?;
        let head_matches = match (index, expression.head()) {
            (
                0,
                SourceTypeHead::Symbol {
                    symbol,
                    contribution,
                },
            ) => symbol.module() == handoff.module_id() && contribution.index() == 0,
            (1 | 2, SourceTypeHead::BuiltinSet) => true,
            _ => false,
        };
        let spelling = if index == 0 { "Task264Carrier" } else { "set" };
        if expression.source_id() != source
            || expression.site() != &TypedSiteRef::Node(TypedNodeId::new(site))
            || expression.source_range() != range(source, start, end)
            || expression.spelling() != spelling
            || expression.head_site() != &TypedSiteRef::Node(TypedNodeId::new(head_site))
            || expression.head_range() != range(source, start, end)
            || expression.head_spelling() != spelling
            || expression.form() != SourceTypeApplicationForm::Bare
            || expression.recovery() != NodeRecoveryState::Normal
            || !head_matches
        {
            return Err(SourcePropertyImplementationError::DependencyMismatch);
        }
    }
    for (index, (site, start, end)) in [(member_nodes[0], 45, 66), (member_nodes[1], 71, 94)]
        .into_iter()
        .enumerate()
    {
        let member = handoff
            .structure_members()
            .get(SourceTypeStructureMemberId::new(index))
            .ok_or(SourcePropertyImplementationError::DependencyMismatch)?;
        if member.member_site() != &TypedSiteRef::Node(TypedNodeId::new(site))
            || member.member_range() != range(source, start, end)
            || member.source_ordinal() != index
            || member.root() != SourceTypeExpressionId::new(index + 1)
        {
            return Err(SourcePropertyImplementationError::DependencyMismatch);
        }
    }
    Ok(())
}

fn validate_profile_lower(
    profile: Profile,
    source: SourceId,
    terms: &SourcePrimaryTermHandoff,
    applications: Option<&SourceFunctorApplicationHandoff>,
    structures: Option<&SourceStructureHandoff>,
    set_terms: Option<&SourceSetTermHandoff>,
    formulas: Option<&SourceAtomicFormulaHandoff>,
) -> Result<(), SourcePropertyImplementationError> {
    if applications.is_some() || set_terms.is_some() {
        return Err(SourcePropertyImplementationError::DependencyMismatch);
    }
    match profile {
        Profile::Means => {
            if structures.is_some()
                || formulas.is_none()
                || !terms.references().is_empty()
                || !terms.numeric_type_requests().is_empty()
            {
                return Err(SourcePropertyImplementationError::DependencyMismatch);
            }
            if terms.terms().len() != 2 {
                return Err(SourcePropertyImplementationError::InvalidDefiniens { index: 0 });
            }
            for (index, (node, start, end)) in
                [(66, 172, 174), (68, 177, 179)].into_iter().enumerate()
            {
                let term = terms
                    .terms()
                    .get(SourcePrimaryTermId::new(index))
                    .ok_or(SourcePropertyImplementationError::InvalidDefiniens { index: 0 })?;
                if term.kind() != SourcePrimaryTermKind::It
                    || term.role() != SourcePrimaryTermRole::CurrentDefinitionResult
                    || term.recovery() != SourcePrimaryTermRecovery::Normal
                    || term.parent().is_some()
                    || term.context() != BindingContextId::new(1)
                    || term.site() != &TypedSiteRef::Node(TypedNodeId::new(node))
                    || term.source_range() != range(source, start, end)
                    || term.spelling() != "it"
                {
                    return Err(SourcePropertyImplementationError::InvalidDefiniens { index: 0 });
                }
            }
            let formulas = formulas.expect("checked");
            let formula = formulas
                .formulas()
                .get(SourceAtomicFormulaId::new(0))
                .ok_or(SourcePropertyImplementationError::InvalidDefiniens { index: 0 })?;
            if formulas.formulas().len() != 1
                || !formulas.wrappers().is_empty()
                || !formulas.predicate_segments().is_empty()
                || !formulas.predicate_heads().is_empty()
                || !formulas.candidates().is_empty()
                || !formulas.type_sites().is_empty()
                || !formulas.attributes().is_empty()
                || formulas.edges().len() != 2
                || formulas.requests().len() != 2
                || formula.kind() != SourceAtomicFormulaKind::Equality
                || formula.site() != &TypedSiteRef::Node(TypedNodeId::new(70))
                || formula.source_range() != range(source, 172, 179)
                || formula.source_ordinal() != 0
                || formula.context() != BindingContextId::new(1)
                || formula.recovery()
                    != crate::source_atomic_formula::SourceAtomicFormulaRecovery::Normal
                || formula.spelling() != "it = it"
            {
                return Err(SourcePropertyImplementationError::InvalidDefiniens { index: 0 });
            }
            for (index, role) in [
                SourceAtomicEdgeRole::BuiltinLeftOperand,
                SourceAtomicEdgeRole::BuiltinRightOperand,
            ]
            .into_iter()
            .enumerate()
            {
                let edge = formulas
                    .edges()
                    .get(SourceAtomicEdgeId::new(index))
                    .ok_or(SourcePropertyImplementationError::InvalidDefiniens { index: 0 })?;
                if edge.formula() != SourceAtomicFormulaId::new(0)
                    || edge.ordinal() != index
                    || edge.role() != role
                    || edge.target()
                        != SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(index))
                {
                    return Err(SourcePropertyImplementationError::InvalidDefiniens { index: 0 });
                }
                let request = formulas
                    .requests()
                    .iter()
                    .nth(index)
                    .map(|(_, row)| row)
                    .ok_or(SourcePropertyImplementationError::InvalidDefiniens { index: 0 })?;
                if request.formula() != SourceAtomicFormulaId::new(0)
                    || request.ordinal() != index
                    || request.kind() != SourceAtomicRequestKind::OperandExpectedType
                    || request.edge() != Some(SourceAtomicEdgeId::new(index))
                    || request.candidate().is_some()
                    || request.type_site().is_some()
                    || request.attribute().is_some()
                {
                    return Err(SourcePropertyImplementationError::InvalidDefiniens { index: 0 });
                }
            }
        }
        Profile::Equals => {
            let Some(structures) = structures else {
                return Err(SourcePropertyImplementationError::DependencyMismatch);
            };
            if formulas.is_some()
                || terms.terms().len() != 1
                || terms.references().len() != 1
                || !terms.numeric_type_requests().is_empty()
                || structures.terms().len() != 1
                || !structures.wrappers().is_empty()
                || !structures.roots().is_empty()
                || structures.members().len() != 1
                || !structures.field_updates().is_empty()
                || structures.edges().len() != 1
                || structures.requests().len() != 3
            {
                return Err(SourcePropertyImplementationError::DependencyMismatch);
            }
            let term = terms
                .terms()
                .get(SourcePrimaryTermId::new(0))
                .ok_or(SourcePropertyImplementationError::InvalidDefiniens { index: 0 })?;
            let structure = structures
                .terms()
                .get(SourceStructureTermId::new(0))
                .ok_or(SourcePropertyImplementationError::InvalidDefiniens { index: 0 })?;
            let reference = terms
                .references()
                .iter()
                .next()
                .map(|(_, row)| row)
                .ok_or(SourcePropertyImplementationError::InvalidDefiniens { index: 0 })?;
            let member = structures
                .members()
                .get(SourceStructureMemberId::new(0))
                .ok_or(SourcePropertyImplementationError::InvalidDefiniens { index: 0 })?;
            let edge = structures
                .edges()
                .iter()
                .next()
                .map(|(_, row)| row)
                .ok_or(SourcePropertyImplementationError::InvalidDefiniens { index: 0 })?;
            if term.kind() != SourcePrimaryTermKind::VariableReference
                || term.role() != SourcePrimaryTermRole::Value
                || term.site() != &TypedSiteRef::Node(TypedNodeId::new(48))
                || term.source_range() != range(source, 173, 174)
                || term.context() != BindingContextId::new(1)
                || term.spelling() != "M"
                || reference.term() != SourcePrimaryTermId::new(0)
                || reference.binding() != BindingId::new(0)
                || reference.role() != SourcePrimaryTermReferenceRole::Variable
                || reference
                    .lexical_scope()
                    .is_none_or(|scope| scope.path() != [4])
                || reference.use_ordinal() != 1
                || structure.kind() != SourceStructureTermKind::SelectorAccess
                || structure.site() != &TypedSiteRef::Node(TypedNodeId::new(49))
                || structure.source_range() != range(source, 173, 182)
                || structure.source_ordinal() != 0
                || structure.context() != BindingContextId::new(1)
                || structure.recovery() != crate::source_structure::SourceStructureRecovery::Normal
                || structure.spelling() != "M.carrier"
                || member.term() != SourceStructureTermId::new(0)
                || member.ordinal() != 0
                || member.site() != &TypedSiteRef::Node(TypedNodeId::new(31))
                || member.source_range() != range(source, 175, 182)
                || member.spelling() != "carrier"
                || member.role() != SourceStructureMemberRole::Selector
                || member.parent().is_some()
                || edge.term() != SourceStructureTermId::new(0)
                || edge.ordinal() != 0
                || edge.role() != SourceStructureEdgeRole::SelectorBase
                || edge.member().is_some()
                || edge.target() != SourceStructureTarget::Primary(SourcePrimaryTermId::new(0))
            {
                return Err(SourcePropertyImplementationError::InvalidDefiniens { index: 0 });
            }
            for (index, (member, kind)) in [
                (
                    Some(SourceStructureMemberId::new(0)),
                    SourceStructureRequestKind::MemberIdentity,
                ),
                (
                    Some(SourceStructureMemberId::new(0)),
                    SourceStructureRequestKind::InheritancePath,
                ),
                (None, SourceStructureRequestKind::ResultType),
            ]
            .into_iter()
            .enumerate()
            {
                let request = structures
                    .requests()
                    .iter()
                    .nth(index)
                    .map(|(_, row)| row)
                    .ok_or(SourcePropertyImplementationError::InvalidDefiniens { index: 0 })?;
                if request.term() != SourceStructureTermId::new(0)
                    || request.member() != member
                    || request.request_ordinal() != index
                    || request.kind() != kind
                {
                    return Err(SourcePropertyImplementationError::InvalidDefiniens { index: 0 });
                }
            }
        }
    }
    Ok(())
}

fn validate_correctness(
    profile: Profile,
    input: &SourcePropertyImplementationHandoffInput,
) -> Result<(), SourcePropertyImplementationError> {
    match profile {
        Profile::Equals if input.correctness.is_empty() => Ok(()),
        Profile::Equals => Err(SourcePropertyImplementationError::InvalidCorrectness { index: 0 }),
        Profile::Means => {
            if input.correctness.len() != 2 {
                return Err(SourcePropertyImplementationError::InvalidCorrectness { index: 0 });
            }
            let rows = [
                (
                    SourcePropertyCorrectnessKind::Existence,
                    76,
                    183,
                    218,
                    193,
                    217,
                    "existence by computation(steps: 1);",
                ),
                (
                    SourcePropertyCorrectnessKind::Uniqueness,
                    80,
                    221,
                    257,
                    232,
                    256,
                    "uniqueness by computation(steps: 1);",
                ),
            ];
            for (index, (kind, node, start, end, proof_start, proof_end, spelling)) in
                rows.into_iter().enumerate()
            {
                let row = &input.correctness[index];
                if row.owner != SourcePropertyImplementationId::new(0)
                    || row.ordinal != index
                    || row.kind != kind
                    || row.site != TypedSiteRef::Node(TypedNodeId::new(node))
                    || row.source_range != range(input.source_id, start, end)
                    || row.justification
                        != SourceAnchor::Range(range(input.source_id, proof_start, proof_end))
                    || row.recovery != SourcePropertyImplementationRecovery::Normal
                    || row.spelling != spelling
                {
                    return Err(SourcePropertyImplementationError::InvalidCorrectness { index });
                }
            }
            Ok(())
        }
    }
}

fn validate_arena(
    profile: Profile,
    source: SourceId,
    arena: &TypedArena,
) -> Result<(), SourcePropertyImplementationError> {
    const MEANS_RANGES: &[[usize; 2]] = &[
        [0, 10],
        [13, 19],
        [20, 34],
        [35, 40],
        [45, 50],
        [51, 58],
        [59, 61],
        [62, 65],
        [65, 66],
        [71, 79],
        [80, 86],
        [87, 89],
        [90, 93],
        [93, 94],
        [97, 100],
        [100, 101],
        [102, 105],
        [105, 106],
        [108, 118],
        [121, 124],
        [125, 126],
        [127, 129],
        [130, 144],
        [144, 145],
        [148, 156],
        [157, 158],
        [158, 159],
        [159, 165],
        [166, 171],
        [172, 174],
        [175, 176],
        [177, 179],
        [179, 180],
        [183, 192],
        [193, 195],
        [196, 207],
        [207, 208],
        [208, 213],
        [213, 214],
        [215, 216],
        [216, 217],
        [217, 218],
        [221, 231],
        [232, 234],
        [235, 246],
        [246, 247],
        [247, 252],
        [252, 253],
        [254, 255],
        [255, 256],
        [256, 257],
        [258, 261],
        [261, 262],
        [20, 34],
        [62, 65],
        [62, 65],
        [45, 66],
        [90, 93],
        [90, 93],
        [71, 94],
        [13, 101],
        [0, 106],
        [130, 144],
        [130, 144],
        [130, 144],
        [121, 145],
        [172, 174],
        [172, 174],
        [177, 179],
        [177, 179],
        [172, 179],
        [172, 179],
        [172, 179],
        [208, 216],
        [196, 217],
        [193, 217],
        [183, 218],
        [247, 255],
        [235, 256],
        [232, 256],
        [221, 257],
        [108, 262],
        [0, 262],
        [0, 262],
        [0, 262],
    ];
    const EQUALS_RANGES: &[[usize; 2]] = &[
        [0, 10],
        [13, 19],
        [20, 34],
        [35, 40],
        [45, 50],
        [51, 58],
        [59, 61],
        [62, 65],
        [65, 66],
        [71, 79],
        [80, 86],
        [87, 89],
        [90, 93],
        [93, 94],
        [97, 100],
        [100, 101],
        [102, 105],
        [105, 106],
        [108, 118],
        [121, 124],
        [125, 126],
        [127, 129],
        [130, 144],
        [144, 145],
        [148, 156],
        [157, 158],
        [158, 159],
        [159, 165],
        [166, 172],
        [173, 174],
        [174, 175],
        [175, 182],
        [182, 183],
        [184, 187],
        [187, 188],
        [20, 34],
        [62, 65],
        [62, 65],
        [45, 66],
        [90, 93],
        [90, 93],
        [71, 94],
        [13, 101],
        [0, 106],
        [130, 144],
        [130, 144],
        [130, 144],
        [121, 145],
        [173, 174],
        [173, 182],
        [173, 182],
        [173, 182],
        [108, 188],
        [0, 188],
        [0, 188],
        [0, 188],
    ];
    let rows: &[(usize, usize, usize, &str)] = match profile {
        Profile::Means => &[
            (54, 62, 65, "source.type.head"),
            (55, 62, 65, "source.type.expression"),
            (57, 90, 93, "source.type.head"),
            (58, 90, 93, "source.type.expression"),
            (64, 130, 144, "source.type.head"),
            (63, 130, 144, "source.type.expression"),
            (56, 45, 66, "source.definition.structure.member"),
            (59, 71, 94, "source.definition.structure.member"),
            (60, 13, 101, "source.definition.structure"),
            (
                65,
                125,
                126,
                "source.definition.property-implementation.parameter",
            ),
            (66, 172, 174, "source.term.it"),
            (68, 177, 179, "source.term.it"),
            (70, 172, 179, "source.formula.atomic.equality"),
            (
                72,
                172,
                179,
                "source.definition.property-implementation.definiens",
            ),
            (
                76,
                183,
                218,
                "source.definition.property-implementation.correctness",
            ),
            (
                80,
                221,
                257,
                "source.definition.property-implementation.correctness",
            ),
            (81, 108, 262, "source.definition.property-implementation"),
            (84, 0, 262, "source.module"),
        ],
        Profile::Equals => &[
            (36, 62, 65, "source.type.head"),
            (37, 62, 65, "source.type.expression"),
            (39, 90, 93, "source.type.head"),
            (40, 90, 93, "source.type.expression"),
            (46, 130, 144, "source.type.head"),
            (45, 130, 144, "source.type.expression"),
            (38, 45, 66, "source.definition.structure.member"),
            (41, 71, 94, "source.definition.structure.member"),
            (42, 13, 101, "source.definition.structure"),
            (
                47,
                125,
                126,
                "source.definition.property-implementation.parameter",
            ),
            (31, 175, 182, "source.term.structure.member.selector"),
            (48, 173, 174, "source.term.variable-reference"),
            (49, 173, 182, "source.term.structure.selector"),
            (
                51,
                173,
                182,
                "source.definition.property-implementation.definiens",
            ),
            (52, 108, 188, "source.definition.property-implementation"),
            (55, 0, 188, "source.module"),
        ],
    };
    let expected_len = match profile {
        Profile::Means => 85,
        Profile::Equals => 56,
    };
    let exact_ranges = match profile {
        Profile::Means => MEANS_RANGES,
        Profile::Equals => EQUALS_RANGES,
    };
    if arena.len() != expected_len || arena.root() != Some(TypedNodeId::new(expected_len - 1)) {
        return Err(SourcePropertyImplementationError::InvalidArenaOwnership);
    }
    for (id, row) in arena.iter() {
        let owned = rows.iter().find(|(node, ..)| *node == id.index());
        let expected_kind = owned.map_or("source.surface.unowned", |(_, _, _, kind)| *kind);
        let expected_context = if id.index() == expected_len - 1 {
            LocalTypeContextId::new(0)
        } else {
            LocalTypeContextId::new(1)
        };
        let [start, end] = owned.map_or(exact_ranges[id.index()], |(_, start, end, _)| {
            [*start, *end]
        });
        let anchor_matches = row.anchor == SourceAnchor::Range(range(source, start, end));
        if row.kind.as_str() != expected_kind
            || !anchor_matches
            || row.resolved_node.is_some()
            || row.typing != TypingState::Unknown
            || row.recovery != NodeRecoveryState::Normal
            || row.links.context != Some(expected_context)
            || row.links.type_entry.is_some()
            || !row.links.facts.is_empty()
            || !row.links.coercions.is_empty()
            || !row.links.initial_obligations.is_empty()
            || !row.links.diagnostics.is_empty()
        {
            return Err(SourcePropertyImplementationError::InvalidArenaOwnership);
        }
    }
    Ok(())
}

// Rationale: replay validation authenticates the same complete lower bundle as construction.
#[allow(clippy::too_many_arguments)]
fn validate_handoff(
    handoff: &SourcePropertyImplementationHandoff,
    source_context: &SourceBindingContextHandoff,
    source_type: &SourceTypeApplicationHandoff,
    source_term: &SourcePrimaryTermHandoff,
    applications: Option<&SourceFunctorApplicationHandoff>,
    structures: Option<&SourceStructureHandoff>,
    set_terms: Option<&SourceSetTermHandoff>,
    atomic_formulas: Option<&SourceAtomicFormulaHandoff>,
    obligations: &InitialObligationTable,
    arena: &TypedArena,
) -> Result<(), SourcePropertyImplementationError> {
    if handoff.implementations.rows.len() != 1
        || handoff.parameters.rows.len() != 1
        || handoff.targets.rows.len() != 1
        || handoff.definientia.rows.len() != 1
        || !matches!(handoff.correctness.rows.len(), 0 | 2)
    {
        return Err(SourcePropertyImplementationError::UnsupportedTaskShape);
    }
    let input = SourcePropertyImplementationHandoffInput {
        source_id: handoff.source_id,
        module_id: handoff.module_id.clone(),
        implementations: handoff
            .implementations
            .rows
            .iter()
            .map(|row| SourcePropertyImplementationInput {
                shell: row.shell,
                site: row.site.clone(),
                source_range: row.source_range,
                source_ordinal: row.source_ordinal,
                context: row.context,
                recovery: row.recovery,
                spelling: row.spelling.clone(),
                style: row.style,
                parameter: row.parameter,
                target: row.target,
                definiens: row.definiens,
            })
            .collect(),
        parameters: handoff
            .parameters
            .rows
            .iter()
            .map(|row| SourcePropertyParameterInput {
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
        targets: handoff
            .targets
            .rows
            .iter()
            .map(|row| SourcePropertyTargetInput {
                owner: row.owner,
                ordinal: row.ordinal,
                subject: row.subject,
                symbol: row.symbol.clone(),
                definition: row.definition,
                contribution: row.contribution,
                site: row.site.clone(),
                source_range: row.source_range,
                subject_range: row.subject_range,
                name_range: row.name_range,
                spelling: row.spelling.clone(),
                return_type: row.return_type,
            })
            .collect(),
        definientia: handoff
            .definientia
            .rows
            .iter()
            .map(|row| SourcePropertyDefiniensInput {
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
            .rows
            .iter()
            .map(|row| SourcePropertyCorrectnessInput {
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
    validate_shape(&input)?;
    validate_input(
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
    validate_carrier_identity(handoff, source_type)?;
    validate_obligations(handoff, obligations)
}

fn validate_carrier_identity(
    handoff: &SourcePropertyImplementationHandoff,
    source_type: &SourceTypeApplicationHandoff,
) -> Result<(), SourcePropertyImplementationError> {
    let identity = &handoff.carrier_identity;
    let target = &handoff.targets.rows[0];
    let contribution = identity.structure.contribution;
    let distinct_symbols = identity.structure.symbol != identity.field.symbol
        && identity.structure.symbol != identity.property.symbol
        && identity.field.symbol != identity.property.symbol;
    let structure_head_matches = matches!(
        source_type
            .expressions()
            .get(SourceTypeExpressionId::new(0))
            .map(|row| row.head()),
        Some(SourceTypeHead::Symbol {
            symbol,
            contribution: head_contribution,
        }) if symbol == &identity.structure.symbol && *head_contribution == contribution
    );
    if identity.structure.definition.index() != 0
        || identity.field.definition.index() != 1
        || identity.property.definition.index() != 2
        || contribution.index() != 0
        || identity.field.contribution != contribution
        || identity.property.contribution != contribution
        || identity.structure.symbol.module() != &handoff.module_id
        || identity.field.symbol.module() != &handoff.module_id
        || identity.property.symbol.module() != &handoff.module_id
        || !distinct_symbols
        || !normal_origin(
            &identity.structure.origin,
            handoff.source_id,
            &handoff.module_id,
            range(handoff.source_id, 13, 101),
            &[4, 0, 11, 0],
        )
        || !normal_origin(
            &identity.field.origin,
            handoff.source_id,
            &handoff.module_id,
            range(handoff.source_id, 45, 66),
            &[4, 0, 11, 0, 18, 0],
        )
        || !normal_origin(
            &identity.property.origin,
            handoff.source_id,
            &handoff.module_id,
            range(handoff.source_id, 71, 94),
            &[4, 0, 11, 0, 19, 1],
        )
        || target.symbol != identity.property.symbol
        || target.definition != identity.property.definition
        || target.contribution != identity.property.contribution
        || target.origin != identity.property.origin
        || !structure_head_matches
    {
        return Err(SourcePropertyImplementationError::InvalidResolverTarget { index: 0 });
    }
    Ok(())
}

fn validate_obligations(
    handoff: &SourcePropertyImplementationHandoff,
    table: &InitialObligationTable,
) -> Result<(), SourcePropertyImplementationError> {
    let appended = handoff.correctness.rows.len();
    let base_len = handoff
        .correctness
        .rows
        .first()
        .map_or(table.len(), |row| row.obligation.index());
    if table.len() != base_len + appended
        || table
            .iter()
            .take(base_len)
            .any(|(_, row)| property_family_kind(row.kind))
    {
        return Err(SourcePropertyImplementationError::InvalidObligation);
    }
    let kinds = [
        InitialObligationKind::PropertyImplementationExistence,
        InitialObligationKind::PropertyImplementationUniqueness,
    ];
    let goals = [
        "source.definition.property-implementation.correctness:implementation=0:existence",
        "source.definition.property-implementation.correctness:implementation=0:uniqueness",
    ];
    let provenances = [
        "source.definition.property-implementation:implementation=0:correctness=0",
        "source.definition.property-implementation:implementation=0:correctness=1",
    ];
    for index in 0..appended {
        let row = &handoff.correctness.rows[index];
        let id = InitialObligationId::new(base_len + index);
        let obligation = table
            .get(id)
            .ok_or(SourcePropertyImplementationError::InvalidObligation)?;
        if row.obligation != id
            || obligation.id != id
            || obligation.kind != kinds[index]
            || obligation.owner != row.site
            || obligation.source_range != row.source_range
            || !obligation.assumptions.is_empty()
            || obligation.goal.as_str() != goals[index]
            || obligation.provenance.as_str() != provenances[index]
            || obligation.status != InitialObligationStatus::Pending
        {
            return Err(SourcePropertyImplementationError::InvalidObligation);
        }
    }
    Ok(())
}

const fn property_family_kind(kind: InitialObligationKind) -> bool {
    matches!(
        kind,
        InitialObligationKind::PredicatePropertyCorrectness
            | InitialObligationKind::FunctorExistence
            | InitialObligationKind::FunctorUniqueness
            | InitialObligationKind::PropertyImplementationExistence
            | InitialObligationKind::PropertyImplementationUniqueness
    )
}

fn normal_origin(
    origin: &SemanticOrigin,
    source: SourceId,
    module: &ModuleId,
    source_range: SourceRange,
    path: &[u32],
) -> bool {
    origin.source_id() == source
        && origin.module_id() == module
        && origin.anchor() == &SourceAnchor::Range(source_range)
        && origin.structural_path() == path
        && origin.import_edge().is_none()
        && !origin.is_recovered()
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

const fn style_key(style: SourcePropertyImplementationStyle) -> &'static str {
    match style {
        SourcePropertyImplementationStyle::Equals => "equals",
        SourcePropertyImplementationStyle::Means => "means",
    }
}
const fn correctness_key(kind: SourcePropertyCorrectnessKind) -> &'static str {
    match kind {
        SourcePropertyCorrectnessKind::Existence => "existence",
        SourcePropertyCorrectnessKind::Uniqueness => "uniqueness",
    }
}
const fn recovery_key(recovery: SourcePropertyImplementationRecovery) -> &'static str {
    match recovery {
        SourcePropertyImplementationRecovery::Normal => "normal",
        SourcePropertyImplementationRecovery::Degraded => "degraded",
    }
}

fn write_optional_fingerprint(output: &mut String, label: &str, value: Option<&str>) {
    let _ = write!(output, "{label}: ");
    match value {
        Some(value) => {
            let _ = write!(output, "some({value:?})");
        }
        None => output.push_str("none"),
    }
    output.push('\n');
}

fn write_target(output: &mut String, target: SourcePropertyDefiniensTarget) {
    match target {
        SourcePropertyDefiniensTarget::Primary(id) => {
            let _ = write!(output, "primary#{}", id.index());
        }
        SourcePropertyDefiniensTarget::Application(id) => {
            let _ = write!(output, "application#{}", id.index());
        }
        SourcePropertyDefiniensTarget::Structure(id) => {
            let _ = write!(output, "structure#{}", id.index());
        }
        SourcePropertyDefiniensTarget::SetTerm(id) => {
            let _ = write!(output, "set-term#{}", id.index());
        }
        SourcePropertyDefiniensTarget::AtomicFormula(id) => {
            let _ = write!(output, "atomic-formula#{}", id.index());
        }
    }
}

fn write_site(output: &mut String, site: &TypedSiteRef) {
    match site {
        TypedSiteRef::Node(node) => {
            let _ = write!(output, "node#{}", node.index());
        }
        TypedSiteRef::Role { node, role } => {
            let _ = write!(output, "role#{}:{}", node.index(), role.as_str());
        }
    }
}

fn write_anchor(output: &mut String, anchor: &SourceAnchor) {
    match anchor {
        SourceAnchor::Range(row) => {
            let _ = write!(output, "range:{}..{}", row.start, row.end);
        }
        SourceAnchor::Point { offset, .. } => {
            let _ = write!(output, "point:{offset}");
        }
        SourceAnchor::Generated(_) => output.push_str("generated"),
        _ => output.push_str("unsupported"),
    }
}

fn write_carrier_identity(
    output: &mut String,
    index: usize,
    role: &str,
    identity: &SourcePropertyResolverIdentity,
) {
    let _ = write!(
        output,
        "carrier-identity#{index} role={role} symbol={:?} definition={} contribution={} origin_range=",
        identity.symbol.fqn().as_str(),
        identity.definition.index(),
        identity.contribution.index(),
    );
    write_anchor_range(output, identity.origin.anchor());
    let _ = writeln!(
        output,
        " origin_path={:?}",
        identity.origin.structural_path()
    );
}

fn write_anchor_range(output: &mut String, anchor: &SourceAnchor) {
    match anchor {
        SourceAnchor::Range(row) => {
            let _ = write!(output, "{}..{}", row.start, row.end);
        }
        SourceAnchor::Point { offset, .. } => {
            let _ = write!(output, "{offset}..{offset}");
        }
        SourceAnchor::Generated(_) => output.push_str("generated"),
        _ => output.push_str("unsupported"),
    }
}

#[cfg(test)]
#[path = "../tests/support/source_property_implementation_unit.rs"]
pub(crate) mod tests;
