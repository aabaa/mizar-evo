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
    source_proof_local_declaration::SourceProofLocalIntakeReceipt,
    source_type::{
        SourceProofLocalGivenConditionTypeHandoff, SourceProofLocalGivenDescendantTypeHandoff,
        SourceProofLocalGivenUseTypeHandoff,
    },
    typed_ast::{NodeRecoveryState, TypedArena, TypedNode, TypedNodeId, TypedSiteRef, TypingState},
};
use mizar_lexer::is_identifier;
use mizar_resolve::{
    names::{
        LocalTermScope, ResolvedVariableScope, SourceVariableBindingKind,
        SourceVariableReferenceKind,
    },
    resolved_ast::ModuleId,
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
            validate_term_node(term, arena, None)
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
        let intake = self.dependency.dependency().intake();
        let expected = SourcePrimaryTermProducer::build_with_profile(
            task269gu_input(source_id, module_id.clone(), intake, 1),
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
        if !exact_task269gu_arena(intake, arena, "given-use") {
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
        let intake = dependency.dependency().intake();
        if !exact_task269gu_input(&input, intake, 1) {
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
        if !exact_task269gu_arena(intake, arena, "given-use") {
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
        let intake = self.dependency.dependency().intake();
        let expected = SourcePrimaryTermProducer::build_with_profile(
            task269gu_input(source_id, module_id.clone(), intake, 2),
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
        if !exact_task269gu_arena(intake, arena, "given-descendant-use") {
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
        let intake = dependency.dependency().intake();
        if !exact_task269gu_input(&input, intake, 2) {
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
        if !exact_task269gu_arena(intake, arena, "given-descendant-use") {
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
        let intake = self.dependency.dependency().intake();
        let expected = SourcePrimaryTermProducer::build_with_profile(
            task269gu_input(source_id, module_id.clone(), intake, 1),
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
        if !exact_task269gu_arena(intake, arena, "given-condition") {
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
        let intake = dependency.dependency().intake();
        if !exact_task269gu_input(&input, intake, 1) {
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
        if !exact_task269gu_arena(intake, arena, "given-condition") {
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
    ScopedTheoremOccurrences,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SourcePrimaryTermProducer;

impl SourcePrimaryTermProducer {
    pub(crate) fn build_from_original_reserved_statement_occurrences(
        source: &mizar_resolve::resolved_ast::SurfaceResolvedArena,
        symbols: &mizar_resolve::env::SymbolEnv,
        scope: &ResolvedVariableScope,
        arena: &TypedArena,
        mut parenthesized: Vec<(TypedSiteRef, SourceRange, TypedSiteRef)>,
    ) -> Result<(BindingEnv, SourcePrimaryTermHandoff), SourcePrimaryTermError> {
        let invalid = || SourcePrimaryTermError::InvalidTransaction;
        let bindings = crate::type_checker::SourceVariableSemanticsChecker::original_reserved_statement_binding_env(
            source, symbols, scope,
        ).map_err(|_| invalid())?;
        let (scopes, _) = mizar_resolve::symbols::source_proof_scope_maps(source, symbols, scope)
            .map_err(|_| invalid())?;
        let root_scope = scopes
            .iter()
            .find(|(node, _)| source.resolved_node_for(**node) == Some(source.arena().root()))
            .map(|(_, local)| local)
            .ok_or_else(invalid)?;
        let [declaration] = scope.bindings() else {
            return Err(invalid());
        };
        let binding = bindings
            .bindings()
            .get(BindingId::new(0))
            .ok_or_else(invalid)?;
        if binding.identity
            != (BinderIdentity::ReservedVariable {
                spelling: declaration.spelling().to_owned(),
                declaration_range: declaration.range(),
            })
        {
            return Err(invalid());
        }
        let mut inputs = Vec::new();
        let mut references = Vec::new();
        let mut seen = std::collections::BTreeSet::new();
        for (ordinal, actual) in scope.references().iter().enumerate() {
            let invalid = || SourcePrimaryTermError::InvalidReference {
                reference: SourcePrimaryTermReferenceId::new(ordinal),
            };
            let resolved = source
                .resolved_node_for(actual.node())
                .ok_or_else(invalid)?;
            let (typed, _) = arena
                .iter()
                .find(|(_, node)| node.resolved_node == Some(resolved))
                .ok_or_else(invalid)?;
            let context = if actual.scope() == root_scope {
                BindingContextId::new(0)
            } else {
                bindings
                    .contexts()
                    .iter()
                    .find(|(_, context)| context.lexical_scope.as_ref() == Some(actual.scope()))
                    .map(|(id, _)| id)
                    .ok_or_else(invalid)?
            };
            if !seen.insert(actual.node())
                || actual.kind() != SourceVariableReferenceKind::Term
                || actual.binding() != declaration.id()
                || actual.spelling() != declaration.spelling()
                || actual.ordinal() != ordinal
                || scopes.get(&actual.node()) != Some(actual.scope())
                || declaration.range().end > actual.range().start
                || !bindings
                    .contexts()
                    .get(context)
                    .ok_or_else(invalid)?
                    .visible_bindings
                    .contains(&binding.id)
            {
                return Err(invalid());
            }
            let mut parent = None;
            if parenthesized
                .last()
                .is_some_and(|(_, _, child)| child == &TypedSiteRef::Node(typed))
            {
                if parenthesized.windows(2).any(|pair| pair[0].2 != pair[1].0) {
                    return Err(invalid());
                }
                let depth = parenthesized.len();
                for (index, (site, source_range, _)) in
                    std::mem::take(&mut parenthesized).into_iter().enumerate()
                {
                    let id = SourcePrimaryTermId::new(inputs.len());
                    inputs.push(SourcePrimaryTermInput {
                        site,
                        source_range,
                        source_ordinal: inputs.len(),
                        context,
                        recovery: SourcePrimaryTermRecovery::Normal,
                        spelling: format!(
                            "{}{}{}",
                            "( ".repeat(depth - index),
                            actual.spelling(),
                            " )".repeat(depth - index)
                        ),
                        kind: SourcePrimaryTermKind::Parenthesized,
                        role: SourcePrimaryTermRole::Value,
                        parent,
                    });
                    parent = Some(id);
                }
            }
            let term = SourcePrimaryTermId::new(inputs.len());
            inputs.push(SourcePrimaryTermInput {
                site: TypedSiteRef::Node(typed),
                source_range: actual.range(),
                source_ordinal: term.index(),
                context,
                recovery: SourcePrimaryTermRecovery::Normal,
                spelling: actual.spelling().to_owned(),
                kind: SourcePrimaryTermKind::VariableReference,
                role: SourcePrimaryTermRole::Value,
                parent,
            });
            references.push(SourcePrimaryTermReference {
                term,
                binding: binding.id,
                role: SourcePrimaryTermReferenceRole::Variable,
                lexical_scope: Some(actual.scope().clone()),
                use_ordinal: actual.ordinal(),
            });
        }
        if !parenthesized.is_empty() {
            return Err(invalid());
        }
        let terms = validate_terms(source.source_id(), &inputs, &bindings, arena)?;
        validate_reference_cardinality(&terms, &references)?;
        let handoff = SourcePrimaryTermHandoff {
            source_id: source.source_id(),
            module_id: source.module().clone(),
            terms: SourcePrimaryTermTable { rows: terms },
            references: SourcePrimaryTermReferenceTable { rows: references },
            numeric_type_requests: SourceNumericTypeRequestTable { rows: Vec::new() },
        };
        Ok((bindings, handoff))
    }

    pub fn build_from_scoped_theorem_occurrences(
        input: SourcePrimaryTermHandoffInput,
        binding_env: &BindingEnv,
        arena: &TypedArena,
        source: &mizar_resolve::resolved_ast::SurfaceResolvedArena,
        symbols: &mizar_resolve::env::SymbolEnv,
        scope: &ResolvedVariableScope,
    ) -> Result<SourcePrimaryTermHandoff, SourcePrimaryTermError> {
        let authentic =
            crate::type_checker::SourceVariableSemanticsChecker::source_theorem_binding_env(
                source, symbols, scope,
            )
            .map_err(|_| SourcePrimaryTermError::InvalidTransaction)?;
        if input.source_id != binding_env.source_id()
            || &input.module_id != binding_env.module_id()
            || binding_env != &authentic
        {
            return Err(SourcePrimaryTermError::InvalidTransaction);
        }
        let terms = validate_terms(input.source_id, &input.terms, binding_env, arena)?;
        let mut ordinals = derive_reference_ordinals(
            input.source_id,
            &input.references,
            &terms,
            binding_env,
            SourcePrimaryTermBindingProfile::ScopedTheoremOccurrences,
        )?;
        for (index, term) in terms.iter().enumerate() {
            if term.recovery != SourcePrimaryTermRecovery::Normal
                || term.kind != SourcePrimaryTermKind::VariableReference
            {
                return Err(SourcePrimaryTermError::InvalidTerm {
                    term: SourcePrimaryTermId::new(index),
                });
            }
        }
        let mut matched = std::collections::BTreeSet::new();
        let mut occurrences = Vec::with_capacity(input.references.len());
        for (index, reference) in input.references.iter().enumerate() {
            let invalid = || SourcePrimaryTermError::InvalidReference {
                reference: SourcePrimaryTermReferenceId::new(index),
            };
            let term = terms.get(reference.term.index()).ok_or_else(invalid)?;
            let resolved = arena
                .node(term.site.node())
                .and_then(|node| node.resolved_node)
                .ok_or_else(invalid)?;
            let (actual_id, actual) = scope
                .references()
                .iter()
                .enumerate()
                .find(|(_, actual)| source.resolved_node_for(actual.node()) == Some(resolved))
                .ok_or_else(invalid)?;
            let declaration = scope
                .bindings()
                .get(actual.binding().index())
                .ok_or_else(invalid)?;
            let binding = binding_env
                .bindings()
                .get(reference.binding)
                .ok_or_else(invalid)?;
            if !matched.insert(actual_id)
                || actual.kind() != SourceVariableReferenceKind::Term
                || actual.range() != term.source_range
                || actual.spelling() != term.spelling
                || binding.identity
                    != (BinderIdentity::ResolverLocal {
                        scope: declaration.scope().clone(),
                        ordinal: declaration.ordinal(),
                        declaration_range: declaration.range(),
                    })
                || binding_env
                    .contexts()
                    .get(term.context)
                    .and_then(|context| context.lexical_scope.as_ref())
                    != Some(actual.scope())
            {
                return Err(invalid());
            }
            if declaration.range().end > actual.range().start {
                if declaration.kind() != SourceVariableBindingKind::ComprehensionGenerator {
                    return Err(invalid());
                }
                // Logical visibility follows only this authenticated generator winner.
                ordinals[index] = binding.visible_after_ordinal + 1;
            }
            occurrences.push(actual);
        }
        if matched.len() != scope.references().len() {
            return Err(SourcePrimaryTermError::InvalidReference {
                reference: SourcePrimaryTermReferenceId::new(input.references.len()),
            });
        }
        let mut references = validate_references(
            &input.references,
            &ordinals,
            &terms,
            binding_env,
            SourcePrimaryTermBindingProfile::ScopedTheoremOccurrences,
        )?;
        validate_reference_cardinality(&terms, &references)?;
        let numeric_type_requests =
            validate_numeric_requests(&input.numeric_type_requests, &terms)?;
        for (reference, actual) in references.iter_mut().zip(occurrences) {
            reference.lexical_scope = Some(actual.scope().clone());
            reference.use_ordinal = actual.ordinal();
        }
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

    /// Builds the source-term handoff from sealed resolver occurrences.
    pub fn build_from_occurrences(
        input: SourcePrimaryTermHandoffInput,
        binding_env: &BindingEnv,
        arena: &TypedArena,
        scope: &ResolvedVariableScope,
    ) -> Result<SourcePrimaryTermHandoff, SourcePrimaryTermError> {
        let invalid_transaction = || SourcePrimaryTermError::InvalidTransaction;
        if input.source_id != scope.source_id()
            || &input.module_id != scope.module_id()
            || *binding_env
                != crate::type_checker::SourceVariableSemanticsChecker::occurrence_binding_env(
                    scope,
                )
        {
            return Err(invalid_transaction());
        }
        let context_id = BindingContextId::new(0);

        let terms = validate_terms(input.source_id, &input.terms, binding_env, arena)?;
        for (index, term) in terms.iter().enumerate() {
            if term.context != context_id
                || term.recovery != SourcePrimaryTermRecovery::Normal
                || !matches!(
                    term.kind,
                    SourcePrimaryTermKind::VariableReference | SourcePrimaryTermKind::Parenthesized
                )
                || term.role != SourcePrimaryTermRole::Value
            {
                return Err(SourcePrimaryTermError::InvalidTerm {
                    term: SourcePrimaryTermId::new(index),
                });
            }
            if term.kind == SourcePrimaryTermKind::Parenthesized {
                let child = terms
                    .iter()
                    .find(|child| child.parent == Some(SourcePrimaryTermId::new(index)))
                    .ok_or(SourcePrimaryTermError::InvalidTerm {
                        term: SourcePrimaryTermId::new(index),
                    })?;
                validate_term_node(term, arena, Some(child)).map_err(|()| {
                    SourcePrimaryTermError::InvalidTerm {
                        term: SourcePrimaryTermId::new(index),
                    }
                })?;
            }
        }

        let mut matched = std::collections::BTreeSet::new();
        let mut references = Vec::with_capacity(input.references.len());
        let invalid_reference = |index| SourcePrimaryTermError::InvalidReference {
            reference: SourcePrimaryTermReferenceId::new(index),
        };
        for (index, input_reference) in input.references.iter().enumerate() {
            let term = terms
                .get(input_reference.term.index())
                .ok_or_else(|| invalid_reference(index))?;
            let (actual_id, actual) = scope
                .references()
                .iter()
                .enumerate()
                .find(|(_, reference)| reference.node().index() == term.site.node().index())
                .ok_or_else(|| invalid_reference(index))?;
            if !matched.insert(actual_id)
                || input_reference.role != SourcePrimaryTermReferenceRole::Variable
                || actual.kind() != SourceVariableReferenceKind::Term
                || actual.range() != term.source_range
                || actual.spelling() != term.spelling
                || actual.binding().index() != input_reference.binding.index()
            {
                return Err(invalid_reference(index));
            }
            references.push(SourcePrimaryTermReference {
                term: input_reference.term,
                binding: input_reference.binding,
                role: input_reference.role,
                // This scope is authenticated by the resolver occurrence,
                // not used as a legacy BindingEnv lookup-time authority.
                lexical_scope: Some(actual.scope().clone()),
                use_ordinal: actual.ordinal(),
            });
        }
        if matched.len() != scope.references().len() {
            return Err(invalid_reference(input.references.len()));
        }
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
        validate_term_node(&row, arena, None)
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
                && !matches!(
                    binding_profile,
                    SourcePrimaryTermBindingProfile::NestedFraenkelMapperPrimary
                        // All scoped identities and the census were authenticated before lookup.
                        | SourcePrimaryTermBindingProfile::ScopedTheoremOccurrences
                ))
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

pub(crate) fn validate_term_node(
    term: &SourcePrimaryTerm,
    arena: &TypedArena,
    authenticated_child: Option<&SourcePrimaryTerm>,
) -> Result<(), ()> {
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
    if let Some(child) = authenticated_child {
        if term.kind != SourcePrimaryTermKind::Parenthesized
            || term.recovery != SourcePrimaryTermRecovery::Normal
            || child.recovery != SourcePrimaryTermRecovery::Normal
            || child.context != term.context
            || child.parent != Some(SourcePrimaryTermId::new(term.source_ordinal))
            || !strictly_contains(term.source_range, child.source_range)
            || term.spelling != format!("( {} )", child.spelling)
        {
            return Err(());
        }
        let [open, bridge, close] = node.children.as_slice() else {
            return Err(());
        };
        let bridge = arena.node(*bridge).ok_or(())?;
        if bridge.kind.as_str() != "TermExpression"
            || bridge.recovery != NodeRecoveryState::Normal
            || bridge.anchor != SourceAnchor::Range(child.source_range)
            || bridge.children.as_slice() != [child.site.node()]
        {
            return Err(());
        }
        for (site, kind, start, end) in [
            (
                *open,
                r#"Token(SurfaceToken { kind: ReservedSymbol, text: "(" })"#,
                term.source_range.start,
                term.source_range.start + 1,
            ),
            (
                *close,
                r#"Token(SurfaceToken { kind: ReservedSymbol, text: ")" })"#,
                term.source_range.end - 1,
                term.source_range.end,
            ),
        ] {
            let token = arena.node(site).ok_or(())?;
            if token.kind.as_str() != kind
                || !token.children.is_empty()
                || token.recovery != NodeRecoveryState::Normal
                || !matches!(&token.anchor, SourceAnchor::Range(range)
                    if range.source_id == term.source_range.source_id && range.start == start && range.end == end)
            {
                return Err(());
            }
        }
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
    if kind == BindingKind::FunctionBinder {
        return role == SourcePrimaryTermReferenceRole::Variable
            && binding_profile == SourcePrimaryTermBindingProfile::ScopedTheoremOccurrences;
    }
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

fn task269gu_input(
    source_id: SourceId,
    module_id: ModuleId,
    receipt: &SourceProofLocalIntakeReceipt,
    context: usize,
) -> SourcePrimaryTermHandoffInput {
    let rows = receipt
        .term_rows()
        .expect("validated proof-local term roles");
    SourcePrimaryTermHandoffInput {
        source_id,
        module_id,
        terms: rows
            .iter()
            .enumerate()
            .map(|(index, row)| SourcePrimaryTermInput {
                site: TypedSiteRef::Node(TypedNodeId::new(index + 3)),
                source_range: row.range(),
                source_ordinal: index,
                context: crate::binding_env::BindingContextId::new(context),
                recovery: SourcePrimaryTermRecovery::Normal,
                spelling: "y".to_owned(),
                kind: SourcePrimaryTermKind::VariableReference,
                role: SourcePrimaryTermRole::Value,
                parent: None,
            })
            .collect(),
        references: (0..rows.len())
            .map(|index| SourcePrimaryTermReferenceInput {
                term: SourcePrimaryTermId::new(index),
                binding: BindingId::new(1),
                role: SourcePrimaryTermReferenceRole::Variable,
            })
            .collect(),
        numeric_type_requests: Vec::new(),
    }
}

fn exact_task269gu_input(
    input: &SourcePrimaryTermHandoffInput,
    receipt: &SourceProofLocalIntakeReceipt,
    context: usize,
) -> bool {
    input == &task269gu_input(input.source_id, input.module_id.clone(), receipt, context)
}

fn exact_task269gu_arena(
    receipt: &SourceProofLocalIntakeReceipt,
    arena: &TypedArena,
    profile: &str,
) -> bool {
    let mut rows = receipt
        .term_rows()
        .expect("validated proof-local term roles");
    let root = TypedNodeId::new(rows.len() + 3);
    if arena.len() != root.index() + 1 || arena.root() != Some(root) {
        return false;
    }
    rows.push(receipt.root_row());
    rows.into_iter().enumerate().all(|(index, row)| {
        let id = TypedNodeId::new(index + 3);
        let (kind, children) = if id == root {
            (
                format!("source.proof-local.{profile}.term-root"),
                (2..root.index()).map(TypedNodeId::new).collect(),
            )
        } else {
            ("source.term.variable-reference".to_owned(), Vec::new())
        };
        arena.node(id).is_some_and(|node| {
            node.kind.as_str() == kind
                && node.resolved_node.is_none()
                && node.anchor == SourceAnchor::Range(row.range())
                && node.children == children
                && node.typing == TypingState::Unknown
                && node.recovery == NodeRecoveryState::Normal
                && node.links == Default::default()
        })
    })
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

#[cfg(test)]
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
        type_checker::SourceVariableSemanticsChecker,
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
        names::{LocalTermBinding, SourceVariableScopeInput, SourceVariableScopeResolver},
        resolved_ast::{FullyQualifiedName, LocalSymbolId, SemanticOrigin, SymbolId},
    };
    use mizar_session::{
        BuildSnapshotId, InMemorySessionIdAllocator, ModulePath, PackageId, SessionIdAllocator as _,
    };
    use mizar_syntax as syntax;
    use syntax::{SurfaceAstBuilder, SurfaceNodeKind, SurfaceTokenKind};

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

    fn task269gu_test_input(source: SourceId, module: ModuleId) -> SourcePrimaryTermHandoffInput {
        let receipt =
            crate::source_proof_local_declaration::task286_receipt(source, module.clone(), 2);
        task269gu_input(source, module, &receipt, 1)
    }

    fn task269gcu_test_input(source: SourceId, module: ModuleId) -> SourcePrimaryTermHandoffInput {
        let receipt =
            crate::source_proof_local_declaration::task286_receipt(source, module.clone(), 3);
        task269gu_input(source, module, &receipt, 1)
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

    fn occurrence_fixture() -> (
        mizar_resolve::names::ResolvedVariableScope,
        BindingEnv,
        TypedArena,
        SourcePrimaryTermHandoffInput,
    ) {
        let source = source_id_for("b3");
        let module = module("source.occurrences");
        let mut builder = SurfaceAstBuilder::new(source);
        let reserve = builder.add_token(
            SurfaceTokenKind::ReservedWord,
            "reserve",
            range(source, 0, 7),
        );
        let binder = builder.add_token(SurfaceTokenKind::Identifier, "x", range(source, 8, 9));
        let be = builder.add_token(SurfaceTokenKind::ReservedWord, "be", range(source, 10, 12));
        let object = builder.add_token(
            SurfaceTokenKind::ReservedWord,
            "object",
            range(source, 13, 19),
        );
        let head = builder.add_node(
            SurfaceNodeKind::TypeHead,
            range(source, 13, 19),
            vec![object],
        );
        let ty = builder.add_node(
            SurfaceNodeKind::TypeExpression,
            range(source, 13, 19),
            vec![head],
        );
        let segment = builder.add_node(
            SurfaceNodeKind::ReserveSegment,
            range(source, 8, 19),
            vec![binder, be, ty],
        );
        let reserve_item = builder.add_node(
            SurfaceNodeKind::ReserveItem,
            range(source, 0, 19),
            vec![reserve, segment],
        );
        let reference_token =
            builder.add_token(SurfaceTokenKind::Identifier, "x", range(source, 24, 25));
        let reference = builder.add_node(
            SurfaceNodeKind::TermReference,
            range(source, 24, 25),
            vec![reference_token],
        );
        let term = builder.add_node(
            SurfaceNodeKind::TermExpression,
            range(source, 24, 25),
            vec![reference],
        );
        let root = builder.add_node(
            SurfaceNodeKind::Root,
            range(source, 0, 25),
            vec![reserve_item, term],
        );
        let ast = builder.finish(Some(root), None);
        let symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        let scope = SourceVariableScopeResolver::resolve_occurrences(
            SourceVariableScopeInput::new(&ast, &module, &symbols),
        )
        .expect("occurrence scope");
        let binding_env = SourceVariableSemanticsChecker::occurrence_binding_env(&scope);
        let actual = &scope.references()[0];
        let input = SourcePrimaryTermHandoffInput {
            source_id: source,
            module_id: module,
            terms: vec![SourcePrimaryTermInput {
                site: TypedSiteRef::Node(TypedNodeId::new(actual.node().index())),
                source_range: actual.range(),
                source_ordinal: 0,
                context: BindingContextId::new(0),
                recovery: SourcePrimaryTermRecovery::Normal,
                spelling: actual.spelling().to_owned(),
                kind: SourcePrimaryTermKind::VariableReference,
                role: SourcePrimaryTermRole::Value,
                parent: None,
            }],
            references: vec![SourcePrimaryTermReferenceInput {
                term: SourcePrimaryTermId::new(0),
                binding: BindingId::new(actual.binding().index()),
                role: SourcePrimaryTermReferenceRole::Variable,
            }],
            numeric_type_requests: Vec::new(),
        };
        let node = TypedNode::new(
            typed_kind_key(SourcePrimaryTermKind::VariableReference),
            SourceAnchor::Range(actual.range()),
        );
        let arena = TypedArena::try_new(None, vec![node; actual.node().index() + 1])
            .expect("typed occurrence arena");
        (scope, binding_env, arena, input)
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
        let lower_fingerprint =
            crate::source_proof_local_declaration::task286_receipt(source, module.clone(), 2);
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
            input: task269gu_test_input(source, module),
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
        let lower_fingerprint =
            crate::source_proof_local_declaration::task286_receipt(source, module.clone(), 0);
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
        let lower_fingerprint =
            crate::source_proof_local_declaration::task286_receipt(source, module.clone(), 1);
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
        let lower_fingerprint =
            crate::source_proof_local_declaration::task286_receipt(source, module.clone(), 4);
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
    fn task257b5_native_reference_rows_and_actual_clocks_are_replayed() {
        use crate::typed_ast::tests::*;
        for mutation in 0..3 {
            task257b5_native_case(
                TASK257B5_NATIVE_TEXT,
                |installed, parts, mut primary, expressions, _| {
                    match mutation {
                        0 => primary.references.rows.swap(0, 1),
                        1 => {
                            primary.references.rows[0].use_ordinal =
                                primary.references.rows[1].use_ordinal
                        }
                        _ => primary.terms.rows.push(primary.terms.rows[0].clone()),
                    }
                    let (lower, final_result) = task257b5_native_forged_replay(
                        installed.unwrap(),
                        parts,
                        primary,
                        expressions,
                    );
                    if mutation == 2 {
                        // Existing atomic direct-target cardinality rejects the duplicate term.
                        assert_eq!(
                            lower.unwrap_err(),
                            TypedAstError::InvalidSourceAtomicFormula
                        );
                    } else {
                        lower.expect(
                            "reference order/clock mutation must pass ordinary lower validation",
                        );
                    }
                    assert!(final_result.is_err());
                },
            );
        }
    }

    #[test]
    fn occurrence_builder_authenticates_resolver_target_and_scope() {
        let (scope, binding_env, arena, input) = occurrence_fixture();
        let handoff =
            SourcePrimaryTermProducer::build_from_occurrences(input, &binding_env, &arena, &scope)
                .expect("sealed occurrence should build");
        let reference = handoff
            .references()
            .get(SourcePrimaryTermReferenceId::new(0))
            .expect("reference row");
        assert_eq!(reference.use_ordinal(), scope.references()[0].ordinal());
        assert_eq!(
            reference.lexical_scope(),
            Some(scope.references()[0].scope())
        );
    }

    #[test]
    fn occurrence_builder_rejects_counterfeit_or_incomplete_inputs() {
        let (scope, binding_env, arena, mut input) = occurrence_fixture();
        input.terms[0].source_range.start += 1;
        assert!(matches!(
            SourcePrimaryTermProducer::build_from_occurrences(input, &binding_env, &arena, &scope,),
            Err(SourcePrimaryTermError::InvalidTerm { .. })
        ));

        let (scope, binding_env, arena, mut input) = occurrence_fixture();
        input.references.clear();
        assert!(matches!(
            SourcePrimaryTermProducer::build_from_occurrences(input, &binding_env, &arena, &scope,),
            Err(SourcePrimaryTermError::InvalidReference { .. })
        ));

        let (scope, mut binding_env, arena, input) = occurrence_fixture();
        binding_env
            .binding_mut_for_test(BindingId::new(0))
            .unwrap()
            .declaration_range
            .start += 1;
        assert_eq!(
            SourcePrimaryTermProducer::build_from_occurrences(input, &binding_env, &arena, &scope,),
            Err(SourcePrimaryTermError::InvalidTransaction)
        );

        let (scope, binding_env, arena, mut input) = occurrence_fixture();
        input.module_id = module("source.wrong");
        assert_eq!(
            SourcePrimaryTermProducer::build_from_occurrences(input, &binding_env, &arena, &scope,),
            Err(SourcePrimaryTermError::InvalidTransaction)
        );

        let (scope, binding_env, arena, mut input) = occurrence_fixture();
        input.terms[0].context = BindingContextId::new(1);
        assert!(matches!(
            SourcePrimaryTermProducer::build_from_occurrences(input, &binding_env, &arena, &scope,),
            Err(SourcePrimaryTermError::InvalidTerm { .. })
        ));
    }

    #[test]
    fn task257c_native_comprehension_contexts_feed_complete_scoped_primary() {
        use mizar_frontend::{
            lexing::{ParserLexingPlan, TokenizeRequest, tokenize},
            parsing::{MizarParserSeam, ParseRequest, ParserInputs, ParserSeam},
            preprocess::preprocess,
            source::{
                FrontendSourceLoader, SourceUnitLoader, SourceUnitRequest, register_source_unit,
            },
            span_bridge::SpanBridge,
        };
        use mizar_resolve::{
            declarations::DeclarationShellCollector, env::NamespacePath,
            resolved_ast::SurfaceResolvedArena, symbols::SignatureProjectionExtractor,
        };
        use mizar_session::{DiskSourceLoader, Edition, SourceInput, SourceOriginInput};

        let mut grouped_corruptions_checked = false;
        let mut previous_chain_dependencies: Option<(
            crate::source_application::SourceFunctorApplicationHandoff,
            crate::source_set_term::SourceSetTermHandoff,
            crate::source_formula_composition::SourceConditionFormulaCompositionHandoff,
        )> = None;
        // Literal reference winners include unused declarations and lexical shadowing.
        for (case, (text, comp_count, binding_count, winners, primary)) in [
            ("theorem T: for A being set holds A in { A where z is object : z in A } proof let A be set; thus A in { A where z is object : z in A }; end;", 2, 4, vec![0,0,1,0,2,2,3,2], Some(true)),
            ("theorem T: for A being set holds A in { A where z is object : z in A };", 1, 2, vec![0,0,1,0], Some(true)),
            ("theorem T: for A being set holds A = A proof let A be set; thus A in { A where z is object : z in A }; end;", 1, 3, vec![0,0,1,1,2,1], Some(true)),
            ("theorem T: for A being set holds A in { A where z is object : z in { A where q is object : q in A } };", 2, 3, vec![0,0,1,0,2,0], Some(true)),
            ("theorem T: for A being set holds A in { A where z is object : z in A } & A in { A where w is object : w in A };", 2, 3, vec![0,0,1,0,0,0,2,0], Some(true)),
            ("theorem T: for A being set holds A in { A where z is object : A = A };", 1, 2, vec![0,0,0,0], Some(true)),
            ("theorem T: for A being set holds A in { A where z is object : z in { A where z is object : z in A } };", 2, 3, vec![0,0,1,0,2,0], Some(true)),
            ("\n:: alpha/layout\ntheorem T: for parcel being\tset holds parcel in { parcel where needle is object : needle in parcel } proof\nlet parcel be set; thus parcel in { parcel where needle is object : needle in parcel }; end;", 2, 4, vec![0,0,1,0,2,2,3,2], Some(true)),
            ("theorem T: for A being set holds A in { z where z is object : z in A };", 1, 2, vec![0,1,1,0], Some(true)),
            ("theorem T: for A being set holds A = A proof let A be set; thus A = A; end;", 0, 2, vec![0,0,1,1], Some(true)),
            ("theorem T: for A being set holds A = A proof let A be set; thus (A |-> A) = A; end;", 0, 3, vec![0,0,2,1], Some(true)),
            ("theorem T: for A being set holds A in { z where z is object : z in A } proof let A be set; thus A in { z where z is object : z in A }; end;", 2, 4, vec![0,1,1,0,2,3,3,2], Some(true)),
            ("theorem T: for A being set holds A in { [z,w] where z is object, w is object : z in A & w in A };", 1, 3, vec![0,1,2,1,0,2,0], Some(true)),
            ("theorem T: for A being set holds A in { { z where z is object : z in A } where z is object : z in A };", 2, 3, vec![0,1,1,0,2,0], Some(true)),
            ("\n:: mapper alpha/layout\ntheorem T: for parcel being\tset holds parcel in { needle where needle is object : needle in parcel } proof\nlet parcel be set; thus parcel in { needle where needle is object : needle in parcel }; end;", 2, 4, vec![0,1,1,0,2,3,3,2], Some(true)),
            ("reserve A for set; theorem T: for A holds A in { A where z is object : z in A };", 1, 0, vec![], None),
            ("theorem T: for A being set holds A = A proof let A be set; take A; thus A in { A where z is object : z in A }; end;", 1, 0, vec![], None),
            ("theorem T: for A being set holds A = A proof let A be set; set B = A; thus A in { A where z is object : z in A }; end;", 1, 0, vec![], None),
            ("theorem T: for A being set holds A = A proof let A be set; thus (A |-> A) in { A where z is object : z in A }; end;", 1, 0, vec![], None),
            ("definition let X,Y be set; pred LikesDef: X likes Y means X = Y; end; theorem T: for A being set holds A in { z where z is set : z likes A };", 1, 2, vec![0,1,1,0], Some(true)),
            ("definition let X,Y be set; pred LikesDef: X likes Y means X = Y; end; theorem T: for A being set holds A = A proof let A be set; thus A in { z where z is set : z likes A }; end;", 1, 3, vec![0,0,1,2,2,1], Some(true)),
            ("definition let X,Y be set; pred LikesDef: X likes Y means X = Y; end; theorem T: for A being set holds A in { z where z is set : z likes A } proof let A be set; thus A in { z where z is set : z likes A }; end;", 2, 4, vec![0,1,1,0,2,3,3,2], Some(true)),
            (":: prefix alpha/layout\ndefinition\n let parcel, needle be object; pred Rel: parcel knows needle means parcel = needle; end;\ntheorem T: for item being set holds item in { point where point is set : point knows item };", 1, 2, vec![0,1,1,0], Some(true)),
            ("definition let X,Y be set; pred LikesDef: X likes Y means X = Y; end; theorem T: for A being set holds A in { z where z is set : z in A }; theorem U: for A being set holds A in { z where z is set : z in A };", 2, 4, vec![0,1,1,0,2,3,3,2], Some(true)),
            ("definition let X,Y be set; pred LikesDef: Y likes X means X = Y; end; theorem T: for A being set holds A in { z where z is set : z in A };", 1, 0, vec![], None),
            ("definition let X,Y be set; pred LikesDef: X likes Y means Y = X; end; theorem T: for A being set holds A in { z where z is set : z in A };", 1, 0, vec![], None),
            ("definition let X,Y be set; pred LikesDef: X likes Y means X = X; end; theorem T: for A being set holds A in { z where z is set : z in A };", 1, 0, vec![], None),
            ("definition let X be set; let Y be set; pred LikesDef: X likes Y means X = Y; end; theorem T: for A being set holds A in { z where z is set : z in A };", 1, 0, vec![], None),
            ("definition let X,Y,Z be set; pred LikesDef: X likes Y means X = Y; end; theorem T: for A being set holds A in { z where z is set : z in A };", 1, 0, vec![], None),
            ("definition let X,Y be set; pred LikesDef: X likes Y means X in Y; end; theorem T: for A being set holds A in { z where z is set : z in A };", 1, 0, vec![], None),
            ("definition let X,Y be set; pred LikesDef: X likes Y means X = Y; pred OtherDef: X knows Y means X = Y; end; theorem T: for A being set holds A in { z where z is set : z in A };", 1, 0, vec![], None),
            ("definition let X,Y be set; pred LikesDef: X likes Y means X = Y; end; definition let X,Y be set; pred OtherDef: X knows Y means X = Y; end; theorem T: for A being set holds A in { z where z is set : z in A };", 1, 0, vec![], None),
            ("theorem T: for A being set holds A in { z where z is set : z in A }; definition let X,Y be set; pred LikesDef: X likes Y means X = Y; end;", 1, 0, vec![], None),
            ("definition let X,Y be set; pred LikesDef: X likes Y means X = Y; end; theorem T: for A being set holds A = A; definition let X,Y be set; pred OtherDef: X knows Y means X = Y; end; theorem U: for A being set holds A in { z where z is set : z in A };", 1, 0, vec![], None),
            ("definition let X,Y be set; pred LikesDef: X likes Y means X = Y; end; theorem T: for A being set holds A in { z where z is set : z does not likes A };", 1, 2, vec![0,1,1,0], Some(true)),
            ("definition let X,Y be set; pred LikesDef: X likes Y means X = Y; end; theorem T: for A being set holds A = A proof let A be set; thus A in { z where z is set : z does not likes A }; end;", 1, 3, vec![0,0,1,2,2,1], Some(true)),
            ("definition let X,Y be set; pred LikesDef: X likes Y means X = Y; end; theorem T: for A being set holds A in { z where z is set : z does not likes A } proof let A be set; thus A in { z where z is set : z does not likes A }; end;", 2, 4, vec![0,1,1,0,2,3,3,2], Some(true)),
            (":: alpha/layout\ndefinition\n let parcel, needle be object; pred Rel: parcel knows needle means parcel = needle; end;\ntheorem T: for item being set holds item in { point where point is set : point do not knows item };", 1, 2, vec![0,1,1,0], Some(true)),
            ("definition let X,Y be set; pred LikesDef: X likes Y means X = Y; end; theorem T: for A being set holds A in { z where z is set : z likes A } & A in { w where w is set : w does not likes A };", 2, 3, vec![0,1,1,0,0,2,2,0], Some(true)),
            ("definition let X,Y be set; pred LikesDef: X likes Y means X = Y; end; theorem T: for A being set holds A in { z where z is set : z likes A likes z };", 1, 2, vec![0,1,1,0,1], Some(true)),
            ("definition let X,Y be set; pred LikesDef: X likes Y means X = Y; end; theorem T: for A being set holds A in { z where z is set : z likes A likes z likes A };", 1, 2, vec![0,1,1,0,1,0], Some(true)),
            ("definition let X,Y be set; pred LikesDef: X likes Y means X = Y; end; theorem T: for A being set holds A in { z where z is set : z likes A likes z likes A likes z };", 1, 2, vec![0,1,1,0,1,0,1], Some(true)),
            ("definition let X,Y be set; pred LikesDef: X likes Y means X = Y; end; theorem T: for A being set holds A in { z where z is set : z does not likes A likes z likes A };", 1, 2, vec![0,1,1,0,1,0], Some(true)),
            ("definition let X,Y be set; pred LikesDef: X likes Y means X = Y; end; theorem T: for A being set holds A in { z where z is set : z likes A does not likes z likes A };", 1, 2, vec![0,1,1,0,1,0], Some(true)),
            ("definition let X,Y be set; pred LikesDef: X likes Y means X = Y; end; theorem T: for A being set holds A in { z where z is set : z likes A likes z does not likes A };", 1, 2, vec![0,1,1,0,1,0], Some(true)),
            ("definition let X,Y be set; pred LikesDef: X likes Y means X = Y; end; theorem T: for A being set holds A in { z where z is set : z do not likes A likes z likes A };", 1, 2, vec![0,1,1,0,1,0], Some(true)),
            ("definition let X,Y be set; pred LikesDef: X likes Y means X = Y; end; theorem T: for A being set holds A in { z where z is set : z likes A do not likes z likes A };", 1, 2, vec![0,1,1,0,1,0], Some(true)),
            ("definition let X,Y be set; pred LikesDef: X likes Y means X = Y; end; theorem T: for A being set holds A in { z where z is set : z likes A likes z do not likes A };", 1, 2, vec![0,1,1,0,1,0], Some(true)),
            ("definition let X,Y be set; pred LikesDef: X likes Y means X = Y; end; theorem T: for A being set holds A = A proof let A,z be set; thus A in { z where z is set : z likes A do not likes z }; end;", 1, 4, vec![0,0,1,3,3,1,3], Some(true)),
            ("definition let X,Y be set; pred LikesDef: X likes Y means X = Y; end; theorem T: for A being set holds A in { z where z is set : z likes A likes z } proof let A be set; thus A in { z where z is set : z do not likes A likes z does not likes A }; end;", 2, 4, vec![0,1,1,0,1,2,3,3,2,3,2], Some(true)),
            ("definition let X,Y be set; pred LikesDef: X likes Y means X = Y; end; theorem T: for A being set holds A in { z where z is set : z likes A } & A in { w where w is set : w likes A do not likes w likes A };", 2, 3, vec![0,1,1,0,0,2,2,0,2,0], Some(true)),
            ("definition let X,Y be set; pred LikesDef: X likes Y means X = Y; end; theorem T: for A being set holds A in { z where z is set : z likes A likes z } & A in { w where w is set : w does not likes A };", 2, 3, vec![0,1,1,0,1,0,2,2,0], Some(true)),
            (":: alpha/layout\ndefinition\n let parcel, needle be object; pred Rel: parcel knows needle means parcel = needle; end;\ntheorem T: for item being\tset holds item in { point where point is set : point knows item do not knows point knows item };", 1, 2, vec![0,1,1,0,1,0], Some(true)),
            ("definition let X,Y be set; pred LikesDef: X likes Y means X = Y; end; theorem T: for A being set holds A in { z where z is set : z likes A likes z } & A in { w where w is set : w likes A likes w };", 2, 3, vec![0,1,1,0,1,0,2,2,0,2], Some(true)),
            ("definition let X,Y be set; pred LikesDef: X likes Y means X = Y; end; theorem T: for A being set holds A in { z where z is set : z does not likes A likes z };", 1, 2, vec![0,1,1,0,1], Some(true)),
            ("definition let X,Y be set; pred LikesDef: X likes Y means X = Y; end; theorem T: for A being set holds A in { z where z is set : z likes A do not likes z };", 1, 2, vec![0,1,1,0,1], Some(true)),
            ("definition let X,Y be set; pred LikesDef: X likes Y means X = Y; end;\n:: alpha/layout\ntheorem T: for parcel being\tset holds parcel in { point where point is set : point likes parcel likes point } & parcel in { needle where needle is set : needle does not likes parcel };", 2, 3, vec![0,1,1,0,1,0,2,2,0], Some(true)),
            ("definition let X,Y be set; pred LikesDef: X likes Y means X = Y; end; theorem T: for A being set holds A in { z where z is set : (z likes A) };", 1, 2, vec![0,1,1,0], Some(true)),
            ("definition let X,Y be set; pred LikesDef: X likes Y means X = Y; end; theorem T: for A being set holds A in { z where z is set : ((z likes A)) };", 1, 2, vec![0,1,1,0], Some(true)),
            ("definition let X,Y be set; pred LikesDef: X likes Y means X = Y; end; theorem T: for A being set holds A in { z where z is set : ((((z likes A)))) };", 1, 2, vec![0,1,1,0], Some(true)),
            ("definition let X,Y be set; pred LikesDef: X likes Y means X = Y; end; theorem T: for A being set holds A in { z where z is set : (z does not likes A) };", 1, 2, vec![0,1,1,0], Some(true)),
            ("definition let X,Y be set; pred LikesDef: X likes Y means X = Y; end; theorem T: for A being set holds A in { z where z is set : ((z do not likes A)) };", 1, 2, vec![0,1,1,0], Some(true)),
            ("definition let X,Y be set; pred LikesDef: X likes Y means X = Y; end; theorem T: for A being set holds A in { z where z is set : (z likes A likes z) };", 1, 2, vec![0,1,1,0,1], Some(true)),
            ("definition let X,Y be set; pred LikesDef: X likes Y means X = Y; end; theorem T: for A being set holds A in { z where z is set : (z does not likes A likes z likes A) };", 1, 2, vec![0,1,1,0,1,0], Some(true)),
            ("definition let X,Y be set; pred LikesDef: X likes Y means X = Y; end; theorem T: for A being set holds A in { z where z is set : ((z does not likes A likes z likes A)) };", 1, 2, vec![0,1,1,0,1,0], Some(true)),
            ("definition let X,Y be set; pred LikesDef: X likes Y means X = Y; end; theorem T: for A being set holds A in { z where z is set : (z do not likes A likes z likes A) };", 1, 2, vec![0,1,1,0,1,0], Some(true)),
            ("definition let X,Y be set; pred LikesDef: X likes Y means X = Y; end; theorem T: for A being set holds A in { z where z is set : ((z do not likes A likes z likes A)) };", 1, 2, vec![0,1,1,0,1,0], Some(true)),
            ("definition let X,Y be set; pred LikesDef: X likes Y means X = Y; end; theorem T: for A being set holds A in { z where z is set : (z likes A does not likes z likes A) };", 1, 2, vec![0,1,1,0,1,0], Some(true)),
            ("definition let X,Y be set; pred LikesDef: X likes Y means X = Y; end; theorem T: for A being set holds A in { z where z is set : ((z likes A does not likes z likes A)) };", 1, 2, vec![0,1,1,0,1,0], Some(true)),
            ("definition let X,Y be set; pred LikesDef: X likes Y means X = Y; end; theorem T: for A being set holds A in { z where z is set : (z likes A do not likes z likes A) };", 1, 2, vec![0,1,1,0,1,0], Some(true)),
            ("definition let X,Y be set; pred LikesDef: X likes Y means X = Y; end; theorem T: for A being set holds A in { z where z is set : ((z likes A do not likes z likes A)) };", 1, 2, vec![0,1,1,0,1,0], Some(true)),
            ("definition let X,Y be set; pred LikesDef: X likes Y means X = Y; end; theorem T: for A being set holds A in { z where z is set : (z likes A likes z does not likes A) };", 1, 2, vec![0,1,1,0,1,0], Some(true)),
            ("definition let X,Y be set; pred LikesDef: X likes Y means X = Y; end; theorem T: for A being set holds A in { z where z is set : ((z likes A likes z does not likes A)) };", 1, 2, vec![0,1,1,0,1,0], Some(true)),
            ("definition let X,Y be set; pred LikesDef: X likes Y means X = Y; end; theorem T: for A being set holds A in { z where z is set : (z likes A likes z do not likes A) };", 1, 2, vec![0,1,1,0,1,0], Some(true)),
            ("definition let X,Y be set; pred LikesDef: X likes Y means X = Y; end; theorem T: for A being set holds A in { z where z is set : ((z likes A likes z do not likes A)) };", 1, 2, vec![0,1,1,0,1,0], Some(true)),
        ].into_iter().enumerate() {
        let directory = std::env::temp_dir().join(format!(
            "native-comprehension-contexts-{}-{case}",
            std::process::id()
        ));
        let path = directory.join("src/main.miz");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, text).unwrap();
        let loaded = FrontendSourceLoader::new(DiskSourceLoader::new(&directory))
            .load_source_unit(
                SourceUnitRequest {
                    snapshot: BuildSnapshotId::from_published_schema_str(&format!(
                        "mizar-session-build-snapshot-v1:{}",
                        "39".repeat(32)
                    ))
                    .unwrap(),
                    input: SourceInput {
                        package_id: PackageId::new("pkg"),
                        module_path: ModulePath::new("comprehension_contexts"),
                        normalized_path: mizar_session::normalize_path(&directory, &path).unwrap(),
                        edition: Edition::new("2026"),
                        origin: SourceOriginInput::Disk { path },
                    },
                },
                &InMemorySessionIdAllocator::new(),
            )
            .unwrap();
        std::fs::remove_dir_all(directory).unwrap();
        assert_eq!(loaded.source_text.as_ref(), text);
        let mut bridge = SpanBridge::new();
        register_source_unit(&mut bridge, &loaded).unwrap();
        let preprocessed = preprocess(&loaded, &mut bridge).unwrap();
        assert!(preprocessed.diagnostics.is_empty());
        assert!(preprocessed.import_stubs.is_empty());
        let lexical = mizar_lexer::build_lexical_environment(&[], &[]).unwrap();
        let tokens = tokenize(
            TokenizeRequest::with_plan(
                &preprocessed,
                &lexical,
                ParserLexingPlan::for_lexical_text(preprocessed.lexical_text.as_str()),
            ),
            &bridge,
        )
        .unwrap();
        assert!(
            tokens.diagnostics().is_empty(),
            "{:?}",
            tokens.diagnostics()
        );
        let parsed = MizarParserSeam.parse(ParseRequest::new(
            &tokens,
            ParserInputs::from_active_environment(loaded.edition, &lexical),
        ));
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let ast = parsed.ast.unwrap();
        let actual_module = module("comprehension_contexts");
        let shells = DeclarationShellCollector::new(&ast, &actual_module).collect();
        let collected = SignatureProjectionExtractor::new(
            &ast,
            &shells,
            NamespacePath::new(actual_module.path().as_str()),
        )
        .collect(&actual_module);
        assert!(
            collected.diagnostics().is_empty(),
            "{:?}",
            collected.diagnostics()
        );
        let symbols = collected.into_env();
        let resolved = SurfaceResolvedArena::lower(&ast, &actual_module).unwrap();
        mizar_resolve::symbols::validate_source_symbol_env(&resolved, &symbols).unwrap();
        let scope = SourceVariableScopeResolver::resolve_proof_occurrences(
            SourceVariableScopeInput::new(&ast, &actual_module, &symbols),
        );
        let scope = scope.unwrap_or_else(|error| panic!("case {case}: {error:?}"));
        let (scopes, parents) =
            mizar_resolve::symbols::source_proof_scope_maps(&resolved, &symbols, &scope)
                .expect("original sealed lexical maps");
        assert_eq!(ast.node_views().filter(|view| view.kind() == &SurfaceNodeKind::SetComprehension).count(), comp_count);
        let acquisition = SourceVariableSemanticsChecker::source_theorem_binding_env(
            &resolved, &symbols, &scope,
        );
        if primary.is_none() {
            assert_eq!(acquisition.unwrap_err(), "theorems.bindings.invalid_lexical_association", "case {case}");
            continue;
        }
        assert_eq!(scope.bindings().len(), binding_count, "case {case}");
        assert_eq!(scope.references().iter().map(|actual| actual.binding().index()).collect::<Vec<_>>(), winners, "case {case}");
        let mut boundaries = scopes
            .iter()
            .filter(|(id, local)| {
                parents
                    .get(id)
                    .is_none_or(|parent| scopes.get(parent) != Some(local))
            })
            .collect::<Vec<_>>();
        boundaries.sort_by_key(|(_, local)| (local.path().len(), local.path()));
        assert_eq!(boundaries.len(), 1 + comp_count + ast.node_views().filter(|view| matches!(view.kind(), SurfaceNodeKind::QuantifiedFormula(_) | SurfaceNodeKind::ProofBlock | SurfaceNodeKind::BinderExpression | SurfaceNodeKind::DefinitionBlockItem)).count());
        let expected_context = |local: &LocalTermScope| {
            BindingContextId::new(
                boundaries
                    .iter()
                    .position(|(_, scope)| *scope == local)
                    .unwrap(),
            )
        };
        let bindings = acquisition.expect("genuine comprehension lexical acquisition");
        assert_eq!(bindings.contexts().len(), boundaries.len());
        assert_eq!(bindings.bindings().len(), scope.bindings().len());
        assert!(bindings.diagnostics().is_empty());
        for (index, (syntax, local)) in boundaries.iter().enumerate() {
            let id = BindingContextId::new(index);
            let context = bindings.contexts().get(id).unwrap();
            let actual = ast.node_view(**syntax).unwrap();
            let parent = parents.get(syntax).map(|parent| expected_context(scopes.get(parent).unwrap()));
            let (owner, layer) = match actual.kind() {
                SurfaceNodeKind::QuantifiedFormula(_) => (BindingContextOwner::SourceFormula { source_range: actual.range() }, BindingContextLayer::Expression),
                SurfaceNodeKind::ProofBlock => (BindingContextOwner::SourceStatement { source_range: actual.range() }, BindingContextLayer::Proof),
                SurfaceNodeKind::DefinitionBlockItem => (BindingContextOwner::SourceStatement { source_range: actual.range() }, BindingContextLayer::Declaration),
                SurfaceNodeKind::SetComprehension => (BindingContextOwner::SourceComprehension { source_range: actual.range() }, BindingContextLayer::Expression),
                SurfaceNodeKind::TermExpression => {
                    let binder = ast.node_view(*parents.get(syntax).unwrap()).unwrap();
                    assert_eq!(binder.kind(), &SurfaceNodeKind::BinderExpression);
                    assert_eq!(binder.children().last(), Some(*syntax));
                    (BindingContextOwner::SourceFunctionBinder { source_range: binder.range() }, BindingContextLayer::Expression)
                }
                _ if index == 0 => (BindingContextOwner::Module, BindingContextLayer::Module),
                kind => panic!("unexpected source boundary {kind:?}"),
            };
            assert_eq!(context.id, id);
            assert_eq!(context.owner, owner, "case {case}");
            assert_eq!(context.layer, layer);
            assert_eq!(context.parent, parent);
            assert_eq!(context.lexical_scope.as_ref(), parent.map(|_| *local));
            assert_eq!(context.recovery, BindingContextRecovery::Normal);
            let own = scope.bindings().iter().filter(|binding| binding.scope() == *local).map(|binding| BindingId::new(binding.id().index())).collect::<Vec<_>>();
            let visible = scope.bindings().iter().filter(|binding| local.path().starts_with(binding.scope().path())).map(|binding| BindingId::new(binding.id().index())).collect::<Vec<_>>();
            assert_eq!(context.bindings, own);
            assert_eq!(context.visible_bindings, visible, "case {case}");
        }
        let flat = SourceVariableSemanticsChecker::occurrence_binding_env(&scope);
        for ((id, entry), declaration) in bindings.bindings().iter().zip(scope.bindings()) {
            let mut expected = flat.bindings().get(id).unwrap().clone();
            expected.owner_context = expected_context(declaration.scope());
            if comp_count == 0 { expected.type_site = entry.type_site.clone(); }
            assert_eq!(entry, &expected, "every projected field, case {case}");
            assert_eq!(entry.id.index(), declaration.id().index());
            assert_eq!(entry.identity, BinderIdentity::ResolverLocal { scope: declaration.scope().clone(), ordinal: declaration.ordinal(), declaration_range: declaration.range() });
            assert_eq!(entry.visible_after_ordinal, declaration.ordinal());
            assert_eq!(entry.declaration_range, declaration.range());
            if comp_count > 0 { assert_eq!(entry.type_site, BindingTypeSite::Missing); }
        }
        let arena = TypedArena::try_new(
            Some(TypedNodeId::new(resolved.arena().root().index())),
            resolved
                .arena()
                .iter()
                .map(|(id, current)| {
                    TypedNode::new(
                        if current.kind() == &SurfaceNodeKind::TermReference {
                            "source.term.variable-reference".to_owned()
                        } else {
                            format!("{:?}", current.kind())
                        },
                        current.origin().anchor().clone(),
                    )
                    .with_resolved_node(id)
                    .with_children(
                        current
                            .children()
                            .iter()
                            .map(|child| TypedNodeId::new(child.index()))
                            .collect(),
                    )
                })
                .collect(),
        )
        .unwrap();
        // Resolver deliberately excludes the entire real definition subtree.
        // Derive that exclusion from the AST, independently of acquired metadata.
        let definition_ranges = ast.node_views().filter(|view| view.kind() == &SurfaceNodeKind::DefinitionBlockItem).map(|view| view.range()).collect::<Vec<_>>();
        let in_definition = |range: SourceRange| definition_ranges.iter().any(|block| block.start <= range.start && range.end <= block.end);
        let excluded = ast.node_views().filter(|view| view.kind() == &SurfaceNodeKind::TermReference && in_definition(view.range())).collect::<Vec<_>>();
        assert_eq!(excluded.len(), 2 * definition_ranges.len());
        assert!(scope.bindings().iter().all(|binding| !in_definition(binding.range())));
        assert!(scope.references().iter().all(|reference| !in_definition(reference.range())));
        for range in &definition_ranges {
            let context = bindings.contexts().iter().find(|(_, context)| context.owner == BindingContextOwner::SourceStatement { source_range: *range }).unwrap().1;
            assert_eq!(context.layer, BindingContextLayer::Declaration);
            assert!(context.bindings.is_empty() && context.visible_bindings.is_empty());
            let formals = ast.node_views().filter(|view| view.kind() == &SurfaceNodeKind::QualifiedVariableSegment && in_definition(view.range())).collect::<Vec<_>>();
            assert_eq!(formals.len(), 1);
            for formal in [formals[0].children()[0], formals[0].children()[2]] {
                assert!(scope.bindings().iter().all(|binding| binding.node() != formal));
            }
        }
        let mut leaves = ast
            .node_views()
            .filter(|view| view.kind() == &SurfaceNodeKind::TermReference && !in_definition(view.range()))
            .collect::<Vec<_>>();
        leaves.sort_by_key(|view| view.range().start);
        assert_eq!(
            leaves.len(),
            scope.references().len(),
            "complete structural leaf census"
        );
        let input = SourcePrimaryTermHandoffInput {
            source_id: ast.source_id,
            module_id: actual_module,
            terms: leaves
                .iter()
                .enumerate()
                .map(|(ordinal, leaf)| SourcePrimaryTermInput {
                    site: node(resolved.resolved_node_for(leaf.id()).unwrap().index()),
                    source_range: leaf.range(),
                    source_ordinal: ordinal,
                    context: expected_context(scopes.get(&leaf.id()).unwrap()),
                    recovery: SourcePrimaryTermRecovery::Normal,
                    spelling: text[leaf.range().start..leaf.range().end].to_owned(),
                    kind: SourcePrimaryTermKind::VariableReference,
                    role: SourcePrimaryTermRole::Value,
                    parent: None,
                })
                .collect(),
            references: leaves
                .iter()
                .enumerate()
                .map(|(ordinal, leaf)| {
                    let actual = scope
                        .references()
                        .iter()
                        .find(|actual| actual.node() == leaf.id())
                        .unwrap();
                    SourcePrimaryTermReferenceInput {
                        term: SourcePrimaryTermId::new(ordinal),
                        binding: BindingId::new(actual.binding().index()),
                        role: SourcePrimaryTermReferenceRole::Variable,
                    }
                })
                .collect(),
            numeric_type_requests: Vec::new(),
        };
        let provenance = (resolved.clone(), symbols.clone(), scopes.clone(), parents.clone(), ast.snapshot_text(), loaded.source_text.clone());
        let pristine = (input.clone(), bindings.clone(), arena.clone(), scope.clone());
        let consume = |candidate, environment: &BindingEnv| SourcePrimaryTermProducer::build_from_scoped_theorem_occurrences(
            candidate, environment, &arena, &resolved, &symbols, &scope,
        );
        let handoff = consume(input.clone(), &bindings).expect("complete real scoped comprehension Primary consumer");
        assert_eq!(handoff.terms().len(), leaves.len());
        assert_eq!(handoff.references().len(), leaves.len());
        assert!(handoff.numeric_type_requests().is_empty());
        for ((_, reference), actual) in handoff.references().iter().zip(scope.references()) {
            let term = handoff.terms().get(reference.term()).unwrap();
            let expected = &input.terms[reference.term().index()];
            assert_eq!(term.site(), &expected.site);
            assert_eq!(term.source_range(), actual.range());
            assert_eq!(term.source_ordinal(), reference.term().index());
            assert_eq!(term.spelling(), actual.spelling());
            assert_eq!(term.kind(), SourcePrimaryTermKind::VariableReference);
            assert_eq!(term.role(), SourcePrimaryTermRole::Value);
            assert_eq!(term.recovery(), SourcePrimaryTermRecovery::Normal);
            assert_eq!(term.parent(), None);
            assert_eq!(term.context(), expected_context(actual.scope()));
            // Lookup uses written declaration events, independently of reference clocks.
            let declaration_clock = scope.bindings().iter().filter(|binding| binding.range().end <= actual.range().start).count();
            let declaration = &scope.bindings()[actual.binding().index()];
            let lookup_clock = if declaration.range().end > actual.range().start {
                assert_eq!(declaration.kind(), mizar_resolve::names::SourceVariableBindingKind::ComprehensionGenerator);
                assert!(SourcePrimaryTermProducer::build(input.clone(), &bindings, &arena).is_err());
                assert!(SourcePrimaryTermProducer::build_from_occurrences(input.clone(), &flat, &arena, &scope).is_err());
                declaration.ordinal() + 1
            } else { declaration_clock };
            assert_eq!(bindings.lookup(&BindingLookupSite::new(actual.spelling(), term.context(), Some(actual.scope().clone()), lookup_clock)), Ok(BindingLookupResult::Local(BindingId::new(actual.binding().index()))));
            assert_eq!(reference.binding().index(), actual.binding().index());
            assert_eq!(reference.lexical_scope(), Some(actual.scope()));
            assert_eq!(reference.use_ordinal(), actual.ordinal());
        }
        assert_eq!(consume(input.clone(), &bindings), Ok(handoff.clone()));
        if (19..=22).contains(&case) || case >= 39 || text.contains(" does not ") || text.contains(" do not ") {
            use crate::source_application::{SourceFunctorApplicationHandoffInput, SourceFunctorApplicationProducer};
            use crate::source_atomic_formula::*;
            use crate::source_formula_composition::*;
            use crate::source_set_term::*;
            use mizar_resolve::env::SymbolKind;

            let predicates = symbols.symbols().iter().filter(|entry| entry.kind() == SymbolKind::Predicate).collect::<Vec<_>>();
            let [predicate] = predicates.as_slice() else { panic!("one original local binary predicate"); };
            let declaration = ast.node_views().find(|view| view.kind() == &SurfaceNodeKind::PredicateDefinition).unwrap();
            let pattern = declaration.child_views().find(|view| view.kind() == &SurfaceNodeKind::PredicatePattern).unwrap();
            let pattern_text = pattern.child_views().map(|view| text[view.range().start..view.range().end].to_owned()).collect::<Vec<_>>();
            assert_eq!(pattern_text.len(), 3);
            assert_eq!(predicate.primary_spelling(), pattern_text.join(" "));
            assert_eq!(predicate.notation_spelling(), Some(predicate.primary_spelling()));
            assert_eq!(predicate.origin().anchor(), &SourceAnchor::Range(declaration.range()));
            assert_eq!(predicate.origin().source_id(), ast.source_id);
            assert_eq!(predicate.origin().module_id(), &input.module_id);
            assert!(predicate.origin().import_edge().is_none() && !predicate.origin().is_recovered());
            let definition = symbols.definitions().by_symbol(predicate.symbol()).unwrap();
            assert_eq!(definition.kind(), DefinitionKind::Predicate);
            assert_eq!(definition.origin(), predicate.origin());
            assert_eq!(definition.contribution(), predicate.contribution());
            assert_eq!(definition.notation_shape(), Some(predicate.primary_spelling()));
            let contribution = symbols.contributions().get(predicate.contribution()).unwrap();
            assert_eq!(contribution.kind(), &ContributionKind::LocalSource { source_id: ast.source_id });
            assert_eq!(contribution.module(), &input.module_id);
            assert!(contribution.effects().symbols().contains(predicate.symbol()));
            assert!(contribution.effects().definitions().contains(&definition.id()));
            let mut comprehensions = ast.node_views().filter(|view| view.kind() == &SurfaceNodeKind::SetComprehension).collect::<Vec<_>>();
            comprehensions.sort_by_key(|view| view.range().start);
            assert_eq!(ast.node_views().filter(|view| view.kind() == &SurfaceNodeKind::PredicateApplication && !in_definition(view.range())).count(), comp_count);
            let primary_before = handoff.clone();
            let mut set_input = SourceSetTermHandoffInput {
                source_id: ast.source_id, module_id: input.module_id.clone(),
                terms: Vec::new(), wrappers: Vec::new(), generators: Vec::new(),
                type_sites: Vec::new(), conditions: Vec::new(), edges: Vec::new(), requests: Vec::new(),
            };
            let mut nodes = arena.iter().map(|(_, node)| node.clone()).collect::<Vec<_>>();
            let mut retagged = Vec::new();
            let canonical_spelling = |view: syntax::SurfaceNodeView<'_>| {
                let mut leaves = ast.node_views().filter(|leaf| {
                    leaf.children().is_empty()
                        && view.range().start <= leaf.range().start
                        && leaf.range().end <= view.range().end
                }).collect::<Vec<_>>();
                leaves.sort_by_key(|leaf| leaf.range().start);
                leaves.into_iter().map(|leaf| text[leaf.range().start..leaf.range().end].to_owned()).collect::<Vec<_>>().join(" ")
            };
            for (ordinal, comprehension) in comprehensions.iter().copied().enumerate() {
                let term = SourceSetTermId::new(ordinal);
                let generator_id = SourceSetGeneratorId::new(ordinal);
                let type_id = SourceSetTypeSiteId::new(ordinal);
                let mapper = comprehension.child_views().find(|view| view.kind() == &SurfaceNodeKind::TermExpression).unwrap();
                let [mapper_leaf] = mapper.children() else { panic!("one direct mapper reference"); };
                let mapper_primary = SourcePrimaryTermId::new(leaves.iter().position(|view| view.id() == *mapper_leaf).unwrap());
                let generator = comprehension.child_views().find(|view| view.kind() == &SurfaceNodeKind::ComprehensionVariableSegment).unwrap();
                let identifier = ast.node_view(generator.children()[0]).unwrap();
                let written_type = generator.child_views().find(|view| view.kind() == &SurfaceNodeKind::TypeExpression).unwrap();
                let [type_head] = written_type.children() else { panic!("bare written type"); };
                let type_head = ast.node_view(*type_head).unwrap();
                assert_eq!(type_head.kind(), &SurfaceNodeKind::TypeHead);
                assert_eq!(&text[written_type.range().start..written_type.range().end], "set");
                assert_eq!(written_type.range(), type_head.range());
                let colon = comprehension.child_views().find(|view| &text[view.range().start..view.range().end] == ":").unwrap();
                let condition = comprehension.child_views().find(|view| view.kind() == &SurfaceNodeKind::FormulaExpression).unwrap();
                let context = expected_context(scopes.get(&mapper.id()).unwrap());
                assert_eq!(bindings.contexts().get(context).unwrap().owner, BindingContextOwner::SourceComprehension { source_range: comprehension.range() });
                let actual_generator = scope.bindings().iter().find(|binding| binding.node() == identifier.id()).unwrap();
                let binding = BindingId::new(actual_generator.id().index());
                assert_eq!(actual_generator.kind(), mizar_resolve::names::SourceVariableBindingKind::ComprehensionGenerator);
                assert_eq!(actual_generator.scope(), scopes.get(&mapper.id()).unwrap());
                assert_eq!(actual_generator.range(), identifier.range());
                let condition_generator = scope.references().iter().filter(|reference| condition.range().start <= reference.range().start && reference.range().end <= condition.range().end && reference.spelling() == actual_generator.spelling()).collect::<Vec<_>>();
                assert!(!condition_generator.is_empty());
                assert!(condition_generator.iter().all(|reference| reference.binding() == actual_generator.id() && reference.scope() == actual_generator.scope()));
                assert_eq!(bindings.bindings().get(binding).unwrap().type_site, BindingTypeSite::Missing);
                assert_eq!(handoff.references().get(SourcePrimaryTermReferenceId::new(mapper_primary.index())).unwrap().binding(), binding);
                let spelling = |view: syntax::SurfaceNodeView<'_>| text[view.range().start..view.range().end].to_owned();
                let site = |view: syntax::SurfaceNodeView<'_>| node(resolved.resolved_node_for(view.id()).unwrap().index());
                set_input.terms.push(SourceSetTermInput { site: site(comprehension), source_range: comprehension.range(), source_ordinal: ordinal, context, recovery: SourceSetTermRecovery::Normal, spelling: canonical_spelling(comprehension), kind: SourceSetTermKind::Comprehension });
                set_input.generators.push(SourceSetGeneratorInput { term, ordinal: 0, site: site(identifier), source_range: identifier.range(), spelling: spelling(identifier), context, recovery: SourceSetTermRecovery::Normal, type_site: type_id });
                set_input.type_sites.push(SourceSetTypeSiteInput { owner: SourceSetTypeOwner::Generator(generator_id), site: site(written_type), source_range: written_type.range(), spelling: spelling(written_type), head_site: site(type_head), head_range: type_head.range(), head_spelling: spelling(type_head), context, recovery: SourceSetTermRecovery::Normal, head: SourceSetTypeHead::BuiltinSet });
                set_input.conditions.push(SourceSetConditionInput { term, ordinal: 0, colon_site: site(colon), colon_range: colon.range(), colon_spelling: spelling(colon), condition_site: site(condition), source_range: condition.range(), spelling: canonical_spelling(condition), recovery: SourceSetTermRecovery::Normal });
                set_input.edges.push(SourceSetEdgeInput { term, ordinal: 0, role: SourceSetEdgeRole::ComprehensionMapper, target: SourceSetTarget::Primary(mapper_primary) });
                set_input.requests.extend([
                    SourceSetRequestInput { term, ordinal: 0, kind: SourceSetRequestKind::GeneratorSethood, generator: Some(generator_id), type_site: Some(type_id) },
                    SourceSetRequestInput { term, ordinal: 1, kind: SourceSetRequestKind::ResultType, generator: None, type_site: None },
                ]);
                for (view, key) in [(comprehension, "source.term.set.comprehension"), (identifier, "source.term.set.comprehension-generator"), (written_type, "source.term.set.target-type"), (type_head, "source.term.set.target-type-head"), (colon, "source.term.set.comprehension-condition-colon"), (condition, "source.term.set.comprehension-condition")] {
                    let index = site(view).node().index();
                    assert!(!retagged.contains(&index));
                    retagged.push(index);
                    nodes[index].kind = key.into();
                }
                for view in ast.node_views().filter(|view| condition.range().start <= view.range().start && view.range().end <= condition.range().end) {
                    let key = match view.kind() {
                        SurfaceNodeKind::ParenthesizedFormula => "source.formula.atomic.parenthesized",
                        SurfaceNodeKind::PredicateApplication => "source.formula.atomic.predicate",
                        SurfaceNodeKind::PredicateSegment => "source.formula.atomic.predicate-segment",
                        SurfaceNodeKind::PredicateHead => "source.formula.atomic.predicate-head",
                        _ => continue,
                    };
                    let index = site(view).node().index();
                    assert!(!retagged.contains(&index));
                    retagged.push(index);
                    nodes[index].kind = key.into();
                    if view.kind() == &SurfaceNodeKind::PredicateSegment {
                        for child in view.children() {
                            let token = ast.node_view(*child).unwrap();
                            let spelling = &text[token.range().start..token.range().end];
                            let token_key = match spelling {
                                "does" | "do" => "source.formula.atomic.predicate-negation-verb",
                                "not" => "source.formula.atomic.predicate-negation-not",
                                _ => continue,
                            };
                            let token_index = site(token).node().index();
                            assert!(!retagged.contains(&token_index));
                            retagged.push(token_index);
                            nodes[token_index].kind = token_key.into();
                        }
                    }
                }
            }
            for comprehension in &comprehensions {
                let rhs = ast.node_view(*parents.get(&comprehension.id()).unwrap()).unwrap();
                let membership = ast.node_view(*parents.get(&rhs.id()).unwrap()).unwrap();
                assert_eq!(rhs.kind(), &SurfaceNodeKind::TermExpression);
                assert_eq!(membership.kind(), &SurfaceNodeKind::BuiltinPredicateApplication);
                let index = resolved.resolved_node_for(membership.id()).unwrap().index();
                retagged.push(index);
                nodes[index].kind = "source.formula.atomic.membership".into();
            }
            let set_arena = TypedArena::try_new(arena.root(), nodes).unwrap();
            let set = SourceSetTermProducer::build(set_input.clone(), &bindings, &handoff, None, None, &set_arena).unwrap_or_else(|error| panic!("genuine scoped Set case {case}: {error:?}"));
            assert_eq!(set.source_id(), ast.source_id);
            assert_eq!(set.module_id(), &input.module_id);
            assert_eq!(set.primary_term_fingerprint(), handoff.debug_text());
            assert!(set.application_fingerprint().is_none() && set.structure_fingerprint().is_none());
            assert_eq!((set.terms().len(), set.wrappers().len(), set.generators().len(), set.type_sites().len(), set.conditions().len(), set.edges().len(), set.requests().len()), (comp_count, 0, comp_count, comp_count, comp_count, comp_count, 2 * comp_count));
            for ((id, row), expected) in set.terms().iter().zip(&set_input.terms) {
                assert_eq!((row.site(), row.source_range(), row.source_ordinal(), row.context(), row.recovery(), row.spelling(), row.kind()), (&expected.site, expected.source_range, id.index(), expected.context, SourceSetTermRecovery::Normal, expected.spelling.as_str(), SourceSetTermKind::Comprehension));
            }
            for ((id, row), expected) in set.generators().iter().zip(&set_input.generators) {
                assert_eq!((row.term(), row.ordinal(), row.site(), row.source_range(), row.spelling(), row.context(), row.recovery(), row.type_site()), (SourceSetTermId::new(id.index()), 0, &expected.site, expected.source_range, expected.spelling.as_str(), expected.context, SourceSetTermRecovery::Normal, SourceSetTypeSiteId::new(id.index())));
            }
            for ((id, row), expected) in set.type_sites().iter().zip(&set_input.type_sites) {
                assert_eq!((row.owner(), row.site(), row.source_range(), row.spelling(), row.head_site(), row.head_range(), row.head_spelling(), row.context(), row.recovery(), row.head()), (SourceSetTypeOwner::Generator(SourceSetGeneratorId::new(id.index())), &expected.site, expected.source_range, "set", &expected.head_site, expected.source_range, "set", expected.context, SourceSetTermRecovery::Normal, SourceSetTypeHead::BuiltinSet));
            }
            for ((id, row), expected) in set.conditions().iter().zip(&set_input.conditions) {
                assert_eq!((row.term(), row.ordinal(), row.colon_site(), row.colon_range(), row.colon_spelling(), row.condition_site(), row.source_range(), row.spelling(), row.recovery()), (SourceSetTermId::new(id.index()), 0, &expected.colon_site, expected.colon_range, ":", &expected.condition_site, expected.source_range, expected.spelling.as_str(), SourceSetTermRecovery::Normal));
            }
            for ((id, row), expected) in set.edges().iter().zip(&set_input.edges) {
                assert_eq!((row.term(), row.ordinal(), row.role(), row.target()), (SourceSetTermId::new(id.index()), 0, SourceSetEdgeRole::ComprehensionMapper, expected.target));
            }
            for (id, row) in set.requests().iter() {
                let ordinal = id.index() % 2;
                let owner = id.index() / 2;
                assert_eq!((row.term(), row.ordinal(), row.kind(), row.generator(), row.type_site()), (SourceSetTermId::new(owner), ordinal, if ordinal == 0 { SourceSetRequestKind::GeneratorSethood } else { SourceSetRequestKind::ResultType }, (ordinal == 0).then_some(SourceSetGeneratorId::new(owner)), (ordinal == 0).then_some(SourceSetTypeSiteId::new(owner))));
            }
            for ((id, original), (_, actual)) in arena.iter().zip(set_arena.iter()) {
                let mut expected = original.clone();
                if retagged.contains(&id.index()) { expected.kind = actual.kind.clone(); }
                assert_eq!(actual, &expected, "only original Set/Atomic metadata keys are retagged");
                assert_eq!(actual.typing, TypingState::Unknown);
                assert_eq!(actual.links, crate::typed_ast::TypedNodeLinks::default());
            }
            let set_pristine = (set_input.clone(), set_arena.clone(), bindings.clone(), handoff.clone());
            let build_set = |candidate| SourceSetTermProducer::build(candidate, &bindings, &handoff, None, None, &set_arena);
            for mutation in 0..14 {
                let mut candidate = set_input.clone();
                match mutation {
                    0 => candidate.source_id = other_source_id(),
                    1 => candidate.terms[0].site = input.terms[0].site.clone(),
                    2 => candidate.terms[0].context = BindingContextId::new(0),
                    3 => candidate.edges[0].target = SourceSetTarget::Primary(SourcePrimaryTermId::new(0)),
                    4 => candidate.type_sites[0].owner = SourceSetTypeOwner::Generator(SourceSetGeneratorId::new(comp_count)),
                    5 => candidate.type_sites[0].head_site = candidate.type_sites[0].site.clone(),
                    6 => candidate.generators[0].context = BindingContextId::new(0),
                    7 => candidate.generators[0].type_site = SourceSetTypeSiteId::new(comp_count),
                    8 => candidate.generators[0].site = input.terms[0].site.clone(),
                    9 => candidate.conditions[0].condition_site = candidate.terms[0].site.clone(),
                    10 => candidate.conditions[0].colon_range.start += 1,
                    11 => candidate.requests[0].generator = None,
                    12 => candidate.requests[1].type_site = Some(SourceSetTypeSiteId::new(0)),
                    _ => candidate.requests.clear(),
                }
                assert!(build_set(candidate).is_err(), "genuine Set case {case} mutation {mutation}");
                assert_eq!(build_set(set_input.clone()), Ok(set.clone()));
            }
            assert_eq!(SourcePrimaryTermProducer::build_from_scoped_theorem_occurrences(input.clone(), &bindings, &set_arena, &resolved, &symbols, &scope), Ok(primary_before.clone()));
            let mut aggregate_input = SourceAtomicFormulaHandoffInput {
                source_id: ast.source_id, module_id: input.module_id.clone(),
                formulas: Vec::new(), wrappers: Vec::new(), predicate_segments: Vec::new(),
                predicate_heads: Vec::new(), candidates: Vec::new(), type_sites: Vec::new(),
                attributes: Vec::new(), edges: Vec::new(), requests: Vec::new(),
            };
            let mut physical_mutations = Vec::new();
            let mut typed_replay_mutations = Vec::new();
            for (pair, comprehension) in comprehensions.iter().copied().enumerate() {
                let conditions = comprehension.child_views().filter(|view| view.kind() == &SurfaceNodeKind::FormulaExpression).collect::<Vec<_>>();
                let [condition] = conditions.as_slice() else { panic!("one actual condition"); };
                let mut formula = *condition;
                let mut formula_wrappers = Vec::new();
                loop {
                    match formula.kind() {
                        SurfaceNodeKind::FormulaExpression => {
                            let [child] = formula.children() else { panic!("one actual formula child"); };
                            formula = ast.node_view(*child).unwrap();
                        }
                        SurfaceNodeKind::ParenthesizedFormula => {
                            let [open, child, close] = formula.children() else { panic!("actual transparent formula bridge"); };
                            assert_eq!(&text[ast.node_view(*open).unwrap().range().start..ast.node_view(*open).unwrap().range().end], "(");
                            assert_eq!(&text[ast.node_view(*close).unwrap().range().start..ast.node_view(*close).unwrap().range().end], ")");
                            formula_wrappers.push(formula);
                            formula = ast.node_view(*child).unwrap();
                        }
                        _ => break,
                    }
                }
                assert_eq!(formula.kind(), &SurfaceNodeKind::PredicateApplication);
                let segment_count = formula.children().len();
                assert!(segment_count > 0);
                let segment = ast.node_view(formula.children()[0]).unwrap();
                assert_eq!(segment.kind(), &SurfaceNodeKind::PredicateSegment);
                let (left, head_id, right, polarity_sites) = match segment.children() {
                    [left, head, right] => (*left, *head, *right, None),
                    [left, verb, not, head, right] => (*left, *head, *right, Some((*verb, *not))),
                    children => panic!("singleton binary segment children: {children:?}"),
                };
                let head = ast.node_view(head_id).unwrap();
                assert_eq!(head.kind(), &SurfaceNodeKind::PredicateHead);
                let head_text = &text[head.range().start..head.range().end];
                assert_eq!(head_text, pattern_text[1]);
                assert!(declaration.range().end < formula.range().start);
                let context = expected_context(scopes.get(&formula.id()).unwrap());
                assert_eq!(bindings.contexts().get(context).unwrap().owner, BindingContextOwner::SourceComprehension { source_range: comprehension.range() });
                let operands = [left, right].map(|expression| {
                    let expression = ast.node_view(expression).unwrap();
                    assert_eq!(expression.kind(), &SurfaceNodeKind::TermExpression);
                    let [leaf] = expression.children() else { panic!("direct Primary operand"); };
                    let ordinal = leaves.iter().position(|view| view.id() == *leaf).unwrap();
                    let term = handoff.terms().get(SourcePrimaryTermId::new(ordinal)).unwrap();
                    assert_eq!(term.context(), context);
                    assert_eq!(term.site(), &node(resolved.resolved_node_for(*leaf).unwrap().index()));
                    assert_eq!(input.references[ordinal].binding.index(), winners[ordinal]);
                    SourcePrimaryTermId::new(ordinal)
                });
                let negative_polarity = polarity_sites.map(|(verb, not)| {
                    let verb = ast.node_view(verb).unwrap();
                    let not = ast.node_view(not).unwrap();
                    SourcePredicateSegmentPolarityInput::Negative {
                        verb_site: node(resolved.resolved_node_for(verb.id()).unwrap().index()),
                        verb_range: verb.range(), verb_spelling: text[verb.range().start..verb.range().end].to_owned(),
                        verb_recovery: SourceAtomicFormulaRecovery::Normal,
                        not_site: node(resolved.resolved_node_for(not.id()).unwrap().index()),
                        not_range: not.range(), not_spelling: text[not.range().start..not.range().end].to_owned(),
                        not_recovery: SourceAtomicFormulaRecovery::Normal,
                    }
                }).unwrap_or(SourcePredicateSegmentPolarityInput::Positive);
                let negative = matches!(negative_polarity, SourcePredicateSegmentPolarityInput::Negative { .. });
                let mut atomic_input = SourceAtomicFormulaHandoffInput {
                    source_id: ast.source_id, module_id: input.module_id.clone(),
                    formulas: vec![SourceAtomicFormulaInput {
                        site: node(resolved.resolved_node_for(formula.id()).unwrap().index()), source_range: formula.range(), source_ordinal: 0,
                        context, recovery: SourceAtomicFormulaRecovery::Normal, spelling: canonical_spelling(formula), kind: SourceAtomicFormulaKind::PredicateApplication,
                    }],
                    wrappers: formula_wrappers.iter().enumerate().map(|(ordinal, wrapper)| SourceAtomicWrapperInput {
                        formula: SourceAtomicFormulaId::new(0), ordinal,
                        site: node(resolved.resolved_node_for(wrapper.id()).unwrap().index()),
                        source_range: wrapper.range(), context, recovery: SourceAtomicFormulaRecovery::Normal,
                        spelling: canonical_spelling(*wrapper),
                    }).collect(),
                    predicate_segments: vec![SourcePredicateSegmentInput {
                        formula: SourceAtomicFormulaId::new(0), ordinal: 0, site: node(resolved.resolved_node_for(segment.id()).unwrap().index()),
                        source_range: segment.range(), context, recovery: SourceAtomicFormulaRecovery::Normal, spelling: text[segment.range().start..segment.range().end].to_owned(),
                        head: SourcePredicateHeadId::new(0), polarity: negative_polarity.clone(),
                        left_edge: SourceAtomicEdgeId::new(0), right_edge: SourceAtomicEdgeId::new(1),
                    }],
                    predicate_heads: vec![SourcePredicateHeadInput {
                        formula: SourceAtomicFormulaId::new(0), site: node(resolved.resolved_node_for(head.id()).unwrap().index()), source_range: head.range(),
                        context, recovery: SourceAtomicFormulaRecovery::Normal, spelling: head_text.to_owned(), left_arity: 1, right_arity: 1,
                    }],
                    candidates: vec![SourcePredicateCandidateInput { head: SourcePredicateHeadId::new(0), ordinal: 0, symbol: predicate.symbol().clone(), contribution: predicate.contribution() }],
                    type_sites: Vec::new(), attributes: Vec::new(),
                    edges: operands.into_iter().enumerate().map(|(ordinal, id)| SourceAtomicEdgeInput {
                        formula: SourceAtomicFormulaId::new(0), ordinal, role: if ordinal == 0 { SourceAtomicEdgeRole::PredicateLeftArgument } else { SourceAtomicEdgeRole::PredicateRightArgument }, target: SourceAtomicTermTarget::Primary(id),
                    }).collect(),
                    requests: vec![SourceAtomicRequestInput { formula: SourceAtomicFormulaId::new(0), ordinal: 0, kind: SourceAtomicRequestKind::PredicateCandidateSignature, edge: None, candidate: Some(SourcePredicateCandidateId::new(0)), type_site: None, attribute: None }],
                };
                for (ordinal, segment_id) in formula.children().iter().enumerate().skip(1) {
                    let current = ast.node_view(*segment_id).unwrap();
                    assert_eq!(current.kind(), &SurfaceNodeKind::PredicateSegment);
                    let (head_id, right_id, polarity) = match current.children() {
                        [head, right] => (*head, *right, SourcePredicateSegmentPolarityInput::Positive),
                        [verb, not, head, right] => {
                            let verb = ast.node_view(*verb).unwrap();
                            let not = ast.node_view(*not).unwrap();
                            (*head, *right, SourcePredicateSegmentPolarityInput::Negative {
                                verb_site: node(resolved.resolved_node_for(verb.id()).unwrap().index()), verb_range: verb.range(), verb_spelling: text[verb.range().start..verb.range().end].into(), verb_recovery: SourceAtomicFormulaRecovery::Normal,
                                not_site: node(resolved.resolved_node_for(not.id()).unwrap().index()), not_range: not.range(), not_spelling: text[not.range().start..not.range().end].into(), not_recovery: SourceAtomicFormulaRecovery::Normal,
                            })
                        }
                        children => panic!("written continuation children: {children:?}"),
                    };
                    let head = ast.node_view(head_id).unwrap();
                    assert_eq!(head.kind(), &SurfaceNodeKind::PredicateHead);
                    assert_eq!(&text[head.range().start..head.range().end], pattern_text[1]);
                    let right = ast.node_view(right_id).unwrap();
                    assert_eq!(right.kind(), &SurfaceNodeKind::TermExpression);
                    let [leaf] = right.children() else { panic!("direct continuation Primary"); };
                    let primary = SourcePrimaryTermId::new(leaves.iter().position(|view| view.id() == *leaf).unwrap());
                    assert_eq!(handoff.terms().get(primary).unwrap().context(), context);
                    atomic_input.predicate_segments.push(SourcePredicateSegmentInput {
                        formula: SourceAtomicFormulaId::new(0), ordinal, site: node(resolved.resolved_node_for(current.id()).unwrap().index()), source_range: current.range(), context, recovery: SourceAtomicFormulaRecovery::Normal, spelling: text[current.range().start..current.range().end].into(), head: SourcePredicateHeadId::new(ordinal), polarity,
                        left_edge: SourceAtomicEdgeId::new(ordinal), right_edge: SourceAtomicEdgeId::new(ordinal + 1),
                    });
                    atomic_input.predicate_heads.push(SourcePredicateHeadInput { formula: SourceAtomicFormulaId::new(0), site: node(resolved.resolved_node_for(head.id()).unwrap().index()), source_range: head.range(), context, recovery: SourceAtomicFormulaRecovery::Normal, spelling: pattern_text[1].clone(), left_arity: 1, right_arity: 1 });
                    atomic_input.candidates.push(SourcePredicateCandidateInput { head: SourcePredicateHeadId::new(ordinal), ordinal: 0, symbol: predicate.symbol().clone(), contribution: predicate.contribution() });
                    atomic_input.edges.last_mut().unwrap().role = SourceAtomicEdgeRole::PredicateChainBoundary;
                    atomic_input.edges.push(SourceAtomicEdgeInput { formula: SourceAtomicFormulaId::new(0), ordinal: ordinal + 1, role: SourceAtomicEdgeRole::PredicateRightArgument, target: SourceAtomicTermTarget::Primary(primary) });
                    atomic_input.requests.push(SourceAtomicRequestInput { formula: SourceAtomicFormulaId::new(0), ordinal, kind: SourceAtomicRequestKind::PredicateCandidateSignature, edge: None, candidate: Some(SourcePredicateCandidateId::new(ordinal)), type_site: None, attribute: None });
                }
                // All rows above come from actual written children, including shared boundaries.
                let atomic_arena = set_arena.clone();
                if !formula_wrappers.is_empty() {
                    assert_eq!(atomic_input.wrappers.len(), formula_wrappers.len());
                    for (ordinal, (actual, row)) in formula_wrappers.iter().zip(&atomic_input.wrappers).enumerate() {
                        assert_eq!((row.formula, row.ordinal, &row.site, row.source_range, row.context, row.recovery, row.spelling.as_str()), (
                            SourceAtomicFormulaId::new(0), ordinal,
                            &node(resolved.resolved_node_for(actual.id()).unwrap().index()), actual.range(), context,
                            SourceAtomicFormulaRecovery::Normal, canonical_spelling(*actual).as_str(),
                        ));
                    }
                }
                assert!(SourceAtomicFormulaProducer::build(atomic_input.clone(), &bindings, &symbols, &handoff, None, None, None, &set_arena).is_ok(), "genuine whole-chain isolated Atomic before paired ownership");
                let rhs = ast.node_view(*parents.get(&comprehension.id()).unwrap()).unwrap();
                let membership = ast.node_view(*parents.get(&rhs.id()).unwrap()).unwrap();
                let [lhs, token, actual_rhs] = membership.children() else { panic!("actual Membership children"); };
                assert_eq!(*actual_rhs, rhs.id());
                assert_eq!(rhs.children(), &[comprehension.id()]);
                let lhs = ast.node_view(*lhs).unwrap();
                let token = ast.node_view(*token).unwrap();
                assert_eq!(lhs.kind(), &SurfaceNodeKind::TermExpression);
                assert_eq!(&text[token.range().start..token.range().end], "in");
                let [left_leaf] = lhs.children() else { panic!("bare original left Primary"); };
                let left_primary = SourcePrimaryTermId::new(leaves.iter().position(|view| view.id() == *left_leaf).unwrap());
                let outer_context = expected_context(scopes.get(&membership.id()).unwrap());
                assert_ne!(outer_context, context);
                assert_eq!(bindings.contexts().get(context).unwrap().parent, Some(outer_context));
                let set_id = set.terms().iter().find(|(_, row)| row.site() == &node(resolved.resolved_node_for(comprehension.id()).unwrap().index())).unwrap().0;
                let outer_input = SourceAtomicFormulaHandoffInput {
                    source_id: ast.source_id, module_id: input.module_id.clone(),
                    formulas: vec![SourceAtomicFormulaInput {
                        site: node(resolved.resolved_node_for(membership.id()).unwrap().index()), source_range: membership.range(), source_ordinal: 0,
                        context: outer_context, recovery: SourceAtomicFormulaRecovery::Normal, spelling: canonical_spelling(membership), kind: SourceAtomicFormulaKind::Membership,
                    }],
                    wrappers: Vec::new(), predicate_segments: Vec::new(), predicate_heads: Vec::new(), candidates: Vec::new(), type_sites: Vec::new(), attributes: Vec::new(),
                    edges: vec![
                        SourceAtomicEdgeInput { formula: SourceAtomicFormulaId::new(0), ordinal: 0, role: SourceAtomicEdgeRole::BuiltinLeftOperand, target: SourceAtomicTermTarget::Primary(left_primary) },
                        SourceAtomicEdgeInput { formula: SourceAtomicFormulaId::new(0), ordinal: 1, role: SourceAtomicEdgeRole::BuiltinRightOperand, target: SourceAtomicTermTarget::SetTerm(set_id) },
                    ],
                    requests: vec![SourceAtomicRequestInput { formula: SourceAtomicFormulaId::new(0), ordinal: 0, kind: SourceAtomicRequestKind::OperandExpectedType, edge: Some(SourceAtomicEdgeId::new(1)), candidate: None, type_site: None, attribute: None }],
                };
                let outer = SourceAtomicFormulaProducer::build(outer_input.clone(), &bindings, &symbols, &handoff, None, None, Some(&set), &set_arena)
                    .unwrap_or_else(|error| panic!("genuine original-source outer Membership case {case}: {error:?}"));
                assert_eq!(outer.source_id(), ast.source_id);
                assert_eq!(outer.module_id(), &input.module_id);
                assert_eq!(outer.primary_term_fingerprint(), handoff.debug_text());
                assert_eq!(outer.set_term_fingerprint(), Some(set.debug_text().as_str()));
                assert!(outer.application_fingerprint().is_none() && outer.structure_fingerprint().is_none());
                assert_eq!((outer.formulas().len(), outer.edges().len(), outer.requests().len()), (1, 2, 1));
                assert!(outer.wrappers().is_empty() && outer.predicate_segments().is_empty() && outer.predicate_heads().is_empty() && outer.candidates().is_empty() && outer.type_sites().is_empty() && outer.attributes().is_empty());
                let row = outer.formulas().get(SourceAtomicFormulaId::new(0)).unwrap();
                assert_eq!((row.site(), row.source_range(), row.source_ordinal(), row.context(), row.recovery(), row.spelling(), row.kind()), (&outer_input.formulas[0].site, membership.range(), 0, outer_context, SourceAtomicFormulaRecovery::Normal, outer_input.formulas[0].spelling.as_str(), SourceAtomicFormulaKind::Membership));
                for ((id, row), expected) in outer.edges().iter().zip(&outer_input.edges) {
                    assert_eq!((row.formula(), row.ordinal(), row.role(), row.target()), (SourceAtomicFormulaId::new(0), id.index(), expected.role, expected.target));
                }
                let row = outer.requests().get(SourceAtomicRequestId::new(0)).unwrap();
                assert_eq!((row.formula(), row.ordinal(), row.kind(), row.edge(), row.candidate(), row.type_site(), row.attribute()), (SourceAtomicFormulaId::new(0), 0, SourceAtomicRequestKind::OperandExpectedType, Some(SourceAtomicEdgeId::new(1)), None, None, None));
                let outer_pristine = (outer_input.clone(), outer.clone(), set.clone());
                let build_outer = |candidate| SourceAtomicFormulaProducer::build(candidate, &bindings, &symbols, &handoff, None, None, Some(&set), &set_arena);
                assert_eq!(build_outer(outer_input.clone()), Ok(outer.clone()));
                assert_eq!(outer.validate_installation(ast.source_id, &input.module_id, &handoff, None, None, Some(&set), &set_arena), Ok(()));
                assert!(SourceAtomicFormulaProducer::build(outer_input.clone(), &flat, &symbols, &handoff, None, None, Some(&set), &set_arena).is_err());
                for mutation in 0..12 {
                    let mut candidate = outer_input.clone();
                    match mutation {
                        0 => candidate.formulas[0].context = context,
                        1 => candidate.edges[1].target = SourceAtomicTermTarget::SetTerm(SourceSetTermId::new(comp_count)),
                        2 => candidate.edges[1].target = SourceAtomicTermTarget::Primary(operands[0]),
                        3 => candidate.edges[1].role = SourceAtomicEdgeRole::BuiltinLeftOperand,
                        4 => candidate.edges.swap(0, 1),
                        5 => { candidate.edges.pop(); }
                        6 => candidate.requests[0].edge = Some(SourceAtomicEdgeId::new(0)),
                        7 => candidate.requests[0].kind = SourceAtomicRequestKind::PredicateCandidateSignature,
                        8 => candidate.requests.clear(),
                        9 => candidate.source_id = other_source_id(),
                        10 => candidate.formulas[0].site = atomic_input.formulas[0].site.clone(),
                        _ => candidate.edges[1].target = SourceAtomicTermTarget::SetTerm(SourceSetTermId::new((set_id.index() + 1) % comp_count)),
                    }
                    if mutation == 11 && comp_count == 1 { continue; }
                    assert!(build_outer(candidate).is_err(), "outer case {case} payload mutation {mutation}");
                }
                // Rebuild only structurally valid environments; these refusals reach
                // the new owner/range/layer/recovery/direct-parent authentication.
                for mutation in 0..6 {
                    let mut contexts = BindingContextTable::new();
                    for (id, original) in bindings.contexts().iter() {
                        let mut draft = BindingContextDraft {
                            owner: original.owner.clone(), parent: original.parent, layer: original.layer,
                            lexical_scope: original.lexical_scope.clone(), bindings: original.bindings.clone(),
                            visible_bindings: original.visible_bindings.clone(), recovery: original.recovery,
                        };
                        if id == context {
                            match mutation {
                                0 => draft.owner = BindingContextOwner::SourceFormula { source_range: comprehension.range() },
                                1 => if let BindingContextOwner::SourceComprehension { source_range } = &mut draft.owner { source_range.start += 1; },
                                2 => draft.layer = BindingContextLayer::Block,
                                3 => draft.recovery = BindingContextRecovery::Recovered,
                                4 => { draft.parent = Some(BindingContextId::new(0)); draft.visible_bindings = draft.bindings.clone(); }
                                _ => { draft.parent = bindings.contexts().iter().find(|(_, row)| row.layer == BindingContextLayer::Declaration).map(|(id, _)| id); draft.visible_bindings = draft.bindings.clone(); }
                            }
                        }
                        assert_eq!(contexts.insert(draft), id);
                    }
                    let environment = BindingEnv::try_new(BindingEnvParts {
                        source_id: bindings.source_id(), module_id: bindings.module_id().clone(), contexts,
                        bindings: bindings.bindings().clone(), diagnostics: bindings.diagnostics().clone(),
                    }).expect("structurally valid outer context mutation");
                    assert_eq!(SourceAtomicFormulaProducer::build(outer_input.clone(), &environment, &symbols, &handoff, None, None, Some(&set), &set_arena), Err(SourceAtomicFormulaError::PrimaryDependencyMismatch), "outer case {case} context mutation {mutation}");
                }
                for wrong_context in [outer_context, BindingContextId::new(1)] {
                    let mut primary = handoff.clone();
                    primary.terms.rows[operands[0].index()].context = wrong_context;
                    assert!(SourceSetTermProducer::build(set_input.clone(), &bindings, &primary, None, None, &set_arena).is_err(), "Set already authenticates condition leaf contexts");
                    assert!(SourceAtomicFormulaProducer::build(outer_input.clone(), &bindings, &symbols, &primary, None, None, Some(&set), &set_arena).is_err());
                    assert!(outer.validate_installation(ast.source_id, &input.module_id, &primary, None, None, Some(&set), &set_arena).is_err());
                }
                assert!(SourceAtomicFormulaProducer::build(outer_input.clone(), &bindings, &symbols, &handoff, None, None, None, &set_arena).is_err());
                for mutation in 0..17 {
                    let mut nodes = set_arena.iter().map(|(_, row)| row.clone()).collect::<Vec<_>>();
                    let rhs_id = resolved.resolved_node_for(rhs.id()).unwrap().index();
                    let lhs_id = resolved.resolved_node_for(lhs.id()).unwrap().index();
                    let token_id = resolved.resolved_node_for(token.id()).unwrap().index();
                    let formula_id = outer_input.formulas[0].site.node().index();
                    match mutation {
                        0 => nodes[rhs_id].children.clear(),
                        1 => nodes[rhs_id].children.push(input.terms[left_primary.index()].site.node()),
                        2 => nodes[formula_id].children[2] = input.terms[left_primary.index()].site.node(),
                        3 => nodes[rhs_id].children[0] = input.terms[operands[0].index()].site.node(),
                        4 => nodes[rhs_id].recovery = NodeRecoveryState::Recovered,
                        5 => { let mut range = rhs.range(); range.start += 1; nodes[rhs_id].anchor = SourceAnchor::Range(range); }
                        6 => nodes[token_id].kind = r#"Token(SurfaceToken { kind: Identifier, text: "in" })"#.into(),
                        7 => nodes[token_id].children.push(input.terms[left_primary.index()].site.node()),
                        8 => nodes[token_id].recovery = NodeRecoveryState::Recovered,
                        9 => nodes[token_id].anchor = SourceAnchor::Range(lhs.range()),
                        10 => nodes[set_input.terms[set_id.index()].site.node().index()].recovery = NodeRecoveryState::Recovered,
                        11 => nodes[lhs_id].recovery = NodeRecoveryState::Recovered,
                        12 => nodes[rhs_id].kind = "ParenthesizedTerm".into(),
                        13 => { nodes[formula_id].children.remove(0); }
                        14 => nodes[formula_id].children.swap(0, 2),
                        15 => nodes[token_id].kind = r#"Token(SurfaceToken { kind: ReservedWord, text: "not" })"#.into(),
                        _ => nodes[formula_id].children.push(input.terms[left_primary.index()].site.node()),
                    };
                    let candidate_arena = TypedArena::try_new(set_arena.root(), nodes).unwrap();
                    assert!(SourceAtomicFormulaProducer::build(outer_input.clone(), &bindings, &symbols, &handoff, None, None, Some(&set), &candidate_arena).is_err(), "outer case {case} physical mutation {mutation}");
                    assert!(outer.validate_installation(ast.source_id, &input.module_id, &handoff, None, None, Some(&set), &candidate_arena).is_err(), "outer case {case} physical replay mutation {mutation}");
                    physical_mutations.push(candidate_arena);
                }
                for kind in [SourceAtomicFormulaKind::Equality, SourceAtomicFormulaKind::Inequality, SourceAtomicFormulaKind::PredicateApplication, SourceAtomicFormulaKind::TypeAssertion] {
                    let mut candidate = outer_input.clone();
                    candidate.formulas[0].kind = kind;
                    let mut nodes = set_arena.iter().map(|(_, row)| row.clone()).collect::<Vec<_>>();
                    nodes[candidate.formulas[0].site.node().index()].kind = match kind {
                        SourceAtomicFormulaKind::Equality => "source.formula.atomic.equality",
                        SourceAtomicFormulaKind::Inequality => "source.formula.atomic.inequality",
                        SourceAtomicFormulaKind::PredicateApplication => "source.formula.atomic.predicate",
                        _ => "source.formula.atomic.type-assertion",
                    }.into();
                    let candidate_arena = TypedArena::try_new(set_arena.root(), nodes).unwrap();
                    assert!(SourceAtomicFormulaProducer::build(candidate, &bindings, &symbols, &handoff, None, None, Some(&set), &candidate_arena).is_err());
                }
                let mut overlap = outer_input.clone();
                let mut nested = atomic_input.clone();
                nested.formulas[0].source_ordinal = 1;
                for row in &mut nested.wrappers { row.formula = SourceAtomicFormulaId::new(1); }
                for row in &mut nested.predicate_segments { row.formula = SourceAtomicFormulaId::new(1); row.left_edge = SourceAtomicEdgeId::new(row.left_edge.index() + 2); row.right_edge = SourceAtomicEdgeId::new(row.right_edge.index() + 2); }
                for row in &mut nested.predicate_heads { row.formula = SourceAtomicFormulaId::new(1); }
                for edge in &mut nested.edges { edge.formula = SourceAtomicFormulaId::new(1); }
                for row in &mut nested.requests { row.formula = SourceAtomicFormulaId::new(1); }
                overlap.formulas.extend(nested.formulas);
                overlap.wrappers.extend(nested.wrappers);
                overlap.predicate_segments = nested.predicate_segments;
                overlap.predicate_heads = nested.predicate_heads;
                overlap.candidates = nested.candidates;
                overlap.edges.extend(nested.edges);
                overlap.requests.extend(nested.requests);
                // EN13.4.2/EN14.2: transport the actual predicate guard without
                // claiming applicability, sethood, formation or truth.
                let paired = build_outer(overlap.clone());
                if negative {
                    assert!(SourceAtomicFormulaProducer::build(atomic_input.clone(), &bindings, &symbols, &handoff, None, None, None, &set_arena).is_ok(), "genuine negative lower Atomic succeeds before paired ownership");
                    assert!(paired.is_ok(), "genuine negative paired Atomic owner should authenticate actual condition children and polarity sites: {paired:?}");
                }
                let combined = paired.expect("paired original whole-chain predicate condition after genuine Frontend/Resolver/Primary/Set/isolated Atomic success");
                if !grouped_corruptions_checked && formula_wrappers.len() > 1 && segment_count > 1 {
                    grouped_corruptions_checked = true;
                    let reject_wrapped_pair = |candidate, arena: &TypedArena| SourceAtomicFormulaProducer::build(
                        candidate, &bindings, &symbols, &handoff, None, None, Some(&set), arena,
                    );
                    for mutation in 0..9 {
                        let mut candidate = overlap.clone();
                        let group = &mut candidate.wrappers;
                        match mutation {
                            0 => group.clear(),
                            1 => {
                                let mut extra = group[0].clone();
                                extra.ordinal = group.len();
                                group.push(extra);
                            }
                            2 => { let duplicate = group[0].clone(); group.push(duplicate); }
                            3 if group.len() > 1 => group.swap(0, 1),
                            3 => group[0].ordinal += 1,
                            4 => group[0].spelling.replace_range(..1, "["),
                            5 => group[0].source_range.start += 1,
                            6 => group[0].context = BindingContextId::new(0),
                            7 => group[0].site = node(usize::MAX),
                            _ => group[0].recovery = SourceAtomicFormulaRecovery::Degraded,
                        }
                        assert!(reject_wrapped_pair(candidate, &set_arena).is_err(), "paired grouped wrapper row mutation {case}/{mutation}");
                    }
                    let wrapper_node = overlap.wrappers[0].site.node();
                    let condition_node = set.conditions().get(SourceSetConditionId::new(set_id.index())).unwrap().condition_site().node();
                    for mutation in 0..10 {
                        let mut nodes = set_arena.iter().map(|(_, row)| row.clone()).collect::<Vec<_>>();
                        match mutation {
                            0 => nodes[condition_node.index()].children.clear(),
                            1 => nodes[condition_node.index()].children[0] = atomic_input.formulas[0].site.node(),
                            2 => nodes[condition_node.index()].children.push(atomic_input.formulas[0].site.node()),
                            3 => nodes[wrapper_node.index()].children.clear(),
                            4 => nodes[wrapper_node.index()].children.push(atomic_input.predicate_heads[0].site.node()),
                            5 => nodes[wrapper_node.index()].children[1] = atomic_input.predicate_heads[0].site.node(),
                            6 => nodes[wrapper_node.index()].children.swap(0, 2),
                            7 => nodes[wrapper_node.index()].kind = "ParenthesizedFormula".into(),
                            8 => nodes[wrapper_node.index()].recovery = NodeRecoveryState::Recovered,
                            _ => nodes[wrapper_node.index()].anchor = SourceAnchor::Range(formula.range()),
                        }
                        let candidate_arena = TypedArena::try_new(set_arena.root(), nodes).unwrap();
                        assert!(reject_wrapped_pair(overlap.clone(), &candidate_arena).is_err(), "paired grouped bridge mutation {case}/{mutation}");
                    }
                }
                assert_eq!((combined.source_id(), combined.module_id(), combined.primary_term_fingerprint(), combined.set_term_fingerprint()), (ast.source_id, &input.module_id, handoff.debug_text().as_str(), Some(set.debug_text().as_str())));
                assert!(combined.application_fingerprint().is_none() && combined.structure_fingerprint().is_none());
                assert_eq!((combined.formulas().len(), combined.wrappers().len(), combined.predicate_segments().len(), combined.predicate_heads().len(), combined.candidates().len(), combined.edges().len(), combined.requests().len()), (2, formula_wrappers.len(), segment_count, segment_count, segment_count, segment_count + 3, segment_count + 1));
                assert!(combined.type_sites().is_empty() && combined.attributes().is_empty());
                assert!(membership.range().start < comprehension.range().start && comprehension.range().start < formula.range().start);
                for ((id, row), expected) in combined.formulas().iter().zip(&overlap.formulas) {
                    assert_eq!((row.site(), row.source_range(), row.source_ordinal(), row.context(), row.recovery(), row.spelling(), row.kind()), (&expected.site, expected.source_range, id.index(), expected.context, expected.recovery, expected.spelling.as_str(), expected.kind));
                }
                let (id, row) = combined.predicate_segments().iter().next().unwrap();
                assert_eq!((id, row.formula(), row.ordinal(), row.site(), row.source_range(), row.context(), row.recovery(), row.spelling()), (SourcePredicateSegmentId::new(0), SourceAtomicFormulaId::new(1), 0, &atomic_input.predicate_segments[0].site, segment.range(), context, SourceAtomicFormulaRecovery::Normal, atomic_input.predicate_segments[0].spelling.as_str()));
                assert_eq!((row.head(), row.polarity(), row.left_edge(), row.right_edge()), (SourcePredicateHeadId::new(0), &atomic_input.predicate_segments[0].polarity, SourceAtomicEdgeId::new(2), SourceAtomicEdgeId::new(3)));
                let (id, row) = combined.predicate_heads().iter().next().unwrap();
                assert_eq!((id, row.formula(), row.site(), row.source_range(), row.context(), row.recovery(), row.spelling(), row.left_arity(), row.right_arity()), (SourcePredicateHeadId::new(0), SourceAtomicFormulaId::new(1), &atomic_input.predicate_heads[0].site, head.range(), context, SourceAtomicFormulaRecovery::Normal, head_text, 1, 1));
                let (id, row) = combined.candidates().iter().next().unwrap();
                assert_eq!((id, row.head(), row.ordinal(), row.symbol(), row.contribution()), (SourcePredicateCandidateId::new(0), SourcePredicateHeadId::new(0), 0, predicate.symbol(), predicate.contribution()));
                for ((_, row), expected) in combined.edges().iter().zip(&overlap.edges) {
                    assert_eq!((row.formula(), row.ordinal(), row.role(), row.target()), (expected.formula, expected.ordinal, expected.role, expected.target));
                }
                for ((_, row), expected) in combined.requests().iter().zip(&overlap.requests) {
                    assert_eq!((row.formula(), row.ordinal(), row.kind(), row.edge(), row.candidate(), row.type_site(), row.attribute()), (expected.formula, expected.ordinal, expected.kind, expected.edge, expected.candidate, expected.type_site, expected.attribute));
                }
                assert_eq!(build_outer(overlap.clone()), Ok(combined.clone()));
                assert_eq!(combined.validate_installation(ast.source_id, &input.module_id, &handoff, None, None, Some(&set), &set_arena), Ok(()));
                let mut reindexed = overlap.clone();
                for (local, row) in reindexed.formulas.iter_mut().enumerate() { row.source_ordinal = 2 * pair + local; }
                let segment_base = aggregate_input.predicate_segments.len();
                let edge_base = aggregate_input.edges.len();
                for row in &mut reindexed.predicate_segments {
                    row.formula = SourceAtomicFormulaId::new(2 * pair + 1);
                    row.head = SourcePredicateHeadId::new(segment_base + row.head.index());
                    row.left_edge = SourceAtomicEdgeId::new(edge_base + row.left_edge.index());
                    row.right_edge = SourceAtomicEdgeId::new(edge_base + row.right_edge.index());
                }
                for row in &mut reindexed.wrappers { row.formula = SourceAtomicFormulaId::new(2 * pair + 1); }
                for row in &mut reindexed.predicate_heads { row.formula = SourceAtomicFormulaId::new(2 * pair + 1); }
                for row in &mut reindexed.candidates { row.head = SourcePredicateHeadId::new(segment_base + row.head.index()); }
                for row in &mut reindexed.edges { row.formula = SourceAtomicFormulaId::new(2 * pair + row.formula.index()); }
                for row in &mut reindexed.requests {
                    row.formula = SourceAtomicFormulaId::new(2 * pair + row.formula.index());
                    row.edge = row.edge.map(|id| SourceAtomicEdgeId::new(edge_base + id.index()));
                    row.candidate = row.candidate.map(|id| SourcePredicateCandidateId::new(segment_base + id.index()));
                }
                aggregate_input.formulas.extend(reindexed.formulas);
                aggregate_input.wrappers.extend(reindexed.wrappers);
                aggregate_input.predicate_segments.extend(reindexed.predicate_segments);
                aggregate_input.predicate_heads.extend(reindexed.predicate_heads);
                aggregate_input.candidates.extend(reindexed.candidates);
                aggregate_input.edges.extend(reindexed.edges);
                aggregate_input.requests.extend(reindexed.requests);
                assert!(SourceAtomicFormulaProducer::build(atomic_input.clone(), &bindings, &symbols, &handoff, None, None, Some(&set), &set_arena).is_err(), "isolated predicate plus overlapping Set has no paired Membership");
                assert!(SourceAtomicFormulaProducer::build(overlap.clone(), &flat, &symbols, &handoff, None, None, Some(&set), &set_arena).is_err());
                assert!(combined.validate_installation(ast.source_id, &input.module_id, &handoff, None, None, None, &set_arena).is_err());
                for mutation in 0..23 {
                    let mut candidate = overlap.clone();
                    match mutation {
                        0 => candidate.predicate_segments.clear(),
                        1 => candidate.predicate_segments.push(candidate.predicate_segments[0].clone()),
                        2 => candidate.predicate_heads[0].left_arity = 2,
                        3 => candidate.predicate_segments[0].head = SourcePredicateHeadId::new(segment_count),
                        4 => candidate.candidates[0].symbol = SymbolId::new(module("foreign"), LocalSymbolId::new("foreign"), FullyQualifiedName::new("foreign::predicate")),
                        5 => candidate.requests[1].candidate = None,
                        6 => candidate.formulas[1].context = outer_context,
                        7 => candidate.formulas.swap(0, 1),
                        8 => candidate.edges[1].role = SourceAtomicEdgeRole::BuiltinLeftOperand,
                        9 => candidate.edges[1].target = SourceAtomicTermTarget::SetTerm(SourceSetTermId::new(comp_count)),
                        10 => candidate.edges[2].target = candidate.edges[3].target,
                        11 => candidate.edges.swap(2, 3),
                        12 => candidate.requests[0].edge = Some(SourceAtomicEdgeId::new(2)),
                        13 => { candidate.requests.pop(); }
                        14 => candidate.source_id = other_source_id(),
                        15 => candidate.formulas[1].recovery = SourceAtomicFormulaRecovery::Degraded,
                        16 => candidate.predicate_segments[0].polarity = SourcePredicateSegmentPolarityInput::Negative {
                            verb_site: candidate.predicate_heads[0].site.clone(), verb_range: head.range(), verb_spelling: "does".into(), verb_recovery: SourceAtomicFormulaRecovery::Normal,
                            not_site: candidate.predicate_heads[0].site.clone(), not_range: head.range(), not_spelling: "not".into(), not_recovery: SourceAtomicFormulaRecovery::Normal,
                        },
                        17 => { let mut segment = candidate.predicate_segments[0].clone(); segment.ordinal = 1; candidate.predicate_segments.push(segment); }
                        18 => candidate.wrappers.push(SourceAtomicWrapperInput { formula: SourceAtomicFormulaId::new(1), ordinal: 0, site: set.conditions().iter().find(|(_, row)| row.term() == set_id).unwrap().1.condition_site().clone(), source_range: formula.range(), context, recovery: SourceAtomicFormulaRecovery::Normal, spelling: format!("( {} )", atomic_input.formulas[0].spelling) }),
                        20 => candidate.candidates.last_mut().unwrap().symbol = SymbolId::new(module("foreign"), LocalSymbolId::new("foreign"), FullyQualifiedName::new("foreign::predicate")),
                        21 => { candidate.candidates.pop(); }
                        22 => candidate.requests.last_mut().unwrap().candidate = Some(SourcePredicateCandidateId::new(segment_count)),
                        _ => { candidate.edges[1].target = SourceAtomicTermTarget::SetTerm(SourceSetTermId::new((set_id.index() + 1) % comp_count)); if comp_count == 1 { continue; } }
                    }
                    assert!(build_outer(candidate).is_err(), "paired case {case} payload mutation {mutation}");
                    assert_eq!(build_outer(overlap.clone()), Ok(combined.clone()));
                }
                // Exercise the explicit unwrapped-root profile independently of
                // the historical differing-context RHS bridge.
                let mut same_context_primary = handoff.clone();
                same_context_primary.terms.rows[left_primary.index()].context = context;
                let mut wrapped_input = set_input.clone();
                let mut wrapper_range = comprehension.range();
                wrapper_range.start -= 1;
                wrapper_range.end += 1;
                let wrapper_site = node(resolved.resolved_node_for(rhs.id()).unwrap().index());
                wrapped_input.wrappers.push(SourceSetWrapperInput { term: set_id, ordinal: 0, site: wrapper_site.clone(), source_range: wrapper_range, context, recovery: SourceSetTermRecovery::Normal, spelling: format!("( {} )", set_input.terms[set_id.index()].spelling) });
                let mut nodes = set_arena.iter().map(|(_, row)| row.clone()).collect::<Vec<_>>();
                nodes[wrapper_site.node().index()].kind = "source.term.set.parenthesized".into();
                nodes[wrapper_site.node().index()].anchor = SourceAnchor::Range(wrapper_range);
                let mut same_context_outer = outer_input.clone();
                same_context_outer.formulas[0].context = context;
                same_context_outer.formulas[0].source_range.end += 1;
                same_context_outer.formulas[0].spelling = format!("{} in ( {} )", input.terms[left_primary.index()].spelling, set_input.terms[set_id.index()].spelling);
                nodes[same_context_outer.formulas[0].site.node().index()].anchor = SourceAnchor::Range(same_context_outer.formulas[0].source_range);
                let wrapped_arena = TypedArena::try_new(set_arena.root(), nodes).unwrap();
                let wrapped = SourceSetTermProducer::build(wrapped_input, &bindings, &same_context_primary, None, None, &wrapped_arena).expect("valid wrapped Set metadata control");
                assert!(SourceAtomicFormulaProducer::build(same_context_outer.clone(), &bindings, &symbols, &same_context_primary, None, None, Some(&wrapped), &wrapped_arena).is_ok(), "historical equal-context outer route remains valid");
                assert!(SourceAtomicFormulaProducer::build(atomic_input.clone(), &bindings, &symbols, &same_context_primary, None, None, None, &wrapped_arena).is_ok());
                let mut wrapped_overlap = overlap.clone();
                wrapped_overlap.formulas[0] = same_context_outer.formulas[0].clone();
                assert!(SourceAtomicFormulaProducer::build(wrapped_overlap, &bindings, &symbols, &same_context_primary, None, None, Some(&wrapped), &wrapped_arena).is_err(), "paired predicate rejects owning Set wrapper even with equal contexts");
                if case == 19 {
                    // Synthetic metadata is a refusal-only counterexample, not
                    // a new genuine-source profile: put a child Comprehension
                    // in the mapper while preserving the real predicate chain.
                    assert_eq!(comp_count, 1);
                    let SourceSetTarget::Primary(mapper) = set_input.edges[0].target else { panic!("actual mapper Primary"); };
                    let mapper_row = handoff.terms().get(mapper).unwrap();
                    let child_range = range(ast.source_id, comprehension.range().start + 1, set_input.generators[0].source_range.start);
                    let generator_range = range(ast.source_id, mapper_row.source_range().end + 1, mapper_row.source_range().end + 2);
                    let type_range = range(ast.source_id, generator_range.end + 1, generator_range.end + 4);
                    assert!(child_range.start < mapper_row.source_range().start && mapper_row.source_range().end < generator_range.start && type_range.end < child_range.end);
                    let mut nodes = set_arena.iter().map(|(_, row)| row.clone()).collect::<Vec<_>>();
                    let first = nodes.len();
                    for (key, source_range) in [("source.term.set.comprehension", child_range), ("source.term.set.comprehension-generator", generator_range), ("source.term.set.target-type", type_range), ("source.term.set.target-type-head", type_range)] {
                        nodes.push(TypedNode::new(key, SourceAnchor::Range(source_range)));
                    }
                    let nested_arena = TypedArena::try_new(set_arena.root(), nodes).unwrap();
                    let mut nested_set = set_input.clone();
                    let child_spelling = format!("{{ {} where q is set }}", mapper_row.spelling());
                    nested_set.terms.push(SourceSetTermInput { site: node(first), source_range: child_range, source_ordinal: 1, context, recovery: SourceSetTermRecovery::Normal, spelling: child_spelling.clone(), kind: SourceSetTermKind::Comprehension });
                    nested_set.generators[0].type_site = SourceSetTypeSiteId::new(1);
                    nested_set.generators.push(SourceSetGeneratorInput { term: SourceSetTermId::new(1), ordinal: 0, site: node(first + 1), source_range: generator_range, spelling: "q".into(), context, recovery: SourceSetTermRecovery::Normal, type_site: SourceSetTypeSiteId::new(0) });
                    // Type rows follow source order independently of Set preorder.
                    nested_set.type_sites.insert(0, SourceSetTypeSiteInput { owner: SourceSetTypeOwner::Generator(SourceSetGeneratorId::new(1)), site: node(first + 2), source_range: type_range, spelling: "set".into(), head_site: node(first + 3), head_range: type_range, head_spelling: "set".into(), context, recovery: SourceSetTermRecovery::Normal, head: SourceSetTypeHead::BuiltinSet });
                    nested_set.edges[0].target = SourceSetTarget::SetTerm(SourceSetTermId::new(1));
                    nested_set.edges.push(SourceSetEdgeInput { term: SourceSetTermId::new(1), ordinal: 0, role: SourceSetEdgeRole::ComprehensionMapper, target: SourceSetTarget::Primary(mapper) });
                    nested_set.requests[0].type_site = Some(SourceSetTypeSiteId::new(1));
                    nested_set.requests.extend([
                        SourceSetRequestInput { term: SourceSetTermId::new(1), ordinal: 0, kind: SourceSetRequestKind::GeneratorSethood, generator: Some(SourceSetGeneratorId::new(1)), type_site: Some(SourceSetTypeSiteId::new(0)) },
                        SourceSetRequestInput { term: SourceSetTermId::new(1), ordinal: 1, kind: SourceSetRequestKind::ResultType, generator: None, type_site: None },
                    ]);
                    nested_set.terms[0].spelling = format!("{{ {child_spelling} where {} is set : {} }}", nested_set.generators[0].spelling, nested_set.conditions[0].spelling);
                    let mut nested_outer = outer_input.clone();
                    nested_outer.formulas[0].spelling = format!("{} in {}", input.terms[left_primary.index()].spelling, nested_set.terms[0].spelling);
                    let nested_set = SourceSetTermProducer::build(nested_set, &bindings, &handoff, None, None, &nested_arena).expect("lower-valid synthetic mapper Comprehension");
                    assert_eq!(nested_set.primary_term_fingerprint(), handoff.debug_text());
                    assert!(SourceAtomicFormulaProducer::build(atomic_input.clone(), &bindings, &symbols, &handoff, None, None, None, &nested_arena).is_ok(), "isolated real predicate remains valid");
                    assert!(SourceAtomicFormulaProducer::build(nested_outer.clone(), &bindings, &symbols, &handoff, None, None, Some(&nested_set), &nested_arena).is_ok(), "outer-only original-context RHS bridge remains valid");
                    let mut nested_overlap = overlap.clone();
                    nested_overlap.formulas[0] = nested_outer.formulas[0].clone();
                    assert!(SourceAtomicFormulaProducer::build(nested_overlap, &bindings, &symbols, &handoff, None, None, Some(&nested_set), &nested_arena).is_err(), "paired predicate rejects contained Comprehension row");
                    assert_eq!(build_outer(overlap.clone()), Ok(combined.clone()));
                }
                for (ordinal, written_id) in formula.children().iter().enumerate() {
                    let written = ast.node_view(*written_id).unwrap();
                    let expected = &atomic_input.predicate_segments[ordinal];
                    let row = combined.predicate_segments().get(SourcePredicateSegmentId::new(ordinal)).unwrap();
                    assert_eq!((row.formula(), row.ordinal(), row.site(), row.source_range(), row.context(), row.recovery(), row.spelling(), row.head(), row.polarity(), row.left_edge(), row.right_edge()), (SourceAtomicFormulaId::new(1), ordinal, &expected.site, written.range(), context, expected.recovery, expected.spelling.as_str(), expected.head, &expected.polarity, SourceAtomicEdgeId::new(ordinal + 2), SourceAtomicEdgeId::new(ordinal + 3)));
                    let head = combined.predicate_heads().get(expected.head).unwrap();
                    let expected_head = &atomic_input.predicate_heads[ordinal];
                    assert_eq!((head.site(), head.source_range(), head.context(), head.recovery(), head.spelling(), head.left_arity(), head.right_arity()), (&expected_head.site, expected_head.source_range, context, expected_head.recovery, pattern_text[1].as_str(), 1, 1));
                    let candidate = combined.candidates().get(SourcePredicateCandidateId::new(ordinal)).unwrap();
                    assert_eq!((candidate.head(), candidate.ordinal(), candidate.symbol(), candidate.contribution()), (expected.head, 0, predicate.symbol(), predicate.contribution()));
                    if segment_count == 1 { continue; }
                    let segment_node = expected.site.node();
                    let head_node = expected_head.site.node();
                    let operand_node = node(resolved.resolved_node_for(*written.children().last().unwrap()).unwrap().index()).node();
                    for mutation in 0..13 {
                        let mut nodes = set_arena.iter().map(|(_, row)| row.clone()).collect::<Vec<_>>();
                        let atom_node = atomic_input.formulas[0].site.node();
                        match mutation {
                            0 => { nodes[atom_node.index()].children.remove(ordinal); }
                            1 => nodes[atom_node.index()].children.push(segment_node),
                            2 => nodes[atom_node.index()].children.swap(ordinal, (ordinal + 1) % segment_count),
                            3 => nodes[atom_node.index()].children[ordinal] = head_node,
                            4 => nodes[segment_node.index()].children.clear(),
                            5 => nodes[segment_node.index()].children.push(head_node),
                            6 => { nodes[segment_node.index()].children.pop(); }
                            7 => nodes[segment_node.index()].children[written.children().len() - 2] = atomic_input.predicate_heads[(ordinal + 1) % segment_count].site.node(),
                            8 => nodes[operand_node.index()].children.clear(),
                            9 => nodes[operand_node.index()].children[0] = handoff.terms().get(operands[0]).unwrap().site().node(),
                            10 => nodes[segment_node.index()].recovery = NodeRecoveryState::Recovered,
                            11 => nodes[head_node.index()].recovery = NodeRecoveryState::Recovered,
                            _ => nodes[operand_node.index()].recovery = NodeRecoveryState::Recovered,
                        }
                        let candidate_arena = TypedArena::try_new(set_arena.root(), nodes).unwrap();
                        assert!(build_outer(overlap.clone()).is_ok());
                        assert!(SourceAtomicFormulaProducer::build(overlap.clone(), &bindings, &symbols, &handoff, None, None, Some(&set), &candidate_arena).is_err(), "chain child ownership case {case} segment {ordinal} mutation {mutation}");
                        assert!(combined.validate_installation(ast.source_id, &input.module_id, &handoff, None, None, Some(&set), &candidate_arena).is_err(), "chain child replay case {case} segment {ordinal} mutation {mutation}");
                        if mutation < 4 { typed_replay_mutations.push(candidate_arena.clone()); }
                        physical_mutations.push(candidate_arena);
                    }
                    if ordinal > 0 {
                        let mut wrong_boundary = overlap.clone();
                        wrong_boundary.predicate_segments[ordinal].left_edge = SourceAtomicEdgeId::new(2);
                        assert!(build_outer(wrong_boundary).is_err(), "each continuation inherits exactly the preceding right edge");
                    }
                    if let SourcePredicateSegmentPolarityInput::Negative { verb_site, not_site, .. } = &expected.polarity {
                        let offset = usize::from(ordinal == 0);
                        for mutation in 0..7 {
                            let mut nodes = set_arena.iter().map(|(_, row)| row.clone()).collect::<Vec<_>>();
                            match mutation {
                                0 => { nodes[segment_node.index()].children.remove(offset); }
                                1 => nodes[segment_node.index()].children.insert(offset, verb_site.node()),
                                2 => nodes[segment_node.index()].children.swap(offset, offset + 1),
                                3 => nodes[segment_node.index()].children[offset] = head_node,
                                4 => nodes[verb_site.node().index()].recovery = NodeRecoveryState::Recovered,
                                5 => nodes[segment_node.index()].children[offset + 1] = verb_site.node(),
                                _ => nodes[not_site.node().index()].recovery = NodeRecoveryState::Recovered,
                            }
                            let candidate_arena = TypedArena::try_new(set_arena.root(), nodes).unwrap();
                            assert!(SourceAtomicFormulaProducer::build(overlap.clone(), &bindings, &symbols, &handoff, None, None, Some(&set), &candidate_arena).is_err(), "chain polarity ownership case {case} segment {ordinal} mutation {mutation}");
                            assert!(combined.validate_installation(ast.source_id, &input.module_id, &handoff, None, None, Some(&set), &candidate_arena).is_err());
                            physical_mutations.push(candidate_arena);
                        }
                    }
                }
                let condition_node = set.conditions().iter().find(|(_, row)| row.term() == set_id).unwrap().1.condition_site().node();
                let atom_node = atomic_input.formulas[0].site.node();
                let segment_node = atomic_input.predicate_segments[0].site.node();
                let head_node = atomic_input.predicate_heads[0].site.node();
                let [left_node, right_node] = [left, right].map(|id| resolved.resolved_node_for(id).unwrap());
                if negative {
                    let [_, verb_id, not_id, _, _] = segment.children() else { unreachable!() };
                    let verb_node = node(resolved.resolved_node_for(*verb_id).unwrap().index()).node();
                    let not_node = node(resolved.resolved_node_for(*not_id).unwrap().index()).node();
                    for mutation in 0..8 {
                        let mut nodes = set_arena.iter().map(|(_, row)| row.clone()).collect::<Vec<_>>();
                        match mutation {
                            0 => { nodes[segment_node.index()].children.remove(1); }
                            1 => { nodes[segment_node.index()].children.remove(2); }
                            2 => nodes[segment_node.index()].children.insert(2, verb_node),
                            3 => nodes[segment_node.index()].children.swap(1, 2),
                            4 => nodes[segment_node.index()].children[1] = node(resolved.resolved_node_for(excluded[0].id()).unwrap().index()).node(),
                            5 => nodes[not_node.index()].recovery = NodeRecoveryState::Recovered,
                            6 => nodes[condition_node.index()].children[0] = segment_node,
                            _ => nodes[condition_node.index()].children.push(head_node),
                        }
                        let candidate_arena = TypedArena::try_new(set_arena.root(), nodes).unwrap();
                        let lower = SourceAtomicFormulaProducer::build(atomic_input.clone(), &bindings, &symbols, &handoff, None, None, None, &candidate_arena);
                        if mutation != 5 {
                            assert!(lower.is_ok(), "negative physical control retains valid lower Atomic metadata: {mutation}");
                        } else {
                            assert!(lower.is_err(), "recovered token node fails the existing Atomic recovery validator");
                        }
                        if mutation >= 6 {
                            assert_eq!(SourceSetTermProducer::build(set_input.clone(), &bindings, &handoff, None, None, &candidate_arena), Ok(set.clone()));
                            typed_replay_mutations.push(candidate_arena.clone());
                        }
                        assert!(SourceAtomicFormulaProducer::build(overlap.clone(), &bindings, &symbols, &handoff, None, None, Some(&set), &candidate_arena).is_err(), "negative paired physical mutation {mutation}");
                        assert!(combined.validate_installation(ast.source_id, &input.module_id, &handoff, None, None, Some(&set), &candidate_arena).is_err(), "negative paired installation replay mutation {mutation}");
                        physical_mutations.push(candidate_arena);
                    }
                    // The recorded token sites are valid TypedArena metadata, but
                    // reusing unrelated source nodes must not authenticate the AST children.
                    let SourcePredicateSegmentPolarityInput::Negative {
                        verb_range, verb_spelling, not_range, not_spelling, ..
                    } = &atomic_input.predicate_segments[0].polarity else { unreachable!() };
                    let unrelated_tokens = ast.node_views().filter(|view| {
                        declaration.range().start <= view.range().start
                            && view.range().end <= declaration.range().end
                            && matches!(view.kind(), SurfaceNodeKind::Token(_))
                    }).collect::<Vec<_>>();
                    assert!(unrelated_tokens.len() >= 2, "the local definition has unrelated real source tokens");
                    let unrelated_views = [unrelated_tokens[0], unrelated_tokens[1]];
                    let unrelated_sites = unrelated_views.map(|view| node(resolved.resolved_node_for(view.id()).unwrap().index()));
                    let mut nodes = set_arena.iter().map(|(_, row)| row.clone()).collect::<Vec<_>>();
                    nodes[unrelated_sites[0].node().index()].kind = "source.formula.atomic.predicate-negation-verb".into();
                    nodes[unrelated_sites[0].node().index()].anchor = SourceAnchor::Range(*verb_range);
                    nodes[unrelated_sites[1].node().index()].kind = "source.formula.atomic.predicate-negation-not".into();
                    nodes[unrelated_sites[1].node().index()].anchor = SourceAnchor::Range(*not_range);
                    let unrelated_arena = TypedArena::try_new(set_arena.root(), nodes).unwrap();
                    let mut unrelated_input = atomic_input.clone();
                    unrelated_input.predicate_segments[0].polarity = SourcePredicateSegmentPolarityInput::Negative {
                        verb_site: unrelated_sites[0].clone(), verb_range: *verb_range, verb_spelling: verb_spelling.clone(), verb_recovery: SourceAtomicFormulaRecovery::Normal,
                        not_site: unrelated_sites[1].clone(), not_range: *not_range, not_spelling: not_spelling.clone(), not_recovery: SourceAtomicFormulaRecovery::Normal,
                    };
                    assert!(SourceAtomicFormulaProducer::build(unrelated_input.clone(), &bindings, &symbols, &handoff, None, None, None, &unrelated_arena).is_ok(), "unrelated sites carry otherwise valid negative token metadata");
                    let mut unrelated_overlap = overlap.clone();
                    unrelated_overlap.predicate_segments[0].polarity = unrelated_input.predicate_segments[0].polarity.clone();
                    assert!(SourceAtomicFormulaProducer::build(unrelated_overlap, &bindings, &symbols, &handoff, None, None, Some(&set), &unrelated_arena).is_err(), "paired Atomic authenticates actual source token children, not unrelated token reuse");
                    let mut installation_nodes = unrelated_arena.iter().map(|(_, row)| row.clone()).collect::<Vec<_>>();
                    installation_nodes[segment_node.index()].children[1] = unrelated_sites[0].node();
                    installation_nodes[segment_node.index()].children[2] = unrelated_sites[1].node();
                    let installation_arena = TypedArena::try_new(set_arena.root(), installation_nodes).unwrap();
                    assert!(combined.validate_installation(ast.source_id, &input.module_id, &handoff, None, None, Some(&set), &installation_arena).is_err(), "installation reauthenticates physical negative-token ownership");
                    physical_mutations.push(installation_arena);
                }
                for mutation in 0..20 {
                    let mut nodes = set_arena.iter().map(|(_, row)| row.clone()).collect::<Vec<_>>();
                    match mutation {
                        0 => nodes[condition_node.index()].children.clear(),
                        1 => nodes[condition_node.index()].children.push(head_node),
                        2 => nodes[condition_node.index()].children[0] = segment_node,
                        3 => nodes[atom_node.index()].children.clear(),
                        4 => nodes[atom_node.index()].children.push(head_node),
                        5 => nodes[atom_node.index()].children[0] = head_node,
                        6 => nodes[segment_node.index()].children.clear(),
                        7 => nodes[segment_node.index()].children.push(head_node),
                        8 => nodes[segment_node.index()].children.swap(0, if negative { 3 } else { 2 }),
                        9 => nodes[segment_node.index()].children[if negative { 3 } else { 1 }] = handoff.terms().get(operands[0]).unwrap().site().node(),
                        10 => nodes[segment_node.index()].children[0] = handoff.terms().get(operands[0]).unwrap().site().node(),
                        11 => nodes[left_node.index()].children.clear(),
                        12 => nodes[right_node.index()].children.push(handoff.terms().get(operands[0]).unwrap().site().node()),
                        13 => nodes[left_node.index()].children[0] = handoff.terms().get(operands[1]).unwrap().site().node(),
                        14 => nodes[left_node.index()].anchor = SourceAnchor::Range(segment.range()),
                        15 => nodes[right_node.index()].recovery = NodeRecoveryState::Recovered,
                        16 => nodes[left_node.index()].kind = "ParenthesizedTerm".into(),
                        17 => nodes[segment_node.index()].children[if negative { 4 } else { 2 }] = node(left_node.index()).node(),
                        18 => nodes[head_node.index()].recovery = NodeRecoveryState::Recovered,
                        _ => nodes[condition_node.index()].recovery = NodeRecoveryState::Recovered,
                    }
                    let candidate_arena = TypedArena::try_new(set_arena.root(), nodes).unwrap();
                    if mutation < 18 {
                        assert_eq!(SourceSetTermProducer::build(set_input.clone(), &bindings, &handoff, None, None, &candidate_arena), Ok(set.clone()), "new guard Set lower-success case {case} mutation {mutation}");
                        let isolated = SourceAtomicFormulaProducer::build(atomic_input.clone(), &bindings, &symbols, &handoff, None, None, None, &candidate_arena);
                        assert!(isolated.is_ok(), "new guard isolated Atomic lower-success case {case} mutation {mutation}: {isolated:?}");
                    } else if mutation == 18 {
                        assert!(SourceAtomicFormulaProducer::build(atomic_input.clone(), &bindings, &symbols, &handoff, None, None, None, &candidate_arena).is_err(), "earlier Atomic metadata refusal");
                    } else {
                        assert!(SourceSetTermProducer::build(set_input.clone(), &bindings, &handoff, None, None, &candidate_arena).is_err(), "earlier Set metadata refusal");
                    }
                    assert!(SourceAtomicFormulaProducer::build(overlap.clone(), &bindings, &symbols, &handoff, None, None, Some(&set), &candidate_arena).is_err(), "paired physical case {case} mutation {mutation}");
                    assert!(combined.validate_installation(ast.source_id, &input.module_id, &handoff, None, None, Some(&set), &candidate_arena).is_err(), "paired physical replay case {case} mutation {mutation}");
                    physical_mutations.push(candidate_arena);
                    assert_eq!(build_outer(overlap.clone()), Ok(combined.clone()));
                    assert_eq!(combined.validate_installation(ast.source_id, &input.module_id, &handoff, None, None, Some(&set), &set_arena), Ok(()));
                }
                assert_eq!(build_outer(outer_input.clone()), Ok(outer.clone()));
                assert_eq!(outer.validate_installation(ast.source_id, &input.module_id, &handoff, None, None, Some(&set), &set_arena), Ok(()));
                assert_eq!((outer_input, outer, set.clone()), outer_pristine);
                let atomic_pristine = (atomic_input.clone(), atomic_arena.clone());
                let build = |candidate| SourceAtomicFormulaProducer::build(candidate, &bindings, &symbols, &handoff, None, None, None, &atomic_arena);
                let atomic = build(atomic_input.clone()).unwrap_or_else(|error| panic!("genuine predicate condition case {case}: {error:?}"));
                let build = |candidate| SourceAtomicFormulaProducer::build(candidate, &bindings, &symbols, &handoff, None, None, None, &set_arena);
                assert_eq!(build(atomic_input.clone()), Ok(atomic.clone()), "same Atomic product on the final Set metadata arena without a Set dependency");
                assert_eq!(atomic.source_id(), ast.source_id);
                assert_eq!(atomic.module_id(), &input.module_id);
                assert_eq!(atomic.primary_term_fingerprint(), handoff.debug_text());
                assert!(atomic.application_fingerprint().is_none() && atomic.structure_fingerprint().is_none() && atomic.set_term_fingerprint().is_none());
                assert_eq!((atomic.formulas().len(), atomic.wrappers().len(), atomic.predicate_segments().len(), atomic.predicate_heads().len(), atomic.candidates().len(), atomic.edges().len(), atomic.requests().len()), (1, formula_wrappers.len(), segment_count, segment_count, segment_count, segment_count + 1, segment_count));
                assert!(atomic.type_sites().is_empty() && atomic.attributes().is_empty());
                let row = atomic.formulas().get(SourceAtomicFormulaId::new(0)).unwrap();
                assert_eq!((row.site(), row.source_range(), row.source_ordinal(), row.context(), row.recovery(), row.spelling(), row.kind()), (&atomic_input.formulas[0].site, formula.range(), 0, context, SourceAtomicFormulaRecovery::Normal, atomic_input.formulas[0].spelling.as_str(), SourceAtomicFormulaKind::PredicateApplication));
                let row = atomic.predicate_segments().get(SourcePredicateSegmentId::new(0)).unwrap();
                assert_eq!((row.formula(), row.ordinal(), row.site(), row.source_range(), row.context(), row.recovery(), row.spelling()), (SourceAtomicFormulaId::new(0), 0, &atomic_input.predicate_segments[0].site, segment.range(), context, SourceAtomicFormulaRecovery::Normal, atomic_input.predicate_segments[0].spelling.as_str()));
                assert_eq!((row.head(), row.polarity(), row.left_edge(), row.right_edge()), (SourcePredicateHeadId::new(0), &atomic_input.predicate_segments[0].polarity, SourceAtomicEdgeId::new(0), SourceAtomicEdgeId::new(1)));
                if negative {
                    for mutation in 0..9 {
                        let mut candidate = atomic_input.clone();
                        if mutation == 0 {
                            candidate.predicate_segments[0].polarity = SourcePredicateSegmentPolarityInput::Positive;
                        } else {
                            let SourcePredicateSegmentPolarityInput::Negative {
                                verb_site, verb_range, verb_spelling, verb_recovery,
                                not_site, not_range, not_spelling, not_recovery,
                            } = &mut candidate.predicate_segments[0].polarity else { unreachable!() };
                            match mutation {
                                1 => *verb_site = candidate.predicate_heads[0].site.clone(),
                                2 => verb_range.start += 1,
                                3 => *verb_spelling = "is".into(),
                                4 => *verb_recovery = SourceAtomicFormulaRecovery::Degraded,
                                5 => *not_site = verb_site.clone(),
                                6 => not_range.start += 1,
                                7 => *not_spelling = "non".into(),
                                _ => *not_recovery = SourceAtomicFormulaRecovery::Degraded,
                            }
                        }
                        assert!(build(candidate).is_err(), "genuine negative polarity metadata mutation {case}/{mutation}");
                        assert_eq!(build(atomic_input.clone()), Ok(atomic.clone()));
                    }
                }
                let row = atomic.predicate_heads().get(SourcePredicateHeadId::new(0)).unwrap();
                assert_eq!((row.formula(), row.site(), row.source_range(), row.context(), row.recovery(), row.spelling(), row.left_arity(), row.right_arity()), (SourceAtomicFormulaId::new(0), &atomic_input.predicate_heads[0].site, head.range(), context, SourceAtomicFormulaRecovery::Normal, head_text, 1, 1));
                let row = atomic.candidates().get(SourcePredicateCandidateId::new(0)).unwrap();
                assert_eq!((row.head(), row.ordinal(), row.symbol(), row.contribution()), (SourcePredicateHeadId::new(0), 0, predicate.symbol(), predicate.contribution()));
                for ((id, row), expected) in atomic.edges().iter().zip(&atomic_input.edges) {
                    assert_eq!((row.formula(), row.ordinal(), row.role(), row.target()), (expected.formula, id.index(), expected.role, expected.target));
                }
                let row = atomic.requests().get(SourceAtomicRequestId::new(0)).unwrap();
                assert_eq!((row.formula(), row.ordinal(), row.kind(), row.edge(), row.candidate(), row.type_site(), row.attribute()), (SourceAtomicFormulaId::new(0), 0, SourceAtomicRequestKind::PredicateCandidateSignature, None, Some(SourcePredicateCandidateId::new(0)), None, None));
                // Allocate a missing ID only for refusal; the original collector stays authoritative.
                let mut invalid_contributions = symbols.contributions().clone();
                let missing_contribution = invalid_contributions.insert(input.module_id.clone(), contribution.kind().clone(), contribution.anchor().clone());
                assert!(symbols.contributions().get(missing_contribution).is_none());
                for mutation in 0..10 {
                    let mut candidate = atomic_input.clone();
                    match mutation {
                        0 => candidate.candidates[0].symbol = SymbolId::new(module("foreign"), LocalSymbolId::new("foreign"), FullyQualifiedName::new("foreign::predicate")),
                        1 => candidate.candidates[0].contribution = missing_contribution,
                        2 => { candidate.formulas[0].context = BindingContextId::new(0); candidate.predicate_segments[0].context = BindingContextId::new(0); candidate.predicate_heads[0].context = BindingContextId::new(0); }
                        3 => candidate.edges[0].target = candidate.edges[1].target,
                        4 => candidate.edges.swap(0, 1),
                        5 => candidate.source_id = other_source_id(),
                        6 => candidate.formulas[0].site = input.terms[operands[0].index()].site.clone(),
                        7 => candidate.predicate_heads[0].source_range = formula.range(),
                        8 => candidate.predicate_segments[0].head = SourcePredicateHeadId::new(segment_count),
                        _ => candidate.requests.clear(),
                    }
                    assert!(build(candidate).is_err(), "genuine atom case {case} mutation {mutation}");
                    assert_eq!(build(atomic_input.clone()), Ok(atomic.clone()));
                }
                assert_eq!((atomic_input, atomic_arena), atomic_pristine);
                assert_eq!(handoff, primary_before);
                assert_eq!(consume(input.clone(), &bindings), Ok(primary_before.clone()));
            }
            let application_input = SourceFunctorApplicationHandoffInput {
                source_id: ast.source_id, module_id: input.module_id.clone(), applications: Vec::new(),
                wrappers: Vec::new(), candidates: Vec::new(), arguments: Vec::new(), type_requests: Vec::new(),
            };
            let application = SourceFunctorApplicationProducer::build(application_input.clone(), &symbols, &bindings, &handoff, &set_arena).expect("genuine empty Application");
            assert_eq!(SourceFunctorApplicationProducer::build(application_input.clone(), &symbols, &bindings, &handoff, &set_arena), Ok(application.clone()));
            assert_eq!(application.primary_term_fingerprint(), handoff.debug_text());
            assert!(application.applications().is_empty() && application.wrappers().is_empty() && application.candidates().is_empty() && application.arguments().is_empty() && application.type_requests().is_empty());
            let build_aggregate = |candidate| SourceAtomicFormulaProducer::build(candidate, &bindings, &symbols, &handoff, None, None, Some(&set), &set_arena);
            let aggregate = build_aggregate(aggregate_input.clone()).expect("all original Membership/predicate pairs in source order");
            let total_segments = comprehensions.iter().map(|comprehension| {
                let condition = comprehension.child_views().find(|view| view.kind() == &SurfaceNodeKind::FormulaExpression).unwrap();
                let mut actual = condition;
                loop {
                    match actual.kind() {
                        SurfaceNodeKind::FormulaExpression => actual = ast.node_view(actual.children()[0]).unwrap(),
                        SurfaceNodeKind::ParenthesizedFormula => actual = ast.node_view(actual.children()[1]).unwrap(),
                        _ => break,
                    }
                }
                actual.children().len()
            }).sum::<usize>();
            assert_eq!((aggregate.formulas().len(), aggregate.wrappers().len(), aggregate.predicate_segments().len(), aggregate.predicate_heads().len(), aggregate.candidates().len(), aggregate.edges().len(), aggregate.requests().len()), (2 * comp_count, aggregate_input.wrappers.len(), total_segments, total_segments, total_segments, total_segments + 3 * comp_count, total_segments + comp_count));
            assert!(aggregate.type_sites().is_empty() && aggregate.attributes().is_empty());
            assert_eq!((aggregate.primary_term_fingerprint(), aggregate.set_term_fingerprint()), (handoff.debug_text().as_str(), Some(set.debug_text().as_str())));
            assert!(aggregate.application_fingerprint().is_none() && aggregate.structure_fingerprint().is_none());
            let mut condition_input = SourceConditionFormulaCompositionHandoffInput { source_id: ast.source_id, module_id: input.module_id.clone(), edges: Vec::new() };
            for (pair, comprehension) in comprehensions.iter().enumerate() {
                let condition = comprehension.child_views().find(|view| view.kind() == &SurfaceNodeKind::FormulaExpression).unwrap();
                let mut atom = condition;
                loop {
                    match atom.kind() {
                        SurfaceNodeKind::FormulaExpression => atom = ast.node_view(atom.children()[0]).unwrap(),
                        SurfaceNodeKind::ParenthesizedFormula => atom = ast.node_view(atom.children()[1]).unwrap(),
                        _ => break,
                    }
                }
                let rhs = ast.node_view(*parents.get(&comprehension.id()).unwrap()).unwrap();
                let membership = ast.node_view(*parents.get(&rhs.id()).unwrap()).unwrap();
                assert_eq!(rhs.children(), &[comprehension.id()]);
                assert_eq!(membership.children()[2], rhs.id());
                let condition_id = SourceSetConditionId::new(pair);
                let actual_condition = set.conditions().get(condition_id).unwrap();
                assert_eq!((actual_condition.term(), actual_condition.condition_site(), actual_condition.ordinal()), (SourceSetTermId::new(pair), &node(resolved.resolved_node_for(condition.id()).unwrap().index()), 0));
                for (local, actual, kind) in [(0, membership, SourceAtomicFormulaKind::Membership), (1, atom, SourceAtomicFormulaKind::PredicateApplication)] {
                    let id = SourceAtomicFormulaId::new(2 * pair + local);
                    let row = aggregate.formulas().get(id).unwrap();
                    assert_eq!((row.source_ordinal(), row.site(), row.source_range(), row.context(), row.kind()), (id.index(), &node(resolved.resolved_node_for(actual.id()).unwrap().index()), actual.range(), expected_context(scopes.get(&actual.id()).unwrap()), kind));
                    if id.index() > 0 { assert!(aggregate.formulas().get(SourceAtomicFormulaId::new(id.index() - 1)).unwrap().source_range().start < row.source_range().start); }
                }
                let predicate_id = SourceAtomicFormulaId::new(2 * pair + 1);
                let segment_base = comprehensions[..pair].iter().map(|comprehension| {
                    let condition = comprehension.child_views().find(|view| view.kind() == &SurfaceNodeKind::FormulaExpression).unwrap();
                    let mut actual = condition;
                    loop {
                        match actual.kind() {
                            SurfaceNodeKind::FormulaExpression => actual = ast.node_view(actual.children()[0]).unwrap(),
                            SurfaceNodeKind::ParenthesizedFormula => actual = ast.node_view(actual.children()[1]).unwrap(),
                            _ => break,
                        }
                    }
                    actual.children().len()
                }).sum::<usize>();
                let edge_base = segment_base + 3 * pair;
                let request_base = segment_base + pair;
                for (local, segment_id) in atom.children().iter().enumerate() {
                    let written = ast.node_view(*segment_id).unwrap();
                    let segment = aggregate.predicate_segments().get(SourcePredicateSegmentId::new(segment_base + local)).unwrap();
                    assert_eq!((segment.formula(), segment.ordinal(), segment.site(), segment.source_range(), segment.head(), segment.left_edge(), segment.right_edge()), (predicate_id, local, &node(resolved.resolved_node_for(written.id()).unwrap().index()), written.range(), SourcePredicateHeadId::new(segment_base + local), SourceAtomicEdgeId::new(edge_base + 2 + local), SourceAtomicEdgeId::new(edge_base + 3 + local)));
                    assert_eq!(aggregate.predicate_heads().get(SourcePredicateHeadId::new(segment_base + local)).unwrap().formula(), predicate_id);
                    assert_eq!(aggregate.candidates().get(SourcePredicateCandidateId::new(segment_base + local)).unwrap().head(), SourcePredicateHeadId::new(segment_base + local));
                }
                for local in 0..atom.children().len() + 3 {
                    let edge = aggregate.edges().get(SourceAtomicEdgeId::new(edge_base + local)).unwrap();
                    let expected = &aggregate_input.edges[edge_base + local];
                    assert_eq!((edge.formula(), edge.ordinal(), edge.target()), (expected.formula, expected.ordinal, expected.target));
                }
                for local in 0..atom.children().len() + 1 {
                    let request = aggregate.requests().get(SourceAtomicRequestId::new(request_base + local)).unwrap();
                    assert_eq!((request.formula(), request.ordinal(), request.edge(), request.candidate()), (if local == 0 { SourceAtomicFormulaId::new(2 * pair) } else { predicate_id }, local.saturating_sub(1), (local == 0).then_some(SourceAtomicEdgeId::new(edge_base + 1)), (local > 0).then_some(SourcePredicateCandidateId::new(segment_base + local.saturating_sub(1)))));
                }
                condition_input.edges.push(SourceConditionFormulaEdgeInput { condition: condition_id, ordinal: 0, formula: predicate_id });
            }
            // C2 must fail only after every complete lower product replays successfully.
            assert_eq!(handoff.validate_installation(ast.source_id, &input.module_id, &set_arena), Ok(()));
            assert_eq!(application.validate_installation(ast.source_id, &input.module_id, &handoff), Ok(()));
            assert_eq!(set.validate_installation(ast.source_id, &input.module_id, &handoff, Some(&application), None, &set_arena), Ok(()));
            assert_eq!(aggregate.validate_installation(ast.source_id, &input.module_id, &handoff, Some(&application), None, Some(&set), &set_arena), Ok(()));
            assert_eq!(build_aggregate(aggregate_input.clone()), Ok(aggregate.clone()));
            let composition_pristine = (application_input.clone(), aggregate_input.clone(), condition_input.clone(), application.clone(), aggregate.clone(), set.clone(), handoff.clone(), set_arena.clone(), bindings.clone());
            let build_condition = |candidate, arena: &TypedArena| SourceConditionFormulaCompositionProducer::build(candidate, &handoff, &application, &set, &aggregate, arena);
            let composition = build_condition(condition_input.clone(), &set_arena).unwrap_or_else(|error| panic!("genuine complete scoped predicate C2 case {case}, all four lower replays passed: {error:?}"));
            let c3_matrix_case = text.contains("holds A in { z where z is set : z likes A likes z };")
                || text.contains("holds A in { z where z is set : z likes A likes z likes A };")
                || text.contains("holds A in { z where z is set : z likes A likes z } & A in { w where w is set : w likes A likes w };")
                || text.contains("holds A in { z where z is set : z does not likes A likes z };")
                || text.contains("holds A in { z where z is set : z likes A do not likes z };")
                || text.contains("holds A in { z where z is set : z likes A does not likes z likes A };")
                || text.contains("proof let A be set; thus A in { z where z is set : z do not likes A likes z does not likes A }; end;")
                || text.contains("for parcel being\tset holds parcel in { point where point is set : point likes parcel likes point } & parcel in { needle where needle is set : needle does not likes parcel };")
                || comprehensions.iter().any(|comprehension| {
                    let condition = comprehension.child_views().find(|view| view.kind() == &SurfaceNodeKind::FormulaExpression).unwrap();
                    let mut actual = condition;
                    loop {
                        match actual.kind() {
                            SurfaceNodeKind::FormulaExpression => actual = ast.node_view(actual.children()[0]).unwrap(),
                            SurfaceNodeKind::ParenthesizedFormula => actual = ast.node_view(actual.children()[1]).unwrap(),
                            _ => break,
                        }
                    }
                    actual.kind() == &SurfaceNodeKind::PredicateApplication && actual.children().len() > 1
                });
            if c3_matrix_case {
                let mut chain_input = SourcePredicateChainCompositionHandoffInput {
                    source_id: ast.source_id,
                    module_id: input.module_id.clone(),
                    conjunctions: Vec::new(),
                    negations: Vec::new(),
                };
                let mut segment_base = 0;
                for (pair, comprehension) in comprehensions.iter().enumerate() {
                    let condition = comprehension.child_views().find(|view| view.kind() == &SurfaceNodeKind::FormulaExpression).unwrap();
                    let mut atom = condition;
                    loop {
                        match atom.kind() {
                            SurfaceNodeKind::FormulaExpression => atom = ast.node_view(atom.children()[0]).unwrap(),
                            SurfaceNodeKind::ParenthesizedFormula => atom = ast.node_view(atom.children()[1]).unwrap(),
                            _ => break,
                        }
                    }
                    let segments = atom.children().iter().map(|id| ast.node_view(*id).unwrap()).collect::<Vec<_>>();
                    let formula = SourceAtomicFormulaId::new(2 * pair + 1);
                    for ordinal in 0..segments.len().saturating_sub(1) {
                        let edge_base = segment_base + 3 * pair;
                        chain_input.conjunctions.push(SourcePredicateChainConjunctionInput {
                            formula,
                            ordinal,
                            left_segment: SourcePredicateSegmentId::new(segment_base + ordinal),
                            right_segment: SourcePredicateSegmentId::new(segment_base + ordinal + 1),
                            boundary: SourceAtomicEdgeId::new(edge_base + 3 + ordinal),
                        });
                    }
                    if segments.len() > 1 {
                        let mut negative_ordinal = 0;
                        for (ordinal, segment) in segments.iter().enumerate() {
                            let negative = match segment.children() {
                                [_, _, _] | [_, _] => false,
                                [_, verb, not, _, _] => {
                                    let verb = ast.node_view(*verb).unwrap();
                                    let not = ast.node_view(*not).unwrap();
                                    assert!(matches!(&text[verb.range().start..verb.range().end], "does" | "do"));
                                    assert_eq!(&text[not.range().start..not.range().end], "not");
                                    true
                                }
                                [verb, not, _, _] => {
                                    let verb = ast.node_view(*verb).unwrap();
                                    let not = ast.node_view(*not).unwrap();
                                    assert!(matches!(&text[verb.range().start..verb.range().end], "does" | "do"));
                                    assert_eq!(&text[not.range().start..not.range().end], "not");
                                    true
                                }
                                children => panic!("unexpected written predicate segment shape: {children:?}"),
                            };
                            if negative {
                                chain_input.negations.push(SourcePredicateChainNegationInput {
                                    formula,
                                    ordinal: negative_ordinal,
                                    segment: SourcePredicateSegmentId::new(segment_base + ordinal),
                                });
                                negative_ordinal += 1;
                            }
                        }
                    }
                    segment_base += segments.len();
                }
                let chain = SourcePredicateChainCompositionProducer::build_conditioned(
                    chain_input.clone(),
                    &handoff,
                    &application,
                    &set,
                    &aggregate,
                    &composition,
                    &set_arena,
                ).unwrap_or_else(|error| panic!("genuine conditioned C3 case {case}: {error:?}"));
                assert_eq!(SourcePredicateChainCompositionProducer::build_conditioned(
                    chain_input.clone(), &handoff, &application, &set, &aggregate, &composition, &set_arena,
                ), Ok(chain.clone()), "deterministic conditioned C3 build, case {case}");
                assert_eq!(chain.conjunctions().len(), chain_input.conjunctions.len());
                for (expected_id, ((id, actual), expected)) in chain.conjunctions().iter().zip(&chain_input.conjunctions).enumerate() {
                    assert_eq!((id.index(), actual.formula(), actual.ordinal(), actual.left_segment(), actual.right_segment(), actual.boundary()),
                        (expected_id, expected.formula, expected.ordinal, expected.left_segment, expected.right_segment, expected.boundary), "AST-derived dense C3 conjunction row, case {case}");
                }
                assert_eq!(chain.negations().len(), chain_input.negations.len());
                for (expected_id, ((id, actual), expected)) in chain.negations().iter().zip(&chain_input.negations).enumerate() {
                    assert_eq!((id.index(), actual.formula(), actual.ordinal(), actual.segment()),
                        (expected_id, expected.formula, expected.ordinal, expected.segment), "AST-derived dense C3 negation row, case {case}");
                }
                assert_eq!(chain.validate_installation(ast.source_id, &input.module_id, &handoff, &aggregate, None, Some(&composition), &set_arena),
                    Err(SourcePredicateChainCompositionError::DependencyMismatch), "conditioned C3 replay requires both Application and Set, case {case}");

                if let Some((previous_application, previous_set, _)) = &previous_chain_dependencies {
                    for lower in [(previous_application, &set), (&application, previous_set)] {
                        assert_eq!(chain.validate_installation(ast.source_id, &input.module_id, &handoff, &aggregate, Some(lower), Some(&composition), &set_arena),
                            Err(SourcePredicateChainCompositionError::DependencyMismatch), "conditioned C3 replay refuses mismatched lower dependencies, case {case}");
                    }
                }
                let typed = task269gcu_empty_typed_for(ast.source_id, input.module_id.clone(), set_arena.clone())
                    .with_source_term(handoff.clone()).unwrap()
                    .with_source_application(application.clone()).unwrap()
                    .with_source_set_term(set.clone()).unwrap()
                    .with_source_atomic_formula(aggregate.clone()).unwrap()
                    .with_source_condition_formula_composition(composition.clone())
                    .unwrap_or_else(|error| panic!("conditioned C2 co-installation, case {case}: {error:?}"))
                    .with_source_predicate_chain_composition(chain.clone())
                    .unwrap_or_else(|error| panic!("conditioned C3 Typed replay, case {case}: {error:?}"));
                assert_eq!(typed.source_predicate_chain_composition(), Some(&chain));
                assert_eq!((typed.source_term(), typed.source_application(), typed.source_set_term(), typed.source_atomic_formula()),
                    (Some(&handoff), Some(&application), Some(&set), Some(&aggregate)), "conditioned C3 retains lower products, case {case}");
                assert_eq!(typed.source_condition_formula_composition(), Some(&composition), "conditioned C2 is co-installed with C3, case {case}");
                assert!(typed.types().is_empty() && typed.facts().is_empty() && typed.coercions().is_empty() && typed.initial_obligations().is_empty() && typed.diagnostics().is_empty());
                assert_eq!(typed.clone(), typed);
                assert_eq!(typed.clone().debug_text(), typed.debug_text());
                let final_ast = task269gcu_resolved_result_with_inputs(&typed, Vec::new(), Vec::new())
                    .unwrap_or_else(|error| panic!("conditioned C3 final replay, case {case}: {error:?}"));
                assert_eq!(final_ast.source_predicate_chain_composition(), Some(&chain));
                assert_eq!((final_ast.source_term(), final_ast.source_application(), final_ast.source_set_term(), final_ast.source_atomic_formula()),
                    (Some(&handoff), Some(&application), Some(&set), Some(&aggregate)), "final conditioned C3 retains lower products, case {case}");
                assert_eq!(final_ast.source_condition_formula_composition(), Some(&composition));
                assert_eq!(final_ast.clone(), final_ast);
                assert_eq!(final_ast.clone().debug_text(), final_ast.debug_text());
                assert_eq!(task269gcu_resolved_result_with_inputs(&typed, Vec::new(), Vec::new()), Ok(final_ast.clone()));

                let set_first = task269gcu_empty_typed_for(ast.source_id, input.module_id.clone(), set_arena.clone())
                    .with_source_term(handoff.clone()).unwrap()
                    .with_source_set_term(set.clone()).unwrap()
                    .with_source_atomic_formula(aggregate.clone()).unwrap()
                    .with_source_application(application.clone()).unwrap()
                    .with_source_condition_formula_composition(composition.clone()).unwrap()
                    .with_source_predicate_chain_composition(chain.clone()).unwrap();
                assert_eq!(set_first, typed, "both lower orders retain conditioned C2/C3, case {case}");
                assert_eq!(task269gcu_resolved_result_with_inputs(&set_first, Vec::new(), Vec::new()), Ok(final_ast.clone()),
                    "both lower orders replay conditioned C2/C3, case {case}");

                let mut injected_without_c2 = task269gcu_empty_typed_for(ast.source_id, input.module_id.clone(), set_arena.clone())
                    .with_source_term(handoff.clone()).unwrap()
                    .with_source_application(application.clone()).unwrap()
                    .with_source_set_term(set.clone()).unwrap()
                    .with_source_atomic_formula(aggregate.clone()).unwrap();
                injected_without_c2.inject_source_predicate_chain_composition_for_test(chain.clone());
                let injected_without_c2_before = injected_without_c2.clone();
                assert!(task269gcu_resolved_result_with_inputs(&injected_without_c2, Vec::new(), Vec::new()).is_err(),
                    "final replay refuses test-injected conditioned C3 without installed C2, case {case}");
                assert_eq!(injected_without_c2, injected_without_c2_before, "failed final replay leaves injected Typed state unchanged, case {case}");

                let mut injected_without_application = task269gcu_empty_typed_for(ast.source_id, input.module_id.clone(), set_arena.clone())
                    .with_source_term(handoff.clone()).unwrap()
                    .with_source_set_term(set.clone()).unwrap()
                    .with_source_atomic_formula(aggregate.clone()).unwrap();
                injected_without_application.inject_source_condition_formula_composition_for_test(composition.clone());
                injected_without_application.inject_source_predicate_chain_composition_for_test(chain.clone());
                let injected_without_application_before = injected_without_application.clone();
                assert!(task269gcu_resolved_result_with_inputs(&injected_without_application, Vec::new(), Vec::new()).is_err(),
                    "final replay refuses conditioned C3 with missing Application, case {case}");
                assert_eq!(injected_without_application, injected_without_application_before,
                    "failed missing-Application replay leaves Typed state unchanged, case {case}");

                if let Some((_, _, conflicting_composition)) = &previous_chain_dependencies {
                    let mut injected_conflict = typed.clone();
                    injected_conflict.inject_source_condition_formula_composition_for_test(conflicting_composition.clone());
                    let injected_conflict_before = injected_conflict.clone();
                    assert!(task269gcu_resolved_result_with_inputs(&injected_conflict, Vec::new(), Vec::new()).is_err(),
                        "final replay refuses a test-injected conflicting C2/C3 pair, case {case}");
                    assert_eq!(injected_conflict, injected_conflict_before, "failed conflicting final replay leaves Typed state unchanged, case {case}");
                }
                previous_chain_dependencies = Some((application.clone(), set.clone(), composition.clone()));
            }
            assert_eq!((composition.source_id(), composition.module_id()), (ast.source_id, &input.module_id));
            assert_eq!((composition.primary_term_fingerprint(), composition.application_fingerprint(), composition.set_term_fingerprint(), composition.atomic_formula_fingerprint()), (handoff.debug_text().as_str(), application.debug_text().as_str(), set.debug_text().as_str(), aggregate.debug_text().as_str()));
            assert_eq!(composition.edges().len(), comp_count);
            for (id, edge) in composition.edges().iter() { assert_eq!((edge.condition(), edge.ordinal(), edge.formula()), (SourceSetConditionId::new(id.index()), 0, SourceAtomicFormulaId::new(2 * id.index() + 1))); }
            assert_eq!(build_condition(condition_input.clone(), &set_arena), Ok(composition.clone()));
            assert_eq!(composition.validate_installation(ast.source_id, &input.module_id, &handoff, &application, &set, &aggregate, &set_arena), Ok(()));
            for mutation in 0..13 {
                let mut candidate = condition_input.clone();
                let expected = match mutation {
                    0 => { candidate.edges.pop(); SourceConditionFormulaCompositionError::DependencyMismatch }
                    1 => { candidate.edges.clear(); SourceConditionFormulaCompositionError::DependencyMismatch }
                    2 => { candidate.edges.push(candidate.edges[0].clone()); SourceConditionFormulaCompositionError::InvalidAggregate }
                    3 => { candidate.edges[0].condition = SourceSetConditionId::new(comp_count); SourceConditionFormulaCompositionError::InvalidEdge { edge: SourceConditionFormulaEdgeId::new(0) } }
                    4 => { candidate.edges[0].ordinal = 1; SourceConditionFormulaCompositionError::InvalidEdge { edge: SourceConditionFormulaEdgeId::new(0) } }
                    5 => { candidate.edges[0].formula = SourceAtomicFormulaId::new(0); SourceConditionFormulaCompositionError::DependencyMismatch }
                    6 => { candidate.edges[0].formula = SourceAtomicFormulaId::new(2 * comp_count); SourceConditionFormulaCompositionError::DependencyMismatch }
                    7 => { candidate.source_id = other_source_id(); SourceConditionFormulaCompositionError::DependencyMismatch }
                    8 => { candidate.module_id = module("foreign"); SourceConditionFormulaCompositionError::DependencyMismatch }
                    9 if comp_count > 1 => { candidate.edges[1] = candidate.edges[0].clone(); SourceConditionFormulaCompositionError::DependencyMismatch }
                    10 if comp_count > 1 => { candidate.edges.swap(0, 1); SourceConditionFormulaCompositionError::InvalidEdge { edge: SourceConditionFormulaEdgeId::new(0) } }
                    11 if comp_count > 1 => { candidate.edges[0].formula = SourceAtomicFormulaId::new(3); candidate.edges[1].formula = SourceAtomicFormulaId::new(1); SourceConditionFormulaCompositionError::InvalidEdge { edge: SourceConditionFormulaEdgeId::new(0) } }
                    12 if comp_count > 1 => { candidate.edges[0].condition = SourceSetConditionId::new(1); SourceConditionFormulaCompositionError::InvalidEdge { edge: SourceConditionFormulaEdgeId::new(0) } }
                    _ => continue,
                };
                assert_eq!(build_condition(candidate, &set_arena), Err(expected), "C2 case {case} mutation {mutation}");
                assert_eq!((&application_input, &aggregate_input, &condition_input, &application, &aggregate, &set, &handoff, &set_arena, &bindings), (&composition_pristine.0, &composition_pristine.1, &composition_pristine.2, &composition_pristine.3, &composition_pristine.4, &composition_pristine.5, &composition_pristine.6, &composition_pristine.7, &composition_pristine.8));
                assert_eq!(build_condition(condition_input.clone(), &set_arena), Ok(composition.clone()));
                assert_eq!(composition.validate_installation(ast.source_id, &input.module_id, &handoff, &application, &set, &aggregate, &set_arena), Ok(()));
            }
            for candidate_arena in physical_mutations {
                assert_eq!(build_condition(condition_input.clone(), &candidate_arena), Err(SourceConditionFormulaCompositionError::DependencyMismatch), "delegated lower physical replay");
                assert_eq!(composition.validate_installation(ast.source_id, &input.module_id, &handoff, &application, &set, &aggregate, &candidate_arena), Err(SourceConditionFormulaCompositionError::DependencyMismatch));
                assert_eq!((&application_input, &aggregate_input, &condition_input, &application, &aggregate, &set, &handoff, &set_arena, &bindings), (&composition_pristine.0, &composition_pristine.1, &composition_pristine.2, &composition_pristine.3, &composition_pristine.4, &composition_pristine.5, &composition_pristine.6, &composition_pristine.7, &composition_pristine.8));
                assert_eq!(build_condition(condition_input.clone(), &set_arena), Ok(composition.clone()));
                assert_eq!(composition.validate_installation(ast.source_id, &input.module_id, &handoff, &application, &set, &aggregate, &set_arena), Ok(()));
            }
            for candidate_arena in typed_replay_mutations {
                let typed_before_atomic = task269gcu_empty_typed_for(ast.source_id, input.module_id.clone(), candidate_arena)
                    .with_source_term(handoff.clone()).unwrap()
                    .with_source_application(application.clone()).unwrap()
                    .with_source_set_term(set.clone()).unwrap();
                assert!(typed_before_atomic.with_source_atomic_formula(aggregate.clone()).is_err(), "Typed installation replays negative condition ownership");
            }
            let base = task269gcu_empty_typed_for(ast.source_id, input.module_id.clone(), set_arena.clone()).with_source_term(handoff.clone()).unwrap();
            for missing in [base.clone().with_source_application(application.clone()).unwrap(), base.clone().with_source_application(application.clone()).unwrap().with_source_set_term(set.clone()).unwrap(), base.clone().with_source_set_term(set.clone()).unwrap().with_source_atomic_formula(aggregate.clone()).unwrap()] {
                assert!(matches!(missing.with_source_condition_formula_composition(composition.clone()), Err(TypedAstError::InvalidSourceConditionFormulaComposition)));
            }
            let mut installed = None;
            for application_first in [true, false] {
                let typed = if application_first { base.clone().with_source_application(application.clone()).unwrap().with_source_set_term(set.clone()).unwrap().with_source_atomic_formula(aggregate.clone()).unwrap() }
                    else { base.clone().with_source_set_term(set.clone()).unwrap().with_source_atomic_formula(aggregate.clone()).unwrap().with_source_application(application.clone()).unwrap() }
                    .with_source_condition_formula_composition(composition.clone()).expect("genuine C2 Typed installation");
                assert_eq!((typed.source_term(), typed.source_application(), typed.source_set_term(), typed.source_atomic_formula(), typed.source_condition_formula_composition()), (Some(&handoff), Some(&application), Some(&set), Some(&aggregate), Some(&composition)));
                assert!(typed.types().is_empty() && typed.facts().is_empty() && typed.coercions().is_empty() && typed.initial_obligations().is_empty() && typed.diagnostics().is_empty());
                assert!(typed.source_predicate_chain_composition().is_none(), "conditioned chains defer C3");
                assert_eq!(typed.nodes(), &set_arena);
                assert_eq!(typed.clone(), typed);
                assert_eq!(typed.clone().debug_text(), typed.debug_text());
                assert!(matches!(typed.clone().with_source_condition_formula_composition(composition.clone()), Err(TypedAstError::InvalidSourceConditionFormulaComposition)));
                let final_ast = task269gcu_resolved_result_with_inputs(&typed, Vec::new(), Vec::new()).expect("genuine C2 final assembly");
                assert_eq!((final_ast.source_term(), final_ast.source_application(), final_ast.source_set_term(), final_ast.source_atomic_formula(), final_ast.source_condition_formula_composition()), (Some(&handoff), Some(&application), Some(&set), Some(&aggregate), Some(&composition)));
                assert!(final_ast.source_predicate_chain_composition().is_none(), "final conditioned chains defer C3");
                assert_eq!(final_ast.clone(), final_ast);
                assert_eq!(final_ast.clone().debug_text(), final_ast.debug_text());
                assert_eq!(task269gcu_resolved_result_with_inputs(&typed, Vec::new(), Vec::new()), Ok(final_ast.clone()));
                if let Some((previous, previous_final)) = &installed { assert_eq!((&typed, &final_ast), (previous, previous_final)); }
                installed = Some((typed, final_ast));
            }
            assert_eq!((application_input, aggregate_input, condition_input, application, aggregate, set.clone(), handoff.clone(), set_arena.clone(), bindings.clone()), composition_pristine);
            assert_eq!((set_input, set_arena, bindings.clone(), handoff.clone()), set_pristine);
        }
        if case == 8 || case == 21 {
            assert_eq!(input.terms[1].spelling, "z");
            for mutation in 0..7 {
                let mut candidate = input.clone();
                match mutation {
                    0 => candidate.terms[1].site = candidate.terms[2].site.clone(),
                    1 => candidate.terms[1].context = candidate.terms[0].context,
                    2 => candidate.references[1].binding = BindingId::new(0),
                    3 => { candidate.references.remove(1); }
                    4 => candidate.references.insert(1, candidate.references[1].clone()),
                    5 => candidate.references.swap(1, 2),
                    _ => candidate.terms[1].source_range = candidate.terms[2].source_range,
                }
                assert!(consume(candidate, &bindings).is_err(), "generator mapper mutation {mutation}");
            }
            for mutation in 0..2 {
                let mut candidate = bindings.clone();
                let generator = candidate.binding_mut_for_test(BindingId::new(1)).unwrap();
                if mutation == 0 { generator.visible_after_ordinal += 1; }
                else if let BinderIdentity::ResolverLocal { scope, .. } = &mut generator.identity {
                    *scope = LocalTermScope::new(vec![999]);
                }
                assert_eq!(consume(input.clone(), &candidate), Err(SourcePrimaryTermError::InvalidTransaction));
            }
        }
        if case == 0 || case == 21 {
            for mutation in 0..9 {
                let mut candidate = input.clone();
                match mutation {
                    0 => { candidate.terms.pop(); candidate.references.pop(); }
                    1 => candidate.references.push(candidate.references[0].clone()),
                    2 => candidate.references.swap(0, 1),
                    3 => candidate.references[0].binding = BindingId::new(2),
                    4 => candidate.terms[1].context = candidate.terms[0].context,
                    5 => candidate.terms[0].recovery = SourcePrimaryTermRecovery::Degraded,
                    6 => candidate.source_id = other_source_id(),
                    7 => candidate.module_id = module("foreign"),
                    _ => candidate.terms[0].source_ordinal += 1,
                }
                assert!(consume(candidate, &bindings).is_err(), "Primary mutation {mutation}");
            }
            let comp = bindings.contexts().iter().find(|(_, context)| matches!(context.owner, BindingContextOwner::SourceComprehension { .. })).unwrap().0;
            let definition = bindings.contexts().iter().find(|(_, context)| context.layer == BindingContextLayer::Declaration).map(|(id, _)| id);
            for mutation in 0..if definition.is_some() { 11 } else { 7 } {
                let mut contexts = BindingContextTable::new();
                for (id, context) in bindings.contexts().iter() {
                    let mut draft = BindingContextDraft {
                        owner: context.owner.clone(), parent: context.parent, layer: context.layer,
                        lexical_scope: context.lexical_scope.clone(), bindings: context.bindings.clone(),
                        visible_bindings: context.visible_bindings.clone(), recovery: context.recovery,
                    };
                    if id == comp && mutation < 7 {
                        match mutation {
                            0 => if let BindingContextOwner::SourceComprehension { source_range } = &mut draft.owner { source_range.start += 1; },
                            1 => draft.owner = BindingContextOwner::SourceStatement { source_range: ast.node_view(*boundaries[comp.index()].0).unwrap().range() },
                            2 => draft.lexical_scope = Some(LocalTermScope::new(vec![999])),
                            3 => draft.layer = BindingContextLayer::Block,
                            4 => draft.recovery = BindingContextRecovery::Recovered,
                            5 => { draft.parent = Some(BindingContextId::new(0)); draft.visible_bindings = draft.bindings.clone(); }
                            _ => draft.visible_bindings.clear(),
                        }
                    }
                    if Some(id) == definition && mutation >= 7 {
                        match mutation {
                            7 => if let BindingContextOwner::SourceStatement { source_range } = &mut draft.owner { source_range.start += 1; },
                            8 => draft.layer = BindingContextLayer::Block,
                            9 => draft.lexical_scope = Some(LocalTermScope::new(vec![999])),
                            _ => draft.recovery = BindingContextRecovery::Recovered,
                        }
                    }
                    assert_eq!(contexts.insert(draft), id);
                }
                let candidate = BindingEnv::try_new(BindingEnvParts {
                    source_id: bindings.source_id(), module_id: bindings.module_id().clone(),
                    contexts, bindings: bindings.bindings().clone(), diagnostics: bindings.diagnostics().clone(),
                }).expect("structurally valid context mutation before source authentication");
                assert_eq!(consume(input.clone(), &candidate), Err(SourcePrimaryTermError::InvalidTransaction), "context mutation {mutation}");
            }
            for mutation in 0..4 {
                let mut candidate = bindings.clone();
                let entry = candidate.binding_mut_for_test(BindingId::new(1)).unwrap();
                match mutation {
                    0 => { entry.visible_after_ordinal += 1; if let BinderIdentity::ResolverLocal { ordinal, .. } = &mut entry.identity { *ordinal += 1; } }
                    1 => entry.recovery = BindingRecoveryState::Recovered,
                    2 => if let BinderIdentity::ResolverLocal { scope, .. } = &mut entry.identity { *scope = LocalTermScope::new(vec![999]); },
                    _ => entry.type_site = BindingTypeSite::Deferred("foreign".into()),
                }
                let candidate = BindingEnv::try_new(BindingEnvParts {
                    source_id: candidate.source_id(), module_id: candidate.module_id().clone(),
                    contexts: candidate.contexts().clone(), bindings: candidate.bindings().clone(), diagnostics: candidate.diagnostics().clone(),
                }).expect("structurally valid binding mutation before source authentication");
                assert_eq!(consume(input.clone(), &candidate), Err(SourcePrimaryTermError::InvalidTransaction), "binding mutation {mutation}");
            }
            let foreign_module = module("foreign");
            let foreign_source = SurfaceResolvedArena::lower(&ast, &foreign_module).unwrap();
            let foreign_symbols = SymbolEnv::new(foreign_module.clone(), SymbolEnvIndexes::default());
            let foreign_scope = SourceVariableScopeResolver::resolve_proof_occurrences(SourceVariableScopeInput::new(&ast, &foreign_module, &foreign_symbols)).unwrap();
            for (source, symbols, scope) in [(&foreign_source, &symbols, &scope), (&resolved, &foreign_symbols, &scope), (&resolved, &symbols, &foreign_scope)] {
                assert_eq!(SourcePrimaryTermProducer::build_from_scoped_theorem_occurrences(input.clone(), &bindings, &arena, source, symbols, scope), Err(SourcePrimaryTermError::InvalidTransaction));
            }
        }
        // Existing generic installation replays node metadata; full source/context/
        // reference authentication belongs to the scoped producer above.
        let typed = task269gcu_empty_typed_for(ast.source_id, input.module_id.clone(), arena.clone()).with_source_term(handoff.clone()).unwrap();
        assert_eq!(typed.source_term(), Some(&handoff));
        assert!(typed.types().is_empty() && typed.facts().is_empty() && typed.initial_obligations().is_empty());
        assert!(typed.source_set_term().is_none() && typed.source_atomic_formula().is_none() && typed.source_formula_composition().is_none());
        let final_ast = task269gcu_resolved_result_with_inputs(&typed, Vec::new(), Vec::new()).expect("generic Primary metadata replay with no inferred phases");
        assert_eq!(final_ast.source_term(), Some(&handoff));
        assert_eq!(task269gcu_resolved_result_with_inputs(&typed, Vec::new(), Vec::new()), Ok(final_ast));
        let mut nodes = arena.iter().map(|(_, node)| node.clone()).collect::<Vec<_>>();
        nodes[input.terms[0].site.node().index()].recovery = NodeRecoveryState::Recovered;
        let recovered_arena = TypedArena::try_new(arena.root(), nodes).unwrap();
        assert!(matches!(task269gcu_empty_typed_for(ast.source_id, input.module_id.clone(), recovered_arena).with_source_term(handoff), Err(TypedAstError::InvalidSourceTerm)));
        assert_eq!((&input, &bindings, &arena, &scope), (&pristine.0, &pristine.1, &pristine.2, &pristine.3));
        assert_eq!((resolved, symbols, scopes, parents, ast.snapshot_text(), loaded.source_text), provenance);
        }
    }

    #[test]
    fn occurrence_builder_authenticates_genuine_parenthesized_variable_chains() {
        use mizar_frontend::{
            lexing::{ParserLexingPlan, TokenizeRequest, tokenize},
            parsing::{MizarParserSeam, ParseRequest, ParserInputs, ParserSeam},
            preprocess::preprocess,
            source::{
                FrontendSourceLoader, SourceUnitLoader, SourceUnitRequest, register_source_unit,
            },
            span_bridge::SpanBridge,
        };
        use mizar_resolve::{
            declarations::DeclarationShellCollector, env::NamespacePath,
            resolved_ast::SurfaceResolvedArena, symbols::SignatureProjectionExtractor,
        };
        use mizar_session::{DiskSourceLoader, Edition, SourceInput, SourceOriginInput};

        let text = "reserve x,y for object; theorem Group: ((x)) = y;";
        let directory =
            std::env::temp_dir().join(format!("primary-group-owner-{}", std::process::id()));
        std::fs::create_dir_all(directory.join("src")).unwrap();
        let path = directory.join("src/main.miz");
        std::fs::write(&path, text).unwrap();
        let loaded = FrontendSourceLoader::new(DiskSourceLoader::new(&directory))
            .load_source_unit(
                SourceUnitRequest {
                    snapshot: BuildSnapshotId::from_published_schema_str(&format!(
                        "mizar-session-build-snapshot-v1:{}",
                        "38".repeat(32),
                    ))
                    .unwrap(),
                    input: SourceInput {
                        package_id: PackageId::new("pkg"),
                        module_path: ModulePath::new("groups"),
                        normalized_path: mizar_session::normalize_path(&directory, &path).unwrap(),
                        edition: Edition::new("2026"),
                        origin: SourceOriginInput::Disk { path },
                    },
                },
                &InMemorySessionIdAllocator::new(),
            )
            .unwrap();
        std::fs::remove_dir_all(directory).unwrap();
        let mut bridge = SpanBridge::new();
        register_source_unit(&mut bridge, &loaded).unwrap();
        let preprocessed = preprocess(&loaded, &mut bridge).unwrap();
        assert!(preprocessed.diagnostics.is_empty());
        assert!(preprocessed.import_stubs.is_empty());
        let lexical = mizar_lexer::build_lexical_environment(&[], &[]).unwrap();
        let tokens = tokenize(
            TokenizeRequest::with_plan(
                &preprocessed,
                &lexical,
                ParserLexingPlan::for_lexical_text(preprocessed.lexical_text.as_str()),
            ),
            &bridge,
        )
        .unwrap();
        assert!(
            tokens.diagnostics().is_empty(),
            "{:?}",
            tokens.diagnostics()
        );
        let parsed = MizarParserSeam.parse(ParseRequest::new(
            &tokens,
            ParserInputs::from_active_environment(loaded.edition, &lexical),
        ));
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let ast = parsed.ast.unwrap();
        let actual_module = module("groups");
        let shells = DeclarationShellCollector::new(&ast, &actual_module).collect();
        let collected =
            SignatureProjectionExtractor::new(&ast, &shells, NamespacePath::new("groups"))
                .collect(&actual_module);
        assert!(collected.diagnostics().is_empty());
        let symbols = collected.into_env();
        let scope = SourceVariableScopeResolver::resolve_occurrences(
            SourceVariableScopeInput::new(&ast, &actual_module, &symbols),
        )
        .unwrap();
        let bindings = SourceVariableSemanticsChecker::occurrence_binding_env(&scope);
        let resolved = SurfaceResolvedArena::lower(&ast, &actual_module).unwrap();
        let nodes = resolved
            .arena()
            .iter()
            .map(|(id, current)| {
                let kind = match current.kind() {
                    SurfaceNodeKind::TermReference => "source.term.variable-reference".to_owned(),
                    SurfaceNodeKind::ParenthesizedTerm => "source.term.parenthesized".to_owned(),
                    kind => format!("{kind:?}"),
                };
                TypedNode::new(kind, current.origin().anchor().clone())
                    .with_resolved_node(id)
                    .with_children(
                        current
                            .children()
                            .iter()
                            .map(|id| TypedNodeId::new(id.index()))
                            .collect(),
                    )
            })
            .collect::<Vec<_>>();
        let arena = TypedArena::try_new(
            Some(TypedNodeId::new(resolved.arena().root().index())),
            nodes,
        )
        .unwrap();
        let mut sites = ast
            .node_views()
            .filter(|view| {
                matches!(
                    view.kind(),
                    SurfaceNodeKind::ParenthesizedTerm | SurfaceNodeKind::TermReference
                )
            })
            .collect::<Vec<_>>();
        sites.sort_by_key(|view| (view.range().start, std::cmp::Reverse(view.range().end)));
        let terms = sites
            .iter()
            .enumerate()
            .map(|(index, view)| SourcePrimaryTermInput {
                site: node(view.id().index()),
                source_range: view.range(),
                source_ordinal: index,
                context: BindingContextId::new(0),
                recovery: SourcePrimaryTermRecovery::Normal,
                spelling: text[view.range().start..view.range().end]
                    .replace('(', " ( ")
                    .replace(')', " ) ")
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" "),
                kind: if matches!(view.kind(), SurfaceNodeKind::ParenthesizedTerm) {
                    SourcePrimaryTermKind::Parenthesized
                } else {
                    SourcePrimaryTermKind::VariableReference
                },
                role: SourcePrimaryTermRole::Value,
                parent: sites[..index]
                    .iter()
                    .rposition(|parent| {
                        parent.range().start < view.range().start
                            && view.range().end < parent.range().end
                    })
                    .map(SourcePrimaryTermId::new),
            })
            .collect::<Vec<_>>();
        let references = scope
            .references()
            .iter()
            .map(|actual| SourcePrimaryTermReferenceInput {
                term: SourcePrimaryTermId::new(
                    sites
                        .iter()
                        .position(|site| site.id() == actual.node())
                        .unwrap(),
                ),
                binding: BindingId::new(actual.binding().index()),
                role: SourcePrimaryTermReferenceRole::Variable,
            })
            .collect();
        let input = SourcePrimaryTermHandoffInput {
            source_id: ast.source_id,
            module_id: actual_module,
            terms,
            references,
            numeric_type_requests: Vec::new(),
        };
        let pristine = (
            input.clone(),
            bindings.clone(),
            arena.clone(),
            scope.clone(),
        );
        let handoff = SourcePrimaryTermProducer::build_from_occurrences(
            input.clone(),
            &bindings,
            &arena,
            &scope,
        )
        .expect("genuine nested variable groups");
        assert_eq!(
            handoff
                .terms()
                .iter()
                .map(|(_, term)| (term.spelling(), term.parent().map(|id| id.index())))
                .collect::<Vec<_>>(),
            [
                ("( ( x ) )", None),
                ("( x )", Some(0)),
                ("x", Some(1)),
                ("y", None)
            ]
        );
        assert_eq!(
            handoff
                .references()
                .iter()
                .map(|(_, reference)| reference.term().index())
                .collect::<Vec<_>>(),
            [2, 3]
        );
        assert!(handoff.numeric_type_requests().is_empty());
        for ((_, reference), actual) in handoff.references().iter().zip(scope.references()) {
            let leaf = handoff.terms().get(reference.term()).unwrap();
            assert_eq!(leaf.site().node().index(), actual.node().index());
            assert_eq!(
                arena
                    .node(leaf.site().node())
                    .unwrap()
                    .resolved_node
                    .unwrap()
                    .index(),
                actual.node().index()
            );
            assert_eq!(leaf.source_range(), actual.range());
            assert_eq!(reference.binding().index(), actual.binding().index());
            assert_eq!(reference.lexical_scope(), Some(actual.scope()));
            assert_eq!(reference.use_ordinal(), actual.ordinal());
        }
        assert_eq!(
            (&input, &bindings, &arena, &scope),
            (&pristine.0, &pristine.1, &pristine.2, &pristine.3)
        );

        let outer = input.terms[0].site.node().index();
        let inner = input.terms[1].site.node().index();
        let leaf = input.terms[2].site.node().index();
        let shell = arena.node(TypedNodeId::new(outer)).unwrap();
        let open = shell.children[0].index();
        let expression = shell.children[1].index();
        let close = shell.children[2].index();
        for mutation in [
            "skip bridge",
            "extra shell child",
            "nonsole bridge",
            "wrong bridge kind",
            "bridge range",
            "bridge recovery",
            "wrong open text",
            "wrong close text",
            "wrong token kind",
            "nonleaf punctuation",
            "open anchor",
            "close anchor",
            "punctuation recovery",
            "child substitution",
            "parent recovery",
            "missing parent",
            "wrong parent",
            "wrong context",
            "wrong spelling",
            "missing reference",
            "extra reference",
            "wrapper reference",
            "rebound reference",
            "foreign source",
            "foreign module",
            "foreign environment",
            "foreign scope",
        ] {
            let mut changed = input.clone();
            let mut nodes = arena
                .iter()
                .map(|(_, node)| node.clone())
                .collect::<Vec<_>>();
            let mut environment = bindings.clone();
            let mut resolver = scope.clone();
            match mutation {
                "skip bridge" => nodes[outer].children[1] = TypedNodeId::new(inner),
                "extra shell child" => nodes[outer].children.push(TypedNodeId::new(open)),
                "nonsole bridge" => nodes[expression].children.push(TypedNodeId::new(leaf)),
                "wrong bridge kind" => nodes[expression].kind = "TermReference".into(),
                "bridge range" => nodes[expression].anchor = nodes[outer].anchor.clone(),
                "bridge recovery" => nodes[expression].recovery = NodeRecoveryState::Recovered,
                "wrong open text" => {
                    nodes[open].kind =
                        r#"Token(SurfaceToken { kind: ReservedSymbol, text: "[" })"#.into()
                }
                "wrong close text" => nodes[close].kind = nodes[open].kind.clone(),
                "wrong token kind" => {
                    nodes[open].kind =
                        r#"Token(SurfaceToken { kind: Identifier, text: "(" })"#.into()
                }
                "nonleaf punctuation" => nodes[open].children = nodes[leaf].children.clone(),
                "open anchor" => nodes[open].anchor = nodes[close].anchor.clone(),
                "close anchor" => nodes[close].anchor = nodes[open].anchor.clone(),
                "punctuation recovery" => nodes[open].recovery = NodeRecoveryState::Recovered,
                "child substitution" => nodes[expression].children = vec![TypedNodeId::new(leaf)],
                "parent recovery" => nodes[outer].recovery = NodeRecoveryState::Recovered,
                "missing parent" => changed.terms[1].parent = None,
                "wrong parent" => changed.terms[2].parent = Some(SourcePrimaryTermId::new(0)),
                "wrong context" => changed.terms[1].context = BindingContextId::new(1),
                "wrong spelling" => changed.terms[0].spelling = "( x )".to_owned(),
                "missing reference" => {
                    changed.references.pop();
                }
                "extra reference" => changed.references.push(changed.references[0].clone()),
                "wrapper reference" => changed.references[0].term = SourcePrimaryTermId::new(0),
                "rebound reference" => {
                    changed.references[0].binding = changed.references[1].binding
                }
                "foreign source" => changed.source_id = other_source_id(),
                "foreign module" => changed.module_id = module("foreign"),
                "foreign environment" => {
                    environment
                        .binding_mut_for_test(BindingId::new(0))
                        .unwrap()
                        .declaration_range
                        .start += 1
                }
                "foreign scope" => {
                    resolver = occurrence_fixture().0;
                }
                _ => unreachable!(),
            }
            let changed_arena = TypedArena::try_new(arena.root(), nodes).expect(mutation);
            let error = SourcePrimaryTermProducer::build_from_occurrences(
                changed,
                &environment,
                &changed_arena,
                &resolver,
            )
            .unwrap_err();
            assert!(
                match mutation {
                    "foreign source"
                    | "foreign module"
                    | "foreign environment"
                    | "foreign scope" => error == SourcePrimaryTermError::InvalidTransaction,
                    "missing reference" | "extra reference" | "wrapper reference"
                    | "rebound reference" =>
                        matches!(error, SourcePrimaryTermError::InvalidReference { .. }),
                    _ => matches!(error, SourcePrimaryTermError::InvalidTerm { .. }),
                },
                "{mutation}: {error:?}"
            );
        }
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
            fixture.dependency.clone(),
            fixture.input.clone(),
            &fixture.arena,
        )
        .expect("Task269GU valid handoff");
        // Generic congruence accepts the counterfeit; the unchanged receipt must reject it.
        let mut shifted = fixture.input.clone();
        let mut nodes = fixture
            .arena
            .iter()
            .map(|(_, node)| node.clone())
            .collect::<Vec<_>>();
        for (index, term) in shifted.terms.iter_mut().enumerate() {
            term.source_range.start += 1;
            term.source_range.end += 1;
            nodes[index + 3].anchor = SourceAnchor::Range(term.source_range);
        }
        let arena = TypedArena::try_new(fixture.arena.root(), nodes).unwrap();
        let generic = SourcePrimaryTermProducer::build_with_profile(
            shifted.clone(),
            fixture.dependency.binding_env(),
            &arena,
            SourcePrimaryTermBindingProfile::ProofLocalGivenUse,
        )
        .expect("coherent primary term has valid generic geometry");
        assert_eq!(
            SourceProofLocalGivenUseTermProducer::build(
                fixture.dependency.clone(),
                shifted,
                &arena
            ),
            Err(SourceProofLocalGivenUseTermError::InvalidSourceTerm)
        );
        let mut counterfeit = handoff.clone();
        counterfeit.source_term = generic;
        counterfeit.source_term_fingerprint = counterfeit.source_term.debug_text();
        let mut typed = task269gu_empty_typed(&fixture, arena).unwrap();
        assert_eq!(
            typed
                .clone()
                .with_source_proof_local_given_use_term(counterfeit.clone()),
            Err(TypedAstError::InvalidSourceProofLocalGivenUseTerm)
        );
        typed.inject_source_proof_local_given_use_term_for_test(counterfeit);
        assert_eq!(
            task269gu_resolved_result(&typed),
            Err(ResolvedTypedAstError::InvalidSourceProofLocalGivenUseTerm)
        );
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
        let lower_fingerprint =
            crate::source_proof_local_declaration::task286_receipt(source, module.clone(), 3);
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
            input: task269gcu_test_input(source, module),
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
            task269sdu_test_input(&fixture),
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
            task269sdu_test_input(&fixture),
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
            task269sdu_test_input(&fixture),
            &fixture.arena,
        )
        .expect("Task269SDU exact handoff");

        let mut wrong_dependency = task269gu_input(
            fixture.source,
            module("task269sdu-wrong"),
            &crate::source_proof_local_declaration::task286_receipt(
                fixture.source,
                fixture.module.clone(),
                4,
            ),
            2,
        );
        wrong_dependency.terms[0].source_range.end += 1;
        assert_eq!(
            SourceProofLocalGivenDescendantUseTermProducer::build(
                fixture.dependency.clone(),
                wrong_dependency,
                &task269sdu_test_arena(fixture.source, true, true),
            ),
            Err(SourceProofLocalGivenDescendantUseTermError::InvalidDependency)
        );

        let mut wrong_input = task269sdu_test_input(&fixture);
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
                task269sdu_test_input(&fixture),
                &task269sdu_test_arena(fixture.source, false, true),
            ),
            Err(SourceProofLocalGivenDescendantUseTermError::InvalidSourceTerm)
        );
        assert_eq!(
            SourceProofLocalGivenDescendantUseTermProducer::build(
                fixture.dependency.clone(),
                task269sdu_test_input(&fixture),
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
            task269sdu_test_input(&fixture),
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
            task269sdu_test_input(&fixture),
            &fixture.arena,
        )
        .expect("Task269SDU exact handoff");
        let input = task269sdu_test_input(&fixture);
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

    fn task269sdu_test_input(fixture: &Task269sduFixture) -> SourcePrimaryTermHandoffInput {
        task269gu_input(
            fixture.source,
            fixture.module.clone(),
            fixture.dependency.dependency().intake(),
            2,
        )
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
