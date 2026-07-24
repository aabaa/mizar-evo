//! Syntax-free transport for source functor-application occurrences.

use crate::{
    binding_env::{BindingContextId, BindingEnv},
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

dense_id!(SourceFunctorApplicationId);
dense_id!(SourceFunctorWrapperId);
dense_id!(SourceFunctorCandidateId);
dense_id!(SourceFunctorArgumentId);
dense_id!(SourceFunctorTypeRequestId);

/// Complete syntax-free input for one source/module application transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFunctorApplicationHandoffInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub applications: Vec<SourceFunctorApplicationInput>,
    pub wrappers: Vec<SourceFunctorWrapperInput>,
    pub candidates: Vec<SourceFunctorCandidateInput>,
    pub arguments: Vec<SourceFunctorArgumentInput>,
    pub type_requests: Vec<SourceFunctorTypeRequestInput>,
}

/// One source functor-application occurrence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFunctorApplicationInput {
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub source_ordinal: usize,
    pub context: BindingContextId,
    pub recovery: SourceFunctorApplicationRecovery,
    pub spelling: String,
    pub kind: SourceFunctorApplicationKind,
    pub form: SourceFunctorApplicationForm,
    pub head_ordinal: usize,
    pub head: SourceFunctorHeadSite,
}

/// One transparent parenthesized application wrapper.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFunctorWrapperInput {
    pub application: SourceFunctorApplicationId,
    pub ordinal: usize,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub context: BindingContextId,
    pub spelling: String,
    pub recovery: SourceFunctorApplicationRecovery,
}

/// One resolver-authenticated functor reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFunctorCandidateInput {
    pub application: SourceFunctorApplicationId,
    pub ordinal: usize,
    pub symbol: SymbolId,
    pub contribution: SourceContributionId,
}

/// One ordered cross-family argument edge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFunctorArgumentInput {
    pub application: SourceFunctorApplicationId,
    pub ordinal: usize,
    pub target: SourceFunctorArgumentTarget,
}

/// One unresolved signature or result-type dependency request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFunctorTypeRequestInput {
    pub application: SourceFunctorApplicationId,
    pub candidate: Option<SourceFunctorCandidateId>,
    pub request_ordinal: usize,
    pub kind: SourceFunctorTypeRequestKind,
}

/// Source application family admitted by Task 253.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceFunctorApplicationKind {
    Symbolic,
    Inline,
}

/// Recovery state retained at the source-application boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceFunctorApplicationRecovery {
    Normal,
    Degraded,
}

/// Source-written punctuation and head position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceFunctorApplicationForm {
    Bare,
    Prefix,
    Infix,
    Postfix,
    Bracket,
    Functional,
}

/// One single or paired source functor head.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceFunctorHeadSite {
    Single {
        site: TypedSiteRef,
        source_range: SourceRange,
        spelling: String,
    },
    Paired {
        left_site: TypedSiteRef,
        left_range: SourceRange,
        left_spelling: String,
        right_site: TypedSiteRef,
        right_range: SourceRange,
        right_spelling: String,
    },
}

/// Cross-family target owned by one application argument occurrence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceFunctorArgumentTarget {
    Primary(SourcePrimaryTermId),
    Application(SourceFunctorApplicationId),
}

/// Unresolved dependency request kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceFunctorTypeRequestKind {
    CandidateSignature,
    ApplicationResultType,
}

/// Immutable validated application handoff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFunctorApplicationHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    primary_term_fingerprint: String,
    applications: SourceFunctorApplicationTable,
    wrappers: SourceFunctorWrapperTable,
    candidates: SourceFunctorCandidateTable,
    arguments: SourceFunctorArgumentTable,
    type_requests: SourceFunctorTypeRequestTable,
}

impl SourceFunctorApplicationHandoff {
    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    pub fn primary_term_fingerprint(&self) -> &str {
        &self.primary_term_fingerprint
    }

    pub const fn applications(&self) -> &SourceFunctorApplicationTable {
        &self.applications
    }

    pub const fn wrappers(&self) -> &SourceFunctorWrapperTable {
        &self.wrappers
    }

    pub const fn candidates(&self) -> &SourceFunctorCandidateTable {
        &self.candidates
    }

    pub const fn arguments(&self) -> &SourceFunctorArgumentTable {
        &self.arguments
    }

    pub const fn type_requests(&self) -> &SourceFunctorTypeRequestTable {
        &self.type_requests
    }

    pub fn debug_text(&self) -> String {
        let mut output = String::from("source-functor-application-debug-v1\n");
        let _ = writeln!(output, "module: {}", self.module_id.path().as_str());
        let _ = writeln!(
            output,
            "primary-term-fingerprint: {:?}",
            self.primary_term_fingerprint
        );
        for (id, application) in self.applications.iter() {
            let _ = write!(
                output,
                "application#{} ordinal={} kind={} form={} head_ordinal={} range={}..{} site={} context={} recovery={} spelling={:?} head=",
                id.index(),
                application.source_ordinal,
                application_kind_key(application.kind),
                application_form_key(application.form),
                application.head_ordinal,
                application.source_range.start,
                application.source_range.end,
                application.site.node().index(),
                application.context.index(),
                application_recovery_key(application.recovery),
                application.spelling,
            );
            write_head(&mut output, &application.head);
            output.push('\n');
        }
        for (id, wrapper) in self.wrappers.iter() {
            let _ = writeln!(
                output,
                "wrapper#{} application={} ordinal={} range={}..{} site={} context={} recovery={} spelling={:?}",
                id.index(),
                wrapper.application.index(),
                wrapper.ordinal,
                wrapper.source_range.start,
                wrapper.source_range.end,
                wrapper.site.node().index(),
                wrapper.context.index(),
                application_recovery_key(wrapper.recovery),
                wrapper.spelling,
            );
        }
        for (id, candidate) in self.candidates.iter() {
            let _ = writeln!(
                output,
                "candidate#{} application={} ordinal={} symbol={:?} contribution={} origin={:?} visibility={:?} export={:?} signature={:?}",
                id.index(),
                candidate.application.index(),
                candidate.ordinal,
                candidate.symbol,
                candidate.contribution.index(),
                candidate.origin,
                candidate.visibility,
                candidate.export_status,
                candidate.signature,
            );
        }
        for (id, argument) in self.arguments.iter() {
            let _ = write!(
                output,
                "argument#{} application={} ordinal={} target=",
                id.index(),
                argument.application.index(),
                argument.ordinal,
            );
            write_target(&mut output, argument.target);
            output.push('\n');
        }
        for (id, request) in self.type_requests.iter() {
            let _ = write!(
                output,
                "type-request#{} application={} ordinal={} kind={} candidate=",
                id.index(),
                request.application.index(),
                request.request_ordinal,
                request_kind_key(request.kind),
            );
            if let Some(candidate) = request.candidate {
                let _ = write!(output, "{}", candidate.index());
            } else {
                output.push('-');
            }
            output.push('\n');
        }
        output
    }

    pub(crate) fn validate_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        primary_terms: &SourcePrimaryTermHandoff,
    ) -> Result<(), SourceFunctorApplicationError> {
        if self.source_id != source_id
            || &self.module_id != module_id
            || primary_terms.source_id() != source_id
            || primary_terms.module_id() != module_id
            || primary_terms.debug_text() != self.primary_term_fingerprint
        {
            return Err(SourceFunctorApplicationError::PrimaryDependencyMismatch);
        }

        let effective = output_effective_occurrences(self);
        let mut primary_targets = BTreeSet::new();
        let mut application_targets = BTreeSet::new();
        for (id, argument) in self.arguments.iter() {
            let Some(parent) = self.applications.get(argument.application) else {
                return Err(SourceFunctorApplicationError::InvalidArgument { argument: id });
            };
            let parent_range = effective[argument.application.index()].0;
            match argument.target {
                SourceFunctorArgumentTarget::Primary(primary_id) => {
                    let Some(primary) = primary_terms.terms().get(primary_id) else {
                        return Err(SourceFunctorApplicationError::InvalidArgument {
                            argument: id,
                        });
                    };
                    if primary.parent().is_some()
                        || primary.context() != parent.context
                        || !properly_contains(parent_range, primary.source_range())
                        || !primary_targets.insert(primary_id)
                    {
                        return Err(SourceFunctorApplicationError::InvalidArgument {
                            argument: id,
                        });
                    }
                }
                SourceFunctorArgumentTarget::Application(application_id) => {
                    let Some(child) = self.applications.get(application_id) else {
                        return Err(SourceFunctorApplicationError::InvalidArgument {
                            argument: id,
                        });
                    };
                    if application_id <= argument.application
                        || child.context != parent.context
                        || !properly_contains(parent_range, effective[application_id.index()].0)
                        || !application_targets.insert(application_id)
                    {
                        return Err(SourceFunctorApplicationError::InvalidArgument {
                            argument: id,
                        });
                    }
                }
            }
        }
        Ok(())
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
    SourceFunctorApplicationTable,
    SourceFunctorApplication,
    SourceFunctorApplicationId
);
table!(
    SourceFunctorWrapperTable,
    SourceFunctorWrapper,
    SourceFunctorWrapperId
);
table!(
    SourceFunctorCandidateTable,
    SourceFunctorCandidate,
    SourceFunctorCandidateId
);
table!(
    SourceFunctorArgumentTable,
    SourceFunctorArgument,
    SourceFunctorArgumentId
);
table!(
    SourceFunctorTypeRequestTable,
    SourceFunctorTypeRequest,
    SourceFunctorTypeRequestId
);

/// One validated source application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFunctorApplication {
    site: TypedSiteRef,
    source_range: SourceRange,
    source_ordinal: usize,
    context: BindingContextId,
    recovery: SourceFunctorApplicationRecovery,
    spelling: String,
    kind: SourceFunctorApplicationKind,
    form: SourceFunctorApplicationForm,
    head_ordinal: usize,
    head: SourceFunctorHeadSite,
}

impl SourceFunctorApplication {
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
    pub const fn recovery(&self) -> SourceFunctorApplicationRecovery {
        self.recovery
    }
    pub fn spelling(&self) -> &str {
        &self.spelling
    }
    pub const fn kind(&self) -> SourceFunctorApplicationKind {
        self.kind
    }
    pub const fn form(&self) -> SourceFunctorApplicationForm {
        self.form
    }
    pub const fn head_ordinal(&self) -> usize {
        self.head_ordinal
    }
    pub const fn head(&self) -> &SourceFunctorHeadSite {
        &self.head
    }
}

/// One validated transparent wrapper.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFunctorWrapper {
    application: SourceFunctorApplicationId,
    ordinal: usize,
    site: TypedSiteRef,
    source_range: SourceRange,
    context: BindingContextId,
    spelling: String,
    recovery: SourceFunctorApplicationRecovery,
}

impl SourceFunctorWrapper {
    pub const fn application(&self) -> SourceFunctorApplicationId {
        self.application
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
    pub const fn recovery(&self) -> SourceFunctorApplicationRecovery {
        self.recovery
    }
}

/// One validated resolver functor reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFunctorCandidate {
    application: SourceFunctorApplicationId,
    ordinal: usize,
    symbol: SymbolId,
    contribution: SourceContributionId,
    origin: SemanticOrigin,
    visibility: Visibility,
    export_status: ExportStatus,
    signature: Option<SignatureShell>,
}

impl SourceFunctorCandidate {
    pub const fn application(&self) -> SourceFunctorApplicationId {
        self.application
    }
    pub const fn ordinal(&self) -> usize {
        self.ordinal
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

/// One validated ordered argument edge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFunctorArgument {
    application: SourceFunctorApplicationId,
    ordinal: usize,
    target: SourceFunctorArgumentTarget,
}

impl SourceFunctorArgument {
    pub const fn application(&self) -> SourceFunctorApplicationId {
        self.application
    }
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }
    pub const fn target(&self) -> SourceFunctorArgumentTarget {
        self.target
    }
}

/// One validated unresolved type request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFunctorTypeRequest {
    application: SourceFunctorApplicationId,
    candidate: Option<SourceFunctorCandidateId>,
    request_ordinal: usize,
    kind: SourceFunctorTypeRequestKind,
}

impl SourceFunctorTypeRequest {
    pub const fn application(&self) -> SourceFunctorApplicationId {
        self.application
    }
    pub const fn candidate(&self) -> Option<SourceFunctorCandidateId> {
        self.candidate
    }
    pub const fn request_ordinal(&self) -> usize {
        self.request_ordinal
    }
    pub const fn kind(&self) -> SourceFunctorTypeRequestKind {
        self.kind
    }
}

/// Atomically validates and constructs source functor-application handoffs.
pub struct SourceFunctorApplicationProducer;

impl SourceFunctorApplicationProducer {
    pub fn build(
        input: SourceFunctorApplicationHandoffInput,
        symbols: &SymbolEnv,
        bindings: &BindingEnv,
        primary_terms: &SourcePrimaryTermHandoff,
        arena: &TypedArena,
    ) -> Result<SourceFunctorApplicationHandoff, SourceFunctorApplicationError> {
        validate_input(&input, symbols, bindings, primary_terms, arena)?;
        let primary_term_fingerprint = primary_terms.debug_text();

        let applications = SourceFunctorApplicationTable {
            rows: input
                .applications
                .into_iter()
                .map(|row| SourceFunctorApplication {
                    site: row.site,
                    source_range: row.source_range,
                    source_ordinal: row.source_ordinal,
                    context: row.context,
                    recovery: row.recovery,
                    spelling: row.spelling,
                    kind: row.kind,
                    form: row.form,
                    head_ordinal: row.head_ordinal,
                    head: row.head,
                })
                .collect(),
        };
        let wrappers = SourceFunctorWrapperTable {
            rows: input
                .wrappers
                .into_iter()
                .map(|row| SourceFunctorWrapper {
                    application: row.application,
                    ordinal: row.ordinal,
                    site: row.site,
                    source_range: row.source_range,
                    context: row.context,
                    spelling: row.spelling,
                    recovery: row.recovery,
                })
                .collect(),
        };
        let candidates = SourceFunctorCandidateTable {
            rows: input
                .candidates
                .into_iter()
                .map(|row| {
                    let entry = symbols
                        .symbols()
                        .get(&row.symbol)
                        .expect("candidate was authenticated");
                    SourceFunctorCandidate {
                        application: row.application,
                        ordinal: row.ordinal,
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
        let arguments = SourceFunctorArgumentTable {
            rows: input
                .arguments
                .into_iter()
                .map(|row| SourceFunctorArgument {
                    application: row.application,
                    ordinal: row.ordinal,
                    target: row.target,
                })
                .collect(),
        };
        let type_requests = SourceFunctorTypeRequestTable {
            rows: input
                .type_requests
                .into_iter()
                .map(|row| SourceFunctorTypeRequest {
                    application: row.application,
                    candidate: row.candidate,
                    request_ordinal: row.request_ordinal,
                    kind: row.kind,
                })
                .collect(),
        };

        Ok(SourceFunctorApplicationHandoff {
            source_id: input.source_id,
            module_id: input.module_id,
            primary_term_fingerprint,
            applications,
            wrappers,
            candidates,
            arguments,
            type_requests,
        })
    }
}

/// Atomic Task-253 producer failure.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceFunctorApplicationError {
    EnvironmentMismatch,
    PrimaryDependencyMismatch,
    InvalidApplication {
        application: SourceFunctorApplicationId,
    },
    InvalidHead {
        application: SourceFunctorApplicationId,
    },
    InvalidWrapper {
        wrapper: SourceFunctorWrapperId,
    },
    InvalidCandidate {
        candidate: SourceFunctorCandidateId,
    },
    InvalidArgument {
        argument: SourceFunctorArgumentId,
    },
    InvalidTypeRequest {
        request: SourceFunctorTypeRequestId,
    },
    DuplicateSite,
    ReorderedApplication {
        application: SourceFunctorApplicationId,
    },
    ReorderedWrapper {
        wrapper: SourceFunctorWrapperId,
    },
    ReorderedCandidate {
        candidate: SourceFunctorCandidateId,
    },
    ReorderedArgument {
        argument: SourceFunctorArgumentId,
    },
    DuplicateArgumentTarget {
        argument: SourceFunctorArgumentId,
    },
    MultipleParents {
        application: SourceFunctorApplicationId,
    },
    OverlappingArguments {
        application: SourceFunctorApplicationId,
    },
    InvalidForm {
        application: SourceFunctorApplicationId,
    },
}

impl fmt::Display for SourceFunctorApplicationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EnvironmentMismatch => {
                formatter.write_str("source application environment identity mismatch")
            }
            Self::PrimaryDependencyMismatch => {
                formatter.write_str("source application primary-term dependency mismatch")
            }
            Self::InvalidApplication { application } => write!(
                formatter,
                "source application {} is invalid",
                application.index()
            ),
            Self::InvalidHead { application } => write!(
                formatter,
                "source application {} has an invalid head",
                application.index()
            ),
            Self::InvalidWrapper { wrapper } => {
                write!(formatter, "source wrapper {} is invalid", wrapper.index())
            }
            Self::InvalidCandidate { candidate } => write!(
                formatter,
                "source application candidate {} is invalid",
                candidate.index()
            ),
            Self::InvalidArgument { argument } => write!(
                formatter,
                "source application argument {} is invalid",
                argument.index()
            ),
            Self::InvalidTypeRequest { request } => write!(
                formatter,
                "source application type request {} is invalid",
                request.index()
            ),
            Self::DuplicateSite => formatter.write_str("source application repeats a typed site"),
            Self::ReorderedApplication { application } => write!(
                formatter,
                "source application {} is out of source pre-order",
                application.index()
            ),
            Self::ReorderedWrapper { wrapper } => {
                write!(
                    formatter,
                    "source wrapper {} is out of order",
                    wrapper.index()
                )
            }
            Self::ReorderedCandidate { candidate } => write!(
                formatter,
                "source application candidate {} is out of order",
                candidate.index()
            ),
            Self::ReorderedArgument { argument } => write!(
                formatter,
                "source application argument {} is out of order",
                argument.index()
            ),
            Self::DuplicateArgumentTarget { argument } => write!(
                formatter,
                "source application argument {} repeats an owned occurrence",
                argument.index()
            ),
            Self::MultipleParents { application } => write!(
                formatter,
                "source application {} has multiple parents",
                application.index()
            ),
            Self::OverlappingArguments { application } => write!(
                formatter,
                "source application {} has overlapping arguments",
                application.index()
            ),
            Self::InvalidForm { application } => write!(
                formatter,
                "source application {} has an invalid source form",
                application.index()
            ),
        }
    }
}

impl Error for SourceFunctorApplicationError {}

fn validate_input(
    input: &SourceFunctorApplicationHandoffInput,
    symbols: &SymbolEnv,
    bindings: &BindingEnv,
    primary_terms: &SourcePrimaryTermHandoff,
    arena: &TypedArena,
) -> Result<(), SourceFunctorApplicationError> {
    if symbols.module_id() != &input.module_id
        || bindings.source_id() != input.source_id
        || bindings.module_id() != &input.module_id
        || primary_terms.source_id() != input.source_id
        || primary_terms.module_id() != &input.module_id
    {
        return Err(SourceFunctorApplicationError::EnvironmentMismatch);
    }
    if input.applications.is_empty()
        && (!input.wrappers.is_empty()
            || !input.candidates.is_empty()
            || !input.arguments.is_empty()
            || !input.type_requests.is_empty())
    {
        return Err(SourceFunctorApplicationError::EnvironmentMismatch);
    }

    let mut sites = BTreeSet::new();
    validate_applications(input, bindings, arena, &mut sites)?;
    let wrapper_groups = validate_wrappers(input, arena, &mut sites)?;
    let effective = input_effective_occurrences(input, &wrapper_groups);
    validate_application_preorder(input, &effective)?;
    validate_candidates(input, symbols)?;
    let argument_groups = validate_arguments(input, primary_terms, &effective, &mut sites)?;
    validate_application_ownership(input, &effective)?;
    validate_forms(input, primary_terms, &argument_groups, &effective)?;
    validate_requests(input)?;
    Ok(())
}

fn validate_applications(
    input: &SourceFunctorApplicationHandoffInput,
    bindings: &BindingEnv,
    arena: &TypedArena,
    sites: &mut BTreeSet<TypedSiteRef>,
) -> Result<(), SourceFunctorApplicationError> {
    for (index, application) in input.applications.iter().enumerate() {
        let id = SourceFunctorApplicationId::new(index);
        if application.source_ordinal != index
            || bindings.contexts().get(application.context).is_none()
            || !valid_range(input.source_id, application.source_range)
            || !canonical_spelling(&application.spelling)
        {
            return Err(SourceFunctorApplicationError::InvalidApplication { application: id });
        }
        validate_arena_site(
            &application.site,
            application.source_range,
            application_kind_node_key(application.kind),
            application.recovery,
            arena,
        )
        .map_err(|()| SourceFunctorApplicationError::InvalidApplication { application: id })?;
        if !sites.insert(application.site.clone()) {
            return Err(SourceFunctorApplicationError::DuplicateSite);
        }
        validate_head(input.source_id, id, application, arena, sites)?;
    }
    Ok(())
}

fn validate_head(
    source_id: SourceId,
    id: SourceFunctorApplicationId,
    application: &SourceFunctorApplicationInput,
    arena: &TypedArena,
    sites: &mut BTreeSet<TypedSiteRef>,
) -> Result<(), SourceFunctorApplicationError> {
    let invalid = || SourceFunctorApplicationError::InvalidHead { application: id };
    match (&application.form, &application.head) {
        (
            SourceFunctorApplicationForm::Bracket,
            SourceFunctorHeadSite::Paired {
                left_site,
                left_range,
                left_spelling,
                right_site,
                right_range,
                right_spelling,
            },
        ) => {
            if !valid_range(source_id, *left_range)
                || !valid_range(source_id, *right_range)
                || left_range.end > right_range.start
                || !range_contains(application.source_range, *left_range)
                || !range_contains(application.source_range, *right_range)
                || !canonical_spelling(left_spelling)
                || !canonical_spelling(right_spelling)
            {
                return Err(invalid());
            }
            validate_arena_site(
                left_site,
                *left_range,
                "source.term.functor-head.bracket",
                application.recovery,
                arena,
            )
            .map_err(|()| invalid())?;
            validate_arena_site(
                right_site,
                *right_range,
                "source.term.functor-head.bracket",
                application.recovery,
                arena,
            )
            .map_err(|()| invalid())?;
            if !sites.insert(left_site.clone()) || !sites.insert(right_site.clone()) {
                return Err(SourceFunctorApplicationError::DuplicateSite);
            }
        }
        (
            SourceFunctorApplicationForm::Bare
            | SourceFunctorApplicationForm::Prefix
            | SourceFunctorApplicationForm::Infix
            | SourceFunctorApplicationForm::Postfix
            | SourceFunctorApplicationForm::Functional,
            SourceFunctorHeadSite::Single {
                site,
                source_range,
                spelling,
            },
        ) => {
            if !valid_range(source_id, *source_range)
                || !range_contains(application.source_range, *source_range)
                || !canonical_spelling(spelling)
            {
                return Err(invalid());
            }
            validate_arena_site(
                site,
                *source_range,
                "source.term.functor-head.single",
                application.recovery,
                arena,
            )
            .map_err(|()| invalid())?;
            if !sites.insert(site.clone()) {
                return Err(SourceFunctorApplicationError::DuplicateSite);
            }
        }
        _ => return Err(invalid()),
    }
    Ok(())
}

fn validate_wrappers(
    input: &SourceFunctorApplicationHandoffInput,
    arena: &TypedArena,
    sites: &mut BTreeSet<TypedSiteRef>,
) -> Result<Vec<Vec<usize>>, SourceFunctorApplicationError> {
    let groups = grouped_rows(
        input.applications.len(),
        &input.wrappers,
        |row| row.application,
        |row| row.ordinal,
        |index| SourceFunctorApplicationError::ReorderedWrapper {
            wrapper: SourceFunctorWrapperId::new(index),
        },
    )?;
    for (application_index, group) in groups.iter().enumerate() {
        let application_id = SourceFunctorApplicationId::new(application_index);
        let application = &input.applications[application_index];
        let mut contained_range = application.source_range;
        let mut contained_spelling = application.spelling.as_str();
        for wrapper_index in group.iter().rev().copied() {
            let id = SourceFunctorWrapperId::new(wrapper_index);
            let wrapper = &input.wrappers[wrapper_index];
            if wrapper.application != application_id
                || wrapper.context != application.context
                || !valid_range(input.source_id, wrapper.source_range)
                || !strictly_contains(wrapper.source_range, contained_range)
                || wrapper.spelling != format!("( {contained_spelling} )")
            {
                return Err(SourceFunctorApplicationError::InvalidWrapper { wrapper: id });
            }
            validate_arena_site(
                &wrapper.site,
                wrapper.source_range,
                "source.term.functor-application.parenthesized",
                wrapper.recovery,
                arena,
            )
            .map_err(|()| SourceFunctorApplicationError::InvalidWrapper { wrapper: id })?;
            if !sites.insert(wrapper.site.clone()) {
                return Err(SourceFunctorApplicationError::DuplicateSite);
            }
            contained_range = wrapper.source_range;
            contained_spelling = &wrapper.spelling;
        }
    }
    Ok(groups)
}

fn validate_application_preorder(
    input: &SourceFunctorApplicationHandoffInput,
    effective: &[(SourceRange, String)],
) -> Result<(), SourceFunctorApplicationError> {
    for right in 1..input.applications.len() {
        for left in 0..right {
            let left_range = effective[left].0;
            let right_range = effective[right].0;
            if right_range.start < left_range.start
                || (right_range.start < left_range.end
                    && !properly_contains(left_range, right_range))
            {
                return Err(SourceFunctorApplicationError::ReorderedApplication {
                    application: SourceFunctorApplicationId::new(right),
                });
            }
        }
    }
    Ok(())
}

fn validate_candidates(
    input: &SourceFunctorApplicationHandoffInput,
    symbols: &SymbolEnv,
) -> Result<(), SourceFunctorApplicationError> {
    let groups = grouped_rows(
        input.applications.len(),
        &input.candidates,
        |row| row.application,
        |row| row.ordinal,
        |index| SourceFunctorApplicationError::ReorderedCandidate {
            candidate: SourceFunctorCandidateId::new(index),
        },
    )?;
    for (application_index, group) in groups.iter().enumerate() {
        let application = &input.applications[application_index];
        match application.kind {
            SourceFunctorApplicationKind::Symbolic if group.is_empty() => {
                return Err(SourceFunctorApplicationError::InvalidApplication {
                    application: SourceFunctorApplicationId::new(application_index),
                });
            }
            SourceFunctorApplicationKind::Inline if !group.is_empty() => {
                return Err(SourceFunctorApplicationError::InvalidCandidate {
                    candidate: SourceFunctorCandidateId::new(group[0]),
                });
            }
            _ => {}
        }
        let mut previous_symbol: Option<&SymbolId> = None;
        for candidate_index in group.iter().copied() {
            let id = SourceFunctorCandidateId::new(candidate_index);
            let candidate = &input.candidates[candidate_index];
            if previous_symbol.is_some_and(|previous| previous >= &candidate.symbol) {
                return Err(SourceFunctorApplicationError::ReorderedCandidate { candidate: id });
            }
            validate_candidate(input, id, application, candidate, symbols)?;
            previous_symbol = Some(&candidate.symbol);
        }
    }
    Ok(())
}

fn validate_candidate(
    input: &SourceFunctorApplicationHandoffInput,
    id: SourceFunctorCandidateId,
    application: &SourceFunctorApplicationInput,
    candidate: &SourceFunctorCandidateInput,
    symbols: &SymbolEnv,
) -> Result<(), SourceFunctorApplicationError> {
    let invalid = || SourceFunctorApplicationError::InvalidCandidate { candidate: id };
    let entry = symbols
        .symbols()
        .get(&candidate.symbol)
        .ok_or_else(invalid)?;
    let contribution = symbols
        .contributions()
        .get(candidate.contribution)
        .ok_or_else(invalid)?;
    if entry.kind() != SymbolKind::Functor
        || entry.contribution() != candidate.contribution
        || entry.namespace().as_str() != input.module_id.path().as_str()
        || contribution.module() != candidate.symbol.module()
        || !contribution.effects().symbols().contains(&candidate.symbol)
        || entry.origin().is_recovered()
        || matches!(entry.signature(), Some(SignatureShell::Malformed { .. }))
    {
        return Err(invalid());
    }

    match contribution.kind() {
        ContributionKind::LocalSource { source_id } => {
            let definition = symbols
                .definitions()
                .by_symbol(&candidate.symbol)
                .ok_or_else(invalid)?;
            let origin_range = source_range(entry.origin().anchor()).ok_or_else(invalid)?;
            if *source_id != input.source_id
                || contribution.module() != &input.module_id
                || candidate.symbol.module() != &input.module_id
                || entry.origin().source_id() != input.source_id
                || entry.origin().module_id() != &input.module_id
                || entry.origin().import_edge().is_some()
                || !valid_range(input.source_id, origin_range)
                || origin_range.end > application.source_range.start
                || definition.kind() != DefinitionKind::Functor
                || definition.symbol() != &candidate.symbol
                || definition.contribution() != candidate.contribution
                || definition.origin() != entry.origin()
                || definition.visibility() != entry.visibility()
                || definition.signature() != entry.signature()
                || definition.conflict().is_some()
                || !contribution
                    .effects()
                    .definitions()
                    .contains(&definition.id())
                || matches!(
                    (entry.notation_spelling(), definition.notation_shape()),
                    (Some(left), Some(right)) if left != right
                )
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
                    == Some(candidate.symbol.module())
            });
            if *source_id != input.source_id
                || !valid_imported_candidate_provenance(
                    entry,
                    &candidate.symbol,
                    input.source_id,
                    application.source_range,
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

fn valid_imported_candidate_provenance(
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

fn validate_arguments(
    input: &SourceFunctorApplicationHandoffInput,
    primary_terms: &SourcePrimaryTermHandoff,
    effective: &[(SourceRange, String)],
    sites: &mut BTreeSet<TypedSiteRef>,
) -> Result<Vec<Vec<usize>>, SourceFunctorApplicationError> {
    let groups = grouped_rows(
        input.applications.len(),
        &input.arguments,
        |row| row.application,
        |row| row.ordinal,
        |index| SourceFunctorApplicationError::ReorderedArgument {
            argument: SourceFunctorArgumentId::new(index),
        },
    )?;
    let mut primary_targets = BTreeSet::new();
    let mut application_targets = BTreeSet::new();
    for (application_index, group) in groups.iter().enumerate() {
        let application_id = SourceFunctorApplicationId::new(application_index);
        let application = &input.applications[application_index];
        let parent_range = effective[application_index].0;
        let mut previous_range = None;
        for argument_index in group.iter().copied() {
            let id = SourceFunctorArgumentId::new(argument_index);
            let argument = &input.arguments[argument_index];
            let target_range =
                match argument.target {
                    SourceFunctorArgumentTarget::Primary(primary_id) => {
                        let primary = primary_terms.terms().get(primary_id).ok_or(
                            SourceFunctorApplicationError::InvalidArgument { argument: id },
                        )?;
                        if primary.parent().is_some()
                            || primary.context() != application.context
                            || !primary_targets.insert(primary_id)
                            || !sites.insert(primary.site().clone())
                        {
                            return Err(SourceFunctorApplicationError::DuplicateArgumentTarget {
                                argument: id,
                            });
                        }
                        primary.source_range()
                    }
                    SourceFunctorArgumentTarget::Application(child_id) => {
                        let child = input.applications.get(child_id.index()).ok_or(
                            SourceFunctorApplicationError::InvalidArgument { argument: id },
                        )?;
                        if child_id <= application_id || child.context != application.context {
                            return Err(SourceFunctorApplicationError::InvalidArgument {
                                argument: id,
                            });
                        }
                        if !application_targets.insert(child_id) {
                            return Err(SourceFunctorApplicationError::MultipleParents {
                                application: child_id,
                            });
                        }
                        effective[child_id.index()].0
                    }
                };
            if !properly_contains(parent_range, target_range) {
                return Err(SourceFunctorApplicationError::InvalidArgument { argument: id });
            }
            if previous_range.is_some_and(|previous: SourceRange| previous.end > target_range.start)
            {
                return Err(SourceFunctorApplicationError::OverlappingArguments {
                    application: application_id,
                });
            }
            previous_range = Some(target_range);
        }
    }
    Ok(groups)
}

fn validate_application_ownership(
    input: &SourceFunctorApplicationHandoffInput,
    effective: &[(SourceRange, String)],
) -> Result<(), SourceFunctorApplicationError> {
    let targets = input
        .arguments
        .iter()
        .filter_map(|argument| match argument.target {
            SourceFunctorArgumentTarget::Application(application) => Some(application),
            SourceFunctorArgumentTarget::Primary(_) => None,
        })
        .collect::<BTreeSet<_>>();
    for index in 0..input.applications.len() {
        let application = SourceFunctorApplicationId::new(index);
        let geometrically_nested =
            (0..index).any(|parent| properly_contains(effective[parent].0, effective[index].0));
        if geometrically_nested != targets.contains(&application) {
            return Err(SourceFunctorApplicationError::InvalidApplication { application });
        }
    }
    Ok(())
}

fn validate_forms(
    input: &SourceFunctorApplicationHandoffInput,
    primary_terms: &SourcePrimaryTermHandoff,
    argument_groups: &[Vec<usize>],
    effective: &[(SourceRange, String)],
) -> Result<(), SourceFunctorApplicationError> {
    for (index, application) in input.applications.iter().enumerate() {
        let id = SourceFunctorApplicationId::new(index);
        let arguments = &argument_groups[index];
        if application.kind == SourceFunctorApplicationKind::Inline
            && application.form != SourceFunctorApplicationForm::Functional
        {
            return Err(SourceFunctorApplicationError::InvalidForm { application: id });
        }
        let valid_cardinality = match application.form {
            SourceFunctorApplicationForm::Bare => arguments.is_empty(),
            SourceFunctorApplicationForm::Prefix | SourceFunctorApplicationForm::Postfix => {
                arguments.len() == 1
            }
            SourceFunctorApplicationForm::Infix => arguments.len() == 2,
            SourceFunctorApplicationForm::Bracket => !arguments.is_empty(),
            SourceFunctorApplicationForm::Functional => {
                application.kind == SourceFunctorApplicationKind::Inline || !arguments.is_empty()
            }
        };
        let expected_head_ordinal = match application.form {
            SourceFunctorApplicationForm::Infix => 1,
            SourceFunctorApplicationForm::Postfix => arguments.len(),
            SourceFunctorApplicationForm::Bare
            | SourceFunctorApplicationForm::Prefix
            | SourceFunctorApplicationForm::Bracket
            | SourceFunctorApplicationForm::Functional => 0,
        };
        if !valid_cardinality || application.head_ordinal != expected_head_ordinal {
            return Err(SourceFunctorApplicationError::InvalidForm { application: id });
        }

        let target_occurrences = arguments
            .iter()
            .map(
                |argument_index| match input.arguments[*argument_index].target {
                    SourceFunctorArgumentTarget::Primary(primary_id) => primary_terms
                        .terms()
                        .get(primary_id)
                        .map(|primary| (primary.source_range(), primary.spelling())),
                    SourceFunctorArgumentTarget::Application(child) => Some((
                        effective[child.index()].0,
                        effective[child.index()].1.as_str(),
                    )),
                },
            )
            .collect::<Option<Vec<_>>>()
            .ok_or(SourceFunctorApplicationError::InvalidForm { application: id })?;
        if target_occurrences
            .iter()
            .any(|(range, _)| !properly_contains(application.source_range, *range))
        {
            return Err(SourceFunctorApplicationError::InvalidForm { application: id });
        }
        let argument_spellings = target_occurrences
            .iter()
            .map(|(_, spelling)| *spelling)
            .collect::<Vec<_>>();
        let argument_ranges = target_occurrences
            .iter()
            .map(|(range, _)| *range)
            .collect::<Vec<_>>();

        let (expected_spelling, positions_valid) = match &application.head {
            SourceFunctorHeadSite::Single {
                source_range,
                spelling,
                ..
            } => match application.form {
                SourceFunctorApplicationForm::Bare => {
                    (spelling.clone(), application.source_range == *source_range)
                }
                SourceFunctorApplicationForm::Prefix => (
                    format!("{spelling} {}", argument_spellings[0]),
                    application.source_range.start == source_range.start
                        && application.source_range.end == argument_ranges[0].end
                        && source_range.end <= argument_ranges[0].start,
                ),
                SourceFunctorApplicationForm::Infix => (
                    format!(
                        "{} {spelling} {}",
                        argument_spellings[0], argument_spellings[1]
                    ),
                    application.source_range.start == argument_ranges[0].start
                        && application.source_range.end == argument_ranges[1].end
                        && argument_ranges[0].end <= source_range.start
                        && source_range.end <= argument_ranges[1].start,
                ),
                SourceFunctorApplicationForm::Postfix => (
                    format!("{} {spelling}", argument_spellings[0]),
                    application.source_range.start == argument_ranges[0].start
                        && application.source_range.end == source_range.end
                        && argument_ranges[0].end <= source_range.start,
                ),
                SourceFunctorApplicationForm::Functional => {
                    let spelling = if argument_spellings.is_empty() {
                        format!("{spelling} ( )")
                    } else {
                        format!("{spelling} ( {} )", argument_spellings.join(" , "))
                    };
                    (
                        spelling,
                        application.source_range.start == source_range.start
                            && application.source_range.end > source_range.end
                            && argument_ranges.first().is_none_or(|first| {
                                source_range.end <= first.start
                                    && application.source_range.end
                                        > argument_ranges.last().expect("nonempty").end
                            }),
                    )
                }
                SourceFunctorApplicationForm::Bracket => (String::new(), false),
            },
            SourceFunctorHeadSite::Paired {
                left_range,
                left_spelling,
                right_range,
                right_spelling,
                ..
            } => (
                format!(
                    "{left_spelling} {} {right_spelling}",
                    argument_spellings.join(" , ")
                ),
                application.form == SourceFunctorApplicationForm::Bracket
                    && application.source_range.start == left_range.start
                    && application.source_range.end == right_range.end
                    && left_range.end <= argument_ranges[0].start
                    && argument_ranges.last().expect("bracket nonempty").end <= right_range.start,
            ),
        };
        if !positions_valid || application.spelling != expected_spelling {
            return Err(SourceFunctorApplicationError::InvalidForm { application: id });
        }
    }
    Ok(())
}

fn validate_requests(
    input: &SourceFunctorApplicationHandoffInput,
) -> Result<(), SourceFunctorApplicationError> {
    let request_groups = grouped_rows(
        input.applications.len(),
        &input.type_requests,
        |row| row.application,
        |row| row.request_ordinal,
        |index| SourceFunctorApplicationError::InvalidTypeRequest {
            request: SourceFunctorTypeRequestId::new(index),
        },
    )?;
    let candidate_groups = group_candidate_ids(input);
    for (application_index, requests) in request_groups.iter().enumerate() {
        let candidates = &candidate_groups[application_index];
        let expected_len = match input.applications[application_index].kind {
            SourceFunctorApplicationKind::Symbolic => candidates.len() + 1,
            SourceFunctorApplicationKind::Inline => 0,
        };
        if requests.len() != expected_len {
            let request = requests
                .first()
                .copied()
                .unwrap_or(input.type_requests.len());
            return Err(SourceFunctorApplicationError::InvalidTypeRequest {
                request: SourceFunctorTypeRequestId::new(request),
            });
        }
        for (ordinal, candidate) in candidates.iter().copied().enumerate() {
            let request_index = requests[ordinal];
            let request = &input.type_requests[request_index];
            if request.candidate != Some(candidate)
                || request.kind != SourceFunctorTypeRequestKind::CandidateSignature
            {
                return Err(SourceFunctorApplicationError::InvalidTypeRequest {
                    request: SourceFunctorTypeRequestId::new(request_index),
                });
            }
        }
        if let Some(request_index) = requests.last().copied() {
            let request = &input.type_requests[request_index];
            if request.candidate.is_some()
                || request.kind != SourceFunctorTypeRequestKind::ApplicationResultType
            {
                return Err(SourceFunctorApplicationError::InvalidTypeRequest {
                    request: SourceFunctorTypeRequestId::new(request_index),
                });
            }
        }
    }
    Ok(())
}

fn group_candidate_ids(
    input: &SourceFunctorApplicationHandoffInput,
) -> Vec<Vec<SourceFunctorCandidateId>> {
    let mut groups = vec![Vec::new(); input.applications.len()];
    for (index, candidate) in input.candidates.iter().enumerate() {
        if let Some(group) = groups.get_mut(candidate.application.index()) {
            group.push(SourceFunctorCandidateId::new(index));
        }
    }
    groups
}

fn grouped_rows<T, FApplication, FOrdinal, FError>(
    application_count: usize,
    rows: &[T],
    application: FApplication,
    ordinal: FOrdinal,
    error: FError,
) -> Result<Vec<Vec<usize>>, SourceFunctorApplicationError>
where
    FApplication: Fn(&T) -> SourceFunctorApplicationId,
    FOrdinal: Fn(&T) -> usize,
    FError: Fn(usize) -> SourceFunctorApplicationError,
{
    let mut groups = vec![Vec::new(); application_count];
    let mut previous_application = 0;
    for (index, row) in rows.iter().enumerate() {
        let application_id = application(row);
        let application_index = application_id.index();
        let Some(group) = groups.get_mut(application_index) else {
            return Err(error(index));
        };
        if (index > 0 && application_index < previous_application) || ordinal(row) != group.len() {
            return Err(error(index));
        }
        group.push(index);
        previous_application = application_index;
    }
    Ok(groups)
}

fn input_effective_occurrences(
    input: &SourceFunctorApplicationHandoffInput,
    wrapper_groups: &[Vec<usize>],
) -> Vec<(SourceRange, String)> {
    input
        .applications
        .iter()
        .enumerate()
        .map(|(index, application)| {
            wrapper_groups[index].first().map_or_else(
                || (application.source_range, application.spelling.clone()),
                |wrapper| {
                    let wrapper = &input.wrappers[*wrapper];
                    (wrapper.source_range, wrapper.spelling.clone())
                },
            )
        })
        .collect()
}

fn output_effective_occurrences(
    handoff: &SourceFunctorApplicationHandoff,
) -> Vec<(SourceRange, &str)> {
    handoff
        .applications
        .iter()
        .map(|(application_id, application)| {
            handoff
                .wrappers
                .iter()
                .find(|(_, wrapper)| wrapper.application == application_id && wrapper.ordinal == 0)
                .map_or(
                    (application.source_range, application.spelling.as_str()),
                    |(_, wrapper)| (wrapper.source_range, wrapper.spelling.as_str()),
                )
        })
        .collect()
}

fn validate_arena_site(
    site: &TypedSiteRef,
    source_range: SourceRange,
    kind: &str,
    recovery: SourceFunctorApplicationRecovery,
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

fn recovery_matches(
    recovery: SourceFunctorApplicationRecovery,
    node_recovery: NodeRecoveryState,
) -> bool {
    match recovery {
        SourceFunctorApplicationRecovery::Normal => node_recovery == NodeRecoveryState::Normal,
        SourceFunctorApplicationRecovery::Degraded => matches!(
            node_recovery,
            NodeRecoveryState::Recovered | NodeRecoveryState::Degraded
        ),
    }
}

fn application_kind_node_key(kind: SourceFunctorApplicationKind) -> &'static str {
    match kind {
        SourceFunctorApplicationKind::Symbolic => "source.term.functor-application.symbolic",
        SourceFunctorApplicationKind::Inline => "source.term.functor-application.inline",
    }
}

fn application_kind_key(kind: SourceFunctorApplicationKind) -> &'static str {
    match kind {
        SourceFunctorApplicationKind::Symbolic => "symbolic",
        SourceFunctorApplicationKind::Inline => "inline",
    }
}

fn application_form_key(form: SourceFunctorApplicationForm) -> &'static str {
    match form {
        SourceFunctorApplicationForm::Bare => "bare",
        SourceFunctorApplicationForm::Prefix => "prefix",
        SourceFunctorApplicationForm::Infix => "infix",
        SourceFunctorApplicationForm::Postfix => "postfix",
        SourceFunctorApplicationForm::Bracket => "bracket",
        SourceFunctorApplicationForm::Functional => "functional",
    }
}

fn application_recovery_key(recovery: SourceFunctorApplicationRecovery) -> &'static str {
    match recovery {
        SourceFunctorApplicationRecovery::Normal => "normal",
        SourceFunctorApplicationRecovery::Degraded => "degraded",
    }
}

fn request_kind_key(kind: SourceFunctorTypeRequestKind) -> &'static str {
    match kind {
        SourceFunctorTypeRequestKind::CandidateSignature => "candidate-signature",
        SourceFunctorTypeRequestKind::ApplicationResultType => "application-result-type",
    }
}

fn write_head(output: &mut String, head: &SourceFunctorHeadSite) {
    match head {
        SourceFunctorHeadSite::Single {
            site,
            source_range,
            spelling,
        } => {
            let _ = write!(
                output,
                "single(site={},range={}..{},spelling={:?})",
                site.node().index(),
                source_range.start,
                source_range.end,
                spelling
            );
        }
        SourceFunctorHeadSite::Paired {
            left_site,
            left_range,
            left_spelling,
            right_site,
            right_range,
            right_spelling,
        } => {
            let _ = write!(
                output,
                "paired(left_site={},left_range={}..{},left_spelling={:?},right_site={},right_range={}..{},right_spelling={:?})",
                left_site.node().index(),
                left_range.start,
                left_range.end,
                left_spelling,
                right_site.node().index(),
                right_range.start,
                right_range.end,
                right_spelling,
            );
        }
    }
}

fn write_target(output: &mut String, target: SourceFunctorArgumentTarget) {
    match target {
        SourceFunctorArgumentTarget::Primary(id) => {
            let _ = write!(output, "primary({})", id.index());
        }
        SourceFunctorArgumentTarget::Application(id) => {
            let _ = write!(output, "application({})", id.index());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        binding_env::{
            BinderIdentity, BindingContextDraft, BindingContextLayer, BindingContextOwner,
            BindingContextRecovery, BindingContextTable, BindingDiagnosticTable, BindingDraft,
            BindingEnvParts, BindingKind, BindingRecoveryState, BindingStatus, BindingTable,
            BindingTypeSite, CapturedFreeVariables,
        },
        source_term::{
            SourceNumericTypeRequestInput, SourcePrimaryTermHandoffInput, SourcePrimaryTermInput,
            SourcePrimaryTermKind, SourcePrimaryTermProducer, SourcePrimaryTermRecovery,
            SourcePrimaryTermReferenceInput, SourcePrimaryTermReferenceRole, SourcePrimaryTermRole,
        },
        typed_ast::{
            CoercionTable, InitialObligationTable, LocalTypeContextTable, TypeDiagnosticTable,
            TypeFactTable, TypeTable, TypedAst, TypedAstError, TypedAstParts, TypedNode,
            TypedNodeId,
        },
    };
    use crate::{
        cluster_trace::ClusterFactTable,
        overload_resolution::{
            CandidateViabilityInput, CandidateViabilityOutput, OverloadCandidateInput,
            OverloadCollectionOutput, OverloadSelectionOutput, OverloadSiteInput,
            OverloadSiteResolutionInput, SpecificityComparisonInput, SpecificityGraphOutput,
            TemplateExpansionOutput,
        },
        resolved_typed_ast::{ResolvedTypedAst, ResolvedTypedAstInputs},
    };
    use mizar_resolve::{
        env::{
            ContributionKind, DeclarationConflictClass, DefinitionShell, NamespacePath,
            SignatureShell, SymbolEntry, SymbolEnvIndexes,
        },
        names::LocalTermScope,
        resolved_ast::{FullyQualifiedName, LocalSymbolId},
    };
    use mizar_session::{
        BuildSnapshotId, InMemorySessionIdAllocator, ModulePath, PackageId, SessionIdAllocator as _,
    };

    #[derive(Clone)]
    struct Fixture {
        source: SourceId,
        module: ModuleId,
        input: SourceFunctorApplicationHandoffInput,
        symbols: SymbolEnv,
        bindings: BindingEnv,
        primary: SourcePrimaryTermHandoff,
        arena: TypedArena,
    }

    #[derive(Clone)]
    struct CandidateOptions {
        kind: SymbolKind,
        symbol_signature: Option<SignatureShell>,
        definition_signature: Option<SignatureShell>,
        conflict: Option<DeclarationConflictClass>,
        symbol_notation: Option<String>,
        definition_notation: Option<String>,
        recovered: bool,
        include_definition: bool,
        definition_kind: DefinitionKind,
        origin_after_use: bool,
        definition_origin_drift: bool,
        definition_contribution_drift: bool,
    }

    impl Default for CandidateOptions {
        fn default() -> Self {
            Self {
                kind: SymbolKind::Functor,
                symbol_signature: None,
                definition_signature: None,
                conflict: None,
                symbol_notation: None,
                definition_notation: None,
                recovered: false,
                include_definition: true,
                definition_kind: DefinitionKind::Functor,
                origin_after_use: false,
                definition_origin_drift: false,
                definition_contribution_drift: false,
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

    fn symbol_id(module: &ModuleId, ordinal: usize) -> SymbolId {
        SymbolId::new(
            module.clone(),
            LocalSymbolId::new(format!("functor-{ordinal}")),
            FullyQualifiedName::new(format!("{}::functor/{ordinal}", module.path().as_str())),
        )
    }

    fn local_symbols(
        source: SourceId,
        module: &ModuleId,
        options: CandidateOptions,
    ) -> (SymbolEnv, SymbolId, SourceContributionId) {
        let mut indexes = SymbolEnvIndexes::default();
        let contribution = indexes.contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 9)),
        );
        let symbol = symbol_id(module, 0);
        let origin_range = if options.origin_after_use {
            range(source, 20, 25)
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
        let mut symbol_entry = SymbolEntry::new(
            symbol.clone(),
            options.kind,
            NamespacePath::new(module.path().as_str()),
            "f",
            origin.clone(),
            contribution,
        );
        if let Some(signature) = options.symbol_signature {
            symbol_entry = symbol_entry.with_signature(signature);
        }
        if let Some(notation) = options.symbol_notation {
            symbol_entry = symbol_entry.with_notation_spelling(notation);
        }
        indexes.symbols.insert(symbol_entry);
        indexes
            .contributions
            .add_symbol(contribution, symbol.clone());

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
            if let Some(signature) = options.definition_signature {
                definition = definition.with_signature(signature);
            }
            if let Some(conflict) = options.conflict {
                definition = definition.with_conflict(conflict);
            }
            if let Some(notation) = options.definition_notation {
                definition = definition.with_notation_shape(notation);
            }
            let definition = indexes.definitions.insert(definition);
            indexes
                .contributions
                .add_definition(definition_contribution, definition);
        }
        (
            SymbolEnv::new(module.clone(), indexes),
            symbol,
            contribution,
        )
    }

    fn install_matching_local_symbols(fixture: &mut Fixture, count: usize, supplied: usize) {
        let mut indexes = SymbolEnvIndexes::default();
        let contribution = indexes.contributions.insert(
            fixture.module.clone(),
            ContributionKind::LocalSource {
                source_id: fixture.source,
            },
            SourceAnchor::Range(range(fixture.source, 0, 9)),
        );
        let mut symbols = Vec::new();
        for ordinal in 0..count {
            let symbol = symbol_id(&fixture.module, ordinal);
            let origin = SemanticOrigin::new(
                fixture.source,
                fixture.module.clone(),
                SourceAnchor::Range(range(fixture.source, 1, 5)),
                vec![ordinal as u32],
            );
            indexes.symbols.insert(SymbolEntry::new(
                symbol.clone(),
                SymbolKind::Functor,
                NamespacePath::new(fixture.module.path().as_str()),
                "f",
                origin.clone(),
                contribution,
            ));
            indexes
                .contributions
                .add_symbol(contribution, symbol.clone());
            let definition = indexes.definitions.insert(DefinitionShell::new(
                symbol.clone(),
                DefinitionKind::Functor,
                origin,
                contribution,
            ));
            indexes
                .contributions
                .add_definition(contribution, definition);
            symbols.push(symbol);
        }
        fixture.symbols = SymbolEnv::new(fixture.module.clone(), indexes);
        fixture.input.candidates = symbols
            .into_iter()
            .take(supplied)
            .enumerate()
            .map(|(ordinal, symbol)| SourceFunctorCandidateInput {
                application: SourceFunctorApplicationId::new(0),
                ordinal,
                symbol,
                contribution,
            })
            .collect();
        fixture.input.type_requests = (0..supplied)
            .map(|ordinal| SourceFunctorTypeRequestInput {
                application: SourceFunctorApplicationId::new(0),
                candidate: Some(SourceFunctorCandidateId::new(ordinal)),
                request_ordinal: ordinal,
                kind: SourceFunctorTypeRequestKind::CandidateSignature,
            })
            .chain(std::iter::once(SourceFunctorTypeRequestInput {
                application: SourceFunctorApplicationId::new(0),
                candidate: None,
                request_ordinal: supplied,
                kind: SourceFunctorTypeRequestKind::ApplicationResultType,
            }))
            .collect();
    }

    fn arena_nodes(
        source: SourceId,
        wrapper: bool,
        recovery: SourceFunctorApplicationRecovery,
    ) -> TypedArena {
        let node_recovery = match recovery {
            SourceFunctorApplicationRecovery::Normal => NodeRecoveryState::Normal,
            SourceFunctorApplicationRecovery::Degraded => NodeRecoveryState::Degraded,
        };
        let mut nodes = vec![
            TypedNode::new(
                "source.term.numeral",
                SourceAnchor::Range(range(source, 14, 15)),
            ),
            TypedNode::new(
                "source.term.functor-application.symbolic",
                SourceAnchor::Range(range(source, 10, 18)),
            )
            .with_recovery(node_recovery),
            TypedNode::new(
                "source.term.functor-head.single",
                SourceAnchor::Range(range(source, 10, 11)),
            )
            .with_recovery(node_recovery),
        ];
        if wrapper {
            nodes.push(
                TypedNode::new(
                    "source.term.functor-application.parenthesized",
                    SourceAnchor::Range(range(source, 9, 19)),
                )
                .with_recovery(node_recovery),
            );
        }
        TypedArena::try_new(None, nodes).expect("typed arena")
    }

    fn primary_handoff(
        source: SourceId,
        module: &ModuleId,
        bindings: &BindingEnv,
        arena: &TypedArena,
        spelling: &str,
    ) -> SourcePrimaryTermHandoff {
        SourcePrimaryTermProducer::build(
            SourcePrimaryTermHandoffInput {
                source_id: source,
                module_id: module.clone(),
                terms: vec![SourcePrimaryTermInput {
                    site: node(0),
                    source_range: range(source, 14, 15),
                    source_ordinal: 0,
                    context: BindingContextId::new(0),
                    recovery: SourcePrimaryTermRecovery::Normal,
                    spelling: spelling.to_owned(),
                    kind: SourcePrimaryTermKind::Numeral,
                    role: SourcePrimaryTermRole::Value,
                    parent: None,
                }],
                references: Vec::new(),
                numeric_type_requests: vec![SourceNumericTypeRequestInput {
                    term: SourcePrimaryTermId::new(0),
                    owner: node(0),
                    source_range: range(source, 14, 15),
                    spelling: spelling.to_owned(),
                    request_ordinal: 0,
                }],
            },
            bindings,
            arena,
        )
        .expect("primary handoff")
    }

    fn fixture_with(options: CandidateOptions) -> Fixture {
        let source = source_id("c3");
        let module = module("source.application");
        let bindings = binding_env(source, &module);
        let arena = arena_nodes(source, false, SourceFunctorApplicationRecovery::Normal);
        let primary = primary_handoff(source, &module, &bindings, &arena, "1");
        let (symbols, symbol, contribution) = local_symbols(source, &module, options);
        let input = SourceFunctorApplicationHandoffInput {
            source_id: source,
            module_id: module.clone(),
            applications: vec![SourceFunctorApplicationInput {
                site: node(1),
                source_range: range(source, 10, 18),
                source_ordinal: 0,
                context: BindingContextId::new(0),
                recovery: SourceFunctorApplicationRecovery::Normal,
                spelling: "f ( 1 )".to_owned(),
                kind: SourceFunctorApplicationKind::Symbolic,
                form: SourceFunctorApplicationForm::Functional,
                head_ordinal: 0,
                head: SourceFunctorHeadSite::Single {
                    site: node(2),
                    source_range: range(source, 10, 11),
                    spelling: "f".to_owned(),
                },
            }],
            wrappers: Vec::new(),
            candidates: vec![SourceFunctorCandidateInput {
                application: SourceFunctorApplicationId::new(0),
                ordinal: 0,
                symbol,
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
        };
        Fixture {
            source,
            module,
            input,
            symbols,
            bindings,
            primary,
            arena,
        }
    }

    fn fixture() -> Fixture {
        fixture_with(CandidateOptions::default())
    }

    fn build(
        fixture: &Fixture,
    ) -> Result<SourceFunctorApplicationHandoff, SourceFunctorApplicationError> {
        SourceFunctorApplicationProducer::build(
            fixture.input.clone(),
            &fixture.symbols,
            &fixture.bindings,
            &fixture.primary,
            &fixture.arena,
        )
    }

    fn shape_fixture(
        form: SourceFunctorApplicationForm,
        kind: SourceFunctorApplicationKind,
        argument_count: usize,
    ) -> Fixture {
        let source = source_id("c4");
        let module = module("source.application.shapes");
        let bindings = binding_env(source, &module);
        let argument_spellings = (1..=argument_count)
            .map(|ordinal| ordinal.to_string())
            .collect::<Vec<_>>();
        let argument_ranges = match form {
            SourceFunctorApplicationForm::Postfix => vec![(10, 11)],
            SourceFunctorApplicationForm::Infix => vec![(10, 11), (14, 15)],
            SourceFunctorApplicationForm::Bracket => (0..argument_count)
                .map(|ordinal| (12 + ordinal * 2, 13 + ordinal * 2))
                .collect(),
            SourceFunctorApplicationForm::Bare => Vec::new(),
            SourceFunctorApplicationForm::Prefix | SourceFunctorApplicationForm::Functional => (0
                ..argument_count)
                .map(|ordinal| (14 + ordinal * 3, 15 + ordinal * 3))
                .collect(),
        };
        let (application_range, application_spelling, head_ranges, head_spellings) = match form {
            SourceFunctorApplicationForm::Bare => {
                ((10, 11), "f".to_owned(), vec![(10, 11)], vec!["f"])
            }
            SourceFunctorApplicationForm::Prefix => (
                (10, argument_ranges[0].1),
                format!("f {}", argument_spellings[0]),
                vec![(10, 11)],
                vec!["f"],
            ),
            SourceFunctorApplicationForm::Infix => (
                (10, 15),
                format!("{} f {}", argument_spellings[0], argument_spellings[1]),
                vec![(12, 13)],
                vec!["f"],
            ),
            SourceFunctorApplicationForm::Postfix => (
                (10, 15),
                format!("{} f", argument_spellings[0]),
                vec![(14, 15)],
                vec!["f"],
            ),
            SourceFunctorApplicationForm::Bracket => (
                (10, 13 + argument_count * 2),
                format!("[ {} ]", argument_spellings.join(" , ")),
                vec![(10, 11), (12 + argument_count * 2, 13 + argument_count * 2)],
                vec!["[", "]"],
            ),
            SourceFunctorApplicationForm::Functional => {
                let spelling = if argument_spellings.is_empty() {
                    "f ( )".to_owned()
                } else {
                    format!("f ( {} )", argument_spellings.join(" , "))
                };
                (
                    (
                        10,
                        argument_ranges
                            .last()
                            .map_or(13, |(_, end)| end.saturating_add(2)),
                    ),
                    spelling,
                    vec![(10, 11)],
                    vec!["f"],
                )
            }
        };
        let mut nodes = argument_ranges
            .iter()
            .map(|(start, end)| {
                TypedNode::new(
                    "source.term.numeral",
                    SourceAnchor::Range(range(source, *start, *end)),
                )
            })
            .collect::<Vec<_>>();
        let application_site = nodes.len();
        nodes.push(TypedNode::new(
            application_kind_node_key(kind),
            SourceAnchor::Range(range(source, application_range.0, application_range.1)),
        ));
        let head_start = nodes.len();
        for (head_range, _) in head_ranges.iter().zip(&head_spellings) {
            nodes.push(TypedNode::new(
                if form == SourceFunctorApplicationForm::Bracket {
                    "source.term.functor-head.bracket"
                } else {
                    "source.term.functor-head.single"
                },
                SourceAnchor::Range(range(source, head_range.0, head_range.1)),
            ));
        }
        let arena = TypedArena::try_new(None, nodes).unwrap();
        let primary = SourcePrimaryTermProducer::build(
            SourcePrimaryTermHandoffInput {
                source_id: source,
                module_id: module.clone(),
                terms: argument_ranges
                    .iter()
                    .enumerate()
                    .map(|(ordinal, (start, end))| SourcePrimaryTermInput {
                        site: node(ordinal),
                        source_range: range(source, *start, *end),
                        source_ordinal: ordinal,
                        context: BindingContextId::new(0),
                        recovery: SourcePrimaryTermRecovery::Normal,
                        spelling: argument_spellings[ordinal].clone(),
                        kind: SourcePrimaryTermKind::Numeral,
                        role: SourcePrimaryTermRole::Value,
                        parent: None,
                    })
                    .collect(),
                references: Vec::new(),
                numeric_type_requests: argument_ranges
                    .iter()
                    .enumerate()
                    .map(|(ordinal, (start, end))| SourceNumericTypeRequestInput {
                        term: SourcePrimaryTermId::new(ordinal),
                        owner: node(ordinal),
                        source_range: range(source, *start, *end),
                        spelling: argument_spellings[ordinal].clone(),
                        request_ordinal: ordinal,
                    })
                    .collect(),
            },
            &bindings,
            &arena,
        )
        .unwrap();
        let (symbols, symbol, contribution) =
            local_symbols(source, &module, CandidateOptions::default());
        let application = SourceFunctorApplicationId::new(0);
        let head = if form == SourceFunctorApplicationForm::Bracket {
            SourceFunctorHeadSite::Paired {
                left_site: node(head_start),
                left_range: range(source, head_ranges[0].0, head_ranges[0].1),
                left_spelling: head_spellings[0].to_owned(),
                right_site: node(head_start + 1),
                right_range: range(source, head_ranges[1].0, head_ranges[1].1),
                right_spelling: head_spellings[1].to_owned(),
            }
        } else {
            SourceFunctorHeadSite::Single {
                site: node(head_start),
                source_range: range(source, head_ranges[0].0, head_ranges[0].1),
                spelling: head_spellings[0].to_owned(),
            }
        };
        let symbolic = kind == SourceFunctorApplicationKind::Symbolic;
        Fixture {
            source,
            module: module.clone(),
            input: SourceFunctorApplicationHandoffInput {
                source_id: source,
                module_id: module,
                applications: vec![SourceFunctorApplicationInput {
                    site: node(application_site),
                    source_range: range(source, application_range.0, application_range.1),
                    source_ordinal: 0,
                    context: BindingContextId::new(0),
                    recovery: SourceFunctorApplicationRecovery::Normal,
                    spelling: application_spelling,
                    kind,
                    form,
                    head_ordinal: match form {
                        SourceFunctorApplicationForm::Infix => 1,
                        SourceFunctorApplicationForm::Postfix => argument_count,
                        _ => 0,
                    },
                    head,
                }],
                wrappers: Vec::new(),
                candidates: symbolic
                    .then_some(SourceFunctorCandidateInput {
                        application,
                        ordinal: 0,
                        symbol,
                        contribution,
                    })
                    .into_iter()
                    .collect(),
                arguments: (0..argument_count)
                    .map(|ordinal| SourceFunctorArgumentInput {
                        application,
                        ordinal,
                        target: SourceFunctorArgumentTarget::Primary(SourcePrimaryTermId::new(
                            ordinal,
                        )),
                    })
                    .collect(),
                type_requests: if symbolic {
                    vec![
                        SourceFunctorTypeRequestInput {
                            application,
                            candidate: Some(SourceFunctorCandidateId::new(0)),
                            request_ordinal: 0,
                            kind: SourceFunctorTypeRequestKind::CandidateSignature,
                        },
                        SourceFunctorTypeRequestInput {
                            application,
                            candidate: None,
                            request_ordinal: 1,
                            kind: SourceFunctorTypeRequestKind::ApplicationResultType,
                        },
                    ]
                } else {
                    Vec::new()
                },
            },
            symbols,
            bindings,
            primary,
            arena,
        }
    }

    fn nested_two_actual_fixture() -> Fixture {
        let source = source_id("c5");
        let module = module("source.application.nested");
        let scope = LocalTermScope::new(vec![0]);
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
            owner: BindingContextOwner::Generated("definition".to_owned()),
            parent: Some(BindingContextId::new(0)),
            layer: BindingContextLayer::Expression,
            lexical_scope: Some(scope.clone()),
            bindings: vec![crate::binding_env::BindingId::new(0)],
            visible_bindings: vec![crate::binding_env::BindingId::new(0)],
            recovery: BindingContextRecovery::Normal,
        });
        let declaration_range = range(source, 5, 6);
        let mut binding_rows = BindingTable::new();
        binding_rows.insert(BindingDraft {
            spelling: "x".to_owned(),
            kind: BindingKind::DefinitionParameter,
            identity: BinderIdentity::ResolverLocal {
                scope,
                ordinal: 0,
                declaration_range,
            },
            owner_context: BindingContextId::new(1),
            declaration_range,
            visible_after_ordinal: 0,
            type_site: BindingTypeSite::Missing,
            status: BindingStatus::Active,
            captured: CapturedFreeVariables::default(),
            diagnostics: Vec::new(),
            recovery: BindingRecoveryState::Normal,
        });
        let bindings = BindingEnv::try_new(BindingEnvParts {
            source_id: source,
            module_id: module.clone(),
            contexts,
            bindings: binding_rows,
            diagnostics: BindingDiagnosticTable::new(),
        })
        .unwrap();
        let arena = TypedArena::try_new(
            None,
            vec![
                TypedNode::new(
                    "source.term.numeral",
                    SourceAnchor::Range(range(source, 18, 19)),
                ),
                TypedNode::new(
                    "source.term.variable-reference",
                    SourceAnchor::Range(range(source, 24, 25)),
                ),
                TypedNode::new(
                    "source.term.functor-application.symbolic",
                    SourceAnchor::Range(range(source, 10, 28)),
                ),
                TypedNode::new(
                    "source.term.functor-head.single",
                    SourceAnchor::Range(range(source, 10, 11)),
                ),
                TypedNode::new(
                    "source.term.functor-application.symbolic",
                    SourceAnchor::Range(range(source, 14, 22)),
                ),
                TypedNode::new(
                    "source.term.functor-head.single",
                    SourceAnchor::Range(range(source, 14, 15)),
                ),
            ],
        )
        .unwrap();
        let primary = SourcePrimaryTermProducer::build(
            SourcePrimaryTermHandoffInput {
                source_id: source,
                module_id: module.clone(),
                terms: vec![
                    SourcePrimaryTermInput {
                        site: node(0),
                        source_range: range(source, 18, 19),
                        source_ordinal: 0,
                        context: BindingContextId::new(1),
                        recovery: SourcePrimaryTermRecovery::Normal,
                        spelling: "1".to_owned(),
                        kind: SourcePrimaryTermKind::Numeral,
                        role: SourcePrimaryTermRole::Value,
                        parent: None,
                    },
                    SourcePrimaryTermInput {
                        site: node(1),
                        source_range: range(source, 24, 25),
                        source_ordinal: 1,
                        context: BindingContextId::new(1),
                        recovery: SourcePrimaryTermRecovery::Normal,
                        spelling: "x".to_owned(),
                        kind: SourcePrimaryTermKind::VariableReference,
                        role: SourcePrimaryTermRole::Value,
                        parent: None,
                    },
                ],
                references: vec![SourcePrimaryTermReferenceInput {
                    term: SourcePrimaryTermId::new(1),
                    binding: crate::binding_env::BindingId::new(0),
                    role: SourcePrimaryTermReferenceRole::Variable,
                }],
                numeric_type_requests: vec![SourceNumericTypeRequestInput {
                    term: SourcePrimaryTermId::new(0),
                    owner: node(0),
                    source_range: range(source, 18, 19),
                    spelling: "1".to_owned(),
                    request_ordinal: 0,
                }],
            },
            &bindings,
            &arena,
        )
        .unwrap();
        let (symbols, symbol, contribution) =
            local_symbols(source, &module, CandidateOptions::default());
        Fixture {
            source,
            module: module.clone(),
            input: SourceFunctorApplicationHandoffInput {
                source_id: source,
                module_id: module,
                applications: vec![
                    SourceFunctorApplicationInput {
                        site: node(2),
                        source_range: range(source, 10, 28),
                        source_ordinal: 0,
                        context: BindingContextId::new(1),
                        recovery: SourceFunctorApplicationRecovery::Normal,
                        spelling: "f ( g ( 1 ) , x )".to_owned(),
                        kind: SourceFunctorApplicationKind::Symbolic,
                        form: SourceFunctorApplicationForm::Functional,
                        head_ordinal: 0,
                        head: SourceFunctorHeadSite::Single {
                            site: node(3),
                            source_range: range(source, 10, 11),
                            spelling: "f".to_owned(),
                        },
                    },
                    SourceFunctorApplicationInput {
                        site: node(4),
                        source_range: range(source, 14, 22),
                        source_ordinal: 1,
                        context: BindingContextId::new(1),
                        recovery: SourceFunctorApplicationRecovery::Normal,
                        spelling: "g ( 1 )".to_owned(),
                        kind: SourceFunctorApplicationKind::Symbolic,
                        form: SourceFunctorApplicationForm::Functional,
                        head_ordinal: 0,
                        head: SourceFunctorHeadSite::Single {
                            site: node(5),
                            source_range: range(source, 14, 15),
                            spelling: "g".to_owned(),
                        },
                    },
                ],
                wrappers: Vec::new(),
                candidates: vec![
                    SourceFunctorCandidateInput {
                        application: SourceFunctorApplicationId::new(0),
                        ordinal: 0,
                        symbol: symbol.clone(),
                        contribution,
                    },
                    SourceFunctorCandidateInput {
                        application: SourceFunctorApplicationId::new(1),
                        ordinal: 0,
                        symbol,
                        contribution,
                    },
                ],
                arguments: vec![
                    SourceFunctorArgumentInput {
                        application: SourceFunctorApplicationId::new(0),
                        ordinal: 0,
                        target: SourceFunctorArgumentTarget::Application(
                            SourceFunctorApplicationId::new(1),
                        ),
                    },
                    SourceFunctorArgumentInput {
                        application: SourceFunctorApplicationId::new(0),
                        ordinal: 1,
                        target: SourceFunctorArgumentTarget::Primary(SourcePrimaryTermId::new(1)),
                    },
                    SourceFunctorArgumentInput {
                        application: SourceFunctorApplicationId::new(1),
                        ordinal: 0,
                        target: SourceFunctorArgumentTarget::Primary(SourcePrimaryTermId::new(0)),
                    },
                ],
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
                    SourceFunctorTypeRequestInput {
                        application: SourceFunctorApplicationId::new(1),
                        candidate: Some(SourceFunctorCandidateId::new(1)),
                        request_ordinal: 0,
                        kind: SourceFunctorTypeRequestKind::CandidateSignature,
                    },
                    SourceFunctorTypeRequestInput {
                        application: SourceFunctorApplicationId::new(1),
                        candidate: None,
                        request_ordinal: 1,
                        kind: SourceFunctorTypeRequestKind::ApplicationResultType,
                    },
                ],
            },
            symbols,
            bindings,
            primary,
            arena,
        }
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

    fn assemble_empty(typed_ast: &TypedAst) -> ResolvedTypedAst {
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
            typed_ast,
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
        .expect("empty resolved handoff")
    }

    #[test]
    fn dense_tables_accessors_authenticated_candidate_and_debug_are_deterministic() {
        let fixture = fixture_with(CandidateOptions {
            symbol_signature: Some(SignatureShell::Opaque {
                schema: "test".to_owned(),
                payload: "opaque".to_owned(),
            }),
            definition_signature: Some(SignatureShell::Opaque {
                schema: "test".to_owned(),
                payload: "opaque".to_owned(),
            }),
            ..CandidateOptions::default()
        });
        let handoff = build(&fixture).expect("valid source application");
        assert_eq!(handoff.source_id(), fixture.source);
        assert_eq!(handoff.module_id(), &fixture.module);
        assert_eq!(handoff.applications().len(), 1);
        assert!(handoff.wrappers().is_empty());
        assert_eq!(handoff.candidates().len(), 1);
        assert_eq!(handoff.arguments().len(), 1);
        assert_eq!(handoff.type_requests().len(), 2);
        let application = handoff
            .applications()
            .get(SourceFunctorApplicationId::new(0))
            .expect("application");
        assert_eq!(application.site(), &node(1));
        assert_eq!(application.source_range(), range(fixture.source, 10, 18));
        assert_eq!(application.source_ordinal(), 0);
        assert_eq!(application.context(), BindingContextId::new(0));
        assert_eq!(
            application.recovery(),
            SourceFunctorApplicationRecovery::Normal
        );
        assert_eq!(application.spelling(), "f ( 1 )");
        assert_eq!(application.kind(), SourceFunctorApplicationKind::Symbolic);
        assert_eq!(application.form(), SourceFunctorApplicationForm::Functional);
        assert_eq!(application.head_ordinal(), 0);
        let candidate = handoff
            .candidates()
            .get(SourceFunctorCandidateId::new(0))
            .expect("candidate");
        assert_eq!(candidate.application(), SourceFunctorApplicationId::new(0));
        assert_eq!(candidate.ordinal(), 0);
        assert_eq!(
            candidate.signature(),
            Some(&SignatureShell::Opaque {
                schema: "test".to_owned(),
                payload: "opaque".to_owned()
            })
        );
        assert_eq!(candidate.visibility(), Visibility::Private);
        assert_eq!(candidate.export_status(), ExportStatus::LocalOnly);
        assert_eq!(
            handoff.primary_term_fingerprint(),
            fixture.primary.debug_text()
        );
        assert_eq!(handoff.debug_text(), build(&fixture).unwrap().debug_text());
        assert!(handoff.debug_text().contains("candidate#0 application=0"));
    }

    #[test]
    fn all_six_symbolic_forms_and_three_actual_functional_are_positive() {
        for (form, arguments) in [
            (SourceFunctorApplicationForm::Bare, 0),
            (SourceFunctorApplicationForm::Prefix, 1),
            (SourceFunctorApplicationForm::Infix, 2),
            (SourceFunctorApplicationForm::Postfix, 1),
            (SourceFunctorApplicationForm::Bracket, 2),
            (SourceFunctorApplicationForm::Functional, 3),
        ] {
            let fixture = shape_fixture(form, SourceFunctorApplicationKind::Symbolic, arguments);
            let handoff = build(&fixture).unwrap_or_else(|error| {
                panic!("{form:?} with {arguments} actuals failed: {error}")
            });
            assert_eq!(handoff.applications().len(), 1);
            assert_eq!(handoff.arguments().len(), arguments);
            assert_eq!(handoff.candidates().len(), 1);
            assert_eq!(handoff.type_requests().len(), 2);
            assert_eq!(
                handoff
                    .applications()
                    .get(SourceFunctorApplicationId::new(0))
                    .unwrap()
                    .form(),
                form
            );
        }
    }

    #[test]
    fn inline_functional_zero_one_two_actual_schema_has_no_candidate_or_requests() {
        for arguments in 0..=2 {
            let fixture = shape_fixture(
                SourceFunctorApplicationForm::Functional,
                SourceFunctorApplicationKind::Inline,
                arguments,
            );
            let handoff = build(&fixture)
                .unwrap_or_else(|error| panic!("inline {arguments} actuals failed: {error}"));
            assert_eq!(handoff.arguments().len(), arguments);
            assert!(handoff.candidates().is_empty());
            assert!(handoff.type_requests().is_empty());
            assert_eq!(
                handoff
                    .applications()
                    .get(SourceFunctorApplicationId::new(0))
                    .unwrap()
                    .kind(),
                SourceFunctorApplicationKind::Inline
            );
        }
    }

    #[test]
    fn empty_excluded_transaction_and_inline_zero_actual_schema_are_valid() {
        let mut empty = fixture();
        empty.input.applications.clear();
        empty.input.candidates.clear();
        empty.input.arguments.clear();
        empty.input.type_requests.clear();
        let handoff = build(&empty).expect("whole-subtree exclusion");
        assert!(handoff.applications().is_empty());
        assert!(handoff.debug_text().contains("primary-term-fingerprint"));

        let mut inline = fixture();
        inline.input.applications[0].kind = SourceFunctorApplicationKind::Inline;
        inline.input.applications[0].spelling = "f ( )".to_owned();
        inline.input.applications[0].source_range.end = 13;
        inline.input.candidates.clear();
        inline.input.arguments.clear();
        inline.input.type_requests.clear();
        let mut nodes = inline
            .arena
            .iter()
            .map(|(_, node)| node.clone())
            .collect::<Vec<_>>();
        nodes[1].kind = "source.term.functor-application.inline".into();
        nodes[1].anchor = SourceAnchor::Range(range(inline.source, 10, 13));
        inline.arena = TypedArena::try_new(None, nodes).unwrap();
        build(&inline).expect("inline zero-actual schema");

        inline.input.applications[0].form = SourceFunctorApplicationForm::Bare;
        assert!(matches!(
            build(&inline),
            Err(SourceFunctorApplicationError::InvalidForm { .. })
        ));
    }

    #[test]
    fn application_head_arena_context_and_spelling_corruptions_are_atomic() {
        let mut cases = Vec::new();
        let mut ordinal = fixture();
        ordinal.input.applications[0].source_ordinal = 1;
        cases.push(ordinal);
        let mut context = fixture();
        context.input.applications[0].context = BindingContextId::new(1);
        cases.push(context);
        let mut wrong_range = fixture();
        wrong_range.input.applications[0].source_range.source_id = other_source_id("d4");
        cases.push(wrong_range);
        let mut spelling = fixture();
        spelling.input.applications[0].spelling = "f(1)".to_owned();
        cases.push(spelling);
        let mut head_kind = fixture();
        head_kind.input.applications[0].head = SourceFunctorHeadSite::Paired {
            left_site: node(2),
            left_range: range(head_kind.source, 10, 11),
            left_spelling: "f".to_owned(),
            right_site: node(2),
            right_range: range(head_kind.source, 16, 17),
            right_spelling: "]".to_owned(),
        };
        cases.push(head_kind);
        let mut role_site = fixture();
        role_site.input.applications[0].site = TypedSiteRef::Role {
            node: TypedNodeId::new(1),
            role: "application".into(),
        };
        cases.push(role_site);
        for corrupt in cases {
            assert!(build(&corrupt).is_err());
        }

        let mut recovery = fixture();
        recovery.input.applications[0].recovery = SourceFunctorApplicationRecovery::Degraded;
        assert!(matches!(
            build(&recovery),
            Err(SourceFunctorApplicationError::InvalidApplication { .. })
        ));
        let mut duplicate_site = fixture();
        if let SourceFunctorHeadSite::Single { site, .. } =
            &mut duplicate_site.input.applications[0].head
        {
            *site = node(1);
        }
        assert_eq!(
            build(&duplicate_site),
            Err(SourceFunctorApplicationError::InvalidHead {
                application: SourceFunctorApplicationId::new(0)
            })
        );

        for (node_index, kind) in [
            (1, "source.term.functor-head.single"),
            (2, "source.term.functor-application.symbolic"),
        ] {
            let mut corrupt = fixture();
            let mut nodes = corrupt
                .arena
                .iter()
                .map(|(_, node)| node.clone())
                .collect::<Vec<_>>();
            nodes[node_index].kind = kind.into();
            corrupt.arena = TypedArena::try_new(None, nodes).unwrap();
            assert!(build(&corrupt).is_err());
        }
        for node_index in [1, 2] {
            let mut corrupt = fixture();
            let mut nodes = corrupt
                .arena
                .iter()
                .map(|(_, node)| node.clone())
                .collect::<Vec<_>>();
            nodes[node_index].anchor = SourceAnchor::Range(range(corrupt.source, 1, 2));
            corrupt.arena = TypedArena::try_new(None, nodes).unwrap();
            assert!(build(&corrupt).is_err());
        }
    }

    #[test]
    fn transaction_source_module_and_task252_context_range_are_exact() {
        let mut source = fixture();
        source.input.source_id = other_source_id("c9");
        assert_eq!(
            build(&source),
            Err(SourceFunctorApplicationError::EnvironmentMismatch)
        );
        let mut module_identity = fixture();
        module_identity.input.module_id = module("wrong.application");
        assert_eq!(
            build(&module_identity),
            Err(SourceFunctorApplicationError::EnvironmentMismatch)
        );

        let mut context = fixture();
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
            owner: BindingContextOwner::Generated("other".to_owned()),
            parent: Some(BindingContextId::new(0)),
            layer: BindingContextLayer::Expression,
            lexical_scope: None,
            bindings: Vec::new(),
            visible_bindings: Vec::new(),
            recovery: BindingContextRecovery::Normal,
        });
        context.bindings = BindingEnv::try_new(BindingEnvParts {
            source_id: context.source,
            module_id: context.module.clone(),
            contexts,
            bindings: BindingTable::new(),
            diagnostics: BindingDiagnosticTable::new(),
        })
        .unwrap();
        context.primary = SourcePrimaryTermProducer::build(
            SourcePrimaryTermHandoffInput {
                source_id: context.source,
                module_id: context.module.clone(),
                terms: vec![SourcePrimaryTermInput {
                    site: node(0),
                    source_range: range(context.source, 14, 15),
                    source_ordinal: 0,
                    context: BindingContextId::new(1),
                    recovery: SourcePrimaryTermRecovery::Normal,
                    spelling: "1".to_owned(),
                    kind: SourcePrimaryTermKind::Numeral,
                    role: SourcePrimaryTermRole::Value,
                    parent: None,
                }],
                references: Vec::new(),
                numeric_type_requests: vec![SourceNumericTypeRequestInput {
                    term: SourcePrimaryTermId::new(0),
                    owner: node(0),
                    source_range: range(context.source, 14, 15),
                    spelling: "1".to_owned(),
                    request_ordinal: 0,
                }],
            },
            &context.bindings,
            &context.arena,
        )
        .unwrap();
        assert!(matches!(
            build(&context),
            Err(SourceFunctorApplicationError::DuplicateArgumentTarget { .. })
        ));

        let mut outside = fixture();
        let mut nodes = outside
            .arena
            .iter()
            .map(|(_, node)| node.clone())
            .collect::<Vec<_>>();
        nodes[0].anchor = SourceAnchor::Range(range(outside.source, 30, 31));
        outside.arena = TypedArena::try_new(None, nodes).unwrap();
        outside.primary = SourcePrimaryTermProducer::build(
            SourcePrimaryTermHandoffInput {
                source_id: outside.source,
                module_id: outside.module.clone(),
                terms: vec![SourcePrimaryTermInput {
                    site: node(0),
                    source_range: range(outside.source, 30, 31),
                    source_ordinal: 0,
                    context: BindingContextId::new(0),
                    recovery: SourcePrimaryTermRecovery::Normal,
                    spelling: "1".to_owned(),
                    kind: SourcePrimaryTermKind::Numeral,
                    role: SourcePrimaryTermRole::Value,
                    parent: None,
                }],
                references: Vec::new(),
                numeric_type_requests: vec![SourceNumericTypeRequestInput {
                    term: SourcePrimaryTermId::new(0),
                    owner: node(0),
                    source_range: range(outside.source, 30, 31),
                    spelling: "1".to_owned(),
                    request_ordinal: 0,
                }],
            },
            &outside.bindings,
            &outside.arena,
        )
        .unwrap();
        assert!(matches!(
            build(&outside),
            Err(SourceFunctorApplicationError::InvalidArgument { .. })
        ));
    }

    #[test]
    fn transparent_wrapper_is_outer_owned_dense_and_exactly_spelled() {
        let mut fixture = fixture();
        fixture.arena = arena_nodes(
            fixture.source,
            true,
            SourceFunctorApplicationRecovery::Normal,
        );
        fixture.input.wrappers.push(SourceFunctorWrapperInput {
            application: SourceFunctorApplicationId::new(0),
            ordinal: 0,
            site: node(3),
            source_range: range(fixture.source, 9, 19),
            context: BindingContextId::new(0),
            spelling: "( f ( 1 ) )".to_owned(),
            recovery: SourceFunctorApplicationRecovery::Normal,
        });
        let handoff = build(&fixture).expect("transparent wrapper");
        let wrapper = handoff
            .wrappers()
            .get(SourceFunctorWrapperId::new(0))
            .expect("wrapper");
        assert_eq!(wrapper.application(), SourceFunctorApplicationId::new(0));
        assert_eq!(wrapper.ordinal(), 0);
        assert_eq!(wrapper.site(), &node(3));
        assert_eq!(wrapper.context(), BindingContextId::new(0));
        assert_eq!(wrapper.spelling(), "( f ( 1 ) )");

        let mut ordinal = fixture.clone();
        ordinal.input.wrappers[0].ordinal = 1;
        assert!(matches!(
            build(&ordinal),
            Err(SourceFunctorApplicationError::ReorderedWrapper { .. })
        ));
        let mut spelling = fixture.clone();
        spelling.input.wrappers[0].spelling = "( f ( 1 )  )".to_owned();
        assert!(matches!(
            build(&spelling),
            Err(SourceFunctorApplicationError::InvalidWrapper { .. })
        ));
        let mut detached = fixture;
        detached.input.wrappers[0].source_range.start = 10;
        assert!(matches!(
            build(&detached),
            Err(SourceFunctorApplicationError::InvalidWrapper { .. })
        ));
    }

    #[test]
    fn multiple_transparent_wrappers_are_outer_to_inner_and_fail_closed() {
        let mut fixture = fixture();
        let mut nodes = fixture
            .arena
            .iter()
            .map(|(_, node)| node.clone())
            .collect::<Vec<_>>();
        nodes.push(TypedNode::new(
            "source.term.functor-application.parenthesized",
            SourceAnchor::Range(range(fixture.source, 9, 19)),
        ));
        nodes.push(TypedNode::new(
            "source.term.functor-application.parenthesized",
            SourceAnchor::Range(range(fixture.source, 8, 20)),
        ));
        fixture.arena = TypedArena::try_new(None, nodes).unwrap();
        fixture.input.wrappers = vec![
            SourceFunctorWrapperInput {
                application: SourceFunctorApplicationId::new(0),
                ordinal: 0,
                site: node(4),
                source_range: range(fixture.source, 8, 20),
                context: BindingContextId::new(0),
                spelling: "( ( f ( 1 ) ) )".to_owned(),
                recovery: SourceFunctorApplicationRecovery::Normal,
            },
            SourceFunctorWrapperInput {
                application: SourceFunctorApplicationId::new(0),
                ordinal: 1,
                site: node(3),
                source_range: range(fixture.source, 9, 19),
                context: BindingContextId::new(0),
                spelling: "( f ( 1 ) )".to_owned(),
                recovery: SourceFunctorApplicationRecovery::Normal,
            },
        ];
        let handoff = build(&fixture).expect("((f(1)))");
        assert_eq!(handoff.wrappers().len(), 2);
        assert_eq!(
            handoff
                .wrappers()
                .get(SourceFunctorWrapperId::new(0))
                .unwrap()
                .source_range(),
            range(fixture.source, 8, 20)
        );

        let mut row_order = fixture.clone();
        row_order.input.wrappers.swap(0, 1);
        assert!(matches!(
            build(&row_order),
            Err(SourceFunctorApplicationError::ReorderedWrapper { .. })
        ));
        let mut nesting = fixture.clone();
        nesting.input.wrappers[0].source_range = range(nesting.source, 9, 20);
        assert!(matches!(
            build(&nesting),
            Err(SourceFunctorApplicationError::InvalidWrapper { .. })
        ));
        let mut context = fixture.clone();
        context.input.wrappers[1].context = BindingContextId::new(1);
        assert!(matches!(
            build(&context),
            Err(SourceFunctorApplicationError::InvalidWrapper { .. })
        ));
        let mut recovery = fixture.clone();
        recovery.input.wrappers[0].recovery = SourceFunctorApplicationRecovery::Degraded;
        assert!(matches!(
            build(&recovery),
            Err(SourceFunctorApplicationError::InvalidWrapper { .. })
        ));
        let mut duplicate = fixture.clone();
        duplicate.input.wrappers[1].site = duplicate.input.wrappers[0].site.clone();
        assert!(build(&duplicate).is_err());
        let mut wrong_kind = fixture.clone();
        let mut nodes = wrong_kind
            .arena
            .iter()
            .map(|(_, node)| node.clone())
            .collect::<Vec<_>>();
        nodes[4].kind = "source.term.functor-head.single".into();
        wrong_kind.arena = TypedArena::try_new(None, nodes).unwrap();
        assert!(matches!(
            build(&wrong_kind),
            Err(SourceFunctorApplicationError::InvalidWrapper { .. })
        ));
        let mut wrong_anchor = fixture;
        let mut nodes = wrong_anchor
            .arena
            .iter()
            .map(|(_, node)| node.clone())
            .collect::<Vec<_>>();
        nodes[4].anchor = SourceAnchor::Range(range(wrong_anchor.source, 7, 20));
        wrong_anchor.arena = TypedArena::try_new(None, nodes).unwrap();
        assert!(matches!(
            build(&wrong_anchor),
            Err(SourceFunctorApplicationError::InvalidWrapper { .. })
        ));
    }

    #[test]
    fn degraded_application_head_and_wrapper_are_positive_when_arena_matches() {
        let mut fixture = fixture();
        fixture.arena = arena_nodes(
            fixture.source,
            true,
            SourceFunctorApplicationRecovery::Degraded,
        );
        fixture.input.applications[0].recovery = SourceFunctorApplicationRecovery::Degraded;
        fixture.input.wrappers = vec![SourceFunctorWrapperInput {
            application: SourceFunctorApplicationId::new(0),
            ordinal: 0,
            site: node(3),
            source_range: range(fixture.source, 9, 19),
            context: BindingContextId::new(0),
            spelling: "( f ( 1 ) )".to_owned(),
            recovery: SourceFunctorApplicationRecovery::Degraded,
        }];
        let handoff = build(&fixture).expect("degraded source application");
        assert_eq!(
            handoff
                .applications()
                .get(SourceFunctorApplicationId::new(0))
                .unwrap()
                .recovery(),
            SourceFunctorApplicationRecovery::Degraded
        );
        assert_eq!(
            handoff
                .wrappers()
                .get(SourceFunctorWrapperId::new(0))
                .unwrap()
                .recovery(),
            SourceFunctorApplicationRecovery::Degraded
        );
    }

    #[test]
    fn nested_wrapped_application_uses_wrapper_as_outer_argument_occurrence() {
        let mut fixture = nested_two_actual_fixture();
        let mut nodes = fixture
            .arena
            .iter()
            .map(|(_, node)| node.clone())
            .collect::<Vec<_>>();
        nodes[1].anchor = SourceAnchor::Range(range(fixture.source, 18, 19));
        nodes.push(TypedNode::new(
            "source.term.functor-application.parenthesized",
            SourceAnchor::Range(range(fixture.source, 13, 23)),
        ));
        fixture.arena = TypedArena::try_new(None, nodes).unwrap();
        fixture.primary = SourcePrimaryTermProducer::build(
            SourcePrimaryTermHandoffInput {
                source_id: fixture.source,
                module_id: fixture.module.clone(),
                terms: vec![SourcePrimaryTermInput {
                    site: node(1),
                    source_range: range(fixture.source, 18, 19),
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
                    binding: crate::binding_env::BindingId::new(0),
                    role: SourcePrimaryTermReferenceRole::Variable,
                }],
                numeric_type_requests: Vec::new(),
            },
            &fixture.bindings,
            &fixture.arena,
        )
        .unwrap();
        fixture.input.applications[0].spelling = "f ( ( g ( x ) ) )".to_owned();
        fixture.input.applications[1].spelling = "g ( x )".to_owned();
        fixture.input.arguments = vec![
            SourceFunctorArgumentInput {
                application: SourceFunctorApplicationId::new(0),
                ordinal: 0,
                target: SourceFunctorArgumentTarget::Application(SourceFunctorApplicationId::new(
                    1,
                )),
            },
            SourceFunctorArgumentInput {
                application: SourceFunctorApplicationId::new(1),
                ordinal: 0,
                target: SourceFunctorArgumentTarget::Primary(SourcePrimaryTermId::new(0)),
            },
        ];
        fixture.input.wrappers.push(SourceFunctorWrapperInput {
            application: SourceFunctorApplicationId::new(1),
            ordinal: 0,
            site: node(6),
            source_range: range(fixture.source, 13, 23),
            context: BindingContextId::new(1),
            spelling: "( g ( x ) )".to_owned(),
            recovery: SourceFunctorApplicationRecovery::Normal,
        });
        let handoff = build(&fixture).expect("f((g(x)))");
        assert_eq!(handoff.wrappers().len(), 1);
        assert_eq!(fixture.primary.terms().len(), 1);
        assert_eq!(fixture.primary.references().len(), 1);
        assert_eq!(
            handoff
                .wrappers()
                .get(SourceFunctorWrapperId::new(0))
                .unwrap()
                .application(),
            SourceFunctorApplicationId::new(1)
        );
    }

    #[test]
    fn primary_dependency_root_ownership_and_exact_fingerprint_are_revalidated() {
        let fixture = fixture();
        let handoff = build(&fixture).expect("handoff");
        handoff
            .validate_installation(fixture.source, &fixture.module, &fixture.primary)
            .expect("same dependency");
        let equivalent = fixture.primary.clone();
        handoff
            .validate_installation(fixture.source, &fixture.module, &equivalent)
            .expect("equivalent clone");
        let substituted = primary_handoff(
            fixture.source,
            &fixture.module,
            &fixture.bindings,
            &fixture.arena,
            "2",
        );
        assert_eq!(
            handoff.validate_installation(fixture.source, &fixture.module, &substituted),
            Err(SourceFunctorApplicationError::PrimaryDependencyMismatch)
        );
        assert_eq!(
            handoff.validate_installation(other_source_id("e5"), &fixture.module, &fixture.primary),
            Err(SourceFunctorApplicationError::PrimaryDependencyMismatch)
        );

        let mut duplicate = fixture.clone();
        duplicate
            .input
            .arguments
            .push(duplicate.input.arguments[0].clone());
        duplicate.input.arguments[1].ordinal = 1;
        duplicate.input.applications[0].form = SourceFunctorApplicationForm::Infix;
        duplicate.input.applications[0].head_ordinal = 1;
        assert!(matches!(
            build(&duplicate),
            Err(SourceFunctorApplicationError::DuplicateArgumentTarget { .. })
        ));
        let mut dangling = fixture;
        dangling.input.arguments[0].target =
            SourceFunctorArgumentTarget::Primary(SourcePrimaryTermId::new(10));
        assert!(matches!(
            build(&dangling),
            Err(SourceFunctorApplicationError::InvalidArgument { .. })
        ));
    }

    #[test]
    fn primary_parenthesized_root_is_owned_but_its_inner_descendant_is_rejected() {
        let mut fixture = fixture();
        fixture.arena = TypedArena::try_new(
            None,
            vec![
                TypedNode::new(
                    "source.term.parenthesized",
                    SourceAnchor::Range(range(fixture.source, 13, 16)),
                ),
                TypedNode::new(
                    "source.term.numeral",
                    SourceAnchor::Range(range(fixture.source, 14, 15)),
                ),
                TypedNode::new(
                    "source.term.functor-application.symbolic",
                    SourceAnchor::Range(range(fixture.source, 10, 19)),
                ),
                TypedNode::new(
                    "source.term.functor-head.single",
                    SourceAnchor::Range(range(fixture.source, 10, 11)),
                ),
            ],
        )
        .unwrap();
        fixture.primary = SourcePrimaryTermProducer::build(
            SourcePrimaryTermHandoffInput {
                source_id: fixture.source,
                module_id: fixture.module.clone(),
                terms: vec![
                    SourcePrimaryTermInput {
                        site: node(0),
                        source_range: range(fixture.source, 13, 16),
                        source_ordinal: 0,
                        context: BindingContextId::new(0),
                        recovery: SourcePrimaryTermRecovery::Normal,
                        spelling: "( 1 )".to_owned(),
                        kind: SourcePrimaryTermKind::Parenthesized,
                        role: SourcePrimaryTermRole::Value,
                        parent: None,
                    },
                    SourcePrimaryTermInput {
                        site: node(1),
                        source_range: range(fixture.source, 14, 15),
                        source_ordinal: 1,
                        context: BindingContextId::new(0),
                        recovery: SourcePrimaryTermRecovery::Normal,
                        spelling: "1".to_owned(),
                        kind: SourcePrimaryTermKind::Numeral,
                        role: SourcePrimaryTermRole::Value,
                        parent: Some(SourcePrimaryTermId::new(0)),
                    },
                ],
                references: Vec::new(),
                numeric_type_requests: vec![SourceNumericTypeRequestInput {
                    term: SourcePrimaryTermId::new(1),
                    owner: node(1),
                    source_range: range(fixture.source, 14, 15),
                    spelling: "1".to_owned(),
                    request_ordinal: 0,
                }],
            },
            &fixture.bindings,
            &fixture.arena,
        )
        .unwrap();
        fixture.input.applications[0].site = node(2);
        fixture.input.applications[0].source_range = range(fixture.source, 10, 19);
        fixture.input.applications[0].spelling = "f ( ( 1 ) )".to_owned();
        fixture.input.applications[0].head = SourceFunctorHeadSite::Single {
            site: node(3),
            source_range: range(fixture.source, 10, 11),
            spelling: "f".to_owned(),
        };
        build(&fixture).expect("outer Task-252 parenthesized root");

        fixture.input.arguments[0].target =
            SourceFunctorArgumentTarget::Primary(SourcePrimaryTermId::new(1));
        assert!(matches!(
            build(&fixture),
            Err(SourceFunctorApplicationError::DuplicateArgumentTarget { .. })
        ));
    }

    #[test]
    fn typed_ast_installation_is_task252_ordered_one_shot_and_clone_authenticated() {
        let fixture = fixture();
        let handoff = build(&fixture).expect("application handoff");
        let no_dependency = TypedAst::try_new(empty_typed_parts(&fixture)).unwrap();
        assert_eq!(
            no_dependency.with_source_application(handoff.clone()),
            Err(TypedAstError::InvalidSourceApplication)
        );

        let typed = TypedAst::try_new(empty_typed_parts(&fixture))
            .unwrap()
            .with_source_term(fixture.primary.clone())
            .unwrap()
            .with_source_application(handoff.clone())
            .expect("equivalent Task-252 dependency");
        assert_eq!(typed.source_application(), Some(&handoff));
        assert_eq!(
            typed.clone().with_source_application(handoff.clone()),
            Err(TypedAstError::InvalidSourceApplication)
        );

        let equivalent_primary = fixture.primary.clone();
        TypedAst::try_new(empty_typed_parts(&fixture))
            .unwrap()
            .with_source_term(equivalent_primary)
            .unwrap()
            .with_source_application(handoff.clone())
            .expect("equivalent clone accepted");

        let substituted = primary_handoff(
            fixture.source,
            &fixture.module,
            &fixture.bindings,
            &fixture.arena,
            "2",
        );
        assert_eq!(
            TypedAst::try_new(empty_typed_parts(&fixture))
                .unwrap()
                .with_source_term(substituted)
                .unwrap()
                .with_source_application(handoff.clone()),
            Err(TypedAstError::InvalidSourceApplication)
        );

        let resolved = assemble_empty(&typed);
        assert_eq!(resolved.source_term(), typed.source_term());
        assert_eq!(resolved.source_application(), typed.source_application());
        assert!(resolved.debug_text().contains(&handoff.debug_text()));
    }

    #[test]
    fn candidate_signature_shell_and_cross_index_matrix_is_enforced() {
        for signature in [
            None,
            Some(SignatureShell::Pending),
            Some(SignatureShell::Opaque {
                schema: "schema".to_owned(),
                payload: "payload".to_owned(),
            }),
        ] {
            let fixture = fixture_with(CandidateOptions {
                symbol_signature: signature.clone(),
                definition_signature: signature,
                ..CandidateOptions::default()
            });
            build(&fixture).expect("accepted unresolved signature provenance");
        }

        let malformed = SignatureShell::Malformed {
            class: "bad".to_owned(),
        };
        let malformed = fixture_with(CandidateOptions {
            symbol_signature: Some(malformed.clone()),
            definition_signature: Some(malformed),
            ..CandidateOptions::default()
        });
        assert!(matches!(
            build(&malformed),
            Err(SourceFunctorApplicationError::InvalidCandidate { .. })
        ));
        let wrong_kind = fixture_with(CandidateOptions {
            kind: SymbolKind::Predicate,
            ..CandidateOptions::default()
        });
        assert!(matches!(
            build(&wrong_kind),
            Err(SourceFunctorApplicationError::InvalidCandidate { .. })
        ));
        let recovered = fixture_with(CandidateOptions {
            recovered: true,
            ..CandidateOptions::default()
        });
        assert!(matches!(
            build(&recovered),
            Err(SourceFunctorApplicationError::InvalidCandidate { .. })
        ));
        let conflict = fixture_with(CandidateOptions {
            conflict: Some(DeclarationConflictClass::DuplicateSpelling),
            ..CandidateOptions::default()
        });
        assert!(matches!(
            build(&conflict),
            Err(SourceFunctorApplicationError::InvalidCandidate { .. })
        ));
        let notation = fixture_with(CandidateOptions {
            symbol_notation: Some("left".to_owned()),
            definition_notation: Some("right".to_owned()),
            ..CandidateOptions::default()
        });
        assert!(matches!(
            build(&notation),
            Err(SourceFunctorApplicationError::InvalidCandidate { .. })
        ));
        let signature_drift = fixture_with(CandidateOptions {
            symbol_signature: Some(SignatureShell::Pending),
            definition_signature: None,
            ..CandidateOptions::default()
        });
        assert!(matches!(
            build(&signature_drift),
            Err(SourceFunctorApplicationError::InvalidCandidate { .. })
        ));

        for corrupt in [
            fixture_with(CandidateOptions {
                include_definition: false,
                ..CandidateOptions::default()
            }),
            fixture_with(CandidateOptions {
                definition_kind: DefinitionKind::Predicate,
                ..CandidateOptions::default()
            }),
            fixture_with(CandidateOptions {
                origin_after_use: true,
                ..CandidateOptions::default()
            }),
            fixture_with(CandidateOptions {
                definition_origin_drift: true,
                ..CandidateOptions::default()
            }),
            fixture_with(CandidateOptions {
                definition_contribution_drift: true,
                ..CandidateOptions::default()
            }),
        ] {
            assert!(matches!(
                build(&corrupt),
                Err(SourceFunctorApplicationError::InvalidCandidate { .. })
            ));
        }
    }

    #[test]
    fn imported_visibility_export_reexport_and_import_policy_matrix_is_exact() {
        let fixture = fixture();
        let dependency = module("dependency.functors");
        let symbol = SymbolId::new(
            dependency.clone(),
            LocalSymbolId::new("imported"),
            FullyQualifiedName::new("dependency.functors::f"),
        );
        let contribution = fixture.input.candidates[0].contribution;
        let entry = |visibility, export_status, origin_module: ModuleId| {
            SymbolEntry::new(
                symbol.clone(),
                SymbolKind::Functor,
                NamespacePath::new(fixture.module.path().as_str()),
                "f",
                SemanticOrigin::new(
                    fixture.source,
                    origin_module,
                    SourceAnchor::Range(range(fixture.source, 1, 5)),
                    vec![0],
                ),
                contribution,
            )
            .with_visibility(visibility)
            .with_export_status(export_status)
        };
        for export_status in [ExportStatus::Exported, ExportStatus::ReExported] {
            assert!(valid_imported_candidate_provenance(
                &entry(Visibility::Public, export_status, dependency.clone()),
                &symbol,
                fixture.source,
                range(fixture.source, 10, 18),
                range(fixture.source, 1, 5),
                true,
            ));
        }
        for (visibility, export_status, origin_module, authenticated) in [
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
                fixture.module.clone(),
                true,
            ),
            (
                Visibility::Public,
                ExportStatus::Exported,
                dependency.clone(),
                false,
            ),
        ] {
            assert!(!valid_imported_candidate_provenance(
                &entry(visibility, export_status, origin_module),
                &symbol,
                fixture.source,
                range(fixture.source, 10, 18),
                range(fixture.source, 1, 5),
                authenticated,
            ));
        }
    }

    #[test]
    fn candidate_and_request_dense_association_is_never_sorted_or_repaired() {
        let mut wrong_contribution = fixture();
        wrong_contribution.input.candidates[0].symbol = symbol_id(&wrong_contribution.module, 99);
        assert!(matches!(
            build(&wrong_contribution),
            Err(SourceFunctorApplicationError::InvalidCandidate { .. })
        ));
        let mut candidate_ordinal = fixture();
        candidate_ordinal.input.candidates[0].ordinal = 1;
        assert!(matches!(
            build(&candidate_ordinal),
            Err(SourceFunctorApplicationError::ReorderedCandidate { .. })
        ));
        let mut missing = fixture();
        missing.input.type_requests.pop();
        assert!(matches!(
            build(&missing),
            Err(SourceFunctorApplicationError::InvalidTypeRequest { .. })
        ));
        let mut wrong_candidate = fixture();
        wrong_candidate.input.type_requests[0].candidate = Some(SourceFunctorCandidateId::new(1));
        assert!(matches!(
            build(&wrong_candidate),
            Err(SourceFunctorApplicationError::InvalidTypeRequest { .. })
        ));
        let mut wrong_kind = fixture();
        wrong_kind.input.type_requests[0].kind =
            SourceFunctorTypeRequestKind::ApplicationResultType;
        assert!(matches!(
            build(&wrong_kind),
            Err(SourceFunctorApplicationError::InvalidTypeRequest { .. })
        ));
        let mut reordered = fixture();
        reordered.input.type_requests.swap(0, 1);
        assert!(matches!(
            build(&reordered),
            Err(SourceFunctorApplicationError::InvalidTypeRequest { .. })
        ));
        let mut inline_smuggling = fixture();
        inline_smuggling.input.applications[0].kind = SourceFunctorApplicationKind::Inline;
        let mut nodes = inline_smuggling
            .arena
            .iter()
            .map(|(_, node)| node.clone())
            .collect::<Vec<_>>();
        nodes[1].kind = "source.term.functor-application.inline".into();
        inline_smuggling.arena = TypedArena::try_new(None, nodes).unwrap();
        assert!(matches!(
            build(&inline_smuggling),
            Err(SourceFunctorApplicationError::InvalidCandidate { .. })
        ));
        let mut inline_request = fixture();
        inline_request.input.applications[0].kind = SourceFunctorApplicationKind::Inline;
        inline_request.input.candidates.clear();
        let mut nodes = inline_request
            .arena
            .iter()
            .map(|(_, node)| node.clone())
            .collect::<Vec<_>>();
        nodes[1].kind = "source.term.functor-application.inline".into();
        inline_request.arena = TypedArena::try_new(None, nodes).unwrap();
        assert!(matches!(
            build(&inline_request),
            Err(SourceFunctorApplicationError::InvalidTypeRequest { .. })
        ));
        let mut duplicate_request = fixture();
        duplicate_request
            .input
            .type_requests
            .insert(1, duplicate_request.input.type_requests[0].clone());
        duplicate_request.input.type_requests[1].request_ordinal = 1;
        duplicate_request.input.type_requests[2].request_ordinal = 2;
        assert!(matches!(
            build(&duplicate_request),
            Err(SourceFunctorApplicationError::InvalidTypeRequest { .. })
        ));
    }

    #[test]
    fn individually_authenticated_candidate_subset_does_not_claim_environment_completeness() {
        let mut fixture = fixture();
        install_matching_local_symbols(&mut fixture, 3, 2);
        assert_eq!(fixture.symbols.symbols().len(), 3);
        let handoff = build(&fixture).expect("strict supplied subset");
        assert_eq!(handoff.candidates().len(), 2);
        assert_eq!(handoff.type_requests().len(), 3);
        assert_eq!(
            handoff
                .candidates()
                .iter()
                .map(|(_, candidate)| candidate.ordinal())
                .collect::<Vec<_>>(),
            [0, 1]
        );
    }

    #[test]
    fn form_cardinality_head_position_and_token_spelling_are_exact() {
        let mut prefix = fixture();
        prefix.input.applications[0].form = SourceFunctorApplicationForm::Prefix;
        prefix.input.applications[0].spelling = "f 1".to_owned();
        prefix.input.applications[0].source_range.end = 15;
        let mut nodes = prefix
            .arena
            .iter()
            .map(|(_, node)| node.clone())
            .collect::<Vec<_>>();
        nodes[1].anchor = SourceAnchor::Range(range(prefix.source, 10, 15));
        prefix.arena = TypedArena::try_new(None, nodes).unwrap();
        build(&prefix).expect("prefix form");

        let mut wrong_head_ordinal = prefix.clone();
        wrong_head_ordinal.input.applications[0].head_ordinal = 1;
        assert!(matches!(
            build(&wrong_head_ordinal),
            Err(SourceFunctorApplicationError::InvalidForm { .. })
        ));
        let mut wrong_spelling = prefix.clone();
        wrong_spelling.input.applications[0].spelling = "f, 1".to_owned();
        assert!(matches!(
            build(&wrong_spelling),
            Err(SourceFunctorApplicationError::InvalidForm { .. })
        ));
        let mut bare_with_actual = prefix;
        bare_with_actual.input.applications[0].form = SourceFunctorApplicationForm::Bare;
        assert!(matches!(
            build(&bare_with_actual),
            Err(SourceFunctorApplicationError::InvalidForm { .. })
        ));
    }

    #[test]
    fn nested_application_preorder_single_parent_and_effective_spelling_are_exact() {
        let mut fixture = fixture();
        fixture.input.applications[0].source_range = range(fixture.source, 10, 21);
        fixture.input.applications[0].spelling = "f ( g ( 1 ) )".to_owned();
        fixture
            .input
            .applications
            .push(SourceFunctorApplicationInput {
                site: node(3),
                source_range: range(fixture.source, 12, 18),
                source_ordinal: 1,
                context: BindingContextId::new(0),
                recovery: SourceFunctorApplicationRecovery::Normal,
                spelling: "g ( 1 )".to_owned(),
                kind: SourceFunctorApplicationKind::Symbolic,
                form: SourceFunctorApplicationForm::Functional,
                head_ordinal: 0,
                head: SourceFunctorHeadSite::Single {
                    site: node(4),
                    source_range: range(fixture.source, 12, 13),
                    spelling: "g".to_owned(),
                },
            });
        let candidate = fixture.input.candidates[0].clone();
        fixture.input.candidates.push(SourceFunctorCandidateInput {
            application: SourceFunctorApplicationId::new(1),
            ordinal: 0,
            symbol: candidate.symbol,
            contribution: candidate.contribution,
        });
        fixture.input.arguments = vec![
            SourceFunctorArgumentInput {
                application: SourceFunctorApplicationId::new(0),
                ordinal: 0,
                target: SourceFunctorArgumentTarget::Application(SourceFunctorApplicationId::new(
                    1,
                )),
            },
            SourceFunctorArgumentInput {
                application: SourceFunctorApplicationId::new(1),
                ordinal: 0,
                target: SourceFunctorArgumentTarget::Primary(SourcePrimaryTermId::new(0)),
            },
        ];
        fixture.input.type_requests.extend([
            SourceFunctorTypeRequestInput {
                application: SourceFunctorApplicationId::new(1),
                candidate: Some(SourceFunctorCandidateId::new(1)),
                request_ordinal: 0,
                kind: SourceFunctorTypeRequestKind::CandidateSignature,
            },
            SourceFunctorTypeRequestInput {
                application: SourceFunctorApplicationId::new(1),
                candidate: None,
                request_ordinal: 1,
                kind: SourceFunctorTypeRequestKind::ApplicationResultType,
            },
        ]);
        let mut nodes = fixture
            .arena
            .iter()
            .map(|(_, node)| node.clone())
            .collect::<Vec<_>>();
        nodes[1].anchor = SourceAnchor::Range(range(fixture.source, 10, 21));
        nodes.push(TypedNode::new(
            "source.term.functor-application.symbolic",
            SourceAnchor::Range(range(fixture.source, 12, 18)),
        ));
        nodes.push(TypedNode::new(
            "source.term.functor-head.single",
            SourceAnchor::Range(range(fixture.source, 12, 13)),
        ));
        fixture.arena = TypedArena::try_new(None, nodes).unwrap();
        let handoff = build(&fixture).expect("nested preorder");
        assert_eq!(handoff.applications().len(), 2);
        assert_eq!(
            handoff
                .arguments()
                .get(SourceFunctorArgumentId::new(0))
                .expect("outer edge")
                .target(),
            SourceFunctorArgumentTarget::Application(SourceFunctorApplicationId::new(1))
        );

        let mut orphan = fixture.clone();
        orphan.input.arguments.remove(0);
        orphan.input.arguments[0].ordinal = 0;
        assert!(matches!(
            build(&orphan),
            Err(SourceFunctorApplicationError::InvalidApplication {
                application
            }) if application == SourceFunctorApplicationId::new(1)
        ));
        let mut backwards = fixture.clone();
        backwards.input.arguments[1].target =
            SourceFunctorArgumentTarget::Application(SourceFunctorApplicationId::new(0));
        assert!(matches!(
            build(&backwards),
            Err(SourceFunctorApplicationError::InvalidArgument { .. })
        ));
        let mut duplicate_parent = fixture;
        duplicate_parent
            .input
            .arguments
            .insert(1, duplicate_parent.input.arguments[0].clone());
        duplicate_parent.input.arguments[1].ordinal = 1;
        assert!(matches!(
            build(&duplicate_parent),
            Err(SourceFunctorApplicationError::MultipleParents { .. })
        ));
    }

    #[test]
    fn exact_nested_two_actual_shape_and_multi_application_ordering_are_enforced() {
        let fixture = nested_two_actual_fixture();
        let handoff = build(&fixture).expect("f(g(1), x)");
        assert_eq!(handoff.applications().len(), 2);
        assert_eq!(handoff.arguments().len(), 3);
        assert_eq!(handoff.candidates().len(), 2);
        assert_eq!(handoff.type_requests().len(), 4);
        assert_eq!(fixture.primary.references().len(), 1);
        let reference = fixture
            .primary
            .references()
            .get(crate::source_term::SourcePrimaryTermReferenceId::new(0))
            .unwrap();
        assert_eq!(reference.binding(), crate::binding_env::BindingId::new(0));
        assert_eq!(reference.use_ordinal(), 1);
        assert_eq!(
            fixture
                .bindings
                .bindings()
                .get(reference.binding())
                .unwrap()
                .kind,
            BindingKind::DefinitionParameter
        );

        let mut candidate_group = fixture.clone();
        candidate_group.input.candidates.swap(0, 1);
        assert!(matches!(
            build(&candidate_group),
            Err(SourceFunctorApplicationError::ReorderedCandidate { .. })
        ));
        let mut duplicate_candidate = fixture.clone();
        let mut duplicate = duplicate_candidate.input.candidates[0].clone();
        duplicate.ordinal = 1;
        duplicate_candidate.input.candidates.insert(1, duplicate);
        assert!(matches!(
            build(&duplicate_candidate),
            Err(SourceFunctorApplicationError::ReorderedCandidate { .. })
        ));
        let mut cross_request = fixture.clone();
        cross_request.input.type_requests[2].candidate = Some(SourceFunctorCandidateId::new(0));
        assert!(matches!(
            build(&cross_request),
            Err(SourceFunctorApplicationError::InvalidTypeRequest { .. })
        ));
        let mut request_group = fixture.clone();
        request_group.input.type_requests.swap(1, 2);
        assert!(matches!(
            build(&request_group),
            Err(SourceFunctorApplicationError::InvalidTypeRequest { .. })
        ));
        let mut argument_ordinal = fixture.clone();
        argument_ordinal.input.arguments[1].ordinal = 0;
        assert!(matches!(
            build(&argument_ordinal),
            Err(SourceFunctorApplicationError::ReorderedArgument { .. })
        ));
        let mut argument_order = fixture.clone();
        argument_order.input.arguments.swap(0, 1);
        argument_order.input.arguments[0].ordinal = 0;
        argument_order.input.arguments[1].ordinal = 1;
        assert!(matches!(
            build(&argument_order),
            Err(SourceFunctorApplicationError::OverlappingArguments { .. })
        ));
        let mut applications = fixture;
        applications.input.applications.swap(0, 1);
        assert!(matches!(
            build(&applications),
            Err(SourceFunctorApplicationError::InvalidApplication { .. })
                | Err(SourceFunctorApplicationError::ReorderedApplication { .. })
        ));
    }
}
