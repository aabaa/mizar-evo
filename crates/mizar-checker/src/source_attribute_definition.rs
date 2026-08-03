//! Syntax-free attribute-definition intake for checker phase 6.

use crate::{
    binding_env::{
        BindingContextId, BindingId, BindingKind, BindingRecoveryState, BindingStatus,
        BindingTypeSite,
    },
    source_atomic_formula::{
        SourceAtomicEdgeId, SourceAtomicEdgeRole, SourceAtomicFormulaHandoff,
        SourceAtomicFormulaId, SourceAtomicFormulaKind, SourceAtomicFormulaRecovery,
        SourceAtomicRequestId, SourceAtomicRequestKind, SourceAtomicTermTarget,
    },
    source_context::{
        SourceBindingContextHandoff, SourceBindingSiteRole, SourceDeclarationId, SourceItemId,
        SourceItemRecovery, SourceItemRole, SourceItemVisibility,
    },
    source_term::{
        SourcePrimaryTermHandoff, SourcePrimaryTermId, SourcePrimaryTermKind,
        SourcePrimaryTermRecovery, SourcePrimaryTermReferenceId, SourcePrimaryTermReferenceRole,
        SourcePrimaryTermRole,
    },
    source_type::{
        SourceTypeApplicationForm, SourceTypeApplicationHandoff, SourceTypeApplicationId,
        SourceTypeExpressionId, SourceTypeHead,
    },
    typed_ast::{LocalTypeContextId, NodeRecoveryState, TypedArena, TypedNodeId, TypedSiteRef},
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

dense_id!(SourceAttributeDefinitionId);
dense_id!(SourceAttributeParameterId);
dense_id!(SourceAttributeSubjectId);
dense_id!(SourceAttributeDefiniensId);

/// Complete syntax-free input for one ordinary attribute-definition transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceAttributeDefinitionHandoffInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub definitions: Vec<SourceAttributeDefinitionInput>,
    pub parameters: Vec<SourceAttributeParameterInput>,
    pub subjects: Vec<SourceAttributeSubjectInput>,
    pub definientia: Vec<SourceAttributeDefiniensInput>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceAttributeDefinitionInput {
    pub symbol: SymbolId,
    pub definition: DefinitionId,
    pub contribution: SourceContributionId,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub source_ordinal: usize,
    pub context: BindingContextId,
    pub recovery: SourceAttributeDefinitionRecovery,
    pub spelling: String,
    pub subject: SourceAttributeSubjectId,
    pub definiens: SourceAttributeDefiniensId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceAttributeParameterInput {
    pub owner: SourceAttributeDefinitionId,
    pub ordinal: usize,
    pub binding: BindingId,
    pub written_type: SourceTypeApplicationId,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub declaration_range: SourceRange,
    pub context: BindingContextId,
    pub recovery: SourceAttributeDefinitionRecovery,
    pub spelling: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceAttributeSubjectInput {
    pub owner: SourceAttributeDefinitionId,
    pub binding: BindingId,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub context: BindingContextId,
    pub recovery: SourceAttributeDefinitionRecovery,
    pub spelling: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceAttributeDefiniensInput {
    pub owner: SourceAttributeDefinitionId,
    pub ordinal: usize,
    pub formula: SourceAtomicFormulaId,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub context: BindingContextId,
    pub recovery: SourceAttributeDefinitionRecovery,
    pub spelling: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceAttributeDefinitionRecovery {
    Normal,
    Degraded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceAttributeDefinition {
    id: SourceAttributeDefinitionId,
    symbol: SymbolId,
    definition: DefinitionId,
    contribution: SourceContributionId,
    site: TypedSiteRef,
    source_range: SourceRange,
    source_ordinal: usize,
    context: BindingContextId,
    recovery: SourceAttributeDefinitionRecovery,
    spelling: String,
    subject: SourceAttributeSubjectId,
    definiens: SourceAttributeDefiniensId,
    origin: SemanticOrigin,
}

impl SourceAttributeDefinition {
    pub const fn id(&self) -> SourceAttributeDefinitionId {
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
    pub const fn recovery(&self) -> SourceAttributeDefinitionRecovery {
        self.recovery
    }
    pub fn spelling(&self) -> &str {
        &self.spelling
    }
    pub const fn subject(&self) -> SourceAttributeSubjectId {
        self.subject
    }
    pub const fn definiens(&self) -> SourceAttributeDefiniensId {
        self.definiens
    }
    pub const fn origin(&self) -> &SemanticOrigin {
        &self.origin
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceAttributeParameter {
    id: SourceAttributeParameterId,
    owner: SourceAttributeDefinitionId,
    ordinal: usize,
    binding: BindingId,
    written_type: SourceTypeApplicationId,
    site: TypedSiteRef,
    source_range: SourceRange,
    declaration_range: SourceRange,
    context: BindingContextId,
    recovery: SourceAttributeDefinitionRecovery,
    spelling: String,
}

impl SourceAttributeParameter {
    pub const fn id(&self) -> SourceAttributeParameterId {
        self.id
    }
    pub const fn owner(&self) -> SourceAttributeDefinitionId {
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
    pub const fn recovery(&self) -> SourceAttributeDefinitionRecovery {
        self.recovery
    }
    pub fn spelling(&self) -> &str {
        &self.spelling
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceAttributeSubject {
    id: SourceAttributeSubjectId,
    owner: SourceAttributeDefinitionId,
    binding: BindingId,
    site: TypedSiteRef,
    source_range: SourceRange,
    context: BindingContextId,
    recovery: SourceAttributeDefinitionRecovery,
    spelling: String,
}

impl SourceAttributeSubject {
    pub const fn id(&self) -> SourceAttributeSubjectId {
        self.id
    }
    pub const fn owner(&self) -> SourceAttributeDefinitionId {
        self.owner
    }
    pub const fn binding(&self) -> BindingId {
        self.binding
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
    pub const fn recovery(&self) -> SourceAttributeDefinitionRecovery {
        self.recovery
    }
    pub fn spelling(&self) -> &str {
        &self.spelling
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceAttributeDefiniens {
    id: SourceAttributeDefiniensId,
    owner: SourceAttributeDefinitionId,
    ordinal: usize,
    formula: SourceAtomicFormulaId,
    site: TypedSiteRef,
    source_range: SourceRange,
    context: BindingContextId,
    recovery: SourceAttributeDefinitionRecovery,
    spelling: String,
}

impl SourceAttributeDefiniens {
    pub const fn id(&self) -> SourceAttributeDefiniensId {
        self.id
    }
    pub const fn owner(&self) -> SourceAttributeDefinitionId {
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
    pub const fn recovery(&self) -> SourceAttributeDefinitionRecovery {
        self.recovery
    }
    pub fn spelling(&self) -> &str {
        &self.spelling
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
    SourceAttributeDefinitionTable,
    SourceAttributeDefinition,
    SourceAttributeDefinitionId
);
table!(
    SourceAttributeParameterTable,
    SourceAttributeParameter,
    SourceAttributeParameterId
);
table!(
    SourceAttributeSubjectTable,
    SourceAttributeSubject,
    SourceAttributeSubjectId
);
table!(
    SourceAttributeDefiniensTable,
    SourceAttributeDefiniens,
    SourceAttributeDefiniensId
);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceAttributeDefinitionHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    resolver_identity: SourceAttributeResolverIdentity,
    source_context_fingerprint: String,
    source_type_fingerprint: String,
    source_term_fingerprint: String,
    source_atomic_formula_fingerprint: String,
    definitions: SourceAttributeDefinitionTable,
    parameters: SourceAttributeParameterTable,
    subjects: SourceAttributeSubjectTable,
    definientia: SourceAttributeDefiniensTable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SourceAttributeResolverIdentity {
    symbol: SymbolId,
    definition: DefinitionId,
    contribution: SourceContributionId,
    origin: SemanticOrigin,
}

impl SourceAttributeDefinitionHandoff {
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
    pub const fn definitions(&self) -> &SourceAttributeDefinitionTable {
        &self.definitions
    }
    pub const fn parameters(&self) -> &SourceAttributeParameterTable {
        &self.parameters
    }
    pub const fn subjects(&self) -> &SourceAttributeSubjectTable {
        &self.subjects
    }
    pub const fn definientia(&self) -> &SourceAttributeDefiniensTable {
        &self.definientia
    }

    pub fn debug_text(&self) -> String {
        let mut output = String::from("source-attribute-definition-debug-v1\n");
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
                recovery_key(row.recovery)
            );
            write_anchor_range(&mut output, row.origin.anchor());
            let _ = writeln!(
                output,
                " origin_path={:?} spelling={:?} subject={} definiens={}",
                row.origin.structural_path(),
                row.spelling,
                row.subject.index(),
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
                row.spelling
            );
        }
        for (id, row) in self.subjects.iter() {
            let _ = write!(
                output,
                "subject#{} owner={} binding={} range={}..{} site=",
                id.index(),
                row.owner.index(),
                row.binding.index(),
                row.source_range.start,
                row.source_range.end,
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
        for (id, row) in self.definientia.iter() {
            let _ = write!(
                output,
                "definiens#{} owner={} ordinal={} formula={} range={}..{} site=",
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
                row.spelling
            );
        }
        output
    }

    // Rationale: installation revalidates the one frozen handoff against all four lower owners.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn validate_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        source_context: &SourceBindingContextHandoff,
        source_type: &SourceTypeApplicationHandoff,
        source_term: &SourcePrimaryTermHandoff,
        source_atomic_formula: &SourceAtomicFormulaHandoff,
        arena: &TypedArena,
    ) -> Result<(), SourceAttributeDefinitionError> {
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
            return Err(SourceAttributeDefinitionError::SourceIdentityMismatch);
        }
        if self.source_context_fingerprint != source_context.debug_text()
            || self.source_type_fingerprint != source_type.debug_text()
            || self.source_term_fingerprint != source_term.debug_text()
            || self.source_atomic_formula_fingerprint != source_atomic_formula.debug_text()
        {
            return Err(SourceAttributeDefinitionError::DependencyMismatch);
        }
        validate_handoff_rows(
            self,
            source_context,
            source_type,
            source_term,
            source_atomic_formula,
            arena,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceAttributeDefinitionError {
    SourceIdentityMismatch,
    DependencyMismatch,
    InvalidResolverDefinition { index: usize },
    InvalidDefinition { index: usize },
    InvalidParameter { index: usize },
    InvalidSubject { index: usize },
    InvalidDefiniens { index: usize },
    InvalidArenaOwnership,
    UnsupportedTaskShape,
}

impl fmt::Display for SourceAttributeDefinitionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SourceIdentityMismatch => {
                formatter.write_str("attribute-definition source identity mismatch")
            }
            Self::DependencyMismatch => {
                formatter.write_str("attribute-definition dependency mismatch")
            }
            Self::InvalidResolverDefinition { index } => {
                write!(formatter, "invalid attribute resolver definition {index}")
            }
            Self::InvalidDefinition { index } => {
                write!(formatter, "invalid source attribute definition {index}")
            }
            Self::InvalidParameter { index } => {
                write!(formatter, "invalid source attribute parameter {index}")
            }
            Self::InvalidSubject { index } => {
                write!(formatter, "invalid source attribute subject {index}")
            }
            Self::InvalidDefiniens { index } => {
                write!(formatter, "invalid source attribute definiens {index}")
            }
            Self::InvalidArenaOwnership => {
                formatter.write_str("invalid attribute-definition typed-arena ownership")
            }
            Self::UnsupportedTaskShape => {
                formatter.write_str("unsupported attribute-definition task shape")
            }
        }
    }
}

impl Error for SourceAttributeDefinitionError {}

pub struct SourceAttributeDefinitionProducer;

impl SourceAttributeDefinitionProducer {
    // Rationale: the exact producer authenticates resolver state and four independent lower owners.
    #[allow(clippy::too_many_arguments)]
    pub fn build(
        input: SourceAttributeDefinitionHandoffInput,
        env: &SymbolEnv,
        source_context: &SourceBindingContextHandoff,
        source_type: &SourceTypeApplicationHandoff,
        source_term: &SourcePrimaryTermHandoff,
        source_atomic_formula: &SourceAtomicFormulaHandoff,
        arena: &TypedArena,
    ) -> Result<SourceAttributeDefinitionHandoff, SourceAttributeDefinitionError> {
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
            return Err(SourceAttributeDefinitionError::SourceIdentityMismatch);
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

        let source_id = input.source_id;
        let module_id = input.module_id;
        let definitions = SourceAttributeDefinitionTable {
            rows: input
                .definitions
                .into_iter()
                .enumerate()
                .map(|(index, row)| SourceAttributeDefinition {
                    id: SourceAttributeDefinitionId::new(index),
                    symbol: row.symbol,
                    definition: row.definition,
                    contribution: row.contribution,
                    site: row.site,
                    source_range: row.source_range,
                    source_ordinal: row.source_ordinal,
                    context: row.context,
                    recovery: row.recovery,
                    spelling: row.spelling,
                    subject: row.subject,
                    definiens: row.definiens,
                    origin: origin.clone(),
                })
                .collect(),
        };
        let parameters = SourceAttributeParameterTable {
            rows: input
                .parameters
                .into_iter()
                .enumerate()
                .map(|(index, row)| SourceAttributeParameter {
                    id: SourceAttributeParameterId::new(index),
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
        let subjects = SourceAttributeSubjectTable {
            rows: input
                .subjects
                .into_iter()
                .enumerate()
                .map(|(index, row)| SourceAttributeSubject {
                    id: SourceAttributeSubjectId::new(index),
                    owner: row.owner,
                    binding: row.binding,
                    site: row.site,
                    source_range: row.source_range,
                    context: row.context,
                    recovery: row.recovery,
                    spelling: row.spelling,
                })
                .collect(),
        };
        let definientia = SourceAttributeDefiniensTable {
            rows: input
                .definientia
                .into_iter()
                .enumerate()
                .map(|(index, row)| SourceAttributeDefiniens {
                    id: SourceAttributeDefiniensId::new(index),
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
        let handoff = SourceAttributeDefinitionHandoff {
            source_id,
            module_id,
            resolver_identity: SourceAttributeResolverIdentity {
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
            subjects,
            definientia,
        };
        validate_handoff_rows(
            &handoff,
            source_context,
            source_type,
            source_term,
            source_atomic_formula,
            arena,
        )?;
        Ok(handoff)
    }
}

// Rationale: dependency identity is a single atomic comparison across all frozen lower handoffs.
#[allow(clippy::too_many_arguments)]
fn validate_dependency_identity(
    source_id: SourceId,
    module_id: &ModuleId,
    source_context: &SourceBindingContextHandoff,
    source_type: &SourceTypeApplicationHandoff,
    source_term: &SourcePrimaryTermHandoff,
    source_atomic_formula: &SourceAtomicFormulaHandoff,
    arena: &TypedArena,
) -> Result<(), SourceAttributeDefinitionError> {
    if source_context.source_id() != source_id
        || source_context.module_id() != module_id
        || source_type.source_id() != source_id
        || source_type.module_id() != module_id
        || source_term.source_id() != source_id
        || source_term.module_id() != module_id
        || source_atomic_formula.source_id() != source_id
        || source_atomic_formula.module_id() != module_id
    {
        return Err(SourceAttributeDefinitionError::SourceIdentityMismatch);
    }
    source_type
        .validate_installation(source_id, module_id, arena)
        .map_err(|_| SourceAttributeDefinitionError::DependencyMismatch)?;
    source_term
        .validate_installation(source_id, module_id, arena)
        .map_err(|_| SourceAttributeDefinitionError::DependencyMismatch)?;
    source_atomic_formula
        .validate_installation(source_id, module_id, source_term, None, None, None, arena)
        .map_err(|_| SourceAttributeDefinitionError::DependencyMismatch)?;
    Ok(())
}

fn validate_input_shape(
    input: &SourceAttributeDefinitionHandoffInput,
) -> Result<(), SourceAttributeDefinitionError> {
    if input.definitions.len() != 1
        || input.parameters.len() != 2
        || input.subjects.len() != 1
        || input.definientia.len() != 1
    {
        return Err(SourceAttributeDefinitionError::UnsupportedTaskShape);
    }
    Ok(())
}

fn validate_resolver_definition(
    input: &SourceAttributeDefinitionHandoffInput,
    env: &SymbolEnv,
) -> Result<SemanticOrigin, SourceAttributeDefinitionError> {
    if env.symbols().len() != 1 || env.definitions().len() != 1 || env.contributions().len() != 1 {
        return Err(SourceAttributeDefinitionError::InvalidResolverDefinition { index: 0 });
    }
    let row = &input.definitions[0];
    let symbol = env
        .symbols()
        .get(&row.symbol)
        .ok_or(SourceAttributeDefinitionError::InvalidResolverDefinition { index: 0 })?;
    let definition = env
        .definitions()
        .get(row.definition)
        .ok_or(SourceAttributeDefinitionError::InvalidResolverDefinition { index: 0 })?;
    let contribution = env
        .contributions()
        .get(row.contribution)
        .ok_or(SourceAttributeDefinitionError::InvalidResolverDefinition { index: 0 })?;
    let expected_anchor = SourceAnchor::Range(range(input.source_id, 0, 115));
    if row.definition.index() != 0
        || row.contribution.index() != 0
        || row.symbol.module() != &input.module_id
        || symbol.symbol() != &row.symbol
        || symbol.kind() != SymbolKind::Attribute
        || symbol.visibility() != Visibility::Public
        || symbol.export_status() != ExportStatus::Exported
        || symbol.primary_spelling() != "task261_marked"
        || symbol.notation_spelling() != Some("task261_marked")
        || symbol.contribution() != row.contribution
        || symbol.origin() != definition.origin()
        || symbol.signature() != definition.signature()
        || !symbol.relations().is_empty()
        || definition.id() != row.definition
        || definition.symbol() != &row.symbol
        || definition.kind() != DefinitionKind::Attribute
        || definition.visibility() != Visibility::Public
        || !definition.parameters().is_empty()
        || !definition.binders().is_empty()
        || definition.arity().is_some()
        || definition.notation_shape() != Some("task261_marked")
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
        || contribution.effects().symbols().len() != 1
        || contribution.effects().definitions().len() != 1
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
            &[4, 0, 7, 0],
        )
    {
        return Err(SourceAttributeDefinitionError::InvalidResolverDefinition { index: 0 });
    }
    Ok(definition.origin().clone())
}

// Rationale: exact row validation must keep the four lower-owner inputs explicit and syntax-free.
#[allow(clippy::too_many_arguments)]
fn validate_input_rows(
    input: &SourceAttributeDefinitionHandoffInput,
    source_context: &SourceBindingContextHandoff,
    source_type: &SourceTypeApplicationHandoff,
    source_term: &SourcePrimaryTermHandoff,
    source_atomic_formula: &SourceAtomicFormulaHandoff,
    arena: &TypedArena,
) -> Result<(), SourceAttributeDefinitionError> {
    let source = input.source_id;
    let context = BindingContextId::new(1);
    let local_context = validate_context_profile(input, source_context, arena)?;
    let definition = &input.definitions[0];
    if definition.site != TypedSiteRef::Node(TypedNodeId::new(40))
        || definition.source_range != range(source, 45, 110)
        || definition.source_ordinal != 0
        || definition.context != context
        || definition.recovery != SourceAttributeDefinitionRecovery::Normal
        || definition.spelling
            != "attr Task261AttributeDefinition: x is task261_marked means x = y;"
        || definition.subject != SourceAttributeSubjectId::new(0)
        || definition.definiens != SourceAttributeDefiniensId::new(0)
    {
        return Err(SourceAttributeDefinitionError::InvalidDefinition { index: 0 });
    }
    if !valid_site(
        &definition.site,
        definition.source_range,
        "source.definition.attribute",
        local_context,
        arena,
    ) {
        return Err(SourceAttributeDefinitionError::InvalidArenaOwnership);
    }
    let parameter_ranges = [(13, 26, 17, 18), (29, 42, 33, 34)];
    let parameter_sites = [27, 31];
    let parameter_spellings = ["let x be set;", "let y be set;"];
    for (index, row) in input.parameters.iter().enumerate() {
        let (start, end, declaration_start, declaration_end) = parameter_ranges[index];
        if row.owner != SourceAttributeDefinitionId::new(0)
            || row.ordinal != index
            || row.binding != BindingId::new(index)
            || row.written_type != SourceTypeApplicationId::new(index)
            || row.site != TypedSiteRef::Node(TypedNodeId::new(parameter_sites[index]))
            || row.source_range != range(source, start, end)
            || row.declaration_range != range(source, declaration_start, declaration_end)
            || row.context != context
            || row.recovery != SourceAttributeDefinitionRecovery::Normal
            || row.spelling != parameter_spellings[index]
        {
            return Err(SourceAttributeDefinitionError::InvalidParameter { index });
        }
        if !valid_site(
            &row.site,
            row.declaration_range,
            "source.definition.attribute.parameter",
            local_context,
            arena,
        ) {
            return Err(SourceAttributeDefinitionError::InvalidArenaOwnership);
        }
    }
    validate_type_profile(input, source_type, local_context, arena)?;
    validate_term_and_atomic_profile(input, source_term, source_atomic_formula)?;

    let subject = &input.subjects[0];
    if subject.owner != SourceAttributeDefinitionId::new(0)
        || subject.binding != BindingId::new(0)
        || subject.site != definition.site
        || subject.source_range != range(source, 78, 79)
        || subject.context != context
        || subject.recovery != SourceAttributeDefinitionRecovery::Normal
        || subject.spelling != "x"
    {
        return Err(SourceAttributeDefinitionError::InvalidSubject { index: 0 });
    }
    // The subject is a token field of the definition rather than a Surface row. It therefore
    // authenticates the shared definition site while preserving its own token range.
    if !valid_site(
        &subject.site,
        definition.source_range,
        "source.definition.attribute",
        local_context,
        arena,
    ) {
        return Err(SourceAttributeDefinitionError::InvalidArenaOwnership);
    }

    let definiens = &input.definientia[0];
    if definiens.owner != SourceAttributeDefinitionId::new(0)
        || definiens.ordinal != 0
        || definiens.formula != SourceAtomicFormulaId::new(0)
        || definiens.site != TypedSiteRef::Node(TypedNodeId::new(39))
        || definiens.source_range != range(source, 104, 109)
        || definiens.context != context
        || definiens.recovery != SourceAttributeDefinitionRecovery::Normal
        || definiens.spelling != "x = y"
    {
        return Err(SourceAttributeDefinitionError::InvalidDefiniens { index: 0 });
    }
    if !valid_site(
        &definiens.site,
        definiens.source_range,
        "source.definition.attribute.definiens",
        local_context,
        arena,
    ) {
        return Err(SourceAttributeDefinitionError::InvalidArenaOwnership);
    }
    Ok(())
}

fn validate_context_profile(
    input: &SourceAttributeDefinitionHandoffInput,
    source_context: &SourceBindingContextHandoff,
    arena: &TypedArena,
) -> Result<LocalTypeContextId, SourceAttributeDefinitionError> {
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
        return Err(SourceAttributeDefinitionError::DependencyMismatch);
    }
    let item = source_context
        .items()
        .get(SourceItemId::new(0))
        .ok_or(SourceAttributeDefinitionError::DependencyMismatch)?;
    let module_context = BindingContextId::new(0);
    let module_local = LocalTypeContextId::new(0);
    let module_link = source_context
        .context_links()
        .get(module_context)
        .ok_or(SourceAttributeDefinitionError::DependencyMismatch)?;
    let definition_link = source_context
        .context_links()
        .get(definition_context)
        .ok_or(SourceAttributeDefinitionError::DependencyMismatch)?;
    let definition_local = definition_link.local_context;
    if item.id != SourceItemId::new(0)
        || item.shell.index() != 0
        || item.shell_ordinal != 0
        || item.role != SourceItemRole::DefinitionBlock
        || item.source_range != range(source, 0, 115)
        || item.parent.is_some()
        || item.visibility != SourceItemVisibility::Unspecified
        || item.site != TypedSiteRef::Node(TypedNodeId::new(41))
        || item.local_scope.is_none()
        || item.recovery != SourceItemRecovery::Normal
        || item.binding_context != definition_context
        || definition_context != BindingContextId::new(1)
        || definition_local != LocalTypeContextId::new(1)
        || item.local_context != definition_local
        || item.predecessor.is_some()
        || module_link.binding_context != module_context
        || module_link.local_context != module_local
        || module_link.item.is_some()
        || definition_link.binding_context != definition_context
        || definition_link.local_context != item.local_context
        || definition_link.item != Some(item.id)
    {
        return Err(SourceAttributeDefinitionError::DependencyMismatch);
    }
    if !valid_site(
        &item.site,
        item.source_range,
        "source.definition",
        definition_local,
        arena,
    ) {
        return Err(SourceAttributeDefinitionError::InvalidArenaOwnership);
    }
    for (index, row) in input.parameters.iter().enumerate() {
        let declaration = source_context
            .declarations()
            .get(SourceDeclarationId::new(index))
            .ok_or(SourceAttributeDefinitionError::InvalidParameter { index })?;
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
            return Err(SourceAttributeDefinitionError::InvalidParameter { index });
        }
        let binding = binding_env
            .bindings()
            .get(row.binding)
            .ok_or(SourceAttributeDefinitionError::InvalidParameter { index })?;
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
            return Err(SourceAttributeDefinitionError::InvalidParameter { index });
        }
    }
    Ok(definition_local)
}

fn validate_type_profile(
    input: &SourceAttributeDefinitionHandoffInput,
    source_type: &SourceTypeApplicationHandoff,
    local_context: LocalTypeContextId,
    arena: &TypedArena,
) -> Result<(), SourceAttributeDefinitionError> {
    if source_type.applications().len() != 2
        || source_type.expressions().len() != 2
        || !source_type.arguments().is_empty()
    {
        return Err(SourceAttributeDefinitionError::DependencyMismatch);
    }
    let expression_sites = [25, 29];
    let head_sites = [24, 28];
    let ranges = [(22, 25), (38, 41)];
    for index in 0..2 {
        let application = source_type
            .applications()
            .get(SourceTypeApplicationId::new(index))
            .ok_or(SourceAttributeDefinitionError::InvalidParameter { index })?;
        let expression = source_type
            .expressions()
            .get(SourceTypeExpressionId::new(index))
            .ok_or(SourceAttributeDefinitionError::InvalidParameter { index })?;
        let written_range = range(input.source_id, ranges[index].0, ranges[index].1);
        let expression_site = TypedSiteRef::Node(TypedNodeId::new(expression_sites[index]));
        let head_site = TypedSiteRef::Node(TypedNodeId::new(head_sites[index]));
        if application.binding() != input.parameters[index].binding
            || application.source_ordinal() != index
            || application.root() != SourceTypeExpressionId::new(index)
            || expression.id() != SourceTypeExpressionId::new(index)
            || expression.source_id() != input.source_id
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
            return Err(SourceAttributeDefinitionError::InvalidParameter { index });
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
            return Err(SourceAttributeDefinitionError::InvalidArenaOwnership);
        }
    }
    Ok(())
}

fn validate_term_and_atomic_profile(
    input: &SourceAttributeDefinitionHandoffInput,
    source_term: &SourcePrimaryTermHandoff,
    source_atomic_formula: &SourceAtomicFormulaHandoff,
) -> Result<(), SourceAttributeDefinitionError> {
    if source_term.terms().len() != 2
        || source_term.references().len() != 2
        || !source_term.numeric_type_requests().is_empty()
        || source_atomic_formula.formulas().len() != 1
        || !source_atomic_formula.wrappers().is_empty()
        || !source_atomic_formula.predicate_segments().is_empty()
        || !source_atomic_formula.predicate_heads().is_empty()
        || !source_atomic_formula.candidates().is_empty()
        || !source_atomic_formula.type_sites().is_empty()
        || !source_atomic_formula.attributes().is_empty()
        || source_atomic_formula.edges().len() != 2
        || source_atomic_formula.requests().len() != 2
    {
        return Err(SourceAttributeDefinitionError::DependencyMismatch);
    }
    let term_sites = [33, 35];
    let term_ranges = [(104, 105), (108, 109)];
    let term_spellings = ["x", "y"];
    for index in 0..2 {
        let term = source_term
            .terms()
            .get(SourcePrimaryTermId::new(index))
            .ok_or(SourceAttributeDefinitionError::DependencyMismatch)?;
        let reference = source_term
            .references()
            .get(SourcePrimaryTermReferenceId::new(index))
            .ok_or(SourceAttributeDefinitionError::DependencyMismatch)?;
        if term.site() != &TypedSiteRef::Node(TypedNodeId::new(term_sites[index]))
            || term.source_range()
                != range(input.source_id, term_ranges[index].0, term_ranges[index].1)
            || term.source_ordinal() != index
            || term.context() != BindingContextId::new(1)
            || term.recovery() != SourcePrimaryTermRecovery::Normal
            || term.spelling() != term_spellings[index]
            || term.kind() != SourcePrimaryTermKind::VariableReference
            || term.role() != SourcePrimaryTermRole::Value
            || term.parent().is_some()
            || reference.term() != SourcePrimaryTermId::new(index)
            || reference.binding() != BindingId::new(index)
            || reference.role() != SourcePrimaryTermReferenceRole::Variable
        {
            return Err(SourceAttributeDefinitionError::DependencyMismatch);
        }
    }
    let formula = source_atomic_formula
        .formulas()
        .get(SourceAtomicFormulaId::new(0))
        .ok_or(SourceAttributeDefinitionError::DependencyMismatch)?;
    if formula.site() != &TypedSiteRef::Node(TypedNodeId::new(37))
        || formula.source_range() != range(input.source_id, 104, 109)
        || formula.source_ordinal() != 0
        || formula.context() != BindingContextId::new(1)
        || formula.recovery() != SourceAtomicFormulaRecovery::Normal
        || formula.spelling() != "x = y"
        || formula.kind() != SourceAtomicFormulaKind::Equality
    {
        return Err(SourceAttributeDefinitionError::DependencyMismatch);
    }
    for index in 0..2 {
        let edge = source_atomic_formula
            .edges()
            .get(SourceAtomicEdgeId::new(index))
            .ok_or(SourceAttributeDefinitionError::DependencyMismatch)?;
        let request = source_atomic_formula
            .requests()
            .get(SourceAtomicRequestId::new(index))
            .ok_or(SourceAttributeDefinitionError::DependencyMismatch)?;
        let role = if index == 0 {
            SourceAtomicEdgeRole::BuiltinLeftOperand
        } else {
            SourceAtomicEdgeRole::BuiltinRightOperand
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
            return Err(SourceAttributeDefinitionError::DependencyMismatch);
        }
    }
    Ok(())
}

fn validate_handoff_rows(
    handoff: &SourceAttributeDefinitionHandoff,
    source_context: &SourceBindingContextHandoff,
    source_type: &SourceTypeApplicationHandoff,
    source_term: &SourcePrimaryTermHandoff,
    source_atomic_formula: &SourceAtomicFormulaHandoff,
    arena: &TypedArena,
) -> Result<(), SourceAttributeDefinitionError> {
    validate_handoff_dense_ids(handoff)?;
    validate_handoff_resolver_identity(handoff)?;
    let input = SourceAttributeDefinitionHandoffInput {
        source_id: handoff.source_id,
        module_id: handoff.module_id.clone(),
        definitions: handoff
            .definitions
            .iter()
            .map(|(_, row)| SourceAttributeDefinitionInput {
                symbol: row.symbol.clone(),
                definition: row.definition,
                contribution: row.contribution,
                site: row.site.clone(),
                source_range: row.source_range,
                source_ordinal: row.source_ordinal,
                context: row.context,
                recovery: row.recovery,
                spelling: row.spelling.clone(),
                subject: row.subject,
                definiens: row.definiens,
            })
            .collect(),
        parameters: handoff
            .parameters
            .iter()
            .map(|(_, row)| SourceAttributeParameterInput {
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
        subjects: handoff
            .subjects
            .iter()
            .map(|(_, row)| SourceAttributeSubjectInput {
                owner: row.owner,
                binding: row.binding,
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
            .map(|(_, row)| SourceAttributeDefiniensInput {
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
        &[4, 0, 7, 0],
    ) {
        return Err(SourceAttributeDefinitionError::InvalidDefinition { index: 0 });
    }
    Ok(())
}

fn validate_handoff_dense_ids(
    handoff: &SourceAttributeDefinitionHandoff,
) -> Result<(), SourceAttributeDefinitionError> {
    for (index, row) in handoff.definitions.rows.iter().enumerate() {
        if row.id != SourceAttributeDefinitionId::new(index) {
            return Err(SourceAttributeDefinitionError::InvalidDefinition { index });
        }
    }
    for (index, row) in handoff.parameters.rows.iter().enumerate() {
        if row.id != SourceAttributeParameterId::new(index) {
            return Err(SourceAttributeDefinitionError::InvalidParameter { index });
        }
    }
    for (index, row) in handoff.subjects.rows.iter().enumerate() {
        if row.id != SourceAttributeSubjectId::new(index) {
            return Err(SourceAttributeDefinitionError::InvalidSubject { index });
        }
    }
    for (index, row) in handoff.definientia.rows.iter().enumerate() {
        if row.id != SourceAttributeDefiniensId::new(index) {
            return Err(SourceAttributeDefinitionError::InvalidDefiniens { index });
        }
    }
    Ok(())
}

fn validate_handoff_resolver_identity(
    handoff: &SourceAttributeDefinitionHandoff,
) -> Result<(), SourceAttributeDefinitionError> {
    let definition = handoff
        .definitions
        .rows
        .first()
        .ok_or(SourceAttributeDefinitionError::UnsupportedTaskShape)?;
    let expected_fqn = format!(
        "{}::{}::{}",
        handoff.module_id.package().as_str(),
        handoff.module_id.path().as_str(),
        definition.symbol.local().as_str()
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
        return Err(SourceAttributeDefinitionError::InvalidResolverDefinition { index: 0 });
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

const fn recovery_key(recovery: SourceAttributeDefinitionRecovery) -> &'static str {
    match recovery {
        SourceAttributeDefinitionRecovery::Normal => "normal",
        SourceAttributeDefinitionRecovery::Degraded => "degraded",
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
#[path = "../tests/support/source_attribute_definition_unit.rs"]
mod tests;
