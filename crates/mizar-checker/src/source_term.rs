//! Syntax-free transport for source primary-term occurrences.

use crate::{
    binding_env::{
        BinderIdentity, BindingContextDraft, BindingContextId, BindingContextLayer,
        BindingContextOwner, BindingContextRecovery, BindingContextTable, BindingDiagnosticTable,
        BindingDraft, BindingEnv, BindingEnvParts, BindingId, BindingKind, BindingLookupResult,
        BindingLookupSite, BindingRecoveryState, BindingStatus, BindingTable, BindingTypeSite,
        CapturedFreeVariables,
    },
    source_formula_composition::SourceNestedFraenkelBinderUseHandoff,
    source_type::{
        SourceProofLocalGivenConditionTypeHandoff, SourceProofLocalGivenDescendantTypeHandoff,
        SourceProofLocalGivenUseTypeHandoff,
    },
    typed_ast::{NodeRecoveryState, TypedArena, TypedNode, TypedNodeId, TypedSiteRef, TypingState},
};
use mizar_lexer::is_identifier;
use mizar_resolve::{names::LocalTermScope, resolved_ast::ModuleId};
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

dense_id!(SourcePrimaryTermId);
dense_id!(SourcePrimaryTermReferenceId);
dense_id!(SourceNumericTypeRequestId);

/// Complete input for one source/module primary-term transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePrimaryTermHandoffInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub terms: Vec<SourcePrimaryTermInput>,
    pub references: Vec<SourcePrimaryTermReferenceInput>,
    pub numeric_type_requests: Vec<SourceNumericTypeRequestInput>,
}

/// One source primary-term occurrence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePrimaryTermInput {
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub source_ordinal: usize,
    pub context: crate::binding_env::BindingContextId,
    pub recovery: SourcePrimaryTermRecovery,
    pub spelling: String,
    pub kind: SourcePrimaryTermKind,
    pub role: SourcePrimaryTermRole,
    pub parent: Option<SourcePrimaryTermId>,
}

/// One binding reference attached to a primary term.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePrimaryTermReferenceInput {
    pub term: SourcePrimaryTermId,
    pub binding: BindingId,
    pub role: SourcePrimaryTermReferenceRole,
}

/// One unresolved numeric-type request attached to a numeral.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceNumericTypeRequestInput {
    pub term: SourcePrimaryTermId,
    pub owner: TypedSiteRef,
    pub source_range: SourceRange,
    pub spelling: String,
    pub request_ordinal: usize,
}

/// Source primary-term shape admitted by Task 252.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourcePrimaryTermKind {
    VariableReference,
    ConstantReference,
    It,
    Numeral,
    Parenthesized,
}

/// Source role of a primary-term occurrence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourcePrimaryTermRole {
    Value,
    CurrentDefinitionResult,
}

/// Binding role of a source reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourcePrimaryTermReferenceRole {
    Variable,
    LocalConstant,
}

/// Recovery state retained at the source-term boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourcePrimaryTermRecovery {
    Normal,
    Degraded,
}

/// Immutable validated primary-term handoff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePrimaryTermHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    terms: SourcePrimaryTermTable,
    references: SourcePrimaryTermReferenceTable,
    numeric_type_requests: SourceNumericTypeRequestTable,
}

impl SourcePrimaryTermHandoff {
    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    pub const fn terms(&self) -> &SourcePrimaryTermTable {
        &self.terms
    }

    pub const fn references(&self) -> &SourcePrimaryTermReferenceTable {
        &self.references
    }

    pub const fn numeric_type_requests(&self) -> &SourceNumericTypeRequestTable {
        &self.numeric_type_requests
    }

    pub fn debug_text(&self) -> String {
        let mut output = String::from("source-primary-term-debug-v1\n");
        let _ = writeln!(output, "module: {}", self.module_id.path().as_str());
        for (id, term) in self.terms.iter() {
            let _ = write!(
                output,
                "term#{} ordinal={} kind={} role={} range={}..{} site={} context={} recovery={} spelling={:?} parent=",
                id.index(),
                term.source_ordinal,
                kind_key(term.kind),
                role_key(term.role),
                term.source_range.start,
                term.source_range.end,
                term.site.node().index(),
                term.context.index(),
                recovery_key(term.recovery),
                term.spelling
            );
            write_optional_term_id(&mut output, term.parent);
            output.push('\n');
        }
        for (id, reference) in self.references.iter() {
            let _ = write!(
                output,
                "reference#{} term={} binding={} role={} use_ordinal={} scope=",
                id.index(),
                reference.term.index(),
                reference.binding.index(),
                reference_role_key(reference.role),
                reference.use_ordinal
            );
            write_scope(&mut output, reference.lexical_scope.as_ref());
            output.push('\n');
        }
        for (id, request) in self.numeric_type_requests.iter() {
            let _ = writeln!(
                output,
                "numeric-request#{} term={} ordinal={} owner={} range={}..{} spelling={:?}",
                id.index(),
                request.term.index(),
                request.request_ordinal,
                request.owner.node().index(),
                request.source_range.start,
                request.source_range.end,
                request.spelling
            );
        }
        output
    }

    pub(crate) fn validate_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        arena: &TypedArena,
    ) -> Result<(), SourcePrimaryTermError> {
        if self.source_id != source_id || &self.module_id != module_id {
            return Err(SourcePrimaryTermError::InvalidTransaction);
        }
        for (id, term) in self.terms.iter() {
            validate_term_node(term, arena)
                .map_err(|()| SourcePrimaryTermError::InvalidTerm { term: id })?;
        }
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn set_reference_use_ordinal_for_test(
        &mut self,
        reference: SourcePrimaryTermReferenceId,
        use_ordinal: usize,
    ) {
        if let Some(row) = self.references.rows.get_mut(reference.index()) {
            row.use_ordinal = use_ordinal;
        }
    }

    #[cfg(test)]
    pub(crate) fn corrupt_for_test(&mut self, corruption: SourcePrimaryTermCorruptionForTest) {
        match corruption {
            SourcePrimaryTermCorruptionForTest::Truncate(len) => self.terms.rows.truncate(len),
            SourcePrimaryTermCorruptionForTest::Duplicate(term) => {
                if let Some(mut row) = self.terms.rows.get(term.index()).cloned() {
                    row.source_ordinal = self.terms.rows.len();
                    self.terms.rows.push(row);
                }
            }
            SourcePrimaryTermCorruptionForTest::Rewrite {
                term,
                site_and_range,
                spelling,
                kind_and_role,
            } => {
                if let Some(row) = self.terms.rows.get_mut(term.index()) {
                    if let Some((site, source_range)) = site_and_range {
                        row.site = site;
                        row.source_range = source_range;
                    }
                    if let Some(spelling) = spelling {
                        row.spelling = spelling;
                    }
                    if let Some((kind, role)) = kind_and_role {
                        row.kind = kind;
                        row.role = role;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
pub(crate) enum SourcePrimaryTermCorruptionForTest {
    Truncate(usize),
    Duplicate(SourcePrimaryTermId),
    Rewrite {
        term: SourcePrimaryTermId,
        site_and_range: Option<(TypedSiteRef, SourceRange)>,
        spelling: Option<String>,
        kind_and_role: Option<(SourcePrimaryTermKind, SourcePrimaryTermRole)>,
    },
}

/// Dense immutable primary-term table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePrimaryTermTable {
    rows: Vec<SourcePrimaryTerm>,
}

impl SourcePrimaryTermTable {
    pub fn get(&self, id: SourcePrimaryTermId) -> Option<&SourcePrimaryTerm> {
        self.rows.get(id.index())
    }

    pub fn iter(&self) -> impl Iterator<Item = (SourcePrimaryTermId, &SourcePrimaryTerm)> {
        self.rows
            .iter()
            .enumerate()
            .map(|(index, row)| (SourcePrimaryTermId::new(index), row))
    }

    pub const fn len(&self) -> usize {
        self.rows.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

/// One validated source primary-term row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePrimaryTerm {
    site: TypedSiteRef,
    source_range: SourceRange,
    source_ordinal: usize,
    context: crate::binding_env::BindingContextId,
    recovery: SourcePrimaryTermRecovery,
    spelling: String,
    kind: SourcePrimaryTermKind,
    role: SourcePrimaryTermRole,
    parent: Option<SourcePrimaryTermId>,
}

impl SourcePrimaryTerm {
    pub const fn site(&self) -> &TypedSiteRef {
        &self.site
    }

    pub const fn source_range(&self) -> SourceRange {
        self.source_range
    }

    pub const fn source_ordinal(&self) -> usize {
        self.source_ordinal
    }

    pub const fn context(&self) -> crate::binding_env::BindingContextId {
        self.context
    }

    pub const fn recovery(&self) -> SourcePrimaryTermRecovery {
        self.recovery
    }

    pub fn spelling(&self) -> &str {
        &self.spelling
    }

    pub const fn kind(&self) -> SourcePrimaryTermKind {
        self.kind
    }

    pub const fn role(&self) -> SourcePrimaryTermRole {
        self.role
    }

    pub const fn parent(&self) -> Option<SourcePrimaryTermId> {
        self.parent
    }
}

/// Dense immutable source-reference table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePrimaryTermReferenceTable {
    rows: Vec<SourcePrimaryTermReference>,
}

impl SourcePrimaryTermReferenceTable {
    pub fn get(&self, id: SourcePrimaryTermReferenceId) -> Option<&SourcePrimaryTermReference> {
        self.rows.get(id.index())
    }

    pub fn iter(
        &self,
    ) -> impl Iterator<Item = (SourcePrimaryTermReferenceId, &SourcePrimaryTermReference)> {
        self.rows
            .iter()
            .enumerate()
            .map(|(index, row)| (SourcePrimaryTermReferenceId::new(index), row))
    }

    pub const fn len(&self) -> usize {
        self.rows.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

/// One validated binding reference with producer-derived lookup coordinates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePrimaryTermReference {
    term: SourcePrimaryTermId,
    binding: BindingId,
    role: SourcePrimaryTermReferenceRole,
    lexical_scope: Option<LocalTermScope>,
    use_ordinal: usize,
}

impl SourcePrimaryTermReference {
    pub const fn term(&self) -> SourcePrimaryTermId {
        self.term
    }

    pub const fn binding(&self) -> BindingId {
        self.binding
    }

    pub const fn role(&self) -> SourcePrimaryTermReferenceRole {
        self.role
    }

    pub const fn lexical_scope(&self) -> Option<&LocalTermScope> {
        self.lexical_scope.as_ref()
    }

    pub const fn use_ordinal(&self) -> usize {
        self.use_ordinal
    }
}

/// Dense immutable numeric-type-request table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceNumericTypeRequestTable {
    rows: Vec<SourceNumericTypeRequest>,
}

impl SourceNumericTypeRequestTable {
    pub fn get(&self, id: SourceNumericTypeRequestId) -> Option<&SourceNumericTypeRequest> {
        self.rows.get(id.index())
    }

    pub fn iter(
        &self,
    ) -> impl Iterator<Item = (SourceNumericTypeRequestId, &SourceNumericTypeRequest)> {
        self.rows
            .iter()
            .enumerate()
            .map(|(index, row)| (SourceNumericTypeRequestId::new(index), row))
    }

    pub const fn len(&self) -> usize {
        self.rows.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

/// One validated unresolved numeric-type request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceNumericTypeRequest {
    term: SourcePrimaryTermId,
    owner: TypedSiteRef,
    source_range: SourceRange,
    spelling: String,
    request_ordinal: usize,
}

impl SourceNumericTypeRequest {
    pub const fn term(&self) -> SourcePrimaryTermId {
        self.term
    }

    pub const fn owner(&self) -> &TypedSiteRef {
        &self.owner
    }

    pub const fn source_range(&self) -> SourceRange {
        self.source_range
    }

    pub fn spelling(&self) -> &str {
        &self.spelling
    }

    pub const fn request_ordinal(&self) -> usize {
        self.request_ordinal
    }
}

/// Atomic Task-252 producer failure.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourcePrimaryTermError {
    InvalidTransaction,
    InvalidTerm {
        term: SourcePrimaryTermId,
    },
    InvalidReference {
        reference: SourcePrimaryTermReferenceId,
    },
    InvalidNumericTypeRequest {
        request: SourceNumericTypeRequestId,
    },
    InvalidBindingEvent {
        event: usize,
    },
}

impl fmt::Display for SourcePrimaryTermError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTransaction => {
                formatter.write_str("source primary-term transaction is inconsistent")
            }
            Self::InvalidTerm { term } => {
                write!(
                    formatter,
                    "source primary term {} is inconsistent",
                    term.index()
                )
            }
            Self::InvalidReference { reference } => write!(
                formatter,
                "source primary-term reference {} is inconsistent",
                reference.index()
            ),
            Self::InvalidNumericTypeRequest { request } => write!(
                formatter,
                "source numeric-type request {} is inconsistent",
                request.index()
            ),
            Self::InvalidBindingEvent { event } => {
                write!(formatter, "source binding event {event} is inconsistent")
            }
        }
    }
}

impl Error for SourcePrimaryTermError {}

/// Immutable exact Task-269GU proof-local `given` later-use term handoff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalGivenUseTermHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    dependency: SourceProofLocalGivenUseTypeHandoff,
    dependency_fingerprint: String,
    source_term: SourcePrimaryTermHandoff,
    source_term_fingerprint: String,
}

impl SourceProofLocalGivenUseTermHandoff {
    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    pub const fn dependency(&self) -> &SourceProofLocalGivenUseTypeHandoff {
        &self.dependency
    }

    pub fn dependency_fingerprint(&self) -> &str {
        &self.dependency_fingerprint
    }

    pub const fn source_term(&self) -> &SourcePrimaryTermHandoff {
        &self.source_term
    }

    pub fn source_term_fingerprint(&self) -> &str {
        &self.source_term_fingerprint
    }

    pub fn debug_text(&self) -> String {
        format!(
            concat!(
                "source-proof-local-given-use-term-debug-v1\n",
                "module: {}::{}\n",
                "dependency-fingerprint: {:?}\n",
                "source-term-fingerprint: {:?}\n",
            ),
            self.module_id.package().as_str(),
            self.module_id.path().as_str(),
            self.dependency_fingerprint,
            self.source_term_fingerprint,
        )
    }

    pub(crate) fn validate_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        arena: &TypedArena,
    ) -> Result<(), SourceProofLocalGivenUseTermError> {
        let dependency_arena = task269gu_dependency_arena(arena)?;
        self.dependency
            .validate_installation(source_id, module_id, &dependency_arena)
            .map_err(|_| SourceProofLocalGivenUseTermError::InvalidDependency)?;
        if self.source_id != source_id
            || &self.module_id != module_id
            || self.dependency_fingerprint != self.dependency.debug_text()
        {
            return Err(SourceProofLocalGivenUseTermError::InvalidDependency);
        }
        let expected = SourcePrimaryTermProducer::build_with_profile(
            task269gu_input(source_id, module_id.clone()),
            self.dependency.binding_env(),
            arena,
            SourcePrimaryTermBindingProfile::ProofLocalGivenUse,
        )
        .map_err(|_| SourceProofLocalGivenUseTermError::InvalidSourceTerm)?;
        if self.source_term != expected
            || self.source_term_fingerprint != self.source_term.debug_text()
        {
            return Err(SourceProofLocalGivenUseTermError::InvalidSourceTerm);
        }
        if !exact_task269gu_arena(source_id, arena) {
            return Err(SourceProofLocalGivenUseTermError::InvalidInstallation);
        }
        Ok(())
    }

    pub(crate) fn validate_complete_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        arena: &TypedArena,
        installation_available: bool,
    ) -> Result<(), SourceProofLocalGivenUseTermError> {
        self.validate_installation(source_id, module_id, arena)?;
        if !installation_available {
            return Err(SourceProofLocalGivenUseTermError::InvalidInstallation);
        }
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn set_dependency_fingerprint_for_test(&mut self, fingerprint: String) {
        self.dependency_fingerprint = fingerprint;
    }

    #[cfg(test)]
    pub(crate) fn source_term_mut_for_test(&mut self) -> &mut SourcePrimaryTermHandoff {
        &mut self.source_term
    }
}

/// Builds only the exact Task-269GU proof-local `given` later-use terms.
pub struct SourceProofLocalGivenUseTermProducer;

impl SourceProofLocalGivenUseTermProducer {
    pub fn build(
        dependency: SourceProofLocalGivenUseTypeHandoff,
        input: SourcePrimaryTermHandoffInput,
        arena: &TypedArena,
    ) -> Result<SourceProofLocalGivenUseTermHandoff, SourceProofLocalGivenUseTermError> {
        let dependency_arena = task269gu_dependency_arena(arena)?;
        dependency
            .validate_installation(input.source_id, &input.module_id, &dependency_arena)
            .map_err(|_| SourceProofLocalGivenUseTermError::InvalidDependency)?;
        let dependency_fingerprint = dependency.debug_text();
        if !exact_task269gu_input(&input) {
            return Err(SourceProofLocalGivenUseTermError::InvalidSourceTerm);
        }
        let source_term = SourcePrimaryTermProducer::build_with_profile(
            input,
            dependency.binding_env(),
            arena,
            SourcePrimaryTermBindingProfile::ProofLocalGivenUse,
        )
        .map_err(|_| SourceProofLocalGivenUseTermError::InvalidSourceTerm)?;
        let source_term_fingerprint = source_term.debug_text();
        if !exact_task269gu_arena(dependency.source_id(), arena) {
            return Err(SourceProofLocalGivenUseTermError::InvalidInstallation);
        }
        let handoff = SourceProofLocalGivenUseTermHandoff {
            source_id: dependency.source_id(),
            module_id: dependency.module_id().clone(),
            dependency,
            dependency_fingerprint,
            source_term,
            source_term_fingerprint,
        };
        handoff.validate_installation(handoff.source_id, &handoff.module_id, arena)?;
        Ok(handoff)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceProofLocalGivenUseTermError {
    InvalidDependency,
    InvalidSourceTerm,
    InvalidInstallation,
}

impl fmt::Display for SourceProofLocalGivenUseTermError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDependency => {
                formatter.write_str("source proof-local given-use term dependency is invalid")
            }
            Self::InvalidSourceTerm => {
                formatter.write_str("source proof-local given-use source term is invalid")
            }
            Self::InvalidInstallation => {
                formatter.write_str("source proof-local given-use term installation is invalid")
            }
        }
    }
}

impl Error for SourceProofLocalGivenUseTermError {}

/// Immutable exact Task-269SDU proof-local `given` descendant-use term handoff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalGivenDescendantUseTermHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    dependency: SourceProofLocalGivenDescendantTypeHandoff,
    dependency_fingerprint: String,
    source_term: SourcePrimaryTermHandoff,
    source_term_fingerprint: String,
}

impl SourceProofLocalGivenDescendantUseTermHandoff {
    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }
    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
    }
    pub const fn dependency(&self) -> &SourceProofLocalGivenDescendantTypeHandoff {
        &self.dependency
    }
    pub fn dependency_fingerprint(&self) -> &str {
        &self.dependency_fingerprint
    }
    pub const fn source_term(&self) -> &SourcePrimaryTermHandoff {
        &self.source_term
    }
    pub fn source_term_fingerprint(&self) -> &str {
        &self.source_term_fingerprint
    }

    pub fn debug_text(&self) -> String {
        format!(
            concat!(
                "source-proof-local-given-descendant-use-term-debug-v1\n",
                "module: {}::{}\n",
                "dependency-fingerprint: {:?}\n",
                "source-term-fingerprint: {:?}\n",
            ),
            self.module_id.package().as_str(),
            self.module_id.path().as_str(),
            self.dependency_fingerprint,
            self.source_term_fingerprint
        )
    }

    pub(crate) fn validate_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        arena: &TypedArena,
    ) -> Result<(), SourceProofLocalGivenDescendantUseTermError> {
        let dependency_arena = task269sdu_dependency_arena(arena)?;
        self.dependency
            .validate_installation(source_id, module_id, &dependency_arena)
            .map_err(|_| SourceProofLocalGivenDescendantUseTermError::InvalidDependency)?;
        if self.source_id != source_id
            || &self.module_id != module_id
            || self.dependency_fingerprint != self.dependency.debug_text()
        {
            return Err(SourceProofLocalGivenDescendantUseTermError::InvalidDependency);
        }
        let expected = SourcePrimaryTermProducer::build_with_profile(
            task269sdu_input(source_id, module_id.clone()),
            self.dependency.binding_env(),
            arena,
            SourcePrimaryTermBindingProfile::ProofLocalGivenDescendantUse,
        )
        .map_err(|_| SourceProofLocalGivenDescendantUseTermError::InvalidSourceTerm)?;
        if self.source_term != expected
            || self.source_term_fingerprint != self.source_term.debug_text()
        {
            return Err(SourceProofLocalGivenDescendantUseTermError::InvalidSourceTerm);
        }
        if !exact_task269sdu_arena(source_id, arena) {
            return Err(SourceProofLocalGivenDescendantUseTermError::InvalidInstallation);
        }
        Ok(())
    }

    pub(crate) fn validate_complete_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        arena: &TypedArena,
        installation_available: bool,
    ) -> Result<(), SourceProofLocalGivenDescendantUseTermError> {
        self.validate_installation(source_id, module_id, arena)?;
        if !installation_available {
            return Err(SourceProofLocalGivenDescendantUseTermError::InvalidInstallation);
        }
        Ok(())
    }
}

/// Builds only the exact Task-269SDU proof-local `given` descendant-use term.
pub struct SourceProofLocalGivenDescendantUseTermProducer;

impl SourceProofLocalGivenDescendantUseTermProducer {
    pub fn build(
        dependency: SourceProofLocalGivenDescendantTypeHandoff,
        input: SourcePrimaryTermHandoffInput,
        arena: &TypedArena,
    ) -> Result<
        SourceProofLocalGivenDescendantUseTermHandoff,
        SourceProofLocalGivenDescendantUseTermError,
    > {
        let dependency_arena = task269sdu_dependency_arena(arena)?;
        dependency
            .validate_installation(input.source_id, &input.module_id, &dependency_arena)
            .map_err(|_| SourceProofLocalGivenDescendantUseTermError::InvalidDependency)?;
        let dependency_fingerprint = dependency.debug_text();
        if !exact_task269sdu_input(&input) {
            return Err(SourceProofLocalGivenDescendantUseTermError::InvalidSourceTerm);
        }
        let source_term = SourcePrimaryTermProducer::build_with_profile(
            input,
            dependency.binding_env(),
            arena,
            SourcePrimaryTermBindingProfile::ProofLocalGivenDescendantUse,
        )
        .map_err(|_| SourceProofLocalGivenDescendantUseTermError::InvalidSourceTerm)?;
        let source_term_fingerprint = source_term.debug_text();
        if !exact_task269sdu_arena(dependency.source_id(), arena) {
            return Err(SourceProofLocalGivenDescendantUseTermError::InvalidInstallation);
        }
        let handoff = SourceProofLocalGivenDescendantUseTermHandoff {
            source_id: dependency.source_id(),
            module_id: dependency.module_id().clone(),
            dependency,
            dependency_fingerprint,
            source_term,
            source_term_fingerprint,
        };
        handoff.validate_installation(handoff.source_id, &handoff.module_id, arena)?;
        Ok(handoff)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceProofLocalGivenDescendantUseTermError {
    InvalidDependency,
    InvalidSourceTerm,
    InvalidInstallation,
}

impl fmt::Display for SourceProofLocalGivenDescendantUseTermError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDependency => formatter
                .write_str("source proof-local given-descendant-use term dependency is invalid"),
            Self::InvalidSourceTerm => formatter
                .write_str("source proof-local given-descendant-use source term is invalid"),
            Self::InvalidInstallation => formatter
                .write_str("source proof-local given-descendant-use term installation is invalid"),
        }
    }
}
impl Error for SourceProofLocalGivenDescendantUseTermError {}

/// Immutable exact Task-269GCU proof-local `given` condition term handoff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalGivenConditionUseTermHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    dependency: SourceProofLocalGivenConditionTypeHandoff,
    dependency_fingerprint: String,
    source_term: SourcePrimaryTermHandoff,
    source_term_fingerprint: String,
}

impl SourceProofLocalGivenConditionUseTermHandoff {
    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    pub const fn dependency(&self) -> &SourceProofLocalGivenConditionTypeHandoff {
        &self.dependency
    }

    pub fn dependency_fingerprint(&self) -> &str {
        &self.dependency_fingerprint
    }

    pub const fn source_term(&self) -> &SourcePrimaryTermHandoff {
        &self.source_term
    }

    pub fn source_term_fingerprint(&self) -> &str {
        &self.source_term_fingerprint
    }

    pub fn debug_text(&self) -> String {
        format!(
            concat!(
                "source-proof-local-given-condition-use-term-debug-v1\n",
                "module: {}::{}\n",
                "dependency-fingerprint: {:?}\n",
                "source-term-fingerprint: {:?}\n",
            ),
            self.module_id.package().as_str(),
            self.module_id.path().as_str(),
            self.dependency_fingerprint,
            self.source_term_fingerprint,
        )
    }

    pub(crate) fn validate_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        arena: &TypedArena,
    ) -> Result<(), SourceProofLocalGivenConditionUseTermError> {
        let dependency_arena = task269gcu_dependency_arena(arena)?;
        self.dependency
            .validate_installation(source_id, module_id, &dependency_arena)
            .map_err(|_| SourceProofLocalGivenConditionUseTermError::InvalidDependency)?;
        if self.source_id != source_id
            || &self.module_id != module_id
            || self.dependency_fingerprint != self.dependency.debug_text()
        {
            return Err(SourceProofLocalGivenConditionUseTermError::InvalidDependency);
        }
        let expected = SourcePrimaryTermProducer::build_with_profile(
            task269gcu_input(source_id, module_id.clone()),
            self.dependency.binding_env(),
            arena,
            SourcePrimaryTermBindingProfile::ProofLocalGivenConditionUse,
        )
        .map_err(|_| SourceProofLocalGivenConditionUseTermError::InvalidSourceTerm)?;
        if self.source_term != expected
            || self.source_term_fingerprint != self.source_term.debug_text()
        {
            return Err(SourceProofLocalGivenConditionUseTermError::InvalidSourceTerm);
        }
        if !exact_task269gcu_arena(source_id, arena) {
            return Err(SourceProofLocalGivenConditionUseTermError::InvalidInstallation);
        }
        Ok(())
    }

    pub(crate) fn validate_complete_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        arena: &TypedArena,
        installation_available: bool,
    ) -> Result<(), SourceProofLocalGivenConditionUseTermError> {
        self.validate_installation(source_id, module_id, arena)?;
        if !installation_available {
            return Err(SourceProofLocalGivenConditionUseTermError::InvalidInstallation);
        }
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn set_dependency_fingerprint_for_test(&mut self, fingerprint: String) {
        self.dependency_fingerprint = fingerprint;
    }

    #[cfg(test)]
    pub(crate) fn source_term_mut_for_test(&mut self) -> &mut SourcePrimaryTermHandoff {
        &mut self.source_term
    }
}

/// Builds only the exact Task-269GCU proof-local `given` condition terms.
pub struct SourceProofLocalGivenConditionUseTermProducer;

impl SourceProofLocalGivenConditionUseTermProducer {
    pub fn build(
        dependency: SourceProofLocalGivenConditionTypeHandoff,
        input: SourcePrimaryTermHandoffInput,
        arena: &TypedArena,
    ) -> Result<
        SourceProofLocalGivenConditionUseTermHandoff,
        SourceProofLocalGivenConditionUseTermError,
    > {
        let dependency_arena = task269gcu_dependency_arena(arena)?;
        dependency
            .validate_installation(input.source_id, &input.module_id, &dependency_arena)
            .map_err(|_| SourceProofLocalGivenConditionUseTermError::InvalidDependency)?;
        let dependency_fingerprint = dependency.debug_text();
        if !exact_task269gcu_input(&input) {
            return Err(SourceProofLocalGivenConditionUseTermError::InvalidSourceTerm);
        }
        let source_term = SourcePrimaryTermProducer::build_with_profile(
            input,
            dependency.binding_env(),
            arena,
            SourcePrimaryTermBindingProfile::ProofLocalGivenConditionUse,
        )
        .map_err(|_| SourceProofLocalGivenConditionUseTermError::InvalidSourceTerm)?;
        let source_term_fingerprint = source_term.debug_text();
        if !exact_task269gcu_arena(dependency.source_id(), arena) {
            return Err(SourceProofLocalGivenConditionUseTermError::InvalidInstallation);
        }
        let handoff = SourceProofLocalGivenConditionUseTermHandoff {
            source_id: dependency.source_id(),
            module_id: dependency.module_id().clone(),
            dependency,
            dependency_fingerprint,
            source_term,
            source_term_fingerprint,
        };
        handoff.validate_installation(handoff.source_id, &handoff.module_id, arena)?;
        Ok(handoff)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceProofLocalGivenConditionUseTermError {
    InvalidDependency,
    InvalidSourceTerm,
    InvalidInstallation,
}

impl fmt::Display for SourceProofLocalGivenConditionUseTermError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDependency => formatter
                .write_str("source proof-local given-condition-use term dependency is invalid"),
            Self::InvalidSourceTerm => {
                formatter.write_str("source proof-local given-condition-use source term is invalid")
            }
            Self::InvalidInstallation => formatter
                .write_str("source proof-local given-condition-use term installation is invalid"),
        }
    }
}

impl Error for SourceProofLocalGivenConditionUseTermError {}

/// Immutable Task-252 projection of the single nested Fraenkel mapper primary.
#[derive(Clone, PartialEq, Eq)]
pub struct SourceNestedFraenkelMapperPrimaryHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    dependency: SourceNestedFraenkelBinderUseHandoff,
    dependency_fingerprint: String,
    binding_env: BindingEnv,
    binding_fingerprint: String,
    projection_arena: TypedArena,
    source_term: SourcePrimaryTermHandoff,
    source_term_fingerprint: String,
}

impl SourceNestedFraenkelMapperPrimaryHandoff {
    #[must_use]
    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    #[must_use]
    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    #[must_use]
    pub const fn dependency(&self) -> &SourceNestedFraenkelBinderUseHandoff {
        &self.dependency
    }

    #[must_use]
    pub fn dependency_fingerprint(&self) -> &str {
        &self.dependency_fingerprint
    }

    #[must_use]
    pub const fn binding_env(&self) -> &BindingEnv {
        &self.binding_env
    }

    #[must_use]
    pub fn binding_fingerprint(&self) -> &str {
        &self.binding_fingerprint
    }

    #[must_use]
    pub const fn projection_arena(&self) -> &TypedArena {
        &self.projection_arena
    }

    #[must_use]
    pub const fn source_term(&self) -> &SourcePrimaryTermHandoff {
        &self.source_term
    }

    #[must_use]
    pub fn source_term_fingerprint(&self) -> &str {
        &self.source_term_fingerprint
    }

    #[must_use]
    pub fn debug_text(&self) -> String {
        format!(
            "source-nested-fraenkel-mapper-primary-debug-v1\nmodule: {}::{}\ndependency-fingerprint: {:?}\nbinding-fingerprint: {:?}\nprojection: nodes=1 root=0\nsource-term-fingerprint: {:?}\n",
            self.module_id.package().as_str(),
            self.module_id.path().as_str(),
            self.dependency_fingerprint,
            self.binding_fingerprint,
            self.source_term_fingerprint,
        )
    }

    fn validate(&self) -> Result<(), SourceNestedFraenkelMapperPrimaryError> {
        if self.source_id != self.dependency.source_id()
            || &self.module_id != self.dependency.module_id()
            || self.dependency_fingerprint != self.dependency.debug_text()
            || self.dependency.validate_complete().is_err()
        {
            return Err(SourceNestedFraenkelMapperPrimaryError::InvalidDependency);
        }
        let expected_bindings =
            nested_fraenkel_mapper_primary_binding_env(self.source_id, self.module_id.clone())
                .map_err(|_| SourceNestedFraenkelMapperPrimaryError::InvalidBindingEnvironment)?;
        if self.binding_env != expected_bindings
            || self.binding_fingerprint != self.binding_env.debug_text()
            || !nested_fraenkel_mapper_primary_lookup_profile(&self.binding_env)
        {
            return Err(SourceNestedFraenkelMapperPrimaryError::InvalidBindingEnvironment);
        }
        let expected_arena = nested_fraenkel_mapper_primary_arena(self.source_id)
            .map_err(|_| SourceNestedFraenkelMapperPrimaryError::InvalidSourceTerm)?;
        if self.projection_arena != expected_arena {
            return Err(SourceNestedFraenkelMapperPrimaryError::InvalidSourceTerm);
        }
        let expected_source_term = SourcePrimaryTermProducer::build_with_profile(
            nested_fraenkel_mapper_primary_input(self.source_id, self.module_id.clone()),
            &self.binding_env,
            &self.projection_arena,
            SourcePrimaryTermBindingProfile::NestedFraenkelMapperPrimary,
        )
        .map_err(|_| SourceNestedFraenkelMapperPrimaryError::InvalidSourceTerm)?;
        if self.source_term != expected_source_term
            || self.source_term_fingerprint != self.source_term.debug_text()
        {
            return Err(SourceNestedFraenkelMapperPrimaryError::InvalidSourceTerm);
        }
        Ok(())
    }

    /// Reauthenticates the complete retained C4C4 transaction.
    ///
    /// This is intentionally crate-private: a later structural association may
    /// retain the handoff, but cannot bypass its C4C3, binding, arena, or
    /// primary-term validation.
    pub(crate) fn validate_complete(&self) -> Result<(), SourceNestedFraenkelMapperPrimaryError> {
        self.validate()
    }
}

/// A rejected nested Fraenkel mapper-primary transport.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceNestedFraenkelMapperPrimaryError {
    InvalidDependency,
    InvalidBindingEnvironment,
    InvalidSourceTerm,
}

impl fmt::Display for SourceNestedFraenkelMapperPrimaryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDependency => {
                formatter.write_str("nested Fraenkel mapper-primary dependency is invalid")
            }
            Self::InvalidBindingEnvironment => {
                formatter.write_str("nested Fraenkel mapper-primary binding environment is invalid")
            }
            Self::InvalidSourceTerm => {
                formatter.write_str("nested Fraenkel mapper-primary source term is invalid")
            }
        }
    }
}

impl Error for SourceNestedFraenkelMapperPrimaryError {}

/// Builds only the exact nested Fraenkel mapper-primary Task-252 transport.
#[derive(Debug, Clone, Copy)]
pub struct SourceNestedFraenkelMapperPrimaryProducer;

impl SourceNestedFraenkelMapperPrimaryProducer {
    pub fn build(
        dependency: SourceNestedFraenkelBinderUseHandoff,
    ) -> Result<SourceNestedFraenkelMapperPrimaryHandoff, SourceNestedFraenkelMapperPrimaryError>
    {
        dependency
            .validate_complete()
            .map_err(|_| SourceNestedFraenkelMapperPrimaryError::InvalidDependency)?;
        let source_id = dependency.source_id();
        let module_id = dependency.module_id().clone();
        let binding_env = nested_fraenkel_mapper_primary_binding_env(source_id, module_id.clone())
            .map_err(|_| SourceNestedFraenkelMapperPrimaryError::InvalidBindingEnvironment)?;
        if !nested_fraenkel_mapper_primary_lookup_profile(&binding_env) {
            return Err(SourceNestedFraenkelMapperPrimaryError::InvalidBindingEnvironment);
        }
        let projection_arena = nested_fraenkel_mapper_primary_arena(source_id)
            .map_err(|_| SourceNestedFraenkelMapperPrimaryError::InvalidSourceTerm)?;
        let source_term = SourcePrimaryTermProducer::build_with_profile(
            nested_fraenkel_mapper_primary_input(source_id, module_id.clone()),
            &binding_env,
            &projection_arena,
            SourcePrimaryTermBindingProfile::NestedFraenkelMapperPrimary,
        )
        .map_err(|_| SourceNestedFraenkelMapperPrimaryError::InvalidSourceTerm)?;
        let handoff = SourceNestedFraenkelMapperPrimaryHandoff {
            source_id,
            module_id,
            dependency_fingerprint: dependency.debug_text(),
            dependency,
            binding_fingerprint: binding_env.debug_text(),
            binding_env,
            projection_arena,
            source_term_fingerprint: source_term.debug_text(),
            source_term,
        };
        handoff.validate()?;
        Ok(handoff)
    }
}

fn nested_fraenkel_mapper_primary_range(
    source_id: SourceId,
    start: usize,
    end: usize,
) -> SourceRange {
    SourceRange {
        source_id,
        start,
        end,
    }
}

fn nested_fraenkel_mapper_primary_binding_env(
    source_id: SourceId,
    module_id: ModuleId,
) -> Result<BindingEnv, crate::binding_env::BindingEnvError> {
    let module = BindingContextId::new(0);
    let outer = BindingContextId::new(1);
    let binding = BindingId::new(0);
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
    contexts.insert(BindingContextDraft {
        owner: BindingContextOwner::SourceComprehension {
            source_range: nested_fraenkel_mapper_primary_range(source_id, 90, 157),
        },
        parent: Some(module),
        layer: BindingContextLayer::Expression,
        lexical_scope: None,
        bindings: vec![binding],
        visible_bindings: vec![binding],
        recovery: BindingContextRecovery::Normal,
    });
    contexts.insert(BindingContextDraft {
        owner: BindingContextOwner::SourceComprehension {
            source_range: nested_fraenkel_mapper_primary_range(source_id, 92, 123),
        },
        parent: Some(outer),
        layer: BindingContextLayer::Expression,
        lexical_scope: None,
        bindings: Vec::new(),
        visible_bindings: vec![binding],
        recovery: BindingContextRecovery::Normal,
    });
    let mut bindings = BindingTable::new();
    bindings.insert(BindingDraft {
        spelling: "x".to_owned(),
        kind: BindingKind::QuantifierBinder,
        identity: BinderIdentity::SourceBound {
            context: outer,
            ordinal: 0,
        },
        owner_context: outer,
        declaration_range: nested_fraenkel_mapper_primary_range(source_id, 136, 137),
        visible_after_ordinal: 0,
        type_site: BindingTypeSite::Source(nested_fraenkel_mapper_primary_range(
            source_id, 141, 155,
        )),
        status: BindingStatus::Active,
        captured: CapturedFreeVariables::default(),
        diagnostics: Vec::new(),
        recovery: BindingRecoveryState::Normal,
    });
    BindingEnv::try_new(BindingEnvParts {
        source_id,
        module_id,
        contexts,
        bindings,
        diagnostics: BindingDiagnosticTable::new(),
    })
}

fn nested_fraenkel_mapper_primary_lookup_profile(binding_env: &BindingEnv) -> bool {
    let forward = BindingLookupSite::new("x", BindingContextId::new(2), None, 0);
    let local = BindingLookupSite::new("x", BindingContextId::new(2), None, 1);
    matches!(
        binding_env.lookup(&forward),
        Ok(BindingLookupResult::ForwardReference { candidates, .. }) if candidates == [BindingId::new(0)]
    ) && matches!(binding_env.lookup(&local), Ok(BindingLookupResult::Local(binding)) if binding == BindingId::new(0))
}

fn nested_fraenkel_mapper_primary_arena(
    source_id: SourceId,
) -> Result<TypedArena, crate::typed_ast::TypedArenaError> {
    TypedArena::try_new(
        Some(TypedNodeId::new(0)),
        vec![TypedNode::new(
            "source.term.variable-reference",
            SourceAnchor::Range(nested_fraenkel_mapper_primary_range(source_id, 94, 95)),
        )],
    )
}

fn nested_fraenkel_mapper_primary_input(
    source_id: SourceId,
    module_id: ModuleId,
) -> SourcePrimaryTermHandoffInput {
    SourcePrimaryTermHandoffInput {
        source_id,
        module_id,
        terms: vec![SourcePrimaryTermInput {
            site: TypedSiteRef::Node(TypedNodeId::new(0)),
            source_range: nested_fraenkel_mapper_primary_range(source_id, 94, 95),
            source_ordinal: 0,
            context: BindingContextId::new(2),
            recovery: SourcePrimaryTermRecovery::Normal,
            spelling: "x".to_owned(),
            kind: SourcePrimaryTermKind::VariableReference,
            role: SourcePrimaryTermRole::Value,
            parent: None,
        }],
        references: vec![SourcePrimaryTermReferenceInput {
            term: SourcePrimaryTermId::new(0),
            binding: BindingId::new(0),
            role: SourcePrimaryTermReferenceRole::Variable,
        }],
        numeric_type_requests: Vec::new(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SourcePrimaryTermBindingProfile {
    Generic,
    ProofLocalGivenUse,
    ProofLocalGivenConditionUse,
    ProofLocalGivenDescendantUse,
    NestedFraenkelMapperPrimary,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SourcePrimaryTermProducer;

impl SourcePrimaryTermProducer {
    pub fn build(
        input: SourcePrimaryTermHandoffInput,
        binding_env: &BindingEnv,
        arena: &TypedArena,
    ) -> Result<SourcePrimaryTermHandoff, SourcePrimaryTermError> {
        Self::build_with_profile(
            input,
            binding_env,
            arena,
            SourcePrimaryTermBindingProfile::Generic,
        )
    }

    fn build_with_profile(
        input: SourcePrimaryTermHandoffInput,
        binding_env: &BindingEnv,
        arena: &TypedArena,
        binding_profile: SourcePrimaryTermBindingProfile,
    ) -> Result<SourcePrimaryTermHandoff, SourcePrimaryTermError> {
        if input.source_id != binding_env.source_id() || &input.module_id != binding_env.module_id()
        {
            return Err(SourcePrimaryTermError::InvalidTransaction);
        }

        let terms = validate_terms(input.source_id, &input.terms, binding_env, arena)?;
        let derived_ordinals = derive_reference_ordinals(
            input.source_id,
            &input.references,
            &terms,
            binding_env,
            binding_profile,
        )?;
        let references = validate_references(
            &input.references,
            &derived_ordinals,
            &terms,
            binding_env,
            binding_profile,
        )?;
        validate_reference_cardinality(&terms, &references)?;
        let numeric_type_requests =
            validate_numeric_requests(&input.numeric_type_requests, &terms)?;

        Ok(SourcePrimaryTermHandoff {
            source_id: input.source_id,
            module_id: input.module_id,
            terms: SourcePrimaryTermTable { rows: terms },
            references: SourcePrimaryTermReferenceTable { rows: references },
            numeric_type_requests: SourceNumericTypeRequestTable {
                rows: numeric_type_requests,
            },
        })
    }
}

fn validate_terms(
    source_id: SourceId,
    inputs: &[SourcePrimaryTermInput],
    binding_env: &BindingEnv,
    arena: &TypedArena,
) -> Result<Vec<SourcePrimaryTerm>, SourcePrimaryTermError> {
    let mut rows: Vec<SourcePrimaryTerm> = Vec::with_capacity(inputs.len());
    let mut child_by_parent = vec![None; inputs.len()];
    let mut seen_sites = std::collections::BTreeSet::new();

    for (index, input) in inputs.iter().enumerate() {
        let id = SourcePrimaryTermId::new(index);
        if input.source_ordinal != index
            || !valid_range(source_id, input.source_range)
            || !seen_sites.insert(input.site.node())
            || binding_env.contexts().get(input.context).is_none()
            || !valid_term_role(input.kind, input.role)
            || !valid_leaf_spelling(input.kind, &input.spelling)
        {
            return Err(SourcePrimaryTermError::InvalidTerm { term: id });
        }

        let row = SourcePrimaryTerm {
            site: input.site.clone(),
            source_range: input.source_range,
            source_ordinal: input.source_ordinal,
            context: input.context,
            recovery: input.recovery,
            spelling: input.spelling.clone(),
            kind: input.kind,
            role: input.role,
            parent: input.parent,
        };
        validate_term_node(&row, arena)
            .map_err(|()| SourcePrimaryTermError::InvalidTerm { term: id })?;

        if let Some(parent) = input.parent {
            let Some(parent_row) = rows.get(parent.index()) else {
                return Err(SourcePrimaryTermError::InvalidTerm { term: id });
            };
            if parent.index() >= index
                || parent_row.kind != SourcePrimaryTermKind::Parenthesized
                || parent_row.context != input.context
                || !strictly_contains(parent_row.source_range, input.source_range)
                || child_by_parent[parent.index()].replace(index).is_some()
            {
                return Err(SourcePrimaryTermError::InvalidTerm { term: id });
            }
        }
        rows.push(row);
    }

    for (index, row) in rows.iter().enumerate() {
        match (row.kind, child_by_parent[index]) {
            (SourcePrimaryTermKind::Parenthesized, Some(child))
                if row.spelling != format!("( {} )", rows[child].spelling) =>
            {
                return Err(SourcePrimaryTermError::InvalidTerm {
                    term: SourcePrimaryTermId::new(index),
                });
            }
            (SourcePrimaryTermKind::Parenthesized, Some(_)) => {}
            (SourcePrimaryTermKind::Parenthesized, None) | (_, Some(_)) => {
                return Err(SourcePrimaryTermError::InvalidTerm {
                    term: SourcePrimaryTermId::new(index),
                });
            }
            _ => {}
        }
    }

    for left in 0..rows.len() {
        for right in left + 1..rows.len() {
            let left_range = rows[left].source_range;
            let right_range = rows[right].source_range;
            if left_range.end <= right_range.start {
                continue;
            }
            if strictly_contains(left_range, right_range)
                && is_ancestor(SourcePrimaryTermId::new(left), &rows[right], &rows)
            {
                continue;
            }
            return Err(SourcePrimaryTermError::InvalidTerm {
                term: SourcePrimaryTermId::new(right),
            });
        }
    }

    Ok(rows)
}

fn validate_references(
    inputs: &[SourcePrimaryTermReferenceInput],
    ordinals: &[usize],
    terms: &[SourcePrimaryTerm],
    binding_env: &BindingEnv,
    binding_profile: SourcePrimaryTermBindingProfile,
) -> Result<Vec<SourcePrimaryTermReference>, SourcePrimaryTermError> {
    let mut rows = Vec::with_capacity(inputs.len());
    for (index, (input, use_ordinal)) in inputs.iter().zip(ordinals).enumerate() {
        let id = SourcePrimaryTermReferenceId::new(index);
        let Some(term) = terms.get(input.term.index()) else {
            return Err(SourcePrimaryTermError::InvalidReference { reference: id });
        };
        if !valid_reference_role(term.kind, input.role) {
            return Err(SourcePrimaryTermError::InvalidReference { reference: id });
        }
        let Some(context) = binding_env.contexts().get(term.context) else {
            return Err(SourcePrimaryTermError::InvalidReference { reference: id });
        };
        let lexical_scope = context.lexical_scope.clone();
        let lookup = BindingLookupSite::new(
            term.spelling.clone(),
            term.context,
            lexical_scope.clone(),
            *use_ordinal,
        );
        let Ok(BindingLookupResult::Local(binding)) = binding_env.lookup(&lookup) else {
            return Err(SourcePrimaryTermError::InvalidReference { reference: id });
        };
        let Some(binding_row) = binding_env.bindings().get(input.binding) else {
            return Err(SourcePrimaryTermError::InvalidReference { reference: id });
        };
        if binding != input.binding
            || binding_row.spelling != term.spelling
            || (binding_row.declaration_range.end > term.source_range.start
                && binding_profile != SourcePrimaryTermBindingProfile::NestedFraenkelMapperPrimary)
            || !valid_binding_role(binding_row.kind, input.role, binding_profile)
        {
            return Err(SourcePrimaryTermError::InvalidReference { reference: id });
        }
        rows.push(SourcePrimaryTermReference {
            term: input.term,
            binding: input.binding,
            role: input.role,
            lexical_scope,
            use_ordinal: *use_ordinal,
        });
    }
    Ok(rows)
}

fn validate_reference_cardinality(
    terms: &[SourcePrimaryTerm],
    references: &[SourcePrimaryTermReference],
) -> Result<(), SourcePrimaryTermError> {
    let mut counts = vec![0usize; terms.len()];
    for (index, reference) in references.iter().enumerate() {
        let Some(count) = counts.get_mut(reference.term.index()) else {
            return Err(SourcePrimaryTermError::InvalidReference {
                reference: SourcePrimaryTermReferenceId::new(index),
            });
        };
        *count += 1;
        if *count > 1 {
            return Err(SourcePrimaryTermError::InvalidReference {
                reference: SourcePrimaryTermReferenceId::new(index),
            });
        }
    }
    for (index, term) in terms.iter().enumerate() {
        let expected = usize::from(matches!(
            term.kind,
            SourcePrimaryTermKind::VariableReference | SourcePrimaryTermKind::ConstantReference
        ));
        if counts[index] != expected {
            return Err(SourcePrimaryTermError::InvalidTerm {
                term: SourcePrimaryTermId::new(index),
            });
        }
    }
    Ok(())
}

fn validate_numeric_requests(
    inputs: &[SourceNumericTypeRequestInput],
    terms: &[SourcePrimaryTerm],
) -> Result<Vec<SourceNumericTypeRequest>, SourcePrimaryTermError> {
    let mut rows = Vec::with_capacity(inputs.len());
    let mut counts = vec![0usize; terms.len()];
    let mut previous_term = None;
    for (index, input) in inputs.iter().enumerate() {
        let id = SourceNumericTypeRequestId::new(index);
        let Some(term) = terms.get(input.term.index()) else {
            return Err(SourcePrimaryTermError::InvalidNumericTypeRequest { request: id });
        };
        if term.kind != SourcePrimaryTermKind::Numeral
            || input.request_ordinal != index
            || input.owner != term.site
            || input.source_range != term.source_range
            || input.spelling != term.spelling
            || previous_term.is_some_and(|previous| previous >= input.term.index())
        {
            return Err(SourcePrimaryTermError::InvalidNumericTypeRequest { request: id });
        }
        previous_term = Some(input.term.index());
        counts[input.term.index()] += 1;
        if counts[input.term.index()] > 1 {
            return Err(SourcePrimaryTermError::InvalidNumericTypeRequest { request: id });
        }
        rows.push(SourceNumericTypeRequest {
            term: input.term,
            owner: input.owner.clone(),
            source_range: input.source_range,
            spelling: input.spelling.clone(),
            request_ordinal: input.request_ordinal,
        });
    }
    for (index, term) in terms.iter().enumerate() {
        let expected = usize::from(term.kind == SourcePrimaryTermKind::Numeral);
        if counts[index] != expected {
            return Err(SourcePrimaryTermError::InvalidTerm {
                term: SourcePrimaryTermId::new(index),
            });
        }
    }
    Ok(rows)
}

fn derive_reference_ordinals(
    source_id: SourceId,
    references: &[SourcePrimaryTermReferenceInput],
    terms: &[SourcePrimaryTerm],
    binding_env: &BindingEnv,
    binding_profile: SourcePrimaryTermBindingProfile,
) -> Result<Vec<usize>, SourcePrimaryTermError> {
    if binding_profile == SourcePrimaryTermBindingProfile::NestedFraenkelMapperPrimary {
        return (references.len() == 1 && terms.len() == 1)
            .then_some(vec![1])
            .ok_or(SourcePrimaryTermError::InvalidReference {
                reference: SourcePrimaryTermReferenceId::new(0),
            });
    }
    let bindings = binding_env.bindings().iter().collect::<Vec<_>>();
    let mut group_start = 0;
    while group_start < bindings.len() {
        let (_, first) = bindings[group_start];
        if !valid_range(source_id, first.declaration_range)
            || group_start > 0
                && bindings[group_start - 1].1.declaration_range.end > first.declaration_range.start
        {
            return Err(SourcePrimaryTermError::InvalidBindingEvent { event: group_start });
        }

        let mut group_end = group_start + 1;
        while group_end < bindings.len()
            && bindings[group_end].1.declaration_range == first.declaration_range
        {
            group_end += 1;
        }
        let final_index = group_end - 1;
        if group_end - group_start == 1 {
            if first.visible_after_ordinal != group_start {
                return Err(SourcePrimaryTermError::InvalidBindingEvent { event: group_start });
            }
        } else {
            for (index, (_, binding)) in bindings[group_start..group_end].iter().enumerate() {
                let event = group_start + index;
                if binding.spelling != first.spelling
                    || binding.kind != first.kind
                    || binding.owner_context != first.owner_context
                    || binding.identity != first.identity
                    || binding.visible_after_ordinal != final_index
                {
                    return Err(SourcePrimaryTermError::InvalidBindingEvent { event });
                }
            }
        }
        group_start = group_end;
    }

    let mut reference_ranges: Vec<SourceRange> = Vec::with_capacity(references.len());
    let mut previous_term = None;
    for (index, reference) in references.iter().enumerate() {
        let Some(term) = terms.get(reference.term.index()) else {
            return Err(SourcePrimaryTermError::InvalidReference {
                reference: SourcePrimaryTermReferenceId::new(index),
            });
        };
        if previous_term.is_some_and(|previous| previous >= reference.term.index())
            || index > 0 && reference_ranges[index - 1].end > term.source_range.start
        {
            return Err(SourcePrimaryTermError::InvalidReference {
                reference: SourcePrimaryTermReferenceId::new(index),
            });
        }
        previous_term = Some(reference.term.index());
        reference_ranges.push(term.source_range);
    }

    let mut binding_index = 0;
    let mut ordinals = Vec::with_capacity(reference_ranges.len());
    for reference_range in reference_ranges {
        while let Some((_, binding)) = bindings.get(binding_index) {
            if binding.declaration_range.end <= reference_range.start {
                binding_index += 1;
            } else {
                break;
            }
        }
        if bindings
            .get(binding_index)
            .is_some_and(|(_, binding)| binding.declaration_range.start < reference_range.end)
        {
            return Err(SourcePrimaryTermError::InvalidBindingEvent {
                event: binding_index,
            });
        }
        ordinals.push(binding_index);
    }
    Ok(ordinals)
}

fn validate_term_node(term: &SourcePrimaryTerm, arena: &TypedArena) -> Result<(), ()> {
    let TypedSiteRef::Node(node_id) = &term.site else {
        return Err(());
    };
    let Some(node) = arena.node(*node_id) else {
        return Err(());
    };
    if node.anchor != SourceAnchor::Range(term.source_range)
        || node.kind.as_str() != typed_kind_key(term.kind)
        || !recovery_matches(term.recovery, node.recovery)
    {
        return Err(());
    }
    Ok(())
}

fn valid_term_role(kind: SourcePrimaryTermKind, role: SourcePrimaryTermRole) -> bool {
    match kind {
        SourcePrimaryTermKind::It => role == SourcePrimaryTermRole::CurrentDefinitionResult,
        _ => role == SourcePrimaryTermRole::Value,
    }
}

fn valid_reference_role(kind: SourcePrimaryTermKind, role: SourcePrimaryTermReferenceRole) -> bool {
    matches!(
        (kind, role),
        (
            SourcePrimaryTermKind::VariableReference,
            SourcePrimaryTermReferenceRole::Variable
        ) | (
            SourcePrimaryTermKind::ConstantReference,
            SourcePrimaryTermReferenceRole::LocalConstant
        )
    )
}

fn valid_binding_role(
    kind: BindingKind,
    role: SourcePrimaryTermReferenceRole,
    binding_profile: SourcePrimaryTermBindingProfile,
) -> bool {
    match role {
        SourcePrimaryTermReferenceRole::Variable => {
            matches!(
                kind,
                BindingKind::ReservedVariable
                    | BindingKind::LetBinding
                    | BindingKind::QuantifierBinder
                    | BindingKind::DefinitionParameter
            ) || matches!(
                binding_profile,
                SourcePrimaryTermBindingProfile::ProofLocalGivenUse
                    | SourcePrimaryTermBindingProfile::ProofLocalGivenConditionUse
                    | SourcePrimaryTermBindingProfile::ProofLocalGivenDescendantUse
            ) && kind == BindingKind::GivenWitness
        }
        SourcePrimaryTermReferenceRole::LocalConstant => kind == BindingKind::LocalAbbreviation,
    }
}

fn task269gu_dependency_arena(
    arena: &TypedArena,
) -> Result<TypedArena, SourceProofLocalGivenUseTermError> {
    let nodes = (0..3)
        .map(|index| {
            arena
                .node(TypedNodeId::new(index))
                .cloned()
                .ok_or(SourceProofLocalGivenUseTermError::InvalidDependency)
        })
        .collect::<Result<Vec<_>, _>>()?;
    TypedArena::try_new(Some(TypedNodeId::new(2)), nodes)
        .map_err(|_| SourceProofLocalGivenUseTermError::InvalidDependency)
}

fn task269gu_input(source_id: SourceId, module_id: ModuleId) -> SourcePrimaryTermHandoffInput {
    SourcePrimaryTermHandoffInput {
        source_id,
        module_id,
        terms: [(3, 116, 117), (4, 120, 121)]
            .into_iter()
            .enumerate()
            .map(
                |(source_ordinal, (node, start, end))| SourcePrimaryTermInput {
                    site: TypedSiteRef::Node(TypedNodeId::new(node)),
                    source_range: task269gu_range(source_id, start, end),
                    source_ordinal,
                    context: crate::binding_env::BindingContextId::new(1),
                    recovery: SourcePrimaryTermRecovery::Normal,
                    spelling: "y".to_owned(),
                    kind: SourcePrimaryTermKind::VariableReference,
                    role: SourcePrimaryTermRole::Value,
                    parent: None,
                },
            )
            .collect(),
        references: (0..2)
            .map(|index| SourcePrimaryTermReferenceInput {
                term: SourcePrimaryTermId::new(index),
                binding: BindingId::new(1),
                role: SourcePrimaryTermReferenceRole::Variable,
            })
            .collect(),
        numeric_type_requests: Vec::new(),
    }
}

fn exact_task269gu_input(input: &SourcePrimaryTermHandoffInput) -> bool {
    input == &task269gu_input(input.source_id, input.module_id.clone())
}

fn exact_task269gu_arena(source_id: SourceId, arena: &TypedArena) -> bool {
    if arena.len() != 6 || arena.root() != Some(TypedNodeId::new(5)) {
        return false;
    }
    let expected = [
        (
            "source.proof-local.given-use.reserve-type",
            14,
            17,
            Vec::new(),
        ),
        ("source.proof-local.given-use.type", 84, 87, Vec::new()),
        (
            "source.proof-local.given-use.type-root",
            0,
            127,
            vec![TypedNodeId::new(0), TypedNodeId::new(1)],
        ),
        ("source.term.variable-reference", 116, 117, Vec::new()),
        ("source.term.variable-reference", 120, 121, Vec::new()),
        (
            "source.proof-local.given-use.term-root",
            0,
            127,
            vec![
                TypedNodeId::new(2),
                TypedNodeId::new(3),
                TypedNodeId::new(4),
            ],
        ),
    ];
    expected
        .into_iter()
        .enumerate()
        .all(|(index, (kind, start, end, children))| {
            arena.node(TypedNodeId::new(index)).is_some_and(|node| {
                node.kind.as_str() == kind
                    && node.resolved_node.is_none()
                    && node.anchor == SourceAnchor::Range(task269gu_range(source_id, start, end))
                    && node.children == children
                    && node.typing == TypingState::Unknown
                    && node.recovery == NodeRecoveryState::Normal
                    && node.links == Default::default()
            })
        })
}

fn task269gu_range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
    SourceRange {
        source_id,
        start,
        end,
    }
}

fn task269sdu_dependency_arena(
    arena: &TypedArena,
) -> Result<TypedArena, SourceProofLocalGivenDescendantUseTermError> {
    let nodes = (0..3)
        .map(|index| {
            arena
                .node(TypedNodeId::new(index))
                .cloned()
                .ok_or(SourceProofLocalGivenDescendantUseTermError::InvalidDependency)
        })
        .collect::<Result<Vec<_>, _>>()?;
    TypedArena::try_new(Some(TypedNodeId::new(2)), nodes)
        .map_err(|_| SourceProofLocalGivenDescendantUseTermError::InvalidDependency)
}

fn task269sdu_input(source_id: SourceId, module_id: ModuleId) -> SourcePrimaryTermHandoffInput {
    SourcePrimaryTermHandoffInput {
        source_id,
        module_id,
        terms: vec![SourcePrimaryTermInput {
            site: TypedSiteRef::Node(TypedNodeId::new(3)),
            source_range: task269sdu_range(source_id, 118, 119),
            source_ordinal: 0,
            context: crate::binding_env::BindingContextId::new(2),
            recovery: SourcePrimaryTermRecovery::Normal,
            spelling: "y".to_owned(),
            kind: SourcePrimaryTermKind::VariableReference,
            role: SourcePrimaryTermRole::Value,
            parent: None,
        }],
        references: vec![SourcePrimaryTermReferenceInput {
            term: SourcePrimaryTermId::new(0),
            binding: BindingId::new(1),
            role: SourcePrimaryTermReferenceRole::Variable,
        }],
        numeric_type_requests: Vec::new(),
    }
}

fn exact_task269sdu_input(input: &SourcePrimaryTermHandoffInput) -> bool {
    input == &task269sdu_input(input.source_id, input.module_id.clone())
}

fn exact_task269sdu_arena(source_id: SourceId, arena: &TypedArena) -> bool {
    if arena.len() != 5 || arena.root() != Some(TypedNodeId::new(4)) {
        return false;
    }
    let expected = [
        (
            "source.proof-local.given-descendant.reserve-type",
            14,
            17,
            vec![],
        ),
        ("source.proof-local.given-descendant.type", 95, 98, vec![]),
        (
            "source.proof-local.given-descendant.type-root",
            0,
            179,
            vec![TypedNodeId::new(0), TypedNodeId::new(1)],
        ),
        ("source.term.variable-reference", 118, 119, vec![]),
        (
            "source.proof-local.given-descendant-use.term-root",
            0,
            179,
            vec![TypedNodeId::new(2), TypedNodeId::new(3)],
        ),
    ];
    expected
        .into_iter()
        .enumerate()
        .all(|(index, (kind, start, end, children))| {
            arena.node(TypedNodeId::new(index)).is_some_and(|node| {
                node.kind.as_str() == kind
                    && node.resolved_node.is_none()
                    && node.anchor == SourceAnchor::Range(task269sdu_range(source_id, start, end))
                    && node.children == children
                    && node.typing == TypingState::Unknown
                    && node.recovery == NodeRecoveryState::Normal
                    && node.links == Default::default()
            })
        })
}

fn task269sdu_range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
    SourceRange {
        source_id,
        start,
        end,
    }
}

fn task269gcu_dependency_arena(
    arena: &TypedArena,
) -> Result<TypedArena, SourceProofLocalGivenConditionUseTermError> {
    let nodes = (0..3)
        .map(|index| {
            arena
                .node(TypedNodeId::new(index))
                .cloned()
                .ok_or(SourceProofLocalGivenConditionUseTermError::InvalidDependency)
        })
        .collect::<Result<Vec<_>, _>>()?;
    TypedArena::try_new(Some(TypedNodeId::new(2)), nodes)
        .map_err(|_| SourceProofLocalGivenConditionUseTermError::InvalidDependency)
}

fn task269gcu_input(source_id: SourceId, module_id: ModuleId) -> SourcePrimaryTermHandoffInput {
    SourcePrimaryTermHandoffInput {
        source_id,
        module_id,
        terms: [(3, 107, 108), (4, 111, 112)]
            .into_iter()
            .enumerate()
            .map(
                |(source_ordinal, (node, start, end))| SourcePrimaryTermInput {
                    site: TypedSiteRef::Node(TypedNodeId::new(node)),
                    source_range: task269gcu_range(source_id, start, end),
                    source_ordinal,
                    context: crate::binding_env::BindingContextId::new(1),
                    recovery: SourcePrimaryTermRecovery::Normal,
                    spelling: "y".to_owned(),
                    kind: SourcePrimaryTermKind::VariableReference,
                    role: SourcePrimaryTermRole::Value,
                    parent: None,
                },
            )
            .collect(),
        references: (0..2)
            .map(|index| SourcePrimaryTermReferenceInput {
                term: SourcePrimaryTermId::new(index),
                binding: BindingId::new(1),
                role: SourcePrimaryTermReferenceRole::Variable,
            })
            .collect(),
        numeric_type_requests: Vec::new(),
    }
}

fn exact_task269gcu_input(input: &SourcePrimaryTermHandoffInput) -> bool {
    input == &task269gcu_input(input.source_id, input.module_id.clone())
}

fn exact_task269gcu_arena(source_id: SourceId, arena: &TypedArena) -> bool {
    if arena.len() != 6 || arena.root() != Some(TypedNodeId::new(5)) {
        return false;
    }
    let expected = [
        (
            "source.proof-local.given-condition.reserve-type",
            14,
            17,
            Vec::new(),
        ),
        (
            "source.proof-local.given-condition.type",
            90,
            93,
            Vec::new(),
        ),
        (
            "source.proof-local.given-condition.type-root",
            0,
            133,
            vec![TypedNodeId::new(0), TypedNodeId::new(1)],
        ),
        ("source.term.variable-reference", 107, 108, Vec::new()),
        ("source.term.variable-reference", 111, 112, Vec::new()),
        (
            "source.proof-local.given-condition.term-root",
            0,
            133,
            vec![
                TypedNodeId::new(2),
                TypedNodeId::new(3),
                TypedNodeId::new(4),
            ],
        ),
    ];
    expected
        .into_iter()
        .enumerate()
        .all(|(index, (kind, start, end, children))| {
            arena.node(TypedNodeId::new(index)).is_some_and(|node| {
                node.kind.as_str() == kind
                    && node.resolved_node.is_none()
                    && node.anchor == SourceAnchor::Range(task269gcu_range(source_id, start, end))
                    && node.children == children
                    && node.typing == TypingState::Unknown
                    && node.recovery == NodeRecoveryState::Normal
                    && node.links == Default::default()
            })
        })
}

fn task269gcu_range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
    SourceRange {
        source_id,
        start,
        end,
    }
}

fn valid_leaf_spelling(kind: SourcePrimaryTermKind, spelling: &str) -> bool {
    match kind {
        SourcePrimaryTermKind::VariableReference | SourcePrimaryTermKind::ConstantReference => {
            is_identifier(spelling)
        }
        SourcePrimaryTermKind::It => spelling == "it",
        SourcePrimaryTermKind::Numeral => {
            !spelling.is_empty() && spelling.bytes().all(|byte| byte.is_ascii_digit())
        }
        SourcePrimaryTermKind::Parenthesized => !spelling.is_empty(),
    }
}

fn recovery_matches(recovery: SourcePrimaryTermRecovery, node_recovery: NodeRecoveryState) -> bool {
    match recovery {
        SourcePrimaryTermRecovery::Normal => node_recovery == NodeRecoveryState::Normal,
        SourcePrimaryTermRecovery::Degraded => matches!(
            node_recovery,
            NodeRecoveryState::Recovered | NodeRecoveryState::Degraded
        ),
    }
}

fn valid_range(source_id: SourceId, range: SourceRange) -> bool {
    range.source_id == source_id && range.start < range.end
}

fn strictly_contains(parent: SourceRange, child: SourceRange) -> bool {
    parent.source_id == child.source_id && parent.start < child.start && parent.end > child.end
}

fn is_ancestor(
    expected: SourcePrimaryTermId,
    row: &SourcePrimaryTerm,
    rows: &[SourcePrimaryTerm],
) -> bool {
    let mut cursor = row.parent;
    while let Some(parent) = cursor {
        if parent == expected {
            return true;
        }
        cursor = rows.get(parent.index()).and_then(|term| term.parent);
    }
    false
}

fn typed_kind_key(kind: SourcePrimaryTermKind) -> &'static str {
    match kind {
        SourcePrimaryTermKind::VariableReference => "source.term.variable-reference",
        SourcePrimaryTermKind::ConstantReference => "source.term.constant-reference",
        SourcePrimaryTermKind::It => "source.term.it",
        SourcePrimaryTermKind::Numeral => "source.term.numeral",
        SourcePrimaryTermKind::Parenthesized => "source.term.parenthesized",
    }
}

fn kind_key(kind: SourcePrimaryTermKind) -> &'static str {
    match kind {
        SourcePrimaryTermKind::VariableReference => "variable-reference",
        SourcePrimaryTermKind::ConstantReference => "constant-reference",
        SourcePrimaryTermKind::It => "it",
        SourcePrimaryTermKind::Numeral => "numeral",
        SourcePrimaryTermKind::Parenthesized => "parenthesized",
    }
}

fn role_key(role: SourcePrimaryTermRole) -> &'static str {
    match role {
        SourcePrimaryTermRole::Value => "value",
        SourcePrimaryTermRole::CurrentDefinitionResult => "current-definition-result",
    }
}

fn reference_role_key(role: SourcePrimaryTermReferenceRole) -> &'static str {
    match role {
        SourcePrimaryTermReferenceRole::Variable => "variable",
        SourcePrimaryTermReferenceRole::LocalConstant => "local-constant",
    }
}

fn recovery_key(recovery: SourcePrimaryTermRecovery) -> &'static str {
    match recovery {
        SourcePrimaryTermRecovery::Normal => "normal",
        SourcePrimaryTermRecovery::Degraded => "degraded",
    }
}

fn write_optional_term_id(output: &mut String, term: Option<SourcePrimaryTermId>) {
    if let Some(term) = term {
        let _ = write!(output, "{}", term.index());
    } else {
        output.push('-');
    }
}

fn write_scope(output: &mut String, scope: Option<&LocalTermScope>) {
    let Some(scope) = scope else {
        output.push('-');
        return;
    };
    output.push('[');
    for (index, segment) in scope.path().iter().enumerate() {
        if index > 0 {
            output.push('.');
        }
        let _ = write!(output, "{segment}");
    }
    output.push(']');
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        binding_env::{
            BinderIdentity, BindingContextDraft, BindingContextId, BindingContextLayer,
            BindingContextOwner, BindingContextRecovery, BindingContextTable,
            BindingDiagnosticTable, BindingDraft, BindingEnvParts, BindingRecoveryState,
            BindingStatus, BindingTable, BindingTypeSite, CapturedFreeVariables,
        },
        cluster_trace::ClusterFactTable,
        overload_resolution::{
            CandidateViabilityInput, CandidateViabilityOutput, OverloadCandidateInput,
            OverloadCollectionOutput, OverloadSelectionOutput, OverloadSiteInput,
            OverloadSiteResolutionInput, SpecificityComparisonInput, SpecificityGraphOutput,
            TemplateExpansionOutput,
        },
        resolved_typed_ast::{
            ExprId, ExpressionMetadataInput, ResolvedNodeKindHint, ResolvedNodeKindHintKind,
            ResolvedTypedAst, ResolvedTypedAstError, ResolvedTypedAstInputs, ResolvedTypedNodeKind,
            SourceNodeRole,
        },
        source_proof_local_declaration::{
            SourceProofLocalDeclarationHandoff, SourceProofLocalGivenBindingHandoff,
            SourceProofLocalGivenBindingHandoffInput, SourceProofLocalGivenBindingProducer,
            SourceProofLocalGivenBindingRecovery,
            SourceProofLocalGivenConditionBindingHandoffInput,
            SourceProofLocalGivenConditionBindingProducer,
            SourceProofLocalGivenDescendantBindingHandoff,
            SourceProofLocalGivenDescendantBindingHandoffInput,
            SourceProofLocalGivenDescendantBindingProducer,
            SourceProofLocalGivenUseBindingHandoffInput, SourceProofLocalGivenUseBindingProducer,
            SourceProofLocalLetBindingHandoff, SourceProofLocalLetBindingHandoffInput,
            SourceProofLocalLetBindingProducer, SourceProofLocalLetBindingRecovery,
        },
        source_type::{
            SourceProofLocalGivenConditionTypeProducer,
            SourceProofLocalGivenDescendantTypeProducer, SourceProofLocalGivenTypeHandoff,
            SourceProofLocalGivenTypeProducer, SourceProofLocalGivenUseTypeProducer,
            SourceProofLocalLetTypeHandoff, SourceProofLocalLetTypeProducer,
            SourceTypeApplicationForm, SourceTypeApplicationInput, SourceTypeExpressionId,
            SourceTypeExpressionInput, SourceTypeHandoffInput, SourceTypeHead,
        },
        typed_ast::{
            CoercionTable, InitialObligationTable, LocalTypeContextTable, TypeDiagnosticTable,
            TypeFactId, TypeFactTable, TypeRole, TypeTable, TypedArenaBuilder, TypedAst,
            TypedAstError, TypedAstParts, TypedNode, TypedNodeId,
        },
    };
    use mizar_resolve::{
        env::{
            ContributionKind, DefinitionIndex, DefinitionKind, DefinitionShell,
            SourceContributionIndex, SymbolEnv, SymbolEnvIndexes,
        },
        names::LocalTermBinding,
        resolved_ast::{FullyQualifiedName, LocalSymbolId, SemanticOrigin, SymbolId},
    };
    use mizar_session::{
        BuildSnapshotId, InMemorySessionIdAllocator, ModulePath, PackageId, SessionIdAllocator as _,
    };

    #[derive(Clone)]
    struct Fixture {
        source: SourceId,
        module: ModuleId,
        input: SourcePrimaryTermHandoffInput,
        bindings: BindingEnv,
        arena: TypedArena,
    }

    fn source_id() -> SourceId {
        source_id_for("b2")
    }

    fn other_source_id() -> SourceId {
        let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{}",
            "b2".repeat(32)
        ))
        .expect("snapshot");
        let allocator = InMemorySessionIdAllocator::new();
        allocator.next_source_id(snapshot).expect("first source");
        allocator.next_source_id(snapshot).expect("second source")
    }

    fn source_id_for(byte: &str) -> SourceId {
        let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{}",
            byte.repeat(32)
        ))
        .expect("snapshot");
        InMemorySessionIdAllocator::new()
            .next_source_id(snapshot)
            .expect("source")
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

    fn context(bindings: Vec<BindingId>, scope: Option<Vec<u32>>) -> BindingContextDraft {
        BindingContextDraft {
            owner: BindingContextOwner::Module,
            parent: None,
            layer: BindingContextLayer::Module,
            lexical_scope: scope.map(LocalTermScope::new),
            bindings: bindings.clone(),
            visible_bindings: bindings,
            recovery: BindingContextRecovery::Normal,
        }
    }

    fn reserved_binding(
        source: SourceId,
        spelling: &str,
        start: usize,
        visible_after_ordinal: usize,
    ) -> BindingDraft {
        let declaration_range = range(source, start, start + 1);
        BindingDraft {
            spelling: spelling.to_owned(),
            kind: BindingKind::ReservedVariable,
            identity: BinderIdentity::ReservedVariable {
                spelling: spelling.to_owned(),
                declaration_range,
            },
            owner_context: BindingContextId::new(0),
            declaration_range,
            visible_after_ordinal,
            type_site: BindingTypeSite::Missing,
            status: BindingStatus::Reserved,
            captured: CapturedFreeVariables::default(),
            diagnostics: Vec::new(),
            recovery: BindingRecoveryState::Normal,
        }
    }

    fn local_binding(
        source: SourceId,
        spelling: &str,
        start: usize,
        visible_after_ordinal: usize,
        kind: BindingKind,
    ) -> BindingDraft {
        let declaration_range = range(source, start, start + 1);
        BindingDraft {
            spelling: spelling.to_owned(),
            kind,
            identity: BinderIdentity::ResolverLocal {
                scope: LocalTermScope::new(vec![0]),
                ordinal: visible_after_ordinal,
                declaration_range,
            },
            owner_context: BindingContextId::new(0),
            declaration_range,
            visible_after_ordinal,
            type_site: BindingTypeSite::Missing,
            status: BindingStatus::Active,
            captured: CapturedFreeVariables::default(),
            diagnostics: Vec::new(),
            recovery: BindingRecoveryState::Normal,
        }
    }

    fn binding_env(
        source: SourceId,
        module: &ModuleId,
        drafts: Vec<BindingDraft>,
        scope: Option<Vec<u32>>,
    ) -> BindingEnv {
        let binding_ids = (0..drafts.len()).map(BindingId::new).collect::<Vec<_>>();
        let mut contexts = BindingContextTable::new();
        contexts.insert(context(binding_ids, scope));
        let mut bindings = BindingTable::new();
        for draft in drafts {
            bindings.insert(draft);
        }
        BindingEnv::try_new(BindingEnvParts {
            source_id: source,
            module_id: module.clone(),
            contexts,
            bindings,
            diagnostics: BindingDiagnosticTable::new(),
        })
        .expect("binding environment")
    }

    fn binding_env_with_child_context(
        source: SourceId,
        module: &ModuleId,
        drafts: Vec<BindingDraft>,
    ) -> BindingEnv {
        let binding_ids = (0..drafts.len()).map(BindingId::new).collect::<Vec<_>>();
        let mut contexts = BindingContextTable::new();
        contexts.insert(context(binding_ids.clone(), Some(vec![0])));
        contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::Generated("child".to_owned()),
            parent: Some(BindingContextId::new(0)),
            layer: BindingContextLayer::Expression,
            lexical_scope: Some(LocalTermScope::new(vec![0, 0])),
            bindings: Vec::new(),
            visible_bindings: binding_ids,
            recovery: BindingContextRecovery::Normal,
        });
        let mut bindings = BindingTable::new();
        for draft in drafts {
            bindings.insert(draft);
        }
        BindingEnv::try_new(BindingEnvParts {
            source_id: source,
            module_id: module.clone(),
            contexts,
            bindings,
            diagnostics: BindingDiagnosticTable::new(),
        })
        .expect("binding environment with child context")
    }

    fn binding_env_with_split_owners(
        source: SourceId,
        module: &ModuleId,
        mut drafts: Vec<BindingDraft>,
    ) -> BindingEnv {
        assert_eq!(drafts.len(), 2);
        drafts[0].owner_context = BindingContextId::new(0);
        drafts[1].owner_context = BindingContextId::new(1);
        let mut contexts = BindingContextTable::new();
        contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::Module,
            parent: None,
            layer: BindingContextLayer::Module,
            lexical_scope: Some(LocalTermScope::new(vec![0])),
            bindings: vec![BindingId::new(0)],
            visible_bindings: vec![BindingId::new(0)],
            recovery: BindingContextRecovery::Normal,
        });
        contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::Generated("child".to_owned()),
            parent: Some(BindingContextId::new(0)),
            layer: BindingContextLayer::Expression,
            lexical_scope: Some(LocalTermScope::new(vec![0, 0])),
            bindings: vec![BindingId::new(1)],
            visible_bindings: vec![BindingId::new(0), BindingId::new(1)],
            recovery: BindingContextRecovery::Normal,
        });
        let mut bindings = BindingTable::new();
        for draft in drafts {
            bindings.insert(draft);
        }
        BindingEnv::try_new(BindingEnvParts {
            source_id: source,
            module_id: module.clone(),
            contexts,
            bindings,
            diagnostics: BindingDiagnosticTable::new(),
        })
        .expect("binding environment with split owners")
    }

    fn term(
        source: SourceId,
        index: usize,
        start: usize,
        end: usize,
        spelling: &str,
        kind: SourcePrimaryTermKind,
        parent: Option<usize>,
    ) -> SourcePrimaryTermInput {
        let role = if kind == SourcePrimaryTermKind::It {
            SourcePrimaryTermRole::CurrentDefinitionResult
        } else {
            SourcePrimaryTermRole::Value
        };
        SourcePrimaryTermInput {
            site: node(index),
            source_range: range(source, start, end),
            source_ordinal: index,
            context: BindingContextId::new(0),
            recovery: SourcePrimaryTermRecovery::Normal,
            spelling: spelling.to_owned(),
            kind,
            role,
            parent: parent.map(SourcePrimaryTermId::new),
        }
    }

    fn arena_nodes_for(terms: &[SourcePrimaryTermInput]) -> Vec<TypedNode> {
        let mut nodes = vec![None; terms.len()];
        for term in terms {
            nodes[term.site.node().index()] = Some(
                TypedNode::new(
                    typed_kind_key(term.kind),
                    SourceAnchor::Range(term.source_range),
                )
                .with_recovery(match term.recovery {
                    SourcePrimaryTermRecovery::Normal => NodeRecoveryState::Normal,
                    SourcePrimaryTermRecovery::Degraded => NodeRecoveryState::Degraded,
                }),
            );
        }
        nodes
            .into_iter()
            .map(|node| node.expect("dense fixture nodes"))
            .collect()
    }

    fn arena_for(terms: &[SourcePrimaryTermInput]) -> TypedArena {
        TypedArena::try_new(None, arena_nodes_for(terms)).expect("typed arena")
    }

    fn fixture() -> Fixture {
        let source = source_id();
        let module = module("source.term");
        let terms = vec![
            term(
                source,
                0,
                10,
                15,
                "( x )",
                SourcePrimaryTermKind::Parenthesized,
                None,
            ),
            term(
                source,
                1,
                11,
                12,
                "x",
                SourcePrimaryTermKind::VariableReference,
                Some(0),
            ),
            term(
                source,
                2,
                20,
                21,
                "x",
                SourcePrimaryTermKind::VariableReference,
                None,
            ),
            term(source, 3, 30, 31, "1", SourcePrimaryTermKind::Numeral, None),
            term(source, 4, 40, 42, "it", SourcePrimaryTermKind::It, None),
            term(
                source,
                5,
                50,
                51,
                "c",
                SourcePrimaryTermKind::ConstantReference,
                None,
            ),
        ];
        let input = SourcePrimaryTermHandoffInput {
            source_id: source,
            module_id: module.clone(),
            terms,
            references: vec![
                SourcePrimaryTermReferenceInput {
                    term: SourcePrimaryTermId::new(1),
                    binding: BindingId::new(0),
                    role: SourcePrimaryTermReferenceRole::Variable,
                },
                SourcePrimaryTermReferenceInput {
                    term: SourcePrimaryTermId::new(2),
                    binding: BindingId::new(0),
                    role: SourcePrimaryTermReferenceRole::Variable,
                },
                SourcePrimaryTermReferenceInput {
                    term: SourcePrimaryTermId::new(5),
                    binding: BindingId::new(1),
                    role: SourcePrimaryTermReferenceRole::LocalConstant,
                },
            ],
            numeric_type_requests: vec![SourceNumericTypeRequestInput {
                term: SourcePrimaryTermId::new(3),
                owner: node(3),
                source_range: range(source, 30, 31),
                spelling: "1".to_owned(),
                request_ordinal: 0,
            }],
        };
        let bindings = binding_env(
            source,
            &module,
            vec![
                reserved_binding(source, "x", 0, 0),
                local_binding(source, "c", 2, 1, BindingKind::LocalAbbreviation),
            ],
            Some(vec![0]),
        );
        let arena = arena_for(&input.terms);
        Fixture {
            source,
            module,
            input,
            bindings,
            arena,
        }
    }

    fn build(fixture: &Fixture) -> Result<SourcePrimaryTermHandoff, SourcePrimaryTermError> {
        SourcePrimaryTermProducer::build(fixture.input.clone(), &fixture.bindings, &fixture.arena)
    }

    fn replace_bindings(fixture: &mut Fixture, drafts: Vec<BindingDraft>, scope: Option<Vec<u32>>) {
        fixture.bindings = binding_env(fixture.source, &fixture.module, drafts, scope);
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

    #[derive(Clone)]
    struct Task269guFixture {
        source: SourceId,
        module: ModuleId,
        dependency: SourceProofLocalGivenUseTypeHandoff,
        input: SourcePrimaryTermHandoffInput,
        arena: TypedArena,
    }

    #[derive(Debug, Clone, Copy)]
    enum Task269guInputMutation {
        TermCount,
        TermSite,
        TermRange,
        TermOrdinal,
        TermContext,
        TermRecovery,
        TermSpelling,
        TermKind,
        TermRole,
        TermParent,
        ReferenceCount,
        ReferenceTerm,
        ReferenceBinding,
        ReferenceRole,
        NumericRequest,
    }

    #[derive(Debug, Clone, Copy)]
    enum Task269guArenaMutation {
        Kind,
        ResolvedNode,
        Anchor,
        Children,
        Typing,
        Recovery,
        Links,
    }

    fn task269gu_fixture() -> Task269guFixture {
        let source = source_id_for("f7");
        let module = module("task269gup");
        let local = concat!(
            "contribution=0:namespace=task269gup:owner=theorem#1:shell=theorem:",
            "kind=theorem:name=FormulaStatementGivenSmoke:notation=_:arity=_:",
            "definition=theorem:registration=_:policy=non-overloadable:",
            "slot=non-overloadable:_:theorem:_"
        );
        let theorem_symbol = SymbolId::new(
            module.clone(),
            LocalSymbolId::new(local),
            FullyQualifiedName::new(format!("pkg::task269gup::{local}")),
        );
        let mut contributions = SourceContributionIndex::new();
        let contribution = contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 18)),
        );
        let mut definitions = DefinitionIndex::new();
        let theorem_definition = definitions.insert(DefinitionShell::new(
            theorem_symbol.clone(),
            DefinitionKind::Theorem,
            SemanticOrigin::new(
                source,
                module.clone(),
                SourceAnchor::Range(range(source, 19, 127)),
                vec![2, 1],
            ),
            contribution,
        ));
        let lower_fingerprint = format!(
            concat!(
                "source-proof-local-given-use-lower-debug-v1\n",
                "module: {}::{}\n",
                "source-fingerprint: \"ec15ded78ae96022840a8419a85d74643de3b37337e9a202cbda77ee97aa7c01\"\n",
                "surface-fingerprint: \"c64297ce72e380a2e4146276966e085d780f8b38f2528d5abaa440a50c67db6d\"\n",
                "theorem symbol={:?} definition=0 contribution=0 range=19..127 proof=62..126\n",
                "given range=70..108 segment=76..87 source_ordinal=1\n",
                "name range=76..77 spelling=\"y\"\n",
                "type range=84..87 head=84..87 spelling=\"set\" form=bare\n",
            ),
            module.package().as_str(),
            module.path().as_str(),
            theorem_symbol.fqn().as_str(),
        );
        let binding = SourceProofLocalGivenUseBindingProducer::build(
            SourceProofLocalGivenUseBindingHandoffInput {
                source_id: source,
                module_id: module.clone(),
                lower_fingerprint,
                theorem_symbol,
                theorem_definition,
                contribution,
                theorem_range: range(source, 19, 127),
                proof_range: range(source, 62, 126),
                given_range: range(source, 70, 108),
                segment_range: range(source, 76, 87),
                name_range: range(source, 76, 77),
                source_ordinal: 1,
                local: LocalTermBinding::new(
                    "y",
                    LocalTermScope::new(vec![0]),
                    range(source, 76, 77),
                    1,
                ),
                recovery: SourceProofLocalGivenBindingRecovery::Normal,
            },
            &task269gu_base_binding_env(source, module.clone()),
        )
        .expect("Task269GU exact GUP dependency");
        let symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        let dependency = SourceProofLocalGivenUseTypeProducer::build(
            binding,
            task269gupt_type_input(source, module.clone()),
            &symbols,
            &task269gupt_arena(source),
        )
        .expect("Task269GU exact GUPT dependency");
        Task269guFixture {
            source,
            module: module.clone(),
            dependency,
            input: task269gu_input(source, module),
            arena: task269gu_arena(source, 5, false),
        }
    }

    fn task269gu_base_binding_env(source: SourceId, module: ModuleId) -> BindingEnv {
        let mut bindings = BindingTable::new();
        let binding = bindings.insert(BindingDraft {
            spelling: "x".to_owned(),
            kind: BindingKind::ReservedVariable,
            identity: BinderIdentity::ReservedVariable {
                spelling: "x".to_owned(),
                declaration_range: range(source, 8, 9),
            },
            owner_context: BindingContextId::new(0),
            declaration_range: range(source, 8, 9),
            visible_after_ordinal: 0,
            type_site: BindingTypeSite::Source(range(source, 14, 17)),
            status: BindingStatus::Reserved,
            captured: CapturedFreeVariables::default(),
            diagnostics: Vec::new(),
            recovery: BindingRecoveryState::Normal,
        });
        let mut contexts = BindingContextTable::new();
        contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::Module,
            parent: None,
            layer: BindingContextLayer::Module,
            lexical_scope: None,
            bindings: vec![binding],
            visible_bindings: vec![binding],
            recovery: BindingContextRecovery::Normal,
        });
        BindingEnv::try_new(BindingEnvParts {
            source_id: source,
            module_id: module,
            contexts,
            bindings,
            diagnostics: BindingDiagnosticTable::new(),
        })
        .expect("Task269GU base binding environment")
    }

    fn task269gu_let_neighbor() -> SourceProofLocalLetBindingHandoff {
        let source = source_id_for("f7");
        let module = module("task269gup");
        let local = concat!(
            "contribution=0:namespace=task269gup:owner=theorem#1:shell=theorem:",
            "kind=theorem:name=FormulaStatementLetSmoke:notation=_:arity=_:",
            "definition=theorem:registration=_:policy=non-overloadable:",
            "slot=non-overloadable:_:theorem:_"
        );
        let theorem_symbol = SymbolId::new(
            module.clone(),
            LocalSymbolId::new(local),
            FullyQualifiedName::new(format!("pkg::task269gup::{local}")),
        );
        let mut contributions = SourceContributionIndex::new();
        let contribution = contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 18)),
        );
        let mut definitions = DefinitionIndex::new();
        let theorem_definition = definitions.insert(DefinitionShell::new(
            theorem_symbol.clone(),
            DefinitionKind::Theorem,
            SemanticOrigin::new(
                source,
                module.clone(),
                SourceAnchor::Range(range(source, 19, 99)),
                vec![2, 1],
            ),
            contribution,
        ));
        let lower_fingerprint = format!(
            concat!(
                "source-proof-local-let-lower-debug-v1\n",
                "module: {}::{}\n",
                "source-fingerprint: \"7860a3fe5af89063ac6a2b9a4465cac36d26f6d64e892ba6e2c89bcbaaf9763a\"\n",
                "surface-fingerprint: \"1fc35ec18db82efc0968b2f42b08cfaae678184983210cd26f060d45354c7f68\"\n",
                "theorem symbol={:?} definition=0 contribution=0 range=19..99 proof=59..98\n",
                "let range=67..80 segment=71..79 source_ordinal=1\n",
                "name range=71..72 spelling=\"y\" scope=[0] visible_after=1\n",
                "type range=76..79 head=76..79 spelling=\"set\" form=bare\n",
            ),
            module.package().as_str(),
            module.path().as_str(),
            theorem_symbol.fqn().as_str(),
        );
        SourceProofLocalLetBindingProducer::build(
            SourceProofLocalLetBindingHandoffInput {
                source_id: source,
                module_id: module.clone(),
                lower_fingerprint,
                theorem_symbol,
                theorem_definition,
                contribution,
                theorem_range: range(source, 19, 99),
                proof_range: range(source, 59, 98),
                let_range: range(source, 67, 80),
                segment_range: range(source, 71, 79),
                name_range: range(source, 71, 72),
                source_ordinal: 1,
                local: LocalTermBinding::new(
                    "y",
                    LocalTermScope::new(vec![0]),
                    range(source, 71, 72),
                    1,
                ),
                recovery: SourceProofLocalLetBindingRecovery::Normal,
            },
            &task269gu_base_binding_env(source, module),
        )
        .expect("Task269GU same-identity Let neighbor")
    }

    fn task269gu_given_neighbor() -> SourceProofLocalGivenBindingHandoff {
        let source = source_id_for("f7");
        let module = module("task269gup");
        let local = concat!(
            "contribution=0:namespace=task269gup:owner=theorem#1:shell=theorem:",
            "kind=theorem:name=FormulaStatementGivenSmoke:notation=_:arity=_:",
            "definition=theorem:registration=_:policy=non-overloadable:",
            "slot=non-overloadable:_:theorem:_"
        );
        let theorem_symbol = SymbolId::new(
            module.clone(),
            LocalSymbolId::new(local),
            FullyQualifiedName::new(format!("pkg::task269gup::{local}")),
        );
        let mut contributions = SourceContributionIndex::new();
        let contribution = contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 18)),
        );
        let mut definitions = DefinitionIndex::new();
        let theorem_definition = definitions.insert(DefinitionShell::new(
            theorem_symbol.clone(),
            DefinitionKind::Theorem,
            SemanticOrigin::new(
                source,
                module.clone(),
                SourceAnchor::Range(range(source, 19, 128)),
                vec![2, 1],
            ),
            contribution,
        ));
        let lower_fingerprint = format!(
            concat!(
                "source-proof-local-given-lower-debug-v1\n",
                "module: {}::{}\n",
                "source-fingerprint: \"04e54b8ada9af54fde9f937e1bb0f96bd8cf85002b2b57f4d348b11c8eb72a2f\"\n",
                "surface-fingerprint: \"58ac16a3c75860180a8bec5dc8e87ec8b269fe75715a6d8363f7ef064e3deea8\"\n",
                "theorem symbol={:?} definition=0 contribution=0 range=19..128 proof=62..127\n",
                "given range=70..108 segment=76..87 source_ordinal=1\n",
                "name range=76..77 spelling=\"y\"\n",
                "type range=84..87 head=84..87 spelling=\"set\" form=bare\n",
            ),
            module.package().as_str(),
            module.path().as_str(),
            theorem_symbol.fqn().as_str(),
        );
        SourceProofLocalGivenBindingProducer::build(
            SourceProofLocalGivenBindingHandoffInput {
                source_id: source,
                module_id: module.clone(),
                lower_fingerprint,
                theorem_symbol,
                theorem_definition,
                contribution,
                theorem_range: range(source, 19, 128),
                proof_range: range(source, 62, 127),
                given_range: range(source, 70, 108),
                segment_range: range(source, 76, 87),
                name_range: range(source, 76, 77),
                source_ordinal: 1,
                local: LocalTermBinding::new(
                    "y",
                    LocalTermScope::new(vec![0]),
                    range(source, 76, 77),
                    1,
                ),
                recovery: SourceProofLocalGivenBindingRecovery::Normal,
            },
            &task269gu_base_binding_env(source, module),
        )
        .expect("Task269GU same-identity old-Given neighbor")
    }

    fn task269sdc_neighbor() -> SourceProofLocalGivenDescendantBindingHandoff {
        let source = source_id_for("f8");
        let module = module("task269sdc");
        task269sdc_neighbor_for(source, module)
    }

    fn task269sdc_neighbor_for(
        source: SourceId,
        module: ModuleId,
    ) -> SourceProofLocalGivenDescendantBindingHandoff {
        let escaped_module_path = module
            .path()
            .as_str()
            .replace('\\', "\\\\")
            .replace(':', "\\c")
            .replace('|', "\\p")
            .replace('/', "\\s");
        let local = format!(
            concat!(
                "contribution=0:namespace={}:owner=theorem#1:shell=theorem:",
                "kind=theorem:name=ProofLocalGivenDescendantCaptureSmoke:notation=_:arity=_:",
                "definition=theorem:registration=_:policy=non-overloadable:",
                "slot=non-overloadable:_:theorem:_"
            ),
            escaped_module_path
        );
        let theorem_symbol = SymbolId::new(
            module.clone(),
            LocalSymbolId::new(local.clone()),
            FullyQualifiedName::new(format!(
                "{}::{}::{local}",
                module.package().as_str(),
                module.path().as_str()
            )),
        );
        let mut contributions = SourceContributionIndex::new();
        let contribution = contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 18)),
        );
        let mut definitions = DefinitionIndex::new();
        let theorem_definition = definitions.insert(DefinitionShell::new(
            theorem_symbol.clone(),
            DefinitionKind::Theorem,
            SemanticOrigin::new(
                source,
                module.clone(),
                SourceAnchor::Range(range(source, 19, 179)),
                vec![2, 1],
            ),
            contribution,
        ));
        let lower_fingerprint = format!(
            concat!(
                "source-proof-local-given-descendant-set-lower-debug-v1\n",
                "module: {}::{}\n",
                "source-fingerprint: \"efa21af05a15f611815a4eb573577d0a368a3134693b225bdb56177f3637c2a8\"\n",
                "surface-fingerprint: \"cbeae821434b0db13d77d7dac9984d8d6bf8012de9e7c680be12e8371e87ceaa\"\n",
                "theorem symbol={:?} definition=0 contribution=0 range=19..179 proof=73..178\n",
                "given range=81..99 segment=87..98 source_ordinal=1\n",
                "given-name range=87..88 spelling=\"y\"\n",
                "given-type range=95..98 head=95..98 spelling=\"set\" form=bare\n",
                "descendant-now range=102..159\n",
                "set#0 statement=110..120 equating=114..119 source_ordinal=0\n",
                "set#0 name range=114..115 spelling=\"z\" rhs range=118..119 spelling=\"y\"\n",
                "set#1 statement=125..135 equating=129..134 source_ordinal=1\n",
                "set#1 name range=129..130 spelling=\"q\" rhs range=133..134 spelling=\"z\"\n",
                "conclusions inner=140..152 outer=162..174\n",
            ),
            module.package().as_str(),
            module.path().as_str(),
            theorem_symbol.fqn().as_str(),
        );
        SourceProofLocalGivenDescendantBindingProducer::build(
            SourceProofLocalGivenDescendantBindingHandoffInput {
                source_id: source,
                module_id: module.clone(),
                lower_fingerprint,
                theorem_symbol,
                theorem_definition,
                contribution,
                theorem_range: range(source, 19, 179),
                proof_range: range(source, 73, 178),
                given_range: range(source, 81, 99),
                segment_range: range(source, 87, 98),
                name_range: range(source, 87, 88),
                descendant_range: range(source, 102, 159),
                source_ordinal: 1,
                local: LocalTermBinding::new(
                    "y",
                    LocalTermScope::new(vec![0]),
                    range(source, 87, 88),
                    1,
                ),
                descendant_scope: LocalTermScope::new(vec![0, 0]),
                recovery: SourceProofLocalGivenBindingRecovery::Normal,
            },
            &task269gu_base_binding_env(source, module),
        )
        .expect("Task269SDC ownership neighbor")
    }

    fn task269gu_let_type_neighbor() -> (SourceProofLocalLetTypeHandoff, TypedArena) {
        let source = source_id_for("f7");
        let module = module("task269gup");
        let dependency = task269gu_let_neighbor();
        let arena = task269gu_neighbor_type_arena(
            source,
            "source.proof-local.let.reserve-type",
            "source.proof-local.let.type",
            "source.proof-local.let.type-root",
            76,
            79,
            99,
        );
        let symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        let handoff = SourceProofLocalLetTypeProducer::build(
            dependency,
            task269gu_neighbor_type_input(source, module, 76, 79),
            &symbols,
            &arena,
        )
        .expect("Task269GU same-identity Let-type neighbor");
        (handoff, arena)
    }

    fn task269gu_given_type_neighbor() -> (SourceProofLocalGivenTypeHandoff, TypedArena) {
        let source = source_id_for("f7");
        let module = module("task269gup");
        let dependency = task269gu_given_neighbor();
        let arena = task269gu_neighbor_type_arena(
            source,
            "source.proof-local.given.reserve-type",
            "source.proof-local.given.type",
            "source.proof-local.given.type-root",
            84,
            87,
            128,
        );
        let symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        let handoff = SourceProofLocalGivenTypeProducer::build(
            dependency,
            task269gu_neighbor_type_input(source, module, 84, 87),
            &symbols,
            &arena,
        )
        .expect("Task269GU same-identity old-Given-type neighbor");
        (handoff, arena)
    }

    fn task269gu_neighbor_type_input(
        source: SourceId,
        module: ModuleId,
        local_start: usize,
        local_end: usize,
    ) -> SourceTypeHandoffInput {
        SourceTypeHandoffInput {
            source_id: source,
            module_id: module.clone(),
            applications: (0..2)
                .map(|index| SourceTypeApplicationInput {
                    binding: BindingId::new(index),
                    source_ordinal: index,
                    root: SourceTypeExpressionId::new(index),
                })
                .collect(),
            expressions: [(14, 17), (local_start, local_end)]
                .into_iter()
                .enumerate()
                .map(|(index, (start, end))| SourceTypeExpressionInput {
                    source_id: source,
                    module_id: module.clone(),
                    site: TypedSiteRef::Role {
                        node: TypedNodeId::new(index),
                        role: TypeRole::new("source.type.expression"),
                    },
                    source_range: range(source, start, end),
                    spelling: "set".to_owned(),
                    head_site: TypedSiteRef::Role {
                        node: TypedNodeId::new(index),
                        role: TypeRole::new("source.type.head"),
                    },
                    head_range: range(source, start, end),
                    head_spelling: "set".to_owned(),
                    form: SourceTypeApplicationForm::Bare,
                    head: SourceTypeHead::BuiltinSet,
                    recovery: NodeRecoveryState::Normal,
                })
                .collect(),
            arguments: Vec::new(),
        }
    }

    fn task269gu_neighbor_type_arena(
        source: SourceId,
        reserve_kind: &str,
        local_kind: &str,
        root_kind: &str,
        local_start: usize,
        local_end: usize,
        root_end: usize,
    ) -> TypedArena {
        let mut builder = TypedArenaBuilder::new();
        let reserve = builder
            .push(TypedNode::new(
                reserve_kind,
                SourceAnchor::Range(range(source, 14, 17)),
            ))
            .expect("Task269GU neighbor reserve type node");
        let local = builder
            .push(TypedNode::new(
                local_kind,
                SourceAnchor::Range(range(source, local_start, local_end)),
            ))
            .expect("Task269GU neighbor local type node");
        let root = builder
            .push(
                TypedNode::new(root_kind, SourceAnchor::Range(range(source, 0, root_end)))
                    .with_children(vec![reserve, local]),
            )
            .expect("Task269GU neighbor type root");
        builder
            .finish(Some(root))
            .expect("Task269GU neighbor type arena")
    }

    fn task269gupt_type_input(source: SourceId, module: ModuleId) -> SourceTypeHandoffInput {
        SourceTypeHandoffInput {
            source_id: source,
            module_id: module.clone(),
            applications: (0..2)
                .map(|index| SourceTypeApplicationInput {
                    binding: BindingId::new(index),
                    source_ordinal: index,
                    root: SourceTypeExpressionId::new(index),
                })
                .collect(),
            expressions: [(14, 17), (84, 87)]
                .into_iter()
                .enumerate()
                .map(|(index, (start, end))| SourceTypeExpressionInput {
                    source_id: source,
                    module_id: module.clone(),
                    site: TypedSiteRef::Role {
                        node: TypedNodeId::new(index),
                        role: TypeRole::new("source.type.expression"),
                    },
                    source_range: range(source, start, end),
                    spelling: "set".to_owned(),
                    head_site: TypedSiteRef::Role {
                        node: TypedNodeId::new(index),
                        role: TypeRole::new("source.type.head"),
                    },
                    head_range: range(source, start, end),
                    head_spelling: "set".to_owned(),
                    form: SourceTypeApplicationForm::Bare,
                    head: SourceTypeHead::BuiltinSet,
                    recovery: NodeRecoveryState::Normal,
                })
                .collect(),
            arguments: Vec::new(),
        }
    }

    fn task269gupt_arena(source: SourceId) -> TypedArena {
        let mut builder = TypedArenaBuilder::new();
        let reserve = builder
            .push(TypedNode::new(
                "source.proof-local.given-use.reserve-type",
                SourceAnchor::Range(range(source, 14, 17)),
            ))
            .expect("Task269GU GUPT reserve node");
        let local = builder
            .push(TypedNode::new(
                "source.proof-local.given-use.type",
                SourceAnchor::Range(range(source, 84, 87)),
            ))
            .expect("Task269GU GUPT local node");
        let root = builder
            .push(
                TypedNode::new(
                    "source.proof-local.given-use.type-root",
                    SourceAnchor::Range(range(source, 0, 127)),
                )
                .with_children(vec![reserve, local]),
            )
            .expect("Task269GU GUPT root node");
        builder.finish(Some(root)).expect("Task269GU GUPT arena")
    }

    fn task269gu_arena(source: SourceId, root: usize, wrong_kind: bool) -> TypedArena {
        let mut builder = TypedArenaBuilder::new();
        let reserve = builder
            .push(TypedNode::new(
                "source.proof-local.given-use.reserve-type",
                SourceAnchor::Range(range(source, 14, 17)),
            ))
            .expect("Task269GU reserve node");
        let local = builder
            .push(TypedNode::new(
                "source.proof-local.given-use.type",
                SourceAnchor::Range(range(source, 84, 87)),
            ))
            .expect("Task269GU type node");
        let type_root = builder
            .push(
                TypedNode::new(
                    "source.proof-local.given-use.type-root",
                    SourceAnchor::Range(range(source, 0, 127)),
                )
                .with_children(vec![reserve, local]),
            )
            .expect("Task269GU type root");
        let first = builder
            .push(TypedNode::new(
                "source.term.variable-reference",
                SourceAnchor::Range(range(source, 116, 117)),
            ))
            .expect("Task269GU first term");
        let second = builder
            .push(TypedNode::new(
                "source.term.variable-reference",
                SourceAnchor::Range(range(source, 120, 121)),
            ))
            .expect("Task269GU second term");
        let term_root = builder
            .push(
                TypedNode::new(
                    if wrong_kind {
                        "source.proof-local.given-use.term-root.wrong"
                    } else {
                        "source.proof-local.given-use.term-root"
                    },
                    SourceAnchor::Range(range(source, 0, 127)),
                )
                .with_children(vec![type_root, first, second]),
            )
            .expect("Task269GU term root");
        assert_eq!(term_root, TypedNodeId::new(5));
        builder
            .finish(Some(TypedNodeId::new(root)))
            .expect("Task269GU arena")
    }

    fn mutated_task269gu_input(
        fixture: &Task269guFixture,
        mutation: Task269guInputMutation,
    ) -> SourcePrimaryTermHandoffInput {
        let mut input = fixture.input.clone();
        match mutation {
            Task269guInputMutation::TermCount => {
                input.terms.pop();
            }
            Task269guInputMutation::TermSite => {
                input.terms[1].site = TypedSiteRef::Node(TypedNodeId::new(3));
            }
            Task269guInputMutation::TermRange => {
                input.terms[1].source_range.end += 1;
            }
            Task269guInputMutation::TermOrdinal => {
                input.terms[1].source_ordinal = 2;
            }
            Task269guInputMutation::TermContext => {
                input.terms[1].context = BindingContextId::new(0);
            }
            Task269guInputMutation::TermRecovery => {
                input.terms[1].recovery = SourcePrimaryTermRecovery::Degraded;
            }
            Task269guInputMutation::TermSpelling => {
                input.terms[1].spelling = "x".to_owned();
            }
            Task269guInputMutation::TermKind => {
                input.terms[1].kind = SourcePrimaryTermKind::ConstantReference;
            }
            Task269guInputMutation::TermRole => {
                input.terms[1].role = SourcePrimaryTermRole::CurrentDefinitionResult;
            }
            Task269guInputMutation::TermParent => {
                input.terms[1].parent = Some(SourcePrimaryTermId::new(0));
            }
            Task269guInputMutation::ReferenceCount => {
                input.references.pop();
            }
            Task269guInputMutation::ReferenceTerm => {
                input.references[1].term = SourcePrimaryTermId::new(0);
            }
            Task269guInputMutation::ReferenceBinding => {
                input.references[1].binding = BindingId::new(0);
            }
            Task269guInputMutation::ReferenceRole => {
                input.references[1].role = SourcePrimaryTermReferenceRole::LocalConstant;
            }
            Task269guInputMutation::NumericRequest => {
                input
                    .numeric_type_requests
                    .push(SourceNumericTypeRequestInput {
                        term: SourcePrimaryTermId::new(0),
                        owner: TypedSiteRef::Node(TypedNodeId::new(3)),
                        source_range: range(fixture.source, 116, 117),
                        spelling: "y".to_owned(),
                        request_ordinal: 0,
                    });
            }
        }
        input
    }

    fn mutated_task269gu_arena(
        source: SourceId,
        node: usize,
        mutation: Task269guArenaMutation,
    ) -> TypedArena {
        let exact = task269gu_arena(source, 5, false);
        let mut nodes = exact
            .iter()
            .map(|(_, node)| node.clone())
            .collect::<Vec<_>>();
        match mutation {
            Task269guArenaMutation::Kind => {
                nodes[node].kind = "source.proof-local.given-use.wrong".into();
            }
            Task269guArenaMutation::ResolvedNode => {
                use mizar_syntax as syntax;

                let mut builder = mizar_resolve::resolved_ast::ResolvedArenaBuilder::new();
                let resolved = builder
                    .push(mizar_resolve::resolved_ast::ResolvedNode::new(
                        syntax::SurfaceNodeKind::CompilationUnit,
                        Vec::new(),
                        SemanticOrigin::new(
                            source,
                            module("task269gu.resolved"),
                            SourceAnchor::Range(range(source, 0, 127)),
                            vec![node as u32],
                        ),
                    ))
                    .expect("Task269GU resolved-node mutation id");
                nodes[node].resolved_node = Some(resolved);
            }
            Task269guArenaMutation::Anchor => {
                nodes[node].anchor = SourceAnchor::Range(range(source, 1, 2));
            }
            Task269guArenaMutation::Children => {
                nodes[node].children = match node {
                    0 => vec![TypedNodeId::new(1)],
                    1 => vec![TypedNodeId::new(0)],
                    2 => vec![TypedNodeId::new(1), TypedNodeId::new(0)],
                    3 => vec![TypedNodeId::new(4)],
                    4 => vec![TypedNodeId::new(3)],
                    5 => vec![
                        TypedNodeId::new(4),
                        TypedNodeId::new(3),
                        TypedNodeId::new(2),
                    ],
                    _ => unreachable!("Task269GU arena node is bounded"),
                };
            }
            Task269guArenaMutation::Typing => {
                nodes[node].typing = TypingState::Successful;
            }
            Task269guArenaMutation::Recovery => {
                nodes[node].recovery = NodeRecoveryState::Recovered;
            }
            Task269guArenaMutation::Links => {
                nodes[node].links.facts.push(TypeFactId::new(0));
            }
        }
        TypedArena::try_new(Some(TypedNodeId::new(5)), nodes)
            .expect("Task269GU mutated arena remains structurally constructible")
    }

    fn task269gu_generic_neighbor(fixture: &Task269guFixture) -> SourcePrimaryTermHandoff {
        SourcePrimaryTermProducer::build(
            SourcePrimaryTermHandoffInput {
                source_id: fixture.source,
                module_id: fixture.module.clone(),
                terms: vec![SourcePrimaryTermInput {
                    site: TypedSiteRef::Node(TypedNodeId::new(3)),
                    source_range: range(fixture.source, 116, 117),
                    source_ordinal: 0,
                    context: BindingContextId::new(1),
                    recovery: SourcePrimaryTermRecovery::Normal,
                    spelling: "x".to_owned(),
                    kind: SourcePrimaryTermKind::VariableReference,
                    role: SourcePrimaryTermRole::Value,
                    parent: None,
                }],
                references: vec![SourcePrimaryTermReferenceInput {
                    term: SourcePrimaryTermId::new(0),
                    binding: BindingId::new(0),
                    role: SourcePrimaryTermReferenceRole::Variable,
                }],
                numeric_type_requests: Vec::new(),
            },
            fixture.dependency.binding_env(),
            &fixture.arena,
        )
        .expect("Task269GU generic source-term neighbor")
    }

    fn task269gu_typed_ast(
        fixture: &Task269guFixture,
        handoff: SourceProofLocalGivenUseTermHandoff,
    ) -> Result<TypedAst, TypedAstError> {
        task269gu_empty_typed(fixture, fixture.arena.clone())?
            .with_source_proof_local_given_use_term(handoff)
    }

    fn task269gu_empty_typed(
        fixture: &Task269guFixture,
        nodes: TypedArena,
    ) -> Result<TypedAst, TypedAstError> {
        TypedAst::try_new(TypedAstParts {
            source_id: fixture.source,
            module_id: fixture.module.clone(),
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
    }

    fn task269gu_resolved(typed: &TypedAst) -> ResolvedTypedAst {
        task269gu_resolved_result(typed).expect("Task269GU resolved assembly")
    }

    fn task269gu_resolved_result(
        typed: &TypedAst,
    ) -> Result<ResolvedTypedAst, ResolvedTypedAstError> {
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
            typed_ast: typed,
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
    fn task269gu_exact_occurrence_references_and_fingerprints_are_stable() {
        let fixture = task269gu_fixture();
        let handoff = SourceProofLocalGivenUseTermProducer::build(
            fixture.dependency.clone(),
            fixture.input.clone(),
            &fixture.arena,
        )
        .expect("Task269GU exact checker transaction");
        assert_eq!(handoff.source_id(), fixture.source);
        assert_eq!(handoff.module_id(), &fixture.module);
        assert_eq!(handoff.dependency(), &fixture.dependency);
        assert_eq!(
            handoff.dependency_fingerprint(),
            fixture.dependency.debug_text()
        );
        assert_eq!(
            handoff.dependency().dependency_fingerprint(),
            handoff.dependency().dependency().debug_text()
        );
        assert_eq!(
            handoff.dependency().binding_fingerprint(),
            handoff.dependency().binding_env().debug_text()
        );
        assert_eq!(
            handoff.dependency().source_type_fingerprint(),
            handoff.dependency().source_type().debug_text()
        );
        assert_eq!(
            handoff.source_term_fingerprint(),
            handoff.source_term().debug_text()
        );
        assert_eq!(
            (
                handoff.source_term().terms().len(),
                handoff.source_term().references().len(),
                handoff.source_term().numeric_type_requests().len(),
            ),
            (2, 2, 0)
        );
        for (index, (node, start, end)) in [(3, 116, 117), (4, 120, 121)].into_iter().enumerate() {
            let term = handoff
                .source_term()
                .terms()
                .get(SourcePrimaryTermId::new(index))
                .expect("Task269GU term row");
            assert_eq!(term.site(), &TypedSiteRef::Node(TypedNodeId::new(node)));
            assert_eq!(term.source_range(), range(fixture.source, start, end));
            assert_eq!(term.source_ordinal(), index);
            assert_eq!(term.context(), BindingContextId::new(1));
            assert_eq!(term.recovery(), SourcePrimaryTermRecovery::Normal);
            assert_eq!(term.spelling(), "y");
            assert_eq!(term.kind(), SourcePrimaryTermKind::VariableReference);
            assert_eq!(term.role(), SourcePrimaryTermRole::Value);
            assert_eq!(term.parent(), None);

            let reference = handoff
                .source_term()
                .references()
                .get(SourcePrimaryTermReferenceId::new(index))
                .expect("Task269GU reference row");
            assert_eq!(reference.term(), SourcePrimaryTermId::new(index));
            assert_eq!(reference.binding(), BindingId::new(1));
            assert_eq!(reference.role(), SourcePrimaryTermReferenceRole::Variable);
            assert_eq!(
                reference.lexical_scope().map(LocalTermScope::path),
                Some(&[0][..])
            );
            assert_eq!(reference.use_ordinal(), 2);
        }
        for (index, (kind, start, end, children)) in [
            (
                "source.proof-local.given-use.reserve-type",
                14,
                17,
                Vec::new(),
            ),
            ("source.proof-local.given-use.type", 84, 87, Vec::new()),
            (
                "source.proof-local.given-use.type-root",
                0,
                127,
                vec![TypedNodeId::new(0), TypedNodeId::new(1)],
            ),
            ("source.term.variable-reference", 116, 117, Vec::new()),
            ("source.term.variable-reference", 120, 121, Vec::new()),
            (
                "source.proof-local.given-use.term-root",
                0,
                127,
                vec![
                    TypedNodeId::new(2),
                    TypedNodeId::new(3),
                    TypedNodeId::new(4),
                ],
            ),
        ]
        .into_iter()
        .enumerate()
        {
            assert_eq!(
                fixture.arena.node(TypedNodeId::new(index)),
                Some(
                    &TypedNode::new(kind, SourceAnchor::Range(range(fixture.source, start, end)))
                        .with_children(children)
                )
            );
        }
        assert_eq!(fixture.arena.root(), Some(TypedNodeId::new(5)));
        assert_eq!(
            handoff.debug_text(),
            format!(
                concat!(
                    "source-proof-local-given-use-term-debug-v1\n",
                    "module: pkg::task269gup\n",
                    "dependency-fingerprint: {:?}\n",
                    "source-term-fingerprint: {:?}\n",
                ),
                handoff.dependency_fingerprint(),
                handoff.source_term_fingerprint(),
            )
        );
        let replay = SourceProofLocalGivenUseTermProducer::build(
            fixture.dependency,
            fixture.input,
            &fixture.arena,
        )
        .expect("Task269GU replay");
        assert_eq!(replay, handoff);
    }

    #[test]
    fn task269gu_dependency_term_input_and_arena_corruption_fail_closed() {
        let fixture = task269gu_fixture();
        let mut wrong_source = fixture.input.clone();
        wrong_source.source_id = other_source_id();
        assert_eq!(
            SourceProofLocalGivenUseTermProducer::build(
                fixture.dependency.clone(),
                wrong_source,
                &fixture.arena,
            ),
            Err(SourceProofLocalGivenUseTermError::InvalidDependency)
        );
        let mut wrong_dependency = fixture.input.clone();
        wrong_dependency.module_id = module("task269gu.wrong");
        assert_eq!(
            SourceProofLocalGivenUseTermProducer::build(
                fixture.dependency.clone(),
                wrong_dependency,
                &fixture.arena,
            ),
            Err(SourceProofLocalGivenUseTermError::InvalidDependency)
        );

        for mutation in [
            Task269guInputMutation::TermCount,
            Task269guInputMutation::TermSite,
            Task269guInputMutation::TermRange,
            Task269guInputMutation::TermOrdinal,
            Task269guInputMutation::TermContext,
            Task269guInputMutation::TermRecovery,
            Task269guInputMutation::TermSpelling,
            Task269guInputMutation::TermKind,
            Task269guInputMutation::TermRole,
            Task269guInputMutation::TermParent,
            Task269guInputMutation::ReferenceCount,
            Task269guInputMutation::ReferenceTerm,
            Task269guInputMutation::ReferenceBinding,
            Task269guInputMutation::ReferenceRole,
            Task269guInputMutation::NumericRequest,
        ] {
            assert_eq!(
                SourceProofLocalGivenUseTermProducer::build(
                    fixture.dependency.clone(),
                    mutated_task269gu_input(&fixture, mutation),
                    &fixture.arena,
                ),
                Err(SourceProofLocalGivenUseTermError::InvalidSourceTerm),
                "Task269GU input mutation {mutation:?}",
            );
        }

        for node in 0..6 {
            for mutation in [
                Task269guArenaMutation::Kind,
                Task269guArenaMutation::ResolvedNode,
                Task269guArenaMutation::Anchor,
                Task269guArenaMutation::Children,
                Task269guArenaMutation::Typing,
                Task269guArenaMutation::Recovery,
                Task269guArenaMutation::Links,
            ] {
                let expected = if node < 3 {
                    SourceProofLocalGivenUseTermError::InvalidDependency
                } else if node < 5
                    && matches!(
                        mutation,
                        Task269guArenaMutation::Kind
                            | Task269guArenaMutation::Anchor
                            | Task269guArenaMutation::Recovery
                    )
                {
                    SourceProofLocalGivenUseTermError::InvalidSourceTerm
                } else {
                    SourceProofLocalGivenUseTermError::InvalidInstallation
                };
                assert_eq!(
                    SourceProofLocalGivenUseTermProducer::build(
                        fixture.dependency.clone(),
                        fixture.input.clone(),
                        &mutated_task269gu_arena(fixture.source, node, mutation),
                    ),
                    Err(expected),
                    "Task269GU arena node {node} mutation {mutation:?}",
                );
            }
        }
        for arena in [
            task269gu_arena(fixture.source, 4, false),
            task269gu_arena(fixture.source, 5, true),
        ] {
            assert_eq!(
                SourceProofLocalGivenUseTermProducer::build(
                    fixture.dependency.clone(),
                    fixture.input.clone(),
                    &arena,
                ),
                Err(SourceProofLocalGivenUseTermError::InvalidInstallation)
            );
        }
        let mut extra_nodes = fixture
            .arena
            .iter()
            .map(|(_, node)| node.clone())
            .collect::<Vec<_>>();
        extra_nodes.push(TypedNode::new(
            "source.proof-local.given-use.extra",
            SourceAnchor::Range(range(fixture.source, 0, 127)),
        ));
        let extra_arena =
            TypedArena::try_new(Some(TypedNodeId::new(5)), extra_nodes).expect("extra GU arena");
        assert_eq!(
            SourceProofLocalGivenUseTermProducer::build(
                fixture.dependency.clone(),
                fixture.input.clone(),
                &extra_arena,
            ),
            Err(SourceProofLocalGivenUseTermError::InvalidInstallation)
        );

        let handoff = SourceProofLocalGivenUseTermProducer::build(
            fixture.dependency,
            fixture.input,
            &fixture.arena,
        )
        .expect("Task269GU valid handoff");
        let mut dependency_corrupt = handoff.clone();
        dependency_corrupt.set_dependency_fingerprint_for_test("corrupt".to_owned());
        assert_eq!(
            dependency_corrupt.validate_complete_installation(
                fixture.source,
                &fixture.module,
                &fixture.arena,
                false,
            ),
            Err(SourceProofLocalGivenUseTermError::InvalidDependency)
        );
        let mut source_corrupt = handoff.clone();
        source_corrupt.source_term_mut_for_test().references.rows[0].binding = BindingId::new(0);
        source_corrupt.source_term_fingerprint = source_corrupt.source_term().debug_text();
        assert_eq!(
            source_corrupt.validate_complete_installation(
                fixture.source,
                &fixture.module,
                &fixture.arena,
                false,
            ),
            Err(SourceProofLocalGivenUseTermError::InvalidSourceTerm)
        );
        let mut source_fingerprint_corrupt = handoff.clone();
        source_fingerprint_corrupt
            .source_term_fingerprint
            .push_str("corrupt");
        assert_eq!(
            source_fingerprint_corrupt.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.arena,
            ),
            Err(SourceProofLocalGivenUseTermError::InvalidSourceTerm)
        );
        let mut dependency_before_source = source_corrupt;
        dependency_before_source.set_dependency_fingerprint_for_test("corrupt".to_owned());
        assert_eq!(
            dependency_before_source.validate_complete_installation(
                fixture.source,
                &fixture.module,
                &fixture.arena,
                false,
            ),
            Err(SourceProofLocalGivenUseTermError::InvalidDependency)
        );
        assert_eq!(
            handoff.validate_complete_installation(
                fixture.source,
                &fixture.module,
                &fixture.arena,
                false,
            ),
            Err(SourceProofLocalGivenUseTermError::InvalidInstallation)
        );
        for (error, expected) in [
            (
                SourceProofLocalGivenUseTermError::InvalidDependency,
                "source proof-local given-use term dependency is invalid",
            ),
            (
                SourceProofLocalGivenUseTermError::InvalidSourceTerm,
                "source proof-local given-use source term is invalid",
            ),
            (
                SourceProofLocalGivenUseTermError::InvalidInstallation,
                "source proof-local given-use term installation is invalid",
            ),
        ] {
            assert_eq!(error.to_string(), expected);
            let _: &dyn std::error::Error = &error;
        }
    }

    #[test]
    fn task269gu_typed_and_resolved_ownership_is_atomic() {
        let fixture = task269gu_fixture();
        let handoff = SourceProofLocalGivenUseTermProducer::build(
            fixture.dependency.clone(),
            fixture.input.clone(),
            &fixture.arena,
        )
        .expect("Task269GU handoff");
        let typed =
            task269gu_typed_ast(&fixture, handoff.clone()).expect("Task269GU typed installation");
        assert_eq!(typed.source_proof_local_given_use_term(), Some(&handoff));
        assert!(typed.source_proof_local_given_use_type().is_none());
        assert_eq!(
            typed
                .clone()
                .with_source_proof_local_given_use_term(handoff.clone()),
            Err(TypedAstError::InvalidSourceProofLocalGivenUseTerm)
        );

        let gupt_typed = TypedAst::try_new(TypedAstParts {
            source_id: fixture.source,
            module_id: fixture.module.clone(),
            resolved_root: None,
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes: task269gupt_arena(fixture.source),
            contexts: LocalTypeContextTable::new(),
            types: TypeTable::new(),
            facts: TypeFactTable::new(),
            coercions: CoercionTable::new(),
            initial_obligations: InitialObligationTable::new(),
            diagnostics: TypeDiagnosticTable::new(),
        })
        .expect("Task269GU dependency typed AST")
        .with_source_proof_local_given_use_type(fixture.dependency.clone())
        .expect("Task269GU dependency typed owner");
        assert_eq!(
            gupt_typed.with_source_proof_local_given_use_term(handoff.clone()),
            Err(TypedAstError::InvalidSourceProofLocalGivenUseTerm)
        );

        let generic_neighbor = task269gu_generic_neighbor(&fixture);
        let generic_before_use = task269gu_empty_typed(&fixture, fixture.arena.clone())
            .expect("Task269GU generic-before-use base")
            .with_source_term(generic_neighbor.clone())
            .expect("Task269GU generic-before-use owner");
        assert_eq!(
            generic_before_use.with_source_proof_local_given_use_term(handoff.clone()),
            Err(TypedAstError::InvalidSourceProofLocalGivenUseTerm)
        );
        assert_eq!(
            typed.clone().with_source_term(generic_neighbor),
            Err(TypedAstError::InvalidSourceTerm)
        );

        let let_neighbor = task269gu_let_neighbor();
        let mut let_before_use = task269gu_empty_typed(&fixture, fixture.arena.clone())
            .expect("Task269GU Let-before-use base");
        let_before_use.inject_source_proof_local_let_binding_for_test(let_neighbor.clone());
        assert_eq!(
            let_before_use.with_source_proof_local_given_use_term(handoff.clone()),
            Err(TypedAstError::InvalidSourceProofLocalGivenUseTerm)
        );
        let mut resolved_let_hybrid = typed.clone();
        resolved_let_hybrid.inject_source_proof_local_let_binding_for_test(let_neighbor.clone());
        assert!(matches!(
            task269gu_resolved_result(&resolved_let_hybrid),
            Err(ResolvedTypedAstError::InvalidSourceProofLocalLetBinding)
                | Err(ResolvedTypedAstError::InvalidSourceProofLocalGivenUseTerm)
        ));
        let mut use_before_let = task269gu_empty_typed(
            &fixture,
            TypedArena::try_new(None, Vec::new()).expect("Task269GU empty Let arena"),
        )
        .expect("Task269GU use-before-Let base");
        use_before_let.inject_source_proof_local_given_use_term_for_test(handoff.clone());
        assert_eq!(
            use_before_let.with_source_proof_local_let_binding(let_neighbor),
            Err(TypedAstError::InvalidSourceProofLocalLetBinding)
        );

        let (let_type_neighbor, let_type_arena) = task269gu_let_type_neighbor();
        let mut let_type_before_use = task269gu_empty_typed(&fixture, fixture.arena.clone())
            .expect("Task269GU Let-type-before-use base");
        let_type_before_use.inject_source_proof_local_let_type_for_test(let_type_neighbor.clone());
        assert_eq!(
            let_type_before_use.with_source_proof_local_given_use_term(handoff.clone()),
            Err(TypedAstError::InvalidSourceProofLocalGivenUseTerm)
        );
        let mut resolved_let_type_hybrid = typed.clone();
        resolved_let_type_hybrid
            .inject_source_proof_local_let_type_for_test(let_type_neighbor.clone());
        assert!(matches!(
            task269gu_resolved_result(&resolved_let_type_hybrid),
            Err(ResolvedTypedAstError::InvalidSourceProofLocalLetType)
                | Err(ResolvedTypedAstError::InvalidSourceProofLocalGivenUseTerm)
        ));
        let mut use_before_let_type = task269gu_empty_typed(&fixture, let_type_arena)
            .expect("Task269GU use-before-Let-type base");
        use_before_let_type.inject_source_proof_local_given_use_term_for_test(handoff.clone());
        assert_eq!(
            use_before_let_type.with_source_proof_local_let_type(let_type_neighbor),
            Err(TypedAstError::InvalidSourceProofLocalLetType)
        );

        let given_neighbor = task269gu_given_neighbor();
        let mut given_before_use = task269gu_empty_typed(&fixture, fixture.arena.clone())
            .expect("Task269GU old-Given-before-use base");
        given_before_use.inject_source_proof_local_given_binding_for_test(given_neighbor.clone());
        assert_eq!(
            given_before_use.with_source_proof_local_given_use_term(handoff.clone()),
            Err(TypedAstError::InvalidSourceProofLocalGivenUseTerm)
        );
        let mut resolved_given_hybrid = typed.clone();
        resolved_given_hybrid
            .inject_source_proof_local_given_binding_for_test(given_neighbor.clone());
        assert!(matches!(
            task269gu_resolved_result(&resolved_given_hybrid),
            Err(ResolvedTypedAstError::InvalidSourceProofLocalGivenBinding)
                | Err(ResolvedTypedAstError::InvalidSourceProofLocalGivenUseTerm)
        ));
        let mut use_before_given = task269gu_empty_typed(
            &fixture,
            TypedArena::try_new(None, Vec::new()).expect("Task269GU empty Given arena"),
        )
        .expect("Task269GU use-before-old-Given base");
        use_before_given.inject_source_proof_local_given_use_term_for_test(handoff.clone());
        assert_eq!(
            use_before_given.with_source_proof_local_given_binding(given_neighbor),
            Err(TypedAstError::InvalidSourceProofLocalGivenBinding)
        );

        let (given_type_neighbor, given_type_arena) = task269gu_given_type_neighbor();
        let mut given_type_before_use = task269gu_empty_typed(&fixture, fixture.arena.clone())
            .expect("Task269GU old-Given-type-before-use base");
        given_type_before_use
            .inject_source_proof_local_given_type_for_test(given_type_neighbor.clone());
        assert_eq!(
            given_type_before_use.with_source_proof_local_given_use_term(handoff.clone()),
            Err(TypedAstError::InvalidSourceProofLocalGivenUseTerm)
        );
        let mut resolved_given_type_hybrid = typed.clone();
        resolved_given_type_hybrid
            .inject_source_proof_local_given_type_for_test(given_type_neighbor.clone());
        assert!(matches!(
            task269gu_resolved_result(&resolved_given_type_hybrid),
            Err(ResolvedTypedAstError::InvalidSourceProofLocalGivenType)
                | Err(ResolvedTypedAstError::InvalidSourceProofLocalGivenUseTerm)
        ));
        let mut use_before_given_type = task269gu_empty_typed(&fixture, given_type_arena)
            .expect("Task269GU use-before-old-Given-type base");
        use_before_given_type.inject_source_proof_local_given_use_term_for_test(handoff.clone());
        assert_eq!(
            use_before_given_type.with_source_proof_local_given_type(given_type_neighbor),
            Err(TypedAstError::InvalidSourceProofLocalGivenType)
        );

        let mut gupt_before_use = task269gu_empty_typed(&fixture, fixture.arena.clone())
            .expect("Task269GU injected GUPT-before-use base");
        gupt_before_use
            .inject_source_proof_local_given_use_type_for_test(fixture.dependency.clone());
        assert_eq!(
            gupt_before_use.with_source_proof_local_given_use_term(handoff.clone()),
            Err(TypedAstError::InvalidSourceProofLocalGivenUseTerm)
        );
        let mut resolved_gupt_hybrid = typed.clone();
        resolved_gupt_hybrid
            .inject_source_proof_local_given_use_type_for_test(fixture.dependency.clone());
        assert!(matches!(
            task269gu_resolved_result(&resolved_gupt_hybrid),
            Err(ResolvedTypedAstError::InvalidSourceProofLocalGivenUseType)
                | Err(ResolvedTypedAstError::InvalidSourceProofLocalGivenUseTerm)
        ));
        let mut use_before_gupt =
            task269gu_empty_typed(&fixture, task269gupt_arena(fixture.source))
                .expect("Task269GU injected use-before-GUPT base");
        use_before_gupt.inject_source_proof_local_given_use_term_for_test(handoff.clone());
        assert_eq!(
            use_before_gupt.with_source_proof_local_given_use_type(fixture.dependency.clone()),
            Err(TypedAstError::InvalidSourceProofLocalGivenUseType)
        );
        assert_eq!(
            handoff.dependency().dependency(),
            fixture.dependency.dependency()
        );

        let resolved = task269gu_resolved(&typed);
        assert_eq!(resolved.source_proof_local_given_use_term(), Some(&handoff));
        assert_eq!(resolved.nodes().len(), 6);
        for (_, node) in resolved.nodes().iter() {
            assert!(matches!(
                &node.kind,
                ResolvedTypedNodeKind::SourcePreserved { role }
                    if role.as_str() == "source.proof-local.given-use.term"
            ));
        }
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
        assert!(resolved.checked_formulas().is_empty());
        assert!(resolved.statement_semantics().is_empty());
        assert!(resolved.checked_proofs().is_empty());
        assert!(resolved.checked_proof_nodes().is_empty());
        assert!(resolved.checked_terminal_goals().is_empty());
        assert!(resolved.diagnostics().is_empty());

        let mut injected = TypedAst::try_new(TypedAstParts {
            source_id: fixture.source,
            module_id: fixture.module,
            resolved_root: None,
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes: fixture.arena,
            contexts: LocalTypeContextTable::new(),
            types: TypeTable::new(),
            facts: TypeFactTable::new(),
            coercions: CoercionTable::new(),
            initial_obligations: InitialObligationTable::new(),
            diagnostics: TypeDiagnosticTable::new(),
        })
        .expect("Task269GU injectable typed AST");
        injected.inject_source_proof_local_given_use_term_for_test(handoff);
        assert_eq!(task269gu_resolved(&injected), resolved);
    }

    #[test]
    fn task269gu_generic_and_neighbor_routes_remain_isolated() {
        let task_fixture = task269gu_fixture();
        assert!(matches!(
            SourcePrimaryTermProducer::build(
                task_fixture.input.clone(),
                task_fixture.dependency.binding_env(),
                &task_fixture.arena,
            ),
            Err(SourcePrimaryTermError::InvalidReference { .. })
        ));
        assert!(
            SourceProofLocalGivenUseTermProducer::build(
                task_fixture.dependency.clone(),
                task_fixture.input.clone(),
                &task_fixture.arena,
            )
            .is_ok()
        );

        let mut parent_context = task_fixture.input.clone();
        parent_context.terms[0].context = BindingContextId::new(0);
        assert_eq!(
            SourceProofLocalGivenUseTermProducer::build(
                task_fixture.dependency.clone(),
                parent_context,
                &task_fixture.arena,
            ),
            Err(SourceProofLocalGivenUseTermError::InvalidSourceTerm)
        );
        let mut extra_term = task_fixture.input.clone();
        let mut row = extra_term.terms[1].clone();
        row.source_ordinal = 2;
        extra_term.terms.push(row);
        assert_eq!(
            SourceProofLocalGivenUseTermProducer::build(
                task_fixture.dependency,
                extra_term,
                &task_fixture.arena,
            ),
            Err(SourceProofLocalGivenUseTermError::InvalidSourceTerm)
        );

        let generic = fixture();
        assert!(build(&generic).is_ok());
    }

    #[derive(Clone)]
    struct Task269gcuFixture {
        source: SourceId,
        module: ModuleId,
        dependency: SourceProofLocalGivenConditionTypeHandoff,
        input: SourcePrimaryTermHandoffInput,
        arena: TypedArena,
    }

    fn task269gcu_fixture() -> Task269gcuFixture {
        let source = source_id_for("f7");
        let module = module("task269gc");
        let local = concat!(
            "contribution=0:namespace=task269gc:owner=theorem#1:shell=theorem:",
            "kind=theorem:name=ProofLocalGivenConditionUseSmoke:notation=_:arity=_:",
            "definition=theorem:registration=_:policy=non-overloadable:",
            "slot=non-overloadable:_:theorem:_"
        );
        let theorem_symbol = SymbolId::new(
            module.clone(),
            LocalSymbolId::new(local),
            FullyQualifiedName::new(format!("pkg::task269gc::{local}")),
        );
        let mut contributions = SourceContributionIndex::new();
        let contribution = contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 18)),
        );
        let mut definitions = DefinitionIndex::new();
        let theorem_definition = definitions.insert(DefinitionShell::new(
            theorem_symbol.clone(),
            DefinitionKind::Theorem,
            SemanticOrigin::new(
                source,
                module.clone(),
                SourceAnchor::Range(range(source, 19, 133)),
                vec![2, 1],
            ),
            contribution,
        ));
        let lower_fingerprint = format!(
            concat!(
                "source-proof-local-given-condition-lower-debug-v1\n",
                "module: {}::{}\n",
                "source-fingerprint: \"2c2d767a0654670412b377bdcc6c5970ecec05b41c02aa754766320927bc6aad\"\n",
                "surface-fingerprint: \"49d46d5f24338772e6e968f12c2216a8957b35242474132690db843b510b430f\"\n",
                "theorem symbol={:?} definition=0 contribution=0 range=19..133 proof=68..132\n",
                "given range=76..113 segment=82..93 source_ordinal=1\n",
                "name range=82..83 spelling=\"y\"\n",
                "type range=90..93 head=90..93 spelling=\"set\" form=bare\n",
            ),
            module.package().as_str(),
            module.path().as_str(),
            theorem_symbol.fqn().as_str(),
        );
        let binding = SourceProofLocalGivenConditionBindingProducer::build(
            SourceProofLocalGivenConditionBindingHandoffInput {
                source_id: source,
                module_id: module.clone(),
                lower_fingerprint,
                theorem_symbol,
                theorem_definition,
                contribution,
                theorem_range: range(source, 19, 133),
                proof_range: range(source, 68, 132),
                given_range: range(source, 76, 113),
                segment_range: range(source, 82, 93),
                name_range: range(source, 82, 83),
                source_ordinal: 1,
                local: LocalTermBinding::new(
                    "y",
                    LocalTermScope::new(vec![0]),
                    range(source, 82, 83),
                    1,
                ),
                recovery: SourceProofLocalGivenBindingRecovery::Normal,
            },
            &task269gu_base_binding_env(source, module.clone()),
        )
        .expect("Task269GCU exact GC dependency");
        let symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        let type_arena = task269gcu_type_arena(source);
        let dependency = SourceProofLocalGivenConditionTypeProducer::build(
            binding,
            task269gcu_type_input(source, module.clone()),
            &symbols,
            &type_arena,
        )
        .expect("Task269GCU exact GCT dependency");
        Task269gcuFixture {
            source,
            module: module.clone(),
            dependency,
            input: task269gcu_input(source, module),
            arena: task269gcu_test_arena(source),
        }
    }

    fn task269gcu_type_input(source: SourceId, module: ModuleId) -> SourceTypeHandoffInput {
        SourceTypeHandoffInput {
            source_id: source,
            module_id: module.clone(),
            applications: (0..2)
                .map(|index| SourceTypeApplicationInput {
                    binding: BindingId::new(index),
                    source_ordinal: index,
                    root: SourceTypeExpressionId::new(index),
                })
                .collect(),
            expressions: [(14, 17), (90, 93)]
                .into_iter()
                .enumerate()
                .map(|(index, (start, end))| SourceTypeExpressionInput {
                    source_id: source,
                    module_id: module.clone(),
                    site: TypedSiteRef::Role {
                        node: TypedNodeId::new(index),
                        role: TypeRole::new("source.type.expression"),
                    },
                    source_range: range(source, start, end),
                    spelling: "set".to_owned(),
                    head_site: TypedSiteRef::Role {
                        node: TypedNodeId::new(index),
                        role: TypeRole::new("source.type.head"),
                    },
                    head_range: range(source, start, end),
                    head_spelling: "set".to_owned(),
                    form: SourceTypeApplicationForm::Bare,
                    head: SourceTypeHead::BuiltinSet,
                    recovery: NodeRecoveryState::Normal,
                })
                .collect(),
            arguments: Vec::new(),
        }
    }

    fn task269gcu_type_arena(source: SourceId) -> TypedArena {
        let mut builder = TypedArenaBuilder::new();
        let reserve = builder
            .push(TypedNode::new(
                "source.proof-local.given-condition.reserve-type",
                SourceAnchor::Range(range(source, 14, 17)),
            ))
            .expect("Task269GCU reserve type node");
        let local = builder
            .push(TypedNode::new(
                "source.proof-local.given-condition.type",
                SourceAnchor::Range(range(source, 90, 93)),
            ))
            .expect("Task269GCU local type node");
        let root = builder
            .push(
                TypedNode::new(
                    "source.proof-local.given-condition.type-root",
                    SourceAnchor::Range(range(source, 0, 133)),
                )
                .with_children(vec![reserve, local]),
            )
            .expect("Task269GCU type root");
        builder.finish(Some(root)).expect("Task269GCU type arena")
    }

    fn task269gcu_test_arena(source: SourceId) -> TypedArena {
        let mut nodes = task269gcu_type_arena(source)
            .iter()
            .map(|(_, node)| node.clone())
            .collect::<Vec<_>>();
        nodes.push(TypedNode::new(
            "source.term.variable-reference",
            SourceAnchor::Range(range(source, 107, 108)),
        ));
        nodes.push(TypedNode::new(
            "source.term.variable-reference",
            SourceAnchor::Range(range(source, 111, 112)),
        ));
        nodes.push(
            TypedNode::new(
                "source.proof-local.given-condition.term-root",
                SourceAnchor::Range(range(source, 0, 133)),
            )
            .with_children(vec![
                TypedNodeId::new(2),
                TypedNodeId::new(3),
                TypedNodeId::new(4),
            ]),
        );
        TypedArena::try_new(Some(TypedNodeId::new(5)), nodes).expect("Task269GCU term arena")
    }

    fn mutated_task269gcu_input(
        fixture: &Task269gcuFixture,
        mutation: Task269guInputMutation,
    ) -> SourcePrimaryTermHandoffInput {
        let mut input = fixture.input.clone();
        match mutation {
            Task269guInputMutation::TermCount => {
                input.terms.pop();
            }
            Task269guInputMutation::TermSite => {
                input.terms[1].site = TypedSiteRef::Node(TypedNodeId::new(3));
            }
            Task269guInputMutation::TermRange => {
                input.terms[1].source_range.end += 1;
            }
            Task269guInputMutation::TermOrdinal => {
                input.terms[1].source_ordinal = 2;
            }
            Task269guInputMutation::TermContext => {
                input.terms[1].context = BindingContextId::new(0);
            }
            Task269guInputMutation::TermRecovery => {
                input.terms[1].recovery = SourcePrimaryTermRecovery::Degraded;
            }
            Task269guInputMutation::TermSpelling => {
                input.terms[1].spelling = "x".to_owned();
            }
            Task269guInputMutation::TermKind => {
                input.terms[1].kind = SourcePrimaryTermKind::ConstantReference;
            }
            Task269guInputMutation::TermRole => {
                input.terms[1].role = SourcePrimaryTermRole::CurrentDefinitionResult;
            }
            Task269guInputMutation::TermParent => {
                input.terms[1].parent = Some(SourcePrimaryTermId::new(0));
            }
            Task269guInputMutation::ReferenceCount => {
                input.references.pop();
            }
            Task269guInputMutation::ReferenceTerm => {
                input.references[1].term = SourcePrimaryTermId::new(0);
            }
            Task269guInputMutation::ReferenceBinding => {
                input.references[1].binding = BindingId::new(0);
            }
            Task269guInputMutation::ReferenceRole => {
                input.references[1].role = SourcePrimaryTermReferenceRole::LocalConstant;
            }
            Task269guInputMutation::NumericRequest => {
                input
                    .numeric_type_requests
                    .push(SourceNumericTypeRequestInput {
                        term: SourcePrimaryTermId::new(0),
                        owner: TypedSiteRef::Node(TypedNodeId::new(3)),
                        source_range: range(fixture.source, 107, 108),
                        spelling: "y".to_owned(),
                        request_ordinal: 0,
                    });
            }
        }
        input
    }

    fn mutated_task269gcu_arena(
        fixture: &Task269gcuFixture,
        node: usize,
        mutation: Task269guArenaMutation,
    ) -> TypedArena {
        let mut nodes = fixture
            .arena
            .iter()
            .map(|(_, node)| node.clone())
            .collect::<Vec<_>>();
        match mutation {
            Task269guArenaMutation::Kind => {
                nodes[node].kind = "source.proof-local.given-condition.wrong".into();
            }
            Task269guArenaMutation::ResolvedNode => {
                use mizar_syntax as syntax;

                let mut builder = mizar_resolve::resolved_ast::ResolvedArenaBuilder::new();
                let resolved = builder
                    .push(mizar_resolve::resolved_ast::ResolvedNode::new(
                        syntax::SurfaceNodeKind::CompilationUnit,
                        Vec::new(),
                        SemanticOrigin::new(
                            fixture.source,
                            module("task269gcu.resolved"),
                            SourceAnchor::Range(range(fixture.source, 0, 133)),
                            vec![node as u32],
                        ),
                    ))
                    .expect("Task269GCU resolved-node mutation id");
                nodes[node].resolved_node = Some(resolved);
            }
            Task269guArenaMutation::Anchor => {
                nodes[node].anchor = SourceAnchor::Range(range(fixture.source, 1, 2));
            }
            Task269guArenaMutation::Children => {
                nodes[node].children = match node {
                    0 => vec![TypedNodeId::new(1)],
                    1 => vec![TypedNodeId::new(0)],
                    2 => vec![TypedNodeId::new(1), TypedNodeId::new(0)],
                    3 => vec![TypedNodeId::new(4)],
                    4 => vec![TypedNodeId::new(3)],
                    5 => vec![
                        TypedNodeId::new(4),
                        TypedNodeId::new(3),
                        TypedNodeId::new(2),
                    ],
                    _ => unreachable!("Task269GCU arena node is bounded"),
                };
            }
            Task269guArenaMutation::Typing => {
                nodes[node].typing = TypingState::Successful;
            }
            Task269guArenaMutation::Recovery => {
                nodes[node].recovery = NodeRecoveryState::Recovered;
            }
            Task269guArenaMutation::Links => {
                nodes[node].links.facts.push(TypeFactId::new(0));
            }
        }
        TypedArena::try_new(Some(TypedNodeId::new(5)), nodes)
            .expect("Task269GCU mutated arena remains structurally constructible")
    }

    fn task269gcu_arena_with_root(fixture: &Task269gcuFixture, root: usize) -> TypedArena {
        TypedArena::try_new(
            Some(TypedNodeId::new(root)),
            fixture.arena.iter().map(|(_, node)| node.clone()).collect(),
        )
        .expect("Task269GCU alternate-root arena")
    }

    fn task269gcu_empty_typed(fixture: &Task269gcuFixture, arena: TypedArena) -> TypedAst {
        task269gcu_empty_typed_for(fixture.source, fixture.module.clone(), arena)
    }

    fn task269gcu_empty_typed_for(
        source: SourceId,
        module: ModuleId,
        arena: TypedArena,
    ) -> TypedAst {
        TypedAst::try_new(TypedAstParts {
            source_id: source,
            module_id: module,
            resolved_root: None,
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes: arena,
            contexts: LocalTypeContextTable::new(),
            types: TypeTable::new(),
            facts: TypeFactTable::new(),
            coercions: CoercionTable::new(),
            initial_obligations: InitialObligationTable::new(),
            diagnostics: TypeDiagnosticTable::new(),
        })
        .expect("Task269GCU empty typed AST")
    }

    fn task269gcu_generic_neighbor(fixture: &Task269gcuFixture) -> SourcePrimaryTermHandoff {
        SourcePrimaryTermProducer::build(
            SourcePrimaryTermHandoffInput {
                source_id: fixture.source,
                module_id: fixture.module.clone(),
                terms: vec![SourcePrimaryTermInput {
                    site: TypedSiteRef::Node(TypedNodeId::new(3)),
                    source_range: range(fixture.source, 107, 108),
                    source_ordinal: 0,
                    context: BindingContextId::new(1),
                    recovery: SourcePrimaryTermRecovery::Normal,
                    spelling: "x".to_owned(),
                    kind: SourcePrimaryTermKind::VariableReference,
                    role: SourcePrimaryTermRole::Value,
                    parent: None,
                }],
                references: vec![SourcePrimaryTermReferenceInput {
                    term: SourcePrimaryTermId::new(0),
                    binding: BindingId::new(0),
                    role: SourcePrimaryTermReferenceRole::Variable,
                }],
                numeric_type_requests: Vec::new(),
            },
            fixture.dependency.binding_env(),
            &fixture.arena,
        )
        .expect("Task269GCU generic source-term neighbor")
    }

    fn task269gcu_resolved_result_with_inputs(
        typed: &TypedAst,
        expressions: Vec<ExpressionMetadataInput>,
        node_hints: Vec<ResolvedNodeKindHint>,
    ) -> Result<ResolvedTypedAst, ResolvedTypedAstError> {
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
            typed_ast: typed,
            cluster_facts: &cluster_facts,
            overload_collection: &collection,
            template_expansion: &expansion,
            viability: &viability,
            specificity: &specificity,
            overload_selection: &selection,
            expressions,
            node_hints,
            statement_semantics: None,
            statement_proofs: None,
        })
    }

    #[test]
    fn task269gcu_exact_occurrence_references_and_fingerprints_are_stable() {
        let fixture = task269gcu_fixture();
        let handoff = SourceProofLocalGivenConditionUseTermProducer::build(
            fixture.dependency.clone(),
            fixture.input.clone(),
            &fixture.arena,
        )
        .expect("Task269GCU exact checker transaction");
        assert_eq!(handoff.source_id(), fixture.source);
        assert_eq!(handoff.module_id(), &fixture.module);
        assert_eq!(handoff.dependency(), &fixture.dependency);
        assert_eq!(
            handoff.dependency_fingerprint(),
            fixture.dependency.debug_text()
        );
        assert_eq!(
            handoff.source_term_fingerprint(),
            handoff.source_term().debug_text()
        );
        assert_eq!(
            (
                handoff.source_term().terms().len(),
                handoff.source_term().references().len(),
                handoff.source_term().numeric_type_requests().len(),
            ),
            (2, 2, 0)
        );
        for (index, (node, start, end)) in [(3, 107, 108), (4, 111, 112)].into_iter().enumerate() {
            let term = handoff
                .source_term()
                .terms()
                .get(SourcePrimaryTermId::new(index))
                .expect("Task269GCU term row");
            assert_eq!(term.site(), &TypedSiteRef::Node(TypedNodeId::new(node)));
            assert_eq!(term.source_range(), range(fixture.source, start, end));
            assert_eq!(term.source_ordinal(), index);
            assert_eq!(term.context(), BindingContextId::new(1));
            assert_eq!(term.spelling(), "y");
            assert_eq!(term.kind(), SourcePrimaryTermKind::VariableReference);
            assert_eq!(term.role(), SourcePrimaryTermRole::Value);
            assert_eq!(term.parent(), None);
            let reference = handoff
                .source_term()
                .references()
                .get(SourcePrimaryTermReferenceId::new(index))
                .expect("Task269GCU reference row");
            assert_eq!(reference.term(), SourcePrimaryTermId::new(index));
            assert_eq!(reference.binding(), BindingId::new(1));
            assert_eq!(reference.role(), SourcePrimaryTermReferenceRole::Variable);
            assert_eq!(reference.use_ordinal(), 2);
            assert_eq!(
                reference.lexical_scope().map(LocalTermScope::path),
                Some(&[0][..])
            );
        }
        assert_eq!(
            handoff.debug_text(),
            format!(
                concat!(
                    "source-proof-local-given-condition-use-term-debug-v1\n",
                    "module: pkg::task269gc\n",
                    "dependency-fingerprint: {:?}\n",
                    "source-term-fingerprint: {:?}\n",
                ),
                handoff.dependency_fingerprint(),
                handoff.source_term_fingerprint(),
            )
        );
        let replay = SourceProofLocalGivenConditionUseTermProducer::build(
            fixture.dependency,
            fixture.input,
            &fixture.arena,
        )
        .expect("Task269GCU replay");
        assert_eq!(replay, handoff);
    }

    #[test]
    fn task269gcu_dependency_term_input_and_arena_corruption_fail_closed() {
        let fixture = task269gcu_fixture();
        let mut wrong_source = fixture.input.clone();
        wrong_source.source_id = other_source_id();
        assert_eq!(
            SourceProofLocalGivenConditionUseTermProducer::build(
                fixture.dependency.clone(),
                wrong_source,
                &fixture.arena,
            ),
            Err(SourceProofLocalGivenConditionUseTermError::InvalidDependency)
        );
        let mut wrong_module = fixture.input.clone();
        wrong_module.module_id = module("task269gcu.wrong");
        assert_eq!(
            SourceProofLocalGivenConditionUseTermProducer::build(
                fixture.dependency.clone(),
                wrong_module,
                &fixture.arena,
            ),
            Err(SourceProofLocalGivenConditionUseTermError::InvalidDependency)
        );
        for mutation in [
            Task269guInputMutation::TermCount,
            Task269guInputMutation::TermSite,
            Task269guInputMutation::TermRange,
            Task269guInputMutation::TermOrdinal,
            Task269guInputMutation::TermContext,
            Task269guInputMutation::TermRecovery,
            Task269guInputMutation::TermSpelling,
            Task269guInputMutation::TermKind,
            Task269guInputMutation::TermRole,
            Task269guInputMutation::TermParent,
            Task269guInputMutation::ReferenceCount,
            Task269guInputMutation::ReferenceTerm,
            Task269guInputMutation::ReferenceBinding,
            Task269guInputMutation::ReferenceRole,
            Task269guInputMutation::NumericRequest,
        ] {
            assert_eq!(
                SourceProofLocalGivenConditionUseTermProducer::build(
                    fixture.dependency.clone(),
                    mutated_task269gcu_input(&fixture, mutation),
                    &fixture.arena,
                ),
                Err(SourceProofLocalGivenConditionUseTermError::InvalidSourceTerm),
                "Task269GCU input mutation {mutation:?}",
            );
        }
        for node in 0..6 {
            for mutation in [
                Task269guArenaMutation::Kind,
                Task269guArenaMutation::ResolvedNode,
                Task269guArenaMutation::Anchor,
                Task269guArenaMutation::Children,
                Task269guArenaMutation::Typing,
                Task269guArenaMutation::Recovery,
                Task269guArenaMutation::Links,
            ] {
                let expected = if node < 3 {
                    SourceProofLocalGivenConditionUseTermError::InvalidDependency
                } else if node < 5
                    && matches!(
                        mutation,
                        Task269guArenaMutation::Kind
                            | Task269guArenaMutation::Anchor
                            | Task269guArenaMutation::Recovery
                    )
                {
                    SourceProofLocalGivenConditionUseTermError::InvalidSourceTerm
                } else {
                    SourceProofLocalGivenConditionUseTermError::InvalidInstallation
                };
                assert_eq!(
                    SourceProofLocalGivenConditionUseTermProducer::build(
                        fixture.dependency.clone(),
                        fixture.input.clone(),
                        &mutated_task269gcu_arena(&fixture, node, mutation),
                    ),
                    Err(expected),
                    "Task269GCU arena node {node} mutation {mutation:?}",
                );
            }
        }
        assert_eq!(
            SourceProofLocalGivenConditionUseTermProducer::build(
                fixture.dependency.clone(),
                fixture.input.clone(),
                &task269gcu_arena_with_root(&fixture, 4),
            ),
            Err(SourceProofLocalGivenConditionUseTermError::InvalidInstallation)
        );
        let missing_root = TypedArena::try_new(
            Some(TypedNodeId::new(4)),
            fixture
                .arena
                .iter()
                .take(5)
                .map(|(_, node)| node.clone())
                .collect(),
        )
        .expect("Task269GCU missing term-root arena");
        assert_eq!(
            SourceProofLocalGivenConditionUseTermProducer::build(
                fixture.dependency.clone(),
                fixture.input.clone(),
                &missing_root,
            ),
            Err(SourceProofLocalGivenConditionUseTermError::InvalidInstallation)
        );
        let mut extra_nodes = fixture
            .arena
            .iter()
            .map(|(_, node)| node.clone())
            .collect::<Vec<_>>();
        extra_nodes.push(TypedNode::new(
            "source.proof-local.given-condition.extra",
            SourceAnchor::Range(range(fixture.source, 0, 133)),
        ));
        let extra_arena = TypedArena::try_new(Some(TypedNodeId::new(5)), extra_nodes)
            .expect("Task269GCU extra-node arena");
        assert_eq!(
            SourceProofLocalGivenConditionUseTermProducer::build(
                fixture.dependency.clone(),
                fixture.input.clone(),
                &extra_arena,
            ),
            Err(SourceProofLocalGivenConditionUseTermError::InvalidInstallation)
        );
        let handoff = SourceProofLocalGivenConditionUseTermProducer::build(
            fixture.dependency,
            fixture.input,
            &fixture.arena,
        )
        .expect("Task269GCU valid handoff");
        let mut dependency_corrupt = handoff.clone();
        dependency_corrupt.set_dependency_fingerprint_for_test("corrupt".to_owned());
        assert_eq!(
            dependency_corrupt.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.arena,
            ),
            Err(SourceProofLocalGivenConditionUseTermError::InvalidDependency)
        );
        let mut term_corrupt = handoff.clone();
        term_corrupt.source_term_mut_for_test().references.rows[0].binding = BindingId::new(0);
        term_corrupt.source_term_fingerprint = term_corrupt.source_term().debug_text();
        assert_eq!(
            term_corrupt.validate_installation(fixture.source, &fixture.module, &fixture.arena),
            Err(SourceProofLocalGivenConditionUseTermError::InvalidSourceTerm)
        );
        let mut source_fingerprint_corrupt = handoff.clone();
        source_fingerprint_corrupt
            .source_term_fingerprint
            .push_str("corrupt");
        assert_eq!(
            source_fingerprint_corrupt.validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.arena,
            ),
            Err(SourceProofLocalGivenConditionUseTermError::InvalidSourceTerm)
        );
        let mut dependency_before_source = term_corrupt.clone();
        dependency_before_source.set_dependency_fingerprint_for_test("corrupt".to_owned());
        assert_eq!(
            dependency_before_source.validate_complete_installation(
                fixture.source,
                &fixture.module,
                &fixture.arena,
                false,
            ),
            Err(SourceProofLocalGivenConditionUseTermError::InvalidDependency)
        );
        assert_eq!(
            term_corrupt.validate_complete_installation(
                fixture.source,
                &fixture.module,
                &fixture.arena,
                false,
            ),
            Err(SourceProofLocalGivenConditionUseTermError::InvalidSourceTerm)
        );
        assert_eq!(
            handoff.validate_complete_installation(
                fixture.source,
                &fixture.module,
                &fixture.arena,
                false,
            ),
            Err(SourceProofLocalGivenConditionUseTermError::InvalidInstallation)
        );
        for (error, text) in [
            (
                SourceProofLocalGivenConditionUseTermError::InvalidDependency,
                "source proof-local given-condition-use term dependency is invalid",
            ),
            (
                SourceProofLocalGivenConditionUseTermError::InvalidSourceTerm,
                "source proof-local given-condition-use source term is invalid",
            ),
            (
                SourceProofLocalGivenConditionUseTermError::InvalidInstallation,
                "source proof-local given-condition-use term installation is invalid",
            ),
        ] {
            assert_eq!(error.to_string(), text);
            let _: &dyn std::error::Error = &error;
        }
    }

    #[test]
    fn task269gcu_typed_and_resolved_ownership_is_atomic() {
        let fixture = task269gcu_fixture();
        let handoff = SourceProofLocalGivenConditionUseTermProducer::build(
            fixture.dependency.clone(),
            fixture.input.clone(),
            &fixture.arena,
        )
        .expect("Task269GCU handoff");
        let typed = task269gcu_empty_typed(&fixture, fixture.arena.clone())
            .with_source_proof_local_given_condition_use_term(handoff.clone())
            .expect("Task269GCU typed installation");
        assert_eq!(
            typed.source_proof_local_given_condition_use_term(),
            Some(&handoff)
        );
        assert!(typed.source_proof_local_given_condition_type().is_none());
        assert_eq!(
            typed
                .clone()
                .with_source_proof_local_given_condition_use_term(handoff.clone()),
            Err(TypedAstError::InvalidSourceProofLocalGivenConditionUseTerm)
        );
        let generic = task269gcu_generic_neighbor(&fixture);
        let generic_first = task269gcu_empty_typed(&fixture, fixture.arena.clone())
            .with_source_term(generic.clone())
            .expect("Task269GCU generic-first owner");
        let generic_first_before = generic_first.debug_text();
        assert_eq!(
            generic_first
                .clone()
                .with_source_proof_local_given_condition_use_term(handoff.clone()),
            Err(TypedAstError::InvalidSourceProofLocalGivenConditionUseTerm)
        );
        assert_eq!(generic_first.debug_text(), generic_first_before);
        let typed_before_generic = typed.debug_text();
        assert_eq!(
            typed.clone().with_source_term(generic),
            Err(TypedAstError::InvalidSourceTerm)
        );
        assert_eq!(typed.debug_text(), typed_before_generic);

        macro_rules! assert_task269gcu_sibling_both_orders {
            (
                $sibling:expr,
                $inject:ident,
                $install:ident,
                $sibling_error:expr,
                $source:expr,
                $module:expr,
                $arena:expr
            ) => {{
                let sibling = $sibling;
                let mut sibling_first = task269gcu_empty_typed(&fixture, fixture.arena.clone());
                sibling_first.$inject(sibling.clone());
                let sibling_first_before = sibling_first.debug_text();
                assert_eq!(
                    sibling_first
                        .clone()
                        .with_source_proof_local_given_condition_use_term(handoff.clone()),
                    Err(TypedAstError::InvalidSourceProofLocalGivenConditionUseTerm),
                );
                assert_eq!(sibling_first.debug_text(), sibling_first_before);
                assert!(
                    sibling_first
                        .source_proof_local_given_condition_use_term()
                        .is_none()
                );

                let mut gcu_first = task269gcu_empty_typed_for($source, $module, $arena);
                gcu_first
                    .inject_source_proof_local_given_condition_use_term_for_test(handoff.clone());
                let gcu_first_before = gcu_first.debug_text();
                assert_eq!(gcu_first.clone().$install(sibling), Err($sibling_error),);
                assert_eq!(gcu_first.debug_text(), gcu_first_before);
                assert!(
                    gcu_first
                        .source_proof_local_given_condition_use_term()
                        .is_some()
                );
            }};
        }

        let let_binding = task269gu_let_neighbor();
        let declaration = SourceProofLocalDeclarationHandoff::ownership_sentinel_for_test(
            fixture.source,
            fixture.module.clone(),
            fixture.dependency.binding_env().clone(),
        );
        assert_task269gcu_sibling_both_orders!(
            declaration,
            inject_source_proof_local_declaration_for_test,
            with_source_proof_local_declaration,
            TypedAstError::InvalidSourceProofLocalDeclaration,
            fixture.source,
            fixture.module.clone(),
            fixture.arena.clone()
        );
        assert_task269gcu_sibling_both_orders!(
            let_binding,
            inject_source_proof_local_let_binding_for_test,
            with_source_proof_local_let_binding,
            TypedAstError::InvalidSourceProofLocalLetBinding,
            source_id_for("f7"),
            module("task269gup"),
            TypedArena::try_new(None, Vec::new()).expect("Task269GCU Let-binding arena")
        );
        let (let_type, let_type_arena) = task269gu_let_type_neighbor();
        assert_task269gcu_sibling_both_orders!(
            let_type,
            inject_source_proof_local_let_type_for_test,
            with_source_proof_local_let_type,
            TypedAstError::InvalidSourceProofLocalLetType,
            source_id_for("f7"),
            module("task269gup"),
            let_type_arena
        );
        let given_binding = task269gu_given_neighbor();
        assert_task269gcu_sibling_both_orders!(
            given_binding,
            inject_source_proof_local_given_binding_for_test,
            with_source_proof_local_given_binding,
            TypedAstError::InvalidSourceProofLocalGivenBinding,
            source_id_for("f7"),
            module("task269gup"),
            TypedArena::try_new(None, Vec::new()).expect("Task269GCU Given-binding arena")
        );
        let (given_type, given_type_arena) = task269gu_given_type_neighbor();
        assert_task269gcu_sibling_both_orders!(
            given_type,
            inject_source_proof_local_given_type_for_test,
            with_source_proof_local_given_type,
            TypedAstError::InvalidSourceProofLocalGivenType,
            source_id_for("f7"),
            module("task269gup"),
            given_type_arena
        );
        let gu_fixture = task269gu_fixture();
        let gu_type = gu_fixture.dependency.clone();
        assert_task269gcu_sibling_both_orders!(
            gu_type,
            inject_source_proof_local_given_use_type_for_test,
            with_source_proof_local_given_use_type,
            TypedAstError::InvalidSourceProofLocalGivenUseType,
            gu_fixture.source,
            gu_fixture.module.clone(),
            task269gupt_arena(gu_fixture.source)
        );
        let gu_term = SourceProofLocalGivenUseTermProducer::build(
            gu_fixture.dependency,
            gu_fixture.input,
            &gu_fixture.arena,
        )
        .expect("Task269GCU GU-term sibling");
        assert_task269gcu_sibling_both_orders!(
            gu_term,
            inject_source_proof_local_given_use_term_for_test,
            with_source_proof_local_given_use_term,
            TypedAstError::InvalidSourceProofLocalGivenUseTerm,
            gu_fixture.source,
            gu_fixture.module,
            gu_fixture.arena
        );
        assert_task269gcu_sibling_both_orders!(
            fixture.dependency.dependency().clone(),
            inject_source_proof_local_given_condition_binding_for_test,
            with_source_proof_local_given_condition_binding,
            TypedAstError::InvalidSourceProofLocalGivenConditionBinding,
            fixture.source,
            fixture.module.clone(),
            TypedArena::try_new(None, Vec::new()).expect("Task269GCU GC-binding arena")
        );
        assert_task269gcu_sibling_both_orders!(
            fixture.dependency.clone(),
            inject_source_proof_local_given_condition_type_for_test,
            with_source_proof_local_given_condition_type,
            TypedAstError::InvalidSourceProofLocalGivenConditionType,
            fixture.source,
            fixture.module.clone(),
            task269gcu_type_arena(fixture.source)
        );
        assert_task269gcu_sibling_both_orders!(
            task269sdc_neighbor(),
            inject_source_proof_local_given_descendant_binding_for_test,
            with_source_proof_local_given_descendant_binding,
            TypedAstError::InvalidSourceProofLocalGivenDescendantBinding,
            source_id_for("f8"),
            module("task269sdc"),
            TypedArena::try_new(None, Vec::new()).expect("Task269SDC ownership arena")
        );

        macro_rules! assert_task269sdc_sibling_both_orders {
            (
                $sibling:expr,
                $inject:ident,
                $install:ident,
                $getter:ident,
                $sibling_error:expr,
                $resolved_error:expr,
                $source:expr,
                $module:expr,
                $arena:expr
            ) => {{
                let sibling_source = $source;
                let sibling_module = $module;
                let sdc = task269sdc_neighbor_for(sibling_source, sibling_module.clone());
                let sibling = $sibling;
                let sibling_arena = $arena;
                let sdc_source = sdc.source_id();
                let sdc_module = sdc.module_id().clone();
                let mut sibling_first = task269gcu_empty_typed_for(
                    sdc_source,
                    sdc_module.clone(),
                    TypedArena::try_new(None, Vec::new())
                        .expect("Task269SDC predecessor-first arena"),
                );
                sibling_first.$inject(sibling.clone());
                let sibling_first_before = sibling_first.debug_text();
                assert_eq!(
                    sibling_first
                        .clone()
                        .with_source_proof_local_given_descendant_binding(sdc.clone()),
                    Err(TypedAstError::InvalidSourceProofLocalGivenDescendantBinding),
                );
                assert_eq!(sibling_first.debug_text(), sibling_first_before);
                assert!(
                    sibling_first
                        .source_proof_local_given_descendant_binding()
                        .is_none()
                );
                assert!(sibling_first.$getter().is_some());

                let mut sdc_first = task269gcu_empty_typed_for(
                    sibling_source,
                    sibling_module,
                    sibling_arena.clone(),
                );
                sdc_first.inject_source_proof_local_given_descendant_binding_for_test(sdc.clone());
                let sdc_first_before = sdc_first.debug_text();
                assert_eq!(
                    sdc_first.clone().$install(sibling.clone()),
                    Err($sibling_error)
                );
                assert_eq!(sdc_first.debug_text(), sdc_first_before);
                assert!(
                    sdc_first
                        .source_proof_local_given_descendant_binding()
                        .is_some()
                );
                assert!(sdc_first.$getter().is_none());

                let mut final_collision =
                    task269gcu_empty_typed_for(sdc_source, sdc_module, sibling_arena);
                final_collision.$inject(sibling);
                final_collision.inject_source_proof_local_given_descendant_binding_for_test(sdc);
                let collision_before = final_collision.debug_text();
                assert_eq!(
                    task269gcu_resolved_result_with_inputs(
                        &final_collision,
                        Vec::new(),
                        Vec::new(),
                    ),
                    Err($resolved_error),
                );
                assert_eq!(final_collision.debug_text(), collision_before);
            }};
        }

        let sdc = task269sdc_neighbor();
        let declaration = SourceProofLocalDeclarationHandoff::ownership_sentinel_for_test(
            sdc.source_id(),
            sdc.module_id().clone(),
            sdc.binding_env().clone(),
        );
        assert_task269sdc_sibling_both_orders!(
            declaration,
            inject_source_proof_local_declaration_for_test,
            with_source_proof_local_declaration,
            source_proof_local_declaration,
            TypedAstError::InvalidSourceProofLocalDeclaration,
            ResolvedTypedAstError::InvalidSourceProofLocalGivenDescendantBinding,
            source_id_for("f8"),
            module("task269sdc"),
            TypedArena::try_new(None, Vec::new()).expect("Task269SDC declaration arena")
        );
        assert_task269sdc_sibling_both_orders!(
            task269gu_let_neighbor(),
            inject_source_proof_local_let_binding_for_test,
            with_source_proof_local_let_binding,
            source_proof_local_let_binding,
            TypedAstError::InvalidSourceProofLocalLetBinding,
            ResolvedTypedAstError::InvalidSourceProofLocalGivenDescendantBinding,
            source_id_for("f7"),
            module("task269gup"),
            TypedArena::try_new(None, Vec::new()).expect("Task269SDC Let-binding arena")
        );
        let (let_type, let_type_arena) = task269gu_let_type_neighbor();
        assert_task269sdc_sibling_both_orders!(
            let_type,
            inject_source_proof_local_let_type_for_test,
            with_source_proof_local_let_type,
            source_proof_local_let_type,
            TypedAstError::InvalidSourceProofLocalLetType,
            ResolvedTypedAstError::InvalidSourceProofLocalGivenDescendantBinding,
            source_id_for("f7"),
            module("task269gup"),
            let_type_arena
        );
        assert_task269sdc_sibling_both_orders!(
            task269gu_given_neighbor(),
            inject_source_proof_local_given_binding_for_test,
            with_source_proof_local_given_binding,
            source_proof_local_given_binding,
            TypedAstError::InvalidSourceProofLocalGivenBinding,
            ResolvedTypedAstError::InvalidSourceProofLocalGivenDescendantBinding,
            source_id_for("f7"),
            module("task269gup"),
            TypedArena::try_new(None, Vec::new()).expect("Task269SDC Given-binding arena")
        );
        let (given_type, given_type_arena) = task269gu_given_type_neighbor();
        assert_task269sdc_sibling_both_orders!(
            given_type,
            inject_source_proof_local_given_type_for_test,
            with_source_proof_local_given_type,
            source_proof_local_given_type,
            TypedAstError::InvalidSourceProofLocalGivenType,
            ResolvedTypedAstError::InvalidSourceProofLocalGivenDescendantBinding,
            source_id_for("f7"),
            module("task269gup"),
            given_type_arena
        );
        let gu_fixture = task269gu_fixture();
        assert_task269sdc_sibling_both_orders!(
            gu_fixture.dependency.clone(),
            inject_source_proof_local_given_use_type_for_test,
            with_source_proof_local_given_use_type,
            source_proof_local_given_use_type,
            TypedAstError::InvalidSourceProofLocalGivenUseType,
            ResolvedTypedAstError::InvalidSourceProofLocalGivenDescendantBinding,
            gu_fixture.source,
            gu_fixture.module.clone(),
            task269gupt_arena(gu_fixture.source)
        );
        let gu_term = SourceProofLocalGivenUseTermProducer::build(
            gu_fixture.dependency,
            gu_fixture.input,
            &gu_fixture.arena,
        )
        .expect("Task269SDC GU-term predecessor");
        assert_task269sdc_sibling_both_orders!(
            gu_term,
            inject_source_proof_local_given_use_term_for_test,
            with_source_proof_local_given_use_term,
            source_proof_local_given_use_term,
            TypedAstError::InvalidSourceProofLocalGivenUseTerm,
            ResolvedTypedAstError::InvalidSourceProofLocalGivenDescendantBinding,
            gu_fixture.source,
            gu_fixture.module,
            gu_fixture.arena
        );
        assert_task269sdc_sibling_both_orders!(
            fixture.dependency.dependency().clone(),
            inject_source_proof_local_given_condition_binding_for_test,
            with_source_proof_local_given_condition_binding,
            source_proof_local_given_condition_binding,
            TypedAstError::InvalidSourceProofLocalGivenConditionBinding,
            ResolvedTypedAstError::InvalidSourceProofLocalGivenDescendantBinding,
            fixture.source,
            fixture.module.clone(),
            TypedArena::try_new(None, Vec::new()).expect("Task269SDC GC-binding arena")
        );
        assert_task269sdc_sibling_both_orders!(
            fixture.dependency.clone(),
            inject_source_proof_local_given_condition_type_for_test,
            with_source_proof_local_given_condition_type,
            source_proof_local_given_condition_type,
            TypedAstError::InvalidSourceProofLocalGivenConditionType,
            ResolvedTypedAstError::InvalidSourceProofLocalGivenDescendantBinding,
            fixture.source,
            fixture.module.clone(),
            task269gcu_type_arena(fixture.source)
        );
        assert_task269sdc_sibling_both_orders!(
            handoff.clone(),
            inject_source_proof_local_given_condition_use_term_for_test,
            with_source_proof_local_given_condition_use_term,
            source_proof_local_given_condition_use_term,
            TypedAstError::InvalidSourceProofLocalGivenConditionUseTerm,
            ResolvedTypedAstError::InvalidSourceProofLocalGivenDescendantBinding,
            fixture.source,
            fixture.module.clone(),
            fixture.arena.clone()
        );
        assert_eq!(
            TypedAstError::InvalidSourceProofLocalGivenConditionUseTerm.to_string(),
            "source proof-local given-condition-use term handoff is invalid"
        );
        assert_eq!(
            ResolvedTypedAstError::InvalidSourceProofLocalGivenConditionUseTerm.to_string(),
            "resolved typed AST source proof-local given-condition-use term handoff is invalid"
        );
        let resolved = task269gu_resolved(&typed);
        assert_eq!(
            resolved.source_proof_local_given_condition_use_term(),
            Some(&handoff)
        );
        assert_eq!(resolved.nodes().len(), 6);
        for (_, node) in resolved.nodes().iter() {
            assert!(matches!(
                &node.kind,
                ResolvedTypedNodeKind::SourcePreserved { role }
                    if role.as_str() == "source.proof-local.given-condition.term"
            ));
        }
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
        assert!(resolved.checked_formulas().is_empty());
        assert!(resolved.statement_semantics().is_empty());
        assert!(resolved.checked_proofs().is_empty());
        assert!(resolved.checked_proof_nodes().is_empty());
        assert!(resolved.checked_terminal_goals().is_empty());
        assert!(resolved.diagnostics().is_empty());
        let mut conflicting = typed.clone();
        conflicting
            .inject_source_proof_local_given_condition_type_for_test(fixture.dependency.clone());
        assert!(matches!(
            task269gcu_resolved_result_with_inputs(&conflicting, Vec::new(), Vec::new()),
            Err(ResolvedTypedAstError::InvalidSourceProofLocalGivenConditionType)
                | Err(ResolvedTypedAstError::InvalidSourceProofLocalGivenConditionUseTerm)
        ));
        assert_eq!(
            task269gcu_resolved_result_with_inputs(
                &typed,
                Vec::new(),
                vec![ResolvedNodeKindHint {
                    typed_node: TypedNodeId::new(0),
                    kind: ResolvedNodeKindHintKind::SourcePreserved {
                        role: SourceNodeRole::new("source.statement.transport"),
                    },
                }],
            ),
            Err(ResolvedTypedAstError::InvalidSourceProofLocalGivenConditionUseTerm)
        );
        assert_eq!(
            task269gcu_resolved_result_with_inputs(
                &typed,
                vec![ExpressionMetadataInput {
                    expr: ExprId::new("task269gcu.semantic-input"),
                    typed_site: TypedSiteRef::Node(TypedNodeId::new(3)),
                    local_context: None,
                    cluster_facts: Vec::new(),
                }],
                Vec::new(),
            ),
            Err(ResolvedTypedAstError::InvalidSourceProofLocalGivenConditionUseTerm)
        );
        let mut injected = task269gcu_empty_typed(&fixture, fixture.arena.clone());
        injected.inject_source_proof_local_given_condition_use_term_for_test(handoff);
        assert_eq!(task269gu_resolved(&injected), resolved);
    }

    #[test]
    fn task269gcu_generic_and_neighbor_routes_remain_isolated() {
        let fixture = task269gcu_fixture();
        assert!(matches!(
            SourcePrimaryTermProducer::build(
                fixture.input.clone(),
                fixture.dependency.binding_env(),
                &fixture.arena,
            ),
            Err(SourcePrimaryTermError::InvalidReference { .. })
        ));
        assert!(
            SourceProofLocalGivenConditionUseTermProducer::build(
                fixture.dependency.clone(),
                fixture.input.clone(),
                &fixture.arena,
            )
            .is_ok()
        );
        let mut parent_context = fixture.input.clone();
        parent_context.terms[0].context = BindingContextId::new(0);
        assert_eq!(
            SourceProofLocalGivenConditionUseTermProducer::build(
                fixture.dependency.clone(),
                parent_context,
                &fixture.arena,
            ),
            Err(SourceProofLocalGivenConditionUseTermError::InvalidSourceTerm)
        );
        let typed = task269gcu_empty_typed(&fixture, fixture.arena.clone())
            .with_source_proof_local_given_condition_use_term(
                SourceProofLocalGivenConditionUseTermProducer::build(
                    fixture.dependency,
                    fixture.input,
                    &fixture.arena,
                )
                .expect("Task269GCU exact neighbor-isolation handoff"),
            )
            .expect("Task269GCU exact neighbor-isolation owner");
        assert!(typed.source_proof_local_given_use_term().is_none());
        assert!(typed.source_term().is_none());
        assert!(build(&self::fixture()).is_ok());
    }

    #[test]
    fn dense_rows_accessors_and_debug_cover_every_task252_kind() {
        let fixture = fixture();
        let handoff = build(&fixture).expect("valid handoff");
        assert_eq!(handoff.source_id(), fixture.source);
        assert_eq!(handoff.module_id(), &fixture.module);
        assert_eq!(handoff.terms().len(), 6);
        assert_eq!(handoff.references().len(), 3);
        assert_eq!(handoff.numeric_type_requests().len(), 1);
        assert!(!handoff.terms().is_empty());

        let kinds = handoff
            .terms()
            .iter()
            .map(|(id, row)| {
                assert_eq!(id.index(), row.source_ordinal());
                assert_eq!(row.context(), BindingContextId::new(0));
                assert_eq!(row.recovery(), SourcePrimaryTermRecovery::Normal);
                assert_eq!(row.site(), &node(id.index()));
                row.kind()
            })
            .collect::<Vec<_>>();
        assert_eq!(
            kinds,
            [
                SourcePrimaryTermKind::Parenthesized,
                SourcePrimaryTermKind::VariableReference,
                SourcePrimaryTermKind::VariableReference,
                SourcePrimaryTermKind::Numeral,
                SourcePrimaryTermKind::It,
                SourcePrimaryTermKind::ConstantReference,
            ]
        );
        assert_eq!(
            handoff
                .terms()
                .get(SourcePrimaryTermId::new(1))
                .expect("child")
                .parent(),
            Some(SourcePrimaryTermId::new(0))
        );
        assert_eq!(
            handoff
                .terms()
                .get(SourcePrimaryTermId::new(5))
                .expect("constant")
                .spelling(),
            "c"
        );
        for (id, reference) in handoff.references().iter() {
            assert_eq!(
                id.index(),
                [
                    SourcePrimaryTermId::new(1),
                    SourcePrimaryTermId::new(2),
                    SourcePrimaryTermId::new(5)
                ]
                .iter()
                .position(|term| *term == reference.term())
                .expect("known reference term")
            );
            assert_eq!(reference.use_ordinal(), 2);
            assert_eq!(
                reference.lexical_scope().map(LocalTermScope::path),
                Some(&[0][..])
            );
        }
        let request = handoff
            .numeric_type_requests()
            .get(SourceNumericTypeRequestId::new(0))
            .expect("numeric request");
        assert_eq!(request.term(), SourcePrimaryTermId::new(3));
        assert_eq!(request.owner(), &node(3));
        assert_eq!(request.source_range(), range(fixture.source, 30, 31));
        assert_eq!(request.spelling(), "1");
        assert_eq!(request.request_ordinal(), 0);

        let debug = handoff.debug_text();
        assert_eq!(debug, build(&fixture).expect("replay").debug_text());
        assert!(debug.starts_with("source-primary-term-debug-v1\n"));
        assert!(debug.contains("kind=parenthesized"));
        assert!(debug.contains("role=current-definition-result"));
        assert!(debug.contains("role=local-constant"));
        assert!(debug.contains("numeric-request#0 term=3 ordinal=0"));
    }

    #[test]
    fn environment_site_range_kind_recovery_context_and_spelling_corruption_fail_closed() {
        let fixture = fixture();

        let mut cases = Vec::new();
        let mut changed = fixture.clone();
        changed.input.source_id = other_source_id();
        cases.push(changed);
        let mut changed = fixture.clone();
        changed.input.module_id = module("wrong");
        cases.push(changed);
        let mut changed = fixture.clone();
        changed.input.terms[2].source_ordinal = 9;
        cases.push(changed);
        let mut changed = fixture.clone();
        changed.input.terms[2].source_range.end = changed.input.terms[2].source_range.start;
        cases.push(changed);
        let mut changed = fixture.clone();
        changed.input.terms[2].site = TypedSiteRef::Role {
            node: TypedNodeId::new(2),
            role: TypeRole::new("term"),
        };
        cases.push(changed);
        let mut changed = fixture.clone();
        changed.input.terms[2].context = BindingContextId::new(9);
        cases.push(changed);
        let mut changed = fixture.clone();
        changed.input.terms[4].spelling = "IT".to_owned();
        cases.push(changed);
        let mut changed = fixture.clone();
        changed.input.terms[3].spelling = "1a".to_owned();
        cases.push(changed);
        let mut changed = fixture.clone();
        changed.input.terms[4].role = SourcePrimaryTermRole::Value;
        cases.push(changed);

        for changed in cases {
            assert!(build(&changed).is_err());
        }

        let mut wrong_kind = fixture.clone();
        let mut nodes = arena_nodes_for(&wrong_kind.input.terms);
        nodes[2] = TypedNode::new(
            "source.term.constant-reference",
            SourceAnchor::Range(range(fixture.source, 20, 21)),
        );
        wrong_kind.arena = TypedArena::try_new(None, nodes).expect("wrong-kind arena");
        assert!(build(&wrong_kind).is_err());

        let mut wrong_recovery = fixture.clone();
        wrong_recovery.input.terms[2].recovery = SourcePrimaryTermRecovery::Degraded;
        assert!(build(&wrong_recovery).is_err());

        for malformed in ["1", "x y", "'x", "x-", "it", "theorem"] {
            let mut malformed_reference = fixture.clone();
            malformed_reference.input.terms[1].spelling = malformed.to_owned();
            malformed_reference.input.terms[2].spelling = malformed.to_owned();
            replace_bindings(
                &mut malformed_reference,
                vec![
                    reserved_binding(fixture.source, malformed, 0, 0),
                    local_binding(fixture.source, "c", 2, 1, BindingKind::LocalAbbreviation),
                ],
                Some(vec![0]),
            );
            assert!(matches!(
                build(&malformed_reference),
                Err(SourcePrimaryTermError::InvalidTerm { term })
                    if term == SourcePrimaryTermId::new(1)
            ));
        }

        let mut degraded = fixture.clone();
        degraded.input.terms[2].recovery = SourcePrimaryTermRecovery::Degraded;
        let mut nodes = arena_nodes_for(&degraded.input.terms);
        nodes[2].recovery = NodeRecoveryState::Recovered;
        degraded.arena = TypedArena::try_new(None, nodes).expect("recovered arena");
        assert!(build(&degraded).is_ok());
    }

    #[test]
    fn reference_cardinality_roles_binding_kinds_and_winners_are_authenticated() {
        let fixture = fixture();
        let mut missing = fixture.clone();
        missing.input.references.remove(0);
        assert!(build(&missing).is_err());

        let mut duplicate = fixture.clone();
        duplicate
            .input
            .references
            .insert(1, duplicate.input.references[0].clone());
        assert!(build(&duplicate).is_err());

        let mut wrong_role = fixture.clone();
        wrong_role.input.references[0].role = SourcePrimaryTermReferenceRole::LocalConstant;
        assert!(build(&wrong_role).is_err());

        let mut wrong_binding = fixture.clone();
        wrong_binding.input.references[0].binding = BindingId::new(1);
        assert!(build(&wrong_binding).is_err());

        let mut wrong_kind = fixture.clone();
        replace_bindings(
            &mut wrong_kind,
            vec![
                local_binding(fixture.source, "x", 0, 0, BindingKind::LocalAbbreviation),
                local_binding(fixture.source, "c", 2, 1, BindingKind::LocalAbbreviation),
            ],
            Some(vec![0]),
        );
        assert!(build(&wrong_kind).is_err());

        let mut different_winner = fixture.clone();
        replace_bindings(
            &mut different_winner,
            vec![
                reserved_binding(fixture.source, "x", 0, 0),
                reserved_binding(fixture.source, "x", 2, 1),
            ],
            Some(vec![0]),
        );
        different_winner.input.references.truncate(2);
        different_winner.input.terms.truncate(5);
        different_winner.input.numeric_type_requests.truncate(1);
        different_winner.arena = arena_for(&different_winner.input.terms);
        assert!(build(&different_winner).is_err());
    }

    #[test]
    fn numeric_request_cardinality_association_and_order_are_atomic() {
        let fixture = fixture();
        let mut missing = fixture.clone();
        missing.input.numeric_type_requests.clear();
        assert!(build(&missing).is_err());

        let mut duplicate = fixture.clone();
        duplicate
            .input
            .numeric_type_requests
            .push(duplicate.input.numeric_type_requests[0].clone());
        assert!(build(&duplicate).is_err());

        let mut wrong_term = fixture.clone();
        wrong_term.input.numeric_type_requests[0].term = SourcePrimaryTermId::new(2);
        assert!(build(&wrong_term).is_err());

        let mut wrong_owner = fixture.clone();
        wrong_owner.input.numeric_type_requests[0].owner = node(2);
        assert!(build(&wrong_owner).is_err());

        let mut wrong_range = fixture.clone();
        wrong_range.input.numeric_type_requests[0].source_range.end += 1;
        assert!(build(&wrong_range).is_err());

        let mut wrong_spelling = fixture.clone();
        wrong_spelling.input.numeric_type_requests[0].spelling = "01".to_owned();
        assert!(build(&wrong_spelling).is_err());

        let mut wrong_ordinal = fixture.clone();
        wrong_ordinal.input.numeric_type_requests[0].request_ordinal = 1;
        assert!(build(&wrong_ordinal).is_err());

        let source = source_id_for("d4");
        let module = module("source.term.numeric-order");
        let terms = vec![
            term(source, 0, 10, 11, "1", SourcePrimaryTermKind::Numeral, None),
            term(source, 1, 20, 21, "2", SourcePrimaryTermKind::Numeral, None),
        ];
        let mut ordered = Fixture {
            source,
            module: module.clone(),
            input: SourcePrimaryTermHandoffInput {
                source_id: source,
                module_id: module.clone(),
                terms: terms.clone(),
                references: Vec::new(),
                numeric_type_requests: vec![
                    SourceNumericTypeRequestInput {
                        term: SourcePrimaryTermId::new(0),
                        owner: node(0),
                        source_range: range(source, 10, 11),
                        spelling: "1".to_owned(),
                        request_ordinal: 0,
                    },
                    SourceNumericTypeRequestInput {
                        term: SourcePrimaryTermId::new(1),
                        owner: node(1),
                        source_range: range(source, 20, 21),
                        spelling: "2".to_owned(),
                        request_ordinal: 1,
                    },
                ],
            },
            bindings: binding_env(source, &module, Vec::new(), None),
            arena: arena_for(&terms),
        };
        let handoff = build(&ordered).expect("two numeric requests in source order");
        assert_eq!(
            handoff
                .numeric_type_requests()
                .iter()
                .map(|(_, request)| (request.term().index(), request.request_ordinal()))
                .collect::<Vec<_>>(),
            [(0, 0), (1, 1)]
        );
        ordered.input.numeric_type_requests.swap(0, 1);
        for (request_ordinal, request) in ordered.input.numeric_type_requests.iter_mut().enumerate()
        {
            request.request_ordinal = request_ordinal;
        }
        assert!(matches!(
            build(&ordered),
            Err(SourcePrimaryTermError::InvalidNumericTypeRequest { request })
                if request == SourceNumericTypeRequestId::new(1)
        ));
    }

    #[test]
    fn parent_graph_preorder_containment_context_and_ownership_are_enforced() {
        let fixture = fixture();
        let mut dangling = fixture.clone();
        dangling.input.terms[1].parent = Some(SourcePrimaryTermId::new(9));
        assert!(build(&dangling).is_err());

        let mut forward = fixture.clone();
        forward.input.terms[0].parent = Some(SourcePrimaryTermId::new(1));
        assert!(build(&forward).is_err());

        let mut non_parent = fixture.clone();
        non_parent.input.terms[2].parent = Some(SourcePrimaryTermId::new(1));
        assert!(build(&non_parent).is_err());

        let mut second_child = fixture.clone();
        second_child.input.terms[2].parent = Some(SourcePrimaryTermId::new(0));
        second_child.input.terms[2].source_range = range(fixture.source, 12, 13);
        let mut nodes = arena_nodes_for(&second_child.input.terms);
        nodes[2].anchor = SourceAnchor::Range(range(fixture.source, 12, 13));
        second_child.arena = TypedArena::try_new(None, nodes).expect("second-child arena");
        assert!(build(&second_child).is_err());

        let mut containment = fixture.clone();
        containment.input.terms[1].source_range = range(fixture.source, 15, 16);
        let mut nodes = arena_nodes_for(&containment.input.terms);
        nodes[1].anchor = SourceAnchor::Range(range(fixture.source, 15, 16));
        containment.arena = TypedArena::try_new(None, nodes).expect("containment arena");
        assert!(build(&containment).is_err());

        let mut spelling = fixture.clone();
        spelling.input.terms[0].spelling = "(  x  )".to_owned();
        assert!(build(&spelling).is_err());

        let mut missing_child = fixture.clone();
        missing_child.input.terms[1].parent = None;
        assert!(build(&missing_child).is_err());

        let mut crossed_context = fixture.clone();
        crossed_context.bindings = binding_env_with_child_context(
            fixture.source,
            &fixture.module,
            vec![
                reserved_binding(fixture.source, "x", 0, 0),
                local_binding(fixture.source, "c", 2, 1, BindingKind::LocalAbbreviation),
            ],
        );
        crossed_context.input.terms[1].context = BindingContextId::new(1);
        assert!(matches!(
            build(&crossed_context),
            Err(SourcePrimaryTermError::InvalidTerm { term })
                if term == SourcePrimaryTermId::new(1)
        ));
    }

    #[test]
    fn roots_siblings_references_and_overlaps_must_stay_in_source_order() {
        let fixture = fixture();
        let mut reordered_root = fixture.clone();
        reordered_root.input.terms[2].source_range = range(fixture.source, 5, 6);
        let mut nodes = arena_nodes_for(&reordered_root.input.terms);
        nodes[2].anchor = SourceAnchor::Range(range(fixture.source, 5, 6));
        reordered_root.arena = TypedArena::try_new(None, nodes).expect("reordered arena");
        assert!(build(&reordered_root).is_err());

        let mut overlapping_roots = fixture.clone();
        overlapping_roots.input.terms[2].source_range = range(fixture.source, 14, 16);
        let mut nodes = arena_nodes_for(&overlapping_roots.input.terms);
        nodes[2].anchor = SourceAnchor::Range(range(fixture.source, 14, 16));
        overlapping_roots.arena = TypedArena::try_new(None, nodes).expect("overlap arena");
        assert!(build(&overlapping_roots).is_err());

        let mut reordered_references = fixture.clone();
        reordered_references.input.references.swap(0, 1);
        assert!(build(&reordered_references).is_err());
    }

    #[test]
    fn binding_ordinals_count_only_preceding_declarations_across_interleaved_uses() {
        let source = source_id();
        let module = module("source.term.interleaved");
        let terms = vec![
            term(
                source,
                0,
                10,
                11,
                "x",
                SourcePrimaryTermKind::VariableReference,
                None,
            ),
            term(
                source,
                1,
                20,
                21,
                "x",
                SourcePrimaryTermKind::VariableReference,
                None,
            ),
            term(
                source,
                2,
                40,
                41,
                "y",
                SourcePrimaryTermKind::VariableReference,
                None,
            ),
        ];
        let fixture = Fixture {
            source,
            module: module.clone(),
            input: SourcePrimaryTermHandoffInput {
                source_id: source,
                module_id: module.clone(),
                terms: terms.clone(),
                references: vec![
                    SourcePrimaryTermReferenceInput {
                        term: SourcePrimaryTermId::new(0),
                        binding: BindingId::new(0),
                        role: SourcePrimaryTermReferenceRole::Variable,
                    },
                    SourcePrimaryTermReferenceInput {
                        term: SourcePrimaryTermId::new(1),
                        binding: BindingId::new(0),
                        role: SourcePrimaryTermReferenceRole::Variable,
                    },
                    SourcePrimaryTermReferenceInput {
                        term: SourcePrimaryTermId::new(2),
                        binding: BindingId::new(1),
                        role: SourcePrimaryTermReferenceRole::Variable,
                    },
                ],
                numeric_type_requests: Vec::new(),
            },
            bindings: binding_env(
                source,
                &module,
                vec![
                    reserved_binding(source, "x", 0, 0),
                    reserved_binding(source, "y", 30, 1),
                ],
                None,
            ),
            arena: arena_for(&terms),
        };
        let handoff = build(&fixture).expect("interleaved binding/use transaction");
        assert_eq!(
            handoff
                .references()
                .iter()
                .map(|(_, row)| row.use_ordinal())
                .collect::<Vec<_>>(),
            [1, 1, 2]
        );

        let mut forward = fixture.clone();
        forward.input.terms[1].spelling = "y".to_owned();
        forward.input.references[1].binding = BindingId::new(1);
        assert!(build(&forward).is_err());

        let mut stale_singleton = fixture.clone();
        replace_bindings(
            &mut stale_singleton,
            vec![
                reserved_binding(source, "x", 0, 1),
                reserved_binding(source, "y", 30, 1),
            ],
            None,
        );
        assert!(matches!(
            build(&stale_singleton),
            Err(SourcePrimaryTermError::InvalidBindingEvent { event: 0 })
        ));

        let mut reordered_bindings = fixture.clone();
        replace_bindings(
            &mut reordered_bindings,
            vec![
                reserved_binding(source, "y", 30, 0),
                reserved_binding(source, "x", 0, 1),
            ],
            None,
        );
        assert!(matches!(
            build(&reordered_bindings),
            Err(SourcePrimaryTermError::InvalidBindingEvent { event: 1 })
        ));

        let mut overlapping_bindings = fixture.clone();
        let mut first = reserved_binding(source, "x", 0, 0);
        first.declaration_range = range(source, 0, 31);
        first.identity = BinderIdentity::ReservedVariable {
            spelling: "x".to_owned(),
            declaration_range: first.declaration_range,
        };
        replace_bindings(
            &mut overlapping_bindings,
            vec![first, reserved_binding(source, "y", 30, 1)],
            None,
        );
        assert!(matches!(
            build(&overlapping_bindings),
            Err(SourcePrimaryTermError::InvalidBindingEvent { event: 1 })
        ));
        assert!(!valid_range(source, range(other_source_id(), 0, 1)));
        assert!(!valid_range(source, range(source, 1, 1)));
    }

    #[test]
    fn duplicate_priority_groups_reach_ambiguous_and_group_drift_is_rejected() {
        let mut fixture = fixture();
        let mut first = reserved_binding(fixture.source, "x", 0, 1);
        let mut second = first.clone();
        first.visible_after_ordinal = 1;
        second.visible_after_ordinal = 1;
        replace_bindings(&mut fixture, vec![first.clone(), second.clone()], None);
        fixture.input.terms = vec![term(
            fixture.source,
            0,
            10,
            11,
            "x",
            SourcePrimaryTermKind::VariableReference,
            None,
        )];
        fixture.input.references = vec![SourcePrimaryTermReferenceInput {
            term: SourcePrimaryTermId::new(0),
            binding: BindingId::new(0),
            role: SourcePrimaryTermReferenceRole::Variable,
        }];
        fixture.input.numeric_type_requests.clear();
        fixture.arena = arena_for(&fixture.input.terms);
        assert!(matches!(
            build(&fixture),
            Err(SourcePrimaryTermError::InvalidReference { .. })
        ));

        let mut stale_ordinal = fixture.clone();
        let first = reserved_binding(fixture.source, "x", 0, 0);
        let second = first.clone();
        replace_bindings(&mut stale_ordinal, vec![first, second], None);
        assert!(matches!(
            build(&stale_ordinal),
            Err(SourcePrimaryTermError::InvalidBindingEvent { .. })
        ));

        let mut different_spelling = fixture.clone();
        let first = reserved_binding(fixture.source, "x", 0, 1);
        let second = reserved_binding(fixture.source, "y", 0, 1);
        replace_bindings(&mut different_spelling, vec![first, second], None);
        assert!(matches!(
            build(&different_spelling),
            Err(SourcePrimaryTermError::InvalidBindingEvent { .. })
        ));

        let mut different_kind = fixture.clone();
        let first = local_binding(fixture.source, "x", 0, 1, BindingKind::LetBinding);
        let mut second = first.clone();
        second.kind = BindingKind::QuantifierBinder;
        replace_bindings(&mut different_kind, vec![first, second], Some(vec![0]));
        assert!(matches!(
            build(&different_kind),
            Err(SourcePrimaryTermError::InvalidBindingEvent { event: 1 })
        ));

        let mut different_identity = fixture.clone();
        let first = local_binding(fixture.source, "x", 0, 1, BindingKind::LetBinding);
        let mut second = first.clone();
        second.identity = BinderIdentity::ResolverLocal {
            scope: LocalTermScope::new(vec![1]),
            ordinal: 1,
            declaration_range: second.declaration_range,
        };
        replace_bindings(&mut different_identity, vec![first, second], Some(vec![0]));
        assert!(matches!(
            build(&different_identity),
            Err(SourcePrimaryTermError::InvalidBindingEvent { event: 1 })
        ));

        let mut different_owner = fixture.clone();
        let mut first = local_binding(fixture.source, "x", 0, 1, BindingKind::LetBinding);
        let mut second = first.clone();
        first.identity = BinderIdentity::ResolverLocal {
            scope: LocalTermScope::new(vec![0, 0]),
            ordinal: 1,
            declaration_range: first.declaration_range,
        };
        second.identity = first.identity.clone();
        different_owner.bindings =
            binding_env_with_split_owners(fixture.source, &fixture.module, vec![first, second]);
        different_owner.input.terms = vec![term(
            fixture.source,
            0,
            10,
            11,
            "x",
            SourcePrimaryTermKind::VariableReference,
            None,
        )];
        different_owner.input.terms[0].context = BindingContextId::new(1);
        different_owner.input.references = vec![SourcePrimaryTermReferenceInput {
            term: SourcePrimaryTermId::new(0),
            binding: BindingId::new(0),
            role: SourcePrimaryTermReferenceRole::Variable,
        }];
        different_owner.input.numeric_type_requests.clear();
        different_owner.arena = arena_for(&different_owner.input.terms);
        assert!(matches!(
            build(&different_owner),
            Err(SourcePrimaryTermError::InvalidBindingEvent { event: 1 })
        ));
    }

    #[test]
    fn every_frozen_variable_binding_kind_is_accepted_with_authenticated_scope() {
        let source = source_id_for("e5");
        let module = module("source.term.variable-kinds");
        for kind in [
            BindingKind::LetBinding,
            BindingKind::QuantifierBinder,
            BindingKind::DefinitionParameter,
        ] {
            let terms = vec![term(
                source,
                0,
                10,
                11,
                "x",
                SourcePrimaryTermKind::VariableReference,
                None,
            )];
            let fixture = Fixture {
                source,
                module: module.clone(),
                input: SourcePrimaryTermHandoffInput {
                    source_id: source,
                    module_id: module.clone(),
                    terms: terms.clone(),
                    references: vec![SourcePrimaryTermReferenceInput {
                        term: SourcePrimaryTermId::new(0),
                        binding: BindingId::new(0),
                        role: SourcePrimaryTermReferenceRole::Variable,
                    }],
                    numeric_type_requests: Vec::new(),
                },
                bindings: binding_env(
                    source,
                    &module,
                    vec![local_binding(source, "x", 0, 0, kind)],
                    Some(vec![0]),
                ),
                arena: arena_for(&terms),
            };
            let handoff = build(&fixture)
                .unwrap_or_else(|error| panic!("{kind:?} should be accepted: {error}"));
            let reference = handoff
                .references()
                .get(SourcePrimaryTermReferenceId::new(0))
                .expect("variable reference");
            assert_eq!(reference.role(), SourcePrimaryTermReferenceRole::Variable);
            assert_eq!(reference.binding(), BindingId::new(0));
            assert_eq!(reference.use_ordinal(), 1);
            assert_eq!(
                reference.lexical_scope().map(LocalTermScope::path),
                Some(&[0][..])
            );
        }
    }

    #[test]
    fn forward_missing_scope_unresolved_and_different_local_winners_are_rejected() {
        let mut forward = fixture();
        forward.input.terms = vec![term(
            forward.source,
            0,
            1,
            2,
            "x",
            SourcePrimaryTermKind::VariableReference,
            None,
        )];
        forward.input.references = vec![SourcePrimaryTermReferenceInput {
            term: SourcePrimaryTermId::new(0),
            binding: BindingId::new(0),
            role: SourcePrimaryTermReferenceRole::Variable,
        }];
        forward.input.numeric_type_requests.clear();
        let forward_source = forward.source;
        replace_bindings(
            &mut forward,
            vec![reserved_binding(forward_source, "x", 2, 0)],
            None,
        );
        forward.arena = arena_for(&forward.input.terms);
        assert!(build(&forward).is_err());

        let mut missing_scope = fixture();
        let missing_scope_source = missing_scope.source;
        replace_bindings(
            &mut missing_scope,
            vec![
                local_binding(
                    missing_scope_source,
                    "x",
                    0,
                    0,
                    BindingKind::QuantifierBinder,
                ),
                local_binding(
                    missing_scope_source,
                    "c",
                    2,
                    1,
                    BindingKind::LocalAbbreviation,
                ),
            ],
            None,
        );
        assert!(build(&missing_scope).is_err());

        let mut unresolved = fixture();
        unresolved.input.terms[1].spelling = "missing".to_owned();
        assert!(build(&unresolved).is_err());
    }

    #[test]
    fn eligible_deep_parentheses_form_a_dense_iterative_preorder_tree() {
        let source = source_id_for("c3");
        let module = module("source.term.deep");
        let depth = 64;
        let mut terms = Vec::with_capacity(depth + 1);
        let mut spelling = "7".to_owned();
        let mut spellings = vec![String::new(); depth];
        for index in (0..depth).rev() {
            spelling = format!("( {spelling} )");
            spellings[index] = spelling.clone();
        }
        for (index, spelling) in spellings.into_iter().enumerate() {
            terms.push(term(
                source,
                index,
                index,
                depth * 2 + 1 - index,
                &spelling,
                SourcePrimaryTermKind::Parenthesized,
                if index > 0 { Some(index - 1) } else { None },
            ));
        }
        terms.push(term(
            source,
            depth,
            depth,
            depth + 1,
            "7",
            SourcePrimaryTermKind::Numeral,
            Some(depth - 1),
        ));
        let fixture = Fixture {
            source,
            module: module.clone(),
            input: SourcePrimaryTermHandoffInput {
                source_id: source,
                module_id: module.clone(),
                terms: terms.clone(),
                references: Vec::new(),
                numeric_type_requests: vec![SourceNumericTypeRequestInput {
                    term: SourcePrimaryTermId::new(depth),
                    owner: node(depth),
                    source_range: range(source, depth, depth + 1),
                    spelling: "7".to_owned(),
                    request_ordinal: 0,
                }],
            },
            bindings: binding_env(source, &module, Vec::new(), None),
            arena: arena_for(&terms),
        };
        let handoff = build(&fixture).expect("deep eligible tree");
        assert_eq!(handoff.terms().len(), depth + 1);
        assert_eq!(
            handoff
                .terms()
                .get(SourcePrimaryTermId::new(depth))
                .expect("leaf")
                .parent(),
            Some(SourcePrimaryTermId::new(depth - 1))
        );
    }

    #[test]
    fn typed_ast_installation_revalidates_arena_and_rejects_replacement() {
        let fixture = fixture();
        let handoff = build(&fixture).expect("handoff");
        let typed = TypedAst::try_new(empty_typed_parts(&fixture))
            .expect("typed AST")
            .with_source_term(handoff.clone())
            .expect("source term installation");
        assert_eq!(typed.source_term(), Some(&handoff));
        assert!(
            typed
                .debug_text()
                .contains("source-primary-term-debug-v1\n")
        );
        assert!(matches!(
            typed.clone().with_source_term(handoff.clone()),
            Err(TypedAstError::InvalidSourceTerm)
        ));

        let mut wrong_arena = fixture.clone();
        let mut nodes = arena_nodes_for(&wrong_arena.input.terms);
        nodes[0].anchor = SourceAnchor::Range(range(fixture.source, 9, 15));
        wrong_arena.arena = TypedArena::try_new(None, nodes).expect("wrong arena");
        let wrong_parts = empty_typed_parts(&wrong_arena);
        assert!(matches!(
            TypedAst::try_new(wrong_parts)
                .expect("legacy typed AST")
                .with_source_term(handoff),
            Err(TypedAstError::InvalidSourceTerm)
        ));
    }

    #[test]
    fn production_boundary_stays_syntax_free_and_has_no_semantic_result_payloads() {
        let source = include_str!("source_term.rs");
        let production = &source[..source.find("#[cfg(test)]").expect("test module")];
        for forbidden in [
            "mizar_syntax",
            "SurfaceAst",
            "SurfaceNodeId",
            "SyntaxKind",
            "NormalizedType",
            "TypeFact",
            "CheckedFormula",
            "Fol",
            "Axiom",
            "numeric_type_result",
        ] {
            assert!(
                !production.contains(forbidden),
                "production source term transport must not contain {forbidden}"
            );
        }
    }

    #[test]
    fn task269sdu_exact_descendant_occurrence_reference_profile_is_frozen() {
        let fixture = task269sdu_fixture();
        let handoff = SourceProofLocalGivenDescendantUseTermProducer::build(
            fixture.dependency.clone(),
            task269sdu_input(fixture.source, fixture.module.clone()),
            &fixture.arena,
        )
        .expect("Task269SDU exact handoff");
        assert_eq!(handoff.source_id(), fixture.source);
        assert_eq!(handoff.module_id(), &fixture.module);
        assert_eq!(handoff.dependency(), &fixture.dependency);
        assert_eq!(
            handoff.dependency_fingerprint(),
            handoff.dependency().debug_text()
        );
        assert_eq!(
            handoff.source_term_fingerprint(),
            handoff.source_term().debug_text()
        );
        let source_term = handoff.source_term();
        assert_eq!(
            (
                source_term.terms().len(),
                source_term.references().len(),
                source_term.numeric_type_requests().len()
            ),
            (1, 1, 0)
        );
        let term = source_term
            .terms()
            .get(SourcePrimaryTermId::new(0))
            .expect("Task269SDU exact term");
        assert_eq!(
            (
                term.source_range().start,
                term.source_range().end,
                term.source_ordinal(),
                term.context().index(),
                term.spelling()
            ),
            (118, 119, 0, 2, "y")
        );
        assert_eq!(term.kind(), SourcePrimaryTermKind::VariableReference);
        assert_eq!(term.role(), SourcePrimaryTermRole::Value);
        assert_eq!(term.parent(), None);
        let reference = source_term
            .references()
            .get(SourcePrimaryTermReferenceId::new(0))
            .expect("Task269SDU exact reference");
        assert_eq!(
            (
                reference.term().index(),
                reference.binding().index(),
                reference.role()
            ),
            (0, 1, SourcePrimaryTermReferenceRole::Variable)
        );
        assert_eq!(
            reference.lexical_scope().map(LocalTermScope::path),
            Some(&[0, 0][..])
        );
        assert_eq!(reference.use_ordinal(), 2);
        assert_eq!(fixture.arena.root(), Some(TypedNodeId::new(4)));

        let replay = SourceProofLocalGivenDescendantUseTermProducer::build(
            fixture.dependency.clone(),
            task269sdu_input(fixture.source, fixture.module.clone()),
            &fixture.arena,
        )
        .expect("Task269SDU exact replay");
        assert_eq!(replay, handoff);
        assert_eq!(replay.debug_text(), handoff.debug_text());

        let typed = task269gcu_empty_typed_for(
            fixture.source,
            fixture.module.clone(),
            fixture.arena.clone(),
        )
        .with_source_proof_local_given_descendant_use_term(handoff.clone())
        .expect("Task269SDU Typed owner");
        assert!(
            typed
                .debug_text()
                .contains("source-proof-local-given-descendant-use-term-debug-v1")
        );
        let resolved = task269gcu_resolved_result_with_inputs(&typed, Vec::new(), Vec::new())
            .expect("Task269SDU Resolved owner");
        assert_eq!(
            resolved.source_proof_local_given_descendant_use_term(),
            Some(&handoff)
        );
        assert!(
            resolved
                .debug_text()
                .contains("source-proof-local-given-descendant-use-term-debug-v1")
        );
    }

    #[test]
    fn task269sdu_input_and_arena_corruption_are_rejected() {
        let fixture = task269sdu_fixture();
        let exact = SourceProofLocalGivenDescendantUseTermProducer::build(
            fixture.dependency.clone(),
            task269sdu_input(fixture.source, fixture.module.clone()),
            &fixture.arena,
        )
        .expect("Task269SDU exact handoff");

        let mut wrong_dependency = task269sdu_input(fixture.source, module("task269sdu-wrong"));
        wrong_dependency.terms[0].source_range.end += 1;
        assert_eq!(
            SourceProofLocalGivenDescendantUseTermProducer::build(
                fixture.dependency.clone(),
                wrong_dependency,
                &task269sdu_test_arena(fixture.source, true, true),
            ),
            Err(SourceProofLocalGivenDescendantUseTermError::InvalidDependency)
        );

        let mut wrong_input = task269sdu_input(fixture.source, fixture.module.clone());
        wrong_input.references[0].binding = BindingId::new(0);
        assert_eq!(
            SourceProofLocalGivenDescendantUseTermProducer::build(
                fixture.dependency.clone(),
                wrong_input,
                &task269sdu_test_arena(fixture.source, true, false),
            ),
            Err(SourceProofLocalGivenDescendantUseTermError::InvalidSourceTerm)
        );
        assert_eq!(
            SourceProofLocalGivenDescendantUseTermProducer::build(
                fixture.dependency.clone(),
                task269sdu_input(fixture.source, fixture.module.clone()),
                &task269sdu_test_arena(fixture.source, false, true),
            ),
            Err(SourceProofLocalGivenDescendantUseTermError::InvalidSourceTerm)
        );
        assert_eq!(
            SourceProofLocalGivenDescendantUseTermProducer::build(
                fixture.dependency.clone(),
                task269sdu_input(fixture.source, fixture.module.clone()),
                &task269sdu_test_arena(fixture.source, true, false),
            ),
            Err(SourceProofLocalGivenDescendantUseTermError::InvalidInstallation)
        );

        let mut combined = exact.clone();
        combined.dependency_fingerprint.push_str("corrupt");
        combined.source_term_fingerprint.push_str("corrupt");
        assert_eq!(
            combined.validate_complete_installation(
                fixture.source,
                &fixture.module,
                &fixture.arena,
                false,
            ),
            Err(SourceProofLocalGivenDescendantUseTermError::InvalidDependency)
        );
        combined.dependency_fingerprint = combined.dependency.debug_text();
        assert_eq!(
            combined.validate_complete_installation(
                fixture.source,
                &fixture.module,
                &fixture.arena,
                false,
            ),
            Err(SourceProofLocalGivenDescendantUseTermError::InvalidSourceTerm)
        );
        combined.source_term_fingerprint = combined.source_term.debug_text();
        assert_eq!(
            combined.validate_complete_installation(
                fixture.source,
                &fixture.module,
                &fixture.arena,
                false,
            ),
            Err(SourceProofLocalGivenDescendantUseTermError::InvalidInstallation)
        );
        assert_eq!(
            SourceProofLocalGivenDescendantUseTermError::InvalidDependency.to_string(),
            "source proof-local given-descendant-use term dependency is invalid"
        );
        assert_eq!(
            SourceProofLocalGivenDescendantUseTermError::InvalidSourceTerm.to_string(),
            "source proof-local given-descendant-use source term is invalid"
        );
        assert_eq!(
            SourceProofLocalGivenDescendantUseTermError::InvalidInstallation.to_string(),
            "source proof-local given-descendant-use term installation is invalid"
        );
    }

    #[test]
    fn task269sdu_typed_and_resolved_ownership_is_atomic_and_one_shot() {
        let fixture = task269sdu_fixture();
        let handoff = SourceProofLocalGivenDescendantUseTermProducer::build(
            fixture.dependency.clone(),
            task269sdu_input(fixture.source, fixture.module.clone()),
            &fixture.arena,
        )
        .expect("Task269SDU exact handoff");
        let typed = task269gcu_empty_typed_for(
            fixture.source,
            fixture.module.clone(),
            fixture.arena.clone(),
        )
        .with_source_proof_local_given_descendant_use_term(handoff.clone())
        .expect("Task269SDU first install");
        let before = typed.debug_text();
        assert_eq!(
            typed
                .clone()
                .with_source_proof_local_given_descendant_use_term(handoff.clone()),
            Err(TypedAstError::InvalidSourceProofLocalGivenDescendantUseTerm)
        );
        assert_eq!(typed.debug_text(), before);
        assert!(typed.contexts().is_empty());
        assert!(typed.types().is_empty());
        assert!(typed.facts().is_empty());
        assert!(typed.coercions().is_empty());
        assert!(typed.initial_obligations().is_empty());
        assert!(typed.diagnostics().is_empty());

        let mut predecessor_first = task269gcu_empty_typed_for(
            fixture.source,
            fixture.module.clone(),
            fixture.arena.clone(),
        );
        predecessor_first
            .inject_source_proof_local_given_descendant_type_for_test(fixture.dependency.clone());
        let predecessor_before = predecessor_first.debug_text();
        assert_eq!(
            predecessor_first
                .clone()
                .with_source_proof_local_given_descendant_use_term(handoff.clone()),
            Err(TypedAstError::InvalidSourceProofLocalGivenDescendantUseTerm)
        );
        assert_eq!(predecessor_first.debug_text(), predecessor_before);
        assert_eq!(
            typed
                .clone()
                .with_source_proof_local_given_descendant_type(fixture.dependency.clone()),
            Err(TypedAstError::InvalidSourceProofLocalGivenDescendantType)
        );

        let sibling = fixture.dependency.dependency().clone();
        let mut sibling_first = task269gcu_empty_typed_for(
            fixture.source,
            fixture.module.clone(),
            fixture.arena.clone(),
        );
        sibling_first.inject_source_proof_local_given_descendant_binding_for_test(sibling.clone());
        assert_eq!(
            sibling_first.with_source_proof_local_given_descendant_use_term(handoff.clone()),
            Err(TypedAstError::InvalidSourceProofLocalGivenDescendantUseTerm)
        );
        assert_eq!(
            typed
                .clone()
                .with_source_proof_local_given_descendant_binding(sibling),
            Err(TypedAstError::InvalidSourceProofLocalGivenDescendantBinding)
        );

        let resolved = task269gcu_resolved_result_with_inputs(&typed, Vec::new(), Vec::new())
            .expect("Task269SDU final assembly");
        assert_eq!(
            resolved.source_proof_local_given_descendant_use_term(),
            typed.source_proof_local_given_descendant_use_term()
        );
        assert!(resolved.expr_metadata().is_empty());
        assert!(resolved.checked_formulas().is_empty());
        assert!(resolved.checked_proofs().is_empty());
        assert!(resolved.diagnostics().is_empty());

        let dependency_arena =
            task269sdu_dependency_arena(&fixture.arena).expect("Task269SDU SDT arena");
        for sdu_first in [false, true] {
            let mut hybrid = task269gcu_empty_typed_for(
                fixture.source,
                fixture.module.clone(),
                dependency_arena.clone(),
            );
            if sdu_first {
                hybrid
                    .inject_source_proof_local_given_descendant_use_term_for_test(handoff.clone());
                hybrid.inject_source_proof_local_given_descendant_type_for_test(
                    fixture.dependency.clone(),
                );
            } else {
                hybrid.inject_source_proof_local_given_descendant_type_for_test(
                    fixture.dependency.clone(),
                );
                hybrid
                    .inject_source_proof_local_given_descendant_use_term_for_test(handoff.clone());
            }
            let hybrid_before = hybrid.debug_text();
            assert_eq!(
                task269gcu_resolved_result_with_inputs(&hybrid, Vec::new(), Vec::new()),
                Err(ResolvedTypedAstError::InvalidSourceProofLocalGivenDescendantType)
            );
            assert_eq!(hybrid.debug_text(), hybrid_before);
        }

        let direct_binding = fixture.dependency.dependency().clone();
        for sdu_first in [false, true] {
            let mut hybrid = task269gcu_empty_typed_for(
                fixture.source,
                fixture.module.clone(),
                TypedArena::try_new(None, Vec::new()).expect("Task269SDU SDC arena"),
            );
            if sdu_first {
                hybrid
                    .inject_source_proof_local_given_descendant_use_term_for_test(handoff.clone());
                hybrid.inject_source_proof_local_given_descendant_binding_for_test(
                    direct_binding.clone(),
                );
            } else {
                hybrid.inject_source_proof_local_given_descendant_binding_for_test(
                    direct_binding.clone(),
                );
                hybrid
                    .inject_source_proof_local_given_descendant_use_term_for_test(handoff.clone());
            }
            let hybrid_before = hybrid.debug_text();
            assert_eq!(
                task269gcu_resolved_result_with_inputs(&hybrid, Vec::new(), Vec::new()),
                Err(ResolvedTypedAstError::InvalidSourceProofLocalGivenDescendantBinding)
            );
            assert_eq!(hybrid.debug_text(), hybrid_before);
        }

        let condition_fixture = task269gcu_fixture();
        let condition = SourceProofLocalGivenConditionUseTermProducer::build(
            condition_fixture.dependency,
            condition_fixture.input,
            &condition_fixture.arena,
        )
        .expect("Task269SDU condition sibling");
        for sdu_first in [false, true] {
            let mut hybrid = task269gcu_empty_typed_for(
                condition_fixture.source,
                condition_fixture.module.clone(),
                condition_fixture.arena.clone(),
            );
            if sdu_first {
                hybrid
                    .inject_source_proof_local_given_descendant_use_term_for_test(handoff.clone());
                hybrid
                    .inject_source_proof_local_given_condition_use_term_for_test(condition.clone());
            } else {
                hybrid
                    .inject_source_proof_local_given_condition_use_term_for_test(condition.clone());
                hybrid
                    .inject_source_proof_local_given_descendant_use_term_for_test(handoff.clone());
            }
            let hybrid_before = hybrid.debug_text();
            assert_eq!(
                task269gcu_resolved_result_with_inputs(&hybrid, Vec::new(), Vec::new()),
                Err(ResolvedTypedAstError::InvalidSourceProofLocalGivenConditionUseTerm)
            );
            assert_eq!(hybrid.debug_text(), hybrid_before);
        }
    }

    #[test]
    fn task269sdu_near_miss_predecessor_sibling_and_active_routes_are_isolated() {
        let fixture = task269sdu_fixture();
        let exact = SourceProofLocalGivenDescendantUseTermProducer::build(
            fixture.dependency.clone(),
            task269sdu_input(fixture.source, fixture.module.clone()),
            &fixture.arena,
        )
        .expect("Task269SDU exact handoff");
        let input = task269sdu_input(fixture.source, fixture.module.clone());
        assert_eq!(
            input
                .terms
                .iter()
                .map(|term| term.source_range)
                .collect::<Vec<_>>(),
            vec![task269sdu_range(input.source_id, 118, 119)]
        );
        assert!(input.numeric_type_requests.is_empty());
        for (start, end, spelling) in [(114, 115, "z"), (129, 130, "q"), (133, 134, "z")] {
            assert!(!input.terms.iter().any(|term| {
                term.source_range == task269sdu_range(input.source_id, start, end)
                    || term.spelling == spelling
            }));
        }

        for (start, end, spelling) in [(114, 115, "z"), (129, 130, "q"), (133, 134, "z")] {
            let mut near_miss = input.clone();
            near_miss.terms[0].source_range = task269sdu_range(input.source_id, start, end);
            near_miss.terms[0].spelling = spelling.to_owned();
            assert_eq!(
                SourceProofLocalGivenDescendantUseTermProducer::build(
                    fixture.dependency.clone(),
                    near_miss,
                    &fixture.arena,
                ),
                Err(SourceProofLocalGivenDescendantUseTermError::InvalidSourceTerm)
            );
        }

        let dependency_replay = task269sdu_dependency(&fixture);
        assert_eq!(dependency_replay, fixture.dependency);
        assert_eq!(
            dependency_replay.debug_text(),
            fixture.dependency.debug_text()
        );
        let condition_fixture = task269gcu_fixture();
        let condition_replay = SourceProofLocalGivenConditionUseTermProducer::build(
            condition_fixture.dependency.clone(),
            condition_fixture.input.clone(),
            &condition_fixture.arena,
        )
        .expect("Task269SDU sibling replay");
        let condition_replay_again = SourceProofLocalGivenConditionUseTermProducer::build(
            condition_fixture.dependency,
            condition_fixture.input,
            &condition_fixture.arena,
        )
        .expect("Task269SDU sibling replay again");
        assert_eq!(condition_replay, condition_replay_again);
        assert_eq!(
            exact.source_term().terms().len(),
            1,
            "SDU must not absorb sibling or active-route terms"
        );
    }

    #[derive(Clone)]
    struct Task269sduFixture {
        source: SourceId,
        module: ModuleId,
        dependency: SourceProofLocalGivenDescendantTypeHandoff,
        arena: TypedArena,
    }

    fn task269sdu_fixture() -> Task269sduFixture {
        let source = source_id_for("f8");
        let module = module("task269sdc");
        let arena = task269sdu_test_arena(source, false, false);
        let binding = task269sdc_neighbor_for(source, module.clone());
        let dependency_arena =
            task269sdu_dependency_arena(&arena).expect("Task269SDU prefix arena");
        let dependency = SourceProofLocalGivenDescendantTypeProducer::build(
            binding,
            task269sdu_type_input(source, module.clone()),
            &SymbolEnv::new(module.clone(), SymbolEnvIndexes::default()),
            &dependency_arena,
        )
        .expect("Task269SDU exact dependency");
        Task269sduFixture {
            source,
            module,
            dependency,
            arena,
        }
    }

    fn task269sdu_dependency(
        fixture: &Task269sduFixture,
    ) -> SourceProofLocalGivenDescendantTypeHandoff {
        let binding = task269sdc_neighbor_for(fixture.source, fixture.module.clone());
        let dependency_arena =
            task269sdu_dependency_arena(&fixture.arena).expect("Task269SDU prefix arena");
        SourceProofLocalGivenDescendantTypeProducer::build(
            binding,
            task269sdu_type_input(fixture.source, fixture.module.clone()),
            &SymbolEnv::new(fixture.module.clone(), SymbolEnvIndexes::default()),
            &dependency_arena,
        )
        .expect("Task269SDU exact dependency")
    }

    fn task269sdu_type_input(source: SourceId, module: ModuleId) -> SourceTypeHandoffInput {
        SourceTypeHandoffInput {
            source_id: source,
            module_id: module.clone(),
            applications: (0..2)
                .map(|index| SourceTypeApplicationInput {
                    binding: BindingId::new(index),
                    source_ordinal: index,
                    root: SourceTypeExpressionId::new(index),
                })
                .collect(),
            expressions: [(14, 17), (95, 98)]
                .into_iter()
                .enumerate()
                .map(|(index, (start, end))| SourceTypeExpressionInput {
                    source_id: source,
                    module_id: module.clone(),
                    site: TypedSiteRef::Role {
                        node: TypedNodeId::new(index),
                        role: TypeRole::new("source.type.expression"),
                    },
                    source_range: range(source, start, end),
                    spelling: "set".to_owned(),
                    head_site: TypedSiteRef::Role {
                        node: TypedNodeId::new(index),
                        role: TypeRole::new("source.type.head"),
                    },
                    head_range: range(source, start, end),
                    head_spelling: "set".to_owned(),
                    form: SourceTypeApplicationForm::Bare,
                    head: SourceTypeHead::BuiltinSet,
                    recovery: NodeRecoveryState::Normal,
                })
                .collect(),
            arguments: Vec::new(),
        }
    }

    fn task269sdu_test_arena(
        source_id: SourceId,
        wrong_root: bool,
        wrong_kind: bool,
    ) -> TypedArena {
        let mut builder = TypedArenaBuilder::new();
        let reserve = builder
            .push(TypedNode::new(
                "source.proof-local.given-descendant.reserve-type",
                SourceAnchor::Range(task269sdu_range(source_id, 14, 17)),
            ))
            .expect("reserve");
        let given = builder
            .push(TypedNode::new(
                "source.proof-local.given-descendant.type",
                SourceAnchor::Range(task269sdu_range(source_id, 95, 98)),
            ))
            .expect("given");
        let prefix = builder
            .push(
                TypedNode::new(
                    "source.proof-local.given-descendant.type-root",
                    SourceAnchor::Range(task269sdu_range(source_id, 0, 179)),
                )
                .with_children(vec![reserve, given]),
            )
            .expect("prefix");
        let term = builder
            .push(TypedNode::new(
                if wrong_kind {
                    "wrong"
                } else {
                    "source.term.variable-reference"
                },
                SourceAnchor::Range(task269sdu_range(source_id, 118, 119)),
            ))
            .expect("term");
        let root = builder
            .push(
                TypedNode::new(
                    "source.proof-local.given-descendant-use.term-root",
                    SourceAnchor::Range(task269sdu_range(source_id, 0, 179)),
                )
                .with_children(vec![prefix, term]),
            )
            .expect("root");
        builder
            .finish(Some(if wrong_root { term } else { root }))
            .expect("arena")
    }

    fn task257c4c4_dependency() -> SourceNestedFraenkelBinderUseHandoff {
        crate::source_formula_composition::tests::task257c4c3_handoff_for_test()
    }

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Task257c4c4ContextVariant {
        OwnerRange,
        Scope,
        Visibility,
        Parent,
        Layer,
        Recovery,
        ExtraContext,
        Reordered,
    }

    fn task257c4c4_context_variant(
        source: SourceId,
        module: ModuleId,
        variant: Task257c4c4ContextVariant,
    ) -> BindingEnv {
        let outer = BindingContextId::new(if variant == Task257c4c4ContextVariant::Reordered {
            2
        } else {
            1
        });
        let binding = BindingId::new(0);
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
        let outer_draft = BindingContextDraft {
            owner: BindingContextOwner::SourceComprehension {
                source_range: nested_fraenkel_mapper_primary_range(
                    source,
                    if variant == Task257c4c4ContextVariant::OwnerRange {
                        89
                    } else {
                        90
                    },
                    157,
                ),
            },
            parent: Some(BindingContextId::new(0)),
            layer: if variant == Task257c4c4ContextVariant::Layer {
                BindingContextLayer::Block
            } else {
                BindingContextLayer::Expression
            },
            lexical_scope: None,
            bindings: vec![binding],
            visible_bindings: vec![binding],
            recovery: BindingContextRecovery::Normal,
        };
        let inner_draft = BindingContextDraft {
            owner: BindingContextOwner::SourceComprehension {
                source_range: nested_fraenkel_mapper_primary_range(source, 92, 123),
            },
            parent: Some(if variant == Task257c4c4ContextVariant::Parent {
                BindingContextId::new(0)
            } else {
                outer
            }),
            layer: BindingContextLayer::Expression,
            lexical_scope: (variant == Task257c4c4ContextVariant::Scope)
                .then(|| LocalTermScope::new(vec![0])),
            bindings: Vec::new(),
            visible_bindings: if matches!(
                variant,
                Task257c4c4ContextVariant::Visibility | Task257c4c4ContextVariant::Parent
            ) {
                Vec::new()
            } else {
                vec![binding]
            },
            recovery: if variant == Task257c4c4ContextVariant::Recovery {
                BindingContextRecovery::Recovered
            } else {
                BindingContextRecovery::Normal
            },
        };
        if variant == Task257c4c4ContextVariant::Reordered {
            contexts.insert(inner_draft);
            contexts.insert(outer_draft);
        } else {
            contexts.insert(outer_draft);
            contexts.insert(inner_draft);
        }
        if variant == Task257c4c4ContextVariant::ExtraContext {
            contexts.insert(BindingContextDraft {
                owner: BindingContextOwner::Generated("extra".to_owned()),
                parent: Some(BindingContextId::new(2)),
                layer: BindingContextLayer::Expression,
                lexical_scope: None,
                bindings: Vec::new(),
                visible_bindings: vec![binding],
                recovery: BindingContextRecovery::Normal,
            });
        }
        let mut bindings = BindingTable::new();
        bindings.insert(BindingDraft {
            spelling: "x".to_owned(),
            kind: BindingKind::QuantifierBinder,
            identity: BinderIdentity::SourceBound {
                context: outer,
                ordinal: 0,
            },
            owner_context: outer,
            declaration_range: nested_fraenkel_mapper_primary_range(source, 136, 137),
            visible_after_ordinal: 0,
            type_site: BindingTypeSite::Source(nested_fraenkel_mapper_primary_range(
                source, 141, 155,
            )),
            status: BindingStatus::Active,
            captured: CapturedFreeVariables::default(),
            diagnostics: Vec::new(),
            recovery: BindingRecoveryState::Normal,
        });
        BindingEnv::try_new(BindingEnvParts {
            source_id: source,
            module_id: module,
            contexts,
            bindings,
            diagnostics: BindingDiagnosticTable::new(),
        })
        .expect("structurally valid context variant")
    }

    #[derive(Clone)]
    struct Task257c4c4BindingVariant {
        spelling: &'static str,
        kind: BindingKind,
        identity: BinderIdentity,
        owner_context: BindingContextId,
        declaration_range: SourceRange,
        visible_after_ordinal: usize,
        type_site: BindingTypeSite,
        status: BindingStatus,
        captured: CapturedFreeVariables,
        diagnostics: Vec<crate::binding_env::BindingDiagnosticId>,
        recovery: BindingRecoveryState,
    }

    impl Task257c4c4BindingVariant {
        fn exact(source: SourceId) -> Self {
            Self {
                spelling: "x",
                kind: BindingKind::QuantifierBinder,
                identity: BinderIdentity::SourceBound {
                    context: BindingContextId::new(1),
                    ordinal: 0,
                },
                owner_context: BindingContextId::new(1),
                declaration_range: nested_fraenkel_mapper_primary_range(source, 136, 137),
                visible_after_ordinal: 0,
                type_site: BindingTypeSite::Source(nested_fraenkel_mapper_primary_range(
                    source, 141, 155,
                )),
                status: BindingStatus::Active,
                captured: CapturedFreeVariables::default(),
                diagnostics: Vec::new(),
                recovery: BindingRecoveryState::Normal,
            }
        }
    }

    fn task257c4c4_binding_variant(
        source: SourceId,
        module: ModuleId,
        variant: Task257c4c4BindingVariant,
    ) -> BindingEnv {
        let mut env = nested_fraenkel_mapper_primary_binding_env(source, module)
            .expect("exact C4C4 binding environment");
        *env.binding_mut_for_test(BindingId::new(0)).unwrap() = crate::binding_env::BindingEntry {
            id: BindingId::new(0),
            spelling: variant.spelling.to_owned(),
            kind: variant.kind,
            identity: variant.identity,
            owner_context: variant.owner_context,
            declaration_range: variant.declaration_range,
            visible_after_ordinal: variant.visible_after_ordinal,
            type_site: variant.type_site,
            status: variant.status,
            captured: variant.captured,
            diagnostics: variant.diagnostics,
            recovery: variant.recovery,
        };
        env
    }

    #[test]
    fn task257c4c4_builds_exact_nested_mapper_primary_handoff() {
        let dependency = task257c4c4_dependency();
        let expected_dependency = dependency.debug_text();
        let handoff = SourceNestedFraenkelMapperPrimaryProducer::build(dependency).unwrap();

        assert_eq!(handoff.source_id(), handoff.dependency().source_id());
        assert_eq!(handoff.module_id(), handoff.dependency().module_id());
        assert_eq!(handoff.dependency_fingerprint(), expected_dependency);
        assert_eq!(handoff.binding_env().contexts().len(), 3);
        assert_eq!(handoff.binding_env().bindings().len(), 1);
        assert!(handoff.binding_env().diagnostics().is_empty());
        assert_eq!(
            handoff
                .binding_env()
                .contexts()
                .get(BindingContextId::new(0))
                .unwrap()
                .owner,
            BindingContextOwner::Module
        );
        assert_eq!(
            handoff
                .binding_env()
                .contexts()
                .get(BindingContextId::new(1))
                .unwrap()
                .owner,
            BindingContextOwner::SourceComprehension {
                source_range: nested_fraenkel_mapper_primary_range(handoff.source_id(), 90, 157),
            }
        );
        let inner = handoff
            .binding_env()
            .contexts()
            .get(BindingContextId::new(2))
            .unwrap();
        assert_eq!(inner.parent, Some(BindingContextId::new(1)));
        assert!(inner.lexical_scope.is_none());
        assert_eq!(inner.visible_bindings, vec![BindingId::new(0)]);
        let binding = handoff
            .binding_env()
            .bindings()
            .get(BindingId::new(0))
            .unwrap();
        assert_eq!(binding.spelling, "x");
        assert_eq!(binding.kind, BindingKind::QuantifierBinder);
        assert_eq!(
            binding.identity,
            BinderIdentity::SourceBound {
                context: BindingContextId::new(1),
                ordinal: 0,
            }
        );
        assert_eq!(
            binding.declaration_range,
            nested_fraenkel_mapper_primary_range(handoff.source_id(), 136, 137)
        );
        assert_eq!(
            binding.type_site,
            BindingTypeSite::Source(nested_fraenkel_mapper_primary_range(
                handoff.source_id(),
                141,
                155
            ))
        );
        assert!(binding.captured.identities().is_empty());
        assert!(binding.diagnostics.is_empty());
        assert!(matches!(
            handoff.binding_env().lookup(&BindingLookupSite::new(
                "x",
                BindingContextId::new(2),
                None,
                0,
            )),
            Ok(BindingLookupResult::ForwardReference { candidates, .. })
                if candidates == vec![BindingId::new(0)]
        ));
        assert_eq!(
            handoff.binding_env().lookup(&BindingLookupSite::new(
                "x",
                BindingContextId::new(2),
                None,
                1,
            )),
            Ok(BindingLookupResult::Local(BindingId::new(0)))
        );
        assert_eq!(
            handoff.binding_fingerprint(),
            handoff.binding_env().debug_text()
        );
        assert_eq!(handoff.projection_arena().len(), 1);
        assert_eq!(handoff.projection_arena().root(), Some(TypedNodeId::new(0)));
        let node = handoff
            .projection_arena()
            .node(TypedNodeId::new(0))
            .unwrap();
        assert_eq!(node.kind.as_str(), "source.term.variable-reference");
        assert_eq!(
            node.anchor,
            SourceAnchor::Range(nested_fraenkel_mapper_primary_range(
                handoff.source_id(),
                94,
                95
            ))
        );
        assert!(node.resolved_node.is_none());
        assert!(node.children.is_empty());
        assert_eq!(node.typing, TypingState::Unknown);
        assert_eq!(node.recovery, NodeRecoveryState::Normal);
        assert_eq!(node.links, Default::default());
        assert_eq!(handoff.source_term().terms().len(), 1);
        assert_eq!(handoff.source_term().references().len(), 1);
        assert!(handoff.source_term().numeric_type_requests().is_empty());
        let term = handoff
            .source_term()
            .terms()
            .get(SourcePrimaryTermId::new(0))
            .unwrap();
        assert_eq!(term.site(), &TypedSiteRef::Node(TypedNodeId::new(0)));
        assert_eq!(
            term.source_range(),
            nested_fraenkel_mapper_primary_range(handoff.source_id(), 94, 95)
        );
        assert_eq!(term.source_ordinal(), 0);
        assert_eq!(term.context(), BindingContextId::new(2));
        assert_eq!(term.recovery(), SourcePrimaryTermRecovery::Normal);
        assert_eq!(term.spelling(), "x");
        assert_eq!(term.kind(), SourcePrimaryTermKind::VariableReference);
        assert_eq!(term.role(), SourcePrimaryTermRole::Value);
        assert_eq!(term.parent(), None);
        let reference = handoff
            .source_term()
            .references()
            .get(SourcePrimaryTermReferenceId::new(0))
            .unwrap();
        assert_eq!(reference.term(), SourcePrimaryTermId::new(0));
        assert_eq!(reference.binding(), BindingId::new(0));
        assert_eq!(reference.role(), SourcePrimaryTermReferenceRole::Variable);
        assert!(reference.lexical_scope().is_none());
        assert_eq!(reference.use_ordinal(), 1);
        assert_eq!(
            handoff.source_term_fingerprint(),
            handoff.source_term().debug_text()
        );
        assert_eq!(
            handoff.debug_text(),
            format!(
                "source-nested-fraenkel-mapper-primary-debug-v1\nmodule: {}::{}\ndependency-fingerprint: {:?}\nbinding-fingerprint: {:?}\nprojection: nodes=1 root=0\nsource-term-fingerprint: {:?}\n",
                handoff.module_id().package().as_str(),
                handoff.module_id().path().as_str(),
                handoff.dependency_fingerprint(),
                handoff.binding_fingerprint(),
                handoff.source_term_fingerprint(),
            )
        );
    }

    #[test]
    fn task257c4c4_rejects_dependency_and_binding_projection_corruption() {
        use crate::source_formula_composition::tests::Task257c4c3HandoffCorruption;

        for corruption in [
            Task257c4c3HandoffCorruption::Source,
            Task257c4c3HandoffCorruption::Module,
            Task257c4c3HandoffCorruption::Summary,
            Task257c4c3HandoffCorruption::Row,
            Task257c4c3HandoffCorruption::RetainedResolver,
            Task257c4c3HandoffCorruption::RetainedTypedAst,
        ] {
            assert!(matches!(
                SourceNestedFraenkelMapperPrimaryProducer::build(
                    crate::source_formula_composition::tests::task257c4c3_corrupted_handoff_for_test(
                        corruption,
                    ),
                ),
                Err(SourceNestedFraenkelMapperPrimaryError::InvalidDependency)
            ));
        }
        let valid =
            SourceNestedFraenkelMapperPrimaryProducer::build(task257c4c4_dependency()).unwrap();
        let mut dependency = valid.clone();
        dependency.dependency_fingerprint = "stale".to_owned();
        assert!(matches!(
            dependency.validate(),
            Err(SourceNestedFraenkelMapperPrimaryError::InvalidDependency)
        ));
        let mut source = valid.clone();
        source.source_id = other_source_id();
        source.binding_fingerprint = "stale".to_owned();
        assert!(matches!(
            source.validate(),
            Err(SourceNestedFraenkelMapperPrimaryError::InvalidDependency)
        ));
        let mut binding_fingerprint = valid.clone();
        binding_fingerprint.binding_fingerprint = "stale".to_owned();
        assert!(matches!(
            binding_fingerprint.validate(),
            Err(SourceNestedFraenkelMapperPrimaryError::InvalidBindingEnvironment)
        ));
        let mut context = valid.clone();
        context.binding_env = binding_env(context.source_id, &context.module_id, Vec::new(), None);
        assert!(matches!(
            context.validate(),
            Err(SourceNestedFraenkelMapperPrimaryError::InvalidBindingEnvironment)
        ));
        let mut binding = valid.clone();
        binding
            .binding_env
            .binding_mut_for_test(BindingId::new(0))
            .unwrap()
            .spelling = "y".to_owned();
        assert!(matches!(
            binding.validate(),
            Err(SourceNestedFraenkelMapperPrimaryError::InvalidBindingEnvironment)
        ));
        let source = valid.source_id;
        let module = valid.module_id.clone();
        let exact = Task257c4c4BindingVariant::exact(source);
        let binding_variants = vec![
            {
                let mut variant = exact.clone();
                variant.kind = BindingKind::LetBinding;
                variant
            },
            {
                let mut variant = exact.clone();
                variant.identity = BinderIdentity::Generated {
                    context: BindingContextId::new(1),
                    counter: 0,
                };
                variant
            },
            {
                let mut variant = exact.clone();
                variant.owner_context = BindingContextId::new(0);
                variant
            },
            {
                let mut variant = exact.clone();
                variant.declaration_range = nested_fraenkel_mapper_primary_range(source, 135, 137);
                variant
            },
            {
                let mut variant = exact.clone();
                variant.visible_after_ordinal = 1;
                variant
            },
            {
                let mut variant = exact.clone();
                variant.type_site = BindingTypeSite::Missing;
                variant
            },
            {
                let mut variant = exact.clone();
                variant.status = BindingStatus::Reserved;
                variant
            },
            {
                let mut variant = exact.clone();
                variant.captured = CapturedFreeVariables::new(vec![BinderIdentity::Generated {
                    context: BindingContextId::new(1),
                    counter: 1,
                }]);
                variant
            },
            {
                let mut variant = exact.clone();
                variant.diagnostics = vec![crate::binding_env::BindingDiagnosticId::new(0)];
                variant
            },
            {
                let mut variant = exact;
                variant.recovery = BindingRecoveryState::Recovered;
                variant
            },
        ];
        for variant in binding_variants {
            let mut corrupted = valid.clone();
            corrupted.binding_env = task257c4c4_binding_variant(source, module.clone(), variant);
            assert!(matches!(
                corrupted.validate(),
                Err(SourceNestedFraenkelMapperPrimaryError::InvalidBindingEnvironment)
            ));
        }
        for variant in [
            Task257c4c4ContextVariant::OwnerRange,
            Task257c4c4ContextVariant::Scope,
            Task257c4c4ContextVariant::Visibility,
            Task257c4c4ContextVariant::Parent,
            Task257c4c4ContextVariant::Layer,
            Task257c4c4ContextVariant::Recovery,
            Task257c4c4ContextVariant::ExtraContext,
            Task257c4c4ContextVariant::Reordered,
        ] {
            let mut corrupted = valid.clone();
            corrupted.binding_env =
                task257c4c4_context_variant(valid.source_id, valid.module_id.clone(), variant);
            assert!(matches!(
                corrupted.validate(),
                Err(SourceNestedFraenkelMapperPrimaryError::InvalidBindingEnvironment)
            ));
        }
    }

    #[test]
    fn task257c4c4_rejects_arena_term_reference_and_precedence_corruption() {
        let valid =
            SourceNestedFraenkelMapperPrimaryProducer::build(task257c4c4_dependency()).unwrap();
        let mut source_term_precedence = valid.clone();
        source_term_precedence.binding_fingerprint = "stale".to_owned();
        source_term_precedence.projection_arena = TypedArena::try_new(None, Vec::new()).unwrap();
        assert!(matches!(
            source_term_precedence.validate(),
            Err(SourceNestedFraenkelMapperPrimaryError::InvalidBindingEnvironment)
        ));
        let mut arena = valid.clone();
        arena.projection_arena = TypedArena::try_new(None, Vec::new()).unwrap();
        assert!(matches!(
            arena.validate(),
            Err(SourceNestedFraenkelMapperPrimaryError::InvalidSourceTerm)
        ));
        let exact_node = || {
            TypedNode::new(
                "source.term.variable-reference",
                SourceAnchor::Range(nested_fraenkel_mapper_primary_range(
                    valid.source_id,
                    94,
                    95,
                )),
            )
        };
        let mut wrong_root = valid.clone();
        wrong_root.projection_arena = TypedArena::try_new(None, vec![exact_node()]).unwrap();
        assert!(matches!(
            wrong_root.validate(),
            Err(SourceNestedFraenkelMapperPrimaryError::InvalidSourceTerm)
        ));
        // Test-only corruption construction; production remains syntax-free.
        use mizar_syntax as syntax;

        let mut resolved_builder = mizar_resolve::resolved_ast::ResolvedArenaBuilder::new();
        let resolved_node = resolved_builder
            .push(mizar_resolve::resolved_ast::ResolvedNode::new(
                syntax::SurfaceNodeKind::CompilationUnit,
                Vec::new(),
                SemanticOrigin::new(
                    valid.source_id,
                    valid.module_id.clone(),
                    SourceAnchor::Range(nested_fraenkel_mapper_primary_range(
                        valid.source_id,
                        94,
                        95,
                    )),
                    vec![0],
                ),
            ))
            .expect("resolved-node corruption id");
        for node in [
            TypedNode::new(
                "Identifier",
                SourceAnchor::Range(nested_fraenkel_mapper_primary_range(
                    valid.source_id,
                    94,
                    95,
                )),
            ),
            TypedNode::new(
                "source.term.variable-reference",
                SourceAnchor::Range(nested_fraenkel_mapper_primary_range(
                    valid.source_id,
                    94,
                    96,
                )),
            )
            .with_recovery(NodeRecoveryState::Recovered),
            TypedNode::new(
                "source.term.variable-reference",
                SourceAnchor::Range(nested_fraenkel_mapper_primary_range(
                    valid.source_id,
                    94,
                    95,
                )),
            )
            .with_resolved_node(resolved_node),
            TypedNode::new(
                "source.term.variable-reference",
                SourceAnchor::Range(nested_fraenkel_mapper_primary_range(
                    valid.source_id,
                    94,
                    95,
                )),
            )
            .with_typing(TypingState::Successful),
            TypedNode::new(
                "source.term.variable-reference",
                SourceAnchor::Range(nested_fraenkel_mapper_primary_range(
                    valid.source_id,
                    94,
                    95,
                )),
            )
            .with_links(crate::typed_ast::TypedNodeLinks {
                context: None,
                type_entry: None,
                facts: Vec::new(),
                coercions: Vec::new(),
                initial_obligations: Vec::new(),
                diagnostics: vec![crate::typed_ast::TypeDiagnosticId::new(0)],
            }),
        ] {
            let mut corrupted = valid.clone();
            corrupted.projection_arena =
                TypedArena::try_new(Some(TypedNodeId::new(0)), vec![node]).unwrap();
            assert!(matches!(
                corrupted.validate(),
                Err(SourceNestedFraenkelMapperPrimaryError::InvalidSourceTerm)
            ));
        }
        let mut children = valid.clone();
        children.projection_arena = TypedArena::try_new(
            Some(TypedNodeId::new(0)),
            vec![
                TypedNode::new(
                    "source.term.variable-reference",
                    SourceAnchor::Range(nested_fraenkel_mapper_primary_range(
                        valid.source_id,
                        94,
                        95,
                    )),
                )
                .with_children(vec![TypedNodeId::new(1)]),
                TypedNode::new(
                    "source.term.variable-reference",
                    SourceAnchor::Range(nested_fraenkel_mapper_primary_range(
                        valid.source_id,
                        94,
                        95,
                    )),
                ),
            ],
        )
        .unwrap();
        assert!(matches!(
            children.validate(),
            Err(SourceNestedFraenkelMapperPrimaryError::InvalidSourceTerm)
        ));
        let mut term = valid.clone();
        term.source_term
            .corrupt_for_test(SourcePrimaryTermCorruptionForTest::Truncate(0));
        assert!(matches!(
            term.validate(),
            Err(SourceNestedFraenkelMapperPrimaryError::InvalidSourceTerm)
        ));
        let mut duplicate_term = valid.clone();
        duplicate_term
            .source_term
            .corrupt_for_test(SourcePrimaryTermCorruptionForTest::Duplicate(
                SourcePrimaryTermId::new(0),
            ));
        assert!(matches!(
            duplicate_term.validate(),
            Err(SourceNestedFraenkelMapperPrimaryError::InvalidSourceTerm)
        ));
        let mut term_variants = Vec::new();
        for mutate in [
            |row: &mut SourcePrimaryTerm| row.site = TypedSiteRef::Node(TypedNodeId::new(1)),
            |row: &mut SourcePrimaryTerm| row.source_range.end += 1,
            |row: &mut SourcePrimaryTerm| row.source_ordinal = 1,
            |row: &mut SourcePrimaryTerm| row.context = BindingContextId::new(1),
            |row: &mut SourcePrimaryTerm| row.recovery = SourcePrimaryTermRecovery::Degraded,
            |row: &mut SourcePrimaryTerm| row.spelling = "y".to_owned(),
            |row: &mut SourcePrimaryTerm| row.kind = SourcePrimaryTermKind::ConstantReference,
            |row: &mut SourcePrimaryTerm| row.role = SourcePrimaryTermRole::CurrentDefinitionResult,
            |row: &mut SourcePrimaryTerm| row.parent = Some(SourcePrimaryTermId::new(0)),
        ] {
            let mut corrupted = valid.clone();
            mutate(corrupted.source_term.terms.rows.get_mut(0).unwrap());
            term_variants.push(corrupted);
        }
        for corrupted in term_variants {
            assert!(matches!(
                corrupted.validate(),
                Err(SourceNestedFraenkelMapperPrimaryError::InvalidSourceTerm)
            ));
        }
        let mut reference = valid.clone();
        reference
            .source_term
            .set_reference_use_ordinal_for_test(SourcePrimaryTermReferenceId::new(0), 0);
        assert!(matches!(
            reference.validate(),
            Err(SourceNestedFraenkelMapperPrimaryError::InvalidSourceTerm)
        ));
        let mut missing_reference = valid.clone();
        missing_reference.source_term.references.rows.clear();
        assert!(matches!(
            missing_reference.validate(),
            Err(SourceNestedFraenkelMapperPrimaryError::InvalidSourceTerm)
        ));
        let mut extra_reference = valid.clone();
        extra_reference
            .source_term
            .references
            .rows
            .push(extra_reference.source_term.references.rows[0].clone());
        assert!(matches!(
            extra_reference.validate(),
            Err(SourceNestedFraenkelMapperPrimaryError::InvalidSourceTerm)
        ));
        let mut reference_variants = Vec::new();
        for mutate in [
            |row: &mut SourcePrimaryTermReference| row.term = SourcePrimaryTermId::new(1),
            |row: &mut SourcePrimaryTermReference| row.binding = BindingId::new(1),
            |row: &mut SourcePrimaryTermReference| {
                row.role = SourcePrimaryTermReferenceRole::LocalConstant
            },
            |row: &mut SourcePrimaryTermReference| {
                row.lexical_scope = Some(LocalTermScope::new(vec![0]))
            },
        ] {
            let mut corrupted = valid.clone();
            mutate(corrupted.source_term.references.rows.get_mut(0).unwrap());
            reference_variants.push(corrupted);
        }
        for corrupted in reference_variants {
            assert!(matches!(
                corrupted.validate(),
                Err(SourceNestedFraenkelMapperPrimaryError::InvalidSourceTerm)
            ));
        }
        let mut request = valid.clone();
        request
            .source_term
            .numeric_type_requests
            .rows
            .push(SourceNumericTypeRequest {
                term: SourcePrimaryTermId::new(0),
                owner: TypedSiteRef::Node(TypedNodeId::new(0)),
                source_range: nested_fraenkel_mapper_primary_range(valid.source_id, 94, 95),
                spelling: "x".to_owned(),
                request_ordinal: 0,
            });
        assert!(matches!(
            request.validate(),
            Err(SourceNestedFraenkelMapperPrimaryError::InvalidSourceTerm)
        ));
        let mut source_term_fingerprint = valid;
        source_term_fingerprint.source_term_fingerprint = "stale".to_owned();
        assert!(matches!(
            source_term_fingerprint.validate(),
            Err(SourceNestedFraenkelMapperPrimaryError::InvalidSourceTerm)
        ));
    }

    #[test]
    fn task257c4c4_replays_deterministically_and_preserves_generic_task252_rejection() {
        let first = SourceNestedFraenkelMapperPrimaryProducer::build(task257c4c4_dependency());
        let second = SourceNestedFraenkelMapperPrimaryProducer::build(task257c4c4_dependency());
        assert!(
            matches!((&first, &second), (Ok(left), Ok(right)) if left == right && left.debug_text() == right.debug_text())
        );
        let handoff = first.unwrap();
        let generic = SourcePrimaryTermProducer::build(
            nested_fraenkel_mapper_primary_input(handoff.source_id(), handoff.module_id().clone()),
            handoff.binding_env(),
            handoff.projection_arena(),
        );
        assert!(
            generic.is_err(),
            "generic forward-written mapper must be rejected: {generic:?}"
        );
        let raw_identifier_arena = TypedArena::try_new(
            Some(TypedNodeId::new(0)),
            vec![TypedNode::new(
                "Identifier",
                SourceAnchor::Range(nested_fraenkel_mapper_primary_range(
                    handoff.source_id(),
                    94,
                    95,
                )),
            )],
        )
        .unwrap();
        assert!(
            SourcePrimaryTermProducer::build(
                nested_fraenkel_mapper_primary_input(
                    handoff.source_id(),
                    handoff.module_id().clone()
                ),
                handoff.binding_env(),
                &raw_identifier_arena,
            )
            .is_err()
        );
    }
}
