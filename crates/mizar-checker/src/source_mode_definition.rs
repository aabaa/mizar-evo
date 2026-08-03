//! Syntax-free mode-definition intake for checker phase 6.

use crate::{
    binding_env::{
        BindingContextId, BindingId, BindingKind, BindingRecoveryState, BindingStatus,
        BindingTypeSite,
    },
    source_context::{
        SourceBindingContextHandoff, SourceBindingSiteRole, SourceItemRecovery, SourceItemRole,
        SourceItemVisibility,
    },
    source_type::{
        SourceTypeApplicationForm, SourceTypeApplicationHandoff, SourceTypeApplicationId,
        SourceTypeExpressionId, SourceTypeHead, SourceTypeModeRhsId,
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

dense_id!(SourceModeDefinitionId);
dense_id!(SourceModeParameterId);
dense_id!(SourceModeApplicationId);
dense_id!(SourceModeExpansionId);
dense_id!(SourceModeInhabitationRequestId);
dense_id!(SourceModePropertyId);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceModeDefinitionHandoffInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub definitions: Vec<SourceModeDefinitionInput>,
    pub parameters: Vec<SourceModeParameterInput>,
    pub applications: Vec<SourceModeApplicationInput>,
    pub expansions: Vec<SourceModeExpansionInput>,
    pub inhabitation_requests: Vec<SourceModeInhabitationRequestInput>,
    pub properties: Vec<SourceModePropertyInput>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceModeDefinitionInput {
    pub symbol: SymbolId,
    pub definition: DefinitionId,
    pub contribution: SourceContributionId,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub source_ordinal: usize,
    pub context: BindingContextId,
    pub recovery: SourceModeDefinitionRecovery,
    pub spelling: String,
    pub application: SourceModeApplicationId,
    pub expansion: SourceModeExpansionId,
    pub inhabitation_request: SourceModeInhabitationRequestId,
    pub property: Option<SourceModePropertyId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceModeParameterInput {
    pub owner: SourceModeDefinitionId,
    pub ordinal: usize,
    pub binding: BindingId,
    pub written_type: SourceTypeApplicationId,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub declaration_range: SourceRange,
    pub pattern_range: SourceRange,
    pub context: BindingContextId,
    pub recovery: SourceModeDefinitionRecovery,
    pub spelling: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceModeApplicationInput {
    pub owner: SourceModeDefinitionId,
    pub ordinal: usize,
    pub parameters: Vec<SourceModeParameterId>,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub context: BindingContextId,
    pub recovery: SourceModeDefinitionRecovery,
    pub spelling: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceModeExpansionInput {
    pub owner: SourceModeDefinitionId,
    pub ordinal: usize,
    pub rhs: SourceTypeModeRhsId,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub context: BindingContextId,
    pub recovery: SourceModeDefinitionRecovery,
    pub spelling: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceModeInhabitationRequestInput {
    pub owner: SourceModeDefinitionId,
    pub ordinal: usize,
    pub expansion: SourceModeExpansionId,
    pub kind: SourceModeInhabitationRequestKind,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub context: BindingContextId,
    pub recovery: SourceModeDefinitionRecovery,
    pub spelling: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceModePropertyInput {
    pub owner: SourceModeDefinitionId,
    pub ordinal: usize,
    pub kind: SourceModePropertyKind,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub justification: SourceAnchor,
    pub recovery: SourceModeDefinitionRecovery,
    pub spelling: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceModeInhabitationRequestKind {
    Rhs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceModePropertyKind {
    Sethood,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceModeDefinitionRecovery {
    Normal,
    Degraded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceModeDefinition {
    id: SourceModeDefinitionId,
    symbol: SymbolId,
    definition: DefinitionId,
    contribution: SourceContributionId,
    site: TypedSiteRef,
    source_range: SourceRange,
    source_ordinal: usize,
    context: BindingContextId,
    recovery: SourceModeDefinitionRecovery,
    spelling: String,
    application: SourceModeApplicationId,
    expansion: SourceModeExpansionId,
    inhabitation_request: SourceModeInhabitationRequestId,
    property: Option<SourceModePropertyId>,
    origin: SemanticOrigin,
}

impl SourceModeDefinition {
    pub const fn id(&self) -> SourceModeDefinitionId {
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
    pub const fn recovery(&self) -> SourceModeDefinitionRecovery {
        self.recovery
    }
    pub fn spelling(&self) -> &str {
        &self.spelling
    }
    pub const fn application(&self) -> SourceModeApplicationId {
        self.application
    }
    pub const fn expansion(&self) -> SourceModeExpansionId {
        self.expansion
    }
    pub const fn inhabitation_request(&self) -> SourceModeInhabitationRequestId {
        self.inhabitation_request
    }
    pub const fn property(&self) -> Option<SourceModePropertyId> {
        self.property
    }
    pub const fn origin(&self) -> &SemanticOrigin {
        &self.origin
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceModeParameter {
    id: SourceModeParameterId,
    owner: SourceModeDefinitionId,
    ordinal: usize,
    binding: BindingId,
    written_type: SourceTypeApplicationId,
    site: TypedSiteRef,
    source_range: SourceRange,
    declaration_range: SourceRange,
    pattern_range: SourceRange,
    context: BindingContextId,
    recovery: SourceModeDefinitionRecovery,
    spelling: String,
}

impl SourceModeParameter {
    pub const fn id(&self) -> SourceModeParameterId {
        self.id
    }
    pub const fn owner(&self) -> SourceModeDefinitionId {
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
    pub const fn pattern_range(&self) -> SourceRange {
        self.pattern_range
    }
    pub const fn context(&self) -> BindingContextId {
        self.context
    }
    pub const fn recovery(&self) -> SourceModeDefinitionRecovery {
        self.recovery
    }
    pub fn spelling(&self) -> &str {
        &self.spelling
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceModeApplication {
    id: SourceModeApplicationId,
    owner: SourceModeDefinitionId,
    ordinal: usize,
    parameters: Vec<SourceModeParameterId>,
    site: TypedSiteRef,
    source_range: SourceRange,
    context: BindingContextId,
    recovery: SourceModeDefinitionRecovery,
    spelling: String,
}

impl SourceModeApplication {
    pub const fn id(&self) -> SourceModeApplicationId {
        self.id
    }
    pub const fn owner(&self) -> SourceModeDefinitionId {
        self.owner
    }
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }
    pub fn parameters(&self) -> &[SourceModeParameterId] {
        &self.parameters
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
    pub const fn recovery(&self) -> SourceModeDefinitionRecovery {
        self.recovery
    }
    pub fn spelling(&self) -> &str {
        &self.spelling
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceModeExpansion {
    id: SourceModeExpansionId,
    owner: SourceModeDefinitionId,
    ordinal: usize,
    rhs: SourceTypeModeRhsId,
    site: TypedSiteRef,
    source_range: SourceRange,
    context: BindingContextId,
    recovery: SourceModeDefinitionRecovery,
    spelling: String,
}

impl SourceModeExpansion {
    pub const fn id(&self) -> SourceModeExpansionId {
        self.id
    }
    pub const fn owner(&self) -> SourceModeDefinitionId {
        self.owner
    }
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }
    pub const fn rhs(&self) -> SourceTypeModeRhsId {
        self.rhs
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
    pub const fn recovery(&self) -> SourceModeDefinitionRecovery {
        self.recovery
    }
    pub fn spelling(&self) -> &str {
        &self.spelling
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceModeInhabitationRequest {
    id: SourceModeInhabitationRequestId,
    owner: SourceModeDefinitionId,
    ordinal: usize,
    expansion: SourceModeExpansionId,
    kind: SourceModeInhabitationRequestKind,
    site: TypedSiteRef,
    source_range: SourceRange,
    context: BindingContextId,
    recovery: SourceModeDefinitionRecovery,
    spelling: String,
}

impl SourceModeInhabitationRequest {
    pub const fn id(&self) -> SourceModeInhabitationRequestId {
        self.id
    }
    pub const fn owner(&self) -> SourceModeDefinitionId {
        self.owner
    }
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }
    pub const fn expansion(&self) -> SourceModeExpansionId {
        self.expansion
    }
    pub const fn kind(&self) -> SourceModeInhabitationRequestKind {
        self.kind
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
    pub const fn recovery(&self) -> SourceModeDefinitionRecovery {
        self.recovery
    }
    pub fn spelling(&self) -> &str {
        &self.spelling
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceModeProperty {
    id: SourceModePropertyId,
    owner: SourceModeDefinitionId,
    ordinal: usize,
    kind: SourceModePropertyKind,
    site: TypedSiteRef,
    source_range: SourceRange,
    justification: SourceAnchor,
    recovery: SourceModeDefinitionRecovery,
    spelling: String,
    obligation: InitialObligationId,
}

impl SourceModeProperty {
    pub const fn id(&self) -> SourceModePropertyId {
        self.id
    }
    pub const fn owner(&self) -> SourceModeDefinitionId {
        self.owner
    }
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }
    pub const fn kind(&self) -> SourceModePropertyKind {
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
    pub const fn recovery(&self) -> SourceModeDefinitionRecovery {
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
    SourceModeDefinitionTable,
    SourceModeDefinition,
    SourceModeDefinitionId
);
table!(
    SourceModeParameterTable,
    SourceModeParameter,
    SourceModeParameterId
);
table!(
    SourceModeApplicationTable,
    SourceModeApplication,
    SourceModeApplicationId
);
table!(
    SourceModeExpansionTable,
    SourceModeExpansion,
    SourceModeExpansionId
);
table!(
    SourceModeInhabitationRequestTable,
    SourceModeInhabitationRequest,
    SourceModeInhabitationRequestId
);
table!(
    SourceModePropertyTable,
    SourceModeProperty,
    SourceModePropertyId
);

#[derive(Debug, Clone, PartialEq, Eq)]
struct SourceModeResolverIdentity {
    symbol: SymbolId,
    definition: DefinitionId,
    contribution: SourceContributionId,
    origin: SemanticOrigin,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceModeDefinitionHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    resolver_identity: SourceModeResolverIdentity,
    source_context_fingerprint: String,
    source_type_fingerprint: String,
    base_initial_obligation_count: usize,
    definitions: SourceModeDefinitionTable,
    parameters: SourceModeParameterTable,
    applications: SourceModeApplicationTable,
    expansions: SourceModeExpansionTable,
    inhabitation_requests: SourceModeInhabitationRequestTable,
    properties: SourceModePropertyTable,
}

impl SourceModeDefinitionHandoff {
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
    pub const fn base_initial_obligation_count(&self) -> usize {
        self.base_initial_obligation_count
    }
    pub const fn definitions(&self) -> &SourceModeDefinitionTable {
        &self.definitions
    }
    pub const fn parameters(&self) -> &SourceModeParameterTable {
        &self.parameters
    }
    pub const fn applications(&self) -> &SourceModeApplicationTable {
        &self.applications
    }
    pub const fn expansions(&self) -> &SourceModeExpansionTable {
        &self.expansions
    }
    pub const fn inhabitation_requests(&self) -> &SourceModeInhabitationRequestTable {
        &self.inhabitation_requests
    }
    pub const fn properties(&self) -> &SourceModePropertyTable {
        &self.properties
    }

    pub fn debug_text(&self) -> String {
        let mut output = String::from("source-mode-definition-debug-v1\n");
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
            "base-initial-obligation-count: {}",
            self.base_initial_obligation_count
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
                row.source_range.end
            );
            write_site(&mut output, &row.site);
            let _ = write!(
                output,
                " context={} recovery={} origin_range=",
                row.context.index(),
                recovery_key(row.recovery)
            );
            write_anchor_range(&mut output, row.origin.anchor());
            let property = row
                .property
                .map_or_else(|| "none".to_owned(), |id| id.index().to_string());
            let _ = writeln!(
                output,
                " origin_path={:?} spelling={:?} application={} expansion={} inhabitation_request={} property={}",
                row.origin.structural_path(),
                row.spelling,
                row.application.index(),
                row.expansion.index(),
                row.inhabitation_request.index(),
                property
            );
        }
        for (id, row) in self.parameters.iter() {
            let _ = write!(
                output,
                "parameter#{} owner={} ordinal={} binding={} written_type={} range={}..{} declaration_range={}..{} pattern_range={}..{} site=",
                id.index(),
                row.owner.index(),
                row.ordinal,
                row.binding.index(),
                row.written_type.index(),
                row.source_range.start,
                row.source_range.end,
                row.declaration_range.start,
                row.declaration_range.end,
                row.pattern_range.start,
                row.pattern_range.end
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
        for (id, row) in self.applications.iter() {
            let parameters: Vec<_> = row.parameters.iter().map(|id| id.index()).collect();
            let _ = write!(
                output,
                "application#{} owner={} ordinal={} parameters={:?} range={}..{} site=",
                id.index(),
                row.owner.index(),
                row.ordinal,
                parameters,
                row.source_range.start,
                row.source_range.end
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
        for (id, row) in self.expansions.iter() {
            let _ = write!(
                output,
                "expansion#{} owner={} ordinal={} rhs={} range={}..{} site=",
                id.index(),
                row.owner.index(),
                row.ordinal,
                row.rhs.index(),
                row.source_range.start,
                row.source_range.end
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
        for (id, row) in self.inhabitation_requests.iter() {
            let _ = write!(
                output,
                "inhabitation-request#{} owner={} ordinal={} expansion={} kind={} range={}..{} site=",
                id.index(),
                row.owner.index(),
                row.ordinal,
                row.expansion.index(),
                request_key(row.kind),
                row.source_range.start,
                row.source_range.end
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
        for (id, row) in self.properties.iter() {
            let _ = write!(
                output,
                "property#{} owner={} ordinal={} kind={} range={}..{} site=",
                id.index(),
                row.owner.index(),
                row.ordinal,
                property_key(row.kind),
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

    pub(crate) fn validate_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        source_context: &SourceBindingContextHandoff,
        source_type: &SourceTypeApplicationHandoff,
        initial_obligations: &InitialObligationTable,
        arena: &TypedArena,
    ) -> Result<(), SourceModeDefinitionError> {
        validate_dependency_identity(source_id, module_id, source_context, source_type, arena)?;
        if self.source_id != source_id || &self.module_id != module_id {
            return Err(SourceModeDefinitionError::SourceIdentityMismatch);
        }
        if self.source_context_fingerprint != source_context.debug_text()
            || self.source_type_fingerprint != source_type.debug_text()
        {
            return Err(SourceModeDefinitionError::DependencyMismatch);
        }
        validate_handoff_rows(
            self,
            source_context,
            source_type,
            initial_obligations,
            arena,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceModeDefinitionProjection {
    base_initial_obligations: InitialObligationTable,
    handoff: SourceModeDefinitionHandoff,
    initial_obligations: InitialObligationTable,
}

impl SourceModeDefinitionProjection {
    pub const fn base_initial_obligations(&self) -> &InitialObligationTable {
        &self.base_initial_obligations
    }
    pub const fn handoff(&self) -> &SourceModeDefinitionHandoff {
        &self.handoff
    }
    pub const fn initial_obligations(&self) -> &InitialObligationTable {
        &self.initial_obligations
    }
    pub fn into_parts(
        self,
    ) -> (
        InitialObligationTable,
        SourceModeDefinitionHandoff,
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
pub enum SourceModeDefinitionError {
    SourceIdentityMismatch,
    DependencyMismatch,
    InvalidResolverDefinition { index: usize },
    InvalidDefinition { index: usize },
    InvalidParameter { index: usize },
    InvalidApplication { index: usize },
    InvalidExpansion { index: usize },
    InvalidInhabitationRequest { index: usize },
    InvalidProperty { index: usize },
    InvalidObligation,
    InvalidArenaOwnership,
    UnsupportedTaskShape,
}

impl fmt::Display for SourceModeDefinitionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SourceIdentityMismatch => f.write_str("mode-definition source identity mismatch"),
            Self::DependencyMismatch => f.write_str("mode-definition dependency mismatch"),
            Self::InvalidResolverDefinition { index } => {
                write!(f, "invalid mode resolver definition {index}")
            }
            Self::InvalidDefinition { index } => {
                write!(f, "invalid source mode definition {index}")
            }
            Self::InvalidParameter { index } => write!(f, "invalid source mode parameter {index}"),
            Self::InvalidApplication { index } => {
                write!(f, "invalid source mode application {index}")
            }
            Self::InvalidExpansion { index } => write!(f, "invalid source mode expansion {index}"),
            Self::InvalidInhabitationRequest { index } => {
                write!(f, "invalid source mode inhabitation request {index}")
            }
            Self::InvalidProperty { index } => write!(f, "invalid source mode property {index}"),
            Self::InvalidObligation => f.write_str("invalid mode-definition obligation"),
            Self::InvalidArenaOwnership => {
                f.write_str("invalid mode-definition typed-arena ownership")
            }
            Self::UnsupportedTaskShape => f.write_str("unsupported mode-definition task shape"),
        }
    }
}

impl Error for SourceModeDefinitionError {}

pub struct SourceModeDefinitionProducer;

impl SourceModeDefinitionProducer {
    pub fn build(
        input: SourceModeDefinitionHandoffInput,
        env: &SymbolEnv,
        source_context: &SourceBindingContextHandoff,
        source_type: &SourceTypeApplicationHandoff,
        base_initial_obligations: &InitialObligationTable,
        arena: &TypedArena,
    ) -> Result<SourceModeDefinitionProjection, SourceModeDefinitionError> {
        validate_dependency_identity(
            input.source_id,
            &input.module_id,
            source_context,
            source_type,
            arena,
        )?;
        if env.module_id() != &input.module_id {
            return Err(SourceModeDefinitionError::SourceIdentityMismatch);
        }
        validate_input_shape(&input)?;
        validate_baseline(base_initial_obligations)?;
        let origin = validate_resolver_definition(&input, env)?;
        validate_input_rows(&input, source_context, source_type, arena)?;

        let mut initial_obligations = base_initial_obligations.clone();
        let property = &input.properties[0];
        let obligation = initial_obligations.insert(InitialObligationDraft {
            kind: InitialObligationKind::Sethood,
            owner: property.site.clone(),
            source_range: property.source_range,
            assumptions: Vec::new(),
            goal: InitialObligationGoal::new(
                "source.definition.mode.correctness:definition=0:sethood",
            ),
            provenance: InitialObligationProvenance::new(
                "source.definition.mode:definition=0:property=0",
            ),
            status: InitialObligationStatus::Pending,
        });
        if obligation.index() != base_initial_obligations.len() {
            return Err(SourceModeDefinitionError::InvalidObligation);
        }

        let definitions = SourceModeDefinitionTable {
            rows: input
                .definitions
                .into_iter()
                .enumerate()
                .map(|(index, row)| SourceModeDefinition {
                    id: SourceModeDefinitionId::new(index),
                    symbol: row.symbol,
                    definition: row.definition,
                    contribution: row.contribution,
                    site: row.site,
                    source_range: row.source_range,
                    source_ordinal: row.source_ordinal,
                    context: row.context,
                    recovery: row.recovery,
                    spelling: row.spelling,
                    application: row.application,
                    expansion: row.expansion,
                    inhabitation_request: row.inhabitation_request,
                    property: row.property,
                    origin: origin.clone(),
                })
                .collect(),
        };
        let parameters = SourceModeParameterTable {
            rows: input
                .parameters
                .into_iter()
                .enumerate()
                .map(|(index, row)| SourceModeParameter {
                    id: SourceModeParameterId::new(index),
                    owner: row.owner,
                    ordinal: row.ordinal,
                    binding: row.binding,
                    written_type: row.written_type,
                    site: row.site,
                    source_range: row.source_range,
                    declaration_range: row.declaration_range,
                    pattern_range: row.pattern_range,
                    context: row.context,
                    recovery: row.recovery,
                    spelling: row.spelling,
                })
                .collect(),
        };
        let applications = SourceModeApplicationTable {
            rows: input
                .applications
                .into_iter()
                .enumerate()
                .map(|(index, row)| SourceModeApplication {
                    id: SourceModeApplicationId::new(index),
                    owner: row.owner,
                    ordinal: row.ordinal,
                    parameters: row.parameters,
                    site: row.site,
                    source_range: row.source_range,
                    context: row.context,
                    recovery: row.recovery,
                    spelling: row.spelling,
                })
                .collect(),
        };
        let expansions = SourceModeExpansionTable {
            rows: input
                .expansions
                .into_iter()
                .enumerate()
                .map(|(index, row)| SourceModeExpansion {
                    id: SourceModeExpansionId::new(index),
                    owner: row.owner,
                    ordinal: row.ordinal,
                    rhs: row.rhs,
                    site: row.site,
                    source_range: row.source_range,
                    context: row.context,
                    recovery: row.recovery,
                    spelling: row.spelling,
                })
                .collect(),
        };
        let inhabitation_requests = SourceModeInhabitationRequestTable {
            rows: input
                .inhabitation_requests
                .into_iter()
                .enumerate()
                .map(|(index, row)| SourceModeInhabitationRequest {
                    id: SourceModeInhabitationRequestId::new(index),
                    owner: row.owner,
                    ordinal: row.ordinal,
                    expansion: row.expansion,
                    kind: row.kind,
                    site: row.site,
                    source_range: row.source_range,
                    context: row.context,
                    recovery: row.recovery,
                    spelling: row.spelling,
                })
                .collect(),
        };
        let properties = SourceModePropertyTable {
            rows: input
                .properties
                .into_iter()
                .enumerate()
                .map(|(index, row)| SourceModeProperty {
                    id: SourceModePropertyId::new(index),
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
        let definition = &definitions.rows[0];
        let handoff = SourceModeDefinitionHandoff {
            source_id: input.source_id,
            module_id: input.module_id,
            resolver_identity: SourceModeResolverIdentity {
                symbol: definition.symbol.clone(),
                definition: definition.definition,
                contribution: definition.contribution,
                origin: definition.origin.clone(),
            },
            source_context_fingerprint: source_context.debug_text(),
            source_type_fingerprint: source_type.debug_text(),
            base_initial_obligation_count: base_initial_obligations.len(),
            definitions,
            parameters,
            applications,
            expansions,
            inhabitation_requests,
            properties,
        };
        validate_handoff_rows(
            &handoff,
            source_context,
            source_type,
            &initial_obligations,
            arena,
        )?;
        Ok(SourceModeDefinitionProjection {
            base_initial_obligations: base_initial_obligations.clone(),
            handoff,
            initial_obligations,
        })
    }
}

fn validate_dependency_identity(
    source_id: SourceId,
    module_id: &ModuleId,
    source_context: &SourceBindingContextHandoff,
    source_type: &SourceTypeApplicationHandoff,
    arena: &TypedArena,
) -> Result<(), SourceModeDefinitionError> {
    if source_context.source_id() != source_id
        || source_context.module_id() != module_id
        || source_type.source_id() != source_id
        || source_type.module_id() != module_id
    {
        return Err(SourceModeDefinitionError::SourceIdentityMismatch);
    }
    source_type
        .validate_installation(source_id, module_id, arena)
        .map_err(|_| SourceModeDefinitionError::DependencyMismatch)
}

fn validate_input_shape(
    input: &SourceModeDefinitionHandoffInput,
) -> Result<(), SourceModeDefinitionError> {
    if input.definitions.len() != 1
        || input.parameters.len() != 2
        || input.applications.len() != 1
        || input.expansions.len() != 1
        || input.inhabitation_requests.len() != 1
        || input.properties.len() != 1
    {
        return Err(SourceModeDefinitionError::UnsupportedTaskShape);
    }
    Ok(())
}

fn validate_baseline(table: &InitialObligationTable) -> Result<(), SourceModeDefinitionError> {
    if table.iter().any(|(_, row)| {
        matches!(
            row.kind,
            InitialObligationKind::PredicatePropertyCorrectness
                | InitialObligationKind::FunctorExistence
                | InitialObligationKind::FunctorUniqueness
        )
    }) {
        return Err(SourceModeDefinitionError::InvalidObligation);
    }
    validate_source_mode_definition_absence(table)
}

pub(crate) fn validate_source_mode_definition_absence(
    initial_obligations: &InitialObligationTable,
) -> Result<(), SourceModeDefinitionError> {
    if initial_obligations.iter().any(|(_, row)| {
        in_mode_definition_domain(row.goal.as_str())
            || in_mode_definition_domain(row.provenance.as_str())
    }) {
        return Err(SourceModeDefinitionError::InvalidObligation);
    }
    Ok(())
}

fn validate_resolver_definition(
    input: &SourceModeDefinitionHandoffInput,
    env: &SymbolEnv,
) -> Result<SemanticOrigin, SourceModeDefinitionError> {
    if env.symbols().len() != 1 || env.definitions().len() != 1 || env.contributions().len() != 1 {
        return Err(SourceModeDefinitionError::InvalidResolverDefinition { index: 0 });
    }
    let row = &input.definitions[0];
    let symbol = env
        .symbols()
        .get(&row.symbol)
        .ok_or(SourceModeDefinitionError::InvalidResolverDefinition { index: 0 })?;
    let definition = env
        .definitions()
        .get(row.definition)
        .ok_or(SourceModeDefinitionError::InvalidResolverDefinition { index: 0 })?;
    let contribution = env
        .contributions()
        .get(row.contribution)
        .ok_or(SourceModeDefinitionError::InvalidResolverDefinition { index: 0 })?;
    let expected_anchor = SourceAnchor::Range(range(input.source_id, 0, 140));
    if row.definition.index() != 0
        || row.symbol.module() != &input.module_id
        || symbol.symbol() != &row.symbol
        || symbol.kind() != SymbolKind::Mode
        || symbol.visibility() != Visibility::Public
        || symbol.export_status() != ExportStatus::Exported
        || symbol.primary_spelling() != "Task262Mode [ x , y ]"
        || symbol.notation_spelling() != Some("Task262Mode [ x , y ]")
        || symbol.contribution() != row.contribution
        || symbol.origin() != definition.origin()
        || symbol.signature() != definition.signature()
        || !symbol.relations().is_empty()
        || definition.id() != row.definition
        || definition.symbol() != &row.symbol
        || definition.kind() != DefinitionKind::Mode
        || definition.visibility() != Visibility::Public
        || !definition.parameters().is_empty()
        || !definition.binders().is_empty()
        || definition.arity().is_some()
        || definition.notation_shape() != Some("Task262Mode [ x , y ]")
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
            range(input.source_id, 45, 135),
            &[4, 0, 10, 0],
        )
    {
        return Err(SourceModeDefinitionError::InvalidResolverDefinition { index: 0 });
    }
    Ok(definition.origin().clone())
}

fn validate_input_rows(
    input: &SourceModeDefinitionHandoffInput,
    source_context: &SourceBindingContextHandoff,
    source_type: &SourceTypeApplicationHandoff,
    arena: &TypedArena,
) -> Result<(), SourceModeDefinitionError> {
    let source = input.source_id;
    let context = BindingContextId::new(1);
    let definition = &input.definitions[0];
    if definition.site != TypedSiteRef::Node(TypedNodeId::new(49))
        || definition.source_range != range(source, 45, 135)
        || definition.source_ordinal != 0
        || definition.context != context
        || definition.recovery != SourceModeDefinitionRecovery::Normal
        || definition.spelling
            != "mode Task262ModeDefinition: Task262Mode [x, y] is set;\n  sethood by computation(steps: 1);"
        || definition.application != SourceModeApplicationId::new(0)
        || definition.expansion != SourceModeExpansionId::new(0)
        || definition.inhabitation_request != SourceModeInhabitationRequestId::new(0)
        || definition.property != Some(SourceModePropertyId::new(0))
    {
        return Err(SourceModeDefinitionError::InvalidDefinition { index: 0 });
    }
    if !valid_site(
        &definition.site,
        definition.source_range,
        "source.definition.mode",
        LocalTypeContextId::new(1),
        arena,
    ) {
        return Err(SourceModeDefinitionError::InvalidArenaOwnership);
    }
    let local_context = validate_context_profile(input, source_context, arena)?;
    let parameter_ranges = [(13, 26, 17, 18, 86, 87), (29, 42, 33, 34, 89, 90)];
    let parameter_sites = [37, 41];
    let parameter_spellings = ["let x be set;", "let y be set;"];
    for (index, row) in input.parameters.iter().enumerate() {
        let (start, end, declaration_start, declaration_end, pattern_start, pattern_end) =
            parameter_ranges[index];
        if row.owner != SourceModeDefinitionId::new(0)
            || row.ordinal != index
            || row.binding != BindingId::new(index)
            || row.written_type != SourceTypeApplicationId::new(index)
            || row.site != TypedSiteRef::Node(TypedNodeId::new(parameter_sites[index]))
            || row.source_range != range(source, start, end)
            || row.declaration_range != range(source, declaration_start, declaration_end)
            || row.pattern_range != range(source, pattern_start, pattern_end)
            || row.context != context
            || row.recovery != SourceModeDefinitionRecovery::Normal
            || row.spelling != parameter_spellings[index]
        {
            return Err(SourceModeDefinitionError::InvalidParameter { index });
        }
        if !valid_site(
            &row.site,
            row.declaration_range,
            "source.definition.mode.parameter",
            local_context,
            arena,
        ) {
            return Err(SourceModeDefinitionError::InvalidArenaOwnership);
        }
    }
    validate_type_profile(input, source_type, local_context, arena)?;

    let application = &input.applications[0];
    if application.owner != SourceModeDefinitionId::new(0)
        || application.ordinal != 0
        || application.parameters != [SourceModeParameterId::new(0), SourceModeParameterId::new(1)]
        || application.site != TypedSiteRef::Node(TypedNodeId::new(42))
        || application.source_range != range(source, 73, 91)
        || application.context != context
        || application.recovery != SourceModeDefinitionRecovery::Normal
        || application.spelling != "Task262Mode [ x , y ]"
    {
        return Err(SourceModeDefinitionError::InvalidApplication { index: 0 });
    }
    if !valid_site(
        &application.site,
        application.source_range,
        "source.definition.mode.application",
        local_context,
        arena,
    ) {
        return Err(SourceModeDefinitionError::InvalidArenaOwnership);
    }

    let expansion = &input.expansions[0];
    if expansion.owner != SourceModeDefinitionId::new(0)
        || expansion.ordinal != 0
        || expansion.rhs != SourceTypeModeRhsId::new(0)
        || expansion.site != TypedSiteRef::Node(TypedNodeId::new(44))
        || expansion.source_range != range(source, 95, 98)
        || expansion.context != context
        || expansion.recovery != SourceModeDefinitionRecovery::Normal
        || expansion.spelling != "set"
    {
        return Err(SourceModeDefinitionError::InvalidExpansion { index: 0 });
    }
    if !valid_site(
        &expansion.site,
        expansion.source_range,
        "source.type.expression",
        local_context,
        arena,
    ) {
        return Err(SourceModeDefinitionError::InvalidArenaOwnership);
    }

    let request = &input.inhabitation_requests[0];
    if request.owner != SourceModeDefinitionId::new(0)
        || request.ordinal != 0
        || request.expansion != SourceModeExpansionId::new(0)
        || request.kind != SourceModeInhabitationRequestKind::Rhs
        || request.site != expansion.site
        || request.source_range != expansion.source_range
        || request.context != context
        || request.recovery != SourceModeDefinitionRecovery::Normal
        || request.spelling != "set"
    {
        return Err(SourceModeDefinitionError::InvalidInhabitationRequest { index: 0 });
    }

    let property = &input.properties[0];
    if property.owner != SourceModeDefinitionId::new(0)
        || property.ordinal != 0
        || property.kind != SourceModePropertyKind::Sethood
        || property.site != TypedSiteRef::Node(TypedNodeId::new(48))
        || property.source_range != range(source, 102, 135)
        || property.justification != SourceAnchor::Range(range(source, 113, 134))
        || property.recovery != SourceModeDefinitionRecovery::Normal
        || property.spelling != "sethood by computation(steps: 1);"
    {
        return Err(SourceModeDefinitionError::InvalidProperty { index: 0 });
    }
    if !valid_site(
        &property.site,
        property.source_range,
        "source.definition.mode.property",
        local_context,
        arena,
    ) || !valid_node(
        TypedNodeId::new(46),
        range(source, 113, 134),
        "source.definition.mode.property.justification",
        local_context,
        arena,
    ) {
        return Err(SourceModeDefinitionError::InvalidArenaOwnership);
    }
    Ok(())
}

fn validate_context_profile(
    input: &SourceModeDefinitionHandoffInput,
    source_context: &SourceBindingContextHandoff,
    arena: &TypedArena,
) -> Result<LocalTypeContextId, SourceModeDefinitionError> {
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
        return Err(SourceModeDefinitionError::DependencyMismatch);
    }
    let item = source_context
        .items()
        .get(crate::source_context::SourceItemId::new(0))
        .ok_or(SourceModeDefinitionError::DependencyMismatch)?;
    let module_context = BindingContextId::new(0);
    let definition_context = BindingContextId::new(1);
    let module_local = LocalTypeContextId::new(0);
    let definition_local = LocalTypeContextId::new(1);
    let module_link = source_context
        .context_links()
        .get(module_context)
        .ok_or(SourceModeDefinitionError::DependencyMismatch)?;
    let definition_link = source_context
        .context_links()
        .get(definition_context)
        .ok_or(SourceModeDefinitionError::DependencyMismatch)?;
    if item.id != crate::source_context::SourceItemId::new(0)
        || item.shell.index() != 0
        || item.shell_ordinal != 0
        || item.role != SourceItemRole::DefinitionBlock
        || item.source_range != range(source, 0, 140)
        || item.parent.is_some()
        || item.visibility != SourceItemVisibility::Unspecified
        || item.site != TypedSiteRef::Node(TypedNodeId::new(50))
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
        return Err(SourceModeDefinitionError::DependencyMismatch);
    }
    if !valid_site(
        &item.site,
        item.source_range,
        "source.definition",
        definition_local,
        arena,
    ) {
        return Err(SourceModeDefinitionError::InvalidArenaOwnership);
    }
    for (index, row) in input.parameters.iter().enumerate() {
        let declaration = source_context
            .declarations()
            .get(crate::source_context::SourceDeclarationId::new(index))
            .ok_or(SourceModeDefinitionError::InvalidParameter { index })?;
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
            return Err(SourceModeDefinitionError::InvalidParameter { index });
        }
        let binding = binding_env
            .bindings()
            .get(row.binding)
            .ok_or(SourceModeDefinitionError::InvalidParameter { index })?;
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
            return Err(SourceModeDefinitionError::InvalidParameter { index });
        }
    }
    Ok(definition_local)
}

fn validate_type_profile(
    input: &SourceModeDefinitionHandoffInput,
    source_type: &SourceTypeApplicationHandoff,
    local_context: LocalTypeContextId,
    arena: &TypedArena,
) -> Result<(), SourceModeDefinitionError> {
    if source_type.applications().len() != 2
        || source_type.expressions().len() != 3
        || !source_type.arguments().is_empty()
        || !source_type.definition_returns().is_empty()
        || source_type.mode_rhs().len() != 1
    {
        return Err(SourceModeDefinitionError::DependencyMismatch);
    }
    let source = input.source_id;
    let ranges = [(22, 25), (38, 41), (95, 98)];
    let expression_sites = [35, 39, 44];
    let head_sites = [34, 38, 43];
    for index in 0..3 {
        let expression = source_type
            .expressions()
            .get(SourceTypeExpressionId::new(index))
            .ok_or(SourceModeDefinitionError::DependencyMismatch)?;
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
            return Err(SourceModeDefinitionError::DependencyMismatch);
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
            return Err(SourceModeDefinitionError::InvalidArenaOwnership);
        }
    }
    for index in 0..2 {
        let application = source_type
            .applications()
            .get(SourceTypeApplicationId::new(index))
            .ok_or(SourceModeDefinitionError::InvalidParameter { index })?;
        if application.id() != SourceTypeApplicationId::new(index)
            || application.binding() != input.parameters[index].binding
            || application.source_ordinal() != index
            || application.root() != SourceTypeExpressionId::new(index)
        {
            return Err(SourceModeDefinitionError::InvalidParameter { index });
        }
    }
    let rhs = source_type
        .mode_rhs()
        .get(SourceTypeModeRhsId::new(0))
        .ok_or(SourceModeDefinitionError::InvalidExpansion { index: 0 })?;
    if rhs.id() != SourceTypeModeRhsId::new(0)
        || rhs.definition_site() != &input.definitions[0].site
        || rhs.definition_range() != input.definitions[0].source_range
        || rhs.source_ordinal() != 0
        || rhs.root() != SourceTypeExpressionId::new(2)
    {
        return Err(SourceModeDefinitionError::InvalidExpansion { index: 0 });
    }
    Ok(())
}

fn validate_handoff_rows(
    handoff: &SourceModeDefinitionHandoff,
    source_context: &SourceBindingContextHandoff,
    source_type: &SourceTypeApplicationHandoff,
    initial_obligations: &InitialObligationTable,
    arena: &TypedArena,
) -> Result<(), SourceModeDefinitionError> {
    if handoff.definitions.len() != 1
        || handoff.parameters.len() != 2
        || handoff.applications.len() != 1
        || handoff.expansions.len() != 1
        || handoff.inhabitation_requests.len() != 1
        || handoff.properties.len() != 1
    {
        return Err(SourceModeDefinitionError::UnsupportedTaskShape);
    }
    validate_dense_ids(handoff)?;
    let definition = &handoff.definitions.rows[0];
    let identity = &handoff.resolver_identity;
    let expected_fqn = format!(
        "{}::{}::{}",
        handoff.module_id.package().as_str(),
        handoff.module_id.path().as_str(),
        definition.symbol.local().as_str()
    );
    if definition.symbol != identity.symbol
        || definition.definition != identity.definition
        || definition.contribution != identity.contribution
        || definition.origin != identity.origin
        || definition.symbol.module() != &handoff.module_id
        || definition.symbol.fqn().as_str() != expected_fqn
        || definition.definition.index() != 0
        || definition.contribution.index() != 0
        || !normal_origin(
            &definition.origin,
            handoff.source_id,
            &handoff.module_id,
            definition.source_range,
            &[4, 0, 10, 0],
        )
    {
        return Err(SourceModeDefinitionError::InvalidResolverDefinition { index: 0 });
    }
    let input = SourceModeDefinitionHandoffInput {
        source_id: handoff.source_id,
        module_id: handoff.module_id.clone(),
        definitions: handoff
            .definitions
            .iter()
            .map(|(_, row)| SourceModeDefinitionInput {
                symbol: row.symbol.clone(),
                definition: row.definition,
                contribution: row.contribution,
                site: row.site.clone(),
                source_range: row.source_range,
                source_ordinal: row.source_ordinal,
                context: row.context,
                recovery: row.recovery,
                spelling: row.spelling.clone(),
                application: row.application,
                expansion: row.expansion,
                inhabitation_request: row.inhabitation_request,
                property: row.property,
            })
            .collect(),
        parameters: handoff
            .parameters
            .iter()
            .map(|(_, row)| SourceModeParameterInput {
                owner: row.owner,
                ordinal: row.ordinal,
                binding: row.binding,
                written_type: row.written_type,
                site: row.site.clone(),
                source_range: row.source_range,
                declaration_range: row.declaration_range,
                pattern_range: row.pattern_range,
                context: row.context,
                recovery: row.recovery,
                spelling: row.spelling.clone(),
            })
            .collect(),
        applications: handoff
            .applications
            .iter()
            .map(|(_, row)| SourceModeApplicationInput {
                owner: row.owner,
                ordinal: row.ordinal,
                parameters: row.parameters.clone(),
                site: row.site.clone(),
                source_range: row.source_range,
                context: row.context,
                recovery: row.recovery,
                spelling: row.spelling.clone(),
            })
            .collect(),
        expansions: handoff
            .expansions
            .iter()
            .map(|(_, row)| SourceModeExpansionInput {
                owner: row.owner,
                ordinal: row.ordinal,
                rhs: row.rhs,
                site: row.site.clone(),
                source_range: row.source_range,
                context: row.context,
                recovery: row.recovery,
                spelling: row.spelling.clone(),
            })
            .collect(),
        inhabitation_requests: handoff
            .inhabitation_requests
            .iter()
            .map(|(_, row)| SourceModeInhabitationRequestInput {
                owner: row.owner,
                ordinal: row.ordinal,
                expansion: row.expansion,
                kind: row.kind,
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
            .map(|(_, row)| SourceModePropertyInput {
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
    validate_input_rows(&input, source_context, source_type, arena)?;
    validate_obligations(handoff, initial_obligations)
}

fn validate_dense_ids(
    handoff: &SourceModeDefinitionHandoff,
) -> Result<(), SourceModeDefinitionError> {
    for (index, row) in handoff.definitions.rows.iter().enumerate() {
        if row.id != SourceModeDefinitionId::new(index) {
            return Err(SourceModeDefinitionError::InvalidDefinition { index });
        }
    }
    for (index, row) in handoff.parameters.rows.iter().enumerate() {
        if row.id != SourceModeParameterId::new(index) {
            return Err(SourceModeDefinitionError::InvalidParameter { index });
        }
    }
    for (index, row) in handoff.applications.rows.iter().enumerate() {
        if row.id != SourceModeApplicationId::new(index) {
            return Err(SourceModeDefinitionError::InvalidApplication { index });
        }
    }
    for (index, row) in handoff.expansions.rows.iter().enumerate() {
        if row.id != SourceModeExpansionId::new(index) {
            return Err(SourceModeDefinitionError::InvalidExpansion { index });
        }
    }
    for (index, row) in handoff.inhabitation_requests.rows.iter().enumerate() {
        if row.id != SourceModeInhabitationRequestId::new(index) {
            return Err(SourceModeDefinitionError::InvalidInhabitationRequest { index });
        }
    }
    for (index, row) in handoff.properties.rows.iter().enumerate() {
        if row.id != SourceModePropertyId::new(index) {
            return Err(SourceModeDefinitionError::InvalidProperty { index });
        }
    }
    Ok(())
}

fn validate_obligations(
    handoff: &SourceModeDefinitionHandoff,
    table: &InitialObligationTable,
) -> Result<(), SourceModeDefinitionError> {
    let base_len = handoff.base_initial_obligation_count;
    if table.len() != base_len + 1 {
        return Err(SourceModeDefinitionError::InvalidObligation);
    }
    if table.iter().take(base_len).any(|(_, row)| {
        matches!(
            row.kind,
            InitialObligationKind::PredicatePropertyCorrectness
                | InitialObligationKind::FunctorExistence
                | InitialObligationKind::FunctorUniqueness
        ) || in_mode_definition_domain(row.goal.as_str())
            || in_mode_definition_domain(row.provenance.as_str())
    }) {
        return Err(SourceModeDefinitionError::InvalidObligation);
    }
    let property = &handoff.properties.rows[0];
    let expected_id = InitialObligationId::new(base_len);
    let obligation = table
        .get(expected_id)
        .ok_or(SourceModeDefinitionError::InvalidObligation)?;
    if property.obligation != expected_id
        || obligation.id != expected_id
        || obligation.kind != InitialObligationKind::Sethood
        || obligation.owner != property.site
        || obligation.source_range != property.source_range
        || !obligation.assumptions.is_empty()
        || obligation.goal.as_str() != "source.definition.mode.correctness:definition=0:sethood"
        || obligation.provenance.as_str() != "source.definition.mode:definition=0:property=0"
        || obligation.status != InitialObligationStatus::Pending
    {
        return Err(SourceModeDefinitionError::InvalidObligation);
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

fn in_mode_definition_domain(value: &str) -> bool {
    value == "source.definition.mode"
        || value.starts_with("source.definition.mode:")
        || value.starts_with("source.definition.mode.")
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
    valid_node(*node_id, source_range, kind, local_context, arena)
}

fn valid_node(
    node_id: TypedNodeId,
    source_range: SourceRange,
    kind: &str,
    local_context: LocalTypeContextId,
    arena: &TypedArena,
) -> bool {
    arena.node(node_id).is_some_and(|node| {
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

const fn request_key(kind: SourceModeInhabitationRequestKind) -> &'static str {
    match kind {
        SourceModeInhabitationRequestKind::Rhs => "rhs",
    }
}

const fn property_key(kind: SourceModePropertyKind) -> &'static str {
    match kind {
        SourceModePropertyKind::Sethood => "sethood",
    }
}

const fn recovery_key(recovery: SourceModeDefinitionRecovery) -> &'static str {
    match recovery {
        SourceModeDefinitionRecovery::Normal => "normal",
        SourceModeDefinitionRecovery::Degraded => "degraded",
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
#[path = "../tests/support/source_mode_definition_unit.rs"]
pub(crate) mod tests;
