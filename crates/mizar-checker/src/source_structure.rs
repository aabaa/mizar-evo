//! Syntax-free transport for source structure-term occurrences.

use crate::{
    binding_env::{BindingContextId, BindingEnv},
    source_application::{SourceFunctorApplicationHandoff, SourceFunctorApplicationId},
    source_term::{SourcePrimaryTermHandoff, SourcePrimaryTermId},
    typed_ast::{NodeRecoveryState, TypedArena, TypedSiteRef},
};
use mizar_resolve::{
    env::{
        ContributionKind, DefinitionKind, ExportStatus, SignatureShell, SourceContributionId,
        SymbolEntry, SymbolEnv, SymbolKind, Visibility,
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

dense_id!(SourceStructureTermId);
dense_id!(SourceStructureWrapperId);
dense_id!(SourceStructureRootId);
dense_id!(SourceStructureMemberId);
dense_id!(SourceFieldUpdateId);
dense_id!(SourceStructureEdgeId);
dense_id!(SourceStructureRequestId);

/// Complete syntax-free input for one source/module structure transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStructureHandoffInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub terms: Vec<SourceStructureTermInput>,
    pub wrappers: Vec<SourceStructureWrapperInput>,
    pub roots: Vec<SourceStructureRootInput>,
    pub members: Vec<SourceStructureMemberInput>,
    pub field_updates: Vec<SourceFieldUpdateInput>,
    pub edges: Vec<SourceStructureEdgeInput>,
    pub requests: Vec<SourceStructureRequestInput>,
}

/// One structure-family term occurrence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStructureTermInput {
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub source_ordinal: usize,
    pub context: BindingContextId,
    pub recovery: SourceStructureRecovery,
    pub spelling: String,
    pub kind: SourceStructureTermKind,
}

/// One transparent parenthesized structure wrapper.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStructureWrapperInput {
    pub term: SourceStructureTermId,
    pub ordinal: usize,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub context: BindingContextId,
    pub spelling: String,
    pub recovery: SourceStructureRecovery,
}

/// One resolver-authenticated structure-constructor root reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStructureRootInput {
    pub term: SourceStructureTermId,
    pub symbol: SymbolId,
    pub contribution: SourceContributionId,
}

/// One source-written structure member occurrence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStructureMemberInput {
    pub term: SourceStructureTermId,
    pub ordinal: usize,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub spelling: String,
    pub role: SourceStructureMemberRole,
    pub parent: Option<SourceStructureMemberId>,
}

/// One non-term parser `FieldUpdate` association container.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFieldUpdateInput {
    pub term: SourceStructureTermId,
    pub ordinal: usize,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub spelling: String,
    pub first_member: SourceStructureMemberId,
    pub final_member: SourceStructureMemberId,
}

/// One ordered child edge owned by a structure-family term.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStructureEdgeInput {
    pub term: SourceStructureTermId,
    pub ordinal: usize,
    pub role: SourceStructureEdgeRole,
    pub member: Option<SourceStructureMemberId>,
    pub target: SourceStructureTarget,
}

/// One unresolved structure dependency request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStructureRequestInput {
    pub term: SourceStructureTermId,
    pub member: Option<SourceStructureMemberId>,
    pub request_ordinal: usize,
    pub kind: SourceStructureRequestKind,
}

/// Source structure family admitted by Task 254.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceStructureTermKind {
    Constructor,
    SelectorAccess,
    FunctionalUpdate,
}

/// Recovery state retained at the source-structure boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceStructureRecovery {
    Normal,
    Degraded,
}

/// Source role of one written structure member.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceStructureMemberRole {
    ConstructorAssignment,
    Selector,
    UpdatePathSegment,
}

/// Source role of one structure child edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceStructureEdgeRole {
    ConstructorValue,
    SelectorBase,
    SelectorArgument,
    UpdateBase,
    UpdateValue,
}

/// Cross-family target owned by one structure child occurrence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceStructureTarget {
    Primary(SourcePrimaryTermId),
    Application(SourceFunctorApplicationId),
    Structure(SourceStructureTermId),
}

/// Unresolved structure dependency request kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceStructureRequestKind {
    ConstructorSignature,
    MemberIdentity,
    InheritancePath,
    ResultType,
}

/// Immutable validated source-structure handoff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStructureHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    primary_term_fingerprint: String,
    application_fingerprint: Option<String>,
    terms: SourceStructureTermTable,
    wrappers: SourceStructureWrapperTable,
    roots: SourceStructureRootTable,
    members: SourceStructureMemberTable,
    field_updates: SourceFieldUpdateTable,
    edges: SourceStructureEdgeTable,
    requests: SourceStructureRequestTable,
}

impl SourceStructureHandoff {
    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    pub fn primary_term_fingerprint(&self) -> &str {
        &self.primary_term_fingerprint
    }

    pub fn application_fingerprint(&self) -> Option<&str> {
        self.application_fingerprint.as_deref()
    }

    pub const fn terms(&self) -> &SourceStructureTermTable {
        &self.terms
    }

    pub const fn wrappers(&self) -> &SourceStructureWrapperTable {
        &self.wrappers
    }

    pub const fn roots(&self) -> &SourceStructureRootTable {
        &self.roots
    }

    pub const fn members(&self) -> &SourceStructureMemberTable {
        &self.members
    }

    pub const fn field_updates(&self) -> &SourceFieldUpdateTable {
        &self.field_updates
    }

    pub const fn edges(&self) -> &SourceStructureEdgeTable {
        &self.edges
    }

    pub const fn requests(&self) -> &SourceStructureRequestTable {
        &self.requests
    }

    /// Stable, source-ordered representation used as the dependency fingerprint.
    pub fn debug_text(&self) -> String {
        let mut output = String::from("source-structure-debug-v1\n");
        let _ = writeln!(output, "module: {}", self.module_id.path().as_str());
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
        for (id, term) in self.terms.iter() {
            let _ = writeln!(
                output,
                "term#{} ordinal={} kind={} range={}..{} site={} context={} recovery={} spelling={:?}",
                id.index(),
                term.source_ordinal,
                term_kind_key(term.kind),
                term.source_range.start,
                term.source_range.end,
                term.site.node().index(),
                term.context.index(),
                recovery_key(term.recovery),
                term.spelling,
            );
        }
        for (id, wrapper) in self.wrappers.iter() {
            let _ = writeln!(
                output,
                "wrapper#{} term={} ordinal={} range={}..{} site={} context={} recovery={} spelling={:?}",
                id.index(),
                wrapper.term.index(),
                wrapper.ordinal,
                wrapper.source_range.start,
                wrapper.source_range.end,
                wrapper.site.node().index(),
                wrapper.context.index(),
                recovery_key(wrapper.recovery),
                wrapper.spelling,
            );
        }
        for (id, root) in self.roots.iter() {
            let _ = writeln!(
                output,
                "root#{} term={} symbol={:?} contribution={} origin={:?} visibility={:?} export={:?} signature={:?}",
                id.index(),
                root.term.index(),
                root.symbol,
                root.contribution.index(),
                root.origin,
                root.visibility,
                root.export_status,
                root.signature,
            );
        }
        for (id, member) in self.members.iter() {
            let _ = write!(
                output,
                "member#{} term={} ordinal={} role={} range={}..{} site={} spelling={:?} parent=",
                id.index(),
                member.term.index(),
                member.ordinal,
                member_role_key(member.role),
                member.source_range.start,
                member.source_range.end,
                member.site.node().index(),
                member.spelling,
            );
            write_optional_member(&mut output, member.parent);
            output.push('\n');
        }
        for (id, update) in self.field_updates.iter() {
            let _ = writeln!(
                output,
                "field-update#{} term={} ordinal={} range={}..{} site={} spelling={:?} first_member={} final_member={}",
                id.index(),
                update.term.index(),
                update.ordinal,
                update.source_range.start,
                update.source_range.end,
                update.site.node().index(),
                update.spelling,
                update.first_member.index(),
                update.final_member.index(),
            );
        }
        for (id, edge) in self.edges.iter() {
            let _ = write!(
                output,
                "edge#{} term={} ordinal={} role={} member=",
                id.index(),
                edge.term.index(),
                edge.ordinal,
                edge_role_key(edge.role),
            );
            write_optional_member(&mut output, edge.member);
            output.push_str(" target=");
            write_target(&mut output, edge.target);
            output.push('\n');
        }
        for (id, request) in self.requests.iter() {
            let _ = write!(
                output,
                "request#{} term={} ordinal={} kind={} member=",
                id.index(),
                request.term.index(),
                request.request_ordinal,
                request_kind_key(request.kind),
            );
            write_optional_member(&mut output, request.member);
            output.push('\n');
        }
        output
    }

    pub(crate) fn validate_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        primary_terms: &SourcePrimaryTermHandoff,
        applications: Option<&SourceFunctorApplicationHandoff>,
        arena: &TypedArena,
    ) -> Result<(), SourceStructureError> {
        if self.source_id != source_id
            || &self.module_id != module_id
            || primary_terms.source_id() != source_id
            || primary_terms.module_id() != module_id
            || primary_terms.debug_text() != self.primary_term_fingerprint
        {
            return Err(SourceStructureError::PrimaryDependencyMismatch);
        }

        match (&self.application_fingerprint, applications) {
            (Some(expected), Some(actual))
                if actual.source_id() == source_id
                    && actual.module_id() == module_id
                    && actual.debug_text() == *expected =>
            {
                actual
                    .validate_installation(source_id, module_id, primary_terms)
                    .map_err(|_| SourceStructureError::ApplicationDependencyMismatch)?;
            }
            (Some(_), _) => return Err(SourceStructureError::ApplicationDependencyMismatch),
            (None, Some(actual)) => {
                actual
                    .validate_installation(source_id, module_id, primary_terms)
                    .map_err(|_| SourceStructureError::ApplicationDependencyMismatch)?;
            }
            (None, None) => {}
        }

        validate_output_arena_and_references(self, arena)?;
        validate_output_targets(self, primary_terms, applications)?;
        validate_output_application_ownership(self, applications)
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
    SourceStructureTermTable,
    SourceStructureTerm,
    SourceStructureTermId
);
table!(
    SourceStructureWrapperTable,
    SourceStructureWrapper,
    SourceStructureWrapperId
);
table!(
    SourceStructureRootTable,
    SourceStructureRoot,
    SourceStructureRootId
);
table!(
    SourceStructureMemberTable,
    SourceStructureMember,
    SourceStructureMemberId
);
table!(
    SourceFieldUpdateTable,
    SourceFieldUpdate,
    SourceFieldUpdateId
);
table!(
    SourceStructureEdgeTable,
    SourceStructureEdge,
    SourceStructureEdgeId
);
table!(
    SourceStructureRequestTable,
    SourceStructureRequest,
    SourceStructureRequestId
);

/// One validated structure-family term.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStructureTerm {
    site: TypedSiteRef,
    source_range: SourceRange,
    source_ordinal: usize,
    context: BindingContextId,
    recovery: SourceStructureRecovery,
    spelling: String,
    kind: SourceStructureTermKind,
}

impl SourceStructureTerm {
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
    pub const fn recovery(&self) -> SourceStructureRecovery {
        self.recovery
    }
    pub fn spelling(&self) -> &str {
        &self.spelling
    }
    pub const fn kind(&self) -> SourceStructureTermKind {
        self.kind
    }
}

/// One validated transparent structure wrapper.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStructureWrapper {
    term: SourceStructureTermId,
    ordinal: usize,
    site: TypedSiteRef,
    source_range: SourceRange,
    context: BindingContextId,
    spelling: String,
    recovery: SourceStructureRecovery,
}

impl SourceStructureWrapper {
    pub const fn term(&self) -> SourceStructureTermId {
        self.term
    }
    pub const fn ordinal(&self) -> usize {
        self.ordinal
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
    pub fn spelling(&self) -> &str {
        &self.spelling
    }
    pub const fn recovery(&self) -> SourceStructureRecovery {
        self.recovery
    }
}

/// One validated resolver structure reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStructureRoot {
    term: SourceStructureTermId,
    symbol: SymbolId,
    contribution: SourceContributionId,
    origin: SemanticOrigin,
    visibility: Visibility,
    export_status: ExportStatus,
    signature: Option<SignatureShell>,
}

impl SourceStructureRoot {
    pub const fn term(&self) -> SourceStructureTermId {
        self.term
    }
    pub const fn symbol(&self) -> &SymbolId {
        &self.symbol
    }
    pub const fn contribution(&self) -> SourceContributionId {
        self.contribution
    }
    pub const fn origin(&self) -> &SemanticOrigin {
        &self.origin
    }
    pub const fn visibility(&self) -> Visibility {
        self.visibility
    }
    pub const fn export_status(&self) -> ExportStatus {
        self.export_status
    }
    pub const fn signature(&self) -> Option<&SignatureShell> {
        self.signature.as_ref()
    }
}

/// One validated written member occurrence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStructureMember {
    term: SourceStructureTermId,
    ordinal: usize,
    site: TypedSiteRef,
    source_range: SourceRange,
    spelling: String,
    role: SourceStructureMemberRole,
    parent: Option<SourceStructureMemberId>,
}

impl SourceStructureMember {
    pub const fn term(&self) -> SourceStructureTermId {
        self.term
    }
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }
    pub const fn site(&self) -> &TypedSiteRef {
        &self.site
    }
    pub const fn source_range(&self) -> SourceRange {
        self.source_range
    }
    pub fn spelling(&self) -> &str {
        &self.spelling
    }
    pub const fn role(&self) -> SourceStructureMemberRole {
        self.role
    }
    pub const fn parent(&self) -> Option<SourceStructureMemberId> {
        self.parent
    }
}

/// One validated `FieldUpdate` association container.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFieldUpdate {
    term: SourceStructureTermId,
    ordinal: usize,
    site: TypedSiteRef,
    source_range: SourceRange,
    spelling: String,
    first_member: SourceStructureMemberId,
    final_member: SourceStructureMemberId,
}

impl SourceFieldUpdate {
    pub const fn term(&self) -> SourceStructureTermId {
        self.term
    }
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }
    pub const fn site(&self) -> &TypedSiteRef {
        &self.site
    }
    pub const fn source_range(&self) -> SourceRange {
        self.source_range
    }
    pub fn spelling(&self) -> &str {
        &self.spelling
    }
    pub const fn first_member(&self) -> SourceStructureMemberId {
        self.first_member
    }
    pub const fn final_member(&self) -> SourceStructureMemberId {
        self.final_member
    }
}

/// One validated ordered child edge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStructureEdge {
    term: SourceStructureTermId,
    ordinal: usize,
    role: SourceStructureEdgeRole,
    member: Option<SourceStructureMemberId>,
    target: SourceStructureTarget,
}

impl SourceStructureEdge {
    pub const fn term(&self) -> SourceStructureTermId {
        self.term
    }
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }
    pub const fn role(&self) -> SourceStructureEdgeRole {
        self.role
    }
    pub const fn member(&self) -> Option<SourceStructureMemberId> {
        self.member
    }
    pub const fn target(&self) -> SourceStructureTarget {
        self.target
    }
}

/// One validated unresolved structure request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStructureRequest {
    term: SourceStructureTermId,
    member: Option<SourceStructureMemberId>,
    request_ordinal: usize,
    kind: SourceStructureRequestKind,
}

impl SourceStructureRequest {
    pub const fn term(&self) -> SourceStructureTermId {
        self.term
    }
    pub const fn member(&self) -> Option<SourceStructureMemberId> {
        self.member
    }
    pub const fn request_ordinal(&self) -> usize {
        self.request_ordinal
    }
    pub const fn kind(&self) -> SourceStructureRequestKind {
        self.kind
    }
}

/// Atomically validates and constructs source-structure handoffs.
pub struct SourceStructureProducer;

impl SourceStructureProducer {
    pub fn build(
        input: SourceStructureHandoffInput,
        symbols: &SymbolEnv,
        bindings: &BindingEnv,
        primary_terms: &SourcePrimaryTermHandoff,
        applications: Option<&SourceFunctorApplicationHandoff>,
        arena: &TypedArena,
    ) -> Result<SourceStructureHandoff, SourceStructureError> {
        let uses_applications = input
            .edges
            .iter()
            .any(|edge| matches!(edge.target, SourceStructureTarget::Application(_)));
        validate_input(
            &input,
            symbols,
            bindings,
            primary_terms,
            applications,
            arena,
            uses_applications,
        )?;

        let primary_term_fingerprint = primary_terms.debug_text();
        let application_fingerprint =
            uses_applications.then(|| applications.expect("dependency was validated").debug_text());
        let terms = SourceStructureTermTable {
            rows: input
                .terms
                .into_iter()
                .map(|row| SourceStructureTerm {
                    site: row.site,
                    source_range: row.source_range,
                    source_ordinal: row.source_ordinal,
                    context: row.context,
                    recovery: row.recovery,
                    spelling: row.spelling,
                    kind: row.kind,
                })
                .collect(),
        };
        let wrappers = SourceStructureWrapperTable {
            rows: input
                .wrappers
                .into_iter()
                .map(|row| SourceStructureWrapper {
                    term: row.term,
                    ordinal: row.ordinal,
                    site: row.site,
                    source_range: row.source_range,
                    context: row.context,
                    spelling: row.spelling,
                    recovery: row.recovery,
                })
                .collect(),
        };
        let roots = SourceStructureRootTable {
            rows: input
                .roots
                .into_iter()
                .map(|row| {
                    let entry = symbols
                        .symbols()
                        .get(&row.symbol)
                        .expect("root was authenticated");
                    SourceStructureRoot {
                        term: row.term,
                        symbol: row.symbol,
                        contribution: row.contribution,
                        origin: entry.origin().clone(),
                        visibility: entry.visibility(),
                        export_status: entry.export_status(),
                        signature: entry.signature().cloned(),
                    }
                })
                .collect(),
        };
        let members = SourceStructureMemberTable {
            rows: input
                .members
                .into_iter()
                .map(|row| SourceStructureMember {
                    term: row.term,
                    ordinal: row.ordinal,
                    site: row.site,
                    source_range: row.source_range,
                    spelling: row.spelling,
                    role: row.role,
                    parent: row.parent,
                })
                .collect(),
        };
        let field_updates = SourceFieldUpdateTable {
            rows: input
                .field_updates
                .into_iter()
                .map(|row| SourceFieldUpdate {
                    term: row.term,
                    ordinal: row.ordinal,
                    site: row.site,
                    source_range: row.source_range,
                    spelling: row.spelling,
                    first_member: row.first_member,
                    final_member: row.final_member,
                })
                .collect(),
        };
        let edges = SourceStructureEdgeTable {
            rows: input
                .edges
                .into_iter()
                .map(|row| SourceStructureEdge {
                    term: row.term,
                    ordinal: row.ordinal,
                    role: row.role,
                    member: row.member,
                    target: row.target,
                })
                .collect(),
        };
        let requests = SourceStructureRequestTable {
            rows: input
                .requests
                .into_iter()
                .map(|row| SourceStructureRequest {
                    term: row.term,
                    member: row.member,
                    request_ordinal: row.request_ordinal,
                    kind: row.kind,
                })
                .collect(),
        };

        Ok(SourceStructureHandoff {
            source_id: input.source_id,
            module_id: input.module_id,
            primary_term_fingerprint,
            application_fingerprint,
            terms,
            wrappers,
            roots,
            members,
            field_updates,
            edges,
            requests,
        })
    }
}

/// Atomic Task-254 producer failure.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceStructureError {
    EnvironmentMismatch,
    PrimaryDependencyMismatch,
    ApplicationDependencyMismatch,
    InvalidTerm { term: SourceStructureTermId },
    InvalidWrapper { wrapper: SourceStructureWrapperId },
    InvalidRoot { root: SourceStructureRootId },
    InvalidMember { member: SourceStructureMemberId },
    InvalidFieldUpdate { field_update: SourceFieldUpdateId },
    InvalidEdge { edge: SourceStructureEdgeId },
    InvalidRequest { request: SourceStructureRequestId },
    DuplicateSite,
    ReorderedTerm { term: SourceStructureTermId },
    ReorderedWrapper { wrapper: SourceStructureWrapperId },
    ReorderedRoot { root: SourceStructureRootId },
    ReorderedMember { member: SourceStructureMemberId },
    ReorderedFieldUpdate { field_update: SourceFieldUpdateId },
    ReorderedEdge { edge: SourceStructureEdgeId },
    MultipleParents { term: SourceStructureTermId },
    DuplicateTarget { edge: SourceStructureEdgeId },
    OverlappingChildren { term: SourceStructureTermId },
}

impl fmt::Display for SourceStructureError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EnvironmentMismatch => {
                formatter.write_str("source structure environment identity mismatch")
            }
            Self::PrimaryDependencyMismatch => {
                formatter.write_str("source structure primary-term dependency mismatch")
            }
            Self::ApplicationDependencyMismatch => {
                formatter.write_str("source structure application dependency mismatch")
            }
            Self::InvalidTerm { term } => {
                write!(
                    formatter,
                    "source structure term {} is invalid",
                    term.index()
                )
            }
            Self::InvalidWrapper { wrapper } => {
                write!(
                    formatter,
                    "source structure wrapper {} is invalid",
                    wrapper.index()
                )
            }
            Self::InvalidRoot { root } => {
                write!(
                    formatter,
                    "source structure root {} is invalid",
                    root.index()
                )
            }
            Self::InvalidMember { member } => {
                write!(
                    formatter,
                    "source structure member {} is invalid",
                    member.index()
                )
            }
            Self::InvalidFieldUpdate { field_update } => write!(
                formatter,
                "source field update {} is invalid",
                field_update.index()
            ),
            Self::InvalidEdge { edge } => {
                write!(
                    formatter,
                    "source structure edge {} is invalid",
                    edge.index()
                )
            }
            Self::InvalidRequest { request } => write!(
                formatter,
                "source structure request {} is invalid",
                request.index()
            ),
            Self::DuplicateSite => formatter.write_str("source structure repeats a typed site"),
            Self::ReorderedTerm { term } => write!(
                formatter,
                "source structure term {} is out of source pre-order",
                term.index()
            ),
            Self::ReorderedWrapper { wrapper } => write!(
                formatter,
                "source structure wrapper {} is out of order",
                wrapper.index()
            ),
            Self::ReorderedRoot { root } => {
                write!(
                    formatter,
                    "source structure root {} is out of order",
                    root.index()
                )
            }
            Self::ReorderedMember { member } => write!(
                formatter,
                "source structure member {} is out of order",
                member.index()
            ),
            Self::ReorderedFieldUpdate { field_update } => write!(
                formatter,
                "source field update {} is out of order",
                field_update.index()
            ),
            Self::ReorderedEdge { edge } => {
                write!(
                    formatter,
                    "source structure edge {} is out of order",
                    edge.index()
                )
            }
            Self::MultipleParents { term } => write!(
                formatter,
                "source structure term {} has multiple parents",
                term.index()
            ),
            Self::DuplicateTarget { edge } => write!(
                formatter,
                "source structure edge {} repeats an owned occurrence",
                edge.index()
            ),
            Self::OverlappingChildren { term } => write!(
                formatter,
                "source structure term {} has overlapping children",
                term.index()
            ),
        }
    }
}

impl Error for SourceStructureError {}

fn validate_input(
    input: &SourceStructureHandoffInput,
    symbols: &SymbolEnv,
    bindings: &BindingEnv,
    primary_terms: &SourcePrimaryTermHandoff,
    applications: Option<&SourceFunctorApplicationHandoff>,
    arena: &TypedArena,
    uses_applications: bool,
) -> Result<(), SourceStructureError> {
    if symbols.module_id() != &input.module_id
        || bindings.source_id() != input.source_id
        || bindings.module_id() != &input.module_id
        || primary_terms.source_id() != input.source_id
        || primary_terms.module_id() != &input.module_id
    {
        return Err(SourceStructureError::EnvironmentMismatch);
    }
    if input.terms.is_empty()
        && (!input.wrappers.is_empty()
            || !input.roots.is_empty()
            || !input.members.is_empty()
            || !input.field_updates.is_empty()
            || !input.edges.is_empty()
            || !input.requests.is_empty())
    {
        return Err(SourceStructureError::EnvironmentMismatch);
    }
    if uses_applications && applications.is_none() {
        return Err(SourceStructureError::ApplicationDependencyMismatch);
    }
    if let Some(applications) = applications {
        if applications.source_id() != input.source_id
            || applications.module_id() != &input.module_id
            || applications.primary_term_fingerprint() != primary_terms.debug_text()
        {
            return Err(SourceStructureError::ApplicationDependencyMismatch);
        }
        applications
            .validate_installation(input.source_id, &input.module_id, primary_terms)
            .map_err(|_| SourceStructureError::ApplicationDependencyMismatch)?;
    }

    let mut sites = BTreeSet::new();
    validate_terms(input, bindings, arena, &mut sites)?;
    let wrapper_groups = validate_wrappers(input, arena, &mut sites)?;
    let effective = input_effective_occurrences(input, &wrapper_groups);
    validate_term_preorder(input, &effective)?;
    validate_roots(input, symbols)?;
    let member_groups = validate_members(input, arena, &mut sites)?;
    let update_groups = validate_field_updates(input, arena, &member_groups, &mut sites)?;
    let edge_groups = validate_edges(
        input,
        primary_terms,
        applications,
        &effective,
        &member_groups,
        &update_groups,
        &mut sites,
    )?;
    validate_structure_ownership(input, &effective)?;
    validate_edge_shapes(input, &member_groups, &update_groups, &edge_groups)?;
    validate_input_application_ownership(input, &effective, applications)?;
    validate_requests(input, &member_groups)?;
    Ok(())
}

fn validate_terms(
    input: &SourceStructureHandoffInput,
    bindings: &BindingEnv,
    arena: &TypedArena,
    sites: &mut BTreeSet<TypedSiteRef>,
) -> Result<(), SourceStructureError> {
    for (index, term) in input.terms.iter().enumerate() {
        let id = SourceStructureTermId::new(index);
        if term.source_ordinal != index
            || bindings.contexts().get(term.context).is_none()
            || !valid_range(input.source_id, term.source_range)
            || !canonical_spelling(&term.spelling)
        {
            return Err(SourceStructureError::InvalidTerm { term: id });
        }
        validate_arena_site(
            &term.site,
            term.source_range,
            term_kind_node_key(term.kind),
            term.recovery,
            arena,
        )
        .map_err(|()| SourceStructureError::InvalidTerm { term: id })?;
        if !sites.insert(term.site.clone()) {
            return Err(SourceStructureError::DuplicateSite);
        }
    }
    Ok(())
}

fn validate_wrappers(
    input: &SourceStructureHandoffInput,
    arena: &TypedArena,
    sites: &mut BTreeSet<TypedSiteRef>,
) -> Result<Vec<Vec<usize>>, SourceStructureError> {
    let groups = grouped_rows(
        input.terms.len(),
        &input.wrappers,
        |row| row.term,
        |row| row.ordinal,
        |index| SourceStructureError::ReorderedWrapper {
            wrapper: SourceStructureWrapperId::new(index),
        },
    )?;
    for (term_index, group) in groups.iter().enumerate() {
        let term_id = SourceStructureTermId::new(term_index);
        let term = &input.terms[term_index];
        let mut contained_range = term.source_range;
        let mut contained_spelling = term.spelling.as_str();
        for wrapper_index in group.iter().rev().copied() {
            let id = SourceStructureWrapperId::new(wrapper_index);
            let wrapper = &input.wrappers[wrapper_index];
            if wrapper.term != term_id
                || wrapper.context != term.context
                || !valid_range(input.source_id, wrapper.source_range)
                || !strictly_contains(wrapper.source_range, contained_range)
                || wrapper.spelling != format!("( {contained_spelling} )")
            {
                return Err(SourceStructureError::InvalidWrapper { wrapper: id });
            }
            validate_arena_site(
                &wrapper.site,
                wrapper.source_range,
                "source.term.structure.parenthesized",
                wrapper.recovery,
                arena,
            )
            .map_err(|()| SourceStructureError::InvalidWrapper { wrapper: id })?;
            if !sites.insert(wrapper.site.clone()) {
                return Err(SourceStructureError::DuplicateSite);
            }
            contained_range = wrapper.source_range;
            contained_spelling = &wrapper.spelling;
        }
    }
    Ok(groups)
}

fn validate_term_preorder(
    input: &SourceStructureHandoffInput,
    effective: &[(SourceRange, String)],
) -> Result<(), SourceStructureError> {
    for right in 1..input.terms.len() {
        for left in 0..right {
            let left_range = effective[left].0;
            let right_range = effective[right].0;
            if right_range.start < left_range.start
                || (right_range.start < left_range.end
                    && !properly_contains(left_range, right_range))
            {
                return Err(SourceStructureError::ReorderedTerm {
                    term: SourceStructureTermId::new(right),
                });
            }
        }
    }
    Ok(())
}

fn validate_roots(
    input: &SourceStructureHandoffInput,
    symbols: &SymbolEnv,
) -> Result<(), SourceStructureError> {
    let mut previous_term = None;
    let mut roots_by_term = vec![0usize; input.terms.len()];
    for (index, root) in input.roots.iter().enumerate() {
        let id = SourceStructureRootId::new(index);
        let Some(term) = input.terms.get(root.term.index()) else {
            return Err(SourceStructureError::InvalidRoot { root: id });
        };
        if previous_term.is_some_and(|previous| previous >= root.term)
            || term.kind != SourceStructureTermKind::Constructor
        {
            return Err(SourceStructureError::ReorderedRoot { root: id });
        }
        roots_by_term[root.term.index()] += 1;
        if roots_by_term[root.term.index()] != 1 {
            return Err(SourceStructureError::InvalidRoot { root: id });
        }
        validate_root(input, id, term, root, symbols)?;
        previous_term = Some(root.term);
    }
    for (index, term) in input.terms.iter().enumerate() {
        let expected = usize::from(term.kind == SourceStructureTermKind::Constructor);
        if roots_by_term[index] != expected {
            return Err(SourceStructureError::InvalidTerm {
                term: SourceStructureTermId::new(index),
            });
        }
    }
    Ok(())
}

fn validate_root(
    input: &SourceStructureHandoffInput,
    id: SourceStructureRootId,
    term: &SourceStructureTermInput,
    root: &SourceStructureRootInput,
    symbols: &SymbolEnv,
) -> Result<(), SourceStructureError> {
    let invalid = || SourceStructureError::InvalidRoot { root: id };
    let entry = symbols.symbols().get(&root.symbol).ok_or_else(invalid)?;
    let contribution = symbols
        .contributions()
        .get(root.contribution)
        .ok_or_else(invalid)?;
    if entry.kind() != SymbolKind::Structure
        || entry.contribution() != root.contribution
        || entry.namespace().as_str() != input.module_id.path().as_str()
        || contribution.module() != root.symbol.module()
        || !contribution.effects().symbols().contains(&root.symbol)
        || entry.origin().is_recovered()
        || matches!(entry.signature(), Some(SignatureShell::Malformed { .. }))
    {
        return Err(invalid());
    }

    match contribution.kind() {
        ContributionKind::LocalSource { source_id } => {
            let definition = symbols
                .definitions()
                .by_symbol(&root.symbol)
                .ok_or_else(invalid)?;
            let origin_range = source_range(entry.origin().anchor()).ok_or_else(invalid)?;
            if *source_id != input.source_id
                || contribution.module() != &input.module_id
                || root.symbol.module() != &input.module_id
                || entry.origin().source_id() != input.source_id
                || entry.origin().module_id() != &input.module_id
                || entry.origin().import_edge().is_some()
                || !valid_range(input.source_id, origin_range)
                || origin_range.end > term.source_range.start
                || definition.kind() != DefinitionKind::Structure
                || definition.symbol() != &root.symbol
                || definition.contribution() != root.contribution
                || definition.origin() != entry.origin()
                || definition.visibility() != entry.visibility()
                || definition.signature() != entry.signature()
                || definition.conflict().is_some()
                || !contribution
                    .effects()
                    .definitions()
                    .contains(&definition.id())
            {
                return Err(invalid());
            }
        }
        ContributionKind::ImportedSource { source_id } => {
            let contribution_range = source_range(contribution.anchor()).ok_or_else(invalid)?;
            let authenticated_import = contribution.effects().imports().iter().any(|import| {
                symbols
                    .imports()
                    .get(*import)
                    .and_then(|import| import.module())
                    == Some(root.symbol.module())
            });
            if *source_id != input.source_id
                || !valid_imported_root_provenance(
                    entry,
                    &root.symbol,
                    input.source_id,
                    term.source_range,
                    contribution_range,
                    authenticated_import,
                )
            {
                return Err(invalid());
            }
        }
        ContributionKind::Summary { .. } | ContributionKind::Builtin { .. } | _ => {
            return Err(invalid());
        }
    }
    Ok(())
}

fn valid_imported_root_provenance(
    entry: &SymbolEntry,
    symbol: &SymbolId,
    source_id: SourceId,
    use_range: SourceRange,
    contribution_range: SourceRange,
    authenticated_import: bool,
) -> bool {
    entry.visibility() == Visibility::Public
        && matches!(
            entry.export_status(),
            ExportStatus::Exported | ExportStatus::ReExported
        )
        && valid_range(source_id, contribution_range)
        && contribution_range.end <= use_range.start
        && entry.origin().source_id() == source_id
        && entry.origin().module_id() == symbol.module()
        && authenticated_import
}

fn validate_members(
    input: &SourceStructureHandoffInput,
    arena: &TypedArena,
    sites: &mut BTreeSet<TypedSiteRef>,
) -> Result<Vec<Vec<usize>>, SourceStructureError> {
    let groups = grouped_rows(
        input.terms.len(),
        &input.members,
        |row| row.term,
        |row| row.ordinal,
        |index| SourceStructureError::ReorderedMember {
            member: SourceStructureMemberId::new(index),
        },
    )?;
    for (term_index, group) in groups.iter().enumerate() {
        let term_id = SourceStructureTermId::new(term_index);
        let term = &input.terms[term_index];
        let expected_role = match term.kind {
            SourceStructureTermKind::Constructor => {
                SourceStructureMemberRole::ConstructorAssignment
            }
            SourceStructureTermKind::SelectorAccess => SourceStructureMemberRole::Selector,
            SourceStructureTermKind::FunctionalUpdate => {
                SourceStructureMemberRole::UpdatePathSegment
            }
        };
        if term.kind == SourceStructureTermKind::SelectorAccess && group.len() != 1 {
            return Err(SourceStructureError::InvalidTerm { term: term_id });
        }
        let mut previous_range = None;
        for member_index in group.iter().copied() {
            let id = SourceStructureMemberId::new(member_index);
            let member = &input.members[member_index];
            if member.role != expected_role
                || !valid_range(input.source_id, member.source_range)
                || !properly_contains(term.source_range, member.source_range)
                || !identifier_spelling(&member.spelling)
                || previous_range
                    .is_some_and(|previous: SourceRange| previous.end > member.source_range.start)
            {
                return Err(SourceStructureError::InvalidMember { member: id });
            }
            if expected_role != SourceStructureMemberRole::UpdatePathSegment
                && member.parent.is_some()
            {
                return Err(SourceStructureError::InvalidMember { member: id });
            }
            if let Some(parent_id) = member.parent {
                let Some(parent) = input.members.get(parent_id.index()) else {
                    return Err(SourceStructureError::InvalidMember { member: id });
                };
                if parent_id.index() + 1 != member_index
                    || parent.term != term_id
                    || parent.role != SourceStructureMemberRole::UpdatePathSegment
                {
                    return Err(SourceStructureError::InvalidMember { member: id });
                }
            }
            validate_arena_site(
                &member.site,
                member.source_range,
                member_role_node_key(member.role),
                term.recovery,
                arena,
            )
            .map_err(|()| SourceStructureError::InvalidMember { member: id })?;
            if !sites.insert(member.site.clone()) {
                return Err(SourceStructureError::DuplicateSite);
            }
            previous_range = Some(member.source_range);
        }
    }
    Ok(groups)
}

fn validate_field_updates(
    input: &SourceStructureHandoffInput,
    arena: &TypedArena,
    member_groups: &[Vec<usize>],
    sites: &mut BTreeSet<TypedSiteRef>,
) -> Result<Vec<Vec<usize>>, SourceStructureError> {
    let groups = grouped_rows(
        input.terms.len(),
        &input.field_updates,
        |row| row.term,
        |row| row.ordinal,
        |index| SourceStructureError::ReorderedFieldUpdate {
            field_update: SourceFieldUpdateId::new(index),
        },
    )?;
    let mut owned_members = BTreeSet::new();
    for (term_index, group) in groups.iter().enumerate() {
        let term_id = SourceStructureTermId::new(term_index);
        let term = &input.terms[term_index];
        if term.kind == SourceStructureTermKind::FunctionalUpdate && group.is_empty() {
            return Err(SourceStructureError::InvalidTerm { term: term_id });
        }
        if term.kind != SourceStructureTermKind::FunctionalUpdate && !group.is_empty() {
            return Err(SourceStructureError::InvalidFieldUpdate {
                field_update: SourceFieldUpdateId::new(group[0]),
            });
        }
        let mut previous_range = None;
        for update_index in group.iter().copied() {
            let id = SourceFieldUpdateId::new(update_index);
            let update = &input.field_updates[update_index];
            let first = update.first_member.index();
            let final_member = update.final_member.index();
            if !valid_range(input.source_id, update.source_range)
                || !properly_contains(term.source_range, update.source_range)
                || !canonical_spelling(&update.spelling)
                || first > final_member
                || previous_range
                    .is_some_and(|previous: SourceRange| previous.end > update.source_range.start)
            {
                return Err(SourceStructureError::InvalidFieldUpdate { field_update: id });
            }
            let Some(first_row) = input.members.get(first) else {
                return Err(SourceStructureError::InvalidFieldUpdate { field_update: id });
            };
            let Some(final_row) = input.members.get(final_member) else {
                return Err(SourceStructureError::InvalidFieldUpdate { field_update: id });
            };
            if first_row.term != term_id
                || final_row.term != term_id
                || first_row.parent.is_some()
                || !range_contains(update.source_range, first_row.source_range)
                || !range_contains(update.source_range, final_row.source_range)
            {
                return Err(SourceStructureError::InvalidFieldUpdate { field_update: id });
            }
            for member_index in first..=final_member {
                let member_id = SourceStructureMemberId::new(member_index);
                let Some(member) = input.members.get(member_index) else {
                    return Err(SourceStructureError::InvalidFieldUpdate { field_update: id });
                };
                let expected_parent =
                    (member_index > first).then(|| SourceStructureMemberId::new(member_index - 1));
                if member.term != term_id
                    || member.role != SourceStructureMemberRole::UpdatePathSegment
                    || member.parent != expected_parent
                    || !range_contains(update.source_range, member.source_range)
                    || !owned_members.insert(member_id)
                {
                    return Err(SourceStructureError::InvalidFieldUpdate { field_update: id });
                }
            }
            validate_arena_site(
                &update.site,
                update.source_range,
                "source.term.structure.field-update",
                term.recovery,
                arena,
            )
            .map_err(|()| SourceStructureError::InvalidFieldUpdate { field_update: id })?;
            if !sites.insert(update.site.clone()) {
                return Err(SourceStructureError::DuplicateSite);
            }
            previous_range = Some(update.source_range);
        }
        if term.kind == SourceStructureTermKind::FunctionalUpdate {
            for member_index in &member_groups[term_index] {
                if !owned_members.contains(&SourceStructureMemberId::new(*member_index)) {
                    return Err(SourceStructureError::InvalidMember {
                        member: SourceStructureMemberId::new(*member_index),
                    });
                }
            }
        }
    }
    Ok(groups)
}

#[allow(clippy::too_many_arguments)] // Rationale: keep every cross-family ownership input explicit at this validation boundary.
fn validate_edges(
    input: &SourceStructureHandoffInput,
    primary_terms: &SourcePrimaryTermHandoff,
    applications: Option<&SourceFunctorApplicationHandoff>,
    effective: &[(SourceRange, String)],
    member_groups: &[Vec<usize>],
    update_groups: &[Vec<usize>],
    sites: &mut BTreeSet<TypedSiteRef>,
) -> Result<Vec<Vec<usize>>, SourceStructureError> {
    let groups = grouped_rows(
        input.terms.len(),
        &input.edges,
        |row| row.term,
        |row| row.ordinal,
        |index| SourceStructureError::ReorderedEdge {
            edge: SourceStructureEdgeId::new(index),
        },
    )?;
    let application_roots = applications.map(application_root_ids);
    let mut primary_targets = BTreeSet::new();
    let mut application_targets = BTreeSet::new();
    let mut structure_targets = BTreeSet::new();
    for (term_index, group) in groups.iter().enumerate() {
        let parent = &input.terms[term_index];
        let parent_id = SourceStructureTermId::new(term_index);
        let direct_targets =
            direct_input_targets(input, primary_terms, applications, effective, term_index)?;
        if direct_targets.len() != group.len() {
            return Err(SourceStructureError::InvalidTerm { term: parent_id });
        }
        let mut previous_range = None;
        for (edge_ordinal, edge_index) in group.iter().copied().enumerate() {
            let id = SourceStructureEdgeId::new(edge_index);
            let edge = &input.edges[edge_index];
            if edge.target != direct_targets[edge_ordinal].0 {
                return Err(SourceStructureError::InvalidEdge { edge: id });
            }
            let (target_range, target_site, target_spelling) = match edge.target {
                SourceStructureTarget::Primary(primary_id) => {
                    let primary = primary_terms
                        .terms()
                        .get(primary_id)
                        .ok_or(SourceStructureError::InvalidEdge { edge: id })?;
                    if primary.parent().is_some()
                        || primary.context() != parent.context
                        || !primary_targets.insert(primary_id)
                    {
                        return Err(SourceStructureError::DuplicateTarget { edge: id });
                    }
                    (
                        primary.source_range(),
                        Some(primary.site().clone()),
                        primary.spelling(),
                    )
                }
                SourceStructureTarget::Application(application_id) => {
                    let applications =
                        applications.ok_or(SourceStructureError::ApplicationDependencyMismatch)?;
                    let application = applications
                        .applications()
                        .get(application_id)
                        .ok_or(SourceStructureError::InvalidEdge { edge: id })?;
                    if application.context() != parent.context
                        || !application_roots
                            .as_ref()
                            .is_some_and(|roots| roots.contains(&application_id))
                        || !application_targets.insert(application_id)
                    {
                        return Err(SourceStructureError::DuplicateTarget { edge: id });
                    }
                    let (range, site, spelling) =
                        application_effective_occurrence(applications, application_id)
                            .ok_or(SourceStructureError::InvalidEdge { edge: id })?;
                    (range, Some(site), spelling)
                }
                SourceStructureTarget::Structure(child_id) => {
                    let child = input
                        .terms
                        .get(child_id.index())
                        .ok_or(SourceStructureError::InvalidEdge { edge: id })?;
                    if child_id <= parent_id || child.context != parent.context {
                        return Err(SourceStructureError::InvalidEdge { edge: id });
                    }
                    if !structure_targets.insert(child_id) {
                        return Err(SourceStructureError::MultipleParents { term: child_id });
                    }
                    (
                        effective[child_id.index()].0,
                        None,
                        effective[child_id.index()].1.as_str(),
                    )
                }
            };
            if !properly_contains(parent.source_range, target_range) {
                return Err(SourceStructureError::InvalidEdge { edge: id });
            }
            if previous_range.is_some_and(|previous: SourceRange| previous.end > target_range.start)
            {
                return Err(SourceStructureError::OverlappingChildren { term: parent_id });
            }
            if target_site.is_some_and(|site| !sites.insert(site)) {
                return Err(SourceStructureError::DuplicateTarget { edge: id });
            }
            validate_edge_source_association(
                input,
                edge_index,
                target_range,
                target_spelling,
                member_groups,
                update_groups,
            )?;
            previous_range = Some(target_range);
        }
    }

    validate_member_edge_references(input, member_groups, update_groups)?;
    Ok(groups)
}

fn direct_input_targets(
    input: &SourceStructureHandoffInput,
    primary_terms: &SourcePrimaryTermHandoff,
    applications: Option<&SourceFunctorApplicationHandoff>,
    effective: &[(SourceRange, String)],
    term_index: usize,
) -> Result<Vec<(SourceStructureTarget, SourceRange)>, SourceStructureError> {
    let parent = &input.terms[term_index];
    let application_owned_primaries = applications
        .map(application_argument_primary_ids)
        .unwrap_or_default();
    let mut candidates = Vec::new();
    for (id, primary) in primary_terms.terms().iter() {
        if primary.parent().is_none()
            && primary.context() == parent.context
            && !application_owned_primaries.contains(&id)
            && properly_contains(parent.source_range, primary.source_range())
        {
            candidates.push((SourceStructureTarget::Primary(id), primary.source_range()));
        }
    }
    if let Some(applications) = applications {
        for id in application_root_ids(applications) {
            let application = applications
                .applications()
                .get(id)
                .ok_or(SourceStructureError::ApplicationDependencyMismatch)?;
            let range = application_effective_occurrence(applications, id)
                .ok_or(SourceStructureError::ApplicationDependencyMismatch)?
                .0;
            if application.context() == parent.context
                && properly_contains(parent.source_range, range)
            {
                candidates.push((SourceStructureTarget::Application(id), range));
            }
        }
    }
    for (child_index, child) in input.terms.iter().enumerate().skip(term_index + 1) {
        let range = effective[child_index].0;
        if child.context == parent.context && properly_contains(parent.source_range, range) {
            candidates.push((
                SourceStructureTarget::Structure(SourceStructureTermId::new(child_index)),
                range,
            ));
        }
    }

    let all = candidates.clone();
    candidates.retain(|(_, candidate_range)| {
        !all.iter().any(|(_, container_range)| {
            container_range != candidate_range
                && properly_contains(*container_range, *candidate_range)
        })
    });
    candidates.sort_by_key(|(target, range)| (range.start, range.end, *target));
    Ok(candidates)
}

fn validate_edge_source_association(
    input: &SourceStructureHandoffInput,
    edge_index: usize,
    target_range: SourceRange,
    target_spelling: &str,
    member_groups: &[Vec<usize>],
    update_groups: &[Vec<usize>],
) -> Result<(), SourceStructureError> {
    let id = SourceStructureEdgeId::new(edge_index);
    let edge = &input.edges[edge_index];
    let invalid = || SourceStructureError::InvalidEdge { edge: id };
    match edge.role {
        SourceStructureEdgeRole::ConstructorValue => {
            let member_id = edge.member.ok_or_else(invalid)?;
            let members = &member_groups[edge.term.index()];
            let position = members
                .iter()
                .position(|member| *member == member_id.index())
                .ok_or_else(invalid)?;
            let member = input.members.get(member_id.index()).ok_or_else(invalid)?;
            let before_next_member = members
                .get(position + 1)
                .is_none_or(|next| target_range.end <= input.members[*next].source_range.start);
            if member.source_range.end > target_range.start || !before_next_member {
                return Err(invalid());
            }
        }
        SourceStructureEdgeRole::SelectorBase => {
            let [member_index] = member_groups[edge.term.index()].as_slice() else {
                return Err(invalid());
            };
            if target_range.end > input.members[*member_index].source_range.start {
                return Err(invalid());
            }
        }
        SourceStructureEdgeRole::SelectorArgument => {
            let [member_index] = member_groups[edge.term.index()].as_slice() else {
                return Err(invalid());
            };
            if input.members[*member_index].source_range.end > target_range.start {
                return Err(invalid());
            }
        }
        SourceStructureEdgeRole::UpdateBase => {
            let Some(first_update) = update_groups[edge.term.index()].first() else {
                return Err(invalid());
            };
            if target_range.end > input.field_updates[*first_update].source_range.start {
                return Err(invalid());
            }
        }
        SourceStructureEdgeRole::UpdateValue => {
            let member_id = edge.member.ok_or_else(invalid)?;
            let update_index = update_groups[edge.term.index()]
                .iter()
                .copied()
                .find(|update| input.field_updates[*update].final_member == member_id)
                .ok_or_else(invalid)?;
            let update = &input.field_updates[update_index];
            let final_member = input
                .members
                .get(update.final_member.index())
                .ok_or_else(invalid)?;
            if !properly_contains(update.source_range, target_range)
                || final_member.source_range.end > target_range.start
            {
                return Err(invalid());
            }
            let path = (update.first_member.index()..=update.final_member.index())
                .map(|member| input.members[member].spelling.as_str())
                .collect::<Vec<_>>()
                .join(" . ");
            if update.spelling != format!("{path} := {target_spelling}") {
                return Err(SourceStructureError::InvalidFieldUpdate {
                    field_update: SourceFieldUpdateId::new(update_index),
                });
            }
        }
    }
    Ok(())
}

fn validate_member_edge_references(
    input: &SourceStructureHandoffInput,
    member_groups: &[Vec<usize>],
    update_groups: &[Vec<usize>],
) -> Result<(), SourceStructureError> {
    for (edge_index, edge) in input.edges.iter().enumerate() {
        let id = SourceStructureEdgeId::new(edge_index);
        match edge.role {
            SourceStructureEdgeRole::ConstructorValue => {
                let Some(member_id) = edge.member else {
                    return Err(SourceStructureError::InvalidEdge { edge: id });
                };
                let Some(member) = input.members.get(member_id.index()) else {
                    return Err(SourceStructureError::InvalidEdge { edge: id });
                };
                if member.term != edge.term
                    || member.role != SourceStructureMemberRole::ConstructorAssignment
                    || !member_groups[edge.term.index()].contains(&member_id.index())
                {
                    return Err(SourceStructureError::InvalidEdge { edge: id });
                }
            }
            SourceStructureEdgeRole::UpdateValue => {
                let Some(member_id) = edge.member else {
                    return Err(SourceStructureError::InvalidEdge { edge: id });
                };
                let Some(member) = input.members.get(member_id.index()) else {
                    return Err(SourceStructureError::InvalidEdge { edge: id });
                };
                if member.term != edge.term
                    || member.role != SourceStructureMemberRole::UpdatePathSegment
                    || !update_groups[edge.term.index()]
                        .iter()
                        .any(|update| input.field_updates[*update].final_member == member_id)
                {
                    return Err(SourceStructureError::InvalidEdge { edge: id });
                }
            }
            SourceStructureEdgeRole::SelectorBase
            | SourceStructureEdgeRole::SelectorArgument
            | SourceStructureEdgeRole::UpdateBase => {
                if edge.member.is_some() {
                    return Err(SourceStructureError::InvalidEdge { edge: id });
                }
            }
        }
    }
    Ok(())
}

fn validate_structure_ownership(
    input: &SourceStructureHandoffInput,
    effective: &[(SourceRange, String)],
) -> Result<(), SourceStructureError> {
    let targets = input
        .edges
        .iter()
        .filter_map(|edge| match edge.target {
            SourceStructureTarget::Structure(term) => Some(term),
            SourceStructureTarget::Primary(_) | SourceStructureTarget::Application(_) => None,
        })
        .collect::<BTreeSet<_>>();
    for (index, occurrence) in effective.iter().enumerate().take(input.terms.len()) {
        let term = SourceStructureTermId::new(index);
        let geometrically_nested = (0..index)
            .any(|parent| properly_contains(input.terms[parent].source_range, occurrence.0));
        if geometrically_nested != targets.contains(&term) {
            return Err(SourceStructureError::InvalidTerm { term });
        }
    }
    Ok(())
}

fn validate_edge_shapes(
    input: &SourceStructureHandoffInput,
    member_groups: &[Vec<usize>],
    update_groups: &[Vec<usize>],
    edge_groups: &[Vec<usize>],
) -> Result<(), SourceStructureError> {
    for (term_index, term) in input.terms.iter().enumerate() {
        let term_id = SourceStructureTermId::new(term_index);
        let members = &member_groups[term_index];
        let updates = &update_groups[term_index];
        let edges = &edge_groups[term_index];
        match term.kind {
            SourceStructureTermKind::Constructor => {
                if edges.len() != members.len() {
                    return Err(SourceStructureError::InvalidTerm { term: term_id });
                }
                for (ordinal, edge_index) in edges.iter().copied().enumerate() {
                    let edge = &input.edges[edge_index];
                    if edge.role != SourceStructureEdgeRole::ConstructorValue
                        || edge.member != Some(SourceStructureMemberId::new(members[ordinal]))
                    {
                        return Err(SourceStructureError::InvalidEdge {
                            edge: SourceStructureEdgeId::new(edge_index),
                        });
                    }
                }
            }
            SourceStructureTermKind::SelectorAccess => {
                let Some(first) = edges.first().copied() else {
                    return Err(SourceStructureError::InvalidTerm { term: term_id });
                };
                if input.edges[first].role != SourceStructureEdgeRole::SelectorBase {
                    return Err(SourceStructureError::InvalidEdge {
                        edge: SourceStructureEdgeId::new(first),
                    });
                }
                for edge_index in edges.iter().skip(1).copied() {
                    if input.edges[edge_index].role != SourceStructureEdgeRole::SelectorArgument {
                        return Err(SourceStructureError::InvalidEdge {
                            edge: SourceStructureEdgeId::new(edge_index),
                        });
                    }
                }
            }
            SourceStructureTermKind::FunctionalUpdate => {
                if edges.len() != updates.len() + 1 {
                    return Err(SourceStructureError::InvalidTerm { term: term_id });
                }
                let first = edges[0];
                if input.edges[first].role != SourceStructureEdgeRole::UpdateBase {
                    return Err(SourceStructureError::InvalidEdge {
                        edge: SourceStructureEdgeId::new(first),
                    });
                }
                for (ordinal, edge_index) in edges.iter().skip(1).copied().enumerate() {
                    let edge = &input.edges[edge_index];
                    if edge.role != SourceStructureEdgeRole::UpdateValue
                        || edge.member != Some(input.field_updates[updates[ordinal]].final_member)
                    {
                        return Err(SourceStructureError::InvalidEdge {
                            edge: SourceStructureEdgeId::new(edge_index),
                        });
                    }
                }
            }
        }
    }
    Ok(())
}

fn validate_requests(
    input: &SourceStructureHandoffInput,
    member_groups: &[Vec<usize>],
) -> Result<(), SourceStructureError> {
    let groups = grouped_rows(
        input.terms.len(),
        &input.requests,
        |row| row.term,
        |row| row.request_ordinal,
        |index| SourceStructureError::InvalidRequest {
            request: SourceStructureRequestId::new(index),
        },
    )?;
    for (term_index, requests) in groups.iter().enumerate() {
        let term = &input.terms[term_index];
        let members = &member_groups[term_index];
        let expected_len =
            usize::from(term.kind == SourceStructureTermKind::Constructor) + members.len() * 2 + 1;
        if requests.len() != expected_len {
            return Err(SourceStructureError::InvalidRequest {
                request: SourceStructureRequestId::new(
                    requests.first().copied().unwrap_or(input.requests.len()),
                ),
            });
        }
        let mut ordinal = 0;
        if term.kind == SourceStructureTermKind::Constructor {
            let request_index = requests[ordinal];
            let request = &input.requests[request_index];
            if request.kind != SourceStructureRequestKind::ConstructorSignature
                || request.member.is_some()
            {
                return Err(SourceStructureError::InvalidRequest {
                    request: SourceStructureRequestId::new(request_index),
                });
            }
            ordinal += 1;
        }
        for member_index in members {
            let member = SourceStructureMemberId::new(*member_index);
            for expected_kind in [
                SourceStructureRequestKind::MemberIdentity,
                SourceStructureRequestKind::InheritancePath,
            ] {
                let request_index = requests[ordinal];
                let request = &input.requests[request_index];
                if request.kind != expected_kind || request.member != Some(member) {
                    return Err(SourceStructureError::InvalidRequest {
                        request: SourceStructureRequestId::new(request_index),
                    });
                }
                ordinal += 1;
            }
        }
        let request_index = requests[ordinal];
        let request = &input.requests[request_index];
        if request.kind != SourceStructureRequestKind::ResultType || request.member.is_some() {
            return Err(SourceStructureError::InvalidRequest {
                request: SourceStructureRequestId::new(request_index),
            });
        }
    }
    Ok(())
}

fn validate_output_targets(
    handoff: &SourceStructureHandoff,
    primary_terms: &SourcePrimaryTermHandoff,
    applications: Option<&SourceFunctorApplicationHandoff>,
) -> Result<(), SourceStructureError> {
    let application_roots = applications.map(application_root_ids);
    let effective = output_effective_occurrences(handoff);
    let mut primary_targets = BTreeSet::new();
    let mut application_targets = BTreeSet::new();
    let mut structure_targets = BTreeSet::new();
    for (id, edge) in handoff.edges.iter() {
        let parent = handoff
            .terms
            .get(edge.term)
            .ok_or(SourceStructureError::InvalidEdge { edge: id })?;
        let range = match edge.target {
            SourceStructureTarget::Primary(primary_id) => {
                let primary = primary_terms
                    .terms()
                    .get(primary_id)
                    .ok_or(SourceStructureError::InvalidEdge { edge: id })?;
                if primary.parent().is_some()
                    || primary.context() != parent.context
                    || !primary_targets.insert(primary_id)
                {
                    return Err(SourceStructureError::InvalidEdge { edge: id });
                }
                primary.source_range()
            }
            SourceStructureTarget::Application(application_id) => {
                let applications =
                    applications.ok_or(SourceStructureError::ApplicationDependencyMismatch)?;
                let application = applications
                    .applications()
                    .get(application_id)
                    .ok_or(SourceStructureError::InvalidEdge { edge: id })?;
                if application.context() != parent.context
                    || !application_roots
                        .as_ref()
                        .is_some_and(|roots| roots.contains(&application_id))
                    || !application_targets.insert(application_id)
                {
                    return Err(SourceStructureError::InvalidEdge { edge: id });
                }
                application_effective_occurrence(applications, application_id)
                    .ok_or(SourceStructureError::InvalidEdge { edge: id })?
                    .0
            }
            SourceStructureTarget::Structure(term_id) => {
                let child = handoff
                    .terms
                    .get(term_id)
                    .ok_or(SourceStructureError::InvalidEdge { edge: id })?;
                if term_id <= edge.term
                    || child.context != parent.context
                    || !structure_targets.insert(term_id)
                {
                    return Err(SourceStructureError::InvalidEdge { edge: id });
                }
                effective[term_id.index()].0
            }
        };
        if !properly_contains(parent.source_range, range) {
            return Err(SourceStructureError::InvalidEdge { edge: id });
        }
    }
    Ok(())
}

fn validate_output_arena_and_references(
    handoff: &SourceStructureHandoff,
    arena: &TypedArena,
) -> Result<(), SourceStructureError> {
    let mut sites = BTreeSet::new();
    for (id, term) in handoff.terms.iter() {
        validate_arena_site(
            &term.site,
            term.source_range,
            term_kind_node_key(term.kind),
            term.recovery,
            arena,
        )
        .map_err(|()| SourceStructureError::InvalidTerm { term: id })?;
        if !sites.insert(term.site.clone()) {
            return Err(SourceStructureError::DuplicateSite);
        }
    }
    for (id, wrapper) in handoff.wrappers.iter() {
        let term = handoff
            .terms
            .get(wrapper.term)
            .ok_or(SourceStructureError::InvalidWrapper { wrapper: id })?;
        if wrapper.context != term.context {
            return Err(SourceStructureError::InvalidWrapper { wrapper: id });
        }
        validate_arena_site(
            &wrapper.site,
            wrapper.source_range,
            "source.term.structure.parenthesized",
            wrapper.recovery,
            arena,
        )
        .map_err(|()| SourceStructureError::InvalidWrapper { wrapper: id })?;
        if !sites.insert(wrapper.site.clone()) {
            return Err(SourceStructureError::DuplicateSite);
        }
    }
    for (id, root) in handoff.roots.iter() {
        let term = handoff
            .terms
            .get(root.term)
            .ok_or(SourceStructureError::InvalidRoot { root: id })?;
        if term.kind != SourceStructureTermKind::Constructor {
            return Err(SourceStructureError::InvalidRoot { root: id });
        }
    }
    for (id, member) in handoff.members.iter() {
        let term = handoff
            .terms
            .get(member.term)
            .ok_or(SourceStructureError::InvalidMember { member: id })?;
        if let Some(parent) = member.parent {
            let parent = handoff
                .members
                .get(parent)
                .ok_or(SourceStructureError::InvalidMember { member: id })?;
            if parent.term != member.term {
                return Err(SourceStructureError::InvalidMember { member: id });
            }
        }
        validate_arena_site(
            &member.site,
            member.source_range,
            member_role_node_key(member.role),
            term.recovery,
            arena,
        )
        .map_err(|()| SourceStructureError::InvalidMember { member: id })?;
        if !sites.insert(member.site.clone()) {
            return Err(SourceStructureError::DuplicateSite);
        }
    }
    for (id, update) in handoff.field_updates.iter() {
        let term = handoff
            .terms
            .get(update.term)
            .ok_or(SourceStructureError::InvalidFieldUpdate { field_update: id })?;
        let first = handoff.members.get(update.first_member);
        let final_member = handoff.members.get(update.final_member);
        if term.kind != SourceStructureTermKind::FunctionalUpdate
            || first.is_none_or(|member| member.term != update.term)
            || final_member.is_none_or(|member| member.term != update.term)
        {
            return Err(SourceStructureError::InvalidFieldUpdate { field_update: id });
        }
        validate_arena_site(
            &update.site,
            update.source_range,
            "source.term.structure.field-update",
            term.recovery,
            arena,
        )
        .map_err(|()| SourceStructureError::InvalidFieldUpdate { field_update: id })?;
        if !sites.insert(update.site.clone()) {
            return Err(SourceStructureError::DuplicateSite);
        }
    }
    for (id, edge) in handoff.edges.iter() {
        if handoff.terms.get(edge.term).is_none()
            || edge
                .member
                .is_some_and(|member| handoff.members.get(member).is_none())
        {
            return Err(SourceStructureError::InvalidEdge { edge: id });
        }
    }
    for (id, request) in handoff.requests.iter() {
        if handoff.terms.get(request.term).is_none()
            || request
                .member
                .is_some_and(|member| handoff.members.get(member).is_none())
        {
            return Err(SourceStructureError::InvalidRequest { request: id });
        }
    }
    Ok(())
}

fn grouped_rows<T, FTerm, FOrdinal, FError>(
    term_count: usize,
    rows: &[T],
    term: FTerm,
    ordinal: FOrdinal,
    error: FError,
) -> Result<Vec<Vec<usize>>, SourceStructureError>
where
    FTerm: Fn(&T) -> SourceStructureTermId,
    FOrdinal: Fn(&T) -> usize,
    FError: Fn(usize) -> SourceStructureError,
{
    let mut groups = vec![Vec::new(); term_count];
    let mut previous_term = 0;
    for (index, row) in rows.iter().enumerate() {
        let term_id = term(row);
        let term_index = term_id.index();
        let Some(group) = groups.get_mut(term_index) else {
            return Err(error(index));
        };
        if (index > 0 && term_index < previous_term) || ordinal(row) != group.len() {
            return Err(error(index));
        }
        group.push(index);
        previous_term = term_index;
    }
    Ok(groups)
}

fn input_effective_occurrences(
    input: &SourceStructureHandoffInput,
    wrapper_groups: &[Vec<usize>],
) -> Vec<(SourceRange, String)> {
    input
        .terms
        .iter()
        .enumerate()
        .map(|(index, term)| {
            wrapper_groups[index].first().map_or_else(
                || (term.source_range, term.spelling.clone()),
                |wrapper| {
                    let wrapper = &input.wrappers[*wrapper];
                    (wrapper.source_range, wrapper.spelling.clone())
                },
            )
        })
        .collect()
}

fn output_effective_occurrences(handoff: &SourceStructureHandoff) -> Vec<(SourceRange, &str)> {
    handoff
        .terms
        .iter()
        .map(|(term_id, term)| {
            handoff
                .wrappers
                .iter()
                .find(|(_, wrapper)| wrapper.term == term_id && wrapper.ordinal == 0)
                .map_or(
                    (term.source_range, term.spelling.as_str()),
                    |(_, wrapper)| (wrapper.source_range, wrapper.spelling.as_str()),
                )
        })
        .collect()
}

fn application_root_ids(
    handoff: &SourceFunctorApplicationHandoff,
) -> BTreeSet<SourceFunctorApplicationId> {
    let nested = handoff
        .arguments()
        .iter()
        .filter_map(|(_, argument)| match argument.target() {
            crate::source_application::SourceFunctorArgumentTarget::Application(id) => Some(id),
            crate::source_application::SourceFunctorArgumentTarget::Primary(_) => None,
        })
        .collect::<BTreeSet<_>>();
    handoff
        .applications()
        .iter()
        .map(|(id, _)| id)
        .filter(|id| !nested.contains(id))
        .collect()
}

fn application_argument_primary_ids(
    handoff: &SourceFunctorApplicationHandoff,
) -> BTreeSet<SourcePrimaryTermId> {
    handoff
        .arguments()
        .iter()
        .filter_map(|(_, argument)| match argument.target() {
            crate::source_application::SourceFunctorArgumentTarget::Primary(id) => Some(id),
            crate::source_application::SourceFunctorArgumentTarget::Application(_) => None,
        })
        .collect()
}

fn validate_input_application_ownership(
    input: &SourceStructureHandoffInput,
    effective: &[(SourceRange, String)],
    applications: Option<&SourceFunctorApplicationHandoff>,
) -> Result<(), SourceStructureError> {
    let term_ranges = effective
        .iter()
        .enumerate()
        .map(|(index, (range, _))| (SourceStructureTermId::new(index), *range))
        .collect::<Vec<_>>();
    let targets = input
        .edges
        .iter()
        .map(|edge| (edge.term, edge.target))
        .collect::<Vec<_>>();
    validate_application_ownership(&term_ranges, &targets, applications)
}

fn validate_output_application_ownership(
    handoff: &SourceStructureHandoff,
    applications: Option<&SourceFunctorApplicationHandoff>,
) -> Result<(), SourceStructureError> {
    let term_ranges = output_effective_occurrences(handoff)
        .into_iter()
        .enumerate()
        .map(|(index, (range, _))| (SourceStructureTermId::new(index), range))
        .collect::<Vec<_>>();
    let targets = handoff
        .edges
        .iter()
        .map(|(_, edge)| (edge.term, edge.target))
        .collect::<Vec<_>>();
    validate_application_ownership(&term_ranges, &targets, applications)
}

fn validate_application_ownership(
    term_ranges: &[(SourceStructureTermId, SourceRange)],
    targets: &[(SourceStructureTermId, SourceStructureTarget)],
    applications: Option<&SourceFunctorApplicationHandoff>,
) -> Result<(), SourceStructureError> {
    let Some(applications) = applications else {
        return Ok(());
    };
    let application_owned_primaries = application_argument_primary_ids(applications);
    if targets.iter().any(|(_, target)| {
        matches!(
            target,
            SourceStructureTarget::Primary(primary)
                if application_owned_primaries.contains(primary)
        )
    }) {
        return Err(SourceStructureError::ApplicationDependencyMismatch);
    }

    for application_id in application_root_ids(applications) {
        let application_range = application_effective_occurrence(applications, application_id)
            .ok_or(SourceStructureError::ApplicationDependencyMismatch)?
            .0;
        let mut containing_terms = Vec::new();
        for (term_id, term_range) in term_ranges {
            if properly_contains(application_range, *term_range)
                || (ranges_overlap(application_range, *term_range)
                    && !properly_contains(*term_range, application_range))
            {
                return Err(SourceStructureError::ApplicationDependencyMismatch);
            }
            if properly_contains(*term_range, application_range) {
                containing_terms.push((*term_id, *term_range));
            }
        }
        if let Some((owner, _)) = containing_terms
            .into_iter()
            .min_by_key(|(_, range)| range.end - range.start)
            && !targets.iter().any(|(term, target)| {
                *term == owner && *target == SourceStructureTarget::Application(application_id)
            })
        {
            return Err(SourceStructureError::ApplicationDependencyMismatch);
        }
    }
    Ok(())
}

fn ranges_overlap(left: SourceRange, right: SourceRange) -> bool {
    left.source_id == right.source_id && left.start < right.end && right.start < left.end
}

fn application_effective_occurrence(
    handoff: &SourceFunctorApplicationHandoff,
    id: SourceFunctorApplicationId,
) -> Option<(SourceRange, TypedSiteRef, &str)> {
    let application = handoff.applications().get(id)?;
    Some(
        handoff
            .wrappers()
            .iter()
            .find(|(_, wrapper)| wrapper.application() == id && wrapper.ordinal() == 0)
            .map_or(
                (
                    application.source_range(),
                    application.site().clone(),
                    application.spelling(),
                ),
                |(_, wrapper)| {
                    (
                        wrapper.source_range(),
                        wrapper.site().clone(),
                        wrapper.spelling(),
                    )
                },
            ),
    )
}

fn validate_arena_site(
    site: &TypedSiteRef,
    source_range: SourceRange,
    kind: &str,
    recovery: SourceStructureRecovery,
    arena: &TypedArena,
) -> Result<(), ()> {
    let TypedSiteRef::Node(node_id) = site else {
        return Err(());
    };
    let node = arena.node(*node_id).ok_or(())?;
    if node.anchor != SourceAnchor::Range(source_range)
        || node.kind.as_str() != kind
        || !recovery_matches(recovery, node.recovery)
    {
        return Err(());
    }
    Ok(())
}

fn valid_range(source_id: SourceId, range: SourceRange) -> bool {
    range.source_id == source_id && range.start < range.end
}

fn range_contains(parent: SourceRange, child: SourceRange) -> bool {
    parent.source_id == child.source_id && parent.start <= child.start && parent.end >= child.end
}

fn strictly_contains(parent: SourceRange, child: SourceRange) -> bool {
    parent.source_id == child.source_id && parent.start < child.start && parent.end > child.end
}

fn properly_contains(parent: SourceRange, child: SourceRange) -> bool {
    range_contains(parent, child) && parent != child
}

fn source_range(anchor: &SourceAnchor) -> Option<SourceRange> {
    match anchor {
        SourceAnchor::Range(range) => Some(*range),
        SourceAnchor::Point { .. } | SourceAnchor::Generated(_) | _ => None,
    }
}

fn canonical_spelling(spelling: &str) -> bool {
    !spelling.is_empty()
        && spelling.trim() == spelling
        && !spelling.contains("  ")
        && !spelling
            .chars()
            .any(|character| character.is_whitespace() && character != ' ')
}

fn identifier_spelling(spelling: &str) -> bool {
    let mut characters = spelling.chars();
    characters
        .next()
        .is_some_and(|first| first == '_' || first.is_ascii_alphabetic())
        && characters.all(|character| character == '_' || character.is_ascii_alphanumeric())
}

fn recovery_matches(recovery: SourceStructureRecovery, node_recovery: NodeRecoveryState) -> bool {
    match recovery {
        SourceStructureRecovery::Normal => node_recovery == NodeRecoveryState::Normal,
        SourceStructureRecovery::Degraded => matches!(
            node_recovery,
            NodeRecoveryState::Recovered | NodeRecoveryState::Degraded
        ),
    }
}

fn term_kind_node_key(kind: SourceStructureTermKind) -> &'static str {
    match kind {
        SourceStructureTermKind::Constructor => "source.term.structure.constructor",
        SourceStructureTermKind::SelectorAccess => "source.term.structure.selector",
        SourceStructureTermKind::FunctionalUpdate => "source.term.structure.update",
    }
}

fn term_kind_key(kind: SourceStructureTermKind) -> &'static str {
    match kind {
        SourceStructureTermKind::Constructor => "constructor",
        SourceStructureTermKind::SelectorAccess => "selector-access",
        SourceStructureTermKind::FunctionalUpdate => "functional-update",
    }
}

fn member_role_node_key(role: SourceStructureMemberRole) -> &'static str {
    match role {
        SourceStructureMemberRole::ConstructorAssignment => {
            "source.term.structure.member.constructor-assignment"
        }
        SourceStructureMemberRole::Selector => "source.term.structure.member.selector",
        SourceStructureMemberRole::UpdatePathSegment => {
            "source.term.structure.member.update-path-segment"
        }
    }
}

fn member_role_key(role: SourceStructureMemberRole) -> &'static str {
    match role {
        SourceStructureMemberRole::ConstructorAssignment => "constructor-assignment",
        SourceStructureMemberRole::Selector => "selector",
        SourceStructureMemberRole::UpdatePathSegment => "update-path-segment",
    }
}

fn edge_role_key(role: SourceStructureEdgeRole) -> &'static str {
    match role {
        SourceStructureEdgeRole::ConstructorValue => "constructor-value",
        SourceStructureEdgeRole::SelectorBase => "selector-base",
        SourceStructureEdgeRole::SelectorArgument => "selector-argument",
        SourceStructureEdgeRole::UpdateBase => "update-base",
        SourceStructureEdgeRole::UpdateValue => "update-value",
    }
}

fn request_kind_key(kind: SourceStructureRequestKind) -> &'static str {
    match kind {
        SourceStructureRequestKind::ConstructorSignature => "constructor-signature",
        SourceStructureRequestKind::MemberIdentity => "member-identity",
        SourceStructureRequestKind::InheritancePath => "inheritance-path",
        SourceStructureRequestKind::ResultType => "result-type",
    }
}

fn recovery_key(recovery: SourceStructureRecovery) -> &'static str {
    match recovery {
        SourceStructureRecovery::Normal => "normal",
        SourceStructureRecovery::Degraded => "degraded",
    }
}

fn write_optional_member(output: &mut String, member: Option<SourceStructureMemberId>) {
    if let Some(member) = member {
        let _ = write!(output, "{}", member.index());
    } else {
        output.push('-');
    }
}

fn write_target(output: &mut String, target: SourceStructureTarget) {
    match target {
        SourceStructureTarget::Primary(id) => {
            let _ = write!(output, "primary({})", id.index());
        }
        SourceStructureTarget::Application(id) => {
            let _ = write!(output, "application({})", id.index());
        }
        SourceStructureTarget::Structure(id) => {
            let _ = write!(output, "structure({})", id.index());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        binding_env::{
            BindingContextDraft, BindingContextLayer, BindingContextOwner, BindingContextRecovery,
            BindingContextTable, BindingDiagnosticTable, BindingEnvParts, BindingTable,
        },
        source_application::{
            SourceFunctorApplicationForm, SourceFunctorApplicationHandoffInput,
            SourceFunctorApplicationInput, SourceFunctorApplicationKind,
            SourceFunctorApplicationProducer, SourceFunctorApplicationRecovery,
            SourceFunctorArgumentInput, SourceFunctorArgumentTarget, SourceFunctorCandidateId,
            SourceFunctorCandidateInput, SourceFunctorHeadSite, SourceFunctorTypeRequestInput,
            SourceFunctorTypeRequestKind, SourceFunctorWrapperInput,
        },
        source_term::{
            SourceNumericTypeRequestInput, SourcePrimaryTermHandoffInput, SourcePrimaryTermInput,
            SourcePrimaryTermKind, SourcePrimaryTermProducer, SourcePrimaryTermRecovery,
            SourcePrimaryTermRole,
        },
        typed_ast::{
            CoercionTable, InitialObligationTable, LocalTypeContextTable, TypeDiagnosticTable,
            TypeFactTable, TypeTable, TypedAst, TypedAstError, TypedAstParts, TypedNode,
            TypedNodeId,
        },
    };
    use mizar_resolve::{
        env::{DeclarationConflictClass, DefinitionShell, NamespacePath, SymbolEnvIndexes},
        resolved_ast::{FullyQualifiedName, LocalSymbolId},
    };
    use mizar_session::{
        BuildSnapshotId, InMemorySessionIdAllocator, ModulePath, PackageId, SessionIdAllocator as _,
    };

    #[derive(Clone)]
    struct Fixture {
        source: SourceId,
        module: ModuleId,
        input: SourceStructureHandoffInput,
        symbols: SymbolEnv,
        bindings: BindingEnv,
        primary: SourcePrimaryTermHandoff,
        applications: Option<SourceFunctorApplicationHandoff>,
        arena: TypedArena,
    }

    #[derive(Clone)]
    struct RootOptions {
        kind: SymbolKind,
        signature: Option<SignatureShell>,
        definition_signature: Option<SignatureShell>,
        conflict: Option<DeclarationConflictClass>,
        recovered: bool,
        include_definition: bool,
        definition_kind: DefinitionKind,
        origin_after_use: bool,
        namespace_drift: bool,
        imported_contribution: bool,
        contribution_module_drift: bool,
        contribution_source_drift: bool,
        omit_symbol_effect: bool,
        symbol_contribution_drift: bool,
        definition_contribution_drift: bool,
        definition_origin_drift: bool,
        definition_visibility_drift: bool,
        omit_definition_effect: bool,
    }

    impl Default for RootOptions {
        fn default() -> Self {
            Self {
                kind: SymbolKind::Structure,
                signature: None,
                definition_signature: None,
                conflict: None,
                recovered: false,
                include_definition: true,
                definition_kind: DefinitionKind::Structure,
                origin_after_use: false,
                namespace_drift: false,
                imported_contribution: false,
                contribution_module_drift: false,
                contribution_source_drift: false,
                omit_symbol_effect: false,
                symbol_contribution_drift: false,
                definition_contribution_drift: false,
                definition_origin_drift: false,
                definition_visibility_drift: false,
                omit_definition_effect: false,
            }
        }
    }

    fn source_id(byte: &str) -> SourceId {
        let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{}",
            byte.repeat(32)
        ))
        .expect("snapshot");
        InMemorySessionIdAllocator::new()
            .next_source_id(snapshot)
            .expect("source")
    }

    fn other_source_id(byte: &str) -> SourceId {
        let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{}",
            byte.repeat(32)
        ))
        .expect("snapshot");
        let allocator = InMemorySessionIdAllocator::new();
        allocator.next_source_id(snapshot).expect("first source");
        allocator.next_source_id(snapshot).expect("second source")
    }

    fn module(path: &str) -> ModuleId {
        ModuleId::new(PackageId::new("pkg"), ModulePath::new(path))
    }

    fn range(source: SourceId, start: usize, end: usize) -> SourceRange {
        SourceRange {
            source_id: source,
            start,
            end,
        }
    }

    fn node(index: usize) -> TypedSiteRef {
        TypedSiteRef::Node(TypedNodeId::new(index))
    }

    fn binding_env(source: SourceId, module: &ModuleId) -> BindingEnv {
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
        BindingEnv::try_new(BindingEnvParts {
            source_id: source,
            module_id: module.clone(),
            contexts,
            bindings: BindingTable::new(),
            diagnostics: BindingDiagnosticTable::new(),
        })
        .expect("binding environment")
    }

    fn binding_env_with_two_contexts(source: SourceId, module: &ModuleId) -> BindingEnv {
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
        contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::Module,
            parent: Some(root),
            layer: BindingContextLayer::Expression,
            lexical_scope: None,
            bindings: Vec::new(),
            visible_bindings: Vec::new(),
            recovery: BindingContextRecovery::Normal,
        });
        BindingEnv::try_new(BindingEnvParts {
            source_id: source,
            module_id: module.clone(),
            contexts,
            bindings: BindingTable::new(),
            diagnostics: BindingDiagnosticTable::new(),
        })
        .expect("two-context binding environment")
    }

    fn empty_typed_parts(fixture: &Fixture) -> TypedAstParts {
        TypedAstParts {
            source_id: fixture.source,
            module_id: fixture.module.clone(),
            resolved_root: None,
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes: fixture.arena.clone(),
            contexts: LocalTypeContextTable::new(),
            types: TypeTable::new(),
            facts: TypeFactTable::new(),
            coercions: CoercionTable::new(),
            initial_obligations: InitialObligationTable::new(),
            diagnostics: TypeDiagnosticTable::new(),
        }
    }

    fn symbol_id(module: &ModuleId, name: &str) -> SymbolId {
        SymbolId::new(
            module.clone(),
            LocalSymbolId::new(name),
            FullyQualifiedName::new(format!("{}::{name}", module.path().as_str())),
        )
    }

    fn local_structure_symbols(
        source: SourceId,
        module: &ModuleId,
        options: RootOptions,
    ) -> (SymbolEnv, SymbolId, SourceContributionId) {
        let mut indexes = SymbolEnvIndexes::default();
        let contribution_module = if options.contribution_module_drift {
            self::module("source.structure.drift")
        } else {
            module.clone()
        };
        let contribution_source = if options.contribution_source_drift {
            other_source_id("db")
        } else {
            source
        };
        let contribution = indexes.contributions.insert(
            contribution_module,
            if options.imported_contribution {
                ContributionKind::ImportedSource {
                    source_id: contribution_source,
                }
            } else {
                ContributionKind::LocalSource {
                    source_id: contribution_source,
                }
            },
            SourceAnchor::Range(range(source, 0, 9)),
        );
        let symbol_contribution = if options.symbol_contribution_drift {
            indexes.contributions.insert(
                module.clone(),
                ContributionKind::LocalSource { source_id: source },
                SourceAnchor::Range(range(source, 0, 9)),
            )
        } else {
            contribution
        };
        let symbol = symbol_id(module, "Pair");
        let origin_range = if options.origin_after_use {
            range(source, 40, 45)
        } else {
            range(source, 1, 5)
        };
        let mut origin = SemanticOrigin::new(
            source,
            module.clone(),
            SourceAnchor::Range(origin_range),
            vec![0],
        );
        if options.recovered {
            origin = origin.recovered();
        }
        let mut entry = SymbolEntry::new(
            symbol.clone(),
            options.kind,
            NamespacePath::new(if options.namespace_drift {
                "source.structure.other"
            } else {
                module.path().as_str()
            }),
            "Pair",
            origin.clone(),
            symbol_contribution,
        );
        if let Some(signature) = options.signature {
            entry = entry.with_signature(signature);
        }
        indexes.symbols.insert(entry);
        if !options.omit_symbol_effect {
            indexes
                .contributions
                .add_symbol(contribution, symbol.clone());
        }
        if options.include_definition {
            let definition_origin = if options.definition_origin_drift {
                SemanticOrigin::new(
                    source,
                    module.clone(),
                    SourceAnchor::Range(range(source, 2, 6)),
                    vec![1],
                )
            } else {
                origin
            };
            let definition_contribution = if options.definition_contribution_drift {
                indexes.contributions.insert(
                    module.clone(),
                    ContributionKind::LocalSource { source_id: source },
                    SourceAnchor::Range(range(source, 0, 9)),
                )
            } else {
                contribution
            };
            let mut definition = DefinitionShell::new(
                symbol.clone(),
                options.definition_kind,
                definition_origin,
                definition_contribution,
            );
            if options.definition_visibility_drift {
                definition = definition.with_visibility(Visibility::Public);
            }
            if let Some(signature) = options.definition_signature {
                definition = definition.with_signature(signature);
            }
            if let Some(conflict) = options.conflict {
                definition = definition.with_conflict(conflict);
            }
            let definition = indexes.definitions.insert(definition);
            if !options.omit_definition_effect {
                indexes
                    .contributions
                    .add_definition(definition_contribution, definition);
            }
        }
        (
            SymbolEnv::new(module.clone(), indexes),
            symbol,
            contribution,
        )
    }

    fn primary_handoff(
        source: SourceId,
        module: &ModuleId,
        bindings: &BindingEnv,
        arena: &TypedArena,
        occurrences: &[(usize, usize, usize, &str)],
    ) -> SourcePrimaryTermHandoff {
        SourcePrimaryTermProducer::build(
            SourcePrimaryTermHandoffInput {
                source_id: source,
                module_id: module.clone(),
                terms: occurrences
                    .iter()
                    .enumerate()
                    .map(
                        |(ordinal, (site, start, end, spelling))| SourcePrimaryTermInput {
                            site: node(*site),
                            source_range: range(source, *start, *end),
                            source_ordinal: ordinal,
                            context: BindingContextId::new(0),
                            recovery: SourcePrimaryTermRecovery::Normal,
                            spelling: (*spelling).to_owned(),
                            kind: SourcePrimaryTermKind::Numeral,
                            role: SourcePrimaryTermRole::Value,
                            parent: None,
                        },
                    )
                    .collect(),
                references: Vec::new(),
                numeric_type_requests: occurrences
                    .iter()
                    .enumerate()
                    .map(
                        |(ordinal, (site, start, end, spelling))| SourceNumericTypeRequestInput {
                            term: SourcePrimaryTermId::new(ordinal),
                            owner: node(*site),
                            source_range: range(source, *start, *end),
                            spelling: (*spelling).to_owned(),
                            request_ordinal: ordinal,
                        },
                    )
                    .collect(),
            },
            bindings,
            arena,
        )
        .expect("primary handoff")
    }

    fn constructor_fixture_with(options: RootOptions) -> Fixture {
        let source = source_id("d4");
        let module = module("source.structure");
        let bindings = binding_env(source, &module);
        let arena = TypedArena::try_new(
            None,
            vec![
                TypedNode::new(
                    "source.term.numeral",
                    SourceAnchor::Range(range(source, 22, 23)),
                ),
                TypedNode::new(
                    "source.term.structure.constructor",
                    SourceAnchor::Range(range(source, 10, 30)),
                ),
                TypedNode::new(
                    "source.term.structure.member.constructor-assignment",
                    SourceAnchor::Range(range(source, 13, 20)),
                ),
            ],
        )
        .expect("arena");
        let primary = primary_handoff(source, &module, &bindings, &arena, &[(0, 22, 23, "1")]);
        let (symbols, symbol, contribution) = local_structure_symbols(source, &module, options);
        let input = SourceStructureHandoffInput {
            source_id: source,
            module_id: module.clone(),
            terms: vec![SourceStructureTermInput {
                site: node(1),
                source_range: range(source, 10, 30),
                source_ordinal: 0,
                context: BindingContextId::new(0),
                recovery: SourceStructureRecovery::Normal,
                spelling: "Pair ( carrier : 1 )".to_owned(),
                kind: SourceStructureTermKind::Constructor,
            }],
            wrappers: Vec::new(),
            roots: vec![SourceStructureRootInput {
                term: SourceStructureTermId::new(0),
                symbol,
                contribution,
            }],
            members: vec![SourceStructureMemberInput {
                term: SourceStructureTermId::new(0),
                ordinal: 0,
                site: node(2),
                source_range: range(source, 13, 20),
                spelling: "carrier".to_owned(),
                role: SourceStructureMemberRole::ConstructorAssignment,
                parent: None,
            }],
            field_updates: Vec::new(),
            edges: vec![SourceStructureEdgeInput {
                term: SourceStructureTermId::new(0),
                ordinal: 0,
                role: SourceStructureEdgeRole::ConstructorValue,
                member: Some(SourceStructureMemberId::new(0)),
                target: SourceStructureTarget::Primary(SourcePrimaryTermId::new(0)),
            }],
            requests: vec![
                SourceStructureRequestInput {
                    term: SourceStructureTermId::new(0),
                    member: None,
                    request_ordinal: 0,
                    kind: SourceStructureRequestKind::ConstructorSignature,
                },
                SourceStructureRequestInput {
                    term: SourceStructureTermId::new(0),
                    member: Some(SourceStructureMemberId::new(0)),
                    request_ordinal: 1,
                    kind: SourceStructureRequestKind::MemberIdentity,
                },
                SourceStructureRequestInput {
                    term: SourceStructureTermId::new(0),
                    member: Some(SourceStructureMemberId::new(0)),
                    request_ordinal: 2,
                    kind: SourceStructureRequestKind::InheritancePath,
                },
                SourceStructureRequestInput {
                    term: SourceStructureTermId::new(0),
                    member: None,
                    request_ordinal: 3,
                    kind: SourceStructureRequestKind::ResultType,
                },
            ],
        };
        Fixture {
            source,
            module,
            input,
            symbols,
            bindings,
            primary,
            applications: None,
            arena,
        }
    }

    fn constructor_fixture() -> Fixture {
        constructor_fixture_with(RootOptions::default())
    }

    fn selector_fixture() -> Fixture {
        let mut fixture = constructor_fixture();
        fixture.input.terms[0].kind = SourceStructureTermKind::SelectorAccess;
        fixture.input.terms[0].spelling = "1 . carrier".to_owned();
        fixture.input.roots.clear();
        fixture.input.members[0].role = SourceStructureMemberRole::Selector;
        fixture.input.edges[0].role = SourceStructureEdgeRole::SelectorBase;
        fixture.input.edges[0].member = None;
        fixture.input.requests.remove(0);
        for (ordinal, request) in fixture.input.requests.iter_mut().enumerate() {
            request.request_ordinal = ordinal;
        }
        fixture.arena = TypedArena::try_new(
            None,
            vec![
                TypedNode::new(
                    "source.term.numeral",
                    SourceAnchor::Range(range(fixture.source, 11, 12)),
                ),
                TypedNode::new(
                    "source.term.structure.selector",
                    SourceAnchor::Range(range(fixture.source, 10, 30)),
                ),
                TypedNode::new(
                    "source.term.structure.member.selector",
                    SourceAnchor::Range(range(fixture.source, 13, 20)),
                ),
            ],
        )
        .expect("selector arena");
        fixture.primary = primary_handoff(
            fixture.source,
            &fixture.module,
            &fixture.bindings,
            &fixture.arena,
            &[(0, 11, 12, "1")],
        );
        fixture
    }

    fn functional_update_fixture() -> Fixture {
        let source = source_id("d5");
        let module = module("source.structure.update");
        let bindings = binding_env(source, &module);
        let arena = TypedArena::try_new(
            None,
            vec![
                TypedNode::new(
                    "source.term.numeral",
                    SourceAnchor::Range(range(source, 12, 13)),
                ),
                TypedNode::new(
                    "source.term.numeral",
                    SourceAnchor::Range(range(source, 30, 31)),
                ),
                TypedNode::new(
                    "source.term.structure.update",
                    SourceAnchor::Range(range(source, 10, 35)),
                ),
                TypedNode::new(
                    "source.term.structure.member.update-path-segment",
                    SourceAnchor::Range(range(source, 20, 25)),
                ),
                TypedNode::new(
                    "source.term.structure.member.update-path-segment",
                    SourceAnchor::Range(range(source, 26, 27)),
                ),
                TypedNode::new(
                    "source.term.structure.field-update",
                    SourceAnchor::Range(range(source, 18, 33)),
                ),
            ],
        )
        .expect("arena");
        let primary = primary_handoff(
            source,
            &module,
            &bindings,
            &arena,
            &[(0, 12, 13, "1"), (1, 30, 31, "2")],
        );
        let symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        let members = vec![
            SourceStructureMemberInput {
                term: SourceStructureTermId::new(0),
                ordinal: 0,
                site: node(3),
                source_range: range(source, 20, 25),
                spelling: "start".to_owned(),
                role: SourceStructureMemberRole::UpdatePathSegment,
                parent: None,
            },
            SourceStructureMemberInput {
                term: SourceStructureTermId::new(0),
                ordinal: 1,
                site: node(4),
                source_range: range(source, 26, 27),
                spelling: "x".to_owned(),
                role: SourceStructureMemberRole::UpdatePathSegment,
                parent: Some(SourceStructureMemberId::new(0)),
            },
        ];
        let requests = [
            (
                Some(SourceStructureMemberId::new(0)),
                SourceStructureRequestKind::MemberIdentity,
            ),
            (
                Some(SourceStructureMemberId::new(0)),
                SourceStructureRequestKind::InheritancePath,
            ),
            (
                Some(SourceStructureMemberId::new(1)),
                SourceStructureRequestKind::MemberIdentity,
            ),
            (
                Some(SourceStructureMemberId::new(1)),
                SourceStructureRequestKind::InheritancePath,
            ),
            (None, SourceStructureRequestKind::ResultType),
        ]
        .into_iter()
        .enumerate()
        .map(|(ordinal, (member, kind))| SourceStructureRequestInput {
            term: SourceStructureTermId::new(0),
            member,
            request_ordinal: ordinal,
            kind,
        })
        .collect();
        Fixture {
            source,
            module: module.clone(),
            input: SourceStructureHandoffInput {
                source_id: source,
                module_id: module,
                terms: vec![SourceStructureTermInput {
                    site: node(2),
                    source_range: range(source, 10, 35),
                    source_ordinal: 0,
                    context: BindingContextId::new(0),
                    recovery: SourceStructureRecovery::Normal,
                    spelling: "1 with ( start . x := 2 )".to_owned(),
                    kind: SourceStructureTermKind::FunctionalUpdate,
                }],
                wrappers: Vec::new(),
                roots: Vec::new(),
                members,
                field_updates: vec![SourceFieldUpdateInput {
                    term: SourceStructureTermId::new(0),
                    ordinal: 0,
                    site: node(5),
                    source_range: range(source, 18, 33),
                    spelling: "start . x := 2".to_owned(),
                    first_member: SourceStructureMemberId::new(0),
                    final_member: SourceStructureMemberId::new(1),
                }],
                edges: vec![
                    SourceStructureEdgeInput {
                        term: SourceStructureTermId::new(0),
                        ordinal: 0,
                        role: SourceStructureEdgeRole::UpdateBase,
                        member: None,
                        target: SourceStructureTarget::Primary(SourcePrimaryTermId::new(0)),
                    },
                    SourceStructureEdgeInput {
                        term: SourceStructureTermId::new(0),
                        ordinal: 1,
                        role: SourceStructureEdgeRole::UpdateValue,
                        member: Some(SourceStructureMemberId::new(1)),
                        target: SourceStructureTarget::Primary(SourcePrimaryTermId::new(1)),
                    },
                ],
                requests,
            },
            symbols,
            bindings,
            primary,
            applications: None,
            arena,
        }
    }

    fn repeated_update_fixture() -> Fixture {
        let mut fixture = functional_update_fixture();
        fixture.input.terms[0].source_range = range(fixture.source, 10, 60);
        fixture.input.terms[0].spelling = "1 with ( start . x := 2 , start . x := 3 )".to_owned();
        fixture.input.members.extend([
            SourceStructureMemberInput {
                term: SourceStructureTermId::new(0),
                ordinal: 2,
                site: node(6),
                source_range: range(fixture.source, 40, 45),
                spelling: "start".to_owned(),
                role: SourceStructureMemberRole::UpdatePathSegment,
                parent: None,
            },
            SourceStructureMemberInput {
                term: SourceStructureTermId::new(0),
                ordinal: 3,
                site: node(7),
                source_range: range(fixture.source, 46, 47),
                spelling: "x".to_owned(),
                role: SourceStructureMemberRole::UpdatePathSegment,
                parent: Some(SourceStructureMemberId::new(2)),
            },
        ]);
        fixture.input.field_updates.push(SourceFieldUpdateInput {
            term: SourceStructureTermId::new(0),
            ordinal: 1,
            site: node(8),
            source_range: range(fixture.source, 38, 55),
            spelling: "start . x := 3".to_owned(),
            first_member: SourceStructureMemberId::new(2),
            final_member: SourceStructureMemberId::new(3),
        });
        fixture.input.edges.push(SourceStructureEdgeInput {
            term: SourceStructureTermId::new(0),
            ordinal: 2,
            role: SourceStructureEdgeRole::UpdateValue,
            member: Some(SourceStructureMemberId::new(3)),
            target: SourceStructureTarget::Primary(SourcePrimaryTermId::new(2)),
        });
        fixture.input.requests.pop();
        for member in [2, 3] {
            fixture.input.requests.extend([
                SourceStructureRequestInput {
                    term: SourceStructureTermId::new(0),
                    member: Some(SourceStructureMemberId::new(member)),
                    request_ordinal: fixture.input.requests.len(),
                    kind: SourceStructureRequestKind::MemberIdentity,
                },
                SourceStructureRequestInput {
                    term: SourceStructureTermId::new(0),
                    member: Some(SourceStructureMemberId::new(member)),
                    request_ordinal: fixture.input.requests.len() + 1,
                    kind: SourceStructureRequestKind::InheritancePath,
                },
            ]);
        }
        fixture.input.requests.push(SourceStructureRequestInput {
            term: SourceStructureTermId::new(0),
            member: None,
            request_ordinal: fixture.input.requests.len(),
            kind: SourceStructureRequestKind::ResultType,
        });
        fixture.arena = TypedArena::try_new(
            None,
            vec![
                TypedNode::new(
                    "source.term.numeral",
                    SourceAnchor::Range(range(fixture.source, 12, 13)),
                ),
                TypedNode::new(
                    "source.term.numeral",
                    SourceAnchor::Range(range(fixture.source, 30, 31)),
                ),
                TypedNode::new(
                    "source.term.structure.update",
                    SourceAnchor::Range(range(fixture.source, 10, 60)),
                ),
                TypedNode::new(
                    "source.term.structure.member.update-path-segment",
                    SourceAnchor::Range(range(fixture.source, 20, 25)),
                ),
                TypedNode::new(
                    "source.term.structure.member.update-path-segment",
                    SourceAnchor::Range(range(fixture.source, 26, 27)),
                ),
                TypedNode::new(
                    "source.term.structure.field-update",
                    SourceAnchor::Range(range(fixture.source, 18, 33)),
                ),
                TypedNode::new(
                    "source.term.structure.member.update-path-segment",
                    SourceAnchor::Range(range(fixture.source, 40, 45)),
                ),
                TypedNode::new(
                    "source.term.structure.member.update-path-segment",
                    SourceAnchor::Range(range(fixture.source, 46, 47)),
                ),
                TypedNode::new(
                    "source.term.structure.field-update",
                    SourceAnchor::Range(range(fixture.source, 38, 55)),
                ),
                TypedNode::new(
                    "source.term.numeral",
                    SourceAnchor::Range(range(fixture.source, 52, 53)),
                ),
            ],
        )
        .unwrap();
        fixture.primary = primary_handoff(
            fixture.source,
            &fixture.module,
            &fixture.bindings,
            &fixture.arena,
            &[(0, 12, 13, "1"), (1, 30, 31, "2"), (9, 52, 53, "3")],
        );
        fixture
    }

    fn application_target_fixture_variant(module_path: &str, wrapped: bool) -> Fixture {
        let source = source_id("d9");
        let module = module(module_path);
        let bindings = binding_env(source, &module);
        let mut nodes = vec![
            TypedNode::new(
                "source.term.numeral",
                SourceAnchor::Range(range(source, 25, 26)),
            ),
            TypedNode::new(
                "source.term.functor-application.symbolic",
                SourceAnchor::Range(range(source, 20, 30)),
            ),
            TypedNode::new(
                "source.term.functor-head.single",
                SourceAnchor::Range(range(source, 20, 21)),
            ),
            TypedNode::new(
                "source.term.structure.constructor",
                SourceAnchor::Range(range(source, 10, 50)),
            ),
            TypedNode::new(
                "source.term.structure.member.constructor-assignment",
                SourceAnchor::Range(range(source, 12, 19)),
            ),
        ];
        if wrapped {
            nodes.push(TypedNode::new(
                "source.term.functor-application.parenthesized",
                SourceAnchor::Range(range(source, 19, 31)),
            ));
        }
        let arena = TypedArena::try_new(None, nodes).expect("arena");
        let primary = primary_handoff(source, &module, &bindings, &arena, &[(0, 25, 26, "1")]);

        let mut indexes = SymbolEnvIndexes::default();
        let contribution = indexes.contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 9)),
        );
        let structure = symbol_id(&module, "Pair");
        let structure_origin = SemanticOrigin::new(
            source,
            module.clone(),
            SourceAnchor::Range(range(source, 1, 3)),
            vec![0],
        );
        indexes.symbols.insert(SymbolEntry::new(
            structure.clone(),
            SymbolKind::Structure,
            NamespacePath::new(module.path().as_str()),
            "Pair",
            structure_origin.clone(),
            contribution,
        ));
        indexes
            .contributions
            .add_symbol(contribution, structure.clone());
        let structure_definition = indexes.definitions.insert(DefinitionShell::new(
            structure.clone(),
            DefinitionKind::Structure,
            structure_origin,
            contribution,
        ));
        indexes
            .contributions
            .add_definition(contribution, structure_definition);
        let functor = symbol_id(&module, "f");
        let functor_origin = SemanticOrigin::new(
            source,
            module.clone(),
            SourceAnchor::Range(range(source, 4, 6)),
            vec![1],
        );
        indexes.symbols.insert(SymbolEntry::new(
            functor.clone(),
            SymbolKind::Functor,
            NamespacePath::new(module.path().as_str()),
            "f",
            functor_origin.clone(),
            contribution,
        ));
        indexes
            .contributions
            .add_symbol(contribution, functor.clone());
        let functor_definition = indexes.definitions.insert(DefinitionShell::new(
            functor.clone(),
            DefinitionKind::Functor,
            functor_origin,
            contribution,
        ));
        indexes
            .contributions
            .add_definition(contribution, functor_definition);
        let symbols = SymbolEnv::new(module.clone(), indexes);
        let applications = SourceFunctorApplicationProducer::build(
            SourceFunctorApplicationHandoffInput {
                source_id: source,
                module_id: module.clone(),
                applications: vec![SourceFunctorApplicationInput {
                    site: node(1),
                    source_range: range(source, 20, 30),
                    source_ordinal: 0,
                    context: BindingContextId::new(0),
                    recovery: SourceFunctorApplicationRecovery::Normal,
                    spelling: "f ( 1 )".to_owned(),
                    kind: SourceFunctorApplicationKind::Symbolic,
                    form: SourceFunctorApplicationForm::Functional,
                    head_ordinal: 0,
                    head: SourceFunctorHeadSite::Single {
                        site: node(2),
                        source_range: range(source, 20, 21),
                        spelling: "f".to_owned(),
                    },
                }],
                wrappers: if wrapped {
                    vec![SourceFunctorWrapperInput {
                        application: SourceFunctorApplicationId::new(0),
                        ordinal: 0,
                        site: node(5),
                        source_range: range(source, 19, 31),
                        context: BindingContextId::new(0),
                        spelling: "( f ( 1 ) )".to_owned(),
                        recovery: SourceFunctorApplicationRecovery::Normal,
                    }]
                } else {
                    Vec::new()
                },
                candidates: vec![SourceFunctorCandidateInput {
                    application: SourceFunctorApplicationId::new(0),
                    ordinal: 0,
                    symbol: functor,
                    contribution,
                }],
                arguments: vec![SourceFunctorArgumentInput {
                    application: SourceFunctorApplicationId::new(0),
                    ordinal: 0,
                    target: SourceFunctorArgumentTarget::Primary(SourcePrimaryTermId::new(0)),
                }],
                type_requests: vec![
                    SourceFunctorTypeRequestInput {
                        application: SourceFunctorApplicationId::new(0),
                        candidate: Some(SourceFunctorCandidateId::new(0)),
                        request_ordinal: 0,
                        kind: SourceFunctorTypeRequestKind::CandidateSignature,
                    },
                    SourceFunctorTypeRequestInput {
                        application: SourceFunctorApplicationId::new(0),
                        candidate: None,
                        request_ordinal: 1,
                        kind: SourceFunctorTypeRequestKind::ApplicationResultType,
                    },
                ],
            },
            &symbols,
            &bindings,
            &primary,
            &arena,
        )
        .expect("application handoff");
        Fixture {
            source,
            module: module.clone(),
            input: SourceStructureHandoffInput {
                source_id: source,
                module_id: module,
                terms: vec![SourceStructureTermInput {
                    site: node(3),
                    source_range: range(source, 10, 50),
                    source_ordinal: 0,
                    context: BindingContextId::new(0),
                    recovery: SourceStructureRecovery::Normal,
                    spelling: "Pair ( carrier : f ( 1 ) )".to_owned(),
                    kind: SourceStructureTermKind::Constructor,
                }],
                wrappers: Vec::new(),
                roots: vec![SourceStructureRootInput {
                    term: SourceStructureTermId::new(0),
                    symbol: structure,
                    contribution,
                }],
                members: vec![SourceStructureMemberInput {
                    term: SourceStructureTermId::new(0),
                    ordinal: 0,
                    site: node(4),
                    source_range: range(source, 12, 19),
                    spelling: "carrier".to_owned(),
                    role: SourceStructureMemberRole::ConstructorAssignment,
                    parent: None,
                }],
                field_updates: Vec::new(),
                edges: vec![SourceStructureEdgeInput {
                    term: SourceStructureTermId::new(0),
                    ordinal: 0,
                    role: SourceStructureEdgeRole::ConstructorValue,
                    member: Some(SourceStructureMemberId::new(0)),
                    target: SourceStructureTarget::Application(SourceFunctorApplicationId::new(0)),
                }],
                requests: vec![
                    SourceStructureRequestInput {
                        term: SourceStructureTermId::new(0),
                        member: None,
                        request_ordinal: 0,
                        kind: SourceStructureRequestKind::ConstructorSignature,
                    },
                    SourceStructureRequestInput {
                        term: SourceStructureTermId::new(0),
                        member: Some(SourceStructureMemberId::new(0)),
                        request_ordinal: 1,
                        kind: SourceStructureRequestKind::MemberIdentity,
                    },
                    SourceStructureRequestInput {
                        term: SourceStructureTermId::new(0),
                        member: Some(SourceStructureMemberId::new(0)),
                        request_ordinal: 2,
                        kind: SourceStructureRequestKind::InheritancePath,
                    },
                    SourceStructureRequestInput {
                        term: SourceStructureTermId::new(0),
                        member: None,
                        request_ordinal: 3,
                        kind: SourceStructureRequestKind::ResultType,
                    },
                ],
            },
            symbols,
            bindings,
            primary,
            applications: Some(applications),
            arena,
        }
    }

    fn application_target_fixture_for(module_path: &str) -> Fixture {
        application_target_fixture_variant(module_path, false)
    }

    fn application_target_fixture() -> Fixture {
        application_target_fixture_for("source.structure.application")
    }

    fn build(fixture: &Fixture) -> Result<SourceStructureHandoff, SourceStructureError> {
        SourceStructureProducer::build(
            fixture.input.clone(),
            &fixture.symbols,
            &fixture.bindings,
            &fixture.primary,
            fixture.applications.as_ref(),
            &fixture.arena,
        )
    }

    #[test]
    fn constructor_transaction_publishes_all_seven_tables_and_stable_debug() {
        let fixture = constructor_fixture();
        let handoff = build(&fixture).expect("constructor");
        assert_eq!(handoff.terms().len(), 1);
        assert!(handoff.wrappers().is_empty());
        assert_eq!(handoff.roots().len(), 1);
        assert_eq!(handoff.members().len(), 1);
        assert!(handoff.field_updates().is_empty());
        assert_eq!(handoff.edges().len(), 1);
        assert_eq!(handoff.requests().len(), 4);
        assert_eq!(
            handoff.primary_term_fingerprint(),
            fixture.primary.debug_text()
        );
        assert_eq!(handoff.application_fingerprint(), None);
        assert_eq!(handoff, build(&fixture).expect("deterministic replay"));
        assert_eq!(handoff.debug_text(), build(&fixture).unwrap().debug_text());
        assert!(handoff.debug_text().contains("kind=constructor"));
        handoff
            .validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.primary,
                None,
                &fixture.arena,
            )
            .expect("installation");
    }

    #[test]
    fn update_path_and_field_update_remain_distinct_owned_rows() {
        let fixture = functional_update_fixture();
        let handoff = build(&fixture).expect("functional update");
        assert_eq!(handoff.terms().len(), 1);
        assert_eq!(handoff.members().len(), 2);
        assert_eq!(handoff.field_updates().len(), 1);
        assert_eq!(
            handoff
                .members()
                .get(SourceStructureMemberId::new(1))
                .unwrap()
                .parent(),
            Some(SourceStructureMemberId::new(0))
        );
        assert_eq!(
            handoff
                .field_updates()
                .get(SourceFieldUpdateId::new(0))
                .unwrap()
                .final_member(),
            SourceStructureMemberId::new(1)
        );
    }

    #[test]
    fn repeated_update_paths_are_preserved_and_parent_chains_are_closed() {
        let fixture = repeated_update_fixture();
        let handoff = build(&fixture).expect("repeated update paths");
        assert_eq!(handoff.field_updates().len(), 2);
        assert_eq!(
            handoff
                .field_updates()
                .iter()
                .map(|(_, update)| update.spelling())
                .collect::<Vec<_>>(),
            ["start . x := 2", "start . x := 3"]
        );
        assert_eq!(
            handoff
                .members()
                .iter()
                .map(|(_, member)| (member.spelling(), member.parent()))
                .collect::<Vec<_>>(),
            [
                ("start", None),
                ("x", Some(SourceStructureMemberId::new(0))),
                ("start", None),
                ("x", Some(SourceStructureMemberId::new(2))),
            ]
        );

        let mut cycle = fixture.clone();
        cycle.input.members[0].parent = Some(SourceStructureMemberId::new(1));
        assert!(matches!(
            build(&cycle),
            Err(SourceStructureError::InvalidMember { .. })
                | Err(SourceStructureError::InvalidFieldUpdate { .. })
        ));

        let mut cross_field = fixture.clone();
        cross_field.input.members[2].parent = Some(SourceStructureMemberId::new(1));
        assert!(matches!(
            build(&cross_field),
            Err(SourceStructureError::InvalidFieldUpdate { .. })
                | Err(SourceStructureError::InvalidMember { .. })
        ));

        let mut cross_term = fixture.clone();
        cross_term.input.members[3].term = SourceStructureTermId::new(1);
        assert!(matches!(
            build(&cross_term),
            Err(SourceStructureError::ReorderedMember { .. })
                | Err(SourceStructureError::InvalidMember { .. })
        ));
    }

    #[test]
    fn field_update_endpoint_container_and_detachment_corruption_is_atomic() {
        let fixture = repeated_update_fixture();
        let mut cases = Vec::new();

        let mut wrong_first = fixture.clone();
        wrong_first.input.field_updates[0].first_member = SourceStructureMemberId::new(1);
        cases.push(wrong_first);

        let mut cross_update_endpoint = fixture.clone();
        cross_update_endpoint.input.field_updates[0].final_member = SourceStructureMemberId::new(2);
        cases.push(cross_update_endpoint);

        let mut reversed_endpoint = fixture.clone();
        reversed_endpoint.input.field_updates[1].first_member = SourceStructureMemberId::new(3);
        reversed_endpoint.input.field_updates[1].final_member = SourceStructureMemberId::new(2);
        cases.push(reversed_endpoint);

        let mut detached = fixture.clone();
        detached.input.field_updates.pop();
        cases.push(detached);

        let mut wrong_term = fixture.clone();
        wrong_term.input.field_updates[1].term = SourceStructureTermId::new(1);
        cases.push(wrong_term);

        let mut duplicate_owner = fixture.clone();
        duplicate_owner.input.field_updates[1].first_member = SourceStructureMemberId::new(0);
        duplicate_owner.input.field_updates[1].final_member = SourceStructureMemberId::new(1);
        cases.push(duplicate_owner);

        let mut wrong_container = fixture.clone();
        wrong_container.input.field_updates[1].site = node(7);
        cases.push(wrong_container);

        for corrupt in cases {
            assert!(build(&corrupt).is_err());
            assert!(build(&fixture).is_ok(), "failed transaction leaked state");
        }
    }

    #[test]
    fn source_identity_dense_ordinals_and_table_associations_are_atomic() {
        let fixture = constructor_fixture();
        let mut cases = Vec::new();
        let mut corrupt = fixture.clone();
        corrupt.input.source_id = other_source_id("d6");
        cases.push(corrupt);
        let mut corrupt = fixture.clone();
        corrupt.input.terms[0].source_ordinal = 1;
        cases.push(corrupt);
        let mut corrupt = fixture.clone();
        corrupt.input.members[0].ordinal = 1;
        cases.push(corrupt);
        let mut corrupt = fixture.clone();
        corrupt.input.roots.clear();
        cases.push(corrupt);
        let mut corrupt = fixture.clone();
        corrupt.input.edges[0].member = None;
        cases.push(corrupt);
        let mut corrupt = fixture.clone();
        corrupt.input.requests.swap(1, 2);
        cases.push(corrupt);
        for corrupt in cases {
            assert!(build(&corrupt).is_err());
            assert!(build(&fixture).is_ok(), "failure published no state");
        }
    }

    #[test]
    fn edge_and_request_cardinality_order_and_partial_rows_are_rejected() {
        let constructor = constructor_fixture();
        let mut cases = Vec::new();

        let mut missing_edge = constructor.clone();
        missing_edge.input.edges.clear();
        cases.push(missing_edge);

        let mut extra_edge = constructor.clone();
        let mut duplicate = extra_edge.input.edges[0].clone();
        duplicate.ordinal = 1;
        extra_edge.input.edges.push(duplicate);
        cases.push(extra_edge);

        let mut partial_edge = constructor.clone();
        partial_edge.input.edges[0].member = None;
        cases.push(partial_edge);

        let mut missing_request = constructor.clone();
        missing_request.input.requests.pop();
        cases.push(missing_request);

        let mut extra_request = constructor.clone();
        extra_request
            .input
            .requests
            .push(SourceStructureRequestInput {
                term: SourceStructureTermId::new(0),
                member: None,
                request_ordinal: 4,
                kind: SourceStructureRequestKind::ResultType,
            });
        cases.push(extra_request);

        let mut reordered_request = constructor.clone();
        reordered_request.input.requests.swap(1, 2);
        cases.push(reordered_request);

        let mut wrong_request_member = constructor.clone();
        wrong_request_member.input.requests[3].member = Some(SourceStructureMemberId::new(0));
        cases.push(wrong_request_member);

        for corrupt in cases {
            assert!(build(&corrupt).is_err());
        }

        let update = repeated_update_fixture();
        let mut reordered_edges = update.clone();
        reordered_edges.input.edges.swap(1, 2);
        assert!(matches!(
            build(&reordered_edges),
            Err(SourceStructureError::ReorderedEdge { .. })
                | Err(SourceStructureError::InvalidEdge { .. })
        ));
        let mut partial_update = update.clone();
        partial_update.input.edges.pop();
        assert!(build(&partial_update).is_err());
    }

    #[test]
    fn selector_accepts_zero_or_more_arguments_but_requires_one_leading_base() {
        assert!(build(&selector_fixture()).is_ok(), "zero-argument selector");

        let mut with_argument = selector_fixture();
        with_argument.arena = TypedArena::try_new(
            None,
            vec![
                TypedNode::new(
                    "source.term.numeral",
                    SourceAnchor::Range(range(with_argument.source, 11, 12)),
                ),
                TypedNode::new(
                    "source.term.structure.selector",
                    SourceAnchor::Range(range(with_argument.source, 10, 30)),
                ),
                TypedNode::new(
                    "source.term.structure.member.selector",
                    SourceAnchor::Range(range(with_argument.source, 13, 20)),
                ),
                TypedNode::new(
                    "source.term.numeral",
                    SourceAnchor::Range(range(with_argument.source, 24, 25)),
                ),
            ],
        )
        .unwrap();
        with_argument.primary = primary_handoff(
            with_argument.source,
            &with_argument.module,
            &with_argument.bindings,
            &with_argument.arena,
            &[(0, 11, 12, "1"), (3, 24, 25, "2")],
        );
        with_argument.input.edges.push(SourceStructureEdgeInput {
            term: SourceStructureTermId::new(0),
            ordinal: 1,
            role: SourceStructureEdgeRole::SelectorArgument,
            member: None,
            target: SourceStructureTarget::Primary(SourcePrimaryTermId::new(1)),
        });
        assert!(build(&with_argument).is_ok(), "one selector argument");

        let mut missing_base = with_argument.clone();
        missing_base.input.edges.remove(0);
        missing_base.input.edges[0].ordinal = 0;
        assert!(build(&missing_base).is_err());

        let mut second_base = with_argument.clone();
        second_base.input.edges[1].role = SourceStructureEdgeRole::SelectorBase;
        assert!(matches!(
            build(&second_base),
            Err(SourceStructureError::InvalidEdge { .. })
        ));

        let mut argument_with_member = with_argument;
        argument_with_member.input.edges[1].member = Some(SourceStructureMemberId::new(0));
        assert!(matches!(
            build(&argument_with_member),
            Err(SourceStructureError::InvalidEdge { .. })
        ));
    }

    #[test]
    fn child_targets_and_field_update_spelling_match_exact_written_partitions() {
        let mut constructor = constructor_fixture();
        constructor.arena = TypedArena::try_new(
            None,
            vec![
                TypedNode::new(
                    "source.term.numeral",
                    SourceAnchor::Range(range(constructor.source, 22, 23)),
                ),
                TypedNode::new(
                    "source.term.structure.constructor",
                    SourceAnchor::Range(range(constructor.source, 10, 30)),
                ),
                TypedNode::new(
                    "source.term.structure.member.constructor-assignment",
                    SourceAnchor::Range(range(constructor.source, 13, 20)),
                ),
                TypedNode::new(
                    "source.term.numeral",
                    SourceAnchor::Range(range(constructor.source, 11, 12)),
                ),
            ],
        )
        .unwrap();
        constructor.primary = primary_handoff(
            constructor.source,
            &constructor.module,
            &constructor.bindings,
            &constructor.arena,
            &[(3, 11, 12, "9")],
        );
        assert!(matches!(
            build(&constructor),
            Err(SourceStructureError::InvalidEdge { .. })
        ));

        let mut selector = selector_fixture();
        selector.arena = TypedArena::try_new(
            None,
            vec![
                TypedNode::new(
                    "source.term.numeral",
                    SourceAnchor::Range(range(selector.source, 11, 12)),
                ),
                TypedNode::new(
                    "source.term.structure.selector",
                    SourceAnchor::Range(range(selector.source, 10, 30)),
                ),
                TypedNode::new(
                    "source.term.structure.member.selector",
                    SourceAnchor::Range(range(selector.source, 13, 20)),
                ),
                TypedNode::new(
                    "source.term.numeral",
                    SourceAnchor::Range(range(selector.source, 24, 25)),
                ),
            ],
        )
        .unwrap();
        selector.primary = primary_handoff(
            selector.source,
            &selector.module,
            &selector.bindings,
            &selector.arena,
            &[(0, 11, 12, "1"), (3, 24, 25, "2")],
        );
        selector.input.edges[0].target =
            SourceStructureTarget::Primary(SourcePrimaryTermId::new(1));
        assert!(matches!(
            build(&selector),
            Err(SourceStructureError::InvalidTerm { .. })
                | Err(SourceStructureError::InvalidEdge { .. })
        ));

        let mut detached_replacement = functional_update_fixture();
        detached_replacement.input.field_updates[0].source_range =
            range(detached_replacement.source, 18, 28);
        detached_replacement.arena = TypedArena::try_new(
            None,
            vec![
                TypedNode::new(
                    "source.term.numeral",
                    SourceAnchor::Range(range(detached_replacement.source, 12, 13)),
                ),
                TypedNode::new(
                    "source.term.numeral",
                    SourceAnchor::Range(range(detached_replacement.source, 30, 31)),
                ),
                TypedNode::new(
                    "source.term.structure.update",
                    SourceAnchor::Range(range(detached_replacement.source, 10, 35)),
                ),
                TypedNode::new(
                    "source.term.structure.member.update-path-segment",
                    SourceAnchor::Range(range(detached_replacement.source, 20, 25)),
                ),
                TypedNode::new(
                    "source.term.structure.member.update-path-segment",
                    SourceAnchor::Range(range(detached_replacement.source, 26, 27)),
                ),
                TypedNode::new(
                    "source.term.structure.field-update",
                    SourceAnchor::Range(range(detached_replacement.source, 18, 28)),
                ),
            ],
        )
        .unwrap();
        detached_replacement.primary = primary_handoff(
            detached_replacement.source,
            &detached_replacement.module,
            &detached_replacement.bindings,
            &detached_replacement.arena,
            &[(0, 12, 13, "1"), (1, 30, 31, "2")],
        );
        assert!(matches!(
            build(&detached_replacement),
            Err(SourceStructureError::InvalidEdge { .. })
        ));

        let mut spelling_prefix = functional_update_fixture();
        spelling_prefix.input.field_updates[0].spelling = "starter := 2".to_owned();
        assert!(matches!(
            build(&spelling_prefix),
            Err(SourceStructureError::InvalidFieldUpdate { .. })
        ));
    }

    #[test]
    fn task253_argument_and_reverse_structure_ownership_fail_in_both_install_orders() {
        let mut multiply_owned = application_target_fixture();
        multiply_owned.input.edges[0].target =
            SourceStructureTarget::Primary(SourcePrimaryTermId::new(0));
        assert!(matches!(
            build(&multiply_owned),
            Err(SourceStructureError::InvalidTerm { .. })
                | Err(SourceStructureError::InvalidEdge { .. })
                | Err(SourceStructureError::ApplicationDependencyMismatch)
        ));

        let mut reverse = application_target_fixture();
        reverse.arena = TypedArena::try_new(
            None,
            vec![
                TypedNode::new(
                    "source.term.numeral",
                    SourceAnchor::Range(range(reverse.source, 25, 26)),
                ),
                TypedNode::new(
                    "source.term.functor-application.symbolic",
                    SourceAnchor::Range(range(reverse.source, 20, 30)),
                ),
                TypedNode::new(
                    "source.term.functor-head.single",
                    SourceAnchor::Range(range(reverse.source, 20, 21)),
                ),
                TypedNode::new(
                    "source.term.structure.constructor",
                    SourceAnchor::Range(range(reverse.source, 22, 29)),
                ),
                TypedNode::new(
                    "source.term.structure.member.constructor-assignment",
                    SourceAnchor::Range(range(reverse.source, 23, 24)),
                ),
            ],
        )
        .unwrap();
        reverse.input.terms[0].source_range = range(reverse.source, 22, 29);
        reverse.input.terms[0].spelling = "Pair ( carrier : 1 )".to_owned();
        reverse.input.members[0].source_range = range(reverse.source, 23, 24);
        reverse.input.edges[0].target = SourceStructureTarget::Primary(SourcePrimaryTermId::new(0));
        reverse.primary = primary_handoff(
            reverse.source,
            &reverse.module,
            &reverse.bindings,
            &reverse.arena,
            &[(0, 25, 26, "1")],
        );
        let application = reverse
            .applications
            .clone()
            .expect("application target fixture carries Task 253");
        reverse.applications = None;
        let reverse_result = SourceStructureProducer::build(
            reverse.input.clone(),
            &reverse.symbols,
            &reverse.bindings,
            &reverse.primary,
            Some(&application),
            &reverse.arena,
        );
        assert!(
            matches!(
                reverse_result,
                Err(SourceStructureError::ApplicationDependencyMismatch)
                    | Err(SourceStructureError::InvalidEdge { .. })
                    | Err(SourceStructureError::InvalidTerm { .. })
            ),
            "{reverse_result:?}"
        );

        let structure = build(&reverse).expect("structure without Task 253");
        let structure_first = TypedAst::try_new(empty_typed_parts(&reverse))
            .unwrap()
            .with_source_term(reverse.primary.clone())
            .unwrap()
            .with_source_structure(structure.clone())
            .unwrap();
        assert_eq!(
            structure_first.with_source_application(application.clone()),
            Err(TypedAstError::InvalidSourceApplication)
        );

        let application_first = TypedAst::try_new(empty_typed_parts(&reverse))
            .unwrap()
            .with_source_term(reverse.primary.clone())
            .unwrap()
            .with_source_application(application)
            .unwrap();
        assert_eq!(
            application_first.with_source_structure(structure),
            Err(TypedAstError::InvalidSourceStructure)
        );
    }

    #[test]
    fn all_five_structure_arena_keys_reject_cross_role_substitution() {
        assert!(build(&selector_fixture()).is_ok());
        let mut constructor = constructor_fixture();
        constructor.input.members[0].role = SourceStructureMemberRole::Selector;
        assert!(matches!(
            build(&constructor),
            Err(SourceStructureError::InvalidMember { .. })
        ));

        let mut update = functional_update_fixture();
        update.input.members[0].role = SourceStructureMemberRole::ConstructorAssignment;
        assert!(build(&update).is_err());
        let mut update = functional_update_fixture();
        update.input.field_updates[0].site = node(4);
        assert!(build(&update).is_err());

        let fixture = constructor_fixture();
        let mut wrapped = fixture.clone();
        wrapped.arena = TypedArena::try_new(
            None,
            vec![
                TypedNode::new(
                    "source.term.numeral",
                    SourceAnchor::Range(range(fixture.source, 22, 23)),
                ),
                TypedNode::new(
                    "source.term.structure.constructor",
                    SourceAnchor::Range(range(fixture.source, 10, 30)),
                ),
                TypedNode::new(
                    "source.term.structure.member.constructor-assignment",
                    SourceAnchor::Range(range(fixture.source, 13, 20)),
                ),
                TypedNode::new(
                    "source.term.structure.parenthesized",
                    SourceAnchor::Range(range(fixture.source, 9, 31)),
                ),
            ],
        )
        .unwrap();
        wrapped.input.wrappers.push(SourceStructureWrapperInput {
            term: SourceStructureTermId::new(0),
            ordinal: 0,
            site: node(3),
            source_range: range(fixture.source, 9, 31),
            context: BindingContextId::new(0),
            spelling: "( Pair ( carrier : 1 ) )".to_owned(),
            recovery: SourceStructureRecovery::Normal,
        });
        wrapped.primary = primary_handoff(
            fixture.source,
            &fixture.module,
            &fixture.bindings,
            &wrapped.arena,
            &[(0, 22, 23, "1")],
        );
        assert!(build(&wrapped).is_ok());
        wrapped.input.wrappers[0].site = node(2);
        assert!(build(&wrapped).is_err());
    }

    #[test]
    fn nested_wrapper_order_nesting_and_degraded_recovery_are_exact() {
        let mut fixture = constructor_fixture();
        fixture.input.terms[0].recovery = SourceStructureRecovery::Degraded;
        fixture.input.wrappers = vec![
            SourceStructureWrapperInput {
                term: SourceStructureTermId::new(0),
                ordinal: 0,
                site: node(3),
                source_range: range(fixture.source, 8, 32),
                context: BindingContextId::new(0),
                spelling: "( ( Pair ( carrier : 1 ) ) )".to_owned(),
                recovery: SourceStructureRecovery::Degraded,
            },
            SourceStructureWrapperInput {
                term: SourceStructureTermId::new(0),
                ordinal: 1,
                site: node(4),
                source_range: range(fixture.source, 9, 31),
                context: BindingContextId::new(0),
                spelling: "( Pair ( carrier : 1 ) )".to_owned(),
                recovery: SourceStructureRecovery::Degraded,
            },
        ];
        fixture.arena = TypedArena::try_new(
            None,
            vec![
                TypedNode::new(
                    "source.term.numeral",
                    SourceAnchor::Range(range(fixture.source, 22, 23)),
                ),
                TypedNode::new(
                    "source.term.structure.constructor",
                    SourceAnchor::Range(range(fixture.source, 10, 30)),
                )
                .with_recovery(NodeRecoveryState::Degraded),
                TypedNode::new(
                    "source.term.structure.member.constructor-assignment",
                    SourceAnchor::Range(range(fixture.source, 13, 20)),
                )
                .with_recovery(NodeRecoveryState::Degraded),
                TypedNode::new(
                    "source.term.structure.parenthesized",
                    SourceAnchor::Range(range(fixture.source, 8, 32)),
                )
                .with_recovery(NodeRecoveryState::Degraded),
                TypedNode::new(
                    "source.term.structure.parenthesized",
                    SourceAnchor::Range(range(fixture.source, 9, 31)),
                )
                .with_recovery(NodeRecoveryState::Recovered),
            ],
        )
        .unwrap();
        fixture.primary = primary_handoff(
            fixture.source,
            &fixture.module,
            &fixture.bindings,
            &fixture.arena,
            &[(0, 22, 23, "1")],
        );
        assert!(build(&fixture).is_ok(), "degraded nested wrappers");

        let mut reordered = fixture.clone();
        reordered.input.wrappers.swap(0, 1);
        assert!(matches!(
            build(&reordered),
            Err(SourceStructureError::ReorderedWrapper { .. })
        ));

        let mut non_nested = fixture.clone();
        non_nested.input.wrappers[1].source_range = range(fixture.source, 10, 30);
        assert!(matches!(
            build(&non_nested),
            Err(SourceStructureError::InvalidWrapper { .. })
        ));

        let mut wrong_spelling = fixture.clone();
        wrong_spelling.input.wrappers[0].spelling = "( Pair ( carrier : 1 ) )".to_owned();
        assert!(matches!(
            build(&wrong_spelling),
            Err(SourceStructureError::InvalidWrapper { .. })
        ));

        let mut wrong_recovery = fixture;
        wrong_recovery.input.wrappers[1].recovery = SourceStructureRecovery::Normal;
        assert!(matches!(
            build(&wrong_recovery),
            Err(SourceStructureError::InvalidWrapper { .. })
        ));
    }

    #[test]
    fn repeated_labels_and_paths_are_not_deduplicated() {
        let mut fixture = constructor_fixture();
        fixture.input.members.push(SourceStructureMemberInput {
            term: SourceStructureTermId::new(0),
            ordinal: 1,
            site: node(3),
            source_range: range(fixture.source, 24, 27),
            spelling: "carrier".to_owned(),
            role: SourceStructureMemberRole::ConstructorAssignment,
            parent: None,
        });
        fixture.input.terms[0].source_range = range(fixture.source, 10, 40);
        fixture.input.terms[0].spelling = "Pair ( carrier : 1 , carrier : 2 )".to_owned();
        fixture.input.edges.push(SourceStructureEdgeInput {
            term: SourceStructureTermId::new(0),
            ordinal: 1,
            role: SourceStructureEdgeRole::ConstructorValue,
            member: Some(SourceStructureMemberId::new(1)),
            target: SourceStructureTarget::Primary(SourcePrimaryTermId::new(1)),
        });
        fixture.input.requests.splice(
            3..3,
            [
                SourceStructureRequestInput {
                    term: SourceStructureTermId::new(0),
                    member: Some(SourceStructureMemberId::new(1)),
                    request_ordinal: 3,
                    kind: SourceStructureRequestKind::MemberIdentity,
                },
                SourceStructureRequestInput {
                    term: SourceStructureTermId::new(0),
                    member: Some(SourceStructureMemberId::new(1)),
                    request_ordinal: 4,
                    kind: SourceStructureRequestKind::InheritancePath,
                },
            ],
        );
        fixture.input.requests[5].request_ordinal = 5;
        fixture.arena = TypedArena::try_new(
            None,
            vec![
                TypedNode::new(
                    "source.term.numeral",
                    SourceAnchor::Range(range(fixture.source, 22, 23)),
                ),
                TypedNode::new(
                    "source.term.structure.constructor",
                    SourceAnchor::Range(range(fixture.source, 10, 40)),
                ),
                TypedNode::new(
                    "source.term.structure.member.constructor-assignment",
                    SourceAnchor::Range(range(fixture.source, 13, 20)),
                ),
                TypedNode::new(
                    "source.term.structure.member.constructor-assignment",
                    SourceAnchor::Range(range(fixture.source, 24, 27)),
                ),
                TypedNode::new(
                    "source.term.numeral",
                    SourceAnchor::Range(range(fixture.source, 30, 31)),
                ),
            ],
        )
        .unwrap();
        fixture.primary = primary_handoff(
            fixture.source,
            &fixture.module,
            &fixture.bindings,
            &fixture.arena,
            &[(0, 22, 23, "1"), (4, 30, 31, "2")],
        );
        let handoff = build(&fixture).expect("repeated source labels");
        assert_eq!(handoff.members().len(), 2);
        assert_eq!(
            handoff
                .members()
                .iter()
                .map(|(_, row)| row.spelling())
                .collect::<Vec<_>>(),
            ["carrier", "carrier"]
        );
    }

    #[test]
    fn local_root_provenance_and_unresolved_shell_policy_are_exact() {
        for shell in [
            None,
            Some(SignatureShell::Pending),
            Some(SignatureShell::Opaque {
                schema: "structure-v1".to_owned(),
                payload: "pair".to_owned(),
            }),
        ] {
            assert!(
                build(&constructor_fixture_with(RootOptions {
                    signature: shell.clone(),
                    definition_signature: shell,
                    ..RootOptions::default()
                }))
                .is_ok()
            );
        }

        let malformed = SignatureShell::Malformed {
            class: "recovered-shell".to_owned(),
        };
        for corrupt in [
            constructor_fixture_with(RootOptions {
                kind: SymbolKind::Functor,
                ..RootOptions::default()
            }),
            constructor_fixture_with(RootOptions {
                signature: Some(malformed.clone()),
                definition_signature: Some(malformed),
                ..RootOptions::default()
            }),
            constructor_fixture_with(RootOptions {
                recovered: true,
                ..RootOptions::default()
            }),
            constructor_fixture_with(RootOptions {
                include_definition: false,
                ..RootOptions::default()
            }),
            constructor_fixture_with(RootOptions {
                definition_kind: DefinitionKind::Functor,
                ..RootOptions::default()
            }),
            constructor_fixture_with(RootOptions {
                conflict: Some(DeclarationConflictClass::DuplicateSpelling),
                ..RootOptions::default()
            }),
            constructor_fixture_with(RootOptions {
                origin_after_use: true,
                ..RootOptions::default()
            }),
            constructor_fixture_with(RootOptions {
                namespace_drift: true,
                ..RootOptions::default()
            }),
            constructor_fixture_with(RootOptions {
                imported_contribution: true,
                ..RootOptions::default()
            }),
            constructor_fixture_with(RootOptions {
                contribution_module_drift: true,
                ..RootOptions::default()
            }),
            constructor_fixture_with(RootOptions {
                contribution_source_drift: true,
                ..RootOptions::default()
            }),
            constructor_fixture_with(RootOptions {
                omit_symbol_effect: true,
                ..RootOptions::default()
            }),
            constructor_fixture_with(RootOptions {
                symbol_contribution_drift: true,
                ..RootOptions::default()
            }),
            constructor_fixture_with(RootOptions {
                definition_contribution_drift: true,
                ..RootOptions::default()
            }),
            constructor_fixture_with(RootOptions {
                definition_origin_drift: true,
                ..RootOptions::default()
            }),
            constructor_fixture_with(RootOptions {
                definition_visibility_drift: true,
                ..RootOptions::default()
            }),
            constructor_fixture_with(RootOptions {
                omit_definition_effect: true,
                ..RootOptions::default()
            }),
            constructor_fixture_with(RootOptions {
                signature: Some(SignatureShell::Pending),
                definition_signature: None,
                ..RootOptions::default()
            }),
        ] {
            assert!(matches!(
                build(&corrupt),
                Err(SourceStructureError::InvalidRoot { .. })
            ));
        }
    }

    #[test]
    fn imported_root_visibility_export_origin_and_import_policy_is_exact() {
        let source = source_id("d7");
        let current = module("source.structure.import");
        let dependency = module("dependency.structure");
        let mut indexes = SymbolEnvIndexes::default();
        let contribution = indexes.contributions.insert(
            current.clone(),
            ContributionKind::ImportedSource { source_id: source },
            SourceAnchor::Range(range(source, 1, 5)),
        );
        let symbol = symbol_id(&dependency, "ImportedPair");
        let entry = |visibility, export_status, origin_module: ModuleId| {
            SymbolEntry::new(
                symbol.clone(),
                SymbolKind::Structure,
                NamespacePath::new(current.path().as_str()),
                "ImportedPair",
                SemanticOrigin::new(
                    source,
                    origin_module,
                    SourceAnchor::Range(range(source, 1, 5)),
                    vec![0],
                ),
                contribution,
            )
            .with_visibility(visibility)
            .with_export_status(export_status)
        };
        for export in [ExportStatus::Exported, ExportStatus::ReExported] {
            assert!(valid_imported_root_provenance(
                &entry(Visibility::Public, export, dependency.clone()),
                &symbol,
                source,
                range(source, 10, 30),
                range(source, 1, 5),
                true,
            ));
        }
        for (visibility, export, origin, authenticated) in [
            (
                Visibility::Private,
                ExportStatus::Exported,
                dependency.clone(),
                true,
            ),
            (
                Visibility::Public,
                ExportStatus::LocalOnly,
                dependency.clone(),
                true,
            ),
            (
                Visibility::Public,
                ExportStatus::Exported,
                current.clone(),
                true,
            ),
            (
                Visibility::Public,
                ExportStatus::Exported,
                dependency,
                false,
            ),
        ] {
            assert!(!valid_imported_root_provenance(
                &entry(visibility, export, origin),
                &symbol,
                source,
                range(source, 10, 30),
                range(source, 1, 5),
                authenticated,
            ));
        }
    }

    #[test]
    fn structure_child_preorder_context_and_single_parent_are_enforced() {
        let mut fixture = constructor_fixture();
        fixture.input.terms[0].source_range = range(fixture.source, 10, 50);
        fixture.input.terms[0].spelling = "Pair ( carrier : Pair ( ) )".to_owned();
        fixture.input.terms.push(SourceStructureTermInput {
            site: node(3),
            source_range: range(fixture.source, 22, 40),
            source_ordinal: 1,
            context: BindingContextId::new(0),
            recovery: SourceStructureRecovery::Normal,
            spelling: "Pair ( )".to_owned(),
            kind: SourceStructureTermKind::Constructor,
        });
        fixture.input.roots.push(SourceStructureRootInput {
            term: SourceStructureTermId::new(1),
            symbol: fixture.input.roots[0].symbol.clone(),
            contribution: fixture.input.roots[0].contribution,
        });
        fixture.input.edges[0].target =
            SourceStructureTarget::Structure(SourceStructureTermId::new(1));
        fixture.input.requests.push(SourceStructureRequestInput {
            term: SourceStructureTermId::new(1),
            member: None,
            request_ordinal: 0,
            kind: SourceStructureRequestKind::ConstructorSignature,
        });
        fixture.input.requests.push(SourceStructureRequestInput {
            term: SourceStructureTermId::new(1),
            member: None,
            request_ordinal: 1,
            kind: SourceStructureRequestKind::ResultType,
        });
        fixture.arena = TypedArena::try_new(
            None,
            vec![
                TypedNode::new(
                    "source.term.numeral",
                    SourceAnchor::Range(range(fixture.source, 22, 23)),
                ),
                TypedNode::new(
                    "source.term.structure.constructor",
                    SourceAnchor::Range(range(fixture.source, 10, 50)),
                ),
                TypedNode::new(
                    "source.term.structure.member.constructor-assignment",
                    SourceAnchor::Range(range(fixture.source, 13, 20)),
                ),
                TypedNode::new(
                    "source.term.structure.constructor",
                    SourceAnchor::Range(range(fixture.source, 22, 40)),
                ),
            ],
        )
        .unwrap();
        fixture.primary = primary_handoff(
            fixture.source,
            &fixture.module,
            &fixture.bindings,
            &fixture.arena,
            &[],
        );
        assert!(build(&fixture).is_ok());

        let mut cross_context = fixture.clone();
        cross_context.bindings =
            binding_env_with_two_contexts(cross_context.source, &cross_context.module);
        cross_context.input.terms[1].context = BindingContextId::new(1);
        assert!(matches!(
            build(&cross_context),
            Err(SourceStructureError::InvalidEdge { .. })
                | Err(SourceStructureError::InvalidTerm { .. })
        ));

        let mut back_edge = fixture.clone();
        back_edge.input.edges.push(SourceStructureEdgeInput {
            term: SourceStructureTermId::new(1),
            ordinal: 0,
            role: SourceStructureEdgeRole::ConstructorValue,
            member: None,
            target: SourceStructureTarget::Structure(SourceStructureTermId::new(0)),
        });
        assert!(matches!(
            build(&back_edge),
            Err(SourceStructureError::InvalidEdge { .. })
                | Err(SourceStructureError::ReorderedEdge { .. })
                | Err(SourceStructureError::InvalidTerm { .. })
        ));

        let mut reordered = fixture.clone();
        reordered.input.terms.swap(0, 1);
        assert!(matches!(
            build(&reordered),
            Err(SourceStructureError::InvalidTerm { .. })
                | Err(SourceStructureError::ReorderedTerm { .. })
                | Err(SourceStructureError::InvalidRoot { .. })
        ));

        fixture.input.edges.push(SourceStructureEdgeInput {
            term: SourceStructureTermId::new(0),
            ordinal: 1,
            role: SourceStructureEdgeRole::ConstructorValue,
            member: Some(SourceStructureMemberId::new(0)),
            target: SourceStructureTarget::Structure(SourceStructureTermId::new(1)),
        });
        assert!(matches!(
            build(&fixture),
            Err(SourceStructureError::MultipleParents { .. })
                | Err(SourceStructureError::InvalidTerm { .. })
        ));
    }

    #[test]
    fn task252_and_task253_targets_must_share_the_owner_context() {
        let mut primary = constructor_fixture();
        primary.bindings = binding_env_with_two_contexts(primary.source, &primary.module);
        primary.input.terms[0].context = BindingContextId::new(1);
        assert!(matches!(
            build(&primary),
            Err(SourceStructureError::DuplicateTarget { .. })
                | Err(SourceStructureError::InvalidEdge { .. })
                | Err(SourceStructureError::InvalidTerm { .. })
        ));

        let mut application = application_target_fixture();
        application.bindings =
            binding_env_with_two_contexts(application.source, &application.module);
        application.input.terms[0].context = BindingContextId::new(1);
        assert!(matches!(
            build(&application),
            Err(SourceStructureError::DuplicateTarget { .. })
                | Err(SourceStructureError::InvalidEdge { .. })
                | Err(SourceStructureError::InvalidTerm { .. })
        ));
    }

    #[test]
    fn dependency_fingerprint_none_matrix_preserves_unrelated_application() {
        let mut fixture = constructor_fixture();
        fixture.arena = TypedArena::try_new(
            None,
            vec![
                TypedNode::new(
                    "source.term.numeral",
                    SourceAnchor::Range(range(fixture.source, 22, 23)),
                ),
                TypedNode::new(
                    "source.term.structure.constructor",
                    SourceAnchor::Range(range(fixture.source, 10, 30)),
                ),
                TypedNode::new(
                    "source.term.structure.member.constructor-assignment",
                    SourceAnchor::Range(range(fixture.source, 13, 20)),
                ),
                TypedNode::new(
                    "source.term.functor-application.inline",
                    SourceAnchor::Range(range(fixture.source, 40, 50)),
                ),
                TypedNode::new(
                    "source.term.functor-head.single",
                    SourceAnchor::Range(range(fixture.source, 40, 41)),
                ),
            ],
        )
        .unwrap();
        fixture.primary = primary_handoff(
            fixture.source,
            &fixture.module,
            &fixture.bindings,
            &fixture.arena,
            &[(0, 22, 23, "1")],
        );
        let handoff = build(&fixture).unwrap();
        assert_eq!(handoff.application_fingerprint(), None);
        handoff
            .validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.primary,
                None,
                &fixture.arena,
            )
            .expect("no application installed");
        let unrelated = SourceFunctorApplicationProducer::build(
            SourceFunctorApplicationHandoffInput {
                source_id: fixture.source,
                module_id: fixture.module.clone(),
                applications: vec![SourceFunctorApplicationInput {
                    site: node(3),
                    source_range: range(fixture.source, 40, 50),
                    source_ordinal: 0,
                    context: BindingContextId::new(0),
                    recovery: SourceFunctorApplicationRecovery::Normal,
                    spelling: "f ( )".to_owned(),
                    kind: SourceFunctorApplicationKind::Inline,
                    form: SourceFunctorApplicationForm::Functional,
                    head_ordinal: 0,
                    head: SourceFunctorHeadSite::Single {
                        site: node(4),
                        source_range: range(fixture.source, 40, 41),
                        spelling: "f".to_owned(),
                    },
                }],
                wrappers: Vec::new(),
                candidates: Vec::new(),
                arguments: Vec::new(),
                type_requests: Vec::new(),
            },
            &fixture.symbols,
            &fixture.bindings,
            &fixture.primary,
            &fixture.arena,
        )
        .unwrap();
        handoff
            .validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.primary,
                Some(&unrelated),
                &fixture.arena,
            )
            .expect("unrelated same-module dependency remains compatible");

        let typed = TypedAst::try_new(empty_typed_parts(&fixture))
            .unwrap()
            .with_source_term(fixture.primary.clone())
            .unwrap()
            .with_source_structure(handoff)
            .unwrap()
            .with_source_application(unrelated)
            .expect("Task 253 may be installed later when ownership stays disjoint");
        assert!(typed.source_application().is_some());
        assert!(typed.source_structure().is_some());
    }

    #[test]
    fn application_root_target_sets_some_fingerprint_and_requires_exact_dependency() {
        let fixture = application_target_fixture();
        let handoff = build(&fixture).expect("root application target");
        assert_eq!(
            handoff.application_fingerprint(),
            Some(fixture.applications.as_ref().unwrap().debug_text().as_str())
        );
        handoff
            .validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.primary,
                fixture.applications.as_ref(),
                &fixture.arena,
            )
            .expect("matching application dependency");
        assert!(matches!(
            handoff.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.primary,
                None,
                &fixture.arena,
            ),
            Err(SourceStructureError::ApplicationDependencyMismatch)
        ));

        let mut missing = fixture.clone();
        missing.applications = None;
        assert!(matches!(
            build(&missing),
            Err(SourceStructureError::ApplicationDependencyMismatch)
        ));
        let other = application_target_fixture_for("source.structure.application.other");
        assert!(matches!(
            handoff.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.primary,
                other.applications.as_ref(),
                &fixture.arena,
            ),
            Err(SourceStructureError::ApplicationDependencyMismatch)
        ));
    }

    #[test]
    fn task253_application_already_owned_by_task253_is_not_a_structure_target() {
        let source = source_id("da");
        let module = module("source.structure.nested.application");
        let bindings = binding_env(source, &module);
        let arena = TypedArena::try_new(
            None,
            vec![
                TypedNode::new(
                    "source.term.functor-application.inline",
                    SourceAnchor::Range(range(source, 20, 40)),
                ),
                TypedNode::new(
                    "source.term.functor-head.single",
                    SourceAnchor::Range(range(source, 20, 21)),
                ),
                TypedNode::new(
                    "source.term.functor-application.inline",
                    SourceAnchor::Range(range(source, 24, 30)),
                ),
                TypedNode::new(
                    "source.term.functor-head.single",
                    SourceAnchor::Range(range(source, 24, 25)),
                ),
                TypedNode::new(
                    "source.term.structure.constructor",
                    SourceAnchor::Range(range(source, 10, 50)),
                ),
                TypedNode::new(
                    "source.term.structure.member.constructor-assignment",
                    SourceAnchor::Range(range(source, 12, 19)),
                ),
            ],
        )
        .unwrap();
        let primary = primary_handoff(source, &module, &bindings, &arena, &[]);
        let (symbols, structure, contribution) =
            local_structure_symbols(source, &module, RootOptions::default());
        let applications = SourceFunctorApplicationProducer::build(
            SourceFunctorApplicationHandoffInput {
                source_id: source,
                module_id: module.clone(),
                applications: vec![
                    SourceFunctorApplicationInput {
                        site: node(0),
                        source_range: range(source, 20, 40),
                        source_ordinal: 0,
                        context: BindingContextId::new(0),
                        recovery: SourceFunctorApplicationRecovery::Normal,
                        spelling: "f ( g ( ) )".to_owned(),
                        kind: SourceFunctorApplicationKind::Inline,
                        form: SourceFunctorApplicationForm::Functional,
                        head_ordinal: 0,
                        head: SourceFunctorHeadSite::Single {
                            site: node(1),
                            source_range: range(source, 20, 21),
                            spelling: "f".to_owned(),
                        },
                    },
                    SourceFunctorApplicationInput {
                        site: node(2),
                        source_range: range(source, 24, 30),
                        source_ordinal: 1,
                        context: BindingContextId::new(0),
                        recovery: SourceFunctorApplicationRecovery::Normal,
                        spelling: "g ( )".to_owned(),
                        kind: SourceFunctorApplicationKind::Inline,
                        form: SourceFunctorApplicationForm::Functional,
                        head_ordinal: 0,
                        head: SourceFunctorHeadSite::Single {
                            site: node(3),
                            source_range: range(source, 24, 25),
                            spelling: "g".to_owned(),
                        },
                    },
                ],
                wrappers: Vec::new(),
                candidates: Vec::new(),
                arguments: vec![SourceFunctorArgumentInput {
                    application: SourceFunctorApplicationId::new(0),
                    ordinal: 0,
                    target: SourceFunctorArgumentTarget::Application(
                        SourceFunctorApplicationId::new(1),
                    ),
                }],
                type_requests: Vec::new(),
            },
            &symbols,
            &bindings,
            &primary,
            &arena,
        )
        .expect("nested application handoff");
        let input = SourceStructureHandoffInput {
            source_id: source,
            module_id: module,
            terms: vec![SourceStructureTermInput {
                site: node(4),
                source_range: range(source, 10, 50),
                source_ordinal: 0,
                context: BindingContextId::new(0),
                recovery: SourceStructureRecovery::Normal,
                spelling: "Pair ( carrier : g ( ) )".to_owned(),
                kind: SourceStructureTermKind::Constructor,
            }],
            wrappers: Vec::new(),
            roots: vec![SourceStructureRootInput {
                term: SourceStructureTermId::new(0),
                symbol: structure,
                contribution,
            }],
            members: vec![SourceStructureMemberInput {
                term: SourceStructureTermId::new(0),
                ordinal: 0,
                site: node(5),
                source_range: range(source, 12, 19),
                spelling: "carrier".to_owned(),
                role: SourceStructureMemberRole::ConstructorAssignment,
                parent: None,
            }],
            field_updates: Vec::new(),
            edges: vec![SourceStructureEdgeInput {
                term: SourceStructureTermId::new(0),
                ordinal: 0,
                role: SourceStructureEdgeRole::ConstructorValue,
                member: Some(SourceStructureMemberId::new(0)),
                target: SourceStructureTarget::Application(SourceFunctorApplicationId::new(1)),
            }],
            requests: vec![
                SourceStructureRequestInput {
                    term: SourceStructureTermId::new(0),
                    member: None,
                    request_ordinal: 0,
                    kind: SourceStructureRequestKind::ConstructorSignature,
                },
                SourceStructureRequestInput {
                    term: SourceStructureTermId::new(0),
                    member: Some(SourceStructureMemberId::new(0)),
                    request_ordinal: 1,
                    kind: SourceStructureRequestKind::MemberIdentity,
                },
                SourceStructureRequestInput {
                    term: SourceStructureTermId::new(0),
                    member: Some(SourceStructureMemberId::new(0)),
                    request_ordinal: 2,
                    kind: SourceStructureRequestKind::InheritancePath,
                },
                SourceStructureRequestInput {
                    term: SourceStructureTermId::new(0),
                    member: None,
                    request_ordinal: 3,
                    kind: SourceStructureRequestKind::ResultType,
                },
            ],
        };
        assert!(matches!(
            SourceStructureProducer::build(
                input,
                &symbols,
                &bindings,
                &primary,
                Some(&applications),
                &arena,
            ),
            Err(SourceStructureError::DuplicateTarget { .. })
                | Err(SourceStructureError::InvalidEdge { .. })
                | Err(SourceStructureError::InvalidTerm { .. })
                | Err(SourceStructureError::ApplicationDependencyMismatch)
        ));
    }

    #[test]
    fn primary_fingerprint_and_arena_corruption_are_rejected_on_installation() {
        let fixture = constructor_fixture();
        let handoff = build(&fixture).unwrap();
        let alternate_arena = TypedArena::try_new(
            None,
            vec![
                TypedNode::new(
                    "source.term.numeral",
                    SourceAnchor::Range(range(fixture.source, 22, 23)),
                ),
                TypedNode::new(
                    "source.term.structure.selector",
                    SourceAnchor::Range(range(fixture.source, 10, 30)),
                ),
                TypedNode::new(
                    "source.term.structure.member.constructor-assignment",
                    SourceAnchor::Range(range(fixture.source, 13, 20)),
                ),
            ],
        )
        .unwrap();
        assert!(matches!(
            handoff.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.primary,
                None,
                &alternate_arena,
            ),
            Err(SourceStructureError::InvalidTerm { .. })
        ));
        let other_source = other_source_id("d8");
        assert!(matches!(
            handoff.validate_installation(
                other_source,
                &fixture.module,
                &fixture.primary,
                None,
                &fixture.arena,
            ),
            Err(SourceStructureError::PrimaryDependencyMismatch)
        ));
    }

    #[test]
    fn public_typed_ast_installation_rejects_missing_replacement_and_mismatched_dependencies() {
        let fixture = constructor_fixture();
        let handoff = build(&fixture).unwrap();
        let missing_primary = TypedAst::try_new(empty_typed_parts(&fixture)).unwrap();
        assert_eq!(
            missing_primary.with_source_structure(handoff.clone()),
            Err(TypedAstError::InvalidSourceStructure)
        );

        let typed = TypedAst::try_new(empty_typed_parts(&fixture))
            .unwrap()
            .with_source_term(fixture.primary.clone())
            .unwrap()
            .with_source_structure(handoff.clone())
            .expect("first structure installation");
        assert_eq!(typed.source_structure(), Some(&handoff));
        assert_eq!(
            typed.with_source_structure(handoff.clone()),
            Err(TypedAstError::InvalidSourceStructure)
        );

        let application_fixture = application_target_fixture();
        let application_handoff = build(&application_fixture).unwrap();
        let missing_application = TypedAst::try_new(empty_typed_parts(&application_fixture))
            .unwrap()
            .with_source_term(application_fixture.primary.clone())
            .unwrap();
        assert_eq!(
            missing_application.with_source_structure(application_handoff.clone()),
            Err(TypedAstError::InvalidSourceStructure)
        );

        let exact = TypedAst::try_new(empty_typed_parts(&application_fixture))
            .unwrap()
            .with_source_term(application_fixture.primary.clone())
            .unwrap()
            .with_source_application(application_fixture.applications.clone().unwrap())
            .unwrap()
            .with_source_structure(application_handoff.clone())
            .expect("exact Task-253 fingerprint");
        assert_eq!(exact.source_structure(), Some(&application_handoff));

        let wrapped = application_target_fixture_variant("source.structure.application", true);
        let mismatched = TypedAst::try_new(empty_typed_parts(&wrapped))
            .unwrap()
            .with_source_term(wrapped.primary.clone())
            .unwrap()
            .with_source_application(wrapped.applications.clone().unwrap())
            .unwrap();
        assert_eq!(
            mismatched.with_source_structure(application_handoff),
            Err(TypedAstError::InvalidSourceStructure)
        );
    }
}
