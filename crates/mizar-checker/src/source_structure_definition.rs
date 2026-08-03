//! Syntax-free structure-definition intake for checker phase 6.

use crate::{
    source_mode_definition::validate_source_mode_definition_absence,
    source_type::{
        SourceTypeApplicationForm, SourceTypeApplicationHandoff, SourceTypeExpressionId,
        SourceTypeHead, SourceTypeStructureMemberId,
    },
    typed_ast::{
        InitialObligationKind, InitialObligationTable, NodeRecoveryState, TypedArena, TypedNodeId,
        TypedSiteRef,
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
    collections::BTreeSet,
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

dense_id!(SourceStructureDefinitionId);
dense_id!(SourceStructureMemberId);
dense_id!(SourceStructureInheritanceId);
dense_id!(SourceStructureMappingId);
dense_id!(SourceStructureCoherenceRequestId);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStructureDefinitionHandoffInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub definitions: Vec<SourceStructureDefinitionInput>,
    pub members: Vec<SourceStructureMemberInput>,
    pub inheritances: Vec<SourceStructureInheritanceInput>,
    pub mappings: Vec<SourceStructureMappingInput>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStructureDefinitionInput {
    pub symbol: SymbolId,
    pub definition: DefinitionId,
    pub contribution: SourceContributionId,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub source_ordinal: usize,
    pub recovery: SourceStructureDefinitionRecovery,
    pub spelling: String,
    pub members: Vec<SourceStructureMemberId>,
    pub constructor_fields: Vec<SourceStructureMemberId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStructureMemberInput {
    pub symbol: SymbolId,
    pub definition: DefinitionId,
    pub contribution: SourceContributionId,
    pub owner: SourceStructureDefinitionId,
    pub ordinal: usize,
    pub kind: SourceStructureMemberKind,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub recovery: SourceStructureDefinitionRecovery,
    pub spelling: String,
    pub written_type: SourceTypeStructureMemberId,
    pub constructor_ordinal: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStructureInheritanceInput {
    pub child: SourceStructureDefinitionId,
    pub parent: SourceStructureDefinitionId,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub source_ordinal: usize,
    pub recovery: SourceStructureDefinitionRecovery,
    pub spelling: String,
    pub mappings: Vec<SourceStructureMappingId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStructureMappingInput {
    pub symbol: SymbolId,
    pub definition: DefinitionId,
    pub contribution: SourceContributionId,
    pub inheritance: SourceStructureInheritanceId,
    pub ordinal: usize,
    pub kind: SourceStructureMemberKind,
    pub view_member: SourceStructureMemberId,
    pub parent_member: SourceStructureMemberId,
    pub root_member: SourceStructureMemberId,
    pub path: Vec<SourceStructureInheritanceId>,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub recovery: SourceStructureDefinitionRecovery,
    pub spelling: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceStructureMemberKind {
    Field,
    Property,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceStructureCoherenceRequestKind {
    MemberTypeInclusion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceStructureDefinitionRecovery {
    Normal,
    Degraded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStructureDefinition {
    id: SourceStructureDefinitionId,
    symbol: SymbolId,
    definition: DefinitionId,
    contribution: SourceContributionId,
    site: TypedSiteRef,
    source_range: SourceRange,
    source_ordinal: usize,
    recovery: SourceStructureDefinitionRecovery,
    spelling: String,
    members: Vec<SourceStructureMemberId>,
    constructor_fields: Vec<SourceStructureMemberId>,
    origin: SemanticOrigin,
}

impl SourceStructureDefinition {
    pub const fn id(&self) -> SourceStructureDefinitionId {
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
    pub const fn recovery(&self) -> SourceStructureDefinitionRecovery {
        self.recovery
    }
    pub fn spelling(&self) -> &str {
        &self.spelling
    }
    pub fn members(&self) -> &[SourceStructureMemberId] {
        &self.members
    }
    pub fn constructor_fields(&self) -> &[SourceStructureMemberId] {
        &self.constructor_fields
    }
    pub const fn origin(&self) -> &SemanticOrigin {
        &self.origin
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStructureMember {
    id: SourceStructureMemberId,
    symbol: SymbolId,
    definition: DefinitionId,
    contribution: SourceContributionId,
    owner: SourceStructureDefinitionId,
    ordinal: usize,
    kind: SourceStructureMemberKind,
    site: TypedSiteRef,
    source_range: SourceRange,
    recovery: SourceStructureDefinitionRecovery,
    spelling: String,
    written_type: SourceTypeStructureMemberId,
    constructor_ordinal: Option<usize>,
    origin: SemanticOrigin,
}

impl SourceStructureMember {
    pub const fn id(&self) -> SourceStructureMemberId {
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
    pub const fn owner(&self) -> SourceStructureDefinitionId {
        self.owner
    }
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }
    pub const fn kind(&self) -> SourceStructureMemberKind {
        self.kind
    }
    pub const fn site(&self) -> &TypedSiteRef {
        &self.site
    }
    pub const fn source_range(&self) -> SourceRange {
        self.source_range
    }
    pub const fn recovery(&self) -> SourceStructureDefinitionRecovery {
        self.recovery
    }
    pub fn spelling(&self) -> &str {
        &self.spelling
    }
    pub const fn written_type(&self) -> SourceTypeStructureMemberId {
        self.written_type
    }
    pub const fn constructor_ordinal(&self) -> Option<usize> {
        self.constructor_ordinal
    }
    pub const fn origin(&self) -> &SemanticOrigin {
        &self.origin
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStructureInheritance {
    id: SourceStructureInheritanceId,
    child: SourceStructureDefinitionId,
    parent: SourceStructureDefinitionId,
    site: TypedSiteRef,
    source_range: SourceRange,
    source_ordinal: usize,
    recovery: SourceStructureDefinitionRecovery,
    spelling: String,
    mappings: Vec<SourceStructureMappingId>,
}

impl SourceStructureInheritance {
    pub const fn id(&self) -> SourceStructureInheritanceId {
        self.id
    }
    pub const fn child(&self) -> SourceStructureDefinitionId {
        self.child
    }
    pub const fn parent(&self) -> SourceStructureDefinitionId {
        self.parent
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
    pub const fn recovery(&self) -> SourceStructureDefinitionRecovery {
        self.recovery
    }
    pub fn spelling(&self) -> &str {
        &self.spelling
    }
    pub fn mappings(&self) -> &[SourceStructureMappingId] {
        &self.mappings
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStructureMapping {
    id: SourceStructureMappingId,
    symbol: SymbolId,
    definition: DefinitionId,
    contribution: SourceContributionId,
    inheritance: SourceStructureInheritanceId,
    ordinal: usize,
    kind: SourceStructureMemberKind,
    view_member: SourceStructureMemberId,
    parent_member: SourceStructureMemberId,
    root_member: SourceStructureMemberId,
    path: Vec<SourceStructureInheritanceId>,
    site: TypedSiteRef,
    source_range: SourceRange,
    recovery: SourceStructureDefinitionRecovery,
    spelling: String,
    origin: SemanticOrigin,
}

impl SourceStructureMapping {
    pub const fn id(&self) -> SourceStructureMappingId {
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
    pub const fn inheritance(&self) -> SourceStructureInheritanceId {
        self.inheritance
    }
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }
    pub const fn kind(&self) -> SourceStructureMemberKind {
        self.kind
    }
    pub const fn view_member(&self) -> SourceStructureMemberId {
        self.view_member
    }
    pub const fn parent_member(&self) -> SourceStructureMemberId {
        self.parent_member
    }
    pub const fn root_member(&self) -> SourceStructureMemberId {
        self.root_member
    }
    pub fn path(&self) -> &[SourceStructureInheritanceId] {
        &self.path
    }
    pub const fn site(&self) -> &TypedSiteRef {
        &self.site
    }
    pub const fn source_range(&self) -> SourceRange {
        self.source_range
    }
    pub const fn recovery(&self) -> SourceStructureDefinitionRecovery {
        self.recovery
    }
    pub fn spelling(&self) -> &str {
        &self.spelling
    }
    pub const fn origin(&self) -> &SemanticOrigin {
        &self.origin
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStructureCoherenceRequest {
    id: SourceStructureCoherenceRequestId,
    mapping: SourceStructureMappingId,
    kind: SourceStructureCoherenceRequestKind,
    site: TypedSiteRef,
    source_range: SourceRange,
}

impl SourceStructureCoherenceRequest {
    pub const fn id(&self) -> SourceStructureCoherenceRequestId {
        self.id
    }
    pub const fn mapping(&self) -> SourceStructureMappingId {
        self.mapping
    }
    pub const fn kind(&self) -> SourceStructureCoherenceRequestKind {
        self.kind
    }
    pub const fn site(&self) -> &TypedSiteRef {
        &self.site
    }
    pub const fn source_range(&self) -> SourceRange {
        self.source_range
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
                self.rows.iter().map(|row| (row.id, row))
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
    SourceStructureDefinitionTable,
    SourceStructureDefinition,
    SourceStructureDefinitionId
);
table!(
    SourceStructureMemberTable,
    SourceStructureMember,
    SourceStructureMemberId
);
table!(
    SourceStructureInheritanceTable,
    SourceStructureInheritance,
    SourceStructureInheritanceId
);
table!(
    SourceStructureMappingTable,
    SourceStructureMapping,
    SourceStructureMappingId
);
table!(
    SourceStructureCoherenceRequestTable,
    SourceStructureCoherenceRequest,
    SourceStructureCoherenceRequestId
);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStructureDefinitionHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    source_type_fingerprint: String,
    base_initial_obligation_count: usize,
    base_initial_obligations_snapshot: InitialObligationTable,
    resolver_identity_snapshot: Vec<(SymbolId, DefinitionId, SourceContributionId)>,
    definitions: SourceStructureDefinitionTable,
    members: SourceStructureMemberTable,
    inheritances: SourceStructureInheritanceTable,
    mappings: SourceStructureMappingTable,
    coherence_requests: SourceStructureCoherenceRequestTable,
}

impl SourceStructureDefinitionHandoff {
    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }
    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
    }
    pub fn source_type_fingerprint(&self) -> &str {
        &self.source_type_fingerprint
    }
    pub const fn base_initial_obligation_count(&self) -> usize {
        self.base_initial_obligation_count
    }
    pub const fn definitions(&self) -> &SourceStructureDefinitionTable {
        &self.definitions
    }
    pub const fn members(&self) -> &SourceStructureMemberTable {
        &self.members
    }
    pub const fn inheritances(&self) -> &SourceStructureInheritanceTable {
        &self.inheritances
    }
    pub const fn mappings(&self) -> &SourceStructureMappingTable {
        &self.mappings
    }
    pub const fn coherence_requests(&self) -> &SourceStructureCoherenceRequestTable {
        &self.coherence_requests
    }

    pub fn debug_text(&self) -> String {
        let mut output = String::from("source-structure-definition-debug-v1\n");
        let _ = writeln!(output, "module: {}", self.module_id.path().as_str());
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
        let _ = writeln!(
            output,
            "profile: definitions={} members={} inheritances={} mappings={} coherence_requests={}",
            self.definitions.len(),
            self.members.len(),
            self.inheritances.len(),
            self.mappings.len(),
            self.coherence_requests.len()
        );
        for (id, row) in self.definitions.iter() {
            let members = row.members.iter().map(|id| id.index()).collect::<Vec<_>>();
            let fields = row
                .constructor_fields
                .iter()
                .map(|id| id.index())
                .collect::<Vec<_>>();
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
            let _ = writeln!(
                output,
                " recovery={} origin_range={}..{} origin_path={:?} spelling={:?} members={:?} constructor_fields={:?}",
                recovery_key(row.recovery),
                anchor_start(row.origin.anchor()),
                anchor_end(row.origin.anchor()),
                row.origin.structural_path(),
                row.spelling,
                members,
                fields
            );
        }
        for (id, row) in self.members.iter() {
            let constructor = row
                .constructor_ordinal
                .map_or_else(|| "none".to_owned(), |value| value.to_string());
            let _ = write!(
                output,
                "member#{} symbol={:?} definition={} contribution={} owner={} ordinal={} kind={} range={}..{} site=",
                id.index(),
                row.symbol.fqn().as_str(),
                row.definition.index(),
                row.contribution.index(),
                row.owner.index(),
                row.ordinal,
                member_kind_key(row.kind),
                row.source_range.start,
                row.source_range.end
            );
            write_site(&mut output, &row.site);
            let _ = writeln!(
                output,
                " recovery={} origin_range={}..{} origin_path={:?} spelling={:?} written_type={} constructor_ordinal={}",
                recovery_key(row.recovery),
                anchor_start(row.origin.anchor()),
                anchor_end(row.origin.anchor()),
                row.origin.structural_path(),
                row.spelling,
                row.written_type.index(),
                constructor
            );
        }
        for (id, row) in self.inheritances.iter() {
            let mappings = row.mappings.iter().map(|id| id.index()).collect::<Vec<_>>();
            let _ = write!(
                output,
                "inheritance#{} child={} parent={} ordinal={} range={}..{} site=",
                id.index(),
                row.child.index(),
                row.parent.index(),
                row.source_ordinal,
                row.source_range.start,
                row.source_range.end
            );
            write_site(&mut output, &row.site);
            let _ = writeln!(
                output,
                " recovery={} spelling={:?} mappings={:?}",
                recovery_key(row.recovery),
                row.spelling,
                mappings
            );
        }
        for (id, row) in self.mappings.iter() {
            let path = row.path.iter().map(|id| id.index()).collect::<Vec<_>>();
            let _ = write!(
                output,
                "mapping#{} symbol={:?} definition={} contribution={} inheritance={} ordinal={} kind={} view_member={} parent_member={} root_member={} path={:?} range={}..{} site=",
                id.index(),
                row.symbol.fqn().as_str(),
                row.definition.index(),
                row.contribution.index(),
                row.inheritance.index(),
                row.ordinal,
                member_kind_key(row.kind),
                row.view_member.index(),
                row.parent_member.index(),
                row.root_member.index(),
                path,
                row.source_range.start,
                row.source_range.end
            );
            write_site(&mut output, &row.site);
            let _ = writeln!(
                output,
                " recovery={} origin_range={}..{} origin_path={:?} spelling={:?}",
                recovery_key(row.recovery),
                anchor_start(row.origin.anchor()),
                anchor_end(row.origin.anchor()),
                row.origin.structural_path(),
                row.spelling
            );
        }
        for (id, row) in self.coherence_requests.iter() {
            let _ = write!(
                output,
                "coherence-request#{} mapping={} kind={} range={}..{} site=",
                id.index(),
                row.mapping.index(),
                coherence_kind_key(row.kind),
                row.source_range.start,
                row.source_range.end
            );
            write_site(&mut output, &row.site);
            output.push('\n');
        }
        output
    }

    pub(crate) fn validate_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        source_type: &SourceTypeApplicationHandoff,
        initial_obligations: &InitialObligationTable,
        arena: &TypedArena,
    ) -> Result<(), SourceStructureDefinitionError> {
        if self.source_id != source_id
            || &self.module_id != module_id
            || source_type.source_id() != source_id
            || source_type.module_id() != module_id
        {
            return Err(SourceStructureDefinitionError::SourceIdentityMismatch);
        }
        if self.source_type_fingerprint != source_type.debug_text() {
            return Err(SourceStructureDefinitionError::DependencyMismatch);
        }
        validate_handoff_rows(self, source_type)?;
        if initial_obligations != &self.base_initial_obligations_snapshot
            || initial_obligations.len() != self.base_initial_obligation_count
        {
            return Err(SourceStructureDefinitionError::InvalidObligation);
        }
        validate_baseline(initial_obligations)?;
        validate_arena(self, source_type, arena)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStructureDefinitionProjection {
    base_initial_obligations: InitialObligationTable,
    handoff: SourceStructureDefinitionHandoff,
    initial_obligations: InitialObligationTable,
}

impl SourceStructureDefinitionProjection {
    pub const fn base_initial_obligations(&self) -> &InitialObligationTable {
        &self.base_initial_obligations
    }
    pub const fn handoff(&self) -> &SourceStructureDefinitionHandoff {
        &self.handoff
    }
    pub const fn initial_obligations(&self) -> &InitialObligationTable {
        &self.initial_obligations
    }
    pub fn into_parts(
        self,
    ) -> (
        InitialObligationTable,
        SourceStructureDefinitionHandoff,
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
pub enum SourceStructureDefinitionError {
    SourceIdentityMismatch,
    DependencyMismatch,
    InvalidResolverDefinition { index: usize },
    InvalidDefinition { index: usize },
    InvalidMember { index: usize },
    InvalidInheritance { index: usize },
    InvalidMapping { index: usize },
    InvalidCoherenceRequest { index: usize },
    InvalidObligation,
    InvalidArenaOwnership,
    UnsupportedTaskShape,
}

impl fmt::Display for SourceStructureDefinitionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SourceIdentityMismatch => {
                formatter.write_str("structure-definition source identity mismatch")
            }
            Self::DependencyMismatch => {
                formatter.write_str("structure-definition dependency mismatch")
            }
            Self::InvalidResolverDefinition { index } => {
                write!(formatter, "invalid structure resolver definition {index}")
            }
            Self::InvalidDefinition { index } => {
                write!(formatter, "invalid source structure definition {index}")
            }
            Self::InvalidMember { index } => {
                write!(formatter, "invalid source structure member {index}")
            }
            Self::InvalidInheritance { index } => {
                write!(formatter, "invalid source structure inheritance {index}")
            }
            Self::InvalidMapping { index } => {
                write!(formatter, "invalid source structure mapping {index}")
            }
            Self::InvalidCoherenceRequest { index } => write!(
                formatter,
                "invalid source structure coherence request {index}"
            ),
            Self::InvalidObligation => {
                formatter.write_str("invalid structure-definition obligation baseline")
            }
            Self::InvalidArenaOwnership => {
                formatter.write_str("invalid structure-definition typed-arena ownership")
            }
            Self::UnsupportedTaskShape => {
                formatter.write_str("unsupported structure-definition task shape")
            }
        }
    }
}

impl Error for SourceStructureDefinitionError {}

pub struct SourceStructureDefinitionProducer;

impl SourceStructureDefinitionProducer {
    pub fn build(
        input: SourceStructureDefinitionHandoffInput,
        env: &SymbolEnv,
        source_type: &SourceTypeApplicationHandoff,
        base_initial_obligations: &InitialObligationTable,
        arena: &TypedArena,
    ) -> Result<SourceStructureDefinitionProjection, SourceStructureDefinitionError> {
        if env.module_id() != &input.module_id
            || source_type.source_id() != input.source_id
            || source_type.module_id() != &input.module_id
        {
            return Err(SourceStructureDefinitionError::SourceIdentityMismatch);
        }
        validate_shape(&input)?;
        let (definition_origins, member_origins, mapping_origins) = validate_resolver(&input, env)?;
        validate_definition_inputs(&input)?;
        validate_member_inputs(&input)?;
        validate_inheritance_inputs(&input)?;
        validate_mapping_inputs(&input)?;
        validate_lower(source_type, &input)?;
        validate_baseline(base_initial_obligations)?;
        let resolver_identity_snapshot = input
            .definitions
            .iter()
            .map(|row| (row.symbol.clone(), row.definition, row.contribution))
            .chain(
                input
                    .members
                    .iter()
                    .map(|row| (row.symbol.clone(), row.definition, row.contribution)),
            )
            .chain(
                input
                    .mappings
                    .iter()
                    .map(|row| (row.symbol.clone(), row.definition, row.contribution)),
            )
            .collect();

        let handoff = SourceStructureDefinitionHandoff {
            source_id: input.source_id,
            module_id: input.module_id,
            source_type_fingerprint: source_type.debug_text(),
            base_initial_obligation_count: base_initial_obligations.len(),
            base_initial_obligations_snapshot: base_initial_obligations.clone(),
            resolver_identity_snapshot,
            definitions: SourceStructureDefinitionTable {
                rows: input
                    .definitions
                    .into_iter()
                    .enumerate()
                    .map(|(index, row)| SourceStructureDefinition {
                        id: SourceStructureDefinitionId::new(index),
                        symbol: row.symbol,
                        definition: row.definition,
                        contribution: row.contribution,
                        site: row.site,
                        source_range: row.source_range,
                        source_ordinal: row.source_ordinal,
                        recovery: row.recovery,
                        spelling: row.spelling,
                        members: row.members,
                        constructor_fields: row.constructor_fields,
                        origin: definition_origins[index].clone(),
                    })
                    .collect(),
            },
            members: SourceStructureMemberTable {
                rows: input
                    .members
                    .into_iter()
                    .enumerate()
                    .map(|(index, row)| SourceStructureMember {
                        id: SourceStructureMemberId::new(index),
                        symbol: row.symbol,
                        definition: row.definition,
                        contribution: row.contribution,
                        owner: row.owner,
                        ordinal: row.ordinal,
                        kind: row.kind,
                        site: row.site,
                        source_range: row.source_range,
                        recovery: row.recovery,
                        spelling: row.spelling,
                        written_type: row.written_type,
                        constructor_ordinal: row.constructor_ordinal,
                        origin: member_origins[index].clone(),
                    })
                    .collect(),
            },
            inheritances: SourceStructureInheritanceTable {
                rows: input
                    .inheritances
                    .into_iter()
                    .enumerate()
                    .map(|(index, row)| SourceStructureInheritance {
                        id: SourceStructureInheritanceId::new(index),
                        child: row.child,
                        parent: row.parent,
                        site: row.site,
                        source_range: row.source_range,
                        source_ordinal: row.source_ordinal,
                        recovery: row.recovery,
                        spelling: row.spelling,
                        mappings: row.mappings,
                    })
                    .collect(),
            },
            mappings: SourceStructureMappingTable {
                rows: input
                    .mappings
                    .into_iter()
                    .enumerate()
                    .map(|(index, row)| SourceStructureMapping {
                        id: SourceStructureMappingId::new(index),
                        symbol: row.symbol,
                        definition: row.definition,
                        contribution: row.contribution,
                        inheritance: row.inheritance,
                        ordinal: row.ordinal,
                        kind: row.kind,
                        view_member: row.view_member,
                        parent_member: row.parent_member,
                        root_member: row.root_member,
                        path: row.path,
                        site: row.site,
                        source_range: row.source_range,
                        recovery: row.recovery,
                        spelling: row.spelling,
                        origin: mapping_origins[index].clone(),
                    })
                    .collect(),
            },
            coherence_requests: SourceStructureCoherenceRequestTable { rows: Vec::new() },
        };
        validate_arena(&handoff, source_type, arena)?;
        let initial_obligations = base_initial_obligations.clone();
        Ok(SourceStructureDefinitionProjection {
            base_initial_obligations: base_initial_obligations.clone(),
            handoff,
            initial_obligations,
        })
    }
}

fn validate_shape(
    input: &SourceStructureDefinitionHandoffInput,
) -> Result<(), SourceStructureDefinitionError> {
    if input.definitions.len() != 2
        || input.members.len() != 4
        || input.inheritances.len() != 1
        || input.mappings.len() != 2
    {
        return Err(SourceStructureDefinitionError::UnsupportedTaskShape);
    }
    Ok(())
}

type ResolverOrigins = (
    Vec<SemanticOrigin>,
    Vec<SemanticOrigin>,
    Vec<SemanticOrigin>,
);

fn validate_resolver(
    input: &SourceStructureDefinitionHandoffInput,
    env: &SymbolEnv,
) -> Result<ResolverOrigins, SourceStructureDefinitionError> {
    if env.symbols().len() != 8 || env.definitions().len() != 8 || env.contributions().len() != 1 {
        return Err(SourceStructureDefinitionError::InvalidResolverDefinition { index: 0 });
    }
    let expected = [
        (
            0,
            SymbolKind::Structure,
            DefinitionKind::Structure,
            "Task263Base",
            Some("Task263Base"),
            13,
            98,
            &[4, 0, 11, 0][..],
        ),
        (
            3,
            SymbolKind::Structure,
            DefinitionKind::Structure,
            "Task263Derived",
            Some("Task263Derived"),
            102,
            190,
            &[4, 0, 11, 1][..],
        ),
    ];
    let mut definition_origins = Vec::new();
    for (index, row) in input.definitions.iter().enumerate() {
        let (definition, symbol_kind, definition_kind, spelling, notation, start, end, path) =
            expected[index];
        definition_origins.push(validate_resolver_row(
            input,
            env,
            row.symbol.clone(),
            row.definition,
            row.contribution,
            definition,
            symbol_kind,
            definition_kind,
            spelling,
            notation,
            start,
            end,
            path,
            index,
        )?);
    }
    let member_expected = [
        (1, "carrier", 42, 63, &[4, 0, 11, 0, 18, 0][..]),
        (2, "marker", 68, 91, &[4, 0, 11, 0, 19, 1][..]),
        (4, "carrier", 134, 155, &[4, 0, 11, 1, 18, 0][..]),
        (5, "marker", 160, 183, &[4, 0, 11, 1, 19, 1][..]),
    ];
    let mut member_origins = Vec::new();
    for (index, row) in input.members.iter().enumerate() {
        let (definition, spelling, start, end, path) = member_expected[index];
        member_origins.push(validate_resolver_row(
            input,
            env,
            row.symbol.clone(),
            row.definition,
            row.contribution,
            definition,
            SymbolKind::Selector,
            DefinitionKind::Selector,
            spelling,
            None,
            start,
            end,
            path,
            index + 2,
        )?);
    }
    let mapping_expected = [
        (
            6,
            "carrier",
            "field carrier from carrier ;",
            247,
            274,
            &[4, 0, 20, 2, 21, 0][..],
        ),
        (
            7,
            "marker",
            "property marker from marker ;",
            279,
            307,
            &[4, 0, 20, 2, 22, 1][..],
        ),
    ];
    let mut mapping_origins = Vec::new();
    for (index, row) in input.mappings.iter().enumerate() {
        let (definition, spelling, notation, start, end, path) = mapping_expected[index];
        mapping_origins.push(validate_resolver_row(
            input,
            env,
            row.symbol.clone(),
            row.definition,
            row.contribution,
            definition,
            SymbolKind::Redefinition,
            DefinitionKind::Redefinition,
            spelling,
            Some(notation),
            start,
            end,
            path,
            index + 6,
        )?);
    }
    let contribution = env
        .contributions()
        .get(input.definitions[0].contribution)
        .ok_or(SourceStructureDefinitionError::InvalidResolverDefinition { index: 0 })?;
    if contribution.module() != &input.module_id
        || contribution.kind()
            != &(ContributionKind::LocalSource {
                source_id: input.source_id,
            })
        || contribution.anchor() != &SourceAnchor::Range(range(input.source_id, 0, 319))
        || contribution.effects().symbols().len() != 8
        || contribution.effects().definitions().len() != 8
        || !contribution.effects().overload_groups().is_empty()
        || !contribution.effects().registrations().is_empty()
        || !contribution.effects().lexical_summaries().is_empty()
        || !contribution.effects().labels().is_empty()
        || !contribution.effects().namespace_edges().is_empty()
        || !contribution.effects().declaration_dependencies().is_empty()
        || !contribution.effects().imports().is_empty()
        || !contribution.effects().exports().is_empty()
        || !contribution.effects().diagnostics().is_empty()
    {
        return Err(SourceStructureDefinitionError::InvalidResolverDefinition { index: 0 });
    }
    Ok((definition_origins, member_origins, mapping_origins))
}

// Rationale: one frozen resolver row is authenticated against its complete identity tuple.
#[allow(clippy::too_many_arguments)]
fn validate_resolver_row(
    input: &SourceStructureDefinitionHandoffInput,
    env: &SymbolEnv,
    symbol_id: SymbolId,
    definition_id: DefinitionId,
    contribution_id: SourceContributionId,
    expected_definition: usize,
    symbol_kind: SymbolKind,
    definition_kind: DefinitionKind,
    spelling: &str,
    notation: Option<&str>,
    start: usize,
    end: usize,
    path: &[u32],
    error_index: usize,
) -> Result<SemanticOrigin, SourceStructureDefinitionError> {
    let error = SourceStructureDefinitionError::InvalidResolverDefinition { index: error_index };
    let symbol = env.symbols().get(&symbol_id).ok_or_else(|| error.clone())?;
    let definition = env
        .definitions()
        .get(definition_id)
        .ok_or_else(|| error.clone())?;
    let contribution = env
        .contributions()
        .get(contribution_id)
        .ok_or_else(|| error.clone())?;
    let expected_origin = SourceAnchor::Range(range(input.source_id, start, end));
    if definition_id.index() != expected_definition
        || contribution_id.index() != 0
        || symbol_id.module() != &input.module_id
        || symbol.symbol() != &symbol_id
        || symbol.kind() != symbol_kind
        || symbol.visibility() != Visibility::Public
        || symbol.export_status() != ExportStatus::Exported
        || symbol.primary_spelling() != spelling
        || symbol.notation_spelling() != notation
        || symbol.contribution() != contribution_id
        || !contribution.effects().symbols().contains(&symbol_id)
        || !contribution
            .effects()
            .definitions()
            .contains(&definition_id)
        || symbol.origin() != definition.origin()
        || symbol.signature() != definition.signature()
        || !symbol.relations().is_empty()
        || definition.id() != definition_id
        || definition.symbol() != &symbol_id
        || definition.kind() != definition_kind
        || definition.visibility() != Visibility::Public
        || !definition.parameters().is_empty()
        || !definition.binders().is_empty()
        || definition.arity().is_some()
        || definition.notation_shape() != notation
        || definition.doc_attachment().is_some()
        || definition.contribution() != contribution_id
        || definition.conflict().is_some()
        || !definition.dependencies().is_empty()
        || definition.origin().source_id() != input.source_id
        || definition.origin().module_id() != &input.module_id
        || definition.origin().anchor() != &expected_origin
        || definition.origin().structural_path() != path
        || definition.origin().import_edge().is_some()
        || definition.origin().is_recovered()
    {
        return Err(error);
    }
    Ok(definition.origin().clone())
}

fn validate_definition_inputs(
    input: &SourceStructureDefinitionHandoffInput,
) -> Result<(), SourceStructureDefinitionError> {
    let expected = [
        (
            57,
            13,
            98,
            0,
            "struct Task263Base where\n    field carrier -> set;\n    property marker -> set;\n  end;",
            &[0, 1][..],
            &[0][..],
        ),
        (
            65,
            102,
            190,
            1,
            "struct Task263Derived where\n    field carrier -> set;\n    property marker -> set;\n  end;",
            &[2, 3][..],
            &[2][..],
        ),
    ];
    for (index, row) in input.definitions.iter().enumerate() {
        let (node, start, end, ordinal, spelling, members, fields) = expected[index];
        if row.site != node_site(node)
            || row.source_range != range(input.source_id, start, end)
            || row.source_ordinal != ordinal
            || row.recovery != SourceStructureDefinitionRecovery::Normal
            || row.spelling != spelling
            || row
                .members
                .iter()
                .map(|id| id.index())
                .ne(members.iter().copied())
            || row
                .constructor_fields
                .iter()
                .map(|id| id.index())
                .ne(fields.iter().copied())
        {
            return Err(SourceStructureDefinitionError::InvalidDefinition { index });
        }
    }
    Ok(())
}

fn validate_member_inputs(
    input: &SourceStructureDefinitionHandoffInput,
) -> Result<(), SourceStructureDefinitionError> {
    let expected = [
        (
            0,
            0,
            SourceStructureMemberKind::Field,
            53,
            42,
            63,
            "field carrier -> set;",
            0,
            Some(0),
        ),
        (
            0,
            1,
            SourceStructureMemberKind::Property,
            56,
            68,
            91,
            "property marker -> set;",
            1,
            None,
        ),
        (
            1,
            0,
            SourceStructureMemberKind::Field,
            61,
            134,
            155,
            "field carrier -> set;",
            2,
            Some(0),
        ),
        (
            1,
            1,
            SourceStructureMemberKind::Property,
            64,
            160,
            183,
            "property marker -> set;",
            3,
            None,
        ),
    ];
    for (index, row) in input.members.iter().enumerate() {
        let (owner, ordinal, kind, node, start, end, spelling, written_type, constructor) =
            expected[index];
        if row.owner.index() != owner
            || row.ordinal != ordinal
            || row.kind != kind
            || row.site != node_site(node)
            || row.source_range != range(input.source_id, start, end)
            || row.recovery != SourceStructureDefinitionRecovery::Normal
            || row.spelling != spelling
            || row.written_type.index() != written_type
            || row.constructor_ordinal != constructor
        {
            return Err(SourceStructureDefinitionError::InvalidMember { index });
        }
    }
    Ok(())
}

fn validate_inheritance_inputs(
    input: &SourceStructureDefinitionHandoffInput,
) -> Result<(), SourceStructureDefinitionError> {
    let row = &input.inheritances[0];
    if row.child != SourceStructureDefinitionId::new(1)
        || row.parent != SourceStructureDefinitionId::new(0)
        || row.child == row.parent
        || row.site != node_site(70)
        || row.source_range != range(input.source_id, 194, 314)
        || row.source_ordinal != 0
        || row.recovery != SourceStructureDefinitionRecovery::Normal
        || row.spelling
            != "inherit Task263Derived extends Task263Base where\n    field carrier from carrier;\n    property marker from marker;\n  end;"
        || row.mappings
            != [
                SourceStructureMappingId::new(0),
                SourceStructureMappingId::new(1),
            ]
    {
        return Err(SourceStructureDefinitionError::InvalidInheritance { index: 0 });
    }
    Ok(())
}

fn validate_mapping_inputs(
    input: &SourceStructureDefinitionHandoffInput,
) -> Result<(), SourceStructureDefinitionError> {
    let expected = [
        (
            SourceStructureMemberKind::Field,
            2,
            0,
            0,
            68,
            247,
            274,
            "field carrier from carrier;",
        ),
        (
            SourceStructureMemberKind::Property,
            3,
            1,
            1,
            69,
            279,
            307,
            "property marker from marker;",
        ),
    ];
    let mut parent_coverage = BTreeSet::new();
    let mut view_coverage = BTreeSet::new();
    for (index, row) in input.mappings.iter().enumerate() {
        let (kind, view, parent, root, node, start, end, spelling) = expected[index];
        if row.inheritance != SourceStructureInheritanceId::new(0)
            || row.ordinal != index
            || row.kind != kind
            || row.view_member.index() != view
            || row.parent_member.index() != parent
            || row.root_member.index() != root
            || row.path != [SourceStructureInheritanceId::new(0)]
            || row.site != node_site(node)
            || row.source_range != range(input.source_id, start, end)
            || row.recovery != SourceStructureDefinitionRecovery::Normal
            || row.spelling != spelling
            || !parent_coverage.insert(parent)
            || !view_coverage.insert(view)
        {
            return Err(SourceStructureDefinitionError::InvalidMapping { index });
        }
        if input.members[view].owner != SourceStructureDefinitionId::new(1)
            || input.members[parent].owner != SourceStructureDefinitionId::new(0)
            || input.members[root].owner != SourceStructureDefinitionId::new(0)
            || input.members[view].kind != input.members[parent].kind
            || input.members[parent].kind != input.members[root].kind
        {
            return Err(SourceStructureDefinitionError::InvalidMapping { index });
        }
    }
    if parent_coverage != BTreeSet::from([0, 1]) || view_coverage != BTreeSet::from([2, 3]) {
        return Err(SourceStructureDefinitionError::InvalidMapping { index: 0 });
    }
    Ok(())
}

fn validate_lower(
    source_type: &SourceTypeApplicationHandoff,
    input: &SourceStructureDefinitionHandoffInput,
) -> Result<(), SourceStructureDefinitionError> {
    if !source_type.applications().is_empty()
        || source_type.expressions().len() != 4
        || !source_type.arguments().is_empty()
        || !source_type.definition_returns().is_empty()
        || !source_type.mode_rhs().is_empty()
        || source_type.structure_members().len() != 4
    {
        return Err(SourceStructureDefinitionError::DependencyMismatch);
    }
    let expected = [
        (53, 42, 63, 52, 51, 59, 62),
        (56, 68, 91, 55, 54, 87, 90),
        (61, 134, 155, 60, 59, 151, 154),
        (64, 160, 183, 63, 62, 179, 182),
    ];
    for (index, (member_node, member_start, member_end, expression_node, head_node, start, end)) in
        expected.into_iter().enumerate()
    {
        let member = source_type
            .structure_members()
            .get(SourceTypeStructureMemberId::new(index))
            .ok_or(SourceStructureDefinitionError::InvalidMember { index })?;
        let expression = source_type
            .expressions()
            .get(SourceTypeExpressionId::new(index))
            .ok_or(SourceStructureDefinitionError::InvalidMember { index })?;
        if member.id().index() != index
            || member.member_site() != &node_site(member_node)
            || member.member_range() != range(input.source_id, member_start, member_end)
            || member.source_ordinal() != index
            || member.root() != SourceTypeExpressionId::new(index)
            || expression.id() != SourceTypeExpressionId::new(index)
            || expression.source_id() != input.source_id
            || expression.module_id() != &input.module_id
            || expression.site() != &node_site(expression_node)
            || expression.source_range() != range(input.source_id, start, end)
            || expression.spelling() != "set"
            || expression.head_site() != &node_site(head_node)
            || expression.head_range() != range(input.source_id, start, end)
            || expression.head_spelling() != "set"
            || expression.form() != SourceTypeApplicationForm::Bare
            || expression.head() != &SourceTypeHead::BuiltinSet
            || expression.recovery() != NodeRecoveryState::Normal
        {
            return Err(SourceStructureDefinitionError::InvalidMember { index });
        }
    }
    Ok(())
}

fn validate_handoff_rows(
    handoff: &SourceStructureDefinitionHandoff,
    source_type: &SourceTypeApplicationHandoff,
) -> Result<(), SourceStructureDefinitionError> {
    if handoff.definitions.len() != 2
        || handoff.members.len() != 4
        || handoff.inheritances.len() != 1
        || handoff.mappings.len() != 2
    {
        return Err(SourceStructureDefinitionError::UnsupportedTaskShape);
    }
    validate_handoff_resolver_rows(handoff)?;
    validate_dense_ids(handoff)?;
    let input = SourceStructureDefinitionHandoffInput {
        source_id: handoff.source_id,
        module_id: handoff.module_id.clone(),
        definitions: handoff
            .definitions
            .rows
            .iter()
            .map(|row| SourceStructureDefinitionInput {
                symbol: row.symbol.clone(),
                definition: row.definition,
                contribution: row.contribution,
                site: row.site.clone(),
                source_range: row.source_range,
                source_ordinal: row.source_ordinal,
                recovery: row.recovery,
                spelling: row.spelling.clone(),
                members: row.members.clone(),
                constructor_fields: row.constructor_fields.clone(),
            })
            .collect(),
        members: handoff
            .members
            .rows
            .iter()
            .map(|row| SourceStructureMemberInput {
                symbol: row.symbol.clone(),
                definition: row.definition,
                contribution: row.contribution,
                owner: row.owner,
                ordinal: row.ordinal,
                kind: row.kind,
                site: row.site.clone(),
                source_range: row.source_range,
                recovery: row.recovery,
                spelling: row.spelling.clone(),
                written_type: row.written_type,
                constructor_ordinal: row.constructor_ordinal,
            })
            .collect(),
        inheritances: handoff
            .inheritances
            .rows
            .iter()
            .map(|row| SourceStructureInheritanceInput {
                child: row.child,
                parent: row.parent,
                site: row.site.clone(),
                source_range: row.source_range,
                source_ordinal: row.source_ordinal,
                recovery: row.recovery,
                spelling: row.spelling.clone(),
                mappings: row.mappings.clone(),
            })
            .collect(),
        mappings: handoff
            .mappings
            .rows
            .iter()
            .map(|row| SourceStructureMappingInput {
                symbol: row.symbol.clone(),
                definition: row.definition,
                contribution: row.contribution,
                inheritance: row.inheritance,
                ordinal: row.ordinal,
                kind: row.kind,
                view_member: row.view_member,
                parent_member: row.parent_member,
                root_member: row.root_member,
                path: row.path.clone(),
                site: row.site.clone(),
                source_range: row.source_range,
                recovery: row.recovery,
                spelling: row.spelling.clone(),
            })
            .collect(),
    };
    validate_definition_inputs(&input)?;
    validate_member_inputs(&input)?;
    validate_inheritance_inputs(&input)?;
    validate_mapping_inputs(&input)?;
    validate_lower(source_type, &input)?;
    if !handoff.coherence_requests.is_empty() {
        return Err(SourceStructureDefinitionError::InvalidCoherenceRequest { index: 0 });
    }
    Ok(())
}

fn validate_dense_ids(
    handoff: &SourceStructureDefinitionHandoff,
) -> Result<(), SourceStructureDefinitionError> {
    for (index, row) in handoff.definitions.rows.iter().enumerate() {
        if row.id != SourceStructureDefinitionId::new(index) {
            return Err(SourceStructureDefinitionError::InvalidDefinition { index });
        }
    }
    for (index, row) in handoff.members.rows.iter().enumerate() {
        if row.id != SourceStructureMemberId::new(index) {
            return Err(SourceStructureDefinitionError::InvalidMember { index });
        }
    }
    for (index, row) in handoff.inheritances.rows.iter().enumerate() {
        if row.id != SourceStructureInheritanceId::new(index) {
            return Err(SourceStructureDefinitionError::InvalidInheritance { index });
        }
    }
    for (index, row) in handoff.mappings.rows.iter().enumerate() {
        if row.id != SourceStructureMappingId::new(index) {
            return Err(SourceStructureDefinitionError::InvalidMapping { index });
        }
    }
    Ok(())
}

fn validate_handoff_resolver_rows(
    handoff: &SourceStructureDefinitionHandoff,
) -> Result<(), SourceStructureDefinitionError> {
    if handoff.resolver_identity_snapshot.len() != 8 {
        return Err(SourceStructureDefinitionError::InvalidResolverDefinition { index: 0 });
    }
    let definition_expected = [
        (0, 13, 98, &[4, 0, 11, 0][..]),
        (3, 102, 190, &[4, 0, 11, 1][..]),
    ];
    for (index, row) in handoff.definitions.rows.iter().enumerate() {
        let (definition, start, end, path) = definition_expected[index];
        let identity = &handoff.resolver_identity_snapshot[index];
        if identity != &(row.symbol.clone(), row.definition, row.contribution)
            || row.symbol.module() != &handoff.module_id
            || row.definition.index() != definition
            || row.contribution.index() != 0
            || !normal_origin(
                &row.origin,
                handoff.source_id,
                &handoff.module_id,
                range(handoff.source_id, start, end),
                path,
            )
        {
            return Err(SourceStructureDefinitionError::InvalidResolverDefinition { index });
        }
    }

    let member_expected = [
        (1, 42, 63, &[4, 0, 11, 0, 18, 0][..]),
        (2, 68, 91, &[4, 0, 11, 0, 19, 1][..]),
        (4, 134, 155, &[4, 0, 11, 1, 18, 0][..]),
        (5, 160, 183, &[4, 0, 11, 1, 19, 1][..]),
    ];
    for (index, row) in handoff.members.rows.iter().enumerate() {
        let (definition, start, end, path) = member_expected[index];
        let identity = &handoff.resolver_identity_snapshot[index + 2];
        if identity != &(row.symbol.clone(), row.definition, row.contribution)
            || row.symbol.module() != &handoff.module_id
            || row.definition.index() != definition
            || row.contribution.index() != 0
            || !normal_origin(
                &row.origin,
                handoff.source_id,
                &handoff.module_id,
                range(handoff.source_id, start, end),
                path,
            )
        {
            return Err(SourceStructureDefinitionError::InvalidResolverDefinition {
                index: index + 2,
            });
        }
    }

    let mapping_expected = [
        (6, 247, 274, &[4, 0, 20, 2, 21, 0][..]),
        (7, 279, 307, &[4, 0, 20, 2, 22, 1][..]),
    ];
    for (index, row) in handoff.mappings.rows.iter().enumerate() {
        let (definition, start, end, path) = mapping_expected[index];
        let identity = &handoff.resolver_identity_snapshot[index + 6];
        if identity != &(row.symbol.clone(), row.definition, row.contribution)
            || row.symbol.module() != &handoff.module_id
            || row.definition.index() != definition
            || row.contribution.index() != 0
            || !normal_origin(
                &row.origin,
                handoff.source_id,
                &handoff.module_id,
                range(handoff.source_id, start, end),
                path,
            )
        {
            return Err(SourceStructureDefinitionError::InvalidResolverDefinition {
                index: index + 6,
            });
        }
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

fn validate_baseline(table: &InitialObligationTable) -> Result<(), SourceStructureDefinitionError> {
    if table.iter().any(|(_, row)| {
        matches!(
            row.kind,
            InitialObligationKind::PredicatePropertyCorrectness
                | InitialObligationKind::FunctorExistence
                | InitialObligationKind::FunctorUniqueness
        ) || row.goal.as_str().starts_with("source.definition.predicate")
            || row
                .provenance
                .as_str()
                .starts_with("source.definition.predicate")
            || row.goal.as_str().starts_with("source.definition.functor")
            || row
                .provenance
                .as_str()
                .starts_with("source.definition.functor")
            || row.goal.as_str().starts_with("source.definition.attribute")
            || row
                .provenance
                .as_str()
                .starts_with("source.definition.attribute")
            || row.goal.as_str().starts_with("source.definition.structure")
            || row
                .provenance
                .as_str()
                .starts_with("source.definition.structure")
    }) {
        return Err(SourceStructureDefinitionError::InvalidObligation);
    }
    validate_source_mode_definition_absence(table)
        .map_err(|_| SourceStructureDefinitionError::InvalidObligation)
}

fn validate_arena(
    handoff: &SourceStructureDefinitionHandoff,
    source_type: &SourceTypeApplicationHandoff,
    arena: &TypedArena,
) -> Result<(), SourceStructureDefinitionError> {
    source_type
        .validate_installation(handoff.source_id, &handoff.module_id, arena)
        .map_err(|_| SourceStructureDefinitionError::InvalidArenaOwnership)?;
    for row in &handoff.definitions.rows {
        validate_site(
            arena,
            &row.site,
            row.source_range,
            "source.definition.structure",
        )?;
    }
    for row in &handoff.members.rows {
        validate_site(
            arena,
            &row.site,
            row.source_range,
            "source.definition.structure.member",
        )?;
    }
    for row in &handoff.inheritances.rows {
        validate_site(
            arena,
            &row.site,
            row.source_range,
            "source.definition.structure.inheritance",
        )?;
    }
    for row in &handoff.mappings.rows {
        validate_site(
            arena,
            &row.site,
            row.source_range,
            "source.definition.structure.mapping",
        )?;
    }
    Ok(())
}

fn validate_site(
    arena: &TypedArena,
    site: &TypedSiteRef,
    source_range: SourceRange,
    kind: &str,
) -> Result<(), SourceStructureDefinitionError> {
    let TypedSiteRef::Node(node) = site else {
        return Err(SourceStructureDefinitionError::InvalidArenaOwnership);
    };
    let row = arena
        .node(*node)
        .ok_or(SourceStructureDefinitionError::InvalidArenaOwnership)?;
    if row.kind.as_str() != kind
        || row.recovery != NodeRecoveryState::Normal
        || row.anchor != SourceAnchor::Range(source_range)
    {
        return Err(SourceStructureDefinitionError::InvalidArenaOwnership);
    }
    Ok(())
}

const fn range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
    SourceRange {
        source_id,
        start,
        end,
    }
}
const fn node_site(index: usize) -> TypedSiteRef {
    TypedSiteRef::Node(TypedNodeId::new(index))
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

const fn recovery_key(value: SourceStructureDefinitionRecovery) -> &'static str {
    match value {
        SourceStructureDefinitionRecovery::Normal => "normal",
        SourceStructureDefinitionRecovery::Degraded => "degraded",
    }
}
const fn member_kind_key(value: SourceStructureMemberKind) -> &'static str {
    match value {
        SourceStructureMemberKind::Field => "field",
        SourceStructureMemberKind::Property => "property",
    }
}
const fn coherence_kind_key(value: SourceStructureCoherenceRequestKind) -> &'static str {
    match value {
        SourceStructureCoherenceRequestKind::MemberTypeInclusion => "member-type-inclusion",
    }
}
fn anchor_start(anchor: &SourceAnchor) -> usize {
    match anchor {
        SourceAnchor::Range(range) => range.start,
        SourceAnchor::Point { offset, .. } => *offset,
        _ => 0,
    }
}
fn anchor_end(anchor: &SourceAnchor) -> usize {
    match anchor {
        SourceAnchor::Range(range) => range.end,
        SourceAnchor::Point { offset, .. } => *offset,
        _ => 0,
    }
}

#[cfg(test)]
#[path = "../tests/support/source_structure_definition_unit.rs"]
mod tests;
